//! Decommission grace periods (#291).
//!
//! Each contract runs the same suite through a small adapter, because
//! `issuer-registry` binds lifecycle calls to single-use proposal ids while the
//! other two contracts do not. The guarantees under test are documented in
//! `docs/decommission-grace.md`:
//!
//! * grace starts only with a nominated successor and an in-range window;
//! * during grace every entrypoint a finalized contract rejects is rejected,
//!   the window is fixed, and reads keep working;
//! * finalization is admin-only and allowed at `grace_ends_ledger`, not before;
//! * nothing reopens a finalized contract, and replays are rejected.

use core::cell::Cell;

use earnproof_shared::{
    ContractError, DecommissionPhase, DecommissionState, DECOMMISSION_STATE_VERSION,
    MAX_DECOMMISSION_GRACE_LEDGERS,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{
    contracttype,
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env,
};

type Outcome<T> = Result<T, Option<ContractError>>;

fn flatten<T, C, E>(result: Result<Result<T, C>, Result<ContractError, E>>) -> Outcome<T> {
    match result {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) => Err(None),
        Err(Ok(error)) => Err(Some(error)),
        Err(Err(_)) => Err(None),
    }
}

/// Local mirror of the contracts' pending layout key, used to model a layout
/// migration that is still running when grace is requested.
#[contracttype]
enum LayoutKey {
    StorageLayoutMigration,
}

trait GraceAdapter {
    fn id(&self) -> Address;
    fn nominate_successor(&self, successor: &Address) -> bool;
    fn activate_successor(&self) -> bool;
    fn begin(&self, grace_ledgers: u32) -> Outcome<DecommissionState>;
    fn finalize(&self) -> Outcome<DecommissionState>;
    fn state(&self) -> DecommissionState;
    fn is_decommissioned(&self) -> bool;
    fn successor(&self) -> Option<Address>;
    fn set_domain(&self, domain: &BytesN<32>) -> Outcome<()>;
    fn begin_layout(&self, target: u32) -> Outcome<()>;
}

struct Issuer {
    client: IssuerRegistryContractClient<'static>,
    env: Env,
    next_proposal: Cell<u8>,
}

impl Issuer {
    fn proposal(&self) -> BytesN<32> {
        let value = self.next_proposal.get();
        self.next_proposal.set(value + 1);
        BytesN::from_array(&self.env, &[value; 32])
    }
}

impl GraceAdapter for Issuer {
    fn id(&self) -> Address {
        self.client.address.clone()
    }
    fn nominate_successor(&self, successor: &Address) -> bool {
        self.client
            .try_nominate_successor(&self.proposal(), successor)
            .is_ok()
    }
    fn activate_successor(&self) -> bool {
        self.client.try_activate_successor(&self.proposal()).is_ok()
    }
    fn begin(&self, grace_ledgers: u32) -> Outcome<DecommissionState> {
        flatten(
            self.client
                .try_begin_decommission_grace(&self.proposal(), &grace_ledgers),
        )
    }
    fn finalize(&self) -> Outcome<DecommissionState> {
        flatten(self.client.try_finalize_decommission(&self.proposal()))
    }
    fn state(&self) -> DecommissionState {
        self.client.get_decommission_state()
    }
    fn is_decommissioned(&self) -> bool {
        self.client.is_decommissioned()
    }
    fn successor(&self) -> Option<Address> {
        self.client.get_successor()
    }
    fn set_domain(&self, domain: &BytesN<32>) -> Outcome<()> {
        flatten(self.client.try_set_deployment_domain(domain)).map(|_| ())
    }
    fn begin_layout(&self, target: u32) -> Outcome<()> {
        flatten(self.client.try_begin_layout_migration(&target)).map(|_| ())
    }
}

macro_rules! plain_adapter {
    ($name:ident, $client:ident) => {
        struct $name {
            client: $client<'static>,
        }

        impl GraceAdapter for $name {
            fn id(&self) -> Address {
                self.client.address.clone()
            }
            fn nominate_successor(&self, successor: &Address) -> bool {
                self.client.try_nominate_successor(successor).is_ok()
            }
            fn activate_successor(&self) -> bool {
                self.client.try_activate_successor().is_ok()
            }
            fn begin(&self, grace_ledgers: u32) -> Outcome<DecommissionState> {
                flatten(self.client.try_begin_decommission_grace(&grace_ledgers))
            }
            fn finalize(&self) -> Outcome<DecommissionState> {
                flatten(self.client.try_finalize_decommission())
            }
            fn state(&self) -> DecommissionState {
                self.client.get_decommission_state()
            }
            fn is_decommissioned(&self) -> bool {
                self.client.is_decommissioned()
            }
            fn successor(&self) -> Option<Address> {
                self.client.get_successor()
            }
            fn set_domain(&self, domain: &BytesN<32>) -> Outcome<()> {
                flatten(self.client.try_set_deployment_domain(domain)).map(|_| ())
            }
            fn begin_layout(&self, target: u32) -> Outcome<()> {
                flatten(self.client.try_begin_layout_migration(&target)).map(|_| ())
            }
        }
    };
}

plain_adapter!(Config, ProtocolConfigContractClient);
plain_adapter!(Proofs, ProofRegistryContractClient);

const START: u32 = 1_000;
const GRACE: u32 = 100;

fn env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger()
        .with_mut(|ledger| ledger.sequence_number = START);
    env
}

fn issuer(env: &Env) -> Box<dyn GraceAdapter> {
    let id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(env, &id);
    client.initialize(&Address::generate(env));
    Box::new(Issuer {
        client,
        env: env.clone(),
        next_proposal: Cell::new(1),
    })
}

fn config(env: &Env) -> Box<dyn GraceAdapter> {
    let id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(env, &id);
    client.initialize(&Address::generate(env));
    Box::new(Config { client })
}

fn proofs(env: &Env) -> Box<dyn GraceAdapter> {
    let admin = Address::generate(env);
    let config_id = env.register(ProtocolConfigContract, ());
    ProtocolConfigContractClient::new(env, &config_id).initialize(&admin);
    let issuer_id = env.register(IssuerRegistryContract, ());
    IssuerRegistryContractClient::new(env, &issuer_id).initialize(&admin);
    let id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(env, &id);
    client.initialize(&admin, &issuer_id, &config_id);
    Box::new(Proofs { client })
}

fn all(env: &Env) -> [Box<dyn GraceAdapter>; 3] {
    [issuer(env), config(env), proofs(env)]
}

fn set_ledger(env: &Env, sequence: u32) {
    env.ledger()
        .with_mut(|ledger| ledger.sequence_number = sequence);
}

#[test]
fn active_contract_reports_active_phase() {
    let env = env();
    for contract in all(&env) {
        let state = contract.state();
        assert_eq!(state.state_version, DECOMMISSION_STATE_VERSION);
        assert_eq!(state.phase, DecommissionPhase::Active);
        assert_eq!(state.successor, None);
        assert_eq!(
            (
                state.grace_started_ledger,
                state.grace_ends_ledger,
                state.finalized_ledger
            ),
            (0, 0, 0)
        );
    }
}

#[test]
fn grace_requires_a_nominated_successor_and_an_in_range_window() {
    let env = env();
    for contract in all(&env) {
        assert_eq!(contract.begin(GRACE), Err(Some(ContractError::NotFound)));
        let successor = Address::generate(&env);
        assert!(contract.nominate_successor(&successor));
        for invalid in [0, MAX_DECOMMISSION_GRACE_LEDGERS + 1, u32::MAX] {
            assert_eq!(
                contract.begin(invalid),
                Err(Some(ContractError::InvalidTimingConfig))
            );
        }
        assert_eq!(contract.state().phase, DecommissionPhase::Active);

        let state = contract.begin(MAX_DECOMMISSION_GRACE_LEDGERS).unwrap();
        assert_eq!(
            state.grace_ends_ledger,
            START + MAX_DECOMMISSION_GRACE_LEDGERS
        );
    }
}

#[test]
fn grace_is_read_only_and_the_window_is_fixed() {
    let env = env();
    for contract in all(&env) {
        let successor = Address::generate(&env);
        assert!(contract.nominate_successor(&successor));
        let state = contract.begin(GRACE).unwrap();
        assert_eq!(state.phase, DecommissionPhase::Grace);
        assert_eq!(state.successor, Some(successor.clone()));
        assert_eq!(state.grace_started_ledger, START);
        assert_eq!(state.grace_ends_ledger, START + GRACE);
        assert_eq!(state.finalized_ledger, 0);
        assert_eq!(contract.state(), state);
        assert!(!contract.is_decommissioned());

        // Reads still work.
        assert_eq!(contract.successor(), Some(successor));

        // Mutations a finalized contract rejects are already rejected.
        assert!(!contract.nominate_successor(&Address::generate(&env)));
        assert!(!contract.activate_successor());
        assert_eq!(
            contract.set_domain(&BytesN::from_array(&env, &[7; 32])),
            Err(Some(ContractError::InvalidState))
        );
        assert_eq!(
            contract.begin_layout(2),
            Err(Some(ContractError::InvalidState))
        );

        // The window cannot be restarted, extended, or shortened.
        assert_eq!(
            contract.begin(GRACE * 2),
            Err(Some(ContractError::InvalidState))
        );
        assert_eq!(contract.begin(1), Err(Some(ContractError::InvalidState)));
        assert_eq!(contract.state(), state);
    }
}

#[test]
fn finalization_boundary_is_exact_and_terminal() {
    let env = env();
    let contracts = all(&env);
    for contract in contracts.iter() {
        assert!(contract.nominate_successor(&Address::generate(&env)));
        contract.begin(GRACE).unwrap();
    }

    set_ledger(&env, START + GRACE - 1);
    for contract in contracts.iter() {
        assert_eq!(
            contract.finalize(),
            Err(Some(ContractError::InvalidTimingConfig))
        );
        assert_eq!(contract.state().phase, DecommissionPhase::Grace);
    }

    set_ledger(&env, START + GRACE);
    for contract in contracts.iter() {
        let finalized = contract.finalize().unwrap();
        assert_eq!(finalized.phase, DecommissionPhase::Finalized);
        assert_eq!(finalized.finalized_ledger, START + GRACE);
        assert_eq!(finalized.grace_ends_ledger, START + GRACE);
        assert!(contract.is_decommissioned());
        assert_eq!(contract.state(), finalized);

        // Replays and attempts to reopen are rejected and change nothing.
        assert_eq!(contract.finalize(), Err(Some(ContractError::InvalidState)));
        assert_eq!(
            contract.begin(GRACE),
            Err(Some(ContractError::InvalidState))
        );
        assert!(!contract.nominate_successor(&Address::generate(&env)));
        assert!(!contract.activate_successor());
        assert_eq!(contract.state(), finalized);
    }

    // Still terminal long after the window.
    set_ledger(&env, START + GRACE + 10_000);
    for contract in contracts.iter() {
        assert_eq!(contract.state().phase, DecommissionPhase::Finalized);
        assert_eq!(contract.finalize(), Err(Some(ContractError::InvalidState)));
    }
}

#[test]
fn finalize_without_grace_is_rejected() {
    let env = env();
    for contract in all(&env) {
        assert_eq!(contract.finalize(), Err(Some(ContractError::InvalidState)));
        assert!(contract.nominate_successor(&Address::generate(&env)));
        assert_eq!(contract.finalize(), Err(Some(ContractError::InvalidState)));
        assert!(!contract.is_decommissioned());
    }
}

#[test]
fn legacy_immediate_decommission_reports_finalized_and_cannot_enter_grace() {
    let env = env();
    for contract in all(&env) {
        let successor = Address::generate(&env);
        assert!(contract.nominate_successor(&successor));
        assert!(contract.activate_successor());
        let state = contract.state();
        assert_eq!(state.phase, DecommissionPhase::Finalized);
        assert_eq!(state.successor, Some(successor));
        assert_eq!(
            (
                state.grace_started_ledger,
                state.grace_ends_ledger,
                state.finalized_ledger
            ),
            (0, 0, 0)
        );
        assert_eq!(
            contract.begin(GRACE),
            Err(Some(ContractError::InvalidState))
        );
        assert_eq!(contract.finalize(), Err(Some(ContractError::InvalidState)));
    }
}

#[test]
fn only_the_admin_can_begin_or_finalize() {
    let env = env();
    for contract in all(&env) {
        assert!(contract.nominate_successor(&Address::generate(&env)));
        env.set_auths(&[]);
        assert_eq!(contract.begin(GRACE), Err(None));
        env.mock_all_auths();
        assert_eq!(contract.state().phase, DecommissionPhase::Active);

        contract.begin(GRACE).unwrap();
        set_ledger(&env, START + GRACE);
        env.set_auths(&[]);
        assert_eq!(contract.finalize(), Err(None));
        env.mock_all_auths();
        assert_eq!(contract.state().phase, DecommissionPhase::Grace);
        assert!(!contract.is_decommissioned());
        contract.finalize().unwrap();
        set_ledger(&env, START);
    }
}

#[test]
fn grace_cannot_start_while_a_layout_migration_is_pending() {
    let env = env();
    for contract in all(&env) {
        assert!(contract.nominate_successor(&Address::generate(&env)));
        let id = contract.id();
        env.as_contract(&id, || {
            env.storage()
                .instance()
                .set(&LayoutKey::StorageLayoutMigration, &2_u32)
        });
        assert_eq!(
            contract.begin(GRACE),
            Err(Some(ContractError::InvalidState))
        );
        assert_eq!(contract.state().phase, DecommissionPhase::Active);
        env.as_contract(&id, || {
            env.storage()
                .instance()
                .remove(&LayoutKey::StorageLayoutMigration)
        });
        assert!(contract.begin(GRACE).is_ok());
    }
}

#[test]
fn issuer_registry_grace_consumes_single_use_proposals() {
    let env = env();
    let id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(&env, &id);
    client.initialize(&Address::generate(&env));
    let successor = Address::generate(&env);
    client.nominate_successor(&BytesN::from_array(&env, &[1; 32]), &successor);

    let zero = BytesN::from_array(&env, &[0; 32]);
    assert_eq!(
        client.try_begin_decommission_grace(&zero, &GRACE),
        Err(Ok(ContractError::InvalidInput))
    );
    // A proposal id already used for the nomination cannot be replayed.
    let used = BytesN::from_array(&env, &[1; 32]);
    assert_eq!(
        client.try_begin_decommission_grace(&used, &GRACE),
        Err(Ok(ContractError::AlreadyExists))
    );
    let begin = BytesN::from_array(&env, &[2; 32]);
    client.begin_decommission_grace(&begin, &GRACE);
    set_ledger(&env, START + GRACE);
    assert_eq!(
        client.try_finalize_decommission(&begin),
        Err(Ok(ContractError::AlreadyExists))
    );
    client.finalize_decommission(&BytesN::from_array(&env, &[3; 32]));
    assert!(client.is_decommissioned());
}

#[test]
fn registrations_are_rejected_during_proof_registry_grace() {
    use earnproof_shared::ProofError;
    let env = env();
    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    ProtocolConfigContractClient::new(&env, &config_id).initialize(&admin);
    let issuer_id = env.register(IssuerRegistryContract, ());
    IssuerRegistryContractClient::new(&env, &issuer_id).initialize(&admin);
    let id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(&env, &id);
    client.initialize(&admin, &issuer_id, &config_id);
    client.nominate_successor(&Address::generate(&env));
    client.begin_decommission_grace(&GRACE);

    let hash = BytesN::from_array(&env, &[9; 32]);
    assert_eq!(
        client.try_register_proof(
            &hash,
            &hash,
            &Address::generate(&env),
            &1,
            &u64::MAX,
            &None,
            &hash,
        ),
        Err(Ok(ProofError::ProofNotFound))
    );
}

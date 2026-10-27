//! Storage layout versions (#297): layouts advance only through the reviewed
//! begin/complete pair, a started migration never reports completion, and
//! failure, replay, authorization, and concurrency paths leave state unchanged.

use earnproof_shared::{
    ContractError, StorageLayoutCompatibility, ISSUER_REGISTRY_STORAGE_LAYOUT,
    LEGACY_STORAGE_LAYOUT, PROOF_REGISTRY_STORAGE_LAYOUT, PROTOCOL_CONFIG_STORAGE_LAYOUT,
    STORAGE_LAYOUT_STATUS_VERSION,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{contracttype, testutils::Address as _, Address, Env};

/// Local mirror of the contracts' instance key; used only to simulate a
/// legacy deployment (key absent) or a newer stored layout.
#[contracttype]
enum LayoutKey {
    StorageLayoutVersion,
}

fn make_legacy(env: &Env, id: &Address) {
    env.as_contract(id, || {
        env.storage()
            .instance()
            .remove(&LayoutKey::StorageLayoutVersion)
    });
}

fn set_active_layout(env: &Env, id: &Address, layout: u32) {
    env.as_contract(id, || {
        env.storage()
            .instance()
            .set(&LayoutKey::StorageLayoutVersion, &layout)
    });
}

fn issuer_registry(env: &Env) -> (IssuerRegistryContractClient<'static>, Address) {
    let id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(env, &id);
    client.initialize(&Address::generate(env));
    (client, id)
}

fn protocol_config(env: &Env) -> (ProtocolConfigContractClient<'static>, Address) {
    let id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(env, &id);
    client.initialize(&Address::generate(env));
    (client, id)
}

fn proof_registry(env: &Env) -> (ProofRegistryContractClient<'static>, Address) {
    let (_, issuer_id) = issuer_registry(env);
    let (_, config_id) = protocol_config(env);
    let id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(env, &id);
    client.initialize(&Address::generate(env), &issuer_id, &config_id);
    (client, id)
}

macro_rules! layout_suite {
    ($module:ident, $factory:ident, $client:ident, $code_layout:expr) => {
        mod $module {
            use super::*;

            fn layout_client(env: &Env, id: &Address) -> Box<dyn LayoutClient> {
                Box::new($client::new(env, id))
            }

            fn setup() -> (Env, Address) {
                let env = Env::default();
                env.mock_all_auths();
                let (_, id) = $factory(&env);
                (env, id)
            }

            #[test]
            fn fresh_deployment_reports_the_current_layout() {
                let (env, id) = setup();
                let status = layout_client(&env, &id).get_storage_layout();
                assert_eq!(status.status_version, STORAGE_LAYOUT_STATUS_VERSION);
                assert_eq!(status.active_layout, $code_layout);
                assert_eq!(status.code_layout, $code_layout);
                assert_eq!(status.migration_target, 0);
                assert_eq!(status.compatibility, StorageLayoutCompatibility::Current);
                assert_eq!(
                    layout_client(&env, &id).try_begin_layout_migration(&$code_layout),
                    Err(Ok(ContractError::InvalidState))
                );
            }

            #[test]
            fn legacy_layout_advances_only_through_begin_and_complete() {
                let (env, id) = setup();
                make_legacy(&env, &id);
                let c = layout_client(&env, &id);
                let status = c.get_storage_layout();
                assert_eq!(status.active_layout, LEGACY_STORAGE_LAYOUT);
                assert_eq!(status.migration_target, $code_layout);
                assert_eq!(
                    status.compatibility,
                    StorageLayoutCompatibility::MigrationRequired
                );

                // Completion without a started step is rejected.
                assert_eq!(
                    c.try_complete_layout_migration(&$code_layout),
                    Err(Ok(ContractError::InvalidState))
                );

                let started = c.begin_layout_migration(&$code_layout);
                assert_eq!(
                    started.compatibility,
                    StorageLayoutCompatibility::MigrationInProgress
                );
                // A started step never reports the target as active.
                assert_eq!(started.active_layout, LEGACY_STORAGE_LAYOUT);
                assert_eq!(started.migration_target, $code_layout);

                // Replay of the same begin is a no-op; a different target is not.
                assert_eq!(c.begin_layout_migration(&$code_layout), started);
                assert_eq!(
                    c.try_begin_layout_migration(&($code_layout + 1)),
                    Err(Ok(ContractError::InvalidState))
                );
                assert_eq!(
                    c.try_complete_layout_migration(&($code_layout + 1)),
                    Err(Ok(ContractError::InvalidState))
                );
                assert_eq!(c.get_storage_layout(), started);

                let done = c.complete_layout_migration(&$code_layout);
                assert_eq!(done.active_layout, $code_layout);
                assert_eq!(done.migration_target, 0);
                assert_eq!(done.compatibility, StorageLayoutCompatibility::Current);

                // Replays after completion are rejected and change nothing.
                assert_eq!(
                    c.try_complete_layout_migration(&$code_layout),
                    Err(Ok(ContractError::InvalidState))
                );
                assert_eq!(
                    c.try_begin_layout_migration(&$code_layout),
                    Err(Ok(ContractError::InvalidState))
                );
                assert_eq!(c.get_storage_layout(), done);
            }

            #[test]
            fn unreviewed_targets_are_rejected_at_the_boundary() {
                let (env, id) = setup();
                make_legacy(&env, &id);
                let c = layout_client(&env, &id);
                let before = c.get_storage_layout();
                for target in [0, LEGACY_STORAGE_LAYOUT, $code_layout + 1, u32::MAX] {
                    assert_eq!(
                        c.try_begin_layout_migration(&target),
                        Err(Ok(ContractError::InvalidInput))
                    );
                }
                assert_eq!(c.get_storage_layout(), before);
            }

            #[test]
            fn newer_stored_layout_is_unsupported_and_frozen() {
                let (env, id) = setup();
                set_active_layout(&env, &id, $code_layout + 1);
                let c = layout_client(&env, &id);
                let status = c.get_storage_layout();
                assert_eq!(
                    status.compatibility,
                    StorageLayoutCompatibility::Unsupported
                );
                assert_eq!(status.migration_target, 0);
                assert_eq!(
                    c.try_begin_layout_migration(&($code_layout + 1)),
                    Err(Ok(ContractError::InvalidState))
                );
            }

            #[test]
            fn only_the_admin_can_start_or_complete_a_step() {
                let (env, id) = setup();
                make_legacy(&env, &id);
                let c = layout_client(&env, &id);
                let before = c.get_storage_layout();
                env.set_auths(&[]);
                assert!(c.try_begin_layout_migration(&$code_layout).is_err());
                assert_eq!(c.get_storage_layout(), before);

                env.mock_all_auths();
                c.begin_layout_migration(&$code_layout);
                let started = c.get_storage_layout();
                env.set_auths(&[]);
                assert!(c.try_complete_layout_migration(&$code_layout).is_err());
                assert_eq!(c.get_storage_layout(), started);
            }

            #[test]
            fn data_migration_in_progress_blocks_starting_a_layout_step() {
                let (env, id) = setup();
                make_legacy(&env, &id);
                let c = layout_client(&env, &id);
                c.begin_data_migration(2, 5);
                assert_eq!(
                    c.try_begin_layout_migration(&$code_layout),
                    Err(Ok(ContractError::InvalidState))
                );
                c.advance_data_migration(0, 5);
                assert_eq!(
                    c.begin_layout_migration(&$code_layout).compatibility,
                    StorageLayoutCompatibility::MigrationInProgress
                );
            }

            #[test]
            fn data_migration_in_progress_blocks_completing_a_layout_step() {
                let (env, id) = setup();
                make_legacy(&env, &id);
                let c = layout_client(&env, &id);
                let started = c.begin_layout_migration(&$code_layout);
                c.begin_data_migration(2, 2);
                assert_eq!(
                    c.try_complete_layout_migration(&$code_layout),
                    Err(Ok(ContractError::InvalidState))
                );
                assert_eq!(c.get_storage_layout(), started);
                c.advance_data_migration(0, 2);
                assert_eq!(
                    c.complete_layout_migration(&$code_layout).compatibility,
                    StorageLayoutCompatibility::Current
                );
            }
        }
    };
}

/// Uniform view over the three generated clients.
trait LayoutClient {
    fn get_storage_layout(&self) -> earnproof_shared::StorageLayoutStatus;
    fn begin_layout_migration(&self, target: &u32) -> earnproof_shared::StorageLayoutStatus;
    fn complete_layout_migration(&self, target: &u32) -> earnproof_shared::StorageLayoutStatus;
    fn try_begin_layout_migration(
        &self,
        target: &u32,
    ) -> Result<earnproof_shared::StorageLayoutStatus, Result<ContractError, ()>>;
    fn try_complete_layout_migration(
        &self,
        target: &u32,
    ) -> Result<earnproof_shared::StorageLayoutStatus, Result<ContractError, ()>>;
    fn begin_data_migration(&self, target_contract_version: u32, total_items: u32);
    fn advance_data_migration(&self, cursor: u32, processed: u32);
}

macro_rules! impl_layout_client {
    ($client:ident) => {
        impl LayoutClient for $client<'static> {
            fn get_storage_layout(&self) -> earnproof_shared::StorageLayoutStatus {
                $client::get_storage_layout(self)
            }
            fn begin_layout_migration(
                &self,
                target: &u32,
            ) -> earnproof_shared::StorageLayoutStatus {
                $client::begin_layout_migration(self, target)
            }
            fn complete_layout_migration(
                &self,
                target: &u32,
            ) -> earnproof_shared::StorageLayoutStatus {
                $client::complete_layout_migration(self, target)
            }
            fn try_begin_layout_migration(
                &self,
                target: &u32,
            ) -> Result<earnproof_shared::StorageLayoutStatus, Result<ContractError, ()>> {
                match $client::try_begin_layout_migration(self, target) {
                    Ok(value) => Ok(value.unwrap()),
                    Err(Ok(error)) => Err(Ok(error)),
                    Err(Err(_)) => Err(Err(())),
                }
            }
            fn try_complete_layout_migration(
                &self,
                target: &u32,
            ) -> Result<earnproof_shared::StorageLayoutStatus, Result<ContractError, ()>> {
                match $client::try_complete_layout_migration(self, target) {
                    Ok(value) => Ok(value.unwrap()),
                    Err(Ok(error)) => Err(Ok(error)),
                    Err(Err(_)) => Err(Err(())),
                }
            }
            fn begin_data_migration(&self, target_contract_version: u32, total_items: u32) {
                $client::begin_migration(self, &target_contract_version, &total_items);
            }
            fn advance_data_migration(&self, cursor: u32, processed: u32) {
                $client::advance_migration(self, &cursor, &processed);
            }
        }
    };
}

impl_layout_client!(IssuerRegistryContractClient);
impl_layout_client!(ProtocolConfigContractClient);
impl_layout_client!(ProofRegistryContractClient);

layout_suite!(
    issuer_registry_layout,
    issuer_registry,
    IssuerRegistryContractClient,
    ISSUER_REGISTRY_STORAGE_LAYOUT
);
layout_suite!(
    protocol_config_layout,
    protocol_config,
    ProtocolConfigContractClient,
    PROTOCOL_CONFIG_STORAGE_LAYOUT
);
layout_suite!(
    proof_registry_layout,
    proof_registry,
    ProofRegistryContractClient,
    PROOF_REGISTRY_STORAGE_LAYOUT
);

//! Table-driven negative matrix over every mutating public entry point.
//!
//! Each row names one mutating function, the fixture it needs, and the code
//! that attempts it. The same row is replayed under three identities:
//!
//! * **Missing** — no authorization entry is provided at all.
//! * **Wrong** — an entry is provided but signed by an unrelated address (for
//!   `proof-registry` rows, a *different active issuer* where that is the
//!   realistic threat).
//! * **Authorized** — the documented authority signs. This is the control
//!   that makes the negative verdicts attributable to authorization rather
//!   than to a broken precondition.
//!
//! Missing and wrong identities must be rejected *and* leave the deployment
//! byte-for-byte unchanged (storage, TTLs, events, cross-contract state). The
//! authorized attempt must succeed. Read-only entry points are asserted
//! separately in [`reads_do_not_require_authorization`].
//!
//! The table is the executable form of `docs/authorization-matrix.md`, and
//! [`matrix_covers_every_mutating_public_function`] fails when the count
//! drifts, so the matrix stays synchronized with the generated client surface.

use crate::harness::{authorize, hash, issuer_id_hash, Deployment, APPROVED_SCHEMA};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, IntoVal, Val};

/// The identity attempting the call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Identity {
    /// No authorization entry at all.
    Missing,
    /// An authorization entry signed by an address that is not the documented
    /// authority for this entry point.
    Wrong,
    /// The documented authority signs (control case).
    Authorized,
}

/// One row of the authorization matrix.
struct Case {
    /// Fully-qualified entry point, matching the table in
    /// `docs/authorization-matrix.md`.
    name: &'static str,
    /// Whether the row runs against an uninitialized deployment (the three
    /// `initialize` rows).
    uninitialized: bool,
    /// Fixture state that must exist before the entry point can be exercised,
    /// built through authorized calls.
    setup: fn(&Deployment),
    /// Attempts the entry point. Returns whether the invocation was accepted.
    call: fn(&Deployment, Identity) -> bool,
}

/// Default [`Case::setup`] for rows that need no extra fixture.
fn no_setup(_: &Deployment) {}

/// [`Case::setup`] for rows that need a registered proof.
fn register_fixture_proof(d: &Deployment) {
    d.register_proof(FIXTURE_PROOF);
}

/// Discriminator of the proof created by [`register_fixture_proof`].
const FIXTURE_PROOF: u8 = 0x11;

/// Runs one entry point as each identity. `wrong` signs for the `Wrong` row and
/// `right` for the `Authorized` row; `Missing` installs no authorization.
#[allow(clippy::too_many_arguments)]
fn run_as(
    d: &Deployment,
    identity: Identity,
    wrong: &Address,
    right: &Address,
    contract: &Address,
    fn_name: &str,
    args: soroban_sdk::Vec<Val>,
    invoke: &dyn Fn() -> bool,
) -> bool {
    match identity {
        Identity::Missing => invoke(),
        Identity::Wrong => {
            authorize(&d.env, wrong, contract, fn_name, args);
            invoke()
        }
        Identity::Authorized => {
            authorize(&d.env, right, contract, fn_name, args);
            invoke()
        }
    }
}

/// [`Case::setup`] for rows that need a delayed proof that is already eligible.
fn register_eligible_delayed_proof(d: &Deployment) {
    let proof_id = hash(&d.env, FIXTURE_PROOF);
    let commitment = hash(&d.env, FIXTURE_PROOF ^ 0xFF);
    let now = d.env.ledger().timestamp();
    let (expires_at, activates_at) = (now + 100_000, now + 10);
    authorize(
        &d.env,
        &d.issuer,
        &d.proofs_address,
        "register_proof_with_activation",
        (
            &proof_id,
            &commitment,
            &d.issuer,
            &APPROVED_SCHEMA,
            &expires_at,
            &activates_at,
        )
            .into_val(&d.env),
    );
    d.proofs.register_proof_with_activation(
        &proof_id,
        &commitment,
        &d.issuer,
        &APPROVED_SCHEMA,
        &expires_at,
        &activates_at,
    );
    d.env.ledger().set_timestamp(activates_at);
}

/// [`Case::setup`] for rows that need an open dispute and a registered
/// resolver. The second issuer is the resolver and the admin is the disputant,
/// so the resolver has no conflict with the proof.
fn open_dispute_with_resolver(d: &Deployment) {
    register_fixture_proof(d);
    let proof_id = hash(&d.env, FIXTURE_PROOF);
    let evidence = hash(&d.env, 0x31);
    authorize(
        &d.env,
        &d.admin,
        &d.proofs_address,
        "open_dispute",
        (&proof_id, &d.admin, &evidence).into_val(&d.env),
    );
    d.proofs.open_dispute(&proof_id, &d.admin, &evidence);
    let commitment = hash(&d.env, 0x32);
    authorize(
        &d.env,
        &d.admin,
        &d.proofs_address,
        "register_dispute_resolver",
        (&d.second_issuer, &commitment).into_val(&d.env),
    );
    d.proofs
        .register_dispute_resolver(&d.second_issuer, &commitment);
}

/// [`Case::setup`] for rows that need an approved commitment hash algorithm.
fn approve_fixture_algorithm(d: &Deployment) {
    let id = 7_u32;
    authorize(
        &d.env,
        &d.admin,
        &d.config_address,
        "approve_hash_algorithm",
        (&id,).into_val(&d.env),
    );
    d.config.approve_hash_algorithm(&id);
}

/// Deploys the deployment a row runs against.
fn deployment_for(case: &Case) -> Deployment<'_> {
    if case.uninitialized {
        Deployment::uninitialized()
    } else {
        Deployment::new()
    }
}

/// Every mutating public entry point across the three contracts.
///
/// A new mutating function without a row here is a documentation gap:
/// [`matrix_covers_every_mutating_public_function`] fails when the count
/// drifts. `docs/authorization-matrix.md` must change together with this table.
fn matrix() -> std::vec::Vec<Case> {
    std::vec![
        // -----------------------------------------------------------------
        // protocol-config
        // -----------------------------------------------------------------
        Case {
            name: "protocol-config::initialize",
            uninitialized: true,
            setup: no_setup,
            call: |d, identity| {
                let args: soroban_sdk::Vec<Val> = (&d.admin,).into_val(&d.env);
                match identity {
                    Identity::Missing => d.config.try_initialize(&d.admin).is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "initialize",
                            args.clone(),
                        );
                        d.config.try_initialize(&d.admin).is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.config_address, "initialize", args);
                        d.config.try_initialize(&d.admin).is_ok()
                    }
                }
            },
        },
        Case {
            name: "protocol-config::set_admin",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let next = Address::generate(&d.env);
                let args: soroban_sdk::Vec<Val> = (&next,).into_val(&d.env);
                match identity {
                    Identity::Missing => {
                        let r = d.config.try_nominate_admin(&next);
                        if r.is_ok() {
                            let _ = d.config.try_accept_admin();
                        }
                        r
                    }
                    .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "nominate_admin",
                            args.clone(),
                        );
                        {
                            let r = d.config.try_nominate_admin(&next);
                            if r.is_ok() {
                                authorize(
                                    &d.env,
                                    &next,
                                    &d.config_address,
                                    "accept_admin",
                                    ().into_val(&d.env),
                                );
                                let _ = d.config.try_accept_admin();
                            }
                            r
                        }
                        .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.config_address, "nominate_admin", args);
                        {
                            let r = d.config.try_nominate_admin(&next);
                            if r.is_ok() {
                                authorize(
                                    &d.env,
                                    &next,
                                    &d.config_address,
                                    "accept_admin",
                                    ().into_val(&d.env),
                                );
                                let _ = d.config.try_accept_admin();
                            }
                            r
                        }
                        .is_ok()
                    }
                }
            },
        },
        Case {
            name: "protocol-config::pause",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let args: soroban_sdk::Vec<Val> = ().into_val(&d.env);
                match identity {
                    Identity::Missing => d.config.try_pause().is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "pause",
                            args.clone(),
                        );
                        d.config.try_pause().is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.config_address, "pause", args);
                        d.config.try_pause().is_ok()
                    }
                }
            },
        },
        Case {
            name: "protocol-config::unpause",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let args: soroban_sdk::Vec<Val> = ().into_val(&d.env);
                match identity {
                    Identity::Missing => d.config.try_unpause().is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "unpause",
                            args.clone(),
                        );
                        d.config.try_unpause().is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.config_address, "unpause", args);
                        d.config.try_unpause().is_ok()
                    }
                }
            },
        },
        Case {
            name: "protocol-config::approve_schema_version",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let version = 7_u32;
                let args: soroban_sdk::Vec<Val> = (&version,).into_val(&d.env);
                match identity {
                    Identity::Missing => d.config.try_approve_schema_version(&version).is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "approve_schema_version",
                            args.clone(),
                        );
                        d.config.try_approve_schema_version(&version).is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.admin,
                            &d.config_address,
                            "approve_schema_version",
                            args,
                        );
                        d.config.try_approve_schema_version(&version).is_ok()
                    }
                }
            },
        },
        Case {
            name: "protocol-config::deprecate_schema_version",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let args: soroban_sdk::Vec<Val> = (&APPROVED_SCHEMA,).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .config
                        .try_deprecate_schema_version(&APPROVED_SCHEMA)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "deprecate_schema_version",
                            args.clone(),
                        );
                        d.config
                            .try_deprecate_schema_version(&APPROVED_SCHEMA)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.admin,
                            &d.config_address,
                            "deprecate_schema_version",
                            args,
                        );
                        d.config
                            .try_deprecate_schema_version(&APPROVED_SCHEMA)
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "protocol-config::set_history_retention",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let kind = earnproof_shared::HistoryKind::ConfigChange;
                let max_entries = 16_u32;
                let revision = 0_u32;
                let args: soroban_sdk::Vec<Val> = (&kind, &max_entries, &revision).into_val(&d.env);
                let attempt = || {
                    d.config
                        .try_set_history_retention(&kind, &max_entries, &revision)
                        .is_ok()
                };
                match identity {
                    Identity::Missing => attempt(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.config_address,
                            "set_history_retention",
                            args.clone(),
                        );
                        attempt()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.admin,
                            &d.config_address,
                            "set_history_retention",
                            args,
                        );
                        attempt()
                    }
                }
            },
        },
        // -----------------------------------------------------------------
        // issuer-registry
        // -----------------------------------------------------------------
        Case {
            name: "issuer-registry::initialize",
            uninitialized: true,
            setup: no_setup,
            call: |d, identity| {
                let args: soroban_sdk::Vec<Val> = (&d.admin,).into_val(&d.env);
                match identity {
                    Identity::Missing => d.issuers.try_initialize(&d.admin).is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "initialize",
                            args.clone(),
                        );
                        d.issuers.try_initialize(&d.admin).is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.issuers_address, "initialize", args);
                        d.issuers.try_initialize(&d.admin).is_ok()
                    }
                }
            },
        },
        Case {
            name: "issuer-registry::register_issuer",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let id = issuer_id_hash(&d.env, 0x70);
                let address = Address::generate(&d.env);
                let metadata = hash(&d.env, 0x71);
                let args: soroban_sdk::Vec<Val> =
                    (&id, &address, &metadata, &metadata).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .issuers
                        .try_register_issuer(&id, &address, &metadata, &metadata)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "register_issuer",
                            args.clone(),
                        );
                        d.issuers
                            .try_register_issuer(&id, &address, &metadata, &metadata)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.admin,
                            &d.issuers_address,
                            "register_issuer",
                            args,
                        );
                        d.issuers
                            .try_register_issuer(&id, &address, &metadata, &metadata)
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "issuer-registry::update_issuer",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let metadata = hash(&d.env, 0x72);
                let args: soroban_sdk::Vec<Val> = (&d.issuer_id, &metadata).into_val(&d.env);
                match identity {
                    Identity::Missing => {
                        d.issuers.try_update_issuer(&d.issuer_id, &metadata).is_ok()
                    }
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "update_issuer",
                            args.clone(),
                        );
                        d.issuers.try_update_issuer(&d.issuer_id, &metadata).is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.issuers_address, "update_issuer", args);
                        d.issuers.try_update_issuer(&d.issuer_id, &metadata).is_ok()
                    }
                }
            },
        },
        Case {
            name: "issuer-registry::suspend_issuer",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let pid = hash(&d.env, 0x81);
                let reason = soroban_sdk::BytesN::from_array(&d.env, &[1u8; 32]);
                let pid = soroban_sdk::BytesN::from_array(&d.env, &[0xA6u8; 32]);
                let args: soroban_sdk::Vec<Val> = (&pid, &d.issuer_id, &reason).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .issuers
                        .try_suspend_issuer(&pid, &d.issuer_id, &reason)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "suspend_issuer",
                            args.clone(),
                        );
                        d.issuers
                            .try_suspend_issuer(&pid, &d.issuer_id, &reason)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.issuers_address, "suspend_issuer", args);
                        d.issuers
                            .try_suspend_issuer(&pid, &d.issuer_id, &reason)
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "issuer-registry::reactivate_issuer",
            uninitialized: false,
            // The fixture issuer must be suspended first, or reactivation is
            // rejected by a state precondition rather than by authorization.
            setup: |d| d.suspend_issuer(&d.issuer_id),
            call: |d, identity| {
                let proposal = soroban_sdk::BytesN::from_array(&d.env, &[2u8; 32]);
                let reason = soroban_sdk::BytesN::from_array(&d.env, &[1u8; 32]);
                let pid = soroban_sdk::BytesN::from_array(&d.env, &[0xA6u8; 32]);
                let args: soroban_sdk::Vec<Val> = (&pid, &d.issuer_id, &reason).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .issuers
                        .try_reactivate_issuer(&pid, &d.issuer_id, &reason)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "reactivate_issuer",
                            args.clone(),
                        );
                        d.issuers
                            .try_reactivate_issuer(&pid, &d.issuer_id, &reason)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.admin,
                            &d.issuers_address,
                            "reactivate_issuer",
                            args,
                        );
                        d.issuers
                            .try_reactivate_issuer(&pid, &d.issuer_id, &reason)
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "issuer-registry::revoke_issuer",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let proposal = soroban_sdk::BytesN::from_array(&d.env, &[3u8; 32]);
                let reason = soroban_sdk::BytesN::from_array(&d.env, &[1u8; 32]);
                let pid = soroban_sdk::BytesN::from_array(&d.env, &[0xA6u8; 32]);
                let args: soroban_sdk::Vec<Val> = (&pid, &d.issuer_id, &reason).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .issuers
                        .try_revoke_issuer(&pid, &d.issuer_id, &reason)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "revoke_issuer",
                            args.clone(),
                        );
                        d.issuers
                            .try_revoke_issuer(&pid, &d.issuer_id, &reason)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.issuers_address, "revoke_issuer", args);
                        d.issuers
                            .try_revoke_issuer(&pid, &d.issuer_id, &reason)
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "issuer-registry::rotate_issuer_address",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let next = Address::generate(&d.env);
                let args: soroban_sdk::Vec<Val> = (&d.issuer_id, &next).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .issuers
                        .try_rotate_issuer_address(&d.issuer_id, &next)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.issuers_address,
                            "rotate_issuer_address",
                            args.clone(),
                        );
                        d.issuers
                            .try_rotate_issuer_address(&d.issuer_id, &next)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.issuer,
                            &d.issuers_address,
                            "rotate_issuer_address",
                            args,
                        );
                        d.issuers
                            .try_rotate_issuer_address(&d.issuer_id, &next)
                            .is_ok()
                    }
                }
            },
        },
        // -----------------------------------------------------------------
        // proof-registry
        // -----------------------------------------------------------------
        Case {
            name: "proof-registry::initialize",
            uninitialized: true,
            setup: no_setup,
            call: |d, identity| {
                let args: soroban_sdk::Vec<Val> =
                    (&d.admin, &d.issuers_address, &d.config_address).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .proofs
                        .try_initialize(&d.admin, &d.issuers_address, &d.config_address)
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.attacker(),
                            &d.proofs_address,
                            "initialize",
                            args.clone(),
                        );
                        d.proofs
                            .try_initialize(&d.admin, &d.issuers_address, &d.config_address)
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.admin, &d.proofs_address, "initialize", args);
                        d.proofs
                            .try_initialize(&d.admin, &d.issuers_address, &d.config_address)
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "proof-registry::register_proof",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let proof_id = hash(&d.env, 0x80);
                let commitment = hash(&d.env, 0x81);
                let expires_at = d.env.ledger().timestamp() + 100_000;
                let pt = soroban_sdk::BytesN::from_array(&d.env, &[1u8; 32]);
                let pred: Option<soroban_sdk::BytesN<32>> = None;
                let args: soroban_sdk::Vec<Val> = (
                    &proof_id,
                    &commitment,
                    &d.issuer,
                    &APPROVED_SCHEMA,
                    &expires_at,
                    &pred,
                    &pt,
                )
                    .into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .proofs
                        .try_register_proof_typed(
                            &proof_id,
                            &commitment,
                            &d.issuer,
                            &APPROVED_SCHEMA,
                            &expires_at,
                            &pred,
                            &pt,
                        )
                        .is_ok(),
                    // The realistic "wrong" signer is a *different active
                    // issuer*: someone who holds valid issuer credentials but
                    // is not the issuer named in the registration.
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.second_issuer,
                            &d.proofs_address,
                            "register_proof_typed",
                            args.clone(),
                        );
                        d.proofs
                            .try_register_proof_typed(
                                &proof_id,
                                &commitment,
                                &d.issuer,
                                &APPROVED_SCHEMA,
                                &expires_at,
                                &pred,
                                &pt,
                            )
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.issuer,
                            &d.proofs_address,
                            "register_proof_typed",
                            args,
                        );
                        d.proofs
                            .try_register_proof_typed(
                                &proof_id,
                                &commitment,
                                &d.issuer,
                                &APPROVED_SCHEMA,
                                &expires_at,
                                &pred,
                                &pt,
                            )
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "proof-registry::revoke_proof",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                let args: soroban_sdk::Vec<Val> = (&proof_id,).into_val(&d.env);
                match identity {
                    Identity::Missing => d.proofs.try_revoke_proof(&proof_id).is_ok(),
                    // A different active issuer must not be able to revoke a
                    // proof it does not own.
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.second_issuer,
                            &d.proofs_address,
                            "revoke_proof",
                            args.clone(),
                        );
                        d.proofs.try_revoke_proof(&proof_id).is_ok()
                    }
                    Identity::Authorized => {
                        authorize(&d.env, &d.issuer, &d.proofs_address, "revoke_proof", args);
                        d.proofs.try_revoke_proof(&proof_id).is_ok()
                    }
                }
            },
        },
        Case {
            name: "proof-registry::commit_disclosure_consent",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                let policy_hash = hash(&d.env, 0x12);
                let receipt_version = 1_u32;
                let receipt_hash = hash(&d.env, 0x13);
                let args: soroban_sdk::Vec<Val> =
                    (&proof_id, &policy_hash, &receipt_version, &receipt_hash).into_val(&d.env);
                match identity {
                    Identity::Missing => d
                        .proofs
                        .try_commit_disclosure_consent(
                            &proof_id,
                            &policy_hash,
                            &receipt_version,
                            &receipt_hash,
                        )
                        .is_ok(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.second_issuer,
                            &d.proofs_address,
                            "commit_disclosure_consent",
                            args.clone(),
                        );
                        d.proofs
                            .try_commit_disclosure_consent(
                                &proof_id,
                                &policy_hash,
                                &receipt_version,
                                &receipt_hash,
                            )
                            .is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.issuer,
                            &d.proofs_address,
                            "commit_disclosure_consent",
                            args,
                        );
                        d.proofs
                            .try_commit_disclosure_consent(
                                &proof_id,
                                &policy_hash,
                                &receipt_version,
                                &receipt_hash,
                            )
                            .is_ok()
                    }
                }
            },
        },
        Case {
            name: "proof-registry::withdraw_disclosure_consent",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                let policy_hash = hash(&d.env, 0x14);
                let sequence = 1_u64;
                let reason = 1_u32;
                let salt = hash(&d.env, 0x15);
                let args: soroban_sdk::Vec<Val> =
                    (&proof_id, &policy_hash, &sequence, &reason, &salt).into_val(&d.env);
                let attempt = || {
                    d.proofs
                        .try_withdraw_disclosure_consent(
                            &proof_id,
                            &policy_hash,
                            &sequence,
                            &reason,
                            &salt,
                        )
                        .is_ok()
                };
                match identity {
                    Identity::Missing => attempt(),
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.second_issuer,
                            &d.proofs_address,
                            "withdraw_disclosure_consent",
                            args.clone(),
                        );
                        attempt()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.issuer,
                            &d.proofs_address,
                            "withdraw_disclosure_consent",
                            args,
                        );
                        attempt()
                    }
                }
            },
        },
        Case {
            name: "proof-registry::admin_revoke_proof",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                let args: soroban_sdk::Vec<Val> = (&proof_id,).into_val(&d.env);
                match identity {
                    Identity::Missing => d.proofs.try_admin_revoke_proof(&proof_id).is_ok(),
                    // The proof's own issuer must not be able to use the admin
                    // path: the two revocation entry points demand different
                    // identities.
                    Identity::Wrong => {
                        authorize(
                            &d.env,
                            &d.issuer,
                            &d.proofs_address,
                            "admin_revoke_proof",
                            args.clone(),
                        );
                        d.proofs.try_admin_revoke_proof(&proof_id).is_ok()
                    }
                    Identity::Authorized => {
                        authorize(
                            &d.env,
                            &d.admin,
                            &d.proofs_address,
                            "admin_revoke_proof",
                            args,
                        );
                        d.proofs.try_admin_revoke_proof(&proof_id).is_ok()
                    }
                }
            },
        },
        Case {
            name: "proof-registry::set_authority_policy_version",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let version = 2_u32;
                run_as(
                    d,
                    identity,
                    &d.attacker(),
                    &d.admin,
                    &d.proofs_address,
                    "set_authority_policy_version",
                    (&version,).into_val(&d.env),
                    &|| d.proofs.try_set_authority_policy_version(&version).is_ok(),
                )
            },
        },
        Case {
            name: "proof-registry::revoke_proof_at_policy",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                let version = 1_u32;
                run_as(
                    d,
                    identity,
                    &d.second_issuer,
                    &d.issuer,
                    &d.proofs_address,
                    "revoke_proof_at_policy",
                    (&proof_id, &version).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_revoke_proof_at_policy(&proof_id, &version)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::admin_revoke_proof_at_policy",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                let version = 1_u32;
                run_as(
                    d,
                    identity,
                    &d.issuer,
                    &d.admin,
                    &d.proofs_address,
                    "admin_revoke_proof_at_policy",
                    (&proof_id, &version).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_admin_revoke_proof_at_policy(&proof_id, &version)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::set_activation_delegate",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let enabled = true;
                run_as(
                    d,
                    identity,
                    &d.second_issuer,
                    &d.issuer,
                    &d.proofs_address,
                    "set_activation_delegate",
                    (&d.issuer, &d.second_issuer, &enabled).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_set_activation_delegate(&d.issuer, &d.second_issuer, &enabled)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::acknowledge_activation",
            uninitialized: false,
            setup: register_eligible_delayed_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                run_as(
                    d,
                    identity,
                    &d.second_issuer,
                    &d.issuer,
                    &d.proofs_address,
                    "acknowledge_activation",
                    (&proof_id, &d.issuer).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_acknowledge_activation(&proof_id, &d.issuer)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::register_dispute_resolver",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let commitment = hash(&d.env, 0x32);
                run_as(
                    d,
                    identity,
                    &d.issuer,
                    &d.admin,
                    &d.proofs_address,
                    "register_dispute_resolver",
                    (&d.second_issuer, &commitment).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_register_dispute_resolver(&d.second_issuer, &commitment)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::remove_dispute_resolver",
            uninitialized: false,
            setup: open_dispute_with_resolver,
            call: |d, identity| {
                run_as(
                    d,
                    identity,
                    &d.issuer,
                    &d.admin,
                    &d.proofs_address,
                    "remove_dispute_resolver",
                    (&d.second_issuer,).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_remove_dispute_resolver(&d.second_issuer)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::bind_participant_commitment",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let commitment = hash(&d.env, 0x33);
                run_as(
                    d,
                    identity,
                    &d.issuer,
                    &d.second_issuer,
                    &d.proofs_address,
                    "bind_participant_commitment",
                    (&d.second_issuer, &commitment).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_bind_participant_commitment(&d.second_issuer, &commitment)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::authorize_subject_actor",
            uninitialized: false,
            setup: register_fixture_proof,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                run_as(
                    d,
                    identity,
                    &d.second_issuer,
                    &d.issuer,
                    &d.proofs_address,
                    "authorize_subject_actor",
                    (&proof_id, &d.second_issuer).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_authorize_subject_actor(&proof_id, &d.second_issuer)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::resolve_dispute_as",
            uninitialized: false,
            setup: open_dispute_with_resolver,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                run_as(
                    d,
                    identity,
                    &d.issuer,
                    &d.second_issuer,
                    &d.proofs_address,
                    "resolve_dispute_as",
                    (&proof_id, &d.second_issuer).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_resolve_dispute_as(&proof_id, &d.second_issuer)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "proof-registry::reject_dispute_as",
            uninitialized: false,
            setup: open_dispute_with_resolver,
            call: |d, identity| {
                let proof_id = hash(&d.env, FIXTURE_PROOF);
                run_as(
                    d,
                    identity,
                    &d.issuer,
                    &d.second_issuer,
                    &d.proofs_address,
                    "reject_dispute_as",
                    (&proof_id, &d.second_issuer).into_val(&d.env),
                    &|| {
                        d.proofs
                            .try_reject_dispute_as(&proof_id, &d.second_issuer)
                            .is_ok()
                    },
                )
            },
        },
        Case {
            name: "protocol-config::approve_hash_algorithm",
            uninitialized: false,
            setup: no_setup,
            call: |d, identity| {
                let id = 7_u32;
                run_as(
                    d,
                    identity,
                    &d.attacker(),
                    &d.admin,
                    &d.config_address,
                    "approve_hash_algorithm",
                    (&id,).into_val(&d.env),
                    &|| d.config.try_approve_hash_algorithm(&id).is_ok(),
                )
            },
        },
        Case {
            name: "protocol-config::activate_hash_algorithm",
            uninitialized: false,
            setup: approve_fixture_algorithm,
            call: |d, identity| {
                let id = 7_u32;
                run_as(
                    d,
                    identity,
                    &d.attacker(),
                    &d.admin,
                    &d.config_address,
                    "activate_hash_algorithm",
                    (&id,).into_val(&d.env),
                    &|| d.config.try_activate_hash_algorithm(&id).is_ok(),
                )
            },
        },
        Case {
            name: "protocol-config::deprecate_hash_algorithm",
            uninitialized: false,
            setup: approve_fixture_algorithm,
            call: |d, identity| {
                let id = 7_u32;
                run_as(
                    d,
                    identity,
                    &d.attacker(),
                    &d.admin,
                    &d.config_address,
                    "deprecate_hash_algorithm",
                    (&id,).into_val(&d.env),
                    &|| d.config.try_deprecate_hash_algorithm(&id).is_ok(),
                )
            },
        },
    ]
}
/// Guards against a mutating entry point being added without a documented
/// authorization expectation. Bump only together with
/// `docs/authorization-matrix.md`.
const DOCUMENTED_MUTATIONS: usize = 20;

#[test]
fn matrix_covers_every_mutating_public_function() {
    assert_eq!(
        matrix().len(),
        DOCUMENTED_MUTATIONS,
        "authorization matrix and docs/authorization-matrix.md disagree on the mutating surface"
    );
}

#[test]
fn every_mutation_rejects_a_missing_identity_without_side_effects() {
    for case in matrix() {
        let deployment = deployment_for(&case);
        (case.setup)(&deployment);
        let before = deployment.snapshot();

        let accepted = (case.call)(&deployment, Identity::Missing);

        assert!(
            !accepted,
            "{} was accepted with no authorization at all",
            case.name
        );
        deployment.assert_no_side_effects(&before, case.name);
    }
}

#[test]
fn every_mutation_rejects_a_wrong_identity_without_side_effects() {
    for case in matrix() {
        let deployment = deployment_for(&case);
        (case.setup)(&deployment);
        let before = deployment.snapshot();

        let accepted = (case.call)(&deployment, Identity::Wrong);

        assert!(
            !accepted,
            "{} was accepted when signed by the wrong identity",
            case.name
        );
        deployment.assert_no_side_effects(&before, case.name);
    }
}

#[test]
fn every_mutation_succeeds_for_the_documented_authority() {
    // The mirror of the matrix: no row may be "rejected" merely because its
    // precondition was already broken. Each case must succeed when the
    // documented authority signs, so the negative verdicts above are
    // attributable to authorization and nothing else.
    for case in matrix() {
        let deployment = deployment_for(&case);
        (case.setup)(&deployment);

        assert!(
            (case.call)(&deployment, Identity::Authorized),
            "{} failed for its documented authority; the negative verdicts above are not attributable",
            case.name
        );
    }
}

#[test]
fn read_only_entry_points_require_no_authorization() {
    // Reads are deliberately unauthenticated. If one of them were accidentally
    // gated, verifiers and indexers would break; if one of the mutations were
    // accidentally ungated, the matrix above would fail. This test pins the
    // read side of that boundary: every read works with zero auth entries.
    let deployment = Deployment::new();
    let proof_id = deployment.register_proof(0x90);

    assert!(deployment.config.try_get_admin().is_ok());
    assert!(deployment.config.try_is_paused().is_ok());
    assert!(deployment.config.try_get_config_version().is_ok());
    assert!(deployment
        .config
        .try_is_schema_version_approved(&APPROVED_SCHEMA)
        .is_ok());

    assert!(deployment.issuers.try_get_admin().is_ok());
    assert!(deployment
        .issuers
        .try_get_issuer(&deployment.issuer_id)
        .is_ok());
    assert!(deployment
        .issuers
        .try_get_issuer_by_address(&deployment.issuer)
        .is_ok());
    assert!(deployment
        .issuers
        .try_is_active_issuer(&deployment.issuer_id)
        .is_ok());
    assert!(deployment
        .issuers
        .try_is_active_address(&deployment.issuer)
        .is_ok());

    assert!(deployment.proofs.try_get_admin().is_ok());
    assert!(deployment.proofs.try_get_issuer_registry().is_ok());
    assert!(deployment.proofs.try_get_protocol_config().is_ok());
    assert!(deployment.proofs.try_get_proof(&proof_id).is_ok());
    assert!(deployment.proofs.try_is_valid_proof(&proof_id).is_ok());
    assert!(deployment.proofs.try_is_revoked(&proof_id).is_ok());
    assert!(deployment
        .proofs
        .try_has_consent_receipt_commitment(&hash(&deployment.env, 0x99))
        .is_ok());
}

#[test]
fn new_history_and_consent_rows_accept_only_their_documented_authority() {
    for case in matrix().into_iter().filter(|case| {
        case.name == "protocol-config::set_history_retention"
            || case.name == "proof-registry::withdraw_disclosure_consent"
    }) {
        for (identity, expected) in [
            (Identity::Missing, false),
            (Identity::Wrong, false),
            (Identity::Authorized, true),
        ] {
            let deployment = deployment_for(&case);
            (case.setup)(&deployment);
            assert_eq!(
                (case.call)(&deployment, identity),
                expected,
                "{}",
                case.name
            );
        }
    }
}

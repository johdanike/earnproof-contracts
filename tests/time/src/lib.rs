#![no_std]

//! Deterministic ledger-time boundary tests. All timestamps are explicit.
//!
//! Tests cover:
//! - Proof expiry boundaries (before, at, after)
//! - Schema deprecation time boundaries
//! - Timestamp overflow and zero values
//! - Revocation dominates expiry
//! - Record timestamp correctness (created_at, revoked_at)
//! - Cross-cutting interactions (multi-proof expiry, revoke-then-expire)

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use earnproof_shared::{
        DisputeFilingAnchor, DisputeFilingPolicy, DisputeFilingStatus, ProofError, ProofStatus,
        DISPUTE_FILING_POLICY_VERSION,
    };
    use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
    use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
    use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, BytesN, Env};

    const NOW: u64 = 1_000;

    // ── Reusable LedgerClock ────────────────────────────────────────────────

    /// Deterministic clock for controlling `env.ledger().timestamp()`.
    ///
    /// All time progression is explicit: call [`LedgerClock::advance`] or
    /// [`LedgerClock::set_to`] rather than reading wall-clock time. This makes
    /// every test fully reproducible.
    struct LedgerClock {
        env: Env,
    }

    #[allow(dead_code)]
    impl LedgerClock {
        /// Create a new clock anchored at `start_timestamp`.
        fn new(start_timestamp: u64) -> Self {
            let env = Env::default();
            env.mock_all_auths();
            env.ledger().set_timestamp(start_timestamp);
            Self { env }
        }

        /// Advance the clock by `seconds` from its current position.
        fn advance(&self, seconds: u64) {
            let now = self.env.ledger().timestamp();
            self.env.ledger().set_timestamp(now + seconds);
        }

        /// Set the clock to an absolute `timestamp`.
        fn set_to(&self, timestamp: u64) {
            self.env.ledger().set_timestamp(timestamp);
        }

        /// Return the current ledger timestamp.
        fn now(&self) -> u64 {
            self.env.ledger().timestamp()
        }
    }

    // ── Fixture ─────────────────────────────────────────────────────────────

    struct Fixture {
        clock: LedgerClock,
        proofs: ProofRegistryContractClient<'static>,
        config: ProtocolConfigContractClient<'static>,
        issuer: Address,
    }

    fn bytes(env: &Env, value: u8) -> BytesN<32> {
        BytesN::from_array(env, &[value; 32])
    }

    fn fixture() -> Fixture {
        let clock = LedgerClock::new(NOW);
        let admin = Address::generate(&clock.env);
        let issuer = Address::generate(&clock.env);
        let config_id = clock.env.register(ProtocolConfigContract, ());
        let config = ProtocolConfigContractClient::new(&clock.env, &config_id);
        config.initialize(&admin);
        config.approve_schema_version(&1);
        config.approve_proof_type(&soroban_sdk::BytesN::from_array(&clock.env, &[1u8; 32]));
        let issuers_id = clock.env.register(IssuerRegistryContract, ());
        let issuers = IssuerRegistryContractClient::new(&clock.env, &issuers_id);
        issuers.initialize(&admin);
        issuers.register_issuer(
            &bytes(&clock.env, 1),
            &issuer,
            &bytes(&clock.env, 2),
            &bytes(&clock.env, 99),
        );
        let proofs_id = clock.env.register(ProofRegistryContract, ());
        let proofs = ProofRegistryContractClient::new(&clock.env, &proofs_id);
        proofs.initialize(&admin, &issuers_id, &config_id);
        Fixture {
            clock,
            proofs,
            config,
            issuer,
        }
    }

    fn register(fixture: &Fixture, id: u8, expires_at: u64) {
        fixture.proofs.register_proof_typed(
            &bytes(&fixture.clock.env, id),
            &bytes(&fixture.clock.env, id.wrapping_add(10)),
            &fixture.issuer,
            &1,
            &expires_at,
            &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32]),
        );
    }

    // ── Existing tests (preserved) ──────────────────────────────────────────

    #[test]
    fn validity_is_inclusive_at_expiration_and_false_after() {
        let fixture = fixture();
        register(&fixture, 1, NOW + 10);
        fixture.clock.set_to(NOW + 10);
        assert!(fixture.proofs.is_valid_proof(&bytes(&fixture.clock.env, 1)));
        fixture.clock.set_to(NOW + 11);
        assert!(!fixture.proofs.is_valid_proof(&bytes(&fixture.clock.env, 1)));
    }

    #[test]
    fn registration_requires_strictly_future_expiration() {
        let fixture = fixture();
        for (id, expires_at) in [(1, NOW - 1), (2, NOW), (3, 0)] {
            assert_eq!(
                fixture.proofs.try_register_proof_typed(
                    &bytes(&fixture.clock.env, id),
                    &bytes(&fixture.clock.env, id + 10),
                    &fixture.issuer,
                    &1,
                    &expires_at,
                    &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
                ),
                Err(Ok(ProofError::ProofExpired))
            );
        }
    }

    #[test]
    fn revocation_dominates_expiration() {
        let fixture = fixture();
        register(&fixture, 4, NOW + 10);
        fixture.clock.set_to(NOW + 10);
        fixture.proofs.revoke_proof(&bytes(&fixture.clock.env, 4));
        fixture.clock.set_to(NOW + 11);
        assert!(!fixture.proofs.is_valid_proof(&bytes(&fixture.clock.env, 4)));
        assert!(fixture.proofs.is_revoked(&bytes(&fixture.clock.env, 4)));
    }

    #[test]
    fn zero_schema_and_pause_are_deterministic_guards() {
        let fixture = fixture();
        assert_eq!(
            fixture.proofs.try_register_proof_typed(
                &bytes(&fixture.clock.env, 5),
                &bytes(&fixture.clock.env, 6),
                &fixture.issuer,
                &0,
                &(NOW + 1),
                &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
            ),
            Err(Ok(ProofError::InvalidSchemaVersion))
        );
        fixture.config.pause();
        assert_eq!(
            fixture.proofs.try_register_proof_typed(
                &bytes(&fixture.clock.env, 7),
                &bytes(&fixture.clock.env, 8),
                &fixture.issuer,
                &1,
                &(NOW + 1),
                &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
            ),
            Err(Ok(ProofError::ContractPaused))
        );
    }

    #[test]
    fn maximum_timestamp_is_representable_without_interval_overflow() {
        let fixture = fixture();
        register(&fixture, 9, u64::MAX);
        assert!(fixture.proofs.is_valid_proof(&bytes(&fixture.clock.env, 9)));
    }

    // ── Proof expiry boundary tests ─────────────────────────────────────────

    #[test]
    fn proof_valid_before_expiry() {
        let fixture = fixture();
        register(&fixture, 10, NOW + 10);
        fixture.clock.set_to(NOW + 9);
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 10)));
    }

    #[test]
    fn proof_valid_at_exact_expiry() {
        let fixture = fixture();
        register(&fixture, 11, NOW + 10);
        fixture.clock.set_to(NOW + 10);
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 11)));
    }

    #[test]
    fn proof_invalid_after_expiry() {
        let fixture = fixture();
        register(&fixture, 12, NOW + 10);
        fixture.clock.set_to(NOW + 11);
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 12)));
    }

    // ── Schema deprecation time boundary tests ──────────────────────────────

    #[test]
    fn deprecated_schema_rejects_new_registrations() {
        let fixture = fixture();
        fixture.config.deprecate_schema_version(&1);
        assert_eq!(
            fixture.proofs.try_register_proof_typed(
                &bytes(&fixture.clock.env, 20),
                &bytes(&fixture.clock.env, 30),
                &fixture.issuer,
                &1,
                &(NOW + 1),
                &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
            ),
            Err(Ok(ProofError::UnsupportedSchema))
        );
    }

    #[test]
    fn schema_deprecation_takes_effect_immediately() {
        let fixture = fixture();
        // Approve and deprecate at the same timestamp — no time passes.
        fixture.config.approve_schema_version(&2);
        fixture.config.deprecate_schema_version(&2);
        assert!(!fixture.config.is_schema_version_approved(&2));
        assert_eq!(
            fixture.proofs.try_register_proof_typed(
                &bytes(&fixture.clock.env, 21),
                &bytes(&fixture.clock.env, 31),
                &fixture.issuer,
                &2,
                &(NOW + 1),
                &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
            ),
            Err(Ok(ProofError::UnsupportedSchema))
        );
    }

    #[test]
    fn proof_registered_before_deprecation_remains_valid() {
        let fixture = fixture();
        register(&fixture, 22, NOW + 100);
        // Schema is deprecated after the proof was registered.
        fixture.config.deprecate_schema_version(&1);
        // The existing proof is still valid — deprecation only gates new registrations.
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 22)));
        // But a new registration against the same (now deprecated) schema fails.
        assert_eq!(
            fixture.proofs.try_register_proof_typed(
                &bytes(&fixture.clock.env, 23),
                &bytes(&fixture.clock.env, 33),
                &fixture.issuer,
                &1,
                &(NOW + 100),
                &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
            ),
            Err(Ok(ProofError::UnsupportedSchema))
        );
    }

    // ── Timestamp overflow and zero value tests ─────────────────────────────

    #[test]
    fn zero_expires_at_rejected() {
        let fixture = fixture();
        assert_eq!(
            fixture.proofs.try_register_proof_typed(
                &bytes(&fixture.clock.env, 40),
                &bytes(&fixture.clock.env, 50),
                &fixture.issuer,
                &1,
                &0,
                &soroban_sdk::BytesN::from_array(&fixture.clock.env, &[1u8; 32])
            ),
            Err(Ok(ProofError::ProofExpired))
        );
    }

    #[test]
    fn one_is_valid_expires_at_when_now_is_zero() {
        let clock = LedgerClock::new(0);
        let admin = Address::generate(&clock.env);
        let issuer = Address::generate(&clock.env);
        let config_id = clock.env.register(ProtocolConfigContract, ());
        let config = ProtocolConfigContractClient::new(&clock.env, &config_id);
        config.initialize(&admin);
        config.approve_schema_version(&1);
        config.approve_proof_type(&soroban_sdk::BytesN::from_array(&clock.env, &[1u8; 32]));
        let issuers_id = clock.env.register(IssuerRegistryContract, ());
        let issuers = IssuerRegistryContractClient::new(&clock.env, &issuers_id);
        issuers.initialize(&admin);
        issuers.register_issuer(
            &bytes(&clock.env, 1),
            &issuer,
            &bytes(&clock.env, 2),
            &bytes(&clock.env, 99),
        );
        let proofs_id = clock.env.register(ProofRegistryContract, ());
        let proofs = ProofRegistryContractClient::new(&clock.env, &proofs_id);
        proofs.initialize(&admin, &issuers_id, &config_id);

        // expires_at = 1, now = 0 → 1 > 0, should succeed.
        proofs.register_proof_typed(
            &bytes(&clock.env, 41),
            &bytes(&clock.env, 51),
            &issuer,
            &1,
            &1,
            &soroban_sdk::BytesN::from_array(&clock.env, &[1u8; 32]),
        );
        assert!(proofs.is_valid_proof(&bytes(&clock.env, 41)));
    }

    // ── Revocation dominates expiry (comprehensive) ─────────────────────────

    #[test]
    fn revocation_before_expiry_makes_invalid() {
        let fixture = fixture();
        register(&fixture, 50, NOW + 100);
        // Revoke well before natural expiry.
        fixture.clock.set_to(NOW + 5);
        fixture.proofs.revoke_proof(&bytes(&fixture.clock.env, 50));
        // Even though we're before expiry, revocation makes it invalid.
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 50)));
        assert!(fixture.proofs.is_revoked(&bytes(&fixture.clock.env, 50)));
    }

    #[test]
    fn revoked_at_timestamp_is_correct() {
        let fixture = fixture();
        register(&fixture, 51, NOW + 100);
        let revoke_time = NOW + 42;
        fixture.clock.set_to(revoke_time);
        fixture.proofs.revoke_proof(&bytes(&fixture.clock.env, 51));
        let record = fixture.proofs.get_proof(&bytes(&fixture.clock.env, 51));
        assert_eq!(record.revoked_at, revoke_time);
    }

    // ── Record timestamp correctness ────────────────────────────────────────

    #[test]
    fn created_at_matches_registration_timestamp() {
        let fixture = fixture();
        register(&fixture, 60, NOW + 100);
        let record = fixture.proofs.get_proof(&bytes(&fixture.clock.env, 60));
        assert_eq!(record.created_at, NOW);
        assert_eq!(record.expires_at, NOW + 100);
        assert_eq!(record.status, ProofStatus::Active);
    }

    #[test]
    fn revoked_at_zero_until_revoked() {
        let fixture = fixture();
        register(&fixture, 61, NOW + 100);
        let record = fixture.proofs.get_proof(&bytes(&fixture.clock.env, 61));
        assert_eq!(record.revoked_at, 0);
    }

    // ── Interaction tests ───────────────────────────────────────────────────

    #[test]
    fn multiple_proofs_different_expiry_times() {
        let fixture = fixture();
        register(&fixture, 70, NOW + 10);
        register(&fixture, 71, NOW + 20);
        register(&fixture, 72, NOW + 30);

        // All valid at NOW.
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 70)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 71)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 72)));

        // At NOW+10: 70 valid (inclusive), 71 and 72 still valid.
        fixture.clock.set_to(NOW + 10);
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 70)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 71)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 72)));

        // At NOW+11: 70 expired, 71 and 72 still valid.
        fixture.clock.set_to(NOW + 11);
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 70)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 71)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 72)));

        // At NOW+20: 70 expired, 71 valid (inclusive), 72 still valid.
        fixture.clock.set_to(NOW + 20);
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 70)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 71)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 72)));

        // At NOW+21: 70 and 71 expired, 72 still valid.
        fixture.clock.set_to(NOW + 21);
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 70)));
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 71)));
        assert!(fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 72)));

        // At NOW+31: all expired.
        fixture.clock.set_to(NOW + 31);
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 70)));
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 71)));
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 72)));
    }

    #[test]
    fn revoked_proof_ignores_expiry_advance() {
        let fixture = fixture();
        register(&fixture, 80, NOW + 10);
        // Revoke before expiry.
        fixture.clock.set_to(NOW + 5);
        fixture.proofs.revoke_proof(&bytes(&fixture.clock.env, 80));
        // Advance well past expiry.
        fixture.clock.set_to(NOW + 100);
        // Still revoked, not valid — revocation is terminal.
        assert!(fixture.proofs.is_revoked(&bytes(&fixture.clock.env, 80)));
        assert!(!fixture
            .proofs
            .is_valid_proof(&bytes(&fixture.clock.env, 80)));
        // revoked_at was set at the time of revocation, not at expiry.
        let record = fixture.proofs.get_proof(&bytes(&fixture.clock.env, 80));
        assert_eq!(record.revoked_at, NOW + 5);
    }

    #[test]
    fn dispute_filing_created_window_enforces_exact_boundaries_and_replay() {
        let fixture = fixture();
        register(&fixture, 90, NOW + 100);
        fixture
            .config
            .set_dispute_filing_policy(&DisputeFilingPolicy {
                version: DISPUTE_FILING_POLICY_VERSION,
                anchor: DisputeFilingAnchor::Created,
                opens_after_seconds: 5,
                closes_after_seconds: 10,
            });

        let proof_id = bytes(&fixture.clock.env, 90);
        fixture.clock.set_to(NOW + 4);
        assert_eq!(
            fixture
                .proofs
                .get_dispute_filing_eligibility(&proof_id)
                .status,
            DisputeFilingStatus::NotYetOpen
        );
        assert_eq!(
            fixture.proofs.try_open_dispute(
                &proof_id,
                &fixture.issuer,
                &bytes(&fixture.clock.env, 91)
            ),
            Err(Ok(ProofError::DisputeFilingWindowClosed))
        );

        fixture.clock.set_to(NOW + 5);
        assert_eq!(
            fixture
                .proofs
                .get_dispute_filing_eligibility(&proof_id)
                .status,
            DisputeFilingStatus::Eligible
        );
        fixture
            .proofs
            .open_dispute(&proof_id, &fixture.issuer, &bytes(&fixture.clock.env, 91));
        fixture.proofs.withdraw_dispute(&proof_id);

        fixture.clock.set_to(NOW + 10);
        assert_eq!(
            fixture
                .proofs
                .get_dispute_filing_eligibility(&proof_id)
                .status,
            DisputeFilingStatus::Closed
        );
        assert_eq!(
            fixture.proofs.try_open_dispute(
                &proof_id,
                &fixture.issuer,
                &bytes(&fixture.clock.env, 92)
            ),
            Err(Ok(ProofError::DisputeFilingWindowClosed))
        );
    }

    #[test]
    fn dispute_filing_revocation_anchor_requires_terminal_revocation_timestamp() {
        let fixture = fixture();
        register(&fixture, 92, NOW + 100);
        fixture
            .config
            .set_dispute_filing_policy(&DisputeFilingPolicy {
                version: DISPUTE_FILING_POLICY_VERSION,
                anchor: DisputeFilingAnchor::Revoked,
                opens_after_seconds: 0,
                closes_after_seconds: 10,
            });
        let proof_id = bytes(&fixture.clock.env, 92);
        assert_eq!(
            fixture
                .proofs
                .get_dispute_filing_eligibility(&proof_id)
                .status,
            DisputeFilingStatus::AnchorUnavailable
        );

        fixture.clock.set_to(NOW + 7);
        fixture.proofs.revoke_proof(&proof_id);
        let eligibility = fixture.proofs.get_dispute_filing_eligibility(&proof_id);
        assert_eq!(eligibility.anchor_timestamp, NOW + 7);
        assert_eq!(eligibility.status, DisputeFilingStatus::Eligible);
    }

    #[test]
    fn test_error_catalog_version_queries() {
        let fixture = fixture();
        let pc_cat = fixture.config.get_error_catalog_version();
        assert_eq!(
            pc_cat.catalog_version,
            earnproof_shared::ERROR_CATALOG_VERSION
        );
        assert_eq!(pc_cat.min_code, earnproof_shared::PROTOCOL_CONFIG_ERROR_MIN);
        assert_eq!(pc_cat.max_code, earnproof_shared::PROTOCOL_CONFIG_ERROR_MAX);

        let ir_id = fixture
            .clock
            .env
            .register(issuer_registry::IssuerRegistryContract, ());
        let issuers =
            issuer_registry::IssuerRegistryContractClient::new(&fixture.clock.env, &ir_id);
        let ir_cat = issuers.get_error_catalog_version();
        assert_eq!(
            ir_cat.catalog_version,
            earnproof_shared::ERROR_CATALOG_VERSION
        );
        assert_eq!(ir_cat.min_code, earnproof_shared::ISSUER_REGISTRY_ERROR_MIN);
        assert_eq!(ir_cat.max_code, earnproof_shared::ISSUER_REGISTRY_ERROR_MAX);

        let pr_cat = fixture.proofs.get_error_catalog_version();
        assert_eq!(
            pr_cat.catalog_version,
            earnproof_shared::ERROR_CATALOG_VERSION
        );
        assert_eq!(pr_cat.min_code, earnproof_shared::PROOF_REGISTRY_ERROR_MIN);
        assert_eq!(pr_cat.max_code, earnproof_shared::PROOF_REGISTRY_ERROR_MAX);
    }

    #[test]
    fn test_successor_timelock_and_activation_delay() {
        let fixture = fixture();
        let env = &fixture.clock.env;
        assert_eq!(fixture.config.get_successor_activation_delay(), 0);
        assert_eq!(
            fixture.config.get_successor_status().state,
            earnproof_shared::SuccessorState::Absent
        );

        // Governed delay
        fixture.config.set_successor_activation_delay(&100);
        assert_eq!(fixture.config.get_successor_activation_delay(), 100);

        let successor = Address::generate(env);
        let start_ledger = env.ledger().sequence();
        fixture.config.nominate_successor(&successor);

        let status = fixture.config.get_successor_status();
        assert_eq!(status.state, earnproof_shared::SuccessorState::Pending);
        assert_eq!(status.successor, Some(successor.clone()));
        assert_eq!(status.nominated_at_ledger, start_ledger);
        assert_eq!(status.earliest_activation_ledger, start_ledger + 100);

        // Activating before boundary fails
        let early_res = fixture.config.try_activate_successor();
        assert_eq!(
            early_res,
            Err(Ok(
                earnproof_shared::ContractError::UpgradeTimelockNotElapsed
            ))
        );

        // Shortening governed delay cannot shorten existing nomination
        fixture.config.set_successor_activation_delay(&10);
        let status_after_delay_change = fixture.config.get_successor_status();
        assert_eq!(
            status_after_delay_change.earliest_activation_ledger,
            start_ledger + 100
        );

        // Still fails at start_ledger + 50
        env.ledger().set_sequence_number(start_ledger + 50);
        assert_eq!(
            fixture.config.try_activate_successor(),
            Err(Ok(
                earnproof_shared::ContractError::UpgradeTimelockNotElapsed
            ))
        );

        // Boundary: exact ledger start_ledger + 100 becomes Ready and succeeds
        env.ledger().set_sequence_number(start_ledger + 100);
        assert_eq!(
            fixture.config.get_successor_status().state,
            earnproof_shared::SuccessorState::Ready
        );
        assert!(fixture.config.try_activate_successor().is_ok());

        assert_eq!(
            fixture.config.get_successor_status().state,
            earnproof_shared::SuccessorState::Decommissioned
        );
    }

    #[test]
    fn test_successor_cancellation() {
        let fixture = fixture();
        let env = &fixture.clock.env;
        fixture.config.set_successor_activation_delay(&50);
        let successor = Address::generate(env);
        fixture.config.nominate_successor(&successor);
        assert_eq!(
            fixture.config.get_successor_status().state,
            earnproof_shared::SuccessorState::Pending
        );

        fixture.config.cancel_successor_nomination();
        assert_eq!(
            fixture.config.get_successor_status().state,
            earnproof_shared::SuccessorState::Absent
        );
        assert_eq!(fixture.config.get_successor(), None);
    }

    #[test]
    fn test_versioned_query_response_envelopes() {
        let fixture = fixture();
        let env = &fixture.clock.env;

        let cap_env = fixture.config.get_capacity_envelope();
        assert_eq!(
            cap_env.header.schema_version,
            earnproof_shared::QUERY_ENVELOPE_SCHEMA_VERSION
        );
        assert_eq!(cap_env.header.contract_address, fixture.config.address);
        assert_eq!(cap_env.header.network_id, env.ledger().network_id());
        assert_eq!(
            cap_env.max_capacity,
            earnproof_shared::CONFIG_HISTORY_CAPACITY
        );

        let policy_env = fixture.config.get_policy_envelope(&1);
        assert_eq!(
            policy_env.header.schema_version,
            earnproof_shared::QUERY_ENVELOPE_SCHEMA_VERSION
        );
        assert!(policy_env.policy.max_validity_seconds > 0);

        let hist_env = fixture.config.get_history_envelope();
        assert_eq!(
            hist_env.header.schema_version,
            earnproof_shared::QUERY_ENVELOPE_SCHEMA_VERSION
        );
    }

    #[test]
    fn test_migration_checkpoint_checksum_and_resume() {
        let fixture = fixture();
        let env = &fixture.clock.env;

        fixture.config.begin_migration(&2, &100);
        let status = fixture.config.advance_migration(&0, &10);
        assert_eq!(status.cursor, 10);

        let checkpoint = fixture.config.get_migration_checkpoint().unwrap();
        assert_eq!(
            checkpoint.checkpoint_version,
            earnproof_shared::MIGRATION_CHECKPOINT_VERSION
        );
        assert_eq!(checkpoint.cursor_start, 0);
        assert_eq!(checkpoint.cursor_end, 10);
        assert_eq!(checkpoint.processed_count, 10);
        assert_eq!(checkpoint.total_items, 100);

        // Envelope query
        let env_res = fixture.config.get_checkpoint_envelope();
        assert_eq!(env_res.checkpoint, checkpoint);

        // Resume with valid checkpoint succeeds
        let resume_res = fixture.config.resume_migration(&checkpoint);
        assert_eq!(resume_res.cursor, 10);

        // Checksum mismatch blocks resume
        let mut corrupted = checkpoint.clone();
        corrupted.checksum = soroban_sdk::BytesN::from_array(env, &[0xFFu8; 32]);
        let fail_res = fixture.config.try_resume_migration(&corrupted);
        assert_eq!(
            fail_res,
            Err(Ok(earnproof_shared::ContractError::InvalidInput))
        );
    }
}

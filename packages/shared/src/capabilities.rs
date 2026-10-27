//! Governed contract capability identifiers and versioned capability bitmaps.
//!
//! A client that talks to more than one deployed build of a contract needs to
//! know which optional entrypoints and policy features a given build supports
//! without probing each one. Every contract therefore exposes
//! `get_capabilities`, which returns a [`ContractCapabilities`] report: the
//! bitmap encoding version, the interface version, the stored contract version,
//! and a `u64` bitmap of the capabilities that build implements.
//!
//! ## Rules
//!
//! * Capability identifiers are bit indexes governed by [`CAPABILITY_REGISTRY`].
//!   An identifier is never renumbered or reused for a different meaning.
//! * Within one [`CAPABILITY_BITMAP_VERSION`], bitmaps are additive: a later
//!   build may set new bits but may not clear a bit an earlier build of the
//!   same contract set. Clearing a bit requires a bitmap version bump.
//! * A build may only set bits defined for its bitmap version
//!   ([`KNOWN_CAPABILITIES_V1`]), and only bits it actually implements. The
//!   per-contract constants below are the single source of truth and the
//!   contract tests call a representative entrypoint for every claimed bit.
//! * The bitmap is a compile-time constant of the deployed code, so it is a
//!   deterministic function of the WASM: it does not read mutable storage and
//!   cannot drift during an upgrade, migration, pause, or decommission.

use crate::InterfaceVersion;
use soroban_sdk::contracttype;

/// Encoding version of the capability bitmap returned by `get_capabilities`.
pub const CAPABILITY_BITMAP_VERSION: u32 = 1;

/// Number of capability identifiers defined for bitmap version 1.
pub const CAPABILITY_COUNT_V1: u32 = 24;

/// Every bit defined for bitmap version 1. Bits above this mask are reserved.
pub const KNOWN_CAPABILITIES_V1: u64 = (1u64 << CAPABILITY_COUNT_V1) - 1;

/// `interface_version` query for cross-contract handshakes.
pub const CAP_INTERFACE_VERSION_QUERY: u32 = 0;
/// Two-step administrator transfer with nomination expiry.
pub const CAP_TWO_STEP_ADMIN_TRANSFER: u32 = 1;
/// Resumable storage migration checkpoints (`begin_migration`, `advance_migration`).
pub const CAP_MIGRATION_STATUS: u32 = 2;
/// Successor nomination and decommission (`nominate_successor`, `activate_successor`).
pub const CAP_SUCCESSOR_DECOMMISSION: u32 = 3;
/// Timelocked, allowlisted WASM upgrades (`approve_upgrade`, `upgrade_contract`).
pub const CAP_UPGRADE_TIMELOCK: u32 = 4;
/// Canonical configuration digest query (`get_config_digest`).
pub const CAP_CONFIG_DIGEST: u32 = 5;
/// Storage TTL status queries (`get_instance_ttl_status`).
pub const CAP_TTL_STATUS_QUERY: u32 = 6;
/// Pause acknowledgement commitments (`acknowledge_pause`).
pub const CAP_PAUSE_ACKNOWLEDGEMENTS: u32 = 7;
/// Scoped pause switches.
pub const CAP_SCOPED_PAUSE: u32 = 8;
/// Issuer signing-key lifecycle sequences.
pub const CAP_SIGNING_KEY_LIFECYCLE: u32 = 9;
/// Compromised issuer signing-key markers.
pub const CAP_COMPROMISED_SIGNING_KEYS: u32 = 10;
/// Governance role activation delays.
pub const CAP_GOVERNANCE_ROLE_DELAYS: u32 = 11;
/// Issuer assurance level commitments.
pub const CAP_ISSUER_ASSURANCE_LEVELS: u32 = 12;
/// Proof dispute lifecycle.
pub const CAP_PROOF_DISPUTES: u32 = 13;
/// Proof archival.
pub const CAP_PROOF_ARCHIVAL: u32 = 14;
/// Threshold critical-action approvals.
pub const CAP_CRITICAL_ACTION_APPROVALS: u32 = 15;
/// Emergency governance fast-track.
pub const CAP_EMERGENCY_FAST_TRACK: u32 = 16;
/// Minimum schema activation notice.
pub const CAP_SCHEMA_ACTIVATION_NOTICE: u32 = 17;
/// Schema deprecation replacement pointers.
pub const CAP_SCHEMA_REPLACEMENT_POINTERS: u32 = 18;
/// This capability report (`get_capabilities`, `has_capability`).
pub const CAP_CAPABILITY_BITMAP: u32 = 19;
/// Storage layout version and migration target queries (`get_storage_layout`).
pub const CAP_STORAGE_LAYOUT_QUERY: u32 = 20;
/// Deployment identity commitment (`get_deployment_identity`).
pub const CAP_DEPLOYMENT_IDENTITY: u32 = 21;
/// Validation of dependency deployment identity on bind, replacement, and
/// registration.
pub const CAP_DEPENDENCY_IDENTITY_VALIDATION: u32 = 22;
/// Bounded read-only decommission grace period.
pub const CAP_DECOMMISSION_GRACE: u32 = 23;

/// One governed capability identifier.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CapabilitySpec {
    /// Bit index in the capability bitmap.
    pub id: u32,
    /// Stable machine-readable name.
    pub name: &'static str,
    /// First bitmap version that defines this identifier.
    pub since_bitmap_version: u32,
}

/// Every governed capability identifier, ordered by id. Ids are contiguous
/// from zero so the table index equals the id.
pub const CAPABILITY_REGISTRY: [CapabilitySpec; CAPABILITY_COUNT_V1 as usize] = [
    spec(CAP_INTERFACE_VERSION_QUERY, "interface_version_query"),
    spec(CAP_TWO_STEP_ADMIN_TRANSFER, "two_step_admin_transfer"),
    spec(CAP_MIGRATION_STATUS, "migration_status"),
    spec(CAP_SUCCESSOR_DECOMMISSION, "successor_decommission"),
    spec(CAP_UPGRADE_TIMELOCK, "upgrade_timelock"),
    spec(CAP_CONFIG_DIGEST, "config_digest"),
    spec(CAP_TTL_STATUS_QUERY, "ttl_status_query"),
    spec(CAP_PAUSE_ACKNOWLEDGEMENTS, "pause_acknowledgements"),
    spec(CAP_SCOPED_PAUSE, "scoped_pause"),
    spec(CAP_SIGNING_KEY_LIFECYCLE, "signing_key_lifecycle"),
    spec(CAP_COMPROMISED_SIGNING_KEYS, "compromised_signing_keys"),
    spec(CAP_GOVERNANCE_ROLE_DELAYS, "governance_role_delays"),
    spec(CAP_ISSUER_ASSURANCE_LEVELS, "issuer_assurance_levels"),
    spec(CAP_PROOF_DISPUTES, "proof_disputes"),
    spec(CAP_PROOF_ARCHIVAL, "proof_archival"),
    spec(CAP_CRITICAL_ACTION_APPROVALS, "critical_action_approvals"),
    spec(CAP_EMERGENCY_FAST_TRACK, "emergency_fast_track"),
    spec(CAP_SCHEMA_ACTIVATION_NOTICE, "schema_activation_notice"),
    spec(
        CAP_SCHEMA_REPLACEMENT_POINTERS,
        "schema_replacement_pointers",
    ),
    spec(CAP_CAPABILITY_BITMAP, "capability_bitmap"),
    spec(CAP_STORAGE_LAYOUT_QUERY, "storage_layout_query"),
    spec(CAP_DEPLOYMENT_IDENTITY, "deployment_identity"),
    spec(
        CAP_DEPENDENCY_IDENTITY_VALIDATION,
        "dependency_identity_validation",
    ),
    spec(CAP_DECOMMISSION_GRACE, "decommission_grace"),
];

const fn spec(id: u32, name: &'static str) -> CapabilitySpec {
    CapabilitySpec {
        id,
        name,
        since_bitmap_version: 1,
    }
}

/// Returns the bitmap bit for a capability identifier, or `0` for an
/// identifier outside the 64-bit range.
pub const fn capability_bit(id: u32) -> u64 {
    if id >= 64 {
        0
    } else {
        1u64 << id
    }
}

/// Capabilities implemented by `issuer-registry`.
pub const ISSUER_REGISTRY_CAPABILITIES: u64 = capability_bit(CAP_INTERFACE_VERSION_QUERY)
    | capability_bit(CAP_TWO_STEP_ADMIN_TRANSFER)
    | capability_bit(CAP_MIGRATION_STATUS)
    | capability_bit(CAP_SUCCESSOR_DECOMMISSION)
    | capability_bit(CAP_UPGRADE_TIMELOCK)
    | capability_bit(CAP_CONFIG_DIGEST)
    | capability_bit(CAP_TTL_STATUS_QUERY)
    | capability_bit(CAP_SIGNING_KEY_LIFECYCLE)
    | capability_bit(CAP_COMPROMISED_SIGNING_KEYS)
    | capability_bit(CAP_GOVERNANCE_ROLE_DELAYS)
    | capability_bit(CAP_ISSUER_ASSURANCE_LEVELS)
    | capability_bit(CAP_CAPABILITY_BITMAP)
    | capability_bit(CAP_STORAGE_LAYOUT_QUERY)
    | capability_bit(CAP_DEPLOYMENT_IDENTITY)
    | capability_bit(CAP_DECOMMISSION_GRACE);

/// Capabilities implemented by `proof-registry`.
pub const PROOF_REGISTRY_CAPABILITIES: u64 = capability_bit(CAP_TWO_STEP_ADMIN_TRANSFER)
    | capability_bit(CAP_MIGRATION_STATUS)
    | capability_bit(CAP_SUCCESSOR_DECOMMISSION)
    | capability_bit(CAP_UPGRADE_TIMELOCK)
    | capability_bit(CAP_CONFIG_DIGEST)
    | capability_bit(CAP_TTL_STATUS_QUERY)
    | capability_bit(CAP_PAUSE_ACKNOWLEDGEMENTS)
    | capability_bit(CAP_SCOPED_PAUSE)
    | capability_bit(CAP_PROOF_DISPUTES)
    | capability_bit(CAP_PROOF_ARCHIVAL)
    | capability_bit(CAP_CAPABILITY_BITMAP)
    | capability_bit(CAP_STORAGE_LAYOUT_QUERY)
    | capability_bit(CAP_DEPLOYMENT_IDENTITY)
    | capability_bit(CAP_DEPENDENCY_IDENTITY_VALIDATION)
    | capability_bit(CAP_DECOMMISSION_GRACE);

/// Capabilities implemented by `protocol-config`.
pub const PROTOCOL_CONFIG_CAPABILITIES: u64 = capability_bit(CAP_INTERFACE_VERSION_QUERY)
    | capability_bit(CAP_TWO_STEP_ADMIN_TRANSFER)
    | capability_bit(CAP_MIGRATION_STATUS)
    | capability_bit(CAP_SUCCESSOR_DECOMMISSION)
    | capability_bit(CAP_UPGRADE_TIMELOCK)
    | capability_bit(CAP_PAUSE_ACKNOWLEDGEMENTS)
    | capability_bit(CAP_SCOPED_PAUSE)
    | capability_bit(CAP_CRITICAL_ACTION_APPROVALS)
    | capability_bit(CAP_EMERGENCY_FAST_TRACK)
    | capability_bit(CAP_SCHEMA_ACTIVATION_NOTICE)
    | capability_bit(CAP_SCHEMA_REPLACEMENT_POINTERS)
    | capability_bit(CAP_CAPABILITY_BITMAP)
    | capability_bit(CAP_STORAGE_LAYOUT_QUERY)
    | capability_bit(CAP_DEPLOYMENT_IDENTITY)
    | capability_bit(CAP_DECOMMISSION_GRACE);

/// Versioned capability report returned by every contract's `get_capabilities`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCapabilities {
    /// Encoding version of `capabilities`; see [`CAPABILITY_BITMAP_VERSION`].
    pub bitmap_version: u32,
    /// Interface version implemented by the deployed code.
    pub interface_version: InterfaceVersion,
    /// Stored monotonic contract version (`0` before `initialize`).
    pub contract_version: u32,
    /// Bitmap of implemented capability identifiers.
    pub capabilities: u64,
}

/// Returns the bits defined for a bitmap version, or `0` for an unknown version.
pub const fn known_capabilities(bitmap_version: u32) -> u64 {
    match bitmap_version {
        1 => KNOWN_CAPABILITIES_V1,
        _ => 0,
    }
}

/// A bitmap is valid when its version is known and it sets only bits that
/// version defines.
pub const fn capabilities_are_valid(bitmap_version: u32, capabilities: u64) -> bool {
    let known = known_capabilities(bitmap_version);
    known != 0 && capabilities & !known == 0
}

/// Upgrade rule within one bitmap version: the next build keeps every bit the
/// previous build set.
pub const fn capabilities_are_additive(previous: u64, next: u64) -> bool {
    previous & !next == 0
}

/// Answers `has_capability` for any identifier. Identifiers that are out of
/// range or undefined for the bitmap version are reported as unsupported.
pub const fn has_capability(bitmap_version: u32, capabilities: u64, id: u32) -> bool {
    let bit = capability_bit(id);
    bit != 0 && known_capabilities(bitmap_version) & bit != 0 && capabilities & bit != 0
}

/// Builds the report for a contract.
pub fn capability_report(
    interface_version: InterfaceVersion,
    contract_version: u32,
    capabilities: u64,
) -> ContractCapabilities {
    ContractCapabilities {
        bitmap_version: CAPABILITY_BITMAP_VERSION,
        interface_version,
        contract_version,
        capabilities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_are_contiguous_unique_and_named() {
        for (index, entry) in CAPABILITY_REGISTRY.iter().enumerate() {
            assert_eq!(entry.id, index as u32);
            assert!(!entry.name.is_empty());
            assert_eq!(entry.since_bitmap_version, 1);
            for other in CAPABILITY_REGISTRY.iter().skip(index + 1) {
                assert_ne!(entry.name, other.name);
            }
        }
        let mut mask = 0u64;
        for entry in CAPABILITY_REGISTRY.iter() {
            mask |= capability_bit(entry.id);
        }
        assert_eq!(mask, KNOWN_CAPABILITIES_V1);
    }

    #[test]
    fn contract_bitmaps_only_claim_defined_bits() {
        for bits in [
            ISSUER_REGISTRY_CAPABILITIES,
            PROOF_REGISTRY_CAPABILITIES,
            PROTOCOL_CONFIG_CAPABILITIES,
        ] {
            assert!(capabilities_are_valid(CAPABILITY_BITMAP_VERSION, bits));
            assert!(has_capability(1, bits, CAP_CAPABILITY_BITMAP));
        }
        // Only proof-registry consumes dependencies.
        assert!(!has_capability(
            1,
            ISSUER_REGISTRY_CAPABILITIES,
            CAP_DEPENDENCY_IDENTITY_VALIDATION
        ));
        assert!(has_capability(
            1,
            PROOF_REGISTRY_CAPABILITIES,
            CAP_DEPENDENCY_IDENTITY_VALIDATION
        ));
    }

    #[test]
    fn invalid_versions_and_reserved_bits_are_rejected() {
        assert!(!capabilities_are_valid(0, 1));
        assert!(!capabilities_are_valid(2, 1));
        assert!(!capabilities_are_valid(1, 1u64 << CAPABILITY_COUNT_V1));
        assert!(!capabilities_are_valid(1, u64::MAX));
        assert!(capabilities_are_valid(1, 0));
        assert!(capabilities_are_valid(1, KNOWN_CAPABILITIES_V1));
    }

    #[test]
    fn has_capability_boundaries_are_deterministic() {
        let all = KNOWN_CAPABILITIES_V1;
        assert!(has_capability(1, all, 0));
        assert!(has_capability(1, all, CAPABILITY_COUNT_V1 - 1));
        assert!(!has_capability(1, all, CAPABILITY_COUNT_V1));
        assert!(!has_capability(1, u64::MAX, 63));
        assert!(!has_capability(1, u64::MAX, 64));
        assert!(!has_capability(1, u64::MAX, u32::MAX));
        assert!(!has_capability(2, all, 0));
        assert_eq!(capability_bit(64), 0);
    }

    #[test]
    fn additive_rule_rejects_cleared_bits() {
        let previous = capability_bit(1) | capability_bit(2);
        assert!(capabilities_are_additive(previous, previous));
        assert!(capabilities_are_additive(
            previous,
            previous | capability_bit(5)
        ));
        assert!(!capabilities_are_additive(previous, capability_bit(1)));
        assert!(capabilities_are_additive(0, 0));
    }
}

# Contract Capability Bitmaps

Clients that talk to more than one deployed build of a contract need a cheap
way to learn which optional entrypoints and policy features a build supports.
Every contract exposes:

| Entrypoint | Result | Authorization |
|---|---|---|
| `get_capabilities()` | `ContractCapabilities` | none |
| `has_capability(capability: u32)` | `bool` | none |

`ContractCapabilities` carries:

- `bitmap_version`: encoding version of the bitmap (currently `1`).
- `interface_version`: the interface version of the deployed code.
  `proof-registry` reports `PROOF_REGISTRY_INTERFACE_VERSION` (1.0.0) here; it
  has no `interface_version` entrypoint because no contract handshakes on it.
- `contract_version`: the stored monotonic contract version (`0` before
  `initialize`).
- `capabilities`: a `u64` bitmap of implemented capability identifiers.

## Governed identifiers

Identifiers are bit indexes defined in
[`packages/shared/src/capabilities.rs`](../packages/shared/src/capabilities.rs)
(`CAPABILITY_REGISTRY`). Version 1 defines ids `0..=23`:

| Id | Name | issuer-registry | proof-registry | protocol-config |
|---|---|---|---|---|
| 0 | `interface_version_query` | yes | | yes |
| 1 | `two_step_admin_transfer` | yes | yes | yes |
| 2 | `migration_status` | yes | yes | yes |
| 3 | `successor_decommission` | yes | yes | yes |
| 4 | `upgrade_timelock` | yes | yes | yes |
| 5 | `config_digest` | yes | yes | |
| 6 | `ttl_status_query` | yes | yes | |
| 7 | `pause_acknowledgements` | | yes | yes |
| 8 | `scoped_pause` | | yes | yes |
| 9 | `signing_key_lifecycle` | yes | | |
| 10 | `compromised_signing_keys` | yes | | |
| 11 | `governance_role_delays` | yes | | |
| 12 | `issuer_assurance_levels` | yes | | |
| 13 | `proof_disputes` | | yes | |
| 14 | `proof_archival` | | yes | |
| 15 | `critical_action_approvals` | | | yes |
| 16 | `emergency_fast_track` | | | yes |
| 17 | `schema_activation_notice` | | | yes |
| 18 | `schema_replacement_pointers` | | | yes |
| 19 | `capability_bitmap` | yes | yes | yes |
| 20 | `storage_layout_query` | yes | yes | yes |
| 21 | `deployment_identity` | yes | yes | yes |
| 22 | `dependency_identity_validation` | | yes | |
| 23 | `decommission_grace` | yes | yes | yes |

## Rules

- **Additive within a version.** A later build of a contract may set new bits
  but never clears a bit an earlier build set. Clearing a bit, renumbering an
  id, or renaming one requires a new `bitmap_version`. The golden
  [`tests/compatibility/goldens/capabilities.json`](../tests/compatibility/goldens/capabilities.json)
  gates this in CI; append to it when a capability is added.
- **No unavailable claims.** A build only sets bits defined for its bitmap
  version and only bits it implements. Each contract's
  `tests/capability_bitmaps.rs` calls a representative entrypoint for every
  claimed bit and fails if a claimed bit has no probe.
- **Deterministic.** The bitmap is a compile-time constant of the WASM. It does
  not read storage, so pausing, migrating, entering decommission grace, or
  changing the stored contract version never changes it. Only
  `contract_version` is read from storage.
- **Unknown identifiers.** `has_capability` returns `false` for an id that is
  undefined for the bitmap version or `>= 64`. Clients must treat an unknown
  `bitmap_version` as "no capabilities known" and fall back to probing.

Both entrypoints are read-only, need no authorization, and cannot fail.

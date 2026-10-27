# Emergency operations

This document defines what the EarnProof contracts do while the protocol is
paused, who may change that state, and what an operator can still rely on during
an incident. It is the specification that `tests/emergency/` executes: every rule
below has a corresponding assertion, and the two must be changed together.

Scope is the on-chain surface only. Backend containment (API rate limits, key
custody, webhook suspension) is out of scope here and belongs to
`earnproof-backend`.

For what to monitor while paused, how to investigate, escalation timing, who
to notify and when, and the post-incident review process, see
[`docs/runbooks/incident-communication.md`](runbooks/incident-communication.md) —
this document is the on-chain specification that runbook builds on top of.

## The pause switch

New integrations should call `pause_with_metadata(incident_id,
reason_commitment)`. Both values are fixed 32-byte commitments: `incident_id`
correlates the transition with the operator's incident system, while
`reason_commitment` binds the on-chain action to a reason kept off-chain.
Plaintext incident descriptions are never accepted. `get_current_pause`
returns only an active incident and `get_latest_pause` retains the most recent
active or closed incident. `unpause` closes that same incident by setting its
end timestamp; it does not create a second incident.

An identical pause retry and an already-unpaused retry are no-ops. A second
pause with different metadata is rejected until the active incident is closed.
`migrate_pause_metadata` is the admin-only path for a deployment that was
already paused by an older WASM. The zero-argument `pause` entry point remains
for ABI compatibility and records zero commitments; operational tooling must
use `pause_with_metadata` for incident correlation.

`protocol-config` owns a single boolean. `proof-registry` reads it over a
cross-contract call before admitting new proofs. `issuer-registry` does not read
it at all.

That asymmetry is deliberate. Pausing is meant to stop the protocol taking on
**new** obligations while leaving every tool an operator needs to contain the
existing ones. A pause that also froze revocation would remove the responder's
only lever at the moment they need it.

| Contract | Reads the pause flag |
|---|---|
| `contracts/protocol-config/src/lib.rs` | owns it |
| `contracts/proof-registry/src/lib.rs` | yes — in `register_proof` only |
| `contracts/issuer-registry/src/lib.rs` | no |

## Behaviour under pause

Every public entry point across the three contracts appears below. The
`tests/emergency/src/pause_matrix.rs` table mirrors this list exactly; a
mismatch in either direction fails `matrix_covers_every_public_entry_point`.

"Available" means the call behaves identically to an unpaused deployment.
"Contained" means it is rejected for as long as the pause is in force.

### `protocol-config`

| Entry point | Under pause | Why |
|---|---|---|
| `get_admin` | Available | Read. |
| `is_paused` | Available | Read. Operators and integrators must be able to observe containment. |
| `get_config_version` | Available | Read. Monotonic counter used to detect unaccounted changes. |
| `is_schema_version_approved` | Available | Read. |
| `approve_schema_version` | Available | Schema administration is a response tool, not a new obligation. |
| `deprecate_schema_version` | Available | Withdrawing a bad schema must not require unpausing first. |
| `set_admin` | Available | Authority handover must work during an incident. |
| `pause` | Available | Idempotent; a repeat must not toggle. |
| `unpause` | Available | The recovery path. |

### `issuer-registry`

| Entry point | Under pause | Why |
|---|---|---|
| `get_admin` | Available | Read. |
| `get_issuer` | Available | Read. |
| `get_issuer_by_address` | Available | Read. |
| `is_active_issuer` | Available | Read. Relying parties keep verifying. |
| `is_active_address` | Available | Read. |
| `register_issuer` | Available | The registry is not pause-gated; see the asymmetry note above. |
| `update_issuer` | Available | Metadata correction is a response tool. |
| `suspend_issuer` | Available | **Containment operation.** |
| `reactivate_issuer` | Available | Reversal of a suspension. |
| `revoke_issuer` | Available | **Containment operation.** Terminal. |
| `rotate_issuer_address` | Available | **Containment operation** — cuts a compromised key away from an identity. |

### `proof-registry`

| Entry point | Under pause | Why |
|---|---|---|
| `get_admin` | Available | Read. |
| `get_issuer_registry` | Available | Read. |
| `get_protocol_config` | Available | Read. |
| `get_proof` | Available | Read. Verification must not go dark during an incident. |
| `is_valid_proof` | Available | Read. |
| `is_revoked` | Available | Read. A relying party must still learn that a credential was revoked. |
| `register_proof` | **Contained** | The only operation that admits new obligations. |
| `revoke_proof` | Available | **Containment operation.** |
| `admin_revoke_proof` | Available | **Containment operation.** |

`initialize` is excluded from the table on all three contracts: it is
single-shot and unreachable on a live deployment. Its rejection is asserted in
`tests/emergency/src/sequences.rs`.

## Authority rules

1. Only the **current** administrator may change pause state. Authority is read
   from storage at call time, never cached, so a rotation takes effect
   immediately.
2. Rotation moves authority and nothing else. It does not clear the pause flag.
   An operator handing over control mid-incident does not silently re-open
   registration.
3. A rotated-out administrator retains nothing. They cannot pause, unpause, or
   rotate authority back to themselves.
4. Each contract holds its own administrator record. Rotating the
   `protocol-config` admin does not move authority over either registry — a
   single rotation contains less than it might appear to.
5. Re-initialisation is rejected on an initialised deployment. Were it not, an
   attacker could reset the administrator without emitting a rotation event,
   which would be the quietest available privilege escalation.

### Stranding

`set_admin` accepts any address. The contract cannot verify that a successor is
able to authorise, so an operator can strand a paused contract by rotating to an
address they do not control.

This is an **accepted risk**, not a mitigated one. What the contracts guarantee
is that the change is never silent: every rotation advances `config_version` and
emits `AdminChanged`, so a monitor watching either signal sees it. The
operational control is the handover procedure below, not a contract check.

## Recovery procedure

1. **Contain.** `pause()` on `protocol-config`. Confirm with `is_paused()`.
2. **Assess.** Reads remain available. Use `get_proof`, `is_revoked`, and
   `get_issuer` to establish scope. No mutation is required to investigate.
3. **Revoke.** For each affected credential, `admin_revoke_proof`. For a
   compromised issuer, `suspend_issuer` (reversible) or `revoke_issuer`
   (terminal). For a compromised issuer *key* where the identity is sound,
   `rotate_issuer_address` — the old address stops resolving immediately.
4. **Withdraw schemas** if the incident is schema-borne:
   `deprecate_schema_version`. Callers holding transactions built before the
   incident will be rejected on retry, including after the pause lifts.
5. **Hand over**, if required: `set_admin`. Verify the successor with
   `get_admin()` and confirm `config_version` advanced *before* proceeding.
6. **Recover.** `unpause()`. Exactly one operation returns: `register_proof`.
   Everything revoked during the incident stays revoked.

### What does not come back

`unpause` restores registration and nothing else. A revoked issuer cannot
register proofs afterwards; a revoked proof stays revoked; a deprecated schema
stays deprecated. Recovery is not a rollback.

## Retiring a contract

To retire a contract without cutting readers off, nominate the successor, then
call `begin_decommission_grace` to open a bounded read-only window and
`finalize_decommission` once it has elapsed. The phases, allowed operations,
and exact boundary are in [Decommission Grace Periods](./decommission-grace.md).

## Evidence and privacy

Operators reconstruct an incident from two sources:

- **Events.** `Paused`, `Unpaused`, `AdminChanged`, `SchemaApproved`,
  `SchemaDeprecated` on `protocol-config`; the `Issuer*` events on
  `issuer-registry`. Versioned fixtures live in `tests/fixtures/events/`.
- **`config_version`.** Monotonic across every privileged `protocol-config`
  mutation. A gap is evidence of a change the operator has not accounted for.

Neither source carries private data. The contracts store only hashes —
`proof_id_hash`, `commitment_hash`, `metadata_hash` — never a wallet-linked
identifier, an amount, a memo, or an off-chain payload. Panic messages are fixed
strings (`"protocol is paused"`, `"proof already revoked"`) and never interpolate
call arguments.

One consequence is worth stating plainly: a cross-contract rejection surfaces to
the caller as `Error(WasmVm, InvalidAction)`, not as the underlying message. The
specific reason is visible in the diagnostic event log but is not part of the
returned error. Callers must not attempt to distinguish failure causes from the
error value alone.

`proof-registry` emits no events. Registration and revocation are observable
only through storage reads, which is a **known gap** for off-chain indexers
building an incident timeline.

## Test coverage

`tests/emergency/` (30 tests):

| Module | Covers |
|---|---|
| `harness.rs` | Three-contract deployment; synthetic fixtures only. |
| `pause_matrix.rs` | The table above, asserted in both directions — every entry point must also be reachable while unpaused, so a "contained" verdict is attributable to the pause and not to a pre-existing break. |
| `admin_rotation.rs` | Authority rules 1–4, containment-tool availability, issuer key rotation, terminal revocation. Uses recorded auth trees rather than rejection, since `mock_all_auths` admits every call. |
| `sequences.rs` | Every ordering of 2 and 3 operations from an 8-symbol alphabet (576 sequences) replayed against an independent model written from *this document*, plus repetition, conflicting pairs, stale callers, cross-contract disagreement, and no-partial-state on rejection. |

Run with `cargo test -p emergency-tests`, or `cargo test --workspace` for
everything.

## Refresh checklist

This document is stale when any of the following happens:

- [ ] A public entry point is added or removed → update the tables and
      `DOCUMENTED_ENTRY_POINTS` in `pause_matrix.rs`.
- [ ] A contract starts or stops reading the pause flag → update the asymmetry
      table and the affected rows.
- [ ] An authority rule changes → update "Authority rules" and `admin_rotation.rs`.
- [ ] An operation's acceptance conditions change → update `Model::apply` in
      `sequences.rs`, which is written from this document by design.
- [ ] `proof-registry` gains events → remove the known gap above, and update
      the corresponding note in `docs/runbooks/incident-communication.md`'s
      investigation procedure.

# Proof Activation Acknowledgements

A proof registered with `register_proof_with_activation` becomes eligible when the ledger timestamp reaches `activates_at`. `acknowledge_activation` leaves an auditable record of that moment.

## Who and when

`acknowledge_activation(proof_id_hash, activator)` needs `activator` to sign and to be either the proof's issuer or an address that issuer authorized with `set_activation_delegate(issuer, delegate, enabled)`. A delegate grant is scoped to the issuer that made it.

Check order is fixed: unknown proof, pause, authorization (`UnauthorizedActivator`), replay (`ActivationAlreadyAcknowledged`), eligibility (`ActivationNotEligible`).

`get_activation_blocker(proof_id_hash)` returns the first reason a proof cannot be acknowledged, in this order, or `None`:

| Blocker | Condition |
|---|---|
| `Revoked` | The proof was revoked. |
| `Superseded` | A successor proof exists, so this one was cancelled. |
| `Disputed` | A dispute is `Open` or `Resolved`. `Withdrawn` and `Rejected` disputes do not block. |
| `Expired` | `now > expires_at`. |
| `NotDelayed` | `activates_at` is `0`. |
| `NotYetActive` | `now < activates_at`. |

Boundaries are exact: `now == activates_at` is eligible, and `now == expires_at` is still eligible, matching `get_proof_validity`.

## The record

`get_activation_acknowledgement` returns `ActivationAcknowledgement`: `acknowledged_by`, `actor_role` (`Issuer` or `Delegate`), `ledger`, `timestamp`, `sequence`, and the proof's `activates_at`. `sequence` is registry-wide, starts at `1`, and has no gaps because a rejected attempt never consumes one. The record is written once. A second attempt, in the same ledger or later, fails and leaves it untouched. One `proof_activation_acknowledged` event is published per success.

## Compatibility

Acknowledgement is an audit trail only. It does not change `get_proof_validity`, `is_valid_proof`, or any existing entry point, and proofs without an acknowledgement stay valid. `ProofRecord` is unchanged; records live under `ActivationAck(proof_id_hash)`, with the counter under `ActivationAckCount`.

## Tests

`contracts/proof-registry/tests/activation_acknowledgements.rs` and `packages/shared/src/policy_bindings.rs`.

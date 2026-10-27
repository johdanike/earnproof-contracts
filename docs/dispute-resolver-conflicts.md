# Dispute Resolver Conflicts of Interest

A resolver must not decide a dispute it is a party to. The check runs before any dispute state changes.

## Who is a party

| Role | How it is identified |
|---|---|
| `Issuer` | The proof's issuer address, and the stable issuer id that `issuer-registry` returns for it, including addresses the issuer has rotated away from (`get_issuer_id_for_address`). |
| `EvidenceSubmitter` | The address that opened the dispute, and its bound participant commitment. |
| `SubjectAuthorizedActor` | An address the issuer named with `authorize_subject_actor(proof_id_hash, actor)`, stored by commitment. |

A resolver conflicts when its address equals a party's address, or when any commitment it carries equals a party's commitment. The commitments compared are the registered resolver commitment, a bound participant commitment, the default commitment derived from its address, and its issuer id. Rotating to a new address that carries the same commitment therefore does not clear a conflict, and neither does a delegated actor that shares the commitment.

## Entry points

- `register_dispute_resolver(resolver, commitment)` and `remove_dispute_resolver(resolver)`: admin-only. The commitment must be non-zero.
- `bind_participant_commitment(participant, commitment)`: the participant signs. Write-once per address; the same value is accepted again, a different one fails with `ParticipantCommitmentLocked`.
- `resolve_dispute_as` and `reject_dispute_as`: a registered resolver decides. An unregistered address fails with `ResolverNotRegistered`.
- `resolve_dispute` and `reject_dispute` keep their admin-only signature and now apply the same conflict check to the admin.
- `get_resolver_conflict(proof_id_hash, resolver)` returns the first conflicting role or `None`, without changing state.

A conflict fails with `ResolverConflictOfInterest`. The dispute stays `Open` and nothing is written. A decided dispute fails with `DisputeNotOpen` as before.

## Limits

Address rotation is only covered for participants that have an issuer id or a bound commitment. A party that never bound a commitment is matched by address alone. Bind commitments before a dispute starts: they cannot be changed afterwards.

## Compatibility

`DisputeRecord` is unchanged and no existing entry point changed signature. The new check can reject an admin who opened or is party to a dispute, which was allowed before.

## Tests

`contracts/proof-registry/tests/resolver_conflicts.rs`, `contracts/issuer-registry/tests/former_address_identity.rs`, `packages/shared/src/policy_bindings.rs`, and the authorization matrix rows in `tests/authorization`.

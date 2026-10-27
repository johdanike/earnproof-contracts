# Revocation Authority Binding

Every proof revocation records the authority that justified it. The record is evidence: a later policy change cannot rewrite it.

## What is committed

`get_revocation_authority(proof_id_hash)` returns a `RevocationAuthority`:

| Field | Meaning |
|---|---|
| `policy_version` | Authority policy version in force when the revocation committed. |
| `actor_role` | `Issuer` (the proof's recorded issuer) or `Admin` (the registry admin). |
| `actor` | Address that signed. |
| `authorization_domain` | SHA-256 over a fixed label, the registry address, the role, and the policy version. Different roles, versions, or registries never share a domain. |
| `revoked_at`, `revoked_ledger` | Ledger time and sequence of the revocation. |

The same `policy_version`, `actor_role`, and `authorization_domain` are published on `proof_revoked` and on `ProofRevokedInBatch`. The record is written in the same call that revokes the proof and is never rewritten. A second revocation fails with `ProofAlreadyRevoked` before anything is written.

## Policy version

`get_authority_policy_version` reports the version in force. A deployment that never set one reports `1`. `set_authority_policy_version(new_version)` is admin-only and requires a strictly greater value, so a replayed or reordered call fails with `InvalidPolicyVersion`. It publishes `authority_policy_version_changed`.

## Pinned revocation

`revoke_proof_at_policy` and `admin_revoke_proof_at_policy` take the version the caller reviewed. Any other version, older or newer, fails with `StalePolicyVersion` and changes nothing. The plain `revoke_proof` and `admin_revoke_proof` keep working and record whatever version is current.

Check order is fixed: unknown proof, authorization, `ProofAlreadyRevoked`, `StalePolicyVersion`.

## Compatibility and migration

- The `ProofRecord` layout is unchanged. Evidence lives under a separate storage key, `RevocationAuthority(proof_id_hash)`.
- Proofs revoked before this change have no record. `get_revocation_authority` returns `ProofNotFound` for them. Nothing is back-filled, because the version in force at the time is unknown.
- `proof_revoked` gains three trailing fields. Indexers that ignore unknown fields keep working (fixture compatibility: additive).
- Batch revocation now writes one extra entry per proof. At `MAX_PROOF_BATCH_SIZE` this stays well under the ledger footprint limit described in `packages/shared`.

## Tests

`contracts/proof-registry/tests/revocation_authority.rs`, `packages/shared/src/policy_bindings.rs`, and the authorization matrix rows in `tests/authorization`.

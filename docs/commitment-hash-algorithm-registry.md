# Commitment Hash Algorithm Registry

`protocol-config` holds one governed registry of the algorithm identifiers that evidence, policy, consent, and metadata commitments may use.

## Lifecycle

```
approve_hash_algorithm -> activate_hash_algorithm -> deprecate_hash_algorithm
        Approved                  Active                    Deprecated
```

All three calls are admin-only. Deprecation is also allowed straight from `Approved`, to withdraw an identifier that never went live. Every other transition fails with `InvalidHashAlgorithmTransition`, so replays and skipped steps change nothing. Identifier `0` is reserved (`InvalidHashAlgorithmId`).

| Status | Create new commitment | Verify existing commitment |
|---|---|---|
| unknown | no (`HashAlgorithmNotFound`) | no (`HashAlgorithmNotFound`) |
| `Approved` | no (`HashAlgorithmNotActive`) | no (`HashAlgorithmNotActive`) |
| `Active` | yes | yes |
| `Deprecated` | no (`HashAlgorithmDeprecated`) | yes |

Deprecated algorithms stay verifiable so historical records keep validating. They can never create a new commitment again.

## Queries

- `get_hash_algorithm(id)` returns the `HashAlgorithmRecord` (status, `approved_at`, `activated_at`, `deprecated_at`, `revision`).
- `can_create_commitment(id)` and `can_verify_commitment(id)` return booleans.
- `check_hash_algorithm(id, for_creation)` returns the typed error, for cross-contract callers.

## Events

`hash_algorithm_approved`, `hash_algorithm_activated`, and `hash_algorithm_deprecated`, each with `algorithm_id`, `revision`, and the change timestamp. Fixtures are under `tests/fixtures/events/protocol-config/v1/`. Each change also advances the config version.

## Compatibility

The older `set_commitment_algorithm` toggle (identifiers `0` and `1`) is untouched, and so are existing commitments. The new registry does not infer anything from it: governance must approve and activate identifier `1` to mirror it. Nothing in `proof-registry` calls the registry yet, so enforcement at the commitment-creating call sites is the integration step that follows.

## Tests

`contracts/protocol-config/tests/hash_algorithm_registry.rs` and `packages/shared/src/policy_bindings.rs`.

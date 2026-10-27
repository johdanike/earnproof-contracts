# Versioned policy, issuance snapshots, and dispute evidence commitments

Closes #248
Closes #249
Closes #252

## Summary

- Version issuer policy commitment updates monotonically and retain the latest 16 privacy-safe summaries with effective ledger and timestamp. Failed updates do not advance version or create history; legacy current-policy storage remains readable.
- Bind each newly registered proof to immutable digest/version references for the proof registry, issuer registry, protocol configuration, schema policy, and issuer policy. Store the snapshot in a sidecar so legacy `ProofRecord` values remain decodable. Include it in existing registration events without adding extra registration events.
- Allow authorized parties to append up to 16 hash-only evidence commitments per open dispute round. Each entry records submitter role, algorithm, sequence, ledger, and timestamp; raw evidence, URLs, and claim text are not accepted. Terminal disputes reject additions.

## Validation

Passed:

- `cargo test -p event-fixtures` — 17 passed.
- `cargo test -p earnproof-shared --lib` — 12 passed.
- `git diff --check` — passed.

Blocked by existing checkout compilation errors:

- `cargo fmt --check` — cannot parse `contracts/issuer-registry/src/lib.rs` (unclosed delimiters around signing-key lifecycle code) or `contracts/protocol-config/src/lib.rs` (unclosed delimiter in schema activation). It also reports pre-existing formatting diffs in authorization and error-catalog tests.
- `cargo clippy --all-targets --all-features -- -D warnings` — blocked by those issuer/protocol parse errors and the existing Soroban function-name limit on `register_proof_with_type_identifier` (35 characters; maximum 32).
- `cargo test --workspace` — blocked by the same issuer/protocol parse errors and the existing Soroban function-name limit.
- `cargo run -p earnproof-fuzz --bin fuzz_proof_context -- -runs=1000` — blocked while compiling the same contract dependencies; the fuzz target did not execute.
- `cargo check -p proof-registry --lib` — blocked by the existing `register_proof_with_type_identifier` ABI name limit (35 characters; maximum 32).

The issuer-registry and proof-registry focused unit tests could not be executed until these existing source/ABI blockers are repaired.

## Coverage

Policy tests cover rotation, bounded eviction, failed-update atomicity, maximum sequence, legacy reads, authorization, and event parity. Proof tests cover snapshot immutability under later schema configuration changes and legacy absence. Dispute tests cover ordered append, duplicate rejection (including the opening commitment), authorization, algorithm and zero-hash validation, capacity limits, terminal-state rejection, and round rotation without loss of prior evidence. Encoding tests pin the domain-separated evidence hash vector.

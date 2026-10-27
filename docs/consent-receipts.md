# Disclosure Consent Receipt Commitments

`proof-registry::commit_disclosure_consent` anchors a verifier's off-chain
disclosure receipt to an existing proof without sending the receipt, verifier
identity, or disclosed claims to the contract. The proof issuer authorizes the
anchor. The issuer authorization is not a verifier identity.

## Off-chain receipt hash

The caller supplies `receipt_hash`, a SHA-256 digest of the canonical receipt
bytes together with a fresh, secret 32-byte nonce. The nonce and receipt remain
off-chain. Do not submit an unhashed receipt or an unsalted digest of
low-entropy claims: a public hash can otherwise be tested against guesses.

## Canonical commitment

The contract and backend helper hash the Soroban XDR encoding of this tuple, in
this exact order:

1. `1_u32`, the commitment encoding version.
2. `Symbol("earnproof_consent_receipt")`, the domain separator.
3. The Soroban network ID (`BytesN<32>`).
4. The proof-registry contract address.
5. `proof_id_hash` (`BytesN<32>`).
6. `policy_hash` (`BytesN<32>`).
7. `receipt_version` (`u32`).
8. `receipt_hash` (`BytesN<32>`).

The resulting SHA-256 digest is the `commitment_hash`. Including network,
registry, proof, policy, and receipt versions prevents a commitment from being
replayed into another network, registry, proof, or policy version. Backend
implementations should use
`earnproof_shared::disclosure_consent_commitment` rather than recreating the
encoding.

## Storage and events

Multiple consent receipts are independently indexed by their commitment hash
in persistent storage. The stored value is only `true`; the receipt hash is not
stored. Recommitting the same inputs returns
`ConsentReceiptAlreadyCommitted`. A different receipt hash or receipt version
produces a separate index entry.

On success the contract emits `consent_receipt_committed` with only
`proof_id_hash`, `policy_hash`, `receipt_version`, and `commitment_hash`. The
event excludes `receipt_hash`, nonce, verifier identity, and claim values.
`has_consent_receipt_commitment` is a public read-only membership check.

The proof must exist, be unrevoked, and not be expired. Only its registered
issuer may commit a receipt. A failed call writes no index entry and emits no
event. Revocation is terminal; an archived commitment can be restored by the
ledger when accessed, while a commitment that has expired and been removed no
longer prevents a fresh write.

## Consent withdrawal records

`withdraw_disclosure_consent(proof_id_hash, policy_hash, expected_sequence,
reason_code, withdrawal_hash)` lets the proof's issuer commit a hash-only record
that future disclosure consent for one proof and policy was withdrawn. The call
returns a `ConsentWithdrawalRecord`.

* `expected_sequence` must equal the next withdrawal sequence (the current
  `get_consent_withdrawal_count()` plus one). A replayed or raced request fails
  with `StaleConsentWithdrawalSequence` and writes nothing.
* `reason_code` is a published code: 1 subject request, 2 policy change,
  3 issuer action, 4 expired by agreement. Anything else fails with
  `InvalidConsentWithdrawalReason`. Reason text is never accepted or stored.
* `withdrawal_hash` is a caller-side salted hash. The stored
  `commitment_hash` is scoped to the network id and registry address and binds
  the proof, policy, sequence, reason code and `withdrawal_hash`, so the same
  withdrawal cannot be replayed on another network or registry.
* A pair is withdrawn at most once (`ConsentAlreadyWithdrawn`). Withdrawal is
  final and is allowed for revoked or expired proofs.
* After withdrawal, `commit_disclosure_consent` for the same proof and policy
  fails with `ConsentWithdrawn`. Earlier receipts are not rewritten or removed,
  and other policies on the same proof are unaffected.

Queries: `is_consent_withdrawn`, `get_consent_withdrawal`,
`get_consent_withdrawal_count`, and `get_consent_withdrawals(from_sequence,
limit)` (limit 1 to 20, oldest first). The event is `consent_withdrawn` with
`proof_id_hash`, `policy_hash`, `sequence`, `reason_code`, `commitment_hash`.

Storage: `ConsentWithdrawal(sequence)`, `ConsentWithdrawalIndex(proof, policy)`
(persistent) and `ConsentWithdrawalCount` (instance).

# History retention bounds and event sequence ranges

## Retention bounds (protocol-config)

Two bounded histories can be given a governed retention cap:

| Kind | Minimum | Default and maximum |
|---|---|---|
| `ConfigChange` | 8 | 32 (the physical ring size) |
| `PauseAcknowledgement` | 16 | 1024 |

`set_history_retention(kind, max_entries, expected_revision)` is admin-only and
returns the new `HistoryRetention` view. `get_history_retention(kind)` reads it.

* Values outside the range fail with `InvalidInput`; the revision does not move.
* `expected_revision` must equal the current revision. A replayed, stale or
  competing update fails with `InvalidState`, as does re-submitting the cap
  already in force. Each accepted update increments the revision by one.
* Sequence numbers are never renumbered. The retained window is
  `[max(next - cap, pruned_to), next)`. `pruned_to` only moves forward, so
  raising the cap later never brings back entries that were removed.
* Reads (`get_config_history`, `get_pause_acknowledgements`) clamp a cursor that
  points before the window to the oldest retained entry, so a page never has a
  hole.
* Lowering a cap narrows the window immediately but removes at most 8 stored
  entries per call, bounding the work of one transaction. Remaining old entries
  are removed by later writes. The pause-acknowledgement hash index is kept
  after its history entry is pruned, so a commitment still cannot be replayed.
* Each accepted change emits `history_retention_set` and is recorded in the
  change history as category `HistoryRetention`.

Defaults equal the previous fixed behaviour, so existing deployments see no
change until governance lowers a cap.

## Event sequence ranges

`get_event_sequence_range(domain)` returns an `EventSequenceRange`:

* `latest_emitted`: the count of events ever committed in the domain
  (one-based; `0` while empty).
* `first_retained`: the lowest sequence whose record is still queryable (`0`
  while empty).
* `checkpoint`: `latest_emitted` rounded down to a multiple of 16, a fixed
  anchor for indexers to compare.

Domains: `ConfigChange` and `PauseAcknowledgement` on protocol-config,
`ConsentWithdrawal` on proof-registry. A contract asked for a domain it does not
emit returns `UnsupportedEventDomain` (proof-registry) or `InvalidInput`
(protocol-config). Sequences only advance in the transaction that writes the
event, so failed or rolled-back calls leave the range unchanged.

## Proof policy version summaries (proof-registry)

`get_proof_policy_versions(proof_id_hash)` returns a `ProofPolicyVersions`
summary of the versions bound when a proof was issued through the typed
registration path: schema version, proof type, issuer policy encoding version,
commitment algorithm, nullifier domain version (`0` without a nullifier),
config digest version and the binding ledger. It holds versions only, never
claims. The summary is immutable; `register_proof_with_nullifier` fills in the
nullifier domain at registration.

Proofs registered through paths that do not store a summary (activation-delayed,
payload-less legacy and batch registration) are reported with `legacy = true`,
with schema version, proof type and nullifier domain read from the immutable
record and the other versions reported as `0`. The versions are the ones
compiled into the registry at registration time; the registry does not call
other contracts to look them up, so they describe the registry's own encoding
versions rather than a live governance setting.

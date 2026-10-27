# Decommission Grace Periods

`activate_successor` decommissions a contract at once. A retiring contract
often needs a bounded window in which clients can still read it and discover
the successor, while no new state can be written. Every contract supports:

| Entrypoint | Result | Authorization |
|---|---|---|
| `begin_decommission_grace(grace_ledgers: u32)` | `Result<DecommissionState, ContractError>` | admin |
| `finalize_decommission()` | `Result<DecommissionState, ContractError>` | admin |
| `get_decommission_state()` | `DecommissionState` | none |

`issuer-registry` takes a leading single-use `proposal_id: BytesN<32>` on both
mutating calls, like its `nominate_successor` and `activate_successor`. A zero
id fails with `InvalidInput`; a reused id fails with `AlreadyExists`.

## Phases

```text
Active --begin_decommission_grace--> Grace --finalize_decommission--> Finalized
Active --activate_successor (immediate, unchanged)-----------------> Finalized
```

| Phase | Allowed |
|---|---|
| `Active` | everything the contract normally allows |
| `Grace` | reads, TTL keepalives, `finalize_decommission`, and the entrypoints that remain open after final decommission |
| `Finalized` | the existing decommissioned behaviour |

During grace a contract rejects exactly the entrypoints it rejects after final
decommission, through the same guards: `proof-registry` returns
`ProofNotFound`, `protocol-config` returns `InvalidState`, and
`issuer-registry` returns `InvalidTransition` where it returns that after
decommission and otherwise panics with "contract is in decommission grace"
where it already panics after decommission. It also
rejects `set_deployment_domain`, layout migration steps, a new successor
nomination, `activate_successor`, and any second `begin_decommission_grace`,
so the window and successor are fixed. Entrypoints that a finalized contract
still accepts stay open in grace, which keeps permissions monotonic:
`Active ⊇ Grace ⊇ Finalized`.

## Rules

- **Start.** Requires a nominated successor (`NotFound` otherwise), no pending
  layout migration and no incomplete data migration (`InvalidState`), and
  `1 <= grace_ledgers <= MAX_DECOMMISSION_GRACE_LEDGERS` (518,400 ledgers,
  about 30 days). Out-of-range windows fail with `InvalidTimingConfig`. The
  record stores `grace_started_ledger` and
  `grace_ends_ledger = grace_started_ledger + grace_ledgers`.
- **Finalize.** Allowed at `grace_ends_ledger` and later; one ledger earlier
  fails with `InvalidTimingConfig`. Without a grace record, or after
  decommission, it fails with `InvalidState`. It sets the decommissioned flag,
  records `finalized_ledger`, and publishes the contract's existing
  `contract_decommissioned` event.
- **Terminal.** No entrypoint clears the decommissioned flag or the grace
  record. Replays of `finalize_decommission` or `begin_decommission_grace`
  after finalization fail with `InvalidState` and change nothing.
- **Legacy.** A contract decommissioned through `activate_successor` reports
  `Finalized` with zero ledger fields and cannot enter grace.
- **Failures are atomic.** Every rejected call leaves the phase, window, and
  successor unchanged.

`DecommissionState` (`state_version = 1`) carries `phase`, `successor`,
`grace_started_ledger`, `grace_ends_ledger`, and `finalized_ledger`. Event:
`decommission_grace_started` (see [`events.md`](./events.md)).

Tests: [`tests/emergency/tests/decommission_grace.rs`](../tests/emergency/tests/decommission_grace.rs).

# Storage Layout Versions

The contract version says which code is deployed. The storage layout version
says which shape the stored state is in. Upgrade and migration tooling reads
both before acting.

| Entrypoint | Result | Authorization |
|---|---|---|
| `get_storage_layout()` | `StorageLayoutStatus` | none |
| `begin_layout_migration(target_layout: u32)` | `Result<StorageLayoutStatus, ContractError>` | admin |
| `complete_layout_migration(target_layout: u32)` | `Result<StorageLayoutStatus, ContractError>` | admin |

`StorageLayoutStatus` (`status_version = 1`):

- `active_layout`: the layout of the stored state. Deployments that predate the
  layout key report `1` (`LEGACY_STORAGE_LAYOUT`).
- `code_layout`: the layout the deployed code writes. All three contracts are
  at layout `2`.
- `migration_target`: the target of a started step, the next reviewed step
  when one is required, or `0`.
- `compatibility`:

| Value | Meaning |
|---|---|
| `Current` | `active_layout == code_layout` |
| `MigrationRequired` | older layout with a reviewed step; not started |
| `MigrationInProgress` | a step was started and has not completed |
| `Unsupported` | stored layout is newer than the code, or no reviewed path exists |

## Reviewed steps

`REVIEWED_LAYOUT_MIGRATIONS` in
[`packages/shared/src/storage_layout.rs`](../packages/shared/src/storage_layout.rs)
lists every allowed `(from, to)` step. Layout `1 -> 2` only adds instance keys
(`DeploymentDomain`, `DecommissionGrace`, `StorageLayoutVersion`,
`StorageLayoutMigration`) and rewrites nothing, so it needs no data batches.

## Rules

- `initialize` writes the code layout. Existing deployments stay at layout 1
  until migrated.
- `active_layout` moves only in `complete_layout_migration`, only by one
  reviewed step, and only to the target recorded by
  `begin_layout_migration`. An upgrade alone never moves it, and a started
  step reports `MigrationInProgress` with the old `active_layout`, so the
  status never implies a migration completed when it did not.
- `begin_layout_migration` fails with `InvalidState` unless the status is
  `MigrationRequired`, and with `InvalidInput` unless the target is the
  reported next reviewed step. Replaying it with the pending target returns
  the same status and publishes nothing; another target fails with
  `InvalidState`.
- `complete_layout_migration` fails with `InvalidState` without a matching
  pending target. Replaying it after completion fails with `InvalidState` and
  changes nothing.
- Neither step runs while a data migration (`begin_migration` /
  `advance_migration`) is incomplete, during decommission grace, or after
  decommission (`InvalidState`).
- A future step that rewrites data must drive its `MigrationStatus` to
  `complete` before `complete_layout_migration` can succeed.

Events: `storage_layout_migration_started` and `storage_layout_migrated` (see
[`events.md`](./events.md)).

Tests: [`tests/migrations/tests/storage_layout.rs`](../tests/migrations/tests/storage_layout.rs).

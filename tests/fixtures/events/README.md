# Contract Event Fixtures

Versioned, machine-readable event fixtures for EarnProof Soroban contracts. These fixtures enable backend indexers to parse, validate, and react to on-chain events with stable serialization guarantees.

## Directory Structure

```text
tests/fixtures/events/
  schema.json                       JSON Schema for fixture validation
  README.md                         This file
  protocol-config/
    v1/
      initialized.json
      admin-changed.json
      paused.json
      unpaused.json
      schema-approved.json
      schema-deprecated.json
      history-retention-set.json
  issuer-registry/
    v1/
      events.json                   No events emitted yet
  proof-registry/
    v1/
      consent-receipt-committed.json Consent commitment event
      consent-withdrawn.json         Consent withdrawal event
```

## Fixture Format

Each fixture is a JSON file conforming to `schema.json` with these fields:

| Field               | Description                                                        |
| ------------------- | ------------------------------------------------------------------ |
| `event`             | Event struct name matching the `#[contractevent]` identifier       |
| `contract`          | Contract crate name (`protocol-config`, `issuer-registry`, `proof-registry`) |
| `contract_version`  | Semver version of the contract crate at generation time            |
| `schema_version`    | Monotonically increasing fixture schema version                    |
| `topics`            | Indexed Soroban event topics (first is the event discriminant)     |
| `payload`           | Non-indexed payload fields with types, descriptions, and examples  |
| `emitted_by`        | Public contract function that emits the event                      |
| `compatibility`     | Change classification: `stable`, `additive`, or `breaking`         |

## Versioning Rules

Fixture schema versions follow these compatibility guarantees:

- **stable**: No structural change. The same fixture applies across contract versions.
- **additive**: New optional payload fields may appear. Existing fields and topics are unchanged. Indexers that ignore unknown fields remain compatible.
- **breaking**: A field was removed, renamed, or its type changed. Indexers must update parsing logic.

### Upgrade policy

1. Bump `schema_version` on every structural change.
2. Set `compatibility` to `additive` when extending payload without removing fields.
3. Set `compatibility` to `breaking` when removing or renaming fields or topics.
4. Create a new version directory (e.g. `v2/`) when the breaking change cannot coexist with the previous version.

## Generating Fixtures

Fixtures are generated deterministically from contract test runs. The validation test in each contract's test module:

1. Asserts that every emitted event has a corresponding fixture file.
2. Asserts that fixture topics and payload fields match the event definition.
3. Asserts that fixture `contract_version` matches the crate version.

To regenerate fixtures after a contract change:

```bash
cargo test --workspace
```

The test suite will panic with a descriptive message if fixtures drift out of sync with the contract code.

## Privacy Boundary

Fixtures contain only public event schemas and example values. No fixture contains:

- Private income or payment data
- Personal identity information
- Raw transaction lists
- Unencrypted personal data

Example `Address` values in fixtures are test-only Stellar public keys used in the contract test suite.

## Usage by Indexers

Backend indexers should:

1. Load the fixture files at startup or after deployment.
2. Match incoming Soroban events by topic[0] against fixture `topics[0]`.
3. Deserialize payload fields using the types defined in `payload`.
4. Reject events whose `schema_version` exceeds the maximum fixture version to avoid silent data loss.

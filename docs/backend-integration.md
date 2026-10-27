# Backend Integration

This document lists the contract calls the EarnProof API should use when writing proof commitments, reading issuer status, and validating public proof state.

**See Also:** [Executable Examples](./executable-examples.md) for runnable demonstrations of all contract invocation patterns.

## Error Handling

All contracts return typed Soroban error codes instead of panic strings. Backend integrations must map these machine-readable codes to appropriate HTTP status codes and user-facing messages.

The authoritative catalog - every code with its cause, retry classification, remediation, and suggested HTTP status - is [Contract Error Catalog](./errors.md), generated from [`packages/shared/src/error_catalog.rs`](../packages/shared/src/error_catalog.rs). The tables in this section are a summary of it; where they differ, the catalog is correct.

### Error Code Ranges

Error codes are allocated to prevent collisions across contracts:

- **Common errors (1-99)**: Shared across all contracts
- **Protocol Config errors (100-199)**: Allocated but currently unused; protocol-config returns common errors only
- **Issuer Registry errors (200-299)**: Issuer-specific errors
- **Proof Registry errors (300-399)**: Proof-specific errors

### Handling unknown codes

A backend outlives the release it was written against. It will eventually receive a code that did not exist at compile time, and the one thing it must not do is guess. Attribute the code by range, refuse to retry it, and surface it as an incomplete request rather than as a rejection:

```rust
use earnproof_shared::error_catalog::{domain_for, spec, Retry};

pub struct Decision {
    pub http_status: u16,
    pub message: &'static str,
    pub retry: Retry,
    pub recognised: bool,
}

pub fn decide(code: u32) -> Decision {
    match spec(code) {
        Some(entry) => Decision {
            http_status: entry.http_status,
            message: entry.client_message,
            retry: entry.retry,
            recognised: true,
        },
        None => {
            // Log `domain_for(code)` and the raw code for triage. Do not map an
            // unknown code onto a known one, and do not retry it: the call may
            // have had a side effect this build cannot reason about.
            let _domain = domain_for(code);
            Decision {
                http_status: 502,
                message: "Request could not be completed",
                retry: Retry::Never,
                recognised: false,
            }
        }
    }
}
```

The same shape in TypeScript, for a backend that does not link the Rust crate:

```ts
const DOMAINS = [
  { lo: 1, hi: 99, name: "common" },
  { lo: 100, hi: 199, name: "protocol-config" },
  { lo: 200, hi: 299, name: "issuer-registry" },
  { lo: 300, hi: 399, name: "proof-registry" },
] as const;

export function domainFor(code: number): string {
  return DOMAINS.find((d) => code >= d.lo && code <= d.hi)?.name ?? "unallocated";
}

export function decide(code: number) {
  const known = CATALOG[code]; // generated from docs/errors.md
  if (known) return known;
  return {
    httpStatus: 502,
    message: "Request could not be completed",
    retry: "never",
    recognised: false,
    domain: domainFor(code),
  };
}
```

Three rules make this safe:

1. **Switch on the number, never on the variant name.** Names are for logs. Codes are the contract.
2. **Never widen an unknown code into a known one.** An unrecognised `3xx` is not "some proof error we can treat like 301".
3. **Never retry automatically on an unrecognised code.** Only `after-operator-action` codes are safe to repeat, and only alongside the corresponding read.

### Retry classification

| Retry | Codes | What the backend should do |
|---|---|---|
| `never` | 1, 20, 40, 42, 200, 202, 204, 206, 300, 302 | Surface the failure. Do not retry on any schedule. |
| `after-operator-action` | 2, 80, 205, 304, 305 | Poll the relevant read (`is_paused`, `is_active_address`, `is_schema_version_approved`) and retry the write only once it flips. |
| `after-caller-change` | 41, 60, 201, 203, 301, 303 | Fix the request. The identical request will fail identically. |
| unknown code | anything else | Treat as `never`. Log the raw code and its range. |

### Codes you will not see in this release

Six codes are declared and reserved but returned by no contract path today: `20 Unauthorized`, `40 AlreadyExists`, `41 NotFound`, `42 InvalidState`, `80 ProtocolPaused`, and `205 IssuerInactive`. Handle them for forward compatibility, but do not build a control flow that waits for one. In particular:

- **A paused protocol surfaces as `304`, not `80`.** Poll `is_paused` to detect a pause.
- **A suspended issuer surfaces as `304`, not `205`.** Call `is_active_address` before registering.
- **Authorization failures abort the invocation** through the host auth check rather than returning `20`. Treat a host authorization abort and code `20` as the same outcome.

See the overloaded-code note in [Contract Error Catalog](./errors.md) for the full explanation.

### Common Contract Errors (1-99)

| Code | Error Name | Description | Suggested HTTP Status | Safe API Response |
|------|------------|-------------|----------------------|-------------------|
| 1 | AlreadyInitialized | Contract already initialized | 409 Conflict | "Contract is already initialized" |
| 2 | NotInitialized | Contract not initialized | 500 Internal Server Error | "Service temporarily unavailable" |
| 20 | Unauthorized | Caller lacks required authorization (reserved) | 403 Forbidden | "Insufficient permissions" |
| 40 | AlreadyExists | Resource already exists (reserved) | 409 Conflict | "Resource already exists" |
| 41 | NotFound | Resource not found (reserved) | 404 Not Found | "Resource not found" |
| 42 | InvalidState | Operation invalid for current state (reserved) | 400 Bad Request | "Operation not permitted in current state" |
| 60 | InvalidInput | Invalid input parameters | 400 Bad Request | "Invalid input provided" |
| 80 | ProtocolPaused | Protocol is paused (reserved) | 503 Service Unavailable | "Service temporarily paused" |

### Issuer Registry Errors (200-299)

| Code | Error Name | Description | Suggested HTTP Status | Safe API Response |
|------|------------|-------------|----------------------|-------------------|
| 200 | IssuerAlreadyRegistered | Issuer ID already registered | 409 Conflict | "Issuer already registered" |
| 201 | IssuerNotFound | Issuer not found | 404 Not Found | "Issuer not found" |
| 202 | IssuerAddressAlreadyRegistered | Issuer address already in use | 409 Conflict | "Issuer address already registered" |
| 203 | IssuerAddressNotFound | Issuer address not found | 404 Not Found | "Issuer address not found" |
| 204 | IssuerRevoked | Issuer has been revoked | 403 Forbidden | "Issuer has been revoked" |
| 205 | IssuerInactive | Issuer is not active (reserved) | 403 Forbidden | "Issuer is not active" |
| 206 | InvalidTransition | Invalid status transition | 400 Bad Request | "Invalid status transition" |

### Proof Registry Errors (300-399)

| Code | Error Name | Description | Suggested HTTP Status | Safe API Response |
|------|------------|-------------|----------------------|-------------------|
| 300 | ProofAlreadyRegistered | Proof ID already registered | 409 Conflict | "Proof already registered" |
| 301 | ProofNotFound | Proof not found, or registry uninitialized | 404 Not Found | "Proof not found" |
| 302 | ProofAlreadyRevoked | Proof has already been revoked | 400 Bad Request | "Proof already revoked" |
| 303 | ProofExpired | Proof expiration is invalid | 400 Bad Request | "Invalid proof expiration" |
| 304 | InvalidSchemaVersion | Schema version invalid, protocol paused, or issuer inactive | 400 Bad Request | "Invalid schema version" |
| 305 | SchemaVersionNotApproved | Schema version not approved | 400 Bad Request | "Schema version not approved" |

### Error Handling Best Practices

1. **Never expose raw contract errors to end users**: Map error codes to safe, user-friendly messages
2. **Log full error context server-side**: Include contract error codes, transaction IDs, and context for debugging
3. **No sensitive data in error messages**: Do not include personal information, wallet addresses, or internal IDs in user-facing errors
4. **Consistent HTTP status codes**: Use the suggested HTTP status codes for consistency
5. **Graceful degradation**: Handle unexpected error codes gracefully with generic error messages, and never by widening them into a code you do recognise

## Protocol Config

Contract responsibility:

- Store protocol administrator.
- Store pause state.
- Store approved schema versions.
- Expose a configuration version counter.

Backend reads:

```text
get_admin() -> Address
is_paused() -> bool
is_schema_version_approved(version: u32) -> bool
get_config_version() -> u32
```

Backend writes:

```text
approve_schema_version(version: u32)
deprecate_schema_version(version: u32)
pause()
unpause()
```

Admin authorization is required for writes.

### Protocol Config Events

The protocol-config contract emits typed events on every state mutation. Backend indexers should subscribe to these topics for real-time protocol lifecycle tracking.

| Event | Topic | Payload | Emitted by | Fixture |
| --- | --- | --- | --- | --- |
| Initialized | `Initialized` | `admin: Address` | `initialize` | [initialized.json](../tests/fixtures/events/protocol-config/v1/initialized.json) |
| AdminChanged | `AdminChanged` | `new_admin: Address` | `set_admin` | [admin-changed.json](../tests/fixtures/events/protocol-config/v1/admin-changed.json) |
| Paused | `Paused` | `paused: bool` | `pause` | [paused.json](../tests/fixtures/events/protocol-config/v1/paused.json) |
| Unpaused | `Unpaused` | `paused: bool` | `unpause` | [unpaused.json](../tests/fixtures/events/protocol-config/v1/unpaused.json) |
| SchemaApproved | `SchemaApproved` | `version: u32` | `approve_schema_version` | [schema-approved.json](../tests/fixtures/events/protocol-config/v1/schema-approved.json) |
| SchemaDeprecated | `SchemaDeprecated` | `version: u32` | `deprecate_schema_version` | [schema-deprecated.json](../tests/fixtures/events/protocol-config/v1/schema-deprecated.json) |

See [tests/fixtures/events/](../tests/fixtures/events/) for the fixture schema, versioning rules, and compatibility guarantees.

## Issuer Registry

Contract responsibility:

- Store approved issuer records.
- Store issuer status.
- Store public metadata hash.
- Rotate issuer wallet addresses.
- Resolve issuer records by ID hash or Stellar address.

Backend reads:

```text
get_issuer(issuer_id_hash: BytesN<32>) -> IssuerRecord
get_issuer_by_address(issuer_address: Address) -> IssuerRecord
is_active_issuer(issuer_id_hash: BytesN<32>) -> bool
is_active_address(issuer_address: Address) -> bool
```

Backend writes:

```text
register_issuer(issuer_id_hash: BytesN<32>, issuer_address: Address, metadata_hash: BytesN<32>)
update_issuer(issuer_id_hash: BytesN<32>, metadata_hash: BytesN<32>)
suspend_issuer(issuer_id_hash: BytesN<32>)
reactivate_issuer(issuer_id_hash: BytesN<32>)
revoke_issuer(issuer_id_hash: BytesN<32>)
rotate_issuer_address(issuer_id_hash: BytesN<32>, new_address: Address)
```

Admin authorization is required for writes.

### Issuer Registry Events

Issuer-registry mutations emit typed Soroban events (delivered by #2). See [tests/fixtures/events/issuer-registry/](../tests/fixtures/events/issuer-registry/) for the fixture payloads.
Every successful mutation emits exactly one typed event. Failed, unauthorized, or duplicate operations emit no success event.

All payloads contain only public hashes, addresses, status, and timestamps. No personal data, salary, or payment amounts are included.

#### `IssuerRegistered`

Emitted when an issuer is successfully registered for the first time.

Topic: `issuer_registered`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `issuer_id_hash` | `BytesN<32>` | SHA-256 hash of the issuer's internal ID |
| `issuer_address` | `Address` | On-chain Stellar wallet address |
| `metadata_hash` | `BytesN<32>` | SHA-256 hash of the issuer's public metadata |
| `created_at` | `u64` | Ledger timestamp at registration time |

#### `IssuerMetadataUpdated`

Emitted when the issuer's public metadata hash is replaced.

Topic: `issuer_metadata_updated`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `issuer_id_hash` | `BytesN<32>` | SHA-256 hash of the issuer's internal ID |
| `metadata_hash` | `BytesN<32>` | New SHA-256 hash of the issuer's public metadata |
| `updated_at` | `u64` | Ledger timestamp at update time |

#### `IssuerSuspended`

Emitted when an active or previously-suspended issuer is suspended.

Topic: `issuer_suspended`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `issuer_id_hash` | `BytesN<32>` | SHA-256 hash of the issuer's internal ID |
| `updated_at` | `u64` | Ledger timestamp at suspension time |

#### `IssuerReactivated`

Emitted when a suspended issuer is restored to active status.

Topic: `issuer_reactivated`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `issuer_id_hash` | `BytesN<32>` | SHA-256 hash of the issuer's internal ID |
| `updated_at` | `u64` | Ledger timestamp at reactivation time |

#### `IssuerRevoked`

Emitted when an issuer is permanently revoked. Revocation is irreversible.

Topic: `issuer_revoked`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `issuer_id_hash` | `BytesN<32>` | SHA-256 hash of the issuer's internal ID |
| `updated_at` | `u64` | Ledger timestamp at revocation time |

#### `IssuerAddressRotated`

Emitted when the issuer's on-chain wallet address is rotated to a new address. Both old and new addresses are included so indexers can update their mapping atomically without scanning storage.

Topic: `issuer_address_rotated`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `issuer_id_hash` | `BytesN<32>` | SHA-256 hash of the issuer's internal ID |
| `old_address` | `Address` | Previous on-chain wallet address |
| `new_address` | `Address` | Replacement on-chain wallet address |
| `updated_at` | `u64` | Ledger timestamp at rotation time |

## Proof Registry

Contract responsibility:

- Store proof commitment records.
- Reject duplicate proof IDs.
- Reject expired proof registrations.
- Revoke proof records.
- Expose issuer registry and protocol config contract references.

Backend reads:

```text
get_proof(proof_id_hash: BytesN<32>) -> ProofRecord
is_valid_proof(proof_id_hash: BytesN<32>) -> bool
is_revoked(proof_id_hash: BytesN<32>) -> bool
get_issuer_registry() -> Address
get_protocol_config() -> Address
```

Backend writes:

```text
register_proof(
  proof_id_hash: BytesN<32>,
  commitment_hash: BytesN<32>,
  issuer_address: Address,
  schema_version: u32,
  expires_at: u64
)
revoke_proof(proof_id_hash: BytesN<32>)
admin_revoke_proof(proof_id_hash: BytesN<32>)
```

Issuer authorization is required for normal proof registration and revocation. Admin authorization is required for administrative revocation.

### Proof Registry Events

The proof-registry contract currently emits no typed events. Proof lifecycle changes (registration, revocation) are stored on-chain via `ProofRecord` updates but are not announced via Soroban event topics. Future contract versions are expected to add events for proof registration and revocation. See [tests/fixtures/events/proof-registry/](../tests/fixtures/events/proof-registry/).
Every successful mutation emits its documented typed event or events. Failed,
unauthorized, duplicate, expired, or paused-protocol operations emit no success
event.

#### Issuer policy and issuance configuration snapshots

`set_issuer_policy_commitments` preserves the current commitment getter and
adds a monotonic per-issuer version. `get_issuer_policy_version` returns `0`
for a legacy issuer that has not yet received a versioned update;
`get_issuer_policy_commitment_at_version` and `get_issuer_policy_history`
query the bounded 16-version retained history. Each history record contains
only the two opaque commitments, version, effective ledger, and effective
timestamp. Failed updates do not advance the version or emit an event.

Successful proof registration captures the proof, issuer-registry, and
protocol-config identities/digests, issuer policy version, and versioned
schema-policy digest in `ProofConfigurationCommitments`. The immutable
sidecar also retains the opaque issuer policy commitment pair, so the version
remains auditable after a bounded history slot is evicted. For legacy issuers
with policy version `0`, the pair uses an all-zero encoding-version/hash
sentinel. It is queryable
through `get_proof_configuration_commitments` and is
included in the existing registration event payload, so the invocation still
emits exactly one registration event. Legacy proofs return `None`.
Registration requires compatible issuer/protocol interfaces and available
versioned schema digest data.

#### Dispute evidence commitments

`append_dispute_evidence(proof_id_hash, submitter, algorithm, evidence_hash)`
accepts only an algorithm identifier and fixed-size evidence hash. The
submitter must be the dispute opener, proof issuer, or registry admin and
must authorize the call. Only an open dispute accepts additions; each dispute
round retains at most 16 entries. `get_dispute_sequence` returns the active
round identifier, which is supplied to the bounded cursor-based
`get_dispute_evidence_history` query. Records expose submitter role, algorithm,
sequence, ledger, and timestamp. Evidence bytes, URLs, and claim text are not
part of the contract interface.

#### Dispute decisions

`resolve_dispute(proof_id_hash, reason)` and
`reject_dispute(proof_id_hash, reason)` require a
`DisputeDecisionReason { version: 1, code, reason_commitment }`. Version 1 uses
code `1` for resolution and codes `2` through `5` for rejection (claim
unsubstantiated, duplicate, out of scope, and procedural). The optional
`reason_commitment` is an opaque 32-byte commitment to rationale held off
chain; callers must never submit evidence or free-form reason text.

Successful decisions retain the legacy `dispute_resolved` or
`dispute_rejected` event and then emit `dispute_decision_committed_v2` with the
reason version/code, optional commitment, resolver address/role, ledger, and
timestamp. `get_dispute_decision_metadata(proof_id_hash)` returns `None` for
open, withdrawn, and legacy terminal disputes. New dispute records clear any
prior decision sidecar. Existing serialized `DisputeRecord` values remain
readable without migration, but the required decision arguments change the
invocation ABI and require coordinated backend deployment.

All payloads contain only public hashes, addresses, schema version, timestamps, and expiration. No payment amounts, wallet history, personal names, or raw credential data are included.

#### `ProofRegistered`

Emitted when a proof commitment is successfully registered.

Topic: `proof_registered`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `proof_id_hash` | `BytesN<32>` | SHA-256 hash of the proof's internal ID |
| `commitment_hash` | `BytesN<32>` | SHA-256 hash of the canonical credential payload (without signature) |
| `issuer_address` | `Address` | On-chain address of the issuer that registered the proof |
| `schema_version` | `u32` | Approved schema version used for this proof |
| `expires_at` | `u64` | Ledger timestamp after which the proof is no longer valid |
| `created_at` | `u64` | Ledger timestamp at registration time |

#### `ProofRevokedByIssuer`

Emitted when the issuer that originally registered a proof revokes it. Distinguishable from admin revocation by the event name `proof_revoked_by_issuer`.

Topic: `proof_revoked_by_issuer`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `proof_id_hash` | `BytesN<32>` | SHA-256 hash of the proof's internal ID |
| `issuer_address` | `Address` | On-chain address of the revoking issuer |
| `revoked_at` | `u64` | Ledger timestamp at revocation time |

#### `ProofRevokedByAdmin`

Emitted when an admin revokes a proof. Distinguishable from issuer revocation by the event name `proof_revoked_by_admin`.

Topic: `proof_revoked_by_admin`

Payload fields:

| Field | Type | Description |
|---|---|---|
| `proof_id_hash` | `BytesN<32>` | SHA-256 hash of the proof's internal ID |
| `admin_address` | `Address` | On-chain address of the admin that performed the revocation |
| `revoked_at` | `u64` | Ledger timestamp at revocation time |

## Event Replay and Indexer Expectations

- Events are emitted only on success. Any panic or authorization failure before state mutation guarantees no success event is emitted.
- Each mutation emits at most one event. Indexers should not expect batched or partial emissions.
- Topics follow the snake_case convention derived automatically from the struct name by the `#[contractevent]` macro. No custom topic overrides are applied.
- Indexers can identify the actor type for revocations from the topic alone (`proof_revoked_by_issuer` vs `proof_revoked_by_admin`) without decoding the payload.
- Event data is encoded as a Soroban `Map` with field name keys in alphabetical order (the default `data_format = "map"` behavior of `#[contractevent]`).
- To replay from genesis, query the Horizon or RPC event endpoint for the contract address and filter by topic. Events are permanently available at the ledger they were emitted and do not expire.
- Address rotation events include both `old_address` and `new_address` so indexers can rebuild the address-to-issuer mapping without reading contract storage.

## Hashing Rules

The backend should hash public identifiers before passing them to contracts:

```text
proof_id_hash = sha256(proof_id)
issuer_id_hash = sha256(issuer_id)
commitment_hash = sha256(canonical_credential_payload_without_signature)
metadata_hash = sha256(canonical_public_issuer_metadata)
```

## On-Chain Data Boundary

Do not send exact income, raw transaction lists, personal names, emails, or full wallet history to contracts. Store only hashes, status, schema version, issuer address, expiration, and timestamps.

For the complete list of every storage key, TTL policy, lifecycle event, and privacy boundary see the [Storage Model](./storage-model.md) reference.


### Revocation Checkpoints

Indexers can verify revocation drift using the `/api/v1/registry/revocation-checkpoint` query. 
The checkpoint commitment is a deterministic Sha256 chained hash.

**Verification Algorithm (Pseudocode):**
```python
def verify_checkpoint(events, expected_root):
    current_root = ""
    for event in events:
        if event.type in ["revoke", "archive"]:
            hasher = sha256()
            hasher.update(current_root)
            hasher.update(event.proof_id)
            hasher.update(event.ledger_height)
            current_root = hasher.hexdigest()
    assert current_root == expected_root
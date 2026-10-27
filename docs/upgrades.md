# Contract Upgrade and Migration Strategy

This document records the upgradeability decision, threat model, compatibility
rules, and rollback boundaries for each EarnProof Soroban contract.  It is the
authoritative reference for anyone performing or reviewing an upgrade.

---

## Table of Contents

1. [Upgrade mechanism overview](#1-upgrade-mechanism-overview)
2. [Per-contract decisions](#2-per-contract-decisions)
   - [protocol-config](#21-protocol-config)
   - [issuer-registry](#22-issuer-registry)
   - [proof-registry](#23-proof-registry)
3. [Governance model](#3-governance-model)
4. [Storage schema versioning and compatibility rules](#4-storage-schema-versioning-and-compatibility-rules)
5. [Deployment manifest provenance](#5-deployment-manifest-provenance)
6. [Rollback boundaries](#6-rollback-boundaries)
7. [Emergency pause procedure](#7-emergency-pause-procedure)
8. [Dry-run rehearsal procedure](#8-dry-run-rehearsal-procedure)
9. [Step-by-step upgrade procedure](#9-step-by-step-upgrade-procedure)

---

## 1. Upgrade mechanism overview

All three contracts use **in-place WASM upgrade** via Soroban's built-in
`env.deployer().update_current_contract_wasm(new_wasm_hash)`.  This replaces
the executable code at the existing contract address while leaving all
persistent and instance storage intact.  No state migration, no new contract
ID, no client reconfiguration.

Redeployment (fresh contract address) was explicitly ruled out for all three
contracts because:

- Each contract holds persistent state (issuer records, proof records, schema
  version approvals) that cannot be atomically migrated on-chain.
- Clients and the backend reference contracts by their stable Stellar contract
  IDs, recorded in `scripts/deployment-manifest.*.json`.
- Redeployment would require a coordinated cutover window and risk data loss or
  a gap in liveness — an unnecessary cost when Soroban's WASM update path
  preserves both.

### Upgrade API added to every contract

| Function | Who can call | Effect |
|---|---|---|
| `approve_upgrade(wasm_hash, new_version)` | Admin | Adds hash → version mapping to allowlist in instance storage; emits `UpgradeAllowlisted` |
| `revoke_upgrade(wasm_hash)` | Admin | Removes hash from allowlist; emits `UpgradeRevoked` |
| `is_upgrade_allowed(wasm_hash)` | Anyone | Read-only check |
| `upgrade_contract(wasm_hash)` | Admin | Verifies allowlist, checks version guard, applies WASM update, advances `ContractVersion`, emits `ContractUpgraded` |
| `get_contract_version()` | Anyone | Returns current monotonic `ContractVersion` |

---

## 2. Per-contract decisions

### 2.1 protocol-config

**Decision: in-place WASM upgrade**

**Rationale:**  
Protocol-config holds only instance storage (admin address, pause flag,
`ConfigVersion` counter, and schema version approvals in persistent storage).
Its state is low-volume and self-contained.  In-place upgrade is safe and
sufficient.

**Threat model:**

| Threat | Mitigation |
|---|---|
| Unauthorized upgrade by a non-admin | `upgrade_contract` requires admin `require_auth`; rejected on-chain without admin signature |
| Arbitrary WASM installed (supply-chain attack) | Only hashes explicitly pre-approved via `approve_upgrade` can be applied |
| Schema version approvals wiped by a buggy upgrade | `ContractVersion` guard prevents downgrade; persistent storage is untouched by WASM swap |
| Pause state flipped during upgrade | Optional `--PauseProtocol` flag in `upgrade-contract.ps1` holds the pause for the upgrade window |
| Admin key compromise | Blast radius: full protocol control — admin can pause, approve arbitrary schema versions, and trigger upgrades.  Key rotation via `set_admin` should be done immediately on any suspected compromise |

**Blast radius:** If the admin key is compromised and an upgrade is applied, an
attacker could install malicious logic that approves arbitrary schema versions
or bypasses the pause gate.  This would affect all downstream proof
registrations.  Mitigated by: allowlist requirement (attacker must also upload a
WASM before it can be applied), version guard (downgrade cannot be silent), and
event emission (observable on-chain).

---

### 2.2 issuer-registry

**Decision: in-place WASM upgrade**

**Rationale:**  
Issuer-registry stores issuer records and address→ID mappings in persistent
storage.  These records are referenced by proof-registry at proof registration
time.  In-place upgrade preserves all records and their on-chain addresses
without any migration step.

**Threat model:**

| Threat | Mitigation |
|---|---|
| Unauthorized upgrade | Admin auth required |
| Arbitrary WASM installed | WASM hash allowlist enforced |
| Issuer records corrupted by upgraded logic | ContractVersion guard; upgrade must advance version; existing records in persistent storage are untouched by WASM swap |
| Issuer lookup broken during upgrade window | WASM swap is a single atomic ledger operation; there is no gap |
| Address→ID index inconsistency after upgrade | Persistent storage layout is additive-only; no index keys are removed or retyped in this upgrade |

**Blast radius:** Admin key compromise allows unauthorized upgrade and potential
bypass of issuer status checks.  Downstream effect: proof-registry would accept
proofs from issuers that should be suspended or revoked.

---

### 2.3 proof-registry

**Decision: in-place WASM upgrade**

**Rationale:**  
Proof-registry stores immutable proof commitment records.  These records must
never be lost.  In-place upgrade is the only safe path; redeployment would
orphan all existing proof records.

Proof-registry also holds references to the issuer-registry and protocol-config
contract addresses in instance storage.  These cross-contract references survive
a WASM upgrade unchanged.  If the referenced contracts themselves are upgraded
(in-place), proof-registry continues to call them at the same addresses with no
reconfiguration needed.

**Threat model:**

| Threat | Mitigation |
|---|---|
| Unauthorized upgrade | Admin auth required |
| Arbitrary WASM installed | WASM hash allowlist enforced |
| Proof records lost | Persistent storage untouched by WASM swap; ContractVersion guard prevents downgrade |
| Upgraded logic bypasses schema version or issuer checks | Any WASM hash must be pre-approved; upgrade events are emitted and auditable |
| Cross-contract reference broken after upgrade | References stored in instance storage survive WASM swap |
| New proof registrations during upgrade window | Use `--PauseProtocol` to pause protocol-config, blocking `register_proof` for the upgrade window |

**Blast radius:** Admin key compromise allows bypass of proof validity logic,
potentially allowing invalid proofs to be registered or valid proofs to be
improperly revoked.  This is the highest-impact contract; extra care should be
taken with the admin key.

---

## 3. Governance model

**Who can upgrade:** The single admin address stored in each contract's instance
storage under `DataKey::Admin`.

**Two-step process:** Upgrade is intentionally two calls:
1. `approve_upgrade(wasm_hash, new_version)` — pre-authorizes the hash.
2. `upgrade_contract(wasm_hash)` — applies it.

This means the upgrade can be reviewed (and revoked) between the two steps.

**Allowlist consumption:** Once `upgrade_contract` succeeds, the allowlist
entry is removed.  The same hash cannot be applied a second time unless
explicitly re-approved.

**Admin rotation:** `set_admin` (protocol-config) or direct instance-storage
replacement (issuer-registry, proof-registry) requires admin auth.  Rotate the
admin key before any upgrade if the current key is suspected compromised.

---

## 4. Storage schema versioning and compatibility rules

### `ContractVersion` (u32, instance storage)

- Initialized to `1` in `initialize()`.
- Advanced to `new_version` by a successful `upgrade_contract` call.
- `approve_upgrade` rejects any `new_version ≤ current_version`, preventing
  a downgrade from being pre-approved.
- `upgrade_contract` re-checks `new_version > current_version` at apply time,
  providing a second guard even if the allowlist was populated out of order.

### Schema compatibility rules for WASM authors

When writing a new WASM version, the following rules must be respected to
preserve storage compatibility:

1. **Additive only for `DataKey` variants** — new variants may be added; existing
   variants must not be renamed, removed, or have their associated value type
   changed.  The `#[contracttype]` encoding is positional; changing a variant
   breaks deserialization of existing ledger entries.
2. **Additive only for `#[contracttype]` structs** — fields may be appended to
   structs stored in persistent storage (e.g. `IssuerRecord`, `ProofRecord`)
   only if the SDK version supports optional/default fields.  Field removal or
   reordering is never safe.
3. **Schema version gating in proof-registry** — if a new WASM introduces a new
   proof record structure, a corresponding new `schema_version` value must be
   approved in protocol-config before the new client sends proofs using it.
   Old schema versions remain approved unless explicitly deprecated.
4. **Client compatibility** — backends must read `get_contract_version()` after
   any upgrade and update their ABI bindings accordingly.  The backend must
   never submit a transaction that targets a function that no longer exists in
   the installed WASM.

### Preventing accidental downgrade at the client level

- The backend should record the `contractVersion` from the deployment manifest
  and refuse to operate against a contract whose `get_contract_version()` is
  lower than the known deployed version.
- Deployment manifests record `contractVersion` alongside `sha256` so any
  discrepancy between the manifest and the live on-chain version is immediately
  detectable.

---

## 5. Deployment manifest provenance

Every deployment and upgrade writes a manifest JSON file under `scripts/`.
Each WASM entry now includes:

```json
{
  "sha256": "<hex>",
  "contractVersion": 1,
  "buildMetadata": {
    "rustToolchain": "stable",
    "cargoPackageVersion": "0.1.0",
    "sorobanSdkVersion": "27.0.0",
    "buildProfile": "release",
    "gitCommit": "<sha>"
  }
}
```

This allows any deployed instance to be traced to its exact source commit,
build configuration, and on-chain version number.

`deploy-testnet.ps1` writes this automatically.  `upgrade-contract.ps1` writes
a separate upgrade manifest (`scripts/upgrade-manifest.<contract>.<timestamp>.json`)
with the uploaded WASM hash, new version, and the same build provenance fields.

---

## 6. Rollback boundaries

### What CAN be rolled back

| Scenario | Rollback path |
|---|---|
| A WASM hash was approved but not yet applied | Call `revoke_upgrade(wasm_hash)` — removes the allowlist entry; no on-chain state changed |
| The upgrade was applied but the new logic has a bug | Deploy a new fixed WASM, upload it, approve it with `new_version + 1`, apply it |

### What CANNOT be rolled back

| Scenario | Reason |
|---|---|
| A `ContractVersion` advance | The `ContractVersion` is monotonically increasing; there is no on-chain operation to decrement it |
| The WASM swap itself | Once `upgrade_contract` succeeds, the old WASM is no longer the installed executable.  The old code is not deleted from the ledger (uploaded WASMs persist), but the contract will not re-install it unless a new `approve_upgrade` + `upgrade_contract` sequence is executed |
| State written by the new WASM before the bug was caught | Persistent storage mutations made under the new WASM are permanent.  Recovery requires a further upgrade that compensates for or corrects the bad state |
| Proof records | Proof commitments are intentionally immutable; no upgrade path removes them |

**Key implication:** there is no one-click rollback to a prior WASM version.
Forward-fix is the only recovery path once an upgrade is applied.  This makes
the dry-run rehearsal procedure (section 8) and the two-step approve→apply
separation critical safeguards.

---

## 7. Emergency pause procedure

protocol-config exposes `pause()` / `unpause()`, and proof-registry checks
`is_paused()` before every `register_proof` call.  This is the primary circuit
breaker for an active incident.

**When to use pause:**
- A bug is discovered in proof-registry's validation logic after an upgrade.
- A suspicious burst of proof registrations is observed.
- The admin key is suspected compromised and further proof writes need to be
  halted while the key is rotated.

**Pause does not affect:**
- Reads (`get_proof`, `is_valid_proof`, `is_revoked`, `get_issuer`, etc.)
- Revocations (`revoke_proof`, `admin_revoke_proof`)
- Issuer management (issuer-registry is not gated on the pause flag)
- Upgrades (upgrade calls do not check `is_paused`)

**To pause:**
```powershell
stellar contract invoke `
  --source earnproof-deployer --network testnet `
  --auth-mode root --auto-sign `
  --id <protocol-config-contract-id> -- pause
```

**To unpause:**
```powershell
stellar contract invoke `
  --source earnproof-deployer --network testnet `
  --auth-mode root --auto-sign `
  --id <protocol-config-contract-id> -- unpause
```

Both calls emit events (`Paused`, `Unpaused`) that are observable on-chain.

---

## 8. Dry-run rehearsal procedure

A dry-run builds the new WASM and uploads it to the target network, then prints
exactly what `approve_upgrade` and `upgrade_contract` calls would be made —
without making any on-chain state changes.  Run this against testnet before any
production upgrade.

```powershell
.\scripts\upgrade-contract.ps1 `
  -Contract protocol-config `
  -ContractId <contract-id> `
  -Source earnproof-deployer `
  -NewVersion 2 `
  -Network testnet `
  -DryRun
```

Expected output structure:

```
==> Install wasm32v1-none target
==> Build WASM artifacts
WASM sha256: <hex>
==> Upload protocol-config WASM
Uploaded WASM hash: <64-char hex>

=== DRY RUN — no on-chain state changes made ===

Would invoke on contract <contract-id> (network: testnet):
  approve_upgrade --wasm_hash <hash> --new_version 2
  upgrade_contract --wasm_hash <hash>
  get_contract_version  -- expected: 2

Rehearsal complete.  Re-run without -DryRun to apply.
```

The dry-run confirms:
- The Rust toolchain and WASM target are installed and functional.
- The WASM builds cleanly and produces an artifact.
- The WASM can be uploaded to the network (the hash is a real ledger key).
- The exact hash and version that would be applied are visible for review.

Run the dry-run at least once on testnet before applying any upgrade to a
production environment.

---

## 9. Step-by-step upgrade procedure

This procedure applies to all three contracts.  Repeat per-contract in
deployment order: protocol-config → issuer-registry → proof-registry.

### Pre-upgrade checklist

- [ ] New WASM has been reviewed and the git commit is recorded.
- [ ] Dry-run has been executed on testnet (`-DryRun`).
- [ ] `NewVersion` is exactly one greater than the current `get_contract_version()`.
- [ ] Backend team is notified; they are prepared to update ABI bindings.
- [ ] (proof-registry only) Decide whether to pause protocol-config during
      the upgrade window.

### Upgrade steps

```powershell
# 1. Run the upgrade script (add -PauseProtocol -ProtocolConfigId <id> for proof-registry)
.\scripts\upgrade-contract.ps1 `
  -Contract <contract-name> `
  -ContractId <contract-id> `
  -Source earnproof-deployer `
  -NewVersion <N> `
  -Network testnet

# 2. Verify on-chain version
stellar contract invoke `
  --source earnproof-deployer --network testnet `
  --id <contract-id> -- get_contract_version
# Expected output: <N>

# 3. Verify the upgrade manifest was written
Get-Content scripts/upgrade-manifest.<contract>.<timestamp>.json

# 4. Notify backend to update contract ABI bindings and verify reads.
```

### Post-upgrade validation

- Read a known record from persistent storage (e.g. `get_issuer`, `get_proof`,
  `is_schema_version_approved`) and confirm expected values are returned.
- Confirm `get_contract_version()` returns the expected new version.
- Confirm `is_upgrade_allowed(<just-applied-hash>)` returns `false` (entry
  consumed).
- For proof-registry: register a test proof against an approved schema version
  and confirm it is accepted.

### Storage layout and capability checks

- Before and after an upgrade, read `get_storage_layout()`. An upgrade never
  moves `active_layout`; if the status is `MigrationRequired`, run
  `begin_layout_migration` and `complete_layout_migration` for the reported
  target. See [Storage Layout Versions](./storage-layout-versions.md).
- Read `get_capabilities()` and confirm the new build keeps every bit the old
  build reported. See [Contract Capability Bitmaps](./contract-capabilities.md).

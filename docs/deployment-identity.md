# Deployment Identity

An interface version says what a dependency implements, not which deployment
it belongs to. A testnet `issuer-registry` and a mainnet one built from the
same code report the same interface version. `proof-registry` therefore also
checks a deployment identity before it trusts a dependency.

## Identity

Every contract exposes:

| Entrypoint | Result | Authorization |
|---|---|---|
| `set_deployment_domain(deployment_domain: BytesN<32>)` | `Result<DeploymentIdentity, ContractError>` | admin |
| `get_deployment_identity()` | `Option<DeploymentIdentity>` | none |

`DeploymentIdentity` (`identity_version = 1`) holds:

- `network_id`: `env.ledger().network_id()`, the SHA-256 of the network
  passphrase. The contract cannot choose it.
- `deployment_domain`: a governed, non-secret 32-byte label for one
  deployment, for example the SHA-256 of `"earnproof:testnet:blue"`.
- `commitment`: SHA-256 of the XDR encoding of
  `(identity_version, network_id, deployment_domain)`.

The domain is **write-once**. Setting the same domain again returns the same
identity and publishes nothing; a different domain fails with
`DeploymentIdentityMismatch` (65); the zero domain fails with `InvalidInput`.
It cannot be set during decommission grace or after decommission
(`InvalidState`). Before a domain is set, `get_deployment_identity` returns
`None` and the contract is treated as a legacy deployment.

## Consumer rules in `proof-registry`

A dependency *matches* when it reports an identity whose version is known,
whose commitment recomputes from its own fields, and whose commitment equals
the registry's pinned identity. A dependency that does not implement
`get_deployment_identity`, or whose call fails, reports no identity and never
matches. Copying the pinned commitment onto another domain does not match.

| Call | When the registry is pinned | When unpinned (legacy) |
|---|---|---|
| `initialize_in_domain(admin, issuer_registry, protocol_config, domain)` | both dependencies must match `domain`, checked after the interface handshake and before any write | n/a |
| `initialize(...)` | n/a | unchanged; no identity check |
| `set_deployment_domain(domain)` | write-once rules above | both bound dependencies must match `domain` before it is pinned |
| `set_issuer_registry`, `set_protocol_config` | the replacement must match, else `DeploymentIdentityMismatch` (65) | unchanged |
| `register_proof` and its variants, `register_proof_with_payload`, `register_proof_with_activation`, `register_proofs_batch` | both bound dependencies must still match, else `DependencyIdentityMismatch` (327) | no extra call |

All checks run before state is written, and every failure aborts the whole
invocation, so a mismatch leaves the registry exactly as it was. In
registration the check runs after the pause check, so a paused protocol still
reports `ContractPaused` first.

## Migration and compatibility

- Existing deployments keep working unchanged: nothing is validated until the
  admin pins a domain.
- To adopt identity checks: set the same domain on `issuer-registry` and
  `protocol-config`, then call `set_deployment_domain` on `proof-registry`.
  The last step fails if either bound dependency does not match.
- A pinned registry cannot bind a legacy dependency. Upgrade the dependency
  and set its domain first.
- Interface versions advance to `issuer-registry` 1.2.0 and `protocol-config`
  1.3.0. `proof-registry` still accepts any 1.x dependency for the interface
  handshake.

Tests: [`tests/cross-contract/tests/dependency_identity.rs`](../tests/cross-contract/tests/dependency_identity.rs).

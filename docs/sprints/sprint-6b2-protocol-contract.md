# Sprint 6B2 Signed Protocol Contract

Status: first implementation foundation complete on 2026-07-23.

This contract is owned by `tessara-module-contract`. It is transport-neutral and contains no HTTP, persistence, browser-cookie, Core-session, or database-credential behavior.

## Signed Envelope

`SignedEnvelopeV1<T>` contains:

- `schema_version`
- explicit `issuer`
- explicit `key_id`
- one `purpose`
- the typed `payload`
- an Ed25519 signature encoded as unpadded base64url

Every signing and verification key is bound to exactly one purpose. A valid signature made with the same Ed25519 material is still rejected when the trusted purpose, issuer, or key ID differs.

The signing input is compact JSON containing the schema version, issuer, key ID, purpose, and payload. Object keys are recursively sorted in ascending order, arrays retain declared order, and strings and numbers use standard JSON encoding. The signature field is excluded. `canonical_protocol_signing_bytes` is the executable reference.

## Development Trust

The canonical development fixture uses three different keys:

| Purpose | Issuer | Key ID | Runtime consumer |
| --- | --- | --- | --- |
| Shell context | `tessara.core` | `shell-context-dev-1` | modules |
| Authorization grant | `tessara.core` | `authorization-grant-dev-1` | target module |
| Fixture external identity | `tessara.fixture-identity` | `fixture-external-dev-1` | Core enrollment |

The committed trust fixture contains verification keys only. Deterministic private material exists solely in the fixture generator and tests, is visibly development-only, and must never be loaded as production configuration.

## `ShellContextV1`

The signed shell projection binds:

- installation, Module Definition, and Module Instance
- original actor display projection
- theme and authorized navigation projection
- return destination
- locale and time zone
- correlation ID
- active, disabled, degraded, stale-context, or recovery document state
- issue and expiry times

Validation requires the expected installation, module audience, correlation ID, schema version, and a maximum 60-second lifetime. Shell context carries presentation context only and cannot acquire product authority through extra wire fields.

## Authorization Grant

The signed grant binds:

- installation and original actor
- presenting service and target Module Instance audience
- declared dependency binding and functional contract
- exact action and read/mutation operation
- independent capability-to-Organization-scope bindings
- optional resource ownership assertion and delegation basis
- authorization and Organization revisions
- unique `jti`
- issue and expiry times

Read grants permit at most 60 seconds. Mutation grants permit at most 30 seconds and require one-time `jti` consumption by the receiving module.

Each capability/scope binding remains independent. A read binding for subtree A and manage binding for subtree B never authorizes read on B or manage on A unless those exact combinations are separately present.

## Wire And Fixture Guarantees

- All envelope and payload objects reject unknown fields.
- Signed payload mutation fails verification.
- Wrong issuer, key ID, purpose, installation, presenting service, audience, dependency, contract, action, operation, revision, or time window fails closed.
- Canonical fixtures are UTF-8, LF-terminated, digest-pinned JSON.
- `invalid-tampered-shell-context-v1.json` proves that a well-formed payload with a retained signature is rejected after display data changes.

Canonical evidence:

- `crates/tessara-module-contract/tests/fixtures/valid-protocol-messages-v1.json`
- `crates/tessara-module-contract/tests/fixtures/valid-protocol-messages-v1.json.sha256`
- `crates/tessara-module-contract/tests/fixtures/invalid-tampered-shell-context-v1.json`
- `crates/tessara-module-contract/tests/fixtures/invalid-tampered-shell-context-v1.json.sha256`

## Next Contract Layer

Installation-control claim lifecycle, signed Core eligibility, recovery operator authorization, enrollment reservation/finalization, and stable restricted exchange results build on this envelope without changing its signing rules.

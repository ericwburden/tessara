# Validation Platform Ownership

Tessara validation has three explicit owners. The boundaries below keep a
process-only correction from invalidating unrelated application behavior while
preserving the full application cone whenever assertions, fixtures, contracts,
or product interfaces change.

## Owners

### Application acceptance contract

The application owns product assertions, scenarios, expected behavior,
fixtures, and application tests. These inputs are candidate-affecting.

### Validation platform

The validation platform owns process execution, topology and port lifecycle,
readiness synchronization, locks and checkpoints, cleanup and restoration,
evidence publication, hashing, indexes, and certificate assembly. Platform
behavior has an independently versioned release identity and synthetic tests
that do not execute Tessara application behavior.

### Sprint adapter

A sprint adapter declaratively maps scenarios to commands, topology
requirements, prerequisites, and evidence contracts. It does not implement its
own process, cleanup, topology, or evidence machinery.

## Independent identities

Validation evidence binds three identities independently:

- the application candidate fingerprint;
- the acceptance-contract fingerprint; and
- the validation-platform release fingerprint.

A platform-only correction invalidates the platform certificate, affected
adapters, affected formal lanes, and narrow application/platform integration
checks. It does not automatically invalidate unrelated application targets.
An acceptance or application change still selects its complete declared
implementation and formal-validation cone.

## Cargo build storage platform slice

The first bounded platform component is
`scripts/tessara-cargo-build-policy.psm1`. Its contract is
`tessara.validation.cargo-build-policy`, its initial release is `1.0.0`, and
`Get-TessaraCargoBuildPolicyIdentity` publishes the release plus the exact
module SHA-256 fingerprint.

The component owns only Cargo build-storage lifecycle:

- one unique target directory per sequential validation lane;
- non-incremental, reduced-debug validation builds;
- explicit retained diagnostic builds;
- minimum free-space preflight;
- capability-bound target cleanup in `finally`; and
- restoration of the caller's Cargo environment.

Cleanup requires the private active lease, exact generated target identity,
authenticated marker bytes, ordinary repository/target paths, and a path chain
without reparse points. Caller-provided path containment alone is never cleanup
authority.

The application-independent self-test uses a synthetic temporary Cargo crate.
It proves release identity, deterministic free-space rejection, forged-state
and marker-tamper rejection, ancestor-reparse rejection on Windows, failed-lane
cleanup, exact environment restoration, retained diagnostic behavior, and no
temporary residue.

## Adapter inventory

| Adapter | Cargo policy status | Rationale |
|---|---|---|
| `scripts/validate.ps1` full gate | Current `1.0.0` consumer | One sequential general validation lane |
| `scripts/validate.ps1 -Fast` | Development mode; no isolated target | The developer owns the reusable worktree target |
| Closed Sprint 8B runners | Retained historical implementation | Their sealed evidence keeps the runner identity under which it was issued |
| Next active sprint adapter | Must consume the platform release | No new sprint-specific Cargo lifecycle implementation is permitted |

Changing the module, its self-test, or an active adapter requires the platform
self-test, the affected adapter boundary proof, PowerShell parsing, and source
integrity checks. Changing only this platform component does not claim that
Tessara application assertions ran.

## Forward extraction order

The next validation-platform work extracts the mixed responsibilities in
`scripts/uat-sprint-8b.ps1` behind one canonical platform boundary, beginning
with process execution, topology/port ownership, cleanup/restoration, and
evidence finalization. A synthetic minimal-service certification suite must
prove success, failure, interruption, port handoff, immutable evidence, and
restoration before another sprint adapter consumes that boundary.

The extraction is forward-only: keep one canonical platform path, migrate an
active adapter completely, and delete superseded lifecycle code in the touched
cone. Do not add a compatibility runner or weaken an acceptance assertion,
fixture meaning, retry policy, timeout, or baseline.

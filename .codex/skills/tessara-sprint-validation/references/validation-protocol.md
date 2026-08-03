# Tessara Validation Protocol

This protocol is the shared contract for `tessara-validation-preflight`,
`tessara-sit`, `tessara-uat`, `tessara-sprint-validation`, and
`tessara-sprint-closeout`.

## Receipt chain

Store receipts in the sprint evidence directory:

```text
validation-readiness-result.json
candidate-rehearsal-result.json
preflight-result.json
candidate.json
sit-result.json
uat-result.json
closeout-authorization.json
attempts/<phase>-<attempt>.json
evidence-manifest.json
evidence-manifest.json.sha256
```

Each downstream receipt names and hashes its prerequisite receipts. Reject a
missing, malformed, stale, failed, or mismatched prerequisite.

Readiness and rehearsal receipts bind mutable source and environment
identities rather than claiming a frozen candidate fingerprint. Preflight
verifies those identities still match, then creates the immutable candidate
fingerprint and receipt. Neither receipt is authoritative SIT or UAT evidence.

## Fingerprints

### Candidate fingerprint

Hash canonical values for:

- implementation commit and tree
- dirty state at freeze
- tracked product, test, harness, migration, fixture, seed, manifest,
  bootstrap, deployment, and acceptance-inventory identity
- deployment profile/configuration digest
- migration-baseline identity
- expected source-provenance keys and values

Documentation-only changes after freeze do not change this fingerprint when
they cannot alter executable behavior or test interpretation.

No fingerprint is frozen until the mandatory Test Readiness Gate and complete
Candidate Rehearsal both pass. Any correction after rehearsal requires both
gates to repeat before freeze.

### Environment fingerprint

Hash non-secret identities for:

- operating system and required tool versions
- database host/port/name identities and reset authorization presence
- Compose project/profile and required ports
- service topology and intended slot
- account/role fixture identities without credentials
- evidence root and runner output-path mode

Never include passwords, tokens, signing secrets, private keys, or secret
values in a receipt.

## Phase and lane stages

Use these states:

```text
not_started -> preparing -> executing -> finalizing -> passed|failed|blocked
```

Record `assertions_started` separately. A setup failure before assertions has
different invalidation scope from a product assertion failure.

Every receipt includes at least:

- schema version, sprint, phase/lane, attempt, authoritative flag, and state
- candidate and environment fingerprints
- prerequisite receipt paths and SHA-256 hashes
- exact commands with start/end timestamps, duration, and exit status
- assertion counts when available
- raw log and evidence paths
- classification, correction, narrow proof, and invalidation decision
- cleanup/restoration result

Write a start receipt before expensive work. Append logs continuously. Write
completion through a temporary sibling and atomic rename when repository
automation supports it. Inspect receipts and logs before rerunning a command
whose controlling tool session disappeared.

## Result collection

- Run independent checks within a lane or isolated scenario set fail-late.
- Record every safe sibling result even after one fails.
- Stop dependent or destructive work when its prerequisite state is invalid.
- A phase passes only when every required check passes.
- A narrow reproducer diagnoses; it never replaces the authoritative command.

## Classifications

Use exactly:

- `preflight/setup`
- `product`
- `harness`
- `environment`
- `flaky`
- `evidence-finalization`
- `product-decision`

`product-decision` pauses for user direction and is never converted into a
test failure.

## Invalidation matrix

The coordinator records the decision and rationale.

| Cause | Minimum invalidation |
|---|---|
| Candidate fingerprint changed | All SIT and UAT |
| Acceptance inventory or tracked harness changed | All SIT and UAT |
| Shared environment changed materially | Affected lane and downstream phases |
| Lane-local setup failed before assertions | Failed lane |
| Evidence finalization failed; raw results are complete and immutable | Finalization only |
| Test assertion failed; candidate unchanged | Complete failed lane; coordinator assesses upstream environment relevance |
| UAT scenario setup failed before product actions | Affected isolated scenario set, when prerequisites reconfirm |
| Product defect corrected | Refreeze, all SIT, then all UAT |
| Missing acceptance assertion discovered | Update inventory/candidate, all SIT, then all UAT |

When the last row or any other candidate-affecting correction occurs, restore
the canonical environment and stop formal testing before creating the next
candidate. Complete the readiness-and-rehearsal cycle before refreeze. A
narrow reproducer remains diagnostic only.

Never choose a smaller scope merely to save time. Reuse an earlier receipt only
when its candidate and environment fingerprints still match and the failure
could not have affected its assertions.

## Authority boundaries

- Preflight may freeze a candidate but cannot authorize SIT results.
- `tessara-sprint-validation` owns pre-freeze readiness, rehearsal, defect
  batching, and permission to enter preflight.
- Readiness and rehearsal cannot authorize SIT, UAT, or closeout.
- SIT may authorize UAT but cannot authorize closeout.
- UAT may report acceptance but cannot authorize closeout.
- `tessara-sprint-validation` alone validates the chain, decides invalidation,
  and writes `closeout-authorization.json`.
- Closeout consumes the chain and cannot originate acceptance tests.

## Evidence manifest

Declare required evidence during preflight. Update it during SIT and UAT.
Before authorization:

- verify every required file exists
- parse every structured artifact
- validate repository Markdown links
- distinguish authoritative and superseded attempts
- hash every retained file
- verify the manifest sidecar
- confirm the canonical handoff topology and health

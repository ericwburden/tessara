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
uat-defect-harvest.json
defect-batch.json
correction-impact-assessment.json
focused-repair-validation/attempt-<n>.json
canonical-restoration.json
final-certification-entry.json
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

Post-SIT convergence records are conditional: plan their paths before
validation, then require them when a candidate-invalidating UAT defect occurs.
They are diagnostic history and never substitute for the successor
candidate's complete authoritative receipt chain.

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
- candidate and environment fingerprints for frozen-candidate records, or the
  mutable source/environment identities for readiness, rehearsal, and focused
  repair records
- prerequisite receipt paths and SHA-256 hashes
- exact commands with start/end timestamps, duration, and exit status
- assertion counts when available
- raw log and evidence paths
- classification, correction, narrow proof, and invalidation decision
- cleanup/restoration result

Write a start receipt before any fallible phase prerequisite is authenticated
or probed, not only before expensive assertions. The start receipt may mark a
claimed source/environment identity `unverified`; the prerequisite lane updates
it only after exact receipt, source, and environment checks pass. A missing,
malformed, stale, or mismatched prerequisite must therefore finish as one
retained failed lane and block only its declared dependents. Append logs
continuously. Write completion through a temporary sibling and atomic rename
when repository automation supports it. Inspect receipts and logs before
rerunning a command whose controlling tool session disappeared.

For mutable readiness and rehearsal, retain the declared graph in that initial
receipt. Readiness additionally writes a hashed, create-once start snapshot
before its live attempt receipt can be checkpointed. Its live attempt receipt
and SHA-256 sidecar are rewritten after every terminal pass, failure, or block,
and the validation-state index names the exact current checkpoint hash. This
preserves an immutable launch boundary while making an interrupted partial gate
harvestable.

Readiness acquires an operating-system exclusive file handle during its
pre-publication reservation and holds it through final attempt and
validation-state publication. Lock contention or an unauthorized attempt number
therefore rejects the Readiness launch before any attempt namespace or receipt
exists. Candidate Rehearsal instead acquires and holds the same kind of handle in
its first declared attempt-state prerequisite after publishing its immutable
start receipt; contention there is a retained prerequisite failure, while safe
independent checks still run fail-late and destructive or dependent checks
remain blocked. In both phases the persistent lock path is not proof of
ownership; exclusivity comes from the open handle.

## Result collection

- Run independent checks within a lane or isolated scenario set fail-late.
- Serialize independent Cargo checks that share a target directory, or assign
  each a distinct explicit `CARGO_TARGET_DIR`; logical independence alone does
  not make concurrent artifact cleanup safe.
- Record every safe sibling result even after one fails.
- Stop dependent or destructive work when its prerequisite state is invalid.
- Declare the check dependency graph before assertions start. On first failure,
  move the attempt to `harvesting`; every declared check must finish as passed,
  failed, or blocked with an exact prerequisite reason.
- Forbid tracked correction, invalidation/restart, or a new attempt number until
  the harvest is complete. Then write one consolidated defect batch for the
  whole diagnostic pass and invalidate from that batch.
- Preserve partial results and raw logs append-only. A narrow reproducer adds
  evidence to the active batch and never closes harvesting by itself.
- Before creating an attempt receipt, start snapshot, sidecar, or attempt log
  directory, acquire the evidence-root lock and authenticate that the requested
  attempt number is permitted by current state and the pending correction-lineage
  authorization. Reject an out-of-sequence probe without occupying any canonical
  attempt namespace. Hold that same lock through start publication and one-time
  authorization consumption.
- A completed failed rehearsal may authorize exactly one successor Readiness
  start only after its harvest and consolidated batch pass the executable
  harvest guard. That Readiness consumes the authorization through a
  create-once, hashed receipt bound to the failed predecessor receipt and its
  own immutable start snapshot. Duplicate consumption or a different successor
  is forbidden.
- A completed failed Readiness follows the same terminal discipline: retain one
  typed Readiness harvest, one consolidated defect batch, and one create-once
  authorization for the exact next Readiness attempt. This finalization may be
  recovered without starting the successor. The successor alone creates the
  append-only consumption receipt bound to its immutable start snapshot.
- The sole failed-Readiness finalizer recovery exception is the already-terminal
  Sprint 8A Readiness 38 attempt. Its complete immutable receipt, raw evidence,
  two failures, one exact block, and documented consolidated correction set were
  frozen before the finalizer existed and before the user-directed testing exit.
  Commit only the finalizer/enforcement correction, then run
  `-Attempt 38 -FinalizeFailedAttempt` before R39. It must not rerun R38 checks or
  start R39. No later attempt may use this ordering exception.
- Preserve every rehearsal-to-Readiness and failed-Readiness-to-Readiness link
  in one authenticated correction lineage. Each link binds its failed receipt,
  harvest, batch, authorization, consumption, immutable successor start, and
  terminal successor receipt. Only the last link may be pending; gaps, forks,
  duplicate consumption, skipped successor attempts, and reuse of an earlier
  authorization are forbidden.
- Correction epochs may alternate. When a later rehearsal fails after a
  correction-authorized Readiness has passed, its new rehearsal link must name
  that passing Readiness as its sole prerequisite and retain the complete prior
  lineage prefix. A later failed Readiness then appends after that rehearsal
  link; it never replaces or forks the earlier epoch.
- Every correction-lineage successor terminal is the immutable
  `attempts/readiness-N.json` receipt. Only the current passing Readiness may
  also be published through `validation-readiness-result.json`, and that alias
  must have the same SHA-256 and JSON document as its immutable counterpart.
  Never retain the mutable alias as a historical lineage terminal. An alias
  rollover leaves each earlier epoch authenticatable through its immutable
  terminal while the current lineage tip accepts only the new alias/counterpart
  pair.
- Enforce the complete lineage topology, not a valid-looking suffix. A root
  candidate has exactly one immutable passing-Readiness prerequisite and no
  prior lineage; a root failed Readiness has no predecessor correction
  authorization, consumption, or prior lineage. Every later candidate retains
  the exact prefix and every later failed Readiness binds the immediately prior
  failed terminal. Reject truncated roots, gaps, and forks.
- Authenticate the exact canonical predecessor, harvest, consolidated batch,
  authorization, authorization-consumption, successor-start, and successor-
  terminal paths and documents. The predecessor, harvest, batch, and
  authorization retain identical mutable-source identity and environment
  fingerprint. Before authorizing a failed-Readiness correction, reject any
  occupied artifact or evidence directory for the exact successor attempt.
- The sole legacy conversion is the retained rehearsal 30 to Readiness 38
  transition. Authenticate rehearsal 30's legacy prerequisite SHA through
  immutable `attempts/readiness-37.json`; do not hash the historical canonical
  alias. No other legacy path or topology exception is allowed.
- Maintain one validation-state index naming the sole current Readiness receipt
  path and SHA-256. Rehearsal requires exact equality with that path and digest
  and validates the complete correction lineage through the current passing
  Readiness; an older passing Readiness, an incomplete lineage, or a used
  attempt number cannot be selected again.
- Readiness, Candidate Rehearsal, and Validation Preflight authenticate the
  validation-state sidecar and exact attempt numbers before consuming its
  references. Preflight also authenticates the passing rehearsal's immutable
  `attempts/candidate-rehearsal-N-attempt.json` sidecar and SHA, requires its
  phase/attempt/state to match the canonical rehearsal result, and requires the
  attempt, result, and validation-state correction lineage to be identical.
- Candidate Rehearsal's state lane alone binds the canonical current Readiness
  to its immutable counterpart. The separately declared-independent
  Readiness/source/environment lane authenticates the supplied receipt and its
  contained correction-consumption evidence without reading state-lane runtime
  outputs, so a state defect cannot suppress that sibling diagnostic. On a
  pass, both the immutable rehearsal attempt and canonical rehearsal result
  name the same immutable Readiness prerequisite; preflight authenticates that
  equality separately from the current canonical alias/state check.
- Treat a null `produced_evidence` value as no additional evidence. Preserve
  every non-null produced reference exactly alongside the primary check log,
  and resolve every raw reference inside the repository evidence root before
  hashing. An absolute or traversal path outside that root is invalid evidence.
- When an automated UAT diagnostic projects nested semantic assertions, retain
  each failed assertion under its exact scenario/assertion identity with its
  allowed classification, failure reason, and hashed raw evidence. A healthy
  outer projection lane stays passed when only nested semantics fail, so the
  consolidated batch records each semantic defect once. Fail the outer lane
  separately only for an independent wrapper or harness defect.
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

After authoritative SIT, a candidate-invalidating UAT failure additionally
enters the Post-SIT Defect Convergence Cycle. Invalidate immediately, harvest
remaining safe independent UAT scenarios diagnostically, correct one
consolidated batch while mutable, authorize an evidence-based impact cone, and
repeat focused repair validation until converged. Then rerun complete
readiness, rehearsal, SIT, and UAT for a successor candidate. The convergence
cycle never narrows final certification.

Never choose a smaller scope merely to save time. Reuse an earlier receipt only
when its candidate and environment fingerprints still match and the failure
could not have affected its assertions.

## Authority boundaries

- Preflight may freeze a candidate but cannot authorize SIT results.
- `tessara-sprint-validation` owns pre-freeze readiness, rehearsal, defect
  batching, post-SIT diagnostic harvesting, impact-cone authorization,
  convergence, and permission to enter preflight or return to full
  readiness/rehearsal.
- Readiness and rehearsal cannot authorize SIT, UAT, or closeout.
- SIT may authorize UAT but cannot authorize closeout.
- UAT may report acceptance but cannot authorize closeout.
- SIT and UAT may execute coordinator-assigned focused repair checks but
  cannot authorize the cone, issue authoritative results for them, or reduce
  final certification scope.
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

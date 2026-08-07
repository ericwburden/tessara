# Tessara Validation Protocol

This protocol is the shared contract for `tessara-validation-preflight`,
`tessara-sit`, `tessara-uat`, `tessara-sprint-validation`, and
`tessara-sprint-closeout`.

## Receipt chain

Store receipts in the sprint evidence directory:

```text
validation-readiness-result.json
candidate-rehearsal-result.json
attempts/candidate-rehearsal-<n>-start.json
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

Changes to validation skills or protocol references, repository runners,
tests, fixtures, acceptance inventory, deployment inputs, environment-contract
logic, or product source are candidate-affecting whenever they can alter
execution or interpretation. They invalidate superseded mutable receipts and
place every intersecting rehearsal lane in Wave A for the next source-bound
cycle.

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
not_started -> preparing -> executing -> finalizing -> passed|failed|blocked|deferred
```

Record `assertions_started` separately. A setup failure before assertions has
different invalidation scope from a product assertion failure.
`deferred` is a Candidate Rehearsal-only terminal state for an eligible Wave B
lane after Wave A has failed. It never means passed, blocked, skipped, reused
evidence, or authoritative proof.

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

Before creating attempt-specific evidence, acquire the exclusive
pre-publication reservation and confirm that the target namespace is empty.
Lock contention rejects the launch without producing attempt evidence. Under
the retained lock, Candidate Rehearsal may retain its prelaunch state capture
and authenticate scheduling inputs; an unauthenticated scheduling history
selects the conservative schedule rather than proving lifecycle eligibility.
Write the start receipt before any declared lane assertion or live
source/environment probe. The start may mark a claimed source/environment
identity `unverified`; the declared prerequisite lane updates it only after its
exact receipt, source, and environment checks pass. A missing, malformed, stale,
or mismatched declared prerequisite must finish as one retained failed lane and
block only its declared dependents. Append logs continuously. Write completion
through a temporary sibling and atomic rename when repository automation
supports it. Inspect receipts and logs before rerunning a command whose
controlling tool session disappeared.

For mutable readiness and rehearsal, retain the declared graph in that initial
receipt. Readiness additionally writes a hashed, create-once start snapshot
before its live attempt receipt can be checkpointed. Its live attempt receipt
and SHA-256 sidecar are rewritten after every terminal pass, failure, or block,
and the validation-state index names the exact current checkpoint hash. This
preserves an immutable launch boundary while making an interrupted partial gate
harvestable.

Readiness and Candidate Rehearsal each acquire an operating-system exclusive
file handle during their pre-publication reservation and hold it through final
attempt and validation-state publication. Lock contention rejects the launch
before any attempt-specific evidence exists; it is not retained as an
attempt-lane result. Candidate Rehearsal's first declared attempt-state
prerequisite authenticates the already-retained handle, requested attempt,
current state, and lifecycle transition within the published attempt. A failure
of that declared authentication is retained, safe independent checks still run
fail-late, and destructive or dependent checks remain blocked. In both phases
the persistent lock path is not proof of ownership; exclusivity comes from the
open handle.

### Candidate Rehearsal bounded scheduling

After successful pre-publication reservation and launch-admission
authentication, but before declared lane assertions or source/environment
probing, Candidate Rehearsal writes a create-once, hashed start receipt
containing its complete declared graph, authenticated scheduling inputs,
per-lane impact decisions and deferral counters, and the deterministic order of
Wave A, Wave B execution or deferral, aggregate sinks, terminal cleanup sinks,
and safety finalizers. That ordering is immutable for the attempt. Recovery and
every terminal checkpoint must name and hash the same start receipt; do not
recompute a more favorable schedule after a failure or process loss.

The complete declared graph is the full per-lane contract, not only a list of
lane names. It includes each lane's prerequisites, scheduler role and segment,
impact paths, evidence paths and roots, and nested-result path. Canonicalize
every persisted path to repository-relative forward-slash form inside the
declared evidence root before publishing the immutable start. Resume and
recovery authenticate the start digest and require complete graph equality; a
names-only graph, rooted path, traversal, missing member, or changed member
rejects continuation.

Wave A contains:

- lifecycle prerequisites, retained-lock/state-transition authentication,
  current Readiness/source/environment authentication, and required diagnostic
  failure-containment recovery;
- lanes that failed in the preceding rehearsal;
- never-executed or newly reachable lanes;
- lanes affected by the current correction impact cone;
- lanes already deferred three consecutive times; and
- the prerequisite closure needed to execute those lanes safely.

Run safe independent Wave A siblings fail-late. A lane whose source, harness,
fixture, environment contract, acceptance inventory, dependency, or relevant
prerequisite changed is in Wave A regardless of its prior state or counter.
Aggregate lanes are sinks: their dependency fan-in never expands Wave A. Run an
aggregate only after its current-attempt prerequisites become eligible and
pass; otherwise retain it as blocked with the exact current-attempt dependency
reason.

A terminal canonical-restoration/final-health cleanup sink is neither the
Wave A failure-containment diagnostic nor the certification aggregate. It must
remain independently runnable after either fails. Its endpoint probes use the
declared service-specific status, redirect, media-type, and body contract; a
followed redirect, login document, generic HTTP-success range, or another
service's readiness endpoint is not health evidence.

Wave B contains authenticated prior-passing lanes outside the correction cone
that have fewer than three consecutive deferrals. Wave B disposition depends on
the diagnostic Wave A result, not on the later terminal cleanup sink. If Wave A
passes, execute Wave B in the same attempt. If Wave A fails, finish every safe
Wave A sibling, then terminalize eligible Wave B lanes as `deferred` without
starting their assertions. Run aggregate sinks after that disposition, then
always run terminal cleanup/restoration and safety finalizers. A lane
may be deferred at most three consecutive attempts; it must execute on the
fourth, and any execution resets its counter. Safety finalizers, teardown,
recovery, canonical restoration, and other mandatory cleanup are never
deferrable.

Each deferred lane result records its exact lane name and `state: deferred`,
prior passing receipt path and SHA-256, prior source and environment identities,
current correction-impact decision and explicit non-impact rationale,
consecutive-deferral count, `mandatory_by_attempt`, prerequisite state,
`assertions_started: false`, null execution timestamps and duration, and an
explicit statement that prior evidence is diagnostic history only. Authenticate
the prior passing receipt and identities; do not treat them as evidence for the
current attempt.

If prior evidence, its hashes, the correction impact, or deferral counters
cannot be authenticated, use the conservative full-harvest fallback: put every
non-sink diagnostic lane in Wave A and retain the normal Wave B, aggregate-sink,
terminal-cleanup-sink, and safety-finalizer ordering. Never retrofit this
scheduler into or rewrite a historical receipt.

Any deferred lane makes the rehearsal failed and incomplete. It forbids
`candidate-rehearsal-result.json`, preflight authorization, candidate freeze,
SIT, UAT, closeout, or any claim that rehearsal passed.

## Result collection

- Run independent checks within a lane or isolated scenario set fail-late.
- Serialize independent Cargo checks that share a target directory, or assign
  each a distinct explicit `CARGO_TARGET_DIR`; logical independence alone does
  not make concurrent artifact cleanup safe.
- Record every safe sibling result even after one fails.
- Stop dependent or destructive work when its prerequisite state is invalid.
- Declare the check dependency graph and immutable two-wave order before
  assertions start. On first failure, move the attempt to `harvesting`; every
  declared check must finish as passed, failed, blocked, or an exactly eligible
  deferred Wave B lane with complete diagnostic-only provenance.
- Forbid tracked correction, invalidation/restart, or a new attempt number until
  the terminal-accounting and harvest guards succeed. Then write one
  consolidated defect batch for the whole diagnostic pass and invalidate from
  that batch. Record deferrals separately from real defects and blocked checks;
  accepting complete deferred accounting must not fabricate a failure, while a
  missing lane must reject harvest and correction authorization.
- Preserve partial results and raw logs append-only. A narrow reproducer adds
  evidence to the active batch and never closes harvesting by itself.
- On process loss, authenticate the immutable start receipt and the latest
  sidecar-bound checkpoint before resuming the same attempt. Preserve its fixed
  ordering and counters, retain raw evidence for any orphaned executing lane,
  terminalize that lane truthfully, and continue only safe remaining work.
  Recovery also covers a validation-state capture published before the start, a
  start published before the initial attempt checkpoint, a deferred-loop
  checkpoint, and a terminal attempt published before its append-only
  harvest/batch/authorization or passing-result/state tail. Authenticate and
  reuse every complete immutable receipt/sidecar pair; never rerun lanes or
  rewrite immutable evidence after the attempt itself is terminal. The
  lifecycle-prerequisite receipt may retain its exact pre-authentication
  placeholder identity. After authenticating a nonterminal recovery, persist
  the recovered source/environment identity in the attempt checkpoint before
  resuming any lane; a terminal failure that never reached authentication keeps
  its explicit placeholder identity through harvest rather than being
  retroactively rebound.
- Acquire the evidence-root reservation lock before creating an attempt receipt,
  start snapshot, sidecar, or attempt log directory. Lock contention rejects
  launch without producing attempt evidence. Under the retained handle,
  Candidate Rehearsal captures the current scheduling inputs and either
  authenticates them or fixes the conservative fallback before start
  publication. The declared attempt-state lane later authenticates the requested
  attempt and lifecycle transition and retains any failure. Hold the same handle
  through protected publication and one-time authorization consumption.
- Candidate Rehearsal checkpoints and harvest directly name and hash the
  immutable schedule start and retain exact terminal-lane accounting. The
  consolidated batch binds through the hashed harvest and retains the exact
  deferred-lane inventory. Correction authorization binds the predecessor,
  harvest, and batch and retains the aggregate deferred count. Validation state
  binds the expected attempt and schedule SHA-256 to the full schedule retained
  in the passing Readiness receipt and immutable rehearsal start. Successor
  Readiness authenticates that complete reference chain and its exact counters.
  Correction lineage accepts a failed attempt with complete eligible deferral
  accounting, but rejects a missing lane, altered schedule, or unauthenticated
  counter and never treats the deferred history as proof.
- Complete terminal accounting permits the immutable harvest and consolidated
  batch to be retained when cleanup/restoration is not yet proven, but it does
  not permit correction authorization. The authorization guard requires
  `cleanup_restoration.required = true` and
  `cleanup_restoration.result = canonical_successor_healthy`, backed by exact
  passing current-attempt `final-successor-health` and
  `final-environment-identity` lane receipts. If either receipt is missing,
  blocked, failed, deferred, stale, or mismatched, withhold authorization while
  preserving the terminal attempt, harvest, batch, and raw evidence. A
  passing final-health lane must itself bind a hash-authenticated
  materialization receipt proving the exact attempt, source, environment,
  first apply, semantic no-op, and post-materialization health contract. A
  schema-3 authorization embeds both canonical receipt references and hashes;
  validation state and successor Readiness authenticate them as part of the
  correction-lineage link.
- The sole pre-enforcement exception is retained Sprint 8A Rehearsal 32. Its
  schema-2 authorization is quarantined unless paired with the exact append-only
  post-restoration qualification defined in the Candidate Rehearsal reference,
  and that tuple permits only Readiness 42. Do not generalize or rewrite it.
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
- A passing Readiness may be superseded without a new correction-lineage link
  or consumption only at the clean pre-rehearsal boundary: rehearsal is still
  ineligible, preflight is false, and the requested Readiness number is the
  exact sequential next unused attempt. Authenticate that boundary under the
  retained lock before namespace creation. The superseding immutable start and
  terminal receipts name the immediately preceding immutable passing Readiness
  as their exact prerequisite. When the predecessor is already the terminal of
  a consumed correction lineage, both receipts carry that complete
  `correction_lineage` value with its authorization and consumption bindings
  unchanged. The original Readiness remains the one authorization consumer;
  never drop or retarget the lineage
  and never create a second consumption. The clean successor's own
  `predecessor_correction_authorization` and `correction_consumption_receipt`
  fields remain null. Validation may reach the current
  Readiness from the consumed tip only through a complete chain of canonical,
  hashed, sequential clean-pre-rehearsal supersession edges. Any gap, altered
  predecessor, reused attempt, non-clean boundary, or changed lineage binding
  rejects the launch before its namespace is created.
- A Readiness that directly consumes a pending correction authorization does
  not embed the in-progress `correction_lineage` object in its immutable start
  or terminal receipt. Its direct authorization/consumption fields and the
  validation-state lineage bind that terminal after its SHA-256 exists. This
  avoids a self-referential terminal hash. Only a clean supersession carries an
  already-complete, byte-semantically unchanged lineage in its receipts.
- The sole failed-Readiness finalizer recovery exception was the already-terminal
  Sprint 8A Readiness 38 attempt. Its complete immutable receipt, raw evidence,
  two failures, one exact block, and documented consolidated correction set were
  frozen before the finalizer existed and before the user-directed testing exit.
  R38 was finalized once without rerunning its checks, and R39 subsequently
  passed. No later attempt may use this ordering exception. The sprint
  verification record and sidecar-bound validation state, not this reusable
  reference, retain the coordinator-authorized live attempt boundary.
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
  terminal. The current alias/counterpart pair must be either the consumed
  lineage tip itself or the terminal of the authenticated clean-pre-rehearsal
  supersession chain rooted at that tip.
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
  Readiness, either directly or through its exact clean-pre-rehearsal
  supersession chain. An older passing Readiness cannot be selected as current,
  an incomplete lineage is invalid, and a used attempt number cannot be reused.
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
- A phase passes only when every required check passes. Candidate Rehearsal
  additionally requires zero deferred lanes; only then may it publish its
  canonical result receipt.
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

# Tessara Validation Readiness and Candidate Rehearsal

This reference defines the mandatory mutable-build process before every
candidate freeze. `tessara-sprint-validation` owns both gates. Preflight audits
their receipts and freezes the candidate; SIT and UAT do not rerun them.

## Gate 1: Test Readiness

Derive an executable checklist from the sprint's actual validation record,
acceptance inventory, runners, deployment configuration, and source. Do not
reuse a generic list without reconciling it to the sprint.

The checklist must verify:

- every SIT, deployed-smoke, scripted UAT, manual UAT, evidence, recovery, and
  environment need;
- exact environment-variable names, database identities, destructive-reset
  acknowledgements, and safe disposable scope;
- required tool versions and every shell/runtime the repository claims to
  support, including runner parsing and invocation in each supported shell;
- ports, Compose project/profile, databases, topology, service health,
  handoff slot, and source-provenance inputs and label keys;
- fixtures semantically: actors, credentials usability, roles, capability and
  scope assignments, products, allowed and blocked resources, provider and
  security revisions, recognizable data, negative identifiers/services, and
  repeat-run idempotence;
- every runner, argument contract, output-path form, receipt writer, SHA-256
  helper, atomic finalization path, and failure/supersession path by self-test
  or a safe disposable probe;
- every non-interactive destructive rehearsal command disables host prompting
  only after its repository-owned explicit authorization and exact-target
  guards, and readiness executes that command's `WhatIf` path under the same
  output-capture boundary used by rehearsal; host-rendered `WhatIf` text need
  not become pipeline data, but the invocation must return successfully;
- every acceptance clause mapped to automated, deployed-smoke, and manual
  evidence, with explicit justified `N/A` entries rather than blanks; and
- a clean repository plus source-exact build inputs before rehearsal begins.

A passing Readiness also derives the exact schedule for the next authorized
Candidate Rehearsal attempt. It authenticates the preceding rehearsal's start,
terminal lane receipts, prior-passing references and hashes, source/environment
identities, correction lineage, changed paths, impact scope, and consecutive-
deferral counters. It records the resulting deterministic Wave A, cleanup-sink,
Wave B, aggregate-sink, and finalizer order as the full schedule in the passing
Readiness receipt. Validation state binds the exact expected rehearsal attempt
and that schedule's SHA-256; Candidate Rehearsal copies the full schedule into
its create-once start receipt and verifies the digest instead of recomputing it
after launch.

If any prior receipt, hash, identity, impact decision, or counter cannot be
authenticated, Readiness records the reason and emits the conservative
full-harvest schedule instead of inferring history. Legacy receipts remain
unchanged and provide diagnostic history only.

Run independent checks fail-late. Retain a checklist result for every item,
including exact command/runtime, timestamps, exit status, evidence path, and
failure classification. Write `validation-readiness-result.json` only when all
items pass. Hash it and include its evidence in the manifest. A failed gate
keeps the build mutable and forbids candidate freeze.

Before any canonical attempt artifact exists, Readiness acquires the evidence-
root operating-system exclusive lock and authenticates only the current state,
pending authorization, requested number, and empty attempt namespace. It then
writes both its live `unverified` start receipt and a create-once hashed copy of
that start snapshot with the complete declared checklist. Source, environment,
and product assertions begin only after start publication. After every terminal
check or block, Readiness publishes the live attempt receipt and sidecar and
updates validation state to the exact checkpoint hash. The same lock is held
until the final attempt/result and state index are published. A process loss
therefore leaves the immutable start plus the latest hashed terminal checkpoint
rather than only in-memory results.

If Readiness fails after its complete fail-late harvest, publish one typed
Readiness harvest and one consolidated batch covering every failed check and
exact blocked dependency. Only the completed harvest guard may issue a
create-once authorization for the exact next Readiness attempt. Failed-attempt
finalization is recoverable without launching that successor and must never
rerun or rewrite the failed attempt. The successor consumes the authorization
once through a receipt bound to its immutable start snapshot. If that successor
also fails, append its own failed-Readiness link after harvesting rather than
reusing an earlier authorization or entering a consumed-authorization dead
end.

Before the failed-attempt finalizer authorizes the exact next attempt, verify
that its immutable start, terminal receipt, sidecars, and normal evidence
directory are all unoccupied. Perform this collision check before publishing
the harvest, batch, or authorization so retry remains deterministic. The
finalizer branch returns before normal-attempt evidence directories are
created; finalization does not launch or partially materialize its successor.

A passing Readiness may be rerun without correction lineage only at the narrow
clean pre-rehearsal boundary: rehearsal remains ineligible and preflight is
false. Any failed predecessor requires one pending authenticated correction
link. A consumed corrected Readiness cannot be silently rerun because doing so
would orphan the retained lineage.

Reserve the requested attempt number before its canonical receipt, start
snapshot, sidecars, or `readiness-N/` directory exists. Under the exclusive
evidence-root lock, authenticate current validation state, the pending lineage
tip, and any exact `allowed_successor_attempt`; only then create the namespace
and immutable start. Hold the same lock through consumption. An out-of-sequence
probe is rejected without occupying any collision target, so the authorized
attempt can still fail, finalize, and authorize its exact successor.

Readiness 38 was the sole legacy failed-Readiness finalizer recovery. Its
complete terminal receipt/raw evidence and documented consolidated defect set
were frozen before this finalizer existed. It was finalized once without
rerunning checks after the user-directed testing exit, and R39 subsequently
passed. Do not generalize or repeat this exception. Sprint 8A's current
coordinator-authorized boundary is R40 followed by Candidate Rehearsal 32.

## Gate 2: Candidate Rehearsal

Build or materialize a source-exact but explicitly mutable and
non-authoritative rehearsal build. Record a rehearsal source identity from the
commit, tree, dirty state, acceptance inventory, deployment inputs, and
environment contract. Do not issue a candidate fingerprint or candidate
receipt.

Run a complete validation-shaped pass containing:

1. all static, formatting, compilation, lint, schema, manifest, link, and
   boundary checks;
2. the full Rust workspace test contract and required optimized/timing lanes;
3. source-exact image build, provenance audit, deployment/materialization,
   migrations, topology health, and exact no-op/idempotence proof;
4. the complete Playwright acceptance inventory with its required workers,
   retries, runtime binding, discovery, and retained outputs;
5. authorization conformance and nondisclosure checks;
6. general and sprint-specific deployed smoke;
7. failure containment, recovery, canonical restoration, and final health;
   and
8. automated diagnostic equivalents of every UAT scenario, including semantic
   fixture verification. These checks are not formal UAT.

### Two-wave rehearsal schedule

Before creating Candidate Rehearsal attempt-specific evidence, the launcher
acquires the exclusive reservation lock and confirms the target namespace is
empty. Contention rejects launch without attempt evidence. Under that retained
handle, it captures current state and Readiness scheduling inputs; authenticated
history selects the bounded schedule and any authentication gap selects the
conservative fallback. The immutable start receipt then fixes the complete
declared graph, each lane's scheduler role and impact decision, captured prior
history, deferral counters, and deterministic segment order before declared
source/environment probes or assertions. Every checkpoint and any recovery must
authenticate that exact start receipt. The Wave A attempt-state lane then
authenticates the retained lock handle, requested attempt, current state, and
lifecycle transition as a terminally accounted lane; it does not acquire a
second lock.

Wave A always includes lifecycle prerequisites, retained-lock/state-transition
authentication, current Readiness/source/environment authentication, required
cleanup/restoration,
lanes that failed in the preceding rehearsal, never-executed and newly
reachable lanes, lanes in the current correction impact cone, and every lane
already deferred three consecutive times. Add only the prerequisite closure
required to execute those lanes safely. A changed source, validation skill,
runner, test, fixture, environment contract, acceptance inventory, deployment
input, dependency, or relevant prerequisite puts the affected lane in Wave A
regardless of its deferral counter. Run safe independent Wave A siblings
fail-late.

Wave B contains only authenticated prior-passing lanes outside the current
impact cone with fewer than three consecutive deferrals. When Wave A passes,
continue directly into Wave B in the same attempt; a potentially passing
rehearsal must execute every required lane. When Wave A fails, finish every safe
Wave A sibling, complete mandatory cleanup/restoration, and terminalize each
eligible Wave B lane as `deferred` without beginning assertions. Execution of a
lane resets its counter. Three consecutive deferrals make it mandatory in Wave
A on the fourth attempt.

Treat aggregate lanes as sinks. A previously blocked certification or final-
health fan-in does not pull all of its prerequisites into Wave A. Execute an
aggregate only when its current-attempt prerequisites are eligible and pass;
otherwise record the aggregate as blocked with its exact current dependency
reason; one of those prerequisites may itself be deferred. A separate final-
health check that proves canonical restoration remains mandatory when it is
declared as a cleanup sink. Teardown, recovery, canonical restoration, cleanup
sinks, and final source/environment safety checks remain mandatory even when
certification-oriented lanes are deferred.

A deferred receipt is neither a pass nor a skipped or blocked execution. It
contains the lane name, `state: deferred`, authenticated prior passing receipt
path and SHA-256, prior source and environment identities, current impact
decision and non-impact rationale, consecutive-deferral count,
`mandatory_by_attempt`, prerequisite state, `assertions_started: false`, null
execution timestamps and duration, and the explicit notice that prior evidence
is diagnostic history only. It has no current-attempt assertion evidence.

If prior evidence, hashes, impact scope, or counters cannot be authenticated,
fall back to full harvest: all non-sink diagnostic lanes execute in Wave A,
with cleanup sinks, Wave B execution, aggregate sinks, and safety finalizers
retaining their declared order. Do not retrofit schedules or counters into
historical receipts.

After process loss, resume only the same attempt after authenticating its
immutable schedule and latest checkpoint. Preserve lane order and counters,
retain raw evidence and terminalize any orphaned executing lane truthfully, and
continue only safe remaining work. Never allocate a successor merely because
the controller disappeared.

Any deferred lane makes the attempt failed and incomplete. It cannot produce
`candidate-rehearsal-result.json`, authorize preflight, freeze a candidate,
satisfy SIT/UAT, authorize closeout, or be represented as authoritative proof.

The automated-UAT receipt preserves every scenario's exact semantic assertion
inventory. Each failed assertion carries its scenario/assertion identity,
producer/evaluator identity, allowed classification, exact failure reason, and
hashed raw evidence into rehearsal harvesting. If those nested assertions are
the only failures, the outer UAT projection lane remains passed and contributes
no generic duplicate defect. An independent wrapper, parsing, or evidence
failure may fail the outer lane separately.

Never label rehearsal output authoritative SIT or UAT. Formal UAT remains
forbidden until authoritative SIT passes after freeze.

Run independent sibling checks fail-late when it is safe, so one failure does
not hide other defects. Stop dependent or destructive work whose prerequisite
state is invalid. Retain raw logs and per-lane diagnostic receipts under a
rehearsal namespace.

Every declared lane must retain an explicit `assertions_started` boolean and,
when true, an assertion-start timestamp. A passed lane has
`assertions_started = true`. A failed lane records true only when its assertions
or product actions began; a setup failure before that boundary records false
and no assertion-start timestamp but remains failed rather than blocked. A
blocked lane has `assertions_started = false` and no assertion-start timestamp.
A deferred lane likewise has `assertions_started = false` and null execution
timestamps, but carries its authenticated diagnostic-history and deferral
fields instead of a block reason or current evidence. Attempt-level assertion
counts count only lanes whose assertions actually started, never setup-only
failures or terminal blocked/deferred receipts.

When a retained structured child receipt provides a canonical defect
classification, the outer lane and consolidated harvest must project that
classification and record the structured receipt as its classification source.
Log matching and a lane's declared default are fallbacks only; they cannot
override a structured classification. Raw lane classifications remain
immutable evidence even when diagnosis later consolidates several symptoms
under one different root cause.

The graph must keep every safe static or otherwise topology-independent lane
free of the fallible state/readiness prerequisite. A retained-lock/state-lane
failure blocks only work that actually requires that prerequisite or whose
destructive execution would be unsafe. The attempt-state lane authenticates the
already-held evidence-root operating-system exclusive handle and state
transition; the launcher retains that handle through terminal publication.

When a sprint-specific rehearsal lane must resolve deployed service identities
or compose several generic helpers, use a repository-owned orchestration runner
whose exact argument contract is parsed and self-tested by readiness. Do not
assemble those bindings ad hoc at the operator prompt: a helper invocation that
omits an environment identity, evidence binding, or declared parameter is a
harness defect and blocks its dependent lanes even if sibling diagnostics pass.

On Windows, or whenever commands share one Cargo target directory, serialize
complete Cargo checks that can clean or replace build artifacts. Parallel
logical siblings must use distinct explicit `CARGO_TARGET_DIR` values. A
missing executable caused by another check's cleanup is an environment defect,
not a product failure, and still leaves the originating check failed for that
diagnostic pass.

The launch reservation authenticates only the current state and supplied
Readiness schedule references needed to admit and bind the attempt. The
rehearsal start then declares every lane, whether independent or dependent, the
prerequisite lane names for dependent work, and the complete immutable two-wave
schedule. After start publication, the declared Readiness/source/environment
lane independently authenticates the supplied receipt's contents and live
identities. An initially claimed identity remains explicitly `unverified` until
that prerequisite lane succeeds. After the first failure, set the attempt to
`harvesting`; do not edit tracked candidate inputs, invalidate/restart the
attempt, or allocate a successor attempt number until every declared lane is
recorded as passed, failed, blocked with its exact dependency reason, or
eligible `deferred` with its complete diagnostic-only provenance. A repository-
owned terminal-accounting and harvest guard must reject completion, correction,
and restart while any lane remains unaccounted for.

Environment-identity comparisons must retain secret-free expected and actual
fingerprints plus the names of changed contract sections even when comparison
fails. A generic mismatch message by itself is not sufficient diagnostic
evidence.

New receipts persist evidence references as canonical repository-relative
paths with forward slashes, contained by the declared evidence root. Consumers
reject traversal and paths outside that root. To consume an already-issued
one-use transition, a consumer may canonicalize an immutable historical
absolute reference only when it resolves inside the same repository and
evidence root; it must still persist every new reference canonically. This
exception does not authorize rewriting historical evidence.

After the pass, collect every discovered product, test, harness, fixture,
acceptance-inventory, deployment, environment-contract, and evidence defect
into one batch. Correct the batch while the build remains mutable. Do not
freeze an intermediate correction.

Write exactly one harvest receipt and one consolidated defect-batch receipt per
diagnostic pass. Keep deferred-lane inventory distinct from true failures and
blocked checks. The harvest and correction-authorization guards accept complete
deferred accounting but reject any missing declaration, ineligible deferral,
unauthenticated prior pass, or altered counter. Narrow reproducers attach
evidence to that batch; they do not create correction batches or authorize a
restart on their own.

The attempt checkpoints and harvest directly bind the immutable start and exact
terminal-lane accounting. The batch binds through the hashed harvest and stores
the exact deferred inventory; correction authorization binds the predecessor,
harvest, and batch with the aggregate deferred count. Validation state stores
the expected rehearsal attempt and schedule SHA-256, binding them to the full
schedule retained in passing Readiness and copied into the immutable start.

Only the completed harvest guard may issue a correction authorization, and it
authorizes one successor Readiness start for that exact failed rehearsal. The
successor Readiness writes a create-once hashed consumption receipt binding the
authorization, predecessor rehearsal receipt, and successor immutable start
snapshot. Validation state records that transition; duplicate consumption, a
different successor, or another attempt while an attempt is active is rejected.

Validation state retains these transitions as one correction lineage of
authenticated append-only references, including any intervening failed
Readiness attempts. Every non-final link is consumed exactly once, only the
final link may be pending, and each failed-Readiness successor is the exact
attempt named by its authorization. Rehearsal and preflight validate the whole
lineage and require its tip to terminate at the exact current passing
Readiness.

Each consumed lineage link terminates at immutable
`attempts/readiness-N.json`. `validation-readiness-result.json` is only the
current passing alias and must have the same JSON document and SHA-256 as that
attempt's immutable receipt. Historical links never point at the alias. When a
later passing Readiness replaces the alias, earlier links remain valid through
their immutable terminals and only the latest tip may match the current
alias/counterpart pair.

Validate topology from the root. A root candidate has one immutable passing
Readiness prerequisite and no prefix; a root failed Readiness has no prior
authorization, consumption, or lineage. Later candidate failures retain the
complete exact prefix, while later failed Readiness attempts bind the preceding
failed terminal and its authorization/consumption. Reject any root that is a
truncated suffix. Authenticate exact canonical paths for predecessor, harvest,
batch, authorization, consumption, successor start, and successor terminal,
and require source/environment identity continuity across the four correction
authorization documents. The only legacy exception is rehearsal 30 to
Readiness 38, whose retained prerequisite SHA authenticates immutable
Readiness 37 rather than the historical canonical alias.

If that passing Readiness enters a later rehearsal which fails, append a new
candidate-rehearsal anchor after the consumed lineage tip. The failed rehearsal
receipt must bind the exact passing Readiness as its sole prerequisite and
retain the complete prior lineage prefix. This permits repeating epochs such
as rehearsal failure, failed Readiness, passing Readiness, later rehearsal
failure, and its authorized successor without losing earlier evidence.

Candidate Rehearsal accepts only the passing Readiness path and SHA-256 named as
current by validation state. It verifies exact equality again against the
receipt, source/environment identity, and any predecessor authorization and
consumption receipt before dependent work. A formerly passing Readiness cannot
be reused after a failed rehearsal or a successor Readiness start, and attempt
numbers whose start or terminal evidence exists are never reusable.

Validation Preflight additionally authenticates the passing rehearsal's exact
immutable `attempts/candidate-rehearsal-N-attempt.json` receipt and sidecar. Its
embedded SHA-256, phase, attempt, state, source/environment identity, and
correction lineage must agree with the canonical rehearsal result; the attempt,
result, and validation-state lineage must be identical before candidate freeze.
It also authenticates the immutable start schedule, requires every declared
lane to have executed and passed, and rejects any deferred result or nonzero
deferred count. A forged canonical result cannot hide deferred attempt state.
The rehearsal attempt and canonical result must also retain the same single
immutable passing-Readiness prerequisite. The current canonical Readiness alias
is validated separately against validation state and its exact immutable
counterpart; it is not persisted as that historical rehearsal prerequisite.

Keep the state/current-Readiness lane and the supplied Readiness/source/
environment lane truly independent. Only the state lane establishes the
canonical-alias-to-immutable binding. The sibling Readiness lane authenticates
its supplied receipt, source, environment, and any correction-consumption
receipt using evidence-root-contained resolution, without consuming state-lane
runtime fields. This preserves useful diagnostics when state validation fails.

Harvest treats null `produced_evidence` as an empty set and retains every
non-null produced reference exactly. Primary logs and produced evidence must
resolve inside the declared repository evidence root before their SHA-256 is
accepted; rooted or traversal references outside that root are rejected.

Repeat the complete Test Readiness Gate and Candidate Rehearsal after every
correction batch until both pass cleanly. The bounded scheduler prioritizes the
affected cone and previous failures, but any attempt that could pass still
executes all required lanes. Focused or narrow reproducers may diagnose
corrections but cannot satisfy rehearsal or replace an affected Wave A lane.

Write and hash `candidate-rehearsal-result.json` only after every rehearsal
lane executes and passes, the deferred count is zero, canonical restoration
succeeds, and no defect remains open. It
must name and hash the passing readiness receipt and bind the exact mutable
source/environment identities that preflight will audit.

## Freeze boundary

Preflight may freeze a candidate only when:

- both result receipts parse, hash, and pass;
- rehearsal names the exact current readiness receipt path and SHA-256, with
  any required one-use correction transition consumed by that same Readiness;
- current clean source, acceptance inventory, deployment inputs, and
  environment match the passing rehearsal identities exactly; and
- no correction or acceptance decision occurred after the passing rehearsal.

Any mismatch returns to the complete readiness-and-rehearsal cycle. After
freeze, use the existing candidate fingerprint, receipt chain, evidence
retention, invalidation matrix, and phase authority rules without relaxation.

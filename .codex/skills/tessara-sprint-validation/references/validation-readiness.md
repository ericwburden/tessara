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

Run independent checks fail-late. Retain a checklist result for every item,
including exact command/runtime, timestamps, exit status, evidence path, and
failure classification. Write `validation-readiness-result.json` only when all
items pass. Hash it and include its evidence in the manifest. A failed gate
keeps the build mutable and forbids candidate freeze.

Before any source, environment, state, or prerequisite probe, Readiness writes
both its live `unverified` start receipt and a create-once hashed copy of that
start snapshot with the complete declared checklist. Its first declared check
acquires the evidence-root operating-system exclusive attempt lock. After every
terminal check or block, Readiness publishes the live attempt receipt and
sidecar and updates validation state to the exact checkpoint hash. The lock is
held until the final attempt/result and state index are published. A process
loss therefore leaves the immutable start plus the latest hashed terminal
checkpoint rather than only in-memory results.

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
Attempt-level assertion counts count only lanes whose assertions actually
started, never setup-only failures or terminal blocked receipts.

When a retained structured child receipt provides a canonical defect
classification, the outer lane and consolidated harvest must project that
classification and record the structured receipt as its classification source.
Log matching and a lane's declared default are fallbacks only; they cannot
override a structured classification. Raw lane classifications remain
immutable evidence even when diagnosis later consolidates several symptoms
under one different root cause.

The graph must keep every safe static or otherwise topology-independent lane
free of the fallible state/readiness prerequisite. State, readiness, or lock
failure blocks only work that actually requires that prerequisite or whose
destructive execution would be unsafe. The attempt-state lane acquires the same
evidence-root operating-system exclusive lock and holds it through terminal
publication.

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

The rehearsal start receipt must be retained before authenticating the
Readiness receipt or probing source/environment prerequisites. It declares
every lane, whether it is independent or dependent, and the prerequisite lane
names for dependent work. An initially claimed identity remains explicitly
`unverified` until the prerequisite lane succeeds. After the first failure,
set the attempt to `harvesting`; do not edit tracked candidate inputs,
invalidate/restart the attempt, or allocate a successor attempt number until
every declared lane is recorded as passed, failed, or blocked with its exact
dependency reason. A repository-owned runner or equivalent executable guard
must reject completion, correction, and restart while any lane remains
unaccounted for.

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
diagnostic pass. Narrow reproducers attach evidence to that batch; they do not
create correction batches or authorize a restart on their own.

Only the completed harvest guard may issue a correction authorization, and it
authorizes one successor Readiness start for that exact failed rehearsal. The
successor Readiness writes a create-once hashed consumption receipt binding the
authorization, predecessor rehearsal receipt, and successor immutable start
snapshot. Validation state records that transition; duplicate consumption, a
different successor, or another attempt while an attempt is active is rejected.

Candidate Rehearsal accepts only the passing Readiness path and SHA-256 named as
current by validation state. It verifies exact equality again against the
receipt, source/environment identity, and any predecessor authorization and
consumption receipt before dependent work. A formerly passing Readiness cannot
be reused after a failed rehearsal or a successor Readiness start, and attempt
numbers whose start or terminal evidence exists are never reusable.

Repeat the complete Test Readiness Gate and the complete Candidate Rehearsal
after every correction batch until both pass cleanly. Focused or narrow
reproducers may diagnose corrections but cannot satisfy rehearsal and cannot
replace the complete affected-lane rerun inside the next full rehearsal.

Write and hash `candidate-rehearsal-result.json` only after every rehearsal
lane passes, canonical restoration succeeds, and no defect remains open. It
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

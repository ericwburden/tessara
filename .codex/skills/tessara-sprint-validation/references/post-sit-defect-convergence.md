# Tessara Post-SIT Defect Convergence

This reference defines the mandatory convergence cycle after authoritative SIT
has passed and formal UAT exposes a candidate-invalidating defect. The
`tessara-sprint-validation` coordinator owns every authorization decision.

## Contents

- Trigger and invariants
- Fail-late UAT defect harvest
- Consolidated mutable correction batch
- Correction-impact assessment
- Focused repair-validation loop
- Final certification
- Structured record contracts

## Trigger and invariants

Enter this cycle on the first UAT failure requiring a tracked product, test,
harness, fixture, acceptance-inventory, migration, deployment, or other
candidate-affecting correction. Immediately:

1. retain the failed scenario and raw evidence;
2. classify it using the shared protocol vocabulary;
3. mark the frozen candidate invalid and all of its SIT/UAT evidence
   superseded for certification;
4. forbid a passing `uat-result.json`; and
5. restore or reconfirm the failed scenario's prerequisites before deciding
   whether diagnostic harvesting can continue.

Preserve these invariants:

- Formal UAT begins only after authoritative SIT passes.
- Evidence collected after invalidation is diagnostic and non-authoritative.
- Unsafe, dependent, or uninterpretable scenarios do not run.
- Focused repair validation never constitutes SIT or UAT authorization.
- Product decisions pause for user direction.
- Final certification always reruns complete readiness, rehearsal, SIT, and
  UAT for a new source-exact fingerprint.

## Fail-late UAT defect harvest

The coordinator instructs `tessara-uat` to examine every remaining scenario
in the frozen inventory. For each scenario:

- run it when it is independent, its prerequisites can be reconfirmed, shared
  state is trustworthy, and no security or product decision makes the result
  unreliable;
- restore and reconfirm scenario prerequisites when safe before execution;
- label all execution after invalidation `diagnostic defect harvest` and
  `authoritative: false`;
- retain the actual result and every discovered defect even though a pass
  cannot contribute to certification; or
- record `blocked` with the exact failed dependency, corrupted state,
  security constraint, or unresolved product decision.

Do not silently skip inventory entries. Do not create or update a passing
`uat-result.json`. Finish by restoring the canonical environment and writing
`uat-defect-harvest.json`.

## Consolidated mutable correction batch

Merge all harvested findings into `defect-batch.json`. Include product,
harness, fixture, environment, acceptance-inventory, deployment, evidence,
and product-decision findings. Deduplicate findings without discarding their
scenario evidence.

Track each defect as `open`, `corrected`, `passed`, `blocked`, or
`superseded`. A product decision remains `blocked` until the user decides it.
Apply all authorized corrections while source remains mutable. Do not freeze
an intermediate correction. Use the repository-local Tessara implementation
skill for every tracked product, fixture, harness, migration, acceptance
source, or deployment correction.

## Correction-impact assessment

Before any repair rerun, write `correction-impact-assessment.json`. Map every
changed file, contract, fixture, environment input, and behavior to:

- unit and integration tests;
- static, lint, schema, manifest, link, and boundary checks;
- provider and consumer contracts;
- migrations, bootstrap, deployment, and materialization;
- Playwright scenarios;
- authorization, identity, nondisclosure, smoke, recovery, and deployment
  lanes; and
- scripted and manual UAT scenarios.

Support every inclusion and exclusion with repository evidence such as call
sites, ownership boundaries, manifests, imports, fixture consumers, test
inventory, or deployment topology. Never select a cone merely to reduce
runtime.

Default to a broad cone for authorization, identity, protocol, migration,
shared fixture, shared environment, or cross-module changes. Include every
known consumer of a harness or fixture. If any effect cannot be bounded with
confidence, set `escalation: complete-candidate-rehearsal` and require the
complete rehearsal contract rather than guessing a reduced cone.

The coordinator alone authorizes the cone. SIT and UAT may report dependency
evidence but cannot reduce or approve the cone.

## Focused repair-validation loop

For each authorized attempt:

1. run every declared affected static/SIT check and affected automated/manual
   UAT diagnostic scenario;
2. label all outputs `focused repair validation` and `authoritative: false`;
3. omit candidate fingerprints and never emit `sit-result.json` or
   `uat-result.json`;
4. run safe independent siblings fail-late;
5. retain blocked checks with their exact dependency reason;
6. collect new defects into the next consolidated batch;
7. apply the complete next batch while source is mutable;
8. reassess the full impact cone; and
9. repeat until the declared cone passes and no defect is open.

A narrow reproducer may diagnose a correction but cannot satisfy an item in
the focused cone. A changed correction batch supersedes prior focused results;
preserve those results as history.

## Final certification

When the focused cone passes and no defect remains open, the coordinator
writes `final-certification-entry.json`. This record permits only a return to
the complete Test Readiness Gate and Candidate Rehearsal. It does not freeze a
candidate or authorize any phase.

After complete readiness and rehearsal pass for the final clean mutable
source, preflight may freeze a new source-exact candidate. Run authoritative
SIT in full, then formal UAT in full from the beginning. Never reuse
pre-correction SIT, UAT, harvest, or focused evidence as authoritative evidence
for the successor fingerprint. Closeout requires this final complete chain.

## Structured record contracts

Store records under the sprint evidence directory. Include `schema_version`,
`sprint`, timestamps, mutable source/environment identities where applicable,
prerequisite paths and SHA-256 hashes, evidence paths, and status in every
record. Do not include secrets.

### `uat-defect-harvest.json`

Include invalidated candidate/environment fingerprints, first invalidating
failure, invalidation timestamp, `authoritative: false`, and one disposition
for every UAT inventory item. Each disposition contains scenario ID, status
(`passed`, `failed`, `blocked`, or `superseded`), executed flag, prerequisite
reconfirmation, exact blocked reason, defect IDs, evidence paths, cleanup, and
restoration result.

### `defect-batch.json`

Include batch number, source identity before/after correction, and defects.
Each defect contains stable ID, classification, status (`open`, `corrected`,
`passed`, `blocked`, or `superseded`), discovery scenario/lane, description,
evidence, correction files or environment actions, narrow diagnostic proof,
and supersession links.

### `correction-impact-assessment.json`

Include batch number, changed files/contracts/fixtures/environment inputs and
behavior, affected and explicitly excluded checks in every required category,
evidence/rationale per decision, default-broad-cone triggers, confidence,
escalation, and coordinator authorization timestamp.

### `focused-repair-validation/attempt-<n>.json`

Include assessment hash, mutable source/environment identity,
`authoritative: false`, exact declared cone, check/scenario results, blocked
reasons, newly discovered defect IDs, restoration result, and attempt outcome.
Use only `passed`, `failed`, `blocked`, or `superseded` for check status.

### `canonical-restoration.json`

Include the triggering phase/attempt, topology, service health, source
provenance, fixture/prerequisite reconfirmation, restoration actions, and
result. Write a new attempt-specific record when canonical state changes.

### `final-certification-entry.json`

Include hashes of the final defect batch, impact assessment, last passing
focused attempt, canonical restoration, all defect/scenario statuses, open
defect count, coordinator decision, and the required next boundary. The
decision may be `enter-full-readiness-rehearsal` only when the declared cone
passed, restoration passed, no defect is open or blocked, and no product
decision remains.

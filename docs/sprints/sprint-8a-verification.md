# Sprint 8A Validation Record

Status: Validation Readiness 41 passed its complete 15-check graph against clean
commit `d703e7a67c3f1177b621b0e14161e0661209ea42`, tree
`889ddc5f784ea1e81686ce63dd22669aabde956d`, and environment fingerprint
`53fbb1f74375f96abb292ba610fc030686fca55864a70ca8149932f1a4820e28`.
Candidate Rehearsal 32 then completed its immutable 32-lane conservative
full-harvest schedule with 19 passes, 2 raw `product` failures, 11 exact
dependency blocks, and 0 deferrals. Cleanup/restoration was not proven and no
`candidate-rehearsal-result.json` was created. Its immutable evidence, one
harvest, and one raw two-defect batch remain unchanged. Append-only diagnosis
and a separate source-exact restoration qualified the prematurely issued R32
authorization for exactly one Readiness 42 start; that qualification does not
make R32 pass or authorize preflight, freeze, SIT, UAT, or closeout. The
consolidated product/harness/evidence correction is now mutable. After focused
verification and a clean commit, complete Readiness 42 and complete Rehearsal
33 remain the next coordinator-controlled gates.

- Sprint: Sprint 8A — Component Module Separation Slice
- Branch: `codex/sprint-8a`
- Planned evidence root: `artifacts/sprint-8a-closeout/`
- Execution contract: [Sprint 8A plan](./sprint-8a-plan.md)

## Implementation Readiness Snapshot

### Readiness 41 pass, Rehearsal 32 failure, and restoration-qualified correction

Readiness 41 ran from 13:56:43 through 14:01:45 EDT on 2026-08-07 and passed all
15 checks. Its immutable start SHA-256 is
`8365d2b96048b1e5c2c60f776b16ee9048cc9272933ef1549284c5f2d8a48ee5`;
its immutable terminal and byte-identical canonical alias SHA-256 is
`547940b0ca6685d98304a1f51d9810e38bd54463f6fd378dd375d46c7407c2aa`.
It bound Rehearsal 32's conservative full-harvest schedule at SHA-256
`7b09c1b33568e6b1bf16b9347efd31d9d99291fc40176d7e4de7d8aa1c3a7f22`.

Rehearsal 32 ran from 14:06:12 through 15:37:28 EDT (about 5,476 seconds by its
receipt timestamps). Its
immutable start SHA-256 is
`c397c6f4b1290911724fd07cabc74bc866371abc762b23392f6dc4ab19fff670`;
its terminal failed-attempt SHA-256 is
`98cddc03977a476af5c5843131b9114ec2f025a1f5eefffa57514daccd5448c5`.
All 32 declarations are terminal: 19 passed, 2 failed, 11 blocked, and 0 were
deferred. Twenty-one lanes began assertions. The immutable raw failures remain:

1. `source-exact-materialization-no-op` — `product`, with the first
   materialization failure retained at SHA-256
   `6fcccdf3d31607e578eccbf22004049d38f9d2c57757f9e4090f40911916b99e`.
2. `failure-containment-successor-health` — `product`, with the containment
   result retained at SHA-256
   `2ca4015419ef96993900755d8965b6fab3518d18ff7feeed0a8c64926f63bf9a`.

The exact blocked checks are:

- `deployed-inventory-navigation-audit`, `deployment-evidence`,
  `product-smoke`, and `component-upgrade-rollback`, each blocked by
  `source-exact-materialization-no-op`;
- `playwright-execution`, blocked by `source-exact-materialization-no-op` and
  `deployment-evidence`;
- `successor-inventory-navigation-audit`,
  `successor-deployment-evidence`, `successor-product-smoke`, and
  `final-successor-health`, each blocked by
  `failure-containment-successor-health`;
- `live-product-diagnostics`, blocked by `deployment-evidence` and
  `product-smoke`; and
- `uat-diagnostics`, blocked by the failed materialization and containment
  lanes plus their six unavailable live/successor dependents.

The terminal harvest SHA-256 is
`89861d8add519812993934dba182e6939b1437d38e7f524e4469693001c2c809`;
the one immutable raw two-defect batch SHA-256 is
`4a5219fd0e16d13fbf0e15286ae75f53c84e261a264bbef877127ff8866a1249`.
The raw classifications are retained even though the append-only diagnostic
supplement, SHA-256
`8bcfc25ca7eee86681fa1208a8bfc01da3e51b6505da6316b7e793aa0f08a897`,
found three implementation roots: one shared `harness` health-probe contract,
one `harness` restoration-before-authorization guard, and one
`evidence-finalization` path-serialization defect.

The health diagnosis is endpoint-exact. Core owns `GET /health` = HTTP 200,
`text/plain`, exact body `ok`; Supervisor owns `GET /health/ready` = HTTP 204,
empty body. The materializer queried Core through the Supervisor endpoint,
followed the resulting redirect to `/login`, accepted a 200 HTML document as
Core health, then rejected Supervisor's valid 204 response. Materialization,
smoke, upgrade/rollback, Supervisor owner verification, containment, and final
restoration now share `scripts/sprint-8a-health-contract.ps1`, one
redirect-disabled contract helper. Broad 200/204 acceptance is not the
correction: each owner must match its own exact contract.
The retained false-Core-health HTML body is 4,710 bytes at SHA-256
`f00cefaa866bef8a84a0dff0ecaa73c6437ab295e961c20779d5b31a9cc4a574`.

R32 also exposed the lifecycle enforcement gap. Its attempt recorded
`cleanup_restoration.result = not_proven`, recovery recorded `restored=false`,
and the mandatory `final-successor-health` lane was blocked, yet the old harvest
helper issued schema-2 authorization SHA-256
`8b8a44ec3c76d441495bb100e7268dfbfc9e4d400d63dc7956f5db17f81829f5`.
Prospectively, complete terminal accounting may retain a harvest and batch in
that state, but `-HarvestOnly` emits no authorization. Schema-3 authorization
requires exact passing current-attempt `final-successor-health` and
`final-environment-identity` receipts and result
`canonical_successor_healthy`.

The other receipt enforcement gap was narrower than the already-sufficient
two-wave policy. R41 persisted six rooted Windows declaration paths and R32
persisted twenty-one; R32's immutable start also retained lane names rather
than the complete declaration graph. New starts persist the full lane contract
and canonical repository-relative forward-slash paths inside the evidence root;
resume/recovery requires exact full-graph equality. No historical receipt is
retrofitted or rewritten.

Post-harvest restoration generation 2 is retained at SHA-256
`e082eefd3be3aeb6dffa1e639047d7173b3306ba530848271625074a29f426e2`.
It proved source-exact images, first apply, semantic no-op, exact Core and
Supervisor health, canonical topology, five exact Core transitions, Dashboard
exactly once through its real Module Release/Instance, and no duplicate
navigation. Its platform-health evidence SHA-256 is
`228e3e9a1ce72ee98e5e98c83ce64ef0bd783f298bf2491e7e4ee7ceb7af1050`;
its topology evidence SHA-256 is
`3196c746b49ef25e953cf30d48f2c912a5452e9674ac7fa12dbd3a187b401fdc`;
its contract-aware product diagnostic SHA-256 is
`1711c1e582306446508f66e87dbd69b9057a2c69771039941ad29426ae11465f`.
The existing smoke then truthfully retained one `harness` failure at SHA-256
`86acf08d770c43d68efbbbeb0d0240aced2e384657d8e145dc5b1ed686b78ebc`:
it required bootstrap-only `placement_key` in the public response. The stronger
replacement asserts opaque placement IDs, exact geometry/resolution, six exact
authorized Component references, and restricted-placement nondisclosure.

That diagnosis also found one real `product` defect: Dashboard bootstrap wrote
an unversioned placement config, so all seven rows were read through fallback
geometry. The correction stores the canonical V1 layout and validates unique
IDs/keys, dense positions, and the exact one-based row/column/width/height
matrix: 1/1/4/2, 3/1/12/6, 9/1/6/4, 9/7/6/4, and row 13 at columns 1/5/9 with
width 4 and height 2. No migration compatibility is retained because Tessara is
pre-production and these databases are disposable.

Together, the shared health contract, restoration authorization guard,
canonical-path/full-graph enforcement, stale smoke expectation, and Dashboard
layout persistence are five diagnosed correction findings handled as one
tracked correction batch. They do not create a second R32 defect-batch receipt.

The implementation-to-validation mapping for that batch is explicit:

| Governing validation clause | Implementation slice | Focused proof required before handoff |
|---|---|---|
| Fresh-baseline materialization and exact semantic no-op | `materialize-sprint-8a.ps1`, owner health helper, Supervisor health client | clean disposable materialization receipt, first apply, no-op apply, exact Core/Supervisor health |
| Failure containment, recovery, and canonical restoration | failure-containment runner and final cleanup sink | injected failure receipt, clean successor materialization/no-op, inventory/navigation, final health |
| Receipt-bound Dashboard seed and nondisclosure | Dashboard bootstrap/config writer and smoke projection | exact seven IDs/geometries, six Component references, restricted nondisclosure, no `placement_key` |
| Complete fail-late terminal accounting and one consolidated batch | two-wave scheduler, runner recovery, harvest guard | adversarial ordering, dependency, deferral, process-loss, missing-lane, and exact-counter self-tests |
| No deferred or unauthenticated evidence may authorize certification | correction lineage, validation state, and preflight guards | canonical contained references, exact graph/lineage binding, deferred-result/preflight rejection |

These are implementation diagnostics only. They cannot create or replace a
passing Readiness receipt, complete Candidate Rehearsal result, candidate
freeze, SIT evidence, or UAT evidence.

The append-only qualification SHA-256 is
`859a8dba81792127820a12367e9d0430aaebb9bf7cb77442b50e34322ba4c7bb`.
It authenticates the diagnostic supplement and restoration and makes the old
authorization effective only as a tuple for one Readiness 42 consumption. The
qualification is not independently consumable. The sidecar-bound validation
state and its immutable snapshot are byte-identical at SHA-256
`85ac129f40204aef8d4b9a876c72c9ba948400da76e258b0da068c8afd443c8c`,
with preflight ineligible and the R32 lineage tip unconsumed.

The correction retains the bounded failure-first scheduler already specified
after R31 and closes its executable gaps. Wave A includes lifecycle,
prior-failed, never-executed/newly-reachable, impact-cone, three-times-deferred,
and prerequisite-closure lanes; safe siblings run fail-late. Aggregate lanes
remain sinks. If Wave A fails, mandatory cleanup/restoration still executes and
eligible Wave B prior passes become diagnostic-only `deferred` first; if Wave A
passes, the same attempt executes Wave B. Aggregate sinks follow that Wave B
disposition, then terminal canonical restoration and safety finalizers execute
regardless. Any unauthenticated history/hash/cone or counter selects
conservative full harvest, and any deferred lane keeps the attempt
failed/incomplete. Focused adversarial self-tests must cover deterministic
ordering, dependency closure, aggregate sinks, both Wave outcomes, the
three-deferral limit, impact overrides, recovery, cleanup, preflight rejection,
and complete-versus-missing terminal accounting. R9-R11 materialization and
R19-R22 Playwright histories remain regression fixtures only.

Process-loss recovery now covers both sides of the pre-authentication lifecycle
boundary. It accepts the exact lifecycle placeholder receipt, checkpoints the
authenticated source/environment identity before a nonterminal attempt resumes,
and preserves a terminal pre-authentication failure's recorded identity through
harvest. Final cleanup cannot pass from healthy leftover services alone: its
hashed materialization receipt must prove the exact attempt, source,
environment, first apply, semantic no-op, and final health before the cleanup
lane can authorize correction.

### Historical Readiness 41 pre-start rejection and lineage-preserving correction

The R41 launcher evaluated clean source
`231bd8d9ea1889870653d991b2cbaa9531074522`, tree
`359789b12ec3769e8a3b7cce43101d7b924acc11`, against the sidecar-bound R40
state at SHA-256
`2face70ef797e33aa597292c0c60de61949557d4edc4ba1c37e87ff6e6aa1ccb`,
then rejected during pre-start correction-lineage authentication. The retained
evidence is:

- launch log
  `artifacts/sprint-8a-closeout/attempts/readiness-41-prestart-launch.log`,
  SHA-256
  `886992e57ebcdfb18301b153a25f7397f06544d3e3cd7c6bcc0ec0cc40e3aa73`;
- diagnostic log
  `artifacts/sprint-8a-closeout/attempts/readiness-41-prestart-diagnostic.log`,
  SHA-256
  `cd6e080ea2b3c1eb982cc17f0e303f0b805d800d39c112348124358996c8bb9c`;
- typed failure
  `artifacts/sprint-8a-closeout/attempts/readiness-41-prestart-failure.json`,
  SHA-256
  `8e5b6dcb01dc58b55f2cada5a08279cb5fad62cebfd0b06c5fc6a0329b081a2c`;
  and
- consolidated two-defect batch
  `artifacts/sprint-8a-closeout/attempts/readiness-41-prestart-defect-batch.json`,
  SHA-256
  `3c9279fab557470212a11957fabc018927f6e7a6277332ad87dddbe90e4f4426`.

The first defect is exact schema enforcement that recognizes historical
schema-2 Readiness terminals but not R40's schema-3 terminal. The second is the
missing lineage-preserving clean supersession path: clean rerun admission was
evaluated only when no correction lineage existed, while R40 is the legitimate
consumer at the tip of a complete three-link lineage. Corrected R41 must keep
that lineage and R40's consumption immutable, carry the unchanged lineage in
both its immutable start and terminal, create no new correction link or
consumption, and name immutable R40 through the exact sequential clean-
pre-rehearsal Readiness prerequisite edge. Downstream lineage authentication
may reach current R41 through that clean edge without relabeling R41 as the
authorization consumer. R41's own `predecessor_correction_authorization` and
`correction_consumption_receipt` remain null.

At that pre-start boundary, the sidecar-bound validation state remained the passing R40 state, with
Rehearsal ineligible and `preflight_eligible=false`. There is no
`readiness-41-start.json`, `readiness-41.json`, `readiness-41/` evidence
directory, or R41 correction consumption. The pre-start records do not rewrite
R40 or any historical receipt. The final tracked correction bytes passed the
Readiness, Candidate Rehearsal, Validation Preflight, and complete Sprint 8A
acceptance-contract self-tests; a file-backed supersession regression proves
one- and two-edge chains plus the failed-descendant and adversarial rejection
cases. PowerShell parsing, retained R40 admission, Markdown links, formatting,
all-target/all-feature compilation, and warnings-denied Clippy also passed.
The receipt projection also keeps lineage null for a Readiness that directly
consumes a pending authorization, so validation state can bind the final
terminal hash without embedding a stale self-reference; only clean successors
carry the already-complete preserved lineage.

The documented next action at that boundary was to authorize the same unused
R41 identity so a passing R41 could issue the then-unused R32 schedule.

### Readiness 40 pass and Rehearsal 32 pre-start rejection

Readiness 40 consumed the exact Rehearsal 31 correction authorization and
passed all 15 checks. Its immutable receipt and canonical alias are
byte-identical at `artifacts/sprint-8a-closeout/attempts/readiness-40.json`,
SHA-256
`f4fd1745606a0094f648aa92b8fdca97bdc5e0101e2b74179f274e5614e1b939`.
It fixed Rehearsal 32's conservative full-harvest schedule at SHA-256
`7b09c1b33568e6b1bf16b9347efd31d9d99291fc40176d7e4de7d8aa1c3a7f22`.

The Rehearsal 32 launcher rejected that schedule before creating either
`candidate-rehearsal-32-start.json` or
`candidate-rehearsal-32-attempt.json`. The canonical runner declarations are
`OrderedDictionary` values, while the validator inspected only exposed
`PSObject` properties. The retained launch log is SHA-256
`e0f2370f97a49cde7cf9ddc37c44a18d731c57f7dbfef0d9f4c4971c0904016b`;
the typed pre-start failure is SHA-256
`db5710cacf602e0ad7a9f853561e59b504778f666ef40a7f165f86f4986f880f`;
and the one-defect pre-start batch is SHA-256
`44c22ea8caa3f5b5ee0b09e15da5d9b485efb51c6ac20e77ac4841b63951e17a`.
The two empty reservation directories left by the rejected launch were
authenticated as empty and removed; no receipt or raw evidence was removed.
Because no attempt began, terminal lane accounting, harvest, and rehearsal
correction lineage do not apply. The coordinator's clean pre-rehearsal boundary
decision supersedes R40 and requires a complete R41 before the same unused R32
identity may launch.

The correction makes scheduler declaration-member validation work for both the
live `IDictionary` representation and object-backed test inputs, and changes the
adversarial scheduler fixture to use the live ordered-dictionary shape. The
validation protocol already required immutable pre-start scheduling and
fail-closed launch; the enforcement gap was the divergent self-test
representation, so no duplicate protocol rule is added.

### Readiness 39 pass, Rehearsal 31 failure, and two-wave correction history

Readiness 39 passed against the exact R31 source/environment pair. Its
immutable receipt and canonical alias are byte-identical at
`artifacts/sprint-8a-closeout/attempts/readiness-39.json`, SHA-256
`b28da23ad7d3cc3046e3331f85f6616858ec4d14e46071725ad12015b1e80916`.
It does not authorize preflight because the immediately following Rehearsal 31
failed and the tracked correction changes validation interpretation and
candidate inputs.

Rehearsal 31 retained its complete fail-late attempt at
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-31-attempt.json`,
SHA-256
`e6efd0e5f480de39e4474e3a3e6c1bd13ae292885591ec10f5621e68ce3bd611`.
All 32 declared lanes are terminal: 19 passed, 3 failed, and 10 blocked. The
typed harvest is SHA-256
`ac2aae8d20ab5d67ed244e73acd142a56e997904b72afafcb957e7adfd92c365`;
the one three-finding defect batch is SHA-256
`20109843c4e15a79792f7f8eb8a2521b6ae3ffea371996f98acad72bc9d365cf`;
and the exact-successor-Readiness authorization, consumed once by R40, is
SHA-256
`9f6c60a81d0ecb538a49113fc98b69e0f1d8ebb63f09f5424eef2b113f3b9aae`.
No historical receipt is rewritten by the correction.

The retained raw findings remain exactly classified by the failed attempt:

1. `workspace-tests` — `product`; diagnosis found that the enrollment test
   reused a non-fresh `TEST_API_ENROLLMENT_DATABASE_URL` generation. The
   product expectation remains valid; Readiness must reject user tables in all
   six disposable test databases before another gate.
2. `source-exact-materialization-no-op` — `product`; the Supervisor Compose
   adapter lacked the four module targets required by the applied plan, and
   supplemental diagnosis exposed a materializer parser that did not accept
   every valid normalized Compose JSON/array/NDJSON form.
3. `failure-containment-successor-health` — `harness`; the containment runner
   must retain and authenticate an unexpected precursor failure before it can
   evaluate the intended semantic fault and successor recovery.

The ten lane blocks and eight nested UAT blocks retain their exact dependency
reasons. Post-terminal restoration diagnostics brought up the complete owner
topology and proved a semantic no-op through corrected temporary inputs; that
work is diagnostic only and does not alter R31 or satisfy a gate.

The correction introduces one bounded failure-first two-wave scheduler. Its
launcher first acquires the exclusive reservation; contention rejects launch
without attempt evidence. Under that retained handle, it captures and attempts
to authenticate state/Readiness scheduling inputs, selecting conservative full
harvest if they cannot support bounded deferral. Its create-once start receipt
then fixes the lane decisions and deterministic order before declared source or
environment probes. The Wave A attempt-state lane authenticates the retained
handle, requested attempt, current state, and lifecycle transition. Wave A
prioritizes prior failures, newly reachable and impact-cone lanes, mandatory
lifecycle/cleanup, lanes deferred three times,
and only their required prerequisite closure. Wave B contains authenticated
prior passes outside the cone. A Wave A pass continues into Wave B in the same
attempt; a Wave A failure completes safe Wave A harvesting and mandatory
siblings, records eligible Wave B lanes as diagnostic-only `deferred`, then
terminalizes aggregate sinks before mandatory restoration and safety
finalizers. Aggregate lanes remain sinks and do not reverse-expand Wave A.

Complete Candidate Rehearsal is the final certification pass, not the primary
debugging loop. Its deterministic Wave A order puts the two cheap lifecycle
prerequisites first, then source-exact materialization and containment/recovery
as the first expensive branch; stable expensive checks remain later unless a
real prerequisite or correction impact requires them. A lane with two
consecutive full-attempt failures must first pass its clean focused reproducer;
a third consecutive failure is a concentrated validation-platform incident
that must be resolved before another full launch.

The immediate post-R31 cycle must use the conservative full-harvest form of
Wave A. R31 predates immutable scheduler decisions and authenticated deferral
counters, while this correction changes the scheduler, validation skills,
runner, acceptance contract, fixtures, environment contract, and deployment
inputs. Its prior lane passes therefore cannot be promoted into Wave B. The
retained R9-R11 materialization and R19-R22 Playwright histories are projected
through `scripts/fixtures/sprint-8a-rehearsal-history-regressions.json` as
regression fixtures for authentication/fallback behavior only; they are not
reusable proof. Any later attempt may defer a lane only from a fully authenticated
two-wave predecessor, and no lane may be deferred more than three consecutive
times.

Focused implementation verification must prove adversarially that:

- the immutable start fixes deterministic failure-first ordering and safe
  dependency closure;
- aggregate sinks do not reverse-expand Wave A;
- Wave A failure defers eligible Wave B lanes, while Wave A success continues
  into Wave B in the same attempt;
- three consecutive deferrals force execution next time and any impact-cone
  change overrides Wave B eligibility;
- deferred lanes cannot appear in a passing result or authorize preflight;
- mandatory cleanup/restoration still executes;
- process-loss recovery preserves ordering and counters; and
- harvest and correction authorization accept complete eligible deferred
  accounting but reject a missing lane.

That historical implementation phase ran focused correction checks only. Its
next formal boundary was a fresh complete Readiness 41 and then the then-unused
Candidate Rehearsal 32; both later ran as recorded above.

### Historical Readiness 38 terminal failure and correction

Readiness 38 retained all 15 terminal results at
`artifacts/sprint-8a-closeout/attempts/readiness-38.json`, SHA-256
`7dcdc6a3d9866311ba12f251b080a4fe732b3fe10d1eb0ff6ad7de1d44a0f86d`.
Its exact result is 12 passes, 2 failures, 1 block, and 14 assertion-bearing
checks. The two failed-check logs are:

- `artifacts/sprint-8a-closeout/readiness-38/compose-database-contract.log`,
  SHA-256
  `f69a6e944f30e1ce3dc998cf212a28c09462b0128de2e9d14c0d9bd074bcd57a`;
  and
- `artifacts/sprint-8a-closeout/readiness-38/runner-self-tests.log`, SHA-256
  `db508e3b5958bf3dce71af76d64fe053feb068d980f39bd86949461c02ab0c9f`.

`environment-contract` did not run because
`compose-database-contract` failed. Its retained dependency reason is exactly
`blocked by failed prerequisite(s): compose-database-contract`. The R30
authorization consumption is retained with SHA-256
`867f2628800584bb59bbe287c02673a852c4007320dc9608620b78dbb739fb79`.
No R38 evidence is rewritten by the correction.

The plan-to-source audit consolidated six correction areas:

1. Bind all six disposable database URLs, reset acknowledgement, and validation
   container identity in the same Readiness process.
2. Prove that failure-containment rejects overwrite of a deliberately corrupt
   no-journal pair, then delete only that exact pair inside the validated
   temporary self-test root and republish it create-once. Real corrupt retained
   evidence remains preserved.
3. Add one typed failed-Readiness harvest, consolidated batch, exact-next-
   attempt authorization, and append-only correction lineage so consumed
   predecessor authorization cannot strand a failed successor.
4. Preserve immutable Rehearsal declarations separately from exact terminal
   results and checkpoint every completed pass, failure, and block.
5. Reject the retired Component `missing_policy` key and preserve only the four
   purpose-specific current keys throughout the active first-party surface.
6. Bind the eight manual UAT scenarios to exact tester, precondition, evidence,
   AC-18/AC-19, semantic-predicate, and document identities, and enforce that
   contract before preflight and formal publication.

That source remained mutable until the consolidated correction was committed.
R38 was then finalized without rerunning its checks, R39 passed, and R31 failed
as recorded in the current section above. These historical receipts remain
immutable and do not authorize the post-R31 lifecycle.

### Readiness 37 and Rehearsal 30 historical consolidated correction

Validation Readiness 37 passed its complete 15-check graph with zero failures
and zero blocks. Its result is retained at
`artifacts/sprint-8a-closeout/validation-readiness-result.json`, SHA-256
`a4ab361af6606ff94eaef8c5be5ca3e864dc90cb41efabdcb3aba73014eacf9b`.
Because runners, acceptance contracts, validation references, and Sprint
documentation now change, that mutable result is superseded and cannot
authorize preflight.

Candidate Rehearsal 30 ran against that exact source and environment and
completed the full fail-late terminal graph: 18 lanes passed, 4 raw lanes
failed, and 10 were blocked by failed prerequisites. Cleanup/restoration was
`not_proven`, and no `candidate-rehearsal-result.json` was written. The raw
failed lanes remain immutable evidence:

- `source-exact-materialization-no-op` — `environment`;
- `failure-containment-successor-health` — `product`;
- `uat-diagnostics` — `harness`; and
- `final-environment-identity` — `environment`.

The ten blocked lanes retain their exact dependency reasons:

- `deployed-inventory-navigation-audit`, `deployment-evidence`,
  `product-smoke`, and `component-upgrade-rollback` depend on
  `source-exact-materialization-no-op`;
- `playwright-execution` depends on `source-exact-materialization-no-op` and
  `deployment-evidence`;
- `live-product-diagnostics` depends on `deployment-evidence` and
  `product-smoke`; and
- `successor-inventory-navigation-audit`, `successor-deployment-evidence`,
  `successor-product-smoke`, and `final-successor-health` depend on
  `failure-containment-successor-health`.

Diagnosis consolidates the four raw symptoms under three `harness` roots:

1. Bootstrap action dispatch passed a string from `switch` where
   `set_enablement` required the complete action object's `enabled` field. This
   caused both materialization symptoms and the failure-containment cascade.
2. Source identity was returned as an ordered dictionary rather than the exact
   object shape required by automated-UAT receipt guards, producing 12 false
   prerequisite failures and 8 blocked scenarios.
3. Bootstrap did not restore seven process environment variables used for
   signing, source identity, and installation identity. A fresh external
   environment recomputation after process exit matched R37 exactly, proving
   harness leakage rather than persistent environment drift.

The same audit found four evidence-enforcement gaps: blocked terminal receipts
were counted as assertions, structured child classification could lose to a
lane default, final environment mismatch evidence omitted expected/actual
comparison detail, and already-retained authorization paths were not uniformly
canonical. The correction adds exact per-lane assertion-start evidence,
structured-classification precedence with a named source, secret-free compared
fingerprints and changed sections, and contained repository-relative paths for
new evidence. The immutable in-root R30 authorization was consumable once
without rewriting it or permitting an outside-root reference; R38 consumed it
and retained that consumption.

The retained evidence is:

- attempt: `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-attempt.json`,
  SHA-256
  `9ce1bba709b0b4bf1eee23cd0eea406fd49f16152cd25d45ea9dc3dac98617b5`;
- harvest: `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-harvest.json`,
  SHA-256
  `6167ab682922e7cfe582c9b461f38e31f4260c6259fffd3c3877eef7b62e3d13`;
- defect batch:
  `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-defect-batch.json`,
  SHA-256
  `397260d3b4382ddc4731a2c8f7bcc3341ed5f6bc3b77d3dcaf8b03cf3cb36291`;
  and
- correction authorization:
  `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-correction-authorization.json`,
  SHA-256
  `efed4d4c53d84454372936067a5a9c454457cf2f42419afe331fc89a8d154792`.

The authorization permitted exactly one successor Readiness start. Readiness
38 consumed it and failed, so it cannot be reused. The resulting second
append-only lineage link authorized only Readiness 39; R39 consumed it and
passed. Rehearsal 31 then failed and issued the authorization later consumed
exactly once by passing R40. R40 is now superseded by the retained R32
pre-start correction and the subsequent retained R41 pre-start lineage/rerun
correction. Readiness and Rehearsal must both pass against the same corrected
clean source and fresh six-database environment before preflight.

### Prior user-directed return from Readiness 36 and Rehearsal 29

Validation Readiness attempt 36 passed its complete 15-check graph against
clean commit `3f7e32cb7e1948a36185a68352901b873934c0b0`, tree
`252e651e0d031c04d61547b6d6eac964bb04f84f`, and environment fingerprint
`2e235b070dd2f3663242fe5b3f983f561b834161ed65847ad6dd399d35e01da0`.
It recorded zero failures and zero blocks. Its mutable result is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-36.json` with SHA-256
`f10f4c63d96e8bf484f9a19084359e58cf81858adcfb5e4bf3cb49b84c716710`.
That pass describes only the prior clean source. The harness, fixture,
contract, product, boundary, test, and documentation changes required below
are candidate-affecting and supersede it.

Candidate Rehearsal attempt 29 began against the same source and environment
identity. The user-directed phase exit stopped the attempt before a terminal
harvest: 8 lane receipts passed, 3 failed, the independent
`optimized-resource-reference-timing` lane was interrupted while executing,
20 declared checks were never executed, and no blocked receipt had yet been
recorded. The nonterminal attempt receipt and raw lane evidence remain at
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-29-attempt.json` and
`artifacts/sprint-8a-closeout/rehearsal/attempt-29/`. The zero recorded blocks
must not be read as 20 independent passes or as a complete dependency harvest;
the checks are unexecuted because testing was stopped.

The three failed receipts and their retained diagnostic classifications are:

- `dashboard-source-boundaries` and `markdown-links` each recorded a `product`
  lane failure, while their raw child-script logs explicitly reported success.
  One stale parent `$LASTEXITCODE` was reused after each PowerShell script
  invocation. The two receipts are retained, but the consolidated root defect
  is `harness`; neither is evidence of a product-boundary or Markdown defect.
- `workspace-tests` retained its failed `product` lane receipt and the exact
  `extracted_component_product_owns_crud_versions_lifecycle_render_and_nondisclosure`
  failure. Its Table fixture declared only `label` as required while its
  configuration used both `label` and `amount`; the product correctly returned
  `400 component.bad_request` before provider invocation rather than the
  fixture's expected provider-not-ready `503`. The consolidated root defect is
  `harness` with test-fixture subtype; the product validation must not be
  weakened.

The plan-to-source audit then expanded the same return-to-implementation batch:

- `product`: `tessara-components-contract` must own the exact render response
  DTOs and render-kind discriminant; Component provider/product routes return
  them, Dashboard validates them, and the placement renderer has neither a
  direct Dataset-contract edge nor local wire copies. Table and every visual
  kind reject another kind's fields/branch. The output omits the unused Dataset
  identity.
- `product`: persistent execution requires matching non-nil Component and
  ComponentVersion identities. Unsaved preview alone uses an explicit both-nil
  identity; a partial-nil identity is invalid in either path.
- `product`: Component, Core authorization-exchange, and Core Dataset service
  receivers must verify the exact raw-body digest, authorization/service
  envelopes, and correlation identity before typed deserialization while
  retaining JSON media-type and body-size enforcement. A semantically equal
  but byte-different unsigned mutation fails closed.
- `product`: Dashboard-mediated rendering requires one common governing node
  across stored/actor-authorized Dashboard scope, Component scope, inbound
  Dashboard authority, and downstream Component authority. The exchange is
  bound to the exact ComponentVersion type/id/authority revision/canonical
  scope assertion, which Component compares with its row. Disjoint metadata,
  title, scope, Dataset identity, and render output remain undisclosed.
- `harness` / acceptance inventory: active permissions, general smoke, and
  general UAT helpers must not normalize retired Core Component shapes or flat
  Dataset-major fields. First-party fixtures use exact v3 identities and typed
  references, enforced by a source guard.

The earlier identity-based Dashboard correction remains controlling: Core has
exactly five transitions (Forms, Workflows, Responses, Datasets, Migration),
and Dashboard appears only through its real Release/Instance and Manifest at
reference order 9, after Scoped Records 7 and Components 8. It is retained as a
`product` defect correction rather than reduced to another copied count.

The prior testing-entry correction included the observed child-exit and fixture
harness defects plus the related product, security, acceptance-inventory,
transition-ownership, and fail-late enforcement cone. The tracked Playwright
inventory is 75 exact identities, with the shared-node success/disjoint-node
redaction and render-denial scenario mapped to UAT-8A-04. That correction was
exercised by the implementation-level checks below and subsequently reached
Readiness 38, whose terminal failure and completed correction are recorded at
the top of this file as historical context. R38 was finalized, R39 passed, R31
failed, and R40 passed before the R32 pre-start rejection exposed its
declaration-shape harness defect. The corrected R41 launch then rejected before
reservation on the two lineage/rerun harness defects retained above. The next
boundary remains the coordinator-authorized corrected Readiness 41/Rehearsal 32
cycle described below; both attempt identities are unused.

Implementation-level verification then reset and recreated all six isolated
readiness databases and passed on the final corrected product source:

- `cargo test --workspace --all-features --locked --offline` in 619.6 seconds;
- all-target, all-feature `cargo clippy` with warnings denied, `cargo check`,
  `cargo fmt --check`, and `git diff --check`;
- Components contract, Component module, Dashboard module, Component product
  integration, and exact-body API checks;
- the Sprint 6E boundary and Sprint 8A acceptance contracts, Markdown links,
  candidate-rehearsal and UAT runner self-tests, and PowerShell parser checks;
- exact 75/75 acceptance inventory, TypeScript 5.9.3 compilation of the changed
  specifications, and Playwright discovery of all 75 tests.

These are implementation-entry checks, not formal Validation Readiness or
Candidate Rehearsal receipts. Documentation-only handoff edits followed the
Rust run and are covered by the final formatting, link, boundary, acceptance,
and diff audits. A complete formal gate pair remains mandatory on one clean
committed source and one verified environment before preflight.

### Validation Readiness attempt 35 evidence-finalization correction

Validation Readiness attempt 35 ran against clean commit
`49c0ae73c3c82dd12b3e2fea68d53b5d497b3b74`, tree
`57f07e9cb39721ea994cf462c3753eb5b500b4e0`, and environment fingerprint
`bfcbf8df8e719b581152dfd6e7ed3d74baa047417b6e1f9f10c001bd79f8efda`.
All 15 declared checks reached terminal `passed`: zero failed, zero blocked,
and zero skipped. The six authenticated database probes, exact 74-test
Playwright inventory, property-safe Compose self-test, captured materializer
WhatIf, package/metadata/link checks, and final clean-source check all passed.

After the final check checkpoint, terminalization evaluated
`$failures.classification` under StrictMode while `$failures` was empty. The
resulting `PropertyNotFoundException` occurred before the terminal attempt
receipt, canonical `validation-readiness-result.json`, or terminal validation
state could be published. The last live receipt therefore remains
non-authoritative `executing`; it is not a Readiness pass, Candidate Rehearsal
29 remains ineligible, and preflight stays closed.

The immutable start receipt SHA-256 is
`773410def7e4e95a1fc2f5a1b3cdc6cb4e9095a8b09137b2da35b8cb7a887e7f`;
the last live receipt SHA-256 is
`b98435d822b14253e82638dcba907fe788c6bf01c6f824431770ba6e915f9589`.
Every lane log, produced artifact, receipt, and state sidecar verified, and the
attempt lock was released. The previously missing process failure is retained
at `artifacts/sprint-8a-closeout/readiness-35/finalization-failure.log` with
SHA-256 `55c153e41c98fa0e380573f380a035a7d50ecbefb6404ea0b32225d4057789ee`.

One consolidated finding is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-35-consolidated-correction-batch.json`
with SHA-256
`827ffd273282c2fb149003f2625508cc27814c2c4162ad196586f1359e0ed454`.
Its classification is `evidence-finalization`; no product assertion or product
action failed. The bounded sibling audit found the same empty-array success
path in Candidate Rehearsal terminalization. The correction projects nonblank
classifications through one StrictMode-safe helper in both runners and
self-tests empty, single, duplicate, missing, null, and multiple inputs. The
acceptance contract enforces the helper and both runner call sites.

The validation protocol already required atomic finalization and complete raw
evidence. The enforcement gap was executable zero-failure terminalization
coverage, so the correction belongs in runners, self-tests, and acceptance
rather than duplicated protocol prose. Because those tracked files changed, a
new complete Readiness result and then a complete Candidate Rehearsal result
remain required against the same corrected clean source and environment
identity.

### Validation Readiness attempt 34 consolidated correction

Validation Readiness attempt 34 ran against clean commit
`c920b8a3dd1e35d5617e5f3d634eabff75499e77`, tree
`ba5c5f764d7d4b49d15417a1edd08cfdab16e0b1`. All 15 declared checks reached a
terminal state: 12 passed, `compose-database-contract` and `reset-dry-run`
failed, and `environment-contract` was blocked exactly by the failed
prerequisite: `blocked by failed prerequisite(s):
compose-database-contract`. Candidate Rehearsal 29 did not start.

The terminal receipt is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-34.json` with SHA-256
`f2b377e4a1fee943d5940fed25d4c6d03f807820001ea429a14711a5b516867c`.
Raw evidence remains at
`artifacts/sprint-8a-closeout/readiness-34/compose-database-contract.log` and
`artifacts/sprint-8a-closeout/readiness-34/reset-dry-run.log`. Start, terminal,
state, raw-evidence, and Playwright-inventory hashes all verified; the attempt
lock was released. No product or destructive database action began in either
failed lane.

The complete harvest produced one two-finding correction batch at
`artifacts/sprint-8a-closeout/attempts/readiness-34-consolidated-correction-batch.json`
with SHA-256
`dd8ffbdb742e184c76fb448fb58ddb720bb01efcfe0919a4bbfdf0d2360efa58`:

- `environment`: the attempt process lacked all six pairwise-distinct test
  database URLs, the exact disposable-reset acknowledgement, and its inspected
  PostgreSQL client container identity;
- `harness`: the materializer read normalized Compose's optional `ports`
  property as mandatory under StrictMode; the same bounded audit found the
  latent optional `environment` and resource-name assumptions.

The correction centralizes optional-property handling and normalized service
projection, covers services without ports or environment in an executable
runner self-test, and binds those helpers in the acceptance contract. The next
attempt must receive six fresh loopback databases and the exact reset/client
contract in the same PowerShell process. These tracked runner, contract, and
documentation changes invalidate attempt 34 as an authorization source; a new
complete Readiness result and then a complete Candidate Rehearsal result remain
required against the same clean source and environment identity.

### Post-Readiness-33 implementation completion audit

The user stopped the testing phase after Readiness 33 and required a fresh
plan-to-product review before another testing entry. No formal lifecycle gate
was run during this audit. It found one consolidated candidate-affecting batch
that crossed product, contracts, baseline schemas, deployment, fixtures,
acceptance tests, UAT evidence, and runner enforcement:

- the public gateway could be available while owner materialization was still
  mutating the disposable topology;
- Component bootstrap validated only local shape and could persist before the
  Core-owned Dataset provider evaluated the exact references and compatibility;
- normal Component create did not express Component type requirements in the
  provider compatibility request and did not honor an already-successful
  idempotent replay during a later provider outage;
- malformed configuration shape was projected as provider unavailability, and
  Module Management discarded the module's sanitized diagnostic details;
- Dashboard's composition surfaces could project a synthetic outage as
  provider-unavailable without a matching current disclosure basis, while
  finding/action/recovery episodes were insufficiently isolated by semantic
  actor, scope, authorization revision, organization revision, and expiry;
- the shared UAT fixture path could still write owner-controlled product tables,
  automated UAT could evaluate an embedded summary instead of authenticated raw
  evidence, and exact browser configuration/outage/responsive scenarios were
  absent from the acceptance inventory; and
- Candidate Rehearsal's nominally safe static lanes still depended on the
  fallible readiness prerequisite despite the protocol's fail-late rule.

The implementation now keeps the gateway offline until owner completion; uses
an exact locked-Manifest-derived, apply-bound, one-time signed bootstrap
authorization for real Dataset validation; performs provider compatibility and
exact-reference, materialization-readiness, and scope checks before Component
writes; preserves authoritative mutation replays;
projects configuration failures and sanitized diagnostics correctly; and binds
Dashboard disclosure, findings, actions, recovery, and repeat outages to the
exact unexpired semantic authorization context. Owner-controlled fixture
boundaries are AST-enforced, UAT predicates compare authenticated raw evidence,
the acceptance inventory carries exact browser and placement identities, and
the rehearsal graph keeps safe static siblings independent.

These changes supersede the mutable source exercised by Readiness 33. Focused
formatting, parsing, contract self-tests, discovery, compile, Clippy, and narrow
owner integration checks are implementation evidence only. A new complete
Validation Readiness result and complete Candidate Rehearsal result must both
pass against the same clean source and environment identity before preflight.

### Validation Readiness attempt 33 diagnostic correction

The first complete gate against implementation commit
`0cf7dfa0fda3f30cca2b1c1c7f2181ca0ad43045`, tree
`490da6f62b41eb26dfd6458dc76e51fe2bc23ca0`, and environment fingerprint
`6d8b4753386ca703037a49e08758044d9720f86145bb82c644aff81d3b698de8`
finished fail-late with 12 passed checks, two failed checks, and no blocked
checks. Its source-bound receipt and raw logs remain under
`artifacts/sprint-8a-closeout/attempts/readiness-33.json` and
`artifacts/sprint-8a-closeout/readiness-33/`. Candidate Rehearsal attempt 29
did not start.

The two failed checks exposed three harness defects: the readiness runner
called the Component baseline builder and upgrade verifier through removed
parameter contracts, and the materialization safety guard treated Compose's
optional `external` network property as mandatory under strict mode. The
complete bounded runner audit found two additional enforcement defects before
correction: Candidate Rehearsal could fail prerequisite validation before
publishing its start receipt, and its declared lanes omitted the required
release-only known/random resource-reference timing proof. The controlling
five-finding consolidated batch is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-33-consolidated-correction-batch.json`;
all findings are classified `harness`.

The correction advances every caller to the current upgrade self-test
contracts, treats an omitted Compose `external` property as `false`, moves
readiness/source/environment verification inside the receipt-governed
prerequisite lane, and adds an independent
`optimized-resource-reference-timing` lane using the exact release-mode test.
The acceptance contract and runner self-test enforce the new lane and the
start-receipt boundary. These tracked harness and documentation changes are
candidate-affecting, so attempt 33 is superseded for authorization. A complete
unused Validation Readiness attempt and then a complete Candidate Rehearsal
remain required before preflight.

### Post-attempt-28 implementation correction

On 2026-08-06 the user explicitly stopped testing after prolonged failure to
reach preflight and returned Sprint 8A to implementation. The source identity
at exit was commit `7b326838269c3b4218dd579f649a8011feba4d97`, tree
`fa78175797fddba7c7a0b14bb141a33cf4d40fd0`, clean. The append-only local
invalidation receipt is
`artifacts/sprint-8a-closeout/attempts/implementation-return-2026-08-06-invalidation.json`
with SHA-256
`39da96d3672ebd078c551e3179a1b3cf97a4a13671a61627425feed9e894e150`.
It supersedes readiness 31, marks readiness 32 invalid, and records
user-directed abandonment of rehearsal 27 and 28. Candidate freeze, preflight,
SIT, and UAT remain prohibited.

Attempt 28 ended before a protocol-complete harvest because testing was
explicitly exited. Its partial raw evidence remains non-authoritative in
`artifacts/sprint-8a-closeout/rehearsal/materialization-attempt-28.log` and
`artifacts/sprint-8a-closeout/rehearsal/full-validation-attempt-28.log`. Three
observed harness/evidence failures were retained rather than silently retried:

- Validation Readiness parsed database URLs without authenticated reachability
  and produced an environment receipt that Candidate Rehearsal could not
  compare structurally.
- Automated UAT accepted a stale failure-containment receipt by existence
  instead of binding it to the current attempt, source, environment, time, and
  content hash.
- Attempt startup tried to add undeclared `assertions_started_at` data to its
  receipt schema.

The Dashboard duplication is retained separately as a product/architecture
defect, not reclassified as a count-only test failure. Its raw classification
is preserved in
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-4-defect-batch.json`
(`overall_classification: product`), with its source-exact invalidation in
`candidate-rehearsal-4-invalidation.json`. That evidence records why the
earlier seven-to-six correction was incomplete: Dashboard simultaneously
appeared as a frozen Core transition and an enrolled Module Release/Instance.
The corrected contract is the exact five Core transition identities, with
Dashboard supplied only by its enrolled manifest. Narrow correction evidence
never discharged the requirement for a new complete Readiness and Candidate
Rehearsal cycle.

The subsequent sprint-plan/architecture audit confirmed one consolidated
implementation-readiness correction batch spanning product, fixtures, tests,
deployment inputs, and validation runners:

- Core's transition catalog is exactly Forms, Workflows, Responses, Datasets,
  and Migration. Dashboard is removed from every canonical transition input
  and can appear only once through its real Release/Instance and manifest.
  Inventory now rejects transition/release identity overlap rather than
  suppressing the duplicate after composition.
- Generic signed-bootstrap receipt bindings carry exact Component owner
  read-back identities into Dashboard seed input before input hashing and
  idempotency. No copied Dashboard-to-Component seed identity or
  Dashboard-specific orchestration branch remains.
- Every module-to-module hop exchanges the verified inbound grant through Core
  for the exact downstream audience. Enrolled manifests declare both the
  consumer request and provider service action; Core accepts only the exact
  lockfile-resolved provider/action/capability tuple and projected live service
  identities. Component and Dataset provider routes contain no sibling caller
  names, caller-specific keys, or first-use identity enrollment.
- Component provider authorization preserves known-versus-random resource
  nondisclosure, while Dashboard presents Components as an installed module
  dependency rather than a Core transition.
- Component-specific lifecycle handling is removed from Core's web host. The
  host renders generic module lifecycle contracts, and extracted Component
  integration coverage exercises the durable module database and API boundary.
- Current navigation and acceptance tests assert exact identities: the five
  Core transitions plus real Scoped Records, Components, and Dashboard
  contributions, ordered 7, 8, and 9. The resolved lockfile policy is
  transactionally projected and is the only mutable navigation source;
  historical seven-entry fixtures are explicitly historical and never reused
  as current expectations.
- Core's startup seed contains no Component or Dashboard capability, role
  grant, service-action, product-route, product-data, or product-asset
  declaration. Those capabilities/actions enter through release enrollment,
  and both products' CSS and JavaScript remain in their owner release images.
  Component direct-load SSR and hydrated visual identity tests now assert the
  owner document rather than obsolete Core loading placeholders.
- Validation Readiness authenticates six pairwise-distinct disposable
  databases and emits one secret-free source/environment fingerprint.
  `localhost`, `127.0.0.1`, and `::1` are one canonical loopback server
  identity for uniqueness, so aliases cannot make one database satisfy two
  required bindings.
  Candidate Rehearsal is now a repository-owned dependency graph with safe
  fail-late siblings, exact blocked reasons, append-only lane evidence, one
  consolidated batch per harvest, and correction authorization only after the
  harvest guard passes.
- Materialization records resolved targets, empty baseline, first apply,
  unchanged semantic no-op, owner receipt bindings, final health, failure logs,
  exact teardown, and source/environment identity. Deterministic failure
  containment proves a clean from-empty successor.
- Canonical bootstrap now has exactly seven Component shells, eight versions,
  and seven Dashboard placements. The Stat Card shell carries a
  superseded/inactive predecessor with a published/active declared successor;
  three independent Dashboard placements bind that predecessor for
  Defer/Upgrade, Replace, and Remove, while the blocked-scope placement remains
  distinct.
- The Dashboard dependency diagnostic exercises those exact live identities.
  It requires exactly three initial lifecycle findings, authorized successor
  disclosure, blocked-scope nondisclosure, Defer followed by Upgrade, Replace,
  Remove, the exact five remaining authorized placements during a Component
  provider outage, and healthy zero-finding convergence after recovery. Its
  structured receipt is a mandatory UAT-8A-05 diagnostic dependency; broad
  lane labels cannot satisfy the scenario. Its check-by-check JSON and SHA-256
  sidecar are retained even when the inner diagnostic fails, and the outer
  fail-late harvest embeds both the partial result and raw artifact identity.
  The independent broad sibling uses the canonical Sprint 8A ownership smoke,
  not the removed Core-era demo Component/Dashboard seed.
- Component calls the versioned Dataset compatibility operation and retains a
  sanitized observation of the selected binding, Core-installation provider,
  contract/version, compatibility, health, observation time, and stable
  result/failure codes. The Component Manifest declares configuration defaults
  `Components` and `5` and uses the real
  `/usr/local/bin/component-module` runtime/migration executable path.
- Component upgrade rehearsal now builds a distinct compatible `0.9.0`
  release from source. Exact one-owner Blueprint deltas are applied by the
  Supervisor through its Compose adapter for baseline establishment,
  `0.9.0` to `1.0.0` upgrade, rollback to `0.9.0`, and intended-`1.0.0`
  restoration; preservation snapshots precede each transition. Supervisor
  finalization commits receipt, operation, emergency override, and override
  reconciliation atomically; Core commits inventory and receipt projection in
  one transaction. Failed projection/finalization retires the unprojected
  receipt, terminalizes the operation, and permits an exact-sequence retry.
  An unchanged desired revision resolves to only `VerifyReadBack`, approves no
  effects, carries prior state, and marks carried bootstrap receipts unchanged.
- Playwright failure evidence and all eight automated UAT diagnostics are
  attempt-, source-, environment-, chronology-, and hash-bound. A failed
  browser run retains reports, traces, logs, test results, and error context.
  Component browser fixtures consume the canonical nested Core-owned Dataset
  major-line reference and derive its positive major from `<uuid>@<major>`;
  removed flat Dataset fields cannot satisfy the test contract.
- The Dataset compatibility contract, Core reference resolver, provider, and
  storage now share one exact major-line identity boundary: canonical
  `<uuid>@<positive-i32-major>`. Core rejects the obsolete colon spelling,
  alternate integer spellings, zero/negative versions, extra separators, and
  values beyond the signed storage range; provider paths contain no unchecked
  unsigned-to-signed major casts.

Focused implementation checks for this batch passed at handoff:

- `cargo fmt --all -- --check`;
- zero-warning `cargo check` and Clippy across all targets/features for the
  Dataset contract, Core API, Supervisor, composition engine, Component
  module, Dashboard module, and web host;
- Supervisor transaction/no-op/rollback tests, composition tests, all-feature
  Component library tests, Dataset-contract tests, and exact Core catalog,
  navigation, manifest-digest, restricted-reference, and Dataset-major
  boundary tests;
- parsing for all 15 changed PowerShell scripts and all four changed JSON
  documents, local Markdown-link validation, and Sprint 8A Compose
  configuration rendering;
- the acceptance contract plus Dashboard semantic, product diagnostic,
  readiness graph, candidate graph, harvest guard, and automated-UAT runner
  self-tests; and
- exact acceptance-manifest discovery of 74 Playwright scenarios, including
  six Sprint 8A Component UI scenarios and the exact Module configuration and
  diagnostic-authority scenario. This was inventory discovery only; no browser
  or deployed-system execution occurred.

These focused checks do not constitute Validation Readiness or Candidate
Rehearsal. Database-backed integration, live deployment, full Playwright,
preflight, SIT, and formal UAT remain Not Run. The next phase must run
`validate-sprint-8a-readiness.ps1` and then the complete repository-owned
`run-sprint-8a-candidate-rehearsal.ps1` against the same clean source and
environment identity.

### Earlier post-attempt-27 correction (superseded context)

The 2026-08-06 implementation audit reconstructed the sprint behavior from the
Sprint 8A acceptance criteria and the closed Sprint 4A/4B/7B Component
contracts instead of treating repeated harness progress as product readiness.
It identified one consolidated implementation-readiness batch:

- **Product lifecycle defect:** the extracted Component module permitted
  transitions outside the closed state machine, including draft lifecycle
  actions and invalid tombstone/reactivation paths.
- **Product authoring defect:** extracted validation accepted unknown keys,
  unavailable fields, invalid type/operator combinations, invalid visual
  modes and limits, and overlong version notes.
- **Product execution defect:** the cross-process Dataset execution path had
  lost table search, runtime projection narrowing, configured/runtime sort,
  complete filters, typed comparisons, unique count, median, do-not-summarize,
  display labels, smoothing, and value/dimension missing-policy semantics.
- **Architecture/fixture defect:** Sprint 8A reused a Sprint 7A fixture helper
  that directly mutated Core, Component, and Dashboard product tables after
  owner bootstrap. This crossed database ownership boundaries and could hide
  incomplete signed bootstrap inputs.
- **Acceptance/bootstrap defect:** the signed Blueprint did not own the full
  recognizable 30-row/blocked-scope Dataset seed and exact Dashboard
  placements, while Sprint 8A acceptance identities still named superseded
  Component versions. Core bootstrap also attempted to write a nonexistent
  Dataset-revision authority column.

The consolidated correction restores the lifecycle and authoring contracts,
extends the typed Dataset execution contract and Core adapter, moves all
Sprint 8A product seed into the owning signed bootstraps, makes the legacy
fixture helper account/security-only for Sprint 8A, and binds acceptance to
exact owner-bootstrap identities. The Dashboard bootstrap now establishes its
own authority revision, and the materialization audit asserts exact Component
external keys rather than a copied count. Validation Readiness now parses the
owner-controlled fixture preparer and Sprint 8A acceptance contract explicitly,
closing the enforcement gap that let seed-path syntax sit outside its runner
inventory.

Focused implementation verification for this mutable batch is recorded below.
These checks do not constitute Validation Readiness or Candidate Rehearsal:

- touched Sprint 8A PowerShell files parse: Passed;
- `cargo check --locked` for Dataset contract, Component module, Core API, and
  Dashboard module: Passed with zero warnings;
- focused Clippy for those four packages across all targets with
  `-D warnings`: Passed;
- Component, Dashboard, and Dataset-contract library tests: Passed (18, 13,
  and 3 tests), and the focused Core Dataset-adapter/bootstrap suites passed
  (3 and 1 tests);
- Sprint 7A/Sprint 8A acceptance-contract evaluation and owner-controlled
  fixture-preparer self-test: Passed.

A fresh complete Validation Readiness run and then a fresh complete Candidate
Rehearsal run are required against the same new clean source/environment
identity before preflight may begin.

Observed on the mutable `codex/sprint-8a` implementation tree on 2026-08-05:

- `cargo fmt --all -- --check`: Passed.
- `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings`:
  Passed with zero warnings.
- `cargo test --workspace --lib --bins --locked --offline` with the two
  database-dependent API tests filtered: Passed. The filtered tests require
  `TEST_API_ENROLLMENT_DATABASE_URL` or `TEST_API_DATABASE_URL` and remain part
  of validation readiness/rehearsal rather than being treated as skips.
- Component, Components v3, Dataset v1, Dashboard module/UI/placement renderer,
  composition, Core navigation/baseline, and module lifecycle focused suites:
  Passed.
- `scripts/check-web-crate-boundaries.ps1`: Passed.
- Sprint 8A PowerShell parser checks and
  `scripts/smoke-sprint-8a.ps1 -SelfTest`: Passed.
- `docker compose -f deploy/sprint-8a/compose.yaml --profile reference config --quiet`:
  Passed.
- Sprint 8A catalog sign, signature verification, and Blueprint resolution:
  Passed against the current canonical manifest digests.

The deployed smoke, destructive materialization, database-backed integration
tests, Playwright, failure teardown/rerun, upgrade/rollback, SIT, and all eight
UAT scenarios remain Not Run. Their execution is reserved for the specialized
validation and UAT workflows.

## Scope And Acceptance Inventory

| Roadmap clause | Risk/contract | Automated proof | Deployed smoke proof | Manual UAT proof |
|---|---|---|---|---|
| Independent full-stack Component module | Hidden Core/build/runtime coupling | native/WASM package graph, forbidden-source, image inventory, route-owner and manifest/testkit assertions | Component process/database/routes/assets healthy while Core product paths are absent | UAT-8A-01/06/08 |
| Canonical packages; no copied source or sibling implementation | Source ownership drift | dependency/source/package/image audits for Component and Dashboard | provenance and image-content read-back | UAT-8A-06 |
| Module owns configuration, manifest, capabilities, database/schema migrations, health, documents and assets | Incomplete extraction | manifest, configuration, capability, schema, probe, document and asset conformance | generic Module Management plus every product/operational route | UAT-8A-01/03 |
| APIs/contracts/typed references only | Cross-database or private DTO coupling | source/SQL/credential and exact-contract integration tests | deny cross-database credentials while product flow passes | UAT-8A-04/06 |
| Exact Components-owned render response | Copied wire DTO, mixed-kind payload or ambiguous identity | Table/visual schema and per-kind field matrix; persistent/preview identity tests; Dashboard exact decode; renderer dependency audit | every deployed kind returns exact contract output without Dataset identity | UAT-8A-01/04/05/06 |
| Exact signed request body | Parsed/reserialized bytes bypass the signed request identity | raw-body authorization-before-deserialization checks; whitespace tamper rejection; media-type/body-limit tests | altered unsigned request bytes fail across Component/Dataset service boundaries | UAT-8A-04/06 |
| Common Dashboard/Component governing node | Cross-product authority across disjoint scope grants | exact resource assertion, authorized-scope intersection, same-node and redacted-projection tests; exact Playwright identity | shared-node placement renders; disjoint placement exposes no title/metadata/identity and cannot render | UAT-8A-04/05 |
| No active legacy Component facade | First-party helper hides retired Core payload | source guard plus permissions/smoke/UAT exact v3 and nested Dataset-reference fixtures | active clients exercise exact current request/response only | UAT-8A-01/04/06 |
| Phase 8 fresh materialization from empty | Partial/stale transition state | reset-target, empty-baseline, owner-order, semantic seed and no-op tests | first source-exact bootstrap plus unchanged second run | UAT-8A-02 |
| Lockfile-owned bootstrap dependency validation | Product branching, payload reinterpretation, replay, or pre-validation writes | Manifest/lockfile/JSON-Pointer resolution; exact signed-binding and negative replay/mismatch tests; zero-write assertions | Component materialization invokes real Dataset validation before owner bootstrap | UAT-8A-02/04 |
| Failed fresh materialization destroys and reruns | Partial topology reused | induced failure, exact teardown, volume absence and complete rerun assertions | retained failed receipt followed by new from-empty healthy attempt | UAT-8A-07 |
| Old transition references and payloads unsupported | Hidden legacy compatibility | historical fixture integrity plus old owner/type/version/payload rejection | old inputs rejected; no adapter, ledger, reader or live old reference | UAT-8A-06 |
| Rerun Phase 7 scope/lifecycle/outage/compatibility | Logical behavior changes at physical boundary | contract, lifecycle, nondisclosure, scope, timeout, compatibility and outage suites | real Component/Dataset/Dashboard process-boundary smoke | UAT-8A-04/05 |
| Rerun source ownership, package graph, independent upgrade/rollback | Coupled or synthetic release | boundary audits; source-built release/binary identity; exact one-owner plan assertions | Supervisor/Compose `0.9.0` to `1.0.0` upgrade, rollback and restoration; unrelated digests/restarts unchanged | UAT-8A-06/08 |
| Move all Component product and operational ownership | Split ownership or missing surface | module product/API/UI/operations integration suites | same-origin module-owned routes and diagnostics | UAT-8A-01/03 |
| Real Release/Instance and fresh database | Wrong identity/provenance/storage | release/instance/catalog/schema/database-isolation tests | composition read-back and exact provenance | UAT-8A-02/06 |
| Dashboard has one canonical inventory/navigation owner | Duplicate transition and deployed-module presentation | exact five-transition identity assertion; Dashboard absent from Core destination resolver; one manifest contribution | inventory exposes Dashboard only as its live 3.0.0 Release/Instance and navigation exposes it exactly once | UAT-8A-02/06 |
| Exact Component/Dashboard acceptance seed | Missing or non-executable lifecycle coverage | exact seven-shell/eight-version/seven-placement identity assertions, including predecessor/successor and three action placements | every placement resolves through Components v3; exact action fixtures are live; zero old owners/types | UAT-8A-02/05 |
| Typed Core Dataset compatibility references/contracts | Direct Dataset relationship retained | typed composite identity, exact version, source/credential audits | Component authors/renders across Core contract only | UAT-8A-04 |
| Preserve Dashboard public Components behavior | Contract or consumer-policy regression | Sprint 7B regression suites plus exact structured dependency diagnostic | Defer/Upgrade/Replace/Remove, nondisclosure, five-placement Component outage and zero-finding recovery across real provider | UAT-8A-05 |
| Rebuild seed/test data through owners | Stale fixtures or cross-owner writes | semantic owner/read-back/idempotence and negative credential checks | recognizable full reference app from owning APIs | UAT-8A-02/06 |
| Outage, timeout, scope, capability and compatibility | Leak, corruption or cascading failure | authorization/nondisclosure matrix and failure-containment tests | stop/degrade Dataset or Component; unrelated services healthy; recovery converges | UAT-8A-04/05 |
| UI unchanged plus module configuration/diagnostics | Visual/interaction or configuration regression | Playwright at 1280/768/390, dark/light, keyboard, 200%, no-JS, hydration and console | direct/shell documents and exact configuration/diagnostics | UAT-8A-01/03 |
| Exit: fresh seed contains only new Component references | Wrong owner/type or incomplete seed | source-exact semantic read-back and old-input absence/rejection | all Dashboard placements resolve against selected Component instance | UAT-8A-02 |
| Exit: author/execute Components through Dataset contract | Product parity, authorization or isolation failure | every Component kind, scope, timeout, contract and credential tests | author and execute recognizable fixtures through gateway | UAT-8A-04 |
| Exit: Dashboard continues consumption and degrades coherently | Cascading outage or disclosure | Dashboard dependency/lifecycle/outage regression | stop/recover Component and Dataset provider paths | UAT-8A-05 |
| Canonical Component API break | Stale first-party consumer or hidden translation | exact v3 request/response golden tests and old-shape failures | UI, Dashboard, seed, smoke and UAT use v3 only | UAT-8A-01/05/06 |
| Exact configuration product decisions | Label/timeout drift or over-broad authority | schema v1, default/range, normalization, display and authority tests | navigation/Module Management label and 5-second effective timeout | UAT-8A-03/04 |

Every row requires automated and manual evidence. No `N/A` is currently
planned. If a proof becomes unsafe or inapplicable, amend this inventory before
freeze with a concrete rationale and equal-or-stronger evidence.

The corrected Playwright acceptance manifest declares 84 exact scenario
identities. Nine UI-correction identities cover the shared canvas, active
navigation/title behavior, the required deterministic Component baselines,
and cross-module parity. This is an inventory declaration, not a browser
execution result.

## Required Evidence Inventory

Readiness 38 is the one legacy recovery boundary for the failed-Readiness
finalizer. It was finalized once without rerunning its checks; no other attempt
may use that exception. R39 subsequently passed and R31 produced the complete
failed-attempt/harvest/batch/authorization chain. R32 is separately the sole
historical cleanup-authorization qualification described above. The current
correction advances schemas and enforcement prospectively; none of these
retained receipts is retrofitted.

| Artifact | Producer | Required before | Status |
|---|---|---|---|
| `attempts/readiness-38.json` and sidecar | Validation coordinator | R38 correction finalization | Failed; retained with 12 pass, 2 fail, 1 block |
| `attempts/readiness-38-harvest.json`, `attempts/readiness-38-defect-batch.json`, and `attempts/readiness-38-correction-authorization.json` | Validation coordinator | Readiness 39 | Passed historical finalization; authorization consumed exactly once by R39 |
| `attempts/readiness-39.json` and `validation-readiness-result.json` | Validation coordinator | Rehearsal 31 | Passed for R31 source/environment; now superseded by the tracked correction |
| `attempts/candidate-rehearsal-31-attempt.json`, `-harvest.json`, `-defect-batch.json`, and `-correction-authorization.json` | Validation coordinator | Readiness 40 | Complete failed chain; 19 pass, 3 fail, 10 block; one-use authorization consumed exactly once by R40 |
| `attempts/readiness-40.json` and `validation-readiness-result.json` | Validation coordinator | Rehearsal 32 | Passed all 15 checks for `b4f1581e` / `af0ab5cf` / `6967aa2c...`; superseded by the pre-start harness correction |
| `attempts/candidate-rehearsal-32-prestart-launch.log`, `attempts/candidate-rehearsal-32-prestart-failure.json`, and `attempts/candidate-rehearsal-32-prestart-defect-batch.json` | Validation coordinator | Corrected R41 | Historical pre-start `harness` defect; no attempt or lane assertions began; the identity was later admitted after R41 passed |
| `attempts/readiness-41-prestart-launch.log`, `attempts/readiness-41-prestart-diagnostic.log`, `attempts/readiness-41-prestart-failure.json`, and `attempts/readiness-41-prestart-defect-batch.json` | Validation coordinator | Corrected R41 | Historical two-defect pre-start batch; no reservation/check began; the identity was later admitted and passed after correction |
| `attempts/readiness-41-start.json`, `attempts/readiness-41.json`, and `validation-readiness-result.json` | Validation coordinator | Rehearsal 32 | Passed 15/15 for `d703e7a6` / `889ddc5f` / `53fbb1f7...`; immutable terminal/canonical alias SHA-256 `547940b0...`; superseded by the R32 correction |
| `attempts/candidate-rehearsal-32-start.json` and `attempts/candidate-rehearsal-32-attempt.json` | Validation coordinator | R32 harvest | Terminal failed; immutable start `c397c6f4...`, attempt `98cddc03...`; 19 pass, 2 fail, 11 block, 0 defer |
| `attempts/candidate-rehearsal-32-harvest.json` and `attempts/candidate-rehearsal-32-defect-batch.json` | Validation coordinator | R32 correction | Complete immutable harvest `89861d8a...` and one raw two-defect batch `4a5219fd...`; raw `product` classifications retained |
| `attempts/candidate-rehearsal-32-diagnostic-supplement.json` and `attempts/candidate-rehearsal-32-post-harvest-restoration.json` | Validation coordinator | R32 correction authorization effectiveness | Append-only diagnosis `8bcfc25c...` and proven canonical restoration `e082eefd...`; neither rewrites nor passes R32 |
| `attempts/candidate-rehearsal-32-correction-authorization.json` and `attempts/candidate-rehearsal-32-correction-authorization-qualification.json` | Validation coordinator | Readiness 42 | Original schema-2 authorization remains quarantined; effective only with qualification `859a8dba...` for exactly one R42 start |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | Not created; R32 failed and cleanup was not proven within its immutable attempt |
| `attempts/preflight-<n>.json` | Preflight | Preflight terminal decision | Not Run |
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `attempts/sit-<n>-start.json`, `attempts/sit-<n>.json`, and `sit/attempts/<lane>-<n>.json` | SIT | SIT terminal decision | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
| `attempts/uat-<n>.json` and `attempts/uat-<n>-manual-checkpoint.json` | UAT | Formal UAT finalization | Not Run |
| `docs/sprints/sprint-8a-uat/scenario-contract.json` | Sprint plan / acceptance contract | Preflight and manual-UAT start | Tracked canonical eight-scenario contract |
| `uat/attempt-<n>/manual-leases/uat-8a-<nn>-start.json`, optional `-resume.json`, `-publication-prepared.json`, and `-complete.json` | UAT manual publisher | Each manual receipt / Finalize | Not Run |
| `uat/attempt-<n>/finalizations/run-<nnn>/finalization-completion-checkpoint.json` | UAT Finalize | Canonical UAT publication or finalization-only retry | Not Run |
| `uat/attempt-<n>/finalizations/run-<nnn>/result-commit.json` | UAT Finalize | Canonical UAT publication | Not Run |
| `uat/attempt-<n>/finalizations/run-<nnn>/publication-retry-checkpoint.json` and `uat/attempt-<n>/publication-retries/run-<nnn>/` | UAT Finalize | Publication-only retry, when triggered | Planned / Conditional |
| `uat-result.json` | UAT | Authorization | Not Run |
| `uat-defect-harvest.json` | UAT/coordinator | Correction batch, when triggered | Planned / Conditional |
| `defect-batch.json` | Coordinator | Impact assessment, when triggered | Planned / Conditional |
| `correction-impact-assessment.json` | Coordinator | Focused repair validation, when triggered | Planned / Conditional |
| `focused-repair-validation/attempt-<n>.json` | SIT/UAT/coordinator | Convergence, when triggered | Planned / Conditional |
| `canonical-restoration.json` | Coordinator | Convergence/final certification, when triggered | Planned / Conditional |
| `final-certification-entry.json` | Coordinator | Final readiness/rehearsal, when triggered | Planned / Conditional |
| `evidence-manifest.json` and `.sha256` sidecar | Validation phases | Authorization | Not Run |
| `closeout-authorization.json` | Coordinator | Closeout | Not Run |

Additional required evidence namespaces:

- `readiness/`: executable checklist, destructive-target authorization,
  tool/shell/runner self-tests, semantic fixture audit, acceptance mapping and
  clean-source proof.
- `rehearsal/`: static, Rust, Playwright, source-exact fresh deployment,
  contracts/nondisclosure, smoke, teardown/rerun and automated-UAT diagnostics.
- `sit/`: lane receipts, raw logs, deployment/provenance, empty baseline,
  first/no-op bootstrap, old-input rejection, outage/recovery,
  upgrade/rollback and final convergence.
- `uat/attempt-<n>/`: scripted inventory/smoke logs, final source-exact
  materialization/no-op, canonical inventory/smoke restoration, and final
  health evidence.
- `uat/attempt-<n>/manual/`: UAT-8A-01 through UAT-8A-08 receipts,
  screenshots/traces and reviewer sign-off, all bound to the exact formal-UAT
  attempt.
- `uat/attempt-<n>/manual-leases/`: one authenticated start/completion pair and
  immutable prepared-publication checkpoint per scenario, plus an authenticated
  process-lineage resume marker only when interruption requires it. The
  evidence-root lock is held from lease start through transactional receipt and
  completion publication; an open or interrupted lease blocks Finalize.
- `uat/attempt-<n>/finalizations/` and `publication-retries/`: append-only
  canonical-restoration runs, durable completion checkpoints, result commits,
  retained publication failures, and any authorized publication-only retry.
- `materialization/`: resolved destructive targets, empty baseline,
  owner-bootstrap order/read-back, first/no-op results, induced failed attempt,
  teardown proof and successor from-empty result.
- `upgrade-rollback/`: baseline/candidate/rollback identities and unrelated
  service digest/restart chronology.

## Candidate Identity

- Implementation commit: To be frozen after passing readiness and rehearsal.
- Tree: To be frozen.
- Dirty state: Must be clean at freeze.
- Candidate fingerprint: Not issued.
- Acceptance-inventory identity: SHA-256 of this record plus tracked acceptance
  runners, tests, fixtures, seeds, manifests, bootstrap and deployment inputs.
- Deployment identity: one digest covers the tracked Sprint 8A deployment
  profile and manifests; a separate
  `normalized_deployment_configuration_sha256` is the SHA-256 of the exact live
  normalized `docker compose ... config` output retained by the environment
  contract. Candidate, preflight, SIT, and UAT receipts must bind that same
  normalized digest rather than re-hashing YAML as a substitute.
- Schema/baseline identity: empty Core, Component, Dashboard, Supervisor and
  selected-module schema migration digests plus bootstrap/seed contract
  versions. No legacy populated-data baseline exists.
- Expected provenance labels: OCI revision, source tree, dirty state, build
  profile, module definition/release, manifest, asset, schema migration,
  contract, fixture, seed and acceptance-inventory digests.
- Observed image digests: Not Run.

## Validation Readiness

- Latest completed check graph: Readiness 41 passed all 15 checks against clean
  source `d703e7a6`, tree `889ddc5f`, and environment `53fbb1f7...`; immutable
  terminal and canonical alias SHA-256 `547940b0...`. R32 then failed, and the
  current product/harness/evidence correction changes source, validation
  interpretation, fixtures, acceptance, and deployment/materialization inputs.
  It therefore supersedes R41. The qualified R32 correction transition permits
  exactly Readiness 42 after this correction is committed and all six test
  databases are recreated empty.

- Derived executable checklist: regenerate from the final plan, this inventory,
  source, runners, Compose profile and evidence schemas.
- Environment variables and reset acknowledgements: enumerate exact Sprint 8A
  installation ID, gateway/Supervisor ports, database URLs/role identities,
  signing key IDs, service endpoints, Compose project, every disposable
  database/container/volume target, explicit destructive authorization and
  evidence root without recording secrets.
- Supported tools, shells and runtimes: PowerShell runner parsing/invocation,
  Rust/Cargo, wasm tooling, Node/npm/Playwright, Docker/Compose, PostgreSQL
  client, browser and SHA-256 helpers.
- Ports, topology, health and provenance: one gateway, Core, Supervisor,
  Component, Dashboard, Dataset compatibility provider and required reference
  services; isolated stores/roles; intended handoff slot; exact image labels.
- Semantic fixture and idempotence audit: administrator, global Module
  Management reader/manager, Component manager, scoped Dashboard manager,
  constrained reader, out-of-scope actor, service identities, all Component
  kinds, lifecycle states, Dataset major lines, allowed/blocked references,
  recognizable output, v3 and historical negative inputs, timeouts/outages and
  unchanged second-run behavior.
- Reset safety: resolve absolute Compose project and every exact disposable
  volume/database/container target; reject missing/ambiguous/broad targets and
  prove deletion cannot escape the declared environment.
- Runner/output/receipt/hash/finalization self-tests: safe disposable probes for
  every argument/output path, atomic completion, SHA-256, failure/supersession,
  heartbeat, teardown and completion sentinel.
- Attempt lifecycle: acquire the evidence-root OS-exclusive lock and
  authenticate current state, pending authorization, attempt number, and the
  empty five-target namespace before publishing an `unverified` live receipt
  plus immutable hashed start snapshot; probe source/environment only after
  publication; and publish a sidecar-verified attempt/state checkpoint after
  every terminal pass, failure, or block.
- Correction transition: after one complete failed-rehearsal harvest and batch,
  permit exactly one successor Readiness start through an append-only
  consumption receipt bound to the predecessor and successor start snapshot.
  Reject duplicate consumption, active attempts, used attempt identities, and
  any Readiness receipt other than the exact path/hash currently named by
  validation state.
- R32 correction transition: R32 is the sole historical cleanup-authorization
  exception. Its schema-2 authorization is effective only with qualification
  SHA-256 `859a8dba...`, which authenticates diagnostic supplement
  `8bcfc25c...` and restoration `e082eefd...` and permits exactly R42. The tuple
  remains unconsumed; it authorizes no other attempt or phase.
- Rehearsal schedule: R42 must authenticate the full R32 lineage and current
  changed-path/identity cone, then bind R33's complete declaration graph and
  deterministic two-wave schedule. Because the correction changes shared
  health, materialization, smoke, cleanup/authorization, evidence paths,
  scheduler/runner declarations, acceptance fixtures, and Dashboard seed
  behavior, every intersecting lane belongs in Wave A. Any unauthenticated
  history or counter selects conservative full harvest.
- Acceptance mapping: every inventory row maps to SIT, smoke and UAT evidence.
- Clean repository and source-exact inputs: required before rehearsal.
- Result receipt: `artifacts/sprint-8a-closeout/validation-readiness-result.json`.

## Candidate Rehearsal

Attempt 32 is terminal and may not resume. It retained 19 passes, 2 raw
`product` failures, 11 exact lane blocks, 0 deferrals, and 21 assertion-bearing
lanes against passing R41's exact source/environment pair. Its immutable start,
attempt, harvest, raw batch, append-only diagnosis, restoration, and qualified
authorization are named above. No passing result exists. The current correction
is candidate-affecting; R33 cannot start until it is committed, R42 passes, and
the validation coordinator authorizes the formal rehearsal.

Sprint 8A rehearsal uses the dependency-aware inventory below. A create-once
start receipt declares the graph, exact Wave A/Wave B/aggregate-sink/terminal-
cleanup-sink/safety-finalizer order, prior evidence hashes, impact decisions, and deferral
counters before
assertions. After any failure the attempt is `harvesting`: safe independent
Wave A siblings continue, true dependents are blocked, and eligible Wave B
lanes become `deferred` only after Wave A harvesting finishes. Aggregate sinks
follow, then mandatory teardown, terminal restoration, and safety finalizers
continue. No tracked correction or new attempt may begin
until one harvest receipt and one consolidated defect batch account for every
row as passed, failed, blocked, or deferred.
`scripts/test-sprint-validation-harvest.ps1` enforces terminal accounting and
the single-batch contract; materialization retains failed apply responses and
service logs before exact teardown.

Before the start receipt exists, the launcher acquires the evidence-root
OS-exclusive reservation; contention rejects launch without attempt evidence.
Under that retained handle, it captures current state and Readiness scheduling
inputs and either authenticates bounded history or selects conservative full
harvest. The start receipt initially carries unverified source/environment
identities. The Wave A attempt-state lane authenticates the retained handle,
requested attempt, current state, and lifecycle transition, while the
independent Readiness lane authenticates the supplied passing receipt,
source/environment identity, and one-use predecessor-correction consumption.
Safe static lanes remain independent of those fallible prerequisites; only true
dependents or unsafe destructive work are blocked. Process-loss recovery
authenticates that immutable start plus the latest checkpoint, preserves the
order and counters, and resumes only safe work in the same attempt.

Aggregate fan-in lanes are sinks and cannot expand Wave A merely because they
depend on many lanes. Execute them only after their current-attempt
prerequisites are eligible and pass. Cleanup sinks, failure recovery, canonical
restoration, and final source/environment identity checks are mandatory. Any
deferred lane keeps the attempt failed/incomplete and prohibits a canonical
rehearsal result or preflight.

The automated-UAT lane projects each failed semantic predicate as its exact
`UAT-8A-xx/assertion-id` identity with classification, reason, and hashed raw
evidence. If only nested semantics fail, the outer projection lane remains
passed so the consolidated batch contains one defect per failed predicate and
no generic outer duplicate. Wrapper or parsing failures remain separate harness
defects, and blocked scenarios retain their exact dependency reasons.

Every next-attempt lane records whether assertions actually started and the
aggregate counts only those executed lanes. Structured child classification is
canonical when present and carries its source into the outer receipt and
harvest. Final environment comparison retains expected and actual fingerprints
plus changed contract sections, and every newly persisted evidence reference is
canonical, repository-relative, and contained by the evidence root.

An eligible deferred lane records no execution timestamps, duration, assertion
start, or current evidence. It retains the authenticated prior-passing receipt
path/hash and source/environment identities, current impact decision and non-
impact rationale, prerequisite state, consecutive count,
`mandatory_by_attempt`, and the exact diagnostic-history-only notice. A lane
executes on the fourth attempt after three consecutive deferrals, and any
execution resets its counter. Missing or unauthenticated history selects full
Wave A rather than a speculative deferral.

The following table is the planned lane summary for the next successor
Candidate Rehearsal, not a restatement of R32's terminal results.

| Diagnostic lane | Planned command/evidence | Assertions | Successor result | Defect batch |
|---|---|---|---|---|
| Static and boundaries | fmt/check/Clippy, manifests, links, native/WASM graphs, source/image audits | zero warnings; no forbidden owner/dependency/route/storage/legacy edge; exact-body and joint-scope enforcement present | Not Run | |
| Full Rust | `cargo test --workspace --locked` plus targeted contract/schema/authorization tests | all pass, including render kind/identity, raw-body tamper, exact assertion and same-node matrix | Not Run | |
| Source-exact materialization | authorized reset, empty schemas, first/no-op owner bootstrap | exact provenance; Core 200/text/plain/`ok` and Supervisor 204/empty redirect-disabled health; canonical Dashboard V1 layout; semantic seed and exact no-op; manifest-declared target and canonical opaque payload; request-bound one-use authorization; provider-owned result; no product-specific Core/Supervisor branch; zero writes on mismatch, expiry, replay, incompatibility, or outage | Not Run | |
| Playwright | `scripts/validate-e2e.ps1` with exact gateway, deployment receipt, fresh-state, Sprint 8A profile, and evidence bindings | complete inventory; zero unexpected skip/retry/flake; retained outputs | Not Run | |
| Conformance and nondisclosure | module testkit plus Components/Dataset/Dashboard matrix | owner/version/scope/audience/known-random/timing/lifecycle cases pass; shared-node render succeeds and disjoint metadata/render stay restricted | Not Run | |
| Deployed smoke | general and Sprint 8A smoke in rehearsal namespace | real boundaries, fixtures, old-input rejection, outage/recovery and final health | Not Run | |
| Live product diagnostics | attempt-bound product receipt plus raw structured Dashboard dependency JSON/SHA sidecar | exact predecessor/successor placements; Defer/Upgrade/Replace/Remove; blocked nondisclosure; five-placement Component outage; zero-finding recovery; partial evidence retained on failure | Not Run | |
| Component release transition | source-built `0.9.0` metadata plus Supervisor/Compose apply receipts and stage snapshots | exact one-owner `0.9.0`/`1.0.0` upgrade, rollback and restoration; Component preservation; unrelated identity stability | Not Run | |
| Failure teardown/rerun | induced partial materialization failure | evidence retained; exact topology/volumes removed; new empty rerun healthy | Not Run | |
| Automated UAT diagnostics | automated equivalents of UAT-01 through UAT-08, including the structured live-product receipt | every precondition and expected semantic state reproducible; UAT-8A-04 includes `joint-dashboard-component-scope`; failed nested predicates retain exact identity/classification/reason/raw hash without outer double count; UAT-8A-05 cannot pass from lane labels alone | Not Run | |
| Terminal canonical restoration | independent `final-successor-health` cleanup sink after Wave B and aggregate disposition, then final environment identity | source-exact canonical topology; exact Core/Supervisor health and inventory/navigation; cleanup result `canonical_successor_healthy`; executes despite diagnostic failure/deferral | Not Run | |

The first two lanes are independent of a deployed topology. Playwright locked
installation/discovery, runner self-tests, and acceptance-inventory checks are
also independent. Healthy materialization/no-op is the prerequisite for live
Playwright execution, deployed smoke, deployed inventory/navigation audit,
automated UAT diagnostics, and the Component upgrade/rollback baseline.
Failure-containment recovery owns an isolated diagnostic topology; terminal
canonical restoration is independently mandatory after Wave B/aggregate
disposition. Formal deployed acceptance smoke remains SIT-owned and is not a
rehearsal substitute.

The Sprint 8A reference materialization rebuilds the canonical seed through
signed owner bootstrap and receipt read-back before deployment evidence is
captured: exactly seven Component shells, eight versions, and seven Dashboard
placements, including the predecessor/successor and independent action
fixtures. The legacy helper prepares only account/security state. This is a
pre-production rebuild, not a compatibility migration. The complete Playwright
inventory must never rely on one earlier test file to create shared fixtures.
Exact deployed
inventory/navigation evidence is produced by
`scripts/audit-sprint-8a-deployed-inventory.ps1`; ad hoc route guesses, envelope
parsers, and copied counts are not rehearsal evidence.

Logical independence does not authorize concurrent Cargo commands to share the
repository `target` directory. `scripts/validate.ps1` cleans selected build
artifacts, so complete Rust siblings run serially unless each has a distinct
explicit `CARGO_TARGET_DIR`. Attempt 12 retained the missing-executable failure
that demonstrated this enforcement gap.

After removing the old Core-owned Component and Dashboard transition entries,
the fresh default Main placements after Datasets are Scoped Records at 7,
Components at 8, and Dashboard at 9. The Sprint 8A blueprint and acceptance
contract pin those exact identities and orders; Dashboard remains a single
manifest-owned contribution.

Readiness must run `npm ci --prefix end2end` and complete Playwright discovery,
not merely read the declared package version. It must also parse and self-test
the harvest guard and verify that `scripts/uat-sprint-8a.ps1` plus all eight
`docs/sprints/sprint-8a-uat/uat-8a-*.md` scripts exist.

Complete validation requires six pairwise-distinct, freshly created disposable
database bindings: API, fresh-baseline API, reference module, extracted
Component module, API enrollment, and Installation Control. A database
identity used by one owner integration target is not reused by another target
in the same rehearsal.

- Mutable source/environment identity: Readiness 41 and Rehearsal 32 verified
  clean commit `d703e7a67c3f1177b621b0e14161e0661209ea42`, tree
  `889ddc5f784ea1e81686ce63dd22669aabde956d`, and environment fingerprint
  `53fbb1f74375f96abb292ba610fc030686fca55864a70ca8149932f1a4820e28`.
  The current candidate-affecting correction supersedes that source.
- Passing readiness prerequisite: Readiness 41 passed with SHA-256
  `547940b0ca6685d98304a1f51d9810e38bd54463f6fd378dd375d46c7407c2aa`,
  but R32 failed. Its qualified correction transition permits exactly R42; R41
  cannot be reused by a later rehearsal.
- Current consolidated correction: preserve R32's two raw `product` findings
  while correcting the shared endpoint-specific health harness, independent
  canonical-restoration sink, restoration-before-authorization guard,
  canonical full-graph evidence paths, stale public `placement_key` assertion,
  and Dashboard's unversioned fallback-layout seed. The bounded scheduler,
  deferred schema/counters, validation state, harvest/batch lineage, and
  preflight rejection receive adversarial enforcement in the same cone.
- Preceding pre-start correction: the rejected R32 launch retained one
  `harness` defect before an attempt existed. Commit `231bd8d9` made
  declaration-member validation representation-aware and added adversarial
  ordered-dictionary fixtures matching the live runner. The subsequent R41
  pre-start rejection superseded that source but did not invalidate its retained
  diagnostic history.
- Preceding consolidated correction: Attempt 31 retained 19 passes, 3 raw
  failures, 10 dependency blocks, and 8 nested UAT blocks. Its correction
  advanced fresh-database readiness, Supervisor target enrollment, normalized
  Compose parsing, containment evidence, the two-wave scheduler, receipt/state/
  harvest schemas, and their adversarial enforcement together; R40 verified
  that source before the later pre-start defect superseded it.
- Historical consolidated defects and correction batches: Attempt 15 retained six
  Playwright findings in one fail-late batch after 10 independent checks
  passed. Four acceptance-inventory defects used the former Core Component
  `id`, the removed searchable Dataset Version picker, or stale generic Dataset
  dependency copy. One Component product defect disclosed a known unreadable
  identity as `403` instead of making it indistinguishable from a random `404`.
  One Dashboard product-asset defect served release `3.0.0` with entry imports
  pinned to `2.1.0`, preventing module hydration. The remaining 34 serial-suite
  cases were retained as blocked (2 Components, 8 Dashboards, 24 Permissions),
  and the all-prerequisite final check was blocked specifically by Playwright.
  Containment and all eight non-authoritative UAT diagnostic mappings still
  passed. The correction updates canonical identities/interactions, closes
  Component nondisclosure, binds Dashboard assets and digests to release
  `3.0.0`, and requires complete readiness and rehearsal restart. Two narrow
  dirty-source materialization invocations reached only the release build and
  were stopped by the external 15-minute command timeout; they are retained as
  diagnostic build evidence and do not replace either full gate. Attempt 14's
  two-finding, Attempt 13's ten-finding, and Attempt 12's eight-finding batches
  remain retained and superseded.
- Attempt 16 retained one deployment-input defect through three independent
  observations: Supervisor apply failed closed and both complete Rust suites
  reported the same Dashboard manifest/catalog mismatch. Tessara's manifest
  identity is the SHA-256 of RFC 8785/JCS canonical JSON produced by
  `tessara_composition::canonical_digest`, not a hash of platform-dependent
  working-tree bytes. The catalog now uses the canonical `cec6af45...` digest;
  the existing `checked_catalog_manifest_digests_match_runtime_manifests` test
  is the governing derived assertion and deliberately remains the single
  enforcement point.
- Attempt 17 retained two operator-harness defects as one batch after seven
  independent checks passed. Manual orchestration omitted the exact Core,
  gateway, and database container identities required by deployment-evidence
  capture, then invoked general smoke without its evidence/data-state bindings;
  the independent Sprint 8A semantic smoke still passed all 26 checks. The
  manual Component exercise also supplied the invented `-OutputImage`
  parameter instead of the baseline builder's declared `-OutputTag`. Playwright
  execution was blocked on deployment evidence; UAT diagnostics and final
  health were blocked on their declared failed prerequisites. Repository-owned
  wrappers now resolve these identities and arguments, and readiness parses and
  self-tests both wrappers so the commands cannot drift back to ad hoc manual
  assembly.
- Attempt 18 retained one operator-harness defect after four independent lanes
  passed: a 60-second client observation timeout terminated the active
  materialization build. No tracked input changed; the corrected execution used
  a non-terminating command boundary with short observation waits, and the
  complete readiness/rehearsal cycle restarted.
- Attempt 19 retained four first-failure findings after eight checks passed: the new general
  smoke wrapper incorrectly requested the forbidden Sprint 6A acceptance
  schema; one Component test still expected the removed projection-builder UI;
  Dashboard document deserialization expected `component_reference` where the
  canonical Components V3 metadata envelope exposes `reference`; and the
  permission suite inferred scoped Component fixtures from seed coincidence.
  Narrow correction verification then exposed one underlying product/seed
  finding in that same consolidated batch: demo seeding used a duplicate
  Dataset-major materializer that omitted the canonical restriction-tier and
  semantic-version columns, making freshly rebuilt seeded Components fail at
  execution. The batch therefore contains five defects, not a restarted
  rehearsal or a second micro-batch.
  The consolidated correction removes authoritative evidence publication from
  rehearsal smoke, tests the simplicity-first Configuration JSON form, aligns
  Dashboard document types with the V3 contract, and creates exact named scoped
  Component fixtures, and makes demo seed major lines conform to the canonical
  Dataset materialization schema. Playwright retained 33 passes, three first failures, and
  34 serial dependents; UAT diagnostics and final health were dependency-blocked.
- Attempt 20 retained three findings after nine checks passed. Playwright
  completed 48 tests, retained two first failures, and marked 20 serial
  dependents blocked: the Dashboard placement renderer still decoded the
  removed `dataset_id`/`dataset_version_major` response fields instead of the
  canonical Components V3 `dataset_reference`, and scoped Component execution
  disclosed a known hidden identity as `403` instead of the required
  nondisclosing `404`. The independently passing upgrade/rollback wrapper then
  exposed a harness handoff defect: a caller-selected evidence path was not
  accompanied by the canonical receipt consumed by UAT diagnostics. Live UAT
  smoke still passed, but scenario mapping failed on that exact missing
  prerequisite; the aggregate final check was blocked by Playwright and UAT.
  The consolidated correction aligns Dashboard render DTOs, makes scoped
  execution nondisclosing, and makes the upgrade wrapper atomically retain the
  canonical receipt plus any requested evidence copy.
- Attempt 21 retained two findings after ten checks passed. Playwright
  completed 48 tests, retained two first failures, and marked 20 serial
  dependents blocked. The deployed Dashboard image embedded the tracked
  `dashboard.wasm` generated before the Components V3 response change, so its
  decoder still required removed `dataset_id`/`dataset_version_major` fields
  even though the current Rust DTO and provider response used the canonical
  typed `dataset_reference`. The permission inventory also assumed Core's
  `#app-root` for independently deployed Component documents whose exact owned
  root is `#module-content`; the sibling JavaScript-disabled Component routes
  carried the same copied assumption. All other rehearsal lanes, including
  source-exact materialization, inventory/navigation, deployed smoke,
  upgrade/rollback, all eight UAT diagnostics, complete Rust validation,
  all-feature tests, Clippy, and containment health passed. The aggregate final
  check was blocked only by Playwright. The consolidated correction rebuilds
  and re-pins Dashboard bindings/WASM, tests the actual embedded asset hashes
  and V3 wire identity during readiness, and binds every Component route
  assertion to its exact module document root.
- Attempt 22 retained two findings after ten checks passed. Playwright completed
  52 tests, retained two first failures, and marked 16 serial dependents
  blocked. The Dashboard paging scenario selected the first available Table by
  type and then assumed every identity other than the Sprint 7A table had more
  than ten rows; the selected Sprint 8A reference table correctly had four.
  Separately, the independently deployed Component complete-document script
  installed its behavior but did not publish the shared
  `data-hydration="ready"` lifecycle marker. Source-exact materialization/no-op,
  inventory/navigation, deployed smoke/evidence, Sprint 8A semantic smoke,
  Component upgrade/rollback, all eight UAT diagnostics, complete Rust
  validation, all-feature tests, warning-free Clippy, exact Playwright
  discovery, and containment health passed. The aggregate final check was
  blocked only by Playwright. The consolidated correction binds paging to the
  exact multi-page demo Session Log Table and makes Component complete
  documents publish and test the shared hydration marker.
- Attempt 23 retained one harness finding after six independent checks passed
  and six deployment-dependent checks were blocked with exact reasons. The
  source-exact materialization command failed before destructive work when the
  documented interactive `-Confirm` prompt was invoked through the rehearsal
  evidence pipeline. Complete Rust validation, all-feature tests, warning-free
  Clippy, then-current exact Playwright inventory discovery, clean source, and current
  environment containment health passed. Inventory/navigation, deployed
  smoke, Playwright execution, Component upgrade/rollback, UAT diagnostics,
  and the aggregate final check were correctly blocked because no corrected
  source-exact topology existed. A captured-output `WhatIf` reproducer passed
  with `-Confirm:$false`; the correction makes that automation-safe invocation
  canonical and readiness-enforced without weakening the explicit disposable
  reset authorization or exact-target guards.
- Validation Readiness attempt 27 completed all 13 sibling checks and retained
  one harness failure in the new reset dry-run enforcement. The guarded
  `WhatIf -Confirm:$false` invocation returned successfully, but PowerShell
  rendered the `WhatIf` message directly through the host rather than into the
  assigned pipeline, and readiness incorrectly required that host-only text in
  the capture. The corrected check exercises the same capture boundary and
  requires successful return without treating host rendering as pipeline
  evidence. Candidate Rehearsal attempt 24 did not begin.
- Attempt 24 retained one shared harness/environment-contract finding after 11
  independent checks passed. Bare `npm test` used the Playwright development
  default at `127.0.0.1:8080`, so 30 tests failed with `ECONNREFUSED` and 40
  serial dependents did not run even though the source-exact Sprint 8A gateway
  was healthy at `127.0.0.1:8088`. Materialization/no-op,
  inventory/navigation, deployed smoke, Component upgrade/rollback, all eight
  UAT diagnostics, complete Rust validation, all-feature tests, warning-free
  Clippy, discovery, and containment health passed; only the aggregate final
  check was blocked. The correction makes the existing repository-owned
  `validate-e2e.ps1` runner canonical for rehearsal and makes readiness parse,
  self-test, and require its exact gateway/deployment/fresh-state/profile/
  evidence bindings.
- Attempt 25 retained two findings after 11 safe independent/dependency-valid
  checks passed. Playwright completed 52 tests, retained two first failures,
  and marked 16 serial dependents not run. The Dashboard paging scenario still
  named the removed Core demo Table instead of the canonical module-owned
  `sprint-8a-record-table`; the canonical four-tier Dataset also had only four
  rows and therefore could not prove paging. Narrow correction diagnosis kept
  the same batch open and exposed the underlying product contract gap: the
  Component provider discarded Dashboard cursors while Core rejected Dataset
  execution cursors and always returned no successor. The Component detail
  document separately allowed its version table to impose a 558-pixel
  min-content width in a 390-pixel viewport. The consolidated correction binds
  paging to the exact module identity, extends the canonical tier fixture with
  26 deterministic public rows, preserves/validates bounded forward cursors,
  performs deterministic limit-plus-one Dataset paging, and constrains the
  module page/panel grid items so the table owns its narrow overflow. The
  aggregate final check was blocked exactly because Playwright did not pass;
  all other lanes, including materialization/no-op, inventory/navigation,
  smoke, upgrade/rollback, all eight UAT diagnostics, complete Rust checks,
  Clippy, discovery, and containment health, passed.
- The restart-behavior audit found the protocol and readiness references now
  state the fail-late rule completely. The earlier enforcement gap was
  executable orchestration: ad hoc commands could bypass the dependency graph,
  harvest terminal-state guard, or source-exact Playwright wrapper. The
  repository-owned harvest guard, its readiness self-test, the declared attempt
  graph, and the readiness-enforced `validate-e2e.ps1` binding close that gap;
  no duplicate skill prose is added. Attempt 25 exercised this enforcement by
  retaining one harvest and one consolidated batch before tracked correction.
- Attempt 26 retained six findings after nine checks passed. Playwright
  completed 51 tests, retained three failures, and marked 16 serial dependents
  not run: the four named tier identities were incorrectly required on the
  first deterministic 25-row page; Core's generic module gateway dropped the
  browser query before the Dashboard module, so the embedded 10-row request
  became the Component provider's 25-row default; and the scoped Component
  route still expected two Core-era admin fetches even though module-owned SSR
  now hydrates without them. Materialization/no-op, exact inventory/navigation,
  deployed smoke, complete Rust validation, workspace all-features, warning-free
  Clippy, exact discovery, and containment health passed. The Component upgrade
  sibling failed before assertions because the operator supplied undeclared
  `-Overwrite` and was not retried. UAT live smoke passed, but its UAT-8A-08
  mapping incorrectly accepted a stale prior-source upgrade receipt. The first
  harvest-finalization call similarly used undeclared `-BatchPath`; immutable
  results were complete, so only finalization was rerun with the declared
  `-DefectBatchPath`. Those two operator findings require no tracked command
  change because the scripts already expose the exact parameters and the
  canonical command omits `-Overwrite`.
- The consolidated correction preserves exact query strings through the
  manifest-driven module gateway and tests that boundary, explicitly requests
  all 30 semantic rows for the tier identity assertion, removes stale admin-GET
  expectations from module-owned Component documents, stamps upgrade evidence
  with the clean source identity, and makes UAT diagnostics reject an upgrade
  receipt from another source or from before the current materialization. The
  UAT self-test exercises the stale-receipt rejection. Readiness 30 and
  rehearsal 26 are superseded by this tracked correction.
- Readiness 31 is superseded. Readiness 32 is invalid because it did not prove
  authenticated database reachability or issue a structurally shared
  environment contract. Rehearsals 27 and 28 were explicitly abandoned when
  the user exited testing; neither reached a complete terminal graph or may be
  resumed. Attempt 28's three partial harness/evidence findings and raw logs
  are retained under the invalidation receipt named above. The subsequent
  implementation audit and consolidated correction replace those attempts;
  they do not convert their partial evidence into a rehearsal result.
- Readiness 36 subsequently passed its complete mutable gate, and Rehearsal 29
  began against that exact clean source/environment pair. Rehearsal 29 was
  stopped during harvesting under that user-directed return to
  implementation; its 8 passes, 3 failed receipts, 1 interrupted lane, 20
  unexecuted checks, and 0 recorded blocks remain diagnostic evidence only.
  The attempt may not resume, and the then-current candidate-affecting
  correction superseded Readiness 36.
- Readiness 37 then passed against `84964c7b`/`4f39cb71` and environment
  `96c2a32e...`. Rehearsal 30 completed its terminal graph with 18 passes, 4
  raw failures, and 10 blocks, retained one harvest/batch/authorization chain,
  and returned to implementation for the three-root harness correction. It may
  not resume, and the correction supersedes Readiness 37.
- Readiness 39 passed against `ffe05ace`/`1d17533d` and environment
  `96c2a32e...`. Rehearsal 31 then terminalized all 32 lanes with 19 passes, 3
  failures, and 10 blocks, retained one harvest/batch/authorization chain, and
  returned to implementation for its scheduler/materialization correction. It
  may not resume.
- Readiness 40 passed against `b4f1581e`/`af0ab5cf` and environment
  `6967aa2c...`. The requested R32 launch was rejected before an immutable start
  or attempt receipt because the scheduler validator did not recognize the
  runner's ordered-dictionary declarations. The retained pre-start batch
  required corrected R41 before that attempt identity was later admitted.
- The first corrected R41 launch against `231bd8d9`/`359789b1` rejected before
  reservation because correction-lineage validation required every successor
  terminal to be schema 2 and clean rerun admission was unreachable for an
  already-consumed lineage. Two `harness` defects and all reachable pre-start
  diagnostics are retained. The subsequent corrected R41 passed against
  `d703e7a6`/`889ddc5f` and environment `53fbb1f7...`.
- Rehearsal 32 then terminalized all 32 lanes with 19 passes, 2 raw `product`
  failures, 11 blocks, and 0 deferrals. Its immutable harvest and raw batch,
  append-only diagnosis, canonical restoration, and R42-only authorization
  qualification are retained. It may not resume.
- Complete-cycle repetitions after the current correction: 0.
- Result receipt: `artifacts/sprint-8a-closeout/candidate-rehearsal-result.json`.

Rehearsal is mutable, diagnostic and non-authoritative. Any correction requires
new complete readiness and a potentially passing rehearsal that executes all
required lanes before freeze; a failed rehearsal with deferred lanes remains
incomplete.

## Environment Contract

- Corrected successor environment fingerprint: Not issued. The latest
  superseded R41/R32 fingerprint is
  `53fbb1f74375f96abb292ba610fc030686fca55864a70ca8149932f1a4820e28`.
- Intended gateway: `http://127.0.0.1:8088`; intended Supervisor endpoint:
  `http://127.0.0.1:8098`.
- Exact health contracts: Core `GET /health` returns HTTP 200,
  `text/plain`, exact body `ok`; Supervisor `GET /health/ready` returns HTTP
  204 with an empty body. Every probe disables redirects and retains the exact
  endpoint, status, media/body observation, and contract result.
- Compose project/profile: `tessara-sprint-8a`,
  `deploy/sprint-8a/compose.yaml`, required profiles explicitly enabled.
- Validation database bindings: `TEST_API_DATABASE_URL`,
  `TEST_API_FRESH_DATABASE_URL`, `TEST_REFERENCE_MODULE_DATABASE_URL`,
  `TEST_COMPONENT_MODULE_DATABASE_URL`, `TEST_API_ENROLLMENT_DATABASE_URL`, and
  `TEST_INSTALLATION_CONTROL_DATABASE_URL`. Readiness requires pairwise-
  distinct loopback, token-bounded test database identities and completes an
  authenticated transactional temporary-table round trip against each one.
  Before R42, each database must be a freshly recreated generation with no
  user tables; a reachable but reused database is a failed readiness contract.
  All six bindings, the destructive-reset acknowledgement, and
  `TEST_POSTGRES_CLIENT_CONTAINER_ID` must be exported in the same PowerShell
  process that invokes the top-level Readiness runner; a prior shell process
  does not satisfy the contract.
- Materialized owner databases: empty disposable installation-scoped Core,
  Component, Dashboard, Supervisor, and selected-module databases with
  owner-specific runtime and schema-migration roles.
- Reset authorization: explicit approval bound to resolved absolute Sprint 8A
  project/container/volume/database identities only. Broad targets, unresolved
  variables and neighboring projects fail closed.
- Account/role fixtures: administrator, global Module Management reader/manager,
  Component manager, scoped Dashboard manager, constrained reader, out-of-scope
  actor and Core/Component/Dashboard service identities; no secrets in receipts.
- Evidence root/output mode: `artifacts/sprint-8a-closeout/`, repository-relative
  canonical paths, append-only attempts and atomic completion where supported.
- Identity binding: readiness writes a secret-free environment contract and
  SHA-256 fingerprint. Every rehearsal lane, raw artifact, UAT diagnostic, and
  aggregate result must match that fingerprint and the same source identity.

## Next Authorized Commands

No formal gate is authorized while the consolidated correction is mutable.
After review, focused verification, a clean correction commit, recreation of
all six empty databases, and validation-coordinator authorization, the mutable-
build coordinator may enter Readiness and Rehearsal only through the exact top-
level runners shown below. Those runners own the complete check inventory,
immutable schedule, dependency graph, evidence paths, and subordinate commands;
manually assembling lanes is not a Candidate Rehearsal and cannot authorize
preflight.

```powershell
.\scripts\validate-sprint-8a-readiness.ps1 -Attempt 42
.\scripts\run-sprint-8a-candidate-rehearsal.ps1 -Attempt 33 -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json"
```

R42 must consume the exact R32 schema-2 authorization together with its
append-only qualification, bind the immutable failed attempt/harvest/batch and
proven restoration, and leave the complete preceding correction lineage
unchanged. It then fixes R33's full declaration graph and schedule. If R33 Wave
A succeeds, the same attempt continues through every Wave B lane; if Wave A
fails, it finishes safe siblings and terminalizes eligible Wave B deferrals,
then runs aggregate sinks, mandatory restoration, and safety finalizers. Only
an all-passing, zero-deferred result can enter preflight.
Neither corrected-source gate has run, so candidate freeze, SIT, and formal UAT
remain closed.

Readiness requires and authenticates the six declared disposable database
URLs, proves their identities are pairwise distinct, records the normalized
Compose/tool/source contract, and issues the environment fingerprint consumed
by rehearsal. The complete runner serializes Cargo work that shares one target
directory and owns materialization/no-op, inventory/navigation, smoke,
Playwright, a non-acceptance live product diagnostic, Component
upgrade/rollback, deterministic failure containment, successor restoration,
automated scenario diagnostics, and final identity/health. The live diagnostic
runs before containment; containment destroys its disposable writes and proves
a fresh canonical successor. It cannot publish UAT evidence, and formal UAT is
not started by rehearsal.
Deployed acceptance smoke remains SIT-owned and manual scenarios remain
formal-UAT-owned.

## Preflight

- Status: Not Run.
- Receipt: `artifacts/sprint-8a-closeout/preflight-result.json`.
- Environment and reset authorization: must match passing readiness/rehearsal.
- Harness/inventory reconciliation: this record and every tracked runner, test,
  fixture, seed, manifest, bootstrap and deployment input match rehearsal.
- Bootstrap/no-op/teardown/rerun and post-rollback intended-release restoration
  commands: exact Sprint 8A commands frozen from passing rehearsal. Product
  data restoration or partial-materialization resume is not supported.
- Evidence paths and required-artifact audit: complete before freeze.
- Freeze output: `candidate.json` binds one immutable fingerprint; no validation
  result exists before it.

After the successor complete Readiness and Rehearsal pass, run only the tracked
top-level preflight command:

```powershell
.\scripts\run-sprint-8a-validation-preflight.ps1 -Attempt <n> -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json" -RehearsalReceipt "artifacts/sprint-8a-closeout/candidate-rehearsal-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -ExpectedBranch "codex/sprint-8a" -HandoffUrl "http://127.0.0.1:8088"
```

The runner, rather than operator-assembled helper calls, retains one terminal
result for each exact check identity below:

| Check identity | Depends on | Explicit retained check |
|---|---|---|
| `receipt-chain` | — | Parse and hash the exact passing Readiness and Rehearsal receipts; prove their source/environment and prerequisite binding. |
| `repository-scope` | — | Reconfirm repository instructions, worktree, branch, sprint scope, and one clean implementation commit. |
| `clean-source` | `receipt-chain`, `repository-scope` | Recompute commit, tree, dirty state, acceptance-inventory digest, and deployment-input digest and compare them exactly with Rehearsal. |
| `acceptance-traceability` | `receipt-chain`, `repository-scope` | Reconcile every roadmap exit and acceptance clause with the frozen 84-scenario browser inventory, smoke proof, and UAT-8A-01 through UAT-8A-08. |
| `environment-contract` | `receipt-chain`, `repository-scope` | Revalidate the secret-free fingerprint, tools, ports, Compose project/profile, handoff URL, reset authorization, and output-path forms. |
| `database-contract` | `environment-contract` | Revalidate all six named database variables, pairwise-distinct disposable identities, authenticated reachability, and migration-ledger tables. |
| `deployment-contract` | `clean-source`, `environment-contract` | Validate Compose/configuration, exact provenance label keys, bootstrap/no-op/teardown/recovery commands, active slot, and canonical restoration command without building product images. |
| `downstream-command-contract` | `repository-scope` | Parse the exact four SIT command sets and staged formal-UAT interface recorded below; reject a missing argument or obsolete runner contract. |
| `evidence-path-contract` | `repository-scope` | Reject unsupported absolute, traversal, and outside-root inputs; normalize contained repository-relative inputs; and prove no required path collides with immutable prior evidence. The only absolute-path exception is canonical consumption of the already-issued in-root R30 authorization. |
| `evidence-inventory` | `acceptance-traceability`, `deployment-contract`, `downstream-command-contract`, `evidence-path-contract` | Create the complete SIT/UAT/failure/manifest inventory before freeze and retain every mandatory output path. |

#### Acceptance-clause traceability

| Clause | Certification coverage |
| --- | --- |
| AC-01 | Module ownership, migration, manifest, route, asset, smoke, and UAT-8A-01 evidence. |
| AC-02 | Core boundary checks, workspace tests, module inventory, and UAT-8A-06 evidence. |
| AC-03 | Clean source-exact materialization and UAT-8A-02 first-apply evidence. |
| AC-04 | Exact semantic no-op and UAT-8A-02 evidence. |
| AC-05 | Dashboard/Component placement, bootstrap, and UAT-8A-02/UAT-8A-04 evidence. |
| AC-06 | Retired-input rejection, source-boundary checks, and UAT-8A-06 evidence. |
| AC-07 | Complete browser inventory, smoke, and UAT-8A-01 evidence. |
| AC-08 | Configuration-schema browser scenarios and UAT-8A-03 evidence. |
| AC-09 | Dataset reference contract tests, browser scenarios, and UAT-8A-04 evidence. |
| AC-10 | Authorization, nondisclosure, outage recovery, browser, and UAT-8A-04 evidence. |
| AC-11 | Dashboard lifecycle/dependency diagnostics and UAT-8A-05 evidence. |
| AC-12 | Compose credential boundaries, module boundaries, and UAT-8A-06 evidence. |
| AC-13 | Failure containment, exact teardown, successor health, and UAT-8A-07 evidence. |
| AC-14 | Component-only upgrade/rollback and UAT-8A-08 evidence. |
| AC-15 | Responsive, theme, keyboard, reduced-motion browser, and UAT-8A-01 evidence. |
| AC-16 | Exact five-transition Core inventory/navigation and UAT-8A-02 evidence. |
| AC-17 | Format, check, warnings-denied Clippy, workspace tests, smoke, SIT, and UAT evidence. |
| AC-18 | Components contract tests, exact rendering identities, browser, and UAT-8A-04/UAT-8A-05 evidence. |
| AC-19 | Exact v3 first-party inputs across seeds, smoke, browser, and UAT-8A-01/UAT-8A-04/UAT-8A-06 evidence. |

Before authenticating either prerequisite, write
`attempts/preflight-<n>.json` in `preparing` state with unverified claimed
source/environment identity, the ten declared check identities and their
dependencies, and `assertions_started = false`. Advance legally through
`executing`, optional failure `harvesting`, and `finalizing`; checkpoint every terminal check with exact
command, timestamps/duration, exit status, assertion-start marker,
classification, and hashed raw evidence. After a failure, continue every safe
independent check fail-late, mark true dependents `blocked` with the exact
failed prerequisite, and publish the terminal failed attempt without
`preflight-result.json` or `candidate.json`. Preserve and mark superseded
attempts; never reuse or merge their evidence.

After all ten results pass, the runner calls the lifecycle helper to publish
`preflight-result.json`, computes candidate identity from the exact source,
tracked deployment inputs, and live normalized Compose configuration digest,
and publishes `candidate.json`. The helper validates exact inventories, hashes,
prerequisite chain, source/environment identity, and candidate fingerprint; it
does not execute, checkpoint, or fabricate a missing check. The runner then
initializes the evidence manifest from all evidence that exists at freeze,
while its preflight inventory declares every future SIT/UAT and conditional-
failure path.

## SIT

SIT is authorized only by the passing preflight/candidate pair and enters
through the staged repository runner:

```powershell
.\scripts\run-sprint-8a-sit.ps1 -Stage Run -Attempt <n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/sit-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

Do not run the subordinate command sets independently. `Run` authenticates the
frozen identity, holds the evidence-root lock, executes all four lanes, retains
safe siblings fail-late, records exact dependency blocks, restores the
canonical topology, and publishes the aggregate only after a complete pass.

| Lane | Prepare receipt | Command/evidence | Assertions | Result | Duration |
|---|---|---|---|---|---|
| `static-and-boundaries` | `sit/attempts/static-and-boundaries-<n>.json` | exact static command set below | zero warnings and no forbidden edge | Not Run | |
| `rust-workspace` | `sit/attempts/rust-workspace-<n>.json` | exact database-bound Rust command set below | all pass on frozen candidate | Not Run | |
| `playwright` | `sit/attempts/playwright-<n>.json` | exact source/evidence-bound `validate-e2e.ps1` invocation below | complete inventory; UI/config/auth/outage/old-input cases pass | Not Run | |
| `deployed-acceptance-smoke` | `sit/attempts/deployed-acceptance-smoke-<n>.json` | exact materialization/smoke/recovery command set below | exact candidate; receipts complete; canonical topology restored | Not Run | |

Before authenticating preflight/candidate, write the phase start receipt at
`attempts/sit-<n>.json` in `preparing` state with the four declared lane
identities, unverified claimed candidate/environment identity, and
`assertions_started = false`. Each lane then owns a separate receipt and moves
through `preparing -> executing -> finalizing -> passed|failed|blocked`. Write
the lane start before expensive work, append logs and long-command heartbeats,
and retain every exact command, start/end/duration, exit status,
assertion-start marker, classification, raw evidence hash, and cleanup result.
Within each lane, continue safe independent checks fail-late; stop only true
dependents or unsafe destructive work and record the exact block reason.
Serialize the Playwright and deployed-smoke topology mutations even though each
owns its own source-exact preparation and evidence namespace.

After any failure, finish every safe lane, publish the terminal failed phase
attempt, and withhold `sit-result.json`; the coordinator owns classification
and invalidation. Preserve and mark superseded attempts rather than reusing a
number or merging evidence. Only after all four lanes pass and the canonical
topology is independently restored may the phase enter `finalizing`, publish
the aggregate SIT receipt through the helper, and immediately update and hash
the evidence manifest. A manifest/finalization failure remains retained and
blocks UAT even when the four raw lane results are intact.

The runner's static lane uses the following frozen command set and retains each
result:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config --quiet
. .\scripts\sprint-8a-acceptance-contract.ps1
Test-Sprint8AAcceptanceContract
.\scripts\check-web-crate-boundaries.ps1
.\scripts\verify-module-sdk-boundaries.ps1
.\scripts\verify-sprint-6e-boundaries.ps1
.\scripts\verify-markdown-links.ps1
```

After resetting the six exact preflight-approved database identities, the
runner executes the Rust lane serially in one Cargo target directory:

```powershell
cargo test --workspace --all-features --locked --offline --jobs 1
cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture
cargo test --locked --offline -p tessara-components-contract
cargo test --locked --offline -p tessara-dashboard-module
cargo test --locked --offline -p tessara-component-module
cargo test --locked --offline -p tessara-module-testkit
```

The runner invokes Playwright through the source/evidence-aware repository
wrapper, never a bare development-default command. Its lane prepares and proves its own
source-exact canonical topology, so it is not silently coupled to a sibling
lane receipt:

```powershell
.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/playwright-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -OutputPath "artifacts/sprint-8a-closeout/sit/playwright-<n>/deployment.json" -ApiContainerId <core> -GatewayContainerId <gateway> -DatabaseContainerId <postgres>
.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8088" -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/deployment.json" -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -EvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/playwright.json" -FailureEvidenceDirectory "artifacts/sprint-8a-closeout/sit/playwright-<n>/failures"
```

The runner's deployed lane uses a new attempt namespace and the exact frozen
environment fingerprint. It retains first/no-op materialization, initial
inventory/provenance/product smoke, release transition, induced failure,
empty-successor recovery, and final canonical read-back:

```powershell
.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp
.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-inventory.json"
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-deployment.json" -ApiContainerId <core> -GatewayContainerId <gateway> -DatabaseContainerId <postgres>
.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-smoke.json"
.\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/component-upgrade.json"
.\scripts\run-sprint-8a-failure-containment.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/failure-containment.json" -AuthorizeDisposableReset -SkipBuild
.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-inventory.json"
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-deployment.json" -ApiContainerId <core> -GatewayContainerId <gateway> -DatabaseContainerId <postgres>
.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-smoke.json"
```

- SIT result receipt: `artifacts/sprint-8a-closeout/sit-result.json`.
- Canonical topology restoration: current Sprint 8A Component release active;
  all services healthy; canonical fresh seed; only v3 Component references;
  no old reader, open failure or partial topology.
- Receipt publication: the SIT runner calls `Publish-Sprint8ALifecycleReceipt`
  with phase `sit`, the exact preflight and candidate references,
  candidate/environment/normalized-Compose identity, all four terminal results,
  and hashed `canonical_topology_verified` restoration evidence. The helper
  rejects a missing/renamed lane, another candidate, stale evidence, or
  incomplete restoration before writing `sit-result.json`; it does not execute
  or checkpoint the lanes.
- Finalization-only recovery: only an exact `evidence-finalization` failure at
  `publication-failed`, with the immutable raw-terminal checkpoint proving all
  four lanes passed and identity unchanged, may run:

```powershell
.\scripts\run-sprint-8a-sit.ps1 -Stage Finalize -Attempt <same-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/sit-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

`Finalize` never reruns a raw SIT lane. It reauthenticates the exact source,
candidate, environment, normalized Compose digest, and checkpoint, restores
canonical topology, and retries aggregate/manifest publication. Any other
failure returns to the coordinator-selected invalidation boundary.

## UAT

### Scripted UAT

- Tracked staged command interface (implemented; not run):

```powershell
.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Start -Attempt <n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088"
# Acquire, hold, execute, and publish UAT-8A-01 through UAT-8A-08 one at a time.
.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Finalize -Attempt <same-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

Readiness/preflight validates this interface without starting UAT by running
`.\scripts\run-sprint-8a-formal-uat.ps1 -SelfTest`.

- `Start` writes the attempt before fallible work, authenticates the exact
  preflight/candidate/SIT chain, recomputes clean source, candidate, and
  environment and normalized Compose identities, and runs fresh
  inventory/navigation plus Sprint 8A product smoke. It writes immutable
  `attempts/uat-<n>-manual-checkpoint.json` before manual execution.
- A user-authorized UAT-harness-only source advance may be applied at
  `Finalize` without rerunning signed manual scenarios when every changed path
  is in the exact formal-UAT runner, lifecycle helper, nondisclosure verifier,
  or Sprint 8A verification-document set. The runner retains the failed
  mutable attempt, writes an authenticated source-advance recovery receipt,
  proves that no product, fixture, deployment, acceptance inventory, or
  upstream-gate input changed, and then revalidates every manual receipt before
  canonical restoration and aggregate publication. An empty change set or any
  outside path is rejected.
- If scripted checks pass, manual scenarios are authoritative. If one or more
  scripted product/harness/environment assertions fail while identity remains
  trustworthy, the attempt enters `diagnostic-manual`: execute every safe
  independent manual scenario as non-authoritative diagnostic evidence and
  record true dependents blocked with exact reasons. Do not correct or restart
  until all eight scenarios have terminal receipts and Finalize consolidates
  the diagnostic pass. A source/candidate/environment identity failure blocks
  manual work. A `product-decision` blocks manual work and pauses for user
  direction rather than becoming a test failure.
- Manual receipts are exactly
  `uat/attempt-<n>/manual/uat-8a-01.json` through
  `uat/attempt-<n>/manual/uat-8a-08.json`. The attempt number is part of each
  typed receipt and its immutable path; a later attempt cannot reuse or
  supersede an earlier attempt's evidence. One scenario at a time must use the
  following held-lease sequence; the live lease spans all human actions and is
  consumed by receipt publication:

```powershell
. .\scripts\sprint-8a-lifecycle-chain.ps1
$scenario = "UAT-8A-<nn>"
$attempt = <uat-n>
$repositoryRoot = (Get-Location).Path
$evidenceRoot = "artifacts/sprint-8a-closeout"
$candidateFingerprint = "<candidate-fingerprint>"
$environmentFingerprint = "<environment-fingerprint>"
$preflightReference = [pscustomobject][ordered]@{ path = "artifacts/sprint-8a-closeout/preflight-result.json"; sha256 = "<sha256>" }
$sitReference = [pscustomobject][ordered]@{ path = "artifacts/sprint-8a-closeout/sit-result.json"; sha256 = "<sha256>" }
$startedAt = [DateTimeOffset]::UtcNow
$contract = Get-Sprint8AManualUatScenarioContract -Scenario $scenario
$plan = Get-Sprint8AManualUatEvidencePlan -Scenario $scenario -Attempt $attempt -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot
$executionLease = Open-Sprint8AManualUatScenarioLease -Scenario $scenario -Attempt $attempt -CandidateFingerprint $candidateFingerprint -EnvironmentFingerprint $environmentFingerprint -StartedAt $startedAt -Authoritative $true -Diagnostic $false -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot

$testerIdentity = [pscustomobject][ordered]@{
    tester_id = "<stable tester identity>"
    display_name = "<tester display name>"
    actor_bindings = @($contract.actor_bindings | ForEach-Object {
        [pscustomobject][ordered]@{ id = [string]$_.id; actor_id = "<exact account used for $([string]$_.id)>" }
    })
}
$preconditions = @(
    [pscustomobject][ordered]@{ id = "candidate-fingerprint"; state = "satisfied"; value = $candidateFingerprint; reference = $null },
    [pscustomobject][ordered]@{ id = "environment-fingerprint"; state = "satisfied"; value = $environmentFingerprint; reference = $null },
    [pscustomobject][ordered]@{ id = "preflight-receipt"; state = "satisfied"; value = $null; reference = $preflightReference },
    [pscustomobject][ordered]@{ id = "sit-result-receipt"; state = "satisfied"; value = $null; reference = $sitReference },
    [pscustomobject][ordered]@{ id = "evidence-root"; state = "satisfied"; value = $evidenceRoot; reference = $null },
    [pscustomobject][ordered]@{ id = "execution-start"; state = "satisfied"; value = $startedAt.ToString("o"); reference = $null }
)
$startingState = @($contract.required_starting_state | ForEach-Object {
    [pscustomobject][ordered]@{ id = [string]$_.id; observed_value = "<observed exact value>" }
})

# While $executionLease remains live, execute every manifest-bound step and
# write exactly one artifact at every $plan.evidence.path. For every
# authenticated-JSON requirement, including UAT-8A-05, UAT-8A-07, and
# UAT-8A-08, first write each assertion_raw_evidence path, then publish the
# authenticated wrapper/producer pair with the repository helper.
$evidence = [Collections.Generic.List[object]]::new()
foreach ($requirement in @($plan.evidence)) {
    if ([string]$requirement.kind -ceq "authenticated-json") {
        $assertionEvidence = @($requirement.assertion_raw_evidence | ForEach-Object {
            [pscustomobject][ordered]@{
                assertion_id = [string]$_.assertion_id
                raw_evidence = @([pscustomobject][ordered]@{
                    path = [string]$_.path
                    sha256 = Get-Sprint8AFileSha256 -Path (Join-Path $repositoryRoot ([string]$_.path))
                })
            }
        })
        $evidence.Add((Publish-Sprint8AManualUatStructuredEvidence -Scenario $scenario -Attempt $attempt -CandidateFingerprint $candidateFingerprint -EnvironmentFingerprint $environmentFingerprint -StartedAt $startedAt -RequirementId ([string]$requirement.requirement_id) -AssertionEvidence $assertionEvidence -ExecutionLease $executionLease -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot))
    } else {
        $evidence.Add([pscustomobject][ordered]@{
            step = [int]$requirement.step
            requirement_id = [string]$requirement.requirement_id
            kind = [string]$requirement.kind
            capture = $requirement.capture
            path = [string]$requirement.path
            sha256 = Get-Sprint8AFileSha256 -Path (Join-Path $repositoryRoot ([string]$requirement.path))
        })
    }
}
$actions = @($contract.steps | ForEach-Object { [pscustomobject][ordered]@{ step = [int]$_.step; action = [string]$_.action; expected_result = [string]$_.expected_result; actual_result = "<observed result>"; state = "passed" } })
$cleanupEvidence = @([pscustomobject][ordered]@{
    kind = "canonical-restoration"
    path = [string]$plan.cleanup.path
    sha256 = Get-Sprint8AFileSha256 -Path (Join-Path $repositoryRoot ([string]$plan.cleanup.path))
})
$endedAt = [DateTimeOffset]::UtcNow
Publish-Sprint8AManualUatReceipt -Scenario $scenario -Attempt $attempt -CandidateFingerprint $candidateFingerprint -EnvironmentFingerprint $environmentFingerprint -State passed -Authoritative $true -Diagnostic $false -AssertionsStarted $true -Role $contract.role -TesterIdentity $testerIdentity -Preconditions $preconditions -StartingState $startingState -Actions $actions -ExpectedResult "Every canonical step expectation in $scenario is satisfied." -ActualResult "<observed scenario result>" -Evidence @($evidence) -CleanupEvidence $cleanupEvidence -StartedAt $startedAt -EndedAt $endedAt -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot -OutputPath "$evidenceRoot/uat/attempt-$attempt/manual/$($scenario.ToLowerInvariant()).json" -ExecutionLease $executionLease
```

Every manifest requirement has exact cardinality one. Ordinary JSON evidence
uses the exact ordered fields `schema_version`, `sprint`, `phase`, `scenario`,
`attempt`, `evidence_id`, `candidate_fingerprint`, `environment_fingerprint`,
`captured_at`, and its typed array payload: `observations` for
`operator-record`, `entries` for `browser-console`, or `exchanges` for
`http-transcript`. Screenshots are real PNGs with `IHDR`; browser traces are
ZIPs containing non-empty `trace.trace` and `trace.network`. The one cleanup
JSON uses the exact ordered fields `schema_version`, `sprint`, `phase`,
`scenario`, `attempt`, `result`, `candidate_fingerprint`,
`environment_fingerprint`, `restored_at`, and `observations`, with phase
`uat-manual-canonical-restoration` and result
`canonical_topology_verified`. All capture/restoration timestamps must be at or
after `$startedAt`, and every path must be the exact path returned by `$plan`.

For `diagnostic-manual`, both lease and receipt use
`-Authoritative $false -Diagnostic $true`. A failed receipt supplies an
allowed `-Classification` and `-FailureMessage`; a blocked receipt uses
`-AssertionsStarted $false`, an exact `-BlockedReason`, the full canonical
action inventory with dependent actions marked `blocked`, and may use an empty
scenario `-Evidence @()` without fabricating observations. Cleanup evidence
remains mandatory. After one
authoritative manual defect, every later safe scenario is diagnostic. A manual
`product-decision` pauses later scenarios. If execution is interrupted, reopen
only the exact same lease binding and start time with `-Resume`; do not create a
replacement scenario identity. Finalize rejects every open or interrupted
start lease. Resume writes an authenticated marker bound to the original and
current process IDs. A resumed execution can retain a terminal failed or
blocked receipt, but it cannot establish an authoritative pass; a passing
scenario must be observed without a process-lineage break.
- `Publish-Sprint8AManualUatReceipt` validates the manifest/document hashes,
  exact acceptance-criterion and semantic-predicate bindings, tester/actor
  identity, typed preconditions and starting state, ordered
  step/action/expected-result identities, observed results, terminal state,
  assertion boundary, candidate/environment/checkpoint binding, every exact
  evidence identity/content/digest, canonical-restoration evidence, and the
  matching execution-lease start/completion pair. Before publishing either
  terminal file it writes
  an immutable prepared checkpoint containing the exact receipt and completion
  bytes; retry repairs only that bound publication transaction and never
  reruns the human scenario. Finalization can pass only with all eight exact
  authoritative, non-diagnostic passing receipts. An explicit
  `-ManualReceiptDirectory`, when supplied to the staged runner, must resolve
  to that same attempt-scoped canonical directory.
- `Finalize` requires the unchanged Start checkpoint and all eight exact manual
  receipts, revalidates source/environment, frozen endpoints, and the
  prerequisite chain, and requires explicit disposable-reset authorization. It
  then performs source-exact from-empty materialization plus semantic no-op,
  inventory/navigation audit, and Sprint 8A smoke with the frozen gateway,
  materialization-control, and Supervisor endpoints. The runner merges the
  prepared UAT result commitment and every retained artifact into the existing
  preflight/SIT evidence manifest. After scripted, manual, and restoration
  checks all pass, it writes and hashes a durable
  `finalization-completion-checkpoint.json`, then commits and publishes the
  canonical UAT JSON/sidecar pair. It does not consume or relabel rehearsal's
  `uat-sprint-8a.ps1` diagnostics.
- If publication alone fails after that completion checkpoint, rerun the same
  `Finalize` command. It reauthenticates the completion and retry checkpoints,
  source, candidate, environment, normalized Compose digest, endpoints,
  prerequisites, and passing SIT receipt, then executes only the publication
  tail; it does not rerun manual validation or destructive restoration.
  Canonical pair recovery recognizes exactly four states: `absent` publishes
  the committed JSON and sidecar; `json-only` authenticates JSON and writes only
  the committed sidecar; `sidecar-only` authenticates the sidecar and writes
  only the committed JSON; `complete` authenticates both without rewriting
  them. Mismatched or malformed partial bytes remain a retained
  `evidence-finalization` failure and are never blessed.
- Start rule: only after authoritative SIT passes on the same fingerprint.
- Result: Not Run.
- Evidence: `artifacts/sprint-8a-closeout/uat/attempt-<n>/`,
  `artifacts/sprint-8a-closeout/attempts/uat-<n>.json`, the immutable
  `attempts/uat-<n>-manual-checkpoint.json`, eight manual receipts and eight
  execution-lease pairs, finalization completion/result-commit evidence, any
  retained publication-retry evidence, `uat-result.json`, and
  `evidence-manifest.json`.

### Manual UAT

The machine-readable scenario contract is normative; this table is its exact
human-readable summary.

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| UAT-8A-01 unchanged Component experience | Component manager/reader; fresh canonical seed | Browse, create/edit/publish/version, exercise lifecycle and every kind; capture light/dark at 1280/768/390, keyboard, 200% zoom, no-JS SSR, hydration and console | Same accepted product behavior and canonical vocabulary; module owns documents/assets; responsive/theme/accessibility evidence complete; disposal succeeds | Not Run | `uat/attempt-<n>/manual/uat-8a-01.json` plus exact operator records, named screenshots, browser traces, and browser-console record |
| UAT-8A-02 from-empty seed and new references | Operator and Dashboard reader; explicitly authorized empty Sprint 8A databases | Run source-exact materialization; inspect exact owner read-back; open the seeded Dashboard and inspect resolved placements; inspect module inventory/navigation; rerun unchanged | Exactly 7 Component shells, 8 versions and 7 placements; predecessor/successor/action fixtures correct; only selected Component instance/v3; Dashboard appears once through its real Release/Instance; Core exposes exactly Forms, Workflows, Responses, Datasets, and Migration transitions; second run no-op | Not Run | `uat/attempt-<n>/manual/uat-8a-02.json` plus exact operator records and HTTP transcripts |
| UAT-8A-03 configuration and diagnostics | Global Module Management manager/reader plus Components manager | Set label and valid timeout; try blank/long label, 0/31 timeout, unknown field/version; inspect navigation/admin/product headings/diagnostics | Manifest defaults are Components/5; label/range/authority correct; binding/provider/contract/compatibility/health/result diagnostics are sanitized | Not Run | `uat/attempt-<n>/manual/uat-8a-03.json` plus exact browser trace, HTTP transcripts, and operator records |
| UAT-8A-04 Dataset contract, joint scope and outage | Operator, Component manager, dual-scope administrator, and restricted actors; Dataset and mixed Dashboard fixtures | Author/preview/execute; prove shared-node Dashboard render; inspect/attempt a disjoint placement while authorized on both scopes; compare known/random restriction; stop/timeout/recover Dataset provider; retry preserved editor | Typed reference only; shared-node path works; disjoint title/metadata/identity/data stay undisclosed and render is denied; known/random outcomes match; outage is read-only with no pending write; recovery succeeds | Not Run | `uat/attempt-<n>/manual/uat-8a-04.json` plus exact browser traces and HTTP transcript |
| UAT-8A-05 Dashboard lifecycle and outages | Operator, Component manager, Dashboard manager, and Dashboard reader; exact predecessor-bound action placements | Refresh exact findings; Defer then Upgrade; independently Replace and Remove; verify blocked nondisclosure; stop/recover Component | Three lifecycle findings and all four actions are exact; blocked placement absent; outage covers exact five authorized remaining placements; recovery has zero findings | Not Run | `uat/attempt-<n>/manual/uat-8a-05.json` plus exact browser traces and authenticated semantic receipt |
| UAT-8A-06 isolation and unsupported old inputs | Operator plus authorized/restricted actors | Inspect images/graphs/credentials/Core absence; submit V1/V2, old owner/type, retired `missing_policy`, flat Dataset-major, and old payload requests | No forbidden source/storage edge; old inputs fail exact normal contract with nondisclosure; no adapter/ledger/fallback | Not Run | `uat/attempt-<n>/manual/uat-8a-06.json` plus exact operator record and HTTP transcripts |
| UAT-8A-07 failed materialization rerun | Operator; disposable authorized topology; induced owner-bootstrap failure | Run failing attempt, retain hashed raw apply response and service-log evidence, verify exact teardown, remove the fault and run a complete successor, then repeat bootstrap unchanged | Failure retained; partial project/volumes absent; successor begins empty, reaches canonical health, and reports an exact semantic no-op on repeat | Not Run | `uat/attempt-<n>/manual/uat-8a-07.json` plus exact operator records and authenticated failure-containment receipt |
| UAT-8A-08 Component-only upgrade/rollback | Operator/reviewer; healthy `1.0.0` current release plus source-built `0.9.0` fixture | Establish `0.9.0`, upgrade to `1.0.0`, roll back, restore `1.0.0` through Supervisor/Compose; compare snapshots | Exact one-owner health-gated deltas; distinct release/binary identities; Component state retained; unrelated identities/restarts unchanged; final topology healthy | Not Run | `uat/attempt-<n>/manual/uat-8a-08.json` plus exact operator records and authenticated upgrade/rollback receipt |

- UAT result receipt: `artifacts/sprint-8a-closeout/uat-result.json`.
- Final topology restoration: intended Sprint 8A route/slot and current
  Component release; healthy Core/Component/Dashboard/Dataset path; canonical
  fresh seed; only v3 live references; no open defect.

## Changed Integration Contracts

| Boundary | Implemented change (formal proof pending) | Required negative proof |
|---|---|---|
| Component public contract | exact-current `3.0.0`, Module Instance owner and `tessara.components.component_version` | old/mixed/wrong-instance/cross-installation/malformed/unauthorized inputs fail closed |
| Component render response | Components-owned exact Table/visual DTOs and render kind; non-nil exact persistent identity; explicit both-nil preview identity | unknown/cross-kind fields, wrong branch/kind/schema, Dataset identity, mismatched/nil persistent identity, and non-nil/partial-nil preview fail closed |
| Component product API | canonical module-owned typed-reference bodies on stable same-origin product paths | old Core payloads rejected; no translation reader/facade |
| Signed module service requests | exact transmitted body bytes verified before typed decode across JSON-bearing Component, Core authorization-exchange and Core Dataset routes | changed whitespace/field bytes without resigning, wrong media type, wrong correlation/grant/service identity, or stale digest fail before product work |
| Core Dataset compatibility | typed Dataset-major-line reference and versioned catalog/schema/distinct/execution/compatibility operations | no private DTO/SQL/credential; wrong audience/scope/version and outage do not disclose or fall back |
| Module configuration and diagnostics | Manifest schema v1 defaults `Components`/`5`; real image command paths; selected Dataset binding compatibility/health observation | unknown schema/field, invalid label, 0/31 timeout and wrong authority rejected; no raw references/secrets; command paths exist in image |
| Dashboard dependency | v3 provider binding; exact predecessor/successor and action placements; authorized Dashboard-scope intersection; exact ComponentVersion resource assertion; one common governing node; structured semantic evidence | no old owner/type or blocked/disjoint-scope metadata/title/data disclosure; disjoint render denied; Defer/Upgrade/Replace/Remove and Component outage/recovery cannot pass from broad labels |
| Active first-party acceptance inputs | exact v3 Component and nested typed Dataset-reference fixtures; 84 exact Playwright identities | retired normalization aliases, flat Dataset-major fields, copied counts, or unmanifested scenario changes rejected |
| Fresh bootstrap | destructive owner-ordered seed plus generic lockfile-owned dependency validation and exact 7-shell/8-version/7-placement idempotent seed | ambiguous or mismatched target, altered payload/request/apply/owner/audience, expiry, and replay rejected; no cross-owner or pre-validation writes; exact predecessor/successor/action identities; second run no-op; failed partial topology never reused |
| Component release transition | source-built compatible `0.9.0` and corrected candidate `1.0.1`; exact Supervisor/Compose one-owner deltas | candidate relabel rejected; release/binary identities distinct; unrelated owners absent from plan and unchanged in snapshots |
| Core ownership | generic platform integration only | Component product storage/code/routes/adapter/readers absent |

## Source Provenance, Schema Baseline, And Evidence Rules

- Freeze inputs include commit/tree/dirty state and every tracked product,
  test, harness, schema migration, fixture, seed, manifest, bootstrap,
  deployment, configuration, contract and acceptance identity.
- Candidate provenance matches image labels, release/instance read-back,
  contract/manifest/asset/schema/seed digests and deployed diagnostics.
- The baseline is empty, owner-specific schema materialization. No populated
  Core Component baseline, product-data mapping or legacy reference ledger is
  a candidate input.
- Seed assertions use semantic identity sets. Any contractual exact count comes
  from one shared acceptance source of truth.
- Receipts record commands, timestamps, duration, exit status, assertions, raw
  paths/hashes, prerequisites, candidate/environment identity, classification,
  invalidation, teardown and restoration.
- Secrets, credentials, private keys and token values never enter receipts.

## Failure And Invalidation Chronology

| Cause | Minimum invalidation |
|---|---|
| Candidate fingerprint changed | All SIT and UAT |
| Acceptance inventory or tracked harness changed | All SIT and UAT |
| Shared environment materially changed | Affected lane and downstream phases |
| Lane-local setup failure before assertions | Failed lane after prerequisites reconfirm |
| Evidence finalization failure with immutable complete raw results | Finalization only |
| Test assertion failure with unchanged candidate | Complete failed lane; coordinator assesses upstream relevance |
| UAT scenario setup failure before product actions | Affected isolated scenario set when prerequisites reconfirm |
| Product defect corrected | Refreeze after complete readiness/rehearsal, then all SIT and UAT |
| Missing acceptance assertion discovered | Update inventory/candidate, then all SIT and UAT |
| Candidate-affecting correction after a mutable gate pass | Supersede that pass; run new complete readiness and rehearsal on one corrected source/environment identity |

Runtime chronology:

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction/narrow proof | Invalidation scope | Authoritative replacement |
|---|---|---|---|---|---|---|---|
| 2026-08-06 22:00 EDT | Readiness 34 / `compose-database-contract` | Yes | `c920b8a3` | `environment` | Fresh six-binding runner environment required; retained in the single Readiness 34 batch | Readiness 34 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:00 EDT | Readiness 34 / `environment-contract` | No; blocked by `compose-database-contract` | `c920b8a3` | `environment` | Exact dependency reason retained in the same batch | Readiness 34 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:00 EDT | Readiness 34 / `reset-dry-run` | Yes | `c920b8a3` | `harness` | Shared optional-property projection and executable self-test; retained in the same batch | Readiness 34 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:19 EDT | Readiness 35 / terminal evidence publication | Yes; all 15 checks passed | `49c0ae73` | `evidence-finalization` | Shared empty-result classification projection for Readiness and Rehearsal; one R35 batch | Readiness 35 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:34 EDT | Readiness 36 / complete gate | Yes; all 15 checks passed | `3f7e32cb` | N/A — passed, later superseded | Complete receipt retained for prior clean source and environment `2e235b07...` | Readiness 36 and all downstream phases after candidate-affecting audit findings | New complete Readiness and Rehearsal |
| 2026-08-06 22:36 EDT | Rehearsal 29 / `dashboard-source-boundaries`, `markdown-links` | Yes; both child scripts passed | `3f7e32cb` | `harness` | Two failed receipts retained; one stale parent `$LASTEXITCODE` root defect identified from raw logs | Rehearsal 29 and all downstream phases | Consolidated implementation correction, then new complete Readiness and Rehearsal |
| 2026-08-06 22:53 EDT | Rehearsal 29 / `workspace-tests` | Yes | `3f7e32cb` | `harness` (test fixture) | Failed receipt and exact 400/503 assertion retained; fixture omitted required `amount` while product correctly rejected invalid input | Rehearsal 29 and all downstream phases | Consolidated implementation correction, then new complete Readiness and Rehearsal |
| 2026-08-06 after 22:53 EDT | Rehearsal 29 / `optimized-resource-reference-timing` and remaining graph | Optimized lane interrupted; 20 checks not started | `3f7e32cb` | N/A — user-directed phase exit | One interrupted receipt, 20 unexecuted checks, and zero recorded blocked receipts retained without inferring results | Rehearsal 29 and all downstream phases | New complete Readiness and Rehearsal; Attempt 29 may not resume |
| 2026-08-06 after phase exit | Implementation audit / Component render boundary | Source audit started | mutable successor | `product` | Components contract owns exact kind-specific responses; provider/routes return them; Dashboard validates them; renderer has no Dataset-contract edge or copied DTOs; persistent and preview identities are disjoint exact contracts | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Implementation audit / signed service body | Source audit started | mutable successor | `product` | Verify exact raw bytes before typed decode across Component, authorization exchange and Dataset routes; retain JSON/body limits; add byte-tamper proof | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Implementation audit / Dashboard joint scope and resource assertion | Source audit started | mutable successor | `product` | Forward authorized Dashboard-scope intersection; bind exact ComponentVersion assertion; require one governing node; redact disjoint projection; add Rust/browser/UAT proof | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Implementation audit / active legacy acceptance facade | Source audit started | mutable successor | `harness` | Remove normalization aliases and flat Dataset-major fixtures; enforce exact v3 typed inputs; advance manifest to 75 exact scenarios | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Intermediate implementation diagnostics | Yes; narrow and workspace assertions ran | mutable successor identities | N/A — later superseded | Focused checks and one 684.9-second all-feature workspace pass completed before final preview/joint-scope edits | No gate or downstream phase authorized | Rerun final implementation checks, then complete Readiness and Rehearsal |
| 2026-08-07 | Final implementation-entry verification | Yes; complete workspace and static implementation suites ran | final corrected mutable product source | N/A — implementation checks passed | Six isolated databases reset; 619.6-second all-feature workspace suite, warnings-denied clippy, check, formatting, focused integrations, boundary/acceptance/link/runner audits, 75/75 inventory, TypeScript, and Playwright discovery passed | No formal gate or downstream phase authorized | Commit cleanly, then run complete Readiness and Rehearsal |
| 2026-08-07 01:38 EDT | Readiness 37 / complete gate | Yes; all 15 checks passed | `84964c7b` / `4f39cb71`, environment `96c2a32e...` | N/A — passed, later superseded | Complete receipt retained with SHA-256 `a4ab361a...` | Readiness 37 and all downstream phases after R30 correction | Successor complete Readiness consuming R30 authorization, then complete Rehearsal |
| 2026-08-07 01:39–02:52 EDT | Rehearsal 30 / four raw failed lanes | Yes in fact; R30 omitted the required per-lane marker | `84964c7b` | Raw: two `environment`, one `product`, one `harness`; diagnosed: three `harness` roots | Action-object projection, source-identity shape, and process-environment restoration corrected as one batch; raw receipts unchanged | Rehearsal 30, Readiness 37, and all downstream phases | Successor complete Readiness and complete Rehearsal |
| 2026-08-07 01:39–02:52 EDT | Rehearsal 30 / ten blocked lanes | No in fact; R30 omitted the required per-lane marker | `84964c7b` | N/A — exact dependency blocks | Ten exact blocked reasons retained; assertion counts corrected to exclude blocked terminal receipts | Rehearsal 30 and all downstream phases | Successor complete Readiness and complete Rehearsal |
| 2026-08-07 02:52 EDT | Rehearsal 30 / harvest and correction authorization | No product assertions | `84964c7b` | Four raw symptoms; three diagnosed `harness` roots | One immutable harvest, one consolidated batch, and one unconsumed one-use successor-Readiness authorization retained | Correction is candidate-affecting; no freeze | Correct batch, consume authorization in successor Readiness, then complete Rehearsal |
| 2026-08-07 after user-directed testing exit | Implementation audit / Dashboard executable identity | Source audit started | mutable successor | `product` | Current runtime/migration commands now name `/usr/local/bin/dashboard-module`, the executable installed by the image; exact Manifest/Dockerfile assertions and canonical catalog digest advanced together | Readiness 37, Rehearsal 30, and all downstream phases | Commit the consolidated implementation correction, then successor complete Readiness and Rehearsal |
| 2026-08-07 after user-directed testing exit | Implementation audit / Component exact-current configuration | Source audit started | mutable successor | `product` | Removed retired object-shaped `visible_columns` and filter `field` readers from validation, provider and browser; added rejection/source guards and advanced asset/Manifest/catalog digests together | Readiness 37, Rehearsal 30, and all downstream phases | Commit the consolidated implementation correction, then successor complete Readiness and Rehearsal |
| 2026-08-07 06:30 EDT | Readiness 38 / `compose-database-contract` | Yes | `91c9936b` / `564493fd`; environment unverified | `environment` | Missing same-process `TEST_API_DATABASE_URL`; raw log and complete failed attempt retained | Readiness 38 and all downstream phases | Exit testing; correct the consolidated entry batch; finalize R38; then Readiness 39 and Rehearsal 31 |
| 2026-08-07 06:30 EDT | Readiness 38 / `runner-self-tests` | Yes | `91c9936b` / `564493fd` | `harness` | Failure-containment self-test used invalid overwrite after deliberate corruption; raw log retained. The correction proves overwrite rejection, removes only the exact corrupt no-journal pair inside the validated temporary self-test root, and republishes create-once; real corrupt retained evidence remains preserved. | Readiness 38 and all downstream phases | Same consolidated correction and full-gate boundary |
| 2026-08-07 06:30 EDT | Readiness 38 / `environment-contract` | No; blocked by `compose-database-contract` | `91c9936b` / `564493fd` | N/A — exact dependency block | Exact blocked reason retained; not counted as an independent defect | Readiness 38 and all downstream phases | Same consolidated correction and full-gate boundary |
| 2026-08-07 after Readiness 38 testing exit | Plan-to-source implementation audit | Focused source review only | mutable correction successor | `product`, `harness`, and evidence/process enforcement | Added failed-Readiness correction lineage, truthful Rehearsal checkpoints, exact Component missing-policy contract, and canonical UAT scenario/evidence/AC-18/AC-19 enforcement | R38, all mutable receipts, and every downstream phase | Focused implementation verification and clean commit; no formal gate in this phase |
| 2026-08-07 after Readiness 38 testing exit | Focused lifecycle/formal-UAT/diagnostic-UAT/preflight self-tests / initial diagnostic pass | Yes in lifecycle/formal checks; dependent self-tests also started | mutable correction successor | `harness` | Raw self-test output retained in the implementation session: lifecycle JSON evidence incorrectly required structured-authority parameters and formal-UAT partial-pair repair removed an authenticated JSON-only state; diagnostic-UAT and preflight then failed through the lifecycle dependency. Two root defects and two dependent outcomes were corrected together. | No formal gate result; no candidate or downstream phase authorized | Rerun the complete four-self-test batch only after the consolidated correction |
| 2026-08-07 after Readiness 38 testing exit | Focused lifecycle/formal-UAT/diagnostic-UAT/preflight self-tests / three-root fail-late pass | Yes; all four independent self-tests ran; zero blocked | mutable correction successor | `harness` | Four raw failures exposed three roots before correction: temporary structured-evidence cleanup left stale manifest commitments (lifecycle and formal UAT), acceptance still required retired `{01..08}` lease patterns (diagnostic UAT), and preflight's exact inventory selector omitted `manual-leases`. The three roots were one consolidated narrow correction batch. | No formal gate result; no candidate or downstream phase authorized | Rerun all four self-tests after the shared acceptance correction |
| 2026-08-07 after Readiness 38 testing exit | Focused four-self-test successor diagnostic pass | Lifecycle and formal UAT passed; diagnostic UAT and preflight started and failed; zero blocked | mutable correction successor | `harness` | Both failures had one shared root: preflight extracted `$declaredChecks` with a broad source span, so new readiness self-test fixtures appeared as duplicate declarations. Exact AST assignment extraction replaces the broad scan. | No formal gate result; no candidate or downstream phase authorized | Rerun the same complete four-self-test batch; only a clean result is implementation-entry proof |
| 2026-08-07 after Readiness 38 testing exit | Focused four-self-test post-AST diagnostic pass | Lifecycle, formal UAT, and preflight passed; diagnostic UAT started and failed; zero blocked | mutable correction successor | `harness` | Acceptance correctly found that declared-independent Rehearsal lane `validation-readiness-prerequisite` consumed `validation_state` and immutable-readiness context produced only by sibling `attempt-state-prerequisite`. The readiness/source/environment lane must authenticate its supplied receipt independently; the state lane alone owns current-state binding. | No formal gate result; no candidate or downstream phase authorized | Correct the Rehearsal lane coupling, then rerun the same complete four-self-test batch |
| 2026-08-07 after Readiness 38 testing exit | Focused four-self-test zero-evidence-block diagnostic pass | All four reached the same new lifecycle publisher proof; zero blocked | mutable correction successor | `harness` | The public manual-receipt publisher accepted an empty scenario-evidence array, but its private evidence resolver still rejected the empty `References` binding. The end-to-end dependent-block self-test exposed the remaining representation gap; the inner resolver now explicitly accepts an empty scenario array while cleanup evidence stays mandatory. | No formal gate result; no candidate or downstream phase authorized | Rerun the same complete four-self-test batch |
| 2026-08-07 after Readiness 38 testing exit | Focused post-coupling four-self-test diagnostic pass | Lifecycle, formal UAT, and preflight passed; diagnostic UAT started and failed; zero blocked | mutable correction successor | `harness` | The acceptance ordering guard selected the lane-marker literal inside the new isolation self-test instead of the live `validation-readiness-prerequisite` invocation. Selecting the last exact marker binds the guard to the live lane; a focused diagnostic-UAT repro then passed. | No formal gate result; no candidate or downstream phase authorized | Rerun all four self-tests against the same stable bytes |
| 2026-08-07 after Readiness 38 testing exit | Focused correction-lineage/evidence compatibility review | Source assertions and disposable self-tests only | mutable correction successor | `harness` and evidence/process enforcement | One sub-batch corrected four related findings: live callers now supply `receipt`, not `path`, to `ExpectedCurrentReadiness`; passing Rehearsal attempt/result use one exact Readiness-prerequisite representation; `produced_evidence: null` no longer becomes a bogus raw-evidence entry during failed-Readiness finalization; and raw-evidence resolution rejects outside-root absolute paths. | No formal gate result; no candidate or downstream phase authorized | Parser and runner-local/preflight self-tests, followed by the complete four-self-test batch |
| 2026-08-07 after Readiness 38 testing exit | Final focused lifecycle/formal-UAT/diagnostic-UAT/preflight self-test batch | Yes; all four passed; zero failed and zero blocked | final mutable correction source | N/A — focused implementation proof passed | Lifecycle schema/identity, formal-UAT prerequisite/staging/receipt, automated-UAT diagnostic identity/freshness, and validation-preflight graph/receipt/classification/no-execution self-tests all passed against the same stable working tree. This includes authenticated structured evidence, exact inventory, true zero-evidence dependent blocks, and Rehearsal lane isolation. | No formal gate result; no candidate or downstream phase authorized | Complete implementation review and clean commit, then new complete Readiness and Candidate Rehearsal |
| 2026-08-07 after Readiness 38 testing exit | Pre-audit consolidated testing-entry verification | Focused assertions only; no lifecycle runner attempt | mutable correction source | N/A — implementation checks passed | All 12 fail-late harness checks passed; formatting, JavaScript syntax, 33 Component library tests, warnings-denied Component clippy, exact five-transition identity, three exact Dashboard/inventory/navigation API tests, and 75-scenario Playwright discovery also passed. No browser scenario or formal gate ran. | No formal receipt, candidate, or downstream phase authorized | Independent final diff audit before commit |
| 2026-08-07 final diff audit | Attempt reservation, UAT-8A-07 raw evidence, and documentation reconciliation | Read-only source audit | mutable correction source | `harness` and evidence/process enforcement | Four actionable findings were retained as one final batch: an out-of-sequence Readiness probe could occupy the authorized successor namespace; UAT-8A-07 could claim raw failure retention through a generic operator record; the Rehearsal section still called R30 authorization unconsumed; and the manual-UAT summary differed from the exact manifest. The batch reserves the attempt under lock before all five namespace targets, requires authenticated hashed apply/log assertions, and reconciles the documentation. | No formal receipt, candidate, or downstream phase authorized | Rerun the complete affected focused cone before commit |
| 2026-08-07 final correction verification | Readiness/lifecycle/formal-UAT/diagnostic-UAT/preflight/acceptance/harvest/Rehearsal self-tests | Focused assertions only; no lifecycle runner attempt | final mutable correction source | N/A — affected implementation checks passed | PowerShell/JSON parsing, Readiness reservation/finalization, lifecycle schema/identity, formal UAT, diagnostic UAT, preflight, acceptance, harvest adversarial, and Candidate Rehearsal self-tests passed. UAT now has 46 globally unique exact-one requirements; the new failure-containment receipt rejects either missing raw assertion. | No formal receipt, candidate, or downstream phase authorized | Clean diff/evidence audit and one correction commit; then R38 finalization at the documented boundary |
| 2026-08-07 09:17–09:22 EDT | Readiness 39 / complete gate | Yes; all 15 checks passed | `ffe05ace` / `1d17533d`, environment `96c2a32e...` | N/A — passed, later superseded | Exact immutable receipt and canonical alias retained with SHA-256 `b28da23a...` | R39 and all downstream phases after R31/candidate-affecting correction | Consume R31 authorization in a new complete R40 |
| 2026-08-07 09:22–10:44 EDT | Rehearsal 31 / complete fail-late harvest | Yes in 22 lanes; 10 lanes and 8 nested UAT scenarios dependency-blocked | `ffe05ace` / `1d17533d`, environment `96c2a32e...` | Raw: 2 `product`, 1 `harness`; 10 exact lane blocks | 19 passes, 3 failures, 10 blocks; one harvest, one three-defect batch, one unconsumed exact-R40 authorization; raw evidence unchanged | Rehearsal 31, R39, and every downstream phase | Correct the consolidated batch; then R40 and R32 |
| 2026-08-07 after R31 terminalization | Bounded failure-first scheduler and R31 defect correction | Focused implementation assertions only | mutable successor | `harness`, `environment`, deployment/acceptance/evidence enforcement; raw R31 labels retained | Add fresh-database generation guard, exact Supervisor target enrollment, normalized Compose parser, precursor-evidence containment guard, immutable two-wave schedule, exact deferral schema/state/harvest/lineage enforcement, recovery, and adversarial self-tests | No formal gate result; no candidate or downstream phase authorized | Clean correction commit; coordinator-authorized R40 then conservative full-Wave-A R32 |
| 2026-08-07 12:27–12:32 EDT | Readiness 40 / complete gate | Yes; all 15 checks passed | `b4f1581e` / `af0ab5cf`, environment `6967aa2c...` | N/A — passed, later superseded | Exact immutable receipt and canonical alias retained with SHA-256 `f4fd1745...`; R32 conservative schedule SHA-256 `7b09c1b3...` | R40 and all downstream phases after the pre-start correction | Complete corrected Readiness 41, then the then-unused R32 identity |
| 2026-08-07 12:35 EDT | Rehearsal 32 / rejected pre-start launch | No; no immutable start, attempt receipt, lane registration, assertions, or product actions | `b4f1581e` / `af0ab5cf`, environment `6967aa2c...` | One `harness` defect | Ordered-dictionary declarations were rejected by PSObject-only member validation; raw log, typed failure, and one consolidated pre-start batch retained; authenticated empty reservation residue removed | R40 and every downstream phase; R32 was still unused at this boundary | Correct representation-aware validation and live-shape self-test together; commit; complete R41; then R32 |
| 2026-08-07 after R32 pre-start rejection | Ordered-dictionary scheduler correction / focused implementation verification | Focused self-tests and static checks only; no lifecycle attempt | mutable correction source | N/A — focused implementation proof passed | Candidate Rehearsal and Readiness self-tests, complete Sprint 8A acceptance contract, PowerShell parsing, Markdown links, formatting, workspace check, and all-target/all-feature warnings-denied Clippy passed; missing scheduler role/impact members fail closed on the live declaration shape | No formal receipt, candidate, or downstream phase authorized | Clean commit; coordinator-authorized R41 then the still-unused R32 identity |
| 2026-08-07 12:58–13:00 EDT | Readiness 41 / rejected pre-start launch and diagnostic harvest | No; no reservation, immutable start, attempt receipt, check registration, assertion, or product action | `231bd8d9` / `359789b1`; current retained environment `6967aa2c...` | Two `harness` defects | R40's schema-3 terminal was rejected by schema-2-only lineage validation; the same diagnostic pass proved clean rerun admission could not preserve an already-consumed lineage. Launch log, diagnostic log, typed failure, and one two-defect pre-start batch are retained; no R41 namespace exists. | `231bd8d9`, R40, and every downstream phase; R41 and R32 remain unused | Correct lineage-terminal schema enforcement and exact clean-pre-rehearsal supersession together; focused verification and clean commit; coordinator-authorized R41 then R32 |
| 2026-08-07 13:56–14:01 EDT | Readiness 41 / complete gate | Yes; all 15 checks passed | `d703e7a6` / `889ddc5f`, environment `53fbb1f7...` | N/A — passed, later superseded | Immutable start `8365d2b9...`; immutable terminal/canonical alias `547940b0...`; R32 schedule `7b09c1b3...` | R41 and downstream phases after R32 candidate-affecting findings | Consume qualified R32 transition in complete R42 after correction |
| 2026-08-07 14:06–15:37 EDT | Rehearsal 32 / complete conservative fail-late harvest | Yes in 21 lanes; 11 exact dependency blocks | `d703e7a6` / `889ddc5f`, environment `53fbb1f7...` | Raw: 2 `product`; 11 exact blocks; diagnosed shared `harness` health root | 19 passes, 2 failures, 11 blocks, 0 deferrals; immutable attempt `98cddc03...`, harvest `89861d8a...`, and one raw two-defect batch `4a5219fd...`; no passing result | R32, R41, and every downstream phase | Preserve raw evidence; restore canonical environment; correct one consolidated batch; then R42/R33 |
| 2026-08-07 15:37–16:04 EDT | R32 append-only diagnosis, canonical restoration, and authorization qualification | Yes in diagnostic restoration only; no formal gate | Same retained `d703e7a6` / `889ddc5f`; predecessor environment `53fbb1f7...` | `harness`, `evidence-finalization`, stale smoke `harness`, and Dashboard seed `product`; raw R32 labels unchanged | Diagnostic supplement `8bcfc25c...`; restoration `e082eefd...`; qualification `859a8dba...`; validation state `85ac129f...`. The old authorization is usable only as the R42 tuple and authorizes no later phase. | Current tracked product/harness/evidence cone and all downstream phases | Focused implementation verification and clean commit; coordinator-authorized complete R42 then complete R33 |
| 2026-08-09 | SIT 1 / prerequisite setup | No SIT lane started | frozen candidate `a8cfdfce`; environment `e3fef334...` | `harness` | Absolute in-repository evidence paths were rejected instead of normalized; retained receipt and sidecar remain unchanged | SIT only; Readiness, Rehearsal, and Preflight behavior unaffected | Normalize contained paths and reject outside-repository paths, then start a new SIT attempt |
| 2026-08-09 | SIT 2 / complete prerequisite harvest | Prerequisite assertion started; 4 lanes blocked | same frozen candidate/environment | `evidence-finalization` | One failure and four exact dependency blocks retained; the frozen manifest could not yet inventory the post-freeze SIT 1 receipt | SIT only | Add append-only failed-SIT manifest finalization, inventory the retained failure without rewriting it, then start a new SIT attempt |
| 2026-08-09 | SIT 3 / runner interruption | Formatting completed, but the lane result was not committed; 4 lanes terminalized blocked | same frozen candidate/environment | `harness` | `Task[VoidTaskResult]` values leaked from asynchronous stream-copy completion ahead of the named result. Focused process-result-shape self-test now requires exactly one named result. | SIT only | Commit the focused runner correction, then start a new SIT attempt |
| 2026-08-09–10 | SIT 4 / complete fail-late harvest and restoration | All 4 lanes terminal; 2 passed and 2 failed; 0 blocked | same frozen product candidate; SIT harness advance recorded | Raw: 4 `product`; diagnosed: 1 stale SIT smoke-contract root | Formatting, full Rust, 75/75 Playwright, materialization/no-op, inventory, exact Core/Supervisor health, component upgrade/rollback, failure containment, and recovery passed. Four deployment-evidence checks all failed only because the generic legacy shell smoke still required `Transitional — not independently deployable`; exact inventory proved five Core transitions and Dashboard only as independently deployed. | SIT command contract and Preflight declaration only; Readiness and Rehearsal unaffected | Replace redundant generic shell smoke in SIT with direct deployment-evidence capture, rerun Preflight only to bind the corrected downstream command/source, then run a new complete SIT |
| 2026-08-10 | Preflight 19 / complete fail-late harvest | 5 checks passed, 2 failed, and 3 were dependency-blocked | corrected SIT/preflight harness source; Readiness 51/Rehearsal 40 unchanged | `environment` invocation error and `harness` freeze-supersession gap | The launch omitted `TEST_API_DATABASE_URL`. Separately, the runner rejected the still-present Preflight 18 freeze but had no authenticated way to archive it after failed SIT. The correction moves the exact old preflight, candidate, manifest, and sidecars to an immutable attempt archive, writes a hashed mapping bound to the failed SIT receipt, and only then permits replacement. | Preflight and downstream only; Readiness and Rehearsal unaffected | Pass all six exact database variables and rerun Preflight with the existing harness-only source-advance authorization |
| 2026-08-10 | UAT 9 / first Finalize attempt | Eight signed manual scenarios passed; aggregate publication stopped before manual receipt validation or canonical restoration | frozen candidate `51933925...`; UAT harness source advanced after Start | `preflight/setup` plus dependent catch-harvest `harness` | Finalize rejected an authorized UAT-only source advance, then its catch collector converted valid JSON timestamps to locale text and rejected the lost offsets. Raw failure evidence and the failed mutable attempt are retained. The correction accepts only an authenticated subset of the exact UAT validation/document paths, archives the failed attempt, writes a source-advance recovery receipt, preserves all eight signed scenario receipts, and normalizes typed timestamps back to offset-bearing ISO 8601. | UAT only; Readiness 51, Rehearsal 40, Preflight 22, and SIT 5 remain valid | Run focused formal-UAT/lifecycle self-tests, commit cleanly, then retry UAT 9 Finalize with the explicit UAT-harness-only source authorization; do not rerun manual scenarios or upstream gates |
| 2026-08-10 | UAT 9 / second Finalize attempt | Eight manual receipts revalidated and all three canonical-restoration checks passed; aggregate assembly then failed | same frozen candidate/environment; corrected UAT harness source `ac8e659a...` | `harness` empty-result projection | Strict mode rejected direct `.classification` access on the empty scripted-failure array. The failed attempt, source-advance receipt, manual revalidation, materialization/no-op, exact inventory, and exact smoke evidence remain retained. The correction projects classifications item by item, includes an empty-list reproducer, and permits an aggregate-only retry only when all three retained restoration checks and every referenced digest reauthenticate. | UAT aggregate publication only; no manual scenario, restoration action, or upstream gate is affected | Run the focused UAT/acceptance/link checks and commit; retry UAT 9 Finalize with explicit UAT-harness-only authorization, reusing the authenticated passing restoration evidence |
| 2026-08-10 | UAT 9 / aggregate-recovery precheck | No product, manual, or restoration assertion started; the retained second-attempt state was unchanged | same frozen candidate/environment; aggregate-only correction source `e269fc0d...` | concentrated `harness` validation-platform incident | A singleton failure-message projection was unwrapped to a scalar before recovery qualification checked `.Count`. This was the third consecutive Finalize-path failure, so formal retries stopped. The correction centralizes failure-message projection and normalizes its caller to an explicit collection, with singleton plus empty cardinality reproducers before any further launch. | UAT finalization platform only; all retained UAT scenario/restoration evidence and upstream gates remain unaffected | Pass the focused formal-UAT, lifecycle, acceptance, and link checks; commit cleanly; only then retry aggregate-only Finalize |
| 2026-08-10 | UAT 9 / cumulative recovery qualification | No product, manual, restoration, or aggregate assertion started; the retained failed attempt accumulated the stopped-retry evidence | same frozen candidate/environment; cardinality correction source `4d393a03...` | concentrated `harness` validation-platform incident | The stopped retries correctly retained three exact Finalize-path errors, but recovery qualification still required the original error to be the only error. Recovery therefore never reached the manifest override synchronization, and the stale-manifest error was retained as the third incident message. The correction accepts only the unique one-to-three-message prefix/set of this known incident containing the original aggregate failure, rejects unknown or duplicate errors, and builds one exact override set containing the mutable attempt and every retained recovery receipt. | UAT finalization evidence only; retained manual/restoration proof and Readiness, Rehearsal, Preflight, and SIT remain unaffected | Pass the focused manifest/UAT/lifecycle/acceptance/link checks; commit cleanly; retry aggregate-only Finalize without rerunning manual scenarios, restoration, or upstream gates |
| 2026-08-10 | UAT 9 / restoration-evidence reuse aggregate | Eight manual receipts and all three retained restoration receipts reauthenticated; aggregate assembly then stopped | same frozen candidate/environment; cumulative recovery source `30d814f0...` | concentrated `harness` validation-platform incident | The reuse branch supplied the passing restoration aggregate and its three checks but did not initialize the empty failure and blocked collections shared with the normal restoration branch. Strict mode stopped on the first later read. The correction initializes those collections before branch selection and adds an exact recovery reproducer for this retained failure. | UAT aggregate publication only; no manual scenario, restoration action, product source, or upstream gate is affected | Pass the focused UAT/lifecycle/acceptance/link checks; commit cleanly; retry aggregate-only Finalize while reusing the authenticated manual and restoration evidence |
| 2026-08-10 | UAT 9 / manual chronology aggregation | Eight manual receipts and all three retained restoration receipts reauthenticated; final manual aggregate validation then stopped | same frozen candidate/environment; restoration-reuse source `5eba1c1e...` | concentrated `harness` validation-platform incident | PowerShell parsed the valid offset-bearing JSON timestamps as date objects, then the normal manual-check projection cast them to locale strings and dropped their UTC offsets. The manual receipt validation itself had already accepted the chronology. The correction routes assertion, start, and end timestamps through the canonical offset-preserving converter before aggregate validation and includes a deserialized-JSON reproducer. | UAT aggregate publication only; signed manual evidence, restoration evidence, product source, and upstream gates remain valid | Pass the focused UAT/lifecycle/acceptance/link checks; commit cleanly; retry aggregate-only Finalize without rerunning manual scenarios, restoration, or upstream gates |
| 2026-08-10 | UAT 9 / canonical result publication | Eight manual receipts, all three retained restoration receipts, and final aggregate chronology passed; canonical lifecycle publication then stopped | same frozen candidate/environment; chronology source `97b987af...` | concentrated `harness` validation-platform incident | The canonical UAT result paired the frozen candidate fingerprint with the newer UAT validation-harness source, so lifecycle publication correctly rejected the mismatched pair. The correction publishes against the frozen candidate source and records the newer validation source plus its authenticated UAT-only advance in result details. | UAT result publication only; all UAT checks, product source, and upstream gates remain valid | Pass the focused UAT/lifecycle/acceptance/link checks; commit cleanly; retry UAT publication/finalization only |
| 2026-08-10 | UAT 9 / result-reference recording | The canonical result content and its commit artifact were prepared; recording their references on the mutable attempt then stopped | same frozen candidate/environment; canonical-publication source `b10a1656...` | concentrated `evidence-finalization` validation-platform incident | Strict mode rejected direct assignment of the two new attempt properties after the state had advanced to `passed/result-committed`, leaving an exact partial commit with no canonical result published. The correction adds both fields explicitly and extends the append-only recovery path to archive only this exact partial shape, reuse the authenticated UAT checks, and rebuild publication. | UAT result publication only; all UAT checks, product source, and upstream gates remain valid | Pass the focused partial-commit/UAT/lifecycle/acceptance/link checks; commit cleanly; retry UAT finalization/publication only |
| 2026-08-11 | Readiness 52 / complete fail-late harvest | 12 checks passed, 2 failed, and 1 was dependency-blocked | corrected UI SDK source `d95e32e6` / `b5ea99f2`; environment identity not issued | `environment` and `harness` | The launch omitted the same-process `TEST_API_DATABASE_URL`, blocking final environment identity. Independent Playwright discovery also found the newly added lifecycle-title regression as test 84 while the durable manifest and Preflight traceability contract still declared 83. Raw evidence, harvest `a85b5592...`, consolidated two-defect batch `77ab46c7...`, and one-use successor authorization `2b6aa315...` are retained. | Readiness and all downstream phases | Add the exact test identity and 84-count to every canonical acceptance consumer, export six fresh database URLs in the successor process, pass both focused reproducers, commit cleanly, then consume the authorization in complete Readiness 53. |
| 2026-08-11 | Readiness 53 / complete gate | All 15 checks passed | `4436fb71` / `d1753210`; environment `878fa8fc...` | N/A — passed, superseded by the Rehearsal 41 correction | The complete readiness gate authenticated the clean source, six isolated databases, environment contract, exact 84-test inventory, implementation evidence, and validation-runner self-tests. | Rehearsal 41 only; the receipt remains valid history but cannot authorize the corrected source. | Correct the complete R41 batch, commit cleanly, then run a successor complete Readiness and Rehearsal against one new source/environment identity. |
| 2026-08-11 | Rehearsal 41 / complete conservative fail-late harvest | 28 lanes passed, 3 failed, 1 was dependency-blocked, and 0 were deferred; cleanup, restoration, and final health passed | `4436fb71` / `d1753210`; environment `878fa8fc...` | `product` as recorded; diagnosed as product, harness, and acceptance-regression roots | Workspace tests exposed the Component SSR renderer's missing native all-features executor initialization. Playwright exposed one external-font 404 root shared by two scenarios, a collapsed Dashboard visibility accessible name, the lost pre-extraction Components versions heading, and one stale `1.0.0` diagnostic assertion. Upgrade/rollback exposed the same stale `1.0.0` candidate expectation in its verifier. Harvest `bcb62626...`, consolidated batch `563e17fc...`, and correction authorization `7e23a4d7...` are retained; UAT diagnostics were blocked by the failed upgrade and Playwright prerequisites. | R41 and every downstream phase | Correct the batch together, regenerate source-exact Component and Dashboard browser assets/digests, pass focused reproducers for every failed lane, then run complete successor Readiness and Rehearsal. |
| 2026-08-11 | Readiness 54 / complete gate | All 15 checks passed | `92ac039f` / `f719ea71`; environment `25550ff1...` | N/A — passed, superseded by the Rehearsal 42 correction | The complete gate authenticated clean source, six isolated databases, the environment contract, exact 84-test inventory, implementation evidence, and runner self-tests. | Rehearsal 42 only; retained as immutable history | Correct the complete R42 batch, commit, then run successor complete Readiness and Rehearsal on one identity. |
| 2026-08-11 | Rehearsal 42 / complete fail-late harvest | 29 lanes passed, 2 failed, 1 was dependency-blocked, and 0 were deferred; cleanup, restoration, final health, source, and environment checks passed | `92ac039f` / `f719ea71`; environment `25550ff1...` | Raw lane defaults retained; diagnosed as `harness`, acceptance-regression, and test-fixture roots | Workspace linking stopped with Windows `LNK1318`; Playwright passed 82 of 84 scenarios, exposing one intentionally changed title baseline and one first-page pagination assumption. Harvest, batch `acd48e91...`, and one-use correction authorization are retained; UAT diagnostics were blocked only by failed Playwright. | R42 and all downstream phases | Correct together, pass clean focused reproducers, commit, then run successor complete Readiness and Rehearsal. |
| 2026-08-11 | Readiness 55 and Rehearsal 43 / complete gates | Readiness passed 15/15; Rehearsal passed all 32 lanes with 0 failures, blocks, or deferrals, including all 84 Playwright scenarios | `4ba03222` / `f0f44681`; environment `f8c66c6...` | N/A — both passed | Canonical Readiness SHA-256 `ada86945...`; canonical Rehearsal SHA-256 `7756b4c3...`. Source-exact materialization/no-op, failure recovery, workspace tests, browser acceptance, upgrade/rollback, restoration, health, and final identity all passed. | Preflight and downstream authorized for the same identity | Run Preflight against the authenticated receipts. |
| 2026-08-11 | Preflight 23 / complete fail-late harvest | 7 checks passed, 2 failed, and 1 was dependency-blocked; no candidate was frozen | corrected product source with passing Readiness 55/Rehearsal 43 | Raw: `product` and `harness`; consolidated diagnosis: 2 `harness` roots | The traceability check searched TypeScript source for literal title fragments and rejected valid loop-generated Playwright identities already authenticated by discovery. The path check had no safe archive transition for an earlier fully certified candidate invalidated by the later UI SDK correction. Attempt 23 and raw classifications remain immutable. | Preflight only; Readiness 55 and Rehearsal 43 remain valid because no product, fixture, deployment, acceptance inventory, or upstream runner input changes | Replace source-text matching with exact Playwright discovery; archive the authenticated prior Preflight→SIT→UAT chain with hashes and diagnostic-only status; pass focused self-tests and discovery; commit; rerun Preflight only. |
| 2026-08-11 | Preflight 24 / passing checks, failed final publication | All 10 checks passed; final candidate publication failed; no candidate was frozen | Preflight-only correction `7aa104c4`; Readiness 55/Rehearsal 43 unchanged | `evidence-finalization` | The new discovery and completed-candidate archive both passed, and the prior candidate was preserved in its authenticated immutable archive. Final lifecycle publication exposed a duplicate exact-four-path comparison that rejected the legitimate two-file correction subset. The terminal attempt and archive mapping remain immutable. | Preflight publication only; all checks and both upstream gates remain valid evidence | Apply the same bounded allowed-subset rule at lifecycle publication, pass lifecycle/Preflight self-tests, commit, then run a new Preflight attempt without repeating the already-completed archive transition. |

Classifications are exactly `preflight/setup`, `product`, `harness`,
`environment`, `flaky`, `evidence-finalization`, or `product-decision`.
Passed or user-interrupted rows have no defect classification. Product
decisions pause for user direction.

## Post-SIT Defect Convergence (Conditional)

- On the first candidate-invalidating UAT failure, retain the failure,
  invalidate the candidate, forbid passing `uat-result.json`, and finish only
  safe independent scenarios as non-authoritative diagnostics.
- Write `uat-defect-harvest.json`, consolidate `defect-batch.json`, restore the
  canonical fresh topology, and create
  `correction-impact-assessment.json`.
- Identity, authorization, protocol, schema, seed/bootstrap, shared fixture/
  environment and cross-module changes default to a broad cone.
- Focused repair attempts diagnose mutable corrections only. They never
  authorize a candidate, SIT, UAT or closeout.
- After convergence, record restoration and final-certification entry, then
  rerun complete readiness, rehearsal, preflight, SIT and UAT for a successor.

## Evidence Integrity

- Required files complete: Not Run.
- Structured artifacts parse: Not Run.
- Markdown links pass: Not Run.
- Authoritative/superseded attempts distinguished: Not Run.
- Manifest file count: Not Run.
- Manifest SHA-256: Not Run.

Superseded failures remain retained and explicitly excluded rather than
overwritten. Every phase result names and hashes its prerequisites. One
immutable fingerprint covers all authoritative SIT and UAT evidence.

## Closeout Authorization

- Status: Not Authorized.
- Authorization receipt:
  `artifacts/sprint-8a-closeout/closeout-authorization.json` (not created).
- Authorized candidate/fingerprint: None.
- SIT passed: No.
- UAT passed: No.
- Acceptance mapping complete: Planned, not executed.
- Invalidation decisions satisfied: No — R32 is terminal failed with two raw
  `product` findings, eleven blocks, and no passing result. Its append-only
  restoration qualifies correction only; the tracked product/harness/evidence
  cone remains formally unverified. Complete Readiness 42 and complete
  Candidate Rehearsal 33 remain required after the clean commit and fresh
  database generations.
- Unresolved product decisions: None.
- Intended active route/slot: source-exact Sprint 8A gateway with current
  Component release and canonical fresh seed.
- Application health: source-exact canonical restoration after R32 is retained
  diagnostically, but not proven by a passing corrected-source rehearsal.
- Evidence source commit: Not frozen.
- Documentation commit: Not created.
- Authorization timestamp: None.

Closeout is forbidden until the validation coordinator verifies the complete
hashed chain and writes authorization. This record implies no authoritative
frozen-candidate deployment, SIT, formal-UAT, or acceptance result.

## Rehearsal 33 consolidated correction and final-certification entry

The later R32 correction history is append-only. Readiness 43 passed all 15
checks against `2cdbf5432f9a737e82ea470ebabe7b55e42cd4ec` / tree
`43c3b45c96b17b1fc3da9dabedcc36976ea97f8c` and environment fingerprint
`5576717182719a9e6f53ba0ffe9ecdaad4db230f8b62336b3bc26197a1e298a3`.
Candidate Rehearsal 33 then terminalized all 32 declared lanes with 29 passed,
2 failed, 1 blocked, and 0 deferred. Its retained attempt, harvest, and single
defect batch remain unchanged. No Candidate Rehearsal result, preflight,
candidate freeze, SIT, or UAT was authorized.

The R33 correction is one implementation batch. Full Candidate Rehearsal is
final certification, not its debugging loop. The batch maps implementation to
the governing Sprint 8A validation clauses as follows:

| Governing clause | Correction | Focused non-authoritative proof |
| --- | --- | --- |
| Specifications 1 and 6; AC-01, AC-11, AC-13; exact composed navigation | Core derives the shell key for each real manifest contribution from its same-origin top-level route while retaining the exact contribution identity. Components and Dashboard each project once as `components` / `tessara.components.navigation` and `dashboards` / `tessara.dashboards.navigation`; the five frozen transitions remain unchanged. | Exact catalog unit tests, live navigation identity probe, root/Dataset/Workflow/module-configuration browser reproducers. |
| Specifications 1, 3, and 6; AC-04, AC-05, AC-16; generic signed module routing | The generic module gateway deterministically selects the most-specific matching manifest route. Static Component routes such as `datasets` and `validate` therefore cannot be captured by `{component_id}` and receive their exact declared signed action grant. | Route-specificity unit test, live `GET /api/admin/components/datasets` 200 probe, Component and permissions browser reproducers. |
| Specifications 4 and 5; AC-07, AC-08, AC-09; canonical fresh seed | Analytics expects the current 30-row Sprint 8A primary Dataset result. Dashboard SSR preserves nondisclosure when an invalid cross-scope Component binding is hidden from both actor projections. | Exact analytics render and JavaScript-disabled Dashboard SSR reproducers. |
| Verification and UAT plan; durable exact test identities | Module diagnostics assert `Readiness`, `Liveness`, `Module database`, and `Core authorization`, not a copied count or retired class. Component tests bind exact roles, Component Version render paths, and menu semantics. | Nine-test R33 regression set; concentrated Component reproducer passes independently before broader verification. |
| Validation/evidence plan; exact final restoration and fail-late harvest | Final restoration compares the materializer's canonical source fields instead of requiring the richer Rehearsal source shape. Harvest accepts only the exact pre-authentication placeholder on `attempt-state-prerequisite`; all authenticated lanes remain source/environment exact. The retained R32 qualification regression authenticates immutable Readiness 41 in memory rather than consulting the later canonical alias. | Readiness, Candidate Rehearsal, and harvest adversarial self-tests, including dirty restoration-source rejection and post-authentication mismatch rejection. |

The repository-local implementation skill already enforces this mapping and
requires behavior changes to ship with their affected materialization, no-op,
recovery, fixture, runner, smoke, acceptance, and evidence consumers. It also
requires clean disposable materialization, exact no-op, focused recovery, all
known focused reproducers, and the two-/three-failure lane policy before
implementation handoff. No duplicate skill rule or unrelated validation
protocol/schema/scheduler/lineage enhancement is added in this batch.

Focused evidence is retained under
`artifacts/sprint-8a-focused-r32-correction/`. It is explicitly diagnostic and
non-authoritative. The clean source command is:

```powershell
.\scripts\materialize-sprint-8a.ps1 -Attempt 3201 -EvidenceRoot artifacts/sprint-8a-focused-r32-correction -EnvironmentFingerprint 5576717182719a9e6f53ba0ffe9ecdaad4db230f8b62336b3bc26197a1e298a3 -AuthorizeDisposableReset -VerifyNoOp -Confirm:$false
```

The same focused boundary must retain induced-failure teardown, from-empty
successor materialization, exact no-op, canonical restoration, exact Core and
Supervisor health, the complete known-regression set, and validation-runner
self-tests. Those receipts do not replace complete Readiness or complete
Candidate Rehearsal against one later clean source/environment identity.

### Rehearsal 33 terminal-tail recovery boundary

Coordinator recovery authenticated R33's immutable start, terminal attempt,
32 lane receipts, harvest, and consolidated defect batch without rewriting
them. Recovery now treats the immutable start declarations as the historical
attempt contract rather than comparing them with a later corrected runner, and
it consumes an existing append-only harvest/batch instead of reconstructing
their timestamps. The exact immutable Readiness prerequisite is
`attempts/readiness-43.json`; the mutable canonical alias is not a valid
terminal-recovery substitute.

The validation state records R33 as failed with 29 passed, 2 failed, 1 blocked,
0 deferred, and `harvest_guard = passed`. The immutable attempt truthfully
retains `cleanup_restoration.result = not_proven`. On 2026-08-08 the user
approved one separate append-only record for this exact comparator mistake.
The repository-owned finalizer authenticates the immutable attempt, harvest,
batch, failed cleanup lane, successful source-exact apply/no-op, exact Core and
Supervisor health, five-transition/one-Dashboard inventory, passing final
environment lane, and clean committed correction source. It may issue and
qualify exactly one authorization for Readiness 44 without changing R33 or
making it pass. The qualification cannot authorize Rehearsal success,
preflight, candidate freeze, SIT, UAT, or closeout. No other rehearsal receives
this exception.

After the correction commit, the coordinator records the approved bridge with:

```powershell
.\scripts\run-sprint-8a-candidate-rehearsal.ps1 -Attempt 33 -AuthorizeApprovedR33Correction
```

Only then may complete Readiness 44 start. If it passes, complete Rehearsal 34
uses its immutable `attempts/readiness-44.json` receipt.

### Readiness 44 consolidated correction boundary

Readiness 44 consumed the approved R33 correction record and terminalized all
15 declared checks. It retained 13 passes, two `harness` failures, no blocked
checks, one complete harvest, and one two-defect batch. The failed
`compose-and-fixture-contract` and `runner-self-tests` checks share one root
cause: the preflight check-list reader treated the rehearsal's resume-time
`$declaredChecks = $historicalDeclaredChecks` assignment as another canonical
declaration.

The correction narrows the parsed identity to the one literal array
declaration and deliberately keeps the resume assignment present in the
self-test fixture. The readiness self-test also authenticates R33 in both its
pending and consumed positions, rather than assuming it remains the newest
correction record. Focused non-authoritative acceptance, readiness, rehearsal,
preflight, smoke, and automated UAT reproducers pass. They do not change
Readiness 44 or replace a formal gate. The next permitted full gate is
Readiness 45 against the clean correction commit and six fresh databases. Only
a complete pass may authorize Candidate Rehearsal 34.

### Rehearsal 35 correction boundary

Readiness 46 passed all 15 checks for commit
`bcd685d5c1ef2ecca5518eb1ea14f0790aa57541` and environment
`4654e4dbc58cc31c471dfe826b751e39a7b826fb94826d8579c7fb266cb8e953`.
Rehearsal 35 then terminalized all 32 lanes: 27 passed, three retained
controlling-process-loss failures, one Playwright failure, one UAT diagnostic
block, and no deferrals. Canonical restoration passed. The original attempt,
harvest, consolidated batch, lane receipts, logs, traces, and timestamps remain
immutable.

The correction maps to the governing clauses as follows:

| Governing clause | Correction | Focused non-authoritative proof |
| --- | --- | --- |
| Validation protocol process-loss recovery and complete harvest | Restore JSON declarations to the canonical dictionary shape and retain offset-qualified orphan start boundaries. For R35's already-retained locale strings, accept only the exact hashed executing captures with matching lane and assertion boundaries. | Candidate Rehearsal and harvest adversarial self-tests; R35 harvest authentication and append-only correction authorization. |
| AC-04/AC-05 Component ownership and dependency recovery | Canonicalize Dataset reference JSON recursively before using it as the native picker identity, preserving the exact selected major line across metadata retry. | Focused outage/retry/save Playwright scenario and Component package tests. |
| Exact acceptance identities and module-owned errors | Bind structured-editor controls to stable `data-config-control` identities; permit only the deliberate invalid-save HTTP 400 in the console guard; assert the complete `component.forbidden` envelope and the nondisclosing `component.not_found` publish response. | Focused visual-authoring and scoped-permission Playwright scenarios. |
| Save Draft product path | Remove the native `required` constraint from the hidden closed publish-note dialog while preserving the dialog's explicit new-version note validation. | Component document contract tests and the full focused visual-authoring browser scenario. |
| Clean candidate entry | Re-run source-exact disposable materialization, exact no-op, induced-failure containment, canonical recovery, Core/Supervisor health, acceptance, and runner self-tests against the final correction commit. | Diagnostic successors under `artifacts/sprint-8a-focused-r35-correction/`; these receipts are not formal gate evidence. |

No focused check makes R35 pass. A later complete Readiness and complete
Candidate Rehearsal must run cleanly against one corrected committed source and
environment before preflight, candidate freeze, SIT, or UAT.

### Rehearsal 36 correction boundary

Readiness 47 passed all 15 checks. Rehearsal 36 terminalized all 32 lanes with
30 passes, one failed browser lane, one dependent blocked UAT diagnostic lane,
and no deferrals; its cleanup and final identity checks passed. The retained
failure showed that the historical Component-table permission scenario expected
Core's retired generic `not_found` body even though the independently deployed
Component module correctly returned its canonical nondisclosing
`component.not_found` envelope.

The correction maps to the exact acceptance-identity and module-owned-error
clause: the scenario now checks the full `component.not_found` envelope through
the common Component response helper. The exact historical-version scenario,
acceptance inventory/manifest contract, Candidate Rehearsal and harvest-guard
self-tests, and Markdown-link check pass as focused proof. These checks are
diagnostic only. A new complete Readiness and Candidate Rehearsal against one
corrected committed source and environment remain mandatory before preflight or
later phases.

### Rehearsal 37 correction boundary

Readiness 48 passed all 15 checks. Rehearsal 37 terminalized all 32 lanes with
30 passes, one failed Playwright lane, one dependent blocked UAT diagnostic
lane, and no deferrals. Exact materialization/no-op, failure containment,
upgrade/rollback, restored deployment evidence and smoke, final health,
clean-source, and environment identity all passed.

The retained trace proves `/components` returned the correct independently
owned native document and `.components-page` content. The acceptance scenario
still looked for Core's generic `.route-panel`, so the defect is a stale
acceptance identity rather than a product rendering failure. The six Component
route declarations now name `.components-page` explicitly; Dashboard route
declarations continue to use their module-owned `.route-panel` markup.

The isolated scenario also now owns its draft-visibility fixture. It creates a
draft Component from the established in-scope Dataset reference, verifies that
exact identity in the manager directory, and deletes the draft version in a
`finally` boundary. It no longer relies on state left by an earlier serial
permission test. Responsive directory assertions bind to that Component ID and
the visible desktop/mobile projection rather than an ambiguous first text node.

R36 and R37 are consecutive failures of the Playwright lane. The exact scenario
and complete 75-scenario browser lane must therefore pass cleanly as focused,
non-authoritative proof before another full lifecycle launch. A later complete
Readiness and Candidate Rehearsal remain mandatory for preflight eligibility.

### Rehearsal 38 result and correction boundary

Rehearsal 38 completed fail-late collection with 31 passing lanes, one failed
lane, no blocked lanes, and no deferred lanes. Source-exact materialization,
the exact no-op path, induced-failure recovery, workspace tests, Component
nondisclosure, product smoke, all 75 Playwright scenarios, Component
upgrade/rollback, final canonical restoration, Core and Supervisor health,
and final source/environment identity all passed.

The only failure was `uat-diagnostics`. Its reader required the retired
schema-v1 prerequisite receipt even though the rehearsal scheduler correctly
published schema-v2 receipts bound with `identity_binding: attempt_identity`.
The correction aligns the UAT reader and its adversarial self-test with that
current contract while rejecting legacy, coerced, and pre-authentication
receipts. This is focused, non-authoritative correction evidence and does not
replace the next complete Validation Readiness and Candidate Rehearsal.

### Rehearsal 39 result and correction boundary

Readiness 50 passed all 15 checks for commit
`af5981a27d7445413a62f73b7102877bceeedc2b`. Rehearsal 39 terminalized every
lane with no blocked or deferred lanes. The source-exact materialization and
no-op, induced-failure recovery, workspace tests, module conformance, product
smoke, all 75 browser scenarios, upgrade/rollback, canonical restoration,
health contracts, and final identities passed. The UAT diagnostic projector
alone produced 13 nested failures, so no passing rehearsal result or preflight
authority was issued.

The retained batch's raw `product` classifications remain immutable. The
corrected assessment is a single `harness` batch with three root causes:

- leaf Playwright suites legitimately omit child `suites` and sometimes
  `specs`, while the top-level report must still publish `suites`;
- materialization now contains the exact ordered owners `core`,
  `tessara.components`, `tessara.dashboards`, and
  `tessara.reference.scoped-records`, and its gateway boundary uses exact
  offline and ready health observations;
- failure-containment successor applies publish `{ operation, receipt }`, with
  bootstrap results under `receipt.bootstrap_receipts` and no-op lineage bound
  to the first operation receipt digest.

The corrected projector validates those exact identities and states. Its
self-test uses the retained Rehearsal 39 files, when available, to prove all 75
Playwright results and every affected materialization and recovery predicate.
Because the UAT diagnostic lane failed in two consecutive rehearsals, this
focused reproducer must pass on the clean correction commit before the next
formal launch. It remains non-authoritative and cannot substitute for complete
Readiness and Rehearsal results from the same corrected source and environment.

### Sprint 8A Module UI SDK correction boundary

Closeout inspection found a product architecture defect against AC-07, AC-15,
UAT-8A-01, and the Independent Module Pathway: Components used handwritten
structural HTML, DOM construction, and an independent stylesheet instead of
the canonical Module UI SDK. Dashboard still carried a copied shared-style
snapshot, Core did not import the same canonical canvas rules, and lifecycle
navigation displayed the generic title `Module` instead of the active manifest
navigation label. The visible result was inconsistent Components styling, a
Core canvas color that differed from Dashboard, and a Dashboard top title that
did not match its navigation entry. This is classified as one consolidated
`product` defect with architecture, visual-continuity, hydration, asset-
ownership, and lifecycle-title subtypes. All earlier Sprint 8A candidate
evidence is superseded for certification; retained receipts remain immutable
diagnostic history.

The correction establishes one canonical presentation path:

- `tessara-module-ui` `0.3.0`, design-system asset ABI `2.0.0`, and conformance
  suite `1.2.0` own resets, themes, canvas, shell, and shared primitives;
- Core imports that exact CSS source; direct module documents load it plus
  namespace-rooted product CSS, while lifecycle navigation loads product CSS
  only;
- Components, Dashboard, Scoped Records, and the SDK reference module render
  typed Leptos views through the SDK document renderer; no deployed module
  injects raw structural HTML;
- Components and Dashboard use the shared lifecycle adapter, and Core derives
  the active navigation item and top title from the authenticated current
  route; and
- corrected immutable releases are Components `1.0.1`, Dashboard `3.0.1`,
  Scoped Records `1.0.1`, and SDK reference `1.0.1`.

The new `ui-sdk-conformance` implementation proof enumerates every first-party
manifest, authenticates the exact SDK/design/conformance tuple and CSS digest,
rejects raw HTML/DOM construction, copied/generic product styles and product
token declarations, and verifies canonical primitive styling and canvas
ownership. Focused visual parity must additionally prove direct-load versus
lifecycle navigation title, active navigation, computed tokens, background,
typography, controls, tables/dialogs, responsive behavior, accessibility,
hydration, and clean console output. Formal Readiness, Candidate Rehearsal,
Preflight, SIT, and UAT remain required before closeout. Readiness 53 passed for
the first corrected identity, but complete Rehearsal 41 invalidated it with the
consolidated correction batch recorded in the runtime history above.

The first focused browser parity run also exposed that Core's signed module
shell contexts still forced `dark` even when Core itself resolved the user's
`system` or stored theme preference. That stale host policy made direct module
loads use Dashboard's dark canvas while Core used the light canvas. The module
gateway and the Core-owned direct-document contexts now declare `system`, so
the SDK's one theme bootstrap resolves the same stored or operating-system
preference for Core, Components, Dashboard, and Scoped Records. The parity
reproducer records the resolved preference, computed tokens, and body canvas
for every route; this focused evidence remains non-authoritative.

A subsequent wide-desktop lifecycle review found Core deriving the activated
module title from the complete navigation link text, which included hidden icon
metadata, instead of the displayed navigation label. Core now selects the
longest matching route but reads its title exclusively from the visible
`.sidebar-link__label`, and active-state identity is matched by the exact route.
The focused browser reproducer exercises in-app Dashboard and Components
navigation at 1594 px and asserts the exact displayed labels. The same review
removed the redundant Components directory eyebrow while retaining the page
title and accepted descriptive copy.

Rehearsal 41 then closed five gaps in the implementation-to-validation map:

- AC-07 and the SDK architecture contract require typed SSR to work under the
  workspace's all-features test profile, so Component document rendering now
  initializes the same native test executor as Dashboard;
- AC-15 and UI SDK conformance require repository-owned presentation with a
  clean browser console, so Core no longer loads its heading font from an
  unauthenticated external network dependency;
- UAT-8A-01 and the accepted pre-extraction baseline require the Component
  versions route to retain the exact `<Component name> versions` heading and
  the Dashboard visibility disclosure to expose the spaced accessible name
  `Visibility <count> Node(s)`; and
- the immutable Components `1.0.1` identity is now asserted consistently by
  Module Management acceptance and the `0.9.0` upgrade/rollback verifier. The
  verifier self-test authenticates that expectation directly against the
  current Component Manifest so another stale candidate literal fails before
  formal rehearsal; and
- the Readiness lineage self-test isolates its authenticated historical R33
  prefix before testing one-use consumption. Later valid correction links can
  no longer make that historical negative test inspect the wrong active tip.

### Rehearsal 42 result and correction boundary

Readiness 54 passed all 15 checks for commit
`92ac039f1cb3c0fe5753b1f86e312d995b09d546`. Rehearsal 42 then completed its
fail-late harvest with 29 passing lanes, two failed lanes, one dependent
blocked UAT-diagnostics lane, and no deferrals. Source-exact materialization,
the semantic no-op, failure recovery, module boundaries, product smoke,
upgrade/rollback, canonical restoration, Core and Supervisor health, and final
source/environment identity all passed. The retained harvest and consolidated
batch remain immutable diagnostic history; no rehearsal result or preflight
authority was issued.

The consolidated correction addresses four exact roots:

- the Windows workspace-test lane now uses `--jobs 1` in Candidate Rehearsal
  and SIT, preventing concurrent MSVC PDB writes that produced `LNK1318`; the
  command inventory and runner self-tests require that exact stable form;
- the Component versions visual baseline now reflects the accepted exact
  `<Component name> versions` route title introduced by the Rehearsal 41
  correction;
- the no-JavaScript ownership scenario keeps its exact scenario-owned draft
  Component on the first alphabetically sorted ten-row SSR page, preserving
  rather than bypassing the product's pagination contract; and
- the module-contract valid-manifest test builder and byte-pinned fixture now
  use the exact current `0.3.0 / 2.0.0 / 1.2.0` platform tuple instead of the
  superseded SDK tuple.

Focused, non-authoritative verification passed on fresh disposable databases:
the complete serialized Rust workspace including integration and doc tests;
the module-contract crate and canonical fixture digest; both failed Playwright
scenarios together without snapshot-update mode; formatting, workspace check,
zero-warning Clippy, the acceptance contract, Module SDK boundaries; and the
Readiness, Candidate Rehearsal, Preflight, SIT, and Playwright runner self-tests.
These focused results prove correction readiness but do not replace formal
certification. A successor complete Readiness and Candidate Rehearsal must pass
against one new committed source/environment identity before Preflight, freeze,
SIT, or UAT may start.

### Readiness 55, Rehearsal 43, and Preflight 23 boundary

Readiness 55 passed all 15 checks and Candidate Rehearsal 43 passed all 32
lanes against commit `4ba03222b1aad2e1513421f6f5eb5d35fb31614f`, tree
`f0f4468151b5b3197c8b08b41b032386d967f04b`, and environment
`f8c66c6c8f9a47a5cf134cbff4327086f0a58feba8987cab2b1e57dbefcbe0b5`.
Rehearsal included source-exact materialization and no-op, failure recovery,
the complete serialized Rust workspace, all 84 browser scenarios, Component
upgrade/rollback, canonical restoration, Core and Supervisor health, and final
source/environment authentication. Their canonical SHA-256 values are
`ada86945f4d2027606eb798bb6e30a67386ac8b79f0a4dbedfb29db39324d1c2`
and `7756b4c3761ee887da52c9db8139ff166fb86ed3f4dfe5980ea3a0451b2a4580`.

Preflight 23 then completed fail-late with seven passes, two failures, and one
exact dependency block before candidate freeze. Its raw evidence and recorded
classifications remain unchanged. Consolidated diagnosis found two Preflight-
only harness roots: traceability incorrectly searched generated TypeScript
test source for literal titles instead of authenticating Playwright discovery,
and the evidence-path guard supported archiving a failed-SIT freeze but not an
earlier fully certified candidate invalidated by the later product correction.

The correction reuses `validate-e2e.ps1 -InventoryOnly` so the durable 84-test
manifest is compared with Playwright's real discovered identities. It also
authenticates the prior Preflight, candidate, manifest, SIT, UAT, hashes,
fingerprint, and prerequisite links before moving their canonical aliases and
sidecars into one immutable diagnostic-history archive. An adversarial runner
self-test proves the archive transition, and retained focused discovery proves
all 84 exact identities. These changes affect only Preflight validation and
evidence handling; Readiness 55 and Rehearsal 43 remain authoritative and need
not be rerun. Focused proof does not replace the required successor Preflight,
SIT, or UAT.

Preflight 24 subsequently passed all ten declared checks. Exact Playwright
discovery and the authenticated archive transition both succeeded, preserving
the earlier completed candidate as immutable diagnostic history. Candidate
publication then failed before freeze because the shared lifecycle publisher
still duplicated the older exact-four-file comparison. Its terminal attempt
records one `evidence-finalization` failure and no blocked checks. The focused
correction makes that publisher accept any non-empty subset of the same four
Preflight-only paths, requires the Preflight runner itself to be present, and
continues rejecting product, fixture, deployment, acceptance, and other
validation changes. Lifecycle and Preflight adversarial self-tests cover the
allowed subset, empty set, and outside-path cases. This remains a Preflight-
only correction; Readiness 55 and Rehearsal 43 remain valid.

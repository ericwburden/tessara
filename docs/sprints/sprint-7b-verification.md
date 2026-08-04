# Sprint 7B Verification Record

Status: implementation, Test Readiness, and complete Candidate Rehearsal have
passing receipts for the current clean implementation commit; preflight, SIT,
formal UAT, and closeout remain `Not Run`.

Plan authority: `docs/sprints/sprint-7b-plan.md`

## Roadmap Acceptance Inventory

| Requirement | Failure condition | Automated/integration evidence | Deployed/manual evidence | Status |
|---|---|---|---|---|
| R1 typed resolution and revision/state-change contracts | Ambiguous, malformed, mixed-version, or policy-bearing platform observation | Module/Components contract unit, golden, invalid-fixture, serde, conformance | Exact typed Component/Dataset exchanges; UAT-01/03/06 | Planned |
| R2 provider-owned lifecycle/versioning | Consumer invents provider state or current published update is rejected/silent | Component state-machine, in-place update, publication, audit, ownership tests | UAT-01/03/04/06 | Planned |
| R3 declared observation mechanism | Change is missed or event delivery is assumed | Strategy declaration and monotonic revision tests; editor/live-read integration | Real Dashboard refresh; UAT-01/03 | Planned |
| R4 changelog/stale/carry-forward/rebinding | Silent repoint, lost finding, or partial/stale action | Provider change, Dashboard DB/API/action, Dataset adapter, replay tests | UAT-02–05/08 | Planned |
| R5 stable owner/type across mutable state | Lifecycle or semantic update changes typed identity | Reference digest and before/after persistence tests | UAT-01/03/06 | Planned |
| R6 provider-owned guards only | Core publication depends on Dashboard layout policy | Dependency-direction and publication regression tests | Publish/update and observe Dashboard-owned finding; UAT-03/04 | Planned |
| R7 outcome matrix and nondisclosure | State dimensions collapse or restricted data leaks | Full resolution/render/action matrix; known/random API/UI/log/timing tests | UAT-06/07 | Planned |
| R8 contract-version regression | V1 runtime fallback survives or V2 mismatch is accepted | Immutable V1 fixtures; exact V2 and invalid-version tests | Captured real contract exchange; UAT-06 | Planned |
| R9 dependency health and observed-state UI | Dedicated route, missing editor states, viewer writes, or unapproved visual drift | SSR/wasm/accessibility/Playwright/no-write and approved-mockup comparisons | Editor/viewer smoke; UAT-01/02/07/09 | Planned |
| R10 upgrade/carry-forward/rebinding UI | Wrong target semantics, action bypass, or mismatch from approved action UI | Tagged UI/API actions, authorization, target, conflict, and visual tests | UAT-04/05/08/09 | Planned |
| R11 resolution/deferral ownership | Manual resolve exists, deferral improves health, or Core owns disposition | API inventory, persistence, health, later-revision, ownership tests | UAT-02/05 | Planned |
| Exit: Component change crosses real Dashboard boundary | In-process shortcut or stale/automatic mutation | Source-exact process-boundary smoke and Playwright | UAT-01–07 | Planned |
| Exit: equivalent Dataset/Component adapters | One-off semantics or cross-module DB read | Dataset/Component conformance and dependency audit | UAT-08 | Planned |

## Acceptance-Criteria Index

| Criterion | Observable result | Required evidence | Status |
|---|---|---|---|
| AC-01 | Exact valid V2 observation round-trip; malformed/mixed versions fail closed | contract fixtures and conformance | Planned |
| AC-02 | Restricted known/random API, UI, logs, and bounded timing are indistinguishable | nondisclosure matrix and UAT-07 | Planned |
| AC-03 | Lifecycle/semantic changes advance revision with stable owner/type/ID | provider/consumer integration and UAT-01/03 | Planned |
| AC-04 | Current published version updates in place and Dashboard observes it without reference change | update audit, exchange, finding, UAT-03 | Planned |
| AC-05 | Successor leaves old reference pinned; Upgrade is same-Component declared successor only | publication/action tests and UAT-04 | Planned |
| AC-06 | Replace permits any authorized renderable target; stale/invalid/replay is atomic and safe | API/DB fault and UAT-05 | Planned |
| AC-07 | Any disclosed finding defers without note, stays degraded, and later revision opens a new finding | persistence/health tests and UAT-02 | Planned |
| AC-08 | No manual resolve; fresh health or successful action closes atomically | API inventory/transaction tests and UAT-02/04/05 | Planned |
| AC-09 | Full lifecycle, render/metadata/tombstone/audit/immutability matrix holds; drafts have no actions | provider matrix and UAT-01/06 | Planned |
| AC-10 | Dashboard owns layout findings and Core publication has no Dashboard-policy dependency | ownership tests and UAT-03/04 | Planned |
| AC-11 | Editor load/retry is idempotent, viewer is read-only, and no scheduler/event path exists | integration/dependency tests and UAT-01/07 | Planned |
| AC-12 | Dataset/Component adapters share observation/nondisclosure semantics without cross-DB reads | conformance and UAT-08 | Planned |
| AC-13 | V2 is sole runtime version; accepted V1 fixture files remain byte-identical | fixture hashes/runtime inventory and UAT-06 | Planned |
| AC-14 | Fresh and unchanged source-exact runs are warning-free, healthy, deterministic, and evidenced | readiness/rehearsal/SIT/UAT receipts | Planned |
| AC-15 | Production affected UI matches the approved mockup at matching route/state/theme/viewport/density/content/role; Core and Dashboard use identical SDK-owned chrome and responsive behavior; parallel local shells and prototype controls are absent | SDK-boundary audit, visual regression evidence, and UAT-09 | Planned |

## Evidence Inventory And Retention

The canonical evidence root will be created by the specialized validation
workflow. It must retain:

- readiness, rehearsal, preflight, SIT, UAT, and closeout receipts;
- candidate source fingerprint and exact image, contract, schema, fixture, and
  tool identities;
- approved UI package/approval record hashes, deployed-baseline captures,
  production captures, and same-state comparison evidence;
- raw command stdout/stderr and exit codes;
- contract requests/responses, HTTP captures, database read-back, structured
  logs, screenshots, browser traces, and action/audit receipts;
- old/new typed-reference digests and prior/current resource revisions;
- evidence manifest hashes, invalidation history, and recovery/rollback receipts.

Raw evidence is append-only. A failed attempt remains retained and linked to any
correction batch and successor candidate.

## Candidate Identity Contract

- Candidate fingerprint inputs include product source, contracts, tests,
  harnesses, squashed baselines, fixtures, manifests, Compose/bootstrap scripts,
  lockfiles, and validation documentation.
- Core and Dashboard schema identity must name their updated fresh squashed
  baselines. No additive Sprint 7B migration or populated-upgrade claim is valid.
- Runtime Components contract identity must be exact V2. V1 fixture hashes are
  recorded as historical-test integrity evidence only.
- Mutable-source images must carry the source fingerprint; all formal receipts
  must report the same identity.
- Any change to a fingerprint input invalidates readiness/rehearsal/preflight,
  the frozen candidate, and all downstream SIT/UAT evidence.
- The approved Sprint 7B UI package and product-owner approval record are
  fingerprint inputs for every affected production UI candidate.

## Validation Readiness Checklist

| Check | Passing condition | Status | Receipt |
|---|---|---|---|
| Plan completeness | Every roadmap row maps to AC, slice, automated, deployed, and manual proof | Pass | `artifacts/sprint-7b-closeout/validation-readiness-result.json` |
| Decision completeness | Lifecycle, update, tombstone, refresh, action, UI, ownership, and baseline decisions match the approved plan | Pass | `artifacts/sprint-7b-closeout/validation-readiness-result.json` |
| UI approval | Product owner explicitly approved the retained interactive mockup and bounded screen deltas; production UI has not preceded approval | Pass | `sprint-7b-ui-review/approval.md`; approved 2026-08-04 before product UI implementation |
| Test integrity | Existing accepted tests are not weakened/deleted; V1 fixtures remain byte-identical; V2 fixtures are additive | Pass | readiness attempt 21; complete 70-test inventory |
| Warning policy | Exact fmt/check/clippy/test commands defined with `-D warnings` | Pass | readiness attempt 21; zero-warning Clippy |
| Environment | Required ports, Docker, Rust, Node, browser, DB, disk, and evidence paths available | Pass | `readiness12_*`; reference composition on 8086/8096 |
| Evidence schema | Attempt receipts, raw artifacts, hashes, timestamps, and invalidation links validate | Pass | readiness `be336c35…`; rehearsal `331d204a…` |
| Recovery | Same-candidate recovery and prior-composition rollback procedures are executable | Pass | `artifacts/sprint-7b-closeout/rehearsal/recovery.json` |

Readiness is passing only when every row is evidenced. A waiver is not a pass.

## Candidate Rehearsal Checklist

| Lane | Command or action | Passing condition | Status | Receipt |
|---|---|---|---|---|
| Format | `cargo fmt --all -- --check` | exit 0; no diff | Pass | readiness attempt 21 |
| Check | `cargo check --workspace --all-features --locked` | exit 0; zero warnings | Pass | readiness attempt 21 |
| Clippy | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | exit 0; zero warnings | Pass | readiness attempt 21 |
| Rust | `cargo test --workspace --locked` | all tests pass | Pass | readiness attempt 21 |
| Browser | `npm --prefix .\end2end test` | all tests pass | Pass | 70 passed; rehearsal Playwright summary |
| Contract/conformance | Sprint 7B targeted suites | full version/state/action/nondisclosure matrix passes | Pass | SDK, analytics, and nondisclosure rehearsal evidence |
| Fresh materialization | source-exact Sprint 7B bootstrap | exact candidate healthy with expected schema/contracts/fixtures | Pass | deployment evidence `441440f4…` |
| Idempotent rerun | unchanged source-exact rerun | semantic no-op; no duplicates/revision drift | Pass | revision 2; `no_op=true`; zero changed owners |
| Deployed smoke | general plus Sprint 7B smoke | real process boundary and final health pass | Pass | general, Sprint 7B, and post-recovery smoke evidence |
| Supply/docs | `cargo audit`, `validate-e2e`, Markdown links, clean diff | no blocking finding or documentation drift | Pass | five policy-allowed warnings; current clean implementation commit |

The developer-only `scripts\local-launch.ps1` cannot substitute for formal
source-exact materialization evidence.

## Preflight And Freeze Checklist

- Audit passing readiness and rehearsal receipts and their raw artifacts.
- Recompute the candidate fingerprint and compare every image, contract, schema,
  fixture, and source identity.
- Prove the environment is clean and the source-exact deployment is the only
  topology under test.
- Freeze the candidate. Product source, tests, fixtures, schemas, manifests, and
  harnesses become immutable through SIT and UAT.
- Emit explicit authorization for SIT or fail closed. UAT authorization can be
  emitted only after all SIT lanes pass on this same fingerprint.

## Changed Integration Contracts

| Boundary | Approved change | Required negative proof |
|---|---|---|
| Module resource resolution | Add reference/contract/strategy/resource-revision observation primitives only | No finding, disposition, Upgrade, Replace, or Remove product policy in platform types |
| Core Components V2 | Separate publication/lifecycle; revision/change/successor/authorized metadata | Wrong version, forbidden transition, stale prior revision, restricted metadata, invalid successor fail safely |
| Dashboard dependency API | Read/live resolution, manager refresh, and tagged defer/upgrade/replace/remove commands | Viewer does not write; no manual resolve; stale/replay/unauthorized/non-renderable target does not partially mutate |
| Approved UI contract | Deployed-baseline-preserving Dashboard editor and Component Versions deltas | Same-state/theme/viewport captures match approval; prototype navigator/scenario controls absent |
| Dashboard persistence | Observations, findings, idempotent action receipts in fresh squashed baseline | Unchanged refresh/rerun creates no duplicate; later provider revision is distinct |
| Dataset transition adapter | Current DatasetRevision identity projected into typed observation semantics | No new public Dataset boundary or cross-module DB access |
| Bootstrap/acceptance | Exact Sprint 7B profile and semantic fixtures | Old binaries/new baseline and additive migration are not claimed or exercised |

## SIT Execution Matrix

| Lane | Required proof | Status | Attempt receipt |
|---|---|---|---|
| Rust workspace | Full and targeted unit/integration/migration/provider/consumer suites | Not Run | |
| Browser workspace | Full Playwright plus editor, viewer, Versions, accessibility, approved visual contract, and no-dedicated-route/prototype-control assertions | Not Run | |
| Contracts | V1 fixture integrity, exact V2, malformed/mixed version, conformance | Not Run | |
| Fresh deployment | Exact source/schema/image/fixture identity and healthy topology | Not Run | |
| Idempotent deployment | Unchanged no-op and no duplicate observation/finding/receipt | Not Run | |
| Deployed acceptance | Real Dashboard-to-Core boundary, actions, adapter equivalence, outage/recovery | Not Run | |
| Nondisclosure | Known/random API/UI/log/timing equivalence across resolution/action matrix | Not Run | |
| Recovery/rollback | Same-candidate recovery and prior-composition/snapshot restoration audit | Not Run | |
| Final convergence | All services healthy; provenance/evidence hashes complete; source clean | Not Run | |

Every attempt records command, environment, start/end time, exit code, expected
and actual result, raw artifact paths/hashes, candidate fingerprint, and failure
classification. A rerun never overwrites a failed receipt.

## UAT Execution Matrix

| Scenario | Roles and preconditions | Actions | Expected result | Status | Evidence |
|---|---|---|---|---|---|
| UAT-01 lifecycle observation | Component manager and scoped Dashboard editor; active referenced version | Capture reference/revision, deactivate, load editor, reactivate, reload | Identity stable; revisions/findings advance; inactive is not renderable; audit exists; fresh health closes finding | Not Run | `uat/manual/uat-7b-01.json` plus screenshots |
| UAT-02 deferral/recovery | Editor; disclosed open finding | Defer without note, reload, restore health, then make later change | Health stays degraded while deferred; fresh health closes; later revision creates new open finding | Not Run | `uat/manual/uat-7b-02.json` plus screenshots |
| UAT-03 in-place update | Component manager and editor; current published version referenced | Update semantic payload in place and load editor | Same typed reference; revision/audit advance; Dashboard-owned finding appears; Core does not apply Dashboard policy | Not Run | `uat/manual/uat-7b-03.json` |
| UAT-04 successor Upgrade | Manager/editor; old active version referenced | Publish successor, inspect pinned reference, execute Upgrade | Old reference remains until action; only declared active published same-Component successor is offered; atomic receipt closes finding | Not Run | `uat/manual/uat-7b-04.json` plus screenshots |
| UAT-05 Replace/Remove/conflict | Editor with two authorized renderable targets | Replace with other ComponentVersion, replay stale request, then exercise Remove fixture | Valid actions atomic; stale replay deterministic no-op; findings and placements converge | Not Run | `uat/manual/uat-7b-05.json` |
| UAT-06 archive/tombstone | Component manager; published and superseded fixtures | Archive, attempt reactivate, tombstone, resolve as authorized/restricted, attempt mutation | Confirmation and state machine enforced; payload immutable; authorized archived metadata only; tombstone typed result only; restricted generic | Not Run | `uat/manual/uat-7b-06.json` plus screenshots |
| UAT-07 resolution/outage matrix | Owner/editor/viewer/restricted actors; known/random IDs | Exercise lifecycle, availability, compatibility, outage, retry, and recovery | Dimensions remain distinct internally; viewer writes nothing; restricted projections match; recovery converges | Not Run | `uat/manual/uat-7b-07.json` |
| UAT-08 Dataset equivalence | Dataset/Component managers | Exercise existing Dataset impact/carry-forward through typed adapter and compare Component observation | Shared typed/nondisclosure semantics; provider-specific actions remain owned; no cross-module DB access/new workspace | Not Run | `uat/manual/uat-7b-08.json` |
| UAT-09 approved visual contract | Product owner/reviewer; approved UI package; manager/editor/viewer roles | Capture affected routes/states at 1280/768/390, 1×, dark/light; compare the richer deployed Core chrome and approved mockup; inspect identical Core/Dashboard shell anatomy, keyboard/focus, and 200% zoom | Both routes use identical SDK-owned chrome and responsive behavior; unchanged product UI matches deployed baseline; additions match approved deltas; no parallel shell, P0/P1/P2 mismatch, overflow, hidden action, or prototype-only control | Not Run | `uat/manual/uat-7b-09.json`, comparison sheets, screenshots, traces |

Manual UAT may begin only after scripted UAT passes on the SIT-approved frozen
candidate. Each manual receipt records actor, timestamp, precondition, exact
action, expected/actual result, screenshot/trace links, and reviewer sign-off.

## Failure Classification And Invalidation

| Failure class | Examples | Required response |
|---|---|---|
| Product/contract | Wrong lifecycle, missing revision, invalid action, V1 fallback | Retain failure; correct outside validation; restart readiness onward |
| Authorization/nondisclosure | Metadata leak, wrong scope, known/random difference | Fail closed; invalidate candidate; broad security retest |
| Persistence/transaction | Duplicate finding, partial rebind, viewer write | Invalidate; correct baseline/service; fresh rehearsal |
| Test/harness/fixture | Weak assertion, changed accepted fixture, false pass | Invalidate; repair integrity; rerun full dependency cone |
| Environment/transient | Port conflict, tool outage with unchanged source | Retain receipt; prove identity; rerun affected lane plus convergence as authorized |
| Evidence finalization | Missing hash, corrupt manifest, incomplete linkage | No product rerun only if immutable raw evidence is complete and policy permits finalization repair |

There is no implementation or test correction inside SIT/UAT. Any correction
batch is separately identified, reviewed, and linked to a new candidate chain.

## Evidence Integrity And Closeout Authorization

Before closeout:

- recompute every raw artifact hash and validate the evidence manifest;
- prove one fingerprint spans preflight, all passing SIT lanes, scripted UAT,
  nine manual UAT receipts, final convergence, and documentation;
- confirm all R1–R11, both exit rows, and AC-01–AC-15 have passing evidence;
- confirm zero warnings, failures, waivers presented as passes, missing receipts,
  uncommitted product changes, or runtime V1 fallback;
- reconcile plan, verification record, roadmap, progress report, operator docs,
  and rollback instructions; and
- obtain explicit closeout authorization through the specialized closeout skill.

Until those conditions are met, Sprint 7B is not validated or complete.

## Planning Record

The execution contract was revised and approved on 2026-08-03 after settling
all outstanding product decisions. Implementation and the mutable pre-freeze
validation cycle passed on 2026-08-04. The canonical readiness and rehearsal
receipts under `artifacts/sprint-7b-closeout/` bind the current clean commit,
tree, environment, and evidence hashes. Those receipts are non-authoritative
and do not claim SIT or formal UAT; their identities are intentionally not
duplicated in this tracked record because doing so would make the record a
self-invalidating fingerprint input.

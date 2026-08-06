# Sprint 8A Validation Record

Status: Candidate Rehearsal attempt 17 completed a fail-late diagnostic harvest; its consolidated runner correction batch requires a complete restarted readiness/rehearsal cycle.
Its six findings are retained in one correction batch spanning acceptance
inventory, Component nondisclosure, and Dashboard release assets. No candidate
has been frozen; SIT, formal UAT, and closeout remain Not Run.

- Sprint: Sprint 8A — Component Module Separation Slice
- Branch: `codex/sprint-8a`
- Planned evidence root: `artifacts/sprint-8a-closeout/`
- Execution contract: [Sprint 8A plan](./sprint-8a-plan.md)

## Implementation Readiness Snapshot

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
| Phase 8 fresh materialization from empty | Partial/stale transition state | reset-target, empty-baseline, owner-order, semantic seed and no-op tests | first source-exact bootstrap plus unchanged second run | UAT-8A-02 |
| Failed fresh materialization destroys and reruns | Partial topology reused | induced failure, exact teardown, volume absence and complete rerun assertions | retained failed receipt followed by new from-empty healthy attempt | UAT-8A-07 |
| Old transition references and payloads unsupported | Hidden legacy compatibility | historical fixture integrity plus old owner/type/version/payload rejection | old inputs rejected; no adapter, ledger, reader or live old reference | UAT-8A-06 |
| Rerun Phase 7 scope/lifecycle/outage/compatibility | Logical behavior changes at physical boundary | contract, lifecycle, nondisclosure, scope, timeout, compatibility and outage suites | real Component/Dataset/Dashboard process-boundary smoke | UAT-8A-04/05 |
| Rerun source ownership, package graph, independent upgrade/rollback | Coupled release | boundary audits and upgrade runner assertions | Component-only upgrade/rollback; unrelated digests/restarts unchanged | UAT-8A-06/08 |
| Move all Component product and operational ownership | Split ownership or missing surface | module product/API/UI/operations integration suites | same-origin module-owned routes and diagnostics | UAT-8A-01/03 |
| Real Release/Instance and fresh database | Wrong identity/provenance/storage | release/instance/catalog/schema/database-isolation tests | composition read-back and exact provenance | UAT-8A-02/06 |
| Dashboard has one canonical inventory/navigation owner | Duplicate transition and deployed-module presentation | exact five-transition identity assertion; Dashboard absent from Core destination resolver; one manifest contribution | inventory exposes Dashboard only as its live 3.0.0 Release/Instance and navigation exposes it exactly once | UAT-8A-02/06 |
| Dashboard placements created with new module references | Stale transition owner in seed/live data | Dashboard seed/reference schema and semantic inventory assertions | every placement resolves through Components v3; zero old owners/types | UAT-8A-02/05 |
| Typed Core Dataset compatibility references/contracts | Direct Dataset relationship retained | typed composite identity, exact version, source/credential audits | Component authors/renders across Core contract only | UAT-8A-04 |
| Preserve Dashboard public Components behavior | Contract or consumer-policy regression | Sprint 7B observation/finding/action regression suites | lifecycle, findings and actions across real provider | UAT-8A-05 |
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

## Required Evidence Inventory

| Artifact | Producer | Required before | Status |
|---|---|---|---|
| `validation-readiness-result.json` | Validation coordinator | Rehearsal | Not Run |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | Not Run |
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
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
- `uat/manual/`: UAT-8A-01 through UAT-8A-08 receipts, screenshots/traces and
  reviewer sign-off.
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
- Deployment profile/configuration digest: planned
  `deploy/sprint-8a/compose.yaml`, override and normalized environment contract.
- Schema/baseline identity: empty Core, Component, Dashboard, Supervisor and
  selected-module schema migration digests plus bootstrap/seed contract
  versions. No legacy populated-data baseline exists.
- Expected provenance labels: OCI revision, source tree, dirty state, build
  profile, module definition/release, manifest, asset, schema migration,
  contract, fixture, seed and acceptance-inventory digests.
- Observed image digests: Not Run.

## Validation Readiness

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
- Acceptance mapping: every inventory row maps to SIT, smoke and UAT evidence.
- Clean repository and source-exact inputs: required before rehearsal.
- Result receipt: `artifacts/sprint-8a-closeout/validation-readiness-result.json`.

## Candidate Rehearsal

Sprint 8A rehearsal uses the dependency-aware inventory below. The attempt
receipt declares this graph before assertions. After any failure the attempt is
`harvesting`: independent siblings continue, dependent checks are recorded as
blocked with the exact failed prerequisite, and no tracked correction or new
attempt may begin until one harvest receipt and one consolidated defect batch
account for every row. `scripts/test-sprint-validation-harvest.ps1` enforces the
terminal-state and single-batch contract; materialization retains failed apply
responses and service logs before exact teardown.

| Diagnostic lane | Planned command/evidence | Assertions | Result | Defect batch |
|---|---|---|---|---|
| Static and boundaries | fmt/check/Clippy, manifests, links, native/WASM graphs, source/image audits | zero warnings; no forbidden owner/dependency/route/storage/legacy edge | Not Run | |
| Full Rust | `cargo test --workspace --locked` plus targeted contract/schema/authorization tests | all pass | Not Run | |
| Source-exact materialization | authorized reset, empty schemas, first/no-op owner bootstrap | exact provenance, healthy topology, semantic seed, exact no-op | Not Run | |
| Playwright | `npm --prefix .\end2end test` with final worker/retry contract | complete inventory; zero unexpected skip/retry/flake; retained outputs | Not Run | |
| Conformance and nondisclosure | module testkit plus Components/Dataset/Dashboard matrix | owner/version/scope/audience/known-random/timing/lifecycle cases pass | Not Run | |
| Deployed smoke | general and Sprint 8A smoke in rehearsal namespace | real boundaries, fixtures, old-input rejection, outage/recovery and final health | Not Run | |
| Failure teardown/rerun | induced partial materialization failure | evidence retained; exact topology/volumes removed; new empty rerun healthy | Not Run | |
| Automated UAT diagnostics | automated equivalents of UAT-01 through UAT-08 | every precondition and expected semantic state reproducible | Not Run | |

The first two lanes are independent of a deployed topology. Playwright locked
installation/discovery, runner self-tests, and acceptance-inventory checks are
also independent. Healthy materialization/no-op is the prerequisite for live
Playwright execution, deployed smoke, deployed inventory/navigation audit,
automated UAT diagnostics, failure-containment successor health, and the
Component upgrade/rollback baseline. Formal deployed acceptance smoke remains
SIT-owned and is not a rehearsal substitute.

The Sprint 8A reference materialization rebuilds the established demo and
Sprint 7A semantic acceptance fixtures through their owning APIs and databases
before deployment evidence is captured. This is a pre-production rebuild, not
a compatibility migration. The complete Playwright inventory must never rely
on one earlier test file to create shared fixtures. Exact deployed
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

Complete validation requires five pairwise-distinct, freshly created disposable
database bindings: API, fresh-baseline API, reference module, API enrollment,
and Installation Control. A database identity used by one complete command is
not reused by another command in the same rehearsal.

- Mutable source/environment identity: Not Run.
- Passing readiness prerequisite: Not Run.
- Consolidated defects and correction batch: Attempt 15 retained six
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
- Complete-cycle repetitions: 0.
- Result receipt: `artifacts/sprint-8a-closeout/candidate-rehearsal-result.json`.

Rehearsal is mutable, diagnostic and non-authoritative. Any correction requires
complete readiness and complete rehearsal to repeat before freeze.

## Environment Contract

- Environment fingerprint: Not issued.
- Intended gateway: `http://127.0.0.1:8088`; intended Supervisor endpoint:
  `http://127.0.0.1:8098`.
- Compose project/profile: `tessara-sprint-8a`,
  `deploy/sprint-8a/compose.yaml`, required profiles explicitly enabled.
- Databases: empty disposable installation-scoped Core, Component, Dashboard,
  Supervisor and selected-module databases with owner-specific runtime and
  schema-migration roles.
- Reset authorization: explicit approval bound to resolved absolute Sprint 8A
  project/container/volume/database identities only. Broad targets, unresolved
  variables and neighboring projects fail closed.
- Account/role fixtures: administrator, global Module Management reader/manager,
  Component manager, scoped Dashboard manager, constrained reader, out-of-scope
  actor and Core/Component/Dashboard service identities; no secrets in receipts.
- Evidence root/output mode: `artifacts/sprint-8a-closeout/`, repository-relative
  canonical paths, append-only attempts and atomic completion where supported.

## Planned Commands

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo test --locked --offline -p tessara-components-contract
cargo test --locked --offline -p tessara-component-module
cargo test --locked --offline -p tessara-dashboard-module
docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config
.\scripts\smoke-sprint-8a.ps1 -SelfTest
.\scripts\uat-sprint-8a.ps1 -SelfTest
.\scripts\test-sprint-validation-harvest.ps1 -SelfTest
.\scripts\validate-sprint-8a-readiness.ps1 -Attempt <n>
.\scripts\materialize-sprint-8a.ps1 -AuthorizeDisposableReset -Confirm -VerifyNoOp
.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/rehearsal/deployed-inventory-navigation.json"
.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/rehearsal/deployment-fresh.json" -AcceptanceEvidencePath "artifacts/sprint-8a-closeout/rehearsal/deployed-smoke.json"
.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098"
.\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath "artifacts/sprint-8a-closeout/rehearsal/component-upgrade-rollback.json"
.\scripts\uat-sprint-8a.ps1
.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8088" -DeploymentEvidencePath "artifacts/sprint-8a-closeout/rehearsal/deployment-fresh.json" -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a
```

The database-backed tests require their validation database URL variables.
Both `scripts/validate.ps1` and direct workspace all-features execution also
require
`SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET=I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET`;
an attempt that omits it is retained as an invocation failure and is not
silently retried during diagnostic harvesting.
Failure teardown/rerun, nondisclosure, deployed smoke, browser coverage, and
upgrade/rollback are executed and retained by the specialized validation
workflow. Deployed acceptance smoke belongs to SIT; manual scenarios belong to
UAT.

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

## SIT

| Lane | Prepare receipt | Command/evidence | Assertions | Result | Duration |
|---|---|---|---|---|---|
| Static and boundaries | `sit/attempts/static-<n>.json` | fmt/check/Clippy, manifest/link/schema, native/WASM/package/source/image/legacy audits | zero warnings and no forbidden edge | Not Run | |
| Rust workspace | `sit/attempts/rust-<n>.json` | `cargo test --workspace --locked` plus targeted contract/schema/authorization suites | all pass on frozen candidate | Not Run | |
| Playwright | `sit/attempts/playwright-<n>.json` | `npm --prefix .\end2end test` | complete inventory; UI/config/auth/outage/old-input cases pass | Not Run | |
| Deployed acceptance smoke | `sit/attempts/deployed-smoke-<n>.json` | source-exact reset/materialization, first/no-op, old rejection, outage/recovery, failure rerun and upgrade/rollback | exact candidate; receipts complete; current topology healthy | Not Run | |

- SIT result receipt: `artifacts/sprint-8a-closeout/sit-result.json`.
- Canonical topology restoration: current Sprint 8A Component release active;
  all services healthy; canonical fresh seed; only v3 Component references;
  no old reader, open failure or partial topology.

## UAT

### Scripted UAT

- Command: `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` plus
  the Sprint 8A source-exact wrapper fixed before readiness.
- Start rule: only after authoritative SIT passes on the same fingerprint.
- Result: Not Run.
- Evidence: `artifacts/sprint-8a-closeout/uat/scripted/`.

### Manual UAT

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| UAT-8A-01 unchanged Component experience | Component manager/reader; fresh canonical seed | Browse, create disposable Component, edit/publish/version, exercise lifecycle and view every supported kind through direct/shell routes | Same accepted product behavior and canonical vocabulary; module owns documents/assets; disposal succeeds | Not Run | `uat/manual/uat-8a-01.json` plus screenshots/trace |
| UAT-8A-02 from-empty seed and new references | Operator; explicitly authorized empty Sprint 8A databases | Run source-exact materialization, inspect owner order/read-back, rerun unchanged, open seeded Dashboard | Full recognizable app; Dashboard placements use selected Component instance/v3; no old owners/types; second run no-op | Not Run | `uat/manual/uat-8a-02.json` |
| UAT-8A-03 configuration and diagnostics | Global Module Management manager/reader | Set label and valid timeout; try blank/long label, 0/31 timeout, unknown field/version; inspect navigation/admin/product headings/diagnostics | Label affects nav/admin only; default 5 and range 1–30 enforced; authority correct; diagnostics sanitized | Not Run | `uat/manual/uat-8a-03.json` plus screenshots |
| UAT-8A-04 Dataset contract, scope and outage | Component manager plus scoped/out-of-scope actors; Dataset fixtures | Author/preview/execute; attempt wrong audience/scope; stop/timeout/recover Dataset provider; retry preserved editor | Typed reference only; allowed paths work; restricted cases do not disclose; outage is read-only with no pending write; recovery succeeds | Not Run | `uat/manual/uat-8a-04.json` plus screenshots |
| UAT-8A-05 Dashboard lifecycle and outages | Component manager, Dashboard manager/reader; v3 placement | Change Component lifecycle/revision, refresh/defer/act; stop/recover Component and Dataset paths | Sprint 7B findings/actions preserved; Dashboard degrades coherently; unrelated routes stay healthy; recovery converges | Not Run | `uat/manual/uat-8a-05.json` plus screenshots |
| UAT-8A-06 isolation and unsupported old inputs | Operator plus authorized/restricted actors | Inspect images/graphs/credentials/Core absence; submit V1/V2, old owner/type and old payload requests | No forbidden source/storage edge; old inputs fail exact normal contract with nondisclosure; no adapter/ledger/fallback | Not Run | `uat/manual/uat-8a-06.json` |
| UAT-8A-07 failed materialization rerun | Operator; disposable authorized topology; induced owner-bootstrap failure | Run failing attempt, inspect evidence, verify exact teardown, remove fault and rerun | Failure retained; partial project/volumes absent; successor begins empty and reaches canonical health | Not Run | `uat/manual/uat-8a-07.json` |
| UAT-8A-08 Component-only upgrade/rollback | Operator/reviewer; healthy extracted current release | Record identities, upgrade only Component, exercise routes/data/configuration, roll back, restore intended current release | Health-gated switch; Component state retained; unrelated digests/restarts unchanged; final topology healthy | Not Run | `uat/manual/uat-8a-08.json` |

- UAT result receipt: `artifacts/sprint-8a-closeout/uat-result.json`.
- Final topology restoration: intended Sprint 8A route/slot and current
  Component release; healthy Core/Component/Dashboard/Dataset path; canonical
  fresh seed; only v3 live references; no open defect.

## Changed Integration Contracts

| Boundary | Planned change | Required negative proof |
|---|---|---|
| Component public contract | exact-current `3.0.0`, Module Instance owner and `tessara.components.component_version` | old/mixed/wrong-instance/cross-installation/malformed/unauthorized inputs fail closed |
| Component product API | canonical module-owned typed-reference bodies on stable same-origin product paths | old Core payloads rejected; no translation reader/facade |
| Core Dataset compatibility | typed Dataset-major-line reference and versioned catalog/schema/distinct/execution/compatibility operations | no private DTO/SQL/credential; wrong audience/scope/version and outage do not disclose or fall back |
| Module configuration | schema v1 label and Dataset timeout | unknown schema/field, invalid label, 0/31 timeout and wrong authority rejected |
| Dashboard dependency | v3 provider binding and directly seeded new references | no old owner/type in storage; no Component implementation dependency |
| Fresh bootstrap | destructive full reset and owner-ordered idempotent seed | ambiguous target rejected; no cross-owner writes; second run no-op; failed partial topology never reused |
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

Runtime chronology, initially empty:

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction/narrow proof | Invalidation scope | Authoritative replacement |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

Classifications are exactly `preflight/setup`, `product`, `harness`,
`environment`, `flaky`, `evidence-finalization`, or `product-decision`.
Product decisions pause for user direction.

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
- Invalidation decisions satisfied: N/A before execution.
- Unresolved product decisions: None.
- Intended active route/slot: source-exact Sprint 8A gateway with current
  Component release and canonical fresh seed.
- Application health: Not evaluated during planning.
- Evidence source commit: Not frozen.
- Documentation commit: Not created.
- Authorization timestamp: None.

Closeout is forbidden until the validation coordinator verifies the complete
hashed chain and writes authorization. This planned record implies no candidate,
deployment, SIT, UAT or product result.

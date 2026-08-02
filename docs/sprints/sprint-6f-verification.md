# Sprint 6F Validation Record

Status: **Passed and authorized for sprint closeout** on 2026-08-01/02.
The evidence source is the clean frozen candidate below. The later commit that
records these results is documentation-only and does not redefine the tested
candidate.

## Candidate identity

- Branch/worktree: `codex/sprint-6f` at
  `C:\Users\eric-dev\Projects\tessara-sprint-6f`
- Evidence-source commit:
  `599680992771fb2ac05633e36cae2ad84026318d`
- Tree: `773b15f3798c79e7223de75de9e956d033cae901`
- Dirty state at freeze: clean
- Deployment profile: `deploy/sprint-6f/compose.yaml` using
  `tessara-oci-v1`
- Source labels on Core, Dashboards, Scoped Records, and Supervisor:
  commit/tree above and `com.tessara.source-dirty=false`
- Product image IDs:
  - Core: `sha256:fffa177c35712e6d65a58f634404ab6c9536fc7dbd408bda14032c21b281b4f4`
  - Dashboards: `sha256:17cfe49cd49c2abb03bf1d3d5ef2d431ee53a2cdcb483a6a15fa7441eace32f7`
  - Scoped Records: `sha256:116fb75b1a3a6be068e7aee7fcd559fe44b7b64235c5a2fdfda2b2b577fbb949`
  - Supervisor: `sha256:8e588e79e564e01c36d0dc21f91d36e78a97077bab1226b3de7a8468ca21ce0a`
- Reference plan:
  `sha256:dcc3a54d48231425e921d638d4966067ffc40a17d8b2f3353d26f3d6cd67e2af`
- Reference receipt:
  `sha256:48fd7b780e1c3ff4f712a44c2ef2c98f60121ea8a4eedee063ff93684258b406`
- Reduced plan:
  `sha256:1807fec0e50e7911e03cb8780b421e490eeaa9ad253f38d1b103893905d7d897`
- Reduced receipt:
  `sha256:344f8eb5bcaa04145b59a2fcaf3d5d0a9f5dd9144c6365797c878683e6cc58ee`

## Acceptance mapping

| Exit condition | Evidence and result |
|---|---|
| Bootstrap directly from a Blueprint | Fresh reference install, exact image labels, receipt/lockfile identity, navigation, roles, routes, and starter record all passed. |
| Bootstrap a detached signed lockfile | A newly resolved detached envelope using the frozen reference catalog matched the applied reduced lockfile exactly. |
| Resolve complete and reduced Blueprints | Reference selected two modules; reduced selected zero modules and ran only Core, Gateway, PostgreSQL, and Supervisor. |
| Separate planning and approval | Reader and constrained users were denied; planner created/resolved revision 2 but received 403 for approve/apply; admin approved and applied the exact planner digest. |
| Hand plans and authorization to Supervisor | Supervisor receipts and Core projections matched; unavailable Supervisor status was contained and recovered without loss. |
| Reproduce exact releases/config/bootstrap | Image, plan, configuration, role, module-instance, and bootstrap identities were checked; owner starter data remained stable. |
| Survive Core restart | Module IDs, Core bootstrap receipts, plan/receipt, roles, and unrelated container IDs were stable; smoke passed after restart. |
| Detect/adopt/reconcile drift | Configuration drift produced one adopted and one reconciled audit record; desired and observed state normalized. |
| Emergency disable and restore | Scoped Records returned 403 while Core and Dashboards remained 200; recovery restored 200 and retained the starter record. |
| Unchanged rerun is a no-op | Identical reference bootstrap advanced to revision 2 with `no_op=true` and a stable plan digest. |

## Preflight and freeze

- Clean branch/worktree and candidate topology confirmed.
- The six disposable PostgreSQL test databases were explicitly reset before
  each database-sensitive Rust lane.
- Harness reconciliation added Sprint 6F navigation presence/absence checks,
  exact detached-catalog handling, source provenance checks, and the eight UAT
  scripts before the final freeze.
- Migration/baseline proof was included in `scripts/validate.ps1`, including
  from-scratch and populated-upgrade coverage.
- Candidate freeze occurred after commit `59968099`; no tracked product, test,
  or harness file changed during SIT or UAT.

## SIT

| Lane | Command/evidence | Result | Duration |
|---|---|---|---|
| Full validation | `.\scripts\validate.ps1` | Passed, including format/static/boundaries, module contracts, native/WASM/web/API integration, migration/upgrade, and 200-sample timing | 22m 07.7s authoritative rerun |
| Rust workspace | `cargo test --workspace --locked` | Passed; all database-dependent integration targets and workspace doc tests ran | 9m 06.9s authoritative rerun |
| Source-exact reference deployment | Fresh `bootstrap-sprint-6f-composition.ps1 -Composition reference` plus provenance audit and smoke | Passed; all four product images matched commit/tree with `dirty=false` | 4m 43.8s build/apply; smoke 10.9s |
| Roles and access | Exact role-capability SQL/read-back plus 200/403/401 access matrix | Passed | <2s |
| Unchanged no-op | Reference bootstrap with `-SkipBuild` | Passed; receipt revision 2, `no_op=true`, stable plan | <11s |
| Core restart/recovery | Before/after DB, receipt, service identity, and smoke comparison | Passed | 16.7s |
| Drift management | Owner configuration change, adopt, second change, reconcile | Passed; one `adopted`, one `reconciled`, zero open | 1.7s |
| Emergency containment | Signed disable, route isolation, retained record, reconcile, smoke | Passed | 1.7s |
| Focused composition browser | `npm --prefix .\end2end test -- composition.spec.ts` | 3/3 passed | 6.1s |
| Detached reduced composition | Detached resolve/sign, fresh reduced bootstrap, smoke, Core restart | Passed; zero locked modules and zero module bootstrap receipts | 50.2s |
| Compatibility topology/provenance | Fresh Sprint 6E build, bootstrap, seed, five-image label audit | Passed | Included in compatibility batch |
| Full browser compatibility | `npm --prefix .\end2end test` | 65/65 passed | 3m 13s test runtime; 15m 55.1s including exact rebuild/bootstrap/audit |
| Supervisor failure containment | Stop/status failure/Core checks/restart/receipt read-back/smoke | Passed; Core, shell, and composition page stayed 200 | <30s |
| Final canonical smoke | Fresh canonical reference restore and `smoke-sprint-6f.ps1` | Passed at reference plan identity | 44.3s |

## UAT

### Scripted UAT

- Command: `.\scripts\uat-sprint-6f.ps1`
- Result: **Passed**
- Evidence: reference smoke passed at receipt revision 2 and the composition
  browser suite passed 3/3.

### Business scenarios

Detailed retained outcomes are in
[Sprint 6F UAT Results](./sprint-6f-uat-results.md).

| Scenario | Result |
|---|---|
| Complete reference bootstrap | Passed |
| Detached signed reduced bootstrap | Passed |
| Planning, approval, and restricted access | Passed |
| Core restart recovery | Passed |
| Drift adopt and reconcile | Passed |
| Emergency disable and restore | Passed |
| Owner bootstrap/idempotency | Passed |
| Responsive and failure states | Passed |

## Failure and restart chronology

| Gate/lane | Classification | Observation | Resolution and retained proof |
|---|---|---|---|
| Initial preflight | Product/harness | The original intended candidate lacked executable detached bootstrap, drift/emergency paths, exact provenance, and complete UAT coverage. | Corrected in commits `62f15977`, `e310aa76`, `b3305d2a`, and `59968099`; preflight restarted before freeze. |
| Restart checkpoint | Harness | An ad-hoc probe queried authenticated `/api/health/ready` and received 401 for 90 seconds. | Corrected probe to public `/health/ready`; the checkpoint alone reran and passed in 16.7s. No candidate change. |
| Compatibility Playwright attempt 1 | Flaky/state | 63 passed, Dashboard SSR expected one unavailable card but observed two, and one later test did not run. | Same unchanged deployment: focused test 1/1, Dashboard file 8/8, then full suite 65/65. Build/bootstrap/provenance were retained. |
| Emergency UAT precondition | Harness | PowerShell aggregation misread a one-record array as absent and stopped before mutation. | Corrected response handling; emergency batch passed without repeating prior lanes. |
| UAT authorization collector | Harness | Plain-text 403 and HTTP 201 success were initially treated as JSON/200-only failures. | Collector corrected; no candidate change. Valid-body reader probes returned 403 and full separation scenario passed. |
| Canonical restore | Expected environment guard | Non-replacing bootstrap refused the UAT revision-2 Blueprint. | Disposable project recreated with `-ReplaceExisting`; final canonical smoke passed. |
| Closeout evidence audit | Missing prerequisite | The closeout evidence directory was absent, so closeout stopped without changing roadmap or progress state. | Reopened the full validation regime and generated a source-exact retained evidence set before resuming closeout. |
| Reopened SIT preflight | Environment | The first validation command omitted the explicit destructive-reset acknowledgement. | Added the required acknowledgement and restarted SIT from lane one; the failure log is retained. |
| Reopened browser attempt 1 | Transient environment | One intentionally unknown module navigation emitted a single `Failed to fetch`; 60 tests passed and 4 did not run. | Focused reproduction passed 1/1 unchanged; SIT restarted from lane one. The authoritative full batch later passed 65/65. |
| Reopened Rust lane | Environment | `TEST_INSTALLATION_CONTROL_DATABASE_URL` was absent because a similarly named non-test variable was supplied. | Audited all six exact database variable names and restarted SIT from lane one; the authoritative workspace passed. |
| Reopened browser pre-test gate | Evidence harness | Source-exact build/bootstrap/seed passed, but the audit queried the wrong provenance label keys and stopped before tests. | Corrected the verified label mapping, restarted SIT from lane one, and retained the authoritative five-image audit plus 65/65 result. |
| Restart-containment checkpoint | Evidence harness | Core restart and identity checks passed, but an absolute smoke output path was rejected before smoke; Supervisor had not been stopped. | Corrected the repository-relative output path and reran the complete checkpoint; restart smoke and Supervisor containment/recovery passed. |

## Retained closeout evidence

- Directory: `artifacts/sprint-6f-closeout/`
- Evidence manifest: `evidence-manifest.json`
- Manifest file count: 100
- Manifest SHA-256:
  `1b760a3405a06247f830e0de2050b1f0d9fca8232883c0b435524051846b64d6`
- Required evidence includes source provenance, rendered Compose
  configuration, migration baselines, signed catalogs and authorization
  envelopes, both Blueprints/lockfiles/Materialization Plans, Supervisor and
  installation receipts, first/no-op/restart results, reference and reduced
  smoke, scripted/manual UAT, the authoritative Playwright summary, and all
  failure classifications and logs.
- The manifest and its `.sha256` sidecar were regenerated only after every
  required JSON artifact parsed successfully.

## Closeout authorization

- Status: **Authorized**
- Authorized candidate:
  `599680992771fb2ac05633e36cae2ad84026318d` /
  `773b15f3798c79e7223de75de9e956d033cae901`
- SIT passed: Yes
- UAT passed: Yes
- Acceptance mapping complete: Yes
- Unresolved product decisions: None
- Intended active route/slot: canonical Sprint 6F reference composition at
  `http://127.0.0.1:8080`
- Application health: Core 200, Supervisor 204, final smoke passed
- Evidence-source commit: `599680992771fb2ac05633e36cae2ad84026318d`
- Documentation commit: the closeout-only commit containing this record
- Authorization timestamp: 2026-08-02T03:24:00Z

# Sprint 6F Validation Record

Status: validation preflight failed on 2026-08-01. Candidate freeze, SIT, and
UAT did not begin. The exploratory reference-composition results remain
diagnostic only and are not candidate evidence.

## Scope and acceptance inventory

| Roadmap exit-condition clause | Risk/contract | Automated proof | Smoke proof | Manual UAT proof |
|---|---|---|---|---|
| Bootstrap directly from an Application Blueprint. | Product/deployment: first install must use the shared engine and Supervisor, not handwritten database or deployment glue. | Disposable-install test resolves the reference Blueprint, materializes it through the Supervisor, and verifies lockfile/plan/receipt identities. | Fresh deployed smoke verifies healthy Core, gateway, modules, resolved configuration/policy, bootstrap receipts, and provenance match the reference lockfile. | From an empty authorized environment, bootstrap the reference Blueprint and inspect the installed composition and receipt. |
| Bootstrap from a detached signature over a resolved lockfile/plan digest. | Authorization/trust: pre-resolved bootstrap must verify the signer and exact digest without treating planning as approval. | Signature tests accept the trusted exact digest and reject tampered, untrusted, expired, or cross-installation inputs without mutation. | Reduced-composition smoke verifies the signed plan identity in the Supervisor ledger and observed receipt. | Bootstrap a second disposable installation from the signed reduced lockfile/plan digest and inspect trust and provenance. |
| Resolve two different Blueprints. | Product/compatibility: complete and reduced compositions must resolve deterministically without inventing omitted services. | Golden tests resolve complete and reduced fixtures identically through Core and CLI and assert different expected closures/digests. | Smoke compares observed inventories to each lockfile and proves reduced-composition omissions. | Resolve both in the UI/CLI and compare dependencies, selected releases, actions, and omissions. |
| Separately approve the resolved compositions. | Authorization: a plan must remain inert until a correctly scoped, current Apply Authorization Envelope is issued. | Envelope tests cover valid independent approvals plus missing, replayed, stale, expired, conflicting, cross-installation, and under-scoped rejection. | Smoke proves each applied receipt references its own accepted envelope and base/target plan. | Plan without approval as a planner, then approve each composition independently as an authorized administrator. |
| Hand Materialization Plans and Apply Authorization Envelopes to the local Supervisor. | Deployment/authority: Core and machine clients must share the out-of-process handoff; Core cannot replace itself. | Contract/integration tests submit plan+envelope through Core and CLI, assert ledger serialization, and reject direct or malformed apply paths. | Deployed smoke reads Supervisor status/ledger and confirms the active receipt came from the approved plan handoff. | Submit from UI and CLI, observe pending/running/completed states, and verify Core exposes read-back rather than executing materialization. |
| Reproduce exact Core components, Module Releases, configuration/policy, and declared bootstrap composition from lockfiles plus external secrets. | Deployment/product/security/rollback: artifact, configuration, secret-reference, bootstrap, provenance, and rollback identities must be exact and secrets must not leak. | Reproduction test rebuilds disposable installations from both lockfiles, compares all required digests/receipts, validates owner-only bootstrap idempotency, and scans artifacts/logs for secret values. | Smoke verifies exact images, versions, desired/observed enablement, configuration/navigation/role/bootstrap digests, health, provenance, and rollback anchors. | Inspect complete and reduced installations, exercise declared product bootstrap records through owner UIs, and verify omitted modules and secret redaction. |
| Survive a Core restart during apply. | Lifecycle/deployment: the Supervisor ledger must own and resume/complete the operation exactly once while Core recovers read-back. | Fault-injection test restarts Core during a serialized apply and asserts one final receipt, no duplicated migrations/bootstrap, and consistent read-back. | Smoke after restart verifies application health, intended active routes, one operation outcome, and unchanged unrelated service identities. | Start an apply, restart Core, monitor Supervisor status, then confirm recovered UI shows the same final receipt and usable application. |
| Detect and reconcile a deliberate UI configuration change. | Product/authorization/lifecycle: desired/actual drift must be explicit and adopt/reconcile must be audited and authorized. | API/web/ledger tests create drift and verify detection, adopt, reconcile, conflict handling, permissions, revisions, and receipt outcomes. | Smoke changes an approved non-secret setting, verifies drift, reconciles it, and confirms restored lockfile/read-back digests. | Change module configuration/navigation in UI, inspect drift, then exercise adopt and reconcile paths, including a constrained non-admin denial. |
| Rerun an unchanged Blueprint as a no-op without handwritten deployment or database glue. | Idempotency/data safety: replay must not duplicate product records, rerun migrations, replace services, or advance unintended state. | End-to-end idempotency test runs the same bootstrap twice and compares ledger, receipt, database, bootstrap, image, and container/restart identities. | Second deployed bootstrap emits explicit no-op evidence while health and provenance remain unchanged. | Rerun the unchanged Blueprint command and verify no actions, no duplicated records, no service replacement, and no manual remediation. |

## Cross-cutting risk inventory

- Product: deterministic normalization, dependency closure, configuration and
  bootstrap schema validation, complete/reduced composition behavior, drift
  semantics, and module-owned product mutation.
- Authorization: separate planning and approval; envelope installation/base/
  digest/revision/nonce/expiry/effect binding; non-admin denial; capability
  floor; emergency override; and secret nondisclosure.
- Lifecycle: desired versus observed enablement, Core restart during apply,
  Supervisor serialization/recovery, bootstrap retry/read-back, drift adopt or
  reconcile, and emergency-override expiry.
- Deployment: exact trusted artifacts, Deployment Profile compatibility,
  migrations, health gates, traffic switching, content-addressed acquisition,
  locally operable Supervisor, and complete receipts/provenance.
- Rollback: prior plan/receipt and service/data anchors remain explicit;
  failed health/bootstrap changes do not silently advance active state; any
  destructive effect requires explicit authorization and documented limits.

## Candidate identity

- Implementation commit: Pending implementation commit; not frozen
- Tree: Not frozen
- Dirty state: Not evaluated
- Image digest(s): Not recorded
- Source-provenance labels: Planned exact commit/tree and `dirty=false`
- Deployment profile/configuration digest: Planned `deploy/sprint-6f/compose.yaml`; not recorded
- Migration/baseline identity: Not recorded
- Acceptance-manifest identity: This roadmap-seeded inventory; final digest not recorded
- Composition engine/schema version: Not recorded
- Catalog/Blueprint/lockfile/plan identities: Not recorded
- Supervisor/deployment-adapter version: Not recorded

## Preflight

- Status: **Failed — stopped before candidate freeze**
- Intended branch/worktree: `codex/sprint-6f` at `C:\Users\eric-dev\Projects\tessara-sprint-6f`
- Intended implementation commit: `56be613d333aee3800a8b698b50618fce77cb515`
  with tree `4f807d79cf0789edd781fe13570b4d578c19fa14`; worktree was
  clean when audited.
- Environment and reset authorization: Must be confirmed before SIT; fresh/disposable database and installation destruction require explicit confirmation at execution time.
- Test state: Planned fresh Core, Dashboard, and Scoped Records PostgreSQL
  databases plus a protected Supervisor SQLite ledger named by the final
  profile. No Sprint 6E database state is migrated.
- Deployment profile: Planned `deploy/sprint-6f/compose.yaml` using `tessara-oci-v1`.
- Bootstrap/materialization and no-op proof: Implemented as
  `scripts/bootstrap-sprint-6f-composition.ps1`; exploratory reference first
  apply and unchanged no-op passed. Candidate evidence has not been retained.
- Harness/inventory reconciliation: Not Run; reconcile routes, navigation, roles, catalogs, releases, services, seeds, lifecycle schemas, bootstrap declarations, smoke, UAT, and Playwright before freeze.
- Migration from-scratch proof: Not Run; after the final schema change, squash sprint-owned migrations and apply each finalized baseline to disposable empty databases. Record N/A only if no schema changes land.
- Evidence paths: Planned `artifacts/sprint-6f-closeout/`; must be empty or explicitly overwriteable before SIT.

### Preflight failures

1. **Harness inventory is stale.** `scripts/smoke.ps1`,
   `scripts/uat-sprint.ps1`, and the 62-test Playwright acceptance manifest have
   no Sprint 6F Application Composition assertions. The only smoke occurrence
   of `supervisor_materializable` is older module-manifest metadata, not
   composition acceptance coverage.
2. **Roadmap exit paths are not executable.** The CLI has no detached
   pre-resolved lockfile/plan signature verification path. Core exposes only
   disposition updates for an already-existing drift row; no implemented path
   creates deliberate UI drift or performs adoption/reconciliation. The
   Supervisor schema contains `emergency_overrides`, but no emergency-disable
   API, ledger workflow, expiry behavior, or UI action uses it. No supported
   fault hook or resumable Core-restart-during-apply scenario exists.
3. **Observed deployment provenance is not observed.** The deployed adapter
   records `AcquireImage` digests from the submitted plan as
   `observed_artifacts`; it does not inspect or verify the running source-exact
   images. Compose starts and migrates services before the Supervisor apply, so
   the current receipt cannot prove Supervisor-owned reproduction of the
   locked composition.
4. **Manual UAT is not executable as frozen.** Six prepared scripts retain
   `NEEDS INFO` prerequisites for isolated reduced bootstrap, detached input,
   constrained accounts, restart timing, drift UI, emergency disable, and CAS
   tamper execution. Per the script rules, those scenarios would be Blocked.
5. **The running stack is diagnostic only.** Its images were built before the
   intended implementation/UAT commit and therefore cannot establish the
   frozen candidate's source provenance.

Required correction: implement the missing roadmap behaviors and source-exact
deployment observation, reconcile smoke/scripted-UAT/Playwright coverage in
the same change, eliminate candidate-independent `NEEDS INFO` blockers, then
rerun preflight from the beginning. Migration baseline proof remains pending
until that corrected implementation is final.

## SIT

| Lane | Command/evidence | Result | Duration |
|---|---|---|---|
| Static and boundaries | `cargo fmt --all -- --check`; composition/catalog/schema/package/secret/provenance checks; `docker compose -f .\deploy\sprint-6f\compose.yaml config` | Not Run | |
| Targeted Rust | `cargo test -p tessara-composition`; `cargo test -p tessara-supervisor`; `cargo test -p tessara-api`; `cargo test -p tessara-web` | Not Run | |
| Rust workspace | `cargo test --workspace --locked` | Not Run | |
| Playwright | `npm --prefix .\end2end test` with Sprint 6F evidence wrapper/manifest | Not Run | |
| Deployed acceptance smoke | `.\scripts\smoke.ps1` against the frozen `deploy/sprint-6f/compose.yaml` candidate, including Blueprint/signature, authorization, restart, drift, no-op, non-admin, and exact-provenance assertions | Not Run | |

SIT passes only when all lanes pass from the beginning for one clean candidate.
Deployed acceptance smoke is part of SIT, not a closeout activity.

## UAT

Tester-ready source scripts are indexed in
[Sprint 6F UAT Scripts](./sprint-6f-uat/README.md). Preparing these scripts does
not start UAT or satisfy any evidence requirement.

### Scripted UAT

- Command: `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- Result: Not Run
- Evidence: Planned `artifacts/sprint-6f-closeout/uat-scripted.json`
- Frozen script/inventory identity: Not recorded

### Manual UAT

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| Direct Blueprint bootstrap and complete composition | Local authorized operator; empty disposable installation | Resolve, approve, and apply the reference Blueprint; inspect UI and receipt | Exact complete composition, healthy routes, locked configuration/policy/bootstrap/provenance | Not Run | Planned `uat-manual.json` plus screenshots/receipts |
| Signed pre-resolved bootstrap and reduced composition | Local authorized operator; second empty disposable installation | Verify detached signature, create separate envelope, apply reduced plan | Exact reduced composition; omitted modules absent; trust and receipt identities visible | Not Run | Planned signed-bootstrap evidence |
| Planning versus approval and non-admin scope | Planner plus constrained non-admin; healthy installation | Create plan, attempt approval/apply, inspect restricted state | Planner cannot self-approve; non-admin cannot approve/apply or infer secrets/restricted metadata | Not Run | Planned permissions evidence |
| Core restart during Supervisor apply | Administrator; authorized pending change | Begin apply, restart Core, observe Supervisor, reopen Core status | Exactly one completed operation/receipt; no duplicate migration/bootstrap; UI recovers | Not Run | Planned restart chronology |
| UI drift adopt and reconcile | Administrator; lockfile-matched installation | Make deliberate configuration/navigation edit; inspect drift; adopt then create/reconcile another drift | Explicit revisions and audited outcomes restore or update desired state correctly | Not Run | Planned drift evidence |
| Emergency disable override | Authorized administrator; healthy enabled module | Issue reasoned expiring disable; inspect degraded UI and drift; expire/adopt/reconcile | Non-destructive audited override, contained module state, persistent explicit drift | Not Run | Planned override evidence |
| Module-owned bootstrap and content digest | Administrator; module with inline and referenced declarations | Apply/read back, rerun, then tamper referenced content | Owner API creates expected records once; second run no-op; tamper rejected | Not Run | Planned bootstrap receipts |
| Responsive, accessible, and failure states | Administrator and constrained non-admin; desktop/tablet/mobile | Exercise plan/diff/approval/status/provenance/conflict/unavailable views, keyboard, theme, direct SSR/hydration | Clear accessible state, clean console, stable non-disclosing failures at 1280/768/390 px | Not Run | Planned Playwright/manual screenshots |

## Failure and restart chronology

| Time | Gate/lane | Candidate | Classification | Correction | Narrow proof | SIT restart |
|---|---|---|---|---|---|---|
| 2026-08-01 06:15 -04:00 | Preflight | Intended `56be613d` / `4f807d79` | `preflight/setup` plus implementation gaps | Not corrected during validation request; recorded exact missing product and harness paths | Source/harness inventory, Compose/runtime identity audit | SIT never began |

Permitted classifications are `preflight/setup`, `product`, `harness`,
`environment`, `flaky`, and `product-decision`. A `product-decision` pauses for
user direction. Any SIT or UAT failure or executable candidate change requires
a new or reconfirmed candidate and a full SIT restart before UAT resumes.

## Closeout authorization

- Status: Not Authorized
- Authorized candidate: None
- SIT passed: No
- UAT passed: No
- Acceptance mapping complete: Inventory established; evidence pending
- Unresolved product decisions: None recorded at kickoff
- Intended active route/slot: To be frozen during preflight
- Application health: Not evaluated
- Evidence source commit: None
- Authorization timestamp: None

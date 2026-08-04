# Sprint 7B: Cross-Module Resource Lifecycle And Dependency Slice

Status: execution plan and interactive UI contract approved; implementation in progress.

Branch: `codex/sprint-7b`

Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-7b`

## Sprint Summary, Outcome, And Roadmap Authority

Sprint 7B makes provider-owned Component and Dataset change observable and
actionable across the real Dashboard process boundary. The sprint establishes a
small policy-neutral observation vocabulary, advances the transition-only
Components contract, gives Components a complete lifecycle, and lets Dashboard
own durable dependency findings and actions without inventing provider policy.

The complete Sprint 7B block in `docs/roadmap.md` is authoritative. It requires:

- typed resolution and revision/state-change contracts;
- provider-owned lifecycle and versioning rules;
- a declared observation mechanism and durable change markers;
- stale findings, carry-forward, upgrade, replacement, removal, and deferral;
- stable owner, resource type, and resource ID through mutable state;
- provider-only publication and activation guards;
- distinct internal resolution outcomes with a non-disclosing restricted result;
- contract-version and consumer regression coverage;
- Dashboard dependency health and action UI; and
- a real-boundary Component proof with equivalent Dataset adapter semantics.

## Scope

### In scope

- Add policy-neutral typed reference, structured resolution, observation
  strategy, contract identity, and monotonic resource-revision primitives to
  `tessara-module-contract`.
- Advance `tessara-components-contract` to exact current V2 and add immutable V2
  fixtures. Retain accepted V1 fixture files as historical evidence, but remove
  V1 runtime readers and fallback behavior.
- Expose Component publication state, lifecycle state, resource revision,
  authorized metadata, categorized changes since an optional prior revision,
  and an optional provider-declared successor.
- Add separate Component lifecycle and publication state, immutable lifecycle
  audit records, and a monotonic resource revision to Core's fresh squashed
  baseline.
- Preserve the existing rule that the current published ComponentVersion may be
  updated in place. Superseded versions and archived or tombstoned versions are
  payload-immutable.
- Persist Dashboard-owned dependency observations, findings, and idempotent
  action receipts in Dashboard's fresh squashed baseline.
- Refresh authorized findings on Dashboard editor load and explicit retry;
  provide viewer live resolution without persistence.
- Add editor-integrated dependency summary, filtering, issue detail, and
  defer/upgrade/replace/remove actions. Do not add a dedicated dependency route.
- Obtain product-owner approval of the interactive, deployed-UI-grounded review
  package under `docs/sprints/sprint-7b-ui-review/` before production UI work.
  Treat its bounded screen deltas and approved captures as the visual contract.
- Converge Core and Dashboard on one policy-neutral shell presentation owned by
  `tessara-module-ui`. Port the richer deployed Core chrome into that owner,
  make Core and Dashboard consume it, and remove their parallel local shell
  renderers in the touched cone.
- Retain the Dataset product's existing impact/carry-forward UI. Prove Component
  and Dataset observation equivalence through typed adapters and deployed tests.
- Remove or narrow Core's Dashboard-specific layout compatibility veto and raw
  dependency projection so Dashboard computes Dashboard layout findings.
- Add source-exact bootstrap, fixtures, smoke, SIT, scripted UAT, manual UAT,
  evidence retention, recovery, and rollback coverage.

### Explicitly out of scope

- Product code, schema, fixture, test, or deployment changes during planning.
- Production UI implementation before explicit approval of the Sprint 7B UI
  review package.
- A new Component dependency workspace or dedicated Dashboard dependency page.
- A new public `tessara-datasets-contract` or physical Dataset module extraction;
  Phase 8 retains that ownership migration.
- Scheduled polling, event delivery, queues, webhooks, or background refresh.
- Automatic rebinding, a manual `Mark resolved` action, or health suppression by
  deferral.
- Lifecycle actions for drafts, reactivation from archived, or recovery from
  tombstoned.
- A mandatory reason or note for lifecycle transitions or deferral.
- Incremental production migration, populated-schema upgrade, down migration,
  or compatibility with old runtime contract readers. Tessara is pre-production.

## Current-State Findings And Affected Components

- `tessara-module-contract` already owns typed resource resolution and restricted
  construction. It lacks a compact observation identity and resource revision.
- `tessara-components-contract` V1 is the transition boundary used by the real
  Dashboard process. It exposes current Component data but not the approved
  lifecycle, change, revision, or successor semantics.
- Core currently combines publication concerns with limited Component state and
  contains Dashboard-specific compatibility behavior that violates consumer
  ownership.
- The current published ComponentVersion can already be updated in place. Sprint
  7B must make that rule observable rather than silently replacing it with an
  immutable-publication policy.
- Dashboard stores typed Component references but has no durable observations,
  findings, deferrals, or atomic rebinding receipts.
- Dashboard editor/viewer and Component Versions surfaces already exist and are
  the canonical UI homes for the new behavior.
- DatasetRevision already has publication, compatibility, impact, and
  carry-forward behavior. Sprint 7B wraps the current relational identity in a
  transitional typed adapter; it does not preempt Phase 8 extraction.
- Current migration sets are pre-production squashed baselines. Sprint 7B edits
  those baselines and verifies only fresh materialization.

Affected areas are the module and Components contracts, Core Components
persistence/services/routes/UI, Dashboard persistence/services/routes/UI, the
Dataset transition adapter, shared fixtures, Compose/bootstrap scripts, smoke,
Playwright, and sprint-validation evidence tooling.

## Specifications

### 1. Policy-neutral observation contract

- A typed observation binds a typed resource reference, exact provider contract
  identity/version, declared observation strategy, and monotonic
  `resource_revision`.
- The shared contract represents structured resolution dimensions without
  defining Component or Dataset lifecycle policy and without defining findings,
  dispositions, upgrades, replacements, or other consumer actions.
- Observation strategy for this sprint is live resolution plus a monotonic
  revision marker. No event-delivery promise is implied.
- Owner, resource type, and resource ID remain stable when publication,
  lifecycle, availability, compatibility, or resource revision changes.
- Authorization is evaluated before metadata, change detail, successor, or
  existence is disclosed. Known and random unauthorized IDs return the same
  restricted envelope, response shape, UI projection, logging class, and bounded
  timing behavior.

### 2. Components contract V2 and provider policy

- V2 is the only runtime Components contract after the change. V1 golden and
  invalid fixtures remain immutable historical evidence; runtime V1 readers,
  negotiation, and fallback are removed.
- V2 returns publication state (`draft`, `published`, `superseded`) separately
  from lifecycle state (`active`, `inactive`, `archived`, `tombstoned`). Drafts
  have no lifecycle actions.
- Allowed lifecycle transitions for published and superseded versions are:
  `active -> inactive`, `inactive -> active`, `active|inactive -> archived`, and
  `archived -> tombstoned`. Archived cannot reactivate; tombstoned is terminal.
- Publishing a successor changes the prior version to `superseded` without
  changing its lifecycle.
- Active published or superseded versions are renderable. Inactive and archived
  versions are metadata-visible to authorized callers but not renderable.
  Tombstoned versions retain their full internal record for audit/history but
  expose only typed tombstone resolution—no metadata or render payload.
- The current published version may receive authorized semantic updates in
  place while active or inactive. Such updates advance `resource_revision` and
  write change audit. Superseded, archived, and tombstoned payloads are immutable.
- Every publication, lifecycle, and observable semantic change advances a
  monotonic resource revision and exposes provider-authored change categories
  since an optional prior revision. Authorization freshness remains separate
  and cannot substitute for resource revision.
- A provider-declared successor is optional. When present it identifies an
  active published version of the same Component and is the only `Upgrade`
  target.
- Lifecycle transitions require existing scoped `components:manage` authority
  and record immutable actor, timestamp, from-state, and to-state audit data. No
  reason is required.

### 3. Dashboard-owned dependency behavior

- Dashboard persists manager-authorized observations, open/deferred/resolved
  findings, and immutable idempotent action receipts. Suggested canonical tables
  are `dashboard_dependency_observations`, `dashboard_dependency_findings`, and
  `dashboard_dependency_action_receipts`.
- An editor load performs an authorized idempotent refresh. Explicit retry uses
  the same operation. A viewer resolves live for display but never writes
  observations, findings, or receipts.
- A finding is keyed to the saved reference and observed provider revision. A
  later provider revision creates a new open finding even when the prior finding
  was deferred.
- Dashboard computes its own compatibility and layout impact. Provider change
  categories and resolution outcomes are evidence, not Dashboard policy.
- Any disclosed finding may be deferred without a note. Deferral leaves
  dependency health degraded and affects only that finding revision.
- There is no manual resolve operation. A fresh healthy observation, successful
  Upgrade, Replace, or Remove resolves/supersedes affected findings atomically.
- `Upgrade` uses only the provider-declared active published successor of the
  same Component. `Replace` accepts any authorized renderable ComponentVersion.
  `Remove` deletes the placement/reference according to existing Dashboard edit
  authority.
- All actions validate authorization, target renderability, expected finding
  revision, and current saved reference. Stale or replayed requests are safe,
  produce deterministic receipts, and never partially mutate placement/finding
  state.
- Restricted callers receive generic dependency health only; they do not receive
  provider IDs, saved references, lifecycle state, change detail, successor, or
  action targets.

### 4. Dataset adapter equivalence

- Transitional Dataset adapters synthesize typed references and observations
  around current DatasetRevision relational IDs and existing publication and
  compatibility behavior.
- Existing Dataset impact and carry-forward UI remains the Dataset product
  surface. No Component-to-Dataset dependency workspace is added.
- Conformance tests prove Dataset and Component adapters share typed reference,
  observation, authorization-first nondisclosure, stable identity, and revision
  semantics while retaining provider-specific lifecycle/action rules.
- No consumer reads another module's database and no adapter becomes a second
  owner of provider policy.

### 5. UI, API, observability, and operations

- Component lifecycle controls live on the existing Component Versions page.
  Activate/deactivate are direct scoped actions; archive/tombstone require
  confirmation and clearly state irreversibility.
- Dashboard dependency health is integrated into the existing editor as a
  summary, filter, and issue sheet. It contains observed state/revision, impact,
  retry, defer, Upgrade, Replace, and Remove where authorized.
- The richer deployed Core chrome is the canonical visual shell. Core and
  Dashboard must obtain it from `tessara-module-ui`, with differences limited
  to Shell Context, active destination, route title, and product content. Other
  unchanged navigation, typography, density, spacing, color, borders,
  responsive behavior, and existing controls must remain a 1:1 visual match to
  the deployed application. Sprint 7B additions must match the product-owner-
  approved interactive mockup at the same route, state, theme, viewport, pixel
  density, content, and authorization context.
- APIs separate read/live-resolution, authorized refresh, and tagged action
  commands. Action commands include expected finding revision and an idempotency
  key; no command accepts `resolved` as an action.
- Structured logs include correlation, actor class, reference digest, prior and
  current revision, action/result code, and provider contract identity without
  leaking restricted resource data.
- Health/provenance identifies exact Core, Dashboard, contract, fixture, schema,
  and source identity used by validation.

## Decisions, Assumptions, Dependencies, And Blockers

### Settled product decisions

- Preserve authorized in-place updates of the current published ComponentVersion.
- Keep publication and lifecycle separate and implement the full state machine.
- Use logical terminal tombstones with internal retention and external metadata
  suppression.
- Use existing `components:manage`; lifecycle and deferral require no reason.
- Refresh Dashboard findings on editor load and explicit retry; viewer reads do
  not persist; no scheduler or event pipeline is introduced.
- Remove manual resolution. Findings close only from fresh health or an atomic
  Upgrade, Replace, or Remove.
- Allow any disclosed finding to be deferred; deferral does not improve health.
- Distinguish provider-successor Upgrade from arbitrary authorized Replace.
- Integrate dependency work into the Dashboard editor and lifecycle work into
  Component Versions; do not add dedicated routes.
- Keep consumer findings and action receipts out of the platform contract.
- Use fresh squashed baselines only and retain Phase 8 Dataset extraction scope.
- Require an interactive mockup approval gate and visual-conformance UAT for all
  affected Dashboard editor and Component Versions states.
- Require one SDK-owned shell for both affected routes; the deployed compact
  Dashboard shell is transitional drift and is not an approved baseline.

### Assumptions and dependencies

- Sprint 7A's source-exact closeout is the baseline authority.
- Existing scoped authorization and nondisclosure helpers remain canonical.
- The sprint may add narrowly typed transition adapters but cannot create a
  second provider-policy owner.
- Formal validation uses the repository's readiness, candidate rehearsal,
  preflight, SIT, UAT, and closeout skills and receipt formats.

### Open questions and blockers

No semantic product decision or implementation blocker remains. The product
owner approved `docs/sprints/sprint-7b-ui-review/` on 2026-08-04 before product
UI implementation began. Any discovery that changes these ownership,
lifecycle, action, approved visual, UI, or migration decisions must stop the
affected slice and amend this plan before proceeding.

## Acceptance Criteria

- **AC-01:** Valid V2 observations round-trip exact identity, strategy, contract,
  and resource revision; malformed or mixed-version forms fail closed.
- **AC-02:** Restricted known/random requests have indistinguishable public API,
  UI, log, and bounded timing projections.
- **AC-03:** Component lifecycle and semantic change advance resource revision
  without changing owner, type, or resource ID.
- **AC-04:** An authorized in-place update of the current published version is
  observed by Dashboard and creates/updates the correct finding without changing
  the saved typed reference.
- **AC-05:** Successor publication leaves the old reference pinned; Upgrade is
  offered only for the declared active published same-Component successor.
- **AC-06:** Replace accepts any authorized renderable ComponentVersion; stale,
  unauthorized, non-renderable, and replayed actions do not partially mutate.
- **AC-07:** Any disclosed finding can be deferred without a note, remains
  degraded, and a later resource revision creates a new open finding.
- **AC-08:** No manual resolution exists; fresh health or successful Upgrade,
  Replace, or Remove closes the affected finding atomically.
- **AC-09:** The complete Component lifecycle state machine, renderability,
  metadata visibility, tombstone projection, audit, and payload immutability
  rules are enforced for published/superseded versions; drafts have no actions.
- **AC-10:** Dashboard owns findings/layout impact and Core can publish Component
  changes without reading or enforcing Dashboard layout policy.
- **AC-11:** Editor load/retry refresh is idempotent; viewer reads do not persist;
  no scheduled/event-driven refresh exists.
- **AC-12:** Dataset and Component transition adapters satisfy the same typed
  observation and nondisclosure conformance without cross-module DB access.
- **AC-13:** V2 is exact current runtime, V1 fixtures remain immutable, and V1
  runtime readers/fallback are absent.
- **AC-14:** Fresh source-exact materialization and unchanged rerun are healthy,
  deterministic, warning-free, and produce complete validation evidence.
- **AC-15:** At matching routes, states, themes, viewports, density, content, and
  roles, production Dashboard editor and Component Versions UI matches the
  approved Sprint 7B mockup; both use the one `tessara-module-ui` shell with
  identical chrome and responsive behavior, no parallel local shell renderer
  remains in the touched cone, and all prototype-only controls are absent.

## Traceability Matrix

| Roadmap requirement | Specifications / acceptance | Slices | Automated and deployed proof | Manual proof |
|---|---|---:|---|---|
| R1 typed resolution and revision/state-change contracts | Specs 1–2; AC-01/03 | 1–2 | contract unit/golden/conformance; real resolution smoke | UAT-01/03/06 |
| R2 provider-owned lifecycle/versioning | Spec 2; AC-04/05/09 | 2–3 | state-machine, update, publication, audit tests | UAT-01/03/04/06 |
| R3 declared observation mechanism | Specs 1/3; AC-03/11 | 1/4 | marker monotonicity; editor/live read integration | UAT-01/03 |
| R4 changelog/stale/carry-forward/rebinding | Specs 2–4; AC-04–08/12 | 3–6 | change, finding, action, conflict, adapter tests | UAT-02–05/08 |
| R5 stable owner/type across mutable state | Specs 1–3; AC-03/04 | 1–4 | reference digest and before/after tests | UAT-01/03/06 |
| R6 provider-owned guards only | Specs 2–3; AC-10 | 3–4 | dependency-direction and publication regression | UAT-03/04 |
| R7 full outcome matrix and nondisclosure | Specs 1–5; AC-02/06/09 | 1–7 | conformance and known/random matrix | UAT-06/07 |
| R8 contract-version compatibility/regression | Specs 1–2; AC-01/13 | 1–2/7 | V1 immutable fixtures; V2 exact/failure tests | UAT-06 |
| R9 dependency health and observed-state UI | Specs 3/5; AC-07/11/15 | 5–6 | SSR/wasm/Playwright/deployed smoke and approved-mockup comparison | UAT-01/02/07/09 |
| R10 upgrade/carry-forward/rebinding UI | Specs 3–5; AC-05/06/08/12/15 | 5–6 | UI/action/API, adapter, and approved-mockup tests | UAT-04/05/08/09 |
| R11 resolve/defer without Core lifecycle policy | Specs 2–4; AC-07/08/10 | 4–6 | ownership, persistence, later-revision tests | UAT-02/03/05 |
| Exit: Component change across real Dashboard boundary | AC-03–11 | 2–8 | source-exact adapter, smoke, Playwright | UAT-01–07 |
| Exit: equivalent Dataset/Component adapters | AC-12 | 6–8 | conformance and deployed adapter proof | UAT-08 |

## Ordered Implementation Slices

| Slice | Work and touchpoints | Prerequisites | Required tests and completion signal |
|---:|---|---|---|
| 1 | Add policy-neutral observation primitives to `tessara-module-contract`; add valid/invalid/golden fixtures | Sprint 7A baseline | Unit/serde/nondisclosure tests pass; no product finding/action types in platform contract |
| 2 | Create exact Components V2 contract and immutable fixtures; remove runtime V1 readers/fallback | Slice 1 | V1 fixture immutability and V2 exact-version/failure suites pass |
| 3 | Update Core squashed baseline, repositories, services, routes, and Versions UI for publication/lifecycle/revision/audit/change/successor; remove Dashboard policy veto | Slice 2 | State-machine, immutability, in-place update, auth, audit, contract, UI tests pass |
| 4 | Update Dashboard squashed baseline and service layer for observations/findings/receipts; implement editor refresh, viewer read-only behavior, and action transactions | Slices 2–3 | Persistence, idempotency, stale action, no-manual-resolve, no-viewer-write tests pass |
| 5 | After explicit UI-review approval, establish the canonical SDK shell for Core and Dashboard, then add Dashboard API and editor-integrated dependency UX plus Component Versions lifecycle controls exactly matching the approved deltas | Slice 4 and approved `sprint-7b-ui-review` package | SDK-boundary, SSR/wasm/accessibility/Playwright/API and visual-conformance tests pass; one shell renderer remains and no dedicated route or prototype-only control exists |
| 6 | Add transitional Dataset typed adapter and retain existing Dataset impact/carry-forward surface | Slices 1–5 | Dataset/Component conformance and no-cross-DB tests pass |
| 7 | Reconcile fixtures, manifests, Compose, bootstrap, smoke, logging, health/provenance, and docs | Slices 1–6 | Fresh materialization and exact identity checks pass; docs and links agree |
| 8 | Execute readiness, rehearsal, preflight, SIT, UAT, evidence finalization, and closeout | Slice 7 | One frozen candidate satisfies every receipt and AC; no code changes during SIT/UAT |

Each slice must leave the workspace formatted, warning-free, and testable. A
slice is incomplete if required negative, authorization, nondisclosure,
contract-version, or idempotency coverage is missing.

## Automated, Integration, Smoke, And UAT Plan

Targeted suites must cover contract serialization and malformed inputs; exact V2
negotiation; V1 fixture retention; lifecycle transition and payload immutability;
in-place update and successor publication; resource revision monotonicity;
authorization and audit; Dashboard observation/finding state; refresh and action
idempotency; stale/replayed requests; viewer no-write; Dataset adapter
equivalence; UI states; and known/random nondisclosure.

Workspace and deployed gates are:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
npm --prefix .\end2end test
.\scripts\smoke.ps1
```

The developer-only `scripts\local-launch.ps1` is diagnostic, not formal
evidence. Formal evidence uses a source-exact Sprint 7B bootstrap profile, an
unchanged no-op rerun, targeted lifecycle/dependency smoke, `validate-e2e`,
`cargo audit`, Markdown-link verification, provenance checks, and a clean-diff
audit.

Manual UAT scenarios are:

1. **UAT-01 lifecycle observation:** deactivate/reactivate a referenced version
   from Component Versions and verify stable identity, editor refresh, finding,
   renderability, and audit.
2. **UAT-02 deferral/recovery:** defer without a note, verify degraded health,
   restore health, and verify automatic closure; then create a later revision
   and verify a new open finding.
3. **UAT-03 in-place update:** update the current published version, retain its
   typed reference, and observe the advanced revision and Dashboard finding.
4. **UAT-04 successor Upgrade:** publish a successor, verify the old reference
   stays pinned, then Upgrade only to the declared same-Component successor.
5. **UAT-05 Replace/Remove/conflict:** Replace with another authorized renderable
   ComponentVersion, exercise Remove, and prove stale replay is a safe no-op.
6. **UAT-06 archive/tombstone:** verify confirmations, terminal transitions,
   metadata/render matrix, internal retention, audit, and restricted projection.
7. **UAT-07 resolution matrix:** exercise owner, lifecycle, availability,
   compatibility, outage, restricted known/random, and recovery behavior.
8. **UAT-08 Dataset equivalence:** exercise existing Dataset impact/carry-forward
   through the transitional typed adapter and compare conformance.
9. **UAT-09 approved visual contract:** capture Dashboard editor, dependency
   sheet/action dialogs, Component Versions, and lifecycle confirmations in the
   approved roles/states at 1280, 768, and 390 pixels in dark and light themes;
   compare them with the approved mockup and deployed-baseline references at 1×,
   and verify prototype-only controls are absent.

## Validation, Evidence, Candidate Freeze, And Closeout Plan

- Validation readiness proves plan/verification traceability, test integrity,
  warning policy, exact commands, environment readiness, and evidence paths.
- Candidate rehearsal materializes mutable source, runs the full broad suite,
  and proves fresh/idempotent deployment before freeze.
- Preflight audits passing readiness and rehearsal receipts, computes one source
  fingerprint, freezes exact image/contract/schema/fixture identities, and
  permits SIT only for that candidate.
- SIT runs all unit/integration/contract/browser/deployed/recovery lanes and
  retains structured attempt receipts, raw stdout/stderr, screenshots, HTTP
  exchanges, DB read-back, logs, provenance, and manifest hashes.
- UAT starts only from passing SIT on the same fingerprint. Scripted and eight
  manual product scenarios plus the explicit visual-conformance scenario retain
  actor, precondition, action, expected/actual result,
  timestamps, screenshots, and linked raw artifacts.
- Any product, contract, fixture, migration, harness, deployment, or test change
  invalidates the candidate and restarts readiness/rehearsal/preflight. No
  implementation correction occurs inside SIT or UAT.
- The approved UI package, product-owner approval record, bounded delta record,
  source captures, and comparison method are candidate fingerprint inputs. A
  visual-contract change invalidates UI readiness and downstream evidence.
- Closeout requires all ACs and roadmap rows traced to passing evidence, zero
  unresolved warnings/failures, clean source identity, finalized evidence
  manifest, recovery/rollback audit, documentation reconciliation, and explicit
  closeout authorization.

The companion `docs/sprints/sprint-7b-verification.md` is the execution record
and must be populated without weakening expected results after failures.

## Rollout, Migration, Recovery, And Rollback

- Tessara remains pre-production. Core and Dashboard Sprint 7B schemas replace
  their current single squashed baselines; no additive upgrade or backfill path
  is implemented or validated.
- Fresh source-exact bootstrap creates the approved lifecycle, audit,
  observation, finding, and receipt state. An unchanged rerun must be a semantic
  no-op with no duplicates or revision drift.
- Runtime contract rollout is coordinated: provider, Dashboard consumer,
  fixtures, and deployment profile move to exact V2 together. Historical V1
  fixtures remain evidence, not a compatibility path.
- Recovery restores the same candidate's provider/network/database service and
  proves saved references, finding history, audits, and health converge without
  duplicate actions.
- Rollback restores the prior complete application composition and its matching
  database snapshot or disposable volumes. Old binaries are never run against
  the Sprint 7B baseline, and down migrations are not used.

## Risk Register

| Risk | Prevention | Detection | Recovery |
|---|---|---|---|
| Platform contract absorbs product policy | Keep findings/actions in Dashboard; review dependency direction | API/type inventory and ownership tests | Move policy type to Dashboard before next slice |
| Lifecycle and publication collapse | Separate fields and exhaustive state machine | transition/render matrix | correct provider model and rematerialize fresh baseline |
| In-place update is missed | monotonic resource revision and change audit | real-boundary update tests/UAT-03 | repair marker generation; refresh same candidate only after refreeze |
| Restricted data leaks | authorize before resolution detail; common restricted constructor | known/random API/UI/log/timing tests | fail closed, invalidate candidate, correct and restart validation |
| Deferral hides health or future change | finding revision key; health independent of disposition | persistence/later-revision tests/UAT-02 | rebuild derived findings from authorized refresh |
| Rebinding silently or partially mutates | expected revision, idempotency, atomic transaction | stale/replay/fault injection | transaction rollback and deterministic retry |
| Core retains Dashboard policy | narrow provider guards to provider invariants | dependency-direction/publication regression | remove consumer policy before Dashboard slice closes |
| Dataset adapter preempts Phase 8 | transitional adapter only; no new public boundary | crate/schema ownership audit | revert extraction and retain adapter proof |
| Production UI drifts from deployed baseline or approved deltas | block UI work on explicit approval; implement from retained source captures and delta records | matching-state automated screenshots and UAT-09 comparison | correct UI outside validation, recapture, and restart affected readiness chain |
| Fresh baseline drifts from deployment | source-exact identity and unchanged rerun | schema/provenance/read-back receipts | destroy disposable state and rematerialize exact candidate |

## Planning Audit

- The plan covers every Sprint 7B roadmap clause, implementation cone,
  acceptance criterion, test layer, manual proof, validation gate, evidence
  requirement, migration rule, recovery path, and closeout condition.
- All outstanding product choices were explicitly settled on 2026-08-03 and are
  normative in this plan.
- The product owner approved the retained interactive UI contract on 2026-08-04;
  `sprint-7b-ui-review/approval.md` records the authorization and frozen scope.
- Implementation must use the repository-local `tessara-implementation` skill;
  formal validation, UAT, and closeout remain owned by their specialized skills.

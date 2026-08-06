# Sprint 8A: Component Module Separation Slice

Status: Implementation-readiness correction complete after validation was
explicitly exited during incomplete Candidate Rehearsal attempt 27. Focused
implementation checks pass, but no current readiness or rehearsal result is
authorized. A new complete Validation Readiness and Candidate Rehearsal cycle
must start from the corrected clean commit. No candidate has been frozen, and
preflight, SIT, UAT, and closeout have not run.

- Branch: `codex/sprint-8a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-8a`
- Base commit: `37aa9c8da45491ef02dc4d62e5df5f3ece2af444`
- Roadmap authority:
  `Sprint 8A: Component Module Separation Slice (Next)` and the reconciled
  Phase 8 fresh-materialization rules in `docs/roadmap.md`
- Planned evidence root: `artifacts/sprint-8a-closeout/`
- Validation record: [Sprint 8A verification](./sprint-8a-verification.md)

## Sprint Summary, Outcome, And Roadmap Authority

Sprint 8A extracts Components from Core into the separately built and deployed
`tessara.components` full-stack module. The module owns Component authoring,
viewing, versions, execution, lifecycle, persistence, configuration,
diagnostics, routes, assets, manifest, capabilities, and schema migrations. It
consumes the still-in-Core Dataset provider only through a versioned public
compatibility contract and typed Core-owned Dataset-major-line references.
Dashboard consumes Components through the exact module-owned public contract
across a real process and database boundary.

Tessara is pre-production, and the reference application contains disposable
seed data rather than user-owned production data. Sprint 8A therefore uses one
offline, destructive, source-exact materialization from empty databases. Core
and module owners rebuild the complete canonical seed in dependency order;
Dashboard creates placements directly with new Component Module Instance
references. The sprint does not migrate Component product data, rebind stored
legacy references, retain a compatibility ledger, or preserve old Core API
payloads. Supported import and legacy mapping remain Phase 9 work.

## Scope

### In scope

- Create Module Definition `tessara.components`, initial release `1.0.0`, one
  selected Module Instance, an isolated Component database, and separate
  runtime and schema-migration identities.
- Move all Component UI, API, version/lifecycle behavior, execution,
  persistence, complete-document SSR, hydration, assets, configuration,
  health/readiness, diagnostics, manifest, capabilities, and conformance
  ownership out of Core.
- Publish exact-current Components contract `3.0.0` with Module Instance
  ownership and resource type `tessara.components.component_version`.
- Replace Component-to-Dataset foreign keys and implementation calls with a
  versioned Core Dataset compatibility contract and typed Dataset-major-line
  references.
- Keep Component product page URLs and accepted authoring/viewing behavior,
  while replacing old Core Component payloads with canonical module-owned
  typed-reference request/response shapes.
- Rebuild the entire disposable reference-application seed from empty through
  owning Core/module bootstrap contracts. Seed Components before Dashboards;
  Dashboard bootstrap uses read-back new Component references.
- Remove Core Component tables, handlers, route adapters, product DTOs,
  execution, transition adapter/readers, and old payload acceptance in the
  same offline cutover.
- Preserve Dashboard-owned layout, dependency findings, deferral,
  Upgrade/Replace/Remove, lifecycle observation, nondisclosure, outage, and
  recovery behavior against the new provider.
- Add Component module configuration with a shared-shell display label and a
  bounded Dataset-provider timeout, plus sanitized operational diagnostics.
- Pair every stale seed, smoke, UAT, Playwright, manifest, deployment, and
  bootstrap update with the behavior that requires it.
- Prove Component-only health-gated upgrade and rollback without rebuilding or
  restarting Core, Dashboard, Dataset compatibility, or unrelated modules.

### Explicitly out of scope

- Preserving or migrating current Core Component, Dashboard, or other
  reference-application product rows.
- Old-to-new reference mappings, Dashboard rebinding commands, migration
  receipts, migrated/retired reference ledgers, partial-data repair, checkpoint
  resume, dual write, or online/read-only cutover behavior.
- Any normal-runtime acceptance of old `core_installation` Component references
  or old Core Component API payload shapes.
- Dataset physical extraction; Sprint 8B owns that boundary and will use the
  same Phase 8 fresh-materialization policy.
- New Component kinds, lifecycle rules, Dashboard product policy, navigation
  policy, or visual redesign.
- General supported import/migration UX or a migration coordinator; Sprint 9A
  owns that work.
- Multi-instance selection, external module repositories, or package
  publishing.

## Current-State Findings And Affected Components

- Core `tessara-api` owns Component routes, DTOs, execution, lifecycle, and
  persistence, and its squashed baseline contains Component tables with direct
  Dataset foreign keys.
- Component UI is split between root-owned routes, `tessara-web-components`,
  and the route-free `tessara-web-component-viewer` implementation.
- `tessara-components-contract` V2 requires `core_installation` ownership and
  the transition resource type. A new exact generation is required; permissive
  dual-owner parsing would preserve an obsolete boundary.
- Dashboard is independently deployed but its persisted placement schema and
  manifest still target Core-owned Component references. With the approved
  fresh reset, its database and seed are recreated directly against new
  references rather than mutated through a rebind protocol.
- Current Component execution reads Dataset state through Core implementation
  code and storage. The extraction requires catalog, schema, distinct-value,
  compatibility, and execution/materialization contract operations.
- Sprint 6D/6E established canonical contract/runtime/UI/testkit,
  complete-document, generic routing, package-boundary, independent-image
  upgrade, and rollback patterns. Components must reuse those patterns without
  depending on root Core/web/API or another module implementation.
- Existing Component-specific rendering limits remain stored in each
  ComponentVersion. The only new installation-level product configuration is
  the display label and Dataset request timeout.
- The retained Sprint 7A deployment profile is immutable. Sprint 8A requires a
  new profile and explicit disposable-scope reset authorization.

Expected future implementation touchpoints include new Component module/UI
ownership, Components/Dataset contract packages, Dashboard dependency and
seed ownership, Core removal, generic gateway/composition registration, a
Sprint 8A deployment/bootstrap path, end-to-end tests, and acceptance runners.

## Specifications

### 1. Module, source, UI, and operational ownership

- `tessara.components` release `1.0.0` declares product and operational routes,
  Feature Declarations, capabilities `components:read` and
  `components:manage`, configuration schema v1, database, probes, assets,
  contracts, dependencies, and provenance.
- The Component release is the sole production owner of `/components`,
  `/components/new`, `/components/{component_ref}`,
  `/components/{component_ref}/edit`,
  `/components/{component_ref}/versions`, and
  `/components/{component_ref}/view`, plus their canonical API routes.
- Complete documents use verified Shell Context and `tessara-module-ui`.
  Direct loads, useful JavaScript-disabled SSR, hydration, history, theme,
  keyboard, accessibility, responsive layouts, and clean browser consoles
  remain required.
- User-facing product behavior and page vocabulary remain “Components.” No
  redesign or new migration status UI is introduced.
- Dashboard must not compile a Component feature/viewer implementation. Its
  placement renderer is consumer-owned and consumes only public execution
  envelopes plus canonical platform/UI packages.
- Native/WASM package and image audits prohibit root `tessara-web`,
  `tessara-api`, Core-private code, Dashboard implementation, Dataset
  implementation, and copied shared source from the Component release.

### 2. Configuration and diagnostics

Configuration schema v1 contains exactly:

- `display_label`: required string, trimmed, 1–80 Unicode scalar values,
  default `Components`; shown in shared-shell navigation and Module Management
  only. Product headings, breadcrumbs, API names, capability IDs, route names,
  and contract IDs remain canonical “Components” vocabulary.
- `dataset_request_timeout_seconds`: required integer, inclusive range 1–30,
  default 5; applied to Component-to-Dataset provider requests.

Installation-global Module Management manager authority continues to control
configuration through the existing generic path. `components:manage` does not
grant module-configuration authority, and no new capability is introduced.
Unknown fields and unsupported schema versions fail validation; normalization
does not silently clamp out-of-range timeout values.

Diagnostics expose release/instance identity, manifest and contract versions,
effective redacted configuration, Dataset dependency binding and compatibility,
last provider health observation, readiness/liveness, and stable sanitized
failure codes. They expose no product rows, counts, secrets, credentials,
resource identifiers, obsolete migration receipts, or old-reference details.

### 3. Component persistence and Dataset compatibility

- The Component database owns component shells, versions, lifecycle/publication
  state, change audit, typed Dataset bindings, configuration, and idempotency.
  No other runtime receives its credentials.
- ComponentVersion stores a typed Core-owned Dataset-major-line reference rather
  than a Dataset foreign key. The compatibility wrapper validates installation,
  owner, transition resource type, canonical composite Dataset UUID/positive
  major identity, contract ID/version, and schema centrally.
- Core provides versioned Dataset catalog, major-line metadata/schema,
  distinct-values, compatibility, and execution/materialization operations
  sufficient for every current Component kind and editor flow.
- Component exchanges the actor context through Core for the Dataset audience.
  Core authorizes the presenting Component service for the declared action;
  Dataset scope/capability checks remain provider-owned.
- Known and random unauthorized Dataset references are indistinguishable.
  Unavailable, timed out, incompatible, unauthorized/not-evaluated, missing,
  retired, and materialization-not-ready remain distinct internally and map to
  stable non-leaking Component results.
- During Dataset outage or timeout, existing Component metadata/configuration
  loads read-only with retry. Preview, save, publish, lifecycle, and all other
  Dataset-dependent mutations are unavailable until validation succeeds.
  Unsaved browser state is retained across retry. No operation falls back to
  direct SQL or persists a validation-pending draft.

### 4. Components v3 and Dashboard behavior

- `tessara.components.component-version` `3.0.0` is the only normal runtime
  generation. Its resource type is
  `tessara.components.component_version`, and every reference requires the
  selected Component Module Instance owner.
- V3 preserves Sprint 7B catalog, resolution, observation, lifecycle,
  publication, resource-revision, change, successor, execution, and
  nondisclosure semantics while adopting canonical module ownership.
- Historical V1/V2 fixtures remain immutable test evidence only. Normal
  Component and Dashboard readers reject V1/V2, old Core owners, the transition
  resource type, mixed generations, wrong instances, cross-installation
  references, malformed envelopes, and old Core payload shapes.
- Existing same-origin API paths may remain for product continuity, but their
  request/response bodies change directly to canonical v3 typed-reference
  shapes. All first-party clients, tests, smoke, UAT, and seed code change in
  the same candidate. No translation facade is retained.
- Dashboard continues to own layout, observations/findings, deferral,
  Upgrade/Replace/Remove, degraded presentation, and consumer action policy.
  Components and Core do not acquire Dashboard persistence or policy.
- Dashboard-to-Component calls use generic service registration, downstream
  audience exchange, the configured timeout behavior at the provider boundary,
  and exact v3 contracts. Component outage remains contained to Dashboard
  placement/dependency state; unrelated routes remain healthy.

### 5. Offline fresh materialization and seed ownership

- Sprint 8A uses a dedicated disposable Compose project/profile. Validation
  resolves and records exact project, volume, database, and port targets and
  requires explicit destructive-reset authorization before teardown.
- The public gateway is unavailable for the entire extraction. There is no
  maintenance page, read-only product mode, hidden-action protocol, or online
  compatibility interval.
- Materialization creates empty Core, Supervisor, Component, Dashboard, and
  other selected-module databases, applies each owner's schema migrations, and
  starts the new source-exact topology.
- Canonical seeding runs only through owning APIs/bootstrap contracts in this
  dependency order:
  1. Core installation, identity/RBAC, Organization, and still-in-process
     provider records;
  2. Component module records using Dataset compatibility references;
  3. Dashboard module records using Component v3 references returned by
     Component read-back.
- Orchestration may pass typed IDs and content-addressed seed inputs between
  owners, but it cannot write product tables or use another owner's credentials.
- A successful seed exposes recognizable fixtures for every Component kind,
  lifecycle/authorization/outage case, and Dashboard dependency action. Exact
  counts, where contractual, come from one shared acceptance source of truth.
- Repeating bootstrap without reset is an exact semantic no-op: no duplicate
  product rows, revisions, references, roles, configuration, or receipts.
- Any failure retains the failed attempt and raw evidence, destroys the partial
  disposable topology and volumes, and restarts from empty after correction.
  Partial repair, stage resume, old-stack restoration, and data salvage are not
  supported.

### 6. Core removal, observability, upgrade, and rollback

- Final Core contains no Component product tables, schema types, handlers,
  routes, DTOs, UI adapters, execution, seed writes, transition adapter,
  transition reader, old-payload reader, or Component-specific gateway branch.
- Old Core Component references and payloads are unsupported inputs. They fail
  exact contract validation and are never reported as migrated or retired.
- Generic platform integration retains only manifest/catalog, Shell Context,
  authorization exchange, service registration, dependency resolution,
  configuration/diagnostics proxy, and unavailable-route behavior.
- Logs and diagnostics include correlation, actor/service class,
  installation/instance/release/contract identity, reference digests, timeout
  and stable result codes without restricted resource or secret data.
- OCI labels and evidence bind commit, tree, dirty state, release, manifest,
  assets, schemas, fixtures, contracts, bootstrap, and acceptance inventory.
- After fresh extraction, prove a compatible Component-only health-gated
  upgrade and rollback. Component data, instance identity, configuration,
  routes, and behavior persist while Core, gateway, Supervisor, Dashboard,
  Dataset compatibility, and unrelated module image digests, container
  identities, restart counts, data, and availability remain unchanged.
- Final handoff restores the intended current Component release in the healthy
  freshly seeded Sprint 8A topology. Rollback evidence is not the handoff state.

### 7. Compatibility posture

Tessara remains pre-production. Sprint 8A is forward-only and intentionally
destructive. There is no transition-data preservation, dual write, populated
legacy upgrade, old contract negotiation, old reference resolution, old API
translation, checkpoint resume, or extraction rollback to the Core-owned
product. Schema migrations, fresh bootstrap idempotence, and post-extraction
Component release rollback remain required. Supported legacy import and
cross-module mapping are deferred to Phase 9.

## Decisions, Assumptions, Open Questions, Dependencies, And Blockers

### Approved product decisions

- Prioritize transition simplicity over availability and preservation because
  the application is not in production use.
- Apply the destructive fresh-materialization policy to every Phase 8 module
  extraction, not only Sprint 8A.
- Take the full reference application offline, rebuild every disposable product
  database from empty, and fresh-seed through owning contracts.
- On failure, destroy the partial topology and rerun from empty; do not restore,
  resume, or repair transition data.
- Do not migrate current seed data or create a synthetic legacy migration
  fixture. Real legacy migration/rebinding acceptance is removed from Phase 8.
- Do not retain a migration ledger or receipt UI. Old Component references and
  old Core payload shapes are unsupported.
- Preserve same-origin product URLs and accepted UI behavior, but make the API
  wire contract a direct canonical break with all first-party consumers updated
  together.
- Configure only display label and Dataset request timeout. The label affects
  navigation and Module Management, and the timeout defaults to 5 seconds with
  a strict 1–30 second range.
- During normal Dataset outage, Component authoring is read-only with retry;
  validation-dependent mutations do not save pending state.
- No visual mockup approval is needed unless implementation discovers a real
  redesign requirement, which pauses that surface for a plan amendment.

### Assumptions and dependencies

- The closed Sprint 7B behavior is the functional baseline, not a data-retention
  obligation.
- Sprint 6D/6E canonical packages and Dashboard extraction patterns remain the
  implementation baseline.
- Core generic manifest, gateway, composition, Shell Context, authorization,
  and Module Management paths can enroll Components without a module-ID branch.
- All destructive commands operate only on an explicitly resolved disposable
  Sprint 8A project and never on broad or ambiguous targets.

### Open questions and blockers

No product decision or planning blocker remains. Internal crate layout, port
allocation, opaque Dataset-major-line encoding, and runner names are bounded
implementation details. Any request to preserve legacy data/references, add an
online transition, restore old payload compatibility, or redesign product UI
requires a plan amendment and user approval.

## Acceptance Criteria

- **AC-01:** The Component image owns its binary, database/schema migration,
  manifest, complete documents, hydration, assets, configuration, diagnostics,
  and every Component product route, with no root Core/web/API or sibling
  implementation dependency.
- **AC-02:** Final Core has no Component product storage, code, routes, seed
  writes, adapter, legacy reader, or module-specific integration branch.
- **AC-03:** From explicitly authorized empty disposable databases, one
  source-exact materialization produces a healthy complete reference
  application and owner-controlled canonical seed in the declared order.
- **AC-04:** An unchanged second bootstrap is a semantic no-op with no duplicate
  data, references, revisions, roles, configuration, or receipts.
- **AC-05:** Dashboard seed creates every placement directly with a Component
  v3 Module Instance reference returned by Component read-back; no old owner or
  transition resource type exists in live state.
- **AC-06:** Old Core Component references, V1/V2 runtime envelopes, and old
  Core payload shapes fail exact normal-contract validation; no translation,
  migrated/retired result, or fallback exists.
- **AC-07:** Component directory, create, edit, versions, view, every supported
  kind, publication, lifecycle, and execution remain user-visible and
  behaviorally equivalent through module-owned same-origin routes.
- **AC-08:** Configuration validates exact schema v1, label behavior, timeout
  default/range, authority, persistence, and sanitized diagnostics.
- **AC-09:** Component stores typed Core-owned Dataset-major-line references and
  performs catalog, validation, distinct-value, compatibility, and execution
  only through the versioned Dataset contract.
- **AC-10:** Dataset scope/audience/capability, known/random nondisclosure,
  incompatible, timeout, outage, not-ready, and recovery cases pass without
  direct database access or validation-pending writes.
- **AC-11:** Dashboard preserves Sprint 7B lifecycle/revision observation,
  findings, deferral, Upgrade/Replace/Remove, restricted disclosure, outage,
  and recovery against Components v3.
- **AC-12:** Runtime/migration credentials cannot cross Core, Component,
  Dataset, or Dashboard database ownership; no cross-database SQL, FDW, shared
  writable schema, or foreign key exists.
- **AC-13:** A failed fresh materialization retains evidence, removes the exact
  partial disposable topology and volumes, and succeeds only after a complete
  from-empty rerun.
- **AC-14:** Component-only upgrade and rollback preserve Component data,
  identity, configuration, routes, and behavior while unrelated digests,
  identities, restart counts, data, and availability remain unchanged.
- **AC-15:** At 1280, 768, and 390 px, dark/light themes, keyboard operation,
  200% zoom, JavaScript-disabled SSR, and hydrated navigation, Component UI
  matches the accepted baseline and produces no hydration or console errors.
- **AC-16:** Core's frozen transition catalog contains exactly Forms,
  Workflows, Responses, Datasets, and Migration. Dashboard is absent from Core
  transition inputs, inventory, semantic destination resolution, and default
  navigation; it appears exactly once through its enrolled Module Release,
  Module Instance, and manifest navigation contribution.
- **AC-17:** Format, check, zero-warning Clippy, full workspace tests,
  native/WASM/package boundaries, Playwright, source-exact materialization,
  smoke, scripted/manual UAT, failure rerun, upgrade/rollback, provenance, and
  evidence integrity pass on one frozen candidate.

## Traceability Matrix

| Roadmap requirement | Specification / acceptance | Slice | Automated/deployed proof | Manual UAT |
|---|---|---:|---|---|
| Independently built/deployed full-stack module | Specs 1/6; AC-01/12/14 | 1–6 | package/image/source audit; topology; upgrade chronology | UAT-8A-01/06/08 |
| Canonical packages; no Core/root/sibling implementation | Spec 1; AC-01/02 | 1/4 | native/WASM dependency and forbidden-source audits | UAT-8A-01/06 |
| Module-owned admin/config/manifest/capabilities/DB/schema migrations/health/routes/assets/conformance | Specs 1–2/5; AC-01/08/12/16 | 2–6 | manifest, testkit, schema, route, asset, credential suites | UAT-8A-01/03/06 |
| APIs/contracts/typed references only | Specs 3–4; AC-09–12 | 1/3/4 | contract, source, SQL, and credential tests | UAT-8A-04/05/06 |
| Phase 8 fresh materialization and unsupported old references | Specs 5/7; AC-03–06/13 | 1/5/6 | empty bootstrap, no-op, old-input rejection, teardown/rerun | UAT-8A-02/07 |
| Rerun scope/lifecycle/outage/compatibility/source/image/rollback | Specs 1–6; AC-10–16 | 1–6 | conformance, Playwright, smoke, upgrade/rollback | UAT-8A-04/05/06/08 |
| Move all Component product/operational ownership | Specs 1–3/6; AC-01/02/07/08 | 2–4 | module integration and Core absence | UAT-8A-01/03 |
| Real Component Release/Instance in fresh database | Specs 1/5; AC-01/03/12 | 2/5 | composition read-back, isolated DB/provenance | UAT-8A-02/06 |
| Dashboard placements created with new references | Specs 4–5; AC-05/11 | 4/5 | semantic seed and Dashboard regression smoke | UAT-8A-02/05 |
| Replace Dataset DB relationships with compatibility contract | Spec 3; AC-09/10/12 | 1/3/5 | typed reference, scope, outage, no-SQL tests | UAT-8A-04 |
| Preserve Dashboard public Components behavior | Spec 4; AC-05/11 | 3–6 | v3 consumer regression and deployed smoke | UAT-8A-05 |
| Rebuild seed/test data through owners | Spec 5; AC-03–05/12/13 | 5 | seed ownership, read-back, no-op and failure-rerun tests | UAT-8A-02/07 |
| Dependency outage, scope, capability, compatibility | Specs 3–4; AC-10/11 | 3/5/6 | nondisclosure, timeout, outage/recovery smoke | UAT-8A-04/05 |
| Unchanged UI plus module configuration/diagnostics | Specs 1–2; AC-07/08/15 | 2/4/6 | SSR/hydration/visual/accessibility/console tests | UAT-8A-01/03 |
| Exit: fresh seed uses only new Component references | Spec 5; AC-03–06 | 5/6 | source-exact read-back and old-input absence/rejection | UAT-8A-02 |
| Exit: Component execution through Dataset contracts | Spec 3; AC-09/10 | 3/5/6 | real-boundary smoke and credential denial | UAT-8A-04 |
| Exit: Dashboard consumption and coherent degradation | Spec 4; AC-11 | 4–6 | Dashboard deployed outage/recovery lane | UAT-8A-05 |

## Ordered Implementation Slices

### Slice 1 — exact contracts and failing ownership boundaries

- Prerequisites: approved reconciled planning package and clean sprint worktree.
- Touchpoints: Components/Dataset contract packages, Dashboard dependency
  declaration, source/package/route/storage audit helpers.
- Work: lock Components v3, typed Dataset-major-line, canonical API payload,
  package graph, route ownership, unsupported-old-input, and no-cross-storage
  contracts as failing tests.
- Harness in slice: golden/invalid/historical fixtures, native/WASM dependency
  audits, accepted-test inventory integrity.
- Complete when the target boundary is reviewable as red tests without product
  code or data moving opportunistically.

### Slice 2 — Component release, database, configuration, and operations

- Prerequisites: Slice 1 contracts.
- Touchpoints: Component module/UI ownership, manifest, schema migration,
  configuration, diagnostics, Compose image/profile scaffolding.
- Work: establish release/instance, isolated identities/database, contract
  ownership, exact configuration, probes, diagnostics, and testkit conformance
  without switching product traffic.
- Harness in slice: manifest/config/diagnostic/probe/schema/credential tests and
  source/image provenance assertions.
- Complete when a shadow empty Component instance materializes and reports
  exact identity/configuration with no Core product ownership change.

### Slice 3 — Component product and Dataset contract boundary

- Prerequisites: Slices 1–2.
- Touchpoints: Component service/API/persistence/execution, Dataset compatibility
  adapter/contract, authorization exchange, Component seed bootstrap.
- Work: move Component behavior behind module-owned canonical APIs and replace
  every Dataset storage/implementation edge with typed authenticated calls.
- Harness in slice: all kinds and lifecycle paths; scope, capability, audience,
  known/random nondisclosure, timeout, outage, not-ready, recovery, read-only
  editor/retry, and negative credential/source coverage.
- Complete when the shadow module reproduces accepted behavior across the real
  Dataset boundary and stores no Dataset foreign key.

### Slice 4 — module-owned UI/API cutover and Core removal

- Prerequisites: Slice 3 parity.
- Touchpoints: Component documents/hydration/assets, Dashboard consumer
  renderer/v3 client, generic gateway registration, root web/API removal.
- Work: move all same-origin UI/API traffic to Components, update every
  first-party client to canonical payloads, remove Component implementation
  dependencies from Dashboard, and remove every Core product/legacy path.
- Harness in slice: direct/shell/no-JS/hydration/history/theme/keyboard/
  responsive/accessibility/console, API exactness, Core absence, old-input
  rejection, Dashboard v3 regression.
- Complete when normal traffic is module-owned and no runtime reader accepts an
  old owner, type, generation, or payload.

### Slice 5 — destructive materialization and owner-controlled seed

- Prerequisites: Slice 4 complete boundary.
- Touchpoints: Sprint 8A Compose/bootstrap, all owner seed entrypoints,
  Dashboard seed, general/sprint smoke and UAT fixtures/evidence.
- Work: implement explicit safe reset, empty schema materialization, ordered
  owner bootstrap/read-back, Dashboard new-reference seed, exact second-run
  no-op, induced failure teardown, and complete from-empty rerun.
- Harness in slice: target-resolution/reset self-tests, semantic fixture source
  of truth, no cross-owner writes, first/no-op receipts, failure evidence,
  teardown verification, canonical restoration.
- Complete when a mutable rehearsal-shaped run produces the full recognizable
  application from empty and recovers from failure only by full rerun.

### Slice 6 — complete acceptance and independent upgrade/rollback readiness

- Prerequisites: Slice 5 clean rehearsal behavior.
- Touchpoints: Playwright/permissions, deployed smoke, scripted/manual UAT,
  upgrade/rollback runner, final acceptance inventory and documentation.
- Work: prove unchanged UX, configuration, API exactness, Phase 7 behavior,
  source/storage isolation, outage containment, Component-only upgrade/rollback,
  final current-release restoration, and validation readiness.
- Harness in slice: full commands and evidence-schema/manifest self-tests.
- Complete when implementation is clean and ready for the validation
  coordinator. This slice does not freeze or authorize a candidate.

## Verification And UAT Plan

Future command baseline:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo test --locked -p tessara-components-contract
cargo test --locked -p tessara-component-module
cargo test --locked -p tessara-dashboard-module
npm --prefix .\end2end test
.\scripts\smoke.ps1
.\scripts\local-launch.ps1
.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"
```

Sprint-specific contract, boundary, old-input rejection, fresh bootstrap,
failure teardown/rerun, nondisclosure, smoke, and upgrade/rollback commands are
fixed with Slice 1/5 implementation and recorded before readiness.
`local-launch.ps1` remains a root-profile regression check and cannot replace
source-exact Sprint 8A materialization evidence. Deployed acceptance smoke runs
inside SIT.

Manual UAT covers eight scenarios: unchanged Component product experience;
from-empty owner-controlled seed and new Dashboard references; configuration
and diagnostics; Dataset contract/scope/timeout/outage; Dashboard lifecycle and
outage behavior; source/database isolation and old-input rejection; failed
materialization teardown plus full rerun; and Component-only upgrade/rollback.

## Validation, Evidence, Freeze, Failure, And Closeout Plan

- Run complete Test Readiness and complete mutable Candidate Rehearsal. Both
  must pass after the last correction before preflight may freeze a candidate.
- Preflight binds commit/tree/dirty state and every product, test, harness,
  schema migration, fixture, seed, manifest, bootstrap, deployment,
  configuration, contract, and acceptance-inventory identity.
- One fingerprint covers authoritative SIT and UAT. UAT starts only after every
  SIT lane, including deployed acceptance smoke, passes.
- Candidate or tracked harness/fixture/seed/manifest/bootstrap/acceptance change
  invalidates all SIT and UAT. Scoped reruns require unchanged immutable
  candidate/environment receipts and the protocol's explicit justification.
- A candidate-invalidating UAT defect enters Post-SIT Defect Convergence:
  retain failure, forbid passing UAT, harvest safe diagnostics, correct one
  mutable batch, authorize an impact cone, run focused repair validation, then
  rerun complete readiness, rehearsal, preflight, SIT, and UAT.
- Retain append-only hashed receipts under `artifacts/sprint-8a-closeout/` and
  distinguish failed/superseded attempts.
- Closeout requires coordinator-produced `closeout-authorization.json` and a
  healthy freshly seeded topology with the intended current Component release.

## Risks And Controls

| Risk | Prevention | Detection | Recovery |
|---|---|---|---|
| Destructive reset targets wrong resources | resolve exact project/volumes/databases; explicit disposable authorization | readiness target receipt and pre-delete identity check | stop before deletion; no ambiguous target allowed |
| Seed order creates invalid references | owner APIs/read-back; Components before Dashboards | typed-reference and semantic fixture assertions | destroy partial topology and rerun from empty |
| Old reader or payload survives | exact v3 and forbidden-source/route audits | old owner/type/generation/payload negative tests | remove stale path and rerun complete validation cycle |
| Dataset boundary leaks scope/existence | audience exchange; authorization first | known/random API/UI/log/timing conformance | invalidate candidate; broad security retest |
| Dataset outage corrupts authoring | read-only/retry; no pending persistence | timeout/outage browser and database assertions | recover provider and retry preserved browser state |
| Cross-database ownership regression | distinct credentials and source guards | negative credential/SQL/package tests | remove access and rematerialize from empty |
| Dashboard product regression | preserve consumer policy; v3 typed seed | Sprint 7B regression, smoke, Playwright, UAT | invalidate and rerun full chain after correction |
| UI drift/hydration failure | shared shell and baseline captures | viewport/theme/no-JS/console evidence | correct before freeze or pause redesign decision |
| Component outage cascades | generic gateway containment and timeouts | outage/recovery plus unrelated-health probes | restore prior healthy Component release |
| Rollback crosses incompatible schema | compatible release contract and schema audit | upgrade/rollback chronology/read-back | restore compatible Component release; rematerialize only in disposable recovery |
| Evidence is stale or misleading | one fingerprint and hashed prerequisites | receipt/schema/manifest audit | retain superseded evidence and restart required boundary |

## Planning Audit

- Every reconciled Phase 8 and Sprint 8A roadmap clause maps to specifications,
  acceptance, a slice, automated/deployed proof, and manual UAT.
- UI, API, persistence, authorization, integration, deployment, operations,
  compatibility, observability, fresh reset, failure rerun, and rollback are
  covered; legacy product-data migration is explicitly out of scope.
- Happy, negative, boundary, nondisclosure, timeout, outage, destructive-target,
  no-op, failure-rerun, recovery, and rollback cases are explicit.
- Product, seed, fixture, smoke, UAT, Playwright, manifest, deployment, and
  bootstrap changes are paired in dependency-valid slices.
- Roles, topology, commands, data, evidence root, freeze boundary, receipts, and
  invalidation behavior agree with the verification record.
- The planning reconciliation changed documentation only. The subsequent
  implementation goal advances product code, contracts, tests, fixtures,
  deployment inputs, and implementation documentation through the ordered
  slices above.

## Implementation Handoff

All six slices are implemented on `codex/sprint-8a`. The handoff includes the
Components v3 and Dataset v1 contracts, independent Component owner and store,
direct Dashboard consumption, forward-only Core cleanup, from-empty owner seed
and materialization runners, Sprint 8A Compose/catalog inputs, smoke contract,
verified failure teardown, Component-only upgrade/rollback runner, and the
eight planned UAT scenarios. Source-level readiness results and the remaining
environment-bound checks are recorded in the validation record.

The final Core ownership cleanup leaves exactly five transition descriptors:
`tessara.forms`, `tessara.workflows`, `tessara.responses`,
`tessara.datasets`, and `tessara.migration`. Dashboard inventory and
navigation are supplied exclusively by the enrolled `tessara.dashboards`
release/instance and its manifest.

This status authorizes only the next specialized validation workflow. It does
not assert candidate freeze, deployed acceptance, SIT, UAT, or closeout.

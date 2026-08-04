# Sprint 7A Plan: Scoped Analytics And Cross-Module Authorization

- Sprint: `Sprint 7A: Scoped Analytics And Cross-Module Authorization Slice`
- Roadmap authority: `docs/roadmap.md`, lines 1074-1095 at kickoff
- Branch: `codex/sprint-7a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-7a`
- Planning baseline: `634bede53aad2ed46551541f50254c6cc3599dc1`
- Intended deployment profile: a source-exact Sprint 7A copy of the complete
  Sprint 6F reference composition under `deploy/sprint-7a/`
- Intended materialization command:
  `.\scripts\bootstrap-sprint-7a-composition.ps1 -Composition reference`
- Evidence root: `artifacts/sprint-7a-closeout/`
- Status: complete; Candidate 16 passed formal validation and closeout

## Sprint Summary And Outcome

Sprint 7A makes the shipped analytics application obey one scope-safe
authorization contract from Dataset metadata and row preview, through
ComponentVersion metadata and execution, to Dashboard composition and viewing.
The real Dashboard process must present its own identity and exchange the
Core-to-Dashboard grant for an action-bound downstream grant before it can use
the transition-only Core Components provider. The equivalent in-process
Dataset and Component paths must consume the same capability-to-scope decision
shape rather than checking capability presence separately from scope.

The user-visible result is deliberately evolutionary: the existing Dataset,
Component, and Dashboard routes remain native Leptos SSR surfaces, but they
show only authorized assets and rows. A scoped operator receives stable empty,
forbidden, or unavailable states without asset names, linked identities, or
authorization internals; an administrator retains the complete seeded
analytics set.

### Roadmap authority

The complete Sprint 7A roadmap block is authoritative, including its Outcome,
Build, Application UI, and User-testable exit condition. The cross-cutting
constraints additionally require native SSR ownership, stable public errors,
negative operator-scope coverage, typed contracts instead of cross-module
database access, and route/hydration/browser-console verification.

### Approved planning decisions

1. Dataset visibility nodes remain the authored asset-level Organization
   boundary. The existing `public`, `internal`, `restricted`, and
   `confidential` row tiers remain the authored row restriction vocabulary;
   Sprint 7A does not invent a second policy language.
2. A tier capability applies only through its own scope bindings. Dataset
   preview and direct Component execution require the base read capability and
   the row-tier capability to intersect on the same governing Dataset node.
   Dashboard placement execution additionally requires Dashboard visibility to
   contain that same Component-governing node. The implementation must never
   combine capabilities from disjoint subtrees.
3. ComponentVersion visibility continues to derive from its provider-owned
   Dataset major-line binding. Dashboard visibility remains Dashboard-owned.
   A Dashboard placement may disclose or execute a ComponentVersion only when
   the actor's Dashboard grant, the Dashboard scope, the Component grant, and
   the ComponentVersion/Dataset scope all agree.
4. Dashboard embedded Component execution uses the declared `render`
   compatibility action through the Dashboard-owned same-origin route
   `GET /api/dashboards/{dashboard_id}/placements/{placement_id}/render/{kind}`.
   Dashboard loads the stored placement and proxies the request; the browser
   cannot choose a ComponentVersion identifier or call Core directly.
5. The authorization wire remains dual-principal: original actor and
   presenting service are separate, with exact installation, audience,
   dependency binding, functional contract, action, operation, capability
   scope, freshness, and resource basis. No browser cookie, Core bearer token,
   or database credential crosses into Dashboard.
6. Existing global `authorization_revision` and `organization_revision`
   continue to invalidate role, assignment/scope, account, delegation, and
   Organization changes. Touched analytics resource authority gains an exact
   provider revision/assertion (or an equivalent current provider decision)
   so a replayed grant cannot survive an ownership/visibility revision.
7. Dashboard-to-Core requests use a short-lived, per-instance Ed25519-signed
   `ModuleServiceRequestV1`. Core validates the exact method, path, body and
   inbound-grant digests, correlation ID, nonce, installation, instance,
   definition, key, and a maximum 30-second lifetime. Nonces are persisted so
   replay remains rejected across restart. JWTs are not introduced.
8. The 6B non-disclosure profile remains the baseline: known and random
   identifiers must have equal public status/body shape and meet the retained
   warmed timing tolerance. Authorized unknown resources remain distinct only
   after authorization succeeds.
9. Sprint 7A creates a reusable analytics authorization conformance scenario
   set that Phase 8 extractions must rerun unchanged when Dataset or Component
   adapters become process boundaries.
10. No historical upgrade path is required for pre-production fixture data.
   Additive contract/storage changes update the one-file Core and Dashboard
   baselines, and authoritative deployed acceptance starts from a fresh,
   source-exact Sprint 7A composition. Retained Sprint 6F evidence remains
   immutable rollback evidence.

## Scope

### In scope

- Dataset directory/detail/revision metadata, SQL/data preview, published
  materialization reads, and restriction-tier enforcement.
- Component directory/detail/version metadata and table/chart/stat execution
  over a Dataset major-line contract.
- Dashboard directory/detail/editor/viewer metadata, placement resolution, and
  embedded Component execution across the deployed Dashboard process.
- Scope-bound authorization decision/grant production, exchange, validation,
  resource freshness, stable errors, audit-safe diagnostics, and negative
  conformance for the preceding paths.
- Seeded disjoint-scope accounts, authored analytics assets/rows, wrong-service
  fixtures, stale-grant fixtures, and known/random identifiers.
- Native UI empty, restricted, unavailable, and provider-failure states for
  Dataset, Component, and Dashboard surfaces.
- Targeted tests plus smoke, UAT, Playwright, deployment/bootstrap,
  nondisclosure, provenance, receipt, and evidence-manifest updates made in the
  same slices as the behavior they prove.

### Explicitly out of scope

- Sprint 7B resource lifecycle/dependency semantics, rebinding, tombstone
  propagation, and deletion ordering beyond preserving current outcomes.
- Physical Dataset or Component module extraction, database separation, or
  migration of their typed references; those are Phase 8 work.
- A general-purpose attribute policy engine, arbitrary row expressions, or a
  new authoring UI for restriction policy beyond the existing tier fields and
  visibility-node controls.
- Replacing Core as identity, Organization, RBAC, session, or composition
  authority.
- Runtime-loaded remote UI code, iframes, browser bearer tokens, cross-module
  database access, or stored deployment URLs.
- Redesigning the analytics product surfaces, Dashboard layout model, or
  Component presentation types.
- Making deprecated Report/Aggregation/Chart endpoints first-class assets.
- Executing candidate freeze, SIT, UAT, deployment, or closeout during kickoff.

## Current-State Findings And Affected Components

### Platform authorization and security state

- `crates/tessara-module-contract/src/protocol.rs` already defines signed
  `AuthorizationGrantV2`, scope-bound capability bindings, original actor,
  presenting service, audience, declared binding/contract/action, operation,
  authorization and Organization revisions, expiry, replay ID, optional
  resource assertion, and delegation basis. Validation checks all fixed
  context fields but the resource assertion does not yet carry a provider
  authority revision.
- `crates/tessara-api/migrations/001_baseline.sql` advances authorization state
  for role capabilities, role assignments, accounts, credentials, external
  identities, and delegations; Organization mutations advance the Organization
  revision. Analytics ownership/visibility mutations are not presently bound
  to a provider freshness assertion.
- `crates/tessara-api/src/core_security.rs` issues current Core-to-module
  grants, and `crates/tessara-api/src/dashboard_components_adapter.rs`
  validates a Core-to-Dashboard grant before minting a Dashboard-presented
  Component grant. This is the correct dual-principal seam to extend.

### Dataset and Component paths

- `crates/tessara-api/src/datasets/` owns Dataset metadata, revisions,
  visibility nodes, restriction policy compilation, materialization, and
  preview/execution. `crates/tessara-web-datasets/` already exposes visibility
  and restriction authoring.
- Materialized rows already preserve `__restriction_tier`. Component execution
  in `crates/tessara-api/src/components/runtime.rs` filters that tier, but the
  current predicate uses account-wide capability presence. This can form the
  forbidden cross-product when `datasets:read_restricted` and ordinary
  analytics authority are assigned on different subtrees.
- Component metadata scope derives from `dataset_scope_nodes` in
  `crates/tessara-api/src/components/`. Several query paths already filter by
  scope, but the directory, revision, linked metadata, direct execution, and
  typed adapter paths need one consistent decision service and known/random
  nondisclosure contract.
- The large Dataset and Component files remain transitional. Touched
  authorization evaluation should move behind explicit `dto`, `service`, and
  `repo` seams without attempting the Phase 8 process extraction.

### Dashboard boundary and UI

- `crates/tessara-dashboard-module/src/product.rs` validates exact inbound
  Dashboard grants and filters Dashboard records by projected Organization
  scope. `composition.rs` resolves ComponentVersion metadata through Core and
  redacts restricted placements.
- `crates/tessara-api/src/dashboard_components_adapter.rs` already checks the
  real Dashboard instance, current revisions, declared contract/action, and
  Component scope for metadata resolution. It currently authorizes a match on
  any Dataset scope node and must be reconciled with the exact same-governing-
  node intersection rule.
- `crates/tessara-dashboard-ui` reuses
  `tessara-web-component-viewer`; its embedded renderer currently targets
  `/api/components/...` directly. That preserves UI reuse but does not exercise
  the declared Dashboard-presented `render` exchange.
- Dashboard already has contained placement states and generic provider
  failure copy. Sprint 7A must retain those states while ensuring restricted
  metadata never reaches the Dashboard bootstrap or DOM.

### Harness and deployment

- The Sprint 6F complete reference composition deploys Core, gateway,
  Supervisor, Dashboard, and Scoped Records with exact images, Blueprints,
  lockfiles, typed bootstrap, and idempotent no-op proof.
- `scripts/smoke.ps1`, `scripts/uat-sprint.ps1`, the Playwright suites, and the
  Sprint 6A nondisclosure tool provide reusable foundations but lack the full
  Sprint 7A disjoint-scope and Dashboard render-exchange inventory.
- Sprint 7A should clone the source-exact composition profile instead of
  mutating retained Sprint 6F evidence inputs.

### Domain impact disposition

| Domain | Disposition |
|---|---|
| Functional/API | Material: unify analytics access decisions and add action-bound Dashboard execution |
| UI | Material: preserve routes and add stable scoped/forbidden/unavailable states |
| Authorization | Central: dual-principal, scope-bound, fresh, non-disclosing decisions |
| Data/persistence | Additive: provider authority revision/assertion and seeded conformance fixtures; no cross-database access |
| Lifecycle | Preserve current published/superseded/unavailable states; Sprint 7B semantics deferred |
| Deployment | New source-exact Sprint 7A profile using the existing topology |
| Compatibility | Version touched wires; preserve direct Component UI and transition adapter identities |
| Observability | Structured reason codes and correlation IDs internally; public errors remain stable and non-disclosing |
| Rollback | Restore the exact Sprint 6F lockfile/images/receipt; no destructive reverse data migration |

## Functional And Security Specifications

### One analytics authorization decision shape

- Introduce an internal, typed analytics access request/result shared by
  Dataset, Component, and the Core compatibility adapter. Inputs include actor,
  presenting service, installation, action, capability, resource type/ID,
  authored visibility roots, requested restriction tier, and current security
  and provider authority revisions.
- Results must keep capability-to-scope bindings inseparable. They may express
  authorized, unauthorized, not-evaluated, unavailable, and incompatible
  outcomes, but a public caller receives only stable non-disclosing codes and
  messages.
- Global authority is represented as explicit bindings over the installation's
  current Organization roots/descendants, not as an unscoped boolean that can
  be combined with another capability.
- A resource-specific decision is evaluated against current provider metadata
  before identity or linked metadata is disclosed. Known and random IDs follow
  the same restricted path for unauthorized/not-evaluated actors.

### Dataset metadata, preview, and row rules

- Directory, detail, published revision, draft revision, dependency summary,
  and visibility metadata require the applicable Dataset read/manage binding
  on authored visibility roots. Draft/manage operations continue to require
  full coverage; read requires an explicitly documented overlap rule but may
  reveal only nodes inside the actor's authorized projection.
- Dataset preview/execution applies both asset visibility and row tier. Public
  and internal rows retain current baseline access. Restricted or confidential
  rows require their named tier capability and Dataset read capability to
  intersect on at least one identical governing Dataset visibility node.
- Internal columns such as `__restriction_tier`, scope IDs, decision revisions,
  and provider diagnostics never appear as user data.
- Revision lists and linked dependency counts must be computed after
  authorization filtering so hidden revisions, Components, or Dashboards do
  not leak through counts, names, links, empty-table timing, or error copy.

### Component metadata and execution

- ComponentVersion metadata remains governed by the bound Dataset major-line
  visibility. Direct Components UI, Dashboard metadata resolution, and
  execution use the same provider decision service.
- Table, chart, stat, search, sort, filter, paging, and embedded Dashboard
  requests apply the same tier predicate before aggregation, count, pagination,
  or presentation. A filtered response must not leak hidden row counts or
  aggregate contributions.
- Direct known/random ComponentVersion requests by an unauthorized actor are
  byte-equivalent at the public contract. Authorized unknown remains a stable
  not-found/unknown result without conflating provider outage.
- Deprecated analytics endpoints, if a touched shared function reaches them,
  delegate through the canonical decision/execution seam and gain no new
  product behavior or public route.

### Dashboard exchange and viewing

- Core issues a Dashboard grant for the exact browser actor, installation,
  Dashboard instance, Core/Dashboard binding, action, capability scopes,
  current revisions, and short validity window.
- Dashboard validates the grant, then signs the exact Core request with its
  per-instance Ed25519 key and presents both the actor grant and service request
  to obtain an exact downstream `resolve_metadata` or `render` decision. Core
  stores the instance public key, key ID, and fingerprint; Dashboard stores only
  a secret reference to its private key.
- Core re-evaluates the original actor against current role, scope,
  Organization, delegation, and provider authority state. An undeclared
  service, wrong instance/audience, wrong binding/contract/action, wrong
  operation, expired grant, stale revision, altered resource assertion, or
  replay against another resource fails closed.
- The Dashboard viewer's embedded Component requests traverse the placement-
  bound action adapter at
  `/api/dashboards/{dashboard_id}/placements/{placement_id}/render/{kind}`.
  Dashboard resolves the stored ComponentVersion and the shared viewer cannot
  inject a different identifier. Successful execution returns only presentation DTOs. Core-private
  account context, SQL, grants, internal errors, and hidden metadata never
  enter Dashboard bootstrap data or browser-visible diagnostics.
- Dashboard scope alone never grants Component access, and Component access
  alone never grants Dashboard access. A placement is executable only when
  `dashboards:read`, `components:read`, and the required tier capability
  intersect on the same Component-governing node and that node is contained by
  Dashboard visibility. Missing Component base read redacts the placement;
  missing only an elevated tier capability filters those rows. Counts, paging,
  and aggregates are computed after filtering, and zero permitted rows are a
  normal empty state.

### UI, errors, and observability

- Keep `/datasets`, `/components`, and `/dashboards` routes, native SSR,
  hydration ownership, responsive behavior, and existing primary tasks.
- Directory filtering should normally produce a clear empty state. A direct
  restricted ID uses a generic unavailable/forbidden state with no asset name,
  type, scope, linked identity, or policy detail. Provider outage and contract
  incompatibility remain distinct from authorization denial only when that
  distinction is safe to disclose.
- Placement-level restricted states retain the generic “Unavailable placement”
  vocabulary. Authorized provider failures retain recovery guidance.
- Server logs and retained receipts record correlation ID, action, presenting
  service, contract, coarse classification, and revision identities without
  secrets, grants, row values, or hidden resource metadata.

## Data, Compatibility, Rollout, And Recovery

- Version every changed public or cross-boundary schema and reject unknown
  fields. Update provider/consumer golden fixtures and the one exact supported
  protocol tuple together.
- Add only the minimum persistence needed for provider authority freshness and
  deterministic fixtures. Core and Dashboard continue to own separate
  databases, migrations, and runtime identities.
- Authoritative acceptance uses a fresh Sprint 7A composition. The bootstrap
  command must materialize exact roles, disjoint scope assignments, analytics
  fixtures, Dashboard placement references, and provider revisions. An
  unchanged second run must be a verified no-op with stable lockfile/plan and
  one owner receipt per bootstrap declaration.
- Rollout is Core/contract first, then Dashboard with the same locked protocol
  tuple. Mixed old/new protocol combinations fail as incompatible or remain on
  the prior exact tuple; they do not silently weaken authorization.
- On failure, restore the Sprint 6F lockfile, images, Blueprint, and Supervisor
  receipt. Because Sprint 7A authoritative data is disposable pre-production
  fixture data, recovery rebuilds the Sprint 6F composition rather than
  reverse-mutating Sprint 7A security state. Retained Sprint 6F closeout
  evidence proves the rollback source.

## Acceptance Criteria

1. **AC-01 Dataset scope:** a scoped operator sees only authorized Dataset and
   revision metadata; direct blocked known and random IDs have identical public
   outcomes, while admin sees the full seeded set.
2. **AC-02 Dataset rows:** Dataset preview returns public/internal plus only the
   tier rows authorized on the same governing scope; hidden row values, counts,
   tiers, and aggregates never contribute to the response.
3. **AC-03 Component scope:** direct Component directory, version, table,
   chart, and stat execution expose only authorized metadata and rows, with
   filters/paging/aggregation applied after authorization.
4. **AC-04 Dashboard scope:** a scoped operator sees and opens only authorized
   Dashboards; a visible Dashboard redacts each placement whose Component scope
   is not jointly authorized without leaking Component identity.
5. **AC-05 Real boundary:** Dashboard embedded Component execution proves a
   Core-to-Dashboard grant followed by a Dashboard-presented, action-bound
   downstream decision/grant. Receipts assert installation, actor, presenting
   service, audience, binding, contract, action, capability scope, freshness,
   and resource identity.
6. **AC-06 No cross-product:** one fixture actor has Dataset/Component base read
   on subtree A and restricted-tier or Dashboard authority on subtree B. No
   request combines those assignments; every positive result identifies a
   single valid capability/scope binding.
7. **AC-07 Service negatives:** undeclared, wrong-presenting-service,
   wrong-audience, wrong-binding/contract, wrong-action/operation,
   cross-installation, altered-resource, expired, and replay variants fail
   closed and do not disclose or execute the resource.
8. **AC-08 Freshness:** grants issued before role capability, role scope,
   analytics ownership/visibility, Organization, or delegation changes fail as
   stale; a newly evaluated request reflects the new state.
9. **AC-09 Non-disclosure:** blocked known/random Dataset, ComponentVersion,
   and Dashboard identifiers have equal status/body/header shape and pass the
   declared warmed timing profile with retained JSON and SHA-256 evidence.
10. **AC-10 UI states:** Dataset, Component, and Dashboard native routes retain
    usable authorized paths and distinct safe empty, forbidden, unavailable,
    and incompatible states with clean hydration and browser consoles.
11. **AC-11 Compatibility:** deprecated analytics endpoints touched by shared
    code remain adapter-only; direct Component UI stays compatible; current
    Dashboard source/build independence, module isolation, Blueprint, lockfile,
    and no-op composition behavior remain intact.
12. **AC-12 Reusable proof:** the analytics authorization conformance runner is
    provider-neutral at its typed adapter boundary and is named as a Phase 8
    extraction entry gate.
13. **AC-13 Deployment:** exact candidate images, config, migrations, security
    revisions, fixture identities, routes, and receipt chain are proven from a
    fresh Sprint 7A apply; unchanged materialization is an idempotent no-op.
14. **AC-14 Recovery:** a controlled provider outage yields contained UI and
    stable diagnostics, canonical service restoration returns healthy behavior,
    and the documented Sprint 6F rollback input remains usable.

## Traceability Matrix

| Roadmap clause | Specification / acceptance | Slice | Automated proof | Manual UAT |
|---|---|---|---|---|
| Scope-safe execution across Dashboard and typed adapters | One decision shape; AC-02–AC-05 | 1–4 | Contract, integration, deployed exchange | UAT-04, UAT-05 |
| Restriction rules for Dataset previews, Component execution, Dashboard viewing | Dataset/Component/Dashboard rules; AC-02–AC-04 | 2–4 | Tier, aggregate, placement tests | UAT-02–UAT-05 |
| Scoped metadata for datasets, revisions, component versions, composition, linked assets | Metadata rules; AC-01, AC-03, AC-04 | 2–4 | API shape and count tests | UAT-01, UAT-03, UAT-04 |
| Installation, actor, Dashboard service, contract/action, grants, freshness, audience | Dashboard exchange; AC-05 | 1, 4 | Signed fixture and live receipt assertions | UAT-05 |
| Blocked rows/entities/metadata/content | Non-disclosure rules; AC-01–AC-04, AC-09 | 2–5 | Negative matrices and retained profile | UAT-02, UAT-06 |
| Mixed capabilities on disjoint subtrees | No cross-product; AC-06 | 2–5 | Dedicated integration/conformance fixture | UAT-06 |
| Undeclared and wrong audience/action services | Exact service negatives; AC-07 | 1, 4, 5 | Protocol and live adapter negatives | UAT-07 |
| Stale after role/scope/ownership/delegation changes | Freshness; AC-08 | 1–5 | Mutation/replay integration tests | UAT-08 |
| Known versus random under 6B profile | Non-disclosure; AC-09 | 5 | Retained shape/timing runner | UAT-06 |
| Deprecated endpoints adapter-only | Compatibility; AC-11 | 2, 3 | Boundary/static audit | UAT-09 |
| Move Dataset/Component execution toward extraction boundaries | Typed service/repo seams; AC-12 | 1–3 | Boundary/conformance checks | UAT-09 |
| Clear non-leaking empty/unavailable/forbidden states | UI/error rules; AC-10, AC-14 | 2–6 | SSR, Playwright, outage smoke | UAT-01–UAT-05, UAT-10 |
| Existing surfaces remain usable | UI preservation; AC-10, AC-11 | 2–6 | Full Playwright/UAT regression | UAT-01–UAT-05 |
| Understandable scoped/cross-module failures | UI/error rules; AC-10, AC-14 | 4–6 | UI copy and degraded-state tests | UAT-05, UAT-10 |
| Scoped operator can use all three analytics surfaces | End-to-end exit; AC-01–AC-05 | 6 | Deployed acceptance + browser | UAT-01–UAT-05 |
| Administrator sees full seeded analytical set | Admin control; AC-01–AC-04 | 2–6 | Exact fixture inventory assertion | UAT-11 |
| Phase 8 extractions rerun proof | Reusable proof; AC-12 | 5 | Provider-neutral conformance self-test | UAT-09 |

## Manual UAT Plan

- **UAT-01 — Scoped Dataset catalog:** scoped operator opens Dataset directory,
  detail, and published revision; only subtree-A assets/nodes and safe counts
  appear. Direct blocked ID shows generic unavailable state.
- **UAT-02 — Dataset tier rows:** the same operator previews an authored
  Dataset containing all four tiers. Visible values/counts match the allowed
  same-scope tiers; a recognizable blocked value never appears.
- **UAT-03 — Component execution:** operator opens table, chart, and stat
  Components. Results and aggregates exclude blocked rows; a blocked
  ComponentVersion is absent and direct access is non-disclosing.
- **UAT-04 — Dashboard viewing:** operator opens an authorized Dashboard and
  observes authorized placements execute normally while a disjoint-scope
  placement is a generic unavailable tile with no Component metadata.
- **UAT-05 — Cross-boundary recovery:** capture the successful Dashboard render
  exchange, stop or isolate the compatibility provider, observe contained
  unavailable copy, restore it, retry, and confirm healthy authorized rendering.
- **UAT-06 — Disjoint and known/random negatives:** use the mixed-scope account
  and verify no capability cross-product across all three surfaces; compare
  known and random restricted direct routes.
- **UAT-07 — Service misuse:** use the test client to submit wrong service,
  audience, contract, action, and replay cases; confirm stable denial and no
  resource detail.
- **UAT-08 — Freshness:** issue a valid request, change role/scope, Dataset
  visibility, and delegation state through normal admin/product paths, then
  verify the retained grant fails and a fresh request reflects the update.
- **UAT-09 — Compatibility:** exercise direct Component UI and the transitional
  adapter; confirm deprecated endpoints gained no navigation/product surface
  and the Phase 8 conformance command is documented.
- **UAT-10 — Safe states and responsiveness:** verify empty, forbidden,
  unavailable, and recovered states at desktop and narrow widths with keyboard
  navigation, SSR usefulness, no hydration mismatch, and no console error.
- **UAT-11 — Administrator control:** administrator sees every seeded Dataset,
  revision, ComponentVersion, Dashboard, placement, and authorized row tier.

## Automated And Integration Test Plan

### Targeted checks

- `cargo test -p tessara-module-contract --locked`
- `cargo test -p tessara-components-contract --locked`
- `cargo test -p tessara-api --lib --locked`
- `cargo test -p tessara-api --test demo_flow --locked`
- `cargo test -p tessara-api --test modules --locked`
- `cargo test -p tessara-dashboard-module --locked`
- `cargo test -p tessara-dashboard-ui --features ssr --locked`
- `cargo test -p tessara-web-component-viewer --features ssr --locked`
- `cargo check -p tessara-dashboard-ui --target wasm32-unknown-unknown --features hydrate --locked`
- `cargo check -p tessara-web --target wasm32-unknown-unknown --features hydrate --locked`
- `.\scripts\run-analytics-authorization-conformance.ps1 -SelfTest`
- `.\scripts\validate-analytics-nondisclosure.ps1 -SelfTest`
- `.\scripts\verify-module-sdk-boundaries.ps1`
- `.\scripts\verify-sprint-6e-boundaries.ps1`

### Required baseline and deployed checks

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --locked`
- `npm --prefix .\end2end test`
- `.\scripts\local-launch.ps1` for a non-destructive developer refresh only;
  authoritative acceptance instead uses the source-exact Sprint 7A profile.
- `.\scripts\bootstrap-sprint-7a-composition.ps1 -Composition reference`
- Repeat the preceding command unchanged with `-SkipBuild` and assert the exact
  lockfile/plan and no-op receipt.
- `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8086"`
- `.\scripts\smoke-sprint-7a.ps1 -BaseUrl "http://127.0.0.1:8086"`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:8086"`
- `.\scripts\uat-sprint-7a.ps1 -BaseUrl "http://127.0.0.1:8086"`
- `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8086"`
- `.\scripts\validate-analytics-nondisclosure.ps1 -BaseUrl "http://127.0.0.1:8086"`
- `cargo audit --quiet`
- `.\scripts\verify-markdown-links.ps1`
- `git diff --check`

The Sprint 7A-specific commands are tracked implementation outputs. Their
self-tests, exact identities, and change rationale are recorded in
`docs/sprints/sprint-7a-test-change-log.md`; authoritative execution remains a
post-freeze SIT/UAT responsibility.

## Validation, Evidence, And Closeout-Authorization Plan

1. Reconcile the completed implementation with
   `docs/sprints/sprint-7a-verification.md`; no roadmap clause may remain
   without automated, deployed-smoke, and manual proof.
2. Run `tessara-validation-preflight` from a clean candidate branch. Record
   tool versions, reset authorization, acceptance inventory, exact Sprint 7A
   Compose/configuration digest, migration baseline, catalog/Blueprint/
   lockfile identity, expected provenance, account/role/fixture identities,
   and evidence-root mode.
3. Freeze one candidate commit/tree with `dirty=false`. The candidate
   fingerprint includes product, contracts, tests, fixtures, migrations,
   manifests, deployment/bootstrap, smoke, UAT, Playwright, and acceptance
   inventory. Produce passing `preflight-result.json` and `candidate.json`.
4. Run authoritative SIT. Static/boundary, Rust workspace, source-exact fresh
   deployment, idempotent materialization, contract exchange, conformance,
   nondisclosure, Playwright, and **deployed acceptance smoke** are SIT lanes.
   Produce all lane receipts and `sit-result.json` before UAT.
5. Run scripted and every manual UAT scenario only after authoritative SIT
   passes, using the same candidate fingerprint and declared environment.
   Produce `uat-result.json` and retained scenario evidence.
6. The coordinator validates prerequisite hashes, failure chronology,
   evidence-manifest completeness, canonical reference composition, source
   provenance, health, and every traceability mapping before writing
   `closeout-authorization.json`.
7. Retain receipts, append-only logs, raw reports, screenshots where visual
   behavior matters, JSON/SHA-256 nondisclosure evidence, exact authorization
   exchange evidence, fixture read-back, no-op receipt, migration identities,
   image labels/digests, console/hydration results, and final topology health
   beneath `artifacts/sprint-7a-closeout/`.
8. A candidate or tracked harness/inventory change invalidates all SIT and UAT.
   A material shared-environment change invalidates the affected lane and
   downstream phases. A lane-local setup failure before assertions reruns that
   lane. Complete immutable raw results with only finalization failure rerun
   finalization. A failed assertion requires a complete authoritative rerun of
   its lane; upstream reuse requires matching fingerprints and an explicit
   non-impact rationale. Any product correction refreezes and restarts all SIT
   and UAT.
9. Closeout cannot originate a missing acceptance check. Missing coverage
   reopens validation at the coordinator-selected boundary.

## Ordered Implementation Slices

| Slice | Prerequisites | Coherent increment and touchpoints | Tests/harness in same slice | Completion criterion |
|---|---|---|---|---|
| 1. Freeze access and exchange contracts | Approved plan | Extend/version analytics access, provider freshness/resource assertion, and Dashboard render DTOs in `tessara-module-contract` and `tessara-components-contract`; add golden fixtures and stable errors | Contract unit, fixture, wrong-field, signature, audience/action/revision tests | Provider and consumers compile against one exact tuple; every malformed/stale binding fails closed |
| 2. Core analytics decision boundary | Slice 1 | Add bounded `dto/service/repo` seams for scope decisions; bind Dataset/Component metadata and provider revisions in Core migration/state; implement no-cross-product tier evaluation | Core unit/integration tests plus seeded disjoint-scope fixtures | Dataset decisions are current, typed, scope-bound, and non-disclosing before metadata/data access |
| 3. Dataset and direct Component vertical slice | Slice 2 | Apply decisions to Dataset directory/detail/revisions/preview and Component metadata/table/chart/stat execution; preserve native SSR and add safe empty/direct-route states | API, SSR, wasm, Dataset/Component Playwright, fixture and acceptance-manifest updates | Direct Dataset and Component UI satisfy AC-01–AC-03 and AC-10 |
| 4. Real Dashboard render boundary | Slices 1–3 | Add Dashboard-presented `resolve_metadata`/`render` exchange and execution proxy/adapter; validate joint Dashboard/Component scope and freshness; parameterize shared viewer endpoint safely; preserve redacted placement UI | Dashboard/Core integration, provider/consumer contract, SSR/wasm, dashboard Playwright, outage tests | Embedded executions prove the real boundary; blocked placements disclose no metadata or rows |
| 5. Security conformance and nondisclosure | Slices 2–4 | Add provider-neutral analytics conformance runner, wrong-service/replay/stale matrices, known/random shape/timing tool, structured audit assertions | Self-tests, optimized timing run, retained evidence schema/publication rollback tests | AC-06–AC-09 and AC-12 have repeatable source and deployed assertions |
| 6. Source-exact deployment and acceptance harness | Slices 1–5 | Add `deploy/sprint-7a`, exact catalog/Blueprint/lockfile inputs, idempotent bootstrap, smoke, focused UAT, manual scripts, Playwright inventory, provenance and receipt capture | Fresh build/apply, unchanged no-op, full smoke/UAT/Playwright, rollback restore rehearsal | Complete reference composition passes future preflight-ready acceptance inventory and remains healthy |
| 7. Candidate reconciliation | Slice 6 | Update roadmap/progress only after implementation and validation authorization; no new executable acceptance work | Full planned command set and evidence audit | One clean candidate has complete traceability and is ready for validation/closeout, not prematurely closed |

## Dependencies, Assumptions, Questions, And Blockers

### Dependencies

- The completed Sprint 6F Blueprint, Supervisor, source-exact composition, and
  retained closeout evidence remain the deployment and rollback base.
- Core security revisions, Organization hierarchy expansion, Dashboard module
  identity, typed Component compatibility contract, native shared Component
  viewer, and existing restriction-tier materialization remain available.
- Docker/Compose, PostgreSQL, Rust, wasm target, Node/Playwright, PowerShell,
  signing fixtures, and disposable local ports/databases are available during
  implementation validation.

### Assumptions resolved from repository evidence

- “When those rules are authored” refers to existing Dataset visibility nodes
  and restriction tier fields, plus Dashboard visibility nodes; no new policy
  authoring model is required.
- “Ownership revision” for this slice means the current provider authority for
  the governing analytics resource/visibility binding, not Sprint 7B lifecycle
  ownership or a new universal ownership subsystem.
- The reference application can add dedicated test accounts/roles without
  exposing their passwords in retained receipts. UAT credentials remain
  supplied out of band even though local development defaults exist.
- The current exact protocol tuple may be versioned atomically because the
  application is pre-production and the source-exact profile upgrades Core and
  Dashboard together.

### Open questions and blockers

- None. The approved implementation uses same-root intersection, provider-owned
  revisions, a placement-bound Dashboard proxy, and per-instance asymmetric
  signed requests. Direct browser/Core rendering, JWTs, and a new row-level
  Organization policy model are explicitly out of scope.

## Risk Register

| Risk | Prevention | Detection | Recovery |
|---|---|---|---|
| Capability/scope cross-product leaks rows | Inseparable bindings and same-governing-node intersection | Disjoint-subtree fixture across all surfaces | Fail closed; refreeze after correction |
| Metadata/count leak before authorization | Provider decision precedes identity, joins, counts, bootstrap | Known/random exact-shape tests and DOM assertions | Generic restricted result; rerun lane |
| Dashboard grant replay or confused deputy | Exact dual-principal audience/binding/action/resource/revision validation | Wrong-service/action/audience/replay matrix | Reject, retain diagnostic receipt, rotate/refreeze if contract changed |
| Stale resource visibility survives a grant | Provider authority revision/current decision | Mutate-then-replay tests | Re-evaluate; invalidate candidate if schema/harness changes |
| Filtering after aggregation leaks blocked contributions | Apply tier/scope predicate in source query before count/aggregate/page | Recognizable blocked values and exact aggregate fixtures | Correct execution seam and rerun all SIT/UAT |
| UI reveals restricted names in bootstrap/DOM/errors | Metadata-free restricted DTOs and generic copy | SSR HTML, browser DOM, network, console tests | Redact and refreeze candidate |
| Adapter expansion becomes permanent coupling | Versioned narrow contract and boundary audit | Dependency/static checks | Keep direct calls isolated; Phase 8 replaces adapter |
| Source-exact profile mutates Sprint 6F evidence | New Sprint 7A deployment/evidence paths | Path/provenance audit | Restore retained Sprint 6F inputs from its commit |
| Provider outage strands Dashboard | Contained placement states and retry | Fault-injection smoke/UAT | Restore provider and canonical topology; rollback lockfile if needed |
| Large suites produce ambiguous partial evidence | Phase receipts, heartbeats, fail-late sibling checks | Receipt/manifest audit | Inspect retained state; rerun minimum safe authoritative boundary |

## Planning Audit

- Every roadmap Build, UI, and exit-condition clause maps to specifications,
  acceptance criteria, a dependency-ordered slice, automated proof, and manual
  UAT in the traceability matrix.
- UI, API, persistence, authorization, integration, deployment, compatibility,
  lifecycle, observability, recovery, and rollback were considered; deferred
  lifecycle/extraction work is explicit.
- Happy paths, negative paths, mixed-scope boundaries, known/random
  nondisclosure, stale state, provider outage, recovery, and rollback are
  covered.
- Product changes and their test, fixture, smoke, UAT, Playwright, deployment,
  and evidence updates are paired in the same slices.
- Commands, topology, roles, fixture classes, candidate rules, and evidence
  destination are concrete. Credentials are intentionally not retained.
- The plan agrees with `docs/sprints/sprint-7a-verification.md`.
- Kickoff changed only this plan, the seeded validation record, and the progress
  entry in the Sprint 7A worktree. Product implementation has not started.

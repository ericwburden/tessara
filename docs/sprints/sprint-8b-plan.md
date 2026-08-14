# Sprint 8B: Dataset Module Separation Slice

Status: Kickoff complete; implementation has not started.

- Branch: `codex/sprint-8b`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-8b`
- Base commit: `5856576f6d25e510d8ced466a8942cc8a313eeac`
- Roadmap authority: `Sprint 8B: Dataset Module Separation Slice (Next)`
- Validation policy: `tessara-validation-v2`
- Validation record: `docs/sprints/sprint-8b-verification.md`
- Validation contract: `docs/sprints/sprint-8b-validation-contract.json`
- Required extraction playbook: `docs/architecture/module-extraction-playbook.md`

## Sprint Summary, Outcome, And Roadmap Authority

Sprint 8B extracts Datasets from Core into a separately built and deployed
`tessara.datasets` full-stack module. The module becomes the sole owner of
Dataset authoring, revision lifecycle, materialization, preview and execution,
persistence, configuration, diagnostics, routes, assets, capabilities, seed,
and operational behavior. It consumes submitted Response rows, FormVersion
schema, and Core scope-directory facts only through exact versioned provider
contracts and never through another owner's database or credentials.

The reference application is pre-production and its seed data is disposable.
The extraction therefore uses a source-exact rebuild from empty databases.
Dataset and downstream Component/Dashboard state is recreated through owning
bootstrap contracts with typed read-back identities. There is no migration of
old Core Dataset rows or references, no old-to-new mapping, and no compatibility
adapter after cutover.

The complete roadmap block is authoritative:

- Outcome: Datasets is independently deployed and consumes source data through
  explicit provider contracts.
- Build: move all Dataset product and operational ownership; build a fresh
  database and downstream seed from owning contracts; remove the Core adapter;
  replace Response and other source-table reads; preserve Component contracts
  and scoped execution; retain Dataset ownership of batch/catalog/template
  semantics; remove cross-owner seed/storage access; and cover retries,
  outages, scope, and compatibility.
- UI: preserve the Dataset directory, authoring, preview, status,
  configuration, and diagnostics experience through the shared shell.
- Exit condition: materialize and preview from a provider contract, execute a
  Component over the Dataset, and view it on a Dashboard across independent
  modules.

## Scope

### In scope

- Create Module Definition `tessara.datasets`, initial release `1.0.0`, one
  selected Module Instance, isolated Dataset database, runtime identity
  `datasets-runtime`, and migration identity `datasets-migration`.
- Create `tessara-dataset-module` and `tessara-dataset-ui` ownership surfaces,
  reusing `tessara-module-contract`, `tessara-module-runtime`,
  `tessara-module-ui`, and `tessara-module-testkit`.
- Move Dataset product APIs, typed DTOs, domain rules, revision history,
  publication, compatibility review, QuerySpec execution, materialization,
  batch operations, catalog/template behavior, persistence, migrations,
  module-owned bootstrap, configuration, status, health, diagnostics, SSR,
  hydration, lifecycle, routes, and product assets out of Core.
- Advance `tessara.datasets.dataset-major-line` to exact version `2.0.0` and
  resource type `tessara.datasets.dataset_major_line`, owned by a Dataset Module
  Instance. Update Components and every first-party consumer in the same slice.
- Define Module Instance-owned Dataset and DatasetRevision references with
  resource types `tessara.datasets.dataset` and
  `tessara.datasets.dataset_revision`; reject Core-owned transition resource
  types and v1 wire shapes.
- Introduce provider-owned exact contracts for current source dependencies:
  `tessara.responses.submitted-response-export` `1.0.0`,
  `tessara.forms.form-version-schema` `1.0.0`, and the policy-neutral
  `tessara.core.scope-catalog` and `tessara.core.principal-display-catalog`
  `1.0.0`. Dataset declares bindings
  `tessara.datasets.response-export`,
  `tessara.datasets.form-version-schema`, and
  `tessara.datasets.scope-catalog` plus
  `tessara.datasets.principal-display-catalog`.
- Add a Response-owned monotonically increasing change sequence and durable
  export/change-log cursor because the current Response/analytics tables do not
  provide an authoritative incremental synchronization boundary. The cursor
  covers submitted-row/value upserts, status changes, corrections, redactions,
  and tombstones; wall-clock timestamps such as `created_at` are payload facts
  only and never the correctness cursor.
- Exchange audience-bound, request-bound authorization at every source and
  consumer hop. Preserve capability scope, restriction tiers, nondisclosure,
  bounded timeouts, incompatibility results, and recovery.
- Add Dataset-owned resource observation, Form-source usage, Core Operations
  readiness, and application-summary provider actions. Cut every reverse
  consumer to those actions so Forms, Operations, app summary, and generic
  typed-reference resolution no longer read Dataset storage.
- Add a Dataset-owned synchronous refresh action, invoked by create, publish,
  bootstrap, and explicit manager refresh. Product GET/preview/execute reads
  never mutate source state; they serve the last successful materialization
  with exact freshness/degradation metadata.
- Publish immutable Dataset public-route/action declarations and durable
  module-owned replay receipts for every persistent product mutation. SQL
  preview remains a non-persisting action; all other writes consume the
  gateway-forwarded/generated idempotency key.
- Materialize a clean Dataset database and its `dataset_materialized` schema,
  then seed Dataset before Components and Dashboards. Pass Dataset v2 typed
  read-back references to Component bootstrap and Component references to
  Dashboard bootstrap.
- Remove Dataset product schema, enum, functions, triggers, routes, handlers,
  DTOs, direct provider, web route composition, seed writes, static capability
  registration, transition catalog entry, and compatibility adapter from Core.
- Add a disposable Sprint 8B Compose profile, owner health/provenance checks,
  deterministic failed-apply containment, exact teardown, from-empty recovery,
  semantic second-run no-op, and Dataset-only upgrade/rollback/restoration.
- Update fixtures, smoke, Playwright, UAT, manifests, catalogs, Blueprint,
  bootstrap, and validation runners in the same slices as the behavior that
  makes them stale.

### Explicitly out of scope

- Preserving or migrating Core-owned Dataset product rows, materialized tables,
  Dataset references, or downstream seeded product rows.
- Old-to-new mapping ledgers, rebinding receipts, dual reads/writes, online
  cutover, legacy cutover checkpoint resume, or a read-only Core Dataset
  adapter. This does not prohibit retrying a new provider export through
  attempt-private page staging rooted at the last published source cursor.
- Acceptance of `core_installation` Dataset references,
  `tessara.transition.dataset*` resource types, or Dataset contract v1 payloads
  after cutover.
- Physical extraction of Responses, Forms, hierarchy, identity, Components,
  or Dashboards. Only narrow provider contracts necessary for Dataset
  isolation are added to their current owners.
- New Dataset product operations other than the extraction-required explicit
  synchronous refresh/status action; visualization behavior, authoring
  workflow, revision policy, batch semantics, and general UI redesign remain
  out of scope.
- A generic external source marketplace, user-defined connectors, streaming
  ingestion platform, or cross-sprint migration/import UX.
- Multi-instance Dataset selection or external module packaging/publishing.

## Fixed Identities And Contracts

| Surface | Sprint 8B identity |
| --- | --- |
| Module Definition | `tessara.datasets` |
| Initial / intended release | `1.0.0` |
| Prior-compatible upgrade fixture | separately built `0.9.0` |
| Transition identity removed | `tessara.datasets` |
| Runtime / migration identity | `datasets-runtime` / `datasets-migration` |
| Product database | isolated disposable Dataset PostgreSQL database |
| Dataset major-line contract | `tessara.datasets.dataset-major-line` `2.0.0` |
| Dataset resources | `tessara.datasets.dataset`, `.dataset_revision`, `.dataset_major_line` |
| Component consumer release / binding | immutable `tessara.components` `1.1.0` / `tessara.components.dataset-major-line` exact v2 |
| Dashboard release held fixed | `tessara.dashboards` `3.0.1` |
| Response source binding | `tessara.datasets.response-export` |
| Response synchronization | provider-owned opaque monotonic cursor over a stable snapshot/change log |
| Form schema binding | `tessara.datasets.form-version-schema` |
| Scope binding | `tessara.datasets.scope-catalog` |
| Principal display binding | `tessara.datasets.principal-display-catalog` |
| Dataset reverse-consumer contracts | `tessara.datasets.source-usage`, `.operational-status`, `.resource-observation` `1.0.0` |
| Dataset refresh authority | existing `datasets:manage`; no new capability |
| Dataset navigation | `tessara.datasets.navigation`, Blueprint order `6` |
| Test-change record | `docs/sprints/sprint-8b-test-change-log.md` |
| Browser acceptance inventory | `end2end/acceptance-manifest.json` schema v2, exact test identities rather than a copied count |
| Evidence root | `artifacts/sprint-8b-closeout/` (ignored) |

The release catalog and materialization read-back, not copied UUIDs or counts,
are the source of truth for the selected Module Instance and seeded resources.

## Current-State Findings And Affected Components

### Current ownership

- `crates/tessara-api/src/datasets/` owns Dataset CRUD, revision lifecycle,
  compatibility review, SQL planning, materialization, scope evaluation, and
  direct reads of Dataset, Form, hierarchy, and analytics tables.
- `crates/tessara-api/src/dataset_provider.rs` is the Core compatibility
  provider consumed by Components. It queries Core Dataset tables and
  materialized tables directly.
- `crates/tessara-api/migrations/001_baseline.sql` owns the
  `dataset_materialized` schema, `dataset_revision_status` enum, `datasets`,
  `dataset_scope_nodes`, `dataset_tags`, `dataset_revisions`,
  `dataset_major_materializations`, `dataset_sources`, and `dataset_fields`,
  plus Dataset revision/authority functions and triggers.
- Dataset materialization reads `analytics.submission_fact`,
  `analytics.submission_value_fact`, `analytics.node_dim`, Forms/FormVersions,
  Form fields/sections, nodes/node types, and accounts from Core storage.
- Response rows expose `created_at`, `submitted_at`, and an audit-derived
  `last_modified_at`, but no authoritative monotonic export cursor exists.
  `crates/tessara-api/src/analytics.rs` currently deletes and rebuilds the
  Response analytics projection, so timestamp maxima cannot safely detect
  corrections, late rows, deletions/redactions, or tied timestamps.
- `crates/tessara-api/src/app_summary.rs` counts Dataset rows directly;
  `crates/tessara-api/src/operations.rs` derives Dataset readiness by joining
  Dataset, Form, and analytics tables; and `crates/tessara-api/src/forms/mod.rs`
  reads `dataset_sources` for the Form-detail reverse-link table. These are
  active reverse consumers, not incidental test residue.
- `crates/tessara-api/src/modules/reference.rs` constructs, resolves, and
  observes transition Dataset references by reading Dataset tables, while
  `crates/tessara-api/src/analytics_authorization.rs` owns Dataset governing-
  node/restriction-tier policy. Both product responsibilities move to Dataset.
- `crates/tessara-web-datasets/src/api.rs` directly calls Core `/api/me`, Forms,
  hierarchy, user-directory, and FormVersion-render routes. The independent
  browser must instead use signed ShellContext plus Dataset-owned product
  endpoints whose server-side clients call exact provider actions.
- `tessara-data-ops` and `tessara-web-data-ops` remain policy-neutral shared
  operation AST/editor libraries used by both Dataset and Components. Core API
  drops its dependency; Dataset compiler, authorization, and materialization
  semantics are not moved into those shared crates.
- `crates/tessara-datasets-contract` is v1 and deliberately enforces
  `CoreInstallation` ownership and `tessara.transition.dataset_major_line`.
  It must advance exactly rather than gain permissive dual-owner parsing.
- `crates/tessara-web/src/routes/datasets.rs` composes the current Core-owned
  Dataset pages from `crates/tessara-web-datasets`; its APIs target Core paths.
- `crates/tessara-module-contract/tests/fixtures/transition-datasets-v1.json`
  owns the in-process descriptor, transition resource types, capabilities,
  routes, dependencies, and navigation contribution.
- `crates/tessara-component-module` consumes Dataset v1 through a real process
  boundary, but its manifest, client, bootstrap input, product persistence, and
  fixtures still require the Core-owned v1 reference.
- `deploy/sprint-8a` materializes Core, Components, and Dashboards and currently
  declares Dataset as a Core-provided contract. Sprint 8B must replace that
  topology rather than extend the Core adapter.

### Affected implementation areas

- Workspace and build graph: root `Cargo.toml`, `Cargo.lock`, Leptos metadata,
  new Dataset module/UI/provider-contract crates, and removal of root web/Core
  product dependencies.
- Core: router and native document inventory, Dataset modules/provider, auth
  static data, module catalog/navigation/destination/reference seams, demo seed,
  migrations, transition fixture, tests, and generic gateway registration.
- Dataset owner: module manifest, store/migrations, API, UI, bootstrap,
  provider clients, runtime providers, diagnostics, configuration, lifecycle,
  browser assets, and unit/integration tests.
- Providers: narrow versioned Response export, FormVersion catalog/schema,
  Core scope-catalog, and principal-display-catalog contracts and handlers; no
  Dataset-specific Core UI/control branch.
- Consumers: Components contract/client/store/bootstrap/manifest and the
  downstream Dashboard bootstrap chain; Core Forms reverse usage, Core
  Operations Dataset readiness, application summary counters, and generic
  typed-resource observation.
- Current local/developer/test entry points: `scripts/local-launch.ps1`,
  `scripts/local-refresh-api.ps1`, active Core integration tests, and every
  Sprint 7A fixture that writes Dataset storage. Active tests are converted to
  owner APIs; historical fixtures remain quarantined and may only support
  negative/history assertions.
- Deployment and validation: `deploy/sprint-8b`, materialization, failure,
  upgrade, smoke, acceptance, readiness, rehearsal, preflight, SIT, UAT, and
  evidence-publication scripts.
- Browser acceptance: Dataset, Component, Dashboard, modules, permissions, and
  module-UI visual suites plus current Dataset acceptance predicates.

## Core Subtraction Inventory

Every row is removed or reduced to a policy-neutral generic registration.

| Core surface | Required subtraction proof |
| --- | --- |
| Dataset schema | No Dataset tables, enum, materialized schema, functions, triggers, or baseline seed in Core migrations |
| Product API | No `datasets` route merge, Core Dataset handlers/DTOs/review/materialization, or Dataset product SQL |
| Compatibility provider | No `dataset_provider` routes, Core Dataset-major-line service declaration, adapter, reader, or writer |
| Product web | No Core Dataset route components, feature guard branch, raw Dataset API client, or Dataset-specific shell composition |
| Product domain | Dataset policy/execution moves from Core-linked crates to the module-owned graph; Core does not link it for product behavior |
| Dataset authorization | Delete `analytics_authorization` Dataset scope/tier policy and every Core import of Dataset scope loaders; Dataset evaluates its own governing nodes and restriction tiers from projected grants |
| Reverse reads | App summary, Operations, and Forms cannot query Dataset tables; each consumes an exact Dataset action and represents unavailable separately from empty |
| Typed-reference resolution | Core transition registry contains no Dataset resource kind, parser, lifecycle loader, revision reader, or observation SQL; generic dispatch reaches Dataset `resolve` |
| Composition/bootstrap | Core composition and demo code cannot construct Dataset rows, revisions, major lines, or copied IDs; it passes provider fixtures and consumes signed owner read-back only |
| Transition catalog | Delete active `transition-datasets-v1` registration and ensure `tessara.datasets` is enrolled exactly once as a real module |
| Capabilities/grants | Remove Core static Dataset capability creation/default grants; Manifest enrollment and Blueprint role projection own them |
| Seed/fixtures | Core demo and preparation scripts cannot write Dataset or materialized rows; historical fixtures remain clearly historical only |
| Navigation/inventory | No synthesized Dataset destination, copied ordering branch, or transition/release overlap |
| Compatibility shapes | Reject v1 contract, `core_installation` owner, and all `tessara.transition.dataset*` resource types |

Searches for `tessara.datasets`, Dataset table names, Dataset capabilities, and
Dataset routes must classify every remaining match as module ownership, generic
registration, provider-contract declaration, historical fixture, or negative
assertion.

## Provider And Consumer Edge Inventory

| Edge | Current coupling | Target public boundary | Authorization / outage / recovery |
| --- | --- | --- | --- |
| Responses -> Datasets | direct analytics Response facts and values; no authoritative change cursor | Response-owned `submitted-response-export` v1 with scope-bound checkpoint, start/rebase, stable snapshot, and page actions over immutable aggregate upsert/tombstone envelopes | Dataset exchanges a short-lived grant for the Response audience; provider applies scope/nondisclosure; unchanged head is a no-op; failures preserve the last published projection/cursor; expired cursors use an authenticated full-snapshot rebase |
| Forms -> Datasets | direct Forms/FormVersion/field/section tables and browser calls to `/api/forms` plus FormVersion render | Forms-owned `form-version-schema` v1 catalog/schema actions with exact field IDs, keys, labels, types, options, layout/provenance, immutable version identity, and content digest | grant is bound to requested FormVersion; unknown and unauthorized IDs are indistinguishable; incompatibility blocks authoring/publish/refresh without partial state |
| Core scope -> Datasets | direct node/node-type queries and browser `/api/nodes` | policy-neutral `scope-catalog` v1 for authorized node validation/display plus a requested-set revision/content digest | Dataset stores opaque node IDs and sanitized display facts; outage blocks scope-changing writes/refresh but does not erase the last good materialization |
| Core identity -> Datasets | browser `/api/me` and `/api/admin/users`; account join for updater display names | signed ShellContext for current actor plus policy-neutral `principal-display-catalog` v1 for authorized display-name choices and a requested-set digest | no email/credential export; actor and random/unauthorized principal requests are nondisclosing; names are cached only as bounded display facts |
| Dataset -> Dataset | direct same-database upstream joins | Dataset module's own v2 major-line reference plus tracked upstream materialization generation | cycle checks, revision/major pinning, scope intersection, and topological refresh remain owner-local; one transaction promotes the affected dependency closure or none of it |
| Datasets -> Components | Core compatibility provider and v1 Core-owned references | Dataset v2 catalog/schema/distinct/compatibility/execute actions and Module Instance-owned typed major-line references consumed by immutable Component `1.1.0` | preserve shared governing-node semantics, correlation identity, restriction tiers, known/random nondisclosure, timeout, incompatibility, deletion/lifecycle, and recovery |
| Components -> Dashboards | already public Components v3 contract | unchanged Component v3 contract and Dashboard `3.0.1` process | Dataset outage becomes coherent Component then Dashboard degradation; source-provider outage may serve the last good Dataset snapshot while freshness is degraded |
| Datasets -> Forms reverse usage | Form detail directly queries Dataset source tables | Dataset-owned `source-usage` v1 action keyed by Form/FormVersion | authorized empty, unavailable, and undisclosed are distinct internal states; UI never presents outage as zero links or leaks hidden Datasets |
| Datasets -> Core Operations/app summary | Core joins Dataset/analytics tables and counts Dataset rows | Dataset-owned `operational-status` v1 scoped readiness and summary actions | `operations:view`/admin context is forwarded exactly; other Operations sections remain usable during Dataset outage and show Dataset status unavailable rather than false zero |
| Datasets -> resource consumers | Core transition reference registry reads Dataset rows/lifecycle/revision | Dataset-owned `resolve` action returning exact `ResourceResolutionV1`/`ResourceObservationV1` for Dataset, revision, and major-line types | owner/type/version rejected before lookup; known/random and scoped unauthorized inputs collapse; unavailable and lifecycle states remain contract-accurate |
| Core shell -> Datasets | Core-rendered Dataset routes and browser cross-owner calls | generic manifest-driven GET/HEAD gateway, module-owned documents/assets, signed ShellContext, and Dataset public APIs | same-origin gateway exchanges grants; disabled/unhealthy module is unavailable without Core fallback; browser never calls a provider owner directly |
| Supervisor -> Datasets | no independent service | generic release/instance apply, bootstrap, health, configuration, diagnostics, and rollback | exact owner receipt, bounded failure, teardown, and retry; no definition-specific Supervisor branch |
| Core analytics -> Core Operations | Dataset currently consumes Core analytics projection while Core also reports analytics status | Core may retain `/api/admin/analytics/refresh` and reporting status for Core Operations; Dataset has its own refresh/source projection and zero analytics-table/API dependency | `analytics:refresh` never grants Dataset mutation; `datasets:manage` owns Dataset refresh; either projection can degrade without impersonating the other |

## Target Ownership Table

| Concern | Canonical owner after Sprint 8B |
| --- | --- |
| Dataset tables, materialized schema, migrations, and bootstrap | Dataset module |
| Dataset authoring/revision/materialization policy; directory/tags catalog; provider catalog; bootstrap-validation batch | Dataset module |
| Standalone Dataset templates | no current product surface; Sprint 8B adds none and leaves future ownership with Dataset absent a declared contract |
| Dataset product API, SSR/hydration, routes, and product assets | Dataset module |
| Dataset source import, cursor/attempt staging, dependency DAG, and refresh receipts | Dataset module |
| Dataset v2 contract and references | Dataset contract crate/module release |
| Submitted Response export | current Response owner through an explicit transition contract |
| FormVersion schema export | current Forms owner through an explicit transition contract |
| Actor authorization, scope directory, and principal display catalog | Core policy-neutral control plane through exact contracts |
| Shared operation AST/editor primitives | policy-neutral `tessara-data-ops` / `tessara-web-data-ops`; selectors become `data-ops-*`, while Dataset policy stays owner-local |
| Forms reverse Dataset usage, Operations readiness, and app summary Dataset counts | Dataset provider contracts consumed by Core |
| Dataset typed-reference lifecycle/observation | Dataset `resolve` provider action reached by generic platform dispatch |
| Core analytics refresh/reporting projection | Core; explicitly not a Dataset dependency or refresh path |
| Module inventory, grants, same-origin gateway, desired configuration | Core generic platform |
| Dataset service deployment, health, bootstrap ordering, rollback | Supervisor/deployment adapter plus module-owned providers |
| Component execution over Datasets | Component module consuming Dataset v2 |
| Dashboard presentation | Dashboard module consuming Components v3 |

## Exact Dataset Route And Service-Action Inventory

The Dataset `1.0.0` manifest, router, gateway projection, typed client, and
route/action tests must contain the same set. A route absent from any one view
is an implementation-readiness failure.

### Public product API

| Method and path | Authorization action / capability | Operation and replay rule |
| --- | --- | --- |
| `GET /api/datasets` | `datasets.list` / `datasets:read` | read; none |
| `GET /api/datasets/{dataset_id}` | `datasets.get` / `datasets:read` | read; none |
| `GET /api/datasets/{dataset_id}/revisions` | `datasets.list_revisions` / `datasets:read` | read; none |
| `GET /api/datasets/{dataset_id}/revisions/{revision_id}` | `datasets.get_revision` / `datasets:read` | read; none |
| `GET /api/datasets/{dataset_id}/table` | `datasets.preview_table` / `datasets:read` | last-good materialized read; none |
| `GET /api/datasets/{dataset_id}/distinct-values` | `datasets.distinct_values` / `datasets:read` | last-good materialized read; none |
| `POST /api/admin/datasets` | `datasets.create` / `datasets:manage` | persistent mutation; gateway forwards/generates `x-idempotency-key` and owner stores replay receipt |
| `DELETE /api/admin/datasets/{dataset_id}` | `datasets.delete` / `datasets:manage` | persistent mutation; durable replay receipt |
| `PATCH /api/admin/datasets/{dataset_id}/tags` | `datasets.update_tags` / `datasets:manage` | persistent mutation; durable replay receipt |
| `POST /api/admin/datasets/{dataset_id}/draft-revision` | `datasets.save_draft_revision` / `datasets:manage` | persistent mutation; durable replay receipt |
| `POST /api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish` | `datasets.publish_revision` / `datasets:manage` | persistent mutation plus source sync/promotion; durable replay receipt |
| `PATCH /api/admin/datasets/{dataset_id}/revisions/{revision_id}/label` | `datasets.update_revision_label` / `datasets:manage` | persistent mutation; durable replay receipt |
| `PATCH /api/admin/datasets/{dataset_id}/revisions/{revision_id}/options` | `datasets.update_revision_options` / `datasets:manage` | persistent mutation; durable replay receipt |
| `DELETE /api/admin/datasets/{dataset_id}/revisions/{revision_id}` | `datasets.delete_revision` / `datasets:manage` | persistent mutation; durable replay receipt |
| `POST /api/admin/datasets/sql-preview` | `datasets.preview_sql` / `datasets:manage` | non-persisting validation/read; no replay receipt |
| `POST /api/admin/datasets/{dataset_id}/sql-preview` | `datasets.preview_existing_sql` / `datasets:manage` | non-persisting validation/read; no replay receipt |
| `POST /api/admin/datasets/{dataset_id}/refresh` | `datasets.refresh` / `datasets:manage` | extraction-required synchronous refresh/promotion; durable replay receipt |
| `GET /api/admin/datasets/editor-options/forms` | `datasets.editor_forms` / `datasets:manage` | scoped Dataset-owned broker over Forms catalog; none |
| `GET /api/admin/datasets/editor-options/forms/{form_version_id}` | `datasets.editor_form_schema` / `datasets:manage` | exact rendered/schema option broker with ETag/content digest; none |
| `GET /api/admin/datasets/editor-options/scopes` | `datasets.editor_scopes` / `datasets:manage` | authorized node-tree broker; none |
| `GET /api/admin/datasets/editor-options/principals` | `datasets.editor_principals` / `datasets:manage` | actor-safe display-label broker; none |

Every persistent mutation binds replay to Module Instance, original actor,
authorization JTI/action, method/path, exact raw-body digest (including an
empty-body digest), and idempotency key. Same key and identical input returns
the stored result; any actor/action/body mismatch is rejected. Commit-before-
response loss is retried exactly once logically. JSON mutations require the
declared JSON media type and raw-body size bound; media type and grant/body
digest are checked against the received bytes before typed deserialization or
replay lookup. `x-idempotency-key` is the sole current public mutation header;
the retired bare `idempotency-key` spelling is rejected rather than retained
as an alias. The gateway forwards a valid caller value or generates one under
that same canonical header.

### Dataset-provided private actions

| Path / action | Contract and consumers | Required behavior |
| --- | --- | --- |
| `/api/private/datasets/bootstrap-validation` / `datasets.bootstrap_validate` | Dataset major-line v2; Component/bootstrap | batch validation of exact v2 references before consumer writes |
| `/api/private/datasets/catalog` / `datasets.catalog` | Dataset major-line v2; Components | scoped catalog with nondisclosing pagination |
| `/api/private/datasets/schema` / `datasets.schema` | Dataset major-line v2; Components | exact materialized schema/resource revision |
| `/api/private/datasets/distinct-values` / `datasets.distinct_values` | Dataset major-line v2; Components | bounded scoped distinct values |
| `/api/private/datasets/compatibility` / `datasets.compatibility` | Dataset major-line v2; Components | exact compatibility/lifecycle finding |
| `/api/private/datasets/execute` / `datasets.execute` | Dataset major-line v2; Components | bounded last-good execution with restriction tiers |
| `/api/private/datasets/resolve` / `datasets.resolve` | Dataset resource-observation v1; generic reference consumers | exact resolution/observation for Dataset, revision, and major-line refs |
| `/api/private/datasets/source-usage` / `datasets.source_usage` | Dataset source-usage v1; Forms | authorized Datasets using a Form/FormVersion; empty/unavailable/undisclosed distinct |
| `/api/private/datasets/operations-status` / `datasets.operations_status` | Dataset operational-status v1; Core Operations | scoped readiness/source/field/materialized-response facts and freshness state |
| `/api/private/datasets/summary` / `datasets.summary` | Dataset operational-status v1; app summary | authorized Dataset/revision counts and availability state |

Private calls use exact action/path/method/raw-body/correlation/audience binding,
one-use nonces, caller/Module Instance verification, and stable unavailable,
incompatible, undisclosed, malformed, and timeout results. Generic diagnostics
are never a substitute for product status/actions. Receivers validate media
type and raw bytes before deserialization, and consumers validate the canonical
typed response, echoed request/reference/scope/correlation identity, and
contract version before any product write.

The three reverse-consumer contracts are exact rather than diagnostic aliases:

- `source-usage` requests one canonical Form ID plus optional exact FormVersion
  ID and returns authorized Dataset reference, name, source alias, pinned
  FormVersion, lifecycle, and semantic destination link. Its result state is
  exactly `available`, `empty`, `unavailable`, or `undisclosed`; only
  `available` carries links.
- `operational-status` Operations requests carry the forwarded
  `operations:view` scope and return the current readiness label, Dataset ID/
  name, revision status, source count, field count, ready-response count, and
  freshness/provider state. Summary requests carry the forwarded admin context
  and return Dataset/revision counts plus availability. An unavailable provider
  never returns numeric zero as a substitute.
- `resource-observation` accepts only canonical Dataset, DatasetRevision, or
  DatasetMajorLine v2 references and returns the shared typed resolution and,
  only when authorized/resolved, observation with owner revision, lifecycle,
  availability, and contract identity. Wrong owner/instance/type/version fails
  before lookup; known restricted and random identities remain indistinguishable.

### Dataset-consumed private actions

| Binding | Exact provider actions |
| --- | --- |
| `tessara.datasets.response-export` | Response `checkpoint`, `start` (incremental or full rebase), and `page` under `submitted-response-export` `1.0.0` |
| `tessara.datasets.form-version-schema` | Forms scoped catalog and exact FormVersion schema/render under `form-version-schema` `1.0.0` |
| `tessara.datasets.scope-catalog` | Core requested-node tree/catalog plus revision/content digest under `scope-catalog` `1.0.0` |
| `tessara.datasets.principal-display-catalog` | Core authorized actor display labels plus requested-set digest under `principal-display-catalog` `1.0.0` |

The Dataset browser calls none of these providers. It uses ShellContext and the
four Dataset `editor-options` routes; Dataset's server exchanges downstream
grants, forwards pagination/request digests, emits bounded cache validators,
and returns typed loading/restricted/unavailable states. Browser network tests
fail on direct `/api/me`, `/api/forms`, `/api/nodes`, `/api/admin/users`, or
`/api/form-versions/*` traffic.

## UI And Interaction Baseline

The accepted pre-extraction baseline is the current clean `main` Dataset
experience represented by:

- `/datasets`, `/datasets/new`, `/datasets/{id}`, preview, edit, revision
  history/detail/edit routes in `crates/tessara-web/src/routes/datasets.rs`;
- the current typed views in `crates/tessara-web-datasets`;
- `end2end/tests/datasets.spec.ts` behavior and clean-console expectations.

The older
`docs/audits/module-management-consistency-2026-07-27/05-tessara-datasets-overview.png`
is a historical Module Management transition-descriptor screen, not a Dataset
product baseline. S1 must capture and hash the current-main product matrix under
`docs/audits/sprint-8b-dataset-ui-baseline/` before any UI source moves:

- routes: directory, create, detail/preview, edit, revision list, revision
  detail, and revision edit;
- states/roles: populated, empty, loading, read-only, manager, restricted,
  provider-degraded, validation error, and unsaved/dirty navigation;
- themes: light and dark; viewports: desktop `1440x1000`, tablet `1024x1366`,
  and mobile `390x844`; stored-theme precedence, system-theme fallback, 200%
  zoom/overflow, JavaScript-disabled SSR/direct refresh, hydration, and no
  external font/network asset requests; and
- a separate current Module Management configuration/diagnostics baseline,
  never substituted for product-route evidence, plus affected `/operations`
  Dataset Readiness, Form-detail Dataset Sources, and app-summary consumer
  states at desktop/mobile.

The exact baseline index
`docs/audits/sprint-8b-dataset-ui-baseline/baseline-index.json` records source
commit, route, role, fixture key, theme, viewport, screenshot hash, semantic
assertions, console result, and accessibility result. Implementation adds the same cases to
`end2end/tests/module-ui-visual.spec.ts`. Visual change is allowed only where
the shared SDK corrects ownership or the approved refresh/freshness control is
added; information architecture, existing visible fields, workflow order, and
product semantics otherwise remain unchanged.

The only intentional product addition is a freshness panel on Dataset detail/
status: readers see `Current`, `Stale`, `Degraded`, `Refreshing`, `Failed`, or
`Never Materialized`, last-success/check time, and sanitized failure text;
managers additionally see **Refresh now** and retry. The synchronous action
shows progress, keeps the last-good preview usable, never exposes the opaque
cursor/head, and is not offered inside a dirty editor, so unsaved edits cannot
be overwritten.

Freshness meanings are exact. `Current` means the committed cursor/vector
equaled the last successfully observed authenticated head at `checked_at`; it
is not a timeless claim. `Stale` means a successful head observation is newer
than the committed cursor/vector. `Refreshing` means the active single-writer
attempt owns the partition lock/CAS. `Degraded` means a provider, head check, or
sync failed while a last-good materialization remains readable. `Failed` means
an attempted sync left no usable materialization. `Never Materialized` means
no sync attempt has occurred. A GET may return a fresh ephemeral head
observation but cannot persist it or promote data. `POST .../refresh` never
returns `202` or creates a job: it returns only after atomic promotion, exact
no-op, or a complete module-owned failure, with the resulting freshness state
and replay/materialization receipt identity. Head and cursor values remain
hidden from the UI.

### Acceptance inventory and change discipline

The authoritative change record is
`docs/sprints/sprint-8b-test-change-log.md`; the authoritative browser set is
`end2end/acceptance-manifest.json` schema v2. The manifest's literal file/test
set, not `expected_total` or a copied count, is the acceptance identity. Before
any test rewrite, S1 retains these ten exact `datasets.spec.ts` identities:

- `admin can author, edit, save, and view a Sprint 3A dataset`;
- `admin can UAT Sprint 3B advanced dataset authoring`;
- `dataset SQL preview uses pre-projection join keys and stable field identities`;
- `dataset revision navigation handles repeated detail and error states`;
- `dataset SQL preview renders ordered QuerySpec operations as sequential CTEs`;
- `dataset operations keep operation-local state through reorder, save, and reload`;
- `dataset SQL preview merges unioned source fields under the union step alias`;
- `dataset source picker keeps Version N major-line fields after a newer major exists`;
- `admin can review and publish a dataset draft revision`; and
- `frozen Dataset document routes preserve direct-load and refresh ownership`.

The existing permissions identities for Operations visibility, scoped-reader
draft hiding, and JavaScript-disabled Response/Dataset ownership also remain
literal. Sprint 8B adds, without weakening those tests, these exact identities:

- `Sprint 8B independent Dataset module › editor options use only Dataset-owned browser routes`;
- `Sprint 8B independent Dataset module › synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure`;
- `Sprint 8B independent Dataset module › reverse consumers distinguish authorized empty unavailable and undisclosed states`;
- `Sprint 8B independent Dataset module › mutation replay and static route precedence remain exact`;
- `canonical module UI visual baselines › Datasets directory at 1440 px (light)`;
- `canonical module UI visual baselines › Datasets editor at 390 px (light)`;
- `canonical module UI visual baselines › Datasets directory at 1440 px (dark)`;
- `canonical module UI visual baselines › Datasets editor at 390 px (dark)`;
- `canonical module UI visual baselines › Datasets revisions at 1024 px`;
- `canonical module UI visual baselines › Datasets preview at 1440 px`; and
- `canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas`.

Every deletion, rename, expectation, fixture, selector, or snapshot change gets
one test-change-log entry with path, old/new exact identity, governing `ac-xx`,
behavioral reason, equal-or-stronger replacement proof, and invalidated target/
lane. Formal discovery must equal the manifest set exactly and execute it with
one worker, zero retries, no `only`/skip/fixme/filter/max-failure truncation, and
no snapshot-update mode. Each touched scenario owns setup/cleanup and passes in
isolation as well as in the complete suite; serial order is not a fixture.

### UI ownership inventory

| UI surface | Required owner |
| --- | --- |
| Complete HTML document, reset, tokens, themes, canvas, shell, navigation, generic controls | `tessara-module-ui` |
| Dataset typed Leptos markup and product interaction state | `tessara-dataset-ui` |
| Dataset-only layout/selectors | namespace-rooted Dataset product CSS |
| Shared operation controls/selectors | `tessara-module-ui` plus policy-neutral `data-ops-*` classes; no `dataset-*` class emitted by the shared primitive |
| Browser entry/lifecycle assets | Dataset module, generated by the shared asset builder |
| SSR/hydration and direct document | Dataset module using SDK document composition |
| Mount/navigate/suspend/resume/dirty-state/unmount | shared lifecycle adapter |
| Active navigation and title | authenticated manifest route through generic shell host |
| Responsive/accessibility behavior | SDK primitives plus Dataset-owned semantic markup |

No raw module HTML/DOM construction, copied SDK CSS, product token definition,
unnamespaced selector, browser-to-Core product lookup, or Core Dataset markup
is permitted. All Dataset-only rules currently in `style/core.css` move to the
module asset. The focused visual reproducer and mandatory
`ui-sdk-conformance`/`ui-provider-boundaries` targets must pass before consumer
cutover.

## Fresh Materialization And Canonical Fixture Graph

### Disposable topology

- Core database: identity, authorization, installation, module inventory, and
  current transition providers only.
- Dataset database: Dataset schema, revisions, imported Response aggregate/value
  projection, source partitions/cursors/attempt staging, materialized tables,
  mutation/bootstrap/materialization receipts, and module operational state.
- Component and Dashboard databases: rebuilt from their owner baselines.
- Independent Core, Supervisor, gateway, Dataset, Component, and Dashboard
  processes with distinct database credentials and service identities.
- Four separately addressable profile-local provider proxies bind Response,
  Forms, scope, and principal actions to the same current Core process. The
  proxies/test double can independently timeout, reject, or corrupt one action
  without stopping Core auth/control-plane and cannot access product databases.

### Owner-controlled bootstrap order

1. Reset every named disposable database and verify pairwise-distinct bindings.
2. Migrate Core, Dataset, Component, and Dashboard databases through their own
   migration identities.
3. Start private control paths while the public gateway remains stopped.
4. Apply the reference Blueprint and enroll exact releases/instances.
5. Core bootstraps identity/RBAC and transition-provider source fixtures.
6. Dataset bootstrap calls Form/Response/scope/principal providers. For
   Response it first obtains an authenticated head cursor, fixes that value as
   the snapshot upper bound, pages changes from the Dataset's last committed
   cursor through that bound, stages immutable upserts/tombstones idempotently,
   rebuilds the affected Dataset dependency closure locally, and advances the
   published cursor only in the successful promotion transaction. It then
   creates Dataset rows/materializations and returns signed typed Dataset v2
   read-back.
7. Component `1.1.0` bootstrap consumes Dataset v2 read-back and creates current
   Component resources; Dashboard bootstrap consumes Component read-back.
8. Verify all owner receipts, exact health contracts, inventory/navigation,
   source provenance, and cross-owner database isolation; then start gateway.
9. Reapply unchanged input and prove a semantic no-op with stable topology,
   configuration, owner receipts, resource identities, and product state.

### Canonical fixture inventory

Implementation creates tracked
`deploy/sprint-8b/fixtures/reference-fixture-contract.json`. Stable logical keys,
not UUIDs or mutable counts, are the test vocabulary:

- actors `actor.admin`, `actor.dataset-manager`, `actor.operations`,
  `actor.full`, `actor.restricted`, `actor.confidential`, and `actor.disjoint`;
- Forms/FormVersions `form.primary/v1`, `form.secondary/v1`, and
  `form.disjoint/v1`, including exact field IDs/types/options/layout digests;
- Responses `response.initial`, `.same-time-a`, `.same-time-b`, `.new`,
  `.corrected`, `.status-out`, `.status-in`, `.redacted`, `.deleted`, and
  `.outside-scope`, all created/mutated through Response owner APIs/bootstrap;
- two independent Response cursor partitions over related source data, proving
  binding/scope isolation and authorized empty-page advancement;
- Datasets `dataset.base`, `.derived`, `.derived-second-hop`,
  `.independent-binding`, `.disjoint-binding`, `.incompatible`, and
  `.cycle-candidate`, with source, revision, major-line, catalog/tag,
  bootstrap-validation-batch, retry, and outage expectations;
- Component Table/Chart/Stat resources in `tessara.components` `1.1.0` over
  Dataset v2, with compatible, disjoint, lifecycle, and incompatible cases;
- Dashboard `3.0.1` has four receipt-bound non-overlapping placements over
  compatible Components, including `component.dataset-disjoint`; `actor.full`
  has the minimum `components:read`/`dashboards:read` capabilities and observes
  that disjoint placement only as a redacted footprint, plus outage/recovery;
  and
- Forms reverse usage, Operations readiness, app-summary counters, and all
  three Dataset resource-observation types.

Every physical identity is read back from its owner receipt and injected by
logical key. No fixture writes another owner's database, predicts a UUID, uses
copied inventory counts as identity, or mutates Responses with SQL.

### Failure and recovery

Inject one deterministic pre-write provider incompatibility and one bounded
mid-apply Dataset bootstrap failure. Retain diagnostics, prove no unauthorized
or cross-owner state, tear down the exact partial Dataset/downstream topology,
start a successor from empty, and prove the canonical first apply and no-op.

## Response Export And Dataset Refresh State Machine

### Trigger and read semantics

- Dataset checks source dependency heads synchronously during create, publish,
  module bootstrap, and `POST /api/admin/datasets/{dataset_id}/refresh`.
  `datasets:manage` authorizes refresh; no `analytics:refresh` or new capability
  is introduced. The refresh POST is a synchronous terminal operation, never a
  `202`/job queue: it returns after atomic promotion, exact no-op, or a complete
  module-owned failure. One partition writer owns the lock/CAS; a concurrent
  compatible retry receives the winning receipt and stale promotion is rejected.
- Directory/detail/table/distinct/provider execute and Component/Dashboard
  reads are side-effect free. A GET may obtain or consume an authenticated head
  observation solely to report freshness, but it never pulls pages, promotes,
  commits the observation, advances a cursor, or otherwise mutates Dataset
  state. Reads consume only the last successful materialization.
- A transient Response/Form/scope/principal outage fails the current write or
  refresh but leaves the last successful materialization readable and marks
  source freshness degraded. A Dataset with no successful materialization is
  unavailable. Dataset process/database outage remains a hard downstream
  provider outage.

### Response owner sequence and envelope

- One Response-owner mutation primitive covers submission service writes,
  workflow-created/state-mutated Responses, corrections, resubmission,
  redaction/deletion, and bootstrap/demo producers. It updates the Response
  aggregate and appends its export envelope in one database transaction.
- A transactionally serialized counter row, locked before allocation, ensures
  no lower committed position can become visible after a higher position. A
  PostgreSQL sequence/identity alone is insufficient because allocation order
  is not commit order. Rollback persists neither mutation nor envelope.
- Each immutable change envelope is one complete submitted-Response aggregate
  upsert or an explicit tombstone. It contains stable Response/FormVersion/node
  and field IDs, status transition, timestamps, actor-safe last-modified label,
  scalar/array/null values, restriction facts, and the canonical value-text
  conversions required to preserve current analytics behavior. It never
  requires Dataset to reread mutable Response rows while paging.
- The exported head/cursor token is opaque and bound to provider instance/epoch,
  contract version, selected FormVersion/source set, authorization audience,
  scope digest, and export policy. The provider computes the authorized
  partition head, so disjoint-scope activity neither changes the visible head
  nor leaks global activity volume. Timestamps and `MAX(created_at)` are
  diagnostic payload only.

### Published state, staging, and promotion

Dataset maintains two deliberately separate state classes:

1. **Published state:** source-partition key, provider epoch, last committed
   cursor, imported source projection, Dataset/upstream dependency vector,
   materialized revisions/major lines, resource revisions, and signed receipt.
2. **Attempt-private state:** attempt/idempotency key, original actor, input and
   scope/audience digests, starting published cursor, fixed upper cursor,
   immutable snapshot ID/expiry, staged page/change digests, next-page cursor,
   and staged upserts/tombstones. It is never served to product consumers.

The partition key includes provider Module Instance/Core release identity,
contract version, Dataset source binding and revision, FormVersion/source
selection, authorized Dataset scope, audience, and export policy. A module-wide
cursor is forbidden. A scope/provider/source change creates a new partition and
requires a fresh authenticated snapshot.

Start fixes one authenticated upper bound. Pages scan strictly after the
attempt cursor through that bound and return an opaque next cursor even when
all scanned changes are filtered, allowing safe empty-page advancement without
disclosure. Hidden page state may checkpoint to avoid re-fetch inside the same
attempt; correctness remains rooted at the last published cursor, so an
abandoned attempt can be replayed in full without changing served state.

Dataset stages changes into owner-local imported Response aggregate/value
tables. QuerySpec then reads only Dataset-owned imported or Dataset-owned
materialized tables. The source pull is incremental; arbitrary joins,
aggregations, calculations, filters, and unions may be recomputed locally in
full. Sprint 8B does not claim a general incremental query engine.

One single-writer lock/CAS per partition prevents concurrent promotion. The
promotion transaction verifies the starting cursor/generation, applies staged
source changes, rebuilds impacted Datasets in topological order, refreshes
published major lines and downstream Dataset sources, stores the receipt, and
advances the published cursor. If any source, downstream Dataset, or receipt
step fails, none is promoted. A competing stale attempt is rejected or returns
the winning compatible receipt; it never overwrites newer state.

The canonical atomicity case is explicit: a Response change for `dataset.base`
rebuilds `dataset.base` -> `dataset.derived` ->
`dataset.derived-second-hop` as one affected closure, while
`dataset.independent-binding` retains its exact cursor, generation, rows, and
receipt. Components and Dashboards observe either the complete old closure or
the complete new closure, never mixed generations. A proposed cycle is rejected
before staging. An injected failure while rebuilding `dataset.derived` rolls
back the base import, all three Dataset generations/materializations, cursor,
resource revisions, and receipt, so downstream Component/Dashboard reads remain
on the prior last-good closure.

### Initial and expired-cursor rebase

No cursor or an authenticated `cursor_expired` result starts a Response-owned
immutable full snapshot at a fixed upper cursor. The old Dataset projection
continues to serve while Dataset pages and stages the full authorized state.
Successful promotion atomically replaces that partition and its affected
Dataset closure; failure or snapshot expiry discards/restarts private staging
and preserves the old published state. The provider reports no retention floor,
row count, or global position that could disclose other scopes.

### Mandatory crash/concurrency proof points

Tests interrupt before page persistence, after page persistence, after hidden
checkpoint, before promotion, and after promotion commit but before the HTTP
response. They also race Response mutations/sequence allocation, two refreshes
for one partition, two independent partitions, a scope change, and a provider
epoch change. Retry must produce the same rows, cursor, dependency closure, and
single durable receipt as an uninterrupted run.

## Specifications

### Functional and lifecycle

- Dataset directory, create/edit/detail/preview, revisions, publish/delete,
  QuerySpec operation ordering, SQL preview, directory/tags catalog,
  Component-facing provider catalog, bootstrap-validation batch,
  materialization, compatibility, tags, and scope behavior retain current
  observable semantics. There is no current standalone template API/UI to
  preserve or invent.
- Provider snapshots are explicit and versioned. A materialization receipt
  binds provider snapshot identity, Dataset revision, source contract versions,
  starting cursor, inclusive snapshot upper bound, terminal cursor, scope,
  input digest, page/change counts, and resulting materialized major line.
- Response owns the durable monotonic sequence and change log. The sequence is
  allocated transactionally with the Response mutation that it represents;
  export pages are ordered by sequence, bounded by one authenticated snapshot
  upper cursor, and contain idempotent upserts or explicit tombstones. Dataset
  never reads the sequence table or any other Response table directly.
- Dataset performs a cheap authorized Response head/checkpoint comparison
  before pulling pages. Equality with the last committed cursor is an exact
  no-change result. A higher head triggers incremental paging; unavailable,
  malformed, foreign-epoch, or scope-divergent cursors fail closed. An expired
  valid cursor starts only the authenticated full-snapshot rebase defined
  above and cannot advance published state until promotion succeeds.
- Retriable dependency failure cannot publish a partial revision or mark an
  incomplete materialization ready. Retry is idempotent for the same owner
  input and resumes after the last committed cursor; changed snapshot/scope
  identity creates a new authenticated attempt.
- Dataset resources use monotonically advancing owner revisions and Module
  Instance-owned typed references. Old owner/type/version combinations fail
  before lookup.
- Form detail reverse usage, Operations Dataset readiness, application-summary
  counts, and resource observations are Dataset-owned projections. Their Core
  consumers distinguish available-empty from provider-unavailable and never
  derive product data from generic module diagnostics.
- The Operations projection preserves current readiness labels exactly:
  `Ready`, `No Ready Responses`, `Draft`, `Superseded`, `Unavailable`, and
  `No Published Revision`, with Dataset ID/name, revision status, source count,
  field count, and ready-response count scoped as today. A separate freshness/
  provider state prevents `Unavailable` or an outage from being inferred as an
  empty result.

### Authorization and nondisclosure

- Core exchanges short-lived audience/action/method/path/body/correlation-bound
  grants; Dataset and each source provider validate them independently.
- Dataset management requires full authorized scope over every selected node
  and source. Read/preview/execution intersects Dataset scope, source scope,
  field restriction tier, and actor grants.
- Random, known-but-unauthorized, incompatible, and unavailable source/resource
  identities expose no distinguishable product details.
- Components receive only Dataset fields/rows permitted by the exact forwarded
  scope. Dashboard receives no new authority through Component outage state.

### Data and compatibility

- The Dataset database owns all Dataset foreign keys; references to Forms,
  Responses, Core nodes, and Module Instances are opaque typed IDs plus
  authenticated provider metadata, never cross-database foreign keys.
- Dataset contract v2 is the sole active line. Consumers advance atomically;
  v1 and transition resource types are negative fixtures only.
- `created_at`, `submitted_at`, `last_modified_at`, or any `MAX(timestamp)` may
  be exposed as Response facts or diagnostic hints but cannot substitute for
  the provider-owned sequence. Equal timestamps, late imports, updates,
  resubmission, redaction, and deletion must all produce ordered changes.
- The current bootstrap-validation batch, directory/tags catalog, and provider
  catalog remain in Dataset. A future standalone template/batch provider must
  declare an operation contract before ownership moves.
- Core's analytics projection/status and `analytics:refresh` remain Core
  reporting behavior. Dataset compiles against only its imported source tables,
  exposes its own refresh, and contains no Core analytics API/table dependency.
- No Core migration copies Dataset data. Clean rebuild is the only Sprint 8B
  transition path.

### Deployment, configuration, diagnostics, and observability

- Manifest `tessara.datasets` `1.0.0` declares exact runtime/migration images,
  port, probes, routes, assets, capabilities, contracts, dependencies, resource
  types, service actions, platform tuple, and configuration schema.
- Runtime command is `/usr/local/bin/dataset-module serve`; migration command
  is `/usr/local/bin/dataset-module migrate`; internal registration is
  `datasets` on HTTP port `8093`; liveness/readiness are exact GET
  `/health/live` and `/health/ready` JSON routes with no redirect. Live returns
  200/`application/json` and `status=live`; ready returns 200 with
  `status=ready|degraded` or 503 with `status=not_ready`, plus schema,
  definition, release, and sanitized dependency-state keys.
- Configuration is exact: `display_label` string 1–80 default `Datasets`,
  `provider_request_timeout_seconds` integer 1–30 default `5`,
  `provider_retry_limit` integer 0–3 default `1`, and
  `response_export_page_size` integer 1–1000 default `250`; unknown fields,
  coercion, clamping, unsupported schema, and partial save are rejected.
- Platform versions are exact: Core Release `0.1.0`, Shell Context `1.0.0`,
  module control `1.1.0`, module contract/runtime/UI `0.3.0`, design-system
  asset ABI `2.0.0`, and conformance suite `1.2.0`.
- Readiness requires the Dataset database, projected security/configuration,
  structurally compatible required bindings, and either no product state yet
  during controlled bootstrap or a valid last-good materialization. Transient
  upstream availability is degraded freshness, not a reason to hide a valid
  last-good snapshot. Liveness proves the process only. Diagnostics are
  sanitized and report selected bindings, contract/version compatibility,
  last observation, materialization/freshness state, and stable failure codes
  without credentials, raw references, actor grants, row values, or restricted
  counts.
- Logs/traces include correlation, module instance, operation, dependency,
  attempt, duration, result code, and retry classification without secrets or
  product row disclosure.
- Public/private failures use one module-owned JSON envelope only:
  `schema_version`, nested `error.code`/`error.message`, and `correlation_id`.
  Stable code families distinguish `dataset.validation_failed`,
  `dataset.dependency_unavailable`, `dataset.dependency_incompatible`,
  `dataset.conflict`, `dataset.idempotency_mismatch`, and
  `dataset.malformed_request`; known/random unauthorized resources share
  `dataset.not_found_or_forbidden`. Core generic codes and duplicate top-level
  `error`/message aliases are forbidden after gateway forwarding.

### Rollout, recovery, and rollback

- Source-exact first materialization is offline against disposable databases.
  Gateway remains stopped until every owner completes and health is exact.
- A separately built prior-compatible Dataset `0.9.0` release is upgraded to
  `1.0.0`, rolled back to `0.9.0`, and restored to intended `1.0.0` through a
  resolved one-owner Blueprint delta.
- Dataset `0.9.0` is frozen in S2 as a real immutable image/manifest with the
  same Module Instance-owned Dataset v2 provider contract and a database shape
  that can read state after `1.0.0`'s additive sync migration. Component
  `1.1.0` requires Dataset contract `=2.0.0` and remains byte-for-byte selected
  across the Dataset-only transition; Dashboard remains `3.0.1`.
- Core, Component, Dashboard, and unrelated module images, containers, restart
  counts, data, navigation, and availability must remain unchanged during the
  Dataset-only release transition.
- Failed deployment/apply restores the last healthy selected release and exact
  topology. Product rollback never reactivates the removed Core adapter or v1
  contract.

## Assumptions, Decisions, Dependencies, And Blockers

### Decisions

- Tessara remains pre-production; fresh-only forward migration is governing.
- Dataset major-line contract `2.0.0` is required because reference ownership
  changes from Core Installation to Dataset Module Instance.
- Current Response and Forms owners publish narrow transition contracts now;
  their future extraction may advance those contracts but cannot expose their
  storage to Dataset.
- Sprint 8B adds the monotonic Response change sequence/change log because no
  safe cursor exists today. Polling the versioned head and incremental export
  is the current delivery mechanism; future event streaming must reuse the same
  ordered change envelope and cursor semantics rather than introduce a second
  synchronization model.
- Synchronous refresh is explicit and owner-controlled: create, publish,
  bootstrap, and a manager refresh mutation poll dependencies; reads never
  write. This avoids a hidden write-on-read contract while making post-publish
  Response changes user-testable this sprint.
- Response expiry recovery is an authenticated full-snapshot rebase that keeps
  the last-good Dataset visible until atomic replacement.
- Core scope-catalog access is policy-neutral platform behavior, not retained
  Dataset product ownership.
- The current accepted UI is the baseline; Sprint 8B is not a redesign.
- Exact identities and receipt read-back replace copied seed counts wherever
  count is not itself a product contract.

### Dependencies

- Canonical module contract/runtime/UI/testkit, generic enrollment/gateway,
  Supervisor apply/bootstrap/health, Blueprint/lockfile, and policy-v2 tooling
  completed before Sprint 8B.
- Component public contract v3 and Dashboard public contract v3 remain the
  downstream path. An immutable Component `1.1.0` release changes only its
  consumed Dataset contract/binding to v2; Dashboard `3.0.1` remains selected.
- Current Forms, Responses, Core auth/scope/principal, Operations, app summary,
  and Core analytics behavior must be split at the exact boundaries above
  without extracting those owners.

### Open questions and blockers

No product decision remains open: refresh triggers, read behavior, cursor
expiry recovery, Component/Dashboard release identities, reverse-consumer
outage semantics, and idempotency are fixed above. Implementation must treat any newly found
source-table dependency or UI ownership ambiguity as a plan-impacting finding:
add it to the inventories and validation contract before cutting over rather
than silently retaining access. An inability to express a required provider
contract without moving provider product policy is a product/architecture
blocker for user direction.

## End-To-End Process And Proof Closure

This is the implementation/validation handshake. Each row must land as one
coherent change: product behavior, boundary/harness, implementation target,
formal candidate proof, and UAT predicate. A target cannot pass from source
scans alone when the row calls for a live outcome.

| Process | Required modular behavior/change | Implementation targets | Formal lanes and UAT |
| --- | --- | --- | --- |
| Enroll/start/control | Dataset manifest/release/instance, DB/runtime/migration identities, security/config projection, exact probes/routes/assets/navigation order `6`; no definition-specific Core/Supervisor branch | `owner-product`, `inventory-navigation`, `migration-seed`, `ui-sdk-conformance` | `readiness-contract`, `rehearsal-materialization`, `sit-static`, `sit-smoke`; UAT-8B-02/03/06 |
| Direct/lifecycle document | Dataset owns SSR/hydration and browser assets; ShellContext supplies actor; route/theme/viewport/dirty-state baseline remains exact | `ui-sdk-conformance`, `ui-provider-boundaries`, `fixture-acceptance` | `rehearsal-browser`, `rehearsal-conformance`, `sit-browser`; UAT-8B-01/09 |
| Editor lookup | Browser calls Dataset only; server calls Forms catalog/schema, Core scope, and principal display contracts; loading/restricted/outage states preserve unsaved input | `ui-provider-boundaries`, `contract-boundary`, `owner-product` | `rehearsal-conformance`, `sit-rust`, `sit-browser`, `uat-product`; UAT-8B-01/09 |
| Create/save/delete/tag/revision | Exact current route/DTO/QuerySpec semantics run in Dataset DB; every persistent mutation has durable replay semantics | `owner-product`, `api-idempotency`, `fixture-acceptance` | `rehearsal-rust`, `sit-rust`, `sit-browser`, `uat-replay-refresh`; UAT-8B-01/11 |
| Publish/initial source import | Exact provider versions/schema/scope validate before writes; immutable source snapshot stages and promotes with revision/major line/receipt | `response-export-contract`, `response-incremental-sync`, `dataset-refresh-dag`, `clean-materialization` | `rehearsal-source-sync`, `rehearsal-materialization`, `sit-rust`, `sit-smoke`, `uat-providers`; UAT-8B-04 |
| Post-publish refresh | Manager refresh checks dependency vector; unchanged head makes no page call/mutation; changes/rebase promote atomically; GETs remain side-effect free and last-good | `response-incremental-sync`, `dataset-refresh-dag`, `semantic-noop`, `failure-recovery`, `api-idempotency` | `rehearsal-source-sync`, `rehearsal-recovery`, `sit-rust`, `sit-smoke`, `uat-replay-refresh`; UAT-8B-04/11 |
| Query/materialize/preview | Compiler reads only Dataset imported/materialized tables; complete operation pipeline and restriction tiers match current behavior | `owner-product`, `core-subtraction`, `response-incremental-sync` | `rehearsal-rust`, `sit-static`, `sit-rust`, `sit-browser`; UAT-8B-01/04 |
| Dataset dependency DAG | Dataset sources pin v2 major line/revision/generation; base -> derived -> second-hop promotes as one closure, independent binding is stable, cycles reject before staging, rebuild failure rolls back the whole closure, and Component/Dashboard observe no mixed generation | `dataset-refresh-dag`, `owner-product`, `failure-recovery`, `deployed-smoke` | `rehearsal-source-sync`, `rehearsal-recovery`, `sit-rust`, `sit-smoke`, `uat-providers`; UAT-8B-04/11 |
| Component/Dashboard | Component `1.1.0` consumes only Dataset v2 six-action provider; Dashboard `3.0.1` still consumes Component v3; scope/lifecycle/outage remain exact | `consumer-cutover`, `contract-boundary`, `deployed-smoke` | `rehearsal-reverse-consumers`, `rehearsal-smoke`, `sit-browser`, `sit-smoke`, `uat-crossmodule`; UAT-8B-05 |
| Forms reverse usage | Form detail gets authorized Dataset-source links through `source-usage`; semantic destination links and empty/unavailable/undisclosed states are correct | `reverse-consumers`, `core-subtraction`, `deployed-smoke` | `rehearsal-reverse-consumers`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-reverse-consumers`; UAT-8B-10 |
| Operations/app summary | Core delegates Dataset readiness/counts to operational-status; other sections survive outage; no false zero or metadata leak | `reverse-consumers`, `core-subtraction`, `deployed-smoke` | `rehearsal-reverse-consumers`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-reverse-consumers`; UAT-8B-10 |
| Typed-resource lifecycle | Dataset resolves/observes Dataset, revision, and major-line references with owner revision/lifecycle; Core transition SQL/readers disappear | `resource-resolution`, `contract-boundary`, `core-subtraction` | `rehearsal-conformance`, `sit-static`, `sit-rust`, `uat-resource-resolution`; UAT-8B-11 |
| Source/owner outage | Per-binding proxies fault Response/Form/scope/principal independently; last-good read, blocked mutation, degraded diagnostic, nondisclosure, and recovery are exact | `response-incremental-sync`, `ui-provider-boundaries`, `reverse-consumers`, `failure-recovery`, `deployed-smoke` | `rehearsal-source-sync`, `rehearsal-smoke`, `rehearsal-recovery`, `sit-smoke`, provider/reverse UAT lanes; UAT-8B-04/05/09/10 |
| Fresh apply/no-op/failure | Owner-only bootstrap/read-back order, exact isolation, failed-attempt teardown, successor empty apply, and semantic no-op | `migration-seed`, `clean-materialization`, `semantic-noop`, `failure-recovery`, `fixture-acceptance` | readiness/rehearsal materialization and recovery, `sit-smoke`, materialization/recovery UAT; UAT-8B-02/07 |
| Upgrade/rollback | Real Dataset `0.9.0`/`1.0.0`; fixed Component `1.1.0` and Dashboard `3.0.1`; state/provider route preserved, no Core fallback | `independent-upgrade-rollback`, `inventory-navigation`, `deployed-smoke` | `rehearsal-upgrade`, `sit-smoke`, `uat-upgrade`; UAT-8B-08 |
| Plan/proof integrity | Acceptance-criterion, target, lane, fixture, command, and evidence identities are set-equal across plan, record, contract, and UAT contract; representative path-impact tests prevent stale inheritance | `planning-contract-alignment`, `runner-selftest`, `uat-readiness` | `readiness-contract`, `rehearsal-static`, `preflight-freeze`, `uat-scripted`; all UAT preconditions |

## Acceptance Criteria

1. **ac-01 — Independent owner:** `tessara.datasets` is enrolled exactly once
   as a real Release/Instance and independently serves API, UI, persistence,
   migration, configuration, diagnostics, routes, assets, and health.
2. **ac-02 — Product parity:** current Dataset directory, authoring, revision,
   preview, operation pipeline, materialization, directory/tags/provider
   catalogs, bootstrap-validation batch, and lifecycle behaviors pass typed
   API, browser, accessibility, responsive, and clean-console checks; no
   nonexistent standalone template surface is invented.
3. **ac-03 — UI SDK ownership:** direct documents and lifecycle navigation
   match the accepted baseline and pass SDK/CSS/source conformance with no raw
   module HTML/DOM or copied generic styles.
4. **ac-04 — Response contract:** materialization obtains submitted Response
   rows only through the exact source contract and rejects unavailable,
   unauthorized, malformed, stale, and incompatible exports without partial
   Dataset state.
5. **ac-05 — Form/scope/principal contracts:** authoring and materialization
   obtain FormVersion catalog/schema, scope facts, and actor-safe display labels
   only through exact contracts with nondisclosing authorization and bound
   content revisions/digests.
6. **ac-06 — Dataset v2:** Components consumes Module Instance-owned Dataset v2
   references across the real process boundary; v1/Core transition references
   and payloads are rejected before lookup.
7. **ac-07 — Scoped execution:** shared authorized scope renders the expected
   Component/Dashboard result; disjoint or restricted scope reveals neither
   Dataset metadata nor row values.
8. **ac-08 — Outage and recovery:** independently faulted Response/Form/scope/
   principal/Dataset actions produce stable sanitized findings, bounded
   failures, no partial writes, correct last-good availability, coherent
   downstream degradation, and healthy convergence after recovery.
9. **ac-09 — Materialization retry/concurrency:** authenticated retry and
   concurrent attempts are idempotent for identical input, stale promotion is
   rejected, and no revision, row, cursor, receipt, resource, inventory, or
   navigation is duplicated.
10. **ac-10 — Fresh owner seed:** empty databases rebuild in owner order using
    typed read-back only; no script uses another owner's credentials or writes
    another owner's product tables.
11. **ac-11 — Semantic no-op:** unchanged second apply produces no semantic
    change to product state, topology, configuration, receipts, or identities.
12. **ac-12 — Failure containment:** deterministic failed apply is retained,
    exact partial topology is removed, and a successor from-empty apply restores
    the canonical healthy topology without manual repair.
13. **ac-13 — Core subtraction:** Core contains no active Dataset schema,
    routes, DTOs, product handlers, provider adapter, web composition, seed,
    static capabilities/grants, or transition descriptor.
14. **ac-14 — Isolation:** Dataset has no Response, Form, Component, Dashboard,
    or Core database credential/access; providers and consumers have no Dataset
    database credential/access.
15. **ac-15 — Inventory/navigation:** Core transitions are exactly Forms,
    Workflows, Responses, and Migration; Datasets, Components, and Dashboard
    appear exactly once via enrolled manifests; Dataset navigation is exactly
    ID `tessara.datasets.navigation`, key/label `datasets`/`Datasets`, route
    `/datasets`, destination `datasets.directory`, Main order `6`, guarded by
    `datasets:read` or `datasets:manage`.
16. **ac-16 — Operations:** module-owned configuration validates/applies through
    the generic template; readiness/liveness and sanitized diagnostics use exact
    status and content contracts.
17. **ac-17 — Upgrade/rollback:** real Dataset `0.9.0` -> `1.0.0` -> `0.9.0` ->
    `1.0.0` transitions preserve Dataset state and leave unrelated owners
    unchanged.
18. **ac-18 — End-to-end exit:** a tester authors/materializes/previews a
    Dataset from provider contracts, executes a Component over it, and views
    the result on a Dashboard across independently deployed processes/databases.
19. **ac-19 — Incremental Response synchronization:** an unchanged Response
    head performs no pull and no Dataset mutation; ordered new, corrected,
    status-changed, redacted, and deleted Responses are applied exactly once
    through bounded pages; interruption leaves the prior cursor committed; and
    retry reaches the same result as one uninterrupted pull without a Response
    table scan or timestamp-watermark dependency; empty scoped pages,
    concurrent mutations, provider epoch change, and expired-cursor full rebase
    preserve the same nondisclosing atomicity.
20. **ac-20 — UI provider isolation:** Dataset browser traffic targets only
    signed ShellContext and Dataset routes; its server-side provider clients
    preserve current Form, schema, node-tree, and updater-label editor options
    through exact typed contracts and intentional loading/restricted/outage
    states.
21. **ac-21 — Resource observation:** Dataset, DatasetRevision, and
    DatasetMajorLine references resolve/observe through Dataset with exact
    owner revision/lifecycle/availability semantics; v1, wrong owner, wrong
    instance, known restricted, and random inputs are rejected/nondisclosed
    without Core product SQL.
22. **ac-22 — Reverse consumers:** Forms related Dataset sources, Core
    Operations Dataset readiness, and application-summary Dataset counts use
    exact Dataset provider actions. Authorized empty, unavailable, and
    undisclosed render differently internally and outage is never reported as
    a false zero.
23. **ac-23 — API/action and replay completeness:** the manifest, router,
    gateway, client, tests, and acceptance inventory have the exact public and
    private route/action sets above; every persistent mutation has durable
    raw-body-bound replay behavior and every private grant/nonce is one-use.
24. **ac-24 — Refresh and dependency DAG:** create, publish, bootstrap, and
    manager refresh check the complete dependency vector, topologically rebuild
    base -> derived -> second-hop, and atomically promote the whole affected
    Dataset closure. An independent binding remains byte/identity stable; cycle
    or downstream rebuild failure promotes no imported row, Dataset generation,
    cursor, revision, resource revision, or receipt. Component/Dashboard reads
    observe only the complete old or complete new closure. GETs never promote;
    last-good data remains available with exact degraded freshness.
25. **ac-25 — Owner-local source projection:** Dataset stores imported Response
    aggregates/values, compiles only against its own source/materialized tables,
    and locally recomputes arbitrary QuerySpec results; no Dataset runtime SQL,
    credential, or compiled dependency reaches Core product/analytics tables.
26. **ac-26 — Core analytics separation:** Core reporting analytics refresh and
    status may remain Core-owned, but Dataset never invokes or depends on them;
    `datasets:manage` refresh and `analytics:refresh` are separate authority and
    failure domains.

## Canonical Requirement-To-Proof Crosswalk

This table is literal and set-equal to the validation contract. Compact fixture/
environment/evidence identities below expand as follows:

- `reference` = `deploy/sprint-8b/fixtures/reference-fixture-contract.json`;
  `faults` = `deploy/sprint-8b/fixtures/provider-fault-contract.json`;
  `upgrade` = `deploy/sprint-8b/fixtures/upgrade-fixture-contract.json`;
  `ui` = `docs/audits/sprint-8b-dataset-ui-baseline/baseline-index.json`;
  `acceptance` = `end2end/acceptance-manifest.json` plus
  `docs/sprints/sprint-8b-test-change-log.md`; and `uat-contract` =
  `docs/sprints/sprint-8b-uat/scenario-contract.json`.
- `isolated-live` = the lane-owned `tessara-s8b-<lane>` Compose project and
  pairwise-distinct lane databases; `frozen-sit` = `tessara-s8b-sit`; `offline`
  launches no stack. Every live identity includes its fixture-prepare and
  canonical-restoration receipt.
- `target:<id>` =
  `artifacts/sprint-8b-closeout/implementation/targets/<id>/result.json`;
  `lane:<id>` =
  `artifacts/sprint-8b-closeout/<phase>/lanes/<id>/result.json`; and `uat:<id>` =
  `artifacts/sprint-8b-closeout/uat/scenarios/<id>/result.json`. Each row requires
  the target/lane evidence for every ID it lists and, when Manual UAT is named,
  the corresponding `uat:<id>` evidence. A requirement without manual proof
  would say `automated only` explicitly.

| Requirement | Slice(s) | Exact implementation target IDs | Exact formal lane IDs | Fixture / environment / evidence identity | Manual UAT |
| --- | --- | --- | --- | --- | --- |
| `ac-01` | S2, S4–S5 | `owner-product`, `ui-sdk-conformance`, `inventory-navigation`, `migration-seed`, `deployed-smoke` | `readiness-contract`, `readiness-materialization`, `rehearsal-materialization`, `rehearsal-browser`, `rehearsal-conformance`, `preflight-freeze`, `sit-static`, `sit-browser`, `sit-smoke`, `uat-product`, `uat-operations`, `uat-subtraction` | `reference`, `ui`; `offline` + `isolated-live` + `frozen-sit`; listed `target:*`/`lane:*` | UAT-8B-02/03/06 |
| `ac-02` | S2–S3, S5 | `owner-product`, `ui-sdk-conformance`, `fixture-acceptance`, `api-idempotency`, `dataset-refresh-dag` | `readiness-acceptance`, `rehearsal-rust`, `rehearsal-browser`, `rehearsal-source-sync`, `preflight-freeze`, `sit-rust`, `sit-browser`, `uat-product`, `uat-replay-refresh` | `reference`, `ui`, `acceptance`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-01/04/11 |
| `ac-03` | S1–S2 | `ui-sdk-conformance`, `fixture-acceptance`, `ui-provider-boundaries` | `readiness-contract`, `readiness-acceptance`, `rehearsal-browser`, `rehearsal-conformance`, `preflight-freeze`, `sit-static`, `sit-browser`, `uat-product` | `ui`, `acceptance`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-01/09 |
| `ac-04` | S1, S3, S5 | `contract-boundary`, `failure-recovery`, `response-export-contract`, `response-incremental-sync` | `readiness-contract`, `readiness-materialization`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`, `uat-recovery` | `reference`, `faults`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/07 |
| `ac-05` | S1, S3, S5 | `contract-boundary`, `owner-product`, `ui-provider-boundaries` | `readiness-contract`, `rehearsal-conformance`, `rehearsal-source-sync`, `rehearsal-smoke`, `preflight-freeze`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-product`, `uat-providers` | `reference`, `faults`, `ui`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/09 |
| `ac-06` | S1, S3–S5 | `contract-boundary`, `consumer-cutover`, `core-subtraction`, `resource-resolution` | `readiness-contract`, `rehearsal-static`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-resource-resolution`, `uat-crossmodule`, `uat-subtraction` | `reference`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-05/06/11 |
| `ac-07` | S3, S5 | `contract-boundary`, `consumer-cutover`, `deployed-smoke`, `resource-resolution` | `rehearsal-conformance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-providers`, `uat-resource-resolution`, `uat-crossmodule` | `reference`, `faults`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/05/11 |
| `ac-08` | S3, S5 | `failure-recovery`, `deployed-smoke`, `response-incremental-sync`, `ui-provider-boundaries`, `reverse-consumers` | `readiness-materialization`, `readiness-acceptance`, `rehearsal-source-sync`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `rehearsal-recovery`, `preflight-freeze`, `sit-browser`, `sit-smoke`, `uat-providers`, `uat-reverse-consumers`, `uat-crossmodule`, `uat-recovery` | `reference`, `faults`, `ui`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/05/07/09/10 |
| `ac-09` | S3, S5 | `failure-recovery`, `response-incremental-sync`, `api-idempotency`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-rust`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`, `uat-recovery` | `reference`, `faults`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/07/11 |
| `ac-10` | S4–S5 | `core-subtraction`, `migration-seed`, `clean-materialization`, `fixture-acceptance` | `readiness-materialization`, `readiness-acceptance`, `rehearsal-materialization`, `preflight-freeze`, `sit-static`, `sit-smoke`, `uat-materialization`, `uat-subtraction` | `reference`, `acceptance`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-02/06 |
| `ac-11` | S5 | `clean-materialization`, `semantic-noop`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-materialization`, `rehearsal-source-sync`, `preflight-freeze`, `sit-smoke`, `uat-materialization`, `uat-replay-refresh` | `reference`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-02/04 |
| `ac-12` | S5 | `clean-materialization`, `failure-recovery`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-smoke`, `uat-providers`, `uat-recovery` | `reference`, `faults`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/07 |
| `ac-13` | S1, S4 | `contract-boundary`, `core-subtraction`, `inventory-navigation`, `reverse-consumers`, `resource-resolution` | `readiness-contract`, `rehearsal-static`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-reverse-consumers`, `uat-resource-resolution`, `uat-subtraction` | `reference`, `acceptance`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-06/10/11 |
| `ac-14` | S1, S3–S5 | `contract-boundary`, `consumer-cutover`, `core-subtraction`, `migration-seed`, `clean-materialization`, `ui-provider-boundaries` | `readiness-contract`, `readiness-materialization`, `rehearsal-static`, `rehearsal-materialization`, `rehearsal-source-sync`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-smoke`, `uat-providers`, `uat-subtraction` | `reference`, `faults`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/06/09 |
| `ac-15` | S1, S4–S5 | `core-subtraction`, `inventory-navigation`, `migration-seed`, `deployed-smoke` | `readiness-contract`, `readiness-materialization`, `rehearsal-static`, `rehearsal-materialization`, `rehearsal-smoke`, `preflight-freeze`, `sit-static`, `sit-smoke`, `uat-materialization`, `uat-operations`, `uat-subtraction` | `reference`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-02/03/06 |
| `ac-16` | S2–S3, S5 | `owner-product`, `deployed-smoke`, `reverse-consumers` | `readiness-acceptance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-browser`, `sit-smoke`, `uat-operations`, `uat-reverse-consumers` | `reference`, `faults`, `ui`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-03/10 |
| `ac-17` | S2, S5–S6 | `inventory-navigation`, `deployed-smoke`, `independent-upgrade-rollback` | `readiness-materialization`, `rehearsal-smoke`, `rehearsal-upgrade`, `preflight-freeze`, `sit-smoke`, `uat-upgrade` | `reference`, `upgrade`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-08 |
| `ac-18` | S3, S5 | `consumer-cutover`, `clean-materialization`, `deployed-smoke` | `readiness-materialization`, `readiness-acceptance`, `rehearsal-materialization`, `rehearsal-browser`, `rehearsal-smoke`, `preflight-freeze`, `sit-browser`, `sit-smoke`, `uat-crossmodule` | `reference`, `acceptance`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-05 |
| `ac-19` | S1, S3, S5 | `clean-materialization`, `semantic-noop`, `failure-recovery`, `fixture-acceptance`, `deployed-smoke`, `response-export-contract`, `response-incremental-sync`, `api-idempotency`, `dataset-refresh-dag` | `readiness-contract`, `readiness-materialization`, `rehearsal-materialization`, `rehearsal-source-sync`, `rehearsal-smoke`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`, `uat-recovery` | `reference`, `faults`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/07/11 |
| `ac-20` | S1–S3 | `contract-boundary`, `owner-product`, `ui-sdk-conformance`, `ui-provider-boundaries` | `readiness-contract`, `readiness-acceptance`, `rehearsal-browser`, `rehearsal-conformance`, `rehearsal-source-sync`, `preflight-freeze`, `sit-rust`, `sit-browser`, `uat-product`, `uat-providers` | `reference`, `faults`, `ui`, `acceptance`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-01/09 |
| `ac-21` | S1, S3–S4 | `contract-boundary`, `core-subtraction`, `resource-resolution` | `readiness-contract`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-resource-resolution`, `uat-subtraction` | `reference`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-11 |
| `ac-22` | S1, S3–S5 | `core-subtraction`, `deployed-smoke`, `reverse-consumers`, `resource-resolution` | `readiness-contract`, `readiness-acceptance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-operations`, `uat-reverse-consumers` | `reference`, `faults`, `ui`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-10 |
| `ac-23` | S1–S3 | `contract-boundary`, `owner-product`, `fixture-acceptance`, `api-idempotency` | `readiness-contract`, `readiness-acceptance`, `rehearsal-static`, `rehearsal-rust`, `rehearsal-conformance`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-browser`, `uat-scripted`, `uat-product`, `uat-resource-resolution`, `uat-replay-refresh` | `reference`, `acceptance`, `uat-contract`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-01/09/11 |
| `ac-24` | S1, S3, S5 | `failure-recovery`, `response-incremental-sync`, `api-idempotency`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-rust`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh` | `reference`, `faults`; `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/11 |
| `ac-25` | S2–S5 | `contract-boundary`, `owner-product`, `core-subtraction`, `response-incremental-sync` | `readiness-contract`, `readiness-materialization`, `rehearsal-static`, `rehearsal-rust`, `rehearsal-source-sync`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-subtraction` | `reference`; `offline` + `isolated-live` + `frozen-sit`; listed evidence | UAT-8B-04/06 |
| `ac-26` | S1, S3–S4 | `contract-boundary`, `owner-product`, `core-subtraction`, `reverse-consumers` | `readiness-contract`, `rehearsal-static`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-rust`, `uat-operations`, `uat-reverse-consumers`, `uat-subtraction` | `reference`, `faults`; `offline` + `isolated-live`; listed evidence | UAT-8B-03/06/10 |
| `gate-implementation-exit` | S1–S6 | `static-quality`, `contract-boundary`, `owner-product`, `ui-sdk-conformance`, `consumer-cutover`, `core-subtraction`, `inventory-navigation`, `migration-seed`, `clean-materialization`, `semantic-noop`, `failure-recovery`, `fixture-acceptance`, `runner-selftest`, `deployed-smoke`, `independent-upgrade-rollback`, `uat-readiness`, `response-export-contract`, `response-incremental-sync`, `ui-provider-boundaries`, `reverse-consumers`, `resource-resolution`, `api-idempotency`, `dataset-refresh-dag`, `planning-contract-alignment` | `readiness-contract`, `readiness-materialization`, `readiness-acceptance`, `rehearsal-static`, `rehearsal-rust`, `rehearsal-materialization`, `rehearsal-browser`, `rehearsal-conformance`, `rehearsal-source-sync`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `rehearsal-recovery`, `rehearsal-upgrade`, `rehearsal-uat`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-scripted`, `uat-product`, `uat-materialization`, `uat-operations`, `uat-providers`, `uat-reverse-consumers`, `uat-resource-resolution`, `uat-replay-refresh`, `uat-crossmodule`, `uat-subtraction`, `uat-recovery`, `uat-upgrade` | `reference`, `faults`, `upgrade`, `ui`, `acceptance`, `uat-contract`; `offline` + `isolated-live` + `frozen-sit`; all 24 `target:*`, all 31 `lane:*`, and all eleven `uat:*` | UAT-8B-01–11 |

## Roadmap Traceability Matrix

| Roadmap clause | Acceptance | Slice | Exact implementation targets | Formal lanes / UAT |
| --- | --- | --- | --- | --- |
| Independent Dataset deployment | `ac-01`, `ac-15`–`ac-16` | S2, S4–S5 | `owner-product`, `inventory-navigation`, `migration-seed`, `deployed-smoke` | readiness contract/materialization; SIT static/smoke; UAT-8B-02/03/06 |
| Move UI/API/revision/execution/materialization/persistence/config/diagnostics | `ac-01`–`ac-03`, `ac-16`, `ac-20`, `ac-23`–`ac-25` | S1–S4 | `owner-product`, `ui-sdk-conformance`, `ui-provider-boundaries`, `api-idempotency`, `dataset-refresh-dag` | rehearsal Rust/browser/conformance/source-sync; SIT Rust/browser; UAT-8B-01/03/09/11 |
| Fresh DB/downstream seed/new references; remove Core adapter/no old references | `ac-06`, `ac-10`–`ac-15`, `ac-21` | S3–S5 | `contract-boundary`, `resource-resolution`, `core-subtraction`, `migration-seed`, `clean-materialization`, `semantic-noop`, `failure-recovery` | rehearsal materialization/recovery; SIT static/smoke; UAT-8B-02/06/07/11 |
| Replace Response/other source-table reads with contracts | `ac-04`–`ac-05`, `ac-14`, `ac-19`–`ac-20`, `ac-24`–`ac-26` | S1, S3, S5 | `response-export-contract`, `response-incremental-sync`, `ui-provider-boundaries`, `dataset-refresh-dag`, `core-subtraction` | rehearsal source-sync/conformance/recovery; SIT Rust/smoke; UAT-8B-04/09/11 |
| Preserve Dataset-to-Component contract and scope | `ac-06`–`ac-08`, `ac-18`, `ac-21` | S3, S5 | `consumer-cutover`, `resource-resolution`, `deployed-smoke` | rehearsal reverse-consumers/smoke; SIT browser/smoke; UAT-8B-05/11 |
| Retain actual batch/catalog ownership | `ac-02` | S2–S3 | `owner-product`, `fixture-acceptance` | rehearsal Rust/browser; SIT Rust/browser; UAT-8B-01 |
| Rebuild canonical seed through owners; no foreign storage | `ac-10`–`ac-14`, `ac-25` | S4–S5 | `migration-seed`, `clean-materialization`, `fixture-acceptance`, `core-subtraction` | readiness/rehearsal materialization; SIT static/smoke; UAT-8B-02/06 |
| Retry/outage/scope/compatibility | `ac-07`–`ac-09`, `ac-12`, `ac-19`, `ac-22`–`ac-24` | S3, S5–S6 | `response-incremental-sync`, `reverse-consumers`, `api-idempotency`, `dataset-refresh-dag`, `failure-recovery`, `deployed-smoke` | rehearsal source-sync/reverse-consumers/recovery/smoke; SIT Rust/browser/smoke; UAT-8B-04/05/07/10/11 |
| Unchanged UI plus configuration/diagnostics/status | `ac-02`–`ac-03`, `ac-16`, `ac-20`, `ac-22` | S1–S3 | `ui-sdk-conformance`, `ui-provider-boundaries`, `reverse-consumers`, `owner-product` | rehearsal browser/conformance/reverse-consumers; SIT browser/smoke; UAT-8B-01/03/09/10 |
| Cross-process Dataset -> Component -> Dashboard exit | `ac-18` | S3, S5–S6 | `consumer-cutover`, `clean-materialization`, `deployed-smoke` | rehearsal smoke; SIT browser/smoke; UAT-8B-05 |

## Ordered Implementation Slices

### S1 — Freeze contracts and executable subtraction inventory

Prerequisites: this plan and validated policy-v2 contract.

- Advance Dataset contract to v2 Module Instance ownership and define exact
  Dataset/DatasetRevision/major-line reference schemas. First-party code must
  consume canonical types directly: no mirror DTO, permissive
  `serde_json::Value`, missing-major default, alias, or semantic re-encoding.
- Define Response export, FormVersion catalog/schema, Core scope-catalog and
  principal-display-catalog, Dataset reverse-consumer, operational-status, and
  resource-observation contracts,
  service actions, authorization/nondisclosure, timeout, compatibility, and
  snapshot rules. Freeze the Response head/checkpoint request, opaque monotonic
  cursor, inclusive snapshot bound, ordered paged upsert/tombstone envelope,
  full-rebase expiry recovery, partition/staging/promotion state, and
  transactional sequence-allocation rules.
- Freeze the complete public/private route/action matrix, exact media types,
  pre-deserialization raw-byte digest validation, idempotency/replay identity,
  and route-specificity precedence for static paths such as `/datasets/new`,
  `/sql-preview`, and `/refresh` versus `{dataset_id}` siblings.
- Capture/hash the full current-main UI baseline and create the tracked logical
  fixture contracts at
  `deploy/sprint-8b/fixtures/reference-fixture-contract.json`,
  `deploy/sprint-8b/fixtures/provider-fault-contract.json`, and
  `deploy/sprint-8b/fixtures/upgrade-fixture-contract.json`; create
  `docs/sprints/sprint-8b-uat/scenario-contract.json`,
  `docs/sprints/sprint-8b-test-change-log.md`, and the exact Sprint 8B
  additions to `end2end/acceptance-manifest.json` schema v2.
- Create the thin implementation-readiness runner skeleton now with every
  one of the 24 targets selectable, failing until its slice is implemented. Add the plan/
  record/contract/fixture set-equality linter and representative path-impact
  self-tests before product movement.
- Add failing boundary tests for old v1 owners/types/payloads, direct table and
  credential access, Core residue, duplicate inventory/navigation, provider
  substitution, missing contract declarations, and definition-ID branches.
- Encode the complete Core subtraction and provider/consumer inventories in
  repository-owned checks and map every target to this validation contract.

Expected touchpoints: contract crates, module-contract fixtures/tests,
workspace manifests, Core provider declarations, UI baseline index, fixture/
acceptance/test-change contracts, implementation-runner skeleton, boundary
scripts, and focused integration fixtures. Tests change in this slice.

Complete when every target contract and every forbidden dependency/residue is
an executable failing/passable assertion and no implementation relies on a
provisional wire shape.

### S2 — Establish the independent Dataset owner and UI baseline

Prerequisites: S1 contract tests fixed.

- Create module/UI crates, manifest `1.0.0`, database baseline (including
  imported source, sync partition/attempt/staging, replay, bootstrap, and
  materialization receipt tables), runtime,
  migration, configuration, diagnostics, probes, routes, assets, capabilities,
  service actions, and module-owned persistence/bootstrap testkit.
- Freeze a separately built real Dataset `0.9.0` image/manifest/source fixture
  before `1.0.0` implementation diverges. It provides Dataset v2, reads the
  compatible additive schema after rollback, and has exact provenance.
- Move/adapt typed Dataset UI to SDK document/lifecycle composition against the
  S1 baseline and add direct/lifecycle light/dark responsive visual cases.
- Move Dataset-only rules out of `style/core.css`; rename shared
  `tessara-web-data-ops` classes/styles to policy-neutral `data-ops-*`; remove
  root web/Core product build dependencies.
- Build assets with
  `pwsh -NoProfile -File scripts/build-module-ui-browser-assets.ps1 -Module all -Check`
  and reconcile digests with manifest/release catalog/loader.
- Add module store, API, UI, configuration, diagnostics, and health tests plus
  mandatory `ui-sdk-conformance` coverage before traffic cutover.

Expected touchpoints: new Dataset module/UI crates and migrations, existing
Dataset UI/domain crates, module manifest, asset builder/catalog, Compose image,
visual tests/snapshots, conformance scripts. Tests and visual harness change in
this slice.

Complete when Dataset independently builds, migrates, renders, hydrates,
configures, diagnoses, and passes SDK/source/package isolation before serving
product consumers.

### S3 — Move product behavior, source providers, and consumers

Prerequisites: S1–S2.

- Move CRUD, revision, review, the complete QuerySpec operation pipeline,
  directory/tags/provider catalogs and bootstrap-validation batch,
  materialization, preview/execution, lifecycle, and persistence to Dataset.
- Implement Core transition providers for Response/Form/scope/principal
  contracts and replace every direct source/storage query with signed client
  calls. Consolidate all Response mutation producers behind the transactional
  owner primitive/change log. Implement Dataset source import, explicit
  refresh, full rebase, local compile/materialization, CAS promotion, and
  topological downstream-Dataset refresh exactly as frozen in S1.
- Add durable replay handling for all product writes and exact raw-byte/action/
  audience/nonce validation for private provider calls.
- Cut Components through immutable release `1.1.0` to Dataset v2 in
  contract/client/manifest/store/bootstrap/UI, remove all Core URL/owner and v1
  fallback assumptions, and preserve Dashboard `3.0.1` behavior via Components
  v3.
- Cut Forms reverse usage, Core Operations, application summary, and generic
  resource observation to Dataset-owned actions with exact outage/
  nondisclosure UI states.
- Advance every API client, fixture, generated input, smoke assertion, and UAT
  predicate with the cutover.
- Prove scope, authorization, nondisclosure, timeouts, incompatibility, retries,
  outage, and recovery across real processes/databases.

Expected touchpoints: Dataset owner, Core provider handlers, Dataset contract,
Component module/contracts, deployment wiring, integration and browser tests.
Provider/consumer/harness tests change in this slice.

Complete when no Dataset product path or first-party consumer needs Core Dataset
implementation/storage/v1 shapes; every inventoried direct provider/consumer
path (Dataset browser, Dataset-on-Dataset, Components, Forms, Operations, app
summary, and reference resolution) passes focused integration; and Dataset ->
Component -> Dashboard passes focused deployed diagnostics.

### S4 — Remove Core Dataset ownership

Prerequisites: S3 consumer cutover.

- Delete Dataset product schema/baseline entries, handlers/routes/DTOs,
  compatibility provider, web routes/feature branch, seed writes, static
  capabilities/grants, and transition descriptor.
- Delete Core Dataset authorization/tier code; composition/bootstrap Dataset
  input/writes/read-back; app-summary/Operations/Forms Dataset SQL; transition
  reference/destination/navigation/catalog branches; Core service-provider
  Dataset declarations; root route guards/icon policy; Dataset CSS; and active
  test/demo/local-script cross-owner writes.
- Enroll `tessara.datasets` exactly once and let generic manifest projection own
  configuration, diagnostics, inventory, routing, title, and navigation.
- Run source/schema/runtime inventory searches and explain only intentional
  module registration, source-provider, historical, or negative matches.

Expected touchpoints: Core API/web/migrations/demo/module catalog/auth tests,
transition fixtures, smoke and acceptance inventories. Negative and boundary
tests change in this slice.

Complete when Core and root web demonstrate zero Dataset product ownership or
fallback and the four-transition plus three-real-module inventory is exact.

### S5 — Rebuild the reference application from empty

Prerequisites: S1–S4.

- Add `deploy/sprint-8b`, a thin source-exact materializer, owner bootstrap
  contracts, fixture contract, read-back identity flow, gateway-start boundary,
  exact health/provenance, and database isolation audits.
- Add separately addressable Response/Form/scope/principal fault proxies and an
  incompatible/malformed provider double so one binding can fail without
  removing Core auth/control plane. Bind exact proxy provenance and forbid
  product/DB bypass.
- Prove first apply and semantic no-op from authenticated empty disposable
  environments.
- Prove incremental Response synchronization for unchanged head, multiple
  ordered pages, tied timestamps, late creation, update/resubmission,
  redaction/deletion tombstones, interruption between pages, retry from the
  last committed cursor, authorized empty pages, concurrent allocation/refresh,
  provider epoch change, full-rebase cursor expiry, scope/snapshot
  substitution, and the exact base -> derived -> second-hop Dataset closure.
  Prove one atomic promotion visible through Component/Dashboard, an unchanged
  independent binding, cycle rejection before staging, and whole-closure
  rollback after an injected derived rebuild failure.
- Inject deterministic pre-write and bounded mid-apply failure, retain
  diagnostics, remove exact partial topology, and prove clean successor apply,
  no-op, and restoration.
- Ensure seed/test preparation uses only owner APIs and never another owner's
  credentials or product tables.

Expected touchpoints: deployment profile, Blueprint/catalog, module bootstraps,
materialization/failure/fixture scripts, smoke, Playwright, UAT predicates.
All harness changes land with owner behavior.

Complete when a clean disposable topology proves owner-only materialization,
typed read-back, no-op, failure containment, recovery, final health, and the
cross-module product exit without manual repair.

### S6 — Close the implementation acceptance cone

Prerequisites: S1–S5 complete.

- Complete/aggregate the S1 thin implementation-readiness runner and add thin
  readiness, rehearsal, preflight, SIT, UAT, and evidence-publication runners
  over current shared policy-v2 contracts; do not copy Sprint 8A lineage/state
  machinery. S6 cannot introduce a target or acceptance clause for the first
  time.
- Run every required implementation target and self-test, including static
  quality, contracts, owner, UI, consumer, subtraction, inventory,
  materialization/no-op/recovery, fixtures, smoke, and UAT predicates.
- Use the S2-frozen Dataset `0.9.0` to prove
  `0.9.0 -> 1.0.0 -> 0.9.0 -> 1.0.0`
  with unrelated owners unchanged, and restore the intended topology.
- Produce a passing non-authoritative `implementation-readiness-result.json`
  with `run-sprint-8b-implementation-readiness.ps1 -Finalize` for one clean,
  committed source tree, all 24 current-source authenticated target receipts,
  zero known failures, and the tracked contract hash. Finalization remains
  fail-closed while implementation changes are uncommitted; it does not infer
  authority to create that commit or weaken source identity to the dirty tree.

Expected touchpoints: Sprint 8B runner scripts, fixture/acceptance contract,
upgrade manifests/images, validation evidence schemas/consumers only as needed.
Runner self-tests and acceptance predicates change in this slice.

Complete when all policy-v2 implementation proof classes pass and formal
Validation Readiness has no known product, harness, fixture, environment,
acceptance, deployment, or evidence defect left to discover.

## Verification And Validation Plan

### Exact implementation targets

The tracked validation contract requires exactly 24 repository-owned target
commands. S1 creates the selectable runner skeleton; each slice turns its
targets green and S6 only aggregates them:

- `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target static-quality`
- the same runner with `contract-boundary`, `owner-product`,
  `ui-sdk-conformance`, `consumer-cutover`, `core-subtraction`,
  `inventory-navigation`, `migration-seed`, `clean-materialization`,
  `semantic-noop`, `failure-recovery`, `fixture-acceptance`, `runner-selftest`,
  `deployed-smoke`, `independent-upgrade-rollback`, `uat-readiness`,
  `response-export-contract`, `response-incremental-sync`,
  `ui-provider-boundaries`, `reverse-consumers`, `resource-resolution`,
  `api-idempotency`, `dataset-refresh-dag`, and
  `planning-contract-alignment`.

`ui-sdk-conformance`, `migration-seed`, `clean-materialization`,
`semantic-noop`, `failure-recovery`, `response-incremental-sync`, and
`dataset-refresh-dag` use newly authenticated isolated disposable
databases/topologies and declare `clean_environment: true`. The migration/seed
target uses the complete owner-ordered Reference apply and typed fixture
read-back rather than the focused Core/Dataset unit aggregate. The UI target
creates an owned Reference topology
only after the ten source-derived, tracked Dataset visual baselines pass its
preflight, executes the exact four Dataset workflow plus seven visual
predicates, and proves exact teardown.
Every target emits
`artifacts/sprint-8b-closeout/implementation/targets/<target>/result.json`,
`command.log`, and evidence references; the aggregate records all 24 receipts.

### Baseline commands retained inside targets or formal lanes

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-features --locked --offline --jobs 1`
- `pwsh -NoProfile -File scripts/check-web-crate-boundaries.ps1`
- `pwsh -NoProfile -File scripts/verify-module-sdk-boundaries.ps1`
- `pwsh -NoProfile -File scripts/ui-sdk-conformance.ps1`
- `pwsh -NoProfile -File scripts/build-module-ui-browser-assets.ps1 -Module all -Check`
- `npm --prefix .\end2end test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

The generic smoke/local-launch/UAT commands remain baseline acceptance and must
be advanced to discover the active Sprint 8B profile. Sprint-specific formal
runners own exact evidence paths and environment bindings.

### Exact formal lane selectors and isolation

Every ID below is invoked literally with the shown runner and writes
`artifacts/sprint-8b-closeout/<phase>/lanes/<lane>/result.json`, `command.log`,
and an evidence-reference list. The phase runner writes its compact certificate
and sealed `evidence-index.json` under the phase root. Live lanes use dedicated
Compose project/database names `tessara-s8b-<lane>` and perform owner-API fixture
prepare plus canonical-restoration receipt; offline lanes launch no stack.

| Phase / exact command form | Ordered lane IDs | Environment |
| --- | --- | --- |
| `pwsh -NoProfile -File .\scripts\validate-sprint-8b-readiness.ps1 -Lane <id>` | `readiness-contract`, `readiness-materialization`, `readiness-acceptance` | contract/acceptance offline; materialization isolated live |
| `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane <id>` | `rehearsal-static`, `rehearsal-rust`, `rehearsal-materialization`, `rehearsal-browser`, `rehearsal-conformance`, `rehearsal-source-sync`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `rehearsal-recovery`, `rehearsal-upgrade`, `rehearsal-uat` | static offline; every behavior lane isolated live and restored |
| `pwsh -NoProfile -File .\scripts\run-sprint-8b-validation-preflight.ps1` | `preflight-freeze` | offline identity/evidence audit; emits `candidate.json` |
| `pwsh -NoProfile -File .\scripts\run-sprint-8b-sit.ps1 -Lane <id>` | `sit-static`, `sit-rust`, `sit-browser`, `sit-smoke` | one frozen `tessara-s8b-sit` topology, strict listed order, restoration checked after each live lane |
| `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane <id>` | `uat-scripted`, `uat-product`, `uat-materialization`, `uat-operations`, `uat-providers`, `uat-reverse-consumers`, `uat-resource-resolution`, `uat-replay-refresh`, `uat-crossmodule`, `uat-subtraction`, `uat-recovery`, `uat-upgrade` | each live lane uses `tessara-s8b-<lane>` and its own prepared/restored fixture receipt |

Lane prerequisites in the machine contract encode exactly this order, so two
stateful siblings never mutate one topology concurrently. A prerequisite
certificate hash change invalidates every downstream certificate even when its
source fingerprint would otherwise be unchanged. Candidate Rehearsal lanes are
pre-freeze certification, not an implementation/debug loop.

### Formal receipt chain

1. Implementation publishes a passing non-authoritative implementation
   readiness result for clean source and exact contract hash.
2. Validation Readiness executes or inherits only authenticated unaffected v2
   lanes according to domain fingerprints; uncertainty runs the full phase.
3. Candidate Rehearsal certifies the mutable candidate composition and issues a
   compact certificate with sealed phase-local evidence index.
4. Preflight authenticates both certificates and freezes the exact candidate.
5. Complete candidate-bound SIT runs static/boundary, Rust, Playwright, and
   deployed acceptance smoke; deployed smoke belongs inside SIT.
6. Complete UAT runs scripted predicates and all eleven manual scenarios.
7. The coordinator authenticates compact certificates and `evidence-chain.json`;
   closeout performs one final complete integrity audit of sealed phase indexes.

A candidate or tracked harness change invalidates downstream evidence. Before
freeze, policy-v2 permits only dependency-fingerprint-authenticated affected-
lane recertification with prerequisite closure. Unknown paths or fingerprints
select conservative execution. After freeze, any candidate change requires a
successor freeze and complete SIT plus complete UAT; no SIT or manual UAT lane
is inherited between candidates.

## Manual UAT Inventory

`docs/sprints/sprint-8b-uat/scenario-contract.json` is the exact tracked manual
inventory. Each identity below declares role, authenticated start state,
ordered actions, assertion IDs, evidence and cardinality, isolation/cleanup,
and canonical restoration; formal UAT discovery and its eleven `uat:<id>`
receipts must be set-equal to that file.

| Scenario | Role and observable pass condition |
| --- | --- |
| UAT-8B-01 Product parity and UI | Dataset manager authors, previews, revises, publishes, and inspects Dataset/batch/catalog behavior through direct and lifecycle routes with accepted visual/responsive/accessibility behavior |
| UAT-8B-02 Fresh materialization | Operator applies empty topology, verifies exact releases/instances and owner receipts, then observes unchanged semantic second apply |
| UAT-8B-03 Configuration and diagnostics | Administrator validates/applies Manifest configuration and sees exact health plus sanitized provider diagnostics through generic Module Management |
| UAT-8B-04 Provider contracts, cursor, scope, and Dataset DAG | Scoped actor invokes synchronous refresh, observes unchanged-head zero-page/no-mutation behavior, applies new/corrected/status-out/status-in/redacted/deleted changes across ordered and authorized-empty pages, interrupts/retries, races a refresh, and runs an expired-cursor full rebase. The actor then changes `dataset.base` and proves `dataset.base` -> `dataset.derived` -> `dataset.derived-second-hop` plus the downstream Component/Dashboard result appear in one complete generation while `dataset.independent-binding` remains byte/identity stable; a cycle is rejected before staging; an injected derived rebuild failure rolls back the entire closure/cursor/receipt and leaves Component/Dashboard on the prior last-good result; disjoint/restricted/unknown sources remain nondisclosing throughout |
| UAT-8B-05 Cross-module exit and outage | Actor executes Component over Dataset and views Dashboard result, stops Dataset/provider, observes coherent degradation, restores it, and sees healthy convergence |
| UAT-8B-06 Core subtraction/isolation | Operator proves four Core transitions, one Dataset enrollment/navigation item, no Core Dataset schema/adapter/payload, and pairwise credential/database isolation |
| UAT-8B-07 Failure retry and recovery | Operator inspects retained failed apply, exact teardown, clean successor apply, retry/no-op, and canonical health without manual repair |
| UAT-8B-08 Independent upgrade/rollback | Operator upgrades/rolls back/restores only Dataset with state preserved and unrelated owner images/containers/restarts/data/navigation unchanged |
| UAT-8B-09 Editor provider boundaries | Dataset manager exercises Form/version pickers, rendered field options, scope tree, updater display choices, direct/lifecycle hydration, and each isolated provider fault while network evidence proves the browser calls only Dataset routes and retains unsaved input |
| UAT-8B-10 Reverse consumers and Operations | Authorized/disjoint actors inspect a Form's Dataset Sources tab, `/operations` Dataset readiness/attention, and app summary; Dataset outage yields explicit unavailable state while unrelated Form/Operations/summary content remains usable and recovery restores exact scoped results |
| UAT-8B-11 Resource, replay, and routing | Actor resolves all three Dataset v2 resource types, compares known/random restricted cases, retries each representative mutation after response loss, rejects changed replay input/private nonce reuse, and proves static editor/SQL-preview/refresh routes are not captured as Dataset IDs |

## Risks And Controls

| Risk | Prevention | Detection | Recovery |
| --- | --- | --- | --- |
| Hidden cross-database source read | executable package/SQL/credential boundary inventory | source scan, process integration, credential-denial tests | return to S1/S3; remove coupling before cutover |
| Browser retains direct Core provider calls | exact Dataset editor-option routes and ShellContext-only actor input | browser network denylist plus direct/lifecycle picker tests | remove direct client; restore unsaved editor fixture and rerun UI provider cone |
| Scope/nondisclosure regression across hops | request-bound grants and exact owner scope rules | known/random/disjoint/restricted matrix | invalidate candidate; correct product and rerun affected pre-freeze lanes/full successor SIT+UAT |
| Contract v1 silently accepted | v2-only constructors and negative fixtures | contract/boundary/provider tests | remove alias/reader; recreate disposable state |
| Mirror DTO or semantic re-encoding drifts from provider | canonical contract types consumed directly; no permissive `Value`/aliases | package/source audit plus unknown-field/reference-order/range negatives | delete mirror/facade and move consumer to canonical type |
| Signed request is parsed before media/digest validation | validate media type, raw size, grant, and received-byte digest before deserialization | wrong media, whitespace mutation, replay, expiry, action/path/body substitution tests | reject request with module-owned envelope; write no state |
| Partial materialization or duplicate retry | durable idempotency/input/snapshot identity before writes | injected pre/mid-write failures and second-run comparison | exact teardown and from-empty successor |
| GET accidentally pulls/promotes source data | reads may observe head only; page pull/promotion requires an explicit synchronous trigger | request-trace and cursor/row/receipt before-after tests on every GET/provider read | remove write-on-read path; restore last-good state and rerun refresh/browser targets |
| Concurrent or stale cursor attempt promotes mixed state | partition single-writer lock/CAS and starting-generation verification | racing refresh, post-commit response loss, stale-attempt and independent-partition matrix | retain winning receipt; reject stale promotion and replay authenticated input |
| Dataset DAG partially advances | one topological closure transaction and cycle rejection before staging | base/derived/second-hop, injected derived failure, independent-binding and downstream Component/Dashboard assertions | roll back projection/cursor/generations/receipts; keep prior last-good closure |
| Reverse provider outage renders as empty/zero | typed `available`/`empty`/`unavailable`/`undisclosed` states and availability-bearing summary | Form, Operations, summary scoped/outage/recovery integration and UAT | restore provider; never synthesize zero/link-free success from outage |
| UI visual/hydration drift | accepted baseline, SDK typed construction, early conformance | visual matrix, lifecycle/direct parity, console/accessibility tests | correct module UI/SDK usage before consumer cutover |
| UI extraction weakens existing behavior or adds external assets | exact manifest test identities, hashed current-main baseline, no-network/system-theme/JS-disabled assertions | test-change audit, exact discovery, screenshots, console and network evidence | restore accepted behavior; log any stronger replacement before rerun |
| Static route is captured by identifier sibling | one manifest/router/gateway/client route set with explicit static precedence | `/datasets/new`, `/sql-preview`, `/refresh`, revision edit/detail specificity matrix | correct generic route ordering; no definition-specific gateway branch |
| Duplicate inventory/navigation | remove transition before enrollment projection; exact identities | source/runtime inventory and smoke | remove overlap and rematerialize cleanly |
| Stale fixture or copied identity/count | owner bootstrap and typed read-back | semantic fixture contract and acceptance audit | rebuild fixture from empty; do not patch copied IDs |
| Consumer/UI proof remains green after baseline or test-log change | track UI baseline index, acceptance manifest, test-change log, helper scripts, and consumer sources in exact dependency domains | planning-contract path-impact self-tests and affected-lane selection | invalidate affected proof and rerun prerequisite closure before freeze |
| Browser scenario depends on serial residue or weakened selector | scenario-owned setup/cleanup and stable accessible/product identities | isolated plus complete exact-manifest execution; no skip/retry/filter/max-failure truncation | repair fixture/selector without weakening product expectation |
| Provider outage misclassified as invalid input | stable failure vocabulary and bounded client policy | dependency fault matrix and sanitized diagnostics | restore provider and replay authenticated idempotent operation |
| Timestamp watermark loses or duplicates Response changes | provider-owned transactional sequence, stable snapshot bound, upsert/tombstone envelope, commit cursor with Dataset state | tied-time/late/update/delete/multipage/interruption/expired-cursor matrix | retain prior cursor and serve last-good while an authenticated fixed-upper-bound full-snapshot rebase stages and atomically replaces it |
| Health/config/release proof accepts labels instead of executable identity | exact commands, probe path/status/content/body, config schema/defaults, source-built release and catalog digest | no-redirect health checks, container command/provenance read-back, config negatives, upgrade identity comparison | correct manifest/catalog/image together; never relabel candidate as prior release |
| Windows full Rust lane misses feature graph or collides on PDB | all-features locked/offline workspace test with `--jobs 1` | exact command receipt and native SSR/module tests | fix product or harness; do not narrow features/tests |
| Thin runner repeats Sprint 8A exit/environment/reader defects | immediate child-exit capture, safe optional Compose projection, loopback canonicalization, environment restoration, current-shape readers, all-pass finalization self-tests | `runner-selftest` adversarial matrix | correct policy-neutral adapter and invalidate affected pre-freeze evidence |
| Rollback reactivates removed Core path | release-aware module-only delta; no adapter in source | image/container/restart and Core-residue comparison | restore last healthy Dataset release only |
| Validation runner becomes sprint-specific framework | thin profile over policy v2 with self-tests | runner/source boundary audit | extract only policy-neutral capability; delete copied state machine |

## Planning Audit

- Every Outcome, Build, UI, and exit-condition clause maps through the
  traceability matrix to specifications, acceptance criteria, a dependency-
  ordered slice, exact automated targets, formal lanes, and manual UAT.
- UI, API, persistence, authorization, integration, deployment, compatibility,
  migration, observability, recovery, and rollback are covered. Production
  data migration/import and new product features are explicitly out of scope.
- Happy, negative, boundary, nondisclosure, outage, retry, incompatibility,
  failure containment, recovery, no-op, upgrade, and rollback paths are explicit.
- Harness/fixture/manifest/bootstrap changes are paired with the product slice
  that invalidates them.
- The validation record and machine contract use the same identities,
  requirements, targets, lanes, dependency domains, environment, and evidence
  policy.
- The exact planning inventory is 26 canonical `ac-01`–`ac-26` criteria plus
  `gate-implementation-exit`, 24 implementation targets, four source-provider
  contracts, three Dataset-owned reverse-consumer contract families, and
  eleven manual UAT scenarios. Set equality, not copied totals, is the
  executable rule for every inventory.
- The Phase 8 profile binds the canonical playbook and all sixteen proof
  classes; clean materialization, no-op, and recovery are clean-environment
  targets.
- Planning writes are confined to the Sprint 8B worktree. `main` remains clean,
  and no product source, test, migration, fixture, script, manifest, deployment
  configuration, or generated product asset was changed during kickoff.

## Implementation Handoff

Recommended first slice: **S1 — Freeze contracts and executable subtraction
inventory**. It establishes Dataset v2 Module Instance ownership, all four
source-provider contracts, all three Dataset-owned reverse-consumer contract
families, the 26 canonical `ac-01`–`ac-26` criteria plus
`gate-implementation-exit`, exact 24-target alignment, the tracked test-change
log and schema-v2 acceptance manifest identities, and a selectable failing
runner skeleton before module scaffolding or consumer cutover can hide an
ownership gap.

Implementation requires a separate explicit request. Kickoff does not
authorize S1 or any validation phase.

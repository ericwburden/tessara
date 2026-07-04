# Sprint 4A Plan: Table Component Slice

Kickoff status: started from clean `main` on 2026-07-03.

## Sprint Summary

Build Sprint 4A from the roadmap `(Next)` scope: make table-oriented presentation assets first-class components. The sprint delivers `DetailTable` and `AggregateTable` authoring, component draft/version/publish workflows, Dataset major-line binding, validation against major-line contracts, scoped visibility, and application table viewers.

Key correction from kickoff: table components bind to Dataset major version lines, not individual Dataset revisions. A Dataset major line is the execution data surface containing all published minor/patch revisions in that major line.

Kickoff defaults:

- Branch: `codex/sprint-4a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4a`
- Plan artifact: `docs/sprints/sprint-4a-plan.md`

## Settled Decisions

- Sprint 4A authoring supports only `DetailTable` and `AggregateTable`. Future chart/stat component kinds remain in internal enums for compatibility, but Sprint 4A public authoring, validation, and publish flows reject unsupported kinds.
- Component versions bind major-line only: store `dataset_id`, `dataset_version_major`, and `binding_mode = major_line`; enforce `binding_mode = 'major_line'` at the schema boundary. Component versions do not retain a `dataset_revision_id` compatibility field.
- The pre-Sprint component API/schema was revision-bound. Sprint 4A migrates component persistence, DTOs, visibility joins, dashboard joins, demo seed logic, and dependency impact code to major-line binding.
- Component configs validate against the Dataset major-line field contract. The existing dataset major-line source catalog and `dataset_major_materializations` direction should be reused.
- `DetailTable` presents row-level records from the full bound major-line data surface.
- `AggregateTable` owns an aggregation plan over the full bound major-line data surface. It must reuse data-operation semantics from a dedicated shared crate used by both Datasets and Components; do not create a parallel component-only aggregation engine.
- Materialization is a render-time/cache concern. Publish validates binding, contract, config, and authorization; it does not fail only because the major-line materialization is absent or expired.
- Table viewers use the Rust/UI Data Table pattern and server-driven interactions so search, filters, sorting, and pagination operate across the full major-line data surface.
- Add `crates/tessara-web-components` as the component feature crate. Root `tessara-web` remains responsible for native route adapters, route params, auth/session guard integration, app shell, navigation, document metadata, hydration, CSS/assets, and cargo-leptos ownership.
- All touched component routes remain native Leptos SSR routes. Do not add bridge ownership, `inner_html`, or JavaScript-controller-owned UI.
- Component `AggregateTable` authoring and Dataset Aggregation operation authoring reuse the neutral `tessara-web-data-ops` aggregation editor, with Components using metrics-only mode so group/metric behavior, metric labels such as `Count values present`, source-field eligibility, and multi-metric editing stay consistent without feature-crate cross dependencies.

## Resolved Decision Points

- Component lifecycle: at most one working draft version per component. Published and superseded versions are immutable. Editing a published component creates or updates the one working draft. Publishing the draft atomically supersedes the previous published version. Add a partial unique index on `component_versions(component_id)` where `status = 'draft'`.
- Reader lifecycle boundary: public reader routes list and load only published component versions. Management/admin routes expose the working draft plus published/superseded history to users with `components:manage`. Working drafts are authoring state and are not active downstream consumers.
- Publish route: use an explicit publish endpoint as the canonical UI/test flow: `POST /api/admin/components/{component_id}/versions/{version_id}/publish`. Publish locks the component and target draft, then loads, authorizes, and validates the locked draft inside the transaction before superseding prior published versions. The existing `publish` boolean on create-version may remain temporarily for compatibility, but the app should not rely on it.
- Component shell metadata: `name`, `slug`, and `description` are mutable shell metadata and are not versioned in Sprint 4A. Add `PATCH /api/admin/components/{component_id}`. Slug changes update the canonical route; redirects are deferred.
- Revision-to-major-line migration: add `dataset_id`, `dataset_version_major`, and `binding_mode`; remove component-version `dataset_revision_id` from schema/API/frontend contracts rather than preserving a nullable legacy field.
- Major-line field contract: use the latest published/superseded revision contract in the major line as the field surface, matching current major-line materialization behavior. Field keys and compatible field types are contract-bearing. Labels are display metadata for Sprint 4A and should not force a new major line by themselves.
- Compatible same-major changes: adding rows, adding optional fields, adding compatible values, and non-breaking metadata/label changes.
- Breaking changes: removing a used field, renaming a field key, changing field type incompatibly, tightening/nullability or semantics in a way that breaks filters/sorts/aggregates, or changing metric source compatibility.
- Data-operation extraction: create a dedicated shared crate, `crates/tessara-data-ops`, for pure data-operation contracts and validation used by both Datasets and Components. SQL compilation and API/database orchestration stay in `tessara-api`.
- Aggregation grammar: support group fields, metrics, pre-aggregation filters, post-aggregation filters, sorting by group or metric output fields, and pagination of aggregate rows. Defer custom expressions, window functions, date/time bucketing beyond current field semantics, calculated metrics not already available, and row-picker semantics unless reuse makes them effectively free.
- Aggregate functions: expose `count`, `count_values`, `count_distinct`, `sum`, `avg`, `min`, and `max` for components in Sprint 4A. Reuse existing Dataset `count`, `count_values`, `sum`, `avg`, `min`, and `max`; add `count_distinct` to the shared operation facade. `count_values` means a source-field count of non-empty values, not a distinct-value frequency distribution.
- Shared aggregate validation preserves Dataset `min`/`max` ordering semantics for text/static-text, choice-like, numeric, date, datetime, and timestamp fields. Components must not expose a narrower `min`/`max` rule than Datasets.
- Materialization pending behavior: render returns an empty table with `materialization_state = pending` when major-line materialization is missing/expired and a ready table cannot be returned immediately. Synchronous rebuild can run behind the boundary, but the stable API state remains pending/loading/retry rather than a component validation error. TTL/expiration policy is out of scope.
- Component table execution: server-driven by default. Use a simple opaque cursor, `page_size`, and single-sort only in Sprint 4A. Multi-sort is deferred.
- Viewer projection contract: `visible_columns` controls rendered output columns only. Search, sort, and filter controls continue to use the full table contract so hiding a column does not remove it from server-side query behavior.
- Dashboard placement boundary: dashboards may place only immutable published-history component versions (`published` or `superseded`). Draft component versions cannot be placed on dashboards.
- Dashboard placement policy: a dashboard manager may place a published-history component version when they can manage the dashboard's full visibility scope, the dashboard visibility fully encompasses the component Dataset visibility, and the component version is immutable published history rather than a draft. Sprint 4A governs placement through dashboard manage scope plus visibility containment; it does not add a separate `components:manage` requirement for dashboard composition.
- Dependency reporting boundary: Dataset revision history dependency summaries and dependency-impact rows count/show component versions only after publication (`published` or `superseded`). Working drafts do not inflate downstream dependency counts.
- Authorization: require `components:manage` for create/edit/publish; require Dataset read visibility to list/select a Dataset major line; require component-management scope to bind/publish that Dataset major line. Do not require `datasets:manage` unless editing the Dataset itself. Scoped read visibility is audience-based: a user with the required read capability may list or load an asset when at least one asset visibility node overlaps the user's scoped nodes. Explicit historical component-version table rendering authorizes the selected version's bound Dataset, not only the component's current published visibility. Scoped authoring authority is governance-based: a user with the required manage capability may create, bind, edit, publish, or place an asset only when their manage scope fully contains every visibility node implicated by the asset and its bound data sources; component management routes require containment over the component's existing version history before metadata, draft, or publish mutations.
- Column picker persistence: component config stores default visible columns/order. Viewer interactions are session/view state and do not mutate the component version. Per-user persisted preferences are deferred.
- Direct create flow: the backend exposes one canonical atomic create endpoint for shell plus first draft. The UI uses that endpoint; shell-only creation can remain an admin/test helper but is not the normal app flow.

## API And Data Contracts

Component management routes:

- `GET /api/admin/components`
- `POST /api/admin/components` creates shell plus first draft atomically for the app flow
- `PATCH /api/admin/components/{component_id}`
- `GET /api/admin/components/{component_ref}` where `component_ref` may be a component UUID or slug
- `POST /api/admin/components/{component_id}/versions` creates or updates the one working draft for an existing component
- `PATCH /api/admin/components/{component_id}/versions/{version_id}`
- `POST /api/admin/components/{component_id}/versions/{version_id}/publish`
- `POST /api/admin/components/validate`

Reader/viewer routes:

- `GET /api/components`
- `GET /api/components/{component_ref}`
- `GET /api/components/{component_ref}/table`
- `GET /api/components/{component_ref}/versions/{version_id}/table`

Application routes:

- `/components`
- `/components/new`
- `/components/:component_ref`
- `/components/:component_ref/edit`
- `/components/:component_ref/publish`
- `/components/:component_ref/view`

Core binding shape:

```text
ComponentDatasetBinding {
  dataset_id,
  dataset_version_major,
  binding_mode = "major_line"
}
```

Useful non-binding metadata:

```text
validated_against_line_contract_hash
validated_against_head_revision_id
line_head_revision_id
line_materialization_id
line_materialized_at
included_revision_count
```

Avoid `resolved_dataset_revision_id` for execution metadata because it implies one revision is the execution source. Prefer `line_head_revision_id` or `line_materialization_id`.

`DetailTable` minimum config:

```text
columns
default_sort
default_filters
search_fields
page_size
empty_state
```

`AggregateTable` minimum config:

```text
pre_filters
group_fields
metrics
post_filters
default_sort
page_size
presentation
```

Metric functions exposed in Sprint 4A component config are `count`, `count_values`, `count_distinct`, `sum`, `avg`, `min`, and `max`. `count_values` uses the existing shared semantics, `COUNT(NULLIF(source_field, ''))`, so authors can count values present without filtering the whole aggregate population.

Validation response:

```text
valid
findings[]: code, severity, field_path, message
```

Publishing is blocked by any `error` finding.

Validation findings are returned for visible, authorized component payloads with invalid contracts. Unauthorized or out-of-scope bindings return `403 forbidden` and do not disclose component validation details.

Minimum stable validation codes:

- `DATASET_MAJOR_LINE_NOT_FOUND`
- `DATASET_MAJOR_LINE_NOT_PUBLISHED`
- `DATASET_MAJOR_LINE_CONTRACT_UNAVAILABLE`
- `DATASET_MAJOR_LINE_CONTRACT_BROKEN`
- `COMPONENT_FIELD_NOT_IN_MAJOR_LINE`
- `COMPONENT_AGGREGATE_FIELD_NOT_IN_MAJOR_LINE`
- `COMPONENT_SORT_FIELD_NOT_IN_MAJOR_LINE`
- `COMPONENT_FILTER_FIELD_NOT_IN_MAJOR_LINE`
- `COMPONENT_UNSUPPORTED_KIND`
- `COMPONENT_DUPLICATE_METRIC_KEY`
- `COMPONENT_UNSUPPORTED_AGGREGATE_FUNCTION`

Table response includes component/version IDs, dataset ID, dataset major, materialization/line metadata, kind, columns, rows, pagination, and a materialization state when rows are not ready.

Query encoding:

```text
q=...
page_size=50
cursor=...
sort=field_key:asc
filter[field_key][operator]=contains
filter[field_key][value]=smith
visible_columns=field_a,field_b,field_c
```

## Table Behavior

- `DetailTable` columns, default sort, filters, and search fields validate against the major-line contract.
- `DetailTable` search, filtering, sorting, and pagination operate over all rows in the bound major line, not only the latest revision or current client page.
- `AggregateTable` group fields, metric source fields, pre-filters, and source-row sorts validate against the major-line contract. Post-filters and aggregate-result sorts validate against aggregate output fields: group fields plus metric keys.
- `AggregateTable` metrics include rows from every published minor/patch revision in the bound major line.
- Multiple metrics have stable keys, labels, ordering, and formatting.
- Authors can configure multiple simultaneous metrics, including repeated functions over different fields such as `sum(field_a)`, `average(field_b)`, `count_values(field_a)`, and `count_values(field_b)`.
- Pre-aggregation filters affect source rows before grouping. Post-aggregation filters affect aggregate output rows after grouping.
- Every displayed viewer column can be sorted and filtered.
- Global search filters across configured display/search columns.
- Pagination is server-derived.
- Column picker can hide/show configured output columns without mutating component config. Runtime `visible_columns` values are validated against known component table output fields; unknown keys return a stable validation error instead of falling through to SQL. Projection does not narrow the search/sort/filter contract: configured search fields and table output fields remain available for server-side search, filters, and sorting even when hidden.
- Loading, empty, error, materializing, and materialization failure states are native UI states.

Required filter/operator coverage:

- Text: `equals`, `not_equals`, `contains`, `not_contains`, `is_empty`, `is_not_empty`, `is_null`, `is_not_null`
- Number/date: `equals`, `not_equals`, `lt`, `lte`, `gt`, `gte`, `between`, `not_between`, `is_null`, `is_not_null`
- Boolean: `equals`, `not_equals`, `is_null`, `is_not_null`

Existing Dataset filters already include `not_equals` and empty/not-empty; Sprint 4A adds the missing component table operators in the shared filter facade and maps empty/not-empty semantics to null operators only where the field contract treats blank and null equivalently.

## Frontend Boundaries

Add `crates/tessara-web-components` and include it in the workspace.

The crate owns component feature content:

- API adapters and feature DTOs
- validation display
- loaders/actions
- directory/detail/create/edit/publish/viewer pages
- authoring controls for detail tables, aggregate tables, metrics, filters, columns, and Dataset major-line picker
- viewer wrappers around the Rust/UI Data Table pattern

Root `tessara-web` owns only route adapters and shell integration for the component routes listed above.

Do not use Sprint 4A to create a new shared frontend platform crate or move shell/session/routing policy into the feature crate.

## Acceptance Criteria

- A tester can create, validate, publish, and view a `DetailTable` in the app.
- A tester can create, validate, publish, and view an `AggregateTable` in the app.
- Component directory, detail, create, edit, publish, and viewer flows are available.
- Component versions bind to Dataset major version lines, not individual Dataset revisions.
- A component bound to Dataset v1 renders data from all published minor/patch revisions in v1.
- Publishing Dataset v1.1.0 adds compatible rows to the v1 major-line surface after materialization refresh.
- Publishing Dataset v2.0.0 does not affect components bound to v1.
- Component validation rejects fields not present in the bound major-line contract.
- Component publication does not fail only because major-line materialization is absent or expired.
- Component render surfaces materialization pending/failure separately from invalid component config.
- `DetailTable` search/filter/sort/pagination operates across the full bound major-line data surface.
- `AggregateTable` grouping and metrics operate across the full bound major-line data surface.
- `AggregateTable` uses the shared `tessara-data-ops` crate for operation contracts and validation.
- Draft/edit flows preserve the current published version until publish.
- Publishing a draft supersedes the prior published version atomically.
- Published and superseded component versions cannot be mutated.
- Public reader routes and dashboards consume only published-history component versions; drafts remain admin-only authoring state.
- Dataset dependency summaries and impact rows ignore working component drafts and report only published-history consumers.
- Unsupported component kinds are rejected from Sprint 4A authoring/publish flows.
- Detail and aggregate viewers use the Rust/UI Data Table pattern with sorting, filtering, negative operators, column picker, search, pagination, and native loading/empty/error/materializing states.
- Changing sort/filter/search/page/visible columns does not mutate component version config.
- Reader routes expose only published, visible components.
- Management routes expose only manageable drafts/versions.
- Scoped users cannot list hidden Dataset major lines in the authoring picker.
- Scoped users cannot bind a hidden Dataset major line by guessed UUID.
- Scoped users cannot publish a component version bound to an out-of-scope Dataset major line.
- Scoped users cannot direct-load hidden component detail or table viewer routes.
- Touched component routes remain native Leptos SSR routes with no new bridge, `inner_html`, or JavaScript-controller-owned route surface.
- Legacy analytical endpoints touched during the sprint remain adapter-only and do not gain new core behavior.

## Manual Test Plan

Admin happy path:

1. Sign in as an admin and open `/components`.
2. Create a `DetailTable` component from a visible Dataset major line.
3. Select columns, save draft, validate, publish, and view.
4. Verify the viewer uses search, filters, sorting, column picker, pagination, and native table states.
5. Create an `AggregateTable` component from the same or another visible Dataset major line.
6. Define group fields, metrics, pre-filters, and post-filters.
7. Validate, publish, and view aggregate output.
8. Edit a published component and confirm the viewer remains unchanged until the draft is published.
9. Publish the draft and confirm the new version becomes the published viewer output.
10. Confirm the public component list/detail continue to show only published versions while a replacement draft exists.

Major-line behavior:

1. Create a component bound to Dataset v1.
2. Verify the table includes rows from all currently published v1 minor/patch revisions.
3. Publish a compatible Dataset v1 minor/patch revision with new rows.
4. Refresh or rebuild the major-line data surface.
5. Verify the component output includes the new rows.
6. Publish Dataset v2.
7. Verify the component remains bound to v1 and does not include v2 rows.

Materialization behavior:

1. Publish a valid component bound to a Dataset major line.
2. Expire or clear the major-line materialization cache in test setup.
3. Open the component viewer.
4. Verify the table endpoint returns `materialization_state = pending` until a ready materialization is available, and the app renders that as a native loading/retry state.
5. Simulate materialization failure.
6. Verify the UI shows a stable materialization failure message, not a component-invalid error.

Scoped negative paths:

1. Sign in as an operator with restricted scope.
2. Confirm hidden Dataset major lines are absent from the authoring picker.
3. Attempt to direct-load a hidden component detail route.
4. Attempt to direct-load a hidden component table route.
5. Attempt to bind a hidden Dataset major line by guessed ID.
6. Attempt to publish a component version bound to an out-of-scope Dataset major line.
7. Confirm all failures use stable application codes and no raw internal/database strings.

## Automated Test Plan

Run:

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo test -p tessara-web-components`
- Playwright component create/edit/publish/viewer scenarios
- Playwright scoped positive and negative permission scenarios
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Backend/API scenarios:

- create component shell requires `components:manage`
- update component metadata requires `components:manage`
- create `DetailTable` draft bound to a major line
- create `AggregateTable` draft bound to a major line
- reject binding to missing major line
- reject binding to out-of-scope major line
- reject fields/sorts/filters/group fields/metric source fields not in major-line contract
- reject duplicate aggregate metric keys
- reject unsupported aggregate functions
- reject unsupported component kinds for Sprint 4A
- publish draft supersedes existing published version
- published/superseded component versions are immutable
- concurrent version creation preserves version-number and one-draft invariants
- concurrent publish preserves single-published invariant
- reader component list/detail omit draft-only components and expose only published versions
- dashboard placement rejects draft component versions and accepts published-history versions
- Dataset dependency summaries and impact rows exclude working drafts

Data operation scenarios:

- aggregate table matches shared aggregation operation result
- pre-filters apply before grouping
- post-filters apply after grouping
- aggregate table sorts by group field and metric field
- component aggregate table accepts `count_values` and counts non-empty source-field values independently of row-count metrics
- detail table filters use the shared filter compiler
- negative filter operators compile correctly
- global search compiles across configured fields

Major-line materialization scenarios:

- render uses the Dataset major-line data surface
- render includes all minor/patch revision rows in the bound major line
- render excludes other major-line rows
- render calls the major-line materialization boundary
- expired materialization does not invalidate the component
- pending materialization returns a stable state
- materialization failure returns a stable error code

Frontend/E2E scenarios:

- `/components`, `/components/new`, `/components/:component_ref`, `/components/:component_ref/edit`, `/components/:component_ref/publish`, and `/components/:component_ref/view` load natively
- create `DetailTable` -> save draft -> publish -> view
- create `AggregateTable` -> save draft -> publish -> view
- edit published component -> draft visible -> viewer unchanged
- publish edited draft -> viewer changes
- validation findings render next to relevant fields
- table wrappers render headers, rows, toolbar, column picker, filters, and pagination
- sort and filter every displayed `DetailTable` and `AggregateTable` column
- cover `not_equals`, `not_contains`, and `is_not_null` in UI/API tests
- use global search and verify results are server-derived
- hide/show columns with the column picker
- reject unknown `visible_columns` values with a stable validation error
- verify search/sort/filter still operate over configured server-side fields when visible columns are narrowed
- add targeted overlap/containment authorization fixtures: read positive with asset scoped to A+B+C and user scoped to A, read negative with user scoped to D, authoring containment negative with user scoped only to A, and authoring containment positive with user scoped to A+B+C
- paginate through server-driven results
- scoped manager cannot bind hidden Dataset major line by guessed ID
- scoped reader cannot direct-load hidden component detail or table viewer

Update `docs/playwright-permissions-scenarios.md` for component create/edit/publish/viewer coverage.

## Ordered Implementation Plan

1. Inventory current component, dashboard, dependency-impact, demo seed, and legacy analytical endpoint contracts that still join through revision-bound component contracts.
2. Add major-line component API/data contracts and migrate component persistence from revision binding to `dataset_id + dataset_version_major + binding_mode`, removing component-version `dataset_revision_id` from schema/API/frontend contracts.
3. Implement component shell update, one-draft version lifecycle, explicit publish endpoint, immutability checks, stable validation findings, and scoped list/detail/access checks.
4. Add the component table execution API over Dataset major-line materialization with server-driven search/filter/sort/page/query parsing and materialization ready/pending/failed states.
5. Extract pure Dataset operation contracts and validation into `crates/tessara-data-ops`; keep SQL compilation and database-aware execution in `tessara-api`.
6. Add focused backend/API/materialization tests before wiring broad UI flows.
7. Create `crates/tessara-web-components`, add it to the workspace, and move component-local contracts, API calls, loaders/actions, authoring controls, and viewer wrappers there.
8. Add root `tessara-web` route adapters and shell integration for all component routes.
9. Implement `DetailTable` and `AggregateTable` authoring, validation display, publish flow, and Rust/UI Data Table viewers.
10. Extend Playwright, smoke, UAT, route ownership, hydration, and browser-console coverage.
11. Run the planned verification set and update sprint notes/progress with results.

## Risks And Scope Traps

- Revision-binding terminology will push implementation toward the wrong model. Use Dataset major-line binding consistently.
- Component tables must not render only the latest Dataset revision; they execute over the full major-line materialized surface.
- A component-only aggregation engine will duplicate future chart/stat logic. Put shared operation contracts and validation in `tessara-data-ops` so future components can reuse the same foundation.
- Do not treat `count_values` as a distinct-value frequency table. In the current shared operation model it means "count non-empty values for a source field"; grouped frequency tables are already expressible with group field plus `count`.
- Publish-time materialization coupling would make components invalid when cache policy changes. Validate contracts at publish; ensure materialization at render.
- Client-only filtering/sorting/pagination can accidentally operate only on the current page. Use server-driven table execution.
- Major-line visibility must be checked in directory, picker, bind, publish, detail, and viewer routes, including direct URL/API access.
- Future chart/stat components are out of Sprint 4A authoring and viewer scope even if they influence shared operation boundaries.
- Avoid broad platform rewrites beyond the dedicated data-ops crate: no shared frontend platform crate and no route-shell migration beyond touched component/reporting surfaces.

## Dependencies And Blockers

- Seeded or UAT-created data must support Dataset major-line authoring and materialization flows.
- Full verification depends on local app launch, Playwright availability, and component routes hydrating cleanly.
- This plan is the durable Sprint 4A source of truth for implementation and closeout.

# Sprint 4A Plan: Dataset Catalog And Thin Table Components

Kickoff status: started from clean `main` on 2026-07-03.

## Sprint Summary

Sprint 4A pivots from component-owned analytical shaping to a Dataset-first presentation model.
Datasets are the source of truth for reusable analytical and display-ready table shape: joins, aggregations, calculations, labels, and stable field contracts belong in Dataset authoring. Components are last-mile presentation and publication assets that bind to Dataset major lines and render a single Table component with one projection, one default filter set, and simple viewer defaults.

The sprint delivers:

- searchable Dataset catalog improvements, including Dataset tags and provenance;
- a thin Table component bound to a Dataset major line;
- component draft/version/publish workflows;
- published-history dashboard placement;
- scoped read/authoring authorization for Dataset-backed components;
- native Leptos table authoring, versioning, publishing, and viewing routes;
- shared interactive table rendering for Dataset previews and Component viewers.

The earlier Detail Table / Aggregate Table split is intentionally removed before merge. Aggregate and detail shaping should be modeled as display-ready Datasets, then rendered through the same Table component.

Kickoff defaults:

- Branch: `codex/sprint-4a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4a`
- Plan artifact: `docs/sprints/sprint-4a-plan.md`

## Product Decisions

- Datasets own data shaping. Components own presentation, publication, and placement.
- Sprint 4A exposes one public component kind: `table`.
- Component configs own exactly one last-mile table projection and one optional default filter set. They may store `visible_columns`, `filters`, `default_sort`, `page_size`, optional `search_fields`, and optional display-label overrides.
- Component configs must not own aggregate metrics, group fields, pre/post aggregate filters, Dataset-like calculations, joins, or reusable analytical field-contract shaping.
- A table that needs grouped or aggregated output should bind to a Dataset whose final shape is already grouped or aggregated.
- Dataset catalog growth is handled with search, tags, and provenance rather than formal analytical/display Dataset classes.
- Dataset tags are searchable metadata, not authorization or execution semantics.
- Dataset provenance should help authors answer "what produced this Dataset?" by showing a full ancestor lineage rooted at the Dataset and including contributing Forms and upstream Datasets.
- Component versions bind major-line only: store `dataset_id`, `dataset_version_major`, and `binding_mode = major_line`; enforce `binding_mode = 'major_line'`.
- Component versions do not retain `dataset_revision_id`.
- Component publishing is initiated from the edit screen. Authors manually choose whether an edit updates the current published version in place or creates a new version. New-version publishing opens a consumer-review modal with a searchable consumer list placeholder and a required version-note affordance.
- The system does not classify component edits as breaking or non-breaking. The author owns the decision to update the current version or create a new version.
- Public reader routes list/load only published component versions. Management routes expose drafts plus published/superseded history to users with `components:manage`.
- Component list status distinguishes `Draft`, `Published`, and `Updating`; the list also shows the current revision label.
- Dashboard placements may reference only immutable published-history component versions: `published` or `superseded`.
- Read visibility is audience-based overlap. Authoring, binding, publishing, and dashboard placement remain governance-based containment.

## Dataset Catalog

Dataset list and picker surfaces should scale beyond a short demo list.

Dataset catalog metadata:

- `tags`: zero or more user-authored searchable labels.
- `provenance.forms`: source Forms that contribute to the Dataset through the current published/draft source graph.
- `provenance.datasets`: upstream Datasets that contribute to the Dataset through the current published/draft source graph.
- `current_version_major` and `major_versions`.
- `output_fields` for field preview and component picker confidence.
- existing visibility nodes and revision status metadata.

Dataset search should match:

- Dataset name;
- Dataset slug;
- Dataset grain;
- tags;
- output field keys and labels;
- provenance Form names;
- provenance upstream Dataset names.

Dataset tags:

- Stored on `datasets` as metadata.
- Editable by Dataset managers alongside Dataset shell metadata.
- Returned by Dataset list/detail APIs.
- Rendered in Dataset directory rows/cards and Dataset detail.
- Used by Component authoring Dataset picker search.

Dataset provenance:

- Derived from `dataset_sources`, source Forms, and upstream Dataset references.
- Returned by Dataset list/detail APIs as compact summaries and lineage data.
- Rendered in Dataset detail as a tree rooted at the current Dataset, with Form/Dataset icons and expand/collapse behavior.
- Rendered in Dataset and Component picker context as compact source summaries.
- Does not change Dataset version compatibility rules in Sprint 4A.

## Component Contracts

Management routes:

- `GET /api/admin/components`
- `POST /api/admin/components`
- `POST /api/admin/components/save`
- `PATCH /api/admin/components/{component_id}`
- `GET /api/admin/components/{component_ref}`
- `POST /api/admin/components/{component_id}/versions`
- `PATCH /api/admin/components/{component_id}/versions/{version_id}`
- `POST /api/admin/components/{component_id}/versions/{version_id}/publish`
- `POST /api/admin/components/validate`

`POST /api/admin/components/save` is the application authoring workflow for component edit-screen saves, published-version updates, and new-version creation. The granular component admin routes remain internal API/setup primitives for tests, migrations, validation, and narrowly scoped lifecycle operations such as deleting a draft; they are not the documented user-facing author workflow.

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
- `/components/:component_ref/versions`
- `/components/:component_ref/view`

The previous `/components/:component_ref/publish` interstitial route is intentionally removed. Publishing and version decisions belong on the edit screen.

Component binding:

```text
ComponentDatasetBinding {
  dataset_id,
  dataset_version_major,
  binding_mode = "major_line"
}
```

Table component config:

```text
{
  "visible_columns": ["field_a", "field_b"],
  "filters": [
    { "field_key": "field_a", "operator": "contains", "value": "active" }
  ],
  "default_sort": { "field_key": "field_a", "direction": "asc" },
  "page_size": 50,
  "search_fields": ["field_a", "field_b"],
  "display_labels": {
    "field_a": "Display label"
  }
}
```

Validation response:

```text
valid
findings[]: code, severity, field_path, message
```

Minimum stable validation codes:

- `DATASET_MAJOR_LINE_NOT_FOUND`
- `DATASET_MAJOR_LINE_NOT_PUBLISHED`
- `DATASET_MAJOR_LINE_CONTRACT_UNAVAILABLE`
- `COMPONENT_FIELD_NOT_IN_MAJOR_LINE`
- `COMPONENT_SORT_FIELD_NOT_IN_MAJOR_LINE`
- `COMPONENT_UNSUPPORTED_KIND`

Validation findings are returned for visible, authorized payloads with invalid contracts. Unauthorized or out-of-scope bindings return `403 forbidden` and do not disclose validation details.

## Table Behavior

- A Table component renders the Dataset major-line materialized surface directly.
- Component table execution never performs extra grouping, metric calculation, joins, analytical projection/re-keying, or row-shaping beyond Dataset output.
- `visible_columns` defines the component projection contract and rendered column order.
- If `visible_columns` is blank, the component projection contract is the Dataset major-line output contract.
- `filters` defines the component's saved default filter set over the bound Dataset major-line output fields. These filters run before viewer filters.
- `search_fields` controls global search fields. If blank, search runs across every field in the component projection contract using text coercion.
- Default sort, display-label, runtime visible-column, and viewer filter validation use the component projection contract so viewer state cannot reveal fields hidden by the component.
- Default sort and page size come from component config unless overridden by query params.
- Runtime query state does not mutate component version config.
- Materialization pending/failure is a render state, not a component validation failure.
- Current component table route uses only the current published version.
- Explicit version table route allows published or superseded versions and authorizes against the selected version's Dataset scope.
- Dataset previews and Component rendering use the shared interactive table display: search, column selection, header sort/filter menus, reset controls, pagination, and horizontal overflow behavior.
- Header filter controls live in anchored menus and do not replace table horizontal scrolling with page scrolling.
- Column visibility, sort, filter, and search are viewer state. They do not mutate Dataset definitions or Component version config.

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

Supported viewer filter operators:

- Text: `equals`, `not_equals`, `contains`, `not_contains`, `is_empty`, `is_not_empty`, `is_null`, `is_not_null`
- Number/date: `equals`, `not_equals`, `lt`, `lte`, `gt`, `gte`, `between`, `not_between`, `is_null`, `is_not_null`
- Boolean: `equals`, `not_equals`, `is_null`, `is_not_null`

## Authorization

- `components:manage` is required for create/edit/publish.
- `components:read` is required for public component list/detail/table routes.
- Dataset read visibility is required to list/select a Dataset major line.
- Component-management scope must fully contain the Dataset visibility nodes for create/bind/publish.
- Component management routes require containment over existing component version history before metadata, draft, or publish mutations.
- Scoped read visibility uses overlap: a user with the required read capability may list or load an asset when at least one asset visibility node overlaps the user's scoped nodes.
- Scoped authoring authority uses containment: a user with manage capability may create, bind, edit, publish, or place an asset only when their manage scope fully contains every implicated visibility node.
- Dashboard placement is governed by dashboard manage scope plus dashboard visibility encompassing the component Dataset visibility. Sprint 4A does not add a separate `components:manage` requirement for dashboard composition.

## Frontend Boundaries

`crates/tessara-web-components` owns:

- component API adapters and DTOs;
- directory/detail/create/edit/versions/viewer pages;
- Dataset picker with catalog search, tags, provenance, field preview, and major-line choice;
- thin Table config controls for visible fields, saved filters, labels, sort, page size, and viewer defaults;
- edit-screen publish/version workflow and consumer-review placeholder modal;
- validation display;
- viewer wrappers around the shared interactive table display.

`crates/tessara-web-datasets` owns:

- Dataset directory search/filter UX;
- Dataset tag editing;
- Dataset provenance lineage display;
- Dataset detail/catalog presentation.

Root `tessara-web` owns route adapters, app shell integration, session guard integration, document metadata, hydration, CSS/assets, and cargo-leptos ownership.

Do not introduce feature-crate cross dependencies. Component authoring may consume Dataset catalog data through API DTOs, not through `tessara-web-datasets`.

## Acceptance Criteria

- Dataset APIs expose tags and provenance summaries.
- Dataset directory search matches name, slug, grain, tags, fields, and provenance names.
- Dataset detail shows tags and contributing Forms/upstream Datasets.
- Dataset managers can edit Dataset tags.
- Component authoring picker shows Dataset tags, provenance, major versions, and field preview.
- A tester can create, validate, publish, and view a Table component.
- Component versions bind to Dataset major version lines, not revisions.
- Component versions are constrained to `component_type = table`.
- Component config stores only presentation-level table options.
- Component validation rejects fields/sorts/search fields not present in the bound major-line contract.
- Component table execution renders the Dataset major-line table surface directly.
- A component bound to Dataset v1 renders data from all published minor/patch revisions in v1.
- Publishing Dataset v2 does not affect components bound to v1.
- Draft/edit flows preserve the current published version until publish.
- Authors can update an existing published version in place or create a new version.
- Creating a new version records a version note and prepares a consumer-review/re-pinning workflow.
- Superseded component versions cannot be mutated. The current published version may be
  updated in place only through the explicit edit-screen "Update Existing Version" action;
  authors choose this path when pinned consumers should receive the update without being
  repinned to a new version.
- Public reader routes and dashboards consume only published-history component versions; drafts remain admin-only authoring state.
- Dataset dependency summaries and impact rows ignore working component drafts and report only published-history consumers.
- Reader routes expose only published, visible components.
- Management routes expose only manageable drafts/versions.
- Scoped users cannot list hidden Dataset major lines in the authoring picker.
- Scoped users cannot bind a hidden Dataset major line by guessed UUID.
- Scoped users cannot publish a component version bound to an out-of-scope Dataset major line.
- Scoped users cannot direct-load hidden component detail or table viewer routes.
- Touched component routes remain native Leptos SSR routes.
- The old component publish page is absent from route adapters, scripts, and docs.

## Manual Test Plan

Admin happy path:

1. Sign in as admin and open `/datasets`.
2. Confirm Dataset search matches tags and provenance names.
3. Open a Dataset detail page and review tags, source Forms, upstream Datasets, major versions, and output fields.
4. Edit Dataset tags and verify directory/search/detail update.
5. Open `/components/new`.
6. Search for a Dataset by tag or provenance.
7. Select a Dataset major line, choose displayed fields, optional default filters, default sort, and page size.
8. Save the Table component draft.
9. Publish from the edit screen by choosing either update existing version or create new version.
10. For create-new-version, review the consumer modal and add a version note.
11. View the Table component and confirm it uses the shared interactive table display.
12. Edit a published component and confirm the viewer remains unchanged until the draft is published or the existing published version is updated.

Major-line behavior:

1. Create a component bound to Dataset v1.
2. Verify the table includes rows from currently published v1 minor/patch revisions.
3. Publish a compatible Dataset v1 minor/patch revision with new rows.
4. Refresh or rebuild the major-line data surface.
5. Verify the component output includes the new rows.
6. Publish Dataset v2.
7. Verify the component remains bound to v1 and does not include v2 rows.

Scoped negative paths:

1. Sign in as a scoped operator.
2. Confirm hidden Dataset major lines are absent from the Component authoring picker.
3. Attempt to direct-load a hidden component detail route.
4. Attempt to direct-load a hidden component table route.
5. Attempt to bind a hidden Dataset major line by guessed ID.
6. Attempt to publish a component version bound to an out-of-scope Dataset major line.

## Automated Test Plan

Run:

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo test -p tessara-web-components`
- `cargo test -p tessara-web-datasets`
- Playwright Dataset catalog/tag/provenance scenarios
- Playwright component create/edit/publish/viewer scenarios
- Playwright scoped positive and negative permission scenarios
- `.\scripts\validate.ps1`
- `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Backend/API scenarios:

- Dataset list/detail include tags and provenance.
- Dataset tag update requires `datasets:manage` and full Dataset scope containment.
- Dataset search data includes tags, fields, Forms, and upstream Datasets.
- Create Table component draft bound to a major line.
- Reject binding to missing major line.
- Reject binding to out-of-scope major line.
- Reject visible/search/sort fields not in the major-line contract.
- Reject unsupported component kinds.
- Update existing published version mutates that version in place.
- Create new version supersedes the prior published version and records a version note.
- Superseded component versions are immutable, while the current published version supports
  the explicit edit-screen update-in-place workflow.
- Concurrent version creation preserves version-number and one-draft invariants.
- Concurrent publish preserves single-published invariant.
- Reader component list/detail omit draft-only components and expose only published versions.
- Dashboard placement rejects draft component versions and accepts published-history versions.
- Dataset dependency summaries and impact rows exclude working drafts.

Frontend/E2E scenarios:

- Dataset directory search by tag and provenance.
- Dataset detail renders tags and provenance.
- Dataset tag edit/save flow.
- Component Dataset picker search by tag/provenance.
- `/components`, `/components/new`, `/components/:component_ref`, `/components/:component_ref/edit`, `/components/:component_ref/versions`, and `/components/:component_ref/view` load natively.
- Create Table -> save draft or publish from the edit screen -> view.
- Edit published Table -> draft visible -> viewer unchanged.
- From the edit screen, either update the existing published version in place or create a new version after reviewing current consumers and writing a version note.
- Validation findings render next to relevant fields.
- Table viewer renders headers, rows, toolbar, column picker, filters, and pagination.
- Sort/filter/search over Dataset output fields.
- Hide/show columns with the column picker.
- Reject unknown `visible_columns` values with a stable validation error.
- Search/sort/filter still operate over known server-side fields when visible columns are narrowed.
- Overlap/containment authorization fixtures: read positive with asset scoped to A+B+C and user scoped to A, read negative with user scoped to D, authoring containment negative with user scoped only to A, and authoring containment positive with user scoped to A+B+C.

Update `docs/playwright-permissions-scenarios.md` for component create/edit/publish/viewer coverage.

## Ordered Implementation Plan

1. Rewrite Sprint 4A plan around Dataset catalog/provenance/tags and thin Table components.
2. Add Dataset tag/provenance API contracts and schema/storage.
3. Simplify component backend validation/execution to one `table` component type with one last-mile projection and one saved default filter set.
4. Simplify component frontend authoring to a single Table form with Dataset picker/catalog context.
5. Add Dataset catalog/search/tag/provenance UI.
6. Rewrite component and permission tests to remove DetailTable/AggregateTable assumptions.
7. Replace Dataset previews and Component rendering with the shared interactive table display.
8. Remove superseded publish-page/aggregate-detail compatibility paths and enforce table-only component versions.
9. Run full validation and update sprint closeout notes.

## Risks And Scope Traps

- Recreating Dataset authoring inside Components is the main trap. If a table needs shaping, create or edit a Dataset.
- Tags are discoverability metadata only. Do not use tags for permissions, materialization, or version compatibility.
- Provenance must be helpful without becoming a workflow editor. Sprint 4A shows a lineage tree for ancestry and compact summaries in picker contexts.
- Component tables must not render only the latest Dataset revision; they execute over the full major-line materialized surface.
- Publish-time materialization coupling would make components invalid when cache policy changes. Validate contracts at publish; ensure materialization at render.
- Client-only filtering/sorting/pagination can accidentally operate only on the current page. Use server-driven table execution.
- Major-line visibility must be checked in directory, picker, bind, publish, detail, and viewer routes, including direct URL/API access.
- Avoid broad platform rewrites: no shared frontend platform crate and no route-shell migration beyond touched catalog/component surfaces.

## Dependencies And Blockers

- Seeded or UAT-created data must include Datasets with tags and source provenance.
- Full verification depends on local app launch, Playwright availability, and native routes hydrating cleanly.
- This plan is the durable Sprint 4A source of truth for implementation and closeout.

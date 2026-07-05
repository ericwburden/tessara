# Sprint 4A Plan: Dataset Catalog And Thin Table Components

Kickoff status: started from clean `main` on 2026-07-03.

## Sprint Summary

Sprint 4A pivots from component-owned table shaping to a Dataset-first presentation model.
Datasets are the source of truth for analytical and display-ready table shape: joins, projections, filters, aggregations, calculations, labels, and field contracts belong in Dataset authoring. Components are last-mile presentation and publication assets that bind to Dataset major lines and render a single Table component with a small presentation config.

The sprint delivers:

- searchable Dataset catalog improvements, including Dataset tags and provenance;
- a thin Table component bound to a Dataset major line;
- component draft/version/publish workflows;
- published-history dashboard placement;
- scoped read/authoring authorization for Dataset-backed components;
- native Leptos table authoring, publishing, and viewing routes.

The earlier Detail Table / Aggregate Table split is intentionally removed before merge. Aggregate and detail shaping should be modeled as display-ready Datasets, then rendered through the same Table component.

Kickoff defaults:

- Branch: `codex/sprint-4a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4a`
- Plan artifact: `docs/sprints/sprint-4a-plan.md`

## Product Decisions

- Datasets own data shaping. Components own presentation, publication, and placement.
- Sprint 4A exposes one public component kind: `table`.
- Component configs are presentation-only. They may store `visible_columns`, `default_sort`, `page_size`, optional `search_fields`, and optional display-label overrides.
- Component configs must not own aggregate metrics, group fields, pre/post aggregate filters, Dataset-like calculations, joins, or projections.
- A table that needs grouped or aggregated output should bind to a Dataset whose final shape is already grouped or aggregated.
- Dataset catalog growth is handled with search, tags, and provenance rather than formal analytical/display Dataset classes.
- Dataset tags are searchable metadata, not authorization or execution semantics.
- Dataset provenance should help authors answer "what produced this Dataset?" by showing contributing Forms and upstream Datasets.
- Component versions bind major-line only: store `dataset_id`, `dataset_version_major`, and `binding_mode = major_line`; enforce `binding_mode = 'major_line'`.
- Component versions do not retain `dataset_revision_id`.
- The explicit publish endpoint is the only publish flow: `POST /api/admin/components/{component_id}/versions/{version_id}/publish`.
- Public reader routes list/load only published component versions. Management routes expose drafts plus published/superseded history to users with `components:manage`.
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
- Returned by Dataset list/detail APIs as compact summaries.
- Rendered in Dataset detail and picker context.
- Does not change Dataset version compatibility rules in Sprint 4A.

## Component Contracts

Management routes:

- `GET /api/admin/components`
- `POST /api/admin/components`
- `PATCH /api/admin/components/{component_id}`
- `GET /api/admin/components/{component_ref}`
- `POST /api/admin/components/{component_id}/versions`
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
- Component table execution never performs extra grouping, metric calculation, joins, projections, or row-shaping beyond Dataset output.
- `visible_columns` controls rendered output columns and order.
- If `visible_columns` is blank, render the Dataset major-line output contract.
- `search_fields` controls global search fields. If blank, search defaults to visible/output text-like fields.
- Sort and filter validation use the Dataset major-line output contract.
- Default sort and page size come from component config unless overridden by query params.
- Runtime query state does not mutate component version config.
- Materialization pending/failure is a render state, not a component validation failure.
- Current component table route uses only the current published version.
- Explicit version table route allows published or superseded versions and authorizes against the selected version's Dataset scope.

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
- directory/detail/create/edit/publish/viewer pages;
- Dataset picker with catalog search, tags, provenance, field preview, and major-line choice;
- presentation-only Table config controls;
- validation display;
- viewer wrappers around the Rust/UI Data Table pattern.

`crates/tessara-web-datasets` owns:

- Dataset directory search/filter UX;
- Dataset tag editing;
- Dataset provenance display;
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
- Component config stores only presentation-level table options.
- Component validation rejects fields/sorts/search fields not present in the bound major-line contract.
- Component table execution renders the Dataset major-line table surface directly.
- A component bound to Dataset v1 renders data from all published minor/patch revisions in v1.
- Publishing Dataset v2 does not affect components bound to v1.
- Draft/edit flows preserve the current published version until publish.
- Publishing a draft supersedes the prior published version atomically.
- Published and superseded component versions cannot be mutated.
- Public reader routes and dashboards consume only published-history component versions; drafts remain admin-only authoring state.
- Dataset dependency summaries and impact rows ignore working component drafts and report only published-history consumers.
- Reader routes expose only published, visible components.
- Management routes expose only manageable drafts/versions.
- Scoped users cannot list hidden Dataset major lines in the authoring picker.
- Scoped users cannot bind a hidden Dataset major line by guessed UUID.
- Scoped users cannot publish a component version bound to an out-of-scope Dataset major line.
- Scoped users cannot direct-load hidden component detail or table viewer routes.
- Touched component routes remain native Leptos SSR routes.

## Manual Test Plan

Admin happy path:

1. Sign in as admin and open `/datasets`.
2. Confirm Dataset search matches tags and provenance names.
3. Open a Dataset detail page and review tags, source Forms, upstream Datasets, major versions, and output fields.
4. Edit Dataset tags and verify directory/search/detail update.
5. Open `/components/new`.
6. Search for a Dataset by tag or provenance.
7. Select a Dataset major line, choose visible columns, optional search fields, default sort, and page size.
8. Save the Table component draft.
9. Validate, publish, and view the Table component.
10. Edit a published component and confirm the viewer remains unchanged until the draft is published.
11. Publish the draft and confirm the viewer reflects the new presentation config.

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
- Publish draft supersedes existing published version.
- Published/superseded component versions are immutable.
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
- `/components`, `/components/new`, `/components/:component_ref`, `/components/:component_ref/edit`, `/components/:component_ref/publish`, and `/components/:component_ref/view` load natively.
- Create Table -> save draft -> publish -> view.
- Edit published Table -> draft visible -> viewer unchanged.
- Publish edited draft -> viewer changes.
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
3. Simplify component backend validation/execution to one `table` component type and presentation-only config.
4. Simplify component frontend authoring to a single Table form with Dataset picker/catalog context.
5. Add Dataset catalog/search/tag/provenance UI.
6. Rewrite component and permission tests to remove DetailTable/AggregateTable assumptions.
7. Run full validation and update sprint closeout notes.

## Risks And Scope Traps

- Recreating Dataset authoring inside Components is the main trap. If a table needs shaping, create or edit a Dataset.
- Tags are discoverability metadata only. Do not use tags for permissions, materialization, or version compatibility.
- Provenance must be helpful without becoming a graph explorer. Sprint 4A shows compact direct source summaries.
- Component tables must not render only the latest Dataset revision; they execute over the full major-line materialized surface.
- Publish-time materialization coupling would make components invalid when cache policy changes. Validate contracts at publish; ensure materialization at render.
- Client-only filtering/sorting/pagination can accidentally operate only on the current page. Use server-driven table execution.
- Major-line visibility must be checked in directory, picker, bind, publish, detail, and viewer routes, including direct URL/API access.
- Avoid broad platform rewrites: no shared frontend platform crate and no route-shell migration beyond touched catalog/component surfaces.

## Dependencies And Blockers

- Seeded or UAT-created data must include Datasets with tags and source provenance.
- Full verification depends on local app launch, Playwright availability, and native routes hydrating cleanly.
- This plan is the durable Sprint 4A source of truth for implementation and closeout.

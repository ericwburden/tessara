# Progress Report

## 2026-04-14 - Form Versioning Pivot To Major-Version Compatibility

- Pivoted form versioning from compatibility-group-centric behavior to semantic versioning with major-version compatibility.
- Backend changes now:
  - assign `version_major`, `version_minor`, and `version_patch` on publish
  - derive `PATCH` / `MINOR` / `MAJOR` from the form contract delta at publish time
  - keep compatible publishes on the current major line
  - start a new major line for breaking publishes
  - freeze new direct dataset and direct report consumers to the current published form major automatically
- Application and builder surfaces now:
  - create unlabeled draft versions and defer final version assignment until publish
  - show publish previews and major-line compatibility messaging instead of manual compatibility-group entry
  - remove remaining compatibility-group controls from the active forms routes and the legacy admin builder dataset/form authoring surfaces
- Migration updates:
  - `015_form_version_semver.sql` introduces semantic-version fields and major-version binding fields for dataset/report consumers
  - `016_form_version_legacy_label_backfill.sql` backfills older non-semver labels such as `v1` and `legacy-v1`
- Validation:
  - `cargo fmt --all`: completed successfully on 2026-04-14
  - `cargo test -p tessara-api`: completed successfully on 2026-04-14
  - `cargo test -p tessara-web`: completed successfully on 2026-04-14
  - `scripts\local-launch.ps1`: completed successfully on 2026-04-14 after splitting the legacy-label backfill into migration `016`
  - `scripts\smoke.ps1`: completed successfully on 2026-04-14 against the revised major-version model

## 2026-04-14 - Sprint 1D Forms, Fields, And Version Authoring Closeout

- Completed Sprint 1D closeout work in `D:\Projects\tessara`.
- Delivered application-owned forms route coverage for:
  - `/app/forms`
  - `/app/forms/new`
  - `/app/forms/{form_id}`
  - `/app/forms/{form_id}/edit`
- Delivered form-authoring workflow updates in the application shell:
  - top-level form create/edit flows continue through native app routes instead of builder-only fallback behavior
  - form detail now surfaces version summary, workflow attachments, and section/field preview panels
  - form edit now supports draft version creation, section add/update/delete/reorder, field add/update/delete/reorder, and draft publish actions
  - publish validation and stale/double-submit protection are surfaced in route-local status messages
  - option-set and lookup touchpoints remain visible as non-blocking read-only affordances where backend metadata is not yet available
- Expanded closeout evidence coverage for the forms slice:
  - `scripts\uat-sprint.ps1` now checks forms list, new, detail, and edit routes
  - `scripts\smoke.ps1` now checks forms lifecycle and authoring route markers
  - `end2end\tests\app.spec.ts` now includes forms route render and JS-disabled readability checks
- Roadmap update:
  - Marked Sprint 1D as complete in `D:\Projects\tessara\docs\roadmap.md`.
  - Marked Sprint 2A as the next sprint focus.
- Validation:
  - `scripts\local-launch.ps1`: completed successfully on 2026-04-14.
  - `scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`: completed successfully on 2026-04-14.
  - `scripts\smoke.ps1`: completed successfully on 2026-04-14 after updating stale `demo_flow` assertion expectations to match current API behavior.
  - `cargo fmt --all`: completed successfully on 2026-04-14.
  - `cargo test -p tessara-api`: completed successfully on 2026-04-14.
  - `cargo test -p tessara-web`: completed successfully on 2026-04-14.
- Next Sprint: Sprint 2A Workflow Assignment And Response Start

## Sprint Handoff / Demo Instructions

### Forms Directory And Lifecycle Visibility
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/api/forms`
- Steps:
  1. Sign in as `admin@tessara.local`.
  2. Open `/app/forms`.
  3. Verify the forms directory renders cards with scope, published version, and draft-count summaries.
  4. Open one form detail route from the list.
- Expected:
  - the forms list renders without builder-only fallback navigation
  - lifecycle information is visible before entering detail
- Acceptance check:
  - Admin can browse the forms directory and identify published and draft state from the route itself.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `scripts\smoke.ps1`
  - `end2end\tests\app.spec.ts`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Form Creation And Native Route Ownership
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/new`
  - `http://localhost:8080/api/admin/forms`
- Steps:
  1. Open `/app/forms/new`.
  2. Enter a form name, slug, and optional scope node type.
  3. Submit the form.
  4. Confirm the browser redirects into `/app/forms/{form_id}/edit`.
- Expected:
  - the form can be created from the application route
  - the route continues directly into version authoring
- Acceptance check:
  - Admin can create a form without using the legacy builder and land in the authoring route.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `crates/tessara-web/public/bridge/app-legacy.js`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Version Authoring, Sections, Fields, And Publish
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/{form_id}/edit`
  - `http://localhost:8080/api/admin/forms/{form_id}/versions`
  - `http://localhost:8080/api/admin/form-versions/{form_version_id}/sections`
  - `http://localhost:8080/api/admin/form-versions/{form_version_id}/fields`
  - `http://localhost:8080/api/admin/form-versions/{form_version_id}/publish`
- Steps:
  1. Open `/app/forms/{form_id}/edit`.
  2. Create a draft version.
  3. Add a section.
  4. Add one or more fields to that section.
  5. Reorder a section and a field.
  6. Publish the draft version.
- Expected:
  - draft lifecycle controls are visible in the route
  - section and field authoring actions are available without leaving the app route
  - invalid publish attempts show explicit route-local validation messages
- Acceptance check:
  - Admin can complete create draft version -> add section -> add field -> reorder -> publish entirely through `/app/forms/{id}/edit`.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `scripts\smoke.ps1`
  - `crates/tessara-web/public/bridge/app-legacy.js`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Form Detail Review And Workflow Attachments
- Role: admin
- Paths:
  - `http://localhost:8080/app/forms/{form_id}`
  - `http://localhost:8080/api/forms/{form_id}`
  - `http://localhost:8080/api/form-versions/{form_version_id}/render`
- Steps:
  1. Open `/app/forms/{form_id}`.
  2. Verify the summary section shows scope and published/draft state.
  3. Verify the version summary panel renders semantic version, major-line compatibility, and publish metadata.
  4. Verify section and field preview panels render.
  5. Verify related reports and dataset-source workflow attachments are visible.
- Expected:
  - detail route clearly separates summary, version lifecycle, section preview, and workflow attachments
  - route remains readable without JavaScript for core headings and structure
- Acceptance check:
  - A tester can inspect version status and downstream workflow links entirely from the form detail route.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `end2end\tests\app.spec.ts`
  - 2026-04-14 terminal transcript from the sprint closeout run

### Access Control And Non-Admin Readability
- Role: operator
- Paths:
  - `http://localhost:8080/app/forms`
  - `http://localhost:8080/app/forms/{form_id}`
  - `http://localhost:8080/app/forms/{form_id}/edit`
  - `http://localhost:8080/api/forms`
  - `http://localhost:8080/api/admin/forms/{form_id}`
- Steps:
  1. Sign in as `operator@tessara.local`.
  2. Open `/app/forms` and a visible form detail route.
  3. Attempt to open the edit route or call an admin forms endpoint.
  4. Confirm readable forms surfaces remain available while write/admin actions stay gated.
- Expected:
  - readable routes remain usable where the role has access
  - admin-only write flow remains restricted
- Acceptance check:
  - At least one allowed read path and one denied write/admin path are both demonstrated.
- Evidence location:
  - `scripts\uat-sprint.ps1`
  - `crates/tessara-web/public/bridge/app-legacy.js`
  - 2026-04-14 terminal transcript from the sprint closeout run

## Acceptance Mapping

- Exit condition:
  - a tester can create a form, add/edit/remove/reorder fields, publish a version, and inspect status entirely through UI
- Manual demonstration:
  - `Form Creation And Native Route Ownership`
  - `Version Authoring, Sections, Fields, And Publish`
  - `Form Detail Review And Workflow Attachments`
- Automated check:
  - `scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
  - `scripts\smoke.ps1`
  - `end2end\tests\app.spec.ts`

## 2026-04-13 - Sprint 1C Organization Management Closeout

- Completed organization-management closure work in `D:\Projects\dms-migration\tessara`.
- Added organization scope-aware hierarchy browsing and editing in native application routes:
  - `/app/organization` now uses full-tree directory navigation and destination labelling based on scoped node types.
  - `/app/organization/{node_id}` now uses tree-aware detail framing with path, metadata, and add-child actions derived from configured node-type relationships.
  - `/app/organization/new` and `/app/organization/{node_id}/edit` now initialize from node-type metadata and hierarchy rules.
- Added complete organization node-type admin flow in application shell:
  - `/app/administration/node-types` list
  - `/app/administration/node-types/new`
  - `/app/administration/node-types/{node_type_id}`
  - `/app/administration/node-types/{node_type_id}/edit`
- Backend updates in `tessara-api` for the same slice:
  - schema migration `014_node_type_labels.sql` adds singular/plural node-type labeling fields
  - node-type catalog now exposes readable labels and parent/child relationship graph (`/api/node-types`)
  - node-type CRUD enforces relationship consistency, cycle avoidance, access control (`admin:all`), and non-root parent requirements for non-root types
  - node metadata field deletion support (`DELETE /api/admin/node-metadata-fields/{field_id}`)
- Validation completed successfully:
  - `cargo fmt --all`
  - `cargo test -p tessara-api --test demo_flow readable_node_type_catalog_exposes_labels_and_relationships`
  - `cargo test -p tessara-api --test demo_flow node_metadata_fields_can_be_deleted`
  - `cargo test -p tessara-api --test demo_flow non_root_node_types_require_a_parent_node`
  - `cargo test -p tessara-api --test demo_flow operator_cannot_access_admin_node_type_management_routes`
  - `cargo test -p tessara-api --test demo_flow node_type_updates_reject_cycles_in_parent_child_selections`
  - `cargo test -p tessara-web`
- Roadmap update:
- Marked Sprint 1C as complete in `D:\Projects\dms-migration\tessara\docs\roadmap.md`.
  - Marked Sprint 1D as the next sprint focus.

## 2026-04-08

- Added progress-report tracking in the docs root at `D:\Projects\dms-migration\tessara\docs\progress-report.md`.
- Current roadmap position:
  - Slices 11-13 implemented.
  - Slice 14 implemented in a limited v1 form.
  - Slice 15 largely implemented.
  - Slice 16 partially implemented.
  - Slice 17 partially implemented.
  - Slices 18-23 remain.
- Latest completed implementation milestones in `tessara`:
  - `1b55038 Expose source-aware dataset report previews`
  - `e5ffd6a Model dataset composition modes`
  - `1aa6fd1 Add avg min and max aggregation metrics`
- Roadmap was updated to make the next major phase explicit:
  - Slice 18: Real Application Shell and Navigation
  - Slice 19: Entity Lists, Detail Views, and Creation Menus
  - Slice 20: Submission, Admin, and Reporting Workflow Parity
- Next planned development focus:
  - begin the real application shell
  - add home screen and persistent navigation
  - start replacing workbench-style routes with original-project-inspired application structure

## 2026-04-08

- Completed the first Slice 18 implementation checkpoint for the real application UI.
- Added a real application home at `/app` with:
  - overview content
  - persistent navigation
  - create-menu entry points
  - quick-start actions
- Split the submission workflow onto `/app/submissions` so the home route can act as a true landing page.
- Kept `/app/admin`, `/app/reports`, and `/app/migration`, but moved them under the shared application-frame structure with:
  - consistent navigation
  - shared create menu
  - shared output panels
- Updated smoke coverage to verify:
  - `/app` home shell
  - `/app/submissions`
  - `/app/admin`
  - `/app/reports`
  - `/app/migration`
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Next planned development focus:
  - continue Slice 18-20 by turning the new shell into real list/detail and creation-menu flows
  - begin replacing utility-style entry points with app-style entity screens
  - continue Slice 16 join execution in parallel with the new UI shell work

## 2026-04-08

- Completed the next Slice 18 checkpoint for the real application UI.
- Expanded `/app/admin` from a single setup screen into a clearer management workspace with:
  - `Management Areas` entry cards for hierarchy, forms, reporting, and dashboards
  - an `Entity Directory` for node types, nodes, forms, datasets, reports, aggregations, charts, and dashboards
  - direct screen-opening and data-loading actions tied to the existing admin APIs
- Updated the Leptos admin-shell tests and smoke checks so the new management structure is part of the regular quality gate.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 is in progress with the application home, persistent navigation, and the first admin management-area shell.
  - Slice 19 is next: stronger entity lists, detail views, and creation-menu flows.
  - Slice 16 dataset join execution still remains parallel backend work.
- Next planned development focus:
  - add app-style list/detail entry points for reporting and dataset management
  - continue replacing raw utility flows with screen-specific application interactions
  - begin the next set of entity-oriented routes inside the new application shell

## 2026-04-08

- Completed the next Slice 19-oriented application-shell increment for reporting.
- Expanded `/app/reports` into a clearer reporting landing area with:
  - `Reporting Areas` cards for datasets, reports, aggregations, and dashboards
  - a `Reporting Directory` for dataset, report, aggregation, chart, and dashboard list entry points
  - direct loading actions tied to the existing reporting APIs and preview screens
- Kept the existing report runner and dashboard preview screens underneath this new route-level structure so reporting stays testable while the UI becomes more application-like.
- Updated the Leptos route tests and smoke checks so the reporting landing structure is part of the normal validation path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 remains in progress with route-level home/navigation structure now in place for home, admin, and reporting.
  - Slice 19 is actively underway through route-level entity directories and management-area entry points.
  - Slice 16 dataset join execution is still pending as parallel backend work.
- Next planned development focus:
  - continue route-level application-shell upgrades for migration and submission contexts
  - add stronger list/detail entry points so entity browsing relies less on raw IDs
  - return to dataset join execution after the next UI-shell checkpoint

## 2026-04-08

- Completed the next Slice 19-oriented application-shell increment for migration.
- Expanded `/app/migration` into a clearer operator route with:
  - `Migration Stages` cards for fixture intake, validation, dry run, and import
  - a `Migration Directory` for fixture examples, validation, dry runs, and imports
  - direct actions wired to the existing legacy-fixture APIs
- Kept the existing validation and import workbench surfaces underneath this route so migration rehearsal remains testable while the UI becomes more structured.
- Updated the Leptos route tests and smoke checks so the new migration landing structure is part of the regular validation path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 remains in progress, but the focused routes now have route-level structure for home, admin, reporting, and migration.
  - Slice 19 continues through stronger entity directories and route entry points.
  - Slice 16 dataset join execution remains outstanding backend work.
- Next planned development focus:
  - improve the submissions route so it behaves more like a real list/detail application area
  - keep reducing raw-ID dependence across entity browsing
  - return to dataset join execution once the next UI-shell checkpoint lands

## 2026-04-08

- Completed the next Slice 19-oriented application-shell increment for submissions.
- Expanded `/app/submissions` into a clearer route-level workspace with:
  - `Submission Stages` cards for response entry, target selection, response review, and related reports
  - a `Response Directory` for published forms, target nodes, draft responses, submitted responses, all responses, and related reports
  - direct actions wired to the existing published-form, node, submission, and report APIs
- Kept the detailed submission, review, and related-report screens underneath this route so the application shell gets more structured without losing the currently testable workflows.
- Updated the Leptos route tests and smoke checks so the new submissions landing structure is part of the regular validation path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 route-level application shell work is now in place across home, submissions, admin, reporting, and migration.
  - Slice 19 continues through stronger list/detail entry points and reduced raw-ID dependence.
  - Slice 16 dataset join execution remains the next major backend gap.
- Next planned development focus:
  - move back to dataset join execution with explicit join semantics and diagnostics
  - continue refining list/detail entry points where the application shell still depends too heavily on raw IDs

## 2026-04-08

- Completed the next Slice 16 backend checkpoint for dataset composition.
- Added execution support for join-mode dataset tables when:
  - the dataset uses submission grain
  - the dataset has at least two sources
  - every source uses `latest` or `earliest` selection so each source resolves to one row per node
- Join-mode dataset execution now merges selected source rows by node and returns one dataset row with:
  - combined field values across sources
  - a joined `submission_id` trace showing the contributing source/submission pairs
  - `source_alias` set to `join`
- Left dataset-backed reports on union-only execution for now, so report-engine refactoring remains a later slice rather than being mixed into this backend checkpoint.
- Added DB-backed integration coverage for:
  - successful join-mode dataset execution across two forms on the same node
  - invalid join-mode execution when a source uses the `all` selection rule
  - clearer diagnostics for single-source join datasets
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow`
  - `cargo clippy -p tessara-api --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 route-level application shell work is in place across all focused app routes.
  - Slice 19 continues through stronger list/detail entry points and reduced raw-ID dependence.
  - Slice 16 has now advanced from modeled join semantics to actual join-mode dataset table execution.
- Next planned development focus:
  - continue Slice 16 by deciding how dataset-backed reports should query joined datasets
  - keep tightening application-shell list/detail flows where workbench patterns are still visible

## 2026-04-08

- Completed the next Slice 16 backend checkpoint for joined-dataset reporting.
- Added support for dataset-backed reports to run against join-mode datasets by reusing the internal dataset execution path and projecting report bindings over the joined dataset rows.
- Join-backed reports now support:
  - direct dataset field bindings
  - `literal:` computed expressions
  - `bucket_unknown` handling for missing joined values
  - joined submission traces carried through report output
- Kept the existing SQL-backed path for union datasets and form-backed reports, so this change extends the reporting model without forcing a broader report-engine rewrite in the same slice.
- Added validation/test coverage for:
  - creating and running a report on a joined dataset
  - preserving the clearer diagnostics for invalid join datasets
  - pure helper behavior for literal computed expressions and joined missing-data handling
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow`
  - `cargo clippy -p tessara-api --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 has advanced from modeled joins to join-mode dataset execution and joined-dataset report execution.
  - Slice 18 route-level application shell work is in place across all focused routes.
  - Slice 19 remains active through list/detail and reduced raw-ID UI work.
- Next planned development focus:
  - decide whether charts/dashboards need explicit joined-dataset affordances next
  - continue application-shell list/detail improvements where entity browsing still feels workbench-like

## 2026-04-08

- Completed the next reporting-stack verification checkpoint for joined datasets.
- Added DB-backed integration coverage proving that:
  - a report built on a joined dataset can feed an aggregation definition
  - the aggregation engine correctly computes metrics over joined-dataset report rows
  - the joined reporting path behaves like a first-class reporting source rather than a dead-end dataset preview
- This was mainly a coverage/hardening checkpoint rather than a large new code-path change, but it closes an important uncertainty in the dataset-first reporting roadmap.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow join_mode_datasets_merge_selected_source_rows_by_node -- --exact`
  - `cargo clippy -p tessara-api --all-targets -- -D warnings`
  - `cargo test --workspace`
- Current roadmap position:
  - Slice 16 now covers join-mode dataset execution, joined-dataset reports, and aggregation consumption of joined report rows.
  - Slice 18 route-level application shell work remains in place across all focused routes.
  - Slice 19 remains the next major UX area to continue.
- Next planned development focus:
  - return to list/detail and entity-browsing improvements in the application shell
  - add more app-style reporting/admin detail flows where raw-ID workflows still dominate

## 2026-04-08

- Completed the next reporting-stack hardening checkpoint for joined datasets.
- Added DB-backed integration coverage proving that a joined-dataset report can flow through:
  - an aggregation definition
  - an aggregation-backed chart
  - a dashboard component rendered from that aggregation-backed chart
- This was another verification-oriented slice rather than a broad implementation rewrite, but it materially reduces risk in the dataset-first reporting roadmap by confirming that joined data survives the full report-to-dashboard path.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-api --test demo_flow join_mode_datasets_merge_selected_source_rows_by_node -- --exact`
  - `cargo test --workspace`
- Current roadmap position:
  - Slice 16 joined-dataset backend work is now covered through dataset execution, reports, aggregations, and dashboard consumption.
  - Slice 18 route-level application shell work remains in place across all focused routes.
  - Slice 19 remains the next major area to continue.
- Next planned development focus:
  - return to app-style list/detail improvements
  - continue reducing raw-ID-heavy workflows in admin and reporting screens

## 2026-04-08

- Completed the next Slice 19 reporting-route increment.
- Fixed a concrete usability/consistency gap in the focused reporting application route:
  - the route-level shell already advertised dataset entry points
  - the focused app controller now actually supports dataset browsing, inspection, and dataset-result preview on `/app/reports`
- Added:
  - dataset context selection in the focused reporting route
  - dataset definition inspection in the focused reporting route
  - dataset table preview in the focused reporting route
  - report-context selection that carries dataset context when a report is dataset-backed
- Updated route tests and smoke checks so the reporting route now validates:
  - `Choose Dataset`
  - `Inspect Dataset`
  - `Run Dataset`
  - focused dataset API references in the app shell
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `cargo test -p tessara-api --test demo_flow join_mode_datasets_merge_selected_source_rows_by_node -- --exact`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset backend work is substantially advanced and now covered through dataset, report, aggregation, and dashboard paths.
  - Slice 18 route-level application shell work remains in place across all focused routes.
  - Slice 19 continues through stronger list/detail flows and reduced raw-ID dependence in focused app routes.
- Next planned development focus:
  - continue list/detail and entity-browsing improvements on the focused app routes
  - target the next workbench-heavy admin/reporting flow that still lacks good application-style selection and inspection

## 2026-04-08

- Completed a follow-on Slice 19 UI hardening increment for dataset browsing.
- Fixed a concrete joined-dataset usability issue in both the focused reporting route and the admin workbench:
  - dataset preview rows previously assumed one submission ID per row
  - joined dataset rows now render per-source submission actions instead of a broken single “Open Submission” action
- This keeps joined dataset previews usable now that join-mode datasets, joined reports, and joined-report aggregations are supported elsewhere in the stack.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset backend work is substantially advanced.
  - Slice 18 route-level application shell work is in place across all focused routes.
  - Slice 19 continues through smaller usability fixes and stronger entity/detail workflows.
- Next planned development focus:
  - continue reducing raw-ID-heavy admin/reporting flows
  - move another high-friction workbench interaction toward a more application-style detail/browse flow
## 2026-04-08

- Completed the next Slice 19 reporting-route usability increment.
- Added chart definition inspection as a first-class reporting flow instead of treating charts as ID-only launch points.
- Added a new authenticated API detail route for charts that returns:
  - chart/report or chart/aggregation linkage
  - dashboards currently using that chart
- Updated the focused reporting route so testers can:
  - inspect a chart from the chart list
  - inspect a chart from dashboard preview cards
  - follow chart detail into linked reports, aggregations, and dashboards
- Added DB-backed integration coverage for the new chart detail route on seeded report/dashboard data.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution is substantially advanced and usable through datasets, reports, aggregations, and dashboards.
  - Slice 18 route-level application shell work is in place across the focused routes.
  - Slice 19 continues through stronger entity detail and browse flows in reporting/admin screens.
- Next planned development focus:
  - continue reducing raw-ID-heavy admin/reporting interactions
  - move another reporting or admin builder seam toward a clearer list/detail workflow
  - continue Slice 20 hardening where the app still behaves like a workbench instead of a replacement UI
## 2026-04-08

- Completed a follow-on Slice 19 reporting-detail increment in the same batch.
- Extended report definition responses so report detail now includes downstream dependents:
  - aggregations built from the report
  - charts built directly from the report or from those aggregations
- Updated the focused reporting route so report detail cards now support traversal into:
  - linked aggregations
  - linked charts
- This makes report inspection behave more like an application detail page and less like a detached workbench form.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through richer entity detail/traversal flows on reporting and admin routes.
- Next planned development focus:
  - continue replacing remaining raw-ID-heavy reporting/admin interactions
  - improve another browse/detail seam in admin or submission review flows
  - continue Slice 20 hardening where workflows still feel more operator-like than end-user-like
## 2026-04-08

- Completed another Slice 19 admin/reporting detail increment.
- Extended dataset definitions so dataset detail now includes linked reports that depend on the dataset.
- Updated both the focused reporting route and the admin workbench so dataset inspection can now flow directly into report context and report detail.
- This removes another workbench-style dead end: datasets now behave more like application entities with downstream navigation instead of isolated builder records.
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through stronger list/detail and cross-entity traversal flows.
- Next planned development focus:
  - continue replacing raw-ID-heavy admin interactions
  - improve another builder/detail seam, likely around forms or hierarchy setup
  - continue Slice 20 hardening where the UI still behaves more like an operator workbench than a replacement application
## 2026-04-08

- Completed the next Slice 19 admin-detail increment around forms.
- Added a dedicated form detail API surface so forms can now be inspected as first-class entities instead of only appearing in a builder list.
- Form detail now includes:
  - versions
  - linked reports that use the form directly
  - linked dataset sources that use the form directly or a form major line
- Updated the admin shell so testers can:
  - inspect a selected form
  - move directly from form detail into versions, linked reports, and linked datasets
- Validation completed successfully:
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through stronger admin/reporting list/detail and cross-entity traversal flows.
- Next planned development focus:
  - continue replacing raw-ID-heavy hierarchy or form-builder interactions
  - improve another admin/detail seam with clearer contextual navigation
  - continue Slice 20 hardening where the UI still feels more operator-like than replacement-ready
## 2026-04-08

- Completed the next Slice 19 hierarchy-detail increment.
- Added a dedicated node-type detail API surface so node types can now be inspected as first-class admin entities instead of only appearing in the hierarchy builder list.
- Node-type detail now includes:
  - allowed parent node types
  - allowed child node types
  - metadata fields
  - forms scoped to that node type
- Updated the admin shell so testers can:
  - inspect a node type
  - move directly from node-type detail into parent/child relationship context, metadata-field context, and scoped forms
- Validation completed successfully:
  - `cargo fmt --all`
  - `cargo fmt --all --check`
  - `cargo test -p tessara-web`
  - `cargo test -p tessara-api --test demo_flow demo_seed_report_and_dashboard_flow_works_against_database -- --exact`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 16 joined-dataset execution remains substantially advanced.
  - Slice 18 application shell structure remains in place.
  - Slice 19 continues through richer admin/reporting entity detail and traversal flows.
- Next planned development focus:
  - continue replacing raw-ID-heavy builder interactions, likely at the version-level form or node-detail layer
  - improve another admin/detail seam with contextual navigation
  - continue Slice 20 hardening where workflows still feel more operator-like than replacement-ready
## 2026-04-08

- Completed the first explicitly visible application-UI shift on the submissions route.
- Reworked `/app/submissions` so it now presents a route-level response workspace instead of only stacked utility panels.
- Added:
  - a response-console shell
  - queue-style entry cards for published forms, targets, drafts, and submitted responses
  - a guided-path panel describing the normal response flow
  - a split workspace layout that keeps the active entry/review/report sections together on the main side of the route
- This does not complete the application-UI transition, but it is the first change in this thread that should read more like a product workspace than a builder screen.
- Validation completed successfully:
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 application shell structure is in place.
  - Slice 19 is now continuing not only through entity-detail traversal, but also through visibly more application-like route layouts.
- Next planned development focus:
  - keep converting the focused routes into real workspace pages
  - reduce exposed ID-driven controls on the highest-traffic submission/admin surfaces
  - continue Slice 20 hardening as those routes become more end-user-facing
## 2026-04-08

- Paused non-UI roadmap work to focus directly on the application surface.
- Extended the visible workspace treatment beyond submissions so the focused routes now read more like destination pages:
  - `/app/submissions` already had the response console
  - `/app/admin` now has a configuration-console workspace shell
  - `/app/reports` now has an insight-console workspace shell
- Added route-level queue panels and guided-path content for admin and reporting so those routes no longer appear as only stacked builder sections.
- This is still an intermediate UI state, but it is a clearer step toward the desired application structure and away from a pure operator workbench.
- Validation completed successfully:
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - active focus is now intentionally UI-first
  - non-UI roadmap work is paused until the focused routes look more like the intended application
- Next planned development focus:
  - keep converting focused routes into real workspace pages
  - reduce or demote exposed ID-driven controls on submission/admin/reporting surfaces
  - continue reshaping the app shell toward the desired application information architecture before resuming deeper backend roadmap work
## 2026-04-08

- Continued the UI-only catch-up pass without extending beyond supported backend workflows.
- Reworked the remaining focused admin screens so they now use the same task/context layout already introduced on submission and reporting routes:
  - hierarchy setup now separates hierarchy actions from current hierarchy context
  - form builder now separates form actions from current form context
  - report builder now separates reporting configuration actions from current reporting-builder context
- Added shared task-panel styling so the focused routes now use one visible UI grammar instead of mixing workspace shells with older utility-style slabs.
- This is still not full replacement-grade product UI, but it is a meaningful catch-up step because the main focused routes now read more like application workspaces and less like a raw control surface.
- Validation completed successfully:
  - `cargo fmt --all`
  - `cargo test -p tessara-web`
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings`
  - `.\scripts\smoke.ps1`
- Current roadmap position:
  - Slice 18 remains in place as the real application shell foundation
  - Slice 19 UI catch-up is still active and is now focused on making the focused routes visually coherent and less ID-forward
  - non-UI roadmap work remains paused by request
- Next planned development focus:
  - continue demoting raw-ID-heavy controls on the highest-traffic focused routes
  - push `/app/admin` and `/app/reports` farther toward browse/detail/task workflows instead of builder-style control clusters
  - stop short of inventing UI flows the backend cannot already support
## 2026-04-08

- Paused implementation work briefly to review provenance and the legacy application templates before continuing UI catch-up.
- Reviewed:
- `D:\Projects\dms-migration\tessara\docs\provenance\System Requirements Specification.pdf`
- `D:\Projects\dms-migration\tessara\docs\provenance\Software Design Document.pdf`
- `D:\Projects\dms-migration\tessara\docs\provenance\Requirements Traceability Matrix - Sheet1.pdf`
  - legacy templates from `app/mmi/templates` in the previous application
- Key UI findings from the prior application:
  - role-specific home pages matter
  - manage-list pages and detail/drill-down flows are the core structure
  - client/parent workflows center on assigned forms and completed forms
  - reports and dashboards are top-level destinations, not hidden utilities
  - breadcrumbs, search, and contextual actions are part of the expected navigation model
- Created `D:\Projects\dms-migration\tessara\docs\archive\docs\user-interface-design.md` to define the proposed Tessara UI structure based on those materials while still allowing usability improvements over the old application.
- Current roadmap position:
  - UI-only catch-up is still the active focus
  - the next UI work should follow the design note’s structure instead of continuing ad hoc shell refinement
- Next planned development focus:
  - align the current app shell to the role/home/directory/detail structure documented in `user-interface-design.md`
  - continue using the legacy product structure as a guide without reproducing the old UI exactly
## 2026-04-08

- Created `D:\Projects\dms-migration\tessara\docs\ui-direction.md` as the near-term UI implementation charter.
- The new direction document translates `user-interface-design.md` into a shorter, locked set of implementation decisions.
- Locked decisions recorded there:
  - split product areas are the primary information architecture
  - shared home first, role-aware variants later
  - Administration and Migration remain visible but scoped internal surfaces
  - terminology is configuration-driven rather than hardcoded to legacy entities
- The new document also defines:
  - the primary product areas
  - the home strategy
  - the screen-family patterns
  - the mapping from current routes to the target UI structure
  - the acceptance criteria for considering the UI catch-up direction settled
- Current roadmap position:
  - UI-only catch-up remains active
  - the next implementation work should follow `ui-direction.md` rather than continue route-by-route shell refinement without a locked direction
- Next planned development focus:
  - align the current route structure and screen composition to `ui-direction.md`
  - continue making the UI behave like product destinations instead of mixed control surfaces
## 2026-04-08

- Updated `D:\Projects\dms-migration\tessara\docs\roadmap.md` to reflect the current implementation state and the new UI direction.
- The roadmap now clearly separates:
  - completed foundation and reporting work
  - active carry-forward UI gaps
  - future work as discrete, non-overlapping sprints
- Added the UI-direction-driven sprint sequence at the current point in the roadmap:
  - shared shell and product-area navigation
  - shared home and role-ready entry points
  - Organization and Forms product surfaces
  - Responses product surface
  - Reports and Dashboards product surfaces
  - Administration and Migration internal surfaces
- The later reporting/migration/hardening work is now pushed into subsequent sprints instead of overlapping with the active UI catch-up phase.
- Current roadmap position:
  - backend/reporting architecture remains substantially ahead of the UI
  - UI-only catch-up remains the active implementation focus
- Next planned development focus:
  - begin Sprint 1 from the updated roadmap
  - use `ui-direction.md` as the implementation charter for the next UI changes
## 2026-04-08 - Sprint 1 UI Slice: Split Product-Area Shell

Roadmap position:
- Active focus remains Sprint 1: Shared Shell and Product-Area Navigation.
- This slice moved the app from focused utility routes toward the ui-direction.md information architecture without extending beyond backend-supported workflows.

Completed in this slice:
- Added canonical product-area routes for Home, Organization, Forms, Responses, Reports, Dashboards, Administration, and Migration.
- Split navigation into Product Areas and Internal Areas.
- Added shared breadcrumb/title shell framing across the focused application routes.
- Reframed existing supported screens under bridge surfaces for Organization, Forms, Responses, Reports, Dashboards, and Administration.
- Preserved compatibility routes (/app/submissions, /app/admin) while shifting visible language to the new IA.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\scripts\smoke.ps1

Next UI steps:
- Continue Sprint 1 by making the shared shell more consistent across product areas.
- Begin Sprint 2-style home and entry-point refinement only where backend support already exists.
- Keep reducing raw-ID-forward route language in favor of product-area browse/detail/task framing.
## 2026-04-08 - Sprint 1 UI Slice: Keep Builder Shortcuts In Internal Areas

Roadmap position:
- Sprint 1 remains active.
- This checkpoint tightened the split between product-facing routes and internal/operator routes without adding any new backend requirements.

Completed in this slice:
- Added a shared route sidebar component for the focused application shells.
- Removed create-shortcut panels from Home, Organization, Forms, Responses, Reports, Dashboards, and Migration.
- Kept creation shortcuts in Administration, where the supported configuration workflows actually live.
- Repointed the form creation shortcut to Administration so product routes do not advertise builder-first behavior.
- Updated web tests and smoke coverage to enforce that product routes no longer expose internal create shortcuts.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\scripts\smoke.ps1

Next UI steps:
- Continue Sprint 1 shell consistency work across product routes.
- Strengthen page-level title/action framing so the main routes read like one application family.
- Keep product routes focused on browse/detail/task flows and leave configuration entry points in Administration.
## 2026-04-08 - Sprint 1 UI Slice: Unify Product-Area Page Shells

Roadmap position:
- Sprint 1 remains active.
- This checkpoint improved shell consistency across the focused application routes without expanding UI scope beyond existing backend support.

Completed in this slice:
- Added a shared AppAreaShell component for route-level hero, breadcrumb, action-row, sidebar, and main content framing.
- Moved Home, Organization, Forms, Responses, Reports, Dashboards, Administration, and Migration onto the same page-shell pattern.
- Kept the product/internal area split from the prior slice while removing duplicated route-shell markup.
- Simplified the home-route smoke assertion so it validates visible navigation content instead of brittle raw href text.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1

Next UI steps:
- Continue Sprint 1 by tightening route-level page titles, action framing, and browse/detail entry patterns.
- Keep product areas aligned as one application family while leaving unsupported workflows out of the UI.
## 2026-04-08 - Sprint 1 UI Slice: Standardize Area Landing Sections

Roadmap position:
- Sprint 1 remains active.
- This checkpoint tightened the route-level UI inside the shared shell so the focused product and internal areas use a more consistent landing-page grammar.

Completed in this slice:
- Added shared landing-section helpers for titled screen sections, management cards, and directory cards.
- Moved Organization, Forms, Responses, Administration, Reports, Dashboards, and Migration home/landing screens onto the same section/card rendering pattern.
- Preserved existing route titles, actions, and supported workflows while removing duplicated markup across the route landings.
- Kept the UI limited to backend-supported browse, review, run, and configuration flows.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1

Next UI steps:
- Continue Sprint 1 with stronger route-level browse/detail framing.
- Keep reducing the visual gap between current bridge surfaces and the intended application destinations.
- Avoid adding UI breadth beyond existing backend support.
## 2026-04-08 - Sprint 1 UI Slice: Promote Full-Size Brand Mark

Roadmap position:
- Sprint 1 remains active.
- This was a focused shell polish change within the existing application-shell work.

Completed in this slice:
- Switched img.brand-mark in the shared application shell and local admin shell from the 256 asset to the full-size 1024 icon asset.
- Increased the brand mark display size and adjusted spacing so the icon reads as a primary brand element in the shell header.
- Updated web-shell tests to assert the new full-size icon asset reference.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-web --all-targets -- -D warnings

Next UI steps:
- Continue Sprint 1 route-level framing work.
- Keep improving visible application-shell coherence without adding unsupported UI behavior.
## 2026-04-08 - Local Launch Helper

Roadmap position:
- This is a developer-workflow improvement alongside the active Sprint 1 UI work.

Completed in this slice:
- Added scripts/local-launch.ps1 to stop the existing Compose stack, rebuild the API image, recreate services, and wait for /health and /app to return 200.
- Added optional flags for a fresh Postgres volume refresh and log following.
- Documented the helper in README.md as the recommended local rebuild/relaunch path for UI and user-testing updates.
- Verified the helper by running it successfully against the local Compose stack.

Validation:
- powershell -ExecutionPolicy Bypass -File .\\scripts\\local-launch.ps1

Next UI/dev workflow steps:
- Use local-launch.ps1 as the standard refresh path when checking UI changes in Docker Compose.
## 2026-04-09 - Sprint 1 UI Slice: Standardize Workspace Shells

Roadmap position:
- Sprint 1 remains active.
- This checkpoint standardized the workspace layer under the already-shared route shells.

Completed in this slice:
- Added shared workspace-shell helpers for queue cards and path sections.
- Moved Organization, Forms, Responses, Administration, Reports, and Dashboards workspace shells onto the same queue/path/workspace rendering pattern.
- Preserved the existing supported actions, route bridges, and underlying screens while removing more duplicated route-console markup.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1 blocked because Docker Desktop / the Docker daemon was unavailable on the machine at run time.

Next UI steps:
- Continue Sprint 1 route-level browse/detail framing.
- Keep tightening the product-area experience without adding unsupported UI behavior.
## 2026-04-09 - Sprint 1 UI Slice: Keep Product-Area Anchors Out Of Builder Language

Roadmap position:
- Sprint 1 remains active.
- This checkpoint continues tightening route-level framing while staying inside the existing backend-supported UI surfaces.

Completed in this slice:
- Continued standardizing workspace-level route shells through shared queue/path helpers.
- Reduced route-specific UI drift under the product-area shells.
- Preserved existing actions and route bridges while moving the route consoles closer to one consistent application-shell pattern.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1 was blocked because the Docker daemon was unavailable at run time.

Next UI steps:
- Continue Sprint 1 route-level browse/detail framing.
- Keep reducing builder-era language where product routes still inherit it from reused screens.
## 2026-04-09 - Sprint 1 UI Slice: Replace Builder-Era Route Anchors

Roadmap position:
- Sprint 1 remains active.
- This checkpoint keeps aligning route-level UI language with the product-area shell without changing supported behavior.

Completed in this slice:
- Replaced builder-era route anchors like hierarchy-admin-screen, form-admin-screen, report-admin-screen, and submission-screen with route-appropriate screen IDs.
- Updated product and internal route links so route navigation no longer advertises admin-first anchor names where the screens are reused.
- Kept the underlying actions and reused screens intact.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings

Next UI steps:
- Continue Sprint 1 browse/detail framing.
- Keep reducing inherited builder-era language on shared screens and route entry points.
## 2026-04-09 - Sprint 1 UI Slice: Remove More Builder-Era Screen Language

Roadmap position:
- Sprint 1 remains active.
- This checkpoint continues the route-language cleanup inside reused screens.

Completed in this slice:
- Renamed reused screen headers and labels to better match product-area and administration route language.
- Responses screens now use Response Entry, Response Review, and Response Reports language.
- Administration screens now use Organization Setup, Forms Configuration, and Reporting Configuration language.
- Migration fixture screen now uses Fixture Intake and Validation language.
- Updated route tests and smoke expectations to match the new screen wording.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings
- .\\scripts\\smoke.ps1 was blocked because the Docker daemon was unavailable at run time.

Next UI steps:
- Continue Sprint 1 browse/detail framing on the product routes.
- Keep reducing the remaining internal-builder feel on reused screens without adding unsupported UI behavior.
## 2026-04-09 - Sprint 1 UI Slice: Make Reused Screens Route-Aware

Roadmap position:
- Sprint 1 remains active.
- This checkpoint improves reused screen framing on the product routes without changing underlying behavior.

Completed in this slice:
- Made the shared organization/forms management screens route-aware so Organization and Forms routes no longer inherit administration-first titles.
- Organization route now renders product-surface labels like Organization Screen and Organization Directory.
- Forms route now renders product-surface labels like Forms Screen and Forms Directory.
- Administration still keeps Organization Setup and Forms Configuration language.
- Added test coverage to prove the product routes and administration route now diverge correctly in visible labeling.

Validation:
- cargo fmt --all
- cargo test -p tessara-web
- cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings

Next UI steps:
- Continue Sprint 1 browse/detail framing.
- Keep reducing the remaining internal-builder feel on reused screens and route entry points.
## 2026-04-09 - Sprint 1 checkpoint: route interaction language cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Updated product-route cards and task buttons to emphasize browse, review, and view actions instead of generic builder-style `Open` and `Choose` wording.
  - Kept explicit create/configure language scoped to Administration.
  - Tightened reporting and response route wording so the visible UI reads more like a product surface and less like a reused internal workbench.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
  - `.\scripts\smoke.ps1` was blocked because the Docker daemon was unavailable (`dockerDesktopLinuxEngine` pipe not found).
- Next step:
  - Continue Sprint 1 by improving browse/detail framing on the product routes while keeping UI work bounded to the backend-supported workflows already implemented.

## 2026-04-09 - Sprint 1 checkpoint: product workspace framing

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Renamed product-route page shells from `Console` wording to `Workspace` wording.
  - Renamed route-side rail sections from `Queues` to `Browse` on product surfaces.
  - Renamed route-side step sections from generic `Path` wording to `Flow` wording on product surfaces.
  - Kept Administration as `Configuration Console` with `Management Queues`, since it remains the internal management surface.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
  - `.\scripts\smoke.ps1` was blocked because the Docker daemon was unavailable (`dockerDesktopLinuxEngine` pipe not found).
- Next step:
  - Continue Sprint 1 by tightening browse/detail framing inside the product workspaces while leaving creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: product screen framing cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Updated product-route inner screen framing so reused screens read less like technical shells and more like application surfaces.
  - Replaced product-route subheadings such as `Organization Screen`, `Forms Screen`, and `Reports Screen` with `Workspace` wording.
  - Replaced product-route context labels such as `Current ... Context` with selection-oriented labels where those routes act as browse/detail surfaces.
  - Kept Administration-specific configuration wording unchanged.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by tightening browse/detail framing and output presentation on product routes while keeping creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: route-specific detail panels

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Replaced the shared `Screen Output` panel on app routes with route-specific detail/result panels.
  - Replaced the shared `Raw Output` panel with `Raw API Activity` on product routes and `Raw API Output` on internal routes.
  - Made the bottom-of-page framing read like route-specific detail/result space instead of a leftover workbench/debug panel.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by tightening browse/detail framing on the remaining product-route internals while keeping builder and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: landing label consistency

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Normalized the remaining mixed landing-card labels on product routes.
  - Home cards now use `Go to ...` wording where they navigate to another product area.
  - Reporting landing cards now use browse/review/view wording instead of `Open ... Workspace`.
  - Dashboard landing cards now use `View Demo Preview` rather than another generic `Open` label.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing remaining low-level/product-route friction while keeping creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: shared sidebar wording cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Renamed the shared selection panel from `Selection Context` to `Current Selections`.
  - Reframed the shared session/summary actions with more application-oriented wording:
    - `Sign In`
    - `Session Status`
    - `Sign Out`
    - `Refresh Summary`
  - Updated route tests and smoke expectations to match the new shared-shell labels.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing remaining generic shell/test-harness wording on product routes while keeping creation and configuration emphasis in Administration.

## 2026-04-09 - Sprint 1 checkpoint: product-route description cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Removed another layer of transitional `bridge`, `catch-up`, and similar wording from product-route descriptions.
  - Reframed the home, organization, forms, responses, reports, and dashboards descriptions so they read more like stable application areas.
  - Corrected a duplicated forms landing description field introduced during the copy pass.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing the remaining generic product-route wording and tightening the visible information architecture before moving on to the next sprint.

## 2026-04-09 - Sprint 1 checkpoint: dashboard action-row consistency

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Normalized the dashboard route action row to match the rest of the shared shell.
  - Updated dashboard route action labels to:
    - `Sign In`
    - `Session Status`
    - `Sign Out`
    - `Refresh Summary`
  - Added dashboard-route assertions so the shared-shell wording stays consistent across product areas.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by reducing the remaining generic product-route wording and tightening the visible information architecture before moving on to the next sprint.

## 2026-04-09 - Sprint 1 checkpoint: product header action cleanup

- Roadmap position: Sprint 1, Shared Shell and Product-Area Navigation, still in progress.
- Scope: UI-only catch-up, bounded to backend-supported screens and actions.
- Completed:
  - Removed demo/setup shortcuts from product-area header action rows.
  - Product routes now use refresh-oriented header actions instead:
    - `Refresh Organization`
    - `Refresh Forms`
    - `Refresh Responses`
    - `Refresh Reports`
    - `Refresh Dashboards`
  - Home and internal areas remain the place for demo seeding and setup-oriented entry points.
  - Updated route tests and smoke expectations to match the new product-header contract.
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
- Validation:
  - `cargo fmt --all` passed.
  - `cargo test -p tessara-web` passed.
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed.
- Next step:
  - Continue Sprint 1 by tightening the remaining visible information architecture and deciding whether the shared-shell/navigation sprint is complete enough to move to the next sprint.

## 2026-04-10 - Sprint 1 complete

- Roadmap position:
  - Sprint 1, Shared Shell and Product-Area Navigation, is complete.
  - Sprint 2, Home Surfaces and Role-Ready Entry Points, is the next planned sprint.
- Completion rationale:
  - split product-area navigation is in place
  - shared shell framing is consistent across product and internal areas
  - product routes now read as workspaces and viewing destinations rather than mixed builder consoles
  - Administration and Migration remain visible but clearly internal/operator scoped
- Current review point:
  - the UI is now in a good place to review Sprint 1 shell/navigation progress before moving deeper into home-surface and role-ready work

## 2026-04-10 - Sprint 2 start: home surface and layout correction

- Roadmap position:
  - Sprint 1 is closed.
  - Sprint 2, Home Surfaces and Role-Ready Entry Points, is now active.
- Scope:
  - start the shared-home refactor without adding backend scope
  - fix the shared `task-panel context-panel` layout overflow
- Completed:
  - corrected the shared task/context grid so `section.task-panel.context-panel` breaks below the preceding panel instead of escaping the parent layout
  - removed demo seeding from the shared home header actions
  - refactored `/app` into clearer home modules:
    - product areas
    - current deployment readiness
    - current workflow context
    - internal areas
  - wired the home readiness module to the existing `/api/app/summary` surface
  - wired the home current-context module to the existing shared selection state
  - moved demo setup emphasis into Administration through a local testing utility card
  - updated the roadmap to remove stale Sprint 1 carry-forward notes and mark Sprint 2 active
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\app_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - continue Sprint 2 by tightening the shared home modules and reducing any remaining transitional copy on product-facing routes

## 2026-04-10 - Sprint 2 checkpoint: remove visible ID entry fields

- Roadmap position:
  - Sprint 2 remains active.
  - Scope stays bounded to backend-supported UI catch-up work.
- Completed:
  - removed visible `ID` entry fields from the rendered application screens
  - kept selection-driven state in hidden inputs so existing controller flows still work
  - updated task/context panels so creation flows no longer imply that users should type or override database-assigned identifiers
  - added route-test coverage to prevent the main product and administration screens from regressing back to visible `ID` fields
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - continue Sprint 2 by tightening the shared home modules and reducing remaining transitional copy on product-facing routes

## 2026-04-10 - Sprint 2 complete

- Roadmap position:
  - Sprint 2, Home Surfaces and Role-Ready Entry Points, is complete.
  - Sprint 3, Organization And Forms Product Surfaces, is the next planned sprint.
- Completed:
  - added explicit role-ready home modules to the shared home without introducing separate role routes
  - removed remaining transitional product-facing copy on the shared home and key reports/dashboards surfaces
  - removed the demo dashboard shortcut from the Dashboards product surface
  - completed the Sprint 2 contract:
    - shared home as the real product entry point
    - structural role readiness only
    - existing summary and selection surfaces reused
    - demo/testing utilities scoped to internal placement
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - begin Sprint 3 on Organization and Forms product surfaces

## 2026-04-10 - Sprint 2.5 complete

- Roadmap position:
  - Sprint 2.5, Entity CRUD/List Surfaces, is complete.
  - Sprint 3, Organization And Forms Product Surfaces, is next.
- Completed:
  - inserted Sprint 2.5 into the roadmap between Sprint 2 and Sprint 3
  - added runtime organization detail retrieval with `GET /api/nodes/{node_id}`
  - converted product routes to explicit entity list/detail surfaces for:
    - Organization
    - Forms
    - Responses
    - Reports
    - Dashboards
  - split list output from selected-detail output in the browser controllers
  - added clearer Administration create/edit entry points for top-level:
    - Organization
    - Form
    - Report
    - Dashboard
  - kept response creation/editing in Responses through the existing draft lifecycle
  - removed more product-facing ID-driven friction by using selection-driven detail flows and hiding explicit ID entry from the rendered screens
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\src\hierarchy.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\tests\demo_flow.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\app_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
  - `.\scripts\local-launch.ps1` passed
- Next step:
  - resume Sprint 3 on deeper Organization and Forms product surfaces

## 2026-04-10 - Sprint 2.5 replacement complete

- Roadmap position:
  - Sprint 2.5 remains complete, but it now reflects the dedicated-screen replacement rather than the earlier workspace-panel interpretation.
  - Sprint 3, Organization And Forms Product Surfaces, remains next.
- Completed:
  - replaced the product-area workspace-panel approach with dedicated navigable screens for:
    - Organization
    - Forms
    - Responses
    - Reports
    - Dashboards
  - added explicit product-area routes for:
    - list
    - create
    - detail
    - edit
  - kept IDs in route paths only and removed them from visible form fields
  - moved top-level entity CRUD/view workflows into product areas and stopped extending the internal admin/testing screens for those flows
  - kept Administration as an internal advanced/configuration landing area with links to legacy tooling
  - kept Responses as the canonical draft/edit/review surface:
    - dedicated start screen
    - dedicated detail screen
    - dedicated draft-only edit screen
    - submitted responses remain read-only
  - kept Report create/edit on dedicated pages with a minimal binding editor required by the current backend
- Files updated:
  - `D:\Projects\dms-migration\tessara\crates\tessara-api\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\app_script.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\lib.rs`
  - `D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs`
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`
  - `D:\Projects\dms-migration\tessara\docs\roadmap.md`
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
- Next step:
  - begin Sprint 3 by deepening Organization and Forms product surfaces now that the dedicated top-level entity screens are in place

## 2026-04-10 - Sprint 3 sequencing update

- Decision:
  - Sprint 3 should begin with a frontend code organization pass.
- Added to the roadmap:
  - the first Sprint 3 task is now to refactor the product UI code by route/screen before adding deeper Organization and Forms behavior
- Reason:
  - the UI is now route-driven
  - keeping the current large page/controller files in place would slow Sprint 3 work and increase regression risk
- Immediate next step:
  - split the shared shell/navigation code from Organization and Forms screen modules, then continue Sprint 3 feature work on top of that structure

## 2026-04-10 - UAT demo seed dataset and local launch integration

- Completed:
  - expanded the existing deterministic demo seed in `D:\Projects\dms-migration\tessara\crates\tessara-api\src\demo.rs`
  - integrated automatic demo seeding into `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1`
  - added `D:\Projects\dms-migration\tessara\scripts\seed-demo-data.ps1` for manual reseeding against a running Compose stack
- Seed shape:
  - `Partner -> Program -> Activity -> Session`
  - `2` partners, `4` programs, `6` activities, `8` sessions
  - metadata coverage across all supported field types: `text`, `number`, `boolean`, `date`, `single_choice`, `multi_choice`
  - one published form family per hierarchy level
  - `2` submitted responses and `1` draft response per form family
  - `4` reports and `1` compact dashboard for UAT navigation and review
- Launch behavior:
  - normal `.\scripts\local-launch.ps1` now preserves existing local data and ensures the demo dataset exists
  - `.\scripts\local-launch.ps1 -FreshData` still rebuilds from a clean database volume, then reseeds the demo dataset
  - no build-time or container-startup auto-seeding was added
- Supporting changes:
  - updated `D:\Projects\dms-migration\tessara\README.md` to document the new seeding flow
  - updated `D:\Projects\dms-migration\tessara\scripts\smoke.ps1` and `D:\Projects\dms-migration\tessara\crates\tessara-api\tests\demo_flow.rs` for the richer seeded dataset
- Validation:
  - `cargo fmt --all --check` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo test -p tessara-web` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `.\scripts\smoke.ps1` passed
  - `.\scripts\local-launch.ps1` passed and left the refreshed stack running

## 2026-04-10 - Organization List Loader Fix
- Fixed a JavaScript syntax error in 	essara-web/src/app_script.rs that prevented product-route loaders from running.
- Rebuilt and relaunched the local Docker stack with scripts/local-launch.ps1 and verified seeded organization data renders again on /app/organization.

## 2026-04-10 - Role-Based Screen Access

- Implemented role-aware access scaffolding across the dedicated application screens:
  - `admin/system`
  - `scoped operator/partner`
  - `respondent/client/parent`
- Backend changes in `D:\Projects\dms-migration\tessara\crates\tessara-api`:
  - added migration `011_role_access.sql` for:
    - account credentials
    - account-to-node scope assignments
    - parent/subordinate respondent relationships
  - extended auth context to expose:
    - `role_family`
    - assigned scope nodes
    - subordinate respondents
  - added recursive effective-scope resolution for operator access
  - split read access from write access for product APIs:
    - hierarchy reads scoped for operators
    - forms readable through `/api/forms`
    - reports and dashboards filtered for operators
    - responses filtered by scoped nodes or respondent context
  - added `/api/responses/options` for role-aware response-start choices
  - made `/api/app/summary` authenticated and role-aware instead of report-admin-only
- Frontend changes in `D:\Projects\dms-migration\tessara\crates\tessara-web`:
  - added `/app/login`
  - removed automatic admin login from the product shell
  - added role-aware navigation hiding and direct-route access guards
  - kept create/edit product screens admin-only for Organization, Forms, Reports, and Dashboards
  - kept Responses as the create/update surface for operators and respondents
  - added respondent context switching in Responses for parent/subordinate flows
- Demo/UAT support:
  - expanded the demo seed to create:
    - operator account
    - parent account
    - respondent account
    - child respondent account
  - assigned operator scope to non-root hierarchy nodes so descendant scoping is exercised
  - seeded subordinate respondent relationships and pending assigned response starts
  - updated local launch output to show all demo credentials
- Validation:
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1` passed
  - `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1` passed

## 2026-04-10 - Roadmap alignment update

- Updated `D:\Projects\dms-migration\tessara\docs\roadmap.md` so it reflects the current implemented state instead of the earlier UI-only snapshot.
- Corrected the roadmap to include already-landed baseline work for:
  - explicit login
  - role-aware navigation and route guards
  - scoped operator access with descendant expansion
  - parent/subordinate respondent context
  - UAT demo seeding through `local-launch.ps1`
- Updated current position so:
  - Sprint 3 is marked active
  - the next concrete milestone is the route/screen-oriented frontend refactor
- Replaced the stale `Immediate Next Sprint` section that still incorrectly said to start with Sprint 1.

## 2026-04-13 - Access Administration And Delegation Closeout

- Closed the access/admin sprint on `D:\Projects\dms-migration\tessara` with the first application-grade admin/auth vertical slice.
- Backend changes:
  - replaced hard-coded `role_family` responses with capability-derived `ui_access_profile`
  - extended `/api/me` and user detail responses to include roles, capabilities, scope nodes, and delegations
  - replaced subordinate-respondent storage and API usage with generic `account_delegations`
  - added `GET /api/admin/users/{account_id}/access`
  - added `POST /api/admin/roles` so admins can create new role bundles, not just edit seeded roles
  - updated response-context access to use delegation resolution through `delegate_account_id`
- Frontend changes:
  - completed inline login failure handling without dropping users into generic error output
  - added current-user summary content in the shell/home
  - upgraded role edit/create to a filterable capability grid and added the dedicated `/app/administration/roles/new` route
  - upgraded user access to a filterable scope/delegation management surface with effective-access summary
  - generalized delegated response context so it is account-based, not tied to a hard-coded respondent-family assumption
- Demo/UAT changes:
  - renamed demo delegation accounts to `delegator@tessara.local` and `delegate@tessara.local`
  - updated `D:\Projects\dms-migration\tessara\scripts\seed-demo-data.ps1`, `D:\Projects\dms-migration\tessara\scripts\smoke.ps1`, and `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1` to the delegation model
- Roadmap update:
- updated `D:\Projects\dms-migration\tessara\docs\roadmap.md`
  - marked Sprint 1A and Sprint 1B complete
  - left Sprint 1C as the next organization-focused slice
- Validation:
  - `cargo fmt --all` passed
  - `cargo test -p tessara-web` passed
  - `cargo test -p tessara-api --test demo_flow` passed
  - `cargo clippy -p tessara-api -p tessara-web --all-targets -- -D warnings` passed
  - `D:\Projects\dms-migration\tessara\scripts\smoke.ps1` passed
  - `D:\Projects\dms-migration\tessara\scripts\local-launch.ps1` passed and left the refreshed stack running


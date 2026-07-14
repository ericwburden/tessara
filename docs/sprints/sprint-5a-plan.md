# Sprint 5A Plan: Dashboard Composition Slice

Status: accepted and closed out on `codex/sprint-5a` on 2026-07-13 after final UI, correctness, organization/reuse, deployment, smoke, Rust, and browser validation.

## Sprint Summary

Sprint 5A turns the existing Dashboard persistence and scoped API foundation into complete application flows. The sprint delivers a native Dashboard directory, detail, create, edit/composition, and product-facing viewer experience over published `ComponentVersion` records.

The sprint delivers:

- dashboard directory, detail, create, edit, and view flows;
- drag/drop/resize placement and removal of published Component versions using the existing Form builder interaction and layout model;
- readable product-facing dashboard viewers backed by the existing Table, Bar, Line, Pie, Donut, and Stat Card Component renderers;
- native SSR Dashboard routes without bridge-owned state or legacy report/chart dependencies;
- scoped Dashboard and Component visibility at list, detail, composition, and viewer boundaries.

Kickoff defaults:

- Branch: `codex/sprint-5a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-5a`
- Plan artifact: `docs/sprints/sprint-5a-plan.md`

Design approval gate:

- Status: satisfied on 2026-07-12.
- Approved reference: `docs/mockups/sprint-5a-dashboard-symbolic-builder-approved.png`.
- Decision record: `docs/sprints/sprint-5a-dashboard-editor-design.md`.
- The approved Symbolic Builder direction is the visual and interaction target only for `/dashboards/:dashboard_id/edit`; material editor deviations return to review before implementation continues.
- Dashboard directory, create, detail, and focused-viewer surfaces follow existing native Tessara patterns and are not visually gated by the editor mockup.
- Mockup-only `Status Draft`, configurable grid/row-height, and guide-toggle controls are illustrative rather than Sprint 5A requirements. Dashboards remain mutable without a publication lifecycle, the editor uses a fixed 12-column contract, and reading order is derived rather than independently editable.
- Product-owner implementation authorization was received on 2026-07-12.

## Sprint Specifications

### Product Outcome

Dashboards compose Component versions through application-grade authoring and viewing flows.

### Dashboard And Placement Contracts

- `Dashboard` remains the mutable presentation container. Dashboard metadata and visibility continue to live in `dashboards` and `dashboard_scope_nodes`.
- Each `dashboard_components` row remains a placement pinned by stable `component_version_id`; Dashboard composition must not bind to Dataset revisions, legacy reports, aggregations, or legacy chart records.
- Only published or superseded Component versions are placeable. Draft Component versions remain unavailable to Dashboard readers and composers.
- Stable-ID pinning does not freeze the current published version's payload. Sprint 4A's explicit `Update Existing Version` action may change what an already placed published `component_version_id` renders; publishing a separate newer version creates a new id and must not repin the Dashboard. Superseded versions remain immutable.
- Dashboard composition reuses the Form builder's 12-column geometry, collision, delayed drag preview, deterministic drag reflow, and width/height resize behavior. Pure placement geometry, bounds, overlap, reflow, and row-major ordering move behind framework-free types/helpers in `tessara-core`; Leptos canvas, tile, selection, drag, and resize primitives live in `tessara-web-ui`.
- Forms retain their current add-field and configuration-sheet experience. Sprint 5A shares the proven low-level grid engine and applicable canvas/tile interactions. The Dashboard editor remains canvas-first, with Components and Placement details available through accessible side sheets instead of permanently reducing the canvas to three columns.
- Dashboard placement config uses a typed `DashboardPlacementConfigV1` contract containing optional title override plus `grid_row`, `grid_column`, `grid_width`, and `grid_height`. Dashboard `GridConstraints` fix 12 columns, 240 maximum rows, and at most 240 stored placements. Rows and columns start at 1, width is 1 through 12 and must fit from its column, height starts at 1 and may use every row remaining below the placement's starting row, and `grid_row + grid_height - 1` must not exceed 240. There is no smaller independent placement-height maximum.
- Every integer width/height inside the global bounds remains available when it satisfies the Component kind's minimum. Table placements enforce a `6 x 4` minimum—not a fixed size—and may grow to width 12 and through every remaining Dashboard row. Bar, Line, Pie, Donut, Stat Card, and unknown future kinds retain the global `1 x 1` hard minimum. Non-binding add defaults remain Table `6 x 4`, Bar `6 x 3`, Line `6 x 2`, Pie/Donut `3 x 3`, and Stat Card `3 x 2`; the code-defined policy can raise other kind minimums later without imposing a preset-only catalog.
- Desktop editor/viewer tracks use one non-user-configurable responsive rule owned by the shared placement UI: measure the rendered grid track from the DOM, target square column/row cells, and clamp row height between 48px and 80px. Pointer calculations consume the measured track instead of a hard-coded pixel assumption. Narrow screens ignore desktop row height and stack by derived reading order.
- New saves write `DashboardPlacementConfigV1` with `schema_version = 1`. Pre-V1 empty/title-only/arbitrary JSON ignores untyped geometry, preserves a string title when present, and uses deterministic single-column fallback for at most 240 placements: sort fallback rows by signed `(position, placement_id)`, assign the earliest available full-width rows from 1 while skipping every row occupied by valid V1 geometry, and use column 1, width 12, and height 1. Duplicate or negative legacy positions are therefore safe, and one malformed or future-schema row cannot make an otherwise valid mixed-version Dashboard unloadable merely because valid geometry occupies its ordinal fallback row. Legacy rows whose fallback satisfies their kind minimum normalize to V1 on the next successful composition save; a legacy or undersized V1 Table uses the counted, non-executing repair state until a manager supplies at least `6 x 4` geometry. Seeded placements use explicit, non-colliding V1 geometry and stay within the 240-placement/row contract.
- A tagged V1 config with missing, wrongly typed, or invalid geometry uses the same fallback footprint with `config_state = "needs_repair"` and does not execute its Component until an authorized manager supplies valid geometry. An unknown future `schema_version` uses the same geometry only as a non-persisted display footprint, retains its raw JSON unchanged, and remains an opaque unavailable placement that may be retained or removed but is never silently downgraded.
- Sprint 5A provides no overflow repair list and never creates or saves a 241st placement. Demo/seed data is repacked or removed as necessary to satisfy the cap. Before the Sprint 5A application runs against an existing database, a deployment preflight reports every over-cap Dashboard id and count and aborts the upgrade/startup without mutating or truncating user data. `docs/sprints/sprint-5a-dashboard-capacity-runbook.md` supplies the operator cleanup/query procedure; the preflight must pass before any Sprint 5A list, detail, editor, or viewer route serves that database.
- `position` is server-derived zero-based row-major order and is response-only for composition saves. The editor displays one-based reading-order labels; reading order is read-only and changes only when geometry changes.
- Desktop editors can drag, drop, and resize Component placements directly on the Dashboard canvas. Keyboard-accessible move/size controls and explicit placement settings provide equivalent non-pointer operation. Narrow screens render placements in deterministic reading order and do not require precision canvas gestures.
- In the Dashboard editor, a valid move by pointer drag, keyboard control, or direct row/column edit into occupied geometry deterministically reflows affected placements using the shared Form algorithm. Pointer, keyboard, and direct width/height changes reject overlaps or out-of-grid geometry and preserve the last valid size. All Dashboard interaction modes produce the same canonical server layout; the Form adapter retains its existing direct absolute-edit collision rejection.
- `GET /api/admin/dashboards/{dashboard_id}/composition` requires `dashboards:manage` and returns the editor's complete placement set without requiring `dashboards:read`. Available rows include editable binding/title metadata; redacted rows expose only opaque placement id, geometry, position, availability/config state, and allowed operations.
- `PUT /api/admin/dashboards/{dashboard_id}/composition` accepts one typed full-layout reconcile command with three explicit command shapes: retain/move/resize an existing row by `placement_id` while the server preserves its binding; add/replace an available binding with `component_version_id`; and remove by `placement_id`. Redacted rows with valid or fallback geometry may be retained, moved, resized, or removed with `dashboards:manage`, but cannot be retitled, previewed, or rebound without `components:read`; unknown future-schema rows allow retain/remove only.
- Every existing placement must appear exactly once as retain/change/remove. Omission, duplication, or a changed placement-id membership set returns `dashboard_composition_stale` (409) and changes nothing, preventing a concurrent add/remove from being overwritten accidentally.
- Existing placement ids are required for retained/changed/removed rows, must be unique in the request, and must belong to the target Dashboard. New-placement commands carry a client correlation key; the canonical response maps each key to its generated `placement_id` and rejects duplicate, foreign, or stale ids without partial writes.
- Movement interactions resolve deterministic reflow through the shared grid engine before save and include every affected row's final geometry. The reconcile endpoint validates a complete collision-free result and never guesses drag intent or relies on command ordering to perform reflow.
- The composition service authenticates and authorizes the Dashboard before validating candidate Component versions, locks the Dashboard in one transaction, validates the complete resulting layout and every scope relationship, derives `position`, reconciles commands, and returns the canonical composition. One invalid command rolls back the entire save.
- Reconcile updates retained placement rows in place, inserts only explicit additions, and deletes only explicit removals. It never implements a save as delete-all/reinsert, so opaque `dashboard_components.id` values remain stable for stale detection, redacted commands, and subsequent edits.
- Sprint 5A uses transactional row locking and structural stale detection without an explicit edit token. When the placement-id set still matches, concurrent edits to the same retained row are last-write-wins; concurrent add/remove membership changes produce `dashboard_composition_stale` (409). A Dashboard publication/version lifecycle remains out of scope.
- Dashboard creation, metadata updates, scope changes, and composition saves use one lock protocol: lock the Dashboard row first, then load/lock placement and candidate-version state in a fixed order before authorization-dependent validation. A scope update that would make any existing placement incompatible is rejected without changing metadata, scope, or composition; scope edits never remove placements implicitly.
- The editor tracks dirty layout state. `Preview dashboard` opens only the last successfully saved composition and is disabled with clear feedback while unsaved or failed changes remain; `Preview selected` may render the current selected exact version without saving the complete layout.
- Viewer layout uses the same persisted geometry contract as the editor rather than a separate approximation.

### Application Routes

- `/dashboards` is the role-aware directory for Dashboards visible to the current user.
- `/dashboards/new` creates Dashboard metadata and visibility before composition begins.
- `/dashboards/:dashboard_id` provides an application detail/overview surface with metadata, visibility, placement summary, and appropriate view/edit actions.
- `/dashboards/:dashboard_id/edit` is the internal composition surface for authorized Dashboard managers and owns Dashboard metadata/scope editing through a header-level settings action separate from the selected-placement inspector.
- `/dashboards/:dashboard_id/view` is the focused product-facing Dashboard viewer.
- Root `tessara-web` retains route adapters, `AppShell`, session and navigation policy, document integration, hydration, CSS, and cargo-leptos ownership.

### Component Rendering

- Dashboard viewers reuse the Component execution and rendering contracts delivered in Sprints 4A and 4B. Dashboard code must not fork separate Table or visual transformation logic.
- An available placement resolves the exact `component_version_id`, `component_slug`, `component_type`, version number, version label, and version status returned by the Dashboard API and loads the corresponding explicit-version Component endpoint.
- Table, Bar, Line, Pie, Donut, and Stat Card placements render with stable loading, empty, error, and unavailable states. Embedded Table placements show the full explicit-version paged Table viewer and its normal viewer affordances inside the tile; pagination bounds rendered rows without replacing the Table with a truncated summary. Table placement headers replace the geometry count with `View fullscreen`; the fullscreen dialog reuses the same server-backed query state so search, sort, filters, visible columns, page size, and cursor history do not reset.
- A failure in one placement must remain local to that card and must not prevent the rest of the Dashboard from rendering.
- `tessara-web-components` exposes a narrow embedded exact-version viewer facade that accepts Component slug/id, `component_version_id`, and Component kind and owns explicit-version fetch, render, empty, error, isolation, and teardown state. Its Table path is one controlled, server-backed renderer shared by the standalone Component viewer and Dashboard facade; it consumes explicit-version pagination/cursor state and keeps search, sort, filters, visible columns, page size, and next/previous navigation connected to the API rather than paginating only the currently fetched rows. `tessara-web-ui` remains presentation-only and does not own Component API query state. Dashboard code owns placement cards, redaction, titles, and layout; renderer logic is not copied.
- Dashboard detail and viewer responses contain one placement envelope for every stored `dashboard_components` row. When the caller cannot read the Component/Dataset, or the stored row is not currently placeable, the envelope preserves only placement id, geometry, position, and `availability = "unavailable"`; it omits title override, Component name/slug/type, version metadata, and Dataset binding. The UI renders a generic redacted placeholder in the saved grid position.
- Directory and detail counts represent the total number of distinct stored placements, including redacted/unavailable placements. Placement existence and total count are Dashboard metadata intentionally visible to a Dashboard reader; hidden Component, version, and Dataset metadata remain confidential.
- The composition editor is metadata-only by default. Placement tiles show Component-kind iconography, title, exact pinned version, reading order, size, and selection/drag/resize affordances; they do not execute or render live Table/chart/stat payloads while arranging the grid.
- `Preview selected` explicitly and lazily mounts at most one real Component preview for the selected placement. `Preview dashboard` opens the focused viewer and only then executes the visible Dashboard placements. Closing either preview releases its rendering resources.
- Editor initial load, drag, resize, selection, and save must not issue Component execution requests. This performance boundary prevents a dense mixed-Component layout from multiplying chart/table execution cost during composition.
- The focused viewer supports the full 240-placement storage contract without eagerly mounting 240 executions. It reserves every saved or redacted footprint, mounts available Component renderers on viewport approach, and tears down or suspends off-screen work without losing each Table placement's current page state. A named code-level `DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS` constant, lower than the storage cap and not exposed as a Dashboard grid setting, bounds simultaneous execution requests; implementation records its default from the near-cap browser probe.
- Directory, detail, and viewer metadata/layout—including redacted placeholders—must be useful in server-rendered HTML when WASM hydration is unavailable. Available Component payload execution may hydrate independently without leaving an infinite loading-only page.

### Authorization And Scope

- Dashboard directory/detail/viewer access requires `dashboards:read`; Dashboard create/delete, editor load, metadata/scope changes, and composition saves require `dashboards:manage`. Editor loading uses the manage-authorized admin composition endpoint and does not also require `dashboards:read`.
- Available placement metadata and execution additionally require `components:read` over the referenced ComponentVersion's bound Dataset scope. A user with Dashboard read access but without matching Component read access receives the redacted placeholder rather than Component metadata or execution.
- Composition picker and add/replace actions require `dashboards:manage` plus `components:read` for the candidate version. They do not require `components:manage`. The manager's Dashboard capability scope and proposed Dashboard visibility must fully contain every bound Dataset visibility node implicated by the placement.
- Scoped read visibility remains overlap-based for Dashboard metadata. Scoped Dashboard authoring remains containment-based for every Dashboard visibility node and every bound Dataset visibility node implicated by a placement.
- Directory and detail counts use `COUNT(DISTINCT dashboard_components.id)` semantics and return total placements without multiplying rows when multiple Dashboard scope nodes overlap the caller's capability boundary.
- Dashboard/placement authorization occurs before candidate-version validation so guessed ids cannot reveal draft, missing, status, name, version, or Dataset details. Direct out-of-scope Component loads and mutations retain non-leaking failures.
- A visible Dashboard is not permission to bypass hidden Component or Dataset execution boundaries. Upstream role/scope changes or an intentional in-place published-version update may turn an existing placement into a redacted/unavailable placeholder without changing its pinned id.

### Backend Boundaries

- New Dashboard behavior must move the current monolithic API module toward explicit router, handler, service, repository, and DTO boundaries.
- Validation and authorization decisions belong in service-level seams; SQL persistence and loading belong in repository seams; handlers remain transport-focused.
- The existing `tessara-dashboards` domain crate contains typed target-model Dashboard composition/config policy and pure `CompositionError` values for geometry, overlap, config decoding, and deterministic layout policy. Its unused legacy `ChartType` vocabulary and tests are removed rather than carried as a quarantine path.
- API-local `DashboardServiceError` values own authorization, missing/foreign placement, unavailable version, repository, and scope-compatibility failures. They map at minimum to `dashboard_layout_invalid_geometry` (400), `dashboard_layout_overlap` (400), `dashboard_placement_limit_exceeded` (400), `dashboard_component_version_unavailable` (400), `dashboard_scope_incompatible` (409), `dashboard_composition_stale` (409), `dashboard_placement_not_found` (404), and the existing non-leaking authorization code (403), without exposing SQL or internal errors.
- The existing per-placement `POST /api/admin/dashboards/{dashboard_id}/components` and `PUT`/`DELETE /api/admin/dashboard-components/{placement_id}` routes are removed from the public router in Sprint 5A. Tests and setup code move to the typed composition endpoint or repository fixtures; no public compatibility path may bypass full-layout validation, capability checks, atomicity, or server-derived order.

### Project And Frontend Ownership

- Before moving Dashboard implementation out of root `tessara-web`, Sprint 5A records the roadmap-required focused extraction proposal with source-size/churn inventory, dependency review, public facade, route-adapter plan, watch/build/browser validation, release bundle comparison, and rollback path.
- The target boundary is `tessara-web-dashboards`, owning Dashboard web DTOs, loaders/actions, directory/detail/create/editor/viewer content, and Dashboard-specific adapters. Root `tessara-web` retains route registration/parameters, `AppShell`, session/navigation policy, document integration, hydration entrypoints, CSS/assets, and cargo-leptos ownership.
- The proposal is a recorded decision gate. Approval scaffolds `tessara-web-dashboards`; rejection keeps Sprint 5A content in the named root `features::dashboards` boundary with measured reasons, rollback notes, and a later reevaluation trigger.
- The proposal must choose either a smaller leaf embedded-viewer boundary or viewer/authoring Cargo feature partitioning before permitting a one-way `tessara-web-dashboards -> tessara-web-components` dependency. It must account for the current transitive `tessara-web-data-ops` authoring dependency. Selecting the sibling dependency requires an explicit architecture amendment plus matching boundary-audit rule; the proposal cannot silently override the current prohibition.
- Framework-free grid semantics use generic identity (`GridPlacement<Id>`) or a separate `GridRect` in `tessara-core`; Dashboard domain composition policy lives in `tessara-dashboards`; shared Leptos placement interactions live in `tessara-web-ui`. API DTOs and web DTOs remain separate from these domain concepts.
- Authenticated SSR uses a request-scoped bootstrap rather than a server-side HTTP loop. Dashboard HTML handlers receive `AppState`, route params, and session headers; authenticate once; call the same Dashboard service used by JSON handlers; adapt the result into web-owned `ShellSessionBootstrap` and `DashboardRouteBootstrap` DTOs; and pass them to a root `application_html_with_bootstrap` render context plus serialized hydration state. Feature crates do not depend on `tessara-api`, and the serialized bootstrap applies the same redaction contract as JSON responses.

### Application UI

- The Dashboard directory is searchable and shows name, description, visibility context, total placement count, and view/edit actions appropriate to capability.
- Create and edit forms provide clear Dashboard metadata and organization-scope controls using existing native UI patterns.
- The composition editor provides a persistent scoped published-Component selector on the left, a Form-builder-derived 12-column canvas of lightweight iconographic placement tiles in the center, and a persistent selected-placement detail panel on the right. It includes drag handles, width/height resize handles, visible drop targets, title override editing, direct placement settings, removal, exact-version links, `Preview selected`, and `Preview dashboard`.
- The viewer presents Dashboard metadata and responsive placement cards using the saved grid geometry on desktop and deterministic reading order on narrow screens, with readable titles and useful empty/error states.
- Redacted/unavailable placements retain their saved footprint and reading order, render generic copy with no hidden metadata, and never issue Component execution requests.
- All touched Dashboard routes remain native SSR-owned, hydrate cleanly, remain browser-console clean, and request no `/bridge/*` assets.

## Acceptance Criteria

- A tester can browse visible Dashboards through `/dashboards` and search the directory.
- Dashboard list counts equal the total number of distinct stored placements, including unavailable/redacted placements, and never multiply when more than one Dashboard visibility node overlaps the caller's scope.
- A user without `dashboards:read` cannot list or direct-load Dashboard content.
- An authorized manager can create a Dashboard with a name, optional description, and at least one visibility node.
- A user with `dashboards:manage` but without `dashboards:read` can load the manage-authorized composition endpoint and edit an in-scope Dashboard without gaining reader-directory access.
- An authorized manager can edit Dashboard metadata and visibility from the composition route within their effective scope.
- A scope reduction that would make an existing placement incompatible is rejected atomically and leaves Dashboard metadata, scope, and composition unchanged.
- An authorized manager can add a visible published or superseded Component version to a Dashboard.
- Dashboard placement requires `dashboards:manage` and `components:read` for the candidate, but not `components:manage`.
- Draft Component versions are absent from composition choices and rejected by direct mutation requests.
- Existing title-only/empty configs load into deterministic non-overlapping fallback geometry, and the seeded Demo Operations Dashboard uses explicit typed V1 geometry for every placement. Its content-heavy Session Log Table occupies `12 x 6`; simpler seeded Tables remain `6 x 4`, proving the minimum is not a mandatory fixed size.
- A Dashboard can store at most 240 placements, every placement fits within rows 1 through 240, and a 241st placement is rejected with `dashboard_placement_limit_exceeded` without changing the saved composition.
- Every globally valid width 1 through 12 and height from the configured per-Component-kind minimum through the rows remaining below the placement's starting row remains selectable.
- An authorized manager can drag a placement into occupied geometry and receive deterministic reflow, resize it by width and height, change its title, replace its pinned version explicitly, and remove it.
- Invalid pointer, keyboard, or direct-size targets never create overlaps or out-of-grid geometry and leave the last valid size intact.
- A pointer, keyboard, or direct move whose deterministic reflow would cross row 240 rejects the whole movement and retains the last valid complete layout.
- Keyboard/direct controls can move and resize placements without pointer input.
- Pointer and direct controls produce the same canonical geometry and server-derived reading order; reading order is not independently editable.
- One full-layout save atomically reconciles added, changed, and removed placements. If any placement is invalid, no stored placement changes.
- Retained placements keep the same opaque `placement_id` across successful saves; a reconcile never replaces unchanged rows with newly generated ids.
- Saving unrelated available changes retains every redacted placement and its hidden binding. A Dashboard manager can move, resize, or explicitly remove a redacted placement by opaque placement id without receiving its binding or title.
- Opening the editor does not execute any placed Component. Selecting, dragging, resizing, or saving placements does not trigger Component execution.
- `Preview selected` executes only the selected placement on demand. `Preview dashboard` is disabled while the editor is dirty, renders the last saved composition, and leaves no preview resources mounted after exit.
- A tester can inspect Dashboard detail separately from the focused viewer.
- A tester can open `/dashboards/:dashboard_id/view` and render Table, Bar, Line, Pie, Donut, and Stat Card placements from their exact Component versions.
- An embedded Table placement exposes the complete paged Table experience; a tester can move between pages and reach all rows without the tile rendering an unbounded grid, then open `View fullscreen` and continue with the same Table controls and page state.
- A near-cap viewer reserves all saved footprints, remains responsive, and keeps execution concurrency bounded by lazily mounting available placements near the viewport.
- Placement tiles and available viewer cards expose a human-readable exact version number/label/status as well as the pinned id.
- Explicitly updating the current published Component version in place changes what a Dashboard renders without changing its pinned `component_version_id`.
- Publishing a separate newer Component version creates a new id and does not move an existing Dashboard placement until an authorized Dashboard editor replaces it.
- A failed or unavailable placement displays a local stable state while other placements continue to render.
- A caller who can read the Dashboard but cannot read a placed Component receives a generic placeholder that preserves placement geometry and exposes no title, Component, version, or Dataset metadata; the placement still contributes to total counts.
- Reader-only users do not see composition controls or draft Component metadata.
- Scoped operators cannot list, load, place, or execute Dashboards and Components outside their effective scope.
- Dashboard APIs and UI use `ComponentVersion` identifiers and do not introduce Report, Aggregation, Chart, workbench, or bridge-owned product behavior.
- Directory, detail, and viewer metadata/layout are useful in server-rendered HTML without WASM; touched Dashboard routes remain hydration-clean, browser-console-clean, and free of `/bridge/*` requests.

## Manual Test Plan

Admin happy path:

1. Sign in as `admin@tessara.local`.
2. Fresh-seed the environment, open `/dashboards`, and verify the Demo Operations Dashboard appears with the expected total placement count and explicit non-colliding geometry.
3. Create a Dashboard with a name, description, and visible organization scope.
4. Open its detail and edit routes.
5. Add published Table, Bar, Line, Pie, Donut, and Stat Card Component versions.
6. Set placement titles, drag one placement into occupied geometry and confirm deterministic reflow, resize in both dimensions, and remove/re-add one placement.
7. Save the full layout once, reload, and confirm canonical geometry and reading order are preserved.
8. Open the focused viewer and confirm every available placement renders in the saved order; page through the Table placement and confirm the complete Table viewer remains bounded inside its tile.
9. Update one current published Component version in place and confirm the Dashboard output changes while its `component_version_id` stays the same.
10. Publish a separate newer version and confirm the Dashboard stays on the old id until explicitly replaced.
11. Return to the editor and confirm placement tiles remain iconographic and responsive rather than rendering every live Component.

Validation and empty paths:

1. Attempt to create a Dashboard without a name or visibility node and confirm clear validation feedback.
2. Create an empty Dashboard and confirm detail, edit, and viewer routes show useful empty states.
3. Attempt to place a draft Component version through the API and confirm a stable rejection.
4. Force one placement to return an execution error and confirm the remaining cards still render.
5. Submit a full composition containing one invalid placement and confirm the complete stored layout rolls back.
6. Attempt an overlapping or out-of-grid pointer/keyboard/direct size change and confirm the last valid size is preserved with clear feedback.
7. Use the direct/keyboard placement controls and confirm they produce the same saved geometry and reading order as pointer interactions.
8. Make an unsaved change and confirm `Preview dashboard` is disabled until the layout saves successfully.
9. Open `Preview selected` and confirm only the selected Component renders; close it and confirm the preview is disposed.
10. Load a legacy title-only placement config and confirm deterministic fallback geometry is readable and normalizes on save.
11. Attempt to save a 241st placement and confirm `dashboard_placement_limit_exceeded`; separately submit geometry beyond row 240 and confirm `dashboard_layout_invalid_geometry`. Neither failure changes the stored composition.
12. Move a placement so deterministic reflow would cross row 240 and confirm the entire movement is rejected with the prior layout intact.
13. Exercise the deployment preflight against an over-cap fixture and confirm it names the Dashboard id/count, aborts without mutation, and points to the operator cleanup procedure.

Scoped and reader paths:

1. Sign in as a scoped Dashboard manager and confirm only overlapping Dashboards and compatible Components appear.
2. Create and edit an in-scope Dashboard.
3. Attempt to create or update a Dashboard with an out-of-scope visibility node and confirm it is forbidden.
4. Attempt a scope reduction that is incompatible with an existing placement and confirm the entire metadata/scope update rolls back.
5. Attempt to place an out-of-scope Component version and confirm no hidden metadata is exposed.
6. Sign in with `dashboards:read` but without matching `components:read`; confirm total counts remain unchanged and the hidden placement becomes a geometry-preserving generic placeholder.
7. Sign in with `dashboards:manage` but without `dashboards:read`; confirm direct editor load works while the reader directory stays unavailable.
8. As a Dashboard manager without matching Component read access, edit an available placement, save, and confirm the redacted row and hidden binding are retained; then move and explicitly remove the redacted row by opaque id without exposing metadata.
9. Sign in with `dashboards:manage` and `components:read` but without `components:manage`; confirm compatible published versions can be placed.
10. Sign in as a no-access user and confirm Dashboard list and direct routes are denied.

Native route checks:

1. Direct-load every Dashboard route and refresh it.
2. Confirm shell ownership, navigation state, and session redirects remain correct.
3. Confirm no hydration warnings, browser-console errors, or `/bridge/*` requests occur.
4. Check desktop and mobile layouts for readable placement cards without horizontal overflow.
5. Disable WASM/JavaScript and confirm directory, detail, saved geometry, titles for available placements, and generic unavailable placeholders remain useful in server-rendered HTML.

## Automated Test Plan

Planned verification commands:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo audit --quiet`
- `.\scripts\check-web-crate-boundaries.ps1`
- `.\scripts\validate.ps1`
- `.\scripts\local-launch.ps1 -FreshData`
- `npm --prefix .\end2end test`
- `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
- `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8080" -KeepServices`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- `git diff --check`

Backend/API scenarios:

- List and detail responses include only Dashboards visible to the caller.
- Dashboard counts use all distinct stored placements—including redacted/unavailable rows—and do not multiply across matching scope-node joins.
- Create/update rejects blank names, missing visibility, nonexistent nodes, out-of-scope visibility, and scope reductions incompatible with existing placements; failed operations leave all state unchanged.
- Concurrent composition and metadata/scope transactions lock the same Dashboard row first and cannot jointly commit an incompatible composition/scope state.
- Concurrent composition saves with an unchanged placement-id set use documented row-level last-write-wins behavior; a concurrent add/remove causes the stale full-layout save to fail with `dashboard_composition_stale` and no partial writes.
- Placement creation/update rejects missing, draft, and out-of-scope Component versions.
- Full composition reconciliation preserves stable `component_version_id` bindings, derives canonical zero-based positions, and rolls back all changes when any placement fails validation.
- Reconcile tests prove retained opaque placement ids survive unchanged, additions alone receive new ids, removals alone disappear, and the implementation never delete-all/reinserts the layout.
- Reconcile tests reject duplicate, foreign-Dashboard, missing, and stale placement ids and prove new-placement correlation keys map deterministically to generated ids in the canonical response.
- Shared `tessara-core` layout tests cover bounds, overlap, row-major order, deterministic pointer/keyboard/direct movement reflow, rejection when reflow would cross row 240, size-change rejection, and configured constraints. `tessara-web-forms` characterization tests prove existing Form add/config behavior remains intact.
- Legacy empty/title-only config tests prove position-derived fallback and V1 normalization; seeded placement tests prove explicit typed geometry for all Component kinds after the seed version, demo-flow expectations, smoke assertions, and UAT constants are updated together.
- Legacy tests cover negative/duplicate positions, arbitrary pre-V1 JSON, partially formed or wrongly typed V1 config, invalid V1 bounds, and unknown future schema versions without panics, overlaps, silent downgrade, or unintended execution.
- Capacity tests cover exactly 240 stored placements, rejection of the 241st placement, rejection beyond row 240, and a deployment preflight that aborts without mutation for over-cap Dashboards, overlapping valid V1 geometry, or mixed layouts whose occupied V1 rows leave too few full-width fallback rows. The preflight identifies every affected Dashboard, points to the documented operator cleanup procedure, and seed cleanup/repacking remains within the cap.
- Sizing-policy tests prove Table rejects either dimension below `6 x 4`, accepts heights above 6 through the remaining 240-row grid boundary, and prove all configured per-kind minimums are enforced consistently by client and server.
- Current-published update-in-place tests prove Dashboard output may change under the same pinned id; new-version publish tests prove no automatic repin.
- Split-capability tests cover Dashboard read without Component read, Dashboard manage without Dashboard read, Dashboard manage plus Component read without Component manage, an editor save that retains/moves/removes redacted rows by opaque id, and guessed candidate ids. Authorization occurs before candidate validation.
- Hidden or non-placeable stored rows return redacted placement envelopes and remain included in total counts without exposing title, Component, version, or Dataset metadata.
- Stable domain error codes and 400/403/404/409 mappings cover invalid composition, unauthorized scope, missing assets, and incompatible scope changes.
- Removed per-placement mutation routes cannot bypass typed geometry, atomic reconciliation, capability checks, or server-derived order.
- Dashboard composition continues to use `dashboard_components.component_version_id` and never legacy analytical asset identifiers.

Frontend/E2E scenarios:

- `/dashboards` renders populated, searched, empty, loading, and error states.
- `/dashboards/new` creates a scoped Dashboard through the native form.
- `/dashboards/:dashboard_id` renders metadata, visibility, placement summary, and capability-aware actions.
- `/dashboards/:dashboard_id/edit` updates Dashboard metadata/scope, adds/titles/drags/reflows/resizes/directly repositions/replaces/removes placements, and persists the complete composition with one save.
- Dashboard editor network coverage proves initial load and layout manipulation request metadata only and make zero Component execution calls.
- `Preview selected` mounts only the selected exact Component/version renderer and makes no execution requests for unselected placements. A selected Table may issue subsequent explicit-version requests as its page, search, sort, filter, visible-column, or page-size state changes; closing it tears down the renderer before another selection renders.
- Dirty-state coverage proves `Preview dashboard` cannot show a stale/unsaved editor composition.
- `/dashboards/:dashboard_id/view` renders every supported Component kind from explicit-version endpoints.
- Embedded Table E2E coverage pages through the full Table result, proves each page/search/sort/filter/column/page-size change reaches the explicit-version API rather than only the currently loaded rows, and retains normal viewer affordances without unbounded tile growth.
- A near-cap browser fixture proves all footprints render, only near-viewport available placements execute, observed request concurrency never exceeds `DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS`, scrolling remains responsive, and off-screen teardown does not lose an embedded Table's page state.
- Desktop viewers reproduce saved placement geometry; narrow viewers stack the same placements in deterministic reading order.
- Pointer drag, keyboard movement, and direct row/column movement produce the same canonical reflow; pointer/keyboard/direct invalid size changes and moves whose reflow would cross row 240 are rejected and remain stable across save/reload.
- Dashboard browser coverage proves an update-in-place changes output under the same pinned id, while publishing a separate version leaves the placement unchanged until an explicit editor replacement.
- Focused editor/adaptor coverage proves every globally valid integer span is offered and the finalized per-kind minimum policy is enforced consistently in direct controls, pointer sizing, and server responses.
- One failed placement remains isolated while sibling placements render.
- Redacted placements preserve geometry, issue zero execution requests, keep total counts unchanged, and expose no hidden metadata.
- Selector search/kind filters, live-region validity/save announcements, focus after removal, and focus restoration after preview close are covered.
- Reader-only users do not see manage actions; split-capability, scoped, and no-access users receive the expected positive and negative behavior.
- Direct loads and refreshes include useful SSR metadata/layout, remain hydration-clean and browser-console-clean, and are free of `/bridge/*` requests.
- SSR bootstrap coverage proves authenticated route data and capability-aware actions are present before hydration, hidden fields never appear in HTML/bootstrap JSON, and hydration reuses the bootstrap without a mismatched duplicate initial load.
- `docs/playwright-permissions-scenarios.md` is updated for Dashboard directory, composition, viewer, exact-version pinning, and scoped negative cases.

## Ordered Implementation Plan

1. Completed 2026-07-12: produced and reviewed product mockup directions, approved the Symbolic Builder target, and recorded the decision in `docs/sprints/sprint-5a-dashboard-editor-design.md`.
2. Add black-box characterization coverage for current Dashboard visibility/draft filtering and current Form drag/reflow/resize behavior, plus focused contract regressions that reproduce the scoped count multiplication, authorization-order, and hidden-placement gaps before restructuring either feature.
3. Produce and record the roadmap-required Dashboard frontend extraction decision, including the approved/rejected ownership branches, SSR bootstrap design, embedded-viewer dependency/feature partition, boundary-audit rules, release bundle comparison, and rollback path. Implement the request-scoped authenticated render/bootstrap context before the first Dashboard content slice.
4. Correct the list service/query/DTO to return total distinct stored-placement counts, then replace the `/dashboards` placeholder with the first native vertical slice: a capability-aware SSR-bootstrapped directory backed by `GET /api/dashboards`, including loading, error, empty, search, and action states.
5. Extract framework-free generic `GridPlacement<Id>`/`GridRect` and `GridConstraints`, bounds, overlap, deterministic movement reflow, sizing validation, and row-major order into `tessara-core`; move only reusable Leptos canvas/tile/selection/drag/resize primitives into `tessara-web-ui`. Adapt Forms with characterization coverage while preserving its current add-field and configuration-sheet UX.
6. Replace the legacy `tessara-dashboards::ChartType` posture with typed Dashboard placement/config/composition policy and domain errors. Define the API/web DTO adapters, stable-id pinning semantics, redacted placement envelope, total-count semantics, version display metadata, V1 legacy fallback, global bounds plus per-kind minimum sizing policy, 240-placement/row limits, zero-based derived positions, and seeded typed geometry.
7. Decompose the Dashboard API module into router, authenticated transport handlers, service, repository, and DTO seams around the new contracts. Add the manage-authorized composition loader, tagged transactional reconcile command that updates retained rows in place, shared row-lock protocol, scope-contraction validation, domain/service error mapping, and scope-compatible published-history ComponentVersion picker; remove the public per-placement mutation routes.
8. Implement native Dashboard create and detail surfaces plus header-level metadata/scope editing on the composition route with capability-aware actions and atomic failure behavior.
9. Extract one controlled server-backed Table viewer in `tessara-web-components`, reuse it from the standalone, embedded, and fullscreen exact-version viewer facades, then implement the approved native composition editor using Dashboard-owned Component and Placement-details side sheets plus shared low-level grid/canvas/tile interactions. Cover equivalent pointer/keyboard/direct movement reflow, rejected invalid size changes, opaque redacted-row commands, atomic save/rollback, dirty preview behavior, accessibility, and metadata-only network behavior.
10. Implement the focused Dashboard viewer for Table, Bar, Line, Pie, Donut, and Stat Card placements through the facade, including the full paged embedded Table experience, isolated errors, redacted geometry-preserving placeholders, total counts, viewport-lazy mounting, the named concurrency ceiling and browser-probed default, teardown, responsive order, and useful SSR metadata/layout.
11. Update demo seed version/data and the seed helper for explicit typed geometry, the 240-placement/row cap, and transactional replacement; update hard-coded demo-flow, smoke, and UAT seed-version expectations together, and remove or repack any violating seed fixture. Add the non-mutating existing-database count and display-layout preflight—including overlapping V1 and fallback-row exhaustion checks—and an operator-controlled cleanup procedure with backup/export safeguards in `docs/sprints/sprint-5a-dashboard-capacity-runbook.md`. Make the Dashboard-inclusive UAT harness provision and clean up named scoped-manager, read-only, manage-without-read, split Component-capability, no-access, failing-execution, near-cap, and legacy-config fixtures; update API/unit tests, Dashboard-focused Playwright coverage, smoke/UAT scripts, crate-boundary checks, and permission-scenario documentation for all paths.
12. Run the full Sprint 4B-equivalent verification baseline, fresh local deployment, smoke checks, browser validation, dependency audit, and Dashboard-inclusive Sprint UAT script before closeout.

## Dependencies And Blockers

- Post-implementation UI review sets the shipped Table minimum to `6 x 4` while preserving the full granular range through every row remaining in the 240-row grid; `6 x 4` is not a fixed Table size. Other kinds retain their existing global `1 x 1` minimum and recommended add defaults. A legacy or stored undersized Table is counted and displayed with repair geometry but does not execute until an authorized manager repairs it to at least `6 x 4`; future changes to other kind minimums require the same explicit migration/redaction review.
- Sprint 4A and Sprint 4B ComponentVersion authoring, explicit-version execution endpoints, and renderers are the foundation for Dashboard composition and viewing.
- Stable-ID pinning preserves Sprint 4A's intentional current-published update-in-place behavior: an in-place change may alter Dashboard output under the same id, whereas publishing a separate version never repins a Dashboard automatically.
- The existing Form builder 12-column layout, collision-aware movement reflow, drag preview, and collision-rejecting sizing behavior are the interaction foundation. Sprint 5A preserves Form behavior while extracting only framework-free grid rules and genuinely reusable low-level UI interactions.
- Forms and Dashboards never depend on each other's DTOs or API clients. `tessara-core`, `tessara-dashboards`, and `tessara-web-ui` retain their distinct pure-grid, Dashboard-policy, and Leptos-interaction ownership.
- A new `tessara-web-dashboards` boundary is contingent on the required evidence-driven proposal. Approval and rejection branches both name the Sprint 5A code owner; any one-way embedded viewer dependency on `tessara-web-components` must be explicitly documented, architecture-approved, feature-partitioned as necessary, and enforced by the boundary audit.
- Component execution cost is a design constraint: composition remains iconographic and metadata-only, with real rendering isolated to explicit lazy preview/viewer states.
- The existing Dashboard schema and per-placement CRUD APIs provide a usable starting point but do not satisfy full-layout atomicity. The canonical editor save uses the new transactional composition reconcile command, and old public per-placement write routes are removed rather than retained as bypasses.
- Current title-only/empty placement configs remain readable through deterministic fallback; current seeded data is upgraded to explicit typed V1 geometry, cleaned to the 240-placement/row cap, and verified from a fresh database. There is no overflow repair-list behavior: an existing over-cap database is rejected by a non-destructive deployment preflight until the documented operator cleanup procedure brings it inside the invariant.
- The Dashboard viewer uses one controlled, server-backed complete paged Table renderer rather than a truncated or client-only summary. Global grid spans remain granular within bounds, while the shared constraint policy can enforce recommended defaults and minimum dimensions by Component kind.
- Supporting 240 stored placements includes a usable focused viewer: renderer mounting is viewport-lazy and the named code-level concurrency ceiling is lower than the storage cap rather than issuing every available Component request at route load.
- Total placement counts and generic redacted placeholders intentionally reveal placement existence as Dashboard metadata while keeping hidden title, Component, version, and Dataset metadata confidential.
- The pre-Sprint Dashboard frontend routes were native placeholders, so implementation replaced them without a legacy UI migration.
- Full browser and UAT verification depends on the local database, app stack, Playwright browser dependencies, and seeded Dashboard/Component fixtures being available.
- The design approval gate applies only to the composition editor and is satisfied. Product-owner implementation authorization was received on 2026-07-12. Dashboard publication/version lifecycle and mockup-only status/grid-setting controls remain outside Sprint 5A unless the roadmap is explicitly revised.

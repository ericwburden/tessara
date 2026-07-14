# Sprint 5A Dashboard Design QA

Status: **passed on 2026-07-13 after the approved soft-surface, standard-Table, half-strength shared-gradient, and Dashboard-row action follow-ups**

References:

- `docs/mockups/sprint-5a-dashboard-symbolic-builder-approved.png`
- `docs/mockups/sprint-5a-dashboard-viewer-chrome-approved.png`
- `docs/mockups/sprint-5a-dashboard-soft-surfaces-approved.png`

Implementation: `/dashboards`, `/dashboards/:dashboard_id/edit`, and `/dashboards/:dashboard_id/view`, with Forms and Components directories used as application-level comparison surfaces.

Evidence: `docs/audits/sprint-5a-ui-review-2026-07-13/README.md`

## Visual and interaction result

The live seeded 5A deployment was reviewed in the in-app Browser at 1951 × 806 for the approved viewer comparison, at 1440 × 1000 for the minimum `6 x 4` Table treatment, and at 390 × 844 for the phone treatment. The established 1280 × 720 editor and directory surfaces were also rechecked so this viewer-only change did not remove their large outer route panels.

The editor now follows a canvas-first interpretation of the approved symbolic-builder direction. The 12-column grid and symbolic placement tiles retain the established Tessara dark theme, borders, radii, spacing, typography, and teal accent vocabulary. Components open in a left shared side sheet; Placement details open in a right shared side sheet after selection. The full-width canvas no longer pays a permanent three-pane width penalty, while both secondary tasks remain directly discoverable above it.

The focused viewer retains the established outer route panel by explicit product direction. Within it, charts use the approved subdued blue-to-teal rounded surface without a hard border or shadow, and each placement title is the chart title inside the reclaimed visualization area. Stat Cards have no placement chrome and keep their Component-configured semantic fill and radius. Tables now use the same subdued rounded surface without a Dashboard-specific header. The standard route-free Component Table renderer owns the optional title and `View fullscreen` action in its normal toolbar beside Reset and Columns, while preserving complete server-backed paging and bounded scrolling.

The approved soft-surface reference and deployed capture were compared together at 1951 × 806, followed by a focused reference-versus-Table comparison at the same viewport. Surface color, radius, border removal, intrinsic title placement, Stat Card fill behavior, and toolbar ordering match the chosen direction; saved placement geometry intentionally determines the live composition. The outer route panel is an intentional clarification from the user rather than a visual defect. Evidence: `26-viewer-soft-chart-surfaces.png`, `27-viewer-standard-table-toolbar.png`, `28-viewer-compact-table-toolbar.png`, and `29-viewer-table-fullscreen-toolbar.png` in the audit folder.

The subsequent Browser annotation was clarified after the first implementation: the requested treatment was the existing blue-to-teal gradient at 50% of its original visual strength, not a neutral overlay, and it should be shared by charts and Tables. The full-strength gradient reference, neutral-overlay intermediate state, and corrected implementation were compared at the same 1168 × 912 viewport. The final token preserves the original gradient stops and direction, mixes each resolved stop 50% with transparency, and leaves the containing element and all chart/Table content at opacity `1`. The dark computed stop alphas therefore move from approximately `0.690`/`0.783` to `0.345`/`0.391`, exactly halving the original contribution without fading SVG marks, labels, Table controls, or rows. The same token automatically follows light-theme constituent tokens and is used by all chart placements plus embedded and standard Table surfaces; Stat Cards retain their configured fill. Evidence: `36-chart-opacity-annotation-reference.png`, `40-neutral-overlay-before-gradient-correction.png`, `41-half-gradient-charts-and-tables-dark.png`, `42-half-gradient-charts-and-tables-light.png`, `44-half-gradient-inline-table-dark.png`, and `45-half-gradient-mobile-dark.png` in the audit folder.

Placement height is no longer capped at six rows. A placement may use every remaining row through row 240; per-kind minimums, collision rules, the 240-row grid bound, and the 240-placement storage bound remain enforced. Repair mode can replace a malformed fallback rectangle with valid kind-conforming geometry without weakening ordinary resize validation.

The Dashboard directory now matches the established Forms and Components list hierarchy: shared page header, search, semantic table, row-header link, count disclosure, compact actions, and shared pagination. Desktop actions remain visible without a horizontal-scroll detour. At narrow widths the row becomes a semantic card; name and description stack without one-character wrapping, and every action remains exposed.

The final Dashboard-directory annotation identified a P2 table-geometry mismatch: the desktop Actions `<td>` was itself `display: flex`, so it no longer occupied the full table-row height and appeared as a detached field. The source state and corrected deployment were compared together at the annotated 1262 × 912 light-theme viewport. The cell now remains `display: table-cell`, exactly shares the row's top, bottom, and 67.8-pixel height, and centers a reusable inline action group containing Eye and Pencil icon links. Matching dashboard-specific accessible names and native titles preserve View/Edit meaning without visible text. The action column contracts from 22% to 16% and returns that width to the primary Dashboard column. Typography, copy, theme colors, row borders/radii, and imagery remain unchanged; the icons come from the established application icon library rather than custom assets. Dark-theme contrast and the 390 × 844 card layout were also reviewed: the mobile Actions label and 40-pixel buttons occupy the same grid row, all content stays inside the card, and page overflow remains zero. Evidence: `46-dashboard-actions-annotation-before.png`, `47-dashboard-actions-icon-row-light.png`, `48-dashboard-actions-icon-row-dark.png`, and `49-dashboard-actions-icon-row-mobile.png` in the audit folder.

## Accessibility and reuse result

- Shared side-sheet/modal infrastructure owns Portal placement, Escape handling, Tab trapping, background inertness, body scroll lock, nested-dialog coordination, and opener focus restoration, including conditionally unmounted sheets.
- Dashboard scope, editor sheets, selected preview, and standard Component Table full-screen triggers expose stable dialog relationships and reactive expanded state.
- Available Table placements pass only a route-free title/full-screen presentation policy into the shared renderer. Charts and Stat Cards ignore that policy, while unavailable placements retain generic redacted presentation without Component metadata or controls.
- Noninteractive placement grid guides are hidden from the accessibility tree instead of announcing hundreds of decorative row/column cells.
- Dashboard, Forms, and Workflow surfaces reuse the shared side sheet and search controls; interactive tables compose shared search and pagination primitives.
- Keyboard close behavior and paging-state continuity were exercised in the deployed browser. No full WCAG claim is made without a dedicated screen-reader, contrast, zoom, and high-contrast-mode pass.

## Validation completed

- Affected native/all-feature checks, the root WASM hydration graph, strict Clippy, formatting, Tailwind compilation, and diff checks passed.
- Focused core, Dashboard-domain, and Dashboard-web suites passed 23/23, 24/24, and 25/25; the Component viewer passed 18/18.
- All 12 database-backed Dashboard composition/API tests passed against a disposable isolated database, including the 240-row boundary and full-height Table case.
- The saved-viewer Table/chart/Stat presentation contract now asserts that Tables have no Dashboard placement header, that optional title/full-screen controls live in the standard toolbar, that action order is Reset -> Columns -> Fullscreen, and that fullscreen contains no nested trigger.
- Tailwind 4.2.4 compiled the production stylesheet without unresolved imports.
- The final release image was rebuilt and health-checked with only the Sprint 5A API and Postgres containers running.
- Deployed desktop, minimum `6 x 4`, phone, Table, and full-screen captures plus semantic snapshots were reviewed. Column and header-filter popovers remained within the viewport and passed corner hit-testing; Escape restored focus. Fullscreen preserved page size and rows, exposed no nested trigger, and restored focus to the inline trigger. Browser output contains no runtime errors; the only message is the pre-existing WASM initialization deprecation warning.
- The clarified shared-gradient annotation passed deployed visual and computed-style checks at 1168 × 912 in dark and light themes and at 390 × 844 in dark mode. All four chart kinds and every mounted standard/embedded Table surface use the half-strength gradient while remaining fully opaque; the phone page has no horizontal overflow and Stat Card presentation remains isolated. The intermediate neutral overlay is removed. No P0, P1, or P2 visual defects remain from this iteration.
- The Dashboard-row action correction passed deployed light/dark visual checks, exact desktop row/cell geometry assertions at 1262 × 912, named-link and tooltip checks, and the 390 × 844 card reflow. The Dashboard crate passed 25/25 tests, the eight-scenario Dashboard Playwright file passed discovery and TypeScript transformation, and the updated native-routes scenario passed against the deployed service. No P0, P1, or P2 visual defects remain from this iteration.

## Remaining non-blocking organization decisions

- Sequence migration of the remaining legacy Forms, Workflow, Dataset, and Component hand-rolled sheets/modals.
- Decide whether shared presentation primitives should also own the column selector and header filter popovers while client-backed and server-backed table state remains separate.
- Schedule focused decomposition of the large Dashboard editor and Component viewer modules when that refactor can receive dedicated regression coverage.

final result: passed

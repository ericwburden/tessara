# Sprint 4B Plan: Chart And Stat Component Slice

Kickoff status: started from clean `main` on 2026-07-09 after Sprint 4A was fast-forward merged.

## Sprint Summary

Sprint 4B makes visual presentation assets first-class Components. Sprint 4A established Dataset major-line-backed Table components and the Component versioning model; this sprint extends that model so authors can build and view visual components without using deprecated workbench-style visual analysis flows.

The sprint delivers:

- `Bar`, `Line`, `Pie/Donut`, and `StatCard` component authoring;
- component-specific validation and viewing behavior;
- visual component authoring and viewing built directly on `ComponentVersion` and typed validation state;
- legacy visual-analysis endpoint treatment as adapter-only if touched;
- scoped component and dashboard visibility enforcement for any touched legacy visual-analysis metadata endpoint.

Kickoff defaults:

- Branch: `codex/sprint-4b`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4b`
- Plan artifact: `docs/sprints/sprint-4b-plan.md`

## Sprint Specifications

### Product Outcome

Visual presentation assets are first-class components.

### Settled Sprint Decisions

- Component kind storage stays on the existing `component_versions.component_type` enum and `component_versions.config` JSON contract. Sprint 4B widens the enum from `table` to `table`, `bar`, `line`, `pie`, `donut`, and `stat_card`; it does not add a separate chart/report/visual-analysis asset table.
- Visual components bind to Dataset major lines exactly like Table components: `dataset_id`, `dataset_version_major`, and `binding_mode = major_line`. They do not bind directly to Dataset revisions and do not create component-owned aggregation definitions.
- Dataset major-line outputs are expected to be display-ready for the intended visual. Bar/Line/Pie/Donut components map existing output rows to marks; StatCard maps an existing output row/field to a displayed statistic. Any grouping, calculation, filtering, or aggregation needed to make the data display-ready belongs in the Dataset authoring layer.
- The v1 visual renderer should be native Leptos/SVG/CSS in `tessara-web-components`. Do not introduce a JavaScript chart controller, bridge asset, or workbench-owned rendering path for this sprint.
- Existing `/api/components/{component_ref}/table` and versioned table execution endpoints remain table-only compatibility endpoints for Table viewers and tests. Sprint 4B may add visual execution endpoints or a polymorphic component render endpoint, but whichever path is chosen must use `ComponentVersion` as the only visual source of truth and must preserve the existing table endpoints.
- Pie and Donut share the same validation and data contract, with `component_type` selecting the visual treatment. Both are accepted deliverables; the UI may expose them as a single segmented Pie/Donut kind selector.
- Legacy visual-analysis endpoints should not be touched unless implementation proves they are necessary for compatibility. If touched, the sprint must document the exact endpoint and prove it is adapter-only with scoped component/dashboard visibility enforcement.

### V1 Visual Config Contracts

Visual configs are intentionally presentation-level. All field references below must resolve against the bound Dataset major-line output fields.

- `bar`: `category_field`, `value_field`, optional `label`, optional `value_format`, optional `orientation` of `vertical` or `horizontal`, optional `max_items`.
- `line`: `x_field`, `y_field`, optional `label`, optional `value_format`, optional `max_items`.
- `pie` / `donut`: `category_field`, `value_field`, optional `label`, optional `value_format`, optional `max_slices`.
- `stat_card`: `stat_field`, optional `label`, optional `value_format`, optional `supporting_text_field`, optional `trend_field`.

Validation should reject:

- missing required fields for the selected kind;
- field references outside the bound Dataset major-line contract;
- numeric chart value fields that are not Dataset `number` fields for Bar, Line, Pie, and Donut;
- unsupported enum values such as unknown orientation or format options;
- configs containing stale analytical keys such as `metrics`, `group_by`, `aggregation`, or legacy report/chart identifiers.

### Component Kinds

Sprint 4B adds authoring and viewing support for:

- `bar`
- `line`
- `pie` / `donut`
- `stat_card`

The implementation should keep these visual component kinds on the Component/ComponentVersion path introduced in Sprint 4A. Visual components should not create or depend on a separate report, aggregation, chart, or deprecated visual workbench asset family.

### Component Contracts

- Visual component versions are authored and viewed through Component routes and ComponentVersion storage.
- Component-specific configs should be validated through typed validation state before save/publish.
- Visual component configs should bind to Dataset major-line output fields and use those fields as the source for chart/category/value/stat configuration.
- Viewer behavior should render the selected visual component kind directly from its published ComponentVersion.
- The current published version remains the default viewer target; published-history behavior should stay consistent with Sprint 4A where explicit version viewing is supported.

### Legacy Endpoint Constraint

Any retained legacy visual-analysis endpoint touched during this sprint must remain explicitly adapter-only. If such an endpoint returns metadata, it must enforce scoped component and dashboard visibility before returning that metadata.

### Frontend Boundaries

`crates/tessara-web-components` should continue to own Component authoring and viewer content. Root `tessara-web` should continue to own route adapters, app shell integration, session guard integration, document metadata, hydration, CSS/assets, and cargo-leptos ownership.

### Application UI

The application must provide visual component builder and viewer screens for the delivered visual component kinds. The UI should preserve the Sprint 4A Dataset picker/context and versioning workflow where relevant, while adding kind-specific controls and preview/view states for charts and stat cards.

## Acceptance Criteria

- A tester can create a Bar component through the application UI.
- A tester can create a Line component through the application UI.
- A tester can create a Pie or Donut component through the application UI.
- A tester can create a StatCard component through the application UI.
- Visual components save and publish through the existing ComponentVersion authoring workflow.
- Visual component validation rejects configs that reference fields outside the bound Dataset major-line contract.
- Visual component validation rejects unsupported or incomplete visual configs with stable validation findings.
- Published visual components render through application viewer screens.
- Visual component viewers use ComponentVersion as the source of truth, not deprecated report/chart/workbench assets.
- Reader routes expose only visible published visual components.
- Management routes expose only manageable visual component drafts/versions.
- Touched legacy visual-analysis endpoints, if any, remain adapter-only and enforce scoped component/dashboard visibility before returning metadata.
- Touched Component routes remain native Leptos SSR routes.

## Manual Test Plan

Admin happy path:

1. Sign in as `admin@tessara.local`.
2. Open `/components`.
3. Create a new visual component.
4. Select a Dataset major-line source.
5. Configure a Bar chart and publish it.
6. Repeat for Line, Pie/Donut, and StatCard configurations.
7. Open each published component viewer and confirm the expected visual renders.
8. Open component version history and confirm visual component versions use the same versioning workflow as Table components.

Validation and edit paths:

1. Attempt to save a visual component with a missing category/value/stat field.
2. Attempt to save a visual component with a field not present in the selected Dataset major-line output.
3. Confirm validation findings are visible and stable.
4. Edit a published visual component as a draft.
5. Confirm the viewer remains on the current published version until update or new-version publish.

Scoped and reader paths:

1. Sign in as a scoped operator.
2. Confirm visible published visual components load through reader routes.
3. Confirm hidden visual components and hidden Dataset major lines are absent or forbidden.
4. Confirm reader-only users do not see draft metadata or authoring controls.

Legacy adapter check, if legacy visual endpoints are touched:

1. Identify the touched endpoint.
2. Confirm the endpoint is documented as adapter-only.
3. Confirm scoped visibility is enforced before metadata is returned.

## Automated Test Plan

Run:

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo test -p tessara-web-components`
- `cargo test -p tessara-web-datasets`
- `cargo test -p tessara-web-data-ops`
- `npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Backend/API scenarios:

- Accept valid Bar, Line, Pie/Donut, and StatCard component configs.
- Reject visual component configs that reference fields outside the Dataset major-line contract.
- Reject incomplete visual component configs with stable validation findings.
- Execute/render published visual component data from the ComponentVersion and bound Dataset major line.
- Preserve Sprint 4A Table component behavior while adding visual kinds.
- Keep draft-only visual components hidden from reader routes.
- Enforce scoped visibility for visual component list/detail/viewer routes.
- Enforce scoped visibility for any touched legacy visual-analysis metadata endpoint.

Frontend/E2E scenarios:

- `/components`, `/components/new`, `/components/:component_ref`, `/components/:component_ref/edit`, `/components/:component_ref/versions`, and `/components/:component_ref/view` remain native and usable.
- Create/publish/view a Bar component.
- Create/publish/view a Line component.
- Create/publish/view a Pie or Donut component.
- Create/publish/view a StatCard component.
- Validation findings render for incomplete or invalid visual configs.
- Reader-only users do not see draft metadata or authoring actions.
- Scoped users cannot direct-load hidden visual components.

## Ordered Implementation Plan

1. Inventory current Component table contracts, DTOs, validation, and viewer surfaces to identify extension points for visual component kinds.
2. Define typed config contracts for Bar, Line, Pie/Donut, and StatCard components over Dataset major-line fields.
3. Extend backend component validation and execution/view-model APIs for visual component kinds while preserving table behavior.
4. Extend Component authoring UI with visual kind selection and kind-specific config controls.
5. Add visual component viewer rendering for each delivered kind.
6. Add reader/manage scoped authorization coverage for visual component routes and any touched legacy adapter endpoint.
7. Add unit and Playwright coverage for visual authoring, validation, publishing, viewing, and scoped negative paths.
8. Run full closeout validation and update sprint progress notes.

## Dependencies And Blockers

- Sprint 4A ComponentVersion and Dataset major-line behavior is the foundation for this sprint.
- Seeded data must include Datasets with fields suitable for category/value/stat visual examples, or the sprint must add fixtures that make those visual component demos reliable.
- Full verification depends on local app launch, Playwright availability, and native routes hydrating cleanly.
- If legacy visual-analysis endpoints need to be touched, their adapter-only status and scoped visibility behavior must be documented before closeout.

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
- Visual components may own a bounded presentation transform over the bound Dataset major-line output, inspired by the legacy MMI chart model but implemented on the new `Dataset -> ComponentVersion` path. The transform may summarize, group, limit, and handle missing values for the visual; it must not create separate Report, Aggregation, Chart, workbench, or dashboard assets.
- Sprint 4B delivers core legacy-chart parity adapted to Components: StatCard replaces the legacy Badge concept, Bar supports both summary and comparison modes, Line supports trend-style summaries, and Pie/Donut share summary-by-category behavior. Gauges, richer chart controls, and additional chart types remain future scope.
- The v1 visual renderer should be native Leptos/SVG/CSS in `tessara-web-components`. Do not introduce a JavaScript chart controller, bridge asset, or workbench-owned rendering path for this sprint.
- Existing `/api/components/{component_ref}/table` and versioned table execution endpoints remain table-only compatibility endpoints for Table viewers and tests. Visual execution uses kind-specific endpoints: `GET /api/components/{component_ref}/bar`, `/line`, `/pie`, `/donut`, and `/stat-card`, plus versioned variants under `GET /api/components/{component_ref}/versions/{version_id}/{kind}`. These endpoints must load only the selected `ComponentVersion`, its Dataset major-line binding, and the bound Dataset major-line rows/fields.
- The common Component list/detail/version routes remain the entrypoint for all Component kinds. Kind-specific execution endpoints are implementation/API boundaries for viewers and tests, not separate product asset families.
- Pie and Donut share the same validation and data contract, with `component_type` selecting the visual treatment. Both are accepted deliverables; the UI may expose them as a single segmented Pie/Donut kind selector.
- Legacy visual-analysis endpoints should not be touched unless implementation proves they are necessary for compatibility. If touched, the sprint must document the exact endpoint and prove it is adapter-only with scoped component/dashboard visibility enforcement.

### V1 Visual Config Contracts

Visual configs are presentation-level transforms over Dataset major-line rows. All field references below must resolve against the bound Dataset major-line output fields. Unknown config keys are rejected.

Shared config:

- `summary_field`: Dataset output field to summarize.
- `summary_type`: one of `count`, `unique_count`, `sum`, `average`, or `median`.
- `value_format`: optional `plain`, `integer`, `decimal`, or `percent`; default `plain`.
- `missing_policy`: optional `omit`, `zero`, or `explicit_missing`; default `omit`.

Shared validation:

- `sum`, `average`, and `median` require `summary_field` to be a Dataset `number` field.
- `count` and `unique_count` may summarize any field type.
- `missing_policy = omit` drops rows with missing selected chart inputs from the visual transform.
- `missing_policy = zero` treats missing numeric summary values as `0` where a numeric calculation is required.
- `missing_policy = explicit_missing` preserves missing category/group/x values as a visible missing-data bucket. Missing summary-field values are included as an explicit missing bucket for `count` and `unique_count`; numeric blanks are excluded from `sum`, `average`, and `median` instead of being fabricated.
- `max_items`, `max_slices`, and `number_of_points` are positive integers with default `20` and maximum `100`.

Kind-specific config:

- `stat_card`: `summary_field`, `summary_type`, optional `label`, optional `value_format`, optional `supporting_text`, optional `panel_style`.
- `bar`: `mode` of `summary` or `comparison`, `summary_field`, `summary_type`, `category_field`, optional `comparison_field` when `mode = comparison`, optional `orientation` of `vertical` or `horizontal`, optional `number_of_points`, optional `fraction` of `top` or `bottom`.
- `line`: `summary_field`, `summary_type`, `x_field`, optional `number_of_points`.
- `pie` / `donut`: `summary_field`, `summary_type`, `category_field`, optional `max_slices`.

Validation should reject:

- missing required fields for the selected kind;
- field references outside the bound Dataset major-line contract;
- unsupported enum values such as unknown mode, fraction, orientation, missing policy, or format options;
- out-of-range limits;
- stale legacy asset identifiers such as `report_id`, `chart_id`, `aggregation_id`, or workbench references.

Future carry-forward:

- Add a Dataset operation that can unfold multi-select values into separate rows before visual components consume the Dataset major line.
- Add future visual-component support for deferred legacy-inspired chart behavior, including gauges, richer comparison/series controls, and expanded chart settings. These are not Sprint 4B acceptance requirements.

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
- A tester can create both Pie and Donut component treatments through the application UI.
- A tester can create a StatCard component through the application UI.
- Visual components save and publish through the existing ComponentVersion authoring workflow.
- Visual component validation rejects configs that reference fields outside the bound Dataset major-line contract.
- Visual component validation rejects unsupported or incomplete visual configs with stable validation findings.
- Published visual components render through application viewer screens.
- Visual component viewers use ComponentVersion as the source of truth, not deprecated report/chart/workbench assets.
- Kind-specific visual execution endpoints return stable view models for Bar, Line, Pie, Donut, and StatCard, including explicit version viewing.
- Table execution endpoints remain table-only and reject visual component kinds with stable unsupported-kind errors.
- Reader routes expose only visible published visual components.
- Management routes expose only manageable visual component drafts/versions.
- Touched legacy visual-analysis endpoints, if any, remain adapter-only and enforce scoped component/dashboard visibility before returning metadata.
- Touched Component routes remain native Leptos SSR routes.
- Sprint closeout includes evidence that touched Component routes are hydration-clean, browser-console-clean, and do not request `/bridge/*` assets.
- Sprint closeout includes a legacy visual-analysis endpoint inventory: either "No legacy visual-analysis endpoints touched" or a list of touched endpoints with adapter-only proof, source-of-truth proof, scoped visibility tests, and automated coverage.

## Manual Test Plan

Admin happy path:

1. Sign in as `admin@tessara.local`.
2. Open `/components`.
3. Create a new visual component.
4. Select a Dataset major-line source.
5. Configure a Bar chart and publish it.
6. Repeat for Line, Pie, Donut, and StatCard configurations.
7. Open each published component viewer and confirm the expected visual renders.
8. Open component version history and confirm visual component versions use the same versioning workflow as Table components.

Validation and edit paths:

1. Attempt to save a visual component with a missing category/value/stat field.
2. Attempt to save a visual component with a field not present in the selected Dataset major-line output.
3. Attempt to save unsupported summary, missing-policy, format, mode, and limit values.
4. Confirm validation findings are visible and stable.
5. Edit a published visual component as a draft.
6. Confirm the viewer remains on the current published version until update or new-version publish.
7. Update an existing published version and confirm the viewer changes without creating a new version.
8. Create a new version with a required note and confirm version history shows both versions.
9. Open an older explicit visual version and confirm it still renders the older visual config.

Scoped and reader paths:

1. Sign in as a scoped operator.
2. Confirm visible published visual components load through reader routes.
3. Confirm hidden visual components and hidden Dataset major lines are absent or forbidden.
4. Confirm reader-only users do not see draft metadata or authoring controls.
5. Direct-load a hidden visual component execution endpoint and confirm it is forbidden.

Legacy adapter check, if legacy visual endpoints are touched:

1. Identify the touched endpoint.
2. Confirm the endpoint is documented as adapter-only.
3. Confirm scoped visibility is enforced before metadata is returned.
4. If no legacy visual-analysis endpoint is touched, record that explicitly in closeout notes.

## Automated Test Plan

Run:

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo test -p tessara-web-components`
- `cargo test -p tessara-web-datasets`
- `cargo test -p tessara-web-data-ops`
- `npx playwright test`
- `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Backend/API scenarios:

- Accept valid Table, Bar, Line, Pie, Donut, and StatCard component configs after widening the `component_type` enum/constraint.
- Reject unknown, unsupported, and deprecated component kinds.
- Accept valid visual configs using `count`, `unique_count`, `sum`, `average`, and `median` summary functions.
- Reject numeric summary functions against non-number fields.
- Reject unsupported `value_format`, `missing_policy`, Bar mode, Bar fraction, orientation, and out-of-range limit values.
- Reject visual component configs that reference fields outside the Dataset major-line contract.
- Reject incomplete visual component configs with stable validation findings.
- Execute/render published visual component data from the ComponentVersion and bound Dataset major line through kind-specific endpoints.
- Confirm kind-specific explicit-version endpoints return the requested published version instead of always returning the current published version.
- Confirm table endpoints continue to work for Table components and reject visual component kinds with stable unsupported-kind errors.
- Preserve Sprint 4A Table component behavior while adding visual kinds.
- Preserve atomic edit-screen behavior: a failed visual save does not partially mutate component metadata or the existing published config.
- Keep draft-only visual components hidden from reader routes.
- Enforce scoped visibility for visual component list/detail/viewer routes.
- Enforce scoped visibility for any touched legacy visual-analysis metadata endpoint.

Frontend/E2E scenarios:

- `/components`, `/components/new`, `/components/:component_ref`, `/components/:component_ref/edit`, `/components/:component_ref/versions`, and `/components/:component_ref/view` remain native and usable.
- Create/publish/view a Bar component.
- Create/publish/view a Line component.
- Create/publish/view both Pie and Donut component treatments.
- Create/publish/view a StatCard component.
- Validation findings render for incomplete or invalid visual configs.
- Visual component save draft, publish first version, update existing version, create new version with note, and explicit older-version viewing flows work.
- Reader-only users do not see draft metadata or authoring actions.
- Scoped users cannot direct-load hidden visual components.
- Touched `/components` routes remain browser-console clean, hydration clean, and free of `/bridge/*` asset requests.

Permission fixture requirements:

- Seed or UAT fixtures must include one admin, one reader-only user, one component manager, and one scoped operator.
- Fixtures must include at least one visible Dataset major line, one hidden Dataset major line, one published visible visual component, one published hidden visual component, one draft-only visual component, and one attempted out-of-scope Dataset binding/publish path.
- Update `docs/playwright-permissions-scenarios.md` for the visual component reader, manager, and scoped negative cases.

## Ordered Implementation Plan

1. Inventory current Component table contracts, DTOs, validation, and viewer surfaces to identify extension points for visual component kinds.
2. Define typed config contracts for Bar, Line, Pie, Donut, and StatCard components over Dataset major-line fields, including summary functions, Bar modes, missing policies, and limit rules.
3. Update the `component_versions.component_type` schema constraint/enum and API/DTO parsing so `table`, `bar`, `line`, `pie`, `donut`, and `stat_card` are accepted while unknown and deprecated kinds are rejected.
4. Extend backend component validation and kind-specific visual execution APIs while preserving table behavior.
5. Add bounded visual transform logic over Dataset major-line rows for summary, grouping, comparison, missing-value handling, and limits.
6. Extend Component authoring UI with visual kind selection and kind-specific config controls.
7. Add visual component viewer rendering for each delivered kind.
8. Add reader/manage scoped authorization coverage for visual component routes and any touched legacy adapter endpoint.
9. Add unit and Playwright coverage for visual authoring, validation, publishing, viewing, and scoped negative paths.
10. Run full closeout validation, update permission scenario docs, and update sprint progress notes with the legacy endpoint inventory.

## Dependencies And Blockers

- Sprint 4A ComponentVersion and Dataset major-line behavior is the foundation for this sprint.
- Seeded data must include Datasets with fields suitable for summary, grouping, comparison, category, x-axis, and stat visual examples, or the sprint must add fixtures that make those visual component demos reliable.
- Full verification depends on local app launch, Playwright availability, and native routes hydrating cleanly.
- If legacy visual-analysis endpoints need to be touched, their adapter-only status and scoped visibility behavior must be documented before closeout.
- Dataset multi-select unfolding is explicitly deferred to a future Dataset operation; Sprint 4B visual transforms consume the Dataset major-line rows they are given.

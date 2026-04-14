# Tessara Migration Roadmap

This roadmap reflects the current Tessara implementation state and the current
UI direction captured in [ui-direction.md](./ui-direction.md).

The reporting model remains:

```text
Forms -> Dataset -> Report -> Aggregation -> Chart -> Dashboard
```

The UI direction now remains intentionally constrained by backend reality:

- preserve the legacy application's functional structure and main use cases
- do not copy the legacy UI exactly
- use split product areas as the target information architecture
- stage one shared home shell first
- keep Administration and Migration visible but internal/operator scoped
- use configured domain labels rather than hardcoded legacy entity names

## Completed Work

The following work is already implemented and should be treated as baseline,
not future roadmap scope.

### Completed Foundation Slices

1. Workspace, Docker Compose, PostgreSQL, smoke workflows, and local test deployment.
2. Dev identity, admin login, protected admin routes, and session flows.
3. Configurable hierarchy, metadata fields, runtime nodes, and relationship validation.
4. Versioned forms, publish lifecycle, rendered forms, draft/save/submit flows, and submission review.
5. Analytics projection and reportable tables.
6. DataFusion-backed report execution.
7. Charts, dashboards, dashboard preview, and reporting asset discovery.
8. Legacy fixture validation, import rehearsal, dry-run support, and seeded demo workflows.
9. Multi-crate workspace extraction for core domain rules.
10. Leptos SSR application shell, focused routes, favicon/branding integration, and Docker user-test deployment.

### Completed Reporting Refactor Slices

11. Dataset domain and storage.
12. Single-form dataset execution.
13. Dataset-backed reports.
14. Limited v1 row-level computed report fields.
15. Aggregation layer with `count`, `sum`, `avg`, `min`, and `max`.
16. Multi-source datasets with explicit composition mode, source-aware previews, `latest`/`earliest` selection, and implemented join-mode dataset execution for the supported path.
17. Charts and dashboards that can target report or aggregation outputs.

### Completed UI Catch-Up Work

The UI is not fully caught up to the backend yet, but these pieces are already in place:

- shared Leptos application shell
- route-level home/workspace shells for `/app`, `/app/submissions`, `/app/admin`, `/app/reports`, and `/app/migration`
- persistent navigation and create-menu scaffolding
- visible workspace framing for submissions, admin, and reporting
- task/context panel layouts for the focused route screens
- provenance review and UI guidance documents:
  - [user-interface-design.md](./user-interface-design.md)
  - [ui-direction.md](./ui-direction.md)

### Completed Access And UAT Baseline Work

The application now also has the baseline needed for role-aware UAT rather than
admin-only shell testing:

- dedicated `/app/login` flow instead of implicit admin bootstrap
- role-aware navigation and route guards for:
  - `admin/system`
  - `scoped operator/partner`
  - `respondent/client/parent`
- node-based operator scope with descendant expansion
- parent/subordinate respondent context support for the Responses area
- authenticated, role-aware `/api/app/summary`
- deterministic UAT demo seed integrated into:
  - [local-launch.ps1](D:\Projects\dms-migration\tessara\scripts\local-launch.ps1)
  - [seed-demo-data.ps1](D:\Projects\dms-migration\tessara\scripts\seed-demo-data.ps1)
- demo accounts and seeded scope assignments for role-based UAT

## Current Position

Tessara is now past the backend-feasibility stage.

The main remaining gap is not basic capability. The main gap is that the UI
still does not fully realize the backend structure or the legacy application's
working model.

Current status:

- backend/reporting architecture is substantially ahead of the UI
- dataset-backed reporting exists and is usable
- joined dataset execution exists for the currently supported path
- Sprint 1 is complete:
  - split product-area navigation is in place
  - shared shell framing is consistent across product and internal areas
  - product routes now read as workspaces and viewing destinations instead of mixed builder consoles
- UI-only catch-up remains the active focus, with Sprint 3 next
- Sprint 2 is complete:
  - `/app` now reads as the real shared home surface
  - home uses stable role-ready module zones without introducing separate role routes
  - demo/testing utilities are scoped to internal placement rather than the main home surface
  - home summary and current-context modules use existing backend and selection surfaces only
- Sprint 2.5 is complete:
  - Organization, Forms, Responses, Reports, and Dashboards now use dedicated navigable list, detail, create, and edit screens
  - product routes are now the canonical home for top-level entity CRUD/view flows
  - Administration has been pushed back to an internal advanced/configuration surface instead of staying the primary top-level entity workflow surface
  - the product routes no longer depend on visible manual ID entry fields for common record work
- role-aware access scaffolding is complete:
  - login is explicit
  - product/internal navigation is role-aware
  - operator visibility is scoped to assigned hierarchy nodes and descendants
  - respondents and parents are constrained to Responses-centric flows
- UAT demo seeding is complete:
  - local launch now ensures seeded Partner/Program/Activity/Session demo data exists
  - demo accounts exist for admin, operator, parent, respondent, and child respondent scenarios
- Sprint 3 is now active:
  - first task is the planned route/screen-oriented frontend refactor
  - then deepen the Organization and Forms product surfaces beyond the baseline list/detail sprint

## Active Carry-Forward Work

These are not separate future sprints. They are the current state that the next
planned sprints must absorb and complete.

### Carry-Forward A: UI catch-up

Still incomplete:

- the frontend code is still too concentrated in large route/controller files and should be reorganized by route/screen before more product-surface work lands
- Organization screens still need deeper hierarchy browsing, contextual traversal, and calmer detail presentation
- Forms screens still need stronger published/version visibility and clearer scope/report context
- Administration still exposes builder-era child-asset controls that should continue to recede behind clearer entity editing flows

### Carry-Forward B: reporting-product fit

Still incomplete:

- dataset/report/aggregation assets exist in the backend, but are not yet fully
  presented through product-grade list/detail/viewer flows
- report and dashboard viewing still share too much space with builder concerns

## Future Sprint Plan

The future roadmap is now organized as discrete, non-overlapping sprints.

Each sprint assumes the prior sprint is complete.

## Sprint 1: Shared Shell And Product-Area Navigation

**Outcome:** The shell reflects the direction in
[ui-direction.md](./ui-direction.md) and exposes the
real product areas.

**Status:** Complete

**Build:**

- Refactor the current shell to present the target top-level areas:
  - Home
  - Organization
  - Forms
  - Responses
  - Reports
  - Dashboards
  - Administration
  - Migration
- Add consistent page-title rows, breadcrumbs, and primary action regions.
- Preserve current route functionality while reorganizing the shell structure.
- Keep Administration and Migration visually present but clearly internal.

**Acceptance criteria:**

- Navigation reflects the split product areas.
- The shell no longer reads as a reporting/admin workbench.
- Current routes map cleanly into the new IA without inventing unsupported behavior.

**Completed in code:**

- canonical product-area routes are in place for:
  - Home
  - Organization
  - Forms
  - Responses
  - Reports
  - Dashboards
  - Administration
  - Migration
- shared page-shell framing now includes:
  - breadcrumbs
  - page-title/action rows
  - consistent product-area vs internal-area navigation
- product routes now use:
  - workspace framing
  - browse/review/view language
  - route-specific detail/result panels
  - refresh-oriented header actions instead of setup/demo shortcuts
- Administration and Migration remain visible and clearly internal

## Sprint 2: Home Surfaces And Role-Ready Entry Points

**Outcome:** Tessara has a real shared home that behaves like a product entry
point and is structurally ready for role-aware variants later.

**Status:** Complete

**Build:**

- Rework `/app` into the shared home prescribed by
  [ui-direction.md](./ui-direction.md).
- Add stable home module zones that can later support:
  - admin/system
  - scoped operator/partner
  - respondent/client/parent
- Keep one shared home implementation, but structure it so different home
  variants can later compose distinct content.
- Demote demo/testing actions out of the main home surface and into clearly
  internal placement.
- Add a home summary/readiness module using the existing `/api/app/summary`
  endpoint.
- Add a home current-context module using the existing shared selection state.
- Surface organization, response, report, and dashboard entry points from home.

**Acceptance criteria:**

- The home screen reads as a true application entry point.
- The home layout supports future role-aware variants without requiring route-tree rewrites.
- Users can enter the main product areas from home without relying on utility-style buttons.
- The home uses existing summary and selection data rather than new backend-only
  APIs.

**Completed in code:**

- `/app` now presents:
  - role-ready home modules
  - product-area entry points
  - current deployment readiness
  - current workflow context
  - internal areas
- home summary uses the existing `/api/app/summary` surface
- home current-context uses the existing shared selection state
- demo/testing actions have been removed from the main home surface and scoped to Administration
- visible manual `ID` entry fields have been removed from rendered screens so common flows are selection-driven

## Sprint 2.5: Entity CRUD/List Surfaces

**Outcome:** The five primary entity families now use dedicated navigable list,
detail, create, and edit screens under their product areas, replacing the old
workspace-panel approach for top-level entity workflows.

**Status:** Complete

**Build:**

- Insert a focused detour between Sprint 2 and Sprint 3 for these entity families:
  - Organization
  - Form
  - Response
  - Report
  - Dashboard
- Move top-level browse/list/detail/create/edit for Organization, Form, Response, Report, and Dashboard into product-area routes.
- Keep Response creation/editing in the Responses area through the existing draft lifecycle.
- Freeze Administration as an internal advanced/configuration surface instead of expanding it further for top-level entity workflows.
- Add the minimal missing backend support for runtime organization detail retrieval.

**Acceptance criteria:**

- Organization, Forms, Responses, Reports, and Dashboards each expose dedicated list/create/detail/edit screens.
- Product routes do not depend on visible manual ID entry fields.
- Response draft editing stays in Responses, and submitted responses remain read-only.
- Administration remains available, but top-level entity CRUD/view no longer depends on it.

**Completed in code:**

- added `GET /api/nodes/{node_id}` for runtime organization detail
- product routes now render dedicated navigable list/detail/create/edit screens for:
  - Organization
  - Forms
  - Responses
  - Reports
  - Dashboards
- product-route controllers now initialize by route type rather than reusing mixed workspace/task-panel screens
- top-level entity forms now live on dedicated create/edit pages with `Submit` and `Cancel`
- Administration is now an internal landing surface linking to legacy advanced tooling rather than the primary top-level entity workflow area

## Sprint 3: Organization And Forms Product Surfaces

**Outcome:** The legacy partner/program/activity/session mental model is
translated into Tessara-native Organization and Forms areas.

**Status:** Active

**First task:**

- Refactor the frontend codebase to organize the product UI by route/screen before adding more Organization and Forms behavior.
- Split the current large page/controller files into route- or screen-oriented modules so Sprint 3 work lands in a clearer structure:
  - shared shell/navigation
  - Organization screens
  - Forms screens
  - other product-area screens as needed

**Build:**

- Create Organization directory/detail flows over the hierarchy model.
- Present scoped labels through configured terminology rather than hardcoded node language.
- Create Forms directory/detail flows with:
  - form detail
  - version detail
  - publish status
  - related organization scope
- Move builder/configuration actions for hierarchy and forms into contextual task areas instead of mixed control clusters.

**Acceptance criteria:**

- Organization is discoverable as a first-class product area.
- Forms are discoverable without entering the admin workbench.
- Common hierarchy/form navigation no longer depends on manual ID handling.
- Product UI code is organized by route/screen strongly enough that follow-on
  screen work does not continue to accrete into monolithic controller files.

## Sprint 4: Responses Product Surface

**Outcome:** `/app/submissions` becomes a real Responses area that matches the
legacy client/parent working model.

**Build:**

- Rework Responses into:
  - pending responses
  - drafts
  - submitted responses
  - response detail
  - read-only completed view
- Keep draft/save/submit behavior aligned with current backend support.
- Make response review calmer and more product-like than the current mixed route.
- Ensure respondent workflows remain narrower than internal admin workflows.

**Acceptance criteria:**

- A tester can find pending, draft, and submitted responses without knowing IDs.
- Submitted responses are clearly read-only.
- The Responses area feels like a user workflow, not a builder surface.

## Sprint 5: Reports And Dashboards Product Surfaces

**Outcome:** Reports and Dashboards become true viewing destinations, separated
from reporting configuration.

**Build:**

- Create report directory/detail/viewer flows.
- Create dashboard directory/detail/viewer flows.
- Keep reporting configuration available, but move it out of the main viewing surface.
- Surface linked traversal between datasets, reports, aggregations, charts, and dashboards through detail pages instead of workbench-style action clusters.

**Acceptance criteria:**

- Reports are discoverable through a directory and readable through a dedicated viewer flow.
- Dashboards are discoverable through a directory and readable through a dedicated viewer flow.
- Product-facing report/dashboard usage is visually distinct from configuration.

## Sprint 6: Administration And Migration Internal Surfaces

**Outcome:** Internal/operator-only areas are coherent, clearly scoped, and no
longer define the tone of the whole application.

**Build:**

- Reframe Administration as the internal configuration area for:
  - hierarchy
  - forms
  - reporting assets
- Reframe Migration as the internal/operator area for:
  - validation
  - dry-run
  - import
  - verification
- Remove remaining mixed-purpose layout patterns from those internal areas.
- Keep them powerful, but visually subordinate to the product-facing areas.

**Acceptance criteria:**

- Administration is clearly an internal configuration surface.
- Migration is clearly an operator workflow, not a normal product area.
- The main product shell is no longer dominated by builder-era control panels.

## Sprint 7: Dataset/Report/Aggregation UI Completion

**Outcome:** The reporting stack is fully configurable through application-grade
screens rather than legacy workbench controls.

**Build:**

- Finish dataset builder UI for supported composition modes and selection rules.
- Finish report builder UI for dataset field selection, computed fields, and report shaping.
- Finish aggregation builder UI for group-by fields and metrics.
- Keep all reporting-layer validation surfaced next to the layer that owns it.

**Acceptance criteria:**

- A tester can create dataset, report, aggregation, chart, and dashboard assets through application screens without copying IDs.
- Validation messages appear in the right product/configuration area.
- The UI clearly reflects the reporting model:
  - forms capture data
  - datasets define meaning
  - reports select rows
  - aggregations summarize

## Sprint 8: Legacy Mapping And Import Refactor

**Outcome:** Legacy import and mapping fully align with the dataset-first
reporting model and the new UI structure.

**Build:**

- Update mapping docs and fixture-import logic so imported reporting assets are created as:
  - dataset
  - report
  - aggregation
  - chart/dashboard where applicable
- Ensure migration verification flows link into the new product/detail surfaces.
- Update post-import checks to validate dataset-backed reporting outputs.

**Acceptance criteria:**

- Imported fixtures create dataset-first reporting assets.
- Verification paths land in the new application surfaces rather than workbench-only routes.
- Imported grouped reporting examples validate through aggregations, not grouped reports.

## Sprint 9: Hardening And Pilot Readiness

**Outcome:** Tessara is ready for broader local user testing and pilot-oriented
validation.

**Build:**

- Add broader permission coverage for internal mutation routes.
- Add end-to-end tests for:
  - home/navigation
  - organization/form flows
  - response flows
  - report/dashboard flows
  - migration/operator verification flows
- Expand Docker Compose smoke coverage for the final application-shell paths.
- Document unsupported v1 areas clearly.

**Acceptance criteria:**

- Local test, clippy, and smoke gates pass on the application-shell paths.
- The UI is caught up enough to the backend that testers are using product flows, not workbench workarounds.
- Remaining unsupported features are explicit and documented.

## Deferred Work

- Fuzzy joins.
- Complex window functions.
- Cross-dataset joins.
- Full visual dashboard designer.
- Broad role-specific home implementations after the shared home shell settles.
- Selective asset inheritance below operator scope roots.
- Any UI flow that implies unsupported backend permissions, assignment, sharing, or cross-scope behavior.

## Immediate Next Sprint

Continue Sprint 3.

The next concrete milestone should be:

- refactor the frontend into route/screen-oriented modules
- separate shared shell/navigation concerns from Organization and Forms screens
- then continue deeper Organization hierarchy browsing and Forms detail/version work

That work should follow [ui-direction.md](./ui-direction.md)
and the dedicated-screen direction already implemented. It should not reintroduce
multi-purpose testing surfaces as the primary product UX.

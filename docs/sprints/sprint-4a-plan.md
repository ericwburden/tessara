# Sprint 4A Plan: Table Component Slice

Kickoff status: started from clean `main` on 2026-07-03.

## Sprint Summary

Build Sprint 4A from the roadmap `(Next)` scope: make table-oriented presentation assets first-class components. The sprint delivers component authoring, versioning, publication, dataset-revision binding validation, scoped list/detail visibility, and application table viewers.

Kickoff defaults:

- Branch: `codex/sprint-4a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-4a`
- Plan artifact: `docs/sprints/sprint-4a-plan.md`

## Sprint Specifications

- Implement Component frontend surfaces in a dedicated `tessara-web-components` crate from the start.
- Keep root `tessara-web` responsible for route adapters, shell/auth/session/navigation policy, hydration, document integration, CSS, and assets.
- Add `DetailTable` and `AggregateTable` authoring.
- Add component versioning and publication behavior.
- Validate component definitions and bind table components to dataset revisions.
- Keep any retained legacy analytical endpoints adapter-only; no new core behavior should deepen deprecated asset families.
- Continue hybrid-shell removal on touched reporting and component routes rather than creating a second long-lived bridge.
- Enforce scoped dataset and component visibility in component list/detail endpoints.
- Add negative operator coverage for scoped visibility behavior.

## Acceptance Criteria

- A tester can create, version, publish, and view table components in the app.
- Component directory, detail, create, edit, and publish flows are available in the application UI.
- `DetailTable` and `AggregateTable` components can be authored against permitted dataset revisions.
- Table viewers render inside the application for published table components.
- Component versioning preserves published versions while allowing draft/edit flows.
- Component list/detail endpoints only expose datasets and components visible to the active operator.
- Legacy analytical endpoints touched during the sprint remain adapter-only and do not gain new core behavior.

## Manual Test Plan

- Sign in as an admin and open the Components area.
- Create a `DetailTable` component from a visible dataset revision.
- Edit the component, save a new version, publish it, and confirm the published viewer renders in the app.
- Create an `AggregateTable` component from a visible dataset revision and confirm aggregate table output renders correctly.
- Confirm component directory and detail pages show draft, version, and publication state clearly.
- Sign in as an operator with restricted access and confirm hidden datasets/components are unavailable in list, detail, authoring, and viewer flows.
- Confirm reporting/component routes touched during the sprint still load through the intended shell and adapters.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo test -p tessara-web-components` if the new crate exposes runnable tests
- `npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Scenario coverage:

- component create/edit/version/publish API behavior
- dataset-revision binding validation for table components
- `DetailTable` and `AggregateTable` contract validation
- scoped dataset and component visibility, including negative operator cases
- component directory/detail/create/edit/publish web flows
- table viewer rendering for published components

## Ordered Implementation Plan

1. Inventory existing component, dashboard, report, and legacy analytical endpoint contracts to identify adapter boundaries and reusable API behavior.
2. Add or refine API contracts for table component kinds, component definitions, version states, publication, validation, and dataset-revision bindings.
3. Implement component repository/service behavior for create, edit, version, publish, validation, and scoped list/detail access.
4. Add focused API tests, including negative operator coverage for dataset and component visibility.
5. Create the `tessara-web-components` crate and move component-local contracts, API calls, loaders, and UI surfaces there.
6. Add root `tessara-web` route adapters and shell integration for component directory, detail, create, edit, publish, and viewer flows.
7. Implement `DetailTable` and `AggregateTable` authoring and table viewer UI.
8. Extend Playwright, smoke, and UAT coverage for the user-testable exit condition.
9. Run the planned verification set and update sprint notes with results.

## Dependencies And Blockers

- Sprint scope is limited to table-oriented presentation components and the application flows needed to create, version, publish, and view them.
- The new frontend component surfaces should live in `tessara-web-components`; root `tessara-web` should stay focused on adapters and application shell concerns.
- Deprecated analytical asset families may remain reachable through adapters, but Sprint 4A should not add new core behavior to them.
- Any broad migration beyond touched reporting/component routes is out of scope unless needed to avoid a second long-lived bridge.
- Full verification depends on the local app launching successfully and seeded or UAT-created data supporting component authoring flows.

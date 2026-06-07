# Playwright Permissions Scenarios

This report describes the executable Playwright permission coverage in `end2end/tests/permissions.spec.ts`.

The tested access model is capability + scope + ownership. Capabilities decide whether an action or surface exists, scoped role assignments limit that capability to an organization subtree, and response ownership/delegation grants access to assigned response work. The UI label "Responses" maps to the API capability family `submissions:*`.

Project directive: every new permission-controlled surface, action, or data-access path should add or update executable Playwright permission coverage when the behavior is currently testable. Each added scenario should include positive and negative assertions for the relevant capability, scope, and ownership/delegation combination, and this report should be updated in the same change.

## Fixture Accounts And Roles

The suite creates Playwright-owned fixtures with a `pw-permissions-*` prefix through admin APIs after seeding the demo dataset.

## Fixture Lifecycle

Permission fixtures are intentionally durable local records. The suite creates them through supported admin APIs and does not perform direct database cleanup because users, roles, assignments, drafts, submissions, and delegations are linked across the authorization model. Reusing supported APIs keeps the test setup representative of production behavior and avoids a hidden cleanup path that can drift from the app.

When local fixture volume gets noisy, reset the development database with `.\scripts\local-launch.ps1 -FreshData` before rerunning Playwright. Future cleanup work should prefer supported admin lifecycle APIs if user or role deletion/deactivation becomes a product feature; until then, keep fixture names prefixed with `pw-permissions-*` so they remain easy to identify.

| Fixture | Role capabilities | Scope/delegation purpose |
| --- | --- | --- |
| No-access user | empty role | Verifies protected capability surfaces reject accounts with no capabilities. |
| Response owner | `submissions:read_own`, `submissions:respond` | Verifies own assignment pending/start/read and denial for another user's assignment. |
| Scoped manager | `hierarchy:read`, `forms:read`, `workflows:read`, `workflows:manage`, `submissions:read_own`, `submissions:respond`, `submissions:manage`, `operations:view`, `datasets:read`, `components:read`, `dashboards:read` | Verifies scoped positive and negative access from the `Demo Program Family Outreach` subtree, plus own out-of-scope response access. |
| Delegate | `submissions:read_own`, `submissions:respond` | Verifies delegated-to-self work. |
| Delegator | `submissions:read_own`, `submissions:respond` | Verifies delegated work access through `delegate_account_id`. |
| Admin | existing `admin@tessara.local` with `admin:all` | Verifies global visibility and creates fixtures. |

## Implemented Scenario Matrix

| Scenario family | Positive checks | Negative checks |
| --- | --- | --- |
| Capability absence | None; no-access user authenticates only. | No-access user receives forbidden responses from admin, forms, workflows, workflow assignment, submissions, datasets, components, and dashboards APIs. |
| Shell navigation | Scoped manager can see allowed product nav such as Forms and Responses. | Scoped manager cannot see Administration navigation. |
| Hierarchy routes | Scoped manager can load organization list/detail/edit/create routes for visible hierarchy records. | Scoped manager does not see out-of-scope nodes in the organization list and direct out-of-scope detail/edit routes do not expose editable content. |
| Forms UI visibility | Scoped manager sees an in-scope form in `/forms`. | Scoped manager does not see an out-of-scope form in the list and direct detail navigation renders the unavailable state. |
| Forms manage routes | Scoped manager can create and update a form with in-scope visibility and load create/edit routes. | Scoped manager cannot create or update a form with out-of-scope visibility and out-of-scope edit routes do not expose edit actions. |
| Role route and creation | Admin creates a role through the admin API and loads `/administration/roles`. | Non-admin administration access is covered by shell/API denial scenarios. |
| Administration routes | Admin loads users, user detail, user edit, user access alias, and node-type routes; admin creates/updates a node type through APIs. | Scoped non-admin receives forbidden responses for users, user detail/access, and node-type admin APIs. |
| Global capability | Admin reads in-scope and out-of-scope forms, datasets, components, dashboards, and workflow assignments. | Not applicable; `admin:all` is intentionally global. |
| Scoped forms | Scoped manager lists and reads forms whose visibility nodes overlap the assigned subtree. | Out-of-scope form is absent from list and direct detail access is forbidden. |
| Scoped datasets | Scoped manager lists, reads, and previews datasets whose visibility nodes overlap the assigned subtree, with preview rows limited to effective scope. | Out-of-scope dataset is absent from list and direct detail/table access is forbidden. |
| Scoped components | Scoped manager lists and reads components backed by visible dataset revisions. | Out-of-scope component is absent from list and direct detail access is expected to be forbidden. |
| Scoped dashboards | Scoped manager lists and reads dashboards whose visibility overlaps the assigned subtree. | Out-of-scope dashboard is absent from list and direct detail access is forbidden. |
| Workflow candidates and assignments | Scoped manager sees only in-scope assignment candidates, can inspect assignees for an in-scope candidate, and can start an in-scope assignment through `workflows:manage`. | Scoped manager cannot create/start out-of-scope assignment work and should not see out-of-scope workflow assignments in the assignment list. |
| Operations status | Scoped manager with `operations:view` can load `/operations` and `GET /api/operations/status` for scoped workflow assignment and dataset readiness status. | No-access users do not see Operations navigation and receive forbidden status API responses; out-of-scope datasets and workflow assignment records are excluded for scoped users. |
| Workflow manage routes | Scoped manager creates and reads an in-scope workflow and loads create/detail/edit routes. | Scoped manager cannot create/update/read out-of-scope workflows; out-of-scope routes do not expose edit actions. |
| Submissions scope plus ownership | Scoped manager can read/start an out-of-scope assignment assigned to them through ownership and can manage in-scope submissions through scope. | Scoped manager cannot read an unrelated out-of-scope submission. |
| Response ownership | Owner sees and starts their own pending assignment, reads the resulting submission, and an isolated response-editor account can load the edit route for its own draft. | Owner cannot start another user's non-delegated assignment; a different user cannot load the draft edit route. |
| Delegation | Delegator can query delegate pending work through `delegate_account_id`, start it, and read the resulting submission. | Non-delegated owner cannot access that delegated assignment. |
| Dashboard manage routes | Scoped manager can create/update an in-scope dashboard and load create/edit placeholder routes. | Scoped manager cannot create/update dashboards with out-of-scope visibility. |
| Session metadata | Session endpoint exposes capabilities, assigned scope roots, and delegations. | No legacy access switch is asserted or required. |

## Known Remaining Gaps

- UI-level create/edit permission checks combine route assertions with API-level action checks where the current route is a placeholder or where form/editor interactions would require a larger purpose-built fixture.
- There is no New User Screen yet, so user creation is verified through admin APIs rather than a browser form.
- The report should be updated whenever new executable Playwright scenarios are added, especially for future advanced dataset authoring, component, and dashboard UI flows.

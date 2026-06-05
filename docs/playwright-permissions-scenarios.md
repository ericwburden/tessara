# Playwright Permissions Scenarios

This report describes the executable Playwright permission coverage in `end2end/tests/permissions.spec.ts`.

The tested access model is capability + scope + ownership. Capabilities decide whether an action or surface exists, scoped role assignments limit that capability to an organization subtree, and response ownership/delegation grants access to assigned response work. The UI label "Responses" maps to the API capability family `submissions:*`.

Project directive: every new permission-controlled surface, action, or data-access path should add or update executable Playwright permission coverage when the behavior is currently testable. Each added scenario should include positive and negative assertions for the relevant capability, scope, and ownership/delegation combination, and this report should be updated in the same change.

## Fixture Accounts And Roles

The suite creates Playwright-owned fixtures with a `pw-permissions-*` prefix through admin APIs after seeding the demo dataset.

| Fixture | Role capabilities | Scope/delegation purpose |
| --- | --- | --- |
| No-access user | empty role | Verifies protected capability surfaces reject accounts with no capabilities. |
| Response owner | `submissions:read_own`, `submissions:respond` | Verifies own assignment pending/start/read and denial for another user's assignment. |
| Scoped manager | `hierarchy:read`, `forms:read`, `workflows:read`, `workflows:manage`, `submissions:read_own`, `submissions:respond`, `submissions:manage`, `datasets:read`, `components:read`, `dashboards:read` | Verifies scoped positive and negative access from the `Demo Program Family Outreach` subtree, plus own out-of-scope response access. |
| Delegate | `submissions:read_own`, `submissions:respond` | Verifies delegated-to-self work. |
| Delegator | `submissions:read_own`, `submissions:respond` | Verifies delegated work access through `delegate_account_id`. |
| Admin | existing `admin@tessara.local` with `admin:all` | Verifies global visibility and creates fixtures. |

## Implemented Scenario Matrix

| Scenario family | Positive checks | Negative checks |
| --- | --- | --- |
| Capability absence | None; no-access user authenticates only. | No-access user receives forbidden responses from admin, forms, workflows, workflow assignment, submissions, datasets, components, and dashboards APIs. |
| Shell navigation | Scoped manager can see allowed product nav such as Forms and Responses. | Scoped manager cannot see Administration navigation. |
| Global capability | Admin reads in-scope and out-of-scope forms, datasets, components, dashboards, and workflow assignments. | Not applicable; `admin:all` is intentionally global. |
| Scoped forms | Scoped manager lists and reads forms whose visibility nodes overlap the assigned subtree. | Out-of-scope form is absent from list and direct detail access is forbidden. |
| Scoped datasets | Scoped manager lists and reads datasets whose visibility nodes overlap the assigned subtree. | Out-of-scope dataset is absent from list and direct detail access is forbidden. |
| Scoped components | Scoped manager lists and reads components backed by visible dataset revisions. | Out-of-scope component is absent from list and direct detail access is expected to be forbidden. |
| Scoped dashboards | Scoped manager lists and reads dashboards whose visibility overlaps the assigned subtree. | Out-of-scope dashboard is absent from list and direct detail access is forbidden. |
| Workflow candidates and assignments | Scoped manager sees only in-scope assignment candidates, can inspect assignees for an in-scope candidate, and can start an in-scope assignment through `workflows:manage`. | Scoped manager cannot create/start out-of-scope assignment work and should not see out-of-scope workflow assignments in the assignment list. |
| Submissions scope plus ownership | Scoped manager can read/start an out-of-scope assignment assigned to them through ownership and can manage in-scope submissions through scope. | Scoped manager cannot read an unrelated out-of-scope submission. |
| Response ownership | Owner sees and starts their own pending assignment and reads the resulting submission. | Owner cannot start another user's non-delegated assignment. |
| Delegation | Delegator can query delegate pending work through `delegate_account_id`, start it, and read the resulting submission. | Non-delegated owner cannot access that delegated assignment. |
| Session metadata | Session endpoint exposes capabilities, assigned scope roots, and delegations. | No profile-based access switch is asserted or required. |

## Known Remaining Gaps

- UI-level create/edit permission checks are still limited because many detailed authorization paths are most stable through API requests today.
- There is no New User Screen yet, so user creation is verified through admin APIs rather than a browser form.
- The report should be updated whenever new executable Playwright scenarios are added, especially for future form builder, dataset table, component, and dashboard UI flows.

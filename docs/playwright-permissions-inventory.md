# Playwright Permissions Inventory

This inventory records the behavior currently verified by Playwright and the permission-oriented scenarios that still need browser coverage. It is intentionally separate from Rust integration coverage; Rust tests already cover several lower-level authorization branches that Playwright does not yet exercise.

The target access model for browser behavior is capability + scope + ownership: capabilities control which surfaces/actions exist, role assignment scope limits those capabilities globally or to a subtree, and response ownership/delegation controls own-work access.

## Current Playwright Coverage

| Spec | Current tests | Permission behavior exercised |
| --- | --- | --- |
| `end2end/tests/app.spec.ts` | Root route inventory, bare login route, unauthenticated route redirect, authenticated primary route rendering, old `/app` route removal | Session cookie login, login shell isolation, protected-route redirect, admin-visible navigation and primary admin routes |
| `end2end/tests/workflow-mediated-assignments.spec.ts` | Generated single-form workflow after form publish, assignment creation, delegate pending work, assignment-backed response start, removed manual response start, generated workflow promotion/regeneration | Admin form/workflow management, generated workflow visibility, workflow assignment APIs, delegate-owned pending work, submission start/read through workflow assignment ownership |

## Seeded Accounts Used By Playwright

| Account | Current Playwright use | Notes |
| --- | --- | --- |
| `admin@tessara.local` | Used in both specs | Seeded with `admin:all`; exercises broad admin session, route/API visibility, form publishing, workflow assignment creation, and generated workflow management. |
| `delegate@tessara.local` | Used in workflow-mediated assignment tests | Exercises assigned/delegated response work discovery and start/read behavior. |
| `operator@tessara.local` | Not currently used | Needed for scoped subtree and capability-scope browser coverage. |
| `respondent@tessara.local` | Not currently used | Needed for own-assignment response coverage that is not delegated. |
| `delegator@tessara.local` | Not currently used | Needed for delegation-context coverage from the delegator side. |

## Capability Coverage Matrix

| Capability family | Current Playwright status | Needed future coverage |
| --- | --- | --- |
| Login/session | Covered | Add non-admin route/nav expectations when permission suites are added. |
| Administration | Partially covered | Admin-only users/roles/capabilities/node-type routes; future New User Screen flow. |
| Forms | Partially covered | Explicit visibility-scope reads, out-of-scope detail denial, scoped manage containment once UI supports it. |
| Workflows | Partially covered | Scoped available-node filtering, out-of-scope assignment denial, candidate and assignee filtering. |
| Submissions | Covered for assignment-backed delegate start/read | Respondent own work, delegator context, scoped operator review denial/allowance. |
| Datasets | Not covered | Visibility-scope list/detail and table row filtering. |
| Components | Not covered | Dataset-revision-inherited visibility. |
| Dashboards | Not covered | Dashboard visibility and component dataset-scope compatibility. |

## Future Comment-Only Scaffold Targets

The Playwright specs should carry TODO comments for these scenarios without adding `test.skip` placeholders:

- Forms: admin create/publish/read; scoped operator list/read overlap; direct out-of-scope form read denial; future scoped create/edit containment.
- Workflows: scoped operator sees overlapping available-node workflows; cannot assign/start out-of-scope workflow work; candidates and assignees are scope-filtered.
- Submissions: respondent own work; delegate delegated work; delegator/delegate context remains ownership/delegation-based; scoped operator can review only in-scope submissions.
- Datasets/components/dashboards: dataset visibility and row filtering; component visibility inherited from dataset revision; dashboard visibility and component compatibility.
- Administration: admin-only admin routes; future New User Screen; non-admin users cannot see Administration nav or load admin routes.

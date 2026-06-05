# Playwright Permissions Inventory

This inventory records the behavior currently verified by Playwright and the permission-oriented scenarios that still need browser coverage. It is intentionally separate from Rust integration coverage; Rust tests already cover several lower-level authorization branches that Playwright does not yet exercise.

The target access model for browser behavior is capability + scope + ownership: capabilities control which surfaces/actions exist, role assignment scope limits those capabilities globally or to a subtree, and response ownership/delegation controls own-work access.

## Current Playwright Coverage

| Spec | Current tests | Permission behavior exercised |
| --- | --- | --- |
| `end2end/tests/app.spec.ts` | Root assigned-work route, bare login route, unauthenticated route redirect, authenticated primary route rendering, old `/app` route removal | Session cookie login, login shell isolation, protected-route redirect, admin-visible navigation and primary admin routes |
| `end2end/tests/workflow-mediated-assignments.spec.ts` | Generated single-form workflow after form publish, assignment creation, delegate pending work, assignment-backed response start, removed manual response start, generated workflow promotion/regeneration | Admin form/workflow management, generated workflow visibility, workflow assignment APIs, delegate-owned pending work, submission start/read through workflow assignment ownership |
| `end2end/tests/permissions.spec.ts` | Playwright-owned roles/users, scoped role assignments, admin/global access, no-capability denials, scoped form/workflow/submission/dataset/component/dashboard checks, ownership/delegation checks, session metadata, route-level UI checks, and admin-only user/node-type route checks | Primary capability + scope + ownership regression suite. Verifies positive and negative access for global admin, scoped manager, response owner, delegate, delegator, no-access accounts, and current native route permission behavior. |

## Accounts Used By Playwright

| Account | Current Playwright use | Notes |
| --- | --- | --- |
| `admin@tessara.local` | Used across the Playwright specs | Seeded with `admin:all`; exercises broad admin session, route/API visibility, form publishing, workflow assignment creation, generated workflow management, fixture setup, and admin role UI creation. |
| `delegate@tessara.local` | Used in workflow-mediated assignment tests | Exercises assigned/delegated response work discovery and start/read behavior. |
| `pw-permissions-*-scoped-manager@tessara.local` | Created by `permissions.spec.ts` | Exercises scoped subtree capability behavior for forms, workflows, submissions, datasets, components, dashboards, and route/nav visibility. |
| `pw-permissions-*-owner@tessara.local` | Created by `permissions.spec.ts` | Exercises own-assignment response discovery, start, and read behavior. |
| `pw-permissions-*-delegate@tessara.local` | Created by `permissions.spec.ts` | Exercises delegated-to-self response work. |
| `pw-permissions-*-delegator@tessara.local` | Created by `permissions.spec.ts` | Exercises delegation-context access through `delegate_account_id`. |
| `pw-permissions-*-no-access@tessara.local` | Created by `permissions.spec.ts` | Exercises capability-absence denials across protected API families and admin UI/nav denial. |
| `operator@tessara.local`, `respondent@tessara.local`, `delegator@tessara.local` | Not used directly by current Playwright specs | Their behaviors are covered through Playwright-owned accounts with equivalent capability/scope/delegation fixtures, avoiding dependence on durable seeded demo account shape. |

## Capability Coverage Matrix

| Capability family | Current Playwright status | Needed future coverage |
| --- | --- | --- |
| Login/session | Covered | Keep session metadata assertions current when capability, scope, or delegation payloads change. |
| Administration | Covered for current users, roles, user detail/edit/access alias, and node-type route/API checks | Future New User Screen flow remains pending because the screen does not exist yet. |
| Forms | Covered for scoped list/detail visibility, direct out-of-scope denial, create/edit route checks, and scoped manage API containment | Deeper drag/drop form-builder authoring interactions can be added as UI-specific coverage. |
| Workflows | Covered for scoped candidates, assignees, assignment creation denial, assignment list filtering, start denial/allowance, create/detail/edit route checks, and scoped manage API containment | More detailed revision authoring UI interactions can be added as the workflow editor matures. |
| Submissions | Covered for scoped management, own response ownership, delegated work, unrelated out-of-scope denial, and response edit route ownership denial/allowance | Deeper response save/submit UI paths can be added with a purpose-built form fixture. |
| Datasets | Covered for visibility-scope list/detail | Dataset table row filtering once the table UI and API behavior are settled. |
| Components | Covered for dataset-revision-inherited list/detail visibility | UI-level component rendering/inspection checks as component screens mature. |
| Dashboards | Covered for visibility-scope list/detail, create/edit placeholder routes, and scoped manage API containment | UI-level dashboard component compatibility checks as dashboard authoring/viewing matures. |

## Future Comment-Only Scaffold Targets

The Playwright specs should carry TODO comments for these scenarios without adding `test.skip` placeholders:

- Forms: admin create/publish/read; scoped operator list/read overlap; direct out-of-scope form read denial; future scoped create/edit containment.
- Workflows: scoped operator sees overlapping available-node workflows; cannot assign/start out-of-scope workflow work; candidates and assignees are scope-filtered.
- Submissions: respondent own work; delegate delegated work; delegator/delegate context remains ownership/delegation-based; scoped operator can review only in-scope submissions.
- Datasets/components/dashboards: dataset visibility and row filtering; component visibility inherited from dataset revision; dashboard visibility and component compatibility.
- Administration: admin-only admin routes; future New User Screen; non-admin users cannot see Administration nav or load admin routes.

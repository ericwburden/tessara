# Playwright Permissions Inventory

This inventory records the behavior currently verified by Playwright and the permission-oriented scenarios that still need browser coverage. It is intentionally separate from Rust integration coverage; Rust tests already cover several lower-level authorization branches that Playwright does not yet exercise.

The target access model for browser behavior is capability + scope + ownership: capabilities control which surfaces/actions exist, role assignment scope limits those capabilities globally or to a subtree, and response ownership/delegation controls own-work access.

## Current Playwright Coverage

| Spec | Current tests | Permission behavior exercised |
| --- | --- | --- |
| `end2end/tests/app.spec.ts` | Root assigned-work route, bare login route, unauthenticated route redirect, authenticated primary route rendering, old `/app` route removal | Session cookie login, login shell isolation, protected-route redirect, admin-visible navigation and primary admin routes |
| `end2end/tests/dashboards.spec.ts` | Native Dashboard directory/create/detail/editor/viewer routes, metadata-only authoring loads, exact-version viewer execution, add/direct move/resize/remove/save/dirty preview, and narrow stacked layout | Admin Dashboard management and the execution boundary between metadata-only surfaces and explicit Component preview/viewer routes |
| `end2end/tests/workflow-mediated-assignments.spec.ts` | Generated single-form workflow after form publish, assignment creation, delegate pending work, assignment-backed response start, removed manual response start, generated workflow promotion/regeneration | Admin form/workflow management, generated workflow visibility, workflow assignment APIs, delegate-owned pending work, submission start/read through workflow assignment ownership |
| `end2end/tests/permissions.spec.ts` | Playwright-owned roles/users, scoped role assignments, mixed scope-mode role authoring, admin/global access, no-capability denials, scoped form/workflow/submission/dataset/component/dashboard checks, ownership/delegation checks, session metadata, route-level UI checks, admin-only user/node-type route checks, and four JavaScript-disabled direct-load/refresh route-family proofs | Primary capability + scope + ownership regression suite. Verifies ordinary mixed-mode rejection, the sole `admin:all` exception, positive and negative access for global admin, scoped manager, response owner, delegate, delegator, and no-access accounts, plus native SSR ownership across the frozen Core, Organization, Administration, Form, Workflow, Response, Dataset, Component, and Dashboard routes. |
| `end2end/tests/modules.spec.ts` | Five Sprint 6A Module Management scenarios covering shell loading/failure, directory/detail parity, policy mutation, desktop/mobile navigation, no-JavaScript SSR, and bridge-request exclusion | Global `modules:read`, manage-implies-read, scoped-only fail-closed behavior, product-only/no-access denial, `admin:all`, fixed `Admin` item independence, read-only versus mutable controls, exact GET/PUT outcomes, and failure-safe policy restoration. |

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
| `pw-modules-*-reader/manager/scoped-reader/product-only/no-access@tessara.local` | Created by `modules.spec.ts` | Exercises isolated global read, global manage without a stored read row, directly scoped module authority, product-only authority, and no authority. The suite restores the original navigation policy in `afterAll` and removes only `pw-modules-*` fixtures. |
| `operator@tessara.local`, `respondent@tessara.local`, `delegator@tessara.local` | Not used directly by current Playwright specs | Their behaviors are covered through Playwright-owned accounts with equivalent capability/scope/delegation fixtures, avoiding dependence on durable seeded demo account shape. |

## Capability Coverage Matrix

| Capability family | Current Playwright status | Needed future coverage |
| --- | --- | --- |
| Login/session | Covered | Keep session metadata assertions current when capability, scope, or delegation payloads change. |
| Administration | Covered for current users, roles, user detail/edit/access alias, and node-type route/API checks | Future New User Screen flow remains pending because the screen does not exist yet. |
| Forms | Covered for scoped list/detail visibility, direct out-of-scope denial, create/edit route checks, and scoped manage API containment | Deeper drag/drop form-builder authoring interactions can be added as UI-specific coverage. |
| Workflows | Covered for scoped candidates, assignees, assignment creation denial, assignment list filtering, start denial/allowance, create/detail/edit route checks, and scoped manage API containment | More detailed revision authoring UI interactions can be added as the workflow editor matures. |
| Submissions | Covered for scoped management, own response ownership, delegated work, unrelated out-of-scope denial, and response edit route ownership denial/allowance | Deeper response save/submit UI paths can be added with a purpose-built form fixture. |
| Datasets | Covered for visibility-scope list/detail/table access | Explicit dataset restriction filters/rules once advanced authoring supports them. |
| Components | Covered for Dataset major-line-backed list/detail/table visibility, scoped manageable admin routes, public reader published-only behavior, and out-of-scope bind/validate/publish denials | Deeper browser-native authoring interactions for advanced filters, sorting, pagination, aggregate post-filters, and richer column controls as those controls mature. |
| Dashboards | Covered for all five native routes, visibility-scope list/detail/viewer access, scoped create/update containment, composition add/direct move/resize/remove/save, dirty preview gating, exact-version execution, published-history placement boundaries, and redacted-placement isolation | Add richer multi-placement collision and pointer-resize browser cases if those interactions gain additional policies beyond the shared placement-editor contract. |
| Module Management | Covered by five executable scenarios in `modules.spec.ts`; the durable acceptance manifest contains 60 tests across 7 files | Keep the global-read/manage/scoped/product-only/no-access/`admin:all` matrix, fixed `Admin` item, read-only/mutable controls, Core-item immutability, desktop/mobile ordering, SSR/no-JS parity, seven-descriptor human/machine detail parity, and policy-restoring teardown current as the control plane evolves. Runtime/Module Instance scenarios begin in Sprint 6B. |

## Future Comment-Only Scaffold Targets

The Playwright specs should carry TODO comments for these scenarios without adding `test.skip` placeholders:

- Forms: admin create/publish/read; scoped operator list/read overlap; direct out-of-scope form read denial; future scoped create/edit containment.
- Workflows: scoped operator sees overlapping available-node workflows; cannot assign/start out-of-scope workflow work; candidates and assignees are scope-filtered.
- Submissions: respondent own work; delegate delegated work; delegator/delegate context remains ownership/delegation-based; scoped operator can review only in-scope submissions.
- Datasets/components: dataset visibility and future explicit restriction rules; component visibility inherited from the bound Dataset major line. Dashboard published-history placement and redaction behavior are now executable browser scenarios.
- Administration: admin-only Administration item/routes; future New User Screen. Do not generalize this to the entire `Admin` group: the fixed Module Management item has independent global `modules:read` eligibility and executable coverage in `modules.spec.ts`.

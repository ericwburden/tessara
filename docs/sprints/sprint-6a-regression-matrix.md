# Sprint 6A Frozen Regression Matrix

This is the authoritative Sprint 6A inventory of existing user-facing route families and representative behavior that must remain unchanged while the shell and control-plane boundaries are refactored. It replaces “every current primary route” as an acceptance phrase; it does not replace deeper existing feature tests.

Kickoff inspection found one pre-existing route-ownership discrepancy: the Leptos route tree declares `/datasets/:dataset_id/preview` and `/datasets/:dataset_id/revisions/:revision_id/edit`, but the Axum document router at kickoff commit `3625d4de52c5856e4ac3bc642a9422a029e9f375` does not register those two direct-load paths. Their characterization tests are expected to start red. Sprint 6A must restore the missing native document mappings before the shell/navigation refactor and then keep the tests durable; a 404 is not accepted as the frozen baseline and the matrix must not be weakened to hide it. This restores the already-declared screen ownership and introduces no new product workflow.

## Rules Applied To Every Route

For every route below that the fixture actor is authorized to use:

- direct load and browser refresh preserve the current path and native Leptos SSR ownership;
- the native document response and JavaScript-disabled state match the characterized pre-Sprint-6A behavior, remain safe and accurate, and never falsely present hydrated product data as server-rendered; Sprint 6A does not silently convert existing hydrate-dependent screens into new data-complete no-JavaScript experiences;
- hydration reuses the authorized server projection without mismatch or an unsolicited duplicate initial load;
- the browser console has no uncaught error or hydration warning;
- no `/bridge/*`, HTML-string route shell, `inner_html` route injection, or legacy bridge asset is requested;
- desktop and mobile preserve every pre-Sprint-6A shell item, label, group, relative order, and actor-specific visibility until an administrator explicitly changes contribution policy; the sole additive item is fixed Core Module Management in `Admin`, after Datasets, for actors with effective global `modules:read`;
- loading, populated, empty/no-results, validation, restricted, unavailable, not-found, and server-error states remain distinct where applicable;
- keyboard focus, accessible names, live feedback, and established responsive/no-horizontal-overflow behavior do not regress; and
- an unauthorized actor receives the same existing redirect/restricted/error behavior regardless of navigation visibility.

“Pass” requires current behavioral assertions, not only HTTP 200 or the presence of a page heading. Existing API, unit, Playwright, smoke, and UAT assertions remain authoritative and cannot be weakened without the Sprint 6A test-change record.

## Core And Session Routes

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/` | Assigned-work Home loads for the signed-in actor; permanent Home navigation remains; existing capability-filtered links and session context remain correct | Existing web/app Playwright assertions plus admin/operator/respondent navigation characterization |
| `/login` | Bare root-level login has no authenticated app shell; valid/invalid sign-in and session redirect behavior remain unchanged | Web login/session tests and `app.spec.ts` |
| `/operations` | `operations:view` remains the route/API authority; scoped readiness/assignment data remains filtered; no Module Definition identity or mutable product action is introduced | Existing Operations API/UI/permissions coverage and route matrix browser pass |

## Organization

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/organization` | Scoped tree/list data, search/selection, related work, create affordance, and out-of-scope omission | Existing hierarchy API tests and permissions/browser coverage |
| `/organization/new` | Current manage/scope checks, metadata options, validation, create/cancel effects | Exact `hierarchy:read` metadata-schema endpoint authority/shape tests plus create API/UI positive and negative coverage; browser proof renders a required metadata control without console errors |
| `/organization/:node_id` | In-scope detail/related work renders; unknown/out-of-scope behavior does not disclose or enable mutation | Existing route/API/permissions coverage |
| `/organization/:node_id/edit` | Current edit validation, cycle/scope constraints, metadata values, save/cancel semantics | Exact metadata-schema endpoint and hierarchy regression tests plus permissions coverage that renders the in-scope node name/control/value cleanly while retaining non-disclosing out-of-scope behavior |

## Forms

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/forms` | Scoped directory/search, published summaries, read/manage actions, and no out-of-scope rows | Existing Forms/API/permissions coverage |
| `/forms/new` | Form/section/field authoring, grid behavior, validation, create/cancel | Existing Forms crate/API/Playwright coverage |
| `/forms/:form_id` | Form/version/section detail and unavailable/not-found behavior | Existing Forms route/API coverage |
| `/forms/:form_id/edit` | Draft save, field/section editing, publish, attachment/assignment shortcut behavior, scope authorization | Existing Forms unit/API/permissions/browser coverage |

## Workflows

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/workflows` | Scoped workflow directory, assignment summaries, read/manage actions | Existing workflow runtime/UI/permissions tests |
| `/workflows/new` | Explicit ordered-step authoring, availability selection, validation, create/cancel | Existing workflow API/browser tests |
| `/workflows/assignments` | Candidate filtering, assignee/node targeting, assign/start/unassign behavior and scope denials | Workflow-mediated assignment and permissions specs |
| `/workflows/:workflow_id` | Detail, revision/step/assignment tables, scoped unavailable/not-found behavior | Existing workflow API/UI tests |
| `/workflows/:workflow_id/edit` | Draft/update/publish rules, ordered steps, availability/scope constraints | Existing workflow-runtime and authoring tests |

## Responses

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/responses` | Own/delegated/scoped response visibility, filters, status/actions | Existing submission API/UI/permissions tests |
| `/responses/new` | Assignment-only start choices and start behavior; no manual form/node start mode returns | Existing workflow-mediated start tests |
| `/responses/:submission_id` | Draft/submitted detail, ownership/delegation/scope visibility, answer/audit rendering | Existing response lifecycle/permissions tests |
| `/responses/:submission_id/edit` | Draft-only save/submit, answer serialization/validation, ownership denial, submitted read-only guard | Existing API/UI/Playwright response coverage |

## Datasets

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/datasets` | Scoped directory/search/status/actions and restricted/confidential visibility semantics | Existing Dataset API/UI/permissions tests |
| `/datasets/new` | Source selection, definition validation, scope/visibility, create/cancel | Existing Dataset authoring tests |
| `/datasets/:dataset_id` | Detail, published/draft state appropriate to authority, dependencies/materialization summary | Existing Dataset API/UI tests |
| `/datasets/:dataset_id/preview` | Server-backed scoped preview, restriction-tier enforcement, stable pending/failed states | Existing preview/permission coverage |
| `/datasets/:dataset_id/revisions` | Revision history, draft visibility, dependency summaries, scoped redaction | Existing revision tests |
| `/datasets/:dataset_id/revisions/:revision_id` | Authorized revision detail and stable unavailable/forbidden behavior | Existing revision API/UI tests |
| `/datasets/:dataset_id/revisions/:revision_id/edit` | Draft edit/validate/publish and scope/dependency guards | Existing revision authoring/permissions tests |
| `/datasets/:dataset_id/edit` | Dataset metadata/scope edit and incompatible-scope rejection | Existing Dataset edit tests |

## Components

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/components` | Public/current-published directory, manage-state actions, scoped filtering | Existing Component API/UI/permissions tests |
| `/components/new` | Typed authoring, Dataset major-line binding, validation, save/publish | Existing Component spec/API tests |
| `/components/:component_ref` | Current published detail plus authorized authoring state | Existing Component route/API tests |
| `/components/:component_ref/edit` | Draft lifecycle, validation, publish/update/new-version decisions, immutable superseded behavior | Existing Component authoring tests |
| `/components/:component_ref/versions` | Draft/published/superseded history and capability-aware actions | Existing version route tests |
| `/components/:component_ref/view` | Exact/current version execution, table/visual controls, scoped data, stable pending/failed/forbidden outcomes | Existing Components and permissions Playwright specs |

## Dashboards

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/dashboards` | Read-scoped directory/search and reader navigation; manage-only does not gain the reader link | Existing Dashboard API/UI/nav characterization |
| `/dashboards/new` | Scoped create and Component selection constraints | Existing Dashboard permissions/browser coverage |
| `/dashboards/:dashboard_id` | Metadata, visibility, placement counts, capability-aware actions | Existing Dashboard API/UI tests |
| `/dashboards/:dashboard_id/edit` | Atomic composition, opaque stable placement IDs, move/resize/reflow, dirty preview, redacted rows, scope guards | Existing Dashboard composition unit/API/Playwright tests |
| `/dashboards/:dashboard_id/view` | Exact pinned versions, lazy execution, isolated errors, redacted footprints, responsive order | Existing Dashboard viewer/permissions tests |

## Administration

Sprint 6A adds Module Management but does not redesign or regroup existing Administration work.

| Routes | Frozen representative behavior | Required proof |
| --- | --- | --- |
| `/administration` | Existing Users, Node Types, and Roles entries/actions remain; Module Management is the sole new landing entry for this still-`admin:all`-only route. The landing link supplements the standalone shell item and is not the discovery path for a `modules:read`-only actor | Existing app/admin route test plus new modules spec |
| `/administration/users` | User directory/search/actions and admin-only API behavior | Existing admin/permissions tests |
| `/administration/users/:account_id` | User detail and current account/access information | Existing admin route/API tests |
| `/administration/users/:account_id/edit` | Account update validation and authorization | Existing admin tests |
| `/administration/users/:account_id/access` | Existing role/scope/delegation management and effective-capability behavior | Existing permissions/admin tests |
| `/administration/node-types` | Node-type CRUD, relationship/metadata rules, admin-only behavior | Existing node-type API/UI tests |
| `/administration/roles` | Existing Core-owned role create/edit behavior; Sprint 6A adds read-only capability provenance/provider state only | Existing role tests plus provenance isolation tests |

## New Additive Sprint 6A Routes

These are not frozen baseline behavior; they must satisfy the Sprint 6A plan without changing the routes above.

| Routes | Required behavior/proof |
| --- | --- |
| `/administration/modules` | Global `modules:read`; native authenticated SSR; exact inventory/source-digest parity with API; current navigation policy readable for read-only actors; no enabled show/hide/reorder affordance without effective global `modules:manage_navigation`; populated/loading/empty/error/restricted states; no-JS/hydration/accessibility/responsive/no-bridge coverage |
| `/administration/modules/:definition_id` | Global `modules:read`; all seven details preserve exact API/bootstrap/DOM parity for reserved definition ID, source digest, display name, description, and availability; every ordered feature's `id`, `name`, `description`, `use_cases`, `inputs`, `outcomes`, `constraints`, `contracts`, `resource_types`, `destinations`, `capabilities`, and `configuration_pointers`; every contract's `id`, `version`, `kind`, and `description`; every dependency's `contract_id`, `version_requirement`, `binding_key`, and `optional`; every resource's `id` and `description`; every route's `name`, `kind`, optional `resolved_path`, and ordered parameter `name`, `value_type`, and `required`; every navigation declaration's `id`, `destination`, `label`, `group`, `order_hint`, and ordered `required_capabilities_any_of`; every capability's `id` and `description`; and every finding's `code`, `path`, and `message` in stable order. API/bootstrap preserve the exact configuration schema and the DOM proves its exact `Declared`/`Not declared` state; stable unknown/unavailable behavior; no fake Release/Instance controls |

## Additive Shell Contract

`Module Management` is the only new Sprint 6A shell item. It has stable key `module_management`, route `/administration/modules`, group `Admin`, and a fixed default slot after Datasets. It is a permanent Core item: it is absent from mutable navigation-policy members and cannot be hidden, reordered, regrouped, or used as a contribution-band target. The `Admin` group must render when Module Management is its only visible item.

| Actor authority | Module Management item and `Admin` group | Read surfaces | Mutation controls / policy `PUT` |
| --- | --- | --- | --- |
| Global `modules:read` only | Visible | Directory, detail, descriptor, and current policy readable | No enabled mutation affordance; direct `PUT` is `403 modules_manage_navigation_global_required` |
| Global `modules:manage_navigation` only as stored | Visible through manage-implies-read | Readable | Controls enabled; valid `PUT` allowed |
| `admin:all` | Visible through implication | Readable | Controls enabled; valid `PUT` allowed |
| Scoped-only module capability | Hidden; does not make the `Admin` group visible | Restricted | `403`; no controls |
| Product-only or authenticated no-access | Hidden unless another independently eligible Admin item exists | Restricted | `403`; no controls |
| Anonymous | No authenticated shell | Login/`401` behavior | `401`; no controls |

## Actor Matrix

The pre-refactor and post-refactor suites use named isolated fixtures for:

- `admin:all`;
- operator/scoped manager with the current product capabilities;
- respondent with own/delegated response capabilities;
- product manage-only actors, including Dashboard manage without read;
- global `modules:read` only;
- global `modules:manage_navigation` without a separately stored read row;
- scoped-only module capability assignment, which must not satisfy global module administration;
- product capability with no module administration; and
- authenticated no-access plus anonymous browser/API cases.

Navigation-policy tests restore the original policy in failure-safe teardown so one failed hide/reorder scenario cannot alter later regression evidence.

The fixed-item suite also attempts to submit `module_management` as a mutable member and requires atomic `navigation_policy_core_item_immutable` rejection with no revision, policy, ordering, or audit-success mutation. Desktop, collapsed, and mobile-overlay shell tests use the same actor matrix and assert the same item eligibility and Admin-group behavior.

# Tessara UI Screen Inventory

This inventory is the migration map for the native Leptos SSR reset worktree. The reference worktree is `C:\Users\eric-dev\Projects\tessara`; this reset worktree is treated as a new application that only migrates functional/domain code forward intentionally.

| Old path | New path | Nav section | Data/API dependencies | Actions | Overlays | Tables/forms | Functional code worth preserving |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `/app` | `/` | Main | `/api/auth/session`, `/api/me`, `/api/summary` | Navigate to product and administration routes | None initially | Route inventory table/cards | Auth/session DTOs and route inventory behavior |
| `/app/login` | `/login` | Session | `/api/auth/login`, `/api/auth/session`, `/api/auth/logout` | Sign in, sign out, session verification | Auth feedback | Login form | Auth request/response contracts and session bootstrap behavior |
| `/app/organization` | `/organization` | Main | `/api/hierarchy`, `/api/node-types`, `/api/forms`, `/api/submissions`, `/api/dashboards` | Create node, details, edit, create child | Sheet for node details, dropdown row menus | Nested collapsibles, info-list tables, related-work accordions | Hierarchy DTOs, node relationship helpers, metadata label handling, RFC2822 timestamp formatting |
| `/app/organization/new` | `/organization/new` | Main | `/api/admin/nodes`, `/api/node-types` | Create node, cancel | Validation feedback | Node create form | Node payload assembly and metadata field handling |
| `/app/organization/:node_id` | `/organization/:node_id` | Main | `/api/hierarchy/{node_id}` plus related work APIs | Details, edit, create child | Sheet | Info-list tables, related accordions | Related work aggregation and metadata labels |
| `/app/organization/:node_id/edit` | `/organization/:node_id/edit` | Main | `/api/admin/nodes/{node_id}` | Save, cancel | Validation feedback | Node edit form | Existing validation and patch payload behavior |
| `/app/forms` | `/forms` | Main | `/api/forms`, `/api/node-types` | Create form, view, edit | None initially | Data table | Form DTOs, version summary formatting |
| `/app/forms/new` | `/forms/new` | Main | `/api/admin/forms`, `/api/node-types` | Create form, add fields, cancel | Field editing drawer if needed | Form builder | Field schema helpers and validation rules |
| `/app/forms/:form_id` | `/forms/:form_id` | Main | `/api/forms/{form_id}` | Edit, open published versions | Sheet/drawer only when needed | Info-list and field table | Form detail DTOs and published version mapping |
| `/app/forms/:form_id/edit` | `/forms/:form_id/edit` | Main | `/api/admin/forms/{form_id}` | Save draft, publish, cancel | Field editor | Form builder | Draft version editing behavior |
| `/app/workflows` | `/workflows` | Main | `/api/workflows`, `/api/workflow-assignments` | Create workflow, assign, view, edit | None initially | Data table | Workflow DTOs, assignment summary helpers |
| `/app/workflows/new` | `/workflows/new` | Main | `/api/admin/workflows` | Create, cancel | Validation feedback | Workflow form | Workflow payload helpers |
| `/app/workflows/assignments` | `/workflows/assignments` | Main | `/api/workflow-assignments`, `/api/hierarchy` | Assign, unassign, filter | Assignment sheet | Assignment table/form | Assignment filtering and node targeting |
| `/app/workflows/:workflow_id` | `/workflows/:workflow_id` | Main | `/api/workflows/{workflow_id}` | Edit, manage assignments | None initially | Info-list, revisions table | Workflow revision lifecycle behavior |
| `/app/workflows/:workflow_id/edit` | `/workflows/:workflow_id/edit` | Main | `/api/admin/workflows/{workflow_id}` | Save, cancel | Validation feedback | Workflow edit form | Existing workflow update logic |
| `/app/responses` and `/app/submissions` | `/responses` | Main | `/api/submissions`, `/api/workflow-assignments` | Start response, view, edit draft | None initially | Data table | Submission list DTOs, delegate filtering, RFC2822 timestamps |
| `/app/responses/new` | `/responses/new` | Main | `/api/submissions`, `/api/forms` | Start, save draft, submit | Validation feedback | Response form | Response payload and field rendering behavior |
| `/app/responses/:submission_id` | `/responses/:submission_id` | Main | `/api/submissions/{submission_id}` | Edit draft, back to list | None initially | Info-list, answer table | Response detail rendering and submitted/draft guardrails |
| `/app/responses/:submission_id/edit` | `/responses/:submission_id/edit` | Main | `/api/submissions/{submission_id}` | Save draft, submit | Validation feedback | Response edit form | Existing answer serialization |
| `/app/components` | `/components` | Main | `/api/dashboards`, `/api/charts` | View component | None initially | Data table | Component reference parsing |
| `/app/components/:component_ref` | `/components/:component_ref` | Main | `/api/charts`, `/api/dashboards` | Back to list | None initially | Detail panel | Component lookup helpers |
| `/app/dashboards` | `/dashboards` | Main | `/api/dashboards`, `/api/charts` | Create dashboard, view, edit | None initially | Data table/cards | Dashboard DTOs and component count helpers |
| `/app/dashboards/new` | `/dashboards/new` | Main | `/api/admin/dashboards`, `/api/charts` | Create, cancel | Component selector drawer | Dashboard form | Dashboard payload helpers |
| `/app/dashboards/:dashboard_id` | `/dashboards/:dashboard_id` | Main | `/api/dashboards/{dashboard_id}` | Edit, inspect components | Sheet/drawer if needed | Dashboard component list | Dashboard composition logic |
| `/app/dashboards/:dashboard_id/edit` | `/dashboards/:dashboard_id/edit` | Main | `/api/admin/dashboards/{dashboard_id}` | Save, add/remove component | Component selector drawer | Dashboard edit form | Component ordering and update behavior |
| `/app/datasets` | `/datasets` | Admin | Dataset registry APIs | View dataset | None initially | Data table | Dataset descriptor contracts |
| `/app/datasets/:dataset_id` | `/datasets/:dataset_id` | Admin | Dataset registry APIs | Back to list | None initially | Info-list/table | Dataset detail formatting |
| `/app/administration` | `/administration` | Admin | `/api/admin/capabilities` | Open users, node types, roles | None initially | Link grid | Capability DTOs |
| `/app/administration/users` | `/administration/users` | Admin | `/api/admin/users`, `/api/admin/roles` | Create, edit, manage access | Access sheet | User table/forms | User and role assignment DTOs |
| `/app/administration/node-types` | `/administration/node-types` | Admin | `/api/admin/node-types`, relationship and metadata APIs | Create, edit, manage relationships | Relationship and metadata sheets | Node type table/forms | Node type relationship and metadata contracts |
| `/app/administration/roles` | `/administration/roles` | Admin | `/api/admin/roles`, capabilities API | Create, edit | Capability selector | Role table/forms | Capability grouping helpers |
| `/app/migration` | `/migration` | Admin | `/api/admin/legacy-import/*` | Inspect fixtures, run import | Import confirmation | Fixture tables/forms | Legacy import DTOs and validation behavior |
| `/app/reports` | Pending scope | Pending | Reporting APIs | Create, run, edit | Report builder | Report tables | Keep reporting domain code parked until product scope is confirmed |

## Migration guardrails

- Active routes start as native Leptos components returning `impl IntoView`.
- No route migrates string-rendered UI, transitional shell code, or broad feature files.
- `#app-overlays` is a direct child of `body` before `#app-root`.
- Tailwind 4 is the styling target; Bulma assets and shell CSS have been removed from the active reset baseline.
- Lists default to shared table primitives unless a route has a specific UX reason to diverge.

## UI-deprecated DTO fields to review

Track DTO fields that remain in API payloads during the UI refresh but are no longer surfaced by the native UI. Do not remove these during the UI-focused reset pass unless we explicitly decide to make a data-model/API cleanup.

| DTO / payload area | Field | Observed route | UI decision | End-of-refresh follow-up |
| --- | --- | --- | --- | --- |
| Rendered form sections | `column_count` | `/forms/:form_id`, `/forms/new`, `/forms/:form_id/edit` | Do not display section column counts; reset authoring uses a fixed 12-column canvas and per-field grid row, column, width, and height. | Review `CreateFormSectionRequest.column_count`, `UpdateFormSectionRequest.column_count`, `FormSectionResponse.column_count`, and `form_sections.column_count` for compatibility, migration, or persisted rendering needs before removing or formally deprecating the field. |
| Workflow metadata | `form_id` | `/workflows/new`, `/workflows/:workflow_id/edit` | Do not expose a workflow-level linked form in the reset UI; forms belong to ordered workflow-version steps instead. | After the UI reset, review `CreateWorkflowRequest.form_id`, `UpdateWorkflowRequest.form_id`, `WorkflowSummary.form_id`, and `workflows.form_id` for compatibility-only removal or formal deprecation. |
| Workflow scope | `scope_node_type_id` | `/workflows/new`, `/workflows/:workflow_id/edit`, `/workflows/assignments` | The reset UI now treats workflow node type as the workflow scope. Step form versions are filtered to forms scoped to the workflow node type or a descendant node type. | This is the intended replacement for form-version-to-node assignment coupling. Later DTO cleanup should rename this explicitly as workflow scope and decide whether it becomes required at the database/API boundary. |
| Workflow revision authoring | `form_version_id`, `title` | `/workflows/new`, `/workflows/:workflow_id/edit` | Do not expose the single-form workflow-version fallback in reset authoring; users author explicit ordered steps, and each step selects its own form version. The `workflow_versions` storage and `CreateWorkflowVersionRequest` names remain deprecated compatibility language during this pass. | Review `CreateWorkflowVersionRequest.form_version_id`, `CreateWorkflowVersionRequest.title`, fallback step creation in workflow handlers, and `workflow_versions.form_version_id` once workflow routes are fully step-first. |
| Workflow revision read models | `WorkflowVersionSummary.form_version_label` | `/workflows/:workflow_id`, `/workflows/:workflow_id/edit` | The reset UI treats this payload value as the workflow revision label and displays integer revisions, including legacy labels such as `1.0.0` as revision `1`. | Rename the API response field to `workflow_revision_label` and update clients after the reset pass; keep the current field during the UI refresh for compatibility. |
| Workflow read models | `current_form_version_id` | `/workflows`, `/workflows/:workflow_id` | Do not present a single current form version as the workflow identity; render workflow form context from version steps. | Review `WorkflowSummary.current_form_version_id` and detail/version response shape after workflow detail/edit routes are rebuilt. |

Sweep note: on 2026-05-13, the reset and reference worktrees were searched for `compatib`, `deprecated`, `legacy`, `form_id`, `form_version_id`, `scope_node_type_id`, and `column_count`. `scope_node_type_id` is tracked as a future product/model question below rather than a UI-deprecated field because the reset UI still surfaces form scope today. Form version compatibility group fields remain active versioning concepts, not deprecated UI fields from this sweep.

Workflow history note: during the reset pass, workflow history is product-facing "revision" language even though API paths, DTO types, and database tables still use `version` names for compatibility. Step changes create a new integer workflow revision label; workflow name, node type, and description changes update metadata without creating a new revision.

Workflow scope note: workflows are now scoped to a node type, and workflow assignments happen against concrete nodes of that type. Form versions are not directly tied to concrete nodes for workflow compatibility; workflow steps may use form versions whose form scope is the workflow node type or a descendant. Future workflow-step metadata/context passing can decide how a lower-level step targets a specific descendant node.

## UI reset follow-ups

Track workflow gaps discovered during the route rebuild without changing the data model during the UI-focused reset pass.

| Area | Route observed | Gap | Follow-up |
| --- | --- | --- | --- |
| Form node attachment | `/forms/:form_id/edit` | The reset UI can display current attached organization nodes from existing form assignment data, but the edit route does not yet provide a native workflow to attach or detach a form from nodes. | Decide the intended attach/detach UX and API contract after the UI pass, then implement without guessing at data-model changes during reset. |
| Assignment source of truth | `/forms/:form_id/edit`, `/workflows/assignments` | Forms can feel directly assigned to users while the current model also has direct `form_assignments` plus first-class `workflow_assignments` with optional linkage back to form assignments. Workflow compatibility should no longer depend on `form_assignments`; they are transitional legacy delivery records. | After the UI reset, evaluate a single workflow assignment source of truth that preserves the simple "assign one form" user experience without maintaining parallel assignment concepts unnecessarily. |
| Form scope semantics | `/forms/new`, `/forms/:form_id/edit` | Current form payloads include a fixed scope node type, but a form can be legitimately usable in multiple node contexts. A form may tend to make sense in a context without being intrinsically about that context. | After the UI reset, reconsider whether form scope should move from the form definition to workflow-step or assignment compatibility rules, allowing one form to be reused across valid contexts without duplication. |
| User permissions source of truth | `/administration/users/:account_id/access`, `/administration/users/:account_id/edit`, `/administration/roles` | The reset UI currently reflects the compatibility API where a user's effective capabilities are derived from assigned roles. This is not the intended long-term model. A Role should be a database entity that consists only of a collection of individual capabilities and acts as a convenient template for applying capabilities to users. A user's permissions should be based on their assigned capabilities, not statically sourced from roles. | After the UI reset, redesign user permission storage/API so users have direct capability assignments, roles can apply capability templates, and the UI can clearly show capability drift: capabilities a user has outside their role template and role-template capabilities the user does not have. |
| Migration consolidation | Application/database setup | The reset pass has accumulated compatibility migrations while the product model is still settling. | After the UI refresh, tear down the reset application/database, rebuild from the final intended schema, and consolidate the SQL migrations into a cleaner baseline before carrying the app forward. |

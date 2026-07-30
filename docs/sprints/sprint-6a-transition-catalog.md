# Sprint 6A Transition Catalog Contract

This document freezes the human-readable identifiers and current-route mappings for the seven Sprint 6A transition contributions. The checked-in JSON source descriptors are the machine-readable authority and must match this document exactly. A mismatch blocks persistence, API, and UI work.

## Global Rules

- Every source descriptor uses `schema_version: 1`. When Core projects that descriptor into module inventory, the enclosing inventory entry uses discriminator `transitional_in_process`; the discriminator is not duplicated inside the descriptor source document.
- Forms, Workflows, Responses, Datasets, Components, and Dashboards use `availability: active_in_process`, meaning their declared current surfaces execute in the shared Core process. `unavailable` is reserved for an intended current transition surface that retains declarations but temporarily cannot execute. Migration uses `availability: retired`, meaning it was deliberately withdrawn and has no live declarations; it is historical/support discovery, not an executable provider, and restoration requires a new product decision.
- Every provided contract version is `1.0.0`; every declared current dependency requires `^1.0`.
- Transition descriptors are never provider candidates. Every current in-process dependency below projects `transition_internal_only`, even when its contract ID/version text matches a transition contribution.
- Every resource type is owned by the current `core_installation`. No source descriptor contains a Module Release, Module Instance, artifact, deployment, install, enablement, health, or Supervisor field.
- Every current semantic route declaration uses `kind: product`. Every parameter shown in the route registry is `required: true`; a route with no shown parameters has an empty parameter array. Route kind is discovery metadata and does not replace the existing route/API guard.
- Semantic route declarations contain stable names and typed parameters only. The Core registry owns the relative path mapping. Source descriptors and persisted product data contain no scheme, host, port, origin, or deployment-relative URL.
- A navigation contribution's `required_capabilities_any_of` controls display eligibility only. Core evaluates `admin:all` implication separately. Route/API guards remain authoritative.
- The checked-in UTF-8/LF source bytes are hashed as lower-case `sha256:<64 hex>`. Each sidecar must contain exactly that value plus one LF, but the sidecar is not its own authority: executable Rust constants and the deployment-evidence validator independently pin every expected digest and reject a coordinated source-plus-sidecar rewrite.
- Each active feature has non-empty canonical `name`, `description`, `use_cases`, `inputs`, `outcomes`, and `constraints` plus exact realizing contract/resource/destination/capability arrays in its checked-in source descriptor. Those source fields and their array order are machine authority even where this summary uses shorter prose. Empty narrative arrays are not an acceptable placeholder. Configuration pointers remain empty unless a real current configuration contract is identified.
- `transition_internal_only`, normalized `core_installation` resource ownership, and `transition_destination_retired` are Core catalog-projection values, not extra fields in descriptor-source JSON. Their focused projection tests land with the catalog projection; descriptor fixture tests must not claim that source parsing alone proves them.

## Identity And Availability Summary

| Display name | Reserved definition ID | Availability | Navigation ID | Expected normalized finding |
| --- | --- | --- | --- | --- |
| Forms | `tessara.forms` | `active_in_process` | `tessara.forms.navigation` | None solely from transition availability |
| Workflows | `tessara.workflows` | `active_in_process` | `tessara.workflows.navigation` | Declared dependencies are `transition_internal_only` |
| Responses | `tessara.responses` | `active_in_process` | `tessara.responses.navigation` | Declared dependencies are `transition_internal_only` |
| Datasets | `tessara.datasets` | `active_in_process` | `tessara.datasets.navigation` | Declared dependencies are `transition_internal_only` |
| Components | `tessara.components` | `active_in_process` | `tessara.components.navigation` | Declared dependency is `transition_internal_only` |
| Dashboards | `tessara.dashboards` | `active_in_process` | `tessara.dashboards.navigation` | Declared dependency is `transition_internal_only` |
| Migration | `tessara.migration` | `retired` | None | `transition_destination_retired` |

## Canonical Source Artifacts

These seven documents—and only these seven—are the persistence synchronization inputs. The adjacent `.sha256` sidecars contain the expected exact-byte digest. The smaller generic valid/invalid fixtures in the same test directory are conformance examples, not catalog sources.

| Definition | Authoritative source | Expected exact-byte digest |
| --- | --- | --- |
| Forms | [`transition-forms-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-forms-v1.json) | `sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e` |
| Workflows | [`transition-workflows-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-workflows-v1.json) | `sha256:e9bdf51896700ffb982a00e4c80ea198bbdb98056705036a1a948347a71c04cf` |
| Responses | [`transition-responses-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-responses-v1.json) | `sha256:e491986ed43b0f290f0c2ee763e60afb03e5b7babc7117a11e280e37de7b91bc` |
| Datasets | [`transition-datasets-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-datasets-v1.json) | `sha256:ca301f4ac9a589d498bc25c77de4223b33de90569ecf54974976424c07fb4614` |
| Components | [`transition-components-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-components-v1.json) | `sha256:344388304b015421ea71b5e303e7b9699264aef51c116b56d7f52e1b92443499` |
| Dashboards | [`transition-dashboards-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-dashboards-v1.json) | `sha256:c82ecc7c3d121d1e1498c130133e487c8a68899b9255951e97955ce0de76bbe5` |
| Migration | [`transition-migration-v1.json`](../../crates/tessara-module-contract/tests/fixtures/transition-migration-v1.json) | `sha256:de48eeb3edb4a432e5060b817ef50c34c5316879b44aef0ad3d6877c5895b42e` |

The current Module Manifest is a conformance fixture, not a transition synchronization input. Sprint 6D fast-forwarded the former v1 fixture to [`valid-manifest.json`](../../crates/tessara-module-contract/tests/fixtures/valid-manifest.json) and its adjacent exact-digest sidecar. The contract test rejects BOM, CR, NUL, invalid UTF-8, missing or repeated terminal LF, sidecar drift, and source-byte drift for that Manifest and every transition fixture.

## Default Shell Contract

The default composed model preserves every pre-Sprint-6A item, its relative order, and its actor-specific visibility before an administrator changes contribution policy. Sprint 6A adds exactly one shell item: the permanent Core Module Management destination in `Admin`, appended after Datasets for actors with effective global `modules:read`.

| Group/default slot | Item | Owner | Policy mutable | Core-assigned reorder band | Display eligibility |
| --- | --- | --- | --- | --- | --- |
| `Main` / 0 | Home | Core | No | Core anchor | Authenticated shell |
| `Main` / 10 | Organization | Core | No | Core anchor | `hierarchy:read` or `hierarchy:manage` |
| `Main` / 20 | Forms | transition contribution | Visibility/order only | `main_between_organization_and_operations` | `forms:read` or `forms:manage` |
| `Main` / 30 | Workflows | transition contribution | Visibility/order only | `main_between_organization_and_operations` | `workflows:read` or `workflows:manage` |
| `Main` / 40 | Responses | transition contribution | Visibility/order only | `main_between_organization_and_operations` | `submissions:read_own`, `submissions:respond`, or `submissions:manage` |
| `Main` / 50 | Operations | Core | No | Core anchor | `operations:view` |
| `Main` / 60 | Components | transition contribution | Visibility/order only | `main_after_operations` | `components:read` or `components:manage` |
| `Main` / 70 | Dashboards | transition contribution | Visibility/order only | `main_after_operations` | `dashboards:read`; `dashboards:manage` alone does not show the reader directory |
| `Admin` / 10 | Administration | Core | No | Core anchor | `admin:all` |
| `Admin` / 20 | Datasets | transition contribution | Visibility/order only | `admin_between_administration_and_module_management` | `datasets:read` or `datasets:manage` |
| `Admin` / 30 | Module Management (`module_management`) | Core | No | Core anchor | effective global `modules:read`; `modules:manage_navigation` and `admin:all` qualify through implication |

Migration has no shell item. Source descriptors retain their existing group and order hint; Module Management and `reorder_band` are Core-owned normalized shell/catalog metadata, not source-descriptor fields. Contribution group and band values are immutable in Sprint 6A. Policy order is a dense zero-based rank within the assigned band, so no contribution can cross Home, Organization, Operations, Administration, or Module Management. Permanent Core items cannot be hidden or reordered, and contribution policy changes do not affect authorization. Module Management is excluded from policy write members. The `Admin` group renders if it is the actor's only visible Admin item; visibility of the separate Administration item is not a prerequisite.

## Existing Capability Description Contract

Core remains authoritative for every existing capability key and description. Transition synchronization adds provenance only: it never rewrites a Core capability description from descriptor text. Each canonical descriptor must use the exact text below; a mismatch rejects the entire synchronization transaction with `transition_capability_description_mismatch` and leaves the capability row, role mappings, provenance, policy, and audit state unchanged.

| Capability | Exact Core-owned description |
| --- | --- |
| `forms:read` | `Browse top-level form records` |
| `forms:manage` | `Manage form definitions and versions` |
| `workflows:read` | `Browse workflow definitions and assignments` |
| `workflows:manage` | `Manage workflow definitions and assignments` |
| `submissions:read_own` | `Read own and delegated response work` |
| `submissions:respond` | `Start and complete assigned response work` |
| `submissions:manage` | `Manage submissions by hierarchy scope` |
| `datasets:manage` | `Manage dataset definitions` |
| `datasets:read` | `Inspect dataset definitions` |
| `datasets:read_restricted` | `Read restricted dataset rows when dataset visibility allows access` |
| `datasets:read_confidential` | `Read confidential and restricted dataset rows when dataset visibility allows access` |
| `components:manage` | `Manage component definitions` |
| `components:read` | `Inspect component definitions` |
| `dashboards:manage` | `Manage dashboard definitions` |
| `dashboards:read` | `Inspect dashboard definitions` |

## Forms

| Kind | Stable identifiers and links |
| --- | --- |
| Features | `tessara.forms.authoring` → contracts `tessara.forms.form`, `tessara.forms.form-version`; resources `tessara.transition.form`, `tessara.transition.form_version`; destinations `forms.directory`, `forms.create`, `forms.edit`; capabilities `forms:read`, `forms:manage`.<br>`tessara.forms.publication` → contract/resource `tessara.forms.form-version` / `tessara.transition.form_version`; destination `forms.edit`; capability `forms:manage`.<br>`tessara.forms.lookup` → contracts/resources for Form and FormVersion; destinations `forms.directory`, `forms.detail`; capability `forms:read`. |
| Provided contracts | `tessara.forms.form` (`resource`); `tessara.forms.form-version` (`resource`) |
| Dependencies | None |
| Resource types | `tessara.transition.form`; `tessara.transition.form_version` |
| Capabilities | `forms:read`; `forms:manage` |
| Navigation | `tessara.forms.navigation`, label `Forms`, group `Main`, order hint `20`, destination `forms.directory`, any-of `forms:read` / `forms:manage` |

## Workflows

| Kind | Stable identifiers and links |
| --- | --- |
| Features | `tessara.workflows.authoring`; `tessara.workflows.assignment`; `tessara.workflows.execution` |
| Provided contracts | `tessara.workflows.workflow` (`resource`); `tessara.workflows.workflow-version` (`resource`); `tessara.workflows.assignment` (`behavior`) |
| Dependencies | Binding `tessara.workflows.form-version`, requires `tessara.forms.form-version ^1.0`, required, projected `transition_internal_only` |
| Resource types | `tessara.transition.workflow`; `tessara.transition.workflow_version` |
| Capabilities | `workflows:read`; `workflows:manage` |
| Navigation | `tessara.workflows.navigation`, label `Workflows`, group `Main`, order hint `30`, destination `workflows.directory`, any-of `workflows:read` / `workflows:manage` |

Feature realization links are: authoring → Workflow/WorkflowVersion contracts/resources, `workflows.create`/`workflows.edit`, `workflows:manage`; assignment → assignment behavior, `workflows.assignments`, `workflows:read`/`workflows:manage`; execution → WorkflowVersion plus FormVersion dependency, `workflows.detail`, `workflows:read`.

## Responses

| Kind | Stable identifiers and links |
| --- | --- |
| Features | `tessara.responses.start`; `tessara.responses.draft`; `tessara.responses.submit`; `tessara.responses.review` |
| Provided contracts | `tessara.responses.response` (`resource`); `tessara.responses.response-lifecycle` (`behavior`) |
| Dependencies | Binding `tessara.responses.workflow-version`, requires `tessara.workflows.workflow-version ^1.0`, required, `transition_internal_only`.<br>Binding `tessara.responses.form-version`, requires `tessara.forms.form-version ^1.0`, required, `transition_internal_only`. |
| Resource types | `tessara.transition.response` |
| Capabilities | `submissions:read_own`; `submissions:respond`; `submissions:manage` (the legacy/current namespace is intentionally preserved) |
| Navigation | `tessara.responses.navigation`, label `Responses`, group `Main`, order hint `40`, destination `responses.directory`, any-of all three `submissions:*` capabilities above |

Start links to workflow/form dependencies and `responses.start`; draft/submit link to response lifecycle/resource and `responses.edit`; review links to response resource and `responses.detail`. The contribution does not rename capability rows to `responses:*`.

## Datasets

| Kind | Stable identifiers and links |
| --- | --- |
| Features | `tessara.datasets.authoring`; `tessara.datasets.publication`; `tessara.datasets.materialization`; `tessara.datasets.preview` |
| Provided contracts | `tessara.datasets.dataset` (`resource`); `tessara.datasets.dataset-revision` (`resource`); `tessara.datasets.dataset-major-line` (`resource`); `tessara.datasets.materialization` (`behavior`) |
| Dependencies | Binding `tessara.datasets.response`, requires `tessara.responses.response ^1.0`, required, `transition_internal_only`.<br>Binding `tessara.datasets.form-version`, requires `tessara.forms.form-version ^1.0`, required, `transition_internal_only`. |
| Resource types | `tessara.transition.dataset`; `tessara.transition.dataset_revision`; `tessara.transition.dataset_major_line` |
| Capabilities | `datasets:read`; `datasets:manage`; `datasets:read_restricted`; `datasets:read_confidential` |
| Navigation | `tessara.datasets.navigation`, label `Datasets`, group `Admin`, order hint `20`, destination `datasets.directory`, any-of `datasets:read` / `datasets:manage` |

Authoring/publication link to Dataset and DatasetRevision contracts/resources and create/edit/revision destinations. Materialization links to Dataset major-line and materialization contracts. Preview links to Dataset/DatasetRevision/major-line resources and `datasets.preview`. Restricted/confidential read capabilities remain provenance declarations, not independent directory-navigation eligibility.

## Components

| Kind | Stable identifiers and links |
| --- | --- |
| Features | `tessara.components.authoring`; `tessara.components.publication`; `tessara.components.execution`; `tessara.components.viewing` |
| Provided contracts | `tessara.components.component-version` (`resource`); `tessara.components.execution` (`behavior`) |
| Dependencies | Binding `tessara.components.dataset-major-line`, requires `tessara.datasets.dataset-major-line ^1.0`, required, `transition_internal_only` |
| Resource types | `tessara.transition.component_version` |
| Capabilities | `components:read`; `components:manage` |
| Navigation | `tessara.components.navigation`, label `Components`, group `Main`, order hint `60`, destination `components.directory`, any-of `components:read` / `components:manage` |

Authoring/publication link to ComponentVersion, dataset-major-line dependency, create/edit/version destinations, and `components:manage`. Execution/viewing link to exact ComponentVersion plus `components.view` and `components:read`.

## Dashboards

| Kind | Stable identifiers and links |
| --- | --- |
| Features | `tessara.dashboards.composition`; `tessara.dashboards.viewing` |
| Provided contracts | `tessara.dashboards.dashboard` (`resource`); `tessara.dashboards.composition` (`behavior`) |
| Dependencies | Binding `tessara.dashboards.component-version`, requires `tessara.components.component-version ^1.0`, required, `transition_internal_only` |
| Resource types | `tessara.transition.dashboard` |
| Capabilities | `dashboards:read`; `dashboards:manage` |
| Navigation | `tessara.dashboards.navigation`, label `Dashboards`, group `Main`, order hint `70`, destination `dashboards.directory`, any-of `dashboards:read` only |

Composition links to Dashboard, exact ComponentVersion dependency, `dashboards.create`/`dashboards.edit`, and `dashboards:manage`. Viewing links to Dashboard, `dashboards.detail`/`dashboards.view`, and `dashboards:read`.

## Migration

The Migration descriptor contains:

- reserved definition ID `tessara.migration`;
- display name `Migration`;
- `availability: retired`;
- explanatory text that the former Migration surface was deliberately withdrawn and has no current in-process product surface; and
- empty feature, contract, dependency, resource, route, navigation, capability, and configuration declarations.

Its normalized inventory projection emits exactly `transition_destination_retired`. It does not create a provider candidate, restore legacy APIs, or synthesize `/migration`. Returning Migration requires a new approved product decision and roadmap scope; a catalog edit alone is insufficient.

## Semantic Route Registry

All routes below are current same-origin Core paths. UUID parameters use `uuid`; Component references use `string`.

| Contribution | Semantic route → current Core path template |
| --- | --- |
| Forms | `forms.directory` → `/forms`; `forms.create` → `/forms/new`; `forms.detail(form_id: uuid)` → `/forms/{form_id}`; `forms.edit(form_id: uuid)` → `/forms/{form_id}/edit` |
| Workflows | `workflows.directory` → `/workflows`; `workflows.create` → `/workflows/new`; `workflows.assignments` → `/workflows/assignments`; `workflows.detail(workflow_id: uuid)` → `/workflows/{workflow_id}`; `workflows.edit(workflow_id: uuid)` → `/workflows/{workflow_id}/edit` |
| Responses | `responses.directory` → `/responses`; `responses.start` → `/responses/new`; `responses.detail(submission_id: uuid)` → `/responses/{submission_id}`; `responses.edit(submission_id: uuid)` → `/responses/{submission_id}/edit` |
| Datasets | `datasets.directory` → `/datasets`; `datasets.create` → `/datasets/new`; `datasets.detail(dataset_id: uuid)` → `/datasets/{dataset_id}`; `datasets.preview(dataset_id: uuid)` → `/datasets/{dataset_id}/preview`; `datasets.revisions(dataset_id: uuid)` → `/datasets/{dataset_id}/revisions`; `datasets.revision_detail(dataset_id: uuid, revision_id: uuid)` → `/datasets/{dataset_id}/revisions/{revision_id}`; `datasets.revision_edit(dataset_id: uuid, revision_id: uuid)` → `/datasets/{dataset_id}/revisions/{revision_id}/edit`; `datasets.edit(dataset_id: uuid)` → `/datasets/{dataset_id}/edit` |
| Components | `components.directory` → `/components`; `components.create` → `/components/new`; `components.detail(component_ref: string)` → `/components/{component_ref}`; `components.edit(component_ref: string)` → `/components/{component_ref}/edit`; `components.versions(component_ref: string)` → `/components/{component_ref}/versions`; `components.view(component_ref: string)` → `/components/{component_ref}/view` |
| Dashboards | `dashboards.directory` → `/dashboards`; `dashboards.create` → `/dashboards/new`; `dashboards.detail(dashboard_id: uuid)` → `/dashboards/{dashboard_id}`; `dashboards.edit(dashboard_id: uuid)` → `/dashboards/{dashboard_id}/edit`; `dashboards.view(dashboard_id: uuid)` → `/dashboards/{dashboard_id}/view` |
| Migration | No semantic route or path |

Unknown route names, missing/extra/wrongly typed parameters, owner mismatch, or a resolved path outside the same-origin registry returns a structured finding and never falls through to a caller-supplied URL.

## Executable Semantic/Reference Proof Surface

These are the exact Core endpoints exercised by Sprint 6A. All three accept
only strict schema-v1 JSON and authorize before disclosing resource existence.

| Endpoint | Exact responsibility |
| --- | --- |
| `POST /api/platform/destinations/resolve` | Resolve one installation-owned `SemanticDestination` through the registry above; return only a same-origin relative path or a structured rejection finding |
| `POST /api/platform/resource-references` | Construct one installation-bound, `core_installation`-owned transition reference after validating installation, owner, declared resource type, canonical resource ID, and the resource type's current product capability |
| `POST /api/platform/resource-references/resolve` | Return the seven-dimension `ResourceResolutionV1` envelope; restricted known and random identifiers use the same `200` body without an existence lookup disclosure |

The resource registry is exact: `tessara.transition.form`,
`form_version`, `workflow`, `workflow_version`, `response`, `dataset`,
`dataset_revision`, `dataset_major_line`, `component_version`, and `dashboard`.
All IDs are canonical lowercase hyphenated UUIDs except Dataset major lines,
which use canonical `<dataset-uuid>:<non-negative-major-integer>` text. Module
Release/Instance resource types are intentionally absent in Sprint 6A.

Durable fixtures are fixed by purpose rather than by incidental database row
order:

| Proof | Known fixture | Unknown/restricted counterpart |
| --- | --- | --- |
| Strict platform HTTP integration | Form inserted as `name = 'Platform reference fixture'`, `slug = 'platform-reference-fixture'`; route `forms.detail(form_id: uuid)`; resource type `tessara.transition.form` | A newly generated UUID proven through an authorized resolve to be `unknown_resource`; wrong owner/installation/type/parameter/schema and unknown-field variants are asserted separately |
| Response ownership/delegation integration | `seed_demo` respondent submission plus the seeded out-of-scope submission assigned to `delegate@tessara.local`; resource type `tessara.transition.response` | A newly generated Response UUID compared with unrelated, out-of-scope, unauthorized, owned, delegated, and scoped-manager cases |
| Optimized release nondisclosure gate | Explicit `-KnownFormId`, or the first admin-readable Form returned by `GET /api/forms`, preflighted as `authorized` + `resolved` | A generated UUID accepted only after an admin preflight proves `authorized` + `unknown_resource`; the retained artifact binds both exact IDs and tests balanced AB/BA pairs for `unauthorized` and `not_evaluated` actors |

Fixtures and expected envelopes are durable proof. Do not replace an
inconvenient known row, regenerate a checked-in expectation, loosen the exact
wire comparison, or expand timing tolerances without an approved contract
rationale recorded in the Sprint 6A test-change log and equivalent or stronger
positive and negative evidence.

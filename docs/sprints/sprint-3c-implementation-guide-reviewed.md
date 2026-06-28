# Sprint 3C Implementation Guide: Dataset Revision And Compatibility

Status: reviewed and elaborated planning artifact. This is not a kickoff record. Do not treat branch, worktree, validation output, or progress-report setup as complete until the sprint is intentionally started from a clean `main` checkout.

Reviewed: 2026-06-28

Proposed sprint branch: `codex/sprint-3c`

Proposed worktree: `C:\Users\eric-dev\Projects\tessara-sprint-3c`

Proposed committed plan path: `docs/sprints/sprint-3c-plan.md`

## 1. Review Conclusion

Draft 1 is directionally correct: Sprint 3C should make dataset revision behavior explicit through draft revisions, revision publishing, revision history/detail screens, compatibility findings, dependency visibility, and typed compatibility/dependency contracts. That direction matches the roadmap, architecture, and requirements documents.

The plan needs several current-state constraints before it can safely serve as a long-term Codex goal:

1. **Datasets already has an extracted web feature crate.** The current workspace has `tessara-web` as the Leptos application/root route crate, `tessara-web-datasets` as the Datasets feature UI crate, and `tessara-datasets` as a small pure domain crate. Sprint 3C should extend `tessara-web-datasets` for dataset feature UI while keeping route registration, route parameters, shell/auth policy, document integration, hydration entrypoints, CSS, and public assets in root `tessara-web`.
2. **The database already has the core revision table and status enum, but the application does not yet expose the revision lifecycle.** `dataset_revisions` already stores revision status, version number, source/operation/restriction/output JSON snapshots, generated SQL, and materialization metadata. `dataset_revision_status` already has `draft`, `published`, and `superseded` values. The missing work is mostly service/API/UI behavior and typed application contracts, not inventing revisions from scratch.
3. **The current update flow directly publishes.** `PUT /api/admin/datasets/{dataset_id}` currently recompiles the submitted definition, deletes and replaces current `dataset_sources` and `dataset_fields`, inserts a new published revision, supersedes the previous published revision, materializes the new revision, and commits. Sprint 3C must split this into draft save plus explicit publish.
4. **Draft 1 probably underestimates persistence needs for draft metadata and visibility.** Current `dataset_revisions` stores definition snapshots for source, operations, restriction policy, SQL, and output fields, but it does not store draft `name`, `slug`, `grain`, or `visibility_node_ids`. Because the existing editor payload includes all of those fields and Draft 1 says existing edits should save as draft, Sprint 3C should either add a small revision metadata snapshot column or explicitly constrain metadata/visibility changes to publish-only behavior. The recommended plan is to add a small `definition_metadata` JSON snapshot to `dataset_revisions` and backfill existing revisions.
5. **Dependency visibility can be implemented using existing references.** Downstream dependent datasets are visible from current `dataset_sources.dataset_revision_id`; component versions already bind to `component_versions.dataset_revision_id`; dashboards are reachable through `dashboard_components -> component_versions`. The architecture mentions `dataset_revision_dependencies`, but the current baseline migration does not define that table. Sprint 3C should not add a new dependency table unless implementation discovers a narrow blocker.
6. **Carry-forward should be inspect-first in Sprint 3C.** The sprint should not automatically repoint dependent datasets, component versions, or dashboards. It should report typed safe/manual-review/blocked guidance that later Sprint 5C upgrade and stale-dependency flows can consume.
7. **The source-composition checkpoint must be honored.** The progress report records a Sprint 3C checkpoint that unified source operations around `add_source` and removed legacy operation variants. Before Sprint 3C closes, stored revision JSON, seeded data, API fixtures, and Playwright fixtures must be confirmed compatible with that unified contract.

## Pre-Production Implementation Posture

Tessara is currently pre-production and has no systems deployed for external consumption. Sprint 3C implementation may make material changes to existing code, internal APIs, database shape, fixtures, seed data, and tests without preserving backwards compatibility when doing so advances the target architecture.

Default to migrating the application forward rather than carrying compatibility adapters. Temporary bridge code is acceptable only when it materially reduces implementation risk inside the sprint, must be isolated, and must include an explicit removal path before closeout. Existing browser/API callers, test fixtures, and stored demo data should be updated to the accepted Sprint 3C contracts instead of preserving obsolete behavior for its own sake.

## 2. Source Basis Checked

This guide was reviewed against these current repository and documentation areas:

- Uploaded Sprint 3C Draft 1: `sprint-3c-plan.md`.
- Canonical project docs:
  - `docs/README.md`
  - `docs/roadmap.md`
  - `docs/architecture.md`
  - `docs/requirements.md`
  - `docs/sprints/sprint-3c-follow-up-notes.md`
  - `docs/progress-report.md`
- Current workspace and implementation files:
  - `Cargo.toml`
  - `crates/tessara-api/migrations/001_baseline.sql`
  - `crates/tessara-api/src/datasets/mod.rs`
  - `crates/tessara-api/src/datasets/dto.rs`
  - `crates/tessara-api/src/datasets/restriction_tiers.rs`
  - `crates/tessara-datasets/src/lib.rs`
  - `crates/tessara-web-datasets/src/*`
  - `crates/tessara-web/src/*`
  - `end2end/tests/datasets.spec.ts`

## 3. Current-State Findings That Shape Sprint 3C

### 3.1 Roadmap And Requirements Alignment

Sprint 3C is the roadmap’s next Phase 3 slice. Its roadmap outcome is that dataset revision behavior becomes visible and manageable. Its build scope includes revision publishing, revision history, compatibility findings, carry-forward behavior, dependency visibility, normalized typed states, and typed contracts that later component and dashboard work can consume.

The architecture requires the target analytical chain:

```text
Forms/Workflows -> Responses -> Materialized Sources -> DatasetRevision -> ComponentVersion -> Dashboard
```

The same architecture establishes important constraints for this sprint:

- stable dependency edges bind to immutable revisions or versions;
- materialized physical relations may be evicted and rebuilt while semantic revision metadata remains stable;
- compatibility findings classify as `compatible`, `warning`, or `blocking` when a dependent draft is rebound to a newer dependency;
- users may skip some carry-forward work rather than resolving every dependent asset immediately.

The requirements also state that datasets have mutable logical identity with immutable `DatasetRevision`, and that revision history and compatibility behavior must be visible in the application.

### 3.2 Existing Database Baseline

The baseline migration already contains the core tables Sprint 3C should build on:

- `dataset_revision_status` enum: `draft`, `published`, `superseded`.
- `dataset_revisions` with:
  - `id`
  - `dataset_id`
  - `version_number`
  - `version_label`
  - `status`
  - `initial_source`
  - `operations`
  - `restriction_policy`
  - `generated_sql`
  - `output_fields`
  - `materialized_schema`
  - `materialized_table`
  - `materialized_row_count`
  - `materialized_at`
  - `published_at`
  - `created_at`
- a partial unique index enforcing one published revision per dataset.
- `dataset_sources` with optional `dataset_revision_id`, currently usable both as current-published source catalog and as the downstream-dataset dependency edge.
- `dataset_fields`, currently the current-published field catalog.
- `component_versions.dataset_revision_id`, which already references immutable dataset revisions.
- `dashboard_components.component_version_id`, which allows dashboard impact discovery through component versions.

The baseline does **not** include a `dataset_revision_dependencies` table even though the architecture names that family. Use existing dependency edges for Sprint 3C.

### 3.3 Existing Dataset API Baseline

The current dataset API routes are approximately:

- `POST /api/admin/datasets`
- `POST /api/admin/datasets/sql-preview`
- `POST /api/admin/datasets/{dataset_id}/sql-preview`
- `PUT /api/admin/datasets/{dataset_id}`
- `DELETE /api/admin/datasets/{dataset_id}`
- `GET /api/datasets`
- `GET /api/datasets/{dataset_id}`
- `GET /api/datasets/{dataset_id}/table`

Current create behavior is already close to the desired first-revision behavior: it creates the logical dataset, compiles the definition, inserts current `dataset_sources` and `dataset_fields`, inserts a published revision, materializes it, and commits.

Current update behavior is the main mismatch: it directly replaces the dataset metadata/current catalog, inserts a published revision, supersedes the old published revision, materializes immediately, and commits. That behavior must stop being the normal editor save behavior for existing datasets.

Current list/detail behavior is current-published oriented:

- dataset list joins the current published revision by `status = 'published'` and includes minimal revision output-field summaries;
- dataset detail loads current-published revision snapshots but loads sources and fields from current `dataset_sources` and `dataset_fields`;
- dataset table preview reads only the current published materialized revision.

Sprint 3C should preserve those current-published semantics for normal detail and preview routes, while adding explicit revision list/detail/review routes.

### 3.4 Existing DTO Baseline

`crates/tessara-api/src/datasets/dto.rs` currently has request/response types for dataset creation/replacement, source composition, projection, aggregation, calculated fields, row filters, restriction policy, summary, detail, SQL preview, and table rows.

The current DTO layer lacks typed public contracts for:

- dataset revision status;
- revision summary/detail;
- compatibility severity/state;
- dependency kind/state;
- carry-forward state;
- publish eligibility;
- revision dependency summary.

Several existing request fields also still use raw strings for domain values, including operation add type, aggregation functions, calculation functions, row-filter operators, row-picker direction, field types, and restriction tier semantics. Sprint 3C does not need to fully normalize every historical string field, but any new revision, compatibility, dependency, and carry-forward contract must not expand this pattern.

### 3.5 Existing Domain Crate Baseline

`crates/tessara-datasets` exists, but it is currently small. It holds pure dataset validation and `DatasetGrain`. This is a good home for pure typed values and compatibility classification rules that do not require SQL access. The API crate can still own transport DTOs and database orchestration.

Recommended Sprint 3C split:

- Put pure domain enums and compatibility classification helpers in `tessara-datasets` when they do not depend on `sqlx`, `axum`, or auth.
- Keep database queries, authorization, and HTTP response shaping in `tessara-api`.
- Avoid putting new compatibility/dependency logic directly into the already-large `datasets/mod.rs` if a small new module can hold it.

### 3.6 Existing Web Baseline

The current workspace uses the extracted Datasets web feature crate from the completed web refactoring pass. `tessara-web` remains the cargo-leptos application and root route owner. `tessara-web-datasets` owns Datasets feature content, loaders/actions, web DTOs, display helpers, and feature-local support code.

Sprint 3C web work should:

- extend Datasets feature modules inside `tessara-web-datasets`;
- add only thin root route adapters and route parameter wiring in `tessara-web`;
- keep active dataset routes at root-level `/datasets*` URLs;
- avoid extending legacy `/app/datasets` string-template shells;
- avoid HTML-string route shells, `inner_html`, `/bridge/*`, and JavaScript controller ownership for application UI;
- preserve SSR-first behavior and hydration cleanliness.

## 4. Sprint 3C Outcome

At the end of Sprint 3C, a tester should be able to:

1. Open a current published dataset.
2. Edit its definition.
3. Save the edit as an explicit draft revision.
4. Verify that the dataset detail page and dataset table preview still reflect the current published revision before publish.
5. Open revision history.
6. Inspect draft and published revision details.
7. Review compatibility findings produced by comparing the draft revision to the current published revision.
8. Review downstream dependencies that currently point to the published revision.
9. Publish the draft revision.
10. Confirm exactly one revision is current published and the previous revision is superseded.
11. Confirm current dataset source/field catalogs and materialized preview now reflect the newly published revision.
12. Confirm downstream datasets, component versions, and dashboards remain pinned to their existing revision references and show clear impact guidance.

## 5. Non-Goals

Sprint 3C should not include:

- automatic downstream dataset/component/dashboard repointing;
- full stale dependency resolution workflows;
- component authoring beyond using existing `component_versions` dependency rows for visibility;
- dashboard composition changes beyond dependency visibility;
- deletion or garbage collection of superseded materialized dataset tables;
- a broad rewrite of the entire dataset engine;
- moving Datasets feature UI back into root `tessara-web`;
- a new `dataset_revision_dependencies` table unless existing dependency edges prove insufficient for the accepted Sprint 3C scope.

## 6. Core Product Decisions

### 6.1 Draft Revisions

Use explicit draft revisions for edits to existing datasets.

Initial dataset creation may continue to create the first published revision because there is no prior published contract to preserve. Existing dataset edits should no longer replace current-published catalog rows or materialized output until publish.

### 6.2 One Open Draft Per Dataset

Use one open draft revision per dataset for Sprint 3C.

Rationale:

- The current status enum has only `draft`, `published`, and `superseded`; it has no `abandoned`, `archived`, or `cancelled` draft state.
- A single open draft keeps UI, tests, publish semantics, and permissions understandable.
- More advanced multi-draft collaboration can be added later if the product needs it.

Implementation expectation:

- Draft save should create a draft if none exists.
- Draft save should update the existing draft if one exists.
- If a unique partial index for one draft is added, enforce it in the database.
- If no index is added, enforce one-draft behavior inside a transaction using row locks and an explicit check.

### 6.3 Published Catalog Tables

Treat these as current-published catalog tables only:

- `dataset_sources`
- `dataset_fields`
- `dataset_scope_nodes` for current-published visibility metadata unless revision metadata snapshots are being reviewed.

Draft saves must not mutate those tables. Publish mutates them atomically.

### 6.4 Revision Snapshots

Treat `dataset_revisions` as the authoritative revision record.

A revision detail must be renderable from `dataset_revisions` snapshots without reading `dataset_sources` or `dataset_fields` for draft content.

Existing snapshot fields:

- `initial_source`
- `operations`
- `restriction_policy`
- `generated_sql`
- `output_fields`

Recommended new snapshot field:

- `definition_metadata`, a JSON object holding logical dataset metadata needed to review and publish a draft:

```json
{
  "name": "Participant Outcomes",
  "slug": "participant-outcomes",
  "grain": "submission",
  "visibility_node_ids": ["..."]
}
```

If this migration is rejected, Sprint 3C must explicitly narrow draft-save behavior so name, slug, grain, and visibility are not part of draft revision editing. That narrower behavior is less consistent with the existing editor payload and is not recommended.

### 6.5 Publish Does Not Rebind Dependents

Publishing a new dataset revision should not mutate downstream references.

Existing downstream references should remain pinned to their existing dataset revision IDs. Sprint 3C reports whether a future carry-forward looks safe, blocked, or manual-review, but it does not perform the carry-forward.

### 6.6 Breaking Findings Block Carry-Forward, Not Necessarily Dataset Publish

Because downstream assets remain pinned to their existing revision IDs, a breaking dataset revision does not immediately break active dependents. Sprint 3C should block publish only for conditions that make the new revision invalid or unsafe to become the current dataset revision, such as:

- invalid payload;
- unauthorized scope or mutation;
- duplicate slug;
- stale or missing source references;
- SQL compilation failure;
- materialization failure;
- invalid revision status transition.

Breaking compatibility findings should produce a `blocked` carry-forward state for affected dependencies. They should not automatically prevent dataset publish unless product review explicitly adds a publish-acknowledgement guard.

If an acknowledgement guard is desired, implement it narrowly as a publish request field such as `acknowledge_breaking_impact: true`, required only when blocking findings and downstream dependencies both exist. Do not silently block publish without giving the user a review path.

## 7. Recommended Persistence Changes

### 7.1 Minimal Migration

Add revision metadata snapshots and optional draft uniqueness. Example migration shape:

```sql
ALTER TABLE dataset_revisions
ADD COLUMN definition_metadata jsonb;

CREATE UNIQUE INDEX dataset_revisions_one_draft_idx
    ON dataset_revisions (dataset_id)
    WHERE status = 'draft';
```

Backfill `definition_metadata` for existing revisions from current dataset metadata and visibility nodes. Exact SQL can be adjusted for the migration style, but the data should resemble:

```sql
UPDATE dataset_revisions AS revision
SET definition_metadata = jsonb_build_object(
    'name', datasets.name,
    'slug', datasets.slug,
    'grain', datasets.grain,
    'visibility_node_ids', COALESCE(visibility.visibility_node_ids, '[]'::jsonb)
)
FROM datasets
LEFT JOIN LATERAL (
    SELECT jsonb_agg(dataset_scope_nodes.node_id ORDER BY dataset_scope_nodes.node_id) AS visibility_node_ids
    FROM dataset_scope_nodes
    WHERE dataset_scope_nodes.dataset_id = datasets.id
) AS visibility ON true
WHERE revision.dataset_id = datasets.id
  AND revision.definition_metadata IS NULL;
```

If the project currently keeps a single squashed baseline migration rather than incremental migrations, follow the active migration convention for the branch. Do not leave production-like state unable to migrate.

### 7.2 Optional Helper Indexes

The existing dependency queries should work without new indexes for local data volumes. If query plans become problematic, consider small helper indexes:

```sql
CREATE INDEX dataset_sources_dataset_revision_idx
    ON dataset_sources (dataset_revision_id)
    WHERE dataset_revision_id IS NOT NULL;

CREATE INDEX component_versions_dataset_revision_idx
    ON component_versions (dataset_revision_id);

CREATE INDEX dashboard_components_component_version_idx
    ON dashboard_components (component_version_id);
```

Do not add indexes speculatively if the current schema and tests do not need them.

### 7.3 Materialized Table Retention

Publishing a new revision should not drop the old revision’s materialized table.

Reason: stable dependency edges bind to immutable revisions. Future component execution may need to read a superseded revision’s materialized table. The current materialization helper names tables by revision ID, so new publishes should naturally create distinct tables.

Do not build a materialized table garbage collector in Sprint 3C.

## 8. Typed Domain And DTO Contracts

### 8.1 Serialization Rule

Every new public revision, compatibility, dependency, and carry-forward state must serialize as stable `snake_case` strings. Avoid ad hoc raw-string comparisons in API and web code.

Recommended Rust pattern:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetRevisionStatus {
    Draft,
    Published,
    Superseded,
}
```

Prefer typed conversion functions at database boundaries rather than comparing strings throughout services and UI code.

### 8.2 Core Enums

Add typed values for at least:

```rust
pub enum DatasetRevisionStatus {
    Draft,
    Published,
    Superseded,
}

pub enum DatasetCompatibilitySeverity {
    Info,
    Warning,
    Blocking,
}

pub enum DatasetCompatibilityState {
    Compatible,
    ReviewRequired,
    Blocked,
}

pub enum DatasetCompatibilityFindingKind {
    OutputFieldAdded,
    OutputFieldRemoved,
    OutputFieldTypeChanged,
    OutputFieldLabelChanged,
    OutputFieldSourceChanged,
    OutputFieldOrderChanged,
    RestrictionPolicyChanged,
    SourceCompositionChanged,
    OperationPipelineChanged,
    GeneratedSqlChanged,
}

pub enum DatasetDependencyKind {
    DependentDataset,
    ComponentVersion,
    Dashboard,
}

pub enum DatasetDependencyState {
    Safe,
    ManualReview,
    Blocked,
}

pub enum DatasetCarryForwardState {
    NotApplicable,
    Safe,
    ManualReview,
    Blocked,
}

pub enum DatasetPublishEligibilityState {
    Publishable,
    RequiresAcknowledgement,
    NotPublishable,
}
```

The exact names can change, but the contract must remain typed and stable.

### 8.3 DTO Shapes

Add or equivalent DTOs in `crates/tessara-api/src/datasets/dto.rs`.

#### `DatasetRevisionMetadataDto`

```json
{
  "name": "Participant Outcomes",
  "slug": "participant-outcomes",
  "grain": "submission",
  "visibility_node_ids": ["..."]
}
```

#### `DatasetRevisionSummary`

```json
{
  "id": "...",
  "dataset_id": "...",
  "version_number": 3,
  "version_label": "Draft revision 3",
  "status": "draft",
  "is_current_published": false,
  "created_at": "2026-06-28T12:00:00Z",
  "published_at": null,
  "materialized_at": null,
  "materialized_row_count": null,
  "output_field_count": 12,
  "dependency_summary": {
    "dependent_dataset_count": 1,
    "component_version_count": 2,
    "dashboard_count": 1,
    "safe_count": 2,
    "manual_review_count": 1,
    "blocked_count": 1
  },
  "compatibility_summary": {
    "state": "review_required",
    "info_count": 2,
    "warning_count": 1,
    "blocking_count": 0
  }
}
```

#### `DatasetRevisionDetail`

```json
{
  "summary": { "...": "..." },
  "metadata": { "...": "..." },
  "visibility_nodes": [],
  "initial_source": { "kind": "form" },
  "operations": [],
  "restriction_policy": null,
  "generated_sql": "SELECT ...",
  "materialization": {
    "schema": "dataset_materialized",
    "table": "dataset_...",
    "row_count": 200,
    "materialized_at": "2026-06-28T12:00:00Z"
  },
  "output_fields": [],
  "compatibility": {
    "state": "review_required",
    "findings": []
  },
  "dependencies": {
    "items": [],
    "summary": {}
  },
  "publish_eligibility": {
    "state": "publishable",
    "reasons": []
  }
}
```

#### `DatasetCompatibilityFinding`

```json
{
  "kind": "output_field_removed",
  "severity": "blocking",
  "state": "blocked",
  "field_key": "participant_age",
  "field_label": "Participant Age",
  "previous_value": "number",
  "candidate_value": null,
  "message": "Output field participant_age was removed.",
  "dependency_notes": ["Dependent table components using this field cannot be carried forward automatically."]
}
```

#### `DatasetDependencyImpact`

```json
{
  "kind": "component_version",
  "state": "manual_review",
  "carry_forward_state": "manual_review",
  "id": "...",
  "name": "Outcomes Table v2",
  "status": "published",
  "pinned_revision_id": "...",
  "referenced_field_keys": ["participant_age"],
  "affected_field_keys": ["participant_age"],
  "message": "Component version remains pinned to revision 2; carry-forward needs review because participant_age changed type."
}
```

#### `SaveDatasetDraftResponse`

```json
{
  "dataset_id": "...",
  "draft_revision_id": "...",
  "current_revision_id": "...",
  "revision": { "...": "..." },
  "compatibility_summary": { "...": "..." },
  "dependency_summary": { "...": "..." }
}
```

#### `PublishDatasetRevisionRequest`

```json
{
  "acknowledge_breaking_impact": false
}
```

This request field is optional unless a publish acknowledgement guard is accepted. If no acknowledgement guard is implemented, use an empty request body or no body.

#### `PublishDatasetRevisionResponse`

```json
{
  "dataset_id": "...",
  "published_revision_id": "...",
  "previous_revision_id": "...",
  "revision": { "...": "..." },
  "compatibility_summary": { "...": "..." },
  "dependency_summary": { "...": "..." }
}
```

## 9. API Surface

### 9.1 Preserve Existing Public Current-Published Routes

Keep these routes current-published oriented:

| Method | Route | Purpose |
| --- | --- | --- |
| `GET` | `/api/datasets` | List datasets visible to the caller, with current published revision metadata. |
| `GET` | `/api/datasets/{dataset_id}` | Load current published dataset definition. |
| `GET` | `/api/datasets/{dataset_id}/table` | Preview current published materialized output. |

These routes should not return draft definitions by default. A dataset detail page may show that a draft exists, but its normal definition and table should remain current published.

### 9.2 Preserve Initial Create Route

| Method | Route | Purpose |
| --- | --- | --- |
| `POST` | `/api/admin/datasets` | Create dataset and first published revision. |

Initial create can continue to create and materialize the first published revision. It should populate `definition_metadata` if that column is added.

### 9.3 Keep SQL Preview Routes

| Method | Route | Purpose |
| --- | --- | --- |
| `POST` | `/api/admin/datasets/sql-preview` | Preview SQL for a new dataset draft payload. |
| `POST` | `/api/admin/datasets/{dataset_id}/sql-preview` | Preview SQL for an existing dataset draft payload. |

SQL preview remains an unsaved compile path. It must not create a revision, mutate current catalog rows, or materialize output.

### 9.4 Add Revision Read Routes

| Method | Route | Capability | Purpose |
| --- | --- | --- | --- |
| `GET` | `/api/datasets/{dataset_id}/revisions` | `datasets:read` plus dataset visibility | List revisions for one dataset. |
| `GET` | `/api/datasets/{dataset_id}/revisions/{revision_id}` | `datasets:read` plus dataset visibility | Load one revision detail with snapshots, compatibility, dependencies, and publish eligibility. |

Revision read routes must respect dataset visibility. Draft revision detail should not leak to users who can read the published dataset but do not have permission to manage drafts if product review decides drafts are manager-only. Recommended Sprint 3C rule:

- published/superseded revision detail: `datasets:read` and dataset visibility;
- draft revision detail: `datasets:manage` and full dataset manage scope, unless product explicitly wants read-only draft review for scoped readers.

This avoids exposing unreviewed draft metadata to ordinary readers.

### 9.5 Add Draft Save Route

| Method | Route | Capability | Purpose |
| --- | --- | --- | --- |
| `POST` | `/api/admin/datasets/{dataset_id}/revisions/draft` | `datasets:manage` plus full dataset scope | Create or update the dataset’s open draft revision. |

Request body can reuse the existing `CreateDatasetRequest` shape initially, but the transport name should become less misleading when possible. Suggested alias:

- existing: `CreateDatasetRequest`
- new semantic alias: `DatasetDefinitionDraftRequest`

The struct can be reused mechanically while naming is improved.

### 9.6 Add Publish Route

| Method | Route | Capability | Purpose |
| --- | --- | --- | --- |
| `POST` | `/api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish` | `datasets:manage` plus full dataset scope | Publish one draft revision atomically. |

Only draft revisions can be published. Published or superseded revisions must return a typed conflict/bad-request error.

### 9.7 Adapt Or Deprecate Existing Update Route

`PUT /api/admin/datasets/{dataset_id}` currently performs direct publish. Sprint 3C should stop relying on this route from the web editor.

Recommended behavior:

- Prefer changing this route to the new draft-save behavior or removing normal application reliance on it entirely.
- Do not preserve direct-publish semantics for backwards compatibility. Tessara is pre-production, so browser code, tests, scripts, and seed paths should migrate to the accepted revision lifecycle.
- Keep a temporary adapter only if it materially reduces sprint implementation risk; if kept, isolate it, document the removal path, and prove normal browser edit save does not direct-publish through it.
- Prefer the new draft route from all browser editor code.
- Add a regression proving browser edit save does not direct-publish through the old path.

Do not leave the old direct-publish path reachable from normal application UI after Sprint 3C.

## 10. Authorization Rules

### 10.1 Revision Listing And Detail

For revision list/detail of published and superseded revisions:

- require `datasets:read`;
- require dataset visible under the caller’s effective dataset read boundary;
- for scoped users, visibility-node summaries should be filtered the same way current dataset list/detail filtering works.

For draft revisions:

- recommended: require `datasets:manage` and full manage scope for the dataset;
- if product review wants draft visibility for readers, split response data so draft internals are not leaked beyond intended roles.

### 10.2 Draft Save

Draft save must:

- require `datasets:manage`;
- require the dataset to be fully inside the caller’s manage boundary;
- require every requested `visibility_node_id` to be inside the caller’s manage scope;
- validate slug uniqueness against all other datasets;
- compile under the caller’s permissions, so source references cannot bypass scope.

### 10.3 Publish

Publish must:

- require `datasets:manage`;
- require the dataset to be fully inside the caller’s manage boundary;
- require the draft metadata visibility nodes to remain inside the caller’s manage scope;
- revalidate slug uniqueness before commit;
- recompile before materialization to catch stale references;
- reject invalid revision status transitions.

### 10.4 Downstream Dependency Visibility

Dependency visibility should be conservative:

- Admin/global managers may see all dependency records.
- Scoped managers/readers should only see dependent assets they can otherwise see under relevant dataset/component/dashboard visibility rules.
- If component or dashboard scoped visibility is not yet fully implemented, do not leak extra metadata. Show a count or generic “additional hidden dependencies” message only if needed and safe.

## 11. Draft Save Service Algorithm

Use a service function roughly shaped like:

```rust
async fn save_dataset_draft(
    pool: &PgPool,
    account: &AccountContext,
    dataset_id: Uuid,
    payload: DatasetDefinitionDraftRequest,
) -> ApiResult<SaveDatasetDraftResponse>
```

Algorithm:

1. Authenticate and authorize `datasets:manage`.
2. Start a transaction.
3. Lock the logical dataset row with `SELECT ... FOR UPDATE`.
4. Confirm the dataset exists.
5. Confirm the dataset is fully inside the caller’s manage scope.
6. Validate `name`, `slug`, `grain`, and `visibility_node_ids`.
7. Confirm requested visibility nodes exist and are inside caller manage scope.
8. Confirm slug uniqueness excluding the current dataset.
9. Compile the draft definition using the existing compile path with `dataset_id = Some(dataset_id)`.
10. Serialize metadata snapshot:
    - `name`
    - `slug`
    - `grain`
    - `visibility_node_ids`
11. Load current published revision, if any.
12. Load existing open draft revision, if any.
13. If a draft exists:
    - update the same row;
    - keep its `version_number`;
    - keep `status = draft`;
    - clear materialization metadata unless drafts are intentionally materialized later;
    - update `version_label` if needed.
14. If no draft exists:
    - compute `version_number = max(version_number) + 1` for this dataset;
    - insert a new `dataset_revisions` row with `status = draft`, no `published_at`, no materialized table, and full snapshots.
15. Do **not** update `datasets`.
16. Do **not** update `dataset_sources`.
17. Do **not** update `dataset_fields`.
18. Do **not** update `dataset_scope_nodes`.
19. Commit.
20. Compute compatibility findings and dependency impacts for the response.

Compatibility and dependency computation can happen inside or after the transaction. If computed after commit, reload the draft by ID so the response reflects stored state.

## 12. Publish Service Algorithm

Use a service function roughly shaped like:

```rust
async fn publish_dataset_revision(
    pool: &PgPool,
    account: &AccountContext,
    dataset_id: Uuid,
    revision_id: Uuid,
    request: PublishDatasetRevisionRequest,
) -> ApiResult<PublishDatasetRevisionResponse>
```

Algorithm:

1. Authenticate and authorize `datasets:manage`.
2. Start a transaction.
3. Lock the logical dataset row with `SELECT ... FOR UPDATE`.
4. Load the target revision with `FOR UPDATE`.
5. Confirm the revision belongs to the dataset.
6. Confirm revision status is `draft`.
7. Confirm the dataset is fully inside caller manage scope.
8. Deserialize metadata, initial source, operations, restriction policy, and output fields from revision snapshots.
9. Revalidate metadata:
   - non-empty name;
   - valid slug;
   - supported grain;
   - slug unique excluding this dataset;
   - visibility nodes exist;
   - visibility nodes are inside caller manage scope.
10. Recompile the stored draft definition rather than trusting stale stored SQL.
11. Compare recompiled SQL/output snapshots to stored snapshots.
    - If different only because of deterministic formatting changes, update stored snapshots.
    - If source references are missing or invalid, return not-publishable with a typed error.
12. Load current published revision, if any.
13. Compute compatibility findings against the current published revision.
14. Compute downstream dependency impacts for the current published revision.
15. If an acknowledgement guard is implemented and blocking findings plus downstream dependencies exist, require `acknowledge_breaking_impact = true`.
16. Materialize the candidate revision into `dataset_materialized.dataset_{revision_id}` inside the transaction.
17. Update the logical dataset row from revision metadata:
    - `name`
    - `slug`
    - `grain`
18. Replace current-published catalog rows:
    - delete `dataset_sources` for the dataset;
    - insert compiled sources;
    - delete `dataset_fields` for the dataset;
    - insert compiled fields;
    - replace `dataset_scope_nodes` with metadata visibility nodes.
19. Supersede the previous published revision:
    - `UPDATE dataset_revisions SET status = 'superseded' WHERE dataset_id = $1 AND status = 'published'`.
20. Publish the draft revision:
    - set `status = 'published'`;
    - set `published_at = now()`;
    - ensure materialization metadata is set;
    - keep `version_number` stable.
21. Commit.
22. Return the published revision detail, previous revision ID, compatibility summary, and dependency summary.

Postconditions:

- exactly one `dataset_revisions` row has `status = published` for the dataset;
- no row has `status = draft` for the dataset after successful publish if using one-open-draft semantics;
- current dataset detail reflects the newly published revision;
- current dataset table preview reads the newly materialized table;
- downstream references remain unchanged.

## 13. Compatibility Classification

### 13.1 Baseline

Compare a candidate draft revision to the current published revision for the same dataset.

If there is no current published revision, return:

- `state = compatible`
- no blocking findings
- optional informational finding: `no_published_baseline`

### 13.2 Output Field Contract Rules

Compare output fields by stable `key`.

| Change | Severity | Compatibility State | Carry-Forward State | Notes |
| --- | --- | --- | --- | --- |
| Output field added | `info` | `compatible` | `safe` | Existing dependents should not break. |
| Output field removed | `blocking` | `blocked` | `blocked` | Downstream consumers may reference the field. |
| Output field type changed | `blocking` | `blocked` | `blocked` | Treat as breaking even if name/key stays stable. |
| Output field label changed | `info` | `compatible` or `review_required` | `safe` or `manual_review` | Usually display-only; warn only if known dependent config stores labels. |
| Output field position changed | `info` | `compatible` | `safe` | Field order is not a semantic dependency unless a consumer is known to rely on it. |
| Output field source alias changed with same key/type | `warning` | `review_required` | `manual_review` | Semantics may have changed even if contract shape did not. |
| Output field source field key changed with same key/type | `warning` | `review_required` | `manual_review` | Semantics may have changed. |

### 13.3 Restriction Policy Rules

| Change | Severity | Compatibility State | Carry-Forward State | Notes |
| --- | --- | --- | --- | --- |
| Restriction policy added | `warning` | `review_required` | `manual_review` | Rows visible to some readers may shrink. |
| Restriction policy removed | `warning` | `review_required` | `manual_review` | Rows visible to some readers may expand. |
| Restriction tier field changed | `warning` | `review_required` | `manual_review` | Needs human review. |
| No restriction policy change | none | unchanged | unchanged | No finding needed. |

Restriction changes are not simply display changes. They affect row access. They should always appear in the review screen.

### 13.4 Source And Operation Rules

| Change | Severity | Compatibility State | Carry-Forward State | Notes |
| --- | --- | --- | --- | --- |
| Source composition changed but output fields unchanged | `warning` | `review_required` | `manual_review` | Semantics or row counts may change. |
| Filter operation changed but output fields unchanged | `warning` | `review_required` | `manual_review` | Row population may change. |
| Calculation pipeline changed for an output field with same type | `warning` | `review_required` | `manual_review` | Field meaning may change. |
| Aggregation changed with output fields unchanged | `warning` | `review_required` | `manual_review` | Row grain or values may change. |
| Generated SQL changed but no structural change detected | `info` | `compatible` or `review_required` | `safe` or `manual_review` | Prefer more specific operation findings when possible. |

Do not flood users with duplicate low-value findings. Group operation-level changes where detailed diffing is not yet reliable.

### 13.5 Summary State Derivation

Derive the summary from findings:

- any blocking finding -> `blocked`;
- else any warning finding -> `review_required`;
- else -> `compatible`.

For dependency carry-forward:

- direct affected blocking field -> `blocked`;
- warning-only or unknown consumer field usage -> `manual_review`;
- info-only changes or additions -> `safe`.

## 14. Dependency Discovery And Impact Rules

### 14.1 Dependency Sources

For a published baseline revision, discover downstream dependencies from existing tables.

#### Dependent Datasets

Query current dataset source catalog rows:

```sql
SELECT
    dependent_dataset.id AS dataset_id,
    dependent_dataset.name AS dataset_name,
    dependent_dataset.slug AS dataset_slug,
    dataset_sources.source_alias,
    dataset_sources.position
FROM dataset_sources
JOIN datasets AS dependent_dataset
    ON dependent_dataset.id = dataset_sources.dataset_id
WHERE dataset_sources.dataset_revision_id = $1
ORDER BY dependent_dataset.name, dataset_sources.position;
```

These rows represent datasets whose current published definitions reference the baseline revision.

#### Component Versions

Query component versions:

```sql
SELECT
    components.id AS component_id,
    components.name AS component_name,
    component_versions.id AS component_version_id,
    component_versions.version_number,
    component_versions.version_label,
    component_versions.status,
    component_versions.config
FROM component_versions
JOIN components ON components.id = component_versions.component_id
WHERE component_versions.dataset_revision_id = $1
ORDER BY components.name, component_versions.version_number;
```

Include draft, published, and superseded component versions unless visibility rules require filtering. Published versions should be visually emphasized because they are likely active dependencies.

#### Dashboards

Query dashboards through component versions:

```sql
SELECT
    dashboards.id AS dashboard_id,
    dashboards.name AS dashboard_name,
    dashboard_components.id AS dashboard_component_id,
    component_versions.id AS component_version_id,
    components.id AS component_id,
    components.name AS component_name
FROM dashboard_components
JOIN component_versions
    ON component_versions.id = dashboard_components.component_version_id
JOIN components
    ON components.id = component_versions.component_id
JOIN dashboards
    ON dashboards.id = dashboard_components.dashboard_id
WHERE component_versions.dataset_revision_id = $1
ORDER BY dashboards.name, components.name;
```

Dashboard impact inherits the state of its referenced component version unless the dashboard has additional known field references later.

### 14.2 Impact State For Dependent Datasets

For dependent datasets:

1. Identify which `dataset_sources` row uses the baseline revision.
2. Inspect the dependent dataset’s current field catalog and/or revision JSON snapshot.
3. Find fields whose `source_alias` matches the dependency source alias and whose `source_field_key` appears in affected compatibility findings.
4. Classify:
   - `blocked` if a removed or type-changed field is used;
   - `manual_review` if source/operation/restriction warnings affect the dependency and exact field use cannot prove safety;
   - `safe` if only additions or info-only changes exist.

### 14.3 Impact State For Component Versions

For component versions:

1. Inspect known config JSON paths for referenced dataset field keys if such paths exist.
2. If a removed/type-changed field is referenced, classify `blocked`.
3. If field usage cannot be confidently extracted and blocking field changes exist, classify `manual_review` rather than hiding uncertainty.
4. If only additions exist, classify `safe`.
5. If source/restriction/operation warnings exist, classify `manual_review`.

### 14.4 Impact State For Dashboards

For dashboards:

1. Group dashboard components by dashboard.
2. Derive dashboard state as the maximum severity of included component-version states:
   - any `blocked` component -> dashboard `blocked`;
   - else any `manual_review` component -> dashboard `manual_review`;
   - else `safe`.
3. Explain that dashboards remain pinned through their component versions.

### 14.5 Hidden Dependencies

If scoped visibility prevents listing a dependency, return a safe aggregate rather than leaking names:

```json
{
  "hidden_dependency_count": 2,
  "message": "Additional dependencies exist outside your visible scope."
}
```

Only include this if the caller is allowed to know that hidden dependencies exist. Otherwise omit it.

## 15. API Error And Conflict Behavior

Use stable application errors and avoid raw database strings.

Recommended cases:

| Situation | Suggested Status | Suggested Code |
| --- | --- | --- |
| Dataset not found | `404` | `dataset_not_found` |
| Revision not found | `404` | `dataset_revision_not_found` |
| Draft save without manage capability | `403` | `dataset_manage_required` |
| Revision read without visibility | `403` | `dataset_read_required` |
| Publish non-draft revision | `409` or `400` | `dataset_revision_not_draft` |
| Another published revision conflict | `409` | `dataset_revision_publish_conflict` |
| Duplicate slug | `400` | `dataset_slug_unavailable` |
| Invalid stored draft snapshot | `500` plus server log | `dataset_revision_snapshot_invalid` |
| Candidate cannot compile | `400` | `dataset_revision_compile_failed` |
| Candidate cannot materialize | `500` or `400` depending cause | `dataset_revision_materialization_failed` |
| Publish acknowledgement required | `409` | `dataset_revision_acknowledgement_required` |

Follow the project’s existing error envelope style when implementing actual responses.

## 16. Backend Module Plan

Current `crates/tessara-api/src/datasets/mod.rs` is already very large. Sprint 3C should add clear module boundaries for new behavior without forcing a risky full rewrite.

Recommended module direction:

```text
crates/tessara-api/src/datasets/
  mod.rs                  # route registration and public re-exports; keep existing code stable where possible
  dto.rs                  # existing DTOs plus revision/compat/dependency DTOs
  restriction_tiers.rs    # existing restriction helpers
  revisions.rs            # new revision handlers and service entrypoints
  revision_repo.rs        # new SQL helpers for revision list/detail/save/publish/dependency discovery
  compatibility.rs        # compatibility comparison logic if not moved to tessara-datasets
  dependencies.rs         # dependency impact discovery/classification
```

Alternative if the team prefers fewer files:

- `revisions.rs` for handlers/service/repo combined;
- `compatibility.rs` for pure comparison;
- `dependencies.rs` for dependency discovery.

Do not grow `datasets/mod.rs` by another large block of unrelated handler, SQL, and compatibility logic.

### 16.1 `tessara-datasets` Domain Additions

Add pure types and comparison helpers where practical:

```text
crates/tessara-datasets/src/
  lib.rs
  revisions.rs
  compatibility.rs
```

Good candidates for this crate:

- revision status enum;
- compatibility severity/state enums;
- dependency and carry-forward state enums;
- pure output-field comparison based on small structs;
- summary-state derivation.

Keep these out of the domain crate:

- SQL queries;
- authorization;
- materialization;
- Axum DTO response shaping;
- database transaction orchestration.

## 17. Web Implementation Plan

### 17.1 Route Targets

Add application routes under the existing Datasets area:

```text
/datasets/{dataset_id}/revisions
/datasets/{dataset_id}/revisions/{revision_id}
```

Use the route parameter style already used by the active Leptos router. The route text above is the product URL contract, not a requirement to use a specific router macro syntax.

### 17.2 Feature Location

Implement Datasets feature UI in the existing `tessara-web-datasets` crate. Keep root URL registration, route parameter parsing, shell/auth policy, document integration, hydration entrypoint, CSS, and public assets in `tessara-web`.

Recommended structure, adjusted to match the actual route/module organization when coding:

```text
crates/tessara-web-datasets/src/
  mod.rs
  api.rs
  types.rs
  pages/
  editor.rs
  revisions.rs
  revision_detail.rs
  compatibility.rs
  dependencies.rs
```

Root route additions should stay in `crates/tessara-web/src/routes/datasets.rs` as thin adapters that render `tessara-web-datasets` content components.

### 17.3 Dataset Detail Updates

Dataset detail should show:

- current published revision number/label/status;
- current revision publish date;
- materialized row count and materialized date;
- output field count;
- link to revision history;
- draft banner if an open draft exists and caller has manage permission;
- clear message that preview/table data is from the current published revision.

### 17.4 Editor Flow Updates

For new datasets:

- keep the current first-create path;
- create the dataset and first published revision;
- redirect to dataset detail or revision detail according to existing UX.

For existing datasets:

- editor loads current published definition by default;
- save button text should communicate draft behavior, such as `Save Draft Revision`;
- save calls `/api/admin/datasets/{dataset_id}/revisions/draft`;
- after save, navigate to the draft revision detail/review screen;
- do not refresh the current dataset detail or preview as if published changes are live.

If an open draft already exists:

- dataset detail should show `Continue draft review`;
- editor should either load the draft for continued editing or explicitly ask the user to choose current-published editing vs draft continuation;
- for Sprint 3C, prefer loading the open draft for managers to avoid accidental overwrites.

### 17.5 Revision History Screen

Route: `/datasets/{dataset_id}/revisions`

Required content:

- dataset name and breadcrumb back to dataset detail;
- revision table/card list with:
  - version number;
  - version label;
  - status badge;
  - current published marker;
  - created date;
  - published date;
  - materialized date;
  - materialized row count;
  - output field count;
  - compatibility summary;
  - dependency summary;
  - detail link;
- empty state if no revisions exist, though normal datasets should have at least one;
- draft row visually distinct from published/superseded rows.

Use shared table/list primitives where available. Provide mobile cards consistent with Sprint 3A/3B dataset table work.

### 17.6 Revision Detail / Draft Review Screen

Route: `/datasets/{dataset_id}/revisions/{revision_id}`

Required sections:

1. **Header**
   - dataset name;
   - version number/label;
   - status badge;
   - current published marker;
   - actions.
2. **Publish Review Banner** for draft revisions.
   - summarize compatibility state;
   - summarize dependency impact;
   - show publish button when caller can publish;
   - show disabled/not-publishable reasons if not.
3. **Metadata Snapshot**
   - name;
   - slug;
   - grain;
   - visibility nodes.
4. **Source Snapshot**
   - initial source;
   - added sources;
   - joins/unions/add source types;
   - referenced form versions or dataset revisions.
5. **Operation Pipeline**
   - projection;
   - aggregation;
   - calculated fields;
   - filters;
   - restriction policy.
6. **Output Field Contract**
   - key;
   - label;
   - type;
   - source alias;
   - source field;
   - position.
7. **Compatibility Findings**
   - grouped by blocking, warning, info;
   - clear human-readable explanation;
   - affected fields.
8. **Downstream Dependencies**
   - dependent datasets;
   - component versions;
   - dashboards;
   - state and carry-forward guidance;
   - pinned revision explanation.
9. **Generated SQL**
   - collapsible or otherwise manageable;
   - copy-friendly if current UI supports that pattern.
10. **Materialization**
    - schema/table for published revisions;
    - row count;
    - materialized date;
    - draft revisions should show `Not materialized until publish` unless draft materialization is later added.

### 17.7 Publish Action UX

On draft detail:

- primary action: `Publish revision`;
- optional acknowledgement checkbox if the API requires acknowledgement for breaking impact;
- success state redirects to the published revision detail or dataset detail;
- error state shows typed, user-safe error message;
- after publish, status badges should show:
  - new revision: `Published` and `Current`;
  - prior revision: `Superseded`.

### 17.8 SSR And Hydration

The revision history and detail pages are read-heavy. They should render meaningful SSR HTML before hydration.

Hydration should be used for:

- publish action;
- draft save form behavior;
- collapsible panels if implemented;
- client-side table filtering if existing shared components rely on it.

Do not make revision history depend on a client-only fetch if server-side route data patterns already exist.

## 18. Ordered Implementation Plan For Codex

### Phase 0: Sprint Reconciliation And Safety

1. Start from clean `main`.
2. Create branch `codex/sprint-3c` only when the sprint is formally started.
3. Confirm worktree path if using the proposed worktree.
4. Read these files before coding:
   - `docs/roadmap.md`
   - `docs/architecture.md`
   - `docs/requirements.md`
   - `docs/sprints/sprint-3c-follow-up-notes.md`
   - `docs/progress-report.md`
   - current dataset API/web code.
5. Confirm whether the repository is using incremental migrations or a squashed baseline for this branch.
6. Confirm source-operation stored JSON is already unified around `add_source`, or migrate stored demo data, fixtures, tests, and revision JSON forward before closing. Use temporary compatibility parsing only as an isolated bridge if direct migration is not feasible inside the sprint.

### Phase 1: Contracts And Domain Types

1. Add typed revision, compatibility, dependency, and carry-forward enums.
2. Ensure stable `snake_case` serialization.
3. Add DTOs for revision summary/detail, compatibility findings, dependency impacts, dependency summary, draft save response, and publish response.
4. Add conversion helpers between DB status strings and typed status values.
5. Add unit tests for enum serialization/deserialization.

### Phase 2: Persistence And Repository Helpers

1. Add `definition_metadata` snapshot persistence if accepted.
2. Backfill existing revisions.
3. Add one-open-draft enforcement, preferably with a partial unique index if migration convention allows.
4. Add repository helpers for:
   - loading current published revision;
   - loading open draft revision;
   - listing revisions;
   - loading revision detail snapshots;
   - inserting/updating draft revision;
   - publishing status transition;
   - replacing current catalog tables;
   - dependency discovery.
5. Keep helpers transaction-friendly.

### Phase 3: Draft Save Service

1. Extract or wrap existing compile behavior so draft save can compile without publishing.
2. Implement draft upsert behavior.
3. Ensure draft save does not mutate:
   - `datasets`;
   - `dataset_sources`;
   - `dataset_fields`;
   - `dataset_scope_nodes`;
   - published revision statuses;
   - materialized output.
4. Return compatibility and dependency summaries.
5. Add API tests proving current published detail/table remain unchanged after draft save.

### Phase 4: Compatibility And Dependency Services

1. Implement output-field diffing by `key`.
2. Implement restriction-policy diffing.
3. Implement source/operation coarse diffing.
4. Implement summary-state derivation.
5. Implement dependent dataset discovery.
6. Implement component version discovery.
7. Implement dashboard discovery through component versions.
8. Implement dependency impact state derivation.
9. Add pure unit tests for compatibility rules.
10. Add API/integration tests with seeded dependency rows.

### Phase 5: Publish Service

1. Implement publish transaction with dataset row lock.
2. Revalidate and recompile stored draft snapshots.
3. Materialize the candidate revision inside the transaction.
4. Replace current-published catalog rows and visibility rows.
5. Supersede exactly the previous published revision.
6. Mark the draft as published.
7. Commit and return revision/dependency/compatibility response.
8. Add tests for atomic status behavior and catalog replacement.

### Phase 6: API Routing

1. Register new revision routes.
2. Wire handlers to services.
3. Keep current-published routes stable.
4. Change browser-facing existing update behavior to draft-save or remove its use from web code.
5. Add safe error mapping.

### Phase 7: Web Routes And UI

1. Add revision history route.
2. Add revision detail/review route.
3. Add API client functions and typed web contracts.
4. Update dataset detail to link to history and show current published revision metadata.
5. Update existing dataset editor save action to save a draft revision.
6. Add publish action from draft review.
7. Add compatibility/dependency display components.
8. Ensure mobile layouts and SSR read states are usable.
9. Avoid legacy `/app/*` route expansion.

### Phase 8: Tests And Validation

1. Add API unit/integration tests.
2. Add web unit tests if current crate structure supports them.
3. Extend Playwright dataset coverage.
4. Update permission scenario docs if new permission-controlled revision routes/actions are added.
5. Run the verification set.
6. Update `docs/progress-report.md` with kickoff/checkpoint/closeout evidence only when the sprint is actually underway.
7. Update `docs/sprints/sprint-3c-plan.md` if implementation choices differ from this reviewed plan.

## 19. Acceptance Criteria

Sprint 3C is acceptable when all of the following are true:

1. Existing dataset creation still creates a first published revision and previewable materialized output.
2. Existing dataset edit save creates or updates a draft revision instead of direct-publishing.
3. Draft save does not replace current `dataset_sources`, `dataset_fields`, `dataset_scope_nodes`, or current materialized output.
4. Dataset detail and table preview continue to show current published revision before a draft is published.
5. Revision history shows draft, published, and superseded revisions with typed statuses.
6. Revision detail shows metadata, source snapshots, operation pipeline, restriction policy, output fields, generated SQL, materialization metadata, compatibility findings, and dependency impact.
7. Compatibility findings cover at least:
   - added output field;
   - removed output field;
   - changed output field type;
   - changed output field label;
   - changed source/operation semantics;
   - changed restriction policy.
8. Dependency visibility includes:
   - downstream datasets through `dataset_sources.dataset_revision_id`;
   - component versions through `component_versions.dataset_revision_id`;
   - dashboards through `dashboard_components -> component_versions`.
9. Dependency impact uses typed states and explains that dependencies remain pinned after publish.
10. Publish atomically materializes the draft, supersedes exactly the prior current published revision, marks the draft published, updates current catalog rows, and updates current metadata/visibility.
11. After publish, exactly one revision is current published for the dataset.
12. Downstream datasets, component versions, and dashboards remain pinned to the revision IDs they referenced before publish.
13. New API and web code use typed revision/compatibility/dependency/carry-forward contracts rather than scattered raw string comparisons.
14. Scoped reader/manager behavior is covered with positive and negative tests.
15. Existing Sprint 3A and Sprint 3B dataset authoring, SQL preview, table preview, advanced filters, calculated fields, restriction tiers, and source composition continue to work.
16. No active route reintroduces HTML-string route shells, `inner_html`, `/bridge/*`, or JavaScript controller ownership for application UI.
17. Stored revision JSON compatibility with unified `add_source` is verified before closeout.

## 20. Manual Test Plan

### 20.1 Admin Revision Flow

1. Sign in as `admin@tessara.local`.
2. Open `/datasets`.
3. Create a dataset and confirm it has one published revision.
4. Open dataset detail and table preview.
5. Record current revision ID and preview row count.
6. Open the dataset editor.
7. Change output fields, filters, calculated fields, source operation, or restriction policy.
8. Click `Save Draft Revision`.
9. Confirm the app navigates to draft revision review.
10. Confirm compatibility findings are visible.
11. Confirm dependency summary is visible.
12. Return to dataset detail.
13. Confirm current published revision ID and table preview are unchanged.
14. Open revision history.
15. Confirm both current published and draft revisions are listed.
16. Publish the draft.
17. Confirm the draft becomes current published.
18. Confirm the prior revision becomes superseded.
19. Confirm dataset detail and table preview now reflect the new revision.

### 20.2 Compatibility Findings

Create draft changes that produce:

- added output field;
- removed output field;
- changed output field type;
- label-only change;
- source/operation semantic change;
- restriction policy change.

Verify each finding has the expected severity and message.

### 20.3 Dependency Visibility

1. Create Dataset A.
2. Create Dataset B using Dataset A’s current revision as a source.
3. Add or seed a component version that references Dataset A’s current revision.
4. Add or seed a dashboard component using that component version.
5. Draft a new revision for Dataset A.
6. Open the draft review.
7. Confirm Dataset B, the component version, and dashboard appear in dependency impact.
8. Publish Dataset A’s draft.
9. Confirm Dataset B/component/dashboard still reference the old revision.
10. Confirm impact messaging explains pinned behavior.

### 20.4 Scoped Access

1. Sign in as a scoped dataset reader.
2. Confirm visible dataset revisions can be read according to dataset visibility rules.
3. Confirm draft revision detail is hidden unless the user has intended manager access.
4. Sign in as a scoped dataset manager.
5. Save a draft only for a dataset fully inside the manager’s scope.
6. Attempt to set visibility outside scope and confirm rejection.
7. Attempt to publish outside scope and confirm rejection.
8. Sign in as a no-access user and confirm dataset revision routes are unavailable.

### 20.5 Regression Coverage

Re-run Sprint 3A/Sprint 3B dataset flows:

- create dataset;
- edit fields;
- configure source composition with `add_source`;
- configure aggregation;
- configure calculated fields;
- configure row filters;
- configure view restrictions;
- preview generated SQL;
- preview materialized rows;
- save/reopen editor;
- verify restriction tiers.

## 21. Automated Test Plan

Use the current repository’s validation conventions. At minimum, add and run:

```powershell
cargo fmt --all
cargo test -p tessara-api
cargo test -p tessara-web
cargo check -p tessara-web --features hydrate
cargo check -p tessara-web --no-default-features --features ssr
npx playwright test
.\scripts\smoke.ps1
.\scripts\local-launch.ps1
.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"
```

Preferred closeout set if time and local environment allow:

```powershell
.\scripts\validate.ps1
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
npm --prefix end2end test
.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"
.\scripts\smoke.ps1
.\scripts\local-launch.ps1
.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"
```

If `cargo test -p tessara-web` hits the known Windows MSVC PDB/linker limit, follow the Sprint 3B closeout precedent and rerun with reduced debug info:

```powershell
$env:RUSTFLAGS='-C debuginfo=0'
cargo test -p tessara-web
```

### 21.1 API Test Scenarios

Add tests for:

1. initial dataset create creates exactly one published revision;
2. draft save creates a draft without mutating current published catalog;
3. draft save updates an existing open draft rather than creating multiple drafts;
4. draft detail loads from revision snapshots, not current catalog tables;
5. dataset detail/table remain current-published before publish;
6. publish supersedes exactly one current published revision;
7. publish updates current `dataset_sources` and `dataset_fields` from draft compiled output;
8. publish updates current dataset metadata and visibility from revision metadata;
9. prior materialized revision table remains referenced and is not dropped;
10. compatibility finding: added field;
11. compatibility finding: removed field;
12. compatibility finding: type-changed field;
13. compatibility finding: label-only change;
14. compatibility finding: restriction policy change;
15. dependency discovery: dependent dataset;
16. dependency discovery: component version;
17. dependency discovery: dashboard through component version;
18. scoped reader can read allowed published/superseded revision detail;
19. scoped reader cannot publish;
20. scoped manager cannot draft/publish outside scope;
21. no-capability user cannot access revision APIs;
22. stored `add_source` operation snapshots deserialize successfully.

### 21.2 Web Test Scenarios

Add or extend Playwright tests for:

1. admin can create and view first published revision;
2. admin can save existing edit as draft revision;
3. current dataset preview remains unchanged after draft save;
4. revision history lists published and draft rows with status badges;
5. revision detail shows compatibility findings;
6. revision detail shows dependencies and pinned messaging;
7. admin can publish draft and see new current revision;
8. prior revision becomes superseded;
9. dependent assets remain pinned after publish;
10. scoped manager permission boundaries;
11. scoped reader/no-access negative cases;
12. browser console and hydration cleanliness on touched routes.

### 21.3 Pure Unit Tests

Add pure tests for:

- enum `snake_case` serialization;
- compatibility summary derivation;
- output field added/removed/type changed/label changed/order changed;
- restriction policy changes;
- dependency state derivation from findings;
- carry-forward state derivation.

## 22. Documentation Updates

During implementation, update:

- `docs/sprints/sprint-3c-plan.md` with the accepted plan.
- `docs/progress-report.md` at kickoff/checkpoints/closeout only when actual sprint work begins.
- `docs/playwright-permissions-scenarios.md` if new revision-route permission scenarios are implemented.
- Any API contract notes if the project maintains them outside code.

At closeout, record:

- implemented endpoints;
- final UI routes;
- whether `definition_metadata` migration was added;
- compatibility finding taxonomy;
- dependency state taxonomy;
- validation commands and results;
- any known follow-up intentionally deferred to Sprint 5C.

## 23. Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Existing update flow direct-publishes | Draft saves could accidentally mutate current published data | Remove browser use of `PUT` direct publish; add regression tests that draft save does not change current catalog/materialization. |
| Revision snapshots lack metadata/visibility | Draft review cannot faithfully represent full editor changes | Add `definition_metadata` JSON snapshot and backfill. |
| Multiple drafts become possible accidentally | UI and publish behavior becomes ambiguous | Enforce one-open-draft with transaction logic or partial unique index. |
| Stored operation JSON still uses legacy variants | Revision detail/compatibility may fail to deserialize | Migrate stored demo data, fixtures, tests, and revision JSON to unified `add_source`; use temporary compatibility parsing only as an isolated bridge with a removal path. |
| Dependency classification overstates certainty | Users may trust unsafe carry-forward guidance | Use `manual_review` when consumer field usage cannot be proven. |
| Publish transaction partially mutates state | Dataset could have catalog/revision mismatch | Lock dataset, materialize and replace catalog inside one transaction, and add atomicity tests. |
| Superseded materialized tables are dropped | Pinned dependencies may lose historical data | Do not drop old revision tables in Sprint 3C. |
| Scoped users see hidden dependencies | Metadata leak | Apply visibility filtering and use hidden counts only if safe. |
| New web pages revive legacy route shell patterns | Violates roadmap delivery constraints | Implement Datasets feature content in `tessara-web-datasets` with native root `/datasets*` route adapters in `tessara-web`; avoid `/app/*` and bridge patterns. |
| Large `datasets/mod.rs` grows further | Future maintenance cost | Add small revision/compat/dependency modules for new behavior. |

## 24. Final Sprint 3C Definition Of Done

Sprint 3C is done only when:

- revision draft/save/publish lifecycle exists in API and UI;
- current-published dataset detail and preview semantics remain stable;
- typed compatibility, dependency, and carry-forward contracts exist in API and web code;
- downstream dependencies remain pinned after publish and are visibly reported;
- scoped permission behavior is covered by positive and negative tests;
- full validation is run and recorded;
- no active application route regresses to bridge/HTML-string ownership;
- the Sprint 3C progress report documents closeout evidence and any accepted follow-up.

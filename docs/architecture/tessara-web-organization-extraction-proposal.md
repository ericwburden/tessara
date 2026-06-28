# Tessara Web Organization-Crate Extraction Proposal

**Status:** Proposed next refactor after Responses GO
**Target crate:** `crates/tessara-web-organization`
**Governing roadmap:** `docs/architecture/tessara-web-feature-crate-roadmap.md`
**Reference extractions:**

- `docs/architecture/tessara-web-forms-extraction-results.md`
- `docs/architecture/tessara-web-workflows-extraction-results.md`
- `docs/architecture/tessara-web-responses-extraction-results.md`

## 1. Recommendation

Extract Organization next, with one explicit constraint: do not let the deferred Administration refactor retain Organization as a DTO compatibility shelf.

The first Organization inventory found a compact route surface and a feature body of about 2,951 lines, but it also found two cross-feature dependencies that must be removed before the crate move:

- Administration imports Organization DTOs for roles, node type management, and node metadata management.
- Organization imports Administration create/update node payloads for the node editor.

Those are not good long-term boundaries. The Organization extraction should begin by moving Administration-owned DTOs into Administration and Organization-owned node editor payloads into Organization. That adds some temporary duplication, but it keeps forward progress aligned with the roadmap rule that sibling web feature crates must not depend on each other.

Administration remains intentionally deferred. This proposal should decouple it enough for Organization extraction without attempting the larger Administration split.

## 2. Current Inventory Snapshot

Organization currently lives under:

```text
crates/tessara-web/src/features/organization
```

It contains about 2,951 lines across the organization tree, detail page, related work tables, node metadata rendering, node type and node option helpers, and create/edit node editor flows.

Current route registration lives in:

```text
crates/tessara-web/src/routes/organization.rs
```

Routes:

```text
/organization
/organization/new
/organization/:node_id
/organization/:node_id/edit
```

Key blockers from the initial inventory:

- `pages.rs`, `node_editor/create_surface.rs`, and `node_editor/edit_surface.rs` render `AppShell`.
- detail/edit page composition parses `NodeRouteParams` through `require_route_params` inside the feature.
- create/edit/list/detail loaders and actions use root `crate::http` helpers.
- create flow reads query params through root URL helpers.
- organization tables and editor controls use root utility helpers for metadata display, filtering, pagination, labels, nonempty text, and navigation.
- internal paths use `crate::features::organization::*` and crate-relative visibility such as `pub(in crate::features::organization::...)`.
- Organization imports `CreateNodePayload` and `UpdateNodePayload` from `crate::features::administration`.
- Administration imports Organization DTOs:
  - `AdminRoleSummary`
  - `NodeMetadataFieldSummary`
  - `NodeTypeCatalogEntry`
  - `NodeTypeDefinition`
  - `NodeTypeFormLink`
  - `NodeTypeUpsertRequest`
  - `CreateNodeMetadataFieldRequest`
  - `UpdateNodeMetadataFieldRequest`

## 3. Target Architecture

Create:

```text
crates/tessara-web-organization
```

Keep in root `tessara-web`:

```text
routes/organization.rs
NodeRouteParams
require_route_params
AppShell wrapping
route registry
auth/session/navigation policy
hydration/document/CSS/assets/cargo-leptos ownership
```

Move into `tessara-web-organization`:

```text
organization index/detail/new/edit content
organization tree and related-work views
node editor state, loaders, actions, and forms
organization node and node type DTOs needed by Organization UI
feature-local transport helpers
feature-local metadata/filtering/pagination/text/url helpers
feature-local node metadata rendering helpers
```

Keep in root Administration for now:

```text
Administration route adapters and shell-wrapped pages
Administration node type management UI
Administration role/user management UI
Administration-owned role and node type DTOs
Administration create/update node payloads if they are still needed by Administration
```

The important boundary is that root Administration must not import from the new `tessara-web-organization` crate. If Administration needs the same API response shapes, it should own local DTOs until a future Administration sprint designs a better shared contract.

## 4. Public Facade

The new crate should expose only route content components:

```rust
OrganizationIndexContent
OrganizationDetailContent
OrganizationNodeCreateContent
OrganizationNodeEditContent
```

Recommended signatures:

```rust
#[component]
pub fn OrganizationIndexContent() -> impl IntoView;

#[component]
pub fn OrganizationDetailContent(node_id: String) -> impl IntoView;

#[component]
pub fn OrganizationNodeCreateContent() -> impl IntoView;

#[component]
pub fn OrganizationNodeEditContent(node_id: String) -> impl IntoView;
```

Root `routes/organization.rs` should own route params, missing-param handling, and `AppShell` wrapping. The create flow may continue to read organization-specific query params inside the Organization crate through a local helper because those params are feature semantics, not route-adapter semantics.

This intentionally differs from the roadmap's possible create facade that passed `parent_node_id` and `node_type_id` props. Keep the create facade no-arg so root routes do not need to know Organization query-string semantics. The Organization crate should preserve the currently accepted query keys:

```text
node_type_id
parent_node_id
parent_id
```

Do not preserve `crate::features::organization` as a compatibility layer after extraction. The root feature module should be removed once routes import the crate facade directly.

## 5. Dependency Policy

Allowed dependencies for `tessara-web-organization`:

```text
leptos
tessara-web-ui
icon crates already used by the feature
serde
serde_json
gloo-net
js-sys
wasm-bindgen
wasm-bindgen-futures
web-sys
```

Forbidden dependencies:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-forms
tessara-web-workflows
tessara-web-responses
tessara-web-administration
leptos_router
leptos_meta
```

Do not add a shared domain or API-contract crate as part of this extraction unless the inventory finds a stable, non-web contract that is already shared across feature boundaries. The default path is local web DTO ownership, matching the completed Forms, Workflows, and Responses extractions.

## 6. Cross-Boundary Resolution Plan

### 6.1 Administration DTO Decoupling

Before moving Organization into a crate, move the DTOs used by Administration out of Organization and into Administration-local modules.

Administration should own local copies for:

```text
AdminRoleSummary
NodeMetadataFieldSummary
NodeTypeCatalogEntry
NodeTypeDefinition
NodeTypeFormLink
NodeTypeUpsertRequest
CreateNodeMetadataFieldRequest
UpdateNodeMetadataFieldRequest
```

Then update Administration API loaders/actions/components to import those local types. Organization should stop re-exporting DTOs for Administration.

Acceptance audit:

```powershell
rg -n "crate::features::organization|features::organization" crates\tessara-web\src\features\administration
```

Expected result: no matches.

### 6.2 Organization Payload Decoupling

Move or copy the node editor payloads currently imported from Administration into Organization-owned code:

```text
CreateNodePayload
UpdateNodePayload
```

The payloads are used by Organization's node editor against `/api/admin/nodes`, but that endpoint ownership should not force a frontend dependency on Administration.

Acceptance audit:

```powershell
rg -n "crate::features::administration|features::administration" crates\tessara-web\src\features\organization
```

Expected result: no matches.

### 6.3 Route Adapter Prep

Split shell and route parsing from feature content before the physical crate move.

Root `routes/organization.rs` should:

- wrap Organization content in `AppShell`;
- parse `:node_id` through `NodeRouteParams` and `require_route_params`;
- pass `node_id` into detail/edit content;
- keep the existing route paths and SSR mode.

Organization content should:

- render only page bodies;
- accept route IDs as plain `String` props;
- keep feature-specific query param semantics local;
- have no dependency on `AppShell`, `leptos_router`, `leptos_meta`, route params, or root route modules.

Acceptance audit:

```powershell
rg -n "AppShell|NodeRouteParams|require_route_params|leptos_router|leptos_meta|crate::routes" crates\tessara-web\src\features\organization
```

Expected result: no matches.

### 6.4 Helper and UI Rewrites

Rewrite root helper imports before or during extraction:

- root `crate::http` calls become Organization-local browser transport helpers;
- root `crate::utils::metadata` calls become Organization-local metadata helpers or direct `tessara-web-ui` usage;
- root `crate::utils::filtering`, `pagination`, `text`, and `url` calls become local helpers;
- root `crate::ui` imports become direct `tessara_web_ui` imports where the component already lives in the UI crate.

Prefer small local helper copies over broad shared abstractions during this extraction. A later shared-platform pass can merge repeated browser transport and display helpers after enough feature crates have stabilized their local versions.

## 7. Commit Sequence

### O0 - Inventory and Baseline

Capture drift and baseline before changing code.

Commands:

```powershell
git status --short --branch
rustc --version
cargo --version
cargo leptos --version
rg -n "crate::features::(datasets|forms|workflows|responses|organization|administration)|crate::routes|AppShell|leptos_router|leptos_meta|crate::http|crate::utils|crate::types" crates\tessara-web\src\features\organization crates\tessara-web\src\features\administration crates\tessara-web\src\routes
rg -n "features::organization|OrganizationPage|OrganizationDetailPage|OrganizationNewPage|OrganizationEditPage|NodeTypeCatalogEntry|NodeTypeDefinition|NodeTypeFormLink|NodeTypeUpsertRequest|NodeMetadataFieldSummary|AdminRoleSummary|CreateNodePayload|UpdateNodePayload" crates\tessara-web\src
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

Store logs under:

```text
tmp/web-organization-extraction/O0-baseline
```

### O1 - Decouple Administration from Organization DTOs

Move Administration-used DTOs into Administration-local modules and update imports.

Gates:

```powershell
cargo fmt
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
rg -n "crate::features::organization|features::organization" crates\tessara-web\src\features\administration
```

The final `rg` should return no matches.

### O2 - Decouple Organization from Administration Payloads

Move Organization node editor payloads into Organization-local code and update node editor actions/API calls.

Gates:

```powershell
cargo fmt
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
rg -n "crate::features::administration|features::administration" crates\tessara-web\src\features\organization
```

The final `rg` should return no matches.

### O3 - Route Adapter Prep

Move `AppShell` wrapping and route-param parsing into `routes/organization.rs`. Rename exported page functions to content functions while still inside root to make the crate move mechanical.

Gates:

```powershell
cargo fmt
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
rg -n "AppShell|NodeRouteParams|require_route_params|leptos_router|leptos_meta|crate::routes" crates\tessara-web\src\features\organization
```

The final `rg` should return no matches.

### O4 - Extract `tessara-web-organization`

Create the new crate, move Organization code into it, rewrite imports and visibility, wire the root route adapter to the crate facade, and remove the root `features::organization` module.

Gates:

```powershell
cargo fmt
cargo check -p tessara-web-organization --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-web-organization --no-default-features --features ssr
cargo test -p tessara-web-organization --no-default-features --features ssr --no-run -j 1
cargo doc -p tessara-web-organization --no-deps
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

### O5 - Boundary, Build, and Results

Extend boundary checks for `tessara-web-organization`, run full build validation, smoke browser behavior, and write the results document.

Gates:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
cargo tree -p tessara-web-organization --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo tree -p tessara-web-organization --no-default-features --features ssr
cargo leptos build
```

Store logs under:

```text
tmp/web-organization-extraction/O5-clean-current
```

## 8. Measurement Plan

Compare against both the original pre-extraction baseline and the latest post-Responses state.

Known original baseline:

| Probe | Original baseline |
| --- | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s |
| `cargo check -p tessara-api --features ssr` | 392.17s |
| `cargo leptos build` | `>900s` timeout |

Latest post-Responses comparison target:

| Probe | Post-Responses current |
| --- | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 115.00s |
| `cargo check -p tessara-api --features ssr` | 195.49s |
| `cargo check -p tessara-web-responses --no-default-features --features hydrate --target wasm32-unknown-unknown` | 98.05s |
| `cargo check -p tessara-web-responses --no-default-features --features ssr` | 83.09s |
| `cargo test -p tessara-web-responses --no-default-features --features ssr --no-run -j 1` | 290.76s |

Required Organization extraction measurements:

```powershell
cargo check -p tessara-web-organization --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-web-organization --no-default-features --features ssr
cargo test -p tessara-web-organization --no-default-features --features ssr --no-run -j 1
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
```

Also track combined JS/WASM output size and compare it to the post-Responses bundle size. Immediate growth over 5% should block GO unless explained by a measured product change.

## 9. Browser and Behavior Validation

After `cargo leptos watch` is running against a clean local DB, verify:

- `/health` returns healthy.
- unauthenticated Organization routes preserve login redirect behavior.
- authenticated `/organization` returns 200 and renders the organization tree/list.
- authenticated `/organization/new` returns 200.
- authenticated `/organization/new?parent_node_id=<id>&node_type_id=<id>` returns 200 and preselects the parent and node type when data exists.
- authenticated `/organization/new?node_type_id=<id>` returns 200 and preselects the node type when data exists.
- authenticated `/organization/new?parent_id=<id>` returns 200 and preserves the existing parent fallback behavior.
- authenticated `/organization/:node_id` returns 200 and renders detail/related work sections.
- authenticated `/organization/:node_id/edit` returns 200 and renders the edit form.
- create-node and update-node flows still navigate to `/organization/:node_id` after save.

Because Administration DTOs are touched during prep, also smoke:

- `/administration`
- `/administration/users`
- `/administration/roles`
- `/administration/node-types`

The Administration smoke is only a regression guard. It is not a scope expansion into Administration extraction.

## 10. GO / PARTIAL GO / NO-GO Criteria

GO requires:

- `tessara-web-organization` has no dependency on root `tessara-web`, `tessara-api`, or sibling web feature crates.
- `tessara-web-organization` has no dependency on `leptos_router` or `leptos_meta`.
- root routes import only the approved Organization content facade.
- root owns `AppShell`, route params, auth/session/navigation, hydration, document, CSS/assets, and cargo-leptos app ownership.
- Administration does not import Organization web types or the new Organization crate.
- Organization does not import Administration.
- local Organization check/test/doc gates pass.
- root hydrate check and API SSR check pass.
- `cargo leptos build` completes.
- watch detects edits in `crates/tessara-web-organization` and rebuilds the app.
- Organization route smoke passes.
- Administration role/user/node-type smoke passes.
- boundary checker covers the new crate and passes.
- immediate JS/WASM growth remains under 5% or is explicitly explained.

PARTIAL GO is acceptable only if all compile, boundary, build, and route-smoke gates pass but one non-blocking measurement is missing or noisy, such as a watch timing run interrupted by local machine state. The missing measurement must be documented with a rerun command.

NO-GO if any of the following are true:

- Organization extraction requires depending on `tessara-web`, `tessara-api`, or a sibling web feature crate.
- Administration must import from `tessara-web-organization` to keep compiling.
- Organization continues importing from Administration.
- `AppShell`, route params, or route registration move into the Organization crate.
- the public facade expands beyond route content components to preserve legacy root imports.
- implementation requires API/web DTO unification before extraction.
- Organization route smoke fails.
- Administration node-type/role/user smoke regresses.
- `cargo leptos build` fails.
- bundle growth exceeds the threshold without a clear explanation.

## 11. Rollback Plan

Keep commits narrow enough that rollback can stop before the crate move if the dependency inventory changes:

- O1 can be reverted independently if Administration DTO decoupling exposes hidden API assumptions.
- O2 can be reverted independently if Organization node editor payload ownership conflicts with Administration behavior.
- O3 can be reverted independently if shell/route-param separation creates behavioral regressions.
- O4 should be mechanical after O1-O3; if it fails, keep the route-adapter prep and abandon only the physical crate move.

Do not use a compatibility root `features::organization` layer as the rollback mechanism. If rollback is needed, revert the relevant commit instead.

## 12. Expected Retained Debt

The extraction is expected to retain some debt intentionally:

- Organization will own local browser transport helpers, duplicating similar helpers in other feature crates.
- Organization will own local metadata/filtering/pagination/text/url helpers where `tessara-web-ui` does not already provide a stable surface.
- Administration will own local copies of node-type, node-metadata, and role DTOs until the future Administration sprint designs its own smaller crate boundaries.
- Organization web DTOs will remain separate from API DTOs.
- No shared web-platform crate will be introduced during this extraction.
- No Administration extraction or Administration sub-crate split will be attempted during this work.

These are acceptable because they preserve the established feature-crate boundary rules and keep the scope focused on Organization.

## 13. Decision

Proceed with Organization extraction after O0 confirms the inventory has not materially changed.

The implementation should treat DTO decoupling as real architecture work, not a mechanical prelude. Once Administration no longer depends on Organization DTOs and Organization no longer depends on Administration payloads, the remaining extraction should follow the proven Forms, Workflows, and Responses pattern.

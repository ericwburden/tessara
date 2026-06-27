# Tessara Web Responses-Crate Extraction Proposal

**Status:** Proposed next refactor after Workflows GO  
**Target crate:** `crates/tessara-web-responses`  
**Governing roadmap:** `docs/architecture/tessara-web-feature-crate-roadmap.md`  
**Reference extractions:**

- `docs/architecture/tessara-web-forms-extraction-results.md`
- `docs/architecture/tessara-web-workflows-extraction-results.md`

## 1. Recommendation

Extract Responses next. Administration is intentionally deferred to a future sprint.

The short post-Workflows inventory found that Administration is still tightly coupled to Organization web types and currently renders shell-wrapped pages inside the feature. Responses is smaller, more leaf-like, and has the same kind of root-helper and route-adapter work that Forms and Workflows already proved out.

This proposal should be treated as an implementation-ready plan, but not as a reason to skip the W0 inventory and baseline capture. The inventory is deliberately part of the first commit so any drift since this proposal is caught before code movement.

## 2. Current Inventory Snapshot

Responses currently lives under:

```text
crates/tessara-web/src/features/responses
```

It contains about 2,500 lines across list, start, detail, edit, API, loaders, actions, display helpers, value collection, DTOs, and components.

Current route registration lives in:

```text
crates/tessara-web/src/routes/responses.rs
```

Routes:

```text
/responses
/responses/new
/responses/:submission_id
/responses/:submission_id/edit
```

Key blockers from the initial inventory:

- `ResponsesPageContent`, `ResponsesNewPageContent`, `ResponsesDetailPageContent`, and `ResponsesEditPageContent` render `AppShell`.
- detail/edit content parses `SubmissionRouteParams` inside the feature.
- start content reads query params with `crate::utils::url::current_search_param`.
- API/actions/loaders use root `crate::http` helpers.
- list/table/display components use root `crate::utils::{filtering, metadata, pagination, text}` helpers.
- internal paths use `crate::features::responses::*` and `pub(in crate::features::responses)`.
- no direct dependency on Forms, Workflows, Datasets, Administration, or Organization web feature modules appeared in the Responses inventory.

Administration should be deferred because its initial inventory shows direct imports from Organization web types such as `AdminRoleSummary`, `NodeTypeCatalogEntry`, `NodeTypeDefinition`, `NodeTypeFormLink`, `NodeTypeUpsertRequest`, and metadata DTOs. It should be broken into smaller future sprint slices rather than treated as the next monolithic extraction target.

## 3. Target Architecture

Create:

```text
crates/tessara-web-responses
```

Keep in root `tessara-web`:

```text
routes/responses.rs
SubmissionRouteParams
require_route_params
AppShell wrapping
route registry
auth/session/navigation policy
hydration/document/CSS/assets/cargo-leptos ownership
```

Move into `tessara-web-responses`:

```text
response list/start/detail/edit content
response components
response loaders/actions
response browser transport
response web DTOs
response display helpers
response value collection
response-local support helpers
response-local tests
```

Do not preserve `crate::features::responses` as a compatibility namespace. If temporary root shims are needed during extraction, keep them root-only and remove them before GO.

## 4. Public Facade

The target public API should be route-content only:

```rust
pub use detail::ResponseDetailContent;
pub use edit::ResponseEditContent;
pub use list::ResponsesIndexContent;
pub use start::ResponseStartContent;
```

Expected component signatures:

```rust
pub fn ResponsesIndexContent() -> impl IntoView;
pub fn ResponseStartContent() -> impl IntoView;
pub fn ResponseDetailContent(submission_id: String) -> impl IntoView;
pub fn ResponseEditContent(submission_id: String) -> impl IntoView;
```

Root route adapters should become:

```rust
#[component]
fn ResponsesRoute() -> impl IntoView {
    view! {
        <AppShell active_route="responses" title="Responses">
            <tessara_web_responses::ResponsesIndexContent/>
        </AppShell>
    }
}
```

For detail/edit routes, root parses `SubmissionRouteParams` and passes `submission_id` into the extracted content component.

## 5. Dependency Policy

Allowed dependencies:

```text
leptos
tessara-web-ui
icons
serde
serde_json
gloo-net
js-sys
wasm-bindgen
wasm-bindgen-futures
web-sys
```

Use the same dependency pattern as Forms and Workflows unless W0 proves a better split. Browser transport dependencies may remain non-optional if SSR compilation passes and the results document records that this matches the current feature-crate pattern.

Forbidden dependencies:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-forms
tessara-web-workflows
tessara-web-organization
tessara-web-administration
leptos_router
leptos_meta
root auth/session/navigation/shell modules
```

## 6. Cross-Boundary Resolution Plan

### 6.1 Shell and Route Params

Move `AppShell` rendering out of Responses and into `routes/responses.rs`.

Move `SubmissionRouteParams` parsing out of detail/edit content and into root route adapters.

### 6.2 Root HTTP Helpers

Copy minimal local browser transport helpers into `tessara-web-responses`:

```text
IdResponse
send_json_request or send_json_id_request equivalent
redirect_to_login
navigate_to_href
```

Keep these private to Responses for this extraction. Do not create `tessara-web-platform` as part of this proposal.

### 6.3 Root Utility Helpers

Copy tiny support helpers locally unless a helper is already in `tessara-web-ui`:

```text
unique_filter_options
metadata_label
pagination_page_start
text_matches
nonempty_text
current_search_param
```

The goal is to unblock extraction, not to centralize all helper duplication. Record retained helper duplication in the results doc.

### 6.4 Query Parameters

Keep response-start query-string ownership inside the Responses crate with a local browser helper:

```text
workflowAssignmentId
workflow_assignment_id
delegateAccountId
delegate_account_id
```

Do not add `leptos_router` to the Responses crate solely for query strings.

### 6.5 DTOs and Display Helpers

Keep response web DTOs in `tessara-web-responses`.

Do not converge API and web DTOs during extraction. Do not move workflow/form label helpers into a shared crate during this extraction.

## 7. Commit Sequence

### R0 — Inventory and Baseline

Record environment and run initial checks:

```powershell
rustc -Vv
cargo -V
cargo leptos --version
git rev-parse HEAD
git status --short
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
```

Inventory:

```powershell
rg -n "AppShell|require_route_params|SubmissionRouteParams|crate::http|crate::utils|crate::types|crate::routes|leptos_router|leptos_meta|crate::features::(datasets|forms|workflows|organization|administration)" crates\tessara-web\src\features\responses crates\tessara-web\src\routes\responses.rs
```

Baseline route smoke after local seed:

```text
/responses
/responses/new
/responses/:submission_id
/responses/:submission_id/edit
```

### R1 — Root Route-Adapter Preparation

Change root `routes/responses.rs` to own route page wrappers.

Expected root-only responsibilities after R1:

```text
AppShell
SubmissionRouteParams
route registration
```

Expected Responses feature responsibilities after R1:

```text
ResponsesIndexContent
ResponseStartContent
ResponseDetailContent(submission_id)
ResponseEditContent(submission_id)
```

Gate:

```powershell
cargo fmt --all --check
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

Do not proceed if Responses still imports `AppShell`, `SubmissionRouteParams`, `require_route_params`, `leptos_router`, or `leptos_meta`.

### R2 — Extract `tessara-web-responses`

Add workspace member and crate manifest.

Move Responses implementation into:

```text
crates/tessara-web-responses/src
```

Rewrite:

```text
crate::features::responses::* -> crate::*
pub(in crate::features::responses) -> pub(crate)
crate::http::* -> crate-local http helpers
crate::utils::* -> crate-local helpers
crate::ui::* -> tessara_web_ui::* where content still needs generic UI
```

Forward root features:

```toml
hydrate = ["tessara-web-responses/hydrate", ...]
ssr = ["tessara-web-responses/ssr", ...]
```

Remove root `features::responses` implementation before GO.

Gates:

```powershell
cargo fmt --all --check
cargo check -p tessara-web-responses --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-web-responses --no-default-features --features ssr
cargo test -p tessara-web-responses --no-default-features --features ssr --no-run -j 1
cargo doc -p tessara-web-responses --no-deps
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

### R3 — Boundary Checker and Results

Extend `scripts/check-web-crate-boundaries.ps1` for `tessara-web-responses`.

It must reject dependency paths from Responses to:

```text
tessara-web
tessara-api
sibling tessara-web-* feature crates except tessara-web-ui
leptos_router
leptos_meta
```

Source audit should reject:

```text
AppShell
require_route_params
SubmissionRouteParams
crate::routes
crate::features::responses
crate::features::forms
crate::features::workflows
crate::features::organization
crate::features::administration
leptos_router
leptos_meta
```

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
cargo tree -p tessara-web-responses -e features --features ssr --depth 2 --color never
cargo tree -p tessara-web-responses -e features --no-default-features --features hydrate --target wasm32-unknown-unknown --depth 2 --color never
cargo leptos build
```

Run `cargo leptos watch` edit gate:

1. start `cargo leptos watch`;
2. make a behavior-neutral edit inside `crates\tessara-web-responses`;
3. verify `tessara-web-responses` and root `tessara-web` rebuild;
4. verify `Watch updated Front`;
5. verify `/health` returns 200;
6. verify authenticated response routes return 200.

Write results to:

```text
docs/architecture/tessara-web-responses-extraction-results.md
```

Update:

```text
docs/architecture/tessara-web-feature-crate-roadmap.md
```

## 8. Measurement Plan

Use isolated target directories for clean comparisons:

```text
tmp\web-responses-extraction\R0-baseline
tmp\web-responses-extraction\R3-clean-current
tmp\web-responses-extraction\R3-watch
```

Record:

```powershell
cargo check -p tessara-web-responses --no-default-features --features ssr
cargo check -p tessara-web-responses --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-responses --no-default-features --features ssr --no-run -j 1
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
```

Bundle gate:

```text
wasm bytes
js bytes
combined JS/WASM
css bytes
total target/site bytes
```

Cumulative bundle growth is already above the original dataset-pilot baseline, so compare both:

```text
latest Workflows post-extraction bundle
current Responses post-extraction bundle
original dataset-pilot baseline
```

## 9. Browser and Behavior Validation

Required route availability after watch edit:

```text
POST /api/auth/login -> 200
POST /api/demo/seed -> 200
GET /api/submissions -> 200
GET /responses -> 200
GET /responses/new -> 200
GET /responses/:submission_id -> 200
GET /responses/:submission_id/edit -> 200
```

Also verify the direct assignment-start entry still works:

```text
/responses/new?workflowAssignmentId=<id>
/responses/new?delegateAccountId=<id>
```

If suitable IDs are not available from demo seed, record the limitation and at least verify the route renders and the API responds with the expected authorization/error payload.

## 10. GO/PARTIAL/NO-GO Criteria

GO if all are true:

```text
responses-local hydrate check passes
responses-local SSR check passes
responses-local test compile passes
responses-local tests execute or zero tests are documented
root hydrate check passes
API SSR check passes
cargo leptos build passes
cargo-leptos watch detects responses crate edits
authenticated response routes return 200 under watch
boundary checker passes with no permanent exceptions
public API is only the approved route-content facade
root retains route/shell/hydration/CSS/assets ownership
no API/web DTO convergence is required
no sibling web feature dependency is introduced
immediate bundle growth is under 5% or explained
```

PARTIAL GO if the structural extraction compiles and routes pass but one measurement is inconclusive for tooling/environment reasons that are documented and not architectural.

NO-GO if any are true:

```text
responses requires dependency on root tessara-web
responses requires dependency on tessara-web-forms or tessara-web-workflows
route parsing/AppShell/session/auth/CSS/assets must move into responses to compile
public API grows beyond route content to make migration compile
cargo-leptos watch does not detect responses crate edits
full-app regressions exceed thresholds
DTO convergence becomes required to compile
```

## 11. Rollback Plan

If NO-GO:

1. Revert R3 boundary/results tooling changes if Responses-specific.
2. Revert R2 `tessara-web-responses` extraction.
3. Revert R1 route-adapter preparation.
4. Keep any standalone root cleanup only if it is independently useful and passes the full gate matrix.

## 12. Expected Retained Debt

Record these in the results document:

```text
response-local browser transport copied
response-local tiny helpers copied
response web DTOs remain separate from API DTOs
root-owned route/shell/auth/session/navigation retained
no shared web platform crate created
no response/form/workflow contract convergence attempted
```

## 13. Decision

Proceed with Responses extraction next, unless R0 discovers new sibling/root coupling that is materially worse than this inventory.

Organization should be the next major candidate after Responses. Administration is deferred to a future sprint where it can be broken into smaller slices.

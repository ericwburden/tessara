# Tessara Web Forms Crate Extraction Proposal

**Status:** Implementation proposal  
**Target crate:** `crates/tessara-web-forms`  
**Source feature:** `crates/tessara-web/src/features/forms`  
**Governing roadmap:** `docs/architecture/tessara-web-feature-crate-roadmap.md`  
**Prior pilot model:** `tessara-web-ui` + `tessara-web-datasets` dataset pilot  
**Primary goal:** improve focused forms development loops while preserving root app integration

---

## 1. Executive Summary

The dataset pilot validated the core architecture: Tessara can extract large feature areas into frontend crates while keeping a single routed Leptos app, root-owned route adapters, root-owned `AppShell`, root-owned hydration/document/CSS/assets, and one primary API service.

This proposal applies that pattern to Forms.

`forms` is the correct next extraction target because it is large, Leptos-heavy, actively edited, and foundational for later `workflows` and `responses` extraction. Extracting Forms first reduces the risk that Workflows or Responses become dependent on forms web internals.

The proposed extraction creates:

```text
crates/tessara-web-forms
```

as a private workspace `rlib` crate exposing only route-adapter content components to root `tessara-web`.

Root `tessara-web` continues to own:

```text
routes/forms.rs
FormRouteParams
AppShell wrapping
route registry
auth/session/navigation behavior
hydration/document integration
CSS/public assets
cargo-leptos app role
```

The Forms crate owns:

```text
forms list/detail/create/edit content
form builder UI/state/layout/drag/resize/hydrate logic
forms API transport
forms loaders/actions/save orchestration
forms web DTOs/view models
forms display/filtering/version helpers
forms tests
```

This proposal intentionally does **not** converge API/web DTOs, move `AppShell`, create `tessara-web-platform`, extract workflows/responses, rewrite route structure, move CSS/assets, or create placeholder crates.

---

## 2. Decision Context

The accepted roadmap makes Forms the next concrete target after the successful dataset extraction. The roadmap ranks `tessara-web-forms` first among remaining feature-crate candidates because it has very high expected payoff, medium risk, and is foundational for workflows and responses.

The roadmap also states that every future extraction should preserve the dataset pattern:

```text
root route adapters
small content facade
crate-private internals
no sibling web feature dependencies
no API/web DTO convergence in the first extraction
evidence-gated GO/PARTIAL/NO-GO decision
```

This proposal is implementation-ready for the Forms extraction, but it remains a measured pilot stage. It does not authorize Workflows, Responses, Administration, Organization, or Operations extraction.

---

## 3. Goals

### 3.1 Primary goal

Give developers a focused Forms check/test/watch loop:

```powershell
cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-web-forms --no-default-features --features ssr
cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1
```

without requiring every ordinary Forms edit to go through the full root `tessara-web` compilation unit.

### 3.2 Architecture goals

- Preserve the single rooted Leptos application.
- Preserve root route ownership.
- Preserve root `AppShell`, session, navigation, route guards, logout, hydration, document, CSS, and assets.
- Keep `tessara-web-forms` free of root `tessara-web`, `tessara-api`, and sibling web feature dependencies.
- Keep API/web DTO convergence out of scope.
- Keep public API small and intentional.
- Extend the permanent boundary checker to include `tessara-web-forms`.
- Produce measurable GO/PARTIAL/NO-GO results.

---

## 4. Non-Goals

Do **not** include these in the first Forms extraction:

```text
API/web form DTO convergence
canonical ID/timestamp representation
moving CSS/assets
moving AppShell
moving route params
moving root session/navigation/logout/auth guard policy
creating tessara-web-platform
extracting workflows
extracting responses
extracting organization
extracting administration
creating placeholder crates
microservices
route-system rewrite
broad UI redesign
broad forms UX redesign
```

Do not make `tessara-web-forms` depend on:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-workflows
tessara-web-organization
tessara-web-responses
tessara-web-administration
```

---

## 5. Current Forms Structure

Current root module boundary:

```text
crates/tessara-web/src/features/forms/
├── api/
├── attached_nodes/
├── builder/
├── components/
├── create.rs
├── detail.rs
├── detail_content.rs
├── display.rs
├── edit.rs
├── edit_form.rs
├── editor_sections.rs
├── filtering.rs
├── list.rs
├── loaders.rs
├── options_loader.rs
├── pages.rs
├── save/
├── tables/
├── types.rs
├── versions.rs
└── versions_table.rs
```

Current public re-exports in `features/forms/mod.rs` include route pages, builder internals, display helpers, filter helpers, form DTOs, version helpers, and form table components. The extraction should narrow this down so root only imports content facades.

Current route registry:

```text
crates/tessara-web/src/routes/forms.rs
```

currently maps:

```text
/forms
/forms/new
/forms/:form_id
/forms/:form_id/edit
```

to:

```text
FormsPage
FormsNewPage
FormsDetailPage
FormsEditPage
```

Those route components currently live inside `features/forms`.

---

## 6. Current Forms Route/Shell Ownership

The current Forms route pages are not ready to move across a crate boundary unchanged because they still own route parsing and shell rendering.

### 6.1 Current page ownership

| Current component | Current issue | Pilot target |
| --- | --- | --- |
| `FormsPage` | renders `AppShell`; owns list signals and filtering | split into root `FormsPage` adapter + `FormsIndexContent` |
| `FormsNewPage` | renders `AppShell`; owns create page state and builder state | split into root `FormsNewPage` adapter + `FormNewContent` |
| `FormsDetailPage` | parses `FormRouteParams`; renders `AppShell`; loads detail | root parses `FormRouteParams`; feature owns `FormDetailContent(form_id)` |
| `FormsEditPage` | parses `FormRouteParams`; renders `AppShell`; owns edit state | root parses `FormRouteParams`; feature owns `FormEditContent(form_id)` |

### 6.2 Root route adapters after extraction

Root `routes/forms.rs` should become the owner of route pages/adapters:

```rust
use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::AppShell;

use tessara_web_forms::{
    FormDetailContent,
    FormEditContent,
    FormNewContent,
    FormsIndexContent,
};

pub fn form_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/forms") view=FormsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/forms/new") view=FormsNewPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/forms/:form_id") view=FormsDetailPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/forms/:form_id/edit") view=FormsEditPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}

#[component]
fn FormsPage() -> impl IntoView {
    view! {
        <AppShell active_route="forms" title="Forms">
            <FormsIndexContent/>
        </AppShell>
    }
}

#[component]
fn FormsNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="forms" title="Forms">
            <FormNewContent/>
        </AppShell>
    }
}

#[component]
fn FormsDetailPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    view! {
        <AppShell active_route="forms" title="Forms">
            <FormDetailContent form_id=params.form_id/>
        </AppShell>
    }
}

#[component]
fn FormsEditPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    view! {
        <AppShell active_route="forms" title="Forms">
            <FormEditContent form_id=params.form_id/>
        </AppShell>
    }
}
```

### 6.3 Root route titles

Keep the current shell title behavior unless a separate UX change is approved:

| Route | Root `AppShell` title |
| --- | --- |
| `/forms` | `Forms` |
| `/forms/new` | `Forms` |
| `/forms/:form_id` | `Forms` |
| `/forms/:form_id/edit` | `Forms` |

The feature content may still render feature breadcrumbs and `PageHeader` instances, because those are feature presentation rather than root session/navigation policy.

---

## 7. Proposed `tessara-web-forms` Public Facade

### 7.1 Public exports

`crates/tessara-web-forms/src/lib.rs`:

```rust
mod api;
mod attached_nodes;
mod builder;
mod components;
mod create;
mod detail;
mod detail_content;
mod display;
mod edit;
mod edit_form;
mod editor_sections;
mod facade;
mod filtering;
mod list;
mod loaders;
mod options_loader;
mod save;
mod tables;
mod types;
mod versions;
mod versions_table;

pub use facade::{
    FormDetailContent,
    FormEditContent,
    FormNewContent,
    FormsIndexContent,
};
```

No wildcard public re-exports.

### 7.2 Public component signatures

```rust
use leptos::prelude::*;

#[component]
pub fn FormsIndexContent() -> impl IntoView;

#[component]
pub fn FormNewContent() -> impl IntoView;

#[component]
pub fn FormDetailContent(form_id: String) -> impl IntoView;

#[component]
pub fn FormEditContent(form_id: String) -> impl IntoView;
```

### 7.3 Public API rules

The following must stay crate-private:

```text
forms DTOs
form builder state
form builder drafts
save action inputs
loaders
transport helpers
API errors
display helpers
filter helpers
version helpers
tables
attached-node helpers
builder layout/drag/sizing internals
```

Do not make an item `pub` solely because migration is easier. Use `pub(crate)` and internal modules inside `tessara-web-forms`.

### 7.4 Public API review command

```powershell
cargo doc -p tessara-web-forms --no-deps
```

GO requires that the generated docs expose only the approved public facade and any unavoidable generated Leptos props types.

---

## 8. Current Cross-Feature and Root Coupling to Resolve

Before extraction, root/sibling imports inside Forms must be resolved.

### 8.1 Root UI imports

Current Forms uses root UI components including:

```text
AppShell
Breadcrumb*
Button
PageHeader
EmptyState
InfoListTable / InfoRow
Tabs / TabsList / TabsTrigger / TabsContent
Timestamp
DataTable
SearchableDataTable
TableFilterHeader
TablePaginationFooter
empty_view
```

Dataset pilot already moved some generic UI into `tessara-web-ui`, but Forms requires additional UI surface.

### 8.2 Required `tessara-web-ui` additions

Before moving Forms, review and likely move these generic UI items into `tessara-web-ui`:

```text
Button
InfoListTable
InfoRow
Tabs
TabsList
TabsTrigger
TabsContent
Timestamp
SearchableDataTable
TableFilterHeader
empty_view
```

Keep root `crate::ui` as a compatibility facade for non-extracted features.

#### Timestamp caution

`Timestamp` uses hydrate-only browser/JS date formatting. If moving `Timestamp` into `tessara-web-ui`, verify the UI crate feature matrix and dependencies:

```text
js-sys
wasm-bindgen
hydrate-gated browser behavior
```

Do not add broad browser dependencies to `tessara-web-ui` without confirming they are already accepted through the dataset pilot's `DraggablePanelList` move or are otherwise justified.

### 8.3 Root utility imports

Current Forms uses utilities such as:

```text
unique_filter_options
text_matches
sentence_label
pagination_page_start
possibly nonempty_text or related text helpers
```

Pilot policy:

```text
copy tiny pure helpers locally into tessara-web-forms
or move helper privately with tessara-web-ui when it is implementation detail of a UI component
do not create tessara-web-platform or tessara-web-utils yet
```

### 8.4 Root HTTP imports

Current Forms loaders use root `redirect_to_login`, and save/API code likely uses root request helpers in some paths.

Pilot policy:

```text
copy private Forms transport/redirect behavior into tessara-web-forms
record it as intentional debt
do not create tessara-web-platform yet
```

If a platform crate becomes tempting during implementation, stop and write a mini-decision record before adding it.

### 8.5 Root shared feature imports

Forms currently imports root shared feature items such as:

```text
FormAttachmentLink
FormsAttachedNodesSheetData
node_count_label
status_badge_class
```

Pilot policy:

```text
copy narrow forms-local equivalents into tessara-web-forms
or move clearly generic UI behavior into tessara-web-ui
do not depend on root features::shared
```

Recommended treatment:

| Current dependency | Treatment |
| --- | --- |
| `FormAttachmentLink` | move/copy into `tessara-web-forms::types` for forms-owned related node display |
| `FormsAttachedNodesSheetData` | move/copy into `tessara-web-forms::types` beside the forms-owned attached-node display model |
| `node_count_label` | copy to forms display as a private attached-node count helper |
| `status_badge_class` | copy to forms display as a private `form_status_badge_class` or equivalent |

Do not make `tessara-web-forms` depend on root `features::shared`.

### 8.6 Organization dependency

Current Forms imports:

```rust
crate::features::organization::NodeTypeCatalogEntry
```

in create/edit/options/editor sections.

This must not become a dependency on `tessara-web-organization`.

Recommended treatment:

Create a forms-local DTO:

```rust
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormNodeTypeOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) singular_label: String,
}
```

Use it for `/api/node-types` options in form create/edit. Include only the fields Forms actually uses. If implementation discovers more fields are needed, add them locally and document the reason.

Update:

```text
FormCreateOptions.node_types
FormEditOptions.node_types
FormIdentityFields.node_types
FormsNewContent state
FormEditContent state
```

from `NodeTypeCatalogEntry` to `FormNodeTypeOption`.

### 8.7 Workflow dependency

Current Forms related workflow table imports:

```rust
crate::features::workflows::{
    WorkflowSourceMarker,
    workflow_revision_label_from_option,
}
```

This must not become a dependency on `tessara-web-workflows`.

Recommended treatment:

Create forms-local private helpers:

```rust
fn form_related_workflow_revision_label(label: Option<String>) -> String;
```

and:

```rust
#[component]
fn FormRelatedWorkflowSourceMarker(source: String) -> impl IntoView;
```

These may duplicate the current workflow display behavior for `"generated_form"`. Record this as intentional display-helper debt.

Do not promote workflow display helpers to a shared crate during the forms extraction unless a separate review proves they are feature-neutral. The marker is workflow-domain presentation, not generic UI.

### 8.8 Dataset link dependency

Forms related dataset-source tables link to `/datasets/{dataset_id}` but should not depend on `tessara-web-datasets`.

String hrefs are acceptable because root still owns the route registry and feature crates may render links to stable app URLs.

### 8.9 Route/system dependency

Forms must not import:

```text
crate::types::route_params
crate::routes
leptos_router
AppShell
root session/navigation/auth modules
```

after route-adapter preparation.

---

## 9. Cargo Manifests and Feature Rules

### 9.1 New crate manifest

`crates/tessara-web-forms/Cargo.toml`:

```toml
[package]
name = "tessara-web-forms"
version = "0.1.0"
edition = "2024"
publish = false

[features]
default = []
ssr = [
    "leptos/ssr",
    "tessara-web-ui/ssr",
]
hydrate = [
    "leptos/hydrate",
    "tessara-web-ui/hydrate",
    "dep:gloo-net",
    "dep:web-sys",
    "dep:js-sys",
    "dep:wasm-bindgen",
    "dep:wasm-bindgen-futures",
]

[dependencies]
gloo-net = { workspace = true, optional = true }
icons = { workspace = true }
js-sys = { workspace = true, optional = true }
leptos = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tessara-web-ui = { path = "../tessara-web-ui", default-features = false }
wasm-bindgen = { workspace = true, optional = true }
wasm-bindgen-futures = { workspace = true, optional = true }
web-sys = { workspace = true, optional = true }
```

Adjust exact optional dependencies after inventory. Do not include `leptos_router` or `leptos_meta` unless a documented blocker proves root adapters are insufficient.

If `tessara-web-ui` does not define `ssr` / `hydrate` features after the dataset pilot, omit those forwarding entries. Use the current actual `tessara-web-ui` manifest as source of truth.

### 9.2 Root `tessara-web` manifest

Add:

```toml
tessara-web-forms = { path = "../tessara-web-forms", default-features = false }
```

Feature forwarding:

```toml
hydrate = [
    "leptos/hydrate",
    "tessara-web-datasets/hydrate",
    "tessara-web-forms/hydrate",
]

ssr = [
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
    "tessara-web-datasets/ssr",
    "tessara-web-forms/ssr",
]
```

Preserve existing feature entries for `tessara-web-datasets` and `tessara-web-ui`.

Only root `tessara-web` keeps:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

`tessara-web-forms` is an ordinary `rlib` crate and must not declare `cdylib`.

### 9.3 Workspace manifest

Add to workspace members:

```toml
"crates/tessara-web-forms",
```

No changes to `tessara-api` beyond root `tessara-web` feature forwarding should be required.

---

## 10. Boundary Checker Updates

Extend `scripts/check-web-crate-boundaries.ps1`.

### 10.1 Forbidden paths from `tessara-web-forms`

Reject any dependency path from `tessara-web-forms` to:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-workflows
tessara-web-organization
tessara-web-responses
tessara-web-administration
leptos_router
leptos_meta
```

unless an explicit, temporary, documented allowlist entry is approved. No permanent allowlist entry is allowed for the forms extraction.

### 10.2 Source audit for forms crate

Run:

```powershell
rg "AppShell|require_route_params|FormRouteParams|crate::routes|leptos_router|features::organization|features::workflows|features::responses|features::datasets|features::administration|features::shared" crates\tessara-web-forms\src
```

Expected result:

```text
no matches
```

Exceptions must be reviewed before continuing.

### 10.3 Source audit for root route adapters

Run:

```powershell
rg "FormsIndexContent|FormNewContent|FormDetailContent|FormEditContent" crates\tessara-web\src\routes\forms.rs
```

Expected result:

```text
root routes import only the forms facade content components
```

### 10.4 Cargo metadata platforms

Run boundary checker for:

```powershell
cargo metadata --format-version 1 --filter-platform x86_64-pc-windows-msvc
cargo metadata --format-version 1 --filter-platform wasm32-unknown-unknown
```

Continue checking all dependency kinds: normal, build, dev, optional, target-specific, and renamed dependencies.

---

## 11. Implementation Sequence

Each phase should be a separate commit or PR where practical.

### F0 — Inventory and Baseline

No architecture change.

#### F0.1 Record environment

```powershell
git rev-parse HEAD
git status --short
rustc -Vv
cargo -V
cargo leptos --version
cargo metadata --format-version 1 --no-deps
cargo tree -p tessara-web -e features --depth 2 --features ssr --color never
cargo tree -p tessara-web -e features --depth 2 --no-default-features --features hydrate --target wasm32-unknown-unknown --color never
cargo tree -p tessara-api -e features --depth 2 --features ssr --color never
```

#### F0.2 Run import inventory

```powershell
rg "crate::ui" crates\tessara-web\src\features\forms
rg "crate::utils" crates\tessara-web\src\features\forms
rg "crate::http" crates\tessara-web\src\features\forms
rg "crate::types" crates\tessara-web\src\features\forms
rg "crate::state" crates\tessara-web\src\features\forms
rg "crate::features::" crates\tessara-web\src\features\forms
rg "cfg\(feature = ""hydrate""\)|gloo_net|web_sys|wasm_bindgen|js_sys" crates\tessara-web\src\features\forms
rg "#\[test\]|#\[cfg\(test\)\]" crates\tessara-web\src\features\forms
rg "class=""|class=" crates\tessara-web\src\features\forms
```

Record output in:

```text
tmp\web-forms-extraction\F0-inventory
```

and summarize in the result document.

#### F0.3 Baseline commands

Use isolated target directories and a clean site output, following the dataset pilot protocol.

```powershell
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
cargo test -p tessara-web --lib --no-run -j 1
```

#### F0.4 Representative forms edit

Use a reversible behavior-neutral edit.

Candidate baseline patch:

```diff
- <div class="form-grid">
+ <div class="form-grid" data-pilot-benchmark="forms-edit">
```

Apply to:

```text
crates/tessara-web/src/features/forms/editor_sections.rs
```

After extraction, apply the equivalent edit to:

```text
crates/tessara-web-forms/src/editor_sections.rs
```

Run the baseline command matrix after the edit, then revert.

#### F0.5 Representative UI edit

Use a UI component used by Forms and already in `tessara-web-ui`.

Candidate patch:

```diff
- <div class="table-wrap">
+ <div class="table-wrap" data-pilot-benchmark="forms-ui-edit">
```

Apply to `tessara-web-ui` `DataTable` after confirming the actual path in the current local repo.

### F1 — Extend `tessara-web-ui` for Forms

Move only additional generic UI that Forms needs.

Likely additions:

```text
Button
InfoListTable
InfoRow
Tabs
TabsList
TabsTrigger
TabsContent
Timestamp
SearchableDataTable
empty_view
```

Review each candidate before moving.

Do not move:

```text
AppShell
shell/*
feature/shared display concepts
feature-specific helpers
root auth/session/navigation/theme
root route params
browser transport
```

Run:

```powershell
cargo fmt --all --check
cargo check -p tessara-web-ui
cargo doc -p tessara-web-ui --no-deps
cargo tree -p tessara-web-ui -e features --depth 2 --color never
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

Run `cargo leptos watch` UI edit gate if any moved component carries new browser dependencies or is high fan-out.

### F2 — Resolve Forms Cross-Feature Imports In Root

Before crate movement, remove sibling root feature dependencies from Forms while it is still inside `tessara-web`.

Required changes:

1. Replace `NodeTypeCatalogEntry` imports with forms-local `FormNodeTypeOption`.
2. Replace workflow display imports with forms-local helpers.
3. Replace `FormAttachmentLink` shared type usage with forms-local type or forms-owned equivalent.
4. Replace `status_badge_class` dependency with forms-local private helper.
5. Replace `FormsAttachedNodesSheetData` and `node_count_label` dependencies with forms-local equivalents.
6. Replace root `utils` helpers with forms-local helpers where they would otherwise create root dependency.
7. Replace root HTTP helper imports with forms-local transport or temporary local copies.

Acceptance source audit:

```powershell
rg "crate::features::organization|crate::features::workflows|crate::features::responses|crate::features::datasets|crate::features::administration|crate::features::shared|crate::http|crate::utils" crates\tessara-web\src\features\forms
```

Expected:

```text
no matches that would become forbidden once forms moves
```

If matches remain, either resolve them or document them as F3 blockers. Do not move to F3 with forbidden sibling/root references.

Run:

```powershell
cargo fmt --all --check
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

### F3 — Root Forms Route Adapter Preparation

Refactor current Forms route pages so root owns route and shell behavior.

Create shell-free content functions while Forms still lives in root:

```rust
FormsIndexContent
FormNewContent
FormDetailContent
FormEditContent
```

Current route components become root adapters or are moved to `routes/forms.rs`.

After F3, this audit should pass:

```powershell
rg "AppShell|require_route_params|FormRouteParams|crate::routes|leptos_router" crates\tessara-web\src\features\forms
```

Expected:

```text
no matches
```

Run behavior-preserving checks:

```powershell
cargo fmt --all --check
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
```

Run or prepare browser route smoke checks:

```text
/forms
/forms/new
/forms/:form_id
/forms/:form_id/edit
```

### F4 — Extract `tessara-web-forms`

Create:

```text
crates/tessara-web-forms
```

Move forms internals from root into the new crate.

Rewrite root-internal forms paths and visibilities during the move:

```text
crate::features::forms::* -> crate-local module paths
pub(in crate::features::forms) -> pub(crate) or narrower
super paths adjusted to the new crate layout
```

Update:

```text
Cargo.toml workspace members
crates/tessara-web/Cargo.toml
crates/tessara-web/src/routes/forms.rs
```

Remove the old root forms feature module or reduce it to no-op only if root still needs a temporary compatibility surface. Prefer deleting it to avoid confusion.

Root `features/mod.rs` should no longer expose `forms` unless local implementation remains. If route adapters import directly from `tessara_web_forms`, no root `features::forms` re-export should remain.

Post-move namespace audit:

```powershell
rg "crate::features::forms|pub\(in crate::features::forms\)" crates\tessara-web-forms\src
```

Expected:

```text
no matches
```

Run:

```powershell
cargo fmt --all --check
cargo check -p tessara-web-forms --no-default-features --features ssr
cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1
cargo doc -p tessara-web-forms --no-deps
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

### F5 — Boundary, Watch, Bundle, and Results

Update boundary checker to include `tessara-web-forms`.

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
cargo tree -p tessara-web-forms -e features --features ssr --depth 2 --color never
cargo tree -p tessara-web-forms -e features --no-default-features --features hydrate --target wasm32-unknown-unknown --depth 2 --color never
cargo tree -p tessara-web -e features --depth 2 --features ssr --color never
cargo tree -p tessara-web -e features --depth 2 --no-default-features --features hydrate --target wasm32-unknown-unknown --color never
```

Run `cargo leptos watch` edit gate:

1. start `cargo leptos watch`;
2. edit `crates/tessara-web-forms/src/editor_sections.rs` with `data-pilot-benchmark="forms-edit-1"`;
3. verify path dependency detection and front rebuild;
4. verify authenticated `/forms` or `/forms/new` returns HTTP 200;
5. repeat with `forms-edit-2` and `forms-edit-3`;
6. clean benchmark strings afterward.

Run bundle check:

```powershell
cargo leptos build
```

Record `.wasm`, `.js`, combined JS/WASM, and CSS byte counts.

Write results to:

```text
docs/architecture/tessara-web-forms-extraction-results.md
```

---

## 12. Measurement Policy

Use a lighter version of the dataset pilot, but keep enough discipline to avoid false positives.

### 12.1 Required command matrix

Run at baseline and after F4/F5:

```powershell
cargo check -p tessara-web-forms --no-default-features --features ssr
cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

Baseline cannot run `tessara-web-forms` commands because the crate does not exist. Compare those to the old root commands after representative Forms edits.

### 12.2 Required evidence

Record:

```text
git SHA
git status
rustc -Vv
cargo -V
cargo leptos --version
OS/CPU/RAM/power plan
cache policy
target directory
site-root cleanup policy
command logs
elapsed times
bundle-size table
boundary checker output
cargo tree feature output
```

### 12.3 Bundle size

A combined JS/WASM increase over 5% requires explanation.

### 12.4 cargo-leptos watch

The watch gate is mandatory. If `cargo leptos watch` does not detect edits in `tessara-web-forms`, the extraction is NO-GO or PARTIAL at best.

---

## 13. Behavior Validation

### 13.1 Rust checks

```powershell
cargo test -p tessara-web-forms --no-default-features --features ssr
cargo test -p tessara-web --lib
```

If root tests still hit Windows PDB/linker issues, use the established cleanup pattern and report the outcome as root integration/test-link status, not as forms-local failure unless forms-local tests fail.

### 13.2 Browser checks

Run existing E2E tests if present:

```powershell
npx playwright test
```

If that is too broad for local iteration, run or add focused forms route coverage:

```text
/forms
/forms/new
/forms/:form_id
/forms/:form_id/edit
```

Required behavior smoke coverage:

- authenticated forms list loads;
- form list search/filter still works;
- create form page loads;
- create form save/publish buttons remain present and disabled/enabled correctly;
- form detail loads;
- related workflows/dataset sources/attached nodes sections still render;
- edit form page loads;
- form builder controls still render;
- unauthorized/protected route behavior remains root AppShell/auth behavior;
- CSS classes and visible copy remain unchanged.

### 13.3 DOM preservation

Preserve meaningful DOM hierarchy and CSS classes:

```text
route-panel
forms-page
form-editor-panel
form-create-workspace
native-form
form-create-form
form-grid
form-section
form-detail-page
form-detail-content
form-builder*
related-work*
```

Do not rename CSS classes during extraction.

---

## 14. Forms GO/PARTIAL/NO-GO

### GO

GO if all are true:

```text
- forms-local hydrate check passes
- forms-local SSR check passes
- forms-local test compile completes
- root hydrate check regression stays under 10%
- `cargo check -p tessara-api --features ssr` regression stays under 10%
- `cargo leptos build` regression stays under 10%
- cargo-leptos watch detects forms path-dependency edits
- boundary checker passes with no permanent exceptions
- public API is only the approved facade
- root retains route/shell/hydration/CSS/assets ownership
- no API/web DTO convergence was required
- no sibling web crate dependency was introduced
- bundle growth is under 5% or explained
```

### PARTIAL

PARTIAL if:

```text
- forms extraction compiles and behavior is preserved
- forms-local focused loop exists
- root regressions are acceptable
- but focused-loop improvement is modest
```

PARTIAL authorizes keeping the forms crate only if the maintainability and focused-loop benefits are still worth it. It does not authorize workflows extraction.

### NO-GO

NO-GO if any are true:

```text
- tessara-web-forms requires forbidden sibling feature dependencies
- public API grows beyond facade to make migration compile
- cargo-leptos watch does not detect forms crate edits
- full-app regressions exceed thresholds
- DTO convergence becomes required to compile
- route parsing/AppShell/session/auth/CSS/assets must move to make extraction work
- boundary checker requires permanent root/API/sibling exceptions
```

---

## 15. Rollback Plan

Each commit should be independently revertible.

If NO-GO:

1. Revert F5 boundary/results tooling changes if forms-specific.
2. Revert F4 `tessara-web-forms` extraction.
3. Revert F3 root forms route adapters.
4. Revert F2 forms cross-feature import prep only if it was purely migration-driven and not a standalone improvement.
5. Revert F1 UI additions only if they were needed solely for Forms extraction and do not independently improve `tessara-web-ui`.
6. Remove `crates/tessara-web-forms` from workspace members.
7. Remove root `tessara-web` dependency and feature forwarding for `tessara-web-forms`.
8. Rerun validation.

If PARTIAL:

- keep only proven pieces;
- document which commits remain and why;
- do not proceed to Workflows extraction.

If GO:

- keep `tessara-web-forms`;
- write `docs/architecture/tessara-web-forms-extraction-results.md`;
- re-profile before choosing the next feature.

---

## 16. Result Document Template

Create:

```text
docs/architecture/tessara-web-forms-extraction-results.md
```

Required sections:

```text
1. measured environment
2. baseline forms import inventory
3. phase-by-phase implementation summary
4. timing matrix
5. cargo-leptos watch results
6. feature-tree deltas
7. bundle-size deltas
8. public API review
9. dependency-boundary report
10. behavior validation
11. intentional debt retained
12. GO/PARTIAL/NO-GO decision
13. next recommendation
```

Intentional debt section must include:

```text
forms-local browser transport copied
forms-local status/display helpers copied
forms-local node-type option DTO copied
forms-local workflow display helper copied
API/web DTO duplication retained
tessara-web-platform still deferred
```

---

## 17. Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Forms depends on organization/workflows root feature types | Forbidden dependency after extraction | Replace with forms-local DTOs/helpers before moving crate |
| Form builder has browser-heavy code | Feature matrix or wasm build failures | Inventory hydrate-only code and gate browser dependencies |
| Additional UI moves make `tessara-web-ui` high-churn | Shared UI edit cost grows | Move only generic components needed by Forms and measure shared-UI edit |
| API/web form DTOs diverge | Future maintenance debt | Retain intentionally, document, revisit after GO |
| Public API inflation | Long-term maintenance burden | Public facade only; `cargo doc` review |
| Watch does not see forms path dependency | Developer loop not improved | Treat as NO-GO/PARTIAL |
| Full app build regresses | Focused loop improves but integration cost too high | Enforce root regression threshold |
| CSS classes change accidentally | UI regression | DOM/class preservation checks |
| Root tests remain slow/PDB-heavy | Hard to compare full integration | Track forms-local gate separately; treat root test-link as integration signal |

---

## 18. Relationship to Long-Term Roadmap

A GO for Forms authorizes only:

```text
- keeping `tessara-web-forms`
- writing a fresh workflows extraction proposal
```

It does not authorize automatic Workflows extraction.

If Forms extraction succeeds, the next expected architecture discussion is:

```text
docs/architecture/tessara-web-workflows-extraction-proposal.md
```

That proposal must re-inventory current dependencies and decide whether stable workflow contracts require a new domain/contract crate.

---

## 19. Summary

This proposal extends the dataset pilot pattern to Forms.

The extraction is justified because Forms is large, high-churn, and foundational for later Workflows/Responses extraction. The plan is deliberately narrow:

```text
root keeps routes/shell/hydration/CSS/assets
forms exposes only content components
forms internals stay private
API/web DTOs remain duplicated
sibling web dependencies are forbidden
tessara-web-platform remains deferred
```

If the focused Forms loop improves and root regressions stay within threshold, Tessara can keep `tessara-web-forms` and then re-profile for the next feature. If not, the commit-based plan allows a clean rollback without broad architectural damage.

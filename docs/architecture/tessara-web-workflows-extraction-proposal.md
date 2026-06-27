# Tessara Web Workflows Crate Extraction Proposal

**Status:** Implementation proposal  
**Target crate:** `crates/tessara-web-workflows`  
**Source feature:** current Workflows frontend implementation under `tessara-web`  
**Governing roadmap:** `docs/architecture/tessara-web-feature-crate-roadmap.md`  
**Reference extraction:** `docs/architecture/tessara-web-forms-extraction-results.md`  
**Primary goal:** create a focused Workflows development/check/test loop while preserving root app ownership and feature-crate boundary discipline

---

## 1. Executive Summary

The Forms extraction completed successfully and should be treated as the reference model for the next feature-area extraction. It created `crates/tessara-web-forms`, preserved root-owned route adapters and shell behavior, kept Forms internals crate-private, passed the boundary checker, passed Forms-local SSR/hydrate/test gates, preserved browser-facing routes, and produced useful focused-loop commands.

This proposal applies that model to Workflows.

The proposed extraction creates:

```text
crates/tessara-web-workflows
```

as a private workspace `rlib` crate exposing only a small route-content facade to root `tessara-web`.

Root `tessara-web` continues to own:

```text
routes/workflows.rs
WorkflowRouteParams
AppShell wrapping
auth/session/navigation/logout/capability policy
route registry
hydration/document integration
CSS/public assets
cargo-leptos app role
```

`tessara-web-workflows` owns:

```text
workflow list/detail/create/edit/assignments content
workflow editor UI/state/options/steps/payload logic
workflow assignments UI/state/actions/loaders
workflow API transport
workflow loaders/actions
workflow web DTOs/view models
workflow display/filtering helpers
workflow feature-local tests
```

The first Workflows extraction intentionally does **not** converge API/web DTOs, does **not** create `tessara-web-platform`, does **not** create a `tessara-workflows` domain crate unless inventory proves it is required, does **not** move CSS/assets, and does **not** extract Responses/Organization/Administration.

Workflows is more coupled than Forms. The proposal therefore front-loads an explicit import and ownership inventory, especially around:

```text
forms contracts/options
organization node and node-type options
responses/start-response links
operations/pending-work usage
root shared display/helpers
root utils/http/route params
query-string behavior for workflow assignment filters
```

Implementation should proceed only if that inventory confirms Workflows can be extracted without forbidden dependencies on root `tessara-web`, `tessara-api`, `tessara-web-forms`, `tessara-web-datasets`, `tessara-web-responses`, `tessara-web-organization`, or any other sibling feature crate.

---

## 2. Basis and Current Evidence

### 2.1 Forms result summary

The Forms extraction result was **GO**.

Key outcomes to preserve as the model:

- `tessara-web-forms` owns Forms content, builder, loaders, DTOs, transport, save orchestration, and display helpers.
- Root `tessara-web/src/routes/forms.rs` owns route registration, `FormRouteParams`, and `AppShell` wrapping.
- Root `features::forms` was removed rather than preserved as a compatibility layer.
- The Forms public facade exports only `FormsIndexContent`, `FormNewContent`, `FormDetailContent`, and `FormEditContent`, plus generated Leptos props types.
- `tessara-web-forms` has no dependency on root `tessara-web`, `tessara-api`, sibling web feature crates, `leptos_router`, or `leptos_meta`.
- `cargo leptos watch` detected `tessara-web-forms` path-dependency edits.
- Authenticated `/forms`, `/forms/new`, `/forms/:form_id`, and `/forms/:form_id/edit` returned 200 after demo seed.
- Bundle growth remained under the 5% threshold.
- Root web lib test-link still has a Windows/linker caveat, but Forms-local checks, root hydrate, API SSR, `cargo leptos build`, and browser route smoke passed.

This Workflows proposal should follow the same architecture, but it should not assume the coupling shape is identical.

### 2.2 Current Workflows route shape

Current Workflows routes are:

```text
/workflows
/workflows/new
/workflows/assignments
/workflows/:workflow_id
/workflows/:workflow_id/edit
```

The root route module currently wires those URLs to Workflows route page components. The extraction should keep URL registration and route parameter parsing in root, replacing those route page components with root adapters that call a Workflows content facade.

### 2.3 Known Workflows-specific risks

Representative current Workflows code shows coupling to:

```text
root AppShell
root route params
root UI components
root utils
forms summary/options
organization nodes and node type catalog
workflow assignment query parameters
workflow assignment/pending-work DTOs
root shared display conventions
hard-coded links to responses/forms/organization/datasets
```

Some of these may already have been partially resolved by the Forms extraction commit that decoupled workflow and response form contracts. The authoritative source is the current local branch. This proposal therefore requires a W0 inventory before code movement.

---

## 3. Goals

### 3.1 Primary development goal

Create focused Workflows validation commands:

```powershell
cargo check -p tessara-web-workflows --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-web-workflows --no-default-features --features ssr
cargo test -p tessara-web-workflows --no-default-features --features ssr --no-run -j 1
```

so ordinary Workflows list/detail/editor/assignment work can be checked without going through the entire root `tessara-web` compilation unit.

### 3.2 Architecture goals

- Preserve the single rooted Leptos app.
- Preserve root route registry and route params.
- Preserve root `AppShell`, auth/session/navigation/logout/capability policy.
- Preserve root hydration/document/CSS/assets/cargo-leptos ownership.
- Create `tessara-web-workflows` as a private workspace `rlib`.
- Expose only a small route-content facade.
- Keep Workflows internals crate-private.
- Avoid sibling web feature crate dependencies.
- Avoid API/web DTO convergence.
- Extend and pass the boundary checker.
- Preserve current Workflows route behavior.
- Produce a measured GO/PARTIAL/NO-GO result document.

---

## 4. Non-Goals

Do **not** include these in the first Workflows extraction:

```text
API/web workflow DTO convergence
canonical ID/timestamp representation
creating `tessara-web-platform`
creating `tessara-workflows` for symmetry alone
moving AppShell
moving route params
moving route registry
moving auth/session/navigation/logout policy
moving hydration/document integration
moving CSS/public assets
extracting responses
extracting organization
extracting administration
extracting operations
creating placeholder crates
microservices
route-system rewrite
broad workflow UX redesign
broad shared-contract migration
```

Do **not** make `tessara-web-workflows` depend on:

```text
tessara-web
tessara-api
tessara-web-forms
tessara-web-datasets
tessara-web-responses
tessara-web-organization
tessara-web-administration
tessara-web-operations
```

Do **not** use a sibling web feature dependency as a shortcut to get DTOs or display helpers.

---

## 5. Current Workflows Structure and Likely Coupling Risks

### 5.1 Current module structure

Representative current Workflows structure:

```text
crates/tessara-web/src/features/workflows/
├── api.rs
├── assignments/
│   ├── api.rs
│   ├── assignee_picker.rs
│   ├── candidate_pair_picker.rs
│   ├── components.rs
│   ├── create_form.rs
│   ├── detail_sheet.rs
│   ├── display.rs
│   ├── errors.rs
│   ├── filtering.rs
│   ├── lifecycle.rs
│   ├── loaders.rs
│   ├── mobile_cards.rs
│   ├── mutations.rs
│   ├── page_state.rs
│   ├── surface.rs
│   ├── table_row.rs
│   └── types.rs
├── detail.rs
├── detail_tables.rs
├── display.rs
├── editor/
│   ├── action_helpers.rs
│   ├── api.rs
│   ├── available_nodes_picker.rs
│   ├── create.rs
│   ├── create_actions.rs
│   ├── edit.rs
│   ├── edit_form.rs
│   ├── errors.rs
│   ├── options.rs
│   ├── payloads.rs
│   ├── sections.rs
│   ├── seed.rs
│   ├── state.rs
│   ├── step_list.rs
│   ├── steps.rs
│   ├── update_actions.rs
│   ├── update_payloads.rs
│   └── validation.rs
├── list.rs
├── list_panels.rs
├── loaders.rs
├── options.rs
├── pages/
│   ├── assignments.rs
│   ├── detail.rs
│   └── list.rs
├── payloads.rs
└── types.rs
```

This is a coherent feature boundary, but several current page/editor modules still own root concerns such as `AppShell` and route parameter parsing.

### 5.2 Route/shell coupling

Representative modules currently render `AppShell` or parse route params:

| Current area | Current root concern | Target after extraction |
| --- | --- | --- |
| workflows list page | renders `AppShell` | root adapter wraps `WorkflowsIndexContent` |
| workflow detail page | parses `WorkflowRouteParams`; renders `AppShell` | root adapter parses and wraps `WorkflowDetailContent` |
| workflow new/editor page | renders `AppShell` | root adapter wraps `WorkflowNewContent` |
| workflow edit page | parses `WorkflowRouteParams`; renders `AppShell`; uses search param for version | root adapter parses route ID; feature may still own local query-string interpretation if necessary |
| assignments page | renders `AppShell`; owns assignment state effects | root adapter wraps `WorkflowAssignmentsContent` |

### 5.3 Forms coupling

Representative Workflows code has historically used `FormSummary` from Forms for editor options and workflow steps. The Forms extraction result says a commit decoupled workflow and response form contracts, but this must be verified in the current branch.

Allowed patterns:

```text
- Workflows-local `WorkflowFormOption` DTOs populated from `/api/forms`
- Workflows-local rendered/display fields needed for the editor
- future pure contract only if inventory proves it is stable and useful to API + web
```

Forbidden patterns:

```text
tessara-web-workflows -> tessara-web-forms
tessara-web-workflows -> root features::forms
```

### 5.4 Organization/hierarchy coupling

Representative Workflows code has used organization node and node-type DTOs for:

```text
workflow availability
assignment node candidates
workflow editor options
workflow assignment lists
node paths/labels
```

Allowed patterns:

```text
- Workflows-local `WorkflowNodeTypeOption`
- Workflows-local `WorkflowOrganizationNodeOption`
- stable pure hierarchy contract from `tessara-hierarchy`, but only after inventory and dependency review
```

Forbidden patterns:

```text
tessara-web-workflows -> tessara-web-organization
tessara-web-workflows -> root features::organization
```

### 5.5 Responses coupling

Workflows assignment surfaces may link to response start/edit/detail routes or expose pending-work actions. That is acceptable as stable href construction, but not as a dependency on response web code.

Allowed:

```rust
format!("/responses/new?assignment_id={assignment_id}")
format!("/responses/{submission_id}")
```

Forbidden:

```text
tessara-web-workflows -> tessara-web-responses
tessara-web-workflows -> root features::responses
```

If Workflows currently imports response types or response action functions, replace with workflow-local DTOs, href strings, or root-level route composition before extraction.

### 5.6 Operations/Home coupling

Operations and Home may currently import workflow display helpers or assignment DTOs. After extraction:

```text
root features should not import workflow internals through root `features::workflows`
```

Inventory must identify any root-owned consumers of Workflows types/helpers and resolve them by one of:

1. keep the consumer root-local and pass display-ready data;
2. copy a narrow local DTO/helper;
3. move a stable pure contract to an allowed domain crate;
4. explicitly defer extraction if the dependency cannot be untangled.

Do not keep a root `features::workflows` compatibility module indefinitely.

### 5.7 UI/shared/helper coupling

Workflows likely uses:

```text
tessara-web-ui components
root utility text/filtering/pagination/url helpers
root shared status/link helpers
browser APIs
icons
gloo-net
serde/serde_json
```

Policy:

- use `tessara-web-ui` for generic components;
- copy tiny pure helpers locally if they are not already available through `tessara-web-ui`;
- keep browser transport local for now;
- do not create `tessara-web-platform` during Workflows extraction unless an explicit decision record proves the third copy of transport/helper logic is now maintenance-significant and can be made policy-neutral.

---

## 6. Route/Shell Ownership Split

### 6.1 Root route adapters

Root `crates/tessara-web/src/routes/workflows.rs` should own the route components/adapters after extraction.

Hypothetical post-extraction route module:

```rust
use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{WorkflowRouteParams, require_route_params};
use crate::ui::AppShell;

use tessara_web_workflows::{
    WorkflowAssignmentsContent,
    WorkflowDetailContent,
    WorkflowEditContent,
    WorkflowNewContent,
    WorkflowsIndexContent,
};

pub fn workflow_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/workflows") view=WorkflowsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/workflows/new") view=WorkflowsNewPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/workflows/assignments") view=WorkflowAssignmentsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/workflows/:workflow_id") view=WorkflowsDetailPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/workflows/:workflow_id/edit") view=WorkflowsEditPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}

#[component]
fn WorkflowsPage() -> impl IntoView {
    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowsIndexContent/>
        </AppShell>
    }
}

#[component]
fn WorkflowsNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowNewContent/>
        </AppShell>
    }
}

#[component]
fn WorkflowAssignmentsPage() -> impl IntoView {
    view! {
        <AppShell active_route="workflows" title="Workflow Assignments">
            <WorkflowAssignmentsContent/>
        </AppShell>
    }
}

#[component]
fn WorkflowsDetailPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowDetailContent workflow_id=params.workflow_id/>
        </AppShell>
    }
}

#[component]
fn WorkflowsEditPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowEditContent workflow_id=params.workflow_id/>
        </AppShell>
    }
}
```

### 6.2 Root route titles

Preserve the current shell title behavior unless a separate UX change is approved:

| Route | Root `AppShell` title |
| --- | --- |
| `/workflows` | `Workflows` |
| `/workflows/new` | `Workflows` |
| `/workflows/assignments` | `Workflow Assignments` |
| `/workflows/:workflow_id` | `Workflows` |
| `/workflows/:workflow_id/edit` | `Workflows` |

Feature content can still render breadcrumbs, `PageHeader`, route panels, and workflow-specific page headings because those are presentation concerns, not root shell/session/navigation concerns.

### 6.3 Query-string ownership

Workflows has route behavior that may use query parameters:

```text
/workflows/new?form_id=...
/workflows/:workflow_id/edit?version_id=...
/workflows/assignments?workflow_id=...
/workflows/assignments?assignment_id=...
```

Preferred policy:

- root route adapters own path parameters;
- feature content may own feature-local query-string interpretation when the query modifies internal page state and does not affect route matching;
- if feature content uses query-string helpers, prefer local `current_search_param` copied from prior helpers or a small local helper;
- do not depend on root `crate::utils::url`;
- do not add `leptos_router` solely to read search params unless no browser/local helper is sufficient.

Record all query parameters and their owners in the W0 inventory.

---

## 7. Proposed Public Facade

### 7.1 `tessara-web-workflows/src/lib.rs`

```rust
#![recursion_limit = "512"]

mod api;
mod assignments;
mod detail;
mod detail_tables;
mod display;
mod editor;
mod facade;
mod list;
mod list_panels;
mod loaders;
mod payloads;
mod types;

#[cfg(feature = "hydrate")]
mod options;

pub use facade::{
    WorkflowAssignmentsContent,
    WorkflowDetailContent,
    WorkflowEditContent,
    WorkflowNewContent,
    WorkflowsIndexContent,
};
```

The `recursion_limit` should be included if Workflows Leptos view/test compile hits the same recursion behavior as Forms. If not required, omit it.

Browser-only transport and DOM helpers must be handled deliberately. If `gloo-net`, `web-sys`, `js-sys`, `wasm-bindgen`, or `wasm-bindgen-futures` remain optional hydrate-only dependencies, then every module or function that directly requires them must be gated so `cargo check -p tessara-web-workflows --no-default-features --features ssr` compiles. If Workflows needs any of those crates in SSR builds, make that dependency non-optional and document the reason in the results file rather than leaving the manifest/source split ambiguous.

### 7.2 Public component signatures

```rust
use leptos::prelude::*;

#[component]
pub fn WorkflowsIndexContent() -> impl IntoView;

#[component]
pub fn WorkflowNewContent() -> impl IntoView;

#[component]
pub fn WorkflowAssignmentsContent() -> impl IntoView;

#[component]
pub fn WorkflowDetailContent(workflow_id: String) -> impl IntoView;

#[component]
pub fn WorkflowEditContent(workflow_id: String) -> impl IntoView;
```

### 7.3 Optional facade refinement

If inventory shows the assignment route must receive root-owned initial filters, this alternative is acceptable:

```rust
#[component]
pub fn WorkflowAssignmentsContent(
    #[prop(optional)] initial_workflow_id: Option<String>,
    #[prop(optional)] initial_assignment_id: Option<String>,
) -> impl IntoView;
```

But the preferred pattern is to keep assignment query-string interpretation inside Workflows content because it is feature-local page state.

### 7.4 Public API rules

The following must remain crate-private:

```text
workflow DTOs
assignment DTOs
workflow editor state
workflow step drafts
workflow save intent
save action inputs
API errors
transport helpers
loaders
actions
display helpers
filtering helpers
assignment page state
assignment mutations
workflow editor sections
workflow detail/list tables
```

Do not make any item public solely for tests or migration convenience.

### 7.5 Public API review

Run:

```powershell
cargo doc -p tessara-web-workflows --no-deps
```

GO requires that generated docs expose only the approved route-content facade and unavoidable generated Leptos props types.

---

## 8. Cross-Feature Dependencies to Resolve Before Extraction

### 8.1 Forms option contracts

Known risk:

```text
workflow editor create/edit options have historically consumed FormSummary/FormVersionSummary-like shapes
```

Required resolution:

- verify whether Forms extraction already created workflow-local form option DTOs;
- if not, introduce Workflows-local DTOs before extraction.

Suggested Workflows-local types:

```rust
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowFormOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    #[serde(default)]
    pub(crate) versions: Vec<WorkflowFormVersionOption>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowFormVersionOption {
    pub(crate) id: String,
    pub(crate) version_label: Option<String>,
    pub(crate) status: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) field_count: i64,
    #[serde(default)]
    pub(crate) assignment_nodes: Vec<WorkflowFormAssignmentNodeOption>,
}
```

Use only fields Workflows actually reads.

Do **not** depend on `tessara-web-forms`.

### 8.2 Organization and hierarchy contracts

Known risk:

```text
workflow editor/assignments use node types and organization nodes
```

Required resolution:

- replace root organization DTOs with Workflows-local DTOs, unless a pure existing hierarchy contract is already available and suitable.

Suggested types:

```rust
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowNodeTypeOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowNodeOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) parent_node_name: Option<String>,
}
```

Adjust exact fields after inventory.

Do **not** depend on `tessara-web-organization`.

### 8.3 Responses links and pending work

Known risk:

```text
workflow assignments may link to response start/edit/detail routes or pending-work views
```

Allowed:

```text
href string construction
local DTOs describing assignment/pending work
```

Forbidden:

```text
imports from `tessara-web-responses`
imports from root `features::responses`
calling response actions directly
```

If a workflow assignment action starts a response, stop and evaluate ownership. The current architecture after prior audits moved response-start behavior into Responses. Workflows should not regain that behavior during extraction.

### 8.4 Operations/Home consumers

Known risk:

```text
root operations/home may import workflow assignment display helpers or DTOs
```

Required inventory:

```powershell
rg "features::workflows|tessara_web_workflows|Workflow" crates\tessara-web\src\features\operations crates\tessara-web\src\features\home crates\tessara-web\src
```

Resolution options:

- keep operations/home root-owned and copy minimal display helpers locally;
- move stable DTOs to a domain/contract crate only if truly shared and pure;
- expose no extra public API from `tessara-web-workflows` merely to support root aggregator pages.

### 8.5 Shared display helpers

Known risk:

```text
status_badge_class
text helpers
pagination helpers
timestamp helpers
filtering helpers
```

Policy:

- if generic UI already owns a component, use `tessara-web-ui`;
- if a helper is tiny and feature-specific, copy into `tessara-web-workflows`;
- if a helper is truly generic and already exists in `tessara-web-ui`, use it;
- do not create `tessara-web-platform` for text/pagination alone.

### 8.6 Browser transport

Workflows should own private browser transport for the first extraction, as Datasets and Forms did.

Record this as intentional debt:

```text
tessara-web-workflows has private browser transport and unauthorized handling copied from existing feature/root patterns
```

After Workflows, reassess whether three feature-local transport copies justify a small, policy-neutral `tessara-web-platform`.

### 8.7 UI components

Workflows likely uses generic UI already moved for Forms/Datasets, plus possibly:

```text
Button
Breadcrumb
EmptyState
PageHeader
InfoListTable
Tabs
Timestamp
DataTable/SearchableDataTable
TablePaginationFooter
Combobox
DraggablePanelList
```

Required pre-implementation inventory:

- identify any root `crate::ui` items still not available from `tessara-web-ui`;
- move only domain-neutral UI components needed by Workflows;
- do not move AppShell or shell modules;
- do not move workflow-specific display components into `tessara-web-ui`.

---

## 9. Cargo Manifest and Feature Forwarding Plan

### 9.1 New crate manifest

`crates/tessara-web-workflows/Cargo.toml`:

```toml
[package]
name = "tessara-web-workflows"
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

Adjust exact optional dependencies after W0 inventory. The manifest above assumes browser transport and browser APIs are hydrate-only; that requires source-level `#[cfg(feature = "hydrate")]` gating for every direct use of those crates. Do not include `leptos_router` or `leptos_meta` unless inventory proves there is no root-adapter alternative.

If `tessara-web-ui` does not expose `ssr` / `hydrate` features in the current local branch, omit those forwarding entries and use the actual UI manifest as source of truth.

### 9.2 Root `tessara-web` manifest

Add:

```toml
tessara-web-workflows = { path = "../tessara-web-workflows", default-features = false }
```

Feature forwarding:

```toml
hydrate = [
    "leptos/hydrate",
    "tessara-web-datasets/hydrate",
    "tessara-web-forms/hydrate",
    "tessara-web-workflows/hydrate",
]

ssr = [
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
    "tessara-web-datasets/ssr",
    "tessara-web-forms/ssr",
    "tessara-web-workflows/ssr",
]
```

Preserve existing entries for Datasets, Forms, and UI.

Only root `tessara-web` keeps:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

`tessara-web-workflows` is an ordinary `rlib` crate and must not declare `cdylib`.

### 9.3 Workspace manifest

Add:

```toml
"crates/tessara-web-workflows",
```

No direct `tessara-api` dependency changes should be required beyond root `tessara-web` forwarding.

---

## 10. Boundary Checker Updates

Extend `scripts/check-web-crate-boundaries.ps1`.

### 10.1 Forbidden dependency paths

Reject dependency paths from `tessara-web-workflows` to:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-forms
tessara-web-responses
tessara-web-organization
tessara-web-administration
tessara-web-operations
leptos_router
leptos_meta
```

No permanent allowlist entry is allowed for the first Workflows extraction.

### 10.2 Source audit

Run after W2 and W3:

```powershell
rg "AppShell|require_route_params|WorkflowRouteParams|crate::routes|leptos_router|leptos_meta|features::forms|features::organization|features::responses|features::datasets|features::administration|features::operations|features::shared|crate::http|crate::utils|crate::types" crates\tessara-web-workflows\src
```

Expected result:

```text
no forbidden matches
```

If matches remain, either resolve them or stop. Do not move forward with hidden root/sibling imports.

### 10.3 Root route facade audit

Run:

```powershell
rg "WorkflowsIndexContent|WorkflowNewContent|WorkflowAssignmentsContent|WorkflowDetailContent|WorkflowEditContent" crates\tessara-web\src\routes\workflows.rs
```

Expected:

```text
root routes import only the approved Workflows facade content components
```

### 10.4 Metadata platforms

Run for both platforms:

```powershell
cargo metadata --format-version 1 --filter-platform x86_64-pc-windows-msvc
cargo metadata --format-version 1 --filter-platform wasm32-unknown-unknown
```

Checker must inspect:

```text
normal dependencies
build dependencies
dev dependencies
optional dependencies
target-specific dependencies
renamed package dependencies
transitive workspace paths
```

---

## 11. Step-by-Step Implementation Sequence

Each phase should be a separate commit or PR where practical.

### W0 — Inventory and Baseline

No architecture changes.

#### W0.1 Record environment

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

#### W0.2 Workflows import inventory

```powershell
rg "crate::ui" crates\tessara-web\src\features\workflows
rg "crate::utils" crates\tessara-web\src\features\workflows
rg "crate::http" crates\tessara-web\src\features\workflows
rg "crate::types" crates\tessara-web\src\features\workflows
rg "crate::state" crates\tessara-web\src\features\workflows
rg "crate::features::" crates\tessara-web\src\features\workflows
rg "cfg\(feature = ""hydrate""\)|gloo_net|web_sys|wasm_bindgen|js_sys" crates\tessara-web\src\features\workflows
rg "#\[test\]|#\[cfg\(test\)\]" crates\tessara-web\src\features\workflows
rg "class=""|class=" crates\tessara-web\src\features\workflows
```

#### W0.3 Root consumers inventory

```powershell
rg "features::workflows|WorkflowAssignmentsPage|WorkflowsPage|WorkflowsDetailPage|WorkflowsEditPage|WorkflowsNewPage|Workflow" crates\tessara-web\src
```

Classify each match as:

```text
root route adapter
workflow internals
root-owned aggregator
cross-feature dependency to resolve
test-only reference
```

#### W0.4 Query-param inventory

Search:

```powershell
rg "current_search_param|workflow_id|assignment_id|version_id|form_id" crates\tessara-web\src\features\workflows
```

Document:

| Query param | Current owner | Target owner |
| --- | --- | --- |
| `form_id` on `/workflows/new` | workflow new page | workflow content local helper |
| `version_id` on `/workflows/:id/edit` | workflow edit page | workflow content local helper |
| `workflow_id` on `/workflows/assignments` | workflow assignment lifecycle | workflow content local helper |
| `assignment_id` on `/workflows/assignments` | workflow assignment lifecycle | workflow content local helper |

Adjust if current branch differs.

#### W0.5 Baseline commands

Use isolated target directories and the established pilot protocol:

```powershell
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
cargo test -p tessara-web --lib --no-run -j 1
```

#### W0.6 Representative Workflows edit

Use a reversible behavior-neutral edit.

Candidate patch:

```diff
- <section class="route-panel workflows-page">
+ <section class="route-panel workflows-page" data-pilot-benchmark="workflows-edit">
```

Possible source before extraction:

```text
crates/tessara-web/src/features/workflows/editor/create.rs
```

After extraction, apply the equivalent edit under:

```text
crates/tessara-web-workflows/src/editor/create.rs
```

If implementation changes that file shape before measurement, choose the closest stable workflow editor section and record the exact patch.

#### W0.7 Representative UI edit

Use a `tessara-web-ui` component used by Workflows and already shared by Datasets/Forms, such as `DataTable` or `PageHeader`.

Record exact patch and cleanup.

---

### W1 — Resolve Workflows Cross-Feature Imports in Root

Before crate movement, eliminate forbidden sibling/root dependencies while Workflows still lives in root.

Required checks:

```powershell
rg "crate::features::forms|crate::features::organization|crate::features::responses|crate::features::datasets|crate::features::administration|crate::features::operations|crate::features::shared|crate::http|crate::utils|crate::types|crate::state|crate::routes|leptos_router|leptos_meta" crates\tessara-web\src\features\workflows
```

Resolve each match using one of:

1. workflow-local DTO/helper;
2. existing `tessara-web-ui` component;
3. local pure helper copy;
4. allowed domain crate only after explicit review;
5. root route adapter composition.

Known likely work:

- replace forms DTO imports with Workflows-local form option DTOs if not already done;
- replace organization node/node-type DTO imports with Workflows-local option DTOs;
- replace shared display helpers with Workflows-local display helpers;
- copy tiny text/pagination/url helpers locally where needed;
- keep private browser transport and unauthorized handling local;
- ensure response-related behavior remains links or local DTOs, not response web imports.
- classify `crate::types`, route-param, router, and shell matches immediately; root route-param/shell matches may be resolved in W2, but they must not be lost from the blocking inventory.

Run:

```powershell
cargo fmt --all --check
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

Do not proceed to W2 with unresolved sibling/root helper imports. Route-param, router, and shell matches may carry into W2 only when they are explicitly recorded as route-adapter work and are covered by the W2 no-match audit.

---

### W2 — Workflows Route-Adapter Preparation

Before crate movement, refactor current Workflows route pages into shell-free content components.

Create or expose content components inside the current root feature first:

```text
WorkflowsIndexContent
WorkflowNewContent
WorkflowAssignmentsContent
WorkflowDetailContent
WorkflowEditContent
```

Root `routes/workflows.rs` becomes the owner of route components and `AppShell`.

After W2, this audit should pass:

```powershell
rg "AppShell|require_route_params|WorkflowRouteParams|crate::types|crate::routes|leptos_router|leptos_meta" crates\tessara-web\src\features\workflows
```

Expected:

```text
no matches
```

Exception:

```text
feature-local query-string helpers may still use browser APIs or local search-param helpers, but not root route params or `leptos_router`
```

Run behavior-preserving checks:

```powershell
cargo fmt --all --check
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
```

Browser smoke routes:

```text
/workflows
/workflows/new
/workflows/assignments
/workflows/:workflow_id
/workflows/:workflow_id/edit
```

---

### W3 — Extract `tessara-web-workflows`

Create:

```text
crates/tessara-web-workflows
```

Move Workflows internals from root into the new crate.

Update:

```text
Cargo.toml workspace members
crates/tessara-web/Cargo.toml
crates/tessara-web/src/routes/workflows.rs
scripts/check-web-crate-boundaries.ps1
```

Remove root `features::workflows` implementation. Prefer deleting the root module entirely rather than preserving a compatibility surface. If a temporary compatibility layer is necessary, it must be root-only, documented, and removed before GO.

Root routes should import only:

```rust
use tessara_web_workflows::{
    WorkflowAssignmentsContent,
    WorkflowDetailContent,
    WorkflowEditContent,
    WorkflowNewContent,
    WorkflowsIndexContent,
};
```

Run:

```powershell
cargo fmt --all --check
cargo check -p tessara-web-workflows --no-default-features --features ssr
cargo check -p tessara-web-workflows --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-workflows --no-default-features --features ssr --no-run -j 1
cargo doc -p tessara-web-workflows --no-deps
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
```

---

### W4 — Boundary, Feature, Watch, Bundle, and Result Gates

Run boundary and feature-tree gates:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1

cargo tree -p tessara-web-workflows -e features --features ssr --depth 2 --color never
cargo tree -p tessara-web-workflows -e features --no-default-features --features hydrate --target wasm32-unknown-unknown --depth 2 --color never

cargo tree -p tessara-web -e features --features ssr --depth 2 --color never
cargo tree -p tessara-web -e features --no-default-features --features hydrate --target wasm32-unknown-unknown --depth 2 --color never
```

Run `cargo leptos watch` edit gate:

1. start `cargo leptos watch`;
2. edit a workflow source file, for example `crates/tessara-web-workflows/src/editor/create.rs`;
3. add a temporary `data-pilot-benchmark="workflows-edit-1"` marker to an existing element;
4. verify path dependency detection and front rebuild;
5. verify `/health` returns 200;
6. authenticate and verify all required Workflows routes return 200;
7. repeat for `workflows-edit-2` and `workflows-edit-3`;
8. clear authentication and verify protected Workflows routes still redirect to login or render the expected unauthenticated guard behavior;
9. clean benchmark strings;
10. verify cleanup audit:

```powershell
rg "data-pilot-benchmark|workflows-edit-[123]" crates\tessara-web-workflows crates\tessara-web
```

Bundle check:

```powershell
cargo leptos build
```

Record:

```text
.wasm total
.js total
combined JS/WASM
.css total
```

Write results to:

```text
docs/architecture/tessara-web-workflows-extraction-results.md
```

---

## 12. Compile, Watch, Browser, and Bundle Measurement Plan

### 12.1 Required command matrix

Run at baseline and after extraction:

```powershell
cargo check -p tessara-web-workflows --no-default-features --features ssr
cargo check -p tessara-web-workflows --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-workflows --no-default-features --features ssr --no-run -j 1
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

Baseline cannot run `tessara-web-workflows` commands because the crate does not exist. Compare those to old root commands after the representative Workflows edit.

### 12.2 Clean-target comparison

Follow the same clean-target policy as Forms:

```text
isolated target directories
SCCACHE disabled for decision-grade measurements
target/site removed before clean cargo-leptos build comparisons
fixed timeout for long-running root test-link probes
```

Record warm-cache loop evidence separately from clean-target evidence.

### 12.3 Watch gate

Required route availability after watch edit:

```text
/health returns 200
authenticated /workflows returns 200
authenticated /workflows/new returns 200
authenticated /workflows/assignments returns 200
authenticated /workflows/:workflow_id returns 200 after demo seed
authenticated /workflows/:workflow_id/edit returns 200 after demo seed
unauthenticated /workflows redirects to login or renders the expected login guard
unauthenticated /workflows/assignments redirects to login or renders the expected login guard
unauthenticated /workflows/:workflow_id redirects to login or renders the expected login guard after demo seed
```

If seeded data does not include a workflow, use `POST /api/demo/seed` or the existing local seeding command and record the workflow ID.

### 12.4 Bundle gate

A combined JS/WASM increase over 5% requires explanation.

Workflows may move code but should not meaningfully increase shipped bundle through duplication. If local transport/helper duplication adds measurable size, record it as retained debt.

---

## 13. Behavior Validation Plan

### 13.1 Rust checks

```powershell
cargo test -p tessara-web-workflows --no-default-features --features ssr
cargo test -p tessara-web --lib -j 1
```

Root lib test-link failure may remain a known Windows integration caveat. Treat it as root integration/test-link status unless it introduces a new failure mode attributable to Workflows extraction.

### 13.2 Browser checks

Run existing E2E suite if practical:

```powershell
npx playwright test
```

At minimum, run or add focused Workflows route coverage:

```text
/workflows
/workflows/new
/workflows/assignments
/workflows/:workflow_id
/workflows/:workflow_id/edit
```

Required behavior smoke coverage:

- authenticated workflows list loads;
- workflows list search/filter still works;
- create workflow page loads;
- workflow editor options load;
- create workflow from `form_id` query seed still works if supported;
- create workflow save action still works;
- workflow detail loads;
- workflow detail assignments/steps/versions tables render;
- edit workflow page loads;
- edit `version_id` query behavior still works if supported;
- assignments page loads;
- assignments `workflow_id` and `assignment_id` query behavior still works;
- assignment create/toggle actions still work;
- unauthorized/protected route behavior remains root AppShell/auth behavior;
- visible copy and CSS classes remain unchanged.

### 13.3 DOM/class preservation

Preserve meaningful DOM hierarchy and CSS classes, including:

```text
workflows-page
workflow-detail-page
workflow-edit-page
workflow-create-form
workflow-assignment*
workflow-source-marker
workflow-assignment-step-meta
related-work*
native-form
route-panel
form-message
organization-state
```

Do not rename CSS classes during extraction.

---

## 14. GO/PARTIAL/NO-GO Criteria

### GO

GO if all are true:

```text
workflow-local hydrate check passes
workflow-local SSR check passes
workflow-local test compile completes
workflow-local test execution passes or has zero tests and compile gate passes
root hydrate check regression stays under 10%
`cargo check -p tessara-api --features ssr` regression stays under 10%
`cargo leptos build` regression stays under 10%
cargo-leptos watch detects workflow crate edits
authenticated Workflows routes return 200 under watch
unauthenticated representative Workflows routes preserve protected-route redirect/login behavior
boundary checker passes with no permanent exceptions
public API is only the approved facade
root retains route/shell/hydration/CSS/assets ownership
no API/web DTO convergence was required
no sibling web feature dependency was introduced
bundle growth is under 5% or explained
```

### PARTIAL

PARTIAL if:

```text
Workflows extraction compiles and behavior is preserved
workflow-local focused loop exists
root regressions are acceptable
but focused-loop improvement is modest, or the extraction retains more local debt than expected
```

PARTIAL authorizes keeping `tessara-web-workflows` only if the maintainability and focused-loop benefits justify the retained debt. It does not authorize Administration or Responses extraction.

### NO-GO

NO-GO if any are true:

```text
tessara-web-workflows requires dependency on root tessara-web
tessara-web-workflows requires dependency on tessara-api
tessara-web-workflows requires dependency on tessara-web-forms, tessara-web-datasets, tessara-web-responses, tessara-web-organization, or any sibling feature crate
public API grows beyond facade to make migration compile
cargo-leptos watch does not detect workflow crate edits
full-app regressions exceed thresholds
DTO convergence becomes required to compile
route parsing/AppShell/session/auth/CSS/assets must move to make extraction work
boundary checker requires permanent root/API/sibling/router/meta exceptions
```

---

## 15. Rollback Plan

Each commit should be independently revertible.

If NO-GO:

1. Revert W4 boundary/results tooling changes if Workflows-specific.
2. Revert W3 `tessara-web-workflows` extraction.
3. Revert W2 root Workflows route adapters.
4. Revert W1 cross-feature import prep only if it was purely migration-driven and not a standalone improvement.
5. Remove `crates/tessara-web-workflows` from workspace members.
6. Remove root `tessara-web` dependency and feature forwarding for `tessara-web-workflows`.
7. Rerun validation.

If PARTIAL:

- keep only proven pieces;
- document retained debt;
- pause further extraction pending re-profile.

If GO:

- keep `tessara-web-workflows`;
- write `docs/architecture/tessara-web-workflows-extraction-results.md`;
- re-profile before choosing Administration or Responses next.

---

## 16. Results Document Template

Create:

```text
docs/architecture/tessara-web-workflows-extraction-results.md
```

Required sections:

```text
1. measured environment
2. baseline Workflows import inventory
3. cross-feature dependency resolution summary
4. route-adapter split summary
5. extraction implementation summary
6. timing matrix
7. cargo-leptos watch results
8. feature-tree deltas
9. bundle-size deltas
10. public API review
11. dependency-boundary report
12. behavior validation
13. intentional debt retained
14. GO/PARTIAL/NO-GO decision
15. next recommendation
```

Intentional debt section must include:

```text
workflow-local browser transport copied
workflow-local form option DTOs copied
workflow-local organization/node option DTOs copied
workflow-local response/workflow link helpers copied
workflow-local tiny text/pagination/url helpers copied
API/web DTO duplication retained
tessara-web-platform still deferred
tessara-workflows domain crate not created, unless inventory changed that decision
```

---

## 17. Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Workflows depends on Forms web DTOs | Forbidden sibling dependency | Replace with workflow-local form option DTOs |
| Workflows depends on Organization web DTOs | Forbidden sibling dependency | Replace with workflow-local node/node-type DTOs or pure hierarchy contract after review |
| Assignments depend on Responses behavior | Ownership leak | Use links/local DTOs; keep response-start actions in Responses |
| Operations/Home import workflow internals | Root compatibility leak | Inventory and resolve before removing root feature module |
| Query-string behavior requires router hooks | Could force `leptos_router` | Prefer local browser search-param helper; root owns path params |
| Workflow editor has deep Leptos view recursion | Test compile failure | Add `#![recursion_limit = "512"]` if needed, as Forms did |
| Browser transport duplicated across three features | Maintenance debt | Record debt; consider platform only after GO and separate decision |
| Public API inflation | Long-term maintenance cost | Facade-only exports; `cargo doc` review |
| Full app build regresses | Focused loop improves but integration cost high | Enforce root regression threshold |
| CSS classes change | UI regression | DOM/class preservation checks |
| cargo-leptos watch misses path crate | Developer loop not improved | Treat as NO-GO/PARTIAL |

---

## 18. Relationship to Long-Term Roadmap

A GO for Workflows authorizes only:

```text
keeping `tessara-web-workflows`
writing a fresh evidence-based proposal for the next candidate
```

It does not automatically authorize:

```text
tessara-web-administration
tessara-web-responses
tessara-web-organization
tessara-web-operations
tessara-web-platform
tessara-workflows domain crate
```

After Workflows, re-rank Administration vs Responses using fresh measurements and actual cross-feature dependencies.

---

## 19. Summary

This proposal extends the successful Datasets and Forms feature-crate pattern to Workflows.

The extraction is justified because Workflows is large, high-churn, and likely to provide significant focused-loop value. It is also riskier than Forms because it touches Forms, Organization, Responses, assignments, query-string state, and root aggregators.

The extraction should therefore proceed only after a Workflows-specific inventory proves the boundary can hold.

The intended final shape:

```text
root tessara-web:
  routes/workflows.rs
  WorkflowRouteParams
  AppShell/auth/session/navigation
  hydration/document/CSS/assets

tessara-web-workflows:
  WorkflowsIndexContent
  WorkflowNewContent
  WorkflowAssignmentsContent
  WorkflowDetailContent
  WorkflowEditContent
  crate-private internals
```

If the boundary checker passes, focused Workflows checks work, cargo-leptos watch sees Workflows edits, and root regressions remain acceptable, the Workflows extraction should be kept as the next step in Tessara's measured feature-crate migration.

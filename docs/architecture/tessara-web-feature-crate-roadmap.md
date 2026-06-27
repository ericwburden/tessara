# Tessara Web Feature-Crate Migration Plan and Long-Term Roadmap

**Status:** Proposed next architecture plan after the successful `tessara-web-datasets` pilot  
**Primary near-term target:** `tessara-web-forms` extraction proposal and implementation  
**Long-term direction:** product-area frontend crates under a modular monolith  
**Non-goal:** microservices as a frontend compile-time solution

---

## 1. Purpose

The dataset pilot showed that extracting a large, coherent frontend feature area into its own Rust crate can improve focused development and validation loops without unacceptable full-app regressions. This plan turns that pilot result into a controlled, evidence-gated migration path for the rest of `tessara-web`.

This plan is not a mandate to split everything immediately. It defines:

1. the accepted architecture after the dataset pilot;
2. permanent crate-boundary rules;
3. the next concrete extraction plan for `tessara-web-forms`;
4. shared infrastructure and contract policies;
5. measurement and validation standards;
6. a long-term roadmap for the remaining feature-area crates.

Later stages are intentionally less detailed. They should be progressively elaborated after each prior extraction produces fresh measurements and dependency evidence.

---

## 2. Dataset Pilot Findings

The dataset pilot should be treated as **GO**.

The important interpretation is narrow:

> Feature-area crates can improve focused development loops while keeping full-app regressions acceptable.

The pilot did **not** prove that every full application command becomes faster. Full app checks remained slightly slower but within the accepted thresholds.

### 2.1 Full-app timing comparison

| Probe | Baseline | Current | Delta | Result |
| --- | ---: | ---: | ---: | --- |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 2m08s | 2m18s | +10s / +7.7% | pass |
| `cargo check -p tessara-api --features ssr` | 3m05s | 3m06s | +1s / +0.5% | pass |
| `cargo leptos build` | 6m30s | 6m51s | +21s / +5.4% | pass |
| root web test compile | exceeded 600s | replaced by dataset-local compile gate at 6m43s | feature-local gate completes | pass |

### 2.2 Focused-loop result

The new dataset-local gate completed:

```powershell
cargo test -p tessara-web-datasets --no-default-features --features ssr --no-run -j 1
```

in **6m43s**, while the prior root web test compile exceeded the 600-second timeout.

This is the clearest practical win: focused test/check commands are now possible for dataset work without going through the full frontend crate.

### 2.3 cargo-leptos watch result

The final watch/page-availability gate passed after DB reset:

- `cargo leptos watch` detected `tessara-web-datasets` path dependency edits.
- `tessara-web-datasets` and `tessara-web` rebuilt.
- wasm-bindgen completed.
- `/health` returned healthy.
- authenticated `/datasets` returned HTTP 200 after edits.

Steady-state dataset edit rebuild timing, excluding the first cold-ish post-reset edit:

| Metric | Median | Range |
| --- | ---: | ---: |
| cargo compile only | 26.67s | 26.49s to 26.84s |
| cargo compile + wasm-bindgen | 32.71s | 32.40s to 33.52s |

### 2.4 Boundary result

The boundary checker passed and now enforces the most important crate dependency rules:

- `tessara-web-datasets` must not depend on root `tessara-web`, `tessara-api`, or sibling web feature crates.
- `tessara-web-ui` must not depend on root, API, or feature crates.
- domain crates must not depend on web/server transport or UI frameworks.

### 2.5 Public API result

The public API stayed narrow:

- `tessara-web-ui` exposes a limited generic UI facade.
- `tessara-web-datasets` exposes only four content components.
- dataset DTOs, loaders, editor state, transport, validation, and helper modules remain crate-private.

### 2.6 Bundle-size result

The pilot grew combined JS/WASM by about **125 KB** on a roughly **27.7 MB** bundle; CSS size was unchanged. This is acceptable but should continue to be tracked.

---

## 3. Accepted Architecture After the Pilot

The architecture is now:

```text
crates/
├── tessara-web              # root app, route registry, shell, hydration, document, CSS/assets
├── tessara-web-ui           # generic Leptos UI primitives
├── tessara-web-datasets     # extracted dataset feature crate
├── tessara-api              # single deployable API/SSR service
├── tessara-core             # shared domain primitives
├── tessara-datasets         # dataset domain rules
├── tessara-forms            # form domain rules
├── tessara-hierarchy        # hierarchy/domain rules
├── tessara-submissions      # submission/domain rules
└── other domain/stub crates
```

### 3.1 Root `tessara-web` owns

```text
- cargo-leptos frontend app role
- `cdylib` crate type
- hydration entry point
- document rendering
- route registry
- route parameters
- AppShell
- auth guard integration
- session/navigation/logout/capability policy
- CSS and public assets
- root route adapters
- root integration tests
- temporary root `ui` compatibility facade
```

### 3.2 `tessara-web-ui` owns

```text
- generic Leptos UI primitives
- generic table, pagination, combobox, breadcrumb, empty state, and page header components
- generic domain-neutral interactive widgets moved only after review
```

It must not own:

```text
- AppShell
- auth/session/navigation policy
- route registry or route params
- browser transport
- feature-specific display helpers
- product concepts such as datasets, forms, workflows, responses, organization, administration
```

### 3.3 Feature crates own

Each `tessara-web-*` feature crate should own:

```text
- content components for root route adapters
- feature-specific components
- feature API transport
- feature actions/loaders
- feature web DTOs and view models
- feature display helpers
- feature validation
- feature-local tests
```

Feature crates should expose only a small public facade to the root route adapter.

---

## 4. Permanent Boundary Rules

These rules apply to the current dataset crate and all future feature crates.

### 4.1 Feature web crates must not depend on

```text
- root `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates
- root route modules
- root AppShell/session/navigation/auth modules
- `leptos_router`, unless a specific feature proves root adapters cannot cover the need
- `leptos_meta`, unless a specific feature proves it owns metadata composition
```

### 4.2 Feature web crates may depend on

```text
- `tessara-web-ui`
- `leptos`
- `icons`
- `serde` / `serde_json`
- hydrate-gated `gloo-net` / `web-sys` / `wasm-bindgen` where needed
- domain crates for stable pure rules or stable contracts
```

### 4.3 Domain and contract crates must not depend on

```text
- Leptos
- Axum
- SQLx
- gloo-net
- web-sys
- js-sys
- wasm-bindgen
- frontend feature crates
```

### 4.4 Boundary checker

The existing boundary checker should become permanent:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

It should run:

- locally during every extraction;
- in CI once CI coverage for these branches is established;
- before accepting any new `tessara-web-*` feature crate.

Any allowlist must be temporary, documented, and treated as architectural debt.

---

## 5. Current Intentional Debt

The dataset pilot intentionally retained some debt to keep the experiment focused.

### 5.1 Dataset-local browser transport

`tessara-web-datasets` has private copied browser HTTP transport. This avoids a root app dependency.

Do **not** create `tessara-web-platform` yet. Reassess after one or two more feature extractions.

### 5.2 Dataset-local helpers

Minimal text and pagination helpers are local to `tessara-web-datasets`.

If forms and workflows need the same helpers, consider promotion to a small shared crate or `tessara-web-ui` private implementation where appropriate. Do not prematurely centralize.

### 5.3 API/web DTO duplication

Dataset API DTOs and dataset web DTOs remain intentionally separate.

This is correct pilot debt. Contract convergence should be considered only after crate extraction is proven useful and the representation question is understood.

### 5.4 Browser-heavy UI primitive

`DraggablePanelList` moved to `tessara-web-ui` with browser drag dependencies.

This is acceptable because the primitive is domain-neutral, but it is now part of the high-fan-out UI crate and should be monitored. Avoid moving additional browser-heavy UI primitives without measuring shared-UI edit cost.

---

## 6. Candidate Feature-Crate Ranking

This ranking combines expected compile-time payoff, source weight, churn, coupling, and extraction risk.

| Rank | Candidate crate | Payoff | Risk | Notes |
| ---: | --- | --- | --- | --- |
| 1 | `tessara-web-forms` | Very high | Medium | Large, high-churn, foundational for workflows/responses |
| 2 | `tessara-web-workflows` | Very high | Medium-high | Large editor/assignments/detail/list surface; depends on form contracts |
| 3 | `tessara-web-administration` | High | Medium | Substantial, relatively isolated after auth/hierarchy contracts are clear |
| 4 | `tessara-web-responses` | Medium-high | Medium | Leaf-like but depends conceptually on forms and workflow assignments |
| 5 | `tessara-web-organization` | Medium-high | High | High-fan-out hierarchy/organization concepts; extract later |
| 6 | `tessara-web-operations` | Low | Low | Aggregator surface; keep root until it grows |
| defer | auth/login/home/placeholders | Low | Varies | Keep root unless evidence changes |

The ranking is not an automatic sequence. Re-rank after each extraction using fresh measurements and the current dependency graph.

---

## 7. Near-Term Plan: `tessara-web-forms`

Forms should be the next concrete extraction proposal.

### 7.1 Why forms next

Forms is the best next target because it is:

- large and Leptos-heavy;
- actively edited;
- foundational to workflows and responses;
- already internally modular;
- a natural place to reduce future workflow coupling.

Extracting workflows before forms risks creating either:

- a workflow crate that depends on forms web internals; or
- premature contract extraction just to break that dependency.

### 7.2 Near-term goal

Extract the current forms frontend implementation into:

```text
crates/tessara-web-forms
```

using the dataset pattern:

```text
root route adapters + small forms content facade + crate-private internals
```

### 7.3 Non-goals for forms extraction

Do **not** include these in the first forms extraction:

```text
- API/web form DTO convergence
- canonical ID/timestamp representation
- moving CSS/assets
- moving AppShell
- moving route params
- creating `tessara-web-platform`
- extracting workflows or responses
- creating placeholder crates
```

### 7.4 Proposed public facade

Initial facade proposal:

```rust
pub use facade::{
    FormDetailContent,
    FormEditorContent,
    FormNewContent,
    FormsIndexContent,
};
```

Potential signatures:

```rust
#[component]
pub fn FormsIndexContent() -> impl IntoView;

#[component]
pub fn FormNewContent() -> impl IntoView;

#[component]
pub fn FormDetailContent(form_id: String) -> impl IntoView;

#[component]
pub fn FormEditorContent(form_id: String) -> impl IntoView;
```

Alternative editor facade:

```rust
#[component]
pub fn FormEditorContent(form_id: Option<String>) -> impl IntoView;
```

Use the `Option<String>` form only if create and edit share enough internals to justify one facade. Otherwise keep `FormNewContent` and `FormEditorContent` separate.

### 7.5 Root ownership

Root `tessara-web` keeps:

```text
routes/forms.rs
FormRouteParams
AppShell wrapping
route registry
auth/session/navigation behavior
hydration/document/CSS/assets
```

Forms content components must not:

```text
- parse route params
- render AppShell
- import root route modules
- import root session/navigation/auth modules
```

### 7.6 Forms crate ownership

`tessara-web-forms` owns:

```text
forms API transport
forms loaders/actions
forms list/detail/create/edit content
form builder UI
form builder state
form builder layout/drag/resize/sizing/hydrate logic
forms display/filtering/version helpers
forms web DTOs, unchanged initially
forms tests
```

Before extraction, current forms imports from sibling root feature modules must be resolved. In particular, forms currently uses organization node-type DTOs and workflow display helpers. The extraction must choose one of:

```text
- copy a narrow forms-local web DTO/helper
- promote a pure, product-neutral helper to an allowed shared/domain crate
- keep genuinely cross-feature presentation in the root adapter
```

Do not keep those references by adding forbidden `tessara-web-organization` or `tessara-web-workflows` dependencies.

### 7.7 Expected dependencies

Allowed:

```text
tessara-web-ui
leptos
icons
serde
serde_json
hydrate-gated gloo-net/web-sys/js-sys/wasm-bindgen as needed
possibly tessara-forms for pure validation/rules
```

Forbidden:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-workflows
tessara-web-organization
leptos_router unless explicitly justified
leptos_meta
root auth/session/navigation/shell modules
```

### 7.8 Forms-specific inventory before implementation

Before writing code, inventory:

```text
all `crate::ui` imports under forms
all `crate::utils` imports under forms
all `crate::http` imports under forms
all `crate::types` imports under forms
all `crate::features::*` imports under forms
all `cfg(feature = "hydrate")` usage
all `gloo-net`, `web-sys`, `wasm-bindgen`, `js-sys` usage
all form builder tests and pure tests
all CSS classes/assets referenced by forms
all public-ish types currently used by workflows/responses/organization
all direct forms imports from organization/workflows/responses/datasets/administration
```

### 7.9 Forms extraction commit plan

Use the same pattern as the dataset pilot, but lighter unless unexpected complexity appears.

#### Commit F0 — forms inventory and baseline

No architecture change.

Record:

```powershell
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
cargo test -p tessara-web --lib --no-run -j 1
```

Add a representative forms edit benchmark.

#### Commit F1 — forms route-adapter preparation

Refactor forms pages so root route adapters own:

```text
FormRouteParams
AppShell wrapping
route titles
route-to-content wiring
```

Current in-crate forms code should expose shell-free content components before moving crates.

Resolve existing sibling-feature imports in this commit or explicitly carry them as F2 blockers. Known areas to inventory and eliminate are organization node-type DTO usage in form create/edit options and workflow display helper usage on form detail related-workflow tables.

#### Commit F2 — extract `tessara-web-forms`

Create the crate and move forms internals.

Root routes import only the facade.

#### Commit F3 — boundary, feature, test, watch, bundle checks

Add/extend boundary checker for the new forms crate.

Run and record:

```powershell
cargo check -p tessara-web-forms --no-default-features --features ssr
cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
cargo leptos watch forms edit gate
cargo doc -p tessara-web-forms --no-deps
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

### 7.10 Forms GO/PARTIAL/NO-GO

Use lighter thresholds than the original pilot, but keep the same spirit.

GO if:

```text
- forms-local hydrate/SSR checks pass
- forms-local test compile completes
- root full-app regressions stay under 10%
- cargo-leptos watch detects forms crate edits
- boundary checker passes with no permanent exceptions
- public API is only the approved facade
- root retains route/shell ownership
- no CSS/assets migration was required
```

PARTIAL if:

```text
- forms extraction works but focused-loop improvement is modest
- root regressions are acceptable
- maintainability improves enough to keep the crate
- but more feature extraction should pause pending reassessment
```

NO-GO if:

```text
- crate requires forbidden sibling feature dependencies
- public API grows beyond facade to make migration compile
- cargo-leptos watch does not detect forms path-dependency edits
- full-app regressions exceed thresholds
- DTO convergence becomes required to compile
```

---

## 8. Per-Feature Extraction Template

Every future feature extraction should use this template.

### 8.1 Proposal

Before code movement, write:

```text
docs/architecture/tessara-web-<feature>-extraction-proposal.md
```

Include:

```text
- why this feature is next
- current source size/churn
- import inventory
- public facade
- root ownership table
- feature crate ownership table
- dependency expectations
- DTO policy
- test movement plan
- watch gate
- bundle-size check
- rollback plan
```

### 8.2 Public facade pattern

Feature crates expose content components, not route pages.

Example:

```rust
pub use facade::{
    FeatureIndexContent,
    FeatureDetailContent,
    FeatureEditorContent,
};
```

Root owns route adapters:

```rust
#[component]
fn FeatureDetailRoute() -> impl IntoView {
    let id = require_route_params::<FeatureRouteParams>().id;

    view! {
        <AppShell active_route="feature" title="Feature Detail">
            <feature_crate::FeatureDetailContent id/>
        </AppShell>
    }
}
```

### 8.3 Default commit sequence

```text
0. inventory and measurement
1. root route-adapter preparation
2. feature crate extraction
3. boundary/test/watch/bundle documentation
```

Add a support-crate prep commit only when the feature needs new generic UI.

### 8.4 Default non-goals

For every feature extraction, default non-goals are:

```text
- API/web DTO convergence
- AppShell extraction
- route-system rewrite
- CSS/assets migration
- microservices
- unrelated domain crate redesign
- sibling web crate dependencies
```

### 8.5 Default retained-debt section

Every result document must include:

```text
- duplicated DTOs retained
- copied local helpers retained
- copied browser transport retained
- root-owned concepts intentionally not moved
- follow-up contract decisions required
```

---

## 9. Measurement and Validation Policy

### 9.1 Required commands for every extracted feature

Replace `<feature>` with the new crate.

Timing measurements used for GO/PARTIAL/NO-GO decisions must follow the pilot protocol:

```text
- record `git rev-parse HEAD` and `git status --short`
- record `rustc -Vv`, `cargo -V`, `cargo leptos --version`, OS, CPU, RAM, and power plan
- use isolated pilot-owned `CARGO_TARGET_DIR` directories for baseline and current clean-target probes
- remove `RUSTC_WRAPPER`, set `SCCACHE_DISABLE=1`, and keep the Cargo incremental policy explicit
- remove `target/site` before `cargo leptos build` when comparing clean full-app build time
- write raw command logs under ignored `tmp` evidence directories
- keep a fixed timeout for inconclusive long-running probes and report timeout as inconclusive, not as a win
```

Run and record:

```powershell
cargo check -p tessara-web-<feature> --no-default-features --features ssr
cargo check -p tessara-web-<feature> --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo test -p tessara-web-<feature> --no-default-features --features ssr --no-run -j 1
cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
cargo check -p tessara-api --features ssr
cargo leptos build
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1
```

### 9.2 Watch gate

For each feature:

```text
- start `cargo leptos watch`
- make a compile-affecting but behavior-neutral edit in the feature crate
- confirm path dependency detection
- confirm front bundle update
- confirm route/page availability
```

### 9.3 Bundle gate

Track:

```text
- .wasm size
- .js size
- combined JS/WASM size
- CSS size
```

A bundle increase above 5% requires explanation.

### 9.4 Test gate

Move feature-local tests into the feature crate.

Keep root route/integration tests in root.

Do not make internal symbols public solely for tests.

---

## 10. Contract Migration Policy

### 10.1 Default rule

For first extraction of a feature:

```text
move web DTOs unchanged
keep API DTOs unchanged
document duplication
measure
decide later
```

### 10.2 When to promote shared contracts

Promote a shared contract only when all are true:

```text
- API and web both use the shape
- ownership is clear
- representation is stable
- adapter duplication is causing real maintenance cost
- WASM dependency cost is measured and acceptable
- the contract crate remains Leptos/Axum/SQLx/browser-free
```

### 10.3 Representation caution

Do not introduce `uuid`, `chrono`, or other heavier shared dependencies into WASM builds just because server DTOs use them.

Choose one of:

```text
- separate API/web DTOs
- string-oriented wire contracts
- typed shared contracts with measured dependency cost
```

Make this decision per domain, not globally.

---

## 11. Shared Infrastructure Promotion Policy

Do not create `tessara-web-platform` yet.

Promotion rule:

```text
Promote a helper only after at least two extracted feature crates independently need the same helper and local duplication is becoming a maintenance risk.
```

Candidate future shared infrastructure:

```text
browser HTTP transport
unauthorized handling abstraction
tiny text helpers
pagination helpers
query-param helpers
local storage helpers
```

Requirements for promotion:

```text
- policy-neutral API
- no root AppShell/session/navigation dependency
- no product concepts
- no hidden route/auth behavior
- boundary checker coverage
- measured shared-edit cost
```

---

## 12. Rollback and Stop Policy

### 12.1 Rollback

Each extraction must be commit-based and revertible in reverse order.

Default rollback:

```text
1. remove feature crate
2. move code back under root feature module
3. restore root route pages
4. remove workspace member
5. remove feature forwarding
6. rerun validation
```

### 12.2 Stop rules

Stop the extraction program when:

```text
- full-app regressions exceed thresholds
- cargo-leptos watch becomes unreliable
- boundary checker needs permanent root/API/sibling exceptions
- public APIs expand beyond content facades
- feature extraction requires DTO convergence to compile
- support crates become high-churn bottlenecks
- feature is small/low-churn and does not justify Cargo complexity
```

### 12.3 GO does not authorize the next feature automatically

A successful feature extraction authorizes only:

```text
- keeping that extraction
- writing the next evidence-based extraction proposal
```

It does not authorize extracting the rest of the roadmap.

---

## 13. Long-Term Roadmap

This roadmap is directional. Later stages should be progressively elaborated after fresh measurements and dependency inventories.

### Stage A — Foundation and dataset pilot consolidation

**Status:** complete / active.

Completed:

```text
- extracted `tessara-web-ui`
- extracted `tessara-web-datasets`
- installed boundary checker
- validated cargo-leptos path dependency watch
- retained root AppShell/routes/hydration/CSS/assets
- kept dataset DTO convergence out of scope
```

Follow-up:

```text
- commit and preserve dataset pilot results
- document accepted retained debt
- ensure boundary checker remains part of normal validation
```

### Stage B — Forms extraction

**Status:** next concrete target.

Goal:

```text
extract form authoring, builder, list/detail/edit surfaces into `tessara-web-forms`
```

Preconditions:

```text
- forms import inventory complete
- root route-adapter facade agreed
- form builder browser dependencies understood
- `tessara-web-ui` public API additions, if any, reviewed
```

Expected result:

```text
- form-focused edit/test/check loop
- root app keeps routes and shell
- workflows still do not depend on forms web crate unless explicitly rejected by boundary checker
```

### Stage C — Workflows extraction

**Status:** future, after forms.

Goal:

```text
extract workflow list/detail/editor/assignments into `tessara-web-workflows`
```

Preconditions:

```text
- form-facing contracts stable enough for workflow options
- workflow dependencies on organization/hierarchy inventoried
- decision made whether `tessara-workflows` domain/contract crate is warranted
```

Possible facade:

```rust
pub fn WorkflowsIndexContent() -> impl IntoView;
pub fn WorkflowDetailContent(workflow_id: String) -> impl IntoView;
pub fn WorkflowEditorContent(workflow_id: Option<String>) -> impl IntoView;
pub fn WorkflowAssignmentsContent() -> impl IntoView;
```

Do not create `tessara-workflows` just for symmetry. Create it only if stable workflow rules/contracts are used by both API and web.

### Stage D — Administration and responses

**Status:** future, order to be re-evaluated.

Candidate 1:

```text
tessara-web-administration
```

Rationale:

```text
substantial user/role/node-type management surface
potentially strong focused-loop payoff
relatively self-contained after auth/hierarchy contracts are clear
```

Candidate 2:

```text
tessara-web-responses
```

Rationale:

```text
leaf-ish response list/start/detail/edit feature
safer after forms and workflows are extracted
```

Possible administration facade:

```rust
pub fn AdministrationIndexContent() -> impl IntoView;
pub fn AdministrationUsersContent() -> impl IntoView;
pub fn AdministrationUserDetailContent(account_id: String) -> impl IntoView;
pub fn AdministrationUserEditContent(account_id: String) -> impl IntoView;
pub fn AdministrationUserAccessContent(account_id: String) -> impl IntoView;
pub fn AdministrationRolesContent() -> impl IntoView;
pub fn AdministrationNodeTypesContent() -> impl IntoView;
```

Possible responses facade:

```rust
pub fn ResponsesIndexContent() -> impl IntoView;
pub fn ResponseStartContent() -> impl IntoView;
pub fn ResponseDetailContent(submission_id: String) -> impl IntoView;
pub fn ResponseEditContent(submission_id: String) -> impl IntoView;
```

Re-rank after workflow extraction.

### Stage E — Organization extraction

**Status:** future, likely late.

Goal:

```text
extract organization explorer, detail, related work, node editor, metadata, and tree UI into `tessara-web-organization`
```

Why late:

```text
organization/hierarchy concepts are high fan-out
datasets/forms/workflows/admin may all touch node concepts
extracting organization too early risks forbidden sibling web dependencies
```

Preconditions:

```text
- stable hierarchy contracts in `tessara-hierarchy` where useful
- other feature crates no longer depend on organization web types
- node editor payload ownership resolved
```

Possible facade:

```rust
pub fn OrganizationIndexContent() -> impl IntoView;
pub fn OrganizationDetailContent(node_id: String) -> impl IntoView;
pub fn OrganizationNodeCreateContent(parent_node_id: Option<String>, node_type_id: Option<String>) -> impl IntoView;
pub fn OrganizationNodeEditContent(node_id: String) -> impl IntoView;
```

### Stage F — Reassess small/root-owned surfaces

Keep in root by default:

```text
home
login
auth integration
operations
components placeholder
dashboards placeholder
route registry
AppShell
session/navigation/logout policy
hydration/document/CSS/assets
```

Extract only if:

```text
- source size grows substantially
- feature becomes high-churn
- focused local validation becomes valuable
- dependency boundaries are clean
- extraction does not introduce high fan-out
```

Operations is an aggregator surface and may never need a crate.

### Stage G — Optional shared platform

Only after at least two extracted feature crates duplicate the same transport/helpers:

```text
consider `tessara-web-platform` or `tessara-web-utils`
```

This should be a small, policy-neutral crate.

Do not include:

```text
AppShell
auth/session/navigation
route registry
product concepts
hard-coded `/login` redirect policy
```

### Stage H — Optional contract/domain evolution

After multiple feature crates are stable, evaluate contract convergence.

Candidate areas:

```text
forms contracts
workflow contracts
submission/response contracts
hierarchy contracts
dataset contracts
```

Do not converge contracts automatically. Use measured adapter pain and dependency cost as the trigger.

### Stage I — Future service extraction

This remains independent of frontend compile-time work.

Potential future services only if runtime triggers appear:

| Candidate | Trigger |
| --- | --- |
| analytics/materialization worker | long-running refresh, retry isolation, scheduling |
| dataset materialization worker | durable query/materialization lifecycle |
| workflow runtime service | runtime scaling/failure isolation |
| identity service | external identity/compliance/security boundary |

Do not use microservices to solve frontend compile time.

---

## 14. Next Artifact

This roadmap is the durable architecture artifact after the dataset pilot.

The next implementation-specific document is:

```text
docs/architecture/tessara-web-forms-extraction-proposal.md
```

The forms proposal should be implementation-ready and should include inventory, facade, route adapters, dependency rules, cross-feature dependency resolution, validation commands, measurement gates, and rollback steps.

---

## 15. Summary

The dataset pilot validated the core thesis:

> Tessara can use feature-area crates to regain focused development loops while retaining a single routed Leptos app and one primary API service.

The next move is not a mass split. It is a measured extraction of `tessara-web-forms`, followed by re-profiling and re-ranking.

The long-term roadmap points toward feature-area crates for the major product surfaces, but each stage remains conditional on evidence.

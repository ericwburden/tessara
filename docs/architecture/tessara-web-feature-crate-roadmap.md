# Tessara Web Feature-Crate Migration Plan and Long-Term Roadmap

**Status:** Active architecture roadmap after successful `tessara-web-datasets`, `tessara-web-forms`, and `tessara-web-workflows` extractions
**Primary near-term target:** `tessara-web-responses` extraction proposal
**Long-term direction:** product-area frontend crates under a modular monolith
**Non-goal:** microservices as a frontend compile-time solution

---

## 1. Purpose

The dataset pilot showed that extracting a large, coherent frontend feature area into its own Rust crate can improve focused development and validation loops without unacceptable full-app regressions. This plan turns that pilot result into a controlled, evidence-gated migration path for the rest of `tessara-web`.

This plan is not a mandate to split everything immediately. It defines:

1. the accepted architecture after the dataset pilot;
2. permanent crate-boundary rules;
3. the completed `tessara-web-forms` and `tessara-web-workflows` results;
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

## 3. Forms Extraction Findings

The Forms extraction should also be treated as **GO**.

Governing evidence:

- `docs/architecture/tessara-web-forms-extraction-proposal.md`
- `docs/architecture/tessara-web-forms-extraction-results.md`

### 3.1 Architecture result

The extraction created:

```text
crates/tessara-web-forms
```

and preserved the accepted dataset pattern:

- root `tessara-web` owns Forms route adapters, route params, `AppShell`, auth/session/navigation, hydration, document, CSS/assets, and cargo-leptos app ownership;
- `tessara-web-forms` owns Forms content, builder UI/state/layout/drag/resize/hydrate logic, loaders, transport, save orchestration, DTOs, display helpers, and feature-local tests;
- root routes import only the approved content facade:

```rust
FormsIndexContent
FormNewContent
FormDetailContent
FormEditContent
```

The root `features::forms` module was removed rather than preserved as a compatibility layer.

### 3.2 Compile result

Follow-up clean-target comparison after the GO call:

| Probe | Original Forms baseline | Current clean target | Delta |
| --- | ---: | ---: | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 270.47s | -35.45s / -11.6% |
| `cargo check -p tessara-api --features ssr` | 392.17s | 392.15s | flat |
| `cargo leptos build` | `>900s` timeout | 799.46s | completed, at least 100.54s under timeout |
| `cargo check -p tessara-web-forms --no-default-features --features ssr` | n/a | 114.87s | focused loop available |
| `cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | 105.48s | focused loop available |
| `cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1` | n/a | 336.19s | focused test compile available |

The practical result is a successful extraction with no full-app compile regression and a new Forms-focused check loop.

### 3.3 Watch and route result

The Forms watch gate passed:

- `cargo leptos watch` detected edits in `crates\tessara-web-forms`;
- `tessara-web-forms` and root `tessara-web` rebuilt through the path dependency;
- `/health` returned 200 after rebuilds;
- authenticated `/forms`, `/forms/new`, `/forms/:form_id`, and `/forms/:form_id/edit` returned 200 after demo seed;
- unauthenticated protected Forms routes preserved login redirect behavior.

### 3.4 Boundary and bundle result

The boundary checker now covers `tessara-web-forms` and rejects root/API/sibling web feature dependencies plus router/meta dependencies.

`tessara-web-forms` has no dependency on:

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

Bundle growth remained below the 5% threshold.

### 3.5 Retained debt and caveats

Retained intentional debt:

- Forms keeps web DTOs separate from API DTOs.
- Forms owns local transport and small local support helpers.
- Shared workflow/form/response contracts were not generalized during extraction.
- Root web lib test still fails at Windows link time with unresolved Leptos/Tachys symbols, but forms-local tests, root hydrate, API SSR, `cargo leptos build`, and browser-facing route smoke all pass.

The Forms result remains a reference model for future extractions.

---

## 4. Workflows Extraction Findings

The Workflows extraction should also be treated as **GO**.

Governing evidence:

- `docs/architecture/tessara-web-workflows-extraction-proposal.md`
- `docs/architecture/tessara-web-workflows-extraction-results.md`

### 4.1 Architecture result

The extraction created:

```text
crates/tessara-web-workflows
```

and preserved the accepted feature-crate pattern:

- root `tessara-web` owns Workflows route adapters, `WorkflowRouteParams`, auth guard, `AppShell`, hydration, document, CSS/assets, and cargo-leptos app ownership;
- `tessara-web-workflows` owns workflow list/detail/new/edit content, assignments UI, loaders, transport, DTOs, payloads, display helpers, and feature-local support helpers;
- root routes import only the approved content facade:

```rust
WorkflowsIndexContent
WorkflowDetailContent
WorkflowNewContent
WorkflowEditContent
WorkflowAssignmentsContent
```

The root `features::workflows` module was removed rather than preserved as a compatibility layer.

### 4.2 Compile result

Latest clean-target comparison against the original pre-Forms extraction baseline:

| Probe | Original clean baseline | Post-Forms clean target | Current Workflows clean target | Delta vs original |
| --- | ---: | ---: | ---: | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 270.47s | 247.39s | -58.53s / -19.1% |
| `cargo check -p tessara-api --features ssr` | 392.17s | 392.15s | 337.40s | -54.77s / -14.0% |
| `cargo leptos build` | `>900s` timeout | 799.46s | 410.31s | completed, at least 489.69s under timeout |
| `cargo check -p tessara-web-workflows --no-default-features --features ssr` | n/a | n/a | 204.98s | focused loop available |
| `cargo check -p tessara-web-workflows --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | n/a | 216.68s | focused loop available |
| `cargo test -p tessara-web-workflows --no-default-features --features ssr --no-run -j 1` | n/a | n/a | 323.48s | focused test compile available |

The practical result is a successful extraction with no full-app compile regression, materially faster full-app clean checks than the original baseline, and a new Workflows-focused check loop.

### 4.3 Watch and route result

The Workflows watch gate passed:

- `cargo leptos watch` detected edits in `crates\tessara-web-workflows`;
- `tessara-web-workflows` and root `tessara-web` rebuilt through the path dependency;
- wasm-bindgen completed and `Watch updated Front` was reported;
- `/health` returned 200 under watch;
- authenticated `/workflows`, `/workflows/new`, `/workflows/assignments`, `/workflows/:workflow_id`, and `/workflows/:workflow_id/edit` returned 200 after demo seed.

### 4.4 Boundary and bundle result

The boundary checker now covers `tessara-web-workflows` and rejects root/API/sibling web feature dependencies plus router/meta dependencies.

`tessara-web-workflows` has no dependency on:

```text
tessara-web
tessara-api
tessara-web-datasets
tessara-web-forms
tessara-web-responses
tessara-web-organization
tessara-web-administration
leptos_router
leptos_meta
```

Immediate bundle growth from the Forms post-extraction bundle remained below the 5% threshold. Cumulative growth since the dataset pilot is now above 5%, so bundle size should be tracked closely before another large Leptos extraction.

### 4.5 Retained debt and caveats

Retained intentional debt:

- Workflows keeps web DTOs separate from API DTOs.
- Workflows owns local transport, unauthorized handling, and small local support helpers.
- Organization node and node type option DTOs are duplicated locally.
- Shared transport/helper contracts were not generalized during extraction.
- Cumulative JS/WASM growth should remain a gate for the next large extraction.

## 5. Accepted Architecture After Dataset, Forms, and Workflows

The architecture is now:

```text
crates/
├── tessara-web              # root app, route registry, shell, hydration, document, CSS/assets
├── tessara-web-ui           # generic Leptos UI primitives
├── tessara-web-datasets     # extracted dataset feature crate
├── tessara-web-forms        # extracted forms feature crate
├── tessara-web-workflows    # extracted workflows feature crate
├── tessara-api              # single deployable API/SSR service
├── tessara-core             # shared domain primitives
├── tessara-datasets         # dataset domain rules
├── tessara-forms            # form domain rules
├── tessara-hierarchy        # hierarchy/domain rules
├── tessara-submissions      # submission/domain rules
└── other domain/stub crates
```

### 5.1 Root `tessara-web` owns

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

### 5.2 `tessara-web-ui` owns

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

### 5.3 Feature crates own

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

## 6. Permanent Boundary Rules

These rules apply to the current dataset crate and all future feature crates.

### 6.1 Feature web crates must not depend on

```text
- root `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates
- root route modules
- root AppShell/session/navigation/auth modules
- `leptos_router`, unless a specific feature proves root adapters cannot cover the need
- `leptos_meta`, unless a specific feature proves it owns metadata composition
```

### 6.2 Feature web crates may depend on

```text
- `tessara-web-ui`
- `leptos`
- `icons`
- `serde` / `serde_json`
- hydrate-gated `gloo-net` / `web-sys` / `wasm-bindgen` where needed
- domain crates for stable pure rules or stable contracts
```

### 6.3 Domain and contract crates must not depend on

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

### 6.4 Boundary checker

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

## 7. Current Intentional Debt

The dataset pilot intentionally retained some debt to keep the experiment focused.

### 7.1 Dataset-local browser transport

`tessara-web-datasets` has private copied browser HTTP transport. This avoids a root app dependency.

Do **not** create `tessara-web-platform` yet. Reassess after one or two more feature extractions.

### 7.2 Dataset-local helpers

Minimal text and pagination helpers are local to `tessara-web-datasets`.

If forms and workflows need the same helpers, consider promotion to a small shared crate or `tessara-web-ui` private implementation where appropriate. Do not prematurely centralize.

### 7.3 API/web DTO duplication

Dataset API DTOs and dataset web DTOs remain intentionally separate.

This is correct pilot debt. Contract convergence should be considered only after crate extraction is proven useful and the representation question is understood.

Forms and Workflows API DTOs and web DTOs remain intentionally separate after their GO results. Future extractions should assume separate web DTOs by default unless inventory proves a stable shared contract is worth the dependency cost.

### 7.4 Browser-heavy UI primitive

`DraggablePanelList` moved to `tessara-web-ui` with browser drag dependencies.

This is acceptable because the primitive is domain-neutral, but it is now part of the high-fan-out UI crate and should be monitored. Avoid moving additional browser-heavy UI primitives without measuring shared-UI edit cost.

### 7.5 Feature-local transport and helper duplication

Both extracted feature crates currently own local browser transport and tiny local helpers where needed.

Do not create `tessara-web-platform` as part of the next proposal unless inventory shows the repeated copies create real maintenance risk and a small policy-neutral support crate can be justified without root auth/session/navigation coupling.

---

## 8. Candidate Feature-Crate Ranking

This ranking combines expected compile-time payoff, source weight, churn, coupling, and extraction risk.

| Rank | Candidate crate | Payoff | Risk | Notes |
| ---: | --- | --- | --- | --- |
| done | `tessara-web-forms` | Very high | Medium | GO result recorded; keep as reference model |
| done | `tessara-web-workflows` | Very high | Medium-high | GO result recorded; validates another large Leptos feature crate |
| 1 | `tessara-web-responses` | Medium-high | Medium | Leaf-like and cleanest immediate post-Workflows proposal target |
| 2 | `tessara-web-organization` | Medium-high | High | High-fan-out hierarchy concepts, but preferred before Administration to settle node DTO ownership |
| 3 | `tessara-web-administration` | High | Medium-high | Substantial payoff, but should follow Organization or Organization contract prep |
| 4 | `tessara-web-operations` | Low | Low | Aggregator surface; keep root until it grows |
| defer | auth/login/home/placeholders | Low | Varies | Keep root unless evidence changes |

The ranking is not an automatic sequence. Re-rank after each extraction using fresh measurements and the current dependency graph.

---

## 9. Near-Term Plan After Workflows

Workflows is now complete and should be used as the latest large-feature reference model. The next concrete extraction should be selected from current measurements rather than assumed from the pre-Workflows ranking.

### 9.1 Candidate choice

Use the current ranking as a starting point, but treat the short post-Workflows inventory as authoritative for the next proposal:

- `tessara-web-responses` is the next proposal target because it is smaller, leafier, and has no direct Forms/Workflows/Organization web feature dependency in the initial inventory.
- `tessara-web-organization` should come before Administration so hierarchy/node DTO ownership can be settled at the source instead of working around it from Administration.
- `tessara-web-administration` remains high-value, but its initial inventory exposed enough Organization web DTO coupling that it should follow Organization extraction or an Organization contract-prep slice.

### 9.2 Required pre-proposal checkpoint

Before drafting the next proposal, run a short inventory across Administration and Responses:

```powershell
rg -n "crate::features::(datasets|forms|workflows|responses|organization|administration)|crate::routes|AppShell|leptos_router|leptos_meta|crate::http|crate::utils|crate::types" crates\tessara-web\src\features\administration crates\tessara-web\src\features\responses crates\tessara-web\src\routes
```

The Responses proposal should also review the Workflows retained debt:

```text
local browser transport copied in three feature crates
local text/slug/pagination helpers copied across feature crates
organization/node option DTO duplication
cumulative JS/WASM bundle growth above the original dataset-pilot baseline
```

Do not create a shared web platform crate as a default next step. Create one only if the next inventory proves a small policy-neutral support crate removes real duplication without absorbing root auth/session/navigation behavior.

### 9.3 Next deliverable

Create:

```text
docs/architecture/tessara-web-responses-extraction-proposal.md
```

Organization should be the next major candidate after Responses. Administration should follow Organization, or at least follow an Organization contract-prep slice that removes direct Administration dependence on Organization web DTOs. The Responses proposal must include the same inventory, facade, route-adapter, boundary, compile/watch/browser, bundle, GO/PARTIAL/NO-GO, and rollback sections used for Forms and Workflows.

## 10. Per-Feature Extraction Template

Every future feature extraction should use this template.

### 10.1 Proposal

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

### 10.2 Public facade pattern

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

### 10.3 Default commit sequence

```text
0. inventory and measurement
1. root route-adapter preparation
2. feature crate extraction
3. boundary/test/watch/bundle documentation
```

Add a support-crate prep commit only when the feature needs new generic UI.

### 10.4 Default non-goals

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

### 10.5 Default retained-debt section

Every result document must include:

```text
- duplicated DTOs retained
- copied local helpers retained
- copied browser transport retained
- root-owned concepts intentionally not moved
- follow-up contract decisions required
```

---

## 11. Measurement and Validation Policy

### 11.1 Required commands for every extracted feature

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

### 11.2 Watch gate

For each feature:

```text
- start `cargo leptos watch`
- make a compile-affecting but behavior-neutral edit in the feature crate
- confirm path dependency detection
- confirm front bundle update
- confirm route/page availability
```

### 11.3 Bundle gate

Track:

```text
- .wasm size
- .js size
- combined JS/WASM size
- CSS size
```

A bundle increase above 5% requires explanation.

### 11.4 Test gate

Move feature-local tests into the feature crate.

Keep root route/integration tests in root.

Do not make internal symbols public solely for tests.

---

## 12. Contract Migration Policy

### 12.1 Default rule

For first extraction of a feature:

```text
move web DTOs unchanged
keep API DTOs unchanged
document duplication
measure
decide later
```

### 12.2 When to promote shared contracts

Promote a shared contract only when all are true:

```text
- API and web both use the shape
- ownership is clear
- representation is stable
- adapter duplication is causing real maintenance cost
- WASM dependency cost is measured and acceptable
- the contract crate remains Leptos/Axum/SQLx/browser-free
```

### 12.3 Representation caution

Do not introduce `uuid`, `chrono`, or other heavier shared dependencies into WASM builds just because server DTOs use them.

Choose one of:

```text
- separate API/web DTOs
- string-oriented wire contracts
- typed shared contracts with measured dependency cost
```

Make this decision per domain, not globally.

---

## 13. Shared Infrastructure Promotion Policy

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

## 14. Rollback and Stop Policy

### 14.1 Rollback

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

### 14.2 Stop rules

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

### 14.3 GO does not authorize the next feature automatically

A successful feature extraction authorizes only:

```text
- keeping that extraction
- writing the next evidence-based extraction proposal
```

It does not authorize extracting the rest of the roadmap.

---

## 15. Long-Term Roadmap

This roadmap is directional. Later stages should be progressively elaborated after fresh measurements and dependency inventories.

### Stage A — Foundation, dataset, Forms, and Workflows consolidation

**Status:** complete / active.

Completed:

```text
- extracted `tessara-web-ui`
- extracted `tessara-web-datasets`
- extracted `tessara-web-forms`
- extracted `tessara-web-workflows`
- installed boundary checker
- validated cargo-leptos path dependency watch
- retained root AppShell/routes/hydration/CSS/assets
- kept dataset DTO convergence out of scope
- kept Forms DTO convergence out of scope
- kept Workflows DTO convergence out of scope
```

Follow-up:

```text
- preserve dataset, Forms, and Workflows extraction results
- document accepted retained debt
- ensure boundary checker remains part of normal validation
```

### Stage B — Forms extraction

**Status:** complete / GO.

Goal:

```text
extract form authoring, builder, list/detail/edit surfaces into `tessara-web-forms`
```

Completed evidence:

```text
- `tessara-web-forms` extracted with root route adapters and shell ownership preserved
- Forms-local hydrate, SSR, and test-compile gates pass
- cargo-leptos watch detects Forms crate edits
- list, create, detail, and edit routes return 200 after authenticated demo seed
- boundary checker covers the Forms crate
```

Expected result:

```text
- keep as the reference model for future proposal planning
- do not reopen Forms unless a later inventory exposes a concrete contract issue
```

### Stage C — Workflows extraction

**Status:** complete / GO.

Goal:

```text
extract workflow list/detail/editor/assignments into `tessara-web-workflows`
```

Completed evidence:

```text
- `tessara-web-workflows` extracted with root route adapters and shell ownership preserved
- Workflows-local hydrate, SSR, and test-compile gates pass
- cargo-leptos watch detects Workflows crate edits
- list, create, assignments, detail, and edit routes return 200 after authenticated demo seed
- boundary checker covers the Workflows crate
- no sibling web feature dependency or root compatibility namespace was retained
```

Retained debt:

```text
- Workflows owns local browser transport and helper copies
- Workflows duplicates organization/node option DTOs locally
- cumulative bundle growth now needs closer tracking before another large Leptos extraction
```

### Stage D — Responses extraction

**Status:** selected next after short Administration/Responses inventory.

Selected candidate:

```text
tessara-web-responses
```

Rationale:

```text
leaf-ish response list/start/detail/edit feature
safer after forms and workflows are extracted
no direct sibling web feature dependency in initial inventory
```

Possible responses facade:

```rust
pub fn ResponsesIndexContent() -> impl IntoView;
pub fn ResponseStartContent() -> impl IntoView;
pub fn ResponseDetailContent(submission_id: String) -> impl IntoView;
pub fn ResponseEditContent(submission_id: String) -> impl IntoView;
```

Responses is selected for the next proposal. Re-rank again after the Responses result, with Organization preferred before Administration.

### Stage E — Organization extraction

**Status:** preferred next major candidate after Responses.

Goal:

```text
extract organization explorer, detail, related work, node editor, metadata, and tree UI into `tessara-web-organization`
```

Why before Administration:

```text
organization/hierarchy concepts are high fan-out
datasets/forms/workflows/admin may all touch node concepts
Administration already imports Organization web DTOs
settling Organization first should reduce Administration extraction risk
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

### Stage F — Administration extraction

**Status:** after Organization or Organization contract-prep.

Goal:

```text
extract user, role, access, and node-type administration surfaces into `tessara-web-administration`
```

Why after Organization:

```text
substantial user/role/node-type management surface
potentially strong focused-loop payoff
initial inventory shows Organization web DTO coupling to resolve first
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

### Stage G — Reassess small/root-owned surfaces

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

### Stage H — Optional shared platform

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

### Stage I — Optional contract/domain evolution

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

### Stage J — Future service extraction

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

## 16. Next Artifact

This roadmap is the durable architecture artifact after the dataset, Forms, and Workflows extractions.

The next implementation-specific document is:

```text
docs/architecture/tessara-web-responses-extraction-proposal.md
```

After Responses, prepare:

```text
docs/architecture/tessara-web-organization-extraction-proposal.md
```

The Responses proposal should be implementation-ready and should include inventory, facade, route adapters, dependency rules, cross-feature dependency resolution, validation commands, measurement gates, GO/PARTIAL/NO-GO criteria, and rollback steps.

---

## 17. Summary

The dataset, Forms, and Workflows extractions validated the core thesis:

> Tessara can use feature-area crates to regain focused development loops while retaining a single routed Leptos app and one primary API service.

The next move is not a mass split. It is a Responses extraction proposal, followed by Organization before Administration so node/hierarchy DTO ownership is settled at the source.

The long-term roadmap points toward feature-area crates for the major product surfaces, but each stage remains conditional on evidence.

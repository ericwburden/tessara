# Tessara Web Feature-Crate Migration Plan and Long-Term Roadmap

**Status:** Active architecture roadmap after successful `tessara-web-datasets`, `tessara-web-forms`, `tessara-web-workflows`, `tessara-web-responses`, and `tessara-web-organization` extractions
**Primary near-term target:** Post-Organization results review; Administration remains deferred
**Long-term direction:** product-area frontend crates under a modular monolith
**Non-goal:** microservices as a frontend compile-time solution

---

## 1. Purpose

The dataset pilot showed that extracting a large, coherent frontend feature area into its own Rust crate can improve focused development and validation loops without unacceptable full-app regressions. This plan turns that pilot result into a controlled, evidence-gated migration path for the rest of `tessara-web`.

This plan is not a mandate to split everything immediately. It defines:

1. the accepted architecture after the dataset pilot;
2. permanent crate-boundary rules;
3. the completed `tessara-web-forms`, `tessara-web-workflows`, `tessara-web-responses`, and `tessara-web-organization` results;
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

## 5. Responses Extraction Findings

The Responses extraction should also be treated as **GO**.

Governing evidence:

- `docs/architecture/tessara-web-responses-extraction-proposal.md`
- `docs/architecture/tessara-web-responses-extraction-results.md`

### 5.1 Architecture result

The extraction created:

```text
crates/tessara-web-responses
```

and preserved the accepted feature-crate pattern:

- root `tessara-web` owns Responses route adapters, `SubmissionRouteParams`, auth guard, `AppShell`, hydration, document, CSS/assets, and cargo-leptos app ownership;
- `tessara-web-responses` owns response list/start/detail/edit content, loaders, actions, browser transport, DTOs, display helpers, value collection, and feature-local support helpers;
- Home owns its pending-work start action locally instead of importing Responses internals;
- root routes import only the approved content facade:

```rust
ResponsesIndexContent
ResponseStartContent
ResponseDetailContent
ResponseEditContent
```

The root `features::responses` module was removed rather than preserved as a compatibility layer.

### 5.2 Compile result

Latest clean-target comparison against the original pre-Forms extraction baseline:

| Probe | Original clean baseline | Post-Workflows clean target | Current Responses clean target | Delta vs original |
| --- | ---: | ---: | ---: | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 247.39s | 115.00s | -190.92s / -62.4% |
| `cargo check -p tessara-api --features ssr` | 392.17s | 337.40s | 195.49s | -196.68s / -50.2% |
| `cargo check -p tessara-web-responses --no-default-features --features ssr` | n/a | n/a | 83.09s | focused loop available |
| `cargo check -p tessara-web-responses --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | n/a | 98.05s | focused loop available |
| `cargo test -p tessara-web-responses --no-default-features --features ssr --no-run -j 1` | n/a | n/a | 290.76s | focused test compile available |

The practical result is a successful extraction with no full-app compile regression, materially faster full-app clean checks than the original baseline, and a new Responses-focused check loop.

### 5.3 Watch and route result

The Responses watch gate passed:

- `cargo leptos watch` detected edits in `crates\tessara-web-responses`;
- `tessara-web-responses` and root `tessara-web` rebuilt through the path dependency;
- wasm-bindgen completed and `Watch updated Front` was reported;
- `/health` returned 200 under watch;
- authenticated `/`, `/responses`, `/responses/new`, `/responses/:submission_id`, and `/responses/:submission_id/edit` returned 200 after demo seed;
- direct `/responses/new?workflowAssignmentId=...` returned 200;
- the assignment-start endpoint returned a response id, and the resulting edit route returned 200.

### 5.4 Boundary and bundle result

The boundary checker now covers `tessara-web-responses` and rejects root/API/sibling web feature dependencies plus router/meta dependencies.

`tessara-web-responses` has no dependency on:

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
```

Immediate bundle growth from the Workflows post-extraction bundle remained below the 5% threshold. Cumulative growth since the dataset pilot remains above 5%, so bundle size should remain a standing gate for Organization.

### 5.5 Retained debt and caveats

Retained intentional debt:

- Responses keeps web DTOs separate from API DTOs.
- Responses owns local browser transport, unauthorized handling, and small local support helpers.
- Home owns a separate pending-work start action.
- Shared transport/helper contracts were not generalized during extraction.
- Cumulative JS/WASM growth should remain a gate for future large Leptos extractions.

## 6. Organization Extraction Findings

The Organization extraction should also be treated as **GO**.

Governing evidence:

- `docs/architecture/tessara-web-organization-extraction-proposal.md`
- `docs/architecture/tessara-web-organization-extraction-results.md`

### 6.1 Architecture result

The extraction created:

```text
crates/tessara-web-organization
```

and preserved the accepted feature-crate pattern:

- root `tessara-web` owns Organization route adapters, `NodeRouteParams`, `AppShell`, hydration, document, CSS/assets, and cargo-leptos app ownership;
- `tessara-web-organization` owns organization index/detail/create/edit content, tree rendering, related work, node editor state/loaders/actions, DTOs, browser transport, and feature-local support helpers;
- Administration now owns local role/node-type/metadata DTOs instead of importing Organization web DTOs;
- Organization now owns local node editor payloads instead of importing Administration payloads;
- root routes import only the approved content facade:

```rust
OrganizationIndexContent
OrganizationDetailContent
OrganizationNodeCreateContent
OrganizationNodeEditContent
```

The root `features::organization` module was removed rather than preserved as a compatibility layer.

### 6.2 Compile result

Warm active-target validation passed after extraction:

| Probe | Current active target |
| --- | ---: |
| `cargo check -p tessara-web-organization --no-default-features --features hydrate --target wasm32-unknown-unknown` | 0.95s |
| `cargo check -p tessara-web-organization --no-default-features --features ssr` | 0.91s |
| `cargo test -p tessara-web-organization --no-default-features --features ssr --no-run -j 1` | 46.59s |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 2.47s |
| `cargo check -p tessara-api --features ssr` | 5.06s |
| `cargo leptos build` | 74.69s |

These were warm-cache active-target timings, not isolated clean-target benchmarks. They are still useful as final implementation gates, while the earlier clean-target baselines remain the durable comparison points.

Follow-up release frontend validation also passed after fixing the pre-existing release-only recursion limit blocker in `tessara-web-datasets`:

| Probe | Result |
| --- | --- |
| `cargo leptos build --release --frontend-only` | pass |
| `wasm-tools validate target\site\pkg\tessara-web.wasm` | pass |

The release blocker was a Rust query-depth overflow in the deeply nested Leptos/Tachys `DraggablePanelList` hydration type graph. Adding `#![recursion_limit = "512"]` to `tessara-web-datasets` resolved it, matching the existing root `tessara-web` and `tessara-web-forms` crate attributes.

### 6.3 Watch and route result

The Organization watch gate passed:

- `cargo leptos watch` detected edits in `crates\tessara-web-organization`;
- `tessara-web-organization` and root `tessara-web` rebuilt through the path dependency;
- wasm-bindgen completed and `Watch updated Front` was reported;
- `/health` returned `ok`;
- authenticated `/organization`, `/organization/new`, `/administration`, `/administration/users`, `/administration/roles`, and `/administration/node-types` returned 200 after demo seed;
- unauthenticated `/organization` returned 303 to login.

General `scripts\smoke.ps1` was attempted, but its pre-server `cargo test -p tessara-api --test demo_flow` step failed on two unrelated dataset tests. A narrower manual route smoke verified the Organization/Admin web surfaces touched by this extraction.

### 6.4 Boundary and bundle result

The boundary checker now covers `tessara-web-organization` and rejects root/API/sibling web feature dependencies plus router/meta dependencies.

`tessara-web-organization` has no dependency on:

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

Immediate dev-profile bundle growth from the Responses post-extraction bundle remained below the 5% threshold. Cumulative dev-profile growth since the dataset pilot remains above 5%, so bundle size should remain a standing gate for future large Leptos extractions.

The corrected release frontend artifact is much smaller than the dev-profile artifact previously tracked:

| Release artifact | Bytes |
| --- | ---: |
| `tessara-web.wasm` | 10,103,049 |
| `tessara-web.js` | 26,091 |
| `tessara-web.css` | 101,453 |

Future production-size decisions should use release artifacts. Dev-profile bundle measurements remain useful only as a trend signal.

### 6.5 Retained debt and caveats

Retained intentional debt:

- Organization keeps web DTOs separate from API DTOs.
- Organization owns local browser transport, unauthorized handling, and small local support helpers.
- Administration owns local role/node-type/metadata DTO copies while Administration extraction remains deferred.
- Administration still reuses some `organization-*` CSS class names.
- Shared transport/helper contracts were not generalized during extraction.

## 7. Accepted Architecture After Dataset, Forms, Workflows, Responses, and Organization

The architecture is now:

```text
crates/
├── tessara-web              # root app, route registry, shell, hydration, document, CSS/assets
├── tessara-web-ui           # generic Leptos UI primitives
├── tessara-web-datasets     # extracted dataset feature crate
├── tessara-web-forms        # extracted forms feature crate
├── tessara-web-workflows    # extracted workflows feature crate
├── tessara-web-responses    # extracted responses feature crate
├── tessara-web-organization # extracted organization feature crate
├── tessara-api              # single deployable API/SSR service
├── tessara-core             # shared domain primitives
├── tessara-datasets         # dataset domain rules
├── tessara-forms            # form domain rules
├── tessara-hierarchy        # hierarchy/domain rules
├── tessara-submissions      # submission/domain rules
└── other domain/stub crates
```

### 7.1 Root `tessara-web` owns

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

### 7.2 `tessara-web-ui` owns

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

### 7.3 Feature crates own

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

## 8. Permanent Boundary Rules

These rules apply to the current dataset crate and all future feature crates.

### 8.1 Feature web crates must not depend on

```text
- root `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates
- root route modules
- root AppShell/session/navigation/auth modules
- `leptos_router`, unless a specific feature proves root adapters cannot cover the need
- `leptos_meta`, unless a specific feature proves it owns metadata composition
```

### 8.2 Feature web crates may depend on

```text
- `tessara-web-ui`
- `leptos`
- `icons`
- `serde` / `serde_json`
- hydrate-gated `gloo-net` / `web-sys` / `wasm-bindgen` where needed
- domain crates for stable pure rules or stable contracts
```

### 8.3 Domain and contract crates must not depend on

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

### 8.4 Boundary checker

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

## 9. Current Intentional Debt

The completed feature extractions intentionally retained some debt to keep each move focused and reversible.

### 9.1 Dataset-local browser transport

`tessara-web-datasets` has private copied browser HTTP transport. This avoids a root app dependency.

Do **not** create `tessara-web-platform` yet. Reassess only after future feature inventories show helper duplication is becoming real maintenance drag.

### 9.2 Dataset-local helpers

Minimal text and pagination helpers are local to `tessara-web-datasets`.

Forms, Workflows, Responses, and Organization now also own local helper copies where needed. Treat this as standing evidence to review before any future shared web platform proposal, not as an automatic mandate to centralize.

### 9.3 API/web DTO duplication

Dataset API DTOs and dataset web DTOs remain intentionally separate.

This remains correct extraction debt. Contract convergence should be considered only after crate extraction is proven useful and the representation question is understood.

Forms, Workflows, Responses, and Organization API DTOs and web DTOs remain intentionally separate after their GO results. Future extractions should assume separate web DTOs by default unless inventory proves a stable shared contract is worth the dependency cost.

### 9.4 Browser-heavy UI primitive

`DraggablePanelList` moved to `tessara-web-ui` with browser drag dependencies.

This is acceptable because the primitive is domain-neutral, but it is now part of the high-fan-out UI crate and should be monitored. Avoid moving additional browser-heavy UI primitives without measuring shared-UI edit cost.

### 9.5 Feature-local transport and helper duplication

The extracted feature crates currently own local browser transport and tiny local helpers where needed.

Do not create `tessara-web-platform` as part of the next feature extraction by default. Create one only if a fresh inventory shows repeated copies create real maintenance risk and a small policy-neutral support crate can be justified without root auth/session/navigation coupling.

---

## 10. Candidate Feature-Crate Ranking

This ranking combines expected compile-time payoff, source weight, churn, coupling, and extraction risk.

| Rank | Candidate crate | Payoff | Risk | Notes |
| ---: | --- | --- | --- | --- |
| done | `tessara-web-forms` | Very high | Medium | GO result recorded; keep as reference model |
| done | `tessara-web-workflows` | Very high | Medium-high | GO result recorded; validates another large Leptos feature crate |
| done | `tessara-web-responses` | Medium-high | Medium | GO result recorded; validates a smaller leaf-like feature crate |
| done | `tessara-web-organization` | Medium-high | High | GO result recorded; settled node DTO ownership before Administration |
| defer | `tessara-web-administration` | High | Medium-high | Defer to a future sprint; likely split into smaller administration slices |
| 1 | `tessara-web-operations` | Low | Low | Aggregator surface; keep root until it grows |
| defer | auth/login/home/placeholders | Low | Varies | Keep root unless evidence changes |

The ranking is not an automatic sequence. Re-rank after each extraction using fresh measurements and the current dependency graph.

---

## 11. Near-Term Plan After Organization

Organization is now complete and should be used as the latest high-fan-out feature-crate reference model. Administration remains intentionally deferred to a future sprint where it can be broken into smaller slices.

### 11.1 Candidate choice

Use the current ranking as a starting point, but do not automatically start another large extraction:

- `tessara-web-administration` remains high-value, but it is intentionally deferred to a future sprint where it can be split into smaller administration slices.
- `tessara-web-operations`, auth/login, Home, and placeholders should remain root-owned unless their source size or churn changes materially.
- the next implementation proposal should be chosen only after reviewing current source size, churn, coupling, bundle growth, and focused-loop payoff.

### 11.2 Required pre-proposal checkpoint

Before drafting the next non-Administration proposal, run a short inventory across the candidate feature and its consumers:

```powershell
rg -n "crate::features::(datasets|forms|workflows|responses|organization|administration)|crate::routes|AppShell|leptos_router|leptos_meta|crate::http|crate::utils|crate::types" crates\tessara-web\src crates\tessara-web-*\src
```

The next proposal should review the retained debt from Forms, Workflows, Responses, and Organization:

```text
local browser transport copied in five feature crates
local text/slug/pagination/query/status helpers copied across feature crates
organization/node option DTO duplication in Workflows and Administration, with Administration now owning its local node-type DTO copies
dev-profile cumulative JS/WASM bundle growth above the original dataset-pilot baseline
```

Do not create a shared web platform crate as a default next step. Create one only if a fresh inventory proves a small policy-neutral support crate removes real duplication without absorbing root auth/session/navigation behavior.

### 11.3 Next deliverable

Choose and draft the next implementation proposal only after post-Organization results review. Administration should stay deferred unless the sprint is explicitly scoped to split Administration into smaller pieces.

```text
docs/architecture/tessara-web-<next-feature>-extraction-proposal.md
```

The next proposal must include the same inventory, facade, route-adapter, boundary, compile/watch/browser, bundle, GO/PARTIAL/NO-GO, and rollback sections used for Forms, Workflows, Responses, and Organization.

## 12. Per-Feature Extraction Template

Every future feature extraction should use this template.

### 12.1 Proposal

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

### 12.2 Public facade pattern

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

### 12.3 Default commit sequence

```text
0. inventory and measurement
1. root route-adapter preparation
2. feature crate extraction
3. boundary/test/watch/bundle documentation
```

Add a support-crate prep commit only when the feature needs new generic UI.

### 12.4 Default non-goals

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

### 12.5 Default retained-debt section

Every result document must include:

```text
- duplicated DTOs retained
- copied local helpers retained
- copied browser transport retained
- root-owned concepts intentionally not moved
- follow-up contract decisions required
```

---

## 13. Measurement and Validation Policy

### 13.1 Required commands for every extracted feature

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

### 13.2 Watch gate

For each feature:

```text
- start `cargo leptos watch`
- make a compile-affecting but behavior-neutral edit in the feature crate
- confirm path dependency detection
- confirm front bundle update
- confirm route/page availability
```

### 13.3 Bundle gate

Track:

```text
- .wasm size
- .js size
- combined JS/WASM size
- CSS size
```

Production bundle decisions should use `cargo leptos build --release`, preferably with `--frontend-only` when server output is not under review. Dev-profile bundle measurements may be recorded for trend continuity, but they must be labeled as dev-profile measurements because they can include large name/debug metadata.

A release bundle increase above 5% requires explanation. A dev-profile bundle increase above 5% should trigger investigation, not a production-size conclusion.

Future bundle work should include a focused Leptos lazy-loading/code-splitting pilot. `cargo leptos build --split` can split WASM around `#[lazy]`/`#[lazy_route]` boundaries, but this is not automatic at Cargo feature-crate boundaries. Pilot one extracted route area first, then decide whether root route adapters should own lazy route declarations or whether feature crates need a narrow, documented `leptos_router` exception for lazy routes.

### 13.4 Test gate

Move feature-local tests into the feature crate.

Keep root route/integration tests in root.

Do not make internal symbols public solely for tests.

---

## 14. Contract Migration Policy

### 14.1 Default rule

For first extraction of a feature:

```text
move web DTOs unchanged
keep API DTOs unchanged
document duplication
measure
decide later
```

### 14.2 When to promote shared contracts

Promote a shared contract only when all are true:

```text
- API and web both use the shape
- ownership is clear
- representation is stable
- adapter duplication is causing real maintenance cost
- WASM dependency cost is measured and acceptable
- the contract crate remains Leptos/Axum/SQLx/browser-free
```

### 14.3 Representation caution

Do not introduce `uuid`, `chrono`, or other heavier shared dependencies into WASM builds just because server DTOs use them.

Choose one of:

```text
- separate API/web DTOs
- string-oriented wire contracts
- typed shared contracts with measured dependency cost
```

Make this decision per domain, not globally.

---

## 15. Shared Infrastructure Promotion Policy

Do not create `tessara-web-platform` yet.

Promotion rule:

```text
Promote a helper only when multiple extracted feature crates independently need the same helper and local duplication is becoming a maintenance risk.
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

## 16. Rollback and Stop Policy

### 16.1 Rollback

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

### 16.2 Stop rules

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

### 16.3 GO does not authorize the next feature automatically

A successful feature extraction authorizes only:

```text
- keeping that extraction
- writing the next evidence-based extraction proposal
```

It does not authorize extracting the rest of the roadmap.

---

## 17. Long-Term Roadmap

This roadmap is directional. Later stages should be progressively elaborated after fresh measurements and dependency inventories.

### Stage A — Foundation, dataset, Forms, Workflows, Responses, and Organization consolidation

**Status:** complete / active.

Completed:

```text
- extracted `tessara-web-ui`
- extracted `tessara-web-datasets`
- extracted `tessara-web-forms`
- extracted `tessara-web-workflows`
- extracted `tessara-web-responses`
- extracted `tessara-web-organization`
- installed boundary checker
- validated cargo-leptos path dependency watch
- retained root AppShell/routes/hydration/CSS/assets
- kept dataset DTO convergence out of scope
- kept Forms DTO convergence out of scope
- kept Workflows DTO convergence out of scope
- kept Responses DTO convergence out of scope
- kept Organization DTO convergence out of scope
```

Follow-up:

```text
- preserve dataset, Forms, Workflows, Responses, and Organization extraction results
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

**Status:** complete / GO.

Goal:

```text
extract response list/start/detail/edit surfaces into `tessara-web-responses`
```

Completed evidence:

```text
- `tessara-web-responses` extracted with root route adapters and shell ownership preserved
- Responses-local hydrate, SSR, and test-compile gates pass
- cargo-leptos watch detects Responses crate edits
- list, start, detail, edit, and direct assignment-start routes return 200 after authenticated demo seed
- boundary checker covers the Responses crate
- no sibling web feature dependency or root compatibility namespace was retained
```

Retained debt:

```text
- Responses owns local browser transport and helper copies
- Home owns a separate pending-work start action
- cumulative bundle growth remains above the original dataset-pilot baseline
```

### Stage E — Organization extraction

**Status:** complete / GO.

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

Completed evidence:

```text
- `tessara-web-organization` extracted with root route adapters and shell ownership preserved
- Organization-local hydrate, SSR, docs, and test-compile gates pass
- cargo-leptos watch detects Organization crate edits
- organization and administration routes return 200 after authenticated demo seed
- unauthenticated organization route redirect behavior is preserved
- boundary checker covers the Organization crate
- release frontend build passes after raising `tessara-web-datasets` recursion limit for the existing DraggablePanelList/Tachys type graph
- Administration no longer imports Organization web DTOs
- Organization no longer imports Administration node editor payloads
- no sibling web feature dependency or root compatibility namespace was retained
```

Public facade:

```rust
pub fn OrganizationIndexContent() -> impl IntoView;
pub fn OrganizationDetailContent(node_id: String) -> impl IntoView;
pub fn OrganizationNodeCreateContent() -> impl IntoView;
pub fn OrganizationNodeEditContent(node_id: String) -> impl IntoView;
```

Retained debt:

```text
- Organization owns local browser transport and helper copies
- Administration owns local role/node-type/metadata DTO copies while Administration extraction remains deferred
- Administration still reuses some organization CSS class names
- dev-profile cumulative bundle growth remains above the original dataset-pilot baseline
- release bundle measurement is now available and should be used for production-size decisions
```

### Stage F — Administration deferral

**Status:** deferred to a future sprint.

Goal:

```text
break Administration into smaller future extraction slices before attempting a full `tessara-web-administration` crate
```

Why defer:

```text
substantial user/role/node-type management surface
potentially strong focused-loop payoff
Organization web DTO coupling has been removed
likely better handled as user/role/access/node-type slices rather than one large extraction
```

Possible future facade, if a full Administration crate is still useful after slicing:

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

Only if a future inventory proves repeated transport/helper copies are becoming maintenance drag:

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

## 18. Next Artifact

This roadmap is the durable architecture artifact after the dataset, Forms, Workflows, Responses, and Organization extractions.

The latest implementation-specific documents are:

```text
docs/architecture/tessara-web-organization-extraction-proposal.md
docs/architecture/tessara-web-organization-extraction-results.md
```

The next implementation-specific proposal is intentionally undecided. Administration remains deferred unless the sprint is explicitly scoped to split it into smaller slices.

---

## 19. Summary

The dataset, Forms, Workflows, Responses, and Organization extractions validated the core thesis:

> Tessara can use feature-area crates to regain focused development loops while retaining a single routed Leptos app and one primary API service.

The next move is not a mass split. It is a post-Organization review and a fresh candidate selection. Administration is explicitly deferred to a future sprint where it can be broken into smaller slices.

The long-term roadmap points toward feature-area crates for the major product surfaces, but each stage remains conditional on evidence.

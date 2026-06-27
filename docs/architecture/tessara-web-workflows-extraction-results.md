# Tessara Web Workflows-Crate Extraction Results

Status: GO

Governing plan:

- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-workflows-extraction-proposal.md`
- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-feature-crate-roadmap.md`

## Measured Environment

- worktree: `C:\Users\eric-dev\Projects\tessara-refactoring`
- branch: `codex/refactoring-work`
- base commit before this working-tree implementation: `9d9988dfc141f973da9cffd0faff4a67639d43fb`
- rustc: `1.95.0 (59807616e 2026-04-14)`
- Cargo: `1.95.0 (f2d3ce0bd 2026-03-21)`
- cargo-leptos: `0.3.6`
- raw evidence:
  - clean-target timings: `tmp\web-workflows-extraction\W4-clean-current`
  - watch logs and route smoke: `tmp\web-workflows-extraction\W4-watch`
  - generated docs: `target\doc\tessara_web_workflows\index.html`

## Baseline Inventory

The W0/W1 inventory found and resolved the expected Workflows blockers before extraction:

- root-owned `AppShell`, `WorkflowRouteParams`, and route-param parsing stayed in `crates\tessara-web\src\routes\workflows.rs`;
- Workflows content stopped importing root route, router, shell, session, or metadata concerns;
- external consumers in Home and Responses stopped importing Workflows internals by owning local pending-work and revision-label helpers;
- small generic dropdown UI moved to `tessara-web-ui`;
- Workflows owns local web DTOs for organization nodes, node type options, workflow data, assignments, display helpers, slug/text/pagination helpers, and browser transport.

No compatibility `crate::features::workflows` namespace was retained.

## Extraction Summary

Implemented architecture:

- `crates\tessara-web-workflows` owns workflow list, detail, new/edit, assignments, loaders, DTOs, payloads, display helpers, local transport, and feature-local support helpers.
- root `crates\tessara-web\src\routes\workflows.rs` owns route registration, `WorkflowRouteParams`, auth guard, and `AppShell` wrapping.
- root `crates\tessara-web\src\features\mod.rs` no longer exposes a `workflows` feature module.
- root shared workflow-only DTO/helper modules and the old root slug helper were removed after consumers moved local.
- `tessara-web-workflows` public facade exports only:
  - `WorkflowsIndexContent`
  - `WorkflowDetailContent`
  - `WorkflowNewContent`
  - `WorkflowEditContent`
  - `WorkflowAssignmentsContent`

## Command Matrix

Clean-target comparison, using isolated target directories under `tmp\web-workflows-extraction\W4-clean-current`:

| Probe | Original Forms baseline | Post-Forms clean target | Current clean target | Delta vs original |
| --- | ---: | ---: | ---: | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 270.47s | 247.39s | -58.53s / -19.1% |
| `cargo check -p tessara-api --features ssr` | 392.17s | 392.15s | 337.40s | -54.77s / -14.0% |
| `cargo leptos build` | `>900s` timeout | 799.46s | 410.31s | completed, at least 489.69s under timeout |
| `cargo check -p tessara-web-workflows --no-default-features --features ssr` | n/a | n/a | 204.98s | focused loop available |
| `cargo check -p tessara-web-workflows --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | n/a | 216.68s | focused loop available |
| `cargo test -p tessara-web-workflows --no-default-features --features ssr --no-run -j 1` | n/a | n/a | 323.48s | focused test compile available |

Warm active-target gates also passed:

- `cargo fmt --all --check`
- `cargo check -p tessara-web-workflows --no-default-features --features hydrate --target wasm32-unknown-unknown`
- `cargo check -p tessara-web-workflows --no-default-features --features ssr`
- `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown`
- `cargo check -p tessara-api --features ssr`
- `cargo test -p tessara-web-workflows --no-default-features --features ssr --no-run -j 1`
- `cargo test -p tessara-web-workflows --no-default-features --features ssr`: pass, 0 tests
- `cargo doc -p tessara-web-workflows --no-deps`
- `cargo leptos build`

## Boundary Gates

`scripts\check-web-crate-boundaries.ps1` now checks `tessara-web-workflows` for both `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown`.

It rejects dependency paths from `tessara-web-workflows` to:

- `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates except `tessara-web-ui`
- `leptos_router`
- `leptos_meta`

It also audits source for route/shell/router/meta/old-namespace/sibling-feature references.

Result:

- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1`: pass
- old Workflows namespace audit: no matches in root or extracted crate source
- extracted crate route/shell/router/meta/sibling-feature audit: no matches
- root route adapter imports only the approved Workflows content facade

The new crate keeps browser transport dependencies non-optional, matching the current Forms and Datasets extraction pattern. SSR compilation passes with this manifest, and the result avoids router/meta/root dependencies.

## Watch-Mode Results

Command:

```powershell
cargo leptos watch
```

Initial watch build:

- compiled `tessara-web-workflows` for front and server paths;
- served at `http://127.0.0.1:8080`;
- `/health` returned 200;
- known local warnings remained: missing `node_modules`, missing `crates/tessara-web/public`.

Temporary edit target:

- `crates\tessara-web-workflows\src\lib.rs`
- behavior-neutral doc-comment marker

Observed rebuilds:

| Edit | Detection | Elapsed evidence |
| --- | --- | ---: |
| `workflows-edit-1` | pass; watcher compiled `tessara-web-workflows` and `tessara-web`, ran wasm-bindgen, and reported `Watch updated Front` | compile 11.57s + wasm-bindgen 3.28s |
| cleanup | pass; watcher compiled `tessara-web-workflows` and `tessara-web`, ran wasm-bindgen, and reported `Watch updated Front` | observed by poll in 9.09s; wasm-bindgen 3.88s |

Authenticated route smoke under the running watcher:

- `POST /api/auth/login` with local dev admin credentials returned 200.
- `POST /api/demo/seed` returned 200.
- `GET /api/workflows` returned 200 with 7 workflows.
- authenticated `GET /workflows` returned 200.
- authenticated `GET /workflows/new` returned 200.
- authenticated `GET /workflows/assignments` returned 200.
- authenticated `GET /workflows/ffba2d85-c857-43a7-ab86-802f62030a57` returned 200.
- authenticated `GET /workflows/ffba2d85-c857-43a7-ab86-802f62030a57/edit` returned 200.

Cleanup:

- temporary marker was removed;
- cleanup rebuild passed;
- watcher process was stopped;
- no Cargo/Rust/API watcher processes remained.

## Bundle Size

Current post-extraction bundle from `target\site`:

| Artifact | Bytes |
| --- | ---: |
| wasm total | 29,509,636 |
| js total | 60,563 |
| combined JS/WASM | 29,570,199 |
| css total | 120,731 |
| total site files | 29,698,123 |

Comparison:

| Baseline | Combined JS/WASM | Delta |
| --- | ---: | ---: |
| Forms post-extraction bundle | 28,653,760 | +916,439 / +3.2% |
| dataset post-extraction bundle | 27,788,530 | +1,781,669 / +6.4% |
| original dataset pilot baseline | 27,663,145 | +1,907,054 / +6.9% |

The Workflows delta from the immediate prior extraction remains below 5%. The cumulative growth since the dataset pilot exceeds 5%, so bundle size should continue to be monitored before extracting another large Leptos surface.

## Retained Debt

- Workflows keeps web DTOs separate from API DTOs.
- Workflows owns local browser transport and unauthorized redirect handling.
- Workflows owns local text, slug, filtering, pagination, and URL helpers copied from prior root patterns.
- Organization node and node type option DTOs are duplicated locally instead of shared with Organization.
- Root owns routes, route params, auth guard, shell, hydration, document, CSS/assets, and cargo-leptos app wiring.

## GO Decision

Decision: GO.

Evidence:

- workflow-local hydrate check passes
- workflow-local SSR check passes
- workflow-local test compile passes
- workflow-local test execution passes with zero tests
- root hydrate check passes
- API SSR check passes
- `cargo leptos build` passes in an isolated target
- `cargo leptos watch` detects extracted Workflows crate edits
- authenticated Workflows list, new, assignments, detail, and edit routes return HTTP 200 under watch
- boundary checker passes with no permanent exceptions
- public API is the approved route-content facade
- root retains route/shell/hydration/CSS/assets ownership
- no API/web DTO convergence was required
- no sibling web feature crate dependency was introduced
- immediate bundle growth is under 5%

Follow-up:

- Treat cumulative bundle growth as a measurement item for the next large extraction.
- Do not create a shared web platform crate yet; reassess after the next extraction inventory proves whether local transport/helper duplication has become real maintenance drag.

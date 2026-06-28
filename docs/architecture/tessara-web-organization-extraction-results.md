# Tessara Web Organization-Crate Extraction Results

Status: GO

Governing plan:

- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-organization-extraction-proposal.md`
- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-feature-crate-roadmap.md`

## Measured Environment

- worktree: `C:\Users\eric-dev\Projects\tessara-refactoring`
- branch: `codex/refactoring-work`
- rustc: `1.95.0 (59807616e 2026-04-14)`
- Cargo: `1.95.0 (f2d3ce0bd 2026-03-21)`
- cargo-leptos: `0.3.6`
- raw evidence:
  - inventory and baseline: `tmp\web-organization-extraction\O0-baseline`
  - extraction gates: `tmp\web-organization-extraction\O4-extract-crate`
  - final build, boundary, bundle, and route smoke: `tmp\web-organization-extraction\O5-clean-current`
  - watch logs: `tmp\web-organization-extraction\O5-watch`
  - generated docs: `target\doc\tessara_web_organization\index.html`

## Baseline Inventory

The O0 inventory found and resolved the expected Organization blockers:

- root-owned `AppShell`, `NodeRouteParams`, and route-param parsing stayed in `crates\tessara-web\src\routes\organization.rs`;
- Administration stopped importing Organization web DTOs by owning local role, node-type, and metadata DTOs;
- Organization stopped importing Administration create/update node payloads by owning local node editor payloads;
- Organization content stopped importing root route, router, shell, or route-param concerns;
- small root HTTP, metadata, text, pagination, and URL helpers were copied locally;
- root UI imports became direct `tessara-web-ui` imports.

No compatibility `crate::features::organization` namespace was retained.

## Extraction Summary

Implemented architecture:

- `crates\tessara-web-organization` owns organization index/detail/create/edit content, tree rendering, related-work views, node editor state/loaders/actions, DTOs, browser transport, and feature-local support helpers.
- root `crates\tessara-web\src\routes\organization.rs` owns route registration, `NodeRouteParams`, and `AppShell` wrapping.
- root `crates\tessara-web\src\features\mod.rs` no longer exposes an `organization` feature module.
- Administration remains in root and owns its own role/node-type DTOs while Administration extraction stays deferred.
- `tessara-web-organization` public facade exports only:
  - `OrganizationIndexContent`
  - `OrganizationDetailContent`
  - `OrganizationNodeCreateContent`
  - `OrganizationNodeEditContent`

## Command Matrix

Warm active-target gates passed:

| Probe | Result | Time |
| --- | --- | ---: |
| `cargo check -p tessara-web-organization --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass | 0.95s |
| `cargo check -p tessara-web-organization --no-default-features --features ssr` | pass | 0.91s |
| `cargo test -p tessara-web-organization --no-default-features --features ssr --no-run -j 1` | pass | 46.59s |
| `cargo doc -p tessara-web-organization --no-deps` | pass | 3.51s |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass | 2.47s |
| `cargo check -p tessara-api --features ssr` | pass | 5.06s |
| `cargo leptos build` | pass | 74.69s |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1` | pass | 4.25s |

Final post-roadmap revalidation on the current tree also passed:

| Probe | Result | Time |
| --- | --- | ---: |
| `cargo check -p tessara-web-organization --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass | 1.32s |
| `cargo check -p tessara-web-organization --no-default-features --features ssr` | pass | 1.28s |
| `cargo test -p tessara-web-organization --no-default-features --features ssr --no-run -j 1` | pass | 6.45s |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass | 3.00s |
| `cargo check -p tessara-api --features ssr` | pass | 5.40s |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1` | pass | latest run |
| `git diff --check` | pass | latest run |

Known local `cargo leptos build` warnings remain unchanged:

- `node_modules` is not installed, so cargo-leptos continues without it.
- `crates/tessara-web/public` does not exist, so asset copying is skipped.

Release frontend follow-up:

- initial `cargo leptos build --release --frontend-only` failed in `tessara-web-datasets` with a Rust query-depth overflow inside the deeply nested Leptos/Tachys `DraggablePanelList` hydration type graph;
- adding `#![recursion_limit = "512"]` to `tessara-web-datasets` resolved the release frontend blocker, matching the existing root `tessara-web` and `tessara-web-forms` crate attributes;
- `cargo leptos build --release --frontend-only` now passes;
- `wasm-tools validate target\site\pkg\tessara-web.wasm` passes;
- the release site artifact is materially smaller than the dev-profile bundle previously tracked:

| Release artifact | Bytes |
| --- | ---: |
| `tessara-web.wasm` | 10,103,049 |
| `tessara-web.js` | 26,091 |
| `tessara-web.css` | 101,453 |

This confirms the earlier 31 MB wasm figure was a dev-profile measurement. Bundle gates should prefer release artifacts for production-size decisions and may continue to track dev artifacts only as a trend signal.

## Boundary Gates

`scripts\check-web-crate-boundaries.ps1` now checks `tessara-web-organization` for both `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown`.

It rejects dependency paths from `tessara-web-organization` to:

- `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates except `tessara-web-ui`
- `leptos_router`
- `leptos_meta`

It also audits source for route/shell/router/meta/old-namespace/sibling-feature references.

Result:

- boundary checker passes;
- old Organization namespace audit passes;
- extracted crate route/shell/router/meta/sibling-feature audit passes;
- root route adapter imports only the approved Organization content facade.

## Watch And Route Results

Command:

```powershell
cargo leptos watch
```

Initial watch build:

- compiled `tessara-web-organization` for front and server paths;
- served at `http://127.0.0.1:8080`;
- `/health` returned `ok`.

Temporary edit target:

- `crates\tessara-web-organization\src\lib.rs`
- behavior-neutral private marker comment

Observed rebuilds:

| Edit | Detection | Elapsed evidence |
| --- | --- | ---: |
| marker edit | pass; watcher compiled `tessara-web-organization` and root `tessara-web`, ran wasm-bindgen, and reported `Watch updated Front` | compile 9.55s + wasm-bindgen 3.69s |
| cleanup | pass; watcher compiled `tessara-web-organization` and root `tessara-web`, ran wasm-bindgen, and reported `Watch updated Front` | compile 9.05s + wasm-bindgen 4.04s |

Manual authenticated route smoke:

- started Postgres with a fresh local volume;
- ran `tessara-api` on the host;
- `/health` returned `ok`;
- `POST /api/auth/login` with local dev admin credentials returned 200;
- `POST /api/demo/seed` returned 200 with `seed_version = uat-demo-v1`;
- authenticated `GET /organization` returned 200;
- authenticated `GET /organization/new` returned 200;
- authenticated `GET /administration` returned 200;
- authenticated `GET /administration/users` returned 200;
- authenticated `GET /administration/roles` returned 200;
- authenticated `GET /administration/node-types` returned 200;
- unauthenticated `GET /organization` returned 303 to login.

General smoke script note:

- `scripts\smoke.ps1` was attempted, but failed before server startup because `cargo test -p tessara-api --test demo_flow` has two unrelated dataset failures.
- The failing tests were `dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence` and `admin_dataset_query_designer_materializes_generated_sql`.
- The narrower route smoke above bypassed that preflight and verified the Organization/Admin web routes touched by this extraction.

Cleanup:

- temporary marker was removed;
- cleanup rebuild passed;
- watcher process was stopped;
- the Postgres container started for smoke/watch validation was stopped.

## Bundle Size

Current post-extraction bundle from `target\site\pkg`:

| Artifact | Bytes |
| --- | ---: |
| wasm total | 31,021,977 |
| js total | 60,563 |
| combined JS/WASM | 31,082,540 |
| css total | 120,731 |

Comparison:

| Baseline | Combined JS/WASM | Delta |
| --- | ---: | ---: |
| Responses post-extraction bundle | 30,162,754 | +919,786 / +3.0% |
| original dataset pilot baseline | 27,663,145 | +3,419,395 / +12.4% |

Immediate bundle growth from the Responses post-extraction bundle remained below the 5% gate. Cumulative growth since the dataset pilot remains above 5%, so bundle size should remain a standing gate for future large Leptos extractions.

## Retained Debt

- Organization keeps web DTOs separate from API DTOs.
- Organization owns local browser transport and unauthorized redirect handling.
- Organization owns local metadata, text, pagination, and URL helpers copied from prior root patterns.
- Administration owns local role/node-type/metadata DTO copies while Administration extraction remains deferred.
- Administration still reuses some `organization-*` CSS class names; that is styling debt, not a crate-boundary dependency.
- No shared web platform crate was created.
- No Administration extraction or Administration sub-crate split was attempted.

## GO Decision

Decision: GO.

Evidence:

- Organization-local hydrate check passes;
- Organization-local SSR check passes;
- Organization-local test compile passes;
- Organization-local docs build passes;
- root hydrate check passes;
- API SSR check passes;
- `cargo leptos build` passes;
- `cargo leptos watch` detects extracted Organization crate edits;
- authenticated Organization and Administration routes return HTTP 200 after demo seed;
- unauthenticated Organization route redirect behavior is preserved;
- boundary checker passes with no permanent exceptions;
- public API is the approved route-content facade;
- root retains route/shell/hydration/CSS/assets ownership;
- no API/web DTO convergence was required;
- no sibling web feature crate dependency was introduced;
- immediate bundle growth is under 5%.

Follow-up:

- Keep Administration deferred until a future sprint can split it intentionally.
- Treat cumulative bundle growth as a measurement item before another large frontend extraction.
- Explore Leptos lazy loading/code splitting as future bundle work. Use a focused pilot with `cargo leptos build --release --split` and `#[lazy]`/`#[lazy_route]`; do not assume Cargo feature-crate boundaries become lazy chunks automatically.
- Do not create a shared web platform crate yet; reassess only if the remaining root features repeat enough helper duplication to create real maintenance drag.

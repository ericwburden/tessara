# Tessara Web Responses-Crate Extraction Results

Status: GO

Governing plan:

- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-responses-extraction-proposal.md`
- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-feature-crate-roadmap.md`

## Measured Environment

- worktree: `C:\Users\eric-dev\Projects\tessara-refactoring`
- branch: `codex/refactoring-work`
- base commit before this working-tree implementation: `1bb234cc8458d1c4a4d991acf6b551be0b8e7b3f`
- rustc: `1.95.0 (59807616e 2026-04-14)`
- Cargo: `1.95.0 (f2d3ce0bd 2026-03-21)`
- cargo-leptos: `0.3.6`
- raw evidence:
  - clean-target timings: `tmp\web-responses-extraction\R3-clean-current`
  - watch logs and route smoke: `tmp\web-responses-extraction\R3-watch`
  - generated docs: `target\doc\tessara_web_responses\index.html`

## Baseline Inventory

The R0/R1 inventory found and resolved the expected Responses blockers before extraction:

- root-owned `AppShell`, `SubmissionRouteParams`, and route-param parsing stayed in `crates\tessara-web\src\routes\responses.rs`;
- Responses content stopped importing root route, router, shell, session, or metadata concerns;
- Home stopped importing Responses internals by owning its local pending-work start action;
- small root HTTP, metadata, text, filtering, pagination, URL, and status helpers were copied locally;
- Responses owns local web DTOs, loaders, actions, display helpers, value collection, and browser transport.

No compatibility `crate::features::responses` namespace was retained.

## Extraction Summary

Implemented architecture:

- `crates\tessara-web-responses` owns response list, start, detail, edit content, loaders, actions, DTOs, display helpers, value collection, browser transport, and feature-local support helpers.
- root `crates\tessara-web\src\routes\responses.rs` owns route registration, `SubmissionRouteParams`, auth guard, and `AppShell` wrapping.
- root `crates\tessara-web\src\features\mod.rs` no longer exposes a `responses` feature module.
- Home owns its pending-work start action and does not depend on the Responses crate.
- `tessara-web-responses` public facade exports only:
  - `ResponsesIndexContent`
  - `ResponseStartContent`
  - `ResponseDetailContent`
  - `ResponseEditContent`

## Command Matrix

Clean-target comparison, using isolated target directories under `tmp\web-responses-extraction\R3-clean-current`:

| Probe | Original clean baseline | Post-Workflows clean target | Current Responses clean target | Delta vs original |
| --- | ---: | ---: | ---: | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 247.39s | 115.00s | -190.92s / -62.4% |
| `cargo check -p tessara-api --features ssr` | 392.17s | 337.40s | 195.49s | -196.68s / -50.2% |
| `cargo check -p tessara-web-responses --no-default-features --features ssr` | n/a | n/a | 83.09s | focused loop available |
| `cargo check -p tessara-web-responses --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | n/a | 98.05s | focused loop available |
| `cargo test -p tessara-web-responses --no-default-features --features ssr --no-run -j 1` | n/a | n/a | 290.76s | focused test compile available |

Final active-target gates also passed:

- `cargo fmt --all --check`
- `cargo check -p tessara-web-responses --no-default-features --features hydrate --target wasm32-unknown-unknown`
- `cargo check -p tessara-web-responses --no-default-features --features ssr`
- `cargo test -p tessara-web-responses --no-default-features --features ssr --no-run -j 1`
- `cargo test -p tessara-web-responses --no-default-features --features ssr`: pass, 0 tests
- `cargo doc -p tessara-web-responses --no-deps`
- `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown`
- `cargo check -p tessara-api --features ssr`
- `cargo leptos build`: pass; final active-target build completed in 76.8s after prior verification warmed the configured `target/front` path

Known local `cargo leptos build` warnings remain unchanged:

- `node_modules` is not installed, so cargo-leptos continues without it.
- `crates/tessara-web/public` does not exist, so asset copying is skipped.

## Boundary Gates

`scripts\check-web-crate-boundaries.ps1` now checks `tessara-web-responses` for both `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown`.

It rejects dependency paths from `tessara-web-responses` to:

- `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates except `tessara-web-ui`
- `leptos_router`
- `leptos_meta`

It also audits source for route/shell/router/meta/old-namespace/sibling-feature references.

Result:

- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1`: pass
- old Responses namespace audit: no root compatibility module retained
- extracted crate route/shell/router/meta/sibling-feature audit: no matches
- root route adapter imports only the approved Responses content facade

The new crate keeps browser transport dependencies non-optional, matching the current Forms and Workflows extraction pattern. SSR compilation passes with this manifest, and the result avoids router/meta/root dependencies.

## Watch-Mode Results

Command:

```powershell
cargo leptos watch
```

Initial watch build:

- compiled `tessara-web-responses` for front and server paths;
- served at `http://127.0.0.1:8080`;
- `/health` returned 200;
- known local warnings remained: missing `node_modules`, missing `crates/tessara-web/public`.

Temporary edit target:

- `crates\tessara-web-responses\src\lib.rs`
- behavior-neutral private const marker

Observed rebuilds:

| Edit | Detection | Elapsed evidence |
| --- | --- | ---: |
| `responses-edit-2` | pass; watcher compiled `tessara-web-responses` and root `tessara-web`, ran wasm-bindgen, and reported `Watch updated Front` | compile 10.39s + wasm-bindgen 4.51s |
| cleanup | pass; watcher compiled `tessara-web-responses` and root `tessara-web`, ran wasm-bindgen, and reported `Watch updated Front` | compile 10.14s + wasm-bindgen 3.88s |

Authenticated route smoke under the running watcher:

- `POST /api/auth/login` with local dev admin credentials returned 200.
- `POST /api/demo/seed` returned 200 with `seed_version = uat-demo-v1`.
- `GET /api/submissions` returned 200 with 12 submissions.
- authenticated `GET /` returned 200.
- authenticated `GET /responses` returned 200.
- authenticated `GET /responses/new` returned 200.
- authenticated `GET /responses/65d61b91-7c6c-46ca-9c8e-9a018472e679` returned 200.
- authenticated `GET /responses/65d61b91-7c6c-46ca-9c8e-9a018472e679/edit` returned 200.
- direct `GET /responses/new?workflowAssignmentId=d553d50a-3a67-43c7-b3e5-185bc021dc20` returned 200.
- `POST /api/workflow-assignments/d553d50a-3a67-43c7-b3e5-185bc021dc20/start` returned 200 with response id `206f0e36-fe7b-49e1-a110-0b453a8979c9`.
- authenticated `GET /responses/206f0e36-fe7b-49e1-a110-0b453a8979c9/edit` returned 200.

Home pending-work note:

- authenticated Home returned 200;
- the admin session had zero pending work after seed, so the Home button itself was not clicked;
- the same local Home-owned start endpoint path was verified by the assignment-start proxy above.

Cleanup:

- temporary marker was removed;
- cleanup rebuild passed;
- watcher process was stopped;
- no Cargo/Rust/API watcher processes remained.

## Bundle Size

Current post-extraction bundle from `target\site`:

| Artifact | Bytes |
| --- | ---: |
| wasm total | 30,102,191 |
| js total | 60,563 |
| combined JS/WASM | 30,162,754 |
| css total | 120,731 |
| total site files | 30,290,678 |

Comparison:

| Baseline | Combined JS/WASM | Delta |
| --- | ---: | ---: |
| Workflows post-extraction bundle | 29,570,199 | +592,555 / +2.0% |
| Forms post-extraction bundle | 28,653,760 | +1,508,994 / +5.3% |
| dataset post-extraction bundle | 27,788,530 | +2,374,224 / +8.5% |
| original dataset pilot baseline | 27,663,145 | +2,499,609 / +9.0% |

The immediate delta from the prior Workflows result remains below 5%. The cumulative delta since the dataset pilot remains above 5%, so bundle size should continue to be monitored before extracting another large Leptos surface.

## Retained Debt

- Responses keeps web DTOs separate from API DTOs.
- Responses owns local browser transport and unauthorized redirect handling.
- Responses owns local metadata, text, filtering, pagination, URL, and status helpers copied from prior root patterns.
- Home owns a separate pending-work start action that targets the same assignment-start endpoint.
- Root owns routes, route params, auth guard, shell, hydration, document, CSS/assets, and cargo-leptos app wiring.
- No shared web platform crate was created.
- No response/form/workflow contract convergence was attempted.

## GO Decision

Decision: GO.

Evidence:

- response-local hydrate check passes
- response-local SSR check passes
- response-local test compile passes
- response-local test execution passes with zero tests
- root hydrate check passes
- API SSR check passes
- `cargo leptos build` passes
- `cargo leptos watch` detects extracted Responses crate edits
- authenticated Responses list, start, detail, and edit routes return HTTP 200 under watch
- direct assignment-start route and endpoint path remain functional
- boundary checker passes with no permanent exceptions
- public API is the approved route-content facade
- root retains route/shell/hydration/CSS/assets ownership
- no API/web DTO convergence was required
- no sibling web feature crate dependency was introduced
- immediate bundle growth is under 5%

Follow-up:

- Treat cumulative bundle growth as a measurement item for the Organization proposal.
- Keep Administration deferred until Organization/node DTO ownership is settled.
- Do not create a shared web platform crate yet; reassess only if Organization repeats enough local helper duplication to create real maintenance drag.

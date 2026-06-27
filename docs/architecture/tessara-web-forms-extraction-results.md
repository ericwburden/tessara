# Tessara Web Forms-Crate Extraction Results

Status: GO, with warm-cache timing caveat

Governing plan:

- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-forms-extraction-proposal.md`
- `C:\Users\eric-dev\Projects\tessara-refactoring\docs\architecture\tessara-web-feature-crate-roadmap.md`

## Measured Environment

- worktree: `C:\Users\eric-dev\Projects\tessara-refactoring`
- branch: `codex/refactoring-work`
- extraction commit: `6f8cb873f95341e8a31cfed8a28217d6c9c9d0bf`
- rustc: `1.95.0 (59807616e 2026-04-14)`
- Cargo: `1.95.0 (f2d3ce0bd 2026-03-21)`
- cargo-leptos: `0.3.6`
- OS/CPU/RAM/power plan: same local Windows workstation as the dataset pilot results
- raw evidence:
  - baseline inventory: `tmp\web-forms-extraction\F0-inventory`
  - baseline command logs: `tmp\web-forms-extraction\F0-baseline`
  - F5 command logs: `tmp\web-forms-extraction\F5-results`
  - F5 watch logs: `tmp\web-forms-extraction\F5-watch`

## Baseline Inventory

F0 captured the forms import inventory before extraction:

- root UI imports
- root utility imports
- root HTTP imports
- root route/state/type references
- sibling feature imports
- hydrate/browser API references
- tests and CSS class usage

The extraction resolved the blocking forms dependencies before the move:

- `NodeTypeCatalogEntry` became forms-local `FormNodeTypeOption`.
- workflow display helpers became forms-local display helpers.
- shared attached-node/link DTOs became forms-local DTOs.
- tiny root utility helpers became forms-local support helpers.
- root HTTP helpers became forms-local transport helpers.

## Extraction Summary

Implemented commits:

- `e9843f4 Move forms shared UI into web UI crate`
- `fd01fbd Prepare forms feature for crate extraction`
- `308fd1d Prepare forms route adapters`
- `150bf5e Decouple workflow and response form contracts`
- `6f8cb87 Extract web forms crate`

Current architecture:

- `crates\tessara-web-forms` owns Forms content, builder, loaders, DTOs, transport, save orchestration, and display helpers.
- root `crates\tessara-web\src\routes\forms.rs` owns route registration, `FormRouteParams`, and `AppShell` wrapping.
- root `crates\tessara-web\src\features\mod.rs` no longer exposes a `forms` feature module.
- `tessara-web-forms` public facade exports only `FormsIndexContent`, `FormNewContent`, `FormDetailContent`, and `FormEditContent`, plus unavoidable generated Leptos props types in rustdoc.

## Command Matrix

F0 baseline used isolated target directories. F5 timings below are warm-cache measurements from the active workspace, so they are useful as loop evidence but not a strict apples-to-apples regression comparison.

| Probe | Baseline | Current | Result |
| --- | ---: | ---: | --- |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 1.17s warm | pass |
| `cargo check -p tessara-api --features ssr` | 392.17s | 2.50s warm | pass |
| `cargo leptos build` | `>900s` timeout in F0 wrapper | 203.1s pre-commit, 9.84s warm | pass |
| `cargo check -p tessara-web-forms --no-default-features --features ssr` | n/a | 1.15s warm | pass |
| `cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | 1.21s warm | pass |
| `cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1` | n/a | 1.15s warm | pass |

`cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1` originally hit Rust's default Leptos view recursion limit. Adding `#![recursion_limit = "512"]` to `tessara-web-forms` matches the existing root web crate treatment and the gate now passes.

Follow-up clean-target comparison captured after the GO call in `tmp\pilot-targets\forms-current-compile-compare`:

| Probe | Original baseline | Current clean target | Delta |
| --- | ---: | ---: | ---: |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | 305.92s | 270.47s | -35.45s / -11.6% |
| `cargo check -p tessara-api --features ssr` | 392.17s | 392.15s | -0.02s / flat |
| `cargo leptos build` | `>900s` timeout | 799.46s | completed, at least 100.54s under timeout |
| `cargo check -p tessara-web-forms --no-default-features --features ssr` | n/a | 114.87s | focused loop available |
| `cargo check -p tessara-web-forms --no-default-features --features hydrate --target wasm32-unknown-unknown` | n/a | 105.48s | focused loop available |
| `cargo test -p tessara-web-forms --no-default-features --features ssr --no-run -j 1` | n/a | 336.19s | focused test compile available |

## Boundary Gates

`scripts\check-web-crate-boundaries.ps1` now checks `tessara-web-forms` for both `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown`.

It rejects dependency paths from `tessara-web-forms` to:

- `tessara-web`
- `tessara-api`
- sibling `tessara-web-*` feature crates except `tessara-web-ui`
- `leptos_router`
- `leptos_meta`

It also audits source for route/shell/router/meta/old-namespace/sibling-feature references.

Result:

- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check-web-crate-boundaries.ps1`: pass, 6.44s
- old forms namespace audit: no matches in `crates\tessara-web-forms\src`
- root route facade import audit: root routes import the four approved content components

## Watch-Mode Results

Command:

```powershell
cargo leptos watch
```

Initial watch build:

- compiled `tessara-web-forms` for front and server paths
- served at `http://127.0.0.1:8080`
- known local warnings remained: missing `node_modules`, missing `crates/tessara-web/public`

Temporary edit target:

- `crates\tessara-web-forms\src\editor_sections.rs`
- temporary `data-pilot-benchmark="forms-edit-N"` on the existing `form-grid` element

Observed rebuilds:

| Edit | Detection | Health | Elapsed |
| --- | --- | --- | ---: |
| `forms-edit-1` | pass in raw log; parser missed ANSI marker | not checked by parser | compile 23.80s + wasm-bindgen 6.68s |
| `forms-edit-2` | pass | `/health` 200 | 18.60s |
| `forms-edit-3` | pass | `/health` 200 | 12.35s |
| cleanup | pass | `/health` 200 | 20.50s |

Authenticated route smoke under the running watcher:

- `POST /api/auth/login` with local dev admin credentials returned 200.
- authenticated `GET /forms` returned 200.
- authenticated `GET /forms/new` returned 200.
- unauthenticated `GET /forms` and `/forms/new` returned 303 to login, preserving protected-route behavior.

Follow-up detail/edit smoke with seeded demo data:

- `POST /api/auth/login` returned 200.
- `POST /api/demo/seed` returned 200.
- seeded form id: `c9a53d26-2311-4b0c-bafd-026de3d5b7fd`.
- authenticated `GET /forms` returned 200.
- authenticated `GET /forms/new` returned 200.
- authenticated `GET /forms/c9a53d26-2311-4b0c-bafd-026de3d5b7fd` returned 200.
- authenticated `GET /forms/c9a53d26-2311-4b0c-bafd-026de3d5b7fd/edit` returned 200.

Cleanup:

- watcher process tree was stopped.
- benchmark audit for `data-pilot-benchmark|forms-edit-[123]` returned no matches.

## Bundle Size

Current post-extraction bundle from `target\site`:

| Artifact | Bytes |
| --- | ---: |
| wasm total | 28,593,197 |
| js total | 60,563 |
| combined JS/WASM | 28,653,760 |
| css total | 120,731 |

Comparison:

| Baseline | Combined JS/WASM | Delta |
| --- | ---: | ---: |
| dataset post-extraction bundle | 27,788,530 | +865,230 / +3.1% |
| original dataset pilot baseline | 27,663,145 | +990,615 / +3.6% |

Bundle growth is below the 5% explanation threshold.

## Behavior Test Notes

Additional behavior gates:

- `cargo test -p tessara-web-forms --no-default-features --features ssr`: pass, 0 tests, 22.87s compile.
- `cargo test -p tessara-web --lib -j 1`: fails during Windows `link.exe`, after compilation, with unresolved Leptos/Tachys symbols and `warning LNK4003: invalid library format; library ignored` for `target\debug\deps\libtessara_web_datasets-4784e249a804efe4.rlib`.

The root web lib test-link failure is recorded as root integration/test-link status, not as a forms-local failure:

- forms-local test compilation and full forms-local test execution both pass;
- root hydrate, API SSR, and `cargo leptos build` all pass;
- browser-facing SSR route smoke passes for list, create, detail, and edit routes.

## GO Decision

Decision: GO.

Evidence:

- forms-local hydrate check passes
- forms-local SSR check passes
- forms-local test compile passes
- root hydrate check passes
- API SSR check passes
- `cargo leptos build` passes
- `cargo leptos watch` detects extracted forms crate edits
- authenticated `/forms` and `/forms/new` return HTTP 200 under watch
- boundary checker passes with no exceptions
- public API is the approved route-content facade
- root retains route/shell/hydration/CSS/assets ownership
- no API/web DTO convergence was required
- no sibling web feature crate dependency was introduced
- bundle growth is under 5%

Follow-up:

- Re-run clean-target compile comparisons if the next roadmap decision depends on exact regression percentages.
- Use the forms extraction as the reference model before considering Workflows or Responses extraction.

# Tessara Web Dataset-Crate Pilot Results

Status: Commit 3 dataset crate extraction validated

Governing plan:

- `C:\Users\eric-dev\Projects\tessara\docs\architecture\tessara-web-refactoring-plan.md`

## Measured Environment

Commit 0 inventory was captured from:

- worktree: `C:\Users\eric-dev\Projects\tessara-refactoring`
- git SHA: `060abc08d91c355310695b6913f4179143238dfc`
- branch: `codex/refactoring-work`
- rustc: `1.95.0 (59807616e 2026-04-14)`
- Cargo: `1.95.0 (f2d3ce0bd 2026-03-21)`
- cargo-leptos: `0.3.6`
- OS: Microsoft Windows 11 Pro, build `26200`, 64-bit
- CPU: Intel Core i7-9700, 8 cores / 8 logical processors
- RAM: 34,123,059,200 bytes
- power plan: Balanced
- cache policy: `RUSTC_WRAPPER` removed, `SCCACHE_DISABLE=1`, `CARGO_INCREMENTAL=1`

Raw local evidence:

- inventory: `tmp\web-refactor-pilot\commit-0-20260626-221854`
- baseline probes: `tmp\web-refactor-pilot\commit-0-baseline-20260626-221917`
- bundle report: `tmp\web-refactor-pilot\commit-0-bundle-20260626-224632`
- pilot target directories: `tmp\pilot-targets\commit-0-baseline`

The raw directories are intentionally under ignored `tmp` paths. Durable results are summarized here.

## Commit 0 Inventory

The inventory harness recorded:

- `git rev-parse HEAD`
- `git status --short`
- `rustc -Vv`
- `cargo -V`
- `cargo leptos --version`
- `cargo metadata --format-version 1 --no-deps`
- `cargo tree -p tessara-web -e features --depth 1 --features ssr --color never`
- `cargo tree -p tessara-web -e features --depth 1 --no-default-features --features hydrate --target wasm32-unknown-unknown --color never`
- `cargo tree -p tessara-api -e features --depth 1 --features ssr --color never`

Inventory command outputs completed successfully.

## Baseline Timing

These baseline probes used isolated pilot-owned target directories under `tmp\pilot-targets\commit-0-baseline`.

| Probe | Result | Elapsed | Notes |
| --- | --- | ---: | --- |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass | 2m08s | clean pilot target |
| `cargo check -p tessara-api --features ssr` | pass | 3m05s | clean pilot target |
| `cargo leptos build` | pass | 6m30s | clean pilot target; `target/site` removed before build |
| `cargo test -p tessara-web --lib --no-run -j 1` | inconclusive | exceeded 600s | root test-link probe exceeded the plan's fixed timeout window |

The root test-link metric is not used as a claimed improvement baseline. The plan allows this metric to remain inconclusive when the same timeout behavior persists and no new linker failure mode appears. Feature-crate test targets must still complete after extraction.

## Watch-Mode Results

Commit 1 UI watch gate:

- command: `cargo leptos watch`
- source edit: temporary `data-pilot-benchmark="ui-edit-1"` attribute in `crates\tessara-web-ui\src\data_table.rs`
- result: pass for path-dependency detection and front rebuild
- observed initial watch build: `tessara-web-ui` compiled for both front/server paths, then cargo-leptos served at `http://127.0.0.1:8080`
- observed edit rebuild: `tessara-web-ui` and `tessara-web` rebuilt in 14.86s, wasm-bindgen finished in 3.96s, and watch reported `Watch updated Front`
- observed revert rebuild: `tessara-web-ui` and `tessara-web` rebuilt in 14.31s, wasm-bindgen finished in 3.26s, and watch reported `Watch updated Front`
- limitation: browser refresh timing was not separately measured because the local API emitted `migration 1 was previously applied but has been modified` after serving. The hard path-dependency detection gate passed; full browser-refresh timing remains to capture when the local DB migration state is repaired.

Dataset watch gate is not run yet. Required after Commit 3 for `tessara-web-datasets`.

## Feature-Tree Deltas

Commit 0 captured current root feature trees.

Commit 1 added `tessara-web-ui`. Feature-tree review:

- `cargo tree -p tessara-web-ui -e features --depth 2 --color never` passed.
- Direct UI crate dependencies are limited to `leptos` and `icons`.
- No root app, API, router, meta, transport, auth, session, navigation, or product feature crate dependency is declared by `tessara-web-ui`.
- Source audit for `datasets|forms|workflows|responses|organization|administration|AppShell|ShellSession|require_authenticated_route` under `crates\tessara-web-ui\src` found no matches.

Commit 2 kept the dataset feature in `tessara-web` but moved route-only concerns back to the root route adapter:

- `crates\tessara-web\src\routes\datasets.rs` now owns dataset route page adapters, route parameter parsing, shell wrapping, and the explicit preview auth guard.
- `crates\tessara-web\src\features\datasets\pages` now exposes shell-free content components: `DatasetsIndexContent`, `DatasetDetailContent`, `DatasetEditorContent`, and `DatasetPreviewContent`.
- Source audit for `AppShell|require_route_params|DatasetRouteParams|Datasets(Page|DetailPage|EditPage|NewPage|PreviewPage)` under `crates\tessara-web\src\features\datasets` found no matches. Those route/shell references remain root-owned in `crates\tessara-web\src\routes\datasets.rs`.
- Dataset-internal absolute imports such as `crate::features::datasets::types` remain and are tracked for the extraction/import rewrite commit rather than preserved for compatibility.

Commit 3 extracted `crates\tessara-web-datasets`:

- `crates\tessara-web\src\features\datasets` was removed; dataset code now lives under `crates\tessara-web-datasets\src`.
- Root dataset routes import the four content components directly from `tessara_web_datasets`.
- Dataset route params, shell wrappers, and auth guard remain root-owned in `crates\tessara-web\src\routes\datasets.rs`.
- Dataset-internal `crate::features::datasets::*` imports and `pub(in crate::features::datasets)` visibility were rewritten to crate-local paths and `pub(crate)`.
- `tessara-web-datasets` owns small local `http`, `pagination`, and `text` helper modules instead of depending on root `tessara-web` utilities.
- `DraggablePanelList` moved into `tessara-web-ui` because the dataset editor needs it and it is domain-neutral UI behavior.
- Source audit for root app/router/shell/auth references under `crates\tessara-web-datasets\src` found no matches except `crate::http`, which is the new crate-local transport module.

## Bundle-Size Deltas

Commit 0 bundle report, from the `cargo leptos build` output in worktree-local `target/site`:

| Artifact | Bytes | LastWriteTimeUtc |
| --- | ---: | --- |
| `tessara-web.wasm` | 27,602,583 | 2026-06-27T02:29:14Z |
| `tessara-web.js` | 60,562 | 2026-06-27T02:29:14Z |
| combined JS/WASM | 27,663,145 |  |
| `tessara-web.css` | 120,731 | 2026-06-27T02:24:36Z |

Commit 3 bundle report, from the post-extraction `cargo leptos build` output in worktree-local `target/site`:

| Artifact | Bytes | Delta vs Commit 0 |
| --- | ---: | ---: |
| `tessara-web.wasm` | 27,727,967 | +125,384 |
| `tessara-web.js` | 60,563 | +1 |
| combined JS/WASM | 27,788,530 | +125,385 |
| `tessara-web.css` | 120,731 | 0 |

## Public API Review

Commit 1 UI public API:

- `cargo doc -p tessara-web-ui --no-deps` passed.
- Public facade is limited to `Breadcrumb`, breadcrumb item/link/page/separator primitives, `Combobox`, `ComboboxOption`, `DataTable`, `EmptyState`, `PageHeader`, `SegmentedToggle`, `SegmentedToggleOption`, and `TablePaginationFooter`.
- `SearchableDataTable` remains root-local in `tessara-web` and composes the moved `DataTable`.
- Shell, auth, navigation, browser transport, and product concepts remain root-owned.

Dataset public API review is not applicable yet.

Commit 2 dataset feature facade:

- The root-facing dataset boundary now re-exports content components rather than route pages.
- `/datasets/:dataset_id/preview` remains intentionally shell-less, with `require_authenticated_route("datasets")` called by the route adapter before rendering `DatasetPreviewContent`.

Commit 3 dataset crate public API:

- `cargo doc -p tessara-web-datasets --no-deps` passed.
- Public API is limited to `DatasetsIndexContent`, `DatasetDetailContent`, `DatasetEditorContent`, and `DatasetPreviewContent`.
- Dataset DTOs, loaders, editor state, transport, validation, and helper modules remain crate-private.

## Dependency-Boundary Report

Not applicable yet. Boundary tooling is planned for Commit 4.

## Behavior Validation Results

Commit 1 validation:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | pass |
| `cargo check -p tessara-web-ui` | pass |
| `cargo tree -p tessara-web-ui -e features --depth 2 --color never` | pass |
| `cargo doc -p tessara-web-ui --no-deps` | pass |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass |
| `cargo check -p tessara-api --features ssr` | pass |
| `cargo leptos build` | pass |

Commit 2 validation:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | pass |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass, 21.89s |
| `cargo check -p tessara-api --features ssr` | pass, 27.31s |
| `cargo leptos build` | pass, 1m35s; existing missing `node_modules` and missing `crates/tessara-web/public` warnings only |

Commit 3 validation:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | pass |
| `cargo check -p tessara-web-datasets --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass, 7.93s |
| `cargo check -p tessara-web-datasets --features ssr` | pass, 47.64s |
| `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown` | pass, 4.60s after stale root re-export cleanup |
| `cargo check -p tessara-api --features ssr` | pass, 37.06s |
| `cargo test -p tessara-web-datasets --lib` | pass, 21 tests |
| `cargo test -p tessara-web-ui --lib` | pass, 3 tests |
| `cargo tree -p tessara-web-datasets -e features --depth 2 --color never` | pass |
| `cargo doc -p tessara-web-datasets --no-deps` | pass |
| `cargo leptos build` | pass, 1m43s; existing missing `node_modules` and missing `crates/tessara-web/public` warnings only |

## Intentional Contract and Transport Debt Retained

Commit 3 retained intentional local debt:

- `tessara-web-datasets` has its own browser HTTP transport copied from the root app transport shape. This avoids a root-app dependency during extraction; Commit 4 should enforce that no feature crate imports root transport.
- Minimal text and pagination helpers were copied into the dataset crate rather than preserved through root `utils` imports. If more feature crates need them, promote them to a small shared utility crate instead of reintroducing root coupling.
- `DraggablePanelList` moved into `tessara-web-ui` with direct `js-sys`, `wasm-bindgen`, and `web-sys` dependencies because the primitive itself owns browser drag behavior.

## GO/PARTIAL/NO-GO Decision

No final decision yet. The pilot is validated through Commit 3 and still needs automated boundary enforcement plus the post-extraction dataset watch gate.

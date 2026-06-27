# Tessara Web Dataset-Crate Pilot Results

Status: Commit 0 baseline in progress

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

Not run yet. Required after Commit 1 for `tessara-web-ui` and after Commit 3 for `tessara-web-datasets`.

## Feature-Tree Deltas

Commit 0 captured current root feature trees. No new crates exist yet.

## Bundle-Size Deltas

Commit 0 bundle report, from the `cargo leptos build` output in worktree-local `target/site`:

| Artifact | Bytes | LastWriteTimeUtc |
| --- | ---: | --- |
| `tessara-web.wasm` | 27,602,583 | 2026-06-27T02:29:14Z |
| `tessara-web.js` | 60,562 | 2026-06-27T02:29:14Z |
| combined JS/WASM | 27,663,145 |  |
| `tessara-web.css` | 120,731 | 2026-06-27T02:24:36Z |

## Public API Review

Not applicable yet. Required after `tessara-web-ui` and `tessara-web-datasets` exist.

## Dependency-Boundary Report

Not applicable yet. Boundary tooling is planned for Commit 4.

## Behavior Validation Results

Not run yet for structural phases. Commit 0 made no source architecture changes.

## Intentional Contract and Transport Debt Retained

Not applicable yet. Required after dataset extraction.

## GO/PARTIAL/NO-GO Decision

No final decision yet. The pilot is still in Commit 0 baseline setup.

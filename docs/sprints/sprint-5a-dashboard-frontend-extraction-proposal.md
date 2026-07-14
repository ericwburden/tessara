# Sprint 5A Dashboard Frontend Extraction Decision

- **Decision:** APPROVE `tessara-web-dashboards`
- **Decision date:** 2026-07-12
- **Evidence snapshot:** working tree rooted at `a09a17c`
- **Applies to:** Sprint 5A Dashboard directory, create, detail, editor, and focused viewer work

This is an approval, not a request to revisit the ownership choice after implementation has started. The implementation gates below validate the approved boundary and provide an objective rollback path; they do not defer the decision.

The approval has one mandatory dependency condition: `tessara-web-dashboards` must not depend on `tessara-web-components`. The reusable explicit-version Component execution/rendering surface must first be moved into a smaller shared presentation leaf, named `tessara-web-component-viewer` in this decision. Both Dashboard and Component feature crates may depend on that leaf. This is the narrow architecture amendment approved by this record.

## Decision Summary

Approve the extraction before the first Dashboard vertical slice because the current root Dashboard implementation is still only a placeholder, while the accepted Sprint 5A scope will add five distinct surfaces, feature-local transport, an interactive editor, an execution-aware viewer, redaction states, and request-scoped SSR data. Establishing the intended owner now avoids moving that implementation after it has accumulated significant churn.

Keep all route and application integration in root `tessara-web`. The new feature crate owns Dashboard content and web behavior only. Introduce a leaf Component viewer instead of either duplicating rendering or pulling Component authoring dependencies into Dashboard.

The approved dependency direction is:

```text
tessara-api (SSR build only)
        |
        v
tessara-web (routes, AppShell, session/navigation, document, hydration)
        |                                      |
        v                                      v
tessara-web-dashboards                 tessara-web-components
        |                                      | \
        |                                      |  -> tessara-web-data-ops
        +----------> tessara-web-component-viewer
                              |
                              v
                       tessara-web-ui

tessara-web-dashboards -> tessara-web-components       forbidden
tessara-web-dashboards -> tessara-web-data-ops         forbidden, including transitively
feature/viewer crates -> tessara-web or tessara-api    forbidden
```

Pure grid and Dashboard policy remain below the web layer: generic grid primitives belong in `tessara-core`, Dashboard composition rules belong in `tessara-dashboards`, and shared Leptos placement interactions belong in `tessara-web-ui`.

## Evidence

### Current size and churn

The following pre-implementation inventory was captured at the start of this decision work. It counts Rust source files and physical lines under each `src` directory. Churn is the full `git log --numstat` history touching the current path; it is a useful relative signal but a lower bound for code that moved without rename tracking. Cargo manifests are not included in the source counts.

| Area | Current Rust files | Current Rust lines | Commits touching path | Added / deleted lines |
| --- | ---: | ---: | ---: | ---: |
| Root Dashboard content, `tessara-web/src/features/dashboards` | 2 | 36 | 3 | +42 / -6 |
| Root Dashboard route adapter, `tessara-web/src/routes/dashboards.rs` | 1 | 40 | 6 | +46 / -6 |
| `tessara-web-components` | 8 | 5,838 | 24 | +8,883 / -3,021 |
| `tessara-web-datasets` | 46 | 10,101 | 18 | +12,515 / -2,388 |
| `tessara-web-forms` | 74 | 6,714 | 1 | +6,742 / -0 |
| `tessara-web-workflows` | 76 | 6,511 | 1 | +6,539 / -0 |
| `tessara-web-responses` | 34 | 2,722 | 1 | +2,750 / -0 |
| `tessara-web-organization` | 39 | 3,298 | 1 | +3,326 / -0 |
| `tessara-web-ui` | 17 | 1,926 | 8 | +1,957 / -10 |

The current Dashboard content is not large enough to justify extraction by size alone. Its two files are a 9-line module facade and 27 lines of placeholder pages, and its recent history is structural rather than product implementation. That is precisely why the move is inexpensive now. The accepted Sprint 5A plan requires directory, create, detail, editor, and viewer content plus DTOs, loaders/actions, optimistic and error states, redaction, viewport-aware execution, and SSR bootstrap consumption. Even the smallest established extracted feature, Responses, is already 2,722 Rust lines across 34 files. Waiting until the Dashboard scope is implemented would create a larger, riskier ownership move without improving the boundary decision.

The root route adapter is already a separate 40-line file. Comparable extracted feature routes show that the project has a stable pattern: root routes stay small while content modules grow independently in feature crates.

### Prior extraction results

The repository history contains measured evidence that this pattern works in the current toolchain:

- The Forms extraction result at commit `47b5726` recorded green SSR/hydrate checks, detected watch edits, warm rebuilds of approximately 12.35 to 20.5 seconds with one 23.8-second run, and unchanged route behavior.
- Its post-extraction combined JS/WASM result was 28,653,760 bytes, 990,615 bytes or 3.6% above the original Dataset baseline of 27,663,145 bytes. The result remained below the historical 5% explanation threshold.
- The Dataset pilot baseline at commit `bf4bb6d` established the clean root hydrate, API SSR, full Leptos build, browser, and bundle-reporting method now implemented by `scripts/web-refactor-pilot.ps1`.

These results do not prove that any new crate is free. They do show that the root-adapter/content-crate model, Cargo Leptos feature forwarding, watch discovery, and bundle comparison are established and measurable in this repository.

### Dependency review

`tessara-web-components` is not a safe dependency for the Dashboard crate in its current form:

- Its only Cargo features are `hydrate` and `ssr`; there is no viewer/authoring partition.
- It directly depends on `tessara-web-data-ops`. `cargo tree -p tessara-web-components --depth 2 -e normal --no-default-features` confirms that edge.
- Its public facade exports index, editor, versions, and viewer content together.
- Its 5,838 Rust lines are concentrated in modules that mix concerns: `pages.rs` is 2,884 lines, `pages/editor.rs` is 1,168, `pages/editor_config.rs` is 787, and `pages/tests.rs` is 520. The remaining API, HTTP, type, and facade files total 479 lines.
- `pages.rs` imports data-ops at module scope, so a nominally viewer-only consumer cannot demonstrate that authoring code and dependencies are absent.
- The current boundary script audits the extracted Dataset, Forms, Workflows, Responses, Organization, and UI crates, but does not audit Components. A new Dashboard-to-Components edge would therefore both violate the documented sibling-feature prohibition and escape the current automated checks.

Cargo feature partitioning was considered and rejected for Sprint 5A. It would require splitting the same mixed modules, Cargo feature unification would make the effective workspace graph harder to reason about, and the package would still combine authoring and reusable rendering ownership. A small leaf crate makes the allowed edge explicit and independently auditable.

Root callback composition and renderer duplication were also rejected. Callbacks would move feature orchestration into the root adapter, while duplication would create two execution/rendering implementations and two places to fix security, pagination, and visualization behavior.

## Approved Ownership

### `tessara-web-dashboards`

The new crate owns:

- Dashboard-specific web DTOs, including redacted placement envelopes and bootstrap payloads;
- REST loaders and mutation actions for Dashboard endpoints;
- directory, create, detail, editor, and focused viewer content;
- Dashboard-specific loading, empty, error, stale-save, and capability-aware presentation states;
- composition editor orchestration and the named `DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS` policy;
- viewport scheduling and placement lifecycle around the leaf Component renderer;
- client-side consumption of request-scoped bootstrap data, followed by REST refresh and mutation behavior;
- feature tests for DTO adaptation, redaction, loaders/actions, editor state, viewer scheduling, and content rendering.

It may depend directly on framework/runtime libraries, `tessara-core`, `tessara-dashboards`, the policy-neutral `tessara-web-http` and `tessara-web-ui` crates, and `tessara-web-component-viewer` as demonstrated by actual use. It must not depend on root `tessara-web`, `tessara-api`, router/meta crates, `tessara-web-components`, `tessara-web-data-ops`, or another feature-area web crate.

The crate uses the same `default = []`, `hydrate`, and `ssr` feature shape as the existing extracted web crates. Root `tessara-web` forwards its hydrate/SSR features into the Dashboard crate; the Dashboard crate forwards only the matching feature into the viewer leaf.

### Public Dashboard facade

The initial `lib.rs` facade is deliberately narrow:

```text
DashboardsIndexContent
DashboardCreateContent
DashboardDetailContent { dashboard_id: String }
DashboardEditorContent { dashboard_id: String }
DashboardViewerContent { dashboard_id: String }

DashboardRouteBootstrap
```

`DashboardRouteBootstrap` is a serializable feature-owned enum with route-specific variants for directory, create, detail, editor, and viewer initial state. Its fields use Dashboard web DTOs rather than API DTOs or database records. Public constructors expose only the data the SSR adapter needs; REST clients, actions, page modules, and mutable editor internals remain private.

Content components receive normalized route values from root adapters and consume the matching request bootstrap from Leptos context. They fall back to REST loading on client-side navigation or when no matching bootstrap exists. A bootstrap for one route or Dashboard id must never be reused for another.

### Root `tessara-web` route adapters

Root `crates/tessara-web/src/routes/dashboards.rs` remains the sole owner of Dashboard route registration. It adds private adapter components around the public facade:

| Route | Root adapter | Feature content | Route policy |
| --- | --- | --- | --- |
| `/dashboards` | `DashboardsPage` | `DashboardsIndexContent` | read session/navigation policy and directory title |
| `/dashboards/new` | `DashboardCreatePage` | `DashboardCreateContent` | manage session/navigation policy and create title |
| `/dashboards/:dashboard_id` | `DashboardDetailPage` | `DashboardDetailContent` | parse id, read policy, detail title |
| `/dashboards/:dashboard_id/edit` | `DashboardEditorPage` | `DashboardEditorContent` | parse id, manage policy, editor title |
| `/dashboards/:dashboard_id/view` | `DashboardViewerPage` | `DashboardViewerContent` | parse id, read policy, viewer title |

The root adapters own `AppShell`, active navigation, route parameter extraction, route titles, session/capability redirects, document/meta integration, and the primary SSR mode. They do not own Dashboard fetches, DTOs, feature markup, editor state, or Component execution.

Root `tessara-web` continues to own `application_html`, the new bootstrap-aware render entrypoint, hydration entrypoints, CSS/public assets, and cargo-leptos metadata. After adapters use the new facade, the placeholder `tessara-web/src/features/dashboards` module is removed rather than retained as a second owner.

### `tessara-web-component-viewer` leaf

The leaf owns only reusable, explicit-version Component execution and presentation:

- feature-neutral execution request/response web DTOs and REST transport;
- Table, Bar, Line, Pie, Donut, and Stat Card payload rendering;
- paged Table interaction and local execution/loading/error states;
- renderer lifecycle hooks needed to start, cancel, suspend, or remount one execution;
- a narrow facade such as `ComponentVersionExecutionContent { target, mode }`, where `target` contains the public Component reference plus exact version id required by the execution endpoint, and `mode` distinguishes full and embedded presentation without referencing Dashboard concepts.

The leaf does not own Component directory, editor, version publishing, version resolution, data-operation authoring, Dashboard placement geometry, title overrides, redaction policy, viewport scheduling, or route integration. `tessara-web-components` resolves a route reference to an exact version and delegates rendering to the leaf. `tessara-web-dashboards` calls the leaf only for server-authorized available placements that already contain an exact pinned version id. Redacted placements never pass a hidden id to the leaf and never mount an execution request.

The leaf may depend on the policy-neutral `tessara-web-http` and `tessara-web-ui` crates and the minimal Leptos/serialization/browser runtime libraries it uses. It must not depend on root, API, router/meta, Components, Dashboard, data-ops, or any other feature crate. This makes it a shared presentation primitive, not a new feature-area owner.

This record explicitly amends the general no-sibling-web-crate rule only for these two one-way edges:

```text
tessara-web-dashboards -> tessara-web-component-viewer
tessara-web-components -> tessara-web-component-viewer
```

No other feature-to-feature dependency is approved. The implementation must reflect this amendment in `docs/architecture.md` and in the boundary audit in the same boundary-establishing change.

`tessara-web-http` is infrastructure rather than a feature-area dependency. It centralizes policy-neutral JSON request preparation, error-envelope parsing, and authentication/retryable/terminal failure classification; endpoint selection, redirect behavior, session policy, and navigation remain with each owning feature or the root shell.

## Request-Scoped SSR Bootstrap

The first Dashboard content slice must not introduce a server-side HTTP loop. The approved SSR flow is:

1. The native API HTML handler receives `AppState`, the requested path/route parameters, and session headers.
2. It authenticates once and obtains the capability/scope snapshot used for both shell and route data.
3. It calls the same Dashboard service methods used by JSON handlers. It does not call Tessara's own HTTP endpoints.
4. API-local adapters map service results into web-owned `ShellSessionBootstrap` and `DashboardRouteBootstrap` values. API JSON DTOs, web DTOs, and service/domain types remain separate.
5. Root `tessara-web` receives an `ApplicationBootstrap`-style value through `application_html_with_bootstrap`. Root owns the top-level shell bootstrap and a route-bootstrap enum; it re-exports construction-facing Dashboard bootstrap types so `tessara-api` depends only on root `tessara-web`.
6. The renderer inserts the value into request-local Leptos context and serializes the same redacted value into escaped hydration state in the HTML document.
7. The matching Dashboard content component consumes that state for the first render and hydration without a duplicate initial GET. Client navigation, refresh, and all mutations use the feature crate's REST adapters.

No global or process-shared bootstrap state is permitted. The serialized value must contain no session token, database-only field, hidden Component/Dataset metadata, or unredacted placement binding. Directory, detail, editor, and viewer bootstraps must be produced from the same authorized service projections and redaction rules as their JSON equivalents. The create bootstrap may contain only the authorized capability and scope-selection data needed for the first paint.

SSR without WASM must render useful shell, Dashboard metadata, saved geometry, available placement titles/version labels, and generic redacted footprints. Actual available Component payload execution may begin after hydration; a redacted or off-screen placement must not create an execution request.

The feature crate has no dependency on `tessara-api` and no knowledge of `AppState`, headers, cookies, Axum extractors, or API service implementations.

## Boundary Audit Amendment

The boundary-establishing implementation must extend `scripts/check-web-crate-boundaries.ps1` for both `x86_64-pc-windows-msvc` and `wasm32-unknown-unknown` metadata graphs.

Required graph assertions:

1. Starting at `tessara-web-dashboards`, reject any path to `tessara-web`, `tessara-api`, `leptos_router`, `leptos_meta`, `tessara-web-components`, or `tessara-web-data-ops`; reject other `tessara-web-*` packages except itself, `tessara-web-http`, `tessara-web-ui`, and `tessara-web-component-viewer`.
2. Starting at `tessara-web-component-viewer`, reject any path to root/API/router/meta and any `tessara-web-*` package except itself, `tessara-web-http`, and `tessara-web-ui`.
3. Starting at `tessara-web-components`, allow its existing `tessara-web-data-ops` and `tessara-web-ui` dependencies, the policy-neutral `tessara-web-http` transport, and the new viewer leaf, but reject a path to `tessara-web-dashboards`, root, or API.
4. Starting at `tessara-web-http`, reject any path to root, API, Leptos/router/meta, or another `tessara-web-*` crate so the transport cannot acquire endpoint, navigation, or presentation policy.
5. Preserve the existing rule that pure domain crates cannot reach Leptos, Axum, SQLx, Gloo, Web APIs, JavaScript, or WASM dependencies.

Required source assertions:

- Dashboard sources must not import `AppShell`, root route/state namespaces, route hooks, `leptos_router`, `leptos_meta`, `tessara_api`, Components feature modules, or data-ops authoring modules.
- Viewer-leaf sources must not import route/shell concepts, Dashboard concepts, Components authoring/version-management modules, or data-ops authoring modules.
- Root-only `ShellSessionBootstrap` and application render-context types must not move into either feature crate.

The audit is a required test, not a review aid. A broad wildcard exception for Dashboard or Components does not satisfy this decision.

## Implementation and Validation Gates

### Measurement sequence

Capture a pre-extraction baseline immediately before the boundary change, with the same lockfile, toolchain, machine, feature set, and clean target policy used for the post-extraction run:

```powershell
.\scripts\web-refactor-pilot.ps1 `
  -Phase "sprint-5a-dashboard-pre" `
  -Inventory -BaselineChecks -BundleReport
```

After the leaf move, Dashboard scaffold, feature forwarding, root adapter switch, and boundary-audit update, repeat it as `sprint-5a-dashboard-post`. Store the generated inventory, command logs, timings, and artifact report with the Sprint closeout evidence. Do not use `-AllowNormalCache` for the decision-grade comparison.

The pilot's development-profile bundle is a trend signal. Production bundle approval uses two clean, behaviorally equivalent `cargo leptos build --release` runs and reports individual and combined `.js` and `.wasm` bytes under `target/site/pkg`. The boundary-only post build must not grow combined release JS/WASM by more than 5% without a traced, reviewed cause. A greater unexplained increase blocks the extraction. No `--split`, `#[lazy]`, `#[lazy_route]`, router exception, or other code-splitting change may be mixed into this comparison.

Record three warm watch rebuilds before and after the move on the same machine. Compare medians, not a single run. An unexplained regression greater than 20% or a failure to observe a transitive crate edit blocks the extraction pending correction.

### Targeted compilation

At minimum, the boundary change must pass:

```powershell
cargo check -p tessara-web-component-viewer --no-default-features --features ssr
cargo check -p tessara-web-component-viewer --target wasm32-unknown-unknown --no-default-features --features hydrate
cargo check -p tessara-web-dashboards --no-default-features --features ssr
cargo check -p tessara-web-dashboards --target wasm32-unknown-unknown --no-default-features --features hydrate
cargo check -p tessara-web --target wasm32-unknown-unknown --no-default-features --features hydrate
cargo check -p tessara-api --no-default-features --features ssr
cargo test -p tessara-web-component-viewer --all-features
cargo test -p tessara-web-dashboards --all-features
.\scripts\check-web-crate-boundaries.ps1
```

The Sprint validation remains authoritative and must also pass `cargo fmt`, workspace check/clippy/test, audit, `scripts/validate.ps1`, the Leptos build, smoke, UAT, and end-to-end suites listed in the Sprint 5A plan.

### Watch validation

With `cargo leptos watch` running:

1. Make and revert a harmless Rust edit in `tessara-web-dashboards`; confirm the hydrate and SSR sides rebuild and the browser reloads.
2. Repeat in `tessara-web-component-viewer`; confirm both Dashboard and Component consumers rebuild.
3. Repeat in the root Dashboard route adapter; confirm route integration still rebuilds.
4. Record rebuild duration and any watcher path/configuration changes needed. A manual server restart is not an acceptable substitute for watched source discovery.

### Browser and SSR validation

Before the first feature slice, the extraction scaffold must preserve the existing four placeholder routes and add the planned focused-viewer route without changing shell/session behavior. As content lands, direct-load and refresh all five routes in the table above and verify:

- the expected `AppShell`, active navigation, title, session redirect, and capability policy;
- no hydration warning, browser-console error, or `/bridge/*` request;
- no duplicate initial Dashboard JSON request when matching SSR bootstrap exists;
- client navigation still falls back to REST loading correctly;
- Component route viewing still renders through the new leaf after its code move;
- Dashboard placements use the same leaf renderer for every supported kind and preserve Table paging;
- unavailable/redacted placements expose no hidden id, title, Component, version, or Dataset metadata and issue no execution request;
- disabling JavaScript/WASM leaves directory, detail, viewer metadata, geometry, available labels, and generic redacted footprints useful;
- desktop and mobile layouts remain free of horizontal overflow.

Run the authenticated Sprint E2E, smoke, and UAT commands after the first directory slice and again at closeout. The extraction is not accepted based on compilation alone.

## Ordered Boundary Work

Keep these changes reviewable and behavior-preserving before Dashboard product work expands:

1. Capture the pre-extraction pilot and release artifact baseline.
2. Create `tessara-web-component-viewer`, move only explicit-version execution/rendering code, and make `tessara-web-components` consume it without changing Component route behavior.
3. Add the architecture and automated boundary rules described above.
4. Scaffold `tessara-web-dashboards`, move the Dashboard placeholders behind its public facade, forward hydrate/SSR features, and switch the root route adapters. Remove the old root feature module.
5. Capture post-extraction build, watch, browser, and release-bundle evidence. Apply the rollback rules if a gate fails.
6. Implement the request-scoped shell/route bootstrap path before the first Dashboard content slice.
7. Begin the Sprint plan's directory vertical slice in the approved Dashboard crate.

Do not combine the boundary-only measurement with the directory implementation, grid behavior, API contract changes, or code splitting. A behaviorally equivalent boundary commit is what makes bundle and build differences attributable.

## Rollback

Keep the viewer move, Dashboard scaffold, SSR bootstrap, and first product slice as separable commits or equivalently separable changes.

Rollback to root `tessara-web/src/features/dashboards` if any of the following remains unresolved after a focused correction attempt:

- the dependency audit finds a forbidden or cyclic path, especially Dashboard to Components/data-ops or a feature crate to root/API;
- the leaf cannot be separated without exposing authoring APIs or duplicating renderer behavior;
- hydrate and SSR builds disagree, SSR bootstrap causes duplicate initial loads, or route/session behavior changes;
- bootstrap serialization differs from JSON authorization/redaction or exposes hidden metadata;
- the watcher fails to observe Dashboard or viewer-leaf edits;
- the behaviorally equivalent release build grows combined JS/WASM by more than 5% without an accepted cause;
- the three-run median warm rebuild time regresses by more than 20% without an actionable correction;
- Component routes regress after adopting the shared leaf.

Rollback means:

1. Restore Dashboard content ownership to the named root `features::dashboards` module and remove the root-to-Dashboard-crate edge.
2. Keep root route registration and URLs unchanged.
3. Remove the Dashboard-specific audit exception and unused crate/feature forwarding.
4. Restore Component rendering to `tessara-web-components` if the leaf itself failed. If the leaf passed independently and Components uses it without regression, it may remain; Dashboard in root can consume the root's existing Component dependency without creating a feature-to-feature edge.
5. Preserve independently green API, database, domain-grid, and web-UI work. No REST or persistence rollback is implied by a frontend ownership rollback.
6. Record the failed measurement and cause in the Sprint progress/closeout evidence.

No fallback may add a temporary `tessara-web-dashboards -> tessara-web-components` edge, copy the Component renderer, or move route/shell policy into a feature crate.

## Reevaluation Trigger

If the approved extraction is rolled back, keep Dashboard work in root until the failed gate has a concrete remedy. Re-run this decision before the first editor/viewer slice that needs live Component rendering, or when any of these objective thresholds is reached, whichever comes first:

- root Dashboard content exceeds 1,500 non-test Rust lines;
- root Dashboard content exceeds 12 Rust source files;
- 10 commits touch the root Dashboard feature within a rolling 30-day period;
- the viewer leaf becomes independently green and removes the dependency cause of the rollback.

The size thresholds are review triggers rather than automatic approval. They are intentionally below the current smallest extracted feature so ownership can be reconsidered before another expensive move.

After a successful extraction, reevaluate the leaf boundary if it acquires Component authoring/version-management behavior, a data-ops dependency, Dashboard-specific policy, a second feature-specific exception, or measurable duplicate code/bundle cost. Such growth is evidence that the leaf contract needs redesign; it is not permission to broaden the allow-list.

## Consequences

The decision adds two Cargo packages and therefore some manifest, feature-forwarding, and audit maintenance. In return it gives the large Sprint 5A frontend a clear owner before implementation churn begins, keeps the application root thin, prevents Dashboard from inheriting Component authoring/data-ops dependencies, and establishes one reusable renderer for both feature areas.

This decision does not approve frontend lazy loading, route registration inside feature crates, shared API/web DTOs, a broad `tessara-web-platform` crate, or changes to the Sprint 5A product scope.

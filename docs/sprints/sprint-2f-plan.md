# Sprint 2F Plan: Runtime Status And Materialization Slice

## Sprint Summary

Sprint 2F makes runtime execution and materialization readiness visible and usable through native Tessara application surfaces. It starts from the post-RBAC, post-Rust/UI table baseline on branch `codex/sprint-2f` in `C:\Users\eric-dev\Projects\tessara-sprint-2f`.

The roadmap scope is intentionally vertical: operators should be able to inspect workflow/runtime status, materialization readiness, refresh state, and operational errors without leaving the application shell, while end-user workflow and response flows remain intact.

Orpheum is configured to use the local catalog at `C:\Users\eric-dev\Projects\orpheum`. Sprint 2F planning used `delivery-slice-planning`; the planning artifacts now exist under `docs/product`, `docs/architecture`, `docs/planning`, `docs/verification`, and `docs/security`, and the scenario was finalized and closed after artifact-backed checks passed. Implementation uses `implementation-and-release-prep`, and closeout should use `verification-and-release-gate` when the implemented candidate is ready for final evidence and release-readiness review. The broader `secure-delivery-feature-lifecycle` scenario is useful as a lifecycle checklist, but it is intentionally heavier than this sprint needs unless trust-boundary or compliance scope expands.

## Sprint Specifications

- Add workflow/runtime status visibility for active, draft, submitted, completed, and blocked or errored runtime records where those states exist in current data.
- Add materialization readiness and refresh status visibility for the current response to dataset pipeline without expanding Sprint 2F into full dataset authoring.
- Add a read-only native Operations surface at `/operations`, guarded by the new `operations:view` capability.
- Keep `operations:view` separate from `analytics:refresh`; viewing status must not grant refresh authority.
- Deliver coherent internal/operator monitoring surfaces that fit the native Leptos SSR shell and do not disrupt Home, Workflows, Responses, Datasets, Components, or Dashboards.
- Keep touched workflow, response, dataset, dashboard, component, auth, and operator behavior inside bounded modules when changed.
- Split maintenance, import, demo, or refresh commands away from HTTP startup where Sprint 2F touches those concerns.
- Add workflow-aware tracing and stable operator-facing error messages for touched runtime and materialization paths.
- Enforce or document the validation path for `fmt`, `check`, wasm hydrate check, Rust tests, Playwright scenarios, smoke, UAT, legacy import rehearsal, `clippy`, and `cargo audit`.
- Keep permission-controlled behavior covered through positive and negative Playwright scenarios whenever executable.
- Preserve native SSR route ownership, hydration cleanliness, and browser-console cleanliness for touched runtime and materialization routes.
- Preserve Orpheum traceability from slice scope to implementation evidence, verification matrix, evidence review, and release-readiness notes.
- Keep Orpheum planning backfill artifacts lightweight and sprint-owned: update them only when Sprint 2F scope, security posture, verification evidence, or implementation sequencing changes.

## Acceptance Criteria

- Operators can inspect runtime status through application UI without direct database access or script-only workflows.
- Operators can inspect materialization readiness and last refresh state through application UI.
- Operators need `operations:view` or `admin:all` to see Operations navigation or read operations status data.
- Runtime and materialization empty, unavailable, forbidden, and error states use stable user-facing language rather than raw internal strings.
- End users can still start, save, submit, and review response work through the existing assignment-backed flows.
- Scoped operators do not gain out-of-scope runtime, response, dataset, component, or dashboard visibility through new monitoring/status surfaces.
- Touched routes remain native SSR-owned and do not reintroduce `/app` shell assumptions, HTML-string route shells, `inner_html` route injection, `/bridge/*` assets, or JavaScript-owned application UI.
- The active stylesheet path remains explicit, and validation proves newly touched selectors are served by the deployed app.
- Dependency audit status is either green or documented with reachability, acceptance, and replacement/removal path.
- Orpheum scenario checks pass for any applied Sprint 2F scenario before the sprint is considered ready for handoff.

## Manual Test Plan

1. Start from a clean local deployment:
   - `.\scripts\local-launch.ps1`
2. Sign in as `admin@tessara.local`.
3. Open the runtime/materialization monitoring surface added or changed in Sprint 2F.
4. Confirm active workflow instances, assignment state, current step, completed-step history, and response lifecycle status are visible where applicable.
5. Confirm materialization readiness, refresh recency, refresh outcome, and unavailable states are understandable.
6. Open `/workflows`, `/workflows/assignments`, `/responses`, `/datasets`, `/components`, and `/dashboards`; confirm existing user flows still work.
7. Sign in as a scoped operator or use the Playwright-owned scoped-manager scenario shape; confirm out-of-scope runtime and materialization records are not exposed.
8. Run:
   - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
9. Run browser validation:
   - `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
10. Check touched pages in a browser for route ownership, hydration, served stylesheet selectors, and console cleanliness.

## Automated Test Plan

- `cargo fmt --all`
- `.\scripts\validate.ps1`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo audit`
- `npm --prefix end2end test`
- `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Scenario focus:

- Orpheum delivery-slice-planning: keep scope boundaries, sequencing assumptions, readiness conditions, verification framing, and security/compliance watchouts explicit before implementation.
- Orpheum implementation-and-release-prep: record implementation evidence, review posture, verification evidence, and release-preparation notes during the implementation sprint.
- Orpheum verification-and-release-gate: use at closeout to produce an honest evidence review, security/compliance posture when applicable, and release-readiness decision.
- Permissions scenario family: extend `end2end/tests/permissions.spec.ts` and `docs/playwright-permissions-scenarios.md` if Sprint 2F adds permission-controlled runtime/materialization routes, actions, or data paths.
- Workflow-mediated assignment scenario family: keep `end2end/tests/workflow-mediated-assignments.spec.ts` green to prove assignment-backed response starts and delegated/owned work still function.
- UAT scenario family: keep `scripts/uat-sprint.ps1` current with any new route ownership or role-gated behavior markers.
- Smoke scenario family: keep `scripts/smoke.ps1` proving local startup, seed, shell, and representative app surfaces.
- Legacy import rehearsal: keep the existing migration/import validation path documented or wired into CI if Sprint 2F touches maintenance/import command separation.

## Ordered Implementation Plan

1. Inventory current runtime and materialization data contracts, including workflow assignments, workflow instances, submissions, datasets, components, dashboards, and demo seed paths.
2. Use `/operations` as the route owner and `operations:view` as the read-only status capability.
3. Define DTOs for runtime status, materialization readiness, refresh outcome, unavailable state, forbidden state, and stable error display.
4. Apply Orpheum `implementation-and-release-prep` before code changes move beyond inventory and route/DTO decisions, so implementation evidence has an active scenario home.
5. Add or adjust API handlers/services/repos for status and readiness data, keeping authorization at the capability + scope + ownership boundary.
6. Build native SSR UI for the Operations surface using shared page, table, badge, chip, icon-button, and empty-state primitives.
7. Add deployed selector verification for any new CSS selectors touched by the monitoring UI.
8. Split touched maintenance, import, demo, or refresh commands away from HTTP startup if current implementation still conflates them.
9. Add tracing and stable operator-facing errors around touched runtime and materialization paths.
10. Extend Playwright permissions and workflow-mediated scenarios where the new behavior is executable.
11. Run the full validation and UAT matrix, then run `orpheum check run --json` for any applied scenario and update evidence review, verification handoff, route ownership, hydration, console, audit, and release-readiness status.

## Dependencies And Blockers

- Docker and local Postgres must be available for `local-launch`, smoke, UAT, and Playwright validation.
- Playwright browsers must be installed in the Windows session before `validate-e2e`.
- `cargo audit` may expose existing RustSec advisories; Sprint 2F treats those as blockers unless documented with reachability analysis and a replacement/removal path.
- The Orpheum catalog is external to this repository at `C:\Users\eric-dev\Projects\orpheum`; keep `.codex/orpheum/config.json` current if that catalog moves.
- The Orpheum planning session has been finalized and closed. Apply a fresh implementation scenario rather than reopening the archived planning session.
- Sprint 2G remains the planned scoped analytics/reporting compatibility hardening slice. Sprint 2F should not absorb broad report execution scoping or component authoring work except where new runtime/materialization status surfaces directly touch those paths.

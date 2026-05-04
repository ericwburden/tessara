# Sprint 2C Plan

## Sprint Summary

Sprint 2C hardens Tessara workflow and response-entry behavior by decomposing the backend workflow/submission slices, preserving native SSR ownership for workflow and response entry routes, and closing the scoped workflow-assignment authorization gap discovered during audit placement.

## Sprint Specifications

- Preserve the native route ownership already present for `/app/workflows*`, `/app/responses*`, response-start, resume entry surfaces, and touched administration links.
- Keep `/app/admin` explicitly legacy-only; do not add new product behavior there.
- Decompose touched backend slices into bounded-context modules, starting with `workflows` and `submissions`, while continuing the existing `auth`, `hierarchy`, and `forms` movement.
- Keep `tessara-api::lib` focused on router, middleware, asset, and state composition.
- Move transport decoding and response shaping into handlers, orchestration into services, and SQL into repositories for touched workflow and response slices.
- Add targeted integration suites for auth/session behavior, role/capability boundaries, form publish safeguards, workflow assignment, and response-start flows.
- Tighten shared UI primitives used by migrated routes so new SSR surfaces stop depending on raw inline `onclick` strings.
- Close the workflow-assignment authorization gap so operators can only start assignments inside effective scope.
- Add a negative regression proving a scoped operator cannot start another account's out-of-scope workflow assignment by UUID.

## Acceptance Criteria

- Public HTTP endpoints and response shapes remain compatible for `/api/workflows*`, `/api/workflow-assignments*`, `/api/responses/options`, and `/api/submissions*`.
- Workflow and submission handlers primarily decode input, call service functions, and shape responses.
- SQL for touched workflow/submission flows lives in repositories instead of expanding route handlers.
- Admins can start any workflow assignment, operators can only start workflow assignments whose assignment node is in effective scope, and response users can only start their own or delegated assignments.
- `/app/workflows*` and `/app/responses*` continue to render and hydrate through native SSR ownership with stable visible error and permission behavior.
- `/app/admin` remains legacy-only and no new product-facing behavior lands there.
- Targeted regressions isolate auth/session, capability, form publish, workflow assignment, and response-start failures without relying only on the broad demo script.

## Manual Test Plan

- Sign in as admin and browse `/app/workflows`, `/app/workflows/assignments`, `/app/responses`, and `/app/responses/new`; refresh each route and confirm native SSR ownership remains stable.
- As admin, assign workflow work and start a response from the workflow assignment path.
- As operator, start an in-scope response entry flow and verify an out-of-scope workflow assignment UUID is rejected.
- As respondent and delegator, start or resume the correct assignment-backed response work and confirm delegated pending work remains scoped to accessible delegate accounts.
- Confirm visible error and permission behavior remains stable under the UI Overhaul 2.0 shell.
- Confirm no tested product workflow requires `/app/admin`.

## Automated Test Plan

- Use the faster local development loop for UI/test iteration: run the app from the host with Docker Postgres, or use `.\scripts\local-refresh-api.ps1` / `.\scripts\local-launch.ps1 -SkipBuild` when only a container refresh is needed.
- Reserve full teardown/rebuild/redeploy checks for dependency, migration, container, closeout, smoke, and UAT validation.
- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cd end2end; npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Refresh Sprint 2C backlog and GitHub issue state so active work matches the current roadmap rather than the stale route-migration wording.
2. Split the workflow backend into `dto`, `repo`, `service`, and `handlers` modules while keeping router-compatible re-exports.
3. Split the submissions backend into `dto`, `repo`, `service`, and `handlers` modules for response-start, draft, list/detail, save, submit, and delete flows.
4. Fix workflow assignment start authorization with scoped operator enforcement and preserve admin, self, and delegation behavior.
5. Continue touched forms/hierarchy/auth decomposition only where needed for this sprint's workflow/runtime seams.
6. Keep `tessara-api::lib` as the route and state composition root, adding only module/router composition helpers when they reduce route sprawl without changing public endpoints.
7. Tighten native workflow/response UI action primitives and avoid adding raw inline `onclick` strings on touched SSR surfaces.
8. Add targeted integration coverage for the changed auth, capability, form publish, assignment, response-start, and scoped negative-regression behavior.
9. Run automated verification, complete manual UAT, and update progress with results.

## Dependencies And Blockers

- Sprint 2C starts from clean `main` at branch `codex/sprint-2c` in `C:\Users\ericw\Projects\tessara-sprint-2c`.
- The existing Sprint 2C GitHub milestone and issue backlog are stale and must be refreshed before implementation work is tracked.
- Open Sprint 2B issue hygiene remains separate closeout cleanup and is not part of Sprint 2C implementation scope.
- No schema migration is planned unless implementation discovers a hard blocker.

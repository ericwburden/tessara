# Sprint 1F Plan

## Sprint Summary

Sprint 1F inserts an explicit UI-conformance slice before workflow-runtime expansion. The goal is to align the current Tessara application shell and existing core routes with the canonical UI guidance so the application remains user-testable in the intended product shape while deeper roadmap work continues.

## Sprint Specifications

- Add `Sprint 1F: Application UI Guidance Alignment` to the roadmap and mark it as the next sprint.
- Refactor the shared application shell to use:
  - a utility-only top app bar
  - a visible static global search field
  - primary product navigation in a left sidebar
  - page-local headings and actions inside the main workspace
- Keep `/app/reports*` reachable but demote it from primary navigation as a transitional surface.
- Tighten the current `Home`, `Organization`, `Forms`, `Responses`, `Dashboards`, `Administration`, and `Migration` routes so they read as consistent directory/detail/editor surfaces.
- Preserve current route ownership, SSR-first delivery, and existing backend scope.
- Keep Administration application-grade but visually subordinate to primary product routes, and keep Migration reachable without dominating the shell.

## Acceptance Criteria

- The shared application shell renders a top bar with stable theme controls and a static global search field.
- Primary sidebar navigation emphasizes `Home`, `Organization`, `Forms`, `Responses`, and `Dashboards`, with `Reports` presented as transitional.
- Page titles and route actions render inside page-local headers instead of a hero-style shell header.
- Breadcrumbs appear only on deeper routes where hierarchy adds value.
- Core routes remain SSR-readable and hydration-safe.
- The application does not introduce shell-level horizontal scrolling at narrow widths.
- Administration and Migration remain reachable and visually distinct without becoming separate themes.

## Manual Test Plan

- Sign in through `/app/login` and confirm theme controls remain available in the top bar and persist after reload.
- Open `/app` and verify:
  - the shared home exposes product areas, transitional reporting, current readiness, and internal areas
  - the top bar search field is visible
- Open `/app/organization`, `/app/forms`, `/app/responses`, `/app/dashboards`, `/app/administration`, and `/app/migration` and confirm:
  - page headings appear inside the main workspace
  - sidebar navigation remains stable
  - route-specific actions remain in the page header
- Open a detail or edit route such as:
  - `/app/organization/{node_id}`
  - `/app/forms/{form_id}`
  - `/app/forms/{form_id}/edit`
  - `/app/dashboards/{dashboard_id}`
  and confirm breadcrumb rendering appears only on the deeper flows.
- Resize the browser to tablet and mobile widths and confirm the shell stacks cleanly without horizontal overflow.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cd end2end; npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Insert Sprint 1F into `docs/roadmap.md` and record kickoff in `docs/progress-report.md`.
2. Refactor the shared shell in `crates/tessara-web/src/application.rs` so top bar, sidebar, page headers, and breadcrumb behavior match the Sprint 1F contract.
3. Update shared shell styling in `crates/tessara-web/assets/base.css` so the new structure behaves correctly on desktop and narrow widths.
4. Tighten shared-home and route-facing copy so product routes no longer read like a test harness or builder console.
5. Extend Rust HTML-string tests in `crates/tessara-web/src/lib.rs` for top bar, global search, selective breadcrumbs, transitional reports navigation, and internal-area cues.
6. Extend Playwright coverage in `end2end/tests/app.spec.ts` for route readability, shell chrome, console cleanliness, and narrow-width behavior.
7. Run the planned verification commands and record any blocked commands or failures in the sprint progress notes.

## Dependencies And Blockers

- The current route surfaces remain transitional and heavily server-rendered in `application.rs`, so shell and copy cleanup must work with that structure instead of assuming a deeper route-component rewrite in this sprint.
- `/app/reports*` remains a supported transitional surface and cannot be removed while dashboard and component roadmap work is still pending.
- Responsive behavior improvements are limited to the current SSR shell and CSS structure; no backend or routing changes are planned in this sprint.

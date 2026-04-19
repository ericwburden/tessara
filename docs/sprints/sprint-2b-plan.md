# Sprint 2B Plan

## Sprint Summary

Sprint 2B hardens Tessara authentication and session handling so browser `/app` usage moves onto a server-managed session contract, while the settled auth-facing application routes are brought under the native SSR platform without falling back to the legacy bridge.

## Sprint Specifications

- Replace plaintext password comparison and storage with Argon2id password-hash verification and persistence.
- Add safe migration and backfill behavior for seeded and demo accounts plus user create and edit flows.
- Introduce server-managed browser sessions with `HttpOnly` cookie semantics for `/app` routes while retaining bearer-token support only for explicit CLI, script, and test flows.
- Add session expiry, revocation, logout invalidation, and last-seen tracking semantics.
- Introduce a central authenticated-account extractor and request-context boundary so auth parsing is no longer repeated ad hoc in handlers.
- Replace raw internal and database auth/session error exposure with stable application codes/messages and traceable server logging.
- Deliver native SSR ownership for `/app/login`, `/app`, `/app/organization*`, and `/app/forms*` under the settled auth/session contract.
- Remove shipped demo passwords from the public login surface while preserving local-development guidance in documentation or internal-only tooling.
- Avoid adding new inline action handlers on touched shared UI surfaces.

## Acceptance Criteria

- New and updated accounts persist Argon2id password hashes rather than raw passwords.
- Seeded and demo accounts still authenticate correctly after migration and backfill.
- Browser-authenticated `/app` requests succeed via same-origin `HttpOnly` cookie sessions without JavaScript-managed bearer tokens.
- Logout invalidates the current browser session, expired or revoked sessions stop authorizing `/app` access, and session activity updates are persisted.
- Touched auth/session handlers return stable application error codes/messages and do not leak raw database or internal strings.
- `/app/login`, `/app`, `/app/organization*`, and `/app/forms*` render through native Leptos SSR ownership with successful hydration and no bridge dependency.
- A tester can sign in, refresh, browse Organization and Forms, create or edit a form, publish a version, and sign out through native SSR-owned routes without touching the retained hybrid shell.

## Manual Test Plan

- Sign in through `/app/login` with a seeded account and confirm the application lands in `/app` under the intended native shell.
- Refresh the browser on `/app`, `/app/organization`, and `/app/forms` and confirm the session persists without bearer-token bootstrap in the page.
- Sign out and confirm returning to a protected `/app` route redirects back through the settled login flow.
- Open Organization and Forms routes as an authorized user and confirm route content, hydration, and navigation behave without legacy bridge fallback.
- Create or edit a form, publish a version, and confirm the action completes under the settled auth/session contract.
- Verify invalid credentials and expired or revoked sessions show stable, user-facing auth errors rather than raw backend strings.

## Automated Test Plan

- `cargo fmt --all`
- `C:\Users\ericw\.cargo\bin\cargo.exe check -p tessara-api`
- `C:\Users\ericw\.cargo\bin\cargo.exe check -p tessara-web --no-default-features --features hydrate`
- `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-api`
- `C:\Users\ericw\.cargo\bin\cargo.exe test -p tessara-web`
- `.\scripts\local-launch.ps1`
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- `cd end2end; node .\node_modules\@playwright\test\cli.js test`

## Ordered Implementation Plan

1. Implement issue [#46](https://github.com/ericwburden/tessara/issues/46): add password-hash schema, migration, and seeded-account backfill.
2. Implement issue [#47](https://github.com/ericwburden/tessara/issues/47): centralize login verification around Argon2id-backed auth verification.
3. Implement issue [#48](https://github.com/ericwburden/tessara/issues/48): move browser `/app` auth to the server-managed cookie session contract while keeping explicit scripted-token flows.
4. Implement issue [#49](https://github.com/ericwburden/tessara/issues/49): add expiry, revocation, logout invalidation, and last-seen semantics.
5. Implement issue [#50](https://github.com/ericwburden/tessara/issues/50): add the central authenticated-account extractor and request-context boundary.
6. Implement issue [#51](https://github.com/ericwburden/tessara/issues/51): introduce stable auth/session error envelopes and traceable server logging.
7. Implement issue [#52](https://github.com/ericwburden/tessara/issues/52): migrate `/app/login` and `/app` to native SSR ownership under the settled auth/session contract.
8. Implement issue [#53](https://github.com/ericwburden/tessara/issues/53): migrate `/app/organization*` and `/app/forms*` onto the same settled native SSR contract.
9. Implement issue [#54](https://github.com/ericwburden/tessara/issues/54): run the auth hardening UAT and regression matrix and close out the sprint.

## Dependencies And Blockers

- The live GitHub issue chain for Sprint 2B is still fully open from [#46](https://github.com/ericwburden/tessara/issues/46) through [#54](https://github.com/ericwburden/tessara/issues/54).
- The first meaningful parallel split does not open until after the shared auth/session foundation lands: [#52](https://github.com/ericwburden/tessara/issues/52) and [#53](https://github.com/ericwburden/tessara/issues/53) can proceed in parallel only after [#48](https://github.com/ericwburden/tessara/issues/48), [#50](https://github.com/ericwburden/tessara/issues/50), and [#51](https://github.com/ericwburden/tessara/issues/51) are in place.
- `docs/progress-report.md` still contains older Sprint 2A closeout entries that name the pre-resequencing next sprint; this kickoff entry supersedes that stale handoff note for active planning.

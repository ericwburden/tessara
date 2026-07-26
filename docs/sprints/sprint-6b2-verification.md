# Sprint 6B2 Verification Evidence

Status: implementation verification complete on 2026-07-25.

## Implemented Boundaries

- Purpose-bound Ed25519 envelopes, canonical JSON signing, explicit issuer/key
  IDs, `ShellContextV1`, authorization grants, enrollment eligibility,
  reservation/redemption, and fixture external identity contracts.
- Deployment-database installation control with one-way Argon2id claim
  verifiers, one-time secret output, advisory-lock serialization, immutable
  events, uniform public rejection, and signed Core finalization.
- Core Administration Capability Floor v1 (`core:admin`), designated
  enrollment role, viability predicate, edit/designation guards, authorization
  and Organization revisions, local and fixture-external enrollment, declared
  module action exchange, and same-origin module gateway.
- Scoped Records schema-v1 configuration validator, Organization UUID
  ownership, scoped directory/detail/create/update APIs, current-revision grant
  validation, atomic mutation replay consumption, private security-state sync,
  health/diagnostics, and native SSR routes. Core signs a separate
  `ShellContextV1` for page requests; the module verifies its installation,
  audience, correlation, lifetime, issuer, key, and purpose before rendering a
  complete document. A direct module page request without that context fails
  closed.
- Sprint 6B2 Compose topology with only Traefik/Core exposed publicly; the
  installation-control and module services use an internal service network and
  isolated database identities.

## Automated Evidence

The following commands passed in the Sprint 6B2 worktree:

- `cargo check --workspace`
- `cargo test -p tessara-module-contract`
  - 42 library tests, 8 dependency tests, 5 manifest fixture tests, and 2
    signed-protocol fixture tests.
- `cargo test -p tessara-web --lib`
  - 73 tests.
- `cargo test -p tessara-api --lib` with disposable PostgreSQL databases
  supplied through `TEST_DATABASE_URL` and `TEST_ENROLLMENT_DATABASE_URL`
  - 135 tests, including atomic/idempotent local and signed
    fixture-external enrollment.
- `cargo test -p tessara-installation-control` with
  `TEST_DEPLOYMENT_DATABASE_URL`
  - 4 state-machine tests and 1 PostgreSQL integration test proving concurrent
    one-winner reservation, secret-free status, consumption idempotency, and
    terminal nondisclosure.
- `cargo test -p tessara-reference-scoped-records` with
  `TEST_SCOPED_RECORDS_DATABASE_URL` and
  `TEST_SCOPED_RECORDS_UPGRADE_DATABASE_URL`
  - 6 unit tests and 2 PostgreSQL/HTTP integration tests proving signed native
    shell rendering, direct-page fail-closed behavior, A/X versus B/Y
    separation, atomic replay consumption, exact idempotent retry,
    changed-payload rejection, Organization-filtered reads, and compatible
    Sprint 6B1 binary rollback after schema upgrade.
- `npm --prefix end2end test -- --workers=1`
  - 60 Playwright tests passed with one worker and zero retries, including
    authenticated and JavaScript-disabled native shell coverage, permissions,
    module management, and responsive module-navigation behavior.
- `docker compose -f deploy/sprint-6b2/compose.yaml config --quiet`
- All Core, installation-control, and Scoped Records SQL migrations applied
  successfully under their distinct migration roles to isolated PostgreSQL 17
  databases using single-transaction semantics.
- `git diff --check`

## Live Slice Evidence

An isolated Sprint 6B2 PostgreSQL 17 deployment was migrated under separate
Core, installation-control, and module identities. Local Core,
installation-control, and Scoped Records processes were then exercised through
their real HTTP boundaries:

- `/enrollment` returned a complete bare document with no authenticated shell,
  a write-only claim warning, initial/recovery selection, Capability Floor v1
  explanation, and ordinary sign-in destination.
- Core authentication succeeded, and the same-origin Scoped Records gateway
  returned list and native page responses without forwarding browser cookies
  or Core credentials to the module.
- Create returned `201`; an exact idempotency retry returned `200` with the same
  record identity; replay with a different payload was rejected.
- After advancing `authorization_revision`, a previously issued grant returned
  `409 authorization_stale`.
- A Core-proxied health page returned `200` with a verified active shell,
  navigation projection, and distinct Readiness heading. The same direct module
  page request without `ShellContextV1` returned
  `401 shell_context_unavailable`.
- The enrollment page was inspected in the in-app browser at desktop size. The
  production shell and screen deltas remain backed by the approved runnable
  HTML/CSS review suite and the 73 `tessara-web` tests.

## Guided Enrollment And Recovery Follow-up

- `.\scripts\tessara.ps1 enrollment issue -Open` now issues a one-time claim
  and opens a short-lived browser handoff containing only non-secret claim
  context.
- `.\scripts\tessara.ps1 enrollment recover -Reason "..." -Operator "..." -Open`
  records the explicit local operator authorization before issuing a recovery
  claim and opens the same guided browser workflow.
- The browser handoff is stored as a hash, consumed once, and never carries the
  claim secret. Claim ID, generation, kind, and installation ID are prefilled;
  email, display name, password, and claim secret remain write-only user input.
- Password requirements are visible before submission. Submission failures
  remain on the designed enrollment page with retry/reissue guidance.
- A direct GET of the former local redemption endpoint now redirects to
  `/enrollment` instead of exposing an API method error or raw JSON.
- A successful redemption renders a concise `Enrollment successful`
  confirmation, removes the unavailable-state badge and explanatory security
  panel, and redirects to `/login` after 1.8 seconds with an explicit sign-in
  link as a fallback.
- The live Compose workflow issued both initial and recovery handoffs.
  Recovery authorization persisted the supplied operator identity and reason;
  handoff replay did not repopulate claim context.

## Final UI Conformance Follow-up

- `design-qa.md` records the final PASS against the approved Sprint 6B2
  screenshots at desktop, tablet, and mobile viewports.
- Corrected details include the Application state panel, editable Display
  label, canonical breadcrumbs and icons, non-duplicated table actions,
  standalone create/edit screens, the Roles floor note and removed Enrollment
  column, stacked mobile record and role cards, and operational page titles.
- Dark and light themes rendered without horizontal overflow. The in-app
  Browser error log was empty after the final route pass.
- The migration services reuse their corresponding runtime images, ensuring
  the image used for migration and service startup contains the same schema
  state while avoiding duplicate Compose builds.

## Legacy End-to-End Harness Note

The 2026-07-25 post-build invocation of
`npm --prefix end2end test -- --workers=1` reached 25 passing tests before the
legacy harness attempted `docker compose exec postgres` against the stopped
root `docker-compose.yml`. Sprint 6B2 intentionally runs from
`deploy/sprint-6b2/compose.yaml` with separate database names and roles, so
that cleanup/setup hook cannot target this stack as written and the remaining
dependent tests were not a valid Sprint 6B2-stack gate. The earlier complete
60-test result above remains the repository regression baseline; the new
cross-process and UI behavior is covered by the targeted Rust tests and live
Sprint 6B2 browser checks documented here.

## Development Trust

The checked-in verification-key printer and Compose defaults are deterministic
development fixtures only. Core receives its development private signing
material; installation control and Scoped Records receive the matching public
verification keys. Production secrets and rotation remain deployment
responsibilities and must use Compose/Kubernetes Secrets.

## Acceptance Disposition

The Sprint 6B2 plan's implementation and verification gates are satisfied.
The repository-wide Playwright suite is a regression check against the
long-running local stack; Sprint 6B2's new cross-process behavior is verified
against the isolated slice through database-backed integration tests and live
HTTP checks because that API-only acceptance build intentionally does not
hydrate the ordinary Core sign-in shell.

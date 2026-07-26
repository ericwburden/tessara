# Sprint 6B2 Verification Evidence

Status: closeout verification complete on 2026-07-25.

Closing implementation/evidence source commit:
`c21398e2e026b06411292db34fb6ac0e1a871dde`.

Running source-exact Core image:
`sha256:f7aa6bf369f96d9fcc907c10899bee6f924974d30611aa6a3cbc7d3f17969476`.

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

The following commands passed in the Sprint 6B2 worktree. The final retained
browser, UAT, and smoke cycle ran against the source-exact closing image and
fresh database volumes:

- `cargo test -p tessara-module-contract`
  - 42 library tests, 8 dependency tests, 5 manifest fixture tests, and 2
    signed-protocol fixture tests.
- `cargo test -p tessara-web`
  - 77 tests.
- `cargo test -p tessara-api` with disposable PostgreSQL databases
  supplied through `TEST_DATABASE_URL` and `TEST_ENROLLMENT_DATABASE_URL`
  - 144 library tests and all integration groups passed: 1, 14, 7, 1, 4, 3,
    2, and 25 tests respectively, plus doc tests.
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
- `.\scripts\validate-e2e.ps1 ...`
  - The retained run passed all 60 tests with zero failures, skips, retries,
    flakes, or filtered tests.
- `.\scripts\uat-sprint.ps1 ...` and `.\scripts\smoke.ps1 ...`
  - Both retained fresh-data acceptance runs passed.
- `docker compose -f deploy/sprint-6b2/compose.yaml config --quiet`
- All Core, installation-control, and Scoped Records SQL migrations applied
  successfully under their distinct migration roles to isolated PostgreSQL 17
  databases using single-transaction semantics.
- `git diff --check`

## Fresh Baseline And Provenance

The closing stack was destroyed with
`docker compose -f deploy\sprint-6b2\compose.yaml down -v --remove-orphans`,
then rebuilt and launched from the closing source. The Core build received
`TESSARA_SOURCE_COMMIT`, `TESSARA_SOURCE_TREE`, and
`TESSARA_SOURCE_DIRTY=false`; captured deployment evidence confirms the exact
commit and image above.

The migration ledgers contain exactly one successful row each:

- `tessara_core`: `1 | baseline | true`
- `tessara_deployment`: `1 | enrollment claims | true`
- `tessara_module_scoped_records`: `1 | scoped records | true`

The final baseline SHA-256 digests are:

- Core:
  `3d9c03e38baad9416ca8425ad32e4baa245311ab829a3ec465bf60484debe3a7`
- Installation control:
  `e8e85037191a3e267a3c10f3baf8f2f34a3162b509377200fa1f07630f7b1d91`
- Scoped Records:
  `0b012fcf9e6e61301ef33d756e9342514654ffac0fc0e3fc6d62c349bbbd71c7`

`.\scripts\bootstrap-sprint-6b2-deployment.ps1` applied deployment receipt
revision 1 and then proved idempotency by returning a no-op on its second run.

`scripts/local-launch.ps1 -FreshData` was not used because it targets the
legacy root Compose topology. The Sprint-specific destructive reset,
source-provenance build, `up -d`, and deployment bootstrap above are its exact
6B2 closeout equivalent.

## Retained Evidence

- `artifacts/sprint-6b2-closeout/deployment-fresh.json`
- `artifacts/sprint-6b2-closeout/smoke-fresh.json`
- `artifacts/sprint-6b2-closeout/uat-fresh.json`
- `artifacts/sprint-6b2-closeout/e2e-fresh.json`
- `artifacts/sprint-6b2-closeout/e2e-fresh.summary.json`

Each retained JSON artifact has its adjacent SHA-256 sidecar where supported.

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

## Development Trust

The checked-in verification-key printer and Compose defaults are deterministic
development fixtures only. Core receives its development private signing
material; installation control and Scoped Records receive the matching public
verification keys. Production secrets and rotation remain deployment
responsibilities and must use Compose/Kubernetes Secrets.

## Acceptance Disposition

The Sprint 6B2 plan's implementation and verification gates are satisfied.
The complete repository Playwright suite, fresh UAT, fresh smoke, isolated
database-backed integration suites, and source-exact deployment capture all
passed against the final implementation. The Sprint 6B2 stack remains running
at `http://localhost:8080` for user testing.

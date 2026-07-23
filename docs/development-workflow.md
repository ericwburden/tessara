# Tessara Development Workflow

This document separates the day-to-day development loops by speed and intent.
The commands below describe the current single-service transition baseline plus
the first Sprint 6A module-contract checks. As later Phase 6 runtime tooling
lands, module-focused and full-composition workflows will be added without
weakening these baseline gates.

## Recommended Loops

### Fast loop: host-run Tessara with Docker Postgres

Use this when you are actively changing UI or API code and want the shortest
recompile cycle.

```powershell
docker compose up -d postgres
Copy-Item .env.example .env
cargo leptos watch --split
```

What this does:

- keeps Postgres in Docker
- runs the Leptos SSR app and API on the host
- avoids rebuilding the Docker API image on every change
- gives the shortest feedback cycle for route, shell, and handler work

Use this loop for most inner-loop development.

## Medium loop: refresh only the API container

Use this when you want to validate the containerized API image without tearing
down the full stack or reseeding everything from scratch.

```powershell
.\scripts\local-refresh-api.ps1
```

Useful options:

```powershell
.\scripts\local-refresh-api.ps1 -SkipBuild
.\scripts\local-refresh-api.ps1 -SkipSeed
.\scripts\local-refresh-api.ps1 -FollowLogs
```

What this does:

- ensures Postgres is running
- rebuilds only the `api` image unless `-SkipBuild` is supplied
- recreates only the `api` container
- waits for `/health` and `/`
- seeds demo data only when the app database is empty; use
  `.\scripts\local-launch.ps1 -FreshData` when demo data should be recreated
  from scratch

`-SkipSeed` affects that optional demo-data step only. API startup still runs
migrations, capability catalog synchronization, and built-in role-membership
contract convergence.

Use this loop when:

- you changed API or SSR code and want to check the Dockerized runtime path
- you do not need a clean Postgres reset
- you want a faster alternative to `local-launch.ps1`

## Slow loop: full stack rebuild and relaunch

Use this for closeout, smoke/UAT preparation, or when you need a fully refreshed
stack.

```powershell
.\scripts\local-launch.ps1
```

Useful options:

```powershell
.\scripts\local-launch.ps1 -FreshData
.\scripts\local-launch.ps1 -SkipBuild
.\scripts\local-launch.ps1 -SkipSeed
.\scripts\local-launch.ps1 -FollowLogs
.\scripts\local-launch.ps1 -ApiOnly
.\scripts\local-launch.ps1 -ExternalDatabaseUrl '<container-routable-url>' -ExternalDatabaseContainerId '<id>' -SkipSeed
```

Notes:

- `-FreshData` removes the Postgres volume before relaunching
- `-SkipBuild` reuses the current API image
- `-SkipSeed` skips only the optional post-start demo-data helper and leaves the
  current demo dataset untouched; startup migrations, capability catalog
  synchronization, and built-in role-membership convergence still run
- `-ApiOnly` delegates to `local-refresh-api.ps1`
- the paired external-database options are closeout-only: they bind the release
  API to the restored Sprint 5A demo target without volume reset or demo
  seeding, let startup apply migration 3, and verify the published port plus
  `current_database()`; the representative populated-upgrade fixture remains a
  separate compatibility-test database

Use this loop when:

- you want a clean Compose deployment
- you are preparing for manual UAT
- you need to verify image rebuild behavior end to end

## Suggested Usage Pattern

Use the loops in this order:

1. Fast loop while iterating on code.
2. Medium loop when you want to check the containerized API path.
3. Slow loop for smoke, UAT, or sprint closeout.

That keeps the common development path fast while preserving the existing
review-grade deployment path.

## Working Agreement

For routine UI, API, and feature-crate changes, the default loop is the fast
loop. Use host-run Tessara with Docker Postgres where possible, or use
`.\scripts\local-refresh-api.ps1` / `.\scripts\local-launch.ps1 -SkipBuild`
when a containerized app refresh is enough.

Do a full teardown, rebuild, and redeploy only when the change touches Docker,
dependencies, migrations, release-build behavior, closeout validation, smoke,
or manual UAT. Routine UI copy, selector, and layout changes should not pay the
full rebuild cost. Changes to test expectations remain subject to the test
change-control rules below regardless of which development loop is used.

When changing an existing extracted frontend feature area, prefer the focused
crate loop first, then run root integration checks before closeout. Keep current
root route, shell, authentication, hydration, document, CSS, and asset behavior
stable until the module gateway and SDK replace those responsibilities.

Do not assume that every new capability belongs in another root-integrated web
crate. New feature areas should be designed as full-stack module boundaries
owning UI, API, configuration, diagnostics, contracts, migrations, and data.

As Phase 6 tooling is implemented, the development workflow must add:

- a focused loop for one Core or module application and its own database
- manifest, `tessara-oci-v1`, configuration-schema, contract, route, security-capability, and health conformance checks
- generated-client/provider-consumer contract tests
- local same-origin multi-process startup and diagnostics
- local deterministic Materialization Plan plus separate Apply Authorization Envelope, Supervisor-ledger replay/conflict checks, Core/gateway restart, status, rollback, and receipt workflows
- database-isolation, scope-bound grant, freshness, and downstream-audience authorization-exchange checks
- module outage and degraded-state validation
- full-composition validation against an Application Blueprint and lockfile

Sprint closeout for a module-affecting change must run both focused module tests
and the resolved application's integration, browser, and conformance suites.

## Test Evidence And Change Control

Tests are durable executable contracts and proof of correctness, not
implementation debris. A test that fails after an implementation change is a
signal to investigate the implementation, requirement, or fixture; it is not by
itself authorization to change the test.

- Do not delete, skip, ignore, weaken, loosen, or rewrite an existing test merely
  to make a change pass. Do not increase retries or timeouts, relax selectors or
  assertions, or regenerate expected output for that purpose.
- Any changed expectation must cite an approved behavior or contract decision,
  explain why the previous assertion is no longer correct, and preserve
  equivalent or stronger coverage. Record changed tests and their requirement
  rationale in the sprint closeout evidence.
- Accepted versioned contract fixtures are immutable. Correct a faulty fixture
  by adding a new versioned fixture with a written rationale; do not rewrite an
  accepted v1 fixture in place.
- Negative semantic fixtures that deserialize must assert the stable finding
  code, path, message, and deterministic order when more than one finding is
  expected. Structural/Serde rejection fixtures assert the documented category
  and offending field, variant, or profile token. Line/column-dependent Serde
  prose is not a public stable contract. Generic rejection alone is
  insufficient proof, and a decode error must not be presented as a semantic
  validation finding.
- Closeout evidence must map each acceptance criterion to its durable unit, API,
  migration, SSR, browser, smoke, or UAT proof and identify any expected
  exclusions. Unexpected skipped, ignored, or filtered tests are a failure.

## Canonical Closeout Validation

Run the check-only and reproducible gate from the repository root. The
fresh-sprint lifecycle uses two independently provisioned disposable databases:
one general test target and one distinct fresh-start/seed-lock target.
`scripts/validate.ps1` intentionally refuses to run without both URLs and the
exact destructive-reset acknowledgement so database-backed assertions cannot
silently skip and the fresh-start proof cannot reset the general test target.
Each sprint starts from one squashed baseline migration and a freshly seeded
database; upgrade and rollback evidence are not current closeout inputs.
`scripts/validate.ps1 -Fast` is an inner-loop check. Its API step runs
`cargo test -p tessara-api --lib --locked`; provide `TEST_DATABASE_URL` when
that library suite includes an intentional database-backed catalog-sync proof.
Fast mode never claims the destructive fresh-start proof.

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p tessara-module-contract --locked
npm --prefix .\end2end ci
npm --prefix .\end2end run install-browsers

$env:TEST_DATABASE_URL = '<disposable-test-database-url>'
$env:SPRINT_6A_FRESH_DATABASE_URL = '<second-dedicated-disposable-fresh-database-url>'
$env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = 'I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET'
.\scripts\validate.ps1
cargo test --workspace --all-features --locked

.\scripts\check-web-crate-boundaries.ps1
cargo audit --quiet
git diff --check
git status --short
```

The final `git status --short` must emit no unreviewed implementation or
closeout changes. Explicitly preserved user diagnostics may remain unstaged and
must be identified in the handoff.

The non-Fast `validate.ps1` path also invokes the exact optimized
resource-reference timing proof:

```powershell
cargo test -p tessara-api --test modules --release --locked resource_reference_restricted_known_random_latency_profile -- --exact --nocapture
```

The timing test is compiled only for optimized builds; a debug return/skip is
not release evidence.

### Historical pre-fresh-baseline upgrade and rollback protocol

The following procedure records the superseded pre-fresh-baseline migration
protocol. It is retained only to interpret historical Sprint 5A/6A evidence;
it is not a current closeout requirement and must not be used in place of the
single-migration, freshly seeded lifecycle above.

For historical migration or catalog-synchronization work, retain evidence that
a populated prior-sprint database upgrades without reset, restart is safe, and
repeated and concurrent synchronization is deterministic and failure-atomic.
User-managed roles, capability mappings, assignments, sessions, and product
identities are upgrade invariants. Deterministic built-in `admin`, `operator`,
and `respondent` role mappings are versioned seed data: they may be refreshed
only by replacing `role_capabilities` membership for those names with contract
`sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68` (canonical SHA-256
`2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600`).
That contract is exactly `admin = [admin:all]`, the established 10-capability
operator set, and the established 2-capability respondent set. Built-in role
rows/IDs, assignments, accounts, sessions, user-role associations, and every
user-managed role/membership remain untouched. The built-in `admin` role's
universal implication preserves effective product access without mixing
installation-global and scope-aware capability rows. Authorized role-edit UI
and API actions remain available: a built-in membership edit applies to the
running installation but reconverges to the declared set at the next successful
startup; use a user-managed role for a durable custom bundle. Changing the seed
contract itself must bump the digest-coupled version, update the exact-set
proof, and add a Sprint 6A test-change-log entry; a seed edit is not accepted as
an incidental test fix.
The focused proof is `cargo test -p tessara-api --test sprint_6a_populated_upgrade --locked`;
it fails when any of the three URLs is missing or empty, resolves all three
through `current_database()` before either reset, requires a token-bounded
`test`, `tests`, `testing`, `upgrade`, `clone`, `rollback`, `sprint-6a`, or
`sprint6a` marker, and rejects any pair that resolves to the same database. Its migration-2 fixture first proves the exact
Sprint 5A 20-capability catalog plus admin-20/operator-10/respondent-2 mappings
frozen as `sprint-5a-role-capabilities-v1+sha256.7725e889996a` (full digest
`7725e889996a73a5655c57106aca6e12d9a5f95e9103f14d7b0fd50fbac96988`),
then proves invariant preservation, exact-set repair/restart/concurrency, and a
separate fresh exact set. With Node/npm and `cargo-leptos` available, plus either
local PostgreSQL client executables or one explicitly identified running
PostgreSQL container,
build and validate the full Sprint 5A SSR rollback artifact with
`scripts/build-sprint-6a-compatibility-rollback.ps1` and
`scripts/test-sprint-6a-rollback-package.ps1`.
Run the validator's database-free `-SelfTest` first, then `-Mode PackageOnly`
to verify only manifest metadata and immutable payload digests. Database modes
use deterministic `psql` scalar/JSON parsing and write the complete sorted
`admin`/`operator`/`respondent` mapping plus a canonical snapshot SHA-256 both
before and after package startup. Those mappings are not hidden by the broader
invariant fingerprint: they are asserted separately while that fingerprint
continues to cover every user-managed membership and all available module
control-plane tables exactly.

`CompatibilityOnUpgraded` requires the before snapshot to equal
`sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68`, proves rejected startup
with original migrations changes nothing, and requires the successful exact
Sprint 5A-code compatibility package to converge to admin-20/operator-10/
respondent-2 contract `sprint-5a-role-capabilities-v1+sha256.7725e889996a`.
The only allowed difference is restoration of redundant direct
product-capability rows on `admin`; `operator` and `respondent` are identical,
and effective admin authority is unchanged because `admin:all` is present in
both sets. `OriginalAfterRestore` requires exact Sprint 5A mappings before and
after startup on a restored migration-1/2 clone. For closing acceptance, use a
Sprint 5A source that already contains the exact demo actors and assets, retain
its all-table source/target restore fingerprint, and then let the clean Sprint
6A closing image apply migration 3 to that restored target with `-SkipSeed`.
That upgraded restored demo target is the Gate 4 candidate. The representative
`SPRINT_6A_UPGRADE_DATABASE_URL` fixture remains available for invariant and
`CompatibilityOnUpgraded` inspection only. Current-contract convergence is
proved by the populated-upgrade restart test and the closing deployment; it is
not claimed as part of an `OriginalAfterRestore` package run.
Their default package, manifest, and validation evidence paths are under
`artifacts/sprint-6a/`; retain that ignored directory with the closeout or
release artifacts rather than committing generated binaries.

Capture and independently validate the pre-upgrade backup/restore proof before
`OriginalAfterRestore`; a prose restore note or arbitrary identifier is not
accepted:

```powershell
$env:SPRINT_6A_CONFIRM_DESTRUCTIVE_RESTORE_RESET = 'I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET'
$restoreEvidence = 'artifacts/sprint-6a/rollback-restore-evidence.json'
$closing = (git rev-parse HEAD).Trim()
$postgresClientContainer = (docker inspect --type container --format '{{.Id}}' '<running-postgres-container-name-or-id>').Trim()
.\scripts\capture-sprint-6a-rollback-restore-evidence.ps1 `
  -SourceDatabaseUrl '<writable-sprint-5a-demo-source-url>' `
  -ExpectedSourceDatabaseName '<sprint-5a-demo-source-name>' `
  -MaintenanceDatabaseUrl '<same-cluster-postgres-maintenance-url>' `
  -TargetDatabaseUrl '<writable-restored-sprint-5a-demo-target-url>' `
  -ExpectedTargetDatabaseName '<restored-sprint-5a-demo-target-name>' `
  -BackupPath 'artifacts/sprint-6a/pre-upgrade-backup.dump' `
  -EvidencePath $restoreEvidence `
  -PostgresClientContainerId $postgresClientContainer
.\scripts\test-sprint-6a-rollback-package.ps1 `
  -Mode OriginalAfterRestore `
  -ExpectedClosingSprint6ACommit $closing `
  -DatabaseUrl '<writable-restored-sprint-5a-demo-target-url>' `
  -ExpectedDatabaseName '<restored-sprint-5a-demo-target-name>' `
  -RestoreEvidencePath $restoreEvidence `
  -PostgresClientContainerId $postgresClientContainer
```

Retain the Sprint 5A demo source between the two clean proof passes. If that
source is lost, recreate it from the already built and `PackageOnly`-validated
rollback package, never from closing Sprint 6A code and never by editing the
SQLx ledger. First create a new empty token-bounded disposable database, then
run the package's exact historical binary once with only its original
migrations. This is a recovery path for the source; do not run it against the
restored target after migration 3:

```powershell
$package = (Resolve-Path 'artifacts/sprint-6a/compatibility-rollback').Path
$manifest = Get-Content (Join-Path $package 'manifest.json') -Raw | ConvertFrom-Json
$historicalBinary = Join-Path $package $manifest.application.binary_path
$seedEnvironment = [ordered]@{
  DATABASE_URL = '<new-empty-sprint-5a-demo-source-url>'
  TESSARA_MIGRATIONS_DIR = (Join-Path $package 'original-migrations')
  TESSARA_DEV_ADMIN_EMAIL = 'admin@tessara.local'
  TESSARA_DEV_ADMIN_PASSWORD = 'tessara-dev-admin'
}
$previousEnvironment = @{}
try {
  foreach ($entry in $seedEnvironment.GetEnumerator()) {
    $previousEnvironment[$entry.Key] = [Environment]::GetEnvironmentVariable($entry.Key, 'Process')
    [Environment]::SetEnvironmentVariable($entry.Key, $entry.Value, 'Process')
  }
  & $historicalBinary seed-demo
  if ($LASTEXITCODE -ne 0) { throw "Historical Sprint 5A demo seed failed with exit code $LASTEXITCODE." }
} finally {
  foreach ($name in $seedEnvironment.Keys) {
    [Environment]::SetEnvironmentVariable($name, $previousEnvironment[$name], 'Process')
  }
}
```

The capture refuses to overwrite either retained artifact, requires source and
target ledgers exactly `1,2`, records the real PostgreSQL custom archive digest,
length, and header, and proves identical deterministic logical fingerprints
before the original package starts. The displayed container mode requires every
URL to use a literal IPv4 or IPv6 loopback host that matches exactly one
family-compatible `HostIp`/`HostPort` binding for that container's `5432/tcp`;
wrong-family and ambiguous bindings fail before database mutation. It verifies
URL credentials against the container without putting them in `docker exec`
arguments, derives password-free `127.0.0.1:5432` container-local URLs, and
streams the inbound archive through standard input as the container's configured
execution user. Unique container temporary paths are removed in `finally`
blocks even when restore fails. Omit
`-PostgresClientContainerId` only when unambiguous local `psql`, `pg_dump`, and
`pg_restore` executables are installed. Local-client mode remains available for
one deliberately narrow evidence URL form: an absolute `postgres://` or
`postgresql://` URI with one host, optional port, explicit non-empty user and
password, and exactly one database path; user, password, and database may be
percent encoded. Passwordless URIs and query or fragment components are rejected
before a client starts. They are not silently reinterpreted because safely
mapping the full libpq URI option surface to child environment variables is not
part of this rollback-evidence contract. Local mode records and revalidates each
executable's exact path and SHA-256 and supplies host, port, user, decoded
password, and database through the child process environment so
credential-bearing URLs never appear in process arguments. Restore evidence
binds both the capture wrapper and its dot-sourced common helper by SHA-256.
Rollback startup evidence embeds the complete sanitized stdout/stderr only after
the process has stopped. The sanitizer derives the decoded database password,
removes exact secrets, credential-bearing URLs, bearer tokens, and normalized
`password`/`passwd`/`pwd`/`PGPASSWORD` assignments, then records recomputable
UTF-8 byte lengths and SHA-256 digests.
Run every evidence/publication contract without a database or service before
the deployed acceptance passes:

```powershell
.\scripts\local-launch.ps1 -SelfTest
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -SelfTest
.\scripts\validate-e2e.ps1 -SelfTest
.\scripts\validate-resource-reference-nondisclosure.ps1 -SelfTest
.\scripts\test-sprint-6a-rollback-package.ps1 -SelfTest
.\scripts\test-sprint-6a-acceptance-evidence.ps1
```

That self-test rejects coercible-but-wrong JSON types, noncanonical UUIDs/UTC
timestamps, live fixture/digest mismatches, and malformed sidecars; proves JSON
plus SHA-256 sidecar publication; refuses replacement without `-Overwrite`;
rejects lexical and reparse-point aliases; and proves byte-for-byte prior-pair
restoration for failures after the first final move, at the former outer hash
point, and during cleanup. It also requires all temporary/backup/restore paths
to be absent and proves HTTP diagnostics retain only label, status,
content-type, UTF-8 length, and SHA-256 rather than raw response bodies.
The Playwright self-test executes the actual TypeScript demo-seed guard with a
counted mock request for upgraded, fresh, development, and invalid states, and
requires `/api/demo/seed` to appear exactly once in the test tree: inside that
guarded request function. It also proves a failed final deployment-digest gate
cannot replace any retained report. A real acceptance run revalidates the live
deployment, unchanged evidence digest, and exact database/data-state binding
after execution and immediately before summary publication.
After `OriginalAfterRestore`, point the clean closing release image at that
restored Sprint 5A demo target. `-SkipSeed` disables the launcher's optional
demo seed while startup applies migration 3 and current built-in membership:

```powershell
$upgradeDatabaseContainer = '<exact-running-restored-demo-database-container-id>'
$gate4ContainerUrl = '<postgres://credentials@host.docker.internal:published-port/exact-restored-demo-database-name>'
.\scripts\local-launch.ps1 `
  -ExternalDatabaseUrl $gate4ContainerUrl `
  -ExternalDatabaseContainerId $upgradeDatabaseContainer `
  -SkipSeed
$apiContainer = (docker compose ps -q api).Trim()

$upgradedEvidence = 'artifacts/sprint-6a/deployment-upgraded.json'
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -BaseUrl 'http://127.0.0.1:8080' -ExpectedDataState upgraded -OutputPath $upgradedEvidence -ApiContainerId $apiContainer -DatabaseContainerId $upgradeDatabaseContainer
.\scripts\smoke.ps1 -UseExistingService -BaseUrl 'http://127.0.0.1:8080' -KeepServices -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -AcceptanceEvidencePath 'artifacts/sprint-6a/smoke-upgraded.json'
.\scripts\uat-sprint.ps1 -BaseUrl 'http://127.0.0.1:8080' -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -AcceptanceEvidencePath 'artifacts/sprint-6a/uat-upgraded.json'
.\scripts\validate-e2e.ps1 -BaseUrl 'http://127.0.0.1:8080' -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -EvidencePath 'artifacts/sprint-6a/playwright-acceptance-upgraded.json'
.\scripts\validate-resource-reference-nondisclosure.ps1 -BaseUrl 'http://127.0.0.1:8080' -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -OutputPath 'artifacts/sprint-6a/resource-reference-nondisclosure-upgraded.json'
```

The upgraded capture must prove at least one acceptance product row predates
migration 3. With `ExpectedDataState=upgraded`, smoke, UAT, and Playwright never
call `/api/demo/seed`; they resolve and prove the already-restored Demo Session
Log assets. Fresh acceptance retains the established seed path. Any Gate 4 demo
mutation disqualifies the run.

Then launch a fresh seeded deployment from the same closing commit and run the
same acceptance set against that exact deployment:

```powershell
.\scripts\local-launch.ps1 -FreshData
$freshEvidence = 'artifacts/sprint-6a/deployment-fresh.json'
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -BaseUrl 'http://127.0.0.1:8080' -ExpectedDataState fresh -OutputPath $freshEvidence
.\scripts\smoke.ps1 -UseExistingService -BaseUrl 'http://127.0.0.1:8080' -KeepServices -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -AcceptanceEvidencePath 'artifacts/sprint-6a/smoke-fresh.json'
.\scripts\uat-sprint.ps1 -BaseUrl 'http://127.0.0.1:8080' -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -AcceptanceEvidencePath 'artifacts/sprint-6a/uat-fresh.json'
.\scripts\validate-e2e.ps1 -BaseUrl 'http://127.0.0.1:8080' -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -EvidencePath 'artifacts/sprint-6a/playwright-acceptance-fresh.json'
.\scripts\validate-resource-reference-nondisclosure.ps1 -BaseUrl 'http://127.0.0.1:8080' -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -OutputPath 'artifacts/sprint-6a/resource-reference-nondisclosure-fresh.json'
```

The capture is machine-derived. It refuses a dirty source tree or a running
image whose immutable ID and release/source labels do not match the clean
closing commit and tree. It authenticates to the live BaseUrl, matches the API
Application Installation to `current_database()` in the database container,
checks the successful migration ledger and current migration-file checksums,
recomputes the built-in seed contract digest, and matches all seven current
transition source digests between SQL and the API. Data state is historical:
an upgraded populated database has at least one product row created before
migration 3; a fresh database has none. Each acceptance wrapper re-runs those
checks, verifies the retained JSON SHA-256 sidecar, and rejects evidence from a
replaced container/image, different BaseUrl/database, or opposite data state.
For Gate 4, classification is necessary but not sufficient: the source/target
restore fingerprint, `-SkipSeed` launch, and smoke/UAT fallback to the existing
demo assets together prove acceptance data predates migration 3. Creating or
replacing demo assets after that migration disqualifies the upgraded pass.
For a non-default Compose project, pass exact running `-ApiContainerId` and
`-DatabaseContainerId` values to the capture; subsequent validation remains
bound to those IDs. `-DevelopmentMode` on smoke, UAT, or Playwright is an
explicit evidence bypass for local diagnosis and is never closeout proof.

Database identity is the exact evidence-bound
`database_runtime.container_id` + `database_runtime.database_user` +
`database_runtime.current_database` triple. Playwright acceptance derives its
fixture-cleanup environment from that triple, so cleanup cannot silently target
the ordinary Compose service while the API uses the upgraded restored demo
clone. The representative populated-upgrade fixture is a different database.

`validate-e2e.ps1` requires the application to be reachable and runs Playwright
from the `end2end` package. Its default acceptance mode rejects `-Spec` and
arbitrary Playwright arguments, compares discovery and execution with
the schema-v2 `end2end/acceptance-manifest.json`. That manifest freezes every
full `spec file :: describe › test` identity, not only per-file counts; a rename,
move, duplicate, addition, or removal fails both discovery and execution
validation even when the total remains unchanged. Acceptance also requires one
worker, zero retries, `forbidOnly`, every expected test passing once, and zero
skipped, flaky, retried, filtered, or unexpected results. It retains JSON, JUnit, discovery,
and digest-summary evidence. `-DevelopmentMode` explicitly permits targeted
diagnostic filters, but such a run is not acceptance evidence. A root-level
`npx playwright test` invocation is not the repository validation path. For a
direct non-evidence run from the repository root, use
`npm --prefix .\end2end test`; alternatively, change into `end2end` before
running `npx playwright test`. The root package intentionally does not own the
Playwright dependency or configuration, so a bare root invocation may resolve
a second, incompatible runner. Record
the closing commit, environment, exact commands, evidence digests, and test
counts with the closeout; do not normalize a red gate by changing or skipping
the test unless an approved product/contract decision is recorded with stronger
replacement proof.

`-OverwriteEvidence` replaces only an already validated Playwright discovery,
execution JSON, JUnit, and summary artifact set. It does not generate, modify,
or approve `end2end/acceptance-manifest.json`. The summary binds the exact
manifest SHA-256; changing a manifest identity requires an approved requirement
rationale in the sprint test-change log and equivalent or stronger executable
proof, even when the total count remains 60.

Acceptance artifacts are retained proof, not scratch output. Discovery,
execution JSON, JUnit, and summary are built and validated in a unique temporary
directory, then published together. Existing green evidence is not replaced
unless `-OverwriteEvidence` is explicit; a failed run or failed publication
preserves the prior set. Deployment capture follows the same rule with
`-Overwrite` for its JSON/sidecar pair. Smoke and UAT publish allowlisted
structured JSON plus sidecars only after exact current-run session logout,
credential/environment cleanup, and final deployment revalidation; their
deployment and acceptance paths must remain physically distinct, and
intentional replacement requires `-OverwriteAcceptanceEvidence`. The
nondisclosure gate also builds and
schema-validates its JSON in a unique temporary directory, writes and verifies
an exact LF-terminated SHA-256 sidecar, then moves the two files sequentially
inside a rollback-safe publication transaction. It does not promise two-file
reader atomicity. It refuses existing members unless `-Overwrite` is explicit,
rejects reparse-point path chains, keeps prior bytes recoverable through final
hash/result construction and cleanup, and restores the complete prior pair
after any pre-commit failure. Archive old evidence and record the reason before
any intentional replacement option is used.

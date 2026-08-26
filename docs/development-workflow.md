# Tessara Development Workflow

This document separates the day-to-day development loops by speed and intent.
The commands below retain the fast Core/root loop for Core and still-in-process
feature areas while also defining the focused module and full-composition work
required by the current multi-process baseline. Components, Dashboard, and
Scoped Records are independently built and deployed modules; their work is not
validated as a root-web-only change.

## Cargo Build Storage Policy

Cargo build products are reproducible working data, not retained validation
evidence. Use one of these three modes:

- **Development:** the worktree-local `target` directory is reusable and
  incremental compilation remains enabled for the active inner loop. Do not
  share this directory with another worktree or concurrent Cargo process.
- **Validation:** a complete validation lane uses one unique explicit
  `CARGO_TARGET_DIR`, `CARGO_INCREMENTAL=0`, and
  `CARGO_PROFILE_TEST_DEBUG=0`. All sequential Cargo commands in that lane
  reuse that one directory. The runner cleans it in `finally`, whether the lane
  passes or fails; logs, receipts, and evidence remain under `artifacts/`.
- **Diagnostic:** an isolated target may retain level-1 test debug information
  only when investigating a compiler, linker, symbol, or test-binary failure.
  Retention must be explicit, and the reported target path must be removed when
  the investigation ends.

`scripts/tessara-cargo-build-policy.psm1` is the canonical validation-platform
implementation for Cargo build-storage lifecycle. Its contract is
`tessara.validation.cargo-build-policy`; release `1.0.0` publishes the exact
module SHA-256 through `Get-TessaraCargoBuildPolicyIdentity` and prints it for
every entered lane. The ownership and adapter map are recorded in the
[validation-platform architecture](./architecture/validation-platform.md).

`scripts/validate.ps1` is the first current consumer and applies the policy to
its full gate. `-Fast` remains an active-development loop and therefore reuses
the worktree target. `-RetainCargoTarget` changes a full run to explicit
diagnostic mode and prints the retained path and policy fingerprint. Closed
sprint runners retain their historical implementation/evidence identity; every
new active sprint adapter must consume the platform release instead of copying
this lifecycle.

Before creating an isolated validation target, the policy requires at least
20 GB free on its volume. New validation runners must enter the policy once
around their complete sequential Cargo lane and exit it from `finally`; they
must not create a fresh target for every individual Cargo command. Independent
parallel Cargo lanes still require distinct target directories.

Cleanup is authorized by a private active lease, the exact generated target
name, an authenticated marker, and an ordinary path chain without reparse
points. A path prefix or caller-supplied state is not sufficient authority.
`scripts/test-tessara-cargo-build-policy.ps1 -SelfTest` certifies this lifecycle
against a synthetic temporary Cargo crate without running Tessara application
tests.

## Validation Platform Foundation

`scripts/tessara-validation-platform.psm1` is the public lifecycle boundary for
new validation adapters. Platform release `2.0.0` publishes an aggregate
fingerprint over its manifest, boundary inputs, Cargo build policy, validation
policy, lifecycle component, and evidence finalizer. Its independent execution
and finalization fingerprints let a finalization-only correction consume an
unchanged, identity-bound, digest-verified execution checkpoint only when its
positive post-checkpoint integrity pair already committed and every older-
fingerprint attestation is complete, without rerunning application actions. An
older finalizer's data-only partial currently requires rerunning the affected
lane. Caught result/index/attestation data or sidecar write, flush, and dispose
failures remove only the current invocation's partial and can retry the same
checkpoint; this does not cover abrupt host termination. Inspect the current
identity and run its application-independent certification with:

```powershell
Import-Module .\scripts\tessara-validation-platform.psm1 -Force
Get-TessaraValidationPlatformIdentity
.\scripts\test-tessara-validation-platform.ps1 -SelfTest
```

New sprint adapters are strict schema-v2 `tessara.validation.adapter` JSON documents.
They declare the governing validation contract, acceptance inputs, exact lane
inventory, prerequisites, environment bindings, named ports, topology, direct
program/argument arrays, readiness, and timeouts. Adapter files cannot contain
PowerShell callbacks or shell command strings. Every assertion maps to one
governing implementation target, exact structured command, proof classes, and
declared transitive tools; exact required-target coverage is validated before
any lane executes:

```powershell
Assert-TessaraValidationAdapter -AdapterPath <adapter.json>
```

Run one lane at a time through `Invoke-TessaraValidationLane`. Actions within a
lane are sequential and have no implicit retries. Independent lane processes
may run concurrently and receive unique platform leases automatically. Truly
independent processes may use separate evidence roots; a prerequisite or
retained-topology chain must share its authenticated root. The synthetic
concurrency proof establishes namespace isolation, not scheduler admission,
resource quotas, or bounded aggregate load. Retained Docker topology moves only
through an authenticated handoff receipt to its one declared successor.

The synthetic certification uses a local TCP service and deterministic Docker
CLI shim, so it requires neither a Tessara database, external network, nor live
Docker. The foundation also retains a digest-pinned live-Docker producer/
consumer proof with authenticated daemon/context/runtime identity and zero
residue. Every real sprint adapter must repeat the provider
proof for its exact Compose inputs, images/build provenance, capabilities,
resources, required services, readiness endpoints, teardown, and residue
checks. A later product sprint is responsible for its own exact adapter proof;
that consumer proof is not part of validation-platform diversion closeout.

The full `scripts/validate.ps1` gate runs both the Cargo policy self-test and
the aggregate platform certification. `-Fast` remains the reusable developer
loop and does not create isolated lifecycle evidence.

The platform foundation implementation is **ready for independent diversion
closeout**. It has lane-specific
acceptance, fixture, harness, adapter, environment, dependency, and prerequisite
identities; filtered child environments; unique leases, ports, projects, and
attempts; attempt-owned `TEMP`/`TMP`/`TMPDIR`; immediate truthful cleanup
ownership for every OS-started child, closed redirected stdin before successful
acquisition, and certified acquisition-fault cleanup;
contract-bound handoff with complete-claim replay rejection in the certified
sequential cases; immediate cleanup ownership for a returned complete consumer
claim; guarded finalization and transfer-flag clearing; and a separate execution
checkpoint, positive post-checkpoint integrity commit, and finalizer boundary.
Candidate identity is derived from Git source, governing contract, and current
dependency state; diversion closeout requires a clean matching plan.
The strict v2 publisher/authenticator binds that plan to committed lane results,
current finalizer attestations, exact targets, environment, prerequisites, and
evidence indexes. A later sprint's exact adapter consumes the closed release on
that sprint's own evidence chain; it is not evidence for this diversion.

Remaining adoption constraints are OS-native no-reparse source-path traversal
with adversarial Unix symlink-escape
proof; least-authority caller-token containment and bounded OS job/process-group
ownership; CPU/memory/disk/process/global-port/Docker quotas (including attempt-
temp storage); evidence retention/garbage collection;
and sensitive-artifact disposal; final artifact and prerequisite serialization
through the attestation commit; cross-finalizer partial-attestation recovery;
partial topology-claim/cleanup serialization or durable tombstoning; and
capability-authenticated hard-crash orphan cleanup. Release 2 rejects implicit
dotenv/`env_file`, mutable image/build inputs, custom/external resources, and
dangerous host capabilities and binds Docker endpoint/daemon/context plus actual
container/image identity. Each later product adapter must pass that exact live
boundary before its own lanes may use selective reuse.

At sprint closeout, remove the closed worktree's Cargo output with
`cargo clean --manifest-path <worktree>/Cargo.toml`. Never manually delete a
target path that has not first been resolved to the intended worktree, and do
not clean while Cargo or `rustc` is active for that target.

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
module crate/service loop first, then run root integration checks before
closeout. Keep current root route, shell, authentication, hydration, document,
CSS, and asset behavior stable for Core and still-in-process routes.
Components and Dashboard own their complete documents, hydration entrypoints,
and versioned assets through the generic module gateway/SDK seam.

Do not assume that every new capability belongs in another root-integrated web
crate. New feature areas should be designed as full-stack module boundaries
owning UI, API, configuration, diagnostics, contracts, migrations, and data.

Current module work must include:

- a focused loop for one Core or module application and its own database
- manifest, `tessara-oci-v1`, configuration-schema, contract, route, security-capability, and health conformance checks
- generated-client/provider-consumer contract tests
- local same-origin multi-process startup and diagnostics
- local deterministic Materialization Plan plus separate Apply Authorization Envelope, Supervisor-ledger replay/conflict checks, Core/gateway restart, status, rollback, and receipt workflows
- database-isolation, scope-bound grant, freshness, and downstream-audience authorization-exchange checks
- module outage and degraded-state validation
- full-composition validation against an Application Blueprint and lockfile

For a pre-production Phase 8 extraction, build and verify one offline,
destructive, source-exact materialization from empty owner databases. Rebuild
disposable seed data through owner-controlled bootstrap/read-back contracts in
dependency order, create new Module Instance references directly, and remove
the old storage, adapter, readers, payload shapes, and Core transition
descriptor in the same cutover. Do not add legacy migration, mapping,
rebinding, retained-adapter, or partial-resume behavior. For Sprint 8A, verify
the exact five-entry Core catalog (`tessara.forms`, `tessara.workflows`,
`tessara.responses`, `tessara.datasets`, and `tessara.migration`) and the
manifest-only reference order Scoped Records `7`, Components `8`, Dashboard
`9`, with no duplicate inventory or navigation presentation.

Sprint 8B and later extractions must follow the
[Phase 8 Module Extraction Playbook](./architecture/module-extraction-playbook.md).
Their validation contract selects `phase8-module-extraction` and maps exact
implementation commands to every required proof class. The inner loop closes
one ordered extraction slice at a time; clean materialization, semantic no-op,
failure recovery, fixture/runner proof, deployed smoke, and upgrade/rollback
all complete before formal Readiness. The `ui-sdk-conformance` proof is also
mandatory: establish the accepted visual/interaction baseline, map UI
ownership, build typed SDK views before cutover, and prove direct/lifecycle
visual and semantic parity. Do not fork a historical lifecycle runner or its
evidence lineage into a later sprint. Extract only genuinely policy-neutral
helpers; under contract v3, the sprint profile is the tracked adapter and every
lifecycle responsibility remains in the shared validation platform.

Sprint closeout for a module-affecting change must run both focused module tests
and the resolved application's integration, browser, and conformance suites.

## Implementation And Validation Policy V2

The first sprint after Sprint 8A adopts `tessara-validation-v2`. Sprint 8A and
earlier retained evidence remains governed by its original sprint-specific
protocol and runners. Do not retrofit, rewrite, or reassess that evidence merely
because the repository now contains the prospective policy.

Kickoff for a v2 sprint creates
`docs/sprints/<sprint-slug>-validation-contract.json`. The contract maps every
requirement to exact implementation targets and formal validation lanes, every
target and lane to dependency domains, and every domain to tracked input paths.
Validate it with `scripts/tessara-validation-policy.psm1`. An unmapped changed
path selects conservative validation; it is never silently treated as
unaffected.

The contract also selects an implementation profile. `standard` is the default
for ordinary work. A Phase 8 feature extraction uses
`phase8-module-extraction`, identifies the exact module and transition being
replaced, and is rejected if any mandatory playbook proof class is missing or
if materialization, semantic no-op, or failure recovery lacks a required clean-
environment target.

### Implementation exit

Implementation owns the known-target debugging loop. Before formal validation:

1. determine changed paths and affected dependency domains;
2. run every required or intersecting target from the tracked contract;
3. complete clean-environment materialization, semantic no-op, failure
   containment, recovery, fixture, runner, smoke, UI SDK/visual parity, and
   acceptance proof when the affected domains require it;
4. resolve every known failure; and
5. publish a compact, non-authoritative
   `implementation-readiness-result.json` under the ignored sprint evidence
   root.

A missing or failing target, dirty source, runner self-test failure, or missing
clean-environment proof blocks Readiness. Formal validation certifies a
completed implementation; it is not the routine way to discover whether a
known correction works.

### Phase trust and invalidation

Platform-backed Readiness and Candidate Rehearsal publish compact certificates
and, after a correction, rerun
only never-certified, failed, newly reachable, or dependency-affected lanes and
their prerequisite closure. The certifier may inherit authenticated unaffected
lanes with their prior receipt/hash, unchanged compatibility closure, and
explicit non-impact rationale. Before an active sprint adopts its exact adapter,
or whenever impact is uncertain, run the complete affected phase.

A downstream-only change does not reopen an upstream certificate. For example,
a Preflight-runner change leaves Readiness and Rehearsal closed when none of
their declared dependencies changed. A candidate-changing correction still
requires a successor freeze followed by complete SIT and complete UAT; no SIT
lane or manual UAT scenario is inherited across candidate fingerprints.

### Evidence packaging

Generated validation evidence remains untracked under `/artifacts/`. Each
phase attempt seals one local `evidence-index.json`; canonical phase results
contain compact summaries and the index hash. Routine downstream work reads
those certificates and the compact `evidence-chain.json`, not the complete raw
history. Raw evidence remains available for failure diagnosis and explicit
audit. Closeout performs one complete integrity audit over all sealed phase
indexes instead of every phase repeatedly rebuilding a global raw-file
manifest.

Run the shared policy contract tests with:

```powershell
.\scripts\test-tessara-validation-policy.ps1 -SelfTest
```

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
runner requires PowerShell Core 7.3 or newer and fails before importing policy
modules or invoking native tools on any other PowerShell runtime. The
complete gate uses seven freshly provisioned, pairwise-distinct disposable
databases: the general API integration target, the destructive API
fresh-start/seed-lock target, the conventional SQLx target, the independent
reference-module target, the extracted Component-module target, the API
enrollment target isolated from concurrently executing API library tests, and
the installation-control target. Do not reuse these fixture databases for a
second complete suite; recreate them first. `scripts/validate.ps1`
intentionally refuses to run without all seven URLs and the exact
destructive-reset acknowledgement. Every URL must use an explicit nonblank
password and a database in the `tessara_test`, `tessara_tests`, or
`tessara_testing` namespace, and all seven passwords must be pairwise unique.
Preflight compares normalized URL identities, then authenticates one ordinary
`psql` executable by literal path, content hash, and version before every
noninteractive, time-bounded probe. The targets may be spread across one or
more dedicated disposable PostgreSQL clusters. Both the seven authenticated
server/database identities and the seven authenticated server/role identities
must be pairwise distinct. Each validation role must own exactly its declared
database and no other non-template database on its cluster, have no `CONNECT`,
`CREATE`, or `TEMP` capability on any other non-template database there, have
no role memberships, and be `NOSUPERUSER`,
`NOCREATEROLE`, `NOREPLICATION`, and `NOBYPASSRLS`. Only the SQLx role is
`CREATEDB`; all six other roles are `NOCREATEDB`. The probe also requires exact
`CONNECT`, `CREATE`, `TEMP`, public-schema `CREATE`, and database comment
`tessara-validation-disposable-v1`. Thus credentials or hostname aliases cannot
disguise reuse of one physical database or role. URLs may contain at most one
recognized `sslmode`; identity overrides such as `dbname`, `host`, `hostaddr`, `port`,
`user`, `password`, or `service` are rejected.

Provision these roles in one or more dedicated disposable PostgreSQL clusters,
never an operator's shared development cluster. On every selected cluster, the
validation roles must not inherit PostgreSQL's default `PUBLIC` database
capabilities. Revoke those capabilities on its maintenance database and
immediately on every validation database; if a selected cluster contains
another non-template database, revoke them there too or recreate that cluster.
Use a separately generated secret for every role and `CREATEDB` only for the
validation role behind `TEST_SQLX_DATABASE_URL`. The following is a partial
single-cluster example; apply the same restrictions to each selected cluster:

```sql
REVOKE CONNECT, CREATE, TEMPORARY ON DATABASE postgres FROM PUBLIC;

CREATE ROLE tessara_validation_api LOGIN PASSWORD '<generated-api-secret>'
  NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE DATABASE tessara_test_api OWNER tessara_validation_api;
REVOKE CONNECT, CREATE, TEMPORARY ON DATABASE tessara_test_api FROM PUBLIC;
COMMENT ON DATABASE tessara_test_api IS 'tessara-validation-disposable-v1';
GRANT CONNECT, CREATE, TEMP ON DATABASE tessara_test_api TO tessara_validation_api;
-- Connect to tessara_test_api as an administrator before this grant:
GRANT CREATE ON SCHEMA public TO tessara_validation_api;

CREATE ROLE tessara_validation_sqlx LOGIN PASSWORD '<generated-sqlx-secret>'
  NOSUPERUSER CREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE DATABASE tessara_test_sqlx OWNER tessara_validation_sqlx;
REVOKE CONNECT, CREATE, TEMPORARY ON DATABASE tessara_test_sqlx FROM PUBLIC;
COMMENT ON DATABASE tessara_test_sqlx IS 'tessara-validation-disposable-v1';
GRANT CONNECT, CREATE, TEMP ON DATABASE tessara_test_sqlx TO tessara_validation_sqlx;
-- Connect to tessara_test_sqlx as an administrator before this grant:
GRANT CREATE ON SCHEMA public TO tessara_validation_sqlx;
```

Provision five more unique `NOCREATEDB` roles with distinct secrets and the
same restricted attributes for the fresh API, Reference, Component,
enrollment, and installation-control databases, placing them on any selected
dedicated cluster while preserving pairwise-distinct authenticated server/role
and server/database identities. Revoke the three `PUBLIC` database capabilities
immediately after creating each database, then grant them only to that
database's owning role. Do not grant one validation role to another.

The authenticated comment is destructive-use authorization, not a freshness
attestation. Recreate all seven databases before each complete suite; until
runner-issued provisioning receipts land, the operator remains responsible for
that freshness precondition. Do not run two complete validation suites
concurrently against the same seven database targets.

The runner snapshots and clears every process `PG*` setting and all eight
validation inputs (seven URLs plus the reset acknowledgement), and replaces
ambient `DATABASE_URL` with a deny sentinel for the whole gate. It rejects
`PG*` or reserved validation-input declarations in workspace `.env` search
paths. Preflight establishes exact canonical libpq settings. The generic
workspace partition excludes API, Dataset, Reference Scoped Records, Component,
and installation-control and receives no validation URL. Reference, Component,
and installation-control then run separately with only their own named URL.
The API general suite receives only `TEST_API_DATABASE_URL` and skips the exact
enrollment, SQLx, and destructive-fresh tests; enrollment receives only its
named URL; and API/Dataset SQLx targets alone receive
`TEST_SQLX_DATABASE_URL` as `DATABASE_URL`. The destructive fresh API target
runs last, after Dataset, with only `TEST_API_FRESH_DATABASE_URL` and the reset
acknowledgement. Every scope restores its prior state,
the runner reasserts isolation between scopes, and outer cleanup restores the
caller's exact present-or-absent state even when a product action fails. This
blocks process-environment and dotenv fallback from redirecting a test client
after preflight.

Before Cargo metadata or product work, and again before every central Cargo
invocation, the runner rejects undeclared Cargo/Rust execution-control
variables, including target runners/linkers, build targets, compiler or
documentation-tool overrides/wrappers, Rust flag channels, toolchain selection,
and test-thread overrides. It does not emit their values. It authenticates the
resolved Cargo and Rust compiler paths, content hashes, and verbose versions,
then supplies Cargo the exact authenticated compiler through a scoped `RUSTC`
binding and restores the caller's prior present-or-absent state after success
or failure. Each invocation also re-inspects effective `.cargo` configuration
from the repository and its ancestors plus the user Cargo home. Only
`build.jobs`, `build.target-dir`, `build.incremental`, and `net.retry` are
accepted; build target/tool/wrapper/flag controls, target runner/linker/flag
controls, Cargo `[env]`, config inclusion, unreadable syntax, unknown keys, and
ambiguous configuration shapes fail closed before that Cargo process starts.
When both supported config filenames exist in one search directory, the
extensionless `config` file is inspected because that is Cargo's effective
precedence.
Each sprint starts from one squashed baseline migration and a freshly seeded
database; historical populated-database/schema-migration upgrade evidence is
not a current closeout input. Sprint-specific independent module
upgrade/rollback checks remain required when the governing plan calls for them.
`scripts/validate.ps1 -Fast` is an inner-loop check. Its API step runs the
library suite while explicitly excluding the three known database-backed
catalog-sync, enrollment, and SQLx composition tests; `--lib` structurally
excludes the destructive-fresh integration target. Its Dataset step uses the
library target so SQLx integration targets do not run. An exact
source inventory guard fails when a `#[sqlx::test]` is added or moved outside
the declared API composition and Dataset inventories, rather than allowing the
Fast exclusions to become stale. The full gate runs all four API database
proofs and the Dataset, Reference, Component, and installation-control
integration targets; Fast runs only the installation-control library target.
Fast mode never claims database
integration or the destructive fresh-start proof.

The full runner partitions debug-profile Cargo tests exactly once across the
workspace. The later optimized timing proof intentionally repeats one selected
API assertion under the release profile. The runner executes the authoritative
workspace check and warnings-denied Clippy command through the authenticated
central Cargo boundary before product tests. It then
runs `cargo test --workspace --all-features --locked` with all five database-
scoped packages excluded while the deny sentinel is active, then runs each
package through the exact URL scopes above. Dataset library, binary, doctest,
example, benchmark,
and Cargo-named integration targets retain `--all-features`, including
feature-gated rollback proofs. SQLx ownership maps to an exact Cargo target
root; an included helper containing `#[sqlx::test]` fails closed until ownership
is made explicit. A locked-metadata self-test fails if a workspace package or
Dataset target falls outside those partitions. Do not substitute the raw
workspace command: it lacks the seven-database preflight and target-scoped
binding, and can therefore consult an ambient `.env` target.

Before applying a named skip, the runner lists the selected Cargo target and
requires one exact test-name match with no longer substring collision. Each
isolated API proof is also listed with `--exact`, executed with `--exact`, and
accepted only when exactly one assertion passes with none failed, ignored, or
measured. A renamed, duplicated, filtered-away, or silently ignored proof fails
the gate rather than shrinking product evidence.

```powershell
npm --prefix .\end2end ci
npm --prefix .\end2end run install-browsers

$env:TEST_API_DATABASE_URL = '<disposable-api-test-database-url>'
$env:TEST_API_FRESH_DATABASE_URL = '<disposable-api-fresh-database-url>'
$env:TEST_SQLX_DATABASE_URL = '<disposable-sqlx-test-database-url>'
$env:TEST_REFERENCE_MODULE_DATABASE_URL = '<disposable-reference-module-database-url>'
$env:TEST_COMPONENT_MODULE_DATABASE_URL = '<disposable-component-module-database-url>'
$env:TEST_API_ENROLLMENT_DATABASE_URL = '<disposable-api-enrollment-database-url>'
$env:TEST_INSTALLATION_CONTROL_DATABASE_URL = '<disposable-installation-control-database-url>'
$env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = 'I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET'
pwsh -NoProfile -File .\scripts\validate.ps1

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
recomputes the built-in seed contract digest, and matches the exact five Core
transition source identities—Forms, Workflows, Responses, Datasets, and
Migration—between SQL and the API. Dashboard and Components enter inventory
only through their real Module Release and Module Instance records. Data state
is historical:
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

## Future-sprint contract-v3 sequence

New kickoff packages use validation contract schema 3 / policy v3 and preserve
all retained v2 and legacy packages unchanged.

1. Kickoff creates and tracks the contract plus one schema-v2 validation
   adapter, declares platform release `2.0.0`, explicit artifact-fanout edges,
   per-slice exits, fixture/visual rules, exact target prerequisites and
   exclusive resource claims, shared coordinator activation, and Phase 8
   authorization/UI gates when applicable.
2. Kickoff validates through `Assert-TessaraValidationAdapter` and
   `Assert-TessaraFutureSprintPlanningPackage`. Missing or uncertain mappings
   block implementation handoff.
3. Implementation reconciles every declared projection in the touched cone,
   proves authorization at the real boundary and standalone UI ownership early,
   derives fixtures from signed owner read-back, and runs implementation lanes
   through `Invoke-TessaraImplementationHarvest`. The coordinator publishes its
   immutable evidentiary-priority plan before assertions, runs prerequisite
   closure first, continues safe siblings fail-late, serializes resource claims,
   and resumes authenticated checkpoints without duplicating completed work.
4. Implementation-readiness schema 2 binds the current source, contract hash,
   adapter hash, platform fingerprint, passing coordinator finalization,
   deterministic defect batch, fanout receipts, and exact slice exits. Reused
   targets remain labeled inherited/not newly executed. Formal Readiness is not
   a discovery pass for fixtures, environments, acceptance inventories,
   authorization, or generated assets.
5. Every formal lane runs through `Invoke-TessaraValidationLane`. Phase
   certificates and the evidence chain authenticate platform and adapter
   provenance; missing provenance blocks rather than falling back.
6. Closeout validates the chain, reports target/attempt efficiency and finding
   hotspots from retained receipts, and extracts reusable process lessons
   without changing accepted product behavior.

Only adapter actions are sprint-specific. Phase orchestration, topology/ports,
cleanup/restoration state, evidence publication, certificates, and fail-late
coordination remain shared-platform responsibilities. An exception requires an
explicit documented user or architecture authority in the contract.

The coordinator's target order is corrected failures with passing focused
reproducers, never-run targets, dependency-affected targets, authenticated
unchanged targets, then finalization. Open or uncorrected provenance blocks.
Reuse requires explicit policy permission plus matching evidence, command,
contract, adapter, platform, environment, dependency, compatibility, and
prerequisite-closure identities. Unknown impact executes. A source, fixture,
contract, environment, command, adapter, dependency, or correction change
requires a new immutable plan.

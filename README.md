# Tessara

Tessara is a modular application-construction platform. It combines a stable
Core for Organization, identity/users, RBAC, navigation, and composition with
selected full-stack feature modules to produce independently deployed and
supported applications.

This repository is the Rust + Leptos implementation of Tessara. It is developed
as a domain-driven product rather than as a one-for-one port of an earlier
system.

Project direction, architecture, roadmap, and UI rules are authoritative in
[`/docs`](./docs/README.md). This README focuses on
local development and operational workflow for the Rust workspace.

## Intended Architecture

- one Core application plus only the separately deployed full-stack modules an application needs
- one coherent same-origin, Leptos SSR-first user experience
- module manifests that advertise Feature Declarations, functional contracts, resources, semantic routes, navigation/shell contributions, configuration, health, and namespaced security capabilities
- Core-owned roles and assignments incorporating module-provided security capabilities
- one PostgreSQL cluster per v1 installation, with one Core database and one database per module instance
- APIs, events, exports, and typed resource references instead of cross-module table access
- declarative Application Blueprints and exact lockfiles for reproducible human, automation, and LLM-driven composition
- an installation-local Supervisor outside Core for locked Core Release components (including the gateway) and Module Release apply, health-gated traffic switching, and rollback

See [the modular platform contract](./docs/modular-application-platform.md) and
[target architecture](./docs/architecture.md) for the authoritative design.

## Current Workspace And Transition Seams

```text
tessara-api
tessara-analytics
tessara-auth
tessara-core
tessara-data-ops
tessara-datasets
tessara-dashboards
tessara-db
tessara-forms
tessara-hierarchy
tessara-jobs
tessara-module-contract
tessara-submissions
tessara-web
tessara-web-component-viewer
tessara-web-components
tessara-web-dashboards
tessara-web-data-ops
tessara-web-datasets
tessara-web-forms
tessara-web-http
tessara-web-organization
tessara-web-responses
tessara-web-ui
tessara-web-workflows
```

The workspace has these crates scaffolded now. The current implementation keeps
the vertical slice logic in one runnable service while the module runtime is
built. These crate boundaries are useful extraction seams, not the target
deployment topology.

Domain rules should move into their owning Core or full-stack module boundary as
contracts stabilize. Current extracted examples:

- `tessara-core`: shared field type parsing and JSON value validation
- `tessara-module-contract`: framework-neutral versioned Manifest, transition,
  semantic-destination, typed-reference, and resolution wire contracts
- `tessara-dashboards`: dashboard composition rules
- `tessara-forms`: form version lifecycle and section/field compatibility rules
- `tessara-submissions`: draft/edit/submit workflow rules and required value checks
- `tessara-web-*`: feature-owned native Leptos route/UI seams plus shared HTTP
  and UI primitives; these remain one deployed web application in Sprint 6A

The current crate name `tessara-core` predates architectural **Core** and is not
its boundary definition. During module extraction, field/value semantics must
move to their owning module or a genuinely policy-neutral SDK contract rather
than being retained in platform Core because of the crate name.

## Local Development

Copy the environment template if you want to run the API outside Docker:

```powershell
Copy-Item .env.example .env
```

Start the local stack:

```powershell
docker compose up --build
```

For the normal local rebuild/relaunch workflow, use the helper script:

```powershell
.\scripts\local-launch.ps1
```

That script:

- stops the existing Compose stack
- rebuilds the API image
- recreates the Compose services
- waits for `/health` and `/` to return `200`
- seeds the UAT demo dataset only when seeding is enabled and the application
  database is empty

Useful options:

```powershell
.\scripts\local-launch.ps1 -FreshData
.\scripts\local-launch.ps1 -FollowLogs
.\scripts\local-launch.ps1 -SkipBuild
.\scripts\local-launch.ps1 -SkipSeed
.\scripts\local-launch.ps1 -ApiOnly
.\scripts\local-launch.ps1 -ExternalDatabaseUrl '<container-routable-url>' -ExternalDatabaseContainerId '<id>' -SkipSeed
```

`-FreshData` also removes the local Postgres volume before relaunching.
`-FollowLogs` tails the Postgres and API container logs after startup.
`-SkipBuild` reuses the current API image.
`-SkipSeed` skips only the optional post-start demo-data helper and leaves the
current demo dataset untouched. It does not disable startup migrations,
capability catalog synchronization, or the versioned built-in role-membership
contract.
`-ApiOnly` delegates to the fast API refresh path instead of rebuilding the full Compose stack.
The paired external-database options are reserved for closing-build validation
against a restored Sprint 5A demo clone that closing startup upgrades in place.
They verify the running container, its published PostgreSQL port, token-bounded
database name, and `current_database()`; they reject fresh/reset/API-only modes
and require `-SkipSeed`. This browser candidate is separate from the
representative populated-upgrade fixture used by Rust compatibility tests. See the
[Sprint 6A deployment-evidence contract](./docs/sprints/sprint-6a-deployment-evidence.md).

For the fast inner-loop Docker refresh path, use:

```powershell
.\scripts\local-refresh-api.ps1
```

That script:

- keeps Postgres running
- rebuilds only the API image unless `-SkipBuild` is supplied
- recreates only the API container
- waits for `/health` and `/` to return `200`
- seeds demo data only when `-SkipSeed` is absent and the application database
  is empty; startup security/catalog synchronization still runs

Useful options:

```powershell
.\scripts\local-refresh-api.ps1 -SkipBuild
.\scripts\local-refresh-api.ps1 -SkipSeed
.\scripts\local-refresh-api.ps1 -FollowLogs
```

The API listens on:

```text
http://localhost:8080
```

The default development login is:

```text
email: admin@tessara.local
password: tessara-dev-admin
```

Get a bearer token:

```powershell
Invoke-RestMethod `
  -Method Post `
  -Uri http://localhost:8080/api/auth/login `
  -ContentType 'application/json' `
  -Body '{"email":"admin@tessara.local","password":"tessara-dev-admin"}'
```

Frontend development should use the cargo-leptos workflow:

```powershell
cargo leptos watch --split
```

For the fastest UI/API development loop, run Postgres in Docker and run Tessara on the host:

```powershell
docker compose up -d postgres
Copy-Item .env.example .env
cargo leptos watch --split
```

That host-run path avoids a Docker image rebuild for most UI and API changes. Use
`.\scripts\local-refresh-api.ps1` when you specifically want to validate the
containerized API image without doing a full stack reset.

Release packaging for the UI/application binary path:

```powershell
cargo leptos build --release --split
```

End-to-end coverage runs through the repository Playwright wrapper against an
already reachable application:

```powershell
npm --prefix .\end2end ci
npm --prefix .\end2end run install-browsers
.\scripts\local-launch.ps1 -FreshData
.\scripts\validate-e2e.ps1 -DevelopmentMode -BaseUrl "http://127.0.0.1:8080"
```

Useful checks:

```powershell
.\scripts\validate.ps1 -Fast
$env:TEST_DATABASE_URL = '<disposable-test-database-url>'
$env:SPRINT_6A_UPGRADE_DATABASE_URL = '<second-dedicated-disposable-upgrade-database-url>'
$env:SPRINT_6A_FRESH_DATABASE_URL = '<third-dedicated-disposable-fresh-database-url>'
$env:SPRINT_6A_CONFIRM_DESTRUCTIVE_UPGRADE_RESET = 'I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET'
.\scripts\validate.ps1
.\scripts\validate-e2e.ps1 -DevelopmentMode -BaseUrl "http://127.0.0.1:8080"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo audit --quiet
.\scripts\smoke.ps1 -DevelopmentMode
.\scripts\smoke.ps1 -ComposeApi -DevelopmentMode
```

`.\scripts\validate.ps1` is the standard pre-commit Rust validation path. It
runs formatting, the framework-neutral module-contract check/tests, API checks,
the API SSR check, web checks, wasm hydrate checks, and API/web tests
sequentially so Windows Cargo builds do not fight over the same artifact locks.
Full validation requires `TEST_DATABASE_URL`, a distinct
`SPRINT_6A_UPGRADE_DATABASE_URL` for the destructive populated-upgrade proof,
and a third distinct `SPRINT_6A_FRESH_DATABASE_URL` for fresh-start/seed-lock
proof. All three must resolve to token-bounded disposable database names, and
the exact reset acknowledgement is mandatory. Validation fails instead of
silently skipping database-backed assertions when any URL is absent or any two
resolve to the same live database. Use
`-Fast` only for the inner loop when SSR, wasm hydrate, and database checks are
not relevant to the change. Closing browser acceptance additionally restores a
Sprint 5A demo source into a fourth disposable target, validates it with
`OriginalAfterRestore`, and starts the closing image with `-SkipSeed`; it does
not repurpose or post-upgrade-seed the representative fixture database.

Testing should focus on behavior that protects domain and workflow boundaries:
validation rules, capability scope and ownership behavior, projection contracts,
component/dashboard composition, and end-to-end slice regressions. Avoid
placeholder tests that only assert generated boilerplate.

Tests are durable executable contracts. Do not delete, skip, weaken, loosen, or
rewrite a failing test merely to obtain a green run; do not mask failures with
broader retries, timeouts, selectors, or regenerated expected output. A changed
expectation requires an approved behavior or contract decision, a written
rationale, and equivalent or stronger replacement coverage. Accepted versioned
fixtures are immutable. See
[the development workflow](./docs/development-workflow.md#test-evidence-and-change-control)
for the complete change-control and closeout-evidence requirements.

Permission-controlled surfaces and actions must be covered through Playwright
when executable. By default, `validate-e2e.ps1` is an acceptance run: it requires
a retained Sprint 6A deployment-evidence record and expected upgraded/fresh
state, validates that record against the current live source/image/service/
database, and rejects
spec/CLI filters, discovers the exact checked-in acceptance inventory, runs it
with one worker, zero retries, and `forbidOnly`, then retains JSON/JUnit evidence
and fails on a skipped, flaky, retried, non-passing, filtered, or unexpected
test count. It validates discovery, execution JSON, JUnit, and summary in a
unique temporary directory before publishing the complete set; existing green
evidence is preserved unless `-OverwriteEvidence` is explicit, and publication
failure rolls the prior set back. Use `-DevelopmentMode` explicitly for targeted
local diagnosis; that output is never acceptance evidence. The wrapper restores
every Playwright environment variable it touches and runs Playwright from the
`end2end` package and includes `end2end/tests/permissions.spec.ts`; do not
substitute a root-level `npx playwright test` command. Update
`docs/playwright-permissions-scenarios.md` alongside new permission scenarios.

The default smoke script uses Docker for Postgres and runs the API locally with
`cargo run`. Use `.\scripts\smoke.ps1 -ComposeApi -DevelopmentMode` to validate
the fully containerized Compose deployment path during development. Sprint
acceptance instead runs an already-started labeled release image and supplies
`-DeploymentEvidencePath` plus `-ExpectedDataState`; see the canonical closeout
commands in [the development workflow](./docs/development-workflow.md#canonical-closeout-validation).

See [docs/development-workflow.md](./docs/development-workflow.md) for the
recommended fast/medium/slow development loops.

Seed the deterministic demo dataset into a running Compose deployment:

```powershell
.\scripts\seed-demo-data.ps1
```

Or seed it directly through the CLI against a running database:

```powershell
$env:DATABASE_URL='postgres://tessara:tessara@localhost:5432/tessara'
cargo run -p tessara-api -- seed-demo
```

The API serves the native Tessara interface at:

```text
http://localhost:8080/
```

Current Core/product routes are mounted at root-level paths. Organization and
Operations remain Core surfaces in the target architecture; Forms, Workflows,
Responses, Datasets, Components, and Dashboards are current in-process
transition surfaces that later become module-owned behind the same-origin
gateway:

```text
http://localhost:8080/organization
http://localhost:8080/forms
http://localhost:8080/workflows
http://localhost:8080/responses
http://localhost:8080/operations
http://localhost:8080/datasets
http://localhost:8080/components
http://localhost:8080/dashboards
```

Core administration destinations are mounted directly. The redundant exact
`/administration` landing route is intentionally absent. Module Management is
an `Admin`-group Core item gated independently by effective global
`modules:read`:

```text
http://localhost:8080/administration/users
http://localhost:8080/administration/node-types
http://localhost:8080/administration/roles
http://localhost:8080/administration/modules
http://localhost:8080/administration/modules/tessara.forms
```

Sprint 6A control-plane and compatibility API entry points are:

```text
GET  /api/admin/modules
GET  /api/admin/modules/{definition_id}
GET  /api/admin/modules/{definition_id}/descriptor
GET  /api/admin/navigation-policy
PUT  /api/admin/navigation-policy
GET  /api/shell/navigation
POST /api/platform/destinations/resolve
POST /api/platform/resource-references
POST /api/platform/resource-references/resolve
GET  /api/node-types/{node_type_id}/metadata-fields
```

The Organization metadata-field endpoint is a narrow read schema for effective
`hierarchy:read`; full node-type definitions and every metadata mutation remain
on `admin:all` administration APIs. Sprint 6A exposes Module Release/Instance
contract types but intentionally has no Release/Instance persistence or
mutation routes.

The former `/app` shell and JavaScript bridge assets have been retired. For user
testing, start the Compose stack and open the root URL in a browser. The local
launch helper now ensures a near-realistic Partner/Program/Activity/Session demo
hierarchy, published forms, sample responses, datasets, components, and a
compact dashboard path.
Stop and reset the local test deployment with:

```powershell
docker compose down -v
```

To rebuild and relaunch the user-testing stack with the latest UI/backend code:

```powershell
.\scripts\local-launch.ps1
```

The local shell now covers the main demo workflow surfaces through native
Leptos SSR routes:

- Route inventory at `/`, with direct navigation to product surfaces and to
  User Management, Roles & Access, Node Types, and Module Management.
- Root-level organization, forms, workflows, responses, dataset, dashboard,
  and component paths plus direct `/administration/*` management paths. The
  aggregate `/administration` landing route does not exist.
- Workflow revision, assignment, response, user administration, node type, role,
  and metadata management surfaces rebuilt as native Tessara routes.
- Dataset, component, and dashboard routes for analytical assets.

## First Target Slice

The first implementation milestone should prove an end-to-end thread:

1. Admin signs in.
2. Admin configures a two-level hierarchy.
3. Admin creates a metadata-backed node.
4. Admin builds and publishes a versioned form.
5. External user saves a draft and submits a valid response.
6. Analytics refresh materializes the submission.
7. A Dataset major-line contract feeds a ComponentVersion shown on a Dashboard.

## Implemented Slice Status

- Slice 0: workspace, crate scaffold, Docker Compose, local configuration.
- Slice 1: dev admin seeding, login, bearer-token sessions, `/api/me`.
- Slice 2: node type, relationship, metadata field, and node creation.
- Slice 3: form, form version, section, field, publish, and render endpoints.
- Slice 4: draft creation, draft value save, submit transition, audit events.
- Slice 5: manual analytics projection refresh into `analytics.*` tables.
- Slice 6: dataset definition and table execution.
- Slice 7: component and dashboard endpoints, dashboard discovery, and local
  dashboard inspection.
- Slice 8: builder lifecycle hardening, diagnostics, and capability/scope auth
  tests.
- Slice 9: RBAC reset to capability + scope + ownership and single baseline
  migration.
- Current baseline through Sprint 5A: native Leptos shell, application-grade
  Organization/RBAC and feature screens, Dataset/Component authoring, and
  Dashboard composition.
- Next phase: module contracts and the Core module control plane, followed by
  independent module runtime/database infrastructure and Dashboard extraction.

## Next Phase

Phase 6 establishes the modular application platform before external
applications depend on the current shared process and database layout. It adds
versioned module manifests, Core module administration, dynamic navigation and
Feature Declaration and security-capability discovery, typed cross-module references, same-origin module routing,
one database per module instance, and deterministic Blueprint/lockfile
composition. Dashboards will be the first current feature extracted as an
independently deployed full-stack module.

The Dockerfile uses BuildKit cache mounts for Cargo registry, git, and target
caches so repeated `docker compose up -d --build` test deployments avoid
rebuilding the entire Rust dependency graph after small frontend changes.

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
tessara-core
tessara-auth
tessara-hierarchy
tessara-forms
tessara-submissions
tessara-analytics
tessara-dashboards
tessara-db
tessara-jobs
tessara-web
```

The workspace has these crates scaffolded now. The current implementation keeps
the vertical slice logic in one runnable service while the module runtime is
built. These crate boundaries are useful extraction seams, not the target
deployment topology.

Domain rules should move into their owning Core or full-stack module boundary as
contracts stabilize. Current extracted examples:

- `tessara-core`: shared field type parsing and JSON value validation
- `tessara-dashboards`: dashboard composition rules
- `tessara-forms`: form version lifecycle and section/field compatibility rules
- `tessara-submissions`: draft/edit/submit workflow rules and required value checks

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
- ensures the UAT demo dataset is present in the local database

Useful options:

```powershell
.\scripts\local-launch.ps1 -FreshData
.\scripts\local-launch.ps1 -FollowLogs
.\scripts\local-launch.ps1 -SkipBuild
.\scripts\local-launch.ps1 -SkipSeed
.\scripts\local-launch.ps1 -ApiOnly
```

`-FreshData` also removes the local Postgres volume before relaunching.
`-FollowLogs` tails the Postgres and API container logs after startup.
`-SkipBuild` reuses the current API image.
`-SkipSeed` leaves the current local dataset untouched.
`-ApiOnly` delegates to the fast API refresh path instead of rebuilding the full Compose stack.

For the fast inner-loop Docker refresh path, use:

```powershell
.\scripts\local-refresh-api.ps1
```

That script:

- keeps Postgres running
- rebuilds only the API image unless `-SkipBuild` is supplied
- recreates only the API container
- waits for `/health` and `/` to return `200`
- reseeds demo data unless `-SkipSeed` is supplied

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

End-to-end coverage runs through Playwright:

```powershell
cd .\end2end
npm install
cd ..
cargo leptos end-to-end
```

Useful checks:

```powershell
.\scripts\validate.ps1
.\scripts\validate.ps1 -Fast
.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"
cargo clippy --workspace --all-targets -- -D warnings
.\scripts\smoke.ps1
.\scripts\smoke.ps1 -ComposeApi
```

`.\scripts\validate.ps1` is the standard pre-commit Rust validation path. It
runs formatting, API checks, the API SSR check, web checks, wasm hydrate checks,
and API/web tests sequentially so Windows Cargo builds do not fight over the
same artifact locks. Use `-Fast` for the inner loop when SSR and wasm hydrate
checks are not relevant to the change.

Testing should focus on behavior that protects domain and workflow boundaries:
validation rules, capability scope and ownership behavior, projection contracts,
component/dashboard composition, and end-to-end slice regressions. Avoid
placeholder tests that only assert generated boilerplate.

Permission-controlled surfaces and actions must be covered through Playwright
when executable. The standard `validate-e2e.ps1` / `npx playwright test` path
includes `end2end/tests/permissions.spec.ts`; update
`docs/playwright-permissions-scenarios.md` alongside new permission scenarios.

The default smoke script uses Docker for Postgres and runs the API locally with
`cargo run`. Use `.\scripts\smoke.ps1 -ComposeApi` to validate the fully
containerized Compose deployment path, including the API image.

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

Current transition routes are mounted at root-level paths. Organization remains
a Core route in the target architecture; Forms, Workflows, and Responses become
module-owned routes behind the same-origin gateway:

```text
http://localhost:8080/organization
http://localhost:8080/forms
http://localhost:8080/workflows
http://localhost:8080/responses
```

Administration routes are also mounted at root-level paths:

```text
http://localhost:8080/administration
http://localhost:8080/administration/users
http://localhost:8080/administration/node-types
http://localhost:8080/administration/roles
http://localhost:8080/datasets
http://localhost:8080/datasets
http://localhost:8080/components
http://localhost:8080/dashboards
```

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

- Route inventory at `/`, with direct navigation to product and administration
  surfaces.
- Root-level organization, forms, workflows, responses, administration, dataset,
  dashboard, and component paths.
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

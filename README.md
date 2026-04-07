# Tessara

Tessara is a configurable data platform for structuring, collecting, and
analyzing complex hierarchical data.

This repository is intended to become the Rust + Leptos rewrite described in
the migration blueprint. It should be developed as a domain-driven redesign,
not as a direct port of the legacy Django application.

## Intended Architecture

- Rust backend with `axum`, `sqlx`, and PostgreSQL
- Leptos SSR-first frontend
- PostgreSQL OLTP schema for workflow correctness
- Materialized analytics schema for reporting
- DataFusion for report and chart query execution

## Planned Crate Naming

```text
tessara-api
tessara-core
tessara-auth
tessara-hierarchy
tessara-forms
tessara-submissions
tessara-reporting
tessara-analytics
tessara-dashboards
tessara-db
tessara-jobs
tessara-web
```

The workspace has these crates scaffolded now. The first implementation keeps
the vertical slice logic in `tessara-api` so the local service is runnable while
the domain seams are still stabilizing.

Pure domain rules should move out of `tessara-api` as soon as their contracts
stabilize. Current extracted examples:

- `tessara-core`: shared field type parsing and JSON value validation
- `tessara-reporting`: missing-data policy parsing
- `tessara-dashboards`: chart type parsing
- `tessara-forms`: form version lifecycle and section/field compatibility rules
- `tessara-submissions`: draft/edit/submit workflow rules and required value checks

## Local Development

Copy the environment template if you want to run the API outside Docker:

```powershell
Copy-Item .env.example .env
```

Start the local stack:

```powershell
docker compose up --build
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

Useful checks:

```powershell
cargo fmt --all --check
cargo check -p tessara-api
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
.\scripts\smoke.ps1
.\scripts\smoke.ps1 -ComposeApi
.\scripts\rehearse-legacy-import.ps1
```

Testing should focus on behavior that protects domain and workflow boundaries:
validation rules, compatibility/missing-data behavior, projection/reporting
contracts, and end-to-end slice regressions. Avoid placeholder tests that only
assert generated boilerplate.

The default smoke script uses Docker for Postgres and runs the API locally with
`cargo run`. Use `.\scripts\smoke.ps1 -ComposeApi` to validate the fully
containerized Compose deployment path, including the API image.

The legacy import rehearsal script validates and dry-runs
`fixtures/legacy-rehearsal.json`, starts Docker Postgres, imports the fixture
through the CLI importer, starts the API locally, verifies the imported
report/dashboard path, and then tears down the test volume unless
`-KeepServices` is provided.

Seed the deterministic demo dataset into a running database:

```powershell
$env:DATABASE_URL='postgres://tessara:tessara@localhost:5432/tessara'
cargo run -p tessara-api -- seed-demo
```

Run the first legacy migration rehearsal fixture:

```powershell
cargo run -p tessara-api -- validate-legacy-fixture .\fixtures\legacy-rehearsal.json
cargo run -p tessara-api -- dry-run-legacy-fixture .\fixtures\legacy-rehearsal.json
$env:DATABASE_URL='postgres://tessara:tessara@localhost:5432/tessara'
cargo run -p tessara-api -- import-legacy-fixture .\fixtures\legacy-rehearsal.json
```

The API serves the admin workbench shell at:

```text
http://localhost:8080/
```

It also serves the first replacement-oriented application shell at:

```text
http://localhost:8080/app
```

Focused admin setup screens are available at:

```text
http://localhost:8080/app/admin
```

The focused migration workbench is available at:

```text
http://localhost:8080/app/migration
```

For user testing, start the Compose stack and open that URL in a browser. Use
the development login above, then click `Seed Demo` to populate the deterministic
hierarchy, form, submission, report, and dashboard example. Stop and reset the
local test deployment with:

```powershell
docker compose down -v
```

The local shell now covers the main demo workflow surfaces:

- Leptos SSR-rendered shell structure with the current JavaScript controller
  retained for immediate local workflow testing.
- Separate `/app` application shell focused on the published form, draft,
  submit, submission review, and report viewing workflow.
- Separate `/app/admin` setup shell focused on hierarchy, form, and report
  builder workflows without the full migration workbench surface.
- Separate `/app/migration` operator shell focused on legacy fixture example
  loading, validation, and dry-run rehearsal.
- Roadmap-aligned workflow sections and an in-browser user testing guide for
  the Compose deployment path.
- Admin read screens for hierarchy types, forms, reports, dashboards, nodes,
  and submissions.
- Admin builder controls for node types, forms, form versions, sections, fields,
  reports, charts, dashboards, and dashboard components.
- External workflow controls for draft creation, value save, submit, analytics
  refresh, report execution, and dashboard inspection.
- Selection-driven shortcuts for choosing node types, nodes, forms, form
  versions, sections, fields, reports, charts, and dashboards without copying
  raw IDs between most shell workflows.
- Report-builder controls for assembling binding JSON from selected form fields.
- Migration workbench controls for validating and dry-running pasted legacy
  fixture JSON through the API.

## Migration Planning

Slice 9 legacy mapping is tracked in
[docs/legacy-mapping.md](docs/legacy-mapping.md). Treat that document as a
behavior inventory and import-planning guide, not as a schema to reproduce.

## First Target Slice

The first implementation milestone should prove an end-to-end thread:

1. Admin signs in.
2. Admin configures a two-level hierarchy.
3. Admin creates a metadata-backed node.
4. Admin builds and publishes a versioned form.
5. External user saves a draft and submits a valid response.
6. Analytics refresh materializes the submission.
7. A compatibility-aware table report returns the data through DataFusion.

## Implemented Slice Status

- Slice 0: workspace, crate scaffold, Docker Compose, local configuration.
- Slice 1: dev admin seeding, login, bearer-token sessions, `/api/me`.
- Slice 2: node type, relationship, metadata field, and node creation.
- Slice 3: form, form version, section, field, publish, and render endpoints.
- Slice 4: draft creation, draft value save, submit transition, audit events.
- Slice 5: manual analytics projection refresh into `analytics.*` tables.
- Slice 6: report definition and DataFusion-backed table execution.
- Slice 7: dashboard/chart endpoints, report/dashboard discovery, and local
  dashboard inspection.
- Slice 8 start: builder lifecycle hardening, diagnostics, and admin auth tests.
- Slice 9: legacy behavior inventory, fixture coverage inventory, and remaining
  mapping-expansion risks.
- Slice 10: fixture validation, CLI import, clean Docker-backed rehearsal,
  imported report/dashboard inspection, and local validation workbench.
- Next phase: browser shell screens for admin builder, external submission
  workflow, report/dashboard builder workflows, and migration workbench.
- Next phase progress: Leptos shell foundation, selection-driven workflow
  shortcuts, rendered form submission controls, report binding builder controls,
  migration dry-run workbench endpoint, repeatable legacy import coverage, and
  extracted form/submission domain rules.

## Next Phase

The initial roadmap pass now proves the full migration thread from configurable
hierarchy through import rehearsal and reporting. The next milestone should
continue turning the API-first shell into a structured frontend and keep moving
stable domain contracts out of `tessara-api` into the domain crates.

The Dockerfile uses BuildKit cache mounts for Cargo registry, git, and target
caches so repeated `docker compose up -d --build` test deployments avoid
rebuilding the entire Rust dependency graph after small frontend changes.

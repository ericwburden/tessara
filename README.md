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
.\scripts\smoke.ps1
.\scripts\smoke.ps1 -ComposeApi
```

Testing should focus on behavior that protects domain and workflow boundaries:
validation rules, compatibility/missing-data behavior, projection/reporting
contracts, and end-to-end slice regressions. Avoid placeholder tests that only
assert generated boilerplate.

The default smoke script uses Docker for Postgres and runs the API locally with
`cargo run`. Use `.\scripts\smoke.ps1 -ComposeApi` to validate the fully
containerized Compose deployment path, including the API image.

Seed the deterministic demo dataset into a running database:

```powershell
$env:DATABASE_URL='postgres://tessara:tessara@localhost:5432/tessara'
cargo run -p tessara-api -- seed-demo
```

Run the first legacy migration rehearsal fixture:

```powershell
$env:DATABASE_URL='postgres://tessara:tessara@localhost:5432/tessara'
cargo run -p tessara-api -- import-legacy-fixture .\fixtures\legacy-rehearsal.json
```

The API also serves the first local shell at:

```text
http://localhost:8080/
```

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
- Slice 7 start: dashboard/chart endpoints and a minimal local admin shell.

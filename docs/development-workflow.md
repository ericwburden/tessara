# Tessara Development Workflow

This document separates the day-to-day development loops by speed and intent. The commands below describe the current single-service transition baseline. Phase 6 of the roadmap will add module-focused and full-composition workflows; until then these commands remain canonical and must stay working.

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
```

Notes:

- `-FreshData` removes the Postgres volume before relaunching
- `-SkipBuild` reuses the current API image
- `-SkipSeed` leaves the current demo dataset untouched
- `-ApiOnly` delegates to `local-refresh-api.ps1`

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

For routine UI, API, feature-crate, and Playwright expectation tweaks, the
default loop is the fast loop. Use host-run Tessara with Docker Postgres where
possible, or use
`.\scripts\local-refresh-api.ps1` / `.\scripts\local-launch.ps1 -SkipBuild`
when a containerized app refresh is enough.

Do a full teardown, rebuild, and redeploy only when the change touches Docker,
dependencies, migrations, release-build behavior, closeout validation, smoke, or
manual UAT. Routine UI copy, selector, layout, and Playwright expectation
changes should not pay the full rebuild cost.

When changing an existing extracted frontend feature area, prefer the focused
crate loop first, then run root integration checks before closeout. Keep current
root route, shell, authentication, hydration, document, CSS, and asset behavior
stable until the module gateway and SDK replace those responsibilities.

Do not assume that every new capability belongs in another root-integrated web
crate. New feature areas should be designed as full-stack module boundaries
owning UI, API, configuration, diagnostics, contracts, migrations, and data.

Once Phase 6 tooling exists, the development workflow must add:

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

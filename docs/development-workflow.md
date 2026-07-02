# Tessara Development Workflow

This document separates the day-to-day development loops by speed and intent.

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

When changing an extracted frontend feature area, prefer the focused crate loop
first, then run root integration checks before closeout. Major new frontend
feature areas should start in their own `tessara-web-*` crate with root
`tessara-web` retaining route adapters, shell, auth/session/navigation policy,
hydration, document integration, CSS, and assets.

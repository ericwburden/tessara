# Sprint 6B1 Plan: Container Deployment Foundation

Status: complete on 2026-07-23. Core implementation, the local vertical slice, curated-manifest persistence/readback, the approved Module Management UI deltas, and closeout verification are complete. Verified-container distribution and lifecycle management remain explicitly deferred to future platform work.

- Branch: `codex/sprint-6b1`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6b1`
- Roadmap source: `Sprint 6B1: Container Deployment Foundation Slice (Complete)`
- Follow-on: `Sprint 6B2: Secure Module Operation Slice (Next)`

## Sprint Summary

Sprint 6B1 proves the first independently deployed Tessara module boundary without building a custom container control plane. Docker Compose owns single-host container lifecycle, Traefik owns same-origin routing, and one PostgreSQL container hosts isolated logical databases for Core, deployment control, and every Module Instance. Tessara supplies only the product-specific curated deployment contract, thin operator CLI, database topology, module inventory/readback, Core fallback, and reference module.

The sprint is a full vertical slice: an operator can validate, plan, apply, observe, fail, restore, upgrade, and roll back a curated reference-module deployment. Cryptographic admission and production lifecycle management of verified third-party containers are future platform work. Sprint 6B2 adds enrollment and scoped cross-process user authorization after this deployment foundation is proven.

## Sprint Specifications

### Infrastructure Ownership

- `docker compose` is the only supported 6B1 container control mechanism and the deployment target is one host.
- Traefik uses its Docker provider with `exposedByDefault=false`; only generated, validated Tessara labels expose routes.
- `tessara-deploy` is a cross-platform Rust CLI, not a daemon. It validates the curated Tessara release contract, renders deterministic deployment input, and records sanitized receipts.
- `tessara-oci-v1` remains the stable container declaration. No Kubernetes schema, generic adapter framework, Supervisor, custom gateway, or duplicated health scheduler is added.

### Deployment Contract And CLI

- Define `TessaraDeploymentV1`, deterministic plan/diff output, `DeploymentReceiptV1`, and stable findings.
- Implement `validate`, `plan`, `apply`, `status`, and compatible `rollback` commands with structured JSON output and stable exit behavior.
- Retain digest identifiers, current revision binding, plan digest, expiry, and idempotency key for the curated development release without claiming cryptographic verification.
- Production confirmation, backup verification, verified-container admission, and destructive lifecycle controls are explicitly deferred.
- Core and browser processes never receive Docker-socket access.

### PostgreSQL Topology

- Run exactly one PostgreSQL container and cluster.
- Provision separate logical databases for Core, deployment control, and each Module Instance.
- Give each database distinct owner, migration, and runtime roles; revoke default public connection/schema privileges and grant only required access.
- Reject cross-module credentials, shared writable schemas, FDWs, and other cross-database shortcuts.

### Reference Module And Routing

- Ship `tessara.reference.scoped-records` as a minimal non-production full-stack process with migration command, SSR product/admin routes, API, probes, diagnostics, and its own logical database.
- Route Core and the module through Traefik under one origin.
- Convert missing/unhealthy module upstream behavior into a Core-rendered fallback without hiding Module Management or diagnostics.
- Persist and project real Module Release/Instance and deployment receipt state while keeping lifecycle dimensions independent.

### HTML/CSS Product Contract

- Create three coordinated runnable HTML/CSS review screens grounded in the completed 6A-UI application and its production stylesheet.
- Treat the current application as the authoritative baseline. The mockups are contextual review tools, not 1:1 application reproductions or wholesale implementation specifications; omitted icons, components, behaviors, and approved UI remain in place unless an explicit reviewed delta changes them.
- Use semantic HTML, representative assets, and narrowly scoped prototype CSS; no JavaScript and no PNG mockup authority. Mockup DOM structure, classes, spacing, and asset substitutions are illustrative unless specifically accepted as a change.
- Approve the coordinated directory, detail, and deployment suite and expand it into linked status, diagnostics, unavailable, timeout, failure, restore, upgrade, and rollback states.
- Treat the directory and detail screens as additive updates to the approved Sprint 6A-UI surfaces: retain useful shell, runtime context, exact identity, copy/view affordances, findings, navigation-policy access, declaration summaries, and established detail tabs unless a specific reviewed decision replaces them.
- Maintain a per-screen refinement record that separates proposed additions/changes from baseline-preservation notes. Only explicitly discussed and approved deltas enter application implementation scope; everything else remains unchanged.
- Validate both themes at 1280, 768, and 390 pixels, keyboard-only use, 200% zoom, long values, and overflow containment.
- Implement approved deltas with established native Leptos components and patterns. Compare the changed application with the existing screen plus its approved delta; do not recreate the mockup wholesale. Screenshots are comparison evidence only.

## Acceptance Criteria

1. A curated fixture validates and produces byte-stable plan/diff and receipt identities.
2. Apply-record generation fails closed for a mismatched plan, stale revision, expired plan, different installation, or incompatible release.
3. Compose starts Core, Traefik, the single PostgreSQL container, and the reference module with private networking, secrets, probes, restart behavior, and resource declarations.
4. The shared PostgreSQL cluster contains the exact Core, deployment-control, and reference-module logical databases, with separate roles and executable negative cross-database proof.
5. Traefik exposes only approved routes under the Tessara origin; stopping the reference module produces a contained Core fallback while Core administration remains available.
6. Module Management reads real Release, Instance, curated artifact provenance, receipt, configuration-presence, readiness, health, and diagnostics state without container mutation authority.
7. A compatible upgrade and rollback preserve Module Instance identity and database binding.
8. Each reviewed HTML/CSS screen has an explicit approved-delta record, and the responsive, accessible Leptos result preserves all baseline UI outside those deltas.
9. Existing product routes, authorization, native SSR ownership, hydration, and browser-console cleanliness remain intact.

## Manual Test Plan

1. Run `tessara-deploy validate` and `plan` against the curated fixture and inspect stable JSON output and diff.
2. Apply the fixture, inspect `docker compose ps`, Traefik routing, exact database inventory, roles, and Module Management readback.
3. Navigate to the reference module through the public Tessara origin and inspect its product, administration, health, and diagnostics surfaces.
4. Stop the reference-module container, confirm the Core fallback and continued Module Management access, then restore it.
5. Apply the compatible reference-module upgrade and rollback; confirm stable instance/database identity and retained data.
6. Review the existing application, approved delta record, mockup, and implementation together at desktop, tablet, and mobile widths in both themes with keyboard and 200% zoom; confirm unchanged UI was retained.

## Automated Test Plan

- Contract/unit: deterministic deployment mapping, canonical digests, findings, destructive classification, receipt binding, redaction, and tool-output parsing.
- CLI fixtures: deterministic plan/apply-record inputs prove exact binding and fail-closed behavior without mutating a live installation.
- Integration: Compose startup ordering, migration failure, probes, secrets, private networks, stop/recovery, compatible upgrade, and rollback.
- Database: exact single-container topology, logical database/role creation, runtime versus migration access, and prohibited cross-database access.
- Gateway: default-deny exposure, same-origin route ownership, timeout/unhealthy fallback, and Core administration availability.
- Web/API: Release/Instance/receipt projections, lifecycle separation, SSR/no-JavaScript usefulness, permissions, hydration, console, accessibility, and responsive containment.
- Required planned commands:
  - `cargo fmt --all`
  - `cargo test -p tessara-module-contract`
  - `cargo test -p tessara-deploy`
  - `cargo test -p tessara-reference-scoped-records`
  - `cargo test -p tessara-api` with isolated disposable `TEST_DATABASE_URL`
  - `cargo test -p tessara-web`
  - `npm --prefix .\end2end test` for the direct repository-owned runner, plus
    `.\scripts\validate-e2e.ps1 ...` for canonical manifest-bound acceptance
    evidence; bare root `npx playwright test` remains unsupported
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Closeout Result

- The development schema is squashed into one authoritative `001_baseline.sql`; migrations 002–004 were folded into that baseline and removed before final fresh-stack verification.
- The curated install, compatible upgrade, rollback, module stop/fallback, restart, same-origin routing, database isolation, and retained-data lifecycle all pass against the local Compose deployment.
- Module Management uses one shared parameterized detail-page structure and typed projections for transition contributions and independently deployed modules. The directory, detail tabs, responsive selector/action menu, empty states, status semantics, tooltips, and deployment ledger implement only the reviewed Screen A–C deltas.
- Curated manifest bytes are persisted and downloadable. The rendered source digest is verified against those exact bytes in browser acceptance.
- Formatting, API, web, deployment-contract, deployment CLI, reference-module, smoke, UAT, and canonical Playwright acceptance checks pass with the final fresh deployment evidence.
- Root-level `npx playwright test` is not a supported runner because Playwright is package-local under `end2end`; `scripts/validate-e2e.ps1` is the authoritative manifest-bound closeout runner.

## Ordered Implementation Plan

1. Amend roadmap and freeze the infrastructure responsibility/contract decisions.
2. Build and review the three-screen HTML/CSS Module Management suite; record and approve bounded per-screen deltas without treating the mockups as full-screen replacement specifications.
3. Add deployment contract types and deterministic validation/plan/receipt fixtures.
4. Add the thin `tessara-deploy` command surface and exact external-tool boundary tests.
5. Add Compose, Traefik, shared-cluster database provisioning, and secret/network topology.
6. Add real Release/Instance/receipt persistence and Core read projections.
7. Implement the minimal Scoped Records process, database, migrations, probes, and diagnostics.
8. Implement Traefik routing, Core fallback, and only the approved Module Management UI deltas using the existing application surfaces and shared components.
9. Prove failure/recovery, compatible upgrade/rollback, isolation, regression, and final retained evidence.

## Dependencies And Blockers

- Docker Engine with the Compose plugin and PostgreSQL 17 tooling must be available at supported versions.
- Cryptographic publisher verification and production management of verified module containers are not Sprint 6B1 dependencies.
- Windows may retain the now-empty historical `tessara-sprint-6a-ui` directory until the external process holding it open exits; it is not a registered worktree and does not block Sprint 6B1.
- Kubernetes, Helm, multi-host scheduling, administrator enrollment, downstream scoped authorization, and complete Scoped Records business behavior are Sprint 6B2 or later scope.

## Implementation Evidence

Implemented in `codex/sprint-6b1`:

- deterministic deployment, plan, applied-component, release, instance, verification, receipt, and rollback contracts;
- the cross-platform `tessara-deploy` command surface with JSON output, exact plan matching, expiry checks, publisher-evidence checks, monotonic revisions, stable instance identity, and sanitized receipt publication;
- a single-host Compose topology with Traefik default-deny routing, one PostgreSQL cluster, isolated Core/deployment/module databases and roles, one-shot migrations, Core, and the independently deployed Scoped Records reference module;
- Core persistence and readback for Module Releases, Module Instances, receipt history, deployment downloads, configuration/diagnostics state, and contained module-unavailable fallback behavior;
- the three approved Module Management deltas, implemented additively against the existing Leptos screens rather than as a mockup rewrite;
- live install, upgrade, rollback, database-isolation, outage/fallback, restart, and retained-data acceptance scripts.

Validation completed:

- `cargo test -p tessara-module-contract -p tessara-deploy`
- `cargo test -p tessara-reference-scoped-records`
- `cargo test -p tessara-web`
- `cargo check -p tessara-api -p tessara-web --all-targets`
- `cargo fmt --all -- --check`
- `npm run tailwind:build`
- the existing Module Management Playwright suite: 5 passed
- `scripts/test-sprint-6b1-contract.ps1`
- `scripts/test-sprint-6b1-live.ps1 -BaseUrl http://127.0.0.1:8180`
- legacy smoke and Sprint UAT scripts against the canonical application stack

Closeout uses one squashed `001_baseline.sql`; migrations 002–004 were implementation-time steps and are not shipped as closing deployment inputs.

The 6B1 fixture is a curated Tessara development release. It retains exact digest and publisher-provenance fields for forward compatibility but does not claim cryptographic verification. Verified-container admission, registry/catalog distribution, signature policy, and production destructive lifecycle management are recorded as future platform work rather than Sprint 6B1 closeout gates.

The independently deployed detail projection now persists the complete validated curated `ModuleManifestV1` together with the sanitized release/instance/receipt/configuration/diagnostics read model. Descriptor download returns that persisted manifest, and the Declarations, Contracts, Capabilities, Dependencies, Resources, and Navigation tabs project the same authoritative content rather than parallel placeholder objects. Final closeout remains contingent on the recorded fresh-stack verification suite.

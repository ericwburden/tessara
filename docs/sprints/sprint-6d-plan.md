# Sprint 6D Plan: Canonical Module SDK And Runtime Extraction

Status: kicked off on 2026-07-30 from clean `main` commit
`89f133f683c1fb1c549b85f57a08098077ac3fba`.

- Branch: `codex/sprint-6d`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6d`
- Roadmap source:
  `Sprint 6D: Canonical Module SDK And Runtime Extraction Slice (Next)`
- Predecessor:
  `Sprint 6C: Independently Deployed Dashboard Module Slice (Complete)`
- Accepted architecture decision:
  `docs/architecture/module-sdk-source-ownership.md`
- Planned verification record: `docs/sprints/sprint-6d-verification.md`
- Ownership inventory:
  `docs/architecture/module-sdk-ownership-inventory.md`

## Sprint Summary

Sprint 6D extracts the policy-neutral platform code required by every
independently deployed Tessara module into canonical, independently versioned
contract, runtime, UI, asset, and conformance boundaries. The same canonical
source may be statically linked or otherwise compiled into Core and multiple
module images. Shared source is not copied, and a module does not link the Core
application, root route tree, Core API state, Core-private DTOs, or another
module's product implementation.

The sprint proves the boundary with a minimal reference module that renders a
complete same-origin SSR document from authenticated `ShellContextV1`, serves
its own frontend assets, exposes the shared configuration and operational
contracts, degrades coherently, and passes a reusable conformance testkit
without depending on `tessara-web` or `tessara-api`. Existing Core, Dashboard,
and Scoped Records product behavior remains unchanged. Dashboard product
adoption and removal of its root-web dependency belongs to Sprint 6E.

## Sprint Specifications

### Canonical Ownership Inventory

- Inventory shared and duplicated behavior across Core, Dashboard, Scoped
  Records, `tessara-module-contract`, `tessara-web`, `tessara-web-ui`,
  `tessara-web-http`, Dockerfiles, Compose profiles, bootstrap scripts, and
  conformance/acceptance harnesses.
- Record one canonical owner, consumers, current forbidden dependencies,
  target package, wire/runtime status, and Sprint 6D disposition for every
  retained behavior.
- Distinguish platform contracts, module runtime, shared UI/design-system,
  testkit, Core-owned policy, and module-owned product behavior. Do not use a
  broad shared crate as a holding area for business logic.

### Platform Contract Boundary

- Retain `tessara-module-contract` as the policy-neutral public contract owner
  or split it only where dependency or target constraints require a narrower
  package.
- Canonically own manifests, Shell Context, scope-bound grants/decisions,
  semantic destinations, typed resource references, stable error envelopes,
  configuration/control protocol wire types, and generated-schema/client
  conventions.
- Keep verification separate from Core authorization decisions: SDK code may
  validate signed context, audience, installation, action, freshness, and
  integrity, but Core remains the decision owner.
- Define package and wire semantic-versioning rules, manifest declarations,
  compatibility ranges, the supported-version window, deprecation, and
  vulnerable/unsupported release reporting.

### Module Runtime Boundary

- Establish a canonical policy-neutral module runtime package for server
  startup, configuration validation/apply plumbing, private security-state
  application, health, readiness, sanitized diagnostics, correlation/tracing,
  graceful shutdown, and standard operational errors.
- Extract reusable implementations from Dashboard, Scoped Records, Core, and
  scripts instead of copying their source. Module-specific configuration
  schemas, product routes, repositories, and diagnostics facts remain with the
  owning module.
- Expose narrow construction/configuration interfaces so a module supplies its
  identity, manifest, routes, configuration validator, and diagnostics
  provider without the runtime recognizing a module definition ID.
- Keep the package graph free of `tessara-api`, root `tessara-web`, Core
  application state, product modules, SQLx, or Axum dependencies unless a
  responsibility explicitly requires them; record and test every permitted
  infrastructure dependency.

### Module UI And Asset Boundary

- Establish a canonical SSR-compatible module UI package that owns
  complete-document shell rendering from normalized Shell Context, shared UI
  primitives, design tokens, theme behavior, accessibility behavior, standard
  unavailable/restricted states, and module asset conventions.
- Separate shell rendering inputs from Core session and application state. The
  package consumes authenticated wire/context projections and does not make
  authorization decisions.
- Define module-owned CSS, JavaScript/WASM, icon, content-hash, cache-header,
  and hydration conventions. Repeated compiled assets in release images are
  valid when generated from the canonical source.
- Preserve the current Core shell while introducing the shared source seam.
  No Core, Dashboard, or Scoped Records product-flow redesign is in scope.

### Package And Source Enforcement

- Extend or add automated Cargo metadata graph checks for both native and
  `wasm32-unknown-unknown` targets where applicable.
- Reject paths from canonical SDK/runtime/testkit packages to `tessara-api`,
  root `tessara-web`, root route/state modules, Dashboard or Scoped Records
  product code, and any other feature implementation.
- Add source assertions for Core-private state/DTO imports, module definition
  branching, copied shell/runtime implementations, and product-specific
  terminology in policy-neutral packages.
- Record allowlists narrowly and require a reviewed architecture change to
  broaden them.

### Reference Module And Testkit

- Add a minimal first-party reference/fixture module with its own manifest,
  route contribution, configuration schema, security capability, health,
  readiness, diagnostics, complete-document SSR route, and module-owned
  versioned assets.
- The reference module must build and test without `tessara-web` or
  `tessara-api`. It must not become a product module or add domain scope beyond
  proving the roadmap contract.
- Add a reusable testkit for manifest and compatibility validation,
  authenticated/tampered/wrong-audience context, configuration normalization,
  health/readiness, sanitized diagnostics, SSR shell, asset headers,
  unavailable state, graceful shutdown, and package independence.
- Route the fixture through the same-origin gateway in the Sprint 6D
  deployment profile and preserve the Core-owned fallback when it is stopped.

### Module Authoring And Upgrade Guidance

- Document how a module declares, consumes, tests, builds, and upgrades
  canonical contract/runtime/UI/testkit packages.
- Document that shared-source changes affect a deployed module only after that
  module publishes and deploys a new immutable release image.
- Document compatibility-window, deprecation, security advisory, and affected
  Module Release inventory expectations.
- Update the independent-module pathway with the canonical package graph and
  reference-module conformance commands, without pre-implementing Dashboard
  adoption from Sprint 6E.

## Acceptance Criteria

1. Every retained shared platform behavior in scope has one documented
   canonical source owner and no unexplained copied implementation.
2. Canonical contract, runtime, UI/design-system, asset, and testkit
   responsibilities are explicit, independently testable, and free of module
   product semantics.
3. Package/source audits reject paths from the canonical SDK/runtime/testkit
   to Core application code, root `tessara-web`, Core-private DTO/state, or
   module implementations on every relevant target.
4. The reference module builds and runs without `tessara-web` or
   `tessara-api`.
5. An authenticated user can load the reference module's complete same-origin
   SSR document with coherent shell, navigation, theme, and standard states;
   the module serves its own versioned frontend assets.
6. Invalid, tampered, expired, wrong-installation, wrong-audience, and
   unauthorized context fails closed without exposing reusable credentials or
   protected destination existence.
7. Configuration, security-state application, health, readiness, sanitized
   diagnostics, correlation, and graceful shutdown pass through the canonical
   runtime and reusable conformance testkit.
8. Stopping the reference module produces the Core-owned fallback while Core,
   Dashboard, Scoped Records, and unrelated routes remain available.
9. SDK/runtime versions, manifest compatibility, support window, deprecation,
   and unsupported/vulnerable release handling are documented and covered by
   compatibility tests.
10. Existing Core, Dashboard, and Scoped Records product, authorization,
    module-management, SSR, API, smoke, UAT, and Playwright behavior remains
    green; Dashboard retains its intentional root-web transition for Sprint
    6E.
11. The Sprint 6D deployment is reproducible from an exact clean source commit,
    records commit/tree/dirty/image provenance, and its bootstrap is
    idempotent.
12. The module-authoring workflow lets a future module adopt the canonical
    packages without copying source or adding definition-specific Core
    Module Management logic.

## Manual Test Plan

1. Launch the source-exact Sprint 6D stack and bootstrap it twice. Confirm the
   second materialization is a no-op and the reference Module Instance,
   configuration, routes, and receipt remain stable.
2. Sign in as an administrator and open the reference route through the
   same-origin gateway. Verify complete SSR shell chrome, navigation, theme,
   route content, versioned module assets, configuration, health, and
   diagnostics.
3. Disable and re-enable the reference module through shared Module
   Management. Confirm product navigation and route state change without a
   definition-specific Core screen.
4. Stop the reference module. Confirm its route receives the Core-owned
   fallback, Module Management remains usable, and Core, Dashboard, and Scoped
   Records routes continue to work. Restart it and confirm identity and
   configuration are retained.
5. Use a constrained non-admin actor with the reference capability in one
   authorized scope and without it in another. Confirm the allowed route works
   and known/random unauthorized destinations receive the same non-disclosing
   outcome.
6. Exercise tampered, expired, wrong-installation, and wrong-audience Shell
   Context fixtures through the conformance harness and confirm fail-closed
   results without credential or policy leakage.
7. Inspect response headers and image contents to confirm the reference module
   serves its own content-hashed/cache-controlled assets compiled from the
   canonical UI source.
8. Review `cargo metadata` evidence for native and WASM targets and confirm the
   reference module and canonical packages have no path to forbidden Core/root
   or module implementation packages.
9. Exercise keyboard, no-JavaScript, light/dark theme, 1280 px, 768 px, and
   390 px presentation for the reference document and fallback state.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-module-contract`
- targeted tests for each new module runtime, UI, and testkit package
- reference-module native build/test without root application features
- reference-module `wasm32-unknown-unknown` check when hydration is included
- native and WASM package/source boundary audit commands
- context verification tests for valid, tampered, expired, wrong-installation,
  wrong-audience, unauthorized, and nondisclosing known/random cases
- runtime conformance tests for configuration, security state, health,
  readiness, diagnostics redaction, tracing/correlation, and graceful shutdown
- SSR and asset tests for complete-document shell content, accessibility
  landmarks, content hashes, cache headers, no-JavaScript usefulness, and
  standard fallback states
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- existing Dashboard and Scoped Records targeted suites
- `docker compose -f deploy\sprint-6d\compose.yaml config --quiet`
- `.\scripts\bootstrap-sprint-6d-deployment.ps1` twice
- `.\scripts\local-launch.ps1` remains a required compatibility check; the
  Sprint 6D Compose profile is authoritative for retained reference-module
  evidence
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- `npm --prefix .\end2end test`
- retained Playwright validation through `.\scripts\validate-e2e.ps1` against
  the source-exact Sprint 6D deployment evidence

## Closeout-Readiness Plan

- Deployment profile: `deploy/sprint-6d/compose.yaml`, extending the retained
  Sprint 6C topology with only the minimal reference-module and generic route
  registration needed for this sprint.
- Idempotent materialization:
  `.\scripts\bootstrap-sprint-6d-deployment.ps1`; run it twice and retain the
  first apply plus second no-op evidence.
- Source provenance: every rebuilt release image records exact
  `TESSARA_SOURCE_COMMIT`, `TESSARA_SOURCE_TREE`,
  `TESSARA_SOURCE_DIRTY`, release profile, and immutable image digest. Retained
  evidence must bind the running images to one committed clean source.
- Migration checkpoint: no product schema is planned. If any Core or fixture
  schema changes become necessary, squash the development migration before
  closeout and apply every affected baseline to disposable empty databases.
  Otherwise record a deliberate no-schema-change checkpoint and rerun the
  existing Core, Dashboard, Scoped Records, deployment-control, and fixture
  fresh-baseline proofs.
- Final evidence directory: `artifacts/sprint-6d-closeout/`, containing at
  least `deployment-fresh.json`, `bootstrap-first.json`,
  `bootstrap-second-noop.json`, `sdk-ownership.json`,
  `package-boundaries.json`, `reference-conformance.json`,
  `smoke-fresh.json`, `uat-fresh.json`, `e2e-fresh.json`,
  `e2e-fresh.summary.json`, and required hash/provenance sidecars.
- Harness reconciliation: update smoke, UAT, Playwright, navigation/module
  inventory, capability, manifest, Compose, bootstrap, and evidence assertions
  in the same change that adds the reference module. Prefer semantic
  assertions; keep any contractually exact inventory in one shared fixture.
- Non-admin proof: use one constrained actor whose reference capability is
  authorized only at one declared scope. Prove allowed access plus identical
  restricted results for known and random unauthorized destinations and
  wrong-audience context.
- Exit-condition mapping:
  - build/run without root dependencies: manual tests 2 and 8; reference build
    and native/WASM package audits;
  - same-origin coherent shell: manual tests 2, 7, and 9; SSR, asset, smoke,
    UAT, and Playwright assertions;
  - authenticated and unavailable states: manual tests 3–6; context,
    nondisclosure, outage, and fallback automation;
  - shared conformance suite: manual test 6; runtime/UI/testkit conformance
    outputs;
  - one canonical implementation: manual test 8; ownership inventory, source
    duplicate scan, and package/source boundary evidence;
  - repeated code/assets in the module image: manual test 7; image inspection,
    asset hashes, provenance, and module-only build evidence.
- Before implementation-complete status, run a closeout-readiness audit for
  acceptance mapping, Compose validity, provenance inputs, bootstrap
  idempotency, migration status, current harness inventory, and clean targeted
  suites. Commit all implementation and harness corrections before the one
  retained source-exact cycle.

## Ordered Implementation Plan

1. Produce the canonical ownership and dependency inventory, including current
   Dashboard/Scoped Records/root-web coupling and exact target package
   boundaries.
2. Strengthen the platform contract boundary and compatibility/versioning
   policy without importing runtime or UI dependencies.
3. Extract the policy-neutral module runtime and move reusable operational
   implementations behind narrow construction/provider interfaces.
4. Extract complete-document shell/UI primitives and module-owned asset
   conventions while keeping Core rendering behavior stable.
5. Implement native/WASM package-graph and source audits before broad adoption
   can introduce accidental dependencies.
6. Add the minimal reference module and reusable testkit, then prove direct
   build, context verification, runtime operations, SSR shell, asset, outage,
   and graceful-shutdown behavior.
7. Add Sprint 6D Compose, bootstrap, provenance, constrained-actor, smoke,
   UAT, and Playwright coverage in the same slices that alter deployment and
   route inventories.
8. Document module authoring, package upgrades, compatibility support, and
   unsupported/vulnerable release handling; update the reusable independent
   module pathway.
9. Run the closeout-readiness audit, record the migration checkpoint, commit
   clean implementation source, and execute one retained source-exact
   closeout cycle.

## Dependencies And Blockers

- Sprint 6C's independent Dashboard/Scoped Records processes, database
  isolation, Module Management, Shell Context, authorization exchange,
  gateway, Compose, bootstrap, and conformance evidence are required and
  complete.
- Existing `tessara-module-contract`, `tessara-web-ui`, `tessara-web-http`,
  root shell components, module control endpoints, manifests, and boundary
  audit scripts are the extraction inputs, not automatically the final
  package design.
- `tessara-web-ui` currently depends on `tessara-core`; shell components and
  root application state remain coupled inside `tessara-web`.
  `tessara-dashboard-module` still depends on root `tessara-web`. These are
  known transition edges, not kickoff blockers.
- Sprint 6D must not remove the Dashboard root-web edge, migrate Dashboard
  product code/assets, or prove Dashboard-only rollout; those belong to Sprint
  6E.
- Blueprint automation, physical Components extraction, third-party catalog
  verification, and unrelated product behavior are later roadmap scope.
- No blocking conflict is known at kickoff. Any required expansion into Core
  policy or module product semantics must stop for an explicit roadmap or
  architecture decision.

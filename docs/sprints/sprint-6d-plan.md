# Sprint 6D Plan: Canonical Module SDK And Runtime Extraction

Status: implementation complete; retained closeout evidence pending.

- Branch: `codex/sprint-6d`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6d`
- Base decision commit:
  `89f133f647c87e5dfa72891d23a06a012cbd0b05`
- Kickoff commit:
  `d55b56b4dddbf6c80eb6fca3e046726ce274dbba`
- Initial inventory commit:
  `1b86fa658115547f9f762bae9d1814318cd94f16`
- Roadmap authority:
  `Sprint 6D: Canonical Module SDK And Runtime Extraction Slice (Next)`
- Implementation contract:
  [Module SDK Implementation Contract](../architecture/module-sdk-implementation-contract.md)
- Ownership inventory:
  [Module SDK Canonical Ownership Inventory](../architecture/module-sdk-ownership-inventory.md)
- Planned verification:
  [Sprint 6D Verification](./sprint-6d-verification.md)

## Outcome And Scope

Sprint 6D establishes one current, independently versioned source for shared
module contracts, runtime integration, complete-document shell/UI rendering,
frontend assets, and conformance support. The source may be compiled into many
images, but modules do not link Core, root web, Core-private state/DTOs, or
another module implementation.

Tessara is pre-production and advances functionality by pure fast-forward
changes. The sprint updates manifests, packages, consumers, fixtures,
deployment baselines, and tests together. It does not retain obsolete
manifest readers, version windows, crate facades, renderer bridges, or test
expectations for backwards compatibility.

The sprint proves the boundary with:

- the four canonical packages;
- one generic manifest-driven document/asset route seam;
- a minimal non-product reference module;
- Scoped Records adoption of runtime/UI;
- exact native/WASM package and source audits;
- executable deployment and closeout evidence from the first implementation
  slice.

Dashboard adoption, its root-web removal, Dashboard-owned frontend assets, and
Dashboard-only upgrade/rollback proof remain Sprint 6E.

## Required Implementation

### Package and source ownership

- Implement the exact contract/runtime/UI/testkit graph in the implementation
  contract.
- Keep pure signed-envelope and Shell Context validation in contract; move
  header extraction, environment/verifier construction, tracing, startup, and
  shutdown to runtime.
- Move complete-document rendering, pure shell presentation, policy-neutral
  primitives, tokens, shared CSS, accessibility behavior, and asset
  conventions to module UI.
- Move policy-neutral grid geometry from the historically named
  `tessara-core` crate to `tessara-module-contract`, move shared placement DOM
  mechanics to `tessara-module-ui`, keep Dashboard and Forms policy adapters
  with their respective product owners, update consumers directly, and delete
  `tessara-web-ui`.
- Retain `tessara-web-http` as an independent browser-transport leaf.
- Delete `ShellContentV1`, its media type, Core bridge, hydration bootstrap,
  renderer branches, and tests.
- Adopt runtime/UI in Scoped Records without moving its product rules,
  persistence, migrations, product routes, or product diagnostic facts.
- Leave Dashboard's root-web/runtime transition visible and nonconforming for
  Sprint 6E.

### Current public contracts

- Replace `ModuleManifestV1` with the sole current exact-version manifest and
  update every repository consumer and production manifest atomically.
- Add standard configuration validation, projected security state,
  health/readiness, sanitized diagnostics, and runtime error envelopes.
- Implement generic providers for definition/manifest/routes/assets,
  configuration, security state, readiness, and diagnostics.
- Define a normalized presentation model derived only from verified Shell
  Context plus route metadata.
- Keep module configuration semantics and storage with the module; keep
  authorization and navigation policy with Core.
- Support contract on native/WASM, runtime on native, UI through explicit
  `ssr`/`hydrate` features, and target-gated reference builds.

### Generic module documents and assets

- Extend the current manifest with validated GET/HEAD browser path templates,
  required capability, authorization action/contract, and optional
  Organization-scope parameter.
- Add one generic Core document proxy driven by accepted manifests and the
  module endpoint registry.
- Authenticate and authorize in Core, sign short-lived Shell Context and grant
  envelopes, remove browser credentials, and forward only safe metadata.
- Generate contributed navigation from the accepted current manifest instead
  of adding reference-specific Core branches.
- Serve module-owned immutable assets through definition/release/content-hash
  paths without browser credentials.
- Render the Core-owned authenticated fallback on timeout, connection failure,
  or module 5xx; pass through normalized module 4xx responses.
- Do not add generic product API or mutation proxying.

### Reference module

- Add definition `tessara.reference.module-sdk`, release `1.0.0`, and
  capability `tessara.reference.module-sdk:read`.
- Serve `/reference/module-sdk` and the authorized scope probe
  `/reference/module-sdk/scopes/{organization_id}`.
- Require at least one authorized capability binding for the root and exact
  Organization scope for the probe.
- Return identical `403 module action unavailable` envelopes for known and
  random unauthorized Organization IDs.
- Declare one non-product conformance feature/behavior contract, no product
  resources, no product records, and no cross-module dependency.
- Normalize only a `display_label` configuration.
- Persist configuration and projected security state atomically in one exact
  current JSON schema on a module-owned volume; persist no signed/user/secret
  material and claim no PostgreSQL database binding.
- Supply liveness, readiness, sanitized diagnostics, complete SSR, canonical
  CSS, minimal hydration, immutable assets, standard states, outage behavior,
  and bounded graceful shutdown.

### Version and security policy

- Use exact current package/protocol versions; the support window has width
  one.
- Update repository consumers atomically for every breaking pre-production
  change.
- Record deprecation in architecture/release history, but retain no obsolete
  API solely for compatibility.
- Reject older or newer tuples at manifest import, bootstrap, enablement, and
  upgrade.
- Inventory the exact linked SDK versions for every Module Release.
- Block unsupported or critical/high-vulnerable versions without a
  grandfathered lifecycle exception; report moderate/low advisories.
- Require a rebuilt immutable module image to deliver any SDK/security fix.

## Dependency And Source Rules

Allowed Tessara workspace edges are exactly:

| Source | Allowed Tessara dependencies |
| --- | --- |
| `tessara-module-contract` | none |
| `tessara-module-runtime` | contract |
| `tessara-module-ui` | contract |
| `tessara-module-testkit` | contract, runtime, UI |
| `tessara-reference-module-sdk` `ssr` graph | contract, runtime, UI |
| `tessara-reference-module-sdk` `hydrate` graph | contract and UI; runtime and native persistence are forbidden |
| `tessara-reference-module-sdk` dev graph | applicable production graph plus testkit |
| Scoped Records | contract, runtime, UI, and its module-owned infrastructure |
| root `tessara-web` | contract, UI, Core/product web dependencies; it is not an SDK package |
| `tessara-web-http` | no Tessara dependency unless contract wire types are required; browser transport only |
| `tessara-web-ui` | deleted; no dependency or facade remains |

Every other direct or transitive edge from a canonical package is forbidden.
Native and WASM audits also reject:

- Core/root/product package imports;
- Core-private DTO or application-state symbols;
- product Module Definition IDs or product DTO terminology in canonical
  production source;
- definition-ID branches in runtime/UI/testkit;
- copied shell, verifier-construction, control-route, tracing, startup, or
  shutdown implementations.

Dashboard's Sprint 6E debt is an expected failing transition finding, not a
canonical allowlist entry.

## Acceptance Criteria

1. Every shared behavior in Sprint 6D has one documented canonical owner and
   source; no obsolete facade or bridge remains.
2. The four canonical packages have the exact allowed graph on every relevant
   target and contain no Core or product semantics.
3. One current manifest and exact platform tuple replace the development v1
   shape everywhere in the repository.
4. The reference module builds native and WASM without root web, Core API,
   Core application state, SQLx, or a product implementation.
5. An authenticated actor can load the reference complete document through
   its normal same-origin navigation with coherent shell, theme, accessibility,
   standard states, and module-owned assets.
6. Invalid, tampered, expired, wrong-installation, wrong-audience, and
   unauthorized projections fail closed without credential, policy, or
   protected-destination disclosure.
7. Configuration, projected security state, readiness, liveness, diagnostics,
   correlation, and shutdown flow through canonical runtime providers.
8. Stopping the reference module produces the Core fallback while Core,
   Dashboard, Scoped Records, Module Management, and unrelated routes remain
   available.
9. Scoped Records uses canonical runtime/UI while preserving product and
   authorization behavior.
10. Dashboard retains its Sprint 6C product behavior and remains an explicit
    Sprint 6E source/runtime transition finding.
11. Exact-version and security inventory reject unsupported or blocked
    releases without compatibility fallback.
12. The Sprint 6D deployment and evidence bind clean committed source to
    immutable images and repeat bootstrap as a verified no-op.
13. A future module can adopt the current packages and generic document route
    without adding definition-specific Core UI, routing, or control logic.

## Manual Verification

1. Review the ownership and package-boundary outputs and confirm the Dashboard
   finding is visible rather than accepted as conforming.
2. Build the reference native and hydrate targets directly and inspect their
   dependency graphs.
3. Launch a source-exact fresh Sprint 6D deployment and run bootstrap twice;
   compare identity, configuration, route, manifest, and receipt state.
4. Sign in as an administrator and navigate to the reference route through
   normal navigation. Inspect complete HTML, shell, theme, assets,
   configuration, health, and diagnostics.
5. Exercise light, dark, and system theme; keyboard operation; no JavaScript;
   and 1280 px, 768 px, and 390 px layouts.
6. Use one constrained actor with the reference capability in one Organization
   scope. Confirm the root and authorized probe work, while known and random
   unauthorized scopes return indistinguishable results.
7. Disable/re-enable the reference module through generic Module Management.
8. Stop/restart the reference process. Confirm the Core fallback, unrelated
   routes, retained configuration/security state, and recovery.
9. Inspect the reference image and responses for exact SDK versions,
   provenance, content hashes, cache headers, and module-owned assets.
10. Run the shared conformance command and inspect every retained evidence
    record before closeout.

## Automated Verification

Automated changes require explicit user approval before their files are
edited. The planned coverage is:

- current-manifest validation and exact-version mismatch rejection;
- targeted contract/runtime/UI/testkit/reference/Scoped Records suites;
- native and WASM package/source boundary audits;
- valid, tampered, expired, wrong-installation, wrong-audience, unauthorized,
  and nondisclosing known/random context/grant cases;
- configuration persistence and read-back;
- liveness/readiness and sanitized diagnostics;
- complete-document SSR, landmarks, no-JavaScript usefulness, hydration
  stability, asset hashes, and cache headers;
- graceful shutdown and outage/Core-fallback containment;
- existing Core, web, Dashboard, Scoped Records, smoke, UAT, and Playwright
  regression coverage;
- responsive, theme, keyboard, authorization, and unavailable-state
  Playwright coverage;
- Compose validation, fresh baseline, source/image provenance, bootstrap
  first-apply, and bootstrap second-run no-op evidence.

The approved commands and file-level test changes are recorded in the
[Sprint 6D Verification](./sprint-6d-verification.md) document before test
implementation.

## Deployment And Closeout

- `deploy/sprint-6d/compose.yaml` extends Sprint 6C with the reference state
  initializer/runtime, named state volume, endpoint registration, and
  provenance-labelled image.
- The reference state initializer creates or validates the exact current JSON
  schema. Incompatible pre-production fixture state is recreated, not
  migrated.
- Any affected development database baseline is advanced/squashed directly
  and proven against disposable empty databases. No upgrade migration is kept
  for obsolete pre-Sprint-6D development state.
- `scripts/bootstrap-sprint-6d-deployment.ps1` materializes the current
  reference release/instance/configuration/capability/routes, emits first-apply
  evidence, and proves the second run is an exact no-op.
- Every rebuilt image records source commit, tree, dirty state, release
  profile, exact SDK versions, and immutable digest.
- Closeout evidence lives under `artifacts/sprint-6d-closeout/` with the exact
  names in the verification document and SHA-256 sidecars.
- One retained source-exact closeout cycle begins only from a clean committed
  tree after implementation, harness, and readiness corrections are committed.

## Ordered Implementation

1. Commit this specification hardening without production or test changes.
2. Present the complete test-change approval packet and receive explicit user
   approval before editing any test or verification-harness file.
3. Replace the manifest and establish the four package shells plus native/WASM
   boundary gates.
4. Characterize current rendering, then extract runtime/UI, delete
   `ShellContentV1`, move the placement editor, and delete `tessara-web-ui`.
5. Implement the generic document/asset route and reference module.
6. Adopt runtime/UI in Scoped Records and preserve its product behavior.
7. Add Sprint 6D deployment/bootstrap/provenance and only the approved test
   changes.
8. Run the closeout-readiness audit and retained source-exact cycle.

## Non-Goals And 6E Handoff

Sprint 6D does not migrate Dashboard to canonical runtime/UI, remove its
root-web dependency, move its route tree/assets, or prove Dashboard-only
upgrade/rollback. It also excludes generic module product API proxying,
product-flow redesign, shared database adapters, Blueprint automation,
Components extraction, and remote microfrontends.

Sprint 6E receives the current canonical packages, manifest, route/asset
conventions, testkit, and explicit Dashboard findings. It removes the
Dashboard source/runtime debt and proves independent Dashboard release,
upgrade, and rollback.

## Readiness And Blockers

Product decisions are closed. Implementation confirmed that the placement
editor is consumed by both Dashboard and Forms. The approved ownership rule is
therefore applied at the actual seam: generic serializable geometry belongs to
contract, generic DOM mechanics belong to module UI, and product sizing,
validation, and workflow policy remain in each product adapter.

The main delivery risk is breadth: the generic read-only document seam,
immediate shared-UI source move, reference module, and Scoped Records adoption
all land in one sprint. The scope controls above are mandatory; implementation
must not absorb Dashboard adoption, generic product APIs, or compatibility
work to compensate.

The specification commit and test-change approval preconditions are complete.
No product-decision blocker remains. Closeout remains blocked on the retained
source-exact deployment, bootstrap, smoke, UAT, Playwright, outage, and manual
evidence cycle after implementation is committed.

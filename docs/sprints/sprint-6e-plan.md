# Sprint 6E Plan: Dashboard SDK Adoption And Source Independence

Status: draft for review. Implementation is explicitly blocked until this
plan is approved and its open decisions are resolved.

- Branch: `codex/sprint-6e`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6e`
- Base commit: `3f2acf6a7151fa59983bd2fab42123db65b804aa`
- Roadmap authority:
  `Sprint 6E: Dashboard SDK Adoption And Source Independence Slice (Next)`
- Sprint 6D handoff:
  [Sprint 6D Verification](./sprint-6d-verification.md)
- Planned evidence directory: `artifacts/sprint-6e-closeout/`

## Sprint Summary

Sprint 6E completes the first full feature-module extraction. Dashboard keeps
its Sprint 6C process, database, product behavior, configuration, diagnostics,
authorization, and Components compatibility contract, but its release no
longer contains root Core/web code or another module implementation.

Dashboard will compile the Sprint 6D canonical contract, runtime, UI, and
testkit packages into its own native and browser artifacts. Its directory,
create, detail, editor, and viewer routes will remain normal same-origin
Tessara routes rendered and hydrated by the Dashboard release. Core remains
authoritative for authentication, authorization, Organization decisions,
navigation, lifecycle, and fallback documents through generic module
contracts.

This sprint is a boundary and release-independence change, not a product-flow
redesign. No functionality or test/harness changes begin until the user
approves this plan.

## Sprint Specifications

### Source and package ownership

- `tessara-dashboard-module` consumes `tessara-module-contract`,
  `tessara-module-runtime`, `tessara-module-ui`, and
  `tessara-module-testkit` directly.
- Remove the `tessara-dashboard-module -> tessara-web` dependency and every
  transitive path from the Dashboard release to root `tessara-web`,
  `tessara-api`, Core-private DTO/state/bootstrap code, or another module
  implementation.
- Retain `tessara-web-dashboards` as Dashboard-owned UI source unless a rename
  is proven necessary. Make the Dashboard release its sole production
  consumer and remove it from root `tessara-web` features, dependencies,
  routing, hydration, and document bootstrap.
- Remove the Dashboard UI dependency on
  `tessara-web-component-viewer`. Dashboard may consume only the declared
  Components provider contract and policy-neutral shared SDK/UI mechanics;
  Components product rendering code must not enter the Dashboard image.
- Keep Dashboard domain, persistence, migrations, configuration semantics,
  product validation, visibility/redaction rules, and placement policy with
  Dashboard.
- Keep Core session, capability/scope decisions, navigation composition,
  module lifecycle, generic document/asset fallback, and installation receipt
  policy with Core.

### Dashboard-owned documents and assets

- Move `/dashboards`, `/dashboards/new`, `/dashboards/{id}`,
  `/dashboards/{id}/edit`, and `/dashboards/{id}/view` route composition into
  the Dashboard release.
- Render complete documents from verified Shell Context through
  `tessara-module-ui`; do not use root `AppShell`, root route parameters, or
  root Dashboard bootstrap types.
- Build and serve Dashboard-owned CSS, JavaScript/WASM, hydration entrypoint,
  icons, and content-addressed assets from the Dashboard image.
- Keep SSR useful without JavaScript, hydrate without browser-console errors,
  preserve theme/keyboard/responsive behavior, and use immutable cache headers
  for content-hashed assets.
- Preserve the current directory, creation, detail, editor, viewer,
  redaction, and degraded-placement experiences. Text or layout changes are
  allowed only when required to remove a Core-owned implementation detail and
  must be approved as a plan amendment.

### Generic Core integration

- Drive Dashboard browser documents and assets through the Sprint 6D generic
  manifest/service-registration path.
- Delete Dashboard-specific Core document rendering, route bootstrap,
  gateway dispatch, and hydration branches after equivalent generic routing
  is proven.
- Keep Core-owned authenticated fallback documents for disabled, unhealthy,
  upgrading, unavailable, and rollback-transition states.
- Forward only short-lived signed projections and safe request metadata. Do
  not forward browser credentials or reusable Core authority.
- Keep Dashboard product APIs module-owned. Any same-origin product API
  routing added or changed in this sprint must be generic and
  manifest/service-registration driven, not a Dashboard-specific Core path.

### Release, upgrade, and rollback contract

- Add a Sprint 6E deployment profile at `deploy/sprint-6e/compose.yaml` with a
  documented, idempotent bootstrap/materialization command.
- Build Dashboard as an independently replaceable image with exact source
  commit, tree, dirty-state, release identity, and asset digests.
- Capture an installed baseline, upgrade only Dashboard, health-gate the
  switch, then roll back only Dashboard.
- Prove Core, gateway, installation control, Scoped Records, reference SDK,
  and database service image digests and container identities do not change or
  restart during the Dashboard-only upgrade/rollback.
- Preserve Dashboard data, migration ledger, runtime identity,
  configuration, projected security state, and installation receipts across
  the cycle.
- Record baseline, candidate, and rollback Dashboard image/release/asset
  identities in one machine-readable chronology.

### Authorization and provider behavior

- Preserve administrator, scoped manager, reader, and nondisclosure behavior
  for Dashboard routes and APIs.
- Preserve transition-only Components compatibility, configuration
  validation, degraded placement states, provider outage containment, and
  recovery.
- Keep known and random unauthorized resource behavior indistinguishable
  where the existing contract requires nondisclosure.
- A Dashboard outage or failed candidate health check must not interrupt Core,
  Scoped Records, or unrelated routes.

## Acceptance Criteria

1. Native and WASM package/source audits find no Dashboard release path to
   root `tessara-web`, `tessara-api`, Core-private code, or another module
   implementation.
2. The Dashboard image contains its own server binary, complete-document UI,
   hydration artifact, and content-addressed assets.
3. All five Dashboard routes render useful SSR and hydrate through the normal
   same-origin shell with unchanged product behavior.
4. Core authentication, authorization, navigation, lifecycle, and fallback
   ownership remain intact through generic contracts.
5. Existing Dashboard data, migrations, configuration, diagnostics,
   identities, redaction, and Components degraded-state behavior remain
   intact.
6. A tester can upgrade and roll back only Dashboard while unrelated service
   digests and restart counts remain unchanged.
7. The candidate release is observable on a normal Dashboard route through
   approved release identity/asset evidence, without introducing a product UI
   redesign.
8. Every roadmap and exit-condition clause has both a manual walkthrough and
   an automated assertion in retained closeout evidence.

## Manual Test Plan

### Independent release boundary

1. Inspect the Dashboard image/package inventory and confirm it contains the
   Dashboard native binary and Dashboard-owned browser assets.
2. Confirm root Core/web binaries and Components implementation artifacts are
   absent.
3. Open each Dashboard route directly and through shell navigation.
4. Disable JavaScript for directory, editor, and viewer direct loads and
   confirm useful, redaction-safe SSR.
5. Re-enable JavaScript and confirm hydration, keyboard navigation, themes,
   and 1280 px, 768 px, and 390 px layouts.

### Product and authorization preservation

1. As an administrator, create a Dashboard, edit metadata and placements,
   save it, view it, and delete the disposable fixture.
2. As a scoped manager, exercise in-scope read/manage and out-of-scope denial.
3. As a constrained reader, verify redacted placements do not reveal or
   execute hidden Components.
4. Stop or degrade the Components provider and confirm the Dashboard remains
   available with the existing placement fallback; restore it and confirm
   recovery.
5. Inspect Dashboard configuration, diagnostics, lifecycle, and enablement
   through generic Module Management.

### Dashboard-only upgrade and rollback

1. Record the installed baseline release, image, assets, receipts, data, and
   unrelated service digests/restart counts.
2. Deploy the candidate Dashboard release only and wait for readiness before
   switching normal routes.
3. Observe the approved candidate release identity on a normal Dashboard
   route and rerun a representative edit/view flow.
4. Verify Core, gateway, Scoped Records, reference SDK, and unrelated module
   identities and restart counts are unchanged.
5. Roll back only Dashboard, verify the baseline identity is restored, and
   confirm Dashboard data/configuration remain unchanged.
6. Attempt or simulate an unhealthy candidate and confirm Core keeps the
   prior healthy release or serves the Core-owned fallback without affecting
   unrelated routes.

## Automated Test Plan

The approved plan will authorize additive or equal-strength changes in:

- Dashboard module/UI unit and integration tests;
- canonical runtime/UI/testkit conformance coverage for Dashboard;
- native/WASM package and forbidden-source audits;
- Compose/bootstrap/provenance/upgrade/rollback evidence scripts;
- Dashboard, Modules, permissions, no-JavaScript, hydration, asset, outage,
  and rollback Playwright assertions;
- smoke, Sprint UAT, acceptance manifest, and evidence-schema checks.

Existing product, authorization, responsive, accessibility, theme, keyboard,
SSR, no-JavaScript, and hydration expectations may not be deleted, weakened,
skipped, or converted to retries to make the extraction pass.

Planned commands:

- `cargo fmt --all -- --check`
- targeted tests for `tessara-dashboard-module`,
  `tessara-web-dashboards`, canonical SDK packages, and affected Core routing
- native and `wasm32-unknown-unknown` Dashboard release builds
- Dashboard native/WASM dependency and forbidden-source audits
- `cargo test -p tessara-api --locked`
- `cargo test -p tessara-web --locked`
- `npm --prefix .\end2end test`
- `.\scripts\validate-e2e.ps1 ...` against Sprint 6E source/deployment evidence
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`
- `.\scripts\local-launch.ps1` remains a root-profile regression check; the
  retained Sprint 6E cycle uses its dedicated Compose profile

## Closeout-Readiness Plan

- Deployment profile: `deploy/sprint-6e/compose.yaml`, derived from the closed
  Sprint 6D topology without mutating the retained 6D profile.
- Bootstrap: add one Sprint 6E command that materializes the declared
  installation and releases; a second invocation must be an exact no-op.
- Provenance: every built image records exact commit, tree, dirty state, and
  release profile; Dashboard additionally records release and asset digests.
- Migrations: do not change the Dashboard baseline unless product persistence
  requires it. If any schema changes, squash the pre-production baseline and
  apply all affected baselines to disposable empty databases before retained
  evidence.
- Evidence: write canonical retained files under
  `artifacts/sprint-6e-closeout/`, including source provenance, deployment,
  bootstrap first/no-op, migration checkpoint, package boundaries,
  SDK conformance, Dashboard product regression, authorization,
  provider-outage recovery, upgrade/rollback chronology, smoke, UAT,
  Playwright result/summary, and manual UAT.
- Source binding: final deployment, smoke, UAT, Playwright, and manual evidence
  must come from one clean implementation commit. Documentation-only closeout
  may follow without rebuilding images.
- Acceptance mapping: the final verification document maps each roadmap
  clause to one manual and one automated proof.
- Non-admin proof: retain scoped-manager and constrained-reader scenarios.
- Final-cycle budget: resolve Compose, bootstrap, migration, package boundary,
  route inventory, test inventory, and provenance gaps with targeted checks
  before the single fresh source-exact cycle.

## Ordered Implementation Plan

Implementation remains blocked until this plan is approved.

1. Approve the source-ownership decisions and release-observation mechanism
   below.
2. Write the Sprint 6E verification contract and exact test-change inventory
   before changing production code.
3. Add failing native/WASM boundary and source-ownership assertions for the
   current root-web and Components-implementation edges.
4. Make Dashboard UI and bootstrap types module-owned and remove dependencies
   on Core/private and Components implementation code.
5. Adopt canonical runtime/UI/testkit in the Dashboard process.
6. Move complete route composition, SSR, hydration, and assets into the
   Dashboard release.
7. Switch Core to generic manifest/service-registration document, asset, and
   product-route integration; then remove Dashboard-specific root paths.
8. Add the Sprint 6E deployment/bootstrap and Dashboard-only
   upgrade/rollback evidence path.
9. Preserve and extend targeted product, authorization, provider degradation,
   no-JavaScript, hydration, and asset tests alongside each boundary slice.
10. Squash and verify migrations if needed, complete the closeout-readiness
    audit, commit the clean implementation source, and run the retained
    source-exact cycle.

## Dependencies And Blockers

### Required decisions before implementation

1. **Dashboard UI package:** recommended: retain
   `tessara-web-dashboards` as Dashboard-owned source for this sprint, make
   Dashboard its sole production consumer, and defer naming cleanup unless an
   audit proves the name itself creates ambiguity.
2. **Components rendering seam:** recommended: replace the direct
   `tessara-web-component-viewer` dependency with a Dashboard-owned adapter
   over the declared Components provider contract plus canonical UI
   primitives. Do not extract Components product implementation into a new
   shared library.
3. **Release observation:** recommended: bind a module release identity and
   content-addressed asset digest into the normal Dashboard document as
   machine-readable metadata and expose the same identity in diagnostics and
   receipts. The manual walkthrough observes the metadata/asset change; no
   user-facing redesign is added.
4. **Rollback mechanism:** recommended: use immutable Dashboard image/release
   identities and an installation-control receipt change, health-gated before
   route switch. Rollback restores the prior receipt without rebuilding or
   restarting unrelated services.

### Known dependencies

- Sprint 6D canonical contract/runtime/UI/testkit and generic document/asset
  proxy are the implementation baseline.
- Sprint 6C Dashboard database, process, manifest, product APIs, and degraded
  Components behavior remain authoritative behavior.
- The Components provider remains transition-only; Sprint 6E changes only the
  source/build seam required for Dashboard independence.

### Explicit non-goals

- no Dashboard product-flow or visual redesign;
- no Components, Datasets, Responses, Workflows, or Forms extraction;
- no Blueprint/composition automation work from Sprint 6F;
- no external module repository or package publishing;
- no backwards-compatibility facade for obsolete pre-production Dashboard
  release shapes;
- no implementation before plan approval.

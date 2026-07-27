# Sprint 6C Verification

Status: closeout-ready.

This record extends the original Dashboard extraction closeout with the
reusable, definition-independent module pathway requested before Sprint 6C
handoff. The original evidence under `artifacts/sprint-6c-closeout/` remains
historical. Authoritative pathway evidence is retained under
`artifacts/sprint-6c-pathway-closeout/`.

## Source And Deployment

- Reusable-path implementation commit:
  `b1b497689cec0fc0220b6ba26b53deed000a2978`.
- Closeout-discovered module-tab navigation fix and exact deployed commit:
  `f59468fc627d62fd2f8e5d629ba6b7714cc1bd4c`; source tree:
  `963315ddc752f63cdf81be9d7f295be95e9b4cd1`.
- Deployment profile: `deploy/sprint-6c/compose.yaml`.
- Core, Dashboard, Scoped Records, and installation-control release images
  carry the same clean commit/tree labels.
- A destructive fresh-data cycle rebuilt every release image, applied one
  baseline per database, bootstrapped revision 1, and proved a second
  bootstrap invocation was a no-op.
- `deployment-final-fresh.json` is the deployment used for UAT, smoke, and
  Playwright acceptance. After degraded-state testing intentionally recreated
  Core, `deployment-handoff-fresh.json` captured the restored, normal-provider,
  exact-source stack left running for review.

The retained machine records, rather than this document, are authoritative for
container IDs, image IDs, installation identity, timestamps, database
snapshots, and digests.

## Reusable Module Pathway

- Core discovers independently deployed configuration-control endpoints from
  the `TESSARA_MODULE_CONTROL_ENDPOINTS` registry. Adding a module does not
  require a new module-ID branch in Core routing, validation, configuration
  application, security synchronization, findings, or diagnostics.
- The shared configuration form renders supported string, enum, integer,
  number, and boolean fields from each manifest's configuration schema.
  Unsupported schemas fail closed during manifest validation.
- Shared Module Management owns configuration, diagnostics, findings,
  dependencies, lifecycle, enablement, navigation, and route-state behavior.
  Module-specific content is limited to declared metadata, configuration
  schema/values, dependencies, resources, and diagnostics returned by the
  module contract.
- Dashboard and Scoped Records both use this path. Browser conformance iterates
  every independently deployed inventory entry instead of naming one reference
  implementation.
- The migration recipe, ownership boundary, required manifest declarations,
  control endpoint contract, deployment wiring, and conformance checklist are
  documented in `docs/architecture/independent-module-pathway.md`.
- The consistency inventory and custom-function review are retained in
  `docs/audits/module-management-consistency-2026-07-27/README.md`.

## Automated Verification

- `scripts/validate.ps1`: passed in full on pristine disposable Core,
  enrollment, fresh-baseline, and populated-upgrade databases. This included
  evidence self-tests, formatting, native/SSR/hydrate checks, 43 module
  contract tests, 79 web tests, all API feature and integration suites, and
  the release nondisclosure timing proof.
- The first full validation attempt correctly rejected reused one-time
  enrollment state in the disposable test database. After recreating only the
  four named test databases, the complete matrix passed without skips.
- `npm --prefix .\end2end test`: passed all 61 tests in the required
  state-safe one-worker acceptance mode.
- `scripts/validate-e2e.ps1`: passed the manifest-bound 61-test inventory with
  one worker, zero retries, skips, filtered tests, flaky results, or failures.
  The retained JSON report digest is
  `27fd7d9b83cd94732f9f11c6b9bb72179ce874cd1fe42dfae5934cd7df5dd616`.
- `scripts/uat-sprint.ps1`: passed and retained as
  `uat-final-fresh.json` (SHA-256
  `d38874aa06010263af45b54ac52a53a1cf5a7ff14bed4b298f82411669a04814`).
- `scripts/smoke.ps1`: passed and retained as
  `smoke-final-fresh.json` (SHA-256
  `cc3b34eded9784cb9a4f4eea2ea81135be32c67bd53dd47e5b2b8d31e23b4368`).
- `scripts/verify-sprint-6c-isolation.ps1`: passed. Dashboard and Scoped
  Records runtime identities can reach only their owned databases, while
  inverse Core/module access fails closed.
- `scripts/test-sprint-6c-degraded-states.ps1`: passed `available`,
  provider-unavailable, incompatible, inactive, superseded,
  resource-tombstoned, owner-tombstoned, owner-data-destroyed, missing, and
  not-evaluated states. Every state returned nine placements with saved titles
  retained. Proof is `degraded-states-final.json` (SHA-256
  `dc8be774b8540106a7c6254dae3af8b4e9090593eb83fa8cd7469ae69694870f`).

## Closeout Discovery

The first direct four-worker browser run exposed expected cross-file
interference because Module Management intentionally disabled Dashboards while
the Dashboard suite was using it. The canonical acceptance mode already
serializes installation-wide state.

The serialized rerun then found a real shared UI defect: selecting Overview
from a `#configuration` URL changed the active tab only briefly, allowing the
old hash to restore Configuration. Module detail tabs now use real hash
destinations while preserving their ARIA tab semantics. The focused
independent-module control scenario and both complete 61-test runs passed after
the repair.

## Acceptance Mapping

1. A future migration has one documented path for ownership, manifest,
   control endpoint, deployment, bootstrap, and conformance work.
2. Dashboard and Scoped Records expose the same shared configuration,
   diagnostics, lifecycle, enablement, navigation, findings, and route-state
   behavior.
3. Their differences are declarative metadata/configuration and
   module-owned product behavior, not Core module-ID conditionals.
4. Configuration schema support and unsupported-schema rejection are enforced
   by contract tests.
5. Common UI behavior is exercised by iterating the independent-module
   inventory.
6. Exact-source UAT, smoke, browser, database-isolation, degraded-state, and
   full repository gates are retained and green.

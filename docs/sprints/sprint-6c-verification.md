# Sprint 6C Verification

Status: complete. Final corrective closeout completed on 2026-07-29.

This record supersedes the earlier Sprint 6C closeout summaries for final
handoff. Historical evidence remains under `artifacts/sprint-6c-closeout/` and
`artifacts/sprint-6c-pathway-closeout/`. The authoritative final evidence is
under `artifacts/sprint-6c-final-closeout-2026-07-29-r2/`.

## Source And Deployment

- Final implementation commit:
  `a5d694f7ef7c68e52a9ac93135846d29d5a061d7`; source tree:
  `fc5494044be6c8dffa6c38381b5610f49f6619c4`.
- The application, UAT corrections, reusable module pathway, walkthrough, and
  test inventory were committed in `84f0d83a8294685d85120fe943d7ab846495c74b`.
- The final commit corrects explicit PostgreSQL binding validation in the
  retained Playwright harness. Because acceptance evidence is source-bound,
  every release image was rebuilt and all deployment, UAT, smoke, browser, and
  degraded-state evidence was recaptured after that commit.
- Deployment profile: `deploy/sprint-6c/compose.yaml` plus
  `deploy/sprint-6c/compose.override.yaml`, project `tessara`.
- Core, Dashboard, Scoped Records, and installation-control images carry the
  same clean commit/tree labels. `deployment-fresh.json` proves the exact
  source and database-derived `fresh` state.
- Bootstrap found the existing revision-1 deployment receipt and made no
  change, proving the repeat invocation is idempotent.
- The degraded-state matrix intentionally recreated Core. The restored
  available-provider stack was recaptured as
  `deployment-handoff-fresh.json` and left running for review.

Authoritative digests:

- Deployment:
  `4b7b305a9d8a3eae40fb0aa9cf972ffae756cf56bde5705957e9d56929e538e7`.
- Restored handoff deployment:
  `47569c2344e4aa95078c72e9c5061fc33aaabc5b8090ef0913c12a56d225dd64`.
- UAT:
  `2bc71a9100b25097ea79f9001fc744ff0ad288824ef42acf5be3b7fdcb08651d`.
- Smoke:
  `406a9c3e4029970e221084e4880638ab07838a8e55804c7e7ff1672367a62b21`.
- Retained Playwright JSON:
  `c81b4048feacda6f69fbd09221f2822fe820a8ea553b35f84d9b6af45b45bdf6`.
- Degraded states:
  `927f700b2ec1fbe9bc763788b57725336605e94ba6756ade2bd7062cdd2f936a`.

## Reusable Module Pathway

- Core discovers independently deployed configuration-control endpoints from
  `TESSARA_MODULE_CONTROL_ENDPOINTS`. Adding a module requires no module-ID
  branch in Core routing, validation, configuration application, security
  synchronization, findings, or diagnostics.
- The shared configuration form renders supported schema fields and rejects
  unsupported schemas during manifest validation.
- Shared Module Management owns configuration, diagnostics, findings,
  dependencies, lifecycle, enablement, navigation, and route-state behavior.
  Module-specific differences are declared metadata, schema/values,
  dependencies, resources, diagnostics, and product behavior.
- Dashboard and Scoped Records both use the pathway. Common browser
  conformance iterates independently deployed inventory entries.
- The migration recipe is
  `docs/architecture/independent-module-pathway.md`; the consistency and
  custom-function inventory is
  `docs/audits/module-management-consistency-2026-07-27/README.md`.

## Automated Verification

- `cargo fmt --all -- --check`: passed.
- `cargo test -p tessara-api --locked`: passed all unit, integration, native
  route, module, permission, fresh-baseline, and workflow suites using three
  isolated disposable databases. Two demo tests are deliberately delegated to
  the split-stack smoke/UAT coverage; no test failed.
- `cargo test -p tessara-web --locked`: passed 79/79.
- The previously completed full `scripts/validate.ps1` and Clippy verification
  remain applicable: the only later source change is the TypeScript
  acceptance binding fix described above.
- `npm --prefix .\end2end test`: passed 61/61 against the exact deployed
  source.
- `scripts/validate-e2e.ps1`: passed the manifest-bound 61-test inventory with
  one worker, zero retries, skips, filtered tests, flaky results, or failures.
- `scripts/uat-sprint.ps1`: passed the module, organization, forms, datasets,
  components, dashboards, and seed flows.
- `scripts/smoke.ps1`: passed against the retained fresh deployment and
  preserved Dashboard `41933f5c-f02b-47c6-b44f-6edffa32c283`.
- `scripts/verify-sprint-6c-isolation.ps1`: passed negative cross-database
  credential checks.
- `scripts/test-sprint-6c-degraded-states.ps1`: passed available,
  provider-unavailable, incompatible, inactive, superseded, tombstoned,
  owner-tombstoned, owner-data-destroyed, missing, and not-evaluated states.
  All ten states retained nine placements and saved titles, and the original
  enabled/available state was restored.
- Visual UAT inspection covered Module Management parity, configuration and
  findings layout, enablement semantics, product-navigation removal while
  disabled, Dashboard outage containment, editor warning treatment, the
  centered side-sheet icon, responsive states, and normal restored operation.
  The walkthrough is `docs/sprints/sprint-6c-uat-walkthrough.md`.

## Closeout Discoveries

The first final evidence capture used the generic deployment helper without
overriding its historical `api` service name. Sprint 6C names that service
`core`; rerunning with the exact Core, gateway, and PostgreSQL container IDs
passed. No application or evidence contract changed.

The first retained acceptance attempt exposed a real harness defect:
explicit database binding validation referenced derived variables before
initialization. The fix validates the configured database and user values
before deriving the command. A focused reproduction passed, the fix was
committed, the stack was rebuilt from that exact commit, and the complete
source-exact acceptance cycle was repeated successfully.

The first unconfigured API test invocation passed 151 tests and correctly
refused to skip two required database integration tests. The final invocation
supplied isolated, token-bounded disposable databases plus the explicit
destructive-test acknowledgement; the complete API suite passed. The named
test container and its data were removed afterward without touching the
Sprint stack.

## Acceptance Mapping

1. Separate Dashboard service/database/manifest/configuration/health/API/SSR:
   Compose and deployment evidence, UAT, smoke, API/web tests, and Module
   Management walkthrough passed.
2. Runtime database isolation: `verify-sprint-6c-isolation.ps1` passed all
   negative cross-database checks.
3. Typed ComponentVersion references without Core relational dependency:
   module contract, Dashboard repository, baseline, and API tests passed.
4. Versioned transition-only adapter and Sprint 8A migration marker: contract
   tests, manifest inspection, and Module Management configuration note passed.
5. Downstream grant binding: module contract and Core adapter actor,
   installation, audience, action, scope, revision, expiry, and replay tests
   passed.
6. Nondisclosure and complete resolution states: permissions tests and the
   ten-state degraded matrix passed.
7. Sprint 5A product continuity: Dashboard directory, create, detail, editor,
   viewer, SSR, responsive, and canonical Playwright coverage passed.
8. Core-owned Module Management without product-data access: shared
   configuration/diagnostics UAT and isolation tests passed.
9. Contained Dashboard/Components outages: visual outage inspection,
   degraded matrix, retry behavior, and unrelated-route checks passed.
10. Deterministic fresh bootstrap/baselines: database-derived fresh evidence,
    baseline tests, revision-1 receipt, and repeat bootstrap no-op passed.
11. Exact-clean-source verification: commit/tree-labelled deployment, UAT,
    smoke, 61/61 direct and retained Playwright, Rust suites, isolation, and
    degraded-state evidence passed.
12. Reusable Module Management template: Dashboard and Scoped Records use one
    schema-driven control path, shared conformance tests pass, and the
    registration-based third-module recipe contains no definition-specific
    Core branch.

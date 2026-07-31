# Sprint 6D Verification

Status: test-change packet approved by the user on 2026-07-30; pre-commit
implementation verification passed, but retained source-exact closeout
evidence has not yet been captured.

This document maps the
[Sprint 6D roadmap](../roadmap.md#sprint-6d-canonical-module-sdk-and-runtime-extraction-slice-next)
to required manual and automated proof. It is governed by the
[Sprint 6D plan](./sprint-6d-plan.md) and
[Module SDK Implementation Contract](../architecture/module-sdk-implementation-contract.md).
The tester-ready manual scenarios are indexed in
[Sprint 6D UAT Scripts](./sprint-6d-uat/README.md).

The user approved this consolidated test-change packet before test
implementation. Any later change that weakens, deletes, skips, retries, or
materially replaces an existing product expectation requires renewed
approval.

## Approved Test-Change Packet

The approval covers equal-or-stronger additions and fast-forward fixture
updates in these exact areas:

- contract/runtime/UI/testkit/reference and Scoped Records unit/integration
  tests under their owning crates;
- current manifest/deployment fixtures and digests under
  `crates/tessara-module-contract/tests/fixtures/`,
  `crates/tessara-dashboard-module/`, and
  `deploy/sprint-6b1/fixtures/`;
- package/source graph and exact-version inventory checks in
  `scripts/verify-module-sdk-boundaries.ps1`,
  `scripts/verify-module-sdk-compatibility.ps1`, and `scripts/validate.ps1`;
- Sprint 6D Compose/bootstrap/provenance/evidence harnesses under
  `deploy/sprint-6d/` and `scripts/bootstrap-sprint-6d-deployment.ps1`;
- additive Sprint 6D assertions in retained smoke, UAT, and Playwright
  harnesses when exercised against the reference module;
- Markdown link/evidence validators needed to enforce this document.

The approved purpose is to prove exact-version rejection, native/WASM
independence, signed-context/grant failure modes, configuration persistence,
probes/diagnostics, complete SSR/no-JavaScript/hydration behavior, immutable
assets, authorization nondisclosure, outage containment, graceful shutdown,
source/image provenance, and bootstrap idempotence. Existing Dashboard,
Scoped Records, Core, responsive, theme, keyboard, authorization, smoke, UAT,
and Playwright expectations may not be loosened.

## Closeout Preconditions

- HEAD is a committed clean `codex/sprint-6d` source tree.
- `docs/roadmap.md` still marks Sprint 6D as the sole `(Next)` sprint.
- Every production manifest uses the sole current schema and exact supported
  platform tuple.
- No unresolved decision blocker remains in the Sprint plan or implementation
  contract.
- Compose, bootstrap, provenance, migrations/baselines, current harness
  inventories, and evidence publishers pass their readiness checks before the
  retained run.
- No closeout artifact is silently overwritten; a rerun uses a new disposable
  environment and intentionally replaces evidence only through the documented
  publisher behavior.

## Roadmap Clause Mapping

| Roadmap clause | Specification and implementation owner | Manual proof | Automated proof | Retained evidence |
| --- | --- | --- | --- | --- |
| Inventory shared/duplicated behavior and assign one source owner | Ownership inventory and implementation contract | Review every row and the visible Dashboard 6E finding | Source/duplicate scanner and graph audit | `sdk-ownership.json` |
| Establish contract/runtime/UI/testkit boundaries | Implementation contract package graph | Inspect native metadata and package source | Exact allowed-edge audit | `package-boundaries-native.json` |
| Extract verification, destinations, references, errors, control, health, diagnostics, tracing, and shutdown without policy/product semantics | Contract/runtime providers | Exercise conformance output and sanitized diagnostics | Targeted contract/runtime/testkit suites and source audit | `reference-conformance.json` |
| Extract complete-document shell, primitives, tokens, accessibility, assets, and hydration | Module UI and reference module | Inspect SSR/no-JavaScript, themes, keyboard use, assets, and responsive layouts | UI/reference native/WASM tests and Playwright | `e2e-fresh.json` |
| Define semantic versioning, manifest declaration, support window, deprecation, and release inventory | Fast-forward version/security policy | Inspect current release inventory and one rejected obsolete tuple | Exact-version and advisory-policy tests | `compatibility-inventory.json` |
| Prove canonical packages cannot reach Core/root/product implementations | Boundary enforcement | Review native/WASM graph paths | Native/WASM metadata and source scans | `package-boundaries-native.json`, `package-boundaries-wasm.json` |
| Add a non-product reference module and shared testkit | Reference module contract | Run the fixture and shared conformance command | Reference/testkit suites | `reference-conformance.json` |
| Document authoring and upgrade workflow | Independent Module Pathway | Follow the authoring checklist from a clean checkout | Documentation link/check command and conformance entrypoint validation | `manual-uat.md` |
| Preserve Core, Dashboard, and Scoped Records behavior; defer Dashboard adoption | Sprint plan non-goals and 6E handoff | Exercise retained product routes | Core/web/Dashboard/Scoped Records regression suites | `scoped-records-regression.json`, `smoke-fresh.json`, `uat-fresh.json` |

## Exit-Condition Mapping

Every exit-condition clause has at least one manual and one automated proof.

| Exit-condition clause | Manual proof | Automated proof | Evidence |
| --- | --- | --- | --- |
| Build/run the reference without root `tessara-web` or `tessara-api` | Inspect native/WASM metadata and run the image directly | Transitive graph/source audits and target builds | `package-boundaries-native.json`, `package-boundaries-wasm.json` |
| Navigate to a coherent same-origin complete SSR route | Sign in and navigate from normal shell navigation; inspect raw HTML and assets | Smoke/UAT/Playwright SSR, landmarks, theme, responsive, and asset assertions | `smoke-fresh.json`, `uat-fresh.json`, `e2e-fresh.json` |
| Verify authenticated and unavailable states | Use allowed/constrained actors, disable the module, stop it, and restart it | Authorization/nondisclosure and outage/Core-fallback automation | `reference-conformance.json`, `outage-recovery.json` |
| Run the shared conformance suite | Run the documented command and inspect all checks | Contract/runtime/UI/testkit/reference suites | `reference-conformance.json` |
| Show one canonical implementation per extracted behavior | Review ownership/source output and explain the Dashboard 6E finding | Duplicate-source and forbidden-symbol scans | `sdk-ownership.json` |
| Show module-owned compiled runtime/UI/assets in the image | Inspect image contents, asset URLs, headers, and provenance | Image/package/asset digest assertions | `source-provenance.json`, `deployment-fresh.json` |

## Required Manual Matrix

The retained `manual-uat.md` records actor, route, expected result, actual
result, timestamp, source commit/tree, running image digests, and evidence
references for:

1. fresh deployment and first bootstrap;
2. second bootstrap as an exact no-op;
3. administrator reference navigation and complete SSR;
4. no-JavaScript usefulness;
5. keyboard navigation and focus behavior;
6. light, dark, and system themes;
7. 1280 px, 768 px, and 390 px presentation;
8. module-owned content-hashed assets and cache headers;
9. configuration validation, normalization, persistence, and read-back;
10. liveness, readiness, and sanitized diagnostics;
11. constrained actor root access and authorized Organization probe;
12. identical known/random unauthorized Organization results;
13. disable/re-enable behavior;
14. outage/Core fallback and unrelated-route continuity;
15. restart with retained reference state;
16. bounded graceful shutdown;
17. Scoped Records product/authorization regression;
18. Dashboard transition visibility and unchanged product flow;
19. current exact SDK inventory and rejected obsolete version;
20. native/WASM source/package independence review.

## Planned Automated Commands

Exact commands may be refined in the user-approved test-change packet, but the
packet must cover:

- `cargo fmt --all -- --check`;
- targeted contract/runtime/UI/testkit/reference/Scoped Records tests;
- `cargo test -p tessara-api`;
- `cargo test -p tessara-web`;
- retained Dashboard targeted tests;
- reference native build with `ssr`;
- contract, module UI, and reference checks for
  `wasm32-unknown-unknown` with the applicable features;
- native and WASM package/source boundary audits;
- Compose configuration validation;
- disposable fresh baselines and reference state initialization;
- Sprint 6D bootstrap twice;
- smoke and Sprint UAT;
- retained Playwright through the repository validation wrapper;
- evidence-schema, digest, provenance, and cross-file consistency checks.

These commands and paths are authorized by the approved packet above.

## Evidence Inventory

Retained files live under `artifacts/sprint-6d-closeout/`:

| Filename | Required contents |
| --- | --- |
| `source-provenance.json` | clean commit/tree/dirty state, release profile, image digests/labels, exact SDK versions |
| `deployment-fresh.json` | Compose topology, service/image/health inventory, source binding |
| `bootstrap-first.json` | desired/plan/apply/receipt identities and first materialization |
| `bootstrap-second-noop.json` | exact desired/current comparison and zero material changes |
| `migration-checkpoint.json` | direct baseline versions/checksums, empty-state proof, exact reference state schema |
| `sdk-ownership.json` | canonical owner/source/consumer/disposition rows and Dashboard 6E finding |
| `package-boundaries-native.json` | native direct/transitive graph, source findings, pass/fail |
| `package-boundaries-wasm.json` | WASM direct/transitive graph, feature/target findings, pass/fail |
| `compatibility-inventory.json` | current exact tuple, linked versions by release, unsupported/advisory findings |
| `reference-conformance.json` | manifest, context/grant, configuration, probes, diagnostics, SSR/assets, shutdown |
| `scoped-records-regression.json` | runtime/UI adoption plus retained product/authorization results |
| `outage-recovery.json` | stop/fallback/containment/restart chronology and results |
| `shutdown.json` | signal, drain/flush, exit timing, post-shutdown state |
| `smoke-fresh.json` | source-exact smoke results |
| `uat-fresh.json` | source-exact Sprint UAT results |
| `e2e-fresh.json` | complete retained Playwright result |
| `e2e-fresh.summary.json` | suite/test counts, failures, skips, source/deployment binding |
| `manual-uat.md` | signed-off manual matrix |

Each retained JSON/Markdown file has a sibling `.sha256` containing the
lower-case SHA-256 digest followed by one newline.

The canonical publishers are:

- package boundary, compatibility, and conformance scripts named in the
  approved test-change packet;
- `capture-sprint-6d-closeout-evidence.ps1 -Mode Static` for source/image,
  migration, and ownership proof;
- `capture-sprint-6d-closeout-evidence.ps1 -Mode RuntimeResilience` for one
  graceful stop, Core fallback/containment, restart, and state-retention
  chronology;
- `capture-sprint-6d-closeout-evidence.ps1 -Mode
  ScopedRecordsRegression` after smoke, UAT, and Playwright are retained;
- `capture-sprint-6d-closeout-evidence.ps1 -Mode Digests` as the final
  complete-inventory and sidecar gate.

## Migration Checkpoint

Sprint 6D uses fast-forward development baselines:

- affected database baselines are advanced or squashed directly;
- every affected current baseline is applied to a disposable empty database;
- no migration is retained solely to upgrade obsolete pre-Sprint-6D
  development state;
- the reference JSON-state command creates or validates only the current
  schema;
- an incompatible reference fixture volume is recreated for the retained
  fresh run.

The checkpoint must explicitly say which database baselines changed. If none
changed, it records deliberate no-database-schema-change proof plus every
fresh-baseline command still rerun.

## Closeout Decision

Sprint 6D may move from implementation-complete to closeout only when:

- every roadmap and exit-condition row has both proofs;
- every automated change matches the user-approved test packet or a later
  explicit approval;
- no canonical boundary finding remains;
- the sole expected Dashboard transition finding is still assigned to Sprint
  6E and has not expanded;
- all retained evidence is source-exact, internally consistent, and
  digest-bound;
- the repository is clean at the evidenced commit.

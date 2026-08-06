# Sprint 8A Validation Record

Status: Validation was explicitly exited during incomplete Candidate Rehearsal
attempts 27 and 28 so implementation readiness could be re-established.
Neither attempt may be completed or used as a prerequisite. The consolidated
candidate-affecting correction below invalidates every earlier readiness and
rehearsal result. This record is an implementation handoff: no current
readiness or rehearsal pass exists, no candidate has been frozen, and
preflight, SIT, formal UAT, and closeout remain Not Run.

- Sprint: Sprint 8A — Component Module Separation Slice
- Branch: `codex/sprint-8a`
- Planned evidence root: `artifacts/sprint-8a-closeout/`
- Execution contract: [Sprint 8A plan](./sprint-8a-plan.md)

## Implementation Readiness Snapshot

### Post-attempt-28 implementation correction

On 2026-08-06 the user explicitly stopped testing after prolonged failure to
reach preflight and returned Sprint 8A to implementation. The source identity
at exit was commit `7b326838269c3b4218dd579f649a8011feba4d97`, tree
`fa78175797fddba7c7a0b14bb141a33cf4d40fd0`, clean. The append-only local
invalidation receipt is
`artifacts/sprint-8a-closeout/attempts/implementation-return-2026-08-06-invalidation.json`
with SHA-256
`39da96d3672ebd078c551e3179a1b3cf97a4a13671a61627425feed9e894e150`.
It supersedes readiness 31, marks readiness 32 invalid, and records
user-directed abandonment of rehearsal 27 and 28. Candidate freeze, preflight,
SIT, and UAT remain prohibited.

Attempt 28 ended before a protocol-complete harvest because testing was
explicitly exited. Its partial raw evidence remains non-authoritative in
`artifacts/sprint-8a-closeout/rehearsal/materialization-attempt-28.log` and
`artifacts/sprint-8a-closeout/rehearsal/full-validation-attempt-28.log`. Three
observed harness/evidence failures were retained rather than silently retried:

- Validation Readiness parsed database URLs without authenticated reachability
  and produced an environment receipt that Candidate Rehearsal could not
  compare structurally.
- Automated UAT accepted a stale failure-containment receipt by existence
  instead of binding it to the current attempt, source, environment, time, and
  content hash.
- Attempt startup tried to add undeclared `assertions_started_at` data to its
  receipt schema.

The Dashboard duplication is retained separately as a product/architecture
defect, not reclassified as a count-only test failure. Its raw classification
is preserved in
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-4-defect-batch.json`
(`overall_classification: product`), with its source-exact invalidation in
`candidate-rehearsal-4-invalidation.json`. That evidence records why the
earlier seven-to-six correction was incomplete: Dashboard simultaneously
appeared as a frozen Core transition and an enrolled Module Release/Instance.
The corrected contract is the exact five Core transition identities, with
Dashboard supplied only by its enrolled manifest. Narrow correction evidence
never discharged the requirement for a new complete Readiness and Candidate
Rehearsal cycle.

The subsequent sprint-plan/architecture audit confirmed one consolidated
implementation-readiness correction batch spanning product, fixtures, tests,
deployment inputs, and validation runners:

- Core's transition catalog is exactly Forms, Workflows, Responses, Datasets,
  and Migration. Dashboard is removed from every canonical transition input
  and can appear only once through its real Release/Instance and manifest.
  Inventory now rejects transition/release identity overlap rather than
  suppressing the duplicate after composition.
- Generic signed-bootstrap receipt bindings carry exact Component owner
  read-back identities into Dashboard seed input before input hashing and
  idempotency. No copied Dashboard-to-Component seed identity or
  Dashboard-specific orchestration branch remains.
- Every module-to-module hop exchanges the verified inbound grant through Core
  for the exact downstream audience. Enrolled manifests declare both the
  consumer request and provider service action; Core accepts only the exact
  lockfile-resolved provider/action/capability tuple and projected live service
  identities. Component and Dataset provider routes contain no sibling caller
  names, caller-specific keys, or first-use identity enrollment.
- Component provider authorization preserves known-versus-random resource
  nondisclosure, while Dashboard presents Components as an installed module
  dependency rather than a Core transition.
- Component-specific lifecycle handling is removed from Core's web host. The
  host renders generic module lifecycle contracts, and extracted Component
  integration coverage exercises the durable module database and API boundary.
- Current navigation and acceptance tests assert exact identities: the five
  Core transitions plus real Scoped Records, Components, and Dashboard
  contributions, ordered 7, 8, and 9. The resolved lockfile policy is
  transactionally projected and is the only mutable navigation source;
  historical seven-entry fixtures are explicitly historical and never reused
  as current expectations.
- Core's startup seed contains no Component or Dashboard capability, role
  grant, service-action, product-route, product-data, or product-asset
  declaration. Those capabilities/actions enter through release enrollment,
  and both products' CSS and JavaScript remain in their owner release images.
  Component direct-load SSR and hydrated visual identity tests now assert the
  owner document rather than obsolete Core loading placeholders.
- Validation Readiness authenticates six pairwise-distinct disposable
  databases and emits one secret-free source/environment fingerprint.
  `localhost`, `127.0.0.1`, and `::1` are one canonical loopback server
  identity for uniqueness, so aliases cannot make one database satisfy two
  required bindings.
  Candidate Rehearsal is now a repository-owned dependency graph with safe
  fail-late siblings, exact blocked reasons, append-only lane evidence, one
  consolidated batch per harvest, and correction authorization only after the
  harvest guard passes.
- Materialization records resolved targets, empty baseline, first apply,
  unchanged semantic no-op, owner receipt bindings, final health, failure logs,
  exact teardown, and source/environment identity. Deterministic failure
  containment proves a clean from-empty successor.
- Canonical bootstrap now has exactly seven Component shells, eight versions,
  and seven Dashboard placements. The Stat Card shell carries a
  superseded/inactive predecessor with a published/active declared successor;
  three independent Dashboard placements bind that predecessor for
  Defer/Upgrade, Replace, and Remove, while the blocked-scope placement remains
  distinct.
- The Dashboard dependency diagnostic exercises those exact live identities.
  It requires exactly three initial lifecycle findings, authorized successor
  disclosure, blocked-scope nondisclosure, Defer followed by Upgrade, Replace,
  Remove, the exact five remaining authorized placements during a Component
  provider outage, and healthy zero-finding convergence after recovery. Its
  structured receipt is a mandatory UAT-8A-05 diagnostic dependency; broad
  lane labels cannot satisfy the scenario. Its check-by-check JSON and SHA-256
  sidecar are retained even when the inner diagnostic fails, and the outer
  fail-late harvest embeds both the partial result and raw artifact identity.
  The independent broad sibling uses the canonical Sprint 8A ownership smoke,
  not the removed Core-era demo Component/Dashboard seed.
- Component calls the versioned Dataset compatibility operation and retains a
  sanitized observation of the selected binding, Core-installation provider,
  contract/version, compatibility, health, observation time, and stable
  result/failure codes. The Component Manifest declares configuration defaults
  `Components` and `5` and uses the real
  `/usr/local/bin/component-module` runtime/migration executable path.
- Component upgrade rehearsal now builds a distinct compatible `0.9.0`
  release from source. Exact one-owner Blueprint deltas are applied by the
  Supervisor through its Compose adapter for baseline establishment,
  `0.9.0` to `1.0.0` upgrade, rollback to `0.9.0`, and intended-`1.0.0`
  restoration; preservation snapshots precede each transition. Supervisor
  finalization commits receipt, operation, emergency override, and override
  reconciliation atomically; Core commits inventory and receipt projection in
  one transaction. Failed projection/finalization retires the unprojected
  receipt, terminalizes the operation, and permits an exact-sequence retry.
  An unchanged desired revision resolves to only `VerifyReadBack`, approves no
  effects, carries prior state, and marks carried bootstrap receipts unchanged.
- Playwright failure evidence and all eight automated UAT diagnostics are
  attempt-, source-, environment-, chronology-, and hash-bound. A failed
  browser run retains reports, traces, logs, test results, and error context.
  Component browser fixtures consume the canonical nested Core-owned Dataset
  major-line reference and derive its positive major from `<uuid>@<major>`;
  removed flat Dataset fields cannot satisfy the test contract.
- The Dataset compatibility contract, Core reference resolver, provider, and
  storage now share one exact major-line identity boundary: canonical
  `<uuid>@<positive-i32-major>`. Core rejects the obsolete colon spelling,
  alternate integer spellings, zero/negative versions, extra separators, and
  values beyond the signed storage range; provider paths contain no unchecked
  unsigned-to-signed major casts.

Focused implementation checks for this batch passed at handoff:

- `cargo fmt --all -- --check`;
- zero-warning `cargo check` and Clippy across all targets/features for the
  Dataset contract, Core API, Supervisor, composition engine, Component
  module, Dashboard module, and web host;
- Supervisor transaction/no-op/rollback tests, composition tests, all-feature
  Component library tests, Dataset-contract tests, and exact Core catalog,
  navigation, manifest-digest, restricted-reference, and Dataset-major
  boundary tests;
- parsing for all 25 changed PowerShell scripts and all seven changed JSON
  documents, local Markdown-link validation, and Sprint 8A Compose
  configuration rendering;
- the acceptance contract plus Dashboard semantic, product diagnostic,
  readiness graph, candidate graph, harvest guard, and automated-UAT runner
  self-tests; and
- exact acceptance-manifest discovery of 71 Playwright scenarios, including
  the four Sprint 8A Component scenarios. This was inventory discovery only;
  no browser or deployed-system execution occurred.

These focused checks do not constitute Validation Readiness or Candidate
Rehearsal. Database-backed integration, live deployment, full Playwright,
preflight, SIT, and formal UAT remain Not Run. The next phase must run
`validate-sprint-8a-readiness.ps1` and then the complete repository-owned
`run-sprint-8a-candidate-rehearsal.ps1` against the same clean source and
environment identity.

### Earlier post-attempt-27 correction (superseded context)

The 2026-08-06 implementation audit reconstructed the sprint behavior from the
Sprint 8A acceptance criteria and the closed Sprint 4A/4B/7B Component
contracts instead of treating repeated harness progress as product readiness.
It identified one consolidated implementation-readiness batch:

- **Product lifecycle defect:** the extracted Component module permitted
  transitions outside the closed state machine, including draft lifecycle
  actions and invalid tombstone/reactivation paths.
- **Product authoring defect:** extracted validation accepted unknown keys,
  unavailable fields, invalid type/operator combinations, invalid visual
  modes and limits, and overlong version notes.
- **Product execution defect:** the cross-process Dataset execution path had
  lost table search, runtime projection narrowing, configured/runtime sort,
  complete filters, typed comparisons, unique count, median, do-not-summarize,
  display labels, smoothing, and value/dimension missing-policy semantics.
- **Architecture/fixture defect:** Sprint 8A reused a Sprint 7A fixture helper
  that directly mutated Core, Component, and Dashboard product tables after
  owner bootstrap. This crossed database ownership boundaries and could hide
  incomplete signed bootstrap inputs.
- **Acceptance/bootstrap defect:** the signed Blueprint did not own the full
  recognizable 30-row/blocked-scope Dataset seed and exact Dashboard
  placements, while Sprint 8A acceptance identities still named superseded
  Component versions. Core bootstrap also attempted to write a nonexistent
  Dataset-revision authority column.

The consolidated correction restores the lifecycle and authoring contracts,
extends the typed Dataset execution contract and Core adapter, moves all
Sprint 8A product seed into the owning signed bootstraps, makes the legacy
fixture helper account/security-only for Sprint 8A, and binds acceptance to
exact owner-bootstrap identities. The Dashboard bootstrap now establishes its
own authority revision, and the materialization audit asserts exact Component
external keys rather than a copied count. Validation Readiness now parses the
owner-controlled fixture preparer and Sprint 8A acceptance contract explicitly,
closing the enforcement gap that let seed-path syntax sit outside its runner
inventory.

Focused implementation verification for this mutable batch is recorded below.
These checks do not constitute Validation Readiness or Candidate Rehearsal:

- touched Sprint 8A PowerShell files parse: Passed;
- `cargo check --locked` for Dataset contract, Component module, Core API, and
  Dashboard module: Passed with zero warnings;
- focused Clippy for those four packages across all targets with
  `-D warnings`: Passed;
- Component, Dashboard, and Dataset-contract library tests: Passed (18, 13,
  and 3 tests), and the focused Core Dataset-adapter/bootstrap suites passed
  (3 and 1 tests);
- Sprint 7A/Sprint 8A acceptance-contract evaluation and owner-controlled
  fixture-preparer self-test: Passed.

A fresh complete Validation Readiness run and then a fresh complete Candidate
Rehearsal run are required against the same new clean source/environment
identity before preflight may begin.

Observed on the mutable `codex/sprint-8a` implementation tree on 2026-08-05:

- `cargo fmt --all -- --check`: Passed.
- `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings`:
  Passed with zero warnings.
- `cargo test --workspace --lib --bins --locked --offline` with the two
  database-dependent API tests filtered: Passed. The filtered tests require
  `TEST_API_ENROLLMENT_DATABASE_URL` or `TEST_API_DATABASE_URL` and remain part
  of validation readiness/rehearsal rather than being treated as skips.
- Component, Components v3, Dataset v1, Dashboard module/UI/placement renderer,
  composition, Core navigation/baseline, and module lifecycle focused suites:
  Passed.
- `scripts/check-web-crate-boundaries.ps1`: Passed.
- Sprint 8A PowerShell parser checks and
  `scripts/smoke-sprint-8a.ps1 -SelfTest`: Passed.
- `docker compose -f deploy/sprint-8a/compose.yaml --profile reference config --quiet`:
  Passed.
- Sprint 8A catalog sign, signature verification, and Blueprint resolution:
  Passed against the current canonical manifest digests.

The deployed smoke, destructive materialization, database-backed integration
tests, Playwright, failure teardown/rerun, upgrade/rollback, SIT, and all eight
UAT scenarios remain Not Run. Their execution is reserved for the specialized
validation and UAT workflows.

## Scope And Acceptance Inventory

| Roadmap clause | Risk/contract | Automated proof | Deployed smoke proof | Manual UAT proof |
|---|---|---|---|---|
| Independent full-stack Component module | Hidden Core/build/runtime coupling | native/WASM package graph, forbidden-source, image inventory, route-owner and manifest/testkit assertions | Component process/database/routes/assets healthy while Core product paths are absent | UAT-8A-01/06/08 |
| Canonical packages; no copied source or sibling implementation | Source ownership drift | dependency/source/package/image audits for Component and Dashboard | provenance and image-content read-back | UAT-8A-06 |
| Module owns configuration, manifest, capabilities, database/schema migrations, health, documents and assets | Incomplete extraction | manifest, configuration, capability, schema, probe, document and asset conformance | generic Module Management plus every product/operational route | UAT-8A-01/03 |
| APIs/contracts/typed references only | Cross-database or private DTO coupling | source/SQL/credential and exact-contract integration tests | deny cross-database credentials while product flow passes | UAT-8A-04/06 |
| Phase 8 fresh materialization from empty | Partial/stale transition state | reset-target, empty-baseline, owner-order, semantic seed and no-op tests | first source-exact bootstrap plus unchanged second run | UAT-8A-02 |
| Failed fresh materialization destroys and reruns | Partial topology reused | induced failure, exact teardown, volume absence and complete rerun assertions | retained failed receipt followed by new from-empty healthy attempt | UAT-8A-07 |
| Old transition references and payloads unsupported | Hidden legacy compatibility | historical fixture integrity plus old owner/type/version/payload rejection | old inputs rejected; no adapter, ledger, reader or live old reference | UAT-8A-06 |
| Rerun Phase 7 scope/lifecycle/outage/compatibility | Logical behavior changes at physical boundary | contract, lifecycle, nondisclosure, scope, timeout, compatibility and outage suites | real Component/Dataset/Dashboard process-boundary smoke | UAT-8A-04/05 |
| Rerun source ownership, package graph, independent upgrade/rollback | Coupled or synthetic release | boundary audits; source-built release/binary identity; exact one-owner plan assertions | Supervisor/Compose `0.9.0` to `1.0.0` upgrade, rollback and restoration; unrelated digests/restarts unchanged | UAT-8A-06/08 |
| Move all Component product and operational ownership | Split ownership or missing surface | module product/API/UI/operations integration suites | same-origin module-owned routes and diagnostics | UAT-8A-01/03 |
| Real Release/Instance and fresh database | Wrong identity/provenance/storage | release/instance/catalog/schema/database-isolation tests | composition read-back and exact provenance | UAT-8A-02/06 |
| Dashboard has one canonical inventory/navigation owner | Duplicate transition and deployed-module presentation | exact five-transition identity assertion; Dashboard absent from Core destination resolver; one manifest contribution | inventory exposes Dashboard only as its live 3.0.0 Release/Instance and navigation exposes it exactly once | UAT-8A-02/06 |
| Exact Component/Dashboard acceptance seed | Missing or non-executable lifecycle coverage | exact seven-shell/eight-version/seven-placement identity assertions, including predecessor/successor and three action placements | every placement resolves through Components v3; exact action fixtures are live; zero old owners/types | UAT-8A-02/05 |
| Typed Core Dataset compatibility references/contracts | Direct Dataset relationship retained | typed composite identity, exact version, source/credential audits | Component authors/renders across Core contract only | UAT-8A-04 |
| Preserve Dashboard public Components behavior | Contract or consumer-policy regression | Sprint 7B regression suites plus exact structured dependency diagnostic | Defer/Upgrade/Replace/Remove, nondisclosure, five-placement Component outage and zero-finding recovery across real provider | UAT-8A-05 |
| Rebuild seed/test data through owners | Stale fixtures or cross-owner writes | semantic owner/read-back/idempotence and negative credential checks | recognizable full reference app from owning APIs | UAT-8A-02/06 |
| Outage, timeout, scope, capability and compatibility | Leak, corruption or cascading failure | authorization/nondisclosure matrix and failure-containment tests | stop/degrade Dataset or Component; unrelated services healthy; recovery converges | UAT-8A-04/05 |
| UI unchanged plus module configuration/diagnostics | Visual/interaction or configuration regression | Playwright at 1280/768/390, dark/light, keyboard, 200%, no-JS, hydration and console | direct/shell documents and exact configuration/diagnostics | UAT-8A-01/03 |
| Exit: fresh seed contains only new Component references | Wrong owner/type or incomplete seed | source-exact semantic read-back and old-input absence/rejection | all Dashboard placements resolve against selected Component instance | UAT-8A-02 |
| Exit: author/execute Components through Dataset contract | Product parity, authorization or isolation failure | every Component kind, scope, timeout, contract and credential tests | author and execute recognizable fixtures through gateway | UAT-8A-04 |
| Exit: Dashboard continues consumption and degrades coherently | Cascading outage or disclosure | Dashboard dependency/lifecycle/outage regression | stop/recover Component and Dataset provider paths | UAT-8A-05 |
| Canonical Component API break | Stale first-party consumer or hidden translation | exact v3 request/response golden tests and old-shape failures | UI, Dashboard, seed, smoke and UAT use v3 only | UAT-8A-01/05/06 |
| Exact configuration product decisions | Label/timeout drift or over-broad authority | schema v1, default/range, normalization, display and authority tests | navigation/Module Management label and 5-second effective timeout | UAT-8A-03/04 |

Every row requires automated and manual evidence. No `N/A` is currently
planned. If a proof becomes unsafe or inapplicable, amend this inventory before
freeze with a concrete rationale and equal-or-stronger evidence.

## Required Evidence Inventory

| Artifact | Producer | Required before | Status |
|---|---|---|---|
| `validation-readiness-result.json` | Validation coordinator | Rehearsal | Not Run |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | Not Run |
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
| `uat-result.json` | UAT | Authorization | Not Run |
| `uat-defect-harvest.json` | UAT/coordinator | Correction batch, when triggered | Planned / Conditional |
| `defect-batch.json` | Coordinator | Impact assessment, when triggered | Planned / Conditional |
| `correction-impact-assessment.json` | Coordinator | Focused repair validation, when triggered | Planned / Conditional |
| `focused-repair-validation/attempt-<n>.json` | SIT/UAT/coordinator | Convergence, when triggered | Planned / Conditional |
| `canonical-restoration.json` | Coordinator | Convergence/final certification, when triggered | Planned / Conditional |
| `final-certification-entry.json` | Coordinator | Final readiness/rehearsal, when triggered | Planned / Conditional |
| `evidence-manifest.json` and `.sha256` sidecar | Validation phases | Authorization | Not Run |
| `closeout-authorization.json` | Coordinator | Closeout | Not Run |

Additional required evidence namespaces:

- `readiness/`: executable checklist, destructive-target authorization,
  tool/shell/runner self-tests, semantic fixture audit, acceptance mapping and
  clean-source proof.
- `rehearsal/`: static, Rust, Playwright, source-exact fresh deployment,
  contracts/nondisclosure, smoke, teardown/rerun and automated-UAT diagnostics.
- `sit/`: lane receipts, raw logs, deployment/provenance, empty baseline,
  first/no-op bootstrap, old-input rejection, outage/recovery,
  upgrade/rollback and final convergence.
- `uat/manual/`: UAT-8A-01 through UAT-8A-08 receipts, screenshots/traces and
  reviewer sign-off.
- `materialization/`: resolved destructive targets, empty baseline,
  owner-bootstrap order/read-back, first/no-op results, induced failed attempt,
  teardown proof and successor from-empty result.
- `upgrade-rollback/`: baseline/candidate/rollback identities and unrelated
  service digest/restart chronology.

## Candidate Identity

- Implementation commit: To be frozen after passing readiness and rehearsal.
- Tree: To be frozen.
- Dirty state: Must be clean at freeze.
- Candidate fingerprint: Not issued.
- Acceptance-inventory identity: SHA-256 of this record plus tracked acceptance
  runners, tests, fixtures, seeds, manifests, bootstrap and deployment inputs.
- Deployment profile/configuration digest: planned
  `deploy/sprint-8a/compose.yaml`, override and normalized environment contract.
- Schema/baseline identity: empty Core, Component, Dashboard, Supervisor and
  selected-module schema migration digests plus bootstrap/seed contract
  versions. No legacy populated-data baseline exists.
- Expected provenance labels: OCI revision, source tree, dirty state, build
  profile, module definition/release, manifest, asset, schema migration,
  contract, fixture, seed and acceptance-inventory digests.
- Observed image digests: Not Run.

## Validation Readiness

- Derived executable checklist: regenerate from the final plan, this inventory,
  source, runners, Compose profile and evidence schemas.
- Environment variables and reset acknowledgements: enumerate exact Sprint 8A
  installation ID, gateway/Supervisor ports, database URLs/role identities,
  signing key IDs, service endpoints, Compose project, every disposable
  database/container/volume target, explicit destructive authorization and
  evidence root without recording secrets.
- Supported tools, shells and runtimes: PowerShell runner parsing/invocation,
  Rust/Cargo, wasm tooling, Node/npm/Playwright, Docker/Compose, PostgreSQL
  client, browser and SHA-256 helpers.
- Ports, topology, health and provenance: one gateway, Core, Supervisor,
  Component, Dashboard, Dataset compatibility provider and required reference
  services; isolated stores/roles; intended handoff slot; exact image labels.
- Semantic fixture and idempotence audit: administrator, global Module
  Management reader/manager, Component manager, scoped Dashboard manager,
  constrained reader, out-of-scope actor, service identities, all Component
  kinds, lifecycle states, Dataset major lines, allowed/blocked references,
  recognizable output, v3 and historical negative inputs, timeouts/outages and
  unchanged second-run behavior.
- Reset safety: resolve absolute Compose project and every exact disposable
  volume/database/container target; reject missing/ambiguous/broad targets and
  prove deletion cannot escape the declared environment.
- Runner/output/receipt/hash/finalization self-tests: safe disposable probes for
  every argument/output path, atomic completion, SHA-256, failure/supersession,
  heartbeat, teardown and completion sentinel.
- Acceptance mapping: every inventory row maps to SIT, smoke and UAT evidence.
- Clean repository and source-exact inputs: required before rehearsal.
- Result receipt: `artifacts/sprint-8a-closeout/validation-readiness-result.json`.

## Candidate Rehearsal

Sprint 8A rehearsal uses the dependency-aware inventory below. The attempt
receipt declares this graph before assertions. After any failure the attempt is
`harvesting`: independent siblings continue, dependent checks are recorded as
blocked with the exact failed prerequisite, and no tracked correction or new
attempt may begin until one harvest receipt and one consolidated defect batch
account for every row. `scripts/test-sprint-validation-harvest.ps1` enforces the
terminal-state and single-batch contract; materialization retains failed apply
responses and service logs before exact teardown.

| Diagnostic lane | Planned command/evidence | Assertions | Result | Defect batch |
|---|---|---|---|---|
| Static and boundaries | fmt/check/Clippy, manifests, links, native/WASM graphs, source/image audits | zero warnings; no forbidden owner/dependency/route/storage/legacy edge | Not Run | |
| Full Rust | `cargo test --workspace --locked` plus targeted contract/schema/authorization tests | all pass | Not Run | |
| Source-exact materialization | authorized reset, empty schemas, first/no-op owner bootstrap | exact provenance, healthy topology, semantic seed, exact no-op | Not Run | |
| Playwright | `scripts/validate-e2e.ps1` with exact gateway, deployment receipt, fresh-state, Sprint 8A profile, and evidence bindings | complete inventory; zero unexpected skip/retry/flake; retained outputs | Not Run | |
| Conformance and nondisclosure | module testkit plus Components/Dataset/Dashboard matrix | owner/version/scope/audience/known-random/timing/lifecycle cases pass | Not Run | |
| Deployed smoke | general and Sprint 8A smoke in rehearsal namespace | real boundaries, fixtures, old-input rejection, outage/recovery and final health | Not Run | |
| Live product diagnostics | attempt-bound product receipt plus raw structured Dashboard dependency JSON/SHA sidecar | exact predecessor/successor placements; Defer/Upgrade/Replace/Remove; blocked nondisclosure; five-placement Component outage; zero-finding recovery; partial evidence retained on failure | Not Run | |
| Component release transition | source-built `0.9.0` metadata plus Supervisor/Compose apply receipts and stage snapshots | exact one-owner `0.9.0`/`1.0.0` upgrade, rollback and restoration; Component preservation; unrelated identity stability | Not Run | |
| Failure teardown/rerun | induced partial materialization failure | evidence retained; exact topology/volumes removed; new empty rerun healthy | Not Run | |
| Automated UAT diagnostics | automated equivalents of UAT-01 through UAT-08, including the structured live-product receipt | every precondition and expected semantic state reproducible; UAT-8A-05 cannot pass from lane labels alone | Not Run | |

The first two lanes are independent of a deployed topology. Playwright locked
installation/discovery, runner self-tests, and acceptance-inventory checks are
also independent. Healthy materialization/no-op is the prerequisite for live
Playwright execution, deployed smoke, deployed inventory/navigation audit,
automated UAT diagnostics, failure-containment successor health, and the
Component upgrade/rollback baseline. Formal deployed acceptance smoke remains
SIT-owned and is not a rehearsal substitute.

The Sprint 8A reference materialization rebuilds the canonical seed through
signed owner bootstrap and receipt read-back before deployment evidence is
captured: exactly seven Component shells, eight versions, and seven Dashboard
placements, including the predecessor/successor and independent action
fixtures. The legacy helper prepares only account/security state. This is a
pre-production rebuild, not a compatibility migration. The complete Playwright
inventory must never rely on one earlier test file to create shared fixtures.
Exact deployed
inventory/navigation evidence is produced by
`scripts/audit-sprint-8a-deployed-inventory.ps1`; ad hoc route guesses, envelope
parsers, and copied counts are not rehearsal evidence.

Logical independence does not authorize concurrent Cargo commands to share the
repository `target` directory. `scripts/validate.ps1` cleans selected build
artifacts, so complete Rust siblings run serially unless each has a distinct
explicit `CARGO_TARGET_DIR`. Attempt 12 retained the missing-executable failure
that demonstrated this enforcement gap.

After removing the old Core-owned Component and Dashboard transition entries,
the fresh default Main placements after Datasets are Scoped Records at 7,
Components at 8, and Dashboard at 9. The Sprint 8A blueprint and acceptance
contract pin those exact identities and orders; Dashboard remains a single
manifest-owned contribution.

Readiness must run `npm ci --prefix end2end` and complete Playwright discovery,
not merely read the declared package version. It must also parse and self-test
the harvest guard and verify that `scripts/uat-sprint-8a.ps1` plus all eight
`docs/sprints/sprint-8a-uat/uat-8a-*.md` scripts exist.

Complete validation requires six pairwise-distinct, freshly created disposable
database bindings: API, fresh-baseline API, reference module, extracted
Component module, API enrollment, and Installation Control. A database
identity used by one owner integration target is not reused by another target
in the same rehearsal.

- Mutable source/environment identity: Not Run.
- Passing readiness prerequisite: Not Run.
- Consolidated defects and correction batch: Attempt 15 retained six
  Playwright findings in one fail-late batch after 10 independent checks
  passed. Four acceptance-inventory defects used the former Core Component
  `id`, the removed searchable Dataset Version picker, or stale generic Dataset
  dependency copy. One Component product defect disclosed a known unreadable
  identity as `403` instead of making it indistinguishable from a random `404`.
  One Dashboard product-asset defect served release `3.0.0` with entry imports
  pinned to `2.1.0`, preventing module hydration. The remaining 34 serial-suite
  cases were retained as blocked (2 Components, 8 Dashboards, 24 Permissions),
  and the all-prerequisite final check was blocked specifically by Playwright.
  Containment and all eight non-authoritative UAT diagnostic mappings still
  passed. The correction updates canonical identities/interactions, closes
  Component nondisclosure, binds Dashboard assets and digests to release
  `3.0.0`, and requires complete readiness and rehearsal restart. Two narrow
  dirty-source materialization invocations reached only the release build and
  were stopped by the external 15-minute command timeout; they are retained as
  diagnostic build evidence and do not replace either full gate. Attempt 14's
  two-finding, Attempt 13's ten-finding, and Attempt 12's eight-finding batches
  remain retained and superseded.
- Attempt 16 retained one deployment-input defect through three independent
  observations: Supervisor apply failed closed and both complete Rust suites
  reported the same Dashboard manifest/catalog mismatch. Tessara's manifest
  identity is the SHA-256 of RFC 8785/JCS canonical JSON produced by
  `tessara_composition::canonical_digest`, not a hash of platform-dependent
  working-tree bytes. The catalog now uses the canonical `cec6af45...` digest;
  the existing `checked_catalog_manifest_digests_match_runtime_manifests` test
  is the governing derived assertion and deliberately remains the single
  enforcement point.
- Attempt 17 retained two operator-harness defects as one batch after seven
  independent checks passed. Manual orchestration omitted the exact Core,
  gateway, and database container identities required by deployment-evidence
  capture, then invoked general smoke without its evidence/data-state bindings;
  the independent Sprint 8A semantic smoke still passed all 26 checks. The
  manual Component exercise also supplied the invented `-OutputImage`
  parameter instead of the baseline builder's declared `-OutputTag`. Playwright
  execution was blocked on deployment evidence; UAT diagnostics and final
  health were blocked on their declared failed prerequisites. Repository-owned
  wrappers now resolve these identities and arguments, and readiness parses and
  self-tests both wrappers so the commands cannot drift back to ad hoc manual
  assembly.
- Attempt 18 retained one operator-harness defect after four independent lanes
  passed: a 60-second client observation timeout terminated the active
  materialization build. No tracked input changed; the corrected execution used
  a non-terminating command boundary with short observation waits, and the
  complete readiness/rehearsal cycle restarted.
- Attempt 19 retained four first-failure findings after eight checks passed: the new general
  smoke wrapper incorrectly requested the forbidden Sprint 6A acceptance
  schema; one Component test still expected the removed projection-builder UI;
  Dashboard document deserialization expected `component_reference` where the
  canonical Components V3 metadata envelope exposes `reference`; and the
  permission suite inferred scoped Component fixtures from seed coincidence.
  Narrow correction verification then exposed one underlying product/seed
  finding in that same consolidated batch: demo seeding used a duplicate
  Dataset-major materializer that omitted the canonical restriction-tier and
  semantic-version columns, making freshly rebuilt seeded Components fail at
  execution. The batch therefore contains five defects, not a restarted
  rehearsal or a second micro-batch.
  The consolidated correction removes authoritative evidence publication from
  rehearsal smoke, tests the simplicity-first Configuration JSON form, aligns
  Dashboard document types with the V3 contract, and creates exact named scoped
  Component fixtures, and makes demo seed major lines conform to the canonical
  Dataset materialization schema. Playwright retained 33 passes, three first failures, and
  34 serial dependents; UAT diagnostics and final health were dependency-blocked.
- Attempt 20 retained three findings after nine checks passed. Playwright
  completed 48 tests, retained two first failures, and marked 20 serial
  dependents blocked: the Dashboard placement renderer still decoded the
  removed `dataset_id`/`dataset_version_major` response fields instead of the
  canonical Components V3 `dataset_reference`, and scoped Component execution
  disclosed a known hidden identity as `403` instead of the required
  nondisclosing `404`. The independently passing upgrade/rollback wrapper then
  exposed a harness handoff defect: a caller-selected evidence path was not
  accompanied by the canonical receipt consumed by UAT diagnostics. Live UAT
  smoke still passed, but scenario mapping failed on that exact missing
  prerequisite; the aggregate final check was blocked by Playwright and UAT.
  The consolidated correction aligns Dashboard render DTOs, makes scoped
  execution nondisclosing, and makes the upgrade wrapper atomically retain the
  canonical receipt plus any requested evidence copy.
- Attempt 21 retained two findings after ten checks passed. Playwright
  completed 48 tests, retained two first failures, and marked 20 serial
  dependents blocked. The deployed Dashboard image embedded the tracked
  `dashboard.wasm` generated before the Components V3 response change, so its
  decoder still required removed `dataset_id`/`dataset_version_major` fields
  even though the current Rust DTO and provider response used the canonical
  typed `dataset_reference`. The permission inventory also assumed Core's
  `#app-root` for independently deployed Component documents whose exact owned
  root is `#module-content`; the sibling JavaScript-disabled Component routes
  carried the same copied assumption. All other rehearsal lanes, including
  source-exact materialization, inventory/navigation, deployed smoke,
  upgrade/rollback, all eight UAT diagnostics, complete Rust validation,
  all-feature tests, Clippy, and containment health passed. The aggregate final
  check was blocked only by Playwright. The consolidated correction rebuilds
  and re-pins Dashboard bindings/WASM, tests the actual embedded asset hashes
  and V3 wire identity during readiness, and binds every Component route
  assertion to its exact module document root.
- Attempt 22 retained two findings after ten checks passed. Playwright completed
  52 tests, retained two first failures, and marked 16 serial dependents
  blocked. The Dashboard paging scenario selected the first available Table by
  type and then assumed every identity other than the Sprint 7A table had more
  than ten rows; the selected Sprint 8A reference table correctly had four.
  Separately, the independently deployed Component complete-document script
  installed its behavior but did not publish the shared
  `data-hydration="ready"` lifecycle marker. Source-exact materialization/no-op,
  inventory/navigation, deployed smoke/evidence, Sprint 8A semantic smoke,
  Component upgrade/rollback, all eight UAT diagnostics, complete Rust
  validation, all-feature tests, warning-free Clippy, exact Playwright
  discovery, and containment health passed. The aggregate final check was
  blocked only by Playwright. The consolidated correction binds paging to the
  exact multi-page demo Session Log Table and makes Component complete
  documents publish and test the shared hydration marker.
- Attempt 23 retained one harness finding after six independent checks passed
  and six deployment-dependent checks were blocked with exact reasons. The
  source-exact materialization command failed before destructive work when the
  documented interactive `-Confirm` prompt was invoked through the rehearsal
  evidence pipeline. Complete Rust validation, all-feature tests, warning-free
  Clippy, then-current exact Playwright inventory discovery, clean source, and current
  environment containment health passed. Inventory/navigation, deployed
  smoke, Playwright execution, Component upgrade/rollback, UAT diagnostics,
  and the aggregate final check were correctly blocked because no corrected
  source-exact topology existed. A captured-output `WhatIf` reproducer passed
  with `-Confirm:$false`; the correction makes that automation-safe invocation
  canonical and readiness-enforced without weakening the explicit disposable
  reset authorization or exact-target guards.
- Validation Readiness attempt 27 completed all 13 sibling checks and retained
  one harness failure in the new reset dry-run enforcement. The guarded
  `WhatIf -Confirm:$false` invocation returned successfully, but PowerShell
  rendered the `WhatIf` message directly through the host rather than into the
  assigned pipeline, and readiness incorrectly required that host-only text in
  the capture. The corrected check exercises the same capture boundary and
  requires successful return without treating host rendering as pipeline
  evidence. Candidate Rehearsal attempt 24 did not begin.
- Attempt 24 retained one shared harness/environment-contract finding after 11
  independent checks passed. Bare `npm test` used the Playwright development
  default at `127.0.0.1:8080`, so 30 tests failed with `ECONNREFUSED` and 40
  serial dependents did not run even though the source-exact Sprint 8A gateway
  was healthy at `127.0.0.1:8088`. Materialization/no-op,
  inventory/navigation, deployed smoke, Component upgrade/rollback, all eight
  UAT diagnostics, complete Rust validation, all-feature tests, warning-free
  Clippy, discovery, and containment health passed; only the aggregate final
  check was blocked. The correction makes the existing repository-owned
  `validate-e2e.ps1` runner canonical for rehearsal and makes readiness parse,
  self-test, and require its exact gateway/deployment/fresh-state/profile/
  evidence bindings.
- Attempt 25 retained two findings after 11 safe independent/dependency-valid
  checks passed. Playwright completed 52 tests, retained two first failures,
  and marked 16 serial dependents not run. The Dashboard paging scenario still
  named the removed Core demo Table instead of the canonical module-owned
  `sprint-8a-record-table`; the canonical four-tier Dataset also had only four
  rows and therefore could not prove paging. Narrow correction diagnosis kept
  the same batch open and exposed the underlying product contract gap: the
  Component provider discarded Dashboard cursors while Core rejected Dataset
  execution cursors and always returned no successor. The Component detail
  document separately allowed its version table to impose a 558-pixel
  min-content width in a 390-pixel viewport. The consolidated correction binds
  paging to the exact module identity, extends the canonical tier fixture with
  26 deterministic public rows, preserves/validates bounded forward cursors,
  performs deterministic limit-plus-one Dataset paging, and constrains the
  module page/panel grid items so the table owns its narrow overflow. The
  aggregate final check was blocked exactly because Playwright did not pass;
  all other lanes, including materialization/no-op, inventory/navigation,
  smoke, upgrade/rollback, all eight UAT diagnostics, complete Rust checks,
  Clippy, discovery, and containment health, passed.
- The restart-behavior audit found the protocol and readiness references now
  state the fail-late rule completely. The earlier enforcement gap was
  executable orchestration: ad hoc commands could bypass the dependency graph,
  harvest terminal-state guard, or source-exact Playwright wrapper. The
  repository-owned harvest guard, its readiness self-test, the declared attempt
  graph, and the readiness-enforced `validate-e2e.ps1` binding close that gap;
  no duplicate skill prose is added. Attempt 25 exercised this enforcement by
  retaining one harvest and one consolidated batch before tracked correction.
- Attempt 26 retained six findings after nine checks passed. Playwright
  completed 51 tests, retained three failures, and marked 16 serial dependents
  not run: the four named tier identities were incorrectly required on the
  first deterministic 25-row page; Core's generic module gateway dropped the
  browser query before the Dashboard module, so the embedded 10-row request
  became the Component provider's 25-row default; and the scoped Component
  route still expected two Core-era admin fetches even though module-owned SSR
  now hydrates without them. Materialization/no-op, exact inventory/navigation,
  deployed smoke, complete Rust validation, workspace all-features, warning-free
  Clippy, exact discovery, and containment health passed. The Component upgrade
  sibling failed before assertions because the operator supplied undeclared
  `-Overwrite` and was not retried. UAT live smoke passed, but its UAT-8A-08
  mapping incorrectly accepted a stale prior-source upgrade receipt. The first
  harvest-finalization call similarly used undeclared `-BatchPath`; immutable
  results were complete, so only finalization was rerun with the declared
  `-DefectBatchPath`. Those two operator findings require no tracked command
  change because the scripts already expose the exact parameters and the
  canonical command omits `-Overwrite`.
- The consolidated correction preserves exact query strings through the
  manifest-driven module gateway and tests that boundary, explicitly requests
  all 30 semantic rows for the tier identity assertion, removes stale admin-GET
  expectations from module-owned Component documents, stamps upgrade evidence
  with the clean source identity, and makes UAT diagnostics reject an upgrade
  receipt from another source or from before the current materialization. The
  UAT self-test exercises the stale-receipt rejection. Readiness 30 and
  rehearsal 26 are superseded by this tracked correction.
- Readiness 31 is superseded. Readiness 32 is invalid because it did not prove
  authenticated database reachability or issue a structurally shared
  environment contract. Rehearsals 27 and 28 were explicitly abandoned when
  the user exited testing; neither reached a complete terminal graph or may be
  resumed. Attempt 28's three partial harness/evidence findings and raw logs
  are retained under the invalidation receipt named above. The subsequent
  implementation audit and consolidated correction replace those attempts;
  they do not convert their partial evidence into a rehearsal result.
- Complete-cycle repetitions: 0.
- Result receipt: `artifacts/sprint-8a-closeout/candidate-rehearsal-result.json`.

Rehearsal is mutable, diagnostic and non-authoritative. Any correction requires
complete readiness and complete rehearsal to repeat before freeze.

## Environment Contract

- Environment fingerprint: Not issued.
- Intended gateway: `http://127.0.0.1:8088`; intended Supervisor endpoint:
  `http://127.0.0.1:8098`.
- Compose project/profile: `tessara-sprint-8a`,
  `deploy/sprint-8a/compose.yaml`, required profiles explicitly enabled.
- Validation database bindings: `TEST_API_DATABASE_URL`,
  `TEST_API_FRESH_DATABASE_URL`, `TEST_REFERENCE_MODULE_DATABASE_URL`,
  `TEST_COMPONENT_MODULE_DATABASE_URL`, `TEST_API_ENROLLMENT_DATABASE_URL`, and
  `TEST_INSTALLATION_CONTROL_DATABASE_URL`. Readiness requires pairwise-
  distinct loopback, token-bounded test database identities and completes an
  authenticated transactional temporary-table round trip against each one.
- Materialized owner databases: empty disposable installation-scoped Core,
  Component, Dashboard, Supervisor, and selected-module databases with
  owner-specific runtime and schema-migration roles.
- Reset authorization: explicit approval bound to resolved absolute Sprint 8A
  project/container/volume/database identities only. Broad targets, unresolved
  variables and neighboring projects fail closed.
- Account/role fixtures: administrator, global Module Management reader/manager,
  Component manager, scoped Dashboard manager, constrained reader, out-of-scope
  actor and Core/Component/Dashboard service identities; no secrets in receipts.
- Evidence root/output mode: `artifacts/sprint-8a-closeout/`, repository-relative
  canonical paths, append-only attempts and atomic completion where supported.
- Identity binding: readiness writes a secret-free environment contract and
  SHA-256 fingerprint. Every rehearsal lane, raw artifact, UAT diagnostic, and
  aggregate result must match that fingerprint and the same source identity.

## Planned Commands

The validation coordinator enters through exactly these two top-level commands.
The runners own the complete check inventory, dependency graph, evidence paths,
and all subordinate commands; manually assembling lane commands is not a
Candidate Rehearsal and cannot authorize preflight.

```powershell
.\scripts\validate-sprint-8a-readiness.ps1 -Attempt <n>
.\scripts\run-sprint-8a-candidate-rehearsal.ps1 -Attempt <n> -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json"
```

Readiness requires and authenticates the six declared disposable database
URLs, proves their identities are pairwise distinct, records the normalized
Compose/tool/source contract, and issues the environment fingerprint consumed
by rehearsal. The complete runner serializes Cargo work that shares one target
directory and owns materialization/no-op, inventory/navigation, smoke,
Playwright, a non-acceptance live product diagnostic, Component
upgrade/rollback, deterministic failure containment, successor restoration,
automated scenario diagnostics, and final identity/health. The live diagnostic
runs before containment; containment destroys its disposable writes and proves
a fresh canonical successor. It cannot publish UAT evidence, and formal UAT is
not started by rehearsal.
Deployed acceptance smoke remains SIT-owned and manual scenarios remain
formal-UAT-owned.

## Preflight

- Status: Not Run.
- Receipt: `artifacts/sprint-8a-closeout/preflight-result.json`.
- Environment and reset authorization: must match passing readiness/rehearsal.
- Harness/inventory reconciliation: this record and every tracked runner, test,
  fixture, seed, manifest, bootstrap and deployment input match rehearsal.
- Bootstrap/no-op/teardown/rerun and post-rollback intended-release restoration
  commands: exact Sprint 8A commands frozen from passing rehearsal. Product
  data restoration or partial-materialization resume is not supported.
- Evidence paths and required-artifact audit: complete before freeze.
- Freeze output: `candidate.json` binds one immutable fingerprint; no validation
  result exists before it.

## SIT

| Lane | Prepare receipt | Command/evidence | Assertions | Result | Duration |
|---|---|---|---|---|---|
| Static and boundaries | `sit/attempts/static-<n>.json` | fmt/check/Clippy, manifest/link/schema, native/WASM/package/source/image/legacy audits | zero warnings and no forbidden edge | Not Run | |
| Rust workspace | `sit/attempts/rust-<n>.json` | `cargo test --workspace --locked` plus targeted contract/schema/authorization suites | all pass on frozen candidate | Not Run | |
| Playwright | `sit/attempts/playwright-<n>.json` | `npm --prefix .\end2end test` | complete inventory; UI/config/auth/outage/old-input cases pass | Not Run | |
| Deployed acceptance smoke | `sit/attempts/deployed-smoke-<n>.json` | source-exact reset/materialization, first/no-op, old rejection, outage/recovery, failure rerun and upgrade/rollback | exact candidate; receipts complete; current topology healthy | Not Run | |

- SIT result receipt: `artifacts/sprint-8a-closeout/sit-result.json`.
- Canonical topology restoration: current Sprint 8A Component release active;
  all services healthy; canonical fresh seed; only v3 Component references;
  no old reader, open failure or partial topology.

## UAT

### Scripted UAT

- Command: Not authorized in implementation or rehearsal. The specialized UAT
  workflow must bind the exact `candidate.json`, preflight receipt, passing SIT
  receipt, environment fingerprint, `http://127.0.0.1:8088` endpoint, and
  attempt-scoped evidence target before fixing the formal command.
- Start rule: only after authoritative SIT passes on the same fingerprint.
- Result: Not Run.
- Evidence: `artifacts/sprint-8a-closeout/uat/scripted/`.

### Manual UAT

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| UAT-8A-01 unchanged Component experience | Component manager/reader; fresh canonical seed | Browse, create/edit/publish/version, exercise lifecycle and every kind; capture light/dark at 1280/768/390, keyboard, 200% zoom, no-JS SSR, hydration and console | Same accepted product behavior and canonical vocabulary; module owns documents/assets; responsive/theme/accessibility evidence complete; disposal succeeds | Not Run | `uat/manual/uat-8a-01.json` plus named screenshots/trace/console record |
| UAT-8A-02 from-empty seed and new references | Operator; explicitly authorized empty Sprint 8A databases | Run source-exact materialization, inspect exact owner read-back, rerun unchanged, open seeded Dashboard | Exactly 7 Component shells, 8 versions and 7 placements; predecessor/successor/action fixtures correct; only selected Component instance/v3; second run no-op | Not Run | `uat/manual/uat-8a-02.json` |
| UAT-8A-03 configuration and diagnostics | Global Module Management manager/reader | Set label and valid timeout; try blank/long label, 0/31 timeout, unknown field/version; inspect navigation/admin/product headings/diagnostics | Manifest defaults are Components/5; label/range/authority correct; binding/provider/contract/compatibility/health/result diagnostics are sanitized | Not Run | `uat/manual/uat-8a-03.json` plus screenshots |
| UAT-8A-04 Dataset contract, scope and outage | Component manager plus scoped/out-of-scope actors; Dataset fixtures | Author/preview/execute; attempt wrong audience/scope; stop/timeout/recover Dataset provider; retry preserved editor | Typed reference only; allowed paths work; restricted cases do not disclose; outage is read-only with no pending write; recovery succeeds | Not Run | `uat/manual/uat-8a-04.json` plus screenshots |
| UAT-8A-05 Dashboard lifecycle and outages | Component manager, Dashboard manager/reader; exact predecessor-bound action placements | Refresh exact findings; Defer then Upgrade; independently Replace and Remove; verify blocked nondisclosure; stop/recover Component | Three lifecycle findings and all four actions are exact; blocked placement absent; outage covers exact five authorized remaining placements; recovery has zero findings | Not Run | `uat/manual/uat-8a-05.json` plus structured semantic receipt/screenshots |
| UAT-8A-06 isolation and unsupported old inputs | Operator plus authorized/restricted actors | Inspect images/graphs/credentials/Core absence; submit V1/V2, old owner/type and old payload requests | No forbidden source/storage edge; old inputs fail exact normal contract with nondisclosure; no adapter/ledger/fallback | Not Run | `uat/manual/uat-8a-06.json` |
| UAT-8A-07 failed materialization rerun | Operator; disposable authorized topology; induced owner-bootstrap failure | Run failing attempt, inspect evidence, verify exact teardown, remove fault and rerun | Failure retained; partial project/volumes absent; successor begins empty and reaches canonical health | Not Run | `uat/manual/uat-8a-07.json` |
| UAT-8A-08 Component-only upgrade/rollback | Operator/reviewer; healthy `1.0.0` current release plus source-built `0.9.0` fixture | Establish `0.9.0`, upgrade to `1.0.0`, roll back, restore `1.0.0` through Supervisor/Compose; compare snapshots | Exact one-owner health-gated deltas; distinct release/binary identities; Component state retained; unrelated identities/restarts unchanged; final topology healthy | Not Run | `uat/manual/uat-8a-08.json` plus upgrade receipt |

- UAT result receipt: `artifacts/sprint-8a-closeout/uat-result.json`.
- Final topology restoration: intended Sprint 8A route/slot and current
  Component release; healthy Core/Component/Dashboard/Dataset path; canonical
  fresh seed; only v3 live references; no open defect.

## Changed Integration Contracts

| Boundary | Planned change | Required negative proof |
|---|---|---|
| Component public contract | exact-current `3.0.0`, Module Instance owner and `tessara.components.component_version` | old/mixed/wrong-instance/cross-installation/malformed/unauthorized inputs fail closed |
| Component product API | canonical module-owned typed-reference bodies on stable same-origin product paths | old Core payloads rejected; no translation reader/facade |
| Core Dataset compatibility | typed Dataset-major-line reference and versioned catalog/schema/distinct/execution/compatibility operations | no private DTO/SQL/credential; wrong audience/scope/version and outage do not disclose or fall back |
| Module configuration and diagnostics | Manifest schema v1 defaults `Components`/`5`; real image command paths; selected Dataset binding compatibility/health observation | unknown schema/field, invalid label, 0/31 timeout and wrong authority rejected; no raw references/secrets; command paths exist in image |
| Dashboard dependency | v3 provider binding; exact predecessor/successor and action placements; structured semantic evidence | no old owner/type or blocked-scope disclosure; Defer/Upgrade/Replace/Remove and Component outage/recovery cannot pass from broad labels |
| Fresh bootstrap | destructive full reset and owner-ordered exact 7-shell/8-version/7-placement idempotent seed | ambiguous target rejected; no cross-owner writes; exact predecessor/successor/action identities; second run no-op; failed partial topology never reused |
| Component release transition | source-built compatible `0.9.0` and candidate `1.0.0`; exact Supervisor/Compose one-owner deltas | candidate relabel rejected; release/binary identities distinct; unrelated owners absent from plan and unchanged in snapshots |
| Core ownership | generic platform integration only | Component product storage/code/routes/adapter/readers absent |

## Source Provenance, Schema Baseline, And Evidence Rules

- Freeze inputs include commit/tree/dirty state and every tracked product,
  test, harness, schema migration, fixture, seed, manifest, bootstrap,
  deployment, configuration, contract and acceptance identity.
- Candidate provenance matches image labels, release/instance read-back,
  contract/manifest/asset/schema/seed digests and deployed diagnostics.
- The baseline is empty, owner-specific schema materialization. No populated
  Core Component baseline, product-data mapping or legacy reference ledger is
  a candidate input.
- Seed assertions use semantic identity sets. Any contractual exact count comes
  from one shared acceptance source of truth.
- Receipts record commands, timestamps, duration, exit status, assertions, raw
  paths/hashes, prerequisites, candidate/environment identity, classification,
  invalidation, teardown and restoration.
- Secrets, credentials, private keys and token values never enter receipts.

## Failure And Invalidation Chronology

| Cause | Minimum invalidation |
|---|---|
| Candidate fingerprint changed | All SIT and UAT |
| Acceptance inventory or tracked harness changed | All SIT and UAT |
| Shared environment materially changed | Affected lane and downstream phases |
| Lane-local setup failure before assertions | Failed lane after prerequisites reconfirm |
| Evidence finalization failure with immutable complete raw results | Finalization only |
| Test assertion failure with unchanged candidate | Complete failed lane; coordinator assesses upstream relevance |
| UAT scenario setup failure before product actions | Affected isolated scenario set when prerequisites reconfirm |
| Product defect corrected | Refreeze after complete readiness/rehearsal, then all SIT and UAT |
| Missing acceptance assertion discovered | Update inventory/candidate, then all SIT and UAT |

Runtime chronology, initially empty:

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction/narrow proof | Invalidation scope | Authoritative replacement |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

Classifications are exactly `preflight/setup`, `product`, `harness`,
`environment`, `flaky`, `evidence-finalization`, or `product-decision`.
Product decisions pause for user direction.

## Post-SIT Defect Convergence (Conditional)

- On the first candidate-invalidating UAT failure, retain the failure,
  invalidate the candidate, forbid passing `uat-result.json`, and finish only
  safe independent scenarios as non-authoritative diagnostics.
- Write `uat-defect-harvest.json`, consolidate `defect-batch.json`, restore the
  canonical fresh topology, and create
  `correction-impact-assessment.json`.
- Identity, authorization, protocol, schema, seed/bootstrap, shared fixture/
  environment and cross-module changes default to a broad cone.
- Focused repair attempts diagnose mutable corrections only. They never
  authorize a candidate, SIT, UAT or closeout.
- After convergence, record restoration and final-certification entry, then
  rerun complete readiness, rehearsal, preflight, SIT and UAT for a successor.

## Evidence Integrity

- Required files complete: Not Run.
- Structured artifacts parse: Not Run.
- Markdown links pass: Not Run.
- Authoritative/superseded attempts distinguished: Not Run.
- Manifest file count: Not Run.
- Manifest SHA-256: Not Run.

Superseded failures remain retained and explicitly excluded rather than
overwritten. Every phase result names and hashes its prerequisites. One
immutable fingerprint covers all authoritative SIT and UAT evidence.

## Closeout Authorization

- Status: Not Authorized.
- Authorization receipt:
  `artifacts/sprint-8a-closeout/closeout-authorization.json` (not created).
- Authorized candidate/fingerprint: None.
- SIT passed: No.
- UAT passed: No.
- Acceptance mapping complete: Planned, not executed.
- Invalidation decisions satisfied: N/A before execution.
- Unresolved product decisions: None.
- Intended active route/slot: source-exact Sprint 8A gateway with current
  Component release and canonical fresh seed.
- Application health: Not evaluated during planning.
- Evidence source commit: Not frozen.
- Documentation commit: Not created.
- Authorization timestamp: None.

Closeout is forbidden until the validation coordinator verifies the complete
hashed chain and writes authorization. This planned record implies no candidate,
deployment, SIT, UAT or product result.

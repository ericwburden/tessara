# Sprint 7A Validation Record

- Sprint: `Sprint 7A: Scoped Analytics And Cross-Module Authorization Slice`
- Status: Candidate 14 invalidated during SIT; mutable readiness/rehearsal correction cycle executing
- Candidate: Candidate 14 `8f388886a0d440e5d48c8fabf8168bd59ef4be7a65f578494312aada2f0595b1` (invalidated)
- Closeout: Not Authorized
- Evidence root: `artifacts/sprint-7a-closeout/`

This record is the planned acceptance inventory. Kickoff does not execute
preflight, candidate freeze, SIT, deployed smoke, UAT, or closeout.

## Implementation Verification (2026-08-02)

This evidence is an implementation confidence check only. It does not freeze a
candidate or satisfy the formal SIT/UAT/closeout regime below.

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --all-targets --locked`: passed.
- Focused protocol, Components contract, analytics authorization, Dashboard
  module/UI, shared viewer, migration checksum, and catalog digest tests passed.
- Core API library tests passed `163/163` after filtering the two explicitly
  database-environment-gated integration tests; those require
  `TEST_API_DATABASE_URL` and `TEST_API_ENROLLMENT_DATABASE_URL`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`:
  passed after correcting every reported warning without suppressions.
- The implementation was re-audited under the forward-only implementation
  skill pulled from `main`. The audit removed the Dashboard-specific route
  variant from the shared viewer, advanced all repository authorization grant
  producers/consumers and canonical fixtures to `AuthorizationGrantV2`, and
  removed the duplicate direct Core administration port.
- The web-crate boundary, module-SDK boundary, and exact compatibility tuple
  scripts passed with no findings.
- Focused all-feature tests for the contract, composition, Components contract,
  Dashboard module, shared viewer, Components web crate, Supervisor, and
  reference module SDK passed. A full all-workspace/all-feature test attempt was
  blocked by an MSVC `link.exe` internal `LNK1000` crash while linking the
  `tessara-api` `hierarchy_metadata_fields` test executable; the same source
  passed workspace check and strict Clippy, so this remains an environment/tool
  limitation rather than a product-test assertion failure.
- Dashboard `2.1.0` release WASM and bindings were rebuilt; manifest and
  catalog digests were verified.
- Fresh source-exact Sprint 7A reference apply succeeded. The subsequent
  unchanged apply returned `no_op=true` with receipt digest
  `sha256:68db214639016049aed90733ffb6c5542794e4df1c1b843033f63df6b806111a`.
- Placement-owned stat-card and table renders passed through the `8086` public
  gateway, Core's module proxy, and the real Dashboard process. The stat value
  was `1`; the table returned `Reference row`; both reported ready
  materialization.
- Core retained one Dashboard service identity and four consumed one-time
  nonces after the two renders (metadata resolution plus render for each).
- The disposable reference topology uses gateway `8086` and Supervisor `8096`.
  Docker discovery is constrained to the Sprint 7A Compose project so retained
  profiles cannot contribute colliding route/service identities.

## Scope And Acceptance Inventory

| Roadmap clause | Risk / contract | Automated proof | Deployed smoke proof | Manual UAT proof | Status |
|---|---|---|---|---|---|
| Dataset previews enforce authored scoped restriction rules | Blocked rows or tier contributions leak when capability and scope are evaluated separately | Dataset integration fixture asserts source predicates precede count/filter/page/aggregate and tier authority covers the same governing roots | Focused smoke compares exact admin/scoped row values and counts | UAT-02 Dataset tier rows | Not Run |
| Component execution enforces authored scoped restriction rules | Table/chart/stat output includes blocked rows or aggregates | Component runtime integration tests for all presentation kinds, search, filters, pagination, and aggregation | Execute seeded table/chart/stat as admin and scoped actors | UAT-03 Component execution | Not Run |
| Dashboard viewing enforces authored scoped restriction rules | Dashboard scope or placement reference becomes implicit Component authority | Dashboard/Core integration proves joint Dashboard and Component decisions | View seeded mixed-placement Dashboard through real module/gateway boundary | UAT-04 Dashboard viewing | Not Run |
| Dataset and revision metadata is scoped | Names, revisions, fields, policy, counts, links, or scope nodes leak | API known/random and filtered-count tests | Directory/detail/revision requests under admin/scoped/no-access sessions | UAT-01 Scoped Dataset catalog | Not Run |
| ComponentVersion and linked presentation metadata is scoped | Hidden Dataset/Component identity leaks through linked assets | Provider decision and DTO redaction tests | Direct and Dashboard adapter metadata calls for blocked known/random IDs | UAT-03 and UAT-06 | Not Run |
| Dashboard composition metadata is scoped | Placement titles, types, references, or available catalog reveal blocked Components | Restricted resolution serialization and SSR/bootstrap/DOM absence tests | Dashboard detail/editor/viewer response and rendered HTML audit | UAT-04 Dashboard viewing | Not Run |
| Propagate installation and original actor | Cross-installation or actor substitution | Signed fixture and live receipt assertions | Capture exact successful exchange without secret/grant bytes | UAT-05 Cross-boundary recovery | Not Run |
| Propagate Dashboard presenting-service identity | Confused deputy; Core and Dashboard identities collapse | Wrong-presenter and downstream-exchange integration tests | Real Dashboard-presented render succeeds; forged presenter fails | UAT-05 and UAT-07 | Not Run |
| Bind declared compatibility dependency, contract, and action | An installed but undeclared service invokes Components | Wrong binding/contract/action/operation matrix | Live forged calls return one stable restricted result | UAT-07 Service misuse | Not Run |
| Verify scope-bound grants/Core decisions and downstream audience | Capability set and scope set form a cross-product or audience is replayed | Protocol plus mixed-scope provider/consumer tests | Exact grant/decision receipt fields and wrong-audience failure | UAT-05–UAT-07 | Not Run |
| Verify freshness | Old role, scope, ownership/visibility, Organization, or delegation authority remains usable | Issue-mutate-replay integration matrix | Live mutation invalidates retained grant; fresh request reflects change | UAT-08 Freshness | Not Run |
| Blocked rows, entities, metadata, and Dashboard content | Partial redaction still leaks values or counts | Negative assertions over response, SQL result, bootstrap, DOM, and network payloads | Scoped actor exact expected inventory and recognizable blocked sentinels absent | UAT-02–UAT-06 | Not Run |
| Mixed capabilities on disjoint Organization subtrees | `read@A × restricted@B` or `component@A × dashboard@B` is accepted | Dedicated disjoint-role fixture across Dataset, Component, Dashboard, and adapter | Focused conformance smoke with positive controls for A and B | UAT-06 Disjoint negatives | Not Run |
| Undeclared and wrong-audience/action services fail closed | User authority can be exchanged/replayed by another service | Provider-neutral conformance negative matrix | Live wrong-service/action/audience requests produce no execution or metadata | UAT-07 Service misuse | Not Run |
| Known versus random identifiers meet 6B non-disclosure profile | Status/body/header/timing reveals resource existence | Optimized balanced known/random runner and schema/publication self-test | Retained JSON and SHA-256 for Dataset, ComponentVersion, Dashboard states | UAT-06 known/random comparison | Not Run |
| Deprecated analytical endpoints remain adapter-only | Compatibility route becomes new product authority or bypass | Static route/dependency audit and shared-decision integration tests | Baseline smoke inventory unchanged | UAT-09 Compatibility | Not Run |
| Dataset and Component paths move toward extractable boundaries | New policy remains embedded in large route/runtime files | Boundary check for typed dto/service/repo decision seams | Conformance runner invokes typed provider adapter, not private DB access | UAT-09 Compatibility | Not Run |
| Empty, unavailable, and forbidden states are clear and non-leaking | Generic failures are confusing or detailed failures disclose metadata | SSR copy, accessibility, redaction, hydration, and console tests | Fault injection plus focused browser smoke | UAT-01–UAT-05 and UAT-10 | Not Run |
| Existing Dataset, Component, and Dashboard surfaces remain usable | Security fix breaks normal application tasks | Full API/workspace/Playwright regression | General smoke and scripted UAT | UAT-01–UAT-05 and UAT-11 | Not Run |
| Operators receive understandable cross-module failure states | Provider outage strands user or erases shell context | Dashboard degraded-state and retry tests | Stop/restore provider; shell, route, recovery link, and retry remain useful | UAT-05 and UAT-10 | Not Run |
| Scoped operator can preview Datasets, execute/view Components, and view Dashboards | End-to-end exit is proved only in isolated layers | Complete scoped analytics integration scenario | Focused deployed acceptance through gateway and real Dashboard process | UAT-01–UAT-05 | Not Run |
| Administrator sees the full seeded analytical set | Fix accidentally narrows global administrator authority | Exact admin inventory and all-tier results | Admin focused smoke exact assets/rows/placements | UAT-11 Administrator control | Not Run |
| Phase 8 extractions can rerun the proof | Tests are tied to Core-private implementation | Provider-neutral conformance self-test and documented adapter inputs | Run conformance against current transition provider | UAT-09 Compatibility | Not Run |

## Required Evidence Inventory

| Artifact | Producer | Required before | Planned path / rule | Status |
|---|---|---|---|---|
| `validation-readiness-result.json` | Validation coordinator | Rehearsal | Complete derived checklist; passing and hashed | Not Run |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | Complete non-authoritative validation-shaped pass; passing and hashed | Not Run |
| `preflight-result.json` | Validation preflight | Candidate freeze | Evidence root; passing and hashed | Not Run |
| `candidate.json` | Validation preflight | SIT | Exact clean implementation commit/tree and fingerprint | Not Run |
| Phase attempt receipts | Every phase/lane | Result collection | `attempts/<phase>-<attempt>.json`; authoritative flag explicit | Not Run |
| Static/boundary logs | SIT | SIT result | Commands, start/end, exit status, raw logs | Not Run |
| Rust workspace logs/results | SIT | SIT result | Locked workspace plus targeted suites | Not Run |
| Source-exact build/provenance receipt | SIT | Deployed lanes | Image digests and commit/tree/dirty labels | Not Run |
| Fresh apply/materialization receipt | SIT | Deployed lanes | Exact catalog/Blueprint/lockfile/plan/authorization/operation identities | Not Run |
| Idempotent second-run receipt | SIT | Deployed lanes | Stable plan/lockfile and `no_op=true`; exact owner receipt cardinality | Not Run |
| Authorization exchange evidence | SIT | Deployed smoke | Non-secret installation/actor/service/audience/binding/contract/action/revision identities | Not Run |
| Analytics conformance report | SIT | Deployed smoke | Positive and full wrong-service/disjoint/stale matrix | Not Run |
| Nondisclosure JSON and sidecar | SIT | SIT result | Known/random exact shape plus warmed timing profile | Not Run |
| `smoke.json` and sidecar | SIT | SIT result | General deployed acceptance; smoke belongs to SIT | Not Run |
| `smoke-sprint-7a.json` and sidecar | SIT | SIT result | Focused analytics acceptance and fault recovery | Not Run |
| Playwright report/evidence | SIT | SIT result | Exact acceptance inventory, SSR/hydration/console/network assertions | Not Run |
| `sit-result.json` | SIT | UAT | Passing and bound to candidate/environment fingerprints | Not Run |
| Scripted UAT evidence | UAT | UAT result | General plus Sprint 7A focused commands | Not Run |
| Manual scenario evidence | UAT | UAT result | One retained result per UAT-01 through UAT-11 | Not Run |
| `uat-result.json` | UAT | Authorization | Passing and hashes `sit-result.json` | Not Run |
| Canonical restoration receipt | SIT/UAT | Authorization | Complete reference composition, Core 200, Supervisor 204, modules healthy | Not Run |
| `evidence-manifest.json` and SHA-256 sidecar | Coordinator | Authorization | Every retained file parsed/hashed; superseded attempts distinguished | Not Run |
| `closeout-authorization.json` | Coordinator | Closeout | Hashes prerequisite receipts and exact authorized candidate | Not Run |

## Candidate Identity

- Implementation commit: `c14823da7a14a9d21beecc67696a4974c87654ac`
- Tree: `d225d22dc379a133015d306140a642284160c92e`
- Dirty state: `false`
- Candidate fingerprint: `526f4392c4374faa824ead268a47f920e79857a619f4202102fc43e737d7fe07`
- Acceptance-inventory identity: `95a57efc05c9344bfb03a208d39020a2a1a031ac52a1506eb5087ab950941943`
- Deployment profile/configuration digest: `50d6a7dd72d1e469d4ef0dc9ad1bc0790e854cf5b283043ff59c10444dca2f78`
- Migration/baseline identity: `a701c08eaa53325676016d140fbe5a5b1d225b6c23c00966314f22f75fdd7a23`
- Expected provenance labels: repository URL, candidate commit, candidate tree,
  `dirty=false`, build profile, Core/gateway/Dashboard/Scoped Records/Supervisor
  component identity
- Observed image digest(s): Supervisor `sha256:daabbc369eafbfc0f35b510f13eec665ea17e876916800a2902035d71aa89c41`;
  Core `sha256:9f59a9063253e06de7c347aa1d669d91defcb70e6e0fd63023ceb253cb213c1a`;
  Scoped Records `sha256:8e22d9d2494da53c5f4faad4549de8c69ed27eeb07b345609fc2a2c4df8ae86e`;
  Dashboard `sha256:66fb43b551b9929c9ea3df366eeb47be8090104e92e31df1e60df3b108c18570`
- Source rules: product, contract, test, fixture, migration, manifest,
  deployment, bootstrap, smoke, UAT, Playwright, conformance, or acceptance
  inventory changes create a new candidate. Documentation-only corrections may
  retain the fingerprint only when they cannot affect executable behavior or
  test interpretation.

## Environment Contract

- Environment fingerprint: `92db7bd283fc3cba14b20e6170316fde54b1c67508bcf6ae24b20346c0b50853`
- Host: Windows PowerShell orchestration with Docker Desktop/Linux containers
- Tool versions: record `git`, Rust/Cargo, wasm target/tooling, Node/npm,
  Playwright browsers, Docker/Compose, PostgreSQL client where used, and
  PowerShell versions without secret values
- Test database identities and reset authorization: disposable Sprint 7A Core,
  Dashboard, Scoped Records, and Supervisor stores; destructive reset must be
  explicitly authorized by preflight and limited to the Sprint 7A Compose
  project/volumes
- Compose project/profile and ports: source-exact `deploy/sprint-7a/` complete
  reference composition; public gateway `http://127.0.0.1:8086` and Supervisor
  `http://127.0.0.1:8096`; record exact rendered ports and intended active slots
- Services/topology: gateway, Core, out-of-process Supervisor, independent
  Dashboard process/database, Scoped Records process/database; Components and
  Datasets remain Core-owned transition providers
- Account/role fixture identities:
  - administrator with installation-global/full seeded analytics authority
  - scoped analytics operator with base/tier/Component/Dashboard authority on
    subtree A
  - mixed-scope operator with deliberately disjoint capability assignments on
    subtrees A and B
  - no-analytics authenticated actor
  - undeclared and wrong-service test identities/instances
  Passwords or tokens are supplied out of band and never retained.
- Product fixtures: four-tier recognizable Dataset rows; authorized and blocked
  Datasets/revisions; table/chart/stat ComponentVersions; authorized, blocked,
  and mixed-placement Dashboards; known and generated-random IDs; current and
  deliberately stale grant/decision specimens
- Evidence root and output mode: unique paths below
  `artifacts/sprint-7a-closeout/`; authoritative, diagnostic, and superseded
  attempts are distinct; writers refuse accidental overwrite unless the phase
  protocol explicitly supersedes an attempt.

## Changed Integration Contracts And Smoke Assertions

| Contract | Planned change | Required smoke assertion |
|---|---|---|
| Signed platform authorization grant/resource assertion | Version provider authority freshness and exact resource basis as needed | Current grant succeeds; old security/provider revision, altered resource, cross-installation, wrong presenter/audience/binding/action fail |
| Core-to-Dashboard product grant | Preserve exact Dashboard instance/action and scope bindings | Scoped Dashboard directory/detail/view succeeds only in scope; admin full set remains |
| Dashboard-to-Core Components compatibility contract | Make both `resolve_metadata` and `render` real action-bound exchanges | Real Dashboard process renders allowed placement; wrong service/action gets stable denial; blocked placement returns metadata-free result |
| Dataset/Component internal analytics decision DTO | Inseparable capability/scope/tier/resource decision | Disjoint fixture never forms cross-product; direct and embedded results match |
| Shared Component viewer endpoint contract | Allow Dashboard-owned mediated endpoint while preserving direct Core endpoint | Direct Component viewer and Dashboard embedded viewer both work; network audit shows correct route ownership |
| Provider authority revision/state | Bind ownership/visibility changes to freshness | Mutate visibility/authority, replay old decision fails, fresh decision reflects new scope |

## Preflight

- Status: Passed on attempt 3; no SIT or UAT assertions started
- Superseded attempts: `artifacts/sprint-7a-closeout/attempts/preflight-1.json`
  and `artifacts/sprint-7a-closeout/attempts/preflight-2.json`
- Authoritative attempt: `artifacts/sprint-7a-closeout/attempts/preflight-3.json`
- Receipt: `artifacts/sprint-7a-closeout/preflight-result.json`
- Candidate receipt: `artifacts/sprint-7a-closeout/candidate.json`
- Repository gate: clean `codex/sprint-7a`, intended commit, no untracked
  acceptance inputs, all submodule/dependency locks present
- Environment/reset authorization: verify exact Sprint 7A project, databases,
  volumes, ports, evidence root, and explicit fresh-reset permission before any
  destructive operation
- Harness/inventory reconciliation: every roadmap row above maps to implemented
  test IDs, smoke IDs, Playwright IDs, and UAT scenarios; exact contractual
  counts derive from one shared manifest. The tracked Playwright inventory is
  70 tests, including the Sprint 7A identities; eleven manual UAT scripts and
  the focused smoke, conformance, nondisclosure, semantic fixture preparation/
  verification, and scripted-UAT harnesses are present. Preflight accepts the
  harness only when `prepare-sprint-7a-uat-fixtures.ps1 -SelfTest` proves the
  tracked contract covers every actor, product fixture, identifier, and
  freshness category in the Environment Contract.
- Planned non-deployed commands:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace --all-features --locked`
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - targeted contract/API/Dashboard/UI tests from the sprint plan
  - conformance and nondisclosure `-SelfTest` commands
  - Sprint 7A smoke, semantic fixture, and scripted-UAT `-SelfTest` commands
  - boundary, Markdown-link, diff, and dependency-audit checks
- Bootstrap/no-op/restoration commands:
  - `.\scripts\bootstrap-sprint-7a-composition.ps1 -Composition reference`
  - `.\scripts\bootstrap-sprint-7a-composition.ps1 -Composition reference -SkipBuild`
  - restoration uses the same exact reference Blueprint/lockfile, or the
    retained Sprint 6F rollback inputs when rollback is the scenario
- Evidence paths and required artifact audit: declare every inventory row
  before freeze; no late acceptance assertion may be added without a new
  candidate and complete SIT/UAT restart.

## Validation Readiness And Candidate Rehearsal

- Test Readiness Gate: derive the Sprint 7A executable checklist from this
  record and the actual runners; verify exact environment/reset contracts,
  supported PowerShell runtimes and other tools, ports/Compose/databases/
  topology/health/provenance, semantic fixtures and idempotence, runner and
  evidence-finalization paths, and every acceptance-clause mapping.
- Candidate Rehearsal: run static/boundaries, full Rust and optimized timing,
  source-exact build/materialization/no-op, the complete Playwright inventory,
  conformance, nondisclosure, general and focused smoke, recovery/restoration,
  and automated diagnostic equivalents of UAT-01 through UAT-11.
- Authority: both gates are mutable and non-authoritative. They cannot satisfy
  SIT or UAT. Candidate 15 may freeze only after both complete passes are clean
  for the exact source and environment identities.
- Readiness receipt: attempt 1 failed and remains non-authoritative. The
  fail-late shell matrix found PowerShell 5.1 incompatibilities in shared JSON
  parsing, collection insertion, numeric type validation, file publication,
  path rejection, native-command preference handling, and non-ASCII script
  parsing. The consolidated mutable correction passes all 22 focused
  PowerShell 5.1/7.6 evidence self-tests; the complete gate must restart after
  the corrected clean commit.
- Rehearsal receipt: Not Run.

## SIT

| Lane | Planned command/evidence | Assertions | Result |
|---|---|---|---|
| Static and boundaries | fmt, check, clippy, boundary scripts, contract fixtures, Markdown links, diff, audit | Native SSR ownership, typed seams, no cross-module DB/credentials/URLs, exact protocol tuple | Not Run |
| Rust workspace | `cargo test --workspace --locked` plus targeted optimized nondisclosure test | All unit/integration/provider/consumer tests | Not Run |
| Source-exact deployment | Sprint 7A fresh build and reference bootstrap | Exact images/config/migrations/Blueprint/lockfile/plan/provenance; healthy topology | Not Run |
| Idempotent materialization | unchanged `-SkipBuild` bootstrap | Stable desired/actual identity, `no_op=true`, no duplicate fixtures/receipts | Not Run |
| Authorization conformance | `.\scripts\run-analytics-authorization-conformance.ps1` | Positive, disjoint, wrong-service/audience/action, stale, replay, recovery cases | Not Run |
| Nondisclosure | `.\scripts\validate-analytics-nondisclosure.ps1` | Dataset/ComponentVersion/Dashboard known-random shape and timing | Not Run |
| Playwright | `npm --prefix .\end2end test` or source-exact validation wrapper | Full inventory; route ownership, hydration, console, DOM/network leakage, responsive state | Not Run |
| Deployed acceptance smoke | general and Sprint 7A smoke scripts | Admin/scoped inventories, all three surfaces, real exchange, outage/restore, final health | Not Run |
| Recovery/rollback | provider fault restoration and retained Sprint 6F rollback input audit | Containment, canonical restoration, no unrelated image change | Not Run |

- SIT result receipt: `artifacts/sprint-7a-closeout/sit-result.json`
- Canonical topology restoration: complete Sprint 7A reference composition at
  `http://127.0.0.1:8086`, Core readiness 200, Supervisor readiness 204, exact
  intended module instances healthy/enabled, final smoke passed
- Rule: deployed acceptance smoke is part of SIT and must pass before UAT.

## UAT

### Scripted UAT

- Commands:
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:8086"`
  - `.\scripts\uat-sprint-7a.ps1 -BaseUrl "http://127.0.0.1:8086" -OutputPath "artifacts/sprint-7a-closeout/uat/scripted-sprint-7a.json"`
- Result: Not Run
- Evidence: structured JSON/sidecars plus append-only logs beneath the evidence
  root, bound to candidate and environment fingerprints

### Manual UAT

| Scenario | Role / start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| UAT-01 Scoped Dataset catalog | Scoped operator; fresh reference fixtures | Browse directory/detail/revision and direct blocked ID | Only scoped metadata; generic blocked state | Not Run | Planned |
| UAT-02 Dataset tier rows | Scoped operator; four-tier Dataset | Preview/query Dataset | Only same-scope authorized tiers; blocked sentinel/count absent | Not Run | Planned |
| UAT-03 Component execution | Scoped operator; table/chart/stat fixtures | Open and interact with each Component | Scoped rows/aggregates only; blocked version absent | Not Run | Planned |
| UAT-04 Dashboard viewing | Scoped operator; mixed-placement Dashboard | Open directory/detail/viewer | Allowed placement renders; blocked placement generic/redacted | Not Run | Planned |
| UAT-05 Cross-boundary recovery | Scoped operator; healthy provider then controlled outage | View, fault provider, observe, restore, retry | Exact exchange; contained failure; healthy recovery | Not Run | Planned |
| UAT-06 Disjoint and known/random | Mixed-scope operator | Exercise three surfaces and known/random direct paths | No cross-product; equal restricted public outcomes | Not Run | Planned |
| UAT-07 Service misuse | Test client identities | Wrong presenter/audience/binding/action/replay calls | Stable denial; no metadata/execution | Not Run | Planned |
| UAT-08 Freshness | Admin mutator plus scoped operator | Issue, mutate role/scope/visibility/delegation, replay, refresh | Old grant stale; fresh result correct | Not Run | Planned |
| UAT-09 Compatibility | Admin/operator; canonical topology | Direct Component UI, adapter, boundary audit | Existing UI works; no deprecated product surface; reusable gate named | Not Run | Planned |
| UAT-10 Responsive safe states | Scoped/no-access roles; desktop and narrow viewport | Exercise empty/forbidden/unavailable/recovery with keyboard | Useful SSR, accessible copy, no hydration/console leak | Not Run | Planned |
| UAT-11 Administrator control | Administrator; fresh reference fixtures | Browse and execute full analytics set | Exact full assets, rows, tiers, and placements visible | Not Run | Planned |

- UAT result receipt: `artifacts/sprint-7a-closeout/uat-result.json`
- Final topology restoration: same canonical state required by SIT; no
  intentionally stale role, stopped provider, altered scope, or forged service
  fixture remains active.

## Failure And Invalidation Chronology

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction / narrow proof | Invalidation scope | Authoritative replacement |
|---|---|---|---|---|---|---|---|
| 2026-08-02T18:08:47Z | Preflight / candidate readiness / preparing | No | Not Frozen | preflight/setup | Complete the missing Sprint 7A smoke, UAT, authorization-conformance, analytics-nondisclosure, and Playwright inventory; commit a clean implementation candidate; rebuild source-exact images | No SIT or UAT evidence exists; rerun preflight from the corrected clean commit | `artifacts/sprint-7a-closeout/attempts/preflight-1.json` |
| 2026-08-02T19:15:19Z | Preflight / audit / preparing | No | `526f4392...` | preflight/setup | Quote `HEAD^{tree}` as one literal Git revision; narrow proof returned tree `d225d22d...` | Preflight attempt only; candidate and environment unchanged | `artifacts/sprint-7a-closeout/attempts/preflight-2.json` |
| 2026-08-02T19:17:00Z | Preflight / candidate freeze / passed | No | `526f4392...` | — | All environment, harness, inventory, topology, provenance, and evidence-path checks passed | SIT and UAT remain not started and must bind to the exact fingerprints | `artifacts/sprint-7a-closeout/attempts/preflight-3.json` |
| 2026-08-03T12:00:00Z | UAT / manual fixture setup / preparing | No for affected scenarios | `37ccd808...` | harness | Candidate 11 exposed that script/file counts did not prove the promised semantic fixture inventory. Add the tracked fixture contract, idempotent preparer/live verifier, and behavior assertions. | Missing acceptance assertion changes the candidate and invalidates all prior SIT/UAT under the matrix below. | `artifacts/sprint-7a-closeout/attempts/uat-candidate-11.json` |
| 2026-08-03T10:22:50-04:00 | SIT / Playwright / executing | Yes | `8f388886...` | harness | Candidate 14's tracked Playwright assertion expected the superseded two-placement/one-row fixture; the live semantic contract correctly returned four placements and four tiers. The narrow three-test diagnostic reproduced exactly two failures and cannot replace the full lane. | Candidate 14 invalidated; failed evidence retained; canonical reference topology restored; stop before Candidate 15 and complete the new readiness/rehearsal cycle. | `artifacts/sprint-7a-closeout/attempts/sit-playwright-candidate-14-attempt-1.json` |
| 2026-08-03T10:30:00-04:00 | Validation Readiness / supported shells and evidence paths / executing | Yes, non-product self-tests | Mutable, pre-Candidate 15 | harness | Fail-late execution exposed a single PowerShell 5.1 portability batch across deployment, Playwright, nondisclosure, smoke/UAT acceptance evidence, and adversarial finalization paths. Focused correction proof passed all 22 checks under Windows PowerShell 5.1 and PowerShell 7.6. | No candidate exists. Keep source mutable, amend the one implementation commit, and repeat the complete readiness gate; focused proof cannot satisfy it. | `artifacts/sprint-7a-closeout/readiness/shell-selftests.log` |
| 2026-08-03T11:04:00-04:00 | Candidate Rehearsal / source-exact deployment / preparing | No | Mutable, pre-Candidate 15 | preflight/setup | The first Rust database reset submitted `DROP DATABASE` and `CREATE DATABASE` in one transaction, which PostgreSQL correctly rejected. Separate `dropdb --force` and `createdb` commands then produced the required exact disposable databases. | No candidate exists. Repeat the complete readiness gate and the complete rehearsal; the corrected reset is setup proof only. | `artifacts/sprint-7a-closeout/rehearsal/attempt-2-rust-workspace-rerun.log` |
| 2026-08-03T11:10:00-04:00 | Candidate Rehearsal / Playwright and deployed diagnostics / executing | Yes, non-authoritative | Mutable, pre-Candidate 15 | harness | Fail-late rehearsal found one Dashboard test that treated the canonical chart fixture as absent, plus general smoke and UAT scripts that still required the undeployed Module SDK reference route and navigation destination. Authorization conformance, nondisclosure, Sprint 7A smoke/UAT diagnostics, and recovery/restoration independently passed. | No candidate exists. Correct the three tracked harness mismatches as one batch, amend the clean implementation commit, then repeat the complete readiness gate and complete rehearsal. Narrow reproducers cannot replace either pass. | `artifacts/sprint-7a-closeout/rehearsal/attempt-2-general-diagnostic-lanes.json` |
| 2026-08-03T11:14:00-04:00 | Candidate Rehearsal / defect-batch narrow proof / executing | Yes, non-authoritative | Mutable, pre-Candidate 15 | preflight/setup | The first focused Playwright command used the default `8080` base URL and reached no rehearsal service; Core `8086` and Supervisor `8096` remained healthy. The exact-base-URL rerun and both corrected general diagnostic scripts passed. | Retain the setup failure separately. It does not change the defect batch or satisfy rehearsal; repeat the complete gates from the amended clean source. | `artifacts/sprint-7a-closeout/rehearsal/attempt-2-defect-batch-dashboard-focused-reproducer.log` |
| 2026-08-03T11:25:00-04:00 | Candidate Rehearsal / static evidence finalization / executing | Yes, non-authoritative | Mutable, pre-Candidate 15 | evidence-finalization | Two complete static/boundary runs emitted passing results, but their ad hoc collector read an internal native exit value left by a successful PowerShell script. Direct `pwsh -File` execution returned `0`, and the complete rerun receipt was finalized from the retained raw log. | Static lane only; product assertions and the readiness source identity are unchanged. | `artifacts/sprint-7a-closeout/rehearsal/attempt-3-static-boundaries-final.json` |
| 2026-08-03T11:26:00-04:00 | Candidate Rehearsal / Rust workspace / executing | Yes, non-authoritative | Mutable, pre-Candidate 15 | environment | The complete Rust command passed 163 non-database tests, while two database tests failed authentication because the readiness runner verified host, port, and database identity but omitted the exact disposable credential. | Invalidate the prior readiness pass; retain the Rust failure, correct readiness to authenticate every exact URL without retaining the credential, amend the validation record, then repeat the complete readiness gate and rehearsal. | `artifacts/sprint-7a-closeout/rehearsal/attempt-3-rust-workspace.log` |
| 2026-08-03T11:54:00-04:00 | Candidate Rehearsal / Playwright / executing | Yes, non-authoritative | Mutable, pre-Candidate 15 | environment | Thirty-seven tests passed and thirty serial dependents did not run after the lane supplied a Compose container name where acceptance mode requires the immutable 64-character PostgreSQL container ID; the two failures were cleanup/setup errors reporting a malformed binding. | Invalidate the prior readiness pass; retain the complete Playwright outputs, add active exact Playwright database-binding validation to readiness, amend the validation record, then repeat the complete readiness gate and rehearsal. | `artifacts/sprint-7a-closeout/rehearsal/attempt-4-playwright-results.json` |

Allowed classifications are exactly `preflight/setup`, `product`, `harness`,
`environment`, `flaky`, `evidence-finalization`, and `product-decision`.

The coordinator applies this minimum-safe matrix:

| Cause | Minimum invalidation |
|---|---|
| Candidate fingerprint changed | All SIT and UAT |
| Acceptance inventory or tracked harness changed | All SIT and UAT |
| Shared environment materially changed | Affected lane and downstream phases |
| Lane-local setup failed before assertions | Failed lane |
| Evidence finalization failed with complete immutable raw results | Finalization only |
| Test assertion failed with candidate unchanged | Complete failed lane; assess upstream environment relevance |
| UAT setup failed before product actions | Affected isolated scenario set after prerequisites reconfirm |
| Product defect corrected | Refreeze, all SIT, then all UAT |
| Missing acceptance assertion discovered | Update inventory/candidate, all SIT, then all UAT |

Every failure retains its receipt and raw evidence, records stage and
`assertions_started`, uses a narrow reproducer only for diagnosis, and marks
invalidated attempts superseded. Earlier evidence is reusable only with exact
candidate/environment receipt matches and an explicit non-impact rationale.

## Evidence Integrity

- Required files complete: Not Run
- Structured artifacts parse: Not Run
- Repository Markdown links pass: Not Run
- Authoritative/superseded attempts distinguished: Not Run
- Every retained file hashed: Not Run
- Manifest file count: Not Run
- Manifest SHA-256: Not Run
- Secret audit: receipts/manifests/logs must exclude passwords, tokens, signing
  secrets, private keys, browser cookies, database credentials, and grant bytes
- Long-running work: write start receipts, append logs continuously, retain
  heartbeats/durations/completion sentinels, and inspect retained completion
  state before relaunching after a lost controlling session

## Closeout Authorization

- Status: Not Authorized
- Authorization receipt: `artifacts/sprint-7a-closeout/closeout-authorization.json`
- Authorized candidate/fingerprint: Not Available
- Preflight passed before SIT: No
- SIT passed: No
- UAT passed after SIT: No
- Acceptance mapping complete: Planned, not proven
- Invalidation decisions satisfied: Not Applicable
- Unresolved product decisions: None at kickoff; implementation stop conditions
  are recorded in the sprint plan
- Intended active route/slot: complete Sprint 7A reference composition at
  `http://127.0.0.1:8086`
- Application health: Not Run
- Evidence source commit: Not Available
- Documentation commit: Not Available
- Authorization timestamp: Not Available

Closeout may be authorized only after every receipt parses and hashes, one
candidate fingerprint covers all authoritative evidence, all SIT lanes and
deployed smoke pass, every scripted/manual UAT scenario passes after SIT, the
canonical topology is restored and healthy, every roadmap clause has automated
and manual evidence, all invalidations are satisfied, and no acceptance defect
or product decision remains.

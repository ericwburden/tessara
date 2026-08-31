# Sprint 8C Validation Record

- Status: planned acceptance inventory; no validation phase has run.
- Validation policy: `tessara-validation-v2`
- Tracked validation contract: `docs/sprints/sprint-8c-validation-contract.json`
- Implementation profile: `phase8-module-extraction`
- Profile playbook: `docs/architecture/module-extraction-playbook.md`
- Module / transition: `tessara.responses` / `tessara.responses`
- Activation boundary: after Sprint 8A; no legacy evidence retrofit
- Evidence root: `artifacts/sprint-8c-closeout/` (ignored)

## Scope And Acceptance Inventory

| Clause | Risk / contract | Automated proof | Deployed smoke proof | Manual UAT proof |
| --- | --- | --- | --- | --- |
| AC-01 independent owner | accidental Core process/database ownership | `owner-product`, `inventory-navigation`, `migration-seed` | exact Response release/instance/database/health/config/diagnostics | UAT-8C-01, UAT-8C-07 |
| AC-02 product flow | lost lifecycle/audit or partial writes | owner integration plus Playwright product identities | start/save/submit/review through gateway | UAT-8C-01 |
| AC-03 typed providers | Form/Workflow table or DTO mirror coupling | `provider-contracts`, `contract-boundary` | real isolated provider processes and credential denial | UAT-8C-02 |
| AC-04 Dataset source | export/cursor drift or Core fallback | `dataset-export`, real Dataset refresh/rebase | submitted output appears from Response owner | UAT-8C-04 |
| AC-05 Workflow events | duplicate/reordered/lost event advancement | `workflow-events`, replay/order/interruption matrix | Response submit advances Workflow once | UAT-8C-04 |
| AC-06 assignment-only start | unmediated or stale assignment creates draft | `assignment-only`, existing literal acceptance identity | direct legacy start rejected; active assignment succeeds | UAT-8C-02 |
| AC-07 scoped review | existence or value disclosure | `scoped-review`, known/random/disjoint equality | owner/delegate/manager views stay bounded | UAT-8C-03 |
| AC-08 fresh seed/isolation | predicted IDs or shared credentials | `migration-seed`, `clean-materialization`, pairwise isolation | empty owner-ordered bootstrap and signed read-back | UAT-8C-07, UAT-8C-08 |
| AC-09 auth/replay/compatibility | grant/body/key/version substitution | contract and idempotency adversarial matrices | replay succeeds once; mismatches fail | UAT-8C-05 |
| AC-10 outage/recovery | false empty, partial promotion, secret leakage | provider/consumer one-shot fault matrix | last-good state and sanitized degradation recover | UAT-8C-05 |
| AC-11 unchanged UI | shell/theme/SSR/hydration/responsive drift | `ui-sdk-conformance`, visual matrix, clean console | direct and lifecycle Response pages | UAT-8C-01, UAT-8C-06 |
| AC-12 Core subtraction | hidden fallback/duplicate inventory | `core-subtraction`, boundary/schema search | three Core transitions and one Response enrollment | UAT-8C-08 |
| AC-13 semantic no-op | second apply changes identities/state/outbox | `semantic-noop` in clean environment | unchanged apply receipt/topology/health | UAT-8C-07 |
| AC-14 failed apply recovery | residue or manual repair | `failure-recovery` one-shot bounded fault | retained failure, exact teardown, clean successor | UAT-8C-09 |
| AC-15 upgrade/rollback | unrelated owner restart/data drift | source-built Response `0.9.0` transition target | upgrade/rollback/restore exact health | UAT-8C-10 |
| AC-16 roadmap exit | UI success masks shared storage/cut consumer | `consumer-cutover`, full Playwright and UAT predicates | complete/review/export through real modules | UAT-8C-11 |

Each clause has at least one automated assertion and manual scenario. Deployed
smoke is part of SIT; it is not a separate pre-SIT authorization.

## Required Evidence Inventory

| Artifact | Producer | Required before | Status |
| --- | --- | --- | --- |
| `implementation-readiness-result.json` | Implementation | Validation Readiness | Not Run |
| `validation-readiness-result.json` | Validation coordinator | Rehearsal | Not Run |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | Not Run |
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
| `uat-result.json` | UAT | Authorization | Not Run |
| per-failure sibling `defect-provenance.json` | Target/phase owner | correction or broad rerun | Planned / Conditional |
| `uat-defect-harvest.json` | UAT/coordinator | correction batch when triggered | Planned / Conditional |
| `defect-batch.json` | Coordinator | impact assessment when triggered | Planned / Conditional |
| `correction-impact-assessment.json` | Coordinator | focused repair validation when triggered | Planned / Conditional |
| `focused-repair-validation/attempt-<n>.json` | Coordinator/phase owners | convergence when triggered | Planned / Conditional |
| `canonical-restoration.json` | Coordinator | convergence/final certification when triggered | Planned / Conditional |
| `final-certification-entry.json` | Coordinator | final readiness/rehearsal when triggered | Planned / Conditional |
| per-phase `evidence-index.json` and SHA sidecar | Each phase | phase certificate | Not Run |
| `evidence-chain.json` and SHA sidecar | Coordinator | authorization | Not Run |
| `closeout-authorization.json` | Coordinator | Closeout | Not Run |

Every receipt binds source commit/tree/dirty state, validation-contract hash,
dependency fingerprints, environment fingerprint, exact command, fixture and
acceptance inventory, evidence index and cleanup/restoration state. Raw evidence
is retained cold beneath the ignored evidence root.

## Implementation Exit Gate

All targets are required. Commands must resolve to repository-owned runners at
implementation time; a missing selector or zero selected tests fails.

| Target | Proof classes | Affected domains | Exact command | Clean environment | Result |
| --- | --- | --- | --- | --- | --- |
| `static-quality` | static-quality | product/build/harness | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target static-quality` | No | Not Run |
| `contract-boundary` | contract-boundary | product/contract/harness | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target contract-boundary` | No | Not Run |
| `owner-product` | owner-product | product/migrations/fixtures | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target owner-product` | No | Not Run |
| `ui-sdk-conformance` | ui-sdk-conformance | product/acceptance/environment | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target ui-sdk-conformance` | Yes | Not Run |
| `consumer-cutover` | consumer-cutover | product/fixtures/acceptance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target consumer-cutover` | No | Not Run |
| `core-subtraction` | core-subtraction | product/migrations | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target core-subtraction` | No | Not Run |
| `inventory-navigation` | inventory-navigation | product/deployment | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target inventory-navigation` | No | Not Run |
| `migration-seed` | migration-seed | migrations/deployment/fixtures | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target migration-seed` | Yes | Not Run |
| `clean-materialization` | clean-materialization | all executable topology inputs | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target clean-materialization` | Yes | Not Run |
| `semantic-noop` | semantic-noop | migrations/deployment/fixtures | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target semantic-noop` | Yes | Not Run |
| `failure-recovery` | failure-recovery | migrations/deployment/environment | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target failure-recovery` | Yes | Not Run |
| `fixture-acceptance` | fixture-acceptance | fixtures/acceptance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target fixture-acceptance` | No | Not Run |
| `runner-selftest` | runner-selftest | validation/runners/evidence | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target runner-selftest` | No | Not Run |
| `deployed-smoke` | deployed-smoke | topology/fixtures/acceptance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target deployed-smoke` | No | Not Run |
| `independent-upgrade-rollback` | independent-upgrade-rollback | build/migrations/deployment | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target independent-upgrade-rollback` | Yes | Not Run |
| `uat-readiness` | uat-readiness | fixtures/acceptance/environment | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target uat-readiness` | No | Not Run |
| `provider-contracts` | contract-boundary, owner-product | provider/fixture/environment | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target provider-contracts` | No | Not Run |
| `workflow-events` | contract-boundary, consumer-cutover | product/migrations/topology | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target workflow-events` | Yes | Not Run |
| `dataset-export` | contract-boundary, consumer-cutover | product/migrations/topology | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target dataset-export` | Yes | Not Run |
| `assignment-only` | contract-boundary, owner-product | product/fixtures/acceptance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target assignment-only` | No | Not Run |
| `scoped-review` | contract-boundary, owner-product | product/fixtures/acceptance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target scoped-review` | No | Not Run |
| `api-idempotency` | contract-boundary, owner-product | product/migrations/fixtures | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target api-idempotency` | No | Not Run |
| `planning-contract-alignment` | contract-boundary | contract/acceptance/docs | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target planning-contract-alignment` | No | Not Run |

- Clean source and validation-contract hash: pending implementation exit.
- Materialization / first apply: exact `deploy/sprint-8c/compose.yaml` profile,
  `scripts/materialize-sprint-8c.ps1`, empty disposable databases.
- Semantic no-op: unchanged second invocation of the same materializer.
- Failure containment / recovery: one-shot authenticated mid-Response bootstrap
  fault, retained result, exact teardown, new from-empty apply.
- Fixture, runner, smoke and acceptance reproducers: exact target selectors
  above, full Playwright manifest and UAT predicate discovery.
- Known failure count: must be zero.
- Open/blocked defect-provenance records: must be zero.
- Formal fixture/environment/inventory/assertion contract: must be byte-identical
  to the implementation-exit selection.

## Candidate Identity

- Implementation commit: pending
- Tree: pending
- Dirty state: must be false
- Candidate fingerprint: pending Preflight
- Acceptance-inventory identity: SHA-256 of schema-valid
  `end2end/acceptance-manifest.json` plus Sprint 8C UAT scenario contract
- Deployment profile/configuration digest: `deploy/sprint-8c/compose.yaml` and
  resolved override/image/manifest digests
- Migration/baseline identity: Response `001_response_module.sql` plus each
  owner baseline digest
- Expected provenance labels: Core, Supervisor, Forms/Workflow transition
  providers, Response `1.0.0`, Dataset, Component, Dashboard
- Observed image digests: pending source-exact materialization

## Validation Readiness

- Derived checklist: contract/schema, source cleanliness, dependency impact,
  every implementation target, exact fixture/topology, environment tools,
  runner/finalizer/evidence self-tests, no open provenance.
- Environment/reset: explicit disposable database and Compose reset approval;
  no production or undeclared database identity.
- Tools: pinned Rust/Cargo, Node/npm/Playwright, PowerShell 7, Docker Compose,
  PostgreSQL clients and repository-owned validators.
- Ports/topology/health: resolved by `deploy/sprint-8c/compose.yaml`; no occupied
  or foreign-owned resource; exact owner endpoints and provenance.
- Semantic fixture/idempotence: logical-key owner read-back, first apply and
  no-op identities authenticated.
- Result receipt: Not Run.
- Dependency fingerprints / executed or inherited lanes / phase index: pending.

## Candidate Rehearsal

| Lane | Command / evidence | Required assertions | Result |
| --- | --- | --- | --- |
| Static/boundaries | Sprint runner static selector | format/Clippy/source/schema/Core subtraction | Not Run |
| Full Rust | `cargo test --workspace --locked` plus exact DB targets | all owner/provider/consumer tests, zero warnings | Not Run |
| Materialization | Sprint clean-environment runner | first apply/no-op/read-back/isolation | Not Run |
| Browser | `npm --prefix .\end2end test` | complete manifest, one worker, zero retries/skips | Not Run |
| Conformance | SDK and authorization runners | UI ownership, nondisclosure, exact contracts | Not Run |
| Providers/events/consumers | Sprint focused targets | Form/Workflow contexts, outbox, Dataset/reverse consumers | Not Run |
| Deployed smoke | `.\scripts\run-sprint-8c-deployed-smoke.ps1` | real routes/health/owner/output chain | Not Run |
| Recovery/restoration | Sprint failure target | bounded fault, teardown, from-empty recovery | Not Run |
| Upgrade/rollback | Sprint upgrade target | source-built Response-only transition | Not Run |
| Automated UAT diagnostics | `.\scripts\uat-sprint-8c.ps1` diagnostic mode | every manual prerequisite and predicate | Not Run |

- Mutable source/environment identity: pending.
- Passing Readiness prerequisite: required.
- Result, dependency fingerprints, executed/inherited lanes, evidence index:
  pending.

## Environment Contract

- Environment fingerprint: secret-free hash of toolchain, databases, Compose,
  ports, topology, fixtures, browser and evidence-root sections.
- Databases: disposable isolated Core, Supervisor, Response, Dataset, Component,
  Dashboard and reference-owner databases; destructive reset is limited to
  authenticated Sprint 8C identities.
- Compose/project: `deploy/sprint-8c/compose.yaml`; project and port values are
  fixed in the attempt start receipt.
- Roles/accounts: administrator, Response owner/respondent, delegate/delegator,
  scoped manager, disjoint outsider and operator from signed fixture read-back.
- Evidence path mode: ignored `artifacts/sprint-8c-closeout/<phase>/<attempt>/`;
  append-only logs and phase-local sealed indexes.

## Preflight

- Status: Not Run
- Required inputs: passing current-source implementation readiness, Readiness
  and Rehearsal certificates; exact dependency fingerprints; resolved defect
  chronology; clean source and environment; reset authorization.
- Bootstrap/no-op/restoration commands: those authenticated during rehearsal,
  not first-run variants.
- Output: `preflight-result.json` and one immutable `candidate.json`.

## SIT

| Lane | Command / evidence | Assertions | Result |
| --- | --- | --- | --- |
| Static/boundaries | Sprint SIT static lane | exact source, format/Clippy, schemas, subtraction and isolation | Not Run |
| Rust workspace | `cargo test --workspace --locked` | complete frozen-candidate Rust inventory | Not Run |
| Playwright | `npm --prefix .\end2end test` | complete frozen-candidate browser manifest | Not Run |
| Deployed acceptance smoke | `.\scripts\smoke.ps1` plus Sprint 8C profile | Response owner, product flow, providers/events, Dataset output, health | Not Run |

Deployed acceptance smoke runs inside SIT. SIT must restore the canonical
topology and publish one passing `sit-result.json` before UAT begins.

## UAT

### Scripted UAT

- Command: `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` plus
  `.\scripts\uat-sprint-8c.ps1` exact Sprint inventory.
- Result: Not Run
- Evidence: pending sealed UAT phase index.

### Manual UAT

| Scenario | Role / start state | Actions and observable pass condition | Result |
| --- | --- | --- | --- |
| UAT-8C-01 product and UI parity | owner/respondent; active assigned draft | list, start, save, refresh/resume, submit and review in direct/lifecycle routes; accepted shell/theme/responsive/a11y/SSR/hydration behavior and clean console | Not Run |
| UAT-8C-02 assignment and providers | respondent/delegate; active/inactive/completed/substituted contexts | only exact active assignment starts; FormVersion renders from typed snapshot; direct legacy start and invalid contexts create nothing/nondisclose | Not Run |
| UAT-8C-03 scoped review | owner, delegate, scoped manager, disjoint outsider | prove own/delegated/scoped visibility and indistinguishable known/random forbidden responses | Not Run |
| UAT-8C-04 Dataset and Workflow consumers | submitted fixture, Workflow and Dataset healthy | submit once, Workflow advances once, Dataset refresh imports exact output, Component/Dashboard display it; duplicate/reordered event and unchanged export are no-ops | Not Run |
| UAT-8C-05 replay, compatibility and outage | administrator/operator | replay identical mutation; reject changed actor/body/action; fault Forms/Workflow/Response/Dataset binding separately; observe bounded sanitized last-good behavior and recovery | Not Run |
| UAT-8C-06 configuration and diagnostics | administrator/operator | administrator validates/applies Response config and inspects sanitized owner diagnostics in generic Module Management; operator verifies the healthy Response status projection in Operations while installation-global Module Management remains correctly restricted | Not Run |
| UAT-8C-07 fresh materialization/no-op | operator, empty authenticated databases | apply owner order, inspect signed read-back/one enrollment/exact health, reapply unchanged and observe semantic no-op | Not Run |
| UAT-8C-08 Core subtraction/isolation | operator | prove three Core transitions (Forms, Workflows, Migration), one Response enrollment/navigation item, no Core Response schema/routes/adapters/seed, and pairwise credential denial | Not Run |
| UAT-8C-09 failure recovery | operator, empty topology with one-shot fault | inspect retained failed apply, exact partial teardown, clean successor apply and canonical health without manual repair | Not Run |
| UAT-8C-10 upgrade/rollback | operator, healthy Response `0.9.0` fixture | upgrade to `1.0.0`, roll back, restore; preserve state/instance and unrelated owners/images/restarts/data/navigation | Not Run |
| UAT-8C-11 roadmap exit | tester with assigned work | complete and review Response, consume submitted output in Dataset/Component/Dashboard, and inspect proof of no shared database access | Not Run |

- UAT result receipt: pending.
- Final topology restoration: required and pending.

## Post-SIT Defect Convergence (Conditional)

On the first candidate-invalidating UAT failure, retain evidence, mark the
candidate invalid, forbid a passing `uat-result.json`, and finish only safe
independent scenarios as non-authoritative diagnostic harvest. Record blocked
dependencies exactly.

Planned records: `uat-defect-harvest.json`, consolidated `defect-batch.json`,
schema-valid `correction-impact-assessment.json`, focused-repair attempts,
`canonical-restoration.json`, and `final-certification-entry.json`. Identity,
authorization, contracts, migrations, shared fixtures/environment and cross-
module changes default broad unless authenticated dependency fingerprints
prove a narrower cone. Focused repair never authorizes a candidate or closeout;
after convergence, complete final Readiness/Rehearsal, a successor freeze,
complete SIT and complete UAT are mandatory.

## Failure And Invalidation Chronology

| Time | Phase/lane/stage | Assertions started | Candidate | Provenance | Origin | Exit gap/process drift | Focused proof | Invalidation | Replacement |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| None | Kickoff only | No | None | None | N/A | None | N/A | N/A | N/A |

For every future failure, a sibling `defect-provenance.json` must validate
before correction or broad rerun. It records source/fixture/environment/
inventory identity, `assertions_started`, product/process/mixed origin,
implementation-exit gap, process drift, expectation-change authority, focused
reproducer, routing and invalidation scope. Automation may classify, invalidate,
route and block reruns; it may not edit tests, change expected values, or decide
that an assertion is obsolete. Every expectation change requires explicit
authority, equal-or-stronger coverage and a test-change-log entry.

## Evidence Integrity And Closeout Authorization

- Compact phase certificates: pending.
- Phase-local indexes and hashes: pending.
- Raw evidence: retained cold under ignored evidence root.
- Routine downstream review: certificates/index hashes only.
- Final full-integrity audit and `evidence-chain.json`: pending.
- Closeout status: Not Authorized.
- Authorization requires one exact candidate, complete passing SIT and UAT,
  complete acceptance mapping, resolved invalidations/provenance, restored
  intended topology/health, and separate evidence-source/documentation commits.

# Sprint 8B Validation Record

- Status: Complete / Closeout Authorized
- Validation policy: `tessara-validation-v2`
- Tracked validation contract: `docs/sprints/sprint-8b-validation-contract.json`
- Implementation profile: `phase8-module-extraction`
- Profile playbook: `docs/architecture/module-extraction-playbook.md`
- Module Definition / transition identity: `tessara.datasets` / `tessara.datasets`
- Canonical product requirements: exactly `ac-01` through `ac-26`
- Internal gate requirement: `gate-implementation-exit`
- Implementation targets: exactly 24
- Formal lanes: exactly 31 — Readiness 3, Candidate Rehearsal 11,
  Preflight 1, SIT 4, and UAT 12
- Manual UAT scenarios: exactly 11 — `UAT-8B-01` through `UAT-8B-11`
- Activation boundary: first sprint after Sprint 8A; no Sprint 8A or legacy
  receipt is retrofitted, reinterpreted, or reused.
- Evidence root: `artifacts/sprint-8b-closeout/` (ignored; successful raw
  evidence retained cold)

The kickoff inventory below is preserved as the approved contract. Final
execution completed against clean implementation commit
`83e8b123de315ed2c7186ec4f0586f5a03198131`, tree
`ab0aa4449595045ffa02e4e8d435a7a9c7239880`, and candidate fingerprint
`93c5b936807407b5a2007ccb1851d22a1d9a90bfc0cd84619637bbf080ac97f9`.
All 24 implementation targets and all 31 formal lanes passed; the 11 manual
UAT scenarios were accepted by the user. Conditional convergence artifacts
remain historical inputs rather than final-candidate test results.

## Final Authoritative Outcome

- Implementation Readiness: 24/24 targets passed.
- Validation Readiness: 3/3 lanes passed.
- Candidate Rehearsal: 11/11 lanes passed.
- Preflight: 1/1 lane passed and froze the exact candidate above.
- SIT: 4/4 lanes passed, including the locked/offline/all-features/jobs-1
  workspace test command, the complete browser inventory, and deployed smoke.
- UAT: 12/12 lanes and 11/11 manual scenarios passed; open defect count is 0.
- Defect provenance: 23 retained records, 0 unresolved.
- Final integrity audit: 6 sealed phase indexes and 840 retained artifacts
  authenticated.
- Evidence chain: `06b3fe97bcb6d1a702a0076f9f6fff33aa2541b747eeb5f86c3374e8434981d2`.
- Closeout authorization: `513301d0a32f127d1c169898391ad99b697bb0ea1b2376644c9df9bfb8a8e2c3`.
- Reviewer topology: `tessara-s8b-uat-manual` at
  `http://127.0.0.1:49452`, retained healthy and source-exact.

## Canonical Evidence And Environment Identities

The plan, this record, the tracked JSON contract, the test-change log, the
browser acceptance manifest, and the UAT scenario contract must be set-equal.
Copied totals are informative only; the literal IDs and file/test inventories
are authoritative.

- `reference` = `deploy/sprint-8b/fixtures/reference-fixture-contract.json`
- `faults` = `deploy/sprint-8b/fixtures/provider-fault-contract.json`
- `upgrade` = `deploy/sprint-8b/fixtures/upgrade-fixture-contract.json`
- `ui` = `docs/audits/sprint-8b-dataset-ui-baseline/baseline-index.json`
- `acceptance` = `end2end/acceptance-manifest.json` plus
  `docs/sprints/sprint-8b-test-change-log.md`
- `uat-contract` = `docs/sprints/sprint-8b-uat/scenario-contract.json`
- `offline` = no stack launch and no live product state
- `isolated-live` = lane-owned Compose project `tessara-s8b-<lane>` with
  pairwise-distinct lane databases, owner-API fixture preparation, and a
  canonical-restoration receipt
- `frozen-sit` = the one candidate-bound `tessara-s8b-sit` topology
- `target:<id>` =
  `artifacts/sprint-8b-closeout/implementation/targets/<id>/result.json`, with
  sibling `command.log` and evidence-reference list
- `lane:<id>` =
  `artifacts/sprint-8b-closeout/<phase>/lanes/<id>/result.json`, with sibling
  `command.log` and evidence-reference list
- `uat:<id>` =
  `artifacts/sprint-8b-closeout/uat/scenarios/<id>/result.json`

The exact `<phase>` directory substitutions are `validation-readiness`,
`candidate-rehearsal`, `validation-preflight`, `sit`, and `uat`, matching each
lane's machine-contract `phase` value. Therefore every selector's result path
is mechanically fixed by its row; runners may not redirect evidence to an
ad-hoc or shared directory.

Every phase runner also emits its compact certificate and a sealed
`evidence-index.json` plus SHA-256 sidecar under that phase root. Live evidence
must bind the exact fixture-preparation receipt, environment fingerprint,
restoration receipt, source identity, and contract hash.

## Acceptance Inventory

### Observable product criteria and manual proof

| Requirement | Observable pass condition and required negative proof | Manual proof |
| --- | --- | --- |
| `ac-01` | `tessara.datasets` is enrolled once as a real Release/Instance and independently owns API, UI, database, migrations, configuration, diagnostics, routes, assets, and exact health; no Core or Supervisor definition-specific fallback exists. | UAT-8B-02/03/06 |
| `ac-02` | Directory, authoring, revision, preview, full QuerySpec pipeline, materialization, directory/tags/provider catalogs, bootstrap-validation batch, lifecycle, responsive/accessibility, and clean-console behavior retain parity; no invented standalone template surface appears. | UAT-8B-01/04/11 |
| `ac-03` | Direct documents and lifecycle navigation match the accepted baseline and use SDK-owned document/CSS primitives; raw module DOM, copied SDK styles, Core Dataset markup, and unnamespaced product selectors fail conformance. | UAT-8B-01/09 |
| `ac-04` | Submitted Response data enters Dataset only through exact Response provider actions; unavailable, unauthorized, malformed, stale, substituted, or incompatible exports publish no partial Dataset state. | UAT-8B-04/07 |
| `ac-05` | FormVersion catalog/schema, scope facts, and actor-safe principal labels arrive through exact typed providers with content revisions/digests and nondisclosure; provider/browser substitution and partial writes are rejected. | UAT-8B-04/09 |
| `ac-06` | Component `1.1.0` consumes Module Instance-owned Dataset v2 across the real process boundary; v1, Core-owned transition references, and legacy payloads fail before lookup. | UAT-8B-05/06/11 |
| `ac-07` | Shared authorized scope produces the expected Component/Dashboard result; disjoint, restricted, known-unauthorized, and random cases disclose neither Dataset metadata nor values. | UAT-8B-04/05/11 |
| `ac-08` | Independently faulted Response, Form, scope, principal, and Dataset actions yield bounded sanitized findings, no partial writes, exact last-good behavior, coherent downstream degradation, and healthy convergence. | UAT-8B-04/05/07/09/10 |
| `ac-09` | Identical authenticated retries and concurrent attempts are logically idempotent; stale promotion loses safely; no row, revision, cursor, receipt, resource, inventory entry, or navigation entry is duplicated. | UAT-8B-04/07/11 |
| `ac-10` | Empty databases rebuild in owner order from typed owner read-back; scripts neither receive another owner's credentials nor write another owner's product tables. | UAT-8B-02/06 |
| `ac-11` | An unchanged second apply changes no product state, topology, configuration, owner receipt, resource identity, cursor/vector, or materialization generation. | UAT-8B-02/04 |
| `ac-12` | A deterministic failed apply is retained, exact partial topology is removed, and a successor from-empty apply restores canonical health without manual repair. | UAT-8B-04/07 |
| `ac-13` | Core has no active Dataset schema, routes, DTOs, handlers, provider adapter, product web composition, seed, static capability/grant, transition descriptor, reverse SQL, or resource SQL. | UAT-8B-06/10/11 |
| `ac-14` | Dataset has no Core, Response, Form, Component, or Dashboard database credential/access, and providers/consumers have no Dataset database credential/access; pairwise denial is executable. | UAT-8B-04/06/09 |
| `ac-15` | Core transitions are exactly Forms, Workflows, Responses, and Migration; Datasets, Components, and Dashboard appear once through enrolled manifests; Dataset navigation has the exact declared ID, key, label, route, destination, order `6`, and capability guards. | UAT-8B-02/03/06 |
| `ac-16` | Dataset configuration validates/applies through the generic template; exact liveness/readiness and sanitized dependency diagnostics preserve unavailable versus degraded semantics and disclose no secret/product row. | UAT-8B-03/10 |
| `ac-17` | Real Dataset `0.9.0 -> 1.0.0 -> 0.9.0 -> 1.0.0` transitions preserve Dataset state and provider route while unrelated images, containers, restarts, data, navigation, and availability remain unchanged. | UAT-8B-08 |
| `ac-18` | A tester authors, materializes, and previews a Dataset from provider contracts, executes a Component over it, and sees the Dashboard result across independently deployed processes/databases. | UAT-8B-05 |
| `ac-19` | A Response-owned transactional monotonic cursor drives unchanged-head no-op, bounded ordered upsert/tombstone pages, interruption/retry, authorized empty pages, concurrent mutation, epoch/scope isolation, and authenticated expired-cursor full rebase; timestamps never define correctness. | UAT-8B-04/07/11 |
| `ac-20` | Browser traffic uses signed ShellContext and Dataset routes only; server-side typed providers preserve current Form/schema/node/principal options and intentional loading, restricted, outage, hydration, and dirty-input behavior. | UAT-8B-01/09 |
| `ac-21` | Dataset, DatasetRevision, and DatasetMajorLine resolve/observe through Dataset with exact owner revision, lifecycle, and availability; v1, wrong owner/instance, known restricted, and random inputs fail/nondisclose without Core product SQL. | UAT-8B-11 |
| `ac-22` | Form Dataset Sources, Core Operations Dataset readiness, and app-summary counts consume exact Dataset actions; authorized empty, unavailable, and undisclosed remain distinct and outage never becomes false zero. | UAT-8B-10 |
| `ac-23` | Manifest, router, gateway, clients, tests, and acceptance inventory expose exactly the declared public/private route/action sets; every persistent mutation has durable raw-body-bound replay and private grants/nonces are one-use. | UAT-8B-01/09/11 |
| `ac-24` | Create, publish, bootstrap, and explicit manager refresh check the complete dependency vector, topologically rebuild the affected Dataset closure, and atomically promote all or none; reads never refresh and keep last-good data with exact freshness. | UAT-8B-04/11 |
| `ac-25` | Dataset stores imported Response aggregates/values and compiles arbitrary QuerySpec results only from owner-local imported/materialized tables; no runtime SQL, credential, or compiled dependency reaches Core product/analytics tables. | UAT-8B-04/06 |
| `ac-26` | Core reporting analytics may remain Core-owned, but Dataset never calls or depends on it; `datasets:manage` refresh and `analytics:refresh` have separate authority, state, diagnostics, and failure domains. | UAT-8B-03/06/10 |

### Exact requirement-to-proof mapping

The target and formal-lane sets below are literal copies of the final tracked
contract. Each row requires every listed `target:*` and `lane:*` receipt, not
one representative receipt.

| Requirement | Exact implementation target IDs | Exact formal lane IDs |
| --- | --- | --- |
| `ac-01` | `owner-product`, `ui-sdk-conformance`, `inventory-navigation`, `migration-seed`, `deployed-smoke` | `readiness-contract`, `readiness-materialization`, `rehearsal-materialization`, `rehearsal-browser`, `rehearsal-conformance`, `preflight-freeze`, `sit-static`, `sit-browser`, `sit-smoke`, `uat-product`, `uat-operations`, `uat-subtraction` |
| `ac-02` | `owner-product`, `ui-sdk-conformance`, `fixture-acceptance`, `api-idempotency`, `dataset-refresh-dag` | `readiness-acceptance`, `rehearsal-rust`, `rehearsal-browser`, `rehearsal-source-sync`, `preflight-freeze`, `sit-rust`, `sit-browser`, `uat-product`, `uat-replay-refresh` |
| `ac-03` | `ui-sdk-conformance`, `fixture-acceptance`, `ui-provider-boundaries` | `readiness-contract`, `readiness-acceptance`, `rehearsal-browser`, `rehearsal-conformance`, `preflight-freeze`, `sit-static`, `sit-browser`, `uat-product` |
| `ac-04` | `contract-boundary`, `failure-recovery`, `response-export-contract`, `response-incremental-sync` | `readiness-contract`, `readiness-materialization`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`, `uat-recovery` |
| `ac-05` | `contract-boundary`, `owner-product`, `ui-provider-boundaries` | `readiness-contract`, `rehearsal-conformance`, `rehearsal-source-sync`, `rehearsal-smoke`, `preflight-freeze`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-product`, `uat-providers` |
| `ac-06` | `contract-boundary`, `consumer-cutover`, `core-subtraction`, `resource-resolution` | `readiness-contract`, `rehearsal-static`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-resource-resolution`, `uat-crossmodule`, `uat-subtraction` |
| `ac-07` | `contract-boundary`, `consumer-cutover`, `deployed-smoke`, `resource-resolution` | `rehearsal-conformance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-providers`, `uat-resource-resolution`, `uat-crossmodule` |
| `ac-08` | `failure-recovery`, `deployed-smoke`, `response-incremental-sync`, `ui-provider-boundaries`, `reverse-consumers` | `readiness-materialization`, `readiness-acceptance`, `rehearsal-source-sync`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `rehearsal-recovery`, `preflight-freeze`, `sit-browser`, `sit-smoke`, `uat-providers`, `uat-reverse-consumers`, `uat-crossmodule`, `uat-recovery` |
| `ac-09` | `failure-recovery`, `response-incremental-sync`, `api-idempotency`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-rust`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`, `uat-recovery` |
| `ac-10` | `core-subtraction`, `migration-seed`, `clean-materialization`, `fixture-acceptance` | `readiness-materialization`, `readiness-acceptance`, `rehearsal-materialization`, `preflight-freeze`, `sit-static`, `sit-smoke`, `uat-materialization`, `uat-subtraction` |
| `ac-11` | `clean-materialization`, `semantic-noop`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-materialization`, `rehearsal-source-sync`, `preflight-freeze`, `sit-smoke`, `uat-materialization`, `uat-replay-refresh` |
| `ac-12` | `clean-materialization`, `failure-recovery`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-smoke`, `uat-providers`, `uat-recovery` |
| `ac-13` | `contract-boundary`, `core-subtraction`, `inventory-navigation`, `reverse-consumers`, `resource-resolution` | `readiness-contract`, `rehearsal-static`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-reverse-consumers`, `uat-resource-resolution`, `uat-subtraction` |
| `ac-14` | `contract-boundary`, `consumer-cutover`, `core-subtraction`, `migration-seed`, `clean-materialization`, `ui-provider-boundaries` | `readiness-contract`, `readiness-materialization`, `rehearsal-static`, `rehearsal-materialization`, `rehearsal-source-sync`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-smoke`, `uat-providers`, `uat-subtraction` |
| `ac-15` | `core-subtraction`, `inventory-navigation`, `migration-seed`, `deployed-smoke` | `readiness-contract`, `readiness-materialization`, `rehearsal-static`, `rehearsal-materialization`, `rehearsal-smoke`, `preflight-freeze`, `sit-static`, `sit-smoke`, `uat-materialization`, `uat-operations`, `uat-subtraction` |
| `ac-16` | `owner-product`, `deployed-smoke`, `reverse-consumers` | `readiness-acceptance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-browser`, `sit-smoke`, `uat-operations`, `uat-reverse-consumers` |
| `ac-17` | `inventory-navigation`, `deployed-smoke`, `independent-upgrade-rollback` | `readiness-materialization`, `rehearsal-smoke`, `rehearsal-upgrade`, `preflight-freeze`, `sit-smoke`, `uat-upgrade` |
| `ac-18` | `consumer-cutover`, `clean-materialization`, `deployed-smoke` | `readiness-materialization`, `readiness-acceptance`, `rehearsal-materialization`, `rehearsal-browser`, `rehearsal-smoke`, `preflight-freeze`, `sit-browser`, `sit-smoke`, `uat-crossmodule` |
| `ac-19` | `clean-materialization`, `semantic-noop`, `failure-recovery`, `fixture-acceptance`, `deployed-smoke`, `response-export-contract`, `response-incremental-sync`, `api-idempotency`, `dataset-refresh-dag` | `readiness-contract`, `readiness-materialization`, `rehearsal-materialization`, `rehearsal-source-sync`, `rehearsal-smoke`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`, `uat-recovery` |
| `ac-20` | `contract-boundary`, `owner-product`, `ui-sdk-conformance`, `ui-provider-boundaries` | `readiness-contract`, `readiness-acceptance`, `rehearsal-browser`, `rehearsal-conformance`, `rehearsal-source-sync`, `preflight-freeze`, `sit-rust`, `sit-browser`, `uat-product`, `uat-providers` |
| `ac-21` | `contract-boundary`, `core-subtraction`, `resource-resolution` | `readiness-contract`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-resource-resolution`, `uat-subtraction` |
| `ac-22` | `core-subtraction`, `deployed-smoke`, `reverse-consumers`, `resource-resolution` | `readiness-contract`, `readiness-acceptance`, `rehearsal-reverse-consumers`, `rehearsal-smoke`, `preflight-freeze`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-operations`, `uat-reverse-consumers` |
| `ac-23` | `contract-boundary`, `owner-product`, `fixture-acceptance`, `api-idempotency` | `readiness-contract`, `readiness-acceptance`, `rehearsal-static`, `rehearsal-rust`, `rehearsal-conformance`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-browser`, `uat-scripted`, `uat-product`, `uat-resource-resolution`, `uat-replay-refresh` |
| `ac-24` | `failure-recovery`, `response-incremental-sync`, `api-idempotency`, `dataset-refresh-dag` | `readiness-materialization`, `rehearsal-rust`, `rehearsal-source-sync`, `rehearsal-recovery`, `preflight-freeze`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh` |
| `ac-25` | `contract-boundary`, `owner-product`, `core-subtraction`, `response-incremental-sync` | `readiness-contract`, `readiness-materialization`, `rehearsal-static`, `rehearsal-rust`, `rehearsal-source-sync`, `preflight-freeze`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-subtraction` |
| `ac-26` | `contract-boundary`, `owner-product`, `core-subtraction`, `reverse-consumers` | `readiness-contract`, `rehearsal-static`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `preflight-freeze`, `sit-static`, `sit-rust`, `uat-operations`, `uat-reverse-consumers`, `uat-subtraction` |

`gate-implementation-exit` maps to exactly all 24 implementation target IDs in
the Implementation Exit Gate table, all 31 formal lane IDs in the Formal Lane
Inventory, and all eleven manual scenarios in the Manual UAT Inventory. It is
the internal machine gate, not a twenty-seventh product criterion.

## Critical Modular Process Proofs

| Process | Required implementation behavior | Implementation proof | Formal/deployed/manual proof |
| --- | --- | --- | --- |
| Response mutation and export | One Response-owner transaction mutates the aggregate and appends one immutable complete aggregate upsert or tombstone. A transactionally locked counter reflects commit order; PostgreSQL sequence allocation or `MAX(created_at)` is insufficient. | `response-export-contract`, `contract-boundary`, `fixture-acceptance` | `rehearsal-source-sync`, `sit-rust`, `uat-providers`; UAT-8B-04 |
| Cheap checkpoint/no-op | Dataset authenticates the scope-partitioned opaque head before paging. Equal head means zero page calls, zero staging, zero cursor/vector/resource/receipt mutation, and an exact synchronous no-op response. | `semantic-noop`, `response-incremental-sync`, `api-idempotency` | `rehearsal-source-sync`, `sit-rust`, `sit-smoke`, `uat-replay-refresh`; UAT-8B-04/11 |
| Incremental attempt staging | `start` fixes one inclusive upper cursor and immutable snapshot. Pages scan strictly after the attempt cursor through that bound; filtered empty pages can advance an opaque attempt cursor without disclosing hidden activity. Page/change digests and upserts/tombstones remain attempt-private. | `response-export-contract`, `response-incremental-sync` | `rehearsal-source-sync`, `sit-rust`, `uat-providers`; UAT-8B-04 |
| Atomic promotion | One partition lock/CAS verifies the starting published cursor/generation, applies staged source projection, rebuilds the full affected Dataset closure, advances major/resource revisions, stores one receipt, and advances the published cursor in one transaction. Any source, downstream Dataset, or receipt failure promotes none of it. | `response-incremental-sync`, `dataset-refresh-dag`, `failure-recovery` | `rehearsal-source-sync`, `rehearsal-recovery`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`; UAT-8B-04/11 |
| Crash/retry boundary | Interrupt before page persistence, after page persistence, after hidden checkpoint, before promotion, and after commit-before-response. Retrying from the last published cursor yields the same rows, terminal cursor, closure generation, and single durable receipt as uninterrupted execution. | `response-incremental-sync`, `api-idempotency`, `failure-recovery` | `rehearsal-source-sync`, `rehearsal-recovery`, `sit-rust`, `uat-replay-refresh`, `uat-recovery`; UAT-8B-04/07/11 |
| Full rebase | No cursor, authenticated cursor expiry, provider epoch change, or changed provider/scope/source partition starts a new authenticated immutable full snapshot at a fixed upper cursor. It never resumes/publishes an old checkpoint. Last-good state stays served until atomic replacement; failure/expiry discards private staging. | `clean-materialization`, `response-export-contract`, `response-incremental-sync`, `failure-recovery` | `rehearsal-materialization`, `rehearsal-source-sync`, `rehearsal-recovery`, `sit-smoke`, `uat-providers`, `uat-recovery`; UAT-8B-04/07 |
| Dataset dependency DAG | The canonical `dataset.base -> dataset.derived -> dataset.derived-second-hop` closure rebuilds topologically as one generation. `dataset.independent-binding` stays byte/identity stable; cycles reject before staging; a derived failure rolls back projection, closure, cursor, resource revisions, and receipt; Component/Dashboard never observe mixed generations. | `dataset-refresh-dag`, `owner-product`, `failure-recovery`, `deployed-smoke` | `rehearsal-source-sync`, `rehearsal-recovery`, `sit-rust`, `sit-smoke`, `uat-providers`, `uat-replay-refresh`; UAT-8B-04/11 |
| Browser/provider isolation | Browser calls signed ShellContext and Dataset public/editor-options routes only. Direct `/api/me`, `/api/forms`, `/api/nodes`, `/api/admin/users`, or `/api/form-versions/*` traffic fails the test. Dataset server calls four typed providers, and isolated provider faults preserve unsaved input and intentional loading/restricted/outage states. | `ui-provider-boundaries`, `ui-sdk-conformance`, `contract-boundary` | `rehearsal-browser`, `rehearsal-conformance`, `rehearsal-source-sync`, `sit-rust`, `sit-browser`, `uat-product`, `uat-providers`; UAT-8B-09 |
| Reverse consumers | Form detail uses `datasets.source_usage`; Operations uses `datasets.operations_status`; app summary uses `datasets.summary`. Authorized empty, unavailable, and undisclosed are distinct; unrelated Form/Operations/summary content survives Dataset outage and recovers without Core Dataset SQL. | `reverse-consumers`, `core-subtraction`, `deployed-smoke` | `rehearsal-reverse-consumers`, `rehearsal-smoke`, `sit-rust`, `sit-browser`, `sit-smoke`, `uat-operations`, `uat-reverse-consumers`; UAT-8B-10 |
| Resource resolution/observation | `datasets.resolve` owns Dataset, DatasetRevision, and DatasetMajorLine v2 resolution/observation. Wrong owner/type/version/instance rejects before lookup; known restricted and random inputs collapse; lifecycle/revision/availability remain exact without Core transition SQL. | `resource-resolution`, `contract-boundary`, `core-subtraction` | `rehearsal-conformance`, `rehearsal-reverse-consumers`, `sit-static`, `sit-rust`, `sit-smoke`, `uat-resource-resolution`, `uat-subtraction`; UAT-8B-11 |
| Mutation replay and private one-use calls | Every persistent route binds replay to Module Instance, actor, authorization JTI/action, method/path, exact raw-body digest including empty body, and idempotency key. Identical retry returns the stored response; changed actor/action/body rejects. Gateway forwards/generates the key. Private action/path/body/audience/correlation bindings and nonces are one-use. | `api-idempotency`, `contract-boundary`, `owner-product`, `fixture-acceptance` | `rehearsal-rust`, `rehearsal-conformance`, `sit-rust`, `sit-browser`, `uat-scripted`, `uat-replay-refresh`; UAT-8B-11 |
| Core analytics separation | Dataset imports source envelopes and compiles only owner-local tables. Core analytics schema/refresh/status may continue solely for Core reporting, but Dataset has no analytics table/API/credential dependency and `datasets:manage` cannot impersonate `analytics:refresh` or vice versa. Either failure domain can degrade independently. | `core-subtraction`, `contract-boundary`, `owner-product`, `response-incremental-sync`, `reverse-consumers` | `rehearsal-static`, `rehearsal-rust`, `rehearsal-conformance`, `rehearsal-reverse-consumers`, `sit-static`, `sit-rust`, `uat-operations`, `uat-subtraction`; UAT-8B-03/06/10 |

## Required Evidence Inventory

| Artifact | Producer | Required before | Status |
| --- | --- | --- | --- |
| `implementation/implementation-readiness-result.json` | Implementation | Validation Readiness | Passed |
| 24 `implementation/targets/<id>/result.json` receipts, logs, references | Implementation | Aggregate implementation result | Passed |
| `validation-readiness/validation-readiness-result.json` | Validation coordinator | Candidate Rehearsal | Passed |
| `candidate-rehearsal/candidate-rehearsal-result.json` | Validation coordinator | Preflight | Passed |
| `validation-preflight/preflight-result.json` | Preflight | Candidate freeze | Passed |
| `validation-preflight/candidate.json` | Preflight | SIT | Passed |
| `sit/sit-result.json` | SIT | UAT | Passed |
| `uat/uat-result.json` | UAT | Authorization | Passed |
| 11 `uat/scenarios/UAT-8B-<nn>/result.json` receipts | UAT | UAT certificate | Passed |
| `uat-defect-harvest.json` | UAT/coordinator | Correction batch, if triggered | Planned / Conditional |
| `defect-batch.json` | Coordinator | Impact assessment, if triggered | Planned / Conditional |
| `correction-impact-assessment.json` | Coordinator | Focused repair, if triggered | Planned / Conditional |
| `focused-repair-validation/attempt-<n>.json` | SIT/UAT/coordinator | Convergence, if triggered | Planned / Conditional |
| `canonical-restoration.json` | Coordinator | Convergence/final certification, if triggered | Planned / Conditional |
| `final-certification-entry.json` | Coordinator | Final certification, if triggered | Planned / Conditional |
| Per-phase `evidence-index.json` and SHA-256 sidecar | Each phase | Phase certificate | Passed |
| `evidence-chain.json` and SHA-256 sidecar | Validation coordinator | Closeout authorization | Passed |
| `closeout-authorization.json` | Validation coordinator | Closeout | Passed |

All paths above are relative to `artifacts/sprint-8b-closeout/`. A missing,
unhashed, stale, malformed, unsealed, or source/environment-mismatched artifact
is a failed prerequisite, never an implicit pass.

## Implementation Exit Gate

Every target is required. The runner records exact source/contract identity,
dependency fingerprints, command, result, duration, clean-environment identity
where required, log, and evidence references. The 24 rows and their order are
canonical.

| # | Target | Proof classes | Dependency domains | Exact command | Clean | Result |
| --- | --- | --- | --- | --- | --- | --- |
| 01 | `static-quality` | `static-quality` | product-source, build-dependencies, acceptance-inventory, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target static-quality` | No | Passed |
| 02 | `contract-boundary` | `contract-boundary` | product-source, build-dependencies, migrations-seeds, fixtures, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target contract-boundary` | No | Passed |
| 03 | `owner-product` | `owner-product` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target owner-product` | No | Passed |
| 04 | `ui-sdk-conformance` | `ui-sdk-conformance` | product-source, build-dependencies, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target ui-sdk-conformance` | **Yes** | Passed |
| 05 | `consumer-cutover` | `contract-boundary` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target consumer-cutover` | No | Passed |
| 06 | `core-subtraction` | `core-subtraction` | product-source, build-dependencies, migrations-seeds, fixtures, acceptance-inventory, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target core-subtraction` | No | Passed |
| 07 | `inventory-navigation` | `inventory-navigation` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target inventory-navigation` | No | Passed |
| 08 | `migration-seed` | `migration-seed` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target migration-seed` | **Yes** | Passed |
| 09 | `clean-materialization` | `clean-materialization` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target clean-materialization` | **Yes** | Passed |
| 10 | `semantic-noop` | `semantic-noop` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target semantic-noop` | **Yes** | Passed |
| 11 | `failure-recovery` | `failure-recovery` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target failure-recovery` | **Yes** | Passed |
| 12 | `fixture-acceptance` | `fixture-acceptance` | product-source, build-dependencies, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target fixture-acceptance` | No | Passed |
| 13 | `runner-selftest` | `runner-selftest` | validation-shared, sprint-contract, implementation-runner, implementation-harness, readiness-runner, rehearsal-runner, preflight-runner, sit-runner, uat-runner, evidence-publication | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target runner-selftest` | No | Passed |
| 14 | `deployed-smoke` | `deployed-smoke`, `consumer-cutover` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target deployed-smoke` | No | Passed |
| 15 | `independent-upgrade-rollback` | `independent-upgrade-rollback` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target independent-upgrade-rollback` | No | Passed |
| 16 | `uat-readiness` | `uat-readiness` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness, uat-runner | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target uat-readiness` | No | Passed |
| 17 | `response-export-contract` | `contract-boundary` | product-source, build-dependencies, migrations-seeds, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target response-export-contract` | No | Passed |
| 18 | `response-incremental-sync` | `contract-boundary`, `owner-product` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target response-incremental-sync` | **Yes** | Passed |
| 19 | `ui-provider-boundaries` | `contract-boundary`, `owner-product` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target ui-provider-boundaries` | No | Passed |
| 20 | `reverse-consumers` | `contract-boundary` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target reverse-consumers` | No | Passed |
| 21 | `resource-resolution` | `contract-boundary`, `owner-product` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target resource-resolution` | No | Passed |
| 22 | `api-idempotency` | `contract-boundary`, `owner-product` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target api-idempotency` | No | Passed |
| 23 | `dataset-refresh-dag` | `owner-product` | product-source, build-dependencies, migrations-seeds, deployment-materialization, fixtures, acceptance-inventory, environment-contract, validation-shared, sprint-contract, implementation-runner, implementation-harness | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target dataset-refresh-dag` | **Yes** | Passed |
| 24 | `planning-contract-alignment` | `contract-boundary` | fixtures, acceptance-inventory, validation-shared, sprint-contract, implementation-runner, implementation-harness, readiness-runner, rehearsal-runner, preflight-runner, sit-runner, uat-runner, evidence-publication, documentation | `.\scripts\run-sprint-8b-implementation-readiness.ps1 -Target planning-contract-alignment` | No | Passed |

Formal Readiness is forbidden until the current clean source passes all 24
targets with zero known failures and a result bound to the current validation-
contract SHA-256. Formal validation is certification, not the place to discover
an omitted extraction surface.

The repository-owned `-Finalize` mode additionally requires the exact source
commit and tree to be clean and every target receipt to bind that same identity.
It therefore cannot publish during an uncommitted implementation session and
does not imply authority to create a commit; after an authorized commit, all 24
targets must be rerun from that source before finalization.

### Locked baseline commands

These exact commands run inside their declared targets/formal lanes. A narrower
or online Rust invocation does not satisfy the contract.

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-features --locked --offline --jobs 1`
- `pwsh -NoProfile -File scripts/check-web-crate-boundaries.ps1`
- `pwsh -NoProfile -File scripts/verify-module-sdk-boundaries.ps1`
- `pwsh -NoProfile -File scripts/ui-sdk-conformance.ps1`
- `pwsh -NoProfile -File scripts/build-module-ui-browser-assets.ps1 -Module all -Check`
- `npm --prefix .\end2end test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

`rehearsal-rust` and `sit-rust` must both retain the exact locked/offline/all-
features/jobs-1 test command above. The browser manifest runs with one worker,
zero retries, no `only`/skip/fixme/filter/max-failure truncation, and no
snapshot-update mode.

- Clean source: `83e8b123de315ed2c7186ec4f0586f5a03198131` /
  `ab0aa4449595045ffa02e4e8d435a7a9c7239880`; validation-contract SHA-256:
  `a05585ab12e2ff151985bbe629e3a5d074cd44dca81b9ecc62799092b1c9482e`
- First apply from authenticated empty databases: Passed
- Semantic no-op: Passed
- Failure containment and from-empty recovery: Passed
- Fixture, runner, smoke, visual, and UAT-predicate reproducers: Passed
- Known failure count: 0
- Aggregate implementation-readiness result: Produced

## Formal Lane Inventory

The sequence below is exact. Prerequisites form one explicit chain, so stateful
siblings never mutate one topology concurrently. Each selector emits the
corresponding `lane:<id>` receipt/log/reference set.

| # | Phase / lane | Exact prerequisite | Exact selector | Environment | Result |
| --- | --- | --- | --- | --- | --- |
| 01 | Readiness `readiness-contract` | none | `pwsh -NoProfile -File .\scripts\validate-sprint-8b-readiness.ps1 -Lane readiness-contract` | `offline` | Passed |
| 02 | Readiness `readiness-materialization` | `readiness-contract` | `pwsh -NoProfile -File .\scripts\validate-sprint-8b-readiness.ps1 -Lane readiness-materialization` | `tessara-s8b-readiness-materialization`; isolated live/restored | Passed |
| 03 | Readiness `readiness-acceptance` | `readiness-materialization` | `pwsh -NoProfile -File .\scripts\validate-sprint-8b-readiness.ps1 -Lane readiness-acceptance` | `offline`; consumes authenticated materialization/restoration receipt | Passed |
| 04 | Rehearsal `rehearsal-static` | `readiness-acceptance` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-static` | `offline` | Passed |
| 05 | Rehearsal `rehearsal-rust` | `rehearsal-static` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-rust` | `tessara-s8b-rehearsal-rust`; isolated live/restored | Passed |
| 06 | Rehearsal `rehearsal-materialization` | `rehearsal-rust` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-materialization` | `tessara-s8b-rehearsal-materialization`; isolated live/restored | Passed |
| 07 | Rehearsal `rehearsal-browser` | `rehearsal-materialization` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-browser` | `tessara-s8b-rehearsal-browser`; isolated live/restored | Passed |
| 08 | Rehearsal `rehearsal-conformance` | `rehearsal-browser` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-conformance` | `tessara-s8b-rehearsal-conformance`; isolated live/restored | Passed |
| 09 | Rehearsal `rehearsal-source-sync` | `rehearsal-conformance` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-source-sync` | `tessara-s8b-rehearsal-source-sync`; isolated live/restored | Passed |
| 10 | Rehearsal `rehearsal-reverse-consumers` | `rehearsal-source-sync` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-reverse-consumers` | `tessara-s8b-rehearsal-reverse-consumers`; isolated live/restored | Passed |
| 11 | Rehearsal `rehearsal-smoke` | `rehearsal-reverse-consumers` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-smoke` | `tessara-s8b-rehearsal-smoke`; isolated live/restored | Passed |
| 12 | Rehearsal `rehearsal-recovery` | `rehearsal-smoke` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-recovery` | `tessara-s8b-rehearsal-recovery`; isolated live/restored | Passed |
| 13 | Rehearsal `rehearsal-upgrade` | `rehearsal-recovery` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-upgrade` | `tessara-s8b-rehearsal-upgrade`; isolated live/restored | Passed |
| 14 | Rehearsal `rehearsal-uat` | `rehearsal-upgrade` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-candidate-rehearsal.ps1 -Lane rehearsal-uat` | `tessara-s8b-rehearsal-uat`; isolated live/restored | Passed |
| 15 | Preflight `preflight-freeze` | `rehearsal-uat` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-validation-preflight.ps1` | `offline`; identity/evidence audit, emits `candidate.json` | Passed |
| 16 | SIT `sit-static` | `preflight-freeze` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-sit.ps1 -Lane sit-static` | `offline`; bound to frozen `tessara-s8b-sit` identity | Passed |
| 17 | SIT `sit-rust` | `sit-static` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-sit.ps1 -Lane sit-rust` | frozen `tessara-s8b-sit`; restored/checkpointed | Passed |
| 18 | SIT `sit-browser` | `sit-rust` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-sit.ps1 -Lane sit-browser` | same frozen `tessara-s8b-sit`; restored/checkpointed | Passed |
| 19 | SIT `sit-smoke` | `sit-browser` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-sit.ps1 -Lane sit-smoke` | same frozen `tessara-s8b-sit`; canonical restoration | Passed |
| 20 | UAT `uat-scripted` | `sit-smoke` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-scripted` | `tessara-s8b-uat-scripted`; isolated live/restored | Passed |
| 21 | UAT `uat-product` | `uat-scripted` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-product` | `tessara-s8b-uat-product`; isolated live/restored | Passed |
| 22 | UAT `uat-materialization` | `uat-product` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-materialization` | `tessara-s8b-uat-materialization`; isolated live/restored | Passed |
| 23 | UAT `uat-operations` | `uat-materialization` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-operations` | `tessara-s8b-uat-operations`; isolated live/restored | Passed |
| 24 | UAT `uat-providers` | `uat-operations` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-providers` | `tessara-s8b-uat-providers`; isolated live/restored | Passed |
| 25 | UAT `uat-reverse-consumers` | `uat-providers` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-reverse-consumers` | `tessara-s8b-uat-reverse-consumers`; isolated live/restored | Passed |
| 26 | UAT `uat-resource-resolution` | `uat-reverse-consumers` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-resource-resolution` | `tessara-s8b-uat-resource-resolution`; isolated live/restored | Passed |
| 27 | UAT `uat-replay-refresh` | `uat-resource-resolution` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-replay-refresh` | `tessara-s8b-uat-replay-refresh`; isolated live/restored | Passed |
| 28 | UAT `uat-crossmodule` | `uat-replay-refresh` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-crossmodule` | `tessara-s8b-uat-crossmodule`; isolated live/restored | Passed |
| 29 | UAT `uat-subtraction` | `uat-crossmodule` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-subtraction` | `tessara-s8b-uat-subtraction`; isolated live/restored | Passed |
| 30 | UAT `uat-recovery` | `uat-subtraction` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-recovery` | `tessara-s8b-uat-recovery`; isolated live/restored | Passed |
| 31 | UAT `uat-upgrade` | `uat-recovery` | `pwsh -NoProfile -File .\scripts\run-sprint-8b-formal-uat.ps1 -Lane uat-upgrade` | `tessara-s8b-uat-upgrade`; isolated live/restored | Passed |

Candidate Rehearsal is final pre-freeze certification of the mutable candidate
composition. It is not an implementation loop, debugging lane, diagnostic
wave, or substitute for candidate-bound SIT/UAT. A passing Rehearsal
certificate requires every declared lane newly passed or validly inherited by
authenticated non-impact, no failed/blocked/open item, successful restoration,
and a sealed phase-local evidence index.

### Formal receipt chain

1. Implementation publishes a passing non-authoritative
   `implementation-readiness-result.json` for clean source and the exact
   contract hash.
2. Validation Readiness executes or inherits only authenticated unaffected v2
   lanes according to dependency fingerprints; uncertainty executes the full
   affected phase.
3. Candidate Rehearsal certifies the mutable composition and publishes its
   compact certificate and sealed evidence index.
4. Preflight authenticates both certificates and freezes the exact candidate.
5. Complete candidate-bound SIT runs static/boundary, the exact locked/offline
   Rust command, Playwright, and deployed acceptance smoke. Deployed smoke is
   part of SIT.
6. Complete UAT runs `uat-scripted`, the remaining eleven thematic lanes, and
   all eleven manual scenarios.
7. The coordinator authenticates compact certificates and
   `evidence-chain.json`; closeout performs one final complete integrity audit
   over all sealed phase indexes.

## Environment And Fixture Contract

- Compose/profile: `tessara-sprint-8b` /
  `deploy/sprint-8b/compose.yaml`.
- Processes: Core, Supervisor, gateway, Dataset, Component, and Dashboard are
  independently addressable; source providers remain in their current owner
  process.
- Databases: pairwise-distinct disposable Core, Dataset, Component, Dashboard,
  and validation databases. Every destructive reset requires explicit named-
  target authorization.
- Provider isolation: four profile-local proxies independently fault Response,
  Form, scope, and principal actions without stopping Core auth/control-plane
  and without product-database access.
- Gateway: stopped until owner migrations/bootstrap/read-back and exact health
  complete; then started for acceptance.
- Ports, image digests, source labels, exact health JSON/content types, release
  identities, topology, toolchain, and database bindings are read from the
  tracked profile and fingerprinted; a redirect or broad HTTP 2xx is not health
  evidence.
- Accounts: administrator, Dataset manager, full-scope operator,
  restricted-tier operator, confidential-tier operator, disjoint-scope
  operator, Operations actor, and service principals derived from the applied
  Blueprint/Manifest.
- Identity rule: physical IDs come only from signed owner receipts and typed
  read-back keyed by logical fixture name; no fixture predicts UUIDs or treats
  copied counts as identity.

Required logical fixtures include:

- actors `actor.admin`, `actor.dataset-manager`, `actor.operations`,
  `actor.full`, `actor.restricted`, `actor.confidential`, and `actor.disjoint`;
- Forms/FormVersions `form.primary/v1`, `form.secondary/v1`, and
  `form.disjoint/v1` with exact field/type/option/layout digests;
- Responses `response.initial`, `.same-time-a`, `.same-time-b`, `.new`,
  `.corrected`, `.status-out`, `.status-in`, `.redacted`, `.deleted`, and
  `.outside-scope`, created/mutated only through the Response owner;
- two independent Response cursor partitions, including authorized empty-page
  advancement;
- Datasets `dataset.base`, `.derived`, `.derived-second-hop`,
  `.independent-binding`, `.disjoint-binding`, `.incompatible`, and
  `.cycle-candidate`;
- Component Table/Chart/Stat resources in immutable Component `1.1.0`,
  four Dashboard `3.0.2` placements including the disjoint Component redacted
  for the minimum-capability `actor.full`, reverse-consumer states, and all
  three Dataset resource-observation types; and
- real Dataset `0.9.0` and `1.0.0` release/image/provenance fixtures.

- Environment fingerprints: authenticated per phase certificate and lane
  receipt
- Tool versions: retained in the sealed SIT and implementation evidence
- Reset authorization: exact lane-scoped disposable reset authorization passed
- Fixture preparation receipts: Produced
- Canonical restoration receipts: Produced

## Acceptance Manifest And Test-Change Discipline

`end2end/acceptance-manifest.json` schema v2 owns the literal browser test set;
`docs/sprints/sprint-8b-test-change-log.md` explains every deletion, rename,
expectation, fixture, selector, or snapshot change with its old/new exact
identity, governing `ac-xx`, behavioral reason, equal-or-stronger replacement,
and invalidated target/lane.

The following ten current Dataset identities must remain discoverable unless a
logged equal-or-stronger replacement is accepted:

- `admin can author, edit, save, and view a Sprint 3A dataset`
- `admin can UAT Sprint 3B advanced dataset authoring`
- `dataset SQL preview uses pre-projection join keys and stable field identities`
- `dataset revision navigation handles repeated detail and error states`
- `dataset SQL preview renders ordered QuerySpec operations as sequential CTEs`
- `dataset operations keep operation-local state through reorder, save, and reload`
- `dataset SQL preview merges unioned source fields under the union step alias`
- `dataset source picker keeps Version N major-line fields after a newer major exists`
- `admin can review and publish a dataset draft revision`
- `frozen Dataset document routes preserve direct-load and refresh ownership`

Sprint 8B adds these exact identities:

- `Sprint 8B independent Dataset module › editor options use only Dataset-owned browser routes`
- `Sprint 8B independent Dataset module › synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure`
- `Sprint 8B independent Dataset module › reverse consumers distinguish authorized empty unavailable and undisclosed states`
- `Sprint 8B independent Dataset module › mutation replay and static route precedence remain exact`
- `canonical module UI visual baselines › Datasets directory at 1440 px (light)`
- `canonical module UI visual baselines › Datasets editor at 390 px (light)`
- `canonical module UI visual baselines › Datasets directory at 1440 px (dark)`
- `canonical module UI visual baselines › Datasets editor at 390 px (dark)`
- `canonical module UI visual baselines › Datasets revisions at 1024 px`
- `canonical module UI visual baselines › Datasets preview at 1440 px`
- `canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas`

Existing permissions identities for Operations visibility, scoped-reader draft
hiding, and JavaScript-disabled Response/Dataset ownership also remain literal.
Each touched scenario must pass alone and in the complete manifest; serial
ordering is never fixture setup.

## Candidate Identity

- Implementation commit: `83e8b123de315ed2c7186ec4f0586f5a03198131`
- Tree: `ab0aa4449595045ffa02e4e8d435a7a9c7239880`
- Dirty state at freeze and every candidate-bound phase: false
- Candidate fingerprint:
  `93c5b936807407b5a2007ccb1851d22a1d9a90bfc0cd84619637bbf080ac97f9`
- Acceptance-manifest/test-change-log/UAT-contract identity: Recorded
- Deployment profile/configuration digest: Recorded
- Migration/baseline identity: Recorded
- Expected provenance: exact Core, Supervisor, Dataset, Component, and
  Dashboard source/image/release labels
- Observed image digests: Recorded

## Validation Readiness

- Formal selectors: rows 01–03 in the Formal Lane Inventory
- Required input: passing current implementation-readiness result; clean
  source; exact contract hash; dependency/changed-path impact plan; authenticated
  environment/fixture/reset identities; complete target and acceptance mapping
- Result receipt: Produced
- Newly executed lanes: 3/3
- Authenticated inherited lanes: None; complete execution was used
- Open defects: 0
- Restoration: Passed
- Phase-local evidence index: Produced

## Candidate Rehearsal

- Formal selectors: rows 04–14 in the Formal Lane Inventory
- Purpose: final pre-freeze certification of static boundaries, exact Rust,
  source-exact materialization, browser parity, conformance/nondisclosure,
  source synchronization/DAG closure, reverse consumers, deployed smoke,
  recovery, upgrade/rollback, and UAT readiness
- Mutable source/environment identity: commit
  `83e8b123de315ed2c7186ec4f0586f5a03198131`, tree
  `ab0aa4449595045ffa02e4e8d435a7a9c7239880`
- Passing Readiness prerequisite: Passed prerequisite
- Impact plan: Produced
- Result receipt: Produced
- Newly executed lanes: 11/11
- Authenticated inherited lanes: None; complete execution was used
- Open defects: 0
- Restoration: Passed
- Phase-local evidence index: Produced

## Preflight

- Exact command: `pwsh -NoProfile -File .\scripts\run-sprint-8b-validation-preflight.ps1`
- Status: Passed
- Required prerequisite: passing authenticated Readiness and Rehearsal compact
  certificates for the current source/environment with complete lane coverage,
  sealed indexes, zero open defects, and satisfied restoration
- Required audit: environment/reset authority, changed-path impact,
  contract/inventory alignment, bootstrap/no-op/recovery commands, provenance,
  baseline, evidence paths, hashes, and candidate fingerprint inputs
- `preflight-result.json`: Produced
- `candidate.json`: Produced

## SIT

- Formal selectors: rows 16–19 in the Formal Lane Inventory
- Topology: one exact frozen candidate on `tessara-s8b-sit`
- Order: `sit-static` -> `sit-rust` -> `sit-browser` -> `sit-smoke`
- Rust acceptance: `cargo test --workspace --all-features --locked --offline --jobs 1`
- Browser acceptance: full manifest, one worker, zero retries, no filtered or
  skipped inventory
- Deployed acceptance smoke: exact owner/provider/consumer route, provenance,
  health, outage, recovery, Dataset DAG generation, and Core subtraction; it
  belongs inside SIT
- SIT result receipt: Produced
- Canonical topology restoration: Passed
- Phase-local evidence index: Produced

## Manual UAT Inventory

Every manual scenario is candidate-bound, runs only after passing SIT, and has
its own `uat:<id>` receipt. `uat-scripted` establishes executable preconditions;
the thematic UAT lanes retain the scenario's automated and manual evidence.

| Scenario | Formal lane(s) | Role / start state | Actions and observable pass condition | Result |
| --- | --- | --- | --- | --- |
| `UAT-8B-01` Product parity and UI | `uat-product` | Dataset manager; healthy canonical topology | Author, preview, revise, publish, and inspect Dataset/batch/catalog behavior across direct/lifecycle routes, accepted light/dark responsive baselines, accessibility, hydration, and clean console. | Passed |
| `UAT-8B-02` Fresh materialization | `uat-materialization` | Operator; authenticated empty databases | Apply the owner topology, verify exact releases/instances, typed read-back, owner receipts, isolation, and gateway boundary; reapply unchanged input and observe semantic no-op. | Passed |
| `UAT-8B-03` Configuration and diagnostics | `uat-operations` | Administrator; healthy Dataset module | Validate/apply exact Manifest fields, reject unknown/coerced/partial input, and observe exact health plus sanitized dependency/freshness diagnostics through generic Module Management. | Passed |
| `UAT-8B-04` Provider contracts, cursor, scope, and Dataset DAG | `uat-providers`, `uat-replay-refresh` | Full, restricted, and disjoint actors; canonical source/Dataset chain | Invoke synchronous refresh; prove unchanged-head zero-page/no-mutation; apply new/corrected/status-out/status-in/redacted/deleted changes over ordered and authorized-empty pages; interrupt/retry and race refresh; perform expired-cursor full rebase. Change `dataset.base` and observe base/derived/second-hop plus Component/Dashboard in one generation while independent binding stays stable; reject a cycle before staging; inject derived failure and prove whole closure/cursor/receipt rollback and prior last-good downstream result; preserve nondisclosure. | Passed |
| `UAT-8B-05` Cross-module exit and outage | `uat-crossmodule` | Authorized actor; healthy independent processes | Preview Dataset, execute Component, view Dashboard, stop Dataset/source provider, observe coherent exact degradation, restore, and observe healthy convergence without new authority. | Passed |
| `UAT-8B-06` Core subtraction/isolation | `uat-subtraction` | Operator; healthy topology | Prove exactly four Core transitions, one Dataset enrollment/navigation item, no active Core Dataset schema/adapter/payload/reverse SQL, owner-local analytics projection, separate analytics authority, and pairwise credential/database denial. | Passed |
| `UAT-8B-07` Failure retry and recovery | `uat-recovery` | Operator; deterministic pre-write and mid-apply fault profiles | Inspect retained failure, prove no unauthorized state, remove exact partial topology, execute clean successor apply and no-op, and restore canonical health without manual product repair. | Passed |
| `UAT-8B-08` Independent upgrade/rollback | `uat-upgrade` | Operator; real Dataset `0.9.0` baseline | Upgrade to `1.0.0`, roll back to `0.9.0`, restore `1.0.0`, and prove Dataset state/provider route preserved with unrelated owner images/containers/restarts/data/navigation unchanged and no Core fallback. | Passed |
| `UAT-8B-09` Editor provider boundaries | `uat-product`, `uat-providers` | Dataset manager; direct and lifecycle documents | Exercise Form/version pickers, rendered schema options, scope tree, principal labels, hydration, dirty navigation, and each isolated provider fault. Network evidence shows browser calls only Dataset routes; typed state remains intentional and unsaved input survives. | Passed |
| `UAT-8B-10` Reverse consumers and Operations | `uat-operations`, `uat-reverse-consumers` | Authorized and disjoint actors; populated and empty fixtures | Inspect Form Dataset Sources, `/operations` Dataset readiness/attention, and app summary. Prove scoped results, authorized empty versus undisclosed, explicit unavailable rather than false zero, unrelated content usable during outage, and exact recovery. | Passed |
| `UAT-8B-11` Resource, replay, and routing | `uat-resource-resolution`, `uat-replay-refresh` | Authorized/restricted actors; representative persistent mutations | Resolve/observe all three Dataset v2 resource types and compare wrong-owner/v1/known-restricted/random cases; retry create/update/publish/refresh representatives after commit-before-response loss; reject changed replay input and private nonce reuse; prove `/datasets/new`, `/sql-preview`, `/refresh`, and editor-option routes are never captured as Dataset IDs. | Passed |

- UAT result receipt: Produced
- Manual scenario evidence count: 11 / 11
- Final topology restoration: Passed
- Phase-local evidence index: Produced

## Dependency-Impact And Invalidation Proof

`planning-contract-alignment` and `runner-selftest` must exercise representative
changed-path cases against the contract's overlapping tracked patterns. The
expected result is the union of every matched domain, followed by lane
prerequisite closure and certificate-hash propagation; no first-match shortcut
is allowed.

| Representative change | Required domain/target result | Required formal result |
| --- | --- | --- |
| `docs/sprints/sprint-8b-validation-contract.json` | `sprint-contract` plus `documentation`; all 24 target mappings and the contract validator rerun | All 31 lanes are challenged because every lane consumes `sprint-contract`; no old certificate authorizes the changed contract. |
| `scripts/run-sprint-8b-implementation-readiness.ps1` only | `implementation-runner`; all 24 implementation targets and the aggregate implementation-readiness certificate are invalidated and rerun | Every downstream certificate is challenged by the changed implementation prerequisite hash; no Readiness, Rehearsal, Preflight, SIT, or UAT certificate may silently retain authority until the new implementation result authenticates. |
| `crates/tessara-dataset-module/**` or another Dataset product source | `product-source`; every intersecting product/boundary/materialization/consumer target reruns | All three Readiness and all eleven Rehearsal lanes are selected by their declared dependencies; after freeze, product change requires successor candidate plus complete SIT and UAT. |
| Response mutation/export migration under `crates/tessara-api/migrations/**`, or `scripts/materialize-sprint-8b.ps1` | Migration matches `product-source` plus `migrations-seeds`; materializer matches `migrations-seeds`, `deployment-materialization`, and `implementation-harness`. Rerun every intersecting Response contract/sync, migration, clean materialization, no-op, recovery, smoke, DAG, fixture, and alignment target. | Execute all intersecting Readiness/Rehearsal lanes with prerequisite closure. Because the materializer is shared implementation harness and live topology input, an uncertain or post-freeze change selects a successor candidate and complete SIT/UAT. |
| `scripts/run-sprint-8b-candidate-rehearsal.ps1` only | `rehearsal-runner`; rerun `runner-selftest` and `planning-contract-alignment` | All eleven Rehearsal lanes and downstream Preflight certificate are invalidated; Readiness may remain only with authenticated unchanged dependencies. |
| `scripts/run-sprint-8b-formal-uat.ps1` only | `uat-runner`; rerun `runner-selftest`, `uat-readiness`, and `planning-contract-alignment` | `readiness-acceptance`, `rehearsal-uat`, and all twelve UAT lanes are affected, with authenticated prerequisite closure; unrelated executed pre-freeze lanes may be inherited only by exact non-impact. |
| `scripts/run-sprint-8b-sit.ps1` only | `sit-runner`; rerun runner/alignment self-tests | All four SIT lanes are invalidated. UAT is downstream and cannot remain authoritative; rerun complete SIT then complete UAT under the coordinator's phase-local-runner decision. |
| `docs/sprints/sprint-8b-test-change-log.md`, the UI baseline, or `end2end/**` | `acceptance-inventory` and, for docs, `documentation`; rerun every consuming target including fixture/UI/smoke/UAT/alignment proofs | Execute the intersecting Readiness/Rehearsal lanes plus prerequisite closure. A changed acceptance interpretation after freeze invalidates downstream candidate evidence conservatively. |
| Validation policy/schema/playbook change | `validation-shared`; all consuming targets and lanes | Full affected pre-freeze certification; after freeze, uncertainty requires a successor with complete SIT/UAT. |
| Unknown/unmapped path, missing digest, overlapping-path union not proven, or unauthenticated prior receipt | mapping is uncertain | Conservative complete affected-phase execution; after freeze, successor candidate and complete SIT/UAT. |

Before freeze, only authenticated prior-passing lanes with every declared
dependency fingerprint and prerequisite certificate unchanged may use
`certification_basis: inherited_nonimpact`. A prerequisite certificate hash
change invalidates downstream certification even when a source fingerprint is
otherwise unchanged. After freeze, a candidate-affecting product, fixture,
environment, acceptance, deployment, or harness correction creates a successor
candidate and complete candidate-bound SIT and UAT; SIT lanes and manual UAT
scenarios are never inherited across candidates.

## Failure And Recovery Rules

- Retain every failed receipt and raw artifact; record phase/lane/stage,
  whether assertions or product actions began, source/environment/candidate
  identity, classification, safe narrow reproducer, correction, impact map,
  selected invalidation boundary, and authoritative replacement.
- A narrow reproducer diagnoses; it never satisfies an implementation target,
  Rehearsal certification lane, SIT lane, or UAT scenario.
- Pre-freeze failure produces one terminal attempt and consolidated defect
  batch. After correction and a passing implementation exit gate, recertify the
  affected dependency plan with prerequisite closure.
- A candidate-invalidating UAT defect forbids a passing `uat-result.json`.
  Finish only safe independent scenarios as non-authoritative defect harvest,
  restore canonically, assess the correction cone, perform focused repair
  validation if authorized, then return to final implementation readiness,
  pre-freeze certification, a successor freeze, complete SIT, and complete UAT.
- Evidence-publication-only repair may rerun finalization only when immutable
  raw results, source/environment identity, and hashes authenticate. Never
  choose a narrower boundary merely to avoid work.

## Post-SIT Defect Convergence (Conditional)

If triggered, retain under the evidence root:

- `uat-defect-harvest.json`
- `defect-batch.json`
- `correction-impact-assessment.json`, validated against the policy-v2 schema
- `focused-repair-validation/attempt-<n>.json`
- `canonical-restoration.json`
- `final-certification-entry.json`

Focused repair proves only a mutable correction cone. It cannot freeze a
candidate, authorize SIT/UAT, or authorize closeout.

## Failure And Invalidation Chronology

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction/narrow proof | Invalidation scope | Authoritative replacement |
| --- | --- | --- | --- | --- | --- | --- | --- |
| — | No validation activity during kickoff | No | None | N/A | N/A | N/A | N/A |

## Evidence Integrity

- Compact phase certificates authenticate: Passed — Implementation Readiness,
  Validation Readiness, Candidate Rehearsal, Preflight, SIT, and UAT.
- Phase-local indexes parse, reconcile, seal, and hash: Passed — 6 indexes.
- Raw evidence retained cold under ignored `artifacts/sprint-8b-closeout/`:
  840 authenticated artifacts.
- Routine downstream authorization consumed certificate and index hashes; raw
  evidence was opened once for the final full-integrity audit.
- Final complete integrity audit across all sealed indexes: Passed at closeout.
- Evidence-chain SHA-256:
  `06b3fe97bcb6d1a702a0076f9f6fff33aa2541b747eeb5f86c3374e8434981d2`.

## Closeout Authorization

- Status: Authorized
- Authorization receipt:
  `artifacts/sprint-8b-closeout/runs/83e8b123/closeout-authorization.json`
- Authorization SHA-256:
  `513301d0a32f127d1c169898391ad99b697bb0ea1b2376644c9df9bfb8a8e2c3`
- Authorized candidate fingerprint:
  `93c5b936807407b5a2007ccb1851d22a1d9a90bfc0cd84619637bbf080ac97f9`
- SIT passed: Yes — 4/4 candidate-bound lanes.
- UAT passed: Yes — 12/12 lanes and all 11 manual scenarios.
- Acceptance mapping complete: Yes — `ac-01` through `ac-26` and the roadmap
  exit condition map to implementation, formal, deployed, and manual proof.
- Invalidation decisions satisfied: Yes — complete chronology passed with 0
  unresolved records.
- Unresolved product decisions: None.
- Intended active route/slot: Dataset `1.0.0`, exact route selected by the
  applied Reference Blueprint.
- Application health: Passed — retained reviewer topology
  `tessara-s8b-uat-manual` at `http://127.0.0.1:49452`.
- Evidence source commit:
  `83e8b123de315ed2c7186ec4f0586f5a03198131`.
- Documentation commit: recorded after this closeout-only documentation commit.
- Authorization timestamp: retained in the authorization receipt.

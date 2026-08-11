# UAT-8A-06 — Isolation and unsupported old inputs

## 1. Test Script Summary

- System / Module: Tessara Components platform boundary
- Requirement: Sprint 8A AC-01, AC-02, AC-06, AC-12, AC-16, AC-18, and AC-19
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator plus authorized and restricted users
- Scenario: Confirm Component ownership/isolation and submit unsupported historical references, the retired `missing_policy` alias, and old payloads.
- Acceptance: There is one canonical v3 module path, no Core adapter or cross-owner storage edge, and old inputs fail without disclosure.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-06`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor bindings: `operator` account __________; `authorized-user` account __________; `restricted-user` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `negative-input-set`, `selected-component-release-instance`.
- Evidence folder and execution start time: ____________________
- Obtain the retained approved negative-input set for v1, v2, Core owner/type, the retired `missing_policy` alias, flat Dataset-major fields, and old payload shape.
- Record the selected Component Release/Instance shown in Module Management: ____________________

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | Review inventory, routes, process, database, manifest, and provenance. | Components and Dashboard each appear as real deployed modules; Dashboard appears once; Core exposes no Component product owner. | `module-isolation-record` | | | |
| 2 | Submit each approved old reference/payload, including `missing_policy` and flat Dataset-major fields, as an authorized user. | Each is rejected by the exact v3/typed-reference contract; no normalization helper, alias, adapter, ledger, copied-count expectation, or fallback is shown. | `old-input-rejection-record` | | | |
| 3 | Repeat known and random restricted inputs as the restricted user. | Responses are indistinguishable and disclose no protected identity. | `restricted-input-equivalence-record` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)

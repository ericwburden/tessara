# UAT-8A-03 — Configuration and diagnostics

## 1. Test Script Summary

- System / Module: Tessara Module Management / Components
- Requirement: Sprint 8A AC-08
- Environment: Frozen Sprint 8A UAT candidate
- User role: Global Module Management manager and reader plus Components manager
- Scenario: Configure the Components display label and Dataset timeout and review sanitized diagnostics.
- Acceptance: Manifest defaults are `Components` and `5`; only navigation/admin display uses the label; timeout range is 1–30; authority is correct; and diagnostics expose the selected Dataset binding/contract compatibility and health without product data, raw references, or secrets.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-03`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor bindings: `module-management-manager` account __________; `module-management-reader` account __________; `components-manager` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `original-display-label`, `original-dataset-timeout`, `temporary-settings`.
- Evidence folder and execution start time: ____________________
- Record the current display label and timeout: ____________________
- Use temporary label `Visual Parts` and timeout `6`.

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | Confirm the initial Manifest-driven values, then save `Visual Parts` and timeout `6`. | Defaults are `Components` and `5`; navigation/admin show the temporary label; product headings remain Components; diagnostics show redacted effective values. | `configuration-update-trace` | | | |
| 2 | Try blank/81-character labels, timeouts 0/31, an unknown field, and unsupported schema version. | Every invalid value is rejected without clamping or partial save. | `configuration-rejection-record` | | | |
| 3 | Repeat as a reader and as a Components manager without global configuration authority. | Read access follows role; unauthorized writes are denied. | `configuration-authority-record` | | | |
| 4 | Inspect diagnostics after a compatibility-dependent Component operation. | The selected `tessara.components.dataset-major-line` binding, Core-installation provider, contract ID/version, compatibility, health, observation time, and stable result/failure code appear; raw Dataset references, rows, counts, credentials, and secrets do not. | `sanitized-diagnostics-record` | | | |
| 5 | Restore the recorded settings. | Original configuration and navigation are restored. | `configuration-restoration-record` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)

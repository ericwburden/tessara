# UAT-8A-03 — Configuration and diagnostics

## 1. Test Script Summary

- System / Module: Tessara Module Management / Components
- Requirement: Sprint 8A AC-08
- Environment: Frozen Sprint 8A UAT candidate
- User role: Global Module Management manager and reader
- Scenario: Configure the Components display label and Dataset timeout and review sanitized diagnostics.
- Acceptance: Manifest defaults are `Components` and `5`; only navigation/admin display uses the label; timeout range is 1–30; authority is correct; and diagnostics expose the selected Dataset binding/contract compatibility and health without product data, raw references, or secrets.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Evidence folder and execution start time: ____________________
- Record the current display label and timeout: ____________________
- Use temporary label `Visual Parts` and timeout `6`.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Confirm the initial Manifest-driven values, then save `Visual Parts` and timeout `6`. | Defaults are `Components` and `5`; navigation/admin show the temporary label; product headings remain Components; diagnostics show redacted effective values. | | | |
| 2 | Try blank/81-character labels, timeouts 0/31, an unknown field, and unsupported schema version. | Every invalid value is rejected without clamping or partial save. | | | |
| 3 | Repeat as a reader and as a Components manager without global configuration authority. | Read access follows role; unauthorized writes are denied. | | | |
| 4 | Inspect diagnostics after a compatibility-dependent Component operation. | The selected `tessara.components.dataset-major-line` binding, Core-installation provider, contract ID/version, compatibility, health, observation time, and stable result/failure code appear; raw Dataset references, rows, counts, credentials, and secrets do not. | | | |
| 5 | Restore the recorded settings. | Original configuration and navigation are restored. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

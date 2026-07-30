# UAT-6D-04 — Configuration And Operational State

## 1. Test Script Summary

- System / Module: Module Management and Module SDK Reference
- Requirement: Standard configuration, probes, and sanitized diagnostics
- Environment: `http://127.0.0.1:8080`
- User role: Administrator
- Business scenario: An administrator manages the reference module through
  the standard Module Management surface and verifies operational state.
- Acceptance criteria: Valid configuration is normalized and retained;
  invalid input is rejected; liveness, readiness, and diagnostics report the
  current module without secrets or browser credentials.

## 2. Before You Start

Preconditions:

1. UAT-6D-01 passed.
2. Sign in as `admin@tessara.local`.

Input values:

- Valid display label: `SDK Reference UAT`
- Invalid display label: 129 consecutive `X` characters

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Open `/administration/modules/tessara.reference.module-sdk`. | The current release, enabled/configured/ready state, configuration, probes, and diagnostics are visible. |  |  |  |
| 2 | Save the valid display label, reload the page, and restart the reference service. | The normalized value reads back unchanged after reload and restart. |  |  |  |
| 3 | Try to save the invalid display label. | The change is rejected with actionable validation and the retained valid value is unchanged. |  |  |  |
| 4 | Review liveness, readiness, and diagnostics. | Probes are healthy and diagnostics contain bounded operational facts without keys, cookies, tokens, or personal data. |  |  |  |

## 4. Overall Test Result

- Overall result:
- Tester:
- Execution date:
- Defects: None /
- Comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

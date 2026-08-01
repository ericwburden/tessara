# UAT-6F-08 — Use Composition UI Across Sizes and Failure States

## 1. Test Script Summary

- System / Module: Native Application Composition UI
- Enhancement / Requirement: Responsive, accessible, non-disclosing status and
  failure experiences
- Test environment: Healthy Sprint 6F UAT reference installation
- User role: Tessara administrator and constrained read-only user
- Business scenario: Users inspect and act on composition state on desktop,
  tablet, and mobile-sized screens, including when Supervisor becomes
  temporarily unavailable. The UI remains understandable and preserves access
  boundaries.
- Acceptance criteria: Core composition pages render and hydrate cleanly at all
  sizes, keyboard operation remains usable, failures are clear and contained,
  and restricted users see no secrets or unauthorized details.

## 2. Before You Start

Preconditions:

1. A reference composition is healthy and has Blueprint, plan, approval, and
   receipt state available for inspection.
2. Use the temporary `UAT Composition Reader` account created in UAT-6F-03;
   the coordinator supplies its one-time password out of band.
3. The UAT operator may stop and restart only Supervisor for the failure check.
4. Browser developer tools are available for viewport and console checks.

Record Selection Criteria:

1. Use the current approved composition and latest receipt.
2. Use a non-destructive plan if an action must be demonstrated.

Record Actually Tested:

- Candidate commit/tree:
- Browser/version:
- Administrator account:
- Constrained account:
- Receipt revision:

Input Values to Use During Test:

1. Viewports: 1280 px, 768 px, and 390 px wide

Tester Instructions: Record screenshots at each viewport and any browser
console error. Restore Supervisor before finishing.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Open `/administration/composition` directly at 1280 px width. | A complete server-rendered page appears, then remains stable after hydration; Blueprint, plan, approval, and receipt are visually distinct. |  |  |  |
| 2 | Repeat at 768 px and 390 px widths. | Content remains readable without clipped actions or required horizontal page scrolling. |  |  |  |
| 3 | Navigate the page using keyboard only and activate a safe available control. | Focus order is logical, focus is visible, and controls have understandable accessible names. |  |  |  |
| 4 | Inspect browser history, theme behavior, and console while navigating away and back. | Back/forward navigation and theme remain coherent; no hydration or controller errors appear. |  |  |  |
| 5 | Sign in as the constrained user and open the composition route. | Only permitted read-only state is shown; approval/apply actions and restricted details or secrets are absent or denied clearly. |  |  |  |
| 6 | Have the operator stop Supervisor, then attempt an approved apply or status refresh. | The UI reports Supervisor unavailability clearly without losing Core navigation, leaking internal authorization data, or falsely reporting success. |  |  |  |
| 7 | Have the operator restart Supervisor and reload the page. | Supervisor returns healthy and the same persisted receipt/status is available again. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD

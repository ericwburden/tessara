# UAT-6E-03 — Use SSR, Hydration, And Responsive Layouts

## 1. Test Script Summary

- System / Module: Tessara Dashboard documents
- Requirement: Dashboard-owned SSR, hydration, assets, and presentation
- Test environment: `http://127.0.0.1:8080`, Dashboard `2.0.0`
- User role: Administrator / Dashboard reader
- Business scenario: A user directly loads Dashboard documents with and without
  JavaScript and uses them at common desktop, tablet, and mobile widths.
- Acceptance criteria: Essential content is useful without JavaScript; with
  JavaScript enabled the page hydrates without a visible error, remains
  keyboard usable, and has no horizontal page overflow.

## 2. Before You Start

Preconditions:

1. Select an existing Dashboard with placements.
2. Use a browser that permits JavaScript toggling and viewport resizing.

Record Actually Tested:

- Dashboard name/ID:
- Browser/version:
- Dashboard release metadata:
- Hydration asset digest:

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Disable JavaScript and directly load directory, detail, editor, and viewer routes. | Each route returns useful, authorized Dashboard content rather than a blank root or Core-owned Dashboard bootstrap. |  |  |  |
| 2 | Re-enable JavaScript and reload the viewer. | The document remains stable and interactive; no hydration failure is visible. |  |  |  |
| 3 | Inspect the document metadata and Dashboard assets. | Release `2.0.0` and content-addressed CSS, loader JS, bindings JS, and WASM assets are present; immutable assets load successfully. |  |  |  |
| 4 | Use keyboard navigation through primary actions and editor controls. | Focus is visible, order is logical, and actions can be reached without a pointer. |  |  |  |
| 5 | Inspect at approximately 1280 px, 768 px, and 390 px widths. | Content remains readable and operable without horizontal page overflow or obscured primary actions. |  |  |  |
| 6 | Switch between light, dark, or system theme when available. | Dashboard-owned content remains legible and visually consistent with the shell. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester:
- Test execution date:
- Defect IDs: None /
- Tester comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects


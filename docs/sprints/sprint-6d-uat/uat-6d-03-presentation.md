# UAT-6D-03 — Accessible And Responsive Presentation

## 1. Test Script Summary

- System / Module: Tessara Module SDK Reference UI
- Requirement: Shared theme, accessibility, responsive, and hydration behavior
- Environment: `http://127.0.0.1:8080`
- User role: Administrator
- Business scenario: An administrator uses the reference document across
  supported viewport and theme settings with keyboard-only interaction.
- Acceptance criteria: Content remains readable and operable at desktop,
  tablet, and phone widths; theme changes remain coherent; focus order and
  landmarks are usable; hydration enhances without replacing SSR content.

## 2. Before You Start

Preconditions:

1. UAT-6D-02 passed.
2. Open `/reference/module-sdk` while signed in.

Record actually tested:

- Browser and version:
- Operating-system theme:
- Input method:

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | View the page at 1280 px, 768 px, and 390 px widths. | No content is clipped or overlaps; cards and navigation remain readable. |  |  |  |
| 2 | Check light, dark, and system theme settings. | Text, surfaces, focus indicators, and status treatments remain legible and consistent. |  |  |  |
| 3 | Navigate using Tab and Shift+Tab only. | Focus is visible, follows document order, and reaches every actionable link. |  |  |  |
| 4 | Reload with JavaScript enabled and inspect the hydration status. | Existing SSR content remains stable and the enhancer reports successful hydration without a remount flash. |  |  |  |

## 4. Overall Test Result

- Overall result:
- Tester:
- Execution date:
- Defects: None /
- Comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

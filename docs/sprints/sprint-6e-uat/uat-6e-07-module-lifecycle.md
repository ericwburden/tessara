# UAT-6E-07 — Navigate Dashboard Through The Reusable Module Lifecycle

## 1. Test Script Summary

- System / Module: Tessara shell and Dashboards
- Enhancement / Requirement Reference: Module Browser Lifecycle v1
- Test environment: `http://127.0.0.1:8080`, Dashboard candidate `2.0.2`
- User role running the test: Administrator / Dashboard author
- Business scenario: An administrator moves between Core and Dashboard pages,
  edits a Dashboard, and uses browser history while the Tessara shell remains
  active. Direct links and recovery still work when lifecycle loading cannot.
- Acceptance criteria for this scenario: Navigation preserves the signed-in
  shell without a full-page reload, Dashboard state follows the requested URL,
  unsaved changes are protected, and direct-load/recovery paths remain usable.

## 2. Before You Start

Preconditions:

1. Dashboard candidate `2.0.2` is healthy, installed, and active.
2. Sign in as `admin@tessara.local` in a supported desktop browser.
3. Keep browser developer tools open to the Network and Console panels.
4. At least one editable Dashboard exists.

Record Selection Criteria:

1. Select a disposable or UAT-owned Dashboard that the administrator can edit.
2. The Dashboard should have a saved name and at least one placement.

Record Actually Tested:

- Dashboard name / ID:
- Browser / version:
- Starting Core route:

Input Values to Use During Test:

1. Unsaved name: `Sprint 6E lifecycle guard — unsaved`

Tester Instructions:

- Follow the steps in order and record what actually happens.
- Treat a new top-level HTML document request during soft navigation as a failure.
- Record any console error, visible duplicate shell, or repeated style corruption.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | From a Core route, note a visible shell marker such as the current theme and select **Dashboards**. | The Dashboard directory appears without requesting a new top-level HTML document; the signed-in shell and marker remain in place. |  |  |  |
| 2 | Open the selected Dashboard, then move through **Edit**, detail, and **View**. | Each URL and Dashboard view updates inside the existing shell without a full-page reload, duplicate navigation, or stale content. |  |  |  |
| 3 | Use browser **Back** and **Forward** across the Dashboard routes. | The URL, heading, title, and visible Dashboard state stay synchronized and no lifecycle or console error appears. |  |  |  |
| 4 | In the editor, change the name to the supplied unsaved value. Select a Core navigation destination and cancel the leave prompt. | The editor remains active and the unsaved value is retained. |  |  |  |
| 5 | Select the Core destination again and confirm leaving. | The Core page opens in the same shell; Dashboard styles and controls are removed and no Dashboard listener affects later Core navigation. |  |  |  |
| 6 | Repeat Core → Dashboard → Core navigation twice. | Each mount is clean: one Dashboard view, one shell, correct styling, and no accumulating prompts or duplicate actions. |  |  |  |
| 7 | Paste the selected Dashboard detail URL into a new tab and load it directly. | A useful complete Dashboard document loads, then becomes interactive with JavaScript enabled. |  |  |  |
| 8 | Disable JavaScript and reload that direct URL. | The complete document remains useful for authorized reading and navigation; it is not a blank module outlet. |  |  |  |
| 9 | Re-enable JavaScript, return to Core, temporarily make Dashboard lifecycle loading unavailable, and select **Dashboards**. | Core remains usable and shows the contained module failure state with an explicit complete-document recovery action. |  |  |  |
| 10 | Restore Dashboard availability and use the recovery action or retry navigation. | Dashboard recovers without signing in again or reloading unrelated Core state. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs, if any: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defects
- Business owner or reviewer:
- Review date: YYYY-MM-DD

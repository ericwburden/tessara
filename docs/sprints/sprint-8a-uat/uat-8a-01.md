# UAT-8A-01 — Unchanged Component experience

## 1. Test Script Summary

- System / Module: Tessara Components
- Requirement: Sprint 8A AC-01, AC-09, AC-14
- Environment: Frozen Sprint 8A UAT candidate
- User role: Component manager and reader
- Scenario: Create, edit, publish, view, and retire a disposable Component while confirming all supported seeded kinds remain usable.
- Acceptance: Product vocabulary and behavior remain familiar, and Component documents/assets load through the module-owned routes.

## 2. Before You Start

- Sign in with the supplied Component manager account.
- Select one recognizable seeded Component of each kind; record their displayed names below.
- Use a new disposable name: `UAT 8A Component <date-time>`.
- Record actually tested: ____________________

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Open Components from navigation and open every supported seeded kind. | Each list/detail/view route loads with canonical Components wording and recognizable output. | | | |
| 2 | Create the disposable Component, edit it, publish a new version, and open its view. | Each action succeeds and the published view reflects the saved change. | | | |
| 3 | Exercise the allowed lifecycle actions, then remove the disposable record. | State changes are clear and cleanup succeeds. | | | |
| 4 | Reload a direct Component URL and use browser back/forward. | The complete page, hydration, and navigation remain usable without console-visible failure. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects


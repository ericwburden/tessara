# Sprint 6C UAT Walkthrough

## Purpose

This guide validates Sprint 6C from an administrator and dashboard-author perspective. It covers the independently deployed Dashboards module, the reusable module-management pathway shared with Scoped Records, contained degraded states, and the approved responsive UI.

Run the scripts in order. Record any discrepancy in **Actual Result**, mark the step **Pass** or **Fail**, and add a defect ID when needed. Unless a step explicitly says otherwise, use the Tessara administrator account and the seeded **Demo Operations Dashboard**.

## Reference material

- Sprint scope and acceptance criteria: [sprint-6c-plan.md](sprint-6c-plan.md)
- Approved UI proposal: [sprint-6c-ui-review/README.md](sprint-6c-ui-review/README.md)
- UI delta decisions: [sprint-6c-ui-review/screen-delta-records.md](sprint-6c-ui-review/screen-delta-records.md)
- Engineering verification: [sprint-6c-verification.md](sprint-6c-verification.md)

---

# UAT-6C-01 — Use Dashboards normally

## 1. Test Script Summary

| Field | Value |
|---|---|
| Objective | Confirm that users can find, open, edit, and view a Dashboard through the independently deployed module without a visible loss of Sprint 6B2 behavior. |
| User role | Tessara administrator / dashboard author |
| Business outcome | Dashboard authoring and viewing remain usable after modularization. |

## 2. Before You Start

- The Sprint 6C stack is running.
- Sign in as `admin@tessara.local`.
- The seeded **Demo Operations Dashboard** exists.
- Use a desktop viewport of approximately 1280 × 720 or larger.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes/Defect ID |
|---:|---|---|---|---|---|
| 1 | Select **Dashboards** from the main navigation. | The Dashboard directory opens inside the Tessara shell and lists the seeded Dashboard. |  |  |  |
| 2 | Open **Demo Operations Dashboard**. | The detail page shows the saved title, status, metadata, and available authoring/viewing actions. |  |  |  |
| 3 | Open the Dashboard editor. | The editor loads the saved 12-column layout and all nine placements. |  |  |  |
| 4 | Select a placement and review its details without saving changes. | The placement is selected, its details are available, and its saved title and geometry remain unchanged. |  |  |  |
| 5 | Open the Dashboard viewer. | The Dashboard renders in view mode with all placements available and no module-unavailable fallback. |  |  |  |
| 6 | Return to the Dashboard directory. | Navigation stays within the Tessara shell and the Dashboard remains listed. |  |  |  |

## 4. Overall Test Result

| Result | Tester | Date | Notes |
|---|---|---|---|
|  |  |  |  |

---

# UAT-6C-02 — Administer independent modules through one shared pathway

## 1. Test Script Summary

| Field | Value |
|---|---|
| Objective | Confirm that Dashboards and Scoped Records use the same Core-owned module administration experience while retaining only module-specific metadata and configuration fields. |
| User role | Tessara administrator |
| Business outcome | A future independently deployed module can follow the same management template. |

## 2. Before You Start

- UAT-6C-01 passes.
- Both `tessara.dashboards` and `tessara.reference.scoped-records` are deployed and healthy.
- Keep the Dashboards product route enabled at the start of the script.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes/Defect ID |
|---:|---|---|---|---|---|
| 1 | Open **Admin → Module Management**. | The directory lists Dashboards and Scoped Records as independently deployed modules with availability, release/instance, serving state, and findings. |  |  |  |
| 2 | Open **Dashboards → Configuration**. | The page uses the shared Configuration/Application state layout and shows Dashboard-specific fields: schema version, display label, default page size, validation, and authoritative validator. |  |  |  |
| 3 | Review the final row of the Configuration panel. | The first-party transition-binding note appears as the last Configuration row, not as an unrelated Application state item. |  |  |  |
| 4 | Select **Open health and diagnostics**. | The link opens the Diagnostics view and displays readiness, liveness, isolated Dashboard database, Core authorization freshness, and Components compatibility. |  |  |  |
| 5 | Return to Configuration and turn **Product route enabled** off. | The control is clickable; the label and supporting copy reflect the disabled state; the module is shown as **Disabled**, not Blocked or Attention required; Dashboard navigation is removed. |  |  |  |
| 6 | Review the Dashboards Overview while disabled. | Deployment/container health remains separate from application enablement, and the lifecycle assessment describes a healthy but disabled application accurately. |  |  |  |
| 7 | Turn the product route back on. | The module returns to an enabled/ready serving state and Dashboard navigation returns. |  |  |  |
| 8 | Open **Scoped Records → Configuration**. | The same shared panels, tabs, actions, status vocabulary, findings presentation, and enablement interaction are used; only Scoped Records metadata/configuration differs. |  |  |  |

## 4. Overall Test Result

| Result | Tester | Date | Notes |
|---|---|---|---|
|  |  |  |  |

---

# UAT-6C-03 — Continue working when the Components provider is unavailable

## 1. Test Script Summary

| Field | Value |
|---|---|
| Objective | Confirm that a Components-provider outage is contained to affected placements and communicates the approved recovery path. |
| User role | Tessara administrator / dashboard author |
| Business outcome | Users retain access to the Dashboard and unaffected work while receiving actionable placement-level diagnostics. |

## 2. Before You Start

- A test operator has placed the Components provider in the **unavailable** test state.
- The Dashboards module itself remains healthy.
- Open **Demo Operations Dashboard** in the editor.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes/Defect ID |
|---:|---|---|---|---|---|
| 1 | Review an affected placement in the editor. | The entire placement panel uses warning styling and its normal icon is replaced by a warning icon. The saved title and geometry remain visible. |  |  |  |
| 2 | Review the panel without opening details. | Full error copy and recovery controls do not clutter the placement panel. |  |  |  |
| 3 | Select the warning icon. | A warning side sheet opens with the full error message, a large prominent centered warning icon, and **Retry resolution**. |  |  |  |
| 4 | Select **Retry resolution** while the provider is still unavailable. | The attempt is safe, the editor remains usable, and the affected placement remains in the contained warning state. |  |  |  |
| 5 | Open the Dashboard viewer. | Affected placements show concise contained unavailable cards while the Dashboard shell and any unaffected content remain usable. |  |  |  |
| 6 | Ask the test operator to restore the Components provider, then retry or refresh. | Placements resolve normally without losing their saved titles or geometry. |  |  |  |

## 4. Overall Test Result

| Result | Tester | Date | Notes |
|---|---|---|---|
|  |  |  |  |

---

# UAT-6C-04 — Recover from a Dashboards module outage

## 1. Test Script Summary

| Field | Value |
|---|---|
| Objective | Confirm that a Dashboards service outage is contained by Core and that the same Dashboard is usable after recovery. |
| User role | Tessara administrator |
| Business outcome | Tessara remains available, communicates the outage safely, and preserves Dashboard data. |

## 2. Before You Start

- Record the URL and title of **Demo Operations Dashboard**.
- A test operator has stopped only the Dashboards module.
- Core, Module Management, and Scoped Records remain running.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes/Defect ID |
|---:|---|---|---|---|---|
| 1 | Open the recorded Dashboard URL. | Core renders the Tessara shell and an explicit **Dashboards cannot be reached right now** fallback instead of a generic error page. |  |  |  |
| 2 | Review the fallback. | It explains that Core remains available and data is retained, and offers **Try Dashboards again** and **Open Module diagnostics**. |  |  |  |
| 3 | Select **Open Module diagnostics**. | Module Management opens the Dashboards diagnostics/health view and reports the unavailable module state. |  |  |  |
| 4 | Open Module Management and Scoped Records. | Both remain usable; the outage is contained to the Dashboards module. |  |  |  |
| 5 | Ask the test operator to restart Dashboards, then select **Try Dashboards again** or reopen the recorded URL. | The same Dashboard loads again with its saved title, layout, and placements. |  |  |  |

## 4. Overall Test Result

| Result | Tester | Date | Notes |
|---|---|---|---|
|  |  |  |  |

---

# UAT-6C-05 — Review responsive presentation

## 1. Test Script Summary

| Field | Value |
|---|---|
| Objective | Confirm that the approved Module Management and degraded-state experiences remain usable on desktop and mobile. |
| User role | Tessara administrator / dashboard author |
| Business outcome | Administrators can diagnose and recover the module from common supported viewport sizes. |

## 2. Before You Start

- Use a browser that can switch between 1280 × 720 and 390 × 844 viewports.
- Have the Dashboards Configuration page available.
- Have the Components-provider unavailable state available for the editor check.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes/Defect ID |
|---:|---|---|---|---|---|
| 1 | At 1280 × 720, review Dashboards Configuration and Diagnostics. | Information hierarchy matches the approved desktop proposal; panels do not overlap or clip. |  |  |  |
| 2 | At 390 × 844, review Dashboards Configuration. | Shared cards stack into a readable single-column layout with no horizontal page overflow. |  |  |  |
| 3 | At 390 × 844, open an unavailable placement’s side sheet. | The sheet remains readable, the warning icon is prominent, the close and retry controls are reachable, and content does not overflow horizontally. |  |  |  |
| 4 | At 390 × 844, review the Dashboard viewer’s unavailable placement. | The contained warning card, status, concise copy, and recovery action remain legible without obscuring the rest of the Dashboard. |  |  |  |

## 4. Overall Test Result

| Result | Tester | Date | Notes |
|---|---|---|---|
|  |  |  |  |

---

## Sprint 6C UAT Sign-off

| Field | Entry |
|---|---|
| Overall result | **Pass — all planned scenarios and previously identified exceptions pass** |
| Tested build/commit | Sprint 6C working-tree build based on `bf96f8e3fffe47dcbe71689ac67e454e5beb4c19` |
| Tester | Codex visual verification |
| Test date | 2026-07-27 |
| Open defect IDs | None |
| Sign-off notes | The stack was restored with the Components provider available, Dashboards healthy and enabled, and no Playwright Dashboard fixtures present. |

## Codex Visual Verification Record — 2026-07-27

The live Sprint 6C stack was exercised from the working-tree build above. Temporary provider-unavailable, disabled, and Dashboard-service-outage states were introduced during the walkthrough. The Components provider was restored to **available**, Dashboards was restarted, the product route was re-enabled, and the seeded Dashboard recovered with all nine saved placements.

| Scenario | Result | Observed outcome |
|---|---|---|
| UAT-6C-01 — Use Dashboards normally | **Pass** | Directory, detail, editor, and viewer loaded. **Demo Operations Dashboard** retained nine saved placements; the final 61-test Playwright run left no `pw-permissions-*` Dashboards in the directory. |
| UAT-6C-02 — Shared module administration | **Pass** | Dashboards and Scoped Records use the same shared configuration/application-state pathway. The transition binding is the final Dashboards Configuration row. Enablement, disabled vocabulary, hidden navigation, lifecycle separation, diagnostics, and recovery are consistent. |
| UAT-6C-03 — Components provider unavailable | **Pass** | Warning-tinted editor panels retain only the title, geometry, and warning icon. The provider-specific side sheet contains the large centered warning icon, full message, and retry action. All ten resolution states preserve nine placements and saved titles. |
| UAT-6C-04 — Dashboards module outage | **Pass** | Core rendered the approved contained warning panel and remained usable. Diagnostics changed to Not ready, Unhealthy, and database Unavailable while Dashboards was stopped, then returned to healthy after restart. |
| UAT-6C-05 — Responsive presentation | **Pass** | At 390 × 844, Configuration, the placement side sheet, and the degraded viewer had no horizontal page overflow; controls and warning content remained reachable. Desktop layouts did not overlap or clip. |

### Resolved UAT exceptions

| Defect | Resolution |
|---|---|
| UAT-6C-D01 | The definition-driven transition-binding note is the last row in the Dashboards Configuration panel. |
| UAT-6C-D02 | The issue sheet title is derived from the placement resolution state, including **Component provider unavailable**. |
| UAT-6C-D03 | The Core-owned outage fallback matches the approved warning composition and recovery actions. |
| UAT-6C-D04 | Module inventory/detail reads probe manifest-declared readiness and liveness endpoints, so diagnostics reflect outage and recovery state. |
| UAT-6C-D05 | Direct Playwright runs bind to the active Compose database safely, run statefully with one worker, and remove Dashboard fixtures during setup and teardown. |

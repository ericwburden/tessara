# UAT-8A-01 — Unchanged Component experience

## 1. Test Script Summary

- System / Module: Tessara Components
- Requirement: Sprint 8A AC-01, AC-07, AC-15, and AC-19
- Environment: Frozen Sprint 8A UAT candidate
- User role: Component manager and reader
- Scenario: Create, edit, publish, view, and retire a disposable Component while confirming all supported seeded kinds and accepted responsive/theme/accessibility behavior remain usable.
- Acceptance: Product vocabulary and behavior remain familiar; Component documents/assets load through module-owned routes; and light/dark, 1280/768/390 px, keyboard, 200% zoom, no-JavaScript SSR, and hydrated behavior have explicit evidence.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-01`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor bindings: `component-manager` account __________; `component-reader` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `seeded-component-kinds`, `disposable-component-name`, `presentation-matrix`.
- Evidence folder and execution start time: ____________________
- Sign in with the supplied Component manager account.
- Select one recognizable seeded Component of each kind; record their displayed names below.
- Use a new disposable name: `UAT 8A Component <date-time>`.
- Prepare named screenshot/trace targets for light and dark at 1280, 768, and
  390 CSS pixels, plus a 200% zoom capture and a JavaScript-disabled capture.
- Record actually tested: ____________________

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | Open Components from navigation and open every supported seeded kind. | Each list/detail/view route loads with canonical Components wording and recognizable output. | `seeded-kind-route-record` | | | |
| 2 | Create the disposable Component, edit it, publish a new version, and open its view. | Each action succeeds and the published view reflects the saved change. | `component-lifecycle-trace` | | | |
| 3 | Exercise the allowed lifecycle actions, then remove the disposable record. | State changes are clear and cleanup succeeds. | `component-cleanup-record` | | | |
| 4 | At 1280, 768, and 390 CSS pixels, inspect the directory, editor, detail, and viewer in both light and dark themes; capture the named evidence. | Layout, structured controls, table/visual output, focus, contrast, and vocabulary match the accepted baseline with no horizontal overflow or hidden required action. | `light-1280-screenshot`, `light-768-screenshot`, `light-390-screenshot`, `dark-1280-screenshot`, `dark-768-screenshot`, `dark-390-screenshot` | | | |
| 5 | Use keyboard-only navigation through filters, editor controls, preview, lifecycle confirmation, and back/forward; repeat a representative directory/editor/view flow at 200% browser zoom. | Focus order and restoration are visible, dialogs are operable, and zoom does not hide content or require two-dimensional scrolling. | `keyboard-operation-trace`, `zoom-200-screenshot` | | | |
| 6 | Disable JavaScript and directly load manager and reader Component URLs, then re-enable JavaScript and repeat direct load/back/forward while recording the console. | SSR is useful and role-correct without JavaScript; hydration/navigation preserve content with no hydration or browser-console errors. | `no-javascript-ssr-screenshot`, `hydration-browser-console` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Light/dark viewport screenshots: 1280 __________; 768 __________; 390 __________
- Keyboard/200%/no-JavaScript/hydration evidence: ____________________
- Evidence paths, console record, and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)

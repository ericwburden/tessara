# UAT-6D-02 — Reference Module Navigation And SSR

## 1. Test Script Summary

- System / Module: Tessara Module SDK Reference
- Requirement: Same-origin complete-document module experience
- Environment: `http://127.0.0.1:8080`
- User role: Administrator
- Business scenario: An administrator signs in through Core, follows normal
  navigation to the SDK reference, and receives a coherent module-owned
  document without a browser credential being forwarded to the module.
- Acceptance criteria: The reference destination is visible, returns useful
  SSR through the same origin, and uses content-addressed module assets.

## 2. Before You Start

Preconditions:

1. UAT-6D-01 passed.
2. Use `admin@tessara.local` with the development administrator password.

Record actually tested:

- Reference release:
- Browser and version:
- Asset digest/path:

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Sign in and inspect primary navigation. | `Module SDK Reference` is visible once. |  |  |  |
| 2 | Open `/reference/module-sdk`. | A complete Tessara-branded document shows the reference heading, manifest summary, capability, configuration, health, diagnostics, assets, and shutdown information. |  |  |  |
| 3 | Disable JavaScript and reload the route. | The same essential content and navigation remain useful without JavaScript. |  |  |  |
| 4 | Inspect the stylesheet and hydration script requests. | Both use `/_tessara/modules/{definition}/{release}/{digest}/...`; assets return immutable caching while the document is `no-store`. |  |  |  |

## 4. Overall Test Result

- Overall result:
- Tester:
- Execution date:
- Defects: None /
- Comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

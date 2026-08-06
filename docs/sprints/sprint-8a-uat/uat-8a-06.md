# UAT-8A-06 — Isolation and unsupported old inputs

## 1. Test Script Summary

- System / Module: Tessara Components platform boundary
- Requirement: Sprint 8A AC-01, AC-02, AC-06, AC-12, and AC-16
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator plus authorized and restricted users
- Scenario: Confirm Component ownership/isolation and submit unsupported historical references and payloads.
- Acceptance: There is one canonical v3 module path, no Core adapter or cross-owner storage edge, and old inputs fail without disclosure.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Evidence folder and execution start time: ____________________
- Obtain the retained approved negative-input set for v1, v2, Core owner/type, and old payload shape.
- Record the selected Component Release/Instance shown in Module Management: ____________________

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Review inventory, routes, process, database, manifest, and provenance. | Components and Dashboard each appear as real deployed modules; Dashboard appears once; Core exposes no Component product owner. | | | |
| 2 | Submit each approved old reference/payload as an authorized user. | Each is rejected by the exact current contract; no adapter, ledger, or fallback is shown. | | | |
| 3 | Repeat known and random restricted inputs as the restricted user. | Responses are indistinguishable and disclose no protected identity. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

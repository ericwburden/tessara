# UAT-6F-07 — Apply Inline and Content-Addressed Owner Bootstrap

## 1. Test Script Summary

- System / Module: Core, Dashboard, and Scoped Records bootstrap contracts
- Enhancement / Requirement: Digest-verified, owner-only, idempotent bootstrap
- Test environment: Empty or coordinator-prepared Sprint 6F UAT installation
- User role: Authorized local deployment operator and Tessara administrator
- Business scenario: An operator applies declared starter content from both an
  inline Blueprint value and a trusted local content-addressed source. The
  owning product surfaces show the created content exactly once.
- Acceptance criteria: Input digests are verified, owners create only their
  own content, receipts identify each result, unchanged replay is a no-op, and
  tampered content is rejected without mutation.

## 2. Before You Start

Preconditions:

1. The reference Blueprint provides the inline bootstrap declaration.
2. Use the repository's composition unit fixture for the local-CAS contract:
   `cargo test -p tessara-composition local_cas_acquisition_rejects_tampered_content`.
   Use the reference Blueprint's inline Core, Dashboard, and Scoped Records
   inputs for owner-level business verification.
3. The named test creates an isolated temporary CAS object, verifies the exact
   digest, tampers that disposable object, and proves rejection without touching
   the reference installation or a trusted original.

Record Selection Criteria:

1. Inline content must include at least one user-visible owner record.
2. CAS content must be non-secret and owned by exactly one product module.

Record Actually Tested:

- Inline input digest/owner:
- CAS input digest/owner:
- Starting owner record counts:
- First receipt digest:

Input Values to Use During Test:

1. Inline composition: `reference`
2. CAS source: isolated test-owned `sha256/<digest>` fixture

Tester Instructions: Never place a secret in inline or CAS bootstrap content.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Apply the reference composition with its inline bootstrap declarations. | Supervisor returns Core, Dashboard, and Scoped Records owner receipts, and the declared content is visible in each owning experience. |  |  |  |
| 2 | Record owner record counts and rerun the unchanged reference composition. | The new receipt reports a no-op and owner record counts do not increase. |  |  |  |
| 3 | Apply the coordinator-supplied composition containing the CAS declaration. | Supervisor acquires the exact locked digest and the owning module creates the expected business content with an owner receipt. |  |  |  |
| 4 | Open the owning product experience and locate the CAS-created content. | The content is visible once and unrelated product content is unchanged. |  |  |  |
| 5 | Rerun the unchanged CAS-backed composition. | The owner reports unchanged behavior and creates no duplicate content. |  |  |  |
| 6 | Attempt the coordinator-supplied tampered acquisition. | Digest verification rejects the input before owner mutation; the previous receipt and business content remain unchanged. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD

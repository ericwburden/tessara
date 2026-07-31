# UAT-6E-04 — Preserve Authorization And Nondisclosure

## 1. Test Script Summary

- System / Module: Tessara Dashboards authorization
- Requirement: Role, organization-scope, and nondisclosure preservation
- Test environment: `http://127.0.0.1:8080`, Dashboard candidate `2.0.2`
- User roles: Administrator, scoped manager, constrained reader
- Business scenario: Users with different authority open the same Dashboard
  area and receive only the actions, records, and placement information they
  are allowed to use.
- Acceptance criteria: Manage actions and organization data are scope-bound;
  redacted Components leak no metadata or execution; unknown and unauthorized
  resources are indistinguishable where nondisclosure applies.

## 2. Before You Start

Preconditions:

1. Three test accounts exist with administrator, scoped-manager, and
   constrained-reader authority.
2. Test Dashboards exist inside and outside the scoped manager's organization
   scope, including one placement hidden from the constrained reader.

Record Selection Criteria:

1. Select an in-scope Dashboard and an out-of-scope Dashboard.
2. Select a random nonexistent Dashboard identifier for comparison.

Record Actually Tested:

- Administrator account:
- Scoped-manager account and scope:
- Reader account:
- In-scope Dashboard:
- Out-of-scope Dashboard:

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | As administrator, open and manage the in-scope Dashboard. | All authorized read/manage actions are available. |  |  |  |
| 2 | As scoped manager, open and edit the in-scope Dashboard. | Read/manage actions work only within the assigned organization scope. |  |  |  |
| 3 | As scoped manager, request the out-of-scope Dashboard and a random unknown ID. | Responses disclose no useful difference in existence, metadata, or organization information. |  |  |  |
| 4 | As constrained reader, open the authorized Dashboard. | Reader actions work; create, edit, delete, and hidden organization choices are absent. |  |  |  |
| 5 | Inspect a placement whose Component is unauthorized for the reader. | Approved redacted/degraded presentation appears without Component name, configuration, binding ID, or execution. |  |  |  |
| 6 | Attempt direct manage API actions as the constrained reader. | The actions are rejected and no state changes. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester:
- Test execution date:
- Defect IDs: None /
- Tester comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

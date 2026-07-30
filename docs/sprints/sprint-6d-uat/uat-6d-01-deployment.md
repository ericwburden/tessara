# UAT-6D-01 — Fresh Deployment And Idempotent Bootstrap

## 1. Test Script Summary

- System / Module: Tessara Sprint 6D deployment
- Requirement: Fresh source-exact stack and repeatable materialization
- Environment: `http://127.0.0.1:8080`, Sprint 6D Compose profile
- User role: Deployment operator
- Business scenario: An operator deploys Tessara from an empty current
  baseline and registers the canonical reference module. Repeating bootstrap
  must make no material change.
- Acceptance criteria: Every service becomes healthy, the reference module is
  registered once, and the second bootstrap reports an exact no-op.

## 2. Before You Start

Preconditions:

1. The repository is clean on `codex/sprint-6d`.
2. No Tessara deployment is running.
3. The active `tessara` deployment volume has been removed.

Record actually tested:

- Source commit:
- Source tree:
- Core image ID:
- Reference image ID:

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Run `docker compose -f deploy/sprint-6d/compose.yaml up --build -d` with the clean source provenance variables. | The fresh stack builds and all long-running services become healthy. |  |  |  |
| 2 | Run `.\scripts\bootstrap-sprint-6d-deployment.ps1`. | Core imports one current reference-module release and one live instance. |  |  |  |
| 3 | Run the same bootstrap command again. | It reports `exact_noop` with zero material changes. |  |  |  |
| 4 | Open `http://127.0.0.1:8080/health`. | Core returns HTTP 200 and `ok`. |  |  |  |

## 4. Overall Test Result

- Overall result:
- Tester:
- Execution date:
- Defects: None /
- Comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

# Sprint 6F UAT Scripts

Run each script against a frozen Sprint 6F UAT candidate. Each file covers one
business scenario and records its own acceptance decision.

The frozen-candidate execution outcomes are retained in
[Sprint 6F UAT Results](../sprint-6f-uat-results.md).

1. [Bootstrap the complete reference application](./uat-6f-01-reference-composition.md)
2. [Bootstrap the signed reduced application](./uat-6f-02-reduced-composition.md)
3. [Keep planning, approval, and restricted access separate](./uat-6f-03-approval-and-access.md)
4. [Recover composition status after a Core restart](./uat-6f-04-core-restart-recovery.md)
5. [Adopt and reconcile composition drift](./uat-6f-05-drift-management.md)
6. [Emergency-disable and restore a module](./uat-6f-06-emergency-disable.md)
7. [Apply inline and content-addressed owner bootstrap](./uat-6f-07-owner-bootstrap.md)
8. [Use composition UI across sizes and failure states](./uat-6f-08-responsive-failure-states.md)

## Execution Rules

1. Record the candidate commit, Compose configuration identity, environment,
   browser, and tester in every script.
2. Run scripts in order unless the UAT coordinator supplies isolated
   environments for scenarios that require an empty installation.
3. Use the candidate-supported paths and fixtures recorded in each script. The
   UAT coordinator supplies temporary account passwords out of band after the
   candidate and environment are frozen.
4. Mark a scenario **Blocked** if its stated environment or temporary account
   cannot be prepared exactly as written.
5. Any failed or blocked scenario prevents unconditional Sprint 6F business
   acceptance.

Completed results belong in the candidate's retained UAT evidence directory;
these source scripts contain no execution result.

# Sprint 2F Rollout And Operations Notes

## Rollout

- Run `scripts/local-launch.ps1` to rebuild and seed the local review stack.
- Review `/operations` with `admin@tessara.local`.
- Review scoped behavior using Playwright permissions fixtures or a scoped operator account.

## Operational Notes

- `operations:view` is read-only.
- `analytics:refresh` remains separate.
- Reporting Data Status is derived from existing analytics fact availability.
- Dataset readiness is derived from published dataset revisions and ready response counts.
- Workflow assignment links use the existing assignment list filtered by assignment id until an assignment detail route is implemented.

## Support Notes

- If Operations is missing from navigation, verify the session has `operations:view` or `admin:all`.
- If status data is empty, verify seed/demo data and analytics materialization state.
- If Playwright leaves generated data behind, run the permissions suite cleanup or re-run the permissions suite; cleanup is now invoked before and after the serial test group.

## Purpose
Capture rollout and operations notes for Sprint 2F.
## Completion Guidance
Use during local review and handoff.
## Related Checks
`rollout-and-operations-notes`.
## Target Environments Or Adoption Context
Local Docker stack at `http://localhost:8080`.
## Protection Rules And Approval Constraints
Only users with `operations:view` or `admin:all` should access Operations.
## Sequencing Or Rollout Notes
Run `local-launch.ps1`, then UAT and browser review.
## Operational Assumptions
Seeded UAT demo data is present.
## Monitoring And Validation Watchouts
Watch scoped rows, table filters, and dataset readiness labels.
## Rollback, Pause, Or Escalation Triggers
Pause if unauthorized users see Operations or scoped users see out-of-scope rows.
## Communication Notes
Tell reviewers Operations is read-only in Sprint 2F.
## Trust-Boundary And Human-Control-Point Notes
No refresh or mutation controls are granted by `operations:view`.
## Known Gaps Or Operational Unknowns
Assignment admin detail page remains future work.

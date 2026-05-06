# Sprint 2E Plan

## Sprint Summary

Sprint 2E turns workflows into composed form-collection experiences. A workflow version contains ordered steps, each step may use a published version from a completely different form, and the workflow assignment still targets one organization node and one assignee. Runtime handoff is automatic for this sprint: submitting the current step activates the next step for the same assignee until the workflow instance is complete.

## Sprint Specifications

- Add ordered multi-step workflow version authoring where every step has a title and published form version.
- Allow a form and any of its published form versions to be included in any number of workflows; workflow form references are reusable ingredients, not exclusive ownership links.
- Preserve single-step compatibility for existing workflow version creation and existing response-start flows.
- Treat `workflows.form_id` and `workflow_versions.form_version_id` as legacy anchors while runtime and UI behavior read step form versions from `workflow_steps`.
- Enforce publish-time structural validation for steps, published form versions, and single-node assignment compatibility across all step forms.
- Advance workflow instances from one step to the next through the decomposed workflow/submission service layer.
- Keep same-assignee automatic handoff for Sprint 2E; per-step assignees and operator-mediated handoff are deferred.
- Add contextual assignment candidate and assignee APIs shared by the organization-node action and global assignment console.
- Keep touched workflow, organization, and response screens native SSR-owned, hydrated, and console-clean.

## Acceptance Criteria

- A workflow author can create a version with more than one ordered step, and steps can reference different forms.
- Publishing rejects incomplete, unpublished, or node-incompatible step collections with stable workflow-aware errors.
- Operators can assign eligible workflow/node candidates from both an organization node and the global assignment console.
- Bulk assignment creation is idempotent and reports created, reactivated, and skipped assignee outcomes.
- A response user can start step 1, submit it, see step 2 become active with its own form, complete the final step, and leave the workflow instance complete.
- Runtime surfaces show current step, next step, step count, current form, and completed-step history.
- Existing single-step workflow assignments, starts, drafts, submits, and review behavior remain compatible.

## Manual Test Plan

- Create or select published form versions from at least two different forms, then create a workflow version with both as ordered steps.
- Attempt to publish an invalid workflow version with missing steps, an unpublished form version, and an incompatible assignment-node scope; confirm stable errors.
- Publish the valid multi-step workflow version.
- From an organization node, use `Assign Workflow` and confirm only eligible workflow candidates and assignees appear.
- From `/app/workflows/assignments`, use the `Node path - Workflow` picker and assignee multiselect to create assignments.
- Sign in as the assignee, start the assigned work from `/app/responses`, complete step 1, return to the queue, and confirm step 2 is pending with the second form.
- Complete the final step and confirm the workflow instance is complete and history remains visible.
- Refresh touched workflow, organization, and response routes and confirm native route ownership, hydration, and browser-console cleanliness.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cd end2end; npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Add the Sprint 2E schema migration for workflow instance status and any missing runtime indexes needed by step handoff.
2. Extend workflow DTOs for ordered step authoring, version detail, assignment candidates, assignee options, and bulk assignment results.
3. Move touched workflow version, step, assignment, and runtime SQL into `workflows::repo` helpers while keeping handlers focused on extraction and response shaping.
4. Add workflow service validation for multi-step publish readiness and step-form node compatibility.
5. Update workflow version creation, publish, and detail endpoints for multi-step form collections while preserving single-step request compatibility.
6. Add shared assignment candidate, assignee option, and bulk assignment endpoints.
7. Update submission submit orchestration so completing a step advances or completes the workflow instance through the workflow service.
8. Update workflow authoring, assignment console, organization-node assignment, and response runtime UI surfaces.
9. Add API, web, and Playwright coverage for multi-step authoring, assignment eligibility, runtime handoff, and route quality.
10. Run the sprint verification set and record closeout evidence in the progress report.

## Dependencies And Blockers

- Sprint starts from clean `main` on branch `codex/sprint-2e` in `C:\Users\ericw\Projects\tessara-sprint-2e`.
- The roadmap `(Next)` marker selected `Sprint 2E: Multi-Step Workflow Authoring And Execution`.
- Full local UAT depends on Docker and Playwright availability in the current Windows session.

## Future Work

- Define the durable form/workflow version lifecycle model: at most one draft and one active version, publish retires the prior active version, retained responses stay pinned to retired versions, and operators choose whether retired-version assignments migrate to the new active version.
- Redesign the Home screen for delegated response work so accounts with accessible delegate work can discover, switch, or default into delegated work without relying on the Responses route first.
- Take a dedicated design detour after Sprint 2E to outfit the broader Tessara application in Rust/UI-style components, starting from the workflow data table, assignment picker, status badges, action buttons, and form footer patterns introduced here.
- Include stylesheet consolidation in that Rust/UI detour: make `style/main.scss` the documented app stylesheet entrypoint, split global/component/feature styles into named partials, clarify or remove the parallel `crates/tessara-web/assets/base.css` path, and add a lightweight verification check that deployed CSS contains newly introduced UI selectors.

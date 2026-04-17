# Sprint 2A Manual UAT Checkpoint

Date: 2026-04-17
Environment: local stack at `http://localhost:8080/app`

## Current Step Status

1. Step 1: Complete
   Native shell and route behavior verified.
2. Step 2: Complete
   Forms authoring on native SSR verified.
3. Step 3: Complete
   Workflow catalog and assignment console verified.
4. Step 4: Complete
   Response queue start, pending filtering, and assignee start behavior verified after the follow-up fixes.
5. Step 5: Complete
   Draft save and submit flow passed in manual UAT.
6. Step 6: Complete
   Delegate-aware response queue behavior passed in manual UAT.
7. Step 7: Complete
   Role gating passed in manual UAT.
8. Step 8: Complete
   Final spot check passed in manual UAT.

## Resume Point

Manual UAT is complete. No resume point is currently open.

## Step 4 Findings Captured During UAT

1. Starting the second pending assignment opened the wrong workflow-linked draft.
2. The assignment-backed start screen showed the wrong selected workflow work.
3. The assignee-facing `New Response` interstitial was unnecessary for pending assignment starts.
4. Submitted workflow work still appeared in `Pending Work`.
5. Submitted responses leaked into `Pending Work` instead of remaining only in `Submitted Responses`.
6. Draft-backed assignments were correctly removed from `Pending Work` and that behavior should remain.

## Confirmed Root Causes

- The pending-work query excluded drafts but not submitted responses.
- The assignment start endpoint reused the first submission for an assignment regardless of submission status.
- Assignee `/app/responses/new` still exposed an unnecessary chooser flow instead of returning users to the queue.

## Future User Stories Captured During UAT

- As an admin, my Assignments list should include a tabular view of all assignments. I should be able to sort and filter this list of assignments by workflow, node, assignee, and assignment status.
- As an admin, workflow assignments should eventually support explicit one-time versus recurring behavior, and the queue should display that state.

## Follow-Up Usability Tweaks

- Sidebar navigation now hides inaccessible product-area links for signed-in users.
- The draft response edit screen now uses one clear `Cancel` action instead of two.
- Workflow-card deep links into the assignment console now preserve workflow context and prefill both workflow and node after live redeploy verification.
- Delegate-context switching on `/app/responses` now updates the queue panes in place on the same document instead of forcing a full document reload.

## Local Verification Since The Last Manual Pass

- `respondent` now sees `Responses` in the sidebar and no unauthorized `Organization`, `Forms`, or `Workflows` links in the sidebar or `/app` Product Areas.
- `admin` now sees the full authorized sidebar and matching `/app` Product Areas links.
- Workflow `Assignments` deep links now land on `/app/workflows/assignments?workflowId=...` with the selected workflow and a default node prefilled.
- Left-nav content switching across touched Sprint 2A routes still works after the navigation fix.

## Final Manual UAT Result

- Step 5: Pass
- Step 6: Pass
- Step 7: Pass
- Step 8: Pass
- Sprint 2A manual UAT walkthrough is complete.

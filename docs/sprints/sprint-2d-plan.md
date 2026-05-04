# Sprint 2D Plan

## Sprint Summary

Sprint 2D completes the end-user response lifecycle by making pending starts, draft save/resume, final submit, and submitted read-only review coherent from the native Responses area. The sprint is respondent-first while preserving admin/operator review and scoped access.

## Sprint Specifications

- Deliver pending, draft, submitted, and read-only review flows through `/app/responses*`.
- Preserve existing public routes and endpoints: `/app/responses*`, `/api/responses/options`, `/api/submissions*`, and workflow-assignment start endpoints.
- Keep response edit, save, submit, and review routes native SSR-owned from first delivery with no new bridge fallback.
- Continue the Sprint 2C `handler`, `service`, and `repo` split for touched `submissions` and workflow-runtime behavior.
- Move response-facing browser auth/session use onto `AuthenticatedRequest` or config-aware helpers so customized browser cookie names work across touched flows.
- Keep bearer-token responses reserved for explicit script, test, and API flows rather than normal browser sign-in behavior.
- Make submit server-authoritative, with UI validation and feedback as an early assist rather than the source of truth.

## Acceptance Criteria

- A response user can start assigned work, save a draft, return to the same draft, and see saved values intact.
- Submit fails visibly and leaves the response as a draft when required values are missing or invalid.
- Submit succeeds only once, sets `submitted_at`, completes the linked workflow step, records audit, removes the response from pending/draft queues, and shows read-only review.
- Submitted responses cannot be edited, saved, resubmitted, or deleted.
- Admins can review all responses, scoped operators only see in-scope responses, and response users/delegators only see their own or accessible delegated work.
- Touched `/app/responses*` surfaces hydrate cleanly, remain console-clean, and do not add bridge-backed behavior.

## Manual Test Plan

- Sign in as respondent, start assigned response work from `/app/responses`, save a draft, return to the queue, resume the same draft, and confirm values persist.
- Attempt to submit a draft with missing required values and confirm the UI shows useful feedback while the server keeps the submission in draft status.
- Complete required values, submit, confirm the response opens as read-only detail, and verify it disappears from pending/draft queues.
- Try to open the submitted response edit route and confirm editable controls are unavailable.
- Sign in as admin and operator to confirm review visibility follows full-admin and effective-scope rules.
- Sign in as delegator to confirm delegated response context only exposes accessible delegate work.
- Refresh `/app/responses`, `/app/responses/new`, `/app/responses/{id}`, and `/app/responses/{id}/edit` and confirm route ownership, hydration, and browser-console cleanliness.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-submissions`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cd end2end; npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Move touched response lifecycle SQL for draft save/resume/submit/review into `submissions::repo` helpers.
2. Move response lifecycle orchestration into `submissions::service`, keeping handlers focused on extraction and JSON response shaping.
3. Replace touched response handler auth extraction with `AuthenticatedRequest` or config-aware authentication.
4. Tighten submit rules so null/empty required values are missing, submitted responses are immutable, workflow-step completion happens only after validation, and audit follows successful persistence.
5. Polish the native Responses UI for assigned starts, draft resume/edit, submit feedback, and submitted read-only review.
6. Add API coverage for draft save/resume, strict submit rejection, successful submit, submitted immutability, scoped review denial, delegation context, and custom cookie-name browser auth.
7. Add Playwright coverage for respondent lifecycle behavior and `/app/responses*` hydration/console cleanliness.
8. Run the sprint verification set and update the progress report with results.

## Dependencies And Blockers

- Sprint starts from clean `main` on branch `codex/sprint-2d` in `C:\Users\ericw\Projects\tessara-sprint-2d`.
- The roadmap `(Next)` marker selected `Sprint 2D: Draft, Submit, And Review Response Slice`.
- No schema migration is planned unless implementation discovers a hard blocker.
- Full local UAT depends on Docker and Playwright availability in the current Windows session.

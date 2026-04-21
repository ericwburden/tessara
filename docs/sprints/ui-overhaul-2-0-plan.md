# UI Overhaul 2.0 Sprint Plan

## Sprint Summary

UI Overhaul 2.0 adopts the approved out-of-roadmap UI migration as a dedicated sprint branch and worktree so the redesign can be executed as a normal Tessara backlog rather than as a planning artifact. Scope for this sprint is anchored in:

- [out-of-roadmap-ui-migration-plan.md](./out-of-roadmap-ui-migration-plan.md)
- [out-of-roadmap-ui-migration-github-ready.md](./out-of-roadmap-ui-migration-github-ready.md)
- [ui-guidance.md](../ui-guidance.md)
- [ui-guidance-spec.md](../ui-guidance-spec.md)

This sprint does not add roadmap feature scope. It converts the approved UI migration into a tracked implementation sequence covering shell posture, auth boundary, home, organization, builder, workflow/response alignment, and verification updates.

## Sprint Specifications

- Execute from branch `codex/ui-overhaul-2-0` in sibling worktree `D:\Projects\tessara-ui-overhaul-2-0`.
- Treat the existing out-of-roadmap UI migration plan as the scope source of truth.
- Execute the live GitHub issue chain `#59` through `#68` under the `Out-Of-Roadmap UI Migration` milestone.
- Keep the sprint bounded to UI migration and narrow contract support already called out in the approved plan.
- Do not add new roadmap product features outside the approved UI guidance and spec.

## Acceptance Criteria

- A dedicated UI migration worktree exists and is ready for execution against the approved sprint scope.
- The approved UIM backlog exists as live GitHub issues `#59` through `#68` with labels, milestone context, and dependency notes carried forward from the draft.
- The execution order is explicit enough to identify serial foundation work, the first safe parallel splits, and the closeout gate.
- The plan preserves Tessara delivery gates for SSR ownership, hydration safety, responsive behavior, browser-console cleanliness, and scripted sprint verification on touched routes.

## Manual Test Plan

- Validate the shared shell on desktop, tablet, and mobile widths after each navigation or layout change.
- Verify `/app/login` remains outside the authenticated shell and still routes correctly after successful sign-in.
- Exercise unauthorized deep links to confirm redirect-to-home plus transient feedback behavior.
- Walk the main routes affected by the sprint: `/app`, `/app/organization`, `/app/forms`, `/app/workflows`, `/app/responses`, `/app/admin`, and `/app/migration` when touched.
- Confirm the sidebar footer correctly reflects signed-in account, acted-as context, scope roots, and theme state.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `npx playwright test`
  - targeted in practice through the repository Node/Playwright entrypoint if `npx` is unavailable on `PATH`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Establish shell structure and route-boundary behavior.
   - `UIM-01` / `#59` Rebuild the shared shell information architecture.
   - `UIM-03` / `#61` Replace unauthorized dead-end panels with redirect and transient feedback.
   - `UIM-04` / `#62` Replace the transitional login shell with the bare sign-in surface.
2. Move persistent user context into the shell and make home depend on it.
   - `UIM-02` / `#60` Move account, delegation, scope, and theme into the sidebar footer.
   - `UIM-05` / `#63` Migrate Home to the queue-first operational surface.
3. Rebuild the major route families that depend on the new shell language.
   - `UIM-06` / `#64` Rebuild Organization around the scope-aware explorer pattern.
   - `UIM-07` / `#65` Add section-level builder metadata support required by the approved UI.
   - `UIM-08` / `#66` Rebuild the form builder around stacked section panels.
   - `UIM-09` / `#67` Align Workflows and Responses to the shared shell posture.
4. Close the sprint with the updated regression and verification contract.
   - `UIM-10` / `#68` Rewrite UI regression coverage and sprint-close verification for the new shell.

## Dependencies And Blockers

### Serial foundation

- `UIM-01` is the first execution anchor because it changes navigation, responsive chrome, and the shell contract other tickets rely on.
- `UIM-03` should follow immediately after `UIM-01` because unauthorized routing behavior affects global shell flow and browser assertions.
- `UIM-04` should land before broad route restyling so the auth boundary is stable while shell changes continue.
- `UIM-02` depends on `UIM-01` and should land before `UIM-05` and `UIM-09`, since both use shared shell context.

### First parallel split

- After `UIM-01`, `UIM-03`, and `UIM-04` are stable, two tracks can move in parallel:
  - Track A: `UIM-02` -> `UIM-05` -> `UIM-06` -> `UIM-09`
  - Track B: `UIM-07` -> `UIM-08`
- Track B is intentionally isolated because the builder metadata contract can be implemented and validated without waiting on the home and organization migration.

### Closeout gate

- `UIM-10` remains the final gate after `UIM-03`, `UIM-04`, `UIM-05`, `UIM-06`, `UIM-08`, and `UIM-09` land.
- Sprint closeout stays blocked until the new shell assertions are encoded in browser automation and the scripted verification stack reflects the migrated UI contract.

### Known context

- Kickoff started from a clean local `main` checkout at commit `e165fdc`.
- After fetch, local `main` was ahead of `origin/main` by 5 commits, so this sprint branch intentionally inherits the current local integration state rather than the remote tip alone.

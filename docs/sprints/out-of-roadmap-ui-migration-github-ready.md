# Out-Of-Roadmap UI Migration GitHub-Ready Issues

Derived from:

- [out-of-roadmap-ui-migration-plan.md](./out-of-roadmap-ui-migration-plan.md)
- [ui-guidance.md](../ui-guidance.md)
- [ui-guidance-spec.md](../ui-guidance-spec.md)

## Conventions

- These issues are for an out-of-roadmap migration sprint that updates the current UI foundation without expanding roadmap feature scope.
- Ticket IDs use the `UIM` prefix so they can be turned into GitHub issues directly.
- Labels follow the repo pattern: `phase:*`, `sprint:*`, `type:*`, `status:*`, and `area:*`.
- Every issue inherits the normal Tessara delivery gates for SSR ownership, browser-console cleanliness, and sprint-close verification on touched routes.

## Out-Of-Roadmap UI Migration

### UIM-01: Rebuild the shared shell information architecture
**Scope:** Frontend shell / navigation / responsive chrome  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:navigation`, `area:shell`  
**Depends on:** None  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue replaces the current grouped shell contract in `native_shell.rs` with the approved unified shell. The primary goal is to make the authenticated application read as one permission-gated product shell rather than a mix of product, analytics, transitional, and internal buckets. This ticket owns navigation order, sidebar grouping, responsive shell chrome, and top-bar cleanup.

**Work:**
- Replace the current `PRODUCT_LINKS`, `ANALYTICS_LINKS`, `INTERNAL_LINKS`, and `TRANSITIONAL_LINKS` presentation with the approved primary navigation order plus the secondary `Admin` group.
- Remove `Reports` from the default sidebar contract while preserving route reachability where still needed.
- Keep `Components` in the primary navigation and `Datasets` in the secondary `Admin` group.
- Update shell markup and styling so desktop, tablet, and mobile behavior matches the approved sidebar model.
- Remove top-bar account/session controls so the top bar is limited to search, bell, help, and mobile nav.

**Acceptance criteria:**
- The sidebar shows `Home`, `Organization`, `Forms`, `Workflows`, `Responses`, `Components`, and `Dashboards` in that order.
- The `Admin` group contains `Datasets`, `Administration`, and `Migration`.
- The `Admin` group appears only when the user has `admin:all`.
- `Reports` is absent from the default sidebar.
- The top bar contains only global search, notifications, help, and mobile nav.

### UIM-02: Move account, delegation, scope, and theme into the sidebar footer
**Scope:** Frontend shell / session context  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:auth`, `area:shell`  
**Depends on:** `UIM-01`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue replaces the current `SessionSummary`, top-bar sign-out, and top-bar theme control with the approved footer context block. The primary goal is to make the shell carry stable account context instead of forcing delegation and scope discovery to remain route-local.

**Work:**
- Replace `SessionSummary` with a footer block that shows the signed-in user.
- Surface the current acted-as user when delegation is active.
- Surface visible top-level scope roots in a compact expandable treatment.
- Replace the current theme wording/control with a compact selector for `System`, `Light`, and `Dark`.
- Keep sign-out and account actions in the account/footer context instead of the top bar.
- Extend the account/session projection if needed so the shell gets the correct acted-as and scope-root data shape.

**Acceptance criteria:**
- The sidebar footer shows account context on all authenticated routes.
- Delegation context is visible in the shell when the user is acting for another user.
- Scope roots are visible from the shell without forcing the user into the Organization route.
- Theme options are `System`, `Light`, and `Dark`, and theme selection no longer lives in the top bar.

### UIM-03: Replace unauthorized dead-end panels with redirect and transient feedback
**Scope:** Frontend shell / authorization feedback  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:authorization`, `area:ux`  
**Depends on:** `UIM-01`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue removes the current in-page `Access Restricted` dead-end behavior from `NativePage` and replaces it with the guidance-required redirect-to-home contract. The primary goal is to make unauthorized deep links feel like a normal application boundary rather than a stranded terminal state.

**Work:**
- Introduce a small shared shell notification or toast primitive if one does not already exist.
- Replace route-local unauthorized rendering with redirect to `/app`.
- Surface access-denied feedback after redirect.
- Update touched shell and route tests to assert the new behavior.

**Acceptance criteria:**
- Unauthorized deep links no longer render an in-page `Access Restricted` panel.
- The user is redirected to `/app`.
- The redirect presents transient access-denied feedback.
- The new behavior is covered in browser automation.

### UIM-04: Replace the transitional login shell with the bare sign-in surface
**Scope:** Frontend auth / shell boundary  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:auth`  
**Depends on:** `UIM-01`, `UIM-03`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue brings `/app/login` into line with the approved UI guidance and mockups. The primary goal is to remove the current transitional login shell and replace it with the bare sign-in surface that sits outside the authenticated application shell.

**Work:**
- Replace the current login presentation with a bare auth-only surface.
- Ensure the authenticated shell does not render on the sign-in page.
- Keep the current auth flow and session behavior working through the new presentation.
- Remove any promo, metrics, or post-auth shell cues from the sign-in page.

**Acceptance criteria:**
- `/app/login` renders without the authenticated application shell.
- The sign-in page contains only auth-relevant content.
- Sign-in success still routes the user into the authenticated shell correctly.
- Login behavior remains SSR-safe and hydration-safe.

### UIM-05: Migrate Home to the queue-first operational surface
**Scope:** Frontend home / workspace  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:home`, `area:organization`  
**Depends on:** `UIM-01`, `UIM-02`, `UIM-03`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue replaces the current launcher-card home with the approved operational home. The primary goal is to make `/app` feel like the start of actual work rather than a directory of destinations.

**Work:**
- Remove the current product/transitional/internal launcher sections from `HomePage`.
- Introduce a primary assignment or current-work surface.
- Render compact, text-first metrics rather than wide summary cards.
- Add quieter hierarchy context using the same explorer language approved for Organization.
- Keep internal/admin destinations discoverable through the shell rather than repeated as body launchers.

**Acceptance criteria:**
- `/app` no longer reads as a launcher directory.
- Current work or assignment context is visually primary.
- Metrics are compact and glanceable.
- Hierarchy context is visible without becoming a second destination launcher.

### UIM-06: Rebuild Organization around the scope-aware explorer pattern
**Scope:** Frontend organization / responsive layout  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:organization`, `area:responsive`  
**Depends on:** `UIM-01`, `UIM-05`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue migrates the Organization route family from its current list/detail emphasis to the approved explorer-first model. The primary goal is to make hierarchy traversal quieter, more scope-aware, and more obviously connected to related work.

**Work:**
- Rework `/app/organization` so desktop and tablet use explorer plus selected-node detail.
- Use scope-aware page titles such as `Partner Explorer` instead of generic `Organization` labels when appropriate.
- Reduce node chrome and stop treating each visible node as a separate card.
- Make the selected-node panel lead with related work such as forms, responses, dashboards, open issues, and recent changes.
- Implement the mobile tree-plus-detail-sheet pattern.
- Keep create and edit flows consistent with the calmer explorer language where they remain separate routes.

**Acceptance criteria:**
- Organization uses a quiet indented hierarchy explorer on desktop and tablet.
- Mobile uses a tree-selector plus lower detail region.
- The page title is scope-aware.
- Related work leads the selected-node panel.

### UIM-07: Add section-level builder metadata support required by the approved UI
**Scope:** Backend and frontend contract / forms  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:backend`, `area:frontend`, `area:forms`  
**Depends on:** None  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue closes the contract gap between the approved builder design and the currently visible form section model. The primary goal is to add only the missing section metadata needed to support an honest section-oriented authoring UI.

**Work:**
- Confirm the existing section model and endpoints in `crates/tessara-api/src/forms.rs`.
- Add section description support if it is currently missing.
- Add section column-count support if it is currently missing.
- Update the frontend transport and rendering contracts so the builder can read and write those values.
- Keep the change narrowly scoped to section-level settings needed by the approved builder UI.

**Acceptance criteria:**
- The forms API can persist and return the section settings required by the builder UI.
- The new section settings are available to the native form editor.
- The change does not add unrelated form-builder scope.

### UIM-08: Rebuild the form builder around stacked section panels
**Scope:** Frontend forms / builder UX  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:forms`, `area:ux`  
**Depends on:** `UIM-01`, `UIM-07`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue migrates the form edit workspace to the approved calmer builder posture. The primary goal is to keep the canvas primary, expose section-level settings in context, and reduce the current dense record-card feel.

**Work:**
- Replace the current draft workspace presentation with vertically stacked section panels.
- Surface section title, description, and column count at the section container.
- Keep insert affordances adjacent to the canvas.
- Move deeper field settings into a selection-driven properties panel or responsive drawer.
- Preserve lifecycle and save/publish actions as page-level controls rather than field-level clutter.

**Acceptance criteria:**
- The form builder canvas is organized around stacked section panels.
- Section-level settings are visible in the canvas.
- Field-level controls are calmer and visually subordinate to page-level lifecycle actions.
- The builder remains usable on tablet and mobile widths.

### UIM-09: Align Workflows and Responses to the shared shell posture
**Scope:** Frontend workflows / responses / shared context  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:feature`, `status:backlog`, `area:frontend`, `area:workflows`, `area:responses`  
**Depends on:** `UIM-01`, `UIM-02`, `UIM-05`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue keeps the existing workflow and response route ownership but aligns their presentation to the approved shell and context model. The primary goal is to make them feel like part of the same application system rather than adjacent management pages with route-local context duplication.

**Work:**
- Rework `/app/workflows` to read as a directory-first workflow surface.
- Keep `/app/workflows/assignments` as the dedicated assignment-management surface, but align its layout language to the new shell and directory posture.
- Reduce delegation emphasis inside Responses when the same context is already visible from the shell footer.
- Keep assignment-backed start and resume flows intact while updating the surrounding layout and wording.

**Acceptance criteria:**
- Workflows reads as a directory-first route.
- Assignments remains a dedicated management route.
- Responses still supports delegation and assignment-backed work.
- Shared shell context reduces redundant route-local delegation chrome.

### UIM-10: Rewrite UI regression coverage and sprint-close verification for the new shell
**Scope:** QA / automation / closeout  
**Labels:** `phase:ui-migration`, `sprint:ui-migration`, `type:test`, `status:backlog`, `area:frontend`, `area:qa`, `area:ci`  
**Depends on:** `UIM-03`, `UIM-04`, `UIM-05`, `UIM-06`, `UIM-08`, `UIM-09`  
**Milestone:** `Out-Of-Roadmap UI Migration`


**Description:**
This issue updates the automated and scripted closeout expectations so they validate the approved UI contract rather than the pre-mockup shell. The primary goal is to make sprint-close verification catch regressions against the new UI direction.

**Work:**
- Update `end2end/tests/app.spec.ts` to assert the new shell structure, footer context, access behavior, and route posture.
- Update any touched Rust UI tests that assert old headings or controls.
- Update `scripts/uat-sprint.ps1` and `scripts/smoke.ps1` if they encode old UI assertions.
- Run and document the closeout validation stack for the touched routes.

**Acceptance criteria:**
- Browser coverage asserts the new shell rather than the old one.
- Verification scripts still run cleanly for the touched UI.
- The sprint-close validation stack is ready to be reused when this migration sprint is executed.

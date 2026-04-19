# Out-Of-Roadmap UI Migration Sprint Plan

## Sprint Summary

This out-of-roadmap sprint updates Tessara's current UI toward the approved direction captured in:

- [ui-guidance.md](../ui-guidance.md)
- [ui-guidance-spec.md](../ui-guidance-spec.md)
- [tessara-redesign-exploration.html](../mockups/tessara-redesign-exploration.html)
- [tessara-responsive-exploration.html](../mockups/tessara-responsive-exploration.html)

This sprint does not add new roadmap product scope. It reshapes the existing authenticated shell and the already-delivered route surfaces so they conform to the current UI guidance and specification:

- one shared shell
- one shared user space
- permission-gated navigation and controls
- queue-first home
- explorer-first organization
- calmer section-oriented form authoring
- bare sign-in outside the application shell

The intended result is a UI foundation that future roadmap sprints can build on without carrying forward pre-guidance shell and layout drift.

## Starting Point In The Current Repo

This plan starts from the current repository state after the Sprint 2A and Sprint 2B code changes that are already present in `crates/tessara-web`, even where roadmap and progress docs still describe older sprint sequencing.

### What already exists

- A native Leptos SSR authenticated shell is already in place for the core `/app` experience.
- `/app`, `/app/organization*`, `/app/forms*`, `/app/workflows*`, and `/app/responses*` already have native route ownership in `crates/tessara-web/src/features/`.
- The shell already supports desktop, tablet, and mobile sidebar state changes in `crates/tessara-web/src/features/native_shell.rs`.
- Capability-gated route visibility already exists through the current `NavLinkSpec` model and `visible_links(...)`.
- Delegation-aware response behavior already exists in `crates/tessara-web/src/features/responses.rs`.
- Scope-aware organization naming logic already exists in `crates/tessara-web/src/features/organization.rs`.
- Shared Playwright coverage already exercises shell, forms, organization, workflows, migration, and responses routes through `end2end/tests/app.spec.ts`.

### What is still out of alignment with the approved UI direction

#### Shared shell

- `native_shell.rs` still organizes navigation as `PRODUCT_LINKS`, `ANALYTICS_LINKS`, `INTERNAL_LINKS`, and `TRANSITIONAL_LINKS`.
- `Reports` still appears as a default-shell concept.
- `Datasets` and `Components` are not yet placed according to the approved product-first sidebar contract.
- The top bar still owns `SignOutButton` and `ThemeToggle`.
- The sidebar footer still renders `SessionSummary` rather than account, acted-as user, scope roots, and theme selection.

#### Auth boundary

- `LoginPage` in `crates/tessara-web/src/features/home.rs` is still delivered through the transitional login shell rather than the bare sign-in surface required by the mockups and spec.

#### Access behavior

- Unauthorized route access still renders an in-page `Access Restricted` panel in `NativePage`.
- The UI guidance and Allium spec instead require redirect-to-home behavior plus transient access-denied feedback.
- No shell-level toast presentation is currently implemented.

#### Home

- `HomePage` still renders launcher-style cards for product, transitional reporting, and internal workspaces.
- Compact operational metrics, queue-first work, and the quieter hierarchy explorer are not yet the main home posture.

#### Organization

- `organization.rs` still centers the route family around list/detail/create/edit flows with heavier route-local structure than the approved explorer-first model.
- The current hierarchy presentation still uses route-local cards/disclosure styling rather than the lighter explorer-plus-detail pattern from the mockups.

#### Forms / builder

- `forms.rs` still renders the draft workspace around dense record-card structures rather than stacked section panels with section-level settings.
- The current section model in `crates/tessara-api/src/forms.rs` appears to support title and position, but not section description or column-count settings.

#### Workflows and responses

- Workflow surfaces are native, but still visually read as conventional management pages rather than the calmer directory-first pattern from the mockups.
- Delegation context is still primarily route-local in Responses rather than owned by the shared shell.

#### Tests

- `end2end/tests/app.spec.ts` still encodes the old shell shape, top-bar sign-out control, transitional reporting presence, and older home expectations.

## Sprint Goals

### Primary goals

- Bring the authenticated shell into line with `ui-guidance.md` and `ui-guidance-spec.md`.
- Replace the current launcher-style home with an operational home that foregrounds queue work and hierarchy context.
- Rework Organization into the approved explorer-first pattern without inventing a second shell or role-specific UI mode.
- Rework form authoring toward stacked section panels and section-level controls.
- Move acted-as context, scope context, and theme controls into the shared sidebar footer.
- Replace in-page unauthorized dead ends with redirect plus transient feedback.
- Align responsive behavior with the approved desktop, tablet, and mobile shell contract.
- Leave the repo with a verification story and issue decomposition that can be executed like a normal sprint backlog.

### Non-goals

- Do not add roadmap features that are not already implied by the approved UI direction.
- Do not redesign dataset, component, or dashboard feature scope beyond the shell and route posture required for navigation consistency.
- Do not reopen the hybrid-shell migration problem for untouched legacy routes beyond what is necessary for the shared shell contract.

## UI Contract For This Sprint

The sprint must satisfy the following approved UI contract.

### Shell

- One shared authenticated shell across product and internal surfaces.
- One shared user space governed by permissions, not role names.
- Primary sidebar order:
  - `Home`
  - `Organization`
  - `Forms`
  - `Workflows`
  - `Responses`
  - `Components`
  - `Dashboards`
- Secondary `Admin` group:
  - `Datasets`
  - `Administration`
  - `Migration`
- `Admin` appears only when the user has `admin:all`.
- `Reports` does not appear in the default sidebar.
- The top bar contains only mobile nav, global search, notifications, and help.
- Notifications render as a bell-style control.
- Account, acted-as user, scope, and theme controls live in the sidebar footer.
- Theme choices are `System`, `Light`, and `Dark`.

### Sign-in and access behavior

- Sign-in remains outside the authenticated shell.
- Sign-in is bare and auth-only.
- Unauthorized deep links redirect to `/app`.
- The redirect surfaces access-denied feedback through a transient shell notification.

### Home

- Home is queue-first.
- Metrics are compact text, not banner cards.
- Hierarchy context is secondary but visible.
- Sidebar destinations are not repeated as large launcher cards in the page body.

### Organization

- `Organization` remains the sidebar destination.
- The page title is scope-aware.
- Desktop and tablet use a quiet explorer-plus-selected-node-detail layout.
- Mobile uses a tree-plus-detail-sheet pattern.
- Related work leads the selected-node panel.

### Forms / builder

- Builder keeps the canvas primary.
- Sections appear as vertically stacked panels.
- Section-level settings are visible at the section container.
- Deeper field settings stay in a selection-driven properties panel or drawer.

## Workstreams

### 1. Shared shell refactor

- rewrite navigation groups and order
- remove default-shell `Reports`
- move account and theme out of the top bar
- create the footer context block
- align responsive rail behavior to the approved shell model

### 2. Auth and access-boundary refactor

- replace the transitional sign-in surface with the bare auth screen
- add redirect-plus-toast behavior for unauthorized deep links
- centralize shell-level feedback behavior

### 3. Operational home migration

- replace launcher cards with assignment queue, compact metrics, and hierarchy context
- make hierarchy presentation use the same quieter explorer language as Organization

### 4. Organization explorer migration

- move the route experience from list/detail posture to explorer-plus-detail posture
- preserve current data access and scope logic
- reduce chrome and card density

### 5. Builder migration

- restyle and restructure the draft workspace around section panels
- expose section-level settings in-canvas
- add missing section metadata support if required by the API

### 6. Workflow and response alignment

- keep the existing route ownership
- align workflows to directory-first presentation
- reduce route-local delegation redundancy in favor of shell-level context

### 7. Regression and closeout updates

- rewrite browser and Rust-facing UI expectations to the new shell contract
- preserve SSR, hydration, and responsive coverage on touched routes

## Backend And Contract Gaps That May Need To Be Pulled Into The Sprint

These are not new roadmap features. They are enabling work that may be required to make the approved UI honest rather than decorative.

### Section settings support for the builder

The approved builder mockups assume section-level settings such as:

- title
- description
- number of columns

Current API code in `crates/tessara-api/src/forms.rs` visibly validates section title, but does not currently show section description or column-count support. If the current backend truly lacks those fields, this sprint needs a narrow supporting API/data-model slice so the UI does not fake unsupported controls.

### Shared shell context projection

The approved shell expects the footer to show:

- current signed-in account
- acted-as user when delegation is active
- visible top-level scope roots

If the existing account context payload does not cleanly express the exact footer model needed by the guidance, this sprint should extend the existing authenticated account/session projection rather than reconstructing that context independently on each route.

### Shared notification primitive

The access-denied redirect requirement and the UI spec's state-feedback expectations imply a small shared shell notification or toast mechanism. This sprint should introduce that primitive once and use it consistently for access-denied feedback.

## Acceptance Criteria

### Shell acceptance

- The authenticated shell uses the approved navigation order and grouping.
- `Reports` is absent from the default sidebar.
- `Admin` appears only for users with `admin:all`.
- The top bar contains only search, bell, help, and mobile nav.
- The sidebar footer owns account, acted-as, scope, and theme affordances.
- No touched route requires shell-level horizontal scrolling.

### Sign-in and access acceptance

- `/app/login` renders without the authenticated shell chrome.
- Sign-in does not include post-auth promotional or operational content.
- Unauthorized deep links redirect to `/app`.
- The redirect surfaces transient access-denied feedback.

### Home acceptance

- `/app` reads as an operational home rather than a destination launcher.
- The queue or current work surface is visually primary.
- Home metrics are compact and glanceable.
- Home hierarchy context uses the quieter explorer treatment.

### Organization acceptance

- `/app/organization` uses a scope-aware explorer title.
- Desktop and tablet present explorer plus selected-node detail.
- Mobile presents a tree selector plus lower detail region.
- The selected-node panel leads with related work.

### Builder acceptance

- `/app/forms/{id}/edit` presents stacked section panels.
- Section-level settings are visible on each section container.
- Field-level controls are calmer and subordinate to page-level lifecycle actions.
- If section description and column count are surfaced, they persist through the real API contract.

### Workflow and response acceptance

- `/app/workflows` reads as a directory-first workflow surface.
- `/app/workflows/assignments` still works as the assignment-management surface.
- `/app/responses` still supports delegated work, with shared-shell context replacing redundant route-local emphasis where possible.

### Verification acceptance

- `cargo test -p tessara-web` passes.
- `cargo test -p tessara-api` passes.
- Playwright coverage passes with the new shell and route expectations.
- `.\scripts\smoke.ps1`, `.\scripts\local-launch.ps1`, and `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` remain valid closeout gates for the touched UI.

## Manual Verification Matrix

- Sign in as an admin and verify the full shell contract, including the secondary `Admin` group.
- Sign in as a non-admin and verify the shell reflows cleanly with the `Admin` group absent.
- Deep-link to an unauthorized route and verify redirect plus access-denied feedback.
- Open `/app` and verify queue-first home behavior.
- Open `/app/organization` on desktop, tablet, and mobile widths and verify the approved explorer patterns.
- Open `/app/forms/{id}/edit` and verify stacked section panels plus section-level settings.
- Open `/app/workflows` and `/app/workflows/assignments` and verify directory-first workflow posture.
- Open `/app/responses` as a delegated account and verify the shared footer reflects the acting context.

## Automation And Test Impact

The following areas will need coordinated updates as part of this sprint:

- `end2end/tests/app.spec.ts`
- any route-local Rust tests that assert current shell wording or control placement
- `scripts/uat-sprint.ps1` if it encodes old headings or shell assertions
- `scripts/smoke.ps1` if it relies on old login or shell behaviors

## Execution Order

1. Land the shared shell contract, including nav order, top bar, footer context, and responsive sidebar behavior.
2. Land the sign-in boundary and unauthorized redirect-plus-toast behavior.
3. Migrate Home to the operational queue-first surface.
4. Migrate Organization to the approved explorer-first model.
5. Add any missing builder section metadata support needed by the approved UI.
6. Migrate the builder UI to stacked section panels.
7. Align Workflows and Responses to the new shell posture.
8. Rewrite test expectations and rerun the full closeout verification stack.

## GitHub-Ready Decomposition

The detailed issue breakdown for this sprint is maintained in:

- [out-of-roadmap-ui-migration-github-ready.md](./out-of-roadmap-ui-migration-github-ready.md)

That issue list uses the same `Labels`, `Depends on`, `Milestone`, `Description`, `Work`, and `Acceptance criteria` format already used in [tessara-roadmap-backlog-github-ready-described.md](../tessara-roadmap-backlog-github-ready-described.md).

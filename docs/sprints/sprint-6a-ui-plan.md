# Sprint 6A-UI Plan: Application UI And UX Coherence Slice

Kickoff status: started from clean `main` at `c37153b19787d4164eaccbb4752980772e6ec84a` on 2026-07-15. That commit is the post-closeout sequencing commit immediately after closed Sprint 6A commit `f145e059fc1f4d81c960cb35e586c802831ecea2`.

Kickoff defaults:

- Branch: `codex/sprint-6a-ui`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6a-ui`
- Plan artifact: `docs/sprints/sprint-6a-ui-plan.md`
- Baseline inventory: `docs/sprints/sprint-6a-ui-baseline-inventory.md`
- Test change log: `docs/sprints/sprint-6a-ui-test-change-log.md`
- Roadmap source: the sole heading marked `(Next)`, `Sprint 6A-UI: Application UI And UX Coherence Slice`
- Initial implementation state: source-backed baseline inventory started; visual audit, issue prioritization, UI code, and mockups are held at Decision Gate 0.

## Sprint Summary

Sprint 6A-UI makes the current Tessara application visually coherent, accessible, responsive, and easier to use through shared native Leptos patterns. It covers the authenticated shell, the bare sign-in surface, all current Core and in-process product route families, Administration, role/access management, and Module Management.

This sprint is presentation and interaction work over the closed behavior boundary. It does not reopen or amend Sprint 6A, regenerate its acceptance or rollback artifacts, or begin Sprint 6B's Module Release/Instance persistence, materialization, gateway, Supervisor, or runtime work. Sprint 6B follows with its scope unchanged.

The sprint delivers:

- a frozen route/state/viewport audit with representative before-state evidence;
- approved application information architecture and visual direction;
- a coherent token layer and shared native Leptos primitives;
- a clearer shell and consistent page framework;
- standardized tables, forms, actions, overlays, feedback, and route states;
- improved hierarchy and ergonomics for the highest-frequency product and administration workflows;
- keyboard, focus, contrast, reduced-motion, touch-target, overflow, and responsive remediation;
- durable semantic, accessibility, viewport, and narrowly reviewed visual-regression proof; and
- unchanged functional behavior proven by the exact Sprint 6A browser inventory against one final fresh release build.

## Decision Gate 0: Product Direction Before Visual Work

No visual audit conclusions, information-architecture changes, production UI edits, or mockups proceed until these three decisions are recorded. The source inventory may continue because it does not choose a design direction.

| Decision | Current safe contract | Recommended decision | Effect after approval |
| --- | --- | --- | --- |
| Information architecture | The shipped Sprint 6A groups, fixed Core items, and reorder bands remain unchanged throughout Sprint 6A-UI. | Keep the implemented audit direction and sprint mockups inside the current groups/bands. If useful, permit one clearly separated future-slice concept that cannot enter this sprint without a roadmap amendment. | The approved brief states whether to omit IA alternatives entirely or include a separately labeled future-slice concept; neither choice changes Sprint 6A-UI navigation behavior. |
| Visual direction | `docs/ui-guidance.md` supplies the current Tessara identity, palette, typography, density, and native component posture. A rebrand is not implied by the roadmap. | Make a substantial redesign using the existing Tessara identity rather than introducing a new brand. | The visual brief fixes the degree of change and whether existing brand assets/tokens are refined or replaced. |
| Coverage depth | Every primary route receives a coherent responsive baseline; the sprint still needs a bounded list of surfaces that receive deep workflow redesign. | Apply a coherent baseline to all 48 mounted route patterns, with deeper work on the shell, Home, Administration, Module Management, and the highest-frequency end-to-end workflows instead of bespoke deep redesign of every screen. | The audit freezes the deep-workflow list and allocates remaining routes to the shared-pattern baseline. |

Navigation regrouping is deferred by the roadmap. Implementing it requires a roadmap amendment or later slice, not merely a change to this plan. Any other decision that expands the roadmap boundary—for example, changing a route or adding product behavior—also requires an explicit scope amendment before implementation and a reassessment of validation.

## Behavior-Preservation Contract

The following boundary is immutable unless the product owner approves a specific change and this plan plus the test-change log record it before code or test updates.

| Area | Allowed Sprint 6A-UI change | Behavior that remains unchanged |
| --- | --- | --- |
| Product workflows | Layout, visual hierarchy, labeling clarity, focus handling, responsive presentation, and interaction ergonomics | Supported inputs, outputs, state transitions, persisted effects, lifecycle rules, validation meaning, stable errors, and action availability |
| Routes and documents | Native page composition and semantic markup inside the existing route set | Route paths, direct-load/refresh ownership, guards, redirects, SSR usefulness, and no-JavaScript behavior |
| APIs and persistence | None by default | Paths, methods, payload meaning, error envelopes, database schemas, migrations, identifiers, transactions, and stored data |
| Authorization | Visibility and clarity of already-authorized actions and states | Capabilities, implications, scope modes, ownership/delegation rules, route/API authorization, redaction, and nondisclosure |
| Module contracts | Presentation of existing inventory, findings, declarations, capabilities, and policy | Wire types, descriptor content, lifecycle semantics, semantic destinations, typed references, navigation authorization, and absence of real Release/Instance persistence |
| Navigation | Legibility, active/hover/focus treatment, responsive shell behavior, and account/context presentation | Current groups, fixed Core destinations, default relative order, capability eligibility, existing reorder bands, and display-policy/authorization separation |
| Tests | Add stronger semantic, accessibility, viewport, and reviewed visual proof | Existing supported behavior and the exact accepted 60-test identity inventory |

The inherited navigation bands are:

- Forms, Workflows, and Responses: `main_between_organization_and_operations`
- Components and Dashboards: `main_after_operations`
- Datasets: `admin_between_administration_and_module_management`

Core Home, Organization, Operations, Administration, and Module Management anchors remain policy-immutable. Migration remains retired and receives no route or navigation item.

## Frozen Baseline And Audit Contract

The kickoff source baseline is recorded in [the baseline inventory](./sprint-6a-ui-baseline-inventory.md). Before production UI implementation begins, the audit must add and freeze:

1. representative desktop, tablet, and mobile before-state captures for every route family;
2. normal, loading, empty/no-results, validation, error, forbidden, unavailable/degraded, and destructive states wherever the application can currently produce them;
3. keyboard order, focus visibility/restoration, landmarks, labels, live-region, contrast, reduced-motion, touch-target, overflow, SSR, hydration, and console observations;
4. a prioritized issue matrix whose rows contain a stable ID, evidence link, route/pattern, persona, viewport/input, severity, behavior risk, proposed shared owner, and acceptance proof; and
5. the approved visual brief, IA constraint, deep-workflow list, representative keyboard flows, viewport minima, and small reviewed visual-baseline set.

Audit evidence describes the current application; it does not turn accidental behavior into a requirement. A suspected functional defect is separated from visual debt and routed to a product decision before this sprint changes behavior.

## Test Evidence And Change Control

Tests are durable executable contracts, not implementation debris. The kickoff browser boundary is `end2end/acceptance-manifest.json`, schema version 2, containing 60 exact tests across seven files with SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`.

Rules:

1. A failing existing test is presumed to expose a production regression. Fix production code first.
2. Do not delete, skip, rename, filter, weaken, widen tolerances, loosen semantic assertions, add retries, increase timeouts, or regenerate expected output merely to obtain green.
3. Every edit to an existing test is recorded in [the Sprint 6A-UI test change log](./sprint-6a-ui-test-change-log.md), including selector-only changes. The row must cite the approved requirement, show why the old assertion is no longer correct or stable, and identify equal-or-stronger replacement proof.
4. A behavior expectation changes only after an explicit product decision is recorded in this plan. Cosmetic markup changes do not authorize expectation changes.
5. New tests must assert user-visible semantics and stable accessibility contracts. Route-local CSS classes, incidental DOM depth, and implementation-only timing are not acceptable behavioral contracts.
6. Visual baselines are narrow and reviewed individually. Broad screenshots, bulk updates, auto-accept commands, and unexplained pixel churn are prohibited.
7. Characterization coverage lands before changing an under-specified shared pattern. New expected behavior uses red/green proof where practical.
8. Targeted checks during development are diagnostic, not closeout proof. The unchanged complete manifest runs once against the final clean release build after targeted work is green.
9. Closeout reconciles the test-change log against the Git diff and reports zero unexplained edits, skips, retries, filters, or weakened expectations.

Sprint 6A's closed migration, populated-upgrade, rollback-package, upgraded-deployment, and nondisclosure artifacts remain historical proof and are not overwritten. They are rerun only if an explicitly approved scope change touches persistence, migration, APIs, authorization, or module contracts.

## Sprint Specifications

### Surface Inventory And Prioritization

- Treat the live `crates/tessara-web/src/routes` tree as route authority. It currently mounts 48 route patterns across Home/session, Organization, Forms, Workflows, Responses, Operations, Datasets, Components, Dashboards, and Administration/Module Management.
- Treat `docs/ui-screen-inventory.md` as historical migration context, not current route authority.
- Cover list, detail, create, edit/author, history/revision, viewer/preview, access-management, and route-state families.
- Preserve two connected seeded product chains as parity anchors: Form -> generated Workflow -> assignment -> Response, and Dataset -> Component -> Dashboard.
- Freeze deep-workflow priority only after Decision Gate 0; every non-deep route still receives the shared shell/pattern/accessibility/responsive baseline.

### Tokens And Native Primitive Ownership

- Consolidate named typography, semantic color, spacing, density, radius, elevation, border, focus, motion, breakpoint, content-width, and z-index tokens in the shared styling layer.
- Retain the guidance defaults unless the approved visual brief changes them: Inter body, DM Sans headings, JetBrains Mono structured code; 8px spacing rhythm; border-first low-shadow surfaces; named motion; and breakpoints at 640/768/1024/1280/1536px.
- Prefer shared policy-neutral primitives in `crates/tessara-web-ui`; keep Core shell presentation under `crates/tessara-web/src/ui` and navigation/session/theme policy or state under the existing `crates/tessara-web/src/state` and feature owners.
- Evaluate existing native Rust/UI components before adding a custom primitive. Record why a new custom component is needed when the existing component cannot satisfy the contract.
- Remove duplicate route-local pattern styling as routes migrate, without combining unrelated domain DTOs or product policy into the UI crate.
- Preserve Tailwind 4 and native Leptos `view!` ownership. Do not add HTML-string rendering, `inner_html`, `/bridge/*`, retained JavaScript controllers, or broad compatibility shells.

### Shell, Navigation, And Page Framework

- Use one authenticated `AppShell`; `/login` remains a bare route outside it.
- Establish consistent shell hierarchy, active/hover/focus navigation states, section legibility, page orientation, content widths, account identity, delegation/acting context, scope-root context, and theme controls.
- Preserve semantic destination resolution and separate display policy from authorization.
- Provide desktop expanded, tablet collapsed, and mobile overlay behavior with no clipped primary action or unreachable navigation.
- Use one route-level `h1`, consistent `PageHeader` structure, restrained breadcrumbs only for genuine hierarchy, and page-local actions in the page header rather than global chrome.
- Keep administration visually distinct through a restrained cue, not a separate theme.

### Shared Interaction Patterns

- Standardize buttons and icon buttons, fields and field groups, select/combobox patterns, validation feedback, menus, tabs, notices, status badges, dialogs, side sheets, and destructive confirmation.
- Standardize table headers, row actions, search, filters, column controls, pagination, responsive alternatives, empty/no-result distinctions, loading skeletons, and server-backed state.
- Standardize normal, loading, empty, no-results, validation, success, error, forbidden, disabled, unconfigured, unavailable, incompatible, and degraded presentation without collapsing distinct system states.
- Preserve the existing action, request, mutation, and error semantics behind each pattern.
- Use live regions only for meaningful asynchronous feedback; restore focus after overlays, destructive cancellation/completion, row removal, and route-owned state transitions.

### Highest-Frequency Workflows

Subject to the approved depth decision, deep work should cover:

- shell entry, orientation, session/account context, and Home work discovery;
- Organization browse/detail/create/edit orientation;
- Form authoring, publish, generated-workflow assignment, Response start/draft/submit/review;
- Dataset authoring/revision/preview, Component authoring/publish/view, Dashboard composition/view;
- Administration user/access/role flows; and
- Module Management directory/detail and read-only versus navigation-manager policy states.

Deep redesign may change interaction presentation but not the workflow's inputs, outcomes, state machine, authorization, or persistence.

### Accessibility And Responsive Contract

- Every touched pattern has semantic labels, landmarks, heading order, visible focus, logical keyboard order, and a keyboard-operable equivalent for pointer actions.
- Representative flows complete keyboard-only without focus traps or lost focus.
- Text, controls, status, and focus indicators meet the project's adopted WCAG contrast target; color is never the only state signal.
- `prefers-reduced-motion` removes nonessential transitions; route navigation remains instant.
- Touch targets, overlays, tables, builders, and page actions remain usable below 768px; tablet and desktop behavior follow the documented breakpoint model.
- Horizontal overflow is contained to intentionally scrollable data regions and never hides the route title, primary action, validation, or overlay close control.

### Rendering And Delivery Contract

- Preserve useful native SSR documents, direct loads, refreshes, and JavaScript-disabled route information.
- Hydration reuses server state without mismatch, duplicate initial mutation, or visual/semantic divergence.
- Browser consoles remain clean of application errors and hydration warnings.
- CSS and font delivery are production-build assets with no development-only dependency.
- Shared visual changes must not increase route ownership coupling or move product policy into Core shell components.

### Explicit Non-Goals

- No route additions/removals, API or DTO changes, schema or migration work, seed-contract changes, authorization changes, or product lifecycle changes.
- No Module Release/Instance persistence, mutation, materialization, Supervisor, gateway, OCI, module database, or runtime work.
- No restoration of Migration or Reports.
- No command palette, keyboard shortcut system, or broad dashboard product redesign beyond presentation of the current workflow.
- No rebrand unless Decision Gate 0 explicitly expands scope.

## Acceptance Criteria

1. The baseline inventory contains all 48 mounted route patterns, representative before-state evidence, and a frozen prioritized issue matrix before production UI changes begin.
2. The approved visual/IA/depth brief is recorded and every implementation issue maps to it or to existing UI guidance.
3. Shared tokens and primitives cover the required shell, page, table, form, overlay, action, feedback, status, focus, motion, and responsive patterns without route-local forks for equivalent behavior.
4. Every primary route family uses coherent page hierarchy, action placement, state presentation, and desktop/mobile framing.
5. The approved deep workflows complete with their existing inputs, outputs, transitions, persisted effects, permissions, and stable errors.
6. The shell preserves exact current destination eligibility, fixed items, groups, bands, direct-route authorization, and display-policy separation.
7. Module Management remains visible/read-only for effective global `modules:read`, mutable only for effective global `modules:manage_navigation`, and separate from `admin:all`-only Administration.
8. Representative normal, loading, empty/no-results, validation, error, forbidden, unavailable/degraded, and destructive states are distinguishable and actionable without disclosing restricted data.
9. Representative flows pass keyboard-only, focus restoration, semantic landmark/label, contrast, reduced-motion, touch-target, overflow, tablet, and mobile checks.
10. Direct loads, refreshes, no-JavaScript documents, SSR bootstrap, hydration, and browser consoles remain clean and useful; no `/bridge/*`, HTML-string route UI, or JavaScript controller ownership appears.
11. The exact 60-test schema-v2 browser inventory passes unchanged against one final fresh release build with zero skips, retries, filters, unexpected failures, or unexplained test edits.
12. Fresh-deployment smoke and UAT, targeted source/unit checks, web boundary checks, accessibility/viewport tests, and reviewed narrow visual baselines pass against the same clean closing commit.
13. Sprint 6A artifacts remain byte-for-byte untouched, Sprint 6B scope remains unchanged, and the test-change log reconciles exactly with the final diff.

## Manual Test Plan

### Personas

- Seeded administrator with `admin:all`.
- Global `modules:read` actor without `admin:all`.
- Global `modules:manage_navigation` actor.
- Scoped product manager/operator.
- Respondent/owner or active delegator.
- Authenticated no-capability actor and unauthenticated visitor.

### Viewports And Inputs

- Desktop: at least 1440px wide, keyboard and pointer.
- Tablet: 768-1023px, keyboard and touch/pointer.
- Mobile: below 768px using the approved minimum width from the design gate.
- Reduced-motion mode, light/dark/system themes, JavaScript enabled, and representative JavaScript-disabled direct loads.

### Required Journeys

1. Sign in, land on Home, inspect assigned work, traverse every eligible navigation group, inspect account/delegation/scope context, change theme, and sign out.
2. Browse Organization, inspect a node, create a permitted child, validate an invalid edit, save a valid edit, and return without losing orientation.
3. Create and publish a Form, use the generated single-form Workflow shortcut, assign it, start as the assignee/delegator, save a Response draft, submit it, and review the read-only result.
4. Create/edit/publish a Dataset revision, preview it, create/publish/view a Component bound to the intended major line, then create/edit/view a Dashboard using an exact Component version.
5. Use Operations with and without its capability and confirm unavailable/error rows remain distinct.
6. As administrator, use users, access, roles, and node types; verify capability provenance and unchanged scope/assignment semantics.
7. As module reader and navigation manager, compare Module Management directory/detail/policy controls; confirm fixed Core items and band limits remain enforced.
8. Repeat representative journeys keyboard-only and at tablet/mobile widths; open and close every touched menu/dialog/sheet and verify focus restoration.
9. Exercise current loading, empty/no-results, validation, forbidden, unavailable/degraded, execution error, and destructive confirmation paths without changing their underlying response meaning.
10. Direct-load and refresh representative routes with JavaScript enabled and disabled; inspect SSR usefulness, hydration, network ownership, and console output.

The audit matrix assigns exact seeded records and expected outcomes before manual execution so a visual review cannot substitute for functional proof.

## Automated Test Plan

### Baseline And Environment Rules

- The closed Sprint 6A 60/60 result is inherited historical proof, not a Sprint 6A-UI execution result.
- Development uses proportional checks for the changed crate/pattern and the directly affected Playwright file(s). A targeted pass does not replace final acceptance.
- Browser dependencies are installed through `npm --prefix .\end2end ci` and `npm --prefix .\end2end run install-browsers`.
- Bare root `npx playwright test` is deliberately unsupported because the repository has no root Playwright dependency/configuration and the command bypasses wrapper environment/evidence rules. Its required kickoff baseline status is `unsupported/not run; no evidence claimed`; record the repository-root working directory and supported replacement rather than installing Playwright at root. Use `npm --prefix .\end2end test` for local diagnostics and `scripts/validate-e2e.ps1` for final acceptance.
- New Sprint 6A-UI browser identities do not enter `end2end/tests`, whose discovery set is frozen by the 60-test manifest. Add them under `end2end/ui-tests` with `end2end/playwright.ui.config.ts`, a dedicated exact-identity `end2end/ui-acceptance-manifest.json`, and `scripts/validate-sprint-6a-ui-e2e.ps1`. That wrapper owns semantic, keyboard, viewport, `@axe-core/playwright`, and narrowly reviewed `toHaveScreenshot` proof. The original and UI suites run independently.
- Final acceptance uses a clean committed release image and one freshly created database. It does not reuse or overwrite `artifacts/sprint-6a/*`.
- Sprint 6A migration/rollback/upgraded-data and nondisclosure gates are excluded while this sprint remains presentation-only.

### Required Command Matrix

| Command requested by kickoff baseline | Sprint 6A-UI use |
| --- | --- |
| `cargo fmt --all` | Run while editing; final proof uses `cargo fmt --all -- --check`. |
| `cargo test -p tessara-api` | Do not run the unqualified target as final proof because it includes Sprint 6A's destructive populated-upgrade integration target. Use `cargo test -p tessara-api --lib --locked` for the parity-preserving source gate and targeted non-migration integration targets only if affected. |
| `cargo test -p tessara-web` | Run as `cargo test -p tessara-web -j 1 --locked`; this is a required final source gate. |
| `npx playwright test` | Unsupported/not run at repository root; no evidence claimed. Record the repository-root cwd and use the package command for diagnostics plus the two manifest-bound wrappers for final evidence. |
| `.\scripts\smoke.ps1` | Run against the final existing fresh release service with new Sprint 6A-UI evidence paths. |
| `.\scripts\local-launch.ps1` | Use during development; final release candidate uses `.\scripts\local-launch.ps1 -FreshData`. |
| `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"` | Retained as the literal kickoff-baseline command, but not used for commit-bound evidence because the deployment record binds the exact host. The final run uses `http://127.0.0.1:8080`, matching capture, smoke, and both browser suites. |

### Proportional Development Gates

- Documentation-only kickoff: `git diff --check`, link/path checks, plan-structure checks, and clean status after commit.
- Shared CSS/primitives: formatting, `cargo check/test` for `tessara-web-ui` and `tessara-web`, hydrate check, relevant semantic/accessibility/viewport Playwright specs, and representative browser inspection.
- Shell/navigation: web tests plus `app.spec.ts`, `modules.spec.ts`, and affected permission scenarios.
- Product route family: owning feature/web crate tests plus its existing Playwright spec and connected parity journey.
- API is untouched by default. If a UI change appears to require API/persistence/auth/contract work, stop and amend scope rather than quietly widening validation.

### Final Ordered Gates

Use a two-pass closeout so the retained deployment proof binds the actual final commit rather than a pre-documentation candidate.

1. **Candidate source and dependency quality:** `cargo fmt --all -- --check`; `.\scripts\validate.ps1 -Fast`; `cargo check --workspace --all-features --locked`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown --locked`; `cargo test --workspace --all-features --locked --exclude tessara-api`; `cargo test -p tessara-web -j 1 --locked`; `cargo test -p tessara-api --lib --locked`; `.\scripts\check-web-crate-boundaries.ps1`; `cargo audit --quiet`; `git diff --check`.
2. **Candidate frontend and deployed validation:** `npm ci`; `npm run tailwind:build`; install the locked browser package; run the source gates, fresh deployment, smoke, UAT, original 60-test wrapper, separate UI wrapper, and manual audit as a provisional release pass. Correct defects before closeout documentation is frozen.
3. **Closeout commit:** reconcile the issue matrix and test-change log; record provisional results and the exact canonical commands in roadmap/progress/closeout documentation; commit every tracked implementation, test, baseline, and documentation change. Require a clean worktree. The canonical evidence pass begins only after this commit.
4. **Canonical source gate:** rerun Gate 1 against the clean final commit. Any tracked fix invalidates the candidate and returns to Gate 1.
5. **Canonical fresh release deployment:** run `npm --prefix .\end2end ci`; `npm --prefix .\end2end run install-browsers`; `.\scripts\local-launch.ps1 -FreshData`; then:

   ```powershell
   $fresh = 'artifacts/sprint-6a-ui/deployment-fresh.json'
   .\scripts\capture-sprint-6a-deployment-evidence.ps1 `
       -BaseUrl 'http://127.0.0.1:8080' `
       -ExpectedDataState fresh `
       -OutputPath $fresh
   ```

6. **Canonical fresh smoke, UAT, and unchanged browser inventory:** run without overwrite flags on first publication:

   ```powershell
   .\scripts\smoke.ps1 `
       -UseExistingService `
       -BaseUrl 'http://127.0.0.1:8080' `
       -KeepServices `
       -DeploymentEvidencePath $fresh `
       -ExpectedDataState fresh `
       -AcceptanceEvidencePath 'artifacts/sprint-6a-ui/smoke-fresh.json'

   .\scripts\uat-sprint.ps1 `
       -BaseUrl 'http://127.0.0.1:8080' `
       -DeploymentEvidencePath $fresh `
       -ExpectedDataState fresh `
       -AcceptanceEvidencePath 'artifacts/sprint-6a-ui/uat-fresh.json'

   .\scripts\validate-e2e.ps1 `
       -BaseUrl 'http://127.0.0.1:8080' `
       -DeploymentEvidencePath $fresh `
       -ExpectedDataState fresh `
       -EvidencePath 'artifacts/sprint-6a-ui/playwright-acceptance-fresh.json'
   ```

7. **Canonical Sprint 6A-UI proof and manual review:** run the separate exact-identity suite and execute the frozen manual journeys against the same release:

   ```powershell
   .\scripts\validate-sprint-6a-ui-e2e.ps1 `
       -BaseUrl 'http://127.0.0.1:8080' `
       -DeploymentEvidencePath $fresh `
       -ExpectedDataState fresh `
       -AcceptanceManifestPath 'end2end/ui-acceptance-manifest.json' `
       -EvidencePath 'artifacts/sprint-6a-ui/ui-acceptance-fresh.json'
   ```

   Retain reviewed narrow visual results and record SSR/hydration/console, keyboard, automated accessibility, reduced-motion, and responsive results in ignored commit-bound evidence. No tracked file changes are allowed after this gate. If a tracked change is required, return to Gate 1 and repeat the canonical pass after a new final commit.
8. **Final cleanliness:** require `git status --short` to emit no lines and verify no file under `artifacts/sprint-6a/` changed. Report canonical results without amending the tracked closing tree.

### Acceptance-To-Proof Matrix

| Acceptance area | Durable proof |
| --- | --- |
| Behavior and authorization parity | Unchanged 60-test manifest, fresh smoke/UAT, existing web/API source tests, test-change-log reconciliation |
| Shell, route orientation, and responsive behavior | Shared web tests, semantic Playwright assertions, desktop/tablet/mobile route matrix |
| Tables, forms, actions, overlays, and feedback | Primitive unit/component tests plus representative route scenarios |
| Accessibility | Semantic DOM assertions, keyboard/focus scenarios, automated accessibility checks, contrast review, reduced-motion and touch/overflow cases |
| SSR, direct load, hydration, no-JS, console | Existing acceptance scenarios plus representative new route-state assertions |
| Visual coherence | Small approved cross-pattern baseline set reviewed individually; no broad snapshot regeneration |
| Closed-sprint isolation | No changes under `artifacts/sprint-6a/`; no migration/rollback/upgraded-data rerun; Git diff inspection |

## Ordered Implementation Plan

0. **Kickoff and decision gate.** Create the branch/worktree, plan, baseline inventory, and test-change log from clean `main`; record source/route/test facts; obtain the three product decisions. No visual conclusions or UI code before approval.
1. **Freeze audit and visual brief.** Capture representative before states, conduct the route/pattern accessibility and responsive audit, freeze the prioritized issue matrix, approve IA constraints, visual direction, deep-workflow list, viewport minimum, keyboard journeys, and narrow visual-baseline set. Reinforce this plan with exact issue IDs and evidence commands.
2. **Characterize and consolidate foundations.** Add missing characterization proof for shared patterns, then establish tokens and policy-neutral primitives. Prove each primitive's semantic, focus, responsive, and state contract before migrating routes.
3. **Shell and page framework.** Implement authenticated shell, navigation, top bar, page header/content widths, account/context, responsive shell, and bare sign-in improvements. Land shell-specific parity, SSR, accessibility, and viewport proof in the same change.
4. **Shared data-entry and state patterns.** Migrate tables, search/filter/pagination, forms, validation, actions, menus, dialogs/sheets, notices, loading/empty/error/restricted/unavailable/degraded states, and destructive confirmation through shared primitives.
5. **Approved deep workflows.** Improve the selected Home, product, Administration, and Module Management journeys in vertical slices. Each slice retains its existing functional acceptance and adds semantic/accessibility/responsive proof before moving on.
6. **All-route coherence baseline.** Apply the approved shared framework to every remaining primary route and reconcile the 48-route audit without creating route-local pattern forks.
7. **Cross-cutting hardening.** Close keyboard/focus, contrast, reduced-motion, overflow, touch-target, SSR/hydration/console, no-JS, and narrow visual-regression issues; freeze the final issue disposition.
8. **Final fresh release validation and closeout.** Run the ordered gates once, fix production regressions without weakening proof, reconcile every test edit, update roadmap/progress/sprint index, and retain reviewer-ready Sprint 6A-UI evidence without altering Sprint 6A artifacts.

## Risk, Abort, And Rollback Matrix

| Risk | Preventive proof | Abort/rollback response |
| --- | --- | --- |
| Functional or authorization regression | Existing semantic tests and vertical-slice parity runs | Revert the offending UI slice or fix production code; do not edit the failing expectation to fit the regression. |
| CSS blast radius | Shared token/primitive slices and representative route matrix | Revert or narrow the shared selector; avoid route-specific compensating overrides. |
| SSR/hydration/no-JS regression | Native/hydrate checks and direct-load scenarios per shared pattern | Stop rollout of the pattern and restore native server ownership before continuing. |
| Accessibility regression | Keyboard/focus/semantic/viewport proof in each slice | Treat as a production defect and block migration of additional routes. |
| Brittle visual snapshots | Small approved baseline set and individual review | Reject bulk regeneration; remove incidental snapshots or redesign them around stable pattern states. |
| Test weakening disguised as maintenance | Mandatory log reconciliation and unchanged manifest identity | Reject the test edit until its approved rationale and stronger replacement proof exist. |
| Hidden API, persistence, auth, or contract need | Diff boundary checks and product decision gate | Stop, request an explicit scope decision, and reassess migration/rollback/full API validation before proceeding. |
| Scope too broad for a clean pass | Baseline-for-all plus approved deep-workflow list | Preserve shared baseline coverage and defer bespoke depth with explicit issue disposition rather than shipping inconsistent half-redesigns. |

## Dependencies And Blockers

- Decision Gate 0 is the only product blocker at kickoff. Source inventory and documentation may proceed; visual audit conclusions, plan reinforcement, mockups, and production UI changes wait for the three decisions.
- The live application and seeded fixtures must be available to capture real before states and route-state variants.
- Existing `docs/ui-guidance.md` and `docs/ui-guidance-spec.md` are the starting human/behavioral authorities. Any approved visual departure must update them before implementation relies on it.
- Shared primitives must remain policy-neutral and compatible with the current extracted feature crates; Core shell policy stays in `tessara-web`.
- Final Playwright proof depends on installed locked `end2end` dependencies, Playwright browsers, a clean current-commit release image, and a fresh labeled database.
- The current full `.\scripts\validate.ps1` is Sprint-6A-specific and intentionally invokes destructive upgrade/fresh database proof. Sprint 6A-UI uses `-Fast` plus scoped source gates unless this sprint explicitly introduces a generalized non-migration closeout wrapper.
- No dedicated automated accessibility or visual-regression wrapper exists at kickoff. Ordered step 1 must add and self-test the named `end2end/ui-tests`, `playwright.ui.config.ts`, UI manifest, and `validate-sprint-6a-ui-e2e.ps1` boundary before production UI migration. New identities, accessibility scans, viewport cases, and reviewed screenshots stay isolated there so the closed 60-test manifest remains independently verifiable.
- No current technical blocker is known after the three product decisions are resolved.

# Sprint 6A-UI Plan: Sprint 6A Surface Harmonization Slice

Kickoff status: started from clean `main` at `c37153b19787d4164eaccbb4752980772e6ec84a` on 2026-07-15. That post-closeout sequencing commit follows the closed Sprint 6A commit `f145e059fc1f4d81c960cb35e586c802831ecea2`.

- Branch: `codex/sprint-6a-ui`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6a-ui`
- Baseline inventory: `docs/sprints/sprint-6a-ui-baseline-inventory.md`
- Test change log: `docs/sprints/sprint-6a-ui-test-change-log.md`
- Roadmap source: `Sprint 6A-UI: Sprint 6A Surface Harmonization Slice (Next)`
- Product direction: approved; no product decision currently blocks design exploration or implementation.

## Sprint Summary

Sprint 6A-UI corrects awkward presentation introduced by Sprint 6A and harmonizes those additions with the existing Tessara application. It is not an application redesign.

Editable UI is limited to:

1. `/administration/modules`;
2. `/administration/modules/:definition_id`;
3. navigation-policy presentation and controls rendered on those routes;
4. the Sprint 6A-added Module Management entry on `/administration`; and
5. Sprint 6A-added capability-provenance presentation in role/access administration.

Existing Tessara screens, shell components, and shared patterns are references and regression surfaces only. The 48 mounted route patterns remain useful as parity inventory, not as visual implementation scope.

This sprint does not reopen or amend Sprint 6A, replace its retained evidence, or begin Sprint 6B persistence, materialization, gateway, Supervisor, database, installation, or runtime work.

## Approved Product Direction

| Question | Approved direction | Implementation consequence |
| --- | --- | --- |
| Information architecture | Keep the shipped navigation groups, fixed Core items, anchors, and existing reorder bands. No regrouping concept is part of this sprint. | Module Management remains a fixed `Admin` item. Contributions reorder only inside their existing bands. No sidebar or mobile-navigation redesign. |
| Visual direction | Harmonize only the newly added Sprint 6A pages and fragments with the current Tessara identity. | Reuse existing typography, colors, spacing, radii, page framing, tables, cards, status treatments, and native Leptos components. No rebrand or replacement design system. |
| Coverage depth | Correct UI introduced by Sprint 6A; do not pursue broad route or workflow redesign. | Home, sign-in, shell, Organization, product routes, general Administration pages, and pre-existing workflows receive no intentional UI change. They remain regression proof. |

The earlier ideas of a future regrouping mockup, a substantial redesign, and a coherent baseline across all 48 routes are explicitly rejected and removed from this sprint.

## Behavior-Preservation Contract

The following boundary is immutable unless the product owner approves a specific change and this plan plus the test change log record it before code or test changes.

| Area | Allowed Sprint 6A-UI change | Must remain unchanged |
| --- | --- | --- |
| Targeted presentation | Layout, hierarchy, density, responsive behavior, semantic markup, labels that clarify existing meaning, focus treatment, and scoped styling | Returned data, supported actions, state transitions, persisted effects, lifecycle meaning, and action availability |
| Routes and documents | Native composition inside the two Module Management pages and the named Sprint 6A fragments | Paths, route ownership, guards, redirects, direct load/refresh, useful SSR/no-JavaScript output, and hydration behavior |
| Shell and navigation | No intentional change | Shell layout/styling, groups, fixed items, default order, contribution bands, capability eligibility, mobile/desktop behavior, and display-policy/authorization separation |
| APIs and persistence | None | Paths, methods, payloads, errors, schemas, migrations, identifiers, transactions, seed behavior, and stored data |
| Authorization | Clearer presentation of already-authorized states and actions | `modules:read`, `modules:manage_navigation`, implication rules, global scope, `admin:all`, mixed-scope exception, route/API authorization, and nondisclosure |
| Module contracts | Clearer presentation of current descriptors, features, findings, statuses, and policy | Wire types/content, transition semantics, retired Migration, semantic destinations, typed references, and absence of real Release/Instance persistence |
| Existing tests | Add focused UI proof; record any justified semantic-selector adjustment | Supported behavior and the exact accepted 60-test identity inventory |

The inherited reorder bands remain:

- Forms, Workflows, and Responses: `main_between_organization_and_operations`;
- Components and Dashboards: `main_after_operations`; and
- Datasets: `admin_between_administration_and_module_management`.

Core Home, Organization, Operations, Administration, and Module Management remain fixed. Migration remains retired and has no live route or navigation item.

## Targeted Audit And Design Contract

Before production UI code changes, retain current-run evidence for:

- the directory header and Core runtime context;
- the seven-row module inventory at a supported desktop width;
- the Forms detail page with representative long descriptor content;
- navigation-policy reader and manager presentation;
- no-access, retired, unavailable, and finding states where available;
- capability provenance and the Administration entry; and
- a small reference set of existing Tessara list/detail/Administration patterns.

The audit records task-impacting defects, not an aesthetic wishlist. Each accepted issue includes evidence, target surface, severity, behavior risk, proposed existing pattern, and proof. [The design exploration](../mockups/sprint-6a-ui/README.md) provides exactly three targeted, screenshot-grounded alternatives. Selection of an option is a product decision before UI implementation; selecting an option does not authorize new behavior.

## Existing Pattern Reuse And Scoped Styling

- Keep `AppShell`, `Breadcrumb`, `PageHeader`, and `route-panel` framing unchanged.
- Prefer existing `DataTable`, `InfoListTable`, `EmptyState`, `organization-detail-card`, `organization-state`, button, status-badge, and form-action patterns when they preserve the current semantics.
- Preserve explicit status variants for Active, Unavailable, and Retired; do not flatten distinct lifecycle meanings into one generic state.
- Use scoped Module Management selectors for overflow, wrapping, responsive layout, and content hierarchy. Do not make global token or shared-selector changes unless the same established pattern demonstrably requires them and regression proof is added.
- Do not add search, sorting, filtering, or pagination to the fixed seven-entry inventory unless separately approved; those would be new interaction scope.
- Long definition IDs, digests, routes, finding paths, and capability IDs must wrap or otherwise remain legible without causing page-level horizontal overflow. Their exact values remain available to users and assistive technology.
- Keep native Leptos SSR/hydration ownership. No HTML-string route shell, `inner_html`, `/bridge/*`, handcrafted JavaScript controller, or parallel frontend is permitted.

## Targeted UX Outcomes

### Module Directory

- The page purpose, runtime context, inventory, and navigation policy have clear hierarchy.
- Core runtime metadata is compact and scannable rather than consuming the useful first viewport.
- The inventory remains semantically tabular on desktop or uses an equally semantic established responsive representation at narrow widths.
- Contribution name, definition ID, digest, transition type, availability, Release/Instance absence, and finding count remain visible without collisions or illegible columns.
- Retired Migration remains clearly distinct from active in-process contributions.

### Module Detail

- Overview, feature declarations, contracts, capabilities, dependencies, resources, routes, findings, and runtime dimensions remain peer sections with their existing content.
- Long values do not overlap adjacent content, controls, or viewport edges.
- The exact source-descriptor action has an unambiguous place and accessible focus treatment.
- Section hierarchy remains understandable on desktop and mobile without hiding or inventing data.

### Navigation Policy

- Read-only and manager modes remain visibly distinct.
- Permanent Core destinations, contributed destinations, placement bands, visibility, order actions, policy revision, save/discard state, and messages remain present.
- Row labels and destination IDs do not run together.
- Keyboard focus remains stable after reorder, save, and discard; disabled actions remain understandable.
- No control suggests that display choices grant route authorization.

### Administration Entry And Capability Provenance

- The Module Management card aligns with neighboring Administration cards.
- Capability origin, global/scope mode, and descriptor provenance remain legible without changing assignments, bundles, or authority.
- Existing role/access workflows receive no broader visual redesign.

## Accessibility And Responsive Requirements

For the targeted surfaces only:

- preserve semantic headings, regions, lists, definition lists, tables, labels, status/alert/live-region behavior, and useful link/button names;
- prove keyboard traversal, visible focus, and focus restoration for policy controls;
- prevent shell- or page-level horizontal overflow at 1280px desktop, 768px tablet, and 390px mobile viewports;
- preserve readable content at 200% zoom where practical for the supported viewport;
- verify text/status contrast in both existing themes and do not rely on color alone;
- preserve reduced-motion behavior and minimum practical touch targets; and
- keep direct load, refresh, no-JavaScript content, hydration, network ownership, and console behavior clean.

This is not a claim of application-wide WCAG conformance.

## Explicit Non-Goals

- No redesign of the authenticated shell, top bar, sidebar, mobile navigation, sign-in, Home, or unrelated routes.
- No navigation grouping, anchor, order, eligibility, or capability-semantic change.
- No application-wide token, design-system, component, accessibility, or visual-baseline overhaul.
- No new route, workflow, filter, search, pagination, bulk action, installation action, lifecycle action, or product behavior.
- No API, DTO, schema, migration, seed, authorization, or module-contract change.
- No Module Release/Instance persistence or mutation, Supervisor, gateway, OCI, module database, materialization, or runtime work.
- No restoration of Migration or Reports and no rebrand.

## Acceptance Criteria

1. The targeted before-state audit and issue matrix are retained before production UI edits.
2. One of three screenshot-grounded harmonization directions is explicitly selected before implementation.
3. Only the named Sprint 6A UI surfaces receive intentional presentation changes; unrelated diffs require a product decision and plan amendment.
4. The directory and detail remain complete, semantically structured, and legible with representative long values at desktop, tablet, and mobile widths.
5. Navigation-policy reader/manager modes preserve all controls, band restrictions, immutable Core items, authorization separation, save/discard behavior, and keyboard focus.
6. Active, unavailable, retired, finding, empty, loading, restricted, not-found, and error states remain semantically distinct wherever currently supported.
7. Module Management remains readable for effective global `modules:read`, mutable only for effective global `modules:manage_navigation`, and independent of the `admin:all`-only Administration destination.
8. The Administration entry and capability-provenance fragments use existing Tessara patterns without changing assignments or authority.
9. Direct load, refresh, no-JavaScript documents, SSR bootstrap, hydration, and browser console behavior remain clean; no `/bridge/*` or JavaScript-owned application UI appears.
10. Targeted Rust/web tests and focused UI accessibility/viewport/visual tests pass for each implementation slice.
11. The exact schema-v2 60-test browser inventory passes unchanged against the final fresh release build with zero skips, retries, filters, unexpected failures, or unexplained edits.
12. Fresh smoke/UAT and final source-quality gates pass against the same closing commit, the test change log exactly reconciles with the diff, and no Sprint 6A retained artifact changes.

## Manual Test Plan

Personas:

- global `modules:read` reader without `admin:all`;
- global `modules:manage_navigation` manager;
- seeded `admin:all` administrator; and
- authenticated no-capability actor.

Journeys:

1. Reader: open the fixed Admin-group Module Management item, scan Core context and all seven contributions, open Forms detail, inspect every declaration/dimension, and return to the directory.
2. Manager: compare the same pages, change visibility and order only within an existing band, verify focus after each move, discard once, save once, refresh, and confirm persistence.
3. No access: direct-load directory and detail and confirm stable restricted presentation without module data disclosure.
4. Administrator: verify the Administration entry and capability provenance remain aligned and understandable while role composition semantics remain unchanged.
5. State review: inspect active and retired entries plus representative finding, empty, unavailable, error, not-found, read-only, dirty, saving, success, and disabled-control states that current fixtures can produce.
6. Repeat representative reader/manager checks with keyboard only and at 1280px, 768px, and 390px in both existing themes.
7. Direct-load and refresh representative routes with JavaScript enabled and disabled; inspect SSR usefulness, hydration, `/bridge/*` absence, and console output.
8. Verify the shell and navigation appearance, grouping, eligibility, and order are unchanged; unrelated workflows rely on the frozen automated regression inventory rather than a manual redesign review.

## Durable Test Policy

The accepted browser baseline is `end2end/acceptance-manifest.json`, schema 2, with 60 exact identities across seven files. Those tests are durable proof of supported behavior.

- A failing existing test is presumed to expose a production regression, unstable environment, or unapproved requirement conflict. It is not permission to edit the test.
- Do not delete, skip, rename, filter, loosen, increase timeouts/retries, replace semantic selectors with incidental structure, or regenerate visual baselines to obtain a pass.
- Every existing-test, fixture, manifest, selector, timeout, screenshot, smoke, or UAT edit requires a pre-approved row in `sprint-6a-ui-test-change-log.md` and equal-or-stronger replacement proof.
- New Sprint 6A-UI tests are limited to the targeted surfaces and remain separate from the frozen 60-test manifest unless the product owner explicitly approves changing that inventory.
- Each tracked visual baseline records its route/state, fixture, persona, viewport, theme, and purpose and is reviewed individually. Bulk regeneration is prohibited.

### Proportional Validation During Iteration

UI iteration does not require the entire closeout battery after every tweak. Run the smallest durable checks that cover the touched surface:

- `directory.rs`, `detail.rs`, or `policy.rs`: relevant Rust/web unit tests, `cargo test -p tessara-web -j 1 --locked`, focused `modules.spec.ts` diagnostics, and the targeted UI viewport/accessibility cases;
- Administration entry or capability provenance: affected web tests plus the exact relevant `app.spec.ts` and permission scenarios;
- scoped CSS only: formatting/diff hygiene, targeted web tests, the affected UI cases, and visual comparison at supported viewports;
- shell, navigation, API, persistence, authorization, or contract diff: stop because it is outside approved scope and request a product decision before widening validation.

The complete 60-test suite, fresh smoke/UAT, and final source gates run once after the UI is stabilized for closeout. If production or tracked proof changes after that canonical final pass, the affected targeted checks and commit-bound final gates must run again; documentation-only corrections are handled according to the closeout evidence contract rather than reflexively repeating destructive Sprint 6A migration proof.

### Focused UI Proof

Keep new UI identities outside `end2end/tests` under `end2end/ui-tests`, with a dedicated config, exact-identity manifest, and validation wrapper. Cover only:

- semantic directory/detail/policy landmarks and names;
- reader, manager, and no-access presentation;
- keyboard focus after policy reorder/save/discard;
- 1280px, 768px, and 390px overflow behavior;
- targeted automated accessibility checks; and
- a small, reviewed screenshot set for stable targeted states.

The original and UI suites run independently so new visual proof cannot weaken the frozen behavior suite.

## Ordered Implementation Plan

1. Freeze the targeted current-run audit, established-Tessara references, issue IDs, and selected design option.
2. Add or confirm characterization proof for current Module Management/provenance behavior without changing existing assertions.
3. Harmonize the directory and Core runtime context using established Tessara patterns; land targeted responsive/semantic proof.
4. Harmonize detail sections and navigation-policy controls; land long-content, keyboard, reader/manager, and viewport proof.
5. Harmonize the Administration entry and capability-provenance fragments only where the audit identifies a Sprint 6A presentation defect.
6. Run targeted accessibility, SSR/no-JavaScript, hydration, console, theme, and responsive hardening; reconcile the issue matrix and every test edit.
7. Stabilize the final release candidate, run the unchanged 60-test inventory plus fresh smoke/UAT and source gates once, then close out without modifying Sprint 6A artifacts.

## Risk And Abort Rules

| Risk | Prevention | Response |
| --- | --- | --- |
| Behavior or authorization regression | Existing semantic tests remain authoritative | Fix or revert production UI; do not weaken proof. |
| CSS blast radius | Reuse existing patterns and scope new selectors to Module Management/provenance | Narrow or revert the selector and run affected regression checks. |
| Long metadata still breaks layout | Representative real digests, routes, IDs, findings, and declarations in each viewport case | Block the slice until overflow and semantic reading order pass. |
| Snapshot churn | Small stable baseline set with individual review | Reject bulk updates and redesign the proof around stable state. |
| Scope creep | Diff review against the explicit editable-surface list | Stop and request a product decision plus plan amendment before proceeding. |
| Hidden API/persistence/auth need | UI-only contract and route/API diff checks | Defer to a later sprint; Sprint 6A-UI does not absorb it implicitly. |

## Dependencies And Current Blockers

- The three product-direction decisions are resolved in this plan.
- A live seeded application is available for the targeted audit and visual comparison.
- `docs/ui-guidance.md`, `docs/ui-guidance-spec.md`, `style/main.css`, `crates/tessara-web-ui`, and established Administration/list/detail pages are the design references.
- Locked end-to-end dependencies and browsers are required for focused UI proof and the final unchanged-suite run.
- The full `scripts/validate.ps1` remains Sprint-6A-specific and includes destructive upgrade/fresh proof. This presentation-only sprint uses scoped source gates during development and does not rerun Sprint 6A populated-upgrade or rollback-package proof unless an approved scope change touches persistence, migration, API, authorization, or contracts.
- No product or technical blocker is currently known. The next product checkpoint is selection among the three visual directions before implementation.

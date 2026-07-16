# Sprint 6A-UI Baseline Inventory

Status: source baseline initiated on 2026-07-15 at kickoff commit `c37153b19787d4164eaccbb4752980772e6ec84a`. Visual capture, issue scoring, and prioritization are intentionally pending Decision Gate 0 in [the sprint plan](./sprint-6a-ui-plan.md).

This document is the audit ledger for Sprint 6A-UI. It separates verified source facts, retained evidence, observations, and approved implementation issues so the sprint does not confuse design preference with functional behavior.

## Authority And Evidence Order

1. `docs/roadmap.md` defines Sprint 6A-UI scope and immutable product boundaries.
2. `docs/architecture.md` defines current frontend and target module ownership.
3. `docs/ui-guidance.md` is the human-first UI authority; `docs/ui-guidance-spec.md` is its observable-behavior companion.
4. `crates/tessara-web/src/routes/*.rs` is the live route authority.
5. `end2end/acceptance-manifest.json` and its executable specs freeze the supported browser behavior inventory.
6. `docs/playwright-permissions-scenarios.md`, `scripts/smoke.ps1`, and `scripts/uat-sprint.ps1` supply actor, route, and seeded workflow evidence.
7. `docs/ui-screen-inventory.md` is historical `/app/*` migration context only; it is not current route authority.

An audit capture may show a defect or accidental state. It becomes an implementation requirement only after reconciliation with the authorities above.

## Kickoff Source Facts

- Branch: `codex/sprint-6a-ui`
- Base: clean post-closeout `main` commit `c37153b19787d4164eaccbb4752980772e6ec84a`
- Closed Sprint 6A boundary: `f145e059fc1f4d81c960cb35e586c802831ecea2`
- Live mounted route patterns: 48, plus the shared not-found fallback
- Frozen browser manifest: schema 2; 60 exact identities in seven files; SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`
- Browser identity distribution: app 5; Components 3; Dashboards 8; Datasets 10; Modules 5; Permissions 25; workflow-mediated assignments 4
- Global stylesheet: `style/main.css`, 10,133 lines at kickoff, with current root semantic tokens, light/dark theme overrides, shared pattern styling, route styling, responsive queries, and reduced-motion handling
- Policy-neutral shared Leptos primitives: `crates/tessara-web-ui/src`
- Core shell presentation: `crates/tessara-web/src/ui`; navigation/session/theme policy and state remain under `crates/tessara-web/src/state` and their existing feature owners
- Default navigation before an administrator policy change:
  - Main: Home, Organization, Forms, Workflows, Responses, Operations, Components, Dashboards
  - Admin: Administration, Datasets, Module Management
- Migration and Reports have no live routes. Migration remains retired.

The repository contains no usage telemetry or measured workflow-frequency ranking. Any label such as “highest-frequency” is a product prioritization decision, not an empirical claim.

## Mounted Route Matrix

| Route family | Count | Mounted patterns | Current browser/operational evidence |
| --- | ---: | --- | --- |
| Home and sign-in | 2 | `/`; `/login` | `app.spec.ts`, permission SSR cases, smoke/UAT session checks |
| Organization | 4 | `/organization`; `/organization/new`; `/organization/:node_id`; `/organization/:node_id/edit` | permission scenarios, JavaScript-disabled Core/Organization proof, UAT hierarchy flow |
| Forms | 4 | `/forms`; `/forms/new`; `/forms/:form_id`; `/forms/:form_id/edit` | permission scenarios, workflow-mediated shortcuts, JavaScript-disabled Form proof, UAT |
| Workflows | 5 | `/workflows`; `/workflows/new`; `/workflows/assignments`; `/workflows/:workflow_id`; `/workflows/:workflow_id/edit` | permission scenarios, workflow-mediated assignments, JavaScript-disabled Workflow proof, UAT |
| Responses | 4 | `/responses`; `/responses/new`; `/responses/:submission_id`; `/responses/:submission_id/edit` | permission ownership/delegation scenarios, workflow-mediated assignments, no-JS proof, UAT |
| Operations | 1 | `/operations` | permission visibility scenario and JavaScript-disabled route coverage |
| Datasets | 8 | `/datasets`; `/datasets/new`; detail, preview, revisions, revision detail/edit, edit | ten Dataset scenarios, permission revision/read cases, no-JS proof, smoke/UAT |
| Components | 6 | `/components`; `/components/new`; detail, edit, versions, view | three Component workflow scenarios, permission/scope cases, no-JS proof, smoke/UAT |
| Dashboards | 5 | `/dashboards`; `/dashboards/new`; detail, edit, view | eight Dashboard scenarios, permission/redaction cases, no-JS proof, smoke/UAT |
| Administration and Module Management | 9 | `/administration`; users list/detail/edit/access; node types; roles; modules directory/detail | 25-scenario permission coverage includes admin gates; five Module scenarios; smoke/UAT |

Dynamic parameter routes require seeded representative URLs during capture. The frozen audit must record the fixture identity without turning raw IDs into visual labels.

## Connected Parity Journeys

These are documented ordinary workflows and existing acceptance/UAT anchors. Their ranking for deep redesign remains a product decision.

| Journey | Existing behavior boundary |
| --- | --- |
| Session and work entry | Sign in -> Home assigned-work queue -> start eligible work -> account/delegation/scope context -> sign out |
| Organization | Browse visible hierarchy -> inspect detail -> create permitted child -> edit -> preserve scoped visibility |
| Form to response | Create/edit/publish Form -> generated single-form Workflow -> assignment -> assignee/delegator Response start -> draft -> submit -> read-only review |
| Authored workflow | Create/edit ordered Workflow steps -> assign at an available node -> start and progress assigned work |
| Data to presentation | Create/revise/preview/publish Dataset -> author/publish/view Component -> compose/save/view Dashboard using the intended Component version |
| Administration | User/account/access/delegation -> role/capability provenance -> node-type administration with unchanged capability and scope semantics |
| Module administration | Module directory/detail readback -> reader-only policy presentation -> navigation-manager visibility/order edit within existing bands |

## Current Shared Pattern Inventory

### Policy-Neutral `tessara-web-ui`

- Page structure: breadcrumb and page header
- Commands: button variants/sizes/types and dropdown menus
- Data display: data table, interactive/searchable data table, info list, timestamps, status/empty/skeleton presentation
- Table controls: toolbar, search, filters, pagination, column selection
- Inputs and choices: combobox, segmented toggle, tabs
- Overlays: modal/fullscreen dialog and side sheet
- Complex shared interaction: draggable panel list and placement editor/grid primitives

### Core `tessara-web` UI And State

- Authenticated `AppShell`, desktop sidebar, mobile navigation, navigation composition, and top app bar
- Core-owned button/dropdown/info-list/status/table-filter/tabs/timestamp support
- Route/session/navigation/theme state in `crates/tessara-web/src/state` and feature-owned policy outside the presentation directory

### Styling Facts To Audit

- Current root tokens include Inter and DM Sans, semantic primary/secondary/info/success/warning/danger colors, surface/control/table colors, radius, and shadow.
- Light and dark theme overrides live in the same global stylesheet.
- Responsive rules and `prefers-reduced-motion` handling already exist.
- Equivalent-looking patterns may be owned by root UI, policy-neutral UI, feature crates, or global route selectors. The audit must identify duplicates before consolidation; source size alone is not a defect.

## Retained Visual And Design Evidence

| Evidence | Coverage | Boundary |
| --- | --- | --- |
| `design-qa.md` and `docs/audits/sprint-5a-ui-review-2026-07-13/` | 50 retained PNG captures centered on Dashboard desktop/tablet/phone, light/dark, editor/viewer/Table/fullscreen/sheets, with Forms/Components comparisons | Not an application-wide baseline and not a claim of full WCAG conformance |
| `docs/mockups/` | Historical application explorations and approved Sprint 5A Dashboard references | Product decisions are historical unless the current sprint explicitly adopts them |
| `docs/ui-guidance.md` | Canonical brand, shell, tokens, patterns, states, and responsive direction | Human-first guidance; exact current conformance remains to be audited |
| `docs/ui-guidance-spec.md` | Observable shell, navigation, Home, Organization, form-builder, feedback, module-state, and responsive rules | Does not specify exact CSS or every route |
| Sprint 6A closeout/browser artifacts | Functional, authorization, SSR, hydration, console, fresh/upgraded acceptance | Closed proof; not a current visual audit and not to be overwritten |

The Sprint 5A design review explicitly excluded broad screen-reader, contrast, zoom, and platform high-contrast conformance claims. Sprint 6A-UI must gather its own application-wide accessibility evidence.

## Before-State Capture Matrix

Status values are `pending`, `captured`, `reviewed`, and `frozen`. All rows remain `pending` until Decision Gate 0 fixes the visual brief and capture protocol.

| Capture set | Desktop | Tablet | Mobile | Keyboard/focus | Light/dark | State variants | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Bare sign-in and session transition | Pending | Pending | Pending | Pending | Pending | invalid credentials, redirect, logout | Pending |
| Authenticated shell and Home | Pending | Pending | Pending | Pending | Pending | assigned work, empty, loading/error | Pending |
| Organization | Pending | Pending | Pending | Pending | Pending | list/detail/create/edit/validation/forbidden | Pending |
| Forms and Workflows | Pending | Pending | Pending | Pending | Pending | list/detail/create/edit/publish/assign/validation | Pending |
| Responses | Pending | Pending | Pending | Pending | Pending | queue/start/draft/submit/review/forbidden | Pending |
| Operations | Pending | Pending | Pending | Pending | Pending | normal/empty/unavailable/error/forbidden | Pending |
| Datasets | Pending | Pending | Pending | Pending | Pending | author/preview/history/review/publish/errors | Pending |
| Components | Pending | Pending | Pending | Pending | Pending | author/validate/publish/history/view/errors | Pending |
| Dashboards | Pending | Pending | Pending | Pending | Pending | directory/editor/preview/view/redacted/error | Pending |
| Administration | Pending | Pending | Pending | Pending | Pending | users/access/roles/node types/forbidden | Pending |
| Module Management | Pending | Pending | Pending | Pending | Pending | reader/manager/no access/retired/unavailable/finding | Pending |

Every capture record must include commit, URL pattern, seeded fixture/persona, viewport, input mode, theme, JavaScript mode, state setup, screenshot/DOM evidence path, console result, and reviewer. Secrets, session tokens, and user-managed credentials are never retained.

## Prioritized Issue Matrix Schema

The audit will append issue rows here after Decision Gate 0. Priority is based on task completion, accessibility, consistency/blast radius, and behavior risk—not aesthetic preference alone.

| ID | Evidence | Route/pattern | Persona | Viewport/input | Finding | Severity | Shared owner | Behavior risk | Proposed outcome | Acceptance proof | Decision/status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| _No issues frozen at kickoff_ | | | | | | | | | | | Pending Decision Gate 0 |

Severity rubric:

- `P0`: blocks a supported workflow or creates a serious security/accessibility failure; implementation pauses for explicit triage.
- `P1`: prevents or materially impairs a primary task, keyboard use, or a supported viewport.
- `P2`: recurring hierarchy, consistency, state, or efficiency problem with a shared solution.
- `P3`: localized polish that does not materially impede task completion.

## Freeze Checklist

Production UI implementation begins only when:

- [ ] the three product decisions are recorded in the sprint plan;
- [ ] the visual brief and information-architecture constraint are approved;
- [ ] the 48-route matrix has representative before-state evidence;
- [ ] normal and exceptional state gaps are explicitly marked captured, not applicable, or blocked with reason;
- [ ] the prioritized issue matrix is reviewed and frozen;
- [ ] deep workflows and baseline-only route families are named;
- [ ] representative keyboard flows and the minimum mobile width are named;
- [ ] the small visual-regression baseline set and review/update protocol are named;
- [ ] accessibility and visual test tooling/evidence formats are selected; and
- [ ] the reinforced implementation plan maps every first-pass slice to issue IDs and proof.

# Sprint 6A-UI Targeted Baseline Inventory

Status: narrowed product scope and first current-run visual audit recorded on 2026-07-15. Production UI implementation remains pending selection among three screenshot-grounded directions.

Sprint 6A-UI corrects presentation defects introduced by Sprint 6A on the Module Management directory/detail, its navigation-policy controls, the Sprint 6A-added Administration entry, and capability-provenance presentation. Existing Tessara pages are reference and regression surfaces only.

## Authority And Evidence Order

1. `docs/roadmap.md` and `docs/sprints/sprint-6a-ui-plan.md` define the approved scope and immutable behavior boundary.
2. `docs/ui-guidance.md` and `docs/ui-guidance-spec.md` define the existing Tessara identity and observable UI posture.
3. The live Sprint 6A source defines the current targeted content and semantics.
4. `end2end/acceptance-manifest.json` and its executable specs freeze supported browser behavior.
5. Current-run screenshots and DOM inspection identify presentation defects; they do not authorize behavior changes.
6. Existing Administration/list/detail pages and shared components are comparison references, not redesign targets.

## Source And Regression Facts

- Branch: `codex/sprint-6a-ui`
- Sprint branch base: `c37153b19787d4164eaccbb4752980772e6ec84a`
- Closed Sprint 6A boundary: `f145e059fc1f4d81c960cb35e586c802831ecea2`
- Sprint 6A production UI boundary: `6580b040236f563c30b5162fa833d7b0fed16478`
- Pre-Sprint 6A implementation base used for footprint review: `3625d4de`
- Live route inventory: 48 patterns plus the shared not-found fallback; all but the named Sprint 6A surfaces are regression-only.
- Frozen browser manifest: schema 2, 60 exact identities in seven files, SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`.
- Sprint 6A Module Management proof: five exact identities in `end2end/tests/modules.spec.ts` covering read/manager/no-access authority, directory/detail/API parity, route states, keyboard policy editing and band limits, desktop/mobile navigation, SSR/no-JavaScript usefulness, and `/bridge/*` exclusion.
- Administration entry proof: `end2end/tests/app.spec.ts`.
- Role/capability scope and `admin:all` exception proof: relevant `end2end/tests/permissions.spec.ts` scenarios.
- The closed Sprint 6A final fresh acceptance passed all 60 identities. That is historical parity evidence; Sprint 6A-UI must run the unchanged inventory once against its final release candidate.

## Editable Surface Inventory

| Surface | Route/source | Sprint 6A addition | 6A-UI boundary |
| --- | --- | --- | --- |
| Module directory | `/administration/modules`; `features/modules/pages.rs`, `directory.rs` | New route, runtime context, seven-entry inventory, navigation policy | Layout, hierarchy, semantics, wrapping, responsive presentation, scoped styles |
| Module detail | `/administration/modules/:definition_id`; `features/modules/pages.rs`, `detail.rs` | New descriptor detail with peer declaration/dimension sections | Layout, hierarchy, long-content handling, action placement, responsive presentation |
| Navigation policy | Both Module Management routes; `features/modules/policy.rs` | New read/manager policy display and controls | Legibility, row hierarchy, action layout, focus presentation, responsive behavior |
| Administration entry | `/administration`; `features/administration/pages/landing.rs` | New Module Management card/link | Alignment with neighboring cards only |
| Capability provenance | Role/access administration; Sprint 6A capability metadata and related fragments | New global/scope/provenance explanations | Legibility and established-pattern alignment only |

Supporting edits may touch `style/main.css` or a shared UI component only when narrowly required by these surfaces and proven not to change unrelated pages. Shell, navigation composition, session/auth, route/API, persistence, contract, and lifecycle files are outside the UI implementation boundary.

## Existing Tessara Patterns To Reuse

- Framing: `AppShell`, `Breadcrumb`, `PageHeader`, and `route-panel`.
- Metadata: `InfoListTable` or the established definition-list/card treatment when exact semantics fit.
- Collections: shared `DataTable`, including its existing horizontal containment behavior.
- States: `EmptyState`, `organization-state`, and explicit status-badge variants.
- Detail layouts: `organization-detail-content`, `organization-detail-content__grid`, `organization-detail-card`, and `organization-detail-card--wide`.
- Actions: existing button variants, `form-actions`, and table action groups.
- Tokens: the current `style/main.css` semantic colors, typography, spacing, radius, surface, border, focus, theme, responsive, and reduced-motion values.

The Sprint 6A Module Management files introduced new page structure but no Module-specific style rules. Generic two-column detail and five-column data-table rules therefore receive content they were not designed to contain. The correction should use existing patterns plus narrowly scoped layout rules, not a new design system.

## Current-Run Evidence

Captured from the live seeded application at `http://localhost:8080` on 2026-07-15, authenticated as the seeded administrator, dark theme, 1280×720 viewport:

| Evidence | State | Observation |
| --- | --- | --- |
| [Directory first viewport](../audits/sprint-6a-ui-module-management-2026-07-15/01-module-directory-current-accepted.png) | Directory, top | Existing shell and page framing are coherent; Core runtime metadata consumes most of the first viewport and delays the primary inventory task. |
| [Directory inventory](../audits/sprint-6a-ui-module-management-2026-07-15/02-module-directory-inventory-policy-current-accepted.png) | Directory, inventory | Five dense columns, full digests, long status badges, explanations, and Release/Instance text make rows difficult to scan and create severe horizontal clipping. |
| [Forms module detail](../audits/sprint-6a-ui-module-management-2026-07-15/03-module-detail-current-accepted.png) | Detail, representative content | The generic two-column grid allows long overview/digest/declaration content to overlap; the source-descriptor action collides with content and page-level horizontal scrolling appears. |
| [Navigation policy controls](../audits/sprint-6a-ui-module-management-2026-07-15/04-navigation-policy-current-accepted.png) | Manager, contributed destinations | Labels and destination IDs run together, placement bands wrap unpredictably, and repeated order controls create a dense row at 1280px. |

The DOM retains useful headings, regions, table semantics, link/button names, read/manager authority, exact IDs, and explicit lifecycle text. Those are strengths to preserve.

## Prioritized Issue Matrix

| ID | Evidence | Finding | Severity | Behavior risk | Existing-pattern outcome | Required proof | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MM-UI-01 | Directory inventory | Fixed-width generic table plus long machine values makes the inventory unreadable and horizontally clipped at a supported desktop width. | P1 | Medium: restructuring must preserve every field and row/table semantics. | Establish intentional column hierarchy, safe wrapping, and responsive containment using the shared collection posture. | Field/parity assertions, semantic table or approved equivalent, 1280/768/390 overflow cases, reviewed screenshot. | Approved |
| MM-UI-02 | Forms detail | Two-column peer grid permits content overlap, a misplaced action, and page-level horizontal scrolling. | P1 | Medium: all declaration/dimension content and source action must remain. | Use an established wide/stacked responsive detail composition with `min-width: 0` and long-value handling. | Existing detail parity, source-link assertion, heading order, viewport/zoom/keyboard proof. | Approved |
| MM-UI-03 | Navigation policy | Destination labels/IDs concatenate visually and controls/placement data are too dense to scan. | P1 | High: policy order, visibility, disabled rules, focus restoration, and authorization cannot change. | Apply established stacked-label and action-group hierarchy with scoped responsive layout. | Existing keyboard/persistence scenario unchanged plus focused reader/manager viewport and focus proof. | Approved |
| MM-UI-04 | Directory first viewport | Runtime context is verbose and vertically dominant relative to the directory task. | P2 | Low: values and explanatory caveat must remain available. | Compact metadata using an established information-list/card treatment and clearer section hierarchy. | Exact values/text present, heading/region semantics, 1280/390 review. | Approved |
| MM-UI-05 | Directory/detail | Full IDs, digests, finding paths, routes, and capability IDs lack safe wrapping and secondary hierarchy. | P2 | Medium: machine-readable values must not be truncated from assistive or copyable content. | Scoped wrapping and secondary-code treatment using existing muted text/token patterns. | Long-value fixtures, no page overflow, DOM contains exact values. | Approved |
| MM-UI-06 | Directory/detail/policy | Lifecycle, provenance, and policy explanations compete with primary labels instead of supporting them. | P2 | Medium: Active, Unavailable, Retired, finding, Release, and Instance meanings must remain exact. | Preserve explicit badges and explanations while separating primary state from supporting detail. | State-specific semantic assertions and reviewed visual states. | Approved |
| MM-UI-07 | Administration/provenance | Target fragments require current visual comparison before deciding whether code changes are necessary. | P3 pending evidence | Low | Match neighboring Administration cards and established role metadata; make no change if already aligned. | Current capture/source comparison and affected app/permission tests only if edited. | Capture pending |

Severity:

- `P0`: serious security/accessibility failure or supported-workflow block;
- `P1`: materially impairs the primary task, keyboard use, or supported viewport;
- `P2`: recurring hierarchy, consistency, state, or efficiency defect; and
- `P3`: localized polish that does not materially impede completion.

## Remaining Targeted Capture Matrix

| Capture | Desktop | Tablet | Mobile | Keyboard/focus | Themes | State variants | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Directory/runtime/inventory | Captured | Pending | Pending | Pending | Dark captured | active, retired, findings captured | In progress |
| Forms detail | Captured | Pending | Pending | Pending | Dark captured | representative declarations/findings | In progress |
| Navigation policy manager | Captured | Pending | Pending | Pending | Dark captured | dirty/save/discard/disabled pending | In progress |
| Navigation policy reader | Pending | Pending | Pending | Pending | Pending | read-only | Pending |
| Restricted/unavailable/error/not-found | Pending | N/A unless layout differs | Pending | Pending | One theme minimum | explicit route states | Pending |
| Administration entry/provenance | Pending | Reference only | Mobile if edited | If edited | Existing themes if edited | global/scope/provenance | Pending |

Every retained capture records commit, URL, fixture/persona, viewport, theme, JavaScript mode, state setup, and evidence path. Secrets and session tokens are never retained.

## Regression-Only Inventory

Home/sign-in, Organization, Forms, Workflows, Responses, Operations, Datasets, Components, Dashboards, the existing shell, and pre-Sprint 6A Administration pages are not redesign targets. Their existing Rust/web tests, the frozen 60 browser identities, smoke, and UAT prove that targeted UI work did not alter application behavior. No all-route before/after visual baseline will be created.

## Implementation Freeze Checklist

- [x] narrowed scope and the three product decisions are recorded;
- [x] current directory, inventory, detail, and manager-policy evidence is retained;
- [x] initial issue IDs and existing Tessara reference patterns are recorded;
- [ ] reader/no-access and Administration/provenance evidence needed by the selected implementation is captured;
- [ ] one of three screenshot-grounded directions is selected;
- [ ] selected option is mapped to issue IDs without adding behavior;
- [ ] focused UI identities, viewports, fixtures, and visual baselines are named; and
- [ ] the implementation slice begins with characterization proof and a scoped diff boundary.

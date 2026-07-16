# Sprint 6A-UI Direction 1 Mockup Suite

Status: selected visual direction, expanded into an implementation-oriented route and state suite on 2026-07-16. The approved product contract and initial navigation order are frozen. The approved functional Module directory search/status filtering and filtered no-match/reset treatments are incorporated in assets 01, 10, and 16.

The suite follows the selected structured-registry direction while correcting non-contractual details from the original concept image:

- Module Management gains functional search by display name or stable definition ID plus an availability/status filter with `All statuses`, `Active in Core process`, `Unavailable`, and `Retired`. Sorting, pagination, installation/lifecycle actions, and Module Release/Instance controls are not currently approved.
- The authoritative sidebar mapping is `core.admin.modules` / Module Management -> the canonical Tessara `Blocks` icon from the existing `icons` system. File, Document, Package, and Cubes glyphs are not substitutes.
- `core.main` and `core.admin` are labeled **Required group**, not **Core group**. Main contains both protected Core destinations and optional destinations.

The approved targeted revision updated assets 01 and 10 with the established Tessara search/status treatment and asset 16 with a filtered no-match/reset state distinct from a truly empty inventory. It also corrected only the Module Management sidebar glyph in assets 01, 03-08, and 13-15. Assets 02, 09-12, and 16 required no sidebar-icon correction; no unaffected mockup was regenerated.

Raster mockups communicate layout and hierarchy. Exact labels, lifecycle meanings, capability rules, routes, digests, and error semantics in the sprint documentation remain authoritative if generated text differs.

## Approved Navigation Composition

- Main: Home, Organization, Forms, Workflows, Responses, Operations, Datasets, Components, Dashboards.
- Admin: User Management, Roles & Access, Node Types, Module Management.
- Main and Admin are required, reorderable groups with stable identities `core.main` and `core.admin`.
- Optional destinations may be hidden, reordered, or moved to another group by effective global `modules:manage_navigation`.
- Home stays shown in Main. Organization stays in Main but may be hidden. The four initial Admin destinations stay shown in Admin. All protected destinations remain reorderable inside their required group.
- Custom groups may be created, renamed, reordered, and deleted only when empty.
- Display configuration never grants route or API access.
- `/administration` and the Administration navigation item do not exist.

## Asset Matrix

| # | Mockup | Route or surface | Persona/state | Viewport | Implementation question answered |
| --- | --- | --- | --- | --- | --- |
| 01 | [Module directory manager desktop](./01-module-directory-manager-desktop.png) | `/administration/modules` | Manager, populated | Desktop | Compact runtime context, canonical seven-row inventory, safe IDs/digests, non-redundant view switching, and established search/status toolbar. |
| 02 | [Forms overview desktop](./02-module-detail-forms-overview-desktop.png) | `/administration/modules/tessara.forms` | Reader/manager, active transitional | Desktop | Collision-safe identity, lifecycle assessment, source descriptor action, declaration summary, and current placement. |
| 03 | [Responses dependencies desktop](./03-module-detail-responses-dependencies-desktop.png) | `/administration/modules/tessara.responses` | Reader/manager, transition-internal findings | Desktop | Separate dependency evidence and catalog findings with long machine values contained. |
| 04 | [Migration retired desktop](./04-module-detail-migration-retired-desktop.png) | `/administration/modules/tessara.migration` | Reader/manager, retired | Desktop | Deliberate retirement, absent surfaces, exact retirement finding, and no restore or unavailable semantics. |
| 05 | [Navigation reader desktop](./05-navigation-composer-reader-desktop.png) | `/administration/modules`, Navigation | Effective global `modules:read` only | Desktop | Complete actor-independent readback without disabled mutation clutter; shell eligibility remains independent. |
| 06 | [Navigation manager clean desktop](./06-navigation-composer-manager-clean-desktop.png) | `/administration/modules`, Navigation | Manager, clean revision | Desktop | Required groups/items, optional item movement, visibility, explicit keyboard order controls, and disabled clean actions. |
| 07 | [Navigation manager dirty desktop](./07-navigation-composer-manager-dirty-desktop.png) | `/administration/modules`, Navigation | Manager, unsaved custom group | Desktop | Custom group creation, cross-group movement, persisted-shell separation, and atomic Save/Discard bar. |
| 08 | [Navigation protection/conflict desktop](./08-navigation-composer-protection-conflict-desktop.png) | `/administration/modules`, Navigation | Manager, revision conflict and rejected delete | Desktop | Revision recovery, required-group protection, non-empty custom-group deletion rejection, and no force overwrite. |
| 09 | [Navigation manager tablet](./09-navigation-composer-manager-tablet.png) | `/administration/modules`, Navigation | Manager, clean revision | 768px-class portrait | Table-to-stacked composition, protected placement, enabled Organization visibility, and explicit movement controls. |
| 10 | [Module directory mobile](./10-module-directory-mobile.png) | `/administration/modules` | Populated | 390px-class portrait | Grouped stacked inventory, compact runtime disclosure, readable lifecycle metadata, natural vertical scrolling, and stacked mobile search/status controls. |
| 11 | [Forms detail mobile](./11-module-detail-forms-mobile.png) | `/administration/modules/tessara.forms` | Active transitional | 390px-class portrait | Section selector instead of multirow tabs, one-column definition rows, and complete lifecycle dimensions. |
| 12 | [Navigation manager mobile](./12-navigation-composer-manager-mobile.png) | `/administration/modules`, Navigation | Manager, item menu open | 390px-class portrait | Touch/keyboard action sheet for show/hide, earlier/later movement, and cross-group placement without drag-only behavior. |
| 13 | [Role capability provenance desktop](./13-role-capability-provenance-desktop.png) | `/administration/roles` | Operator role selected | Desktop | Existing Roles structure with readable Scope and Provenance columns and safely abbreviated source digests. |
| 14 | [Role editor scope validation desktop](./14-role-editor-scope-validation-desktop.png) | `/administration/roles` side sheet | Invalid ordinary mixed-scope role | Desktop | Scroll-contained editor, exact mixed-scope rejection, `admin:all` exception guidance, and disabled Save. |
| 15 | [User role assignment provenance desktop](./15-user-role-assignment-provenance-desktop.png) | `/administration/users/:id/edit` | Legal separate scoped/global roles | Desktop | Dedicated global Module reader role coexisting with a scoped product role without new scope controls. |
| 16 | [Module Management state reference](./16-module-management-state-reference.png) | Shared route components | Loading, empty, filtered no-match, restricted, unavailable, error, not found, policy unavailable, saving, saved | Desktop reference board | Exact reusable hierarchy and action treatment for non-ready states, including distinct genuine-empty and clearable filtered no-match outcomes. |

## Shared UI Contract

- Match Tessara rather than redesign it: existing dark shell, DM Sans headings, Inter/system body, restrained slate surfaces, teal primary actions, orange focus/warning, lime success, indigo information, and red danger.
- Treat mockup colors as directional, not as authorization for new application colors or tokens. Implementation must reuse Tessara's existing semantic color tokens and current theme values; do not shoehorn a generated raster color into production merely to match the mockup.
- Use Tessara's standard semantic `<section>` panel structure wherever comparable application content is conventionally nested in one. Apply this consistently across the suite during implementation and revision review rather than copying a missing or flattened raster container.
- Use one page title. Top-level routes omit redundant breadcrumbs and purpose prose; detail routes use one concise breadcrumb and no duplicate Back control.
- Use spacing, alignment, typography, and row dividers before adding tinted surfaces, borders, or shadows. Do not nest cards without a true object boundary.
- Use one strong action per state. Supporting actions remain visibly secondary.
- Body copy remains 14–16px. Long identifiers, digests, routes, capability keys, evidence, and finding messages wrap or truncate inside their container with copy/reveal access to the complete value.
- No page-level horizontal scrolling, panel overflow, overlapping content, multirow tabs, or tab-strip horizontal scrolling.
- Directory rows have one detail affordance. Navigation tabs are not duplicated by a second route button.
- Directory search and status filters are working native controls, not decorative placeholders. The initial state shows the complete canonical inventory; trimmed case-insensitive name/ID search and exact status criteria combine without reordering results; zero matches provide `Clear filters` and remain distinct from an empty catalog.
- Desktop data rows become labeled stacks or action sheets at narrow widths; they are never merely squeezed.
- Drag-and-drop may be additive only. Explicit keyboard-capable movement controls are authoritative at every viewport.

## Route, Lifecycle, And Security Guardrails

- Sprint 6A defines Module Definition, Module Release, and Module Instance types. Persistence and mutation for Release/Instance begin in Sprint 6B. Every transition contribution therefore shows `No Module Release` and `No Module Instance` without creation controls.
- The exact transition label is `Transitional — not independently deployable`.
- Forms, Workflows, Responses, Datasets, Components, and Dashboards are `Active in Core process`.
- Migration is `Retired`, never unavailable, and has no executable or navigation destination.
- Restricted detail for a known or unknown identifier uses the same nondisclosing treatment.
- A navigation-policy failure is localized; it does not replace a healthy inventory or authorized detail.
- `modules:manage_navigation` implies `modules:read`. `admin:all` implies both and is the sole valid mixed-scope role exception.
- Provenance is read-only metadata. Transition contributions do not create, mutate, own, or assign Core roles.

## Responsive Contract

- Desktop/1280-class: fluid wide content; complete inventory and group rows remain contained.
- 768-class: Tessara's overlay-shell breakpoint, stacked metadata and controls, and no squeezed desktop table.
- 390-class: 16px content padding, 44px controls, one-column details, grouped inventory rows, section selector, and item action sheets.
- Implementation validation still covers both themes, 200% zoom, keyboard-only use, focus stability, and 1280/768/390 viewports. The selected dark mockups do not replace those checks.

## Durable Proof Contract

These mockups are implementation guidance, not proof of correctness. Acceptance requires durable automated and manual evidence for content parity, search/status semantics and reset behavior, authorization/nondisclosure, navigation persistence and revision conflicts, no-JavaScript SSR ownership, hydration/console behavior, keyboard/focus behavior, and responsive containment.

Tests must not be edited merely to make the new implementation pass. Any changed expectation must cite the approved product-contract change, retain all unaffected assertions, add equal-or-stronger replacement proof, and be recorded in `docs/sprints/sprint-6a-ui-test-change-log.md` before the test edit.

## Product Review Ledger

- Asset 01: approved as shown. The canonical Blocks glyph, desktop search/status toolbar, control labels, and seven-row readability are accepted.
- Asset 02: revision approved. Rename `Open exact source descriptor` to `View source descriptor (JSON)` so the machine-readable API representation is not confused with the human-oriented detail page, and place the right-side declaration/current-placement content inside Tessara's standard `<section>` panel treatment.
- Asset 03: approved. The declared-dependency/catalog-finding hierarchy and terminology are accepted; any missing raster container is governed by the suite-wide standard `<section>` implementation rule.
- Asset 04: approved. The retired/unavailable distinction, absence of restore affordances, declared-surface values, and retirement finding are accepted. Apply the suite-wide `View source descriptor (JSON)` action label.
- Asset 05: revision approved. Preserve the read-only presentation and full Main/Admin density, but replace descriptive eligibility phrases such as `Eligible users` with semantic unordered lists of exact capability keys. Label multi-key predicates as `Any of`; retain an explicit `Always eligible` value only where no capability predicate applies.
- Asset 06: approved with one presentation correction. Retain the visibility toggles, `Move to...` cross-group control, group-order controls, omitted manager-mode eligibility, and clean Save/Discard state. For protected placement, use one accessible lock icon with an explanatory accessible name/tooltip instead of redundant icon-plus-`Locked` text or adjacent disabled movement controls.
- Asset 07: revision approved. Preserve the persisted-versus-draft distinction and bottom Save/Discard bar; remove the unnecessary `Moved from Main` badge. Every group is collapsible, so show a consistent stateful caret on expanded Main and Insights as well as collapsed Admin. Apply the suite-wide accessible lock-icon simplification wherever protected placement appears.
- Asset 08: approved as shown. The revision-conflict hierarchy, distinct discard/reload recovery actions, disabled Save state, non-empty-group deletion explanation, and expanded/collapsed carets are accepted.
- Asset 09: revision approved. Preserve the readable stacked tablet composition, touch-sized visibility/movement controls, and cross-group movement. Replace redundant protected-state controls with the suite-wide accessible lock-icon-only treatment, and add consistent expanded/collapsed carets to every group heading; the current raster omits them.
- Asset 10: approved as shown. The stacked mobile search/status controls, compact runtime summary, card readability, lifecycle-metadata hierarchy, and reduced initial card count in exchange for usable filtering are accepted.
- Asset 11: approved with the suite-wide descriptor-label correction. The mobile section selector, module identity/lifecycle hierarchy, one-column definition values, long-identifier treatment, and overall density are accepted; use `View source descriptor (JSON)` for the machine-readable descriptor action.
- Asset 12: approved with suite-wide corrections. The mobile action sheet, item/group relationship, and Show/Hide, earlier/later, and cross-group actions are accepted; apply the accessible lock-icon-only and stateful group-caret rules wherever relevant. This raster does not visibly demonstrate a disabled action, so it does not approve an unseen disabled-state explanation; implementation must provide an accessible reason when an action is unavailable.
- Asset 13: revision approved. Retain the Scope column because `Scope-aware` and `Installation-global` are distinct capability modes, and retain unobtrusive abbreviated digests with copy/reveal access. Simplify redundant provenance wording to `Authoritative source: Core` and, where applicable, `Also declared by: {module} — Transitional in-process`; do not render `Core — Core authoritative` as two user-facing labels.
- Asset 14: approved under the current authorization model. Retain ordinary mixed Scope-aware/Installation-global rejection, actionable separate-role guidance, the documented `admin:all` exception, and disabled Save for an invalid bundle. A user may receive multiple roles, including a dedicated installation-global module role alongside separate scope-aware product roles. Mixed-role support is recorded as future roadmap work requiring unambiguous per-capability or equivalent scope semantics.
- Asset 15: revision approved for Sprint 6A-UI with narrow supporting behavior and evidence. Replace `Usage` with `Assigned on`, expose the existing durable role-assignment `created_at` value through the user-role read model, show the assignment date for persisted selections, `Pending save` for a newly selected unsaved role, and an em dash for unassigned roles. Remove the redundant `New assignment` tag. Preserve the accepted separate scope-aware Operator and installation-global Module reader pattern.
- Asset 16: approved as shown. The genuine-empty and filtered-zero-match distinction, `Clear filters` action, restricted/unavailable/error/not-found separation, localized navigation-policy failure, saving/saved treatments, and overall state-board density are accepted.

The approved targeted raster corrections from this ledger were applied in place on 2026-07-16 to assets 02, 04-07, 09, 11-13, and 15. Assets 01, 03, 08, 10, 14, and 16 were not regenerated during that correction pass.

## Generation Provenance

The suite was generated with the built-in ImageGen workflow using the selected Direction 1 image, current Sprint 6A captures, Tessara design tokens, exact route data, and the approved product decisions as references. Each asset used an independent `ui-mockup` prompt; targeted corrections used `precise-object-edit`. The shared prompt contract was:

> Preserve the selected structured-registry Tessara direction; depict only the named route/persona/state; use exact approved navigation, lifecycle, capability, behavior, and error semantics; remove redundant prose and controls; contain long content; preserve readable type; avoid nested cards, clipping, overlap, page-level horizontal scroll, drag-only controls, unrelated product-workflow expansion, authorization changes, Release/Instance lifecycle work, and broad redesign of pre-existing routes.

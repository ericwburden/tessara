# Tessara UI Guidance

**Status:** Canonical UI guidance for Tessara  
**Date:** 2026-04-14  
**Audience:** Designers, engineers, and reviewers implementing or auditing the Tessara user interface  
**Scope:** Naming, brand expression, information architecture, shell behavior, rendering strategy, layout, components, states, messaging, responsiveness, and transitional UI constraints

## Purpose, authority, and interpretation

This document is the canonical source for Tessara UI decisions. It consolidates the active guidance that was previously split across separate brand, design-language, direction, and primitive-contract UI documents.

Use this document to:

1. guide future UI implementation
2. audit current screens against the intended standard
3. resolve design questions without needing to cross-reference multiple UI planning documents
4. find the current shared primitive contracts and adoption notes in the appendices at the end of this file
5. pair the prose guidance here with [ui-guidance-spec.md](./ui-guidance-spec.md) when a formal Allium behavior contract is useful

Authority rules:

- If another active UI document disagrees with this file, this file wins.
- `roadmap.md`, `requirements.md`, and `architecture.md` remain authoritative for delivery scope, product requirements, and system architecture.
- Historical or transitional route behavior does not override the standards in this file unless the behavior is required to preserve a usable application during migration.

Interpretation:

- **MUST** = binding standard
- **SHOULD** = default behavior unless there is a strong product-specific reason to differ
- **MAY** = optional pattern that still fits Tessara

## Product posture, naming, and delivery rules

### Product posture

Tessara is a configurable data platform for structuring, collecting, and analyzing complex hierarchical data.

The UI MUST read as:

- precise
- calm
- trustworthy
- structured
- modular
- operational rather than decorative

The product SHOULD feel:

- modern but restrained
- efficient for long working sessions
- data-forward without feeling cramped
- like a platform for structured composition and insight, not a one-off admin utility

### Core product principles

1. Quiet by default. Structure and hierarchy should do more work than ornament.
2. One strong action at a time. Most local action groups should expose one clear primary action.
3. Context matters. Keep users in context with drawers, subordinate expansion, and page-local controls when that preserves workflow clarity.
4. Density with breathing room. Tessara is medium-compact inside work surfaces and calmer at the page level.
5. Desktop-prioritized, mobile-friendly. Deep work is optimized for desktop, but tablet and mobile must remain intentionally usable.
6. States must be explicit. Empty, loading, no-results, error, read-only, restricted, and unavailable states must never be conflated.
7. Text-first clarity. Use clear language, readable typography, and predictable hierarchy before reaching for decoration.

### Naming guidance

- Use `Tessara` as the root product name everywhere.
- Prefer clear functional labels over abstract internal branding.
- Keep module and area names consistent with the application information architecture.
- Avoid unnecessary hardcoded legacy labels when configurable terminology is available.

Useful naming posture:

- product name: `Tessara`
- asset names: `Dataset`, `Component`, `Dashboard`
- platform tone: structured, precise, modular, not playful

### Delivery rule

Every future sprint is a full vertical slice.

- Every sprint must deliver both underlying functionality and usable application UI.
- The application must remain in a user-testable condition in the intended end-user-facing shape after each sprint.
- Backend-only or builder-only completion does not satisfy roadmap completion.

## Brand, palette, icon, and theme system

### Brand concept

The visual identity is built around a tessera/tesseract-inspired cube:

- modular geometry
- composition from smaller pieces
- one structured whole assembled from distinct parts

This should reinforce the product themes of:

- organization and hierarchy
- configurable forms and responses
- analytical composition into higher-level views

### Core palette

| Name | Hex | Usage |
| --- | --- | --- |
| Ink | `#0F172A` | text, outlines, code or preformatted backgrounds |
| Slate Dark | `#334155` | secondary text, darker surface accents |
| Slate Mid | `#64748B` | muted text and helper information |
| Neutral | `#E2E8F0` | borders and light structural accents |
| Light | `#F8FAFC` | page background |
| Surface | `#FFFFFF` | cards, panels, inputs |
| Teal | `#14B8A6` | primary action, primary accent |
| Orange | `#F59E0B` | focus outline and highlight accent |
| Lime | `#84CC16` | secondary workflow accent |

### Color and theme rules

- Prefer light application surfaces with `Light` backgrounds and `Surface` cards.
- Use `Ink` for primary text and `Slate Mid` for muted text.
- Use `Teal` for primary buttons and major action emphasis.
- Use `Orange` for keyboard focus outlines.
- Use `Teal`, `Lime`, and `Orange` deliberately. Do not introduce unrelated accent colors without updating this document.
- Tessara supports both light and dark themes through the same shared shell and component system.
- The dark theme MUST use only the approved palette plus opacity variants. Do not introduce new hues without updating this document.
- In dark theme, use `Ink` for shell background, `Slate Dark` for surfaces, `Light` for primary foreground text, `Slate Mid` for muted text, `Neutral` for borders, `Teal` for links and primary actions, `Orange` for focus, and `Lime` for secondary accent states.

### Theme selector behavior

The shell-level theme selector MUST:

- appear in the shared shell chrome rather than inside individual page actions
- live in the sidebar account/footer context area rather than as a full theme control group in the top app bar
- offer `System`, `Light`, and `Dark`
- follow system theme by default
- persist explicit user choice between sessions

### Icon and wordmark guidance

Brand assets already tracked in the repo:

- [tessara-favicon-16.svg](../crates/tessara-web/assets/tessara-favicon-16.svg)
- [tessara-favicon-32.svg](../crates/tessara-web/assets/tessara-favicon-32.svg)
- [tessara-favicon-64.svg](../crates/tessara-web/assets/tessara-favicon-64.svg)
- [tessara-favicon-mono.svg](../crates/tessara-web/assets/tessara-favicon-mono.svg)
- [tessara-icon-256.svg](../crates/tessara-web/assets/tessara-icon-256.svg)
- [tessara-icon-512.svg](../crates/tessara-web/assets/tessara-icon-512.svg)
- [tessara-icon-1024.svg](../crates/tessara-web/assets/tessara-icon-1024.svg)
- [tessara-wordmark.svg](../crates/tessara-web/assets/tessara-wordmark.svg)

Usage rules:

- keep one accent tile per face in the brand mark
- use simplified favicons at small sizes
- avoid distorting icon proportions
- keep sufficient contrast on light backgrounds
- treat application CSS tokens as the implementation expression of this palette

### Metadata and asset integration

When app metadata is updated, prefer:

- route-specific `meta name="description"`
- `meta name="theme-color" content="#F8FAFC"`
- `meta name="color-scheme" content="light dark"`
- Open Graph and Twitter summary metadata that references the 512px icon
- SVG favicon links for the tracked favicon assets

## Information architecture and shell model

### Primary information architecture

The application MUST use a single coherent shell with permission-gated navigation across these main areas:

- Home
- Organization
- Forms
- Workflows
- Responses
- Components
- Dashboards
- Datasets
- Administration
- Migration

Guiding rules:

- product-facing areas should read as real application destinations
- internal or operator areas should stay available but not define the tone of the whole app
- access to routes, navigation groups, and actions SHOULD be governed by permissions rather than role names
- `admin:all` SHOULD unlock a small secondary `Admin` sidebar group rather than a separate mode or shell
- IDs and workbench-style shortcuts should not be required for common user-testing flows
- the shell should respect the active theme through shared shell chrome

### Surface model

Product-facing surfaces:

- Home
- Organization
- Forms
- Responses
- Dashboards

Datasets and Components MAY have product-grade viewers, but authoring is primarily internal or operator-oriented in v1.

Internal or operator surfaces:

- Administration
- Migration
- dataset authoring
- component authoring
- access and role-assignment management
- workflow and materialization monitoring

Internal surfaces SHOULD still feel like part of the same application, but remain visually and structurally subordinate to the core product journey.

### Home strategy

Home SHOULD remain a shared entry surface that supports different permission sets without route-tree fragmentation.

Home SHOULD provide:

- current context
- the user's next queue or assignment work as the primary surface
- a compact hierarchy explorer as the secondary surface
- selected-node related work when hierarchy context is active
- compact glanceable metrics rather than a full row of summary cards
- obvious distinction between product destinations and internal areas

### Screen families

1. **Home / workspace**  
   Shared entry and permission-aware orientation.
2. **Directory**  
   Browseable lists of users, roles, organization nodes, forms, datasets, components, and dashboards.
3. **Detail**  
   Calm inspection of one asset or record with related dependencies and next actions.
4. **Editor / builder**  
   Controlled authoring of forms, fields, datasets, components, dashboards, roles, and assignments. Editors should be task-focused rather than generic workbenches.
5. **Completion / review**  
   Respondent-facing response completion and read-only review.
6. **Viewer**  
   Rendered end-user-facing outputs such as dashboards and component-backed tabular or visual views.

For scoped hierarchy areas, directory screens SHOULD NOT default to a flat card wall. Where users traverse assigned hierarchy branches, prefer a full-width hierarchy navigation pattern with clear parent and child expansion and selection behavior.

### Product and internal boundaries

- Organization, Forms, and Responses should behave like first-class product areas.
- Administration should hold powerful configuration work, but should not remain the only route to core authoring flows.
- Migration should remain clearly operator-focused and visually subordinate to the primary application.
- User management and RBAC should live in internal or admin surfaces, but they must still be application-grade UI.

### Shell model

Tessara MUST use a responsive two-region default shell with an optional right contextual panel.

Desktop (`lg` and up):

- persistent left sidebar
- main workspace
- optional right contextual panel only when the page benefits from it

Tablet (`md` to `lg`):

- sidebar collapsed by default
- main workspace remains primary
- right contextual content becomes a drawer

Mobile (below `md`):

- sidebar becomes overlay navigation
- main content becomes single-column flow
- right contextual content becomes a drawer, sheet, or modal

Responsive requirements:

- No page may require horizontal scrolling at the app-shell level.
- Page headers MUST stack cleanly on narrower screens.
- Multi-column forms MUST collapse to one column.
- Dashboard tiles MUST reflow to a single-column stack on narrow widths.
- Tables MAY scroll horizontally inside their own container, reduce visible columns, or transform into card or list views. They MUST NOT force shell-level horizontal scroll.

### Sidebar

Widths:

- desktop expanded: `256px`
- tablet collapsed: `72px`
- mobile overlay: `288px` or full width on very narrow screens

Behavior:

- Desktop MUST default to expanded.
- Tablet MUST default to collapsed.
- Mobile MUST use overlay behavior, not reserved width.
- Collapsed state MUST show icons, active state, and tooltips or reveal behavior.
- Sidebar nav groups SHOULD be separated by spacing rather than heavy dividers.
- Avoid deep always-expanded trees in main navigation.
- The Organization nav item MAY expose a quiet node-type ladder beneath it when hierarchy context is important.

Navigation structure:

- Primary order SHOULD be:
  - Home
  - Organization
  - Forms
  - Workflows
  - Responses
  - Components
  - Dashboards
- Secondary `Admin` group SHOULD contain:
  - Datasets
  - Administration
  - Migration
- Transitional `Reports` SHOULD NOT appear in the default sidebar contract.

Navigation item style:

- quiet default state
- subtle hover treatment
- active state clearly stronger than hover
- icons supportive, not dominant

### Top app bar and page header

Top app bar:

- height: `56px`
- purpose: global utilities only
- suitable contents: mobile nav toggle, notifications, help, and global search
- page-specific actions MUST NOT live here by default
- account or session controls SHOULD NOT be duplicated in the top app bar
- notifications SHOULD default to a bell-icon treatment rather than a labeled control

Page header:

- lives inside main content
- carries page title, subtitle or metadata, primary action, secondary actions, and page-local controls

Global search:

- MUST be a static field in the top app bar
- SHOULD remain visible and stable rather than hidden behind a launcher

Sidebar footer/context block:

- SHOULD combine account identity, acting-as/delegation context, scope roots, and a compact theme selector trigger
- SHOULD show the specific user being acted as when delegation is active
- SHOULD represent scope as top-level visible organization nodes rather than a vague single branch label
- MAY collapse longer scope-root lists behind an expandable affordance

### Page widths and spacing

Page width tokens:

| Token | Value | Use |
| --- | --- | --- |
| `page-width-readable` | `800px` | Help, docs, reading-heavy pages |
| `page-width-form` | `960px` | Simple forms and settings |
| `page-width-default` | `1200px` | Standard app pages |
| `page-width-fluid` | `100%` of workspace | Dashboards, tables, builders, dense detail pages |

Rules:

- Data-heavy pages SHOULD be wide or fluid.
- Reading-heavy pages SHOULD be constrained.
- Page header alignment MUST match page body width.

Main content horizontal padding:

- mobile: `16px`
- tablet: `24px`
- desktop: `32px`
- `xl` and up: `40px`

Vertical page rhythm:

- top of page body to page header: `24px`
- page header to first content block: `24px`
- between major page sections: `32px`
- between related stacked panels or cards: `16px`
- tight internal grouping: `12px`

### Right contextual panel, drawers, and modals

Right contextual panel widths:

- standard fixed panel: `360px`
- wide variant: `420px`

Rules:

- Default to `360px`.
- Use `420px` only for inspector or configuration panels that truly need it.
- Right contextual panels SHOULD be optional and page-owned.
- On tablet and below, they MUST become drawers.

Modal widths:

- small: `480px`
- medium: `640px`
- large: `800px`

Drawer widths:

- standard: `360px`
- wide: `420px`
- mobile: full-width or near-full-width sheet when needed

Rules:

- Modals are for short, focused tasks and confirmations.
- Drawers are for contextual editing, inspection, and supporting workflows.
- Long or multi-step work MUST use a full page.

### Breadcrumbs and admin distinction

Breadcrumbs SHOULD be used selectively, not universally.

- Use them when the user is clearly inside a hierarchy.
- Omit them when the sidebar and page title already provide enough context.
- Breadcrumbs SHOULD sit above the page title and remain visually subdued.

Administration contexts MUST be visually distinct in a subtle way.

- Use a restrained admin indicator, such as an accent treatment or sidebar-group cue.
- Do not create a completely separate visual theme.

### Hierarchy navigation direction

Organization browsing SHOULD become more scope-aware and less generic.

- Keep `Organization` as the sidebar destination label.
- When a user's highest assigned scope is `Partner`, the page title should read as `Partner Explorer` rather than a generic `Organization List`.
- Higher-level scoped hierarchy screens should present the assigned tree structure directly instead of flattening everything into disconnected cards.
- The canonical desktop and tablet pattern is `Explorer + Selected Node Detail`.
- The explorer SHOULD use indented rows, minimal separators, and restrained selection styling rather than connector-line trees or card-per-node treatments.
- The selected-node panel SHOULD remain a compact summary surface that leads with related forms, responses, dashboards, open issues, and recent changes.
- Management actions SHOULD appear in the selected-node panel but remain secondary to related-work context.
- Capability bundles and scope assignments in Administration should use accessible data-grid layouts once those surfaces need to support larger data sets.

Responsive Organization behavior:

- On tablet, preserve the same explorer/detail model with the sidebar rail collapsed by default.
- On mobile, use a `Tree + Sheet` model:
  - a compact branch selector in the main flow
  - an expandable hierarchy list for choosing a node
  - a lower sheet or lower-panel detail surface for selected-node work
- After a node selection, the hierarchy control SHOULD be able to collapse so selected-node work becomes primary again.

When tabular interaction is required, prefer an accessible data-grid pattern over a static table so keyboard navigation, row and column focus, and dense editing behavior remain coherent.

## Rendering, hydration, and lazy-loading rules

- Default to SSR-first route delivery with progressive enhancement.
- Keep core route state in the URL whenever practical so read-heavy surfaces remain useful even if hydration fails.
- Prefer native links and forms where they preserve workflow clarity. Client-side enhancement should improve the experience, not become the only way the page works.
- Keep the shared shell light. Navigation, titles, breadcrumbs, and core layout should load immediately without depending on heavy lazy chunks.
- Treat browser hydration errors as release-blocking defects.

Lazy loading is for heavy, low-frequency operator widgets and richer analytics viewers, not for core shell or navigation or ordinary browse and detail pages.

Do not lazy-load by default:

- Home
- Organization browse and detail flows
- Forms browse and detail flows
- Responses browse and detail flows
- shared navigation, shell chrome, auth or session bootstrap, and theme controls

First-class route or widget candidates:

- `/app/migration`
- administration capability or scope management grids once they become larger and more interactive
- future dataset or component authoring routes
- dashboard viewer enrichments, chart renderers, JSON or fixture editors, large preview or result tables, and drilldown or inspector panels

Use islands selectively for widget-level enhancements on otherwise read-heavy pages. Islands are not the whole-app architecture for the current migration phase.

## Foundations and tokens

### Typography

Font families:

- primary UI font: `Inter`
- monospace font: `JetBrains Mono`

Recommended weights:

- Inter: `400, 500, 600, 700`
- JetBrains Mono: `400, 500, 600`

Type scale:

| Token | Size / Line height | Weight | Use |
| --- | --- | --- | --- |
| `text-display` | `32 / 40` | `700` | rare landing or hero-like headings only |
| `text-page-title` | `24 / 32` | `600` | page titles |
| `text-section-title` | `18 / 24` | `600` | section headings |
| `text-panel-title` | `16 / 24` | `600` | panel or tile headings |
| `text-body` | `14 / 20` | `400` | standard body text |
| `text-body-strong` | `14 / 20` | `500` | slight emphasis in body text |
| `text-label` | `13 / 18` | `500` | field labels |
| `text-meta` | `12 / 16` | `400` | metadata, helper text, column headers |
| `text-caption` | `12 / 16` | `400` | captions and supporting text |
| `text-table` | `13 / 18` | `400` | table body text |
| `text-table-strong` | `13 / 18` | `500` | table emphasis |
| `text-button` | `14 / 20` | `500` | button text |
| `text-input` | `14 / 20` | `400` | input text |
| `text-chip` | `12 / 16` | `500` | badge or chip text |
| `text-stat-lg` | `28 / 32` | `600` | large metrics |
| `text-stat-md` | `22 / 28` | `600` | medium metrics |
| `text-stat-sm` | `18 / 24` | `600` | small metrics |

Typographic behavior:

- Default body copy MUST use `14px`, not `16px`.
- Tables MUST default to `13px` body text.
- Supporting text SHOULD generally use `12px`.
- Hierarchy SHOULD come from weight, spacing, and placement before large jumps in size.

Numerals:

- Structured data contexts MUST use tabular numerals.
- Apply tabular numerals to tables, stat cards, percentages, counts, currency, aligned IDs, and data-heavy chart or tooltip content.
- Normal paragraph copy SHOULD use proportional numerals.

### Spacing

Use an `8px` base spacing system with controlled intermediate values.

| Token | Value |
| --- | --- |
| `space-0` | `0px` |
| `space-1` | `4px` |
| `space-2` | `8px` |
| `space-3` | `12px` |
| `space-4` | `16px` |
| `space-5` | `20px` |
| `space-6` | `24px` |
| `space-8` | `32px` |
| `space-10` | `40px` |
| `space-12` | `48px` |
| `space-16` | `64px` |

Rules:

- Use `8px` rhythm as the default mental model.
- Use `4px` and `12px` only for tighter internal tuning.
- Use `16px` and `24px` most often inside components and panels.
- Use `32px` and `40px` for major section separation.
- Do not introduce ad hoc spacing values without a named token.

### Corner radius

| Token | Value | Default use |
| --- | --- | --- |
| `radius-0` | `0px` | rare square edges |
| `radius-1` | `4px` | fine sub-elements |
| `radius-2` | `8px` | inputs, buttons, small controls |
| `radius-3` | `12px` | cards, panels, dropdowns |
| `radius-4` | `16px` | dialogs, large drawers |
| `radius-full` | `9999px` | intentional pill shapes only |

Rules:

- Controls SHOULD default to `8px` radius.
- Containers SHOULD default to `12px` radius.
- Large elevated surfaces SHOULD use `16px` radius.
- Pill shapes SHOULD be reserved for intentional chip or avatar treatments, not everything.

### Elevation

Tessara uses a border-first, low-shadow model.

| Token | Value | Use |
| --- | --- | --- |
| `elevation-0` | `none` | page-level and flat surfaces |
| `elevation-1` | `0 1px 2px rgba(0,0,0,0.04)` | cards, panels, sticky headers only if needed |
| `elevation-2` | `0 4px 12px rgba(0,0,0,0.08)` | dropdowns, menus, popovers |
| `elevation-3` | `0 12px 32px rgba(0,0,0,0.12)` | modals, high-priority overlays |

Rules:

- Prefer tonal separation and borders before shadow.
- Most ordinary surfaces SHOULD use no shadow or only `elevation-1`.
- Overlays MUST use shadow to communicate layering.
- Avoid stacking many shadowed surfaces on one screen.

### Borders

| Token | Value | Use |
| --- | --- | --- |
| `border-width-default` | `1px` | standard UI structure |
| `border-width-strong` | `1px` | same weight; stronger color if needed |
| `border-width-heavy` | `2px` | rare emphasis, selected states, non-shadow focus treatments |

Rules:

- Nearly all borders MUST be `1px`.
- Prefer color change before thickness change.
- `2px` SHOULD be rare.

### Motion

Durations:

| Token | Value |
| --- | --- |
| `motion-instant` | `100ms` |
| `motion-fast` | `150ms` |
| `motion-normal` | `200ms` |
| `motion-slow` | `250ms` |

Easing:

| Token | Value |
| --- | --- |
| `ease-standard` | `cubic-bezier(0.2, 0, 0, 1)` |
| `ease-exit` | `cubic-bezier(0.4, 0, 1, 1)` |
| `ease-enter` | `cubic-bezier(0, 0, 0, 1)` |

Rules:

- Hover, focus, and small state changes SHOULD use `100-150ms`.
- Dropdowns and smaller overlays SHOULD use `150-200ms`.
- Drawers and modals SHOULD use `200-250ms`.
- Do not use springy or bouncy motion.
- Page navigation MUST remain instant for now.

### Breakpoints

| Token | Value |
| --- | --- |
| `bp-sm` | `640px` |
| `bp-md` | `768px` |
| `bp-lg` | `1024px` |
| `bp-xl` | `1280px` |
| `bp-2xl` | `1536px` |

Rules:

- below `768px`: mobile layout
- `768-1023px`: tablet or narrow laptop layout
- `1024px+`: full desktop shell available
- `1280px+`: comfortable multi-panel layouts
- `1536px+`: wider data-heavy layouts allowed, but still structured

### Z-index

| Token | Value |
| --- | --- |
| `z-base` | `0` |
| `z-sticky` | `100` |
| `z-dropdown` | `200` |
| `z-popover` | `300` |
| `z-drawer` | `400` |
| `z-modal-backdrop` | `500` |
| `z-modal` | `600` |
| `z-toast` | `700` |
| `z-tooltip` | `800` |

Rules:

- Do not invent ad hoc values like `9999`.
- Sticky elements MUST use `z-sticky`.
- True blocking overlays MUST begin at the modal layers.

## Components, layouts, and interaction patterns

### Buttons

Size scale:

| Size | Height | Horizontal padding |
| --- | --- | --- |
| Small | `32px` | `12px` |
| Medium | `40px` | `16px` |
| Large | `48px` | `20px` |

- Medium is the default.
- Small is for dense toolbars and compact contexts.
- Large is for higher-prominence or touch-friendlier moments.

Variants:

- Primary
- Secondary
- Tertiary or ghost
- Destructive

Styling:

- Primary: solid `Teal` fill, no gradient, no glossy effect
- Secondary: bordered, neutral or lightly tinted surface
- Tertiary or ghost: minimal surface, text-led emphasis
- Destructive: semantic destructive styling in the same family and scale

Rules:

- Most local action groups SHOULD have one obvious primary action at most.
- Buttons MUST NOT rely on shadow as the main emphasis mechanism.
- Icon-only buttons MUST follow the same visual family.
- Standalone button labels MUST use title case.

### Inputs

Size scale:

| Size | Height |
| --- | --- |
| Small | `32px` |
| Medium | `40px` |
| Large | `48px` |

Rules:

- Medium is the default.
- Inputs and buttons in the same action row SHOULD share height.
- Inputs SHOULD use a `1px` border, light or white fill, neutral text, and clear but understated placeholder styling.
- Focus states MUST be strong and visible.
- Disabled states must remain readable but clearly non-interactive.
- Error states must be semantic and accompanied by message text.

Field annotation pattern:

- label above field
- label to field gap: `8px`
- helper or error zone below field
- field to helper or error gap: `6px`
- placeholder text MUST NOT be the sole label

Labels use `text-label`. Helper and error text use `text-meta`. Validation should say what needs to be fixed, not just that something is invalid.

### Select, combobox, autocomplete, and multi-select

Tessara uses three distinct patterns with shared styling:

- **Select** for short scannable option lists
- **Combobox** for longer searchable lists
- **Autocomplete** for search-driven suggestion lookup

Rules:

- Do not use giant unsearchable selects for long lists.
- Overlay menus MUST use dropdown elevation.
- Keyboard support MUST be first-class.
- Empty results MUST explain what happened.

Multi-select:

- The default multi-select pattern is a combobox with chips.
- Selected items render as removable chips inside or directly associated with the field.

### Checkbox, radio, and switch semantics

- Checkbox = independent on or off selections or bulk selection
- Radio = one choice from a small visible mutually exclusive set
- Switch = immediate on or off state change

Rules:

- Do not use switches for option comparison.
- Do not use radios for long lists.
- Do not use checkboxes where only one choice is allowed.

### Tabs

Base style:

- height: `40px`
- text-first
- icons only when useful
- active state: stronger text plus underline or bottom-border indicator
- inactive state: quiet

Overflow behavior:

- When tabs no longer fit, they MUST collapse into a dropdown menu.
- Do not use wrapped multi-row tabs.
- Do not rely on horizontal tab scrolling as the default overflow solution.

### Badges, chips, search, help, avatars, and action icons

Tessara separates:

- status badge = semantic state label
- chip = selected value, filter, or removable token

Defaults:

- height: `24px`
- badge radius: `8px`
- chip radius: pill shape allowed when intentional

Rules:

- Do not use chips and badges interchangeably.
- Keep table badges visually restrained.
- Status must not rely on color alone.

Search fields share the input family and have three scopes:

- global search
- table or list search
- picker or search-with-suggestions

Rules:

- Table search belongs in the table toolbar.
- Global search belongs in the top app bar.
- Leading search icon is allowed.
- Clear button appears when text is present.
- Search scope must be clear from placement and copy.

Tooltips SHOULD be text-only, slightly delayed, concise for simple concepts, and moderately detailed for complex ones. They should clarify, not teach an entire workflow.

Default help or onboarding cue: help icon.

- Prefer on-demand help via help icon rather than intrusive walkthrough overlays.

Avatars:

- default avatar treatment = initials
- do not fall back to generic silhouettes if initials are available

Action icon sizing:

- default compact action icon size: `16px`
- this size MUST NOT force table rows taller than their intended density

### Cards and panels

Tessara uses two distinct container patterns.

Card:

- for summary, concise, or compact content
- default padding: `16px`
- tight variant: `12px`
- radius: `12px`
- border: `1px`
- minimal or no shadow

Panel:

- for substantive working content
- default padding: `24px`
- tight variant: `16px`
- radius: `12px`
- border: `1px`
- shadow: none by default

Nesting rule:

- Prefer one strong outer container.
- Inside it, prefer spacing, dividers, tonal sub-sections, or tight blocks before adding more full cards or panels.
- Avoid repeated `24px` padding inside repeated `24px` padding.
- Add a nested bordered surface only when the inner content is meaningfully distinct.

### Tables and data-heavy work

Table density:

| Element | Height |
| --- | --- |
| Header row | `40px` |
| Body row, default | `44px` |
| Body row, compact | `36px` |

Rules:

- Default density is `44px` rows.
- Compact density is `36px` rows for denser admin or data-quality views.
- Keep row height consistent within a table.
- Header background SHOULD use subtle tonal contrast.
- Zebra striping is off by default.
- Row separators MUST use `1px` lines.
- Sticky headers are allowed and preferred for longer tables.
- Numeric columns MUST be right-aligned and use tabular numerals.
- Hover and selection MUST be visually distinct.
- Keep badges, icons, and row actions restrained.

Row interaction model:

- Clicking a row SHOULD open the primary detail surface when rows represent navigable records.
- Checkbox selection SHOULD appear only when bulk actions exist.
- Keep visible row actions minimal and predictable.
- Use a trailing menu for lower-frequency actions.

Expandable subordinate row:

- expansion MUST use a dedicated affordance such as a chevron
- do not overload default row click to also expand
- default to one expanded row at a time
- expanded content should use `12-16px` internal spacing
- use expansion for quick details, validation, lightweight actions, or child content
- do not use it for long forms or complex editing

Inline editing:

- MUST be deliberately entered via a small edit icon
- do not auto-enter edit mode on general cell click

Pagination:

- default page sizes: `25`, `50`, `100`
- preserve filter and sort state while paging
- reset to page `1` when filters materially change the result set
- show exact totals when available and say so honestly when they are not

Desktop pagination SHOULD show result summary, page-size selector, previous or next, and page numbers when space allows. Mobile SHOULD simplify controls without changing the model.

Table toolbar:

- left side: context or title when needed, search field, high-value inline filters, active filter chips when helpful
- right side: column visibility, density or view controls if supported, export action, saved view selector later if introduced
- when rows are selected, show a selection action bar that replaces or overlays normal toolbar context

### Forms and editing

Tessara-authored forms on wider screens SHOULD prefer a two-column layout.

Rules:

- Use two columns as the default starting pattern on wide screens.
- Use full-width fields for long text, complex controls, or helper-heavy fields.
- Collapse to single column on tablet and mobile.
- Field stack gap within a column: `16px`
- Column gap: `24px`
- Between form sections: `32px`

Admin-built forms:

- per section, administrators MAY choose `1` column or `2` columns
- respect configured column count on wide screens
- collapse to one column on tablet and mobile

Edit placement hierarchy:

- full page = major editing and multi-section configuration
- drawer = contextual editing and inspection
- modal = short, focused tasks and confirmations

Rules:

- Long or multi-step editing MUST use a full page.
- Context-preserving quick edits SHOULD use drawers.
- Modals SHOULD stay short and focused.

Unsaved changes:

- show a calm unsaved-changes indicator near the relevant action area
- confirm on navigation away only when there are real unsaved changes
- do not interrupt users repeatedly while they are still editing

Save model for admin forms:

- MUST use explicit save
- no implicit autosave required
- simple administrative forms SHOULD avoid unnecessary draft workflows
- versioned authoring surfaces MAY use an explicit draft/publish lifecycle when the product model requires it

Mobile form actions:

- on mobile and very small screens, longer forms SHOULD use a floating save or cancel action bar pinned at the bottom of the screen

### Form builder and draft version authoring

The canonical builder guidance for `/app/forms/{form_id}/edit` applies to draft version authoring, not to read-only form detail or respondent completion views.

The screenshots in `docs/form-builder-examples/` are interaction references only. They are useful because both the Google Forms and JotForm examples converge on similar builder patterns, but Tessara MUST keep its own calmer palette, typography, density, and shell behavior.

Desktop builder layout:

- centered authoring canvas is the primary workspace
- visible section rail supports fast section switching without losing canvas context
- floating insert rail stays reachable while authoring fields and sections
- right contextual properties panel appears only when a section or field is selected
- sticky page-level builder actions hold save, publish, and version-lifecycle actions separately from field-card controls
- multiple authored sections SHOULD appear as vertically stacked section panels in the canvas flow

Rules:

- The authoring canvas MUST remain the dominant region.
- The section rail SHOULD be lightweight and utility-focused rather than visually dominant.
- The insert rail SHOULD stay near the canvas edge rather than moving into the global shell chrome.
- The properties panel MUST be selection-driven. When nothing is selected, keep the canvas wide and uncluttered.
- Page-level save/publish/version actions MUST NOT be mixed into field-card footers or section-local control clusters.

Section model:

- sections are the primary authoring containers
- section headers expose title, optional description, order context, and section-level actions
- section-level settings SHOULD include title, description, and configured column count within the section container itself
- section navigation SHOULD support direct jump between sections on desktop
- blank builders SHOULD use guided first actions such as `Add section` and `Add field`, not a drag-only empty state

Rules:

- A form with no authored draft content MUST explain the next one or two useful actions in plain language.
- Section navigation MAY collapse into a dropdown, drawer, or compact rail on narrower screens, but it MUST remain discoverable.
- Section-level actions SHOULD stay in the section header area and remain visually subordinate to page-level save/publish controls.

Field-card model:

- each field is edited in a distinct card on the canvas
- high-frequency edits stay inline on the selected card
- deeper configuration moves into the right contextual panel
- selected cards expose only core visible actions: reorder handle, required toggle, duplicate, delete, and overflow
- the card body previews the respondent-facing control shape whenever practical

Rules:

- Selected state MUST be clearly stronger than hover and default states.
- Reorder affordance MUST be explicit. Do not rely on imprecise drag discovery alone.
- Duplicate and delete SHOULD remain quick actions, but advanced settings SHOULD stay out of the card footer.
- Page-level workflow actions and field-level editing actions MUST remain visually distinct.

Properties and configuration model:

- use a hybrid inline + panel editing pattern
- inline editing covers label changes, control preview, and fast option editing
- the right panel holds deeper section or field configuration
- advanced configuration SHOULD preserve canvas context rather than forcing a full route change

Choice-field source model:

- option-based fields use exclusive source modes
- a field may use field-specific inline options or a reusable option source/lookup, never both
- inline option editing is the fast path for field-owned choice lists
- reusable option-source selection and advanced option metadata belong in the contextual properties panel

Rules:

- The UI MUST make the active option-source mode explicit.
- Switching source modes SHOULD clearly signal that the other mode is unavailable for that field.
- Option-based fields SHOULD keep add, remove, and reorder actions close to the inline option list when the field owns its options.

Draft, published, and read-only boundaries:

- section and field authoring is draft-only
- published versions are read-only
- published-version views SHOULD direct authors toward creating or selecting a draft rather than implying inline mutation
- publish-time validation and lifecycle status SHOULD stay visible in the builder workspace without taking over the page

Responsive builder behavior:

- on tablet and mobile, keep the canvas primary
- convert the insert rail and properties panel into drawers, sheets, or toggles rather than leaving desktop sidebars permanently open
- longer authoring sessions on smaller screens SHOULD still preserve a floating or sticky save/cancel action area

States the builder guidance MUST cover:

- blank draft
- selected field
- selected section
- read-only published version
- validation-blocked draft
- loading or unavailable configuration state

### Object pages, dashboards, charts, drag and drop, focus, and selection

Object detail page template:

1. page header
2. compact metadata or status strip
3. tabs when needed
4. primary content region
5. optional right contextual panel

Metadata strip MAY include status, owner, updated date, scope or organization, version, and read-only indicator.

Rules:

- Tabs are optional, not mandatory.
- Metadata should be compact and scannable.

Dashboard layout:

- MUST use a modular grid
- use consistent tile gaps
- use cards for lighter summary tiles and panels for denser work tiles
- reflow to fewer columns on tablet
- collapse to a single-column stack on mobile
- tables inside dashboards should usually be concise summaries rather than giant full-detail grids

Dashboard tile sizing:

- use a constrained tile sizing system
- support a small set of standard tile spans or sizes
- keep alignment on a visible grid
- avoid masonry chaos
- snap-to-grid behavior is preferred

Chart visualization style:

- restrained and analytic
- no gradients
- no 3D effects
- no glossy treatment
- minimal gridlines
- clear axes and labels
- clean legends and tooltips
- direct labeling preferred when practical
- distinguish clearly among no-data, zero, loading, and error

Chart container pattern:

- optional header with title, subtitle, or actions
- visualization body
- optional footer or meta zone

Drag and drop:

- draggable items MUST have clear handles
- users MUST be able to visualize movement while dragging
- destination and placement feedback should remain obvious

Focus and selection:

- use a consistent visible focus ring across interactive components
- focus ring should be `2px` and remain visible against light surfaces and borders
- hover and focus MUST not look identical
- selection MUST be stronger than hover and distinct from focus
- do not make selection rely only on color

## States, messaging, and feedback

### State separation

Tessara MUST keep these states distinct:

- empty = nothing exists yet
- loading = content is expected but not yet present
- no results = current filters or search returned nothing
- error = something failed
- read-only = visible but not editable
- restricted = access is limited
- unavailable or not found = content does not exist or cannot be reached

### Empty, loading, no-results, and error states

Empty states are for true emptiness only.

Structure:

- title
- one short explanation
- primary next-step action when appropriate
- optional secondary guidance

Rules:

- do not use empty-state messaging for loading, no-results, or errors
- keep empty states calm and product-like

Loading states:

- use skeletons for content-shaped placeholders
- use progress bars or indicators for long-running work
- use spinners only for very small localized waits
- prefer skeletons over generic spinners for page or section loading
- use determinate progress when real progress is known
- keep shimmer subtle

Loading placement:

- panel-specific long-running work should show progress centered within the affected panel
- application-wide work should use a global overlay loading state across the app

No-results states:

- use compact no-results states local to the affected table, list, or panel
- make active filters obvious
- include a direct recovery action such as clearing filters or adjusting search
- do not phrase no-results as if nothing exists yet

Error states:

- use plainspoken, recovery-oriented errors
- prefer local error placement near the affected surface
- distinguish temporary failure, permission issue, validation problem, and unavailable content
- include retry when sensible
- avoid vague "something went wrong" copy without specifics

### Read-only, restricted, and unavailable

Read-only:

- user can view but not edit
- show a small read-only indicator when that might not be obvious
- hide or disable edit actions appropriately

Restricted or no permission:

- explain what is unavailable
- prefer hide versus disable based on whether showing the unavailable action helps the user understand the system

Unavailable or not found:

- use unavailable or error messaging, not permission language

### Alerts, toasts, confirmations, and major success feedback

Use two inline message patterns:

- inline alert for page, panel, or section-level messages
- field-level message for control-specific issues

Inline alert types:

- info
- success
- warning
- error

Rules:

- prefer local alerts over global banners unless the whole page is affected
- success alerts should be quieter than warnings and errors

Toasts:

- types: success, info, warning, error
- placement: top-right, consistently, whether or not a right panel is open
- success or info auto-dismiss: `4 seconds`
- warning or error auto-dismiss: `6 seconds`
- user may dismiss manually
- optional action only when truly useful, such as Undo

Rules:

- do not use toasts for complex explanations
- do not use toasts for field validation
- important state should still exist inline when needed

For major actions, prefer a temporary success banner at the top of the affected page or surface instead of only a toast.

Destructive or irreversible actions SHOULD use an informative confirmation dialog.

- state the consequence clearly
- keep the dialog plain and direct
- typed confirmations are not the default
- destructive color treatment alone is not sufficient protection

### Microcopy and date or time formatting

Tessara UI copy MUST be:

- plainspoken
- competent
- calm
- direct
- helpful without being chatty

Rules:

- prefer clear verb-led actions
- use sentence case for general UI text
- avoid jargon where plain language works
- avoid mascot-like or playful tone
- validation should say what to fix
- confirmations should say what will happen

Capitalization:

- buttons: title case
- everything else by default: sentence case
- preserve acronyms and proper nouns as-is

Default date and time display:

- date: `Apr 13, 2026`
- time: `3:42 PM`
- timestamp: `Apr 13, 2026 at 3:42 PM`

Rules:

- avoid ambiguous numeric-only dates
- relative time is allowed in recent activity and feeds
- use absolute time where precision matters, such as metadata, tables, audits, and formal records

Number formatting beyond the tabular-numeral rules above is not yet globally ratified. Keep formatting consistent within a local surface, but do not treat any unapproved global number-format pattern as binding.

## Transitional constraints and roadmap alignment

### Target asset language

Target UI language MUST move to:

- `Dataset`
- `Component`
- `Dashboard`

Do not plan new future-state screens around separate `Report`, `Aggregation`, or `Chart` asset families.

Preferred future authoring and viewing split:

- Datasets: authoring, detail, and preview
- Components: authoring, publish or version detail, and viewer
- Dashboards: composition and viewer

### Transitional UI constraints

The current app still exposes transitional reporting concepts in code and screens:

- report list, detail, and edit flows
- aggregation configuration and execution paths
- chart-specific builder and viewer paths

Until the transition is complete:

- existing report, aggregation, and chart routes may remain in service if needed for user testing
- new planning should describe them as transitional, not final
- new screen work should avoid deepening the old model unless needed to preserve a usable application between sprints
- the retained JavaScript controller is a temporary bridge and should be tracked route-by-route until each bridged surface has a native Leptos replacement

### Immediate roadmap implications

The next UI work should directly support the roadmap sequence:

- user management and authentication screens
- RBAC and role-assignment screens
- organization management flows
- form, field, and version authoring screens
- response assignment, start, and review flows
- dataset and component authoring in the new model

At every stage, the app should remain usable through the intended shell rather than regress into internal-only builder behavior.

### Deferred or out of scope

Deferred:

- keyboard shortcuts
- command palette or quick-action launcher

Out of scope for this UI guidance:

- exact supported chart or component types beyond what the product requirements and architecture already define
- printable report artifacts composed from prose and components
- a full visual dashboard designer beyond required v1 composition flows
- unsupported permissions or scope-sharing behavior not established elsewhere in canonical docs

## Alignment audit checklist

### Brand and product posture

- [ ] `Tessara` is used as the root product name.
- [ ] Navigation and detail pages use `Dataset`, `Component`, and `Dashboard` as the target asset language.
- [ ] The shell reads as a product, not a utility console.
- [ ] Theme behavior supports `System`, `Light`, and `Dark`.
- [ ] Focus styling uses the approved `Orange` accent.
- [ ] Primary action styling uses the approved `Teal` accent.

### Shell and information architecture

- [ ] The product uses a single coherent shell with permission-gated navigation.
- [ ] Main areas include Home, Organization, Forms, Workflows, Responses, Components, Dashboards, Datasets, Administration, and Migration.
- [ ] Product and internal surfaces are distinct but visually related.
- [ ] The top app bar is global-utility-only and `56px` high.
- [ ] Global search is a static field in the top app bar.
- [ ] Sidebar widths and collapse behavior match the standard.
- [ ] Right contextual panels use `360px` or `420px` widths and become drawers below desktop.
- [ ] Breadcrumbs appear only when they add real hierarchical value.
- [ ] Account, delegation, scope roots, and theme selection live in the sidebar footer/context block.
- [ ] The top app bar does not duplicate account or session controls.

### Rendering and frontend delivery

- [ ] Core routes are SSR-first and remain useful if hydration fails.
- [ ] URL state is used for core route state where practical.
- [ ] Shared shell chrome is not lazy-loaded by default.
- [ ] Hydration errors are treated as release-blocking defects.
- [ ] Lazy loading is reserved for heavy operator and analytics surfaces.

### Foundations

- [ ] Inter is the default UI font.
- [ ] JetBrains Mono is used only for code-like or system text.
- [ ] Type sizes match the approved scale.
- [ ] Structured data uses tabular numerals.
- [ ] Spacing uses approved tokens only.
- [ ] Controls and surfaces use the approved radius scale.
- [ ] Elevation is border-first and low-shadow.
- [ ] Motion uses approved timings and page navigation remains instant.

### Buttons, inputs, and small controls

- [ ] Button heights are `32`, `40`, or `48` only.
- [ ] Button variants are limited to primary, secondary, tertiary or ghost, and destructive.
- [ ] Button labels use title case.
- [ ] Inputs use the shared visual family and size scale.
- [ ] Labels sit above fields and placeholders are not the only labels.
- [ ] Multi-select uses combobox with chips.
- [ ] Checkbox, radio, and switch semantics are respected.
- [ ] Tabs are text-first and `40px` high.
- [ ] Tab overflow becomes a dropdown rather than wrapped rows.
- [ ] Badges and chips are not used interchangeably.
- [ ] Notifications use a quiet bell-icon treatment.

### Surfaces, tables, and forms

- [ ] Cards and panels are used distinctly.
- [ ] Nested surfaces are conservative and not overly padded.
- [ ] Table row heights are `44px` default and `36px` compact.
- [ ] Header row is `40px`.
- [ ] Numeric columns are right-aligned.
- [ ] Hover, focus, and selection are distinct.
- [ ] Pagination is used instead of infinite scroll for primary tables.
- [ ] Table toolbars follow the two-zone pattern.
- [ ] Wide-screen forms prefer two columns.
- [ ] Forms collapse to one column on smaller screens.
- [ ] Edit placement follows full page versus drawer versus modal rules.
- [ ] Admin forms use explicit save rather than autosave.
- [ ] The form builder uses a centered authoring canvas rather than a builder-first control wall.
- [ ] Section navigation stays visible on desktop and remains discoverable on smaller screens.
- [ ] Field and section insertion uses a persistent canvas-adjacent affordance.
- [ ] The right properties panel is selection-driven and does not stay open unnecessarily.
- [ ] Save, publish, and version actions are separated from field-card actions.
- [ ] Authored sections render as stacked section panels with section-level settings visible in the canvas flow.
- [ ] Field cards expose only core direct actions: reorder, required, duplicate, delete, overflow.
- [ ] Choice fields enforce one option source mode at a time: inline field-owned options or reusable option source/lookup.
- [ ] Draft version authoring is editable and published versions are clearly read-only.
- [ ] Blank builder states guide the user toward the first useful actions instead of showing only a drag target.
- [ ] On tablet and mobile, insert rails and properties panels collapse into drawers, sheets, or toggles while the canvas remains primary.

### States and messaging

- [ ] Empty, loading, no-results, error, read-only, restricted, and unavailable states are visually and semantically distinct.
- [ ] Loading uses skeletons or progress rather than empty-state copy.
- [ ] Panel-specific progress is centered in the affected panel.
- [ ] Global progress uses an application overlay.
- [ ] Toasts appear top-right and use approved durations.
- [ ] Major successes use a temporary success banner at the top of the affected surface.
- [ ] Destructive actions use informative confirmation dialogs.
- [ ] Copy is plainspoken, calm, and direct.

### Dashboards, charts, and responsiveness

- [ ] Dashboards use a modular grid and constrained tile sizing.
- [ ] Mobile dashboard layouts collapse to one column.
- [ ] Charts use restrained analytic styling.
- [ ] Drag and drop uses clear handles and live movement feedback.
- [ ] No screen forces shell-level horizontal scrolling.
- [ ] Tables, tabs, forms, and drawers adapt intentionally on smaller screens.
- [ ] Mobile remains usable for viewing, review, lookup, lightweight edits, and shorter forms.
- [ ] Home metrics remain compact and glanceable rather than consuming full-width summary panels.
- [ ] Organization uses explorer-plus-detail on desktop/tablet and tree-plus-sheet on mobile.

## Practical implementation note

When there is a conflict between a legacy screen and this document, default to this document unless:

1. the legacy behavior is required by product functionality
2. a transitional route must remain in service to preserve a usable application between sprints
3. the governing product requirement or architecture document requires a narrower constraint

When in doubt, favor:

- calmer surfaces over decorative styling
- consistent layout over one-off exceptions
- context-preserving patterns over unnecessary page switches
- explicit state communication over visual ambiguity
- fewer, stronger patterns over many custom ones

## Appendix A: Shared Primitive Contracts

These appendices describe the shared primitive contracts that exist in the repo today. They do not override the design rules above. When a primitive contract conflicts with the main body of this document, the main body wins.

### Current primitive layers

There are two real primitive layers in the repo today:

1. `crates/tessara-web/src/features/native_shell.rs`
   - Leptos-native wrappers for shared SSR route structure.
   - Current stable surface primitives:
     - `NativePage`
     - `PageHeader`
     - `MetadataStrip`
     - `Panel`
   - These are the default choice for native `/app` routes.
2. `crates/tessara-ui/src/lib.rs`
   - HTML helper primitives used by compatibility surfaces and older string-rendered route builders.
   - Current stable helpers:
     - `action_group`
     - `page_header`
     - `panel`
     - `panel_with_header`
     - `card`
     - `field_wrapper`
     - `checkbox_field`
     - `toolbar`

These layers are not fully converged yet. New native SSR route work SHOULD prefer the `native_shell` components. Use raw `tessara-ui` HTML helpers only when extending a compatibility surface or when extracting a shared contract that does not yet have a Leptos-native wrapper.

### Native page shell

Use `NativePage` as the outer frame for product-facing SSR routes.

```rust
view! {
    <NativePage
        title="Tessara Home"
        description="Tessara application home for local replacement workflow testing."
        page_key="home"
        active_route="home"
        workspace_label="Shared Home"
        breadcrumbs=vec![BreadcrumbItem::current("Home")]
    >
        // page content
    </NativePage>
}
```

Contract:

- route title, active route, and page key are explicit
- shared shell chrome comes from the native application shell, not page-local markup
- product routes should not rebuild sidebar or top-bar structure locally

### Page header

Use `PageHeader` once per route as the top route summary.

```rust
view! {
    <PageHeader
        eyebrow="Workflows"
        title="Workflow Directory"
        description="Current workflow records, linked forms, and assignment counts appear here."
    />
}
```

Contract:

- one eyebrow
- one page title
- one concise route summary
- page-level actions belong in the header area, not repeated in later panels

### Metadata strip

Use `MetadataStrip` for compact route state.

```rust
view! {
    <MetadataStrip items=vec![
        ("Mode", "Directory".into()),
        ("Surface", "Workflow runtime shell".into()),
        ("State", "Loading workflow records".into()),
    ]/>
}
```

Contract:

- use short label/value pairs only
- route mode, surface, and state are the common first choices
- do not turn this into a general-purpose detail grid

### Panel

Use `Panel` for substantive route sections.

```rust
view! {
    <Panel
        title="Product Areas"
        description="These are the primary destinations for top-level entity browsing and workflow entry."
    >
        // section content
    </Panel>
}
```

Contract:

- title is required
- description is short and explanatory
- body holds the route-local content
- use multiple panels instead of one oversized page body

### Action group

Use `tessara_ui::action_group` when a string-rendered or compatibility-owned surface needs shared action markup.

```rust
let actions = tessara_ui::action_group(&[
    ActionItem::link("Create Workflow", "/app/workflows/new", ActionStyle::Primary),
    ActionItem::link(
        "Open Assignment Console",
        "/app/workflows/assignments",
        ActionStyle::Light,
    ),
]);
```

Contract:

- group route-level or section-level actions only
- prefer links for navigation and buttons for in-place operations
- do not create page-local action wrappers when this shared shape is sufficient

### Card

`tessara_ui::card` exists, but native route adoption is incomplete. Today it is best treated as the compatibility-layer card contract, not as a fully adopted native route primitive.

```rust
let card_html = tessara_ui::card(
    "directory-card",
    "Forms",
    r#"<p>Browse forms, inspect lifecycle state, and review workflow attachments.</p>"#,
);
```

Contract:

- use for concise summary or navigation blocks
- keep content short
- do not invent a new card class family when `record-card`, `directory-card`, or `home-card` already covers the need

### Field wrapper and checkbox field

Use `field_wrapper` or `checkbox_field` when a shared form control block is sufficient.

```rust
let name_field = tessara_ui::field_wrapper(
    "form-name",
    "Name",
    &tessara_ui::text_input("form-name", "text", "off", "Form name", ""),
    Some("Use the top-level label shown in navigation."),
    "wide-field",
);

let active_field =
    tessara_ui::checkbox_field("assignment-active", "Assignment is active", true, None);
```

Contract:

- label above control
- helper text only when it adds meaning
- `wide-field` is the current shared full-width modifier

### Toolbar

Use `toolbar` for compact filter/search/action rows.

```rust
let toolbar_html = tessara_ui::toolbar(
    &tessara_ui::field_wrapper(
        "delegate-context-select",
        "Acting For",
        r#"<select id="delegate-context-select"></select>"#,
        None,
        "",
    ),
    "",
);
```

Contract:

- primary zone holds the main filter/search control
- secondary zone is optional
- do not create new ad hoc filter-row wrappers when this layout works

## Appendix B: Current Primitive Adoption Notes

Current strong adoption:

- native shared page shell on `Home`, `Forms`, `Workflows`, and `Responses`
- permission-gated navigation driven from shared link specifications in `native_shell.rs`
- shared route framing through `NativePage`, `PageHeader`, `MetadataStrip`, and `Panel`

Current partial adoption:

- cards, field grids, and action rows inside `Forms`, `Workflows`, and `Responses` still rely heavily on page-local `record-card`, `form-field`, `button-link`, and `page-title-row` markup
- `Home` still uses route-local `home-card` and `directory-card` markup for destination tiles

Current non-adoption:

- `Organization` routes still render through `inner_html` and retained application-shell builders
- `Administration`, `Reporting`, `Dashboards`, and `Migration` remain compatibility surfaces

Review rule:

- use an approved primitive if one already exists
- if no approved primitive exists, reuse the existing shared class family instead of inventing a new one
- if a bespoke structure is unavoidable, document the reason and the intended replacement target in the PR or issue

## Appendix C: Implementation Notes And Known Gaps

These notes are implementation-facing and remain subordinate to the main policy sections above.

Known gaps that are not redefined here and must be mapped from existing assets or handled elsewhere:

- color token values beyond the approved palette names
- the iconography set itself, beyond the behavior rules in this document
- a globally ratified number-formatting standard beyond the rules already stated above

When auditing or implementing against this document:

- use the main body for policy and design decisions
- use Appendices A and B for current shared primitive and adoption contracts
- treat transitional implementation details as temporary unless they are explicitly promoted into the main body

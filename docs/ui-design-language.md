# Tessara UI Design Language — Codex Implementation Spec

**Status:** Ratified synthesis of accepted UI decisions  
**Date:** 2026-04-14  
**Audience:** Codex, designers, and anyone auditing Tessara’s current UI against the target design language  
**Scope:** Structure, layout, interaction, component behavior, content styling, responsiveness, and state handling. This document does **not** redefine Tessara’s existing color palette or iconography set.

## How to use this document

This spec is meant to do two jobs at once:

1. **Guide future implementation.** When Codex builds or updates UI, this is the default standard unless a newer ratified decision supersedes it.
2. **Audit the current product.** Any screen or component that diverges from the rules below should be treated as intentionally legacy, a product-specific exception, or a design debt item to bring into alignment.

Interpretation:

- **MUST** = binding standard.
- **SHOULD** = default behavior unless a screen has a strong product-specific reason to differ.
- **MAY** = optional pattern that still fits the system.

## External dependencies and known gaps

The following are **not** redefined here and must be mapped from Tessara’s existing assets or handled in a separate spec:

- **Color palette values.** Tessara already has colors. Codex should map them into named UI tokens referenced by this document, including: brand, neutral, surface, border, text hierarchy, focus, success, warning, danger, info, and a subtle admin accent.
- **Iconography set.** Tessara already has iconography. Use it consistently with the sizing and behavior rules in this doc.
- **Number formatting pattern.** A default approach was proposed during decision-making, but it was **not formally ratified**. Keep number formatting consistent locally, but do not treat any specific global rule as binding until approved.

## Core design posture

Tessara’s UI MUST feel:

- **Precise**
- **Calm**
- **Trustworthy**

The product SHOULD read as:

- modern but restrained
- operational rather than decorative
- structured rather than playful
- efficient for long working sessions
- data-forward without feeling cramped

## Core product principles

1. **Quiet by default.** Structure and hierarchy should do more work than ornament.
2. **One strong action at a time.** Most local action groups should expose one clear primary action.
3. **Context matters.** Keep users in context when possible with drawers, subordinate row expansion, and page-local controls.
4. **Density with breathing room.** Tessara is medium-compact: efficient inside work surfaces, calmer at the page level.
5. **Desktop-prioritized, mobile-friendly.** Deep work is optimized for desktop, but the product MUST remain intentionally usable on tablet and mobile.
6. **States must be explicit.** Empty, loading, no-results, error, read-only, restricted, and unavailable states must never be conflated.
7. **Text-first clarity.** Use plain language, readable typography, and predictable hierarchy before reaching for color or decoration.

---

# 1) Foundations

## 1.1 Typography

### Font families

- **Primary UI font:** `Inter`
- **Monospace font:** `JetBrains Mono`

### Recommended weights

- Inter: `400, 500, 600, 700`
- JetBrains Mono: `400, 500, 600`

### Type scale

| Token | Size / Line height | Weight | Use |
|---|---:|---:|---|
| `text-display` | 32 / 40 | 700 | Rare landing or hero-like headings only |
| `text-page-title` | 24 / 32 | 600 | Page titles |
| `text-section-title` | 18 / 24 | 600 | Section headings |
| `text-panel-title` | 16 / 24 | 600 | Panel or tile headings |
| `text-body` | 14 / 20 | 400 | Standard body text |
| `text-body-strong` | 14 / 20 | 500 | Slight emphasis in body text |
| `text-label` | 13 / 18 | 500 | Field labels |
| `text-meta` | 12 / 16 | 400 | Metadata, helper text, column headers |
| `text-caption` | 12 / 16 | 400 | Captions and supporting text |
| `text-table` | 13 / 18 | 400 | Table body text |
| `text-table-strong` | 13 / 18 | 500 | Table emphasis |
| `text-button` | 14 / 20 | 500 | Button text |
| `text-input` | 14 / 20 | 400 | Input text |
| `text-chip` | 12 / 16 | 500 | Badge/chip text |
| `text-stat-lg` | 28 / 32 | 600 | Large metrics |
| `text-stat-md` | 22 / 28 | 600 | Medium metrics |
| `text-stat-sm` | 18 / 24 | 600 | Small metrics |

### Typographic behavior

- Default body copy MUST use `14px`, not `16px`.
- Tables MUST default to `13px` body text.
- Supporting text SHOULD generally use `12px`.
- Hierarchy SHOULD come from weight, spacing, and placement before large jumps in size.

### Numerals

- Structured data contexts MUST use **tabular numerals**.
- Apply tabular numerals to: tables, stat cards, percentages, counts, currency, aligned IDs, and data-heavy chart/tooltips.
- Normal paragraph copy SHOULD use proportional numerals.

## 1.2 Spacing

Use an **8px base spacing system** with controlled intermediate values.

| Token | Value |
|---|---:|
| `space-0` | 0px |
| `space-1` | 4px |
| `space-2` | 8px |
| `space-3` | 12px |
| `space-4` | 16px |
| `space-5` | 20px |
| `space-6` | 24px |
| `space-8` | 32px |
| `space-10` | 40px |
| `space-12` | 48px |
| `space-16` | 64px |

Rules:

- Use `8px` rhythm as the default mental model.
- Use `4px` and `12px` only for tighter internal tuning.
- Use `16px` and `24px` most often inside components and panels.
- Use `32px` and `40px` for major section separation.
- Do not introduce ad hoc spacing values without a named token.

## 1.3 Corner radius

| Token | Value | Default use |
|---|---:|---|
| `radius-0` | 0px | Rare square edges |
| `radius-1` | 4px | Fine sub-elements |
| `radius-2` | 8px | Inputs, buttons, small controls |
| `radius-3` | 12px | Cards, panels, dropdowns |
| `radius-4` | 16px | Dialogs, large drawers |
| `radius-full` | 9999px | Intentional pill shapes only |

Rules:

- Controls SHOULD default to `8px` radius.
- Containers SHOULD default to `12px` radius.
- Large elevated surfaces SHOULD use `16px` radius.
- Pill shapes SHOULD be reserved for intentional chip/avatar treatments, not everything.

## 1.4 Elevation

Tessara uses a **border-first, low-shadow** model.

| Token | Value | Use |
|---|---|---|
| `elevation-0` | none | Page-level and flat surfaces |
| `elevation-1` | `0 1px 2px rgba(0,0,0,0.04)` | Cards, panels, sticky headers only if needed |
| `elevation-2` | `0 4px 12px rgba(0,0,0,0.08)` | Dropdowns, menus, popovers |
| `elevation-3` | `0 12px 32px rgba(0,0,0,0.12)` | Modals, high-priority overlays |

Rules:

- Prefer tonal separation and borders before shadow.
- Most ordinary surfaces SHOULD use no shadow or only `elevation-1`.
- Overlays MUST use shadow to communicate layering.
- Avoid stacking many shadowed surfaces on one screen.

## 1.5 Borders

| Token | Value | Use |
|---|---:|---|
| `border-width-default` | 1px | Standard UI structure |
| `border-width-strong` | 1px | Same weight; stronger color if needed |
| `border-width-heavy` | 2px | Rare emphasis, selected states, non-shadow focus treatments |

Rules:

- Nearly all borders MUST be `1px`.
- Prefer color change before thickness change.
- `2px` SHOULD be rare.

## 1.6 Motion

### Durations

| Token | Value |
|---|---:|
| `motion-instant` | 100ms |
| `motion-fast` | 150ms |
| `motion-normal` | 200ms |
| `motion-slow` | 250ms |

### Easing

| Token | Value |
|---|---|
| `ease-standard` | `cubic-bezier(0.2, 0, 0, 1)` |
| `ease-exit` | `cubic-bezier(0.4, 0, 1, 1)` |
| `ease-enter` | `cubic-bezier(0, 0, 0, 1)` |

Rules:

- Hover/focus and small state changes SHOULD use `100–150ms`.
- Dropdowns and smaller overlays SHOULD use `150–200ms`.
- Drawers and modals SHOULD use `200–250ms`.
- Do not use springy or bouncy motion.
- **Page navigation MUST remain instant** for now.

## 1.7 Breakpoints

| Token | Value |
|---|---:|
| `bp-sm` | 640px |
| `bp-md` | 768px |
| `bp-lg` | 1024px |
| `bp-xl` | 1280px |
| `bp-2xl` | 1536px |

Rules:

- Below `768px`: mobile layout.
- `768–1023px`: tablet / narrow laptop layout.
- `1024px+`: full desktop shell available.
- `1280px+`: comfortable multi-panel layouts.
- `1536px+`: wider data-heavy layouts allowed, but still structured.

## 1.8 Z-index

| Token | Value |
|---|---:|
| `z-base` | 0 |
| `z-sticky` | 100 |
| `z-dropdown` | 200 |
| `z-popover` | 300 |
| `z-drawer` | 400 |
| `z-modal-backdrop` | 500 |
| `z-modal` | 600 |
| `z-toast` | 700 |
| `z-tooltip` | 800 |

Rules:

- Do not invent ad hoc values like `9999`.
- Sticky elements MUST use `z-sticky`.
- True blocking overlays MUST begin at the modal layers.

---

# 2) Layout and shell

## 2.1 Shell model

Tessara MUST use a **responsive two-region default shell** with an **optional right contextual panel**.

### Desktop (`lg` and up)

- Persistent left sidebar
- Main workspace
- Optional right contextual panel only when the page benefits from it

### Tablet (`md` to `lg`)

- Sidebar collapsed by default
- Main workspace remains primary
- Right contextual content becomes a drawer

### Mobile (below `md`)

- Sidebar becomes overlay navigation
- Main content becomes single-column flow
- Right contextual content becomes a drawer, sheet, or modal

### Mobile-friendly requirements

- No page may require horizontal scrolling at the app-shell level.
- Page headers MUST stack cleanly.
- Multi-column forms MUST collapse to one column.
- Dashboard tiles MUST reflow to a single-column stack on narrow widths.
- Tables MAY scroll horizontally **inside their own container**, switch to reduced-column views, or transform into mobile-friendly card/list views. They MUST NOT force shell-level horizontal scroll.

## 2.2 Sidebar

### Widths

- Desktop expanded: **256px**
- Tablet collapsed: **72px**
- Mobile overlay: **288px** or full width on very narrow screens

### Behavior

- Desktop MUST default to expanded.
- Tablet MUST default to collapsed.
- Mobile MUST use overlay behavior, not reserved width.
- Collapsed state MUST show icons, active state, and tooltips or reveal behavior.
- Sidebar nav groups SHOULD be separated by spacing rather than heavy dividers.

### Navigation item style

- Quiet default state
- Subtle hover treatment
- Active state MUST be clearly stronger than hover
- Icons are supportive, not dominant
- Avoid deep always-expanded trees in the main nav

### Grouping

- Group by user mental model, not backend architecture
- Separate core work from utilities/admin/help/account
- Section headers, where used, MUST be subdued and compact

## 2.3 Top app bar and page header

### Top app bar

- Height: **56px**
- Purpose: global utilities only
- Suitable contents: mobile nav toggle, user/account menu, notifications/help, org/workspace context, global search
- Page-specific actions MUST NOT live here by default

### Page header

- Lives inside main content
- Carries: page title, subtitle/metadata, primary action, secondary actions, page-local controls

### Global search

- Global search MUST be a **static field in the top app bar**.
- It SHOULD remain visible and stable rather than hidden behind a launcher.

## 2.4 Page widths

Use page-type-specific widths.

| Token | Value | Use |
|---|---:|---|
| `page-width-readable` | 800px | Help, docs, reading-heavy pages |
| `page-width-form` | 960px | Simple forms and settings |
| `page-width-default` | 1200px | Standard app pages |
| `page-width-fluid` | 100% of workspace | Dashboards, tables, builders, dense detail pages |

Rules:

- Data-heavy pages SHOULD be wide or fluid.
- Reading-heavy pages SHOULD be constrained.
- Page header alignment MUST match page body width.

## 2.5 Workspace padding

Main content horizontal padding:

- Mobile: **16px**
- Tablet: **24px**
- Desktop: **32px**
- `xl` and up: **40px**

## 2.6 Vertical page rhythm

- Top of page body to page header: **24px**
- Page header to first content block: **24px**
- Between major page sections: **32px**
- Between related stacked panels/cards: **16px**
- Tight internal grouping: **12px**

## 2.7 Right contextual panel

### Widths

- Standard fixed panel: **360px**
- Wide variant: **420px**

### Rules

- Default to `360px`.
- Use `420px` only for inspector/config panels that truly need it.
- Right contextual panels SHOULD be optional and page-owned.
- On tablet and below, they MUST become drawers.

## 2.8 Modal and drawer scales

### Modal widths

- Small: **480px**
- Medium: **640px**
- Large: **800px**

### Drawer widths

- Standard: **360px**
- Wide: **420px**
- Mobile: full-width or near-full-width sheet when needed

Rules:

- Modals are for short, focused tasks and confirmations.
- Drawers are for contextual editing, inspection, and supporting workflows.
- Long or multi-step work MUST use a full page.

## 2.9 Breadcrumbs

Use breadcrumbs **selectively**, not universally.

- Use when the user is clearly inside a hierarchy.
- Omit when the sidebar + page title already provide enough context.
- Breadcrumbs SHOULD sit above the page title and remain visually subdued.

## 2.10 Admin mode distinction

Administration contexts MUST be **visually distinct in a subtle way**.

Rules:

- Use a restrained admin indicator, such as an accent treatment, badge, or header/sidebar cue.
- Do **not** create a completely separate visual theme.
- Exact admin accent values should come from Tessara’s existing palette.

---

# 3) Buttons, controls, and small components

## 3.1 Buttons

### Size scale

| Size | Height | Horizontal padding |
|---|---:|---:|
| Small | 32px | 12px |
| Medium | 40px | 16px |
| Large | 48px | 20px |

- Medium is the default.
- Small is for dense toolbars and compact contexts.
- Large is for higher-prominence or touch-friendlier moments.

### Variants

Tessara MUST support exactly four standard button variants:

- **Primary**
- **Secondary**
- **Tertiary / ghost**
- **Destructive**

### Styling

- Primary: solid brand fill, no gradient, no glossy effect
- Secondary: bordered, neutral or lightly tinted surface
- Tertiary / ghost: minimal surface, text-led emphasis
- Destructive: semantic destructive styling, same general shape and scale as other buttons

Rules:

- Most local action groups SHOULD have one obvious primary action at most.
- Buttons MUST NOT rely on shadow as the main emphasis mechanism.
- Icon-only buttons MUST follow the same visual family.

### Capitalization

- **Standalone button labels MUST use title case.**

Examples:

- `Save Changes`
- `Create Dashboard`
- `Delete Rule`

## 3.2 Inputs

### Size scale

| Size | Height |
|---|---:|
| Small | 32px |
| Medium | 40px |
| Large | 48px |

- Medium is the default.
- Inputs and buttons in the same action row SHOULD share height.

### Styling

- 1px border
- Light or white fill
- Neutral text
- Clear but understated placeholder styling
- Strong visible focus state
- Disabled state must remain readable but clearly non-interactive
- Error state must be semantic and accompanied by message text

### Field annotation pattern

- Label above field
- Label to field gap: **8px**
- Helper/error zone below field
- Field to helper/error gap: **6px**
- Placeholder text MUST NOT be used as the sole label

### Labels and validation

- Labels use `text-label`
- Helper and error text use `text-meta`
- Validation should say what needs to be fixed, not just that something is invalid

## 3.3 Selects, comboboxes, autocomplete, and multi-select

Tessara uses three distinct patterns with shared styling:

- **Select** = short scannable option lists
- **Combobox** = longer searchable lists
- **Autocomplete** = search-driven suggestion lookup

Rules:

- Do not use giant unsearchable selects for long lists.
- Overlay menus MUST use dropdown elevation.
- Keyboard support MUST be first-class.
- Empty results MUST explain what happened.

### Multi-select

- The default multi-select pattern is a **combobox with chips**.
- Selected items render as removable chips inside or directly associated with the field.

## 3.4 Checkbox, radio, switch semantics

- **Checkbox** = independent on/off selections or bulk selection
- **Radio** = one choice from a small visible mutually exclusive set
- **Switch** = immediate on/off state change

Rules:

- Do not use switches for option comparison.
- Do not use radios for long lists.
- Do not use checkboxes where only one choice is allowed.

## 3.5 Tabs

### Base style

- Height: **40px**
- Text-first, icons only when useful
- Active state: stronger text plus underline/bottom border indicator
- Inactive state: quiet

### Overflow behavior

- When tabs no longer fit, especially on narrower screens, they MUST collapse into a **dropdown menu**.
- Do **not** use wrapped multi-row tabs.
- Do **not** rely on horizontal tab scrolling as the default overflow solution.

## 3.6 Badges and chips

Tessara separates:

- **Status badge** = semantic state label
- **Chip** = selected value, filter, or removable token

Defaults:

- Height: **24px**
- Badge radius: **8px**
- Chip radius: pill shape allowed when intentional

Rules:

- Do not use chips and badges interchangeably.
- Keep table badges visually restrained.
- Status must not rely on color alone.

## 3.7 Search fields

Search fields share the input family and have three scopes:

- **Global search**
- **Table/list search**
- **Picker/search-with-suggestions**

Rules:

- Table search belongs in the table toolbar.
- Global search belongs in the top app bar.
- Leading search icon is allowed.
- Clear button appears when text is present.
- Search scope must be clear from placement and copy.

## 3.8 Tooltips

Tooltips SHOULD be:

- text-only
- slightly delayed
- moderately detailed when the concept is complex
- concise when the concept is simple

Rules:

- No icons or decorative chrome needed.
- Tooltips should clarify, not teach an entire workflow.

## 3.9 Help affordance

Default help/onboarding cue: **help icon**.

Rules:

- Prefer on-demand help via help icon rather than intrusive walkthrough overlays.
- The help icon MAY appear in global chrome and/or in complex local contexts.

## 3.10 Avatars

- Default avatar treatment = **initials**.
- Do not fall back to generic silhouettes if initials are available.

## 3.11 Action icon sizing

- Default compact action icon size: **16px**.
- This size MUST NOT force table rows taller than their intended density.

---

# 4) Surfaces, cards, panels, and content framing

## 4.1 Card vs panel

Tessara uses two distinct container patterns:

### Card

- For summary, concise, or compact content
- Default padding: **16px**
- Tight variant: **12px**
- Radius: **12px**
- Border: `1px`
- Minimal or no shadow

### Panel

- For substantive working content
- Default padding: **24px**
- Tight variant: **16px**
- Radius: **12px**
- Border: `1px`
- Shadow: none by default

## 4.2 Nesting rule

Tessara MUST nest conservatively.

Rules:

- Prefer **one strong outer container**.
- Inside it, prefer spacing, dividers, tonal sub-sections, or tight blocks before adding more full cards/panels.
- Avoid repeated `24px` padding inside repeated `24px` padding.
- Add a nested bordered surface only when the inner content is meaningfully distinct.

---

# 5) Tables and data-heavy work

## 5.1 Table density

| Element | Height |
|---|---:|
| Header row | 40px |
| Body row, default | 44px |
| Body row, compact | 36px |

Rules:

- Default density = `44px` rows.
- Compact density = `36px` rows for denser admin/data-quality views.
- Keep row height consistent within a table.

## 5.2 Table structure and styling

Rules:

- Header background SHOULD use subtle tonal contrast.
- Zebra striping is **off by default**.
- Row separators MUST use `1px` lines.
- Sticky headers are allowed and preferred for longer tables.
- Numeric columns MUST be right-aligned and use tabular numerals.
- Hover and selection MUST be visually distinct.
- Keep badges, icons, and row actions restrained.

## 5.3 Row interaction model

Tessara tables use a clear split between navigation, selection, actions, and expansion.

### Row click

- Where rows represent navigable records, clicking the row SHOULD open the primary detail surface for that record.

### Selection

- Checkbox selection SHOULD appear **only when bulk actions exist**.
- Header checkbox MAY support select-all for the current result set.

### Inline actions

- Keep visible row actions minimal and predictable.
- Use a trailing menu for lower-frequency actions.

### Expandable subordinate row

Tessara MAY use an expandable subordinate row for lightweight contextual detail.

Rules:

- Expansion MUST use a dedicated affordance, such as a chevron.
- Do not overload default row click to also expand.
- Default to **one expanded row at a time**.
- Expanded content should use **12–16px** internal spacing.
- Use expansion for quick details, validation, lightweight actions, or child content.
- Do **not** use it for long forms or complex editing.

## 5.4 Inline editing in tables

- Inline editing MUST be deliberately entered via a **small edit icon**.
- Do not auto-enter edit mode on general cell click.
- Use the standard 16px action icon size.

## 5.5 Pagination

Pagination is the default table/list pattern.

### Default page sizes

- 25
- 50
- 100

### Pagination bar

Desktop SHOULD show:

- result summary
- page-size selector
- previous / next
- page numbers when space allows

Mobile SHOULD simplify controls without changing the underlying model.

Rules:

- Preserve filter and sort state while paging.
- Reset to page 1 when filters materially change the result set.
- Show exact totals when available; if not, say so honestly.

## 5.6 Table toolbar

Tables use a standard two-zone toolbar.

### Left side

- context/title when needed
- search field
- high-value inline filters
- active filter chips when helpful

### Right side

- column visibility
- density/view controls if supported
- export action
- saved view selector later if introduced

### Bulk actions

- When rows are selected, show a **selection action bar** that replaces or overlays normal toolbar context.

Rules:

- Search should be prominent when the table is search-driven.
- Do not expose every possible filter inline.
- Use a small number of inline filters plus a “more filters” drawer/panel for deeper filtering.

---

# 6) Forms and editing

## 6.1 Form layout

### Tessara-authored forms

On wider screens, Tessara-authored forms SHOULD prefer a **two-column layout**.

Rules:

- Use two columns as the default starting pattern on wide screens.
- Use full-width fields for long text, complex controls, or helper-heavy fields.
- Collapse to single column on tablet and mobile.
- Field stack gap within a column: **16px**
- Column gap: **24px**
- Between form sections: **32px**

### Admin-built forms

Per section, administrators MAY choose:

- **1 column**
- **2 columns**

Rendering rules:

- Respect configured column count on wide screens.
- Collapse to one column on tablet/mobile.

## 6.2 Edit placement hierarchy

Use a clear hierarchy:

- **Full page** = major editing and multi-section configuration
- **Drawer** = contextual editing and inspection
- **Modal** = short, focused tasks and confirmations

Rules:

- Long or multi-step editing MUST use a full page.
- Context-preserving quick edits SHOULD use drawers.
- Modals SHOULD stay short and focused.

## 6.3 Unsaved changes

- Show a calm unsaved-changes indicator near the relevant action area.
- On navigation away, if there are real unsaved changes, show a confirmation dialog with options to stay or leave without saving.
- Do not interrupt users repeatedly while they are still editing.

## 6.4 Save model for admin forms

Administrative forms MUST use **explicit save**, not autosave.

Rules:

- No drafts required.
- No implicit autosave required.
- These forms are assumed to be short enough for intentional manual save.

## 6.5 Mobile form actions

On mobile and very small screens, forms SHOULD use a **floating save/cancel action bar** pinned at the bottom of the screen.

Rules:

- Keep actions visible during scroll.
- Use this only when a form is long enough that bottom-only actions would be inconvenient.

---

# 7) States, feedback, and messaging

## 7.1 State separation

Tessara MUST keep these states distinct:

- **Empty** = nothing exists yet
- **Loading** = content is expected but not yet present
- **No results** = current filters/search returned nothing
- **Error** = something failed
- **Read-only** = visible but not editable
- **Restricted** = access is limited
- **Unavailable / not found** = content does not exist or cannot be reached

## 7.2 Empty states

Use practical, instructional empty states **only for true emptiness**.

Structure:

- title
- one short explanation
- primary next-step action when appropriate
- optional secondary guidance

Rules:

- Do not use empty-state messaging for loading, no-results, or errors.
- Keep empty states calm and product-like, not decorative.

## 7.3 Loading states

Use a two-part loading system:

- **Skeletons** for content-shaped placeholders
- **Progress bars / indicators** for long-running work
- **Spinners** only for very small localized waits

Rules:

- Prefer skeletons over generic spinners for page/section loading.
- Use determinate progress when real progress is known.
- Keep shimmer subtle.

### Loading placement

- For panel-specific long-running work, show the loading bar or progress state **centered within the affected panel**.
- For application-wide work, use a **global overlay loading state** across the app.

## 7.4 No-results states

Use compact no-results states local to the affected table/list/panel.

Structure:

- title
- one short explanation
- direct recovery action, such as clear filters or adjust search

Rules:

- Make active filters obvious.
- Do not phrase no-results as if nothing exists yet.

## 7.5 Error states

Use plainspoken, recovery-oriented errors.

Rules:

- Prefer local error placement near the affected surface.
- Distinguish between temporary failure, permission issue, validation problem, and unavailable content.
- Retry action SHOULD appear when sensible.
- Avoid vague “something went wrong” copy without specifics.

## 7.6 Read-only, restricted, unavailable

Tessara MUST distinguish among these clearly.

### Read-only

- User can view but not edit
- Show a small read-only indicator when that might not be obvious
- Hide or disable edit actions appropriately

### Restricted / no permission

- Explain what is unavailable
- Prefer hide vs disable based on whether showing the unavailable action helps the user understand the system

### Unavailable / not found

- Use unavailable/error messaging, not permission language

## 7.7 Alerts and inline messages

Use two message patterns:

- **Inline alert** for page/panel/section-level messages
- **Field-level message** for control-specific issues

Inline alert types:

- info
- success
- warning
- error

Rules:

- Prefer local alerts over global banners unless the whole page is affected.
- Success alerts should be quieter than warnings and errors.

## 7.8 Toasts

Use toasts for brief, non-blocking feedback.

Types:

- success
- info
- warning
- error

Behavior:

- **Placement:** top-right, consistently, whether or not a right panel is open
- success/info auto-dismiss: **4 seconds**
- warning/error auto-dismiss: **6 seconds**
- user may dismiss manually
- optional action only when truly useful, such as Undo

Rules:

- Do not use toasts for complex explanations.
- Do not use toasts for field validation.
- Important state should still exist inline when needed.

## 7.9 Success feedback for major actions

For **major actions**, prefer a **temporary success banner at the top of the affected page or surface**.

Examples:

- dashboard created
- major configuration saved
- import completed

Use toasts for smaller routine confirmations; use the banner for larger moments that deserve stronger acknowledgment.

## 7.10 Confirmation and destructive actions

Destructive or irreversible actions SHOULD use an **informative confirmation dialog**.

Rules:

- State the consequence clearly.
- Keep the dialog plain and direct.
- Typed confirmations are **not** the default.
- Destructive color treatment alone is not sufficient protection.

## 7.11 Microcopy and UI writing tone

Tessara’s UI copy MUST be:

- plainspoken
- competent
- calm
- direct
- helpful without being chatty

Rules:

- Prefer clear verb-led actions.
- Use sentence case for general UI text.
- Avoid jargon where plain language works.
- Avoid mascot-like or playful tone.
- Validation should say what to fix.
- Confirmations should say what will happen.

### Capitalization

- **Buttons:** title case
- **Everything else by default:** sentence case
- Preserve acronyms and proper nouns as-is

## 7.12 Date and time formatting

Default date/time display:

- date: `Apr 13, 2026`
- time: `3:42 PM`
- timestamp: `Apr 13, 2026 at 3:42 PM`

Rules:

- Avoid ambiguous numeric-only dates.
- Relative time is allowed in recent activity and feeds.
- Use absolute time where precision matters, such as metadata, tables, audits, and formal records.

---

# 8) Object pages, dashboards, charts, and interaction patterns

## 8.1 Object detail page template

Major objects SHOULD follow a consistent template:

1. page header
2. compact metadata/status strip
3. tabs when needed
4. primary content region
5. optional right contextual panel

Metadata strip contents may include:

- status
- owner
- updated date
- scope / org
- version
- read-only indicator

Rules:

- Tabs are optional, not mandatory.
- Metadata should be compact and scannable, not oversized.

## 8.2 Dashboard layout

Dashboards MUST use a **modular grid**.

Rules:

- Use consistent tile gaps.
- Use cards for lighter summary tiles and panels for denser work tiles.
- Reflow to fewer columns on tablet.
- Collapse to a single-column stack on mobile.
- Tables inside dashboards should usually be concise summaries rather than giant full-detail grids.

## 8.3 Dashboard tile sizing discipline

Use a **constrained tile sizing system**, not arbitrary freeform placement.

Rules:

- Support a small set of standard tile spans/sizes.
- Keep alignment on a visible grid.
- Avoid masonry chaos.
- Snap-to-grid behavior is preferred.
- Responsive layouts should reflow; do not preserve desktop span math on narrow screens.

## 8.4 Chart visualization style

Tessara charts MUST be restrained and analytic.

Rules:

- No gradients
- No 3D effects
- No glossy treatment
- Minimal gridlines
- Clear axes and labels
- Clean legends and tooltips
- Color should be sparse and meaningful
- Direct labeling is preferred when practical
- Distinguish clearly among no-data, zero, loading, and error

## 8.5 Chart container pattern

All visualizations SHOULD use a standard container structure:

- optional header with title/subtitle/actions
- visualization body
- optional footer/meta zone

Rules:

- Visualizations should live inside the same card/panel system as the rest of the product.
- Loading, no-data, and error states should render inside the same container shape.

## 8.6 Drag and drop

When drag-and-drop is available:

- Draggable items MUST have clear handles.
- Users MUST be able to visualize movement while dragging.
- This may be the actual object moving or a ghost/shadow representation.
- Destination and placement feedback should remain obvious.

## 8.7 Focus and selection

### Focus

- Use a consistent visible focus ring across interactive components.
- Focus ring should be **2px** and remain visible against light surfaces and borders.
- Hover and focus MUST not look identical.

### Selection

- Selection MUST be stronger than hover and distinct from focus.
- Use persistent selection styling through background, text, and/or border emphasis.
- Do not make selection rely only on color.

---

# 9) Responsive/mobile summary

These rules apply across the system:

- Desktop is the best experience for complex configuration and dense operational work.
- Tablet MUST remain comfortable and functional.
- Mobile MUST remain usable for viewing, review, lookup, lightweight edits, and shorter forms.
- Sidebar becomes overlay on mobile.
- Right contextual panels become drawers/sheets on tablet and mobile.
- Forms collapse to one column on smaller screens.
- Mobile and very small screens SHOULD use floating save/cancel actions for longer forms.
- Tab overflow MUST collapse to a dropdown rather than wrap.
- Tables must adapt within their container rather than break the shell.
- Toasts remain top-right aligned within safe viewport bounds.

---

# 10) Deferred and out-of-scope items

These are intentionally **not** part of the binding design language at this time.

## Deferred

- **Keyboard shortcuts**: later phase
- **Command palette / quick-action launcher**: later phase

## Out of scope for this design language doc

- Exact supported chart/component types in v1: product functionality decision, not design language
- Exact color token values: map from Tessara’s existing palette
- Exact icon library definition: use Tessara’s existing iconography

## Open / not yet ratified

- **Number formatting pattern**: proposed previously but not formally accepted; do not treat as binding yet

---

# 11) Alignment audit checklist

Use this section to measure current UI against the standard.

## Foundations

- [ ] Inter is the default UI font.
- [ ] JetBrains Mono is used only for code-like/system text.
- [ ] Type sizes match the approved scale.
- [ ] Structured data uses tabular numerals.
- [ ] Spacing uses approved tokens only.
- [ ] Controls and surfaces use the approved radius scale.
- [ ] Elevation is border-first and low-shadow.
- [ ] Motion uses approved timings and page navigation remains instant.

## Shell and layout

- [ ] The product uses the two-region shell with optional right contextual panel.
- [ ] Sidebar widths and collapse behavior match the standard.
- [ ] The top app bar is global-utility-only and 56px high.
- [ ] Global search is a static field in the top app bar.
- [ ] Page widths follow the correct category for the screen type.
- [ ] Main content padding matches breakpoint rules.
- [ ] Right contextual panels use 360px / 420px widths.
- [ ] Modals and drawers use approved size scales.

## Navigation

- [ ] Sidebar items are quiet by default and clearly active when selected.
- [ ] Sidebar groups are separated by spacing, not heavy dividers.
- [ ] Breadcrumbs appear only when they add real hierarchical value.
- [ ] Admin contexts are subtly but clearly distinguished.

## Buttons and inputs

- [ ] Button heights are 32 / 40 / 48 only.
- [ ] Button variants are limited to primary, secondary, tertiary/ghost, and destructive.
- [ ] Button labels use title case.
- [ ] Inputs use the shared visual family and size scale.
- [ ] Labels sit above fields; placeholders are not the only labels.
- [ ] Multi-select uses combobox-with-chips.
- [ ] Checkbox, radio, and switch semantics are respected.

## Tabs, badges, search, help

- [ ] Tabs are text-first and 40px high.
- [ ] Tab overflow becomes a dropdown, not wrapped rows.
- [ ] Badges and chips are not used interchangeably.
- [ ] Search field scope is clear from placement and copy.
- [ ] Tooltips are text-only, slightly delayed, and concise.
- [ ] Help uses a help icon rather than intrusive walkthroughs by default.

## Surfaces

- [ ] Cards and panels are used distinctly.
- [ ] Cards use 16px default / 12px tight padding.
- [ ] Panels use 24px default / 16px tight padding.
- [ ] Nested surfaces are conservative and not overly padded.

## Tables

- [ ] Table row heights are 44px default / 36px compact.
- [ ] Header row is 40px.
- [ ] No zebra striping is used by default.
- [ ] Numeric columns are right-aligned.
- [ ] Hover, focus, and selection are distinct.
- [ ] Row click, checkbox selection, inline actions, and subordinate row expansion are not overloaded.
- [ ] Inline editing uses an explicit edit icon.
- [ ] Pagination is used instead of infinite scroll for primary tables.
- [ ] Table toolbars follow the two-zone pattern.

## Forms and editing

- [ ] Wide-screen forms prefer two columns.
- [ ] Forms collapse to one column on smaller screens.
- [ ] Admin-built forms support 1- or 2-column section configuration.
- [ ] Edit placement follows full page vs drawer vs modal rules.
- [ ] Unsaved changes are indicated calmly and guarded on navigation.
- [ ] Admin forms require explicit save and do not autosave.
- [ ] Mobile forms use floating save/cancel actions when needed.

## States and feedback

- [ ] Empty, loading, no-results, and error states are visually and semantically distinct.
- [ ] Loading uses skeletons/progress, not empty-state copy.
- [ ] Panel-specific progress is centered in the affected panel.
- [ ] Global progress uses an application overlay.
- [ ] Read-only, restricted, and unavailable are distinguished correctly.
- [ ] Toasts appear top-right and use approved durations.
- [ ] Major successes use a temporary success banner at the top of the affected surface.
- [ ] Destructive actions use informative confirmation dialogs.
- [ ] Copy is plainspoken, calm, and direct.

## Dashboards and charts

- [ ] Dashboards use a modular grid and constrained tile sizing.
- [ ] Mobile dashboard layouts collapse to one column.
- [ ] Charts use restrained analytic styling.
- [ ] Chart containers follow the standard header/body/footer pattern.
- [ ] Drag-and-drop uses clear handles and live movement feedback.

## Mobile and responsiveness

- [ ] No screen forces shell-level horizontal scrolling.
- [ ] Tabs, tables, forms, and drawers adapt intentionally on smaller screens.
- [ ] The product remains usable for view/review/lightweight tasks on mobile.

---

# 12) Practical implementation note for Codex

When there is a conflict between a legacy screen and this document, default to this document unless:

1. the legacy behavior is required by product functionality,
2. the color/iconography source of truth requires a local mapping choice, or
3. an open item in this doc has not yet been ratified.

When in doubt, favor:

- calmer surfaces over decorative styling
- consistent layout over one-off exceptions
- context-preserving patterns over unnecessary page switches
- explicit state communication over visual ambiguity
- fewer, stronger patterns over many custom ones

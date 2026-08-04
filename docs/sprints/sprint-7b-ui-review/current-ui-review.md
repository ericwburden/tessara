# Sprint 7B Current UI Review

Review date: 2026-08-03

Visual authority: deployed Sprint 7A application at
`http://127.0.0.1:8086`.

## Inspected routes

- Dashboard editor:
  `/dashboards/01980000-0003-7000-8000-000000000001/edit`
- Component Versions:
  `/components/sprint-7a-metric-card/versions`

The deployed application was inspected with the seeded Tessara Administrator
session. No production data or UI state was changed.

## Deployed Dashboard editor baseline and shell drift

- Independently deployed Dashboard shell: 240-pixel desktop sidebar, 64-pixel
  top bar, and a centered 994-pixel editor workspace at 1280 pixels.
- Dashboard title, four-placement summary, save state, Details, Preview, and
  Save layout actions appear in the existing header.
- Dashboard settings remains a disclosure row.
- Components and Placement details remain the editor toolbar controls.
- The canvas retains its 12-column grid, numbered placement cards, component
  name/type/version metadata, and existing selection affordance.
- At the deployed mobile breakpoint the shell stacks: a compact Tessara/nav
  band, 54-pixel route header, 16-pixel outer gutter, vertical header actions,
  stacked editor controls, and contained canvas.

The compact Dashboard shell is an observed transitional implementation, not an
approved second shell design. Dashboard already renders through
`tessara-module-ui`; Core still renders a parallel local shell. The approved
target uses the richer Core chrome as the canonical SDK presentation for both
routes while retaining the Dashboard product-content measurements and states.

Reference captures:

- `reference/dashboard-editor-1280-dark.png`
- `reference/dashboard-editor-desktop-light.png`
- `reference/dashboard-editor-mobile-dark.png`
- `reference/dashboard-editor-mobile-light.png`

## Deployed Component Versions baseline

- Canonical Core shell: 288-pixel desktop sidebar and 992-pixel application
  region at 1280 pixels.
- The route panel is exactly 944 pixels wide at x=312/y=92.
- Existing breadcrumb, Component heading, Edit/View actions, Versions heading,
  table treatment, border, density, and typography are authoritative.
- The deployed table currently exposes Version, Status, Kind, Dataset Version,
  and Note.
- At 390 pixels the sidebar collapses behind the existing menu control, the
  route panel uses 24-pixel gutters, and the table scrolls inside its own
  container without page-level horizontal overflow.

Reference captures:

- `reference/component-versions-1280-dark.png`
- `reference/component-versions-desktop-light.png`
- `reference/component-versions-mobile-dark.png`
- `reference/component-versions-mobile-light.png`

## Visual system observed

- Atkinson Hyperlegible body text and Nunito Sans headings/actions.
- Dark shell `#0c1528`, dark panels near `#2a394e`, Core/Dashboard sidebars near
  `#2b394f`/`#2d415b`, teal actions and landmarks, lime healthy indicators,
  and existing semantic orange/red states.
- Eight-pixel control radii, one-pixel slate borders, dense tables, compact
  labels, and the existing teal/amber focus language.
- Exact Tessara icon and wordmark assets; Rust/UI/Lucide-family line icons.

## Baseline rule

Mockup omissions and approximations are not replacement instructions. The
deployed application remains authoritative for everything not explicitly
listed as a Sprint 7B delta, except that its two shell implementations must not
be preserved: the canonical SDK shell is the richer deployed Core shell.

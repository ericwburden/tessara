# Sprint 7B Prototype Design QA

QA date: 2026-08-03

## Comparison target

- Source visual truth:
  - `../reference/component-versions-1280-dark.png` for the canonical richer
    Core shell presentation.
  - `../reference/dashboard-editor-1280-dark.png` for unchanged Dashboard
    product content.
  - Matching light and mobile captures under `../reference/` for theme and
    responsive behavior.
- Rendered implementation:
  - `screenshots/15-unified-dashboard-baseline-desktop-dark.png`
  - `screenshots/16-unified-component-desktop-dark.png`
  - `screenshots/17-unified-dashboard-mobile-dark.png`
  - `screenshots/18-unified-dashboard-mobile-menu-dark.png`
  - `screenshots/20-dashboard-annotations-resolved-desktop-dark.png`
  - `screenshots/21-component-vertical-menu-desktop-dark.png`
  - `screenshots/22-dashboard-annotations-resolved-mobile-dark.png`
  - `screenshots/23-dashboard-feedback-final-1280-dark.png`
  - `screenshots/24-component-feedback-final-1280-dark.png`
- Combined full-view comparison:
  `screenshots/19-shell-comparison-desktop-dark.png` (canonical Core reference,
  deployed Dashboard reference, revised Dashboard prototype).
- Latest feedback comparison:
  `screenshots/25-feedback-comparison-1280-dark.png` (deployed Dashboard and
  Component references paired with the final revised states at the same
  1280 x 720 viewport).
- Desktop viewport and pixels: 1280 x 720 CSS px, device scale 1, source and
  implementation captures 1280 x 720 pixels.
- Mobile viewport: 390 x 844 CSS px, device scale 1, implementation capture
  390 x 844 pixels. The browser's 15-pixel scrollbar leaves a 375-pixel content
  client width; page content has no horizontal overflow.
- State: seeded administrator, dark shell convergence baseline, successor
  dependency finding open/closed, mobile menu open, and Component Versions
  active lifecycle state. Light theme was also toggled and verified in-browser.

## Findings

No actionable P0, P1, or P2 differences remain.

- The Dashboard and Component routes both render the same canonical shell
  component, identified in the prototype as `data-shell-owner="tessara-module-ui"`.
- At 1280 pixels both routes measure a 288 x 720 sidebar, 992-pixel application
  region, and 92-pixel top region. Branding, full navigation, search, theme,
  notification, and help controls match the richer deployed Core reference.
- Context differences are limited to active destination, route title, and
  product content: Dashboards / Edit Dashboard versus Components / Component
  Versions.
- Dashboard product content preserves the deployed heading, settings row,
  toolbar, canvas geometry, placement order, buttons, and compact density while
  adding only the recorded Sprint 7B dependency controls.
- The affected placement issue glyph and unaffected placement glyphs each
  measure 31 x 31 pixels and share the exact x=1308 center at the annotated
  1594 x 760 viewport.
- Component lifecycle commands are contained in a vertical-dot row menu rather
  than displayed as inline links. The menu exposes only state-eligible actions.
- The redundant Dashboard toolbar note is absent. The provider-state selector
  is present only in the bottom-right prototype review panel and is explicitly
  labeled as prototype-only.

## Required fidelity surfaces

- Fonts and typography: Atkinson Hyperlegible body text and Nunito Sans
  headings/actions retain the deployed weights, scale, hierarchy, wrapping,
  and antialiasing behavior.
- Spacing and layout rhythm: the canonical 288/992 shell split and 92-pixel top
  region are exact. Dashboard content begins at x=312 under the same 24-pixel
  application gutter as Core. Panel borders, eight-pixel radii, compact table
  and toolbar density, and mobile stacking remain consistent.
- Colors and tokens: deployed navy/slate shell and panels, teal landmarks,
  lime healthy state, amber dependency state, and red destructive state are
  retained in dark and light themes.
- Image quality and asset fidelity: the exact Tessara icon/wordmark assets are
  reused. UI symbols come from the existing icon library; no placeholder,
  hand-drawn, or generated replacement assets were introduced.
- Copy and content: existing deployed labels remain unchanged. New lifecycle,
  observed-revision, dependency-impact, defer, Upgrade, Replace, Remove, and
  confirmation copy matches the recorded Sprint 7B delta contract.

Focused-region comparison was required for shell geometry, navigation state,
global controls, Dashboard toolbar, dependency sheet, Component table, and the
mobile menu. Browser measurements and captures above cover each region.

## Interaction and browser verification

- Closed and reopened Dependency health without losing the Dashboard state.
- Opened and cancelled the Upgrade confirmation dialog.
- Opened the Component version vertical-dot menu, verified Deactivate and
  Archive, launched Deactivate, and cancelled its confirmation dialog.
- Changed the mock provider state from the prototype review panel and verified
  that the dependency sheet updated to the selected state without adding a
  selector to the product workspace.
- Navigated between Component Versions and Dashboard through the shared shell;
  route title and active destination updated correctly.
- Toggled dark to light theme and retained it across route navigation.
- Opened the 288-pixel responsive SDK navigation drawer at 390 pixels.
- Verified the mobile dependency sheet keeps its close control and all actions
  reachable.
- Browser console: no errors or warnings during desktop, mobile, theme,
  navigation, sheet, or action-dialog checks.

## Comparison history

1. Earlier P1: the prototype preserved the deployed compact Dashboard shell as
   a second intentional visual baseline. That contradicted the approved
   SDK-ownership decision.
2. Fix: replaced both prototype render paths with one shared `SdkShell`, using
   the richer deployed Core chrome. Removed the Dashboard-local sidebar, top
   bar, and responsive shell CSS. Updated the sprint execution and UAT contract.
3. Post-fix evidence: captures 15-19 and browser measurements prove identical
   desktop chrome; capture 18 proves Dashboard now uses the canonical mobile
   navigation drawer. No P0/P1/P2 issue remains.
4. Review feedback identified a misaligned issue glyph, inline lifecycle links,
   redundant health copy, and a confusing in-product prototype selector.
5. Fix: normalized the issue glyph to the existing 31-pixel placement slot,
   introduced the vertical-dot row menu, removed the duplicate prose summary,
   and moved provider-state selection into the 7B review panel.
6. Post-fix evidence: captures 20-21 and browser measurements/interactions prove
   aligned glyph centers, absence of duplicate/product-tooling copy, and a
   working state-aware lifecycle menu. No P0/P1/P2 issue remains.
7. Responsive and normalized evidence: capture 22 proves the prototype-only
   selector remains usable at 390 x 844 with no page overflow; captures 23-25
   provide matched 1280 x 720 final comparison evidence.

## Follow-up polish

None required before product-owner review.

final result: passed

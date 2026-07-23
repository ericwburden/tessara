# Sprint 6B1 Directory And Screen B Regression QA

## Directory, Deployment, And Shared Detail Fidelity Pass

- Source visual truth:
  `docs/mockups/sprint-6b1/direction-c-deployment-ledger.html`,
  `docs/mockups/sprint-6b1/direction-b-lifecycle-workspace.html`, the live
  Forms detail at `/administration/modules/tessara.forms`, and the ten supplied
  1922 × 794 browser annotations. The annotations are scoped change requests;
  unaffected application chrome and behavior remain the approved baseline.
- Implementation evidence:
  `outputs/post-fidelity-directory-final-1922x794.png`,
  `outputs/post-fidelity-scoped-final-1922x794.png`,
  `outputs/post-fidelity-deployment-final-1922x794.png`, and the below-fold
  deployment capture
  `outputs/post-fidelity-deployment-final-full.png`.
- Viewport normalization: the three primary implementation captures use a
  1922 × 794 CSS viewport, 1922 × 794 image pixels, and device scale factor 1.
  The source annotations use the same reported browser viewport. Focused
  annotation crops were judged against the corresponding regions in the
  implementation captures rather than against unrelated browser chrome.
- Directory full-view comparison: the search and status controls now consume
  the complete toolbar width in a measured 3:1 ratio (1137.75 px to
  379.25 px), with a 12 px gap before the table and zero document overflow.
  Search still filters to one Scoped Records result and restores all eight
  rows when cleared.
- Screen B focused comparison: Scoped Records now uses the same shared
  module-detail card contract as Forms. Both Definition cards measure
  `padding: 0`, `gap: 0`, the same one-pixel token border, a
  `16px 16px 10.4px` heading inset, and `10.4px 16px` divided rows.
  Diagnostics uses an inline status-value treatment with a measured 7.19 px
  gap between the badge and route.
- Screen C full-view and focused comparison: the receipt download action is
  inside the Resolved Components header at its upper-right corner and uses the
  standard button icon spacing. Plan and artifact digests keep their copy
  controls on the same line. Applied change, Receipt history, Release
  provenance, and Module identities all use the shared zero-padding card plus
  divided-row treatment; their row counts and padding were verified in the
  rendered DOM.
- Time-zone behavior: deployment timestamps are parsed as instants and
  rendered with the browser's default locale/time zone. The displayed
  `7/22/2026, 3:30:00 PM` value is therefore local to this browser; the control
  also exposes the title “Displayed in your local time zone.”
- Fonts and typography: existing Tessara font families, weights, display
  hierarchy, machine-value treatment, and badge typography are unchanged.
- Spacing and layout rhythm: shared card, row, toolbar, button, and inline
  value patterns replace the earlier one-off spacing. No horizontal overflow
  was measured at 1922 × 794.
- Colors and visual tokens: all surfaces, borders, foregrounds, statuses, and
  controls continue to use existing Tessara tokens; no new palette was added.
- Image and icon fidelity: no raster assets changed. The canonical Rust/UI
  Download and Copy icons are retained and aligned through existing button
  icon conventions.
- Copy and content: dynamic receipt/module values are unchanged. Only the
  approved layout and grouping changed.
- Primary interactions tested: directory search/filtering, Deployment tab
  activation, independently deployed detail rendering, copy-button placement,
  and deployment download placement. Browser logs contained no errors.
- Comparison history:
  1. The initial implementation constrained the directory search to 22 rem,
     placed deployment copy controls in separate grid rows, rendered receipt
     metadata as plain card content, and gave Scoped Records one-off panel
     padding.
  2. The first correction introduced the shared row structures and moved the
     download action, but inherited selector specificity kept the independent
     detail cards at 16 px outer padding and the search control at 352 px.
  3. The final correction removed the inherited search cap and raised the
     shared card selector specificity. Post-fix measurements match the Forms
     card geometry, the toolbar ratio is exactly 3, and no actionable P0, P1,
     or P2 differences remain.

## Screen B Mobile Actions Follow-up

- Source visual truth: the supplied 594 × 912 browser annotation showing the
  two full-text module actions stacked below the lifecycle badges, followed by
  the refinement annotation requesting the compact trigger in the heading row
  at its right edge.
- Implementation evidence:
  `outputs/sprint-6b1-mobile-module-actions-title-row-final.png` (594 × 912
  CSS px and image pixels, device scale factor 1), dark theme, Scoped Records
  Overview.
- Full-view comparison: the heading, identity, lifecycle badges, section
  selector, Definition card, typography, colors, spacing, and surrounding
  mobile shell remain unchanged. Only the annotated action area changed.
- Focused action-region comparison: the two 40 px-tall text buttons were
  replaced by one canonical 40 × 40 Tessara ExternalLink icon button. It uses
  the desktop “View…” action's dark control surface, light border, and light
  text/icon tokens, and sits at the far-right edge of the same flex row as the
  “scoped records” heading. The closed state removes mobile wrapping pressure
  without changing the information hierarchy above or below it.
- The menu uses the shared `DropdownMenu` interaction and tokens. It retains
  the exact “View source descriptor (JSON)” and “View deployment receipt”
  labels, their canonical ExternalLink icons, and their original destinations.
  The trigger exposes `aria-haspopup="menu"`, a descriptive accessible name,
  and dynamic expanded state.
- Responsive verification at 594 × 912 confirmed the desktop action group is
  hidden, the mobile menu is visible, and document overflow is zero. The title
  row measured 489 px wide; its trigger's right edge exactly matched the row's
  right edge at x = 534 px. At 1280 × 720 the two desktop buttons remain
  visible and the mobile trigger is hidden. The shared dropdown was exercised
  through open and close transitions; both menu items and their destination
  URLs were inspected in the rendered DOM. Browser logs contained no errors.
- No actionable P0, P1, or P2 differences remain for this annotation.

## Screen B Empty-State And Responsive Follow-up

- Source visual truth: the supplied browser annotations at 1433 × 912 for the
  blank Configuration and Navigation tab states, and at 579 × 912 for the
  missing mobile section control.
- Configuration was not actually empty. Its existing read-only values were
  present in the document but omitted from the active-section display rule.
  The running application now shows Display label, Retention mode, and
  Validation when Configuration is selected.
- Navigation has no declared contribution for this fixture. The running
  application now shows the explicit empty state, “No navigation contribution
  was declared for this module.”
- The independent module detail now uses the same responsive section-control
  pattern as the existing module detail: desktop tabs remain visible above
  780 px; at 780 px and below the tabs are hidden and a full-width native
  select is shown. The select contains all nine detail sections and drives the
  same active-section state.
- Browser verification on the rebuilt local container confirmed both desktop
  states at a 1295 × 912 viewport with zero document overflow. Responsive CSS
  inspection confirmed `display: none` for the detail tabs and
  `display: block; width: 100%` for the detail select at the 780 px breakpoint.
  Browser logs contained no errors; only the pre-existing wasm initialization
  deprecation warning was present.

## Evidence

- Directory source visual truth:
  `docs/mockups/sprint-6a-ui/direction-1/01-module-directory-manager-desktop.png`
  (1487 × 1058 px), specifically the approved contained table, row chevrons,
  and non-scrolling desktop treatment. Sprint 6B1's approved Screen A adds the
  Serving state column and independently deployed module without replacing
  those 6A-UI behaviors.
- Screen B source visual truth: the approved 1394 × 912 browser-annotation
  capture for `docs/mockups/sprint-6b1/direction-b-lifecycle-workspace.html`
  supplied in the review thread, plus its approved HTML structure. The mockup
  is a bounded change reference, not a 1:1 replacement for unaffected live UI.
- Directory implementation: `outputs/sprint-6b1-directory-final.png`
  (1920 × 1080 desktop capture containing the 1394 × 912 in-app Browser
  viewport) and focused combined comparison
  `outputs/sprint-6b1-directory-comparison.png` (2672 × 910).
- Screen B implementation: `outputs/sprint-6b1-detail-final.png`
  (1920 × 1080 desktop capture containing the 1394 × 912 in-app Browser
  viewport).
- Theme/state: dark theme, populated Modules directory and independently
  deployed Scoped Records Overview.
- Density normalization: device scale factor 1. The focused directory
  comparison scales the 6A-UI source to the 910 px implementation crop height;
  browser chrome and Codex workspace chrome are excluded from the comparison
  judgment.

## Required Fidelity Surfaces

- Fonts and typography: existing Tessara DM Sans/system treatments, weights,
  badges, machine-value typography, and compact metadata are retained. The
  Screen B Ready badge is explicitly non-wrapping.
- Spacing and layout rhythm: the directory is a contained standard data table
  with six tracks including the detail action; the inner table wrapper and
  document both measured zero horizontal overflow at 1394 px. Screen B
  measured two equal 492.5 px columns, and Definition rows resolve to a
  155.875 px label track plus a 290.625 px value track.
- Colors and visual tokens: only existing Tessara surface, border, text,
  primary, information, success, warning, and danger tokens are used.
- Image and icon fidelity: no raster product assets changed. The canonical
  Rust/UI `ChevronRight` icon is restored once per directory row.
- Copy and content: the Scoped Records description now matches the approved
  mockup; Definition, Lifecycle assessment, Diagnostics, and Artifact
  provenance use the approved label/value hierarchy. Existing tabs, actions,
  and surrounding application content remain intact.

## Comparison History

1. The earlier Sprint 6B1 implementation removed the 6A-UI row chevrons and
   incorrectly reported horizontal overflow fixed after measuring the outer
   directory container rather than the inner `.table-wrap`.
2. The correction restored one chevron per row and moved the directory onto
   the shared `DataTable` plus `TablePaginationFooter` pattern. The post-fix
   browser measurement at the narrower 1394 px viewport was
   `clientWidth = 999`, `scrollWidth = 999`, with eight rows and eight action
   links. Search reduced the result to one row and updated the footer to
   `Showing 1-1 of 1 module definitions`; clearing search restored all eight.
3. The initial Screen B correction exposed the correct two-column row layout,
   but its Ready badge wrapped in the narrow left panel. The badge was made
   non-wrapping; the post-fix measurement is 71.09 × 32 px with
   `white-space: nowrap`.

## Findings

- No actionable P0, P1, or P2 differences remain for the annotated directory
  and Screen B regressions.
- The Sprint 6B1 Serving state column is an intentional approved addition to
  the 6A-UI baseline, not visual drift.

## Primary Interactions Tested

- Filtered by `scoped`, confirmed one rendered result and the updated shared
  pagination summary, then cleared the filter.
- Activated the restored Scoped Records row chevron and confirmed navigation
  to `/administration/modules/tessara.reference.scoped-records`.
- Confirmed the detail route renders Definition, Lifecycle assessment,
  Diagnostics, and Artifact provenance with equal-width parent columns and
  side-by-side label/value rows.
- Checked in-app Browser logs after both routes. No errors were reported; the
  existing wasm initialization deprecation warning remains unrelated.

## Follow-up Polish

- None required for this annotation set.

final result: passed

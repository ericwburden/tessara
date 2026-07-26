# Sprint 6B2 mockup-to-live audit

Date: 2026-07-24

## Scope

This audit compares the approved Sprint 6B2 HTML/CSS review suite with the
authenticated local Sprint 6B2 stack at the same 1460 × 912 viewport.

Later browser-review decisions override the original prototype where they
conflict. In particular:

- the Roles table intentionally omits the Enrollment column;
- row-level duplicate open buttons remain removed;
- health cards intentionally use a distinct title plus plain status text
  rather than duplicate badges;
- create and edit remain separate routes.

## Findings before correction

| Step | Surface | Health | Evidence-based finding |
| --- | --- | --- | --- |
| 1 | Administrator enrollment | Needs correction | The live route is a minimally styled fallback. It omits the initial/recovery segmented choice, local/external identity choice, Core Administrator assignment summary, recovery explanation, and completed state from the approved flow. It also offers enrollment when a viable administrator already exists. |
| 2 | Roles and Capability Floor | Mostly aligned | The selected-role treatment now reflects the approved obligations and later review decisions. The page-level floor summary is visually much weaker than the approved floor banner and does not expose compliance/designation as scannable metadata. |
| 3 | Module configuration | Needs correction | The live Configuration tab flattens Application State into a text row. It omits the dedicated Application State panel, the separated configuration read/edit treatment, schema/validator fields, enablement explanation, and health/diagnostics action. |
| 4 | Records directory | Mostly aligned | Search, filtering, scope summary, authority, and primary row links are present. The directory omits the breadcrumb hierarchy and the New Record icon from the approved screen. |
| 5 | Record detail | Healthy | Two-column record and authorization context are present. Production data and full capability identifiers correctly differ from fixtures. The presenting-service value should identify the module receiving the decision, not Core. |
| 6 | Create record | Mostly aligned | The standalone route and manage-only organization options work. The form is materially narrower than the approved workspace, and the authority confirmation lacks the approved success treatment and icon. |
| 7 | Edit record | Mostly aligned | The standalone edit route and record identity are present. It has the same workspace-width and authority-confirmation mismatch as create. |
| 8 | Health | Mostly aligned | The revised heading/status hierarchy follows the later browser feedback. Refresh is available. The approved diagnostic-card icons are absent. |
| 9 | Diagnostics | Mostly aligned | Diagnostic context and findings match structurally. Refresh is missing on this tab, and the empty-findings confirmation icon is absent. |
| 10 | Denied/stale/disabled/degraded states | Needs correction | The approved bounded state treatments are not represented by equivalent module content. A read-only user opening a manage route receives a generic 404 instead of the designed non-disclosing denial treatment. Disabled and degraded route handling also collapses to generic Core errors. |

## Accessibility risks visible from the captures

- The live enrollment route has a long, undifferentiated form and does not
  reveal conditional recovery or identity-path context.
- Generic 404/error handling for authorization and module lifecycle states
  does not give users an actionable recovery path.
- Configuration and lifecycle state are compressed into a single prose row,
  making status relationships harder to scan.

Keyboard behavior, screen-reader announcements, zoom/reflow, and contrast
ratios require implementation-level verification beyond screenshots.

## Captured evidence

- `01-enrollment-mockup.png` / `01-enrollment-live-before.png`
- `02-roles-mockup.png` / `02-roles-live-before.png`
- `03-module-mockup.png` / `03-module-live-before.png`
- `04-records-mockup.png` / `04-records-live-before.png`
- `05-record-detail-mockup.png` / `05-record-detail-live-before.png`
- `06-create-mockup.png` / `06-create-live-before.png`
- `07-edit-mockup.png` / `07-edit-live-before.png`
- `08-health-mockup.png` / `08-health-live-before.png`
- `09-diagnostics-mockup.png` / `09-diagnostics-live-before.png`
- `10-states-mockup.png`

## Correction and acceptance pass

| Step | Surface | Health after correction | Verified outcome |
| --- | --- | --- | --- |
| 1 | Administrator enrollment | Healthy | The live route now presents initial/recovery claim kind, local/fixture identity paths, claim inputs, and the Core Administrator assignment summary. The server closes enrollment when its floor-compliant administrator query succeeds. |
| 2 | Roles and Capability Floor | Healthy | The page-level summary now uses the approved floor banner hierarchy with a compliance badge and designated-role metadata. The table keeps the later-approved three-column treatment. The Core Administrator detail retains enrollment designation, checked Floor v1 obligations, and the module-capability separation notice. |
| 3 | Module configuration | Healthy | Configuration and Application State are separate side-by-side panels. Schema version, display label, validator provenance, validation state, health, navigation, enablement explanation, edit mode, and the health/diagnostics action are present. |
| 4 | Records directory | Healthy | The Core shell owns navigation and the app bar; module content supplies the heading and route panel. Breadcrumbs, scope summary, search, Organization filter, authority labels, primary record links, pagination, and capability-gated New Record action are present. |
| 5 | Record detail | Healthy | Record and authorization context remain side-by-side. Presenting service now identifies `tessara.reference.scoped-records`; Core credential separation is explicit. |
| 6 | Create record | Healthy | The standalone create route uses the approved wider workspace, manageable Organizations only, success-styled authority confirmation, and bounded form actions. |
| 7 | Edit record | Healthy | The standalone edit route uses the same corrected form treatment while preserving record identity and detail-route cancellation. |
| 8 | Health | Healthy | Breadcrumbs, refresh, tabs, distinct card headings, plain status values, and supporting descriptions are present. The later review decision against duplicate badges is preserved. |
| 9 | Diagnostics | Healthy | Breadcrumbs and Refresh status are now present alongside diagnostic context, revision values, findings, and sanitized download. |
| 10 | Denied/stale/disabled/degraded states | Healthy with bounded evidence | Disabled/degraded product routes and read-only manage routes now render non-disclosing module content inside the Core shell with an actionable next route. Unit coverage verifies disabled, enabled, and manage-denied rendering. A live disabled/degraded screenshot was intentionally not forced because doing so would mutate the shared deployment state. |

## After-state evidence

- `01-enrollment-live-after.png`
- `02-roles-live-after.png`
- `03-module-live-after.png`
- `04-records-live-after.png`
- `05-record-detail-live-after.png`
- `06-create-live-after.png`
- `07-edit-live-after.png`
- `08-health-live-after.png`
- `09-diagnostics-live-after.png`
- `compare-02-roles.png`
- `compare-03-module.png`
- `compare-04-records.png`
- `compare-06-create.png`
- `compare-09-diagnostics.png`

## Evidence limits

- The mockup uses fixture labels and narrower fixture scope; the live stack uses
  current database records and assignments. Those content differences are
  expected and were not treated as visual regressions.
- The original prototype's Enrollment column, duplicate row-open buttons, and
  duplicate health badges were superseded by later annotated decisions.
- State-treatment HTML is covered by focused tests, but deployment state was
  not changed solely to obtain screenshots.

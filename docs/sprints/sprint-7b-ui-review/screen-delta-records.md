# Sprint 7B Screen Delta Records

Status: product-owner approved on 2026-08-04; frozen implementation contract.

## Screen and state inventory

| Screen | Material states | Desktop 1280 | Tablet 768 | Mobile 390 | Dark/light |
|---|---|---:|---:|---:|---:|
| Dashboard editor | healthy, changed in place, successor, inactive, archived, tombstoned, deferred, action confirmation/result | Required | Required | Required | Both |
| Dependency sheet | filters, issue detail, refresh, Defer, Upgrade, Replace, Remove | Required | Required | Required | Both |
| Component Versions | active, inactive, archived, tombstoned, published, superseded | Required | Required | Required | Both |
| Lifecycle confirmation | deactivate/activate, archive, tombstone | Required | Required | Required | Both |
| Canonical SDK shell | Core and Dashboard route ownership with identical chrome | Required | Required | Required | Both |

Long Component names, version labels, revision markers, notes, action groups,
and narrow table containment are part of the review matrix.

## Dashboard editor

### Current deployed pattern

The current Dashboard editor settings disclosure, toolbar, 12-column canvas,
placement geometry, save behavior, and product-content treatment are preserved
from the deployed application. Its compact module shell is transitional and is
replaced by the canonical SDK shell defined below.

### Proposed additions

- Add **Dependency health** beside Components and Placement details. It shows a
  semantic issue/healthy summary and opens the dependency sheet.
- Add a restrained semantic border/tint to an affected placement and replace
  its normal content glyph with one accessible issue button. The issue glyph
  uses the same 31-pixel slot and horizontal alignment as unaffected placement
  glyphs; semantic color, not different geometry, distinguishes it.
- Add an end-aligned dependency sheet containing:
  - overall health and issue count;
  - Needs review, Deferred, and Healthy filters;
  - explicit refresh;
  - a compact finding list and selected finding detail;
  - provider revision, placement, observation time, typed reference, and
    Dashboard-owned impact;
  - quick **Defer** with no note field;
  - **Upgrade**, **Replace**, and **Remove** actions where eligible.
- Keep Upgrade visually primary only when the provider declares an active
  published successor of the same Component.
- Use a confirmation dialog for Upgrade, Replace, and Remove. Replace exposes
  authorized renderable ComponentVersion choices. Every dialog previews the
  atomic placement/finding/receipt result.
- After Upgrade, Replace, or Remove, close the issue and show healthy dependency
  summary. Deferral retains degraded health and changes only the finding state.
- Render the editor inside the same canonical SDK shell as Component Versions,
  with Dashboards active and `Edit Dashboard` as the route title.
- Keep the Dependency health count as the sole toolbar summary; do not repeat a
  prose health note beside it.

### Explicitly preserved

- No dedicated dependency route or workspace.
- No manual Mark resolved control.
- No Dashboard product-layout redesign, new navigation information architecture,
  canvas resizing, or silent automatic rebinding.
- Existing Details, Preview, Save, settings, placement ordering, and canvas
  interactions remain unchanged.
- Viewer refresh remains read-only and is not represented as persistence in the
  prototype.

### Responsive behavior

The sheet fills the narrow viewport while preserving a visible close control,
horizontal filter containment, readable finding detail, and reachable actions.
The underlying editor retains its vertical product controls while adopting the
same responsive SDK-shell collapse and mobile menu as Core.

## Canonical SDK shell

- One policy-neutral shell implementation in `tessara-module-ui` is the only
  presentation owner for Core and independently rendered Dashboard documents.
- The visual target is the richer deployed Core chrome: Tessara icon and
  wordmark, full context navigation, account-aware/global search area, theme,
  notifications, help, desktop sidebar geometry, and responsive mobile menu.
- Core supplies shell policy and verified Shell Context; the SDK renders the
  chrome. Route owners supply only active destination, route title, and product
  content.
- Dashboard must not retain its compact 240-pixel sidebar, text-only brand,
  two-link navigation, 64-pixel header, or parallel responsive shell.
- Production must contain no Core-local or Dashboard-local parallel shell
  renderer in the touched shell dependency cone.

## Component Versions

### Current deployed pattern

The Core shell, route panel, breadcrumb, Component heading, Edit/View actions,
Versions heading, table container, density, responsive menu, and horizontal
table containment are preserved.

### Proposed changes

- Rename the existing Status column to **Publication** so publication and
  lifecycle are visibly separate.
- Add a **Lifecycle** column with Active, Inactive, Archived, and Tombstoned
  semantic labels.
- Add a compact vertical-dot **Actions** menu using the established application
  row-action pattern. Its eligible commands are:
  - Active → Deactivate or Archive;
  - Inactive → Activate or Archive;
  - Archived → Tombstone;
  - Tombstoned → no lifecycle action.
- Show both the current published version and a superseded version so the
  separate publication/lifecycle model can be reviewed.
- Activate/deactivate use the same compact confirmation treatment as existing
  actions. Archive and Tombstone confirmations carry stronger irreversible
  guidance; Tombstone uses the destructive action color.
- No transition asks for a reason.

### Explicitly preserved

- No new Component route or dependency workspace.
- No lifecycle action for drafts.
- No reactivation from archived and no action from tombstoned.
- Canonical SDK shell appearance, theme menu intent, navigation, table style,
  Edit/View actions, and mobile table scroll remain unchanged.

## Prototype-only controls

- The bottom-right **7B** navigator.
- The **Review finding state** selector inside the 7B navigator. It swaps mock
  provider states for review and does not filter or mutate product data.
- The review-theme switch inside the 7B navigator.

These controls must never appear in production or UAT acceptance selectors.

## Approval boundary

Approval authorizes the canonical SDK-shell convergence and the additions and
changes above. Production UI must be
compared against the approved prototype at matching route, state, theme,
viewport, density, and content. Any material deviation requires product-owner
review rather than silent interpretation.

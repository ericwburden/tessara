# Sprint 6C UI Screen Delta Records

Status: proposed for product-owner review on 2026-07-26. Approval covers only
the recorded deltas. Unlisted Tessara shell, component, route, responsive, and
interaction behavior remains unchanged.

## Baseline Sources

- Sprint 5A production captures under
  `docs/audits/sprint-5a-ui-review-2026-07-13/`.
- Approved Sprint 5A Dashboard mockups under `docs/mockups/`.
- Sprint 6B2 Module Management captures and approved screen deltas.
- Production tokens and components from `style/main.css`.
- Product behavior from `docs/architecture.md`,
  `docs/sprints/sprint-6c-plan.md`, and current Dashboard sources.
- Runnable review prototype under `prototype/`.

## Screen 1: Dashboard Module Configuration

Route class: bounded reuse of the independently deployed Module detail.

Baseline preserved:

- Module Management heading, breadcrumb, Definition identity, deployment and
  health badges, descriptor/receipt actions, tabs, and mobile selector;
- module-owned configuration remains separate from enablement, navigation,
  product-route availability, health, and deployment state;
- existing configuration and application-state card anatomy.

Proposed additions and changes:

- present the Dashboard Module Instance as independently deployed;
- show Dashboard schema version, display label, default page size, validation
  result, and authoritative Dashboard-owned validator;
- add Application state readback for configuration, health, navigation,
  product-route enablement, and Component adapter compatibility;
- add a compact transition-binding note that names the first-party Core Release
  adapter, external-Blueprint restriction, and Sprint 8A migration requirement;
- link to Dashboard health and diagnostics.

Explicitly unchanged:

- deployment remains outside Core mutation UI;
- configuration does not imply enablement;
- Components is not represented as a Module Instance;
- no Dashboard product data is shown in Core Module Management.

## Screen 2: Dashboard Health And Diagnostics

Route class: independently deployed module administration reached from Module
Management.

Baseline preserved:

- Sprint 6B2 health/readiness/diagnostics card patterns;
- sanitized download and refresh actions;
- stable status language with no secrets, Core credentials, browser cookies,
  signed grants, or reusable authorization.

Proposed additions and changes:

- show Dashboard readiness, liveness, isolated database connection, and Core
  authorization-exchange freshness;
- add a dedicated Components compatibility-dependency card;
- show binding key, contract ID, Core-installation provider identity, declared
  metadata/render actions, compatibility result, and last check time;
- present findings and revision information without exposing restricted
  Component resources.

## Screen 3: Dashboard Editor Placement Degradation

Route class: bounded state additions inside the existing Dashboard editor.

Baseline preserved:

- existing Dashboard builder heading, save state, actions, settings row,
  Components/Placement details controls, 12-column canvas, placement geometry,
  drag behavior, and save workflow;
- the placement remains selected and retains its saved footprint;
- healthy placements remain visually unchanged.

Proposed additions and changes:

- tint the affected placement's entire saved canvas footprint with its
  resolution-state color;
- replace the placement's normal content icon with a warning icon, keeping
  diagnostic copy out of the editor canvas;
- open a side sheet from the warning icon with the full authorized diagnostic
  message and recovery action;
- support authorized states for Components unavailable, inactive,
  superseded, provider-resource tombstoned, owner Module Instance
  tombstoned/data destroyed, missing, incompatible, and not evaluated;
- use a stable non-disclosing **Placement unavailable** treatment for
  unauthorized resolution;
- direct authoring remediation to the existing Placement details surface;
- allow retry only for transient unavailable/not-evaluated states;
- preserve the saved typed reference until the author explicitly replaces or
  removes it.

Explicitly unchanged:

- no automatic rebinding to a newer ComponentVersion;
- no lifecycle semantics are invented by Dashboard;
- no new editor mode or separate repair wizard;
- the side sheet is transient issue detail and does not change the saved layout
  or create a second placement-management model;
- the dashed placement-state selector in the prototype is review tooling only.

## Screen 4: Dashboard Viewer Placement Degradation

Route class: bounded state additions inside the existing Dashboard viewer.

Baseline preserved:

- current viewer heading, breadcrumb, placement count, responsive placement
  grid/cards, embedded Component chrome, fullscreen affordance, and healthy
  content rendering;
- resolved placements remain interactive while another placement is degraded.

Proposed additions and changes:

- contain resolution failure to the affected placement card;
- show one status badge, concise explanation, and optional retry;
- show detailed lifecycle/compatibility/provider state only after
  authorization;
- show the same non-disclosing copy for known and random unauthorized
  ComponentVersion references;
- add a concise containment note and diagnostics link when at least one
  placement is degraded.

Explicitly unchanged:

- the whole Dashboard does not become unavailable because one placement fails;
- no placeholder chart/table is fabricated for an unresolved Component;
- the dashed placement-state selector in the prototype is review tooling only.

## Screen 5: Dashboard Module Unavailable

Route class: Core-rendered same-origin fallback for Dashboard product routes.

Baseline preserved:

- authenticated Core shell, Dashboard navigation placement, normal breadcrumb,
  Tessara status language, and Module diagnostics destination;
- Core and unrelated product routes remain usable.

Proposed additions and changes:

- state that the Dashboard module cannot currently be reached;
- state that Dashboard data remains in its Module Instance database;
- name the protected boundaries: no provider substitution, no forwarded Core
  credentials/browser cookies, and no lost configuration/reference state;
- provide **Try Dashboards again** as the primary action and
  **Open Module diagnostics** as the secondary action;
- show only safe last-known Module Instance and route context.

## Placement State Vocabulary

- **Restricted:** no resource identity or lifecycle detail; open action omitted.
- **Provider unavailable:** retry and diagnostics; Dashboard remains usable.
- **Inactive:** retain or explicitly replace through Placement details.
- **Superseded:** informational; no automatic rebinding.
- **Provider-resource tombstoned:** replace or remove while retaining diagnosis.
- **Owner Module Instance tombstoned/data destroyed:** explicit irreversible
  owner state after authorization.
- **Missing:** authorized provider returned no matching resource.
- **Incompatible:** provider reachable but contract version unsupported.
- **Not evaluated:** no decision available; retry for a current decision.

## Responsive Contract

- Desktop: preserve the fixed sidebar, top header, two-column Module cards,
  12-column editor canvas, and two-column viewer.
- Tablet: narrower sidebar, stacked Module cards where required, readable
  dependency diagnostics, and contained placement copy.
- Mobile: compact top bar, hidden desktop sidebar, full-width Module selector,
  stacked placements, state copy below the placement heading, and full-width
  recovery actions where needed.
- All widths: keyboard-visible focus, semantic status treatment, no hidden
  persistent action, no horizontal page overflow, and useful no-JavaScript SSR
  copy for degraded states.

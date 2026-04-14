# Tessara UI Direction

This document defines the active UI direction for Tessara. It keeps the durable information architecture and screen guidance from earlier UI planning, but aligns future work to the target `Dataset -> Component -> Dashboard` model.

## Delivery Rule

Every future sprint is a full vertical slice.

- Every sprint must deliver both underlying functionality and usable application UI.
- The application must remain in a user-testable condition in the intended end-user-facing shape after each sprint.
- Backend-only or builder-only completion does not satisfy roadmap completion.

This rule is shared with [roadmap.md](./roadmap.md).

## Primary Information Architecture

The application should continue to use a single coherent shell with role-aware navigation across these main areas:

- Home
- Organization
- Forms
- Responses
- Datasets
- Components
- Dashboards
- Administration
- Migration

Guiding rules:

- product-facing areas should read as real application destinations
- internal/operator areas should stay available but not define the tone of the whole app
- IDs and workbench-style shortcuts should not be required for common user-testing flows
- the shell should respect system theme by default while allowing explicit Light and Dark overrides through shared shell chrome

## Surface Model

### Product-facing surfaces

- Home
- Organization
- Forms
- Responses
- Dashboards

Datasets and Components may have product-grade viewers, but authoring is primarily internal/operator-oriented in v1.

### Internal/operator surfaces

- Administration
- Migration
- dataset authoring
- component authoring
- access and role-assignment management
- workflow/materialization monitoring

Internal surfaces should still feel like part of the same application, but remain visually and structurally subordinate to the core product journey.

## Home Strategy

Home should remain a shared entry surface that can support different user roles without route-tree fragmentation.

Home should provide:

- current context
- readiness or system summary where useful
- clear entry points into the user's next likely tasks
- obvious distinction between product destinations and internal areas
- shared theme controls that belong to the shell rather than individual page actions

## Rendering And Hydration Rules

- Default to SSR-first route delivery with progressive enhancement.
- Keep core route state in the URL whenever practical so read-heavy surfaces remain useful even if hydration fails.
- Prefer native links and forms where they preserve workflow clarity; client-side enhancement should improve the experience, not become the only way the page works.
- Keep the shared shell light. Navigation, titles, breadcrumbs, and core layout should load immediately without depending on heavy lazy chunks.
- Treat browser hydration errors as release-blocking defects.

## Lazy Loading Guidance

Lazy loading is for heavy, low-frequency operator widgets and richer analytics viewers, not for core shell/navigation or ordinary browse/detail pages.

Do not lazy-load by default:

- Home
- Organization browse/detail flows
- Forms browse/detail flows
- Responses browse/detail flows
- shared navigation, shell chrome, auth/session bootstrap, and theme controls

First-class route/widget candidates:

- `/app/migration`
- administration capability/scope management grids once they become larger and more interactive
- future dataset/component authoring routes
- dashboard viewer enrichments, chart renderers, JSON/fixture editors, large preview/result tables, and drilldown/inspector panels

Use islands selectively for these kinds of widget-level enhancements on otherwise read-heavy pages. Islands are not the whole-app architecture for the current migration phase.

## Screen Families

### 1. Home / workspace

Used for shared entry and role-aware orientation.

### 2. Directory

Used for browseable lists of users, roles, organization nodes, forms, datasets, components, and dashboards.

For scoped hierarchy areas, directory screens should not default to a flat card wall. Where users are traversing assigned hierarchy branches, prefer a full-width hierarchy navigation pattern with clear parent/child expansion and selection behavior.

### 3. Detail

Used for calm inspection of one asset or record with related dependencies and next actions.

### 4. Editor / builder

Used for controlled authoring of forms, fields, datasets, components, dashboards, roles, and assignments. Editors should be task-focused rather than exposing a generic workbench.

### 5. Completion / review

Used for respondent-facing response completion and read-only review.

### 6. Viewer

Used for rendered end-user-facing outputs such as dashboards and component-backed tabular or visual views.

## Product And Internal Boundaries

- Organization, Forms, and Responses should behave like first-class product areas.
- Administration should hold powerful configuration work, but should not remain the only route to core authoring flows.
- Migration should remain clearly operator-focused and visually subordinate to the primary application.
- User management and RBAC should live in internal/admin surfaces, but they must still be application-grade UI, not hidden tooling.

## Hierarchy Navigation Direction

Organization browsing should become more scope-aware and less generic.

- When a user's highest assigned scope is `Partner`, the primary destination should read as `Partner List` rather than a generic `Organization List`.
- Higher-level scoped hierarchy screens should present the assigned tree structure directly instead of flattening everything into disconnected cards.
- Capability bundles and scope assignments in Administration should use accessible data-grid layouts once those surfaces need to support larger data sets.

When tabular interaction is required, prefer an accessible data-grid pattern over a static table so keyboard navigation, row/column focus, and dense editing behavior remain coherent.

## Target Asset Language

Target UI language should move to:

- `Dataset`
- `Component`
- `Dashboard`

Do not plan new future-state screens around separate `Report`, `Aggregation`, or `Chart` asset families.

Preferred future authoring/viewing split:

- Datasets: authoring + detail + preview
- Components: authoring + publish/version detail + viewer
- Dashboards: composition + viewer

## Transitional UI Constraints

The current app still exposes transitional reporting concepts in code and screens:

- report list/detail/edit flows
- aggregation configuration and execution paths
- chart-specific builder and viewer paths

Until the transition is complete:

- existing report/aggregation/chart routes may remain in service if needed for user testing
- new planning should describe them as transitional, not final
- new screen work should avoid deepening the old model unless needed to preserve a usable application between sprints
- the retained JavaScript controller is a temporary bridge and should be tracked route-by-route until each bridged surface has a native Leptos replacement

## Immediate UI Implications For The Roadmap

The next UI work should directly support the roadmap sequence:

- user management and authentication screens
- RBAC and role-assignment screens
- organization management flows
- form, field, and version authoring screens
- response assignment/start/review flows
- dataset and component authoring in the new model

At every stage, the app should remain usable through the intended shell rather than regress into internal-only builder behavior.

## Visual And Naming Guidance

Use [brand-design.md](./brand-design.md) for:

- product naming and copy tone
- palette and CSS token guidance
- icon and wordmark usage
- brand rules that affect shell styling and UI presentation

The shell-level theme selector should:

- appear in the upper-right of the shared hero in both the application shell and the legacy admin shell
- offer `System`, `Light`, and `Dark`
- follow system theme by default
- persist explicit user choice between sessions

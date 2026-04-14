# Tessara User Interface Design

## Purpose

This document defines the target UI shape for Tessara during the current UI catch-up phase.

It is guided by:

- `D:\Projects\dms-migration\tessara\docs\provenance\System Requirements Specification.pdf`
- `D:\Projects\dms-migration\tessara\docs\provenance\Software Design Document.pdf`
- `D:\Projects\dms-migration\tessara\docs\provenance\Requirements Traceability Matrix - Sheet1.pdf`
- legacy Django templates reviewed from:
  - `https://github.com/ericwburden/mmi-dms/tree/main/app/mmi/templates`

This is not a directive to reproduce the old UI exactly.

The goal is to preserve the old system’s functional structure and recognizable user journeys while improving clarity, reducing utility-panel friction, and fitting Tessara’s newer backend model:

- configurable hierarchy instead of fixed partner/program/activity/session tables
- versioned forms
- dataset-first reporting
- aggregations/charts/dashboards as first-class reporting assets
- migration/operator tooling as an internal surface, not a primary end-user surface

## Key Findings From Provenance

### Legacy use cases that must shape the UI

The previous application was organized around a few stable ideas:

1. Login is the only public entry point.
2. Users land on a role-appropriate home screen.
3. Home screens are not generic dashboards; they are working entry points.
4. Management is primarily list-driven:
   - manage partners
   - manage programs
   - manage activities
   - manage sessions
   - manage forms
   - manage reports
5. Records expose contextual actions directly from the list:
   - edit
   - manage children
   - assign forms
   - view entries
   - open dashboards
   - open reports
6. Client and parent users primarily need:
   - forms to complete
   - completed forms to review
7. Reports and dashboards are top-level product surfaces, not hidden admin tools.
8. Breadcrumbs and back-navigation matter because the old app is hierarchical and drill-down oriented.

### Relevant screen families in the old app

From the Software Design Document and the legacy templates, the old UI was built from these screen families:

- login
- MMI home
- partner home
- client home
- parent home
- manage list pages
- add/edit forms
- form assignment
- complete assigned form
- view completed form entry
- reports menu
- view report
- dashboard screen

### Important template patterns in the old app

The legacy templates consistently use:

- a persistent top navigation bar
- page title plus contextual actions
- breadcrumb navigation
- search near the page header
- list cards or tabular lists with per-record action icons
- create actions in the page header
- strong distinction between “manage” pages and “view/use” pages

These patterns should influence Tessara’s structure even when the exact components differ.

## Design Position

Tessara should feel like:

- a role-aware operational application
- with real home screens
- list/detail management areas
- dedicated completion and review screens
- dedicated report and dashboard viewing surfaces

Tessara should not feel like:

- a raw operator console
- a single internal workbench with every control exposed at once
- a builder-first shell that expects users to know IDs

## Design Principles

1. Preserve the legacy information architecture where it helps user orientation.
2. Do not preserve legacy friction just for familiarity.
3. Prefer selection-driven flows over ID-driven controls.
4. Prefer list/detail/task layout over giant builder forms with mixed responsibilities.
5. Separate “use the system” pages from “configure the system” pages.
6. Surface datasets and aggregations where they matter, but do not let them dominate end-user workflows.
7. Keep migration and operator tooling clearly internal.
8. Only design screens that the current backend can support.

## Proposed Application Structure

## Global Shell

Tessara should move toward a single consistent application shell with:

- branded top bar
- primary navigation
- optional role badge / scope indicator
- user/session menu
- contextual breadcrumb
- page title row
- page-level primary actions

### Primary navigation

Recommended top-level navigation:

- Home
- Organization
- Forms
- Responses
- Reports
- Dashboards
- Administration
- Migration

Notes:

- `Migration` should be internal-only.
- `Administration` should only expose configuration surfaces relevant to the current user.
- `Reports` and `Dashboards` may remain separate visually even if they share backend assets.

## Home Screens

The old app’s strongest structural idea is role-specific home pages. Tessara should keep that.

### 1. MMI / system-admin home

Purpose:

- orient administrative users
- show organization summary
- provide direct entry into partner/program-like hierarchy browsing
- surface reporting and dashboard shortcuts

Primary content:

- hierarchy summary cards
- recently active nodes/programs
- form publishing status
- recent submissions
- report/dashboard quick access
- creation shortcuts for internal users

### 2. Partner/operator home

Purpose:

- provide an operational view of the organization slice the user is responsible for
- prioritize programs/nodes, assigned forms, responses, and dashboards

Primary content:

- scoped node/program list
- actionable forms
- recent submissions
- relevant dashboards
- quick links to reports

### 3. Client/parent/respondent home

Purpose:

- show pending forms first
- keep completed responses visible but secondary

Primary content:

- forms to complete
- completed forms / submitted responses
- child/context selector if applicable
- clear “resume draft” / “view submitted” affordances

## Main Product Areas

## Organization

This area replaces the old partner/program/activity/session drill-down concept with Tessara’s hierarchy model while preserving the same mental flow.

Recommended structure:

- list of top-level organizational nodes
- detail page for a selected node
- child-node navigation
- scoped forms
- scoped dashboards
- associated submissions/responses

Key requirement:

- it should feel like “browse and manage the organization tree,” not “edit hierarchy internals.”

## Forms

This area should merge the old “manage forms” and “complete form” concepts into two distinct surfaces:

### Forms directory

For internal users:

- browse forms
- inspect form details
- see where forms are assigned/scoped
- see published versions
- open form builder/editor

### Form completion

For respondents and staff acting on behalf of respondents:

- open assigned or relevant published form
- complete current draft
- review submitted responses in read-only form

## Responses

This area should absorb the old client/parent form list and the old “view form entry” model.

Recommended structure:

- response inbox / pending queue
- drafts queue
- submitted queue
- response detail
- read-only submitted view

Key rule:

- submitted responses should clearly become read-only
- draft actions should remain prominent only when the response is editable

## Reports

This area should preserve the old “reports menu” and “view report” pattern while adapting to the new dataset-first backend.

User-facing reporting surface:

- report directory
- report detail
- run report
- filter/sort if supported
- export/share actions when implemented

Internal reporting configuration surface:

- dataset detail
- report definition detail
- aggregation detail
- chart detail

Important rule:

- datasets and aggregations should be obvious to internal/report-builder users
- but they should not overwhelm end-users who only need to open a report

## Dashboards

This area should preserve the old dashboard-as-destination pattern.

Recommended structure:

- dashboard directory
- dashboard detail / preview
- clear route from dashboard to underlying report or aggregation
- component-level drill-through when supported

The dashboard page should feel like a viewing surface first and a configuration surface second.

## Administration

This area should contain configuration and builder workflows that are currently spread across utility controls.

Recommended subsections:

- hierarchy types
- nodes
- forms and versions
- reporting assets
- choice-list-like assets if and when they are reintroduced
- users/permissions when supported

This area should use:

- list/detail/task flows
- contextual edit panels
- strong breadcrumbs

It should avoid:

- giant mixed-purpose forms
- requiring users to manually copy IDs between tasks

## Migration

This should remain an internal/operator area.

Recommended content:

- fixture intake
- validation results
- dry-run results
- import execution
- verification links into datasets, reports, and dashboards

This should not become part of the primary end-user navigation for normal users.

## Screen Patterns To Reuse

### 1. Home/workspace screen

Use for:

- MMI home
- partner home
- respondent home

Pattern:

- page header
- search or scope selector
- summary cards
- primary work queues
- shortcut actions

### 2. Directory screen

Use for:

- organization lists
- forms
- reports
- dashboards
- admin assets

Pattern:

- breadcrumb
- title
- create action
- search/filter row
- list of records
- row/card-level contextual actions

### 3. Detail screen

Use for:

- node detail
- form detail
- report detail
- dashboard detail

Pattern:

- breadcrumb
- title and metadata
- related assets
- next-step actions
- status summary

### 4. Editor / builder screen

Use for:

- form version editing
- reporting asset setup
- hierarchy type configuration

Pattern:

- clearly bounded task area
- current context panel
- preview/validation panel
- save/publish actions

### 5. Completion screen

Use for:

- respondent data entry

Pattern:

- title and context
- sectioned form
- inline validation
- save draft
- submit
- read-only submitted mode

## Mapping Legacy Screens To Tessara

| Legacy concept | Tessara target |
|---|---|
| Login screen | Login screen |
| MMI home | Admin/system home |
| Partner home | Scoped organization home |
| Client/parent home | Respondent home / responses inbox |
| Manage partners/programs/activities/sessions | Organization directory + node detail flows |
| Manage forms | Forms directory + form detail + builder |
| Form assignment | Form targeting / assignment surface where supported |
| Complete assigned form | Response completion screen |
| View form entry | Response detail read-only screen |
| Reports menu | Report directory |
| View report | Report viewer |
| Dashboard screen | Dashboard viewer |
| Manage reports/charts/aggregations | Internal reporting asset management area |

## Where Tessara Should Intentionally Diverge

These are recommended improvements over the old product:

1. Replace most icon-only actions with labeled actions or mixed icon+label affordances.
2. Use list/detail layouts instead of scattering actions across unrelated screens.
3. Make current context explicit:
   - selected node
   - selected form
   - selected version
   - selected report
4. Make respondent flows calmer and narrower than internal admin flows.
5. Keep reporting usage separate from reporting configuration.
6. Keep migration internal and clearly segmented from normal product usage.
7. Treat datasets and aggregations as internal reporting concepts, not the primary language of the end-user product.

## Immediate UI Work Plan

This plan is intentionally constrained to existing backend support.

### Phase 1: Shell and navigation alignment

- establish the final top-level application shell
- add consistent breadcrumb + title + action rows
- distinguish Home, Organization, Forms, Responses, Reports, Dashboards, Administration, Migration
- ensure each current route fits into that shell coherently

### Phase 2: Real home screens

- replace placeholder home surface with role-aware home layouts
- create:
  - admin/system home
  - scoped organization home
  - respondent home

### Phase 3: Directory/detail conversion

- convert current `/app/admin` and `/app/reports` utility clusters into:
  - directory/list panels
  - detail panels
  - contextual tasks
- focus first on:
  - node types / nodes
  - forms / versions
  - reports / dashboards

### Phase 4: Response experience

- make `/app/submissions` a proper inbox/workspace
- emphasize:
  - pending forms
  - drafts
  - submitted responses
  - read-only completed view

### Phase 5: Reporting and dashboard viewing

- make report and dashboard routes feel like viewing surfaces
- move builder/configuration actions farther into internal/admin contexts

## Current Backend Constraints

The UI should not outrun the backend.

That means:

- no invented permission flows beyond what is implemented
- no fake assignment model beyond current supported behavior
- no polished dashboard/report interactions that imply unsupported filtering or sharing
- no complex role switching unless backed by actual auth/session support

Where the old app had a mature workflow but Tessara’s backend is not there yet, the UI should:

- preserve the destination and structure
- mark the internal path clearly
- avoid suggesting that the full workflow is already complete

## Acceptance Criteria For “UI Caught Up”

The UI can be considered caught up enough to resume broader roadmap work when:

1. The app has a recognizable top-level shell with stable navigation.
2. The main routes look like product pages, not internal control panels.
3. Home screens reflect actual user jobs.
4. Forms, responses, reports, and dashboards are discoverable without knowing IDs.
5. Admin/configuration areas use list/detail/task flows instead of giant mixed input surfaces.
6. The structure clearly reflects the legacy product’s use cases, even where the interaction design is improved.

## Review Focus

The next review should focus on:

- whether this information architecture is the right translation of the legacy product
- whether `Organization`, `Forms`, `Responses`, `Reports`, `Dashboards`, and `Administration` are the right primary areas
- whether Tessara should expose separate role homes immediately or stage them behind one shared home first
- whether any legacy management areas are missing from this plan that you consider essential to preserve early

# Tessara UI Direction

## Purpose

This document is the implementation-direction document for Tessara UI work during the current UI catch-up phase.

It translates the broader analysis in [user-interface-design.md](./user-interface-design.md) into a practical direction for upcoming UI implementation.

Use the two documents differently:

- `user-interface-design.md`: analysis, provenance review, and target-shape rationale
- `ui-direction.md`: locked direction for near-term UI implementation

## Locked Direction

The following decisions are now fixed for the near-term UI direction:

- Tessara will use split product areas as the primary information architecture.
- Tessara will stage one shared home shell first, with role-aware home variants added later.
- Administration and Migration will remain visible in the direction, but they are internal/operator surfaces.
- UI terminology will be configuration-driven, not hardcoded to legacy entity names.
- Tessara should reflect the legacy product's use cases and structural mental model, but it should not copy the legacy page implementations exactly.

## Primary Information Architecture

The near-term top-level product areas are:

- Home
- Organization
- Forms
- Responses
- Reports
- Dashboards
- Administration
- Migration

Responsibilities:

- `Home`: primary entry point and orientation surface
- `Organization`: browse and manage the hierarchy in operational terms
- `Forms`: form directory, form detail, and form configuration entry points
- `Responses`: pending, draft, submitted, and reviewable response workflows
- `Reports`: report directory, report detail, and report viewing
- `Dashboards`: dashboard directory, dashboard detail, and dashboard viewing
- `Administration`: internal configuration for hierarchy, forms, and reporting assets
- `Migration`: internal/operator workflows for validation, dry-run, import, and verification

Product-facing areas:

- Home
- Organization
- Forms
- Responses
- Reports
- Dashboards

Internal/operator areas:

- Administration
- Migration

## Home Strategy

The immediate implementation target is one shared home shell.

Requirements for that shell:

- it must work as the primary home implementation now
- it must be structured to accept role-aware modules later
- it must not force separate route trees for each role yet

Future role-home variants the shell must be able to support:

- admin/system
- scoped operator/partner
- respondent/client/parent

Immediate rule:

- do not hardwire the UI around separate home routes yet
- build one shared home shell with role-ready content zones

## Terminology Strategy

Navigation and route structure should use generic product areas.

That means:

- use `Organization` as the structural area
- avoid hardwiring the shell around `Partner`, `Program`, `Activity`, and `Session`

Page copy, headers, breadcrumbs, cards, and contextual labels should support configured domain terminology.

That means:

- a scoped organization screen may present configured labels such as Partner, Program, Activity, or Session
- the structure remains Tessara-native even when the copy reflects legacy-friendly labels

## Screen Families

The UI should converge toward these core screen families:

### 1. Home / workspace

Expected parts:

- page header
- title row
- summary cards
- work queues
- shortcut actions

### 2. Directory

Expected parts:

- breadcrumb
- title row
- primary create action
- search/filter row where supported
- list of records
- row or card-level contextual actions

### 3. Detail

Expected parts:

- breadcrumb
- title and metadata
- related records/assets
- next-step actions
- status or summary panels

### 4. Editor / builder

Expected parts:

- bounded task area
- current context panel
- preview or validation area where supported
- save/publish actions

### 5. Completion / review

Expected parts:

- title and context
- sectioned form or response content
- inline validation where supported
- save draft / submit / read-only completed behavior

### 6. Viewer

Expected parts:

- breadcrumb
- title row
- view controls relevant to the asset
- clear distinction from configuration actions

Across all families:

- giant mixed control surfaces are transitional and should be removed over time
- common workflows should not depend on knowing IDs

## Current-to-Target Route Mapping

Current route mapping:

- `/app` -> shared home
- `/app/submissions` -> Responses
- `/app/admin` -> temporary umbrella for Organization, Forms configuration, and Administration until split
- `/app/reports` -> temporary umbrella for Reports, Dashboards, and reporting asset configuration until split
- `/app/migration` -> Migration

Direction for future UI work:

- split by target area inside the existing shell first
- improve internal route structure before adding new product breadth
- do not add new top-level workflow areas until the current routes have been reorganized into the target IA

## Implementation Priorities

Near-term UI sequence:

1. Stabilize the shared shell, top navigation, breadcrumbs, and page-title/action rows.
2. Convert the current focused routes into directory/detail/task patterns.
3. Make Home, Responses, Reports, and Dashboards read like product surfaces.
4. Keep builder/configuration actions in internal areas.
5. Add richer role-home differentiation only after the shared shell and core directories are stable.

Constraint:

- UI work must not outrun backend support

That means:

- no speculative permission model
- no unsupported assignment/sharing flows
- no UI promises for backend capabilities that do not exist yet

## Out Of Scope For This Direction

The following are out of scope for this direction document:

- speculative permissions work beyond current backend/auth support
- fake assignment or sharing workflows
- hard commitment to legacy visual styling beyond structural inspiration
- separate role-home implementation as an immediate requirement

This document requires architectural readiness for role-specific homes, not immediate route-level implementation of them.

## Acceptance Criteria For UI Work Guided By This Document

UI work is aligned with this direction when:

1. Navigation reflects the split product areas.
2. Main routes read as product destinations, not control consoles.
3. IDs are no longer the primary interaction method on common workflows.
4. Organization, Forms, Responses, Reports, and Dashboards are discoverable without backend knowledge.
5. Internal surfaces remain available but clearly scoped.
6. The UI can present legacy-friendly terminology through configured labels rather than hardcoded structure.

## Important References

This document should guide future UI work across:

- [application.rs](D:\Projects\dms-migration\tessara\crates\tessara-web\src\application.rs)
- [shell_style.rs](D:\Projects\dms-migration\tessara\crates\tessara-web\src\shell_style.rs)
- [user-interface-design.md](./user-interface-design.md)

## Review Criteria

This direction document is complete when it:

- resolves the major product choices without leaving implementation decisions open
- distinguishes structure from terminology
- distinguishes product-facing areas from internal/operator areas
- provides a direct path from the current route set to the target information architecture
- constrains UI work to currently supported backend capabilities

## Assumptions And Defaults

- `Shared home first` means one primary home implementation with role-ready content zones, not a permanent single-home model.
- `Configured labels` means UI vocabulary can reflect legacy entities where configuration supports it while the structural information architecture remains generic and Tessara-native.
- Administration and Migration remain part of the direction because they already exist in the product and support real internal workflows, but they should not dominate normal end-user navigation.
- This document is intentionally shorter and more directive than [user-interface-design.md](./user-interface-design.md); it is a UI implementation charter, not a discovery memo.

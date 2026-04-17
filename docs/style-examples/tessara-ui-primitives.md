# Tessara UI Primitive Examples

These examples capture the first shared `tessara-ui` primitives introduced during Sprint 1G.

They are not a replacement for [ui-guidance.md](../ui-guidance.md). Use them as implementation-oriented references when building SSR-first route markup.

## Page Header

- Eyebrow: short uppercase route context
- Title: page-level heading
- Description: one concise route summary
- Actions: grouped page-level actions only
- Metadata strip: compact route state such as mode, surface, and loading or read-only state

## Panel

- Use panels for substantive route sections
- Default structure:
  - title
  - optional description
  - body content
- Use the header variant when the section needs an action row such as `Add Binding`

## Card

- Use cards for concise summary or navigation blocks
- Current shared usage:
  - home-area navigation cards
  - directory summary tiles
  - compact status cards inside list or detail routes

## Field Wrapper

- Label above control
- Shared control family for text, select, and textarea inputs
- Optional helper text below the control
- `wide-field` spans both columns in the current form grid

## Toolbar

- Two-zone container for filters, search, and compact table actions
- Current shared usage:
  - scope and delegation filters on user-access screens
  - capability filter on role edit screens

## Initial Shared Surface Coverage

- shared home navigation cards and action group
- shared page-header and panel shells across list, detail, and form screens
- form metadata and authoring sections for forms, reports, dashboards, users, roles, and node types
- response-start and access-management filter surfaces

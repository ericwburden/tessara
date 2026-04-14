# Tessara Repo Bundle

This repository-style bundle remains a reference handoff for the Tessara re-alignment materials.

The active planning and design authority now lives under [`/docs`](../README.md). Historical planning files from this bundle were moved to [`archive/docs/re-alignment`](../archive/docs/re-alignment).

## Current Architecture
`Dataset → Component → Dashboard`

## Structure
- archived planning/design docs live in `../archive/docs/re-alignment/`
- `db/` schema and migration starters
- `rust/` workspace planning and type skeletons

## Major Decisions
- Dataset absorbs former Report responsibilities
- Component replaces Charts and includes:
  - DetailTable
  - AggregateTable
  - Bar
  - Line
  - Pie/Donut
  - StatCard
- Dashboards are mutable and not versioned
- Datasets have internal immutable `DatasetRevision`s
- Components are versioned
- Future printable reports are separate artifacts composed from prose + Components

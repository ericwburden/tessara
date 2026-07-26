# Sprint 6C Dashboard Extraction Inventory

Status: frozen implementation inventory, 2026-07-26.

This inventory records the pre-extraction Dashboard surface. It is the boundary
for relocating Dashboard behavior without silently changing product semantics.

## Product Routes

- `GET /dashboards`
- `GET /dashboards/new`
- `GET /dashboards/{dashboard_id}`
- `GET /dashboards/{dashboard_id}/edit`
- `GET /dashboards/{dashboard_id}/view`

The routes currently originate in `tessara-api`, create Dashboard-specific SSR
bootstrap data, and hydrate through `tessara-web-dashboards`.

## JSON API

- `POST /api/admin/dashboards`
- `GET /api/admin/dashboards/visibility-nodes`
- `PUT /api/admin/dashboards/{dashboard_id}`
- `DELETE /api/admin/dashboards/{dashboard_id}`
- `GET /api/admin/dashboards/{dashboard_id}/composition`
- `PUT /api/admin/dashboards/{dashboard_id}/composition`
- `GET /api/dashboards`
- `GET /api/dashboards/{dashboard_id}`

The public DTO and idempotent full-composition reconciliation behavior are
compatibility constraints for the extracted service.

## Core Database Objects To Extract

- `dashboards`
- `dashboard_scope_nodes`
- `dashboard_components`
- `dashboard_components_dashboard_position_idx`
- `dashboard_components_capacity_chk` enforcement function and trigger
- Dashboard placement layout/capacity migration preflight
- Dashboard seed rows, visibility bindings, placements, and idempotency inputs

The extracted placement row must replace the relational
`component_version_id -> component_versions(id)` foreign key with an
installation-scoped, `core_installation`-owned typed reference.

The extracted visibility row stores Organization node identities as values. It
must not have a foreign key or runtime join to Core `nodes`.

## Dashboard-Owned Rust Surfaces

- `crates/tessara-dashboards`: composition, placement configuration, grid
  policy, and the Sprint 6C transition ComponentVersion reference wrapper
- `crates/tessara-api/src/dashboards`: transport, DTOs, errors, repository,
  projection, reconciliation, scope, native SSR adaptation, and service policy
- `crates/tessara-web-dashboards`: Dashboard route bootstrap, API client,
  directory, create, detail, editor, viewer, and shared Dashboard web types
- `crates/tessara-web/src/routes/dashboards.rs`: current Core route mounting
- `crates/tessara-web/src/state/navigation.rs`: current static Dashboard
  navigation fallback

## Core And Components Dependencies To Remove

Dashboard currently relies on Core-process behavior for:

- cookie/session authentication;
- capability and Organization-scope expansion;
- Organization visibility-node labels and ancestry;
- native shell construction;
- direct `nodes` joins and foreign keys;
- direct `components` / `component_versions` joins and foreign keys;
- Component metadata and availability projection;
- reverse dependency queries from Dataset and Organization screens;
- demo seed orchestration and app-summary counts.

Sprint 6C replacements are:

- signed, audience/action-bound Core authorization grants;
- signed `ShellContextV1`;
- module-local security-state and Organization projection;
- typed transition ComponentVersion references;
- the first-party Core Components compatibility adapter;
- API/event-style dependency projections instead of cross-database joins.

## Security Capabilities

- `dashboards:read`
- `dashboards:manage`

The two capabilities remain independent. Extraction must not reintroduce the
historical manage-implies-read shortcut.

## Preserved Product Constraints

- fixed 12-column, 240-row grid;
- maximum 240 placements;
- component-kind minimum geometry;
- deterministic movement reflow;
- full-layout atomic reconciliation;
- retained placement IDs and idempotent client-key mapping;
- exact ComponentVersion binding without automatic repinning;
- visibility scope filtering;
- restricted placement nondisclosure;
- useful SSR and responsive behavior.

## New Module-Owned Operations

- versioned manifest and Dashboard release identity;
- configuration validation and applied configuration;
- private security-state projection;
- readiness and liveness;
- sanitized health and diagnostics pages;
- isolated database migration and runtime roles;
- transition Components dependency diagnostics;
- same-origin native product routes and JSON API.

## Test Inventory

- `crates/tessara-api/tests/dashboard_composition.rs`
- `crates/tessara-api/tests/dashboard_ssr.rs`
- `crates/tessara-api/tests/component_dashboard_compatibility.rs`
- `crates/tessara-api/tests/fixtures/historical/dashboard_placement_capacity_preflight.sql`
- `crates/tessara-web-dashboards` unit/SSR tests
- canonical Dashboard Playwright coverage under `end2end`
- Sprint 6B2 module, shell, authorization, outage, database-isolation, Compose,
  bootstrap, smoke, UAT, and provenance patterns

Tests move with their owning boundary. Core retains only gateway, manifest,
grant-exchange, compatibility-adapter, and unavailable-fallback coverage.

## Known Extraction Edges

- Dataset and Organization reverse-dependency screens currently query
  Dashboard tables directly. They need a bounded projection or a temporarily
  explicit unavailable/omitted state; they cannot retain cross-database joins.
- Core app-summary Dashboard counts currently read the Dashboard table and must
  stop doing so.
- Core demo seed currently constructs Dashboard rows after Component seed. The
  Sprint 6C bootstrap must seed Dashboard through the module database/API and
  typed transition references.
- Current Dashboard SSR bootstrap consumes Core-owned account DTOs. The module
  must instead derive the minimal Dashboard account projection from verified
  grants and shell context.

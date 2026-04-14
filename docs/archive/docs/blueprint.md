# MMI-DMS Rewrite Blueprint  
**Target Architecture: Rust Backend + Leptos Frontend**

---

## 🔗 Source Repository (Legacy System)
This rewrite is based on the existing Django application:

👉 https://github.com/ericwburden/mmi-dms

This blueprint is **not a migration plan**, but a **domain-driven redesign** inspired by the current system.

---

# 1. Purpose

Build a new platform that preserves the core product intent while replacing:

- hard-coded hierarchy (MMI → Partner → Program → Activity → Session)
- Django view-driven architecture
- polymorphic ORM-heavy design
- tightly coupled reporting logic

### The new system should support:

- dual-surface UX (admin + external users)
- configurable hierarchy per deployment
- strongly typed metadata schemas
- versioned forms with draft support
- strict submission validation
- flexible, configuration-driven reporting
- compatibility-aware aggregation across form versions
- materialized analytics layer
- DataFusion for reporting/aggregation (not app logic)

---

# 2. Guiding Principles

## 2.1 Domain redesign, not framework translation
Do NOT recreate:
- Django class-based views
- screen-oriented routing
- polymorphic model inheritance

## 2.2 Separate OLTP from analytics
- transactional layer = correctness + workflows  
- analytical layer = reporting + aggregation

## 2.3 Explicit configuration > implicit behavior
Everything should be inspectable:
- hierarchy
- metadata schemas
- form versions
- compatibility groups
- reporting rules
- permissions

## 2.4 Validation at workflow boundaries
- drafts = flexible  
- submissions = strict

## 2.5 Reporting must tolerate evolution
- version-aware
- compatibility-controlled
- resilient to schema drift

---

# 3. High-Level Architecture

## Backend
- Rust
- `axum`
- `sqlx`
- PostgreSQL (OLTP)
- DataFusion (analytics)
- background jobs for projections

## Frontend
- Leptos (SSR-first)
- typed UI + shared models

## Layers

```text
Domain → Application → Infrastructure → API → UI
```

---

# 4. Product Model

## Dual-surface system

### Admin Surface
- hierarchy config
- metadata schema builder
- form builder
- report builder
- dashboard builder
- permission management

### External Surface
- browse nodes
- complete forms
- manage drafts
- view reports/dashboards

---

# 5. Bounded Contexts

## 5.1 Identity & Access

Replaces polymorphic user types.

### Core Entities
- Account
- Role
- Capability
- PermissionGrant

### Features
- RBAC default
- scoped overrides (node / node-type / global)

---

## 5.2 Hierarchy Configuration

### Core Entities
- NodeType
- NodeTypeRelationship
- NodeMetadataFieldDefinition

### Key Features
- configurable per deployment
- strict parent/child rules
- schema-driven metadata

---

## 5.3 Hierarchy Runtime

### Core Entities
- Node
- NodeMetadataValue

### Features
- tree structure
- validated metadata
- ancestry traversal

---

## 5.4 Form Definition Engine

### Core Entities
- Form
- FormVersion
- FormSection
- FormField
- ChoiceList
- CompatibilityGroup

### Key Design Changes
- no hard-coded scopes
- forms attach to node types or nodes
- field types via enums (not polymorphism)

---

## 5.5 Submission Engine

### Core Entities
- FormAssignment
- Submission
- SubmissionValue

### Lifecycle
- draft
- submitted

### Rules
- drafts = partial + autosave
- submission = strict validation
- tied to form version
- immutable after submission (initially)

---

## 5.6 Reporting Engine

### Core Entities
- Report
- ReportFieldBinding
- Aggregation

### Key Features
- compatibility-aware reporting
- per-field missing-data policies
- logical field abstraction

### Missing Data Policies (examples)
- null
- default value
- exclude row
- exclude from aggregation
- map alternate field
- bucket as "Unknown"

---

## 5.7 Visualization Engine

### Core Entities
- Chart
- Dashboard
- DashboardComponent

### Features
- layout system
- chart config
- reusable components

---

# 6. Data Model (Conceptual)

## Identity
- accounts
- roles
- capabilities
- role_capabilities
- account_role_assignments
- permission_overrides

## Hierarchy
- node_types
- node_type_relationships
- node_metadata_field_definitions
- nodes
- node_metadata_values

## Forms
- forms
- form_versions
- compatibility_groups
- form_sections
- form_fields
- choice_lists
- choice_list_items

## Submissions
- form_assignments
- submissions
- submission_values
- submission_value_multi
- submission_audit_events

## Reporting
- reports
- report_field_bindings
- aggregations
- aggregation_groupings

## Dashboards
- charts
- dashboards
- dashboard_components

---

# 7. Analytics Architecture

## Strategy
Materialize reporting data instead of querying raw structures.

## Layers

### OLTP (Postgres)
- normalized transactional data

### Analytics Schema
- fact tables
- dimension tables
- compatibility mappings

### DataFusion
- executes queries
- powers reports and charts

---

## Recommended Tables

- `analytics.node_dim`
- `analytics.form_dim`
- `analytics.form_version_dim`
- `analytics.field_dim`
- `analytics.submission_fact`
- `analytics.submission_value_fact`
- `analytics.compatibility_group_dim`

---

## Refresh Model
- background jobs
- incremental refresh later
- start in same DB → split later if needed

---

# 8. API Design

## Auth
- `/api/auth/login`
- `/api/me`

## Nodes
- `/api/nodes`
- `/api/nodes/:id`

## Forms
- `/api/forms`
- `/api/form-versions`

## Submissions
- `/api/submissions`
- `/api/submissions/:id/submit`

## Reports
- `/api/reports/:id/table`

## Dashboards
- `/api/dashboards/:id`

## Permissions
- `/api/admin/roles`
- `/api/admin/permissions`

---

# 9. UI Architecture (Leptos)

## Main Areas
- Admin
- Management
- External User

## Core Page Types
- List pages
- Entity editors
- Form builder
- Report builder
- Dashboard builder
- Submission workflow

---

# 10. Validation Strategy

## Draft Validation
- allow incomplete data
- allow partial progress
- basic type-safe writes

## Submission Validation
- required fields
- type checks
- choice validation
- selector constraints
- cross-field rules

---

# 11. Permission Model

## Model
- roles bundle capabilities
- scoped grants
- explicit overrides

## Scope Levels
- global
- node type
- specific node

## Notes
- RBAC first
- fine-grained override optional
- inheritance down tree unless blocked later

---

# 12. Implementation Plan

## Phase 0: Discovery
- inventory current behaviors
- inventory field/report/chart logic
- identify required workflows

## Phase 1: Core backend foundation
- workspace/crates
- DB migrations
- auth
- roles/capabilities
- node type + node core

## Phase 2: Hierarchy
- hierarchy config
- metadata schema builder
- node CRUD
- node validation

## Phase 3: Forms
- form families
- form versions
- sections/fields
- choice lists
- publish/version flow
- compatibility groups

## Phase 4: Submissions
- assignments
- drafts
- autosave
- submit transition
- audit events

## Phase 5: Analytics
- analytics schema
- refresh jobs
- harmonization pipeline
- fact/dim tables

## Phase 6: Reporting
- report definitions
- field bindings
- missing-data rules
- DataFusion execution

## Phase 7: Dashboards
- chart configs
- dashboard configs
- layout editor
- preview

## Phase 8: Later Enhancements
- import/export tools
- deployment migration helpers
- AI-assisted compatibility suggestions
- tags/cross-links

---

# 13. Suggested Rust Project Structure

```text
crates/
  api/
  app-core/
  auth/
  hierarchy/
  forms/
  submissions/
  reporting/
  dashboards/
  analytics/
  db/
  jobs/
  shared-types/
  web-ui/
```

### Responsibility Split

- `app-core`: shared primitives and errors
- `auth`: accounts, roles, authorization
- `hierarchy`: node types, nodes, metadata
- `forms`: forms, versions, fields, compatibility groups
- `submissions`: drafts, assignments, submission lifecycle
- `reporting`: definitions, policies, execution planning
- `analytics`: projections + DataFusion integration
- `dashboards`: charts and dashboard logic
- `db`: repositories, migrations, SQL
- `jobs`: refresh/maintenance jobs
- `web-ui`: Leptos frontend

---

# 14. Do Not Copy from Legacy App

Do NOT preserve these patterns:

- Django class-based screen-per-action architecture
- hard-coded scope branching
- polymorphic ORM inheritance as the main model
- model-method-centric aggregation/report generation
- view-layer orchestration of dynamic field entry behavior

---

# 15. Preserve Product Intent

Preserve these intentions from the existing system:

- hierarchical organizational data
- configurable forms with sections and typed fields
- assignment + completion workflows
- reporting + aggregation
- charts + dashboards
- scoped access by user role and affiliation

---

# 16. First Recommended Milestone

Build one vertical slice proving the new architecture:

- accounts + roles
- configurable node types
- nodes with metadata schema
- one form family with versioning
- draftable submission flow
- one analytics projection
- one compatibility-aware report
- one dashboard/chart

Do not start by trying to port the whole app.

---

# 17. Final Summary for Codex

This rewrite should produce a new system that:

- replaces the fixed Django hierarchy with a configurable tree
- replaces polymorphic forms/entries with explicit typed models
- supports versioned forms and drafts
- keeps submission validation strict
- handles cross-version cleanup in the reporting layer
- materializes analytics tables for reporting
- uses DataFusion for analytics, not app workflows
- supports RBAC with optional scoped overrides
- provides separate admin and external-facing UX

This is a **new system design inspired by** the legacy repo, not a literal migration.

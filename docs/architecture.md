# Tessara Architecture

This document defines Tessara's target technical architecture and the transition from the current single-service implementation. The product-level module contract and terminology are defined in [modular-application-platform.md](./modular-application-platform.md).

## Architectural Direction

Tessara is moving from a modular monolith to a platform for composing separately deployed applications from Core and selected full-stack modules.

Target installation topology has an out-of-band control/materialization plane and a request/data path:

```text
Installation Supervisor + authoritative ledger
  |-- materializes Core Release (Core application + gateway component)
  |-- materializes selected Module Releases
  `-- provisions identities/databases in the installation's single PostgreSQL cluster

Browser -> same-origin gateway and coherent shell
  |-- Core application --------> Core database
  |-- Forms module ------------> Forms database
  |-- Workflows module --------> Workflows database
  |-- Responses module --------> Responses database
  |-- Datasets module ---------> Datasets database
  |-- Components module -------> Components database
  `-- Dashboards module -------> Dashboards database
```

An application installation may omit any module that its use case does not require. It may add application-specific modules for batch operations, external interfaces, specialized data-collection instruments, or other capabilities. Every module instance serves one application installation.

The normal deployment unit is a full-stack module with its own product or administrative UI, APIs, configuration contract, security capabilities, health and diagnostics, migrations, and database. The platform does not introduce a separate headless-module or content-package class.

## Current Implementation Baseline

The current codebase is a useful transition baseline:

- one Axum application serves APIs and Leptos SSR UI
- `tessara-web` owns the shell, route adapters, authentication policy, document integration, hydration entrypoint, CSS, and public assets
- focused `tessara-web-*` and domain crates separate several feature areas at compile time
- product data currently shares one PostgreSQL database
- current feature routes and DTOs are largely registered at the root application

These are descriptions of current implementation, not target deployment constraints. Feature crates are extraction seams. They should first acquire explicit manifests and contracts, then move behind module-owned APIs and routes, and finally into independent processes and databases.

No production consumer depends on the current internal database layout. The transition may therefore restructure tables, references, and route ownership directly when doing so establishes the target boundary cleanly.

`transitional_in_process` contribution descriptors reserve discovery metadata and possibly a future Module Definition identity, but create no Module Release or Module Instance. If an extracted first-party module must consume a still-in-process provider, it binds to an explicitly versioned Core Release compatibility contract. Resources returned by that adapter remain `core_installation`-owned with transition-specific types. Provider extraction later creates a real Module Instance and performs an explicit old-to-new data/reference migration through provider mappings, consumer-owned rebinding contracts, and completeness receipts; references are never silently reinterpreted.

## Platform Components

### Core

Core owns only installation-wide concerns:

- application-level use of the Supervisor-anchored installation identity
- Organization and organization-scope evaluation
- authentication, users, sessions, delegation, and service identity
- security-capability registry, roles, role assignments, and RBAC audit
- the versioned Core Administration Capability Floor and validation of the Blueprint-designated Administrator Enrollment Role
- coherent application shell
- module registry and manifest validation
- module installation, configuration status, enablement, health, and diagnostics orchestration
- dynamic navigation policy and semantic destination resolution
- Application Blueprints, lockfiles, release inventory, and installation receipts

Core does not own Forms, Workflows, Responses, Datasets, Components, Dashboards, or future module product data.

### Installation supervisor

The Installation Supervisor is a trusted local process with a bootstrap CLI that runs outside Core. It owns materialization for the complete installation:

- acquires and verifies locked `tessara-oci-v1` Core Release component images, including its gateway, and Module Release images
- creates the stable Application Installation identifier, trust anchors, and authoritative pre-Core Supervisor Ledger
- provisions the v1 cluster's Core and per-module databases and restricted identities
- sequences migrations, readiness checks, traffic switching, rollback, and recovery
- remains available while Core or the gateway is being replaced
- exposes a versioned local control/status contract and emits installation receipts
- issues installation-bound, single-use Administrator Enrollment Claims under the initial/recovery rules after the first Core Release becomes healthy

Core owns desired-state UI and orchestration, but hands a deterministic Materialization Plan plus a separate Apply Authorization Envelope to the Supervisor rather than attempting to replace itself. The envelope binds operation kind, installation identifier, current/base receipt, target or current plan digest, monotonic desired/apply revision, nonce/idempotency key, initiator and required approver evidence, expiry, and explicitly approved override or destructive actions. Core and Supervisor mutually authenticate; the Supervisor verifies the envelope, rejects replay/stale or concurrent-base plans, serializes accepted apply-contract mutation, and records the result in its ledger. Core reconciles that observed ledger/read-back after startup; it is not authoritative for whether materialization actually occurred. Administrator Enrollment Claim lifecycle and Supervisor self-upgrade use their separate protocols described below.

The first installation starts through the Supervisor CLI under local operator/host authorization. That CLI creates the installation root/ledger, then either runs the same versioned Composition Engine as Core over a Blueprint and trusted catalog inputs or verifies a detached signature over a pre-resolved lockfile/plan digest, constructs the first authorization envelope, and seeds Core with the resulting desired/resolved records. The locked composition designates an Administrator Enrollment Role and records the Core Release's versioned Core Administration Capability Floor; validation rejects a missing designation or a role that does not contain the floor.

Once Core is healthy, the Supervisor can issue an installation-bound, single-use, expiring Administrator Enrollment Claim only after a current Core decision says no Viable Core Administrator exists. An `initial` claim is allowed only until one has ever been established; a `recovery` claim additionally requires an audited break-glass authorization. A Viable Core Administrator is an active, authenticable identity with an active global role assignment covering the full capability floor. The Supervisor permits at most one issued or reserved claim generation and records `issued -> reserved -> consumed`, with expiry or revocation from either nonterminal state. Reissue revokes the prior generation. It persists a one-way verifier and signed non-secret metadata, never the once-displayed secret.

Redemption is explicitly idempotent across the process boundary. Core asks the Supervisor to reserve the current claim generation, then uses one local database transaction to create or bind the identity, assign the locked Administrator Enrollment Role globally, and record the claim generation/reservation as redeemed. A signed idempotent result lets the Supervisor finalize consumption; interrupted retries reconcile the same reservation rather than repeating the assignment. Core validates current Supervisor claim state for every attempt, so a Core restore predating redemption cannot reuse a consumed or revoked claim. Enrollment is unavailable while a viable administrator exists. Normal users and assignments remain outside the Blueprint and lockfile, and the claim secret is excluded from receipts, logs, diagnostics, status, audit, and recovery output. Planning or LLM access does not imply approval authority, and destructive operations require explicit non-replayable approval. The Supervisor's own signed binary/control-contract and ledger-schema upgrade runs through a separate bootstrap launcher with ledger backup, compatibility checks, health verification, and rollback, so it never self-replaces during an application apply. This is installation-local infrastructure and does not require a central SaaS control plane.

### Full-stack modules

A module owns one bounded product capability end to end:

- product routes and administration/configuration routes
- domain rules and product semantics
- public functional contracts
- namespaced security capabilities
- typed configuration validation
- persistence, migrations, retention, backup metadata, and recovery behavior
- readiness, health, compatibility, and diagnostics
- generated clients or schemas needed by consumers
- module-level and platform-conformance tests

A module may advertise no primary product navigation if it is used only by administrators or other modules, but it still exposes an administration, configuration, or diagnostics experience.

### Same-origin gateway and shell

The browser sees one origin and one coherent application. The gateway:

- resolves Core and module route prefixes for the current installation
- preserves ordinary full-page SSR navigation
- supplies modules with verifiable installation, actor, and scope-bound Authorization Grants or Core authorization decisions
- presents explicit disabled, unavailable, incompatible, and maintenance outcomes
- prevents one stopped module from taking down Core or unrelated modules
- serves a Core-owned fallback document when a target module cannot render its route

Core owns shell policy, navigation composition, and the versioned Shell Context supplied to a route owner. Each module uses the shared UI SDK and that authenticated context to server-render the complete HTML document, including coherent shell chrome, for its own routes. Core does not server-compose a remote content fragment into a separate document. Modules own their page content and behavior, while the Shell Context/SDK keeps policy, layout, theme, identity, and navigation coherent. The default architecture does not require iframes, browser-side remote module loading, or runtime WebAssembly composition.

## Module Contract

Each Module Release publishes a versioned manifest described in [modular-application-platform.md](./modular-application-platform.md). The manifest is part of the supported release artifact and advertises:

- identity, publisher, version, digest, and support metadata
- `tessara-oci-v1` runtime and optional migration-image digests, platform/architecture, runtime and migration commands, listen/service registration, configuration/secret injection points, runtime/migration identities, probes, graceful shutdown, and resource requests/limits
- Core Release and manifest compatibility
- supported Shell Context, UI SDK, and design-system contract versions
- required and optional functional dependencies plus any explicit provider-binding constraints
- machine-readable Feature Declarations describing use cases, inputs, outcomes, constraints, and their realizing contracts/contributions
- required and provided functional contracts
- owned resource types and reference schemas
- product, administration, configuration, and diagnostic destinations
- navigation contributions and ordering hints
- optional versioned Home/work-discovery and global-search contribution contracts with scope-bound grant, latency/failure, and semantic-result requirements
- namespaced security capabilities
- configuration and secret-reference schemas
- readiness, health, compatibility, and migration contracts
- optional module-owned bootstrap schema plus typed validation/apply/read-back and receipt contracts for reproducible catalogs or initial product records

Core validates the manifest and Deployment Profile compatibility before a module becomes ready or enabled. The Supervisor conformance suite proves the image can migrate, start, register, become ready, shut down, and stay within declared identity/resource rules. Feature Declarations, functional contracts, and security capabilities remain separate: discovery metadata does not prove provider equivalence, dependency resolution cannot grant permissions, and a security capability cannot satisfy a functional dependency.

State is scoped to its owner. Registered describes a stable Module Definition; trusted/compatible evaluate a particular Module Release within an installation; live/tombstoned identity, installed/deployed/configured/ready/enabled/healthy operation, and retained/destroyed data describe separate dimensions of a Module Instance; display policy and ordering describe a Navigation Contribution; authorization is evaluated for a particular actor, action or destination, resource, and capability-to-scope binding. These dimensions must not be collapsed into one installed, active, or authorized module flag.

## Dependencies And Inter-Module Calls

Module dependencies are declared against versioned functional contracts, not another module's tables or deployment URL.

Rules:

- dependencies may be required or optional
- Core validates dependency closure, version compatibility, cycles, and explicit bindings
- callers use generated or typed clients and stable error envelopes
- calls carry installation, service, actor, and audience-bound scope-bound grants or Core decision receipts as appropriate
- commands use idempotency keys where retries are possible
- long-running or fan-out operations expose durable job or reconciliation state
- cross-module workflows use retry and compensation rather than distributed transactions
- a module outage degrades only the functions that depend on that module

An application-specific module should reuse existing module contracts and add only the missing capability. It should not copy another module's data or implementation merely to avoid a declared dependency.

## Typed Resource References

Cross-boundary resource relationships use an installation-scoped typed reference:

```text
ResourceReference {
  installation_id
  owner_kind: core_installation | module_instance
  owner_id
  resource_type
  resource_id
}
```

The platform guarantees that the tagged owner and resource type cannot be silently reinterpreted. A `FormVersion` reference remains a reference to a resource of type `FormVersion` owned by the Forms module instance; an Organization-node reference can use the same envelope with Core-installation ownership.

The platform does not impose universal immutability or versioning policy. The owning provider (Core or a module) decides which changes mutate an existing resource, which require a new version, and how active, inactive, superseded, archived, deleted, or tombstoned states behave. Module product resources remain governed by module-owned product rules; Core governs only Core-owned resources. The provider contract states how consumers resolve and observe relevant resource state through live reads, revision markers, events, caches, or a declared combination.

Consumers own their reaction to provider state. A Workflow may reject an inactive FormVersion, a Dashboard may render a warning for a superseded ComponentVersion, or another consumer may continue using it. Those are consumer product rules, not reference-system rules.

Resolution has structured dimensions so lifecycle, owner history, authorization, compatibility, disclosure, and transient infrastructure remain distinct:

```text
ResourceResolution {
  access_state: authorized | unauthorized | not_evaluated
  owner_state:
      CoreOwnerState { live | unknown_core_installation | installation_mismatch }
    | ModuleOwnerState {
        instance: live | owner_module_instance_tombstoned | unknown_module_instance | owner_mismatch
        data: retained | owner_data_destroyed | unknown | not_evaluated
      }
    | undisclosed | not_evaluated
  resource_identity_state: resolved | unknown_resource | undisclosed | not_evaluated
  resource_lifecycle_state: provider_defined | undisclosed | not_evaluated
  contract_state: compatible | incompatible | undisclosed | not_evaluated
  provider_state: available | unavailable | undisclosed | not_evaluated
}
```

Provider-defined resource lifecycle may include inactive, superseded, archived, tombstoned, migrated, or retired. That product-resource tombstone is not the same as an `owner_module_instance_tombstoned` identity or `owner_data_destroyed` state, and clients must preserve the distinction. Core-owned references use the tagged Core owner state rather than Module Instance fields. Evaluated unknown or cross-installation owners use `unknown_*` or `*_mismatch`; `not_evaluated` is reserved for a dimension the resolver could not evaluate.

After installation and audience validation, authorization is evaluated before resource-specific disclosure. An unauthorized or authorization-`not_evaluated` result fails closed, uses one stable restricted envelope, and marks caller-visible owner, resource identity, and lifecycle `undisclosed`; detailed reasons are limited to separately authorized diagnostics. A provider that cannot evaluate a resource-specific dimension after authorization uses `not_evaluated`, never `unknown_resource`. A valid identifier and random identifier must be indistinguishable to a caller without existence-disclosure authority in status and response shape, and in timing under the conformance profile's defined measurement method and tolerance.

## Navigation

Inter-module links persist semantic destinations rather than URLs:

```text
SemanticDestination {
  owner_kind: core_installation | module_instance
  owner_id
  route_name
  typed_parameters
}
```

Core resolves these for the current installation and same-origin gateway. A destination may alternatively carry an explicit dependency-binding key that Core resolves to the installation's live module instance. V1 permits at most one live instance per Module Definition per installation; instance identity is still explicit so upgrades, tombstones, route resolution, deployment addresses, and process placement cannot silently reinterpret stored data. Supporting multiple live instances later requires instance-scoped RBAC/navigation rules.

The left navigation is computed from a schema-v2 Core catalog and installation policy containing ordered groups plus one exact placement for every known Core or module destination. `core.main` and `core.admin` are required groups. Custom groups have immutable UUID-backed identities and may be renamed, reordered, or deleted only while empty. Catalog flags determine whether a destination may be hidden or moved between groups; the policy cannot redefine destination labels, routes, capability requirements, ownership, availability, or protection. The actor shell projection then removes hidden, unavailable, and unauthorized destinations and omits empty groups without changing configured order. Hiding a destination does not disable its module or authorize a user, and route/API enforcement remains mandatory inside the owner. Module Management is the protected `core.admin.modules` destination in the default Admin group, discoverable with effective global `modules:read`; policy mutation requires effective global `modules:manage_navigation`. The `/administration` landing route and aggregate Administration shell item do not exist; the four Core administration destinations are direct shell entries.

## Security Architecture

Core is the source of truth for users, sessions, security capabilities, roles, role assignments, organization scope, and identity/access/composition audit. Each module remains the source of truth for its product audit and history. Modules advertise namespaced security capabilities such as `dashboards:view` or `forms:publish`; they do not publish authoritative roles.

An Administrator Enrollment Claim is a narrow pre-administrator or recovery credential, not a user session, role template, or deployment input. Core validates its installation, kind, generation, expiry, reservation, current Supervisor-ledger state, and Supervisor signature before its local redemption transaction. Expired, revoked, replayed, reserved-by-another-redemption, consumed, and cross-installation claims return the same non-disclosing failure class. Only claim identifier, generation, kind, lifecycle state, reservation, operator authorization, and outcome are auditable; the one-way verifier stays in the Supervisor Ledger and the secret is never persisted in Core-readable audit or support output.

For browser requests, Core authenticates the server-managed session and the gateway passes short-lived, audience-bound context to the target module. Authority is represented either by individual grants that bind one capability to global or explicit descendant-aware Organization roots, or by a Core authorization-decision receipt for the exact actor, presenting gateway or module service, declared route or dependency/contract action, resource, audience, and installation. Independent capability and scope arrays are prohibited because their cross product can over-authorize mixed-scope role assignments.

Core owns descendant expansion and attaches Organization and authorization revisions to grants and decisions. A module validates the relevant binding and expiry or asks Core for a fresh decision; it does not reconstruct Organization closure from product data. Short expiry bounds stale offline grants, and sensitive operations may require an online decision. Delegation, assignment ownership, and similar exceptions use explicit capability/resource-bound assertions.

For service-to-service calls, modules use dedicated service identity. A context intended for one module cannot be forwarded as downstream authority: the caller exchanges it through Core for the downstream audience, preserving original actor, installation, correlation, and delegation basis while binding the presenting service and declared dependency/contract/action. Core reevaluates current role, Organization, and caller-service authority. The provider validates both the actor's scoped authority and the presenting service's permission to call that contract. Service-only grants are restricted to explicitly authorized system jobs. Modules verify all received context and enforce their own authorization.

No module receives:

- another module's database credentials
- unrestricted access to Core identity or RBAC tables
- a shared browser bearer token
- permission solely because its navigation item was visible

Role templates, if supported, are advisory inputs that administrators explicitly adopt in Core. Capability identities remain namespaced and auditable even when a provider is disabled or absent.

## Persistence Architecture

V1 requires one PostgreSQL cluster per Application Installation with separate databases:

```text
cluster
  |-- tessara_core_<installation>
  |-- tessara_forms_<installation>_<instance>
  |-- tessara_workflows_<installation>_<instance>
  |-- tessara_responses_<installation>_<instance>
  |-- tessara_datasets_<installation>_<instance>
  |-- tessara_components_<installation>_<instance>
  `-- tessara_dashboards_<installation>_<instance>
```

Every database has module-specific runtime and migration roles. Network and credential policy prevents cross-module access.

Multi-cluster placement and module-selected external relational databases are deferred. This constraint applies to Core and module relational persistence; a future module contract may separately declare a non-relational external service when its product capability genuinely requires one.

Prohibited integration mechanisms:

- cross-database SQL or foreign keys
- foreign data wrappers used to bypass APIs
- shared writable schemas
- shared runtime credentials
- one module running another module's migrations
- Core or automation clients writing module product tables

Each record has one authoritative owner: Core for Core records or a module instance for module product records. Read models may cache provider data only under an explicit contract that defines revision or event behavior and recovery from missed updates. Cross-boundary invariants are verified through orchestration and reconciliation, not database constraints.

Module disablement or undeployment preserves the Module Instance identity and database binding. Reactivation of retained data reuses that identity; upgrades change the instance's selected Module Release without changing its identity. A new instance cannot inherit it. Explicit data destruction requires authorization and owner-controlled retention checks and leaves an instance tombstone so existing references cannot rebind silently.

## Application Composition

Application construction is declarative and reproducible:

```text
module catalog
    -> Application Blueprint
    -> validation and dependency resolution
    -> plan/diff
    -> Application Lockfile + deterministic Materialization Plan
    -> separate Apply Authorization Envelope
    -> idempotent Supervisor apply
    -> conformance and read-back
    -> installation receipt
```

The Blueprint includes a Core Release version constraint, typed Core configuration such as Organization schema/terminology, selected modules, module version constraints and desired enablement, dependency bindings, typed module configuration, optional module-owned bootstrap declarations whose inputs are normalized non-secret values or durable content-addressed references, navigation policy, Core-owned role definitions and role-to-capability mappings, a designated Administrator Enrollment Role, and references to environment secrets. It excludes users, role assignments, ongoing product records, and secret values. The Composition Engine rejects a designation that is missing or whose role does not cover the Core Release's Core Administration Capability Floor. The lockfile deterministically records the Blueprint revision/digest, exact Core Release version and component image digests including the gateway, exact Module Release image digests, selected Deployment Profile versions, resolved desired module enablement, composition-engine/schema and required Supervisor/deployment-adapter contract versions, contract/configuration/bootstrap schema versions, dependency bindings, resolved normalized non-secret Core/module configuration plus navigation and role policy values and their digests, the designated enrollment role and capability-floor version, normalized bootstrap values or durable content-addressed bootstrap references plus digests, versioned secret-reference identities, and the Materialization Plan plus digest. The plan contains enable/disable actions; approval is carried separately.

Apply runs in the out-of-process Installation Supervisor through its versioned deployment adapter. It verifies the deterministic lockfile/Materialization Plan and separate Apply Authorization Envelope against its ledger, then resolves only locked OCI images from configured trusted sources, verifies publisher/signature or equivalent provenance and digest, executes declared `tessara-oci-v1` migration/runtime commands under separate identities, injects configuration/secrets at declared points, registers services, health-gates the change, switches traffic, and records rollback state. Core UI/API, CLI, automation, and LLM clients all use the same handoff contract and read status/receipts from it; none relies on the Core process to replace itself. Artifact acquisition and runtime materialization are therefore part of the reproducibility contract, not handwritten operator steps.

Declared module bootstraps are validated, applied, read back, and checked for drift only through the owning module's typed, idempotent contract. Before apply, referenced inputs are acquired from a configured durable content-addressed source and verified against the locked digest. The receipt records module-specific results; adopting a later UI edit requires a new Blueprint/bootstrap input revision. This is the only Blueprint-managed product-record path and does not create a shared content-package abstraction.

Module enablement, configuration, navigation, role-definition, or Administrator Enrollment Role designation changes made through UI create a new desired Blueprint revision or appear as explicit drift with adopt/reconcile actions. Emergency disablement may apply immediately only through a constrained, non-destructive Apply Authorization Envelope recorded in the Supervisor Ledger; it is a named, audited, optionally expiring operational override and remains drift until adopted or reconciled. Installation receipts record desired and observed enablement/digests so a release cannot claim reproducibility while silently diverging from its lockfile. Reproduction uses the lockfile's resolved non-secret values plus externally resolved secret references.

Human administration, CLI tooling, deployment automation, and LLM clients use the same catalog, validation, plan, apply, and module-owned configuration APIs. An agent never needs direct database access or undocumented deployment glue to assemble an application.

## Reference Application Capability Flow

The current first-party product flow is the reference application, not the platform topology:

```text
Forms/Workflows -> Responses -> Materialized Sources -> Dataset -> Component -> Dashboard
```

The product responsibilities remain useful:

1. Forms owns fields, forms, form versions, catalogs, publication, and form lifecycle semantics.
2. Workflows owns workflow definitions, versions, steps, assignments, execution policy, and references to FormVersions.
3. Responses owns drafts, submissions, review, response persistence, and source/export contracts.
4. Datasets owns source composition, transformations, exposed contracts, revisions, and materialization.
5. Components owns presentation definitions, versions, execution/view behavior, and Dataset references.
6. Dashboards owns composition, placement, and ComponentVersion references.

### Current reference product rules

Current analytics behavior includes a thin Table Component over a Dataset major line and a mutable Dashboard with a fixed 12-column, 240-row grid and at most 240 placements. Existing DatasetRevision and ComponentVersion compatibility behavior remains product scope for the owning modules. It is not a universal platform rule.

The provider modules may retain current immutability or publish behavior where the product requires it. Future changes to those policies are made inside the owning module and exposed through its public contract; they do not require a platform architecture change.

### Current analytics artifact behavior

The following remains the active product contract until the owning module changes it through its own roadmap and versioned public contract.

Dataset currently owns:

- reusable row-level analytical identity
- source composition, joins, unions, latest/earliest reducers, row grain, filters, calculated fields, and exposed field contract
- a mutable logical `Dataset`
- immutable `DatasetRevision` records as a current Dataset product rule
- rebuildable materialized relations for performance

Component currently owns:

- versioned presentation over a Dataset major line
- a thin Table type with last-mile projection, one saved default filter set, display labels, default sort, page size, and viewer affordances
- chart and stat presentation types added by the current reference application

Analytical shaping, aggregation, grouping, and bucketing remain Dataset responsibilities rather than separate table-component backends unless those module requirements are deliberately changed.

Dashboard currently owns:

- mutable composition that refers to specific `ComponentVersion` resources
- a fixed 12-column, 240-row grid
- no more than 240 stored placements
- component-kind minimum geometry and the rule that a placement's bottom edge cannot exceed row 240

Current dependency behavior is module product policy:

- a separately published replacement does not automatically rebind an existing resource reference
- superseded `ComponentVersion` payloads are immutable
- the current published `ComponentVersion` may be updated in place by an authorized, intentional operation that preserves its id and updates consumers of that id
- archived or inactive resources remain resolvable where the owning module's current contract requires historical integrity
- changelog impacts use `major`, `minor`, and `patch` classifications for current Dataset/Component carry-forward workflows
- publication is blocked for empty Dataset revisions and compilation or materialization failures

Dataset major-line sources currently use an append-all contract. A source labeled `Version N` resolves to the prebuilt major-line materialization populated from published historical revisions in that major line. Minor and patch publishes rebuild it; a new major publish leaves prior-major consumers on their existing line. The materialization uses the latest published contract in the major line as its schema and projects `NULL` for fields introduced after older rows were produced.

These rules demonstrate the distinction between platform and module semantics: typed references preserve owner and resource type, while the Dataset and Component modules decide whether payloads are immutable, which operations preserve an id, and how publication affects consumers.

### Current shared relational baseline

The transition database still contains these table families together:

- Core candidates: `accounts`, `roles`, `capabilities`, `role_capabilities`, `role_assignments`, `account_delegations`, and `nodes`
- Forms: option, lookup, field, form, form-version, and field-placement tables
- Workflows: workflow, version, step, transition, assignment, and instance tables
- Responses: form-response and response-runtime tables
- Datasets: dataset, revision, source, and major-materialization tables
- Components: component and component-version tables
- Dashboards: dashboard and placement tables

This inventory is an extraction map, not permission for new cross-area relationships. As each feature becomes a module, its tables move into the module database and all remaining consumers switch to public contracts before direct access is removed.

### Current flat API baseline

Current root API families for users, roles, role assignments, Organization, fields/options/lookups, Forms, Workflows, Responses, Datasets, Components, and Dashboards remain transitional adapters. A feature's module extraction moves the canonical endpoints, schemas, authentication enforcement, and diagnostics to that module. Core may retain a temporary compatibility adapter, but new consumers must bind to the advertised module contract.

## Frontend And SDK Direction

The shared design system and module SDK must let independently deployed modules remain visually and behaviorally coherent. The SDK should provide:

- shell integration and route metadata contracts
- Shell Context validation plus bounded Home/work-discovery and global-search contribution adapters that keep Core from reading module product data directly
- authenticated request-context verification
- manifest and configuration handling
- `tessara-oci-v1` manifest validation and Supervisor deployment-profile bindings
- typed navigation and resource-reference helpers
- stable errors and standard unavailable states
- common SSR-compatible UI primitives, tokens, accessibility behavior, and assets
- health, readiness, diagnostics, and conformance-test support
- contract schema generation and typed clients

The current `cargo-leptos` pipeline, `tessara-web`, `tessara-web-ui`, `tessara-web-http`, and feature crates remain the implementation baseline until the module runtime exists. New code should avoid deepening root dependencies that would make a feature harder to extract.

## Operational Requirements

Every supported application release must provide:

- observed Composition Engine and Installation Supervisor/deployment-adapter versions compatible with the lockfile
- an exact Core Release/component (including gateway) and Module Release version/artifact inventory
- compatibility and dependency validation
- independent module health and aggregate installation health
- contained disabled, unavailable, and maintenance states
- per-module and whole-installation backup/restore, including protected Supervisor Ledger/trust material, lockfiles/receipts, databases, and required external bootstrap inputs
- Core Release component (including gateway) and Module Release migration, upgrade, health-gated traffic switch, rollback, and failed-migration recovery
- diagnostic bundles with configuration redaction and correlation identifiers
- connection budgets, timeouts, and noisy-neighbor monitoring for the shared cluster

The installation must remain locally operable without a mandatory central Tessara SaaS service.

## Transition Sequence

1. Represent current in-process feature areas through explicitly non-installable `transitional_in_process` contribution descriptors and Core module inventory; do not call them Module Releases or Module Instances.
2. Introduce semantic destinations, namespaced capability contributions, typed references, and explicit module state.
3. Add the out-of-process Installation Supervisor/bootstrap CLI, same-origin gateway, authenticated module context, per-module database provisioning, SDK, and conformance suite.
4. Extract one existing full-stack feature, beginning with Dashboards, behind real module boundaries.
5. Add deterministic Blueprint, lockfile, plan, apply, and read-back tooling.
6. Prove cross-module scope and resource lifecycle behavior.
7. Extract Components, Datasets, Responses, Workflows, and Forms into module-owned processes and databases.
8. Harden migration, upgrades, backup/restore, diagnostics, and multiple independently supported application compositions.

This sequence deliberately establishes the platform contract before many applications depend on the current shared structure.

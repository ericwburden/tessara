# Tessara Requirements

This document consolidates active product and system requirements. Platform-wide module semantics are defined in [modular-application-platform.md](./modular-application-platform.md); technical boundaries are defined in [architecture.md](./architecture.md).

## Product Purpose

Tessara must provide composable building blocks for creating consistent, reliable applications without rebuilding common capabilities for every use case.

An application must be able to combine Core with only the full-stack modules its use case requires. New application development should reuse existing modules and add narrowly scoped modules for missing functionality, including potential batch operations, external interfaces, specialized data-collection instruments, and other application-specific capabilities.

The product must support multiple independently deployed and independently supported application installations. A centralized, monolithic SaaS is not a required operating model.

Human administrators, deployment automation, and LLM clients must be able to discover, validate, configure, and compose applications through supported, machine-readable contracts.

## Shared User Experience

- Each application installation must present one coherent same-origin experience even when its modules run as separate applications.
- Core must own the authenticated shell and compose permanent Core destinations with eligible module navigation contributions.
- Core must publish a versioned authenticated Shell Context; each route-owning module must use the shared UI SDK and that context to server-render a complete coherent HTML document rather than relying on remote-fragment wrapping, iframes, or browser-loaded module code.
- The gateway must serve a Core-owned fallback document that preserves shell/navigation context when a module cannot render.
- Administrators must be able to choose which module destinations appear in left navigation and adjust their order or grouping.
- Navigation visibility, module enablement, module health, and user authorization must remain separate concepts.
- Product destinations normally require an enabled module instance; administration, configuration, and diagnostics destinations must remain recoverable for an installed module that is disabled, unconfigured, or unhealthy.
- Hiding navigation must not grant, revoke, or substitute for authorization.
- Modules must enforce access on routes, APIs, commands, and resources independently of shell behavior.
- Cross-module navigation must use named semantic destinations and typed parameters rather than stored deployment URLs.
- Disabled, unavailable, unconfigured, unauthorized, incompatible, and unknown destinations must have distinct user-facing outcomes after authorization; an unauthorized direct request must not reveal whether the destination is known.
- Product and administrative module surfaces must use the shared design-system and accessibility contract.

## Core Requirements

Core must own:

- application-level use of the Supervisor-anchored installation identity
- Organization and configurable hierarchy
- authentication, users, sessions, delegation, and service identity
- security capability registration
- roles, role capabilities, role assignments, organization scope, and access audit history
- the versioned Core Administration Capability Floor and Administrator Enrollment Role validation
- the shared shell and global navigation policy
- module definition and instance inventory
- manifest trust, compatibility, and dependency validation
- application blueprints, lockfiles, release inventory, and installation receipts
- module configuration status, enablement, readiness, health, and diagnostic orchestration
- semantic destination resolution

Core must not own or directly mutate module product records. Core orchestration must call module-owned contracts.

Core must own desired-state authoring and administrative orchestration but must not replace its own running process. An out-of-process, installation-local Supervisor/bootstrap CLI must materialize and roll back locked Core Release components, including the gateway, and Module Release image sets through one versioned control contract.

Before Core exists, the Supervisor must create and persist the stable Application Installation identifier, trust anchors, and authoritative materialization ledger. Core and the Supervisor must mutually authenticate. Every composition materialization apply and every Supervisor-owned operational override governed by the apply contract must use a separate Apply Authorization Envelope bound to operation kind, installation, current/base receipt, target or current Materialization Plan digest, monotonic desired/apply revision, nonce/idempotency key, initiator and required approver evidence, expiry, and explicitly approved override/destructive actions. The Supervisor must reject replay, stale bases, expired approval, and concurrent apply; serialize accepted mutation; and remain authoritative for observed materialization state while Core reconciles its read model after startup. Administrator Enrollment Claim lifecycle and separately launched Supervisor upgrades use their own explicitly defined authorization protocols. Planning access, including LLM access, must not confer approval authority.

### Identity and access

- The system must support explicit user accounts and authentication.
- Each Core Release must define a versioned Core Administration Capability Floor containing the global capabilities required for identity/user, RBAC, module, navigation, composition, and installation-recovery administration. It is a role-validation floor, not a hardcoded account type.
- Every locked composition must designate one Administrator Enrollment Role. Core and the Composition Engine must reject a missing designation or a role whose capability mapping does not contain the full floor, and must reject later edits that remove the designation or weaken that role below the current floor.
- A Viable Core Administrator must mean an active, authenticable identity with an active global assignment to a role whose effective capabilities cover the current Core Administration Capability Floor. Enrollment closure and recovery eligibility must use this same predicate.
- A fresh installation must establish its first Viable Core Administrator through a Supervisor-issued `initial` Administrator Enrollment Claim that is bound to the Application Installation, single-use, expiring, and created only under local operator/host authorization. Either kind requires a current Core decision that none exists; `initial` is unavailable after one has ever been established, while `recovery` additionally requires explicit, audited Supervisor-ledger break-glass authorization.
- The Supervisor must allow at most one issued or reserved Administrator Enrollment Claim per installation. Claim generations must follow `issued -> reserved -> consumed`, with expiry or revocation permitted from either nonterminal state; replacement issuance must revoke the previous generation. The ledger may persist only a one-way verifier and non-secret signed claim identifier, generation, kind, lifecycle, reservation, authorization, and outcome metadata, never a recoverable claim secret.
- Core must provide a dedicated idempotent enrollment ceremony that reserves the current claim generation through the Supervisor, then in one local Core transaction creates the first local user or binds an external identity, creates a global assignment to the locked Administrator Enrollment Role, and records the claim generation/reservation as redeemed. A signed idempotent result must let the Supervisor finalize consumption or reconcile an interrupted attempt without repeating the assignment. Normal sign-in must not double as this ceremony.
- Core must validate current Supervisor claim state during every redemption so restoring Core to a pre-redemption backup cannot make a consumed or revoked generation reusable. Expired, revoked, replayed, reserved-by-another-redemption, consumed, and cross-installation claims must fail closed under one non-disclosing caller result.
- The claim secret must be shown at most once and excluded from Blueprints, lockfiles, receipts, logs, diagnostics, status APIs, Core audit records, and recovery output. Only the one-way Supervisor verifier and non-secret lifecycle metadata may be retained.
- Administrators must be able to manage users through application UI.
- Roles must be reusable administrator-managed permission bundles, not hardcoded account types.
- Role assignments must support global scope and descendant-aware organization scope.
- Response ownership and delegation may grant access to assigned work independently of operator scope.
- Profiles, personas, and display labels must not act as authorization switches.
- Authorization across module calls must use verifiable, audience-bound grants that bind each security capability to its applicable global or descendant-aware Organization scope, or an authoritative Core decision for the exact original actor, presenting service, declared dependency/contract and action, resource, audience, and installation. Independent capability and scope sets must not be combined.
- Core must define descendant expansion and authorization/Organization revision semantics. Grants must be short-lived; receivers must validate freshness or obtain a current Core decision, with sensitive operations able to require online evaluation.
- A caller must exchange authority through Core for each downstream module audience rather than forwarding another module's context. The exchange must bind the presenting service and declared dependency/contract/action; the provider must validate both actor authority and caller-service authority. Service-only grants are limited to explicitly authorized system jobs. Delegation, assignment ownership, and similar exceptions must remain explicitly capability/resource-bound through the exchange.
- Browser authentication must use server-managed sessions; browser-visible bearer credentials are limited to explicit API-token, CLI, script, or test flows.

### Module capabilities and roles

- Modules must advertise namespaced security capabilities with stable identifiers and descriptions.
- Core must display installed modules' security capabilities in role-management UI.
- Modules must not create, alter, or assign authoritative Core roles.
- Optional module role templates must be advisory and require explicit administrator adoption.
- Disabled or removed providers must not cause Core to reinterpret historical capability, role, or audit identities.

### Organization

- The organization model must be configurable rather than hardcoded to one hierarchy.
- Nodes must support metadata-backed configuration and validated parent/child relationships.
- Users must be able to browse, inspect, create, and edit organization nodes through Core UI.
- UI and module contracts must support configured terminology instead of hardcoded legacy labels.
- Modules must receive only the organization context needed to enforce their declared operations.

## Module Requirements

Every normal Tessara module must be a separately deployable full-stack application. Each exact Module Release must provide:

- a versioned, verifiable manifest
- module identity, version, runtime/optional-migration image digests, and Core Release compatibility
- the v1 `tessara-oci-v1` Deployment Profile: digest-pinned runtime image and optional migration image, supported platform/architecture, runtime and migration commands, listen protocol/port and service-registration name, configuration/secret injection points, runtime versus migration identities, readiness/liveness probes, graceful shutdown, and resource requests/limits
- supported Shell Context, UI SDK, and design-system contract versions
- required and optional functional dependencies
- versioned machine-readable Feature Declarations with stable namespaced identifiers, descriptions, use cases, inputs, outcomes, constraints, and links to realizing contracts, resources, routes, configuration, and security capabilities
- required and provided contract versions
- namespaced security capabilities
- owned resource types and typed-reference schemas
- product routes when the capability has end-user workflows
- at least an administration, configuration, or diagnostics screen
- machine-readable configuration schema and module-owned validation API
- same-origin route integration through semantic destinations
- optional versioned shell-contribution contracts for Home/work discovery or global search, with explicit scope-bound Authorization Grant requirements, bounded latency/failure behavior, and owner-qualified semantic result destinations
- health, readiness, compatibility, and diagnostic endpoints
- module-owned persistence, migrations, retention, and recovery behavior
- platform conformance tests
- an optional module-owned, versioned, typed, idempotent bootstrap/apply/read-back contract when catalogs or initial product records are part of reproducible application construction

A module may have no primary end-user route. It must still be operable and configurable without direct database edits. Tessara must not require a separate headless-module class.

Catalogs, templates, preconfigured forms, instruments, saved batch definitions, and similar content must remain owned by the module that defines their semantics unless a future module explicitly provides a versioned functional contract for them. The platform must not treat them as generic untyped packages.

A module bootstrap contract, when provided, must define its own schema and semantics, accept normalized non-secret values or durable content-addressed input references with locked digests, expose validation and read-back, and emit a result receipt. It must not imply portability to another module implementation.

During the pre-pilot extraction only, current in-process areas may advertise `transitional_in_process` contribution descriptors. A descriptor may reserve a Module Definition identity but must not be represented as a Module Release or Module Instance. A first-party extracted consumer may bind temporarily to a versioned Core Release compatibility contract; that contract must use `core_installation`-owned transition resource references and must not be selectable by new external application Blueprints.

Extraction of that provider must create a real Module Release/Instance and perform an explicit data/reference migration: publish an old-to-new mapping, let each consumer rewrite its own references through a versioned migration/rebinding contract, emit completeness receipts, retain the Core adapter read-only until migration completes, and preserve an explicit migrated/retired result for old Core-owned references. Owner/type identity must never change merely because the new provider is enabled.

## Module Lifecycle And Administration

Core and module administration must assign state to the correct scope:

- Module Definition: registered
- Module Release evaluated in an installation: trusted and compatible
- Module Instance identity: live or tombstoned
- Module Instance operation: installed, deployed, configured, ready, enabled, and healthy
- Module Instance data: retained or explicitly destroyed
- Navigation Contribution: administrator display policy, grouping/order, and lifecycle eligibility
- Access decision: original actor when present, presenting service, declared dependency/contract and action or destination, resource, audience, and capability-to-scope binding

The product must never present a module instance as globally "authorized." Authorization is an access decision for a particular request or contribution.

V1 must allow at most one live (non-tombstoned) Module Instance per Module Definition in one installation. Upgrades must retain the instance identifier and database binding while selecting a new Module Release. A future multiple-live-instance model requires explicit instance-scoped capability, navigation, dependency-binding, and route semantics.

Requirements:

- Administrators must be able to inspect a module's manifest, Feature Declarations, contracts, security capabilities, dependencies, configuration status, readiness, health, version, and support metadata.
- Dependency, compatibility, configuration, readiness, and runtime-health findings must not be collapsed into one status.
- Module administration/configuration/diagnostics must remain reachable through Core or an eligible administrative destination when a product destination is disabled or unconfigured.
- Configuration entered through UI and configuration applied by machines must use the same module-owned schema and validation contract.
- Disabling a module must preserve its data by default.
- Desired per-module enablement must be part of the Blueprint/lockfile/release. An emergency disable may take effect immediately only through a constrained, non-destructive Apply Authorization Envelope recorded in the Supervisor Ledger and as a named, audited override with reason, actor, time, and optional expiry; it remains explicit drift until adopted or reconciled. Enablement must remain separate from navigation visibility and authorization.
- Undeploying runtime artifacts must preserve the Module Instance identity and database binding. Reactivation must reuse that identity. Explicit data destruction must be separately authorized and audited and must leave a tombstone that prevents existing references from rebinding to a new instance.
- A stopped or unhealthy module must produce contained degraded behavior rather than breaking Core or unrelated modules.

## Functional Dependencies

- Modules must declare dependencies against versioned functional contracts rather than database tables, process addresses, or undocumented behavior.
- Core must validate dependency closure, compatibility, cycles, trust, and explicit provider bindings before enablement.
- Feature Declarations must support discovery without acting as provider-equivalence or compatibility claims.
- Functional contract discovery must remain distinct from security-capability discovery.
- Optional dependencies must have declared absent-provider behavior.
- Cross-module calls must use typed clients, stable errors, timeouts, and correlation identifiers.
- Retried commands must be idempotent where duplicate effects would be unsafe.
- Multi-module workflows must use reconciliation and compensation rather than distributed database transactions.
- Consumers must handle provider unavailable, unauthorized, incompatible, disabled, and lifecycle outcomes explicitly.
- Cross-module callers must obtain a downstream-audience grant or decision from Core that binds the presenting service and declared dependency/contract/action; they must not forward an upstream audience credential as authority.

## Typed Resource References

- Cross-boundary resource relationships must use installation-scoped typed references containing installation identity, tagged owner kind (`core_installation` or `module_instance`), authoritative owner identifier, resource type, and resource identifier.
- A reference must remain bound to the authoritative owner and resource type for its lifetime.
- A provider must not reinterpret a referenced identifier as another resource type or silently transfer ownership.
- A reference's durability must not imply that all resource characteristics are immutable.
- The authoritative owner (Core or a module) must define its rules for mutation, publication, versioning, lifecycle, audit, and historical review; module product resources remain module-owned product policy.
- The authoritative owner's public contract must define how consumers resolve current state and observe relevant revisions or lifecycle changes.
- A consumer must define its own reaction to resolved provider state without taking ownership of provider semantics.
- Resolution must expose separate tagged owner, owner-data where applicable, resource-identity, provider-defined resource-lifecycle, access, contract-compatibility, and provider-availability dimensions, each with `undisclosed` or `not_evaluated` where applicable. Core-owned references must not be forced into Module Instance state fields; evaluated unknown or cross-installation owners must use explicit unknown/mismatch outcomes rather than `not_evaluated`.
- An `owner_module_instance_tombstoned` or `owner_data_destroyed` result must remain distinct from a live provider reporting its product resource's lifecycle state as `tombstoned`.
- After installation and audience validation, unauthorized or authorization-not-evaluated resolution must fail closed, use a stable restricted envelope, and leave caller-visible resource-specific owner, identity, and lifecycle dimensions undisclosed; detailed reasons are limited to separately authorized diagnostics. A resource-specific failure to evaluate after authorization must return `not_evaluated`, not `unknown_resource`; a known identifier and random identifier must be indistinguishable to a caller without existence-disclosure authority in status/shape and under the conformance profile's defined timing method/tolerance.

Examples include a Workflow holding a typed `FormVersion` reference and a Dashboard holding a typed `ComponentVersion` reference. A provider may mark the referenced resource inactive or superseded without changing its type. Consumers must be able to observe that state when the provider contract promises it.

## Persistence And Data Ownership

- Each v1 application installation must use exactly one PostgreSQL cluster for Core and module relational persistence, with one Core database and one database per module instance.
- Multi-cluster placement and module-selected external relational databases are deferred and must not be introduced as per-module v1 configuration.
- Each database must have dedicated runtime and migration credentials.
- A record must have one authoritative owner: Core for Core records or a module instance for module product records.
- Only runtime, migration, backup, restore, or recovery identities controlled by the owning Core or module boundary may access that boundary's database.
- Modules and Core must not use cross-database SQL, foreign keys, FDW shortcuts, shared writable schemas, shared runtime credentials, or another module's migrations.
- Cross-module reuse must occur through versioned APIs, events, exports, and typed references.
- Cached provider data must have an explicit revision/event and recovery contract.
- Each module must own its local transactions; multi-module operations must use idempotency, retry, reconciliation, and compensation.
- Backup and restore must support one module database and a complete application installation, including protected Supervisor Ledger/trust material, Core/module databases, lockfiles, receipts, and required external bootstrap inputs.
- Connection budgets, timeouts, and noisy-neighbor behavior must be monitored for the shared cluster.

## Application Blueprint And Release Requirements

An Application Blueprint must be a versioned, declarative description of:

- a Core Release version constraint
- typed Core configuration, including Organization schema and terminology where applicable
- selected modules, version constraints, and desired per-module enablement
- required and optional dependency bindings
- typed module configuration
- optional module-owned bootstrap declarations whose inputs are normalized non-secret values or durable content-addressed references
- navigation visibility, ordering, and grouping policy
- Core-owned role definitions and role-to-capability mappings
- one designated Administrator Enrollment Role that covers the Core Release's Core Administration Capability Floor
- environment-specific secret references

Users, role assignments, Administrator Enrollment Claim secrets or lifecycle records, ongoing module product records, and secret values are runtime data and must not be embedded in the Blueprint or lockfile. Administrator enrollment creates ordinary Core runtime user, assignment, and redemption data; the only product-record exception is an explicit module-owned bootstrap declaration governed by that module's versioned schema and idempotent apply contract.

Composition tooling must:

- discover module manifests, Feature Declarations, contracts, security capabilities, routes, configuration schemas, and Deployment Profile declarations
- reject missing, incompatible, cyclic, untrusted, or unbound compositions, including an absent Administrator Enrollment Role or one below the Core Administration Capability Floor
- generate a deterministic plan and human-readable diff
- deterministically resolve the Blueprint revision/digest, exact Core Release version and component image digests including the gateway, exact Module Release versions/image digests, selected Deployment Profile versions, desired module enablement, composition-engine/schema version, required Installation Supervisor/deployment-adapter contract version, contract/configuration/bootstrap schema versions, dependency bindings, normalized non-secret Core/module configuration plus navigation and role policy values and their digests, designated Administrator Enrollment Role and Core Administration Capability Floor version, normalized bootstrap values or durable content-addressed bootstrap references plus digests, versioned secret-reference identities, and Materialization Plan plus digest into a lockfile before approval
- hand the deterministic Materialization Plan, including enable/disable actions, and a separate authenticated/signed Apply Authorization Envelope to the out-of-process Installation Supervisor, which verifies them against its installation ledger and materializes only the resolved Core Release components and Module Releases through supported Deployment Profiles
- apply the desired state idempotently
- read back and verify installed state through supported APIs
- apply declared module bootstraps only through their owning typed APIs and verify their read-back idempotently
- acquire referenced bootstrap inputs from configured durable content-addressed sources and verify their locked digests before apply
- keep Supervisor status available while Core/gateway restarts and emit provenance, Apply Authorization Envelope identity, observed composition-engine and Supervisor/deployment-adapter versions, Core Release/component and Module Release inventory, desired/observed module enablement, installation state, and module-bootstrap result receipts
- reproduce an application release from its lockfile and externally resolved secret references without handwritten deployment or database steps
- detect desired/actual drift; UI changes to module enablement, Core/module configuration, navigation policy, role definitions, Administrator Enrollment Role designation, or declared bootstrap-managed state must create a new Blueprint/input revision or require an explicit adopt/reconcile action

The same versioned Composition Engine and operations must be available to Core UI/API, the Supervisor bootstrap CLI, deployment automation, and LLM clients. Before Core exists, the bootstrap CLI must either run that engine over a Blueprint plus trusted catalog inputs or verify a detached signature over a pre-resolved lockfile/plan digest from it; future MCP or agent adapters remain clients of these contracts rather than a separate control path.

## Deployment And Support Requirements

- Core and modules must be independently buildable, deployable, migratable, diagnosable, and supportable.
- V1 Core Release components and Module Releases must use the `tessara-oci-v1` Deployment Profile; non-OCI and module-defined executable profiles are deferred. The Supervisor must reject a profile it cannot implement.
- The Installation Supervisor/bootstrap CLI must run outside Core, support first installation through the shared Composition Engine or a detached signature over a resolved lockfile/plan digest, and own Core Release component and Module Release migration sequencing, health-gated traffic switching, rollback, and recovery. Its own signed upgrade must run through a separate bootstrap launcher with ledger backup/schema compatibility, health verification, and rollback rather than through an application apply.
- After first Core health, the Supervisor CLI must support initial and recovery Administrator Enrollment Claim issuance under their distinct eligibility rules, non-secret lifecycle/reservation status inspection, revocation/replacement, and reconciliation without exposing a claim secret after initial display.
- A module instance must serve exactly one application installation unless a later architecture explicitly adds multi-tenancy.
- A same-origin gateway must route browser UI and API traffic without exposing deployment topology in saved links.
- Every supported application release must record the exact Core Release version and component-image digests including the gateway, exact Module Release versions/image digests, selected Deployment Profile versions, and compatibility status.
- Installations must remain operable without a mandatory central Tessara SaaS control plane.
- Core Release component (including gateway) and Module Release upgrades must validate compatibility before migrations or traffic switching.
- Failed migrations, unavailable dependencies, rollback, and recovery must have explicit operational behavior.
- Diagnostic bundles must redact secrets while preserving versions, health, dependency findings, and correlation data needed for support.

## Reference Application Requirements

The first-party reference application consists of Forms, Workflows, Responses, Datasets, Components, and Dashboards. These are module product requirements, not permanent Core responsibilities.

### Forms

- Forms must own forms, fields, option sources, form versions, publication, catalogs, and lifecycle semantics.
- Administrators must be able to create, edit, remove, and reorder fields through UI.
- Form authoring must support typed fields, option sets, and constrained lookup behavior.
- Draft and published behavior must be clear in product UI.
- Forms must expose typed `Form` and `FormVersion` resolution and state-observation contracts.
- Forms must decide which changes require a new version and how active, inactive, superseded, archived, or other lifecycle states behave.

### Workflows

- Workflows must own definitions, versions, steps, assignments, runtime coordination, publication, and handoff rules.
- Published workflow steps must use typed FormVersion references rather than Forms database relationships.
- Workflow consumers must observe FormVersion state according to the Forms contract.
- Workflow operations must preserve original actor, presenting service, declared contract/action, and each capability-to-scope binding through Core-issued downstream-audience authorization exchange across module calls.

### Responses

- Responses must support assignable work, pending work, drafts, submission, and read-only completed review.
- Canonical responses must be stored as structured payloads keyed to the referenced form contract.
- Submission validation must be strict at workflow boundaries.
- End-user response flows must remain understandable without exposing builder, deployment, or migration concerns.
- Responses must expose typed source, export, or event contracts for authorized consumers.

### Datasets

- Datasets must be reusable row-level analytical assets.
- Datasets must own source composition, joins and unions, reducers, filters, calculated fields, row grain, exposed field contracts, revisions, and materialization.
- Dataset authoring must expose validation, lineage, compatibility, and revision behavior through application UI.
- Source data must be consumed through provider APIs, exports, or events rather than Response or other module tables.
- Current DatasetRevision immutability and major-line behavior remain Dataset product policy and may evolve only through the Dataset contract.

### Components

- Components must own versioned presentation assets over Dataset contracts.
- The current Table Component supports last-mile projection, saved default filters, display labels, default sort, page size, and viewer affordances.
- Reusable analytical shaping, aggregation, grouping, and bucketing remain Dataset responsibilities unless the owning module requirements change.
- Components must reference Datasets through typed resources and contracts, not database relationships.
- Current ComponentVersion publication and update-in-place behavior remains Component product policy, not a platform invariant.

### Dashboards

- Dashboards must own mutable dashboard composition and placement behavior.
- A Dashboard must use durable typed `ComponentVersion` references.
- Dashboard authoring and viewing must be available through application-grade UI.
- Dashboard composition uses a fixed 12-column, 240-row grid with at most 240 stored placements.
- Dashboard must display explicit placement-level behavior for unavailable, unauthorized, inactive, superseded, provider-resource tombstoned, owner-module-instance tombstoned/data-destroyed, or incompatible ComponentVersion resolution as applicable. Resource-specific states are shown only after authorized resolution; an unauthorized placement uses the non-disclosing forbidden state.

## Compatibility And Change Observation

Module owners define compatibility and versioning rules for their resources. For the current reference analytics application:

- Dataset revision and major-line compatibility must remain visible to consumers.
- Component and Dashboard consumers must receive declared state and compatibility outcomes from their providers.
- Changelog impact, carry-forward, and rebinding flows must operate over typed references.
- A separate published resource version must not silently replace an existing reference unless the owning provider contract and consumer product rule explicitly define that behavior.
- Publication guards belong to the owning module and must not be generalized into Core policy.

## Application UI Requirements

- Every roadmap sprint must leave its capability testable through intended application UI.
- Core module administration must expose module inventory, configuration, capabilities, dependencies, versions, readiness, health, navigation policy, and diagnostics.
- Core role/composition administration must expose the current Core Administration Capability Floor version and designated Administrator Enrollment Role validation, and must block an edit that leaves no compliant designation.
- Modules must expose their own product and administrative UI through the shared shell.
- Common destinations must not require users to copy identifiers or use workbench-only routes.
- Loading, empty, no-results, error, read-only, restricted, disabled, incompatible, and unavailable states must remain distinct.
- SSR and route behavior must remain useful if hydration fails where practical.
- Hydration mismatches and uncaught browser console errors are release-blocking defects.
- Module outages must not erase shell context or strand users without an actionable recovery path.
- Administrator enrollment must be visually and behaviorally separate from normal sign-in, must not reveal its secret after issuance, and must be unavailable while a Viable Core Administrator exists.

## Quality And Conformance Requirements

- Domain rules and persistence must reside in the owning Core or module boundary.
- Public contracts must be versioned and covered by provider and consumer tests.
- Module releases must pass manifest, `tessara-oci-v1` runtime/migration/configuration/probe/shutdown/resource behavior, route, scope-bound authorization, dual-principal downstream-audience exchange, authentication-context, database-isolation, health, and degraded-state conformance tests.
- Permission-controlled behavior must have positive and negative coverage across module calls, including one actor whose different capabilities apply to different Organization subtrees; no receiver may authorize the cross-product of those grants.
- Permission and resolution coverage must prove that undeclared and declared-but-wrong-audience/action services cannot exchange or replay a user's authority; grants issued before role, scope, ownership, or delegation revision changes fail closed as stale; and unauthorized known-versus-random identifiers are indistinguishable under a conformance profile that defines the normalized environment, measurement method, sample size, and pass/fail timing tolerance.
- Application compositions must have end-to-end tests against their resolved lockfiles.
- Administrator-enrollment conformance must cover local-user and external-identity paths; capability-floor and designated-role validation; at-most-one claim and generation revocation; reservation, local transactional redemption, signed-result reconciliation, and idempotent retry; one-time secret display; expired/revoked/replayed/reserved/consumed/cross-installation rejection; closure while a Viable Core Administrator exists; audited recovery eligibility; and replay resistance after restoring Core to a pre-redemption backup.
- Pilot hardening must execute a Supervisor binary/control-contract and ledger-schema upgrade plus rollback and recovery, not merely document its procedure.
- Current compatibility adapters must remain isolated and removable; new product behavior must not expand them.
- Unsupported behavior must be documented explicitly.

## Out Of Scope Or Deferred

- a requirement that all Tessara installations share one multi-tenant SaaS runtime
- multi-cluster placement or module-selected external relational databases in v1
- non-OCI or module-defined executable Deployment Profiles in v1
- automatic hot-swapping of semantically equivalent modules or automatic transfer of their data
- a generic package/content-pack abstraction separate from module-owned features
- direct database manipulation by LLMs, deployment tools, Core, or other modules
- runtime-loaded remote UI code, iframes, or browser-side microfrontend composition as the default architecture
- printable report artifacts composed from prose and Components
- full visual dashboard design beyond the current required composition flows
- fuzzy joins and analytical behaviors beyond the defined Dataset engine
- permissions or scope-sharing behavior not established by Core RBAC and module contracts

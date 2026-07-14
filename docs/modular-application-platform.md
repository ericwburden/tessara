# Tessara Modular Application Platform

This document defines the product-level contract for composing and operating Tessara applications from Core and independently deployed modules. It is the canonical source for terms and boundaries that apply across modules. Module-specific product behavior remains in the owning module's requirements and roadmap.

## Product Direction

Tessara is a modular application-construction and distribution platform. A Tessara application combines a stable Core with only the feature modules needed for a particular use case. Existing modules provide reusable application capabilities; new development can concentrate on the capabilities that are genuinely missing.

The target is a family of separately deployed and separately supported applications, not a requirement to operate one monolithic SaaS. Each application installation can have its own selected modules, configuration, navigation, roles, data, release cadence, and support boundary.

This direction is designed for both human and machine composition. An LLM or other automation client should be able to discover available modules, understand their contracts and configuration schemas, produce a valid application blueprint, and apply it through the same supported interfaces used by administrators and deployment tooling.

## Vocabulary

- **Core** is the permanent application foundation. It owns Organization, identity and user management, RBAC, the shared shell, module inventory, application composition, access/composition audit, and installation-level status. Product audit remains owned by the relevant module.
- **Core Release** is one exact versioned distribution of Core, including the Core application and its same-origin gateway component artifacts, `tessara-oci-v1` declarations, and digests. The gateway may run as a separate process but is selected, compatibility-tested, upgraded, and rolled back only as part of its Core Release in v1.
- **Module Definition** is the stable logical identity, publisher namespace, and feature family of a module across releases.
- **Module Release** is one exact versioned manifest and `tessara-oci-v1` distributable image set for a Module Definition, including its digests, contracts, schemas, migrations, and support metadata.
- **Module Instance** is the installation-scoped durable identity, data binding, and lifecycle record for one use of a Module Definition. It points to a current Module Release and retains its identity across upgrades, disablement, undeployment, and reactivation. A Module Instance belongs to exactly one Application Installation.
- **Application Blueprint** is the desired, declarative composition: a Core Release version constraint and Core configuration, selected modules and their version constraints, desired per-module enablement, dependency bindings, module configuration, optional Module Bootstrap Declarations whose inputs are normalized non-secret values or durable content-addressed references, navigation policy, Core-owned role definitions and role-to-capability mappings, a designated Administrator Enrollment Role, and environment-specific secret references.
- **Application Lockfile** is the deterministic resolved composition: the Blueprint revision/digest, exact Core Release version and all Core component-image digests (including the gateway), exact Module Release versions and image digests, selected Deployment Profile versions, resolved desired module enablement, composition-engine/schema version, required Installation Supervisor/deployment-adapter contract version, contract/configuration/bootstrap schema versions, dependency bindings, normalized non-secret Core/module configuration plus navigation and role policy values and their digests, the resolved Administrator Enrollment Role and Core Administration Capability Floor version, normalized bootstrap values or durable content-addressed bootstrap references plus digests, versioned secret-reference identities, and the Materialization Plan plus its digest. Approval is separate. It never contains secret values.
- **Application Installation** is one Supervisor-rooted deployment containing the installation-local Supervisor/ledger, one Core Release and its gateway component, selected Module Instances, the single v1 PostgreSQL cluster and databases, configuration, release inventory, and operational state.
- **Installation Receipt** is the versioned result record for one composition apply. It binds the resolved lockfile and Materialization Plan digests plus Apply Authorization Envelope identity to the observed composition-engine and Installation Supervisor/deployment-adapter versions, Core Release/component digests, Module Releases, Module Instance identities/data bindings/enablement, configuration/policy/bootstrap digests and results, deployment provenance, timestamps, and verification outcomes. It is evidence of applied state, not desired state.
- **Composition Engine** is the versioned, policy-neutral catalog/Blueprint validation, resolution, and plan/diff implementation shared by Core and the Supervisor bootstrap CLI. It produces the same lockfile for the same inputs and does not materialize artifacts itself.
- **Installation Supervisor** is the trusted local process and bootstrap CLI that runs outside Core and owns the Application Installation root of trust, artifact/database materialization, migration sequencing, health gates, traffic switching, and rollback for Core, the gateway, and modules. It is the only application component allowed to replace Core while executing an apply; it does not require a central Tessara SaaS.
- **Supervisor Ledger** is the Supervisor-owned pre-Core record of the stable Application Installation identifier, trust anchors, monotonically ordered desired/apply revisions, accepted nonces/idempotency keys, current and previous Materialization Plan and receipt digests, Administrator Enrollment Claim generations/reservations/lifecycle states, privileged approvals, and operation outcomes. It serializes mutation and is authoritative for observed materialization and claim state; Core reconciles a read model from it after startup.
- **Materialization Plan** is the deterministic, versioned, non-secret portion of an Application Lockfile consumed by the Installation Supervisor. It contains the exact Core/Module artifacts, Deployment Profile bindings, database/migration/enablement actions, normalized configuration digests and secret-reference identities, health gates, traffic-switch steps, and rollback anchors. The Composition Engine computes the plan and digest before approval.
- **Apply Authorization Envelope** is the separate authenticated or signed authorization to execute one Materialization Plan or a narrowly typed operational override against the current plan. It binds operation kind, installation identifier, current/base receipt digest, target or current plan digest, monotonic desired revision/apply sequence, nonce/idempotency key, initiator actor/service and required approver evidence, issued/expiry times, and explicitly authorized override/destructive actions. It is not part of deterministic lockfile contents.
- **Core Administration Capability Floor** is the versioned, Core Release-defined set of global capabilities required to manage identity/users, RBAC, modules, navigation, application composition, and installation recovery. It is a validation floor, not a hardcoded user type or an independently assignable role.
- **Administrator Enrollment Role** is the Blueprint-designated Core role that an enrollment claim assigns at global scope. Composition validation requires it to exist and include the complete Core Administration Capability Floor; a Blueprint or UI edit cannot remove the designation or weaken the role below that floor.
- **Viable Core Administrator** is an active, authenticable identity with an active global assignment to a role whose effective capabilities include the current Core Administration Capability Floor. This same predicate controls enrollment closure and recovery eligibility.
- **Administrator Enrollment Claim** is a Supervisor-issued, installation-bound, single-use, expiring authorization for enrolling a Viable Core Administrator. It has an explicit `initial` kind, permitted only while none currently exists and no viable administrator has ever been established, or `recovery` kind, permitted only through an audited break-glass authorization when none currently exists. Its once-displayed secret is accepted only as write-only enrollment input and never belongs in a Blueprint, lockfile, receipt, log, status response, audit payload, or diagnostic bundle.
- **Deployment Profile** is a versioned executable-packaging contract understood by both a release and the Supervisor's deployment adapter. V1 supports one profile, `tessara-oci-v1`: digest-pinned OCI images with declared runtime and migration commands, platform/architecture, listen protocol/port and service-registration name, configuration and secret injection points, runtime versus migration identity, readiness/liveness endpoints, graceful-shutdown behavior, and resource requests/limits.
- **Application Release** is a reproducible, supportable resolved software, module-enablement, configuration, navigation, role-policy, and declared-bootstrap composition represented by a lockfile and installation receipt.
- **Feature Declaration** is versioned machine-readable discovery metadata with a stable namespaced feature identifier, description, use cases, inputs, outcomes, constraints, and links to the contracts, resources, routes, configuration, and security capabilities that realize it.
- **Module Bootstrap Declaration** is an optional, module-owned, versioned and idempotent instruction for creating or reconciling that module's catalogs or initial product records through its supported API. Its locked input is either a normalized non-secret value or a durable content-addressed reference with a verified digest. It is not a generic or portable content package.
- **Functional Contract** is a versioned API, event, resource, or behavior contract provided or required by a module.
- **Security Capability** is a namespaced permission advertised by a module and incorporated into Core-owned RBAC.
- **Authorization Grant** is a short-lived, verifiable assertion that binds one Security Capability and its scope to an Application Installation, target audience, route or dependency/contract action, original actor when acting on a user's behalf, and the presenting gateway or module service identity. Cross-module grants also bind the declared dependency. It carries authorization and Organization revision/freshness data; delegation or ownership authority is separately bound rather than inferred. A service-only grant is valid only for an explicitly authorized system job.
- **Typed Resource Reference** identifies a resource owned by Core or a module without granting database access or transferring authority over that resource.
- **Semantic Destination** names a Core or module-instance route and its typed parameters without embedding a deployment URL.
- **Navigation Contribution** is a versioned manifest entry that advertises a Semantic Destination with its label, group and order hints, lifecycle eligibility, and required Security Capability. Core applies installation display policy and effective authorization before presenting it.
- **Shell Context** is versioned, authenticated, request-scoped context for the Application Installation, actor or service, applicable scope-bound Authorization Grants, theme, resolved navigation model, and return destination. It contains no reusable secret.
- **Shell Contribution** is an optional versioned provider contract for Home, work discovery, or global search. It returns bounded, owner-qualified semantic results under explicit latency, failure, and scope-bound Authorization Grant rules; it is neither raw HTML nor a transfer of module product-data ownership to Core.

## Composition Model

A normal Tessara module is a full-stack application. It provides everything required to operate its capability coherently:

- product UI when the capability has end-user workflows
- at least an administration, configuration, or diagnostics UI
- module-owned APIs and events
- a typed configuration schema and validation contract
- health, readiness, compatibility, and diagnostic information
- namespaced security capabilities
- product and administrative semantic destinations
- module-owned persistence, migrations, and data lifecycle behavior
- conformance tests for the platform contract

A module may have no end-user navigation contribution, but it is not modeled as a separate class of "headless package." Catalogs, templates, instruments, batch definitions, and similar content are features or artifacts owned by the module that understands their semantics. Tessara does not need a second generic content-pack abstraction.

Modules are interchangeable in the compositional sense: an application can omit capabilities it does not need and add narrowly scoped modules for capabilities it lacks. The platform does not promise that independently authored providers with superficially similar functions are drop-in replacements or share data semantics.

## Core Responsibilities

Core is authoritative for:

- application-level use of the Supervisor-anchored installation identity, plus Organization
- authentication, users, sessions, delegation, and service identity
- security capability registration
- roles, role capabilities, role assignments, organization scope, and access audit history
- module definition, release, and instance inventory
- manifest trust, compatibility, and dependency validation
- desired and resolved application composition
- module installation, configuration status, enablement, health, and diagnostics orchestration
- administrator navigation visibility and ordering policy
- resolution of semantic destinations to same-origin application routes
- the coherent authenticated shell and installation-level status experience
- release inventory, lockfiles, installation receipts, and composition provenance

Modules advertise security capabilities; they do not create or mutate authoritative roles. A module may offer advisory role templates, but an administrator explicitly adopts or maps those templates through Core.

Core does not own a module's product semantics or write the module's product data. It orchestrates supported module contracts instead of becoming a generic implementation of every module feature.

Core owns desired-state authoring, validation, and administrative orchestration, but it does not replace its own process. Core or another authorized client hands the deterministic Materialization Plan plus a separate Apply Authorization Envelope to the out-of-process Installation Supervisor and reads its status and receipt through a versioned local control contract.

## Module Manifest And Contract

Every Module Release publishes a signed or otherwise verifiable manifest containing at least:

- stable module identity, publisher, semantic version, and runtime/optional-migration image digests
- `tessara-oci-v1` runtime image and optional migration-image digests plus commands, supported platform/architecture, listen/service registration, configuration/secret injection, health/probe, graceful-shutdown, and resource declarations
- supported Core Release versions and manifest schema version
- supported Shell Context, UI SDK, and design-system contract versions
- required and optional functional dependencies plus any explicit provider-binding constraints
- machine-readable Feature Declarations with stable namespaced identifiers, descriptions, use cases, inputs, outcomes, constraints, and links to their realizing contracts and contributions
- required and provided functional contract names and versions
- owned resource types and their reference schemas
- namespaced security capabilities and human-readable descriptions
- product, administration, configuration, and diagnostics destinations
- navigation contributions, labels, grouping hints, and required security capabilities
- optional versioned shell-contribution contracts such as Home/work-discovery panels and global search providers, including scope-bound Authorization Grant requirements, latency/failure behavior, and semantic result destinations
- typed configuration schema, secret-reference fields, and validation endpoint
- health, readiness, compatibility, and status endpoints
- database migration and data-retention metadata
- optional module-owned bootstrap schema, validation/apply/read-back endpoints, and receipt contract
- conformance suite version and support metadata

Core validates trust, Core Release compatibility, dependency closure, contract bindings, route uniqueness, security-capability namespaces, configuration, and Deployment Profile compatibility before a module can become ready or enabled. The Supervisor conformance suite verifies that declared runtime/migration commands, probes, identity separation, shutdown, and resource behavior match `tessara-oci-v1`; an artifact digest alone is never treated as an executable deployment contract.

Feature Declarations, functional contracts, and security capabilities are separate namespaces. A Feature Declaration helps a human or LLM select a module; it is not an integration contract and does not promise that two similarly described modules are interchangeable. A module can require a functional contract without inheriting security authority from its provider, and possessing a security capability does not prove that a required provider is installed or healthy.

## Module State

State belongs to a specific entity and scope; the platform must not collapse it into one `module_status` or `module_authorized` flag.

Module Definition state includes:

- registered: Core knows the stable definition and namespace

Module Release evaluation within an installation includes:

- trusted: its publisher/artifact evidence satisfies installation policy
- compatible: the release can bind to the current Core Release and declared contracts

Module Instance lifecycle includes:

- identity state: live or tombstoned
- installed: the instance and its durable resources exist
- deployed: its runtime artifact is materialized and its gateway/service-discovery registration exists
- configured: module-owned validation accepts its configuration
- ready: dependencies and migrations satisfy its declared readiness contract
- enabled: administrators permit product operation
- healthy: runtime checks currently succeed
- data state: retained or explicitly destroyed

For v1, an installation may have at most one live (non-tombstoned) Module Instance for a given Module Definition. Instance identifiers are still carried in references, routes, dependency bindings, database names, and receipts so upgrades preserve identity and a future multiple-instance model cannot silently reinterpret existing data. Supporting multiple live instances later requires explicit instance-scoped capability and navigation rules.

Navigation Contribution state includes administrator display policy, resolved grouping/order, and eligibility for the current installation. Authorization is evaluated separately for a particular actor, action or destination, resource, audience, and capability-to-scope binding; a module instance is never globally "authorized."

Changing navigation visibility does not grant or revoke authorization. Disabling a module is not the same as hiding it. A stopped or unhealthy module is not automatically undeployed, tombstoned, or destroyed. These distinctions must remain visible in administration and machine-readable status.

## Navigation And Shared Experience

Core and modules contribute destinations to one coherent, same-origin application experience. Core owns shell policy, navigation composition, and a versioned Shell Context contract. Each separately deployed module uses the shared UI SDK and authenticated Shell Context to server-render the complete HTML document for its own routes, including coherent shell chrome; Core does not fetch and wrap a remote HTML fragment. If the module cannot respond, the gateway serves a Core-owned fallback document that preserves navigation/context and reports the module state.

Cross-boundary links use a semantic destination with a named route, tagged owner (`core_installation` or `module_instance`), authoritative owner identifier, and typed parameters. They never persist environment-specific host names or assume another module's deployment address. Core resolves the destination for the current installation and returns explicit outcomes for unavailable, disabled, unconfigured, unauthorized, incompatible, or unknown destinations. An unauthorized direct request receives the same restricted outcome whether the named destination is known or unknown. An explicit dependency-binding key may be resolved to the installation's one live instance for that Module Definition.

The left navigation is composed from:

1. permanent Core destinations,
2. eligible module contributions: product destinations normally require an enabled instance, while administration/configuration/diagnostics destinations may remain available for an installed but disabled or unconfigured instance,
3. administrator visibility and ordering policy, and
4. the current user's applicable scope-bound Authorization Grants.

Navigation filtering is a usability feature, not an authorization boundary. Every module independently enforces authorization on its routes, APIs, commands, and resource resolution.

## Security Boundary

Core authenticates browser sessions and remains authoritative for authorization. A cross-module request must never carry one independent security-capability set beside one independent scope set, because combining them could grant a capability at a scope where it was never assigned. Instead, Core supplies either scope-bound Authorization Grants or an authoritative authorization-decision/token-exchange result for the exact actor, presenting service, declared dependency/contract and action, resource, audience, and installation.

Each grant binds one security capability to global scope or to explicit Organization roots with descendant semantics at a named Organization/authorization revision. Core defines descendant expansion. A receiving module either validates an unexpired audience-bound grant containing the relevant expanded or revision-bound scope, or asks Core for a fresh decision; it does not reconstruct hierarchy or combine grants in its own database. Short expiry bounds stale authorization, and sensitive operations may require an online Core decision. Delegation, assignment ownership, or other exceptional authority is carried as an explicit capability/resource-bound assertion rather than as an unbound flag.

An audience-bound context issued to one module is not forwarded as authority to another module. The caller exchanges it through Core for a new downstream-audience grant or decision. That exchange preserves the original actor, installation, correlation and delegation basis, binds the presenting service and its declared dependency/contract/action, and evaluates the downstream action against current role and Organization revisions. The provider validates both the actor's scoped authority and the calling service's permission to invoke that bound contract, preventing another installed module from replaying user authority against an undeclared API.

Modules must:

- validate the asserted installation, original actor when present, presenting service identity, declared dependency/contract/action, audience, expiry, authorization revision, and capability-to-scope binding
- enforce their own route, API, command, and resource authorization
- obtain a new Core-issued downstream-audience grant or decision when calling another module
- use dedicated service credentials for server-to-server calls
- avoid sharing browser cookies, bearer credentials, database credentials, or writable schemas with other modules
- return stable forbidden, unavailable, incompatible, and lifecycle outcomes without leaking internal details

Core role editing displays security capabilities contributed by installed modules. Removing or disabling a module does not silently reinterpret historical role or audit records; Core preserves the namespaced security-capability identity and reports its current provider status.

## Data Ownership And Persistence

V1 requires every Application Installation to use one PostgreSQL cluster for Core and module relational persistence, with one Core database and one database per module instance. Each database has dedicated runtime and migration roles. Multi-cluster placement and an externally selected module database are deferred platform models, not per-module configuration in v1.

Hard boundaries:

- a record has one authoritative owning provider: Core for Core records or a module instance for module product records
- only identities controlled by that owner read or write its database
- no cross-module SQL, foreign keys, writable shared schemas, FDW shortcuts, shared migrations, or shared runtime credentials
- Core cannot mutate module product tables
- modules cannot read Core identity or RBAC tables directly
- integration uses versioned APIs, events, exports, and typed resource references
- each module owns its transactions, migrations, retention, backup metadata, and recovery procedures
- cross-module workflows use idempotency, retry, reconciliation, and compensation rather than distributed database transactions

Disabling or undeploying a Module Instance does not destroy its database or identity record. Reactivating retained data reuses the original Module Instance identifier and database binding; a new instance receives a new identifier and cannot inherit existing references accidentally. Explicit data destruction is a separate, audited operation governed by the owning provider's retention contract and leaves a durable tombstone reserving the former instance identity so an authorized resolution returns `owner_module_instance_tombstoned` and `owner_data_destroyed` rather than silently rebinding.

## Typed Resource References

A typed cross-boundary reference contains at least:

- application installation identifier
- owner kind: `core_installation` or `module_instance`
- authoritative owner identifier: the Core installation or module instance identifier selected by the owner kind
- resource type
- resource identifier

The architectural guarantee is stable owner identity and type: a reference advertised as a `FormVersion` continues to resolve, when resolvable, as a `FormVersion`. The provider cannot reinterpret that identifier as another resource type or silently transfer authority to another owner. Core-owned Organization resources use the same envelope with `core_installation` ownership.

Durability does **not** require every characteristic of the resource to be immutable. The owning provider (Core or a module) decides:

- which mutations are permitted
- which changes require a new version or identifier
- whether active, inactive, superseded, archived, or tombstoned states exist
- what audit and historical-review behavior the product provides
- which state and revision changes consumers may observe

For module product resources, these decisions remain module-owned product policy; Core makes the equivalent decisions only for Core-owned resources such as Organization records.

The provider contract declares how consumers resolve current state and observe relevant change: live reads, revision markers, events, caches, or an explicit combination. Consumers decide how to react to those outcomes. A consumer can show a warning, stop execution, retain a snapshot, request rebinding, or continue according to its own product rules without owning the provider's lifecycle policy.

Resolution is structured so owner identity/data, product-resource lifecycle, authorization, compatibility, and runtime availability cannot collapse into one status. At minimum it distinguishes:

- `access_state`: `authorized`, `unauthorized`, or `not_evaluated`; `not_evaluated` fails closed and never permits the operation
- tagged `owner_state`: a Core-installation state (`live`, `unknown_core_installation`, or `installation_mismatch`), a Module Instance state (`live`, `owner_module_instance_tombstoned`, `unknown_module_instance`, or `owner_mismatch`) plus data state (`retained`, `owner_data_destroyed`, `unknown`, or `not_evaluated`), `undisclosed`, or `not_evaluated`
- `resource_identity_state`: `resolved`, `unknown_resource`, `undisclosed`, or `not_evaluated`
- provider-defined `resource_lifecycle_state`, which may include `active`, `inactive`, `superseded`, `archived`, `tombstoned`, `migrated`, or `retired`, plus `undisclosed` and `not_evaluated`
- contract compatibility and provider availability, each allowing `undisclosed` or `not_evaluated`

`owner_module_instance_tombstoned` and `owner_data_destroyed` describe installation-owned Module Instance history. A product resource's lifecycle value `tombstoned` is provider-owned policy while its owner may remain live; consumers must not interpret one as the other. Core-owned references use the tagged Core-installation owner state rather than Module Instance fields.

After validating the request installation and audience, the provider evaluates authorization before disclosing resource-specific resolution. An unauthorized or authorization-not-evaluated response fails closed and returns only the stable restricted envelope; caller-visible owner, resource-identity, and lifecycle dimensions are `undisclosed`. More detailed reasons may appear only in separately authorized internal diagnostics. A provider that cannot evaluate a resource-specific dimension after authorization returns `not_evaluated`, not `unknown_resource`. For a caller without existence-disclosure authority, a valid identifier and a random identifier must be indistinguishable in status and response shape, and in timing within the conformance profile's defined measurement method and tolerance.

## Application Blueprints And LLM Composition

An Application Blueprint is the supported input for building an application. It describes Core/module composition and may include module-owned Bootstrap Declarations whose inputs are normalized non-secret values or durable content-addressed references for reproducible catalogs and initial product records. It never directly manipulates deployment internals or module databases. Composition tooling performs:

```text
discover -> author blueprint -> validate -> resolve -> plan/diff
  -> lockfile + Materialization Plan -> separate approval envelope
  -> Supervisor apply -> verify/read back
```

Validation checks trust, Core Release compatibility, dependency closure, functional contract bindings, configuration schemas, security-capability namespaces, semantic destinations, Deployment Profile support, and environment requirements. Resolution deterministically produces a lockfile containing the exact Materialization Plan and digest. Apply is executed by the out-of-process Installation Supervisor through its versioned deployment adapter: it mutually authenticates the caller, verifies the separate Apply Authorization Envelope, plan/lockfile digest, profile compatibility, base receipt, revision, expiry, nonce, approval and destructive-action scope, then serializes the mutation in its ledger. It acquires only the locked Core Release component OCI images (including its gateway) and Module Release OCI images from configured trusted sources, verifies publisher/signature or equivalent provenance and digest, runs declared migration commands with migration identities, starts runtime commands with runtime identities/configuration, applies health gates, switches traffic, and can roll back without depending on the Core process being replaced. It likewise acquires referenced bootstrap inputs from their configured durable content-addressed source and verifies the locked digest before invoking the module. Apply is idempotent, rejects replay/stale/concurrent base revisions, emits provenance and an installation receipt, and is a no-op when desired and actual state already match.

The first installation is bootstrapped through the Supervisor CLI under local operator/host authorization. Before Core exists, the Supervisor creates the installation identifier, key/trust material, and ledger. The CLI then either (a) runs the same versioned Composition Engine used by Core over a Blueprint, trusted catalog inputs, and environment references, or (b) verifies a detached signature over a pre-resolved lockfile/plan digest produced by that engine. In both cases it constructs the first Apply Authorization Envelope, records provenance, and seeds Core's desired/resolved composition read model after startup. Later applies may be initiated through Core UI/API, CLI, automation, or an LLM client, but every composition materialization apply and apply-contract operational override requires a mutually authenticated, installation-bound envelope and the Supervisor serializes it against the ledger. Administrator Enrollment Claim lifecycle and separately launched Supervisor upgrades use their own explicit authorization protocols. Planning authority never implies approval authority, and destructive actions require explicit approval. Supervisor status remains available while Core or the gateway restarts, and Core reads back the final receipt after recovery. The supervisor itself has an explicit, separately managed upgrade procedure so an application lockfile never asks a process to replace itself.

After Core becomes healthy, the Supervisor may issue an `initial` Administrator Enrollment Claim only under the same local operator/host authorization, while its ledger records that no viable administrator has ever been established, and after a current Core decision confirms none exists. Before issuance, Core and composition validation confirm that the locked Administrator Enrollment Role exists and covers the Core Administration Capability Floor. The Supervisor permits at most one issued or reserved claim per installation. Its lifecycle is `issued -> reserved -> consumed`, with `issued` or `reserved` also able to become `expired` or `revoked`; replacement issuance revokes any prior nonterminal generation. The ledger stores only a one-way verifier and non-secret signed metadata, never a recoverable claim secret.

Redemption is an idempotent protocol rather than a distributed transaction. Core presents the claim to the Supervisor and obtains a claim-generation-bound redemption reservation after the Supervisor validates current state, installation, kind, expiry, and verifier. In one Core database transaction, Core creates or binds the identity, creates a global assignment to the locked Administrator Enrollment Role, and records the claim identifier, generation, and reservation as redeemed. Core then returns a signed, idempotent result so the Supervisor marks the reservation consumed; retry or reconciliation resumes the same reservation and cannot create a second administrator assignment. Core must consult current Supervisor claim state on every redemption, so restoring Core from a pre-redemption backup cannot make a consumed or revoked generation reusable. The enrollment endpoint becomes unavailable whenever a Viable Core Administrator exists.

An `initial` claim may be reissued under local operator/host authorization only before initial redemption, revoking the prior generation. After a viable administrator has previously existed, only a `recovery` claim may be issued, and only through an explicit Supervisor-ledger break-glass authorization based on a current Core viability decision. Recovery uses the same capability floor, designated role, lifecycle, and redemption protocol. Expired, revoked, replayed, reserved-by-another-redemption, consumed, or cross-installation claims fail closed under one non-disclosing caller result. The claim secret is shown once and never returned by status, audit, support, or recovery APIs.

The desired composition cannot silently diverge from administrator changes. A module enable/disable, Core or module configuration, navigation-policy, role-definition, or Administrator Enrollment Role designation edit made through UI either creates a new Blueprint revision or is reported as drift with explicit adopt or reconcile actions. Emergency operational disablement may take effect before Blueprint revision only through a constrained, non-destructive Apply Authorization Envelope recorded in the Supervisor Ledger; it is a named, audited override with reason, actor, time, and optional expiry, and the installation remains drifted until adopted or reconciled. Lockfiles contain resolved desired enablement and normalized non-secret configuration/policy values plus their digests; receipts record desired and observed values. Along with externally resolved versioned secret references, this lets support reproduce a release and distinguish it from local drift.

Ongoing users, assignments, product records, and secret values are not copied into a lockfile. Administrator enrollment creates ordinary Core runtime user, role-assignment, and redemption records; only the Supervisor Ledger's one-way verifier, non-secret claim identifier/generation/kind/lifecycle, reservation, operator authorization, and outcome are retained outside the Blueprint and lockfile. A declared module bootstrap is the narrow product-record exception: the lockfile records its module-specific schema version and normalized value or durable content-addressed input reference plus digest, and the installation receipt records the module's idempotent result. The owning module alone validates, applies, and reports drift for it; adopting a later UI edit into desired bootstrap state requires a new Blueprint/input revision. This supports reproducible Form catalogs or similar initialization without inventing a platform-wide content-pack type.

LLMs use Feature Declarations plus the same versioned catalog, schema, plan, apply, and module-owned configuration or product APIs as other clients. They do not write databases, improvise unrecorded deployment steps, or bypass validation. Future MCP or agent adapters are thin clients over these ordinary platform contracts, not a separate source of product truth.

## Deployment And Support Model

An installation uses a same-origin gateway so the browser experiences one application even though Core and modules are separate processes. Modules may be released and supported independently, while an Application Release records the exact supported combination deployed for one application.

The platform must support:

- independent module build, release, migration, health, and rollback
- supervised Core and gateway install, upgrade, health-gated traffic switch, rollback, and recovery
- installation-scoped module instances rather than a required multi-tenant module runtime
- reproducible application releases from lockfiles plus externally resolved versioned secret references
- contained degraded behavior when one module is unavailable
- per-module and whole-installation backup, restore, diagnostics, and version inventory
- local operation without a mandatory central Tessara SaaS control plane

A central catalog, signing service, or fleet manager may be added later, but it is not required for an installation to operate.

## Reference Application And Transition

The current first-party feature flow remains valuable as a reference application:

```text
Forms/Workflows -> Responses -> Datasets -> Components -> Dashboards
```

That diagram describes product capability and data flow, not deployment topology. Forms, Workflows, Responses, Datasets, Components, and Dashboards become separate full-stack modules. Organization, users, sessions, RBAC, the shell, and the module control plane remain in Core.

The current Rust crates, single Axum service, shared database, and root-owned feature routes are a transition baseline. Existing feature-crate boundaries are useful extraction seams, but compile-time separation is not the target. During Sprint 6A, current areas may publish explicitly non-installable `transitional_in_process` contribution descriptors for discovery, contracts, security capabilities, and navigation. A descriptor may reserve a future Module Definition identity, but it creates neither a Module Release nor a Module Instance, does not satisfy `tessara-oci-v1`, and cannot be materialized by the Supervisor.

When a first extracted module temporarily consumes an in-process provider, the current Core Release may expose a narrowly versioned, first-party Core Release compatibility contract. That binding is trusted as a Core Release contract, not as a module provider, and is prohibited in new external application Blueprints. Typed references to its records use `core_installation` ownership and a transition-specific resource type; they never pretend the descriptor owns a Module Instance.

Physical extraction creates a real Module Release/Instance and explicitly migrates both data and references. The new provider emits an old-to-new mapping; each consumer rewrites its own stored references through a versioned rebinding/migration contract; receipts prove completeness; and the Core compatibility adapter remains read-only until all consumers have moved. Old Core-owned references retain their original owner/type and resolve with an explicit migrated/retired outcome rather than silently becoming module-owned. This deliberate pre-pilot migration is permitted because no production application depends on the transition layout.

Because no production application depends on the current internal database layout, Tessara may restructure data and references directly during this transition. The project should use that freedom to establish clean ownership boundaries instead of preserving accidental coupling.

## Explicit Non-Goals

- requiring every installation to run as part of one monolithic multi-tenant SaaS
- supporting multi-cluster placement or module-selected external relational databases in v1
- supporting non-OCI or module-defined executable deployment profiles in v1
- promising hot-swappable provider implementations with equivalent semantics or automatic data portability
- treating navigation visibility as authorization
- allowing direct cross-module database access for convenience
- making Core the owner of module-specific versioning, mutation, publication, or lifecycle decisions
- introducing a generic second package model for catalogs, templates, instruments, or batch definitions
- allowing modules to create authoritative roles or silently expand user permissions
- requiring runtime-loaded UI code, iframes, or browser-side remote-module execution; full-page same-origin module routes are the default

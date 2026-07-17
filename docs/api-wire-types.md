# API And Module Wire Types

Tessara's current single-service implementation keeps many request and response DTOs in `tessara-api` and mirrors JSON shapes near the Leptos screens that consume them. That remains acceptable for untouched transitional routes. It is not the target contract model for independently deployed modules.

## Direction

- Platform envelope types such as `ResourceReference`, `SemanticDestination`, authenticated `ShellContext`, and module lifecycle/status envelopes must have one authoritative, versioned home in the shared platform contract/SDK.
- Every module must own the versioned schemas for its Feature Declarations, public APIs, events, configuration, advertised resource types, named destinations, destination parameters, and any optional bootstrap contract.
- Public contract schemas must be browser-safe and implementation-neutral. They must not expose ORM entities, database keys without domain meaning, internal table layout, or service-local error strings.
- Consumers should use generated clients or narrow contract crates produced from the provider's schema. Sharing a contract type does not transfer product or persistence ownership.
- API and event envelopes must carry contract version, correlation identity, and stable success or error variants where appropriate.
- Cross-module requests must carry verifiable application-installation, original-actor when present, and presenting-service context plus audience-bound grants that preserve each security-capability-to-scope binding and declared dependency/contract/action, or an exact Core authorization-decision receipt. Wire types must not expose independent capability and scope sets, reusable browser session secrets, or another module's database credentials. A caller exchanges authority through Core for each downstream audience rather than forwarding an upstream audience credential; service-only authority is limited to explicitly authorized system jobs.
- Cross-boundary resource relationships must use the platform `ResourceReference` shape: installation, tagged owner kind (`core_installation` or `module_instance`), authoritative owner identifier, resource type, and resource identifier.
- Cross-boundary links must use semantic destination owners, names, and typed parameters rather than absolute or deployment-relative URLs saved as product data.
- Provider contracts must expose access plus tagged owner, owner-data where applicable, resource-identity, provider-defined resource-lifecycle, compatibility, and runtime-availability outcomes as separate fields with `undisclosed` and `not_evaluated` variants. Core-owned references use the Core-owner variant rather than Module Instance fields; evaluated unknown or cross-installation owners use explicit unknown/mismatch values. A tombstoned/destroyed Module Instance must not share one generic `tombstoned` value with a product resource whose live provider reports a tombstoned lifecycle state.
- After installation and audience validation, unauthorized or authorization-not-evaluated resolution must fail closed and return one stable non-disclosing envelope; caller-visible resource-specific owner/identity/lifecycle fields remain `undisclosed`, while detailed reasons require separately authorized diagnostics. A provider unable to evaluate a resource field after authorization returns `not_evaluated`, not `unknown_resource`. Contract tests must compare known and random identifiers under the conformance profile's defined timing method/tolerance.
- Events and cached read models must include the revision, sequence, or reconciliation information required by the provider's declared change-observation contract.
- Configuration UI, Blueprint automation, and LLM clients must use the same module-owned configuration schema and validation responses.
- Blueprint/lockfile/Materialization Plan/receipt types must carry desired and observed module enablement separately from deployment, health, navigation display policy, and authorization; emergency disablement is an explicit drift/override record rather than an unversioned boolean.
- Administrator Enrollment Claim contracts must version the Core Administration Capability Floor and designated Administrator Enrollment Role binding; distinguish `initial` and `recovery`; and carry installation, identifier, generation, expiry, lifecycle, reservation/idempotency, and signed-result fields. They must keep the once-displayed secret write-only and out of status, audit, receipt, log, recovery, and diagnostic schemas; the Supervisor may store only a one-way verifier plus non-secret metadata. Expired, revoked, replayed, reserved-by-another-redemption, consumed, and cross-installation failures share a non-disclosing caller projection.
- Playwright permission scenarios and provider/consumer contract tests must change whenever a permission-controlled wire contract changes.

## Contract Ownership

Use these ownership rules:

- Core owns wire contracts for Organization, users, sessions, RBAC, module inventory, Blueprints, lockfiles, navigation resolution, and installation-level status.
- The platform SDK owns the versioned deterministic `MaterializationPlan`, separate `ApplyAuthorizationEnvelope`, `tessara-oci-v1` Deployment Profile, Supervisor ledger/read-back/status, and installation-receipt wire contracts; the out-of-process Supervisor is authoritative for observed materialization state.
- A module owns all wire contracts for its product resources, commands, events, configuration, health, and diagnostics.
- A consumer may define a presentation-specific view model, but it must adapt from the provider contract rather than make the provider depend on the consumer.
- A shared SDK may own transport envelopes, authenticated context verification, resource-reference primitives, semantic-destination primitives, health conventions, and generated-client support. It must not become a shared domain-model package.

## Compatibility

Contract versions and application versions are related but distinct. A module release must declare the exact versions of the functional contracts it provides and requires. Core validates compatible bindings before enablement and records the resolved versions in the application lockfile.

Additive schema evolution is not automatically safe. Compatibility must account for required fields, enums, error variants, authorization semantics, lifecycle outcomes, idempotency, ordering, and event replay behavior. Provider and consumer tests must prove every supported version range.

## Current Core Organization Metadata Schema

- Core exposes `GET /api/node-types/{node_type_id}/metadata-fields` to authenticated actors with effective `hierarchy:read`; the historical `hierarchy:manage` implication and `admin:all` therefore qualify as well.
- The response is the ordered list of field-schema rows required by the existing Organization create/edit surfaces: `id`, `node_type_id`, `node_type_name`, `key`, `label`, `field_type`, and `required`.
- This read contract does not expose node values, scoped Forms, node-type relationship administration, or any mutation authority. The complete `GET /api/admin/node-types/{node_type_id}` definition and every node-type/metadata mutation remain `admin:all`-only.
- Unknown node types return the normal not-found envelope; anonymous and insufficient-capability requests retain the established authentication/authorization envelopes.

## Sprint 6A Concrete Platform Types

- `tessara-module-contract` owns the framework-neutral v1 identities, Manifest/transition declarations, typed resource-reference and semantic-destination primitives, and `ResourceResolutionV1`.
- `ResourceResolutionV1` keeps `access_state`, tagged `owner_state` plus owner-data state where applicable, `resource_identity_state`, provider-defined `resource_lifecycle_state`, `compatibility_state`, and `availability_state` independent. It replaces any combined resource-resolution outcome enum.
- Unauthorized and access-not-evaluated resolution uses one restricted projection: every resource-specific dimension is `undisclosed`. Deserialization rejects a restricted envelope that discloses another dimension.
- A Navigation Contribution declares `required_capabilities_any_of`; Core owns `admin:all` implication and final actor authorization. Display eligibility never becomes route/API authorization.
- The authenticated `GET /api/auth/session` account projection retains `capabilities` as the established flat effective-key set and adds `global_capabilities` as the subset of those effective keys backed by installation-global authority. Shell fallback uses flat product keys but accepts direct `modules:*` keys only from `global_capabilities`; `admin:all` remains the universal global sentinel. Capability-to-scope bindings/details remain server-internal, and the established `scope_nodes` summary is unchanged.
- Sprint 6A-UI's schema-v2 navigation policy is a complete Core-owned projection of ordered groups and exact destination placements. `core.main` and `core.admin` are required, reorderable groups; administrators with effective global `modules:manage_navigation` may add, rename, reorder, and delete empty custom groups, and may hide or move only destinations whose catalog protection flags permit it. Module Management is the protected Core destination `core.admin.modules`, with key `module_management`, route `/administration/modules`, default group `core.admin`, effective-global `modules:read` eligibility, and the canonical `Blocks` shell icon. Policy writes submit group identity/label/order plus destination identity/group/visibility/order only; Core remains authoritative for labels, routes, capabilities, ownership, availability, and protection flags. Readers with effective global `modules:read` may inspect the policy; mutations require effective global `modules:manage_navigation`.
- Sprint 6A defines `ModuleRelease` and `ModuleInstance` public types only. Persistence, mutation, materialization, and a supported real Module Manifest artifact begin in Sprint 6B.
- The transitional Migration descriptor is `retired`, with no route, navigation, provider, resource, or executable destination. Its continued discovery is historical/support context and does not authorize restoration.

## Transitional Migration Rule

Do not introduce route-local raw fetch logic for mutations or authenticated JSON parsing. For current in-process feature routes, add typed client functions over the existing policy-neutral HTTP helper. When a feature becomes a module, move its stable public shapes into the module-owned contract package or schema and generate/adapt clients from that source.

An in-process `transitional_in_process` descriptor creates no Module Release or Module Instance. A first-party extracted consumer may temporarily use a versioned Core Release compatibility contract, but returned references must stay `core_installation`-owned with transition-specific resource types. Extraction requires a provider old-to-new mapping, consumer-owned rebinding contract, completeness receipts, and an explicit migrated/retired old-reference result; wire code must never reinterpret the old owner/type as the new module owner/type.

Do not extract a type merely to eliminate duplicate Rust definitions. Extract it when it is a genuine supported wire contract with a clear owner, compatibility policy, and consumer-test obligation.

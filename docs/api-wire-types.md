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

## Transitional Migration Rule

Do not introduce route-local raw fetch logic for mutations or authenticated JSON parsing. For current in-process feature routes, add typed client functions over the existing policy-neutral HTTP helper. When a feature becomes a module, move its stable public shapes into the module-owned contract package or schema and generate/adapt clients from that source.

An in-process `transitional_in_process` descriptor creates no Module Release or Module Instance. A first-party extracted consumer may temporarily use a versioned Core Release compatibility contract, but returned references must stay `core_installation`-owned with transition-specific resource types. Extraction requires a provider old-to-new mapping, consumer-owned rebinding contract, completeness receipts, and an explicit migrated/retired old-reference result; wire code must never reinterpret the old owner/type as the new module owner/type.

Do not extract a type merely to eliminate duplicate Rust definitions. Extract it when it is a genuine supported wire contract with a clear owner, compatibility policy, and consumer-test obligation.

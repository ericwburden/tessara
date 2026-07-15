# Sprint 6A Plan: Module Contract And Core Control Plane Slice

Kickoff status: started from clean `main` at `3625d4de52c5856e4ac3bc642a9422a029e9f375` on 2026-07-13.

Kickoff defaults:

- Branch: `codex/sprint-6a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6a`
- Plan artifact: `docs/sprints/sprint-6a-plan.md`
- Roadmap source: the sole heading marked `(Next)`, `Sprint 6A: Module Contract And Core Control Plane Slice`

## Sprint Summary

Sprint 6A establishes Tessara's module contract and the first Core-owned control-plane slice without pretending that the current in-process feature areas are deployable applications. It represents the current application as one stable Application Installation, its observed Core runtime, explicitly non-installable transition contributions, and any future real module inventory entries through distinct types.

The sprint delivers:

- versioned, machine-readable module and transition contracts with deterministic validation;
- Core persistence and services for module identity, transition inventory, discovery projections, dependency findings, security-capability provenance, semantic destinations, typed resource references, and navigation policy;
- stable transition descriptors for Forms, Workflows, Responses, Datasets, Components, Dashboards, and Migration;
- a Core-owned Module Management directory and detail experience;
- contributed security-capability provenance in Core role management;
- dynamic shell navigation whose display policy and ordering remain separate from route authorization;
- typed transition adapters that preserve current product routes and database behavior while making future ownership boundaries explicit.

Sprint 6A does not deploy a real module. Installation Supervisor materialization, signed artifact acquisition, OCI execution, same-origin process routing, per-module databases, first-administrator enrollment, scope-bound cross-module grants, and real Module Releases/Instances begin in Sprint 6B or later roadmap slices.

## Approved Decisions And Behavior-Preservation Contract

The following decisions are part of the Sprint 6A implementation contract:

- Sprint 6A defines the public `ModuleRelease` and `ModuleInstance` contract types only. It does not add Module Release/Instance persistence, repositories, read models, mutation APIs, seed rows, or materialization behavior. Those begin in Sprint 6B.
- Transitional Migration is `retired`. The former surface was deliberately withdrawn and remains discoverable only for historical/support context. It has no current route, executable destination, provider, or navigation contribution; restoring it requires a new approved product decision and roadmap scope.
- Navigation contribution visibility and ordering are editable only inside the contribution's existing Core-assigned reorder band: `main_between_organization_and_operations` for Forms, Workflows, and Responses; `main_after_operations` for Components and Dashboards; and `admin_between_administration_and_module_management` for Datasets. A contribution cannot cross a Core anchor or change group in Sprint 6A. Permanent Core destinations cannot be hidden or reordered, and grouping changes remain deferred.
- `modules:read` and `modules:manage_navigation` are installation-global capabilities. A scoped assignment does not satisfy either capability. `modules:manage_navigation` implies `modules:read`, and `admin:all` implies both.
- `Module Management` is a permanent, policy-immutable Core navigation item in the `Admin` group. Its default slot is after the existing Datasets item so every pre-Sprint-6A item's relative order remains unchanged. It is visible when the actor has effective installation-global `modules:read`; therefore `modules:manage_navigation` and `admin:all` holders also see it through implication. The existing `Administration` item remains separately gated by `admin:all`.
- A `modules:read`-only actor can use the Module Management directory, detail, descriptor, and read-only navigation-policy presentation but has no enabled show/hide/reorder control. Only effective global `modules:manage_navigation` authorizes those controls and the policy `PUT`. Sprint 6A introduces no generic `modules:manage` capability.
- `admin:all` is the sole exception that may coexist with both scope-aware and installation-global capability rows in one role. The complete role is classified and assigned as installation-global, every other mixed scope-mode bundle is rejected, and the seed-owned `admin` role still converges to the cleaner `[admin:all]` set.
- The immutable historical upgrade precondition is `sprint-5a-role-capabilities-v1+sha256.7725e889996a`, with full canonical SHA-256 `7725e889996a73a5655c57106aca6e12d9a5f95e9103f14d7b0fd50fbac96988`: Sprint 5A `admin` has all 20 historical capabilities, `operator` its exact 10, and `respondent` its exact 2. The populated fixture and integration-test literals cannot be changed together without also tripping this independently recorded digest/version review boundary.
- The replaceable built-in role-membership contract is `sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68`, with full canonical SHA-256 `2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600`: `admin = [admin:all]`; `operator = [hierarchy:read, forms:read, workflows:read, workflows:manage, submissions:respond, submissions:manage, operations:view, datasets:read, components:read, dashboards:read]`; `respondent = [submissions:read_own, submissions:respond]`. Only membership in `role_capabilities` for those three named roles is seed-owned and replaceable. Their role rows/IDs and every assignment, account, session, user-role association, user-managed role, and user-managed membership remain exact invariants. Existing authorized role-edit UI/API actions are not blocked: an edit to built-in membership can take effect for the running installation, but the next successful startup intentionally reconverges it to this seed contract; durable customized bundles belong in user-managed roles. Changing the seed contract itself requires a new version/digest, exact-set test updates, and an explicit test-change-log entry; it is never an incidental seed edit.

Sprint 6A is an additive Core administration and discovery slice, not a complete user-visible no-op. The only intentional user-facing additions are Module Management, capability-provenance display, and administrator navigation display/order controls. Everything else below is a frozen compatibility baseline.

| Area | Allowed Sprint 6A change | Behavior that must remain unchanged |
| --- | --- | --- |
| Product workflows | Typed transition adapters and discovery metadata | Existing create/read/update, draft, publish, assignment, response, review, dataset materialization/preview, Component execution/viewing, and Dashboard composition/viewing semantics |
| Product APIs and data | Additive Core module/discovery APIs, the narrow hierarchy-readable Organization metadata-field schema endpoint required by the existing node editors, and forward-only Core tables | Existing paths, request/response meaning, stable error envelopes, product tables, foreign keys, identifiers, row visibility, and transaction behavior; the full node-type definition and all node-type/metadata mutations remain `admin:all`-only |
| Authorization | Add global module-administration capabilities | Existing capability keys, user-managed role-capability mappings, scope/ownership/delegation semantics, route guards, and effective product access. Deterministic built-in seed roles remain versioned seed data and may be refreshed to the declared current seed contract |
| Shell | Resolve contributed items from Core, add the fixed `Module Management` item in `Admin`, and allow an explicit administrator policy mutation | Before an administrator changes contribution policy, every pre-existing item retains its desktop/mobile label, group, relative order, visibility, and direct-route authorization; the sole additive item is policy-immutable `Module Management`, shown only with effective global `modules:read` |
| Application routes | Add `/administration/modules` and `/administration/modules/:definition_id` | `/`, `/login`, Organization, Forms, Workflows, Responses, Operations, Datasets, Components, Dashboards, and existing Administration routes retain native SSR ownership and supported behavior |
| Migration | Add a retired historical discovery record with a stable finding | No `/migration` route, navigation item, executable destination, provider claim, or restored legacy product behavior is introduced |

“Still works” means the same supported inputs, outcomes, authorization decisions, persisted effects, and stable errors—not merely an HTTP 200 response or a visually similar screenshot. The authoritative route and behavior inventory for this sprint is [the Sprint 6A regression matrix](./sprint-6a-regression-matrix.md).

## Test Evidence And Change Control

Tests are durable executable contracts, not implementation debris. Do not delete, skip, ignore, weaken, broaden tolerances, loosen selectors, increase retries/timeouts, regenerate expected fixtures, or rewrite an existing test merely because Sprint 6A implementation makes it fail. A changed expectation must:

1. cite an approved requirement or contract decision;
2. explain why the former assertion is no longer correct;
3. preserve equivalent or stronger positive and negative coverage; and
4. be listed explicitly in the closeout evidence with its replacement proof.

Additional rules:

- A failing frozen-baseline test is presumed to identify an implementation regression. Fix production code unless an approved decision changes the behavior.
- Characterization tests for the existing shell, route guards, user-managed role mappings, versioned built-in seed-role mappings, capability implications, product flows, and populated Sprint 5A database land and pass before the corresponding refactor.
- New behavior uses red/green proof where practical: the new test fails because the capability is absent or incorrect, then passes after implementation.
- Accepted v1 wire fixtures and expected digests are immutable. A correction requires a new versioned fixture plus a documented compatibility/migration decision; fixtures are never auto-accepted.
- Fixture acceptance occurs only when ordered implementation step 2 is reviewed, every canonical/invalid fixture and digest test is green, and the accepting commit is recorded in closeout evidence. Pre-acceptance corrections remain visible in the test change log; after that point, changing accepted bytes requires a new versioned fixture rather than rewriting history.
- Invalid semantic fixtures that deserialize assert exact finding codes, paths, messages, and deterministic order. Structural/Serde rejection fixtures assert the exact documented error category and offending field, variant, or profile token; line/column-dependent Serde prose is not a public stable contract. Generic `is_err()` coverage alone is insufficient in either case, and deserialization errors are not mislabeled as semantic `ValidationFinding` values.
- Database suites run with a named disposable `TEST_DATABASE_URL`; the destructive populated migration proof requires an independently named disposable `SPRINT_6A_UPGRADE_DATABASE_URL`, and fresh-start/seed-lock proof requires a third independently named disposable `SPRINT_6A_FRESH_DATABASE_URL`. All three must differ so the representative populated-upgrade fixture remains independently inspectable. Gate 4 does not use that fixture database: it uses a fourth disposable target restored from a Sprint 5A database that already contains the acceptance actors and demo assets, validates the restored target with `OriginalAfterRestore`, and lets the clean closing image apply migration 3 while `-SkipSeed` is in force. Absence or an empty value fails the proof instead of skipping it. Before either destructive reset, the proof queries `current_database()` and requires a token-bounded disposable marker (`test`, `tests`, `testing`, `upgrade`, `clone`, `rollback`, or `sprint-6a`/`sprint6a`), so incidental substrings such as `latest`, `contest`, or `production_upgradeable` are rejected. The resets also require the exact acknowledgement `SPRINT_6A_CONFIRM_DESTRUCTIVE_UPGRADE_RESET=I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET`. A skipped database test is not a pass. Closeout records zero unexpected ignored, skipped, filtered, or retried-to-green tests.
- Every acceptance criterion maps to a durable unit, contract, API, migration, SSR, Playwright, smoke, or UAT artifact in the acceptance-to-proof matrix below.
- Every modification to an existing test expectation or pre-acceptance fixture is recorded in [the Sprint 6A test change log](./sprint-6a-test-change-log.md); closeout must reconcile that log with the Git diff.

## Sprint Specifications

### Product Outcome And Boundaries

- The current Tessara application is inspectable as Core plus discoverable transition contributions.
- A transition contribution is never shown or serialized as a Module Release or Module Instance.
- Existing Forms, Workflows, Responses, Datasets, Components, Dashboards, Administration, and Organization routes remain operational throughout the sprint.
- Core remains authoritative for Organization, users, sessions, roles, role assignments, capability registration, navigation policy, module inventory, and access/composition audit.
- Feature Declarations, functional contracts, security capabilities, lifecycle state, navigation display policy, and actor authorization remain separate concepts and storage/projection dimensions.
- Module product rules such as publication, immutability, supersession, and lifecycle semantics remain owned by the relevant feature/module boundary rather than Core.

### Framework-Neutral Contract Boundary

- Add a pure `tessara-module-contract` crate. It owns the public contract and validation types shared by Core, future modules, SDKs, the Supervisor, and machine clients; HTTP and persistence remain in `tessara-api`.
- Define versioned types for Application Installation identity, Core Release, Core runtime observation, Module Definition, Module Release, Module Instance, module manifests, transition descriptors, inventory entries, and typed findings.
- Definition, Release, and Instance states are represented independently:
  - Definition: registered.
  - Release: trusted and compatible.
  - Instance identity: live or tombstoned.
  - Instance operation: installed, deployed, configured, ready, enabled, and healthy.
  - Instance data: retained or destroyed.
- Do not add a composite `module_status` field that can erase those distinctions.
- Stable namespaced identifiers use portable text values. The initial reserved Module Definition identities are `tessara.forms`, `tessara.workflows`, `tessara.responses`, `tessara.datasets`, `tessara.components`, `tessara.dashboards`, and `tessara.migration`.
- Rust/Serde types plus checked-in valid and invalid fixtures are the canonical v1 wire definition for this sprint. A separately generated JSON Schema is deferred unless implementation proves a concrete consumer requires it.
- V1 namespaced identifiers contain lower-case ASCII letters or digits separated by `.`, `:`, `_`, or `-`; they cannot start/end with or repeat separators. Deserialization rejects unknown fields, unsupported schema versions, non-canonical identifier case, and invalid enum values.
- The wire discriminator for a transition inventory entry is exactly `transitional_in_process`. The source descriptor availability values are distinct and must never be inferred from one another: `active_in_process` means the declared current surface executes in the shared Core process; `unavailable` means an intended current transition surface still has declarations but is temporarily unable to execute; `retired` means the former surface was deliberately withdrawn, has no live declarations, and cannot return without a new product decision.
- The seven authoritative transition descriptors are checked-in UTF-8 JSON documents with repository-normalized LF line endings and no byte-order mark. The stored source digest is lower-case `sha256:<64 hex>` over the exact checked-in source bytes; formatting-only source changes intentionally change the digest. Normalized projections store that digest as their provenance key and can never be served against a different source document.
- Expected source digests are checked in with the fixtures and asserted byte-for-byte across restarts and supported development platforms. Semantic round-trip tests are additional proof; they do not replace exact source/digest proof.
- Validation findings are stable ordered wire values. Their `code`, JSON-style `path`, `message`, and declaration-order position are contract assertions.
- [The Sprint 6A transition catalog](./sprint-6a-transition-catalog.md) freezes the feature, contract, dependency-binding, resource-type, route, navigation, capability, availability, and finding identifiers before persistence/API work proceeds.

### Manifest And Transition Descriptor Contracts

- Define `ModuleManifestV1` now even though Sprint 6B supplies the first real manifest. It covers:
  - Module Definition identity, release version, publisher, support metadata, and manifest schema version;
  - compatible Core Release, Shell Context, UI SDK, and design-system versions;
  - the `tessara-oci-v1` runtime image, optional migration image, immutable digests, platform/architecture, commands, listen/service-registration declaration, configuration and secret injection points, separate runtime/migration identities, readiness/liveness probes, graceful shutdown, and resource requests/limits;
  - machine-readable Feature Declarations;
  - required/provided functional contracts, dependency constraints, binding keys, resource types, product/administration routes, navigation contributions, and optional Home/work-discovery/search contributions;
  - namespaced security capabilities, configuration schema, and health/readiness contracts.
- Validation rejects unsupported schema/deployment-profile versions; malformed or duplicate namespaces; missing immutable digests; invalid compatibility ranges; duplicate routes/resources/capabilities; unresolved feature links; invalid contract/dependency bindings; and deployment URLs where semantic destinations are required.
- `TransitionalContributionDescriptorV1` is a separate tagged type, not a permissive Module Manifest. It may reserve a future Module Definition identity and describe current features, contracts, dependencies, capabilities, resources, and Core-owned destinations.
- A transition descriptor:
  - creates no Module Release or Module Instance;
  - carries no installable artifact or runtime assertion;
  - is never eligible for Supervisor materialization;
  - cannot satisfy a dependency as a module provider;
  - uses `core_installation` ownership for typed resource references and Core-owned semantic destinations;
  - reports module lifecycle dimensions as not applicable rather than false installed/deployed/enabled/healthy claims.
- Store and expose the validated versioned source document and digest alongside normalized query projections so support and machine clients can prove what produced an inventory view.

### Current Transition Catalog

- Seed and idempotently synchronize the seven exact source descriptors defined by the checked-in catalog fixtures and [transition catalog](./sprint-6a-transition-catalog.md). Implementations must not invent alternate identifiers in repositories, DTOs, UI code, tests, or seed helpers.
- Forms advertises form/field authoring, FormVersion publication and lookup contracts, Form/FormVersion resource types, current routes, and `forms:*` capabilities.
- Workflows advertises workflow authoring, assignment, execution, and Workflow/WorkflowVersion contracts and `workflows:*` capabilities; its current FormVersion relationship is reported as transition-internal rather than a satisfied module provider binding.
- Responses advertises response start/draft/submit/review and Response contracts. The current `submissions:*` capability namespace is declared explicitly as the legacy/current namespace rather than renamed during this slice.
- Datasets advertises authoring, revision/publication, materialization/preview, Dataset/DatasetRevision or major-line contracts, and `datasets:*` capabilities.
- Components advertises authoring, version/publication, execution/viewing, ComponentVersion contracts, and `components:*` capabilities.
- Dashboards advertises composition/viewing, Dashboard contracts, and `dashboards:*` capabilities.
- Migration remains discoverable because it is in the roadmap contract, but the repository has no live `/migration` route. Its descriptor has `availability = retired`, no feature/provider claim, no resource, route, capability, or navigation declaration, and the normalized finding code `transition_destination_retired`. Sprint 6A does not restore or invent a Migration product surface.
- The existing `/operations` route remains a Core-owned read-only operational status projection under the durable roadmap rule for `operations:view`. Sprint 6A does not invent an eighth Module Definition for that cross-feature status view.
- Re-synchronizing the catalog preserves capability row IDs, role-capability mappings, navigation administrator policy, and audit identity.

### Core Release Observation

- Define an exact Core Release contract that requires immutable component artifacts/digests, including its gateway component.
- The current pre-Supervisor development runtime does not have trustworthy Core/gateway artifact digests. Persist and display it as a Core runtime observation with unresolved release provenance and a separate finding rather than fabricating a valid Core Release.
- Sprint 6B replaces that transition observation with the first Supervisor-materialized exact Core Release record.

### Persistence, Services, And APIs

- Add forward migration `003_module_control_plane.sql` after `002_dashboard_placement_capacity.sql`. `001_baseline.sql` and `002_dashboard_placement_capacity.sql` remain byte-identical and protected by migration-integrity tests.
- Persist the stable Application Installation identity, Core runtime observation, future-definition identity reservations, versioned transition descriptor source bytes/digests, normalized discovery projections, capability provenance, dependency findings, navigation contributions/policy, and Core control-plane audit events.
- A future-definition reservation is not a registered `ModuleDefinition`, `ModuleRelease`, or `ModuleInstance`. Sprint 6A creates no Release/Instance tables or rows and exposes no Release/Instance repository, mutation, installation, enablement, or materialization path. Public Release/Instance Rust types exist only to freeze the future v1 contract boundary.
- Catalog synchronization is one transaction after migrations. First startup, repeated startup, concurrent startup, a malformed source descriptor, and a failure injected between source/projection/capability/navigation writes cannot leave partial catalog state.
- An unchanged source digest produces a no-op synchronization: it preserves source/projection row IDs, capability IDs and descriptions, role-capability rows, navigation overrides, and audit identity and emits no duplicate synchronization audit event.
- A changed source is rejected unless its stable reserved identifiers remain compatible with the catalog contract. Removal uses an explicit unavailable/retired decision; synchronization never silently deletes identity, policy, or audit history.
- Upgrade proof starts from a populated Sprint 5A database at migration 2 and first asserts truthful historical contract `sprint-5a-role-capabilities-v1+sha256.7725e889996a`: all 20 Sprint 5A capabilities with exact descriptions, `admin` mapped to all 20, `operator` mapped to its exact 10, `respondent` mapped to its exact 2, plus an independently assigned user-managed custom role. It records product row counts and identity/mapping snapshots, applies migration 3, restarts twice, and proves product data plus all role rows/IDs, user-managed mappings, assignments, accounts, sessions, and user-role associations remain unchanged. Startup transactionally replaces only the three built-in membership sets with exact contract `sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68`; stale and missing membership repair, repeated/concurrent startup, and fresh-data exact-set proof all use the same runtime declaration. Fresh-data proof is separate and cannot substitute for the populated upgrade path.
- The migration is forward-only and additive. An unmodified historical Sprint 5A package cannot be redeployed against a database that records migration 3: its SQLx migrator does not know version 3 and will reject the database. Before rollout, build and retain a tested compatibility rollback package containing the exact Sprint 5A application binary/code plus the immutable Sprint 6A migration set through `003_module_control_plane.sql`; the application ignores the additive tables while SQLx recognizes the ledger. If the migration transaction itself fails, it rolls back fully. Using the original historical package requires restoring the pre-upgrade backup; no path improvises a down migration or edits an applied checksum.
- Add a reproducible rollback-packaging command and retained manifest that records the Sprint 5A application commit/binary digest, closing Sprint 6A migration-file digests, package/image digest, builder version, and build command. A focused rollback validator starts that exact Sprint 5A-code package on the representative upgraded fixture clone, proves product reads/writes still work, and separately proves backup restore before the original Sprint 5A package is started. Retained JSON captures every built-in role/capability pair plus a deterministic canonical snapshot digest before and after each startup; all module control-plane state and every user-managed role mapping remain covered by a separate exact invariant fingerprint. The representative upgraded fixture must begin at `sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68`; rejected original-package startup leaves that set unchanged; successful Sprint 5A-code compatibility startup converges to historical contract `sprint-5a-role-capabilities-v1+sha256.7725e889996a` (admin-20/operator-10/respondent-2). This is the one documented compatibility exception: only redundant direct product-capability rows return on `admin`, `operator`/`respondent` are identical across the contracts, and effective admin authority is unchanged because `admin:all` was already present. A restored migration-1/2 demo clone must prove exact Sprint 5A membership both before and after original-package startup; only after that proof may the clean closing image upgrade this separate target for Gate 4.
- Add a `modules` API boundary with focused catalog, repository, service, DTO, validation, destination, reference, and route responsibilities rather than expanding `lib.rs` or `users/mod.rs` into a control-plane monolith.
- Add the routes below. DTOs include `schema_version` where they are public discovery/resolution contracts, and all errors use stable Core error codes rather than database/internal strings.

| Method and path | Authority | Success/side-effect contract | Stable negative behavior |
| --- | --- | --- | --- |
| `GET /api/node-types/{node_type_id}/metadata-fields` | effective `hierarchy:read`; existing `hierarchy:manage` and `admin:all` implications qualify | Read-only, top-level ordered `Vec<NodeMetadataFieldSummary>` containing exactly `id`, `node_type_id`, `node_type_name`, `key`, `label`, `field_type`, and `required`; exposes no node values, scoped Forms, relationship administration, or mutation authority | `401` anonymous; established `403` for insufficient authority; established `404` for an unknown node type; `GET /api/admin/node-types/{node_type_id}` and every node-type/metadata mutation remain `admin:all`-only |
| `GET /api/admin/modules` | global `modules:read` | Versioned inventory containing the Application Installation/Core observation and seven transition entries | `401` anonymous; `403 modules_read_global_required` for absent/scoped-only authority |
| `GET /api/admin/modules/{definition_id}` | global `modules:read` | Complete normalized detail projection with source digest | Authorization is evaluated before lookup; authorized unknown ID returns `404 module_definition_not_found` |
| `GET /api/admin/modules/{definition_id}/descriptor` | global `modules:read` | Exact validated source bytes/content type plus a quoted HTTP `ETag` whose opaque value is the stored source digest | Same authorization/not-found ordering as detail; source and projection digest must match; wire header is `"sha256:<64 hex>"`, while the catalog value remains unquoted |
| `GET /api/admin/navigation-policy` | global `modules:read` | Current revision plus immutable group, Core-assigned `reorder_band`, Core-anchor context, and contribution visibility/per-band order values | `401`/`403` as above |
| `PUT /api/admin/navigation-policy` | global `modules:manage_navigation` (implies read) | One atomic, idempotent collection update containing `expected_revision` and every mutable contribution's `id`, existing `group`, existing `reorder_band`, `visible`, and zero-based per-band `order`; emits one attributable audit event when state changes | `403 modules_manage_navigation_global_required` for absent/scoped-only manage authority; reject missing/duplicate/unknown IDs, group or band changes, Core-item mutations, and invalid per-band order; `navigation_policy_core_item_immutable` identifies an attempted Module Management/Core mutation; `navigation_policy_band_change_forbidden` identifies a cross-anchor/band attempt; `409 navigation_policy_revision_conflict` identifies a stale revision; no-op returns the current projection without an audit event |
| `GET /api/shell/navigation` | any authenticated session | Per-user resolved desktop/mobile model composed from permanent Core items, eligibility, stored policy, and the actor's existing product capabilities | The endpoint itself never requires `modules:read`; anonymous returns `401`. Its fixed Module Management item appears only with effective global `modules:read`, and a visible contribution never bypasses its product capability |
| `POST /api/platform/destinations/resolve` | any authenticated session | Validates owner/route/typed parameters and resolves a same-origin current Core path | Unknown, mismatched, unavailable, unauthorized, and not-evaluated cases use stable structured/non-disclosing results; no arbitrary or stored deployment URL is returned |
| `POST /api/platform/resource-references` | any authenticated session plus the adapter's existing product capability | Constructs an installation-bound, `core_installation`-owned representative transition reference | Rejects mismatched installation/owner/type/id and evaluates authority before resource existence |
| `POST /api/platform/resource-references/resolve` | any authenticated session | Returns `ResourceResolutionV1` with independent access, owner/data, resource identity, lifecycle, compatibility, and availability dimensions | Syntactically valid unauthorized/not-evaluated requests return the same restricted `200` envelope for known and random identifiers; malformed wire input returns stable `400` |
| `GET /administration/modules` and `GET /administration/modules/:definition_id` | global `modules:read`; native document routes | Cookie-authenticated native SSR with a hydration bootstrap derived from the same authorized projection | Anonymous redirects to `/login`; insufficient global authority renders the established restricted route state; unknown detail uses the established not-found state |

- Add Core-owned `modules:read` and `modules:manage_navigation` capability rows without renaming or replacing existing capability rows. Stored assignments for these capabilities must be global; a scoped assignment is invalid and does not authorize a request. `admin:all` implies both and `modules:manage_navigation` implies read even if only the manage capability is stored.
- The Core-owned descriptions are exactly `Inspect module inventory and transition contribution metadata` for `modules:read` and `Manage installation navigation visibility and ordering` for `modules:manage_navigation`.
- Represent the scope rule explicitly in Core capability metadata as `installation_global`, with existing product capabilities retaining their current scope-aware behavior and `admin:all` recorded as installation-global. Because scope is stored on a role assignment rather than one capability, v1 roles reject mixed scope-aware and installation-global capability rows with `mixed_capability_scope_modes`, except for the confirmed sole universal-sentinel exception: a bundle containing `admin:all` may also contain redundant scope-aware rows and the complete role is classified and assigned as installation-global. The replaceable built-in admin seed still converges to the cleaner `admin:all`-only set. A role containing either module-administration capability may have only global assignments: adding one to a role that has scoped assignments, or attempting a scoped assignment for such a role, fails atomically with `global_capability_requires_global_role_assignment`. Non-admin actors combine a dedicated global module role with separate scoped product roles. The UI labels these constraints and the `admin:all` exception before save, preventing silent product-access widening or partial-bundle evaluation.
- For every pre-existing product capability, the Core capability row remains authoritative for key and description. A transition descriptor must use the exact catalog-frozen Core description; synchronization records provenance separately, never updates that row from descriptor text, and rejects a mismatch atomically with `transition_capability_description_mismatch`.
- A module/transition descriptor may advertise existing product capabilities for provenance, but it never creates, edits, assigns, deletes, or scopes a Core role. Role mutation remains on the existing Core API and authorization path.

### Functional Dependencies And Findings

- Feature Declarations, functional contracts, and security capabilities use distinct identifiers and validators. A matching feature label or capability never satisfies a functional dependency.
- Feature realization links must resolve to contracts, resources, routes, configuration, or capabilities declared by the same source document.
- Dependency evaluation reports missing provider, incompatible contract/version, ambiguous binding, cycle, transition-internal-only relationship, and not evaluated as separate outcomes.
- Current Workflows-to-Forms, Responses-to-Workflows/Forms, Datasets-to-current providers, Components-to-Datasets, and Dashboards-to-Components seams are declared for discovery.
- A transition descriptor may describe those current seams but never becomes a provider candidate for a real Module Release dependency. The UI explains that they are current in-process coupling to be replaced by future bindings.

### Typed Resource References And Semantic Destinations

- A typed resource reference contains the Application Installation identity, tagged owner kind, authoritative owner identity, namespaced resource type, and opaque resource identifier.
- Construction and deserialization reject missing/mismatched installation, owner, type, or identifier values. Consumers cannot reinterpret the owner or resource type after persistence.
- Transition adapters expose representative current resources—FormVersion, Workflow/WorkflowVersion, Response, Dataset/DatasetRevision or major line, ComponentVersion, and Dashboard—as `core_installation` owned transition resource types. Sprint 6A does not rewrite every existing foreign key.
- Resolution returns versioned `ResourceResolutionV1`; it is not a single status enum. The envelope contains independent `access_state`, tagged `owner_state` (including independent Module Instance data state where applicable), `resource_identity_state`, provider-defined `resource_lifecycle_state`, `compatibility_state`, and `availability_state` fields. Core owners use the Core-installation owner variant and never fake Module Instance/data state.
- Authorized resolution may vary those dimensions independently. Unauthorized and access-not-evaluated resolution use one restricted projection: every resource-specific dimension is `undisclosed`, the HTTP status and JSON keys are identical for known and random identifiers, and detailed reasons appear only in separately authorized diagnostics. Deserialization rejects a restricted envelope that discloses any other dimension.
- Non-disclosure timing proof uses a warmed local release build and disposable populated database, alternates at least 200 known/random requests per access state, and requires both median and p95 latency deltas to remain within the larger of 2 ms or 20 percent. A breach is investigated as a release blocker rather than hidden by widening the tolerance.
- A semantic destination contains an owner, stable named route, and typed parameters. It contains no host, port, scheme, or persisted deployment URL.
- A Core route registry resolves current transition destinations to existing same-origin paths. Unknown/mismatched destinations return structured findings rather than falling through to an arbitrary URL.

### Navigation And Authorization Separation

- Replace the shell's hard-coded contribution display list with a Core-composed navigation model while retaining permanent Core destinations.
- Split the current navigation logic into four explicit decisions:
  - route/API authorization from effective capabilities;
  - contribution lifecycle/route eligibility;
  - administrator display/order policy;
  - final shell visibility from eligibility, policy, and actor authorization.
- Hiding or reordering a contribution changes only navigation policy. It does not change roles, assignments, effective capabilities, route guards, descriptor state, or future module enablement.
- An authorized user may directly load a hidden product route. An unauthorized user cannot gain access because its navigation contribution is visible.
- Product destinations require an eligible product route. Administration/diagnostic destinations may remain discoverable for recovery where their lifecycle rules permit it.
- Desktop and mobile shells consume the same owned dynamic navigation model; API-provided labels, semantic destinations, grouping, and ordering must not rely on `&'static str` route metadata.
- Each contribution declares `required_capabilities_any_of` as an owned list, not one capability string. Final visibility requires at least one declared product capability (or Core's `admin:all` implication); the list is display eligibility only and never replaces route/API authorization. This preserves read/manage combinations and the intentional Dashboard rule that manage-without-read may load an issued editor route but does not receive the reader-directory navigation item.
- The initial contribution policy and every pre-existing Core item exactly match the pre-sprint `NAV_ITEMS` contract recorded in the transition catalog, including the Dashboard manage-without-read exception. Sprint 6A then appends the one approved fixed Core `Module Management` item to `Admin`. Characterization tests capture admin, operator, respondent, product-manage-only, and no-access actors before `NAV_ITEMS` is replaced; post-refactor assertions preserve every old item and add only the new item for actors with effective global `modules:read`.
- `Main` and `Admin` are the only v1 groups. Core normalization assigns each contribution one immutable reorder band that is not a source-descriptor field: Forms, Workflows, and Responses are between Organization and Operations; Components and Dashboards are after Operations; Datasets is between Administration and Module Management. Contributions cannot change group, change band, or cross a Core anchor in Sprint 6A. Permanent Home, Organization, Operations, Administration, and Module Management destinations are policy-immutable; they remain capability filtered where applicable. Only contribution visibility and order within its assigned band are mutable.
- Module Management uses stable Core item key `module_management`, label `Module Management`, route `/administration/modules`, group `Admin`, and the fixed default slot after Datasets. It is excluded from navigation-policy write members. The `Admin` group renders whenever any eligible Admin item is visible, so a global `modules:read` actor sees the group and Module Management even when the separately authorized Administration and Datasets items are absent.
- Navigation policy writes are atomic collection replacements with optimistic `expected_revision` concurrency. Within each band, the server derives a dense zero-based deterministic order and uses contribution ID as the final tie-breaker. It rejects partial, duplicate, unknown, group-changing, band-changing, cross-anchor, Core-mutating, or invalid per-band input without mutation and returns the same projection for an idempotent retry.
- If policy/catalog composition fails, the shell renders capability-filtered permanent Core destinations plus an explicit navigation-unavailable state. It never treats a missing policy as permission, never changes route/API authorization, and never silently persists fallback values. Direct authorized routes remain loadable.

### Application UI

- Add `/administration/modules` and `/administration/modules/:definition_id` as native SSR routes plus matching Axum document routes.
- Add a permanent Core Module Management shell item to the `Admin` group, after Datasets by default, with effective global `modules:read` display eligibility. Also add the same destination to the existing `admin:all`-only Administration landing page; that landing entry is supplementary and is not the read-only actor's discovery path.
- The directory identifies Core/runtime context and each transition contribution without presenting fake installed state.
- The detail experience provides peer sections for Overview, Feature Declarations, Contracts, Capabilities, Dependencies/Findings, Resources/Destinations, and Navigation.
- Transition detail uses the exact label `Transitional — not independently deployable` and explicitly states `No Module Release` and `No Module Instance`. It omits install, enable, health, and data-binding controls that do not apply.
- Feature Declaration UI shows use cases, inputs, outcomes, constraints, and the contracts/resources/routes/capabilities that realize the feature.
- Dependency, compatibility, configuration, readiness, and health findings remain separate rows/sections; do not collapse them into one colored status.
- Global `modules:read` receives a read-only navigation-policy presentation. Show/hide and stable-order controls are enabled only with effective global `modules:manage_navigation` and explain that display changes do not grant or revoke access.
- Role management shows Core or transition-contribution provenance and provider state for each capability while retaining Core-owned role mutation.
- Reuse existing Administration page headers, breadcrumbs, tables, info lists, tabs, accessible controls, loading/empty/error states, and responsive patterns. Do not introduce a workbench or bridge-owned surface.

## Acceptance Criteria

- The inventory exposes one stable Application Installation/Core observation and the seven roadmap transition areas.
- Every transition contribution has the exact stable identifiers, source bytes, expected SHA-256 digest, normalized projection, and findings frozen by the transition catalog; source/projection provenance matches after repeated restarts.
- No transition contribution has a Module Release, Module Instance, artifact, install action, enable action, or Supervisor-materialization claim.
- Sprint 6A creates no Module Release/Instance persistence or APIs; those names appear only as public contract types.
- Migration is inspectable as `retired` with `transition_destination_retired`; no fake `/migration` destination is created, and restoration requires a new product decision.
- `/operations` remains a Core-owned status projection and is not silently promoted into a module identity.
- For each of the seven transition descriptors, exact descriptor bytes, API
  projection, inert SSR bootstrap, and rendered detail remain one projection.
  The DOM proves exact reserved definition ID, source digest, display name,
  description, availability, and the complete ordered declaration field set:
  Feature Declaration `id`, `name`, `description`,
  `use_cases`, `inputs`, `outcomes`, `constraints`, `contracts`,
  `resource_types`, `destinations`, `capabilities`, and
  `configuration_pointers`; provided-contract `id`, `version`, `kind`, and
  `description`; dependency `contract_id`, `version_requirement`, `binding_key`,
  and `optional` rendered as Required/Optional; resource-type `id` and
  `description`; route `name`, `kind`, optional `resolved_path`, and each ordered
  parameter's `name`, `value_type`, and `required`; navigation `id`,
  `destination`, `label`, `group`, `order_hint`, and ordered
  `required_capabilities_any_of`; security-capability `id` and `description`; and
  each typed finding's `code`, `path`, and `message` in stable order. The
  API/bootstrap retain the complete configuration schema, while the visible
  overview exactly reports its corresponding `Declared` or `Not declared` state.
- Valid Module Manifest and all seven transition-descriptor fixtures round-trip semantically; exact source bytes and expected digests remain immutable.
- Invalid namespaces, duplicate identifiers, unresolved feature links, invalid contracts/dependencies, mutable/missing digests, deployment URLs, and unsupported deployment profiles fail with the exact semantic-finding or structural-decode proof defined by the Test Evidence section.
- Definition, Release, Instance identity, instance operation, data retention, navigation policy, authorization, dependency, compatibility, configuration, readiness, and health dimensions are not collapsed.
- Catalog synchronization is transactional and idempotent under fresh, repeated, concurrent, changed-source, invalid-source, and injected-partial-failure scenarios and preserves capability IDs, role mappings, navigation policy, and audit identity.
- Capability catalog and role management expose Core/transition provenance without allowing a contribution to mutate a role.
- Existing Organization create/edit routes obtain metadata schema only through
  `GET /api/node-types/{node_type_id}/metadata-fields`: an authorized scoped
  manager receives a top-level ordered list whose entries contain exactly `id`,
  `node_type_id`, `node_type_name`, `key`, `label`, `field_type`, and `required`,
  and can render the seeded node name, current metadata value, and a required
  create control without a console error. The response exposes no node values,
  scoped Forms, relationship administration, or mutation authority; out-of-scope
  node behavior remains non-disclosing, and the same actor remains forbidden from
  the full admin node-type definition and every metadata-schema mutation.
- Module inventory/policy endpoints enforce installation-global `modules:read` and `modules:manage_navigation`; manage implies read, `admin:all` implies both, and scoped-only/no-access/anonymous actors have negative coverage. Role mutation rejects mixed scope-mode bundles unless they contain the sole `admin:all` sentinel exception, classifies that exception as installation-global, rejects scoped assignments for roles containing either global module capability, and rejects adding one to a currently scoped role. Per-user shell navigation remains available to any authenticated actor; the shell endpoint is not gated by module-administration capability, but its fixed Module Management item is filtered on effective global `modules:read`.
- Module Management is a permanent Core item in `Admin`, excluded from administrator policy mutation. A global `modules:read`-only actor sees that item and the `Admin` group, can load directory/detail/descriptor and view current policy, cannot see an enabled mutation affordance, and receives `403 modules_manage_navigation_global_required` for a direct policy write. A `modules:manage_navigation`-only stored assignment sees the same item/read surfaces plus enabled controls because manage implies read. `admin:all` implies both. Scoped-only module authority, product-only authority, no access, and anonymous actors do not receive the item.
- An administrator can hide/show and reorder contributed navigation within each contribution's immutable Core-assigned band; settings persist across reload and are shared by desktop/mobile shells.
- Before policy mutation, every pre-existing item retains its exact pre-sprint label, group, relative ordering, and visibility for each actor. The only additive shell difference is fixed Module Management after Datasets for effective global `modules:read`. Core destinations remain policy-immutable, contribution groups and bands cannot change, and no contribution can cross Home, Organization, Operations, Administration, or Module Management.
- Hiding Forms removes its shell item without changing Forms authorization; an authorized direct `/forms` load still works.
- A visible contribution does not appear for or authorize an actor lacking its required product capability.
- Semantic destinations resolve through the current same-origin Core registry without stored deployment URLs.
- Representative transition references are installation-bound and Core-owned; owner/type cannot be reinterpreted as a future Module Instance.
- `ResourceResolutionV1` proves access, owner/data, resource identity, provider lifecycle, compatibility, and availability vary independently; unauthorized/not-evaluated known and random identifiers have identical restricted status/body shape and pass the defined timing profile.
- A populated migration-2 Sprint 5A database first proves its exact 20-capability/admin-20/operator-10/respondent-2 historical seed precondition, then upgrades to migration 3 without changing captured product row counts, role rows/IDs, user-managed role mappings/assignments, accounts, sessions, route behavior, or any pre-existing navigation item. Versioned built-in membership alone is replaced by exact current contract `sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68`; the built-in `admin` role contains only `admin:all`, whose existing implication preserves its effective product access without a mixed-scope bundle. Injected stale/missing membership is repaired as one transaction, two restarts and concurrent startup converge to the same set, and a separate fresh-data build produces it exactly. The before/after navigation comparison expects only the approved fixed Module Management addition for effective global `modules:read`.
- The retained compatibility rollback package starts successfully against the representative upgraded fixture clone and passes frozen product smoke coverage. Its evidence proves the explicit built-in seed transition from the current Sprint 6A contract to exact historical `sprint-5a-role-capabilities-v1+sha256.7725e889996a` membership while the separately fingerprinted module control plane and user-managed mappings remain byte-for-byte invariant. The unmodified historical Sprint 5A package is never claimed compatible with migration 3 and is validated only after a pre-upgrade backup restore, where the evidence requires an exact Sprint 5A-to-Sprint 5A built-in mapping result. The restored demo target is then upgraded separately by closing Sprint 6A startup with `-SkipSeed` for Gate 4.
- Successful navigation changes and denied changes are attributable through stable audit events; idempotent/no-op writes and unchanged catalog synchronization do not duplicate events.
- Every route and representative behavior in the Sprint 6A regression matrix—including Home, login/session, Organization, Operations, all Administration routes, and all product route families—retains native document ownership and its characterized SSR/no-JavaScript behavior, hydration parity, clean console behavior, and no `/bridge/*` requests. Data-complete no-JavaScript behavior is not newly imposed on existing hydrate-dependent routes.
- Existing frozen-baseline tests pass without unjustified deletion, skip, weakening, retry/tolerance expansion, selector loosening, or fixture regeneration. Every test expectation change has the evidence required by the change-control section.
- The user-testable roadmap exit condition is covered by manual and automated evidence.

## Manual Test Plan

### Disposable Reader/Manager And Provenance Fixtures

Run this walkthrough only against a disposable manual-test deployment; do not
add reviewer accounts to the retained upgraded/fresh evidence databases after
their capture. Sign in as `admin@tessara.local`, read
`GET /api/admin/capabilities`, and create three uniquely named user-managed
roles through `POST /api/admin/roles`:

- `manual-sprint-6a-modules-reader` with only `modules:read`;
- `manual-sprint-6a-modules-manager` with only
  `modules:manage_navigation` (no separately stored `modules:read` row); and
- `manual-sprint-6a-provenance` with only `forms:read`.

Create reader and manager accounts through `POST /api/admin/users`, assigning
the corresponding role IDs without a scope node so both assignments are
installation-global. Use unique `manual-sprint-6a-*` email/display-name values.
The provenance role does not need an account: open `/administration/roles` as
the administrator and inspect that temporary user-managed role directly. Do
not edit the seed-owned `admin`, `operator`, or `respondent` roles for this
walkthrough.

After restoring the original navigation policy, remove only the prefixed
accounts and roles. Because the product has no delete-user/delete-role API yet,
cleanup uses `docker exec -i <database_runtime.container_id> psql -X -v
ON_ERROR_STOP=1 -U <database_runtime.database_user> -d
<database_runtime.current_database>` and one transaction that deletes the
prefixed accounts before the prefixed roles. The container, user, and database
values must come from the deployment record for the manual-test environment;
an implicit `docker compose exec postgres` target is not an acceptable
evidence-bound cleanup. Query both tables afterward and require zero remaining
`manual-sprint-6a-*` rows. A fresh disposable reset is acceptable only when the
entire manual-test database is intentionally disposable.

The supported API setup shape is:

```powershell
$baseUrl = 'http://127.0.0.1:8080'
$login = Invoke-RestMethod -Method Post -Uri "$baseUrl/api/auth/login" `
  -ContentType 'application/json' `
  -Body (@{ email = 'admin@tessara.local'; password = 'tessara-dev-admin' } | ConvertTo-Json)
$headers = @{ Authorization = "Bearer $($login.token)" }
$capabilities = Invoke-RestMethod -Headers $headers -Uri "$baseUrl/api/admin/capabilities"

function New-ManualRole([string]$name, [string[]]$keys) {
  $ids = @($capabilities | Where-Object key -In $keys | ForEach-Object id)
  if ($ids.Count -ne $keys.Count) { throw "Missing capability for $name" }
  Invoke-RestMethod -Method Post -Headers $headers -Uri "$baseUrl/api/admin/roles" `
    -ContentType 'application/json' `
    -Body (@{ name = $name; capability_ids = $ids } | ConvertTo-Json)
}

$readerRole = New-ManualRole 'manual-sprint-6a-modules-reader' @('modules:read')
$managerRole = New-ManualRole 'manual-sprint-6a-modules-manager' @('modules:manage_navigation')
$provenanceRole = New-ManualRole 'manual-sprint-6a-provenance' @('forms:read')

foreach ($actor in @(
  @{ identity = 'reader'; role_id = $readerRole.id },
  @{ identity = 'manager'; role_id = $managerRole.id }
)) {
  Invoke-RestMethod -Method Post -Headers $headers -Uri "$baseUrl/api/admin/users" `
    -ContentType 'application/json' `
    -Body (@{
      email = "manual-sprint-6a-$($actor.identity)@tessara.local"
      display_name = "manual-sprint-6a-$($actor.identity)"
      password = 'tessara-dev-manual'
      is_active = $true
      role_ids = @($actor.role_id)
    } | ConvertTo-Json)
}
```

Use the same environment's deployment record for exact cleanup binding:

```powershell
$deployment = Get-Content -Raw '<manual-deployment-evidence.json>' | ConvertFrom-Json
$database = $deployment.snapshot.database_runtime
$sql = @'
BEGIN;
DELETE FROM accounts
WHERE email LIKE 'manual-sprint-6a-%@tessara.local'
   OR display_name LIKE 'manual-sprint-6a-%';
DELETE FROM roles WHERE name LIKE 'manual-sprint-6a-%';
COMMIT;
SELECT (SELECT count(*) FROM accounts
        WHERE email LIKE 'manual-sprint-6a-%@tessara.local'
           OR display_name LIKE 'manual-sprint-6a-%')
     + (SELECT count(*) FROM roles WHERE name LIKE 'manual-sprint-6a-%');
'@
$remaining = $sql | docker exec -i $database.container_id psql -X -At `
  -v ON_ERROR_STOP=1 -U $database.database_user -d $database.current_database
if (($remaining | Select-Object -Last 1).Trim() -ne '0') {
  throw 'Manual Sprint 6A fixture cleanup did not remove every prefixed row.'
}
```

### Module Discovery

1. Sign in as `admin@tessara.local`.
2. Open `/administration/modules`.
3. Confirm Forms, Workflows, Responses, Datasets, Components, and Dashboards appear as active in-process transition contributions, while Migration appears separately as retired historical/support inventory.
4. Confirm no row claims a Module Release, Module Instance, install state, enablement, or health state.
5. Open Forms, Components, Dashboards, and Migration details.
6. Confirm features, use cases, inputs, outcomes, constraints, contracts, resources, routes, capabilities, and separate findings are readable.
7. Confirm Migration reports `Retired`, explains that the former surface was deliberately withdrawn, and exposes no current route, provider, navigation item, or action.
8. Sign in with global `modules:read` but without `admin:all`; confirm the `Admin` group and fixed Module Management item appear, the Administration item remains absent, directory/detail/descriptor and current navigation policy are readable, and no show/hide/reorder control is enabled.
9. Sign in with a stored global `modules:manage_navigation` assignment but no separate read row; confirm the item/read surfaces appear through implication and the navigation-policy controls are enabled.
10. Confirm scoped-only module authority, product-only authority, and no-access actors do not receive the Module Management item and cannot direct-load its routes or APIs.

### Navigation Versus Authorization

1. Open navigation policy from Module Management.
2. Hide Forms and move Dashboards before Components.
3. Reload and verify desktop and mobile shell order/visibility.
4. Attempt to move Dashboards before Operations, Forms after Operations, or Datasets after Module Management; confirm each cross-band update is rejected atomically and the Core anchors and all contribution policy values remain unchanged.
5. Attempt to submit `module_management` as a mutable policy member; confirm atomic `navigation_policy_core_item_immutable` rejection with no revision, policy, ordering, or successful-audit mutation.
6. Direct-load `/forms` as the same authorized administrator and verify the page/API still work.
7. Restore the original policy and confirm it persists.
8. Repeat with the named permission fixtures: `admin:all`; global `modules:read`; global `modules:manage_navigation` without a separately stored read row; a scoped-only module capability assignment; product capability without module administration; and no access.
9. Confirm manage implies read, scoped-only module authority is rejected, ordinary authenticated actors still receive their filtered shell model, and visibility never grants product access.
10. Force a stale policy revision and confirm the update fails atomically without changing order, visibility, authorization, or audit history except the attributable denied-attempt event.

### Role Capability Provenance

1. Open `/administration/roles`.
2. Open the temporary `manual-sprint-6a-provenance` user-managed role and confirm `forms:read` shows its Core/Forms transition provenance and current provider state.
3. Add and remove a second compatible scope-aware capability on that temporary role, then save the intended final bundle.
4. Confirm the descriptor and navigation policy did not change and the contribution did not mutate the role independently.
5. Remove the temporary role through the evidence-bound cleanup described above.

### Semantic Destinations And Typed References

1. Read the Application Installation ID from `GET /api/admin/modules`, then call `POST /api/platform/destinations/resolve` with schema v1, owner `core_installation`, route `forms.detail`, and a typed UUID `form_id`; confirm the result is exactly the existing same-origin `/forms/{form_id}` path.
2. Create a representative Form reference through `POST /api/platform/resource-references` using resource type `tessara.transition.form` and the checked integration fixture `forms(name = 'Platform reference fixture', slug = 'platform-reference-fixture')`; resolve it through `POST /api/platform/resource-references/resolve`.
3. Repeat resolution for the deterministic demo Response fixtures used by `response_reference_resolution_preserves_ownership_delegation_scope_and_non_disclosure`: the seeded respondent submission, an out-of-scope delegated submission, and a random UUID with resource type `tessara.transition.response`.
4. Confirm each transition reference is owned by that exact Application Installation and never by a fictional Module Instance.
5. Exercise known, random/unknown, owner/type/installation mismatch, unsupported schema/unknown-field, unauthorized, and scope/ownership `not_evaluated` cases; require the exact status/error or seven-dimension resolution envelopes documented below, including byte-identical restricted known/random bodies.

### SSR And Regression

1. Disable JavaScript and direct-load the module directory/detail routes; confirm useful safe HTML.
2. Re-enable JavaScript and confirm hydration has no mismatch or uncaught console error.
3. Execute every route family and representative behavior in the Sprint 6A regression matrix, including Home, login/session, Organization, Operations, and every Administration route; with JavaScript disabled, compare each existing route to its characterized baseline rather than requiring newly data-complete SSR.
4. Confirm desktop and mobile preserve every pre-sprint item and its relative order before policy mutation, add only fixed Module Management after Datasets for effective global `modules:read`, and share the same changed contribution policy afterward.
5. Exercise keyboard-only show/hide/reorder controls; confirm visible focus, focus restoration, accessible names, live success/error feedback, and no keyboard trap.
6. Confirm populated, loading, empty, no-results, restricted, unavailable, error, and not-found states remain distinct; verify no horizontal overflow at the established desktop and mobile viewports.
7. Confirm no touched route requests `/bridge/*` assets and no browser-console error occurs.

### Existing-Database Upgrade And Rollback

Steps 1–6 use the representative `SPRINT_6A_UPGRADE_DATABASE_URL` fixture and
`CompatibilityOnUpgraded`; they are not Gate 4 browser acceptance. Step 7 uses
the separate Sprint 5A demo source and restored target.

1. Back up a populated migration-2 Sprint 5A database and capture the agreed product row counts, existing capability/role mapping IDs, account/session records, and current route/navigation results.
2. Launch the Sprint 6A binary without `-FreshData`; confirm only migration 3 applies and catalog synchronization completes.
3. Restart twice and run synchronization concurrently; confirm the Application Installation identity, source digests, projection IDs, mappings, policy, and audit identity remain stable.
4. Compare the frozen product snapshots and run the regression matrix before any navigation policy mutation.
5. Exercise an injected invalid/partial synchronization in a disposable database and confirm the transaction leaves no partial source, projection, capability, policy, or audit state.
6. Demonstrate application rollback by starting the prebuilt Sprint 5A compatibility rollback package—which includes the immutable migration set through version 3—against the representative populated-upgrade fixture clone used by `CompatibilityOnUpgraded`, without dropping tables. Separately prove that the unmodified historical Sprint 5A package is used only after restoring the pre-upgrade database backup.
7. Use a separate Sprint 5A source that already contains the acceptance actors and demo assets, restore it into the disposable target, pass `OriginalAfterRestore`, and then start the clean closing Sprint 6A image with `-SkipSeed`. Confirm migration 3 applies, the deployment classifies as upgraded from pre-migration product rows, and smoke/UAT use the preserved demo assets rather than creating post-upgrade replacements.

## Automated Test Plan

### Baseline And Environment Rules

- The pre-refactor baseline is run from a clean worktree at kickoff commit `3625d4de52c5856e4ac3bc642a9422a029e9f375`. Record the commit, environment, exact command, pass count, and ignored/skipped/filtered count. Results from the modified Sprint 6A branch cannot be retroactively labeled the kickoff baseline.
- A kickoff characterization failure is recorded as a red pre-existing defect, not relabeled as supported behavior. In particular, the Leptos Dataset preview and revision-edit routes currently lack matching Axum document-route registrations; add those native direct-load mappings and make their tests green before the shell/navigation refactor. Do not remove the routes from the matrix or weaken SSR assertions to manufacture a baseline pass.
- Install locked browser dependencies with `npm --prefix .\end2end ci` and the repository's browser helper with `npm --prefix .\end2end run install-browsers` before browser validation. Bare root-level `npx playwright test` is not a supported repository command.
- Full Rust validation uses three independently provisioned named disposable PostgreSQL databases through `TEST_DATABASE_URL`, `SPRINT_6A_UPGRADE_DATABASE_URL`, and `SPRINT_6A_FRESH_DATABASE_URL`. The upgrade and fresh databases are reset by their separate proofs; all three live `current_database()` values must be pairwise distinct, token-bounded test-only names, and the exact destructive-reset acknowledgement is mandatory. A suite that reports a missing-database skip is failed evidence. Gate 3 additionally requires a separate Sprint 5A demo source and disposable restore target; the latter becomes Gate 4's upgraded browser candidate only after `OriginalAfterRestore` passes and closing startup applies migration 3 with `-SkipSeed`.
- Every deployment/browser command validates the same [retained schema-v1 deployment-evidence record](./sprint-6a-deployment-evidence.md) against its current live BaseUrl, API container image, and database. The capture derives the clean closing commit/tree from Git and matching immutable release-image labels; the exact PostgreSQL `container_id` + `database_user` + `current_database` triple used by the live API; successful migration ledger exactly 1–3 and current file checksums; the versioned built-in seed mapping/digest; single Application Installation; exact seven-entry transition catalog; absence of Release/Instance tables; and upgraded-versus-fresh state. Playwright fixture cleanup receives that exact validated triple and cannot silently select a different Compose database. `upgraded` requires a product row that predates migration 3, while `fresh` requires none, so the two records cannot be interchanged. A checkbox, filename, or operator assertion is not deployment proof.
- Because the repository currently has no enforced CI workflow, the closeout report is the retained gate: it records exact commands, counts, duration, zero unexpected skips/filters, and links each failure/fix to the closing commit.
- Deployment evidence binds the exact committed tree. Sprint closeout therefore uses two clean proof passes when tracked status/result documentation cannot be truthful before the first pass: prove an implementation commit, record the observed results and handoff in a final documentation commit, then rebuild and rerun every commit-bound Gate 3–6 command against that final commit. Only artifacts from the final commit qualify as closeout evidence; a later documentation-only commit without recapture invalidates the earlier package/deployment proof.

### Ordered Required Gates

Run sequentially to avoid Windows Cargo artifact-lock contention and to ensure browser tests exercise the closing build.

| Gate | Prerequisite | Commands | Required retained evidence |
| --- | --- | --- | --- |
| 1. Non-mutating source/contract | Locked dependency files present | `cargo fmt --all -- --check`; `cargo check --workspace --all-features --locked`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test -p tessara-module-contract --locked`; `.\scripts\check-web-crate-boundaries.ps1`; `.\scripts\local-launch.ps1 -SelfTest`; `.\scripts\capture-sprint-6a-deployment-evidence.ps1 -SelfTest`; `.\scripts\validate-e2e.ps1 -SelfTest`; `.\scripts\validate-resource-reference-nondisclosure.ps1 -SelfTest`; `.\scripts\test-sprint-6a-rollback-package.ps1 -SelfTest`; `.\scripts\test-sprint-6a-acceptance-evidence.ps1`; `cargo audit --quiet`; `git diff --check` | All pass; contract unit/fixture/doc counts; deployment capture/publication, exact environment restoration, Playwright retained-artifact rollback, actual TypeScript upgraded-seed guard plus exact endpoint inventory, final live-deployment revalidation, rollback client/restore/log safety, nondisclosure exact-type/UUID/UTC/live-ID/digest/sidecar/path-alias/bounded-diagnostic/overwrite/finalization-rollback/cleanup, and smoke/UAT exact-schema/session-cleanup/path-alias/concurrent-publication self-tests; advisory result and any already-approved exception/reachability note |
| 2. Disposable-database integration | Three independent Postgres databases ready; `$env:TEST_DATABASE_URL`, `$env:SPRINT_6A_UPGRADE_DATABASE_URL`, and `$env:SPRINT_6A_FRESH_DATABASE_URL` set to pairwise-distinct token-bounded disposable names; destructive acknowledgement set exactly | `$env:SPRINT_6A_CONFIRM_DESTRUCTIVE_UPGRADE_RESET='I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET'`; `.\scripts\validate.ps1` (its non-Fast path must invoke `cargo test -p tessara-api --test modules --release --locked resource_reference_restricted_known_random_latency_profile -- --exact --nocapture`); `cargo test --workspace --all-features --locked` | API/web/contract/workspace counts and zero database skips; native, SSR, and wasm hydrate checks pass; the optimized release-only known/random timing proof executes rather than returning a debug pseudo-pass; populated upgrade proof leaves the migration-3 representative fixture intact for `CompatibilityOnUpgraded`, while fresh/lock-order proof resets only its independently named database; neither is substituted for Gate 4's restored demo target |
| 3. Populated migration-2 upgrade and rollback compatibility | Independently named `SPRINT_6A_UPGRADE_DATABASE_URL` for the representative populated-upgrade fixture; separate backed-up Sprint 5A demo source and disposable restore target whose `current_database()` names contain token-bounded disposable markers; captured invariant snapshot; closing migration and complete closing tree committed/clean; Node/npm and `cargo-leptos` available; either unambiguous local `psql`/`pg_dump`/`pg_restore` executables or one explicitly identified running PostgreSQL client container available | `cargo test -p tessara-api --test sprint_6a_populated_upgrade --locked`; `$closing = git rev-parse HEAD`; `$postgresClientContainer = (docker inspect --type container --format '{{.Id}}' '<running-postgres-container-name-or-id>').Trim()`; `.\scripts\build-sprint-6a-compatibility-rollback.ps1 -ClosingSprint6ACommit $closing`; `.\scripts\test-sprint-6a-rollback-package.ps1 -SelfTest`; `.\scripts\test-sprint-6a-rollback-package.ps1 -Mode PackageOnly -ExpectedClosingSprint6ACommit $closing -EvidencePath 'artifacts/sprint-6a/rollback-package-only.json'`; `.\scripts\test-sprint-6a-rollback-package.ps1 -Mode CompatibilityOnUpgraded -ExpectedClosingSprint6ACommit $closing -DatabaseUrl '<representative-populated-upgrade-fixture-url>' -ExpectedDatabaseName '<representative-populated-upgrade-fixture-name>' -PostgresClientContainerId $postgresClientContainer -EvidencePath 'artifacts/sprint-6a/rollback-compatibility-upgraded.json'`; `$restoreEvidence = 'artifacts/sprint-6a/rollback-restore-evidence.json'`; `.\scripts\capture-sprint-6a-rollback-restore-evidence.ps1 -SourceDatabaseUrl '<sprint-5a-demo-source-url>' -ExpectedSourceDatabaseName '<sprint-5a-demo-source-name>' -MaintenanceDatabaseUrl '<same-cluster-maintenance-url>' -TargetDatabaseUrl '<restored-sprint-5a-demo-target-url>' -ExpectedTargetDatabaseName '<restored-sprint-5a-demo-target-name>' -BackupPath 'artifacts/sprint-6a/pre-upgrade-backup.dump' -EvidencePath $restoreEvidence -PostgresClientContainerId $postgresClientContainer`; `.\scripts\test-sprint-6a-rollback-package.ps1 -Mode OriginalAfterRestore -ExpectedClosingSprint6ACommit $closing -DatabaseUrl '<restored-sprint-5a-demo-target-url>' -ExpectedDatabaseName '<restored-sprint-5a-demo-target-name>' -RestoreEvidencePath $restoreEvidence -PostgresClientContainerId $postgresClientContainer -EvidencePath 'artifacts/sprint-6a/rollback-original-restored.json'`; finally set `$upgradeDatabaseContainer = '<exact-running-restored-demo-database-container-id>'`, construct `$upgradeAcceptanceDatabaseUrl = '<postgres-url-using-host.docker.internal-and-that-container-published-port/restored-sprint-5a-demo-target-name>'`, and run `.\scripts\local-launch.ps1 -ExternalDatabaseUrl $upgradeAcceptanceDatabaseUrl -ExternalDatabaseContainerId $upgradeDatabaseContainer -SkipSeed` so the closing startup applies migration 3 to the restored demo target | Missing/empty/unsafe/shared database configuration fails before reset; rollback package creation refuses an uncommitted/dirty tree or a closing commit other than exact `HEAD`, and every validator invocation binds the manifest to that explicitly expected commit; exact Sprint 5A catalog and admin-20/operator-10/respondent-2 precondition frozen as `sprint-5a-role-capabilities-v1+sha256.7725e889996a` / `7725e889996a73a5655c57106aca6e12d9a5f95e9103f14d7b0fd50fbac96988`; only migration 3 applied; exact before/after product identities, role rows/IDs, user-managed mappings/assignments, account/session, and linked rows; built-in membership equals version `sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68` and full digest `2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600` after upgrade, repair, restart, concurrency, and fresh startup; source/digest/projection/policy/audit state stable; manifest records exact source/binary/site/migration/package digests plus builder/build command; PackageOnly uses no database; rollback JSON retains exact built-in mapping snapshots and deterministic digests before/after; the representative fixture clone is used only for detailed `CompatibilityOnUpgraded` proof and is not substituted for browser acceptance; original migrations reject ledger 3; restored-original validation proves exact Sprint 5A-to-Sprint 5A membership and binds an all-table source/target fingerprint to independently verified backup/restore evidence; the restored target already contains the Gate 4 demo actors and assets before migration 3, and the closing startup uses `-SkipSeed`; deployment classification plus smoke/UAT evidence must prove those pre-migration product rows and exact acceptance assets survived the upgrade; external deployment refuses fresh/reset/seed modes, unsafe database names, or a URL whose published port/current database does not match the supplied running database container |
| 4. Closing-build regression on upgraded data | Gate 3 complete; the clean closing source is built into a running release image whose API is connected to the restored Sprint 5A demo target after closing startup applied migration 3 with `-SkipSeed`; `$upgradeDatabaseContainer` still contains that target's exact running container ID; the representative populated-upgrade fixture remains separate and is not a Gate 4 candidate | `$upgradedEvidence = 'artifacts/sprint-6a/deployment-upgraded.json'`; `.\scripts\capture-sprint-6a-deployment-evidence.ps1 -BaseUrl "http://127.0.0.1:8080" -ExpectedDataState upgraded -OutputPath $upgradedEvidence -DatabaseContainerId $upgradeDatabaseContainer` (also supply `-ApiContainerId` when the release API is outside the default Compose project); `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8080" -KeepServices -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -AcceptanceEvidencePath 'artifacts/sprint-6a/smoke-upgraded.json'`; `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -AcceptanceEvidencePath 'artifacts/sprint-6a/uat-upgraded.json'`; `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -EvidencePath 'artifacts/sprint-6a/playwright-acceptance-upgraded.json'`; `.\scripts\validate-resource-reference-nondisclosure.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath $upgradedEvidence -ExpectedDataState upgraded -OutputPath 'artifacts/sprint-6a/resource-reference-nondisclosure-upgraded.json'` | Retained deployment JSON plus SHA-256 sidecar; every gate re-derives and exactly matches the live commit/tree, container/image, BaseUrl, database/installation, migration checksums, seed digest, catalog, and upgraded classification before testing; the upgraded record proves at least one acceptance product row predates migration 3; when `ExpectedDataState=upgraded`, smoke, UAT, and Playwright never call `/api/demo/seed` and instead resolve and prove the already-restored assets; any Gate 4 demo mutation disqualifies the run; state-specific structured smoke/UAT JSON plus validated sidecars prove exact check sets and current-run session cleanup; exact schema-v2 60-test/7-file Playwright inventory freezes every full file/describe/test identity, retained as a distinct upgraded artifact set only after all reports validate; console/hydration/route ownership evidence; retained nondisclosure JSON plus validated SHA-256 sidecar proving exact restricted status/body-byte parity and passing median/p95 deltas for both `unauthorized` and scoped `not_evaluated`; existing evidence is replaced only with the artifact-specific explicit overwrite switch and otherwise survives failed validation/publication unchanged |
| 5. Fresh installation | Gate 4 complete; destructive reset explicitly acknowledged; same clean closing commit | `.\scripts\local-launch.ps1 -FreshData`; `$freshEvidence = 'artifacts/sprint-6a/deployment-fresh.json'`; `.\scripts\capture-sprint-6a-deployment-evidence.ps1 -BaseUrl "http://127.0.0.1:8080" -ExpectedDataState fresh -OutputPath $freshEvidence`; `.\scripts\smoke.ps1 -UseExistingService -BaseUrl "http://127.0.0.1:8080" -KeepServices -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -AcceptanceEvidencePath 'artifacts/sprint-6a/smoke-fresh.json'`; `.\scripts\uat-sprint.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -AcceptanceEvidencePath 'artifacts/sprint-6a/uat-fresh.json'`; `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -EvidencePath 'artifacts/sprint-6a/playwright-acceptance-fresh.json'`; `.\scripts\validate-resource-reference-nondisclosure.ps1 -BaseUrl "http://127.0.0.1:8080" -DeploymentEvidencePath $freshEvidence -ExpectedDataState fresh -OutputPath 'artifacts/sprint-6a/resource-reference-nondisclosure-fresh.json'` | A separate retained fresh deployment JSON and sidecar that cannot validate as upgraded; state-specific smoke/UAT JSON/sidecars; production CSS/WASM/release image labels match the same clean closing commit/tree; database has successful migrations exactly 1–3 with repository checksums, deterministic seed/catalog, no pre-migration-3 product rows, and the same complete browser/UAT/non-disclosure results retained under state-specific paths beneath `artifacts/sprint-6a/`; fresh nondisclosure proof is a distinct JSON/SHA-256 pair with overwrite refusal and rollback-safe sequential publication (not two-file reader atomicity) |
| 6. Final cleanliness | All fixes included in the closing commit | Rerun Gate 1 and `git status --short` | No formatting/whitespace/dependency-audit regression, and `git status --short` emits no lines; a merely intentional dirty file is still a failed Gate 6 |

Gate 3's “generating structured restore evidence” step is this exact ordered
operation; it is not satisfied by an arbitrary evidence identifier or a prose
statement that a restore occurred:

```powershell
$env:SPRINT_6A_CONFIRM_DESTRUCTIVE_RESTORE_RESET = 'I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET'
$restoreEvidence = 'artifacts/sprint-6a/rollback-restore-evidence.json'
$closing = (git rev-parse HEAD).Trim()
$postgresClientContainer = (docker inspect --type container --format '{{.Id}}' '<running-postgres-container-name-or-id>').Trim()
.\scripts\capture-sprint-6a-rollback-restore-evidence.ps1 `
  -SourceDatabaseUrl '<writable-sprint-5a-demo-source-url>' `
  -ExpectedSourceDatabaseName '<sprint-5a-demo-source-name>' `
  -MaintenanceDatabaseUrl '<same-cluster-postgres-maintenance-url>' `
  -TargetDatabaseUrl '<writable-restored-sprint-5a-demo-target-url>' `
  -ExpectedTargetDatabaseName '<restored-sprint-5a-demo-target-name>' `
  -BackupPath 'artifacts/sprint-6a/pre-upgrade-backup.dump' `
  -EvidencePath $restoreEvidence `
  -PostgresClientContainerId $postgresClientContainer
.\scripts\test-sprint-6a-rollback-package.ps1 `
  -Mode OriginalAfterRestore `
  -ExpectedClosingSprint6ACommit $closing `
  -DatabaseUrl '<writable-restored-sprint-5a-demo-target-url>' `
  -ExpectedDatabaseName '<restored-sprint-5a-demo-target-name>' `
  -RestoreEvidencePath $restoreEvidence `
  -PostgresClientContainerId $postgresClientContainer `
  -EvidencePath 'artifacts/sprint-6a/rollback-original-restored.json'
```

Retain the dump, evidence JSON, and validator output. The evidence binds the
actual PGDMP bytes/digest, capture-helper digest, exact source/target database
names, ledgers `1,2`, and equal full logical fingerprints; the validator
independently recomputes the restored target ledger and fingerprint before
starting the original package. Container mode resolves and records the exact
running container/image identity, requires each supplied URL to match its
published `5432/tcp` host binding and configured credentials, invokes the
clients with password-free `127.0.0.1:5432` URLs, streams inbound archives as
the container execution user, and uses unique, failure-cleaned container paths.
When local PostgreSQL clients are installed, omitting
`-PostgresClientContainerId` selects the local-executable mode only for the
declared canonical URL subset: absolute `postgres`/`postgresql`, one host,
optional port, explicit non-empty user/password, one database path, and no query
or fragment. Percent encoding is decoded into exact child `PG*` environment
values; passwordless and query-bearing libpq URIs fail before client startup.
Schema-v3 evidence records exact local executable paths/digests or immutable
container/image/binding identity, binds both capture and common-helper digests,
and embeds only stopped-process logs after decoded passwords and normalized
password assignments are redacted.

The Sprint 5A source must contain the exact Gate 4 actors and demo assets before
this capture. After `OriginalAfterRestore`, launch the clean closing image
against the restored target with `-SkipSeed`; startup applies migration 3 and
the target, not the representative populated-upgrade fixture, becomes the
browser candidate. If the retained source is lost, recreate a new migration-2
source only with the validated rollback package's historical binary,
`original-migrations`, and `seed-demo` recovery procedure in
[the development workflow](../development-workflow.md#canonical-closeout-validation).

### Acceptance-To-Proof Matrix

| Contract/acceptance area | Durable automated proof required | Primary artifact(s) |
| --- | --- | --- |
| V1 identity, strict wire shapes, exact source/digest, stable findings | Unit tests, one canonical Manifest fixture, seven canonical transition fixtures, invalid fixture per rejection class, exact digest assertions | `tessara-module-contract` unit/fixture tests and checked-in catalog fixtures |
| Transition cannot masquerade as deployable/provider inventory | Serialization and behavior tests prove no Release/Instance/artifact/materialization fields or provider eligibility; compile/dependency boundary remains framework neutral | `tessara-module-contract`; boundary audit |
| Multidimensional typed resolution/non-disclosure | Independent-dimension unit tests, restricted-deserialization tests, API known/random shape proof, and warmed release timing conformance with at least 200 samples per identifier/access state | contract tests; focused modules API tests; `scripts/validate-resource-reference-nondisclosure.ps1` retained JSON report |
| Migration, installation identity, catalog synchronization, provenance | Fresh/upgrade/restart/concurrency/no-op/changed/invalid/injected-failure DB tests and before/after snapshots | focused modules persistence integration tests; migration integrity tests |
| Inventory/detail/descriptor APIs and human-machine parity | Global-capability positive/negative API tests; stable 401/403/404; source `ETag`; exact seven-descriptor API/bootstrap/DOM parity for reserved definition ID, source digest, display name, description, and availability; every ordered feature's `id`, `name`, `description`, `use_cases`, `inputs`, `outcomes`, `constraints`, `contracts`, `resource_types`, `destinations`, `capabilities`, and `configuration_pointers`; every contract's `id`, `version`, `kind`, and `description`; every dependency's `contract_id`, `version_requirement`, `binding_key`, and `optional`; every resource's `id` and `description`; every route's `name`, `kind`, optional `resolved_path`, and ordered parameter `name`, `value_type`, and `required`; every navigation declaration's `id`, `destination`, `label`, `group`, `order_hint`, and ordered `required_capabilities_any_of`; every capability's `id` and `description`; and every finding's `code`, `path`, and `message` in stable order; exact API/bootstrap configuration schema plus exact DOM `Declared`/`Not declared` state | focused modules API/SSR tests; `end2end/tests/modules.spec.ts` |
| Organization metadata-field schema compatibility | `GET /api/node-types/{node_type_id}/metadata-fields` exact ordered seven-key rows; hierarchy read/manage/admin positive authority; anonymous/no-access/unknown nondisclosure; continued denial of full admin node-type definitions; real create/edit metadata controls with no authorized console error | `hierarchy_metadata_fields` integration test; Organization route cases in `end2end/tests/permissions.spec.ts`; regression matrix |
| Dependency/finding, semantic destination, typed references | Exact transition-internal findings; exact `POST /api/platform/destinations/resolve`, `POST /api/platform/resource-references`, and `POST /api/platform/resource-references/resolve` wires; Form fixture `Platform reference fixture` / `platform-reference-fixture`; deterministic seeded own/delegated Response fixtures; known/random parity; route/parameter/owner/type/installation mismatch; same-origin-only paths; no URL persistence | contract tests; `platform_http_apis_enforce_strict_wires_authority_order_and_non_disclosure`; `response_reference_resolution_preserves_ownership_delegation_scope_and_non_disclosure`; release nondisclosure artifact |
| Capability provenance and Core role ownership | Existing capability IDs/descriptions and user-managed mappings preserved; versioned built-in seed mappings converge only to the declared current seed contract; descriptor mismatch rejection; new global scope metadata and scoped-role mutation rejection; contribution cannot mutate roles | DB/API tests; `end2end/tests/permissions.spec.ts`; permission scenario document |
| Navigation policy versus authorization | Pre-refactor default characterization; sole additive fixed Module Management item in `Admin`; global-read visibility/read-only UI; manage-implies-read controls; scoped/no-access omission; Core-item mutation rejection; atomic revisioned policy; hidden-authorized direct load; visible-unauthorized denial; failure fallback; desktop/mobile parity | web unit/SSR tests; modules and permissions Playwright specs |
| Existing product/application behavior | Every route and representative workflow in the regression matrix, including native document ownership and characterized SSR/no-JS state, hydration, console, accessibility, responsive overflow, and no bridge requests | Existing API/web suites plus only the explicitly approved and reconciled changes in `sprint-6a-test-change-log.md`; `app.spec.ts`; feature specs; permissions/modules specs; smoke/UAT |
| Audit and idempotency | Successful/denied policy event content, changed catalog event, and absence of duplicate events for retry/no-op/restart | modules persistence/API tests |

### Targeted Scenario Detail

- Contract validation covers invalid namespaces/case, duplicate identifiers, unresolved feature/configuration links, authority violations, version/profile incompatibility, duplicate/malformed routes/resources/capabilities/bindings, missing/mutable digests, deployment URLs, unsupported/unknown fields, and stable multi-finding order.
- The six active descriptor fixtures prove their exact feature/contract/dependency/resource/route/navigation/capability declarations from the catalog. The seventh, Migration, proves `retired` plus empty feature, contract, dependency, resource, route, navigation, capability, and configuration declarations. The later catalog-projection test separately proves the exact `transition_destination_retired` finding; that projected finding is not fabricated as a descriptor-source field.
- Catalog tests prove first/repeated/concurrent startup; stable Application Installation identity; source/digest/projection linkage; no-op preservation; compatible source change; malformed source and injected mid-sync rollback; and preservation of capability IDs, role mappings, policy, audit, sessions, and product data.
- Permission fixtures cover anonymous, no access, product-only, global read, global manage-without-separate-read, directly injected scoped-only module capability (authorization must fail closed), and `admin:all`. They assert the Module Management item, `Admin` group, routes, policy-read presentation, enabled mutation controls, and direct `PUT` outcome independently for each actor; the read-only actor must receive no enabled mutation affordance and a direct write must fail with `modules_manage_navigation_global_required`. Role API tests separately prove ordinary mixed scope-mode bundles, scoped assignment, and adding a global capability to an already scoped role are rejected atomically; a bundle containing `admin:all` is the sole mixed exception and remains installation-global; and separate global-module plus scoped-product roles compose without widening product scope. Cleanup is failure-safe and restores the initial navigation policy in `finally`/fixture teardown. Acceptance fixture cleanup uses the deployment-evidence container/user/database triple through the shared `runPlaywrightSql` helper; only non-acceptance development mode may fall back to the current Compose PostgreSQL service.
- Native-route permission proof treats expected denials as an exact scoped exception, not a general console allowlist: every expected `403` is bound to an exact `GET` path/query and count, every other `>=400` response or error-console entry fails, and the characterized route-specific readiness text must appear after both direct load and refresh. Component detail/viewer readiness uses the component display name while retaining the stable slug URL.
- Policy tests cover missing/duplicate/unknown IDs; attempted group, immutable-band, cross-anchor, and Core mutation; exclusion of Module Management from mutable collection members; atomic `navigation_policy_core_item_immutable` rejection for an attempted Module Management mutation; dense deterministic ordering/ties within each band; atomic rejection with `navigation_policy_band_change_forbidden`; idempotent retry; stale revision conflict; audit content; preservation of all pre-existing default items plus the authorized additive item; hidden-but-authorized direct load; visible-but-unauthorized denial; Dashboard manage-only behavior; retired Migration; and permanent Operations behavior.
- SSR/browser tests cover populated directory/detail, exact source/digest, unknown detail, insufficient global authority, read-only versus mutable policy presentation, Module Management Admin-group visibility on desktop and mobile, Administration-item independence, no JavaScript, shared bootstrap/hydration parity, zero duplicate initial load, zero console/hydration errors, keyboard/focus/live-region behavior, responsive overflow, and zero `/bridge/*` requests.
- The permissions inventory is updated in the same change as executable module-administration scenarios. Test documentation never claims planned coverage is implemented before the executable scenario passes.

## Ordered Implementation Plan

1. Record the clean-base validation results and add pre-refactor characterization coverage for the regression matrix, exact navigation model/role variants, route authorization, existing capability IDs/mappings, and populated migration-2 data. Do not restructure navigation or persistence until this proof passes.
2. Freeze the transition catalog, exact fixture bytes/digests, identifier grammar, discriminators, stable findings, Release/Instance types-only boundary, and multidimensional resolution envelope. Add the complete valid/invalid fixture and contract unit suite in the same change.
3. Add `003_module_control_plane.sql`, migration-integrity/upgrade tests, and reproducible Sprint 5A-code compatibility rollback packaging before adding services. Prove migration-2 upgrade, rollback-package startup on the representative upgraded fixture clone, original-package startup only after backup restore, restored-demo-target upgrade with `-SkipSeed` for Gate 4, fresh apply, failure transactionality, and byte-identical migrations 1/2.
4. Add the focused Core module catalog/repository/service boundary plus transactional synchronization. Land fresh/repeat/concurrent/no-op/change/invalid/injected-failure persistence tests with the implementation.
5. Add global module capability validation and inventory/detail/descriptor APIs. Land the endpoint authorization/error/ETag/human-machine-parity tests before exposing the UI.
6. Add semantic route registry, typed reference adapters, destination resolver, and `ResourceResolutionV1` APIs. Land owner/type/parameter/same-origin, independent-dimension, authorization-order, restricted-shape, and timing tests with the implementation.
7. Refactor route authorization away from static navigation only after the pre-refactor characterization suite is green. Preserve direct-route semantics and the Dashboard manage-only exception, then prove the same suite against the new authorization source.
8. Add revisioned atomic navigation-policy persistence/API and desktop/mobile resolved shell model, including the fixed `Admin`/`modules:read` Module Management item. Land preservation of every pre-existing item, additive-item eligibility/order, read/manage/scoped/no-access permission matrix, concurrency, idempotency, failure fallback, hidden-authorized, visible-unauthorized, immutable-Core, fixed-group, fixed-band/cross-anchor rejection, audit, and teardown-restoration tests before enabling mutation controls.
9. Add native SSR Module Management directory/detail/navigation UI, its standalone `Admin` shell item, and Administration landing integration with shared authorized bootstrap. Land read-only versus mutable-policy behavior, SSR/no-JavaScript, hydration, accessibility, responsive, loading/empty/error/restricted/unavailable, console, and no-bridge coverage in the same change.
10. Add capability provenance to role management without moving role ownership. Land API/UI provenance and role-mutation isolation tests.
11. Update `scripts/smoke.ps1`, `scripts/uat-sprint.ps1`, `end2end/tests/modules.spec.ts`, `end2end/tests/permissions.spec.ts`, `docs/playwright-permissions-scenarios.md`, API/wire documentation, transition/regression matrices, sprint index, roadmap status, and progress report as each corresponding executable behavior lands. Do not defer all documentation/tests to closeout.
12. Run the ordered upgraded-data and fresh-data gates, fix production defects without weakening frozen tests, complete the acceptance-to-proof table with actual evidence, and record reviewer-ready closeout results against the final commit.

Ordered implementation steps 1–12 are complete. Commit
`6580b040236f563c30b5162fa833d7b0fed16478` is the reviewed implementation,
canonical-fixture, and exact browser-inventory acceptance boundary. Subsequent
commits through evidence-harness cut
`9ba79752a4ee15ea224fa1da2932b26b279b9847` strengthened retained rollback and
deployment proof without changing production code, accepted fixture bytes, or
browser scenario identities. Canonical commit-bound Gates 3–6 are regenerated
from the clean commit containing the final closeout documentation; their exact
commit/tree/image/database bindings and state-specific results are retained
under `artifacts/sprint-6a/`. Any earlier commit-suffixed artifact is diagnostic
history only and is not Sprint acceptance evidence.

## Core Control-Plane Audit Contract

- `module_catalog.synchronized` is emitted by the system actor only when a committed source digest/projection changes. It records correlation ID, installation ID, affected reserved definition IDs, before/after source digests, finding-summary changes, timestamp, and success. An unchanged/repeated/concurrent loser no-op emits no event.
- `module_catalog.sync_rejected` records correlation ID, attempted source digest, stable rejection code, and timestamp after the synchronization transaction rolls back. It contains no unvalidated source secrets and no partial catalog IDs.
- `navigation_policy.updated` records the authenticated actor, correlation ID, installation ID, before/after policy revision, and before/after visibility/group/order values for every changed contribution. The event commits in the same transaction as policy state.
- `navigation_policy.update_denied` records actor, correlation ID, presented revision, stable denial/conflict code, and timestamp without disclosing hidden contribution/resource existence to an unauthorized actor.
- An idempotent policy retry/no-op emits no mutation event. Read-only inventory, descriptor, shell-navigation, destination, and resource-resolution requests are not mutation audit events; existing access/diagnostic logging policy remains unchanged.
- Catalog synchronization and transition descriptors never emit product-domain audit events or claim module-owned audit authority.

## Sprint 6A Conformance Substitutions And Deferrals

The roadmap's module-boundary completion protocol remains mandatory. Sprint 6A has no real module process, grant exchange, database, or Supervisor, so unavailable runtime checks are explicitly deferred rather than falsely marked passed.

| Roadmap conformance area | Sprint 6A executable substitute | Deferred executable proof | Accountable owner | Target |
| --- | --- | --- | --- | --- |
| Manifest/profile validation | Strict `ModuleManifestV1` valid/invalid fixtures exercise every `tessara-oci-v1` field and unsupported profile/version rejection | Supervisor executes and observes every field | Sprint 6B Platform Runtime/Supervisor owner | Sprint 6B |
| Contract compatibility/dependencies | Version-range, binding, ambiguity, cycle, transition-internal-only, and provider-ineligibility contract tests | Real provider/consumer binding against a Module Release | Sprint 6B Module Contract owner | Sprint 6B |
| Same-origin routing | Semantic route registry returns only current Core relative paths and rejects URL/owner/parameter mismatches | Gateway routes a separate module process | Sprint 6B Gateway owner | Sprint 6B |
| Scope-bound grants/downstream audience exchange | Existing in-process product authorization remains frozen; resolution fails closed | Cross-process grant/exchange/replay/freshness conformance | Sprint 6B Security/Grants owner | Sprint 6B |
| Database isolation | No new cross-feature table access or foreign key; Sprint 6A adds Core-owned additive tables only | One database and separate runtime/migration roles per real module instance | Sprint 6B Database Isolation owner | Sprint 6B |
| Module outage/degraded state | Catalog/navigation composition failure preserves Core shell routes with explicit unavailable state | Gateway fallback and real module outage/health behavior | Sprint 6B Platform Runtime/Gateway owner | Sprint 6B |
| Platform conformance suite | Contract, boundary, migration, API, SSR, browser, and transition tests in this plan | OCI runtime/migration/probe/shutdown/resource conformance | Sprint 6B Conformance owner | Sprint 6B |

Each deferred row is a named Sprint 6B entry criterion. The named owner is an
accountable delivery role that must be assigned to a person at Sprint 6B
kickoff; Sprint 6B is the approved target because no calendar date has been
approved. Deferred work is not counted as Sprint 6A passed runtime evidence and
must not be simulated with fake Release/Instance rows.

## Risk, Abort, And Rollback Matrix

| Risk | Detection / abort condition | Prevention and rollback |
| --- | --- | --- |
| Stable identifier or catalog mistake | Fixture/digest/API mismatch or an identifier changes after persistence work starts | Freeze/review catalog before persistence; abort on mismatch; before persistence, correct via a new reviewed fixture; after acceptance, version rather than rewrite |
| Source digest/projection drift | Stored source hash, `ETag`, and projection provenance differ | One transactional source/projection write and exact digest tests; abort startup and retain prior committed catalog |
| Capability/role churn | Existing capability ID, user-managed role mapping, scoped access, or session snapshot changes; a built-in role differs from the package-specific declared seed contract; or rollback evidence hides built-in membership outside the invariant fingerprint | Snapshot before migration/sync; preserve user-managed mappings and replace only named built-in seed roles from the versioned contract; retain exact built-in pairs and canonical digests beside the invariant fingerprint; allow only the documented Sprint 6A-to-Sprint 5A compatibility-package transition, then roll back transaction/binary and restore backup if any other invariant cannot be recovered |
| Navigation re-couples authorization | Hidden authorized route fails, visible unauthorized route succeeds, or default actor matrix changes | Characterize first; separate authorization helper from policy composition; disable policy mutation or deploy the tested Sprint 5A-code compatibility rollback package while additive tables remain |
| Resolver existence disclosure | Known/random restricted response differs in status, keys, fields, or timing profile | Authorize before lookup and sanitize restricted projections; abort release until shape/timing proof passes |
| Migration/sync partial state | Migration 3 or injected synchronization failure leaves rows/mappings/events, or an unmodified Sprint 5A package rejects an applied migration 3 | SQLx migration and sync transactions; abort startup; prebuild/test a Sprint 5A-code compatibility package carrying immutable migrations 1–3; no down migration; restore the pre-upgrade backup before using the original historical package |
| SSR/hydration shell regression | Missing authenticated SSR data, duplicate initial load, mismatch, console error, or no-JS unusability | Shared authorized bootstrap DTO and route matrix; roll back the UI integration while retaining control-plane data |
| Test evidence is weakened to green the branch | Deleted/skipped/loosened assertion or unexplained fixture/snapshot change | Enforce the Test Evidence And Change Control section; reject closeout until prior proof is restored or an approved decision and stronger replacement are recorded |

## Dependencies And Blockers

- Sprint 6B is the first supported real Module Manifest artifact, Module Release/Instance persistence and mutation, Supervisor, `tessara-oci-v1` execution, gateway process routing, and per-module database proof. Sprint 6A defines public types and fixtures but does not add empty persistence or simulate the runtime.
- The current repository supplies no trustworthy Core/gateway artifact digests. The control plane must report unresolved transition provenance until Sprint 6B rather than fabricating a valid Core Release.
- The roadmap names Migration, but the current source has no `/migration` route. This plan records the approved retirement with a discovery-only `retired` descriptor and `transition_destination_retired`; restoring Migration UI requires a separate approved product decision and roadmap scope.
- `/operations` is a current read-only cross-feature status view. This plan retains it as a Core-owned status projection rather than inventing an unplanned module.
- Responses currently uses `submissions:*` capability keys. The transition descriptor declares that current namespace; renaming handlers, roles, and stored capabilities is not required for Sprint 6A.
- Representative public typed adapters are required, but wholesale foreign-key replacement is deferred to physical module extraction. Existing in-process database relationships remain valid during this transition sprint.
- Navigation is the highest regression risk because current static metadata is reused for both shell visibility and route guarding. Tests must prove the split before administrator policy is enabled.
- The shared database remains the current implementation baseline for this sprint. Cross-database isolation starts with the Sprint 6B reference module.

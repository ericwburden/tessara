# Sprint 8A: Component Module Separation Slice

Status: Validation Readiness 41 passed its complete 15-check graph against
clean commit `d703e7a67c3f1177b621b0e14161e0661209ea42`, tree
`889ddc5f784ea1e81686ce63dd22669aabde956d`, and environment fingerprint
`53fbb1f74375f96abb292ba610fc030686fca55864a70ca8149932f1a4820e28`.
Candidate Rehearsal 32 then completed all 32 declared lanes under its
conservative full-harvest schedule with 19 passes, 2 raw `product` failures,
11 exact dependency blocks, and 0 deferrals. It issued no passing rehearsal
result. Post-harvest diagnosis found one shared health-probe harness root, an
authorization-before-restoration guard defect, noncanonical declaration paths,
a stale Dashboard smoke assertion, and a real Dashboard bootstrap-layout
product defect. Canonical source-exact restoration is now retained separately,
and the already-issued R32 authorization is quarantined unless paired with its
one-off append-only qualification for exactly Readiness 42. Implementation is
mutable again; no candidate exists, and preflight, SIT, formal UAT, and closeout
remain closed until complete Readiness 42 and its complete successor Rehearsal
pass against the same corrected clean source and environment identity.

- Branch: `codex/sprint-8a`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-8a`
- Base commit: `37aa9c8da45491ef02dc4d62e5df5f3ece2af444`
- Current correction input and latest failed rehearsal source:
  `d703e7a67c3f1177b621b0e14161e0661209ea42`
- Current correction input tree:
  `889ddc5f784ea1e81686ce63dd22669aabde956d`
- Latest rehearsal environment fingerprint:
  `53fbb1f74375f96abb292ba610fc030686fca55864a70ca8149932f1a4820e28`
- Roadmap authority:
  `Sprint 8A: Component Module Separation Slice (Implementation Correction)`
  and the reconciled
  Phase 8 fresh-materialization rules in `docs/roadmap.md`
- Planned evidence root: `artifacts/sprint-8a-closeout/`
- Validation record: [Sprint 8A verification](./sprint-8a-verification.md)

## Sprint Summary, Outcome, And Roadmap Authority

Sprint 8A extracts Components from Core into the separately built and deployed
`tessara.components` full-stack module. The module owns Component authoring,
viewing, versions, execution, lifecycle, persistence, configuration,
diagnostics, routes, assets, manifest, capabilities, and schema migrations. It
consumes the still-in-Core Dataset provider only through a versioned public
compatibility contract and typed Core-owned Dataset-major-line references.
Dashboard consumes Components through the exact module-owned public contract
across a real process and database boundary.

Tessara is pre-production, and the reference application contains disposable
seed data rather than user-owned production data. Sprint 8A therefore uses one
offline, destructive, source-exact materialization from empty databases. Core
and module owners rebuild the complete canonical seed in dependency order;
Dashboard creates placements directly with new Component Module Instance
references. The sprint does not migrate Component product data, rebind stored
legacy references, retain a compatibility ledger, or preserve old Core API
payloads. Supported import and legacy mapping remain Phase 9 work.

## Scope

### In scope

- Create Module Definition `tessara.components`, initial release `1.0.0`, one
  selected Module Instance, an isolated Component database, and separate
  runtime and schema-migration identities.
- Move all Component UI, API, version/lifecycle behavior, execution,
  persistence, complete-document SSR, hydration, assets, configuration,
  health/readiness, diagnostics, manifest, capabilities, and conformance
  ownership out of Core.
- Publish exact-current Components contract `3.0.0` with Module Instance
  ownership and resource type `tessara.components.component_version`.
- Replace Component-to-Dataset foreign keys and implementation calls with a
  versioned Core Dataset compatibility contract and typed Dataset-major-line
  references.
- Keep Component product page URLs and accepted authoring/viewing behavior,
  while replacing old Core Component payloads with canonical module-owned
  typed-reference request/response shapes.
- Rebuild the entire disposable reference-application seed from empty through
  owning Core/module bootstrap contracts. Seed Components before Dashboards;
  Dashboard bootstrap uses read-back new Component references.
- Remove Core Component tables, handlers, route adapters, product DTOs,
  execution, transition adapter/readers, and old payload acceptance in the
  same offline cutover.
- Preserve Dashboard-owned layout, dependency findings, deferral,
  Upgrade/Replace/Remove, lifecycle observation, nondisclosure, outage, and
  recovery behavior against the new provider.
- Add Component module configuration with a shared-shell display label and a
  bounded Dataset-provider timeout, plus sanitized operational diagnostics.
- Pair every stale seed, smoke, UAT, Playwright, manifest, deployment, and
  bootstrap update with the behavior that requires it.
- Prove Component-only health-gated upgrade and rollback without rebuilding or
  restarting Core, Dashboard, Dataset compatibility, or unrelated modules.

### Explicitly out of scope

- Preserving or migrating current Core Component, Dashboard, or other
  reference-application product rows.
- Old-to-new reference mappings, Dashboard rebinding commands, migration
  receipts, migrated/retired reference ledgers, partial-data repair, checkpoint
  resume, dual write, or online/read-only cutover behavior.
- Any normal-runtime acceptance of old `core_installation` Component references
  or old Core Component API payload shapes.
- Dataset physical extraction; Sprint 8B owns that boundary and will use the
  same Phase 8 fresh-materialization policy.
- New Component kinds, lifecycle rules, Dashboard product policy, navigation
  policy, or visual redesign.
- General supported import/migration UX or a migration coordinator; Sprint 9A
  owns that work.
- Multi-instance selection, external module repositories, or package
  publishing.

## Pre-Implementation Findings And Affected Components

The findings below are the baseline that governed the extraction. They are
retained as decision history; the implementation handoff later in this plan
records the resulting current ownership.

- Core `tessara-api` owns Component routes, DTOs, execution, lifecycle, and
  persistence, and its squashed baseline contains Component tables with direct
  Dataset foreign keys.
- Component UI is split between root-owned routes, `tessara-web-components`,
  and the route-free `tessara-web-component-viewer` implementation.
- `tessara-components-contract` V2 requires `core_installation` ownership and
  the transition resource type. A new exact generation is required; permissive
  dual-owner parsing would preserve an obsolete boundary.
- Dashboard is independently deployed but its persisted placement schema and
  manifest still target Core-owned Component references. With the approved
  fresh reset, its database and seed are recreated directly against new
  references rather than mutated through a rebind protocol.
- Current Component execution reads Dataset state through Core implementation
  code and storage. The extraction requires catalog, schema, distinct-value,
  compatibility, and execution/materialization contract operations.
- Sprint 6D/6E established canonical contract/runtime/UI/testkit,
  complete-document, generic routing, package-boundary, independent-image
  upgrade, and rollback patterns. Components must reuse those patterns without
  depending on root Core/web/API or another module implementation.
- Existing Component-specific rendering limits remain stored in each
  ComponentVersion. The only new installation-level product configuration is
  the display label and Dataset request timeout.
- The retained Sprint 7A deployment profile is immutable. Sprint 8A requires a
  new profile and explicit disposable-scope reset authorization.

The implementation touched Component module/UI ownership,
Components/Dataset contract packages, Dashboard dependency and seed ownership,
Core removal, generic gateway/composition registration, the Sprint 8A
deployment/bootstrap path, end-to-end tests, and acceptance runners.

## Specifications

### 1. Module, source, UI, and operational ownership

- `tessara.components` release `1.0.0` declares product and operational routes,
  Feature Declarations, capabilities `components:read` and
  `components:manage`, configuration schema v1, database, probes, assets,
  contracts, dependencies, and provenance.
- Enrolled Component and Dashboard Manifests are the sole startup owners of
  their capability rows, and composition Blueprint roles are the sole owner of
  module-specific grants and action declarations. Core startup and its fresh
  baseline contain no static Component, Dashboard, or Scoped Records action,
  capability, or built-in grant rows.
- The Component release is the sole production owner of `/components`,
  `/components/new`, `/components/{component_ref}`,
  `/components/{component_ref}/edit`,
  `/components/{component_ref}/versions`, and
  `/components/{component_ref}/view`, plus their canonical API routes.
- Complete documents use verified Shell Context and `tessara-module-ui`.
  Direct loads, useful JavaScript-disabled SSR, hydration, history, theme,
  keyboard, accessibility, responsive layouts, and clean browser consoles
  remain required.
- User-facing product behavior and page vocabulary remain “Components.” No
  redesign or new migration status UI is introduced.
- Dashboard must not compile a Component feature/viewer implementation. Its
  placement renderer is consumer-owned and consumes only public execution
  envelopes plus canonical platform/UI packages.
- Native/WASM package and image audits prohibit root `tessara-web`,
  `tessara-api`, Core-private code, Dashboard implementation, Dataset
  implementation, and copied shared source from the Component release.

### 2. Configuration and diagnostics

Configuration schema v1 contains exactly:

- `display_label`: required string, trimmed, 1–80 Unicode scalar values,
  default `Components`; shown in shared-shell navigation and Module Management
  only. Product headings, breadcrumbs, API names, capability IDs, route names,
  and contract IDs remain canonical “Components” vocabulary.
- `dataset_request_timeout_seconds`: required integer, inclusive range 1–30,
  default 5; applied to Component-to-Dataset provider requests.

Installation-global Module Management manager authority continues to control
configuration through the existing generic path. `components:manage` does not
grant module-configuration authority, and no new capability is introduced.
Unknown fields and unsupported schema versions fail validation; normalization
does not silently clamp out-of-range timeout values.

Diagnostics expose release/instance identity, manifest and contract versions,
effective redacted configuration, Dataset dependency binding and compatibility,
last provider health observation, readiness/liveness, and stable sanitized
failure codes. They expose no product rows, counts, secrets, credentials,
resource identifiers, obsolete migration receipts, or old-reference details.
The Component Manifest carries the actual schema defaults (`Components` and
`5`) and its runtime and migration commands name the executable that exists in
the image, `/usr/local/bin/component-module`. Dataset diagnostics identify the
selected `tessara.components.dataset-major-line` binding, Core-installation
provider, contract ID/version, most recent compatibility result, health state,
stable result/failure code, and observation time without exposing a raw
Dataset reference.

### 3. Component persistence and Dataset compatibility

- The Component database owns component shells, versions, lifecycle/publication
  state, change audit, typed Dataset bindings, configuration, and idempotency.
  No other runtime receives its credentials.
- ComponentVersion stores a typed Core-owned Dataset-major-line reference rather
  than a Dataset foreign key. The compatibility wrapper validates installation,
  owner, transition resource type, canonical composite Dataset UUID/positive
  major identity, contract ID/version, and schema centrally.
- Core provides versioned Dataset catalog, major-line metadata/schema,
  distinct-values, compatibility, and execution/materialization operations
  sufficient for every current Component kind and editor flow.
- Component exchanges the actor context through Core for the Dataset audience.
  Core authorizes the presenting Component service for the declared action;
  Dataset scope/capability checks remain provider-owned.
- Every signed Component, Dataset-compatibility, and authorization-exchange
  service request binds the exact transmitted body bytes. For JSON-bearing
  calls, receivers validate the media type and verify authorization, service
  identity, correlation, and the raw-body digest before typed deserialization.
  Parsing and re-encoding a request is not valid signature verification.
- Known and random unauthorized Dataset references are indistinguishable.
  Unavailable, timed out, incompatible, unauthorized/not-evaluated, missing,
  retired, and materialization-not-ready remain distinct internally and map to
  stable non-leaking Component results.
- During Dataset outage or timeout, existing Component metadata/configuration
  loads read-only with retry. Preview, save, publish, lifecycle, and all other
  Dataset-dependent mutations are unavailable until validation succeeds.
  Unsaved browser state is retained across retry. No operation falls back to
  direct SQL or persists a validation-pending draft.

### 4. Components v3 and Dashboard behavior

- `tessara.components.component-version` `3.0.0` is the only normal runtime
  generation. Its resource type is
  `tessara.components.component_version`, and every reference requires the
  selected Component Module Instance owner.
- V3 preserves Sprint 7B catalog, resolution, observation, lifecycle,
  publication, resource-revision, change, successor, execution, and
  nondisclosure semantics while adopting canonical module ownership.
- Historical V1/V2 fixtures remain immutable test evidence only. Normal
  Component and Dashboard readers reject V1/V2, old Core owners, the transition
  resource type, mixed generations, wrong instances, cross-installation
  references, malformed envelopes, and old Core payload shapes.
- Existing same-origin API paths may remain for product continuity, but their
  request/response bodies change directly to canonical v3 typed-reference
  shapes. All first-party clients, tests, smoke, UAT, and seed code change in
  the same candidate. No translation facade is retained.
- Dashboard continues to own layout, observations/findings, deferral,
  Upgrade/Replace/Remove, degraded presentation, and consumer action policy.
  Components and Core do not acquire Dashboard persistence or policy.
- Dashboard-to-Component calls use generic service registration, downstream
  audience exchange, the configured timeout behavior at the provider boundary,
  and exact v3 contracts. Component outage remains contained to Dashboard
  placement/dependency state; unrelated routes remain healthy.
- `tessara-components-contract` owns the one exact Table/visual render response
  and `ComponentRenderKind`. Table and each visual kind accept only their own
  fields and payload branch. Persistent execution requires matching non-nil
  Component and ComponentVersion identities; unsaved preview alone uses the
  explicit both-nil preview identity, and partial-nil identities fail closed.
  Render output carries no unused Dataset provider reference.
- Dashboard sends only the stored Dashboard scope authorized by its inbound
  Dashboard grant. After resolving authorized Component metadata it exchanges
  an exact resource assertion binding ComponentVersion type/id, authority
  revision, and canonical Component scope. Dashboard and Component both require
  one governing node shared by the Dashboard scope, Component scope, inbound
  Dashboard authority, and downstream Component authority. A disjoint
  placement is restricted before title, Component metadata, scope, or Dataset
  identity can be projected, even for an actor separately authorized in both
  non-overlapping scopes.

### 5. Offline fresh materialization and seed ownership

- Sprint 8A uses a dedicated disposable Compose project/profile. Validation
  resolves and records exact project, volume, database, and port targets and
  requires explicit destructive-reset authorization before teardown.
- The public gateway is unavailable for the entire extraction. There is no
  maintenance page, read-only product mode, hidden-action protocol, or online
  compatibility interval.
- Materialization creates empty Core, Supervisor, Component, Dashboard, and
  other selected-module databases, applies each owner's schema migrations, and
  starts the new source-exact topology.
- Health is owner-specific and redirect-free throughout materialization,
  smoke, upgrade/rollback, recovery, and final restoration. Core proves
  `GET /health` with HTTP 200, `text/plain`, and exact body `ok`; Supervisor
  proves `GET /health/ready` with HTTP 204 and an empty body. A followed login
  redirect, HTML document, swapped endpoint, or generic HTTP-success range is
  not health evidence.
- Canonical seeding runs only through owning APIs/bootstrap contracts in this
  dependency order:
  1. Core installation, identity/RBAC, Organization, and still-in-process
     provider records;
  2. Component module records using Dataset compatibility references;
  3. Dashboard module records using Component v3 references returned by
     Component read-back.
- Orchestration may pass typed IDs and content-addressed seed inputs between
  owners, but it cannot write product tables or use another owner's credentials.
- Component's locked Manifest declares Dataset bootstrap validation and selects
  `/dependency_validation` from its canonical inline input. The generic
  lockfile path resolves the exact target, binds the complete request to a
  one-use authorization, and leaves Dataset semantics with the provider; Core
  and Supervisor contain no Component/Dataset product branch. Any target,
  payload, apply, owner, audience, signature, expiry, or replay failure occurs
  before Component writes.
- A successful seed exposes recognizable fixtures for every Component kind,
  lifecycle/authorization/outage case, and Dashboard dependency action. Exact
  counts, where contractual, come from one shared acceptance source of truth.
- The Sprint 8A reference seed is exactly seven Component shells and eight
  ComponentVersions: one shell for each of Table, Bar, Line, Pie, Donut, and
  Stat Card plus one blocked-scope Table shell. The Stat Card shell has a
  superseded/inactive `1.0.0` predecessor whose declared successor is the
  published/active `2.0.0` version; every other shell has one published/active
  version.
- The one reference Dashboard has exactly seven placements. Four retain the
  normal row-count, table, bar, and blocked-scope coverage. Three independent
  lifecycle-action placements bind the inactive predecessor and are named
  `lifecycle-upgrade`, `lifecycle-replace`, and `lifecycle-remove`; they make
  Defer followed by Upgrade, Replace, and Remove independently executable.
- Dashboard bootstrap persists one canonical versioned placement-config shape
  and the exact one-based row/column/width/height layout: `row-count` at
  1/1/4/2, `records` at 3/1/12/6, `tier-chart` at 9/1/6/4, `blocked-scope` at
  9/7/6/4, and the three lifecycle placements at row 13, columns 1/5/9, each
  width 4 and height 2. Bootstrap-only placement keys are validated for unique
  orchestration identity but are not persisted in product config. The public
  response exposes opaque placement IDs, geometry, resolution, and only
  authorized Component identity; it does not expose bootstrap-only
  `placement_key` labels.
- Repeating bootstrap without reset is an exact semantic no-op: no duplicate
  product rows, revisions, references, roles, configuration, or receipts.
- Any failure retains the failed attempt and raw evidence, destroys the partial
  disposable topology and volumes, and restarts from empty after correction.
  Partial repair, stage resume, old-stack restoration, and data salvage are not
  supported.

### 6. Core removal, observability, upgrade, and rollback

- Final Core contains no Component product tables, schema types, handlers,
  routes, DTOs, UI adapters, execution, seed writes, transition adapter,
  transition reader, old-payload reader, or Component-specific gateway branch.
- Old Core Component references and payloads are unsupported inputs. They fail
  exact contract validation and are never reported as migrated or retired.
- Generic platform integration retains only manifest/catalog, Shell Context,
  authorization exchange, service registration, dependency resolution,
  configuration/diagnostics proxy, and unavailable-route behavior.
- Logs and diagnostics include correlation, actor/service class,
  installation/instance/release/contract identity, reference digests, timeout
  and stable result codes without restricted resource or secret data.
- OCI labels and evidence bind commit, tree, dirty state, release, manifest,
  assets, schemas, fixtures, contracts, bootstrap, and acceptance inventory.
- After fresh extraction, prove a compatible Component-only health-gated
  upgrade and rollback with a distinct source-built compatible Component
  release `0.9.0` and the intended current release `1.0.0`. Each transition is
  a resolved one-owner Blueprint delta applied by the out-of-process
  Supervisor through its Compose deployment adapter. Component data, instance
  identity, configuration, routes, and behavior persist while Core, gateway,
  Supervisor, Dashboard, Dataset compatibility, and unrelated module image
  digests, container identities, restart counts, data, and availability remain
  unchanged.
- Final handoff restores the intended current Component release in the healthy
  freshly seeded Sprint 8A topology. Rollback evidence is not the handoff state.

### 7. Compatibility posture

Tessara remains pre-production. Sprint 8A is forward-only and intentionally
destructive. There is no transition-data preservation, dual write, populated
legacy upgrade, old contract negotiation, old reference resolution, old API
translation, checkpoint resume, or extraction rollback to the Core-owned
product. Schema migrations, fresh bootstrap idempotence, and post-extraction
Component release rollback remain required. Supported legacy import and
cross-module mapping are deferred to Phase 9.

## Decisions, Assumptions, Open Questions, Dependencies, And Blockers

### Approved product decisions

- Prioritize transition simplicity over availability and preservation because
  the application is not in production use.
- Apply the destructive fresh-materialization policy to every Phase 8 module
  extraction, not only Sprint 8A.
- Take the full reference application offline, rebuild every disposable product
  database from empty, and fresh-seed through owning contracts.
- On failure, destroy the partial topology and rerun from empty; do not restore,
  resume, or repair transition data.
- Do not migrate current seed data or create a synthetic legacy migration
  fixture. Real legacy migration/rebinding acceptance is removed from Phase 8.
- Do not retain a migration ledger or receipt UI. Old Component references and
  old Core payload shapes are unsupported.
- Preserve same-origin product URLs and accepted UI behavior, but make the API
  wire contract a direct canonical break with all first-party consumers updated
  together.
- Configure only display label and Dataset request timeout. The label affects
  navigation and Module Management, and the timeout defaults to 5 seconds with
  a strict 1–30 second range.
- During normal Dataset outage, Component authoring is read-only with retry;
  validation-dependent mutations do not save pending state.
- No visual mockup approval is needed unless implementation discovers a real
  redesign requirement, which pauses that surface for a plan amendment.

### Assumptions and dependencies

- The closed Sprint 7B behavior is the functional baseline, not a data-retention
  obligation.
- Sprint 6D/6E canonical packages and Dashboard extraction patterns remain the
  implementation baseline.
- Core generic manifest, gateway, composition, Shell Context, authorization,
  and Module Management paths can enroll Components without a module-ID branch.
- All destructive commands operate only on an explicitly resolved disposable
  Sprint 8A project and never on broad or ambiguous targets.

### Open questions and blockers

No product decision or planning blocker remains. Internal crate layout, port
allocation, opaque Dataset-major-line encoding, and runner names are bounded
implementation details. Any request to preserve legacy data/references, add an
online transition, restore old payload compatibility, or redesign product UI
requires a plan amendment and user approval.

## Acceptance Criteria

- **AC-01:** The Component image owns its binary, database/schema migration,
  manifest, complete documents, hydration, assets, configuration, diagnostics,
  and every Component product route, with no root Core/web/API or sibling
  implementation dependency.
- **AC-02:** Final Core has no Component product storage, code, routes, seed
  writes, static module action declarations, capability rows or built-in
  grants, adapter, legacy reader, or module-specific integration branch.
  Dashboard declarations, capability rows, and grants likewise enter only
  through its real enrolled release/instance, Manifest, and Blueprint.
- **AC-03:** From explicitly authorized empty disposable databases, one
  source-exact materialization produces a healthy complete reference
  application and owner-controlled canonical seed in the declared order:
  exactly seven Component shells, eight ComponentVersions, and seven Dashboard
  placements. Component seed is accepted only after its exact locked
  Manifest/lockfile target and opaque inline payload pass provider-owned
  Dataset validation under the exact one-use signed request; altered or
  replayed invocations are non-disclosing and leave zero Component rows or
  receipts. Health evidence uses Core `GET /health` = 200 `text/plain` body
  `ok` and Supervisor `GET /health/ready` = 204 empty, with redirects disabled.
- **AC-04:** An unchanged second bootstrap is a semantic no-op with no duplicate
  data, references, revisions, roles, configuration, or receipts.
- **AC-05:** Dashboard seed creates every placement directly with a Component
  v3 Module Instance reference returned by Component read-back; no old owner or
  transition resource type exists in live state. Its exact seven placements
  include three independent lifecycle-action fixtures bound to the one
  superseded/inactive predecessor with a published/active successor. Bootstrap
  persists the canonical versioned geometry above; smoke asserts public opaque
  placement IDs and exact geometry without requiring the private
  `placement_key` label.
- **AC-06:** Old Core Component references, V1/V2 runtime envelopes, and old
  Core payload shapes fail exact normal-contract validation; no translation,
  migrated/retired result, or fallback exists.
- **AC-07:** Component directory, create, edit, versions, view, every supported
  kind, publication, lifecycle, and execution remain user-visible and
  behaviorally equivalent through module-owned same-origin routes.
- **AC-08:** Configuration validates exact schema v1, label behavior, timeout
  default/range, authority, persistence, and sanitized diagnostics.
- **AC-09:** Component stores typed Core-owned Dataset-major-line references and
  performs catalog, validation, distinct-value, compatibility, and execution
  only through the versioned Dataset contract. Signed service-request body
  digests are checked against the exact raw bytes before deserialization.
- **AC-10:** Dataset scope/audience/capability, known/random nondisclosure,
  incompatible, timeout, outage, not-ready, and recovery cases pass without
  direct database access or validation-pending writes. Dashboard-mediated
  rendering additionally proves one common governing node across Dashboard and
  Component scopes and grants; independent authority on disjoint scopes cannot
  disclose or render the placement.
- **AC-11:** Dashboard preserves Sprint 7B lifecycle/revision observation,
  findings, restricted disclosure, and recovery against Components v3. One
  executable semantic diagnostic proves three exact lifecycle findings,
  Defer followed by Upgrade through the declared successor, independent
  Replace and Remove actions, blocked-scope nondisclosure, exact contained
  Component-provider outage findings, and convergence to zero findings after
  recovery.
- **AC-12:** Runtime/migration credentials cannot cross Core, Component,
  Dataset, or Dashboard database ownership; no cross-database SQL, FDW, shared
  writable schema, or foreign key exists.
- **AC-13:** A failed fresh materialization retains evidence, removes the exact
  partial disposable topology and volumes, and succeeds only after a complete
  from-empty rerun.
- **AC-14:** A real source-built compatible Component `0.9.0` release upgrades
  to `1.0.0`, rolls back to `0.9.0`, and restores `1.0.0` through exact
  one-owner Supervisor/Compose applies. Component data, identity,
  configuration, routes, and behavior persist while unrelated digests,
  identities, restart counts, data, and availability remain unchanged.
- **AC-15:** At 1280, 768, and 390 px, dark/light themes, keyboard operation,
  200% zoom, JavaScript-disabled SSR, and hydrated navigation, Component UI
  matches the accepted baseline and produces no hydration or console errors.
- **AC-16:** Core's frozen transition catalog contains exactly Forms,
  Workflows, Responses, Datasets, and Migration. Dashboard is absent from Core
  transition inputs, inventory, semantic destination resolution, and default
  navigation; it appears exactly once through its enrolled Module Release,
  Module Instance, and manifest navigation contribution.
- **AC-17:** Format, check, zero-warning Clippy, full workspace tests,
  native/WASM/package boundaries, Playwright, source-exact materialization,
  smoke, scripted/manual UAT, failure rerun, upgrade/rollback, provenance, and
  evidence integrity pass on one frozen candidate.
- **AC-18:** The Components contract is the sole owner of exact Table/visual
  render responses and kind vocabulary. Wrong branch/field/kind/schema,
  unknown fields, Dataset-provider identity, mismatched or nil persistent
  identity, non-nil or partial-nil preview identity, and stale Component
  resource assertions fail closed at the owning boundary.
- **AC-19:** The active first-party clients, smoke, UAT, Playwright, and seed
  fixtures use only exact v3 Component and typed Dataset-reference shapes. No
  normalization helper, alias, flat Dataset-major fields, or copied-count
  expectation preserves the retired Core payload contract.

## Traceability Matrix

| Roadmap requirement | Specification / acceptance | Slice | Automated/deployed proof | Manual UAT |
|---|---|---:|---|---|
| Independently built/deployed full-stack module | Specs 1/6; AC-01/12/14 | 1–6 | package/image/source audit; topology; upgrade chronology | UAT-8A-01/06/08 |
| Canonical packages; no Core/root/sibling implementation | Spec 1; AC-01/02 | 1/4 | native/WASM dependency and forbidden-source audits | UAT-8A-01/06 |
| Module-owned admin/config/manifest/capabilities/DB/schema migrations/health/routes/assets/conformance | Specs 1–2/5; AC-01/08/12/16 | 2–6 | manifest, testkit, schema, route, asset, credential suites | UAT-8A-01/03/06 |
| APIs/contracts/typed references only | Specs 3–4; AC-09–12 | 1/3/4 | contract, source, SQL, and credential tests | UAT-8A-04/05/06 |
| Exact render kind/identity and raw signed bodies | Specs 3–4; AC-09/18 | 1/3/4 | contract invalid-shape matrix; raw-byte tamper; provider/consumer integration | UAT-8A-04/05/06 |
| Joint Dashboard/Component governing scope | Spec 4; AC-10/11/18 | 3/4/6 | exact assertion, common-node, redacted projection and browser scenarios | UAT-8A-04/05 |
| No active legacy first-party Component facade | Specs 4/7; AC-06/19 | 1/4/6 | exact source guard plus smoke/UAT/Playwright typed-reference fixtures | UAT-8A-01/04/06 |
| Phase 8 fresh materialization and unsupported old references | Specs 5/7; AC-03–06/13 | 1/5/6 | empty bootstrap, no-op, old-input rejection, teardown/rerun | UAT-8A-02/07 |
| Rerun scope/lifecycle/outage/compatibility/source/image/rollback | Specs 1–6; AC-10–16 | 1–6 | conformance, Playwright, smoke, upgrade/rollback | UAT-8A-04/05/06/08 |
| Move all Component product/operational ownership | Specs 1–3/6; AC-01/02/07/08 | 2–4 | module integration and Core absence | UAT-8A-01/03 |
| Real Component Release/Instance in fresh database | Specs 1/5; AC-01/03/12 | 2/5 | composition read-back, isolated DB/provenance | UAT-8A-02/06 |
| Dashboard placements created with new references | Specs 4–5; AC-05/11 | 4/5 | semantic seed and Dashboard regression smoke | UAT-8A-02/05 |
| Replace Dataset DB relationships with compatibility contract | Spec 3; AC-09/10/12 | 1/3/5 | typed reference, scope, outage, no-SQL tests | UAT-8A-04 |
| Preserve Dashboard public Components behavior | Spec 4; AC-05/11 | 3–6 | v3 consumer regression and deployed smoke | UAT-8A-05 |
| Rebuild seed/test data through owners | Spec 5; AC-03–05/12/13 | 5 | seed ownership, read-back, no-op and failure-rerun tests | UAT-8A-02/07 |
| Dependency outage, scope, capability, compatibility | Specs 3–4; AC-10/11 | 3/5/6 | nondisclosure, timeout, outage/recovery smoke | UAT-8A-04/05 |
| Unchanged UI plus module configuration/diagnostics | Specs 1–2; AC-07/08/15 | 2/4/6 | SSR/hydration/visual/accessibility/console tests | UAT-8A-01/03 |
| Exit: fresh seed uses only new Component references | Spec 5; AC-03–06 | 5/6 | source-exact read-back and old-input absence/rejection | UAT-8A-02 |
| Exit: Component execution through Dataset contracts | Spec 3; AC-09/10 | 3/5/6 | real-boundary smoke and credential denial | UAT-8A-04 |
| Exit: Dashboard consumption and coherent degradation | Spec 4; AC-11 | 4–6 | Dashboard deployed outage/recovery lane | UAT-8A-05 |

## Implemented Ordered Slices

The slice order is retained as the governing implementation history. All six
slices are represented in the current mutable source; formal acceptance still
depends on the successor complete Readiness and Rehearsal gates.

### Slice 1 — exact contracts and failing ownership boundaries

- Prerequisites: approved reconciled planning package and clean sprint worktree.
- Touchpoints: Components/Dataset contract packages, Dashboard dependency
  declaration, source/package/route/storage audit helpers.
- Work: lock Components v3, typed Dataset-major-line, canonical API payload,
  package graph, route ownership, unsupported-old-input, and no-cross-storage
  contracts as failing tests.
- Harness in slice: golden/invalid/historical fixtures, native/WASM dependency
  audits, accepted-test inventory integrity.
- Complete when the target boundary is reviewable as red tests without product
  code or data moving opportunistically.

### Slice 2 — Component release, database, configuration, and operations

- Prerequisites: Slice 1 contracts.
- Touchpoints: Component module/UI ownership, manifest, schema migration,
  configuration, diagnostics, Compose image/profile scaffolding.
- Work: establish release/instance, isolated identities/database, contract
  ownership, exact configuration, probes, diagnostics, and testkit conformance
  without switching product traffic.
- Harness in slice: manifest/config/diagnostic/probe/schema/credential tests and
  source/image provenance assertions.
- Complete when a shadow empty Component instance materializes and reports
  exact identity/configuration with no Core product ownership change.

### Slice 3 — Component product and Dataset contract boundary

- Prerequisites: Slices 1–2.
- Touchpoints: Component service/API/persistence/execution, Dataset compatibility
  adapter/contract, authorization exchange, Component seed bootstrap.
- Work: move Component behavior behind module-owned canonical APIs and replace
  every Dataset storage/implementation edge with typed authenticated calls.
- Harness in slice: all kinds and lifecycle paths; scope, capability, audience,
  known/random nondisclosure, timeout, outage, not-ready, recovery, read-only
  editor/retry, and negative credential/source coverage.
- Complete when the shadow module reproduces accepted behavior across the real
  Dataset boundary and stores no Dataset foreign key.

### Slice 4 — module-owned UI/API cutover and Core removal

- Prerequisites: Slice 3 parity.
- Touchpoints: Component documents/hydration/assets, Dashboard consumer
  renderer/v3 client, generic gateway registration, root web/API removal.
- Work: move all same-origin UI/API traffic to Components, update every
  first-party client to canonical payloads, remove Component implementation
  dependencies from Dashboard, and remove every Core product/legacy path.
- Harness in slice: direct/shell/no-JS/hydration/history/theme/keyboard/
  responsive/accessibility/console, API exactness, Core absence, old-input
  rejection, Dashboard v3 regression.
- Complete when normal traffic is module-owned and no runtime reader accepts an
  old owner, type, generation, or payload.

### Slice 5 — destructive materialization and owner-controlled seed

- Prerequisites: Slice 4 complete boundary.
- Touchpoints: Sprint 8A Compose/bootstrap, all owner seed entrypoints,
  Dashboard seed, general/sprint smoke and UAT fixtures/evidence.
- Work: implement explicit safe reset, empty schema materialization, ordered
  owner bootstrap/read-back, Dashboard new-reference seed, exact second-run
  no-op, induced failure teardown, and complete from-empty rerun.
- Harness in slice: target-resolution/reset self-tests, semantic fixture source
  of truth, no cross-owner writes, first/no-op receipts, failure evidence,
  teardown verification, canonical restoration.
- Complete when a mutable rehearsal-shaped run produces the full recognizable
  application from empty and recovers from failure only by full rerun.

### Slice 6 — complete acceptance and independent upgrade/rollback readiness

- Prerequisites: Slice 5 clean rehearsal behavior.
- Touchpoints: Playwright/permissions, deployed smoke, scripted/manual UAT,
  upgrade/rollback runner, final acceptance inventory and documentation.
- Work: prove unchanged UX, configuration, API exactness, Phase 7 behavior,
  source/storage isolation, outage containment, Component-only upgrade/rollback,
  final current-release restoration, and validation readiness.
- Harness in slice: full commands and evidence-schema/manifest self-tests.
- Complete when implementation is clean and ready for the validation
  coordinator. This slice does not freeze or authorize a candidate.

## Verification And UAT Plan

Focused implementation verification uses the all-feature, offline, warnings-
denied command set recorded in the verification record. It is not a formal
gate. R32's immutable attempt, harvest, raw two-defect batch, diagnostic
supplement, post-harvest restoration, and authorization qualification remain
append-only. After the mutable correction is committed cleanly and the
validation coordinator authorizes the exact transition, only the qualified
one-use R32 authorization may admit the next formal commands:

```powershell
.\scripts\validate-sprint-8a-readiness.ps1 -Attempt 42
.\scripts\run-sprint-8a-candidate-rehearsal.ps1 -Attempt 33 -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json"
```

Only when both complete gates pass against the same source and environment
identity may the coordinator run the tracked downstream lifecycle:

```powershell
.\scripts\run-sprint-8a-validation-preflight.ps1 -Attempt <preflight-n> -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json" -RehearsalReceipt "artifacts/sprint-8a-closeout/candidate-rehearsal-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -ExpectedBranch "codex/sprint-8a" -HandoffUrl "http://127.0.0.1:8088"
.\scripts\run-sprint-8a-sit.ps1 -Stage Run -Attempt <sit-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/sit-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Start -Attempt <uat-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088"
# Acquire and hold one manual scenario execution lease, execute and publish
# UAT-8A-01 through UAT-8A-08 one at a time, as specified in the verification record.
.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Finalize -Attempt <same-uat-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

The preflight runner owns all ten declared checks and freezes a candidate only
after they pass. The SIT runner owns all four lanes, safe fail-late harvesting,
canonical restoration, and aggregate publication. The formal-UAT runner owns
the Start/manual-lease/Finalize state machine. A scripted defect changes manual
work to non-authoritative diagnostic harvesting; a product decision pauses it.
After scripted, manual, and restoration evidence is durably complete, an
evidence-finalization failure may retry publication only, without rerunning
manual validation or restoration. `local-launch.ps1` remains a root-profile
regression check and cannot replace any source-exact lifecycle evidence.

Manual UAT covers eight scenarios: unchanged Component product experience;
from-empty owner-controlled seed and new Dashboard references; configuration
and diagnostics; Dataset contract/joint-scope/timeout/outage; Dashboard
lifecycle and outage behavior; source/database isolation and old-input
rejection; failed materialization teardown plus full rerun; and Component-only
upgrade/rollback. The tracked Playwright acceptance inventory contains 75 exact
scenario identities, including the shared-node success/disjoint-node redaction
and render-denial case mapped to UAT-8A-04.

## Validation, Evidence, Freeze, Failure, And Closeout Plan

- Run complete Test Readiness and complete mutable Candidate Rehearsal. Both
  must pass after the last correction before preflight may freeze a candidate.
  Treat the complete Rehearsal as final certification, not the primary
  debugging loop. Before handing a correction back, retain non-authoritative
  clean-environment materialization/no-op, containment/recovery, exact-health,
  product-smoke, acceptance, fixture, evidence, and runner reproducers. Two
  consecutive failures of one formal lane forbid another full launch until its
  clean focused reproducer passes; a third makes that lane a concentrated
  validation-platform incident that must be resolved before relaunch.
- Readiness fixes the next Rehearsal's complete declaration graph and bounded
  two-wave order. Wave A contains lifecycle/authentication, preceding failures,
  newly reachable and correction-affected lanes, three-times-deferred lanes,
  their safe prerequisite closure, and failure-containment recovery. If Wave A
  places source-exact materialization and containment/recovery immediately
  after its cheap lifecycle prerequisites as the first expensive branch;
  stable expensive checks remain later unless a real prerequisite or the
  correction cone requires them. If Wave A passes, the same attempt executes
  every Wave B lane. If Wave A fails, it finishes all
  safe Wave A siblings, then records only eligible prior passes outside the
  cone as `deferred`; any deferral keeps the attempt incomplete. After Wave B
  execution/deferral, run aggregate sinks, then mandatory terminal canonical
  restoration and safety finalizers. Unauthenticated history selects full
  harvest.
- A failed rehearsal may retain terminal accounting, one harvest, and one
  consolidated batch while restoration is unproven, but cannot issue usable
  correction authorization. Prospective authorization requires the exact
  passing current-attempt final-health and final-environment receipts and
  `cleanup_restoration.result = canonical_successor_healthy`.
- Preflight binds commit/tree/dirty state and every product, test, harness,
  schema migration, fixture, seed, manifest, bootstrap, deployment,
  configuration, contract, and acceptance-inventory identity.
- One fingerprint covers authoritative SIT and UAT. UAT starts only after every
  SIT lane, including deployed acceptance smoke, passes.
- Candidate or tracked harness/fixture/seed/manifest/bootstrap/acceptance change
  invalidates all SIT and UAT. Scoped reruns require unchanged immutable
  candidate/environment receipts and the protocol's explicit justification.
- A candidate-invalidating UAT defect enters Post-SIT Defect Convergence:
  retain failure, forbid passing UAT, harvest safe diagnostics, correct one
  mutable batch, authorize an impact cone, run focused repair validation, then
  rerun complete readiness, rehearsal, preflight, SIT, and UAT.
- Retain append-only hashed receipts under `artifacts/sprint-8a-closeout/` and
  distinguish failed/superseded attempts.
- Closeout requires coordinator-produced `closeout-authorization.json` and a
  healthy freshly seeded topology with the intended current Component release.

## Risks And Controls

| Risk | Prevention | Detection | Recovery |
|---|---|---|---|
| Destructive reset targets wrong resources | resolve exact project/volumes/databases; explicit disposable authorization | readiness target receipt and pre-delete identity check | stop before deletion; no ambiguous target allowed |
| Seed order creates invalid references | owner APIs/read-back; Components before Dashboards | typed-reference and semantic fixture assertions | destroy partial topology and rerun from empty |
| Old reader or payload survives | exact v3 and forbidden-source/route audits | old owner/type/generation/payload negative tests | remove stale path and rerun complete validation cycle |
| Dataset boundary leaks scope/existence | audience exchange; authorization first | known/random API/UI/log/timing conformance | invalidate candidate; broad security retest |
| Dataset outage corrupts authoring | read-only/retry; no pending persistence | timeout/outage browser and database assertions | recover provider and retry preserved browser state |
| Cross-database ownership regression | distinct credentials and source guards | negative credential/SQL/package tests | remove access and rematerialize from empty |
| Dashboard product regression | preserve consumer policy; v3 typed seed | Sprint 7B regression, smoke, Playwright, UAT | invalidate and rerun full chain after correction |
| UI drift/hydration failure | shared shell and baseline captures | viewport/theme/no-JS/console evidence | correct before freeze or pause redesign decision |
| Component outage cascades | generic gateway containment and timeouts | outage/recovery plus unrelated-health probes | restore prior healthy Component release |
| Rollback crosses incompatible schema | compatible release contract and schema audit | upgrade/rollback chronology/read-back | restore compatible Component release; rematerialize only in disposable recovery |
| Evidence is stale or misleading | one fingerprint and hashed prerequisites | receipt/schema/manifest audit | retain superseded evidence and restart required boundary |

## Planning Audit

- Every reconciled Phase 8 and Sprint 8A roadmap clause maps to specifications,
  acceptance, a slice, automated/deployed proof, and manual UAT.
- UI, API, persistence, authorization, integration, deployment, operations,
  compatibility, observability, fresh reset, failure rerun, and rollback are
  covered; legacy product-data migration is explicitly out of scope.
- Happy, negative, boundary, nondisclosure, timeout, outage, destructive-target,
  no-op, failure-rerun, recovery, and rollback cases are explicit.
- Product, seed, fixture, smoke, UAT, Playwright, manifest, deployment, and
  bootstrap changes are paired in dependency-valid slices.
- Roles, topology, commands, data, evidence root, freeze boundary, receipts, and
  invalidation behavior agree with the verification record.
- The initial planning reconciliation changed documentation only. The
  subsequent implementation advanced product code, contracts, tests, fixtures,
  deployment inputs, validation runners, and implementation documentation
  through the ordered slices above. Those candidate-affecting changes remain
  mutable and formally unverified after R30.

## Implementation Handoff And Current Gaps

The current handoff is the single correction cone opened by terminal failed
Rehearsal 32. It advances every health caller to the exact Core/Supervisor
contracts, makes final canonical restoration an independent mandatory cleanup
sink, withholds prospective correction authorization until current-attempt
cleanup receipts pass, canonicalizes every new declaration path, serializes the
full immutable lane graph, and retains bounded two-wave scheduling with full
harvest as the authentication fallback. It also replaces the stale public
`placement_key` smoke assertion with exact opaque placement identity, geometry,
resolution, and nondisclosure checks, and makes Dashboard bootstrap persist the
canonical versioned placement layout. These product, harness, schema,
acceptance, validation-reference, and runner changes are candidate-affecting.
They require focused implementation verification and one clean commit before
the coordinator may consume the R32 authorization/qualification tuple in
Readiness 42. No formal cycle starts in this implementation phase.

The earlier R37/R30/R38 implementation handoff is retained below as history.

The correction following Rehearsal 29 was committed and handed back to formal
validation. Readiness 37 passed all 15 checks, but Rehearsal 30 failed after a
complete fail-late harvest: 18 lanes passed, 4 failed, and 10 were blocked by
exact failed prerequisites. Its raw evidence, one harvest, one consolidated
defect batch, and one successor-Readiness authorization are retained. Readiness
38 consumed that authorization exactly once, retained a complete 12-pass,
2-failure, 1-block result, and failed before it could issue a verified
environment contract. Formal testing then exited so the implementation audit
could correct the whole testing-entry batch. Neither Readiness 37 nor Readiness
38 can authorize a downstream phase.

The implementation lineage that produced the R37/R30 source covers:

- make `tessara-components-contract` the canonical owner of the exact render
  response DTOs and render-kind discriminant, including exact Table and Visual
  payloads; omit the unused Dataset provider identity; and reject unknown,
  malformed, mixed-kind, or identity-divergent responses;
- distinguish stored execution from unsaved authoring preview: persistent
  responses require matching non-nil Component and ComponentVersion IDs, while
  preview alone requires both IDs to be nil; partial-nil identities are invalid;
- return those typed responses from Component provider and product routes,
  and make Dashboard validate and consume the same contract types directly;
- make `tessara-dashboard-placement-renderer` depend on the Components
  contract, with no direct Dataset-contract dependency and no private copies
  of Component response DTOs;
- verify the signed body digest against the exact raw request bytes before
  deserializing Component, Core authorization-exchange, and Core Dataset
  provider requests, while retaining explicit JSON media-type and body-size
  enforcement;
- restore the Sprint 7A common-governing-node rule at the physical
  Dashboard/Component boundary: forward only actor-authorized Dashboard scope,
  bind render exchange to the exact ComponentVersion resource assertion, make
  Component compare it with its current row, and redact metadata/title before
  projection when Dashboard and Component scopes are disjoint;
- repair the Component product-integration fixture so its declared required
  fields include every field used by its Table configuration, allowing the
  intended provider-not-ready assertion without weakening the product's
  correct invalid-input rejection;
- remove active first-party Component normalization aliases and flat Dataset
  reference fields from permissions, general smoke, and general UAT fixtures;
  enforce exact v3 identities and typed Dataset references instead;
- capture each PowerShell child-script result at its invocation boundary so a
  stale `$LASTEXITCODE` cannot turn a passing boundary or Markdown audit into
  a failed receipt, while preserving independent fail-late sibling execution,
  exact blocked reasons, partial evidence, and one consolidated batch; and
- retain the earlier product correction that removes Dashboard from Core's
  canonical transition inputs: exactly five transition identities remain, and
  Dashboard appears only through its real Release/Instance and manifest at
  reference navigation order 9; and
- advance boundary, integration, acceptance, and documentation expectations
  together. The current acceptance inventory has 75 exact Playwright identities
  and UAT-8A-04 includes the shared-node success/disjoint-node nondisclosure
  scenario. These changes are candidate-affecting and remain formally
  unverified until a new complete Readiness and complete Rehearsal pass on the
  same clean source and environment identity.

The R30 correction adds three harness root causes and the directly related
evidence-enforcement gaps as one batch:

- preserve the complete bootstrap action object before dispatch so
  `set_enablement` reads its declared `enabled` field rather than a `switch`
  pipeline string; this one defect caused both materialization symptoms and the
  failure-containment cascade;
- return source identity as the exact object shape required by UAT receipt
  guards, eliminating the 12 false prerequisite failures and 8 internally
  blocked UAT scenarios without weakening their semantic predicates;
- snapshot and restore all seven bootstrap-owned process environment variables
  so same-process normalized Compose comparison is stable; the post-process
  recomputation matched R37 and proves this was harness leakage, not persistent
  environment drift;
- project a retained structured child classification before log/default
  fallbacks, record its classification source, and preserve the four raw R30
  labels while consolidating the three diagnosed root causes;
- add per-lane assertion-start markers and count only the 22 executed R30 lanes,
  not the 10 blocked terminal receipts;
- retain secret-free expected/actual fingerprint and changed-section evidence
  for environment mismatches; and
- canonicalize new evidence references to contained repository-relative paths,
  while allowing the immutable in-root R30 authorization to be consumed once
  without rewriting it;
- freeze the source-accurate downstream interfaces: a repository-owned
  ten-check preflight runner, a repository-owned four-lane `Run`/`Finalize` SIT
  runner, and the formal-UAT `Start`/manual execution lease/`Finalize` runner.
  All bind the live normalized Compose digest. Manual execution retains exact
  start/completion lease pairs and an immutable prepared-publication
  checkpoint. An interrupted lease may resume only with an authenticated
  process-lineage marker and cannot produce an authoritative pass; scripted or
  manual defects harvest safe siblings diagnostically, while product decisions
  pause. Finalize requires
  explicit disposable-reset authorization, durably checkpoints completed
  scripted/manual/restoration evidence, commits the manifest, and publishes
  the UAT JSON/sidecar pair from the exact committed bytes. An evidence-only
  failure may consume that checkpoint for publication-only retry. None may run
  until corrected successor Readiness and Rehearsal both pass.

Readiness 38 exposed the final testing-entry batch before preflight:

- the operator must export all six disposable database bindings, destructive-
  reset acknowledgement, and validation Postgres container identity in the
  same process that invokes Readiness; the missing binding remains retained as
  an `environment` failure rather than being rewritten as product evidence;
- the failure-containment self-test must reject an unauthenticated overwrite,
  preserve the fail-closed behavior for real corrupt retained evidence, remove
  only its exact deliberately corrupt pair inside the validated temporary
  self-test root, and then republish create-once;
- every failed Readiness must retain its exact terminal checks, raw evidence,
  blocked dependency reasons, one harvest, one consolidated defect batch, one
  exact-next-attempt authorization, and an append-only correction lineage, so a
  failed authorized successor cannot dead-end validation or reuse an earlier
  authorization;
- before any Readiness receipt, start snapshot, sidecar, or log directory is
  created, the exclusive validation lock must authenticate the requested
  attempt number against current state and the pending authorization; an out-
  of-sequence probe must leave every canonical namespace target absent;
- Candidate Rehearsal must retain its immutable declared graph separately from
  the exact terminal check results and checkpoint those results after every
  pass, failure, and block;
- the active Component contract must reject the retired shared
  `missing_policy` key and use only purpose-specific
  `value_missing_policy`, `category_missing_policy`,
  `comparison_missing_policy`, and `x_missing_policy` fields across validation,
  provider, browser, Playwright, seed, smoke, and UAT inputs; and
- one canonical eight-scenario manual-UAT contract must bind exact scenario,
  tester, precondition, evidence identity/cardinality, document, AC-18/AC-19,
  and semantic-predicate expectations. UAT-8A-07 additionally requires an
  authenticated receipt binding separate hashed failed-apply and service-log
  evidence. Formal UAT and preflight must fail closed on drift rather than
  accepting arbitrary per-step hashes or free-text completion claims.

Before R37/R30, the corrected product source passed the complete all-feature
workspace Rust suite against six freshly reset isolated databases in 619.6
seconds. It also passed all-target/all-feature clippy with warnings denied, all-target/
all-feature check, formatting, exact Component/Dashboard/API integration and
contract checks, boundary and acceptance contracts, runner self-tests, 75/75
acceptance inventory, TypeScript compilation, and 75-test Playwright discovery.
Documentation-only handoff edits were covered by the final Markdown-link,
boundary, acceptance-contract, formatting, and diff audits. Those results are
retained implementation history; the current runner, evidence, and
documentation correction is candidate-affecting and requires fresh focused
verification before the successor full gates.

The earlier handoff asserted the following baseline. It remains subject to the
new complete verification cycle after the current correction batch:

- Core's canonical transition inputs contain exactly `tessara.forms`,
  `tessara.workflows`, `tessara.responses`, `tessara.datasets`, and
  `tessara.migration`. Dashboard and Components each appear exactly once from
  a real enrolled Release/Instance and manifest; transition/release overlap is
  rejected instead of hidden.
- Generic signed-bootstrap receipt bindings resolve Component-owned read-back
  identities into Dashboard-owned seed input before its digest and idempotency
  key are calculated. Dashboard no longer embeds copied Component seed IDs or
  depends on an owner-specific orchestration branch.
- Dashboard exchanges its verified route grant through Core for the selected
  Component audience, and Component performs a second exchange for the
  Core-hosted Dataset provider audience. Provider and consumer service actions
  are Manifest declarations resolved through the exact applied lockfile;
  service identities are projected during enrollment/materialization and are
  never registered opportunistically on a first request.
- The non-nil V3 correlation identity is created or accepted once at the Core
  boundary, preserved unchanged through both audience exchanges, forwarded on
  each service request, and validated against the inbound grant; Dashboard and
  Component never mint replacement per-hop correlation identities.
- The Component provider preserves known-versus-random nondisclosure, and
  Dashboard dependency state presents Components as a real module dependency,
  never as a Core transition.
- Dataset catalog and schema responses preserve the accepted Component picker
  context through the canonical Dataset v1 metadata shape: Dataset name and
  major, grain, ordered tags, compact direct Form/upstream-Dataset provenance,
  and field preview. The exact required fields reject copied legacy shapes and
  remain discoverability data rather than authorization inputs.
- Core's Component-specific lifecycle adapter is removed. The shared web host
  handles module lifecycle behavior through policy-neutral module contracts,
  while extracted Component database/API integration tests exercise durable
  ownership across the real store boundary.
- Navigation is computed from Core's five exact transitions plus enrolled
  manifest contributions, with Scoped Records, Components, and Dashboard at
  orders 7, 8, and 9. Applied lockfile navigation is transactionally
  materialized before it is served. Current tests assert exact identities
  instead of copied six/seven-item counts; immutable historical fixtures
  remain explicitly historical.
- Core no longer seeds Component or Dashboard capabilities, role grants,
  service actions, product routes, product data, or product CSS. Release
  enrollment and Blueprint role projection own those inputs. Component's
  complete-document SSR, lifecycle asset, and visual-preview identity are
  module-owned, and JavaScript-disabled acceptance asserts the owner-rendered
  content rather than obsolete Core-era loading placeholders.
- Validation Readiness now proves six pairwise-distinct authenticated
  database bindings and issues one secret-free environment fingerprint. The
  repository-owned Candidate Rehearsal runner declares dependencies, runs safe
  siblings fail-late, retains failure artifacts and blocked reasons, produces
  one consolidated defect batch per diagnostic pass, and refuses correction
  authorization before harvest completion.
- Materialization records empty baseline, first apply, semantic no-op, owner
  receipts, health, teardown, and source/environment identity in append-only
  attempt directories. Failure containment uses a deterministic owner-input
  fault, proves exact teardown, and starts its successor from empty.
- The owner seed now provides exactly seven Component shells and eight
  ComponentVersions. The Stat Card predecessor is superseded/inactive and
  names its published/active successor. Dashboard owns exactly seven
  placements, including three independent predecessor-bound placements for
  Defer/Upgrade, Replace, and Remove; the blocked-scope placement remains a
  separate nondisclosure fixture.
- The non-acceptance product diagnostic performs the lifecycle actions against
  those exact identities, proves restricted nondisclosure, stops only the
  Component provider, observes the exact five remaining authorized placements
  as unavailable, restores the provider, and requires healthy zero-finding
  convergence. UAT-8A-05 can pass only from that structured semantic evidence,
  not from broad lane labels or copied counts.
- Component dependency diagnostics now report the selected binding, provider,
  contract/version, compatibility, health, observation time, and stable
  result/failure codes without raw references. Manifest schema defaults are
  explicit and its runtime/migration commands point to the executable actually
  installed in the image.
- Upgrade evidence uses a separately compiled `0.9.0` Component binary and
  release manifest, not a relabeled `1.0.0` candidate. The runner resolves and
  applies exact Component-only deltas through the Supervisor and Compose
  adapter for baseline establishment, `0.9.0` to `1.0.0` upgrade, rollback,
  and intended-`1.0.0` restoration while comparing pre-transition snapshots.
- Playwright and UAT diagnostics use attempt-, source-, environment-, and
  evidence-hash-bound receipts. Failed browser runs retain raw reports, traces,
  logs, and error context; stale prior-source UAT evidence cannot pass by file
  existence.
- The public gateway remains stopped through first materialization and the
  semantic no-op. A loopback-only Core control binding is used for owner APIs,
  and a retained boundary receipt proves the gateway starts only after every
  owner completes.
- The generic bootstrap dependency-validation path resolves Component's exact
  Manifest declaration and opaque `/dependency_validation` payload from the
  lockfile. Its short-lived one-use authorization binds the full request,
  apply, owner, dependency, contract/version, action, method/path, and resolved
  audience without a Component/Dataset branch in Core or Supervisor. The
  Dataset provider verifies the selected Component service, consumes the grant
  once, and alone interprets the payload. Component bootstrap and normal
  create/save mutations complete scope and Component-semantic compatibility
  checks before opening a write transaction; an unavailable or incompatible
  Dataset cannot leave a shell, version, receipt, or validation-pending draft.
- Configuration shape failures project as validation errors instead of
  provider outages, and Module Management retains the module-owned sanitized
  diagnostic projection. Exact browser acceptance covers configuration
  authority, schema, diagnostics, the viewport/theme matrix, and unsaved-state
  preservation with one retry mutation during Dataset outage.
- Dashboard distinguishes provider-evaluated resolution from synthetic outage
  projection. Outage presentation, observations, findings, actions, recovery,
  and repeat-outage reopening are bound to the exact unexpired semantic
  authorization context, so an outage cannot create disclosure authority and
  one actor/scope/revision context cannot read or mutate another's episode.
- Sprint 8A fixture preparation performs only Core identity/RBAC setup and
  verification outside the owner-controlled seed. An AST-enforced exclusion
  keeps Dataset, Component, and Dashboard product writes in the legacy Sprint
  7A path; Sprint 8A product state is created only through owner bootstrap.
- Automated UAT predicates evaluate their authenticated raw JSON evidence and
  reject divergence from embedded summaries. The product diagnostic is derived
  from its authenticated lane rather than accepted through an independent file
  parameter, and exact assertion identities replace broad receipt labels.

The repository-local validation protocol already stated the fail-late and
single-batch rules. Rehearsal 29 closed stale child-exit accounting. R30 proved
that the remaining enforcement gap was evidence precision: lane receipts did
not distinguish executed assertions from terminal blocks, structured child
classification could lose to defaults, mismatch evidence omitted the compared
identity sections, and new references were not uniformly canonical. The
R30 correction made those obligations executable while preserving the
existing dependency graph, raw evidence, exact blocked reasons, and one-batch
harvest contract.

The required deployable and source dependency boundaries are shown in the two
current Sprint 8A diagrams in [Tessara Architecture](../architecture.md): one
container view and one Rust module/crate view. The mutable implementation and
tracked acceptance inputs are reconciled to those contracts, but formal proof
still starts at the next complete Readiness gate.

This status authorizes implementation correction and focused diagnostic proof
only. R32's append-only post-harvest restoration qualifies, but does not
replace, its quarantined one-use correction authorization for exactly Readiness
42. After a clean correction commit and explicit validation-coordinator
authorization, the next formal boundaries are complete Validation Readiness 42
and complete Candidate Rehearsal 33 against the same corrected clean source and
environment identity. This status does not assert candidate freeze, deployed
acceptance, preflight, SIT, formal UAT, or closeout.

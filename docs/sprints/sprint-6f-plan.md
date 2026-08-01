# Sprint 6F Plan: Application Blueprint And Composition Automation

Status: approved on 2026-08-01. The core implementation slices are complete;
candidate freeze, formal validation, and closeout authorization remain pending.

- Branch: `codex/sprint-6f`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6f`
- Base commit: `57aaf1f1d37d6e240015de1282508409b4947b05`
- Roadmap authority:
  `Sprint 6F: Application Blueprint And Composition Automation Slice (Next)`
- Validation record: [Sprint 6F Verification](./sprint-6f-verification.md)
- Planned deployment profile: `deploy/sprint-6f/compose.yaml`
- Planned evidence directory: `artifacts/sprint-6f-closeout/`

## Sprint Summary

Sprint 6F makes a Tessara application a declarative and reproducible
composition. A versioned Application Blueprint will resolve against trusted,
machine-readable Core and module catalogs into one deterministic lockfile and
Materialization Plan. Applying that plan remains a separate, explicitly
authorized operation executed by the local out-of-process Supervisor.

Core UI/API, the bootstrap CLI, automation, and future agent clients will use
the same composition contract. The sprint will prove both a complete reference
application and a reduced application, bootstrap from either a Blueprint or a
detached signature over an already resolved lockfile/plan digest, reconcile
drift, survive a Core restart during apply, and make an unchanged replay a
verified no-op. Product artifacts may be bootstrapped only through typed,
module-owned APIs.

This plan is the implementation contract. Scope changes require an explicit
plan amendment.

### Approved implementation decisions

- Application Composition is a dedicated native Core administration surface
  at `/administration/composition`, separate from Module Management.
- Core adds installation-global `composition:read`, `composition:plan`, and
  `composition:approve` capabilities. Plan implies read; approve implies plan
  and read; `admin:all` implies all three. V1 separates capabilities and the
  approval action but does not require a different human approver.
- Ordinary UI changes create draft Blueprint revisions. Drift is reserved for
  observed out-of-band change and emergency overrides.
- All public composition documents are strict JSON. RFC 8785 canonical JSON
  bytes are used for digests and signatures.
- Release discovery uses purpose-bound Ed25519-signed catalog snapshots and
  Supervisor-held trust anchors. V1 external inputs are versioned environment
  secret references and SHA-256-addressed files in a configured local CAS.
- Add a host-local `tessara-supervisor` process with a protected SQLite WAL
  ledger and signed request/response envelopes over a private HTTP transport.
  Move enrollment into that boundary; retain `tessara-deploy` only as a
  compatibility wrapper while callers move, then remove it within the sprint.
- The complete composition is Core/gateway plus Dashboard and Scoped Records;
  the reduced composition is Core/gateway only. Core, Scoped Records, and
  Dashboard expose owner-specific typed bootstrap operations. The reference
  Dashboard includes exact ComponentVersion-backed placements.
- Sprint 6F starts from a fresh installation. The retained Sprint 6E Compose
  stack and database volumes are explicitly replaced after exact target
  verification; retained Sprint 6E closeout evidence is preserved. No legacy
  PostgreSQL enrollment-ledger import is in scope.

## Sprint Specifications

### Canonical composition model

- Add a canonical, independently testable composition package, planned as
  `tessara-composition`, with versioned schemas and deterministic
  serialization/digest rules shared by Core and the Supervisor bootstrap CLI.
- Model the Blueprint fields named by the roadmap: Core Release constraint;
  typed Organization configuration; module constraints and desired enablement;
  dependency bindings; module configuration; optional module-owned bootstrap
  declarations; navigation policy; Core-owned role definitions and
  role-to-capability mappings; a designated Administrator Enrollment Role; and
  versioned environment secret references. Secret values never enter a
  Blueprint, lockfile, plan, receipt, status payload, log, or evidence file.
- Define canonical normalization and ordering for every map, set, version
  constraint, binding, configuration value, role mapping, navigation item,
  bootstrap value/reference, and action. Equivalent inputs must produce
  byte-identical lockfiles, plans, and SHA-256 digests.
- Keep the Composition Engine policy-neutral: it validates, resolves, plans,
  and diffs but does not deploy, mutate databases, approve its own output, or
  infer approval from an interactive or LLM planning session.

### Catalog, schema, and dependency discovery

- Publish versioned machine-readable Core Release and Module Release catalog
  records with publisher/trust identity, exact component image digests,
  Deployment Profile versions, Feature Declarations, contribution schemas,
  configuration/bootstrap schemas, functional contracts, capability
  namespaces, semantic destinations, and compatibility constraints.
- Resolve only trusted catalog entries and immutable artifact identities.
  Reject missing releases, unsatisfied constraints, incompatible platform or
  Supervisor/deployment-adapter versions, untrusted publishers/artifacts,
  missing bindings, cycles, ambiguous providers, invalid configuration, and
  unsupported bootstrap declarations with stable error codes.
- Calculate complete dependency closure and bind each consumer requirement to
  one declared provider contract/action. Never use a database relationship,
  stored deployment URL, or module implementation dependency as a binding.
- Validate that the designated Administrator Enrollment Role exists and covers
  the selected Core Release's versioned Core Administration Capability Floor.
  Role or navigation configuration must not grant undeclared product authority.

### Lockfile, plan, and diff

- Emit an Application Lockfile containing every field required by the roadmap:
  Blueprint revision/digest; exact Core Release and gateway/component image
  digests; exact Module Releases and image digests; Deployment Profile
  versions; desired enablement; composition-engine/schema and required
  Supervisor/deployment-adapter versions; contract/configuration/bootstrap
  schema versions; dependency bindings; normalized non-secret Core/module
  configuration, navigation, and role policy plus digests; enrollment role and
  capability-floor version; bootstrap values or content-addressed references
  plus digests; secret-reference identities; and the Materialization Plan and
  digest.
- Make the Materialization Plan a deterministic, non-secret ordered action
  graph with acquisition, database/migration, configuration, bootstrap,
  health-gate, enable/disable, traffic-switch, read-back, receipt, and rollback
  anchors. It must not contain an approval token.
- Produce semantic plan and current-versus-desired diffs. Distinguish no-op,
  additive, update, disable, emergency override, destructive, drift-adopt, and
  drift-reconcile effects. Exact literal inventory counts must come from a
  shared fixture/helper when they are contractual.
- Provide versioned validate, resolve, plan, diff, lockfile read-back, and
  conformance operations through the shared library and API contracts used by
  UI, CLI, automation, and machine clients.

### Authorization envelope and Supervisor ledger

- Extend the out-of-process installation-control/deploy boundary rather than
  adding an in-Core self-update path. Core and other clients submit a frozen
  Materialization Plan plus a separate Apply Authorization Envelope to the
  local Supervisor.
- Bind each envelope to operation kind, installation, current/base receipt,
  target plan digest, monotonically ordered desired revision/apply sequence,
  nonce/idempotency key, actor/service and approver evidence, issuance/expiry,
  and explicitly authorized override or destructive effects.
- Persist Supervisor-ledger state for accepted nonces, serialization/conflict
  control, current/previous plan and receipt digests, operation status,
  rollback anchors, emergency overrides, and observed engine/adapter versions.
  Reject replay, expiry, stale base, cross-installation use, concurrent apply,
  missing approval, and approval broader or narrower than the requested action.
- Keep planning and approval as visibly and technically separate capabilities.
  A plan created by an administrator, CLI, automation client, or LLM is inert
  until a valid envelope is created by an authorized approval path.
- Permit immediate emergency disable only through a constrained,
  non-destructive, reasoned, audited, optionally expiring envelope. The
  override remains desired/actual drift until explicitly adopted or reconciled.

### Supervisor apply, recovery, and receipts

- Use the selected `tessara-oci-v1` Deployment Profile and the existing
  Supervisor/deployment-adapter boundary to acquire digest-pinned Core,
  gateway, and module images from configured trusted sources, validate
  provenance, run migrations with migration identities, inject resolved
  configuration/secrets, start runtimes with runtime identities, health-gate,
  switch traffic, and retain rollback anchors.
- Make apply resumable and idempotent from the Supervisor ledger. A Core
  restart or replacement during apply cannot own, cancel, duplicate, or lose
  the operation; Core must read back final status and receipt after recovery.
- Emit an installation receipt binding the lockfile, plan, authorization
  envelope, engine/adapter versions, exact observed artifacts, configuration
  and policy digests, bootstrap receipts, desired/observed enablement,
  provenance, verification outcomes, and rollback identity.
- Preserve local operability without a mandatory central Tessara SaaS control
  plane. External inputs are limited to configured artifact, secret, trust,
  and content-addressed sources.

### Module-owned bootstrap and product boundaries

- Define typed versioned validate/apply/read-back contracts for optional
  module bootstrap declarations. Each call is idempotent, scoped to the owning
  module, and produces an input digest and result receipt.
- Resolve a referenced bootstrap input from its configured durable
  content-addressed source and verify the locked digest before invocation.
  Normalized inline non-secret values remain allowed.
- Never create or update product artifacts through Core tables, Supervisor
  database access, a generic content-package abstraction, or another module's
  API. Product records are changed only by the owner module's public contract.
- Surface module bootstrap failures as contained, attributable apply findings
  with safe retry/read-back behavior and no secret or internal-detail leakage.

### Core composition API and application UI

- Add versioned Core APIs and a native Leptos SSR composition surface using
  the shared engine and Supervisor contracts. Keep server-managed browser
  sessions and existing route ownership; do not add HTML-string shells,
  `inner_html`, `/bridge/*`, or JavaScript controller ownership.
- Show desired and current modules/enablement separately from navigation
  visibility and authorization. Show dependency findings, resolved versions,
  capability-floor/enrollment-role validation, proposed changes, drift,
  emergency overrides, and adopt/reconcile outcomes.
- Keep Blueprint, lockfile/plan, pending or accepted approval envelope, and
  observed Supervisor ledger/receipt visually distinct. Identify the Core
  Release and gateway, engine/adapter versions, active provenance, restart or
  apply status, stale-base/conflict/replay findings, and rollback limitations.
- Require an explicit approval view for apply and destructive effects. A
  read-only planner and a non-admin operator can inspect only what their role
  and scope authorize and cannot mint an envelope or trigger apply.
- Treat UI edits to enablement, configuration, navigation, roles,
  enrollment-role designation, or declared bootstrap state as a new Blueprint
  revision or explicit drift with adopt/reconcile actions.

### Reference compositions and reproducibility

- Add one complete reference-application Blueprint and one materially reduced
  Blueprint omitting unneeded modules. Both use the same catalog and engine.
- Retain normalized Blueprint, lockfile, plan, authorization, receipt, and
  read-back fixtures for both compositions without secret values.
- Prove independent resolution and apply for both, exact reproduction from
  lockfiles plus externally supplied secrets, and stable read-back after
  restart. Prove the reduced composition does not materialize omitted modules.
- Prove both first-bootstrap paths: direct Blueprint resolution and detached
  signature verification over a pre-resolved lockfile/plan digest.

## Acceptance Criteria

1. Equivalent valid inputs resolve to byte-identical lockfiles and
   Materialization Plans with stable digests across Core and CLI entrypoints.
2. The engine rejects missing, incompatible, cyclic, untrusted, ambiguous, or
   unbound compositions, invalid configuration/bootstrap input, unsupported
   profile/contract versions, and a noncompliant enrollment role.
3. Lockfiles and receipts contain the roadmap-required exact artifact,
   configuration, policy, schema, binding, enablement, engine/adapter,
   bootstrap, provenance, and plan identities, and contain no secret values.
4. Planning cannot authorize apply. The Supervisor rejects missing, expired,
   replayed, stale-base, conflicting, cross-installation, or incorrectly scoped
   authorization envelopes with stable non-disclosing errors.
5. The Supervisor alone serializes apply, survives a Core restart, resumes or
   reads back idempotently, health-gates changes, retains rollback anchors, and
   emits a complete installation receipt.
6. Module bootstrap inputs are digest-verified and applied/read back only
   through typed owner APIs; a second identical application is a no-op.
7. The composition UI accurately separates desired, planned, approved, and
   observed state and supports plan/diff, approval, status, drift adopt, drift
   reconcile, and constrained emergency-disable workflows.
8. An unauthorized non-admin can inspect only permitted composition state and
   cannot approve, apply, expose secrets, or infer restricted module/resource
   details.
9. Complete and reduced Blueprints resolve, apply, and reproduce their exact
   intended compositions. Omitted modules remain absent in the reduced case.
10. Every clause in the roadmap exit condition has one retained automated
    proof and one retained manual proof mapped in the validation record.

## Manual Test Plan

### Blueprint resolution and planning

1. As an administrator, open Application Composition and load the complete
   reference Blueprint. Inspect validation, dependency closure, enrollment-role
   status, resolved versions, exact artifacts, plan, diff, and digest.
2. Resolve the same Blueprint through the CLI and compare the lockfile and plan
   bytes/digests with Core's result.
3. Resolve the reduced Blueprint and confirm the omitted modules and their
   resources/actions are absent while required dependencies remain bound.
4. Exercise representative invalid inputs: missing binding, dependency cycle,
   untrusted catalog entry, invalid configuration, unsupported profile, and an
   enrollment role below the Core capability floor. Confirm stable actionable
   errors and no plan or apply mutation.

### Approval and authorization

1. Generate a plan as a planner without approval authority and confirm it
   remains pending and cannot be applied.
2. Approve the complete and reduced plans separately as an authorized
   administrator. Confirm each envelope names its plan, base receipt, effects,
   expiry, nonce, and approver without exposing secrets.
3. Attempt expired, replayed, stale-base, cross-installation, conflicting, and
   under-scoped envelopes and confirm rejection without state mutation.
4. As a non-admin scoped role, inspect permitted read-only state and confirm
   plan approval, apply, restricted module metadata, and secret values remain
   unavailable.

### Apply, restart, and read-back

1. Bootstrap a fresh installation directly from the complete Blueprint using
   the local Supervisor CLI, then verify the exact Core/gateway/modules,
   configuration, navigation/roles, enrollment role, bootstrap receipts, and
   installation receipt.
2. Bootstrap a second disposable installation from a detached signature over
   the reduced composition's lockfile/plan digest and verify the reduced
   observed inventory.
3. Start an authorized change, restart Core while the Supervisor is applying,
   then confirm the Supervisor finishes exactly once and recovered Core reads
   the same final receipt.
4. Rerun each unchanged Blueprint. Confirm the Supervisor records or returns a
   no-op without migrations, product duplication, service replacement, or
   handwritten deployment/database steps.

### Drift, emergency override, and module bootstrap

1. Change module configuration or navigation through UI and verify it becomes a
   new Blueprint revision or visible drift. Exercise both adopt and reconcile.
2. Emergency-disable a module with a reason and expiry. Verify the audited
   Supervisor override, contained unavailable UI, persistent drift, expiry
   behavior, and later adopt/reconcile choice.
3. Apply one inline and one content-addressed module bootstrap declaration.
   Verify digest acquisition, owner-API invocation, receipt/read-back, and
   second-run no-op. Tamper with referenced content and confirm rejection.
4. Interrupt a bootstrap or module health gate and verify contained failure,
   safe retry/read-back, unchanged unrelated services, and usable Core.

### Responsive and failure-state UI

1. Exercise composition list/detail, plan/diff, approval, apply status,
   provenance, drift, and receipt views at 1280 px, 768 px, and 390 px.
2. Confirm keyboard navigation, focus order, accessible names, contrast, theme,
   direct SSR loads, hydration, browser history, and clean browser console.
3. Confirm unavailable Supervisor, stale catalog, invalid signature, apply
   conflict, Core restart, and restricted-user states are clear and do not leak
   secrets or internal authorization details.

## Automated Test Plan

### Targeted and contract tests

- Composition unit/property/golden tests: normalization, canonical bytes and
  digests, version constraints, dependency closure/cycles, trust, bindings,
  schema validation, role capability floor, lockfile completeness, semantic
  diff, deterministic action ordering, and secret redaction.
- Cross-entrypoint golden tests: Core API and bootstrap CLI produce identical
  lockfile/plan output from the same frozen catalog and Blueprint fixtures.
- Supervisor/ledger tests: envelope validation, monotonic revisions,
  idempotency, nonce replay, stale base, conflict serialization, restart
  recovery, health-gate failure, rollback anchor, emergency override expiry,
  receipt completeness, and no-op replay.
- Module bootstrap contract tests: inline and content-addressed input, digest
  mismatch, owner-only apply/read-back, idempotency, partial failure, retry, and
  receipt binding.
- Core API/web tests: permission gates, stable errors, desired/planned/approved/
  observed separation, drift/adopt/reconcile, secret redaction, and responsive
  SSR/hydration state.

### Required command baseline

- `cargo fmt --all -- --check`
- `cargo test -p tessara-composition` (planned new targeted engine suite)
- `cargo test -p tessara-supervisor`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo test --workspace --locked`
- `npm --prefix .\end2end test`
- `docker compose -f .\deploy\sprint-6f\compose.yaml config`
- `.\scripts\local-launch.ps1` (standard local compatibility lane)
- `.\scripts\bootstrap-sprint-6f-composition.ps1 -Composition reference`
- Repeat the preceding bootstrap command unchanged and assert a no-op.
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

The package-specific commands are targeted feedback lanes. None replaces the
full locked workspace, Playwright, deployed smoke, or UAT gates. If existing
`local-launch.ps1`, smoke, UAT, or Playwright inventories cannot express the
Sprint 6F profile, update those harnesses in the same implementation change as
the corresponding route, role, seed, service, or deployment-contract change.

## Validation And Closeout-Authorization Plan

- Follow [Sprint 6F Verification](./sprint-6f-verification.md) as the acceptance
  inventory and `tessara-sprint-validation` as the gate authority:
  preflight -> freeze clean candidate -> complete SIT -> complete UAT ->
  authorize closeout.
- Use `deploy/sprint-6f/compose.yaml` as the sprint deployment profile and
  `scripts/bootstrap-sprint-6f-composition.ps1` as the repeatable
  materialization entrypoint. Its second identical invocation must be a
  verified no-op.
- Build every sprint image with `org.opencontainers.image.revision`, exact Git
  tree, and `com.tessara.source-dirty=false` labels. Record image digests,
  embedded provenance, Compose configuration digest, migration identity,
  catalog/Blueprint/lockfile/plan/envelope identities, and acceptance-manifest
  identity for the frozen candidate.
- After the final schema change, squash sprint-owned migrations at the planned
  checkpoint. Apply each resulting Core and Supervisor/installation-control
  baseline independently to disposable empty databases and verify migration
  ledgers before candidate freeze. If no schema changes, record that the
  checkpoint is not applicable with the existing baseline identities.
- Retain final evidence under `artifacts/sprint-6f-closeout/`, including at
  minimum `source-provenance.json`, `compose-config.json`, `migration-baselines.json`,
  `catalog.json`, both Blueprints and lockfiles, both Materialization Plans,
  authorization-envelope results, Supervisor ledger/read-back, installation
  receipts, bootstrap-first/no-op/restart evidence, `smoke.json`,
  `uat-scripted.json`, `uat-manual.json`, `playwright.summary.json`, and SHA-256
  sidecars or one signed manifest covering every retained file.
- Every exit-condition clause has one automated and one manual proof in the
  verification record. Authorization-sensitive work includes a constrained
  non-admin role/scoped scenario plus negative envelope/apply coverage.
- Update smoke, scripted UAT, and Playwright assertions in the same change that
  changes routes, navigation, roles, catalogs, seeds, manifests, lifecycle
  schemas, service topology, fallback documents, bootstrap behavior, or
  composition inventory. Prefer semantic assertions to copied literal counts.
- Run deployed acceptance smoke inside SIT. No acceptance check may run for the
  first time during closeout. UAT begins only after all SIT lanes pass for one
  clean source-exact candidate.
- Any SIT or UAT failure stops the gate. Correct the cause, reconfirm or refreeze
  the candidate, restart SIT from the static/boundary lane, and rerun all SIT
  before UAT resumes. Never combine results from different candidates.
- Closeout authorization remains `Not Authorized` until all retained SIT and
  UAT evidence maps to one candidate and every roadmap clause is complete.

## Ordered Implementation Plan

Implementation is intentionally not started by this kickoff. When authorized,
execute these slices in order and update relevant harness coverage in each
slice rather than deferring it.

1. Freeze canonical reference and reduced catalog/Blueprint fixtures and add
   the composition schema, normalization, digest, stable-error, and golden-test
   foundation.
2. Implement trusted catalog discovery, schema/Feature Declaration validation,
   dependency closure, contract binding, capability-floor checks, and negative
   fixtures.
3. Implement deterministic resolution, complete lockfile, Materialization Plan,
   semantic diff, read-back, and Core/CLI cross-entrypoint equivalence tests.
4. Add the separate Apply Authorization Envelope and persistent Supervisor
   ledger rules for approval, monotonic ordering, idempotency, replay, stale
   base, conflict, emergency override, and audit.
5. Extend the `tessara-oci-v1` deployment adapter for plan execution, health
   gates, Core-restart recovery, rollback anchors, exact receipts, and no-op
   replay without an in-Core self-update path.
6. Add typed module bootstrap validate/apply/read-back, content-addressed input
   acquisition, digest enforcement, owner-only product mutation, and receipts.
7. Expose the shared versioned composition operations through Core API and the
   Supervisor/bootstrap CLI, including Blueprint and detached-signature first
   bootstrap paths.
8. Build the native composition UI for desired/current inventory, dependency
   findings, plan/diff, explicit approval, apply/read-back, provenance,
   drift/adopt/reconcile, emergency overrides, and restricted-role behavior.
9. Add the Sprint 6F Compose profile, idempotent bootstrap script, complete and
   reduced deployments, migration-squash/from-scratch checks, restart/no-op
   scenarios, and retained evidence publisher.
10. Reconcile smoke/UAT/Playwright inventories, run targeted feedback after
    each slice, complete the closeout-readiness audit, freeze a clean candidate,
    and invoke the full sprint-validation regime.

## Dependencies And Blockers

- Depends on the merged Sprint 6E baseline at
  `57aaf1f1d37d6e240015de1282508409b4947b05`, including canonical module SDK
  packages, independent Dashboard release, generic gateway/lifecycle routing,
  current deployment receipts, installation-control service, and deployment
  CLI.
- `docs/roadmap.md`, `docs/modular-application-platform.md`,
  `docs/architecture.md`, and `docs/ui-guidance.md` are architectural inputs;
  the roadmap remains scope authority if wording conflicts.
- Trusted local catalog/signature fixtures, digest-pinned local images,
  disposable PostgreSQL databases, Docker Compose, Rust/Node toolchains, and
  controllable Core/Supervisor processes are required for acceptance.
- The plan assumes the existing `tessara-deploy` and
  `tessara-installation-control` boundaries can evolve into the documented
  Supervisor/bootstrap and ledger responsibilities while a new shared
  composition package prevents Core/CLI divergence. If code inspection during
  implementation disproves that seam, pause for a plan amendment rather than
  duplicating policy or placing self-update authority in Core.
- No unresolved product decision is recorded at kickoff. Any decision that
  changes Blueprint ownership, approval authority, destructive semantics,
  emergency-override scope, or module bootstrap product behavior blocks that
  slice pending user direction; it is not classified as a test failure.

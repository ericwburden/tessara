# Phase 8 Module Extraction Playbook

Status: required planning and implementation profile for Sprint 8B, Sprint 8C,
Sprint 8D1, Sprint 8E, and any later pre-production extraction of a Core-owned
feature into an independently deployed Tessara module.

This playbook turns the Components extraction into a repeatable delivery path.
It complements the [Independent Module Pathway](./independent-module-pathway.md),
which defines the target architecture. This document defines how a sprint must
plan, implement, and prove the transition before formal validation starts.

Sprint 8A remains governed by its retained plan and evidence. Future sprints
reuse the lessons and generic platform boundaries, not Sprint 8A's lifecycle
receipts, attempt history, or sprint-specific runners.

## Desired outcome

Each extraction leaves one canonical owner and one supported execution path:

- the new module owns its product UI, API, contracts, persistence, migrations,
  configuration, diagnostics, health, routes, assets, capabilities, seed, and
  lifecycle policy;
- Core retains only policy-neutral platform behavior and removes the feature's
  transition descriptor, product storage, routes, DTOs, readers, writers,
  adapters, seed writes, grants, and definition-specific branches;
- providers and consumers communicate through exact current public contracts,
  typed references, events, or exports rather than shared implementation or
  storage;
- the reference application is rebuilt from empty through owner-controlled,
  dependency-ordered bootstrap and read-back; and
- implementation proves the known targets on a clean disposable environment
  before Readiness and Candidate Rehearsal certify the result.

The sprint is not complete merely because the new service starts. It is
complete only when the old ownership path is absent, every controlled consumer
uses the new boundary, and materialization, recovery, fixtures, and acceptance
agree with the new topology.

## Planning package

Kickoff for a Phase 8 extraction must instantiate this playbook in the sprint
plan and tracked validation contract. The plan must name:

1. the feature, Module Definition, initial Module Release, selected Module
   Instance, owned resource types, database, runtime identity, and migration
   identity;
2. the complete Core subtraction inventory: tables, migrations, routes,
   handlers, DTOs, product policy, transition catalog entry, capabilities,
   grants, seed writes, fixtures, and compatibility readers to remove;
3. every provider and consumer edge, its current coupling, its target public
   contract, typed-reference owner/type, authorization exchange, timeout,
   nondisclosure, outage, and recovery behavior; this inventory must be
   bidirectional and include forward dependencies, downstream consumers,
   reverse-usage views, operational summaries/readiness, generic resource
   resolution, browser bootstrap, and control-plane callers;
4. the manifest, configuration, diagnostics, navigation, route, asset,
   deployment, health, upgrade, and rollback contracts;
5. the fresh materialization topology, exact disposable databases and
   processes, owner bootstrap order, read-back values passed between owners,
   semantic no-op, injected failure, teardown, and canonical recovery; every
   physical product identity must come from the owning provider's signed
   read-back under a logical fixture key rather than a Blueprint-predicted ID;
6. the canonical fixture and acceptance inventory, including lifecycle,
   authorization, unavailable-provider, incompatible-provider, and negative
   old-contract cases;
7. exact implementation commands for every required proof class below; and
8. exact Readiness, Rehearsal, Preflight, SIT, and UAT lanes mapped to the same
   requirements and dependency domains; and
9. an accepted pre-extraction visual/interaction baseline and UI ownership
   inventory covering markup, SDK primitives, styles, assets, SSR/hydration,
   lifecycle behavior, navigation title/state, and responsive behavior; and
10. every cached, replicated, exported, or materialized cross-owner state
    machine, including its authoritative head/sentinel, monotonic cursor or
    revision, hidden attempt checkpoints, fixed read bound, atomic publication,
    concurrency rule, expiry/rebase behavior, and last-good recovery contract.

Use exact identities instead of copied counts unless the count is itself a
product contract. When a count is contractual, declare one canonical source of
truth and derive fixtures, smoke, and acceptance expectations from it.

## Ordered implementation slices

The plan may split or combine work for cohesion, but it must preserve this
dependency order and completion meaning.

### 1. Freeze the target contracts and subtraction inventory

- Define the sole current module contract and typed resource identities.
- Inventory all producers, consumers, storage, routes, seeds, fixtures, and
  acceptance inputs that use the old ownership shape.
- Add failing boundary tests for forbidden dependencies, old payloads,
  cross-database access, duplicate inventory/navigation, and Core residue.
- Record the exact implementation-target and validation-clause mapping.

Complete when the intended boundary and everything to remove are executable
assertions, not only prose.

### 2. Establish the independent module owner

- Create the release/instance, service, database baseline, distinct identities,
  manifest, runtime/UI providers, configuration, diagnostics, probes, routes,
  assets, capabilities, and module-owned tests.
- Reuse the canonical contract/runtime/UI/testkit packages and generic Core
  enrollment, routing, configuration, and diagnostics seams.
- Build typed SDK/Leptos views and canonical asset composition before consumer
  cutover. The SDK owns the outer document, reset, design tokens, theme, shell,
  and generic primitives; product CSS is namespace-rooted and product-only.
- Generate and source-check first-party browser assets with
  `pwsh -NoProfile -File scripts/build-module-ui-browser-assets.ps1 -Module all -Check`;
  reconcile every emitted digest with the loader, manifest, and release catalog.
- Reject any new definition-ID branch in Core or Module Management.

Complete when the module is independently buildable and conforming before
product traffic or consumers switch.

### 3. Move product behavior and cut over consumers

- Move product policy, API, UI, lifecycle, execution, and persistence to the
  module.
- Preserve the accepted information architecture and workflows while proving
  direct-document and lifecycle-navigation parity, including canvas color,
  typography, controls, title/navigation identity, responsive behavior,
  accessibility, hydration, and clean console output.
- Replace direct implementation, table, and credential access with the exact
  public boundary.
- Advance every controlled consumer, generated client, fixture, smoke input,
  and acceptance input in the same slice.
- Prove authorization, scope, nondisclosure, timeout, incompatibility, outage,
  and recovery across the real process boundary.

Complete when no first-party consumer requires the old owner or payload.

### 4. Remove Core ownership

- Delete the Core product tables and baseline entries, handlers, routes, DTOs,
  policy, execution, adapters, readers, writers, seed rows, static capabilities,
  grants, and transition descriptor.
- Enroll the real module exactly once and recalculate navigation from exact
  identities.
- Search Core and the root web application for the feature identity; every
  remaining match must be an intentional platform registration, historical
  fixture, or negative assertion.

Complete when source, schema, runtime inventory, navigation, and negative tests
all demonstrate one owner with no hidden fallback.

### 5. Rebuild the reference application from empty

- Recreate every disposable owner database from its current baseline.
- Seed only through owner bootstrap APIs in provider-to-consumer order.
- Pass typed read-back identities between owners; never copy future IDs or use
  another owner's credentials.
- Prove exact first apply, exact semantic no-op, and canonical health contracts.
- Inject a deterministic pre-write or bounded mid-apply failure, retain the
  diagnostic result, remove the exact partial topology, and prove a new
  from-empty apply restores the canonical topology.

Complete when a clean disposable environment proves materialization, no-op,
failure containment, recovery, and final health without manual repair.

### 6. Close the implementation acceptance cone

- Run module, provider/consumer, boundary, migration/seed, fixture, runner,
  smoke, browser-discovery, and UAT-predicate reproducers.
- Prove a real independently built prior-compatible release upgrade, rollback,
  and intended-current restoration without restarting unrelated owners.
- Run formatting, compilation, zero-warning Clippy, and applicable workspace
  tests.
- Produce the non-authoritative implementation-readiness result with zero
  known failures.

Complete when formal validation has no known product, harness, fixture,
acceptance-inventory, deployment, environment-contract, or evidence-contract
failure left to discover.

## Required implementation proof classes

A validation contract with
`implementation_profile.kind: phase8-module-extraction` must map required exact
commands to every class below. One target may satisfy multiple classes only
when the command actually proves each one.

| Proof class | Minimum implementation-stage evidence |
|---|---|
| `static-quality` | Format, compile, and Clippy with warnings denied for the affected Rust/TypeScript graphs. |
| `contract-boundary` | Exact current contracts, typed references, negative old shapes, authorization, and package/source boundaries. |
| `owner-product` | Module-owned API, UI, persistence, lifecycle, configuration, diagnostics, routes, assets, and health. |
| `ui-sdk-conformance` | Accepted visual/interaction baseline, typed SDK view construction, canonical CSS ownership, namespace-rooted product assets, direct/lifecycle parity, accessibility, and absence of raw module HTML/DOM construction. |
| `consumer-cutover` | Every controlled provider/consumer integration uses the new contract and real process boundary. |
| `core-subtraction` | Exact absence of old Core schema, source, routes, adapters, readers, writers, seed, and policy. |
| `inventory-navigation` | Exact transition and enrolled-module identities with no duplicate inventory or navigation presentation. |
| `migration-seed` | Fresh owner baselines, owner-only bootstrap, dependency order, typed read-back, and canonical fixtures. |
| `clean-materialization` | Source-exact first apply from an authenticated empty disposable environment and exact health contracts. |
| `semantic-noop` | Unchanged second apply produces no semantic product, configuration, receipt, or topology change. |
| `failure-recovery` | Deterministic failure containment, exact teardown, from-empty recovery, and canonical restoration. |
| `fixture-acceptance` | Fixtures, smoke inputs, browser inventory, UAT predicates, and exact acceptance identities match current behavior. |
| `runner-selftest` | Repository-owned materialization, fixture, smoke, evidence, and validation runner self-tests pass. |
| `deployed-smoke` | Focused non-authoritative smoke proves real routes, health, owner identity, and provider/consumer behavior. |
| `independent-upgrade-rollback` | Distinct source-built release upgrade, rollback, and intended-current restoration leave unrelated owners unchanged. |
| `uat-readiness` | Every planned manual scenario has executable preconditions, identities, fixtures, and passing automated predicates. |

Targets for `clean-materialization`, `semantic-noop`, and `failure-recovery`
must declare `clean_environment: true`. A missing class, placeholder command,
optional target, or unproved clean-environment class makes the contract invalid
or blocks the implementation exit gate.

## Lessons retained from Sprint 8A

Future sprints must prevent these defect families during implementation:

- inventory defects caused by treating a real deployed module as both a Core
  transition and an enrolled release;
- consumer DTO or fixture copies that drift from the canonical provider
  contract;
- seed orchestration that predicts IDs, writes another owner's tables, or
  exposes bootstrap-only identities through public APIs;
- materialization comparison code that assumes one historical receipt shape;
- health checks that accept redirects, broad HTTP success, or the wrong owner
  endpoint;
- child-process wrappers that read stale exit state instead of capturing each
  invocation result at its boundary;
- same-process environment leakage that changes later source/environment
  fingerprints;
- acceptance inventories based on copied counts, labels, or file existence
  rather than exact identities and authenticated semantic evidence; and
- formal validation launched before clean materialization, no-op, recovery,
  runner, fixture, smoke, and acceptance reproducers pass.
- UI extraction that copied a visual snapshot or rebuilt structural HTML
  instead of adopting the SDK, allowing Core and module backgrounds, titles,
  controls, and hydration behavior to drift.

These are implementation obligations. Adding more receipt history or rerunning
full certification is not an acceptable substitute.

## Lessons retained from Sprint 8B

The Dataset extraction showed that a module migration is a system-boundary
rewrite, not a crate move. Future extractions must retain the following
findings.

### Inventory the complete interaction graph

- Inventory callers in both directions. Dataset's obvious providers and
  downstream Component/Dashboard consumers were only part of the boundary;
  Core application summary, Operations readiness, Form reverse usage, generic
  resource observation, browser editor options, and control-plane bootstrap
  were also first-class consumers.
- Treat reverse and operational consumers as explicit contract rows with their
  own authorization, nondisclosure, unavailable, empty, degraded, and recovery
  semantics. A product route passing does not prove these surfaces.
- Search by tables, DTOs, route identities, typed resources, capability names,
  fixtures, browser requests, and operational vocabulary. Searching only for
  the feature's main API namespace misses hidden coupling.

### Make provider envelopes sufficient for independent ownership

- A consumer must receive every immutable fact needed to authorize, compile,
  persist, diagnose, and recompute its product without consulting the former
  owner's storage. For Dataset this included stable field identities, display
  snapshots, complete source-scope vectors, content and scope revisions and
  digests, system fields, restriction tier, row scope, and tombstones.
- Define and test canonical contract DTOs before moving handlers. Local mirror
  DTOs, untyped JSON facades, inferred defaults, and post-deserialization digest
  checks allow the new module to retain accidental knowledge of the old owner.
- Verify exact media type, body-size boundary, raw-body signature/digest, and
  correlation identity before deserialization. Then independently validate the
  typed contract and its canonical semantic digest.

### Model incremental synchronization as a publication protocol

- A cheap head or sentinel check is useful, but a timestamp such as
  `max(created_date)` is not a safe cursor. Equal timestamps, corrections,
  status changes, redactions, deletions, and tombstones require a canonical
  monotonically increasing provider change sequence.
- Separate the published committed cursor from hidden attempt/page checkpoints.
  Bind attempts to provider instance, contract, source binding, authorization
  scope, starting cursor, fixed upper bound, and page digests. Staged pages are
  never caller-visible.
- Promote imported facts, materializations, resource/authority revisions,
  receipts, and the published cursor in one transaction. Use a graph lock or
  compare-and-swap rule to reject stale concurrent promotion.
- Decide cursor expiry before implementation. A full-snapshot rebase starts
  from no page cursor, preserves the old last-good publication while staging,
  and atomically replaces the partition only after complete validation.

### Treat derived data as one dependency closure

- Refreshing only the initiating resource is insufficient when the module can
  consume itself. Plan and rebuild the complete affected dependency graph in
  deterministic topological order.
- Reject transitive cycles before synchronization or product writes. A base
  change must atomically advance every affected derived hop and downstream
  observation while leaving unrelated bindings unchanged.
- Inject failure after owner writes but before a derived rebuild completes and
  prove that imported facts, cursors, receipts, materializations, revisions,
  and downstream observations all roll back together.

### Treat browser, SSR, and operational behavior as architecture

- Freeze exact navigation identity, static-versus-parameterized route
  precedence, direct-document and lifecycle destinations, capability/action
  contracts, and module-owned browser API routes before cutover.
- Direct HTML, JavaScript-disabled SSR, lifecycle bootstrap, hydration,
  dirty-state handling, stored/system theme behavior, responsive overflow,
  accessibility, console cleanliness, and source-derived browser assets are
  required owner behavior, not late UI polish.
- Readiness and diagnostics must share one read-only owner status projection
  that reflects configuration, exact dependency bindings, compatibility,
  last-good materialization, source freshness, and sanitized stable failure
  codes. Process liveness alone is not product readiness.

### Make fixtures and upgrades owner- and semantics-exact

- Blueprints name logical fixture resources; owners allocate physical IDs and
  return signed typed read-back. Later owners consume only those authenticated
  logical-key receipts. Generic orchestration must not learn product IDs or
  branch on a module definition.
- Build the prior-compatible release independently from source. Do not relabel
  the candidate image. Upgrade, rollback, and restoration must preserve Module
  Instance identity and leave unrelated owners running and semantically exact.
- Define no-op and preservation comparisons in terms of effective state.
  Explicitly normalize only approved incidental metadata such as observation
  timestamps; retain negative tests proving that authorization,
  configuration, topology, or product drift is still rejected.

### Design executable proof with the implementation

- Create target selectors, exact test identities, zero-test guards, dependency
  domains, evidence schemas, cleanup contracts, and runner self-tests as each
  slice lands. A broad suite name or a successful process start is not proof of
  the intended behavior.
- Use real, isolated, disabled-by-default, one-shot fault controls for provider
  incompatibility and bounded mid-transaction failure. Self-tests must reject
  dual arming, tampered receipts, missing cleanup, and false live-proof claims.
- Every target receipt must bind the clean source commit/tree, validation
  contract, runner, affected dependency fingerprints, exact command, evidence
  index, and cleanup state. Finalization authenticates the complete target set
  and refuses stale, missing, dirty, or failed evidence.
- Periodically test from an empty build directory. Clean compilation and asset
  regeneration expose undeclared inputs, stale generated artifacts, missing
  feature combinations, and cache-dependent success before formal validation.

### Pre-implementation closure questions

Before the first product move, a future extraction plan must answer all of
these with executable contracts or named implementation work:

1. Who reads or writes the capability directly, indirectly, in reverse, for
   operations, through the browser, and through the control plane?
2. Does each provider response contain every fact the new owner needs without
   old storage, mutable lookup, inferred default, or copied policy?
3. What is the authoritative change identity, and how do no-op, paging,
   interruption, concurrency, expiry, rebase, and last-good publication work?
4. What complete derived-resource and downstream-consumer closure must commit
   or roll back together?
5. Which exact public, private, document, lifecycle, health, diagnostics,
   bootstrap, and resource-observation routes does the module own?
6. How are physical fixture identities allocated, signed, read back, and bound
   into later owners without predicted UUIDs or cross-owner writes?
7. Which real failure controls prove pre-write rejection, transaction rollback,
   restoration, and unrelated-owner containment?
8. Which state is semantically invariant across no-op, upgrade, rollback, and
   retry, and which narrowly defined metadata may legitimately change?
9. Which exact implementation target and validation lane proves every
   requirement, and what input change invalidates that proof?
10. Can the complete touched graph rebuild, materialize, render, recover, and
    publish evidence from clean source, clean databases, and an empty build
    cache?

An unanswered question expands the implementation scope. It is not deferred to
formal validation as an expected discovery.

## Reuse boundary

Reuse the canonical SDK/runtime/testkit, generic Core control plane, Supervisor
apply path, Blueprint/lockfile/materialization contracts, validation-policy v2
module, schema contracts, and this proof-class profile.

Do not copy Sprint 8A's attempt lineage, evidence schemas, two-wave scheduler,
or sprint-specific runner state machine into a later sprint. A future runner
should be a thin sprint profile over current shared contracts. If a capability
is genuinely reusable, extract it under a policy-neutral name with self-tests;
do not create another large `<sprint>-common` library.

Formal Readiness begins only after the current clean source has a passing
implementation-readiness result covering every required proof class. Candidate
Rehearsal then certifies the pre-freeze composition. Successor candidates still
run complete SIT and UAT.

## Technical-debt disposition

The repeatable debt paid before Sprint 8B consists of:

- canonical module contract/runtime/UI/testkit packages and boundary checks;
- one canonical module UI asset consumed by Core and every module, typed
  document/lifecycle adapters, namespace ownership checks, and focused visual
  parity required before validation;
- generic enrollment, configuration, diagnostics, navigation, routing,
  Blueprint, lockfile, Supervisor apply, bootstrap, health, and rollback paths;
- the forward-only extraction and Core-subtraction rules in this playbook;
- a machine-validated extraction profile with mandatory implementation proof
  classes and clean-environment enforcement;
- an implementation exit gate that blocks formal validation while a known
  target is missing or failing; and
- dependency-scoped pre-freeze certification plus phase-local evidence bundles
  under validation policy v2.

The following remain intentionally sprint-specific rather than technical debt:
the feature's product rules, exact provider/consumer contracts, owned schema,
canonical seed content, dependency order, accepted UI behavior, and UAT
scenarios. Each sprint must fill those values into the same playbook and
contract. Generalizing them would move product policy into the platform and
recreate the coupling that Phase 8 is removing.

---
name: tessara-implementation
description: Implement and review Tessara product code, refactors, module extractions, APIs, migrations, seeds, fixtures, tests, harnesses, and implementation documentation using the project's forward-only complexity ratchet. Use for every Tessara implementation or implementation-review task, especially when replacing transitional adapters or contracts, changing capability ownership or crate boundaries, simplifying duplicated or disorganized code, advancing current schemas, or preparing a completed change for validation.
---

# Tessara Implementation

Apply a forward-only complexity ratchet: leave the touched capability with fewer
transitional concepts, fewer parallel paths, clearer ownership, and one
canonical way to perform each operation. Complete implementation and focused
verification, then hand formal candidate validation to the existing Tessara
validation skills.

## Establish the governing contract

1. Read the user request, current sprint plan, roadmap entry, and affected code
   before choosing the implementation shape.
2. Read `docs/architecture.md`, `docs/modular-application-platform.md`, and the
   directly affected architecture contract for boundary or ownership changes.
3. Read `docs/development-workflow.md` for migrations, tests, validation, or
   closeout-facing changes. Load UI, security, lifecycle, nondisclosure,
   deployment, and provenance guidance only when the change affects those
   concerns.
4. Inspect the worktree and preserve unrelated user changes.
5. Treat an approved user decision, requirement, or sprint plan as authority
   over an older implementation description. Reconcile affected stale
   documentation in the same slice.
6. When `docs/sprints/<sprint-slug>-validation-contract.json` declares
   `policy_version: tessara-validation-v2`, read
   `../tessara-sprint-validation/references/validation-policy-v2.md` and use the
   contract's exact dependency domains and implementation targets. Sprint 8A
   and earlier remain governed by their retained sprint-specific plans and
   runners; never retrofit their evidence to v2.
7. When that contract declares
   `implementation_profile.kind: phase8-module-extraction`, read
   `docs/architecture/module-extraction-playbook.md` completely. Treat its
   ordered slices, subtraction inventory, and proof classes as governing
   implementation requirements rather than optional validation suggestions.

## Define the forward-only end state

Before editing, identify:

- the canonical capability owner and public boundary;
- the one representation and execution path that should remain;
- every transitional adapter, compatibility branch, duplicate implementation,
  obsolete fixture, and stale document in the touched dependency cone;
- the directly affected producers, consumers, tests, seeds, migrations,
  harnesses, and documentation; and
- focused proof that the resulting behavior and boundary are correct.

Map every implementation work item to the exact governing sprint-plan,
acceptance, architecture, or validation-spec clause it satisfies. Keep that
mapping in the implementation notes or sprint verification document so the
handoff can show which behavior proves each clause; a list of changed files or
test counts is not a substitute.

Under validation policy v2, the tracked validation contract is the executable
mapping. Identify changed paths and affected dependency domains before editing,
then select every required or intersecting implementation target from that
contract. An unmapped path or uncertain consumer expands the implementation
verification cone; it is not deferred to formal validation.

For a Phase 8 module extraction, maintain the plan's owner/consumer/subtraction
inventories as code moves. Do not declare the slice complete until each
required playbook proof class is bound to a required exact target and has
passing focused evidence. The module starting successfully is not sufficient:
consumer cutover, Core subtraction, inventory/navigation, fresh seed,
materialization, no-op, recovery, fixtures, runners, smoke, upgrade/rollback,
and UAT readiness are part of implementation.

Use the touched dependency cone as the cleanup boundary. Remove obsolete paths
from the changed capability and its directly affected consumers without turning
the task into unrelated repository-wide cleanup. Report related debt outside
that cone instead of silently expanding scope.

## Apply the implementation doctrine

### Break forward

- Treat Tessara as pre-production until the user explicitly states that it is
  post-production. Never infer post-production status from version numbers,
  releases, deployments, documentation, or repository state.
- Support one exact current contract and platform tuple while pre-production.
  Advance controlled producers, consumers, manifests, fixtures, digests,
  baselines, seeds, tests, and documentation together.
- Delete retired readers, writers, aliases, routes, manifests, fixtures,
  adapters, dual reads, dual writes, and fallback branches made obsolete by the
  change. Do not preserve old behavior merely to reduce the immediate edit.
- Retain explicit schema and contract version fields when they provide
  deterministic identity. Version identity does not authorize support for old
  behavior.
- Introduce or retain compatibility only when the user, an approved
  requirement, or the governing sprint plan explicitly requires it. Record the
  affected boundary, why coordinated advancement is impossible, and the
  removal condition.
- Distinguish compatibility from resilience. Preserve explicit unavailable,
  degraded, recovery, and fail-closed behavior required by the architecture;
  do not hide failure by falling back to an obsolete implementation.
- If the user explicitly declares Tessara post-production, stop applying the
  pre-production compatibility and migration assumptions and follow the
  production policy supplied or approved by the user.

### Keep one canonical owner

- Place behavior with the capability that owns its policy and lifecycle.
- Keep platform contract, runtime, UI, and testkit packages policy-neutral.
  Keep product UI, API, domain rules, configuration semantics, persistence,
  migrations, product diagnostics, and tests with the functional owner.
- Prevent separately deployable modules from depending on Core application or
  another module's product implementation. Use explicit contracts and the
  canonical SDK/runtime boundaries.
- Consolidate genuinely shared, policy-neutral behavior under one owner. Do
  not copy implementations or create module-definition-specific branches in
  generic platform code.

### Organize for cohesion and simplicity

- Structure code by capability and responsibility, keep public surfaces narrow,
  and keep implementation details private to their owner.
- Extract a module or crate only when it establishes meaningful ownership or an
  auditable dependency boundary. Do not split code into forwarding layers that
  distribute complexity without removing it.
- Avoid generic `common`, `shared`, or `utils` dumping grounds. Name shared code
  for the responsibility it owns and keep product policy out of generic layers.
- Avoid speculative abstractions, placeholder modules, empty scaffolding, and
  parallel representations. Require a present use and a clear owner.
- Consolidate duplication before extending it. Prefer deleting branches,
  indirection, and obsolete concepts over appending another mode or flag.

### Advance data from a fresh baseline

- Maintain one squashed baseline migration for each current database owner and
  update its fresh seed in the same change.
- Do not retain incremental migrations solely to upgrade obsolete
  pre-production states.
- Recreate disposable databases and prove fresh initialization after changing
  a baseline, migration ledger, bootstrap, or seed contract.

### Preserve test and source integrity

- Treat tests as durable executable contracts. Investigate a failure before
  changing its expectation.
- Never delete, skip, ignore, loosen, add retries or timeouts to, or rewrite a
  test merely to make implementation pass.
- Change an expectation only for an approved behavior or contract decision;
  document why the old assertion is superseded and retain equal-or-stronger
  coverage.
- Update focused tests, fixtures, harnesses, and affected documentation in the
  same implementation slice as the behavior.
- Deliver a behavior change together with every directly affected
  materialization, semantic no-op, rollback/recovery, fixture, runner, smoke,
  acceptance-contract, and evidence-schema change. Do not leave validation
  consumers to discover an already-known producer/contract mismatch during a
  full candidate run.
- Treat formal validation as certification of a completed implementation, not
  as the ordinary debugging loop. Reproduce and resolve every known failure in
  the implementation phase. Do not launch Readiness or Rehearsal merely to find
  out whether a known target now passes.
- Require formatting, compilation, and Clippy with warnings denied. Do not add
  blanket warning allowlists or suppressions to defer cleanup.

## Audit the completed slice

Before handoff, answer from the diff and repository rather than intention:

- Does one canonical implementation remain in the touched cone?
- Did every controlled caller and fixture advance to the current contract?
- Did the change remove, rather than extend, the relevant transition?
- Does each responsibility live with its capability owner?
- Did any new abstraction, feature flag, adapter, fallback, or compatibility
  path create a second way to do the same thing?
- Were obsolete code, tests, fixtures, seeds, migrations, and documentation
  deleted or reconciled?
- Are required resilience and fail-closed states still explicit?
- Are tests at least as strong, and are warnings still denied?
- Does the implementation-to-validation-clause mapping have passing focused
  proof for every affected clause?

Resolve findings inside the touched cone before declaring implementation
complete.

## Verify and hand off

1. Run the narrowest relevant format check, compile, Clippy with `-D warnings`,
   and focused tests during implementation.
2. When migrations, seeds, bootstrap, deployment inputs, materialization, or
   owner health changed, complete a source-exact materialization from a clean
   disposable environment and its exact semantic no-op/idempotence pass before
   declaring implementation complete. When rollback, failure containment, or
   recovery changed, also prove focused recovery to the canonical topology.
   Retain the resulting evidence as non-authoritative implementation
   diagnostics; it does not replace Validation Readiness or Candidate
   Rehearsal.
3. Run every focused reproducer for the known product, harness, fixture,
   runner, smoke, acceptance, and evidence-contract regressions in the touched
   cone. If the same formal validation lane has failed twice consecutively, do
   not launch it again until its clean focused reproducer passes. After three
   consecutive failures, treat the lane as a concentrated validation-platform
   incident and resolve its root cause before another full launch.
4. Run applicable repository boundary checks such as
   `scripts/check-web-crate-boundaries.ps1`,
   `scripts/verify-module-sdk-boundaries.ps1`, or
   `scripts/verify-module-sdk-compatibility.ps1` when their contracts are
   affected.
5. Run `git diff --check` and inspect `git status --short`. Identify preserved
   unrelated user changes explicitly.
6. Run broader repository checks in proportion to the change and the sprint
   plan. Do not claim checks that were skipped or silently filtered.
7. Under validation policy v2, write a compact non-authoritative
   `implementation-readiness-result.json` under the ignored sprint evidence
   root. Validate it against the tracked contract and
   `implementation-readiness.schema.json` with
   `scripts/tessara-validation-policy.psm1`. It must bind the clean source and
   contract hash, enumerate every selected exact target, retain clean-
   environment proof where required, report zero known failures, and record
   required materialization/no-op/recovery and restoration results. A missing,
   blocked, or failing selected target forbids formal Readiness entry.
   For `phase8-module-extraction`, also verify that every playbook proof class
   is represented by a required passing target and that the three clean-
   environment classes were actually executed from the declared disposable
   environment.
8. Hand the clean implementation commit and passing implementation-readiness
   result to `tessara-sprint-validation` when formal sprint validation is
   requested. Let `tessara-sprint-validation`,
   `tessara-sit`, `tessara-uat`, and `tessara-sprint-closeout` retain authority
   over candidate freeze, SIT, UAT, evidence, and closeout.

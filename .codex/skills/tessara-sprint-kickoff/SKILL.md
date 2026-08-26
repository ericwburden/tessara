---
name: tessara-sprint-kickoff
description: Start and comprehensively plan a Tessara sprint by validating a clean main checkout, selecting the roadmap sprint marked Next, creating the sprint branch and worktree, producing an implementation-ready sprint plan and seeded validation record, and recording the kickoff without beginning implementation. Use when Tessara sprint planning or sprint branch setup should culminate in an approved execution contract and planning handoff.
---

# Tessara Sprint Kickoff

Start a sprint from the roadmap and finish with a complete, reviewable planning
package. Do not implement product code, tests, migrations, deployment changes,
or harness changes during kickoff.

## Core behavior

- Start from a clean `main` checkout.
- Select the roadmap sprint marked `(Next)` unless the user overrides it.
- Create a separate sprint branch and sibling worktree from `main`.
- Perform every planning write in the sprint worktree so `main` remains clean.
- Turn every roadmap requirement into bounded scope, acceptance criteria,
  implementation slices, and verification coverage.
- Create the `tessara-sprint-validation` record as a planned acceptance
  inventory; do not execute validation gates.
- Create the tracked machine-readable sprint validation contract and schema-v2
  `tessara.validation.adapter`. New kickoffs select contract schema 3 and
  `policy_version: tessara-validation-v3`; retained contracts continue under
  the version they declare and are never retrofitted.
- Read `tessara-sprint-validation/references/validation-policy-v3.md` after its
  v2 base before writing a new kickoff package.
- Prepend a kickoff entry to `docs/progress-report.md`.
- Audit the planning package for completeness and stop at the implementation
  handoff boundary.

## Preconditions

Confirm all of the following before creating sprint artifacts:

- the repository is Tessara
- the current branch is `main`
- `git status --porcelain` is empty
- `docs/roadmap.md` exists
- `docs/progress-report.md` exists
- `scripts/local-launch.ps1` exists
- `scripts/smoke.ps1` exists
- `scripts/uat-sprint.ps1` exists
- `scripts/tessara-validation-policy.psm1` and
  `scripts/test-tessara-validation-policy.ps1` exist and the policy self-test
  passes

If any precondition fails, stop and explain the corrective action. Do not
create a sprint branch, worktree, or plan from a non-`main` checkout.

## Sprint selection

- Default to the roadmap sprint heading marked `(Next)`.
- Stop if there is no `(Next)` sprint or more than one.
- Use the complete sprint heading block as the scope authority, especially:
  `Outcome`, `Build`, `Application UI delivered this sprint`, and
  `User-testable exit condition`.
- Record ambiguities, contradictions, and missing decisions. Resolve them from
  repository evidence when possible; otherwise mark them as planning blockers
  and request user direction instead of inventing scope.

## Artifact naming

Derive a label-only slug from the sprint label before the colon.

Example: `Sprint 2A: Workflow Assignment And Response Start (Next)` produces:

- label: `Sprint 2A`
- slug: `sprint-2a`
- branch: `codex/sprint-2a`
- sibling worktree: `D:\Projects\tessara-sprint-2a`
- plan in the sprint worktree: `docs/sprints/sprint-2a-plan.md`
- validation record in the sprint worktree:
  `docs/sprints/sprint-2a-verification.md`
- validation contract in the sprint worktree:
  `docs/sprints/sprint-2a-validation-contract.json`
- validation adapter in the sprint worktree:
  `docs/sprints/sprint-2a-validation-adapter.json`

Abort if the branch, worktree path, plan, validation record, or validation
contract already exists, unless the user explicitly asks to resume or revise an
existing kickoff. When resuming, preserve useful content and reconcile it with
the current roadmap.

## Required execution order

1. Confirm repository and checkout preconditions.
2. Parse `docs/roadmap.md` and select the sprint.
3. Derive and conflict-check all artifact paths.
4. Create the sprint branch from `main` in a separate worktree.
5. Make the sprint worktree the working directory for all remaining steps;
   leave the `main` checkout untouched.
6. Inspect the roadmap block and the affected code, tests, architecture,
   deployment, and prior sprint artifacts in planning mode only.
   If the sprint extracts a Core-owned feature into an independently deployed
   module, read `docs/architecture/module-extraction-playbook.md` completely
   and use its planning package and ordered slices.
7. Write `docs/sprints/<slug>-plan.md` as the execution contract.
8. Use `tessara-sprint-validation` and its record template to create
   `docs/sprints/<slug>-verification.md` as a planned acceptance inventory.
9. Create `docs/sprints/<slug>-validation-contract.json` and
   `docs/sprints/<slug>-validation-adapter.json`, starting from both assets in
   `tessara-sprint-validation/assets/`. Select schema 3 / policy v3, record the
   canonical adapter path and supported platform release, validate the contract
   with `scripts/tessara-validation-policy.psm1`, validate
   the adapter through public `Assert-TessaraValidationAdapter`, then run
   `Assert-TessaraFutureSprintPlanningPackage`. Ensure every placeholder is
   replaced and every requirement,
   target, lane, prerequisite, dependency domain, environment section, and
   evidence policy is complete. Select `implementation_profile.kind` as
   `phase8-module-extraction` for a Phase 8 extraction, bind the canonical
   playbook and exact module/transition identities, and map required exact
   commands to every playbook proof class.
10. Declare explicit producer-to-projection fanout edges and each slice's exact
    focused exit targets. For Phase 8, also create the actor/action/route/
    capability authorization matrix and bind its early boundary target plus the
    independent UI ownership gate.
    For every v3 sprint, select the shared implementation coordinator, declare
    dependency-complete target prerequisites, and give each target an exclusive
    evidence-path claim plus every process, topology, port, database, Docker,
    or external-service claim it can touch. Read
    `tessara-sprint-validation/references/implementation-target-coordinator.md`.
11. Prepend the kickoff entry to `docs/progress-report.md`.
12. Run the comprehensive planning audit below and correct planning gaps.
13. Present the plan, unresolved decisions, and recommended first
   implementation slice, then stop. Do not begin implementation.

## Comprehensive sprint plan

Write the plan in Markdown with these sections:

- sprint summary, outcome, and roadmap authority
- in-scope and explicitly out-of-scope behavior
- current-state findings and affected components
- functional, UI, authorization, data, lifecycle, deployment, compatibility,
  observability, and rollback specifications, retaining only relevant domains
- assumptions, decisions, open questions, dependencies, and blockers
- traceability matrix mapping every roadmap clause to specifications,
  acceptance criteria, implementation slices, automated checks, and manual UAT
- acceptance criteria with observable pass conditions and negative cases
- ordered implementation slices with prerequisites, expected file/component
  touchpoints, exact focused exit targets, controlled-artifact fanout edges,
  tests changed in the same slice, and receipt-bound completion criteria
- automated, integration, deployed-smoke, and manual UAT plans
- validation, evidence, candidate-freeze, failure-restart, and closeout-
  authorization plan
- rollout, migration, compatibility, recovery, and rollback plan where relevant
- risks with prevention, detection, and recovery measures

For a Phase 8 extraction, also include the playbook's complete Core subtraction
inventory, provider/consumer edge inventory, target ownership table, fresh
materialization/seed graph, canonical fixture inventory, and proof-class-to-
command matrix. Include an accepted pre-extraction visual and interaction
baseline plus a UI ownership inventory for markup, SDK primitives, styles,
assets, SSR/hydration, lifecycle behavior, navigation title/state, and
responsive behavior. Map a focused visual reproducer and
`ui-sdk-conformance` target before consumer cutover. Add the complete eight-
boundary authorization matrix with exact positive, negative, replay,
nondisclosure, revision, and outage cases; exercise it through the real process/
gateway boundary before browser or smoke. Require standalone-module UI proof
for CSS and shared-shell ownership, generated assets/digests, responsive and
navigation parity, route identity, bootstrap media types, SSR, hydration,
accessibility, and a clean console. Reusing the architecture
without these delivery details is not an implementation-ready plan.

Use repository evidence to make the plan concrete, but do not make speculative
code edits. Keep scope bounded by the roadmap. A slice must produce a coherent,
testable increment and identify its required harness updates; avoid a task list
that merely names files or architectural layers.

The tracked validation contract is the executable companion to this prose. It
maps every clause to exact implementation targets and formal lanes, maps every
target/lane to dependency domains, and maps each domain to tracked path
patterns. Unknown paths must select conservative validation rather than being
silently ignored.

For policy v3, it also owns one explicit producer-to-projection edge for every
controlled artifact relationship (migrations/checksums, contracts/generated
clients, manifests/catalogs, browser assets/digests, baselines, fixtures,
acceptance inventories, and evidence schemas), owner-read fixture rules, visual
stability declarations, and per-slice exit targets. Unknown fanout expands the
verification cone. Predicted identities, copied inventories, historical demo
counts, reduced DTO replicas, and cross-owner fixture writes are prohibited.

## Validation and closeout readiness

Seed the validation record before implementation with:

- every roadmap exit-condition clause
- one automated assertion and one manual UAT scenario per clause
- product, authorization, lifecycle, deployment, compatibility, migration,
  observability, recovery, and rollback risks that apply
- required commands, environments, roles/accounts, fixtures, and evidence paths
- changed integration contracts and the smoke assertions that will prove them
- the intended deployment profile or Compose file and bootstrap/materialization
  command, including idempotent second-run proof
- source provenance, candidate identity, migration-baseline, and evidence rules
- the passing non-authoritative implementation-readiness result required before
  formal Readiness entry
- the mandatory defect-provenance gate for every failed target, lane, or
  scenario, including its schema-valid record, automated routing, focused-
  reproducer block, expectation-change authority, and Preflight/Closeout audit
- compact phase certificates, phase-local evidence indexes, and the final
  `evidence-chain.json` integrity audit
- the rule that deployed acceptance smoke runs inside SIT
- the rule that a candidate or harness change invalidates downstream evidence
- the shared validation-protocol invalidation matrix, including complete SIT
  restart for a successor candidate, authenticated affected-lane pre-freeze
  recertification, and certificate reuse only when declared dependency
  fingerprints prove earlier results unaffected

Plan `defect-provenance.json` beside every possible failed attempt. State that
automation may classify, invalidate, route, and block reruns, but may not edit
tests, change expected values, or decide that an assertion is obsolete.

Plan updates to smoke, UAT, Playwright, fixtures, manifests, and deployment
bootstrap in the same implementation slice as the behavior that makes them
stale. Prefer semantic assertions over duplicated literal inventory counts;
when an exact count is contractual, identify one shared source of truth.

Do not record a validation result, freeze a candidate, launch the stack, or run
preflight/SIT/UAT during kickoff. Commands belong in the plan as future
execution steps. Plan the receipt chain produced by
`tessara-validation-preflight`, `tessara-sit`, `tessara-uat`, and the
`tessara-sprint-validation` coordinator.

## Planning audit

Before declaring kickoff complete, verify that:

- every roadmap scope and exit-condition clause has end-to-end traceability
- the machine-readable validation contract agrees with the plan and record,
  validates against `validation-contract-v3.schema.json`, and contains no
  unmapped requirement, target, lane, dependency, or tracked path category
- the tracked schema-v2 adapter passes `Assert-TessaraValidationAdapter`, names
  exactly the contract lanes and prerequisites, maps every required target with
  no extra action target, and the public planning-package audit passes
- the contract selects `Invoke-TessaraImplementationHarvest`,
  `evidentiary-priority-v1`, and `serial-resource-safe`; target prerequisites
  agree with implementation lanes, and live-state topology/ports have matching
  exclusive target claims
- every slice declares its exact focused exit targets and fanout edges; every
  canonical producer has explicit projections and a reconciliation target
- fixture identities are owner-read under logical keys with signed read-back,
  and mutable visual content uses stable regions plus semantic assertions
- a Phase 8 extraction selects `phase8-module-extraction`, instantiates the
  canonical playbook, and covers every required proof class with a required
  exact implementation target; clean materialization, semantic no-op, and
  failure recovery use clean-environment targets
- UI, API, persistence, authorization, integration, deployment, and operational
  impacts were considered and irrelevant domains were explicitly dismissed
- happy paths, negative paths, boundary cases, nondisclosure, recovery, and
  rollback are covered where applicable
- implementation slices have a dependency-valid order and testable boundaries
- browser, smoke, and consumer-cutover targets depend on passing early
  authorization and independent UI ownership targets where Phase 8 applies
- required harness and fixture changes are paired with their product slices
- every extracted UI has an accepted baseline, canonical SDK ownership map,
  namespaced product-style inventory, direct/lifecycle parity target, and
  mandatory passing `ui-sdk-conformance` implementation target
- acceptance commands, roles, environments, data, and evidence destinations
  are concrete
- assumptions and unresolved decisions are visible and no blocker is hidden
- the plan traces the defect-provenance gate through implementation exit,
  every formal phase failure, Preflight rejection, and Closeout audit
- the plan and validation record agree
- the `main` checkout remains clean and all planning changes are confined to
  the sprint worktree
- no implementation files were changed

If a blocker prevents a reliable implementation contract, leave kickoff in a
blocked-planning state and ask for the decision. Branch and planning artifacts
may remain, but do not characterize the sprint as ready for implementation.

## Kickoff progress entry

Prepend a short entry containing:

- date, sprint name, and kickoff/planning status
- branch and worktree paths
- plan and validation-record paths
- planned verification commands
- unresolved decisions or blockers
- recommended first implementation slice
- explicit statement that implementation has not started

## Verification command baseline

Include at least these future commands when relevant:

- `cargo fmt --all -- --check`
- `cargo test --workspace --locked`
- `npm --prefix .\end2end test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Add narrower sprint-specific checks. If a baseline command is inapplicable,
keep it in the plan and mark it deferred, blocked, or replaced with a reason.

## Implementation boundary

Kickoff authorizes planning writes only: the sprint plan, validation record,
and kickoff progress entry. Branch/worktree creation is setup, not
implementation. Do not modify product source, tests, migrations, fixtures,
scripts, manifests, deployment configuration, or generated product assets.

After presenting the planning package, wait for an explicit implementation
request. Do not treat a general request to "kick off" or "start" a sprint as
authorization to execute the first implementation slice.

## Finish criteria

Do not report kickoff complete unless:

- kickoff started from clean `main`
- the sprint came from the roadmap or an explicit override
- the sprint branch and worktree were created
- the comprehensive plan was written and passed the planning audit
- the validation record was created and seeded from the roadmap
- the policy-v3 validation contract and tracked schema-v2 adapter were created,
  validated through the public APIs, and passed the planning-package audit
- the shared coordinator activation, deterministic prerequisites, and resource
  claims can produce one immutable execution plan
- the kickoff progress entry was prepended
- blockers and decisions were surfaced
- `main` remained clean and the sprint worktree contains only planning changes
- no implementation changes were made
- the handoff explicitly states that implementation awaits separate approval

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
- Create the tracked machine-readable sprint validation contract and select
  `policy_version: tessara-validation-v2`. This policy first applies after
  Sprint 8A; never retrofit an older sprint's receipts.
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
9. Create `docs/sprints/<slug>-validation-contract.json`, validate it with
   `scripts/tessara-validation-policy.psm1`, starting from
   `tessara-sprint-validation/assets/sprint-validation-contract.json`, and
   ensure every placeholder is replaced and every requirement,
   target, lane, prerequisite, dependency domain, environment section, and
   evidence policy is complete. Select `implementation_profile.kind` as
   `phase8-module-extraction` for a Phase 8 extraction, bind the canonical
   playbook and exact module/transition identities, and map required exact
   commands to every playbook proof class.
10. Prepend the kickoff entry to `docs/progress-report.md`.
11. Run the comprehensive planning audit below and correct planning gaps.
12. Present the plan, unresolved decisions, and recommended first
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
  touchpoints, tests changed in the same slice, and slice completion criteria
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
`ui-sdk-conformance` target before consumer cutover. Reusing the architecture
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
- compact phase certificates, phase-local evidence indexes, and the final
  `evidence-chain.json` integrity audit
- the rule that deployed acceptance smoke runs inside SIT
- the rule that a candidate or harness change invalidates downstream evidence
- the shared validation-protocol invalidation matrix, including complete SIT
  restart for a successor candidate, authenticated affected-lane pre-freeze
  recertification, and certificate reuse only when declared dependency
  fingerprints prove earlier results unaffected

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
  validates against `validation-contract.schema.json`, and contains no
  unmapped requirement, target, lane, dependency, or tracked path category
- a Phase 8 extraction selects `phase8-module-extraction`, instantiates the
  canonical playbook, and covers every required proof class with a required
  exact implementation target; clean materialization, semantic no-op, and
  failure recovery use clean-environment targets
- UI, API, persistence, authorization, integration, deployment, and operational
  impacts were considered and irrelevant domains were explicitly dismissed
- happy paths, negative paths, boundary cases, nondisclosure, recovery, and
  rollback are covered where applicable
- implementation slices have a dependency-valid order and testable boundaries
- required harness and fixture changes are paired with their product slices
- every extracted UI has an accepted baseline, canonical SDK ownership map,
  namespaced product-style inventory, direct/lifecycle parity target, and
  mandatory passing `ui-sdk-conformance` implementation target
- acceptance commands, roles, environments, data, and evidence destinations
  are concrete
- assumptions and unresolved decisions are visible and no blocker is hidden
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
- the policy-v2 validation contract was created and validated
- the kickoff progress entry was prepended
- blockers and decisions were surfaced
- `main` remained clean and the sprint worktree contains only planning changes
- no implementation changes were made
- the handoff explicitly states that implementation awaits separate approval

---
name: tessara-sprint-kickoff
description: Start a Tessara sprint in a repeatable way by validating a clean main checkout, selecting the roadmap sprint marked Next, creating the sprint branch and worktree, writing the sprint kickoff plan, prepending a kickoff progress entry, and then beginning implementation. Use when Tessara sprint planning or sprint branch setup should be turned into an execution-ready kickoff workflow.
---

# Tessara Sprint Kickoff

Use this skill when starting a new Tessara sprint.

## Core behavior

Treat the roadmap as authoritative and make kickoff execution-ready.

- start from a clean `main` checkout
- derive the sprint from the roadmap marker `(Next)` unless the user explicitly overrides it
- create a separate sprint worktree from `main`
- write an implementation-ready sprint plan artifact under `docs/sprints/`
- instantiate the `tessara-sprint-validation` acceptance inventory before implementation begins
- prepend a kickoff entry to `docs/progress-report.md`
- continue into implementation from the sprint worktree

## Preconditions

Confirm all of the following before making sprint artifacts:

- current repository is Tessara
- current branch is `main`
- `git status --porcelain` is empty
- `docs/roadmap.md` exists
- `docs/progress-report.md` exists
- `scripts/local-launch.ps1` exists
- `scripts/smoke.ps1` exists
- `scripts/uat-sprint.ps1` exists

If any precondition fails, stop and explain the corrective action. Do not create a sprint branch, worktree, or plan file from a non-`main` checkout.

## Sprint selection

- Default to the roadmap sprint heading marked `(Next)`.
- If there is no `(Next)` sprint, stop and report that the roadmap is not ready for kickoff.
- If multiple sprint headings are marked `(Next)`, stop and report the ambiguity.
- Use the sprint heading block as the planning source:
  - `Outcome`
  - `Build`
  - `Application UI delivered this sprint`
  - `User-testable exit condition`

## Artifact naming

Derive a label-only slug from the sprint label before the colon.

Examples:

- `Sprint 2A: Workflow Assignment And Response Start (Next)` -> label `Sprint 2A`
- slug `sprint-2a`
- branch `codex/sprint-2a`
- worktree sibling `D:\Projects\tessara-sprint-2a`
- plan file `D:\Projects\tessara\docs\sprints\sprint-2a-plan.md`

Keep the worktree as a sibling of the current repo directory. Abort if the branch already exists, the worktree path already exists, or the plan file already exists.

## Required execution order

1. Confirm the repo and checkout preconditions.
2. Parse `docs/roadmap.md` and select the sprint target.
3. Derive the branch name, worktree path, and plan file path.
4. Abort on any existing conflicting artifact.
5. Create the sprint branch from `main` in a separate worktree.
6. Review the sprint roadmap block in planning mode before coding.
7. Write `docs/sprints/<slug>-plan.md`.
8. Prepend a kickoff entry to `docs/progress-report.md`.
9. Use `tessara-sprint-validation` and its record template to establish the
   sprint's preflight, SIT, deployed-smoke, UAT, evidence, and closeout-
   authorization contract.
10. Begin implementation in the new sprint worktree using the written plan as the execution contract.

## Sprint plan file requirements

Write the plan file in Markdown and include these sections:

- sprint summary
- sprint specifications
- acceptance criteria
- manual test plan
- automated test plan
- validation and closeout-authorization plan
- ordered implementation plan
- dependencies and blockers

Use the roadmap text directly as the source of scope. Do not invent extra roadmap scope that is not implied by the selected sprint block.

## Closeout-readiness plan

Prevent closeout from becoming a late integration-hardening pass. Record these
items in the sprint plan before coding:

- the deployment profile or Compose file used by the sprint
- the repeatable, idempotent bootstrap/materialization command
- how build images record the exact source commit, tree, and dirty state
- the migration-squash checkpoint and disposable from-scratch verification
- the final evidence directory and expected smoke, UAT, and Playwright files
- one manual and one automated proof for every roadmap exit-condition clause
- which existing smoke/UAT/Playwright assertions must change as the feature
  inventory, navigation, roles, routes, or seed data changes
- the non-admin role/scope scenario required for authorization-sensitive work
- the rule that deployed acceptance smoke runs inside SIT and no acceptance
  check may execute for the first time during closeout
- the rule that any SIT or UAT failure is corrected and followed by a full SIT
  restart before UAT resumes

Prefer semantic assertions over duplicated literal inventory counts. When an
exact count is itself a contract, keep its expected source in one shared
fixture or helper.

## Implementation cadence

Maintain closeout readiness throughout the sprint:

1. Update the relevant smoke, UAT, Playwright, and deployment-bootstrap
   coverage in the same change that alters a route, capability, navigation
   contribution, seed, manifest, or deployment topology.
2. Run the narrowest affected tests after each implementation slice.
3. Exercise bootstrap/materialization against the sprint stack before UI
   acceptance begins, and prove a second invocation is a no-op.
4. After the final schema change, squash migrations and apply each baseline to
   disposable empty databases before the final source-exact cycle.
5. Before declaring implementation complete, run a closeout-readiness audit:
   no acceptance-mapping gaps, no stale harness inventory, valid Compose
   configuration, working provenance inputs, idempotent bootstrap, and clean
   targeted tests.
6. Commit implementation, migrations, bootstrap, and harness corrections,
   then invoke `tessara-sprint-validation` against the clean source-exact
   candidate before requesting closeout.

## Kickoff progress entry requirements

Prepend a new progress report entry with:

- date
- sprint name
- kickoff status
- branch and worktree paths
- plan file path
- planned verification commands
- immediate implementation focus

Keep the entry short and execution-oriented. This is a kickoff record, not a closeout.

## Verification plan baseline

Always include at least these planned verification commands in the kickoff artifact when they remain relevant to the sprint:

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `npm --prefix .\end2end test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

If a command appears too broad for the sprint, keep it in the plan and mark it as targeted, deferred, or blocked with a reason. Do not silently drop it.

## Finish criteria

Do not consider kickoff complete if:

- kickoff did not start from clean `main`
- the sprint target was not derived from the roadmap or an explicit user override
- the sprint branch/worktree was not created
- the sprint plan file was not written
- the kickoff progress entry was not prepended
- the closeout-readiness plan was not recorded
- the sprint validation record was not created and seeded from the roadmap
- implementation did not continue from the sprint worktree

---
name: tessara-sprint-closeout
description: Execute and document a Tessara sprint closeout as a repeatable handoff process, including roadmap and progress updates, mandatory environment and test validation, and reviewer-ready demo instructions. Use when a Tessara sprint is ending or when a user asks to produce the sprint handoff and closeout package.
---

# Tessara Sprint Closeout

Use this skill whenever a Tessara sprint is ending or when asked to produce the next-sprint handoff package.

## Core behavior

Create a closeout that is directly testable by a non-developer:

- audit closeout readiness before any destructive reset or full-suite run
- use targeted checks to resolve gaps before the source-exact cycle
- budget one full source-exact acceptance cycle
- update roadmap and progress-report completion only after evidence passes
- run required verification commands
- produce a structured handoff section with role-based demonstration steps by functionality
- map each sprint exit condition to both manual and automated/scripted evidence

When a kickoff plan exists under `docs/sprints/`, use it as supporting scope input. Treat `docs/roadmap.md` as authoritative if the plan and roadmap drift.

## Inputs

- sprint name (example: `Sprint 2A`)
- sprint status date (`YYYY-MM-DD`)
- evidence set (test outputs, smoke/UAT results, screenshots/transcripts)
- optional kickoff plan path under `docs/sprints/`
- optional next-sprint target if not already inferred

## Required execution order

1. Confirm the sprint completion target and derive a clause-level acceptance
   checklist from the roadmap.
2. Run the closeout-readiness audit below. Resolve and commit every gap with
   targeted checks before any destructive reset or full-suite run.
3. If the sprint changed the database schema, finalize migrations:
   - squash development migrations into the repository baseline
   - apply every baseline to disposable empty databases
   - verify the expected ledger contents
   - commit the finalized migration state
4. Confirm the sprint deployment bootstrap/materialization command works on
   the sprint stack and a second invocation is an idempotent no-op.
5. Commit all implementation, migration, bootstrap, smoke, UAT, and Playwright
   corrections. Record this clean commit as the evidence source commit.
6. Run the one fresh environment bootstrap and source-provenance build:
   - `.\scripts\local-launch.ps1 -FreshData` when the root profile applies
   - when the sprint has a dedicated deployment profile, use its documented
     destructive reset, build, launch, and bootstrap equivalent and record why
     the root launcher is not applicable
7. Capture deployment evidence from the clean implementation commit.
8. Run `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`.
9. Run smoke checks `.\scripts\smoke.ps1` and the mandatory crate tests.
10. Run Playwright coverage:
    - direct runner from the repository root:
      `npm --prefix .\end2end test`
    - retained evidence: `.\scripts\validate-e2e.ps1 ...`
    - never run bare `npx playwright test` from the repository root
11. Run formatting check `cargo fmt --all`.
12. Update the repo roadmap, using Tessara's current path by default:
   - `D:/Projects/tessara/docs/roadmap.md`
   - set completed sprint label to `(Complete)`
   - set next sprint label to `(Next)`
13. Prepend a new entry to the repo progress report, using Tessara's current path by default:
   - `D:/Projects/tessara/docs/progress-report.md`
   - date title
   - achievements
   - validation status
   - next focus
14. Add the **Sprint Handoff / Demo Instructions** subsection to the progress report entry using the required template below.
15. For each sprint acceptance/exit condition:
    - capture at least one manual demo step
    - capture at least one automated/scripted assertion
16. Commit the closeout-only documentation separately. Do not rebuild product
    images for a documentation-only commit; name the implementation/evidence
    commit explicitly.
17. Leave the application running in a user-testable state at the close of the workflow unless the user explicitly asks to shut it down.

## Closeout-readiness audit

Finish this cheap audit before the final cycle:

- working tree changes are understood and belong to the sprint
- every roadmap exit-condition clause has a planned manual and automated proof
- sprint plan, smoke, UAT, Playwright manifest/tests, seed assumptions, and
  navigation/module inventories agree with the implementation
- Compose/deployment configuration parses and identifies the correct databases,
  roles, private/public routes, bootstrap command, and health checks
- bootstrap/materialization succeeds and is idempotent
- source provenance arguments and image labels can represent the exact clean
  commit
- migration baselines apply transactionally to disposable empty databases
- the expected migration-ledger shape is explicit
- targeted tests for all changed crates, routes, contracts, and harnesses pass
- evidence output paths are empty, versioned intentionally, or explicitly
  overwriteable

Do not begin the expensive fresh-stack/full-suite cycle while any audit item is
unknown or failing.

## Rerun policy

- Treat the final source-exact cycle as a one-run budget, not a discovery pass.
- On failure, classify the smallest affected layer, fix it, and run its
  targeted reproducer first.
- A source change invalidates the image and source-bound deployment, smoke,
  UAT, and browser evidence. Rebuild and recapture those after the targeted
  fix passes.
- Do not rerun unrelated Rust or browser suites when their exact-source result
  remains valid and the fix cannot affect them; record the rationale.
- Never reuse evidence from a different source commit, database state, image,
  or acceptance manifest.
- Record command durations and the reason for every repeated full run in the
  verification document.

## Mandatory verification commands

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `npm --prefix .\end2end test`
- `.\scripts\validate-e2e.ps1 ...` with the sprint's deployment evidence, expected data state, and evidence output path
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

If a command is out of scope for the sprint, document explicit rationale and still keep the required checklist complete. Do not silently skip it.

## Mandatory closeout documentation sections

Every closeout entry in `progress-report.md` must include these headings:

### Sprint Handoff / Demo Instructions

Use this section for reviewer-ready demonstration steps.

For each functionality delivered, provide:

- functionality name
- role required (`admin`, `operator`, `respondent`)
- paths to open (URLs and endpoints)
- step-by-step user actions
- expected visible result
- acceptance check (pass/fail criteria)
- evidence location (test output, console output, screenshot, or transcript)

### Acceptance Mapping

For every sprint user-testable exit condition, include:

- exit condition text
- manual walkthrough artifact (`Sprint Handoff / Demo Instructions` step)
- automated/scripted evidence command or assertion

## Suggested handoff section template

```md
## YYYY-MM-DD - <Sprint Name> Closeout

- Completed:
  - ...
- Validation:
  - `local-launch` run completed
  - `scripts\uat-sprint.ps1` run completed
  - `scripts\smoke.ps1` completed
  - Relevant tests run:
    - ...
- Next Sprint: <Sprint Name>

## Sprint Handoff / Demo Instructions

### <Functionality Name>
- Role: admin
- Paths:
  - `http://localhost:8080/...`
- Steps:
  1. ...
- Expected:
  - ...
- Acceptance check:
  - ...
- Evidence location:
  - ...

## Acceptance Mapping

- Exit condition:
  - ...
- Manual demonstration:
  - ...
- Automated check:
  - ...
```

## Baseline requirements

- Base URL: `http://localhost:8080`
- Demonstration account set:
  - admin: `admin@tessara.local`
- In Tessara, prefer repo-local docs and scripts:
  - roadmap: `D:/Projects/tessara/docs/roadmap.md`
  - progress report: `D:/Projects/tessara/docs/progress-report.md`
  - local launch: `D:/Projects/tessara/scripts/local-launch.ps1`
  - sprint UAT: `D:/Projects/tessara/scripts/uat-sprint.ps1`
  - smoke: `D:/Projects/tessara/scripts/smoke.ps1`
  - Playwright: `D:/Projects/tessara/end2end/tests`
- Direct Playwright execution must resolve the repository-owned runner with
  `npm --prefix .\end2end test` from the repository root, or by running
  `npx playwright test` only after changing the working directory to `end2end`.
  The root package does not own `@playwright/test` or its configuration.
- Add one constrained non-admin validation where role gating is relevant.
- Attach at least one evidence artifact for each functional area: screenshot, transcript, or test/log output.
- Any unsupported or deferred demo scenario must be explicitly marked as blocked with owner and next-step date.
- When schema changes are in scope, migration squashing and a from-scratch migration check must precede the final source-exact build and all retained evidence.
- Retained deployment, smoke, UAT, and Playwright evidence is valid only for the exact clean source commit it records. A later source change requires rebuilding and recapturing the affected evidence.
- Unless the user says otherwise, finish with the application still reachable at `http://localhost:8080` for manual walkthrough.

## Standard functionality checklist

- Use the sprint roadmap block to derive the functional checklist instead of hard-coding the prior sprint's areas.
- When a kickoff plan exists in `docs/sprints/`, use it to seed the functionality checklist and acceptance mapping.
- For Tessara, always include:
  - the sprint's product routes and UI flows
  - access control / role gating where relevant
  - read-only and authoring surfaces touched by the sprint
  - UI/build/style surface changes if touched
  - any repo script or smoke/UAT updates made for the sprint

## Finish criteria

Do not finalize closeout if:

- roadmap/progress updates are missing
- schema-changing sprint migrations have not been squashed and verified from scratch before final evidence capture
- uat/smoke/test/format checks are not recorded
- at least one functionality in the sprint has no handoff demo step
- any acceptance condition lacks both a manual and scripted evidence entry

---
name: tessara-sprint-closeout
description: Execute and document a Tessara sprint closeout as a repeatable handoff process, including roadmap and progress updates, mandatory environment and test validation, and reviewer-ready demo instructions. Use when a Tessara sprint is ending or when a user asks to produce the sprint handoff and closeout package.
---

# Tessara Sprint Closeout

Use this skill whenever a Tessara sprint is ending or when asked to produce the next-sprint handoff package.

## Core behavior

Create a closeout that is directly testable by a non-developer:

- update roadmap and progress-report status first
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

1. Confirm the sprint completion target and evidence scope from roadmap acceptance points.
2. Update the repo roadmap, using Tessara's current path by default:
   - `D:/Projects/tessara/docs/roadmap.md`
   - set completed sprint label to `(Complete)`
   - set next sprint label to `(Next)`
3. Prepend a new entry to the repo progress report, using Tessara's current path by default:
   - `D:/Projects/tessara/docs/progress-report.md`
   - date title
   - achievements
   - validation status
   - next focus
4. Run environment bootstrap:
   - `.\scripts\local-launch.ps1` (or `.\scripts\local-launch.ps1 -FreshData` when seed-sensitive)
5. Run `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`.
6. Run smoke checks `.\scripts\smoke.ps1` and targeted crate tests.
7. Run Playwright coverage for the application routes touched by the sprint.
8. Run formatting check `cargo fmt --all`.
9. Add the **Sprint Handoff / Demo Instructions** subsection to the progress report entry using the required template below.
10. For each sprint acceptance/exit condition:
    - capture at least one manual demo step
    - capture at least one automated/scripted assertion
11. Leave the application running in a user-testable state at the close of the workflow unless the user explicitly asks to shut it down.

## Mandatory verification commands

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `npx playwright test`
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
- Add one constrained non-admin validation where role gating is relevant.
- Attach at least one evidence artifact for each functional area: screenshot, transcript, or test/log output.
- Any unsupported or deferred demo scenario must be explicitly marked as blocked with owner and next-step date.
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
- uat/smoke/test/format checks are not recorded
- at least one functionality in the sprint has no handoff demo step
- any acceptance condition lacks both a manual and scripted evidence entry

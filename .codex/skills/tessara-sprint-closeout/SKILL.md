---
name: tessara-sprint-closeout
description: Execute and document a Tessara sprint closeout after the tessara-sprint-validation regime has authorized the exact candidate, including evidence audit, roadmap and progress updates, reviewer-ready demo instructions, and final handoff state. Use when a validated Tessara sprint is ending or when a user asks to produce its closeout and next-sprint handoff package.
---

# Tessara Sprint Closeout

Use this skill only after `tessara-sprint-validation` has authorized closeout.
Treat `docs/roadmap.md` as the authoritative scope and the sprint plan and
verification record as supporting inputs.

When retained history contains a candidate-invalidating post-SIT failure,
read
[`../tessara-sprint-validation/references/post-sit-defect-convergence.md`](../tessara-sprint-validation/references/post-sit-defect-convergence.md)
completely and validate its records against
[`../tessara-sprint-validation/references/post-sit-defect-convergence.schema.json`](../tessara-sprint-validation/references/post-sit-defect-convergence.schema.json).

## Boundary with validation

Closeout consumes validation evidence; it does not create it.

- Do not run an acceptance, smoke, SIT, or UAT check for the first time here.
- If no authorized verification record exists, invoke
  `tessara-sprint-validation` and complete its full regime before continuing.
- Require the coordinator-issued `closeout-authorization.json` and verify its
  prerequisite receipt and evidence-manifest hashes.
- If evidence is missing, stale, or tied to multiple candidates, reopen
  validation rather than filling the gap during closeout.
- If a post-SIT convergence cycle occurred, require its complete retained
  harvest, batch, impact, focused-attempt, restoration, and final-entry chain;
  verify that the authorized candidate was frozen only after the subsequent
  complete readiness and rehearsal passed.
- If closeout reveals a missing acceptance assertion or changes executable,
  harness, migration, seed, manifest, bootstrap, or deployment source, create
  a new candidate and restart SIT.
- Allow documentation-only corrections to remain in closeout when they cannot
  alter executable behavior or test interpretation.

## Inputs

- sprint name and status date
- `docs/roadmap.md`
- optional `docs/sprints/<slug>-plan.md`
- authorized `docs/sprints/<slug>-verification.md`
- retained source-exact evidence and its candidate identity
- optional next-sprint target when the roadmap does not identify one

## Required execution order

1. Derive the clause-level completion checklist from the roadmap.
2. Audit the verification record and retained evidence against the closeout
   authorization requirements below.
3. Confirm the evidence-source implementation commit is clean and identify
   any later documentation-only commit separately.
4. Confirm the intended candidate slot is active, all required services are
   healthy, and the application is reachable at the handoff URL.
5. Update the roadmap:
   - mark the completed sprint `(Complete)`
   - mark exactly one next sprint `(Next)`
6. Prepend the closeout entry to `docs/progress-report.md` with achievements,
   validation status, next focus, handoff instructions, and acceptance mapping.
7. Update the sprint plan and verification file to reflect final completion
   without rewriting source-exact test results.
8. Commit closeout-only documentation separately. Do not rebuild images for a
   documentation-only commit.
9. Verify clean Git state, active route, application health, and documentation
   links after the closeout commit.
10. Leave the application running and reviewer-testable unless the user asks
    to stop it.

## Closeout authorization audit

Require all of the following before changing roadmap status:

- preflight preceded SIT
- one complete clean candidate identity covers all retained SIT and UAT evidence
- static/boundary, Rust workspace, Playwright, and deployed acceptance-smoke
  SIT lanes passed
- scripted and manual UAT passed after SIT
- the failure chronology shows a coordinator-issued invalidation decision for
  every correction and proves every required scoped or complete rerun passed
- every post-SIT defect harvest accounts for every UAT scenario as executed or
  explicitly blocked, every defect converged, and the coordinator authorized
  return to final readiness/rehearsal
- focused repair evidence is marked non-authoritative and was not reused to
  satisfy the successor candidate's complete SIT or UAT
- the final successor fingerprint has its own complete readiness, rehearsal,
  preflight, SIT, and UAT chain from the beginning
- every roadmap exit-condition clause maps to automated and manual evidence
- changed route, navigation, lifecycle, role, seed, manifest, bootstrap, and
  deployment contracts have explicit coverage
- no acceptance test first appeared during closeout
- no unresolved product decision, unsupported scenario, or unowned blocker
- intended candidate routing, health, provenance, and rollback state are recorded

Stop if any item is false or unknown. Return to `tessara-sprint-validation`.

## Evidence integrity

- Match deployment, smoke, Playwright, scripted UAT, and manual UAT evidence
  to the recorded commit, tree, image digest, configuration, migration state,
  and acceptance inventory.
- Never combine passing results from different candidate identities.
- Verify evidence hashes and files without rerunning the tests they represent.
- Verify `preflight-result.json`, `candidate.json`, `sit-result.json`,
  `uat-result.json`, and `closeout-authorization.json` form one valid hashed
  receipt chain.
- Preserve superseded pre-correction authoritative evidence and diagnostic
  convergence evidence as history, but exclude both from the final passing
  candidate's authoritative proof.
- Record command durations and restart reasons already captured during
  validation; do not manufacture missing chronology during closeout.
- Preserve the immutable rollback baseline and confirm the intended candidate
  route is restored after validation.

## Progress report requirements

Prepend a dated closeout entry containing:

- completed functionality
- evidence-source implementation commit and tree
- closeout documentation commit when available
- SIT and UAT pass summary
- active candidate/release and health state
- next sprint
- `Sprint Handoff / Demo Instructions`
- `Acceptance Mapping`

## Sprint Handoff / Demo Instructions

For each delivered functionality, provide:

- functionality name
- required role
- URLs and endpoints
- step-by-step user actions
- expected visible result
- explicit pass/fail acceptance check
- evidence location

Use the validated manual UAT scenarios as the source. Do not invent an
untested demonstration path during closeout.

## Acceptance Mapping

For every roadmap exit-condition clause, include:

- exact or faithfully preserved exit-condition text
- corresponding handoff/manual demonstration
- automated assertion or command
- deployed-smoke assertion when the clause changes an integration contract
- evidence location and candidate identity

## Handoff template

```md
## YYYY-MM-DD - <Sprint Name> Closeout

- Completed:
  - ...
- Validation:
  - Candidate: `<commit>` / `<tree>` / `<image digest>`
  - SIT: Passed
  - UAT: Passed
- Active release: ...
- Next Sprint: ...

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
- Smoke check:
  - ...
```

## Tessara defaults

- Base URL: `http://localhost:8080`
- Admin account: `admin@tessara.local`
- Roadmap: `docs/roadmap.md`
- Progress report: `docs/progress-report.md`
- Sprint artifacts: `docs/sprints/`
- Validation authority: `tessara-sprint-validation`
- Keep one constrained non-admin demonstration for authorization-sensitive work.
- Mark a deferred scenario with owner and next-step date; do not call the
  sprint complete while it remains an exit-condition blocker.

## Finish criteria

Do not finalize closeout if:

- the verification record did not authorize closeout
- closeout was the first execution point for any acceptance check
- roadmap or progress updates are missing or inconsistent
- any exit condition lacks both automated and manual evidence
- evidence does not resolve to one clean candidate
- an executable or harness change was made without restarting SIT
- the closeout documentation commit is not distinguished from the evidence
  source commit
- the intended application route is unhealthy or not left reviewer-testable

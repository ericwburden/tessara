---
name: tessara-sprint-validation
description: Execute Tessara sprint validation as a strict preflight, source-exact candidate freeze, SIT, deployed acceptance smoke, UAT, and closeout-authorization regime. Use when planning or running Tessara tests, determining whether SIT or UAT passed, correcting a failed validation gate, preparing acceptance evidence, deciding whether a sprint may enter closeout, or investigating a defect discovered during closeout.
---

# Tessara Sprint Validation

Use this skill as the test authority between implementation and
`tessara-sprint-closeout`. Treat the sprint roadmap and plan as scope inputs;
do not let them weaken this gate contract.

## Invariant

Execute this state machine exactly:

```text
preflight -> freeze candidate -> SIT -> UAT -> authorize closeout
                                  ^       |
                                  |_______|
                                  any failure or candidate change
```

- Include deployed acceptance smoke inside SIT.
- Do not start UAT until all SIT lanes pass for one candidate.
- On any SIT or UAT failure, correct the issue and restart SIT from its first
  lane against a newly identified or reconfirmed candidate.
- Do not run an acceptance check for the first time during closeout.
- Stop for user direction when correction requires an unresolved product
  decision. Do not classify a product decision as a test failure.

## Establish the validation record

Copy `assets/sprint-validation-record.md` to
`docs/sprints/<sprint-slug>-verification.md` when the sprint does not already
have a verification file. Preserve useful existing content when one exists.

Before implementation freeze, record:

- every roadmap exit-condition clause
- the product, authorization, lifecycle, deployment, and rollback risks
- one automated assertion and one manual UAT scenario per clause
- the mandatory smoke assertions for changed integration contracts
- commands, environments, accounts, fixtures, and evidence paths

Update the smoke, UAT, and Playwright inventory in the same implementation
change that alters routes, navigation, manifests, lifecycle schemas, roles,
seed data, service topology, fallback documents, or bootstrap behavior.

## Gate 0: preflight

Complete preflight before recording any SIT result:

- confirm the intended sprint worktree, branch, and repository instructions
- understand all worktree changes and ensure they belong to the sprint
- identify the implementation commit intended for candidate freeze
- validate test database names, ports, credentials, reset authorization, and
  required environment variables
- validate the Compose/deployment profile and expected active service slot
- confirm the bootstrap/materialization command and idempotent second run
- confirm smoke, Rust, Playwright, scripted UAT, and manual UAT commands exist
- reconcile harness expectations with the implemented route, navigation,
  lifecycle-schema, role, seed, and module inventory
- verify evidence destinations are empty or intentionally overwriteable
- when schema changed, apply finalized migration baselines to disposable empty
  databases and verify the ledger before candidate freeze

Classify a failure found before SIT begins as `preflight/setup`. Correct it and
rerun preflight. Do not report it as an SIT failure. If its correction changes
tracked source, the later candidate identity must include that change.

## Freeze the candidate

Require a clean implementation commit. Record at least:

- commit and tree
- dirty state
- image digest and embedded source-provenance labels when containers apply
- deployment profile and configuration digest
- migration/baseline identity
- acceptance-manifest or test-inventory identity

Use the same candidate for every retained SIT and UAT result. Any tracked
product, test, harness, migration, seed, bootstrap, manifest, or deployment
configuration change creates a new candidate and invalidates prior downstream
results.

Documentation-only changes made after validation do not invalidate the
candidate when they cannot alter executable behavior or test interpretation.
Name both the evidence-source commit and the later documentation commit.

## Gate 1: SIT

Run every lane from the beginning for the frozen candidate. Order cheap lanes
before expensive ones when dependencies allow, but retain one complete result
set.

### Static and boundary lane

- run `cargo fmt --all -- --check`
- run sprint-specific boundary, package, manifest, schema, and Compose checks
- validate migration baselines and source provenance when applicable

### Rust lane

- default to `cargo test --workspace --locked`
- include every required database-backed suite with explicitly isolated test
  databases
- document only genuinely intentional ignored tests

### Playwright lane

- run `npm --prefix .\end2end test` from the repository root
- use the repository-owned runner; never use bare root-level
  `npx playwright test`
- retain the sprint's evidence-producing wrapper when the plan requires it

### Deployed acceptance-smoke lane

Build and deploy the exact frozen candidate, then run `scripts/smoke.ps1` or
the sprint profile's documented equivalent. Smoke must prove the application
is integrated, not merely that processes answer health checks.

For each changed contract, include the smallest high-signal assertion. Review
at least:

- service health, gateway route, candidate slot, and source provenance
- login/session bootstrap and one constrained non-admin case when relevant
- product routes and complete-document fallbacks
- module manifest/navigation/lifecycle schema compatibility
- soft mount, single outlet/resource ownership, and unmount cleanup when a
  lifecycle module changed
- contained module failure while Core remains usable
- bootstrap/materialization data needed by UAT

Treat missing or stale smoke coverage as an SIT failure. Fix the harness,
create a new candidate, and restart SIT. Closeout must never be the first place
these assertions execute.

Mark SIT passed only when all four lanes pass for the same candidate.

## Gate 2: UAT

Freeze the UAT script before execution. Run:

1. `scripts/uat-sprint.ps1 -BaseUrl "http://localhost:8080"` or the profile's
   documented equivalent.
2. Every manual scenario recorded in the validation file.
3. Role, scope, responsive, failure-containment, recovery, upgrade, and
   rollback scenarios when the sprint changed those contracts.

For each scenario, record:

- role and starting state
- exact action and expected visible outcome
- pass/fail result
- evidence location
- candidate identity

Do not expand UAT informally while running it. If a missing scenario is
discovered, add it to the test inventory and treat the gap as a failure:
correct coverage, then restart at SIT.

## Failure handling

When SIT or UAT fails:

1. Stop the current gate.
2. Record the failing lane, command/scenario, candidate, and evidence.
3. Classify the cause as `product`, `harness`, `environment`, `flaky`, or
   `product-decision`.
4. Reproduce with the narrowest safe check.
5. For `product-decision`, present the decision and pause.
6. Otherwise correct the cause and run the narrow reproducer.
7. Reconfirm or refreeze the candidate.
8. Restart SIT from the static and boundary lane.

Do not resume at the failed UAT scenario. Do not combine results from multiple
candidate identities into one pass.

## Authorize closeout

Authorize `tessara-sprint-closeout` only when the validation record shows:

- one candidate identity for all retained evidence
- every SIT lane passed
- scripted and manual UAT passed
- every roadmap clause has automated and manual evidence
- no unresolved product decision or unsupported scenario
- candidate routing and application health are in the intended handoff state

Closeout may verify hashes, evidence completeness, documentation, clean Git
state, health, and handoff routing. It must not originate a product, smoke,
SIT, or UAT test.

If closeout reveals a missing or stale acceptance assertion, do not patch over
the gap in closeout. Reopen validation, add or correct the test, and restart
SIT. If closeout changes executable or harness source, restart SIT. A strictly
documentation-only correction may remain in closeout.

## Finish criteria

Do not report validation complete unless:

- preflight passed before SIT began
- the candidate identity is complete and clean
- deployed acceptance smoke ran and passed inside SIT
- all SIT lanes and all UAT scenarios passed for that candidate
- all failures and restarts are recorded
- the verification file explicitly authorizes closeout

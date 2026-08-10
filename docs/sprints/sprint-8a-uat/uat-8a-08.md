# UAT-8A-08 — Component-only upgrade and rollback

## 1. Test Script Summary

- System / Module: Tessara Components deployment lifecycle
- Requirement: Sprint 8A AC-14
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator and reviewer
- Scenario: Establish the source-built compatible Component `0.9.0` release, upgrade to `1.0.1`, roll back to `0.9.0`, and restore intended `1.0.1` through Supervisor/Compose applies.
- Acceptance: Every exact one-owner switch is health-gated; the two releases have distinct source-built executable/image/manifest identities; Component state persists; unrelated service identities/restarts do not change; and final topology is healthy on `1.0.1`.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-08`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor bindings: `operator` account __________; `reviewer` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `component-baseline-snapshot`, `unrelated-owner-baseline`, `compatible-release-provenance`.
- Evidence folder and execution start time: ____________________
- Record Component and unrelated image digests, container identities, restart counts, instance, configuration, and a recognizable Component: ____________________
- Confirm the baseline metadata/sidecar identifies a source-built `0.9.0`
  binary and manifest from the candidate source, and that its executable/image
  identities differ from candidate `1.0.1`.
- Structured evidence: retain at least one raw artifact for every exact
  `upgrade-rollback-receipt` assertion, then use
  `Publish-Sprint8AManualUatStructuredEvidence`. The canonical producer and
  wrapper bind the attempt, candidate, environment, Start checkpoint, and
  execution lease; every assertion must be `passed` and name the same producer
  receipt plus its retained raw evidence.

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | From healthy `1.0.1`, apply the compatible-baseline selection through the composition API and Supervisor. | The resolved plan is an exact Component-only delta; Supervisor/Compose establishes healthy `0.9.0` without changing Component state or unrelated owners. | `baseline-selection-record` | | | |
| 2 | Apply the `1.0.1` selection and exercise Component plus Dashboard routes. | The exact `0.9.0` to `1.0.1` upgrade is health-gated; recorded Component data/configuration and placements remain usable. | `candidate-upgrade-record` | | | |
| 3 | Compare the pre-transition and post-upgrade snapshots. | Core, gateway, Supervisor, Dashboard, Dataset compatibility, and other modules retain image/container/restart, semantic data, navigation, and availability identity. | `unrelated-owner-stability-record` | | | |
| 4 | Apply rollback to `0.9.0`, verify health/behavior, then restore intended `1.0.1`. | Each plan remains Component-only and health-gated; release read-back matches each stage; final state is healthy intended `1.0.1`; publish the authenticated upgrade/rollback JSON receipt. | `upgrade-rollback-receipt` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)

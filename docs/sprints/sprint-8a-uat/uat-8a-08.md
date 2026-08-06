# UAT-8A-08 — Component-only upgrade and rollback

## 1. Test Script Summary

- System / Module: Tessara Components deployment lifecycle
- Requirement: Sprint 8A AC-14
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator and reviewer
- Scenario: Establish the source-built compatible Component `0.9.0` release, upgrade to `1.0.0`, roll back to `0.9.0`, and restore intended `1.0.0` through Supervisor/Compose applies.
- Acceptance: Every exact one-owner switch is health-gated; the two releases have distinct source-built executable/image/manifest identities; Component state persists; unrelated service identities/restarts do not change; and final topology is healthy on `1.0.0`.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Evidence folder and execution start time: ____________________
- Record Component and unrelated image digests, container identities, restart counts, instance, configuration, and a recognizable Component: ____________________
- Confirm the baseline metadata/sidecar identifies a source-built `0.9.0`
  binary and manifest from the candidate source, and that its executable/image
  identities differ from candidate `1.0.0`.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | From healthy `1.0.0`, apply the compatible-baseline selection through the composition API and Supervisor. | The resolved plan is an exact Component-only delta; Supervisor/Compose establishes healthy `0.9.0` without changing Component state or unrelated owners. | | | |
| 2 | Apply the `1.0.0` selection and exercise Component plus Dashboard routes. | The exact `0.9.0` to `1.0.0` upgrade is health-gated; recorded Component data/configuration and placements remain usable. | | | |
| 3 | Compare the pre-transition and post-upgrade snapshots. | Core, gateway, Supervisor, Dashboard, Dataset compatibility, and other modules retain image/container/restart, semantic data, navigation, and availability identity. | | | |
| 4 | Apply rollback to `0.9.0`, verify health/behavior, then restore intended `1.0.0`. | Each plan remains Component-only and health-gated; release read-back matches each stage; final state is healthy intended `1.0.0`. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

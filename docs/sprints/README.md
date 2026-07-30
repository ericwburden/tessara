# Sprint Plans

`tessara-sprint-kickoff` writes sprint kickoff plans here.

Use the generated `sprint-<label>-plan.md` file as the implementation contract for that sprint worktree.

Out-of-roadmap sprint proposals and migration plans may also live here when they need the same level of delivery detail before they are admitted to the roadmap.

## Current Roadmap Position

Sprint 6C closed on 2026-07-29 with Dashboard operating in an independent
process and database. The accepted post-closeout sequencing decision treats
that as the runtime/data boundary and completes source/build independence in
two steps:

1. **Sprint 6D: Canonical Module SDK And Runtime Extraction** is next.
2. **Sprint 6E: Dashboard SDK Adoption And Source Independence** follows.
3. **Sprint 6F: Application Blueprint And Composition Automation** retains the
   former Sprint 6D Blueprint scope.

Later module extractions apply the completed Sprint 6D/6E pattern. See the
[roadmap](../roadmap.md#sprint-6d-canonical-module-sdk-and-runtime-extraction-slice-next)
and the accepted
[module SDK source-ownership decision](../architecture/module-sdk-source-ownership.md).

## Completed Sprint 6A-UI Artifacts

- [Sprint 6A-UI Navigation Composition And Module Management Harmonization Plan](./sprint-6a-ui-plan.md)
- [Sprint 6A-UI Navigation And Module Management Baseline](./sprint-6a-ui-baseline-inventory.md)
- [Sprint 6A-UI Test Change Log](./sprint-6a-ui-test-change-log.md)
- [Sprint 6A-UI Targeted Audit](../audits/sprint-6a-ui-module-management-2026-07-15/README.md)
- [Sprint 6A-UI Pre-Navigation-Model Directory Directions](../mockups/sprint-6a-ui/README.md)
- [Sprint 6A-UI Roadmap Contract](../roadmap.md#sprint-6a-ui-navigation-composition-and-module-management-harmonization-slice-next)

## Completed Sprint 6A Artifacts

- [Sprint 6A Module Contract And Core Control Plane Plan](./sprint-6a-plan.md)
- [Sprint 6A Kickoff Baseline Evidence](./sprint-6a-kickoff-baseline-evidence.md)
- [Sprint 6A Transition Catalog Contract](./sprint-6a-transition-catalog.md)
- [Sprint 6A Frozen Regression Matrix](./sprint-6a-regression-matrix.md)
- [Sprint 6A Test Change Log](./sprint-6a-test-change-log.md)
- [Sprint 6A Deployment Evidence Contract](./sprint-6a-deployment-evidence.md)

## Historical Planning Artifacts

- [Sprint 3C Plan](./sprint-3c-plan.md)

## Completed Sprint 5A Artifacts

- [Sprint 5A Dashboard Editor Design Decision](./sprint-5a-dashboard-editor-design.md)
- [Sprint 5A Dashboard Composition Plan](./sprint-5a-plan.md)

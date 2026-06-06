# Sequencing And Dependencies

## Purpose

Make Sprint 2F work order and dependencies explicit.

## Completion Guidance

Use this to avoid scope drift during implementation.

## Related Checks

`sequencing-and-dependencies`, `planning-traceability`, `technical-planner-boundary`.

## Sequencing Summary

Inventory first, contracts second, UI third, verification throughout.

## Workstream Order

1. Runtime/materialization inventory.
2. Route and DTO decision.
3. Backend implementation.
4. Native UI implementation.
5. Permission and regression tests.
6. Validation and Orpheum closeout.

## Dependency Map

- UI depends on DTO shape.
- DTO shape depends on inventory.
- Permission tests depend on route/action behavior.
- Closeout depends on validation evidence and Orpheum artifacts.

## Critical Path

Inventory -> contract -> implementation -> tests -> UAT/Orpheum checks.

## Parallelization Opportunities

Validation matrix, selector verification plan, and permissions scenario design can proceed while backend inventory happens.

## Decision Gates And Spikes

Gate 1: Is existing state sufficient? Gate 2: What route owns monitoring? Gate 3: Are any new protected behaviors executable in Playwright?

## Integration, Migration, Or Rollout Checkpoints

Check local launch, demo seed, smoke, UAT, Playwright permissions, and any touched import/refresh command separation.

## Verification Touchpoints

After backend contracts, after UI, before closeout, and after Orpheum artifact checks.

## Remaining Sequencing Risks And Assumptions

Assumes no broad schema migration is needed. If that changes, revisit scope before coding deeply.

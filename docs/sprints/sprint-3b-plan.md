# Sprint 3B Plan: Dataset Advanced Authoring Slice

## Sprint Summary

Sprint 3B makes dataset authoring useful beyond direct source-field projection. Dataset authors should be able to add row filters, explicit restriction rules, and calculated fields, then preview and save definitions with those advanced authoring choices applied consistently.

## Sprint Specifications

- Add row filter authoring for dataset sources or dataset output, with clear UI validation and preview behavior.
- Add explicit dataset restriction filters or rules, including possible row-level node restrictions and custom capabilities, so richer access behavior is deliberately authored instead of implied by system metadata.
- Add calculated field authoring for v1-safe expressions over selected source fields.
- Add typed validation and error states for invalid filters, missing field references, and unsupported calculated-field expressions.
- Ensure preview execution applies filters and calculated fields consistently with saved definitions.
- Keep the basic Sprint 3A authoring workflow intact while exposing row filters and calculated fields in dataset edit screens.

## Acceptance Criteria

- A tester can add a row filter to a dataset and preview rows with the filter applied.
- A tester can add a calculated field over selected source fields and preview the calculated value.
- A tester can save and reopen a dataset definition with row filters, calculated fields, and explicit restriction rules intact.
- Invalid filters, missing field references, and unsupported calculated-field expressions produce typed validation or error states instead of silent failure.
- Explicit restriction behavior is authored through visible rules rather than implicit filtering by system metadata.
- Existing Sprint 3A dataset authoring flows for definition, data sources, fields, aggregation, SQL preview, and visibility remain usable.

## Manual Test Plan

- Sign in as an admin and open `/datasets`.
- Create or edit a dataset at `/datasets/new` or `/datasets/{dataset_id}/edit`.
- Add source or output row filters and confirm the SQL preview and data preview reflect the authored rules.
- Add at least one calculated field using selected source fields and verify it appears in preview output after save and reopen.
- Configure explicit restriction rules and verify access behavior follows the authored restriction rather than implicit node metadata.
- Exercise invalid filter and calculated-field inputs and verify actionable validation appears in the editor.

## Automated Test Plan

- `cargo fmt --all`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `npx playwright test`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Review the Sprint 3A dataset definition, SQL generation, preview, and editor state contracts.
2. Extend dataset definition DTOs and persistence for row filters, calculated fields, and explicit restriction rules.
3. Implement backend validation and SQL generation for supported v1 filter and calculated-field expressions.
4. Update preview execution so filters and calculated fields apply consistently before save and after reload.
5. Add dataset editor UI for row filters and calculated fields without disrupting the existing Sprint 3A sequence.
6. Add explicit restriction rule UI and backend handling for the selected v1 restriction shape.
7. Add focused API, web, and Playwright coverage for save/reopen, preview, validation, and restriction behavior.
8. Run the planned verification commands and capture closeout evidence.

## Dependencies And Blockers

- The sprint depends on the Sprint 3A dataset authoring foundation, especially dataset editor state, generated SQL preview, field projection, aggregation, and visibility behavior.
- The exact v1 expression surface should stay intentionally small enough for typed validation and reliable SQL generation.
- Explicit restriction rules should be implemented as authored access behavior and should not reintroduce implicit filtering by `__node_id`.

## Future Work Notes

- Dataset authoring should evolve from a fixed panel sequence into an operation pipeline. Once the source expression exists, authors should be able to independently add projection, aggregation, calculated-field, filter, and possibly view-restriction steps to the dataset definition, including multiple instances of an operation type where useful. The editor and SQL generator should continue to apply operations in operation-list order so the authored pipeline is also the execution contract.

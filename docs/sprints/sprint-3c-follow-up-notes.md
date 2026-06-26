# Sprint 3C Follow-Up Notes

These notes capture post-PR #100 organization and hardening work that should not block the Sprint 3C merge. They preserve the trigger for each item so future planning can pull work forward when it becomes materially useful.

## Immediate Follow-Up

### Centralize Restriction-Tier Semantics

Trigger: the next change that touches dataset row-tier behavior, tier access predicates, or restriction policy SQL.

Plan: move restriction-tier ordering, rank SQL, greatest/max tier SQL, policy CASE SQL, and access predicates into one helper/module. Keep the current ordering `confidential > restricted > internal > public` and preserve existing tests.

Validation: run dataset compiler unit tests, `demo_flow`, and API clippy.

### Add Backend/Frontend Golden Catalog Tests

Trigger: the next dataset editor or compiler change that affects available fields, output fields, source catalogs, joins, unions, projections, aggregations, calculated fields, filters, or restriction field options.

Plan: add paired backend/frontend test scenarios that assert the backend compiler and frontend editor catalog simulation produce matching field keys/types for representative pipelines. Prioritize dataset-source revision fields, compatible unions, joins, projection renames, aggregation plus calculated fields, and mixed restriction tiers.

Validation: run `cargo test -p tessara-api datasets:: --lib`, `cargo test -p tessara-web --features hydrate --lib`, and the affected integration tests.

### Move Dataset Integration-Test Helpers

Trigger: the next dataset integration test that adds another reusable JSON operation builder or repeats existing dataset-authoring setup.

Plan: move reusable helpers from `crates/tessara-api/tests/demo_flow.rs` into `crates/tessara-api/tests/support/datasets.rs`, keeping scenario assertions in `demo_flow.rs`.

Validation: run `cargo test -p tessara-api --test demo_flow`.

## Deferred Until Needed

### Split `datasets/mod.rs`

Trigger: before the next substantial dataset compiler/materialization feature, or when a bug fix needs to touch more than one conceptual layer in `crates/tessara-api/src/datasets/mod.rs`.

Plan: perform a mechanical extraction into focused modules for handlers, access, repository, materialization, and compiler submodules. Avoid behavior changes in the same PR.

Validation: run full dataset API tests, `demo_flow`, and API clippy.

### Introduce A Pipeline Schema Abstraction

Trigger: after the dataset module split, or when another internal pipeline column is added beyond `__row_id` and `__restriction_tier`.

Plan: introduce a small `PipelineSchema`, `PipelineColumns`, or `CompiledSchema` abstraction that separates internal CTE columns from user-visible dataset fields.

Validation: run compiler unit tests and paired catalog tests.

### Split Revision Field Loading From `DatasetSummary`

Trigger: when `/api/datasets` payload size, latency, or call-site needs show that returning output fields and revision field summaries by default is too heavy.

Plan: add an explicit `include_fields` option or a dedicated revision-fields endpoint, then update dataset editor source catalogs to request fields on demand.

Validation: run web editor unit tests, dataset API contract tests, and manual dataset-source catalog checks.

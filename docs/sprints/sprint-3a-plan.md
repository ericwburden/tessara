# Sprint 3A: Dataset Authoring Foundation

## Sprint Summary

Sprint 3A replaces the native `/datasets` placeholders with practical v1 Dataset Authoring: directory, detail, create, edit, and preview flows backed by the existing dataset APIs. Authoring is admin-only for this slice; scoped users with `datasets:read` can browse and preview visible datasets.

## Sprint Specifications

- Add native SSR routes for `/datasets`, `/datasets/new`, `/datasets/{dataset_id}`, and `/datasets/{dataset_id}/edit`.
- Reuse existing dataset APIs for list/detail/table/create/update; do not add delete UI.
- Create/edit supports submission-grain datasets, union/join composition, published-form sources, `all/latest/earliest` source selection, visibility nodes, and exposed source-field mappings.
- Detail/edit screens show metadata, visibility nodes, source and field tables, and a dataset preview table.
- Dataset Visibility selections are the dataset read gate. Materialized rows are not implicitly filtered by `__node_id`; `__node_id` remains available as normal system metadata for grouping, joins, debugging, and future explicit restriction rules.
- Tighten `/api/form-versions/{id}/render` so form field metadata requires readable form access.
- Defer row filters, calculated fields, explicit dataset restriction filters/rules, dataset revision history, compatibility findings, component authoring, and dashboard work.

## Acceptance Criteria

- Admin can create a dataset, open detail, preview rows, edit the definition, and see the updated definition.
- Scoped readers see only datasets visible to their scope, and can read the full materialized output for those datasets.
- No-capability users cannot see dataset navigation and cannot fetch dataset APIs.
- Dataset directory and preview surfaces use standard searchable/paginated table behavior and mobile cards.
- Existing Operations dataset links continue to land on real dataset detail pages.

## Manual Test Plan

- Sign in as admin and verify `/datasets`, `/datasets/new`, `/datasets/{id}`, and `/datasets/{id}/edit`.
- Create a dataset from a published form, add fields, save it, and confirm preview rows render.
- Edit the dataset name/source/fields and confirm detail reflects the update.
- Sign in as scoped operator and verify only visible datasets are listed, and visible dataset previews include their full materialized output.
- Sign in as no-access user and verify dataset navigation/data access is unavailable.

## Automated Test Plan

- `cargo fmt --all`
- `.\scripts\validate.ps1`
- `cargo test -p tessara-api`
- `cargo test -p tessara-web`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo audit`
- `npm --prefix end2end test`
- `.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8080"`
- `.\scripts\smoke.ps1`
- `.\scripts\local-launch.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Add dataset native route module and route registrations.
2. Tighten form-version render authorization.
3. Build dataset directory with standard table/search/pagination/mobile patterns.
4. Build dataset detail with definition tables and preview.
5. Build admin-only create/edit forms.
6. Update Playwright/API permissions coverage and smoke/UAT assertions.
7. Run validation and deploy for review.

## Dependencies And Blockers

- Existing backend dataset APIs are the implementation baseline.
- Orpheum scenario support is optional unless the catalog is reconfigured; current `orpheum scenario list --json` returned no scenarios.
- Row filters and calculated fields are intentionally blocked to Sprint 3B.

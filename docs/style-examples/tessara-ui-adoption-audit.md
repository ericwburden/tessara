# Tessara UI Adoption Audit And Guardrails

This audit covers the Sprint 1G regression lane for:
- `1G-R1` Component adoption audit
- `1G-R2` `tessara-ui` component contract examples
- `1G-R3` Guardrail against new bespoke shared-surface patterns

It documents the real current state rather than the target state.

## Audit Summary

The current application has a stable shared page-shell layer, but not a fully adopted end-to-end component system.

What is solid:
- native shared shell framing is in place for `Home`, `Forms`, `Workflows`, and `Responses`
- those routes consistently use the shared `NativePage`, `PageHeader`, `MetadataStrip`, and `Panel` contracts
- navigation visibility is centrally driven from shared link specs in `native_shell.rs`

What is still mixed:
- route interiors still build many cards, action rows, and form layouts with page-local HTML strings
- `tessara-ui` includes lower-level card/field/toolbar helpers, but the native route layer has not fully converged on them
- `Organization` remains a compatibility surface built via `inner_html`, so it is not yet part of the native primitive adoption story

## Shared-Surface Adoption Matrix

| Surface | Current shell ownership | Shared primitives clearly adopted | Bespoke or incomplete areas | Replacement target |
| --- | --- | --- | --- | --- |
| `/app` Home | Native SSR | `NativePage`, `PageHeader`, `MetadataStrip`, `Panel`, shared nav visibility model | `home-card`, `directory-card`, and readiness `record-card` remain route-local | extract a native shared directory/status card primitive when Home or another route needs the same pattern again |
| `/app/forms*` | Native SSR | shared page shell primitives | list/detail/edit/create still render many `record-card`, `form-field`, `page-title-row`, and action rows in page-local helpers | promote reusable card, field-grid, and draft-workspace primitives before more form authoring surface expands |
| `/app/workflows*` | Native SSR | shared page shell primitives | assignment console and workflow detail/list internals still use page-local `record-card`, raw field grids, and inline action rows | extract shared assignment/form-grid patterns when workflow admin work grows |
| `/app/responses*` | Native SSR | shared page shell primitives | pending/draft/submitted cards, delegated context selector, and response edit form remain mostly page-local markup | extract response queue card and shared form-field wrappers as response lifecycle work expands |
| `/app/organization*` | Transitional `inner_html` surface | none at the native route layer yet | entire route family is still compatibility-owned | migrate to native SSR first, then adopt shared route-interior primitives |
| `/app/administration*`, `/app/reports*`, `/app/dashboards*`, `/app/migration` | Transitional compatibility surfaces | partial use of `tessara_ui` HTML helpers inside older builders | still builder-oriented and not suitable as a pattern source for new product UI | keep compatibility-only until explicitly migrated |

## Approved Patterns Versus Current Gaps

Approved today:
- native route shell via `NativePage`
- route summary via `PageHeader`
- route state via `MetadataStrip`
- section framing via `Panel`
- compatibility/helper primitives in `tessara-ui` for actions, cards, fields, checkbox fields, and toolbars

Known gaps:
- no fully adopted native shared `RecordCard` primitive
- no native shared field-grid primitive for authoring/editing surfaces
- no shared queue-card primitive for response/workflow status tiles
- no shared native toolbar wrapper for filter rows outside the lower-level HTML helper layer

Implication:
- new shared-surface work should not invent another visual system just because the native wrapper does not exist yet
- reuse the existing shared class families and document the missing primitive as the replacement target

## Guardrail Checklist For Future PRs

Use this checklist when a PR touches `Home`, `Organization`, `Forms`, `Workflows`, `Responses`, or another shared `/app` route.

- The route uses `NativePage`, `PageHeader`, `MetadataStrip`, and `Panel` if it is native SSR-owned.
- The change reuses an approved primitive when one exists.
- If no approved primitive exists, the change reuses an existing shared class family such as `record-card`, `form-field`, `button-link`, `page-title-row`, or the current shell/layout classes.
- The change does not introduce a new route-local card, field, toolbar, or page-shell pattern without documenting why.
- Any necessary exception is explained in the PR or issue with:
  - the reason the existing primitive was insufficient
  - the replacement target
  - the route or surface expected to adopt the extracted primitive next
- The change does not use `application.rs`, `inner_html`, or `/bridge/*` for new product-facing behavior unless it is an explicit compatibility shim already scheduled for deletion.

## Recommended Review Comment Template

Use this when new shared-surface markup drifts away from the approved contracts:

> This change adds a new shared-surface pattern instead of reusing the current Tessara UI primitives or shared class families. Please either switch this to an approved primitive, or document the exception with the replacement target so we do not deepen the parallel visual system.

## Evidence Used For This Audit

Reviewed source of truth:
- `crates/tessara-ui/src/lib.rs`
- `crates/tessara-web/src/features/native_shell.rs`
- `crates/tessara-web/src/features/home.rs`
- `crates/tessara-web/src/features/forms.rs`
- `crates/tessara-web/src/features/workflows.rs`
- `crates/tessara-web/src/features/responses.rs`
- `crates/tessara-web/src/features/organization.rs`

This audit is intentionally documentation-only. It does not claim primitive adoption that the codebase has not yet completed.

# Sprint 8C: Response Module Separation Slice

Status: kickoff planning complete; implementation authorized on 2026-08-23.

- Branch: `codex/sprint-8c`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-8c`
- Roadmap authority: `docs/roadmap.md`, Sprint 8C block
- Validation policy: `tessara-validation-v2`
- Implementation profile: `phase8-module-extraction`

## Sprint Summary, Outcome, And Authority

Sprint 8C replaces the Core-owned Response transition with one independently
deployed Response module. The module owns response start, draft persistence,
save, submit, scoped review, submitted-response export/materialization,
persistence, configuration, diagnostics, health, UI, routes, assets, bootstrap,
and lifecycle. It consumes typed FormVersion and Workflow context without
reading either provider's tables and exposes typed Response/source contracts
and ordered events to Dataset and Workflow consumers.

The user-visible outcome is deliberately conservative: the existing Response
directory, assignment-only start, draft/edit, submission, review,
configuration, and diagnostics workflows remain recognizable and behave the
same through the shared shell. A tester must be able to complete and review a
Response and observe its submitted output in Datasets while Core, Forms,
Workflows, Datasets, and Responses use no shared product database access.

The roadmap entry condition is satisfied by the independently completed
Validation Platform Foundation diversion recorded at the top of
`docs/progress-report.md`. Sprint 8C starts from clean `main` commit
`54f81fe95cf5df3a7124e79d693fc584ed42562f`; diversion evidence is not reused.

## Fixed Identities And Decisions

| Surface | Sprint 8C identity or rule |
| --- | --- |
| Module Definition | `tessara.responses` |
| Initial / intended release | `1.0.0` |
| Prior-compatible upgrade fixture | separately source-built `0.9.0` |
| Transition identity removed | `tessara.responses` |
| Selected Module Instance | owner-allocated and read back under logical key `module.responses.primary` |
| Runtime / migration identity | `responses-runtime` / `responses-migration` |
| Product database | isolated disposable PostgreSQL database `tessara_module_responses` |
| Owned resource | `tessara.responses.response`, Module Instance owned, contract `2.0.0` |
| Lifecycle contract | `tessara.responses.response-lifecycle` `2.0.0` |
| Dataset source contract | `tessara.responses.submitted-response-export` `1.0.0` at the Response instance |
| Workflow event contract | `tessara.responses.response-events` `1.0.0` |
| Workflow provider binding | `tessara.responses.workflow-context` -> `tessara.workflows.response-context` `1.0.0` |
| Forms provider binding | `tessara.responses.form-version` -> `tessara.forms.form-version-schema` `1.0.0` |
| Core provider bindings | signed Shell Context plus grant/scope and principal display contracts |
| Navigation | `tessara.responses.navigation`, `Main`, order hint `40` |
| Capabilities | retain `submissions:read_own`, `submissions:respond`, `submissions:manage` as public authorization vocabulary |
| Evidence root | ignored `artifacts/sprint-8c-closeout/` |

Tessara remains pre-production. Migration is forward-only and fresh-only:
Sprint 8C does not retain a production dual-write path, legacy table reader,
old Core API fallback, compatibility alias, or database-copy migration.

## Scope

### In scope

- Add `tessara-response-module` with its own manifest, executable, database
  baseline, migrations, runtime and migration principals, health/readiness,
  configuration, diagnostics, routes, assets, typed UI, bootstrap, and tests.
- Move the current `crates/tessara-api/src/submissions/`,
  `response_owner_actions.rs`, `response_export_provider.rs`, Response-specific
  demo/bootstrap logic, and Response product persistence/policy to that owner.
- Reuse and advance `tessara-responses-contract`; consumers import canonical
  contract types rather than local mirrors or untyped JSON.
- Replace Response reads of Forms and Workflows with authenticated, bounded,
  typed provider calls. Persist immutable FormVersion and Workflow context
  snapshots sufficient to render, validate, authorize, and diagnose a Response
  without provider storage access.
- Keep Response starts assignment-only. Workflow validates the exact active
  assignment and issues a one-use, audience-bound start context; direct legacy
  `/api/responses/start` remains rejected.
- Publish Response lifecycle events from a transactional monotonic outbox so
  Workflow can advance its step/instance without sharing Response storage.
- Keep the Dataset `submitted-response-export` cursor/page semantics introduced
  in Sprint 8B, but relocate the provider, state, signing identity, and storage
  to the Response module. Dataset changes only its bound provider instance and
  any canonical owner identity assertions.
- Provide Response summary, usage, operational status, and resource-observation
  actions for Core summary, Forms reverse usage, Operations, and generic typed
  resource consumers.
- Remove Core Response storage, handlers, routes, transition inventory,
  definition-specific navigation/policy branches, seed writes, and direct
  consumer queries.
- Rebuild the reference topology and acceptance fixtures from empty using
  owner bootstrap APIs and signed logical-key read-back.
- Add exact boundary, authorization, nondisclosure, replay/idempotency, event,
  outage, incompatibility, recovery, UI, deployed-smoke, and upgrade/rollback
  proof in the slice that introduces each behavior.

### Explicitly out of scope

- Workflow authoring/runtime extraction (Sprint 8D1) and Workflow branching or
  data-flow expansion (Sprint 8D2).
- Forms extraction (Sprint 8E), Dataset redesign, or a new response-builder UX.
- General-purpose event-bus infrastructure, streaming delivery, external
  connectors, multi-instance Response selection, or third-party packaging.
- Production data backfill, dual writes, live cutover, or retained compatibility
  with Core-owned `tessara.transition.response` references.
- Renaming the established `submissions:*` capability vocabulary or `/responses`
  product URLs merely for terminology consistency.

## Current-State Findings And Affected Components

- `crates/tessara-api/migrations/001_baseline.sql` owns `submission_status`,
  `submissions`, `submission_values`, `submission_value_multi`,
  `submission_audit_events`, `response_export_state`,
  `response_export_changes`, `response_owner_action_receipts`, their indexes,
  triggers, and export functions. Submission rows have direct foreign keys to
  Form, organization, Workflow assignment, instance, and step tables.
- `crates/tessara-api/src/submissions/` owns list/detail/save/submit/delete and
  assignment-option policy. `response_owner_actions.rs` owns administrative
  fixture mutations and `response_export_provider.rs` serves Dataset directly
  from the Core database.
- `crates/tessara-api/src/workflows/handlers.rs` creates Response rows and
  directly reads/updates them to coordinate Workflow step instances.
- `crates/tessara-api/src/composition/mod.rs` predicts Response IDs and writes
  Response tables during Core bootstrap. `crates/tessara-api/src/demo/responses.rs`
  does the same for demo data.
- `crates/tessara-api/src/app_summary.rs`, Forms usage guards, generic resource
  resolution, module catalog/destination/navigation code, smoke, UAT, and
  browser fixtures directly assume the Core Response owner.
- The browser routes are composed in `crates/tessara-web/src/routes/responses.rs`
  from typed feature views in `crates/tessara-web-responses`; Core currently
  owns the outer route/document and product HTTP calls.
- `crates/tessara-responses-contract` already defines the exact Dataset export
  v1 and Core owner-action fixtures. It is the canonical starting point, but
  Core-specific issuer/action contracts must become module-owned bootstrap and
  service contracts rather than be preserved as compatibility paths.
- `crates/tessara-dataset-module` consumes the export through binding
  `tessara.datasets.response-export`; its cursor, fixed-bound paging, atomic
  promotion, tombstone, rebase, and last-good semantics are retained.
- Existing acceptance authority includes the literal manifest identities
  `response edit route follows ownership and delegation permissions`,
  `submission management combines scope with response ownership`, and
  `response start options are assignment-only`, plus Response shell and route
  assertions in `permissions.spec.ts`, `workflow-mediated-assignments.spec.ts`,
  `scripts/smoke.ps1`, and `scripts/uat-sprint.ps1`.

## Complete Core Subtraction Inventory

| Core surface | Required end state |
| --- | --- |
| Schema | Remove the Response enum, seven product/export/receipt tables, Response indexes, triggers/functions, and cross-owner foreign keys from the fresh Core baseline. Workflow keeps only opaque typed Response IDs where coordination requires them. |
| API and policy | Remove `submissions`, `response_owner_actions`, and `response_export_provider` modules/routes plus their SQL repositories and Response-specific validation. The generic module gateway is the only Core product route path. |
| UI and assets | Remove Core `/responses*` native route composition and Response product selectors/assets. The root app retains only generic gateway/shell machinery. |
| DTOs/contracts | Remove Core-local Response DTOs and old Core-owned `tessara.transition.response` construction. Canonical types live in `tessara-responses-contract`. |
| Workflow coupling | Remove direct Workflow reads/writes/FKs to Response product rows. Use typed start context, opaque Response reference, and idempotent Response lifecycle events. |
| Forms coupling | Remove direct submission queries used by Form rendering/usage policy. Use typed FormVersion provider calls and Response usage results. |
| Summary/operations/resources | Replace direct submission counts, readiness queries, and `reference_lifecycle_if_accessible` with Response summary/status/resource-observation calls and explicit unavailable states. |
| Bootstrap/demo | Remove `CoreBootstrapResponseV1`, predicted Response UUIDs, `materialize_core_bootstrap_responses`, and demo writes. Generic composition passes logical fixture input to the Response owner and consumes signed read-back. |
| Inventory/navigation | Delete transition fixture/catalog entry and Core static Response navigation entry after enrolling the real module exactly once. Capabilities are declared by the module and projected generically. |
| Credentials/grants | Core, Forms, Workflows, and Datasets receive no Response database credential. Remove static Core product grants/definition branches while retaining generic authorization exchange. |
| Compatibility | Old Core routes, payload aliases, transition resource types, and Core-provider export identity survive only as explicit negative tests or historical documentation. |

Every remaining `response`, `submission`, `tessara.responses`, or
`tessara.transition.response` match in Core/root-web after subtraction must be
classified by the boundary target as generic registration, controlled consumer
contract, historical fixture/documentation, or negative assertion.

## Provider And Consumer Edge Inventory

| Direction / edge | Current coupling | Target contract and behavior |
| --- | --- | --- |
| Workflow -> Response start | direct assignment, workflow, Form and submission table transaction | `tessara.workflows.response-context` v1 issues exact assignment, WorkflowVersion, step, node, actor/delegation, FormVersion reference, revision/digest, expiry, and one-use start authority. Unknown/inactive/completed/out-of-scope assignments are nondisclosing and cannot create a draft. |
| Forms -> Response render/validate | direct FormVersion/field/section queries and shared FKs | existing `tessara.forms.form-version-schema` v1 provides immutable schema/layout/options/content digest. Response stores the authenticated snapshot; provider outage blocks a new start but existing pinned drafts remain readable/editable and validate locally. |
| Core auth/scope -> Response | Core `AccountContext`, hierarchy queries, shared account/node FKs | signed Shell Context plus audience/action/raw-body-bound grants; opaque node/account IDs and bounded display snapshots only. Scope is checked at the provider and Response owner; known and random forbidden IDs are indistinguishable. |
| Response -> Workflow | direct update of workflow step/instance tables | `tessara.responses.response-events` v1 monotonic outbox: `started`, `draft_saved`, `submitted`, `deleted` with exact Workflow context and Response reference. Workflow consumes idempotently; duplicates/reorder/stale events do not double-advance. |
| Response -> Dataset | Core-hosted export provider | unchanged `submitted-response-export` v1 checkpoint/start/page contract served and signed by the Response Module Instance. Existing opaque cursor, fixed upper bound, tombstones, expiry/rebase, scope and last-good rules remain exact. |
| Response -> Forms reverse usage | direct submission query in Forms | Response `form-version-usage` v1 returns authorized Response references/count semantics and `available|empty|unavailable|undisclosed`; outage never impersonates empty. |
| Response -> Core summary | direct `submissions` counts | Response `summary` v1 returns scoped draft/submitted counts plus availability; unavailable is not numeric zero. |
| Response -> Operations | direct Core/analytics status | Response `operations-status` v1 exposes owner readiness, binding compatibility, event/export health, and sanitized stable findings without secrets or raw cursors. |
| Response -> generic resource consumers | Core transition table lookup | Response `resolve` v1 returns canonical resolution/observation for `tessara.responses.response` v2; owner/type/version validation precedes lookup and authorization remains nondisclosing. |
| Browser -> Response | Core document/routes and `/api/submissions*` | module-owned documents/assets and same-origin gateway routes; browser calls only Response-owned public endpoints and generic shell/navigation endpoints, never Forms/Workflow APIs directly. |
| Supervisor/composition -> Response | no independent owner; Core seed writes | generic manifest/release/instance apply, health/configuration/diagnostics/bootstrap/read-back, with no Response definition branch in Supervisor or Core. |

## Target Ownership And UI Inventory

| Surface | Canonical owner |
| --- | --- |
| Complete document, reset, tokens, theme, shell, grouped navigation, generic controls/icons | `tessara-module-ui` |
| Response directory/start/detail/edit markup and state | `tessara-response-module` using typed SDK/Leptos construction |
| Product CSS | Response module asset, namespace-rooted under `.tessara-response`; no shell/reset/token selectors |
| Browser API client and DTOs | Response module importing `tessara-responses-contract` |
| SSR, hydration, direct refresh, lifecycle navigation, title/active state | shared SDK adapters plus Response route declarations |
| Responsive behavior | accepted current layouts at 1440x1000, 1024x1366, and 390x844, including 200% zoom/overflow |
| Configuration and diagnostics | Response manifest/schema and module-owned read-only status projection; Core renders generic Module Management |
| Routes and assets | Response manifest/release catalog; generic Core gateway only |

### Accepted pre-extraction visual and interaction baseline

The accepted baseline is clean-main behavior at `/responses`, `/responses/new`,
`/responses/{id}`, and `/responses/{id}/edit`, represented by
`crates/tessara-web/src/routes/responses.rs`, `crates/tessara-web-responses`, the
three literal acceptance identities named above, and the current Response route
assertions in `permissions.spec.ts` and `workflow-mediated-assignments.spec.ts`.
Before product UI moves, S1 captures and hashes populated, empty, loading,
validation-error, unauthorized/nondisclosing, draft, submitted, delegated,
provider-degraded, and configuration/diagnostic states under
`docs/audits/sprint-8c-response-ui-baseline/baseline-index.json` for light/dark,
desktop/tablet/mobile, JavaScript-disabled SSR, direct refresh, hydration,
stored/system theme, 200% zoom, accessibility, clean console, and no external
asset traffic. That source-exact index is acceptance authority, not a visual
redesign brief.

The focused `ui-sdk-conformance` target must pass before consumer cutover and
must prove identical direct-document and lifecycle destinations, canonical
shell/sidebar/icon components, title/navigation state, responsive layout,
accessible controls, product-only namespaced CSS, clean hydration/console, and
absence of raw module HTML/DOM construction or Core-owned Response styling.

## Data, Events, And Cross-Owner State Machines

The Response database owns Response aggregates, values, audit history,
idempotency receipts, source snapshots, event/export outboxes, bootstrap
receipts, configuration, and migration history. Provider and consumer IDs are
opaque typed references with authenticated metadata; no cross-database foreign
key is allowed.

One owner transaction creates or mutates a Response, appends its audit record,
updates the final submitted-export envelope when applicable, and appends the
Workflow event. Published outbox sequence is monotonic and transactionally
committed. Start/page reads bind provider epoch, consumer/audience, scope,
contract, start cursor, fixed upper bound and page digest. Hidden attempt/page
checkpoints are never product-visible. Publication is append-only; consumers
own their committed cursors and apply idempotently. Cursor expiry starts an
authenticated full snapshot/rebase while preserving the prior last-good
consumer publication until atomic promotion.

The Response aggregate pins immutable FormVersion schema and Workflow start
context digests. A new start requires live compatible providers. Existing
draft display/save uses its pinned snapshot; submit revalidates local rules and
commits the Response plus Workflow event even if Workflow is temporarily down.
Workflow retry consumes the durable event later. A permanent incompatible
Workflow binding blocks new starts and is surfaced in readiness/diagnostics;
it cannot corrupt or hide existing Responses. Dataset export remains readable
from the Response owner during unrelated Workflow outage.

## Fresh Materialization And Canonical Fixture Graph

Disposable databases/processes: Core, Supervisor, Forms/Core-transition,
Workflow/Core-transition, Response module, Dataset module, Component module,
Dashboard module, and their isolated proxies. The exact profile is
`deploy/sprint-8c/compose.yaml` including the retained Sprint 8B topology and a
Response override.

Owner bootstrap order:

1. Core installation, accounts, roles, organization scopes, module control
   plane and generic authorization exchange.
2. Forms owner creates published FormVersion fixtures and returns signed
   logical-key -> typed-reference/schema read-back.
3. Workflow owner creates WorkflowVersion, steps and assignments from those
   FormVersion references and returns signed Workflow/assignment context.
4. Response owner creates selected release/instance, configuration and
   Response fixtures through its bootstrap action using only authenticated
   read-back from steps 1-3; it returns signed Response references and outbox
   positions.
5. Dataset owner binds the actual Response provider instance and consumes the
   submitted export; Component and Dashboard then consume owner read-back.
6. Generic Core navigation, resource, summary, configuration and diagnostics
   projections are verified; gateway starts only after exact health succeeds.

Tracked `deploy/sprint-8c/fixtures/reference-fixture-contract.json` uses logical
keys, never predicted UUIDs or copied counts:

| Logical fixture | Owner and required meaning |
| --- | --- |
| `account.response.owner`, `.delegate`, `.manager`, `.outsider` | Core identities for ownership, delegation, scoped management and nondisclosure |
| `scope.response.primary`, `.sibling`, `.restricted` | Core scope boundaries |
| `form.response.primary/v1`, `.required/v1` | Forms published versions with text, choice, multi-value, required and optional fields |
| `workflow.response.primary/v1`, `assignment.owner`, `assignment.delegate` | Workflow exact active contexts and assignment-only start authority |
| `response.draft.owner`, `.submitted.owner`, `.submitted.delegated`, `.submitted.restricted` | Response lifecycle, review and export inventory |
| `dataset.response.primary`, `component.response.primary`, `dashboard.response.primary` | downstream output chain consuming signed owner references |

First apply proves exact owner allocation/read-back and health. An unchanged
second apply proves semantic no-op across topology, configuration, product
state, references, receipts and outbox positions, excluding only explicitly
declared observation timestamps. A one-shot bounded mid-Response-bootstrap
fault retains diagnostics, prevents gateway publication, removes exact partial
topology, and a new from-empty apply restores the canonical topology. No manual
evidence or database edit is permitted.

## Functional, Authorization, Lifecycle, And Operational Specifications

- Start options contain only active assignments accessible to the actor or an
  authorized delegation. Starting the same assignment/idempotency input
  replays one draft; an inactive, completed, substituted, unknown, expired, or
  out-of-scope context creates nothing and discloses nothing.
- Draft saves validate keys/types/required-on-submit rules against the pinned
  FormVersion snapshot. Save never submits; submit is one logical transition
  with a durable audit and outbox event. Submitted Responses cannot be edited
  or deleted through draft actions.
- `submissions:manage` remains scope-bounded; ownership/delegation never grants
  cross-scope management. Random and known forbidden IDs return indistinguishable
  product results. Provider errors are sanitized and bounded.
- Every persistent public/private mutation binds Module Instance, actor/service,
  grant JTI/action/audience, method/path, exact raw-body digest, and canonical
  `x-idempotency-key`. Same identity/input replays; any mismatch is rejected.
- Response configuration validates exact provider bindings and timeouts before
  apply. Readiness and diagnostics share a read-only owner projection covering
  database migration, provider compatibility, event/export publication, and
  last stable finding; process liveness alone is insufficient.
- Deployed health uses exact owner endpoint/status/content semantics. Redirects,
  broad success, wrong owner, stale release, or Core fallback fail.
- Upgrade `0.9.0 -> 1.0.0`, rollback `1.0.0 -> 0.9.0`, and restore `1.0.0`
  preserve the Module Instance, Response state and compatible contracts without
  restarting or changing unrelated owners. Rollback never reactivates Core.

## Acceptance Criteria

1. **AC-01 owner:** one independently deployed `tessara.responses` `1.0.0`
   instance owns its database, API, UI, health, configuration and diagnostics.
2. **AC-02 product flow:** authorized actors list, start, save, resume, submit,
   review and filter Responses with durable audit/lifecycle state; invalid
   transitions and malformed values fail without partial writes.
3. **AC-03 typed providers:** Response starts/rendering use exact authenticated
   FormVersion and Workflow context contracts with no provider table/credential
   access or local mirror DTOs.
4. **AC-04 Dataset source:** Dataset consumes the Response-instance export with
   exact cursor/page/tombstone/rebase semantics and no Core provider fallback.
5. **AC-05 Workflow events:** Workflow consumes exact Response events
   idempotently; duplicate, stale, reordered, unavailable and incompatible
   cases cannot double-advance or partially mutate Workflow state.
6. **AC-06 assignment-only start:** only a live authorized Workflow assignment
   can start a Response; the removed unmediated start path stays rejected.
7. **AC-07 scoped review:** owner/delegate/scoped manager see only authorized
   Responses; known and random unauthorized identities are nondisclosing.
8. **AC-08 fresh seed/isolation:** empty materialization uses owner bootstrap
   and signed read-back, with pairwise database/credential isolation and no
   predicted physical IDs.
9. **AC-09 authorization/replay/compatibility:** public and private actions
   enforce exact grant, raw-body, media type, size, correlation, idempotency,
   provider/consumer version and owner identity contracts.
10. **AC-10 outage/recovery:** independently faulted Forms, Workflow, Response,
    Dataset consumer or Core exchange produces bounded sanitized degradation,
    preserves last-good state, and converges after restoration.
11. **AC-11 UI parity:** Response entry, submission, review, configuration and
    diagnostics retain accepted information architecture, shell, theme,
    responsive, accessibility, SSR/hydration, title/navigation and clean-console
    behavior through canonical SDK ownership.
12. **AC-12 Core subtraction:** Core contains no active Response product schema,
    route, DTO, handler, adapter, export provider, seed write, static navigation,
    transition descriptor, product grant branch, or compatibility reader.
13. **AC-13 no-op:** unchanged source/configuration/bootstrap produces no
    semantic product, topology, identity, receipt or outbox change.
14. **AC-14 failed apply recovery:** deterministic failure is retained, partial
    topology is removed, and a clean successor apply restores exact health
    without manual repair.
15. **AC-15 independent upgrade/rollback:** Response alone upgrades, rolls back
    and restores with compatible state while unrelated owners remain unchanged.
16. **AC-16 roadmap exit:** a tester completes and reviews a Response and sees
    its submitted output in Dataset/Component/Dashboard through module contracts
    with no shared database access.

## Traceability Matrix

| Roadmap clause | Specifications / AC | Slice | Implementation targets | Formal lanes / manual UAT |
| --- | --- | --- | --- | --- |
| Independently deployed; no shared persistence | identities, isolation, AC-01/08/12 | S2, S4, S5 | `owner-product`, `core-subtraction`, `migration-seed`, `clean-materialization` | readiness materialization; rehearsal materialization; SIT static/smoke; UAT owner/isolation |
| Move start, draft, save, submit, review, export, persistence, config, diagnostics | product/lifecycle/operations, AC-01/02 | S2-S3 | `owner-product`, `api-idempotency`, `deployed-smoke` | rehearsal Rust/browser/smoke; SIT Rust/browser/smoke; UAT product |
| Typed FormVersion and Workflow context; no tables | provider inventory, AC-03/06 | S1, S3 | `contract-boundary`, `provider-contracts`, `assignment-only` | readiness contract; rehearsal providers; SIT Rust/smoke; UAT providers |
| Typed source contracts/events for Dataset and Workflow | event/outbox and consumers, AC-04/05 | S1, S3 | `consumer-cutover`, `dataset-export`, `workflow-events` | rehearsal consumers/events; SIT Rust/smoke; UAT consumers |
| Assignment-only starts and scoped review | authorization, AC-06/07 | S3 | `assignment-only`, `scoped-review`, `fixture-acceptance` | rehearsal browser/conformance; SIT browser; UAT authorization |
| Rebuild empty via owner bootstrap; no Forms/Workflow/Dataset/Core storage | seed graph, AC-08/13/14 | S4-S5 | `migration-seed`, `clean-materialization`, `semantic-noop`, `failure-recovery` | readiness/rehearsal materialization; SIT smoke; UAT materialization/recovery |
| Authorization, idempotency, event, outage, compatibility coverage | security/state machines, AC-09/10 | S1-S3, S6 | `contract-boundary`, `api-idempotency`, `workflow-events`, `failure-recovery` | rehearsal conformance/providers/recovery; SIT Rust/smoke; UAT replay/outage |
| Unchanged application UI through shared shell | UI baseline/ownership, AC-11 | S1-S3 | `ui-sdk-conformance`, `fixture-acceptance` | rehearsal browser/conformance; SIT browser; UAT UI |
| Complete/review Response and consume output in Dataset | exit chain, AC-16 | S3, S5-S6 | `consumer-cutover`, `deployed-smoke`, `uat-readiness` | rehearsal smoke/UAT; SIT browser/smoke; UAT end-to-end |

## Ordered Implementation Slices

### S1 — Freeze contracts, baseline, and executable subtraction inventory

Prerequisite: this validated planning package. Freeze Response v2 resource and
lifecycle types, Workflow context, Workflow event, Dataset export, reverse
consumer, summary/status and resource-observation contracts. Capture the
accepted UI baseline. Add failing boundary/subtraction tests for old owner/type,
Core schema/routes, direct SQL/credentials, local mirrors, unmediated starts,
duplicate inventory/navigation and definition-ID branches. Create target
selectors, zero-test guards, evidence schemas and planning-alignment checks.

Expected touchpoints: `tessara-responses-contract`, module-contract fixtures,
workspace manifests, architecture/baseline index, boundary/acceptance scripts,
contract and test-change log. Contract and harness tests change in this slice.

Complete when every boundary and removal item is executable and the baseline
is hashed before product source moves.

### S2 — Establish the independent Response owner and UI

Prerequisite: S1. Create module crate/database baseline/manifest/executable,
runtime and migration identities, configuration/status/diagnostics/health,
routes/assets, typed SDK views, bootstrap and isolated deployment. Implement
owner-local aggregate, audit, idempotency and outbox persistence. Generate and
check browser assets with
`pwsh -NoProfile -File scripts/build-module-ui-browser-assets.ps1 -Module all -Check`
and pass focused `ui-sdk-conformance` before traffic cutover.

Expected touchpoints: new Response module/migrations/manifest, shared contract
and UI crates, Cargo graph, `deploy/sprint-8c`, asset and module tests.

Complete when the owner builds, migrates, renders, configures and diagnoses
independently without serving production traffic.

### S3 — Move product behavior and cut over every consumer

Prerequisite: S2. Move start/list/detail/save/submit/delete/review and export to
the module. Implement authenticated Form/Workflow/Core provider calls, pinned
snapshots, Workflow outbox/events, Dataset export, reverse usage, summary,
operations and resource observation. Cut browser, Dataset, Workflow, Forms,
Core summary/operations/resources, fixtures, smoke and acceptance in the same
slice. Prove nondisclosure, exact scope, idempotency, incompatibility, timeout,
outage and recovery across real processes.

Expected touchpoints: Response owner/contracts, controlled consumers, generic
gateway/service bindings, deployment, fixtures, Rust integration tests,
Playwright, smoke and UAT predicates.

Complete when no first-party consumer needs the Core Response implementation,
tables, credentials, transition reference or old payload.

### S4 — Remove Core ownership and duplicate inventory

Prerequisite: S3. Delete Core schema, modules, routes, web composition, seed
writes, transition descriptor, static navigation/capability branches and
compatibility readers. Convert Workflow linkage to opaque typed references and
event consumption. Enroll one Response instance and prove exact inventory,
navigation, negative old contracts and pairwise database isolation.

Expected touchpoints: Core baseline/API/web/module catalog, Workflow consumer,
composition/demo, boundary tests and acceptance inventory.

Complete when automated search, schema, runtime inventory and navigation show
one owner and no fallback.

### S5 — Rebuild from empty, no-op, failure recovery, and upgrade/rollback

Prerequisite: S4. Add owner-ordered logical fixtures and source-exact materializer.
Prove first apply, signed read-back, downstream Dataset/Component/Dashboard
materialization, semantic no-op, deterministic partial failure/teardown/from-
empty recovery, and Response-only source-built `0.9.0` upgrade/rollback/restore.

Expected touchpoints: `deploy/sprint-8c`, materialization/fixture/fault/upgrade
scripts, Compose, smoke and clean-environment integration tests.

Complete when clean disposable environments prove all four lifecycle paths and
canonical restoration without manual repair.

### S6 — Close the implementation acceptance cone

Prerequisite: S5. Run every required implementation target, full formatting,
compile/Clippy with warnings denied, workspace tests, Playwright inventory,
deployed smoke and automated UAT predicates. Produce a clean-source, contract-
hash-bound passing `implementation-readiness-result.json` with zero known
failures and no open defect-provenance record.

Expected touchpoints: implementation/readiness runners and evidence publication
only as needed to make the already-defined targets executable; behavior and its
harness remain paired.

Complete when formal validation has no known product, harness, fixture,
environment, acceptance or evidence-contract defect left to discover.

## Proof-Class-To-Command Matrix

Every command is a future implementation-stage command. Targets are required;
clean materialization, semantic no-op and failure recovery run in authenticated
clean disposable environments.

| Target | Proof classes | Exact command | Clean |
| --- | --- | --- | --- |
| `static-quality` | static-quality | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target static-quality` | No |
| `contract-boundary` | contract-boundary | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target contract-boundary` | No |
| `owner-product` | owner-product | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target owner-product` | No |
| `ui-sdk-conformance` | ui-sdk-conformance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target ui-sdk-conformance` | Yes |
| `consumer-cutover` | consumer-cutover | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target consumer-cutover` | No |
| `core-subtraction` | core-subtraction | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target core-subtraction` | No |
| `inventory-navigation` | inventory-navigation | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target inventory-navigation` | No |
| `migration-seed` | migration-seed | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target migration-seed` | Yes |
| `clean-materialization` | clean-materialization | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target clean-materialization` | Yes |
| `semantic-noop` | semantic-noop | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target semantic-noop` | Yes |
| `failure-recovery` | failure-recovery | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target failure-recovery` | Yes |
| `fixture-acceptance` | fixture-acceptance | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target fixture-acceptance` | No |
| `runner-selftest` | runner-selftest | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target runner-selftest` | No |
| `deployed-smoke` | deployed-smoke | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target deployed-smoke` | No |
| `independent-upgrade-rollback` | independent-upgrade-rollback | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target independent-upgrade-rollback` | Yes |
| `uat-readiness` | uat-readiness | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target uat-readiness` | No |
| `provider-contracts` | contract-boundary, owner-product | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target provider-contracts` | No |
| `workflow-events` | contract-boundary, consumer-cutover | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target workflow-events` | Yes |
| `dataset-export` | contract-boundary, consumer-cutover | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target dataset-export` | Yes |
| `assignment-only` | owner-product, contract-boundary | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target assignment-only` | No |
| `scoped-review` | owner-product, contract-boundary | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target scoped-review` | No |
| `api-idempotency` | contract-boundary, owner-product | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target api-idempotency` | No |
| `planning-contract-alignment` | contract-boundary | `.\scripts\run-sprint-8c-implementation-readiness.ps1 -Target planning-contract-alignment` | No |

## Automated, Integration, Smoke, And UAT Plan

Baseline future commands remain required:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --locked`
- `npm --prefix .\end2end test`
- `.\scripts\local-launch.ps1`
- `.\scripts\smoke.ps1`
- `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

Sprint-specific runners will provide exact zero-test-guarded targets for
contract fixtures, owner database integration, Form/Workflow providers,
Workflow event idempotency/order, Dataset export compatibility, scope and
nondisclosure, browser network ownership, UI visual parity, Core subtraction,
materialization/no-op/recovery, runner self-tests, deployed smoke, and upgrade/
rollback. The full Playwright manifest runs one worker, zero retries, no skip,
fixme, only, filter, max-failure truncation, or snapshot-update mode. Any test
identity/expectation change requires `docs/sprints/sprint-8c-test-change-log.md`
with governing authority and equal-or-stronger replacement proof.

Manual UAT scenarios are seeded in `sprint-8c-verification.md`: product/UI,
assignment and provider contracts, scoped review, Dataset/Workflow consumers,
configuration/diagnostics, fresh materialization/no-op, outage/recovery, Core
subtraction/isolation, failure recovery, upgrade/rollback, and complete roadmap
exit. Scripted UAT runs only after authoritative SIT passes.

## Validation, Evidence, Freeze, And Failure Plan

Implementation first publishes a passing non-authoritative
`implementation-readiness-result.json` for a clean source and exact validation-
contract hash. Validation Readiness and Candidate Rehearsal then execute or
inherit only authenticated dependency-complete lanes; uncertainty or an
unmapped path selects conservative execution. Preflight authenticates compact
certificates and freezes one exact candidate. SIT runs complete static, Rust,
Playwright and deployed-smoke lanes. Only passing SIT authorizes complete
scripted/manual UAT. A successor candidate always reruns complete SIT and UAT.

Each phase publishes a sealed phase-local `evidence-index.json`; downstream
consumers use compact certificates and hashes. Raw evidence remains cold under
`artifacts/sprint-8c-closeout/`. The coordinator publishes `evidence-chain.json`
and closeout performs one full raw-integrity audit.

Every failed implementation target, lane or scenario retains its raw evidence
and a schema-valid sibling `defect-provenance.json` before correction or broad
rerun. Automation may classify, invalidate, route and block reruns; it may not
edit tests, change expected values, or decide an assertion is obsolete. The
record binds whether assertions started, product/process/mixed origin,
implementation-exit gap, expectation-change authority, focused reproducer and
minimum safe invalidation. After candidate-changing correction, complete
Readiness/Rehearsal precede a successor freeze and complete SIT/UAT. A post-SIT
UAT defect follows the coordinator-owned diagnostic harvest, consolidated
batch, impact assessment, focused repair proof, canonical restoration and final
certification entry; focused proof never authorizes closeout.

## Rollout, Recovery, Compatibility, And Rollback

Rollout is source-exact and offline against disposable databases. Apply stops
before gateway publication on any owner/bootstrap/health mismatch. Recovery
removes only the authenticated partial Sprint 8C topology and reapplies from
empty. No cleanup command trusts an unauthenticated on-disk process, Compose or
database identity. Current public URLs and capability names remain stable;
the old Core owner/type/wire path is intentionally incompatible and rejected.

Release rollback is module-only and requires a real independently built
compatible `0.9.0`, not a relabeled candidate image. It preserves instance and
product state and proves Core, Forms, Workflow, Dataset, Component, Dashboard,
their data, image identities, restart counts, navigation and availability are
unchanged. Intended `1.0.0` is restored before handoff.

## Risks And Controls

| Risk | Prevention | Detection | Recovery |
| --- | --- | --- | --- |
| Hidden Core or Workflow table coupling | executable bidirectional inventory; no cross-owner credentials/FKs | boundary search, schema and pairwise isolation targets | return to S1/S3; remove coupling before cutover |
| Assignment-only policy weakens | one-use Workflow context and exact assignment state | negative direct-start, inactive/completed/substituted/delegation tests | reject draft; restore provider binding/context |
| Event duplication/reorder advances Workflow twice | transactional monotonic outbox and consumer idempotency/CAS | duplicate, stale, reorder, interruption and replay matrix | retain committed Response; replay from last Workflow cursor |
| Dataset export changes during relocation | preserve v1 bytes/media/cursor semantics; advance provider identity only | contract fixtures plus real Dataset refresh/rebase tests | keep last-good Dataset publication; correct Response provider |
| Provider outage destroys draft usability | pinned immutable source snapshots | isolated provider fault matrix and browser tests | serve existing draft snapshot; block only actions needing live provider |
| Scoped review leaks existence | owner/provider scope checks and nondisclosing result mapping | known/random/disjoint scope equality tests | remove leaking field/path and rerun full authorization cone |
| UI drift or Core CSS remains | accepted baseline, SDK ownership map, early conformance | visual/SSR/hydration/a11y/console/network matrix | correct SDK/product CSS before consumer cutover |
| Duplicate transition/enrollment | delete transition before enrollment projection | exact inventory/navigation source/runtime checks | remove duplicate and rematerialize cleanly |
| Copied UUID/count fixture drift | logical keys and signed owner read-back | fixture digest/no-op/negative substitution tests | rebuild from empty; never patch IDs |
| Partial materialization or unsafe cleanup | authenticated topology claims, fail-closed gateway, bounded fault | retained failure/teardown/residue/health proof | exact teardown and successor from-empty apply |
| Validation discovers a known gap first | mandatory implementation targets and zero-failure exit | contract-bound readiness gate | classify exit gap, return to implementation, recertify |

## Assumptions, Open Questions, Dependencies, And Blockers

Decisions fixed by repository evidence and this plan:

- The Dataset export v1 semantics remain current; only provider ownership and
  deployment binding move.
- Response lifecycle events use a module-owned transactional outbox and typed
  checkpoint/page delivery, not a new general event platform.
- Existing drafts use pinned source snapshots during provider outage; new
  starts require live compatible Form and Workflow providers.
- Workflow remains a Core-hosted transition consumer until Sprint 8D1 but may
  not access Response tables or credentials after cutover.
- Product URLs and `submissions:*` capabilities remain stable; ownership and
  contracts change without an intentional UI redesign.

The user explicitly approved on 2026-08-23 that (1) Response submission commits
locally and durably queues the Workflow event when Workflow is unavailable,
with eventual idempotent delivery, and (2) existing drafts remain readable and
editable from pinned source snapshots while new starts require live compatible
Form and Workflow providers. The same approval confirmed the pre-production
fresh-reset policy: no existing Response data is migrated; disposable databases
are cleared and reseeded after the owner structures and bootstrap contracts are
settled.

Product blockers: none at kickoff. If implementation finds a Response rule
that cannot be expressed without provider storage access, a missing reverse or
operational consumer, an unbounded event/outbox state, or an ambiguous UI owner,
it must update the inventories and validation contract before cutover. If that
requires moving Forms/Workflow product policy rather than exposing a narrow
contract, implementation pauses for user direction.

Dependencies: current Module SDK/runtime/testkit and generic control plane;
Sprint 8B Dataset export consumer; Forms schema provider; Core authorization,
scope and principal providers; validation platform release 2; Docker,
PostgreSQL, Rust/Node/Playwright toolchains and disposable ports/databases.

## Planning Audit And Implementation Handoff

- All nine roadmap clauses map through specifications, acceptance criteria,
  ordered slices, exact implementation targets, formal lanes and manual UAT.
- The Core subtraction, provider/consumer, ownership, seed/fixture, cached-state,
  UI baseline and proof-class inventories are explicit.
- UI, API, persistence, authorization, integration, deployment, observability,
  compatibility, recovery and rollback are covered; production data migration
  is explicitly inapplicable because Tessara is pre-production and fresh-only.
- Harness, fixture, smoke, Playwright, deployment and UAT changes are paired
  with the product slice that makes them stale.
- Validation policy v2, defect provenance, compact certificates, candidate
  freeze, downstream invalidation and final evidence-chain audit are planned.
- Kickoff writes are confined to the Sprint 8C worktree. `main` remains clean.
  No product source, test, migration, fixture, script, manifest, deployment
  configuration or generated product asset changed during kickoff.

Authorized first implementation slice: **S1 — freeze canonical Response v2,
Workflow-context/event and reverse-consumer contracts; capture the source-exact
UI baseline; and turn the complete Core/subscriber subtraction inventory into
executable boundary tests.**

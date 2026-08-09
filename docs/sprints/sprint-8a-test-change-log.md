# Sprint 8A Test Change Log

This log records changes to pre-existing test expectations and accepted
fixtures during Sprint 8A. A failing test does not authorize changing its
expectation: every removal or changed fixture below is tied to the approved
forward-only owner cutover and names equivalent or stronger proof.

## 2026-08-06 — Core capability and built-in-role owner cutover

- Core startup no longer inserts `components:read`, `components:manage`,
  `dashboards:read`, or `dashboards:manage`, and its built-in `operator` role no
  longer receives Component or Dashboard grants. The enrolled Component and
  Dashboard Manifests own those capability rows; composition Blueprint role
  declarations own module-specific grants.
- The Core baseline retains the generic `core_module_action_declarations`
  projection table but no longer inserts Scoped Records, Dashboard, or
  Dashboard-to-Component action rows. Release enrollment projects those exact
  declarations from each enrolled Manifest.
- The current Core-owned membership contract is
  `sprint-8a-role-capabilities-v1+sha256.4f607b6f428c`, with canonical SHA-256
  `4f607b6f428c0de70901dd119f7026b4c700c9e86309e76a3f5085a4da366609`:
  `admin = [admin:all]`; `operator = [hierarchy:read, forms:read,
  workflows:read, workflows:manage, submissions:respond,
  submissions:manage, operations:view, datasets:read]`; and
  `respondent = [submissions:read_own, submissions:respond]`.
- `sprint_6a_populated_upgrade` continues to reproduce the exact historical
  Sprint 5A capability catalog and operator grants before startup. Its current
  post-startup expectation uses the Sprint 8A contract, and its injected
  missing-current-membership case now removes `datasets:read` rather than the
  retired Core-owned `dashboards:read` grant.
- Deployment evidence keeps the frozen Sprint 6A seed identity as the default
  and selects the Sprint 8A identity only with transition-catalog profile
  `sprint-8a`. Database-free self-tests prove both identities,
  composition-owned projection, and rejection in both cross-profile
  directions.

Equivalent or stronger proof is the digest-coupled Core unit contract and
fresh-baseline identity,
`core_seed_catalog_excludes_independent_module_capabilities`, the historical
fixture precondition and exact current-set upgrade assertions, explicit absence
of static module action declarations, Manifest enrollment projection tests,
Blueprint contract checks, and the profile-specific deployment-evidence
self-test. This is an implementation, migration, and harness change, so any
prior mutable readiness or rehearsal receipt is superseded.

## 2026-08-06 — Removed ignored Core Component demo monolith

The permanently ignored
`demo_seed_uses_capability_scope_ownership_and_components` integration test was
deleted. It exercised Component storage, seeding, and product routes through the
old Core API owner, which Sprint 8A removes, and retaining it as an ignored test
would preserve neither executable evidence nor a supported contract.

The replacement proof is owner-exact rather than count-based:

- `tessara-component-module/tests/product_integration.rs` exercises Component
  CRUD, versions, lifecycle, rendering, scope authorization, and
  nondisclosure through the extracted module.
- Component and Dashboard Manifest/bootstrap unit tests pin their release,
  capability, schema, placement, and dependency contracts.
- Core tests assert absence of independent-module capability seeding and
  Component product storage; source/boundary audits assert absence of Core
  routes and implementation.
- Source-exact materialization, deployed smoke, Playwright acceptance, and
  UAT-8A-01/02/05/06 provide the cross-container and user-visible proof.

No timeout, retry, filter, early return, or replacement ignored test was added.

## 2026-08-06 — Extracted product asset ownership and SSR identities

The root `style/main.css` still contained Dashboard and Component product
selectors after both products became independently deployed modules. Those
rules were removed from the Core asset; generic Module Management diagnostics
use neutral `module-*` selectors, and an executable acceptance audit rejects
either extracted product's selector prefixes in the root stylesheet.

Component's owner-served visual renderer now emits the established
`component-visual-preview` identity from its digest-pinned module JavaScript
and stylesheet. The existing browser assertion was retained. The stale
JavaScript-disabled route expectations in `permissions.spec.ts` were changed
from pre-extraction client-loading placeholders (`Loading components`,
`Loading configuration`, and `Loading component`) to the exact headings and
resource identities already present in the module-owned complete SSR
documents. No route, viewport, refresh, no-JavaScript, document-root, table,
or visual assertion was removed.

Equivalent or stronger proof is the Component asset digest unit test, the
Manifest-to-source asset audit, the root product-selector absence audit, exact
module document-root checks on direct load and refresh, the retained visual
preview assertion, and UAT-8A-01/06. These are product asset and acceptance
inventory changes, so all earlier mutable validation receipts remain
superseded.

## 2026-08-06 — Restored the accepted Component product UI contract

The first extracted-Component browser rewrite incorrectly treated reduced UI
behavior as an approved expectation. In particular, it asserted that the
`Displayed Fields` editor and mobile preview action were absent and replaced
the accepted structured kind editors with a raw `Configuration JSON` textarea.
Sprint 8A authorizes an ownership and wire-contract cutover, not a product UI
redesign; those expectations contradicted AC-07, AC-15, and the explicit plan
decision to preserve same-origin URLs and accepted UI behavior.

`end2end/tests/components.spec.ts` now keeps the extracted Component v3 and
Dataset v1 API fixture shapes while restoring the accepted pre-extraction
`37aa9c8` behavior assertions:

- table authoring exposes `Displayed Fields`, `Available fields`, and no raw
  `Configuration JSON` editor;
- every supported kind exposes its structured controls, including Fields &
  Calculation, missing-value policies, comparison layout, axis titles,
  distinct-value-backed category labels, line smoothing/point limits, and Stat
  Card panel settings;
- validation-dependent live preview reports `Needs attention`, retains the
  unsaved editor after a rejected save, recovers to `Valid config`, and renders
  an SVG preview; the recovered draft saves through the one canonical
  `POST /api/admin/components/save` boundary;
- the Component directory supports name search, kind and publication-status
  filters, a keyboard-addressable mobile `Component filters` dialog with
  `Clear All`, mobile cards, and a hidden desktop table at the narrow viewport;
- the table detail and viewer routes expose row search, visible-column
  selection, reset, and the rendered table without folding version history
  into the detail surface; bar, line, pie, and donut viewers expose
  `.component-d3-svg`, while Stat Card retains its semantic card;
- the 390 px editor places Component Kind before Filters, has no horizontal
  overflow, opens a focused `Component preview` dialog, closes it with Escape,
  restores focus to `Open preview`, survives direct reload, and emits no
  browser console or hydration error.

The Component tests no longer use a serial-failure suite boundary. Repository
execution remains single-worker and state-safe, while an independent table,
visual, or mobile failure no longer prevents the remaining Component scenarios
from producing diagnostic evidence. This is a candidate-affecting acceptance
inventory correction, so mutable receipts produced against the reduced suite
are superseded.

The parity correction was subsequently strengthened with behavior-specific
proof rather than copied counts. Component module tests now distinguish the
exact manager and reader SSR projections: a fully authorized manager receives
draft-only definitions and lifecycle/edit controls in the complete document,
while a reader receives only published definitions and no management controls.
The existing JavaScript-disabled permissions scenario makes the same assertion
through the application gateway for both actors.

The Component browser scenario now also proves the searchable Dataset Version
picker's Dataset/Version/Grain/Tags/Provenance contract and field preview;
server-backed table filters, search clearing with a filter still active, column
projection refetch, and invalid-column rejection; slug-based links; Dataset
Version and Version Note history; confirmation for irreversible lifecycle
actions; consumer review plus required New Version Note; exact calculation
vocabulary and kind role surfaces; category/series override transitions; line
axis labels; bar legend order; and keyboard tooltips. Focused Rust unit tests
pin the corresponding closed config, provider, SSR, and asset behavior. No
formal readiness, rehearsal, SIT, or UAT result is claimed by these
implementation-phase checks.

## 2026-08-06 — Replaced Dashboard transition counts with exact ownership

The seven-to-six expectation correction was incomplete because Dashboard was
still represented both by Core's `transitional_in_process` catalog and by its
real enrolled Module Release/Instance. That was a product/architecture defect,
not permission to lower a copied count again.

Current expectations now assert exact identity and ownership:

- Core's transition catalog is exactly `tessara.forms`,
  `tessara.workflows`, `tessara.responses`, `tessara.datasets`, and
  `tessara.migration`;
- Dashboard is absent from canonical Core transition inputs and semantic
  destination resolution and appears exactly once through its enrolled
  Release/Instance and Manifest contribution;
- inventory rejects any transition/release identity overlap; and
- reference navigation pins Scoped Records, Components, and Dashboard to
  orders 7, 8, and 9 rather than inferring correctness from a total count.

Historical seven-entry fixtures remain explicitly historical. The retained
Dashboard duplication evidence stays classified as a product defect, and the
candidate-affecting identity correction supersedes all earlier mutable
readiness/rehearsal receipts.

## 2026-08-06 — Made lifecycle and release acceptance executable

The earlier Sprint 8A acceptance seed proved normal rendering but could not
execute the Dashboard lifecycle action contract: it had no inactive
ComponentVersion with a declared successor and no independent placements for
each destructive action. The generic upgrade fixture also treated a relabeled
candidate image as a compatible baseline, so a different image configuration
could pass without a distinct release binary or Module Release identity.

The accepted fixture is now identity-exact:

- seven Component shells produce eight ComponentVersions;
- the Stat Card `1.0.0` predecessor is `superseded`/`inactive` and names its
  published/active `2.0.0` successor;
- the reference Dashboard owns seven placements, including independent
  `lifecycle-upgrade`, `lifecycle-replace`, and `lifecycle-remove` placements
  bound to that predecessor; and
- the blocked-scope placement remains separate and must not appear in an
  authorized actor's finding or outage inventory.

UAT-8A-05 no longer passes from broad receipt labels. Its non-acceptance
diagnostic dependency must retain structured proof of the exact three initial
lifecycle findings, successor disclosure only for authorized findings, Defer
followed by Upgrade, independent Replace and Remove, the exact five remaining
authorized placements during Component outage, and healthy zero-finding
recovery. The diagnostic requires canonical seed restoration afterward.

Upgrade/rollback acceptance now compiles a distinct compatible Component
`0.9.0` release from the current source with its own executable, OCI, and
Manifest identities. The verifier applies exact Component-only Blueprint
deltas through the Supervisor and Compose adapter for baseline establishment,
`0.9.0` to `1.0.0` upgrade, rollback, and intended-`1.0.0` restoration. It
compares Component preservation and unrelated-owner identity before and after
each transition; relabeling the candidate is no longer accepted evidence.

Component Manifest expectations now also pin schema defaults `Components` and
`5` plus the actual `/usr/local/bin/component-module` runtime and migration
command path. Dependency diagnostics retain the selected Dataset binding,
provider/contract identity, compatibility, health, observation time, and stable
result/failure codes while excluding raw references and secrets.

These fixture, runner, Manifest, and acceptance-inventory changes are
candidate-affecting. All earlier mutable receipts remain superseded. No formal
Validation Readiness, Candidate Rehearsal, preflight, SIT, or UAT result is
claimed by this implementation correction.

## 2026-08-06 — Closed final implementation-entry contract gaps

The final plan-to-source audit found four defects that would have failed or
weakened the next testing entry:

- the Component Playwright suite still modeled the removed flat Dataset
  reference instead of the canonical nested Core-owned typed reference;
- the live product diagnostic invoked a Core-era demo UAT runner whose
  Component/Dashboard fixtures are intentionally absent from the Sprint 8A
  owner seed;
- a failing Dashboard semantic diagnostic emitted detailed fail-late JSON but
  its outer wrapper retained only the summary exception; and
- database uniqueness treated loopback aliases as different servers.

The correction parses the Dataset major from the canonical `<uuid>@<major>`
resource identity, uses Sprint 8A ownership smoke for the independent broad
product check, publishes inner semantic JSON plus a SHA-256 sidecar before any
failure is raised, declares that raw artifact in the rehearsal lane, and
embeds it in the outer consolidated harvest. Validation Readiness now
canonicalizes `localhost`, `127.0.0.1`, and `::1` before database uniqueness
comparison and self-tests that adversarial alias case. Copied Playwright count
labels were replaced by the acceptance-manifest identity.

A final owner-contract review then found that Core's typed-reference resolver
still accepted the obsolete `<uuid>:<major>` spelling even though the Dataset
contract and every Sprint 8A producer use `<uuid>@<major>`. It also found that
the Dataset contract exposed unsigned 32-bit majors while Core storage uses
positive signed 32-bit majors, forcing unchecked provider casts. The Dataset
contract now owns the exact positive `i32` boundary, Core accepts only the
canonical at-sign identity, and the provider carries the validated major
without casts. Boundary coverage accepts `i32::MAX` and rejects zero,
negative, alternate-spelling, extra-delimiter, colon-delimited, and
out-of-range identities.

These are candidate-affecting test, runner, evidence, and environment-contract
changes. They are implementation corrections only; no formal validation phase
was run.

## 2026-08-06 — Closed Readiness 33 harness and rehearsal-enforcement gaps

Validation Readiness attempt 33 ran its complete fail-late graph against clean
commit `0cf7dfa0`, retaining 12 passes, two failed checks, and zero blocked
checks. The failed runner-self-test log contains two independent parameter
contract failures: readiness still passed removed image arguments to the
source-built Component baseline and upgrade-verifier self-tests. The reset
dry-run also failed because the materializer accessed Compose's optional
network `external` property directly under strict mode.

One consolidated audit retained those three observed failures and found two
related protocol-enforcement gaps before source correction. Candidate
Rehearsal performed prerequisite and authenticated environment validation
before writing its start receipt, so a setup failure could consume an attempt
without terminal evidence or harvest. It also lacked the required optimized
known/random resource-reference latency proof; the debug workspace lane cannot
execute that release-only test.

The correction uses the current baseline-metadata/self-test contracts, handles
an omitted Compose `external` property as non-external in both safety
projections, makes the readiness prerequisite a receipt-governed terminal
lane, and declares the independent exact release-mode timing lane. Runner
self-tests and the Sprint 8A acceptance contract now enforce both protocol
properties. The five findings are classified `harness`; Candidate Rehearsal
29 remained Not Run. Because runners, acceptance enforcement, and validation
documentation changed, a fresh complete Readiness gate and complete Rehearsal
are required against the corrected commit and one shared environment identity.

## 2026-08-06 — Returned from testing for implementation completion

Testing was explicitly stopped after failed Validation Readiness attempt 33;
Candidate Rehearsal 29 did not start. A new plan-to-source audit treated the
stalled testing entry as an implementation problem and consolidated the
remaining corrections instead of continuing the lifecycle:

- Sprint 8A materialization keeps the public gateway stopped through first
  apply and semantic no-op, uses a loopback-only Core owner-control binding,
  and retains the exact boundary evidence before opening public ingress.
- The exact locked Component Manifest now declares bootstrap Dataset
  validation through the generic lockfile-owned seam: exact Manifest target and
  JSON Pointer, opaque canonical-inline payload selection, complete signed
  request binding, provider-owned Dataset semantics, one-use consumption, and
  no Component/Dataset branching in Core or Supervisor. Negative proof rejects
  missing or mismatched target/payload/apply/owner/audience/request, expiry,
  replay, incompatibility, and outage before any Component row or receipt is
  written.
- Component create uses the same authoritative idempotent replay behavior as
  save. Fresh create/save and bootstrap require the Dataset provider to echo
  the exact requested major-line reference and report a `ready`
  materialization; provider compatibility emits the distinct
  `materialization_not_ready` result before field evaluation. Mismatched,
  non-ready, incompatible, and unavailable Dataset paths leave zero Component
  rows, versions, mutation receipts, bootstrap receipts, or validation-pending
  drafts.
- Invalid configuration shapes remain validation failures; Module Management
  carries the module's sanitized Dataset dependency diagnostic instead of
  silently discarding it.
- Dashboard outage projection no longer invents disclosure authority. Provider
  observations, findings, action lookup, recovery, and repeat-outage reopening
  are isolated by the exact unexpired semantic authorization context so one
  actor/scope/revision episode cannot expose or mutate another.
- The Sprint 8A fixture path performs Core identity/RBAC setup but cannot write
  Dataset, Component, or Dashboard product tables. Its exclusion is enforced
  structurally from the PowerShell AST, and the unused Core-era Playwright SQL
  cleanup was removed.
- UAT semantic predicates evaluate authenticated raw JSON and reject divergence
  from embedded summaries. The product diagnostic is resolved from its
  authenticated lane, exact Dashboard placement-to-ComponentVersion identities
  are shared by smoke and tests, and Playwright inventory now includes the
  responsive/theme matrix, unsaved-state outage retry, and configuration/
  diagnostic authority scenarios.
- Candidate Rehearsal publishes unverified start evidence before fallible
  state/source/environment work. Readiness first reserves the requested number
  by authenticating state and pending authorization under the evidence-root
  exclusive lock, then publishes its unverified live receipt and immutable
  hashed start before source/environment assertions. Both checkpoint their live
  receipt, sidecar, and validation-state hash after every terminal check or
  block.
- Safe static rehearsal lanes remain independent of the fallible state and
  Readiness prerequisites. Failed rehearsal harvesting still produces exactly
  one batch; its correction authorization now permits one successor Readiness
  only, and that successor records append-only consumption bound to the failed
  predecessor and its own immutable start. Candidate Rehearsal accepts only the
  exact current Readiness path/hash and rejects prior-result or attempt reuse.
- Nested UAT semantic failures are projected with exact scenario/assertion
  identity, allowed classification, failure reason, and hashed raw evidence.
  Semantic-only failures do not also fail the outer projection lane, preventing
  one predicate from becoming both a product defect and a generic harness
  defect; an independent wrapper failure remains separately classifiable.

The governing protocol already required safe fail-late harvesting, exact
blocked reasons, and one consolidated batch. The closed enforcement gap was the
runner-owned state transition and evidence model: it previously lacked an
exclusive attempt lease, immutable/checkpointed Readiness evidence, executable
one-time correction consumption/current-Readiness selection, and exact nested
semantic defect projection.

All of these files are candidate-affecting and invalidate the Readiness 33
source. Only focused implementation checks are recorded for this batch. No new
Validation Readiness, Candidate Rehearsal, preflight, SIT, deployed Playwright,
or formal UAT result is claimed.

## 2026-08-06 — Consolidated Validation Readiness 34 correction

Validation Readiness attempt 34 completed all 15 declared checks against clean
commit `c920b8a3`, retaining 12 passes, two failures, and one exact prerequisite
block. `compose-database-contract` classified the missing six database URLs,
reset acknowledgement, and PostgreSQL client-container identity as one
`environment` defect. `reset-dry-run` classified the materializer's StrictMode
read of optional normalized-Compose properties as one `harness` defect.
`environment-contract` was blocked by `compose-database-contract`; no product
or destructive database action began. All receipt, state, raw-evidence, and
inventory hashes verified.

One batch, not per-failure micro-batches, is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-34-consolidated-correction-batch.json`.
The correction introduces one shared property-safe Compose service projection,
advances materialization to use it for ports and database bindings, hardens
adjacent optional resource-name reads, and adds a portless/environmentless
self-test enforced by Readiness and the acceptance contract. The successor
cycle must use six fresh pairwise-distinct loopback test databases and pass the
exact environment bindings in the same runner process.

These runner, acceptance-contract, and documentation changes are
candidate-affecting and supersede Readiness 34 for authorization. Candidate
Rehearsal 29 remains Not Run. Focused parsing, self-test, projection, contract,
and reset-WhatIf checks are correction evidence only; complete Readiness and
Candidate Rehearsal gates remain required before preflight.

## 2026-08-06 — Consolidated Validation Readiness 35 finalization correction

Validation Readiness attempt 35 executed all 15 declared checks against clean
commit `49c0ae73`; all 15 passed, with zero failed, blocked, or skipped checks.
All raw logs, produced artifacts, live receipts, state bindings, and sidecars
verified. The six-database environment, 74-test Playwright inventory, runner
self-tests, materializer WhatIf, and final clean-source proof all passed.

The gate then exited before terminal publication because StrictMode cannot
read `.classification` from the empty all-pass failure array. The retained live
receipt and validation state remain `executing`, and no canonical Readiness
result was published. Candidate Rehearsal 29 did not start. The process error
is retained at
`artifacts/sprint-8a-closeout/readiness-35/finalization-failure.log`.

One `evidence-finalization` finding, including the same latent success-path bug
in Candidate Rehearsal, is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-35-consolidated-correction-batch.json`.
The correction replaces empty-array member enumeration with one shared
StrictMode-safe classification projection and adds executable empty, single,
duplicate, missing, null, and multiple-result coverage. Readiness, Candidate
Rehearsal, and the acceptance contract all enforce that path.

The protocol already required atomic receipt finalization; missing executable
all-pass coverage was the actual enforcement gap. These tracked runner,
self-test, acceptance, and documentation changes are candidate-affecting and
supersede Readiness 35. A fresh complete Readiness and Candidate Rehearsal cycle
remains required before preflight.

## 2026-08-06 — Returned from Rehearsal 29 to implementation correction

Validation Readiness attempt 36 passed all 15 declared checks against clean
commit `3f7e32cb`, tree `252e651e`, and environment fingerprint
`2e235b070dd2f3663242fe5b3f983f561b834161ed65847ad6dd399d35e01da0`.
Candidate Rehearsal attempt 29 then started against that same identity and was
stopped by user direction before a terminal harvest. The retained snapshot is
8 passed lane receipts, 3 failed receipts, 1 interrupted optimized lane, 20
unexecuted checks, and 0 recorded blocked checks. The unexecuted checks are not
retroactively treated as passes or blocks.

Raw attempt and lane evidence remains under
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-29-attempt.json` and
`artifacts/sprint-8a-closeout/rehearsal/attempt-29/`:

- `dashboard-source-boundaries` and `markdown-links` each retained a failed
  `product` receipt even though its child log reported success. Diagnosis found
  one stale parent `$LASTEXITCODE` reused after the PowerShell invocations.
  Both receipts remain evidence, while the consolidated root classification is
  one `harness` defect rather than two product defects.
- `workspace-tests` retained the exact Component product-integration failure.
  Its Table fixture declared only `label` as required while configuring both
  `label` and `amount`; the product correctly rejected the invalid input with
  `400 component.bad_request` before reaching the unavailable provider, so the
  expected `503` was unreachable. The consolidated root classification is
  `harness` with test-fixture subtype; product validation must not be weakened.
- `optimized-resource-reference-timing` was interrupted while executing when
  formal testing exited. It has no defect classification.

The plan-to-source implementation audit also found one `product` architecture
defect. The Component render response was untyped or privately copied across
the Component/Dashboard boundary. The required correction makes
`tessara-components-contract` own the exact render response DTOs and render
kind, returns those types from Component provider and product routes, makes
Dashboard validate and consume them directly, and removes both the direct
Dataset-contract dependency and local Component response copies from the
Dashboard placement renderer. The response omits the unused Dataset provider
identity. Unknown, malformed, mixed-kind, and identity-divergent responses
must fail closed.

The one child-exit harness defect, one test-fixture harness defect, and one
render-boundary product defect form a single return-to-implementation batch.
Harness, fixture, contract, product source, boundary-test, acceptance, and
documentation changes are candidate-affecting; they supersede Readiness 36 and
the stopped Rehearsal 29 source. The corrections were completed in the
consolidated 2026-08-07 batch below but are not yet formally verified. Formal
testing remains paused. A new complete
Validation Readiness and complete Candidate Rehearsal must both pass against
the same corrected clean source and environment identity before candidate
freeze or preflight; SIT and formal UAT remain Not Run.

## 2026-08-07 — Completed the consolidated testing-entry contract correction

The implementation audit continued after the three Rehearsal 29 root findings
were classified. It found related product and acceptance defects in the same
touched dependency cone, so they were corrected together instead of restarting
testing after an individual narrow check:

- **Render ownership and kind exactness (`product`):** the Dashboard placement
  renderer privately modeled Component wire responses and depended directly on
  the Dataset contract. `tessara-components-contract` now owns the one exact
  Table/visual response and render-kind vocabulary. Component product/provider
  routes return it, Dashboard validates it, and the renderer consumes it without
  a Dataset-contract edge or copied DTO. Unknown fields, wrong Table/visual
  branch, wrong kind, and visual fields or payloads belonging to another kind
  fail closed. The unused Dataset reference was removed from render output.
- **Persistent versus preview identity (`product`):** a persistent response now
  requires matching non-nil Component and ComponentVersion IDs. Unsaved
  authoring preview uses one explicit preview-only identity with both IDs nil.
  A non-nil or partial-nil preview and any nil persistent identity are rejected.
- **Exact signed request bytes (`product`, security-boundary subtype):**
  Component provider, Core authorization exchange, and Core Dataset provider
  receivers previously
  parsed and reserialized typed JSON before comparing the service-request body
  digest. They now retain the raw body, enforce a JSON media type, verify the
  grant/service/correlation/body binding before typed deserialization, and
  reject a byte-only mutation such as appended whitespace unless the request is
  signed again. Product validation and the default body-size bound remain.
- **Dashboard joint scope and exact assertion (`product`, security-boundary
  subtype):** physical extraction had dropped the Sprint 7A rule that rendering
  requires one common governing node. Dashboard now forwards only the
  actor-authorized intersection
  of stored Dashboard scope, exchanges an exact ComponentVersion assertion
  containing type/id/authority revision/canonical Component scope, and checks
  the common node against both audience grants. Component compares the
  assertion with its authoritative row and repeats the common-node check.
  Disjoint placements are restricted before title, Component metadata, scope,
  Dataset identity, or data can be projected, even for an actor separately
  authorized in both disjoint scopes.
- **Active legacy acceptance facade (`harness` / acceptance inventory):** the
  permissions suite's Component normalization aliases and the general smoke/UAT
  flat `dataset_id`/`dataset_version_major` inputs could let retired Core shapes
  survive behind first-party helpers. Active fixtures now use only the exact v3
  Component identity and nested typed Dataset reference, and a source contract
  rejects reintroduction. Historical versioned fixtures remain unchanged.
- **Rehearsal exit accounting (`harness`):** each PowerShell child result is now
  captured at its own invocation, with a stale-`$LASTEXITCODE` self-test. This
  complements the existing dependency graph: safe independent siblings run
  fail-late, true dependents retain exact blocked reasons, partial results stay
  available, one diagnostic pass produces one consolidated batch, and tracked
  correction/restart remains closed until harvesting is complete.

The earlier seven-to-six Dashboard count correction remains part of this
testing-entry lineage because it fixed the same ownership boundary. The durable
expectation is identity-based: Core has exactly five transitions—Forms,
Workflows, Responses, Datasets, and Migration—while Dashboard appears exactly
once through its real Release/Instance and Manifest at reference navigation
order 9. Components appears at 8 and Scoped Records at 7. Duplicate
transition/release inventory or navigation presentation is rejected; no copied
total count substitutes for those identities.

The Playwright acceptance manifest advances from 74 to 75 exact identities by
adding `Dashboard and Component scopes must share a governing node before
disclosure or render`. The new scenario proves the shared-node positive path
and, for a disjoint placement, metadata/title redaction and render denial. It is
mapped to the `joint-dashboard-component-scope` assertion in UAT-8A-04, whose
manual script now requires the same semantic outcome. This is additional
coverage, not a renamed or weakened prior test.

After the correction cone was complete, the final product source passed
`cargo test --workspace --all-features --locked --offline` against six freshly
reset isolated databases in 619.6 seconds. All-target/all-feature clippy with
warnings denied, all-target/all-feature check, formatting, focused contract and
integration checks, boundary and acceptance contracts, Markdown links, runner
self-tests, exact 75/75 inventory, TypeScript compilation, and 75-test
Playwright discovery also passed. Documentation-only handoff edits are covered
by the final static audits.

These results establish implementation readiness but are not formal gate
receipts. Formal testing remains paused. The next authorized lifecycle boundary
is a new complete Validation Readiness followed by a new complete Candidate
Rehearsal against the same clean source and fresh six-database environment
identity; preflight, candidate freeze, SIT, and formal UAT remain forbidden
until both pass.

## 2026-08-07 — Consolidated Readiness 37/Rehearsal 30 correction

Validation Readiness 37 passed all 15 checks against clean commit
`84964c7bdb6b5d4705a2e4899a1fe2c98ee77183`, tree
`4f39cb7126dbe6f24db77f4d644593fb5ef9f0ca`, and environment fingerprint
`96c2a32ed16dfb288a4ca2578c171413d727c16153f5fbed3dd6dff9c2b410b3`.
Its retained result SHA-256 is
`a4ab361af6606ff94eaef8c5be5ca3e864dc90cb41efabdcb3aba73014eacf9b`.
The subsequent candidate-affecting correction supersedes that mutable pass.

Candidate Rehearsal 30 completed its declared 32-lane graph and terminal
fail-late harvest against the same identity. Eighteen lanes passed, four raw
lanes failed, and ten were blocked. Cleanup/restoration was not proven, so no
`candidate-rehearsal-result.json` was issued. The four immutable raw failures
are retained exactly as observed:

- `source-exact-materialization-no-op` — `environment`;
- `failure-containment-successor-health` — `product`;
- `uat-diagnostics` — `harness`; and
- `final-environment-identity` — `environment`.

Diagnosis consolidates those four symptoms under three `harness` roots without
rewriting the raw lane classifications:

- bootstrap action dispatch rebound the current pipeline object to an action
  string before `set_enablement` read `enabled`; retaining the action object
  corrects both the materialization failure and failure-containment cascade;
- source identity was returned as an ordered dictionary instead of the exact
  object shape required by UAT receipt guards, producing 12 false prerequisite
  failures and 8 internally blocked automated-UAT scenarios; and
- bootstrap left seven signing, source, and installation process variables set,
  changing normalized Compose identity in the same process. An external
  recomputation after exit matched R37 exactly, so this is harness leakage and
  not persistent environment drift.

The ten blocked lanes and their exact dependencies are retained:

- `deployed-inventory-navigation-audit`, `deployment-evidence`,
  `product-smoke`, and `component-upgrade-rollback` were blocked by
  `source-exact-materialization-no-op`;
- `playwright-execution` was blocked by
  `source-exact-materialization-no-op` and `deployment-evidence`;
- `live-product-diagnostics` was blocked by `deployment-evidence` and
  `product-smoke`; and
- `successor-inventory-navigation-audit`, `successor-deployment-evidence`,
  `successor-product-smoke`, and `final-successor-health` were blocked by
  `failure-containment-successor-health`.

R30 also exposed four repository-owned evidence enforcement gaps. The
correction gives every terminal lane exact assertion-start evidence and counts
only executed lanes; gives a structured child classification precedence over
log/default inference while naming its source; retains secret-free expected,
actual, and changed-section environment identity evidence; and persists new
evidence references as contained repository-relative paths. The immutable R30
authorization's already-retained in-root absolute references may be
canonicalized for its single consumption, but are not rewritten or generalized
into support for outside-root paths.

The immutable evidence is retained at:

- `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-attempt.json`,
  SHA-256
  `9ce1bba709b0b4bf1eee23cd0eea406fd49f16152cd25d45ea9dc3dac98617b5`;
- `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-harvest.json`,
  SHA-256
  `6167ab682922e7cfe582c9b461f38e31f4260c6259fffd3c3877eef7b62e3d13`;
- `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-defect-batch.json`,
  SHA-256
  `397260d3b4382ddc4731a2c8f7bcc3341ed5f6bc3b77d3dcaf8b03cf3cb36291`;
  and
- `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-correction-authorization.json`,
  SHA-256
  `efed4d4c53d84454372936067a5a9c454457cf2f42419afe331fc89a8d154792`.

At publication, the authorization was unconsumed and permitted exactly one
successor Validation Readiness start. Readiness 38 later consumed it and
retained that consumption; it cannot be reused. The tracked harness,
acceptance, runner, validation-reference, and Sprint documentation corrections
were candidate-affecting. Complete successor Readiness and complete Candidate
Rehearsal still must both pass against the same corrected source and
environment identity before preflight, candidate freeze, SIT, or formal UAT.

The downstream handoff is now executable through three repository-owned
interfaces: `run-sprint-8a-validation-preflight.ps1` owns ten exact checks and
candidate freeze, `run-sprint-8a-sit.ps1 -Stage Run|Finalize` owns four exact
lanes and evidence-only recovery, and
`run-sprint-8a-formal-uat.ps1 -Stage Start|Finalize` owns the complete UAT state
machine. `scripts/sprint-8a-lifecycle-chain.ps1` validates their exact identities
and publications. All bind the live normalized Compose configuration digest in
addition to tracked deployment inputs.

Formal UAT now holds the evidence-root lock through each human scenario with a
typed execution lease. The manual publisher requires that live lease and writes
an authenticated start/completion pair bound to the exact scenario receipt.
Scripted or manual defects retain safe remaining scenarios as
non-authoritative diagnostic evidence and exact dependency blocks; a product
decision pauses. Finalize refuses open leases, requires all eight terminal
receipts, and writes a durable completed scripted/manual/restoration checkpoint
before result publication. An exact evidence-finalization failure may consume
that checkpoint for publication-only retry without repeating manual work or
restoration. Canonical UAT recovery handles only authenticated `absent`,
`json-only`, `sidecar-only`, or `complete` JSON/sidecar states.

Readiness and the Sprint acceptance contract now parse and self-test the new
lifecycle, preflight, SIT, and formal-UAT runners and pin normalized-Compose,
manifest-replacement, execution-lease, fail-late, completion-checkpoint, retry,
and canonical-pair interfaces. Focused parser/self-tests and the combined
acceptance contract passed while source remained mutable. These are
implementation checks, not formal receipts: no corrected-source Readiness,
Rehearsal, preflight, candidate, SIT, or formal UAT has run.

## 2026-08-07 — Implementation-exit product contract correction

The user-directed exit from testing triggered a fresh comparison of product
source with the Sprint 8A plan and the validation contracts. That audit found
two P1 product defects that would have prevented a clean testing entry. They
are corrected together while the candidate remains mutable:

- **Dashboard executable identity (`product` / deployment):** the current
  Dashboard Manifest declared `/app/dashboard-module`, while the Dashboard
  image installs only `/usr/local/bin/dashboard-module`. Runtime and migration
  commands now use the installed path. The authoritative Dashboard manifest
  test and Sprint acceptance contract bind both exact command vectors to the
  Dockerfile copy/entrypoint contract. The current Dashboard canonical
  manifest digest and Sprint 8A release catalog were advanced together. The
  immutable Sprint 6E baseline remains historical evidence and is unchanged.
- **Component configuration generation (`product` / exact-current input):**
  server validation, provider execution, and browser editor code still read
  retired object-shaped `visible_columns` entries and the retired filter
  `field` alias. Fresh-only Sprint 8A accepts only `visible_columns: string[]`
  and filter `field_key`. Rust negatives reject both object variants and the
  old filter key; provider/browser readers and the acceptance source guard no
  longer normalize them. The Component JavaScript, embedded asset constant,
  Manifest asset digest, canonical Manifest digest, and Sprint 8A catalog were
  updated as one derived-input cone.

The same audit reconfirmed the independent-module correction by exact identity:
Core contains only Forms, Workflows, Responses, Datasets, and Migration;
Dashboard and Components appear only through real Module Releases/Instances;
navigation remains Scoped Records 7, Components 8, Dashboard 9; and no
duplicate Dashboard inventory or navigation presentation remains.

Focused implementation proof includes formatting, the Component validation
negative matrix, canonical provider projection, embedded asset/digest pinning,
the Dashboard authoritative-manifest test, both API catalog/manifest digest
tests, PowerShell parsing, the Sprint acceptance contract, and lifecycle-runner
self-tests. These checks do not issue a gate receipt. All earlier mutable
Readiness/Rehearsal evidence is superseded; the next testing boundary remains a
complete successor Validation Readiness followed by a complete Candidate
Rehearsal against the same corrected source and environment identity.

## 2026-08-07 — Readiness 38 testing exit and consolidated entry correction

Validation Readiness 38 consumed the Rehearsal 30 correction authorization and
ran its complete 15-check fail-late graph against clean commit
`91c9936be7e0cd9bc6beef78e04dc1937bf6601d`, tree
`564493fda54badc3d4bc5a99d2336b327424f381`. It retained 12 passed checks, 2
failed checks, 1 blocked check, and 14 assertion-bearing checks. The immutable
attempt receipt SHA-256 is
`7dcdc6a3d9866311ba12f251b080a4fe732b3fe10d1eb0ff6ad7de1d44a0f86d`;
the authorization-consumption SHA-256 is
`867f2628800584bb59bbe287c02673a852c4007320dc9608620b78dbb739fb79`.

The complete retained defect set is:

- `compose-database-contract` — `environment`; the Readiness process omitted
  `TEST_API_DATABASE_URL`. Its raw log SHA-256 is
  `f69a6e944f30e1ce3dc998cf212a28c09462b0128de2e9d14c0d9bd074bcd57a`.
- `runner-self-tests` — `harness`; the failure-containment self-test requested
  an overwrite after deliberately corrupting a publication pair, rather than
  resetting only that exact temporary fixture pair before create-once
  republication. Its raw log SHA-256 is
  `db508e3b5958bf3dce71af76d64fe053feb068d980f39bd86949461c02ab0c9f`.
- `environment-contract` — blocked without assertions by exactly
  `compose-database-contract`; it is not counted as an independent failure.

Formal testing exited at that terminal boundary. The plan, validation
protocol, source, fixtures, and downstream acceptance inventory were reviewed
before any restart. That audit expanded the single implementation correction
batch to close the actual testing-entry gaps:

- require all six disposable database bindings, reset acknowledgement, and
  Postgres client-container identity in the same process as the next Readiness;
- make the failure-containment self-test reject unauthenticated overwrite,
  preserve real corrupt retained evidence, reset only the exact corrupt pair
  inside its validated temporary self-test root, and republish create-once;
- add typed failed-Readiness harvesting, one consolidated defect batch, an
  exact-next-attempt authorization, and an append-only lineage that preserves
  the earlier consumed R30 link and prevents reuse, gaps, or forks;
- reserve a Readiness attempt number under the exclusive validation lock before
  creating its receipt, start snapshot, sidecars, or log directory, so an out-
  of-sequence probe cannot strand the exact authorized successor namespace;
- retain immutable Rehearsal declarations separately from exact terminal
  results, checkpoint every pass/failure/block, and require the attempt and
  harvest terminal inventories to match;
- remove the retired Component `missing_policy` alias from validation,
  provider execution, browser authoring/rehydration, and Playwright fixtures;
  reject it across all five visual kinds and retain only purpose-specific
  current policy keys; and
- define one canonical eight-scenario manual-UAT contract and enforce exact
  scenario, role/tester, precondition, document, evidence kind/cardinality,
  AC-18/AC-19, and semantic-predicate mappings. Arbitrary hashes, free-text
  starting state, ambiguous “accepted with defects” outcomes, or missing named
  evidence cannot produce formal acceptance. UAT-8A-07 additionally binds the
  failed apply response and service logs through two exact hashed raw-evidence
  assertions in an authenticated failure-containment receipt.

The Component JavaScript SHA-256 advances to
`7f13c08219f641055c1fb3bbabc9ab38f724db9b7bda2cc712ca830c9c736e1e`;
its canonical Manifest/catalog digest advances to
`01fdc82cacbc20658a0c2bdcb2177c9aded9fc332e1bcc8d9627c1670944f998`.
Focused implementation verification is recorded separately from formal gate
evidence. No Readiness 39, Rehearsal 31, preflight, candidate freeze, SIT, or
formal UAT runs in this correction phase.

The pre-audit implementation sweep passed all 12 fail-late harness
checks: PowerShell parsing, failure containment, Readiness, Candidate
Rehearsal, harvest, lifecycle-chain, formal-UAT, diagnostic-UAT, preflight,
acceptance-contract, deployed-inventory, and Markdown-link checks. Product
proof also passed formatting, JavaScript syntax, 33 Component library tests,
warnings-denied Component clippy, the exact five-transition contract test,
three exact Dashboard/inventory/navigation API tests, and discovery of all 75
Playwright scenarios without browser execution. These are narrow mutable-source
implementation checks only; they did not create or advance a lifecycle receipt.

The independent final diff audit then exposed the attempt-namespace reservation
defect, weak UAT-8A-07 raw-evidence binding, and two stale verification summaries
as one last batch. After correction, the complete affected focused cone passed:
Readiness, lifecycle, formal-UAT, diagnostic-UAT, preflight, acceptance,
harvest-adversarial, and Candidate Rehearsal self-tests plus PowerShell/JSON
parsing. The canonical UAT inventory now contains 46 globally unique exact-one
requirements. Neither diagnostic pass ran a formal lifecycle attempt.

## 2026-08-07 — Rehearsal 32 health, restoration, and layout correction

Validation Readiness 41 passed 15/15 against clean `d703e7a6` / `889ddc5f`
and environment `53fbb1f7...`. Candidate Rehearsal 32 then terminalized its
32-lane conservative full-harvest schedule with 19 passes, 2 raw `product`
failures, 11 exact dependency blocks, and 0 deferrals. The immutable attempt
SHA-256 is `98cddc03977a476af5c5843131b9114ec2f025a1f5eefffa57514daccd5448c5`;
the harvest is `89861d8add519812993934dba182e6939b1437d38e7f524e4469693001c2c809`;
and its single raw two-defect batch is
`4a5219fd0e16d13fbf0e15286ae75f53c84e261a264bbef877127ff8866a1249`.
Those receipts and classifications remain unchanged.

The consolidated correction changes expectations only where diagnosis proved
the prior harness or product contract stale:

- All Sprint 8A health callers now consume the endpoint-specific,
  redirect-disabled `scripts/sprint-8a-health-contract.ps1`: Core
  `GET /health` is 200 `text/plain` with exact body `ok`; Supervisor
  `GET /health/ready` is 204 with an empty body.
  R32 had followed Core's redirected Supervisor-style request to `/login`,
  accepted HTML as healthy, and then rejected Supervisor's valid 204. No retry,
  timeout increase, broad 200/204 acceptance, or product-health weakening is
  introduced.
- The Dashboard smoke contract no longer requires bootstrap-only
  `placement_key` in a public response. Stronger proof pins all seven opaque
  placement IDs and geometries, six disclosed exact Component references and
  resolution states, and the restricted placement's title/Component identity
  nondisclosure.
- Dashboard bootstrap now persists the one canonical V1 placement-config
  representation instead of an unversioned `{placement_key,width,height}`
  object that every reader treated as legacy fallback geometry. The seed
  validates seven unique IDs/keys, dense positions 0 through 6, and exact
  one-based row/column/width/height values: 1/1/4/2, 3/1/12/6, 9/1/6/4,
  9/7/6/4, and row 13 at columns 1/5/9 with width 4 and height 2. This is a
  forward-only pre-production seed correction; bootstrap-only keys do not enter
  product config, and no old-row reader or migration is retained.
- New Candidate Rehearsal starts serialize the complete lane declarations,
  not a names-only list, and canonicalize every evidence path/root/nested-result
  reference to contained repository-relative forward-slash form. Recovery must
  match that full graph and schedule exactly.
- Wave B execution/deferral depends only on the completed diagnostic Wave A.
  Aggregate sinks follow Wave B, then the terminal canonical-restoration sink
  and safety finalizers run regardless. Failure-containment recovery remains a
  Wave A diagnostic and cannot substitute for final restoration.
- A failed attempt with complete terminal accounting but unproven restoration
  may publish its harvest and one batch through `-HarvestOnly`, but receives no
  prospective correction authorization. Schema-3 authorization requires exact
  passing current-attempt final-health and final-environment receipts and
  `cleanup_restoration.result = canonical_successor_healthy`.

R32 predates that last guard. Its already-issued schema-2 authorization remains
quarantined and is effective only with append-only qualification SHA-256
`859a8dba81792127820a12367e9d0430aaebb9bf7cb77442b50e34322ba4c7bb`,
which authenticates diagnostic supplement `8bcfc25c...` and source-exact
restoration `e082eefd...` and permits exactly one Readiness 42 start. This is a
one-off historical bridge, not a reusable compatibility path, and it authorizes
no rehearsal result, preflight, candidate freeze, SIT, UAT, or closeout.

Focused adversarial verification must cover fixed immutable ordering, safe
dependency closure, aggregate-sink behavior, both Wave outcomes, the
three-deferral limit, impact-cone override, cleanup/restoration, process-loss
recovery, deferred-result/preflight rejection, and complete-versus-missing
terminal accounting. R9-R11 materialization and R19-R22 Playwright histories
remain diagnostic regression fixtures only. These source, harness, fixture,
schema, acceptance, runner, skill-reference, and documentation changes are
candidate-affecting. They require a clean implementation commit followed, only
with validation-coordinator authorization, by complete Readiness 42 and
complete Rehearsal 33.

The final implementation audit additionally requires exact dependency and
deferral-counter semantics, evidence-root containment, full start/attempt graph
authentication, lifecycle-boundary process-loss recovery, and a hashed
materialization receipt before final restoration can pass. Candidate Rehearsal
ordering now places its lifecycle prerequisites first and materialization plus
containment/recovery as the first expensive Wave A branch. Full Rehearsal is
reserved for final certification: the clean disposable materialization/no-op,
recovery, health, smoke, regression, acceptance, and runner reproducers must
pass and retain explicitly non-authoritative evidence before Readiness 42 may be
started.

## 2026-08-07 — Rehearsal 33 consolidated product, fixture, and evidence correction

R33 retained 29 passes, 2 failures, 1 dependency block, and 0 deferrals across
all 32 lanes. The correction batch does not rewrite those receipts and starts
no successor lifecycle attempt.

- Product: real manifest navigation now uses exact shell route keys while
  retaining exact contribution IDs; static module API routes outrank parameter
  siblings, fixing the Component Dataset authorization path without a
  definition-specific gateway branch.
- Fixtures/acceptance: analytics advances from the superseded four-row result
  to the canonical 30-row seed; Dashboard SSR asserts cross-scope
  nondisclosure; diagnostics assert four semantic identities; Component
  Playwright uses exact accessible roles and version-route identities.
- Evidence: final restoration authenticates the compact materializer source
  shape by canonical fields; harvest permits only the declared
  pre-authentication lifecycle placeholder and still rejects every stale
  authenticated lane. The retained R32 qualification self-test binds its
  historical state to immutable Readiness 41 instead of the canonical alias
  that legitimately advanced after later attempts.
- Process: the Component browser path crossed the three-failure threshold and
  was handled as one concentrated validation-platform fixture incident. Its
  clean focused reproducer passed before the nine-regression successor set.

All verification from this correction phase is focused and non-authoritative.
It cannot produce a Candidate Rehearsal result or authorize preflight, freeze,
SIT, UAT, or closeout.

R33 terminal-tail recovery subsequently exposed and corrected two recovery
projection defects: historical declarations were compared with the corrected
runner, and retained append-only harvest/batch timestamps were reconstructed.
Recovery now authenticates the immutable attempt graph and directly consumes
the retained harvest/batch. The recovered state remains failed and correction
authority remains withheld. The protocol names R32 as the sole post-harvest
qualification exception, so R33 cannot receive an analogous qualification
without an explicit governing lifecycle decision. No historical receipt was
rewritten.

## 2026-08-08 — Approved R33 evidence-correction bridge

The user approved a one-time append-only correction record after the retained
R33 evidence proved that canonical recovery succeeded and only the old compact-
source comparison failed. The implementation leaves every R33 attempt, lane,
harvest, batch, log, and raw result unchanged. A repository-owned finalizer may
issue an authorization/qualification tuple for exactly Readiness 44 only after
authenticating the retained clean apply, semantic no-op, exact Core and
Supervisor health, exact five-transition inventory, single real Dashboard
presentation, passing final environment check, and the clean committed
correction source. The tuple does not mark R33 passed and cannot authorize any
downstream phase. Receipt validation rejects changed hashes, another attempt,
another successor, a dirty or different correction source, or use after
Readiness 44 has begun. This is the second and final named Sprint 8A historical
bridge, not a reusable post-harvest exception.

## 2026-08-08 — Consolidated Readiness 44 check-list reader correction

Readiness 44 completed its full fail-late inventory with 13 passes, two
`harness` failures, and no blocked checks. Both retained defects have the same
root cause. The preflight helper used an AST query for every assignment to
`$declaredChecks`; the rehearsal runner legitimately has both its canonical
array declaration and a later assignment that restores the immutable list
when resuming an interrupted attempt.

The helper now selects only an assignment whose right side is an array
expression. Its self-test continues to require the resume assignment and
proves that the canonical declaration is still extracted exactly. This maps
to the verification plan's complete declared-check identity, recovery, smoke,
UAT, and runner-self-test clauses. A related readiness self-test now locates
the exact R33 correction link after consumption instead of assuming it remains
the lineage tip; it authenticates the source that consumed the bridge and
rejects the next unauthorized attempt in either the pending or consumed state.
The direct acceptance contract plus readiness, rehearsal, preflight, smoke,
and automated UAT self-tests pass as focused, non-authoritative reproducers.
Readiness 44 remains failed and unchanged; the clean correction commit requires
complete Readiness 45 before Rehearsal 34 can start.

## 2026-08-09 — Rehearsal 35 consolidated recovery and browser correction

Rehearsal 35 retained 27 passes, four failures, one dependency block, and no
deferrals. Three failures record controlling-process loss; the fourth retains
three Playwright failures and eleven tests that did not run after the suite's
failure limit. Its attempt, lane, harvest, batch, logs, traces, and raw
classifications remain unchanged.

- Recovery now converts JSON-loaded declarations back to the runner's canonical
  ordered dictionary before optional-member checks, and serializes recovered
  orphan start boundaries as offset-qualified ISO 8601 text.
- The harvest guard may interpret an old locale-form orphan timestamp only from
  its exact hashed executing-lane capture. It does not accept a free-standing
  offsetless timestamp or rewrite the retained result.
- Component Dataset references now use recursively key-sorted JSON identity, so
  a metadata retry preserves the selected major line even when equivalent JSON
  objects arrive with a different property order.
- The visual-editor test selects the Calculation combobox inside its exact
  fieldset, and Component permission checks assert the module-owned
  `component.forbidden` contract instead of Core's retired generic code.

These changes map to the Sprint 8A recovery/evidence clauses, AC-04/AC-05
Component ownership and resilience, and the exact acceptance-identity rule.
Focused results remain diagnostic and cannot replace complete Readiness or
Candidate Rehearsal.

### Rehearsal 35 clean-entry completion

The retained browser failures exposed two additional acceptance defects after
the first selectors were corrected. The closed publish dialog carried a native
`required` constraint on a hidden note field, which prevented every ordinary
Save Draft submission before Component validation ran. The visual-authoring
scenario also relied on duplicated accessible labels after the structured
editor began moving category-display controls between sections. The correction
removes the hidden native constraint while retaining the dialog's explicit note
validation, and binds the scenario to stable `data-config-control` identities.

The browser guard now accounts for exactly one expected HTTP 400 console entry
when the scenario deliberately submits invalid visual configuration. Component
permission checks assert the complete module-owned error envelopes, including
the nondisclosing `component.not_found` response for an inaccessible existing
Component. They no longer require Core's optional duplicate `error` field.

Focused Component tests, warnings-denied Clippy, Playwright discovery, the three
retained browser reproducers, the Sprint 8A acceptance contract, Candidate
Rehearsal self-test, and harvest-guard self-test pass. Clean source-exact
materialization/no-op and induced-failure recovery successors are retained under
`artifacts/sprint-8a-focused-r35-correction/`. Every artifact in that directory
is diagnostic history only; none replaces complete Validation Readiness or a
complete Candidate Rehearsal against the same committed source and environment.

## 2026-08-09 — Rehearsal 36 historical Component response correction

Readiness 47 passed all 15 checks against commit
`28104a43e76f7633d9e6c2cae03f459cd230dafe`. Rehearsal 36 then completed its
full fail-late harvest with 30 passes, one Playwright failure, one dependent UAT
diagnostic block, and no deferrals. Recovery, final health, clean-source, and
environment-identity checks passed. The immutable attempt, raw browser output,
trace, harvest, and one-defect batch remain unchanged.

The failed historical-version permission scenario still expected Core's
retired generic `not_found` response. The Component module correctly returned
its canonical nondisclosing `component.not_found` envelope. The scenario now
asserts that complete module-owned envelope with the shared exact-response
helper, matching the already-covered publish nondisclosure contract. This is an
acceptance-test correction only; it does not alter product behavior or make
Rehearsal 36 pass. The exact focused scenario, Sprint 8A acceptance contract,
Candidate Rehearsal and harvest-guard self-tests, and Markdown-link check pass.
The retained focused evidence is diagnostic only and cannot replace a new
complete Readiness/Rehearsal cycle.

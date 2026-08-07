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
- Readiness and Candidate Rehearsal publish unverified start evidence before
  fallible state/source/environment work and serialize attempts with an
  evidence-root OS-exclusive file handle. Readiness retains an immutable hashed
  start snapshot and checkpoints its live receipt, sidecar, and validation-state
  hash after every terminal check or block.
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

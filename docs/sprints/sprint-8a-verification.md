# Sprint 8A Validation Record

Status: Formal testing exited after Validation Readiness 38 completed its
15-check fail-late graph against clean commit
`91c9936be7e0cd9bc6beef78e04dc1937bf6601d`, tree
`564493fda54badc3d4bc5a99d2336b327424f381`: 12 passed, 2 failed, and 1 was
blocked by an exact prerequisite. The failures are classified `environment`
for the missing same-process `TEST_API_DATABASE_URL` binding and `harness` for
the stale failure-containment self-test recovery sequence;
`environment-contract` is blocked by the former. The immutable result consumed
the Rehearsal 30 correction authorization exactly once. The resulting
implementation audit also found a failed-Readiness retry dead end, incomplete
Rehearsal terminal checkpoints, an active retired Component `missing_policy`
alias, and incomplete AC-18/AC-19 UAT/evidence enforcement. These are one
candidate-affecting correction batch. No corrected readiness, rehearsal pass,
or candidate exists, and preflight, SIT, formal UAT, and closeout remain Not
Run.

- Sprint: Sprint 8A — Component Module Separation Slice
- Branch: `codex/sprint-8a`
- Planned evidence root: `artifacts/sprint-8a-closeout/`
- Execution contract: [Sprint 8A plan](./sprint-8a-plan.md)

## Implementation Readiness Snapshot

### Readiness 38 terminal failure and current implementation correction

Readiness 38 retained all 15 terminal results at
`artifacts/sprint-8a-closeout/attempts/readiness-38.json`, SHA-256
`7dcdc6a3d9866311ba12f251b080a4fe732b3fe10d1eb0ff6ad7de1d44a0f86d`.
Its exact result is 12 passes, 2 failures, 1 block, and 14 assertion-bearing
checks. The two failed-check logs are:

- `artifacts/sprint-8a-closeout/readiness-38/compose-database-contract.log`,
  SHA-256
  `f69a6e944f30e1ce3dc998cf212a28c09462b0128de2e9d14c0d9bd074bcd57a`;
  and
- `artifacts/sprint-8a-closeout/readiness-38/runner-self-tests.log`, SHA-256
  `db508e3b5958bf3dce71af76d64fe053feb068d980f39bd86949461c02ab0c9f`.

`environment-contract` did not run because
`compose-database-contract` failed. Its retained dependency reason is exactly
`blocked by failed prerequisite(s): compose-database-contract`. The R30
authorization consumption is retained with SHA-256
`867f2628800584bb59bbe287c02673a852c4007320dc9608620b78dbb739fb79`.
No R38 evidence is rewritten by the correction.

The plan-to-source audit consolidated six correction areas:

1. Bind all six disposable database URLs, reset acknowledgement, and validation
   container identity in the same Readiness process.
2. Prove that failure-containment rejects overwrite of a deliberately corrupt
   no-journal pair, then delete only that exact pair inside the validated
   temporary self-test root and republish it create-once. Real corrupt retained
   evidence remains preserved.
3. Add one typed failed-Readiness harvest, consolidated batch, exact-next-
   attempt authorization, and append-only correction lineage so consumed
   predecessor authorization cannot strand a failed successor.
4. Preserve immutable Rehearsal declarations separately from exact terminal
   results and checkpoint every completed pass, failure, and block.
5. Reject the retired Component `missing_policy` key and preserve only the four
   purpose-specific current keys throughout the active first-party surface.
6. Bind the eight manual UAT scenarios to exact tester, precondition, evidence,
   AC-18/AC-19, semantic-predicate, and document identities, and enforce that
   contract before preflight and formal publication.

This source is mutable until the consolidated correction is committed. Only
focused implementation checks may run in this phase. R38 must then be finalized
into one harvest/batch/authorization without starting R39. Fresh complete
Readiness 39 and Rehearsal 31 are required before preflight.

### Readiness 37 and Rehearsal 30 historical consolidated correction

Validation Readiness 37 passed its complete 15-check graph with zero failures
and zero blocks. Its result is retained at
`artifacts/sprint-8a-closeout/validation-readiness-result.json`, SHA-256
`a4ab361af6606ff94eaef8c5be5ca3e864dc90cb41efabdcb3aba73014eacf9b`.
Because runners, acceptance contracts, validation references, and Sprint
documentation now change, that mutable result is superseded and cannot
authorize preflight.

Candidate Rehearsal 30 ran against that exact source and environment and
completed the full fail-late terminal graph: 18 lanes passed, 4 raw lanes
failed, and 10 were blocked by failed prerequisites. Cleanup/restoration was
`not_proven`, and no `candidate-rehearsal-result.json` was written. The raw
failed lanes remain immutable evidence:

- `source-exact-materialization-no-op` — `environment`;
- `failure-containment-successor-health` — `product`;
- `uat-diagnostics` — `harness`; and
- `final-environment-identity` — `environment`.

The ten blocked lanes retain their exact dependency reasons:

- `deployed-inventory-navigation-audit`, `deployment-evidence`,
  `product-smoke`, and `component-upgrade-rollback` depend on
  `source-exact-materialization-no-op`;
- `playwright-execution` depends on `source-exact-materialization-no-op` and
  `deployment-evidence`;
- `live-product-diagnostics` depends on `deployment-evidence` and
  `product-smoke`; and
- `successor-inventory-navigation-audit`, `successor-deployment-evidence`,
  `successor-product-smoke`, and `final-successor-health` depend on
  `failure-containment-successor-health`.

Diagnosis consolidates the four raw symptoms under three `harness` roots:

1. Bootstrap action dispatch passed a string from `switch` where
   `set_enablement` required the complete action object's `enabled` field. This
   caused both materialization symptoms and the failure-containment cascade.
2. Source identity was returned as an ordered dictionary rather than the exact
   object shape required by automated-UAT receipt guards, producing 12 false
   prerequisite failures and 8 blocked scenarios.
3. Bootstrap did not restore seven process environment variables used for
   signing, source identity, and installation identity. A fresh external
   environment recomputation after process exit matched R37 exactly, proving
   harness leakage rather than persistent environment drift.

The same audit found four evidence-enforcement gaps: blocked terminal receipts
were counted as assertions, structured child classification could lose to a
lane default, final environment mismatch evidence omitted expected/actual
comparison detail, and already-retained authorization paths were not uniformly
canonical. The correction adds exact per-lane assertion-start evidence,
structured-classification precedence with a named source, secret-free compared
fingerprints and changed sections, and contained repository-relative paths for
new evidence. The immutable in-root R30 authorization was consumable once
without rewriting it or permitting an outside-root reference; R38 consumed it
and retained that consumption.

The retained evidence is:

- attempt: `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-attempt.json`,
  SHA-256
  `9ce1bba709b0b4bf1eee23cd0eea406fd49f16152cd25d45ea9dc3dac98617b5`;
- harvest: `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-harvest.json`,
  SHA-256
  `6167ab682922e7cfe582c9b461f38e31f4260c6259fffd3c3877eef7b62e3d13`;
- defect batch:
  `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-defect-batch.json`,
  SHA-256
  `397260d3b4382ddc4731a2c8f7bcc3341ed5f6bc3b77d3dcaf8b03cf3cb36291`;
  and
- correction authorization:
  `artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-30-correction-authorization.json`,
  SHA-256
  `efed4d4c53d84454372936067a5a9c454457cf2f42419afe331fc89a8d154792`.

The authorization permitted exactly one successor Readiness start. Readiness
38 consumed it and failed, so it cannot be reused. The current correction adds
a second append-only lineage link derived from R38's terminal evidence; that
link alone may authorize Readiness 39 after R38 finalization. Readiness and
Rehearsal must both pass against the same corrected clean source and fresh six-
database environment before preflight.

### Prior user-directed return from Readiness 36 and Rehearsal 29

Validation Readiness attempt 36 passed its complete 15-check graph against
clean commit `3f7e32cb7e1948a36185a68352901b873934c0b0`, tree
`252e651e0d031c04d61547b6d6eac964bb04f84f`, and environment fingerprint
`2e235b070dd2f3663242fe5b3f983f561b834161ed65847ad6dd399d35e01da0`.
It recorded zero failures and zero blocks. Its mutable result is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-36.json` with SHA-256
`f10f4c63d96e8bf484f9a19084359e58cf81858adcfb5e4bf3cb49b84c716710`.
That pass describes only the prior clean source. The harness, fixture,
contract, product, boundary, test, and documentation changes required below
are candidate-affecting and supersede it.

Candidate Rehearsal attempt 29 began against the same source and environment
identity. The user-directed phase exit stopped the attempt before a terminal
harvest: 8 lane receipts passed, 3 failed, the independent
`optimized-resource-reference-timing` lane was interrupted while executing,
20 declared checks were never executed, and no blocked receipt had yet been
recorded. The nonterminal attempt receipt and raw lane evidence remain at
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-29-attempt.json` and
`artifacts/sprint-8a-closeout/rehearsal/attempt-29/`. The zero recorded blocks
must not be read as 20 independent passes or as a complete dependency harvest;
the checks are unexecuted because testing was stopped.

The three failed receipts and their retained diagnostic classifications are:

- `dashboard-source-boundaries` and `markdown-links` each recorded a `product`
  lane failure, while their raw child-script logs explicitly reported success.
  One stale parent `$LASTEXITCODE` was reused after each PowerShell script
  invocation. The two receipts are retained, but the consolidated root defect
  is `harness`; neither is evidence of a product-boundary or Markdown defect.
- `workspace-tests` retained its failed `product` lane receipt and the exact
  `extracted_component_product_owns_crud_versions_lifecycle_render_and_nondisclosure`
  failure. Its Table fixture declared only `label` as required while its
  configuration used both `label` and `amount`; the product correctly returned
  `400 component.bad_request` before provider invocation rather than the
  fixture's expected provider-not-ready `503`. The consolidated root defect is
  `harness` with test-fixture subtype; the product validation must not be
  weakened.

The plan-to-source audit then expanded the same return-to-implementation batch:

- `product`: `tessara-components-contract` must own the exact render response
  DTOs and render-kind discriminant; Component provider/product routes return
  them, Dashboard validates them, and the placement renderer has neither a
  direct Dataset-contract edge nor local wire copies. Table and every visual
  kind reject another kind's fields/branch. The output omits the unused Dataset
  identity.
- `product`: persistent execution requires matching non-nil Component and
  ComponentVersion identities. Unsaved preview alone uses an explicit both-nil
  identity; a partial-nil identity is invalid in either path.
- `product`: Component, Core authorization-exchange, and Core Dataset service
  receivers must verify the exact raw-body digest, authorization/service
  envelopes, and correlation identity before typed deserialization while
  retaining JSON media-type and body-size enforcement. A semantically equal
  but byte-different unsigned mutation fails closed.
- `product`: Dashboard-mediated rendering requires one common governing node
  across stored/actor-authorized Dashboard scope, Component scope, inbound
  Dashboard authority, and downstream Component authority. The exchange is
  bound to the exact ComponentVersion type/id/authority revision/canonical
  scope assertion, which Component compares with its row. Disjoint metadata,
  title, scope, Dataset identity, and render output remain undisclosed.
- `harness` / acceptance inventory: active permissions, general smoke, and
  general UAT helpers must not normalize retired Core Component shapes or flat
  Dataset-major fields. First-party fixtures use exact v3 identities and typed
  references, enforced by a source guard.

The earlier identity-based Dashboard correction remains controlling: Core has
exactly five transitions (Forms, Workflows, Responses, Datasets, Migration),
and Dashboard appears only through its real Release/Instance and Manifest at
reference order 9, after Scoped Records 7 and Components 8. It is retained as a
`product` defect correction rather than reduced to another copied count.

The prior testing-entry correction included the observed child-exit and fixture
harness defects plus the related product, security, acceptance-inventory,
transition-ownership, and fail-late enforcement cone. The tracked Playwright
inventory is 75 exact identities, with the shared-node success/disjoint-node
redaction and render-denial scenario mapped to UAT-8A-04. That correction was
exercised by the implementation-level checks below and subsequently reached
Readiness 38, whose terminal failure and current correction are recorded at the
top of this file. Formal testing remains paused. R38 finalization is the next
evidence lifecycle boundary; complete Readiness 39 and Candidate Rehearsal 31
then must pass against the same corrected clean source and environment
identity.

Implementation-level verification then reset and recreated all six isolated
readiness databases and passed on the final corrected product source:

- `cargo test --workspace --all-features --locked --offline` in 619.6 seconds;
- all-target, all-feature `cargo clippy` with warnings denied, `cargo check`,
  `cargo fmt --check`, and `git diff --check`;
- Components contract, Component module, Dashboard module, Component product
  integration, and exact-body API checks;
- the Sprint 6E boundary and Sprint 8A acceptance contracts, Markdown links,
  candidate-rehearsal and UAT runner self-tests, and PowerShell parser checks;
- exact 75/75 acceptance inventory, TypeScript 5.9.3 compilation of the changed
  specifications, and Playwright discovery of all 75 tests.

These are implementation-entry checks, not formal Validation Readiness or
Candidate Rehearsal receipts. Documentation-only handoff edits followed the
Rust run and are covered by the final formatting, link, boundary, acceptance,
and diff audits. A complete formal gate pair remains mandatory on one clean
committed source and one verified environment before preflight.

### Validation Readiness attempt 35 evidence-finalization correction

Validation Readiness attempt 35 ran against clean commit
`49c0ae73c3c82dd12b3e2fea68d53b5d497b3b74`, tree
`57f07e9cb39721ea994cf462c3753eb5b500b4e0`, and environment fingerprint
`bfcbf8df8e719b581152dfd6e7ed3d74baa047417b6e1f9f10c001bd79f8efda`.
All 15 declared checks reached terminal `passed`: zero failed, zero blocked,
and zero skipped. The six authenticated database probes, exact 74-test
Playwright inventory, property-safe Compose self-test, captured materializer
WhatIf, package/metadata/link checks, and final clean-source check all passed.

After the final check checkpoint, terminalization evaluated
`$failures.classification` under StrictMode while `$failures` was empty. The
resulting `PropertyNotFoundException` occurred before the terminal attempt
receipt, canonical `validation-readiness-result.json`, or terminal validation
state could be published. The last live receipt therefore remains
non-authoritative `executing`; it is not a Readiness pass, Candidate Rehearsal
29 remains ineligible, and preflight stays closed.

The immutable start receipt SHA-256 is
`773410def7e4e95a1fc2f5a1b3cdc6cb4e9095a8b09137b2da35b8cb7a887e7f`;
the last live receipt SHA-256 is
`b98435d822b14253e82638dcba907fe788c6bf01c6f824431770ba6e915f9589`.
Every lane log, produced artifact, receipt, and state sidecar verified, and the
attempt lock was released. The previously missing process failure is retained
at `artifacts/sprint-8a-closeout/readiness-35/finalization-failure.log` with
SHA-256 `55c153e41c98fa0e380573f380a035a7d50ecbefb6404ea0b32225d4057789ee`.

One consolidated finding is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-35-consolidated-correction-batch.json`
with SHA-256
`827ffd273282c2fb149003f2625508cc27814c2c4162ad196586f1359e0ed454`.
Its classification is `evidence-finalization`; no product assertion or product
action failed. The bounded sibling audit found the same empty-array success
path in Candidate Rehearsal terminalization. The correction projects nonblank
classifications through one StrictMode-safe helper in both runners and
self-tests empty, single, duplicate, missing, null, and multiple inputs. The
acceptance contract enforces the helper and both runner call sites.

The validation protocol already required atomic finalization and complete raw
evidence. The enforcement gap was executable zero-failure terminalization
coverage, so the correction belongs in runners, self-tests, and acceptance
rather than duplicated protocol prose. Because those tracked files changed, a
new complete Readiness result and then a complete Candidate Rehearsal result
remain required against the same corrected clean source and environment
identity.

### Validation Readiness attempt 34 consolidated correction

Validation Readiness attempt 34 ran against clean commit
`c920b8a3dd1e35d5617e5f3d634eabff75499e77`, tree
`ba5c5f764d7d4b49d15417a1edd08cfdab16e0b1`. All 15 declared checks reached a
terminal state: 12 passed, `compose-database-contract` and `reset-dry-run`
failed, and `environment-contract` was blocked exactly by the failed
prerequisite: `blocked by failed prerequisite(s):
compose-database-contract`. Candidate Rehearsal 29 did not start.

The terminal receipt is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-34.json` with SHA-256
`f2b377e4a1fee943d5940fed25d4c6d03f807820001ea429a14711a5b516867c`.
Raw evidence remains at
`artifacts/sprint-8a-closeout/readiness-34/compose-database-contract.log` and
`artifacts/sprint-8a-closeout/readiness-34/reset-dry-run.log`. Start, terminal,
state, raw-evidence, and Playwright-inventory hashes all verified; the attempt
lock was released. No product or destructive database action began in either
failed lane.

The complete harvest produced one two-finding correction batch at
`artifacts/sprint-8a-closeout/attempts/readiness-34-consolidated-correction-batch.json`
with SHA-256
`dd8ffbdb742e184c76fb448fb58ddb720bb01efcfe0919a4bbfdf0d2360efa58`:

- `environment`: the attempt process lacked all six pairwise-distinct test
  database URLs, the exact disposable-reset acknowledgement, and its inspected
  PostgreSQL client container identity;
- `harness`: the materializer read normalized Compose's optional `ports`
  property as mandatory under StrictMode; the same bounded audit found the
  latent optional `environment` and resource-name assumptions.

The correction centralizes optional-property handling and normalized service
projection, covers services without ports or environment in an executable
runner self-test, and binds those helpers in the acceptance contract. The next
attempt must receive six fresh loopback databases and the exact reset/client
contract in the same PowerShell process. These tracked runner, contract, and
documentation changes invalidate attempt 34 as an authorization source; a new
complete Readiness result and then a complete Candidate Rehearsal result remain
required against the same clean source and environment identity.

### Post-Readiness-33 implementation completion audit

The user stopped the testing phase after Readiness 33 and required a fresh
plan-to-product review before another testing entry. No formal lifecycle gate
was run during this audit. It found one consolidated candidate-affecting batch
that crossed product, contracts, baseline schemas, deployment, fixtures,
acceptance tests, UAT evidence, and runner enforcement:

- the public gateway could be available while owner materialization was still
  mutating the disposable topology;
- Component bootstrap validated only local shape and could persist before the
  Core-owned Dataset provider evaluated the exact references and compatibility;
- normal Component create did not express Component type requirements in the
  provider compatibility request and did not honor an already-successful
  idempotent replay during a later provider outage;
- malformed configuration shape was projected as provider unavailability, and
  Module Management discarded the module's sanitized diagnostic details;
- Dashboard's composition surfaces could project a synthetic outage as
  provider-unavailable without a matching current disclosure basis, while
  finding/action/recovery episodes were insufficiently isolated by semantic
  actor, scope, authorization revision, organization revision, and expiry;
- the shared UAT fixture path could still write owner-controlled product tables,
  automated UAT could evaluate an embedded summary instead of authenticated raw
  evidence, and exact browser configuration/outage/responsive scenarios were
  absent from the acceptance inventory; and
- Candidate Rehearsal's nominally safe static lanes still depended on the
  fallible readiness prerequisite despite the protocol's fail-late rule.

The implementation now keeps the gateway offline until owner completion; uses
an exact locked-Manifest-derived, apply-bound, one-time signed bootstrap
authorization for real Dataset validation; performs provider compatibility and
exact-reference, materialization-readiness, and scope checks before Component
writes; preserves authoritative mutation replays;
projects configuration failures and sanitized diagnostics correctly; and binds
Dashboard disclosure, findings, actions, recovery, and repeat outages to the
exact unexpired semantic authorization context. Owner-controlled fixture
boundaries are AST-enforced, UAT predicates compare authenticated raw evidence,
the acceptance inventory carries exact browser and placement identities, and
the rehearsal graph keeps safe static siblings independent.

These changes supersede the mutable source exercised by Readiness 33. Focused
formatting, parsing, contract self-tests, discovery, compile, Clippy, and narrow
owner integration checks are implementation evidence only. A new complete
Validation Readiness result and complete Candidate Rehearsal result must both
pass against the same clean source and environment identity before preflight.

### Validation Readiness attempt 33 diagnostic correction

The first complete gate against implementation commit
`0cf7dfa0fda3f30cca2b1c1c7f2181ca0ad43045`, tree
`490da6f62b41eb26dfd6458dc76e51fe2bc23ca0`, and environment fingerprint
`6d8b4753386ca703037a49e08758044d9720f86145bb82c644aff81d3b698de8`
finished fail-late with 12 passed checks, two failed checks, and no blocked
checks. Its source-bound receipt and raw logs remain under
`artifacts/sprint-8a-closeout/attempts/readiness-33.json` and
`artifacts/sprint-8a-closeout/readiness-33/`. Candidate Rehearsal attempt 29
did not start.

The two failed checks exposed three harness defects: the readiness runner
called the Component baseline builder and upgrade verifier through removed
parameter contracts, and the materialization safety guard treated Compose's
optional `external` network property as mandatory under strict mode. The
complete bounded runner audit found two additional enforcement defects before
correction: Candidate Rehearsal could fail prerequisite validation before
publishing its start receipt, and its declared lanes omitted the required
release-only known/random resource-reference timing proof. The controlling
five-finding consolidated batch is retained at
`artifacts/sprint-8a-closeout/attempts/readiness-33-consolidated-correction-batch.json`;
all findings are classified `harness`.

The correction advances every caller to the current upgrade self-test
contracts, treats an omitted Compose `external` property as `false`, moves
readiness/source/environment verification inside the receipt-governed
prerequisite lane, and adds an independent
`optimized-resource-reference-timing` lane using the exact release-mode test.
The acceptance contract and runner self-test enforce the new lane and the
start-receipt boundary. These tracked harness and documentation changes are
candidate-affecting, so attempt 33 is superseded for authorization. A complete
unused Validation Readiness attempt and then a complete Candidate Rehearsal
remain required before preflight.

### Post-attempt-28 implementation correction

On 2026-08-06 the user explicitly stopped testing after prolonged failure to
reach preflight and returned Sprint 8A to implementation. The source identity
at exit was commit `7b326838269c3b4218dd579f649a8011feba4d97`, tree
`fa78175797fddba7c7a0b14bb141a33cf4d40fd0`, clean. The append-only local
invalidation receipt is
`artifacts/sprint-8a-closeout/attempts/implementation-return-2026-08-06-invalidation.json`
with SHA-256
`39da96d3672ebd078c551e3179a1b3cf97a4a13671a61627425feed9e894e150`.
It supersedes readiness 31, marks readiness 32 invalid, and records
user-directed abandonment of rehearsal 27 and 28. Candidate freeze, preflight,
SIT, and UAT remain prohibited.

Attempt 28 ended before a protocol-complete harvest because testing was
explicitly exited. Its partial raw evidence remains non-authoritative in
`artifacts/sprint-8a-closeout/rehearsal/materialization-attempt-28.log` and
`artifacts/sprint-8a-closeout/rehearsal/full-validation-attempt-28.log`. Three
observed harness/evidence failures were retained rather than silently retried:

- Validation Readiness parsed database URLs without authenticated reachability
  and produced an environment receipt that Candidate Rehearsal could not
  compare structurally.
- Automated UAT accepted a stale failure-containment receipt by existence
  instead of binding it to the current attempt, source, environment, time, and
  content hash.
- Attempt startup tried to add undeclared `assertions_started_at` data to its
  receipt schema.

The Dashboard duplication is retained separately as a product/architecture
defect, not reclassified as a count-only test failure. Its raw classification
is preserved in
`artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-4-defect-batch.json`
(`overall_classification: product`), with its source-exact invalidation in
`candidate-rehearsal-4-invalidation.json`. That evidence records why the
earlier seven-to-six correction was incomplete: Dashboard simultaneously
appeared as a frozen Core transition and an enrolled Module Release/Instance.
The corrected contract is the exact five Core transition identities, with
Dashboard supplied only by its enrolled manifest. Narrow correction evidence
never discharged the requirement for a new complete Readiness and Candidate
Rehearsal cycle.

The subsequent sprint-plan/architecture audit confirmed one consolidated
implementation-readiness correction batch spanning product, fixtures, tests,
deployment inputs, and validation runners:

- Core's transition catalog is exactly Forms, Workflows, Responses, Datasets,
  and Migration. Dashboard is removed from every canonical transition input
  and can appear only once through its real Release/Instance and manifest.
  Inventory now rejects transition/release identity overlap rather than
  suppressing the duplicate after composition.
- Generic signed-bootstrap receipt bindings carry exact Component owner
  read-back identities into Dashboard seed input before input hashing and
  idempotency. No copied Dashboard-to-Component seed identity or
  Dashboard-specific orchestration branch remains.
- Every module-to-module hop exchanges the verified inbound grant through Core
  for the exact downstream audience. Enrolled manifests declare both the
  consumer request and provider service action; Core accepts only the exact
  lockfile-resolved provider/action/capability tuple and projected live service
  identities. Component and Dataset provider routes contain no sibling caller
  names, caller-specific keys, or first-use identity enrollment.
- Component provider authorization preserves known-versus-random resource
  nondisclosure, while Dashboard presents Components as an installed module
  dependency rather than a Core transition.
- Component-specific lifecycle handling is removed from Core's web host. The
  host renders generic module lifecycle contracts, and extracted Component
  integration coverage exercises the durable module database and API boundary.
- Current navigation and acceptance tests assert exact identities: the five
  Core transitions plus real Scoped Records, Components, and Dashboard
  contributions, ordered 7, 8, and 9. The resolved lockfile policy is
  transactionally projected and is the only mutable navigation source;
  historical seven-entry fixtures are explicitly historical and never reused
  as current expectations.
- Core's startup seed contains no Component or Dashboard capability, role
  grant, service-action, product-route, product-data, or product-asset
  declaration. Those capabilities/actions enter through release enrollment,
  and both products' CSS and JavaScript remain in their owner release images.
  Component direct-load SSR and hydrated visual identity tests now assert the
  owner document rather than obsolete Core loading placeholders.
- Validation Readiness authenticates six pairwise-distinct disposable
  databases and emits one secret-free source/environment fingerprint.
  `localhost`, `127.0.0.1`, and `::1` are one canonical loopback server
  identity for uniqueness, so aliases cannot make one database satisfy two
  required bindings.
  Candidate Rehearsal is now a repository-owned dependency graph with safe
  fail-late siblings, exact blocked reasons, append-only lane evidence, one
  consolidated batch per harvest, and correction authorization only after the
  harvest guard passes.
- Materialization records resolved targets, empty baseline, first apply,
  unchanged semantic no-op, owner receipt bindings, final health, failure logs,
  exact teardown, and source/environment identity. Deterministic failure
  containment proves a clean from-empty successor.
- Canonical bootstrap now has exactly seven Component shells, eight versions,
  and seven Dashboard placements. The Stat Card shell carries a
  superseded/inactive predecessor with a published/active declared successor;
  three independent Dashboard placements bind that predecessor for
  Defer/Upgrade, Replace, and Remove, while the blocked-scope placement remains
  distinct.
- The Dashboard dependency diagnostic exercises those exact live identities.
  It requires exactly three initial lifecycle findings, authorized successor
  disclosure, blocked-scope nondisclosure, Defer followed by Upgrade, Replace,
  Remove, the exact five remaining authorized placements during a Component
  provider outage, and healthy zero-finding convergence after recovery. Its
  structured receipt is a mandatory UAT-8A-05 diagnostic dependency; broad
  lane labels cannot satisfy the scenario. Its check-by-check JSON and SHA-256
  sidecar are retained even when the inner diagnostic fails, and the outer
  fail-late harvest embeds both the partial result and raw artifact identity.
  The independent broad sibling uses the canonical Sprint 8A ownership smoke,
  not the removed Core-era demo Component/Dashboard seed.
- Component calls the versioned Dataset compatibility operation and retains a
  sanitized observation of the selected binding, Core-installation provider,
  contract/version, compatibility, health, observation time, and stable
  result/failure codes. The Component Manifest declares configuration defaults
  `Components` and `5` and uses the real
  `/usr/local/bin/component-module` runtime/migration executable path.
- Component upgrade rehearsal now builds a distinct compatible `0.9.0`
  release from source. Exact one-owner Blueprint deltas are applied by the
  Supervisor through its Compose adapter for baseline establishment,
  `0.9.0` to `1.0.0` upgrade, rollback to `0.9.0`, and intended-`1.0.0`
  restoration; preservation snapshots precede each transition. Supervisor
  finalization commits receipt, operation, emergency override, and override
  reconciliation atomically; Core commits inventory and receipt projection in
  one transaction. Failed projection/finalization retires the unprojected
  receipt, terminalizes the operation, and permits an exact-sequence retry.
  An unchanged desired revision resolves to only `VerifyReadBack`, approves no
  effects, carries prior state, and marks carried bootstrap receipts unchanged.
- Playwright failure evidence and all eight automated UAT diagnostics are
  attempt-, source-, environment-, chronology-, and hash-bound. A failed
  browser run retains reports, traces, logs, test results, and error context.
  Component browser fixtures consume the canonical nested Core-owned Dataset
  major-line reference and derive its positive major from `<uuid>@<major>`;
  removed flat Dataset fields cannot satisfy the test contract.
- The Dataset compatibility contract, Core reference resolver, provider, and
  storage now share one exact major-line identity boundary: canonical
  `<uuid>@<positive-i32-major>`. Core rejects the obsolete colon spelling,
  alternate integer spellings, zero/negative versions, extra separators, and
  values beyond the signed storage range; provider paths contain no unchecked
  unsigned-to-signed major casts.

Focused implementation checks for this batch passed at handoff:

- `cargo fmt --all -- --check`;
- zero-warning `cargo check` and Clippy across all targets/features for the
  Dataset contract, Core API, Supervisor, composition engine, Component
  module, Dashboard module, and web host;
- Supervisor transaction/no-op/rollback tests, composition tests, all-feature
  Component library tests, Dataset-contract tests, and exact Core catalog,
  navigation, manifest-digest, restricted-reference, and Dataset-major
  boundary tests;
- parsing for all 15 changed PowerShell scripts and all four changed JSON
  documents, local Markdown-link validation, and Sprint 8A Compose
  configuration rendering;
- the acceptance contract plus Dashboard semantic, product diagnostic,
  readiness graph, candidate graph, harvest guard, and automated-UAT runner
  self-tests; and
- exact acceptance-manifest discovery of 74 Playwright scenarios, including
  six Sprint 8A Component UI scenarios and the exact Module configuration and
  diagnostic-authority scenario. This was inventory discovery only; no browser
  or deployed-system execution occurred.

These focused checks do not constitute Validation Readiness or Candidate
Rehearsal. Database-backed integration, live deployment, full Playwright,
preflight, SIT, and formal UAT remain Not Run. The next phase must run
`validate-sprint-8a-readiness.ps1` and then the complete repository-owned
`run-sprint-8a-candidate-rehearsal.ps1` against the same clean source and
environment identity.

### Earlier post-attempt-27 correction (superseded context)

The 2026-08-06 implementation audit reconstructed the sprint behavior from the
Sprint 8A acceptance criteria and the closed Sprint 4A/4B/7B Component
contracts instead of treating repeated harness progress as product readiness.
It identified one consolidated implementation-readiness batch:

- **Product lifecycle defect:** the extracted Component module permitted
  transitions outside the closed state machine, including draft lifecycle
  actions and invalid tombstone/reactivation paths.
- **Product authoring defect:** extracted validation accepted unknown keys,
  unavailable fields, invalid type/operator combinations, invalid visual
  modes and limits, and overlong version notes.
- **Product execution defect:** the cross-process Dataset execution path had
  lost table search, runtime projection narrowing, configured/runtime sort,
  complete filters, typed comparisons, unique count, median, do-not-summarize,
  display labels, smoothing, and value/dimension missing-policy semantics.
- **Architecture/fixture defect:** Sprint 8A reused a Sprint 7A fixture helper
  that directly mutated Core, Component, and Dashboard product tables after
  owner bootstrap. This crossed database ownership boundaries and could hide
  incomplete signed bootstrap inputs.
- **Acceptance/bootstrap defect:** the signed Blueprint did not own the full
  recognizable 30-row/blocked-scope Dataset seed and exact Dashboard
  placements, while Sprint 8A acceptance identities still named superseded
  Component versions. Core bootstrap also attempted to write a nonexistent
  Dataset-revision authority column.

The consolidated correction restores the lifecycle and authoring contracts,
extends the typed Dataset execution contract and Core adapter, moves all
Sprint 8A product seed into the owning signed bootstraps, makes the legacy
fixture helper account/security-only for Sprint 8A, and binds acceptance to
exact owner-bootstrap identities. The Dashboard bootstrap now establishes its
own authority revision, and the materialization audit asserts exact Component
external keys rather than a copied count. Validation Readiness now parses the
owner-controlled fixture preparer and Sprint 8A acceptance contract explicitly,
closing the enforcement gap that let seed-path syntax sit outside its runner
inventory.

Focused implementation verification for this mutable batch is recorded below.
These checks do not constitute Validation Readiness or Candidate Rehearsal:

- touched Sprint 8A PowerShell files parse: Passed;
- `cargo check --locked` for Dataset contract, Component module, Core API, and
  Dashboard module: Passed with zero warnings;
- focused Clippy for those four packages across all targets with
  `-D warnings`: Passed;
- Component, Dashboard, and Dataset-contract library tests: Passed (18, 13,
  and 3 tests), and the focused Core Dataset-adapter/bootstrap suites passed
  (3 and 1 tests);
- Sprint 7A/Sprint 8A acceptance-contract evaluation and owner-controlled
  fixture-preparer self-test: Passed.

A fresh complete Validation Readiness run and then a fresh complete Candidate
Rehearsal run are required against the same new clean source/environment
identity before preflight may begin.

Observed on the mutable `codex/sprint-8a` implementation tree on 2026-08-05:

- `cargo fmt --all -- --check`: Passed.
- `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings`:
  Passed with zero warnings.
- `cargo test --workspace --lib --bins --locked --offline` with the two
  database-dependent API tests filtered: Passed. The filtered tests require
  `TEST_API_ENROLLMENT_DATABASE_URL` or `TEST_API_DATABASE_URL` and remain part
  of validation readiness/rehearsal rather than being treated as skips.
- Component, Components v3, Dataset v1, Dashboard module/UI/placement renderer,
  composition, Core navigation/baseline, and module lifecycle focused suites:
  Passed.
- `scripts/check-web-crate-boundaries.ps1`: Passed.
- Sprint 8A PowerShell parser checks and
  `scripts/smoke-sprint-8a.ps1 -SelfTest`: Passed.
- `docker compose -f deploy/sprint-8a/compose.yaml --profile reference config --quiet`:
  Passed.
- Sprint 8A catalog sign, signature verification, and Blueprint resolution:
  Passed against the current canonical manifest digests.

The deployed smoke, destructive materialization, database-backed integration
tests, Playwright, failure teardown/rerun, upgrade/rollback, SIT, and all eight
UAT scenarios remain Not Run. Their execution is reserved for the specialized
validation and UAT workflows.

## Scope And Acceptance Inventory

| Roadmap clause | Risk/contract | Automated proof | Deployed smoke proof | Manual UAT proof |
|---|---|---|---|---|
| Independent full-stack Component module | Hidden Core/build/runtime coupling | native/WASM package graph, forbidden-source, image inventory, route-owner and manifest/testkit assertions | Component process/database/routes/assets healthy while Core product paths are absent | UAT-8A-01/06/08 |
| Canonical packages; no copied source or sibling implementation | Source ownership drift | dependency/source/package/image audits for Component and Dashboard | provenance and image-content read-back | UAT-8A-06 |
| Module owns configuration, manifest, capabilities, database/schema migrations, health, documents and assets | Incomplete extraction | manifest, configuration, capability, schema, probe, document and asset conformance | generic Module Management plus every product/operational route | UAT-8A-01/03 |
| APIs/contracts/typed references only | Cross-database or private DTO coupling | source/SQL/credential and exact-contract integration tests | deny cross-database credentials while product flow passes | UAT-8A-04/06 |
| Exact Components-owned render response | Copied wire DTO, mixed-kind payload or ambiguous identity | Table/visual schema and per-kind field matrix; persistent/preview identity tests; Dashboard exact decode; renderer dependency audit | every deployed kind returns exact contract output without Dataset identity | UAT-8A-01/04/05/06 |
| Exact signed request body | Parsed/reserialized bytes bypass the signed request identity | raw-body authorization-before-deserialization checks; whitespace tamper rejection; media-type/body-limit tests | altered unsigned request bytes fail across Component/Dataset service boundaries | UAT-8A-04/06 |
| Common Dashboard/Component governing node | Cross-product authority across disjoint scope grants | exact resource assertion, authorized-scope intersection, same-node and redacted-projection tests; exact Playwright identity | shared-node placement renders; disjoint placement exposes no title/metadata/identity and cannot render | UAT-8A-04/05 |
| No active legacy Component facade | First-party helper hides retired Core payload | source guard plus permissions/smoke/UAT exact v3 and nested Dataset-reference fixtures | active clients exercise exact current request/response only | UAT-8A-01/04/06 |
| Phase 8 fresh materialization from empty | Partial/stale transition state | reset-target, empty-baseline, owner-order, semantic seed and no-op tests | first source-exact bootstrap plus unchanged second run | UAT-8A-02 |
| Lockfile-owned bootstrap dependency validation | Product branching, payload reinterpretation, replay, or pre-validation writes | Manifest/lockfile/JSON-Pointer resolution; exact signed-binding and negative replay/mismatch tests; zero-write assertions | Component materialization invokes real Dataset validation before owner bootstrap | UAT-8A-02/04 |
| Failed fresh materialization destroys and reruns | Partial topology reused | induced failure, exact teardown, volume absence and complete rerun assertions | retained failed receipt followed by new from-empty healthy attempt | UAT-8A-07 |
| Old transition references and payloads unsupported | Hidden legacy compatibility | historical fixture integrity plus old owner/type/version/payload rejection | old inputs rejected; no adapter, ledger, reader or live old reference | UAT-8A-06 |
| Rerun Phase 7 scope/lifecycle/outage/compatibility | Logical behavior changes at physical boundary | contract, lifecycle, nondisclosure, scope, timeout, compatibility and outage suites | real Component/Dataset/Dashboard process-boundary smoke | UAT-8A-04/05 |
| Rerun source ownership, package graph, independent upgrade/rollback | Coupled or synthetic release | boundary audits; source-built release/binary identity; exact one-owner plan assertions | Supervisor/Compose `0.9.0` to `1.0.0` upgrade, rollback and restoration; unrelated digests/restarts unchanged | UAT-8A-06/08 |
| Move all Component product and operational ownership | Split ownership or missing surface | module product/API/UI/operations integration suites | same-origin module-owned routes and diagnostics | UAT-8A-01/03 |
| Real Release/Instance and fresh database | Wrong identity/provenance/storage | release/instance/catalog/schema/database-isolation tests | composition read-back and exact provenance | UAT-8A-02/06 |
| Dashboard has one canonical inventory/navigation owner | Duplicate transition and deployed-module presentation | exact five-transition identity assertion; Dashboard absent from Core destination resolver; one manifest contribution | inventory exposes Dashboard only as its live 3.0.0 Release/Instance and navigation exposes it exactly once | UAT-8A-02/06 |
| Exact Component/Dashboard acceptance seed | Missing or non-executable lifecycle coverage | exact seven-shell/eight-version/seven-placement identity assertions, including predecessor/successor and three action placements | every placement resolves through Components v3; exact action fixtures are live; zero old owners/types | UAT-8A-02/05 |
| Typed Core Dataset compatibility references/contracts | Direct Dataset relationship retained | typed composite identity, exact version, source/credential audits | Component authors/renders across Core contract only | UAT-8A-04 |
| Preserve Dashboard public Components behavior | Contract or consumer-policy regression | Sprint 7B regression suites plus exact structured dependency diagnostic | Defer/Upgrade/Replace/Remove, nondisclosure, five-placement Component outage and zero-finding recovery across real provider | UAT-8A-05 |
| Rebuild seed/test data through owners | Stale fixtures or cross-owner writes | semantic owner/read-back/idempotence and negative credential checks | recognizable full reference app from owning APIs | UAT-8A-02/06 |
| Outage, timeout, scope, capability and compatibility | Leak, corruption or cascading failure | authorization/nondisclosure matrix and failure-containment tests | stop/degrade Dataset or Component; unrelated services healthy; recovery converges | UAT-8A-04/05 |
| UI unchanged plus module configuration/diagnostics | Visual/interaction or configuration regression | Playwright at 1280/768/390, dark/light, keyboard, 200%, no-JS, hydration and console | direct/shell documents and exact configuration/diagnostics | UAT-8A-01/03 |
| Exit: fresh seed contains only new Component references | Wrong owner/type or incomplete seed | source-exact semantic read-back and old-input absence/rejection | all Dashboard placements resolve against selected Component instance | UAT-8A-02 |
| Exit: author/execute Components through Dataset contract | Product parity, authorization or isolation failure | every Component kind, scope, timeout, contract and credential tests | author and execute recognizable fixtures through gateway | UAT-8A-04 |
| Exit: Dashboard continues consumption and degrades coherently | Cascading outage or disclosure | Dashboard dependency/lifecycle/outage regression | stop/recover Component and Dataset provider paths | UAT-8A-05 |
| Canonical Component API break | Stale first-party consumer or hidden translation | exact v3 request/response golden tests and old-shape failures | UI, Dashboard, seed, smoke and UAT use v3 only | UAT-8A-01/05/06 |
| Exact configuration product decisions | Label/timeout drift or over-broad authority | schema v1, default/range, normalization, display and authority tests | navigation/Module Management label and 5-second effective timeout | UAT-8A-03/04 |

Every row requires automated and manual evidence. No `N/A` is currently
planned. If a proof becomes unsafe or inapplicable, amend this inventory before
freeze with a concrete rationale and equal-or-stronger evidence.

The current Playwright acceptance manifest declares 75 exact scenario
identities. The added identity is `Sprint 7A scoped analytics boundary ›
Dashboard and Component scopes must share a governing node before disclosure
or render`; it is mapped to UAT-8A-04 assertion
`joint-dashboard-component-scope`. This is an inventory declaration, not a
browser execution result.

## Required Evidence Inventory

Readiness 38 is the one legacy recovery boundary for the failed-Readiness
finalizer introduced by this correction. Its complete terminal receipt, raw
logs, exact two-failure/one-block inventory, and documented consolidated batch
were frozen before source edits, but the prior runner could not publish typed
Readiness harvest/batch/authorization artifacts. The user-directed testing exit
therefore permits only this finalizer and enforcement correction to be committed
first. The clean commit must then run `-Attempt 38 -FinalizeFailedAttempt`
without rerunning checks or starting R39; no other attempt may use this
exception.

| Artifact | Producer | Required before | Status |
|---|---|---|---|
| `attempts/readiness-38.json` and sidecar | Validation coordinator | R38 correction finalization | Failed; retained with 12 pass, 2 fail, 1 block |
| `attempts/readiness-38-harvest.json`, `attempts/readiness-38-defect-batch.json`, and `attempts/readiness-38-correction-authorization.json` | Validation coordinator | Readiness 39 | Not Run; sole legacy R38 recovery, required after the clean correction commit and before R39 starts |
| `validation-readiness-result.json` | Validation coordinator | Rehearsal | R38 failed; Readiness 39 Not Run |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | R30 failed and retained; Rehearsal 31 Not Run |
| `attempts/preflight-<n>.json` | Preflight | Preflight terminal decision | Not Run |
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `attempts/sit-<n>-start.json`, `attempts/sit-<n>.json`, and `sit/attempts/<lane>-<n>.json` | SIT | SIT terminal decision | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
| `attempts/uat-<n>.json` and `attempts/uat-<n>-manual-checkpoint.json` | UAT | Formal UAT finalization | Not Run |
| `docs/sprints/sprint-8a-uat/scenario-contract.json` | Sprint plan / acceptance contract | Preflight and manual-UAT start | Tracked canonical eight-scenario contract |
| `uat/attempt-<n>/manual-leases/uat-8a-<nn>-start.json`, optional `-resume.json`, `-publication-prepared.json`, and `-complete.json` | UAT manual publisher | Each manual receipt / Finalize | Not Run |
| `uat/attempt-<n>/finalizations/run-<nnn>/finalization-completion-checkpoint.json` | UAT Finalize | Canonical UAT publication or finalization-only retry | Not Run |
| `uat/attempt-<n>/finalizations/run-<nnn>/result-commit.json` | UAT Finalize | Canonical UAT publication | Not Run |
| `uat/attempt-<n>/finalizations/run-<nnn>/publication-retry-checkpoint.json` and `uat/attempt-<n>/publication-retries/run-<nnn>/` | UAT Finalize | Publication-only retry, when triggered | Planned / Conditional |
| `uat-result.json` | UAT | Authorization | Not Run |
| `uat-defect-harvest.json` | UAT/coordinator | Correction batch, when triggered | Planned / Conditional |
| `defect-batch.json` | Coordinator | Impact assessment, when triggered | Planned / Conditional |
| `correction-impact-assessment.json` | Coordinator | Focused repair validation, when triggered | Planned / Conditional |
| `focused-repair-validation/attempt-<n>.json` | SIT/UAT/coordinator | Convergence, when triggered | Planned / Conditional |
| `canonical-restoration.json` | Coordinator | Convergence/final certification, when triggered | Planned / Conditional |
| `final-certification-entry.json` | Coordinator | Final readiness/rehearsal, when triggered | Planned / Conditional |
| `evidence-manifest.json` and `.sha256` sidecar | Validation phases | Authorization | Not Run |
| `closeout-authorization.json` | Coordinator | Closeout | Not Run |

Additional required evidence namespaces:

- `readiness/`: executable checklist, destructive-target authorization,
  tool/shell/runner self-tests, semantic fixture audit, acceptance mapping and
  clean-source proof.
- `rehearsal/`: static, Rust, Playwright, source-exact fresh deployment,
  contracts/nondisclosure, smoke, teardown/rerun and automated-UAT diagnostics.
- `sit/`: lane receipts, raw logs, deployment/provenance, empty baseline,
  first/no-op bootstrap, old-input rejection, outage/recovery,
  upgrade/rollback and final convergence.
- `uat/attempt-<n>/`: scripted inventory/smoke logs, final source-exact
  materialization/no-op, canonical inventory/smoke restoration, and final
  health evidence.
- `uat/attempt-<n>/manual/`: UAT-8A-01 through UAT-8A-08 receipts,
  screenshots/traces and reviewer sign-off, all bound to the exact formal-UAT
  attempt.
- `uat/attempt-<n>/manual-leases/`: one authenticated start/completion pair and
  immutable prepared-publication checkpoint per scenario, plus an authenticated
  process-lineage resume marker only when interruption requires it. The
  evidence-root lock is held from lease start through transactional receipt and
  completion publication; an open or interrupted lease blocks Finalize.
- `uat/attempt-<n>/finalizations/` and `publication-retries/`: append-only
  canonical-restoration runs, durable completion checkpoints, result commits,
  retained publication failures, and any authorized publication-only retry.
- `materialization/`: resolved destructive targets, empty baseline,
  owner-bootstrap order/read-back, first/no-op results, induced failed attempt,
  teardown proof and successor from-empty result.
- `upgrade-rollback/`: baseline/candidate/rollback identities and unrelated
  service digest/restart chronology.

## Candidate Identity

- Implementation commit: To be frozen after passing readiness and rehearsal.
- Tree: To be frozen.
- Dirty state: Must be clean at freeze.
- Candidate fingerprint: Not issued.
- Acceptance-inventory identity: SHA-256 of this record plus tracked acceptance
  runners, tests, fixtures, seeds, manifests, bootstrap and deployment inputs.
- Deployment identity: one digest covers the tracked Sprint 8A deployment
  profile and manifests; a separate
  `normalized_deployment_configuration_sha256` is the SHA-256 of the exact live
  normalized `docker compose ... config` output retained by the environment
  contract. Candidate, preflight, SIT, and UAT receipts must bind that same
  normalized digest rather than re-hashing YAML as a substitute.
- Schema/baseline identity: empty Core, Component, Dashboard, Supervisor and
  selected-module schema migration digests plus bootstrap/seed contract
  versions. No legacy populated-data baseline exists.
- Expected provenance labels: OCI revision, source tree, dirty state, build
  profile, module definition/release, manifest, asset, schema migration,
  contract, fixture, seed and acceptance-inventory digests.
- Observed image digests: Not Run.

## Validation Readiness

- Latest completed check graph: Readiness 38 retained 12 passes, 2 failures,
  and 1 exact block against clean source `91c9936b`, tree `564493fd`; its
  environment remained unverified because the runner process omitted a
  required database binding. It consumed the R30 authorization and is retained
  as immutable failed evidence. Readiness 37 remains prior-source pass evidence
  only.

- Derived executable checklist: regenerate from the final plan, this inventory,
  source, runners, Compose profile and evidence schemas.
- Environment variables and reset acknowledgements: enumerate exact Sprint 8A
  installation ID, gateway/Supervisor ports, database URLs/role identities,
  signing key IDs, service endpoints, Compose project, every disposable
  database/container/volume target, explicit destructive authorization and
  evidence root without recording secrets.
- Supported tools, shells and runtimes: PowerShell runner parsing/invocation,
  Rust/Cargo, wasm tooling, Node/npm/Playwright, Docker/Compose, PostgreSQL
  client, browser and SHA-256 helpers.
- Ports, topology, health and provenance: one gateway, Core, Supervisor,
  Component, Dashboard, Dataset compatibility provider and required reference
  services; isolated stores/roles; intended handoff slot; exact image labels.
- Semantic fixture and idempotence audit: administrator, global Module
  Management reader/manager, Component manager, scoped Dashboard manager,
  constrained reader, out-of-scope actor, service identities, all Component
  kinds, lifecycle states, Dataset major lines, allowed/blocked references,
  recognizable output, v3 and historical negative inputs, timeouts/outages and
  unchanged second-run behavior.
- Reset safety: resolve absolute Compose project and every exact disposable
  volume/database/container target; reject missing/ambiguous/broad targets and
  prove deletion cannot escape the declared environment.
- Runner/output/receipt/hash/finalization self-tests: safe disposable probes for
  every argument/output path, atomic completion, SHA-256, failure/supersession,
  heartbeat, teardown and completion sentinel.
- Attempt lifecycle: acquire the evidence-root OS-exclusive lock and
  authenticate current state, pending authorization, attempt number, and the
  empty five-target namespace before publishing an `unverified` live receipt
  plus immutable hashed start snapshot; probe source/environment only after
  publication; and publish a sidecar-verified attempt/state checkpoint after
  every terminal pass, failure, or block.
- Correction transition: after one complete failed-rehearsal harvest and batch,
  permit exactly one successor Readiness start through an append-only
  consumption receipt bound to the predecessor and successor start snapshot.
  Reject duplicate consumption, active attempts, used attempt identities, and
  any Readiness receipt other than the exact path/hash currently named by
  validation state.
- Acceptance mapping: every inventory row maps to SIT, smoke and UAT evidence.
- Clean repository and source-exact inputs: required before rehearsal.
- Result receipt: `artifacts/sprint-8a-closeout/validation-readiness-result.json`.

## Candidate Rehearsal

Attempt 30 is terminal and may not resume. It retained 18 passes, 4 raw failed
lanes, and 10 exact dependency blocks against the now-superseded Readiness 37
source and environment. Its complete harvest, consolidated four-symptom batch,
and correction authorization were consumed exactly once by failed Readiness
38. After the clean implementation correction is committed, R38 finalization
must issue one pending exact-R39 authorization. Readiness 39 must consume that
new authorization and pass against the corrected source/environment identity
before Candidate Rehearsal 31 may start.

Sprint 8A rehearsal uses the dependency-aware inventory below. The attempt
receipt declares this graph before assertions. After any failure the attempt is
`harvesting`: independent siblings continue, dependent checks are recorded as
blocked with the exact failed prerequisite, and no tracked correction or new
attempt may begin until one harvest receipt and one consolidated defect batch
account for every row. `scripts/test-sprint-validation-harvest.ps1` enforces the
terminal-state and single-batch contract; materialization retains failed apply
responses and service logs before exact teardown.

The start receipt initially carries unverified source/environment identities.
The first lane acquires the evidence-root OS-exclusive lock and checks that the
supplied passing Readiness path and SHA-256 exactly equal the current validation
state, including any one-use predecessor-correction consumption. Safe static
lanes remain independent of those fallible prerequisites; only true dependents
or unsafe destructive work are blocked.

The automated-UAT lane projects each failed semantic predicate as its exact
`UAT-8A-xx/assertion-id` identity with classification, reason, and hashed raw
evidence. If only nested semantics fail, the outer projection lane remains
passed so the consolidated batch contains one defect per failed predicate and
no generic outer duplicate. Wrapper or parsing failures remain separate harness
defects, and blocked scenarios retain their exact dependency reasons.

Every next-attempt lane records whether assertions actually started and the
aggregate counts only those executed lanes. Structured child classification is
canonical when present and carries its source into the outer receipt and
harvest. Final environment comparison retains expected and actual fingerprints
plus changed contract sections, and every newly persisted evidence reference is
canonical, repository-relative, and contained by the evidence root.

The following table is the planned lane summary for the next successor
Candidate Rehearsal, not a restatement of R30's terminal results.

| Diagnostic lane | Planned command/evidence | Assertions | Successor result | Defect batch |
|---|---|---|---|---|
| Static and boundaries | fmt/check/Clippy, manifests, links, native/WASM graphs, source/image audits | zero warnings; no forbidden owner/dependency/route/storage/legacy edge; exact-body and joint-scope enforcement present | Not Run | |
| Full Rust | `cargo test --workspace --locked` plus targeted contract/schema/authorization tests | all pass, including render kind/identity, raw-body tamper, exact assertion and same-node matrix | Not Run | |
| Source-exact materialization | authorized reset, empty schemas, first/no-op owner bootstrap | exact provenance, healthy topology, semantic seed, exact no-op; manifest-declared target and canonical opaque payload; request-bound one-use authorization; provider-owned result; no product-specific Core/Supervisor branch; zero writes on mismatch, expiry, replay, incompatibility, or outage | Not Run | |
| Playwright | `scripts/validate-e2e.ps1` with exact gateway, deployment receipt, fresh-state, Sprint 8A profile, and evidence bindings | complete inventory; zero unexpected skip/retry/flake; retained outputs | Not Run | |
| Conformance and nondisclosure | module testkit plus Components/Dataset/Dashboard matrix | owner/version/scope/audience/known-random/timing/lifecycle cases pass; shared-node render succeeds and disjoint metadata/render stay restricted | Not Run | |
| Deployed smoke | general and Sprint 8A smoke in rehearsal namespace | real boundaries, fixtures, old-input rejection, outage/recovery and final health | Not Run | |
| Live product diagnostics | attempt-bound product receipt plus raw structured Dashboard dependency JSON/SHA sidecar | exact predecessor/successor placements; Defer/Upgrade/Replace/Remove; blocked nondisclosure; five-placement Component outage; zero-finding recovery; partial evidence retained on failure | Not Run | |
| Component release transition | source-built `0.9.0` metadata plus Supervisor/Compose apply receipts and stage snapshots | exact one-owner `0.9.0`/`1.0.0` upgrade, rollback and restoration; Component preservation; unrelated identity stability | Not Run | |
| Failure teardown/rerun | induced partial materialization failure | evidence retained; exact topology/volumes removed; new empty rerun healthy | Not Run | |
| Automated UAT diagnostics | automated equivalents of UAT-01 through UAT-08, including the structured live-product receipt | every precondition and expected semantic state reproducible; UAT-8A-04 includes `joint-dashboard-component-scope`; failed nested predicates retain exact identity/classification/reason/raw hash without outer double count; UAT-8A-05 cannot pass from lane labels alone | Not Run | |

The first two lanes are independent of a deployed topology. Playwright locked
installation/discovery, runner self-tests, and acceptance-inventory checks are
also independent. Healthy materialization/no-op is the prerequisite for live
Playwright execution, deployed smoke, deployed inventory/navigation audit,
automated UAT diagnostics, failure-containment successor health, and the
Component upgrade/rollback baseline. Formal deployed acceptance smoke remains
SIT-owned and is not a rehearsal substitute.

The Sprint 8A reference materialization rebuilds the canonical seed through
signed owner bootstrap and receipt read-back before deployment evidence is
captured: exactly seven Component shells, eight versions, and seven Dashboard
placements, including the predecessor/successor and independent action
fixtures. The legacy helper prepares only account/security state. This is a
pre-production rebuild, not a compatibility migration. The complete Playwright
inventory must never rely on one earlier test file to create shared fixtures.
Exact deployed
inventory/navigation evidence is produced by
`scripts/audit-sprint-8a-deployed-inventory.ps1`; ad hoc route guesses, envelope
parsers, and copied counts are not rehearsal evidence.

Logical independence does not authorize concurrent Cargo commands to share the
repository `target` directory. `scripts/validate.ps1` cleans selected build
artifacts, so complete Rust siblings run serially unless each has a distinct
explicit `CARGO_TARGET_DIR`. Attempt 12 retained the missing-executable failure
that demonstrated this enforcement gap.

After removing the old Core-owned Component and Dashboard transition entries,
the fresh default Main placements after Datasets are Scoped Records at 7,
Components at 8, and Dashboard at 9. The Sprint 8A blueprint and acceptance
contract pin those exact identities and orders; Dashboard remains a single
manifest-owned contribution.

Readiness must run `npm ci --prefix end2end` and complete Playwright discovery,
not merely read the declared package version. It must also parse and self-test
the harvest guard and verify that `scripts/uat-sprint-8a.ps1` plus all eight
`docs/sprints/sprint-8a-uat/uat-8a-*.md` scripts exist.

Complete validation requires six pairwise-distinct, freshly created disposable
database bindings: API, fresh-baseline API, reference module, extracted
Component module, API enrollment, and Installation Control. A database
identity used by one owner integration target is not reused by another target
in the same rehearsal.

- Mutable source/environment identity: Attempt 30 verified clean commit
  `84964c7bdb6b5d4705a2e4899a1fe2c98ee77183`, tree
  `4f39cb7126dbe6f24db77f4d644593fb5ef9f0ca`, and environment fingerprint
  `96c2a32ed16dfb288a4ca2578c171413d727c16153f5fbed3dd6dff9c2b410b3`;
  all are superseded by the current candidate-affecting correction.
- Passing readiness prerequisite: Readiness 37 passed for Attempt 30's prior
  source and environment; no passing prerequisite exists for the next attempt.
  Its result SHA-256 is
  `a4ab361af6606ff94eaef8c5be5ca3e864dc90cb41efabdcb3aba73014eacf9b`.
- Current consolidated defects and correction batch: Attempt 30 retained 18
  passes, 4 raw failures, and 10 dependency blocks; its three diagnosed harness
  roots were corrected before R38. Attempt 38 then retained 12 passes, the
  missing database-binding and failure-containment self-test failures, and the
  exact blocked environment-contract reason. The implementation audit adds the
  failed-Readiness correction lineage, exact Rehearsal terminal checkpoints,
  retired Component `missing_policy` removal, and canonical UAT/AC-18/AC-19
  evidence enforcement. The correction is candidate-affecting and remains
  formally unverified until complete successor gates pass.
- Historical consolidated defects and correction batches: Attempt 15 retained six
  Playwright findings in one fail-late batch after 10 independent checks
  passed. Four acceptance-inventory defects used the former Core Component
  `id`, the removed searchable Dataset Version picker, or stale generic Dataset
  dependency copy. One Component product defect disclosed a known unreadable
  identity as `403` instead of making it indistinguishable from a random `404`.
  One Dashboard product-asset defect served release `3.0.0` with entry imports
  pinned to `2.1.0`, preventing module hydration. The remaining 34 serial-suite
  cases were retained as blocked (2 Components, 8 Dashboards, 24 Permissions),
  and the all-prerequisite final check was blocked specifically by Playwright.
  Containment and all eight non-authoritative UAT diagnostic mappings still
  passed. The correction updates canonical identities/interactions, closes
  Component nondisclosure, binds Dashboard assets and digests to release
  `3.0.0`, and requires complete readiness and rehearsal restart. Two narrow
  dirty-source materialization invocations reached only the release build and
  were stopped by the external 15-minute command timeout; they are retained as
  diagnostic build evidence and do not replace either full gate. Attempt 14's
  two-finding, Attempt 13's ten-finding, and Attempt 12's eight-finding batches
  remain retained and superseded.
- Attempt 16 retained one deployment-input defect through three independent
  observations: Supervisor apply failed closed and both complete Rust suites
  reported the same Dashboard manifest/catalog mismatch. Tessara's manifest
  identity is the SHA-256 of RFC 8785/JCS canonical JSON produced by
  `tessara_composition::canonical_digest`, not a hash of platform-dependent
  working-tree bytes. The catalog now uses the canonical `cec6af45...` digest;
  the existing `checked_catalog_manifest_digests_match_runtime_manifests` test
  is the governing derived assertion and deliberately remains the single
  enforcement point.
- Attempt 17 retained two operator-harness defects as one batch after seven
  independent checks passed. Manual orchestration omitted the exact Core,
  gateway, and database container identities required by deployment-evidence
  capture, then invoked general smoke without its evidence/data-state bindings;
  the independent Sprint 8A semantic smoke still passed all 26 checks. The
  manual Component exercise also supplied the invented `-OutputImage`
  parameter instead of the baseline builder's declared `-OutputTag`. Playwright
  execution was blocked on deployment evidence; UAT diagnostics and final
  health were blocked on their declared failed prerequisites. Repository-owned
  wrappers now resolve these identities and arguments, and readiness parses and
  self-tests both wrappers so the commands cannot drift back to ad hoc manual
  assembly.
- Attempt 18 retained one operator-harness defect after four independent lanes
  passed: a 60-second client observation timeout terminated the active
  materialization build. No tracked input changed; the corrected execution used
  a non-terminating command boundary with short observation waits, and the
  complete readiness/rehearsal cycle restarted.
- Attempt 19 retained four first-failure findings after eight checks passed: the new general
  smoke wrapper incorrectly requested the forbidden Sprint 6A acceptance
  schema; one Component test still expected the removed projection-builder UI;
  Dashboard document deserialization expected `component_reference` where the
  canonical Components V3 metadata envelope exposes `reference`; and the
  permission suite inferred scoped Component fixtures from seed coincidence.
  Narrow correction verification then exposed one underlying product/seed
  finding in that same consolidated batch: demo seeding used a duplicate
  Dataset-major materializer that omitted the canonical restriction-tier and
  semantic-version columns, making freshly rebuilt seeded Components fail at
  execution. The batch therefore contains five defects, not a restarted
  rehearsal or a second micro-batch.
  The consolidated correction removes authoritative evidence publication from
  rehearsal smoke, tests the simplicity-first Configuration JSON form, aligns
  Dashboard document types with the V3 contract, and creates exact named scoped
  Component fixtures, and makes demo seed major lines conform to the canonical
  Dataset materialization schema. Playwright retained 33 passes, three first failures, and
  34 serial dependents; UAT diagnostics and final health were dependency-blocked.
- Attempt 20 retained three findings after nine checks passed. Playwright
  completed 48 tests, retained two first failures, and marked 20 serial
  dependents blocked: the Dashboard placement renderer still decoded the
  removed `dataset_id`/`dataset_version_major` response fields instead of the
  canonical Components V3 `dataset_reference`, and scoped Component execution
  disclosed a known hidden identity as `403` instead of the required
  nondisclosing `404`. The independently passing upgrade/rollback wrapper then
  exposed a harness handoff defect: a caller-selected evidence path was not
  accompanied by the canonical receipt consumed by UAT diagnostics. Live UAT
  smoke still passed, but scenario mapping failed on that exact missing
  prerequisite; the aggregate final check was blocked by Playwright and UAT.
  The consolidated correction aligns Dashboard render DTOs, makes scoped
  execution nondisclosing, and makes the upgrade wrapper atomically retain the
  canonical receipt plus any requested evidence copy.
- Attempt 21 retained two findings after ten checks passed. Playwright
  completed 48 tests, retained two first failures, and marked 20 serial
  dependents blocked. The deployed Dashboard image embedded the tracked
  `dashboard.wasm` generated before the Components V3 response change, so its
  decoder still required removed `dataset_id`/`dataset_version_major` fields
  even though the current Rust DTO and provider response used the canonical
  typed `dataset_reference`. The permission inventory also assumed Core's
  `#app-root` for independently deployed Component documents whose exact owned
  root is `#module-content`; the sibling JavaScript-disabled Component routes
  carried the same copied assumption. All other rehearsal lanes, including
  source-exact materialization, inventory/navigation, deployed smoke,
  upgrade/rollback, all eight UAT diagnostics, complete Rust validation,
  all-feature tests, Clippy, and containment health passed. The aggregate final
  check was blocked only by Playwright. The consolidated correction rebuilds
  and re-pins Dashboard bindings/WASM, tests the actual embedded asset hashes
  and V3 wire identity during readiness, and binds every Component route
  assertion to its exact module document root.
- Attempt 22 retained two findings after ten checks passed. Playwright completed
  52 tests, retained two first failures, and marked 16 serial dependents
  blocked. The Dashboard paging scenario selected the first available Table by
  type and then assumed every identity other than the Sprint 7A table had more
  than ten rows; the selected Sprint 8A reference table correctly had four.
  Separately, the independently deployed Component complete-document script
  installed its behavior but did not publish the shared
  `data-hydration="ready"` lifecycle marker. Source-exact materialization/no-op,
  inventory/navigation, deployed smoke/evidence, Sprint 8A semantic smoke,
  Component upgrade/rollback, all eight UAT diagnostics, complete Rust
  validation, all-feature tests, warning-free Clippy, exact Playwright
  discovery, and containment health passed. The aggregate final check was
  blocked only by Playwright. The consolidated correction binds paging to the
  exact multi-page demo Session Log Table and makes Component complete
  documents publish and test the shared hydration marker.
- Attempt 23 retained one harness finding after six independent checks passed
  and six deployment-dependent checks were blocked with exact reasons. The
  source-exact materialization command failed before destructive work when the
  documented interactive `-Confirm` prompt was invoked through the rehearsal
  evidence pipeline. Complete Rust validation, all-feature tests, warning-free
  Clippy, then-current exact Playwright inventory discovery, clean source, and current
  environment containment health passed. Inventory/navigation, deployed
  smoke, Playwright execution, Component upgrade/rollback, UAT diagnostics,
  and the aggregate final check were correctly blocked because no corrected
  source-exact topology existed. A captured-output `WhatIf` reproducer passed
  with `-Confirm:$false`; the correction makes that automation-safe invocation
  canonical and readiness-enforced without weakening the explicit disposable
  reset authorization or exact-target guards.
- Validation Readiness attempt 27 completed all 13 sibling checks and retained
  one harness failure in the new reset dry-run enforcement. The guarded
  `WhatIf -Confirm:$false` invocation returned successfully, but PowerShell
  rendered the `WhatIf` message directly through the host rather than into the
  assigned pipeline, and readiness incorrectly required that host-only text in
  the capture. The corrected check exercises the same capture boundary and
  requires successful return without treating host rendering as pipeline
  evidence. Candidate Rehearsal attempt 24 did not begin.
- Attempt 24 retained one shared harness/environment-contract finding after 11
  independent checks passed. Bare `npm test` used the Playwright development
  default at `127.0.0.1:8080`, so 30 tests failed with `ECONNREFUSED` and 40
  serial dependents did not run even though the source-exact Sprint 8A gateway
  was healthy at `127.0.0.1:8088`. Materialization/no-op,
  inventory/navigation, deployed smoke, Component upgrade/rollback, all eight
  UAT diagnostics, complete Rust validation, all-feature tests, warning-free
  Clippy, discovery, and containment health passed; only the aggregate final
  check was blocked. The correction makes the existing repository-owned
  `validate-e2e.ps1` runner canonical for rehearsal and makes readiness parse,
  self-test, and require its exact gateway/deployment/fresh-state/profile/
  evidence bindings.
- Attempt 25 retained two findings after 11 safe independent/dependency-valid
  checks passed. Playwright completed 52 tests, retained two first failures,
  and marked 16 serial dependents not run. The Dashboard paging scenario still
  named the removed Core demo Table instead of the canonical module-owned
  `sprint-8a-record-table`; the canonical four-tier Dataset also had only four
  rows and therefore could not prove paging. Narrow correction diagnosis kept
  the same batch open and exposed the underlying product contract gap: the
  Component provider discarded Dashboard cursors while Core rejected Dataset
  execution cursors and always returned no successor. The Component detail
  document separately allowed its version table to impose a 558-pixel
  min-content width in a 390-pixel viewport. The consolidated correction binds
  paging to the exact module identity, extends the canonical tier fixture with
  26 deterministic public rows, preserves/validates bounded forward cursors,
  performs deterministic limit-plus-one Dataset paging, and constrains the
  module page/panel grid items so the table owns its narrow overflow. The
  aggregate final check was blocked exactly because Playwright did not pass;
  all other lanes, including materialization/no-op, inventory/navigation,
  smoke, upgrade/rollback, all eight UAT diagnostics, complete Rust checks,
  Clippy, discovery, and containment health, passed.
- The restart-behavior audit found the protocol and readiness references now
  state the fail-late rule completely. The earlier enforcement gap was
  executable orchestration: ad hoc commands could bypass the dependency graph,
  harvest terminal-state guard, or source-exact Playwright wrapper. The
  repository-owned harvest guard, its readiness self-test, the declared attempt
  graph, and the readiness-enforced `validate-e2e.ps1` binding close that gap;
  no duplicate skill prose is added. Attempt 25 exercised this enforcement by
  retaining one harvest and one consolidated batch before tracked correction.
- Attempt 26 retained six findings after nine checks passed. Playwright
  completed 51 tests, retained three failures, and marked 16 serial dependents
  not run: the four named tier identities were incorrectly required on the
  first deterministic 25-row page; Core's generic module gateway dropped the
  browser query before the Dashboard module, so the embedded 10-row request
  became the Component provider's 25-row default; and the scoped Component
  route still expected two Core-era admin fetches even though module-owned SSR
  now hydrates without them. Materialization/no-op, exact inventory/navigation,
  deployed smoke, complete Rust validation, workspace all-features, warning-free
  Clippy, exact discovery, and containment health passed. The Component upgrade
  sibling failed before assertions because the operator supplied undeclared
  `-Overwrite` and was not retried. UAT live smoke passed, but its UAT-8A-08
  mapping incorrectly accepted a stale prior-source upgrade receipt. The first
  harvest-finalization call similarly used undeclared `-BatchPath`; immutable
  results were complete, so only finalization was rerun with the declared
  `-DefectBatchPath`. Those two operator findings require no tracked command
  change because the scripts already expose the exact parameters and the
  canonical command omits `-Overwrite`.
- The consolidated correction preserves exact query strings through the
  manifest-driven module gateway and tests that boundary, explicitly requests
  all 30 semantic rows for the tier identity assertion, removes stale admin-GET
  expectations from module-owned Component documents, stamps upgrade evidence
  with the clean source identity, and makes UAT diagnostics reject an upgrade
  receipt from another source or from before the current materialization. The
  UAT self-test exercises the stale-receipt rejection. Readiness 30 and
  rehearsal 26 are superseded by this tracked correction.
- Readiness 31 is superseded. Readiness 32 is invalid because it did not prove
  authenticated database reachability or issue a structurally shared
  environment contract. Rehearsals 27 and 28 were explicitly abandoned when
  the user exited testing; neither reached a complete terminal graph or may be
  resumed. Attempt 28's three partial harness/evidence findings and raw logs
  are retained under the invalidation receipt named above. The subsequent
  implementation audit and consolidated correction replace those attempts;
  they do not convert their partial evidence into a rehearsal result.
- Readiness 36 subsequently passed its complete mutable gate, and Rehearsal 29
  began against that exact clean source/environment pair. Rehearsal 29 was
  stopped during harvesting under that user-directed return to
  implementation; its 8 passes, 3 failed receipts, 1 interrupted lane, 20
  unexecuted checks, and 0 recorded blocks remain diagnostic evidence only.
  The attempt may not resume, and the then-current candidate-affecting
  correction superseded Readiness 36.
- Readiness 37 then passed against `84964c7b`/`4f39cb71` and environment
  `96c2a32e...`. Rehearsal 30 completed its terminal graph with 18 passes, 4
  raw failures, and 10 blocks, retained one harvest/batch/authorization chain,
  and returned to implementation for the three-root harness correction. It may
  not resume, and the correction supersedes Readiness 37.
- Complete-cycle repetitions after the current correction: 0.
- Result receipt: `artifacts/sprint-8a-closeout/candidate-rehearsal-result.json`.

Rehearsal is mutable, diagnostic and non-authoritative. Any correction requires
complete readiness and complete rehearsal to repeat before freeze.

## Environment Contract

- Corrected successor environment fingerprint: Not issued. The superseded
  R37/R30 fingerprint is
  `96c2a32ed16dfb288a4ca2578c171413d727c16153f5fbed3dd6dff9c2b410b3`.
- Intended gateway: `http://127.0.0.1:8088`; intended Supervisor endpoint:
  `http://127.0.0.1:8098`.
- Compose project/profile: `tessara-sprint-8a`,
  `deploy/sprint-8a/compose.yaml`, required profiles explicitly enabled.
- Validation database bindings: `TEST_API_DATABASE_URL`,
  `TEST_API_FRESH_DATABASE_URL`, `TEST_REFERENCE_MODULE_DATABASE_URL`,
  `TEST_COMPONENT_MODULE_DATABASE_URL`, `TEST_API_ENROLLMENT_DATABASE_URL`, and
  `TEST_INSTALLATION_CONTROL_DATABASE_URL`. Readiness requires pairwise-
  distinct loopback, token-bounded test database identities and completes an
  authenticated transactional temporary-table round trip against each one.
  All six bindings, the destructive-reset acknowledgement, and
  `TEST_POSTGRES_CLIENT_CONTAINER_ID` must be exported in the same PowerShell
  process that invokes the top-level Readiness runner; a prior shell process
  does not satisfy the contract.
- Materialized owner databases: empty disposable installation-scoped Core,
  Component, Dashboard, Supervisor, and selected-module databases with
  owner-specific runtime and schema-migration roles.
- Reset authorization: explicit approval bound to resolved absolute Sprint 8A
  project/container/volume/database identities only. Broad targets, unresolved
  variables and neighboring projects fail closed.
- Account/role fixtures: administrator, global Module Management reader/manager,
  Component manager, scoped Dashboard manager, constrained reader, out-of-scope
  actor and Core/Component/Dashboard service identities; no secrets in receipts.
- Evidence root/output mode: `artifacts/sprint-8a-closeout/`, repository-relative
  canonical paths, append-only attempts and atomic completion where supported.
- Identity binding: readiness writes a secret-free environment contract and
  SHA-256 fingerprint. Every rehearsal lane, raw artifact, UAT diagnostic, and
  aggregate result must match that fingerprint and the same source identity.

## Next Authorized Commands

No formal gate is authorized while the consolidated correction is mutable.
After review, focused verification, and a clean correction commit, first
finalize the retained terminal R38 evidence. That command creates no R39 start
or product assertion. The mutable-build coordinator may then enter Readiness
and Rehearsal only through the exact top-level runners shown below. Those
runners own the complete check inventory, dependency graph, evidence paths,
and subordinate commands; manually assembling lanes is not a Candidate
Rehearsal and cannot authorize preflight.

```powershell
.\scripts\validate-sprint-8a-readiness.ps1 -Attempt 38 -FinalizeFailedAttempt
.\scripts\validate-sprint-8a-readiness.ps1 -Attempt 39
.\scripts\run-sprint-8a-candidate-rehearsal.ps1 -Attempt 31 -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json"
```

Neither complete gate has passed for the corrected source. Preflight,
candidate freeze, SIT, and formal UAT remain closed until Readiness 39 and
Rehearsal 31 pass against the same source and environment identity.

Readiness requires and authenticates the six declared disposable database
URLs, proves their identities are pairwise distinct, records the normalized
Compose/tool/source contract, and issues the environment fingerprint consumed
by rehearsal. The complete runner serializes Cargo work that shares one target
directory and owns materialization/no-op, inventory/navigation, smoke,
Playwright, a non-acceptance live product diagnostic, Component
upgrade/rollback, deterministic failure containment, successor restoration,
automated scenario diagnostics, and final identity/health. The live diagnostic
runs before containment; containment destroys its disposable writes and proves
a fresh canonical successor. It cannot publish UAT evidence, and formal UAT is
not started by rehearsal.
Deployed acceptance smoke remains SIT-owned and manual scenarios remain
formal-UAT-owned.

## Preflight

- Status: Not Run.
- Receipt: `artifacts/sprint-8a-closeout/preflight-result.json`.
- Environment and reset authorization: must match passing readiness/rehearsal.
- Harness/inventory reconciliation: this record and every tracked runner, test,
  fixture, seed, manifest, bootstrap and deployment input match rehearsal.
- Bootstrap/no-op/teardown/rerun and post-rollback intended-release restoration
  commands: exact Sprint 8A commands frozen from passing rehearsal. Product
  data restoration or partial-materialization resume is not supported.
- Evidence paths and required-artifact audit: complete before freeze.
- Freeze output: `candidate.json` binds one immutable fingerprint; no validation
  result exists before it.

After the successor complete Readiness and Rehearsal pass, run only the tracked
top-level preflight command:

```powershell
.\scripts\run-sprint-8a-validation-preflight.ps1 -Attempt <n> -ReadinessReceipt "artifacts/sprint-8a-closeout/validation-readiness-result.json" -RehearsalReceipt "artifacts/sprint-8a-closeout/candidate-rehearsal-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -ExpectedBranch "codex/sprint-8a" -HandoffUrl "http://127.0.0.1:8088"
```

The runner, rather than operator-assembled helper calls, retains one terminal
result for each exact check identity below:

| Check identity | Depends on | Explicit retained check |
|---|---|---|
| `receipt-chain` | — | Parse and hash the exact passing Readiness and Rehearsal receipts; prove their source/environment and prerequisite binding. |
| `repository-scope` | — | Reconfirm repository instructions, worktree, branch, sprint scope, and one clean implementation commit. |
| `clean-source` | `receipt-chain`, `repository-scope` | Recompute commit, tree, dirty state, acceptance-inventory digest, and deployment-input digest and compare them exactly with Rehearsal. |
| `acceptance-traceability` | `receipt-chain`, `repository-scope` | Reconcile every roadmap exit and acceptance clause with the frozen 75-scenario browser inventory, smoke proof, and UAT-8A-01 through UAT-8A-08. |
| `environment-contract` | `receipt-chain`, `repository-scope` | Revalidate the secret-free fingerprint, tools, ports, Compose project/profile, handoff URL, reset authorization, and output-path forms. |
| `database-contract` | `environment-contract` | Revalidate all six named database variables, pairwise-distinct disposable identities, authenticated reachability, and migration-ledger tables. |
| `deployment-contract` | `clean-source`, `environment-contract` | Validate Compose/configuration, exact provenance label keys, bootstrap/no-op/teardown/recovery commands, active slot, and canonical restoration command without building product images. |
| `downstream-command-contract` | `repository-scope` | Parse the exact four SIT command sets and staged formal-UAT interface recorded below; reject a missing argument or obsolete runner contract. |
| `evidence-path-contract` | `repository-scope` | Reject unsupported absolute, traversal, and outside-root inputs; normalize contained repository-relative inputs; and prove no required path collides with immutable prior evidence. The only absolute-path exception is canonical consumption of the already-issued in-root R30 authorization. |
| `evidence-inventory` | `acceptance-traceability`, `deployment-contract`, `downstream-command-contract`, `evidence-path-contract` | Create the complete SIT/UAT/failure/manifest inventory before freeze and retain every mandatory output path. |

Before authenticating either prerequisite, write
`attempts/preflight-<n>.json` in `preparing` state with unverified claimed
source/environment identity, the ten declared check identities and their
dependencies, and `assertions_started = false`. Advance legally through
`executing`, optional failure `harvesting`, and `finalizing`; checkpoint every terminal check with exact
command, timestamps/duration, exit status, assertion-start marker,
classification, and hashed raw evidence. After a failure, continue every safe
independent check fail-late, mark true dependents `blocked` with the exact
failed prerequisite, and publish the terminal failed attempt without
`preflight-result.json` or `candidate.json`. Preserve and mark superseded
attempts; never reuse or merge their evidence.

After all ten results pass, the runner calls the lifecycle helper to publish
`preflight-result.json`, computes candidate identity from the exact source,
tracked deployment inputs, and live normalized Compose configuration digest,
and publishes `candidate.json`. The helper validates exact inventories, hashes,
prerequisite chain, source/environment identity, and candidate fingerprint; it
does not execute, checkpoint, or fabricate a missing check. The runner then
initializes the evidence manifest from all evidence that exists at freeze,
while its preflight inventory declares every future SIT/UAT and conditional-
failure path.

## SIT

SIT is authorized only by the passing preflight/candidate pair and enters
through the staged repository runner:

```powershell
.\scripts\run-sprint-8a-sit.ps1 -Stage Run -Attempt <n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/sit-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

Do not run the subordinate command sets independently. `Run` authenticates the
frozen identity, holds the evidence-root lock, executes all four lanes, retains
safe siblings fail-late, records exact dependency blocks, restores the
canonical topology, and publishes the aggregate only after a complete pass.

| Lane | Prepare receipt | Command/evidence | Assertions | Result | Duration |
|---|---|---|---|---|---|
| `static-and-boundaries` | `sit/attempts/static-and-boundaries-<n>.json` | exact static command set below | zero warnings and no forbidden edge | Not Run | |
| `rust-workspace` | `sit/attempts/rust-workspace-<n>.json` | exact database-bound Rust command set below | all pass on frozen candidate | Not Run | |
| `playwright` | `sit/attempts/playwright-<n>.json` | exact source/evidence-bound `validate-e2e.ps1` invocation below | complete inventory; UI/config/auth/outage/old-input cases pass | Not Run | |
| `deployed-acceptance-smoke` | `sit/attempts/deployed-acceptance-smoke-<n>.json` | exact materialization/smoke/recovery command set below | exact candidate; receipts complete; canonical topology restored | Not Run | |

Before authenticating preflight/candidate, write the phase start receipt at
`attempts/sit-<n>.json` in `preparing` state with the four declared lane
identities, unverified claimed candidate/environment identity, and
`assertions_started = false`. Each lane then owns a separate receipt and moves
through `preparing -> executing -> finalizing -> passed|failed|blocked`. Write
the lane start before expensive work, append logs and long-command heartbeats,
and retain every exact command, start/end/duration, exit status,
assertion-start marker, classification, raw evidence hash, and cleanup result.
Within each lane, continue safe independent checks fail-late; stop only true
dependents or unsafe destructive work and record the exact block reason.
Serialize the Playwright and deployed-smoke topology mutations even though each
owns its own source-exact preparation and evidence namespace.

After any failure, finish every safe lane, publish the terminal failed phase
attempt, and withhold `sit-result.json`; the coordinator owns classification
and invalidation. Preserve and mark superseded attempts rather than reusing a
number or merging evidence. Only after all four lanes pass and the canonical
topology is independently restored may the phase enter `finalizing`, publish
the aggregate SIT receipt through the helper, and immediately update and hash
the evidence manifest. A manifest/finalization failure remains retained and
blocks UAT even when the four raw lane results are intact.

The runner's static lane uses the following frozen command set and retains each
result:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config --quiet
. .\scripts\sprint-8a-acceptance-contract.ps1
Test-Sprint8AAcceptanceContract
.\scripts\check-web-crate-boundaries.ps1
.\scripts\verify-module-sdk-boundaries.ps1
.\scripts\verify-sprint-6e-boundaries.ps1
.\scripts\verify-markdown-links.ps1
```

After resetting the six exact preflight-approved database identities, the
runner executes the Rust lane serially in one Cargo target directory:

```powershell
cargo test --workspace --all-features --locked --offline
cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture
cargo test --locked --offline -p tessara-components-contract
cargo test --locked --offline -p tessara-dashboard-module
cargo test --locked --offline -p tessara-component-module
cargo test --locked --offline -p tessara-module-testkit
```

The runner invokes Playwright through the source/evidence-aware repository
wrapper, never a bare development-default command. Its lane prepares and proves its own
source-exact canonical topology, so it is not silently coupled to a sibling
lane receipt:

```powershell
.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/playwright-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp
.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/deployment.json"
.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8088" -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/deployment.json" -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -EvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/playwright.json" -FailureEvidenceDirectory "artifacts/sprint-8a-closeout/sit/playwright-<n>/failures"
```

The runner's deployed lane uses a new attempt namespace and the exact frozen
environment fingerprint. It retains first/no-op materialization, initial
inventory/provenance/product smoke, release transition, induced failure,
empty-successor recovery, and final canonical read-back:

```powershell
.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp
.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-inventory.json"
.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-deployment.json"
.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-smoke.json"
.\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/component-upgrade.json"
.\scripts\run-sprint-8a-failure-containment.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/failure-containment.json" -AuthorizeDisposableReset -SkipBuild
.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-inventory.json"
.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-deployment.json"
.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-smoke.json"
```

- SIT result receipt: `artifacts/sprint-8a-closeout/sit-result.json`.
- Canonical topology restoration: current Sprint 8A Component release active;
  all services healthy; canonical fresh seed; only v3 Component references;
  no old reader, open failure or partial topology.
- Receipt publication: the SIT runner calls `Publish-Sprint8ALifecycleReceipt`
  with phase `sit`, the exact preflight and candidate references,
  candidate/environment/normalized-Compose identity, all four terminal results,
  and hashed `canonical_topology_verified` restoration evidence. The helper
  rejects a missing/renamed lane, another candidate, stale evidence, or
  incomplete restoration before writing `sit-result.json`; it does not execute
  or checkpoint the lanes.
- Finalization-only recovery: only an exact `evidence-finalization` failure at
  `publication-failed`, with the immutable raw-terminal checkpoint proving all
  four lanes passed and identity unchanged, may run:

```powershell
.\scripts\run-sprint-8a-sit.ps1 -Stage Finalize -Attempt <same-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/sit-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

`Finalize` never reruns a raw SIT lane. It reauthenticates the exact source,
candidate, environment, normalized Compose digest, and checkpoint, restores
canonical topology, and retries aggregate/manifest publication. Any other
failure returns to the coordinator-selected invalidation boundary.

## UAT

### Scripted UAT

- Tracked staged command interface (implemented; not run):

```powershell
.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Start -Attempt <n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088"
# Acquire, hold, execute, and publish UAT-8A-01 through UAT-8A-08 one at a time.
.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Finalize -Attempt <same-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset
```

Readiness/preflight validates this interface without starting UAT by running
`.\scripts\run-sprint-8a-formal-uat.ps1 -SelfTest`.

- `Start` writes the attempt before fallible work, authenticates the exact
  preflight/candidate/SIT chain, recomputes clean source, candidate, and
  environment and normalized Compose identities, and runs fresh
  inventory/navigation plus Sprint 8A product smoke. It writes immutable
  `attempts/uat-<n>-manual-checkpoint.json` before manual execution.
- If scripted checks pass, manual scenarios are authoritative. If one or more
  scripted product/harness/environment assertions fail while identity remains
  trustworthy, the attempt enters `diagnostic-manual`: execute every safe
  independent manual scenario as non-authoritative diagnostic evidence and
  record true dependents blocked with exact reasons. Do not correct or restart
  until all eight scenarios have terminal receipts and Finalize consolidates
  the diagnostic pass. A source/candidate/environment identity failure blocks
  manual work. A `product-decision` blocks manual work and pauses for user
  direction rather than becoming a test failure.
- Manual receipts are exactly
  `uat/attempt-<n>/manual/uat-8a-01.json` through
  `uat/attempt-<n>/manual/uat-8a-08.json`. The attempt number is part of each
  typed receipt and its immutable path; a later attempt cannot reuse or
  supersede an earlier attempt's evidence. One scenario at a time must use the
  following held-lease sequence; the live lease spans all human actions and is
  consumed by receipt publication:

```powershell
. .\scripts\sprint-8a-lifecycle-chain.ps1
$scenario = "UAT-8A-<nn>"
$attempt = <uat-n>
$repositoryRoot = (Get-Location).Path
$evidenceRoot = "artifacts/sprint-8a-closeout"
$candidateFingerprint = "<candidate-fingerprint>"
$environmentFingerprint = "<environment-fingerprint>"
$preflightReference = [pscustomobject][ordered]@{ path = "artifacts/sprint-8a-closeout/preflight-result.json"; sha256 = "<sha256>" }
$sitReference = [pscustomobject][ordered]@{ path = "artifacts/sprint-8a-closeout/sit-result.json"; sha256 = "<sha256>" }
$startedAt = [DateTimeOffset]::UtcNow
$contract = Get-Sprint8AManualUatScenarioContract -Scenario $scenario
$plan = Get-Sprint8AManualUatEvidencePlan -Scenario $scenario -Attempt $attempt -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot
$executionLease = Open-Sprint8AManualUatScenarioLease -Scenario $scenario -Attempt $attempt -CandidateFingerprint $candidateFingerprint -EnvironmentFingerprint $environmentFingerprint -StartedAt $startedAt -Authoritative $true -Diagnostic $false -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot

$testerIdentity = [pscustomobject][ordered]@{
    tester_id = "<stable tester identity>"
    display_name = "<tester display name>"
    actor_bindings = @($contract.actor_bindings | ForEach-Object {
        [pscustomobject][ordered]@{ id = [string]$_.id; actor_id = "<exact account used for $([string]$_.id)>" }
    })
}
$preconditions = @(
    [pscustomobject][ordered]@{ id = "candidate-fingerprint"; state = "satisfied"; value = $candidateFingerprint; reference = $null },
    [pscustomobject][ordered]@{ id = "environment-fingerprint"; state = "satisfied"; value = $environmentFingerprint; reference = $null },
    [pscustomobject][ordered]@{ id = "preflight-receipt"; state = "satisfied"; value = $null; reference = $preflightReference },
    [pscustomobject][ordered]@{ id = "sit-result-receipt"; state = "satisfied"; value = $null; reference = $sitReference },
    [pscustomobject][ordered]@{ id = "evidence-root"; state = "satisfied"; value = $evidenceRoot; reference = $null },
    [pscustomobject][ordered]@{ id = "execution-start"; state = "satisfied"; value = $startedAt.ToString("o"); reference = $null }
)
$startingState = @($contract.required_starting_state | ForEach-Object {
    [pscustomobject][ordered]@{ id = [string]$_.id; observed_value = "<observed exact value>" }
})

# While $executionLease remains live, execute every manifest-bound step and
# write exactly one artifact at every $plan.evidence.path. For every
# authenticated-JSON requirement, including UAT-8A-05, UAT-8A-07, and
# UAT-8A-08, first write each assertion_raw_evidence path, then publish the
# authenticated wrapper/producer pair with the repository helper.
$evidence = [Collections.Generic.List[object]]::new()
foreach ($requirement in @($plan.evidence)) {
    if ([string]$requirement.kind -ceq "authenticated-json") {
        $assertionEvidence = @($requirement.assertion_raw_evidence | ForEach-Object {
            [pscustomobject][ordered]@{
                assertion_id = [string]$_.assertion_id
                raw_evidence = @([pscustomobject][ordered]@{
                    path = [string]$_.path
                    sha256 = Get-Sprint8AFileSha256 -Path (Join-Path $repositoryRoot ([string]$_.path))
                })
            }
        })
        $evidence.Add((Publish-Sprint8AManualUatStructuredEvidence -Scenario $scenario -Attempt $attempt -CandidateFingerprint $candidateFingerprint -EnvironmentFingerprint $environmentFingerprint -StartedAt $startedAt -RequirementId ([string]$requirement.requirement_id) -AssertionEvidence $assertionEvidence -ExecutionLease $executionLease -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot))
    } else {
        $evidence.Add([pscustomobject][ordered]@{
            step = [int]$requirement.step
            requirement_id = [string]$requirement.requirement_id
            kind = [string]$requirement.kind
            capture = $requirement.capture
            path = [string]$requirement.path
            sha256 = Get-Sprint8AFileSha256 -Path (Join-Path $repositoryRoot ([string]$requirement.path))
        })
    }
}
$actions = @($contract.steps | ForEach-Object { [pscustomobject][ordered]@{ step = [int]$_.step; action = [string]$_.action; expected_result = [string]$_.expected_result; actual_result = "<observed result>"; state = "passed" } })
$cleanupEvidence = @([pscustomobject][ordered]@{
    kind = "canonical-restoration"
    path = [string]$plan.cleanup.path
    sha256 = Get-Sprint8AFileSha256 -Path (Join-Path $repositoryRoot ([string]$plan.cleanup.path))
})
$endedAt = [DateTimeOffset]::UtcNow
Publish-Sprint8AManualUatReceipt -Scenario $scenario -Attempt $attempt -CandidateFingerprint $candidateFingerprint -EnvironmentFingerprint $environmentFingerprint -State passed -Authoritative $true -Diagnostic $false -AssertionsStarted $true -Role $contract.role -TesterIdentity $testerIdentity -Preconditions $preconditions -StartingState $startingState -Actions $actions -ExpectedResult "Every canonical step expectation in $scenario is satisfied." -ActualResult "<observed scenario result>" -Evidence @($evidence) -CleanupEvidence $cleanupEvidence -StartedAt $startedAt -EndedAt $endedAt -RepositoryRoot $repositoryRoot -EvidenceRoot $evidenceRoot -OutputPath "$evidenceRoot/uat/attempt-$attempt/manual/$($scenario.ToLowerInvariant()).json" -ExecutionLease $executionLease
```

Every manifest requirement has exact cardinality one. Ordinary JSON evidence
uses the exact ordered fields `schema_version`, `sprint`, `phase`, `scenario`,
`attempt`, `evidence_id`, `candidate_fingerprint`, `environment_fingerprint`,
`captured_at`, and its typed array payload: `observations` for
`operator-record`, `entries` for `browser-console`, or `exchanges` for
`http-transcript`. Screenshots are real PNGs with `IHDR`; browser traces are
ZIPs containing non-empty `trace.trace` and `trace.network`. The one cleanup
JSON uses the exact ordered fields `schema_version`, `sprint`, `phase`,
`scenario`, `attempt`, `result`, `candidate_fingerprint`,
`environment_fingerprint`, `restored_at`, and `observations`, with phase
`uat-manual-canonical-restoration` and result
`canonical_topology_verified`. All capture/restoration timestamps must be at or
after `$startedAt`, and every path must be the exact path returned by `$plan`.

For `diagnostic-manual`, both lease and receipt use
`-Authoritative $false -Diagnostic $true`. A failed receipt supplies an
allowed `-Classification` and `-FailureMessage`; a blocked receipt uses
`-AssertionsStarted $false`, an exact `-BlockedReason`, the full canonical
action inventory with dependent actions marked `blocked`, and may use an empty
scenario `-Evidence @()` without fabricating observations. Cleanup evidence
remains mandatory. After one
authoritative manual defect, every later safe scenario is diagnostic. A manual
`product-decision` pauses later scenarios. If execution is interrupted, reopen
only the exact same lease binding and start time with `-Resume`; do not create a
replacement scenario identity. Finalize rejects every open or interrupted
start lease. Resume writes an authenticated marker bound to the original and
current process IDs. A resumed execution can retain a terminal failed or
blocked receipt, but it cannot establish an authoritative pass; a passing
scenario must be observed without a process-lineage break.
- `Publish-Sprint8AManualUatReceipt` validates the manifest/document hashes,
  exact acceptance-criterion and semantic-predicate bindings, tester/actor
  identity, typed preconditions and starting state, ordered
  step/action/expected-result identities, observed results, terminal state,
  assertion boundary, candidate/environment/checkpoint binding, every exact
  evidence identity/content/digest, canonical-restoration evidence, and the
  matching execution-lease start/completion pair. Before publishing either
  terminal file it writes
  an immutable prepared checkpoint containing the exact receipt and completion
  bytes; retry repairs only that bound publication transaction and never
  reruns the human scenario. Finalization can pass only with all eight exact
  authoritative, non-diagnostic passing receipts. An explicit
  `-ManualReceiptDirectory`, when supplied to the staged runner, must resolve
  to that same attempt-scoped canonical directory.
- `Finalize` requires the unchanged Start checkpoint and all eight exact manual
  receipts, revalidates source/environment, frozen endpoints, and the
  prerequisite chain, and requires explicit disposable-reset authorization. It
  then performs source-exact from-empty materialization plus semantic no-op,
  inventory/navigation audit, and Sprint 8A smoke with the frozen gateway,
  materialization-control, and Supervisor endpoints. The runner merges the
  prepared UAT result commitment and every retained artifact into the existing
  preflight/SIT evidence manifest. After scripted, manual, and restoration
  checks all pass, it writes and hashes a durable
  `finalization-completion-checkpoint.json`, then commits and publishes the
  canonical UAT JSON/sidecar pair. It does not consume or relabel rehearsal's
  `uat-sprint-8a.ps1` diagnostics.
- If publication alone fails after that completion checkpoint, rerun the same
  `Finalize` command. It reauthenticates the completion and retry checkpoints,
  source, candidate, environment, normalized Compose digest, endpoints,
  prerequisites, and passing SIT receipt, then executes only the publication
  tail; it does not rerun manual validation or destructive restoration.
  Canonical pair recovery recognizes exactly four states: `absent` publishes
  the committed JSON and sidecar; `json-only` authenticates JSON and writes only
  the committed sidecar; `sidecar-only` authenticates the sidecar and writes
  only the committed JSON; `complete` authenticates both without rewriting
  them. Mismatched or malformed partial bytes remain a retained
  `evidence-finalization` failure and are never blessed.
- Start rule: only after authoritative SIT passes on the same fingerprint.
- Result: Not Run.
- Evidence: `artifacts/sprint-8a-closeout/uat/attempt-<n>/`,
  `artifacts/sprint-8a-closeout/attempts/uat-<n>.json`, the immutable
  `attempts/uat-<n>-manual-checkpoint.json`, eight manual receipts and eight
  execution-lease pairs, finalization completion/result-commit evidence, any
  retained publication-retry evidence, `uat-result.json`, and
  `evidence-manifest.json`.

### Manual UAT

The machine-readable scenario contract is normative; this table is its exact
human-readable summary.

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| UAT-8A-01 unchanged Component experience | Component manager/reader; fresh canonical seed | Browse, create/edit/publish/version, exercise lifecycle and every kind; capture light/dark at 1280/768/390, keyboard, 200% zoom, no-JS SSR, hydration and console | Same accepted product behavior and canonical vocabulary; module owns documents/assets; responsive/theme/accessibility evidence complete; disposal succeeds | Not Run | `uat/attempt-<n>/manual/uat-8a-01.json` plus exact operator records, named screenshots, browser traces, and browser-console record |
| UAT-8A-02 from-empty seed and new references | Operator and Dashboard reader; explicitly authorized empty Sprint 8A databases | Run source-exact materialization; inspect exact owner read-back; open the seeded Dashboard and inspect resolved placements; inspect module inventory/navigation; rerun unchanged | Exactly 7 Component shells, 8 versions and 7 placements; predecessor/successor/action fixtures correct; only selected Component instance/v3; Dashboard appears once through its real Release/Instance; Core exposes exactly Forms, Workflows, Responses, Datasets, and Migration transitions; second run no-op | Not Run | `uat/attempt-<n>/manual/uat-8a-02.json` plus exact operator records and HTTP transcripts |
| UAT-8A-03 configuration and diagnostics | Global Module Management manager/reader plus Components manager | Set label and valid timeout; try blank/long label, 0/31 timeout, unknown field/version; inspect navigation/admin/product headings/diagnostics | Manifest defaults are Components/5; label/range/authority correct; binding/provider/contract/compatibility/health/result diagnostics are sanitized | Not Run | `uat/attempt-<n>/manual/uat-8a-03.json` plus exact browser trace, HTTP transcripts, and operator records |
| UAT-8A-04 Dataset contract, joint scope and outage | Operator, Component manager, dual-scope administrator, and restricted actors; Dataset and mixed Dashboard fixtures | Author/preview/execute; prove shared-node Dashboard render; inspect/attempt a disjoint placement while authorized on both scopes; compare known/random restriction; stop/timeout/recover Dataset provider; retry preserved editor | Typed reference only; shared-node path works; disjoint title/metadata/identity/data stay undisclosed and render is denied; known/random outcomes match; outage is read-only with no pending write; recovery succeeds | Not Run | `uat/attempt-<n>/manual/uat-8a-04.json` plus exact browser traces and HTTP transcript |
| UAT-8A-05 Dashboard lifecycle and outages | Operator, Component manager, Dashboard manager, and Dashboard reader; exact predecessor-bound action placements | Refresh exact findings; Defer then Upgrade; independently Replace and Remove; verify blocked nondisclosure; stop/recover Component | Three lifecycle findings and all four actions are exact; blocked placement absent; outage covers exact five authorized remaining placements; recovery has zero findings | Not Run | `uat/attempt-<n>/manual/uat-8a-05.json` plus exact browser traces and authenticated semantic receipt |
| UAT-8A-06 isolation and unsupported old inputs | Operator plus authorized/restricted actors | Inspect images/graphs/credentials/Core absence; submit V1/V2, old owner/type, retired `missing_policy`, flat Dataset-major, and old payload requests | No forbidden source/storage edge; old inputs fail exact normal contract with nondisclosure; no adapter/ledger/fallback | Not Run | `uat/attempt-<n>/manual/uat-8a-06.json` plus exact operator record and HTTP transcripts |
| UAT-8A-07 failed materialization rerun | Operator; disposable authorized topology; induced owner-bootstrap failure | Run failing attempt, retain hashed raw apply response and service-log evidence, verify exact teardown, remove the fault and run a complete successor, then repeat bootstrap unchanged | Failure retained; partial project/volumes absent; successor begins empty, reaches canonical health, and reports an exact semantic no-op on repeat | Not Run | `uat/attempt-<n>/manual/uat-8a-07.json` plus exact operator records and authenticated failure-containment receipt |
| UAT-8A-08 Component-only upgrade/rollback | Operator/reviewer; healthy `1.0.0` current release plus source-built `0.9.0` fixture | Establish `0.9.0`, upgrade to `1.0.0`, roll back, restore `1.0.0` through Supervisor/Compose; compare snapshots | Exact one-owner health-gated deltas; distinct release/binary identities; Component state retained; unrelated identities/restarts unchanged; final topology healthy | Not Run | `uat/attempt-<n>/manual/uat-8a-08.json` plus exact operator records and authenticated upgrade/rollback receipt |

- UAT result receipt: `artifacts/sprint-8a-closeout/uat-result.json`.
- Final topology restoration: intended Sprint 8A route/slot and current
  Component release; healthy Core/Component/Dashboard/Dataset path; canonical
  fresh seed; only v3 live references; no open defect.

## Changed Integration Contracts

| Boundary | Implemented change (formal proof pending) | Required negative proof |
|---|---|---|
| Component public contract | exact-current `3.0.0`, Module Instance owner and `tessara.components.component_version` | old/mixed/wrong-instance/cross-installation/malformed/unauthorized inputs fail closed |
| Component render response | Components-owned exact Table/visual DTOs and render kind; non-nil exact persistent identity; explicit both-nil preview identity | unknown/cross-kind fields, wrong branch/kind/schema, Dataset identity, mismatched/nil persistent identity, and non-nil/partial-nil preview fail closed |
| Component product API | canonical module-owned typed-reference bodies on stable same-origin product paths | old Core payloads rejected; no translation reader/facade |
| Signed module service requests | exact transmitted body bytes verified before typed decode across JSON-bearing Component, Core authorization-exchange and Core Dataset routes | changed whitespace/field bytes without resigning, wrong media type, wrong correlation/grant/service identity, or stale digest fail before product work |
| Core Dataset compatibility | typed Dataset-major-line reference and versioned catalog/schema/distinct/execution/compatibility operations | no private DTO/SQL/credential; wrong audience/scope/version and outage do not disclose or fall back |
| Module configuration and diagnostics | Manifest schema v1 defaults `Components`/`5`; real image command paths; selected Dataset binding compatibility/health observation | unknown schema/field, invalid label, 0/31 timeout and wrong authority rejected; no raw references/secrets; command paths exist in image |
| Dashboard dependency | v3 provider binding; exact predecessor/successor and action placements; authorized Dashboard-scope intersection; exact ComponentVersion resource assertion; one common governing node; structured semantic evidence | no old owner/type or blocked/disjoint-scope metadata/title/data disclosure; disjoint render denied; Defer/Upgrade/Replace/Remove and Component outage/recovery cannot pass from broad labels |
| Active first-party acceptance inputs | exact v3 Component and nested typed Dataset-reference fixtures; 75 exact Playwright identities | retired normalization aliases, flat Dataset-major fields, copied counts, or unmanifested scenario changes rejected |
| Fresh bootstrap | destructive owner-ordered seed plus generic lockfile-owned dependency validation and exact 7-shell/8-version/7-placement idempotent seed | ambiguous or mismatched target, altered payload/request/apply/owner/audience, expiry, and replay rejected; no cross-owner or pre-validation writes; exact predecessor/successor/action identities; second run no-op; failed partial topology never reused |
| Component release transition | source-built compatible `0.9.0` and candidate `1.0.0`; exact Supervisor/Compose one-owner deltas | candidate relabel rejected; release/binary identities distinct; unrelated owners absent from plan and unchanged in snapshots |
| Core ownership | generic platform integration only | Component product storage/code/routes/adapter/readers absent |

## Source Provenance, Schema Baseline, And Evidence Rules

- Freeze inputs include commit/tree/dirty state and every tracked product,
  test, harness, schema migration, fixture, seed, manifest, bootstrap,
  deployment, configuration, contract and acceptance identity.
- Candidate provenance matches image labels, release/instance read-back,
  contract/manifest/asset/schema/seed digests and deployed diagnostics.
- The baseline is empty, owner-specific schema materialization. No populated
  Core Component baseline, product-data mapping or legacy reference ledger is
  a candidate input.
- Seed assertions use semantic identity sets. Any contractual exact count comes
  from one shared acceptance source of truth.
- Receipts record commands, timestamps, duration, exit status, assertions, raw
  paths/hashes, prerequisites, candidate/environment identity, classification,
  invalidation, teardown and restoration.
- Secrets, credentials, private keys and token values never enter receipts.

## Failure And Invalidation Chronology

| Cause | Minimum invalidation |
|---|---|
| Candidate fingerprint changed | All SIT and UAT |
| Acceptance inventory or tracked harness changed | All SIT and UAT |
| Shared environment materially changed | Affected lane and downstream phases |
| Lane-local setup failure before assertions | Failed lane after prerequisites reconfirm |
| Evidence finalization failure with immutable complete raw results | Finalization only |
| Test assertion failure with unchanged candidate | Complete failed lane; coordinator assesses upstream relevance |
| UAT scenario setup failure before product actions | Affected isolated scenario set when prerequisites reconfirm |
| Product defect corrected | Refreeze after complete readiness/rehearsal, then all SIT and UAT |
| Missing acceptance assertion discovered | Update inventory/candidate, then all SIT and UAT |
| Candidate-affecting correction after a mutable gate pass | Supersede that pass; run new complete readiness and rehearsal on one corrected source/environment identity |

Runtime chronology:

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction/narrow proof | Invalidation scope | Authoritative replacement |
|---|---|---|---|---|---|---|---|
| 2026-08-06 22:00 EDT | Readiness 34 / `compose-database-contract` | Yes | `c920b8a3` | `environment` | Fresh six-binding runner environment required; retained in the single Readiness 34 batch | Readiness 34 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:00 EDT | Readiness 34 / `environment-contract` | No; blocked by `compose-database-contract` | `c920b8a3` | `environment` | Exact dependency reason retained in the same batch | Readiness 34 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:00 EDT | Readiness 34 / `reset-dry-run` | Yes | `c920b8a3` | `harness` | Shared optional-property projection and executable self-test; retained in the same batch | Readiness 34 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:19 EDT | Readiness 35 / terminal evidence publication | Yes; all 15 checks passed | `49c0ae73` | `evidence-finalization` | Shared empty-result classification projection for Readiness and Rehearsal; one R35 batch | Readiness 35 and all downstream phases | New complete Readiness and Rehearsal |
| 2026-08-06 22:34 EDT | Readiness 36 / complete gate | Yes; all 15 checks passed | `3f7e32cb` | N/A — passed, later superseded | Complete receipt retained for prior clean source and environment `2e235b07...` | Readiness 36 and all downstream phases after candidate-affecting audit findings | New complete Readiness and Rehearsal |
| 2026-08-06 22:36 EDT | Rehearsal 29 / `dashboard-source-boundaries`, `markdown-links` | Yes; both child scripts passed | `3f7e32cb` | `harness` | Two failed receipts retained; one stale parent `$LASTEXITCODE` root defect identified from raw logs | Rehearsal 29 and all downstream phases | Consolidated implementation correction, then new complete Readiness and Rehearsal |
| 2026-08-06 22:53 EDT | Rehearsal 29 / `workspace-tests` | Yes | `3f7e32cb` | `harness` (test fixture) | Failed receipt and exact 400/503 assertion retained; fixture omitted required `amount` while product correctly rejected invalid input | Rehearsal 29 and all downstream phases | Consolidated implementation correction, then new complete Readiness and Rehearsal |
| 2026-08-06 after 22:53 EDT | Rehearsal 29 / `optimized-resource-reference-timing` and remaining graph | Optimized lane interrupted; 20 checks not started | `3f7e32cb` | N/A — user-directed phase exit | One interrupted receipt, 20 unexecuted checks, and zero recorded blocked receipts retained without inferring results | Rehearsal 29 and all downstream phases | New complete Readiness and Rehearsal; Attempt 29 may not resume |
| 2026-08-06 after phase exit | Implementation audit / Component render boundary | Source audit started | mutable successor | `product` | Components contract owns exact kind-specific responses; provider/routes return them; Dashboard validates them; renderer has no Dataset-contract edge or copied DTOs; persistent and preview identities are disjoint exact contracts | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Implementation audit / signed service body | Source audit started | mutable successor | `product` | Verify exact raw bytes before typed decode across Component, authorization exchange and Dataset routes; retain JSON/body limits; add byte-tamper proof | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Implementation audit / Dashboard joint scope and resource assertion | Source audit started | mutable successor | `product` | Forward authorized Dashboard-scope intersection; bind exact ComponentVersion assertion; require one governing node; redact disjoint projection; add Rust/browser/UAT proof | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Implementation audit / active legacy acceptance facade | Source audit started | mutable successor | `harness` | Remove normalization aliases and flat Dataset-major fixtures; enforce exact v3 typed inputs; advance manifest to 75 exact scenarios | Readiness 36, Rehearsal 29, and all downstream phases | Correct consolidated batch, then new complete Readiness and Rehearsal |
| 2026-08-07 | Intermediate implementation diagnostics | Yes; narrow and workspace assertions ran | mutable successor identities | N/A — later superseded | Focused checks and one 684.9-second all-feature workspace pass completed before final preview/joint-scope edits | No gate or downstream phase authorized | Rerun final implementation checks, then complete Readiness and Rehearsal |
| 2026-08-07 | Final implementation-entry verification | Yes; complete workspace and static implementation suites ran | final corrected mutable product source | N/A — implementation checks passed | Six isolated databases reset; 619.6-second all-feature workspace suite, warnings-denied clippy, check, formatting, focused integrations, boundary/acceptance/link/runner audits, 75/75 inventory, TypeScript, and Playwright discovery passed | No formal gate or downstream phase authorized | Commit cleanly, then run complete Readiness and Rehearsal |
| 2026-08-07 01:38 EDT | Readiness 37 / complete gate | Yes; all 15 checks passed | `84964c7b` / `4f39cb71`, environment `96c2a32e...` | N/A — passed, later superseded | Complete receipt retained with SHA-256 `a4ab361a...` | Readiness 37 and all downstream phases after R30 correction | Successor complete Readiness consuming R30 authorization, then complete Rehearsal |
| 2026-08-07 01:39–02:52 EDT | Rehearsal 30 / four raw failed lanes | Yes in fact; R30 omitted the required per-lane marker | `84964c7b` | Raw: two `environment`, one `product`, one `harness`; diagnosed: three `harness` roots | Action-object projection, source-identity shape, and process-environment restoration corrected as one batch; raw receipts unchanged | Rehearsal 30, Readiness 37, and all downstream phases | Successor complete Readiness and complete Rehearsal |
| 2026-08-07 01:39–02:52 EDT | Rehearsal 30 / ten blocked lanes | No in fact; R30 omitted the required per-lane marker | `84964c7b` | N/A — exact dependency blocks | Ten exact blocked reasons retained; assertion counts corrected to exclude blocked terminal receipts | Rehearsal 30 and all downstream phases | Successor complete Readiness and complete Rehearsal |
| 2026-08-07 02:52 EDT | Rehearsal 30 / harvest and correction authorization | No product assertions | `84964c7b` | Four raw symptoms; three diagnosed `harness` roots | One immutable harvest, one consolidated batch, and one unconsumed one-use successor-Readiness authorization retained | Correction is candidate-affecting; no freeze | Correct batch, consume authorization in successor Readiness, then complete Rehearsal |
| 2026-08-07 after user-directed testing exit | Implementation audit / Dashboard executable identity | Source audit started | mutable successor | `product` | Current runtime/migration commands now name `/usr/local/bin/dashboard-module`, the executable installed by the image; exact Manifest/Dockerfile assertions and canonical catalog digest advanced together | Readiness 37, Rehearsal 30, and all downstream phases | Commit the consolidated implementation correction, then successor complete Readiness and Rehearsal |
| 2026-08-07 after user-directed testing exit | Implementation audit / Component exact-current configuration | Source audit started | mutable successor | `product` | Removed retired object-shaped `visible_columns` and filter `field` readers from validation, provider and browser; added rejection/source guards and advanced asset/Manifest/catalog digests together | Readiness 37, Rehearsal 30, and all downstream phases | Commit the consolidated implementation correction, then successor complete Readiness and Rehearsal |
| 2026-08-07 06:30 EDT | Readiness 38 / `compose-database-contract` | Yes | `91c9936b` / `564493fd`; environment unverified | `environment` | Missing same-process `TEST_API_DATABASE_URL`; raw log and complete failed attempt retained | Readiness 38 and all downstream phases | Exit testing; correct the consolidated entry batch; finalize R38; then Readiness 39 and Rehearsal 31 |
| 2026-08-07 06:30 EDT | Readiness 38 / `runner-self-tests` | Yes | `91c9936b` / `564493fd` | `harness` | Failure-containment self-test used invalid overwrite after deliberate corruption; raw log retained. The correction proves overwrite rejection, removes only the exact corrupt no-journal pair inside the validated temporary self-test root, and republishes create-once; real corrupt retained evidence remains preserved. | Readiness 38 and all downstream phases | Same consolidated correction and full-gate boundary |
| 2026-08-07 06:30 EDT | Readiness 38 / `environment-contract` | No; blocked by `compose-database-contract` | `91c9936b` / `564493fd` | N/A — exact dependency block | Exact blocked reason retained; not counted as an independent defect | Readiness 38 and all downstream phases | Same consolidated correction and full-gate boundary |
| 2026-08-07 after Readiness 38 testing exit | Plan-to-source implementation audit | Focused source review only | mutable correction successor | `product`, `harness`, and evidence/process enforcement | Added failed-Readiness correction lineage, truthful Rehearsal checkpoints, exact Component missing-policy contract, and canonical UAT scenario/evidence/AC-18/AC-19 enforcement | R38, all mutable receipts, and every downstream phase | Focused implementation verification and clean commit; no formal gate in this phase |
| 2026-08-07 after Readiness 38 testing exit | Focused lifecycle/formal-UAT/diagnostic-UAT/preflight self-tests / initial diagnostic pass | Yes in lifecycle/formal checks; dependent self-tests also started | mutable correction successor | `harness` | Raw self-test output retained in the implementation session: lifecycle JSON evidence incorrectly required structured-authority parameters and formal-UAT partial-pair repair removed an authenticated JSON-only state; diagnostic-UAT and preflight then failed through the lifecycle dependency. Two root defects and two dependent outcomes were corrected together. | No formal gate result; no candidate or downstream phase authorized | Rerun the complete four-self-test batch only after the consolidated correction |
| 2026-08-07 after Readiness 38 testing exit | Focused lifecycle/formal-UAT/diagnostic-UAT/preflight self-tests / three-root fail-late pass | Yes; all four independent self-tests ran; zero blocked | mutable correction successor | `harness` | Four raw failures exposed three roots before correction: temporary structured-evidence cleanup left stale manifest commitments (lifecycle and formal UAT), acceptance still required retired `{01..08}` lease patterns (diagnostic UAT), and preflight's exact inventory selector omitted `manual-leases`. The three roots were one consolidated narrow correction batch. | No formal gate result; no candidate or downstream phase authorized | Rerun all four self-tests after the shared acceptance correction |
| 2026-08-07 after Readiness 38 testing exit | Focused four-self-test successor diagnostic pass | Lifecycle and formal UAT passed; diagnostic UAT and preflight started and failed; zero blocked | mutable correction successor | `harness` | Both failures had one shared root: preflight extracted `$declaredChecks` with a broad source span, so new readiness self-test fixtures appeared as duplicate declarations. Exact AST assignment extraction replaces the broad scan. | No formal gate result; no candidate or downstream phase authorized | Rerun the same complete four-self-test batch; only a clean result is implementation-entry proof |
| 2026-08-07 after Readiness 38 testing exit | Focused four-self-test post-AST diagnostic pass | Lifecycle, formal UAT, and preflight passed; diagnostic UAT started and failed; zero blocked | mutable correction successor | `harness` | Acceptance correctly found that declared-independent Rehearsal lane `validation-readiness-prerequisite` consumed `validation_state` and immutable-readiness context produced only by sibling `attempt-state-prerequisite`. The readiness/source/environment lane must authenticate its supplied receipt independently; the state lane alone owns current-state binding. | No formal gate result; no candidate or downstream phase authorized | Correct the Rehearsal lane coupling, then rerun the same complete four-self-test batch |
| 2026-08-07 after Readiness 38 testing exit | Focused four-self-test zero-evidence-block diagnostic pass | All four reached the same new lifecycle publisher proof; zero blocked | mutable correction successor | `harness` | The public manual-receipt publisher accepted an empty scenario-evidence array, but its private evidence resolver still rejected the empty `References` binding. The end-to-end dependent-block self-test exposed the remaining representation gap; the inner resolver now explicitly accepts an empty scenario array while cleanup evidence stays mandatory. | No formal gate result; no candidate or downstream phase authorized | Rerun the same complete four-self-test batch |
| 2026-08-07 after Readiness 38 testing exit | Focused post-coupling four-self-test diagnostic pass | Lifecycle, formal UAT, and preflight passed; diagnostic UAT started and failed; zero blocked | mutable correction successor | `harness` | The acceptance ordering guard selected the lane-marker literal inside the new isolation self-test instead of the live `validation-readiness-prerequisite` invocation. Selecting the last exact marker binds the guard to the live lane; a focused diagnostic-UAT repro then passed. | No formal gate result; no candidate or downstream phase authorized | Rerun all four self-tests against the same stable bytes |
| 2026-08-07 after Readiness 38 testing exit | Focused correction-lineage/evidence compatibility review | Source assertions and disposable self-tests only | mutable correction successor | `harness` and evidence/process enforcement | One sub-batch corrected four related findings: live callers now supply `receipt`, not `path`, to `ExpectedCurrentReadiness`; passing Rehearsal attempt/result use one exact Readiness-prerequisite representation; `produced_evidence: null` no longer becomes a bogus raw-evidence entry during failed-Readiness finalization; and raw-evidence resolution rejects outside-root absolute paths. | No formal gate result; no candidate or downstream phase authorized | Parser and runner-local/preflight self-tests, followed by the complete four-self-test batch |
| 2026-08-07 after Readiness 38 testing exit | Final focused lifecycle/formal-UAT/diagnostic-UAT/preflight self-test batch | Yes; all four passed; zero failed and zero blocked | final mutable correction source | N/A — focused implementation proof passed | Lifecycle schema/identity, formal-UAT prerequisite/staging/receipt, automated-UAT diagnostic identity/freshness, and validation-preflight graph/receipt/classification/no-execution self-tests all passed against the same stable working tree. This includes authenticated structured evidence, exact inventory, true zero-evidence dependent blocks, and Rehearsal lane isolation. | No formal gate result; no candidate or downstream phase authorized | Complete implementation review and clean commit, then new complete Readiness and Candidate Rehearsal |
| 2026-08-07 after Readiness 38 testing exit | Pre-audit consolidated testing-entry verification | Focused assertions only; no lifecycle runner attempt | mutable correction source | N/A — implementation checks passed | All 12 fail-late harness checks passed; formatting, JavaScript syntax, 33 Component library tests, warnings-denied Component clippy, exact five-transition identity, three exact Dashboard/inventory/navigation API tests, and 75-scenario Playwright discovery also passed. No browser scenario or formal gate ran. | No formal receipt, candidate, or downstream phase authorized | Independent final diff audit before commit |
| 2026-08-07 final diff audit | Attempt reservation, UAT-8A-07 raw evidence, and documentation reconciliation | Read-only source audit | mutable correction source | `harness` and evidence/process enforcement | Four actionable findings were retained as one final batch: an out-of-sequence Readiness probe could occupy the authorized successor namespace; UAT-8A-07 could claim raw failure retention through a generic operator record; the Rehearsal section still called R30 authorization unconsumed; and the manual-UAT summary differed from the exact manifest. The batch reserves the attempt under lock before all five namespace targets, requires authenticated hashed apply/log assertions, and reconciles the documentation. | No formal receipt, candidate, or downstream phase authorized | Rerun the complete affected focused cone before commit |
| 2026-08-07 final correction verification | Readiness/lifecycle/formal-UAT/diagnostic-UAT/preflight/acceptance/harvest/Rehearsal self-tests | Focused assertions only; no lifecycle runner attempt | final mutable correction source | N/A — affected implementation checks passed | PowerShell/JSON parsing, Readiness reservation/finalization, lifecycle schema/identity, formal UAT, diagnostic UAT, preflight, acceptance, harvest adversarial, and Candidate Rehearsal self-tests passed. UAT now has 46 globally unique exact-one requirements; the new failure-containment receipt rejects either missing raw assertion. | No formal receipt, candidate, or downstream phase authorized | Clean diff/evidence audit and one correction commit; then R38 finalization at the documented boundary |

Classifications are exactly `preflight/setup`, `product`, `harness`,
`environment`, `flaky`, `evidence-finalization`, or `product-decision`.
Passed or user-interrupted rows have no defect classification. Product
decisions pause for user direction.

## Post-SIT Defect Convergence (Conditional)

- On the first candidate-invalidating UAT failure, retain the failure,
  invalidate the candidate, forbid passing `uat-result.json`, and finish only
  safe independent scenarios as non-authoritative diagnostics.
- Write `uat-defect-harvest.json`, consolidate `defect-batch.json`, restore the
  canonical fresh topology, and create
  `correction-impact-assessment.json`.
- Identity, authorization, protocol, schema, seed/bootstrap, shared fixture/
  environment and cross-module changes default to a broad cone.
- Focused repair attempts diagnose mutable corrections only. They never
  authorize a candidate, SIT, UAT or closeout.
- After convergence, record restoration and final-certification entry, then
  rerun complete readiness, rehearsal, preflight, SIT and UAT for a successor.

## Evidence Integrity

- Required files complete: Not Run.
- Structured artifacts parse: Not Run.
- Markdown links pass: Not Run.
- Authoritative/superseded attempts distinguished: Not Run.
- Manifest file count: Not Run.
- Manifest SHA-256: Not Run.

Superseded failures remain retained and explicitly excluded rather than
overwritten. Every phase result names and hashes its prerequisites. One
immutable fingerprint covers all authoritative SIT and UAT evidence.

## Closeout Authorization

- Status: Not Authorized.
- Authorization receipt:
  `artifacts/sprint-8a-closeout/closeout-authorization.json` (not created).
- Authorized candidate/fingerprint: None.
- SIT passed: No.
- UAT passed: No.
- Acceptance mapping complete: Planned, not executed.
- Invalidation decisions satisfied: No — terminal failed Readiness 38 requires
  the clean correction commit and R38 finalization; complete Readiness 39 and
  Candidate Rehearsal 31 then remain required.
- Unresolved product decisions: None.
- Intended active route/slot: source-exact Sprint 8A gateway with current
  Component release and canonical fresh seed.
- Application health: Not proven after terminal failed R38;
  cleanup/restoration was not proven.
- Evidence source commit: Not frozen.
- Documentation commit: Not created.
- Authorization timestamp: None.

Closeout is forbidden until the validation coordinator verifies the complete
hashed chain and writes authorization. This record implies no authoritative
frozen-candidate deployment, SIT, formal-UAT, or acceptance result.

# Sprint 6A-UI Test Change Log

Status: open. No existing test, accepted fixture, manifest identity, smoke/UAT assertion, or migration proof has been changed as of the dynamic-navigation scope amendment.

The closed Sprint 6A browser baseline is `end2end/acceptance-manifest.json`, schema version 2, SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`, containing 60 exact identities across seven files. It remains historical proof of the prior contract. Sprint 6A-UI intentionally supersedes only its hard-coded group/band/Administration expectations; every unrelated identity and assertion remains durable parity proof.

Tests are durable proof of supported behavior. A failing test is not authorization to rewrite it. Production code is corrected unless an explicit product decision changes the requirement.

New or changed Sprint 6A-UI proof is limited to the versioned navigation policy and migration, shell group composition, direct Core Admin destinations and exact `/administration` removal, Module Management, and capability-provenance presentation. All unrelated accepted tests remain unchanged parity evidence; no application-wide visual baseline is introduced.

## Approved Behavior Changes

The product owner has approved these reasons for future proof replacement:

- replace hard-coded `Main`/`Admin`, Core-anchor, and reorder-band behavior with ordered configured groups and complete destination placements;
- allow every optional destination to move between groups, reorder, and show/hide;
- allow effective global `modules:manage_navigation` to create, rename, reorder, and delete empty custom groups;
- require stable non-deletable Main/Admin identities; protect Home and Organization in Main under their approved visibility rules; protect User Management, Roles & Access, Node Types, and Module Management in Admin; and make Operations optional;
- remove the Administration navigation destination, page, and exact `/administration` route without redirect while preserving direct `/administration/*` routes; and
- version the management and shell wires and migrate the contribution-only v1 policy without weakening authorization, revision, audit, SSR, no-JavaScript, or fail-closed behavior.

These decisions authorize only the obsolete expectation to change. They do not pre-approve a particular test edit, name, fixture, count, timeout, selector, or weaker assertion. Each actual edit still requires an exact row below before it lands.

## Change Rules

Every edit to an existing test, fixture, manifest, timeout, selector, screenshot, smoke check, or UAT assertion requires a row below before the change is accepted. The row must:

1. identify the exact file and test/evidence identity;
2. cite a recorded product requirement or approved decision;
3. explain why the old assertion or selector is no longer correct or stable;
4. state whether observable behavior changes;
5. identify equal-or-stronger positive and negative replacement proof; and
6. record reviewer approval and the commit that lands the change.

The following are never routine maintenance:

- deleting, skipping, filtering, renaming, or weakening a test;
- increasing timeouts, tolerances, workers, or retries to obtain a pass;
- changing accepted fixture bytes or manifest identities;
- loosening role, scope, ownership, redaction, SSR, hydration, direct-load, no-JavaScript, or console assertions;
- bulk-regenerating visual baselines; or
- changing selectors from user-visible semantics to incidental CSS/DOM structure.

Selector-only updates still require a row. They are acceptable only when the new selector is at least as semantic and the asserted behavior remains unchanged.

New tests do not need a change row unless they replace or alter existing proof, but their purpose and acceptance mapping must appear in the sprint plan or issue matrix. Each visual baseline is reviewed individually and records its stable state, viewport, theme, data fixture, and reason for inclusion.

The first implementation slice must enter planned rows for every affected identity in `app.spec.ts`, `modules.spec.ts`, `permissions.spec.ts`, API/web navigation tests, migration/rollback evidence, `smoke.ps1`, and `uat-sprint.ps1` before changing those files. Rows may share one approved requirement, but each exact proof identity remains individually traceable.

The pre-implementation impact inventory is intentionally finite:

- browser shell composition: `root route renders assigned work in the native shell` and `authenticated primary routes render in the native shell`;
- Module Management: `global read exposes the fixed Admin item without Administration and remains read-only` and `keyboard policy edits retain focus and persist in desktop and mobile shells`;
- permissions/SSR: `non-admin shell hides Administration navigation` and `JavaScript-disabled Core, Organization, and Administration routes preserve native SSR ownership` (both require accurate replacement names); and
- smoke/UAT IDs: smoke `protected_server_rendered_shells` and `module_inventory_policy_and_navigation`; UAT `protected_server_rendered_routes` and `module_inventory_policy_and_navigation`.

New canonical proof must cover the ordinary unmatched Axum `/administration` 404 with no redirect, handler, or application tombstone, plus group CRUD/cross-group protection. The final manifest total is derived only after test discovery. Module-contract manifest/transition-fixture bytes and SHA sidecars are outside the approved change set and remain pinned.

## Recorded Changes

| Date | File and exact identity | Change type | Approved requirement/decision | Why old proof is no longer correct or stable | Behavior changed? | Equal-or-stronger replacement proof | Reviewer/commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| _None_ | | | | | | | |

## Closeout Reconciliation

Before closeout:

- compare this table with the complete Git diff for test, fixture, manifest, script, and visual-baseline paths;
- verify every new UI identity and visual baseline is confined to navigation composition, direct Admin destinations, Module Management, and capability provenance;
- require an exact row for every modification to existing proof;
- verify every unchanged Sprint 6A identity remains present and unchanged, every superseded identity has its logged equal-or-stronger replacement, and the final reconciled manifest/report has zero skipped, filtered, flaky, retried, or unexpected results;
- verify no pass depends on increased timeout or retries;
- review every new visual baseline and reject unexplained pixel churn; and
- derive and record the final manifest identities, count, and hash from current files rather than copying Sprint 6A's historical values.

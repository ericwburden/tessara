# Sprint 6A-UI Test Change Log

Status: reconciled on 2026-07-16. Approved Rust, Playwright, smoke, UAT, and manifest identities were replaced only where the schema-v2 composition, direct Admin routes, Module UX, provenance, or assignment-date contract superseded them. Timeouts, retries, unrelated assertions, descriptor fixtures, and historical migration bytes remain unchanged. The full locked workspace suite and proportional browser/script proof pass.

The closed Sprint 6A browser baseline is `end2end/acceptance-manifest.json`, schema version 2, SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`, containing 60 exact identities across seven files. It remains historical proof of the prior contract. Sprint 6A-UI intentionally supersedes only its hard-coded group/band/Administration expectations; every unrelated identity and assertion remains durable parity proof.

Tests are durable proof of supported behavior. A failing test is not authorization to rewrite it. Production code is corrected unless an explicit product decision changes the requirement.

New or changed Sprint 6A-UI proof is limited to the versioned navigation policy and migration, shell group composition, direct Core Admin destinations and exact `/administration` removal, Module Management UX and approved directory search/status behavior, and capability-provenance presentation. All unrelated accepted tests remain unchanged parity evidence; no application-wide visual baseline is introduced.

## Approved Behavior Changes

The product owner has approved these reasons for future proof replacement:

- replace hard-coded `Main`/`Admin`, Core-anchor, and reorder-band behavior with ordered configured groups and complete destination placements;
- allow every optional destination to move between groups, reorder, and show/hide;
- allow effective global `modules:manage_navigation` to create, rename, reorder, and delete empty custom groups;
- require stable non-deletable Main/Admin identities; protect Home and Organization in Main under their approved visibility rules; protect User Management, Roles & Access, Node Types, and Module Management in Admin; and make Operations optional;
- remove the Administration navigation destination, page, and exact `/administration` route without redirect while preserving direct `/administration/*` routes; and
- version the management and shell wires and migrate the contribution-only v1 policy without weakening authorization, revision, audit, SSR, no-JavaScript, or fail-closed behavior; and
- add functional Module directory search by display name/stable definition ID and exact availability/status filtering over the already-authorized projection, with stable canonical order, combined criteria, a distinct no-match state, and clear/reset behavior.
- expose the existing durable user-role assignment creation timestamp through the assignment read model and UI as `Assigned on`, with `Pending save` for a newly selected unsaved role and no change to assignment authorization or scope behavior.

These decisions authorize only the obsolete expectation to change. They do not pre-approve a particular test edit, name, fixture, count, timeout, selector, or weaker assertion. Each actual edit still requires an exact row below before it lands.

Sprint 6A-UI is UX-led rather than markup/CSS-only. Related behavior needed to deliver an approved UX outcome may be added with explicit acceptance coverage. Unrelated feature expansion must be approved before inclusion and recorded in the sprint plan before production or proof changes.

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

Additive Module directory proof must cover default exact content/order, trimmed case-insensitive display-name and definition-ID search, every exact status option, combined predicates, semantic result count, no-match/reset, keyboard/touch operation, authorization parity, and a useful complete SSR/no-JavaScript inventory. These new cases do not authorize weakening accepted catalog, descriptor, migration, authorization, or default seven-entry assertions.

## Recorded Changes

| Date | File and exact identity | Change type | Approved requirement/decision | Why old proof is no longer correct or stable | Behavior changed? | Equal-or-stronger replacement proof | Reviewer/commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2026-07-16 | `crates/tessara-web/src/state/shell_navigation.rs` — `supported_projection_preserves_core_anchors_and_band_local_reordering` → `supported_projection_accepts_policy_order_and_cross_group_movement`, plus shared schema fixture assertions | Superseded unit proof replaced | Approved arbitrary ordered groups, cross-group optional placement, direct Admin destinations, and shell schema v2 | Fixed ranks, exactly Main/Admin, and the Administration item are no longer valid production contracts | Yes | The replacement accepts policy order and optional cross-group moves, rejects protected Home displacement, retains ownership/route/schema/fail-closed and unknown-field negatives, and adds stable group IDs | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `crates/tessara-web/src/features/modules/policy.rs` — former contribution-band helper tests replaced by `destinations_move_within_and_between_groups_without_duplication`, `protected_destination_rejects_hide_and_cross_group_move`, and `nonempty_custom_group_must_be_emptied_before_deletion` | Superseded unit proof replaced | Approved complete group composer | Band-only helpers cannot represent custom groups or complete destination placement | Yes | Replacement proves dense movement within/between groups, no duplication, protected hide/group rejection, and empty-only custom deletion | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `crates/tessara-web/src/features/administration/components/capability_metadata.rs` — `metadata_html_exposes_scope_source_provider_state_and_digest` assertions | User-visible terminology strengthened | Asset 13 approval: `Authoritative source: Core` / `Also declared by … — Transitional in-process` | `Core — Core authoritative` duplicated the same fact and was explicitly rejected in review | Yes, presentation only | Same test retains Scope, module identity, transitional state, and complete digest proof while asserting the approved provenance label | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `crates/tessara-web/src/features/administration/components/user_forms/account.rs` — `account_role_assignment_explains_global_module_and_scoped_role_composition` | Existing presentation proof extended | Asset 15 approval for persisted `Assigned on`, pending-save, and separate scoped/global roles | Usage counts did not answer when the selected user received the assignment | Yes, additive readback | Existing role-composition assertion remains and additive model/UI tests cover persisted, pending, and unassigned treatments without changing assignment authority | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `end2end/tests/app.spec.ts` — `authenticated primary routes render in the native shell` | Superseded route proof replaced | Remove exact `/administration`; expose direct Core Admin destinations | The landing heading/cards no longer exist | Yes | Replacement proves ordinary 404/no redirect for `/administration`, then loads each direct Admin route in the authenticated native shell and retains the unrelated Datasets assertion | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `end2end/tests/modules.spec.ts` — `global read exposes the fixed Admin item without Administration and remains read-only` and `keyboard policy edits retain focus and persist in desktop and mobile shells` | Superseded browser/API proof replaced | Schema-v2 complete group composer, protected Core placements, cross-group movement, direct Admin destinations | Fixed-item exclusion, contribution bands, and table-row selectors cannot represent the approved policy | Yes | Replacement retains global read/manage authorization, fallback nondisclosure, descriptor parity, save/discard/focus, desktop/mobile shell parity, direct-route authority, atomic protected-item rejection, custom-group/cross-group behavior, and teardown restoration | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `end2end/tests/permissions.spec.ts` — `non-admin shell hides Administration navigation` and `JavaScript-disabled Core, Organization, and Administration routes preserve native SSR ownership` | Superseded names/assertions replaced | Remove aggregate Administration destination/route while preserving direct Admin routes and SSR | Absence of a destination that no longer exists is not meaningful permission proof, and `/administration` can no longer be a successful SSR route | Yes | Replacement proves the scoped actor receives only eligible configured product destinations, proves exact `/administration` 404/no redirect, and retains no-JS SSR proof for every direct Core Admin route | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `scripts/smoke.ps1` — `protected_server_rendered_shells` and `module_inventory_policy_and_navigation` | Superseded smoke assertions replaced | Direct Admin routes and schema-v2 policy/shell | The removed landing and schema-v1 fixed-item fields no longer exist | Yes | Replacement proves exact removed-route response, direct SSR shells, required groups, complete 13-destination policy, protected Module Management placement, schema-v2 shell, and absence of aggregate Administration | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `scripts/uat-sprint.ps1` — `protected_server_rendered_routes` and `module_inventory_policy_and_navigation` | Superseded UAT assertions replaced | Direct Admin routes and schema-v2 policy/shell | The removed landing and schema-v1 fixed-item fields no longer exist | Yes | Replacement proves exact removed-route response, direct routes, required groups, complete policy, protected Module Management placement, schema-v2 shell, and absence of aggregate Administration | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `crates/tessara-web/src/ui/shell/nav.rs` — `failed_fallback_retains_module_management_for_the_global_admin_sentinel` | Existing fail-closed fallback proof strengthened | Direct Core Admin destinations; no aggregate Administration destination | The fallback must not revive the removed landing item during a shell-projection outage | Yes | Replacement retains Module Management and outage disclosure while requiring User Management, Roles & Access, and Node Types and rejecting the aggregate Administration label/route | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `end2end/acceptance-manifest.json` — four superseded browser identities above | Manifest identity reconciliation | Exact logged test-name replacements for the approved navigation/route model | The old identity strings no longer exist after equal-or-stronger replacements | Names only; total remains 60 | Schema stays 2, all seven files and all other identities remain byte-for-byte unchanged, and validation derives the same total from discovery | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `end2end/tests/app.spec.ts` — `root route renders assigned work in the native shell` assertions | Existing shell proof updated in place | Four direct Core Admin destinations replace aggregate Administration | The removed aggregate link cannot remain a positive root-shell assertion | Yes | Home readiness, assigned-work content, Forms, no `/app` links, and clean console remain; additive assertions require all four direct Admin links and reject the aggregate link | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `end2end/tests/modules.spec.ts` — `directory and detail preserve human-machine parity and explicit route states` | Existing parity proof adapted to approved responsive hierarchy | Assets 01–04 and 10–11: one directory-level transition disclosure, working detail section switcher, descriptor-label correction, semantic navigation eligibility list | Repeating identical transition copy in every row and requiring all peer sections simultaneously visible contradict the approved contained directory/detail presentation | Presentation only | Exact seven-row identity/digest/API/bootstrap parity, every declaration field, lifecycle absence, route state, descriptor bytes/ETag, no-bridge/no-duplicate-load, and fault states remain; replacement additionally operates every detail tab and verifies the semantic eligibility list | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `crates/tessara-api/tests/modules.rs` — `module_http_apis_enforce_global_authority_and_preserve_exact_sources`, `navigation_policy_http_rejections_are_atomic_and_exactly_audited`, and `native_module_management_routes_render_authorized_restricted_and_not_found_states` | Superseded integration wire proof replaced | Schema-v2 complete groups/placements, direct Admin shell destinations, harmonized SSR count copy | Schema-v1 `immutable_core_items`/`contributions`, band mutation errors, aggregate Administration, and `7 contributions` are no longer supported outputs | Yes | Replacement retains all global/scoped/anonymous authority, nondisclosure, source/descriptor parity, no-op idempotency, complete atomic rejection/audit counts, stale revision, restoration, native headers/bootstrap/no-bridge/fault states, and adds exact required groups, 13 destinations, protected placement, dense-order/group validation, direct Admin items, and schema-v2 success payload proof | Product approval recorded in Direction 1; this implementation commit |
| 2026-07-16 | `crates/tessara-api/tests/sprint_6a_populated_upgrade.rs` — `populated_sprint_5a_upgrade_preserves_invariants_and_replaces_seed_atomically` and `fresh_startup_and_seed_assignment_lock_order_use_a_separate_database` | Existing mandatory migration proof advanced | Migration 004 is now part of every prepared database and intentionally moves the historical Datasets placement into the approved Main layout | Expecting migrations 1–3 and the schema-v1 Admin placement would stop before the sprint migration and falsely characterize the supported current schema | Yes, for approved navigation placement only | Replacement requires migrations 1–4, schema-v2 shell output, exact two groups/13 placements, deterministic Datasets placement, two versioned control-plane audits, and retains product/auth/session/role bytes, restart, concurrency, lock-order, and exact seed-repair proof | Product approval recorded in the sprint plan; this implementation commit |

## Closeout Reconciliation

Before closeout:

- compare this table with the complete Git diff for test, fixture, manifest, script, and visual-baseline paths;
- verify every new UI identity and visual baseline is confined to navigation composition, direct Admin destinations, Module Management, and capability provenance;
- verify search/status proof preserves the complete default inventory, canonical order, authorization, and useful SSR/no-JavaScript output while covering combined and no-match/reset behavior;
- require an exact row for every modification to existing proof;
- verify every unchanged Sprint 6A identity remains present and unchanged, every superseded identity has its logged equal-or-stronger replacement, and the final reconciled manifest/report has zero skipped, filtered, flaky, retried, or unexpected results;
- verify no pass depends on increased timeout or retries;
- review every new visual baseline and reject unexplained pixel churn; and
- derive and record the final manifest identities, count, and hash from current files rather than copying Sprint 6A's historical values.

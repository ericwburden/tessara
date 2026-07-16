# Sprint 6A-UI Navigation And Module Management Baseline

Status: source, persistence, wire, shell, route, browser-proof, and first visual characterization recorded on 2026-07-15. The approved dynamic-group model supersedes the earlier presentation-only/band-preserving brief. Initial optional-destination placement is the sole current product blocker; refreshed composer mockups are the next planned review artifact after that decision.

## Authority And Evidence Order

1. `docs/roadmap.md` and `docs/sprints/sprint-6a-ui-plan.md` define the approved prospective contract.
2. Closed Sprint 6A source, tests, and retained evidence define the historical behavior being migrated.
3. `docs/ui-guidance.md` and `docs/ui-guidance-spec.md` define the existing Tessara identity and UI posture.
4. Current-run screenshots and DOM inspection identify presentation defects but do not override functional authorities.
5. Every change to historical proof is controlled by `sprint-6a-ui-test-change-log.md`.

## Source And Regression Facts

- Branch: `codex/sprint-6a-ui`
- Sprint base: `c37153b19787d4164eaccbb4752980772e6ec84a`
- Closed Sprint 6A boundary: `f145e059fc1f4d81c960cb35e586c802831ecea2`
- Sprint 6A production boundary: `6580b040236f563c30b5162fa833d7b0fed16478`
- Current live route inventory: 48 mounted patterns plus not-found; target inventory after deleting only `/administration`: 47 plus not-found.
- Closed browser baseline: schema 2, 60 exact identities in seven files, SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`.
- Current migration inventory: `001_baseline.sql`, `002_dashboard_placement_capacity.sql`, and `003_module_control_plane.sql`. Sprint 6A-UI must add `004`; it must not edit `003`.
- Current policy endpoint: `GET|PUT /api/admin/navigation-policy`, guarded by effective global module capabilities and optimistic revision.
- Current actor shell endpoint: `GET /api/shell/navigation`, consumed by both SSR/hydration and desktop/mobile shell rendering.

The 60-test result remains durable historical proof. Band, fixed-Administration, and two-group assertions are characterization of the old contract; unrelated assertions remain acceptance requirements.

## Current Navigation Implementation

### Persistence

`003_module_control_plane.sql` stores:

- `module_navigation_contributions`, whose immutable metadata restricts `group_name` to `Main|Admin` and `reorder_band` to three named bands;
- one `navigation_policies` row per installation with monotonic revision; and
- `navigation_policy_entries` for contributions only, containing visibility and band-local order.

Core destinations and groups are not persisted. Catalog synchronization creates missing contribution policy entries with `ON CONFLICT DO NOTHING`, preserves current values, and rejects stored group/band drift from frozen catalog defaults.

### API And Service

- Policy response v1 exposes immutable Core items plus contributions with group, band, before/after anchors, visibility, and order.
- Policy mutation v1 is an atomic complete contribution collection.
- Service validation rejects all group and band changes and requires dense zero-based order within each band.
- Revision conflicts and successful/denied mutations are atomic and audited.

### Shell

- API shell composition duplicates the exact five Core items, six active contributions, two groups, and three bands.
- Browser validation rejects more than two groups, group names other than Main/Admin, unknown items, and movement across hard-coded ranks.
- The renderer already iterates projected groups generically; server composition, browser validation, and fallback composition do not.
- Capability and module-availability filtering occur after policy composition and must be preserved.

### Routes

- `/administration` mounts `AdministrationPage` and appears as a Core Admin destination.
- Users, Roles, Node Types, and Module Management already have direct `/administration/*` routes.
- Administration descendants currently use the generic `administration` active navigation key.
- Removing only the landing route leaves the `/administration/*` namespace intact and reduces the route inventory by one.

## Approved Target Inventory

### Required Groups

| Stable ID | Default label | Deletable | Reorderable | Label requirement |
| --- | --- | --- | --- | --- |
| `core.main` | Main | No | Yes | Required-group relabeling supported but not exit-critical |
| `core.admin` | Admin | No | Yes | Required-group relabeling supported but not exit-critical |

Custom groups use immutable `custom.<lowercase UUID v4>` IDs, validated labels, persisted order, and deletion only when empty by effective global `modules:manage_navigation`. Management readback retains empty groups; actor shell projections omit groups without eligible visible items.

### Built-In Destinations

| Stable ID | Route | Required group | Hide | Cross-group move | Reorder |
| --- | --- | --- | --- | --- | --- |
| `core.home` | `/` | Main | No | No | Yes |
| `core.organization` | `/organization` | Main | Yes | No | Yes |
| `core.admin.users` | `/administration/users` | Admin | No | No | Yes |
| `core.admin.roles` | `/administration/roles` | Admin | No | No | Yes |
| `core.admin.node_types` | `/administration/node-types` | Admin | No | No | Yes |
| `core.admin.modules` | `/administration/modules` | Admin | No | No | Yes |
| `core.operations` | `/operations` | Pending initial placement | Yes | Yes | Yes |

Forms, Workflows, Responses, Operations, Datasets, Components, Dashboards, and future eligible module contributions are optional placements that can move between groups, reorder, and hide.

After Decision Gate 1 freezes the six current contribution defaults, a later recognized destination must carry a Core-catalog default referencing a required group. Reconciliation appends it once, uses its catalog visibility default, and records a system audit; invalid/missing defaults fail closed. Custom groups are never catalog defaults, and legacy descriptor group/band values remain provenance only.

### Intentionally Removed

- Core `administration` destination;
- `AdministrationPage` and its landing cards;
- exact `/administration` route, application tombstone, and any redirect/fallback compatibility behavior; the request receives the ordinary unmatched Axum 404;
- hard-coded NavigationSection/NavigationBand composition; and
- band/anchor fields and validation in the current policy wire.

## Editable Implementation Footprint

| Layer | Current owners | Approved change |
| --- | --- | --- |
| Migration/schema | `crates/tessara-api/migrations/003_module_control_plane.sql` as historical source | Add `004`; populated SQL backfill only, while fresh materialization follows in startup reconciliation |
| Catalog/reconciliation | API module catalog, repository, service | Core built-in catalog, fresh 13-placement initialization, one-time recognized additions, no reset of valid customization |
| Management API | module DTO/routes/service/repository/native bootstrap | Versioned group-aware read/mutation, atomic graph validation, revision/audit retention |
| Shell API | API module shell projection | Arbitrary ordered groups and actor-filtered items; fail-closed Core fallback |
| Web state | `state/navigation.rs`, `state/shell_navigation.rs` | Remove two-group/band/rank assumptions; validate generic group projection and Core invariants |
| Shell UI | desktop/mobile navigation | Render configured groups and correct direct Admin active keys without rebranding the shell |
| Admin routes | administration route/page/export/breadcrumb owners | Remove landing route/page/nav item; preserve direct descendants |
| Composer UI | Module Management policy model/API/bootstrap/view | Group reader/manager, group CRUD/order, optional visibility/cross-group/order, protected constraints |
| Module UI | directory/detail/provenance/style | Retain targeted hierarchy, long-content, state, and responsive corrections |
| Proof | Rust/API/browser/scripts/migration evidence | Logged replacements plus stronger migration/invariant/authorization/UI proof |

Module descriptors contain frozen default group/order hints. Administrator placement must not rewrite descriptor bytes or digests; the new installation policy overrides those hints after discovery.

## Current-Run Visual Evidence

Captured from the live Sprint 6A application at `http://localhost:8080` on 2026-07-15, seeded administrator, dark theme, 1280×720:

| Evidence | Characterization |
| --- | --- |
| [Directory first viewport](../audits/sprint-6a-ui-module-management-2026-07-15/01-module-directory-current-accepted.png) | Runtime context dominates the useful first viewport; current shell is historical, not a target configuration. |
| [Directory inventory](../audits/sprint-6a-ui-module-management-2026-07-15/02-module-directory-inventory-policy-current-accepted.png) | Full machine values and five verbose columns clip and become difficult to associate. |
| [Forms detail](../audits/sprint-6a-ui-module-management-2026-07-15/03-module-detail-current-accepted.png) | Long overview/digest/declaration content overlaps and creates page-level horizontal scroll. |
| [Band policy manager](../audits/sprint-6a-ui-module-management-2026-07-15/04-navigation-policy-current-accepted.png) | Labels/IDs run together and band controls are dense; the entire band interaction model is now superseded. |

Existing headings, regions, tables, links/buttons, status text, revision/save semantics, and capability separation are strengths to retain.

[The three retained directory mockups](../mockups/sprint-6a-ui/README.md) remain Module inventory references only. Their sidebars and implied band policy are obsolete and cannot be approved as full-page targets.

## Prioritized Issue Matrix

| ID | Finding | Severity | Approved outcome | Durable proof |
| --- | --- | --- | --- | --- |
| NAV-01 | Groups, anchors, bands, catalog defaults, API, and browser ranks duplicate one rigid composition. | P1 | Generic revisioned groups/placements with one server composition authority and fail-closed client validation. | Migration, domain/API, shell projection, malformed-state, SSR/hydration tests. |
| NAV-02 | Current policy cannot create groups or move an optional destination across groups. | P1 | Runtime custom-group CRUD/order and free optional cross-group placement. | Positive/negative CRUD, membership, dense-order, revision, audit, keyboard/mobile proof. |
| NAV-03 | Core protection is encoded as non-persisted anchors rather than explicit independent rules. | P1 | Hard-coded Core catalog plus persisted placements and explicit remove/hide/group-lock/reorder flags. | Complete protection matrix and seed/reconciliation proof. |
| NAV-04 | Administration landing duplicates direct destinations and consumes a fixed anchor. | P1 | Remove nav item, page, and exact route; directly expose four Core Admin destinations. | Ordinary unmatched 404/no-redirect, direct-route auth/SSR, active-navigation, smoke/UAT/browser replacements. |
| NAV-05 | Contribution-only persistence cannot retain Core order, Organization visibility, or dynamic group identity. | P1 | Forward migration to complete groups/placements without resetting valid customization. | Fresh/populated/atomic failure/idempotence/backup-restore fingerprints. |
| NAV-06 | Shell response validation rejects valid future groups and order. | P1 | Versioned arbitrary-group projection with uniqueness, same-origin, ownership, reference, and Core-invariant validation. | API/web parity, malformed wires, unavailable fallback, desktop/mobile/no-JS proof. |
| MM-UI-01 | Inventory is unreadable and horizontally clipped at 1280px. | P1 | Intentional identity/state hierarchy, safe wrapping, and responsive containment. | Exact-field DOM proof and 1280/768/390 cases. |
| MM-UI-02 | Detail grid overlaps long content and actions. | P1 | Established wide/stacked responsive detail composition. | Detail parity, source action, heading order, zoom/viewport proof. |
| MM-UI-03 | Runtime context and lifecycle/provenance explanations compete with primary tasks. | P2 | Compact metadata and explicit supporting hierarchy without content loss. | Exact values/states and reviewed targeted visuals. |
| MM-UI-04 | Capability provenance needs alignment after direct Admin navigation changes. | P2 | Existing role/access patterns, no authority change. | Permission/source tests and focused visual evidence if edited. |

## Required Capture And Design Matrix

| State | Evidence status |
| --- | --- |
| Current v1 directory/detail/manager policy | Captured at 1280 dark |
| Approved initial sidebar with direct Admin destinations | Pending Decision Gate 1 |
| Group composer reader | Pending mockup/capture |
| Manager: create/rename/reorder/delete empty group | Pending mockup/capture |
| Manager: cross-group move, item reorder, visibility | Pending mockup/capture |
| Protection and non-empty delete rejection | Pending mockup/capture |
| Revision conflict/save/discard/focus | Pending implementation proof |
| Arbitrary groups desktop/tablet/mobile and both themes | Pending implementation proof |
| Restricted/unavailable/malformed fallback | Pending implementation proof |
| Harmonized directory/detail | Pending visual selection/implementation |

## Historical Proof Requiring Explicit Reconciliation

- `end2end/tests/app.spec.ts`: `root route renders assigned work in the native shell` and `authenticated primary routes render in the native shell` contain exact old shell composition.
- `end2end/tests/modules.spec.ts`: `global read exposes the fixed Admin item without Administration and remains read-only` and `keyboard policy edits retain focus and persist in desktop and mobile shells` freeze the fixed Admin item, immutable Core anchors, band-only reorder, and exact desktop/mobile ordering.
- `end2end/tests/permissions.spec.ts`: `non-admin shell hides Administration navigation` and `JavaScript-disabled Core, Organization, and Administration routes preserve native SSR ownership` make claims that become false when the exact landing route is removed; rename them only through logged equal-or-stronger replacements.
- API module integration/unit tests: v1 DTO shape, bands/anchors, immutable Core response, group/band rejection, counts, audit payloads, and revision restore.
- web navigation/shell tests: exactly two groups, hard-coded ranks, Core fallback, band crossing, fixed items, and current active route.
- smoke check IDs `protected_server_rendered_shells` and `module_inventory_policy_and_navigation`, plus UAT check IDs `protected_server_rendered_routes` and `module_inventory_policy_and_navigation`, are tied to the removed landing and band model. Replace them with direct-route/removal and group-policy proof; mutating UAT must restore the original policy in `finally`.
- deployment evidence currently requires migrations 1–3. Sprint 6A-UI evidence must require 1–4 under its own artifact namespace without rewriting retained Sprint 6A evidence or rollback manifests.
- `sprint_6a_populated_upgrade.rs` currently stops at migration 3. Add a populated migration-3-to-4 case pinned to migration 3 SHA-256 `a3489240633b6019ec9aa7acc0ee41b1f54ba625c02e30c8e925fd8b429ee50b`.

Known Rust proof identities that must be logged before modification include:

- API service: `policy_collection_validation_rejects_every_immutable_shape_change`, `policy_collection_validation_accepts_dense_within_band_reordering`, and `catalog_sync_is_repeatable_concurrent_and_rolls_back_injected_failure`;
- API routes: `policy_projection_exposes_fixed_anchors_and_never_makes_core_items_mutable`, `all_approved_bands_have_explicit_core_anchor_context`, and `policy_errors_keep_the_approved_stable_codes`;
- API shell projection: `default_navigation_sequences_are_exact_for_named_actors` and `malformed_band_falls_back_to_filtered_core_and_marks_unavailable`;
- API integration: `module_http_apis_enforce_global_authority_and_preserve_exact_sources`, `navigation_policy_http_rejections_are_atomic_and_exactly_audited`, and `native_module_management_routes_render_authorized_restricted_and_not_found_states`;
- web navigation/shell: static catalog/default composition, band-local movement, fixed-Core collision, malformed/missing policy, actor filtering, and hard-coded rank cases; and
- web Module policy: the band-bound movement case changes, while reader-only proof remains valid.

Add canonical browser identities for native `/administration` not-found/no-redirect behavior and manager group CRUD/cross-group protection. Derive the final manifest count only after discovery; do not pre-write an assumed total.

Keep the module-contract manifest, all seven transition descriptor fixtures, and their SHA sidecars byte-identical. Configurable placement supersedes their default-placement hints without changing module descriptor wire scope.

Every still-valid assertion in those tests remains. Only the explicitly superseded expectation may change, with an exact log row and equal-or-stronger replacement.

## Freeze Checklist

- [x] dynamic-group, cross-group movement, runtime group CRUD, required groups/destinations, Organization visibility, Operations optionality, and `/administration` removal are recorded;
- [x] current schema/API/shell/route/proof seams are mapped;
- [x] Module directory/detail visual defects and obsolete band UI are captured;
- [ ] initial optional-destination placement is approved;
- [x] fresh-versus-populated sequencing, legacy-provenance treatment, custom ID/label rules, revision/audit conversion, empty-group projection, and later-contribution default handling are frozen;
- [ ] v1-to-v2 current-placement migration mapping is frozen by Decision Gate 1;
- [ ] group-composer reader/manager/protection/mobile directions are reviewed;
- [ ] exact first test-change rows are approved before test edits; and
- [ ] implementation begins with v1 characterization and migration proof, not UI code.

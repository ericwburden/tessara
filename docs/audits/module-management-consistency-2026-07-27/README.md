# Module Management Consistency Audit

Date: 2026-07-27  
Source: Sprint 6C deployment at commit `d56bc817332ce5fb8f75592bb8fa739fb303b215`

## Verdict

Module Management has a consistent directory, detail header, nine-tab
information architecture, navigation policy surface, and status vocabulary.
All nine tabs were activated successfully for all eight live definitions with
no browser console errors.

At the audited source revision, it was not yet a reusable migration template. The two independently deployed
modules share a renderer, but Core still branches explicitly on
`tessara.dashboards` and Scoped Records. Configuration fields, control-plane
routing, enablement synchronization, diagnostics, findings, and some manifest
sections are module-specific code rather than module-provided metadata.

## Sprint 6C Remediation

The audit findings were used as implementation inputs later on 2026-07-27.
The current Sprint 6C worktree now:

- routes configuration, validation/apply, and enablement through one
  definition-to-control-endpoint registry;
- renders every manifest configuration property through one supported,
  validated schema subset;
- uses one in-product diagnostics, dependency, and findings template for
  Dashboard and Scoped Records;
- emits common lifecycle findings and uses the configured display label as the
  canonical independent-module label;
- renders route, typed-reference, operational-route, and shell contribution
  declarations from the manifest;
- fixed the duplicate transition configuration heading ID;
- aligned the Scoped Records validator/readback with its declared
  `retention_mode`;
- parameterizes the acceptance flow over every independently deployed module.

The reusable contract and adoption gate are documented in
`docs/architecture/independent-module-pathway.md`. The screenshots below remain
the before/remediation evidence from commit `d56bc817`; refreshed clean-source
closeout screenshots must be captured when Sprint 6C is committed and rebuilt.

## Audit Steps

| Step | Surface | Health | Result |
|---|---|---|---|
| 1 | Module directory | Good | One stable table and status vocabulary covers all eight definitions. |
| 2 | Transition overview: Forms | Good | Standard transition shell; no functional dependency or findings. |
| 3 | Transition overview: Workflows | Good | Standard shell; one declared transition-internal dependency finding. |
| 4 | Transition overview: Responses | Good | Standard shell; two declared transition-internal dependency findings. |
| 5 | Transition overview: Datasets | Good | Standard shell; two declared transition-internal dependency findings. |
| 6 | Transition overview: Components | Good | Standard shell; one declared transition-internal dependency finding. |
| 7 | Retired transition: Migration | Good | The same shell adds a state-driven retirement notice. |
| 8 | Independent overview: Dashboards | Good | Common lifecycle and provenance composition. |
| 9 | Independent overview: Scoped Records | Needs metadata cleanup | Common composition, but the displayed definition label is lower-case while the configured label is title case. |
| 10 | Scoped Records configuration | Incomplete | Common application-state card works; declared `retention_mode` is absent. |
| 11 | Dashboard configuration | Custom-coded | `default_page_size`, Components status, and transition note are hardcoded Dashboard branches. |
| 12 | Dashboard diagnostics | Custom-coded | Bespoke administration diagnostics replace the common Findings content. |
| 13 | Scoped Records health | Divergent | Leaves Module Management for a module-owned product route. |
| 14 | Scoped Records diagnostics | Divergent | Uses another visual/interaction contract and a data-URL download. |
| 15 | Transition configuration | Expected legacy state | Same tab exists, but only reports that no Module Instance exists. |
| 16 | Transition dependency detail | Good | Rich descriptor-driven dependency assessment is reusable. |
| 17 | Independent directory rows | Good | Both independent modules use the same release/instance columns and serving-state rules. |
| 18 | Scoped Records configuration edit | Incomplete | Only `display_label` is rendered from a schema that also declares `retention_mode`. |
| 19 | Dashboard configuration edit | Custom-coded | `default_page_size` is manually wired into Core and the web form. |

## Common Functionality Inventory

The following behavior is consistent and suitable for the template:

- directory search, status filter, columns, row navigation, and pagination;
- definition header, definition ID, source digest, status badges, source
  descriptor action, and deployment receipt action when a release exists;
- the nine tabs: Overview, Configuration, Declarations, Contracts,
  Capabilities, Dependencies, Resources, Navigation, and Findings;
- tab selection, URL hash updates, keyboard focus treatment, and zero console
  errors across every definition;
- transition-module overview structure and state-driven retirement treatment;
- independent-module definition, release, instance, lifecycle, diagnostics
  probe summary, and artifact provenance;
- navigation policy placement and visibility management;
- independent enable/disable presentation and readiness preconditions.

## Module-Specific Inventory

| Module | Current form | Module-specific metadata and behavior |
|---|---|---|
| Forms | Transitional, active in Core | 3 features, 2 contracts, 2 resources, 4 routes, 2 capabilities; no dependencies; no Module Instance configuration. |
| Workflows | Transitional, active in Core | 3 features, 3 contracts, 2 resources, 5 routes, 2 capabilities; requires Forms; 1 transition-internal finding. |
| Responses | Transitional, active in Core | 4 features, 2 contracts, 1 resource, 4 routes, 3 capabilities; requires Workflows and Forms; 2 findings. |
| Datasets | Transitional, active in Core | 4 features, 4 contracts, 3 resources, 8 routes, 4 capabilities; requires Responses and Forms; 2 findings. |
| Components | Transitional, active in Core | 4 features, 2 contracts, 1 resource, 6 routes, 2 capabilities; requires Dataset major lines; 1 finding. |
| Migration | Transitional, retired | No features, contracts, resources, routes, navigation, or capabilities; retirement notice and finding. |
| Dashboards | Independently deployed | Configuration: `display_label`, `default_page_size`; Components dependency; 2 features, 2 contracts, 1 resource, 9 routes, 2 capabilities; custom four-metric diagnostics, compatibility panel, transition warning, and diagnostics download. |
| Scoped Records | Independently deployed reference | Configuration: `display_label`, `retention_mode`; no functional dependency; 1 feature, 1 contract, 1 resource, 5 routes, 2 capabilities; module-owned health/diagnostics pages, revision facts, and sanitized diagnostics download. |

The frozen transitional Dashboard descriptor remains a reservation source but
is correctly superseded by the live independently deployed Dashboard row.

## Findings

### P1 — Configuration is not schema-driven

`IndependentConfigurationV1` promotes `display_label` and `retention_mode` to
fixed fields and places everything else in an untyped details map. The web
renderer then checks `definition.id == "tessara.dashboards"` to add
`default_page_size`. The HTML form and Core form decoder repeat the same
branch.

Impact: every upcoming migration with a new configuration field requires Core
API and web releases. Scoped Records already proves the data loss: its
manifest declares `retention_mode`, but the read and edit views omit it.

### P1 — Control-plane orchestration is definition-specific

Configuration validation/apply and enablement synchronization explicitly
branch between Scoped Records and Dashboards and reject every other
definition.

Impact: a new independently deployed module cannot use the otherwise-common
configuration or enablement UI without adding new Core code.

### P1 — Diagnostics have two incompatible navigation and rendering models

Dashboard diagnostics are hardcoded into the Module Management Findings tab.
Scoped Records navigates to module-owned Health and Diagnostics pages with a
different card system, heading model, fact set, and download mechanism.

Impact: administrators cannot learn one diagnostics workflow, and a future
module has no declared template to follow.

### P1 — Independent findings and operational metadata are not generic

The independent inventory projection always emits `findings: []`. Dashboard
diagnostic details are synthesized only when the definition ID is
`tessara.dashboards`. The generic independent manifest sections omit route
declarations, typed-reference schemas, operational routes, and shell
contribution contracts that exist in the manifest.

Impact: a future module can declare the information but Module Management
cannot present it consistently.

### P2 — Transition and independent tabs expose different detail depth

Transition dependencies include assessment, bindings, requirements, and
catalog findings. Independent dependencies are reduced to a two-column list.
Transition resources include semantic destinations; independent resources do
not.

Impact: a module loses administrative detail when it migrates unless the
independent template reaches parity first.

### P2 — User-facing label selection is inconsistent

Scoped Records appears as `scoped records` in the directory and heading while
its configured display label is `Scoped Records`.

Impact: module metadata has more than one user-facing source of truth.

### P2 — Duplicate heading IDs weaken assistive-technology mapping

Transitional detail DOM contains two
`id="module-configuration-heading"` elements and two matching
`aria-labelledby` references: one for the Configuration tab and one for the
configuration dimension inside Findings. Tab roles and `aria-selected`
otherwise behaved correctly.

Impact: assistive technology can resolve the wrong heading even though the
visual tab flow works.

## Recommended Migration Template

1. Replace the fixed independent configuration DTO with a versioned,
   schema-driven projection:
   - normalized JSON Schema;
   - current normalized values;
   - field order, labels, help text, sensitivity, and supported control type;
   - validation findings keyed by JSON Pointer.
2. Declare common control-plane operations in the release/instance contract:
   validate configuration, apply configuration, enable, disable, status,
   diagnostics, and sanitized diagnostics download. Core should invoke a
   generic module-control client, not definition-specific functions.
3. Define one typed diagnostics projection with:
   summary metrics, operational facts, dependency assessments, findings,
   observation timestamps, and a download action. Render it inside the same
   Module Management tab for every independent module.
4. Render all manifest-backed tabs from one shared component for transition
   and independent definitions. Availability and deployment state may change
   the values, not the structure or information depth.
5. Treat Dashboard’s Components binding, migration warning, and adapter status
   as typed dependency/finding records. Treat Scoped Records revisions and
   retention mode as ordinary schema/fact records.
6. Use one canonical display label source for directory, heading, navigation,
   and configuration.
7. Parameterize acceptance tests over every independently deployed module and
   assert:
   identical shell/actions/tabs; every declared configuration property is
   visible and editable; generic health/diagnostics navigation; enable/disable
   behavior; manifest-section parity; findings; keyboard semantics; and no
   definition-ID branches in Core presentation code.

## Source Evidence

- Shared independent configuration/diagnostics DTO:
  `crates/tessara-module-contract/src/inventory.rs:50`.
- Dashboard-specific independent projection and empty findings:
  `crates/tessara-api/src/modules/routes.rs:670`.
- Dashboard-specific UI branches:
  `crates/tessara-web/src/features/modules/pages.rs:575`.
- Definition-specific form, validation, and enablement routing:
  `crates/tessara-api/src/core_security.rs:1556`.
- Duplicate transition heading IDs:
  `crates/tessara-web/src/features/modules/detail.rs:218` and `:393`.
- Existing independent-module acceptance test:
  `end2end/tests/modules.spec.ts:965`.

## Screenshot Evidence

### Directory and every module overview

![Module directory](01-module-directory.png)
![Forms overview](02-tessara-forms-overview.png)
![Workflows overview](03-tessara-workflows-overview.png)
![Responses overview](04-tessara-responses-overview.png)
![Datasets overview](05-tessara-datasets-overview.png)
![Components overview](06-tessara-components-overview.png)
![Retired Migration overview](07-tessara-migration-overview.png)
![Dashboards overview](08-tessara-dashboards-overview.png)
![Scoped Records overview](09-tessara-reference-scoped-records-overview.png)

### Configuration and diagnostics comparisons

![Scoped Records configuration](10-scoped-records-configuration.png)
![Dashboards configuration](11-dashboards-configuration.png)
![Dashboard diagnostics](12-dashboards-diagnostics.png)
![Scoped Records health](13-scoped-records-health.png)
![Scoped Records diagnostics](14-scoped-records-diagnostics.png)
![Transition configuration](15-transition-configuration.png)
![Responses dependencies](16-responses-dependencies.png)
![Independent module directory rows](17-directory-independent-modules.png)
![Scoped Records configuration editor](18-scoped-records-configuration-edit.png)
![Dashboard configuration editor](19-dashboards-configuration-edit.png)

## Evidence Limits

- This was a read-only audit. Configuration values and enablement were not
  submitted or changed.
- Keyboard focus and ARIA tab state were observed, but this is not a complete
  screen-reader or WCAG conformance test.
- Only the two currently independent modules can prove the future template.
  Transitional definitions do not yet expose Module Release/Instance
  operations by design.

# Sprint 8B Candidate Rehearsal Browser Defect Convergence

Formal validation is paused. This record classifies the failed
`rehearsal-browser` attempt at source commit
`c9f05db3839bbe0d0f277a0c24e3a4608d3e2e03`, tree
`62a34930e60666bda03bcef205712eb93410720d`, environment fingerprint
`1b82598e5f10a0c851940b627d5e8de3f49861061453cf8e7d6234b29622982f`.
The attempt retained 32 terminal test failures/errors. Its raw transcript is
`artifacts/sprint-8b-closeout/candidate-rehearsal/lanes/rehearsal-browser/attempts/20260819T024813809Z-7c72cb4a/command.log` with SHA-256
`3691875cfb12c02c5bf2a3004223fcf6b4a8f71bfbe32c58a7081f148e38ed98`.

The schema-valid machine record is retained beside that attempt as
`defect-provenance.json`. Candidate Rehearsal remains blocked until the record
is verified against clean committed implementation evidence.

## Failure classification

| Distinct failure | Affected terminal tests/errors | Classification | Governing requirement | Evidence and root cause | Correction and replacement proof |
|---|---:|---|---|---|---|
| Retired analytics physical identities and result counts | 4 | 4. Superseded assertion caused by an approved contract change | `ac-06`, `ac-07`, `ac-18`, `ac-26` | The assertions selected Sprint 7A physical fixtures, expected four rows, and expected the disjoint Component to be disclosed. The signed Sprint 8B Reference topology creates typed logical owner receipts, one current in-scope row, and one nondisclosing restricted placement. | Discover the five canonical logical products, require the one current result, exclude same-time/out-of-scope changes, and require the disjoint placement without Component identity. Coverage is stronger because it proves exact v2 result and nondisclosure. |
| Core hydration root on module routes | 1 | 4. Superseded assertion caused by an approved contract change | `ac-03`, `ac-20` | The test required `#app-root` after navigating to a module-owned document. The approved lifecycle ABI owns `#module-content` for direct module documents and `#tessara-module-outlet` when hosted by Core. | Require the exact active lifecycle root and `data-hydration=ready`, retaining direct-load and Core-hosted route coverage. |
| Aggregate shell text included decorative SVG titles | 1 | 4. Superseded assertion caused by an approved contract change | `ac-03` | `toHaveText` observed decorative icon `<title>` content despite the icon being presentation-only. | Assert the visible `.sidebar-link__label`, exact active href, grouped navigation, and document title. |
| Visible-column control expected ARIA `group` | 1 | 4. Superseded assertion caused by an approved contract change | `ac-03` | The shared SDK control is an accessible modal dialog, not a passive group. The stale locator waited until test timeout. | Require `dialog` named `Visible columns`, then toggle the exact column and prove the owner update request/result. |
| Component sent Dataset schema v1 | 1 | 1. Product implementation defect | `ac-06`, `ac-07`, `ac-18` | The Component UI and direct distinct-value producer retained literal `schema_version: 1` after Dataset v2 became the only current contract. | Consume `DATASET_CONTRACT_SCHEMA_VERSION` in product code and send exact v2 in the direct boundary proof; v1 remains rejected. |
| Shared theme menu could not reliably select exact options | 2 | 1. Product implementation defect | `ac-03` | Decorative icon titles polluted accessible option names, and SDK-rendered options omitted the `data-theme-value` hook used by the canonical pre-hydration shell script. | Hide decorative icons from accessibility, emit the one canonical theme hook, and retain keyboard, stored-theme, cross-module, and 200% containment proof. |
| Dashboard fixture did not contain more than ten active rows | 1 | 2. Fixture/reference-topology defect | `ac-10`, `ac-18` | The assertion inherited a Sprint 8A multi-page seed assumption; signed Reference has four active owner-created rows and the embedded compact page size is ten. | Require all four rows in the UI, disabled next-page state, and exercise the same mediated owner endpoint at page size two for an exact two-page opaque-cursor proof. No synthetic Responses are added. |
| Dataset route title/root and nonempty publish body | 3 | 4. Superseded assertion caused by an approved contract change | `ac-03`, `ac-09`, `ac-20` | Retained tests expected Core route titles/readiness and sent a body to an action whose current owner contract requires empty bytes. | Require the canonical `Datasets` shell title and lifecycle root; publish with an empty body and retain exact replay/malformed-body negatives. |
| Dataset authoring and revision documents lacked complete SSR/hydration state | 3 | 1. Product implementation defect | `ac-03`, `ac-20` | Route documents did not seed every editor/revision projection from owner DTOs, causing incomplete initial state and redundant hydration option fetches. | Seed complete canonical editor data, issue no duplicate success-path option fetch, fetch exactly one selected FormVersion schema, and retry only a degraded bootstrap through Dataset-owned loaders. |
| Missing Dataset revision surfaced a generic route failure | 1 | 1. Product implementation defect | `ac-03`, `ac-20` | Revision document construction propagated owner `NotFound` instead of producing the frozen sanitized unavailable view. | Add the canonical `RevisionUnavailable` bootstrap variant and render the same safe state in direct and lifecycle documents. |
| Component visual baseline failures | 7 | Mixed: 1. Product implementation defect and 4. Superseded assertion | `ac-03`, `ac-06`, `ac-18` | The seven failures were downstream of the stale visible-title/hydration assertions and the real theme/v2-consumer defects above, not seven independent product causes. | Regenerate source assets/manifests, retain reviewed baseline images, and run normal comparison with no snapshot-update mode after focused semantic tests pass. |
| Exact PostgreSQL binding absent from formal browser action | 2 | 3. Validation harness or environment defect | `ac-10`, `gate-implementation-exit` | Both Module Management tests aborted before assertions because the lane omitted the retained topology's exact container, database, and user. | Project the authenticated setup binding into Playwright and require it in lane environment evidence; never infer or fall back to another database. |
| Dataset catalog schema and nondisclosure status were stale | 1 | 4. Superseded assertion caused by an approved contract change | `ac-06`, `ac-07` | The permissions assertion required Dataset schema v1 and `403` for a known out-of-scope Dataset. Dataset v2 intentionally makes known-hidden and random identities indistinguishable. | Require v2 and exact nondisclosing `404` pairs while retaining in-scope access and capability/scope checks. |
| Demo-only workflow node-type slug | 4 | 2. Fixture/reference-topology defect | `ac-10`, `ac-15`, `ac-18` | The tests selected demo `activity`; Reference creates `scope.organization` through Core owner bootstrap. | Resolve the source-exact Reference node type and retain all four create/publish/assign/start/edit workflow scenarios. |
| Configuration schema version projected twice | latent after the environment blocker was removed | 1. Product implementation defect | `ac-16` | Core used `schema_version` as the configuration envelope and also generated a second visible required form field from the module schema. | Consume the envelope field once, seed the canonical module payload with it, and prove the complete form submits without duplicate authority. |
| Target module security projection lagged Core revisions | latent after the environment blocker was removed | 1. Product implementation defect | `ac-06`, `ac-07`, `ac-18` | Authorization exchange synchronized the presenting module but issued a downstream grant before synchronizing the exact target Module Instance. The target correctly rejected the newer grant revision. | Synchronize only Module Instance targets immediately before signing the downstream grant; preserve the separate Core provider path and exact target manifest/instance binding. |

## Approved expectation changes

No expectation may change merely to obtain a pass. The permitted changes above
are limited to approved Sprint 8B contract consequences:

- Dataset v2 and Module Instance ownership: `ac-06`.
- Nondisclosing scope enforcement and exact cross-module results: `ac-07` and
  `ac-18`.
- Empty-body idempotent publish and retry semantics: `ac-09`.
- Owner-created source-exact Reference topology: `ac-10` and `ac-15`.
- Shared SDK shell, direct/lifecycle parity, SSR, and hydration: `ac-03` and
  `ac-20`.
- Core analytics separation: `ac-26`.

Each changed assertion is recorded in
`docs/sprints/sprint-8b-test-change-log.md`. The replacement coverage keeps the
same accepted inventory and adds exact logical identity, nondisclosure,
no-duplicate-fetch, opaque-cursor, or accessibility assertions.

## Implementation-to-requirement and target mapping

| Capability correction | Requirements | Required implementation targets |
|---|---|---|
| Shared shell, theme, route SSR/hydration, reviewed visuals | `ac-03`, `ac-20` | `static-quality`, `inventory-navigation`, `ui-sdk-conformance`, `fixture-acceptance`, `ui-provider-boundaries`, `deployed-smoke` |
| Dataset v2 Component/Dashboard consumption and target authorization | `ac-06`, `ac-07`, `ac-18` | `contract-boundary`, `consumer-cutover`, `owner-product`, `resource-resolution`, `deployed-smoke`, `clean-materialization` |
| Dataset publish/revision behavior and unavailable state | `ac-03`, `ac-09`, `ac-20` | `owner-product`, `api-idempotency`, `ui-sdk-conformance`, `ui-provider-boundaries` |
| Source-exact analytics, Dashboard pagination, permissions, workflow fixtures | `ac-07`, `ac-10`, `ac-15`, `ac-18`, `ac-26` | `fixture-acceptance`, `migration-seed`, `clean-materialization`, `semantic-noop`, `consumer-cutover`, `core-subtraction`, `reverse-consumers`, `deployed-smoke` |
| Formal database binding and failure provenance rules | `gate-implementation-exit` | `planning-contract-alignment`, `runner-selftest`, `uat-readiness` |
| Failure and restoration semantics after the touched cross-module path | `ac-07`, `ac-09` | `failure-recovery`, `response-incremental-sync`, `dataset-refresh-dag` |

Because the tracked validation skills and shared module SDK also changed, the
conservative dependency map requires a fresh result across all 24 Sprint 8B
implementation targets, not reuse of the earlier receipt.

## Exit conditions

This record may move to verified only after all focused browser families pass,
all affected Rust and boundary checks pass with warnings denied, module assets
and manifest/catalog digests reconcile, the exact 95-test Reference inventory
passes as non-authoritative implementation proof, every implementation target
passes against clean committed source, and a newly finalized
`implementation-readiness-result.json` reports zero known failures. The next
permitted formal boundary is fresh Validation Readiness; Rehearsal, Preflight,
SIT, and UAT are not authorized by this implementation record.

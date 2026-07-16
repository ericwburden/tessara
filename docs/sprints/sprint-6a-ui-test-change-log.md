# Sprint 6A-UI Test Change Log

Status: open. No existing test or accepted fixture has been changed at kickoff.

The accepted browser baseline is `end2end/acceptance-manifest.json`, schema version 2, SHA-256 `95f9a7468315277ab64595f4f36675a0cec7202d7865c257ffd31ecad55eeb1a`, containing 60 exact identities across seven files.

Tests are durable proof of supported behavior. A failing test is not authorization to rewrite it. Production code is corrected unless an explicit product decision changes the requirement.

New or changed Sprint 6A-UI proof is limited to Module Management, its policy controls, the Sprint 6A-added Administration entry, and capability-provenance presentation. All other accepted tests remain unchanged parity evidence; no application-wide visual baseline is introduced.

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

## Recorded Changes

| Date | File and exact identity | Change type | Approved requirement/decision | Why old proof is no longer correct or stable | Behavior changed? | Equal-or-stronger replacement proof | Reviewer/commit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| _None_ | | | | | | | |

## Closeout Reconciliation

Before closeout:

- compare this table with the complete Git diff for test, fixture, manifest, script, and visual-baseline paths;
- verify every new UI identity and visual baseline is confined to the targeted Sprint 6A surfaces;
- require an exact row for every modification to existing proof;
- verify the final browser report contains all 60 frozen identities with zero skipped, filtered, flaky, retried, or unexpected results;
- verify no pass depends on increased timeout or retries;
- review every new visual baseline and reject unexplained pixel churn; and
- record newly observed test counts as current evidence rather than copying Sprint 6A's historical counts.

# UAT scenario classification

Use this boundary when planning, implementing, reviewing, or executing a new
Tessara UAT inventory. Retained sprint inventories and evidence keep their
original classification; do not rewrite historical results.

## Deterministic checks are automated

A check is scripted automation when a program can determine pass or fail from
declared inputs without human judgment. It does not become human UAT because an
operator could read the artifact or run the command manually.

Automate, at minimum:

- parsing JSON, YAML, CSV, logs, receipts, manifests, or evidence indexes and
  checking exact keys, values, counts, order, state, or absence;
- comparing hashes, source/candidate identities, release sequences,
  prerequisites, topology inventories, cleanup residue, or provenance chains;
- scanning source or generated artifacts for ownership/subtraction rules;
- checking API responses, database-isolation matrices, migration/no-op results,
  recovery state, or other machine-readable product and operational contracts;
  and
- deterministic browser behavior that Playwright or another repository-owned
  browser check can assert reliably.

Place these assertions in a focused implementation target, SIT/deployed-smoke
lane, or scripted UAT scenario as appropriate. Execute formal v3 coverage
through its adapter lane. A human may inspect the artifact for diagnosis or
audit, but that inspection is not manual UAT evidence.

## Manual UAT requires a human acceptance question

A manual scenario is justified only when it exercises the actual product
surface and asks for irreducible human observation or judgment, such as
discoverability, comprehensibility, visual hierarchy, legibility, interaction
confidence, perceived continuity, or whether workflow feedback is useful to
the intended actor. Record the exact human-judgment question and why a stable
deterministic oracle is insufficient.

Every sprint-delivered user-facing screen and feature requires at least one
human exploratory touch during the sprint. The standing human question is
whether the intended actor can find, understand, and use the delivered surface
and whether interaction reveals an unexpected defect, regression, inconsistency,
or worthwhile UI/UX improvement that deterministic assertions did not expose.
One coherent journey may cover multiple screens or features, but none may be
unmapped. Materially different role, responsive, direct-document, lifecycle-
navigation, or failure-state experiences need their own touch when the human
experience differs.

Clicking through a UI is not by itself human UAT. If every expected result is
an exact route, status, field, count, or other deterministic state, automate the
scenario. Do not create a manual checklist whose substantive action is to open
or review retained JSON evidence.

For a mixed scenario, split the coverage:

1. Automation establishes fixtures, provenance, exact state, and every
   deterministic assertion.
2. The manual scenario references the passing automated receipt as a
   prerequisite and contains the product interaction, exploratory touch, and
   human acceptance question rather than repeating machine checks.
3. Manual coverage may be not applicable only to a purely technical requirement
   with no user-facing screen or feature. It is never not applicable to a
   screen or feature delivered by the sprint.

Every acceptance requirement needs automated evidence. Manual coverage follows
the delivered screen/feature inventory rather than a one-scenario-per-clause
quota: consolidate related surfaces into useful exploratory journeys without
leaving any delivered user-facing surface untouched.

## Screen and feature touch inventory

Before freeze, maintain an exact inventory of every user-facing screen and
feature implemented or materially changed in the sprint. Record its route or
surface, intended actor, materially distinct experience states, manual scenario
ID, and automated prerequisite receipts. Audit the reverse mapping so every
manual scenario names what it touches and every inventory item has coverage.

During execution, record observations even when the planned acceptance steps
pass. Classify each observation as:

- an acceptance defect or unexpected regression, which follows the normal
  defect-provenance and invalidation policy;
- a previously existing issue discovered during exploration, which is retained
  and routed for an explicit scope/authority decision rather than silently
  ignored or automatically expanding the sprint; or
- a non-blocking UI/UX improvement, which records the affected surface,
  observation, rationale, and suggested follow-up for closeout/future work and
  does not rewrite current accepted behavior.

## Classification audit

Before freezing the acceptance inventory, reject or reclassify any manual
scenario whose pass/fail decision can be derived entirely from machine-readable
artifacts or deterministic commands. Require each retained manual scenario to
name:

- the product surface and actor;
- the direct interaction the person performs;
- the irreducible human-judgment question; and
- the automated prerequisite receipts that already cover deterministic state.

Also reject a UAT inventory when any delivered user-facing screen or feature is
absent from the touch inventory or lacks a manual scenario mapping.

If execution discovers a misclassified frozen scenario, stop and return the
inventory gap to the validation coordinator. Add the automated coverage while
source is mutable and apply the governing inventory/candidate invalidation
rule; do not obtain a human pass by asking someone to review JSON.

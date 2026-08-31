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

Clicking through a UI is not by itself human UAT. If every expected result is
an exact route, status, field, count, or other deterministic state, automate the
scenario. Do not create a manual checklist whose substantive action is to open
or review retained JSON evidence.

For a mixed scenario, split the coverage:

1. Automation establishes fixtures, provenance, exact state, and every
   deterministic assertion.
2. The manual scenario references the passing automated receipt as a
   prerequisite and contains only the remaining product interaction and human
   acceptance question.
3. If no human question remains, omit the manual scenario and record manual
   coverage as not applicable for that requirement.

Every acceptance requirement needs automated evidence. Manual evidence is
required only where the requirement has a genuine human-observable acceptance
concern; never invent manual work to satisfy a one-manual-scenario-per-clause
quota.

## Classification audit

Before freezing the acceptance inventory, reject or reclassify any manual
scenario whose pass/fail decision can be derived entirely from machine-readable
artifacts or deterministic commands. Require each retained manual scenario to
name:

- the product surface and actor;
- the direct interaction the person performs;
- the irreducible human-judgment question; and
- the automated prerequisite receipts that already cover deterministic state.

If execution discovers a misclassified frozen scenario, stop and return the
inventory gap to the validation coordinator. Add the automated coverage while
source is mutable and apply the governing inventory/candidate invalidation
rule; do not obtain a human pass by asking someone to review JSON.

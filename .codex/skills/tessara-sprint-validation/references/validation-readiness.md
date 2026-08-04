# Tessara Validation Readiness and Candidate Rehearsal

This reference defines the mandatory mutable-build process before every
candidate freeze. `tessara-sprint-validation` owns both gates. Preflight audits
their receipts and freezes the candidate; SIT and UAT do not rerun them.

## Gate 1: Test Readiness

Derive an executable checklist from the sprint's actual validation record,
acceptance inventory, runners, deployment configuration, and source. Do not
reuse a generic list without reconciling it to the sprint.

The checklist must verify:

- every SIT, deployed-smoke, scripted UAT, manual UAT, evidence, recovery, and
  environment need;
- exact environment-variable names, database identities, destructive-reset
  acknowledgements, and safe disposable scope;
- required tool versions and every shell/runtime the repository claims to
  support, including runner parsing and invocation in each supported shell;
- ports, Compose project/profile, databases, topology, service health,
  handoff slot, and source-provenance inputs and label keys;
- fixtures semantically: actors, credentials usability, roles, capability and
  scope assignments, products, allowed and blocked resources, provider and
  security revisions, recognizable data, negative identifiers/services, and
  repeat-run idempotence;
- every runner, argument contract, output-path form, receipt writer, SHA-256
  helper, atomic finalization path, and failure/supersession path by self-test
  or a safe disposable probe;
- every acceptance clause mapped to automated, deployed-smoke, and manual
  evidence, with explicit justified `N/A` entries rather than blanks; and
- a clean repository plus source-exact build inputs before rehearsal begins.

Run independent checks fail-late. Retain a checklist result for every item,
including exact command/runtime, timestamps, exit status, evidence path, and
failure classification. Write `validation-readiness-result.json` only when all
items pass. Hash it and include its evidence in the manifest. A failed gate
keeps the build mutable and forbids candidate freeze.

## Gate 2: Candidate Rehearsal

Build or materialize a source-exact but explicitly mutable and
non-authoritative rehearsal build. Record a rehearsal source identity from the
commit, tree, dirty state, acceptance inventory, deployment inputs, and
environment contract. Do not issue a candidate fingerprint or candidate
receipt.

Run a complete validation-shaped pass containing:

1. all static, formatting, compilation, lint, schema, manifest, link, and
   boundary checks;
2. the full Rust workspace test contract and required optimized/timing lanes;
3. source-exact image build, provenance audit, deployment/materialization,
   migrations, topology health, and exact no-op/idempotence proof;
4. the complete Playwright acceptance inventory with its required workers,
   retries, runtime binding, discovery, and retained outputs;
5. authorization conformance and nondisclosure checks;
6. general and sprint-specific deployed smoke;
7. failure containment, recovery, canonical restoration, and final health;
   and
8. automated diagnostic equivalents of every UAT scenario, including semantic
   fixture verification. These checks are not formal UAT.

Never label rehearsal output authoritative SIT or UAT. Formal UAT remains
forbidden until authoritative SIT passes after freeze.

Run independent sibling checks fail-late when it is safe, so one failure does
not hide other defects. Stop dependent or destructive work whose prerequisite
state is invalid. Retain raw logs and per-lane diagnostic receipts under a
rehearsal namespace.

After the pass, collect every discovered product, test, harness, fixture,
acceptance-inventory, deployment, environment-contract, and evidence defect
into one batch. Correct the batch while the build remains mutable. Do not
freeze an intermediate correction.

Repeat the complete Test Readiness Gate and the complete Candidate Rehearsal
after every correction batch until both pass cleanly. Focused or narrow
reproducers may diagnose corrections but cannot satisfy rehearsal and cannot
replace the complete affected-lane rerun inside the next full rehearsal.

Write and hash `candidate-rehearsal-result.json` only after every rehearsal
lane passes, canonical restoration succeeds, and no defect remains open. It
must name and hash the passing readiness receipt and bind the exact mutable
source/environment identities that preflight will audit.

## Freeze boundary

Preflight may freeze a candidate only when:

- both result receipts parse, hash, and pass;
- rehearsal names the current readiness receipt;
- current clean source, acceptance inventory, deployment inputs, and
  environment match the passing rehearsal identities exactly; and
- no correction or acceptance decision occurred after the passing rehearsal.

Any mismatch returns to the complete readiness-and-rehearsal cycle. After
freeze, use the existing candidate fingerprint, receipt chain, evidence
retention, invalidation matrix, and phase authority rules without relaxation.

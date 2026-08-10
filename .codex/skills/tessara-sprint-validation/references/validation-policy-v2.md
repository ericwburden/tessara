# Tessara Validation Policy v2

This policy applies only when the tracked sprint validation contract declares
`policy_version: tessara-validation-v2`. Sprint 8A and earlier evidence remains
governed by the protocol under which it was issued. Never retrofit, rewrite, or
reinterpret a legacy receipt as a v2 certificate.

## Governing rules

1. Known validation targets pass during implementation, before formal
   validation begins.
2. A certified phase remains closed until a changed dependency meaningfully
   challenges its integrity.
3. Downstream phases consume compact certificates and their hashes. Raw
   evidence is retained for diagnosis and audit, not reread during routine
   authorization.
4. Candidate Rehearsal is final pre-freeze certification, not the primary
   implementation or debugging loop.
5. A candidate-changing correction after freeze creates a successor candidate
   and requires complete authoritative SIT and UAT. Pre-freeze lane reuse never
   substitutes for those candidate-bound phases.

## Tracked validation contract

Kickoff creates `docs/sprints/<sprint-slug>-validation-contract.json` and
validates it against `validation-contract.schema.json`. It is the machine-
readable companion to the sprint plan and verification record. It declares:

- roadmap and acceptance requirement identities;
- exact implementation targets and formal validation lanes;
- lane prerequisites and dependency domains;
- tracked input patterns for every dependency domain;
- environment and clean-materialization requirements; and
- the ignored evidence root and retention policy.

Every requirement maps to at least one implementation target and one formal
validation lane. Every target and lane names at least one dependency domain.
Every changed tracked path must map to a declared domain. An unmapped path,
ambiguous mapping, missing digest, or unauthenticated receipt selects the
conservative full affected-phase boundary.

## Dependency domains

Use these canonical domains unless a sprint records a narrower owned domain:

- `product-source`
- `build-dependencies`
- `migrations-seeds`
- `deployment-materialization`
- `fixtures`
- `acceptance-inventory`
- `environment-contract`
- `validation-shared`
- `readiness-runner`
- `rehearsal-runner`
- `preflight-runner`
- `sit-runner`
- `uat-runner`
- `evidence-publication`
- `documentation`

A domain fingerprint is SHA-256 over the canonical sorted inventory of its
tracked path plus content identity. Environment domains use a secret-free
canonical environment fingerprint. Record the observed commit and tree for
provenance, but determine certificate validity from the certificate's declared
dependency fingerprints rather than from unrelated repository changes.

Documentation is non-affecting only when it cannot alter executable behavior,
acceptance scope, validation interpretation, commands, or evidence semantics.
Validation policy, schemas, runner instructions, and acceptance documents are
not ordinary documentation for impact purposes.

## Implementation exit gate

Formal Readiness requires a passing
`implementation-readiness-result.json` validated against its schema and the
tracked validation contract. The result is non-authoritative and records:

- the clean source identity and validation-contract hash;
- changed and affected dependency domains;
- every selected known target, exact command, result, and evidence reference;
- clean-disposable-environment status where required;
- materialization, semantic no-op, failure containment, recovery, fixture,
  runner, smoke, and acceptance proofs selected by the affected domains;
- zero known failures; and
- cleanup/restoration status.

The implementation result does not replace formal validation. It proves that
the known contract was implemented and exercised before certification. A
missing target, failed target, runner self-test failure, dirty source, or
required clean-environment proof that was not run blocks Readiness entry.

## Compact phase certificates

Keep the established canonical result names, including
`validation-readiness-result.json`, `candidate-rehearsal-result.json`,
`preflight-result.json`, `sit-result.json`, and `uat-result.json`. Under v2 they
validate against `phase-certificate.schema.json` and contain summaries rather
than raw logs or embedded historical receipt chains.

Each certificate includes:

- source, environment, and when applicable candidate identity;
- exact prerequisite certificate paths and SHA-256 values;
- current dependency-domain fingerprints;
- the declared-lane inventory digest;
- one summary for every lane;
- executed versus inherited lane counts;
- open-defect count and restoration result; and
- one sealed phase-local evidence-index path and SHA-256.

An inherited pre-freeze lane uses
`certification_basis: inherited_nonimpact`. It retains the prior lane receipt
and hash, prior source/environment identity, its prior dependency fingerprints,
and an explicit non-impact rationale. It has null current execution timestamps
and does not claim that assertions ran again. It is valid only when every
declared dependency fingerprint and prerequisite certificate remains unchanged.

## Pre-freeze recertification

Readiness and Candidate Rehearsal use one impact plan, fixed before assertions:

- execute never-certified, previously failed, newly reachable, and impacted
  lanes plus their prerequisite closure;
- inherit only authenticated prior-passing lanes whose complete dependency set
  is unchanged;
- execute safe independent selected lanes fail-late;
- block a dependent lane when its current prerequisite is invalid;
- always run required teardown, recovery, restoration, and final safety checks
  when the current attempt touched live state; and
- fall back to complete phase execution when impact or prior evidence cannot be
  authenticated.

V2 has no rehearsal deferral counter or diagnostic Wave A/Wave B state. A
failed pass retains one terminal attempt, one harvest, and one consolidated
defect batch. After correction and a passing implementation exit gate, the next
pre-freeze pass executes its affected plan and may inherit unchanged lanes.
A phase passes only when every declared lane is either newly passed or validly
inherited, no lane is failed or blocked, no defect is open, and required
restoration succeeds.

Changing only Preflight or another downstream runner does not reopen
Readiness or Rehearsal when their dependency fingerprints remain unchanged.
Changing a Readiness or Rehearsal dependency recertifies only the intersecting
lanes unless conservative fallback is required.

## Freeze, SIT, and UAT

Preflight authenticates the compact Readiness and Rehearsal certificates, their
prerequisites, dependency fingerprints, lane coverage, evidence-index hashes,
and current non-impact assessment. It then freezes the exact candidate.

SIT and UAT remain complete candidate-bound phases. If the candidate changes,
run complete SIT and complete UAT for the successor candidate. Do not inherit
SIT lanes or manual UAT scenarios from a different candidate. For an unchanged
candidate, the shared invalidation matrix still permits a lane-local setup
rerun or finalization-only repair when assertions and immutable raw results are
unaffected.

## Evidence bundles

Generated evidence remains under the ignored sprint evidence root. Each phase
attempt publishes one sealed `evidence-index.json` validated against
`phase-evidence-index.schema.json`. Hash artifacts when they are published and
reconcile only that phase attempt before sealing it.

The sprint publishes a compact `evidence-chain.json` containing certificate,
impact-assessment, correction, and phase-index references. Routine downstream
authorization validates those documents and hashes without recursively opening
or rehashing raw artifacts. Raw evidence is cold: read it only for a failure,
challenged certificate, focused diagnosis, or explicit audit.

Closeout performs one final complete integrity audit across the sealed phase
indexes, then records its result in the evidence chain. A missing or altered raw
artifact fails that audit. Do not maintain or repeatedly rebuild one growing
raw-file manifest during every phase transition.

## Impact and invalidation

Validate v2 impact records against
`correction-impact-assessment-v2.schema.json`. For every changed path, record
its domains and affected lanes. For every exclusion, record the unchanged
domain fingerprints and repository evidence supporting non-impact.

- A product correction recertifies affected pre-freeze lanes, then freezes a
  successor and runs complete SIT and UAT.
- A fixture, acceptance, deployment, or environment change recertifies every
  consuming pre-freeze lane and follows the same successor-candidate rule when
  it changes the candidate.
- A phase-local runner change invalidates only that runner's consuming phase
  unless shared interpretation or execution changed.
- An evidence-publication defect with complete immutable raw results reruns
  finalization only.
- A shared validation, security, identity, migration, or cross-module change
  defaults broad unless the tracked contract and fingerprints prove a narrower
  cone.
- An unknown or unauthenticated effect requires complete affected-phase
  execution; after freeze, uncertainty requires a successor candidate with
  complete SIT and UAT.

## Authority and finish criteria

Implementation owns the non-authoritative exit gate. The validation
coordinator owns impact decisions, pre-freeze recertification, freeze authority,
and closeout authorization. Preflight freezes; SIT authorizes UAT; UAT reports
acceptance. Closeout consumes certificates and never originates a test.

Do not authorize closeout unless all canonical phase certificates pass, their
prerequisites and evidence indexes authenticate, the final evidence-chain audit
passes, the candidate-bound SIT and UAT are complete, no defect or product
decision remains open, and the intended topology is healthy.

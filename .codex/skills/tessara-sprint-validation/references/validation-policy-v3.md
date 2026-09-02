# Tessara validation policy v3

Policy v3 is a forward-only extension of
[`validation-policy-v2.md`](validation-policy-v2.md). Read v2 first. A tracked
validation contract activates v3 only with `schema_version: 3` and
`policy_version: tessara-validation-v3`. Retained v2 and legacy contracts and
evidence remain governed by their declared policy and are never converted.

## Mandatory platform boundary

The sprint tracks one schema-v2 `tessara.validation.adapter`; the contract names
its canonical path, platform release `2.0.0`, and
`Invoke-TessaraValidationLane`. Kickoff validates both documents with
`Assert-TessaraValidationAdapter` and
`Assert-TessaraFutureSprintPlanningPackage`. Exact lane parity, prerequisites,
dependency domains, required targets, and proof classes are mandatory.

Sprint-specific commands are adapter actions. Phase scheduling, topology and
ports, cleanup/restoration, evidence publication, certificates, and fail-late
coordination belong to the shared platform. A documented user or architecture
authority is required for an exception. Missing adapter or uncertain provenance
blocks; no fallback is permitted.

## Owned dependency domains

Policy v3 does not use a sprint-wide `product-source` bucket. Each dependency
domain declares its semantic class, actual producer and test inputs, whether it
binds the application candidate, its exact target/lane consumers, and either a
proved bounded cone or a full-replay default. The supported vocabulary includes
module UI shell, authentication/session, navigation, theme/layout, response
owner and consumers, Dataset refresh DAG, provider contracts, migrations/seeds,
deployment materialization, fixtures, acceptance inventory, environment
contract, evidence publication, and phase-local runners. Instantiate only
domains justified by real ownership; consumer inventories must equal the
contract's reverse mappings.

The platform records both the full source fingerprint and an application-
candidate fingerprint derived from candidate-binding domains. A harness or
evidence publisher can therefore change source identity without pretending the
product candidate changed. Unknown paths, unauthenticated mappings, or domains
whose `default_impact` is `full-replay` conservatively select complete formal
replay.

## Post-freeze correction state machine

Use the shared `New-TessaraSuccessorImpactPlan` and
`Assert-TessaraSuccessorImpactPlan`; do not build a sprint-local impact planner.
The immutable plan records predecessor/successor source and candidate
fingerprints, correction diff digest and path/domain mapping, affected
implementation targets, Readiness/Rehearsal/Preflight/SIT lanes, scripted and
manual UAT scenarios, deterministic execution order, inherited coverage,
compatibility/dependency fingerprints, non-impact rationale, cleanup, and
fallback reasons.

- A human execution mistake with unchanged source and evidence semantics reruns
  only the affected manual scenario plus cleanup/finalization.
- An evidence-publication defect with intact immutable raw evidence reruns
  finalization only.
- A phase-local runner defect reruns its self-test, consuming lanes, and their
  prerequisite closure.
- A bounded product correction completes focused repair proof, freezes a
  successor candidate, executes affected authoritative coverage, and may
  inherit authenticated non-impact coverage from the immediate predecessor.
- Unknown, shared-risk, or unauthenticated correction executes complete
  Readiness, Rehearsal, Preflight, SIT, and UAT.

For every product correction, run the failed/highest-risk reproducer first,
then affected implementation targets, converge the complete correction batch,
and only then begin successor certification. Focused proof is not closeout
authority. Within certification, previously failed and directly affected items
precede prerequisite-closure items and expensive low-risk coverage. Receipts
remain deterministic and retained.

SIT/UAT inheritance is authoritative only when the prior pass belongs to the
immediate authenticated predecessor; every dependency, fixture, environment,
acceptance, runner, platform, adapter, and recursive prerequisite inheritance
fingerprint is unchanged; no changed path is unknown; no open defect or
undocumented expectation change exists; and the item consumes no affected
domain. Security, identity, authorization, migrations, shared fixtures,
environment, and cross-module domains default broad unless an exact tracked
domain proves a narrower closed cone.

Phase-certificate schema 3 identifies each item as `executed` or
`inherited_nonimpact`, references the impact plan and predecessor certificate,
and leaves execution timestamps null for inherited items. `sit-result.json` and
`uat-result.json` must preserve that distinction and never imply inherited
assertions ran on the successor.

## Implementation exit

The contract declares explicit canonical-producer to controlled-projection
edges and every slice's exact focused exits. Changed producers require every
projection to advance; unknown fanout expands the target cone. Target receipts
bind current source, contract hash, adapter hash, and platform fingerprint.

Implementation lanes execute through `Invoke-TessaraImplementationHarvest`.
Read [`implementation-target-coordinator.md`](implementation-target-coordinator.md)
before planning or running them. The immutable evidentiary-priority schedule
places corrected failures, never-run targets, affected targets, and authenticated
unchanged targets in that order, subject to prerequisite closure. Safe
independent siblings continue after a failure; dependents and unsafe live-state
targets block. Cleanup/topology risk stops continuation. Every target retains
start/completion receipts, every failure publishes schema-v2
`tessara.validation.defect-provenance`, and those records form one deterministic
defect batch before correction. A passing coordinator finalization receipt is
mandatory before formal Readiness.

Phase 8 additionally requires the complete authorization matrix and early real-
boundary target, followed by standalone UI ownership before consumer cutover.
Fixtures use signed owner read-back under logical keys. Mutable visual content
uses declared stable regions plus semantic assertions; whole-frame comparison
requires invariant content.

Implementation-readiness schema 2 authenticates all of these proofs. Formal
Readiness cannot be their first execution.

## Formal evidence and closeout

Every executed formal lane or scenario executes through
`Invoke-TessaraValidationLane`; inherited items are authenticated by the shared
successor certifier and are never executed or relabeled as successor runs.
Phase-certificate schema 3 and evidence-chain schema 2 bind and authenticate the
current platform identity and adapter hash. Evidence-chain schema 2 lists every
successor impact plan used by a certificate. Structurally valid evidence without
that provenance, an immediate-predecessor link, or complete plan hashes is not
authoritative.

Closeout validates `closeout-efficiency.json` and reports target/attempt count,
first-pass rate, findings by classification and target, repeated hotspots,
implementation-exit gaps, cleanup failures, and formal phases avoided through
implementation discovery. Reusable lessons may improve shared workflow but do
not reopen accepted product behavior.

## Adoption

Future kickoff may select v3 only after the successor-certification, policy,
planning, implementation-coordinator, and complete synthetic platform suites
pass with warnings denied. Start from the v3 contract/adapter assets, replace
every placeholder with actual owned inputs and consumers, validate with
`Assert-TessaraValidationAdapter` and
`Assert-TessaraFutureSprintPlanningPackage`, and track both documents. Do not
upgrade or rewrite a retained v1/v2 contract or receipt.

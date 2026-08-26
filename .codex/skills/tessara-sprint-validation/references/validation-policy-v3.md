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

## Implementation exit

The contract declares explicit canonical-producer to controlled-projection
edges and every slice's exact focused exits. Changed producers require every
projection to advance; unknown fanout expands the target cone. Target receipts
bind current source, contract hash, adapter hash, and platform fingerprint.

Implementation lanes execute through `Invoke-TessaraImplementationHarvest`.
Safe independent siblings continue after a failure; dependents and unsafe
live-state targets block. Cleanup/topology risk stops continuation. Every target
retains a receipt, every failure publishes schema-v2
`tessara.validation.defect-provenance`, and those records form one deterministic
defect batch before correction.

Phase 8 additionally requires the complete authorization matrix and early real-
boundary target, followed by standalone UI ownership before consumer cutover.
Fixtures use signed owner read-back under logical keys. Mutable visual content
uses declared stable regions plus semantic assertions; whole-frame comparison
requires invariant content.

Implementation-readiness schema 2 authenticates all of these proofs. Formal
Readiness cannot be their first execution.

## Formal evidence and closeout

Every formal lane executes through `Invoke-TessaraValidationLane`.
Phase-certificate schema 3 and evidence-chain schema 2 bind and authenticate the
current platform identity and adapter hash. Structurally valid evidence without
that provenance is not authoritative.

Closeout validates `closeout-efficiency.json` and reports target/attempt count,
first-pass rate, findings by classification and target, repeated hotspots,
implementation-exit gaps, cleanup failures, and formal phases avoided through
implementation discovery. Reusable lessons may improve shared workflow but do
not reopen accepted product behavior.

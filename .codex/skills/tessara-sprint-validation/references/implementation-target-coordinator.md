# Shared implementation-target coordinator

Read this reference for every contract-v3 implementation run, recovery, or
formal Readiness handoff. It applies only to validation contracts that select
`tessara-validation-v3`; retained v2 and legacy sprint runners remain historical.

## Ownership and activation

The validation contract selects `Invoke-TessaraImplementationHarvest`,
`evidentiary-priority-v1`, `serial-resource-safe`, and either `execute-all` or
`authenticated-unchanged` reuse. Every implementation target declares its exact
prerequisites and exclusive resource claims. The adapter supplies the exact
command and topology. The shared coordinator owns deterministic ordering,
execution state, recovery, cleanup enforcement, evidence accounting, the defect
batch, and finalization. A sprint must not replace it with an ad hoc `foreach`
loop or sprint runner.

The coordinator publishes an immutable start receipt and schedule digest before
assertions. Each schedule entry records its priority group, rationale,
prerequisite closure, dependency fingerprints, prior receipt and provenance,
command and environment identity, resource claims, and `execute`, `reuse`, or
`blocked` disposition. The order is:

1. corrected prior failures with authenticated classification and passing
   focused reproducers;
2. never-run required targets;
3. dependency-affected, unknown-impact, or reuse-ineligible targets;
4. authenticated unchanged targets; and
5. aggregate finalization.

Prerequisite closure runs before a selected dependent without otherwise
changing deterministic priority/id order. An open, blocked, unclassified, or
uncorrected prior failure is blocked by the provenance gate. Unknown dependency
impact selects conservative execution.

## Reuse

Reuse is allowed only when the contract explicitly selects
`authenticated-unchanged`. Authenticate the prior completion receipt, lane
result and evidence hashes, contract and adapter hashes, platform execution
identity, exact command identity, environment fingerprint, every declared
dependency fingerprint, compatibility identity, and recursive prerequisite
closure. Missing or challenged evidence, an unknown path, changed command,
contract, adapter, environment, dependency, or prerequisite invalidates reuse
and executes the target normally.

A reused completion has `disposition: reused`, `newly_executed: false`, and a
reference to its prior receipt. It is passing current implementation evidence,
but it is never described or counted as a new execution.

## Execution, recovery, and finalization

Execution is serial and resource-safe. This conservative mode prevents
overlapping evidence paths and conflicting process, topology, port, database,
Docker, or external-service claims; it never runs Docker-backed targets
concurrently. Safe independent siblings continue fail-late after a failure.
Dependents and unsafe live-state targets block. Every live-state result must
authenticate passing cleanup/restoration after success, assertion failure,
timeout, or interruption; cleanup uncertainty stops further unsafe work.

Each target retains start and completion receipts plus the platform lane result,
logs, indexes, cleanup outcome, and platform heartbeat evidence. The mutable
checkpoint may advance only under the immutable schedule digest. Recovery
authenticates the start, checkpoint, and completed receipts, then resumes the
same order without relaunching completed work. A changed source, fixture,
contract, environment, command, adapter, dependency, or correction state creates
a new schedule digest and requires a new plan.

After all safe selected siblings terminalize, publish one deterministic defect
batch. Aggregate finalization passes only when every required target is passed
or validly reused, every dependency and receipt authenticates, no target is
blocked or failed, no defect remains open, and required topology restoration
passed. Formal Readiness requires this passing finalization receipt.

The canonical documents are validated by
`implementation-target-state.schema.json`,
`implementation-coordinator-start.schema.json`,
`implementation-target-start.schema.json`,
`implementation-target-completion.schema.json`,
`implementation-coordinator-checkpoint.schema.json`, and
`implementation-coordinator-finalization.schema.json`.

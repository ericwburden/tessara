# Sprint 8B Process-Stabilization Intervention

Formal validation first paused at clean candidate `0a7e245342386851b1648f653d31d12055edd703`
after the armed off-ramp triggered in Candidate Rehearsal `rehearsal-uat`.
The failed attempt is retained at
`artifacts/sprint-8b-closeout/candidate-rehearsal/lanes/rehearsal-uat/attempts/20260820T012216623Z-77adfe59`.
Its schema-valid provenance record classifies the failure as process-origin and
blocks another broad certification run.

After the bounded UAT correction, clean implementation commit
`bf07ae6be38a37f5e567f8f1cc1a1b9b98492c7e` passed Implementation Readiness,
Validation Readiness, Candidate Rehearsal, and Validation Preflight. The user's
next-process-defect rule then triggered a second off-ramp in `sit-browser`.
`sit-rust` had retained a topology after running the mutation-bearing deployed
smoke, while `sit-browser` still declared that inherited topology `fresh`. The
failed attempt and schema-valid provenance are retained at
`artifacts/sprint-8b-closeout/sit/lanes/sit-browser/attempts/20260820T165634011Z-a4916ab0`.
Formal progression is paused again; no broad certification restart is implied by
the bounded correction below.

## Accumulated process defects

| Defect | Classification | Governing requirement | Evidence | Broad invalidation or restart |
|---|---|---|---|---|
| Browser asset readiness | Validation harness/environment defect | `ac-03`, `gate-implementation-exit` | Formal browser trace recorded `ERR_NO_BUFFER_SPACE` for `dataset.css` while hydration reached ready; the retained convergence audit and test-change log record the exact attempt and correction. | Invalidated browser-facing implementation targets and Candidate Rehearsal. Required focused visual proof, exact 95-test proof, rebuilt evidence, a clean implementation commit, fresh Implementation Readiness, fresh Validation Readiness, and Candidate Rehearsal restart. |
| UAT result assembly | Evidence-finalization/harness defect | `gate-implementation-exit` | The earlier formal `rehearsal-uat` completed predicate work but PowerShell case-insensitive variable collision broke result assembly. The test-change log records the canonical assembler and exact 11-scenario/64-assertion self-test. | Invalidated `uat-runner` consumers, Implementation Readiness, Validation Readiness acceptance, and Candidate Rehearsal; contributed to the clean `0a7e2453` restart. |
| Reference gateway lifecycle | Validation harness defect | `gate-implementation-exit`; validation-v2 isolated-live environment/restoration rules | Attempt `20260820T012216623Z-77adfe59` retained four `ECONNREFUSED` failures at `127.0.0.1:63923`, zero product actions, passing cleanup, and no Docker stop/restart/OOM/destroy event before the requests. | Triggers the user-authorized off-ramp after ten passing Candidate Rehearsal lanes. Invalidates the affected implementation/readiness and `rehearsal-uat` evidence; no broad rerun is authorized. |
| Materialized-port handoff | Validation harness defect | `gate-implementation-exit`; validation-v2 source-exact environment identity | Focused receipt `target/sprint-8b-process-stabilization/uat-8b-01-live-v2.json` retained two passing isolated Dataset predicates and a refusal at the parent-reserved `127.0.0.1:57918`; the child materialization receipt independently proves the retained Reference gateway healthy at `127.0.0.1:58023`. Cleanup passed with no retained topology. | Exposed inside the bounded intervention and caused no additional broad invalidation. It proves the parent failed to adopt the child materializer's authenticated environment identity. |
| SIT fresh-state handoff | Validation harness/lifecycle defect | `sit-rust`, `sit-browser`, `gate-implementation-exit`; validation-v2 environment identity | Candidate Rehearsal browser passed the exact 95-test fresh inventory. SIT browser then retained two exact `3 -> 6` row-count failures after `sit-rust` ran the Response outage/recovery refresh; four later serial Dashboard assertions did not run. Emergency cleanup passed. | Triggers the user's next-process-defect off-ramp. Invalidates the current candidate and SIT evidence plus the affected implementation-runner proof; no broad rerun is authorized. |

## Shared root cause

The defects share certification-granularity and lifecycle roots. Broad runners
treated early structural state as authority for later actions: hydration stood
in for stylesheet readiness, database-free assembly checks stood in for the
production aggregation path, and running containers plus an old fixture receipt
stood in for an action-local reachable gateway. Long sequential lanes amplified
the distance between setup and consumption, while evidence finalization occurred
too late to localize failures cheaply.

The SIT defect is the same lifecycle family at a cross-lane boundary. A smoke
workflow that deliberately advances Response/Dataset state was named a
"restoration checkpoint" because it restored process health, even though it did
not restore fixture data. The next lane trusted the retained topology and its
`fresh` label instead of requiring a fresh materialization after that mutation.

The bounded correction does not change acceptance meaning. `uat-sprint-8b.ps1`
now:

1. delays Reference materialization until the first predicate that consumes it;
2. proves exact containers, one-shot gateway HTTP reachability, and the signed
   fixture immediately before every existing-Reference predicate; and
3. assigns every destructive owned-clean predicate a deterministic unique child
   Compose project instead of aliasing the retained Reference project.

The first focused proof also exposed a correction-local parameter-binding
defect before any predicate or topology action: isolated predicates preceding
lazy materialization legitimately have no fixture path, but the child function
rejected the empty value. The canonical topology plan now permits an empty path
only at that neutral invocation boundary; every Reference consumer still
requires and revalidates the signed fixture before execution. This finding
caused no additional broad invalidation and was corrected inside the same
bounded intervention.

The second focused proof reached and passed both isolated Dataset predicates,
then exposed a stale port reservation: `materialize-sprint-8b.ps1` selected and
published a new exact port tuple, while the parent UAT process continued probing
its earlier tuple. The parent now validates the materialization receipt's exact
Compose project and three non-privileged port identities, adopts that tuple, and
updates the Playwright base URL before the action-local health gate. Substituted
project and malformed port receipts fail closed in the database-free self-test.

The next focused invocation reused the same parent evidence directory. The
materializer correctly refused to overwrite its retained child receipt, but it
had already created resources and the parent did not yet consider itself their
owner. The runner now rejects an occupied child-evidence path before launch and
marks topology ownership before invoking the child, so every post-launch failure
enters the exact authorized cleanup path. Focused attempts use distinct retained
directories; no evidence is overwritten.

## Impact and resume rule

Focused proof must cover the database-free scheduling/finalization self-test and
one source-exact `UAT-8B-01` live implementation run. The validation-policy impact
calculation determines the affected implementation targets and formal lanes.
No product source, fixture meaning, browser assertion, baseline, retry, timeout,
or acceptance contract may change in this intervention.

For the second off-ramp, `sprint-8b-evidence-runner.ps1` keeps the full smoke as
the canonical outage/recovery proof, then performs an exact teardown followed by
a new source-exact `Reference -KeepTopology` materialization before publishing
the SIT browser handoff. Its SIT self-test requires the exact setup, Rust, smoke,
teardown, fresh-handoff sequence and rejects omission or reordering. Focused live
proof must execute that lifecycle without the workspace-wide Rust step and then
run the two unchanged assertions that previously observed six rows. Broad SIT or
any other formal phase remains blocked after that focused proof.

Broad certification remains blocked after focused proof. The minimum-safe resume
boundary will be reported from authenticated impact evidence and requires explicit
user authorization; no Preflight, SIT, or UAT phase may start from this record.

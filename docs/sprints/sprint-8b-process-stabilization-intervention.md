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
| Retained defect-provenance chronology | Evidence-finalization/validation-platform defect | `gate-implementation-exit`; validation-v2 defect-provenance completion and immutable-evidence rules | Hardened audit finds 11 retained records: 10 schema-valid, one schema-invalid, zero fully verified, zero validly superseded, and 11 unresolved. The historical invalid record keeps SHA-256 `e16b7bb42636d3910cdaf7ea3ca8c57ff5ad76096fba0383c8d93a85e611a613`; older records also expose moved/mismatched evidence and conflicting proof maps. | Triggers this process-only correction and blocks aggregate/final phase publication. The current contract still selects all 24 implementation targets through `validation-shared`; focused proof cannot silently narrow that impact or authorize a broad restart. |

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

The chronology defect is the evidence-finalization form of the same pattern:
individual result/schema checks were treated as sufficient even though no
single boundary authenticated the complete retained chronology immediately
around publication. Superseded-directory moves also broke the repository paths
claimed by older records. The correction makes those failures visible; it does
not reinterpret or repair historical bytes.

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

Broad certification remains blocked until focused proof establishes and
authenticates the minimum-safe resume boundary. Once that boundary is established,
the validation coordinator resumes automatically from it; a separate user
authorization pause is not required. No failed attempt is retried or converted
to a pass, and no later phase may start before its ordinary prerequisite
certificates are valid.

## Retained defect-provenance chronology intervention

A further evidence-finalization defect was discovered while auditing retained
Sprint 8B state. The superseded Candidate Rehearsal `rehearsal-browser` record at
`artifacts/sprint-8b-closeout/candidate-rehearsal-superseded-20836b8-20260820T001522Z/lanes/rehearsal-browser/attempts/20260819T185314991Z-1351ad69/defect-provenance.json`
claims `verified`, but does not satisfy the unchanged
`defect-provenance.schema.json`. It contains forbidden finding fields, a
noncanonical focused-evidence collection, and the retired correction shape. The
retained record is immutable evidence: it must not be rewritten, deleted, or
relabelled to obtain a pass.

The bounded correction adds one canonical chronology gate owned by the shared
validation policy. Implementation-readiness finalization and formal lane/phase
publication consume that same gate. Every retained Sprint record must be either
schema-valid and `verified`, or referenced by exact repository-relative path and
SHA-256 from one later, schema-valid, `verified` superseding record. The gate
rejects open, classified, corrected, blocked, foreign-sprint, invalid,
self-referencing, dangling, hash-mismatched, duplicate-ID, and ambiguously
superseded records. It also authenticates every claimed evidence reference,
requires the complete focused/implementation proof inventory and clean source
behind `verified`, preserves exact RFC 3339 chronology, rejects linked or
traversal-bearing evidence paths, and rechecks retained bytes and inventory
before returning. A `.sha256` sidecar, where present, must authenticate the
exact retained bytes. The historical invalid record remains byte-for-byte
unchanged; the correction incident may resolve it only by publishing a new
schema-valid record that reaches `verified` after focused proof **and every
implementation target selected by the current dependency contract**, and binds
that exact old path and hash.

This intervention changes no product behavior, fixture meaning, acceptance
assertion, snapshot or other baseline, retry policy, or timeout. Proof is limited
to the application-free synthetic chronology suite and the directly affected
runner/finalization boundary checks. The current Sprint 8B dependency contract
continues to determine the full impact cone; it is not narrowed to make the
correction pass. Another broad Implementation Readiness, Validation Readiness,
Candidate Rehearsal, Preflight, SIT, or UAT run remains blocked pending the
authenticated minimum-safe boundary. After the process-only correction and its
affected implementation targets pass, progression resumes automatically at that
boundary without a separate authorization request.

The runner-owned checks and transactional cleanup now close every deterministic
publication window exercised by the synthetic suite. Absolute exclusion of an
independent provenance writer still requires the deferred validation-platform
lease/lock protocol; Sprint 8B must not claim that stronger guarantee from
caller-local rechecks alone.

## Current off-ramp rule

Effective 2026-08-21, a newly discovered process-origin defect still triggers
the Sprint 8B off-ramp immediately. The failed attempt and provenance remain
immutable; only safe fail-late evidence collection, cleanup, and restoration may
continue. The correction must remain process-only, preserve every acceptance
assertion, fixture meaning, baseline, retry policy, and timeout, and pass focused
harness proof plus every implementation target selected by authenticated impact
analysis.

The off-ramp is a correction-and-resume mechanism, not a user-approval stop.
After the correction establishes the minimum-safe resume boundary, the
coordinator continues from that boundary automatically. Broad phases are never
opportunistically restarted, and an application-candidate change still requires
the normal candidate-affecting implementation and certification cone. A
process-only correction invalidates only the validation-platform evidence,
affected sprint adapter, affected formal lanes, and narrow integration checks
identified by the current contract.

## Deferred post-Sprint-8B validation-platform separation

The architectural separation is explicitly deferred until Sprint 8B has passed
its authorized closeout. It is not part of the Sprint 8B candidate, and no
roadmap entry changes during this intervention. Immediately after closeout, the
work must first be recorded in the appropriate roadmap/planning artifact, then
implemented as the first validation-platform work before the next application
sprint proceeds.

The target has three explicit owners:

1. **Application acceptance contract** owns product scenarios, expected
   behavior, fixtures, assertions, and application-owned tests. Changes remain
   candidate-affecting.
2. **Validation platform** owns scheduling and state transitions, child-process
   execution, Compose and port lifecycle, health/readiness synchronization,
   locks, checkpoints, cleanup/restoration/recovery, and immutable evidence
   writing, hashing, indexing, and certificate assembly.
3. **Thin sprint adapter** declaratively maps sprint scenarios to commands,
   topology requirements, and evidence contracts; it owns no lifecycle or
   evidence machinery.

The first extraction point is `scripts/uat-sprint-8b.ps1`, whose acceptance,
topology, process, fixture, and publication responsibilities must be separated
behind one canonical validation-platform boundary. The application candidate,
acceptance contract, and validation-platform release each receive an independent
fingerprint. A synthetic platform-certification suite using fake or minimal
services must prove lifecycle, port handoff, interruption/failure containment,
cleanup/restoration, and evidence finalization without running Tessara product
tests. Impact mapping must then distinguish harness-only, acceptance-contract,
and application changes precisely: process-only corrections invalidate the
platform certificate, affected adapter, affected formal lanes, and narrow
integration boundaries without automatically invalidating unrelated application
targets; acceptance or application changes continue to select their full
required cones.

The migration is forward-only. It must leave one runner path, no compatibility
layer, no duplicate lifecycle implementation, and no superseded machinery in
the touched cone. Handoff requires synthetic certification, selection proofs for
all three change classes, immutable failed/interrupted evidence, restored
topology, and explicit proof that no assertion, fixture meaning, retry policy,
timeout, or baseline was weakened.

## 2026-08-21 browser-process off-ramp

Candidate Rehearsal on clean source `6970a678cea5fea5208a898156a4de11ff6ba6b7`
passed static, Rust, and source-exact materialization before the browser lane
failed after 76 of the 81 functional identities had passed. The exact failure
was a browser-origin `net::ERR_NO_BUFFER_SPACE` while navigating to
`/administration/modules`; one identity failed and four were safely not run.
No product assertion failed before the resource error. The lane retained its
trace and failure receipt, and emergency teardown removed the exact owned
Compose topology successfully.

This is a validation harness/environment defect governed by `ac-03` and
`gate-implementation-exit`, not a Tessara product or Reference-fixture defect.
The retained incident index is
`artifacts/sprint-8b-closeout/process-incidents/20260821T225052881Z-browser-socket-buffer/evidence-index.json`
with SHA-256
`82d33f491fdaa4be7af68fe8dfdf7a762f2010fbce96166c7c217b6edb780803`;
the immutable lane failure is
`0970008a8cbb4b5cd3f5c2f389686a3d06a016a86bafafce77cb3bd4ee31f010`
and the successful emergency teardown is
`6d350f905824116dca74b3096dc74919b7d05f6ca5e58c19831c9ccf551c2732`.
The socket snapshot also retained substantial unrelated host socket ownership,
so the correction must bound the validation process rather than mutate user
applications or host networking.

The earlier functional/visual split reduced one 95-test process to an 81-test
functional process and a 14-test visual process, but this failure proves the
remaining functional lifecycle is still too broad. The bounded forward-only
correction replaces that partition with one manifest-driven runner that starts
a fresh Playwright/Chromium process for each of the 11 unchanged acceptance
files. It keeps one worker, zero retries, comparison-only snapshots, the exact
fresh Reference data state, and all 95 existing identities. It continues
fail-late between files and retains each file's JSON, JUnit, command log, and
failure traces before authenticating the exact union. A synthetic local HTTP
probe certifies all 11 browser-process start/navigation/disconnect lifecycles
without executing Tessara application behavior.

No test identity or expectation changes in this correction, so it does not add
an acceptance-inventory entry to the Sprint test-change log. This process
incident and its execution-only correction are recorded here under validation-
platform ownership; any future assertion change remains subject to the normal
test-change-log authority and application impact cone.

The minimum-safe automatic resume boundary is the failed `rehearsal-browser`
lane, after the synthetic platform proof, both contract-selected implementation
targets, clean committed source, and a verified defect-provenance record pass.
The old failed attempt is never retried or rewritten. If current impact policy
requires an earlier certificate because the validation-platform source identity
changed, that authenticated boundary takes precedence automatically; no
separate user authorization pause is required.

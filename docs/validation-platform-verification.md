# Validation Platform Foundation Verification

Status: implementation verification complete for platform release `2.0.0`;
independent diversion closeout is pending the clean source-exact full gate. This
record is non-authoritative
implementation evidence; it does not
replace Validation Readiness, Candidate Rehearsal, SIT, UAT, or sprint
closeout, and it does not close the residual gates below.

The independent closeout scope is now machine-readable in
[`sprint-0a-validation-platform-foundation-validation-contract.json`](./sprints/sprint-0a-validation-platform-foundation-validation-contract.json).
It maps the complete diversion cone to seven exact implementation targets and
five lifecycle lanes (including two pre-freeze lanes). No Sprint 8C path,
adapter, or evidence is part of that contract.

## Independent closeout readiness

The tracked closeout contract validates under `tessara-validation-v2`; its
current SHA-256 is
`9b733bcd7e164f9918e7f2141d1c9d18b37ffcdf7897e21772b8905f588187a5`.
The current diagnostic implementation proofs are:

| Exact target | Current diagnostic state |
|---|---|
| `cargo-policy-selftest` | Passed warning-free outside the filesystem sandbox |
| `validation-policy-selftest` | Passed |
| `validation-runner-selftest` | Passed warning-free outside the filesystem sandbox |
| `markdown-link-check` | Passed after closeout-contract publication |
| `platform-synthetic-certification` | Passed the complete release-2 fault matrix |
| `platform-live-docker-certification` | Passed on Docker 29.6.1 with zero residue |
| `full-repository-validation` | First diagnostic stopped before product actions on a live-PostgreSQL boolean-shape harness defect; the focused live reproducer and warning-free runner self-test now pass, and the broad rerun remains blocked until the correction is committed in the clean diversion candidate |

These are implementation diagnostics, not phase certificates. The failed
database-backed diagnostic and its corrected provenance chronology are retained
under
`artifacts/validation-platform-closeout/diagnostic-full-20260823T182633212Z/`.
The live PostgreSQL probe exposed that the harness cast booleans to
`true`/`false` while comparing them to `t`/`f`; no product action started. The
query now normalizes all nine boolean fields explicitly, and both the focused
live probe and the warning-free runner self-test pass. Candidate freeze remains
blocked until this correction and the rest of the diversion are committed
cleanly and the complete database-backed target passes on that exact source.
No formal lifecycle attempt has started.

## Requirement-to-proof mapping

| Governing requirement | Implementation owner | Focused proof |
|---|---|---|
| Separate provenance, compatibility, execution-attempt, and finalization identities | Platform manifest, public entry point, lifecycle, and finalizer | `aggregate-identity`; exact four-boundary/four-component inventory; independently computed execution and finalization fingerprints; lane results retain aggregate provenance and raw execution identity |
| Source-authenticated candidate binding | Public planner, lifecycle, and strict phase certifier | Candidate identity is derived from Git commit/tree/dirty state, governing contract, and current dependency fingerprints; candidate-bound lanes reject caller mismatch, and phase certification rejects dirty or mismatched candidate plans |
| Lane-scoped minimum-safe invalidation | Canonical lane compatibility construction | Contract slice, dependency domains, acceptance, fixture, harness, adapter, environment, observed tools, platform execution, and prerequisite compatibility are hashed per lane; `lane-scoped-invalidation` proves the declared-environment case, while `planner-mutation-matrix` proves contract-slice, acceptance, fixture, harness, lane-adapter, and dependency-domain changes affect the owner and dependent but not an independent lane |
| Exact directly declared executable identity | Platform planner plus lifecycle recheck | Compatibility binds resolved path/content observations for directly declared programs and the Docker prefix; the lifecycle re-observes them before attempt start and every action/topology/Docker invocation; `per-invocation-tool-substitution-rejection` proves setup cannot swap the later directly declared topology/cleanup tool or acquire ownership, but wrappers' transitive tools are not inventoried |
| Declared source-byte recheck through publication | Lifecycle post-cleanup and post-checkpoint integrity gates | The lifecycle re-hashes lane-owned acceptance/fixture/harness and command inputs plus declared dependency domains before and immediately after checkpoint creation; only a successful second gate publishes an authenticated positive integrity pair required by the finalizer; `execution-input-drift-rejection` proves both checkpoint-recorded and post-checkpoint-only drift cannot gain that commit or publish result/index even when negative revocation publication fails and the input is restored. This proves stability of resolved bytes, not an OS-native symlink-proof source cone: on Unix the current ancestor guard can skip a literal-backslash symlink name |
| Versioned Cargo build lifecycle | `tessara-cargo-build-policy.psm1` | `test-tessara-cargo-build-policy.ps1 -SelfTest` and aggregate component hash verification |
| One narrow validation-platform boundary | `tessara-validation-platform.psm1` | Module exports identity, candidate identity, compatibility planning, adapter validation, and lane invocation; lifecycle internals remain private |
| Canonical governing contract and declarative sprint adapters | Validation-policy v2 validator plus adapter v2 schema | Policy and platform self-tests cover canonical contract validation, exact prerequisites, unknown fields/tokens, duplicate actions, budgets, port/handoff equality, interpreter bypasses, cycles, and invalid inputs before attempt creation; every assertion must exactly map a governing implementation target, structured command, proof classes, and declared transitive tools, with complete required-target coverage |
| Exact runtime prerequisites | Private lifecycle prerequisite gate | A missing result blocks before assertions; a canonical authoritative compatible passing result permits the dependent lane; result hash and compatibility identity are retained |
| Filtered child environment | Environment-plan builder | `hermetic-environment`, reserved-base collision rejection, declared-source observation, exact caller-environment preservation, missing/blank source setup failures, retained-artifact secret scans, and a two-character noisy-secret regression prove the implemented filtering/redaction boundary; poisoned caller `GIT_DIR` and `GIT_INDEX_FILE` additionally prove platform Git runs with isolated control variables. `TEMP`/`TMP`/`TMPDIR` are replaced with the attempt-owned temp path. This is not an OS sandbox: the caller token and base `PATH`/`PSModulePath` remain available |
| Setup is contained around topology ownership | Ordered lifecycle staging | Missing/blank environment and ordinary setup failures run no assertion, acquire no topology ownership, and close temporary port leases; a consumer's read-only source/input/Compose checks complete before claim publication and leave the receipt reusable on rejection; after a complete claim commits, a failing consumer setup action retains the authenticated claim, starts no assertion, and destroys the transferred topology through cleanup |
| Process execution and cooperative failure containment | Private lifecycle component | Every OS-started direct child immediately enters truthful platform cleanup ownership; redirected stdin closes before successful helper return, and a post-start close/setup/handle-transfer failure runs stop/dispose cleanup and reports incomplete cleanup before propagating. `post-start-process-acquisition-containment` injects action and local-topology acquisition faults, while local success, exact child exit `17`, timeout, cancellation, direct-child and attached-descendant teardown, and closed-port proof cover normal execution. Tool-specific noninteractive invocation, OS job/process-group containment, and deliberately daemonized descendants remain explicit residuals |
| Concurrent namespace isolation | Private lane leases | Two concurrent independent lane processes retain distinct attempts and ports; this proves namespace separation, not scheduler safety, concurrency limits, or host resource quotas |
| Authenticated single-use topology transfer | Docker Compose provider | Capability-bearing canonical receipt binds source/target identities, candidate, evidence root, configuration, project, and ports; a complete create-once claim rejects same-root replay and root binding rejects copied-root replay. `complete-claim-cleanup-ownership` proves a returned complete consumer claim immediately implies cleanup ownership even if later bookkeeping faults. The source retains cleanup ownership through successful receipt/result/index finalization. Positive-integrity creation, finalization, and post-commit transfer-flag clearing share one guarded region; `post-finalizer-handoff-transfer-containment` proves a fault before transfer completes commits revocation before teardown and that consumers reject the revoked receipt. Revocation failure retains topology; absence of both integrity members makes the checkpoint ineligible. `partial-claim-publication-containment` injects failure after consumer claim-data creation and proves pair rollback, teardown, and a sequential producer/consumer retry; overlapping claim recreation before the first teardown is not yet serialized |
| Cleanup and caller-state preservation | Provider cleanup plus child-only environment construction | Success, child failure, timeout, interruption, foreign-project rejection, final teardown, residue rejection, and exact caller sentinel preservation |
| Platform-write-once hashed evidence and bounded finalization-only recovery | Lifecycle checkpoint plus independent finalizer | Create-once SHA-256 pairs; required positive post-checkpoint integrity commit; checkpoint artifact, complete prerequisite-commit, topology-claim, and retained-receipt authentication; lock-held exact ordinary-file inventory with 256-file, 64-directory, eight-level, 512-relative-character, 16-MiB-per-file, and 64-MiB aggregate limits; rejection of absent/partial/substituted integrity commits, late unlisted files, orphan sidecars, oversized files, directory/path overbudget, reparse entries, and revocation observed at the implemented gates; stable result/index pairs plus append-only fingerprint-keyed finalization attestations; recovery rechecks current lane inputs and requires the already committed positive pair rather than minting one from absence; caught write/flush/dispose rollback at all six result/index/attestation data and sidecar publication boundaries; same-checkpoint, same-fingerprint recovery with complete older attestations retained; consumer authentication of the index, hashed canonical checkpoint, and current attestation; idempotent re-finalization; and unchanged action-log/index hashes in the certified cooperative cases. An older-fingerprint data-only partial blocks a corrected finalizer. The finalizer repeats every checkpoint artifact digest after its late prerequisite rechecks and immediately before its final integrity/revocation/attestation checks; cross-root serialization through the attestation-sidecar commit remains residual |
| Compatibility-aware phase certification | Validation policy and `phase-certificate-v2.schema.json` | The generic structural validator remains fail-closed; `New-TessaraPlatformPhaseCertificate` publishes a write-once plan/index/certificate and the strict validator recomputes the plan and authenticates source, environment, results, attempt indexes, current finalizer attestations, target coverage, prerequisites, and permitted pre-freeze inheritance |
| Application-independent certification | Synthetic adapter, local service, and Docker CLI shim | Certification reports `application_suites_executed: false`, emits a separate fingerprint over the certification script plus the exact eight fixture files, rechecks that snapshot before its passing receipt, and requires no database, external network, or live Docker |
| Real Docker provider boundary | Digest-pinned live provider fixture | Docker 29.6.1 producer/consumer certification authenticates daemon/context/runtime image identity, challenge readiness, single-use handoff, teardown, and zero residue; later product adapters repeat it on their own sprint evidence and do not participate in diversion closeout |

## Exact implementation checks

Run from a clean checkout:

```powershell
pwsh -NoProfile -File .\scripts\test-tessara-cargo-build-policy.ps1 -SelfTest
pwsh -NoProfile -File .\scripts\test-tessara-validation-policy.ps1 -SelfTest
pwsh -NoProfile -File .\scripts\test-tessara-validation-platform.ps1 -SelfTest
pwsh -NoProfile -File .\scripts\test-tessara-validation-platform-live-docker.ps1
pwsh -NoProfile -File .\scripts\validate.ps1
git diff --check
git status --short
```

The platform certification is required to report `state: passed`, an exact
three-input public-boundary inventory, an exact four-component platform
inventory, independently valid execution/finalization fingerprints, a separate
certification-harness fingerprint over the certification script and exact eight
fixtures, and `application_suites_executed: false`. The harness inventory and
hashes are snapshotted before module import, rechecked before the passing
receipt, and emitted in that receipt. Its certified set includes prerequisite
gating, the lane-scoped planner mutation matrix, isolated Git behavior under
poisoned caller control variables, filtered child environment and
caller-environment preservation, positive post-checkpoint integrity commitment
and rejection after execution-input drift or failed negative-marker publication,
complete-claim Compose replay rejection in the certified sequential cases, and
pre-attempt rejection of an over-budget topology or mismatched producer/consumer
port set. A short-secret noisy-output regression
forces both captured streams past their raw limit and proves each stored,
post-redaction log remains at most 1 MiB while no retained artifact contains the
secret. Consumer staging regressions distinguish read-only preclaim rejection,
which preserves reuse, from a setup-action failure after a complete claim, which
retains that claim, starts no assertion, and tears down the topology.
The suite also rejects external finalization actions and assertion-free
no-topology lanes and proves exact bounded attempt inventory plus
platform-write-once same-fingerprint finalization-only recovery. The inventory
regressions create only disposable files under the synthetic attempt: a late
extra file, an orphan sidecar, an oversized sparse file, excessive directory
count/depth/path length, and an
in-root reparse entry targeting another disposable directory. A
post-attestation regression removes only the disposable current-attestation
commit sidecar, injects a late file, proves finalization cannot recreate the
sidecar, and proves a dependent lane starts no assertions until clean
finalization-only recovery restores that exact commit pair. Additional
disposable mutations prove that a digest-tampered or stale-finalizer attestation
cannot start prerequisite assertions or acquire a topology handoff claim. A
preexisting `EvidenceRoot/lanes` junction targets only a disposable sibling
directory and proves rejection creates no file beyond that owned root.
Deterministic final-commit injections then revoke a prerequisite or remove its
index sidecar after dependent execution; neither dependent attestation commits,
and restoring only the prerequisite evidence permits finalization-only recovery
with unchanged dependent action logs.
The finalizer writer proof separately injects caught failure at each of the six
result, index, and attestation data/sidecar publication boundaries. Each case
removes only files created by that invocation and successfully reuses the same
checkpoint; it does not simulate an abrupt host or machine termination.
The validation-policy self-test separately proves the v2 structure and that the
generic historical-certificate path rejects v2 rather than treating fabricated
compatibility hashes as reuse authority.
The full validation gate remains the broader repository regression proof. A
synthetic receipt proves platform mechanics and exact synthetic targets, not
Tessara product behavior; candidate authority begins only with a clean frozen
source and the active sprint's exact adapter.

## Implementation diagnostic history

The first disposable full-gate attempt reached `tessara-api --lib` with 124
tests passed before one SQLx setup connection timed out. Failure cleanup then
exposed a validation-platform defect: recursive `validate.ps1 -SelfTest`
module loading used `Import-Module -Force`, which replaced the Cargo policy
module and discarded its private active lease. The correction preserves an
already-loaded exact module identity in both the full gate and aggregate
platform entry point.

A higher-capacity disposable PostgreSQL rerun reproduced the same one-test
timeout and established the underlying integrity gap: `#[sqlx::test]` reads
`DATABASE_URL`, while the full gate had validated dedicated URLs without
binding that variable for the SQLx-backed steps. SQLx therefore loaded the
repository `.env` target instead of a preflight-approved disposable database.
Full validation now keeps a deny sentinel in `DATABASE_URL`, substitutes
`TEST_SQLX_DATABASE_URL` only for the exact API SQLx test and Dataset targets
that contain SQLx tests, then restores the caller's prior present-or-absent
state in `finally`; the database-free self-test proves scoped substitution and
restoration. The platform adapter path additionally
fingerprints declared source-environment observations, supplies child programs
only a filtered environment, and rejects missing or blank required values
during setup before topology ownership or assertions. Neither correction
changes an application assertion, timeout, retry, or expected result.

The then-current source-exact full gate passed all checks, including all 125
API library tests, the database-backed integration targets, and the release
resource-reference timing proof. That pass is diagnostic history, not proof of
later uncommitted platform changes; the exact checks above must pass again for
the current tree.

## Explicit residual gates

Candidate identity, exact target/action/proof mapping, and strict plan-backed v2
certification are delivered. The retained validation-readiness publication proves
the bridge on a dirty pre-freeze tree and is correctly non-authoritative. A
candidate-bound certificate still requires a clean frozen source. Diversion
closeout proves the platform release independently; product-adapter adoption is
a separate downstream lifecycle.

The child environment filter is not an operating-system sandbox. Every child
runs with the caller's OS access token and therefore inherits that token's
filesystem, network, process, and Docker-daemon authority. `PATH` is rebuilt from
declared executable/tool directories, `PSModulePath` is platform-owned, and
declared transitive tools are content-bound and rechecked. `TEMP`, `TMP`, and
`TMPDIR` are attempt-owned. A wrapper-started undeclared runtime remains outside
authority. Completion requires a least-authority OS boundary and adversarial
escape proofs. The temp tree still needs the execution-
time storage limit described with scheduler/resource controls below.

The synthetic interruption proof is cooperative cancellation inside the
platform host. Normal cancellation and timeout cleanup reaches the direct child
and attached descendants, and the direct child's redirected stdin is closed.
The start helper now marks start truth immediately and runs stop/dispose cleanup
when post-start stdin or handle publication fails; action execution enters its
outer cleanup guard before acquisition and retains truthful assertion-start
state. Focused action and local-topology injections prove these acquisition
faults leave no child or port residue. Direct PowerShell actions require
`-NonInteractive`, but execution does not contain a deliberately daemonized or detached
descendant in an OS job/process group. A current hard-crash probe is diagnostic-
only because a killed host cannot publish authoritative cleanup or result
evidence. Completion requires job/cgroup ownership or an independently running
capability-authenticated orphan-discovery and cleanup protocol, plus disposable
detached-descendant and adversarial forged/copied-state proofs. Release `2.0.0`
does not yet claim that capability, and an incomplete checkpoint cannot be
promoted through finalization-only recovery.

TCP readiness uses a per-attempt challenge-response capability and therefore
does not accept an unrelated listener that merely wins the released port.

Repeated resolved-input byte checks detect mutation of the observed targets and
fail closed, but the platform still executes from a mutable working tree. It
does not provide an immutable snapshot or prevent a process from changing inputs
between guarded reads. A source-exact read-only worktree/container boundary
remains a stronger future containment option.

The source-path ancestor guard also splits relative paths on both slash and
backslash. On Unix, backslash is a literal filename character, so a repository
directory with a backslash in its name can be a symlink that the guard fails to
inspect while later file hashing follows it outside the repository. Until the
planner and lifecycle use OS-native segmentation and pass an adversarial Unix
backslash-name/symlink escape proof, their byte hashes are not source-cone
authority for selective reuse.

The Docker CLI shim proves exact command grammar and fault injection without a
live engine. Release 2 additionally rejects custom/external resources, bind
mounts, host namespaces, privileged/capability/device grants, secrets/configs,
builds, mutable images, and implicit environment; it binds daemon/context,
digest image, and actual running container identities. The retained live-Docker
producer/consumer proof passed challenge readiness, handoff, teardown, and zero
residue. Each product adapter must repeat the adversarial gate for its exact
Compose model; that focused gate does not require unrelated lanes to restart.

A stale or incompatible handoff is rejected before a consumer claim, so the
consumer intentionally lacks authority to destroy it. If the producer is no
longer available, cleanup therefore depends on the residual authenticated
orphan protocol rather than broadening consumer permissions.

The two-process concurrency proof establishes separate namespaces, not a safe
global scheduler. There is no platform-wide admission control, total lane
deadline across all actions, concurrency cap, or CPU, memory, disk, process,
port, Docker, or temporary-storage quota. There is also no evidence retention,
quota, garbage-collection, sensitive-artifact disposal, or archival policy for
completed and failed attempts. The synthetic secret scan is not a general DLP
boundary for files a real action writes under its attempt. Those controls and
pressure/failure proofs remain required before unattended or parallel use can be
described as bounded.

Create-once platform publication and SHA-256 sidecars are not filesystem
immutability against another process with the caller's token. The finalizer now
repeats every checkpoint artifact digest after its late prerequisite rechecks
and immediately before its own final integrity, revocation, and attestation
checks. That is still not serialization: a detached or same-token writer can
mutate an artifact after the digest pass, and a prerequisite can be revoked
after its late recheck but before the dependent attestation sidecar commits.
Completion requires locks that cover the entire artifact and recursive
prerequisite closure through the final commit, OS writer containment plus an
immutable snapshot, or an equivalent capability boundary, with deterministic
same-path mutation and late-revocation proofs. Until then the current attestation
is not adversarial concurrent-writer proof.

Partial topology-claim publication deletes its incomplete pair before the
consumer that created the data finishes teardown. The retained local ownership
ensures that consumer cleans up, but the now-absent path can be recreated by an
overlapping consumer, which can then gain a claim to topology the first consumer
is about to destroy. The current proof covers only rollback followed by a
sequential retry. A durable claim tombstone or serialization across claim and
cleanup, plus a deterministic overlapping-consumer proof, remains required for
race-safe single use.

A finalizer can recover its own data-only current attestation, and a corrected
finalizer can coexist with complete older-fingerprint attestations. It cannot
recover when an older finalizer wrote attestation data but failed before its
sidecar: that data is treated as an incomplete prior attestation and blocks the
new fingerprint. A deterministic old-partial/new-finalizer proof and a protocol
that safely authenticates and resolves that state remain required. Until then,
rerun the affected lane rather than manually altering retained evidence. The
caught same-fingerprint writer rollback does not close this older-fingerprint or
hard-termination case.

The schema-v2 strict certifier now authenticates the current source/dependency
snapshot, exact target coverage, compatibility plan, committed lane results,
current finalizer attestations, prerequisite closure, and evidence index. The
generic validator still rejects structural-only v2 documents. Selective product
reuse begins only after the active sprint executes this bridge with its exact
adapter and clean frozen candidate.

Checkpoint pairs and their indexes authenticate exact local bytes and current
platform relationships, but they do not yet establish signed checkpoint origin
across hosts or an independently trusted provenance chain. That provenance must
be bound by the real phase certifier before evidence is portable beyond its
owned local root.

Execution identity has common, local-process, Docker Compose, and orchestration
capability fingerprints; each lane binds the provider component it executes.

## Historical boundary

Sprint 8B is closed. Its runners, policy-v2 contracts, schema-v1 phase
certificates, and sealed evidence retain their original implementation
identity. They are not a consumer of platform release `2.0.0` and are not
retrofitted with compatibility fingerprints. Platform-backed future phase
certificates use schema v2. Downstream sprint adoption starts only after this
diversion has closed and produces a separate evidence chain.

# Validation Platform Ownership

Status: platform release `2.0.0` implementation complete and awaiting independent
diversion closeout. The
source-derived candidate identity, exact governed target/action/proof mapping,
provider-scoped execution identity, strict compatibility-plan phase certifier,
and a digest-pinned live-Docker provider proof are delivered. The residual
limits below remain fail-closed adoption constraints: caller-token and detached-
process containment, immutable source execution, OS-native Unix path escape,
hard-crash/orphan recovery, cross-finalizer partial-attestation recovery,
cross-root commit serialization, scheduler/resource retention policy, and the
downstream product adapters.

Tessara validation has three explicit owners. The boundaries below keep a
process-only correction from invalidating unrelated application behavior while
preserving the full application cone whenever assertions, fixtures, contracts,
or product interfaces change.

## Owners

### Application acceptance contract

The application owns product assertions, scenarios, expected behavior,
fixtures, and application tests. Product source changes affect the candidate;
acceptance and fixture changes affect their owning lane identities. None is
reclassified as a platform-only change.

### Validation platform

The validation platform owns process execution, topology and port lifecycle,
readiness synchronization, locks and checkpoints, cleanup and restoration,
lane-evidence publication, hashing, indexes, and the compatibility fields used
by phase-certificate assembly. Platform behavior has an independently
versioned release identity and synthetic tests that do not execute Tessara
application behavior.

### Sprint adapter

A sprint adapter declaratively maps scenarios to commands, topology
requirements, prerequisites, and evidence contracts. It does not implement its
own process, cleanup, topology, or evidence machinery.

## Compatibility, attempt, and finalization identities

The aggregate application, acceptance, adapter, and platform hashes remain
useful provenance, but an aggregate hash is not the equality test for reusing
one lane. The platform keeps the following identities distinct:

| Identity | Bound inputs | Purpose |
|---|---|---|
| Application candidate | Platform-derived Git commit/tree/dirty-state, governing contract, and current dependency snapshot fingerprint | Binds Validation Preflight, SIT, and UAT; the strict certifier rejects dirty candidate-bound plans or a caller mismatch |
| Aggregate platform | Manifest, four public boundary inputs, and all four platform components | Complete release inventory and provenance; not by itself a lane-restart key |
| Platform execution | Public entry point, adapter schema, governing validation-contract schema, validation-policy module, and lifecycle module | Invalidates every lane when common execution semantics change |
| Lane compatibility | The exact tuple below | Determines whether a prior lane result still describes the current lane |
| Raw execution attempt | Lane compatibility plus attempt-specific runtime observations and resources | Distinguishes one actual execution from another compatible execution |
| Finalization | Platform-write-once execution checkpoint, positive post-checkpoint integrity commit, and finalizer component identity | Permits evidence-publication recovery without rerunning completed actions |

The lane compatibility tuple is the canonical hash of:

- lane ID and phase;
- platform-derived candidate fingerprint for Validation Preflight, SIT, and UAT,
  or a null compatibility binding for Validation Readiness and Candidate
  Rehearsal;
- the governing `tessara-validation-v2` contract slice for that lane, including
  the relevant dependency domains, requirements, and implementation targets;
- current hashes for the lane's declared dependency domains;
- lane-scoped acceptance, fixture, execution-harness, and adapter fingerprints;
- the lane's declared environment/port contract and its observed platform-base
  and declared source-environment values, represented by hashes rather than
  retained secret values;
- resolved path/content observations for each directly declared external
  program and the Docker command prefix, also represented by fingerprints and
  re-observed by the lifecycle immediately before execution;
- the platform execution fingerprint; and
- the compatibility fingerprints of every direct prerequisite, recursively.

The raw execution fingerprint then adds the attempt ID and evidence-root
identity, candidate, realized filtered-environment fingerprint, resolved
tool bindings, exact prerequisite-result hashes, and applicable Compose
project/configuration, capability hash, and port values. It identifies what
actually ran; it is deliberately more specific than compatibility.

Finalization verifies internal equality of the supplied candidate label, lane
acceptance/adapter/compatibility identity, platform execution identity, attempt
intent/start records, checkpoint artifact inventory, and the authenticated
positive post-checkpoint integrity commit. It does not prove that the label
names the frozen source that ran. The authoritative lane-result bytes are
derived only from that checkpoint result, with its authority bit promoted; the
finalizer does not add its current identity to those otherwise stable bytes. The
attempt index stably binds the checkpoint and result. A separate append-only
completion attestation, whose filename is keyed by the finalizer fingerprint,
binds that exact index, result, and checkpoint to the current finalizer and
commits only after the positive integrity pair is re-authenticated. A finalizer-
only correction can therefore re-attest a compatible intact checkpoint without
claiming a new application execution, changing stable result/index bytes, or
replaying actions when every older-fingerprint attestation is already complete.
An older finalizer's data-only partial attestation currently blocks that
recovery as described below.

Aggregate acceptance, adapter, and platform fingerprints remain in lane
results for audit provenance. They do not turn an unrelated aggregate change
into a candidate-wide restart.

## Minimum-safe impact rules

The smallest safe invalidation unit is an owning lane plus its recursive
dependent closure. The current mapping is:

| Changed input | Required invalidation |
|---|---|
| Governing contract content used by one lane | That lane and its dependents; a global contract field included in every slice affects every lane |
| Acceptance assertion or expected-result input | Every lane assigned that acceptance input and their dependents |
| Fixture input | Every lane assigned that fixture and their dependents |
| Adapter lane, topology, action, or execution-harness input | The owning lane and its dependents |
| Declared source-environment value or lane environment/port contract | The owning lane and its dependents; a changed platform-base observation affects all lanes that receive it |
| Resolved external tool binary or Docker command prefix | Every lane that invokes that observed tool and its dependents |
| Product/source input in a dependency domain | Lanes declaring that domain and their dependents before freeze; all candidate-bound formal lanes also change when the frozen candidate changes |
| Validation policy, lifecycle, public entry point, or either public schema | All lanes, through the platform execution fingerprint |
| Finalizer only | Publication/finalization under the new finalizer identity; an intact completed execution checkpoint with no incomplete older-fingerprint attestation does not require action replay |
| Cargo build policy only | Aggregate platform provenance only, unless a lane actually consumes it through its declared harness or dependency inputs |

This precision depends on truthful declarations. Adapter validation proves
that directly referenced repository command and Compose files are owned by a
lane-scoped input inventory, and the governing contract supplies tracked
dependency patterns. The current no-reparse ancestor check is not complete on
Unix for repository names containing a literal backslash, so those declarations
do not yet establish an OS-native symlink-proof source boundary. The platform
also does not infer every semantic or transitive dependency or runtime of an
arbitrary program. In particular, a wrapper whose declared program is `pwsh`
does not automatically bind Cargo, Rust, Node, PostgreSQL,
browser, or other tools that the wrapper starts. A missing declaration, an
unknown changed path or runtime, or uncertainty about impact falls back to the
complete affected phase rather than silently inheriting a lane.

The platform currently computes and publishes these lane identities, but no
real sprint phase certifier consumes them end to end yet. A platform
`authoritative` lane result means only that the local checkpoint/finalizer
contract committed; it is not source-authenticated candidate or phase proof.
Until certifiers authenticate the supplied candidate fingerprint to the frozen
source/current dependency snapshot, assemble schema-v2 phase certificates, and
enforce the prerequisite/inheritance closure, this table defines the intended
safe selection boundary; it is not a claim that every current validation entry
point already avoids a full candidate restart.

## Cargo build storage platform slice

The first bounded platform component is
`scripts/tessara-cargo-build-policy.psm1`. Its contract is
`tessara.validation.cargo-build-policy`, its initial release is `1.0.0`, and
`Get-TessaraCargoBuildPolicyIdentity` publishes the release plus the exact
module SHA-256 fingerprint.

The component owns only Cargo build-storage lifecycle:

- one unique target directory per sequential validation lane;
- non-incremental, reduced-debug validation builds;
- explicit retained diagnostic builds;
- minimum free-space preflight;
- capability-bound target cleanup in `finally`; and
- restoration of the caller's Cargo environment.

Cleanup requires the private active lease, exact generated target identity,
authenticated marker bytes, ordinary repository/target paths, and a path chain
without reparse points. Caller-provided path containment alone is never cleanup
authority.

The application-independent self-test uses a synthetic temporary Cargo crate.
It proves release identity, deterministic free-space rejection, forged-state
and marker-tamper rejection, ancestor-reparse rejection on Windows, failed-lane
cleanup, exact environment restoration, retained diagnostic behavior, and no
temporary residue.

## Adapter inventory

| Adapter | Cargo policy status | Rationale |
|---|---|---|
| `scripts/validate.ps1` full gate | Current `1.0.0` consumer | One sequential general validation lane |
| `scripts/validate.ps1 -Fast` | Development mode; no isolated target | The developer owns the reusable worktree target |
| Closed Sprint 8B runners | Retained historical implementation | Their sealed evidence keeps the runner identity under which it was issued |
| Next active sprint adapter | Must consume the platform release | No new sprint-specific Cargo lifecycle implementation is permitted |

Changing the module, its self-test, or an active adapter requires the platform
self-test, the affected adapter boundary proof, PowerShell parsing, and source
integrity checks. Changing only this platform component does not claim that
Tessara application assertions ran.

## Platform release identity

`scripts/tessara-validation-platform.psm1` is the one supported public platform
entry point. Release `2.0.0` exports only:

- `Get-TessaraValidationPlatformIdentity`;
- `Get-TessaraValidationCandidateIdentity`;
- `Get-TessaraValidationCompatibilityPlan`;
- `Assert-TessaraValidationAdapter`; and
- `Invoke-TessaraValidationLane`.

The aggregate identity binds the exact platform manifest, four public
boundary inputs, and four components:

- public entry point, adapter schema, governing validation-contract schema, and
  phase-certificate-v2 schema;
- Cargo build policy, validation policy, lifecycle, and finalizer modules.

The narrower platform execution fingerprint binds only the public boundary,
validation policy, and lifecycle used by every lane. It excludes the currently
unused Cargo component and the separately recoverable finalizer. The finalizer
has its own fingerprint. All three identities are retained in the appropriate
evidence so a complete release can be audited without treating every platform
file as an application-execution dependency.

Lifecycle leases, port allocation, process control, topology control, cleanup,
restoration, hashing, and publication are not exported by the supported public
platform entry point, so a conforming sprint adapter has no supported way to
call them out of sequence. This API-ownership boundary is not a security sandbox
against repository code running under the caller's OS token.

## Declarative adapter contract

Future sprint adapters use schema-v2 `tessara.validation.adapter` JSON. Before
accepting an adapter, the platform runs the canonical validation-policy
validator over its governing `tessara-validation-v2` contract. The adapter must
then exactly cover the contract's lane inventory and declare the exact
prerequisites for every lane. This prevents a syntactically valid private
contract or a weakened adapter graph from defining a smaller validation cone.

The adapter assigns repository files independently as `acceptance_inputs`,
`fixture_inputs`, or `execution_inputs`, with an explicit lane list on every
entry. Every lane must own at least one acceptance and one execution input; a
fixture is optional. Every explicit no-topology lane must also declare at least
one action whose stage is `assertion`. A managed topology-producer lane may
contain setup only, but that producer result is topology/prerequisite evidence
rather than product-acceptance proof. Directly referenced repository action
files and Compose files must occur in an owned input inventory for that lane.

Every assertion action maps to exactly one governing `implementation_target`,
its exact structured command and declared transitive tools, and the target's
proof classes. Adapter validation requires exact required-target coverage for
each terminal lane and permits assertion-free setup only for the validated
create-and-handoff producer. This turns the stage label into a governed product-
proof binding rather than allowing an arbitrary successful command.

Each lane also declares scoped environment, blocked environment names, named
ports, one fixed topology mode/provider (including the explicit no-topology
mode), monotonic setup/assertion actions, direct program/argument arrays,
timeouts, and readiness. External `finalization` actions are not a legal
adapter stage: only the built-in filesystem finalizer may make evidence
authoritative. The platform rejects unknown fields, callbacks, shell command
strings, unknown placeholders, duplicate identities, cycles, no-op lanes with
no structurally declared assertion in no-topology mode, lexically invalid or
out-of-root repository paths, unowned direct inputs, and incomplete topology
handoffs. The OS-native symlink-ancestor residual below still applies.

Programs are launched directly with argument arrays. There is no platform
shell-string evaluation and no implicit action retry. A lane executes actions
sequentially. Independent lane processes have namespace isolation through
distinct attempts, leases, project identities, and port sets; callers choose
evidence roots, and prerequisite/handoff lanes must share the authenticated
root. The two-lane synthetic concurrency proof establishes namespace separation
only. It is not a scheduler, concurrency limit, or CPU/memory/disk quota.
Planning hashes the resolved executable path/content for every directly
declared topology/action program and the Docker prefix; the lifecycle
re-resolves and compares that observation before writing the attempt-start
record and again immediately before each declared action, topology start, or
Docker call, including cleanup. A detected direct-program substitution is a
setup/execution failure and cannot be used as destruction authority or accepted
evidence. Transitive programs started by a wrapper remain a completion gate.

The lifecycle also re-hashes lane-owned acceptance, fixture and execution-
harness inputs, command inputs, and every declared dependency domain after
actions and cleanup, records any drift as a `source-integrity` failure,
and rechecks the same cone immediately after writing the execution checkpoint.
The first gate records failure in the raw checkpoint and prevents an
authoritative pass. Only a successful second gate may publish the authenticated
`execution-integrity-verified.json` pair bound to that exact checkpoint. The
finalizer requires that positive pair; an absent, partial, substituted, or
checkpoint-mismatched pair cannot authorize result/index publication. A
post-checkpoint failure also attempts an `execution-revoked.json` deny marker,
but eligibility does not depend on the negative marker's presence. When a
retained handoff is still pending, the source keeps cleanup responsibility. It
attempts teardown of ineligible topology before returning and records any
cleanup failure; if a positive-integrity member may exist, the guarded
finalization path instead requires durable revocation before teardown and
retains topology when revocation cannot commit. A checkpoint
that records integrity failure cannot gain a later positive commit even if deny-
marker publication failed and the changed input was restored. Finalization-only recovery rechecks
the current lane input cone and requires the already committed positive pair;
total absence is ineligible rather than authority to mint the missing commit.
This closes the planning-to-publication window without making JSON formatting
or another lane's adapter declaration part of the retry cone. The validated
lane-scoped adapter and governing-contract fingerprints remain bound to the
result and are recomputed by every later plan.

These checks currently prove repeatability of the bytes reached by the path
resolver, not a complete cross-platform source-cone boundary. Both planner and
lifecycle ancestor guards split a relative path on slash and backslash. On Unix,
backslash is a literal filename character, so a backslash-named symlink ancestor
can be skipped by that traversal while later file reads follow it outside the
repository. Until path segmentation uses only OS-native separators and an
adversarial Unix proof closes this case, such evidence cannot authorize
selective reuse.

## Prerequisites, environment, and setup containment

A dependent invocation must supply exactly one canonical, sidecar-verified
lane result for every ordinary direct prerequisite and no others. Each result
must be authoritative, passing, under the same evidence root, indexed by its
own attempt, and equal to the prerequisite's current acceptance, adapter, lane
compatibility, and platform execution identities. Candidate equality is also
required whenever the prerequisite or dependent phase is candidate-bound. A
missing, duplicate, failed, stale, or substituted prerequisite blocks the lane
before assertions start. A Compose topology predecessor is authenticated
through the stronger handoff protocol below rather than accepted as an
ordinary result path.

Child programs do not inherit the caller's arbitrary process environment. The
platform builds a filtered environment from a small operating-system/tool
base allowlist, explicit literal or source bindings in the lane, blocked-name
sentinels, leased ports, and platform-owned Compose values. It records only
binding metadata and value hashes, and redacts process-sourced values from
retained action/topology logs and failure messages when a child echoes their
exact values. `HOME`, `USERPROFILE`, and the caller's Docker configuration are
not forwarded, and Docker gets an attempt-owned `DOCKER_CONFIG`. `TEMP`, `TMP`,
and `TMPDIR` point to an attempt-owned directory under the owned evidence
tree. Every OS-started child immediately enters truthful start accounting and
cleanup ownership. Successful acquisition returns with redirected stdin closed;
a later acquisition failure runs stop/dispose cleanup and reports any incomplete
cleanup before propagating. An absent or empty required source binding fails in
setup with no assertion action and no topology ownership; undeclared ambient variables outside
the documented platform-base allowlist are not forwarded. Lane bindings are
written only into each child process; the platform does not write or restore
them in the caller process, because restoring a stale snapshot could overwrite
a legitimate concurrent runspace change.

This is environment filtering, not an operating-system sandbox. The child keeps
the caller's access token and therefore its filesystem, network, process, and
Docker-daemon authority. The base also still forwards caller `PATH` and
`PSModulePath`; PowerShell can autoload a user module and a wrapper can
resolve an undeclared tool. Exact declared/transitive tool inventories, a
mandatory noninteractive policy for relevant tools, job or process-group
containment, and a least-authority filesystem/network/process sandbox remain
completion gates. The attempt-owned temporary directory isolates names but is
not yet protected by an execution-time storage quota.

Actions marked `setup` run before a newly created topology starts. For a
retained Compose topology, every platform-owned read-only gate (receipt,
prerequisite, tool, input, and normalized Compose-configuration validation) runs
before claim publication, but the claim is published before the first adapter
setup action can mutate the existing service. A read-only consumer defect
therefore leaves the receipt reusable; any failure after adapter code can touch
the topology is owned by the consumer and returns through provider cleanup.

Platform release `2.0.0` has two managed topology providers in addition to the
explicit no-topology mode:

- `local-process` owns its direct child from successful OS start, including
  acquisition-failure cleanup, and descendants still attached when the parent
  is stopped, plus a loopback-port lease;
- `docker-compose` owns a unique project identity, normalized configuration
  checks, required-service health, declared lease-label inspection, teardown
  commands, and project-label residue rejection in the certified shimmed
  boundary.

Those checks do not yet make arbitrary real Compose configuration safe. Before
the first live `up`, the provider must reject or explicitly authorize custom or
external volume/network names, foreign-resource collisions, host bind mounts,
Docker-socket access, host network/PID/IPC modes, privileged/capability/device
access, secrets/config files, and build contexts outside the exact lane-owned
cone. It must also bind the Docker endpoint/daemon/context, Compose dotenv and
`env_file` inputs, immutable image digests or complete build inputs, and actual
running container/image identities. The live-Docker adversarial proof must show
that foreign resources and host paths survive start, failure, handoff, and
cleanup. Until then the shim proves command and namespace logic, not broad
resource ownership or safe real-engine destruction.

Compose topology may be created and destroyed in one lane or transferred by a
capability-bearing, write-once receipt to one declared successor. The receipt
contains a random 256-bit topology capability and binds its hash, the canonical
source attempt, source and target lane compatibility/acceptance/adapter
identities, candidate, platform execution identity, project, Compose file and
normalized configuration, canonical evidence-root fingerprint, and ports. The
source's authoritative result and attempt index must bind the same receipt.
The source remains the cleanup owner while the receipt, checkpoint, lane result,
and attempt index are finalized. Ownership transfers only after the finalizer
has authenticated and published that authoritative handoff evidence.

After all platform-owned read-only gates succeed and before any adapter setup
action, the consumer claims the capability by create-once publication at a path
derived from the capability hash. While the complete claim pair remains, a
second claim therefore fails before assertions, and copying the source attempt
into a different evidence root cannot create a fresh claim namespace because
the root fingerprint no longer matches. From the first successful claim until
verified teardown, the consumer owns cleanup. The invocation that wins creation
of the claim data assumes cleanup ownership immediately: if sidecar publication
then fails, it removes its partial pair and destroys the topology. An invocation
that loses a claim-data creation race never gains cleanup authority. Once the
complete claim reference returns, cleanup ownership remains effective even if a
later bookkeeping assignment faults. Partial-
publication rollback is proved only for a sequential retry; deletion reopens the
claim path before cleanup is serialized, leaving the overlapping-consumer race
identified below. A
handoff-publication failure causes the source to destroy the topology instead
of exposing an unauthenticated transfer. Positive-integrity creation and
finalization now share one guarded retained-handoff region. After finalization
commits, ownership-transfer flags are cleared inside that same region. If a
fault occurs first and either integrity-commit member may exist, the source
commits revocation before teardown and retains the topology when revocation
cannot commit. If neither member exists, the checkpoint is ineligible for
finalization-only recovery and cleanup may proceed.

## Evidence and certification

Each invocation creates a unique attempt directory. Successful action logs,
topology logs, attempt intent/start records, topology receipts and claims, the
execution checkpoint, positive integrity commit, lane result, and attempt index
are published with create-once UTF-8 writes, read back by SHA-256, and paired
with sidecars. Partial pair publication is rolled back only where the lifecycle
retains explicit ownership; the topology-claim overlap limit is recorded below.
Prior attempts are never overwritten.
The finalizer's shared create-once writers likewise track whether the current
invocation created each data or sidecar file. A caught write, flush, or dispose
failure removes only that invocation-owned partial. Focused certification
injects all six result/index/attestation data and sidecar publication boundaries
and proves the same checkpoint can retry without deleting preexisting evidence.
Failure and cooperative interruption still reach a platform-write-once hashed
checkpoint and,
when finalization succeeds, publish a terminal result with the failure stage,
assertion-start/completion state, primary error, and cleanup/restoration
outcome. Cleanup failure fails the lane and is retained alongside the primary
failure.

After all declared actions and cleanup/restoration finish, the lifecycle writes
a platform-write-once `execution-complete.json` checkpoint with the raw result
and exact artifact inventory, then performs the post-checkpoint input-integrity
gate described above and publishes the positive integrity commit only on success.
The separate finalizer requires and authenticates that exact positive pair,
rejects a revoked checkpoint, verifies an intact one, and only then publishes an
authoritative `lane-result.json` and `attempt-index.json`.
Before reading checkpoint or artifact content, the finalizer holds the exact
attempt's finalization lock and inventories its filesystem tree. Every entry
must be an ordinary non-reparse file or directory; the attempt may contain at
most 256 files and 64 subdirectories, no relative path may exceed 512
characters or eight levels, no file may exceed 16 MiB, and aggregate file size
may not exceed 64 MiB. The ordinary-file inventory must equal the checkpoint-
declared artifact data/sidecar pairs plus the checkpoint pair, the required
positive integrity-commit pair, and the known result/index/revocation
publication names and append-only finalization-attestation pairs. A recoverable
result or index data file may precede its sidecar. Only the current-fingerprint
attestation data may temporarily precede its sidecar; every older attestation
must be a complete, digest-valid, semantically authenticated pair. A sidecar
without its data file is never accepted. Unlisted files, orphan sidecars,
special files, links, junctions, reparse paths, and any size/count overflow fail
closed before JSON parsing or artifact publication. The finalizer repeats the
exact inventory and artifact-digest checks before starting result/index/
attestation publication, writes stable result/index pairs, authenticates every
retained prior attestation, and writes the current attestation data. It then
performs the final prerequisite rechecks, repeats every checkpoint artifact
digest, re-authenticates the positive integrity pair, and checks revocation
immediately before publishing the attestation sidecar as the completion commit.
These late checks narrow but do not serialize the concurrent-writer window
described below. Ordinary prerequisite and
handoff consumers authenticate the index contract/schema/state/attempt, its
exact result and hashed canonical checkpoint binding, and the current
fingerprint-keyed attestation pair. Result or index pairs without that current
commit cannot authorize downstream work.
It re-authenticates every prerequisite's complete result/index/checkpoint/current-
attestation chain and no-revocation state, both while validating the dependent
checkpoint and late in its attestation commit sequence. When present, it also
re-authenticates the contained topology-claim reference and exact retained
handoff-receipt artifact. Detected substituted, revoked, or incompletely
committed dependency or ownership evidence cannot become authoritative; the
unserialized final-commit race described below remains open.
The lane-result bytes are the checkpoint's result with authority promoted and
are stable across finalizer-only corrections. The stable attempt index binds
those bytes back to the exact checkpoint. Older authenticated finalizer
attestations may coexist, while consumers require the attestation keyed by the
current finalizer identity.
`-FinalizeCheckpointPath` accepts no topology, prerequisite, or cancellation
inputs and reruns no topology or declared action. It is the only supported
recovery entry point for a publication failure after a complete checkpoint plus its
authenticated positive integrity pair, and it retains the original attempt
identity. Because external finalization subprocesses are
rejected at adapter validation, recovery can only repeat the built-in current-
input check, integrity/digest verification, and filesystem-publication step.
Given an authentic lifecycle-created checkpoint and its committed positive
integrity pair, it cannot replay or convert a failed product assertion into a
passing result; an independently authenticated lifecycle checkpoint capability/
provenance boundary is still a completion requirement.

Finalization-only recovery is not yet complete across a finalizer identity
change. The current finalizer may repair its own data-only attestation, and a new
finalizer can retain complete older attestations, but it rejects an older-
fingerprint attestation data file whose sidecar never committed. Until the
protocol can authenticate and quarantine or complete that partial state, the
safe fallback is to rerun the affected lane; operators must not delete evidence
to manufacture eligibility. The caught-writer rollback above does not claim to
run after abrupt host or machine termination.

Platform-backed phase certificates use
`phase-certificate-v2.schema.json`. Every lane carries its compatibility
fingerprint; an inherited pre-freeze lane must carry the same prior
compatibility fingerprint and unchanged dependency-domain fingerprints.
Candidate-bound phases cannot inherit a lane from another candidate. Schema-v1
phase certificates remain readable for sealed historical evidence, but they do
not contain the compatibility proof required for new platform-backed
inheritance. The generic structural policy/evidence-chain validator deliberately
rejects v2 as unauthenticated. `New-TessaraPlatformPhaseCertificate` publishes a
write-once current compatibility plan, evidence index, and v2 certificate;
`Assert-TessaraPlatformPhaseCertificate` recomputes the plan and authenticates
source/environment identity, lane receipts, attempt indexes, current finalizer
attestations, exact target coverage, prerequisites, and permitted inheritance.

`scripts/test-tessara-validation-platform.ps1 -SelfTest` is the mandatory
application-independent certification. A synthetic local service proves
success, child failure, timeout, interruption, acquisition-fault and direct-
child cleanup, caller-environment preservation, platform-write-once hashed
evidence, and concurrent namespace isolation. A
deterministic Docker CLI shim proves exact Compose commands, project
identity, health, handoff, cleanup, and residue rejection without external
network access, a real Docker engine, or Tessara application execution.

The synthetic certification deliberately stops at process-local and shimmed
failure boundaries. Its cancellation case does not prove cleanup after the
outer PowerShell host, operating system, or machine is killed before `finally`
or before the execution checkpoint exists. Any current abrupt-host-termination
probe is diagnostic-only. Completion requires an independently executing,
capability-authenticated orphan-discovery/cleanup protocol plus adversarial
proof that forged or copied on-disk state cannot authorize destruction. Release
`2.0.0` does not yet claim that hard-crash capability.

The foundation also has an application-independent live-Docker proof using a
digest-pinned non-root/read-only image, authenticated daemon/context,
challenge-response readiness, exact running container/image identity, single-
use producer/consumer handoff, teardown, and zero residue. A downstream sprint
that supplies a product adapter must repeat these checks against its exact
Compose model, but that downstream proof is not diversion-closeout evidence;
the generic fixture is the provider-foundation proof. Closed
Sprint 8B runners and policy-v2 evidence retain the historical implementation
under which they were issued; they are not retrofitted.

The remaining containment limits are deliberate fail-closed boundaries, not
silent reuse authority:

- the planner and lifecycle no-reparse ancestor guards are not OS-native on
  Unix: a literal-backslash directory name can hide a symlink traversal outside
  the repository, so source-cone authority requires corrected segmentation and
  an adversarial cross-platform escape proof;
- directly declared programs are observed, but wrapper-started runtimes and
  user modules reachable through `PATH`/`PSModulePath` are not;
- children keep the caller's full OS-token authority, and no operating-system
  job/process group contains a daemonized descendant; OS-started children enter
  immediate truthful cleanup ownership, successful acquisitions have closed
  stdin, acquisition cleanup failures are reported, and temp paths are attempt-
  owned, but relevant tools are not universally forced into their own
  noninteractive mode;
- platform write-once file creation is not OS immutability: late artifact-
  digest and prerequisite checks do not lock same-token or detached writers out
  of the artifact paths or prerequisite roots through the attestation-sidecar
  commit, so finalization still needs a cross-root commit-locked closure, OS
  writer containment, or an equivalent immutable snapshot;
- an attestation data-only partial left by an older finalizer fingerprint,
  including one left by termination outside caught-writer rollback, blocks a
  corrected finalizer from recovering the otherwise complete checkpoint, so
  cross-finalizer partial-state recovery needs a deterministic proof and safe
  protocol rather than manual evidence deletion;
- partial topology-claim publication removes its incomplete pair before the
  owning consumer finishes teardown, so an overlapping consumer can recreate
  the claim; a durable tombstone or claim/cleanup serialization and an
  adversarial overlap proof are still required for race-safe single use;
- release 2 rejects ungoverned Compose host/resource capabilities, mutable
  images/builds, external/custom resources, and implicit environment; each
  product adapter must still prove its exact allowed model with live Docker;
- a retained handoff that becomes incompatible before its consumer claims it
  has no authenticated cleanup-only successor;
- there is no phase-wide scheduler, aggregate deadline, concurrency or
  CPU/memory/disk/process/global-port/Docker quota, attempt-temp storage budget,
  or capability-bound evidence-root retention/pruning and sensitive-artifact
  disposal policy; and
- product-wrapper transitive tool receipts must be carried into the first real
  adapter where tools are not directly declared.

Until an applicable residual boundary is closed for a lane, uncertainty falls
back to that affected lane/closure, phase, or candidate rather than authorizing
narrower reuse. Diversion closeout is based only on the frozen diversion
candidate and its own evidence. A later sprint may consume the closed release,
but its adapter results neither complete nor reopen this diversion.

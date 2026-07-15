# Sprint 6A Deployment Evidence

Sprint 6A acceptance uses a retained, machine-verified deployment record. A
switch such as “release build confirmed” or “disposable database confirmed” is
not proof and is not accepted by the closing gates.

## Capture

Sprint 6A closeout requires two distinct records in one fixed order. Complete
Gate 4 against the restored Sprint 5A demo target after the clean closing
startup applies migration 3 with demo seeding disabled, then retain the
`upgraded` deployment, smoke, UAT, Playwright, and nondisclosure artifact sets.
Only then may Gate 5 reset/start a fresh installation and publish the separate
`fresh` sets. The two states are non-interchangeable, and neither substitutes
for Gate 3 rollback/restore evidence. The representative populated fixture used
by `sprint_6a_populated_upgrade` and `CompatibilityOnUpgraded` remains separate;
it is not the Gate 4 browser candidate.

The command immediately below is the Gate 5 **fresh** capture reference. Do not
run it first or treat it as the whole closeout. After retaining Gate 4, reset
and start Gate 5 from the same clean committed closing image, then capture:

```powershell
.\scripts\local-launch.ps1 -FreshData
$evidence = 'artifacts/sprint-6a/deployment-fresh.json'
.\scripts\capture-sprint-6a-deployment-evidence.ps1 `
  -BaseUrl 'http://127.0.0.1:8080' `
  -ExpectedDataState fresh `
  -OutputPath $evidence
```

The default capture discovers the running `api` and `postgres` services in the
current Compose project. When the upgraded restored demo target is served by a
separate release deployment, pass the exact running `-ApiContainerId` and
`-DatabaseContainerId`. The API's `DATABASE_URL` must bind to that database
container either through shared-network Docker DNS or through the container's
unique published PostgreSQL port, and the BaseUrl port must be the API
container's unique published port 8080.

The retained database identity is always the exact three-part binding
`database_runtime.container_id` + `database_runtime.database_user` +
`database_runtime.current_database`. A container ID or database name by itself
is insufficient. Capture derives the PostgreSQL user from the live API URL,
queries `current_database()` through that user in the named container, and
records all three values. Every acceptance wrapper must revalidate that same
triple before and after its own checks. Playwright acceptance additionally
exports the validated values as `PLAYWRIGHT_POSTGRES_CONTAINER`,
`PLAYWRIGHT_POSTGRES_USER`, and `PLAYWRIGHT_POSTGRES_DATABASE`, plus the exact
validated `TESSARA_PLAYWRIGHT_DATA_STATE`; its fixture cleanup therefore
targets the evidence-bound database through `docker exec`, not an implicitly
selected Compose service, and upgraded setup cannot invoke the demo seed path.

Capture writes UTF-8 JSON and a sibling `.sha256` file beneath the ignored
`artifacts/sprint-6a/` directory. It first writes a unique temporary pair,
validates that pair against the live deployment, and only then publishes both
files. Existing evidence is preserved by default; `-Overwrite` is required for
an intentional replacement, and backup rollback preserves the last valid pair
if publication fails. Retain both files with closeout artifacts. Do not edit
either file or use `-Overwrite` merely to normalize a failed gate.

### Closing Image On The Restored Sprint 5A Demo Clone

Gate 4 must not silently test the ordinary Compose database. After Gate 3 has
finished `CompatibilityOnUpgraded` on the representative fixture, back up the
separate Sprint 5A demo source, restore it into a disposable target, and
validate that target with `OriginalAfterRestore`. The restore evidence's
all-table fingerprint binds the target to the pre-upgrade source. That source
must already contain the exact
admin/operator/respondent/delegator/delegate actors and Demo Session Log form,
dataset, six component kinds, nine-placement Demo Operations Dashboard,
workflow assignments, and delegation used by acceptance. Do not seed those
assets after migration 3.

Expose the restored target's PostgreSQL container port to the host and
construct the target URL with `host.docker.internal` as its host. Then launch
the clean closing image without reset or demo seed; startup itself applies
migration 3:

```powershell
$evidence = 'artifacts/sprint-6a/deployment-upgraded.json'
$upgradeDatabaseContainer = '<exact-running-restored-demo-database-container-id>'
$containerDatabaseUrl = '<postgres://credentials@host.docker.internal:published-port/exact-restored-demo-database-name>'
.\scripts\local-launch.ps1 `
  -ExternalDatabaseUrl $containerDatabaseUrl `
  -ExternalDatabaseContainerId $upgradeDatabaseContainer `
  -SkipSeed
$apiContainer = (docker compose ps -q api).Trim()
.\scripts\capture-sprint-6a-deployment-evidence.ps1 `
  -BaseUrl 'http://127.0.0.1:8080' `
  -ExpectedDataState upgraded `
  -OutputPath $evidence `
  -ApiContainerId $apiContainer `
  -DatabaseContainerId $upgradeDatabaseContainer
```

The launch path rejects `-FreshData`, `-ApiOnly`, and omitted `-SkipSeed`; it
also rejects a database name without a token-bounded disposable marker or a URL
whose port and `current_database()` do not match the supplied running database
container. Compose still starts its normal dependency container, but the API
uses only the explicit `TESSARA_DATABASE_URL` override. Evidence records the
actual API binding mode (`published_host_port`) and verifies the live API's
Application Installation/catalog against that exact external container.
The upgraded record must have `data.state = upgraded` and
`pre_migration_3_product_rows.total > 0`. With
`ExpectedDataState=upgraded`, smoke, UAT, and Playwright never call
`/api/demo/seed`; they resolve and prove the restored acceptance assets. Any
Gate 4 demo mutation disqualifies the run.
The override is scoped to that launch invocation: `local-launch.ps1` restores
the caller's previous environment value in `finally`, and a normal/fresh launch
explicitly removes any inherited `TESSARA_DATABASE_URL` while Compose is
running. Gate 5 therefore cannot accidentally reuse Gate 4's upgrade URL.

## Schema Version 1

The root has `schema_version = 1`,
`evidence_kind = tessara.sprint-6a.deployment-evidence`, a UTC generation time,
and one `snapshot` containing:

- exact BaseUrl;
- clean Git commit and tree;
- running API container ID, immutable Docker image ID, published port, image
  creation time/reference/repository digests, and matching
  commit/tree/clean/release labels;
- proof that the API container has no mounts or writable-layer changes and does
  not override the release image's command, entrypoint, working directory,
  user, site paths, or migration path;
- database container ID, direct network/hostname/port binding, and
  the non-secret PostgreSQL user plus `current_database()` as one exact
  container/user/database triple;
- successful SQLx ledger exactly 1–3, including installed timestamps and
  checksums, plus SHA-384 checksums independently derived from the current
  migration source files;
- the exact `admin`, `operator`, and `respondent` membership rows, canonical
  seed version, and recomputed canonical SHA-256;
- the single Application Installation identity and Core runtime observation,
  including exact API package version and `Cargo.toml` digest matched across
  source, SQL, and the live inventory response;
- exact seven-definition/seven-source/seven-projection current transition
  catalog, six navigation contributions, one policy, source digests matched to
  their exact checked-in source bytes/digest sidecars and the live
  module-inventory API, exact current policy entry contents, API schema v1, and
  proof that Release/Instance tables do not exist in 6A;
- database-derived `upgraded` or `fresh` state and per-product pre-migration-3
  row counts; and
- live health and module-inventory response facts. The authenticated inventory
  read uses a uniquely identified evidence-only session and deletes exactly that
  session before capture can succeed.

The data-state rule is fixed: `upgraded` means at least one Form, Workflow,
Submission, Dataset, Component, or Dashboard row was created before successful
migration 3; `fresh` means none was. For Gate 4, the pre-migration rows must be
the restored Sprint 5A demo baseline, the closing launch must use `-SkipSeed`,
and upgraded smoke/UAT/Playwright must not call `/api/demo/seed`; they prove the
existing assets directly. Creating or replacing demo data after migration 3
disqualifies the upgraded pass even if the historical classifier still returns
`upgraded`. Gate 5 may create ordinary fresh demo/UAT rows after migration 3;
those rows cannot turn fresh evidence into upgraded evidence.

Rollback backup/restore proof is a separate contract, not deployment schema
v1. `capture-sprint-6a-rollback-restore-evidence.ps1` writes
`schema_version = 3` with evidence kind
`tessara_sprint_6a_pre_upgrade_backup_restore_proof`; it binds the capture and
common-helper digests, exact PostgreSQL client identity, custom archive, source
and restored migration-2 fingerprints, case-sensitive database identities, and
sanitized restore operations. Deployment, rollback-restore, smoke/UAT,
Playwright, and nondisclosure evidence are not interchangeable merely because
they share an artifact directory.

## Validation And Lifetime

Smoke, UAT, Playwright acceptance, and resource-reference nondisclosure each
require `-DeploymentEvidencePath` and `-ExpectedDataState upgraded|fresh`. Before
their own checks, they verify the JSON sidecar and rederive the full snapshot
from the current clean source, running containers, live BaseUrl/API, and live
database. Any byte change, opposite data state, source change, container/image
replacement, port/hostname/database change, migration or seed drift, or catalog
drift fails the gate. Capture a new record only after intentionally deploying a
new environment; never relabel an old record.

Playwright acceptance writes discovery, execution JSON, JUnit, and summary into
a unique temporary directory. It validates the complete set against the
schema-v2 durable manifest's exact 60 full file/describe/test identities before
publishing the state-specific final paths. Discovery and execution must each
match the manifest by exact ordinal file + describe + test identity, with no
duplicates, additions, removals, renames, skips, filters, retries, or count-only
substitution. The retained summary records the exact manifest path and SHA-256
alongside the deployment-bound database name/user, commit, image, and data
state. `-OverwriteEvidence` authorizes replacement of the four already
validated output artifacts only; it never rewrites or accepts
`end2end/acceptance-manifest.json`. Changing that checked-in manifest requires
an approved test-change rationale and equivalent or stronger executable proof.
A failed run leaves prior green evidence untouched, and replacement requires
`-OverwriteEvidence`; publication backs up the whole prior set and rolls it
back if any final move fails. The wrapper also restores the caller's exact
prior presence and value for every Playwright environment variable it touches,
so a later npm command cannot inherit an acceptance reporter path and overwrite
retained evidence.

Smoke and UAT accept `-AcceptanceEvidencePath` and publish an allowlisted
structured JSON summary plus `.sha256` sidecar only after all checks, a final
live deployment-evidence revalidation, exact logout of every bearer/browser
session created by that run, verified credential-file removal, and exact
environment restoration. The evidence binds upgraded/fresh state, deployment
digest, commit/tree, image, database/installation, catalog/seed, runner digest,
and the runner's exact check categories; unknown fields and raw secret material
are rejected. Acceptance and deployment evidence paths must remain physically
distinct. Reparse points, alternate data streams, hard-link aliases, and a
concurrent publisher fail closed. Publication is a rollback-safe sequential
JSON/sidecar transaction; replacement requires
`-OverwriteAcceptanceEvidence`, and cleanup or final-validation failure
preserves the prior pair.

Resource-reference nondisclosure follows the same retained-proof rule for its
JSON and sibling `.sha256` file. The wrapper refuses either pre-existing member
unless `-Overwrite` is explicit, writes only beneath a unique temporary path
while it validates exact JSON types, canonical UUID/UTC representations, the
live deployment/fixture IDs, restricted-envelope body digests, sample counts,
and timing claims. It then moves JSON and sidecar sequentially within a
rollback-safe transaction; this is not a two-file reader-atomic protocol. A
failed move, final hash/result construction, or cleanup removes the replacement
and restores the prior pair byte-for-byte. Input/output/temp/backup path chains
containing a reparse point, junction, or symbolic link are rejected, cleanup
failures surface, and temporary/backup/restore paths are not accepted evidence.
HTTP failure diagnostics contain only label, status, content type, UTF-8 byte
length, and SHA-256—not raw login or restricted response bodies.

`-DevelopmentMode` on smoke, UAT, and Playwright is an explicit local-diagnostic
bypass. It cannot be reported as Sprint acceptance. The nondisclosure timing
gate has no bypass because it is release-only evidence.

The database-free implementation self-test is:

```powershell
.\scripts\local-launch.ps1 -SelfTest
.\scripts\capture-sprint-6a-deployment-evidence.ps1 -SelfTest
.\scripts\validate-e2e.ps1 -SelfTest
.\scripts\validate-resource-reference-nondisclosure.ps1 -SelfTest
.\scripts\test-sprint-6a-rollback-package.ps1 -SelfTest
.\scripts\test-sprint-6a-acceptance-evidence.ps1
```

They prove caller environment override clearing/restoration, the built-in seed
digest, exact migration-checksum rejection, exact catalog shape, schema
validation, upgraded/fresh non-interchangeability, retained-pair/set overwrite
refusal, exact-type/identity/digest/path-alias rejection, bounded HTTP
diagnostics, strict sidecar validation, verified cleanup, and rollback after
partial publication or former post-publication finalization failures. The
acceptance self-test additionally proves exact current-run session revocation,
failed-login cookie cleanup, deployment/acceptance path separation,
hard-link/ADS/reparse/concurrency rejection, and cleanup-failure publication
blocking.

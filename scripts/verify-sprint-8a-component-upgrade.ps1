[CmdletBinding()]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [Parameter(Mandatory = $true)][string]$BaselineMetadataPath,
    [string]$CandidateManifestPath = "crates/tessara-component-module/manifest.json",
    [string]$CatalogTemplatePath = "deploy/sprint-8a/catalogs/local-release-catalog.json",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath = "target/sprint-8a-upgrade/component-upgrade-rollback.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-health-contract.ps1")
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-8a"
$componentDefinition = "tessara.components"
$baselineRelease = "0.9.0"
$candidateRelease = "1.0.0"

function Resolve-RepositoryPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) { return [IO.Path]::GetFullPath($Path) }
    return [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Copy-JsonValue($Value) {
    return ($Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json)
}

function ConvertTo-StableJson($Value) {
    return ($Value | ConvertTo-Json -Depth 100 -Compress)
}

function Assert-ExactDeltaPlan($Lockfile, [string]$ExpectedImageDigest) {
    $actions = @($Lockfile.materialization_plan.actions)
    $expected = @(
        [ordered]@{ action = "acquire_image"; component = $componentDefinition; digest = $ExpectedImageDigest },
        [ordered]@{ action = "migrate"; owner = $componentDefinition; image = $ExpectedImageDigest },
        [ordered]@{ action = "health_gate"; owner = $componentDefinition },
        [ordered]@{ action = "switch_traffic"; owner = $componentDefinition },
        [ordered]@{ action = "verify_read_back" }
    )
    if ((ConvertTo-StableJson $actions) -cne (ConvertTo-StableJson $expected)) {
        throw "Component release transition is not an exact one-owner semantic delta: $(ConvertTo-StableJson $actions)"
    }
}

function Get-RunningContainerId([string]$Service) {
    $ids = @(@(& docker compose -f $composePath --profile reference ps --status running -q $Service) |
        ForEach-Object { ([string]$_).Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($LASTEXITCODE -ne 0 -or $ids.Count -ne 1 -or [string]$ids[0] -cnotmatch '^[0-9a-f]{64}$') {
        throw "Expected exactly one running '$Service' container in $expectedProject."
    }
    return [string]$ids[0]
}

if ($SelfTest) {
    Test-Sprint8AHealthContract | Out-Null
    $mock = [pscustomobject]@{
        materialization_plan = [pscustomobject]@{
            actions = @(
                [pscustomobject]@{ action = "acquire_image"; component = $componentDefinition; digest = "sha256:$('a' * 64)" },
                [pscustomobject]@{ action = "migrate"; owner = $componentDefinition; image = "sha256:$('a' * 64)" },
                [pscustomobject]@{ action = "health_gate"; owner = $componentDefinition },
                [pscustomobject]@{ action = "switch_traffic"; owner = $componentDefinition },
                [pscustomobject]@{ action = "verify_read_back" }
            )
        }
    }
    Assert-ExactDeltaPlan $mock "sha256:$('a' * 64)"
    $mock.materialization_plan.actions = @($mock.materialization_plan.actions + [pscustomobject]@{ action = "configure"; owner = "tessara.dashboards"; digest = "sha256:$('b' * 64)" })
    $rejected = $false
    try { Assert-ExactDeltaPlan $mock "sha256:$('a' * 64)" } catch { $rejected = $true }
    if (-not $rejected) { throw "Component upgrade self-test accepted an unrelated owner mutation." }
    Write-Host "Sprint 8A Component semantic-delta upgrade verifier self-test passed."
    return
}

$configuration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or [string]$configuration.name -cne $expectedProject) {
    throw "Component upgrade verifier is restricted to the exact $expectedProject Compose project."
}
$moduleControlKey = [string]$configuration.services.components.environment.TESSARA_MODULE_CONTROL_SHARED_KEY
if ([string]::IsNullOrWhiteSpace($moduleControlKey)) {
    throw "Component upgrade verifier requires the normalized Module control key."
}
$baselineMetadataFullPath = Resolve-RepositoryPath $BaselineMetadataPath
$candidateManifestFullPath = Resolve-RepositoryPath $CandidateManifestPath
$catalogTemplateFullPath = Resolve-RepositoryPath $CatalogTemplatePath
$outputFullPath = Resolve-RepositoryPath $OutputPath
foreach ($required in @($baselineMetadataFullPath, $candidateManifestFullPath, $catalogTemplateFullPath)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Required upgrade input is missing: $required" }
}
if (-not (Test-Path -LiteralPath "$baselineMetadataFullPath.sha256" -PathType Leaf) -or
    (Get-Content -LiteralPath "$baselineMetadataFullPath.sha256" -Raw).Trim() -cne (Get-Sprint7AFileSha256 -Path $baselineMetadataFullPath)) {
    throw "Component baseline release metadata is missing its matching SHA-256 sidecar."
}
if ((Test-Path -LiteralPath $outputFullPath) -or (Test-Path -LiteralPath "$outputFullPath.sha256")) {
    throw "Component upgrade evidence already exists and cannot be overwritten: $outputFullPath"
}
$source = [ordered]@{
    commit = (& git -C $repoRoot rev-parse HEAD).Trim()
    tree = (& git -C $repoRoot write-tree).Trim()
    dirty = -not [string]::IsNullOrWhiteSpace((& git -C $repoRoot status --porcelain=v1))
}
if ($source.dirty) { throw "Component upgrade/rollback rehearsal requires clean source." }

$baselineMetadata = Get-Content -LiteralPath $baselineMetadataFullPath -Raw | ConvertFrom-Json
if ([string]$baselineMetadata.release_identity.definition_id -cne $componentDefinition -or
    [string]$baselineMetadata.release_identity.version -cne $baselineRelease -or
    [string]$baselineMetadata.release_identity.runtime_image -cnotmatch '^sha256:[0-9a-f]{64}$' -or
    [string]$baselineMetadata.release_identity.manifest_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
    [string]$baselineMetadata.source_identity.commit -cne [string]$source.commit -or
    [string]$baselineMetadata.source_identity.tree -cne [string]$source.tree -or
    [bool]$baselineMetadata.source_identity.dirty) {
    throw "Component baseline metadata is not bound to the clean current source and distinct 0.9.0 release."
}
$baselineManifestFullPath = Resolve-RepositoryPath ([string]$baselineMetadata.manifest_path)
if (-not (Test-Path -LiteralPath $baselineManifestFullPath -PathType Leaf)) {
    throw "Component baseline release manifest is missing: $baselineManifestFullPath"
}
$baselineManifest = Get-Content -LiteralPath $baselineManifestFullPath -Raw | ConvertFrom-Json
if ([string]$baselineManifest.definition_id -cne $componentDefinition -or
    [string]$baselineManifest.release_version -cne $baselineRelease) {
    throw "Component baseline Manifest does not identify the compatible 0.9.0 release."
}
$baselineManifestDigestOutput = @(& cargo run -q -p tessara-supervisor --bin tessara-compose -- digest $baselineManifestFullPath)
if ($LASTEXITCODE -ne 0 -or
    [string]($baselineManifestDigestOutput | Select-Object -Last 1) -cne [string]$baselineMetadata.release_identity.manifest_digest) {
    throw "Component baseline Manifest differs from its release metadata digest."
}
$baselineInspection = @(& docker image inspect ([string]$baselineMetadata.release_identity.image_reference) | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0 -or $baselineInspection.Count -ne 1 -or
    [string]$baselineInspection[0].Id -cne [string]$baselineMetadata.release_identity.runtime_image) {
    throw "Component baseline image reference differs from its immutable release identity."
}
$baselineExecutableOutput = @(& docker run --rm --entrypoint sha256sum ([string]$baselineMetadata.release_identity.image_reference) /usr/local/bin/component-module)
if ($LASTEXITCODE -ne 0 -or [string]$baselineExecutableOutput[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+' -or
    $Matches.hash -cne [string]$baselineMetadata.release_identity.executable_sha256) {
    throw "Component baseline executable differs from its release metadata identity."
}
$candidateManifest = Get-Content -LiteralPath $candidateManifestFullPath -Raw | ConvertFrom-Json
if ([string]$candidateManifest.definition_id -cne $componentDefinition -or
    [string]$candidateManifest.release_version -cne $candidateRelease) {
    throw "Candidate Component Manifest does not identify the intended 1.0.0 release."
}
$candidateManifestDigestOutput = @(& cargo run -q -p tessara-supervisor --bin tessara-compose -- digest $candidateManifestFullPath)
if ($LASTEXITCODE -ne 0) { throw "Could not compute the candidate Component Manifest digest." }
$candidateManifestDigest = [string]($candidateManifestDigestOutput | Select-Object -Last 1)
if ($candidateManifestDigest -cnotmatch '^sha256:[0-9a-f]{64}$') { throw "Candidate Component Manifest digest is invalid." }

$runningComponentId = Get-RunningContainerId "components"
$runningInspection = @(& docker inspect $runningComponentId | ConvertFrom-Json)
$candidateImageDigest = [string]$runningInspection[0].Image
if ($candidateImageDigest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
    $candidateImageDigest -ceq [string]$baselineMetadata.release_identity.runtime_image) {
    throw "Candidate and source-built baseline must have distinct immutable image identities."
}
$candidateExecutableOutput = @(& docker exec $runningComponentId sha256sum /usr/local/bin/component-module)
if ($LASTEXITCODE -ne 0 -or [string]$candidateExecutableOutput[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
    throw "Could not capture the candidate Component executable identity."
}
$candidateExecutableSha256 = $Matches.hash
if ($candidateExecutableSha256 -ceq [string]$baselineMetadata.release_identity.executable_sha256) {
    throw "Candidate and baseline images contain the same Component executable."
}

$session = [Microsoft.PowerShell.Commands.WebRequestSession]::new()
Invoke-RestMethod -Uri "$BaseUrl/api/auth/login" -Method Post -WebSession $session -ContentType "application/json" -Body (@{
    email = $AdminEmail
    password = $AdminPassword
} | ConvertTo-Json) | Out-Null
$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
$initialSummary = Invoke-RestMethod -Uri "$BaseUrl/api/admin/composition" -Method Get -WebSession $session
$initialLockfile = $initialSummary.latest_lockfile
$initialBlueprint = $initialSummary.latest_blueprint
$initialReceipt = $initialSummary.latest_receipt
if ($null -eq $initialLockfile -or $null -eq $initialBlueprint -or $null -eq $initialReceipt) {
    throw "Component upgrade requires an already applied source-exact Sprint 8A composition."
}
$initialComponentRelease = @($initialLockfile.modules | Where-Object definition_id -CEQ $componentDefinition)
if ($initialComponentRelease.Count -ne 1 -or
    [string]$initialComponentRelease[0].version -cne $candidateRelease -or
    [string]$initialComponentRelease[0].runtime_image -cne $candidateImageDigest -or
    [string]$initialComponentRelease[0].manifest_digest -cne $candidateManifestDigest) {
    throw "The pre-exercise composition is not the intended current Component release/image/Manifest."
}
$initialBootstrapReceipts = ConvertTo-StableJson @($initialReceipt.bootstrap_receipts | Sort-Object owner)

function New-ExerciseCatalog {
    $catalog = Get-Content -LiteralPath $catalogTemplateFullPath -Raw | ConvertFrom-Json
    $catalog.revision = [uint64]$initialLockfile.blueprint_revision + 100
    $catalog.issued_at = [DateTimeOffset]::UtcNow.ToString("o")
    $catalog.core_releases[0].version = [string]$initialLockfile.core.version
    $catalog.core_releases[0].core_image = [string]$initialLockfile.core.core_image
    $catalog.core_releases[0].gateway_image = [string]$initialLockfile.core.gateway_image
    $catalog.core_releases[0].database_image = [string]$initialLockfile.core.database_image
    $catalog.core_releases[0].deployment_profile = [string]$initialLockfile.core.deployment_profile
    $catalog.core_releases[0].configuration_schema_version = [string]$initialLockfile.core.configuration_schema_version

    $componentTemplate = @($catalog.module_releases | Where-Object definition_id -CEQ $componentDefinition)
    if ($componentTemplate.Count -ne 1) { throw "Release catalog template must contain one Component release contract." }
    $candidate = Copy-JsonValue $componentTemplate[0]
    $candidate.version = $candidateRelease
    $candidate.manifest_digest = $candidateManifestDigest
    $candidate.runtime_image = $candidateImageDigest
    $baseline = Copy-JsonValue $candidate
    $baseline.version = $baselineRelease
    $baseline.manifest_digest = [string]$baselineMetadata.release_identity.manifest_digest
    $baseline.runtime_image = [string]$baselineMetadata.release_identity.runtime_image

    $otherReleases = @($catalog.module_releases | Where-Object definition_id -CNE $componentDefinition)
    foreach ($release in $otherReleases) {
        $locked = @($initialLockfile.modules | Where-Object definition_id -CEQ ([string]$release.definition_id))
        if ($locked.Count -ne 1) { throw "Initial lockfile omits catalog owner '$($release.definition_id)'." }
        $release.version = [string]$locked[0].version
        $release.manifest_digest = [string]$locked[0].manifest_digest
        $release.runtime_image = [string]$locked[0].runtime_image
        $release.deployment_profile = [string]$locked[0].deployment_profile
        $release.configuration_schema_version = [string]$locked[0].configuration_schema_version
        $release.bootstrap_schema_version = $locked[0].bootstrap_schema_version
    }
    $catalog.module_releases = @($candidate, $baseline) + $otherReleases
    return $catalog
}

function Invoke-ApiJson([string]$Path) {
    $response = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $Path -Token $token
    if ($response.status -ne 200) { throw "GET $Path returned HTTP $($response.status)." }
    return ($response.body | ConvertFrom-Json)
}

function Get-ServiceIdentity([string[]]$Services) {
    $identity = [ordered]@{}
    foreach ($service in $Services) {
        $containerId = Get-RunningContainerId $service
        $inspection = @(& docker inspect $containerId | ConvertFrom-Json)[0]
        $healthProperty = $inspection.State.PSObject.Properties['Health']
        $identity[$service] = [ordered]@{
            container_id = [string]$inspection.Id
            image_id = [string]$inspection.Image
            restart_count = [uint64]$inspection.RestartCount
            running = [bool]$inspection.State.Running
            health = if ($null -ne $healthProperty) { [string]$healthProperty.Value.Status } else { [string]$inspection.State.Status }
        }
    }
    return $identity
}

function Get-ComponentSnapshot([string]$ExpectedRelease, [string]$ExpectedImageDigest) {
    $details = @(Invoke-ApiJson "/api/admin/components") | Sort-Object component_id
    foreach ($detail in $details) { $detail.versions = @($detail.versions | Sort-Object component_version_id) }
    $module = Invoke-ApiJson "/api/admin/modules/$componentDefinition"
    $manifest = Copy-JsonValue $module.entry.manifest
    $manifest.release_version = "<release>"
    $table = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/components/sprint-8a-record-table/table" -Token $token
    $stat = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/components/sprint-8a-row-count/stat-card" -Token $token
    if ($table.status -ne 200 -or $stat.status -ne 200) { throw "Component behavior probe failed at release $ExpectedRelease." }
    $componentContainer = Get-RunningContainerId "components"
    $componentInspection = @(& docker inspect $componentContainer | ConvertFrom-Json)[0]
    if ($LASTEXITCODE -ne 0 -or [string]$componentInspection.Image -cne $ExpectedImageDigest -or
        -not [bool]$componentInspection.State.Running) {
        throw "Component runtime does not bind the expected image at release $ExpectedRelease."
    }
    $executable = @(& docker exec $componentContainer sha256sum /usr/local/bin/component-module)
    if ($LASTEXITCODE -ne 0 -or [string]$executable[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
        throw "Could not capture live Component executable at release $ExpectedRelease."
    }
    if ([string]$module.entry.release.version -cne $ExpectedRelease) {
        throw "Module Management reports Component release '$($module.entry.release.version)', expected '$ExpectedRelease'."
    }
    $componentHealthProperty = $componentInspection.State.PSObject.Properties['Health']
    return [ordered]@{
        release = [string]$module.entry.release.version
        runtime_image = [string]$module.entry.release.runtime_image
        manifest_digest = [string]$module.entry.release.manifest_digest
        executable_sha256 = $Matches.hash
        runtime_identity = [ordered]@{
            container_id = [string]$componentInspection.Id
            image_id = [string]$componentInspection.Image
            restart_count = [uint64]$componentInspection.RestartCount
            running = [bool]$componentInspection.State.Running
            health = if ($null -ne $componentHealthProperty) { [string]$componentHealthProperty.Value.Status } else { [string]$componentInspection.State.Status }
        }
        instance_id = [string]$module.entry.instance.id
        database_name = [string]$module.entry.instance.database_name
        configuration = $module.entry.configuration.values
        manifest_contract = $manifest
        product = $details
        behavior = [ordered]@{
            table = ($table.body | ConvertFrom-Json)
            stat_card = ($stat.body | ConvertFrom-Json)
        }
    }
}

function Get-UnrelatedSnapshot {
    $dashboards = @(Invoke-ApiJson "/api/dashboards") | Sort-Object id
    $dashboardDetails = @($dashboards | ForEach-Object { Invoke-ApiJson "/api/dashboards/$($_.id)" })
    $moduleDetails = @(@("tessara.dashboards", "tessara.reference.scoped-records") | ForEach-Object {
        $module = Invoke-ApiJson "/api/admin/modules/$_"
        [ordered]@{
            definition_id = [string]$module.entry.definition.id
            release = $module.entry.release
            instance_id = [string]$module.entry.instance.id
            database_name = [string]$module.entry.instance.database_name
            configuration = $module.entry.configuration.values
            enabled = [bool]$module.entry.instance.enabled
        }
    })
    $scoped = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/reference/scoped-records/api/records" -Token $token
    if ($scoped.status -ne 200) { throw "Scoped Records availability/data probe returned HTTP $($scoped.status)." }
    $gatewayReady = Invoke-Sprint8AHealthProbe -Target gateway_core -BaseUrl $BaseUrl
    $supervisorReady = Invoke-Sprint8AHealthProbe -Target supervisor -BaseUrl $SupervisorUrl
    Assert-Sprint8AHealthPassed `
        -Observation $gatewayReady `
        -Context "Component exercise gateway/Core health contract"
    Assert-Sprint8AHealthPassed `
        -Observation $supervisorReady `
        -Context "Component exercise Supervisor health contract"
    return [ordered]@{
        services = Get-ServiceIdentity @("postgres", "gateway", "core", "supervisor", "dashboards", "scoped-records")
        availability = [ordered]@{
            gateway_core = $gatewayReady
            supervisor = $supervisorReady
        }
        datasets = @(Invoke-ApiJson "/api/datasets")
        forms = @(Invoke-ApiJson "/api/forms")
        workflows = @(Invoke-ApiJson "/api/workflows")
        dashboards = $dashboardDetails
        scoped_records = ($scoped.body | ConvertFrom-Json)
        modules = $moduleDetails
        navigation = Invoke-ApiJson "/api/shell/navigation"
    }
}

function Get-StageSnapshot([string]$Stage, [string]$ExpectedRelease, [string]$ExpectedImageDigest) {
    & (Join-Path $PSScriptRoot "smoke-sprint-8a.ps1") -BaseUrl $BaseUrl -SupervisorUrl $SupervisorUrl | Out-Null
    $smokeSucceeded = $?
    if (-not $smokeSucceeded) { throw "$Stage Sprint 8A smoke failed." }
    return [ordered]@{
        stage = $Stage
        captured_at = [DateTimeOffset]::UtcNow.ToString("o")
        component = Get-ComponentSnapshot $ExpectedRelease $ExpectedImageDigest
        unrelated = Get-UnrelatedSnapshot
    }
}

function Assert-Preservation($Expected, $Actual, [string]$Stage) {
    $expectedComponent = Copy-JsonValue $Expected.component
    $actualComponent = Copy-JsonValue $Actual.component
    foreach ($snapshot in @($expectedComponent, $actualComponent)) {
        $snapshot.release = "<release>"
        $snapshot.runtime_image = "<runtime-image>"
        $snapshot.manifest_digest = "<manifest-digest>"
        $snapshot.executable_sha256 = "<executable>"
        $snapshot.runtime_identity = "<transition-specific-runtime>"
    }
    if ((ConvertTo-StableJson $expectedComponent) -cne (ConvertTo-StableJson $actualComponent)) {
        throw "$Stage changed Component data, instance identity, configuration, routes, or behavior."
    }
    if ((ConvertTo-StableJson $Expected.unrelated) -cne (ConvertTo-StableJson $Actual.unrelated)) {
        throw "$Stage changed unrelated image/container/restart identity, semantic data, navigation, or availability."
    }
}

function Invoke-ComponentReleaseTransition([string]$Stage, [string]$TargetRelease, [string]$TargetImageDigest) {
    $summary = Invoke-RestMethod -Uri "$BaseUrl/api/admin/composition" -Method Get -WebSession $session
    $blueprint = Copy-JsonValue $summary.latest_blueprint
    $blueprint.revision = [uint64]$summary.latest_blueprint.revision + 1
    $selection = @($blueprint.modules | Where-Object definition_id -CEQ $componentDefinition)
    if ($selection.Count -ne 1) { throw "$Stage Blueprint does not contain one Component selection." }
    $selection[0].version_requirement = "=$TargetRelease"
    Invoke-RestMethod -Uri "$BaseUrl/api/admin/composition/blueprints" -Method Post -WebSession $session -ContentType "application/json" -Body ($blueprint | ConvertTo-Json -Depth 100) | Out-Null
    $catalog = New-ExerciseCatalog
    $resolved = Invoke-RestMethod -Uri "$BaseUrl/api/admin/composition/blueprints/$($blueprint.revision)/resolve" -Method Post -WebSession $session -ContentType "application/json" -Body (@{ catalog = $catalog } | ConvertTo-Json -Depth 100)
    Assert-ExactDeltaPlan $resolved.lockfile $TargetImageDigest
    Invoke-RestMethod -Uri "$BaseUrl/api/admin/composition/blueprints/$($blueprint.revision)/approve" -Method Post -WebSession $session -ContentType "application/json" -Body (@{
        approved_effects = @("install", "upgrade")
        reason = "Sprint 8A Component-only $Stage"
    } | ConvertTo-Json -Depth 10) | Out-Null
    $applied = Invoke-RestMethod -Uri "$BaseUrl/api/admin/composition/blueprints/$($blueprint.revision)/apply" -Method Post -WebSession $session -ContentType "application/json" -Body "{}"
    $observedComponent = $applied.receipt.observed_artifacts.PSObject.Properties[$componentDefinition]
    if ([string]$applied.operation.state -cne "succeeded" -or
        $null -eq $observedComponent -or
        [string]$observedComponent.Value -cne $TargetImageDigest -or
        (ConvertTo-StableJson @($applied.receipt.bootstrap_receipts | Sort-Object owner)) -cne $initialBootstrapReceipts) {
        throw "$Stage apply receipt did not preserve bootstrap evidence or bind the intended Component artifact."
    }
    return [ordered]@{
        stage = $Stage
        blueprint_revision = [uint64]$blueprint.revision
        plan_digest = [string]$resolved.plan_digest
        lockfile_digest = [string]$resolved.lockfile_digest
        actions = @($resolved.lockfile.materialization_plan.actions)
        receipt_revision = [uint64]$applied.receipt.revision
        receipt_lockfile_digest = [string]$applied.receipt.lockfile_digest
        target_release = $TargetRelease
        target_image = $TargetImageDigest
    }
}

$stages = [System.Collections.Generic.List[object]]::new()
$transitions = [System.Collections.Generic.List[object]]::new()
$restoration = [ordered]@{ attempted = $false; succeeded = $false; reason = $null }
try {
    $preExercise = Get-StageSnapshot "pre-exercise-current" $candidateRelease $candidateImageDigest
    $stages.Add($preExercise)

    $transitions.Add((Invoke-ComponentReleaseTransition "establish-compatible-baseline" $baselineRelease ([string]$baselineMetadata.release_identity.runtime_image)))
    $baseline = Get-StageSnapshot "compatible-baseline" $baselineRelease ([string]$baselineMetadata.release_identity.runtime_image)
    Assert-Preservation $preExercise $baseline "compatible baseline establishment"
    $stages.Add($baseline)

    $transitions.Add((Invoke-ComponentReleaseTransition "upgrade-to-candidate" $candidateRelease $candidateImageDigest))
    $candidate = Get-StageSnapshot "candidate-upgrade" $candidateRelease $candidateImageDigest
    Assert-Preservation $preExercise $candidate "candidate upgrade"
    $stages.Add($candidate)

    $transitions.Add((Invoke-ComponentReleaseTransition "rollback-to-baseline" $baselineRelease ([string]$baselineMetadata.release_identity.runtime_image)))
    $rollback = Get-StageSnapshot "baseline-rollback" $baselineRelease ([string]$baselineMetadata.release_identity.runtime_image)
    Assert-Preservation $preExercise $rollback "baseline rollback"
    $stages.Add($rollback)

    $transitions.Add((Invoke-ComponentReleaseTransition "restore-intended-candidate" $candidateRelease $candidateImageDigest))
    $restored = Get-StageSnapshot "candidate-restored" $candidateRelease $candidateImageDigest
    Assert-Preservation $preExercise $restored "intended current release restoration"
    $stages.Add($restored)
    $restoration.succeeded = $true

    $result = [ordered]@{
        schema_version = 2
        evidence_kind = "tessara.sprint-8a.component-upgrade-rollback"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_identity = $source
        project = $expectedProject
        release_fixture = [ordered]@{
            baseline = $baselineMetadata.release_identity
            candidate = [ordered]@{
                definition_id = $componentDefinition
                version = $candidateRelease
                manifest_digest = $candidateManifestDigest
                runtime_image = $candidateImageDigest
                executable_sha256 = $candidateExecutableSha256
            }
        }
        transitions = @($transitions)
        stage_snapshots = @($stages)
        preservation = [ordered]@{
            component_data_identity_configuration_routes_behavior = "exact"
            unrelated_container_image_restart_data_availability = "exact"
            bootstrap_receipts = "carried_forward_exactly"
            final_release = $candidateRelease
        }
        passed = $true
    }
    Publish-Sprint7AEvidence -Document $result -OutputPath $outputFullPath | Out-Null
    $result | ConvertTo-Json -Depth 100
} catch {
    $failure = $_.Exception.Message
    try {
        $projectedModule = Invoke-ApiJson "/api/admin/modules/$componentDefinition"
        $projectedRelease = [string]$projectedModule.entry.release.version
        $liveContainer = Get-RunningContainerId "components"
        $liveManifestOutput = @(& docker exec $liveContainer curl -fsS `
            -H "x-tessara-module-control-key: $moduleControlKey" `
            http://127.0.0.1:8092/api/manifest)
        if ($LASTEXITCODE -ne 0 -or $liveManifestOutput.Count -eq 0) {
            throw "Could not inspect the live Component Manifest during failure restoration."
        }
        $liveManifest = ($liveManifestOutput -join "`n") | ConvertFrom-Json
        $liveRelease = [string]$liveManifest.release_version
        if ($projectedRelease -ceq $candidateRelease -and $liveRelease -cne $candidateRelease) {
            $restoration.attempted = $true
            [void](Invoke-ComponentReleaseTransition "failure-reconcile-baseline" $baselineRelease ([string]$baselineMetadata.release_identity.runtime_image))
            $projectedRelease = $baselineRelease
        }
        if ($projectedRelease -cne $candidateRelease -or $liveRelease -cne $candidateRelease) {
            $restoration.attempted = $true
            [void](Invoke-ComponentReleaseTransition "failure-restoration" $candidateRelease $candidateImageDigest)
        }
        $restoration.succeeded = $true
    } catch {
        $restoration.reason = $_.Exception.Message
    }
    throw "Component upgrade/rollback failed: $failure Restoration: $(ConvertTo-StableJson $restoration)"
} finally {
    if ($token) {
        [void](Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token)
    }
}

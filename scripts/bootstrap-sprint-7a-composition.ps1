[CmdletBinding(SupportsShouldProcess)]
param(
    [ValidateSet("reference", "reduced")]
    [string]$Composition = "reference",
    [string]$ComposeFile = "deploy/sprint-7a/compose.yaml",
    [string]$CoreUrl = "http://127.0.0.1:8086",
    [string]$SupervisorUrl = "http://127.0.0.1:8096",
    [string]$ResolvedCompositionEnvelope,
    [string]$ReleaseCatalogEnvelope,
    [string]$BlueprintPath,
    [string]$RuntimeDirectory,
    [switch]$SkipBuild,
    [switch]$ReplaceExisting,
    [string]$DeploymentDirectory = "sprint-7a",
    [string]$ExpectedProject = "tessara-sprint-7a",
    [string]$InstallationId = "01980000-0000-7000-8000-00000000007a",
    [string]$RuntimeLabel = "sprint-7a",
    [string[]]$AdditionalBuildServices = @(),
    [string[]]$AdditionalExpectedNavigationHrefs = @(),
    [switch]$SkipLegacySeed,
    [switch]$UseCoreApplyAuthorization,
    [switch]$SemanticNoOp,
    [switch]$ExcludePublicGateway,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

function Resolve-RepositoryPath {
    param([Parameter(Mandatory)][string]$Path)
    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

$composePath = Resolve-RepositoryPath -Path $ComposeFile
$expectedProject = $ExpectedProject
$installationId = $InstallationId
$runtimeDirectory = if ([string]::IsNullOrWhiteSpace($RuntimeDirectory)) {
    Join-Path $repoRoot "target/$RuntimeLabel-bootstrap/$Composition"
} elseif ([IO.Path]::IsPathRooted($RuntimeDirectory)) {
    [IO.Path]::GetFullPath($RuntimeDirectory)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $RuntimeDirectory))
}
$resolvedBlueprintPath = if ([string]::IsNullOrWhiteSpace($BlueprintPath)) {
    Join-Path $repoRoot "deploy/$DeploymentDirectory/blueprints/$Composition.json"
} elseif ([IO.Path]::IsPathRooted($BlueprintPath)) {
    [IO.Path]::GetFullPath($BlueprintPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $BlueprintPath))
}
$catalogTemplatePath = Join-Path $repoRoot "deploy/$DeploymentDirectory/catalogs/local-release-catalog.json"
$catalogPayloadPath = Join-Path $runtimeDirectory "release-catalog.json"
$catalogPath = Join-Path $runtimeDirectory "release-catalog.signed.json"
$catalogKeyPath = Join-Path $repoRoot "deploy/$DeploymentDirectory/catalogs/catalog-dev-v1.public.hex"
$lockfilePath = Join-Path $runtimeDirectory "lockfile.json"
$authorizationPath = Join-Path $runtimeDirectory "authorization.json"
$signedAuthorizationPath = Join-Path $runtimeDirectory "authorization.signed.json"
$receiptPath = Join-Path $runtimeDirectory "apply-response.json"
$processEnvironmentVariableNames = @(
    "TESSARA_SOURCE_COMMIT",
    "TESSARA_SOURCE_TREE",
    "TESSARA_SOURCE_DIRTY",
    "TESSARA_INSTALLATION_ID",
    "TESSARA_SIGNING_ISSUER",
    "TESSARA_SIGNING_KEY_ID",
    "TESSARA_SIGNING_SECRET_HEX"
)

function Assert-Sprint7ACatalogSourceContract {
    if (-not (Test-Path -LiteralPath $catalogTemplatePath -PathType Leaf)) {
        throw "Release catalog template not found: $catalogTemplatePath"
    }
    if (-not (Test-Path -LiteralPath $catalogKeyPath -PathType Leaf)) {
        throw "Release catalog public key not found: $catalogKeyPath"
    }
    $catalogPublicKey = (Get-Content -LiteralPath $catalogKeyPath -Raw).Trim()
    if ($catalogPublicKey -cnotmatch '^[0-9a-f]{64}$') {
        throw "Release catalog public key must be one exact lowercase Ed25519 key."
    }
}

function Get-Sprint7AProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][string[]]$Names)

    $current = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Process)
    $snapshot = [ordered]@{}
    foreach ($name in $Names) {
        $snapshot[$name] = [pscustomobject][ordered]@{
            present = $current.Contains($name)
            value = [Environment]::GetEnvironmentVariable($name, [EnvironmentVariableTarget]::Process)
        }
    }
    $snapshot
}

function Restore-Sprint7AProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)]$Snapshot)

    foreach ($name in @($Snapshot.Keys)) {
        $entry = $Snapshot[$name]
        if ([bool]$entry.present) {
            [Environment]::SetEnvironmentVariable(
                [string]$name,
                [string]$entry.value,
                [EnvironmentVariableTarget]::Process
            )
        } else {
            Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
        }
    }
}

function ConvertTo-Sprint7ABootstrapDateTimeOffset {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -is [DateTimeOffset]) {
        return $Value
    }
    if ($Value -is [DateTime]) {
        if ($Value.Kind -eq [DateTimeKind]::Unspecified) {
            throw "$Label has no UTC offset."
        }
        return [DateTimeOffset]::new($Value)
    }
    $text = [string]$Value
    if ($text -notmatch '(?:Z|[+-]\d{2}:\d{2})$') {
        throw "$Label has no UTC offset."
    }
    $parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse(
        $text,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$parsed
    )) {
        throw "$Label is not a valid round-trip timestamp."
    }
    $parsed
}

function Get-Sprint7AApprovedEffects {
    param([AllowEmptyCollection()][object[]]$Actions = @())

    @($Actions | ForEach-Object {
        $action = $_
        switch ([string]$action.action) {
            "acquire_image" { "install" }
            "provision_database" { "install" }
            "migrate" { "upgrade" }
            "switch_traffic" { "upgrade" }
            "configure" { "configure" }
            "bootstrap" { "bootstrap" }
            "set_enablement" { if ([bool]$action.enabled) { "enable" } else { "disable" } }
        }
    } | Sort-Object -Unique)
}

function Get-Sprint7ARuntimeServiceName {
    param([Parameter(Mandatory)][string]$DefinitionId)

    switch -CaseSensitive ($DefinitionId) {
        "tessara.reference.scoped-records" { "scoped-records"; break }
        "tessara.datasets" { "datasets"; break }
        "tessara.components" { "components"; break }
        "tessara.dashboards" { "dashboards"; break }
        default { $null }
    }
}

function Test-Sprint7ABootstrapHelpers {
    Assert-Sprint7ACatalogSourceContract
    $effects = @(Get-Sprint7AApprovedEffects -Actions @(
        [pscustomobject]@{ action = "acquire_image" },
        [pscustomobject]@{ action = "provision_database" },
        [pscustomobject]@{ action = "migrate" },
        [pscustomobject]@{ action = "switch_traffic" },
        [pscustomobject]@{ action = "configure" },
        [pscustomobject]@{ action = "bootstrap" },
        [pscustomobject]@{ action = "set_enablement"; enabled = $true },
        [pscustomobject]@{ action = "set_enablement"; enabled = $false }
    ))
    $expectedEffects = @("bootstrap", "configure", "disable", "enable", "install", "upgrade")
    if (($effects | ConvertTo-Json -Compress) -cne ($expectedEffects | ConvertTo-Json -Compress)) {
        throw "Sprint 7A approved-effect projection self-test failed."
    }
    $expectedRuntimeServices = [ordered]@{
        "tessara.reference.scoped-records" = "scoped-records"
        "tessara.datasets" = "datasets"
        "tessara.components" = "components"
        "tessara.dashboards" = "dashboards"
    }
    foreach ($definitionId in $expectedRuntimeServices.Keys) {
        if ((Get-Sprint7ARuntimeServiceName -DefinitionId $definitionId) -cne
            [string]$expectedRuntimeServices[$definitionId]) {
            throw "Sprint 7A runtime image service projection self-test failed for '$definitionId'."
        }
    }
    if ($null -ne (Get-Sprint7ARuntimeServiceName -DefinitionId "tessara.unknown")) {
        throw "Sprint 7A runtime image service projection accepted an unknown module definition."
    }

    $roundTripTimestamp = (@{ expires_at = "2031-02-03T04:05:06.1234567+00:00" } |
        ConvertTo-Json |
        ConvertFrom-Json).expires_at
    $parsedTimestamp = ConvertTo-Sprint7ABootstrapDateTimeOffset `
        -Value $roundTripTimestamp `
        -Label "Bootstrap timestamp self-test"
    if ($parsedTimestamp.UtcDateTime.Ticks -ne
        [DateTimeOffset]::Parse("2031-02-03T04:05:06.1234567+00:00").UtcDateTime.Ticks) {
        throw "Sprint 7A bootstrap timestamp self-test failed."
    }
    try {
        ConvertTo-Sprint7ABootstrapDateTimeOffset `
            -Value "2031-02-03T04:05:06" `
            -Label "Bootstrap offsetless timestamp self-test" | Out-Null
        throw "Sprint 7A bootstrap timestamp self-test accepted an offsetless value."
    } catch {
        if ($_.Exception.Message -notmatch "has no UTC offset") { throw }
    }

    $original = Get-Sprint7AProcessEnvironmentSnapshot -Names $processEnvironmentVariableNames
    try {
        foreach ($name in $processEnvironmentVariableNames) {
            Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
        }
        [Environment]::SetEnvironmentVariable(
            "TESSARA_SOURCE_COMMIT",
            "caller-value",
            [EnvironmentVariableTarget]::Process
        )
        $caller = Get-Sprint7AProcessEnvironmentSnapshot -Names $processEnvironmentVariableNames
        [Environment]::SetEnvironmentVariable(
            "TESSARA_SOURCE_COMMIT",
            "bootstrap-value",
            [EnvironmentVariableTarget]::Process
        )
        [Environment]::SetEnvironmentVariable(
            "TESSARA_SOURCE_TREE",
            "introduced-value",
            [EnvironmentVariableTarget]::Process
        )
        Restore-Sprint7AProcessEnvironmentSnapshot -Snapshot $caller
        $restored = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Process)
        if ([Environment]::GetEnvironmentVariable("TESSARA_SOURCE_COMMIT", [EnvironmentVariableTarget]::Process) -cne "caller-value" -or
            $restored.Contains("TESSARA_SOURCE_TREE")) {
            throw "Sprint 7A caller process-environment restoration self-test failed."
        }
    } finally {
        Restore-Sprint7AProcessEnvironmentSnapshot -Snapshot $original
    }

    $relativeComposePath = "deploy/sprint-7a/compose.yaml"
    $expectedRelativeComposePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $relativeComposePath))
    if ((Resolve-RepositoryPath -Path $relativeComposePath) -cne $expectedRelativeComposePath) {
        throw "Sprint 7A repository-relative Compose path resolution self-test failed."
    }
    $absoluteComposePath = [IO.Path]::GetFullPath((Join-Path ([IO.Path]::GetTempPath()) "tessara-compose-absolute.yaml"))
    if ((Resolve-RepositoryPath -Path $absoluteComposePath) -cne $absoluteComposePath) {
        throw "Sprint 7A absolute Compose path resolution self-test failed."
    }

    Write-Host "Sprint 7A bootstrap action projection, path resolution, and caller-environment restoration self-test passed."
}

function Prepare-Sprint7AUatFixtures {
    if ($Composition -ne "reference" -or $SkipLegacySeed) { return }
    & (Join-Path $PSScriptRoot "prepare-sprint-7a-uat-fixtures.ps1") `
        -BaseUrl $CoreUrl `
        -AdminEmail "admin@tessara.local" `
        -AdminPassword "tessara-dev-admin" `
        -ComposeProject $expectedProject `
        -OwnerControlledSeed:($RuntimeLabel -ceq "sprint-8a") | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "$RuntimeLabel semantic UAT fixture preparation failed."
    }
}

if ($SelfTest) {
    Test-Sprint7ABootstrapHelpers
    return
}

if (-not (Test-Path -LiteralPath $composePath)) { throw "Compose file not found: $composePath" }
if (-not (Test-Path -LiteralPath $resolvedBlueprintPath -PathType Leaf)) {
    throw "Blueprint not found: $resolvedBlueprintPath"
}
Assert-Sprint7ACatalogSourceContract
[IO.Directory]::CreateDirectory($runtimeDirectory) | Out-Null
$processEnvironmentSnapshot = Get-Sprint7AProcessEnvironmentSnapshot -Names $processEnvironmentVariableNames

Push-Location $repoRoot
try {
    $composeConfiguration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
    $configuredProject = $composeConfiguration.name
    if ($configuredProject -ne $expectedProject) {
        throw "Refusing to operate on unexpected Compose project '$configuredProject'."
    }

    if ($ReplaceExisting) {
        if ($PSCmdlet.ShouldProcess(
            "$expectedProject containers and named volumes",
            "Remove the fresh $RuntimeLabel disposable installation state"
        )) {
            & docker compose -f $composePath --profile reference down --volumes --remove-orphans
            if ($LASTEXITCODE -ne 0) { throw "$RuntimeLabel Compose teardown failed." }
        }
    }

    $sourceCommit = (& git rev-parse HEAD).Trim()
    $sourceTree = (& git write-tree).Trim()
    $sourceDirty = if ([string]::IsNullOrWhiteSpace((& git status --porcelain))) { "false" } else { "true" }
    $env:TESSARA_SOURCE_COMMIT = $sourceCommit
    $env:TESSARA_SOURCE_TREE = $sourceTree
    $env:TESSARA_SOURCE_DIRTY = $sourceDirty
    $env:TESSARA_INSTALLATION_ID = $installationId

    $composeArguments = @("compose", "-f", $composePath)
    if ($Composition -eq "reference") { $composeArguments += @("--profile", "reference") }
    $buildServices = @("supervisor", "core", "scoped-records", "dashboards") + $AdditionalBuildServices
    if (-not $SkipBuild) {
        foreach ($service in $buildServices) {
            & docker @composeArguments build $service
            if ($LASTEXITCODE -ne 0) { throw "$RuntimeLabel $service image build failed." }
        }
    }
    if ($ExcludePublicGateway) {
        $startupServices = @($composeConfiguration.services.PSObject.Properties.Name |
            Where-Object { [string]$_ -cne "gateway" } |
            Sort-Object)
        if ($startupServices.Count -eq 0) {
            throw "$RuntimeLabel did not resolve any non-gateway startup services."
        }
        & docker @composeArguments up -d --no-build @startupServices
    } else {
        & docker @composeArguments up -d --no-build
    }
    if ($LASTEXITCODE -ne 0) { throw "$RuntimeLabel service startup failed." }
    if ($ExcludePublicGateway) {
        $runningGateway = @(& docker @composeArguments ps --status running -q gateway 2>&1 |
            ForEach-Object { ([string]$_).Trim() } |
            Where-Object { $_ })
        if ($LASTEXITCODE -ne 0 -or $runningGateway.Count -ne 0) {
            throw "$RuntimeLabel public gateway must remain stopped during owner materialization."
        }
    }

    $coreSession = [Microsoft.PowerShell.Commands.WebRequestSession]::new()
    $coreReady = $false
    for ($attempt = 1; $attempt -le 60; $attempt++) {
        try {
            Invoke-RestMethod `
                -Uri "$CoreUrl/api/auth/login" `
                -Method Post `
                -WebSession $coreSession `
                -ContentType "application/json" `
                -Body (@{
                    email = "admin@tessara.local"
                    password = "tessara-dev-admin"
                } | ConvertTo-Json) | Out-Null
            $coreReady = $true
            break
        } catch {
            if ($attempt -eq 60) { throw }
            Start-Sleep -Seconds 1
        }
    }
    if (-not $coreReady) { throw "$RuntimeLabel Core did not become ready." }

    # The reference acceptance suite builds on the established UAT demo data.
    # Seed through Core's owning API boundary. Sprint 8A binds that boundary to
    # a loopback-only control port so public ingress can remain unavailable.
    # A no-op rerun skips seeding only when the expected fixture is present.
    if ($Composition -eq "reference" -and -not $SkipLegacySeed) {
        $nodeTypes = Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/node-types" `
            -Method Get `
            -WebSession $coreSession
        if (-not ($nodeTypes | Where-Object slug -eq "activity")) {
            Invoke-RestMethod `
                -Uri "$CoreUrl/api/demo/seed" `
                -Method Post `
                -WebSession $coreSession `
                -ContentType "application/json" `
                -Body "{}" | Out-Null
        }
    }

    function Get-ImageDigest([string]$Image) {
        $digest = (& docker image inspect --format "{{.Id}}" $Image).Trim()
        if ($LASTEXITCODE -ne 0 -or $digest -notmatch '^sha256:[0-9a-f]{64}$') {
            throw "Could not determine immutable image identity for $Image."
        }
        return $digest
    }
    function Get-ConfiguredServiceImage([string]$Service) {
        $serviceConfiguration = $composeConfiguration.services.PSObject.Properties[$Service]
        if ($null -eq $serviceConfiguration -or
            [string]::IsNullOrWhiteSpace([string]$serviceConfiguration.Value.image)) {
            throw "Compose service '$Service' does not declare an image identity."
        }
        return [string]$serviceConfiguration.Value.image
    }
    $env:TESSARA_SIGNING_ISSUER = "tessara.local.$RuntimeLabel"
    $env:TESSARA_SIGNING_KEY_ID = "catalog-dev-v1"
    if ([string]::IsNullOrWhiteSpace($env:TESSARA_SIGNING_SECRET_HEX)) {
        $env:TESSARA_SIGNING_SECRET_HEX = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    }

    if ([string]::IsNullOrWhiteSpace($ReleaseCatalogEnvelope)) {
        $catalog = Get-Content -LiteralPath $catalogTemplatePath -Raw | ConvertFrom-Json
        $catalog.issued_at = [DateTimeOffset]::UtcNow.ToString("o")
        $catalog.core_releases[0].core_image = Get-ImageDigest (Get-ConfiguredServiceImage "core")
        $catalog.core_releases[0].gateway_image = Get-ImageDigest (Get-ConfiguredServiceImage "gateway")
        $catalog.core_releases[0].database_image = Get-ImageDigest (Get-ConfiguredServiceImage "postgres")
        foreach ($moduleRelease in @($catalog.module_releases)) {
            $runtimeService = Get-Sprint7ARuntimeServiceName -DefinitionId ([string]$moduleRelease.definition_id)
            if (-not [string]::IsNullOrWhiteSpace($runtimeService)) {
                $moduleRelease.runtime_image = Get-ImageDigest (Get-ConfiguredServiceImage $runtimeService)
            }
        }
        [IO.File]::WriteAllText($catalogPayloadPath, ($catalog | ConvertTo-Json -Depth 100) + "`n", [Text.UTF8Encoding]::new($false))
        & cargo run -q -p tessara-supervisor --bin tessara-compose -- catalog-sign $catalogPayloadPath $catalogPath
        if ($LASTEXITCODE -ne 0) { throw "Runtime release catalog signing failed." }
    } else {
        $catalogPath = Resolve-RepositoryPath $ReleaseCatalogEnvelope
        if (-not (Test-Path -LiteralPath $catalogPath)) {
            throw "Signed release catalog not found: $catalogPath"
        }
        $catalogEnvelope = Get-Content -LiteralPath $catalogPath -Raw | ConvertFrom-Json
        $catalog = $catalogEnvelope.payload
    }

    & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
        catalog-verify $catalogPath $catalogKeyPath
    if ($LASTEXITCODE -ne 0) { throw "Signed release catalog verification failed." }
    if ($SemanticNoOp) {
        if (-not [string]::IsNullOrWhiteSpace($ResolvedCompositionEnvelope)) {
            throw "Semantic no-op resolution cannot use a detached full-plan envelope."
        }
        $currentComposition = Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/composition" `
            -Method Get `
            -WebSession $coreSession
        if ($null -eq $currentComposition.latest_blueprint -or $null -eq $currentComposition.latest_receipt) {
            throw "Semantic no-op resolution requires one successfully applied current composition."
        }
        $noOpBlueprint = Get-Content -LiteralPath $resolvedBlueprintPath -Raw | ConvertFrom-Json
        $noOpBlueprint.revision = [uint64]$currentComposition.latest_blueprint.revision + 1
        Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/composition/blueprints" `
            -Method Post `
            -WebSession $coreSession `
            -ContentType "application/json" `
            -Body ($noOpBlueprint | ConvertTo-Json -Depth 100) | Out-Null
        $noOpResolved = Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/composition/blueprints/$($noOpBlueprint.revision)/resolve" `
            -Method Post `
            -WebSession $coreSession `
            -ContentType "application/json" `
            -Body (@{ catalog = $catalog } | ConvertTo-Json -Depth 100)
        $noOpActions = @($noOpResolved.lockfile.materialization_plan.actions)
        if ($noOpActions.Count -ne 1 -or [string]$noOpActions[0].action -cne "verify_read_back") {
            throw "Unchanged desired state did not resolve to the exact semantic no-op read-back plan."
        }
        [IO.File]::WriteAllText(
            $lockfilePath,
            ($noOpResolved.lockfile | ConvertTo-Json -Depth 100) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
    } elseif ([string]::IsNullOrWhiteSpace($ResolvedCompositionEnvelope)) {
        & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
            resolve $resolvedBlueprintPath $catalogPath $catalogKeyPath $lockfilePath
        if ($LASTEXITCODE -ne 0) { throw "Blueprint resolution failed." }
    } else {
        if ([string]::IsNullOrWhiteSpace($ReleaseCatalogEnvelope)) {
            throw "Detached bootstrap requires -ReleaseCatalogEnvelope so Core resolves the exact signed catalog digest."
        }
        $resolvedPath = Resolve-RepositoryPath $ResolvedCompositionEnvelope
        & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
            resolved-verify $resolvedPath $catalogKeyPath $lockfilePath
        if ($LASTEXITCODE -ne 0) { throw "Detached resolved composition verification failed." }
    }

    $lockfile = Get-Content -LiteralPath $lockfilePath -Raw | ConvertFrom-Json
    $approvedEffects = @(Get-Sprint7AApprovedEffects -Actions @($lockfile.materialization_plan.actions))

    # Persist the same desired state and explicit approval through Core before
    # the authorized Supervisor apply. New owner-bootstrap protocols use the
    # authenticated Core apply boundary so the signed initiator is the exact
    # account UUID; the legacy detached CLI path remains available to the
    # historical Sprint 7A harness.
    $compositionSummary = Invoke-RestMethod `
        -Uri "$CoreUrl/api/admin/composition" `
        -Method Get `
        -WebSession $coreSession
    $projectedPlanDigest = $null
    if ($null -ne $compositionSummary.latest_lockfile) {
        $projectedPlanDigest = $compositionSummary.latest_lockfile.materialization_plan_digest
    }
    $resolveAndApprove = $false
    if ($SemanticNoOp) {
        $resolveAndApprove = $true
    } elseif ($projectedPlanDigest -ne $lockfile.materialization_plan_digest) {
        if ($null -ne $compositionSummary.latest_blueprint) {
            throw "Core already contains a different Blueprint; use -ReplaceExisting for a fresh $RuntimeLabel installation."
        }
        $blueprintJson = Get-Content -LiteralPath $resolvedBlueprintPath -Raw
        Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/composition/blueprints" `
            -Method Post `
            -WebSession $coreSession `
            -ContentType "application/json" `
            -Body $blueprintJson | Out-Null
        $resolveAndApprove = $true
    } elseif ($compositionSummary.latest_lockfile.catalog_digest -ne $lockfile.catalog_digest) {
        # A freshly signed runtime catalog can produce a source-distinct
        # lockfile while retaining the same materialization plan. Core must
        # still persist and approve that exact lockfile before Supervisor can
        # project its receipt.
        $resolveAndApprove = $true
    } elseif ($compositionSummary.latest_approval.plan_digest -ne $lockfile.materialization_plan_digest) {
        throw "Core has the expected resolved plan without its matching explicit approval."
    }

    if ($resolveAndApprove) {
        $resolved = Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/composition/blueprints/$($lockfile.blueprint_revision)/resolve" `
            -Method Post `
            -WebSession $coreSession `
            -ContentType "application/json" `
            -Body (@{ catalog = $catalog } | ConvertTo-Json -Depth 100)
        if ($resolved.plan_digest -ne $lockfile.materialization_plan_digest) {
            throw "Core resolved a different materialization plan than the verified CLI lockfile."
        }
        $cliLockfileDigest = (& cargo run -q -p tessara-supervisor --bin tessara-compose -- digest $lockfilePath).Trim()
        if ($LASTEXITCODE -ne 0 -or $resolved.lockfile_digest -ne $cliLockfileDigest) {
            throw "Core resolved a different lockfile than the verified CLI lockfile."
        }
        Invoke-RestMethod `
            -Uri "$CoreUrl/api/admin/composition/blueprints/$($lockfile.blueprint_revision)/approve" `
            -Method Post `
            -WebSession $coreSession `
            -ContentType "application/json" `
            -Body (@{
                approved_effects = $approvedEffects
                reason = "$RuntimeLabel $Composition reference materialization"
            } | ConvertTo-Json -Depth 20) | Out-Null
    }

    if ($UseCoreApplyAuthorization) {
        $coreApplyResponse = Invoke-WebRequest `
            -Uri "$CoreUrl/api/admin/composition/blueprints/$($lockfile.blueprint_revision)/apply" `
            -Method Post `
            -WebSession $coreSession `
            -SkipHttpErrorCheck
        $response = @([string]$coreApplyResponse.Content)
        $applyExitCode = if ([int]$coreApplyResponse.StatusCode -ge 200 -and
            [int]$coreApplyResponse.StatusCode -lt 300) { 0 } else { 1 }
    } else {
        $now = [DateTimeOffset]::UtcNow
        if ((Test-Path -LiteralPath $signedAuthorizationPath) -and -not (Test-Path -LiteralPath $receiptPath)) {
        $pendingAuthorization = Get-Content -LiteralPath $signedAuthorizationPath -Raw | ConvertFrom-Json
        if ($pendingAuthorization.payload.target_plan_digest -eq $lockfile.materialization_plan_digest -and `
            (ConvertTo-Sprint7ABootstrapDateTimeOffset `
                -Value $pendingAuthorization.payload.expires_at `
                -Label "Pending authorization expiry") -gt $now) {
            $recoveredResponse = & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
                apply $SupervisorUrl $lockfilePath $signedAuthorizationPath
            if ($LASTEXITCODE -eq 0) {
                [IO.File]::WriteAllLines($receiptPath, $recoveredResponse, [Text.UTF8Encoding]::new($false))
                Prepare-Sprint7AUatFixtures
                Write-Host "Recovered the accepted $RuntimeLabel operation with its original signed authorization."
                Write-Host "Receipt: $receiptPath"
                return
            }
        }
        }
        $baseReceiptDigest = $null
        $applySequence = [uint64]1
        try {
        $currentReceipt = Invoke-RestMethod -Uri "$SupervisorUrl/v1/receipts/current" -Method Get
        $currentReceiptPath = Join-Path $runtimeDirectory "receipt-current.json"
        [IO.File]::WriteAllText(
            $currentReceiptPath,
            ($currentReceipt | ConvertTo-Json -Depth 100) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
        $baseReceiptDigest = (& cargo run -q -p tessara-supervisor --bin tessara-compose -- digest $currentReceiptPath).Trim()
        $applySequence = [uint64]$currentReceipt.revision + 1
        } catch {
            if ($_.Exception.Response.StatusCode.value__ -ne 404) { throw }
        }
        $reuseAuthorization = $false
        if (Test-Path -LiteralPath $signedAuthorizationPath) {
        $existingAuthorization = Get-Content -LiteralPath $signedAuthorizationPath -Raw | ConvertFrom-Json
        $existingBase = $existingAuthorization.payload.base_receipt_digest
        $baseMatches = ($null -eq $existingBase -and $null -eq $baseReceiptDigest) -or `
            ($null -ne $existingBase -and $null -ne $baseReceiptDigest -and $existingBase -eq $baseReceiptDigest)
        $reuseAuthorization = $existingAuthorization.payload.target_plan_digest -eq $lockfile.materialization_plan_digest -and `
            [uint64]$existingAuthorization.payload.desired_revision -eq [uint64]$lockfile.blueprint_revision -and `
            [uint64]$existingAuthorization.payload.apply_sequence -eq $applySequence -and `
            $baseMatches -and `
            (ConvertTo-Sprint7ABootstrapDateTimeOffset `
                -Value $existingAuthorization.payload.expires_at `
                -Label "Existing authorization expiry") -gt $now.AddMinutes(1)
        }
        if (-not $reuseAuthorization) {
        $authorization = [ordered]@{
        api_version = "tessara.io/apply-authorization/v1"
        operation = "materialize"
        installation_id = $installationId
        base_receipt_digest = $baseReceiptDigest
        target_plan_digest = $lockfile.materialization_plan_digest
        desired_revision = [uint64]$lockfile.blueprint_revision
        apply_sequence = $applySequence
        nonce = [Guid]::NewGuid().ToString()
        idempotency_key = "$RuntimeLabel-$Composition-r$($lockfile.blueprint_revision)-a$applySequence"
        initiator = [ordered]@{ actor_id = "local:$RuntimeLabel-bootstrap"; actor_kind = "operator"; authority = "local-cli" }
        approver = [ordered]@{ actor_id = "local:$RuntimeLabel-approver"; actor_kind = "operator"; authority = "composition:approve" }
        issued_at = $now.ToString("o")
        expires_at = $now.AddMinutes(10).ToString("o")
        approved_effects = $approvedEffects
        reason = "$RuntimeLabel $Composition reference materialization"
        }
        [IO.File]::WriteAllText(
            $authorizationPath,
            ($authorization | ConvertTo-Json -Depth 20) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
        $env:TESSARA_SIGNING_ISSUER = "tessara.local.$RuntimeLabel"
        $env:TESSARA_SIGNING_KEY_ID = "apply-dev-v1"
        if ([string]::IsNullOrWhiteSpace($env:TESSARA_SIGNING_SECRET_HEX)) {
            $env:TESSARA_SIGNING_SECRET_HEX = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
        }
        & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
            authorization-sign $authorizationPath $signedAuthorizationPath
        if ($LASTEXITCODE -ne 0) { throw "Apply authorization signing failed." }
        }

        $response = @(& cargo run -q -p tessara-supervisor --bin tessara-compose -- `
            apply $SupervisorUrl $lockfilePath $signedAuthorizationPath 2>&1 | ForEach-Object { [string]$_ })
        $applyExitCode = $LASTEXITCODE
    }
    if ($applyExitCode -ne 0) {
        $failureResponsePath = Join-Path $runtimeDirectory "apply-failure-response.log"
        [IO.File]::WriteAllLines($failureResponsePath, $response, [Text.UTF8Encoding]::new($false))
        try {
            $configuredServiceNames = @($composeConfiguration.services.PSObject.Properties.Name)
            $failureLogServices = @(
                "datasets",
                "response-provider-proxy",
                "form-provider-proxy",
                "supervisor",
                "core"
            ) | Where-Object { $configuredServiceNames -ccontains $_ }
            if ($failureLogServices.Count -gt 0) {
                $failureServiceLogs = @(& docker @composeArguments logs `
                    --no-color --timestamps @failureLogServices 2>&1 |
                    ForEach-Object { [string]$_ })
                $failureServiceLogsPath = Join-Path $runtimeDirectory `
                    "composition-failure-service-logs.log"
                [IO.File]::WriteAllLines(
                    $failureServiceLogsPath,
                    $failureServiceLogs,
                    [Text.UTF8Encoding]::new($false)
                )
            }
        } catch {
            # Apply output remains authoritative. Service logs are retained as
            # bounded diagnostic evidence when Compose can provide them.
        }
        try {
            $failureSummary = Invoke-RestMethod `
                -Uri "$CoreUrl/api/admin/composition" `
                -Method Get `
                -WebSession $coreSession
            $failureSummaryPath = Join-Path $runtimeDirectory "composition-failure-summary.json"
            [IO.File]::WriteAllText(
                $failureSummaryPath,
                ($failureSummary | ConvertTo-Json -Depth 100) + "`n",
                [Text.UTF8Encoding]::new($false)
            )
        } catch {
            # The raw apply failure remains authoritative when Core itself is
            # unavailable. Do not replace it with a diagnostic-capture error.
        }
        throw "Supervisor apply failed. Raw response: $failureResponsePath"
    }
    [IO.File]::WriteAllLines($receiptPath, $response, [Text.UTF8Encoding]::new($false))

    $navigationReady = $false
    for ($attempt = 1; $attempt -le 30; $attempt++) {
        try {
            $navigation = Invoke-RestMethod `
                -Uri "$CoreUrl/api/shell/navigation" `
                -Method Get `
                -WebSession $coreSession
            $navigationItems = @($navigation.groups | ForEach-Object { $_.items })
            $hasComposition = $navigationItems | Where-Object href -eq "/administration/composition"
            $hasReferenceModules = $Composition -ne "reference" -or (
                ($navigationItems | Where-Object href -eq "/dashboards") -and
                ($navigationItems | Where-Object href -eq "/forms")
            )
            $hasAdditionalNavigation = @($AdditionalExpectedNavigationHrefs | Where-Object {
                $expectedHref = $_
                -not ($navigationItems | Where-Object href -eq $expectedHref)
            }).Count -eq 0
            if ($navigation.state -eq "available" -and $hasComposition -and $hasReferenceModules -and $hasAdditionalNavigation) {
                $navigationReady = $true
                break
            }
        } catch {
            if ($attempt -eq 30) { throw }
        }
        Start-Sleep -Seconds 1
    }
    if (-not $navigationReady) {
        throw "$RuntimeLabel shell navigation did not reach the expected post-apply state."
    }

    Prepare-Sprint7AUatFixtures

    Write-Host "$RuntimeLabel $Composition composition materialized."
    Write-Host "Receipt: $receiptPath"
} finally {
    try {
        Pop-Location
    } finally {
        Restore-Sprint7AProcessEnvironmentSnapshot -Snapshot $processEnvironmentSnapshot
    }
}

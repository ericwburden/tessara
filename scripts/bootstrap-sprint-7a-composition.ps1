[CmdletBinding(SupportsShouldProcess)]
param(
    [ValidateSet("reference", "reduced")]
    [string]$Composition = "reference",
    [string]$ComposeFile = "deploy/sprint-7a/compose.yaml",
    [string]$CoreUrl = "http://127.0.0.1:8086",
    [string]$SupervisorUrl = "http://127.0.0.1:8096",
    [string]$ResolvedCompositionEnvelope,
    [string]$ReleaseCatalogEnvelope,
    [switch]$SkipBuild,
    [switch]$ReplaceExisting,
    [string]$DeploymentDirectory = "sprint-7a",
    [string]$ExpectedProject = "tessara-sprint-7a",
    [string]$InstallationId = "01980000-0000-7000-8000-00000000007a",
    [string]$RuntimeLabel = "sprint-7a",
    [string[]]$AdditionalBuildServices = @(),
    [string[]]$AdditionalExpectedNavigationHrefs = @(),
    [switch]$SkipLegacySeed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = $ExpectedProject
$installationId = $InstallationId
$runtimeDirectory = Join-Path $repoRoot "target/$RuntimeLabel-bootstrap/$Composition"
$blueprintPath = Join-Path $repoRoot "deploy/$DeploymentDirectory/blueprints/$Composition.json"
$catalogTemplatePath = Join-Path $repoRoot "deploy/$DeploymentDirectory/catalogs/local-release-catalog.json"
$catalogPayloadPath = Join-Path $runtimeDirectory "release-catalog.json"
$catalogPath = Join-Path $runtimeDirectory "release-catalog.signed.json"
$catalogKeyPath = Join-Path $repoRoot "deploy/$DeploymentDirectory/catalogs/catalog-dev-v1.public.hex"
$lockfilePath = Join-Path $runtimeDirectory "lockfile.json"
$authorizationPath = Join-Path $runtimeDirectory "authorization.json"
$signedAuthorizationPath = Join-Path $runtimeDirectory "authorization.signed.json"
$receiptPath = Join-Path $runtimeDirectory "apply-response.json"

function Resolve-RepositoryPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    return [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
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

if (-not (Test-Path -LiteralPath $composePath)) { throw "Compose file not found: $composePath" }
if (-not (Test-Path -LiteralPath $blueprintPath)) { throw "Blueprint not found: $blueprintPath" }
[IO.Directory]::CreateDirectory($runtimeDirectory) | Out-Null

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
    & docker @composeArguments up -d --no-build
    if ($LASTEXITCODE -ne 0) { throw "$RuntimeLabel service startup failed." }

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
    # Seed through the installation's canonical same-origin gateway. A no-op
    # rerun skips seeding only when the expected fixture is present.
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
        ($catalog.module_releases | Where-Object definition_id -eq "tessara.reference.scoped-records").runtime_image = Get-ImageDigest (Get-ConfiguredServiceImage "scoped-records")
        ($catalog.module_releases | Where-Object definition_id -eq "tessara.dashboards").runtime_image = Get-ImageDigest (Get-ConfiguredServiceImage "dashboards")
        $componentRelease = $catalog.module_releases | Where-Object definition_id -eq "tessara.components"
        if ($componentRelease) { $componentRelease.runtime_image = Get-ImageDigest (Get-ConfiguredServiceImage "components") }
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
    if ([string]::IsNullOrWhiteSpace($ResolvedCompositionEnvelope)) {
        & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
            resolve $blueprintPath $catalogPath $catalogKeyPath $lockfilePath
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
    $approvedEffects = @("install", "upgrade", "configure")
    if ($null -ne $lockfile.core.bootstrap -or @($lockfile.modules | Where-Object { $null -ne $_.bootstrap }).Count -gt 0) {
        $approvedEffects += "bootstrap"
    }
    if (@($lockfile.modules | Where-Object { $_.enabled }).Count -gt 0) { $approvedEffects += "enable" }
    if (@($lockfile.modules | Where-Object { -not $_.enabled }).Count -gt 0) { $approvedEffects += "disable" }

    # Persist the same desired state and explicit approval through Core before
    # the operator-authorized Supervisor apply. This keeps Core read-back
    # complete while retaining the offline/detached CLI execution boundary.
    $compositionSummary = Invoke-RestMethod `
        -Uri "$CoreUrl/api/admin/composition" `
        -Method Get `
        -WebSession $coreSession
    $projectedPlanDigest = $null
    if ($null -ne $compositionSummary.latest_lockfile) {
        $projectedPlanDigest = $compositionSummary.latest_lockfile.materialization_plan_digest
    }
    $resolveAndApprove = $false
    if ($projectedPlanDigest -ne $lockfile.materialization_plan_digest) {
        if ($null -ne $compositionSummary.latest_blueprint) {
            throw "Core already contains a different Blueprint; use -ReplaceExisting for a fresh $RuntimeLabel installation."
        }
        $blueprintJson = Get-Content -LiteralPath $blueprintPath -Raw
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

    $now = [DateTimeOffset]::UtcNow
    if ((Test-Path -LiteralPath $signedAuthorizationPath) -and -not (Test-Path -LiteralPath $receiptPath)) {
        $pendingAuthorization = Get-Content -LiteralPath $signedAuthorizationPath -Raw | ConvertFrom-Json
        if ($pendingAuthorization.payload.target_plan_digest -eq $lockfile.materialization_plan_digest -and `
            [DateTimeOffset]::Parse($pendingAuthorization.payload.expires_at) -gt $now) {
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
            [DateTimeOffset]::Parse($existingAuthorization.payload.expires_at) -gt $now.AddMinutes(1)
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

    $response = & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
        apply $SupervisorUrl $lockfilePath $signedAuthorizationPath
    if ($LASTEXITCODE -ne 0) {
        $failureResponsePath = Join-Path $runtimeDirectory "apply-failure-response.log"
        [IO.File]::WriteAllLines($failureResponsePath, @($response), [Text.UTF8Encoding]::new($false))
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
    Pop-Location
}

[CmdletBinding(SupportsShouldProcess)]
param(
    [ValidateSet("reference", "reduced")]
    [string]$Composition = "reference",
    [string]$ComposeFile = "deploy/sprint-6f/compose.yaml",
    [string]$SupervisorUrl = "http://127.0.0.1:8095",
    [switch]$SkipBuild,
    [switch]$ReplaceExisting
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-6f"
$installationId = "01980000-0000-7000-8000-00000000006f"
$runtimeDirectory = Join-Path $repoRoot "target/sprint-6f-bootstrap/$Composition"
$blueprintPath = Join-Path $repoRoot "deploy/sprint-6f/blueprints/$Composition.json"
$catalogPath = Join-Path $repoRoot "deploy/sprint-6f/catalogs/local-release-catalog.signed.json"
$catalogKeyPath = Join-Path $repoRoot "deploy/sprint-6f/catalogs/catalog-dev-v1.public.hex"
$lockfilePath = Join-Path $runtimeDirectory "lockfile.json"
$authorizationPath = Join-Path $runtimeDirectory "authorization.json"
$signedAuthorizationPath = Join-Path $runtimeDirectory "authorization.signed.json"
$receiptPath = Join-Path $runtimeDirectory "apply-response.json"

if (-not (Test-Path -LiteralPath $composePath)) { throw "Compose file not found: $composePath" }
if (-not (Test-Path -LiteralPath $blueprintPath)) { throw "Blueprint not found: $blueprintPath" }
[IO.Directory]::CreateDirectory($runtimeDirectory) | Out-Null

Push-Location $repoRoot
try {
    $configuredProject = (& docker compose -f $composePath config --format json | ConvertFrom-Json).name
    if ($configuredProject -ne $expectedProject) {
        throw "Refusing to operate on unexpected Compose project '$configuredProject'."
    }

    if ($ReplaceExisting) {
        if ($PSCmdlet.ShouldProcess(
            "$expectedProject containers and named volumes",
            "Remove the fresh Sprint 6F disposable installation state"
        )) {
            & docker compose -f $composePath --profile reference down --volumes --remove-orphans
            if ($LASTEXITCODE -ne 0) { throw "Sprint 6F Compose teardown failed." }
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
    $buildServices = @("supervisor", "core")
    if ($Composition -eq "reference") {
        $buildServices += @("scoped-records", "dashboards")
    }
    if (-not $SkipBuild) {
        foreach ($service in $buildServices) {
            & docker @composeArguments build $service
            if ($LASTEXITCODE -ne 0) { throw "Sprint 6F $service image build failed." }
        }
    }
    & docker @composeArguments up -d --no-build
    if ($LASTEXITCODE -ne 0) { throw "Sprint 6F service startup failed." }

    & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
        catalog-verify $catalogPath $catalogKeyPath
    if ($LASTEXITCODE -ne 0) { throw "Signed release catalog verification failed." }
    & cargo run -q -p tessara-supervisor --bin tessara-compose -- `
        resolve $blueprintPath $catalogPath $catalogKeyPath $lockfilePath
    if ($LASTEXITCODE -ne 0) { throw "Blueprint resolution failed." }

    $lockfile = Get-Content -LiteralPath $lockfilePath -Raw | ConvertFrom-Json
    $now = [DateTimeOffset]::UtcNow
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
    $approvedEffects = @("install", "upgrade", "configure")
    if ($null -ne $lockfile.core.bootstrap -or @($lockfile.modules | Where-Object { $null -ne $_.bootstrap }).Count -gt 0) {
        $approvedEffects += "bootstrap"
    }
    if (@($lockfile.modules | Where-Object { $_.enabled }).Count -gt 0) { $approvedEffects += "enable" }
    if (@($lockfile.modules | Where-Object { -not $_.enabled }).Count -gt 0) { $approvedEffects += "disable" }
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
        idempotency_key = "sprint-6f-$Composition-r$($lockfile.blueprint_revision)-a$applySequence"
        initiator = [ordered]@{ actor_id = "local:sprint-6f-bootstrap"; actor_kind = "operator"; authority = "local-cli" }
        approver = [ordered]@{ actor_id = "local:sprint-6f-approver"; actor_kind = "operator"; authority = "composition:approve" }
        issued_at = $now.ToString("o")
        expires_at = $now.AddMinutes(10).ToString("o")
        approved_effects = $approvedEffects
        reason = "Sprint 6F $Composition reference materialization"
        }
        [IO.File]::WriteAllText(
            $authorizationPath,
            ($authorization | ConvertTo-Json -Depth 20) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
        $env:TESSARA_SIGNING_ISSUER = "tessara.local.sprint-6f"
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
    if ($LASTEXITCODE -ne 0) { throw "Supervisor apply failed." }
    [IO.File]::WriteAllLines($receiptPath, $response, [Text.UTF8Encoding]::new($false))
    Write-Host "Sprint 6F $Composition composition materialized."
    Write-Host "Receipt: $receiptPath"
} finally {
    Pop-Location
}

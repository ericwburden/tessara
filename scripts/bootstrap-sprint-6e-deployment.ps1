[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6e/compose.yaml",
    [string]$ImportToken = "local-deploy-import-token",
    [string]$EvidenceDirectory = "artifacts/sprint-6e-closeout"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$routeDirectory = Join-Path $repoRoot "target/sprint-6e-routing"
$routeTarget = Join-Path $routeDirectory "dashboard.yaml"
[IO.Directory]::CreateDirectory($routeDirectory) | Out-Null
if (-not (Test-Path -LiteralPath $routeTarget)) {
    [IO.File]::WriteAllText(
        $routeTarget,
        [IO.File]::ReadAllText((Join-Path $repoRoot "deploy/sprint-6e/dashboard-route.baseline.yaml")),
        [Text.UTF8Encoding]::new($false)
    )
}

Push-Location $repoRoot
try {
    & .\scripts\bootstrap-sprint-6d-deployment.ps1 `
        -BaseUrl $BaseUrl `
        -ComposeFile $ComposeFile `
        -ImportToken $ImportToken `
        -EvidenceDirectory $EvidenceDirectory
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6E prerequisite materialization failed."
    }

    $composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
    $databaseContainer = (& docker compose -f $composePath ps -q postgres).Trim()
    $release = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
        "SELECT releases.version FROM module_instances instances JOIN module_releases releases ON releases.id=instances.release_id WHERE instances.definition_id='tessara.dashboards' AND instances.identity_state='live'").Trim()
    if ($release -eq "2.0.0") {
        Write-Host "Sprint 6E baseline materialization is ready and repeatable."
        return
    }

    function Get-ImageDigest {
        param([Parameter(Mandatory)][string]$Service)
        $digest = (& docker compose -f $composePath images -q $Service).Trim()
        if ($digest -match "^[0-9a-f]{64}$") {
            return "sha256:$digest"
        }
        throw "The Sprint 6E $Service image is unavailable as an immutable image ID."
    }

    function Set-RuntimeImage {
        param(
            [Parameter(Mandatory)][object]$Manifest,
            [Parameter(Mandatory)][string]$Digest,
            [Parameter(Mandatory)][string]$Repository
        )
        $Manifest.deployment.declaration.runtime_image.digest = $Digest
        $Manifest.deployment.declaration.runtime_image.image_reference = "$Repository@$Digest"
        if ($null -ne $Manifest.deployment.declaration.migration_image) {
            $Manifest.deployment.declaration.migration_image.digest = $Digest
            $Manifest.deployment.declaration.migration_image.image_reference = "$Repository@$Digest"
        }
    }

    function Get-ManifestDigest {
        param([Parameter(Mandatory)][object]$Manifest)
        $bytes = [Text.Encoding]::UTF8.GetBytes(($Manifest | ConvertTo-Json -Depth 100 -Compress))
        "sha256:$([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant())"
    }

    function Set-Property {
        param([Parameter(Mandatory)][object]$InputObject, [Parameter(Mandatory)][string]$Name, $Value)
        if ($InputObject.PSObject.Properties.Name -contains $Name) {
            $InputObject.$Name = $Value
        } else {
            $InputObject | Add-Member -NotePropertyName $Name -NotePropertyValue $Value
        }
    }

    $latestReceiptJson = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
        "SELECT receipt::text FROM deployment_receipts ORDER BY revision DESC LIMIT 1").Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($latestReceiptJson)) {
        throw "The current deployment receipt could not be read."
    }
    $latestReceipt = $latestReceiptJson | ConvertFrom-Json
    $desired = Get-Content -LiteralPath `
        (Join-Path $repoRoot "deploy/sprint-6b1/fixtures/deployment-v1.json") -Raw |
        ConvertFrom-Json
    $desired.installation_id = $latestReceipt.installation_id
    $desired.revision = [int64]$latestReceipt.revision + 1

    $dashboardManifest = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-dashboard-module/manifest.json") -Raw |
        ConvertFrom-Json
    # The active slot is deliberately the committed baseline even when this
    # script is run from the 2.0.1 candidate checkout.
    $dashboardManifest.release_version = "2.0.0"
    $dashboardDigest = Get-ImageDigest -Service "dashboards"
    Set-RuntimeImage $dashboardManifest $dashboardDigest "local/tessara-dashboards"

    $referenceManifest = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-reference-module-sdk/manifest.json") -Raw |
        ConvertFrom-Json
    $referenceDigest = Get-ImageDigest -Service "reference-module-sdk"
    Set-RuntimeImage $referenceManifest $referenceDigest "local/tessara-reference-module-sdk"

    $modules = foreach ($module in @($latestReceipt.modules)) {
        $manifest = $module.manifest
        $version = $module.version
        $runtimeImage = $module.runtime_image
        if ($module.definition_id -eq "tessara.dashboards") {
            $manifest = $dashboardManifest
            $version = "2.0.0"
            $runtimeImage = $dashboardDigest
        } elseif ($module.definition_id -eq "tessara.reference.module-sdk") {
            $manifest = $referenceManifest
            $version = $referenceManifest.release_version
            $runtimeImage = $referenceDigest
        } else {
            Set-Property $manifest "schema_version" 3
            Set-Property $manifest "public_api_routes" @()
            Set-Property $manifest "control_projections" @()
            foreach ($route in @($manifest.browser_routes)) {
                Set-Property $route "dependency_binding" "tessara.core.scoped-records"
            }
            $manifest.linked_packages.module_contract = "0.2.0"
            $manifest.linked_packages.module_runtime = "0.2.0"
            $manifest.linked_packages.module_ui = "0.2.0"
            $manifest.platform_versions.module_contract = "0.2.0"
            $manifest.platform_versions.module_runtime = "0.2.0"
            $manifest.platform_versions.module_ui = "0.2.0"
            $manifest.platform_versions.module_control_protocol = "1.1.0"
            $manifest.platform_versions.conformance_suite = "1.1.0"
            $runtimeImage = Get-ImageDigest -Service "scoped-records"
            Set-RuntimeImage $manifest $runtimeImage "local/tessara-scoped-records"
        }
        [pscustomobject]@{
            definition_id = $module.definition_id
            version = $version
            manifest = $manifest
            manifest_digest = Get-ManifestDigest $manifest
            runtime_image = $runtimeImage
            publisher = $module.publisher
            database_name = $module.database_name
            route_prefix = $module.route_prefix
            configuration = $module.configuration
        }
    }
    $desired.modules = @($modules)

    $workingDirectory = Join-Path $repoRoot "target/sprint-6e-bootstrap"
    [IO.Directory]::CreateDirectory($workingDirectory) | Out-Null
    $desiredPath = Join-Path $workingDirectory "deployment-current.json"
    $planPath = Join-Path $workingDirectory "plan-current.json"
    $receiptPath = Join-Path $workingDirectory "receipt-current.json"
    [IO.File]::WriteAllText(
        $desiredPath,
        ($desired | ConvertTo-Json -Depth 100) + "`n",
        [Text.UTF8Encoding]::new($false)
    )
    foreach ($path in @($planPath, $receiptPath)) {
        if (Test-Path -LiteralPath $path) {
            Remove-Item -LiteralPath $path -Force
        }
    }
    & cargo run -q -p tessara-deploy -- plan $desiredPath $planPath
    if ($LASTEXITCODE -ne 0) { throw "Sprint 6E deployment planning failed." }
    & cargo run -q -p tessara-deploy -- apply $desiredPath $planPath $receiptPath `
        "local:sprint-6e-baseline" ([DateTimeOffset]::UtcNow.ToString("o")) `
        $BaseUrl $ImportToken
    if ($LASTEXITCODE -ne 0) { throw "Sprint 6E baseline receipt import failed." }
    Write-Host "Sprint 6E upgraded the retained deployment to Dashboard release 2.0.0."
} finally {
    Pop-Location
}

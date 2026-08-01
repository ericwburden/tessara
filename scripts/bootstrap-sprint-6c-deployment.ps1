[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6c/compose.yaml",
    [string]$ImportToken = "local-deploy-import-token",
    [string]$DashboardManifestPath = "crates/tessara-dashboard-module/manifest.json",
    [switch]$ModernizeScopedRecordsManifest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = Join-Path $repoRoot $ComposeFile
$fixturePath = Join-Path $repoRoot "deploy/sprint-6b1/fixtures/deployment-v1.json"
$dashboardManifestPath = [IO.Path]::GetFullPath((Join-Path $repoRoot $DashboardManifestPath))
$workingDirectory = Join-Path $repoRoot "target/sprint-6c-bootstrap"
[IO.Directory]::CreateDirectory($workingDirectory) | Out-Null

$databaseContainer = (& docker compose -f $composePath ps -q postgres).Trim()
if ($LASTEXITCODE -ne 0 -or $databaseContainer -notmatch "^[0-9a-f]{64}$") {
    throw "The Sprint 6C PostgreSQL container is not running."
}

$installationId = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
    "SELECT id FROM application_installations WHERE singleton = true").Trim()
if ($LASTEXITCODE -ne 0 -or $installationId -notmatch "^[0-9a-f-]{36}$") {
    throw "The Sprint 6C installation identity is unavailable."
}

$existingRevision = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
    "SELECT COALESCE(max(revision), 0) FROM deployment_receipts").Trim()
if ($LASTEXITCODE -ne 0) {
    throw "The Sprint 6C deployment state is unavailable."
}
if ([int64]$existingRevision -gt 0) {
    Write-Host "Sprint 6C deployment receipt already present at revision $existingRevision."
    exit 0
}

$desired = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json
$manifest = Get-Content -LiteralPath $dashboardManifestPath -Raw | ConvertFrom-Json

function Get-ManifestDigest {
    param([Parameter(Mandatory)][object]$Manifest)

    $bytes = [Text.Encoding]::UTF8.GetBytes(($Manifest | ConvertTo-Json -Depth 100 -Compress))
    "sha256:$([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant())"
}

function Set-ManifestRuntimeImage {
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

if ($ModernizeScopedRecordsManifest) {
    $scopedModule = @($desired.modules | Where-Object definition_id -eq "tessara.reference.scoped-records")
    if ($scopedModule.Count -ne 1) {
        throw "The Sprint 6C fixture must contain one replaceable Scoped Records declaration."
    }
    $scopedManifest = $scopedModule[0].manifest
    $scopedManifest.schema_version = 3
    $scopedManifest | Add-Member -NotePropertyName public_api_routes -NotePropertyValue @() -Force
    $scopedManifest | Add-Member -NotePropertyName control_projections -NotePropertyValue @() -Force
    foreach ($route in @($scopedManifest.browser_routes)) {
        $route | Add-Member -NotePropertyName dependency_binding `
            -NotePropertyValue "tessara.core.scoped-records" -Force
    }
    $scopedManifest.linked_packages.module_contract = "0.2.0"
    $scopedManifest.linked_packages.module_runtime = "0.2.0"
    $scopedManifest.linked_packages.module_ui = "0.2.0"
    $scopedManifest.platform_versions.module_contract = "0.2.0"
    $scopedManifest.platform_versions.module_runtime = "0.2.0"
    $scopedManifest.platform_versions.module_ui = "0.2.0"
    $scopedManifest.platform_versions.module_control_protocol = "1.1.0"
    $scopedManifest.platform_versions.conformance_suite = "1.1.0"
    $scopedImageId = (& docker compose -f $composePath images -q scoped-records).Trim()
    if ($scopedImageId -match "^[0-9a-f]{64}$") {
        $scopedImageId = "sha256:$scopedImageId"
    }
    if ($LASTEXITCODE -ne 0 -or $scopedImageId -notmatch "^sha256:[0-9a-f]{64}$") {
        throw "The Sprint 6C Scoped Records image is not available as an immutable image ID."
    }
    Set-ManifestRuntimeImage -Manifest $scopedManifest -Digest $scopedImageId `
        -Repository "local/tessara-scoped-records"
    $scopedModule[0].version = $scopedManifest.release_version
    $scopedModule[0].manifest = $scopedManifest
    $scopedModule[0].manifest_digest = Get-ManifestDigest $scopedManifest
    $scopedModule[0].runtime_image = $scopedImageId
    $scopedModule[0].publisher = $scopedManifest.publisher
}

$dashboardImageId = (& docker compose -f $composePath images -q dashboards).Trim()
if ($dashboardImageId -match "^[0-9a-f]{64}$") {
    $dashboardImageId = "sha256:$dashboardImageId"
}
if ($LASTEXITCODE -ne 0 -or $dashboardImageId -notmatch "^sha256:[0-9a-f]{64}$") {
    throw "The Sprint 6C Dashboard image is not available as an immutable image ID."
}
$runtimeDigest = $dashboardImageId
Set-ManifestRuntimeImage -Manifest $manifest -Digest $runtimeDigest `
    -Repository "local/tessara-dashboards"

$dashboardModule = [pscustomobject]@{
    definition_id = $manifest.definition_id
    version = $manifest.release_version
    manifest = $manifest
    manifest_digest = Get-ManifestDigest $manifest
    runtime_image = $runtimeDigest
    publisher = $manifest.publisher
    database_name = "tessara_module_dashboards"
    route_prefix = "/dashboards"
    configuration = [pscustomobject]@{
        display_label = "Dashboards"
        default_page_size = "25"
    }
}
$desired.modules = @($desired.modules) + @($dashboardModule)
$desired.installation_id = $installationId
$desired.revision = 1

$desiredPath = Join-Path $workingDirectory "deployment-v1.json"
$planPath = Join-Path $workingDirectory "plan-v1.json"
$receiptPath = Join-Path $workingDirectory "receipt-v1.json"
[IO.File]::WriteAllText(
    $desiredPath,
    ($desired | ConvertTo-Json -Depth 40) + [Environment]::NewLine,
    [Text.UTF8Encoding]::new($false)
)
foreach ($path in @($planPath, $receiptPath)) {
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force
    }
}

Push-Location $repoRoot
try {
    & cargo run -q -p tessara-deploy -- plan $desiredPath $planPath
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6C deployment planning failed."
    }
    & cargo run -q -p tessara-deploy -- apply $desiredPath $planPath $receiptPath `
        "local:sprint-6c-bootstrap" ([DateTimeOffset]::UtcNow.ToString("o")) $BaseUrl $ImportToken
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6C deployment receipt import failed."
    }
} finally {
    Pop-Location
}

Write-Host "Sprint 6C Dashboard and Scoped Records deployments registered for installation $installationId."

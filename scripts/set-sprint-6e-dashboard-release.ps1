[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet("baseline", "candidate")]
    [string]$Slot,
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6e/compose.yaml",
    [string]$ImportToken = "local-deploy-import-token",
    [string]$EvidenceDirectory = "artifacts/sprint-6e-closeout"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$evidencePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceDirectory))
[IO.Directory]::CreateDirectory($evidencePath) | Out-Null

$service = if ($Slot -eq "candidate") { "dashboards-candidate" } else { "dashboards" }
$manifestPath = if ($Slot -eq "candidate") {
    Join-Path $repoRoot "crates/tessara-dashboard-module/manifest.json"
} else {
    Join-Path $repoRoot "deploy/sprint-6e/dashboard-manifest.baseline.json"
}
$expectedVersion = if ($Slot -eq "candidate") { "2.0.2" } else { "2.0.0" }

$container = [string](& docker compose -f $composePath --profile candidate ps -aq $service)
$container = $container.Trim()
if ($LASTEXITCODE -ne 0 -or $container -notmatch "^[0-9a-f]{64}$") {
    throw "Dashboard slot '$Slot' is not running; release metadata was not changed."
}
$health = (& docker inspect --format "{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}" $container).Trim()
if ($LASTEXITCODE -ne 0 -or $health -ne "healthy") {
    throw "Dashboard slot '$Slot' is not healthy; release metadata was not changed."
}
$imageId = (& docker inspect --format "{{.Image}}" $container).Trim()
if ($LASTEXITCODE -ne 0 -or $imageId -notmatch "^sha256:[0-9a-f]{64}$") {
    throw "Dashboard slot '$Slot' does not expose an immutable image ID."
}

$databaseContainer = (& docker compose -f $composePath ps -q postgres).Trim()
if ($LASTEXITCODE -ne 0 -or $databaseContainer -notmatch "^[0-9a-f]{64}$") {
    throw "The Sprint 6E database container is unavailable."
}
$latestReceiptJson = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
    "SELECT receipt::text FROM deployment_receipts ORDER BY revision DESC LIMIT 1").Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($latestReceiptJson)) {
    throw "The current deployment receipt could not be read."
}
$latestReceipt = $latestReceiptJson | ConvertFrom-Json
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.release_version -ne $expectedVersion) {
    throw "The $Slot manifest declares '$($manifest.release_version)', expected '$expectedVersion'."
}
$manifest.deployment.declaration.runtime_image.digest = $imageId
$manifest.deployment.declaration.runtime_image.image_reference = "local/tessara-dashboards@$imageId"
if ($null -ne $manifest.deployment.declaration.migration_image) {
    $manifest.deployment.declaration.migration_image.digest = $imageId
    $manifest.deployment.declaration.migration_image.image_reference = "local/tessara-dashboards@$imageId"
}

function Get-ManifestDigest {
    param([Parameter(Mandatory)][object]$Manifest)
    $bytes = [Text.Encoding]::UTF8.GetBytes(($Manifest | ConvertTo-Json -Depth 100 -Compress))
    "sha256:$([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant())"
}

$desired = Get-Content -LiteralPath `
    (Join-Path $repoRoot "deploy/sprint-6b1/fixtures/deployment-v1.json") -Raw |
    ConvertFrom-Json
$desired.installation_id = $latestReceipt.installation_id
$desired.revision = [int64]$latestReceipt.revision + 1
$desired.modules = @($latestReceipt.modules | ForEach-Object {
    $isDashboard = $_.definition_id -eq "tessara.dashboards"
    $selectedManifest = if ($isDashboard) { $manifest } else { $_.manifest }
    [pscustomobject]@{
        definition_id = $_.definition_id
        version = if ($isDashboard) { $expectedVersion } else { $_.version }
        manifest = $selectedManifest
        manifest_digest = if ($isDashboard) { Get-ManifestDigest $manifest } else { $_.manifest_digest }
        runtime_image = if ($isDashboard) { $imageId } else { $_.runtime_image }
        publisher = $_.publisher
        database_name = $_.database_name
        route_prefix = $_.route_prefix
        configuration = $_.configuration
    }
})

$workingDirectory = Join-Path $repoRoot "target/sprint-6e-release-switch"
[IO.Directory]::CreateDirectory($workingDirectory) | Out-Null
$desiredPath = Join-Path $workingDirectory "deployment-$Slot.json"
$planPath = Join-Path $workingDirectory "plan-$Slot.json"
$receiptPath = Join-Path $workingDirectory "receipt-$Slot.json"
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

Push-Location $repoRoot
try {
    & cargo run -q -p tessara-deploy -- plan $desiredPath $planPath
    if ($LASTEXITCODE -ne 0) { throw "Dashboard $Slot release planning failed." }
    & cargo run -q -p tessara-deploy -- apply $desiredPath $planPath $receiptPath `
        "local:sprint-6e-$Slot" ([DateTimeOffset]::UtcNow.ToString("o")) `
        $BaseUrl $ImportToken
    if ($LASTEXITCODE -ne 0) { throw "Dashboard $Slot release apply failed." }
} finally {
    Pop-Location
}

$record = [ordered]@{
    schema_version = 1
    applied_at = [DateTimeOffset]::UtcNow.ToString("o")
    slot = $Slot
    release_version = $expectedVersion
    container = $container
    image = $imageId
    manifest_digest = Get-ManifestDigest $manifest
    result = "applied"
}
[IO.File]::WriteAllText(
    (Join-Path $evidencePath "release-metadata-$Slot.json"),
    ($record | ConvertTo-Json -Depth 10) + "`n",
    [Text.UTF8Encoding]::new($false)
)
Write-Host "Dashboard release metadata switched to '$expectedVersion' for '$Slot'."

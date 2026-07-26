[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6b2/compose.yaml",
    [string]$ImportToken = "local-deploy-import-token"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = Join-Path $repoRoot $ComposeFile
$fixturePath = Join-Path $repoRoot "deploy/sprint-6b1/fixtures/deployment-v1.json"
$workingDirectory = Join-Path $repoRoot "target/sprint-6b2-bootstrap"
[IO.Directory]::CreateDirectory($workingDirectory) | Out-Null

$databaseContainer = (& docker compose -f $composePath ps -q postgres).Trim()
if ($LASTEXITCODE -ne 0 -or $databaseContainer -notmatch "^[0-9a-f]{64}$") {
    throw "The Sprint 6B2 PostgreSQL container is not running."
}

$installationId = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
    "SELECT id FROM application_installations WHERE singleton = true").Trim()
if ($LASTEXITCODE -ne 0 -or $installationId -notmatch "^[0-9a-f-]{36}$") {
    throw "The Sprint 6B2 installation identity is unavailable."
}

$existingRevision = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
    "SELECT COALESCE(max(revision), 0) FROM deployment_receipts").Trim()
if ($LASTEXITCODE -ne 0) {
    throw "The Sprint 6B2 deployment state is unavailable."
}
if ([int64]$existingRevision -gt 0) {
    Write-Host "Sprint 6B2 deployment receipt already present at revision $existingRevision."
    exit 0
}

$desired = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json
$desired.installation_id = $installationId
$desired.revision = 1

$desiredPath = Join-Path $workingDirectory "deployment-v1.json"
$planPath = Join-Path $workingDirectory "plan-v1.json"
$receiptPath = Join-Path $workingDirectory "receipt-v1.json"
[IO.File]::WriteAllText(
    $desiredPath,
    ($desired | ConvertTo-Json -Depth 30) + [Environment]::NewLine,
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
        throw "Sprint 6B2 deployment planning failed."
    }
    & cargo run -q -p tessara-deploy -- apply $desiredPath $planPath $receiptPath `
        "local:sprint-6b2-bootstrap" ([DateTimeOffset]::UtcNow.ToString("o")) $BaseUrl $ImportToken
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6B2 deployment receipt import failed."
    }
} finally {
    Pop-Location
}

Write-Host "Sprint 6B2 Scoped Records deployment registered for installation $installationId."

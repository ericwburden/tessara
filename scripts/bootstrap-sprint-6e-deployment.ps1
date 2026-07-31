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
    if ($release -ne "2.0.0") {
        throw "Sprint 6E baseline must materialize Dashboard release 2.0.0; observed '$release'."
    }
    Write-Host "Sprint 6E baseline materialization is ready and repeatable."
} finally {
    Pop-Location
}

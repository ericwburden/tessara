[CmdletBinding()]
param([string]$ComposeFile = "deploy/sprint-6c/compose.yaml")

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = Join-Path $repoRoot $ComposeFile
$databaseContainer = (& docker compose -f $composePath ps -q postgres).Trim()
if ($LASTEXITCODE -ne 0 -or $databaseContainer -notmatch "^[0-9a-f]{64}$") {
    throw "The Sprint 6C PostgreSQL container is not running."
}

function Test-DeniedConnection {
    param([string]$Role, [string]$Password, [string]$Database)
    & docker exec -e "PGPASSWORD=$Password" $databaseContainer `
        psql -X -U $Role -d $Database -Atc "SELECT 1" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) {
        throw "$Role unexpectedly connected to $Database."
    }
}

& docker exec -e "PGPASSWORD=local-dashboard-runtime" $databaseContainer `
    psql -X -U tessara_dashboard_runtime -d tessara_module_dashboards -Atc `
    "SELECT COUNT(*) FROM dashboards" | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Dashboard runtime cannot read its own schema."
}

Test-DeniedConnection tessara_dashboard_runtime "local-dashboard-runtime" tessara_core
Test-DeniedConnection tessara_dashboard_runtime "local-dashboard-runtime" tessara_deployment
Test-DeniedConnection tessara_dashboard_runtime "local-dashboard-runtime" tessara_module_scoped_records
Test-DeniedConnection tessara_core_runtime "local-core-runtime" tessara_module_dashboards
Test-DeniedConnection tessara_scoped_runtime "local-scoped-runtime" tessara_module_dashboards

Write-Host "Sprint 6C database isolation checks passed."

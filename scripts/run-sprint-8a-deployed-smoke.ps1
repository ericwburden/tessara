[CmdletBinding()]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [Parameter(Mandatory = $true)][string]$DeploymentEvidencePath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-8a"

function Get-ExactRunningServiceContainer([string]$Service) {
    $ids = @(@(& docker compose -f $composePath --profile reference ps --status running -q $Service) |
        ForEach-Object { ([string]$_).Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($LASTEXITCODE -ne 0 -or $ids.Count -ne 1 -or $ids[0] -cnotmatch '^[0-9a-f]{64}$') {
        throw "Expected exactly one running $expectedProject '$Service' container, found '$($ids -join ',')'."
    }
    $ids[0]
}

$configuration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or [string]$configuration.name -cne $expectedProject) {
    throw "Sprint 8A deployed-smoke runner is restricted to the exact $expectedProject Compose project."
}

if ($SelfTest) {
    foreach ($service in @('core', 'gateway', 'postgres')) {
        if (-not $configuration.services.$service) {
            throw "Sprint 8A deployed-smoke self-test could not find '$service' in normalized Compose."
        }
    }
    $capture = Get-Command (Join-Path $PSScriptRoot 'capture-sprint-6a-deployment-evidence.ps1')
    foreach ($parameter in @('ApiContainerId', 'GatewayContainerId', 'DatabaseContainerId', 'ExpectedDataState', 'TransitionCatalogProfile')) {
        if (-not $capture.Parameters.ContainsKey($parameter)) {
            throw "Deployment-evidence helper no longer declares -$parameter."
        }
    }
    $smoke = Get-Command (Join-Path $PSScriptRoot 'smoke.ps1')
    foreach ($parameter in @('DeploymentEvidencePath', 'ExpectedDataState', 'TransitionCatalogProfile')) {
        if (-not $smoke.Parameters.ContainsKey($parameter)) {
            throw "General smoke runner no longer declares -$parameter."
        }
    }
    Write-Host "Sprint 8A deployed-smoke orchestration self-test passed."
    return
}

$coreContainer = Get-ExactRunningServiceContainer 'core'
$gatewayContainer = Get-ExactRunningServiceContainer 'gateway'
$databaseContainer = Get-ExactRunningServiceContainer 'postgres'

& (Join-Path $PSScriptRoot 'capture-sprint-6a-deployment-evidence.ps1') `
    -BaseUrl $BaseUrl `
    -ExpectedDataState fresh `
    -OutputPath $DeploymentEvidencePath `
    -ApiContainerId $coreContainer `
    -GatewayContainerId $gatewayContainer `
    -DatabaseContainerId $databaseContainer `
    -TransitionCatalogProfile sprint-8a `
    -Overwrite:$Overwrite

& (Join-Path $PSScriptRoot 'smoke.ps1') `
    -UseExistingService `
    -KeepServices `
    -BaseUrl $BaseUrl `
    -DeploymentEvidencePath $DeploymentEvidencePath `
    -ExpectedDataState fresh `
    -TransitionCatalogProfile sprint-8a

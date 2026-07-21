[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [ValidateSet("upgraded", "fresh")][string]$ExpectedDataState,
    [string]$OutputPath,
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$ApiContainerId,
    [string]$DatabaseContainerId,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$artifactRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot "artifacts/sprint-6a-ui"))
$artifactPrefix = "$artifactRoot$([IO.Path]::DirectorySeparatorChar)"
$publisher = Join-Path $PSScriptRoot "capture-sprint-6a-deployment-evidence.ps1"

function Assert-Sprint6AUiArtifactPath {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
    }
    if (-not $fullPath.StartsWith($artifactPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Sprint 6A-UI deployment evidence must be written under '$artifactRoot'."
    }
    return $fullPath
}

if ($SelfTest) {
    & $publisher -SelfTest
    $rejected = $false
    try {
        Assert-Sprint6AUiArtifactPath -Path "artifacts/sprint-6a/deployment-upgraded.json" | Out-Null
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Sprint 6A-UI deployment-evidence self-test failed: a non-UI artifact path was accepted."
    }
    Write-Host "Sprint 6A-UI deployment-evidence publisher self-test passed." -ForegroundColor Green
    exit 0
}

if ([string]::IsNullOrWhiteSpace($ExpectedDataState)) {
    throw "-ExpectedDataState upgraded|fresh is required. The database-derived value is independently verified by the retained publisher."
}
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = "artifacts/sprint-6a-ui/deployment-$ExpectedDataState.json"
}
$fullOutputPath = Assert-Sprint6AUiArtifactPath -Path $OutputPath

$parameters = @{
    BaseUrl = $BaseUrl
    ExpectedDataState = $ExpectedDataState
    OutputPath = $fullOutputPath
    AdminEmail = $AdminEmail
    AdminPassword = $AdminPassword
    Overwrite = [bool]$Overwrite
}
if (-not [string]::IsNullOrWhiteSpace($ApiContainerId)) {
    $parameters.ApiContainerId = $ApiContainerId
}
if (-not [string]::IsNullOrWhiteSpace($DatabaseContainerId)) {
    $parameters.DatabaseContainerId = $DatabaseContainerId
}

& $publisher @parameters

[CmdletBinding()]
param(
    [string]$Spec,
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [switch]$Seed,
    [string[]]$PlaywrightArgs = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$endToEndDir = Join-Path $repoRoot "end2end"
$seedScript = Join-Path $PSScriptRoot "seed-demo-data.ps1"

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory)]
        [string]$Label,
        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    & $Command

    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Assert-AppIsReachable {
    param(
        [Parameter(Mandatory)]
        [string]$Uri
    )

    try {
        $response = Invoke-WebRequest -Uri "$Uri/health" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -ne 200) {
            throw "Expected HTTP 200 from $Uri/health, received $($response.StatusCode)."
        }
    } catch {
        throw "Tessara is not reachable at $Uri. Start it with .\scripts\local-launch.ps1 before running e2e validation."
    }
}

if (-not (Test-Path $endToEndDir)) {
    throw "Could not find end2end directory at $endToEndDir"
}

if ($Seed -and -not (Test-Path $seedScript)) {
    throw "Could not find seed helper at $seedScript"
}

Push-Location $repoRoot
try {
    Assert-AppIsReachable -Uri $BaseUrl.TrimEnd("/")

    if ($Seed) {
        Invoke-CheckedStep -Label "Ensuring UAT demo data" -Command {
            & $seedScript
        }
    }

    $env:PLAYWRIGHT_BASE_URL = $BaseUrl.TrimEnd("/")
    Invoke-CheckedStep -Label "Running Playwright e2e tests, including permission scenarios" -Command {
        if ([string]::IsNullOrWhiteSpace($Spec)) {
            npm --prefix $endToEndDir test -- @PlaywrightArgs
        } else {
            npm --prefix $endToEndDir test -- $Spec @PlaywrightArgs
        }
    }
} finally {
    Pop-Location
}

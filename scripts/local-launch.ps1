[CmdletBinding()]
param(
    [switch]$FreshData,
    [switch]$FollowLogs,
    [switch]$ApiOnly,
    [switch]$SkipBuild,
    [switch]$SkipSeed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composeFile = Join-Path $repoRoot "docker-compose.yml"
$seedScript = Join-Path $PSScriptRoot "seed-demo-data.ps1"
$refreshScript = Join-Path $PSScriptRoot "local-refresh-api.ps1"

if (-not (Test-Path $composeFile)) {
    throw "Could not find docker-compose.yml at $composeFile"
}

if (-not (Test-Path $seedScript)) {
    throw "Could not find seed helper at $seedScript"
}

if (-not (Test-Path $refreshScript)) {
    throw "Could not find API refresh helper at $refreshScript"
}

if ($FreshData -and $ApiOnly) {
    throw "-FreshData and -ApiOnly cannot be used together. Use .\\scripts\\local-refresh-api.ps1 for API-only refreshes."
}

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

function Wait-ForHttpOk {
    param(
        [Parameter(Mandatory)]
        [string]$Uri,
        [int]$TimeoutSeconds = 180
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        try {
            $response = Invoke-WebRequest -Uri $Uri -TimeoutSec 5 -UseBasicParsing
            if ($response.StatusCode -eq 200) {
                return
            }
        } catch {
            Start-Sleep -Seconds 2
            continue
        }

        Start-Sleep -Seconds 2
    }

    Write-Host "`nCurrent docker compose status:" -ForegroundColor Yellow
    docker compose ps
    Write-Host "`nRecent API logs:" -ForegroundColor Yellow
    docker compose logs --tail 80 api
    throw "Timed out waiting for $Uri to return HTTP 200"
}

Push-Location $repoRoot
try {
    if ($ApiOnly) {
        $refreshArgs = @{}
        if ($SkipBuild) {
            $refreshArgs.SkipBuild = $true
        }
        if ($SkipSeed) {
            $refreshArgs.SkipSeed = $true
        }
        if ($FollowLogs) {
            $refreshArgs.FollowLogs = $true
        }

        & $refreshScript @refreshArgs
        return
    }

    $downArgs = @("compose", "down")
    if ($FreshData) {
        $downArgs += "-v"
    }

    Invoke-CheckedStep -Label "Stopping existing Compose stack" -Command {
        docker @downArgs
    }

    if (-not $SkipBuild) {
        Invoke-CheckedStep -Label "Rebuilding Tessara API image" -Command {
            docker compose build api
        }
    } else {
        Write-Host "`n==> Reusing existing API image" -ForegroundColor Cyan
    }

    Invoke-CheckedStep -Label "Starting refreshed Compose stack" -Command {
        docker compose up -d --force-recreate
    }

    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/health"
    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/"

    if (-not $SkipSeed) {
        Invoke-CheckedStep -Label "Ensuring UAT demo data" -Command {
            & $seedScript
        }
    } else {
        Write-Host "`n==> Skipping demo seed" -ForegroundColor Cyan
    }

    Write-Host "`nTessara is ready." -ForegroundColor Green
    Write-Host "Application shell: http://localhost:8080/"
    Write-Host "Administration:   http://localhost:8080/administration"
    Write-Host "Node Types:       http://localhost:8080/administration/node-types"
    Write-Host "Roles:            http://localhost:8080/administration/roles"
    Write-Host "Migration:        http://localhost:8080/migration"
    Write-Host ""
    Write-Host "Demo accounts:" -ForegroundColor Green
    Write-Host "  admin@tessara.local       / tessara-dev-admin"
    Write-Host "  operator@tessara.local    / tessara-dev-operator"
    Write-Host "  delegator@tessara.local   / tessara-dev-delegator"
    Write-Host "  respondent@tessara.local  / tessara-dev-respondent"
    Write-Host "  delegate@tessara.local    / tessara-dev-delegate"
    if ($FreshData) {
        Write-Host ""
        Write-Host "Postgres volume was refreshed because -FreshData was supplied." -ForegroundColor Yellow
    }
    if ($SkipBuild) {
        Write-Host "API image rebuild was skipped because -SkipBuild was supplied." -ForegroundColor Yellow
    }
    if ($SkipSeed) {
        Write-Host "Demo seeding was skipped because -SkipSeed was supplied." -ForegroundColor Yellow
    }

    if ($FollowLogs) {
        Write-Host "`nFollowing Compose logs. Press Ctrl+C to stop log streaming." -ForegroundColor Cyan
        docker compose logs -f postgres api
    }
} finally {
    Pop-Location
}

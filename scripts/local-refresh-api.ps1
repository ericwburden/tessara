[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$SkipSeed,
    [switch]$FollowLogs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composeFile = Join-Path $repoRoot "docker-compose.yml"
$seedScript = Join-Path $PSScriptRoot "seed-demo-data.ps1"

if (-not (Test-Path $composeFile)) {
    throw "Could not find docker-compose.yml at $composeFile"
}

if (-not (Test-Path $seedScript)) {
    throw "Could not find seed helper at $seedScript"
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
    Invoke-CheckedStep -Label "Ensuring Postgres is running" -Command {
        docker compose up -d postgres
    }

    if (-not $SkipBuild) {
        Invoke-CheckedStep -Label "Rebuilding Tessara API image" -Command {
            docker compose build api
        }
    } else {
        Write-Host "`n==> Reusing existing API image" -ForegroundColor Cyan
    }

    Invoke-CheckedStep -Label "Refreshing API container" -Command {
        docker compose up -d --no-deps --force-recreate api
    }

    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/health"
    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/app"

    if (-not $SkipSeed) {
        Invoke-CheckedStep -Label "Ensuring UAT demo data" -Command {
            & $seedScript
        }
    } else {
        Write-Host "`n==> Skipping demo seed" -ForegroundColor Cyan
    }

    Write-Host "`nFast API refresh complete." -ForegroundColor Green
    Write-Host "Application shell: http://localhost:8080/app"
    Write-Host "Administration:   http://localhost:8080/app/administration"
    Write-Host "Reports:          http://localhost:8080/app/reports"
    Write-Host "Migration:        http://localhost:8080/app/migration"
    if ($SkipBuild) {
        Write-Host "API image rebuild was skipped because -SkipBuild was supplied." -ForegroundColor Yellow
    }
    if ($SkipSeed) {
        Write-Host "Demo seeding was skipped because -SkipSeed was supplied." -ForegroundColor Yellow
    }

    if ($FollowLogs) {
        Write-Host "`nFollowing API logs. Press Ctrl+C to stop log streaming." -ForegroundColor Cyan
        docker compose logs -f api
    }
} finally {
    Pop-Location
}

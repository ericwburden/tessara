[CmdletBinding()]
param(
    [switch]$FreshData,
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
    $downArgs = @("compose", "down", "--remove-orphans")
    if ($FreshData) {
        $downArgs += "-v"
    }

    Invoke-CheckedStep -Label "Stopping existing Compose stack" -Command {
        docker @downArgs
    }

    Invoke-CheckedStep -Label "Rebuilding Tessara API image" -Command {
        docker compose build api
    }

    Invoke-CheckedStep -Label "Starting refreshed Compose stack" -Command {
        docker compose up -d --force-recreate
    }

    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/health"
    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/app"

    Invoke-CheckedStep -Label "Ensuring UAT demo data" -Command {
        & $seedScript
    }

    Write-Host "`nTessara is ready." -ForegroundColor Green
    Write-Host "Application shell: http://localhost:8080/app"
    Write-Host "Administration:   http://localhost:8080/app/administration"
    Write-Host "Reports:          http://localhost:8080/app/reports"
    Write-Host "Migration:        http://localhost:8080/app/migration"
    Write-Host ""
    Write-Host "Demo accounts:" -ForegroundColor Green
    Write-Host "  admin@tessara.local       / tessara-dev-admin"
    Write-Host "  operator@tessara.local    / tessara-dev-operator"
    Write-Host "  parent@tessara.local      / tessara-dev-parent"
    Write-Host "  respondent@tessara.local  / tessara-dev-respondent"
    Write-Host "  child@tessara.local       / tessara-dev-child"
    if ($FreshData) {
        Write-Host ""
        Write-Host "Postgres volume was refreshed because -FreshData was supplied." -ForegroundColor Yellow
    }

    if ($FollowLogs) {
        Write-Host "`nFollowing Compose logs. Press Ctrl+C to stop log streaming." -ForegroundColor Cyan
        docker compose logs -f postgres api
    }
} finally {
    Pop-Location
}

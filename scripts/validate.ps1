[CmdletBinding()]
param(
    [switch]$Fast
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory)]
        [string]$Label,
        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    $startedAt = Get-Date

    & $Command

    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }

    $elapsed = (Get-Date) - $startedAt
    Write-Host ("Passed in {0:mm\:ss}" -f $elapsed) -ForegroundColor Green
}

Push-Location $repoRoot
try {
    if ($Fast) {
        Write-Host "Running fast Tessara validation. Use .\scripts\validate.ps1 for the full pre-commit matrix." -ForegroundColor Yellow
    } else {
        Write-Host "Running full Tessara validation sequentially. This avoids Cargo lock contention on Windows." -ForegroundColor Yellow
    }

    Invoke-CheckedStep -Label "Formatting check" -Command {
        cargo fmt --all --check
    }

    Invoke-CheckedStep -Label "API check" -Command {
        cargo check -p tessara-api
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "API SSR check" -Command {
            cargo check -p tessara-api --features ssr
        }
    }

    Invoke-CheckedStep -Label "Web check" -Command {
        cargo check -p tessara-web
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "Web hydrate check" -Command {
            cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown
        }
    }

    Invoke-CheckedStep -Label "Web tests" -Command {
        cargo test -p tessara-web
    }

    Invoke-CheckedStep -Label "API tests" -Command {
        cargo test -p tessara-api
    }

    Write-Host "`nValidation passed." -ForegroundColor Green
} finally {
    Pop-Location
}

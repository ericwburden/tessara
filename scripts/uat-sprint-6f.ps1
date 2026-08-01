[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$SupervisorUrl = "http://127.0.0.1:8095",
    [ValidateSet("reference", "reduced")][string]$Composition = "reference",
    [switch]$SkipBrowser
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

& (Join-Path $PSScriptRoot "smoke-sprint-6f.ps1") -BaseUrl $BaseUrl -SupervisorUrl $SupervisorUrl -Composition $Composition
if ($LASTEXITCODE -ne 0) { throw "Sprint 6F composition smoke failed." }

if (-not $SkipBrowser) {
    Push-Location (Join-Path $repoRoot "end2end")
    try {
        $env:PLAYWRIGHT_BASE_URL = $BaseUrl
        & npm test -- composition.spec.ts
        if ($LASTEXITCODE -ne 0) { throw "Sprint 6F browser acceptance failed." }
    } finally {
        Pop-Location
    }
}
Write-Host "Sprint 6F automated UAT checks passed. Complete the eight business scripts for formal acceptance."

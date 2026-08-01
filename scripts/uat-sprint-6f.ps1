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
    $hadBaseUrl = Test-Path Env:PLAYWRIGHT_BASE_URL
    $callerBaseUrl = if ($hadBaseUrl) { $env:PLAYWRIGHT_BASE_URL } else { $null }
    $hadCompositionRequirement = Test-Path Env:TESSARA_PLAYWRIGHT_REQUIRE_COMPOSITION
    $callerCompositionRequirement = if ($hadCompositionRequirement) {
        $env:TESSARA_PLAYWRIGHT_REQUIRE_COMPOSITION
    } else { $null }
    Push-Location (Join-Path $repoRoot "end2end")
    try {
        $env:PLAYWRIGHT_BASE_URL = $BaseUrl
        $env:TESSARA_PLAYWRIGHT_REQUIRE_COMPOSITION = "1"
        & npm test -- composition.spec.ts
        if ($LASTEXITCODE -ne 0) { throw "Sprint 6F browser acceptance failed." }
    } finally {
        Pop-Location
        if ($hadBaseUrl) { $env:PLAYWRIGHT_BASE_URL = $callerBaseUrl }
        else { Remove-Item Env:PLAYWRIGHT_BASE_URL -ErrorAction SilentlyContinue }
        if ($hadCompositionRequirement) {
            $env:TESSARA_PLAYWRIGHT_REQUIRE_COMPOSITION = $callerCompositionRequirement
        } else {
            Remove-Item Env:TESSARA_PLAYWRIGHT_REQUIRE_COMPOSITION -ErrorAction SilentlyContinue
        }
    }
}
Write-Host "Sprint 6F automated UAT checks passed. Complete the eight business scripts for formal acceptance."

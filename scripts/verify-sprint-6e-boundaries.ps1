[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

$forbidden = & rg -n `
    "tessara-web-dashboards|tessara_web_dashboards|tessara-web\s*=|tessara-api|tessara-core|tessara-web-components" `
    (Join-Path $repoRoot "crates/tessara-dashboard-module") `
    (Join-Path $repoRoot "crates/tessara-dashboard-ui") `
    (Join-Path $repoRoot "crates/tessara-dashboards") `
    (Join-Path $repoRoot "crates/tessara-components-contract") `
    -g "!target/**"
if ($LASTEXITCODE -eq 0 -or $forbidden) {
    throw "Dashboard release boundary contains a forbidden root/Core/feature dependency:`n$forbidden"
}
if ($LASTEXITCODE -ne 1) {
    throw "Dashboard boundary audit failed to execute."
}

$rootReferences = & rg -n "tessara-dashboard-ui|tessara_dashboard_ui" `
    (Join-Path $repoRoot "crates/tessara-web") `
    (Join-Path $repoRoot "crates/tessara-api")
if ($LASTEXITCODE -eq 0 -or $rootReferences) {
    throw "Core/root web still consumes Dashboard UI source:`n$rootReferences"
}
if ($LASTEXITCODE -ne 1) {
    throw "Root Dashboard UI ownership audit failed to execute."
}

$composeOverride = Get-Content -LiteralPath `
    (Join-Path $repoRoot "deploy/sprint-6e/compose.override.yaml") -Raw
if ($composeOverride -notmatch "traefik\.http\.routers\.tessara-core\.entrypoints:\s*web") {
    throw "Sprint 6E must isolate the public Core router to the web entrypoint."
}
foreach ($slot in @("baseline", "candidate")) {
    $route = Get-Content -LiteralPath `
        (Join-Path $repoRoot "deploy/sprint-6e/dashboard-route.$slot.yaml") -Raw
    if ($route -notmatch "entryPoints:\s*\[module\]") {
        throw "Sprint 6E Dashboard slot '$slot' is not isolated to the module entrypoint."
    }
}
$slotSwitch = Get-Content -LiteralPath `
    (Join-Path $repoRoot "scripts/set-sprint-6e-dashboard-slot.ps1") -Raw
if ($slotSwitch -notmatch "docker kill --signal=HUP" `
    -or $slotSwitch -notmatch "gateway_restart_count") {
    throw "Sprint 6E slot switching must reload Traefik without restarting it."
}

Write-Host "Sprint 6E Dashboard source and package boundaries passed."

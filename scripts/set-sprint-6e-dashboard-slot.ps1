[CmdletBinding()]
param(
    [ValidateSet("baseline", "candidate")]
    [string]$Slot,
    [string]$ComposeFile = "deploy/sprint-6e/compose.yaml",
    [string]$EvidenceDirectory = "artifacts/sprint-6e-closeout"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$service = if ($Slot -eq "candidate") { "dashboards-candidate" } else { "dashboards" }
$routeSource = Join-Path $repoRoot "deploy/sprint-6e/dashboard-route.$Slot.yaml"
$routeDirectory = Join-Path $repoRoot "target/sprint-6e-routing"
$routeTarget = Join-Path $routeDirectory "dashboard.yaml"
$evidencePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceDirectory))

[IO.Directory]::CreateDirectory($routeDirectory) | Out-Null
[IO.Directory]::CreateDirectory($evidencePath) | Out-Null

$container = (& docker compose -f $composePath --profile candidate ps -q $service).Trim()
if ($LASTEXITCODE -ne 0 -or $container -notmatch "^[0-9a-f]{64}$") {
    throw "Dashboard slot '$Slot' is not running; active route was not changed."
}
$health = (& docker inspect --format "{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}" $container).Trim()
if ($LASTEXITCODE -ne 0 -or $health -ne "healthy") {
    $refusal = [ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        requested_slot = $Slot
        requested_container = $container
        health = $health
        result = "refused"
        active_route_changed = $false
    }
    [IO.File]::WriteAllText(
        (Join-Path $evidencePath "route-switch-refused-$Slot.json"),
        ($refusal | ConvertTo-Json -Depth 10) + "`n",
        [Text.UTF8Encoding]::new($false)
    )
    throw "Dashboard slot '$Slot' is not healthy; active route was not changed."
}

$temporary = Join-Path $routeDirectory "dashboard.$([Guid]::NewGuid().ToString('N')).tmp"
[IO.File]::WriteAllText(
    $temporary,
    [IO.File]::ReadAllText($routeSource),
    [Text.UTF8Encoding]::new($false)
)
Move-Item -LiteralPath $temporary -Destination $routeTarget -Force

# Docker Desktop propagates the atomically replaced bind-mounted file but may
# not forward the host filesystem notification to Traefik. SIGHUP makes
# Traefik reread the dynamic provider without restarting the gateway process.
$gatewayContainer = (& docker compose -f $composePath --profile candidate ps -q gateway).Trim()
if ($LASTEXITCODE -ne 0 -or $gatewayContainer -notmatch "^[0-9a-f]{64}$") {
    throw "Gateway is not running; Dashboard route notification failed."
}
$gatewayRestartCount = [int](& docker inspect --format "{{.RestartCount}}" $gatewayContainer).Trim()
& docker kill --signal=HUP $gatewayContainer | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Gateway did not accept the Dashboard route reload signal."
}
Start-Sleep -Milliseconds 500
$gatewayState = (& docker inspect --format "{{.State.Status}} {{.RestartCount}}" $gatewayContainer).Trim()
if ($LASTEXITCODE -ne 0 -or $gatewayState -ne "running $gatewayRestartCount") {
    throw "Gateway did not remain running without restart after the route reload signal."
}

$record = [ordered]@{
    schema_version = 1
    switched_at = [DateTimeOffset]::UtcNow.ToString("o")
    active_slot = $Slot
    container = $container
    health = $health
    gateway_container = $gatewayContainer
    gateway_restart_count = $gatewayRestartCount
    gateway_reload_signal = "HUP"
    route_sha256 = (Get-FileHash -LiteralPath $routeTarget -Algorithm SHA256).Hash.ToLowerInvariant()
}
[IO.File]::WriteAllText(
    (Join-Path $evidencePath "route-switch-$Slot.json"),
    ($record | ConvertTo-Json -Depth 10) + "`n",
    [Text.UTF8Encoding]::new($false)
)
Write-Host "Dashboard active slot switched to '$Slot'."

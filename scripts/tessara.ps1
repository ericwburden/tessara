[CmdletBinding()]
param(
    [Parameter(Position = 0, Mandatory = $true)]
    [ValidateSet("enrollment")]
    [string]$Area,

    [Parameter(Position = 1, Mandatory = $true)]
    [ValidateSet("issue", "recover")]
    [string]$Action,

    [switch]$Open,

    [string]$Reason,

    [string]$Operator = "$env:USERNAME@$env:COMPUTERNAME"
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$composeFile = Join-Path $repoRoot "deploy\sprint-6b2\compose.yaml"

if ($Area -ne "enrollment") {
    throw "Unsupported Tessara command area '$Area'."
}
if ($Action -eq "recover" -and [string]::IsNullOrWhiteSpace($Reason)) {
    throw "Administrator recovery requires -Reason."
}
if ($Action -eq "recover" -and [string]::IsNullOrWhiteSpace($Operator)) {
    throw "Administrator recovery requires -Operator."
}

$running = docker compose -f $composeFile ps --status running --services
if ($LASTEXITCODE -ne 0 -or $running -notcontains "core" -or $running -notcontains "installation-control") {
    throw "The Sprint 6B2 Core and installation-control services must be running."
}

$containerArguments = @(
    "compose", "-f", $composeFile,
    "run", "--rm", "--no-deps",
    "-e", "RUST_LOG=error",
    "installation-control",
    "enrollment", $Action
)
if ($Action -eq "recover") {
    $containerArguments += @("--reason", $Reason, "--operator", $Operator)
}

$rawOutput = (& docker @containerArguments | Out-String)
if ($LASTEXITCODE -ne 0) {
    throw "The enrollment Supervisor command failed."
}
try {
    $result = $rawOutput | ConvertFrom-Json
} catch {
    throw "The enrollment Supervisor returned an invalid response."
}

Write-Host ""
Write-Host "Administrator enrollment claim issued." -ForegroundColor Green
Write-Host "Installation: $($result.status.installation_id)"
Write-Host "Claim:        $($result.status.claim_id)"
Write-Host "Generation:   $($result.status.generation)"
Write-Host "Kind:         $($result.status.kind)"
Write-Host "Expires:      $($result.status.expires_at)"
Write-Host ""
Write-Host "Claim secret (shown once):" -ForegroundColor Yellow
Write-Host $result.claim_secret
Write-Host ""
Write-Host $result.warning -ForegroundColor Yellow
Write-Host "Enrollment page: $($result.enrollment_url)"

if ($Open) {
    Start-Process $result.enrollment_url
}

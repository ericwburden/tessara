[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$SupervisorUrl = "http://127.0.0.1:8095",
    [ValidateSet("reference", "reduced")][string]$Composition = "reference",
    [string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$lockfilePath = Join-Path $repoRoot "target/sprint-6f-bootstrap/$Composition/lockfile.json"
if (-not (Test-Path -LiteralPath $lockfilePath)) { throw "Missing resolved lockfile: $lockfilePath" }

$checks = [Collections.Generic.List[object]]::new()
function Assert-Check([string]$Name, [bool]$Passed, [string]$Detail) {
    $checks.Add([pscustomobject]@{ name = $Name; passed = $Passed; detail = $Detail })
    if (-not $Passed) { throw "Sprint 6F smoke failed: $Name - $Detail" }
}

$health = Invoke-WebRequest -Uri "$BaseUrl/health/ready" -UseBasicParsing
Assert-Check "core-ready" ($health.StatusCode -in 200,204) "HTTP $($health.StatusCode)"
$supervisorHealth = Invoke-WebRequest -Uri "$SupervisorUrl/health/ready" -UseBasicParsing
Assert-Check "supervisor-ready" ($supervisorHealth.StatusCode -in 200,204) "HTTP $($supervisorHealth.StatusCode)"

$lockfile = Get-Content -LiteralPath $lockfilePath -Raw | ConvertFrom-Json
$receipt = Invoke-RestMethod -Uri "$SupervisorUrl/v1/receipts/current"
Assert-Check "plan-identity" ($receipt.plan_digest -ceq $lockfile.materialization_plan_digest) $receipt.plan_digest
Assert-Check "installation-identity" ($receipt.installation_id -ceq $lockfile.installation_id) $receipt.installation_id

foreach ($action in @($lockfile.materialization_plan.actions | Where-Object action -eq "acquire_image")) {
    $observed = $receipt.observed_artifacts.PSObject.Properties[$action.component].Value
    Assert-Check "observed-artifact:$($action.component)" ($observed -ceq $action.digest) "locked=$($action.digest); observed=$observed"
}
foreach ($module in @($lockfile.modules)) {
    $observed = $receipt.observed_enablement.PSObject.Properties[$module.definition_id].Value
    Assert-Check "observed-enablement:$($module.definition_id)" ($observed -eq $module.enabled) "desired=$($module.enabled); observed=$observed"
}

$result = [pscustomobject][ordered]@{
    schema_version = 1
    checked_at = [DateTimeOffset]::UtcNow.ToString("o")
    composition = $Composition
    lockfile_path = $lockfilePath
    receipt_revision = $receipt.revision
    no_op = $receipt.no_op
    checks = $checks
    passed = $true
}
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    $fullOutput = [IO.Path]::GetFullPath((Join-Path $repoRoot $OutputPath))
    [IO.Directory]::CreateDirectory((Split-Path -Parent $fullOutput)) | Out-Null
    [IO.File]::WriteAllText($fullOutput, ($result | ConvertTo-Json -Depth 20) + "`n", [Text.UTF8Encoding]::new($false))
}
$result | ConvertTo-Json -Depth 20


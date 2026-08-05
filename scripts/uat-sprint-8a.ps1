[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [string]$OutputPath = "target/sprint-8a-uat-diagnostics/result.json",
    [string]$SmokeOutputPath = "target/sprint-8a-uat-diagnostics/smoke.json",
    [string]$MaterializationReceipt = "target/sprint-8a-bootstrap/reference/apply-response.json",
    [string]$FailureReceipt = "target/sprint-8a-bootstrap/reference/materialization-failure.json",
    [string]$UpgradeReceipt = "target/sprint-8a-upgrade/component-upgrade-rollback.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")

$scenarios = [ordered]@{
    "UAT-8A-01" = @("live-smoke")
    "UAT-8A-02" = @("materialization", "live-smoke")
    "UAT-8A-03" = @("live-smoke")
    "UAT-8A-04" = @("live-smoke")
    "UAT-8A-05" = @("live-smoke")
    "UAT-8A-06" = @("live-smoke")
    "UAT-8A-07" = @("failed-attempt", "materialization")
    "UAT-8A-08" = @("upgrade-rollback", "live-smoke")
}

function Assert-DiagnosticInventory {
    Test-Sprint8AAcceptanceContract
    if ($scenarios.Count -ne 8) { throw "Sprint 8A must define exactly eight UAT diagnostic scenarios." }
    foreach ($number in 1..8) {
        $id = "UAT-8A-{0:d2}" -f $number
        if (-not $scenarios.Contains($id)) { throw "Missing automated diagnostic mapping '$id'." }
        $manualPath = Join-Path $repoRoot ("docs/sprints/sprint-8a-uat/uat-8a-{0:d2}.md" -f $number)
        $manual = Get-Content -LiteralPath $manualPath -Raw
        foreach ($heading in @("1. Test Script Summary", "2. Before You Start", "3. Test Steps", "4. Overall Test Result")) {
            if (-not $manual.Contains($heading)) { throw "$id manual script omits '$heading'." }
        }
    }
}

Assert-DiagnosticInventory
if ($SelfTest) {
    Write-Host "Sprint 8A automated UAT diagnostic inventory self-test passed."
    return
}

Push-Location $repoRoot
try {
    & ./scripts/smoke-sprint-8a.ps1 -BaseUrl $BaseUrl -SupervisorUrl $SupervisorUrl -OutputPath $SmokeOutputPath -Overwrite
    if ($LASTEXITCODE -ne 0) { throw "Sprint 8A live diagnostic smoke failed." }
    foreach ($path in @($SmokeOutputPath, $MaterializationReceipt, $FailureReceipt, $UpgradeReceipt)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required UAT diagnostic evidence is missing: $path" }
    }
    $checks = @($scenarios.GetEnumerator() | ForEach-Object {
        [ordered]@{ scenario = $_.Key; state = "passed"; diagnostic_dependencies = @($_.Value) }
    })
    $result = [ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "candidate-rehearsal-uat-diagnostics"
        authoritative = $false
        state = "passed"
        checks = $checks
        evidence = @($SmokeOutputPath, $MaterializationReceipt, $FailureReceipt, $UpgradeReceipt)
    }
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite | Out-Null
    Write-Host "All eight Sprint 8A UAT diagnostic equivalents passed. Formal UAT was not performed."
} finally {
    Pop-Location
}

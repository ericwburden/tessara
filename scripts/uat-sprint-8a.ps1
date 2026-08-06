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

function Assert-UpgradeReceiptFresh {
    param(
        [Parameter(Mandatory)]$Upgrade,
        [Parameter(Mandatory)]$Materialization,
        [Parameter(Mandatory)][string]$ExpectedCommit,
        [Parameter(Mandatory)][string]$ExpectedTree
    )
    if ([string]$Upgrade.source_identity.commit -cne $ExpectedCommit -or
        [string]$Upgrade.source_identity.tree -cne $ExpectedTree -or
        [bool]$Upgrade.source_identity.dirty) {
        throw "Component upgrade evidence is not bound to the current clean source identity."
    }
    if ([DateTimeOffset]::Parse([string]$Upgrade.generated_at) -lt
        [DateTimeOffset]::Parse([string]$Materialization.operation.updated_at)) {
        throw "Component upgrade evidence predates the current materialization receipt."
    }
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
    $materialization = [pscustomobject]@{ operation = [pscustomobject]@{ updated_at = "2026-01-02T00:00:00Z" } }
    $upgrade = [pscustomobject]@{
        generated_at = "2026-01-02T00:00:01Z"
        source_identity = [pscustomobject]@{ commit = "a"; tree = "b"; dirty = $false }
    }
    Assert-UpgradeReceiptFresh -Upgrade $upgrade -Materialization $materialization -ExpectedCommit "a" -ExpectedTree "b"
    $upgrade.generated_at = "2026-01-01T23:59:59Z"
    try {
        Assert-UpgradeReceiptFresh -Upgrade $upgrade -Materialization $materialization -ExpectedCommit "a" -ExpectedTree "b"
        throw "Stale upgrade evidence was accepted."
    } catch {
        if ($_.Exception.Message -ceq "Stale upgrade evidence was accepted.") { throw }
    }
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
    $materialization = Get-Content -LiteralPath $MaterializationReceipt -Raw | ConvertFrom-Json
    $upgrade = Get-Content -LiteralPath $UpgradeReceipt -Raw | ConvertFrom-Json
    $expectedCommit = (& git -C $repoRoot rev-parse HEAD).Trim()
    $expectedTree = (& git -C $repoRoot rev-parse "HEAD^{tree}").Trim()
    Assert-UpgradeReceiptFresh -Upgrade $upgrade -Materialization $materialization -ExpectedCommit $expectedCommit -ExpectedTree $expectedTree
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

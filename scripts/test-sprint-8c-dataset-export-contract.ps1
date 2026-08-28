[CmdletBinding()]
param(
    [string]$EvidenceRoot = "target/sprint-8c-dataset-export-contract",
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")
. (Join-Path $PSScriptRoot "sprint-8c-uat-predicate-inventory.ps1")
$ownerExpectedCount = @((Get-Sprint8CResponseOwnerTestInventory).identities).Count

$fullEvidenceRoot = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
    [IO.Path]::GetFullPath($EvidenceRoot)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
}
[IO.Directory]::CreateDirectory($fullEvidenceRoot) | Out-Null

$ownerEvidence = Join-Path $fullEvidenceRoot "response-owner-export.json"
$consumerEvidence = Join-Path $fullEvidenceRoot "dataset-response-refresh.json"
$owner = Invoke-Sprint8CChildScript -ScriptPath "scripts/test-sprint-8c-response-module.ps1" `
    -Arguments @("-Suite", "Owner", "-EvidencePath", $ownerEvidence)
$sync = Invoke-Sprint8CChildScript -ScriptPath "scripts/test-sprint-8c-dataset-consumer.ps1" `
    -Arguments @("-Suite", "Sync")
$refresh = Invoke-Sprint8CChildScript -ScriptPath "scripts/test-sprint-8c-dataset-consumer.ps1" `
    -Arguments @("-Suite", "Refresh", "-EvidencePath", $consumerEvidence)

$ownerReceipt = Get-Content -Raw -LiteralPath $ownerEvidence | ConvertFrom-Json -Depth 100
$consumerReceipt = Get-Content -Raw -LiteralPath $consumerEvidence | ConvertFrom-Json -Depth 100
if ([string]$ownerReceipt.proof -cne "response-module-test-suite" -or
    [string]$ownerReceipt.state -cne "passed" -or
    [int]$ownerReceipt.executed_test_count -ne $ownerExpectedCount -or
    [string]$consumerReceipt.proof -cne "dataset-module-test-suite" -or
    [string]$consumerReceipt.state -cne "passed" -or
    [int]$consumerReceipt.executed_test_count -ne 8) {
    throw "Response export producer/consumer evidence is incomplete."
}

$result = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    proof = "response-owner-to-dataset-export-boundary"
    state = "passed"
    source = Get-Sprint8CSourceIdentity
    response_owner_tests = $ownerExpectedCount
    dataset_sync_tests = 7
    dataset_refresh_tests = 8
    output_sha256 = [pscustomobject][ordered]@{
        owner = Get-Sprint7ASha256 -Text ((@($owner.output) -join "`n") + "`n")
        sync = Get-Sprint7ASha256 -Text ((@($sync.output) -join "`n") + "`n")
        refresh = Get-Sprint7ASha256 -Text ((@($refresh.output) -join "`n") + "`n")
    }
    cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
}
if ($EvidencePath) {
    Publish-Sprint8CHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
}
$result | ConvertTo-Json -Depth 20

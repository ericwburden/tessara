[CmdletBinding()]
param(
    [string]$AttemptPath,
    [string]$HarvestPath,
    [string]$DefectBatchPath,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-HarvestComplete {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Harvest,
        [Parameter(Mandatory)]$Batch
    )

    if ([string]$Harvest.state -cne "harvest_complete") {
        throw "Correction/restart is forbidden until the attempt reaches harvest_complete."
    }
    $declared = @($Attempt.checks)
    if ($declared.Count -eq 0) { throw "The attempt must declare its check dependency graph before assertions." }
    $terminal = @($Harvest.checks)
    foreach ($check in $declared) {
        $result = @($terminal | Where-Object name -CEQ ([string]$check.name))
        if ($result.Count -ne 1) { throw "Check '$($check.name)' does not have exactly one terminal result." }
        if (@("passed", "failed", "blocked") -cnotcontains [string]$result[0].state) {
            throw "Check '$($check.name)' is not passed, failed, or blocked."
        }
        if ([string]$result[0].state -ceq "blocked" -and [string]::IsNullOrWhiteSpace([string]$result[0].dependency_reason)) {
            throw "Blocked check '$($check.name)' has no exact dependency reason."
        }
    }
    if (@($terminal).Count -ne $declared.Count) { throw "Harvest contains undeclared or duplicate check results." }
    if ([int]$Batch.batch -ne 1 -or [string]$Batch.harvest_receipt -cne [string]$Harvest.receipt_path) {
        throw "The diagnostic pass must produce exactly one batch bound to its harvest receipt."
    }
}

if ($SelfTest) {
    $attempt = [pscustomobject]@{ checks = @(
        [pscustomobject]@{ name = "independent"; depends_on = @() },
        [pscustomobject]@{ name = "dependent"; depends_on = @("independent") }
    ) }
    $harvest = [pscustomobject]@{
        state = "harvest_complete"
        receipt_path = "attempts/harvest.json"
        checks = @(
            [pscustomobject]@{ name = "independent"; state = "failed"; dependency_reason = $null },
            [pscustomobject]@{ name = "dependent"; state = "blocked"; dependency_reason = "independent failed" }
        )
    }
    $batch = [pscustomobject]@{ batch = 1; harvest_receipt = "attempts/harvest.json" }
    Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch
    $harvest.checks[1].dependency_reason = ""
    try {
        Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch
        throw "Self-test failed: a blocked check without a dependency reason was accepted."
    } catch {
        if ($_.Exception.Message -like "Self-test failed:*") { throw }
    }
    Write-Host "Sprint validation harvest guard self-test passed."
    return
}

foreach ($path in @($AttemptPath, $HarvestPath, $DefectBatchPath)) {
    if ([string]::IsNullOrWhiteSpace($path) -or -not (Test-Path -LiteralPath $path)) {
        throw "Attempt, harvest, and defect-batch paths are required and must exist."
    }
}
$attempt = Get-Content -LiteralPath $AttemptPath -Raw | ConvertFrom-Json
$harvest = Get-Content -LiteralPath $HarvestPath -Raw | ConvertFrom-Json
$batch = Get-Content -LiteralPath $DefectBatchPath -Raw | ConvertFrom-Json
Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch
Write-Host "Validation harvest is complete; consolidated correction/restart is permitted."

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$contractPath = Join-Path $repoRoot "docs/sprints/sprint-8c-validation-contract.json"
$planPath = Join-Path $repoRoot "docs/sprints/sprint-8c-plan.md"
$verificationPath = Join-Path $repoRoot "docs/sprints/sprint-8c-verification.md"
Import-Module (Join-Path $PSScriptRoot "tessara-validation-policy.psm1") -Force

function Assert-ExactSequence {
    param([string[]]$Expected, [string[]]$Actual, [string]$Label)
    if (($Expected -join "`n") -cne ($Actual -join "`n")) {
        throw "$Label is not the exact frozen sequence."
    }
}

$contract = Get-Content -Raw -LiteralPath $contractPath | ConvertFrom-Json -Depth 100
if ([int]$contract.schema_version -ne 2 -or
    [string]$contract.contract -cne "tessara.validation-contract" -or
    [string]$contract.sprint -cne "sprint-8c") {
    throw "Sprint 8C validation contract identity is invalid."
}
$null = Assert-TessaraValidationContract -Contract $contract

$expectedRequirements = @(1..16 | ForEach-Object { "ac-{0:d2}" -f $_ }) +
    @("gate-implementation-exit")
$expectedTargets = @(
    "static-quality", "contract-boundary", "owner-product", "ui-sdk-conformance",
    "consumer-cutover", "core-subtraction", "inventory-navigation", "migration-seed",
    "clean-materialization", "semantic-noop", "failure-recovery", "fixture-acceptance",
    "runner-selftest", "deployed-smoke", "independent-upgrade-rollback", "uat-readiness",
    "provider-contracts", "workflow-events", "dataset-export", "assignment-only",
    "scoped-review", "api-idempotency", "planning-contract-alignment"
)
$requirementIds = @($contract.requirements | ForEach-Object { [string]$_.id })
$targetIds = @($contract.implementation_targets | ForEach-Object { [string]$_.id })
$laneIds = @($contract.lanes | ForEach-Object { [string]$_.id })
Assert-ExactSequence -Expected $expectedRequirements -Actual $requirementIds -Label "Requirement order"
Assert-ExactSequence -Expected $expectedTargets -Actual $targetIds -Label "Implementation target order"
if ($laneIds.Count -ne 31 -or @($laneIds | Sort-Object -Unique).Count -ne 31) {
    throw "Sprint 8C must retain exactly 31 unique validation lanes."
}

foreach ($target in $contract.implementation_targets) {
    $expectedCommand = ".\scripts\run-sprint-8c-implementation-readiness.ps1 -Target $($target.id)"
    if ([string]$target.command -cne $expectedCommand) {
        throw "Target '$($target.id)' does not select itself exactly."
    }
    foreach ($domain in @("implementation-runner", "implementation-harness")) {
        if (@($target.dependency_domains) -cnotcontains $domain) {
            throw "Target '$($target.id)' does not consume shared domain '$domain'."
        }
    }
}
foreach ($requirement in $contract.requirements) {
    foreach ($target in @($requirement.implementation_targets)) {
        if ($targetIds -cnotcontains [string]$target) {
            throw "Requirement '$($requirement.id)' references unknown target '$target'."
        }
    }
    foreach ($lane in @($requirement.validation_lanes)) {
        if ($laneIds -cnotcontains [string]$lane) {
            throw "Requirement '$($requirement.id)' references unknown lane '$lane'."
        }
    }
}

$plan = Get-Content -Raw -LiteralPath $planPath
$verification = Get-Content -Raw -LiteralPath $verificationPath
foreach ($number in 1..16) {
    $identity = "AC-{0:d2}" -f $number
    if ($plan -cnotmatch [regex]::Escape($identity) -or
        $verification -cnotmatch [regex]::Escape($identity)) {
        throw "$identity is not represented in both the plan and verification matrix."
    }
}
foreach ($target in $contract.implementation_targets) {
    if ($plan -cnotmatch [regex]::Escape([string]$target.command) -or
        $verification -cnotmatch [regex]::Escape([string]$target.command)) {
        throw "Target '$($target.id)' is not command-exact in both planning documents."
    }
}

foreach ($relativePath in @(
    "docs/sprints/sprint-8c-plan.md",
    "docs/sprints/sprint-8c-verification.md",
    "docs/sprints/sprint-8c-validation-contract.json",
    "docs/audits/sprint-8c-response-ui-baseline/baseline-index.json",
    "deploy/sprint-8c/fixtures/reference-fixture-contract.json",
    "scripts/check-sprint-8c-response-boundaries.ps1",
    "scripts/capture-sprint-8c-ui-baseline.mjs"
)) {
    if (-not (Test-Path -LiteralPath (Join-Path $repoRoot $relativePath) -PathType Leaf)) {
        throw "Required Sprint 8C planning identity '$relativePath' is missing."
    }
}

$productImpact = Get-TessaraValidationImpact -Contract $contract `
    -ChangedPaths @("crates/tessara-response-module/src/product_store.rs") -CandidateChanged
if (@($productImpact.unknown_paths).Count -ne 0 -or
    @($productImpact.changed_domains) -cnotcontains "product-source" -or
    @($productImpact.phase_decisions | Where-Object {
        [string]$_.phase -in @("sit", "uat") -and [string]$_.action -ceq "rerun_full_phase"
    }).Count -ne 2) {
    throw "Response product changes do not conservatively invalidate successor SIT and UAT."
}
$unknownImpact = Get-TessaraValidationImpact -Contract $contract `
    -ChangedPaths @("unmapped/sprint-8c-input.bin")
if (@($unknownImpact.unknown_paths).Count -ne 1 -or
    @($unknownImpact.phase_decisions | Where-Object {
        [string]$_.action -cne "rerun_full_phase"
    }).Count -ne 0) {
    throw "Unknown Sprint 8C paths do not conservatively invalidate all phases."
}

[pscustomobject][ordered]@{
    sprint = "sprint-8c"
    requirements = $requirementIds.Count
    implementation_targets = $targetIds.Count
    validation_lanes = $laneIds.Count
    status = "passed"
} | ConvertTo-Json

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$contractPath = Join-Path $repoRoot "docs/sprints/sprint-8b-validation-contract.json"
$planPath = Join-Path $repoRoot "docs/sprints/sprint-8b-plan.md"
$verificationPath = Join-Path $repoRoot "docs/sprints/sprint-8b-verification.md"
$policyPath = Join-Path $PSScriptRoot "tessara-validation-policy.psm1"
Import-Module $policyPath -Force

function Get-UniqueOrderedValues {
    param([Parameter(Mandatory)][object[]]$Values, [Parameter(Mandatory)][string]$Label)
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $ordered = [Collections.Generic.List[string]]::new()
    foreach ($value in $Values) {
        $text = [string]$value
        if (-not $seen.Add($text)) { throw "$Label contains duplicate '$text'." }
        $ordered.Add($text)
    }
    @($ordered)
}

function Assert-ExactSequence {
    param([string[]]$Expected, [string[]]$Actual, [string]$Label)
    if ($Expected.Count -ne $Actual.Count) {
        throw "$Label count mismatch: expected $($Expected.Count), found $($Actual.Count)."
    }
    for ($index = 0; $index -lt $Expected.Count; $index++) {
        if ($Expected[$index] -cne $Actual[$index]) {
            throw "$Label mismatch at index ${index}: expected '$($Expected[$index])', found '$($Actual[$index])'."
        }
    }
}

function Get-BacktickIds {
    param([Parameter(Mandatory)][string]$Text)
    @([regex]::Matches($Text, '`([a-z][a-z0-9-]+)`') | ForEach-Object { $_.Groups[1].Value })
}

function Get-RequirementMap {
    param(
        [Parameter(Mandatory)][string]$Text,
        [Parameter(Mandatory)][string]$Heading,
        [Parameter(Mandatory)][int]$TargetColumn,
        [Parameter(Mandatory)][int]$LaneColumn
    )
    $headingIndex = $Text.IndexOf($Heading, [StringComparison]::Ordinal)
    if ($headingIndex -lt 0) { throw "Missing planning heading '$Heading'." }
    $tail = $Text.Substring($headingIndex + $Heading.Length)
    $nextHeading = $tail.IndexOf("`n## ", [StringComparison]::Ordinal)
    if ($nextHeading -ge 0) { $tail = $tail.Substring(0, $nextHeading) }
    $map = [ordered]@{}
    foreach ($line in ($tail -split "`r?`n")) {
        if ($line -notmatch '^\| `(?<id>ac-\d{2})` \|') { continue }
        $columns = @($line.Split('|') | ForEach-Object { $_.Trim() })
        $id = $Matches.id
        if ($map.Contains($id)) { throw "Duplicate '$id' row below '$Heading'." }
        $map[$id] = [ordered]@{
            targets = @(Get-BacktickIds -Text $columns[$TargetColumn])
            lanes = @(Get-BacktickIds -Text $columns[$LaneColumn])
        }
    }
    $map
}

$contract = Get-Content -Raw -LiteralPath $contractPath | ConvertFrom-Json -Depth 100
if ($contract.schema_version -ne 2 -or $contract.contract -cne "tessara.validation-contract") {
    throw "Sprint 8B validation contract has the wrong schema or contract identity."
}

$requirementIds = @(Get-UniqueOrderedValues -Values @($contract.requirements.id) -Label "contract requirements")
$targetIds = @(Get-UniqueOrderedValues -Values @($contract.implementation_targets.id) -Label "implementation targets")
$laneIds = @(Get-UniqueOrderedValues -Values @($contract.lanes.id) -Label "validation lanes")
$expectedRequirements = @(1..26 | ForEach-Object { "ac-{0:d2}" -f $_ }) + @("gate-implementation-exit")
Assert-ExactSequence -Expected $expectedRequirements -Actual $requirementIds -Label "requirement order"
if ($targetIds.Count -ne 24) { throw "Expected 24 implementation targets, found $($targetIds.Count)." }
if ($laneIds.Count -ne 31) { throw "Expected 31 validation lanes, found $($laneIds.Count)." }

foreach ($target in $contract.implementation_targets) {
    $expectedCommand = ".\scripts\run-sprint-8b-implementation-readiness.ps1 -Target $($target.id)"
    if ($target.command -cne $expectedCommand) {
        throw "Target '$($target.id)' command is '$($target.command)', expected '$expectedCommand'."
    }
}
foreach ($requirement in $contract.requirements) {
    foreach ($target in $requirement.implementation_targets) {
        if ($targetIds -cnotcontains $target) { throw "Requirement '$($requirement.id)' references unknown target '$target'." }
    }
    foreach ($lane in $requirement.validation_lanes) {
        if ($laneIds -cnotcontains $lane) { throw "Requirement '$($requirement.id)' references unknown lane '$lane'." }
    }
}

$plan = Get-Content -Raw -LiteralPath $planPath
$verification = Get-Content -Raw -LiteralPath $verificationPath
$planMap = Get-RequirementMap -Text $plan -Heading "## Canonical Requirement-To-Proof Crosswalk" -TargetColumn 3 -LaneColumn 4
$verificationMap = Get-RequirementMap -Text $verification -Heading "### Exact requirement-to-proof mapping" -TargetColumn 2 -LaneColumn 3

foreach ($requirement in @($contract.requirements | Where-Object id -like 'ac-*')) {
    $id = [string]$requirement.id
    if (-not $planMap.Contains($id)) { throw "Plan crosswalk is missing '$id'." }
    if (-not $verificationMap.Contains($id)) { throw "Verification crosswalk is missing '$id'." }
    Assert-ExactSequence -Expected @($requirement.implementation_targets) -Actual @($planMap[$id].targets) -Label "plan targets for $id"
    Assert-ExactSequence -Expected @($requirement.validation_lanes) -Actual @($planMap[$id].lanes) -Label "plan lanes for $id"
    Assert-ExactSequence -Expected @($requirement.implementation_targets) -Actual @($verificationMap[$id].targets) -Label "verification targets for $id"
    Assert-ExactSequence -Expected @($requirement.validation_lanes) -Actual @($verificationMap[$id].lanes) -Label "verification lanes for $id"
}

$fixturePaths = @(
    "deploy/sprint-8b/fixtures/reference-fixture-contract.json",
    "deploy/sprint-8b/fixtures/provider-fault-contract.json",
    "deploy/sprint-8b/fixtures/upgrade-fixture-contract.json",
    "docs/audits/sprint-8b-dataset-ui-baseline/baseline-index.json",
    "docs/sprints/sprint-8b-uat/scenario-contract.json",
    "docs/sprints/sprint-8b-test-change-log.md",
    "end2end/acceptance-manifest.json"
)
foreach ($relativePath in $fixturePaths) {
    if (-not (Test-Path -LiteralPath (Join-Path $repoRoot $relativePath) -PathType Leaf)) {
        throw "Tracked Sprint 8B fixture/acceptance identity '$relativePath' is missing."
    }
}
$scenarioContract = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8b-uat/scenario-contract.json") | ConvertFrom-Json -Depth 100
$scenarioIds = @(Get-UniqueOrderedValues -Values @($scenarioContract.scenarios.id) -Label "UAT scenarios")
$expectedScenarioIds = @(1..11 | ForEach-Object { "UAT-8B-{0:d2}" -f $_ })
Assert-ExactSequence -Expected $expectedScenarioIds -Actual $scenarioIds -Label "UAT scenario order"
$manualHeading = "## Manual UAT Inventory"
$manualStart = $plan.IndexOf($manualHeading, [StringComparison]::Ordinal)
if ($manualStart -lt 0) { throw "Plan is missing the manual UAT inventory." }
$manualTail = $plan.Substring($manualStart + $manualHeading.Length)
$manualEnd = $manualTail.IndexOf("`n## ", [StringComparison]::Ordinal)
if ($manualEnd -ge 0) { $manualTail = $manualTail.Substring(0, $manualEnd) }
$planScenarioIds = @([regex]::Matches($manualTail, '^\| (UAT-8B-\d{2}) ', [Text.RegularExpressions.RegexOptions]::Multiline) | ForEach-Object { $_.Groups[1].Value })
Assert-ExactSequence -Expected $expectedScenarioIds -Actual $planScenarioIds -Label "plan UAT scenario order"

$null = Assert-TessaraValidationContract -Contract $contract
function Assert-ImpactDomains {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string[]]$ExpectedDomains,
        [switch]$CandidateChanged
    )
    $impact = Get-TessaraValidationImpact `
        -Contract $contract `
        -ChangedPaths @($Path) `
        -CandidateChanged:$CandidateChanged
    Assert-ExactSequence `
        -Expected @($ExpectedDomains | Sort-Object) `
        -Actual @($impact.changed_domains | Sort-Object) `
        -Label "impact domains for $Path"
    if (@($impact.unknown_paths).Count -ne 0) {
        throw "Representative path '$Path' was unexpectedly unmapped."
    }
    $impact
}

$contractImpact = Assert-ImpactDomains `
    -Path "docs/sprints/sprint-8b-validation-contract.json" `
    -ExpectedDomains @("documentation", "sprint-contract")
foreach ($decision in @($contractImpact.phase_decisions)) {
    if ([string]$decision.action -ceq "reuse_certificate") {
        throw "A validation-contract change incorrectly retained the '$($decision.phase)' certificate."
    }
}

$productImpact = Assert-ImpactDomains `
    -Path "crates/tessara-dataset-module/src/product.rs" `
    -ExpectedDomains @("product-source") `
    -CandidateChanged
if (@($productImpact.phase_decisions | Where-Object {
    [string]$_.phase -ceq "sit" -and [string]$_.action -ceq "rerun_full_phase"
}).Count -ne 1 -or @($productImpact.phase_decisions | Where-Object {
    [string]$_.phase -ceq "uat" -and [string]$_.action -ceq "rerun_full_phase"
}).Count -ne 1) {
    throw "A Dataset product change must require complete successor SIT and UAT."
}

$materializerImpact = Assert-ImpactDomains `
    -Path "scripts/materialize-sprint-8b.ps1" `
    -ExpectedDomains @("deployment-materialization", "implementation-harness", "migrations-seeds")
$acceptanceImpact = Assert-ImpactDomains `
    -Path "docs/sprints/sprint-8b-test-change-log.md" `
    -ExpectedDomains @("acceptance-inventory", "documentation")
$helperPaths = @(
    "scripts/check-web-crate-boundaries.ps1",
    "scripts/verify-module-sdk-boundaries.ps1",
    "scripts/ui-sdk-conformance.ps1",
    "scripts/build-module-ui-browser-assets.ps1",
    "scripts/run-sprint-8b-dataset-browser-proof.ps1"
)
foreach ($helperPath in $helperPaths) {
    $null = Assert-ImpactDomains -Path $helperPath -ExpectedDomains @("implementation-harness")
}

$additionalTrackedPaths = @(
    [pscustomobject]@{ Path = "README.md"; Domains = @("documentation") },
    [pscustomobject]@{ Path = "deploy/sprint-8b/catalogs/catalog-dev-v1.public.hex"; Domains = @("deployment-materialization", "environment-contract") },
    [pscustomobject]@{ Path = "scripts/assert-sprint-8b-planning-contract.ps1"; Domains = @("implementation-harness", "implementation-runner") },
    [pscustomobject]@{ Path = "scripts/bootstrap-sprint-7a-composition.ps1"; Domains = @("deployment-materialization", "implementation-harness") },
    [pscustomobject]@{ Path = "scripts/build-sprint-8b-dataset-upgrade-baseline.ps1"; Domains = @("deployment-materialization", "implementation-harness") },
    [pscustomobject]@{ Path = "scripts/capture-sprint-8b-ui-baseline.mjs"; Domains = @("acceptance-inventory", "implementation-harness") },
    [pscustomobject]@{ Path = "scripts/check-sprint-8b-dataset-boundaries.ps1"; Domains = @("implementation-harness") },
    [pscustomobject]@{ Path = "scripts/sprint-8b-cargo-test-integrity.ps1"; Domains = @("implementation-harness") },
    [pscustomobject]@{ Path = "scripts/test-sprint-8b-component-consumer.ps1"; Domains = @("implementation-harness") },
    [pscustomobject]@{ Path = "scripts/test-sprint-8b-dataset-module.ps1"; Domains = @("implementation-harness") },
    [pscustomobject]@{ Path = "scripts/test-sprint-8b-response-export-contract.ps1"; Domains = @("implementation-harness") },
    [pscustomobject]@{ Path = "scripts/validate-e2e.ps1"; Domains = @("acceptance-inventory", "implementation-harness") },
    [pscustomobject]@{ Path = "scripts/verify-sprint-8b-response-owner-receipt.mjs"; Domains = @("implementation-harness") }
)
foreach ($trackedPath in $additionalTrackedPaths) {
    $null = Assert-ImpactDomains -Path $trackedPath.Path -ExpectedDomains $trackedPath.Domains
}

$unknownImpact = Get-TessaraValidationImpact `
    -Contract $contract `
    -ChangedPaths @("unmapped/sprint-8b-input.bin")
if (@($unknownImpact.unknown_paths).Count -ne 1 -or
    @($unknownImpact.phase_decisions | Where-Object { [string]$_.action -cne "rerun_full_phase" }).Count -ne 0) {
    throw "Unknown-path impact must conservatively select every complete phase."
}

foreach ($domain in @("validation-shared", "sprint-contract", "implementation-runner")) {
    $consumers = @($contract.implementation_targets | Where-Object {
        @($_.dependency_domains) -ccontains $domain
    })
    if ($consumers.Count -ne 24) {
        throw "Every implementation target must consume '$domain'; found $($consumers.Count) of 24."
    }
}

[pscustomobject]@{
    sprint = "sprint-8b"
    requirements = $requirementIds.Count
    implementation_targets = $targetIds.Count
    validation_lanes = $laneIds.Count
    uat_scenarios = $scenarioIds.Count
    status = "passed"
} | ConvertTo-Json

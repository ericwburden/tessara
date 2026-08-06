[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
    [string]$OutputPath = "target/sprint-8a-uat-diagnostics/result.json",
    [string]$MaterializationLaneReceipt,
    [string]$InventoryLaneReceipt,
    [string]$DeploymentEvidenceLaneReceipt,
    [string]$ProductSmokeLaneReceipt,
    [string]$FailureContainmentLaneReceipt,
    [string]$UpgradeLaneReceipt,
    [string]$ComponentConformanceLaneReceipt,
    [string]$PlaywrightLaneReceipt,
    [string]$ManifestContractLaneReceipt,
    [string]$WebBoundaryLaneReceipt,
    [string]$DashboardBoundaryLaneReceipt,
    [string]$ProductDiagnosticLaneReceipt,
    [string]$ProductDiagnosticEvidence,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-dashboard-dependency-contract.ps1")

$scenarioDependencies = [ordered]@{
    "UAT-8A-01" = @("successor-deployment-evidence", "successor-product-smoke", "live-product-diagnostics", "playwright-execution")
    "UAT-8A-02" = @("source-exact-materialization-no-op", "successor-inventory-navigation-audit", "successor-deployment-evidence", "successor-product-smoke")
    "UAT-8A-03" = @("successor-deployment-evidence", "successor-product-smoke", "playwright-execution")
    "UAT-8A-04" = @("successor-deployment-evidence", "successor-product-smoke", "component-conformance-nondisclosure", "playwright-execution")
    "UAT-8A-05" = @("successor-deployment-evidence", "successor-product-smoke", "live-product-diagnostics", "playwright-execution")
    "UAT-8A-06" = @("successor-inventory-navigation-audit", "successor-deployment-evidence", "successor-product-smoke", "compose-manifest-schema-contract", "web-native-wasm-source-boundaries", "dashboard-source-boundaries")
    "UAT-8A-07" = @("failure-containment-successor-health", "successor-deployment-evidence", "successor-product-smoke")
    "UAT-8A-08" = @("component-upgrade-rollback", "successor-deployment-evidence", "successor-product-smoke")
}
$scenarioAssertions = [ordered]@{
    "UAT-8A-01" = @("component-module-live-script", "complete-browser-inventory", "module-owned-documents-and-assets")
    "UAT-8A-02" = @("empty-first-apply", "semantic-no-op", "exact-five-core-transitions", "receipt-bound-dashboard-references")
    "UAT-8A-03" = @("configuration-schema-authority", "label-navigation-projection", "sanitized-diagnostics")
    "UAT-8A-04" = @("dataset-contract-execution", "known-random-nondisclosure", "timeout-outage-recovery")
    "UAT-8A-05" = @("dashboard-lifecycle-findings", "consumer-actions", "provider-outage-containment")
    "UAT-8A-06" = @("core-component-absence", "native-wasm-source-boundaries", "old-input-rejection", "exact-real-module-inventory")
    "UAT-8A-07" = @("induced-owner-failure", "exact-teardown", "empty-successor", "successor-no-op-health")
    "UAT-8A-08" = @("component-only-upgrade", "rollback", "unrelated-identity-stability", "intended-release-restoration")
}
$requirementMappings = [ordered]@{
    "UAT-8A-01" = "Sprint 8A AC-01, AC-07, and AC-15"
    "UAT-8A-02" = "Sprint 8A AC-03, AC-04, AC-05, and AC-16"
    "UAT-8A-03" = "Sprint 8A AC-08"
    "UAT-8A-04" = "Sprint 8A AC-09 and AC-10"
    "UAT-8A-05" = "Sprint 8A AC-11"
    "UAT-8A-06" = "Sprint 8A AC-01, AC-02, AC-06, AC-12, and AC-16"
    "UAT-8A-07" = "Sprint 8A AC-13"
    "UAT-8A-08" = "Sprint 8A AC-14"
}

function Assert-DiagnosticInventory {
    if ($scenarioDependencies.Count -ne 8 -or $scenarioAssertions.Count -ne 8 -or $requirementMappings.Count -ne 8) {
        throw "Sprint 8A must define exactly eight UAT diagnostic scenarios and requirement mappings."
    }
    foreach ($number in 1..8) {
        $id = "UAT-8A-{0:d2}" -f $number
        if (-not $scenarioDependencies.Contains($id) -or -not $scenarioAssertions.Contains($id) -or -not $requirementMappings.Contains($id)) {
            throw "Missing automated diagnostic mapping '$id'."
        }
    }
}

function Assert-RepositoryDiagnosticContract {
    Test-Sprint8AAcceptanceContract
    foreach ($number in 1..8) {
        $id = "UAT-8A-{0:d2}" -f $number
        $manualPath = Join-Path $repoRoot ("docs/sprints/sprint-8a-uat/uat-8a-{0:d2}.md" -f $number)
        $manual = Get-Content -LiteralPath $manualPath -Raw
        foreach ($heading in @("1. Test Script Summary", "2. Before You Start", "3. Test Steps", "4. Overall Test Result")) {
            if (-not $manual.Contains($heading)) { throw "$id manual script omits '$heading'." }
        }
        $expectedRequirement = "- Requirement: $($requirementMappings[$id])"
        if (-not $manual.Contains($expectedRequirement)) {
            throw "$id must map exactly to '$expectedRequirement'."
        }
    }
}

function Assert-LaneReceiptObject {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][string]$ExpectedName,
        [Parameter(Mandatory)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$ExpectedEnvironment,
        [Parameter(Mandatory)]$ExpectedSource
    )

    $schema = $Receipt.schema_version
    if ($Receipt.PSObject.Properties.Name -notcontains "authoritative" -or
        -not ($schema -is [int] -or $schema -is [long]) -or
        [long]$schema -ne 1 -or
        $Receipt.sprint -isnot [string] -or
        [string]$Receipt.sprint -cne "sprint-8a" -or
        $Receipt.authoritative -isnot [bool] -or
        $Receipt.authoritative -ne $false -or
        $Receipt.phase -isnot [string] -or
        [string]$Receipt.phase -cne "candidate-rehearsal-lane" -or
        [int]$Receipt.attempt -ne $ExpectedAttempt -or
        $ExpectedEnvironment -notmatch '^[0-9a-f]{64}$' -or
        $Receipt.environment_fingerprint -isnot [string] -or
        [string]$Receipt.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$Receipt.environment_fingerprint -cne $ExpectedEnvironment -or
        [string]$Receipt.result.name -cne $ExpectedName) {
        throw "Lane '$ExpectedName' is not one terminal receipt for this rehearsal attempt/environment."
    }
    $expectedSourceProperties = @(
        "commit", "tree", "dirty", "branch",
        "acceptance_inventory_sha256", "deployment_inputs_sha256"
    )
    foreach ($candidate in @($Receipt.mutable_source_identity, $ExpectedSource)) {
        $actualProperties = @($candidate.PSObject.Properties.Name | Sort-Object)
        if (($actualProperties | ConvertTo-Json -Compress) -cne
            (@($expectedSourceProperties | Sort-Object) | ConvertTo-Json -Compress) -or
            $candidate.commit -isnot [string] -or [string]$candidate.commit -notmatch '^[0-9a-f]{40}$' -or
            $candidate.tree -isnot [string] -or [string]$candidate.tree -notmatch '^[0-9a-f]{40}$' -or
            $candidate.dirty -isnot [bool] -or $candidate.dirty -ne $false -or
            $candidate.branch -isnot [string] -or [string]::IsNullOrWhiteSpace([string]$candidate.branch) -or
            $candidate.acceptance_inventory_sha256 -isnot [string] -or [string]$candidate.acceptance_inventory_sha256 -notmatch '^[0-9a-f]{64}$' -or
            $candidate.deployment_inputs_sha256 -isnot [string] -or [string]$candidate.deployment_inputs_sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "Lane '$ExpectedName' carries a malformed or dirty mutable source identity."
        }
    }
    if (($Receipt.mutable_source_identity | ConvertTo-Json -Depth 10 -Compress) -cne
        ($ExpectedSource | ConvertTo-Json -Depth 10 -Compress)) {
        throw "Lane '$ExpectedName' is not bound to the current clean source identity."
    }
    $state = [string]$Receipt.result.state
    $ended = [DateTimeOffset]::Parse([string]$Receipt.result.ended_at)
    if ($state -ceq "blocked") {
        if ($null -ne $Receipt.result.exit_status -or
            -not [string]::IsNullOrWhiteSpace([string]$Receipt.result.started_at) -or
            [string]::IsNullOrWhiteSpace([string]$Receipt.result.dependency_reason)) {
            throw "Blocked lane '$ExpectedName' does not retain its exact dependency reason."
        }
    } elseif ($state -in @("passed", "failed")) {
        $started = [DateTimeOffset]::Parse([string]$Receipt.result.started_at)
        if ($ended -lt $started -or
            ($state -ceq "passed" -and [int]$Receipt.result.exit_status -ne 0) -or
            ($state -ceq "failed" -and [int]$Receipt.result.exit_status -eq 0)) {
            throw "Executed lane '$ExpectedName' lacks valid chronology or status."
        }
    } else {
        throw "Lane '$ExpectedName' has unsupported terminal state '$state'."
    }
    [pscustomobject][ordered]@{
        state = $state
        ended_at = $ended.ToString("o")
        dependency_reason = [string]$Receipt.result.dependency_reason
    }
}

function Assert-LaneReceiptFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ExpectedName,
        [Parameter(Mandatory)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$ExpectedEnvironment,
        [Parameter(Mandatory)]$ExpectedSource
    )

    $fullPath = if ([IO.Path]::IsPathRooted($Path)) { [IO.Path]::GetFullPath($Path) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot $Path)) }
    $sha256 = Assert-Sprint8AReceiptSidecar -Path $fullPath
    $receipt = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json
    $terminal = Assert-LaneReceiptObject -Receipt $receipt -ExpectedName $ExpectedName -ExpectedAttempt $ExpectedAttempt -ExpectedEnvironment $ExpectedEnvironment -ExpectedSource $ExpectedSource
    if ([string]$receipt.result.state -in @("passed", "failed")) {
        $rawPath = if ([IO.Path]::IsPathRooted([string]$receipt.result.evidence_path)) { [IO.Path]::GetFullPath([string]$receipt.result.evidence_path) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$receipt.result.evidence_path))) }
        if ((Get-Sprint8AFileSha256 -Path $rawPath) -cne [string]$receipt.result.evidence_sha256) {
            throw "Lane '$ExpectedName' raw-evidence digest does not match '$rawPath'."
        }
    }
    foreach ($evidence in @($receipt.result.produced_evidence)) {
        $evidencePath = if ([IO.Path]::IsPathRooted([string]$evidence.path)) { [IO.Path]::GetFullPath([string]$evidence.path) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$evidence.path))) }
        if ((Get-Sprint8AFileSha256 -Path $evidencePath) -cne [string]$evidence.sha256) {
            throw "Lane '$ExpectedName' produced-evidence digest does not match '$evidencePath'."
        }
    }
    [pscustomobject][ordered]@{
        name = $ExpectedName
        state = [string]$terminal.state
        path = [IO.Path]::GetRelativePath($repoRoot, $fullPath).Replace("\", "/")
        sha256 = $sha256
        ended_at = [string]$terminal.ended_at
        dependency_reason = [string]$terminal.dependency_reason
    }
}

function Assert-ProductDiagnosticReceiptObject {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$ExpectedEnvironment,
        [Parameter(Mandatory)]$ExpectedSource
    )

    if (($Receipt.schema_version -isnot [int] -and $Receipt.schema_version -isnot [long]) -or
        [long]$Receipt.schema_version -ne 2 -or
        [string]$Receipt.sprint -cne "sprint-8a" -or
        [string]$Receipt.phase -cne "candidate-product-diagnostic" -or
        $Receipt.authoritative -isnot [bool] -or $Receipt.authoritative -ne $false -or
        [string]$Receipt.state -cne "passed" -or
        [int]$Receipt.attempt -ne $ExpectedAttempt -or
        [string]$Receipt.environment_fingerprint -cne $ExpectedEnvironment -or
        ($Receipt.mutable_source_identity | ConvertTo-Json -Depth 10 -Compress) -cne
        ($ExpectedSource | ConvertTo-Json -Depth 10 -Compress) -or
        $Receipt.acceptance_authority.formal_uat -isnot [bool] -or
        $Receipt.acceptance_authority.formal_uat -ne $false -or
        $Receipt.acceptance_authority.acceptance_evidence_published -isnot [bool] -or
        $Receipt.acceptance_authority.acceptance_evidence_published -ne $false -or
        [string]$Receipt.acceptance_authority.mode -cne "non_acceptance_development_diagnostic" -or
        (@($Receipt.diagnostic_checks | ForEach-Object { "$([string]$_.name)|$([string]$_.state)" }) -join ",") -cne
        "broad-product-behavior|passed,dashboard-dependency-semantics|passed") {
        throw "Live product diagnostic evidence is not the exact non-authoritative receipt for this source, attempt, and environment."
    }
    Assert-Sprint8ADashboardDependencyEvidence -Evidence $Receipt.dashboard_dependency_semantics
}

function Assert-ProductDiagnosticReceiptFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$ExpectedEnvironment,
        [Parameter(Mandatory)]$ExpectedSource
    )

    $fullPath = if ([IO.Path]::IsPathRooted($Path)) { [IO.Path]::GetFullPath($Path) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot $Path)) }
    $sha256 = Assert-Sprint8AReceiptSidecar -Path $fullPath
    $receipt = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json
    Assert-ProductDiagnosticReceiptObject -Receipt $receipt -ExpectedAttempt $ExpectedAttempt -ExpectedEnvironment $ExpectedEnvironment -ExpectedSource $ExpectedSource
    [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $fullPath).Replace("\", "/")
        sha256 = $sha256
    }
}

Assert-DiagnosticInventory
if ($SelfTest) {
    Assert-RepositoryDiagnosticContract
    function Invoke-ExpectedLaneGuardFailure {
        param([Parameter(Mandatory)][scriptblock]$Action, [Parameter(Mandatory)][string]$Label)
        try {
            & $Action
            throw "Self-test accepted $Label."
        } catch {
            if ($_.Exception.Message -ceq "Self-test accepted $Label.") { throw }
        }
    }

    $source = [pscustomobject]@{ commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"; acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64 }
    $lane = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = 4
        authoritative = $false; environment_fingerprint = "e" * 64; mutable_source_identity = $source
        result = [pscustomobject]@{
            name = "successor-product-smoke"; state = "passed"; exit_status = 0
            started_at = "2026-01-01T00:00:00Z"; ended_at = "2026-01-01T00:00:01Z"
            dependency_reason = $null
            produced_evidence = @([pscustomobject]@{ path = "fixture"; sha256 = "f" * 64 })
        }
    }
    $validated = Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source
    if ([string]$validated.state -cne "passed") { throw "Self-test did not accept the current passing lane." }
    $lane.authoritative = $true
    Invoke-ExpectedLaneGuardFailure { Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source } "an authoritative prerequisite lane"
    $lane.authoritative = $false
    $lane.authoritative = 0
    Invoke-ExpectedLaneGuardFailure { Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source } "a numerically coerced prerequisite authority flag"
    $lane.authoritative = $false
    $lane.schema_version = "1"
    Invoke-ExpectedLaneGuardFailure { Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source } "a string-coerced prerequisite schema"
    $lane.schema_version = 1
    $lane.phase = "candidate-rehearsal-uat-diagnostics"
    Invoke-ExpectedLaneGuardFailure { Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source } "a prerequisite lane with the wrong phase"
    $lane.phase = "candidate-rehearsal-lane"
    $lane.environment_fingerprint = "E" * 64
    Invoke-ExpectedLaneGuardFailure { Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source } "a prerequisite lane with a malformed environment identity"
    $lane.environment_fingerprint = "e" * 64
    $lane.result.state = "failed"
    $lane.result.exit_status = 1
    $staleSource = [pscustomobject]@{ commit = "f" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"; acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64 }
    Invoke-ExpectedLaneGuardFailure { Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $staleSource } "a stale prerequisite lane"
    $validated = Assert-LaneReceiptObject -Receipt $lane -ExpectedName "successor-product-smoke" -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source
    if ([string]$validated.state -cne "failed") { throw "Self-test did not retain a current failed lane as terminal evidence." }
    if ($scenarioDependencies["UAT-8A-07"] -cnotcontains "failure-containment-successor-health") {
        throw "Self-test found UAT-8A-07 detached from failure-containment successor health."
    }
    if ($scenarioDependencies["UAT-8A-05"] -cnotcontains "live-product-diagnostics") {
        throw "Self-test found UAT-8A-05 detached from executable Dashboard dependency semantics."
    }
    $semanticFixture = [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-8a.dashboard-dependency-semantic-diagnostic"
        checks = @($script:Sprint8ADashboardDependencyCheckCodes | ForEach-Object { [pscustomobject]@{ code = $_; passed = $true } })
        actions = @(@("defer", "upgrade", "replace", "remove") | ForEach-Object { [pscustomobject]@{ action = $_ } })
        final_health = [pscustomobject]@{ health = "healthy"; open_count = 0; deferred_count = 0 }
        canonical_reset_required = $true
        passed = $true
    }
    $productReceipt = [pscustomobject][ordered]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-product-diagnostic"
        attempt = 4; authoritative = $false; state = "passed"
        mutable_source_identity = $source; environment_fingerprint = "e" * 64
        acceptance_authority = [pscustomobject]@{ formal_uat = $false; acceptance_evidence_published = $false; mode = "non_acceptance_development_diagnostic" }
        diagnostic_checks = @(
            [pscustomobject]@{ name = "broad-product-behavior"; state = "passed" },
            [pscustomobject]@{ name = "dashboard-dependency-semantics"; state = "passed" }
        )
        dashboard_dependency_semantics = $semanticFixture
    }
    Assert-ProductDiagnosticReceiptObject -Receipt $productReceipt -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source
    $semanticFixture.passed = $false
    Invoke-ExpectedLaneGuardFailure { Assert-ProductDiagnosticReceiptObject -Receipt $productReceipt -ExpectedAttempt 4 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source } "a label-only product diagnostic without passing Dashboard dependency semantics"
    Write-Host "Sprint 8A automated UAT diagnostic identity/freshness self-test passed."
    return
}

if ($Attempt -lt 1 -or [string]::IsNullOrWhiteSpace($EnvironmentFingerprint)) {
    throw "Sprint 8A UAT diagnostics require the exact rehearsal attempt and environment fingerprint."
}
if ([string]::IsNullOrWhiteSpace($ProductDiagnosticEvidence)) {
    throw "Sprint 8A UAT diagnostics require the exact live product diagnostic evidence document."
}
$paths = [ordered]@{
    "source-exact-materialization-no-op" = $MaterializationLaneReceipt
    "successor-inventory-navigation-audit" = $InventoryLaneReceipt
    "successor-deployment-evidence" = $DeploymentEvidenceLaneReceipt
    "successor-product-smoke" = $ProductSmokeLaneReceipt
    "failure-containment-successor-health" = $FailureContainmentLaneReceipt
    "component-upgrade-rollback" = $UpgradeLaneReceipt
    "component-conformance-nondisclosure" = $ComponentConformanceLaneReceipt
    "playwright-execution" = $PlaywrightLaneReceipt
    "compose-manifest-schema-contract" = $ManifestContractLaneReceipt
    "web-native-wasm-source-boundaries" = $WebBoundaryLaneReceipt
    "dashboard-source-boundaries" = $DashboardBoundaryLaneReceipt
    "live-product-diagnostics" = $ProductDiagnosticLaneReceipt
}
foreach ($entry in $paths.GetEnumerator()) {
    if ([string]::IsNullOrWhiteSpace([string]$entry.Value)) { throw "UAT diagnostics require the '$($entry.Key)' lane receipt." }
}
$outputFullPath = if ([IO.Path]::IsPathRooted($OutputPath)) { [IO.Path]::GetFullPath($OutputPath) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot $OutputPath)) }
if ((Test-Path -LiteralPath $outputFullPath) -or (Test-Path -LiteralPath "$outputFullPath.sha256")) {
    throw "Attempt-scoped UAT diagnostic evidence already exists and cannot be overwritten: $outputFullPath"
}
$source = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
if ($source.dirty) { throw "Sprint 8A UAT diagnostics require clean source." }
$startedAt = [DateTimeOffset]::UtcNow
$prerequisites = [Collections.Generic.List[object]]::new()
$prerequisiteByName = @{}
$failures = [Collections.Generic.List[object]]::new()
foreach ($entry in $paths.GetEnumerator()) {
    try {
        $validated = Assert-LaneReceiptFile -Path ([string]$entry.Value) -ExpectedName ([string]$entry.Key) -ExpectedAttempt $Attempt -ExpectedEnvironment $EnvironmentFingerprint -ExpectedSource $source
        $prerequisites.Add($validated)
        $prerequisiteByName[[string]$entry.Key] = $validated
    } catch {
        $failed = [pscustomobject][ordered]@{ name = [string]$entry.Key; state = "failed"; reason = $_.Exception.Message }
        $prerequisites.Add($failed)
        $prerequisiteByName[[string]$entry.Key] = $failed
        $failures.Add($failed)
    }
}
$productSemanticEvidence = $null
if ([string]$prerequisiteByName["live-product-diagnostics"].state -ceq "passed") {
    try {
        $productSemanticEvidence = Assert-ProductDiagnosticReceiptFile `
            -Path $ProductDiagnosticEvidence `
            -ExpectedAttempt $Attempt `
            -ExpectedEnvironment $EnvironmentFingerprint `
            -ExpectedSource $source
    } catch {
        $failed = [pscustomobject][ordered]@{ name = "live-product-diagnostics-semantic-evidence"; state = "failed"; reason = $_.Exception.Message }
        $prerequisites.Add($failed)
        $prerequisiteByName["live-product-diagnostics"] = $failed
        $failures.Add($failed)
    }
}
if ($prerequisiteByName["failure-containment-successor-health"].state -ceq "passed" -and
    $prerequisiteByName["source-exact-materialization-no-op"].state -ceq "passed" -and
    [DateTimeOffset]::Parse([string]$prerequisiteByName["failure-containment-successor-health"].ended_at) -lt
    [DateTimeOffset]::Parse([string]$prerequisiteByName["source-exact-materialization-no-op"].ended_at)) {
    $failures.Add([pscustomobject]@{ name = "failure-containment-chronology"; state = "failed"; reason = "Failure containment predates the materialization it supersedes." })
}

$checks = @($scenarioDependencies.GetEnumerator() | ForEach-Object {
    $missing = @($_.Value | Where-Object { $prerequisiteByName[[string]$_].state -cne "passed" })
    if ($missing.Count -eq 0) {
        [ordered]@{ scenario = $_.Key; state = "passed"; diagnostic_dependencies = @($_.Value); semantic_assertions = @($scenarioAssertions[[string]$_.Key]); dependency_reason = $null }
    } else {
        [ordered]@{ scenario = $_.Key; state = "blocked"; diagnostic_dependencies = @($_.Value); semantic_assertions = @($scenarioAssertions[[string]$_.Key]); dependency_reason = "blocked by invalid prerequisite(s): $($missing -join ', ')" }
    }
})
$blocked = @($checks | Where-Object state -CEQ "blocked")
$successorHealthy = [string]$prerequisiteByName["failure-containment-successor-health"].state -ceq "passed" -and
    [string]$prerequisiteByName["successor-deployment-evidence"].state -ceq "passed" -and
    [string]$prerequisiteByName["successor-product-smoke"].state -ceq "passed"
$result = [ordered]@{
    schema_version = 2
    sprint = "sprint-8a"
    phase = "candidate-rehearsal-uat-diagnostics"
    attempt = $Attempt
    authoritative = $false
    state = if ($failures.Count -gt 0) { "failed" } elseif ($blocked.Count -gt 0) { "blocked" } else { "passed" }
    started_at = $startedAt.ToString("o")
    ended_at = [DateTimeOffset]::UtcNow.ToString("o")
    mutable_source_identity = $source
    environment_fingerprint = $EnvironmentFingerprint
    prerequisite_receipts = $prerequisites
    product_semantic_evidence = $productSemanticEvidence
    checks = $checks
    failure_count = $failures.Count
    blocked_count = $blocked.Count
    cleanup_restoration = [ordered]@{
        required = $true
        result = if ($successorHealthy) { "canonical_successor_healthy" } else { "not_proven" }
    }
}
Publish-Sprint7AEvidence -Document $result -OutputPath $outputFullPath | Out-Null
if ([string]$result.state -cne "passed") {
    if ([string]$result.state -ceq "failed") {
        throw "Sprint 8A automated UAT diagnostics found $($failures.Count) invalid prerequisite receipts; evidence is retained at $outputFullPath."
    }
    Write-Host "Sprint 8A automated UAT diagnostics retained $($blocked.Count) dependency-blocked scenarios without creating derivative defects."
    return
}
Write-Host "All eight Sprint 8A UAT diagnostic equivalents passed. Formal UAT was not performed."

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

# Every automated UAT diagnostic assertion is an executable predicate over
# source-bound rehearsal evidence. A passing producer lane is necessary but is
# never sufficient by itself. Keep titles exact so renamed or removed browser
# coverage invalidates this projection instead of silently inheriting a count.
$semanticPredicateRegistry = [ordered]@{
    "component-module-live-script" = [ordered]@{
        producers = @("successor-product-smoke", "playwright-execution")
        evaluator = "smoke_and_playwright"
        smoke_checks = @("component_document", "component_kind_inventory")
        playwright_titles = @(
            "Sprint 8A extracted Component UI parity › admin can create, update, publish, and view a major-line table component",
            "Sprint 8A extracted Component UI parity › admin can author, publish, and view visual components"
        )
    }
    "complete-browser-inventory" = [ordered]@{
        producers = @("playwright-execution")
        evaluator = "playwright"
        playwright_titles = @(
            "Sprint 8A extracted Component UI parity › admin can create, update, publish, and view a major-line table component",
            "Sprint 8A extracted Component UI parity › admin can author, publish, and view visual components",
            "Sprint 8A extracted Component UI parity › exact viewport and theme matrix preserves directory editor detail and viewer usability",
            "Sprint 8A extracted Component UI parity › Component and Dashboard visuals stay contained at 200% zoom in light and dark themes",
            "capability + scope + ownership permissions › JavaScript-disabled Component and Dashboard routes preserve native SSR ownership"
        )
    }
    "module-owned-documents-and-assets" = [ordered]@{
        producers = @("successor-product-smoke")
        evaluator = "smoke"
        smoke_checks = @("component_document")
    }
    "empty-first-apply" = [ordered]@{
        producers = @("source-exact-materialization-no-op")
        evaluator = "materialization"
        materialization_facts = @("empty_baseline", "first_apply")
    }
    "semantic-no-op" = [ordered]@{
        producers = @("source-exact-materialization-no-op")
        evaluator = "materialization"
        materialization_facts = @("no_op_apply", "final_health", "public_gateway_after_owner_apply")
    }
    "exact-five-core-transitions" = [ordered]@{
        producers = @("successor-inventory-navigation-audit")
        evaluator = "inventory"
        inventory_facts = @("exact_five_transitions")
    }
    "receipt-bound-dashboard-references" = [ordered]@{
        producers = @("successor-product-smoke")
        evaluator = "smoke"
        smoke_checks = @(
            "dashboard_placement_inventory",
            "dashboard_reference_01980000-0003-7000-8000-000000000002",
            "dashboard_reference_01980000-0003-7000-8000-000000000003",
            "dashboard_reference_01980000-0003-7000-8000-000000000004",
            "dashboard_reference_01980000-0003-7000-8000-000000000005",
            "dashboard_reference_01980000-0003-7000-8000-000000000006",
            "dashboard_reference_01980000-0003-7000-8000-000000000007",
            "dashboard_reference_01980000-0003-7000-8000-000000000008"
        )
    }
    "configuration-schema-authority" = [ordered]@{
        producers = @("playwright-execution")
        evaluator = "playwright"
        playwright_titles = @("Sprint 8A Module Management › Components configuration enforces exact schema authority projection and sanitized diagnostics")
    }
    "label-navigation-projection" = [ordered]@{
        producers = @("playwright-execution")
        evaluator = "playwright"
        playwright_titles = @("Sprint 8A Module Management › Components configuration enforces exact schema authority projection and sanitized diagnostics")
    }
    "sanitized-diagnostics" = [ordered]@{
        producers = @("playwright-execution")
        evaluator = "playwright"
        playwright_titles = @("Sprint 8A Module Management › Components configuration enforces exact schema authority projection and sanitized diagnostics")
    }
    "dataset-contract-execution" = [ordered]@{
        producers = @("successor-product-smoke", "playwright-execution")
        evaluator = "smoke_and_playwright"
        smoke_checks = @("component_execution_contract")
        playwright_titles = @("Sprint 8A extracted Component UI parity › admin can create, update, publish, and view a major-line table component")
    }
    "known-random-nondisclosure" = [ordered]@{
        producers = @("playwright-execution")
        evaluator = "playwright"
        playwright_titles = @("Sprint 7A scoped analytics boundary › mixed-scope Component lookups make known blocked and random identities indistinguishable")
    }
    "timeout-outage-recovery" = [ordered]@{
        producers = @("playwright-execution")
        evaluator = "playwright"
        playwright_titles = @("Sprint 8A extracted Component UI parity › Dataset provider outage retains unsaved editor state and one retry mutation")
    }
    "dashboard-lifecycle-findings" = [ordered]@{
        producers = @("live-product-diagnostics")
        evaluator = "dashboard_dependency"
        dashboard_checks = @("exact_lifecycle_finding_placements")
    }
    "consumer-actions" = [ordered]@{
        producers = @("live-product-diagnostics")
        evaluator = "dashboard_dependency"
        dashboard_checks = @(
            "defer_advances_finding", "defer_preserves_composition",
            "upgrade_uses_declared_successor", "replace_uses_authorized_renderable_reference",
            "remove_consumes_independent_placement", "action_fixtures_remain_independent"
        )
    }
    "provider-outage-containment" = [ordered]@{
        producers = @("live-product-diagnostics")
        evaluator = "dashboard_dependency"
        dashboard_checks = @(
            "provider_outage_is_contained", "outage_blocked_scope_nondisclosure",
            "unrelated_route_remains_healthy", "provider_recovery_converges"
        )
    }
    "core-component-absence" = [ordered]@{
        producers = @("compose-manifest-schema-contract")
        evaluator = "source_contract"
        source_paths = @("scripts/sprint-8a-acceptance-contract.ps1", "deploy/sprint-8a/compose.yaml")
        source_fragments = @(
            "Core retains the consumer-named Component Dataset adapter.",
            "Core Dataset provider retains a Component-specific route or identity branch.",
            "tessara-component-module"
        )
    }
    "native-wasm-source-boundaries" = [ordered]@{
        producers = @("web-native-wasm-source-boundaries", "dashboard-source-boundaries")
        evaluator = "source_contract"
        source_paths = @("scripts/check-web-crate-boundaries.ps1", "scripts/verify-sprint-6e-boundaries.ps1")
        source_fragments = @(
            "The extracted Component module must not depend on Core/root",
            "Core/root web still consumes Dashboard UI source"
        )
    }
    "old-input-rejection" = [ordered]@{
        producers = @("successor-product-smoke")
        evaluator = "smoke"
        smoke_checks = @("old_core_payload_rejected")
    }
    "exact-real-module-inventory" = [ordered]@{
        producers = @("successor-inventory-navigation-audit")
        evaluator = "inventory"
        inventory_facts = @("exact_real_modules", "exact_navigation")
    }
    "induced-owner-failure" = [ordered]@{
        producers = @("failure-containment-successor-health")
        evaluator = "failure_containment"
        containment_facts = @("expected_fault")
    }
    "exact-teardown" = [ordered]@{
        producers = @("failure-containment-successor-health")
        evaluator = "failure_containment"
        containment_facts = @("exact_teardown")
    }
    "empty-successor" = [ordered]@{
        producers = @("failure-containment-successor-health")
        evaluator = "failure_containment"
        containment_facts = @("empty_successor", "first_apply")
    }
    "successor-no-op-health" = [ordered]@{
        producers = @("failure-containment-successor-health")
        evaluator = "failure_containment"
        containment_facts = @("no_op", "health")
    }
    "component-only-upgrade" = [ordered]@{
        producers = @("component-upgrade-rollback")
        evaluator = "upgrade"
        upgrade_facts = @("candidate_upgrade")
    }
    "rollback" = [ordered]@{
        producers = @("component-upgrade-rollback")
        evaluator = "upgrade"
        upgrade_facts = @("baseline_rollback")
    }
    "unrelated-identity-stability" = [ordered]@{
        producers = @("component-upgrade-rollback")
        evaluator = "upgrade"
        upgrade_facts = @("exact_preservation")
    }
    "intended-release-restoration" = [ordered]@{
        producers = @("component-upgrade-rollback")
        evaluator = "upgrade"
        upgrade_facts = @("candidate_restored")
    }
}

function Assert-DiagnosticInventory {
    if ($scenarioDependencies.Count -ne 8 -or $scenarioAssertions.Count -ne 8 -or $requirementMappings.Count -ne 8) {
        throw "Sprint 8A must define exactly eight UAT diagnostic scenarios and requirement mappings."
    }
    $declaredAssertions = @($scenarioAssertions.Values | ForEach-Object { @($_) })
    $registeredAssertions = @($semanticPredicateRegistry.Keys)
    if ($declaredAssertions.Count -ne @($declaredAssertions | Sort-Object -Unique).Count -or
        (($declaredAssertions | Sort-Object) -join "`n") -cne (($registeredAssertions | Sort-Object) -join "`n")) {
        throw "Every Sprint 8A UAT assertion must have exactly one executable semantic predicate registration."
    }
    foreach ($number in 1..8) {
        $id = "UAT-8A-{0:d2}" -f $number
        if (-not $scenarioDependencies.Contains($id) -or -not $scenarioAssertions.Contains($id) -or -not $requirementMappings.Contains($id)) {
            throw "Missing automated diagnostic mapping '$id'."
        }
        foreach ($assertion in @($scenarioAssertions[$id])) {
            $predicate = $semanticPredicateRegistry[[string]$assertion]
            if ($null -eq $predicate -or
                [string]::IsNullOrWhiteSpace([string]$predicate.evaluator) -or
                @($predicate.producers).Count -lt 1) {
                throw "UAT assertion '$assertion' lacks an executable evaluator or producer."
            }
            $undeclaredProducers = @($predicate.producers | Where-Object { $scenarioDependencies[$id] -cnotcontains [string]$_ })
            if ($undeclaredProducers.Count -gt 0) {
                throw "UAT assertion '$assertion' uses producer(s) not declared by ${id}: $($undeclaredProducers -join ', ')."
            }
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
    $rawEvidence = $null
    if ([string]$receipt.result.state -in @("passed", "failed")) {
        $rawPath = if ([IO.Path]::IsPathRooted([string]$receipt.result.evidence_path)) { [IO.Path]::GetFullPath([string]$receipt.result.evidence_path) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$receipt.result.evidence_path))) }
        $rawSha256 = Get-Sprint8AFileSha256 -Path $rawPath
        if ($rawSha256 -cne [string]$receipt.result.evidence_sha256) {
            throw "Lane '$ExpectedName' raw-evidence digest does not match '$rawPath'."
        }
        $rawEvidence = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $rawPath).Replace("\", "/")
            sha256 = $rawSha256
        }
    }
    $producedEvidence = [Collections.Generic.List[object]]::new()
    foreach ($evidence in @($receipt.result.produced_evidence)) {
        $evidencePath = if ([IO.Path]::IsPathRooted([string]$evidence.path)) { [IO.Path]::GetFullPath([string]$evidence.path) } else { [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$evidence.path))) }
        $evidenceSha256 = Get-Sprint8AFileSha256 -Path $evidencePath
        if ($evidenceSha256 -cne [string]$evidence.sha256) {
            throw "Lane '$ExpectedName' produced-evidence digest does not match '$evidencePath'."
        }
        $producedEvidence.Add([pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $evidencePath).Replace("\", "/")
            sha256 = $evidenceSha256
        })
    }
    [pscustomobject][ordered]@{
        name = $ExpectedName
        state = [string]$terminal.state
        path = [IO.Path]::GetRelativePath($repoRoot, $fullPath).Replace("\", "/")
        sha256 = $sha256
        command = [string]$receipt.result.command
        ended_at = [string]$terminal.ended_at
        dependency_reason = [string]$terminal.dependency_reason
        raw_evidence = $rawEvidence
        produced_evidence = @($producedEvidence)
    }
}

function Resolve-UatEvidencePath {
    param([Parameter(Mandatory)][string]$Path)
    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Get-UatLaneEvidence {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string]$FileName
    )
    $matches = @($Lane.produced_evidence | Where-Object {
        [IO.Path]::GetFileName([string]$_.path) -ceq $FileName
    })
    if ($matches.Count -ne 1) {
        throw "Lane '$($Lane.name)' must publish exactly one authenticated '$FileName' evidence file."
    }
    $matches[0]
}

function Read-UatLaneJsonEvidence {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string]$FileName
    )
    $evidence = Get-UatLaneEvidence -Lane $Lane -FileName $FileName
    $fullPath = Resolve-UatEvidencePath -Path ([string]$evidence.path)
    [pscustomobject][ordered]@{
        document = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json
        evidence = $evidence
    }
}

function Read-UatReferencedJsonEvidence {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)]$Artifact,
        [Parameter(Mandatory)][string]$Label
    )
    $fullPath = Resolve-UatEvidencePath -Path ([string]$Artifact.path)
    $actualSha256 = Get-Sprint8AFileSha256 -Path $fullPath
    $published = @($Lane.produced_evidence | Where-Object {
        [string]::Equals(
            (Resolve-UatEvidencePath -Path ([string]$_.path)),
            $fullPath,
            [StringComparison]::OrdinalIgnoreCase
        ) -and
        [string]$_.sha256 -ceq $actualSha256
    })
    if ($actualSha256 -cne [string]$Artifact.sha256 -or $published.Count -ne 1) {
        throw "$Label is not an authenticated file in lane '$($Lane.name)'."
    }
    [pscustomobject][ordered]@{
        document = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json
        evidence = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $fullPath).Replace("\", "/")
            sha256 = $actualSha256
        }
    }
}

function Add-UatPlaywrightSuiteResults {
    param(
        [Parameter(Mandatory)]$Suite,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ParentTitles,
        [Parameter(Mandatory)][AllowEmptyString()][string]$InheritedFile,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Results
    )
    $file = if ($null -ne $Suite.PSObject.Properties['file'] -and
        -not [string]::IsNullOrWhiteSpace([string]$Suite.file)) {
        [string]$Suite.file
    } else {
        $InheritedFile
    }
    $titles = @($ParentTitles)
    if ($null -ne $Suite.PSObject.Properties['title'] -and
        -not [string]::IsNullOrWhiteSpace([string]$Suite.title) -and
        [string]$Suite.title -cne $file) {
        $titles += [string]$Suite.title
    }
    foreach ($spec in @($Suite.specs)) {
        if ($null -eq $spec) { continue }
        $fullTitle = (@($titles) + @([string]$spec.title)) -join " › "
        foreach ($test in @($spec.tests)) {
            if ($null -eq $test) { continue }
            $testResults = @($test.results)
            $passed = [string]$test.expectedStatus -ceq "passed" -and
                $testResults.Count -eq 1 -and
                [string]$testResults[0].status -ceq "passed" -and
                [int]$testResults[0].retry -eq 0
            $Results.Add([pscustomobject][ordered]@{
                file = $file
                title = $fullTitle
                project = [string]$test.projectName
                passed = $passed
            })
        }
    }
    foreach ($child in @($Suite.suites)) {
        if ($null -ne $child) {
            Add-UatPlaywrightSuiteResults -Suite $child -ParentTitles $titles -InheritedFile $file -Results $Results
        }
    }
}

function Get-UatPlaywrightResults {
    param([Parameter(Mandatory)]$Report)
    $results = [Collections.Generic.List[object]]::new()
    foreach ($suite in @($Report.suites)) {
        Add-UatPlaywrightSuiteResults -Suite $suite -ParentTitles @() -InheritedFile "" -Results $results
    }
    @($results)
}

function Assert-UatSmokePredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$ExpectedChecks
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "smoke-successor.json"
    if ([string]$loaded.document.evidence_kind -cne "tessara.sprint-8a.smoke" -or
        $loaded.document.passed -isnot [bool] -or $loaded.document.passed -ne $true) {
        throw "Sprint 8A successor smoke evidence is not a passing semantic document."
    }
    foreach ($code in $ExpectedChecks) {
        $matches = @($loaded.document.checks | Where-Object { [string]$_.code -ceq $code })
        if ($matches.Count -ne 1 -or $matches[0].passed -isnot [bool] -or $matches[0].passed -ne $true) {
            throw "Sprint 8A smoke evidence does not contain one passing '$code' check."
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($loaded.evidence)
        observed = [ordered]@{ passing_check_codes = @($ExpectedChecks) }
    }
}

function Assert-UatPlaywrightPredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$ExpectedTitles
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "playwright-acceptance.json"
    $results = @(Get-UatPlaywrightResults -Report $loaded.document)
    foreach ($title in $ExpectedTitles) {
        $matches = @($results | Where-Object { [string]$_.title -ceq $title })
        if ($matches.Count -ne 1 -or $matches[0].passed -ne $true) {
            throw "Playwright evidence does not contain one non-retried passing exact test '$title'."
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($loaded.evidence)
        observed = [ordered]@{ passing_exact_test_titles = @($ExpectedTitles) }
    }
}

function Assert-UatMaterializationPredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$Facts
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "materialization-result.json"
    $document = $loaded.document
    if ([string]$document.contract -cne "tessara.sprint-8a.materialization-result" -or
        $document.passed -isnot [bool] -or $document.passed -ne $true) {
        throw "Sprint 8A materialization evidence is not a passing result."
    }
    $evidence = [Collections.Generic.List[object]]::new()
    $evidence.Add($loaded.evidence)
    foreach ($fact in $Facts) {
        switch ($fact) {
            "empty_baseline" {
                $baseline = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.evidence.empty_baseline -Label "empty baseline"
                if ($baseline.document.empty -ne $true -or $baseline.document.teardown.passed -ne $true -or
                    $baseline.document.teardown.after.empty -ne $true -or
                    @($baseline.document.teardown.after.containers).Count -ne 0 -or
                    @($baseline.document.teardown.after.present_volumes).Count -ne 0 -or
                    @($baseline.document.teardown.after.present_networks).Count -ne 0) {
                    throw "Materialization did not begin from an exact empty topology."
                }
                $evidence.Add($baseline.evidence)
            }
            "first_apply" {
                if ($document.first_apply.no_op -ne $false -or [string]$document.first_apply.operation_state -cne "succeeded" -or
                    @($document.first_apply.owner_receipts).Count -ne 2) {
                    throw "Materialization first apply is not one successful owner-produced non-no-op."
                }
            }
            "no_op_apply" {
                if ($document.no_op_apply.no_op -ne $true -or [string]$document.no_op_apply.operation_state -cne "succeeded" -or
                    [string]$document.no_op_apply.previous_receipt_digest -cne [string]$document.first_apply.receipt_digest) {
                    throw "Materialization successor apply is not the exact chained semantic no-op."
                }
            }
            "final_health" {
                if ($document.final_health_passed -ne $true) { throw "Materialization final health is not passing." }
            }
            "public_gateway_after_owner_apply" {
                $boundary = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.evidence.public_gateway_boundary -Label "public gateway boundary"
                if ($boundary.document.passed -ne $true -or
                    @($boundary.document.gateway_service_before | Where-Object { [string]$_.state -ceq "running" }).Count -ne 0 -or
                    $boundary.document.public_probe_before.available -ne $false -or
                    [int]$boundary.document.public_probe_before.status -ne 0 -or
                    [int]$boundary.document.start.exit_code -ne 0 -or
                    [DateTimeOffset]::Parse([string]$boundary.document.public_ready_at) -lt
                        [DateTimeOffset]::Parse([string]$boundary.document.owner_materialization_completed_at)) {
                    throw "Public gateway evidence does not prove the offline owner-materialization boundary."
                }
                $evidence.Add($boundary.evidence)
            }
            default { throw "Unknown materialization predicate fact '$fact'." }
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($evidence)
        observed = [ordered]@{ proven_facts = @($Facts) }
    }
}

function Assert-UatInventoryPredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$Facts
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "deployed-inventory-successor.json"
    $document = $loaded.document
    if ([string]$document.evidence_kind -cne "tessara.sprint-8a.deployed-inventory-navigation" -or
        $document.passed -ne $true) {
        throw "Sprint 8A inventory evidence is not a passing exact-identity document."
    }
    foreach ($fact in $Facts) {
        switch ($fact) {
            "exact_five_transitions" {
                $expected = @("tessara.datasets", "tessara.forms", "tessara.migration", "tessara.responses", "tessara.workflows")
                if ((@($document.transition_identities | Sort-Object) -join "`n") -cne ($expected -join "`n")) {
                    throw "Deployed inventory does not contain the exact five Core transitions."
                }
            }
            "exact_real_modules" {
                $actual = @($document.module_inventory | ForEach-Object { "$([string]$_.definition_id)|$([string]$_.release_version)|$([string]$_.instance_id)" } | Sort-Object)
                $expected = @(
                    "tessara.components|1.0.0|142a1ece-f74b-85f6-8ca0-92f4a02e9409",
                    "tessara.dashboards|3.0.0|a6339e9f-1131-870e-aac6-18a8a01e4bbd"
                ) | Sort-Object
                if (($actual -join "`n") -cne ($expected -join "`n")) {
                    throw "Components and Dashboard do not appear exactly once through their real release/instance identities."
                }
            }
            "exact_navigation" {
                $expected = @(
                    "Home", "Organization", "Forms", "Workflows", "Responses", "Operations",
                    "Datasets", "Scoped Records", "Components", "Dashboards", "User Management",
                    "Roles & Access", "Node Types", "Module Management", "Application Composition"
                )
                if ((@($document.navigation_order) -join "`n") -cne ($expected -join "`n")) {
                    throw "Deployed navigation does not match the exact accepted order."
                }
            }
            default { throw "Unknown inventory predicate fact '$fact'." }
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($loaded.evidence)
        observed = [ordered]@{ proven_facts = @($Facts) }
    }
}

function Assert-UatDashboardDependencyPredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$ExpectedChecks
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "product-diagnostic.json"
    $raw = Read-UatReferencedJsonEvidence `
        -Lane $Lane `
        -Artifact $loaded.document.dashboard_dependency_semantics_evidence `
        -Label "raw Dashboard dependency semantics"
    Assert-UatDashboardDependencyEvidenceMatches `
        -Embedded $loaded.document.dashboard_dependency_semantics `
        -Raw $raw.document
    foreach ($code in $ExpectedChecks) {
        $matches = @($raw.document.checks | Where-Object { [string]$_.code -ceq $code })
        if ($matches.Count -ne 1 -or $matches[0].passed -ne $true) {
            throw "Dashboard dependency evidence does not contain one passing '$code' semantic check."
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($loaded.evidence, $raw.evidence)
        observed = [ordered]@{ passing_check_codes = @($ExpectedChecks) }
    }
}

function Assert-UatDashboardDependencyEvidenceMatches {
    param(
        [Parameter(Mandatory)]$Embedded,
        [Parameter(Mandatory)]$Raw
    )

    Assert-Sprint8ADashboardDependencyEvidence -Evidence $Embedded
    Assert-Sprint8ADashboardDependencyEvidence -Evidence $Raw
    $embeddedJson = $Embedded | ConvertTo-Json -Depth 100 -Compress
    $rawJson = $Raw | ConvertTo-Json -Depth 100 -Compress
    if ($embeddedJson -cne $rawJson) {
        throw "Embedded Dashboard dependency semantics diverge from the authenticated raw evidence."
    }
}

function Assert-UatSourceContractPredicate {
    param(
        [Parameter(Mandatory)][object[]]$Lanes,
        [Parameter(Mandatory)][string[]]$SourcePaths,
        [Parameter(Mandatory)][string[]]$RequiredFragments
    )
    $combined = ($SourcePaths | ForEach-Object {
        $fullPath = Resolve-UatEvidencePath -Path $_
        if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) { throw "Source-contract file is missing: $_" }
        Get-Content -LiteralPath $fullPath -Raw
    }) -join "`n"
    foreach ($fragment in $RequiredFragments) {
        if (-not $combined.Contains($fragment)) {
            throw "Source-bound contract omits required enforcement fragment '$fragment'."
        }
    }
    $evidence = @($Lanes | ForEach-Object { $_.raw_evidence }) + @($SourcePaths | ForEach-Object {
        $fullPath = Resolve-UatEvidencePath -Path $_
        [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $fullPath).Replace("\", "/")
            sha256 = Get-Sprint8AFileSha256 -Path $fullPath
        }
    })
    [pscustomobject][ordered]@{
        evidence = $evidence
        observed = [ordered]@{ executed_lanes = @($Lanes.name); source_fragments = @($RequiredFragments) }
    }
}

function Assert-UatFailureContainmentPredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$Facts
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "failure-containment-result.json"
    $document = $loaded.document
    if ([string]$document.contract -cne "tessara.sprint-8a.failure-containment-result" -or $document.passed -ne $true) {
        throw "Failure-containment evidence is not a passing exact result."
    }
    $evidence = [Collections.Generic.List[object]]::new()
    $evidence.Add($loaded.evidence)
    foreach ($fact in $Facts) {
        switch ($fact) {
            "expected_fault" {
                if ($document.failure.expected_fault_observed -ne $true -or
                    [string]$document.fault.classification -cne "validation_only_fault_injection" -or
                    [int]$document.fault.original_width -ne 4 -or [int]$document.fault.injected_width -ne 0) {
                    throw "Failure containment did not observe the exact induced owner fault."
                }
            }
            "exact_teardown" {
                $teardown = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.failure.teardown -Label "failure teardown"
                if ($teardown.document.teardown.passed -ne $true -or $teardown.document.teardown.after.empty -ne $true -or
                    @($teardown.document.teardown.after.containers).Count -ne 0 -or
                    @($teardown.document.teardown.after.present_volumes).Count -ne 0 -or
                    @($teardown.document.teardown.after.present_networks).Count -ne 0) {
                    throw "Failure containment teardown did not remove the exact disposable topology."
                }
                $evidence.Add($teardown.evidence)
            }
            "empty_successor" {
                $baseline = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.successor.empty_baseline -Label "successor empty baseline"
                if ($baseline.document.empty -ne $true -or $baseline.document.teardown.after.empty -ne $true) {
                    throw "Canonical successor did not begin from an empty baseline."
                }
                $evidence.Add($baseline.evidence)
            }
            "first_apply" {
                $first = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.successor.first_apply_response -Label "successor first apply"
                if ($first.document.no_op -ne $false) { throw "Canonical successor first apply was not a real apply." }
                $evidence.Add($first.evidence)
            }
            "no_op" {
                $noOp = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.successor.no_op_apply_response -Label "successor no-op"
                if ($noOp.document.no_op -ne $true) { throw "Canonical successor second apply was not a semantic no-op." }
                $evidence.Add($noOp.evidence)
            }
            "health" {
                $health = Read-UatReferencedJsonEvidence -Lane $Lane -Artifact $document.successor.final_health.receipt -Label "successor final health"
                if ($health.document.preceding_apply_no_op -ne $true -or $health.document.health.passed -ne $true) {
                    throw "Canonical successor health is not bound to its no-op apply."
                }
                $evidence.Add($health.evidence)
            }
            default { throw "Unknown failure-containment predicate fact '$fact'." }
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($evidence)
        observed = [ordered]@{ proven_facts = @($Facts) }
    }
}

function Assert-UatUpgradePredicate {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string[]]$Facts
    )
    $loaded = Read-UatLaneJsonEvidence -Lane $Lane -FileName "component-upgrade-rollback.json"
    $document = $loaded.document
    if ([string]$document.evidence_kind -cne "tessara.sprint-8a.component-upgrade-rollback" -or
        $document.passed -ne $true) {
        throw "Component upgrade evidence is not a passing exact result."
    }
    foreach ($fact in $Facts) {
        switch ($fact) {
            "candidate_upgrade" {
                if (@($document.transitions | Where-Object stage -CEQ "upgrade-to-candidate").Count -ne 1 -or
                    @($document.stage_snapshots | Where-Object stage -CEQ "candidate-upgrade").Count -ne 1) {
                    throw "Component-only candidate upgrade transition is missing."
                }
            }
            "baseline_rollback" {
                if (@($document.transitions | Where-Object stage -CEQ "rollback-to-baseline").Count -ne 1 -or
                    @($document.stage_snapshots | Where-Object stage -CEQ "baseline-rollback").Count -ne 1) {
                    throw "Component rollback transition is missing."
                }
            }
            "exact_preservation" {
                if ([string]$document.preservation.unrelated_container_image_restart_data_availability -cne "exact" -or
                    [string]$document.preservation.component_data_identity_configuration_routes_behavior -cne "exact" -or
                    [string]$document.preservation.bootstrap_receipts -cne "carried_forward_exactly") {
                    throw "Upgrade evidence does not prove exact Component and unrelated identity preservation."
                }
            }
            "candidate_restored" {
                if (@($document.transitions | Where-Object stage -CEQ "restore-intended-candidate").Count -ne 1 -or
                    @($document.stage_snapshots | Where-Object stage -CEQ "candidate-restored").Count -ne 1 -or
                    [string]$document.preservation.final_release -cne [string]$document.release_fixture.candidate.version) {
                    throw "Upgrade exercise did not restore the intended candidate release."
                }
            }
            default { throw "Unknown upgrade predicate fact '$fact'." }
        }
    }
    [pscustomobject][ordered]@{
        evidence = @($loaded.evidence)
        observed = [ordered]@{ proven_facts = @($Facts) }
    }
}

function Invoke-Sprint8AUatPredicate {
    param(
        [Parameter(Mandatory)][string]$AssertionId,
        [Parameter(Mandatory)]$PrerequisiteByName
    )
    $definition = $semanticPredicateRegistry[$AssertionId]
    $lanes = @($definition.producers | ForEach-Object { $PrerequisiteByName[[string]$_] })
    $invalid = @($lanes | Where-Object { $null -eq $_ -or [string]$_.state -cne "passed" })
    if ($invalid.Count -gt 0) {
        throw "Semantic assertion '$AssertionId' is blocked by a nonpassing producer."
    }
    $evaluation = switch ([string]$definition.evaluator) {
        "smoke" { Assert-UatSmokePredicate -Lane $lanes[0] -ExpectedChecks @($definition.smoke_checks); break }
        "playwright" { Assert-UatPlaywrightPredicate -Lane $lanes[0] -ExpectedTitles @($definition.playwright_titles); break }
        "smoke_and_playwright" {
            $smoke = Assert-UatSmokePredicate -Lane $lanes[0] -ExpectedChecks @($definition.smoke_checks)
            $playwright = Assert-UatPlaywrightPredicate -Lane $lanes[1] -ExpectedTitles @($definition.playwright_titles)
            [pscustomobject][ordered]@{
                evidence = @($smoke.evidence) + @($playwright.evidence)
                observed = [ordered]@{ smoke = $smoke.observed; playwright = $playwright.observed }
            }
            break
        }
        "materialization" { Assert-UatMaterializationPredicate -Lane $lanes[0] -Facts @($definition.materialization_facts); break }
        "inventory" { Assert-UatInventoryPredicate -Lane $lanes[0] -Facts @($definition.inventory_facts); break }
        "dashboard_dependency" { Assert-UatDashboardDependencyPredicate -Lane $lanes[0] -ExpectedChecks @($definition.dashboard_checks); break }
        "source_contract" { Assert-UatSourceContractPredicate -Lanes $lanes -SourcePaths @($definition.source_paths) -RequiredFragments @($definition.source_fragments); break }
        "failure_containment" { Assert-UatFailureContainmentPredicate -Lane $lanes[0] -Facts @($definition.containment_facts); break }
        "upgrade" { Assert-UatUpgradePredicate -Lane $lanes[0] -Facts @($definition.upgrade_facts); break }
        default { throw "Semantic assertion '$AssertionId' uses unknown evaluator '$($definition.evaluator)'." }
    }
    [pscustomobject][ordered]@{
        id = $AssertionId
        producers = @($definition.producers)
        evaluator = [string]$definition.evaluator
        state = "passed"
        evidence = @($evaluation.evidence)
        observed = $evaluation.observed
        failure_reason = $null
    }
}

function Get-UatSemanticFailureEvidence {
    param(
        [Parameter(Mandatory)]$Definition,
        [Parameter(Mandatory)]$PrerequisiteByName
    )
    @($Definition.producers | ForEach-Object {
        $lane = $PrerequisiteByName[[string]$_]
        if ($null -ne $lane.raw_evidence) { $lane.raw_evidence }
        @($lane.produced_evidence)
        [pscustomobject][ordered]@{ path = [string]$lane.path; sha256 = [string]$lane.sha256 }
    } | Where-Object {
        -not [string]::IsNullOrWhiteSpace([string]$_.path) -and [string]$_.sha256 -match '^[0-9a-f]{64}$'
    } | Sort-Object path, sha256 -Unique)
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
    $removedPredicate = $semanticPredicateRegistry["sanitized-diagnostics"]
    $semanticPredicateRegistry.Remove("sanitized-diagnostics")
    try {
        Invoke-ExpectedLaneGuardFailure { Assert-DiagnosticInventory } "an assertion label without an executable semantic predicate"
    } finally {
        $semanticPredicateRegistry["sanitized-diagnostics"] = $removedPredicate
    }
    Assert-DiagnosticInventory
    $playwrightFixture = [pscustomobject]@{
        suites = @([pscustomobject]@{
            title = "fixture.spec.ts"; file = "fixture.spec.ts"; suites = @()
            specs = @([pscustomobject]@{
                title = "semantic behavior"; tests = @([pscustomobject]@{
                    projectName = "chromium"; expectedStatus = "passed"
                    results = @([pscustomobject]@{ status = "passed"; retry = 0 })
                })
            })
        })
    }
    $parsedPlaywright = @(Get-UatPlaywrightResults -Report $playwrightFixture)
    if ($parsedPlaywright.Count -ne 1 -or
        [string]$parsedPlaywright[0].title -cne "semantic behavior" -or
        $parsedPlaywright[0].passed -ne $true) {
        throw "Self-test did not retain exact Playwright semantic identity and status."
    }
    $playwrightFixture.suites[0].specs[0].tests[0].results[0].retry = 1
    if (@(Get-UatPlaywrightResults -Report $playwrightFixture)[0].passed -ne $false) {
        throw "Self-test accepted retried Playwright evidence as an exact semantic pass."
    }
    $semanticFixture = [pscustomobject](New-Sprint8ADashboardDependencySelfTestEvidence)
    $rawSemanticFixture = ($semanticFixture | ConvertTo-Json -Depth 100) | ConvertFrom-Json
    Assert-UatDashboardDependencyEvidenceMatches -Embedded $semanticFixture -Raw $rawSemanticFixture
    $rawSemanticFixture.checks[0].detail = "divergent-but-individually-valid semantic detail"
    Invoke-ExpectedLaneGuardFailure {
        Assert-UatDashboardDependencyEvidenceMatches -Embedded $semanticFixture -Raw $rawSemanticFixture
    } "embedded Dashboard semantics that diverge from authenticated raw evidence"
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
$semanticFailures = [Collections.Generic.List[object]]::new()
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
        $canonicalProductDiagnostic = Get-UatLaneEvidence `
            -Lane $prerequisiteByName["live-product-diagnostics"] `
            -FileName "product-diagnostic.json"
        $productSemanticEvidence = Assert-ProductDiagnosticReceiptFile `
            -Path ([string]$canonicalProductDiagnostic.path) `
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
    $scenarioId = [string]$_.Key
    $dependencies = @($_.Value)
    $assertionIds = @($scenarioAssertions[$scenarioId])
    $missing = @($dependencies | Where-Object {
        -not $prerequisiteByName.ContainsKey([string]$_) -or
        [string]$prerequisiteByName[[string]$_].state -cne "passed"
    })
    if ($missing.Count -gt 0) {
        $reason = "blocked by invalid prerequisite(s): $($missing -join ', ')"
        [ordered]@{
            scenario = $scenarioId
            state = "blocked"
            diagnostic_dependencies = $dependencies
            assertion_ids = $assertionIds
            semantic_assertions = @($assertionIds | ForEach-Object {
                [ordered]@{
                    id = [string]$_
                    producers = @($semanticPredicateRegistry[[string]$_].producers)
                    evaluator = [string]$semanticPredicateRegistry[[string]$_].evaluator
                    state = "blocked"
                    classification = $null
                    evidence = @()
                    observed = $null
                    failure_reason = $reason
                }
            })
            dependency_reason = $reason
        }
    } else {
        $assertionResults = [Collections.Generic.List[object]]::new()
        foreach ($assertionId in $assertionIds) {
            try {
                $assertionResults.Add((Invoke-Sprint8AUatPredicate `
                    -AssertionId ([string]$assertionId) `
                    -PrerequisiteByName $prerequisiteByName))
            } catch {
                $failureReason = $_.Exception.Message
                $definition = $semanticPredicateRegistry[[string]$assertionId]
                $failedAssertion = [ordered]@{
                    id = [string]$assertionId
                    producers = @($definition.producers)
                    evaluator = [string]$definition.evaluator
                    state = "failed"
                    classification = "product"
                    evidence = @(Get-UatSemanticFailureEvidence -Definition $definition -PrerequisiteByName $prerequisiteByName)
                    observed = $null
                    failure_reason = $failureReason
                }
                $assertionResults.Add($failedAssertion)
                $semanticFailure = [pscustomobject][ordered]@{
                    name = "$scenarioId/$assertionId"
                    state = "failed"
                    classification = "product"
                    reason = $failureReason
                    evidence = @($failedAssertion.evidence)
                }
                $semanticFailures.Add($semanticFailure)
                $failures.Add($semanticFailure)
            }
        }
        $failedAssertions = @($assertionResults | Where-Object state -CEQ "failed")
        [ordered]@{
            scenario = $scenarioId
            state = if ($failedAssertions.Count -eq 0) { "passed" } else { "failed" }
            diagnostic_dependencies = $dependencies
            assertion_ids = $assertionIds
            semantic_assertions = @($assertionResults)
            dependency_reason = $null
        }
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
    harness_failure_count = $failures.Count - $semanticFailures.Count
    semantic_failure_count = $semanticFailures.Count
    semantic_failures = @($semanticFailures)
    blocked_count = $blocked.Count
    semantic_predicate_registry_version = 1
    cleanup_restoration = [ordered]@{
        required = $true
        result = if ($successorHealthy) { "canonical_successor_healthy" } else { "not_proven" }
    }
}
Publish-Sprint7AEvidence -Document $result -OutputPath $outputFullPath | Out-Null
if ([string]$result.state -cne "passed") {
    if ([string]$result.state -ceq "failed") {
        throw "Sprint 8A automated UAT diagnostics retained $($failures.Count) prerequisite or semantic predicate failure(s) at $outputFullPath."
    }
    Write-Host "Sprint 8A automated UAT diagnostics retained $($blocked.Count) dependency-blocked scenarios without creating derivative defects."
    return
}
Write-Host "All executable predicates for the eight Sprint 8A UAT diagnostic projections passed. Formal UAT was not performed."

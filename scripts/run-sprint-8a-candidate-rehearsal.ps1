[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$ReadinessReceipt = "artifacts/sprint-8a-closeout/validation-readiness-result.json",
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")

function Open-Sprint8AValidationAttemptLock {
    param([Parameter(Mandatory)][string]$Path)
    [IO.File]::Open($Path, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
}

function Test-Sprint8AExclusiveValidationLock {
    $path = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-lock-$([guid]::NewGuid().ToString('N')).lock"
    $first = Open-Sprint8AValidationAttemptLock -Path $path
    try {
        $secondRejected = $false
        try {
            $second = Open-Sprint8AValidationAttemptLock -Path $path
            $second.Dispose()
        } catch [IO.IOException] {
            $secondRejected = $true
        }
        if (-not $secondRejected) { throw "Exclusive validation lock self-test admitted a concurrent process." }
    } finally {
        $first.Dispose()
    }
    $reopened = Open-Sprint8AValidationAttemptLock -Path $path
    $reopened.Dispose()
    Remove-Item -LiteralPath $path -Force
}

function Assert-RehearsalGraph {
    param([Parameter(Mandatory)][object[]]$Checks)

    $names = @($Checks | ForEach-Object { [string]$_.name })
    if ($names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count) {
        throw "Candidate rehearsal requires nonempty, unique check names."
    }
    foreach ($check in $Checks) {
        if ([string]::IsNullOrWhiteSpace([string]$check.command)) {
            throw "Candidate rehearsal check '$($check.name)' lacks its exact command."
        }
        foreach ($dependency in @($check.depends_on)) {
            if ([string]$dependency -ceq [string]$check.name -or $names -cnotcontains [string]$dependency) {
                throw "Candidate rehearsal check '$($check.name)' has invalid dependency '$dependency'."
            }
        }
    }
    $visiting = @{}
    $visited = @{}
    function Visit-RehearsalCheck([string]$Name) {
        if ($visiting.ContainsKey($Name)) { throw "Candidate rehearsal check graph contains a cycle through '$Name'." }
        if ($visited.ContainsKey($Name)) { return }
        $visiting[$Name] = $true
        $check = @($Checks | Where-Object name -CEQ $Name)[0]
        foreach ($dependency in @($check.depends_on)) { Visit-RehearsalCheck -Name ([string]$dependency) }
        $visiting.Remove($Name)
        $visited[$Name] = $true
    }
    foreach ($name in $names) { Visit-RehearsalCheck -Name $name }
}

function Assert-RehearsalIndependentChecks {
    param(
        [Parameter(Mandatory)][object[]]$Checks,
        [Parameter(Mandatory)][string[]]$IndependentNames
    )

    foreach ($name in $IndependentNames) {
        $matches = @($Checks | Where-Object name -CEQ $name)
        if ($matches.Count -ne 1) {
            throw "Candidate rehearsal check '$name' must be declared exactly once."
        }
        if (@($matches[0].depends_on).Count -ne 0) {
            throw "Candidate rehearsal check '$name' must remain independent so state/readiness failure cannot suppress useful safe evidence."
        }
    }
}

function Test-RehearsalScheduler {
    $checks = @(
        [pscustomobject]@{ name = "attempt-state-prerequisite"; depends_on = @(); command = "fail" },
        [pscustomobject]@{ name = "validation-readiness-prerequisite"; depends_on = @(); command = "pass" },
        [pscustomobject]@{ name = "independent-sibling"; depends_on = @(); command = "pass" },
        [pscustomobject]@{ name = "destructive-dependent"; depends_on = @("attempt-state-prerequisite", "validation-readiness-prerequisite"); command = "must-not-run" }
    )
    Assert-RehearsalGraph -Checks $checks
    # The scheduler fixture predates the phase lock and exercises dependency
    # behavior directly, so graph validation is sufficient here.
    $states = @{}
    $executed = [Collections.Generic.List[string]]::new()
    foreach ($check in $checks) {
        $failedDependencies = @($check.depends_on | Where-Object { $states[[string]$_] -cne "passed" })
        if ($failedDependencies.Count -gt 0) {
            $states[[string]$check.name] = "blocked"
            continue
        }
        $executed.Add([string]$check.name)
        $states[[string]$check.name] = if ([string]$check.name -ceq "attempt-state-prerequisite") { "failed" } else { "passed" }
    }
    if ($states.'attempt-state-prerequisite' -cne "failed" -or
        $states.'validation-readiness-prerequisite' -cne "passed" -or
        $states.'independent-sibling' -cne "passed" -or
        $states.'destructive-dependent' -cne "blocked" -or
        $executed -cnotcontains "validation-readiness-prerequisite" -or
        $executed -cnotcontains "independent-sibling" -or
        $executed -ccontains "destructive-dependent") {
        throw "Candidate rehearsal self-test did not fail late and block only the true dependent."
    }
    $defects = @($checks | Where-Object { $states[[string]$_.name] -ceq "failed" })
    if ($defects.Count -ne 1) { throw "Candidate rehearsal self-test did not consolidate one batch for one diagnostic pass." }
}

function Assert-Sprint8ANestedUatReceiptIdentity {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$ExpectedEnvironment,
        [Parameter(Mandatory)]$ExpectedSource
    )

    $schema = $Receipt.schema_version
    if ($Receipt.PSObject.Properties.Name -notcontains "authoritative" -or
        -not ($schema -is [int] -or $schema -is [long]) -or
        [long]$schema -ne 2 -or
        $Receipt.sprint -isnot [string] -or
        [string]$Receipt.sprint -cne "sprint-8a" -or
        $Receipt.authoritative -isnot [bool] -or
        $Receipt.authoritative -ne $false -or
        $Receipt.phase -isnot [string] -or
        [string]$Receipt.phase -cne "candidate-rehearsal-uat-diagnostics" -or
        [int]$Receipt.attempt -ne $ExpectedAttempt -or
        $ExpectedEnvironment -notmatch '^[0-9a-f]{64}$' -or
        $Receipt.environment_fingerprint -isnot [string] -or
        [string]$Receipt.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$Receipt.environment_fingerprint -cne $ExpectedEnvironment) {
        throw "Nested UAT diagnostic receipt is not the exact non-authoritative Sprint 8A receipt for this attempt/environment."
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
            throw "Nested UAT diagnostic receipt carries a malformed or dirty mutable source identity."
        }
    }
    if (($Receipt.mutable_source_identity | ConvertTo-Json -Depth 10 -Compress) -cne
        ($ExpectedSource | ConvertTo-Json -Depth 10 -Compress)) {
        throw "Nested UAT diagnostic receipt is not bound to the current clean source identity."
    }
}

if ($SelfTest) {
    Test-Sprint8AExclusiveValidationLock
    Test-RehearsalScheduler
    Test-Sprint8AResultClassificationProjection
    $source = [pscustomobject]@{
        commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"
        acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64
    }
    $nested = [pscustomobject]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-uat-diagnostics"
        authoritative = $false; attempt = 3; environment_fingerprint = "e" * 64
        mutable_source_identity = $source
    }
    Assert-Sprint8ANestedUatReceiptIdentity -Receipt $nested -ExpectedAttempt 3 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source
    foreach ($mutation in @(
        [pscustomobject]@{ label = "authoritative nested UAT evidence"; property = "authoritative"; value = $true; restore = $false },
        [pscustomobject]@{ label = "numerically coerced nested UAT authority"; property = "authoritative"; value = 0; restore = $false },
        [pscustomobject]@{ label = "wrong nested UAT schema"; property = "schema_version"; value = 1; restore = 2 },
        [pscustomobject]@{ label = "string-coerced nested UAT schema"; property = "schema_version"; value = "2"; restore = 2 },
        [pscustomobject]@{ label = "wrong nested UAT phase"; property = "phase"; value = "candidate-rehearsal-lane"; restore = "candidate-rehearsal-uat-diagnostics" },
        [pscustomobject]@{ label = "malformed nested UAT environment"; property = "environment_fingerprint"; value = ("E" * 64); restore = ("e" * 64) }
    )) {
        $nested.($mutation.property) = $mutation.value
        try {
            Assert-Sprint8ANestedUatReceiptIdentity -Receipt $nested -ExpectedAttempt 3 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source
            throw "Self-test accepted $($mutation.label)."
        } catch {
            if ($_.Exception.Message -ceq "Self-test accepted $($mutation.label).") { throw }
        } finally {
            $nested.($mutation.property) = $mutation.restore
        }
    }
    $nested.mutable_source_identity = [pscustomobject]@{
        commit = "f" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"
        acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64
    }
    try {
        Assert-Sprint8ANestedUatReceiptIdentity -Receipt $nested -ExpectedAttempt 3 -ExpectedEnvironment ("e" * 64) -ExpectedSource $source
        throw "Self-test accepted a stale nested UAT source identity."
    } catch {
        if ($_.Exception.Message -ceq "Self-test accepted a stale nested UAT source identity.") { throw }
    } finally {
        $nested.mutable_source_identity = $source
    }
    Write-Host "Sprint 8A candidate rehearsal fail-late and nested-UAT authority self-test passed."
    return
}

if ($Attempt -lt 1) { throw "Candidate rehearsal requires -Attempt with a positive, unused attempt number." }
$evidenceRootPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
    [IO.Path]::GetFullPath($EvidenceRoot)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
}
$attemptRoot = Join-Path $evidenceRootPath "rehearsal/attempt-$Attempt"
$laneRoot = Join-Path $attemptRoot "lanes"
$logRoot = Join-Path $attemptRoot "logs"
$attemptPath = Join-Path $evidenceRootPath "attempts/candidate-rehearsal-$Attempt-attempt.json"
$harvestPath = Join-Path $evidenceRootPath "attempts/candidate-rehearsal-$Attempt-harvest.json"
$batchPath = Join-Path $evidenceRootPath "attempts/candidate-rehearsal-$Attempt-defect-batch.json"
$correctionAuthorizationPath = Join-Path $evidenceRootPath "attempts/candidate-rehearsal-$Attempt-correction-authorization.json"
$resultPath = Join-Path $evidenceRootPath "candidate-rehearsal-result.json"
$statePath = Join-Path $evidenceRootPath "validation-state.json"
$validationLockPath = Join-Path $evidenceRootPath "validation-attempt.lock"
$validationLockHandle = $null
$readinessPath = if ([IO.Path]::IsPathRooted($ReadinessReceipt)) {
    [IO.Path]::GetFullPath($ReadinessReceipt)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $ReadinessReceipt))
}
foreach ($path in @($attemptPath, "$attemptPath.sha256", $attemptRoot)) {
    if (Test-Path -LiteralPath $path) { throw "Candidate rehearsal attempt $Attempt already exists and cannot be reused: $path" }
}
[IO.Directory]::CreateDirectory((Split-Path -Parent $attemptPath)) | Out-Null
[IO.Directory]::CreateDirectory($laneRoot) | Out-Null
[IO.Directory]::CreateDirectory($logRoot) | Out-Null

$relativeAttemptRoot = [IO.Path]::GetRelativePath($repoRoot, $attemptRoot).Replace("\", "/")
$materializationResult = Join-Path $attemptRoot "materialization/attempt-$Attempt/materialization-result.json"
$inventoryInitial = Join-Path $attemptRoot "deployed-inventory-initial.json"
$deploymentInitial = Join-Path $attemptRoot "deployment-initial.json"
$smokeInitial = Join-Path $attemptRoot "smoke-initial.json"
$playwrightInventoryEvidence = Join-Path $attemptRoot "playwright-inventory.json"
$playwrightInventorySidecar = "$playwrightInventoryEvidence.sha256"
$playwrightEvidence = Join-Path $attemptRoot "playwright-acceptance.json"
$playwrightDiscoveryEvidence = Join-Path $attemptRoot "playwright-acceptance.discovery.json"
$playwrightJunitEvidence = Join-Path $attemptRoot "playwright-acceptance.xml"
$playwrightSummaryEvidence = Join-Path $attemptRoot "playwright-acceptance.summary.json"
$playwrightFailure = Join-Path $attemptRoot "playwright-failure"
$upgradeResult = Join-Path $attemptRoot "component-upgrade-rollback.json"
$failureContainmentResult = Join-Path $attemptRoot "failure-containment/attempt-$Attempt/failure-containment-result.json"
$inventorySuccessor = Join-Path $attemptRoot "deployed-inventory-successor.json"
$deploymentSuccessor = Join-Path $attemptRoot "deployment-successor.json"
$smokeSuccessor = Join-Path $attemptRoot "smoke-successor.json"
$productDiagnosticEvidence = Join-Path $attemptRoot "product-diagnostic.json"
$productDiagnosticRawEvidence = "$productDiagnosticEvidence.dashboard-dependencies.json"
$uatResult = Join-Path $attemptRoot "uat-diagnostics.json"

$declaredChecks = @(
    [ordered]@{ name = "attempt-state-prerequisite"; depends_on = @(); command = "validate active-attempt lock and current readiness transition"; classification = "preflight/setup"; evidence_paths = @() },
    [ordered]@{ name = "validation-readiness-prerequisite"; depends_on = @(); command = "validate readiness receipt/source/environment identity"; classification = "preflight/setup"; evidence_paths = @($readinessPath) },
    [ordered]@{ name = "formatting"; depends_on = @(); command = "cargo fmt --all -- --check"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "workspace-check"; depends_on = @(); command = "cargo check --workspace --all-features --locked --offline"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "workspace-clippy"; depends_on = @(); command = "cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "compose-manifest-schema-contract"; depends_on = @(); command = "quiet Compose validation plus Sprint 8A manifest, asset, schema, fixture, Blueprint, and runner contract"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "web-native-wasm-source-boundaries"; depends_on = @(); command = "check-web-crate-boundaries.ps1 native/WASM/package/source ownership"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "module-sdk-boundaries"; depends_on = @(); command = "verify-module-sdk-boundaries.ps1 native/WASM/package/source audit"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "dashboard-source-boundaries"; depends_on = @(); command = "verify-sprint-6e-boundaries.ps1 Dashboard source/package/gateway ownership"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "markdown-links"; depends_on = @(); command = "verify-markdown-links.ps1"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "workspace-tests"; depends_on = @(); command = "cargo test --workspace --all-features --locked --offline"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "optimized-resource-reference-timing"; depends_on = @(); command = "cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "components-contract-tests"; depends_on = @(); command = "cargo test --locked --offline -p tessara-components-contract"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "dashboard-module-tests"; depends_on = @(); command = "cargo test --locked --offline -p tessara-dashboard-module"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "component-conformance-nondisclosure"; depends_on = @(); command = "cargo test --locked --offline -p tessara-component-module"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "module-testkit-conformance"; depends_on = @(); command = "cargo test --locked --offline -p tessara-module-testkit"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "playwright-discovery"; depends_on = @(); command = "validate-e2e.ps1 -InventoryOnly exact acceptance-manifest identity"; classification = "harness"; evidence_paths = @($playwrightInventoryEvidence, $playwrightInventorySidecar) },
    [ordered]@{ name = "source-exact-materialization-no-op"; depends_on = @("attempt-state-prerequisite", "validation-readiness-prerequisite"); command = "materialize-sprint-8a.ps1 authorized reset, first apply, and exact no-op"; classification = "environment"; evidence_paths = @($materializationResult); evidence_roots = @((Split-Path -Parent $materializationResult)) },
    [ordered]@{ name = "deployed-inventory-navigation-audit"; depends_on = @("source-exact-materialization-no-op"); command = "audit-sprint-8a-deployed-inventory.ps1 initial topology"; classification = "product"; evidence_paths = @($inventoryInitial) },
    [ordered]@{ name = "deployment-evidence"; depends_on = @("source-exact-materialization-no-op"); command = "run-sprint-8a-deployed-smoke.ps1 initial source-exact deployment evidence"; classification = "evidence-finalization"; evidence_paths = @($deploymentInitial) },
    [ordered]@{ name = "product-smoke"; depends_on = @("source-exact-materialization-no-op"); command = "smoke-sprint-8a.ps1 initial product/ownership smoke"; classification = "product"; evidence_paths = @($smokeInitial) },
    [ordered]@{ name = "playwright-execution"; depends_on = @("source-exact-materialization-no-op", "deployment-evidence"); command = "validate-e2e.ps1 complete source-bound acceptance-manifest inventory"; classification = "product"; evidence_paths = @($playwrightEvidence, $playwrightDiscoveryEvidence, $playwrightJunitEvidence, $playwrightSummaryEvidence); evidence_roots = @($playwrightFailure) },
    [ordered]@{ name = "component-upgrade-rollback"; depends_on = @("source-exact-materialization-no-op"); command = "run-sprint-8a-component-upgrade.ps1 isolated upgrade/rollback/restore"; classification = "product"; evidence_paths = @($upgradeResult) },
    [ordered]@{ name = "failure-containment-successor-health"; depends_on = @("attempt-state-prerequisite", "validation-readiness-prerequisite"); command = "run-sprint-8a-failure-containment.ps1 induced fault, teardown, from-empty successor and no-op; reuse images only after source-exact materialization passed"; classification = "product"; evidence_paths = @($failureContainmentResult); evidence_roots = @((Split-Path -Parent $failureContainmentResult)) },
    [ordered]@{ name = "successor-inventory-navigation-audit"; depends_on = @("failure-containment-successor-health"); command = "audit-sprint-8a-deployed-inventory.ps1 successor topology"; classification = "product"; evidence_paths = @($inventorySuccessor) },
    [ordered]@{ name = "successor-deployment-evidence"; depends_on = @("failure-containment-successor-health"); command = "run-sprint-8a-deployed-smoke.ps1 successor source-exact deployment evidence"; classification = "evidence-finalization"; evidence_paths = @($deploymentSuccessor) },
    [ordered]@{ name = "successor-product-smoke"; depends_on = @("failure-containment-successor-health"); command = "smoke-sprint-8a.ps1 successor product/ownership smoke"; classification = "product"; evidence_paths = @($smokeSuccessor) },
    [ordered]@{ name = "live-product-diagnostics"; depends_on = @("deployment-evidence", "product-smoke"); command = "diagnose-sprint-8a-product.ps1 non-acceptance semantic product diagnostic before canonical successor reset"; classification = "product"; evidence_paths = @($productDiagnosticEvidence, $productDiagnosticRawEvidence, "$productDiagnosticRawEvidence.sha256") },
    [ordered]@{ name = "uat-diagnostics"; depends_on = @("validation-readiness-prerequisite"); command = "uat-sprint-8a.ps1 project exact eight-scenario results from every terminal prerequisite receipt"; classification = "harness"; evidence_paths = @($uatResult); nested_results_path = $uatResult },
    [ordered]@{ name = "final-clean-source"; depends_on = @("validation-readiness-prerequisite"); command = "final unchanged clean source identity"; classification = "product"; evidence_paths = @() },
    [ordered]@{ name = "final-environment-identity"; depends_on = @("validation-readiness-prerequisite"); command = "final authenticated unchanged environment identity"; classification = "environment"; evidence_paths = @() },
    [ordered]@{ name = "final-successor-health"; depends_on = @("failure-containment-successor-health"); command = "final canonical successor gateway health"; classification = "product"; evidence_paths = @() }
)
Assert-RehearsalGraph -Checks $declaredChecks
Assert-RehearsalIndependentChecks -Checks $declaredChecks -IndependentNames @(
    "attempt-state-prerequisite", "validation-readiness-prerequisite", "formatting", "workspace-check", "workspace-clippy",
    "compose-manifest-schema-contract", "web-native-wasm-source-boundaries", "module-sdk-boundaries",
    "dashboard-source-boundaries", "markdown-links", "workspace-tests",
    "optimized-resource-reference-timing", "components-contract-tests", "dashboard-module-tests",
    "component-conformance-nondisclosure", "module-testkit-conformance", "playwright-discovery"
)

$source = [pscustomobject][ordered]@{
    commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
    acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
}
$runtimeContext = [ordered]@{
    readiness = $null
    readiness_sha256 = $null
    validation_state = $null
    correction_transition = $null
    launch_authorized = $false
    source_verification_state = "unverified"
    source_verification_failure = $null
    environment = [ordered]@{ fingerprint = "0" * 64; verified = $false; verification_state = "unverified" }
}

$startedAt = [DateTimeOffset]::UtcNow
$attemptReceipt = [ordered]@{
    schema_version = 2
    sprint = "sprint-8a"
    phase = "candidate-rehearsal"
    attempt = $Attempt
    authoritative = $false
    state = "preparing"
    assertions_started = $false
    assertions_started_at = $null
    started_at = $startedAt.ToString("o")
    ended_at = $null
    mutable_source_identity = $source
    source_identity_verification_state = "unverified"
    source_identity_verification_failure = $null
    environment_identity = [ordered]@{
        readiness_receipt = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
        readiness_sha256 = $null
        verification_state = "unverified"
        verification_failure = $null
    }
    environment_fingerprint = [string]$runtimeContext.environment.fingerprint
    prerequisite_receipts = @()
    checks = $declaredChecks
    assertion_count = 0
    failure_count = 0
    blocked_count = 0
    nested_blocked_count = 0
    nested_failure_count = 0
    classification = $null
    invalidation_decision = "candidate freeze forbidden until every declared check passes"
    cleanup_restoration = [ordered]@{ required = $true; result = "pending" }
}
Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath | Out-Null

$terminalChecks = [Collections.Generic.List[object]]::new()
$terminalByName = @{}
$assertionsStartedAt = $null

function Resolve-LaneClassification {
    param([string]$Default, [string]$Detail)
    if ($Detail -match '(?i)Required Sprint 8A tool .* unavailable|Docker daemon is not running|Cannot connect to the Docker daemon|Authenticated database probe failed|connection refused while probing required') { return "environment" }
    if ($Detail -match '(?i)ParameterBindingException|cannot be found that matches parameter name|receipt SHA-256 sidecar|Failure-containment artifact sidecar|declared digest does not match|runner .* does not parse') { return "harness" }
    $Default
}

function Invoke-RehearsalLane {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][scriptblock]$Action
    )

    $declaration = @($declaredChecks | Where-Object name -CEQ $Name)
    if ($declaration.Count -ne 1) { throw "Rehearsal lane '$Name' was not declared exactly once." }
    $failedDependencies = @($declaration[0].depends_on | Where-Object {
        -not $terminalByName.ContainsKey([string]$_) -or [string]$terminalByName[[string]$_].state -cne "passed"
    })
    if ($failedDependencies.Count -gt 0) {
        $blocked = [pscustomobject][ordered]@{
            name = $Name; depends_on = @($declaration[0].depends_on); command = [string]$declaration[0].command
            started_at = $null; ended_at = [DateTimeOffset]::UtcNow.ToString("o"); duration_ms = 0; exit_status = $null
            state = "blocked"; classification = [string]$declaration[0].classification
            dependency_reason = "blocked by failed prerequisite(s): $($failedDependencies -join ', ')"
            evidence_path = $null; evidence_sha256 = $null; produced_evidence = @(); failure_message = $null
            nested_blocked_checks = @(); nested_failed_checks = @()
        }
        $terminalChecks.Add($blocked)
        $terminalByName[$Name] = $blocked
        Publish-Sprint7AEvidence -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
            authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
            result = $blocked
        }) -OutputPath (Join-Path $laneRoot "$Name.json") | Out-Null
        return
    }

    if ($Name -cne "attempt-state-prerequisite" -and -not [bool]$attemptReceipt.assertions_started) {
        $script:assertionsStartedAt = [DateTimeOffset]::UtcNow
        $attemptReceipt.state = "executing"
        $attemptReceipt.assertions_started = $true
        $attemptReceipt.assertions_started_at = $script:assertionsStartedAt.ToString("o")
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
    }
    $start = [DateTimeOffset]::UtcNow
    $logPath = Join-Path $logRoot "$Name.log"
    $laneReceiptPath = Join-Path $laneRoot "$Name.json"
    [IO.File]::WriteAllText(
        $logPath,
        "[$($start.ToString('o'))] lane_started name=$Name`n",
        [Text.UTF8Encoding]::new($false)
    )
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
        authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        result = [ordered]@{
            name = $Name; depends_on = @($declaration[0].depends_on); command = [string]$declaration[0].command
            started_at = $start.ToString("o"); ended_at = $null; duration_ms = $null; exit_status = $null
            state = "executing"; classification = $null; dependency_reason = $null
            evidence_path = [IO.Path]::GetRelativePath($repoRoot, $logPath).Replace("\", "/")
            evidence_sha256 = $null; produced_evidence = @(); failure_message = $null
            nested_blocked_checks = @(); nested_failed_checks = @()
        }
    }) -OutputPath $laneReceiptPath | Out-Null
    $passed = $false
    $failureMessage = $null
    $nestedBlockedChecks = @()
    $nestedFailedChecks = @()
    try {
        & $Action *>&1 | ForEach-Object {
            $rendered = ($_ | Out-String).TrimEnd()
            if (-not [string]::IsNullOrEmpty($rendered)) {
                [IO.File]::AppendAllText(
                    $logPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] $rendered`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        foreach ($evidencePath in @($declaration[0].evidence_paths)) {
            if (-not (Test-Path -LiteralPath $evidencePath -PathType Leaf)) {
                throw "Lane '$Name' did not produce required evidence '$evidencePath'."
            }
        }
        $passed = $true
    } catch {
        $failureMessage = $_.Exception.Message
        [IO.File]::AppendAllText(
            $logPath,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] lane_failure`n$($_ | Out-String)`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    if ($declaration[0].Contains("nested_results_path") -and
        (Test-Path -LiteralPath ([string]$declaration[0].nested_results_path) -PathType Leaf)) {
        try {
            $nestedResult = Get-Content -LiteralPath ([string]$declaration[0].nested_results_path) -Raw | ConvertFrom-Json
            Assert-Sprint8ANestedUatReceiptIdentity `
                -Receipt $nestedResult `
                -ExpectedAttempt $Attempt `
                -ExpectedEnvironment ([string]$runtimeContext.environment.fingerprint) `
                -ExpectedSource $source
            $expectedScenarios = @(1..8 | ForEach-Object { "UAT-8A-{0:d2}" -f $_ })
            $nestedChecks = @($nestedResult.checks)
            $actualScenarios = @($nestedChecks | ForEach-Object { [string]$_.scenario })
            if ($nestedChecks.Count -ne $expectedScenarios.Count -or
                @($actualScenarios | Sort-Object -Unique).Count -ne $actualScenarios.Count -or
                (($actualScenarios | Sort-Object) -join ',') -cne (($expectedScenarios | Sort-Object) -join ',')) {
                throw "Lane '$Name' nested diagnostic receipt does not contain the exact eight UAT scenario identities."
            }
            foreach ($nestedCheck in $nestedChecks) {
                if (@("passed", "failed", "blocked") -cnotcontains [string]$nestedCheck.state) {
                    throw "Lane '$Name' nested scenario '$($nestedCheck.scenario)' has unsupported state '$($nestedCheck.state)'."
                }
                $assertionIds = @($nestedCheck.assertion_ids | ForEach-Object { [string]$_ })
                $semanticAssertions = @($nestedCheck.semantic_assertions)
                $actualAssertionIds = @($semanticAssertions | ForEach-Object { [string]$_.id })
                if ($assertionIds.Count -lt 1 -or
                    $semanticAssertions.Count -ne $assertionIds.Count -or
                    @($actualAssertionIds | Sort-Object -Unique).Count -ne $actualAssertionIds.Count -or
                    (($actualAssertionIds | Sort-Object) -join ',') -cne (($assertionIds | Sort-Object) -join ',')) {
                    throw "Lane '$Name' nested scenario '$($nestedCheck.scenario)' does not retain its exact semantic assertion inventory."
                }
                foreach ($semanticAssertion in $semanticAssertions) {
                    if (@("passed", "failed", "blocked") -cnotcontains [string]$semanticAssertion.state -or
                        [string]::IsNullOrWhiteSpace([string]$semanticAssertion.evaluator) -or
                        @($semanticAssertion.producers).Count -lt 1) {
                        throw "Lane '$Name' nested semantic assertion '$($semanticAssertion.id)' is malformed."
                    }
                    if ([string]$semanticAssertion.state -ceq "passed" -and
                        @($semanticAssertion.evidence).Count -lt 1) {
                        throw "Lane '$Name' nested semantic assertion '$($semanticAssertion.id)' passed without direct evidence."
                    }
                    if ([string]$semanticAssertion.state -ceq "failed" -and
                        ([string]::IsNullOrWhiteSpace([string]$semanticAssertion.failure_reason) -or
                            @("product", "harness", "environment", "evidence-finalization") -cnotcontains [string]$semanticAssertion.classification -or
                            @($semanticAssertion.evidence).Count -lt 1)) {
                        throw "Lane '$Name' nested semantic assertion '$($semanticAssertion.id)' failed without exact reason, evidence, and allowed classification."
                    }
                }
                if ([string]$nestedCheck.state -ceq "passed" -and
                    @($semanticAssertions | Where-Object state -CNE "passed").Count -ne 0) {
                    throw "Lane '$Name' nested scenario '$($nestedCheck.scenario)' passed without every semantic predicate passing."
                }
                if ([string]$nestedCheck.state -ceq "failed" -and
                    @($semanticAssertions | Where-Object state -CEQ "failed").Count -lt 1) {
                    throw "Lane '$Name' nested scenario '$($nestedCheck.scenario)' failed without a failed semantic predicate."
                }
                if ([string]$nestedCheck.state -ceq "blocked" -and
                    [string]::IsNullOrWhiteSpace([string]$nestedCheck.dependency_reason)) {
                    throw "Lane '$Name' nested blocked scenario '$($nestedCheck.scenario)' lacks its exact dependency reason."
                }
                if ([string]$nestedCheck.state -ceq "blocked" -and
                    @($semanticAssertions | Where-Object state -CNE "blocked").Count -ne 0) {
                    throw "Lane '$Name' nested blocked scenario '$($nestedCheck.scenario)' contains a nonblocked semantic predicate."
                }
                if ([string]$nestedCheck.state -ceq "blocked") {
                    $blockedBy = @($nestedCheck.diagnostic_dependencies | Where-Object {
                        -not $terminalByName.ContainsKey([string]$_) -or
                            [string]$terminalByName[[string]$_].state -cne "passed"
                    })
                    if ($blockedBy.Count -eq 0 -or
                        @($blockedBy | Where-Object {
                            -not ([string]$nestedCheck.dependency_reason).Contains([string]$_)
                        }).Count -gt 0) {
                        throw "Lane '$Name' nested blocked scenario '$($nestedCheck.scenario)' does not name every nonpassing diagnostic prerequisite."
                    }
                }
            }
            $nestedFailedScenarios = @($nestedChecks | Where-Object state -CEQ "failed")
            $nestedBlockedScenarios = @($nestedChecks | Where-Object state -CEQ "blocked")
            $expectedNestedState = if ($nestedFailedScenarios.Count -gt 0 -or [int]$nestedResult.failure_count -gt 0) {
                "failed"
            } elseif ($nestedBlockedScenarios.Count -gt 0) {
                "blocked"
            } else {
                "passed"
            }
            if ([string]$nestedResult.state -cne $expectedNestedState -or
                [int]$nestedResult.blocked_count -ne $nestedBlockedScenarios.Count -or
                [int]$nestedResult.semantic_predicate_registry_version -ne 1 -or
                ($expectedNestedState -ceq "failed" -and [int]$nestedResult.failure_count -lt 1) -or
                ($expectedNestedState -cne "failed" -and [int]$nestedResult.failure_count -ne 0)) {
                throw "Lane '$Name' nested diagnostic summary does not match its executable scenario predicates."
            }
            $nestedBlockedChecks = @($nestedChecks | Where-Object state -CEQ "blocked" | ForEach-Object {
                $blockedBy = @($_.diagnostic_dependencies | Where-Object {
                    -not $terminalByName.ContainsKey([string]$_) -or
                        [string]$terminalByName[[string]$_].state -cne "passed"
                } | Sort-Object -Unique)
                [ordered]@{
                    name = "uat-diagnostics/$([string]$_.scenario)"
                    parent_check = "uat-diagnostics"
                    blocked_by = $blockedBy
                    dependency_reason = [string]$_.dependency_reason
                }
            })
            $nestedFailedChecks = @($nestedChecks | ForEach-Object {
                $scenarioId = [string]$_.scenario
                @($_.semantic_assertions | Where-Object state -CEQ "failed" | ForEach-Object {
                    [ordered]@{
                        name = "uat-diagnostics/$scenarioId/$([string]$_.id)"
                        parent_check = "uat-diagnostics"
                        scenario = $scenarioId
                        assertion_id = [string]$_.id
                        classification = [string]$_.classification
                        failure_reason = [string]$_.failure_reason
                        raw_evidence = @($_.evidence)
                    }
                })
            })
            if ($nestedFailedChecks.Count -gt 0 -and [int]$nestedResult.harness_failure_count -eq 0) {
                # The projection runner succeeded and retained product-level
                # semantic defects. Do not double count its outer lane as a
                # generic harness defect.
                $passed = $true
                $failureMessage = $null
            }
        } catch {
            if ($passed) { $passed = $false }
            if ([string]::IsNullOrWhiteSpace($failureMessage)) { $failureMessage = $_.Exception.Message }
            [IO.File]::AppendAllText(
                $logPath,
                "[$([DateTimeOffset]::UtcNow.ToString('o'))] nested_result_failure`n$($_ | Out-String)`n",
                [Text.UTF8Encoding]::new($false)
            )
        }
    }
    $end = [DateTimeOffset]::UtcNow
    [IO.File]::AppendAllText(
        $logPath,
        "[$($end.ToString('o'))] lane_finished state=$(if ($passed) { 'passed' } else { 'failed' })`n",
        [Text.UTF8Encoding]::new($false)
    )
    $detailTail = (Get-Content -LiteralPath $logPath -Tail 500 | Out-String)
    $producedEvidencePaths = [Collections.Generic.List[string]]::new()
    foreach ($path in @($declaration[0].evidence_paths)) {
        if (Test-Path -LiteralPath $path -PathType Leaf) {
            $producedEvidencePaths.Add([IO.Path]::GetFullPath($path))
        }
    }
    if ($declaration[0].Contains("evidence_roots")) {
        foreach ($root in @($declaration[0].evidence_roots)) {
            if (Test-Path -LiteralPath $root -PathType Container) {
                foreach ($file in Get-ChildItem -LiteralPath $root -File -Recurse | Sort-Object FullName) {
                    $producedEvidencePaths.Add($file.FullName)
                }
            }
        }
    }
    $producedEvidence = @($producedEvidencePaths | Sort-Object -Unique | ForEach-Object {
        [ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $_).Replace("\", "/")
            sha256 = Get-Sprint8AFileSha256 -Path $_
        }
    })
    $entry = [pscustomobject][ordered]@{
        name = $Name; depends_on = @($declaration[0].depends_on); command = [string]$declaration[0].command
        started_at = $start.ToString("o"); ended_at = $end.ToString("o"); duration_ms = [math]::Round(($end - $start).TotalMilliseconds)
        exit_status = if ($passed) { 0 } else { 1 }; state = if ($passed) { "passed" } else { "failed" }
        classification = if ($passed) { $null } else { Resolve-LaneClassification -Default ([string]$declaration[0].classification) -Detail $detailTail }
        dependency_reason = $null
        evidence_path = [IO.Path]::GetRelativePath($repoRoot, $logPath).Replace("\", "/")
        evidence_sha256 = Get-Sprint8AFileSha256 -Path $logPath
        produced_evidence = $producedEvidence
        failure_message = $failureMessage
        nested_blocked_checks = @($nestedBlockedChecks)
        nested_failed_checks = @($nestedFailedChecks)
    }
    $terminalChecks.Add($entry)
    $terminalByName[$Name] = $entry
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
        authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        result = $entry
    }) -OutputPath $laneReceiptPath -Overwrite | Out-Null
    if (-not $passed -and [string]$attemptReceipt.state -cne "harvesting") {
        $attemptReceipt.state = "harvesting"
        $attemptReceipt.failure_count = 1
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
    }
}

Push-Location $repoRoot
try {
    Invoke-RehearsalLane "attempt-state-prerequisite" {
        $script:validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath
        if (-not (Test-Path -LiteralPath $statePath -PathType Leaf)) {
            throw "Candidate rehearsal requires the current validation-state index written by Readiness."
        }
        $stateIndex = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        if (@("preparing", "executing", "harvesting") -ccontains [string]$stateIndex.readiness.state -or
            @("preparing", "executing", "harvesting") -ccontains [string]$stateIndex.rehearsal.state) {
            throw "Validation-state identifies an active attempt; concurrent Candidate Rehearsal is locked out."
        }
        $relativeReadinessPath = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
        $stateReadinessSha = Assert-Sprint8AReceiptSidecar -Path $readinessPath
        $stateReadinessDocument = Get-Content -LiteralPath $readinessPath -Raw | ConvertFrom-Json
        if ([string]$stateIndex.readiness.state -cne "passed" -or
            [string]$stateIndex.readiness.receipt -cne $relativeReadinessPath -or
            [string]$stateIndex.readiness.sha256 -cne $stateReadinessSha -or
            [bool]$stateIndex.preflight_eligible) {
            throw "Validation-state does not identify the supplied current passing Readiness as the sole rehearsal prerequisite."
        }
        if ($stateIndex.PSObject.Properties.Name -contains "correction_transition" -and
            $null -ne $stateIndex.correction_transition) {
            $transition = $stateIndex.correction_transition
            if ($null -eq $transition.consumed_by_readiness -or
                [int]$transition.consumed_by_readiness.attempt -ne [int]$stateIndex.readiness.attempt -or
                [string]$transition.consumed_by_readiness.receipt -cne [string]$stateIndex.readiness.receipt -or
                [string]$transition.consumed_by_readiness.receipt_sha256 -notmatch '^[0-9a-f]{64}$' -or
                [string]$transition.consumed_by_readiness.receipt_sha256 -cne [string]$stateIndex.readiness.sha256 -or
                [string]$transition.authorization.sha256 -notmatch '^[0-9a-f]{64}$') {
                throw "Validation-state correction transition lacks exact one-time successor Readiness consumption."
            }
            $consumptionRef = $transition.consumed_by_readiness.consumption_receipt
            $consumptionPath = if ([IO.Path]::IsPathRooted([string]$consumptionRef.path)) {
                [IO.Path]::GetFullPath([string]$consumptionRef.path)
            } else {
                [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$consumptionRef.path)))
            }
            if ((Assert-Sprint8AReceiptSidecar -Path $consumptionPath) -cne [string]$consumptionRef.sha256) {
                throw "Correction-consumption receipt differs from validation-state."
            }
            $consumption = Get-Content -LiteralPath $consumptionPath -Raw | ConvertFrom-Json
            if ([string]$consumption.phase -cne "candidate-rehearsal-correction-consumption" -or
                [string]$consumption.state -cne "consumed" -or
                [int]$consumption.successor_readiness.attempt -ne [int]$transition.consumed_by_readiness.attempt -or
                [int]$transition.consumed_by_readiness.attempt -ne [int]$stateIndex.readiness.attempt -or
                [string]$consumption.authorization.sha256 -cne [string]$transition.authorization.sha256) {
                throw "Correction-consumption receipt does not bind the exact authorization and current successor Readiness."
            }
            if ($stateReadinessDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
                $null -eq $stateReadinessDocument.predecessor_correction_authorization -or
                [string]$stateReadinessDocument.predecessor_correction_authorization.sha256 -cne [string]$transition.authorization.sha256 -or
                $stateReadinessDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
                [string]$stateReadinessDocument.correction_consumption_receipt.path -cne [string]$consumptionRef.path -or
                [string]$stateReadinessDocument.correction_consumption_receipt.sha256 -cne [string]$consumptionRef.sha256) {
                throw "Current passing Readiness does not bind the exact authorization and append-only consumption receipt in validation-state."
            }
            $runtimeContext.correction_transition = $transition
        }
        $runtimeContext.validation_state = $stateIndex
        $runtimeContext.launch_authorized = $true
        Publish-Sprint7AEvidence -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; updated_at = [DateTimeOffset]::UtcNow.ToString("o")
            source_identity = $source; source_identity_verification_state = "unverified"
            environment_fingerprint = "0" * 64
            readiness = $stateIndex.readiness
            rehearsal = [ordered]@{
                attempt = $Attempt; state = "preparing"
                receipt = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
            }
            correction_transition = $runtimeContext.correction_transition
            preflight_eligible = $false
        }) -OutputPath $statePath -Overwrite | Out-Null
        "candidate rehearsal launch authorized and active-attempt lock acquired"
    }
    Invoke-RehearsalLane "validation-readiness-prerequisite" {
        $validatedReadinessSha = Assert-Sprint8AReceiptSidecar -Path $readinessPath
        $validatedReadiness = Get-Content -LiteralPath $readinessPath -Raw | ConvertFrom-Json
        if ([string]$validatedReadiness.state -cne "passed" -or
            [string]$validatedReadiness.phase -cne "validation-readiness") {
            throw "Candidate rehearsal requires one passing Validation Readiness receipt."
        }
        try {
            $script:source = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
            $runtimeContext.source_verification_state = "verified"
            $runtimeContext.source_verification_failure = $null
            $attemptReceipt.mutable_source_identity = $script:source
            $attemptReceipt.source_identity_verification_state = "verified"
            $attemptReceipt.source_identity_verification_failure = $null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        } catch {
            $runtimeContext.source_verification_state = "failed"
            $runtimeContext.source_verification_failure = $_.Exception.Message
            $attemptReceipt.source_identity_verification_state = "failed"
            $attemptReceipt.source_identity_verification_failure = $_.Exception.Message
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            throw
        }
        if ($source.dirty) { throw "Candidate rehearsal requires clean source." }
        if (($source | ConvertTo-Json -Depth 10 -Compress) -cne
            ($validatedReadiness.mutable_source_identity | ConvertTo-Json -Depth 10 -Compress)) {
            throw "Candidate rehearsal source identity differs from passing readiness."
        }
        try {
            $validatedEnvironment = Get-Sprint8AEnvironmentContract `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $EvidenceRoot `
                -ProbeDatabases
        } catch {
            $runtimeContext.environment.verification_state = "failed"
            $attemptReceipt.environment_identity.verification_state = "failed"
            $attemptReceipt.environment_identity.verification_failure = $_.Exception.Message
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            throw
        }
        if ([string]$validatedEnvironment.fingerprint -cne
            [string]$validatedReadiness.environment_fingerprint) {
            throw "Candidate rehearsal environment fingerprint differs from passing readiness."
        }
        if ($validatedReadiness.PSObject.Properties.Name -contains "predecessor_correction_authorization" -and
            $null -ne $validatedReadiness.predecessor_correction_authorization) {
            if ($validatedReadiness.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
                $null -eq $validatedReadiness.correction_consumption_receipt) {
                throw "Corrected Readiness omits its append-only correction-consumption receipt."
            }
            $consumptionPath = if ([IO.Path]::IsPathRooted([string]$validatedReadiness.correction_consumption_receipt.path)) {
                [IO.Path]::GetFullPath([string]$validatedReadiness.correction_consumption_receipt.path)
            } else {
                [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$validatedReadiness.correction_consumption_receipt.path)))
            }
            if ((Assert-Sprint8AReceiptSidecar -Path $consumptionPath) -cne [string]$validatedReadiness.correction_consumption_receipt.sha256) {
                throw "Corrected Readiness correction-consumption digest is invalid."
            }
        }

        $runtimeContext.readiness = $validatedReadiness
        $runtimeContext.readiness_sha256 = $validatedReadinessSha
        $runtimeContext.environment.fingerprint = [string]$validatedEnvironment.fingerprint
        $runtimeContext.environment.verified = $true
        $runtimeContext.environment.verification_state = "verified"
        $attemptReceipt.mutable_source_identity = $source
        $attemptReceipt.source_identity_verification_state = "verified"
        $attemptReceipt.source_identity_verification_failure = $null
        $attemptReceipt.environment_identity.readiness_sha256 = $validatedReadinessSha
        $attemptReceipt.environment_identity.verification_state = "verified"
        $attemptReceipt.environment_identity.verification_failure = $null
        $attemptReceipt.environment_fingerprint = [string]$validatedEnvironment.fingerprint
        $attemptReceipt.prerequisite_receipts = @([ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
            sha256 = $validatedReadinessSha
        })
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        [ordered]@{
            path = $readinessPath
            sha256 = $validatedReadinessSha
            source = $source
            environment_fingerprint = [string]$validatedEnvironment.fingerprint
        } | ConvertTo-Json -Depth 10
    }
    Invoke-RehearsalLane "formatting" {
        & cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "cargo fmt failed." }
    }
    Invoke-RehearsalLane "workspace-check" {
        & cargo check --workspace --all-features --locked --offline; if ($LASTEXITCODE -ne 0) { throw "cargo check failed." }
    }
    Invoke-RehearsalLane "workspace-clippy" {
        & cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings; if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed." }
    }
    Invoke-RehearsalLane "compose-manifest-schema-contract" {
        # Normalized Compose output contains runtime secrets. Validate it
        # quietly; the acceptance contract inspects only redacted projections.
        & docker compose -f ./deploy/sprint-8a/compose.yaml --profile reference config --quiet
        if ($LASTEXITCODE -ne 0) { throw "Compose normalization failed." }
        Test-Sprint8AAcceptanceContract
    }
    Invoke-RehearsalLane "web-native-wasm-source-boundaries" {
        & ./scripts/check-web-crate-boundaries.ps1; if ($LASTEXITCODE -ne 0) { throw "Package-boundary audit failed." }
    }
    Invoke-RehearsalLane "module-sdk-boundaries" {
        & ./scripts/verify-module-sdk-boundaries.ps1; if ($LASTEXITCODE -ne 0) { throw "Module SDK boundary audit failed." }
    }
    Invoke-RehearsalLane "dashboard-source-boundaries" {
        & ./scripts/verify-sprint-6e-boundaries.ps1; if ($LASTEXITCODE -ne 0) { throw "Dashboard source boundary audit failed." }
    }
    Invoke-RehearsalLane "markdown-links" {
        & ./scripts/verify-markdown-links.ps1; if ($LASTEXITCODE -ne 0) { throw "Markdown-link audit failed." }
    }
    Invoke-RehearsalLane "workspace-tests" {
        & cargo test --workspace --all-features --locked --offline; if ($LASTEXITCODE -ne 0) { throw "Full workspace tests failed." }
    }
    Invoke-RehearsalLane "optimized-resource-reference-timing" {
        & cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture
        if ($LASTEXITCODE -ne 0) { throw "Optimized resource-reference timing proof failed." }
    }
    Invoke-RehearsalLane "components-contract-tests" {
        & cargo test --locked --offline -p tessara-components-contract; if ($LASTEXITCODE -ne 0) { throw "Components contract tests failed." }
    }
    Invoke-RehearsalLane "dashboard-module-tests" {
        & cargo test --locked --offline -p tessara-dashboard-module; if ($LASTEXITCODE -ne 0) { throw "Dashboard module tests failed." }
    }
    Invoke-RehearsalLane "component-conformance-nondisclosure" {
        & cargo test --locked --offline -p tessara-component-module; if ($LASTEXITCODE -ne 0) { throw "Component conformance/nondisclosure tests failed." }
    }
    Invoke-RehearsalLane "module-testkit-conformance" {
        & cargo test --locked --offline -p tessara-module-testkit; if ($LASTEXITCODE -ne 0) { throw "Module testkit conformance failed." }
    }
    Invoke-RehearsalLane "playwright-discovery" {
        & ./scripts/validate-e2e.ps1 -InventoryOnly -EvidencePath $playwrightInventoryEvidence
        if (-not $?) { throw "Exact Playwright acceptance-inventory discovery failed." }
    }
    if ($terminalByName.ContainsKey("attempt-state-prerequisite") -and
        [string]$terminalByName["attempt-state-prerequisite"].state -ceq "passed" -and
        $terminalByName.ContainsKey("validation-readiness-prerequisite") -and
        [string]$terminalByName["validation-readiness-prerequisite"].state -ceq "passed") {
        Publish-Sprint7AEvidence -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; updated_at = [DateTimeOffset]::UtcNow.ToString("o")
            source_identity = $source; source_identity_verification_state = "verified"
            environment_fingerprint = [string]$runtimeContext.environment.fingerprint
            readiness = $runtimeContext.validation_state.readiness
            rehearsal = [ordered]@{
                attempt = $Attempt; state = "executing"
                receipt = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
            }
            correction_transition = $runtimeContext.correction_transition
            preflight_eligible = $false
        }) -OutputPath $statePath -Overwrite | Out-Null
    }
    Invoke-RehearsalLane "source-exact-materialization-no-op" {
        & ./scripts/materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot $attemptRoot -EnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp
        if (-not $?) { throw "Sprint 8A first/no-op materialization failed." }
    }
    Invoke-RehearsalLane "deployed-inventory-navigation-audit" {
        & ./scripts/audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath $inventoryInitial
        if (-not $?) { throw "Initial deployed inventory/navigation audit failed." }
    }
    Invoke-RehearsalLane "deployment-evidence" {
        & ./scripts/run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath $deploymentInitial
        if (-not $?) { throw "Initial source-exact deployment evidence failed." }
    }
    Invoke-RehearsalLane "product-smoke" {
        & ./scripts/smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath $smokeInitial
        if (-not $?) { throw "Initial Sprint 8A product/ownership smoke failed." }
    }
    Invoke-RehearsalLane "playwright-execution" {
        & ./scripts/validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8088" -DeploymentEvidencePath $deploymentInitial -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -EvidencePath $playwrightEvidence -FailureEvidenceDirectory $playwrightFailure
        if (-not $?) { throw "Complete Playwright acceptance rehearsal failed." }
    }
    Invoke-RehearsalLane "component-upgrade-rollback" {
        & ./scripts/run-sprint-8a-component-upgrade.ps1 -OutputPath $upgradeResult
        if (-not $?) { throw "Component upgrade/rollback rehearsal failed." }
    }
    Invoke-RehearsalLane "live-product-diagnostics" {
        & ./scripts/diagnose-sprint-8a-product.ps1 -Attempt $Attempt -EnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) -BaseUrl "http://127.0.0.1:8088" -OutputPath $productDiagnosticEvidence
        if (-not $?) { throw "Live non-acceptance Sprint 8A product diagnostic failed." }
    }
    Invoke-RehearsalLane "failure-containment-successor-health" {
        $reuseSourceExactImages = $terminalByName.ContainsKey("source-exact-materialization-no-op") -and
            [string]$terminalByName["source-exact-materialization-no-op"].state -ceq "passed"
        & ./scripts/run-sprint-8a-failure-containment.ps1 -Attempt $Attempt -EvidenceRoot $attemptRoot -EnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) -OutputPath $failureContainmentResult -AuthorizeDisposableReset -SkipBuild:$reuseSourceExactImages
        if (-not $?) { throw "Induced failure containment and successor materialization failed." }
    }
    Invoke-RehearsalLane "successor-inventory-navigation-audit" {
        & ./scripts/audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath $inventorySuccessor
        if (-not $?) { throw "Successor deployed inventory/navigation audit failed." }
    }
    Invoke-RehearsalLane "successor-deployment-evidence" {
        & ./scripts/run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath $deploymentSuccessor
        if (-not $?) { throw "Successor source-exact deployment evidence failed." }
    }
    Invoke-RehearsalLane "successor-product-smoke" {
        & ./scripts/smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath $smokeSuccessor
        if (-not $?) { throw "Successor Sprint 8A product/ownership smoke failed." }
    }
    Invoke-RehearsalLane "uat-diagnostics" {
        & ./scripts/uat-sprint-8a.ps1 -Attempt $Attempt -EnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) -OutputPath $uatResult `
            -MaterializationLaneReceipt (Join-Path $laneRoot "source-exact-materialization-no-op.json") `
            -InventoryLaneReceipt (Join-Path $laneRoot "successor-inventory-navigation-audit.json") `
            -DeploymentEvidenceLaneReceipt (Join-Path $laneRoot "successor-deployment-evidence.json") `
            -ProductSmokeLaneReceipt (Join-Path $laneRoot "successor-product-smoke.json") `
            -FailureContainmentLaneReceipt (Join-Path $laneRoot "failure-containment-successor-health.json") `
            -UpgradeLaneReceipt (Join-Path $laneRoot "component-upgrade-rollback.json") `
            -ComponentConformanceLaneReceipt (Join-Path $laneRoot "component-conformance-nondisclosure.json") `
            -PlaywrightLaneReceipt (Join-Path $laneRoot "playwright-execution.json") `
            -ManifestContractLaneReceipt (Join-Path $laneRoot "compose-manifest-schema-contract.json") `
            -WebBoundaryLaneReceipt (Join-Path $laneRoot "web-native-wasm-source-boundaries.json") `
            -DashboardBoundaryLaneReceipt (Join-Path $laneRoot "dashboard-source-boundaries.json") `
            -ProductDiagnosticLaneReceipt (Join-Path $laneRoot "live-product-diagnostics.json")
        if (-not $?) { throw "Automated Sprint 8A UAT diagnostics failed." }
    }
    Invoke-RehearsalLane "final-clean-source" {
        $finalSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
        if (($finalSource | ConvertTo-Json -Depth 10 -Compress) -cne ($source | ConvertTo-Json -Depth 10 -Compress)) {
            throw "Candidate rehearsal changed the clean source identity."
        }
        $finalSource | ConvertTo-Json -Depth 10
    }
    Invoke-RehearsalLane "final-environment-identity" {
        $finalEnvironment = Get-Sprint8AEnvironmentContract -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -ProbeDatabases
        if ([string]$finalEnvironment.fingerprint -cne [string]$runtimeContext.environment.fingerprint) {
            throw "Candidate rehearsal changed the canonical environment fingerprint."
        }
        [ordered]@{ environment_fingerprint = $finalEnvironment.fingerprint } | ConvertTo-Json -Depth 10
    }
    Invoke-RehearsalLane "final-successor-health" {
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:8088/health" -Method Get
        if ($null -eq $health) { throw "Final gateway health response is empty." }
        $health | ConvertTo-Json -Depth 10
    }
} finally {
    Pop-Location
}

$endedAt = [DateTimeOffset]::UtcNow
$failedChecks = @($terminalChecks | Where-Object state -CEQ "failed")
$blockedChecks = @($terminalChecks | Where-Object state -CEQ "blocked")
$nestedBlockedChecks = @($terminalChecks | ForEach-Object { @($_.nested_blocked_checks) })
$nestedFailedChecks = @($terminalChecks | ForEach-Object { @($_.nested_failed_checks) })
$passed = $terminalChecks.Count -eq $declaredChecks.Count -and
    $failedChecks.Count -eq 0 -and
    $blockedChecks.Count -eq 0 -and
    $nestedBlockedChecks.Count -eq 0 -and
    $nestedFailedChecks.Count -eq 0
$attemptReceipt.state = if ($passed) { "passed" } else { "failed" }
$attemptReceipt.ended_at = $endedAt.ToString("o")
$attemptReceipt.assertion_count = $terminalChecks.Count
$attemptReceipt.failure_count = $failedChecks.Count
$attemptReceipt.blocked_count = $blockedChecks.Count
$attemptReceipt.nested_blocked_count = $nestedBlockedChecks.Count
$attemptReceipt.nested_failure_count = $nestedFailedChecks.Count
$failureClassifications = @(Get-Sprint8AResultClassifications -Results (@($failedChecks) + @($nestedFailedChecks)))
$attemptReceipt.classification = if ($failureClassifications.Count -eq 1) {
    [string]$failureClassifications[0]
} else {
    $null
}
$attemptReceipt.invalidation_decision = if ($passed) { "none" } else { "candidate freeze forbidden pending the one consolidated correction batch" }
$restorationChecks = @(
    "failure-containment-successor-health",
    "successor-deployment-evidence",
    "successor-product-smoke",
    "final-successor-health"
)
$attemptReceipt.cleanup_restoration = [ordered]@{
    required = $true
    result = if (@($restorationChecks | Where-Object {
        -not $terminalByName.ContainsKey($_) -or [string]$terminalByName[$_].state -cne "passed"
    }).Count -eq 0) { "canonical_successor_healthy" } else { "not_proven" }
}
Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
$attemptSha = Assert-Sprint8AReceiptSidecar -Path $attemptPath
$readinessStateRecord = if ([bool]$runtimeContext.environment.verified) {
    [ordered]@{
        attempt = [int]$runtimeContext.readiness.attempt
        state = "passed"
        receipt = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
        sha256 = [string]$runtimeContext.readiness_sha256
    }
} else {
    [ordered]@{
        attempt = if ($null -ne $runtimeContext.readiness -and
            $runtimeContext.readiness.attempt -is [int]) { [int]$runtimeContext.readiness.attempt } else { 0 }
        state = "invalid"
        receipt = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
        sha256 = $runtimeContext.readiness_sha256
    }
}

if (-not $passed) {
    $harvestRelative = [IO.Path]::GetRelativePath($repoRoot, $harvestPath).Replace("\", "/")
    $harvest = [ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-harvest"; attempt = $Attempt
        authoritative = $false; state = "harvest_complete"; completed_at = $endedAt.ToString("o")
        receipt_path = $harvestRelative; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        attempt_receipt = [ordered]@{ path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/"); sha256 = $attemptSha }
        checks = $terminalChecks
        failed_count = $failedChecks.Count
        blocked_count = $blockedChecks.Count
        nested_blocked_count = $nestedBlockedChecks.Count
        nested_failed_count = $nestedFailedChecks.Count
        passed_count = @($terminalChecks | Where-Object state -CEQ "passed").Count
    }
    Publish-Sprint7AEvidence -Document $harvest -OutputPath $harvestPath | Out-Null
    $harvestSha = Assert-Sprint8AReceiptSidecar -Path $harvestPath
    $laneDefects = @($failedChecks | ForEach-Object {
        [ordered]@{
            id = $null
            classification = [string]$_.classification
            summary = [string]$_.failure_message
            check_names = @([string]$_.name)
            nested_assertion = $null
            raw_evidence = @([ordered]@{
                path = [string]$_.evidence_path
                sha256 = [string]$_.evidence_sha256
            }) + @($_.produced_evidence)
            state = "open"
        }
    })
    $nestedDefects = @($nestedFailedChecks | ForEach-Object {
        [ordered]@{
            id = $null
            classification = [string]$_.classification
            summary = [string]$_.failure_reason
            check_names = @()
            nested_assertion = [string]$_.name
            raw_evidence = @($_.raw_evidence)
            state = "open"
        }
    })
    $defects = @(@($laneDefects) + @($nestedDefects))
    for ($defectIndex = 0; $defectIndex -lt $defects.Count; $defectIndex++) {
        $defects[$defectIndex].id = "8A-R$Attempt-$('{0:d2}' -f ($defectIndex + 1))"
    }
    $batch = [ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-defect-batch"; attempt = $Attempt
        authoritative = $false; batch = 1; state = "open"; generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        harvest_receipt = [ordered]@{ path = $harvestRelative; sha256 = $harvestSha }
        defect_count = $defects.Count; defects = $defects
        blocked_checks = @(
            @($blockedChecks | ForEach-Object {
                [ordered]@{
                    scope = "lane"
                    name = [string]$_.name
                    parent_check = $null
                    dependency_reason = [string]$_.dependency_reason
                }
            }) + @($nestedBlockedChecks | ForEach-Object {
                [ordered]@{
                    scope = "scenario"
                    name = [string]$_.name
                    parent_check = [string]$_.parent_check
                    blocked_by = @($_.blocked_by)
                    dependency_reason = [string]$_.dependency_reason
                }
            })
        )
    }
    Publish-Sprint7AEvidence -Document $batch -OutputPath $batchPath | Out-Null
    if (-not [bool]$runtimeContext.launch_authorized) {
        if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
        throw "Sprint 8A Candidate Rehearsal launch was rejected by the active-attempt/state prerequisite. Safe independent evidence and one consolidated batch were retained, but no correction authorization was issued and validation-state was not overwritten."
    }
    & (Join-Path $PSScriptRoot "test-sprint-validation-harvest.ps1") -AttemptPath $attemptPath -HarvestPath $harvestPath -DefectBatchPath $batchPath -CorrectionAuthorizationPath $correctionAuthorizationPath
    if (-not $?) { throw "Candidate rehearsal harvesting could not authorize the consolidated correction batch." }
    $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $correctionAuthorizationPath
    $correctionTransition = [ordered]@{
        predecessor_rehearsal = [ordered]@{
            attempt = $Attempt
            receipt = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
            sha256 = $attemptSha
            harvest = $harvestRelative
            defect_batch = [IO.Path]::GetRelativePath($repoRoot, $batchPath).Replace("\", "/")
        }
        authorization = [ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $correctionAuthorizationPath).Replace("\", "/")
            sha256 = $authorizationSha
        }
        consumed_by_readiness = $null
    }
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 1; sprint = "sprint-8a"; updated_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        readiness = $readinessStateRecord
        rehearsal = [ordered]@{ attempt = $Attempt; state = "failed"; receipt = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/"); sha256 = $attemptSha; harvest = $harvestRelative; defect_batch = [IO.Path]::GetRelativePath($repoRoot, $batchPath).Replace("\", "/") }
        correction_transition = $correctionTransition
        preflight_eligible = $false
    }) -OutputPath $statePath -Overwrite | Out-Null
    if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
    throw "Sprint 8A Candidate Rehearsal failed $($failedChecks.Count) checks, blocked $($blockedChecks.Count) lanes, and retained $($nestedBlockedChecks.Count) blocked UAT scenarios. One consolidated defect batch is retained at $batchPath."
}

$result = [ordered]@{
    schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal"; attempt = $Attempt
    authoritative = $false; state = "passed"; started_at = $startedAt.ToString("o"); ended_at = $endedAt.ToString("o")
    mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
    prerequisite_receipts = @([ordered]@{ path = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/"); sha256 = [string]$runtimeContext.readiness_sha256 })
    attempt_receipt = [ordered]@{ path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/"); sha256 = $attemptSha }
    checks = $terminalChecks; assertion_count = $terminalChecks.Count; failure_count = 0; blocked_count = 0; nested_blocked_count = 0; nested_failure_count = 0
    classification = $null; invalidation_decision = "none"; cleanup_restoration = $attemptReceipt.cleanup_restoration
}
Publish-Sprint7AEvidence -Document $result -OutputPath $resultPath -Overwrite | Out-Null
$resultSha = Assert-Sprint8AReceiptSidecar -Path $resultPath
Publish-Sprint7AEvidence -Document ([ordered]@{
    schema_version = 1; sprint = "sprint-8a"; updated_at = [DateTimeOffset]::UtcNow.ToString("o")
    source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
    readiness = $readinessStateRecord
    rehearsal = [ordered]@{ attempt = $Attempt; state = "passed"; receipt = [IO.Path]::GetRelativePath($repoRoot, $resultPath).Replace("\", "/"); sha256 = $resultSha }
    correction_transition = $runtimeContext.correction_transition
    preflight_eligible = $true
}) -OutputPath $statePath -Overwrite | Out-Null
if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
Write-Host "Sprint 8A Candidate Rehearsal passed cleanly for source $($source.commit) and environment $($runtimeContext.environment.fingerprint)."

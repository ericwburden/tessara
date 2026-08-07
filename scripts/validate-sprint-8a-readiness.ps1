[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidenceRootPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
    [IO.Path]::GetFullPath($EvidenceRoot)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
}
$attemptPath = Join-Path $evidenceRootPath "attempts/readiness-$Attempt.json"
$resultPath = Join-Path $evidenceRootPath "validation-readiness-result.json"
$statePath = Join-Path $evidenceRootPath "validation-state.json"
$validationLockPath = Join-Path $evidenceRootPath "validation-attempt.lock"
$startSnapshotPath = Join-Path $evidenceRootPath "attempts/readiness-$Attempt-start.json"
$logRoot = Join-Path $evidenceRootPath "readiness-$Attempt"
$environmentPath = Join-Path $logRoot "environment-contract.json"
$deploymentProbePath = Join-Path $logRoot "compose-database-probe.json"
$playwrightInventoryPath = Join-Path $logRoot "playwright-inventory.json"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")

function Open-Sprint8AValidationAttemptLock {
    param([Parameter(Mandatory)][string]$Path)
    [IO.File]::Open($Path, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
}

function Publish-Sprint8AAppendOnlyJsonReceipt {
    param([Parameter(Mandatory)]$Document, [Parameter(Mandatory)][string]$Path)
    $json = $Document | ConvertTo-Json -Depth 100
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes("$json`n")
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
    $sha256 = Get-Sprint8AFileSha256 -Path $Path
    $sidecarBytes = [Text.UTF8Encoding]::new($false).GetBytes("$sha256`n")
    $sidecar = [IO.File]::Open("$Path.sha256", [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $sidecar.Write($sidecarBytes, 0, $sidecarBytes.Length) } finally { $sidecar.Dispose() }
    $sha256
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

function Test-Sprint8AAppendOnlyCorrectionConsumption {
    $path = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-consumption-$([guid]::NewGuid().ToString('N')).json"
    $document = [ordered]@{ schema_version = 1; phase = "candidate-rehearsal-correction-consumption" }
    [void](Publish-Sprint8AAppendOnlyJsonReceipt -Document $document -Path $path)
    $duplicateRejected = $false
    try {
        [void](Publish-Sprint8AAppendOnlyJsonReceipt -Document $document -Path $path)
    } catch [IO.IOException] {
        $duplicateRejected = $true
    }
    if (-not $duplicateRejected) { throw "Append-only correction consumption self-test accepted duplicate use." }
    Remove-Item -LiteralPath $path, "$path.sha256" -Force
}

$declaredChecks = @(
    [ordered]@{ name = "attempt-state-prerequisite"; depends_on = @(); classification = "preflight/setup" },
    [ordered]@{ name = "clean-source"; depends_on = @(); classification = "preflight/setup" },
    [ordered]@{ name = "compose-database-contract"; depends_on = @(); classification = "environment"; evidence_paths = @($deploymentProbePath, "$deploymentProbePath.sha256") },
    [ordered]@{ name = "toolchain"; depends_on = @(); classification = "environment" },
    [ordered]@{ name = "playwright-locked-install"; depends_on = @("attempt-state-prerequisite"); classification = "environment" },
    [ordered]@{ name = "environment-contract"; depends_on = @("toolchain", "playwright-locked-install", "compose-database-contract"); classification = "environment"; evidence_paths = @($environmentPath, "$environmentPath.sha256") },
    [ordered]@{ name = "playwright-discovery"; depends_on = @("playwright-locked-install"); classification = "harness"; evidence_paths = @($playwrightInventoryPath, "$playwrightInventoryPath.sha256") },
    [ordered]@{ name = "compose-and-fixture-contract"; depends_on = @(); classification = "harness" },
    [ordered]@{ name = "runner-parsing"; depends_on = @(); classification = "harness" },
    [ordered]@{ name = "runner-self-tests"; depends_on = @("runner-parsing"); classification = "harness" },
    [ordered]@{ name = "reset-dry-run"; depends_on = @("runner-parsing"); classification = "harness" },
    [ordered]@{ name = "package-boundaries"; depends_on = @(); classification = "product" },
    [ordered]@{ name = "cargo-metadata"; depends_on = @(); classification = "product" },
    [ordered]@{ name = "markdown-links"; depends_on = @(); classification = "product" },
    [ordered]@{ name = "final-clean-source"; depends_on = @("clean-source"); classification = "product" }
)

function Assert-Sprint8AReadinessFailLateGraph {
    param([Parameter(Mandatory)][object[]]$Checks)

    foreach ($independentName in @(
        "attempt-state-prerequisite", "clean-source", "compose-database-contract", "toolchain",
        "compose-and-fixture-contract", "runner-parsing",
        "package-boundaries", "cargo-metadata", "markdown-links"
    )) {
        $independent = @($Checks | Where-Object name -CEQ $independentName)
        if ($independent.Count -ne 1 -or @($independent[0].depends_on).Count -ne 0) {
            throw "Safe Readiness check '$independentName' must remain independent of the active-state guard and sibling failures."
        }
    }
    $composeProbe = @($Checks | Where-Object name -CEQ "compose-database-contract")
    $environmentFinalization = @($Checks | Where-Object name -CEQ "environment-contract")
    if ($composeProbe.Count -ne 1 -or @($composeProbe[0].depends_on).Count -ne 0) {
        throw "Authenticated Compose/six-database readiness must remain an independent fail-late check."
    }
    $expectedFinalizationDependencies = @("compose-database-contract", "playwright-locked-install", "toolchain")
    if ($environmentFinalization.Count -ne 1) {
        throw "Environment fingerprint finalization must be declared exactly once."
    }
    $actualFinalizationDependencies = @($environmentFinalization[0].depends_on | Sort-Object)
    if (
        ($actualFinalizationDependencies | ConvertTo-Json -Compress) -cne
            ($expectedFinalizationDependencies | Sort-Object | ConvertTo-Json -Compress)) {
        throw "Environment fingerprint finalization must depend on the independently harvested deployment probe and required toolchain/npm checks."
    }
}

Assert-Sprint8AReadinessFailLateGraph -Checks $declaredChecks
if ($SelfTest) {
    Test-Sprint8AExclusiveValidationLock
    Test-Sprint8AAppendOnlyCorrectionConsumption
    Test-Sprint8AResultClassificationProjection
    Test-Sprint8AEnvironmentContractComparison
    Test-Sprint8AEvidenceReferenceResolution
    $canonicalSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $canonicalSource | Out-Null
    if ($canonicalSource.GetType().FullName -cne "System.Management.Automation.PSCustomObject") {
        throw "Canonical Sprint 8A source identity helper did not return a PSCustomObject."
    }
    $simulatedState = [ordered]@{
        toolchain = "failed"
        "playwright-locked-install" = "failed"
        "compose-database-contract" = "passed"
        "environment-contract" = "blocked"
    }
    if ([string]$simulatedState."compose-database-contract" -cne "passed" -or
        [string]$simulatedState."environment-contract" -cne "blocked") {
        throw "Readiness fail-late self-test did not retain the independent Compose/six-database probe."
    }
    $localhostBinding = ConvertTo-Sprint8ADatabaseBinding `
        -Name "TEST_ALIAS_A_DATABASE_URL" `
        -Value "postgresql://role:secret@localhost:5432/tessara_test_alias"
    $numericBinding = ConvertTo-Sprint8ADatabaseBinding `
        -Name "TEST_ALIAS_B_DATABASE_URL" `
        -Value "postgresql://role:secret@127.0.0.1:5432/tessara_test_alias"
    $ipv6Binding = ConvertTo-Sprint8ADatabaseBinding `
        -Name "TEST_ALIAS_C_DATABASE_URL" `
        -Value "postgresql://role:secret@[::1]:5432/tessara_test_alias"
    $aliasIdentities = @(
        [string]$localhostBinding.identity,
        [string]$numericBinding.identity,
        [string]$ipv6Binding.identity
    )
    if (@($aliasIdentities | Sort-Object -Unique).Count -ne 1) {
        throw "Readiness database uniqueness can be bypassed with equivalent loopback host aliases."
    }
    Write-Host "Sprint 8A readiness fail-late dependency self-test passed."
    return
}
if ($Attempt -lt 1) { throw "Validation Readiness requires -Attempt with a positive, unused attempt number." }
[IO.Directory]::CreateDirectory((Split-Path -Parent $attemptPath)) | Out-Null
[IO.Directory]::CreateDirectory($logRoot) | Out-Null
if ((Test-Path -LiteralPath $attemptPath) -or (Test-Path -LiteralPath "$attemptPath.sha256") -or
    (Test-Path -LiteralPath $startSnapshotPath) -or (Test-Path -LiteralPath "$startSnapshotPath.sha256")) {
    throw "Readiness attempt $Attempt already exists and cannot be reused."
}

$source = [pscustomobject][ordered]@{
    commit = "0" * 40
    tree = "0" * 40
    dirty = $false
    branch = "unverified"
    acceptance_inventory_sha256 = "0" * 64
    deployment_inputs_sha256 = "0" * 64
}
$sourceIdentityVerificationState = "unverified"
$sourceIdentityVerificationFailure = $null
$environmentVerificationState = "unverified"
$launchAuthorized = $false
$priorState = $null
$correctionTransition = $null
$predecessorCorrectionAuthorization = $null
$correctionConsumptionReceipt = $null
$validationLockHandle = $null
$startedAt = [DateTimeOffset]::UtcNow
$startReceipt = [ordered]@{
    schema_version = 2
    sprint = "sprint-8a"
    phase = "validation-readiness"
    attempt = $Attempt
    authoritative = $false
    state = "preparing"
    assertions_started = $false
    assertions_started_at = $null
    started_at = $startedAt.ToString("o")
    ended_at = $null
    mutable_source_identity = $source
    source_identity_verification_state = $sourceIdentityVerificationState
    source_identity_verification_failure = $sourceIdentityVerificationFailure
    environment_identity = [ordered]@{ verification_state = $environmentVerificationState; path = $null; sha256 = $null }
    environment_fingerprint = "0" * 64
    prerequisite_receipts = @()
    predecessor_correction_authorization = $predecessorCorrectionAuthorization
    correction_consumption_receipt = $correctionConsumptionReceipt
    declared_checks = $declaredChecks
    checks = @()
    assertion_count = 0
    failure_count = 0
    classification = $null
    invalidation_decision = "candidate freeze forbidden until readiness and rehearsal pass"
    cleanup_restoration = [ordered]@{ required = $false; result = "not_applicable" }
}
Publish-Sprint7AEvidence -Document $startReceipt -OutputPath $attemptPath | Out-Null
Publish-Sprint7AEvidence -Document $startReceipt -OutputPath $startSnapshotPath | Out-Null

$checks = [Collections.Generic.List[object]]::new()
$resultByName = @{}
$assertionsStartedAt = $null

function Checkpoint-ReadinessAttempt {
    $startReceipt.state = if (@($checks | Where-Object state -CEQ "failed").Count -gt 0) { "harvesting" } else { "executing" }
    $startReceipt.mutable_source_identity = $source
    $startReceipt.source_identity_verification_state = $sourceIdentityVerificationState
    $startReceipt.source_identity_verification_failure = $sourceIdentityVerificationFailure
    $startReceipt.environment_identity.verification_state = $environmentVerificationState
    $startReceipt.environment_fingerprint = if ($null -eq $environment) { "0" * 64 } else { [string]$environment.fingerprint }
    $startReceipt.predecessor_correction_authorization = $predecessorCorrectionAuthorization
    $startReceipt.correction_consumption_receipt = $correctionConsumptionReceipt
    $startReceipt.checks = @($checks)
    $startReceipt.assertion_count = @($checks | Where-Object assertions_started -EQ $true).Count
    $startReceipt.failure_count = @($checks | Where-Object state -CEQ "failed").Count
    $startReceipt.blocked_count = @($checks | Where-Object state -CEQ "blocked").Count
    Publish-Sprint7AEvidence -Document $startReceipt -OutputPath $attemptPath -Overwrite | Out-Null
}

function Write-ReadinessStateCheckpoint {
    if (-not $launchAuthorized) { return }
    $attemptSha = Assert-Sprint8AReceiptSidecar -Path $attemptPath
    $stateDocument = [ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        updated_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_identity = $source
        source_identity_verification_state = $sourceIdentityVerificationState
        environment_fingerprint = if ($null -eq $environment) { "0" * 64 } else { [string]$environment.fingerprint }
        readiness = [ordered]@{
            attempt = $Attempt
            state = [string]$startReceipt.state
            receipt = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
            sha256 = $attemptSha
        }
        rehearsal = [ordered]@{ state = "ineligible"; reason = "the current readiness attempt supersedes every prior rehearsal" }
        correction_transition = $correctionTransition
        preflight_eligible = $false
    }
    Publish-Sprint7AEvidence -Document $stateDocument -OutputPath $statePath -Overwrite | Out-Null
}

function Invoke-ReadinessFailLateSubchecks {
    param([Parameter(Mandatory)][object[]]$Subchecks)

    $failures = [Collections.Generic.List[string]]::new()
    foreach ($subcheck in $Subchecks) {
        $name = [string]$subcheck.name
        try {
            $action = [scriptblock]$subcheck.action
            & $action
            "subcheck_passed=$name"
        } catch {
            $failures.Add("$name`: $($_.Exception.Message)")
            "subcheck_failed=$name`n$($_ | Out-String)"
        }
    }
    if ($failures.Count -gt 0) {
        throw "Readiness retained $($failures.Count) fail-late subcheck failure(s): $($failures -join ' | ')"
    }
}

function Invoke-ReadinessCheck {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][scriptblock]$Action
    )

    $declaration = @($declaredChecks | Where-Object name -CEQ $Name)
    if ($declaration.Count -ne 1) { throw "Readiness check '$Name' was not declared exactly once." }
    $failedDependencies = @($declaration[0].depends_on | Where-Object {
        -not $resultByName.ContainsKey([string]$_) -or [string]$resultByName[[string]$_].state -cne "passed"
    })
    if ($failedDependencies.Count -gt 0) {
        $reason = "blocked by failed prerequisite(s): $($failedDependencies -join ', ')"
        $blocked = [pscustomobject][ordered]@{
            name = $Name
            depends_on = @($declaration[0].depends_on)
            command = $Command
            started_at = $null
            ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            assertions_started = $false
            assertions_started_at = $null
            duration_ms = 0
            exit_status = $null
            state = "blocked"
            passed = $false
            classification = [string]$declaration[0].classification
            dependency_reason = $reason
            evidence_path = $null
            evidence_sha256 = $null
            produced_evidence = @()
        }
        $checks.Add($blocked)
        $resultByName[$Name] = $blocked
        Checkpoint-ReadinessAttempt
        Write-ReadinessStateCheckpoint
        return
    }

    if ($Name -cne "attempt-state-prerequisite" -and -not [bool]$startReceipt.assertions_started) {
        $script:assertionsStartedAt = [DateTimeOffset]::UtcNow
        $startReceipt.assertions_started = $true
        $startReceipt.assertions_started_at = $script:assertionsStartedAt.ToString("o")
    }
    $start = [DateTimeOffset]::UtcNow
    $checkAssertionsStartedAt = $start
    $log = Join-Path $logRoot "$Name.log"
    [IO.File]::WriteAllText(
        $log,
        "[$($start.ToString('o'))] check_started name=$Name`n",
        [Text.UTF8Encoding]::new($false)
    )
    $passed = $false
    try {
        & $Action *>&1 | ForEach-Object {
            $rendered = ($_ | Out-String).TrimEnd()
            if (-not [string]::IsNullOrEmpty($rendered)) {
                [IO.File]::AppendAllText(
                    $log,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] $rendered`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        if ($declaration[0].Contains("evidence_paths")) {
            foreach ($evidencePath in @($declaration[0].evidence_paths)) {
                if (-not (Test-Path -LiteralPath $evidencePath -PathType Leaf)) {
                    throw "Readiness check '$Name' did not produce required evidence '$evidencePath'."
                }
            }
        }
        $passed = $true
    } catch {
        [IO.File]::AppendAllText(
            $log,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] check_failure`n$($_ | Out-String)`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    $end = [DateTimeOffset]::UtcNow
    [IO.File]::AppendAllText(
        $log,
        "[$($end.ToString('o'))] check_finished state=$(if ($passed) { 'passed' } else { 'failed' })`n",
        [Text.UTF8Encoding]::new($false)
    )
    $producedEvidence = if ($declaration[0].Contains("evidence_paths")) {
        @($declaration[0].evidence_paths | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object {
            [ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, [IO.Path]::GetFullPath($_)).Replace("\", "/")
                sha256 = Get-Sprint8AFileSha256 -Path $_
            }
        })
    } else {
        @()
    }
    $entry = [pscustomobject][ordered]@{
        name = $Name
        depends_on = @($declaration[0].depends_on)
        command = $Command
        started_at = $start.ToString("o")
        ended_at = $end.ToString("o")
        assertions_started = $true
        assertions_started_at = $checkAssertionsStartedAt.ToString("o")
        duration_ms = [math]::Round(($end - $start).TotalMilliseconds)
        exit_status = if ($passed) { 0 } else { 1 }
        state = if ($passed) { "passed" } else { "failed" }
        passed = $passed
        classification = if ($passed) { $null } else { [string]$declaration[0].classification }
        dependency_reason = $null
        evidence_path = [IO.Path]::GetRelativePath($repoRoot, $log).Replace("\", "/")
        evidence_sha256 = Get-Sprint8AFileSha256 -Path $log
        produced_evidence = $producedEvidence
    }
    $checks.Add($entry)
    $resultByName[$Name] = $entry
    Checkpoint-ReadinessAttempt
    Write-ReadinessStateCheckpoint
}

$deploymentProbe = $null
$environment = $null
Push-Location $repoRoot
try {
    Invoke-ReadinessCheck "attempt-state-prerequisite" "validate active-attempt lock and consume one predecessor correction authorization" {
        $script:validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath
        $script:priorState = if (Test-Path -LiteralPath $statePath -PathType Leaf) {
            Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        } else {
            $null
        }
        if ($null -ne $script:priorState) {
            $activeReadiness = $script:priorState.readiness -and
                @("preparing", "executing", "harvesting") -ccontains [string]$script:priorState.readiness.state
            $activeRehearsal = $script:priorState.rehearsal -and
                @("preparing", "executing", "harvesting") -ccontains [string]$script:priorState.rehearsal.state
            if ($activeReadiness -or $activeRehearsal) {
                throw "Validation-state identifies an active attempt; a second readiness attempt is locked out."
            }

            if ([string]$script:priorState.rehearsal.state -ceq "failed") {
                if ($script:priorState.PSObject.Properties.Name -notcontains "correction_transition" -or
                    $null -eq $script:priorState.correction_transition) {
                    throw "A failed predecessor rehearsal requires its exact correction authorization before successor Readiness."
                }
                $transition = $script:priorState.correction_transition
                if ($transition.PSObject.Properties.Name -notcontains "consumed_by_readiness" -or
                    $null -ne $transition.consumed_by_readiness) {
                    throw "The predecessor rehearsal correction authorization was already consumed; duplicate consumption is forbidden."
                }
                $authorizationRef = $transition.authorization
                $resolvedAuthorization = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$authorizationRef.path) `
                    -AllowLegacyAbsolute
                $authorizationPath = [string]$resolvedAuthorization.full_path
                $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $authorizationPath
                if ($authorizationSha -cne [string]$authorizationRef.sha256) {
                    throw "Correction authorization digest differs from the validation-state transition binding."
                }
                $authorization = Get-Content -LiteralPath $authorizationPath -Raw | ConvertFrom-Json
                if ([string]$authorization.phase -cne "candidate-rehearsal-correction-authorization" -or
                    [string]$authorization.state -cne "authorized" -or
                    [string]$authorization.consumption_state -cne "unconsumed" -or
                    [int]$authorization.allowed_successor_count -ne 1 -or
                    [string]$authorization.allowed_successor_phase -cne "validation-readiness" -or
                    [int]$authorization.attempt -ne [int]$script:priorState.rehearsal.attempt -or
                    [string]$authorization.predecessor_attempt_receipt.sha256 -cne [string]$script:priorState.rehearsal.sha256) {
                    throw "Correction authorization does not bind the exact failed predecessor rehearsal and one successor Readiness."
                }
                $authorizedAttempt = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$authorization.predecessor_attempt_receipt.path) `
                    -AllowLegacyAbsolute
                $authorizedHarvest = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$authorization.harvest_receipt.path) `
                    -AllowLegacyAbsolute
                $authorizedBatch = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$authorization.defect_batch.path) `
                    -AllowLegacyAbsolute
                $transitionAttempt = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$transition.predecessor_rehearsal.receipt) `
                    -AllowLegacyAbsolute
                $transitionHarvest = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$transition.predecessor_rehearsal.harvest) `
                    -AllowLegacyAbsolute
                $transitionBatch = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$transition.predecessor_rehearsal.defect_batch) `
                    -AllowLegacyAbsolute
                if ([string]$authorizedAttempt.full_path -cne [string]$transitionAttempt.full_path -or
                    [string]$authorizedHarvest.full_path -cne [string]$transitionHarvest.full_path -or
                    [string]$authorizedBatch.full_path -cne [string]$transitionBatch.full_path -or
                    (Assert-Sprint8AReceiptSidecar -Path ([string]$authorizedAttempt.full_path)) -cne [string]$authorization.predecessor_attempt_receipt.sha256 -or
                    (Assert-Sprint8AReceiptSidecar -Path ([string]$authorizedHarvest.full_path)) -cne [string]$authorization.harvest_receipt.sha256 -or
                    (Assert-Sprint8AReceiptSidecar -Path ([string]$authorizedBatch.full_path)) -cne [string]$authorization.defect_batch.sha256) {
                    throw "Correction authorization prerequisite references do not match the contained failed-attempt transition and sidecars."
                }
                $relativeAuthorizationPath = [string]$resolvedAuthorization.path
                $script:predecessorCorrectionAuthorization = [ordered]@{
                    path = $relativeAuthorizationPath
                    sha256 = $authorizationSha
                    predecessor_rehearsal_attempt = [int]$authorization.attempt
                }
                $consumptionPath = "$authorizationPath.consumption.json"
                $startSnapshotSha = Assert-Sprint8AReceiptSidecar -Path $startSnapshotPath
                $consumptionDocument = [ordered]@{
                    schema_version = 1
                    sprint = "sprint-8a"
                    phase = "candidate-rehearsal-correction-consumption"
                    authoritative = $false
                    state = "consumed"
                    consumed_at = [DateTimeOffset]::UtcNow.ToString("o")
                    authorization = [ordered]@{ path = $relativeAuthorizationPath; sha256 = $authorizationSha }
                    predecessor_rehearsal = $transition.predecessor_rehearsal
                    successor_readiness = [ordered]@{
                        attempt = $Attempt
                        start_receipt = [IO.Path]::GetRelativePath($repoRoot, $startSnapshotPath).Replace("\", "/")
                        start_receipt_sha256 = $startSnapshotSha
                    }
                }
                $consumptionSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $consumptionDocument -Path $consumptionPath
                $relativeConsumptionPath = [IO.Path]::GetRelativePath($repoRoot, $consumptionPath).Replace("\", "/")
                $script:correctionConsumptionReceipt = [ordered]@{ path = $relativeConsumptionPath; sha256 = $consumptionSha }
                $script:correctionTransition = [ordered]@{
                    predecessor_rehearsal = $transition.predecessor_rehearsal
                    authorization = [ordered]@{ path = $relativeAuthorizationPath; sha256 = $authorizationSha }
                    consumed_by_readiness = [ordered]@{
                        attempt = $Attempt
                        receipt = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                        started_at = $startedAt.ToString("o")
                        consumption_receipt = $script:correctionConsumptionReceipt
                    }
                }
            } elseif ($script:priorState.PSObject.Properties.Name -contains "correction_transition") {
                $existingTransition = $script:priorState.correction_transition
                if ($null -ne $existingTransition) {
                    if ($null -ne $existingTransition.consumed_by_readiness) {
                        throw "The predecessor correction authorization was consumed by Readiness attempt $($existingTransition.consumed_by_readiness.attempt); a different Readiness attempt cannot reuse it."
                    }
                    throw "Validation-state contains an unconsumed correction transition without its failed predecessor rehearsal."
                }
            }
        }
        $script:launchAuthorized = $true
        "readiness launch authorized with active-attempt lock"
    }
    Invoke-ReadinessCheck "clean-source" "git status --porcelain=v1 and exact Sprint 8A input digests" {
        try {
            $script:source = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
            $script:sourceIdentityVerificationState = "verified"
            $script:sourceIdentityVerificationFailure = $null
        } catch {
            $script:sourceIdentityVerificationState = "failed"
            $script:sourceIdentityVerificationFailure = $_.Exception.Message
            throw
        }
        if ($script:source.dirty) {
            throw "Readiness requires clean tracked and untracked source."
        }
        $script:source | ConvertTo-Json -Depth 10
    }
    Invoke-ReadinessCheck "compose-database-contract" "quiet Compose normalization and authenticated six-database transaction probes" {
        $script:deploymentProbe = Get-Sprint8ADeploymentEnvironmentProbe -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -ProbeDatabases
        $databaseProbes = @($script:deploymentProbe.contract.databases | Where-Object {
            $null -ne $_.authenticated_probe -and [bool]$_.authenticated_probe.transaction_round_trip
        })
        if ($databaseProbes.Count -ne 6) {
            throw "Authenticated readiness did not retain six exact database transaction probes."
        }
        Publish-Sprint7AEvidence -Document $script:deploymentProbe.contract -OutputPath $deploymentProbePath | Out-Null
        [ordered]@{
            fingerprint = [string]$script:deploymentProbe.fingerprint
            receipt = [IO.Path]::GetRelativePath($repoRoot, $deploymentProbePath).Replace("\", "/")
            receipt_sha256 = Assert-Sprint8AReceiptSidecar -Path $deploymentProbePath
        } | ConvertTo-Json -Depth 5
    }
    Invoke-ReadinessCheck "toolchain" "required shell/Rust/Node/Docker tool availability" {
        Invoke-ReadinessFailLateSubchecks -Subchecks @(
            [pscustomobject]@{ name = "rustc"; action = { Get-Sprint8AToolVersion -Command "rustc" -Arguments @("--version") } }
            [pscustomobject]@{ name = "cargo"; action = { Get-Sprint8AToolVersion -Command "cargo" -Arguments @("--version") } }
            [pscustomobject]@{ name = "docker"; action = { Get-Sprint8AToolVersion -Command "docker" -Arguments @("--version") } }
            [pscustomobject]@{ name = "docker-compose"; action = { Get-Sprint8AToolVersion -Command "docker" -Arguments @("compose", "version") } }
            [pscustomobject]@{ name = "node"; action = { Get-Sprint8AToolVersion -Command "node" -Arguments @("--version") } }
            [pscustomobject]@{ name = "npm"; action = { Get-Sprint8AToolVersion -Command "npm" -Arguments @("--version") } }
        )
    }
    Invoke-ReadinessCheck "playwright-locked-install" "npm ci --prefix end2end" {
        & npm ci --prefix end2end 2>&1
        if ($LASTEXITCODE -ne 0) { throw "Locked Playwright dependency installation failed." }
    }
    Invoke-ReadinessCheck "environment-contract" "finalize canonical environment identity from independent deployment/database and toolchain evidence" {
        try {
            $script:environment = Get-Sprint8AEnvironmentContract -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -DeploymentProbe $script:deploymentProbe
        } catch {
            $script:environmentVerificationState = "failed"
            throw
        }
        Publish-Sprint7AEvidence -Document $script:environment.contract -OutputPath $environmentPath | Out-Null
        $script:environmentVerificationState = "verified"
        $startReceipt.environment_identity.path = [IO.Path]::GetRelativePath($repoRoot, $environmentPath).Replace("\", "/")
        $startReceipt.environment_identity.sha256 = Assert-Sprint8AReceiptSidecar -Path $environmentPath
        [ordered]@{
            fingerprint = [string]$script:environment.fingerprint
            receipt = [IO.Path]::GetRelativePath($repoRoot, $environmentPath).Replace("\", "/")
            receipt_sha256 = [string]$startReceipt.environment_identity.sha256
        } | ConvertTo-Json -Depth 5
    }
    Invoke-ReadinessCheck "playwright-discovery" "validate-e2e.ps1 -InventoryOnly exact acceptance-manifest identity" {
        & ./scripts/validate-e2e.ps1 -InventoryOnly -EvidencePath $playwrightInventoryPath
        if (-not $?) { throw "Exact Playwright acceptance-inventory discovery failed." }
    }
    Invoke-ReadinessCheck "compose-and-fixture-contract" "Test-Sprint7AAcceptanceContract; Test-Sprint8AAcceptanceContract" {
        Invoke-ReadinessFailLateSubchecks -Subchecks @(
            [pscustomobject]@{ name = "sprint-7a-acceptance-contract"; action = { Test-Sprint7AAcceptanceContract } }
            [pscustomobject]@{ name = "sprint-8a-acceptance-contract"; action = { Test-Sprint8AAcceptanceContract } }
        )
        "Compose, fixtures, acceptance mappings, and validation runners agree."
    }
    Invoke-ReadinessCheck "runner-parsing" "PowerShell parser for every Sprint 8A lifecycle runner" {
        $parserFailures = [Collections.Generic.List[string]]::new()
        foreach ($file in @(
            "scripts/sprint-8a-validation-environment.ps1",
            "scripts/materialize-sprint-8a.ps1",
            "scripts/bootstrap-sprint-7a-composition.ps1",
            "scripts/run-sprint-8a-failure-containment.ps1",
            "scripts/prepare-sprint-7a-uat-fixtures.ps1",
            "scripts/sprint-8a-acceptance-contract.ps1",
            "scripts/sprint-8a-lifecycle-chain.ps1",
            "scripts/diagnose-sprint-8a-product.ps1",
            "scripts/smoke-sprint-8a.ps1",
            "scripts/audit-sprint-8a-deployed-inventory.ps1",
            "scripts/uat-sprint-8a.ps1",
            "scripts/test-sprint-validation-harvest.ps1",
            "scripts/verify-sprint-8a-component-upgrade.ps1",
            "scripts/build-sprint-8a-component-rehearsal-baseline.ps1",
            "scripts/run-sprint-8a-deployed-smoke.ps1",
            "scripts/run-sprint-8a-component-upgrade.ps1",
            "scripts/run-sprint-8a-candidate-rehearsal.ps1",
            "scripts/run-sprint-8a-validation-preflight.ps1",
            "scripts/run-sprint-8a-sit.ps1",
            "scripts/run-sprint-8a-formal-uat.ps1",
            "scripts/validate-e2e.ps1",
            "scripts/validate-sprint-8a-readiness.ps1"
        )) {
            $tokens = $null
            $errors = $null
            [void][Management.Automation.Language.Parser]::ParseFile((Resolve-Path $file), [ref]$tokens, [ref]$errors)
            if ($errors.Count) {
                $parserFailures.Add("$file`: $($errors.Message -join '; ')")
            } else {
                "parser_passed=$file"
            }
        }
        if ($parserFailures.Count -gt 0) { throw "Parser failures: $($parserFailures -join ' | ')" }
        "runner parsing passed"
    }
    Invoke-ReadinessCheck "runner-self-tests" "Sprint 8A validation-runner adversarial self-tests" {
        Invoke-ReadinessFailLateSubchecks -Subchecks @(
            [pscustomobject]@{ name = "compose-optional-properties"; action = { Test-Sprint8AComposeServiceProjection } }
            [pscustomobject]@{ name = "result-classification-projection"; action = { Test-Sprint8AResultClassificationProjection } }
            [pscustomobject]@{ name = "environment-comparison"; action = { Test-Sprint8AEnvironmentContractComparison } }
            [pscustomobject]@{ name = "evidence-reference-containment"; action = { Test-Sprint8AEvidenceReferenceResolution } }
            [pscustomobject]@{ name = "composition-bootstrap"; action = { & ./scripts/bootstrap-sprint-7a-composition.ps1 -SelfTest; if (-not $?) { throw "Composition bootstrap self-test failed." } } }
            [pscustomobject]@{ name = "lifecycle-chain"; action = { . ./scripts/sprint-8a-lifecycle-chain.ps1; Test-Sprint8ALifecycleChain; if (-not $?) { throw "Lifecycle-chain self-test failed." } } }
            [pscustomobject]@{ name = "validation-preflight"; action = { & ./scripts/run-sprint-8a-validation-preflight.ps1 -SelfTest; if (-not $?) { throw "Validation preflight self-test failed." } } }
            [pscustomobject]@{ name = "sit"; action = { & ./scripts/run-sprint-8a-sit.ps1 -SelfTest; if (-not $?) { throw "SIT self-test failed." } } }
            [pscustomobject]@{ name = "formal-uat"; action = { & ./scripts/run-sprint-8a-formal-uat.ps1 -SelfTest; if (-not $?) { throw "Formal UAT self-test failed." } } }
            [pscustomobject]@{ name = "smoke"; action = { & ./scripts/smoke-sprint-8a.ps1 -SelfTest; if (-not $?) { throw "Smoke self-test failed." } } }
            [pscustomobject]@{ name = "inventory"; action = { & ./scripts/audit-sprint-8a-deployed-inventory.ps1 -SelfTest; if (-not $?) { throw "Inventory self-test failed." } } }
            [pscustomobject]@{ name = "uat"; action = { & ./scripts/uat-sprint-8a.ps1 -SelfTest; if (-not $?) { throw "UAT self-test failed." } } }
            [pscustomobject]@{ name = "product-diagnostic"; action = { & ./scripts/diagnose-sprint-8a-product.ps1 -SelfTest; if (-not $?) { throw "Product diagnostic self-test failed." } } }
            [pscustomobject]@{ name = "harvest"; action = { & ./scripts/test-sprint-validation-harvest.ps1 -SelfTest; if (-not $?) { throw "Harvest self-test failed." } } }
            [pscustomobject]@{ name = "rehearsal"; action = { & ./scripts/run-sprint-8a-candidate-rehearsal.ps1 -SelfTest; if (-not $?) { throw "Rehearsal self-test failed." } } }
            [pscustomobject]@{ name = "failure-containment"; action = { & ./scripts/run-sprint-8a-failure-containment.ps1 -SelfTest; if (-not $?) { throw "Containment self-test failed." } } }
            [pscustomobject]@{ name = "upgrade-verifier"; action = { & ./scripts/verify-sprint-8a-component-upgrade.ps1 -BaselineMetadataPath "self-test-not-read.json" -SelfTest; if (-not $?) { throw "Upgrade verifier self-test failed." } } }
            [pscustomobject]@{ name = "upgrade-baseline"; action = { & ./scripts/build-sprint-8a-component-rehearsal-baseline.ps1 -SelfTest; if (-not $?) { throw "Upgrade baseline self-test failed." } } }
            [pscustomobject]@{ name = "deployed-smoke"; action = { & ./scripts/run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "target/self-test-deployment.json" -SelfTest; if (-not $?) { throw "Deployed smoke self-test failed." } } }
            [pscustomobject]@{ name = "upgrade-runner"; action = { & ./scripts/run-sprint-8a-component-upgrade.ps1 -SelfTest; if (-not $?) { throw "Upgrade runner self-test failed." } } }
            [pscustomobject]@{ name = "playwright"; action = { & ./scripts/validate-e2e.ps1 -SelfTest; if (-not $?) { throw "Playwright self-test failed." } } }
        )
    }
    Invoke-ReadinessCheck "reset-dry-run" "materialize-sprint-8a.ps1 authorized WhatIf under captured output" {
        $captured = @(& ./scripts/materialize-sprint-8a.ps1 -AuthorizeDisposableReset -WhatIf -Confirm:$false *>&1)
        if (-not $?) { throw "Captured-output reset dry-run failed." }
        "Captured-output reset dry-run returned successfully."
        $captured
    }
    Invoke-ReadinessCheck "package-boundaries" "scripts/check-web-crate-boundaries.ps1" {
        & ./scripts/check-web-crate-boundaries.ps1
        if (-not $?) { throw "Package boundaries failed." }
    }
    Invoke-ReadinessCheck "cargo-metadata" "cargo metadata --locked --offline --no-deps --format-version 1" {
        $metadata = & cargo metadata --locked --offline --no-deps --format-version 1 | ConvertFrom-Json
        if ($LASTEXITCODE -ne 0) { throw "Cargo metadata failed." }
        foreach ($name in @("tessara-component-module", "tessara-components-contract", "tessara-datasets-contract", "tessara-dashboard-placement-renderer")) {
            if (@($metadata.packages.name) -notcontains $name) { throw "Workspace omits '$name'." }
        }
        @($metadata.packages.name | Sort-Object)
    }
    Invoke-ReadinessCheck "markdown-links" "scripts/verify-markdown-links.ps1" {
        & ./scripts/verify-markdown-links.ps1
        if (-not $?) { throw "Markdown links failed." }
    }
    Invoke-ReadinessCheck "final-clean-source" "source identity unchanged after readiness probes" {
        $finalSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
        if ($finalSource.dirty -or
            [string]$finalSource.commit -cne [string]$source.commit -or
            [string]$finalSource.tree -cne [string]$source.tree -or
            [string]$finalSource.acceptance_inventory_sha256 -cne [string]$source.acceptance_inventory_sha256 -or
            [string]$finalSource.deployment_inputs_sha256 -cne [string]$source.deployment_inputs_sha256) {
            throw "Readiness probes changed the clean source or validation inputs."
        }
        $finalSource | ConvertTo-Json -Depth 10
    }
} finally {
    Pop-Location
}

$endedAt = [DateTimeOffset]::UtcNow
$failures = @($checks | Where-Object state -CEQ "failed")
$blocked = @($checks | Where-Object state -CEQ "blocked")
$passed = $failures.Count -eq 0 -and $blocked.Count -eq 0 -and $checks.Count -eq $declaredChecks.Count
$failureClassifications = @(Get-Sprint8AResultClassifications -Results $failures)
$receipt = [ordered]@{
    schema_version = 2
    sprint = "sprint-8a"
    phase = "validation-readiness"
    attempt = $Attempt
    authoritative = $false
    state = if ($passed) { "passed" } else { "failed" }
    assertions_started = [bool]$startReceipt.assertions_started
    assertions_started_at = if ($null -eq $assertionsStartedAt) { $null } else { $assertionsStartedAt.ToString("o") }
    started_at = $startedAt.ToString("o")
    ended_at = $endedAt.ToString("o")
    duration_ms = [math]::Round(($endedAt - $startedAt).TotalMilliseconds)
    mutable_source_identity = $source
    source_identity_verification_state = $sourceIdentityVerificationState
    source_identity_verification_failure = $sourceIdentityVerificationFailure
    environment_identity = if ($null -eq $environment) { [ordered]@{
        verification_state = $environmentVerificationState
        path = $null
        sha256 = $null
    } } else { [ordered]@{
        verification_state = $environmentVerificationState
        path = [IO.Path]::GetRelativePath($repoRoot, $environmentPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $environmentPath
    } }
    environment_fingerprint = if ($null -eq $environment) { "0" * 64 } else { [string]$environment.fingerprint }
    prerequisite_receipts = @()
    predecessor_correction_authorization = $predecessorCorrectionAuthorization
    correction_consumption_receipt = $correctionConsumptionReceipt
    declared_checks = $declaredChecks
    checks = $checks
    assertion_count = @($checks | Where-Object assertions_started -EQ $true).Count
    failure_count = $failures.Count
    blocked_count = $blocked.Count
    classification = if ($failureClassifications.Count -eq 1) { [string]$failureClassifications[0] } else { $null }
    invalidation_decision = if ($passed) { "none" } else { "candidate freeze forbidden; correct one consolidated batch before a new attempt" }
    cleanup_restoration = [ordered]@{ required = $false; result = "not_applicable" }
}
Publish-Sprint7AEvidence -Document $receipt -OutputPath $attemptPath -Overwrite | Out-Null
$attemptSha = Assert-Sprint8AReceiptSidecar -Path $attemptPath
$currentReadinessPath = $attemptPath
$currentReadinessSha = $attemptSha
if ($passed) {
    Publish-Sprint7AEvidence -Document $receipt -OutputPath $resultPath -Overwrite | Out-Null
    $currentReadinessPath = $resultPath
    $currentReadinessSha = Assert-Sprint8AReceiptSidecar -Path $resultPath
}
if ($launchAuthorized) {
    if ($null -ne $correctionTransition -and $null -ne $correctionTransition.consumed_by_readiness) {
        $correctionTransition.consumed_by_readiness.receipt = [IO.Path]::GetRelativePath($repoRoot, $currentReadinessPath).Replace("\", "/")
        $correctionTransition.consumed_by_readiness.receipt_sha256 = $currentReadinessSha
        $correctionTransition.consumed_by_readiness.state = [string]$receipt.state
    }
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        updated_at = $endedAt.ToString("o")
        source_identity = $source
        source_identity_verification_state = $sourceIdentityVerificationState
        environment_fingerprint = if ($null -eq $environment) { "0" * 64 } else { [string]$environment.fingerprint }
        readiness = [ordered]@{
            attempt = $Attempt
            state = [string]$receipt.state
            receipt = [IO.Path]::GetRelativePath($repoRoot, $currentReadinessPath).Replace("\", "/")
            sha256 = $currentReadinessSha
        }
        rehearsal = [ordered]@{ state = "ineligible"; reason = "no rehearsal is eligible until this readiness result passes" }
        correction_transition = $correctionTransition
        preflight_eligible = $false
    }) -OutputPath $statePath -Overwrite | Out-Null
}

if (-not $passed) {
    if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
    throw "Sprint 8A readiness failed $($failures.Count) checks and blocked $($blocked.Count); inspect $attemptPath."
}
if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
Write-Host "Sprint 8A Validation Readiness passed for source $($source.commit) and environment $($environment.fingerprint)."

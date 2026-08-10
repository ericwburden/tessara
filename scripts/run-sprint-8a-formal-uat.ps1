[CmdletBinding()]
param(
    [ValidateSet("Start", "Finalize")][string]$Stage = "Start",
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$PreflightReceipt = "artifacts/sprint-8a-closeout/preflight-result.json",
    [string]$CandidateReceipt = "artifacts/sprint-8a-closeout/candidate.json",
    [string]$SitReceipt = "artifacts/sprint-8a-closeout/sit-result.json",
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [string]$ManualReceiptDirectory,
    [string]$OutputPath = "artifacts/sprint-8a-closeout/uat-result.json",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [switch]$AuthorizeDisposableReset,
    [switch]$AuthorizeUatHarnessOnlySourceAdvance,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-8a-lifecycle-chain.ps1")

function Get-Sprint8AFormalUatReference {
    param([Parameter(Mandatory)][string]$Path)

    $referencePath = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetRelativePath($repoRoot, [IO.Path]::GetFullPath($Path)).Replace("\", "/")
    } else { $Path }
    $resolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $referencePath
    [pscustomobject][ordered]@{
        path = [string]$resolved.path
        sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
        full_path = [string]$resolved.full_path
    }
}

function Get-Sprint8AFormalUatPriorAttempts {
    param([Parameter(Mandatory)][string]$CurrentAttemptPath)

    $attemptDirectory = Join-Path $evidenceRootPath "attempts"
    if (-not (Test-Path -LiteralPath $attemptDirectory -PathType Container)) { return @() }
    @(
        Get-ChildItem -LiteralPath $attemptDirectory -File -Filter "uat-*.json" | Where-Object {
            $_.Name -match '^uat-(\d+)\.json$' -and
                [IO.Path]::GetFullPath($_.FullName) -cne [IO.Path]::GetFullPath($CurrentAttemptPath)
        } | ForEach-Object {
            $sha256 = Assert-Sprint8AReceiptSidecar -Path $_.FullName
            $receipt = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
            if (($receipt.schema_version -isnot [int] -and $receipt.schema_version -isnot [long]) -or
                [int]$receipt.schema_version -ne 1 -or
                [string]$receipt.sprint -cne "sprint-8a" -or
                [string]$receipt.phase -cne "uat" -or
                ($receipt.attempt -isnot [int] -and $receipt.attempt -isnot [long])) {
                throw "Prior formal UAT attempt '$($_.FullName)' is malformed."
            }
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $_.FullName).Replace("\", "/")
                sha256 = $sha256
                receipt = $receipt
            }
        }
    )
}

function Test-Sprint8AFormalUatHarnessChangedPaths {
    param([Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ChangedPaths)

    $allowedPaths = @(
        "scripts/run-sprint-8a-formal-uat.ps1",
        "scripts/sprint-8a-lifecycle-chain.ps1",
        "scripts/validate-resource-reference-nondisclosure.ps1",
        "docs/sprints/sprint-8a-verification.md"
    )
    $ChangedPaths.Count -gt 0 -and
        @($ChangedPaths | Where-Object { $_ -cnotin $allowedPaths }).Count -eq 0
}

function ConvertTo-Sprint8AFormalUatTerminalTimestamp {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    (ConvertTo-Sprint8ADateTimeOffset -Value $Value -Label $Label).ToString("o")
}

function Get-Sprint8AFormalUatFailureClassifications {
    param([Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Failures)

    @($Failures | ForEach-Object { [string]$_.classification } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        Sort-Object -Unique)
}

function Get-Sprint8AFormalUatFailureMessages {
    param([Parameter(Mandatory)]$Receipt)

    if (@($Receipt.PSObject.Properties | ForEach-Object { $_.Name }) -contains "failure_batch" -and
        $null -ne $Receipt.failure_batch) {
        $Receipt.failure_batch.defects | ForEach-Object { [string]$_.message }
    }
}

function Get-Sprint8AFormalUatSourceContext {
    param(
        [Parameter(Mandatory)]$CurrentSource,
        [Parameter(Mandatory)]$CandidateSource,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [switch]$AllowHarnessOnlySourceAdvance
    )

    Assert-Sprint8ASourceIdentityObject -Source $CurrentSource -RequireClean | Out-Null
    Assert-Sprint8ASourceIdentityObject -Source $CandidateSource -RequireClean | Out-Null
    if (Test-Sprint8ASourceIdentityMatch -Expected $CandidateSource -Actual $CurrentSource) {
        return [pscustomobject][ordered]@{
            candidate_source_identity = $CandidateSource
            harness_source_identity = $CurrentSource
            harness_only_source_advance = $null
        }
    }
    if (-not $AllowHarnessOnlySourceAdvance) {
        throw "Formal UAT source differs from the frozen candidate."
    }

    & git -C $RepositoryRoot merge-base --is-ancestor ([string]$CandidateSource.commit) ([string]$CurrentSource.commit)
    $ancestorExit = $LASTEXITCODE
    $changedPaths = @(& git -C $RepositoryRoot diff --name-only "$([string]$CandidateSource.commit)..$([string]$CurrentSource.commit)" |
        ForEach-Object { $_.Replace("\", "/") })
    $diffExit = $LASTEXITCODE
    if ($ancestorExit -ne 0 -or $diffExit -ne 0 -or
        -not (Test-Sprint8AFormalUatHarnessChangedPaths -ChangedPaths $changedPaths)) {
        throw "Authorized formal UAT harness advance contains a path outside the exact UAT-runner correction set."
    }

    [pscustomobject][ordered]@{
        candidate_source_identity = $CandidateSource
        harness_source_identity = $CurrentSource
        harness_only_source_advance = [pscustomobject][ordered]@{
            authorization = "user_directed_impact_scoped_validation"
            candidate_source_commit = [string]$CandidateSource.commit
            uat_harness_source_commit = [string]$CurrentSource.commit
            exact_changed_paths = $changedPaths
            product_test_fixture_deployment_changes = $false
            upstream_gates_affected = $false
        }
    }
}

function New-Sprint8AFormalUatAttemptReceipt {
    param([Parameter(Mandatory)][ValidateRange(1, 9999)][int]$AttemptNumber)

    $placeholderSource = [pscustomobject][ordered]@{
        commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
        acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
    }
    [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "uat"; attempt = $AttemptNumber
        authoritative = $false; state = "preparing"; assertions_started = $false
        stage = "prerequisites"
        started_at = [DateTimeOffset]::UtcNow.ToString("o"); ended_at = $null; duration_ms = 0L
        state_history = @([pscustomobject][ordered]@{
            state = "preparing"; stage = "prerequisites"; at = [DateTimeOffset]::UtcNow.ToString("o")
        })
        source_identity = $placeholderSource; source_verification_state = "unverified"
        candidate_source_identity = $placeholderSource
        uat_harness_only_source_advance = $null
        environment_fingerprint = "0" * 64; candidate_fingerprint = "0" * 64
        endpoints = $null
        prerequisite_receipts = @()
        declared_checks = @(
            [pscustomobject][ordered]@{ name = "authenticated-prerequisites"; depends_on = @() },
            [pscustomobject][ordered]@{ name = "scripted-inventory"; depends_on = @("authenticated-prerequisites") },
            [pscustomobject][ordered]@{ name = "scripted-smoke"; depends_on = @("authenticated-prerequisites") },
            [pscustomobject][ordered]@{ name = "post-scripted-identity"; depends_on = @("authenticated-prerequisites") }
        ) + @(Get-Sprint8AManualUatScenarioNames | ForEach-Object {
            [pscustomobject][ordered]@{ name = $_; depends_on = @("authenticated-prerequisites", "post-scripted-identity") }
        })
        checks = @(); assertion_count = 0; failure_count = 0; blocked_count = 0
        classification = $null; failure_batch = $null
        scripted_completed_at = $null; scripted_defects = @()
        manual_scenarios_pending = @(Get-Sprint8AManualUatScenarioNames)
        restoration_check = $null
        cleanup_restoration = [pscustomobject]@{ required = $true; result = "pending"; evidence = @() }
    }
}

function Test-Sprint8AFormalUatAttemptInvalidatesCandidate {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint
    )

    [string]$Receipt.state -in @("failed", "blocked") -and
        [string]$Receipt.candidate_fingerprint -ceq $CandidateFingerprint -and
        [string]$Receipt.classification -in @("product", "product-decision")
}

function Set-Sprint8AFormalUatAttemptState {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][ValidateSet("preparing", "executing", "finalizing", "passed", "failed", "blocked")][string]$State,
        [Parameter(Mandatory)][string]$Stage
    )

    $currentState = [string]$Receipt.state
    $currentStage = [string]$Receipt.stage
    $currentTerminal = $currentState -in @("passed", "failed", "blocked")
    $normalTransitions = @{
        "preparing/prerequisites" = @("executing/scripted")
        "executing/scripted" = @("executing/manual", "executing/diagnostic-manual")
        "executing/manual" = @("finalizing/manual-receipt-validation")
        "executing/diagnostic-manual" = @("finalizing/manual-receipt-validation")
        "finalizing/manual-receipt-validation" = @("finalizing/canonical-restoration")
        "finalizing/canonical-restoration" = @("passed/result-committed")
    }
    $target = "$State/$Stage"
    $current = "$currentState/$currentStage"
    $allowed = if ($normalTransitions.ContainsKey($current)) { @($normalTransitions[$current]) } else { @() }
    $terminalDispositionAllowed = $State -in @("failed", "blocked") -and
        ($Stage -ceq $currentStage -or
            ($current -ceq "finalizing/canonical-restoration" -and $Stage -ceq "diagnostic-harvest-complete"))
    if ($currentTerminal -or
        ($State -in @("failed", "blocked") -and -not $terminalDispositionAllowed) -or
        ($State -notin @("failed", "blocked") -and $allowed -cnotcontains $target)) {
        throw "Illegal formal UAT attempt transition '$current' -> '$target'."
    }
    $Receipt.state = $State
    $Receipt.stage = $Stage
    $history = @($Receipt.state_history)
    $Receipt.state_history = @($history) + @([pscustomobject][ordered]@{
        state = $State
        stage = $Stage
        at = [DateTimeOffset]::UtcNow.ToString("o")
    })
}

function Resolve-Sprint8AFormalUatFailureClassification {
    param(
        [Parameter(Mandatory)][ValidateSet("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization", "product-decision")][string]$DefaultClassification,
        [AllowNull()]$ErrorRecord,
        [AllowEmptyCollection()][string[]]$EvidencePaths = @()
    )

    $allowed = @("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization", "product-decision")
    if ($null -ne $ErrorRecord -and $null -ne $ErrorRecord.Exception.Data["Sprint8AClassification"] -and
        $allowed -ccontains [string]$ErrorRecord.Exception.Data["Sprint8AClassification"]) {
        return [pscustomobject][ordered]@{
            classification = [string]$ErrorRecord.Exception.Data["Sprint8AClassification"]
            source = "structured_exception"
        }
    }
    foreach ($path in @($EvidencePaths)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or [IO.Path]::GetExtension($path) -cne ".json") {
            continue
        }
        try {
            $document = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
            $candidates = [Collections.Generic.List[object]]::new()
            if ($document.PSObject.Properties.Name -contains "classification") {
                $candidates.Add($document.classification)
            }
            foreach ($containerName in @("failure", "details", "result")) {
                if ($document.PSObject.Properties.Name -notcontains $containerName) { continue }
                $container = $document.$containerName
                if ($null -ne $container -and $container.PSObject.Properties.Name -contains "classification") {
                    $candidates.Add($container.classification)
                }
            }
            foreach ($candidate in @($candidates)) {
                if ($allowed -ccontains [string]$candidate) {
                    return [pscustomobject][ordered]@{
                        classification = [string]$candidate
                        source = "structured_evidence"
                    }
                }
            }
        } catch {
            # A malformed evidence document remains attached to the failed check; it cannot override classification.
        }
    }
    $message = if ($null -eq $ErrorRecord) { "" } else { [string]$ErrorRecord.Exception.Message }
    if ($message -match '(?i)(sha-?256 .* (differs|does not match)|stale|evidence digest .* (stale|does not match)|manifest .* (differs|malformed)|publication|canonical result .* differs)') {
        return [pscustomobject][ordered]@{ classification = "evidence-finalization"; source = "failure_detail" }
    }
    [pscustomobject][ordered]@{ classification = $DefaultClassification; source = "declared_check_category" }
}

function Get-Sprint8AFormalUatScriptedDisposition {
    param(
        [AllowEmptyCollection()][object[]]$Defects = @(),
        [AllowEmptyCollection()][object[]]$ProductDecisions = @()
    )

    $classifications = @($Defects | ForEach-Object { [string]$_.classification } | Sort-Object -Unique)
    [pscustomobject][ordered]@{
        terminal_state = if ($ProductDecisions.Count -gt 0 -and $Defects.Count -eq 0) { "blocked" } elseif ($Defects.Count -gt 0) { "failed" } else { "passed" }
        defect_count = $Defects.Count
        blocked_decision_count = $ProductDecisions.Count
        classification = if ($Defects.Count -eq 0 -and $ProductDecisions.Count -gt 0) {
            "product-decision"
        } elseif ($classifications.Count -eq 1) {
            [string]$classifications[0]
        } else { $null }
    }
}

function Initialize-Sprint8AFormalUatAttemptDirectories {
    param(
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)][string]$AttemptPath,
        [Parameter(Mandatory)][string]$LogRoot
    )

    [IO.Directory]::CreateDirectory($AttemptRoot) | Out-Null
    [IO.Directory]::CreateDirectory((Split-Path -Parent $AttemptPath)) | Out-Null
    [IO.Directory]::CreateDirectory($LogRoot) | Out-Null
}

function Merge-Sprint8AFormalUatFailureBatch {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [AllowEmptyCollection()][object[]]$Defects = @(),
        [AllowEmptyCollection()][object[]]$BlockedChecks = @()
    )

    $dedupeEvidence = {
        param([AllowEmptyCollection()][object[]]$Evidence = @())

        $evidenceMap = [ordered]@{}
        foreach ($reference in @($Evidence)) {
            if ($null -eq $reference) { continue }
            $key = "$([string]$reference.path)`u{001f}$([string]$reference.sha256)"
            if (-not $evidenceMap.Contains($key)) { $evidenceMap[$key] = $reference }
        }
        @($evidenceMap.Values)
    }
    $defectMap = [ordered]@{}
    $mergeDefect = {
        param([AllowNull()]$Defect)

        if ($null -eq $Defect) { return }
        $check = if ($Defect.PSObject.Properties.Name -contains "check") {
            [string]$Defect.check
        } elseif ($Defect.PSObject.Properties.Name -contains "name") {
            [string]$Defect.name
        } else { "unidentified-failure" }
        $message = if ($Defect.PSObject.Properties.Name -contains "message") {
            [string]$Defect.message
        } elseif ($Defect.PSObject.Properties.Name -contains "failure_message") {
            [string]$Defect.failure_message
        } else { "" }
        $classification = if ($Defect.PSObject.Properties.Name -contains "classification") {
            [string]$Defect.classification
        } else { "" }
        $classificationSource = if ($Defect.PSObject.Properties.Name -contains "classification_source") {
            [string]$Defect.classification_source
        } else { "" }
        $evidence = if ($Defect.PSObject.Properties.Name -contains "evidence") { @($Defect.evidence) } else { @() }
        $key = @($check, $classification, $classificationSource, $message) -join "`u{001f}"
        if ($defectMap.Contains($key)) {
            $defectMap[$key].evidence = & $dedupeEvidence -Evidence (@($defectMap[$key].evidence) + $evidence)
            return
        }
        $defectMap[$key] = [pscustomobject][ordered]@{
            check = $check
            classification = if ([string]::IsNullOrWhiteSpace($classification)) { $null } else { $classification }
            classification_source = if ([string]::IsNullOrWhiteSpace($classificationSource)) { $null } else { $classificationSource }
            message = $message
            evidence = & $dedupeEvidence -Evidence $evidence
        }
    }
    $blockedMap = [ordered]@{}
    $mergeBlocked = {
        param([AllowNull()]$BlockedCheck)

        if ($null -eq $BlockedCheck) { return }
        $check = if ($BlockedCheck.PSObject.Properties.Name -contains "check") {
            [string]$BlockedCheck.check
        } elseif ($BlockedCheck.PSObject.Properties.Name -contains "name") {
            [string]$BlockedCheck.name
        } else { "unidentified-blocked-check" }
        $dependencyReason = if ($BlockedCheck.PSObject.Properties.Name -contains "dependency_reason") {
            [string]$BlockedCheck.dependency_reason
        } elseif ($BlockedCheck.PSObject.Properties.Name -contains "blocked_reason") {
            [string]$BlockedCheck.blocked_reason
        } else { "" }
        $classification = if ($BlockedCheck.PSObject.Properties.Name -contains "classification") {
            [string]$BlockedCheck.classification
        } else { "" }
        $classificationSource = if ($BlockedCheck.PSObject.Properties.Name -contains "classification_source") {
            [string]$BlockedCheck.classification_source
        } else { "" }
        $evidence = if ($BlockedCheck.PSObject.Properties.Name -contains "evidence") { @($BlockedCheck.evidence) } else { @() }
        $key = @($check, $dependencyReason, $classification, $classificationSource) -join "`u{001f}"
        if ($blockedMap.Contains($key)) {
            $blockedMap[$key].evidence = & $dedupeEvidence -Evidence (@($blockedMap[$key].evidence) + $evidence)
            return
        }
        $blockedMap[$key] = [pscustomobject][ordered]@{
            check = $check
            dependency_reason = $dependencyReason
            classification = if ([string]::IsNullOrWhiteSpace($classification)) { $null } else { $classification }
            classification_source = if ([string]::IsNullOrWhiteSpace($classificationSource)) { $null } else { $classificationSource }
            evidence = & $dedupeEvidence -Evidence $evidence
        }
    }
    if ($AttemptReceipt.PSObject.Properties.Name -contains "scripted_defects") {
        foreach ($defect in @($AttemptReceipt.scripted_defects)) { & $mergeDefect -Defect $defect }
    }
    if ($AttemptReceipt.PSObject.Properties.Name -contains "failure_batch" -and $null -ne $AttemptReceipt.failure_batch) {
        if ($AttemptReceipt.failure_batch.PSObject.Properties.Name -contains "defects") {
            foreach ($defect in @($AttemptReceipt.failure_batch.defects)) { & $mergeDefect -Defect $defect }
        }
        if ($AttemptReceipt.failure_batch.PSObject.Properties.Name -contains "blocked_checks") {
            foreach ($blocked in @($AttemptReceipt.failure_batch.blocked_checks)) { & $mergeBlocked -BlockedCheck $blocked }
        }
    }
    foreach ($defect in @($Defects)) { & $mergeDefect -Defect $defect }
    foreach ($blocked in @($BlockedChecks)) { & $mergeBlocked -BlockedCheck $blocked }
    $mergedDefects = @($defectMap.Values)
    $mergedBlocked = @($blockedMap.Values)
    $AttemptReceipt | Add-Member -Force -NotePropertyName failure_count -NotePropertyValue $mergedDefects.Count
    $AttemptReceipt | Add-Member -Force -NotePropertyName blocked_count -NotePropertyValue $mergedBlocked.Count
    $classifications = @($mergedDefects | ForEach-Object { [string]$_.classification } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Sort-Object -Unique)
    $AttemptReceipt | Add-Member -Force -NotePropertyName classification -NotePropertyValue $(
        if ($classifications.Count -eq 1) { [string]$classifications[0] } else { $null }
    )
    $AttemptReceipt | Add-Member -Force -NotePropertyName failure_batch -NotePropertyValue ([pscustomobject][ordered]@{
        defect_count = $mergedDefects.Count
        blocked_check_count = $mergedBlocked.Count
        defects = $mergedDefects
        blocked_checks = $mergedBlocked
    })
    $AttemptReceipt.failure_batch
}

function Add-Sprint8AFormalUatEvidenceFinalizationFailure {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$Failure,
        [Parameter(Mandatory)][string]$Context
    )

    $entry = [pscustomobject][ordered]@{
        context = $Context
        occurred_at = if ($Failure.PSObject.Properties.Name -contains "occurred_at") {
            [string]$Failure.occurred_at
        } else { [DateTimeOffset]::UtcNow.ToString("o") }
        classification = [string]$Failure.classification
        classification_source = [string]$Failure.classification_source
        message = [string]$Failure.message
        evidence = @($Failure.evidence)
    }
    $historyProperty = $AttemptReceipt.PSObject.Properties["evidence_finalization_failure_history"]
    $history = @(
        if ($null -ne $historyProperty) {
            $AttemptReceipt.evidence_finalization_failure_history
        }
    )
    $entryEvidenceKey = @($entry.evidence | ForEach-Object { "$([string]$_.path)`u{001f}$([string]$_.sha256)" }) -join "`u{001e}"
    $duplicate = @($history | Where-Object {
        [string]$_.context -ceq $Context -and
            (@($_.evidence | ForEach-Object { "$([string]$_.path)`u{001f}$([string]$_.sha256)" }) -join "`u{001e}") -ceq $entryEvidenceKey
    }).Count -gt 0
    if (-not $duplicate) { $history += $entry }
    $AttemptReceipt | Add-Member `
        -Force `
        -NotePropertyName evidence_finalization_failure_history `
        -NotePropertyValue @($history)
    $entry
}

function Add-Sprint8AFormalUatManifestUpdateFailure {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$Failure
    )

    $historyProperty = $AttemptReceipt.PSObject.Properties["manifest_update_failure_history"]
    $history = @(
        if ($null -ne $historyProperty) { $historyProperty.Value }
    )
    $evidenceKey = @($Failure.evidence | ForEach-Object { "$([string]$_.path)`u{001f}$([string]$_.sha256)" }) -join "`u{001e}"
    if (@($history | Where-Object {
        (@($_.evidence | ForEach-Object { "$([string]$_.path)`u{001f}$([string]$_.sha256)" }) -join "`u{001e}") -ceq $evidenceKey
    }).Count -eq 0) {
        $history += $Failure
    }
    $AttemptReceipt | Add-Member -Force -NotePropertyName manifest_update_failure_history -NotePropertyValue @($history)
    $AttemptReceipt | Add-Member -Force -NotePropertyName manifest_update_failure -NotePropertyValue $Failure
    $Failure
}

function Add-Sprint8AFormalUatFinalizationRetryFailure {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$Failure
    )

    $retryProperty = $AttemptReceipt.PSObject.Properties["finalization_retry"]
    if ($null -eq $retryProperty -or $null -eq $retryProperty.Value) {
        $AttemptReceipt | Add-Member -Force -NotePropertyName finalization_retry -NotePropertyValue ([pscustomobject][ordered]@{
            eligible = $false
            failure_history = @()
        })
    } elseif ($AttemptReceipt.finalization_retry.PSObject.Properties.Name -notcontains "eligible") {
        $AttemptReceipt.finalization_retry | Add-Member -NotePropertyName eligible -NotePropertyValue $false
    }
    $history = @(
        if ($AttemptReceipt.finalization_retry.PSObject.Properties.Name -contains "failure_history") {
            $AttemptReceipt.finalization_retry.failure_history
        }
    )
    $evidenceKey = @($Failure.evidence | ForEach-Object { "$([string]$_.path)`u{001f}$([string]$_.sha256)" }) -join "`u{001e}"
    if (@($history | Where-Object {
        (@($_.evidence | ForEach-Object { "$([string]$_.path)`u{001f}$([string]$_.sha256)" }) -join "`u{001e}") -ceq $evidenceKey
    }).Count -eq 0) {
        $history += $Failure
    }
    $AttemptReceipt.finalization_retry | Add-Member -Force -NotePropertyName failure_history -NotePropertyValue @($history)
    $AttemptReceipt.finalization_retry | Add-Member -Force -NotePropertyName last_failure -NotePropertyValue $Failure
    Add-Sprint8AFormalUatEvidenceFinalizationFailure `
        -AttemptReceipt $AttemptReceipt `
        -Failure $Failure `
        -Context "finalization-only-retry" | Out-Null
    $Failure
}

function Get-Sprint8AFormalUatEvidenceReferences {
    param(
        [Parameter(Mandatory)][object[]]$Evidence,
        [Parameter(Mandatory)][string]$Scenario
    )

    @($Evidence | ForEach-Object {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$_.path)
        if ((Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)) -cne [string]$_.sha256) {
            throw "Manual UAT scenario '$Scenario' evidence digest is stale for '$($_.path)'."
        }
        $reference = [pscustomobject][ordered]@{
            path = [string]$resolved.path
            sha256 = [string]$_.sha256
        }
        if ($_.PSObject.Properties.Name -contains "step") {
            $reference | Add-Member -NotePropertyName step -NotePropertyValue ([int]$_.step)
        }
        if ($_.PSObject.Properties.Name -contains "kind") {
            $reference | Add-Member -NotePropertyName kind -NotePropertyValue ([string]$_.kind)
        }
        $reference
    })
}

function Publish-Sprint8AFormalUatResultCommit {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$ResultReference,
        [Parameter(Mandatory)][string]$Path
    )

    $document = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.uat-result-content-commit"
        authoritative = $false
        attempt = [int]$AttemptReceipt.attempt
        source_identity = $AttemptReceipt.source_identity
        environment_fingerprint = [string]$AttemptReceipt.environment_fingerprint
        candidate_fingerprint = [string]$AttemptReceipt.candidate_fingerprint
        result = [pscustomobject][ordered]@{
            path = [string]$ResultReference.path
            sha256 = [string]$ResultReference.sha256
            receipt = $ResultReference.receipt
        }
    }
    Publish-Sprint7AEvidence -Document $document -OutputPath $Path | Out-Null
    [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $Path).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $Path
    }
}

function Assert-Sprint8AFormalUatResultCommit {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)][string]$CommitPath
    )

    $commitReference = Get-Sprint8AFormalUatReference -Path $CommitPath
    if ([string]$commitReference.sha256 -cne [string]$AttemptReceipt.result_commit_artifact.sha256) {
        throw "Formal UAT result content-commit sidecar differs from the attempt receipt."
    }
    $commit = Get-Content -LiteralPath ([string]$commitReference.full_path) -Raw | ConvertFrom-Json
    if (($commit.schema_version -isnot [int] -and $commit.schema_version -isnot [long]) -or
        [int]$commit.schema_version -ne 1 -or
        [string]$commit.sprint -cne "sprint-8a" -or
        [string]$commit.contract -cne "tessara.sprint-8a.uat-result-content-commit" -or
        $commit.authoritative -isnot [bool] -or [bool]$commit.authoritative -or
        [int]$commit.attempt -ne [int]$AttemptReceipt.attempt -or
        ($commit.source_identity | ConvertTo-Json -Depth 20 -Compress) -cne
            ($AttemptReceipt.source_identity | ConvertTo-Json -Depth 20 -Compress) -or
        [string]$commit.environment_fingerprint -cne [string]$AttemptReceipt.environment_fingerprint -or
        [string]$commit.candidate_fingerprint -cne [string]$AttemptReceipt.candidate_fingerprint -or
        [string]$commit.result.path -cne [string]$AttemptReceipt.uat_result_commit.path -or
        [string]$commit.result.sha256 -cne [string]$AttemptReceipt.uat_result_commit.sha256) {
        throw "Formal UAT result content-commit is malformed or bound to another attempt."
    }
    $computed = Get-Sprint8AStringSha256 -Text (($commit.result.receipt | ConvertTo-Json -Depth 30) + "`n")
    if ($computed -cne [string]$commit.result.sha256) {
        throw "Formal UAT result content-commit receipt differs from its canonical SHA-256."
    }
    [pscustomobject][ordered]@{
        reference = $commitReference
        document = $commit
    }
}

function Publish-Sprint8AFormalUatFinalizationFailure {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$ErrorRecord,
        [Parameter(Mandatory)][string]$Directory,
        [ValidateSet("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization", "product-decision")]
        [string]$Classification = "evidence-finalization",
        [string]$ClassificationSource = "final_publication_boundary"
    )

    $now = [DateTimeOffset]::UtcNow
    $failureIdentity = "{0}-{1}" -f $now.ToUnixTimeMilliseconds(), ([guid]::NewGuid().ToString("N").Substring(0, 8))
    $rawPath = Join-Path $Directory "evidence-finalization-$failureIdentity.log"
    [IO.File]::WriteAllText(
        $rawPath,
        "[$($now.ToString('o'))] $($ErrorRecord | Out-String)",
        [Text.UTF8Encoding]::new($false)
    )
    $rawReference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $rawPath).Replace("\", "/")
        sha256 = Get-Sprint8AFileSha256 -Path $rawPath
    }
    $path = Join-Path $Directory "evidence-finalization-$failureIdentity.json"
    $document = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "uat-evidence-finalization"
        authoritative = $false
        state = "failed"
        attempt = [int]$AttemptReceipt.attempt
        source_identity = $AttemptReceipt.source_identity
        environment_fingerprint = [string]$AttemptReceipt.environment_fingerprint
        candidate_fingerprint = [string]$AttemptReceipt.candidate_fingerprint
        classification = $Classification
        classification_source = $ClassificationSource
        occurred_at = $now.ToString("o")
        failure_message = $ErrorRecord.Exception.Message
        raw_evidence = @($rawReference)
        result_commit_artifact = if ($AttemptReceipt.PSObject.Properties.Name -contains "result_commit_artifact") {
            $AttemptReceipt.result_commit_artifact
        } else { $null }
        uat_result_commit = if ($AttemptReceipt.PSObject.Properties.Name -contains "uat_result_commit") {
            $AttemptReceipt.uat_result_commit
        } else { $null }
    }
    Publish-Sprint7AEvidence -Document $document -OutputPath $path | Out-Null
    [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $path).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $path
        raw_evidence = @($rawReference)
    }
}

function Get-Sprint8AFormalUatFailureManifestOverrides {
    param(
        [Parameter(Mandatory)][string]$AttemptPath,
        [Parameter(Mandatory)]$FailureReference,
        [Parameter(Mandatory)][string]$AttemptStatus
    )

    $rawEvidence = if ($FailureReference.PSObject.Properties.Name -contains "raw_evidence") {
        @($FailureReference.raw_evidence)
    } else { @() }
    @(
        [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $AttemptPath).Replace("\", "/")
            phase = "uat-attempt"; authoritative = $false; status = $AttemptStatus
        },
        [pscustomobject][ordered]@{
            path = [string]$FailureReference.path; sha256 = [string]$FailureReference.sha256
            phase = "uat-evidence-finalization"; authoritative = $false; status = "failed"
        },
        @($rawEvidence | ForEach-Object {
            [pscustomobject][ordered]@{
                path = [string]$_.path; sha256 = [string]$_.sha256
                phase = "uat-evidence-finalization-raw"; authoritative = $false; status = "failed"
            }
        })
    )
}

function Invoke-Sprint8AFormalUatCatchHarvest {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$AttemptCheckpoint,
        [Parameter(Mandatory)]$ErrorRecord,
        [Parameter(Mandatory)][string]$AttemptPath,
        [Parameter(Mandatory)][string]$AttemptCheckpointPath,
        [Parameter(Mandatory)][string]$ManualReceiptRoot,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [AllowNull()]$RetainedRestorationCheck,
        [AllowEmptyCollection()][object[]]$RetainedRestorationChecks = @()
    )

    $harvestRoot = Join-Path $AttemptRoot ("catch-harvest-{0}" -f [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
    [IO.Directory]::CreateDirectory($harvestRoot) | Out-Null
    $failureLog = Join-Path $harvestRoot "finalization-prerequisite-failure.log"
    [IO.File]::WriteAllText(
        $failureLog,
        "[$([DateTimeOffset]::UtcNow.ToString('o'))] $($ErrorRecord | Out-String)",
        [Text.UTF8Encoding]::new($false)
    )
    $failureEvidence = @([pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $failureLog).Replace("\", "/")
        sha256 = Get-Sprint8AFileSha256 -Path $failureLog
    })
    $classification = Resolve-Sprint8AFormalUatFailureClassification `
        -DefaultClassification "preflight/setup" `
        -ErrorRecord $ErrorRecord `
        -EvidencePaths @($failureLog)
    $defectMap = [ordered]@{}
    $mergeDefect = {
        param([Parameter(Mandatory)]$Defect)

        $classificationSource = if ($Defect.PSObject.Properties.Name -contains "classification_source") {
            [string]$Defect.classification_source
        } else { "" }
        $check = if ($Defect.PSObject.Properties.Name -contains "check") {
            [string]$Defect.check
        } elseif ($Defect.PSObject.Properties.Name -contains "name") {
            [string]$Defect.name
        } else { "unidentified-failure" }
        $message = if ($Defect.PSObject.Properties.Name -contains "message") {
            [string]$Defect.message
        } elseif ($Defect.PSObject.Properties.Name -contains "failure_message") {
            [string]$Defect.failure_message
        } else { "" }
        $key = @(
            $check,
            [string]$Defect.classification,
            $classificationSource,
            $message
        ) -join "`u{001f}"
        $evidence = if ($Defect.PSObject.Properties.Name -contains "evidence") {
            @($Defect.evidence)
        } else { @() }
        if ($defectMap.Contains($key)) {
            $existing = $defectMap[$key]
            $existing.evidence = @(@($existing.evidence) + $evidence | Group-Object {
                "$([string]$_.path)`u{001f}$([string]$_.sha256)"
            } | ForEach-Object { $_.Group[0] })
            return
        }
        $defectMap[$key] = [pscustomobject][ordered]@{
            check = $check
            classification = [string]$Defect.classification
            classification_source = $classificationSource
            message = $message
            evidence = $evidence
        }
    }
    foreach ($prior in @($AttemptCheckpoint, $AttemptReceipt)) {
        if ($prior.PSObject.Properties.Name -contains "scripted_defects") {
            foreach ($defect in @($prior.scripted_defects)) { & $mergeDefect -Defect $defect }
        }
        if ($prior.PSObject.Properties.Name -contains "failure_batch" -and
            $null -ne $prior.failure_batch -and
            $prior.failure_batch.PSObject.Properties.Name -contains "defects") {
            foreach ($defect in @($prior.failure_batch.defects)) { & $mergeDefect -Defect $defect }
        }
    }
    & $mergeDefect -Defect ([pscustomobject][ordered]@{
        check = "finalization-prerequisite"
        classification = [string]$classification.classification
        classification_source = [string]$classification.source
        message = $ErrorRecord.Exception.Message
        evidence = $failureEvidence
    })
    $manualChecks = [Collections.Generic.List[object]]::new()
    $candidateFingerprint = [string]$AttemptCheckpoint.candidate_fingerprint
    $environmentFingerprint = [string]$AttemptCheckpoint.environment_fingerprint
    $checkpointPath = [IO.Path]::GetRelativePath($repoRoot, $AttemptCheckpointPath).Replace("\", "/")
    $checkpointSha = $null
    try { $checkpointSha = Assert-Sprint8AReceiptSidecar -Path $AttemptCheckpointPath } catch { }
    foreach ($scenario in Get-Sprint8AManualUatScenarioNames) {
        $started = [DateTimeOffset]::UtcNow
        $manualFullPath = Join-Path $ManualReceiptRoot "$($scenario.ToLowerInvariant()).json"
        $manualPath = [IO.Path]::GetRelativePath($repoRoot, $manualFullPath).Replace("\", "/")
        try {
            $reference = Get-Sprint8AFormalUatReference -Path $manualPath
            $receipt = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
            Assert-Sprint8AManualUatReceipt `
                -Receipt $receipt `
                -ExpectedScenario $scenario `
                -ExpectedAttempt ([int]$AttemptReceipt.attempt) `
                -CandidateFingerprint $candidateFingerprint `
                -EnvironmentFingerprint $environmentFingerprint `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $evidenceRootPath | Out-Null
            $leasePair = Assert-Sprint8AManualUatExecutionLeasePair `
                -Receipt $receipt `
                -ReceiptReference $reference `
                -ExpectedScenario $scenario `
                -ExpectedAttempt ([int]$AttemptReceipt.attempt) `
                -CandidateFingerprint $candidateFingerprint `
                -EnvironmentFingerprint $environmentFingerprint `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $evidenceRootPath
            $leaseEvidence = @($leasePair.start, $leasePair.completion)
            if ([string]::IsNullOrWhiteSpace([string]$checkpointSha) -or
                [string]$receipt.start_checkpoint.path -cne $checkpointPath -or
                [string]$receipt.start_checkpoint.sha256 -cne $checkpointSha) {
                throw "Manual UAT scenario '$scenario' is not bound to the retained Start checkpoint."
            }
            $scenarioEvidence = @(Get-Sprint8AFormalUatEvidenceReferences -Evidence @($receipt.evidence) -Scenario $scenario)
            $cleanupEvidence = @(Get-Sprint8AFormalUatEvidenceReferences -Evidence @($receipt.cleanup_restoration.evidence) -Scenario $scenario)
            $manualChecks.Add([pscustomobject][ordered]@{
                name = $scenario
                state = [string]$receipt.state
                authoritative = $false
                diagnostic = $true
                assertions_started = [bool]$receipt.assertions_started
                assertions_started_at = if ([bool]$receipt.assertions_started) {
                    (ConvertTo-Sprint8AFormalUatTerminalTimestamp `
                        -Value $receipt.assertions_started_at `
                        -Label "$scenario assertions start")
                } else { $null }
                command = "diagnostic validation of retained manual scenario $($reference.path)"
                exit_status = if ([string]$receipt.state -ceq "passed") { 0 } elseif ([string]$receipt.state -ceq "failed") { 1 } else { $null }
                started_at = (ConvertTo-Sprint8AFormalUatTerminalTimestamp `
                    -Value $receipt.started_at `
                    -Label "$scenario start")
                ended_at = (ConvertTo-Sprint8AFormalUatTerminalTimestamp `
                    -Value $receipt.ended_at `
                    -Label "$scenario end")
                duration_ms = [long]$receipt.duration_ms
                classification = if ([string]$receipt.state -in @("failed", "blocked")) { [string]$receipt.classification } else { $null }
                classification_source = if ([string]$receipt.state -in @("failed", "blocked")) { [string]$receipt.classification_source } else { $null }
                failure_message = if ([string]$receipt.state -ceq "failed") { [string]$receipt.failure_message } else { $null }
                blocked_reason = if ([string]$receipt.state -ceq "blocked") { [string]$receipt.blocked_reason } else { $null }
                evidence = @([pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = [string]$reference.sha256 }) +
                    $leaseEvidence + $scenarioEvidence + $cleanupEvidence
            })
            if ([string]$receipt.state -ceq "failed") {
                & $mergeDefect -Defect ([pscustomobject][ordered]@{
                    check = $scenario
                    classification = [string]$receipt.classification
                    classification_source = [string]$receipt.classification_source
                    message = [string]$receipt.failure_message
                    evidence = @([pscustomobject][ordered]@{
                        path = [string]$reference.path
                        sha256 = [string]$reference.sha256
                    }) + @($leaseEvidence) + @($scenarioEvidence) + @($cleanupEvidence)
                })
            }
        } catch {
            $ended = [DateTimeOffset]::UtcNow
            $scenarioLog = Join-Path $harvestRoot "$($scenario.ToLowerInvariant()).log"
            [IO.File]::WriteAllText(
                $scenarioLog,
                "[$($ended.ToString('o'))] $($_ | Out-String)",
                [Text.UTF8Encoding]::new($false)
            )
            $scenarioEvidence = @([pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $scenarioLog).Replace("\", "/")
                sha256 = Get-Sprint8AFileSha256 -Path $scenarioLog
            })
            $manualChecks.Add([pscustomobject][ordered]@{
                name = $scenario; state = "failed"; authoritative = $false; diagnostic = $true
                assertions_started = $true; assertions_started_at = $started.ToString("o")
                command = "diagnostic validation of retained manual scenario $scenario"
                exit_status = 1; started_at = $started.ToString("o"); ended_at = $ended.ToString("o")
                duration_ms = [long][Math]::Max(0, ($ended - $started).TotalMilliseconds)
                classification = "harness"; classification_source = "catch_harvest"
                failure_message = $_.Exception.Message; blocked_reason = $null; evidence = $scenarioEvidence
            })
            & $mergeDefect -Defect ([pscustomobject][ordered]@{
                check = $scenario; classification = "harness"; classification_source = "catch_harvest"
                message = $_.Exception.Message; evidence = $scenarioEvidence
            })
        }
    }
    Assert-Sprint8AExactTerminalIdentities `
        -Results @($manualChecks) `
        -ExpectedNames (Get-Sprint8AManualUatScenarioNames) `
        -Label "Formal UAT catch harvest" | Out-Null
    $blockedMap = [ordered]@{}
    $mergeBlockedCheck = {
        param([Parameter(Mandatory)]$BlockedCheck)

        $check = if ($BlockedCheck.PSObject.Properties.Name -contains "check") {
            [string]$BlockedCheck.check
        } elseif ($BlockedCheck.PSObject.Properties.Name -contains "name") {
            [string]$BlockedCheck.name
        } else { "unidentified-blocked-check" }
        $classification = if ($BlockedCheck.PSObject.Properties.Name -contains "classification") {
            [string]$BlockedCheck.classification
        } else { "" }
        $classificationSource = if ($BlockedCheck.PSObject.Properties.Name -contains "classification_source") {
            [string]$BlockedCheck.classification_source
        } else { "" }
        $dependencyReason = if ($BlockedCheck.PSObject.Properties.Name -contains "dependency_reason") {
            [string]$BlockedCheck.dependency_reason
        } elseif ($BlockedCheck.PSObject.Properties.Name -contains "blocked_reason") {
            [string]$BlockedCheck.blocked_reason
        } else { "" }
        $key = @($check, $dependencyReason, $classification, $classificationSource) -join "`u{001f}"
        $evidence = if ($BlockedCheck.PSObject.Properties.Name -contains "evidence") {
            @($BlockedCheck.evidence)
        } else { @() }
        if ($blockedMap.Contains($key)) {
            $existing = $blockedMap[$key]
            $existing.evidence = @(@($existing.evidence) + $evidence | Group-Object {
                "$([string]$_.path)`u{001f}$([string]$_.sha256)"
            } | ForEach-Object { $_.Group[0] })
            return
        }
        $blockedMap[$key] = [pscustomobject][ordered]@{
            check = $check
            dependency_reason = $dependencyReason
            classification = if ([string]::IsNullOrWhiteSpace($classification)) { $null } else { $classification }
            classification_source = if ([string]::IsNullOrWhiteSpace($classificationSource)) { $null } else { $classificationSource }
            evidence = $evidence
        }
    }
    foreach ($prior in @($AttemptCheckpoint, $AttemptReceipt)) {
        if ($prior.PSObject.Properties.Name -contains "failure_batch" -and
            $null -ne $prior.failure_batch -and
            $prior.failure_batch.PSObject.Properties.Name -contains "blocked_checks") {
            foreach ($blocked in @($prior.failure_batch.blocked_checks)) {
                & $mergeBlockedCheck -BlockedCheck $blocked
            }
        }
    }
    foreach ($manualBlocked in @($manualChecks | Where-Object state -CEQ "blocked")) {
        & $mergeBlockedCheck -BlockedCheck ([pscustomobject][ordered]@{
            check = [string]$manualBlocked.name
            dependency_reason = [string]$manualBlocked.blocked_reason
            classification = [string]$manualBlocked.classification
            classification_source = [string]$manualBlocked.classification_source
            evidence = @($manualBlocked.evidence)
        })
    }
    $effectiveRestorationChecks = @(if (@($RetainedRestorationChecks).Count -gt 0) {
        @($RetainedRestorationChecks)
    } elseif ($AttemptReceipt.PSObject.Properties.Name -contains "restoration_checks") {
        @($AttemptReceipt.restoration_checks)
    } else { @() })
    if ($effectiveRestorationChecks.Count -gt 0) {
        Assert-Sprint8AExactTerminalIdentities `
            -Results $effectiveRestorationChecks `
            -ExpectedNames @(
                "canonical-restoration-materialization",
                "canonical-restoration-inventory",
                "canonical-restoration-smoke"
            ) `
            -Label "Formal UAT retained canonical restoration" | Out-Null
        $AttemptReceipt | Add-Member -Force -NotePropertyName restoration_checks -NotePropertyValue $effectiveRestorationChecks
        foreach ($restorationFailure in @($effectiveRestorationChecks | Where-Object state -CEQ "failed")) {
            & $mergeDefect -Defect ([pscustomobject][ordered]@{
                check = [string]$restorationFailure.name
                classification = [string]$restorationFailure.classification
                classification_source = [string]$restorationFailure.classification_source
                message = [string]$restorationFailure.failure_message
                evidence = @($restorationFailure.evidence)
            })
        }
        foreach ($restorationBlocked in @($effectiveRestorationChecks | Where-Object state -CEQ "blocked")) {
            & $mergeBlockedCheck -BlockedCheck ([pscustomobject][ordered]@{
                check = [string]$restorationBlocked.name
                dependency_reason = [string]$restorationBlocked.blocked_reason
                classification = [string]$restorationBlocked.classification
                classification_source = [string]$restorationBlocked.classification_source
                evidence = @($restorationBlocked.evidence)
            })
        }
    }
    $effectiveRestorationCheck = if ($null -ne $RetainedRestorationCheck) {
        $RetainedRestorationCheck
    } elseif ($AttemptReceipt.PSObject.Properties.Name -contains "restoration_check") {
        $AttemptReceipt.restoration_check
    } else { $null }
    if ($null -eq $effectiveRestorationCheck -and $effectiveRestorationChecks.Count -gt 0) {
        $retainedRestorationFailures = @($effectiveRestorationChecks | Where-Object state -CEQ "failed")
        $retainedRestorationBlocked = @($effectiveRestorationChecks | Where-Object state -CEQ "blocked")
        $retainedRestorationNonpassing = @($effectiveRestorationChecks | Where-Object state -CNE "passed")
        $retainedRestorationClassifications = @($retainedRestorationNonpassing | ForEach-Object {
            [string]$_.classification
        } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Sort-Object -Unique)
        $effectiveRestorationCheck = [pscustomobject][ordered]@{
            name = "canonical-restoration"
            state = if ($retainedRestorationFailures.Count -gt 0) {
                "failed"
            } elseif ($retainedRestorationBlocked.Count -gt 0) {
                "blocked"
            } else { "passed" }
            classification = if ($retainedRestorationClassifications.Count -eq 1) {
                [string]$retainedRestorationClassifications[0]
            } else { $null }
            classification_source = if ($retainedRestorationNonpassing.Count -eq 1) {
                [string]$retainedRestorationNonpassing[0].classification_source
            } elseif ($retainedRestorationNonpassing.Count -gt 1) {
                "consolidated_restoration_checks"
            } else { $null }
            failure_message = if ($retainedRestorationFailures.Count -gt 0) {
                ($retainedRestorationFailures.failure_message -join "; ")
            } else { $null }
            blocked_reason = if ($retainedRestorationBlocked.Count -gt 0) {
                ($retainedRestorationBlocked.blocked_reason -join "; ")
            } else { $null }
            evidence = @($effectiveRestorationChecks | ForEach-Object { @($_.evidence) } | Group-Object {
                "$([string]$_.path)`u{001f}$([string]$_.sha256)"
            } | ForEach-Object { $_.Group[0] })
        }
    }
    if ($null -ne $effectiveRestorationCheck) {
        $AttemptReceipt | Add-Member -Force -NotePropertyName restoration_check -NotePropertyValue $effectiveRestorationCheck
        if ([string]$effectiveRestorationCheck.state -ceq "failed" -and
            @($effectiveRestorationChecks | Where-Object state -CEQ "failed").Count -eq 0) {
            & $mergeDefect -Defect ([pscustomobject][ordered]@{
                check = "canonical-restoration"
                classification = [string]$effectiveRestorationCheck.classification
                classification_source = [string]$effectiveRestorationCheck.classification_source
                message = [string]$effectiveRestorationCheck.failure_message
                evidence = @($effectiveRestorationCheck.evidence)
            })
        } elseif ([string]$effectiveRestorationCheck.state -ceq "blocked" -and
            @($effectiveRestorationChecks | Where-Object state -CEQ "blocked").Count -eq 0) {
            & $mergeBlockedCheck -BlockedCheck ([pscustomobject][ordered]@{
                check = "canonical-restoration"
                dependency_reason = [string]$effectiveRestorationCheck.blocked_reason
                classification = [string]$effectiveRestorationCheck.classification
                classification_source = [string]$effectiveRestorationCheck.classification_source
                evidence = @($effectiveRestorationCheck.evidence)
            })
        }
    } else {
        & $mergeBlockedCheck -BlockedCheck ([pscustomobject][ordered]@{
            check = "canonical-restoration"
            dependency_reason = "blocked because finalization prerequisite authentication failed before destructive restoration"
            classification = $null
            classification_source = $null
            evidence = $failureEvidence
        })
    }
    if ([string]$AttemptReceipt.state -notin @("failed", "blocked", "passed")) {
        Set-Sprint8AFormalUatAttemptState `
            -Receipt $AttemptReceipt `
            -State "failed" `
            -Stage ([string]$AttemptReceipt.stage)
    }
    $AttemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
    $AttemptReceipt.duration_ms = [long][Math]::Max(
        0,
        ((ConvertTo-Sprint8ADateTimeOffset -Value $AttemptReceipt.ended_at -Label "catch-harvest end") -
            (ConvertTo-Sprint8ADateTimeOffset -Value $AttemptReceipt.started_at -Label "catch-harvest start")).TotalMilliseconds
    )
    $AttemptReceipt.manual_scenarios_pending = @($manualChecks | Where-Object {
        [string]$_.state -cne "passed" -or -not [bool]$_.authoritative -or [bool]$_.diagnostic
    } | ForEach-Object { [string]$_.name })
    $manualNames = @(Get-Sprint8AManualUatScenarioNames)
    $AttemptReceipt.checks = @($AttemptReceipt.checks | Where-Object {
        $manualNames -cnotcontains [string]$_.name
    }) + @($manualChecks)
    $AttemptReceipt.assertion_count = @($AttemptReceipt.checks | Where-Object assertions_started -EQ $true).Count
    $setAttemptSummary = {
        $defects = @($defectMap.Values)
        $blockedChecks = @($blockedMap.Values)
        $AttemptReceipt.failure_count = $defects.Count
        $AttemptReceipt.blocked_count = $blockedChecks.Count
        $classifications = @($defects | ForEach-Object { [string]$_.classification } | Sort-Object -Unique)
        $AttemptReceipt.classification = if ($classifications.Count -eq 1) {
            [string]$classifications[0]
        } else { $null }
        $AttemptReceipt.failure_batch = [pscustomobject][ordered]@{
            defect_count = $defects.Count
            blocked_check_count = $blockedChecks.Count
            defects = $defects
            blocked_checks = $blockedChecks
        }
    }
    & $setAttemptSummary
    $AttemptReceipt.cleanup_restoration = if ($null -eq $effectiveRestorationCheck) {
        [pscustomobject][ordered]@{
            required = $true
            result = "not_started_prerequisite_failure"
            evidence = $failureEvidence
        }
    } else {
        [pscustomobject][ordered]@{
            required = $true
            result = if ([string]$effectiveRestorationCheck.state -ceq "passed") {
                "canonical_topology_verified"
            } else { [string]$effectiveRestorationCheck.state }
            evidence = @($effectiveRestorationCheck.evidence)
        }
    }
    Publish-Sprint7AEvidence -Document $AttemptReceipt -OutputPath $AttemptPath -Overwrite | Out-Null
    try {
        Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $AttemptPath).Replace("\", "/")
            phase = "uat-attempt"; authoritative = $false; status = "failed"
        }) | Out-Null
    } catch {
        $manifestFailure = $_
        $manifestFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
            -AttemptReceipt $AttemptReceipt `
            -ErrorRecord $manifestFailure `
            -Directory $harvestRoot `
            -Classification "evidence-finalization" `
            -ClassificationSource "manifest_publication_boundary"
        & $mergeDefect -Defect ([pscustomobject][ordered]@{
            check = "evidence-manifest-publication"
            classification = "evidence-finalization"
            classification_source = "manifest_publication_boundary"
            message = $manifestFailure.Exception.Message
            evidence = @($manifestFailureReference)
        })
        & $setAttemptSummary
        Add-Sprint8AFormalUatManifestUpdateFailure `
            -AttemptReceipt $AttemptReceipt `
            -Failure ([pscustomobject][ordered]@{
            classification = "evidence-finalization"
            classification_source = "manifest_publication_boundary"
            message = $manifestFailure.Exception.Message
            evidence = @($manifestFailureReference)
            raw_evidence = @($manifestFailureReference.raw_evidence)
        }) | Out-Null
        Publish-Sprint7AEvidence -Document $AttemptReceipt -OutputPath $AttemptPath -Overwrite | Out-Null
    }
    $AttemptReceipt
}

function Assert-Sprint8AFormalUatManifestResultCommit {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)][string]$ManifestPath
    )

    Assert-Sprint8AReceiptSidecar -Path $ManifestPath | Out-Null
    $manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or
        [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
        throw "Sprint 8A evidence manifest is malformed at the formal UAT publication boundary."
    }
    $matches = @($manifest.entries | Where-Object {
        [string]$_.path -ceq [string]$AttemptReceipt.uat_result_commit.path -and
            [string]$_.sha256 -ceq [string]$AttemptReceipt.uat_result_commit.sha256 -and
            [string]$_.phase -ceq "uat" -and
            $_.authoritative -is [bool] -and [bool]$_.authoritative -and
            [string]$_.status -ceq "committed"
    })
    if ($matches.Count -ne 1) {
        throw "Sprint 8A evidence manifest does not carry the exact canonical UAT result commitment."
    }
    $sidecarPath = "$([string]$AttemptReceipt.uat_result_commit.path).sha256"
    $sidecarSha = Get-Sprint8AStringSha256 -Text "$([string]$AttemptReceipt.uat_result_commit.sha256)`n"
    $sidecarMatches = @($manifest.entries | Where-Object {
        [string]$_.path -ceq $sidecarPath -and
            [string]$_.sha256 -ceq $sidecarSha -and
            [string]$_.phase -ceq "uat" -and
            $_.authoritative -is [bool] -and [bool]$_.authoritative -and
            [string]$_.status -ceq "committed"
    })
    if ($sidecarMatches.Count -ne 1) {
        throw "Sprint 8A evidence manifest does not carry the exact canonical UAT result sidecar commitment."
    }
    $manifest
}

function Get-Sprint8AFormalUatCanonicalPairState {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedSha256
    )

    $sidecarPath = "$Path.sha256"
    $jsonExists = Test-Path -LiteralPath $Path -PathType Leaf
    $sidecarExists = Test-Path -LiteralPath $sidecarPath -PathType Leaf
    if (-not $jsonExists -and -not $sidecarExists) {
        Repair-Sprint7AEvidencePublication -Path $Path
        $jsonExists = Test-Path -LiteralPath $Path -PathType Leaf
        $sidecarExists = Test-Path -LiteralPath $sidecarPath -PathType Leaf
    } else {
        # The authenticated result-content commit makes either canonical half
        # recoverable. Preserve it and discard only uncommitted publisher
        # transients; the caller completes the exact missing half.
        Remove-Sprint7AEvidencePublicationTransients -Path $Path
    }
    if ($jsonExists -and (Get-Sprint8AFileSha256 -Path $Path) -cne $ExpectedSha256) {
        throw "Canonical formal UAT JSON differs from its committed digest."
    }
    if ($sidecarExists) {
        $sidecarText = Get-Content -LiteralPath $sidecarPath -Raw
        if ($sidecarText -cne "$ExpectedSha256`n") {
            throw "Canonical formal UAT sidecar differs from its committed bytes."
        }
    }
    if ($jsonExists -and $sidecarExists) { return "complete" }
    if ($jsonExists) { return "json-only" }
    if ($sidecarExists) { return "sidecar-only" }
    "absent"
}

function Test-Sprint8AFormalUatCanonicalPublicationStarted {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    $publicationControlObserved =
        (Test-Path -LiteralPath "$fullPath.publish-journal.json" -PathType Leaf) -or
        (Test-Path -LiteralPath "$fullPath.rollback" -PathType Leaf) -or
        (Test-Path -LiteralPath "$fullPath.sha256.rollback" -PathType Leaf)
    Repair-Sprint7AEvidencePublication -Path $fullPath
    $publicationControlObserved -or
        (Test-Path -LiteralPath $fullPath -PathType Leaf) -or
        (Test-Path -LiteralPath "$fullPath.sha256" -PathType Leaf)
}

function Complete-Sprint8AFormalUatCanonicalPair {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$Commit,
        [Parameter(Mandatory)][string]$ManifestPath,
        [Parameter(Mandatory)][string]$CanonicalPath
    )

    Assert-Sprint8AFormalUatManifestResultCommit `
        -AttemptReceipt $AttemptReceipt `
        -ManifestPath $ManifestPath | Out-Null
    $expectedSha = [string]$Commit.document.result.sha256
    $state = Get-Sprint8AFormalUatCanonicalPairState -Path $CanonicalPath -ExpectedSha256 $expectedSha
    $directory = Split-Path -Parent $CanonicalPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    if ($state -ceq "absent") {
        Publish-Sprint7AEvidence `
            -Document $Commit.document.result.receipt `
            -OutputPath $CanonicalPath | Out-Null
    } elseif ($state -ceq "json-only") {
        $temporarySidecar = Join-Path $directory ".$([IO.Path]::GetFileName($CanonicalPath)).$([guid]::NewGuid().ToString('N')).sha256.tmp"
        try {
            [IO.File]::WriteAllText($temporarySidecar, "$expectedSha`n", [Text.UTF8Encoding]::new($false))
            Move-Item -LiteralPath $temporarySidecar -Destination "$CanonicalPath.sha256"
        } finally {
            Remove-Item -LiteralPath $temporarySidecar -Force -ErrorAction SilentlyContinue
        }
    } elseif ($state -ceq "sidecar-only") {
        $temporaryJson = Join-Path $directory ".$([IO.Path]::GetFileName($CanonicalPath)).$([guid]::NewGuid().ToString('N')).json.tmp"
        try {
            [IO.File]::WriteAllText(
                $temporaryJson,
                (($Commit.document.result.receipt | ConvertTo-Json -Depth 30) + "`n"),
                [Text.UTF8Encoding]::new($false)
            )
            if ((Get-Sprint8AFileSha256 -Path $temporaryJson) -cne $expectedSha) {
                throw "Prepared canonical formal UAT JSON differs from its committed digest."
            }
            Move-Item -LiteralPath $temporaryJson -Destination $CanonicalPath
        } finally {
            Remove-Item -LiteralPath $temporaryJson -Force -ErrorAction SilentlyContinue
        }
    }
    if ((Get-Sprint8AFormalUatCanonicalPairState -Path $CanonicalPath -ExpectedSha256 $expectedSha) -cne "complete" -or
        (Assert-Sprint8AReceiptSidecar -Path $CanonicalPath) -cne $expectedSha) {
        throw "Canonical formal UAT result pair differs from its evidence-manifest commitment."
    }
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -ManifestPath $ManifestPath | Out-Null
    $state
}

function Assert-Sprint8AFormalUatFinalizationCompletionCheckpoint {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$AttemptCheckpoint,
        [Parameter(Mandatory)][string]$AttemptCheckpointSha256
    )

    if ($AttemptReceipt.PSObject.Properties.Name -notcontains "finalization_completion_checkpoint" -or
        [string]::IsNullOrWhiteSpace([string]$AttemptReceipt.finalization_completion_checkpoint.path) -or
        [string]$AttemptReceipt.finalization_completion_checkpoint.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Formal UAT finalization-only retry requires its durable completion checkpoint reference."
    }
    $reference = Get-Sprint8AFormalUatReference -Path ([string]$AttemptReceipt.finalization_completion_checkpoint.path)
    if ([string]$reference.sha256 -cne [string]$AttemptReceipt.finalization_completion_checkpoint.sha256) {
        throw "Formal UAT finalization completion checkpoint digest differs from its attempt reference."
    }
    $document = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
    if (($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 1 -or [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "uat-finalization-completion" -or
        $document.authoritative -isnot [bool] -or [bool]$document.authoritative -or
        [string]$document.state -cne "complete" -or [int]$document.attempt -ne [int]$AttemptReceipt.attempt -or
        [string]$document.candidate_fingerprint -cne [string]$AttemptReceipt.candidate_fingerprint -or
        [string]$document.environment_fingerprint -cne [string]$AttemptReceipt.environment_fingerprint -or
        [string]$document.normalized_deployment_configuration_sha256 -notmatch '^[0-9a-f]{64}$' -or
        ($document.source_identity | ConvertTo-Json -Depth 20 -Compress) -cne
            ($AttemptReceipt.source_identity | ConvertTo-Json -Depth 20 -Compress) -or
        ($document.prerequisite_receipts | ConvertTo-Json -Depth 20 -Compress) -cne
            ($AttemptReceipt.prerequisite_receipts | ConvertTo-Json -Depth 20 -Compress) -or
        ($document.endpoints | ConvertTo-Json -Depth 20 -Compress) -cne
            ($AttemptReceipt.endpoints | ConvertTo-Json -Depth 20 -Compress) -or
        [string]$document.start_checkpoint.path -cne
            [IO.Path]::GetRelativePath($repoRoot, $attemptCheckpointPath).Replace("\", "/") -or
        [string]$document.start_checkpoint.sha256 -cne $AttemptCheckpointSha256 -or
        @($document.prerequisite_receipts).Count -ne 3 -or
        @($document.manual_receipts).Count -ne (Get-Sprint8AManualUatScenarioNames).Count -or
        [string]$document.scripted_check.name -cne "scripted-uat" -or
        [string]$document.scripted_check.state -cne "passed" -or
        [string]$document.restoration_check.state -cne "passed") {
        throw "Formal UAT finalization completion checkpoint is malformed or bound to another execution."
    }
    Assert-Sprint8AExactTerminalIdentities `
        -Results @($document.manual_checks) `
        -ExpectedNames (Get-Sprint8AManualUatScenarioNames) `
        -Label "Formal UAT finalization completion checkpoint" | Out-Null
    if (@($document.manual_checks | Where-Object {
        [string]$_.state -cne "passed" -or -not [bool]$_.authoritative -or [bool]$_.diagnostic
    }).Count -ne 0) {
        throw "Formal UAT finalization completion checkpoint contains an ineligible manual scenario."
    }
    $manualReceiptPaths = @($document.manual_receipts | ForEach-Object { [string]$_.path })
    if (@($manualReceiptPaths | Sort-Object -Unique).Count -ne (Get-Sprint8AManualUatScenarioNames).Count) {
        throw "Formal UAT finalization completion checkpoint repeats a manual receipt path."
    }
    for ($index = 0; $index -lt (Get-Sprint8AManualUatScenarioNames).Count; $index++) {
        $scenario = (Get-Sprint8AManualUatScenarioNames)[$index]
        $manualReference = $document.manual_receipts[$index]
        $suffix = "/uat/attempt-$([int]$AttemptReceipt.attempt)/manual/$($scenario.ToLowerInvariant()).json"
        if (-not "/$([string]$manualReference.path)".EndsWith($suffix, [StringComparison]::Ordinal) -or
            [string]$manualReference.path -match '(^|/)\.\.(/|$)|\\' -or
            [string]$manualReference.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "Formal UAT finalization completion checkpoint has a non-canonical '$scenario' receipt reference."
        }
    }
    Assert-Sprint8AExactTerminalIdentities `
        -Results @($document.restoration_checks) `
        -ExpectedNames @(
            "canonical-restoration-materialization",
            "canonical-restoration-inventory",
            "canonical-restoration-smoke"
        ) `
        -Label "Formal UAT finalization completion restoration" | Out-Null
    if (@($document.restoration_checks | Where-Object state -CNE "passed").Count -ne 0) {
        throw "Formal UAT finalization completion checkpoint contains a nonpassing restoration check."
    }
    [pscustomobject][ordered]@{ reference = $reference; document = $document }
}

function Invoke-Sprint8AFormalUatPublicationTail {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$CompletionCheckpoint,
        [Parameter(Mandatory)]$CurrentSource,
        [Parameter(Mandatory)][string]$AttemptPath,
        [Parameter(Mandatory)][string]$ResultCommitPath,
        [Parameter(Mandatory)][string]$OutputPath
    )

    $completion = $CompletionCheckpoint.document
    $scriptedCheck = $completion.scripted_check
    $manualChecks = @($completion.manual_checks)
    $restorationCheck = $completion.restoration_check
    $resultReference = Publish-Sprint8ALifecycleReceipt `
        -Phase "uat" `
        -Attempt ([int]$AttemptReceipt.attempt) `
        -Source $CurrentSource `
        -EnvironmentFingerprint ([string]$completion.environment_fingerprint) `
        -NormalizedDeploymentConfigurationSha256 ([string]$completion.normalized_deployment_configuration_sha256) `
        -CandidateFingerprint ([string]$completion.candidate_fingerprint) `
        -PrerequisiteReceipts @($completion.prerequisite_receipts) `
        -Checks (@($scriptedCheck) + @($manualChecks)) `
        -CleanupRestoration ([pscustomobject][ordered]@{
            required = $true
            result = "canonical_topology_verified"
            evidence = @($restorationCheck.evidence)
        }) `
        -Details ([pscustomobject][ordered]@{
            manual_checkpoint = $completion.start_checkpoint
            manual_receipts = @($completion.manual_receipts)
            canonical_restoration = $restorationCheck
            canonical_restoration_checks = @($completion.restoration_checks)
            finalization_completion_checkpoint = [pscustomobject][ordered]@{
                path = [string]$CompletionCheckpoint.reference.path
                sha256 = [string]$CompletionCheckpoint.reference.sha256
            }
        }) `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -OutputPath $OutputPath `
        -PrepareOnly
    $resultCommitReference = Publish-Sprint8AFormalUatResultCommit `
        -AttemptReceipt $AttemptReceipt `
        -ResultReference $resultReference `
        -Path $ResultCommitPath
    Set-Sprint8AFormalUatAttemptState -Receipt $AttemptReceipt -State "passed" -Stage "result-committed"
    $AttemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
    $AttemptReceipt.duration_ms = [long][Math]::Max(
        0,
        ((ConvertTo-Sprint8ADateTimeOffset -Value $AttemptReceipt.ended_at -Label "publication end") -
            (ConvertTo-Sprint8ADateTimeOffset -Value $AttemptReceipt.started_at -Label "publication start")).TotalMilliseconds
    )
    $AttemptReceipt.manual_scenarios_pending = @()
    $AttemptReceipt.checks = @($scriptedCheck) + @($manualChecks)
    $AttemptReceipt.assertion_count = @($AttemptReceipt.checks | Where-Object assertions_started -EQ $true).Count
    $AttemptReceipt.failure_count = 0
    $AttemptReceipt.blocked_count = 0
    $AttemptReceipt.classification = $null
    $AttemptReceipt.failure_batch = $null
    $AttemptReceipt.uat_result_commit = [pscustomobject][ordered]@{
        path = [string]$resultReference.path; sha256 = [string]$resultReference.sha256
    }
    $AttemptReceipt.result_commit_artifact = $resultCommitReference
    $AttemptReceipt | Add-Member -Force -NotePropertyName restoration_check -NotePropertyValue $restorationCheck
    $AttemptReceipt | Add-Member -Force -NotePropertyName restoration_checks -NotePropertyValue @($completion.restoration_checks)
    $AttemptReceipt.cleanup_restoration = [pscustomobject][ordered]@{
        required = $true; result = "canonical_topology_verified"; evidence = @($restorationCheck.evidence)
    }
    if ($AttemptReceipt.PSObject.Properties.Name -contains "finalization_retry") {
        $AttemptReceipt.finalization_retry.eligible = $false
        $AttemptReceipt.finalization_retry | Add-Member `
            -Force `
            -NotePropertyName consumed_at `
            -NotePropertyValue ([DateTimeOffset]::UtcNow.ToString("o"))
    }
    Publish-Sprint7AEvidence -Document $AttemptReceipt -OutputPath $AttemptPath -Overwrite | Out-Null
    $manifestEntries = @(
        @($completion.prerequisite_receipts | ForEach-Object {
            $prerequisitePath = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $evidenceRootPath `
                -Path ([string]$_.path)
            $phase = (Get-Content -LiteralPath ([string]$prerequisitePath.full_path) -Raw | ConvertFrom-Json).phase
            [pscustomobject][ordered]@{
                path = [string]$_.path; sha256 = [string]$_.sha256
                phase = if ($phase -ceq "validation-preflight") { "preflight" } elseif ($phase -ceq "candidate-freeze") { "candidate" } else { "sit" }
                authoritative = $true; status = "passed"
            }
        }),
        [pscustomobject][ordered]@{
            path = [string]$resultReference.path; sha256 = [string]$resultReference.sha256
            phase = "uat"; authoritative = $true; status = "committed"
        },
        [pscustomobject][ordered]@{
            path = "$([string]$resultReference.path).sha256"
            sha256 = Get-Sprint8AStringSha256 -Text "$([string]$resultReference.sha256)`n"
            phase = "uat"; authoritative = $true; status = "committed"
        },
        [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $AttemptPath).Replace("\", "/")
            phase = "uat-attempt"; authoritative = $false; status = "passed"
        },
        [pscustomobject][ordered]@{
            path = [string]$completion.start_checkpoint.path; sha256 = [string]$completion.start_checkpoint.sha256
            phase = "uat-checkpoint"; authoritative = $false; status = "passed"
        },
        [pscustomobject][ordered]@{
            path = [string]$CompletionCheckpoint.reference.path; sha256 = [string]$CompletionCheckpoint.reference.sha256
            phase = "uat-finalization-completion"; authoritative = $false; status = "complete"
        },
        @($completion.manifest_overrides)
    )
    $manifestEntries = @($manifestEntries | Group-Object path | ForEach-Object { $_.Group[0] })
    try {
        Sync-Sprint8AFormalUatEvidenceManifest `
            -Overrides $manifestEntries `
            -AllowMissingCanonicalUatCommitment | Out-Null
    } catch {
        $_.Exception.Data["Sprint8AClassification"] = "evidence-finalization"
        throw
    }
    $manifestPath = Join-Path $evidenceRootPath "evidence-manifest.json"
    Assert-Sprint8AFormalUatManifestResultCommit `
        -AttemptReceipt $AttemptReceipt `
        -ManifestPath $manifestPath | Out-Null
    $canonicalResult = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -Path $OutputPath
    $commit = Assert-Sprint8AFormalUatResultCommit `
        -AttemptReceipt $AttemptReceipt `
        -CommitPath ([string]$AttemptReceipt.result_commit_artifact.path)
    Complete-Sprint8AFormalUatCanonicalPair `
        -AttemptReceipt $AttemptReceipt `
        -Commit $commit `
        -ManifestPath $manifestPath `
        -CanonicalPath ([string]$canonicalResult.full_path) | Out-Null
    $resultReference
}

function Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][int]$ExpectedAttempt
    )

    $receiptProperties = @($Receipt.PSObject.Properties | ForEach-Object { $_.Name })
    $hasRetainedManifestFailure = $receiptProperties -contains "manifest_update_failure"
    $hasRetainedCatchHarvestFailure = $false
    if ($receiptProperties -contains "failure_batch" -and
        $null -ne $Receipt.failure_batch -and
        @($Receipt.failure_batch.PSObject.Properties | ForEach-Object { $_.Name }) -contains "defects") {
        $hasRetainedCatchHarvestFailure = @($Receipt.failure_batch.defects | Where-Object {
                    [string]$_.check -ceq "catch-harvest" -and
                    [string]$_.classification -in @("harness", "evidence-finalization")
                }).Count -gt 0
    }
    return (
        ($Receipt.schema_version -is [int] -or $Receipt.schema_version -is [long]) -and
        [int]$Receipt.schema_version -eq 1 -and
        [string]$Receipt.sprint -ceq "sprint-8a" -and
        [string]$Receipt.phase -ceq "uat" -and
        ($Receipt.attempt -is [int] -or $Receipt.attempt -is [long]) -and
        [int]$Receipt.attempt -eq $ExpectedAttempt -and
        $Receipt.authoritative -is [bool] -and
        -not [bool]$Receipt.authoritative -and
        [string]$Receipt.state -in @("passed", "failed", "blocked") -and
        ($hasRetainedManifestFailure -or $hasRetainedCatchHarvestFailure)
    )
}

function Get-Sprint8AFormalUatTerminalAttemptManifestRepairOverrides {
    param([AllowEmptyCollection()][string[]]$ExcludedPaths = @())

    $excluded = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($excludedPath in @($ExcludedPaths)) { $excluded.Add([string]$excludedPath) | Out-Null }
    $manifestPath = Join-Path $evidenceRootPath "evidence-manifest.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { return @() }
    Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $manifestByPath = @{}
    foreach ($entry in @($manifest.entries)) { $manifestByPath[[string]$entry.path] = $entry }
    $attemptsRoot = Join-Path $evidenceRootPath "attempts"
    if (-not (Test-Path -LiteralPath $attemptsRoot -PathType Container)) { return @() }
    @(
        foreach ($file in @(Get-ChildItem -LiteralPath $attemptsRoot -File -Filter "uat-*.json" | Where-Object {
                    $_.Name -match '^uat-(\d+)\.json$'
                })) {
            $attemptNumber = [int][regex]::Match($file.Name, '^uat-(\d+)\.json$').Groups[1].Value
            $sha = Assert-Sprint8AReceiptSidecar -Path $file.FullName
            $path = [IO.Path]::GetRelativePath($repoRoot, $file.FullName).Replace("\", "/")
            if ($excluded.Contains($path)) { continue }
            $existing = $manifestByPath[$path]
            if ($null -eq $existing -or [string]$existing.sha256 -ceq $sha) { continue }
            $receipt = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
            if (-not (Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility `
                    -Receipt $receipt -ExpectedAttempt $attemptNumber)) {
                throw "Formal UAT cannot repair stale manifest entry '$path' because its receipt is not an authenticated terminal publication/catch-harvest failure."
            }
            [pscustomobject][ordered]@{
                path = $path
                sha256 = $sha
                phase = "uat-attempt"
                authoritative = $false
                status = [string]$receipt.state
            }
        }
    )
}

function Sync-Sprint8AFormalUatEvidenceManifest {
    param(
        [AllowEmptyCollection()][object[]]$Overrides = @(),
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $manifestPath = Join-Path $evidenceRootPath "evidence-manifest.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Formal UAT requires the preflight-declared and SIT-updated evidence manifest."
    }
    Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
    $existingManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $requestedOverridePaths = @($Overrides | ForEach-Object { [string]$_.path })
    $repairOverrides = @(Get-Sprint8AFormalUatTerminalAttemptManifestRepairOverrides `
        -ExcludedPaths $requestedOverridePaths)
    $Overrides = @(@($Overrides) + $repairOverrides | Group-Object path | ForEach-Object { $_.Group[0] })
    $entries = Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -Overrides $Overrides
    $authorizedReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
        -ExistingEntries @($existingManifest.entries) `
        -UpdatedEntries $entries)
    $manifestOutputPath = [IO.Path]::GetRelativePath($repoRoot, $manifestPath).Replace("\", "/")
    $reference = Publish-Sprint8AEvidenceManifest `
        -Entries $entries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -OutputPath $manifestOutputPath `
        -Merge `
        -AuthorizedReplacementPaths @($authorizedReplacementPaths)
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -ManifestPath $manifestPath `
        -AllowMissingCanonicalUatCommitment:$AllowMissingCanonicalUatCommitment | Out-Null
    $reference
}

function Assert-Sprint8AFormalUatReceiptBinding {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$ExpectedReference,
        [Parameter(Mandatory)][string]$Label
    )

    $matches = @($Receipt.prerequisite_receipts | Where-Object {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$_.path)
        [string]$resolved.path -ceq [string]$ExpectedReference.path -and
            [string]$_.sha256 -ceq [string]$ExpectedReference.sha256
    })
    if ($matches.Count -ne 1) {
        throw "$Label does not bind its exact prerequisite receipt and SHA-256."
    }
}

function Get-Sprint8AFormalUatEndpoints {
    param([Parameter(Mandatory)]$Environment)

    $endpoints = $Environment.contract.endpoints
    foreach ($name in @("gateway", "materialization_control", "supervisor")) {
        $uri = $null
        if (-not [Uri]::TryCreate([string]$endpoints.$name, [UriKind]::Absolute, [ref]$uri) -or
            $uri.Scheme -cne [Uri]::UriSchemeHttp -or
            -not $uri.IsLoopback -or
            $uri.AbsolutePath -cne "/" -or
            -not [string]::IsNullOrEmpty($uri.Query) -or
            -not [string]::IsNullOrEmpty($uri.Fragment)) {
            throw "Frozen Sprint 8A environment has an invalid '$name' endpoint."
        }
    }
    $requestedGateway = ([Uri]$BaseUrl).AbsoluteUri.TrimEnd("/")
    $frozenGateway = ([Uri][string]$endpoints.gateway).AbsoluteUri.TrimEnd("/")
    if ($requestedGateway -cne $frozenGateway) {
        throw "Formal UAT -BaseUrl differs from the frozen environment gateway endpoint."
    }
    [pscustomobject][ordered]@{
        gateway = $frozenGateway
        materialization_control = ([Uri][string]$endpoints.materialization_control).AbsoluteUri.TrimEnd("/")
        supervisor = ([Uri][string]$endpoints.supervisor).AbsoluteUri.TrimEnd("/")
    }
}

function Invoke-Sprint8AFormalUatCheck {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$LogPath,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$EvidencePaths,
        [Parameter(Mandatory)][ValidateSet("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization", "product-decision")][string]$DefaultClassification
    )

    $started = [DateTimeOffset]::UtcNow
    [IO.File]::WriteAllText(
        $LogPath,
        "[$($started.ToString('o'))] assertion_started name=$Name`n",
        [Text.UTF8Encoding]::new($false)
    )
    $passed = $false
    $failure = $null
    $failureRecord = $null
    try {
        & $Action *>&1 | ForEach-Object {
            $text = ($_ | Out-String).TrimEnd()
            if (-not [string]::IsNullOrEmpty($text)) {
                [IO.File]::AppendAllText(
                    $LogPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] $text`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        foreach ($path in $EvidencePaths) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                throw "Formal UAT check '$Name' did not produce '$path'."
            }
        }
        $passed = $true
    } catch {
        $failureRecord = $_
        $failure = $_.Exception.Message
        [IO.File]::AppendAllText(
            $LogPath,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] failure`n$($_ | Out-String)`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    $ended = [DateTimeOffset]::UtcNow
    $classification = if ($passed) {
        $null
    } else {
        Resolve-Sprint8AFormalUatFailureClassification `
            -DefaultClassification $DefaultClassification `
            -ErrorRecord $failureRecord `
            -EvidencePaths $EvidencePaths
    }
    $terminalState = if ($passed) {
        "passed"
    } elseif ([string]$classification.classification -ceq "product-decision") {
        "blocked"
    } else {
        "failed"
    }
    $evidence = @($EvidencePaths | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object {
        [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, [IO.Path]::GetFullPath($_)).Replace("\", "/")
            sha256 = Get-Sprint8AFileSha256 -Path $_
        }
    })
    [pscustomobject][ordered]@{
        name = $Name
        command = $Command
        state = $terminalState
        assertions_started = $terminalState -cne "blocked"
        assertions_started_at = if ($terminalState -cne "blocked") { $started.ToString("o") } else { $null }
        started_at = $started.ToString("o")
        ended_at = $ended.ToString("o")
        duration_ms = [long][Math]::Max(0, ($ended - $started).TotalMilliseconds)
        exit_status = if ($terminalState -ceq "passed") { 0 } elseif ($terminalState -ceq "failed") { 1 } else { $null }
        failure_message = if ($terminalState -ceq "failed") { $failure } else { $null }
        classification = if ($terminalState -ceq "passed") { $null } else { [string]$classification.classification }
        classification_source = if ($terminalState -ceq "passed") { $null } else { [string]$classification.source }
        blocked_reason = if ($terminalState -ceq "blocked") { $failure } else { $null }
        evidence = @(
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $LogPath).Replace("\", "/")
                sha256 = Get-Sprint8AFileSha256 -Path $LogPath
            }
        ) + $evidence
    }
}

if ($SelfTest) {
    $pairSelfTestRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-uat-pair-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($pairSelfTestRoot) | Out-Null
    try {
        $repairEligibleFixture = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "uat"; attempt = 5
            authoritative = $false; state = "failed"
            manifest_update_failure = [pscustomobject]@{ message = "retained publication failure" }
        }
        if (-not (Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility `
                -Receipt $repairEligibleFixture -ExpectedAttempt 5)) {
            throw "Formal UAT self-test rejected an authenticated terminal manifest-publication repair."
        }
        $repairEligibleFixture.state = "executing"
        if (Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility `
                -Receipt $repairEligibleFixture -ExpectedAttempt 5) {
            throw "Formal UAT self-test accepted a nonterminal manifest repair."
        }
        $repairEligibleFixture.state = "failed"
        $repairEligibleFixture.PSObject.Properties.Remove("manifest_update_failure")
        if (Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility `
                -Receipt $repairEligibleFixture -ExpectedAttempt 5) {
            throw "Formal UAT self-test accepted a manifest repair without retained publication-failure evidence."
        }
        $repairEligibleFixture | Add-Member -NotePropertyName failure_batch -NotePropertyValue ([pscustomobject]@{})
        if (Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility `
                -Receipt $repairEligibleFixture -ExpectedAttempt 5) {
            throw "Formal UAT self-test accepted an empty failure batch as manifest-repair evidence."
        }
        $repairEligibleFixture.PSObject.Properties.Remove("failure_batch")
        $repairEligibleFixture | Add-Member -NotePropertyName failure_batch -NotePropertyValue ([pscustomobject]@{
            defects = @([pscustomobject]@{ check = "catch-harvest"; classification = "harness" })
        })
        if (-not (Test-Sprint8AFormalUatTerminalAttemptManifestRepairEligibility `
                -Receipt $repairEligibleFixture -ExpectedAttempt 5)) {
            throw "Formal UAT self-test rejected a terminal receipt with retained catch-harvest failure evidence."
        }
        $emptyEvidenceCheck = Invoke-Sprint8AFormalUatCheck `
            -Name "self-test-empty-evidence" `
            -Command "no-op identity assertion" `
            -Action { } `
            -LogPath (Join-Path $pairSelfTestRoot "empty-evidence-check.log") `
            -EvidencePaths @() `
            -DefaultClassification "preflight/setup"
        if ([string]$emptyEvidenceCheck.state -cne "passed" -or
            @($emptyEvidenceCheck.evidence).Count -ne 1 -or
            -not ([string]$emptyEvidenceCheck.evidence[0].path).EndsWith("empty-evidence-check.log")) {
            throw "Formal UAT self-test rejected an evidence-free identity check."
        }
        $preJournalBoundaryPath = Join-Path $pairSelfTestRoot "pre-journal-result.json"
        $preJournalTemporary = Join-Path $pairSelfTestRoot ".pre-journal-result.json.$([guid]::NewGuid().ToString('N')).tmp"
        [IO.File]::WriteAllText($preJournalTemporary, "{`"state`":`"uncommitted`"}`n", [Text.UTF8Encoding]::new($false))
        $preJournalTemporarySha = Get-Sprint8AFileSha256 -Path $preJournalTemporary
        [IO.File]::WriteAllText("$preJournalTemporary.sha256", "$preJournalTemporarySha`n", [Text.UTF8Encoding]::new($false))
        $preJournalControlTemporary = Join-Path $pairSelfTestRoot ".pre-journal-result.json.publish-journal.json.$([guid]::NewGuid().ToString('N')).tmp"
        [IO.File]::WriteAllText($preJournalControlTemporary, "{", [Text.UTF8Encoding]::new($false))
        if (Test-Sprint8AFormalUatCanonicalPublicationStarted -Path $preJournalBoundaryPath) {
            throw "Formal UAT canonical publication self-test treated pre-journal transients as committed publication intent."
        }
        if ((Test-Path -LiteralPath $preJournalTemporary) -or
            (Test-Path -LiteralPath "$preJournalTemporary.sha256") -or
            (Test-Path -LiteralPath $preJournalControlTemporary)) {
            throw "Formal UAT canonical publication self-test retained pre-journal transients."
        }

        $repairableBoundaryPath = Join-Path $pairSelfTestRoot "repairable-result.json"
        $repairableTemporary = Join-Path $pairSelfTestRoot ".repairable-result.json.$([guid]::NewGuid().ToString('N')).tmp"
        [IO.File]::WriteAllText($repairableTemporary, "{`"state`":`"passed`"}`n", [Text.UTF8Encoding]::new($false))
        $repairableTemporarySha = Get-Sprint8AFileSha256 -Path $repairableTemporary
        [IO.File]::WriteAllText("$repairableTemporary.sha256", "$repairableTemporarySha`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText("$repairableBoundaryPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        if (-not (Test-Sprint8AFormalUatCanonicalPublicationStarted -Path $repairableBoundaryPath) -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $repairableBoundaryPath -SidecarPath "$repairableBoundaryPath.sha256")) {
            throw "Formal UAT canonical publication self-test did not retain and recover repairable journal intent."
        }

        $journalOnlyBoundaryPath = Join-Path $pairSelfTestRoot "journal-only-result.json"
        [IO.File]::WriteAllText("$journalOnlyBoundaryPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        if (-not (Test-Sprint8AFormalUatCanonicalPublicationStarted -Path $journalOnlyBoundaryPath) -or
            (Test-Path -LiteralPath "$journalOnlyBoundaryPath.publish-journal.json")) {
            throw "Formal UAT canonical publication self-test did not preserve journal-only publication intent across repair."
        }

        $pairPath = Join-Path $pairSelfTestRoot "uat-result.json"
        $pairDocument = [pscustomobject][ordered]@{ schema_version = 1; sprint = "sprint-8a"; state = "passed" }
        $pairJson = ($pairDocument | ConvertTo-Json -Depth 30) + "`n"
        $pairSha = Get-Sprint8AStringSha256 -Text $pairJson
        if ((Get-Sprint8AFormalUatCanonicalPairState -Path $pairPath -ExpectedSha256 $pairSha) -cne "absent") {
            throw "Formal UAT canonical-pair self-test lost the absent state."
        }
        [IO.File]::WriteAllText($pairPath, $pairJson, [Text.UTF8Encoding]::new($false))
        $orphanSidecarTemporary = Join-Path $pairSelfTestRoot ".uat-result.json.$([guid]::NewGuid().ToString('N')).sha256.tmp"
        [IO.File]::WriteAllText($orphanSidecarTemporary, "$pairSha`n", [Text.UTF8Encoding]::new($false))
        if ((Get-Sprint8AFormalUatCanonicalPairState -Path $pairPath -ExpectedSha256 $pairSha) -cne "json-only") {
            throw "Formal UAT canonical-pair self-test lost the JSON-only state."
        }
        if (Test-Path -LiteralPath $orphanSidecarTemporary) {
            throw "Formal UAT canonical-pair self-test retained an uncommitted sidecar temporary."
        }
        [IO.File]::WriteAllText("$pairPath.sha256", "$pairSha`n", [Text.UTF8Encoding]::new($false))
        if ((Get-Sprint8AFormalUatCanonicalPairState -Path $pairPath -ExpectedSha256 $pairSha) -cne "complete") {
            throw "Formal UAT canonical-pair self-test lost the complete state."
        }
        [IO.File]::Delete($pairPath)
        $orphanJsonTemporary = Join-Path $pairSelfTestRoot ".uat-result.json.$([guid]::NewGuid().ToString('N')).json.tmp"
        [IO.File]::WriteAllText($orphanJsonTemporary, $pairJson, [Text.UTF8Encoding]::new($false))
        if ((Get-Sprint8AFormalUatCanonicalPairState -Path $pairPath -ExpectedSha256 $pairSha) -cne "sidecar-only") {
            throw "Formal UAT canonical-pair self-test lost the sidecar-only state."
        }
        if (Test-Path -LiteralPath $orphanJsonTemporary) {
            throw "Formal UAT canonical-pair self-test retained an uncommitted JSON temporary."
        }
        [IO.File]::WriteAllText("$pairPath.sha256", "$(('0' * 64))`n", [Text.UTF8Encoding]::new($false))
        $malformedPairRejected = $false
        try { Get-Sprint8AFormalUatCanonicalPairState -Path $pairPath -ExpectedSha256 $pairSha | Out-Null } catch {
            $malformedPairRejected = $true
        }
        if (-not $malformedPairRejected) {
            throw "Formal UAT canonical-pair self-test accepted a malformed partial pair."
        }

        $failureBatchFixture = [pscustomobject]@{
            failure_count = 1; blocked_count = 1; classification = "product"
            scripted_defects = @([pscustomobject]@{
                check = "scripted-smoke"; classification = "product"; classification_source = "declared_check_category"
                message = "smoke failed"; evidence = @([pscustomobject]@{ path = "smoke-a.log"; sha256 = "a" * 64 })
            })
            failure_batch = [pscustomobject]@{
                defect_count = 1; blocked_check_count = 1
                defects = @([pscustomobject]@{
                    name = "scripted-smoke"; classification = "product"; classification_source = "declared_check_category"
                    failure_message = "smoke failed"; evidence = @([pscustomobject]@{ path = "smoke-b.log"; sha256 = "b" * 64 })
                })
                blocked_checks = @([pscustomobject]@{
                    name = "UAT-01"; blocked_reason = "blocked by scripted-smoke"
                    evidence = @([pscustomobject]@{ path = "smoke-a.log"; sha256 = "a" * 64 })
                })
            }
        }
        Merge-Sprint8AFormalUatFailureBatch `
            -AttemptReceipt $failureBatchFixture `
            -Defects @([pscustomobject]@{
                check = "scripted-smoke"; classification = "product"; classification_source = "declared_check_category"
                message = "smoke failed"; evidence = @([pscustomobject]@{ path = "smoke-a.log"; sha256 = "a" * 64 })
            }) `
            -BlockedChecks @([pscustomobject]@{
                check = "UAT-01"; dependency_reason = "blocked by scripted-smoke"
                evidence = @([pscustomobject]@{ path = "smoke-b.log"; sha256 = "b" * 64 })
            }) | Out-Null
        if ([int]$failureBatchFixture.failure_batch.defect_count -ne 1 -or
            [int]$failureBatchFixture.failure_batch.blocked_check_count -ne 1 -or
            @($failureBatchFixture.failure_batch.defects[0].evidence).Count -ne 2 -or
            @($failureBatchFixture.failure_batch.blocked_checks[0].evidence).Count -ne 2 -or
            [string]$failureBatchFixture.failure_batch.blocked_checks[0].dependency_reason -cne "blocked by scripted-smoke") {
            throw "Formal UAT failure-batch self-test did not preserve and deduplicate prior defects or blocked reasons."
        }

        $retryHistoryFixture = [pscustomobject]@{}
        $retryFailureOne = [pscustomobject][ordered]@{
            occurred_at = "2026-01-01T00:00:00Z"; classification = "evidence-finalization"
            classification_source = "final_publication_boundary"; message = "first"
            evidence = @([pscustomobject]@{ path = "failure-one.json"; sha256 = "c" * 64 })
        }
        $retryFailureTwo = [pscustomobject][ordered]@{
            occurred_at = "2026-01-01T00:01:00Z"; classification = "environment"
            classification_source = "structured_exception"; message = "second"
            evidence = @([pscustomobject]@{ path = "failure-two.json"; sha256 = "d" * 64 })
        }
        Add-Sprint8AFormalUatFinalizationRetryFailure -AttemptReceipt $retryHistoryFixture -Failure $retryFailureOne | Out-Null
        Add-Sprint8AFormalUatFinalizationRetryFailure -AttemptReceipt $retryHistoryFixture -Failure $retryFailureOne | Out-Null
        Add-Sprint8AFormalUatFinalizationRetryFailure -AttemptReceipt $retryHistoryFixture -Failure $retryFailureTwo | Out-Null
        if ($retryHistoryFixture.finalization_retry.eligible -isnot [bool] -or
            [bool]$retryHistoryFixture.finalization_retry.eligible -or
            @($retryHistoryFixture.finalization_retry.failure_history).Count -ne 2 -or
            @($retryHistoryFixture.evidence_finalization_failure_history).Count -ne 2 -or
            [string]$retryHistoryFixture.finalization_retry.last_failure.message -cne "second") {
            throw "Formal UAT finalization-retry self-test did not append, deduplicate, or link failure history without prior retry metadata."
        }

        $rawFailureReceiptFixture = [pscustomobject]@{
            attempt = 99; source_identity = [pscustomobject]@{ commit = "a" * 40 }
            environment_fingerprint = "e" * 64; candidate_fingerprint = "f" * 64
        }
        $rawFailureError = $null
        try { throw [InvalidOperationException]::new("raw manifest failure") } catch { $rawFailureError = $_ }
        $rawFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
            -AttemptReceipt $rawFailureReceiptFixture `
            -ErrorRecord $rawFailureError `
            -Directory $pairSelfTestRoot `
            -Classification "evidence-finalization" `
            -ClassificationSource "manifest_publication_boundary"
        $rawFailureFullPath = [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$rawFailureReference.path)))
        $rawFailureDocument = Get-Content -LiteralPath $rawFailureFullPath -Raw | ConvertFrom-Json
        $rawFailureOverrides = @(Get-Sprint8AFormalUatFailureManifestOverrides `
            -AttemptPath (Join-Path $pairSelfTestRoot "attempt.json") `
            -FailureReference $rawFailureReference `
            -AttemptStatus "failed")
        $rawManifestFailureEntry = [pscustomobject][ordered]@{
            classification = "evidence-finalization"; classification_source = "manifest_publication_boundary"
            message = "raw manifest failure"; evidence = @($rawFailureReference)
            raw_evidence = @($rawFailureReference.raw_evidence)
        }
        Add-Sprint8AFormalUatManifestUpdateFailure `
            -AttemptReceipt $rawFailureReceiptFixture `
            -Failure $rawManifestFailureEntry | Out-Null
        Add-Sprint8AFormalUatManifestUpdateFailure `
            -AttemptReceipt $rawFailureReceiptFixture `
            -Failure $rawManifestFailureEntry | Out-Null
        $rawEvidenceFullPath = [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$rawFailureReference.raw_evidence[0].path)))
        if (@($rawFailureReference.raw_evidence).Count -ne 1 -or
            -not (Test-Path -LiteralPath $rawEvidenceFullPath) -or
            [string]$rawFailureDocument.classification_source -cne "manifest_publication_boundary" -or
            @($rawFailureDocument.raw_evidence).Count -ne 1 -or
            @($rawFailureReceiptFixture.manifest_update_failure_history).Count -ne 1 -or
            @($rawFailureReceiptFixture.manifest_update_failure.raw_evidence).Count -ne 1 -or
            $rawFailureOverrides.Count -ne 3 -or
            @($rawFailureOverrides | ForEach-Object { [string]$_.phase } | Sort-Object -Unique) -cnotcontains "uat-evidence-finalization-raw") {
            throw "Formal UAT manifest-failure self-test did not retain classified raw evidence and manifest bindings."
        }
        $typedTimestamp = ('{"timestamp":"2026-01-01T00:00:00.0000000+00:00"}' | ConvertFrom-Json).timestamp
        if ((ConvertTo-Sprint8ADateTimeOffset -Value $typedTimestamp -Label "self-test timestamp").ToUniversalTime() -ne
            [DateTimeOffset]::Parse("2026-01-01T00:00:00Z")) {
            throw "Formal UAT self-test lost the instant while normalizing a typed JSON timestamp."
        }
    } finally {
        if (Test-Path -LiteralPath $pairSelfTestRoot -PathType Container) {
            [IO.Directory]::Delete($pairSelfTestRoot, $true)
        }
    }
    $sourceText = Get-Content -LiteralPath $PSCommandPath -Raw
    foreach ($requiredSourceGuard in @(
            '"scripts/run-sprint-8a-formal-uat.ps1",',
            '"scripts/sprint-8a-lifecycle-chain.ps1"',
            '[IO.Path]::GetRelativePath($repoRoot, $manualFullPath)',
            '$effectiveRestorationChecks = @(if (',
            '-ExcludedPaths $requestedOverridePaths'
        )) {
        if (-not $sourceText.Contains($requiredSourceGuard)) {
            throw "Formal UAT self-test cannot find required correction guard: $requiredSourceGuard"
        }
    }
    $attemptShapeFixture = New-Sprint8AFormalUatAttemptReceipt -AttemptNumber 1
    $directAttemptAssignments = @([regex]::Matches($sourceText, '\$attemptReceipt\.([A-Za-z0-9_]+)\s*=') |
        ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
    $missingAttemptFields = @($directAttemptAssignments | Where-Object {
        $attemptShapeFixture.PSObject.Properties.Name -cnotcontains $_
    })
    if ($missingAttemptFields.Count -gt 0) {
        throw "Formal UAT attempt receipt omits directly assigned field(s): $($missingAttemptFields -join ', ')."
    }
    $candidateFingerprintFixture = "e" * 64
    $setupFailureFixture = [pscustomobject]@{
        state = "failed"; candidate_fingerprint = $candidateFingerprintFixture; classification = "preflight/setup"
    }
    $productFailureFixture = [pscustomobject]@{
        state = "failed"; candidate_fingerprint = $candidateFingerprintFixture; classification = "product"
    }
    if ((Test-Sprint8AFormalUatAttemptInvalidatesCandidate `
            -Receipt $setupFailureFixture `
            -CandidateFingerprint $candidateFingerprintFixture) -or
        -not (Test-Sprint8AFormalUatAttemptInvalidatesCandidate `
            -Receipt $productFailureFixture `
            -CandidateFingerprint $candidateFingerprintFixture)) {
        throw "Formal UAT self-test confused a runner setup failure with a candidate-invalidating product failure."
    }
    $rehearsalPhase = "candidate-rehearsal-" + "uat-diagnostics"
    $diagnosticRunner = "uat-sprint-" + "8a.ps1"
    $legacySourceField = "mutable_source_" + "identity"
    if ($sourceText.Contains($rehearsalPhase) -or $sourceText.Contains($diagnosticRunner)) {
        throw "Formal Sprint 8A UAT must not consume or relabel rehearsal diagnostics."
    }
    if (-not $sourceText.Contains('"Start", "Finalize"') -or
        -not $sourceText.Contains('Get-Sprint8AManualUatScenarioNames') -or
        -not $sourceText.Contains('Publish-Sprint8ALifecycleReceipt') -or
        -not $sourceText.Contains('-CleanupRestoration') -or
        -not $sourceText.Contains('-State "passed" -Stage "result-committed"') -or
        -not $sourceText.Contains('-State "failed" -Stage "diagnostic-harvest-complete"') -or
        -not $sourceText.Contains('Publish-Sprint8AFormalUatResultCommit') -or
        -not $sourceText.Contains('Publish-Sprint8AFormalUatFinalizationFailure') -or
        -not $sourceText.Contains('-AuthorizeDisposableReset') -or
        -not $sourceText.Contains('-AuthorizeUatHarnessOnlySourceAdvance') -or
        -not $sourceText.Contains('endpoints = $null') -or
        -not $sourceText.Contains('verified_uat_harness_only_advance') -or
        -not $sourceText.Contains('scripts/run-sprint-8a-formal-uat.ps1') -or
        -not $sourceText.Contains('scripts/validate-resource-reference-nondisclosure.ps1') -or
        -not $sourceText.Contains('docs/sprints/sprint-8a-verification.md') -or
        -not $sourceText.Contains('uat-source-advance-recovery') -or
        -not $sourceText.Contains('prior-failed-attempt.json') -or
        -not $sourceText.Contains('manual_assertions_reexecuted = $false') -or
        -not $sourceText.Contains('-PrepareOnly') -or
        -not $sourceText.Contains('-Merge') -or
        -not $sourceText.Contains('Open-Sprint8AValidationAttemptLock') -or
        -not $sourceText.Contains('Sync-Sprint8AFormalUatEvidenceManifest') -or
        -not $sourceText.Contains('Get-Sprint8AEvidenceManifestReplacementPaths') -or
        -not $sourceText.Contains('Repair-Sprint8AManualUatPreparedPublications') -or
        -not $sourceText.Contains('Merge-Sprint8AFormalUatFailureBatch') -or
        -not $sourceText.Contains('Add-Sprint8AFormalUatManifestUpdateFailure') -or
        -not $sourceText.Contains('evidence_finalization_failure_history') -or
        -not $sourceText.Contains('Get-Sprint8AFormalUatFailureManifestOverrides') -or
        -not $sourceText.Contains('manifest_publication_boundary') -or
        -not $sourceText.Contains('Assert-Sprint8AEvidenceManifestCompleteness') -or
        -not $sourceText.Contains('post-scripted-identity') -or
        -not $sourceText.Contains('-Name "canonical-restoration-materialization"') -or
        -not $sourceText.Contains('-Name "canonical-restoration-inventory"') -or
        -not $sourceText.Contains('-Name "canonical-restoration-smoke"') -or
        -not $sourceText.Contains('evidence-manifest-publication') -or
        -not $sourceText.Contains('finalizations') -or
        -not $sourceText.Contains('uat/attempt-$Attempt') -or
        -not $sourceText.Contains('attempts/uat-$Attempt.json') -or
        $sourceText.Contains($legacySourceField)) {
        throw "Formal Sprint 8A UAT self-test cannot find its staged receipt contract."
    }
    if ((Get-Command Get-Sprint8ACandidateIdentity).Parameters.ContainsKey("EnvironmentFingerprint") -or
        -not (Get-Command Publish-Sprint8ALifecycleReceipt).Parameters.ContainsKey("CleanupRestoration") -or
        -not (Get-Command Publish-Sprint8ALifecycleReceipt).Parameters.ContainsKey("PrepareOnly") -or
        -not (Get-Command Publish-Sprint8AManualUatReceipt).Parameters.ContainsKey("Attempt") -or
        -not (Get-Command Publish-Sprint8AManualUatReceipt).Parameters.ContainsKey("Diagnostic") -or
        -not (Get-Command Publish-Sprint8AManualUatReceipt).Parameters.ContainsKey("TesterIdentity") -or
        -not (Get-Command Publish-Sprint8AManualUatReceipt).Parameters.ContainsKey("Preconditions") -or
        -not (Get-Command Assert-Sprint8AManualUatReceipt).Parameters.ContainsKey("EvidenceRoot") -or
        -not (Get-Command Repair-Sprint8AManualUatPreparedPublications -CommandType Function -ErrorAction SilentlyContinue) -or
        -not (Get-Command Invoke-Sprint8AFormalUatCatchHarvest).Parameters.ContainsKey("RetainedRestorationChecks") -or
        -not (Get-Command Get-Sprint8AEvidenceManifestReplacementPaths -CommandType Function -ErrorAction SilentlyContinue) -or
        -not (Get-Command Publish-Sprint8AEvidenceManifest).Parameters.ContainsKey("Merge") -or
        -not (Get-Command Assert-Sprint8AEvidenceManifestCompleteness).Parameters.ContainsKey("AllowMissingCanonicalUatCommitment")) {
        throw "Formal Sprint 8A UAT self-test found an incompatible lifecycle helper interface."
    }
    $startBlockStart = $sourceText.LastIndexOf('if ($Stage -ceq "Start")', [StringComparison]::Ordinal)
    $finalizeBlockStart = $sourceText.IndexOf('if (-not (Test-Path -LiteralPath $attemptPath', $startBlockStart, [StringComparison]::Ordinal)
    if ($startBlockStart -lt 0 -or $finalizeBlockStart -le $startBlockStart -or
        $sourceText.Substring($startBlockStart, $finalizeBlockStart - $startBlockStart).Contains('Formal UAT committed attempt')) {
        throw "Formal Sprint 8A UAT self-test found committed-attempt validation before Start populated prerequisite references."
    }
    $startRuntimeBlock = $sourceText.Substring($startBlockStart, $finalizeBlockStart - $startBlockStart)
    $startLockIndex = $startRuntimeBlock.IndexOf('Open-Sprint8AValidationAttemptLock', [StringComparison]::Ordinal)
    $startDirectoryIndex = $startRuntimeBlock.IndexOf('Initialize-Sprint8AFormalUatAttemptDirectories', [StringComparison]::Ordinal)
    $finalizeRuntimeStart = $sourceText.IndexOf('$attemptReceipt = $null', $startBlockStart, [StringComparison]::Ordinal)
    $finalizeRuntimeBlock = $sourceText.Substring($finalizeRuntimeStart)
    $finalizeLockIndex = $finalizeRuntimeBlock.IndexOf('Open-Sprint8AValidationAttemptLock', [StringComparison]::Ordinal)
    $finalizeDirectoryIndex = $finalizeRuntimeBlock.IndexOf('Initialize-Sprint8AFormalUatAttemptDirectories', [StringComparison]::Ordinal)
    $preparedRepairIndex = $finalizeRuntimeBlock.IndexOf('Repair-Sprint8AManualUatPreparedPublications `', [StringComparison]::Ordinal)
    $initialCompletenessIndex = $finalizeRuntimeBlock.IndexOf('Assert-Sprint8AEvidenceManifestCompleteness `', [StringComparison]::Ordinal)
    $initialNoOpenLeaseIndex = $finalizeRuntimeBlock.IndexOf('Assert-Sprint8ANoOpenManualUatLeases `', [StringComparison]::Ordinal)
    if ($startLockIndex -lt 0 -or $startDirectoryIndex -le $startLockIndex -or
        $finalizeRuntimeStart -lt 0 -or $finalizeLockIndex -lt 0 -or $finalizeDirectoryIndex -le $finalizeLockIndex -or
        $preparedRepairIndex -le $finalizeDirectoryIndex -or
        $initialCompletenessIndex -le $preparedRepairIndex -or
        $initialNoOpenLeaseIndex -le $preparedRepairIndex) {
        throw "Formal Sprint 8A UAT self-test found an invalid lock, directory, or prepared-publication repair boundary."
    }
    $bindingReference = [pscustomobject]@{ path = "artifacts/sprint-8a-closeout/preflight-result.json"; sha256 = "a" * 64 }
    $bindingReceipt = [pscustomobject]@{ prerequisite_receipts = @($bindingReference) }
    Assert-Sprint8AFormalUatReceiptBinding -Receipt $bindingReceipt -ExpectedReference $bindingReference -Label "self-test" | Out-Null
    $bindingRejected = $false
    try {
        Assert-Sprint8AFormalUatReceiptBinding `
            -Receipt ([pscustomobject]@{ prerequisite_receipts = @() }) `
            -ExpectedReference $bindingReference `
            -Label "self-test empty binding"
    } catch { $bindingRejected = $true }
    if (-not $bindingRejected) {
        throw "Formal Sprint 8A UAT self-test accepted an empty prerequisite binding."
    }
    $stateFixture = [pscustomobject]@{
        state = "preparing"
        stage = "prerequisites"
        state_history = @([pscustomobject]@{ state = "preparing"; stage = "prerequisites"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8AFormalUatAttemptState -Receipt $stateFixture -State "executing" -Stage "scripted"
    Set-Sprint8AFormalUatAttemptState -Receipt $stateFixture -State "executing" -Stage "manual"
    Set-Sprint8AFormalUatAttemptState -Receipt $stateFixture -State "finalizing" -Stage "manual-receipt-validation"
    Set-Sprint8AFormalUatAttemptState -Receipt $stateFixture -State "finalizing" -Stage "canonical-restoration"
    Set-Sprint8AFormalUatAttemptState -Receipt $stateFixture -State "passed" -Stage "result-committed"
    $terminalTransitionRejected = $false
    try {
        Set-Sprint8AFormalUatAttemptState -Receipt $stateFixture -State "executing" -Stage "manual"
    } catch { $terminalTransitionRejected = $true }
    if (-not $terminalTransitionRejected -or @($stateFixture.state_history).Count -ne 6) {
        throw "Formal Sprint 8A UAT self-test found an invalid state-machine transition contract."
    }
    $diagnosticStateFixture = [pscustomobject]@{
        state = "preparing"; stage = "prerequisites"
        state_history = @([pscustomobject]@{ state = "preparing"; stage = "prerequisites"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8AFormalUatAttemptState -Receipt $diagnosticStateFixture -State "executing" -Stage "scripted"
    Set-Sprint8AFormalUatAttemptState -Receipt $diagnosticStateFixture -State "executing" -Stage "diagnostic-manual"
    Set-Sprint8AFormalUatAttemptState -Receipt $diagnosticStateFixture -State "finalizing" -Stage "manual-receipt-validation"
    Set-Sprint8AFormalUatAttemptState -Receipt $diagnosticStateFixture -State "finalizing" -Stage "canonical-restoration"
    Set-Sprint8AFormalUatAttemptState -Receipt $diagnosticStateFixture -State "failed" -Stage "diagnostic-harvest-complete"
    $blockedStateFixture = [pscustomobject]@{
        state = "preparing"; stage = "prerequisites"
        state_history = @([pscustomobject]@{ state = "preparing"; stage = "prerequisites"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8AFormalUatAttemptState -Receipt $blockedStateFixture -State "executing" -Stage "scripted"
    Set-Sprint8AFormalUatAttemptState -Receipt $blockedStateFixture -State "blocked" -Stage "scripted"
    $structuredError = $null
    try {
        $fixtureException = [InvalidOperationException]::new("structured failure")
        $fixtureException.Data["Sprint8AClassification"] = "product"
        throw $fixtureException
    } catch { $structuredError = $_ }
    $structuredClassification = Resolve-Sprint8AFormalUatFailureClassification `
        -DefaultClassification "harness" `
        -ErrorRecord $structuredError
    if ([string]$structuredClassification.classification -cne "product" -or
        [string]$structuredClassification.source -cne "structured_exception") {
        throw "Formal Sprint 8A UAT self-test lost structured failure-classification precedence."
    }
    $mixedDisposition = Get-Sprint8AFormalUatScriptedDisposition `
        -Defects @([pscustomobject]@{ classification = "product" }) `
        -ProductDecisions @([pscustomobject]@{ classification = "product-decision" })
    if ([string]$mixedDisposition.terminal_state -cne "failed" -or
        [int]$mixedDisposition.defect_count -ne 1 -or
        [int]$mixedDisposition.blocked_decision_count -ne 1 -or
        [string]$mixedDisposition.classification -cne "product") {
        throw "Formal Sprint 8A UAT self-test lost a real scripted defect beside a product decision."
    }
    if (-not (Test-Sprint8AFormalUatHarnessChangedPaths -ChangedPaths @(
                "scripts/sprint-8a-lifecycle-chain.ps1"
            )) -or
        -not (Test-Sprint8AFormalUatHarnessChangedPaths -ChangedPaths @(
                "scripts/run-sprint-8a-formal-uat.ps1",
                "scripts/validate-resource-reference-nondisclosure.ps1"
            )) -or
        (Test-Sprint8AFormalUatHarnessChangedPaths -ChangedPaths @()) -or
        (Test-Sprint8AFormalUatHarnessChangedPaths -ChangedPaths @(
                "scripts/sprint-8a-lifecycle-chain.ps1",
                "crates/tessara-core/src/main.rs"
            ))) {
        throw "Formal Sprint 8A UAT self-test accepted an outside/empty source advance or rejected an allowed subset."
    }
    $jsonTimestampFixture = '{"at":"2026-08-10T04:13:48.2064453-04:00"}' | ConvertFrom-Json
    $canonicalTimestamp = ConvertTo-Sprint8AFormalUatTerminalTimestamp `
        -Value $jsonTimestampFixture.at `
        -Label "formal UAT JSON timestamp self-test"
    if ($canonicalTimestamp -notmatch '(Z|[+-][0-9]{2}:[0-9]{2})$') {
        throw "Formal Sprint 8A UAT self-test lost the UTC offset after JSON timestamp conversion."
    }
    if (@(Get-Sprint8AFormalUatFailureClassifications -Failures @()).Count -ne 0 -or
        (@(Get-Sprint8AFormalUatFailureClassifications -Failures @(
                    [pscustomobject]@{ classification = "product" },
                    [pscustomobject]@{ classification = "product" },
                    [pscustomobject]@{ classification = $null }
                )) -join "`n") -cne "product") {
        throw "Formal Sprint 8A UAT self-test mishandled empty or duplicate failure classifications."
    }
    $singletonFailureMessages = @(Get-Sprint8AFormalUatFailureMessages -Receipt ([pscustomobject]@{
        failure_batch = [pscustomobject]@{
            defects = @([pscustomobject]@{ message = "singleton" })
        }
    }))
    $emptyFailureMessages = @(Get-Sprint8AFormalUatFailureMessages -Receipt ([pscustomobject]@{}))
    if ($singletonFailureMessages.Count -ne 1 -or
        [string]$singletonFailureMessages[0] -cne "singleton" -or
        $emptyFailureMessages.Count -ne 0) {
        throw "Formal Sprint 8A UAT self-test lost singleton/empty failure-message cardinality."
    }
    $fixtureSource = [pscustomobject][ordered]@{
        commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "self-test"
        acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64
    }
    $exactSourceContext = Get-Sprint8AFormalUatSourceContext `
        -CurrentSource $fixtureSource `
        -CandidateSource $fixtureSource `
        -RepositoryRoot $repoRoot
    if ($null -ne $exactSourceContext.harness_only_source_advance -or
        -not (Test-Sprint8ASourceIdentityMatch `
            -Expected $fixtureSource `
            -Actual $exactSourceContext.candidate_source_identity)) {
        throw "Formal Sprint 8A UAT self-test rejected an exact candidate/harness source match."
    }
    $fixtureCandidate = Get-Sprint8ACandidateIdentity `
        -RepositoryRoot $repoRoot `
        -Source $fixtureSource `
        -NormalizedDeploymentConfigurationSha256 ("f" * 64)
    if ([string]$fixtureCandidate.fingerprint -notmatch '^[0-9a-f]{64}$' -or
        $fixtureCandidate.contract.PSObject.Properties.Name -contains "environment_fingerprint") {
        throw "Formal Sprint 8A UAT self-test found a malformed or environment-coupled candidate identity."
    }
    Test-Sprint8ALifecycleChain | Out-Null
    Write-Host "Sprint 8A formal UAT prerequisite, staging, and receipt self-test passed."
    return
}

if ($Attempt -lt 1) { throw "Formal Sprint 8A UAT requires a positive -Attempt." }
if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
    throw "Formal Sprint 8A UAT requires a repository-relative -EvidenceRoot."
}
$evidenceRootReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $EvidenceRoot
$evidenceRootPath = [string]$evidenceRootReference.full_path
$attemptRoot = Join-Path $evidenceRootPath "uat/attempt-$Attempt"
$canonicalManualReceiptDirectory = Join-Path $attemptRoot "manual"
$manualReceiptRoot = if ([string]::IsNullOrWhiteSpace($ManualReceiptDirectory)) {
    $canonicalManualReceiptDirectory
} else {
    [string](Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -Path $ManualReceiptDirectory).full_path
}
if ([IO.Path]::GetFullPath($manualReceiptRoot) -cne [IO.Path]::GetFullPath($canonicalManualReceiptDirectory)) {
    throw "Formal UAT manual receipts must use the attempt-scoped directory '$([IO.Path]::GetRelativePath($repoRoot, $canonicalManualReceiptDirectory).Replace('\', '/'))'."
}
$attemptPath = Join-Path $evidenceRootPath "attempts/uat-$Attempt.json"
$attemptCheckpointPath = Join-Path $evidenceRootPath "attempts/uat-$Attempt-manual-checkpoint.json"
$logRoot = Join-Path $attemptRoot "logs"
$scriptedInventoryPath = Join-Path $attemptRoot "scripted-inventory.json"
$scriptedSmokePath = Join-Path $attemptRoot "scripted-smoke.json"
$finalInventoryPath = Join-Path $attemptRoot "final-inventory.json"
$finalSmokePath = Join-Path $attemptRoot "final-smoke.json"
$restorationRoot = Join-Path $attemptRoot "restoration"
$restorationSummaryPath = Join-Path $restorationRoot "materialization/attempt-$Attempt/materialization-result.json"
$resultCommitPath = Join-Path $attemptRoot "result-commit.json"

if ($Stage -ceq "Start") {
    $attemptLock = Open-Sprint8AValidationAttemptLock `
        -Path (Join-Path $evidenceRootPath "validation-attempt.lock")
    try {
    Initialize-Sprint8AFormalUatAttemptDirectories `
        -AttemptRoot $attemptRoot `
        -AttemptPath $attemptPath `
        -LogRoot $logRoot
    $canonicalStartOutput = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -Path $OutputPath
    if (Test-Sprint8AFormalUatCanonicalPublicationStarted -Path ([string]$canonicalStartOutput.full_path)) {
        throw "Formal UAT cannot create an attempt after canonical UAT result publication has begun."
    }
    if (Test-Path -LiteralPath $attemptPath) {
        throw "Formal UAT attempt $Attempt already exists and cannot be reused."
    }
    $attemptReceipt = New-Sprint8AFormalUatAttemptReceipt -AttemptNumber $Attempt
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath | Out-Null
    try {
        Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
            phase = "uat-attempt"
            authoritative = $false
            status = "preparing"
        }) | Out-Null
        $priorAttempts = @(Get-Sprint8AFormalUatPriorAttempts -CurrentAttemptPath $attemptPath)
        $openPriorAttempts = @($priorAttempts | Where-Object {
            [string]$_.receipt.state -notin @("passed", "failed", "blocked")
        })
        if ($openPriorAttempts.Count -gt 0) {
            throw "Formal UAT cannot start while prior nonterminal attempt(s) remain open: $(@($openPriorAttempts.receipt.attempt) -join ', ')."
        }
        $canonicalOutput = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -Path $OutputPath
        if (Test-Path -LiteralPath ([string]$canonicalOutput.full_path) -PathType Leaf) {
            throw "Formal UAT cannot start a new attempt after the canonical UAT result has been published."
        }
        $prerequisiteStarted = [DateTimeOffset]::UtcNow
        $preflightReference = Get-Sprint8AFormalUatReference -Path $PreflightReceipt
        $candidateReference = Get-Sprint8AFormalUatReference -Path $CandidateReceipt
        $sitReference = Get-Sprint8AFormalUatReference -Path $SitReceipt
        $preflight = Assert-Sprint8ALifecyclePrerequisite `
            -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath `
            -Reference $preflightReference -ExpectedPhase "validation-preflight"
        $candidate = Assert-Sprint8ALifecyclePrerequisite `
            -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath `
            -Reference $candidateReference -ExpectedPhase "candidate-freeze"
        $candidateReceiptObject = $candidate.receipt
        $candidateFingerprint = [string]$candidateReceiptObject.candidate_fingerprint
        $environmentFingerprint = [string]$candidateReceiptObject.environment_fingerprint
        if ($candidateFingerprint -notmatch '^[0-9a-f]{64}$' -or $environmentFingerprint -notmatch '^[0-9a-f]{64}$') {
            throw "Candidate receipt omits exact candidate/environment fingerprints."
        }
        $invalidatedSameCandidate = @($priorAttempts | Where-Object {
            Test-Sprint8AFormalUatAttemptInvalidatesCandidate `
                -Receipt $_.receipt `
                -CandidateFingerprint $candidateFingerprint
        })
        if ($invalidatedSameCandidate.Count -gt 0) {
            throw "Formal UAT candidate was already invalidated or blocked by prior attempt(s): $(@($invalidatedSameCandidate.receipt.attempt) -join ', ')."
        }
        Assert-Sprint8AFormalUatReceiptBinding -Receipt $candidateReceiptObject -ExpectedReference $preflightReference -Label "Candidate receipt"
        $sit = Assert-Sprint8ALifecyclePrerequisite `
            -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath `
            -Reference $sitReference -ExpectedPhase "sit" `
            -ExpectedCandidateFingerprint $candidateFingerprint `
            -ExpectedEnvironmentFingerprint $environmentFingerprint
        Assert-Sprint8AFormalUatReceiptBinding -Receipt $sit.receipt -ExpectedReference $candidateReference -Label "SIT receipt"
        Assert-Sprint8AExactTerminalIdentities `
            -Results @($sit.receipt.checks) `
            -ExpectedNames (Get-Sprint8ASitLaneNames) `
            -Label "Authoritative SIT" | Out-Null
        if (@($sit.receipt.checks | Where-Object state -CNE "passed").Count -ne 0) {
            throw "Authoritative SIT receipt contains a failed or blocked terminal lane."
        }
        $currentSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
        $sourceContext = Get-Sprint8AFormalUatSourceContext `
            -CurrentSource $currentSource `
            -CandidateSource $candidateReceiptObject.source_identity `
            -RepositoryRoot $repoRoot `
            -AllowHarnessOnlySourceAdvance:$AuthorizeUatHarnessOnlySourceAdvance
        $environment = Get-Sprint8AEnvironmentContract `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -ProbeDatabases
        if ([string]$environment.fingerprint -cne $environmentFingerprint) {
            throw "Formal UAT environment differs from the frozen candidate."
        }
        $normalizedDeploymentConfigurationSha256 = [string]$environment.contract.compose.normalized_config_sha256
        $candidateIdentity = Get-Sprint8ACandidateIdentity `
            -RepositoryRoot $repoRoot `
            -Source $sourceContext.candidate_source_identity `
            -NormalizedDeploymentConfigurationSha256 $normalizedDeploymentConfigurationSha256
        if ([string]$candidateIdentity.fingerprint -cne $candidateFingerprint) {
            throw "Formal UAT recomputed a different candidate fingerprint."
        }
        $endpoints = Get-Sprint8AFormalUatEndpoints -Environment $environment
        Assert-Sprint8ALifecyclePrerequisiteSet `
            -Phase "uat" `
            -References @($preflightReference, $candidateReference, $sitReference) `
            -Source $sourceContext.candidate_source_identity `
            -EnvironmentFingerprint $environmentFingerprint `
            -NormalizedDeploymentConfigurationSha256 $normalizedDeploymentConfigurationSha256 `
            -CandidateFingerprint $candidateFingerprint `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath | Out-Null
        $prerequisiteEnded = [DateTimeOffset]::UtcNow
        $prerequisiteCheck = [pscustomobject][ordered]@{
            name = "authenticated-prerequisites"
            command = "authenticate preflight/candidate/SIT receipts, source, environment, endpoints, and exact SIT lane inventory"
            state = "passed"
            authoritative = $true
            diagnostic = $false
            assertions_started = $true
            assertions_started_at = $prerequisiteStarted.ToString("o")
            started_at = $prerequisiteStarted.ToString("o")
            ended_at = $prerequisiteEnded.ToString("o")
            duration_ms = [long][Math]::Max(0, ($prerequisiteEnded - $prerequisiteStarted).TotalMilliseconds)
            exit_status = 0
            classification = $null
            classification_source = $null
            failure_message = $null
            blocked_reason = $null
            evidence = @(@($preflightReference, $candidateReference, $sitReference) | ForEach-Object {
                [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
            })
        }
        $attemptReceipt.source_identity = $currentSource
        $attemptReceipt.candidate_source_identity = $sourceContext.candidate_source_identity
        $attemptReceipt.uat_harness_only_source_advance = $sourceContext.harness_only_source_advance
        $attemptReceipt.source_verification_state = if ($null -eq $sourceContext.harness_only_source_advance) {
            "verified"
        } else { "verified_uat_harness_only_advance" }
        $attemptReceipt.environment_fingerprint = $environmentFingerprint
        $attemptReceipt.candidate_fingerprint = $candidateFingerprint
        $attemptReceipt.endpoints = $endpoints
        $attemptReceipt.prerequisite_receipts = @(
            foreach ($reference in @($preflightReference, $candidateReference, $sitReference)) {
                [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = [string]$reference.sha256 }
            }
        )
        Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "executing" -Stage "scripted"
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        $checks = [Collections.Generic.List[object]]::new()
        $checks.Add((Invoke-Sprint8AFormalUatCheck `
            -Name "scripted-inventory" `
            -Command ".\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl '$BaseUrl' -OutputPath '$scriptedInventoryPath'" `
            -LogPath (Join-Path $logRoot "scripted-inventory.log") `
            -EvidencePaths @($scriptedInventoryPath) `
            -DefaultClassification "product" `
            -Action {
                & (Join-Path $repoRoot "scripts/audit-sprint-8a-deployed-inventory.ps1") -BaseUrl $BaseUrl -OutputPath $scriptedInventoryPath
                $childPassed = $?
                if (-not $childPassed) { throw "Formal UAT inventory/navigation assertion failed." }
            }))
        $checks.Add((Invoke-Sprint8AFormalUatCheck `
            -Name "scripted-smoke" `
            -Command ".\scripts\smoke-sprint-8a.ps1 -BaseUrl '$($endpoints.gateway)' -SupervisorUrl '$($endpoints.supervisor)' -OutputPath '$scriptedSmokePath'" `
            -LogPath (Join-Path $logRoot "scripted-smoke.log") `
            -EvidencePaths @($scriptedSmokePath) `
            -DefaultClassification "product" `
            -Action {
                & (Join-Path $repoRoot "scripts/smoke-sprint-8a.ps1") -BaseUrl $endpoints.gateway -SupervisorUrl $endpoints.supervisor -OutputPath $scriptedSmokePath
                $childPassed = $?
                if (-not $childPassed) { throw "Formal UAT live Sprint 8A smoke assertion failed." }
            }))
        $checks.Add((Invoke-Sprint8AFormalUatCheck `
            -Name "post-scripted-identity" `
            -Command "re-authenticate clean source, candidate, environment, and endpoint identities after both scripted siblings" `
            -LogPath (Join-Path $logRoot "post-scripted-identity.log") `
            -EvidencePaths @() `
            -DefaultClassification "preflight/setup" `
            -Action {
                $postScriptedSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
                try {
                    Assert-Sprint8ASourceIdentityObject -Source $postScriptedSource -RequireClean | Out-Null
                    if (-not (Test-Sprint8ASourceIdentityMatch -Expected $sourceContext.harness_source_identity -Actual $postScriptedSource)) {
                        throw "Formal UAT source changed during scripted execution."
                    }
                } catch {
                    $_.Exception.Data["Sprint8AClassification"] = "preflight/setup"
                    throw
                }
                try {
                    $postScriptedEnvironment = Get-Sprint8AEnvironmentContract `
                        -RepositoryRoot $repoRoot `
                        -EvidenceRoot $EvidenceRoot `
                        -ProbeDatabases
                    if ([string]$postScriptedEnvironment.fingerprint -cne $environmentFingerprint) {
                        throw "Formal UAT environment changed during scripted execution."
                    }
                    $postScriptedCandidate = Get-Sprint8ACandidateIdentity `
                        -RepositoryRoot $repoRoot `
                        -Source $sourceContext.candidate_source_identity `
                        -NormalizedDeploymentConfigurationSha256 ([string]$postScriptedEnvironment.contract.compose.normalized_config_sha256)
                    if ([string]$postScriptedCandidate.fingerprint -cne $candidateFingerprint) {
                        throw "Formal UAT candidate identity changed during scripted execution."
                    }
                    $postScriptedEndpoints = Get-Sprint8AFormalUatEndpoints -Environment $postScriptedEnvironment
                    if (($postScriptedEndpoints | ConvertTo-Json -Compress) -cne ($endpoints | ConvertTo-Json -Compress)) {
                        throw "Formal UAT endpoint identity changed during scripted execution."
                    }
                } catch {
                    $_.Exception.Data["Sprint8AClassification"] = "environment"
                    throw
                }
            }))
        $attemptReceipt.assertions_started = $true
        $attemptReceipt.checks = @($prerequisiteCheck) + @($checks)
        $attemptReceipt.assertion_count = @($attemptReceipt.checks | Where-Object assertions_started -EQ $true).Count
        $attemptReceipt.failure_count = @($checks | Where-Object state -CEQ "failed").Count
        $attemptReceipt.blocked_count = @($checks | Where-Object state -CEQ "blocked").Count
        $scriptedDefects = @($checks | Where-Object state -CEQ "failed" | ForEach-Object {
            [pscustomobject][ordered]@{
                check = [string]$_.name
                classification = [string]$_.classification
                classification_source = [string]$_.classification_source
                message = [string]$_.failure_message
                evidence = @($_.evidence)
            }
        })
        $productDecisions = @($checks | Where-Object {
            [string]$_.state -ceq "blocked" -and [string]$_.classification -ceq "product-decision"
        })
        $identityDefects = @($scriptedDefects | Where-Object check -CEQ "post-scripted-identity")
        if ($identityDefects.Count -gt 0) {
            $blockedAt = [DateTimeOffset]::UtcNow
            $blockedManual = @(Get-Sprint8AManualUatScenarioNames | ForEach-Object {
                [pscustomobject][ordered]@{
                    name = $_; state = "blocked"; authoritative = $false; diagnostic = $true
                    assertions_started = $false; assertions_started_at = $null
                    command = "manual scenario blocked before execution"
                    exit_status = $null; started_at = $null; ended_at = $blockedAt.ToString("o"); duration_ms = 0L
                    classification = $null; classification_source = $null; failure_message = $null
                    blocked_reason = "blocked because post-scripted source/environment identity authentication failed"
                    evidence = @($identityDefects | ForEach-Object { @($_.evidence) })
                }
            })
            $attemptReceipt.checks = @($attemptReceipt.checks) + $blockedManual
            $attemptReceipt.failure_count = $scriptedDefects.Count
            $attemptReceipt.blocked_count = $productDecisions.Count + $blockedManual.Count
            $attemptReceipt.manual_scenarios_pending = @()
            $identityClassifications = @($scriptedDefects.classification | Sort-Object -Unique)
            $attemptReceipt.classification = if ($identityClassifications.Count -eq 1) {
                [string]$identityClassifications[0]
            } else { $null }
            $attemptReceipt.failure_batch = [pscustomobject][ordered]@{
                defect_count = $scriptedDefects.Count
                blocked_check_count = $attemptReceipt.blocked_count
                defects = $scriptedDefects
                blocked_checks = @($productDecisions) + $blockedManual
            }
            Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "failed" -Stage "scripted"
            $attemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            $attemptReceipt.duration_ms = [long][Math]::Max(
                0,
                ((ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.ended_at -Label "product-decision end") -
                    (ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.started_at -Label "product-decision start")).TotalMilliseconds
            )
            $attemptReceipt.cleanup_restoration.result = "not_proven"
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                phase = "uat-attempt"; authoritative = $false; status = "failed"
            }) | Out-Null
            $identityException = [InvalidOperationException]::new(
                "Formal UAT retained the complete scripted defect batch but blocked manual work because candidate or environment identity changed."
            )
            $identityException.Data["Sprint8AIdentityInvalidation"] = $true
            throw $identityException
        }
        if ($productDecisions.Count -gt 0) {
            $scriptedDisposition = Get-Sprint8AFormalUatScriptedDisposition `
                -Defects $scriptedDefects `
                -ProductDecisions $productDecisions
            $blockedAt = [DateTimeOffset]::UtcNow
            $blockedManual = @(Get-Sprint8AManualUatScenarioNames | ForEach-Object {
                [pscustomobject][ordered]@{
                    name = $_; state = "blocked"; authoritative = $false; diagnostic = $true
                    assertions_started = $false; assertions_started_at = $null
                    command = "manual scenario blocked before execution"
                    exit_status = $null; started_at = $null; ended_at = $blockedAt.ToString("o"); duration_ms = 0L
                    classification = "product-decision"; classification_source = "scripted_prerequisite"
                    failure_message = $null
                    blocked_reason = "blocked by unresolved scripted product decision: $($productDecisions.name -join ', ')"
                    evidence = @($productDecisions | ForEach-Object { @($_.evidence) })
                }
            })
            $attemptReceipt.checks = @($attemptReceipt.checks) + $blockedManual
            $attemptReceipt.failure_count = $scriptedDefects.Count
            $attemptReceipt.blocked_count = $productDecisions.Count + $blockedManual.Count
            $attemptReceipt.manual_scenarios_pending = @()
            $attemptReceipt.classification = $scriptedDisposition.classification
            $attemptReceipt.failure_batch = [pscustomobject][ordered]@{
                defect_count = $scriptedDefects.Count
                blocked_check_count = $attemptReceipt.blocked_count
                defects = $scriptedDefects
                blocked_checks = @($productDecisions) + $blockedManual
            }
            $productDecisionTerminalState = [string]$scriptedDisposition.terminal_state
            Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State $productDecisionTerminalState -Stage "scripted"
            $attemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            $attemptReceipt.duration_ms = [long][Math]::Max(
                0,
                ((ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.ended_at -Label "scripted failure end") -
                    (ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.started_at -Label "scripted failure start")).TotalMilliseconds
            )
            $attemptReceipt.cleanup_restoration.result = "not_proven"
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                phase = "uat-attempt"
                authoritative = $false
                status = $productDecisionTerminalState
            }) | Out-Null
            $productDecisionException = [InvalidOperationException]::new(
                "Formal UAT paused on an unresolved scripted product decision; manual scenarios are blocked pending user direction."
            )
            $productDecisionException.Data["Sprint8AProductDecision"] = $true
            throw $productDecisionException
        }
        if ($scriptedDefects.Count -gt 0) {
            $attemptReceipt.scripted_defects = $scriptedDefects
            $attemptReceipt.failure_batch = [pscustomobject][ordered]@{
                defect_count = $scriptedDefects.Count
                blocked_check_count = 0
                defects = $scriptedDefects
                blocked_checks = @()
            }
            $scriptedClassifications = @($scriptedDefects.classification | Sort-Object -Unique)
            $attemptReceipt.classification = if ($scriptedClassifications.Count -eq 1) { [string]$scriptedClassifications[0] } else { $null }
            Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "executing" -Stage "diagnostic-manual"
            $attemptReceipt.scripted_completed_at = [DateTimeOffset]::UtcNow.ToString("o")
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptCheckpointPath | Out-Null
            Sync-Sprint8AFormalUatEvidenceManifest -Overrides @(
                [pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                    phase = "uat-attempt"; authoritative = $false; status = "diagnostic-manual"
                },
                [pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($repoRoot, $attemptCheckpointPath).Replace("\", "/")
                    phase = "uat-checkpoint"; authoritative = $false; status = "diagnostic-manual"
                }
            ) | Out-Null
            $diagnosticException = [InvalidOperationException]::new(
                "Formal UAT scripted stage retained $($scriptedDefects.Count) defect(s). Execute every safe manual sibling as non-authoritative diagnostic harvest, record true dependents blocked, then Finalize this attempt."
            )
            $diagnosticException.Data["Sprint8ADiagnosticManualRequired"] = $true
            throw $diagnosticException
        }
        Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "executing" -Stage "manual"
        $attemptReceipt.scripted_completed_at = [DateTimeOffset]::UtcNow.ToString("o")
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptCheckpointPath | Out-Null
        Sync-Sprint8AFormalUatEvidenceManifest -Overrides @(
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                phase = "uat-attempt"; authoritative = $false; status = "awaiting-manual"
            },
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptCheckpointPath).Replace("\", "/")
                phase = "uat-checkpoint"; authoritative = $false; status = "awaiting-manual"
            }
        ) | Out-Null
        Write-Host "Formal Sprint 8A scripted UAT passed. Execute all eight manual scenarios, then rerun this attempt with -Stage Finalize."
        return
    } catch {
        $startFailure = $_
        if ($startFailure.Exception.Data["Sprint8ADiagnosticManualRequired"] -eq $true -or
            $startFailure.Exception.Data["Sprint8AProductDecision"] -eq $true -or
            $startFailure.Exception.Data["Sprint8AIdentityInvalidation"] -eq $true) {
            throw $startFailure
        }
        if ([string]$attemptReceipt.state -notin @("failed", "blocked")) {
            $failureClassification = Resolve-Sprint8AFormalUatFailureClassification `
                -DefaultClassification "preflight/setup" `
                -ErrorRecord $startFailure
            $failureEnded = [DateTimeOffset]::UtcNow
            $failureLogPath = Join-Path $logRoot "start-stage-failure.log"
            [IO.File]::WriteAllText(
                $failureLogPath,
                "[$($failureEnded.ToString('o'))] $($startFailure | Out-String)",
                [Text.UTF8Encoding]::new($false)
            )
            $failureEvidence = @([pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $failureLogPath).Replace("\", "/")
                sha256 = Get-Sprint8AFileSha256 -Path $failureLogPath
            })
            $existingNames = @($attemptReceipt.checks | ForEach-Object { [string]$_.name })
            $failureName = if ($existingNames -cnotcontains "authenticated-prerequisites") {
                "authenticated-prerequisites"
            } elseif ($existingNames -cnotcontains "scripted-inventory") {
                "scripted-inventory"
            } elseif ($existingNames -cnotcontains "scripted-smoke") {
                "scripted-smoke"
            } elseif ($existingNames -cnotcontains "post-scripted-identity") {
                "post-scripted-identity"
            } else { "checkpoint-publication" }
            $failureStarted = if ($null -ne (Get-Variable -Name prerequisiteStarted -ErrorAction SilentlyContinue)) {
                $prerequisiteStarted
            } else { ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.started_at -Label "formal UAT attempt start" }
            $failedCheck = [pscustomobject][ordered]@{
                name = $failureName
                command = "formal UAT Start prerequisite/scripted stage"
                state = "failed"
                authoritative = $false
                diagnostic = $true
                assertions_started = $false
                assertions_started_at = $null
                started_at = $failureStarted.ToString("o")
                ended_at = $failureEnded.ToString("o")
                duration_ms = [long][Math]::Max(0, ($failureEnded - $failureStarted).TotalMilliseconds)
                exit_status = 1
                classification = [string]$failureClassification.classification
                classification_source = [string]$failureClassification.source
                failure_message = $startFailure.Exception.Message
                blocked_reason = $null
                evidence = $failureEvidence
            }
            $terminalChecks = @($attemptReceipt.checks | Where-Object {
                [string]$_.name -cne $failureName
            }) + @($failedCheck)
            $blockedAt = [DateTimeOffset]::UtcNow.ToString("o")
            foreach ($declaration in @($attemptReceipt.declared_checks)) {
                if (@($terminalChecks | Where-Object name -CEQ ([string]$declaration.name)).Count -ne 0) { continue }
                $terminalChecks += [pscustomobject][ordered]@{
                    name = [string]$declaration.name
                    command = "blocked formal UAT check"
                    state = "blocked"
                    authoritative = $false
                    diagnostic = $true
                    assertions_started = $false
                    assertions_started_at = $null
                    started_at = $null
                    ended_at = $blockedAt
                    duration_ms = 0L
                    exit_status = $null
                    classification = $null
                    classification_source = $null
                    failure_message = $null
                    blocked_reason = "blocked by '$failureName': $($startFailure.Exception.Message)"
                    evidence = $failureEvidence
                }
            }
            $attemptReceipt.checks = $terminalChecks
            $attemptReceipt.assertion_count = @($terminalChecks | Where-Object assertions_started -EQ $true).Count
            Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "failed" -Stage ([string]$attemptReceipt.stage)
            $attemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            $attemptReceipt.duration_ms = [long][Math]::Max(
                0,
                ((ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.ended_at -Label "Start failure end") -
                    (ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.started_at -Label "Start failure start")).TotalMilliseconds
            )
            $attemptReceipt.cleanup_restoration.result = "not_proven"
            Merge-Sprint8AFormalUatFailureBatch `
                -AttemptReceipt $attemptReceipt `
                -Defects @([pscustomobject][ordered]@{
                    check = $failureName
                    classification = [string]$failureClassification.classification
                    classification_source = [string]$failureClassification.source
                    message = $startFailure.Exception.Message
                    evidence = $failureEvidence
                }) `
                -BlockedChecks @($terminalChecks | Where-Object state -CEQ "blocked" | ForEach-Object {
                    [pscustomobject][ordered]@{
                        check = [string]$_.name
                        dependency_reason = [string]$_.blocked_reason
                        evidence = @($_.evidence)
                    }
                }) | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            try {
                Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                    phase = "uat-attempt"; authoritative = $false; status = "failed"
                }) | Out-Null
            } catch {
                $manifestFailure = $_
                $manifestFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                    -AttemptReceipt $attemptReceipt `
                    -ErrorRecord $manifestFailure `
                    -Directory $attemptRoot `
                    -Classification "evidence-finalization" `
                    -ClassificationSource "manifest_publication_boundary"
                Merge-Sprint8AFormalUatFailureBatch `
                    -AttemptReceipt $attemptReceipt `
                    -Defects @([pscustomobject][ordered]@{
                        check = "evidence-manifest-publication"
                        classification = "evidence-finalization"
                        classification_source = "manifest_publication_boundary"
                        message = $manifestFailure.Exception.Message
                        evidence = @($manifestFailureReference)
                    }) | Out-Null
                Add-Sprint8AFormalUatManifestUpdateFailure `
                    -AttemptReceipt $attemptReceipt `
                    -Failure ([pscustomobject][ordered]@{
                    classification = "evidence-finalization"
                    classification_source = "manifest_publication_boundary"
                    message = $manifestFailure.Exception.Message
                    evidence = @($manifestFailureReference)
                    raw_evidence = @($manifestFailureReference.raw_evidence)
                }) | Out-Null
                Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            }
        } else {
            $manifestFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -ErrorRecord $startFailure `
                -Directory $attemptRoot `
                -Classification "evidence-finalization" `
                -ClassificationSource "manifest_publication_boundary"
            Merge-Sprint8AFormalUatFailureBatch `
                -AttemptReceipt $attemptReceipt `
                -Defects @([pscustomobject][ordered]@{
                    check = "evidence-manifest-publication"
                    classification = "evidence-finalization"
                    classification_source = "manifest_publication_boundary"
                    message = $startFailure.Exception.Message
                    evidence = @($manifestFailureReference)
                }) | Out-Null
            Add-Sprint8AFormalUatManifestUpdateFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure ([pscustomobject][ordered]@{
                classification = "evidence-finalization"
                classification_source = "manifest_publication_boundary"
                message = $startFailure.Exception.Message
                evidence = @($manifestFailureReference)
                raw_evidence = @($manifestFailureReference.raw_evidence)
            }) | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        }
        throw $startFailure
    }
    } finally {
        $attemptLock.Dispose()
    }
}

$attemptReceipt = $null
$attemptCheckpoint = $null
$attemptLock = Open-Sprint8AValidationAttemptLock `
    -Path (Join-Path $evidenceRootPath "validation-attempt.lock")
try {
Initialize-Sprint8AFormalUatAttemptDirectories `
    -AttemptRoot $attemptRoot `
    -AttemptPath $attemptPath `
    -LogRoot $logRoot
if (-not (Test-Path -LiteralPath $attemptPath -PathType Leaf)) {
    throw "Formal UAT finalization requires the retained Start-stage attempt receipt."
}
Assert-Sprint8AReceiptSidecar -Path $attemptPath | Out-Null
if (-not (Test-Path -LiteralPath $attemptCheckpointPath -PathType Leaf)) {
    throw "Formal UAT finalization requires the immutable awaiting-manual checkpoint."
}
$attemptCheckpointSha = Assert-Sprint8AReceiptSidecar -Path $attemptCheckpointPath
$attemptReceipt = Get-Content -LiteralPath $attemptPath -Raw | ConvertFrom-Json
$attemptCheckpoint = Get-Content -LiteralPath $attemptCheckpointPath -Raw | ConvertFrom-Json
if (-not $AuthorizeDisposableReset) {
    throw "Formal UAT Finalize requires -AuthorizeDisposableReset for source-exact canonical restoration."
}
$sourceAdvanceRecoveryAuthorized = $false
$sourceAdvanceRecoveryReference = $null
$reusableRestorationCheck = $null
$reusableRestorationChecks = @()
$sourceAdvanceFailureMessages = @(Get-Sprint8AFormalUatFailureMessages -Receipt $attemptReceipt)
$isExactSourceAdvanceFailure = [string]$attemptReceipt.state -ceq "failed" -and
    [string]$attemptReceipt.stage -ceq "manual" -and
    @($sourceAdvanceFailureMessages | Where-Object {
        $_ -ceq "Formal UAT source changed after the scripted stage."
    }).Count -eq 1 -and
    @($sourceAdvanceFailureMessages | Where-Object {
        $_ -match "Formal UAT catch harvest 'UAT-8A-01' start has no UTC offset"
    }).Count -eq 1 -and
    $sourceAdvanceFailureMessages.Count -eq 2 -and
    [int]$attemptReceipt.failure_batch.blocked_check_count -eq (Get-Sprint8AManualUatScenarioNames).Count
$isExactAggregateProjectionFailure = [string]$attemptReceipt.state -ceq "failed" -and
    [string]$attemptReceipt.stage -ceq "canonical-restoration" -and
    $sourceAdvanceFailureMessages.Count -eq 1 -and
    [string]$sourceAdvanceFailureMessages[0] -ceq
        "The property 'classification' cannot be found on this object. Verify that the property exists." -and
    [int]$attemptReceipt.failure_batch.blocked_check_count -eq 0 -and
    [string]$attemptReceipt.restoration_check.state -ceq "passed" -and
    @($attemptReceipt.restoration_checks).Count -eq 3 -and
    @($attemptReceipt.restoration_checks | Where-Object state -CNE "passed").Count -eq 0 -and
    [string]$attemptReceipt.cleanup_restoration.result -ceq "canonical_topology_verified"
if (($isExactSourceAdvanceFailure -or $isExactAggregateProjectionFailure) -and
    $AuthorizeUatHarnessOnlySourceAdvance) {
    if ($isExactAggregateProjectionFailure) {
        $reusableRestorationCheck = $attemptReceipt.restoration_check | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        $reusableRestorationChecks = @($attemptReceipt.restoration_checks | ForEach-Object {
            $_ | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        })
    }
    $currentSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $currentSource -RequireClean | Out-Null
    $incrementalSourceContext = Get-Sprint8AFormalUatSourceContext `
        -CurrentSource $currentSource `
        -CandidateSource $attemptReceipt.source_identity `
        -RepositoryRoot $repoRoot `
        -AllowHarnessOnlySourceAdvance
    $candidateSourceContext = Get-Sprint8AFormalUatSourceContext `
        -CurrentSource $currentSource `
        -CandidateSource $attemptReceipt.candidate_source_identity `
        -RepositoryRoot $repoRoot `
        -AllowHarnessOnlySourceAdvance
    $recoveryRoot = Join-Path $attemptRoot "source-advance-recoveries"
    [IO.Directory]::CreateDirectory($recoveryRoot) | Out-Null
    $recoveryNumbers = @(Get-ChildItem -LiteralPath $recoveryRoot -Directory -Filter "run-*" -ErrorAction SilentlyContinue | ForEach-Object {
        $number = 0
        if ([int]::TryParse($_.Name.Substring(4), [ref]$number)) { $number }
    })
    $recoveryNumber = if ($recoveryNumbers.Count -eq 0) { 1 } else {
        [int](($recoveryNumbers | Measure-Object -Maximum).Maximum) + 1
    }
    $recoveryRunRoot = Join-Path $recoveryRoot ("run-{0:d3}" -f $recoveryNumber)
    [IO.Directory]::CreateDirectory($recoveryRunRoot) | Out-Null
    $priorAttemptPath = Join-Path $recoveryRunRoot "prior-failed-attempt.json"
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $priorAttemptPath | Out-Null
    $priorAttemptReference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $priorAttemptPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $priorAttemptPath
    }
    $recoveryPath = Join-Path $recoveryRunRoot "source-advance-recovery.json"
    $recoveryDocument = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "uat-source-advance-recovery"
        state = "authorized"
        attempt = $Attempt
        authorized_at = [DateTimeOffset]::UtcNow.ToString("o")
        authorization = "user_directed_impact_scoped_validation"
        prior_failed_attempt = $priorAttemptReference
        prior_harness_source_identity = $attemptReceipt.source_identity
        corrected_harness_source_identity = $currentSource
        candidate_source_identity = $attemptReceipt.candidate_source_identity
        exact_changed_paths = @($incrementalSourceContext.harness_only_source_advance.exact_changed_paths)
        product_test_fixture_deployment_changes = $false
        upstream_gates_affected = $false
        manual_scenarios_reused = @(Get-Sprint8AManualUatScenarioNames)
        manual_assertions_reexecuted = $false
        canonical_restoration_reused = [bool]$isExactAggregateProjectionFailure
    }
    Publish-Sprint7AEvidence -Document $recoveryDocument -OutputPath $recoveryPath | Out-Null
    $sourceAdvanceRecoveryReference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $recoveryPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $recoveryPath
    }
    Sync-Sprint8AFormalUatEvidenceManifest -Overrides @(
        [pscustomobject][ordered]@{
            path = [string]$priorAttemptReference.path; sha256 = [string]$priorAttemptReference.sha256
            phase = "uat-source-advance-prior-failure"; authoritative = $false; status = "retained"
        },
        [pscustomobject][ordered]@{
            path = [string]$sourceAdvanceRecoveryReference.path; sha256 = [string]$sourceAdvanceRecoveryReference.sha256
            phase = "uat-source-advance-recovery"; authoritative = $false; status = "authorized"
        }
    ) | Out-Null
    $attemptReceipt = $attemptCheckpoint | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $attemptReceipt.source_identity = $currentSource
    $attemptReceipt.candidate_source_identity = $candidateSourceContext.candidate_source_identity
    $attemptReceipt.uat_harness_only_source_advance = $candidateSourceContext.harness_only_source_advance
    $attemptReceipt.source_verification_state = "verified_uat_harness_only_advance"
    $sourceAdvanceRecoveryAuthorized = $true
}
Repair-Sprint8AManualUatPreparedPublications `
    -Attempt $Attempt `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $evidenceRootPath `
    -AllowMissingCanonicalUatCommitment | Out-Null
if ([string]$attemptReceipt.state -ceq "passed" -and [string]$attemptReceipt.stage -ceq "result-committed") {
    try {
        if ($attemptReceipt.authoritative -isnot [bool] -or [bool]$attemptReceipt.authoritative -or
            [int]$attemptReceipt.attempt -ne $Attempt -or
            [string]$attemptReceipt.candidate_fingerprint -notmatch '^[0-9a-f]{64}$' -or
            [string]$attemptReceipt.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
            [string]::IsNullOrWhiteSpace([string]$attemptReceipt.result_commit_artifact.path) -or
            [string]$attemptReceipt.result_commit_artifact.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "Formal UAT committed attempt receipt is malformed."
        }
        $currentSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
        Assert-Sprint8ASourceIdentityObject -Source $currentSource -RequireClean | Out-Null
        if (-not (Test-Sprint8ASourceIdentityMatch -Expected $attemptReceipt.source_identity -Actual $currentSource)) {
            throw "Formal UAT source changed before committed-result publication retry."
        }
        $environment = Get-Sprint8AEnvironmentContract -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -ProbeDatabases
        if ([string]$environment.fingerprint -cne [string]$attemptReceipt.environment_fingerprint) {
            throw "Formal UAT environment changed before committed-result publication retry."
        }
        $endpoints = Get-Sprint8AFormalUatEndpoints -Environment $environment
        if (($endpoints | ConvertTo-Json -Compress) -cne ($attemptReceipt.endpoints | ConvertTo-Json -Compress)) {
            throw "Formal UAT endpoint identity changed before committed-result publication retry."
        }
        $preflightReference = Get-Sprint8AFormalUatReference -Path $PreflightReceipt
        $candidateReference = Get-Sprint8AFormalUatReference -Path $CandidateReceipt
        $sitReference = Get-Sprint8AFormalUatReference -Path $SitReceipt
        foreach ($binding in @(
            [pscustomobject]@{ reference = $preflightReference; label = "Preflight" },
            [pscustomobject]@{ reference = $candidateReference; label = "Candidate" },
            [pscustomobject]@{ reference = $sitReference; label = "SIT" }
        )) {
            Assert-Sprint8AFormalUatReceiptBinding `
                -Receipt $attemptReceipt `
                -ExpectedReference $binding.reference `
                -Label "Formal UAT committed attempt $($binding.label) binding"
        }
        $canonicalPrerequisites = @(@($preflightReference, $candidateReference, $sitReference) | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
        Assert-Sprint8ALifecyclePrerequisiteSet `
            -Phase "uat" `
            -References $canonicalPrerequisites `
            -Source $attemptReceipt.candidate_source_identity `
            -EnvironmentFingerprint ([string]$attemptReceipt.environment_fingerprint) `
            -NormalizedDeploymentConfigurationSha256 ([string]$environment.contract.compose.normalized_config_sha256) `
            -CandidateFingerprint ([string]$attemptReceipt.candidate_fingerprint) `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath | Out-Null
        $retrySitReceipt = Get-Content -LiteralPath ([string]$sitReference.full_path) -Raw | ConvertFrom-Json
        Assert-Sprint8AExactTerminalIdentities `
            -Results @($retrySitReceipt.checks) `
            -ExpectedNames (Get-Sprint8ASitLaneNames) `
            -Label "Committed-attempt SIT" | Out-Null
        if (@($retrySitReceipt.checks | Where-Object state -CNE "passed").Count -ne 0) {
            throw "Committed-attempt SIT receipt contains a failed or blocked terminal lane."
        }
        $commit = Assert-Sprint8AFormalUatResultCommit `
            -AttemptReceipt $attemptReceipt `
            -CommitPath ([string]$attemptReceipt.result_commit_artifact.path)
        $canonicalResult = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -Path $OutputPath
        if ([string]$canonicalResult.path -cne [string]$commit.document.result.path) {
            throw "Formal UAT retry output differs from the committed canonical result path."
        }
        $manifestPath = Join-Path $evidenceRootPath "evidence-manifest.json"
        if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
            throw "Formal UAT committed-result retry cannot recreate a missing lifecycle evidence manifest."
        }
        $canonicalPairState = Get-Sprint8AFormalUatCanonicalPairState `
            -Path ([string]$canonicalResult.full_path) `
            -ExpectedSha256 ([string]$commit.document.result.sha256)
        $commitOverrides = @(
            [pscustomobject][ordered]@{
                path = [string]$commit.document.result.path
                sha256 = [string]$commit.document.result.sha256
                phase = "uat"; authoritative = $true; status = "committed"
            },
            [pscustomobject][ordered]@{
                path = "$([string]$commit.document.result.path).sha256"
                sha256 = Get-Sprint8AStringSha256 -Text "$([string]$commit.document.result.sha256)`n"
                phase = "uat"; authoritative = $true; status = "committed"
            },
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                phase = "uat-attempt"; authoritative = $false; status = "passed"
            }
        )
        if ($attemptReceipt.PSObject.Properties.Name -contains "finalization_completion_checkpoint") {
            $commitOverrides += [pscustomobject][ordered]@{
                path = [string]$attemptReceipt.finalization_completion_checkpoint.path
                sha256 = [string]$attemptReceipt.finalization_completion_checkpoint.sha256
                phase = "uat-finalization-completion"; authoritative = $false; status = "complete"
            }
        }
        if ($canonicalPairState -ceq "absent") {
            Sync-Sprint8AFormalUatEvidenceManifest `
                -Overrides $commitOverrides `
                -AllowMissingCanonicalUatCommitment | Out-Null
        }
        Complete-Sprint8AFormalUatCanonicalPair `
            -AttemptReceipt $attemptReceipt `
            -Commit $commit `
            -ManifestPath $manifestPath `
            -CanonicalPath ([string]$canonicalResult.full_path) | Out-Null
        Write-Host "Formal Sprint 8A UAT committed-result publication retry passed."
        return
    } catch {
        $committedRetryFailure = $_
        $committedRetryClassification = Resolve-Sprint8AFormalUatFailureClassification `
            -DefaultClassification "evidence-finalization" `
            -ErrorRecord $committedRetryFailure
        $committedRetryFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
            -AttemptReceipt $attemptReceipt `
            -ErrorRecord $committedRetryFailure `
            -Directory $attemptRoot `
            -Classification ([string]$committedRetryClassification.classification) `
            -ClassificationSource ([string]$committedRetryClassification.source)
        $committedRetryFailureEntry = [pscustomobject][ordered]@{
            occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
            classification = [string]$committedRetryClassification.classification
            classification_source = [string]$committedRetryClassification.source
            message = $committedRetryFailure.Exception.Message
            evidence = @($committedRetryFailureReference)
        }
        Add-Sprint8AFormalUatEvidenceFinalizationFailure `
            -AttemptReceipt $attemptReceipt `
            -Failure $committedRetryFailureEntry `
            -Context "committed-result-retry" | Out-Null
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        try {
            Sync-Sprint8AFormalUatEvidenceManifest `
                -Overrides (Get-Sprint8AFormalUatFailureManifestOverrides `
                    -AttemptPath $attemptPath `
                    -FailureReference $committedRetryFailureReference `
                    -AttemptStatus "passed") | Out-Null
        } catch {
            $committedRetryManifestFailure = $_
            $committedRetryManifestReference = Publish-Sprint8AFormalUatFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -ErrorRecord $committedRetryManifestFailure `
                -Directory $attemptRoot `
                -Classification "evidence-finalization" `
                -ClassificationSource "manifest_publication_boundary"
            $committedRetryManifestEntry = [pscustomobject][ordered]@{
                occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                classification = "evidence-finalization"
                classification_source = "manifest_publication_boundary"
                message = $committedRetryManifestFailure.Exception.Message
                evidence = @($committedRetryManifestReference)
                raw_evidence = @($committedRetryManifestReference.raw_evidence)
            }
            Add-Sprint8AFormalUatEvidenceFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $committedRetryManifestEntry `
                -Context "committed-result-retry-manifest" | Out-Null
            Add-Sprint8AFormalUatManifestUpdateFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $committedRetryManifestEntry | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        }
        $committedRetryFailure.Exception.Data["Sprint8AFormalUatFailureRetained"] = $true
        throw $committedRetryFailure
    }
}
if ($attemptReceipt.PSObject.Properties.Name -notcontains "finalization_completion_checkpoint" -and
    $attemptReceipt.PSObject.Properties.Name -contains "finalization_run" -and
    [string]$attemptReceipt.state -ceq "finalizing" -and
    [string]$attemptReceipt.stage -ceq "canonical-restoration" -and
    ($attemptReceipt.finalization_run -is [int] -or $attemptReceipt.finalization_run -is [long])) {
    $orphanCompletionPath = Join-Path $attemptRoot (
        "finalizations/run-{0:d3}/finalization-completion-checkpoint.json" -f [int]$attemptReceipt.finalization_run
    )
    if ((Test-Path -LiteralPath $orphanCompletionPath -PathType Leaf) -or
        (Test-Path -LiteralPath "$orphanCompletionPath.sha256" -PathType Leaf) -or
        (Test-Path -LiteralPath "$orphanCompletionPath.publish-journal.json" -PathType Leaf)) {
        Repair-Sprint7AEvidencePublication -Path $orphanCompletionPath
        $orphanCompletionReference = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $orphanCompletionPath).Replace("\", "/")
            sha256 = Assert-Sprint8AReceiptSidecar -Path $orphanCompletionPath
        }
        $attemptReceipt | Add-Member `
            -Force `
            -NotePropertyName finalization_completion_checkpoint `
            -NotePropertyValue $orphanCompletionReference
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
    }
}
$hasFinalizationCompletionCheckpoint =
    $attemptReceipt.PSObject.Properties.Name -contains "finalization_completion_checkpoint"
if ($hasFinalizationCompletionCheckpoint -and
    [string]$attemptReceipt.state -ceq "finalizing" -and
    [string]$attemptReceipt.stage -ceq "canonical-restoration") {
    try {
        $completionCheckpoint = Assert-Sprint8AFormalUatFinalizationCompletionCheckpoint `
            -AttemptReceipt $attemptReceipt `
            -AttemptCheckpoint $attemptCheckpoint `
            -AttemptCheckpointSha256 $attemptCheckpointSha
        if ($attemptReceipt.PSObject.Properties.Name -contains "finalization_retry" -and
            [bool]$attemptReceipt.finalization_retry.eligible) {
            $retryReference = Get-Sprint8AFormalUatReference `
                -Path ([string]$attemptReceipt.finalization_retry.checkpoint.path)
            if ([string]$retryReference.sha256 -cne [string]$attemptReceipt.finalization_retry.checkpoint.sha256) {
                throw "Formal UAT finalization-retry checkpoint digest differs from its attempt reference."
            }
            $retryDocument = Get-Content -LiteralPath ([string]$retryReference.full_path) -Raw | ConvertFrom-Json
            if (($retryDocument.schema_version -isnot [int] -and $retryDocument.schema_version -isnot [long]) -or
                [int]$retryDocument.schema_version -ne 1 -or [string]$retryDocument.sprint -cne "sprint-8a" -or
                [string]$retryDocument.phase -cne "uat-finalization-retry" -or [string]$retryDocument.state -cne "eligible" -or
                [int]$retryDocument.attempt -ne $Attempt -or
                [string]$retryDocument.candidate_fingerprint -cne [string]$attemptReceipt.candidate_fingerprint -or
                [string]$retryDocument.environment_fingerprint -cne [string]$attemptReceipt.environment_fingerprint -or
                [string]$retryDocument.completion_checkpoint.path -cne [string]$completionCheckpoint.reference.path -or
                [string]$retryDocument.completion_checkpoint.sha256 -cne [string]$completionCheckpoint.reference.sha256) {
                throw "Formal UAT finalization-retry checkpoint is malformed or bound to another completion boundary."
            }
        }
        Assert-Sprint8ANoOpenManualUatLeases `
            -Attempt $Attempt `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath
        $currentSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
        Assert-Sprint8ASourceIdentityObject -Source $currentSource -RequireClean | Out-Null
        if (-not (Test-Sprint8ASourceIdentityMatch -Expected $attemptReceipt.source_identity -Actual $currentSource)) {
            throw "Formal UAT source changed before finalization-only retry."
        }
        $environment = Get-Sprint8AEnvironmentContract `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -ProbeDatabases
        if ([string]$environment.fingerprint -cne [string]$attemptReceipt.environment_fingerprint -or
            [string]$environment.contract.compose.normalized_config_sha256 -cne
                [string]$completionCheckpoint.document.normalized_deployment_configuration_sha256) {
            throw "Formal UAT environment or normalized deployment identity changed before finalization-only retry."
        }
        $endpoints = Get-Sprint8AFormalUatEndpoints -Environment $environment
        if (($endpoints | ConvertTo-Json -Compress) -cne ($completionCheckpoint.document.endpoints | ConvertTo-Json -Compress)) {
            throw "Formal UAT endpoint identity changed before finalization-only retry."
        }
        $preflightReference = Get-Sprint8AFormalUatReference -Path $PreflightReceipt
        $candidateReference = Get-Sprint8AFormalUatReference -Path $CandidateReceipt
        $sitReference = Get-Sprint8AFormalUatReference -Path $SitReceipt
        $currentPrerequisites = @(@($preflightReference, $candidateReference, $sitReference) | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
        if (($currentPrerequisites | ConvertTo-Json -Depth 10 -Compress) -cne
            (@($completionCheckpoint.document.prerequisite_receipts) | ConvertTo-Json -Depth 10 -Compress)) {
            throw "Formal UAT prerequisite receipts changed before finalization-only retry."
        }
        Assert-Sprint8ALifecyclePrerequisiteSet `
            -Phase "uat" `
            -References $currentPrerequisites `
            -Source $attemptReceipt.candidate_source_identity `
            -EnvironmentFingerprint ([string]$attemptReceipt.environment_fingerprint) `
            -NormalizedDeploymentConfigurationSha256 ([string]$environment.contract.compose.normalized_config_sha256) `
            -CandidateFingerprint ([string]$attemptReceipt.candidate_fingerprint) `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath | Out-Null
        $retrySitReceipt = Get-Content -LiteralPath ([string]$sitReference.full_path) -Raw | ConvertFrom-Json
        Assert-Sprint8AExactTerminalIdentities `
            -Results @($retrySitReceipt.checks) `
            -ExpectedNames (Get-Sprint8ASitLaneNames) `
            -Label "Finalization-only retry SIT" | Out-Null
        if (@($retrySitReceipt.checks | Where-Object state -CNE "passed").Count -ne 0) {
            throw "Finalization-only retry SIT receipt contains a failed or blocked lane."
        }
        $canonicalRetryTarget = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -Path $OutputPath
        if ((Get-Sprint8AFormalUatCanonicalPairState `
            -Path ([string]$canonicalRetryTarget.full_path) `
            -ExpectedSha256 ("0" * 64)) -cne "absent") {
            throw "A non-committed formal UAT attempt cannot resume after canonical publication began."
        }
        $retryRoot = Join-Path $attemptRoot "publication-retries"
        [IO.Directory]::CreateDirectory($retryRoot) | Out-Null
        $retryNumbers = @(Get-ChildItem -LiteralPath $retryRoot -Directory -Filter "run-*" -ErrorAction SilentlyContinue | ForEach-Object {
            $number = 0
            if ([int]::TryParse($_.Name.Substring(4), [ref]$number)) { $number }
        })
        $retryNumber = if ($retryNumbers.Count -eq 0) { 1 } else {
            [int](($retryNumbers | Measure-Object -Maximum).Maximum) + 1
        }
        $retryRunRoot = Join-Path $retryRoot ("run-{0:d3}" -f $retryNumber)
        [IO.Directory]::CreateDirectory($retryRunRoot) | Out-Null
        Invoke-Sprint8AFormalUatPublicationTail `
            -AttemptReceipt $attemptReceipt `
            -CompletionCheckpoint $completionCheckpoint `
            -CurrentSource $currentSource `
            -AttemptPath $attemptPath `
            -ResultCommitPath (Join-Path $retryRunRoot "result-commit.json") `
            -OutputPath $OutputPath | Out-Null
        Write-Host "Formal Sprint 8A UAT finalization-only publication retry passed without re-running manual validation or restoration."
        return
    } catch {
        $retryFailure = $_
        $retryFailureClassification = Resolve-Sprint8AFormalUatFailureClassification `
            -DefaultClassification "preflight/setup" `
            -ErrorRecord $retryFailure
        $retryFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
            -AttemptReceipt $attemptReceipt `
            -ErrorRecord $retryFailure `
            -Directory $attemptRoot `
            -Classification ([string]$retryFailureClassification.classification) `
            -ClassificationSource ([string]$retryFailureClassification.source)
        $retryFailureEntry = [pscustomobject][ordered]@{
            occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
            classification = [string]$retryFailureClassification.classification
            classification_source = [string]$retryFailureClassification.source
            message = $retryFailure.Exception.Message
            evidence = @($retryFailureReference)
        }
        Add-Sprint8AFormalUatFinalizationRetryFailure `
            -AttemptReceipt $attemptReceipt `
            -Failure $retryFailureEntry | Out-Null
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        try {
            Sync-Sprint8AFormalUatEvidenceManifest `
                -Overrides (Get-Sprint8AFormalUatFailureManifestOverrides `
                    -AttemptPath $attemptPath `
                    -FailureReference $retryFailureReference `
                    -AttemptStatus "finalization-retry-failed") | Out-Null
        } catch {
            $retryManifestFailure = $_
            $retryManifestFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -ErrorRecord $retryManifestFailure `
                -Directory $attemptRoot `
                -Classification "evidence-finalization" `
                -ClassificationSource "manifest_publication_boundary"
            $retryManifestFailureEntry = [pscustomobject][ordered]@{
                occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                classification = "evidence-finalization"
                classification_source = "manifest_publication_boundary"
                message = $retryManifestFailure.Exception.Message
                evidence = @($retryManifestFailureReference)
            }
            Add-Sprint8AFormalUatFinalizationRetryFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $retryManifestFailureEntry | Out-Null
            Add-Sprint8AFormalUatManifestUpdateFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $retryManifestFailureEntry | Out-Null
            $attemptReceipt.finalization_retry | Add-Member `
                -Force `
                -NotePropertyName manifest_update_failure `
                -NotePropertyValue $retryManifestFailureEntry
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        }
        $retryFailure.Exception.Data["Sprint8AFinalizationOnlyRetryFailure"] = $true
        throw $retryFailure
    }
}
$attemptMatchesCheckpoint = ($attemptReceipt | ConvertTo-Json -Depth 50 -Compress) -ceq
    ($attemptCheckpoint | ConvertTo-Json -Depth 50 -Compress)
$resumedAttemptReference = $sourceAdvanceRecoveryReference
if (-not $attemptMatchesCheckpoint -and -not $sourceAdvanceRecoveryAuthorized) {
    $resumableFinalization = [string]$attemptReceipt.state -ceq "finalizing" -and
        [string]$attemptReceipt.stage -in @("manual-receipt-validation", "canonical-restoration") -and
        [int]$attemptReceipt.attempt -eq [int]$attemptCheckpoint.attempt -and
        [string]$attemptReceipt.candidate_fingerprint -ceq [string]$attemptCheckpoint.candidate_fingerprint -and
        [string]$attemptReceipt.environment_fingerprint -ceq [string]$attemptCheckpoint.environment_fingerprint -and
        ($attemptReceipt.source_identity | ConvertTo-Json -Depth 20 -Compress) -ceq
            ($attemptCheckpoint.source_identity | ConvertTo-Json -Depth 20 -Compress) -and
        ($attemptReceipt.prerequisite_receipts | ConvertTo-Json -Depth 20 -Compress) -ceq
            ($attemptCheckpoint.prerequisite_receipts | ConvertTo-Json -Depth 20 -Compress)
    if (-not $resumableFinalization) {
        throw "Formal UAT mutable attempt is neither the awaiting-manual checkpoint nor a resumable finalization checkpoint."
    }
}
if ($attemptMatchesCheckpoint -or $sourceAdvanceRecoveryAuthorized) {
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -ManifestPath (Join-Path $evidenceRootPath "evidence-manifest.json") | Out-Null
}
$finalizationsRoot = Join-Path $attemptRoot "finalizations"
[IO.Directory]::CreateDirectory($finalizationsRoot) | Out-Null
$existingFinalizationNumbers = @(Get-ChildItem -LiteralPath $finalizationsRoot -Directory -Filter "run-*" -ErrorAction SilentlyContinue | ForEach-Object {
    $number = 0
    if ([int]::TryParse($_.Name.Substring(4), [ref]$number)) { $number }
})
$finalizationRunNumber = if ($existingFinalizationNumbers.Count -eq 0) {
    1
} else {
    [int](($existingFinalizationNumbers | Measure-Object -Maximum).Maximum) + 1
}
$finalizationRoot = Join-Path $finalizationsRoot ("run-{0:d3}" -f $finalizationRunNumber)
[IO.Directory]::CreateDirectory($finalizationRoot) | Out-Null
$logRoot = Join-Path $finalizationRoot "logs"
[IO.Directory]::CreateDirectory($logRoot) | Out-Null
if (-not $attemptMatchesCheckpoint -and -not $sourceAdvanceRecoveryAuthorized) {
    $resumedAttemptPath = Join-Path $finalizationRoot "resumed-attempt.json"
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $resumedAttemptPath | Out-Null
    $resumedAttemptReference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $resumedAttemptPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $resumedAttemptPath
    }
    $attemptReceipt = $attemptCheckpoint | ConvertTo-Json -Depth 50 | ConvertFrom-Json
}
$finalInventoryPath = Join-Path $finalizationRoot "final-inventory.json"
$finalSmokePath = Join-Path $finalizationRoot "final-smoke.json"
$restorationRoot = Join-Path $finalizationRoot "restoration"
$restorationSummaryPath = Join-Path $restorationRoot "materialization/attempt-$Attempt/materialization-result.json"
$resultCommitPath = Join-Path $finalizationRoot "result-commit.json"
$attemptReceipt | Add-Member -Force -NotePropertyName finalization_run -NotePropertyValue $finalizationRunNumber
$attemptReceipt | Add-Member -Force -NotePropertyName resumed_attempt -NotePropertyValue $resumedAttemptReference
if ([string]$attemptReceipt.state -cne "executing" -or
    [string]$attemptReceipt.stage -notin @("manual", "diagnostic-manual") -or
    [int]$attemptReceipt.attempt -ne $Attempt -or
    [string]$attemptReceipt.candidate_fingerprint -notmatch '^[0-9a-f]{64}$' -or
    [string]$attemptReceipt.environment_fingerprint -notmatch '^[0-9a-f]{64}$') {
    throw "Formal UAT attempt is not an exact executing/manual checkpoint."
}
$candidateFingerprint = [string]$attemptReceipt.candidate_fingerprint
$environmentFingerprint = [string]$attemptReceipt.environment_fingerprint
$currentSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
Assert-Sprint8ASourceIdentityObject -Source $currentSource -RequireClean | Out-Null
if (-not (Test-Sprint8ASourceIdentityMatch -Expected $attemptReceipt.source_identity -Actual $currentSource)) {
    throw "Formal UAT source changed after the scripted stage."
}
$environment = Get-Sprint8AEnvironmentContract -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -ProbeDatabases
if ([string]$environment.fingerprint -cne $environmentFingerprint) {
    throw "Formal UAT environment changed after the scripted stage."
}
$endpoints = Get-Sprint8AFormalUatEndpoints -Environment $environment
if (($endpoints | ConvertTo-Json -Compress) -cne ($attemptReceipt.endpoints | ConvertTo-Json -Compress)) {
    throw "Formal UAT endpoint identity changed after the scripted stage."
}
Assert-Sprint8ANoOpenManualUatLeases `
    -Attempt $Attempt `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $evidenceRootPath
Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "finalizing" -Stage "manual-receipt-validation"
Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
$finalizationStartOverrides = @([pscustomobject][ordered]@{
    path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
    phase = "uat-attempt"; authoritative = $false; status = "finalizing"
})
if ($null -ne $resumedAttemptReference) {
    $finalizationStartOverrides += [pscustomobject][ordered]@{
        path = [string]$resumedAttemptReference.path
        sha256 = [string]$resumedAttemptReference.sha256
        phase = "uat-finalization-resume"; authoritative = $false; status = "retained"
    }
}
Sync-Sprint8AFormalUatEvidenceManifest -Overrides $finalizationStartOverrides | Out-Null
$manualReferences = [Collections.Generic.List[object]]::new()
$manualEvidenceReferences = [Collections.Generic.List[object]]::new()
$manualManifestEntries = [Collections.Generic.List[object]]::new()
$manualChecks = [Collections.Generic.List[object]]::new()
$manualDefects = [Collections.Generic.List[object]]::new()
if ($attemptReceipt.PSObject.Properties.Name -contains "scripted_defects") {
    foreach ($defect in @($attemptReceipt.scripted_defects)) { $manualDefects.Add($defect) }
}
$scriptedCompletedAt = ConvertTo-Sprint8ADateTimeOffset `
    -Value $attemptReceipt.scripted_completed_at `
    -Label "formal UAT scripted completion"
foreach ($scenario in Get-Sprint8AManualUatScenarioNames) {
    $validationStarted = [DateTimeOffset]::UtcNow
    $manualFullPath = Join-Path $manualReceiptRoot "$($scenario.ToLowerInvariant()).json"
    $manualPath = [IO.Path]::GetRelativePath($repoRoot, $manualFullPath).Replace("\", "/")
    try {
        $reference = Get-Sprint8AFormalUatReference -Path $manualPath
        $receipt = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
        Assert-Sprint8AManualUatReceipt `
            -Receipt $receipt `
            -ExpectedScenario $scenario `
            -ExpectedAttempt $Attempt `
            -CandidateFingerprint $candidateFingerprint `
            -EnvironmentFingerprint $environmentFingerprint `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath | Out-Null
        Assert-Sprint8AManualUatExecutionLeasePair `
            -Receipt $receipt `
            -ReceiptReference $reference `
            -ExpectedScenario $scenario `
            -ExpectedAttempt $Attempt `
            -CandidateFingerprint $candidateFingerprint `
            -EnvironmentFingerprint $environmentFingerprint `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath | Out-Null
        $expectedCheckpointReference = [pscustomobject]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $attemptCheckpointPath).Replace("\", "/")
            sha256 = $attemptCheckpointSha
        }
        if ([string]$receipt.start_checkpoint.path -cne [string]$expectedCheckpointReference.path -or
            [string]$receipt.start_checkpoint.sha256 -cne [string]$expectedCheckpointReference.sha256) {
            throw "Manual UAT scenario '$scenario' names another Start checkpoint."
        }
        if ((ConvertTo-Sprint8ADateTimeOffset -Value $receipt.started_at -Label "$scenario start") -lt $scriptedCompletedAt) {
            throw "Manual UAT scenario '$scenario' predates the scripted UAT checkpoint."
        }
        $scenarioEvidence = @(Get-Sprint8AFormalUatEvidenceReferences -Evidence @($receipt.evidence) -Scenario $scenario)
        $cleanupEvidence = @(Get-Sprint8AFormalUatEvidenceReferences -Evidence @($receipt.cleanup_restoration.evidence) -Scenario $scenario)
        $manualReferences.Add([pscustomobject][ordered]@{
            path = [string]$reference.path
            sha256 = [string]$reference.sha256
        })
        $manualManifestEntries.Add([pscustomobject]@{
            path = [string]$reference.path
            sha256 = [string]$reference.sha256
            phase = "uat-manual"
            authoritative = [bool]$receipt.authoritative
            status = if ([bool]$receipt.diagnostic) { "diagnostic-$($receipt.state)" } else { [string]$receipt.state }
        })
        foreach ($evidence in @($scenarioEvidence) + @($cleanupEvidence)) {
            $manualEvidenceReferences.Add([pscustomobject]@{
                path = [string]$evidence.path
                sha256 = [string]$evidence.sha256
                phase = "uat-manual-evidence"
                authoritative = [bool]$receipt.authoritative
                status = "retained"
            })
        }
        $check = [pscustomobject][ordered]@{
            name = $scenario
            state = [string]$receipt.state
            authoritative = [bool]$receipt.authoritative
            diagnostic = [bool]$receipt.diagnostic
            assertions_started = [bool]$receipt.assertions_started
            assertions_started_at = if ([bool]$receipt.assertions_started) { [string]$receipt.assertions_started_at } else { $null }
            command = "manual scenario receipt $($reference.path)"
            exit_status = if ([string]$receipt.state -ceq "passed") { 0 } elseif ([string]$receipt.state -ceq "failed") { 1 } else { $null }
            started_at = [string]$receipt.started_at
            ended_at = [string]$receipt.ended_at
            duration_ms = [long]$receipt.duration_ms
            classification = if ([string]$receipt.state -in @("failed", "blocked")) { [string]$receipt.classification } else { $null }
            classification_source = if ([string]$receipt.state -in @("failed", "blocked")) { [string]$receipt.classification_source } else { $null }
            failure_message = if ([string]$receipt.state -ceq "failed") { [string]$receipt.failure_message } else { $null }
            blocked_reason = if ([string]$receipt.state -ceq "blocked") { [string]$receipt.blocked_reason } else { $null }
            evidence = @([pscustomobject][ordered]@{
                path = [string]$reference.path
                sha256 = [string]$reference.sha256
            }) + $scenarioEvidence + $cleanupEvidence
        }
        $manualChecks.Add($check)
        if ([string]$check.state -ceq "failed") {
            $manualDefects.Add([pscustomobject][ordered]@{
                check = $scenario
                classification = [string]$check.classification
                classification_source = [string]$check.classification_source
                message = [string]$check.failure_message
                evidence = @($check.evidence)
            })
        }
    } catch {
        $validationEnded = [DateTimeOffset]::UtcNow
        $classification = Resolve-Sprint8AFormalUatFailureClassification `
            -DefaultClassification "harness" `
            -ErrorRecord $_
        $validationLog = Join-Path $logRoot "manual-$($scenario.ToLowerInvariant())-validation.log"
        [IO.File]::WriteAllText(
            $validationLog,
            "[$($validationEnded.ToString('o'))] $($_ | Out-String)",
            [Text.UTF8Encoding]::new($false)
        )
        $rawEvidence = @([pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $validationLog).Replace("\", "/")
            sha256 = Get-Sprint8AFileSha256 -Path $validationLog
        })
        $resolvedManualPath = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -Path $manualPath
        if (Test-Path -LiteralPath ([string]$resolvedManualPath.full_path) -PathType Leaf) {
            $rawEvidence += [pscustomobject][ordered]@{
                path = [string]$resolvedManualPath.path
                sha256 = Get-Sprint8AFileSha256 -Path ([string]$resolvedManualPath.full_path)
            }
        }
        $check = [pscustomobject][ordered]@{
            name = $scenario
            state = "failed"
            authoritative = $false
            diagnostic = $true
            assertions_started = $true
            assertions_started_at = $validationStarted.ToString("o")
            command = "validate retained manual scenario receipt $manualPath"
            exit_status = 1
            started_at = $validationStarted.ToString("o")
            ended_at = $validationEnded.ToString("o")
            duration_ms = [long][Math]::Max(0, ($validationEnded - $validationStarted).TotalMilliseconds)
            classification = [string]$classification.classification
            classification_source = [string]$classification.source
            failure_message = $_.Exception.Message
            blocked_reason = $null
            evidence = $rawEvidence
        }
        $manualChecks.Add($check)
        $manualDefects.Add([pscustomobject][ordered]@{
            check = $scenario
            classification = [string]$check.classification
            classification_source = [string]$check.classification_source
            message = [string]$check.failure_message
            evidence = @($check.evidence)
        })
    }
}
Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State "finalizing" -Stage "canonical-restoration"
Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
    path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
    phase = "uat-attempt"; authoritative = $false; status = "canonical-restoration"
}) | Out-Null
if ($null -ne $reusableRestorationCheck) {
    $restorationCheck = $reusableRestorationCheck
    $restorationChecks = @($reusableRestorationChecks)
    Assert-Sprint8AExactTerminalIdentities `
        -Results $restorationChecks `
        -ExpectedNames @(
            "canonical-restoration-materialization",
            "canonical-restoration-inventory",
            "canonical-restoration-smoke"
        ) `
        -Label "Formal UAT reusable canonical restoration" | Out-Null
    if ([string]$restorationCheck.state -cne "passed" -or
        @($restorationChecks | Where-Object state -CNE "passed").Count -ne 0) {
        throw "Formal UAT aggregate-only retry cannot reuse a nonpassing canonical restoration."
    }
    foreach ($evidence in @($restorationCheck.evidence) + @($restorationChecks | ForEach-Object { @($_.evidence) })) {
        $resolvedEvidence = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -Path ([string]$evidence.path)
        if ((Get-Sprint8AFileSha256 -Path ([string]$resolvedEvidence.full_path)) -cne [string]$evidence.sha256) {
            throw "Formal UAT aggregate-only retry found changed canonical-restoration evidence '$([string]$evidence.path)'."
        }
    }
} else {
$materializationCheck = Invoke-Sprint8AFormalUatCheck `
    -Name "canonical-restoration-materialization" `
    -Command ".\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot '$restorationRoot' -EnvironmentFingerprint '$environmentFingerprint' -AuthorizeDisposableReset -Confirm:`$false -SkipBuild -VerifyNoOp" `
    -LogPath (Join-Path $logRoot "canonical-restoration-materialization.log") `
    -EvidencePaths @($restorationSummaryPath) `
    -DefaultClassification "environment" `
    -Action {
        & (Join-Path $repoRoot "scripts/materialize-sprint-8a.ps1") `
            -Attempt $Attempt `
            -EvidenceRoot $restorationRoot `
            -EnvironmentFingerprint $environmentFingerprint `
            -AuthorizeDisposableReset `
            -Confirm:$false `
            -SkipBuild `
            -VerifyNoOp `
            -CoreUrl $endpoints.gateway `
            -ControlUrl $endpoints.materialization_control `
            -SupervisorUrl $endpoints.supervisor
        if (-not $?) { throw "Canonical from-empty materialization/no-op restoration failed." }
    }
$restorationChecks = @($materializationCheck)
if ([string]$materializationCheck.state -ceq "passed") {
    $restorationChecks += Invoke-Sprint8AFormalUatCheck `
        -Name "canonical-restoration-inventory" `
        -Command ".\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl '$($endpoints.gateway)' -OutputPath '$finalInventoryPath'" `
        -LogPath (Join-Path $logRoot "canonical-restoration-inventory.log") `
        -EvidencePaths @($finalInventoryPath) `
        -DefaultClassification "product" `
        -Action {
            & (Join-Path $repoRoot "scripts/audit-sprint-8a-deployed-inventory.ps1") `
                -BaseUrl $endpoints.gateway `
                -OutputPath $finalInventoryPath
            if (-not $?) { throw "Canonical restored inventory/navigation audit failed." }
        }
    $restorationChecks += Invoke-Sprint8AFormalUatCheck `
        -Name "canonical-restoration-smoke" `
        -Command ".\scripts\smoke-sprint-8a.ps1 -BaseUrl '$($endpoints.gateway)' -SupervisorUrl '$($endpoints.supervisor)' -OutputPath '$finalSmokePath'" `
        -LogPath (Join-Path $logRoot "canonical-restoration-smoke.log") `
        -EvidencePaths @($finalSmokePath) `
        -DefaultClassification "product" `
        -Action {
            & (Join-Path $repoRoot "scripts/smoke-sprint-8a.ps1") `
                -BaseUrl $endpoints.gateway `
                -SupervisorUrl $endpoints.supervisor `
                -OutputPath $finalSmokePath
            if (-not $?) { throw "Canonical restored Sprint 8A smoke failed." }
        }
} else {
    $blockedAt = [DateTimeOffset]::UtcNow
    foreach ($blockedSibling in @(
        [pscustomobject]@{ name = "canonical-restoration-inventory"; command = "canonical restored inventory/navigation audit" },
        [pscustomobject]@{ name = "canonical-restoration-smoke"; command = "canonical restored Sprint 8A smoke" }
    )) {
        $restorationChecks += [pscustomobject][ordered]@{
            name = [string]$blockedSibling.name
            command = [string]$blockedSibling.command
            state = "blocked"
            assertions_started = $false
            assertions_started_at = $null
            started_at = $null
            ended_at = $blockedAt.ToString("o")
            duration_ms = 0L
            exit_status = $null
            failure_message = $null
            classification = $null
            classification_source = $null
            blocked_reason = "blocked because canonical-restoration-materialization did not pass"
            evidence = @($materializationCheck.evidence)
        }
    }
}
Assert-Sprint8AExactTerminalIdentities `
    -Results $restorationChecks `
    -ExpectedNames @(
        "canonical-restoration-materialization",
        "canonical-restoration-inventory",
        "canonical-restoration-smoke"
    ) `
    -Label "Formal UAT canonical restoration" | Out-Null
$restorationFailures = @($restorationChecks | Where-Object state -CEQ "failed")
$restorationBlocked = @($restorationChecks | Where-Object state -CEQ "blocked")
$restorationState = if ($restorationFailures.Count -gt 0) {
    "failed"
} elseif ($restorationBlocked.Count -gt 0) {
    "blocked"
} else { "passed" }
$restorationStartedChecks = @($restorationChecks | Where-Object assertions_started -EQ $true)
$restorationStarted = @($restorationStartedChecks | ForEach-Object {
    ConvertTo-Sprint8ADateTimeOffset -Value $_.started_at -Label "scripted check start"
} | Sort-Object | Select-Object -First 1)[0]
$restorationEnded = @($restorationChecks | ForEach-Object {
    ConvertTo-Sprint8ADateTimeOffset -Value $_.ended_at -Label "scripted check end"
} | Sort-Object | Select-Object -Last 1)[0]
$restorationNonpassing = @($restorationChecks | Where-Object state -CNE "passed")
$restorationClassifications = @($restorationNonpassing | ForEach-Object {
    [string]$_.classification
} | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Sort-Object -Unique)
$restorationCheck = [pscustomobject][ordered]@{
    name = "canonical-restoration"
    state = $restorationState
    assertions_started = $restorationStartedChecks.Count -gt 0
    assertions_started_at = if ($restorationStartedChecks.Count -gt 0) { $restorationStarted.ToString("o") } else { $null }
    command = "source-exact materialization prerequisite followed by fail-late inventory and smoke siblings"
    exit_status = if ($restorationState -ceq "passed") { 0 } elseif ($restorationState -ceq "failed") { 1 } else { $null }
    started_at = if ($restorationStartedChecks.Count -gt 0) { $restorationStarted.ToString("o") } else { $null }
    ended_at = $restorationEnded.ToString("o")
    duration_ms = if ($restorationStartedChecks.Count -gt 0) {
        [long][Math]::Max(0, ($restorationEnded - $restorationStarted).TotalMilliseconds)
    } else { 0L }
    classification = if ($restorationClassifications.Count -eq 1) { [string]$restorationClassifications[0] } else { $null }
    classification_source = if ($restorationNonpassing.Count -eq 1) {
        [string]$restorationNonpassing[0].classification_source
    } elseif ($restorationNonpassing.Count -gt 1) { "consolidated_restoration_checks" } else { $null }
    failure_message = if ($restorationFailures.Count -gt 0) {
        ($restorationFailures.failure_message -join "; ")
    } else { $null }
    blocked_reason = if ($restorationState -ceq "blocked") {
        ($restorationBlocked.blocked_reason -join "; ")
    } else { $null }
    evidence = @($restorationChecks | ForEach-Object { @($_.evidence) } | Group-Object {
        "$([string]$_.path)`u{001f}$([string]$_.sha256)"
    } | ForEach-Object { $_.Group[0] })
}
}
$rawScriptedChecks = @($attemptReceipt.checks | Where-Object {
    [string]$_.name -in @("scripted-inventory", "scripted-smoke")
})
$scriptedStarted = @($rawScriptedChecks | ForEach-Object {
    ConvertTo-Sprint8ADateTimeOffset -Value $_.started_at -Label "restoration check start"
} | Sort-Object | Select-Object -First 1)[0]
$scriptedEnded = @($rawScriptedChecks | ForEach-Object {
    ConvertTo-Sprint8ADateTimeOffset -Value $_.ended_at -Label "restoration check end"
} | Sort-Object | Select-Object -Last 1)[0]
$scriptedFailures = @($rawScriptedChecks | Where-Object state -CEQ "failed")
$scriptedClassifications = @(Get-Sprint8AFormalUatFailureClassifications -Failures $scriptedFailures)
$scriptedCheck = [pscustomobject][ordered]@{
    name = "scripted-uat"
    state = if ($scriptedFailures.Count -eq 0) { "passed" } else { "failed" }
    authoritative = $true; diagnostic = $false; assertions_started = $true
    assertions_started_at = [string]$rawScriptedChecks[0].assertions_started_at
    command = "scripted inventory and Sprint 8A smoke from retained Start checkpoint"
    exit_status = if ($scriptedFailures.Count -eq 0) { 0 } else { 1 }
    started_at = $scriptedStarted.ToString("o")
    ended_at = $scriptedEnded.ToString("o")
    duration_ms = [long][Math]::Max(0, ($scriptedEnded - $scriptedStarted).TotalMilliseconds)
    classification = if ($scriptedClassifications.Count -eq 1) { [string]$scriptedClassifications[0] } else { $null }
    classification_source = if ($scriptedFailures.Count -eq 0) {
        $null
    } elseif ($scriptedFailures.Count -eq 1) {
        [string]$scriptedFailures[0].classification_source
    } else { "consolidated_scripted_failures" }
    failure_message = if ($scriptedFailures.Count -eq 0) { $null } else { ($scriptedFailures.failure_message -join "; ") }
    blocked_reason = $null
    evidence = @($rawScriptedChecks | ForEach-Object { @($_.evidence) })
}
$postRestorationSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
Assert-Sprint8ASourceIdentityObject -Source $postRestorationSource -RequireClean | Out-Null
if (-not (Test-Sprint8ASourceIdentityMatch -Expected $attemptReceipt.source_identity -Actual $postRestorationSource)) {
    throw "Formal UAT source changed during manual validation or canonical restoration."
}
$postRestorationEnvironment = Get-Sprint8AEnvironmentContract `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -ProbeDatabases
if ([string]$postRestorationEnvironment.fingerprint -cne $environmentFingerprint) {
    throw "Formal UAT environment changed during manual validation or canonical restoration."
}
$postRestorationCandidate = Get-Sprint8ACandidateIdentity `
    -RepositoryRoot $repoRoot `
    -Source $attemptReceipt.candidate_source_identity `
    -NormalizedDeploymentConfigurationSha256 ([string]$postRestorationEnvironment.contract.compose.normalized_config_sha256)
if ([string]$postRestorationCandidate.fingerprint -cne $candidateFingerprint) {
    throw "Formal UAT candidate identity changed during manual validation or canonical restoration."
}
$postRestorationEndpoints = Get-Sprint8AFormalUatEndpoints -Environment $postRestorationEnvironment
if (($postRestorationEndpoints | ConvertTo-Json -Compress) -cne ($attemptReceipt.endpoints | ConvertTo-Json -Compress)) {
    throw "Formal UAT endpoint identity changed during manual validation or canonical restoration."
}
$currentSource = $postRestorationSource
$preflightReference = Get-Sprint8AFormalUatReference -Path $PreflightReceipt
$candidateReference = Get-Sprint8AFormalUatReference -Path $CandidateReceipt
$sitReference = Get-Sprint8AFormalUatReference -Path $SitReceipt
foreach ($binding in @(
    [pscustomobject]@{ reference = $preflightReference; label = "Preflight" },
    [pscustomobject]@{ reference = $candidateReference; label = "Candidate" },
    [pscustomobject]@{ reference = $sitReference; label = "SIT" }
)) {
    Assert-Sprint8AFormalUatReceiptBinding `
        -Receipt $attemptReceipt `
        -ExpectedReference $binding.reference `
        -Label "Formal UAT Start checkpoint $($binding.label) binding"
}
$canonicalPrerequisites = @(
    foreach ($reference in @($preflightReference, $candidateReference, $sitReference)) {
        [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = [string]$reference.sha256 }
    }
)
Assert-Sprint8ALifecyclePrerequisiteSet `
    -Phase "uat" `
    -References $canonicalPrerequisites `
    -Source $attemptReceipt.candidate_source_identity `
    -EnvironmentFingerprint $environmentFingerprint `
    -NormalizedDeploymentConfigurationSha256 ([string]$postRestorationEnvironment.contract.compose.normalized_config_sha256) `
    -CandidateFingerprint $candidateFingerprint `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $evidenceRootPath | Out-Null
$restorationEvidence = @($restorationCheck.evidence)
$manualEligibilityFailures = @($manualChecks | Where-Object {
    [string]$_.state -cne "passed" -or -not [bool]$_.authoritative -or [bool]$_.diagnostic
})
$firstInvalidatingFailure = @($manualChecks | Where-Object {
    [string]$_.state -ceq "failed" -and [bool]$_.authoritative
} | Sort-Object { ConvertTo-Sprint8ADateTimeOffset -Value $_.ended_at -Label "manual invalidation end" } | Select-Object -First 1)
if ($firstInvalidatingFailure.Count -eq 1) {
    $invalidationTime = ConvertTo-Sprint8ADateTimeOffset `
        -Value $firstInvalidatingFailure[0].ended_at `
        -Label "manual invalidation end"
    foreach ($check in @($manualChecks | Where-Object {
        [bool]$_.authoritative -and
            (ConvertTo-Sprint8ADateTimeOffset -Value $_.started_at -Label "manual sibling start") -gt $invalidationTime
    })) {
        $manualDefects.Add([pscustomobject][ordered]@{
            check = "$($check.name)/authority-after-invalidation"
            classification = "harness"
            classification_source = "formal_uat_state_machine"
            message = "Scenario remained authoritative after '$($firstInvalidatingFailure[0].name)' invalidated the candidate; safe sibling harvesting must be diagnostic."
            evidence = @($check.evidence)
        })
    }
}
if ($manualEligibilityFailures.Count -gt 0 -and $manualDefects.Count -eq 0 -and
    @($manualChecks | Where-Object state -CEQ "blocked").Count -eq 0) {
    $manualDefects.Add([pscustomobject][ordered]@{
        check = "manual-authority-inventory"
        classification = "harness"
        classification_source = "formal_uat_state_machine"
        message = "The eight-scenario inventory does not contain eight authoritative formal passes."
        evidence = @($manualEligibilityFailures | ForEach-Object { @($_.evidence) })
    })
}
$restorationFailed = $restorationFailures.Count -gt 0
foreach ($restorationFailure in $restorationFailures) {
    $manualDefects.Add([pscustomobject][ordered]@{
        check = [string]$restorationFailure.name
        classification = [string]$restorationFailure.classification
        classification_source = [string]$restorationFailure.classification_source
        message = [string]$restorationFailure.failure_message
        evidence = @($restorationFailure.evidence)
    })
}
$manualBlocked = @($manualChecks | Where-Object state -CEQ "blocked")
$allBlocked = @($manualBlocked) + @($restorationBlocked)
if ($scriptedFailures.Count -gt 0 -or $manualEligibilityFailures.Count -gt 0 -or
    [string]$restorationCheck.state -cne "passed") {
    $terminalAttemptState = if ($manualDefects.Count -eq 0 -and $allBlocked.Count -gt 0 -and -not $restorationFailed) {
        "blocked"
    } else { "failed" }
    Set-Sprint8AFormalUatAttemptState -Receipt $attemptReceipt -State $terminalAttemptState -Stage "diagnostic-harvest-complete"
    $attemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
    $attemptReceipt.duration_ms = [long][Math]::Max(
        0,
        ((ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.ended_at -Label "formal UAT end") -
            (ConvertTo-Sprint8ADateTimeOffset -Value $attemptReceipt.started_at -Label "formal UAT start")).TotalMilliseconds
    )
    $attemptReceipt.manual_scenarios_pending = @($manualChecks | Where-Object {
        [string]$_.state -cne "passed" -or -not [bool]$_.authoritative -or [bool]$_.diagnostic
    } | ForEach-Object { [string]$_.name })
    $attemptReceipt.checks = @($scriptedCheck) + @($manualChecks)
    $attemptReceipt.restoration_check = $restorationCheck
    $attemptReceipt | Add-Member -Force -NotePropertyName restoration_checks -NotePropertyValue $restorationChecks
    $attemptReceipt.assertion_count = @($attemptReceipt.checks | Where-Object assertions_started -EQ $true).Count
    $attemptReceipt.failure_count = $scriptedFailures.Count +
        @($manualChecks | Where-Object state -CEQ "failed").Count +
        $restorationFailures.Count
    $attemptReceipt.blocked_count = $allBlocked.Count
    $attemptReceipt.failure_batch = [pscustomobject][ordered]@{
        defect_count = $manualDefects.Count
        blocked_check_count = $allBlocked.Count
        defects = @($manualDefects)
        blocked_checks = @($allBlocked | ForEach-Object {
            [pscustomobject][ordered]@{
                check = [string]$_.name
                dependency_reason = [string]$_.blocked_reason
                classification = if ([string]::IsNullOrWhiteSpace([string]$_.classification)) { $null } else { [string]$_.classification }
                classification_source = if ([string]::IsNullOrWhiteSpace([string]$_.classification_source)) { $null } else { [string]$_.classification_source }
                evidence = @($_.evidence)
            }
        })
    }
    $classifications = @($manualDefects | ForEach-Object { [string]$_.classification } | Sort-Object -Unique)
    $attemptReceipt.classification = if ($terminalAttemptState -ceq "blocked" -and
        @($allBlocked | Where-Object classification -CEQ "product-decision").Count -gt 0) {
        "product-decision"
    } elseif ($classifications.Count -eq 1) { [string]$classifications[0] } else { $null }
    $attemptReceipt.cleanup_restoration = [pscustomobject][ordered]@{
        required = $true
        result = if ([string]$restorationCheck.state -ceq "passed") {
            "canonical_topology_verified"
        } else { [string]$restorationCheck.state }
        evidence = $restorationEvidence
    }
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
    $failedAttemptEntry = [pscustomobject]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
        phase = "uat-attempt"
        authoritative = $false
        status = $terminalAttemptState
    }
    $checkpointEntry = [pscustomobject]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $attemptCheckpointPath).Replace("\", "/")
        phase = "uat-checkpoint"
        authoritative = $false
        status = "passed"
    }
    $failedManifestEntries = @(
        @($failedAttemptEntry, $checkpointEntry) +
            @($manualManifestEntries) + @($manualEvidenceReferences) |
            Group-Object path | ForEach-Object { $_.Group[0] }
    )
    Sync-Sprint8AFormalUatEvidenceManifest -Overrides $failedManifestEntries | Out-Null
    $harvestComplete = [InvalidOperationException]::new(
        "Formal UAT diagnostic harvest retained $($manualDefects.Count) defect(s) and $($allBlocked.Count) blocked check(s); no canonical UAT result was published."
    )
    $harvestComplete.Data["Sprint8AHarvestComplete"] = $true
    throw $harvestComplete
}
Assert-Sprint8AExactTerminalIdentities `
    -Results @($manualChecks) `
    -ExpectedNames (Get-Sprint8AManualUatScenarioNames) `
    -Label "Formal manual UAT" | Out-Null
$restorationManifestEntries = @($restorationChecks | ForEach-Object {
    foreach ($evidenceReference in @($_.evidence)) {
        [pscustomobject][ordered]@{
            path = [string]$evidenceReference.path; sha256 = [string]$evidenceReference.sha256
            phase = "uat-canonical-restoration"; authoritative = $false; status = [string]$_.state
        }
    }
})
$completionManifestOverrides = @(
    @($manualManifestEntries) + @($manualEvidenceReferences) + @($restorationManifestEntries) |
        Group-Object path | ForEach-Object { $_.Group[0] }
)
$completionCheckpointPath = Join-Path $finalizationRoot "finalization-completion-checkpoint.json"
$completionCheckpointDocument = [pscustomobject][ordered]@{
    schema_version = 1; sprint = "sprint-8a"; phase = "uat-finalization-completion"
    authoritative = $false; attempt = $Attempt; state = "complete"
    completed_at = [DateTimeOffset]::UtcNow.ToString("o")
    source_identity = $currentSource
    environment_fingerprint = $environmentFingerprint
    normalized_deployment_configuration_sha256 = [string]$postRestorationEnvironment.contract.compose.normalized_config_sha256
    candidate_fingerprint = $candidateFingerprint
    endpoints = $postRestorationEndpoints
    prerequisite_receipts = $canonicalPrerequisites
    start_checkpoint = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $attemptCheckpointPath).Replace("\", "/")
        sha256 = $attemptCheckpointSha
    }
    scripted_check = $scriptedCheck
    manual_checks = @($manualChecks)
    manual_receipts = @($manualReferences)
    restoration_check = $restorationCheck
    restoration_checks = @($restorationChecks)
    manifest_overrides = $completionManifestOverrides
}
Publish-Sprint7AEvidence -Document $completionCheckpointDocument -OutputPath $completionCheckpointPath | Out-Null
$completionReference = [pscustomobject][ordered]@{
    path = [IO.Path]::GetRelativePath($repoRoot, $completionCheckpointPath).Replace("\", "/")
    sha256 = Assert-Sprint8AReceiptSidecar -Path $completionCheckpointPath
}
$attemptReceipt | Add-Member `
    -Force `
    -NotePropertyName finalization_completion_checkpoint `
    -NotePropertyValue $completionReference
$attemptReceipt | Add-Member -Force -NotePropertyName restoration_check -NotePropertyValue $restorationCheck
$attemptReceipt | Add-Member -Force -NotePropertyName restoration_checks -NotePropertyValue @($restorationChecks)
Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
try {
    Sync-Sprint8AFormalUatEvidenceManifest -Overrides @(
        [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
            phase = "uat-attempt"; authoritative = $false; status = "finalization-complete"
        },
        [pscustomobject][ordered]@{
            path = [string]$completionReference.path; sha256 = [string]$completionReference.sha256
            phase = "uat-finalization-completion"; authoritative = $false; status = "complete"
        }
    ) | Out-Null
} catch {
    $_.Exception.Data["Sprint8AClassification"] = "evidence-finalization"
    throw
}
$completionCheckpoint = Assert-Sprint8AFormalUatFinalizationCompletionCheckpoint `
    -AttemptReceipt $attemptReceipt `
    -AttemptCheckpoint $attemptCheckpoint `
    -AttemptCheckpointSha256 $attemptCheckpointSha
Invoke-Sprint8AFormalUatPublicationTail `
    -AttemptReceipt $attemptReceipt `
    -CompletionCheckpoint $completionCheckpoint `
    -CurrentSource $currentSource `
    -AttemptPath $attemptPath `
    -ResultCommitPath $resultCommitPath `
    -OutputPath $OutputPath | Out-Null
Write-Host "Formal Sprint 8A UAT passed for candidate $candidateFingerprint and environment $environmentFingerprint."
} catch {
    $caughtError = $_
    if ($caughtError.Exception.Data["Sprint8AFinalizationOnlyRetryFailure"] -eq $true) {
        throw $caughtError
    } elseif ($caughtError.Exception.Data["Sprint8AFormalUatFailureRetained"] -eq $true) {
        throw $caughtError
    } elseif ($null -eq $attemptReceipt -or $null -eq $attemptCheckpoint) {
        $entryFailurePath = Join-Path $attemptRoot ("finalization-entry-failure-{0}.json" -f [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
        $entryFailure = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "uat-finalization-entry"
            authoritative = $false; attempt = $Attempt; state = "failed"
            classification = "preflight/setup"; classification_source = "finalization_entry"
            failure_message = $caughtError.Exception.Message
            occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
        }
        Publish-Sprint7AEvidence -Document $entryFailure -OutputPath $entryFailurePath | Out-Null
    } elseif ([string]$attemptReceipt.state -ceq "passed" -and [string]$attemptReceipt.stage -ceq "result-committed") {
        $passedFailureClassification = Resolve-Sprint8AFormalUatFailureClassification `
            -DefaultClassification "evidence-finalization" `
            -ErrorRecord $caughtError
        $passedFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
            -AttemptReceipt $attemptReceipt `
            -ErrorRecord $caughtError `
            -Directory $attemptRoot `
            -Classification ([string]$passedFailureClassification.classification) `
            -ClassificationSource ([string]$passedFailureClassification.source)
        Add-Sprint8AFormalUatEvidenceFinalizationFailure `
            -AttemptReceipt $attemptReceipt `
            -Failure ([pscustomobject][ordered]@{
                occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                classification = [string]$passedFailureClassification.classification
                classification_source = [string]$passedFailureClassification.source
                message = $caughtError.Exception.Message
                evidence = @($passedFailureReference)
            }) `
            -Context "result-commit-publication" | Out-Null
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        try {
            Sync-Sprint8AFormalUatEvidenceManifest `
                -Overrides (Get-Sprint8AFormalUatFailureManifestOverrides `
                    -AttemptPath $attemptPath `
                    -FailureReference $passedFailureReference `
                    -AttemptStatus "passed") | Out-Null
        } catch {
            $passedManifestFailure = $_
            $passedManifestFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -ErrorRecord $passedManifestFailure `
                -Directory $attemptRoot `
                -Classification "evidence-finalization" `
                -ClassificationSource "manifest_publication_boundary"
            $passedManifestFailureEntry = [pscustomobject][ordered]@{
                occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                classification = "evidence-finalization"
                classification_source = "manifest_publication_boundary"
                message = $passedManifestFailure.Exception.Message
                evidence = @($passedManifestFailureReference)
                raw_evidence = @($passedManifestFailureReference.raw_evidence)
            }
            Add-Sprint8AFormalUatEvidenceFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $passedManifestFailureEntry `
                -Context "result-commit-manifest" | Out-Null
            Add-Sprint8AFormalUatManifestUpdateFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $passedManifestFailureEntry | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        }
    } elseif ([string]$attemptReceipt.state -in @("failed", "blocked")) {
        if ($caughtError.Exception.Data["Sprint8AHarvestComplete"] -ne $true) {
            $finalizationReference = Publish-Sprint8AFormalUatFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -ErrorRecord $caughtError `
                -Directory $attemptRoot
            $terminalFinalizationFailure = [pscustomobject][ordered]@{
                occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                classification = "evidence-finalization"
                classification_source = "final_publication_boundary"
                message = $caughtError.Exception.Message
                evidence = @($finalizationReference)
            }
            Merge-Sprint8AFormalUatFailureBatch `
                -AttemptReceipt $attemptReceipt `
                -Defects @([pscustomobject][ordered]@{
                    check = "evidence-finalization"
                    classification = "evidence-finalization"
                    classification_source = "final_publication_boundary"
                    message = $caughtError.Exception.Message
                    evidence = @($finalizationReference)
                }) | Out-Null
            Add-Sprint8AFormalUatEvidenceFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $terminalFinalizationFailure `
                -Context "terminal-attempt-finalization" | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
        }
    } else {
        $failureClassification = Resolve-Sprint8AFormalUatFailureClassification `
            -DefaultClassification "preflight/setup" `
            -ErrorRecord $caughtError
        $manualVariable = Get-Variable -Name manualChecks -ErrorAction SilentlyContinue
        $retainedRestoration = Get-Variable -Name restorationCheck -ErrorAction SilentlyContinue
        $retainedRestorationChecks = Get-Variable -Name restorationChecks -ErrorAction SilentlyContinue
        $scriptedAggregateVariable = Get-Variable -Name scriptedCheck -ErrorAction SilentlyContinue
        $retainedManualChecks = if ($null -eq $manualVariable) { @() } else { @($manualVariable.Value) }
        $publicationRetryEligible = [string]$failureClassification.classification -ceq "evidence-finalization" -and
            $attemptReceipt.PSObject.Properties.Name -contains "finalization_completion_checkpoint" -and
            $retainedManualChecks.Count -eq (Get-Sprint8AManualUatScenarioNames).Count -and
            @($retainedManualChecks | Where-Object {
                [string]$_.state -cne "passed" -or -not [bool]$_.authoritative -or [bool]$_.diagnostic
            }).Count -eq 0 -and
            $null -ne $retainedRestoration -and [string]$retainedRestoration.Value.state -ceq "passed" -and
            $null -ne $scriptedAggregateVariable -and [string]$scriptedAggregateVariable.Value.state -ceq "passed"
        if ($publicationRetryEligible) {
            $finalizationFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                -AttemptReceipt $attemptReceipt `
                -ErrorRecord $caughtError `
                -Directory $attemptRoot
            $retryRoot = if ($null -ne (Get-Variable -Name finalizationRoot -ErrorAction SilentlyContinue)) {
                $finalizationRoot
            } else { $attemptRoot }
            $retryCheckpointPath = Join-Path $retryRoot "publication-retry-checkpoint.json"
            $retryCheckpoint = [pscustomobject][ordered]@{
                schema_version = 1; sprint = "sprint-8a"; phase = "uat-finalization-retry"
                authoritative = $false; attempt = $Attempt; state = "eligible"
                source_identity = $attemptReceipt.source_identity
                environment_fingerprint = [string]$attemptReceipt.environment_fingerprint
                candidate_fingerprint = [string]$attemptReceipt.candidate_fingerprint
                completion_checkpoint = $attemptReceipt.finalization_completion_checkpoint
                scripted_check = $scriptedAggregateVariable.Value
                manual_checks = $retainedManualChecks
                restoration_check = $retainedRestoration.Value
                restoration_checks = if ($null -eq $retainedRestorationChecks) { @() } else { @($retainedRestorationChecks.Value) }
                failure = [pscustomobject][ordered]@{
                    occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                    classification = [string]$failureClassification.classification
                    classification_source = [string]$failureClassification.source
                    message = $caughtError.Exception.Message
                    evidence = @($finalizationFailureReference)
                }
            }
            Publish-Sprint7AEvidence -Document $retryCheckpoint -OutputPath $retryCheckpointPath | Out-Null
            $retryReference = [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $retryCheckpointPath).Replace("\", "/")
                sha256 = Assert-Sprint8AReceiptSidecar -Path $retryCheckpointPath
            }
            $attemptReceipt | Add-Member -Force -NotePropertyName finalization_retry -NotePropertyValue ([pscustomobject][ordered]@{
                eligible = $true; checkpoint = $retryReference
            })
            Add-Sprint8AFormalUatFinalizationRetryFailure `
                -AttemptReceipt $attemptReceipt `
                -Failure $retryCheckpoint.failure | Out-Null
            Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            try {
                Sync-Sprint8AFormalUatEvidenceManifest -Overrides @([pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                    phase = "uat-attempt"; authoritative = $false; status = "publication-retry-eligible"
                }) | Out-Null
            } catch {
                $manifestFailureError = $_
                $manifestFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                    -AttemptReceipt $attemptReceipt `
                    -ErrorRecord $manifestFailureError `
                    -Directory $attemptRoot `
                    -Classification "evidence-finalization" `
                    -ClassificationSource "manifest_publication_boundary"
                $manifestFailureEntry = [pscustomobject][ordered]@{
                    occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
                    classification = "evidence-finalization"
                    classification_source = "manifest_publication_boundary"
                    message = $manifestFailureError.Exception.Message
                    evidence = @($manifestFailureReference)
                    raw_evidence = @($manifestFailureReference.raw_evidence)
                }
                Add-Sprint8AFormalUatFinalizationRetryFailure `
                    -AttemptReceipt $attemptReceipt `
                    -Failure $manifestFailureEntry | Out-Null
                Add-Sprint8AFormalUatManifestUpdateFailure `
                    -AttemptReceipt $attemptReceipt `
                    -Failure $manifestFailureEntry | Out-Null
                $attemptReceipt.finalization_retry | Add-Member `
                    -Force `
                    -NotePropertyName manifest_update_failure `
                    -NotePropertyValue $manifestFailureEntry
                Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            }
        } else {
            try {
                Invoke-Sprint8AFormalUatCatchHarvest `
                    -AttemptReceipt $attemptReceipt `
                    -AttemptCheckpoint $attemptCheckpoint `
                    -ErrorRecord $caughtError `
                    -AttemptPath $attemptPath `
                    -AttemptCheckpointPath $attemptCheckpointPath `
                    -ManualReceiptRoot $manualReceiptRoot `
                    -AttemptRoot $attemptRoot `
                    -RetainedRestorationCheck $(if ($null -eq $retainedRestoration) { $null } else { $retainedRestoration.Value }) `
                    -RetainedRestorationChecks $(if ($null -eq $retainedRestorationChecks) { @() } else { @($retainedRestorationChecks.Value) }) | Out-Null
            } catch {
                $catchHarvestError = $_
                $originalFailureReference = Publish-Sprint8AFormalUatFinalizationFailure `
                    -AttemptReceipt $attemptReceipt `
                    -ErrorRecord $caughtError `
                    -Directory $attemptRoot `
                    -Classification ([string]$failureClassification.classification) `
                    -ClassificationSource ([string]$failureClassification.source)
                $catchHarvestClassification = Resolve-Sprint8AFormalUatFailureClassification `
                    -DefaultClassification "harness" `
                    -ErrorRecord $catchHarvestError
                $catchHarvestFailure = Publish-Sprint8AFormalUatFinalizationFailure `
                    -AttemptReceipt $attemptReceipt `
                    -ErrorRecord $catchHarvestError `
                    -Directory $attemptRoot `
                    -Classification ([string]$catchHarvestClassification.classification) `
                    -ClassificationSource ([string]$catchHarvestClassification.source)
                if ([string]$attemptReceipt.state -notin @("failed", "blocked", "passed")) {
                    Set-Sprint8AFormalUatAttemptState `
                        -Receipt $attemptReceipt `
                        -State "failed" `
                        -Stage ([string]$attemptReceipt.stage)
                }
                Merge-Sprint8AFormalUatFailureBatch `
                    -AttemptReceipt $attemptReceipt `
                    -Defects @(
                        [pscustomobject]@{
                            check = "finalization-prerequisite"; classification = [string]$failureClassification.classification
                            classification_source = [string]$failureClassification.source; message = $caughtError.Exception.Message
                            evidence = @($originalFailureReference)
                        },
                        [pscustomobject]@{
                            check = "catch-harvest"; classification = [string]$catchHarvestClassification.classification
                            classification_source = [string]$catchHarvestClassification.source
                            message = $catchHarvestError.Exception.Message; evidence = @($catchHarvestFailure)
                        }
                    ) `
                    -BlockedChecks @(Get-Sprint8AManualUatScenarioNames | ForEach-Object {
                        [pscustomobject]@{ check = $_; dependency_reason = "catch harvest could not authenticate manual evidence"; evidence = @($catchHarvestFailure) }
                    }) | Out-Null
                $attemptReceipt.cleanup_restoration = [pscustomobject]@{ required = $true; result = "not_proven"; evidence = @() }
                Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
            }
        }
    }
    throw $caughtError
}
finally {
    $attemptLock.Dispose()
}

[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$ReadinessReceipt = "artifacts/sprint-8a-closeout/validation-readiness-result.json",
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [switch]$ResumeInterruptedAttempt,
    [switch]$AuthorizeApprovedR33Correction,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-rehearsal-scheduler.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-health-contract.ps1")

function Open-Sprint8AValidationAttemptLock {
    param([Parameter(Mandatory)][string]$Path)
    [IO.File]::Open($Path, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
}

function Publish-OrAuthenticateRehearsalImmutableEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    Repair-Sprint7AEvidencePublication -Path $Path
    $documentExists = Test-Path -LiteralPath $Path -PathType Leaf
    $sidecarExists = Test-Path -LiteralPath "$Path.sha256" -PathType Leaf
    if ($documentExists -xor $sidecarExists) {
        throw "$Label has an incomplete immutable receipt/sidecar pair; recovery will not rewrite it."
    }
    if (-not $documentExists) {
        Publish-Sprint7AEvidence -Document $Document -OutputPath $Path | Out-Null
        return Assert-Sprint8AReceiptSidecar -Path $Path
    }

    $sha = Assert-Sprint8AReceiptSidecar -Path $Path
    $retained = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    if (($retained | ConvertTo-Json -Depth 100 -Compress) -cne
        ($Document | ConvertTo-Json -Depth 100 -Compress)) {
        $propertyNames = @(
            @($retained.PSObject.Properties.Name) +
            @($Document.Keys) |
                Sort-Object -Unique
        )
        $changedProperties = @($propertyNames | Where-Object {
            $name = [string]$_
            $retainedValue = if ($retained.PSObject.Properties.Name -contains $name) { $retained.$name } else { $null }
            $projectedValue = if ($Document.Contains($name)) { $Document[$name] } else { $null }
            ($retainedValue | ConvertTo-Json -Depth 100 -Compress) -cne
                ($projectedValue | ConvertTo-Json -Depth 100 -Compress)
        })
        throw "$Label already exists but differs from the authenticated recovery projection in: $($changedProperties -join ', '); recovery will not rewrite it."
    }
    $sha
}

function ConvertTo-RehearsalDeclarationDictionary {
    param([Parameter(Mandatory)]$Declaration)

    if ($Declaration -is [Collections.IDictionary]) { return $Declaration }
    $converted = [ordered]@{}
    foreach ($property in $Declaration.PSObject.Properties) {
        $converted[[string]$property.Name] = $property.Value
    }
    $converted
}

function ConvertTo-RehearsalOffsetTimestampText {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    (ConvertTo-Sprint8ADateTimeOffset -Value $Value -Label $Label).ToString("o")
}

function Assert-RehearsalCorrectionAuthorizationTail {
    param(
        [Parameter(Mandatory)]$Authorization,
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][string]$AttemptPath,
        [Parameter(Mandatory)][string]$AttemptSha256,
        [Parameter(Mandatory)][string]$HarvestPath,
        [Parameter(Mandatory)][string]$HarvestSha256,
        [Parameter(Mandatory)][string]$BatchPath,
        [Parameter(Mandatory)][string]$BatchSha256,
        [Parameter(Mandatory)]$AttemptDocument
    )

    $attemptReference = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $AttemptPath
    $harvestReference = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $HarvestPath
    $batchReference = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $BatchPath
    if (($Authorization.schema_version -isnot [int] -and $Authorization.schema_version -isnot [long]) -or
        [int]$Authorization.schema_version -ne 3 -or
        [string]$Authorization.sprint -cne "sprint-8a" -or
        [string]$Authorization.phase -cne "candidate-rehearsal-correction-authorization" -or
        [int]$Authorization.attempt -ne $Attempt -or
        $Authorization.authoritative -isnot [bool] -or
        $Authorization.authoritative -ne $false -or
        [string]$Authorization.state -cne "authorized" -or
        [string]$Authorization.consumption_state -cne "unconsumed" -or
        [string]$Authorization.allowed_successor_phase -cne "validation-readiness" -or
        [int]$Authorization.allowed_successor_count -ne 1 -or
        [string]$Authorization.authorization -cne "tracked correction and one successor readiness attempt are permitted for this consolidated batch" -or
        [int]$Authorization.deferred_count -ne [int]$AttemptDocument.deferred_count -or
        [string]$Authorization.environment_fingerprint -cne $EnvironmentFingerprint -or
        ($Authorization.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress) -cne
            ($Source | ConvertTo-Json -Depth 20 -Compress) -or
        [string]$Authorization.predecessor_attempt_receipt.path -cne $attemptReference -or
        [string]$Authorization.predecessor_attempt_receipt.sha256 -cne $AttemptSha256 -or
        [string]$Authorization.harvest_receipt.path -cne $harvestReference -or
        [string]$Authorization.harvest_receipt.sha256 -cne $HarvestSha256 -or
        [string]$Authorization.defect_batch.path -cne $batchReference -or
        [string]$Authorization.defect_batch.sha256 -cne $BatchSha256 -or
        $Authorization.cleanup_restoration.required -ne $true -or
        [string]$Authorization.cleanup_restoration.result -cne "canonical_successor_healthy") {
        throw "Candidate Rehearsal recovery rejected an inauthentic existing correction-authorization tail receipt."
    }
    try {
        [void][DateTimeOffset]::Parse([string]$Authorization.generated_at)
    } catch {
        throw "Candidate Rehearsal recovery rejected an undated existing correction-authorization tail receipt."
    }
    $expectedCleanupEvidence = @(@("final-successor-health", "final-environment-identity") | ForEach-Object {
        $lane = @($AttemptDocument.checks | Where-Object name -CEQ $_)
        if ($lane.Count -ne 1) {
            throw "Candidate Rehearsal recovery cannot bind correction authorization without exact cleanup lane '$_'."
        }
        [ordered]@{ lane = $_; path = [string]$lane[0].lane_receipt.path; sha256 = [string]$lane[0].lane_receipt.sha256 }
    })
    if ((@($Authorization.cleanup_restoration.evidence) | ConvertTo-Json -Depth 20 -Compress) -cne
        ($expectedCleanupEvidence | ConvertTo-Json -Depth 20 -Compress)) {
        throw "Candidate Rehearsal recovery rejected changed correction-authorization cleanup evidence."
    }
}

function Assert-RehearsalLaneIdentityBinding {
    param(
        [Parameter(Mandatory)]$LaneDocument,
        [Parameter(Mandatory)]$AttemptDocument,
        [Parameter(Mandatory)][string]$LaneName
    )

    if ([string]$LaneDocument.identity_binding -ceq "attempt_identity") {
        if (($LaneDocument.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress) -cne
                ($AttemptDocument.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress) -or
            [string]$LaneDocument.environment_fingerprint -cne [string]$AttemptDocument.environment_fingerprint) {
            throw "Candidate Rehearsal recovery rejected lane '$LaneName' source/environment binding."
        }
        return
    }

    # The lifecycle prerequisite intentionally executes before current source and environment
    # authentication. Its zero-valued identity is therefore valid diagnostic evidence even when
    # process loss occurs before the subsequent Readiness lane can verify the attempt checkpoint.
    if ([string]$LaneDocument.identity_binding -cne "pre_authentication_lifecycle_placeholder" -or
        $LaneName -cne "attempt-state-prerequisite" -or
        -not (Test-Sprint8ARehearsalPlaceholderSourceIdentity -Source $LaneDocument.mutable_source_identity) -or
        [string]$LaneDocument.environment_fingerprint -cne ("0" * 64) -or
        @("passed", "failed") -cnotcontains [string]$LaneDocument.result.state -or
        $LaneDocument.result.assertions_started -isnot [bool] -or
        -not [bool]$LaneDocument.result.assertions_started) {
        throw "Candidate Rehearsal recovery rejected lane '$LaneName' identity-binding mode."
    }
}

function Set-RehearsalRecoveredAttemptIdentity {
    param(
        [Parameter(Mandatory)]$AttemptDocument,
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][string]$ReadinessSha256
    )

    $AttemptDocument.mutable_source_identity = $Source
    $AttemptDocument.source_identity_verification_state = "verified"
    $AttemptDocument.source_identity_verification_failure = $null
    $AttemptDocument.environment_identity.readiness_sha256 = $ReadinessSha256
    $AttemptDocument.environment_identity.verification_state = "verified"
    $AttemptDocument.environment_identity.verification_failure = $null
    $AttemptDocument.environment_fingerprint = $EnvironmentFingerprint
    $AttemptDocument
}

function Test-RehearsalRecoveredTerminalSourceBinding {
    param(
        [Parameter(Mandatory)]$AttemptDocument,
        [Parameter(Mandatory)]$RecoveredSource
    )

    if (($AttemptDocument.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress) -ceq
        ($RecoveredSource | ConvertTo-Json -Depth 20 -Compress)) {
        return $true
    }
    @("unverified", "failed") -ccontains [string]$AttemptDocument.source_identity_verification_state -and
        (Test-Sprint8ARehearsalPlaceholderSourceIdentity -Source $AttemptDocument.mutable_source_identity)
}

function Assert-RehearsalRestorationMaterializationDocument {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$ExpectedSource,
        [Parameter(Mandatory)][string]$ExpectedEnvironmentFingerprint,
        [Parameter(Mandatory)][int]$ExpectedAttempt
    )

    $sourceMatches = $Document.source.commit -is [string] -and
        [string]$Document.source.commit -ceq [string]$ExpectedSource.commit -and
        $Document.source.tree -is [string] -and
        [string]$Document.source.tree -ceq [string]$ExpectedSource.tree -and
        $Document.source.branch -is [string] -and
        [string]$Document.source.branch -ceq [string]$ExpectedSource.branch -and
        $Document.source.dirty -is [bool] -and
        [bool]$Document.source.dirty -eq [bool]$ExpectedSource.dirty -and
        $Document.source.PSObject.Properties.Name -contains "dirty_paths" -and
        @($Document.source.dirty_paths).Count -eq 0

    if (($Document.schema_version -isnot [int] -and $Document.schema_version -isnot [long]) -or
        [int]$Document.schema_version -ne 1 -or
        [string]$Document.contract -cne "tessara.sprint-8a.materialization-result" -or
        [int]$Document.attempt -ne $ExpectedAttempt -or
        $Document.passed -isnot [bool] -or -not [bool]$Document.passed -or
        -not $sourceMatches -or
        [string]$Document.environment.declared_fingerprint -cne $ExpectedEnvironmentFingerprint -or
        $Document.first_apply.no_op -isnot [bool] -or [bool]$Document.first_apply.no_op -or
        $Document.no_op_apply.no_op -isnot [bool] -or -not [bool]$Document.no_op_apply.no_op -or
        $Document.final_health_passed -isnot [bool] -or -not [bool]$Document.final_health_passed) {
        throw "Mandatory final restoration materialization receipt does not prove the exact attempt, source, environment, first apply, no-op, and final health contract."
    }
}

function Assert-RehearsalRestorationMaterializationReceipt {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$ExpectedSource,
        [Parameter(Mandatory)][string]$ExpectedEnvironmentFingerprint,
        [Parameter(Mandatory)][int]$ExpectedAttempt
    )

    $sha = Assert-Sprint8AReceiptSidecar -Path $Path
    $document = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    Assert-RehearsalRestorationMaterializationDocument `
        -Document $document `
        -ExpectedSource $ExpectedSource `
        -ExpectedEnvironmentFingerprint $ExpectedEnvironmentFingerprint `
        -ExpectedAttempt $ExpectedAttempt
    [ordered]@{
        path = ConvertTo-Sprint8ACanonicalEvidencePath `
            -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $Path
        sha256 = $sha
    }
}

function Assert-RehearsalRecoveredTerminalLaneReceipt {
    param(
        [Parameter(Mandatory)]$Terminal,
        [Parameter(Mandatory)]$AttemptDocument
    )

    $laneName = [string]$Terminal.name
    $laneReceiptPath = Join-Path $laneRoot "$laneName.json"
    Repair-Sprint7AEvidencePublication -Path $laneReceiptPath
    $expectedLaneReference = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $laneReceiptPath
    if ($Terminal.PSObject.Properties.Name -notcontains "lane_receipt" -or
        [string]$Terminal.lane_receipt.path -cne $expectedLaneReference) {
        throw "Candidate Rehearsal recovery rejected lane '$laneName' without its exact immutable receipt reference."
    }
    $laneSha = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
    if ([string]$Terminal.lane_receipt.sha256 -cne $laneSha) {
        throw "Candidate Rehearsal recovery rejected lane '$laneName' receipt digest drift."
    }
    $laneDocument = Get-Content -LiteralPath $laneReceiptPath -Raw | ConvertFrom-Json
    if (($laneDocument.schema_version -isnot [int] -and $laneDocument.schema_version -isnot [long]) -or
        [int]$laneDocument.schema_version -ne 2 -or
        [string]$laneDocument.sprint -cne "sprint-8a" -or
        [string]$laneDocument.phase -cne "candidate-rehearsal-lane" -or
        [int]$laneDocument.attempt -ne $Attempt -or
        $laneDocument.authoritative -isnot [bool] -or [bool]$laneDocument.authoritative -or
        [string]$laneDocument.result.name -cne $laneName) {
        throw "Candidate Rehearsal recovery rejected lane '$laneName' identity."
    }
    $attemptLaneProjection = $Terminal | ConvertTo-Json -Depth 100 | ConvertFrom-Json
    $attemptLaneProjection.PSObject.Properties.Remove("lane_receipt")
    if (($laneDocument.result | ConvertTo-Json -Depth 100 -Compress) -cne
        ($attemptLaneProjection | ConvertTo-Json -Depth 100 -Compress)) {
        throw "Candidate Rehearsal recovery rejected lane '$laneName' result drift."
    }
    Assert-RehearsalLaneIdentityBinding `
        -LaneDocument $laneDocument `
        -AttemptDocument $AttemptDocument `
        -LaneName $laneName
}

function Resolve-RehearsalPreAttemptRecoveryState {
    param(
        [Parameter(Mandatory)][bool]$SnapshotReceiptExists,
        [Parameter(Mandatory)][bool]$SnapshotSidecarExists,
        [Parameter(Mandatory)][bool]$StartReceiptExists,
        [Parameter(Mandatory)][bool]$StartSidecarExists,
        [Parameter(Mandatory)][bool]$AttemptReceiptExists,
        [Parameter(Mandatory)][bool]$AttemptSidecarExists
    )

    if ($SnapshotReceiptExists -xor $SnapshotSidecarExists -or -not $SnapshotReceiptExists) {
        throw "Candidate Rehearsal recovery requires a complete immutable validation-state capture pair."
    }
    if ($StartReceiptExists -xor $StartSidecarExists) {
        throw "Candidate Rehearsal recovery rejected an incomplete immutable start pair."
    }
    if ($AttemptReceiptExists -xor $AttemptSidecarExists) {
        throw "Candidate Rehearsal recovery rejected an incomplete attempt checkpoint pair."
    }
    if ($AttemptReceiptExists -and -not $StartReceiptExists) {
        throw "Candidate Rehearsal recovery rejected an attempt checkpoint without its immutable start."
    }
    if ($AttemptReceiptExists) { return "attempt_checkpoint" }
    if ($StartReceiptExists) { return "start_only" }
    "snapshot_only"
}

function Resolve-RehearsalAttemptRecoveryState {
    param(
        [Parameter(Mandatory)][ValidateSet("preparing", "executing", "harvesting", "failed", "passed")][string]$AttemptState,
        [Parameter(Mandatory)][int]$TerminalLaneCount,
        [Parameter(Mandatory)][int]$DeclaredLaneCount,
        [Parameter(Mandatory)][bool]$HasActiveLane,
        [Parameter(Mandatory)][bool]$HarvestReceiptExists,
        [Parameter(Mandatory)][bool]$HarvestSidecarExists,
        [Parameter(Mandatory)][bool]$BatchReceiptExists,
        [Parameter(Mandatory)][bool]$BatchSidecarExists,
        [Parameter(Mandatory)][bool]$AuthorizationReceiptExists,
        [Parameter(Mandatory)][bool]$AuthorizationSidecarExists,
        [Parameter(Mandatory)][bool]$ResultReceiptExists,
        [Parameter(Mandatory)][bool]$ResultSidecarExists,
        [Parameter(Mandatory)][bool]$StateReceiptExists,
        [Parameter(Mandatory)][bool]$StateSidecarExists,
        [Parameter(Mandatory)][bool]$CurrentPassingResultExists,
        [Parameter(Mandatory)][bool]$CurrentTerminalStateExists
    )

    foreach ($pair in @(
        [pscustomobject]@{ label = "harvest"; receipt = $HarvestReceiptExists; sidecar = $HarvestSidecarExists },
        [pscustomobject]@{ label = "defect batch"; receipt = $BatchReceiptExists; sidecar = $BatchSidecarExists },
        [pscustomobject]@{ label = "correction authorization"; receipt = $AuthorizationReceiptExists; sidecar = $AuthorizationSidecarExists },
        [pscustomobject]@{ label = "candidate result alias"; receipt = $ResultReceiptExists; sidecar = $ResultSidecarExists },
        [pscustomobject]@{ label = "validation state"; receipt = $StateReceiptExists; sidecar = $StateSidecarExists }
    )) {
        if ([bool]$pair.receipt -xor [bool]$pair.sidecar) {
            throw "Candidate Rehearsal recovery rejected an incomplete $([string]$pair.label) pair."
        }
    }
    if ($TerminalLaneCount -lt 0 -or $TerminalLaneCount -gt $DeclaredLaneCount) {
        throw "Candidate Rehearsal recovery rejected impossible terminal lane accounting."
    }

    if (@("preparing", "executing", "harvesting") -ccontains $AttemptState) {
        if ($HarvestReceiptExists -or $BatchReceiptExists -or $AuthorizationReceiptExists -or
            $CurrentPassingResultExists -or $CurrentTerminalStateExists) {
            throw "Candidate Rehearsal recovery rejected terminal-tail evidence before the attempt terminalized."
        }
        if ($HasActiveLane) { return "mid_lane_checkpoint" }
        if ($AttemptState -ceq "harvesting" -and $TerminalLaneCount -lt $DeclaredLaneCount) {
            return "deferred_or_harvest_loop_checkpoint"
        }
        return "lane_loop_checkpoint"
    }

    if ($TerminalLaneCount -ne $DeclaredLaneCount -or $HasActiveLane) {
        throw "Candidate Rehearsal recovery rejected incomplete accounting in a terminal attempt."
    }
    if ($AttemptState -ceq "passed") {
        if ($HarvestReceiptExists -or $BatchReceiptExists -or $AuthorizationReceiptExists) {
            throw "Candidate Rehearsal recovery rejected failed-attempt tail evidence for a passing attempt."
        }
        if (-not $CurrentPassingResultExists) { return "passed_needs_result" }
        if (-not $CurrentTerminalStateExists) { return "passed_needs_state" }
        return "passed_tail_complete"
    }

    if ($CurrentPassingResultExists) {
        throw "Candidate Rehearsal recovery rejected a passing current-attempt result for a failed attempt."
    }
    if ($BatchReceiptExists -and -not $HarvestReceiptExists) {
        throw "Candidate Rehearsal recovery rejected a defect batch without its harvest."
    }
    if ($AuthorizationReceiptExists -and -not $BatchReceiptExists) {
        throw "Candidate Rehearsal recovery rejected correction authorization without its consolidated defect batch."
    }
    if (-not $HarvestReceiptExists) { return "failed_needs_harvest" }
    if (-not $BatchReceiptExists) { return "failed_needs_batch" }
    if (-not $AuthorizationReceiptExists) { return "failed_needs_authorization_or_withheld_state" }
    if (-not $CurrentTerminalStateExists) { return "failed_needs_state" }
    "failed_tail_complete"
}

function Invoke-RehearsalPowerShellCheck {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$FailureMessage
    )

    & $Action
    $childSucceeded = $?
    if (-not $childSucceeded) { throw $FailureMessage }
}

function Test-RehearsalPowerShellCheck {
    $savedNativeExitCodeVariable = Get-Variable -Name LASTEXITCODE -Scope Global -ErrorAction SilentlyContinue
    $savedNativeExitCode = if ($null -eq $savedNativeExitCodeVariable) {
        $null
    } else {
        $savedNativeExitCodeVariable.Value
    }
    try {
        $global:LASTEXITCODE = 1
        $output = @(Invoke-RehearsalPowerShellCheck -Action {
            Write-Output "successful PowerShell child"
        } -FailureMessage "A stale native exit code falsely failed a successful PowerShell child.")
        if (($output -join "") -cne "successful PowerShell child") {
            throw "PowerShell child status self-test did not preserve successful output."
        }

        try {
            Invoke-RehearsalPowerShellCheck -Action {
                throw "intentional PowerShell child failure"
            } -FailureMessage "PowerShell child failure was not propagated."
            throw "PowerShell child status self-test accepted a thrown child failure."
        } catch {
            if ($_.Exception.Message -ceq "PowerShell child status self-test accepted a thrown child failure.") {
                throw
            }
            if ($_.Exception.Message -cne "intentional PowerShell child failure") {
                throw "PowerShell child status self-test did not preserve the thrown child failure."
            }
        }
    } finally {
        if ($null -eq $savedNativeExitCodeVariable) {
            Remove-Variable -Name LASTEXITCODE -Scope Global -ErrorAction SilentlyContinue
        } else {
            $global:LASTEXITCODE = $savedNativeExitCode
        }
    }
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

function Test-RehearsalReadinessLaneIsolation {
    $sourceText = Get-Content -LiteralPath $PSCommandPath -Raw
    $startMarker = 'Invoke-RehearsalLane "validation-readiness-prerequisite" {'
    $endMarker = 'Invoke-RehearsalLane "formatting" {'
    $start = $sourceText.LastIndexOf($startMarker, [StringComparison]::Ordinal)
    $end = $sourceText.IndexOf($endMarker, $start + $startMarker.Length, [StringComparison]::Ordinal)
    if ($start -lt 0 -or $end -le $start) {
        throw "Candidate rehearsal self-test cannot isolate the independent Readiness lane."
    }
    $lane = $sourceText.Substring($start, $end - $start)
    foreach ($forbidden in @(
        '$runtimeContext.validation_state',
        '$runtimeContext.readiness_immutable_reference',
        '$stateIndex',
        '$statePath'
    )) {
        if ($lane.Contains($forbidden)) {
            throw "Independent Readiness lane retains state-lane dependency '$forbidden'."
        }
    }
    if (-not $lane.Contains('Resolve-Sprint8AEvidenceReference') -or
        -not $lane.Contains('Assert-Sprint8AReceiptSidecar')) {
        throw "Independent Readiness lane does not authenticate contained correction-consumption evidence."
    }
}

function Test-Sprint8AFirstRehearsalCorrectionLink {
    $lineage = Add-Sprint8ACorrectionLineageLink `
        -Lineage $null `
        -Predecessor ([ordered]@{
            phase = "candidate-rehearsal"; attempt = 1
            receipt = [ordered]@{ path = "attempts/candidate-rehearsal-1-attempt.json"; sha256 = "1" * 64 }
        }) `
        -Authorization ([ordered]@{
            path = "attempts/candidate-rehearsal-1-correction-authorization.json"; sha256 = "2" * 64
        }) `
        -ConsumedByReadiness $null
    if ($lineage.links -isnot [array] -or @($lineage.links).Count -ne 1 -or
        [int]$lineage.links[0].ordinal -ne 1 -or
        [string]$lineage.links[0].predecessor.phase -cne "candidate-rehearsal") {
        throw "Candidate rehearsal first-link self-test detected scalar/null correction-lineage collapse."
    }
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

    foreach ($candidate in @($Receipt.mutable_source_identity, $ExpectedSource)) {
        try {
            Assert-Sprint8ASourceIdentityObject -Source $candidate -RequireClean | Out-Null
        } catch {
            throw "Nested UAT diagnostic receipt carries a malformed or dirty mutable source identity."
        }
    }
    if (($Receipt.mutable_source_identity | ConvertTo-Json -Depth 10 -Compress) -cne
        ($ExpectedSource | ConvertTo-Json -Depth 10 -Compress)) {
        throw "Nested UAT diagnostic receipt is not bound to the current clean source identity."
    }
}

function Resolve-LaneClassification {
    param(
        [Parameter(Mandatory)][string]$Default,
        [AllowEmptyString()][string]$Detail,
        [AllowNull()][string]$StructuredClassification
    )

    $allowed = @(
        "preflight/setup", "product", "harness", "environment", "flaky",
        "evidence-finalization", "product-decision"
    )
    if (-not [string]::IsNullOrWhiteSpace($StructuredClassification)) {
        if ($allowed -cnotcontains $StructuredClassification) {
            throw "Structured rehearsal failure classification '$StructuredClassification' is unsupported."
        }
        return [pscustomobject][ordered]@{
            classification = $StructuredClassification
            source = "structured_evidence"
        }
    }
    if ($Detail -match '(?i)PropertyNotFoundException|ParameterBindingException|cannot be found that matches parameter name|malformed or dirty mutable source identity|receipt SHA-256 sidecar|Failure-containment artifact sidecar|declared digest does not match|runner .* does not parse') {
        return [pscustomobject][ordered]@{ classification = "harness"; source = "failure_detail" }
    }
    if ($Detail -match '(?i)Required Sprint 8A tool .* unavailable|Docker daemon is not running|Cannot connect to the Docker daemon|Authenticated database probe failed|connection refused while probing required') {
        return [pscustomobject][ordered]@{ classification = "environment"; source = "failure_detail" }
    }
    if ($allowed -cnotcontains $Default) {
        throw "Declared rehearsal failure classification '$Default' is unsupported."
    }
    [pscustomobject][ordered]@{ classification = $Default; source = "declared_default" }
}

function Get-RehearsalStructuredClassification {
    param([Parameter(Mandatory)][string]$Name)

    if ($Name -ceq "failure-containment-successor-health" -and
        (Test-Path -LiteralPath $failureContainmentResult -PathType Leaf)) {
        $receipt = Get-Content -LiteralPath $failureContainmentResult -Raw | ConvertFrom-Json
        return [string]$receipt.original_defect.classification
    }
    if ($Name -ceq "source-exact-materialization-no-op") {
        $failureReceipt = Join-Path $attemptRoot "materialization/attempt-$Attempt/materialization-failure.json"
        if (Test-Path -LiteralPath $failureReceipt -PathType Leaf) {
            $receipt = Get-Content -LiteralPath $failureReceipt -Raw | ConvertFrom-Json
            if ($receipt.failure.PSObject.Properties.Name -contains "classification") {
                return [string]$receipt.failure.classification
            }
        }
    }
    if ($Name -ceq "uat-diagnostics" -and (Test-Path -LiteralPath $uatResult -PathType Leaf)) {
        $receipt = Get-Content -LiteralPath $uatResult -Raw | ConvertFrom-Json
        if ([int]$receipt.harness_failure_count -gt 0 -and [int]$receipt.semantic_failure_count -eq 0) {
            return "harness"
        }
    }
    $null
}

function Test-RehearsalClassificationResolution {
    $structured = Resolve-LaneClassification -Default "product" -Detail "" -StructuredClassification "harness"
    $property = Resolve-LaneClassification -Default "environment" -Detail "System.Management.Automation.PropertyNotFoundException" -StructuredClassification $null
    $environment = Resolve-LaneClassification -Default "product" -Detail "Docker daemon is not running" -StructuredClassification $null
    $fallback = Resolve-LaneClassification -Default "product" -Detail "ordinary assertion mismatch" -StructuredClassification $null
    if ([string]$structured.classification -cne "harness" -or [string]$structured.source -cne "structured_evidence" -or
        [string]$property.classification -cne "harness" -or
        [string]$environment.classification -cne "environment" -or
        [string]$fallback.classification -cne "product" -or [string]$fallback.source -cne "declared_default") {
        throw "Rehearsal classification precedence self-test failed."
    }
    try {
        Resolve-LaneClassification -Default "product" -Detail "" -StructuredClassification "validation_only_fault_injection" | Out-Null
        throw "Rehearsal classification self-test accepted an unsupported structured classification."
    } catch {
        if ($_.Exception.Message -ceq "Rehearsal classification self-test accepted an unsupported structured classification.") { throw }
    }
}

function Test-Sprint8ACandidateTwoWaveRunnerContract {
    $sourceText = Get-Content -LiteralPath $PSCommandPath -Raw
    $requiredFragments = @(
        'candidate-rehearsal-$Attempt-start.json',
        'candidate-rehearsal-$Attempt-validation-state.json',
        '$validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath',
        'Publish-Sprint7AEvidence -Document $startDocument -OutputPath $startPath',
        '$selectedSchedule = $startDocument.schedule',
        'Complete-RehearsalOrphanedLane -Name $orphanedLaneName',
        'candidate-rehearsal-process-loss-capture',
        '$recoveredPublishedTerminal',
        'active_lane_started_at',
        'declared_checks = $declaredChecks',
        'Assert-Sprint8ADeclaredEvidencePaths',
        'cargo test --workspace --all-features --locked --offline --jobs 1',
        'pre_authentication_lifecycle_placeholder',
        'identity_binding = Get-RehearsalLaneIdentityBinding -Name $Name',
        'foreach ($name in @($selectedSchedule.wave_a))',
        'foreach ($name in @($selectedSchedule.cleanup_sinks))',
        'if ($waveBDisposition -ceq "defer")',
        'Add-RehearsalDeferredLane -Name ([string]$name)',
        'foreach ($name in @($selectedSchedule.aggregate_sinks))',
        'foreach ($name in @($selectedSchedule.finalizers))',
        'Assert-Sprint8ARehearsalTerminalAccounting'
    )
    foreach ($fragment in $requiredFragments) {
        if (-not $sourceText.Contains($fragment)) {
            throw "Candidate Rehearsal two-wave runner self-test cannot find enforcement fragment '$fragment'."
        }
    }
    $executionBoundary = $sourceText.LastIndexOf('if ($Attempt -lt 1)', [StringComparison]::Ordinal)
    $reservationLock = $sourceText.LastIndexOf('$validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath', [StringComparison]::Ordinal)
    $publishStart = $sourceText.LastIndexOf('Publish-Sprint7AEvidence -Document $startDocument -OutputPath $startPath', [StringComparison]::Ordinal)
    $waveA = $sourceText.LastIndexOf('foreach ($name in @($selectedSchedule.wave_a))', [StringComparison]::Ordinal)
    $decision = $sourceText.LastIndexOf('if ($waveBDisposition -ceq "defer")', [StringComparison]::Ordinal)
    $aggregate = $sourceText.LastIndexOf('foreach ($name in @($selectedSchedule.aggregate_sinks))', [StringComparison]::Ordinal)
    $cleanup = $sourceText.LastIndexOf('foreach ($name in @($selectedSchedule.cleanup_sinks))', [StringComparison]::Ordinal)
    $finalizers = $sourceText.LastIndexOf('foreach ($name in @($selectedSchedule.finalizers))', [StringComparison]::Ordinal)
    if ($executionBoundary -lt 0 -or $reservationLock -le $executionBoundary -or
        $publishStart -le $reservationLock -or $waveA -le $publishStart -or
        $decision -le $waveA -or $aggregate -le $decision -or $cleanup -le $aggregate -or
        $finalizers -le $cleanup) {
        throw "Candidate Rehearsal reservation, immutable start, Wave A, Wave B decision, aggregate, terminal cleanup, and finalizer ordering is not fixed."
    }
    if (-not $sourceText.Contains('$selectedSchedule = $startDocument.schedule') -or
        -not $sourceText.Contains('Complete-RehearsalOrphanedLane') -or
        -not $sourceText.Contains('($startDocument.declared_checks | ConvertTo-Json -Depth 30 -Compress)') -or
        -not $sourceText.Contains('$attemptReceipt.schedule_sha256 -cne $scheduleSha')) {
        throw "Candidate Rehearsal process-loss recovery does not preserve the immutable ordering and deferral counters."
    }
}

function Test-Sprint8AProcessLossRecoveryContract {
    $sourceText = Get-Content -LiteralPath $PSCommandPath -Raw
    foreach ($fragment in @(
        '$ResumeInterruptedAttempt -and $resumeHasStart',
        '$ResumeInterruptedAttempt -and $resumeHasAttempt',
        '$recoveredTerminalAttempt = @("passed", "failed")',
        'if (-not $recoveredTerminalAttempt) {',
        '$attemptSha = $attemptShaBeforeRecovery',
        'Publish-OrAuthenticateRehearsalImmutableEvidence',
        'Assert-RehearsalCorrectionAuthorizationTail',
        'recovery will not rewrite it'
    )) {
        if (-not $sourceText.Contains($fragment)) {
            throw "Candidate Rehearsal process-loss self-test cannot find recovery enforcement fragment '$fragment'."
        }
    }
    $resumeGate = $sourceText.LastIndexOf('$resumeHasStart = $false', [StringComparison]::Ordinal)
    $resumeStart = $sourceText.LastIndexOf('if ($ResumeInterruptedAttempt -and $resumeHasStart) {', [StringComparison]::Ordinal)
    $resumeAttempt = $sourceText.LastIndexOf('if ($ResumeInterruptedAttempt -and $resumeHasAttempt) {', [StringComparison]::Ordinal)
    $terminalExecutionGuard = $sourceText.LastIndexOf('if (-not $recoveredTerminalAttempt) {', [StringComparison]::Ordinal)
    $terminalAttemptReuse = $sourceText.LastIndexOf('$attemptSha = $attemptShaBeforeRecovery', [StringComparison]::Ordinal)
    $harvestTail = $sourceText.LastIndexOf('if (-not $passed) {', [StringComparison]::Ordinal)
    if ($resumeGate -lt 0 -or $resumeStart -le $resumeGate -or $resumeAttempt -le $resumeStart -or
        $terminalExecutionGuard -le $resumeAttempt -or $terminalAttemptReuse -le $terminalExecutionGuard -or
        $harvestTail -le $terminalAttemptReuse) {
        throw "Candidate Rehearsal process-loss self-test found unsafe pre-attempt, terminal-execution, or append-only-tail ordering."
    }

    $preAttemptStates = @(
        Resolve-RehearsalPreAttemptRecoveryState -SnapshotReceiptExists $true -SnapshotSidecarExists $true -StartReceiptExists $false -StartSidecarExists $false -AttemptReceiptExists $false -AttemptSidecarExists $false
        Resolve-RehearsalPreAttemptRecoveryState -SnapshotReceiptExists $true -SnapshotSidecarExists $true -StartReceiptExists $true -StartSidecarExists $true -AttemptReceiptExists $false -AttemptSidecarExists $false
        Resolve-RehearsalPreAttemptRecoveryState -SnapshotReceiptExists $true -SnapshotSidecarExists $true -StartReceiptExists $true -StartSidecarExists $true -AttemptReceiptExists $true -AttemptSidecarExists $true
    )
    if (($preAttemptStates -join ",") -cne "snapshot_only,start_only,attempt_checkpoint") {
        throw "Candidate Rehearsal recovery self-test did not distinguish snapshot-only, start-only, and checkpoint recovery."
    }
    foreach ($invalidPreAttempt in @(
        { Resolve-RehearsalPreAttemptRecoveryState -SnapshotReceiptExists $true -SnapshotSidecarExists $false -StartReceiptExists $false -StartSidecarExists $false -AttemptReceiptExists $false -AttemptSidecarExists $false },
        { Resolve-RehearsalPreAttemptRecoveryState -SnapshotReceiptExists $true -SnapshotSidecarExists $true -StartReceiptExists $false -StartSidecarExists $false -AttemptReceiptExists $true -AttemptSidecarExists $true }
    )) {
        try {
            & $invalidPreAttempt | Out-Null
            throw "Candidate Rehearsal recovery self-test accepted invalid pre-attempt evidence topology."
        } catch {
            if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted invalid pre-attempt evidence topology.") { throw }
        }
    }

    $placeholderSource = [pscustomobject][ordered]@{
        commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
        acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
    }
    $verifiedSource = [pscustomobject][ordered]@{
        commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"
        acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64
    }
    $recoveryAttempt = [pscustomobject][ordered]@{
        mutable_source_identity = $placeholderSource
        source_identity_verification_state = "unverified"
        source_identity_verification_failure = $null
        environment_identity = [pscustomobject][ordered]@{
            readiness_sha256 = $null; verification_state = "unverified"; verification_failure = $null
        }
        environment_fingerprint = "0" * 64
    }
    $lifecycleLane = [pscustomobject][ordered]@{
        identity_binding = "pre_authentication_lifecycle_placeholder"
        mutable_source_identity = $placeholderSource
        environment_fingerprint = "0" * 64
        result = [pscustomobject][ordered]@{
            state = "passed"; assertions_started = $true
        }
    }
    Assert-RehearsalLaneIdentityBinding `
        -LaneDocument $lifecycleLane `
        -AttemptDocument $recoveryAttempt `
        -LaneName "attempt-state-prerequisite"
    $lifecycleLane.result.state = "failed"
    Assert-RehearsalLaneIdentityBinding `
        -LaneDocument $lifecycleLane `
        -AttemptDocument $recoveryAttempt `
        -LaneName "attempt-state-prerequisite"
    $lifecycleLane.result.state = "passed"
    if (-not (Test-RehearsalRecoveredTerminalSourceBinding `
        -AttemptDocument $recoveryAttempt -RecoveredSource $verifiedSource)) {
        throw "Candidate Rehearsal recovery self-test rejected an exact pre-authentication terminal source identity."
    }
    $recoveryAttempt = Set-RehearsalRecoveredAttemptIdentity `
        -AttemptDocument $recoveryAttempt `
        -Source $verifiedSource `
        -EnvironmentFingerprint ("e" * 64) `
        -ReadinessSha256 ("f" * 64)
    $postRecoveryLifecycleLane = [pscustomobject][ordered]@{
        identity_binding = "attempt_identity"
        mutable_source_identity = $verifiedSource
        environment_fingerprint = "e" * 64
        result = [pscustomobject][ordered]@{
            state = "passed"; assertions_started = $true
        }
    }
    Assert-RehearsalLaneIdentityBinding `
        -LaneDocument $postRecoveryLifecycleLane `
        -AttemptDocument $recoveryAttempt `
        -LaneName "attempt-state-prerequisite"
    if ([string]$recoveryAttempt.source_identity_verification_state -cne "verified" -or
        [string]$recoveryAttempt.environment_identity.verification_state -cne "verified" -or
        [string]$recoveryAttempt.environment_identity.readiness_sha256 -cne ("f" * 64) -or
        -not (Test-RehearsalRecoveredTerminalSourceBinding `
            -AttemptDocument $recoveryAttempt -RecoveredSource $verifiedSource)) {
        throw "Candidate Rehearsal recovery self-test did not checkpoint authenticated identity before lane resumption."
    }
    $postRecoveryLifecycleLane.environment_fingerprint = "9" * 64
    try {
        Assert-RehearsalLaneIdentityBinding `
            -LaneDocument $postRecoveryLifecycleLane `
            -AttemptDocument $recoveryAttempt `
            -LaneName "attempt-state-prerequisite"
        throw "Candidate Rehearsal recovery self-test accepted a post-authentication lane identity mismatch."
    } catch {
        if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted a post-authentication lane identity mismatch.") { throw }
    } finally {
        $postRecoveryLifecycleLane.environment_fingerprint = "e" * 64
    }

    $jsonDeclaration = '{"name":"recovered","depends_on":[],"command":"pass","evidence_paths":[]}' | ConvertFrom-Json
    $recoveredDeclaration = ConvertTo-RehearsalDeclarationDictionary -Declaration $jsonDeclaration
    if ($recoveredDeclaration -isnot [Collections.IDictionary] -or
        -not $recoveredDeclaration.Contains("evidence_paths")) {
        throw "Candidate Rehearsal recovery self-test did not restore the canonical declaration representation."
    }
    $recoveredTimestamp = '"2026-08-08T09:36:46.6836471-04:00"' | ConvertFrom-Json
    if ((ConvertTo-RehearsalOffsetTimestampText `
        -Value $recoveredTimestamp `
        -Label "process-loss recovery self-test") -notmatch '(?:Z|[+-]\d{2}:\d{2})$') {
        throw "Candidate Rehearsal recovery self-test lost the active-lane timestamp offset."
    }

    $restorationDocument = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.materialization-result"
        attempt = 33
        passed = $true
        source = [pscustomobject][ordered]@{
            commit = $verifiedSource.commit
            tree = $verifiedSource.tree
            branch = $verifiedSource.branch
            dirty = $verifiedSource.dirty
            dirty_paths = @()
        }
        environment = [pscustomobject][ordered]@{ declared_fingerprint = "e" * 64 }
        first_apply = [pscustomobject][ordered]@{ no_op = $false }
        no_op_apply = [pscustomobject][ordered]@{ no_op = $true }
        final_health_passed = $true
    }
    Assert-RehearsalRestorationMaterializationDocument `
        -Document $restorationDocument `
        -ExpectedSource $verifiedSource `
        -ExpectedEnvironmentFingerprint ("e" * 64) `
        -ExpectedAttempt 33
    $restorationDocument.source.dirty_paths = @("tracked-change.txt")
    try {
        Assert-RehearsalRestorationMaterializationDocument `
            -Document $restorationDocument `
            -ExpectedSource $verifiedSource `
            -ExpectedEnvironmentFingerprint ("e" * 64) `
            -ExpectedAttempt 33
        throw "Candidate Rehearsal recovery self-test accepted a dirty restoration source."
    } catch {
        if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted a dirty restoration source.") { throw }
    } finally {
        $restorationDocument.source.dirty_paths = @()
    }
    $restorationDocument.no_op_apply.no_op = $false
    try {
        Assert-RehearsalRestorationMaterializationDocument `
            -Document $restorationDocument `
            -ExpectedSource $verifiedSource `
            -ExpectedEnvironmentFingerprint ("e" * 64) `
            -ExpectedAttempt 33
        throw "Candidate Rehearsal recovery self-test accepted restoration without a proven no-op apply."
    } catch {
        if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted restoration without a proven no-op apply.") { throw }
    } finally {
        $restorationDocument.no_op_apply.no_op = $true
    }

    function Resolve-RecoveryFixture {
        param([hashtable]$Overrides)
        $fixture = @{
            AttemptState = "executing"; TerminalLaneCount = 4; DeclaredLaneCount = 10; HasActiveLane = $false
            HarvestReceiptExists = $false; HarvestSidecarExists = $false
            BatchReceiptExists = $false; BatchSidecarExists = $false
            AuthorizationReceiptExists = $false; AuthorizationSidecarExists = $false
            ResultReceiptExists = $true; ResultSidecarExists = $true
            StateReceiptExists = $true; StateSidecarExists = $true
            CurrentPassingResultExists = $false; CurrentTerminalStateExists = $false
        }
        foreach ($key in $Overrides.Keys) { $fixture[$key] = $Overrides[$key] }
        Resolve-RehearsalAttemptRecoveryState @fixture
    }
    $attemptStates = @(
        Resolve-RecoveryFixture @{ HasActiveLane = $true }
        Resolve-RecoveryFixture @{ AttemptState = "harvesting" }
        Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10 }
        Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; HarvestReceiptExists = $true; HarvestSidecarExists = $true }
        Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; HarvestReceiptExists = $true; HarvestSidecarExists = $true; BatchReceiptExists = $true; BatchSidecarExists = $true }
        Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; HarvestReceiptExists = $true; HarvestSidecarExists = $true; BatchReceiptExists = $true; BatchSidecarExists = $true; AuthorizationReceiptExists = $true; AuthorizationSidecarExists = $true }
        Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; HarvestReceiptExists = $true; HarvestSidecarExists = $true; BatchReceiptExists = $true; BatchSidecarExists = $true; AuthorizationReceiptExists = $true; AuthorizationSidecarExists = $true; CurrentTerminalStateExists = $true }
        Resolve-RecoveryFixture @{ AttemptState = "passed"; TerminalLaneCount = 10; ResultReceiptExists = $false; ResultSidecarExists = $false }
        Resolve-RecoveryFixture @{ AttemptState = "passed"; TerminalLaneCount = 10; CurrentPassingResultExists = $true }
        Resolve-RecoveryFixture @{ AttemptState = "passed"; TerminalLaneCount = 10; CurrentPassingResultExists = $true; CurrentTerminalStateExists = $true }
    )
    $expectedAttemptStates = @(
        "mid_lane_checkpoint", "deferred_or_harvest_loop_checkpoint", "failed_needs_harvest",
        "failed_needs_batch", "failed_needs_authorization_or_withheld_state", "failed_needs_state",
        "failed_tail_complete", "passed_needs_result", "passed_needs_state", "passed_tail_complete"
    )
    if (($attemptStates -join ",") -cne ($expectedAttemptStates -join ",")) {
        throw "Candidate Rehearsal recovery self-test did not traverse every checkpoint and terminal-tail state."
    }
    foreach ($invalidAttempt in @(
        { Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; BatchReceiptExists = $true; BatchSidecarExists = $true } },
        { Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; AuthorizationReceiptExists = $true; AuthorizationSidecarExists = $true } },
        { Resolve-RecoveryFixture @{ AttemptState = "passed"; TerminalLaneCount = 9 } },
        { Resolve-RecoveryFixture @{ AttemptState = "passed"; TerminalLaneCount = 10; HarvestReceiptExists = $true; HarvestSidecarExists = $true } },
        { Resolve-RecoveryFixture @{ AttemptState = "failed"; TerminalLaneCount = 10; HarvestReceiptExists = $true; HarvestSidecarExists = $false } }
    )) {
        try {
            & $invalidAttempt | Out-Null
            throw "Candidate Rehearsal recovery self-test accepted an impossible attempt/tail state."
        } catch {
            if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted an impossible attempt/tail state.") { throw }
        }
    }

    $temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    $temporaryRoot = [IO.Path]::GetFullPath((Join-Path $temporaryBase "tessara-sprint-8a-recovery-$([guid]::NewGuid().ToString('N'))"))
    if (-not $temporaryRoot.StartsWith($temporaryBase, [StringComparison]::OrdinalIgnoreCase) -or
        -not [IO.Path]::GetFileName($temporaryRoot).StartsWith("tessara-sprint-8a-recovery-", [StringComparison]::Ordinal)) {
        throw "Candidate Rehearsal recovery self-test refused an unexpected temporary root."
    }
    [IO.Directory]::CreateDirectory($temporaryRoot) | Out-Null
    try {
        $path = Join-Path $temporaryRoot "immutable-tail.json"
        $document = [ordered]@{ schema_version = 1; phase = "recovery-self-test"; state = "complete" }
        $firstSha = Publish-OrAuthenticateRehearsalImmutableEvidence `
            -Document $document -Path $path -Label "Recovery self-test"
        $firstBytes = [IO.File]::ReadAllBytes($path)
        $secondSha = Publish-OrAuthenticateRehearsalImmutableEvidence `
            -Document $document -Path $path -Label "Recovery self-test"
        if ($firstSha -cne $secondSha -or
            [Convert]::ToBase64String($firstBytes) -cne
                [Convert]::ToBase64String([IO.File]::ReadAllBytes($path))) {
            throw "Candidate Rehearsal recovery self-test rewrote authenticated immutable evidence."
        }
        try {
            Publish-OrAuthenticateRehearsalImmutableEvidence `
                -Document ([ordered]@{ schema_version = 1; phase = "recovery-self-test"; state = "mutated" }) `
                -Path $path `
                -Label "Recovery self-test" | Out-Null
            throw "Candidate Rehearsal recovery self-test accepted changed immutable evidence."
        } catch {
            if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted changed immutable evidence.") { throw }
        }

        $incompletePath = Join-Path $temporaryRoot "incomplete-tail.json"
        Publish-Sprint7AEvidence -Document $document -OutputPath $incompletePath | Out-Null
        Remove-Item -LiteralPath "$incompletePath.sha256" -Force
        try {
            Publish-OrAuthenticateRehearsalImmutableEvidence `
                -Document $document -Path $incompletePath -Label "Incomplete recovery self-test" | Out-Null
            throw "Candidate Rehearsal recovery self-test accepted an incomplete immutable pair."
        } catch {
            if ($_.Exception.Message -ceq "Candidate Rehearsal recovery self-test accepted an incomplete immutable pair.") { throw }
        }
    } finally {
        if (Test-Path -LiteralPath $temporaryRoot -PathType Container) {
            [IO.Directory]::Delete($temporaryRoot, $true)
        }
    }
}

function Invoke-ApprovedR33CorrectionAuthorization {
    param([Parameter(Mandatory)][string]$ResolvedEvidenceRoot)

    if ($Attempt -ne 33 -or $ResumeInterruptedAttempt -or $SelfTest) {
        throw "The approved R33 evidence correction requires exactly -Attempt 33 and cannot be combined with resume or self-test."
    }
    $correctionSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    if ($correctionSource.dirty) {
        throw "The approved R33 evidence correction can be recorded only from a clean committed correction source."
    }

    $attemptsRoot = Join-Path $ResolvedEvidenceRoot "attempts"
    $statePath = Join-Path $ResolvedEvidenceRoot "validation-state.json"
    $lockPath = Join-Path $ResolvedEvidenceRoot "validation-attempt.lock"
    $authorizationPath = Join-Path $attemptsRoot "candidate-rehearsal-33-correction-authorization.json"
    $qualificationPath = Join-Path $attemptsRoot "candidate-rehearsal-33-correction-authorization-qualification.json"
    $lockHandle = $null
    try {
        $lockHandle = Open-Sprint8AValidationAttemptLock -Path $lockPath
        $stateSha = Assert-Sprint8AReceiptSidecar -Path $statePath
        $state = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        if ([string]$state.rehearsal.state -cne "failed" -or
            [int]$state.rehearsal.attempt -ne 33 -or
            [string]$state.rehearsal.harvest_guard -cne "passed" -or
            [string]$state.correction_authorization.state -cne "withheld" -or
            [string]$state.correction_authorization.reason -cne "correction_authorization_withheld_cleanup_not_proven" -or
            [bool]$state.preflight_eligible) {
            throw "The approved R33 evidence correction requires the exact terminal failed R33 state with completed harvest and withheld authorization."
        }
        [void]$stateSha
        if (Test-Path -LiteralPath (Join-Path $ResolvedEvidenceRoot "candidate-rehearsal-result.json") -PathType Leaf) {
            throw "The approved R33 evidence correction cannot coexist with a passing Candidate Rehearsal result."
        }
        foreach ($collision in @(
            (Join-Path $attemptsRoot "readiness-44-start.json"),
            (Join-Path $attemptsRoot "readiness-44.json"),
            (Join-Path $ResolvedEvidenceRoot "readiness-44")
        )) {
            if (Test-Path -LiteralPath $collision) {
                throw "Readiness 44 already has evidence; the approved R33 correction bridge cannot be issued after its successor started."
            }
        }

        $attemptPath = Join-Path $attemptsRoot "candidate-rehearsal-33-attempt.json"
        $harvestPath = Join-Path $attemptsRoot "candidate-rehearsal-33-harvest.json"
        $batchPath = Join-Path $attemptsRoot "candidate-rehearsal-33-defect-batch.json"
        $attemptSha = Assert-Sprint8AReceiptSidecar -Path $attemptPath
        $harvestSha = Assert-Sprint8AReceiptSidecar -Path $harvestPath
        $batchSha = Assert-Sprint8AReceiptSidecar -Path $batchPath
        $attemptDocument = Get-Content -LiteralPath $attemptPath -Raw | ConvertFrom-Json
        $harvestDocument = Get-Content -LiteralPath $harvestPath -Raw | ConvertFrom-Json
        $batchDocument = Get-Content -LiteralPath $batchPath -Raw | ConvertFrom-Json
        if ([string]$attemptDocument.state -cne "failed" -or
            [string]$attemptDocument.cleanup_restoration.result -cne "not_proven" -or
            [string]$harvestDocument.state -cne "harvest_complete" -or
            [string]$batchDocument.state -cne "open" -or
            [int]$attemptDocument.deferred_count -ne 0 -or
            [int]$harvestDocument.deferred_count -ne 0 -or
            [int]$batchDocument.deferred_count -ne 0) {
            throw "The approved R33 evidence correction found changed terminal attempt, harvest, batch, or deferral facts."
        }

        function New-R33Reference {
            param([Parameter(Mandatory)][string]$Path)
            [ordered]@{
                path = ConvertTo-Sprint8ACanonicalEvidencePath -RepositoryRoot $repoRoot -EvidenceRoot $ResolvedEvidenceRoot -Path $Path
                sha256 = Assert-Sprint8AReceiptSidecar -Path $Path
            }
        }
        $attemptReference = New-R33Reference $attemptPath
        $harvestReference = New-R33Reference $harvestPath
        $batchReference = New-R33Reference $batchPath
        $authorizationDocument = [ordered]@{
            schema_version = 2
            sprint = "sprint-8a"
            phase = "candidate-rehearsal-correction-authorization"
            attempt = 33
            authoritative = $false
            state = "authorized"
            consumption_state = "unconsumed"
            allowed_successor_phase = "validation-readiness"
            allowed_successor_attempt = 44
            allowed_successor_count = 1
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            mutable_source_identity = $attemptDocument.mutable_source_identity
            environment_fingerprint = [string]$attemptDocument.environment_fingerprint
            predecessor_attempt_receipt = $attemptReference
            harvest_receipt = $harvestReference
            defect_batch = $batchReference
            deferred_count = 0
            authorization = "tracked correction and one successor readiness attempt are permitted for this consolidated batch"
            qualification_required = $true
            qualification_kind = "approved_historical_evidence_correction"
        }
        if (Test-Path -LiteralPath $authorizationPath -PathType Leaf) {
            $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $authorizationPath
            $authorizationDocument = Get-Content -LiteralPath $authorizationPath -Raw | ConvertFrom-Json
        } else {
            Publish-Sprint7AEvidence -Document $authorizationDocument -OutputPath $authorizationPath | Out-Null
            $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $authorizationPath
        }
        $authorizationReference = [ordered]@{
            path = ConvertTo-Sprint8ACanonicalEvidencePath -RepositoryRoot $repoRoot -EvidenceRoot $ResolvedEvidenceRoot -Path $authorizationPath
            sha256 = $authorizationSha
        }

        $failedHealthLanePath = Join-Path $ResolvedEvidenceRoot "rehearsal/attempt-33/lanes/final-successor-health.json"
        $finalHealthPath = Join-Path $ResolvedEvidenceRoot "rehearsal/attempt-33/final-successor-health.json"
        $materializationPath = Join-Path $ResolvedEvidenceRoot "rehearsal/attempt-33/final-restoration/materialization/attempt-33/materialization-result.json"
        $inventoryPath = Join-Path $ResolvedEvidenceRoot "rehearsal/attempt-33/final-restoration-inventory.json"
        $finalEnvironmentPath = Join-Path $ResolvedEvidenceRoot "rehearsal/attempt-33/lanes/final-environment-identity.json"
        $qualificationDocument = [ordered]@{
            schema_version = 1
            contract = "tessara.sprint-8a.correction-authorization-qualification"
            sprint = "sprint-8a"
            phase = "candidate-rehearsal-correction-authorization-qualification"
            attempt = 33
            authoritative = $false
            state = "authorized"
            authorization_kind = "approved_historical_evidence_correction"
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            source_identity = $attemptDocument.mutable_source_identity
            approved_correction_source_identity = $correctionSource
            environment_fingerprint = [string]$attemptDocument.environment_fingerprint
            original_authorization = $authorizationReference
            qualification_basis = [ordered]@{
                immutable_attempt = $attemptReference
                harvest = $harvestReference
                defect_batch = $batchReference
                failed_final_health_lane = New-R33Reference $failedHealthLanePath
                final_health_evidence = New-R33Reference $finalHealthPath
                materialization_result = New-R33Reference $materializationPath
                inventory_navigation = New-R33Reference $inventoryPath
                final_environment_lane = New-R33Reference $finalEnvironmentPath
            }
            authorization_effect = [ordered]@{
                original_authorization_qualified = $true
                correction_scope = "one consolidated tracked correction batch"
                allowed_successor_phase = "validation-readiness"
                allowed_successor_attempt = 44
                allowed_successor_count = 1
                requires_exact_tuple = $true
                independently_consumable = $false
                consumption_state = "unconsumed"
            }
            lifecycle_guards = [ordered]@{
                predecessor_rehearsal_state = "failed"
                candidate_rehearsal_result_authorized = $false
                preflight_authorized = $false
                candidate_freeze_authorized = $false
                sit_authorized = $false
                uat_authorized = $false
                closeout_authorized = $false
                next_formal_cycle_started = $false
            }
            statement = "The immutable R33 failure remains unchanged. Exact retained recovery evidence proves that its old comparator misread a successful clean apply, no-op, health check, inventory check, and environment check. This tuple permits only complete Readiness 44 and does not certify R33 or any downstream phase."
        }
        if (Test-Path -LiteralPath $qualificationPath -PathType Leaf) {
            $qualificationSha = Assert-Sprint8AReceiptSidecar -Path $qualificationPath
        } else {
            Publish-Sprint7AEvidence -Document $qualificationDocument -OutputPath $qualificationPath | Out-Null
            $qualificationSha = Assert-Sprint8AReceiptSidecar -Path $qualificationPath
        }
        $qualificationReference = [ordered]@{
            path = ConvertTo-Sprint8ACanonicalEvidencePath -RepositoryRoot $repoRoot -EvidenceRoot $ResolvedEvidenceRoot -Path $qualificationPath
            sha256 = $qualificationSha
        }

        $lineage = Add-Sprint8ACorrectionLineageLink `
            -Lineage $state.correction_lineage `
            -Predecessor ([ordered]@{
                phase = "candidate-rehearsal"; attempt = 33
                receipt = $attemptReference; harvest = $harvestReference; defect_batch = $batchReference
            }) `
            -Authorization $authorizationReference `
            -ConsumedByReadiness $null
        $lineage.links[-1]["authorization_qualification"] = $qualificationReference
        [void](Assert-Sprint8ACorrectionLineage -Lineage $lineage -RepositoryRoot $repoRoot -EvidenceRoot $ResolvedEvidenceRoot)
        $state.updated_at = [DateTimeOffset]::UtcNow.ToString("o")
        $state.correction_lineage = $lineage
        $state.correction_authorization = [pscustomobject][ordered]@{
            state = "authorized"
            path = [string]$authorizationReference.path
            sha256 = [string]$authorizationReference.sha256
            qualification = $qualificationReference
            allowed_successor_attempt = 44
        }
        $state.preflight_eligible = $false
        Publish-Sprint7AEvidence -Document $state -OutputPath $statePath -Overwrite | Out-Null
        [void](Assert-Sprint8AReceiptSidecar -Path $statePath)
        Write-Host "Recorded the approved R33 evidence correction. The immutable R33 failure remains failed; exactly Readiness 44 may start next."
    } finally {
        if ($null -ne $lockHandle) { $lockHandle.Dispose() }
    }
}

if ($SelfTest) {
    if ($ResumeInterruptedAttempt -or $AuthorizeApprovedR33Correction) { throw "Candidate Rehearsal self-test cannot be combined with recovery or R33 evidence correction." }
    Test-Sprint8AExclusiveValidationLock
    Test-RehearsalScheduler
    Test-RehearsalReadinessLaneIsolation
    Test-Sprint8AFirstRehearsalCorrectionLink
    Test-RehearsalPowerShellCheck
    Test-RehearsalClassificationResolution
    Test-Sprint8ACandidateTwoWaveRunnerContract
    Test-Sprint8AProcessLossRecoveryContract
    Test-Sprint8AResultClassificationProjection
    Test-Sprint8AEnvironmentContractComparison
    Test-Sprint8AEvidenceReferenceResolution
    Test-Sprint8ARehearsalTwoWaveScheduler
    $canonicalSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $canonicalSource | Out-Null
    if ($canonicalSource.GetType().FullName -cne "System.Management.Automation.PSCustomObject") {
        throw "Canonical Sprint 8A source identity helper did not return a PSCustomObject."
    }
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
    Write-Host "Sprint 8A candidate rehearsal fail-late, process-loss recovery, immutable-tail, and nested-UAT authority self-test passed."
    return
}

if ($AuthorizeApprovedR33Correction) {
    $resolvedCorrectionEvidenceRoot = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
    }
    Invoke-ApprovedR33CorrectionAuthorization -ResolvedEvidenceRoot $resolvedCorrectionEvidenceRoot
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
$startPath = Join-Path $evidenceRootPath "attempts/candidate-rehearsal-$Attempt-start.json"
$stateSnapshotPath = Join-Path $evidenceRootPath "attempts/candidate-rehearsal-$Attempt-validation-state.json"
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
[IO.Directory]::CreateDirectory((Split-Path -Parent $attemptPath)) | Out-Null
$validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath
$resumeHasStart = $false
$resumeHasAttempt = $false
if ($ResumeInterruptedAttempt) {
    foreach ($pair in @(
        [pscustomobject]@{ label = "immutable validation-state capture"; path = $stateSnapshotPath; required = $true },
        [pscustomobject]@{ label = "immutable start"; path = $startPath; required = $false },
        [pscustomobject]@{ label = "attempt checkpoint"; path = $attemptPath; required = $false }
    )) {
        Repair-Sprint7AEvidencePublication -Path ([string]$pair.path)
        $receiptExists = Test-Path -LiteralPath ([string]$pair.path) -PathType Leaf
        $sidecarExists = Test-Path -LiteralPath "$([string]$pair.path).sha256" -PathType Leaf
        if ($receiptExists -xor $sidecarExists -or ([bool]$pair.required -and -not $receiptExists)) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal process-loss recovery requires a complete $([string]$pair.label) receipt/sidecar pair."
        }
    }
    $resumeHasStart = Test-Path -LiteralPath $startPath -PathType Leaf
    $resumeHasAttempt = Test-Path -LiteralPath $attemptPath -PathType Leaf
    $preAttemptRecoveryState = Resolve-RehearsalPreAttemptRecoveryState `
        -SnapshotReceiptExists (Test-Path -LiteralPath $stateSnapshotPath -PathType Leaf) `
        -SnapshotSidecarExists (Test-Path -LiteralPath "$stateSnapshotPath.sha256" -PathType Leaf) `
        -StartReceiptExists $resumeHasStart `
        -StartSidecarExists (Test-Path -LiteralPath "$startPath.sha256" -PathType Leaf) `
        -AttemptReceiptExists $resumeHasAttempt `
        -AttemptSidecarExists (Test-Path -LiteralPath "$attemptPath.sha256" -PathType Leaf)
    if (($resumeHasAttempt -and $preAttemptRecoveryState -cne "attempt_checkpoint") -or
        (-not $resumeHasAttempt -and $resumeHasStart -and $preAttemptRecoveryState -cne "start_only") -or
        (-not $resumeHasAttempt -and -not $resumeHasStart -and $preAttemptRecoveryState -cne "snapshot_only")) {
        throw "Candidate Rehearsal process-loss recovery resolved an inconsistent pre-attempt state '$preAttemptRecoveryState'."
    }
    if ($resumeHasAttempt -and -not (Test-Path -LiteralPath $attemptRoot -PathType Container)) {
        $validationLockHandle.Dispose(); $validationLockHandle = $null
        throw "Candidate Rehearsal process-loss recovery found an attempt checkpoint without its retained attempt evidence root."
    }
    [IO.Directory]::CreateDirectory($laneRoot) | Out-Null
    [IO.Directory]::CreateDirectory($logRoot) | Out-Null
    if (-not $resumeHasAttempt -and
        (@(Get-ChildItem -LiteralPath $laneRoot -Force).Count -gt 0 -or
            @(Get-ChildItem -LiteralPath $logRoot -Force).Count -gt 0 -or
            @(Test-Path -LiteralPath @(
                    $harvestPath, "$harvestPath.sha256",
                    $batchPath, "$batchPath.sha256",
                    $correctionAuthorizationPath, "$correctionAuthorizationPath.sha256") |
                Where-Object { $_ }).Count -gt 0)) {
        $validationLockHandle.Dispose(); $validationLockHandle = $null
        throw "Candidate Rehearsal pre-attempt recovery found lane or terminal-tail evidence without an authenticated attempt checkpoint."
    }
} else {
    foreach ($path in @($startPath, "$startPath.sha256", $stateSnapshotPath, "$stateSnapshotPath.sha256", $attemptPath, "$attemptPath.sha256", $attemptRoot)) {
        if (Test-Path -LiteralPath $path) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate rehearsal attempt $Attempt already exists and cannot be reused: $path"
        }
    }
    [IO.Directory]::CreateDirectory($laneRoot) | Out-Null
    [IO.Directory]::CreateDirectory($logRoot) | Out-Null
}

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
$finalEnvironmentEvidence = Join-Path $attemptRoot "final-environment-identity.json"
$finalHealthEvidence = Join-Path $attemptRoot "final-successor-health.json"
$finalRestorationInventory = Join-Path $attemptRoot "final-restoration-inventory.json"
$finalRestorationRoot = Join-Path $attemptRoot "final-restoration"

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
    [ordered]@{ name = "workspace-tests"; depends_on = @(); command = "cargo test --workspace --all-features --locked --offline --jobs 1"; classification = "product"; evidence_paths = @() },
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
    [ordered]@{ name = "final-environment-identity"; depends_on = @("validation-readiness-prerequisite"); command = "final authenticated unchanged environment identity"; classification = "environment"; evidence_paths = @($finalEnvironmentEvidence) },
    [ordered]@{ name = "final-successor-health"; depends_on = @("attempt-state-prerequisite", "validation-readiness-prerequisite"); command = "perform terminal source-exact canonical restoration, then prove topology, inventory, Core health, and Supervisor readiness"; classification = "environment"; evidence_paths = @($finalHealthEvidence, $finalRestorationInventory); evidence_roots = @($finalRestorationRoot) }
)
$lanePolicies = @(Get-Sprint8ARehearsalLanePolicies)
if ($lanePolicies.Count -ne $declaredChecks.Count) {
    throw "Candidate Rehearsal scheduler policy inventory differs from the declared lane inventory."
}
foreach ($declaration in $declaredChecks) {
    $policy = @($lanePolicies | Where-Object name -CEQ ([string]$declaration.name))
    if ($policy.Count -ne 1) { throw "Candidate Rehearsal lane '$($declaration.name)' lacks one canonical scheduler policy." }
    $declaration.depends_on = @($policy[0].depends_on)
    $declaration["scheduler_role"] = [string]$policy[0].scheduler_role
    $declaration["impact_paths"] = @($policy[0].impact_paths)
    $declaration["impact_sources"] = @($policy[0].impact_sources)
    foreach ($pathProperty in @("evidence_paths", "evidence_roots")) {
        if ($declaration.Contains($pathProperty)) {
            $declaration[$pathProperty] = @($declaration[$pathProperty] | ForEach-Object {
                ConvertTo-Sprint8ACanonicalEvidencePath `
                    -RepositoryRoot $repoRoot `
                    -EvidenceRoot $evidenceRootPath `
                    -Path ([string]$_)
            })
        }
    }
    if ($declaration.Contains("nested_results_path")) {
        $declaration["nested_results_path"] = ConvertTo-Sprint8ACanonicalEvidencePath `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -Path ([string]$declaration.nested_results_path)
    }
}
Assert-Sprint8ADeclaredEvidencePaths `
    -Checks $declaredChecks `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $evidenceRootPath `
    -Label "Candidate Rehearsal $Attempt declarations" | Out-Null
foreach ($declaration in $declaredChecks | Where-Object { $_.Contains("nested_results_path") }) {
    Assert-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -Path ([string]$declaration.nested_results_path) `
        -Label "Candidate Rehearsal nested result" | Out-Null
}
Assert-RehearsalGraph -Checks $declaredChecks
Assert-Sprint8ARehearsalSchedulerDeclarations -Checks $declaredChecks
Assert-RehearsalIndependentChecks -Checks $declaredChecks -IndependentNames @(
    "attempt-state-prerequisite", "validation-readiness-prerequisite", "formatting", "workspace-check", "workspace-clippy",
    "compose-manifest-schema-contract", "web-native-wasm-source-boundaries", "module-sdk-boundaries",
    "dashboard-source-boundaries", "markdown-links", "workspace-tests",
    "optimized-resource-reference-timing", "components-contract-tests", "dashboard-module-tests",
    "component-conformance-nondisclosure", "module-testkit-conformance", "playwright-discovery"
)

$relativeReadinessPath = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
$relativeStatePath = [IO.Path]::GetRelativePath($repoRoot, $statePath).Replace("\", "/")
$relativeStateSnapshotPath = [IO.Path]::GetRelativePath($repoRoot, $stateSnapshotPath).Replace("\", "/")
$relativeStartPath = [IO.Path]::GetRelativePath($repoRoot, $startPath).Replace("\", "/")
$selectedSchedule = $null
$scheduleSelection = $null
$immutableStartReceipt = $null
$prelaunchStateCapture = $null
$claimedReadinessSha = "0" * 64
$startReadinessReference = [ordered]@{ path = $relativeReadinessPath; sha256 = $claimedReadinessSha }

if ($ResumeInterruptedAttempt -and $resumeHasStart) {
    $startSha = Assert-Sprint8AReceiptSidecar -Path $startPath
    $startDocument = Get-Content -LiteralPath $startPath -Raw | ConvertFrom-Json
    $historicalDeclaredChecks = @($startDocument.declared_checks | ForEach-Object {
        ConvertTo-RehearsalDeclarationDictionary -Declaration $_
    })
    if (($startDocument.schema_version -isnot [int] -and $startDocument.schema_version -isnot [long]) -or
        [int]$startDocument.schema_version -ne 3 -or
        [string]$startDocument.phase -cne "candidate-rehearsal-start" -or
        [int]$startDocument.attempt -ne $Attempt -or
        $startDocument.PSObject.Properties.Name -notcontains "declared_checks" -or
        (-not $resumeHasAttempt -and
            ($historicalDeclaredChecks | ConvertTo-Json -Depth 30 -Compress) -cne
                ($declaredChecks | ConvertTo-Json -Depth 30 -Compress)) -or
        [string]$startDocument.readiness_receipt.path -cne $relativeReadinessPath -or
        [string]$startDocument.schedule_sha256 -cne (Get-Sprint8ARehearsalJsonSha256 -Document $startDocument.schedule)) {
        $validationLockHandle.Dispose(); $validationLockHandle = $null
        throw "Candidate Rehearsal recovery rejected a mutated or malformed immutable start receipt."
    }
    [void](Assert-Sprint8ARehearsalScheduleContract `
        -Schedule $startDocument.schedule -Checks $historicalDeclaredChecks -ExpectedAttempt $Attempt)
    if ($resumeHasAttempt) {
        # An immutable attempt checkpoint owns its immutable declarations. A later
        # tracked correction may legitimately change the runner before terminal-tail
        # finalization, but it cannot change which lanes the retained attempt ran.
        $declaredChecks = $historicalDeclaredChecks
    }
    $stateSnapshotSha = Assert-Sprint8AReceiptSidecar -Path $stateSnapshotPath
    if ([string]$startDocument.validation_state_receipt.path -cne $relativeStateSnapshotPath -or
        [string]$startDocument.validation_state_receipt.sha256 -cne $stateSnapshotSha) {
        $validationLockHandle.Dispose(); $validationLockHandle = $null
        throw "Candidate Rehearsal recovery rejected a changed immutable validation-state capture."
    }
    $selectedSchedule = $startDocument.schedule
    $scheduleSelection = $startDocument.schedule_selection
    $claimedReadinessSha = [string]$startDocument.readiness_receipt.sha256
    $immutableStartReceipt = [ordered]@{ path = $relativeStartPath; sha256 = $startSha }
} else {
    $captureError = $null
    $capturedStateSha = $null
    $capturedState = $null
    if ($ResumeInterruptedAttempt) {
        $stateSnapshotSha = Assert-Sprint8AReceiptSidecar -Path $stateSnapshotPath
        $prelaunchStateCapture = Get-Content -LiteralPath $stateSnapshotPath -Raw | ConvertFrom-Json
        if (($prelaunchStateCapture.schema_version -isnot [int] -and $prelaunchStateCapture.schema_version -isnot [long]) -or
            [int]$prelaunchStateCapture.schema_version -ne 1 -or
            [string]$prelaunchStateCapture.phase -cne "candidate-rehearsal-validation-state-capture" -or
            [int]$prelaunchStateCapture.attempt -ne $Attempt -or
            [string]$prelaunchStateCapture.captured_path -cne $relativeStatePath) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal process-loss recovery rejected a malformed immutable validation-state capture."
        }
        $captureError = $prelaunchStateCapture.capture_error
        $capturedStateSha = $prelaunchStateCapture.captured_sha256
        $capturedState = $prelaunchStateCapture.document
    } else {
        try {
            $capturedStateSha = Assert-Sprint8AReceiptSidecar -Path $statePath
            $capturedState = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        } catch {
            $captureError = $_.Exception.Message
        }
        $prelaunchStateCapture = [ordered]@{
            schema_version = 1
            sprint = "sprint-8a"
            phase = "candidate-rehearsal-validation-state-capture"
            attempt = $Attempt
            authoritative = $false
            captured_at = [DateTimeOffset]::UtcNow.ToString("o")
            captured_path = $relativeStatePath
            captured_sha256 = $capturedStateSha
            capture_error = $captureError
            document = $capturedState
        }
        Publish-Sprint7AEvidence -Document $prelaunchStateCapture -OutputPath $stateSnapshotPath | Out-Null
        $stateSnapshotSha = Assert-Sprint8AReceiptSidecar -Path $stateSnapshotPath
    }

    $selectionFailure = $null
    try {
        $claimedReadinessSha = Assert-Sprint8AReceiptSidecar -Path $readinessPath
        $claimedReadiness = Get-Content -LiteralPath $readinessPath -Raw | ConvertFrom-Json
        if (($claimedReadiness.schema_version -isnot [int] -and $claimedReadiness.schema_version -isnot [long]) -or
            [int]$claimedReadiness.schema_version -ne 3 -or
            [string]$claimedReadiness.phase -cne "validation-readiness" -or
            [string]$claimedReadiness.state -cne "passed" -or
            $claimedReadiness.PSObject.Properties.Name -notcontains "next_candidate_rehearsal" -or
            [int]$claimedReadiness.next_candidate_rehearsal.attempt -ne $Attempt -or
            [string]$claimedReadiness.next_candidate_rehearsal.schedule_sha256 -cne
                (Get-Sprint8ARehearsalJsonSha256 -Document $claimedReadiness.next_candidate_rehearsal.schedule)) {
            throw "Passing Readiness does not contain the exact authenticated next Candidate Rehearsal schedule."
        }
        [void](Assert-Sprint8ARehearsalScheduleContract `
            -Schedule $claimedReadiness.next_candidate_rehearsal.schedule `
            -Checks $declaredChecks `
            -ExpectedAttempt $Attempt)
        if ($null -eq $capturedState -or $null -ne $captureError -or
            [string]$capturedState.readiness.receipt -cne $relativeReadinessPath -or
            [string]$capturedState.readiness.sha256 -cne $claimedReadinessSha -or
            [int]$capturedState.next_candidate_rehearsal.attempt -ne $Attempt -or
            [string]$capturedState.next_candidate_rehearsal.schedule_sha256 -cne
                [string]$claimedReadiness.next_candidate_rehearsal.schedule_sha256 -or
            [bool]$capturedState.preflight_eligible) {
            throw "Prelaunch validation state does not bind the supplied Readiness schedule."
        }
        $prelaunchReadinessValidation = Assert-Sprint8ACurrentReadinessReference `
            -StateReadiness $capturedState.readiness `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -RequirePassed
        $startReadinessReference = [ordered]@{
            path = [string]$prelaunchReadinessValidation.immutable.path
            sha256 = [string]$prelaunchReadinessValidation.immutable.sha256
        }
        $selectedSchedule = $claimedReadiness.next_candidate_rehearsal.schedule
        $scheduleSelection = [ordered]@{
            source = "readiness_bound"
            reason = if ([bool]$claimedReadiness.next_candidate_rehearsal.conservative_fallback) {
                "Readiness authenticated the conservative full-harvest schedule."
            } else { "Readiness authenticated bounded failure-first history and correction impact." }
        }
    } catch {
        $selectionFailure = $_.Exception.Message
        $selectedSchedule = Resolve-Sprint8ARehearsalSchedule `
            -Checks $declaredChecks `
            -Attempt $Attempt `
            -LaneHistory @{} `
            -ChangedPaths @() `
            -AcceptanceInventoryChanged:$false `
            -DeploymentInputsChanged:$false `
            -EnvironmentContractChanged:$false `
            -HistoryAuthenticated:$false `
            -FallbackReason $selectionFailure
        [void](Assert-Sprint8ARehearsalScheduleContract `
            -Schedule $selectedSchedule -Checks $declaredChecks -ExpectedAttempt $Attempt)
        $scheduleSelection = [ordered]@{
            source = "conservative_full_harvest"
            reason = $selectionFailure
        }
    }
    $scheduleSha = Get-Sprint8ARehearsalJsonSha256 -Document $selectedSchedule
    $startDocument = [ordered]@{
        schema_version = 3
        sprint = "sprint-8a"
        phase = "candidate-rehearsal-start"
        attempt = $Attempt
        authoritative = $false
        created_at = [DateTimeOffset]::UtcNow.ToString("o")
        readiness_receipt = $startReadinessReference
        validation_state_receipt = [ordered]@{ path = $relativeStateSnapshotPath; sha256 = $stateSnapshotSha }
        schedule = $selectedSchedule
        schedule_sha256 = $scheduleSha
        schedule_selection = $scheduleSelection
        declared_lanes = @($declaredChecks | ForEach-Object { [string]$_.name })
        declared_checks = $declaredChecks
        diagnostic_history_notice = $script:Sprint8ADiagnosticHistoryNotice
    }
    Publish-Sprint7AEvidence -Document $startDocument -OutputPath $startPath | Out-Null
    $startSha = Assert-Sprint8AReceiptSidecar -Path $startPath
    $immutableStartReceipt = [ordered]@{ path = $relativeStartPath; sha256 = $startSha }
}
$scheduleSha = [string]$startDocument.schedule_sha256

$source = [pscustomobject][ordered]@{
    commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
    acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
}
$runtimeContext = [ordered]@{
    readiness = $null
    readiness_sha256 = $null
    readiness_immutable_reference = $null
    validation_state = $null
    correction_lineage = $null
    launch_authorized = $false
    source_verification_state = "unverified"
    source_verification_failure = $null
    environment = [ordered]@{ fingerprint = "0" * 64; contract = $null; verified = $false; verification_state = "unverified" }
}

$terminalChecks = [Collections.Generic.List[object]]::new()
$terminalByName = @{}
$assertionsStartedAt = $null
$orphanedLaneName = $null
$recoveredPublishedTerminal = $false
$recoveredTerminalAttempt = $false
$attemptShaBeforeRecovery = $null

if ($ResumeInterruptedAttempt -and $resumeHasAttempt) {
    $attemptShaBeforeRecovery = Assert-Sprint8AReceiptSidecar -Path $attemptPath
    $attemptReceipt = Get-Content -LiteralPath $attemptPath -Raw | ConvertFrom-Json
    if (($attemptReceipt.schema_version -isnot [int] -and $attemptReceipt.schema_version -isnot [long]) -or
        [int]$attemptReceipt.schema_version -ne 3 -or
        [string]$attemptReceipt.phase -cne "candidate-rehearsal" -or
        [int]$attemptReceipt.attempt -ne $Attempt -or
        @("preparing", "executing", "harvesting", "passed", "failed") -cnotcontains [string]$attemptReceipt.state -or
        $attemptReceipt.PSObject.Properties.Name -notcontains "declared_checks" -or
        ($attemptReceipt.declared_checks | ConvertTo-Json -Depth 30 -Compress) -cne
            ($startDocument.declared_checks | ConvertTo-Json -Depth 30 -Compress) -or
        [string]$attemptReceipt.immutable_start_receipt.path -cne $relativeStartPath -or
        [string]$attemptReceipt.immutable_start_receipt.sha256 -cne [string]$immutableStartReceipt.sha256 -or
        [string]$attemptReceipt.schedule_sha256 -cne $scheduleSha) {
        $validationLockHandle.Dispose(); $validationLockHandle = $null
        throw "Candidate Rehearsal process-loss recovery rejected an inauthentic attempt checkpoint."
    }
    $recoveredTerminalAttempt = @("passed", "failed") -ccontains [string]$attemptReceipt.state
    if (-not $recoveredTerminalAttempt) {
        [void](Assert-Sprint8ARehearsalRecoveryScheduleBinding `
            -StartDocument $startDocument `
            -AttemptDocument $attemptReceipt `
            -Checks $declaredChecks `
            -ExpectedAttempt $Attempt `
            -ExpectedStartPath $relativeStartPath `
            -ExpectedStartSha256 ([string]$immutableStartReceipt.sha256))
    }
    $startedAt = [DateTimeOffset]::Parse([string]$attemptReceipt.started_at)
    foreach ($terminal in @($attemptReceipt.checks)) {
        if ($terminalByName.ContainsKey([string]$terminal.name) -or
            @($declaredChecks.name) -cnotcontains [string]$terminal.name -or
            @("passed", "failed", "blocked", "deferred") -cnotcontains [string]$terminal.state) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal recovery checkpoint has invalid terminal lane accounting."
        }
        $terminalChecks.Add($terminal)
        $terminalByName[[string]$terminal.name] = $terminal
    }
    foreach ($declaration in $declaredChecks) {
        $name = [string]$declaration.name
        if ($terminalByName.ContainsKey($name)) { continue }
        $laneReceiptPath = Join-Path $laneRoot "$name.json"
        Repair-Sprint7AEvidencePublication -Path $laneReceiptPath
        if (-not (Test-Path -LiteralPath $laneReceiptPath -PathType Leaf)) { continue }
        $laneSha = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
        $laneDocument = Get-Content -LiteralPath $laneReceiptPath -Raw | ConvertFrom-Json
        if ([string]$laneDocument.phase -cne "candidate-rehearsal-lane" -or
            [int]$laneDocument.attempt -ne $Attempt -or
            [string]$laneDocument.result.name -cne $name) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal recovery found a lane receipt outside the immutable attempt identity."
        }
        if (@("passed", "failed", "blocked", "deferred") -ccontains [string]$laneDocument.result.state) {
            $terminal = $laneDocument.result
            $terminal | Add-Member -NotePropertyName lane_receipt -NotePropertyValue ([pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $laneReceiptPath).Replace("\", "/")
                sha256 = $laneSha
            }) -Force
            $terminalChecks.Add($terminal)
            $terminalByName[$name] = $terminal
            $recoveredPublishedTerminal = $true
            if ($null -ne $attemptReceipt.active_lane -and [string]$attemptReceipt.active_lane -ceq $name) {
                $attemptReceipt.active_lane = $null
                $attemptReceipt.active_lane_started_at = $null
            }
        } elseif ([string]$laneDocument.result.state -cne "executing" -or
            $null -eq $attemptReceipt.active_lane -or [string]$attemptReceipt.active_lane -cne $name) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal recovery found an unaccounted nonterminal lane receipt."
        }
    }
    if ($attemptReceipt.PSObject.Properties.Name -contains "active_lane" -and
        $null -ne $attemptReceipt.active_lane -and
        -not $terminalByName.ContainsKey([string]$attemptReceipt.active_lane)) {
        $orphanedLaneName = [string]$attemptReceipt.active_lane
    }
    foreach ($terminal in @($terminalChecks)) {
        Assert-RehearsalRecoveredTerminalLaneReceipt `
            -Terminal $terminal `
            -AttemptDocument $attemptReceipt
    }
    if ($recoveredTerminalAttempt -and $recoveredPublishedTerminal) {
        throw "Candidate Rehearsal terminal-tail recovery rejected lane evidence omitted from the immutable terminal attempt receipt."
    }
    foreach ($tailPath in @($harvestPath, $batchPath, $correctionAuthorizationPath, $resultPath, $statePath)) {
        Repair-Sprint7AEvidencePublication -Path $tailPath
    }
    $harvestReceiptExists = Test-Path -LiteralPath $harvestPath -PathType Leaf
    $harvestSidecarExists = Test-Path -LiteralPath "$harvestPath.sha256" -PathType Leaf
    $batchReceiptExists = Test-Path -LiteralPath $batchPath -PathType Leaf
    $batchSidecarExists = Test-Path -LiteralPath "$batchPath.sha256" -PathType Leaf
    $authorizationReceiptExists = Test-Path -LiteralPath $correctionAuthorizationPath -PathType Leaf
    $authorizationSidecarExists = Test-Path -LiteralPath "$correctionAuthorizationPath.sha256" -PathType Leaf
    $resultReceiptExists = Test-Path -LiteralPath $resultPath -PathType Leaf
    $resultSidecarExists = Test-Path -LiteralPath "$resultPath.sha256" -PathType Leaf
    $stateReceiptExists = Test-Path -LiteralPath $statePath -PathType Leaf
    $stateSidecarExists = Test-Path -LiteralPath "$statePath.sha256" -PathType Leaf
    $currentPassingResultExists = $false
    if ($resultReceiptExists -and $resultSidecarExists) {
        [void](Assert-Sprint8AReceiptSidecar -Path $resultPath)
        $candidateResultTail = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
        $currentPassingResultExists = [int]$candidateResultTail.attempt -eq $Attempt -and
            [string]$candidateResultTail.state -ceq "passed"
    }
    $currentTerminalStateExists = $false
    if ($stateReceiptExists -and $stateSidecarExists) {
        [void](Assert-Sprint8AReceiptSidecar -Path $statePath)
        $validationStateTail = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        $currentTerminalStateExists = @("failed", "passed") -ccontains [string]$attemptReceipt.state -and
            [int]$validationStateTail.rehearsal.attempt -eq $Attempt -and
            [string]$validationStateTail.rehearsal.state -ceq [string]$attemptReceipt.state
    }
    $attemptRecoveryState = Resolve-RehearsalAttemptRecoveryState `
        -AttemptState ([string]$attemptReceipt.state) `
        -TerminalLaneCount $terminalChecks.Count `
        -DeclaredLaneCount $declaredChecks.Count `
        -HasActiveLane ($null -ne $attemptReceipt.active_lane) `
        -HarvestReceiptExists $harvestReceiptExists `
        -HarvestSidecarExists $harvestSidecarExists `
        -BatchReceiptExists $batchReceiptExists `
        -BatchSidecarExists $batchSidecarExists `
        -AuthorizationReceiptExists $authorizationReceiptExists `
        -AuthorizationSidecarExists $authorizationSidecarExists `
        -ResultReceiptExists $resultReceiptExists `
        -ResultSidecarExists $resultSidecarExists `
        -StateReceiptExists $stateReceiptExists `
        -StateSidecarExists $stateSidecarExists `
        -CurrentPassingResultExists $currentPassingResultExists `
        -CurrentTerminalStateExists $currentTerminalStateExists
    if ($recoveredTerminalAttempt) {
        if (-not ([string]$attemptRecoveryState).StartsWith("$([string]$attemptReceipt.state)_", [StringComparison]::Ordinal)) {
            throw "Candidate Rehearsal terminal-tail recovery resolved an inconsistent recovery state '$attemptRecoveryState'."
        }
    } elseif (-not ([string]$attemptRecoveryState).EndsWith("checkpoint", [StringComparison]::Ordinal)) {
        throw "Candidate Rehearsal lane-loop recovery resolved an inconsistent recovery state '$attemptRecoveryState'."
    }
    if ($recoveredTerminalAttempt) {
        if ($null -ne $attemptReceipt.active_lane -or $null -ne $attemptReceipt.active_lane_started_at -or
            [string]::IsNullOrWhiteSpace([string]$attemptReceipt.ended_at)) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal terminal-tail recovery rejected an active or undated terminal attempt."
        }
        [void](Assert-Sprint8ARehearsalScheduleContract `
            -Schedule $startDocument.schedule -Checks $declaredChecks -ExpectedAttempt $Attempt)
        [void](Assert-Sprint8ARehearsalTerminalAccounting `
            -DeclaredChecks $declaredChecks `
            -TerminalChecks @($terminalChecks) `
            -Attempt $Attempt `
            -AttemptState ([string]$attemptReceipt.state))

        $recoveredFailed = @($terminalChecks | Where-Object state -CEQ "failed")
        $recoveredBlocked = @($terminalChecks | Where-Object state -CEQ "blocked")
        $recoveredDeferred = @($terminalChecks | Where-Object state -CEQ "deferred")
        $recoveredNestedBlocked = @($terminalChecks | ForEach-Object { @($_.nested_blocked_checks) })
        $recoveredNestedFailed = @($terminalChecks | ForEach-Object { @($_.nested_failed_checks) })
        $recoveredAssertions = @($terminalChecks | Where-Object assertions_started -EQ $true)
        $shouldHavePassed = $recoveredFailed.Count -eq 0 -and
            $recoveredBlocked.Count -eq 0 -and
            $recoveredDeferred.Count -eq 0 -and
            $recoveredNestedBlocked.Count -eq 0 -and
            $recoveredNestedFailed.Count -eq 0
        $cleanupProven = @(@("final-successor-health", "final-environment-identity") | Where-Object {
            -not $terminalByName.ContainsKey($_) -or [string]$terminalByName[$_].state -cne "passed"
        }).Count -eq 0
        if ([int]$attemptReceipt.assertion_count -ne $recoveredAssertions.Count -or
            [int]$attemptReceipt.failure_count -ne $recoveredFailed.Count -or
            [int]$attemptReceipt.blocked_count -ne $recoveredBlocked.Count -or
            [int]$attemptReceipt.deferred_count -ne $recoveredDeferred.Count -or
            [int]$attemptReceipt.nested_blocked_count -ne $recoveredNestedBlocked.Count -or
            [int]$attemptReceipt.nested_failure_count -ne $recoveredNestedFailed.Count -or
            $attemptReceipt.cleanup_restoration.required -ne $true -or
            ([string]$attemptReceipt.cleanup_restoration.result -ceq "canonical_successor_healthy") -ne $cleanupProven -or
            (([string]$attemptReceipt.state -ceq "passed") -ne $shouldHavePassed)) {
            $validationLockHandle.Dispose(); $validationLockHandle = $null
            throw "Candidate Rehearsal terminal-tail recovery rejected inconsistent terminal attempt counts or state."
        }

    }
    $stateCapture = Get-Content -LiteralPath $stateSnapshotPath -Raw | ConvertFrom-Json
    $validatedReadinessSha = Assert-Sprint8AReceiptSidecar -Path $readinessPath
    $validatedReadiness = Get-Content -LiteralPath $readinessPath -Raw | ConvertFrom-Json
    $recoveredSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    $recoveredEnvironment = Get-Sprint8AEnvironmentContract `
        -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -ProbeDatabases
    $terminalHistoricalSourceAuthenticated = -not $recoveredTerminalAttempt -or
        (Test-RehearsalRecoveredTerminalSourceBinding `
            -AttemptDocument $attemptReceipt `
            -RecoveredSource $validatedReadiness.mutable_source_identity)
    if ($null -ne $stateCapture.capture_error -or
        [string]$validatedReadinessSha -cne [string]$startDocument.readiness_receipt.sha256 -or
        (-not $recoveredTerminalAttempt -and
            ($recoveredSource | ConvertTo-Json -Depth 20 -Compress) -cne
                ($validatedReadiness.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress)) -or
        [string]$recoveredEnvironment.fingerprint -cne [string]$validatedReadiness.environment_fingerprint -or
        [string]$attemptReceipt.environment_fingerprint -notin @(("0" * 64), [string]$recoveredEnvironment.fingerprint) -or
        -not $terminalHistoricalSourceAuthenticated) {
        $validationLockHandle.Dispose(); $validationLockHandle = $null
        throw "Candidate Rehearsal recovery source, environment, Readiness, or state capture authentication failed."
    }
    $source = $recoveredSource
    $runtimeContext.readiness = $validatedReadiness
    $runtimeContext.readiness_sha256 = $validatedReadinessSha
    $runtimeContext.validation_state = $stateCapture.document
    $runtimeContext.correction_lineage = $attemptReceipt.correction_lineage
    $runtimeContext.launch_authorized = $terminalByName.ContainsKey("attempt-state-prerequisite") -and
        [string]$terminalByName["attempt-state-prerequisite"].state -ceq "passed"
    $runtimeContext.source_verification_state = "verified"
    $runtimeContext.environment.fingerprint = [string]$recoveredEnvironment.fingerprint
    $runtimeContext.environment.contract = $recoveredEnvironment
    $runtimeContext.environment.verified = $true
    $runtimeContext.environment.verification_state = "verified"
    if ($stateCapture.document.PSObject.Properties.Name -contains "readiness") {
        $stateReadinessValidation = Assert-Sprint8ACurrentReadinessReference `
            -StateReadiness $stateCapture.document.readiness `
            -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -RequirePassed
        $runtimeContext.readiness_immutable_reference = $stateReadinessValidation.immutable
    }
    if (-not $recoveredTerminalAttempt) {
        # Persist recovery authentication before re-entering the lane loop. This keeps a second
        # process loss recoverable whether it occurs immediately before or immediately after the
        # lifecycle-prerequisite lane publishes its terminal receipt.
        $attemptReceipt = Set-RehearsalRecoveredAttemptIdentity `
            -AttemptDocument $attemptReceipt `
            -Source $recoveredSource `
            -EnvironmentFingerprint ([string]$recoveredEnvironment.fingerprint) `
            -ReadinessSha256 $validatedReadinessSha
        Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
    } else {
        # Terminal-tail recovery must reproduce the identity that the terminal attempt actually
        # retained. Current authentication makes recovery safe; it does not retroactively replace
        # a pre-authentication failure's explicit placeholder identity.
        $source = $attemptReceipt.mutable_source_identity
        $runtimeContext.source_verification_state = [string]$attemptReceipt.source_identity_verification_state
        $runtimeContext.environment.fingerprint = [string]$attemptReceipt.environment_fingerprint
        $runtimeContext.environment.verified =
            [string]$attemptReceipt.environment_identity.verification_state -ceq "verified"
        $runtimeContext.environment.verification_state =
            [string]$attemptReceipt.environment_identity.verification_state
    }
} else {
    $startedAt = [DateTimeOffset]::UtcNow
    $attemptReceipt = [ordered]@{
        schema_version = 3
        sprint = "sprint-8a"
        phase = "candidate-rehearsal"
        attempt = $Attempt
        authoritative = $false
        state = "preparing"
        assertions_started = $false
        assertions_started_at = $null
        started_at = $startedAt.ToString("o")
        ended_at = $null
        immutable_start_receipt = $immutableStartReceipt
        schedule_sha256 = $scheduleSha
        active_lane = $null
        active_lane_started_at = $null
        mutable_source_identity = $source
        source_identity_verification_state = "unverified"
        source_identity_verification_failure = $null
        environment_identity = [ordered]@{
            readiness_receipt = $relativeReadinessPath
            readiness_sha256 = $null
            verification_state = "unverified"
            verification_failure = $null
        }
        environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        prerequisite_receipts = @()
        correction_lineage = $null
        declared_checks = $declaredChecks
        checks = @()
        assertion_count = 0
        failure_count = 0
        blocked_count = 0
        deferred_count = 0
        nested_blocked_count = 0
        nested_failure_count = 0
        classification = $null
        invalidation_decision = "candidate freeze forbidden until every declared check passes"
        cleanup_restoration = [ordered]@{ required = $true; result = "pending" }
    }
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath | Out-Null
}

function Checkpoint-RehearsalAttempt {
    $attemptReceipt.checks = @($terminalChecks)
    $attemptReceipt.assertion_count = @($terminalChecks | Where-Object assertions_started -EQ $true).Count
    $attemptReceipt.failure_count = @($terminalChecks | Where-Object state -CEQ "failed").Count
    $attemptReceipt.blocked_count = @($terminalChecks | Where-Object state -CEQ "blocked").Count
    $attemptReceipt.deferred_count = @($terminalChecks | Where-Object state -CEQ "deferred").Count
    $attemptReceipt.nested_blocked_count = @(
        $terminalChecks | ForEach-Object { @($_.nested_blocked_checks) }
    ).Count
    $attemptReceipt.nested_failure_count = @(
        $terminalChecks | ForEach-Object { @($_.nested_failed_checks) }
    ).Count
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
}

$laneActions = [ordered]@{}
$collectLaneActions = $true
$scheduleDecisionsByName = @{}
foreach ($decision in @($selectedSchedule.decisions)) {
    $scheduleDecisionsByName[[string]$decision.name] = $decision
}

function Get-RehearsalLaneIdentityBinding {
    param([Parameter(Mandatory)][string]$Name)

    if ($Name -ceq "attempt-state-prerequisite" -and
        [string]$source.commit -ceq ("0" * 40) -and
        [string]$source.tree -ceq ("0" * 40) -and
        [string]$runtimeContext.environment.fingerprint -ceq ("0" * 64)) {
        return "pre_authentication_lifecycle_placeholder"
    }
    "attempt_identity"
}

function Invoke-RehearsalLane {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][scriptblock]$Action
    )

    if ($collectLaneActions) {
        if ($laneActions.Contains($Name)) { throw "Candidate Rehearsal action '$Name' was registered more than once." }
        $laneActions[$Name] = $Action
        return
    }
    if ($terminalByName.ContainsKey($Name)) { return }

    $declaration = @($declaredChecks | Where-Object name -CEQ $Name)
    if ($declaration.Count -ne 1) { throw "Rehearsal lane '$Name' was not declared exactly once." }
    $failedDependencies = @($declaration[0].depends_on | Where-Object {
        -not $terminalByName.ContainsKey([string]$_) -or [string]$terminalByName[[string]$_].state -cne "passed"
    })
    if ($failedDependencies.Count -gt 0) {
        $blocked = [pscustomobject][ordered]@{
            name = $Name; depends_on = @($declaration[0].depends_on); command = [string]$declaration[0].command
            wave = [string]$scheduleDecisionsByName[$Name].wave; scheduling_reason = [string]$scheduleDecisionsByName[$Name].reason
            started_at = $null; ended_at = [DateTimeOffset]::UtcNow.ToString("o"); duration_ms = 0; exit_status = $null
            assertions_started = $false; assertions_started_at = $null
            state = "blocked"; classification = [string]$declaration[0].classification; classification_source = "declared_dependency_category"
            dependency_reason = "blocked by failed prerequisite(s): $($failedDependencies -join ', ')"
            evidence_path = $null; evidence_sha256 = $null; produced_evidence = @(); failure_message = $null
            nested_blocked_checks = @(); nested_failed_checks = @()
        }
        $terminalChecks.Add($blocked)
        $terminalByName[$Name] = $blocked
        $laneReceiptPath = Join-Path $laneRoot "$Name.json"
        Publish-Sprint7AEvidence -Document ([ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
            authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
            identity_binding = Get-RehearsalLaneIdentityBinding -Name $Name
            result = $blocked
        }) -OutputPath $laneReceiptPath | Out-Null
        $blocked | Add-Member -NotePropertyName lane_receipt -NotePropertyValue ([pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $laneReceiptPath).Replace("\", "/")
            sha256 = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
        }) -Force
        $attemptReceipt.active_lane = $null
        $attemptReceipt.active_lane_started_at = $null
        Checkpoint-RehearsalAttempt
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
    $laneAssertionsStartedAt = $start
    $logPath = Join-Path $logRoot "$Name.log"
    $laneReceiptPath = Join-Path $laneRoot "$Name.json"
    $attemptReceipt.active_lane = $Name
    $attemptReceipt.active_lane_started_at = $start.ToString("o")
    Checkpoint-RehearsalAttempt
    [IO.File]::WriteAllText(
        $logPath,
        "[$($start.ToString('o'))] lane_started name=$Name`n",
        [Text.UTF8Encoding]::new($false)
    )
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
        authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        identity_binding = Get-RehearsalLaneIdentityBinding -Name $Name
        result = [ordered]@{
            name = $Name; depends_on = @($declaration[0].depends_on); command = [string]$declaration[0].command
            wave = [string]$scheduleDecisionsByName[$Name].wave; scheduling_reason = [string]$scheduleDecisionsByName[$Name].reason
            started_at = $start.ToString("o"); ended_at = $null; duration_ms = $null; exit_status = $null
            assertions_started = $true; assertions_started_at = $laneAssertionsStartedAt.ToString("o")
            state = "executing"; classification = $null; classification_source = $null; dependency_reason = $null
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
    $classificationResolution = $null
    if (-not $passed) {
        try {
            $structuredClassification = Get-RehearsalStructuredClassification -Name $Name
            $classificationResolution = Resolve-LaneClassification `
                -Default ([string]$declaration[0].classification) `
                -Detail $detailTail `
                -StructuredClassification $structuredClassification
        } catch {
            [IO.File]::AppendAllText(
                $logPath,
                "[$([DateTimeOffset]::UtcNow.ToString('o'))] classification_projection_failure`n$($_ | Out-String)`n",
                [Text.UTF8Encoding]::new($false)
            )
            $classificationResolution = [pscustomobject][ordered]@{
                classification = "harness"
                source = "classification_projection_failure"
            }
        }
    }
    $entry = [pscustomobject][ordered]@{
        name = $Name; depends_on = @($declaration[0].depends_on); command = [string]$declaration[0].command
        wave = [string]$scheduleDecisionsByName[$Name].wave; scheduling_reason = [string]$scheduleDecisionsByName[$Name].reason
        started_at = $start.ToString("o"); ended_at = $end.ToString("o"); duration_ms = [math]::Round(($end - $start).TotalMilliseconds)
        assertions_started = $true; assertions_started_at = $laneAssertionsStartedAt.ToString("o")
        exit_status = if ($passed) { 0 } else { 1 }; state = if ($passed) { "passed" } else { "failed" }
        classification = if ($passed) { $null } else { [string]$classificationResolution.classification }
        classification_source = if ($passed) { $null } else { [string]$classificationResolution.source }
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
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
        authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        identity_binding = Get-RehearsalLaneIdentityBinding -Name $Name
        result = $entry
    }) -OutputPath $laneReceiptPath -Overwrite | Out-Null
    $entry | Add-Member -NotePropertyName lane_receipt -NotePropertyValue ([pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $laneReceiptPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
    }) -Force
        $attemptReceipt.active_lane = $null
        $attemptReceipt.active_lane_started_at = $null
    if (-not $passed -and [string]$attemptReceipt.state -cne "harvesting") {
        $attemptReceipt.state = "harvesting"
    }
    Checkpoint-RehearsalAttempt
}

function Complete-RehearsalOrphanedLane {
    param([Parameter(Mandatory)][string]$Name)

    if ($terminalByName.ContainsKey($Name)) { return }
    $declaration = @($declaredChecks | Where-Object name -CEQ $Name)
    if ($declaration.Count -ne 1) { throw "Recovery active lane '$Name' is not declared." }
    $laneReceiptPath = Join-Path $laneRoot "$Name.json"
    $logPath = Join-Path $logRoot "$Name.log"
    $executingSha = $null
    $executingReceipt = $null
    if (Test-Path -LiteralPath $laneReceiptPath -PathType Leaf) {
        $executingSha = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
        $executingReceipt = Get-Content -LiteralPath $laneReceiptPath -Raw | ConvertFrom-Json
        if ([string]$executingReceipt.result.name -cne $Name -or
            [string]$executingReceipt.result.state -cne "executing") {
            throw "Recovery active lane '$Name' lacks its authenticated executing receipt."
        }
    }
    $retainedPath = Join-Path $laneRoot "$Name-process-loss.json"
    $retainedDocument = if ($null -ne $executingReceipt) { $executingReceipt } else { [ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-process-loss-capture"
        attempt = $Attempt; authoritative = $false; lane = $Name
        executing_receipt_present = $false
        expected_receipt_path = [IO.Path]::GetRelativePath($repoRoot, $laneReceiptPath).Replace("\", "/")
        active_lane_started_at = $attemptReceipt.active_lane_started_at
        diagnostic = "The process ended after the active-lane checkpoint and before its executing receipt was retained."
    } }
    $retainedSha = Publish-OrAuthenticateRehearsalImmutableEvidence `
        -Document $retainedDocument `
        -Path $retainedPath `
        -Label "Candidate Rehearsal process-loss lane capture '$Name'"
    $ended = [DateTimeOffset]::UtcNow
    if (-not (Test-Path -LiteralPath $logPath -PathType Leaf)) {
        [IO.File]::WriteAllText($logPath, "", [Text.UTF8Encoding]::new($false))
    }
    [IO.File]::AppendAllText(
        $logPath,
        "[$($ended.ToString('o'))] process_loss_recovery prior_executing_receipt_sha256=$(if ($null -eq $executingSha) { 'missing' } else { $executingSha })`n",
        [Text.UTF8Encoding]::new($false)
    )
    $laneStartedAt = ConvertTo-RehearsalOffsetTimestampText `
        -Value $(if ($null -ne $executingReceipt) {
            $executingReceipt.result.started_at
        } else {
            $attemptReceipt.active_lane_started_at
        }) `
        -Label "process-loss lane '$Name' start"
    $entry = [pscustomobject][ordered]@{
        name = $Name
        depends_on = @($declaration[0].depends_on)
        command = [string]$declaration[0].command
        wave = [string]$scheduleDecisionsByName[$Name].wave
        scheduling_reason = [string]$scheduleDecisionsByName[$Name].reason
        started_at = $laneStartedAt
        ended_at = $ended.ToString("o")
        duration_ms = $null
        assertions_started = $true
        assertions_started_at = $laneStartedAt
        exit_status = 1
        state = "failed"
        classification = "harness"
        classification_source = "process_loss_recovery"
        dependency_reason = $null
        evidence_path = [IO.Path]::GetRelativePath($repoRoot, $logPath).Replace("\", "/")
        evidence_sha256 = Get-Sprint8AFileSha256 -Path $logPath
        produced_evidence = @([ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $retainedPath).Replace("\", "/")
            sha256 = $retainedSha
        })
        failure_message = "The runner process was lost while this lane was executing; the immutable schedule and prior raw evidence were retained."
        nested_blocked_checks = @()
        nested_failed_checks = @()
    }
    $terminalChecks.Add($entry)
    $terminalByName[$Name] = $entry
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
        authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        identity_binding = Get-RehearsalLaneIdentityBinding -Name $Name
        result = $entry
    }) -OutputPath $laneReceiptPath -Overwrite | Out-Null
    $entry | Add-Member -NotePropertyName lane_receipt -NotePropertyValue ([pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $laneReceiptPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
    }) -Force
    $attemptReceipt.active_lane = $null
    $attemptReceipt.active_lane_started_at = $null
    $attemptReceipt.state = "harvesting"
    Checkpoint-RehearsalAttempt
}

function Add-RehearsalDeferredLane {
    param([Parameter(Mandatory)][string]$Name)

    if ($terminalByName.ContainsKey($Name)) { return }
    $declaration = @($declaredChecks | Where-Object name -CEQ $Name)[0]
    $decision = $scheduleDecisionsByName[$Name]
    $history = [pscustomobject][ordered]@{
        preceding_state = [string]$decision.preceding_state
        consecutive_deferrals = [int]$decision.consecutive_deferrals_before
        ever_executed = [bool]$decision.ever_executed_before
        prior_passing_receipt = $decision.prior_passing_receipt
        prior_source_identity = $decision.prior_source_identity
        prior_environment_fingerprint = [string]$decision.prior_environment_fingerprint
    }
    $entry = New-Sprint8ADeferredLaneResult `
        -Declaration $declaration `
        -Decision $decision `
        -History $history `
        -Attempt $Attempt `
        -TerminalByName $terminalByName
    $terminalChecks.Add($entry)
    $terminalByName[$Name] = $entry
    $laneReceiptPath = Join-Path $laneRoot "$Name.json"
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-lane"; attempt = $Attempt
        authoritative = $false; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        identity_binding = Get-RehearsalLaneIdentityBinding -Name $Name
        result = $entry
    }) -OutputPath $laneReceiptPath | Out-Null
    $entry | Add-Member -NotePropertyName lane_receipt -NotePropertyValue ([pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $laneReceiptPath).Replace("\", "/")
        sha256 = Assert-Sprint8AReceiptSidecar -Path $laneReceiptPath
    }) -Force
    $attemptReceipt.active_lane = $null
    $attemptReceipt.active_lane_started_at = $null
    $attemptReceipt.state = "harvesting"
    Checkpoint-RehearsalAttempt
}

Push-Location $repoRoot
try {
    Invoke-RehearsalLane "attempt-state-prerequisite" {
        if ($null -eq $script:validationLockHandle) {
            $script:validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath
        }
        if (-not (Test-Path -LiteralPath $statePath -PathType Leaf)) {
            throw "Candidate rehearsal requires the current validation-state index written by Readiness."
        }
        [void](Assert-Sprint8AReceiptSidecar -Path $statePath)
        $stateIndex = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        if (@("preparing", "executing", "harvesting") -ccontains [string]$stateIndex.readiness.state -or
            @("preparing", "executing", "harvesting") -ccontains [string]$stateIndex.rehearsal.state) {
            throw "Validation-state identifies an active attempt; concurrent Candidate Rehearsal is locked out."
        }
        $relativeReadinessPath = [IO.Path]::GetRelativePath($repoRoot, $readinessPath).Replace("\", "/")
        $stateReadinessSha = Assert-Sprint8AReceiptSidecar -Path $readinessPath
        $stateReadinessDocument = Get-Content -LiteralPath $readinessPath -Raw | ConvertFrom-Json
        $stateReadinessValidation = Assert-Sprint8ACurrentReadinessReference `
            -StateReadiness $stateIndex.readiness `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -RequirePassed
        if (($stateIndex.schema_version -isnot [int] -and $stateIndex.schema_version -isnot [long]) -or
            [int]$stateIndex.schema_version -ne 1 -or
            [string]$stateIndex.sprint -cne "sprint-8a" -or
            [string]$stateIndex.readiness.state -cne "passed" -or
            [int]$stateIndex.readiness.attempt -ne [int]$stateReadinessDocument.attempt -or
            [string]$stateIndex.readiness.receipt -cne $relativeReadinessPath -or
            [string]$stateIndex.readiness.sha256 -cne $stateReadinessSha -or
            [string]$stateReadinessValidation.current.full_path -cne [IO.Path]::GetFullPath($readinessPath) -or
            [string]$stateIndex.rehearsal.state -cne "ineligible" -or
            [int]$stateIndex.next_candidate_rehearsal.attempt -ne $Attempt -or
            [string]$stateIndex.next_candidate_rehearsal.schedule_sha256 -cne $scheduleSha -or
            [bool]$stateIndex.preflight_eligible) {
            throw "Validation-state does not identify the supplied current passing Readiness as the sole rehearsal prerequisite."
        }
        $attemptReceipt.prerequisite_receipts = @([ordered]@{
            path = [string]$stateReadinessValidation.immutable.path
            sha256 = [string]$stateReadinessValidation.immutable.sha256
        })
        $runtimeContext.readiness_immutable_reference = $stateReadinessValidation.immutable
        $stateLineageProperty = $stateIndex.PSObject.Properties["correction_lineage"]
        $stateCorrectionLineage = if ($null -eq $stateLineageProperty) { $null } else { $stateLineageProperty.Value }
        Assert-Sprint8AReadinessCorrectionLineagePresence `
            -StateLineage $stateCorrectionLineage `
            -ReadinessDocument $stateReadinessDocument
        if ($null -ne $stateCorrectionLineage) {
            $lineageValidation = Assert-Sprint8ACorrectionLineage `
                -Lineage $stateCorrectionLineage `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $evidenceRootPath `
                -ExpectedCurrentReadiness ([pscustomobject]@{
                    attempt = [int]$stateIndex.readiness.attempt
                    receipt = [string]$stateIndex.readiness.receipt
                    sha256 = [string]$stateIndex.readiness.sha256
                    state = "passed"
                }) `
                -RequireConsumedTip
            $tip = $lineageValidation.tip
            $consumptionRef = $tip.consumed_by_readiness.consumption_receipt
            $currentReadinessBinding = $lineageValidation.current_readiness_binding
            if ([string]$currentReadinessBinding.kind -ceq "direct_correction_consumption") {
                if ($stateReadinessDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
                    $null -eq $stateReadinessDocument.predecessor_correction_authorization -or
                    [string]$stateReadinessDocument.predecessor_correction_authorization.sha256 -cne [string]$tip.authorization.sha256 -or
                    $stateReadinessDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
                    [string]$stateReadinessDocument.correction_consumption_receipt.path -cne [string]$consumptionRef.path -or
                    [string]$stateReadinessDocument.correction_consumption_receipt.sha256 -cne [string]$consumptionRef.sha256) {
                    throw "Directly consumed passing Readiness does not bind the correction-lineage tip authorization and consumption."
                }
            } elseif ([string]$currentReadinessBinding.kind -ceq "clean_pre_rehearsal_supersession") {
                if ($null -ne $stateReadinessDocument.predecessor_correction_authorization -or
                    $null -ne $stateReadinessDocument.correction_consumption_receipt) {
                    throw "Clean Readiness supersession must not copy or consume the correction-lineage tip again."
                }
            } else {
                throw "Current passing Readiness lacks one authenticated correction-lineage binding."
            }
            $runtimeContext.correction_lineage = $stateCorrectionLineage
            $attemptReceipt.correction_lineage = $stateCorrectionLineage
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
                immutable_start_receipt = $immutableStartReceipt
                schedule_sha256 = $scheduleSha
            }
            next_candidate_rehearsal = [ordered]@{ attempt = $Attempt; schedule_sha256 = $scheduleSha }
            correction_lineage = $runtimeContext.correction_lineage
            preflight_eligible = $false
        }) -OutputPath $statePath -Overwrite | Out-Null
        "candidate rehearsal launch authorized and active-attempt lock acquired"
    }
    Invoke-RehearsalLane "validation-readiness-prerequisite" {
        $validatedReadinessSha = Assert-Sprint8AReceiptSidecar -Path $readinessPath
        $validatedReadiness = Get-Content -LiteralPath $readinessPath -Raw | ConvertFrom-Json
        if (($validatedReadiness.schema_version -isnot [int] -and $validatedReadiness.schema_version -isnot [long]) -or
            [int]$validatedReadiness.schema_version -ne 3 -or
            [string]$validatedReadiness.state -cne "passed" -or
            [string]$validatedReadiness.phase -cne "validation-readiness" -or
            [string]$validatedReadinessSha -cne [string]$startDocument.readiness_receipt.sha256 -or
            [int]$validatedReadiness.next_candidate_rehearsal.attempt -ne $Attempt -or
            [string]$validatedReadiness.next_candidate_rehearsal.schedule_sha256 -cne $scheduleSha -or
            ($validatedReadiness.next_candidate_rehearsal.schedule | ConvertTo-Json -Depth 100 -Compress) -cne
                ($selectedSchedule | ConvertTo-Json -Depth 100 -Compress)) {
            throw "Candidate rehearsal requires one passing schema-3 Readiness receipt bound to its immutable start schedule."
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
            $consumptionReference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $evidenceRootPath `
                -Path ([string]$validatedReadiness.correction_consumption_receipt.path) `
                -AllowLegacyAbsolute
            if ((Assert-Sprint8AReceiptSidecar -Path ([string]$consumptionReference.full_path)) -cne
                [string]$validatedReadiness.correction_consumption_receipt.sha256) {
                throw "Corrected Readiness correction-consumption digest is invalid."
            }
        }

        $runtimeContext.readiness = $validatedReadiness
        $runtimeContext.readiness_sha256 = $validatedReadinessSha
        $runtimeContext.environment.fingerprint = [string]$validatedEnvironment.fingerprint
        $runtimeContext.environment.contract = $validatedEnvironment
        $runtimeContext.environment.verified = $true
        $runtimeContext.environment.verification_state = "verified"
        $attemptReceipt.mutable_source_identity = $source
        $attemptReceipt.source_identity_verification_state = "verified"
        $attemptReceipt.source_identity_verification_failure = $null
        $attemptReceipt.environment_identity.readiness_sha256 = $validatedReadinessSha
        $attemptReceipt.environment_identity.verification_state = "verified"
        $attemptReceipt.environment_identity.verification_failure = $null
        $attemptReceipt.environment_fingerprint = [string]$validatedEnvironment.fingerprint
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
        Invoke-RehearsalPowerShellCheck -Action {
            & ./scripts/check-web-crate-boundaries.ps1
        } -FailureMessage "Package-boundary audit failed."
    }
    Invoke-RehearsalLane "module-sdk-boundaries" {
        Invoke-RehearsalPowerShellCheck -Action {
            & ./scripts/verify-module-sdk-boundaries.ps1
        } -FailureMessage "Module SDK boundary audit failed."
    }
    Invoke-RehearsalLane "dashboard-source-boundaries" {
        Invoke-RehearsalPowerShellCheck -Action {
            & ./scripts/verify-sprint-6e-boundaries.ps1
        } -FailureMessage "Dashboard source boundary audit failed."
    }
    Invoke-RehearsalLane "markdown-links" {
        Invoke-RehearsalPowerShellCheck -Action {
            & ./scripts/verify-markdown-links.ps1
        } -FailureMessage "Markdown-link audit failed."
    }
    Invoke-RehearsalLane "workspace-tests" {
        & cargo test --workspace --all-features --locked --offline --jobs 1; if ($LASTEXITCODE -ne 0) { throw "Full workspace tests failed." }
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
            -ComponentsContractLaneReceipt (Join-Path $laneRoot "components-contract-tests.json") `
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
        try {
            $finalEnvironment = Get-Sprint8AEnvironmentContract -RepositoryRoot $repoRoot -EvidenceRoot $EvidenceRoot -ProbeDatabases
            $comparison = Compare-Sprint8AEnvironmentContracts `
                -Expected $runtimeContext.environment.contract `
                -Actual $finalEnvironment
            $comparisonReceipt = [ordered]@{
                schema_version = 1
                sprint = "sprint-8a"
                phase = "candidate-rehearsal-final-environment"
                attempt = $Attempt
                authoritative = $false
                recomputation_error = $null
                comparison = $comparison
            }
            Publish-Sprint7AEvidence -Document $comparisonReceipt -OutputPath $finalEnvironmentEvidence | Out-Null
        } catch {
            $comparisonReceipt = [ordered]@{
                schema_version = 1
                sprint = "sprint-8a"
                phase = "candidate-rehearsal-final-environment"
                attempt = $Attempt
                authoritative = $false
                recomputation_error = $_.Exception.Message
                comparison = $null
            }
            Publish-Sprint7AEvidence -Document $comparisonReceipt -OutputPath $finalEnvironmentEvidence | Out-Null
            throw
        }
        if (-not [bool]$comparison.matched) {
            throw "Candidate rehearsal changed the canonical environment fingerprint from $($comparison.expected_fingerprint) to $($comparison.actual_fingerprint); changed sections: $(@($comparison.changed_sections) -join ', ')."
        }
        $comparison | ConvertTo-Json -Depth 30
    }
    Invoke-RehearsalLane "final-successor-health" {
        # This terminal sink runs after both diagnostic waves and every aggregate sink. Even a
        # passing containment lane cannot prove that later lanes left the canonical topology
        # intact, so every attempt performs one final source-exact materialization and no-op
        # verification before health and inventory are certified.
        $restorationRequired = $true
        $restorationFailure = $null
        $restorationEvidence = $null
        $inventoryFailure = $null
        $restorationResultPath = Join-Path $finalRestorationRoot "materialization/attempt-$Attempt/materialization-result.json"
        $reuseSourceExactImages = $terminalByName.ContainsKey("source-exact-materialization-no-op") -and
            [string]$terminalByName["source-exact-materialization-no-op"].state -ceq "passed"
        try {
            Invoke-RehearsalPowerShellCheck -Action {
                & ./scripts/materialize-sprint-8a.ps1 `
                    -Attempt $Attempt `
                    -EvidenceRoot $finalRestorationRoot `
                    -EnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) `
                    -AuthorizeDisposableReset `
                    -VerifyNoOp `
                    -SkipBuild:$reuseSourceExactImages `
                    -Confirm:$false
            } -FailureMessage "Mandatory final canonical restoration materialization failed."
        } catch {
            $restorationFailure = $_.Exception.Message
        }
        if ($null -eq $restorationFailure) {
            try {
                if (-not (Test-Path -LiteralPath $restorationResultPath -PathType Leaf)) {
                    throw "Mandatory final restoration did not retain its materialization result receipt."
                }
                $restorationEvidence = Assert-RehearsalRestorationMaterializationReceipt `
                    -Path $restorationResultPath `
                    -ExpectedSource $source `
                    -ExpectedEnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) `
                    -ExpectedAttempt $Attempt
            } catch {
                $restorationFailure = $_.Exception.Message
            }
        }

        $coreHealth = Invoke-Sprint8AHealthProbe -Target gateway_core -BaseUrl "http://127.0.0.1:8088"
        $supervisorHealth = Invoke-Sprint8AHealthProbe -Target supervisor -BaseUrl "http://127.0.0.1:8098"
        if ([bool]$coreHealth.passed -and [bool]$supervisorHealth.passed) {
            try {
                Invoke-RehearsalPowerShellCheck -Action {
                    & ./scripts/audit-sprint-8a-deployed-inventory.ps1 `
                        -BaseUrl "http://127.0.0.1:8088" `
                        -OutputPath $finalRestorationInventory
                } -FailureMessage "Mandatory final canonical inventory/navigation audit failed."
            } catch {
                $inventoryFailure = $_.Exception.Message
            }
        } else {
            $inventoryFailure = "blocked by failed exact Core or Supervisor health prerequisite"
        }

        $inventoryEvidence = if (Test-Path -LiteralPath $finalRestorationInventory -PathType Leaf) {
            [ordered]@{
                path = ConvertTo-Sprint8ACanonicalEvidencePath -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $finalRestorationInventory
                sha256 = Assert-Sprint8AReceiptSidecar -Path $finalRestorationInventory
            }
        } else { $null }
        $healthy = $null -eq $restorationFailure -and
            $null -ne $restorationEvidence -and
            [bool]$coreHealth.passed -and [bool]$supervisorHealth.passed -and
            $null -eq $inventoryFailure -and $null -ne $inventoryEvidence
        $finalHealth = [ordered]@{
            schema_version = 2
            contract = "tessara.sprint-8a.final-successor-health"
            sprint = "sprint-8a"
            phase = "candidate-rehearsal-final-successor-health"
            attempt = $Attempt
            authoritative = $false
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            mutable_source_identity = $source
            environment_fingerprint = [string]$runtimeContext.environment.fingerprint
            restoration = [ordered]@{
                required = $restorationRequired
                performed = $true
                materialization = $restorationEvidence
                failure = $restorationFailure
            }
            health = [ordered]@{
                core = $coreHealth
                supervisor = $supervisorHealth
            }
            inventory_navigation = [ordered]@{
                evidence = $inventoryEvidence
                failure = $inventoryFailure
            }
            cleanup_restoration = [ordered]@{
                required = $true
                result = if ($healthy) { "canonical_successor_healthy" } else { "not_proven" }
            }
            passed = $healthy
        }
        Publish-Sprint7AEvidence -Document $finalHealth -OutputPath $finalHealthEvidence | Out-Null
        if (-not $healthy) {
            throw "Mandatory final canonical restoration was not proven: restoration=$restorationFailure; core=$($coreHealth.passed); supervisor=$($supervisorHealth.passed); inventory=$inventoryFailure"
        }
        $finalHealth | ConvertTo-Json -Depth 30
    }

    if ($laneActions.Count -ne $declaredChecks.Count -or
        (($laneActions.Keys | Sort-Object) -join "`n") -cne
            ((@($declaredChecks.name) | Sort-Object) -join "`n")) {
        throw "Candidate Rehearsal runner does not implement every declared lane exactly once."
    }
    $collectLaneActions = $false
    if (-not $recoveredTerminalAttempt) {
        if ($recoveredPublishedTerminal) {
            Checkpoint-RehearsalAttempt
        }
        if ($null -ne $orphanedLaneName) {
            Complete-RehearsalOrphanedLane -Name $orphanedLaneName
        }
        $executionStatePublished = $false
        foreach ($name in @($selectedSchedule.wave_a)) {
            Invoke-RehearsalLane -Name ([string]$name) -Action ([scriptblock]$laneActions[[string]$name])
            if (-not $executionStatePublished -and
                $terminalByName.ContainsKey("attempt-state-prerequisite") -and
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
                        immutable_start_receipt = $immutableStartReceipt
                        schedule_sha256 = $scheduleSha
                    }
                    next_candidate_rehearsal = [ordered]@{ attempt = $Attempt; schedule_sha256 = $scheduleSha }
                    correction_lineage = $runtimeContext.correction_lineage
                    preflight_eligible = $false
                }) -OutputPath $statePath -Overwrite | Out-Null
                $executionStatePublished = $true
            }
        }
        $waveBDisposition = Get-Sprint8ARehearsalWaveBDisposition `
            -Schedule $selectedSchedule `
            -TerminalByName $terminalByName
        if ($waveBDisposition -ceq "defer") {
            foreach ($name in @($selectedSchedule.wave_b)) {
                Add-RehearsalDeferredLane -Name ([string]$name)
            }
        } else {
            foreach ($name in @($selectedSchedule.wave_b)) {
                Invoke-RehearsalLane -Name ([string]$name) -Action ([scriptblock]$laneActions[[string]$name])
            }
        }
        foreach ($name in @($selectedSchedule.aggregate_sinks)) {
            Invoke-RehearsalLane -Name ([string]$name) -Action ([scriptblock]$laneActions[[string]$name])
        }
        foreach ($name in @($selectedSchedule.cleanup_sinks)) {
            Invoke-RehearsalLane -Name ([string]$name) -Action ([scriptblock]$laneActions[[string]$name])
        }
        foreach ($name in @($selectedSchedule.finalizers)) {
            Invoke-RehearsalLane -Name ([string]$name) -Action ([scriptblock]$laneActions[[string]$name])
        }
    }
} finally {
    Pop-Location
}

$endedAt = if ($recoveredTerminalAttempt) {
    [DateTimeOffset]::Parse([string]$attemptReceipt.ended_at)
} else {
    [DateTimeOffset]::UtcNow
}
$failedChecks = @($terminalChecks | Where-Object state -CEQ "failed")
$blockedChecks = @($terminalChecks | Where-Object state -CEQ "blocked")
$deferredChecks = @($terminalChecks | Where-Object state -CEQ "deferred")
$nestedBlockedChecks = @($terminalChecks | ForEach-Object { @($_.nested_blocked_checks) })
$nestedFailedChecks = @($terminalChecks | ForEach-Object { @($_.nested_failed_checks) })
$passed = $terminalChecks.Count -eq $declaredChecks.Count -and
    $failedChecks.Count -eq 0 -and
    $blockedChecks.Count -eq 0 -and
    $deferredChecks.Count -eq 0 -and
    $nestedBlockedChecks.Count -eq 0 -and
    $nestedFailedChecks.Count -eq 0
if ($recoveredTerminalAttempt) {
    if (([string]$attemptReceipt.state -ceq "passed") -ne $passed) {
        throw "Candidate Rehearsal terminal-tail recovery state no longer matches its authenticated terminal lane accounting."
    }
    $attemptSha = $attemptShaBeforeRecovery
} else {
    $attemptReceipt.state = if ($passed) { "passed" } else { "failed" }
    $attemptReceipt.ended_at = $endedAt.ToString("o")
    $attemptReceipt.assertion_count = @($terminalChecks | Where-Object assertions_started -EQ $true).Count
    $attemptReceipt.failure_count = $failedChecks.Count
    $attemptReceipt.blocked_count = $blockedChecks.Count
    $attemptReceipt.deferred_count = $deferredChecks.Count
    $attemptReceipt.nested_blocked_count = $nestedBlockedChecks.Count
    $attemptReceipt.nested_failure_count = $nestedFailedChecks.Count
    $failureClassifications = @(Get-Sprint8AResultClassifications -Results (@($failedChecks) + @($nestedFailedChecks)))
    $attemptReceipt.classification = if ($failureClassifications.Count -eq 1) {
        [string]$failureClassifications[0]
    } else {
        $null
    }
    $attemptReceipt.invalidation_decision = if ($passed) { "none" } else { "candidate freeze forbidden; deferred lanes are incomplete and one consolidated correction batch governs any correction" }
    $restorationChecks = @(
        "final-successor-health",
        "final-environment-identity"
    )
    $attemptReceipt.cleanup_restoration = [ordered]@{
        required = $true
        result = if (@($restorationChecks | Where-Object {
            -not $terminalByName.ContainsKey($_) -or [string]$terminalByName[$_].state -cne "passed"
        }).Count -eq 0) { "canonical_successor_healthy" } else { "not_proven" }
    }
    $attemptReceipt.checks = @($terminalChecks)
    $attemptReceipt.active_lane = $null
    $attemptReceipt.active_lane_started_at = $null
    [void](Assert-Sprint8ARehearsalTerminalAccounting `
        -DeclaredChecks $declaredChecks `
        -TerminalChecks @($terminalChecks) `
        -Attempt $Attempt `
        -AttemptState ([string]$attemptReceipt.state))
    Assert-Sprint8ADeclaredEvidencePaths `
        -Checks @($attemptReceipt.declared_checks) `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -Label "Candidate Rehearsal $Attempt terminal declarations" | Out-Null
    Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath -Overwrite | Out-Null
    $attemptSha = Assert-Sprint8AReceiptSidecar -Path $attemptPath
}
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
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-harvest"; attempt = $Attempt
        authoritative = $false; state = "harvest_complete"; completed_at = $endedAt.ToString("o")
        receipt_path = $harvestRelative; mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        attempt_receipt = [ordered]@{ path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/"); sha256 = $attemptSha }
        immutable_start_receipt = $immutableStartReceipt
        schedule_sha256 = $scheduleSha
        checks = $terminalChecks
        failed_count = $failedChecks.Count
        blocked_count = $blockedChecks.Count
        deferred_count = $deferredChecks.Count
        nested_blocked_count = $nestedBlockedChecks.Count
        nested_failed_count = $nestedFailedChecks.Count
        passed_count = @($terminalChecks | Where-Object state -CEQ "passed").Count
    }
    if ($recoveredTerminalAttempt -and (Test-Path -LiteralPath $harvestPath -PathType Leaf)) {
        # A terminal attempt may already have completed its append-only harvest
        # before the controlling process was lost. Authenticate and consume that
        # retained receipt; do not reconstruct timestamps or rewrite history.
        $harvestSha = Assert-Sprint8AReceiptSidecar -Path $harvestPath
        $harvest = Get-Content -LiteralPath $harvestPath -Raw | ConvertFrom-Json
    } else {
        $harvestSha = Publish-OrAuthenticateRehearsalImmutableEvidence `
            -Document $harvest -Path $harvestPath -Label "Candidate Rehearsal harvest"
    }
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
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal-defect-batch"; attempt = $Attempt
        authoritative = $false; batch = 1; state = "open"; generated_at = $endedAt.ToString("o")
        mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        harvest_receipt = [ordered]@{ path = $harvestRelative; sha256 = $harvestSha }
        defect_count = $defects.Count; defects = $defects
        deferred_count = $deferredChecks.Count; deferred_checks = @($deferredChecks)
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
    if ($recoveredTerminalAttempt -and (Test-Path -LiteralPath $batchPath -PathType Leaf)) {
        $batchSha = Assert-Sprint8AReceiptSidecar -Path $batchPath
        $batch = Get-Content -LiteralPath $batchPath -Raw | ConvertFrom-Json
    } else {
        $batchSha = Publish-OrAuthenticateRehearsalImmutableEvidence `
            -Document $batch -Path $batchPath -Label "Candidate Rehearsal consolidated defect batch"
    }
    if (-not [bool]$runtimeContext.launch_authorized) {
        if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
        throw "Sprint 8A Candidate Rehearsal launch was rejected by the active-attempt/state prerequisite. Safe independent evidence and one consolidated batch were retained, but no correction authorization was issued and validation-state was not overwritten."
    }
    & (Join-Path $PSScriptRoot "test-sprint-validation-harvest.ps1") `
        -AttemptPath $attemptPath `
        -HarvestPath $harvestPath `
        -DefectBatchPath $batchPath `
        -EvidenceRoot $evidenceRootPath `
        -HarvestOnly
    if (-not $?) { throw "Candidate rehearsal harvest authentication did not complete." }

    $attemptRelative = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $attemptPath
    $batchRelative = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $batchPath
    $failedStateDocument = [ordered]@{
        schema_version = 1; sprint = "sprint-8a"; updated_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
        readiness = $readinessStateRecord
        rehearsal = [ordered]@{
            attempt = $Attempt; state = "failed"
            receipt = $attemptRelative; sha256 = $attemptSha
            immutable_start_receipt = $immutableStartReceipt; schedule_sha256 = $scheduleSha
            failed_count = $failedChecks.Count; blocked_count = $blockedChecks.Count; deferred_count = $deferredChecks.Count
            harvest = $harvestRelative; defect_batch = $batchRelative
            harvest_guard = "passed"
        }
        next_candidate_rehearsal = $null
        correction_lineage = $runtimeContext.correction_lineage
        correction_authorization = $null
        preflight_eligible = $false
    }
    if ([string]$attemptReceipt.cleanup_restoration.result -cne "canonical_successor_healthy") {
        if ((Test-Path -LiteralPath $correctionAuthorizationPath -PathType Leaf) -or
            (Test-Path -LiteralPath "$correctionAuthorizationPath.sha256" -PathType Leaf)) {
            throw "Candidate Rehearsal recovery found correction authorization despite unproven canonical cleanup/restoration."
        }
        $failedStateDocument.correction_authorization = [ordered]@{
            state = "withheld"
            reason = "correction_authorization_withheld_cleanup_not_proven"
        }
        Publish-Sprint7AEvidence -Document $failedStateDocument -OutputPath $statePath -Overwrite | Out-Null
        if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
        throw "Sprint 8A Candidate Rehearsal terminalized and its complete harvest was authenticated, but correction authorization is withheld because mandatory canonical cleanup/restoration was not proven."
    }

    Repair-Sprint7AEvidencePublication -Path $correctionAuthorizationPath
    $authorizationExists = Test-Path -LiteralPath $correctionAuthorizationPath -PathType Leaf
    $authorizationSidecarExists = Test-Path -LiteralPath "$correctionAuthorizationPath.sha256" -PathType Leaf
    if ($authorizationExists -xor $authorizationSidecarExists) {
        throw "Candidate Rehearsal correction-authorization tail has an incomplete immutable receipt/sidecar pair; recovery will not rewrite it."
    }
    if ($authorizationExists) {
        $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $correctionAuthorizationPath
        $authorizationDocument = Get-Content -LiteralPath $correctionAuthorizationPath -Raw | ConvertFrom-Json
        Assert-RehearsalCorrectionAuthorizationTail `
            -Authorization $authorizationDocument `
            -Source $source `
            -EnvironmentFingerprint ([string]$runtimeContext.environment.fingerprint) `
            -AttemptPath $attemptPath `
            -AttemptSha256 $attemptSha `
            -HarvestPath $harvestPath `
            -HarvestSha256 $harvestSha `
            -BatchPath $batchPath `
            -BatchSha256 $batchSha `
            -AttemptDocument $attemptReceipt
    } else {
        & (Join-Path $PSScriptRoot "test-sprint-validation-harvest.ps1") `
            -AttemptPath $attemptPath `
            -HarvestPath $harvestPath `
            -DefectBatchPath $batchPath `
            -CorrectionAuthorizationPath $correctionAuthorizationPath `
            -EvidenceRoot $evidenceRootPath
        if (-not $?) { throw "Candidate rehearsal harvesting could not authorize the consolidated correction batch." }
        $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $correctionAuthorizationPath
    }
    $correctionLineage = Add-Sprint8ACorrectionLineageLink `
        -Lineage $runtimeContext.correction_lineage `
        -Predecessor ([ordered]@{
            phase = "candidate-rehearsal"
            attempt = $Attempt
            receipt = [ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                sha256 = $attemptSha
            }
            harvest = [ordered]@{ path = $harvestRelative; sha256 = $harvestSha }
            defect_batch = [ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $batchPath).Replace("\", "/")
                sha256 = $batchSha
            }
        }) `
        -Authorization ([ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $correctionAuthorizationPath).Replace("\", "/")
            sha256 = $authorizationSha
        }) `
        -ConsumedByReadiness $null
    [void](Assert-Sprint8ACorrectionLineage `
        -Lineage $correctionLineage `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath)
    $authorizationRelative = ConvertTo-Sprint8ACanonicalEvidencePath `
        -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootPath -Path $correctionAuthorizationPath
    $failedStateDocument.updated_at = [DateTimeOffset]::UtcNow.ToString("o")
    $failedStateDocument.correction_lineage = $correctionLineage
    $failedStateDocument.correction_authorization = [ordered]@{
        state = "authorized"
        path = $authorizationRelative
        sha256 = $authorizationSha
    }
    Publish-Sprint7AEvidence -Document $failedStateDocument -OutputPath $statePath -Overwrite | Out-Null
    if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
    throw "Sprint 8A Candidate Rehearsal failed $($failedChecks.Count) checks, blocked $($blockedChecks.Count) lanes, deferred $($deferredChecks.Count) Wave B lanes, and retained $($nestedBlockedChecks.Count) blocked UAT scenarios. One consolidated defect batch is retained at $batchPath."
}

$result = [ordered]@{
    schema_version = 3; sprint = "sprint-8a"; phase = "candidate-rehearsal"; attempt = $Attempt
    authoritative = $false; state = "passed"; started_at = $startedAt.ToString("o"); ended_at = $endedAt.ToString("o")
    mutable_source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
    prerequisite_receipts = @([ordered]@{
        path = [string]$runtimeContext.readiness_immutable_reference.path
        sha256 = [string]$runtimeContext.readiness_immutable_reference.sha256
    })
    immutable_start_receipt = $immutableStartReceipt
    schedule_sha256 = $scheduleSha
    attempt_receipt = [ordered]@{ path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/"); sha256 = $attemptSha }
    correction_lineage = $runtimeContext.correction_lineage
    checks = $terminalChecks; assertion_count = @($terminalChecks | Where-Object assertions_started -EQ $true).Count; failure_count = 0; blocked_count = 0; deferred_count = 0; nested_blocked_count = 0; nested_failure_count = 0
    classification = $null; invalidation_decision = "none"; cleanup_restoration = $attemptReceipt.cleanup_restoration
}
Repair-Sprint7AEvidencePublication -Path $resultPath
$resultExists = Test-Path -LiteralPath $resultPath -PathType Leaf
$resultSidecarExists = Test-Path -LiteralPath "$resultPath.sha256" -PathType Leaf
if ($resultExists -xor $resultSidecarExists) {
    throw "Candidate Rehearsal result alias has an incomplete receipt/sidecar pair; recovery will not rewrite it."
}
if ($resultExists) {
    $retainedResultSha = Assert-Sprint8AReceiptSidecar -Path $resultPath
    $retainedResult = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
    if ([int]$retainedResult.attempt -eq $Attempt) {
        if (($retainedResult | ConvertTo-Json -Depth 100 -Compress) -cne
            ($result | ConvertTo-Json -Depth 100 -Compress)) {
            throw "Candidate Rehearsal recovery rejected a changed current-attempt passing result; recovery will not rewrite it."
        }
        $resultSha = $retainedResultSha
    } else {
        Publish-Sprint7AEvidence -Document $result -OutputPath $resultPath -Overwrite | Out-Null
        $resultSha = Assert-Sprint8AReceiptSidecar -Path $resultPath
    }
} else {
    Publish-Sprint7AEvidence -Document $result -OutputPath $resultPath | Out-Null
    $resultSha = Assert-Sprint8AReceiptSidecar -Path $resultPath
}
Publish-Sprint7AEvidence -Document ([ordered]@{
    schema_version = 1; sprint = "sprint-8a"; updated_at = [DateTimeOffset]::UtcNow.ToString("o")
    source_identity = $source; environment_fingerprint = [string]$runtimeContext.environment.fingerprint
    readiness = $readinessStateRecord
    rehearsal = [ordered]@{
        attempt = $Attempt; state = "passed"
        receipt = [IO.Path]::GetRelativePath($repoRoot, $resultPath).Replace("\", "/"); sha256 = $resultSha
        immutable_start_receipt = $immutableStartReceipt; schedule_sha256 = $scheduleSha
        failed_count = 0; blocked_count = 0; deferred_count = 0
    }
    next_candidate_rehearsal = $null
    correction_lineage = $runtimeContext.correction_lineage
    preflight_eligible = $true
}) -OutputPath $statePath -Overwrite | Out-Null
if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
Write-Host "Sprint 8A Candidate Rehearsal passed cleanly for source $($source.commit) and environment $($runtimeContext.environment.fingerprint)."

[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [switch]$FinalizeFailedAttempt,
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
$harvestPath = Join-Path $evidenceRootPath "attempts/readiness-$Attempt-harvest.json"
$batchPath = Join-Path $evidenceRootPath "attempts/readiness-$Attempt-defect-batch.json"
$correctionAuthorizationPath = Join-Path $evidenceRootPath "attempts/readiness-$Attempt-correction-authorization.json"
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

function Test-Sprint8AAlternatingCorrectionEpochs {
    $r38Receipt = [ordered]@{ path = "attempts/readiness-38.json"; sha256 = "a" * 64; state = "failed" }
    $r39Receipt = [ordered]@{ path = "attempts/readiness-39.json"; sha256 = "b" * 64; state = "passed" }
    $r30Link = [ordered]@{
        ordinal = 1
        predecessor = [ordered]@{
            phase = "candidate-rehearsal"; attempt = 30
            receipt = [ordered]@{ path = "attempts/candidate-rehearsal-30-attempt.json"; sha256 = "1" * 64 }
        }
        authorization = [ordered]@{ path = "attempts/candidate-rehearsal-30-correction-authorization.json"; sha256 = "4" * 64 }
        consumed_by_readiness = [ordered]@{
            attempt = 38
            receipt = $r38Receipt
            consumption_receipt = [ordered]@{ path = "attempts/candidate-rehearsal-30-correction-authorization.json.consumption.json"; sha256 = "5" * 64 }
        }
    }
    $r38Link = [ordered]@{
        ordinal = 2
        predecessor = [ordered]@{ phase = "validation-readiness"; attempt = 38; receipt = $r38Receipt }
        authorization = [ordered]@{ path = "attempts/readiness-38-correction-authorization.json"; sha256 = "6" * 64 }
        consumed_by_readiness = [ordered]@{ attempt = 39; receipt = $r39Receipt }
    }
    $priorLineage = [ordered]@{ schema_version = 1; links = @($r30Link, $r38Link) }
    $r31Link = [ordered]@{
        ordinal = 3
        predecessor = [ordered]@{
            phase = "candidate-rehearsal"; attempt = 31
            receipt = [ordered]@{ path = "attempts/candidate-rehearsal-31-attempt.json"; sha256 = "3" * 64 }
        }
        authorization = [ordered]@{ path = "attempts/candidate-rehearsal-31-correction-authorization.json"; sha256 = "7" * 64 }
        consumed_by_readiness = $null
    }
    $documents = @(
        [pscustomobject]@{ prerequisite_receipts = @([pscustomobject]@{ path = "attempts/readiness-37.json"; sha256 = "0" * 64 }) },
        [pscustomobject]@{
            predecessor_correction_authorization = [pscustomobject]$r30Link.authorization
            correction_consumption_receipt = [pscustomobject]$r30Link.consumed_by_readiness.consumption_receipt
        },
        [pscustomobject]@{
            prerequisite_receipts = @([pscustomobject]$r39Receipt)
            correction_lineage = [pscustomobject]$priorLineage
        }
    )
    Assert-Sprint8ACorrectionLineageTopology `
        -Links @($r30Link, $r38Link, $r31Link) `
        -PredecessorDocuments $documents

    $wrongPrerequisite = @($documents | ConvertTo-Json -Depth 100 | ConvertFrom-Json)
    $wrongPrerequisite[2].prerequisite_receipts[0].sha256 = "f" * 64
    $rejected = $false
    try {
        Assert-Sprint8ACorrectionLineageTopology `
            -Links @($r30Link, $r38Link, $r31Link) `
            -PredecessorDocuments $wrongPrerequisite
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Alternating correction-epoch self-test accepted a forked R31 Readiness prerequisite."
    }

    $truncatedReadiness = @($r38Link | ConvertTo-Json -Depth 100 | ConvertFrom-Json)
    $truncatedReadiness[0].ordinal = 1
    $rejected = $false
    try {
        Assert-Sprint8ACorrectionLineageTopology `
            -Links $truncatedReadiness `
            -PredecessorDocuments @($documents[1])
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Alternating correction-epoch self-test accepted a truncated failed-Readiness suffix as a root."
    }

    $truncatedCandidate = @($r31Link | ConvertTo-Json -Depth 100 | ConvertFrom-Json)
    $truncatedCandidate[0].ordinal = 1
    $rejected = $false
    try {
        Assert-Sprint8ACorrectionLineageTopology `
            -Links $truncatedCandidate `
            -PredecessorDocuments @($documents[2])
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Alternating correction-epoch self-test accepted a truncated candidate lineage prefix as a root."
    }
}

function Test-Sprint8ACorrectionIdentityContinuity {
    $source = [pscustomobject]@{
        commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"
        acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64
    }
    $documents = @(1..4 | ForEach-Object {
        [pscustomobject]@{ mutable_source_identity = $source; environment_fingerprint = "e" * 64 }
    })
    Assert-Sprint8ACorrectionIdentityContinuity -Documents $documents

    $environmentFork = @($documents | ConvertTo-Json -Depth 30 | ConvertFrom-Json)
    $environmentFork[2].environment_fingerprint = "f" * 64
    $rejected = $false
    try { Assert-Sprint8ACorrectionIdentityContinuity -Documents $environmentFork } catch { $rejected = $true }
    if (-not $rejected) { throw "Correction identity self-test accepted a defect-batch environment fork." }

    $sourceFork = @($documents | ConvertTo-Json -Depth 30 | ConvertFrom-Json)
    $sourceFork[3].mutable_source_identity.tree = "f" * 40
    $rejected = $false
    try { Assert-Sprint8ACorrectionIdentityContinuity -Documents $sourceFork } catch { $rejected = $true }
    if (-not $rejected) { throw "Correction identity self-test accepted an authorization source fork." }
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

function Get-Sprint8AReadinessCorrectionLineage {
    param([AllowNull()]$State)

    if ($null -eq $State) { return $null }
    if ($State.PSObject.Properties.Name -contains "correction_lineage" -and
        $null -ne $State.correction_lineage) {
        return $State.correction_lineage
    }
    if ($State.PSObject.Properties.Name -contains "correction_transition" -and
        $null -ne $State.correction_transition) {
        return ConvertTo-Sprint8ACorrectionLineage `
            -LegacyTransition $State.correction_transition `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath
    }
    $null
}

function Test-Sprint8ACleanReadinessRerunState {
    param([Parameter(Mandatory)]$State)

    [string]$State.readiness.state -ceq "passed" -and
        [string]$State.rehearsal.state -ceq "ineligible" -and
        $State.preflight_eligible -is [bool] -and
        $State.preflight_eligible -eq $false
}

function Get-Sprint8AReadinessAttemptReservation {
    param(
        [AllowNull()]$StateDocument,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$AttemptNumber,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    if ($null -eq $StateDocument) {
        return [pscustomobject][ordered]@{
            prior_state = $null
            clean_rerun = $false
            lineage_validation = $null
            authorization_reference = $null
            authorization_sha256 = $null
            authorization = $null
        }
    }

    $activeReadiness = $StateDocument.readiness -and
        @("preparing", "executing", "harvesting") -ccontains [string]$StateDocument.readiness.state
    $activeRehearsal = $StateDocument.rehearsal -and
        @("preparing", "executing", "harvesting") -ccontains [string]$StateDocument.rehearsal.state
    if ($activeReadiness -or $activeRehearsal) {
        throw "Validation-state identifies an active attempt; a second readiness attempt is locked out."
    }
    if (($StateDocument.schema_version -isnot [int] -and $StateDocument.schema_version -isnot [long]) -or
        [int]$StateDocument.schema_version -ne 1 -or
        [string]$StateDocument.sprint -cne "sprint-8a") {
        throw "Validation-state is not the exact Sprint 8A state schema."
    }
    [void](Assert-Sprint8ACurrentReadinessReference `
        -StateReadiness $StateDocument.readiness `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot)

    $lineage = Get-Sprint8AReadinessCorrectionLineage -State $StateDocument
    if ($null -eq $lineage) {
        if (Test-Sprint8ACleanReadinessRerunState -State $StateDocument) {
            return [pscustomobject][ordered]@{
                prior_state = $StateDocument
                clean_rerun = $true
                lineage_validation = $null
                authorization_reference = $null
                authorization_sha256 = $null
                authorization = $null
            }
        }
        throw "A prior validation attempt exists without one pending canonical correction-lineage authorization."
    }

    $lineageValidation = Assert-Sprint8ACorrectionLineage `
        -Lineage $lineage `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot
    $tip = $lineageValidation.tip
    if ($null -ne $tip.consumed_by_readiness) {
        throw "The correction-lineage authorization was already consumed; duplicate or alternate Readiness consumption is forbidden."
    }
    $predecessorState = if ([string]$tip.predecessor.phase -ceq "candidate-rehearsal") {
        $StateDocument.rehearsal
    } else {
        $StateDocument.readiness
    }
    if ([string]$predecessorState.state -cne "failed" -or
        [int]$predecessorState.attempt -ne [int]$tip.predecessor.attempt -or
        [string]$predecessorState.receipt -cne [string]$tip.predecessor.receipt.path -or
        [string]$predecessorState.sha256 -cne [string]$tip.predecessor.receipt.sha256) {
        throw "Pending correction-lineage tip does not match the exact current failed predecessor."
    }

    $resolvedAuthorization = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$tip.authorization.path) `
        -AllowLegacyAbsolute
    $authorizationSha = Assert-Sprint8AReceiptSidecar -Path ([string]$resolvedAuthorization.full_path)
    if ($authorizationSha -cne [string]$tip.authorization.sha256) {
        throw "Pending correction-lineage authorization differs from its authenticated reference."
    }
    $authorization = Get-Content -LiteralPath ([string]$resolvedAuthorization.full_path) -Raw | ConvertFrom-Json
    $hasExactAttempt = $authorization.PSObject.Properties.Name -contains "allowed_successor_attempt"
    if ([string]$authorization.allowed_successor_phase -cne "validation-readiness" -or
        [int]$authorization.allowed_successor_count -ne 1 -or
        ($hasExactAttempt -and
            (($authorization.allowed_successor_attempt -isnot [int] -and
                    $authorization.allowed_successor_attempt -isnot [long]) -or
                [int]$authorization.allowed_successor_attempt -ne $AttemptNumber)) -or
        ([string]$tip.predecessor.phase -ceq "validation-readiness" -and -not $hasExactAttempt)) {
        $allowed = if ($hasExactAttempt) { [string]$authorization.allowed_successor_attempt } else { "the next unused attempt" }
        throw "Correction authorization permits only Readiness attempt $allowed, not attempt $AttemptNumber."
    }

    [pscustomobject][ordered]@{
        prior_state = $StateDocument
        clean_rerun = $false
        lineage_validation = $lineageValidation
        authorization_reference = $resolvedAuthorization
        authorization_sha256 = $authorizationSha
        authorization = $authorization
    }
}

function Open-Sprint8AReadinessAttemptReservation {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$AttemptNumber,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$StatePath,
        [Parameter(Mandatory)][string]$LockPath,
        [Parameter(Mandatory)][string]$AttemptReceiptPath,
        [Parameter(Mandatory)][string]$StartReceiptPath,
        [Parameter(Mandatory)][string]$AttemptLogRoot
    )

    [IO.Directory]::CreateDirectory($EvidenceRoot) | Out-Null
    $lockHandle = Open-Sprint8AValidationAttemptLock -Path $LockPath
    try {
        $priorState = if (Test-Path -LiteralPath $StatePath -PathType Leaf) {
            [void](Assert-Sprint8AReceiptSidecar -Path $StatePath)
            Get-Content -LiteralPath $StatePath -Raw | ConvertFrom-Json
        } else {
            $null
        }
        $reservation = Get-Sprint8AReadinessAttemptReservation `
            -StateDocument $priorState `
            -AttemptNumber $AttemptNumber `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot
        $namespaceCollisions = @(@(
            $AttemptReceiptPath,
            "$AttemptReceiptPath.sha256",
            $StartReceiptPath,
            "$StartReceiptPath.sha256",
            $AttemptLogRoot
        ) | Where-Object { Test-Path -LiteralPath $_ })
        if ($namespaceCollisions.Count -gt 0) {
            throw "Readiness attempt $AttemptNumber already owns canonical namespace entries: $($namespaceCollisions -join ', ')."
        }
        [IO.Directory]::CreateDirectory((Split-Path -Parent $AttemptReceiptPath)) | Out-Null
        [IO.Directory]::CreateDirectory($AttemptLogRoot) | Out-Null
        return [pscustomobject][ordered]@{
            lock_handle = $lockHandle
            prior_state = $priorState
            reservation = $reservation
        }
    } catch {
        $lockHandle.Dispose()
        throw
    }
}

function Publish-Sprint8AReadinessCreateOnceOrVerify {
    param([Parameter(Mandatory)]$Document, [Parameter(Mandatory)][string]$Path)

    if (Test-Path -LiteralPath $Path -PathType Leaf) {
        $sha = Assert-Sprint8AReceiptSidecar -Path $Path
        $actual = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
        if (($actual | ConvertTo-Json -Depth 100 -Compress) -cne
            ($Document | ConvertTo-Json -Depth 100 -Compress)) {
            throw "Append-only Readiness artifact already exists with different content: $Path"
        }
        return $sha
    }
    if (Test-Path -LiteralPath "$Path.sha256") {
        throw "Append-only Readiness artifact has an orphan sidecar: $Path"
    }
    Publish-Sprint7AEvidence -Document $Document -OutputPath $Path | Out-Null
    Assert-Sprint8AReceiptSidecar -Path $Path
}

function Assert-Sprint8ANoReadinessSuccessorCollision {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$AttemptNumber,
        [Parameter(Mandatory)][string]$EvidenceRootPath
    )

    $collisions = @(@(
        (Join-Path $EvidenceRootPath "attempts/readiness-$AttemptNumber.json"),
        (Join-Path $EvidenceRootPath "attempts/readiness-$AttemptNumber.json.sha256"),
        (Join-Path $EvidenceRootPath "attempts/readiness-$AttemptNumber-start.json"),
        (Join-Path $EvidenceRootPath "attempts/readiness-$AttemptNumber-start.json.sha256"),
        (Join-Path $EvidenceRootPath "readiness-$AttemptNumber")
    ) | Where-Object { Test-Path -LiteralPath $_ })
    if ($collisions.Count -gt 0) {
        throw "Failed-Readiness correction cannot authorize occupied successor attempt $AttemptNumber`: $($collisions -join ', ')."
    }
}

function Complete-Sprint8AFailedReadinessHarvest {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$AttemptNumber,
        [Parameter(Mandatory)]$AttemptDocument,
        [Parameter(Mandatory)][string]$AttemptSha,
        [Parameter(Mandatory)]$StateDocument
    )

    $relativeAttemptPath = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
    if ([string]$AttemptDocument.phase -cne "validation-readiness" -or
        [string]$AttemptDocument.state -cne "failed" -or
        [int]$AttemptDocument.attempt -ne $AttemptNumber -or
        [string]$StateDocument.readiness.state -cne "failed" -or
        [int]$StateDocument.readiness.attempt -ne $AttemptNumber -or
        [string]$StateDocument.readiness.receipt -cne $relativeAttemptPath -or
        [string]$StateDocument.readiness.sha256 -cne $AttemptSha) {
        throw "Failed-Readiness finalization requires the exact current terminal failed attempt."
    }
    [void](Assert-Sprint8ACurrentReadinessReference `
        -StateReadiness $StateDocument.readiness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath)
    $declared = @($AttemptDocument.declared_checks)
    $terminal = @($AttemptDocument.checks)
    if ($declared.Count -eq 0 -or $terminal.Count -ne $declared.Count -or
        @($terminal | Where-Object { @("passed", "failed", "blocked") -cnotcontains [string]$_.state }).Count -ne 0) {
        throw "Failed-Readiness finalization is forbidden before every declared check is terminal."
    }
    $failed = @($terminal | Where-Object state -CEQ "failed")
    $blocked = @($terminal | Where-Object state -CEQ "blocked")
    if ($failed.Count -eq 0) { throw "A failed Readiness without a failed check cannot authorize correction." }
    $terminalEndedAt = ([DateTimeOffset]$AttemptDocument.ended_at).ToUniversalTime().ToString("o")

    $lineage = Get-Sprint8AReadinessCorrectionLineage -State $StateDocument
    if ($null -ne $lineage) {
        $lineageValidation = Assert-Sprint8ACorrectionLineage `
            -Lineage $lineage `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $evidenceRootPath `
            -ExpectedCurrentReadiness ([pscustomobject]@{
                attempt = $AttemptNumber; receipt = $relativeAttemptPath; sha256 = $AttemptSha; state = "failed"
            }) `
            -RequireConsumedTip
        $lineage = [ordered]@{ schema_version = 1; links = @($lineageValidation.links) }
    }
    if (-not (Test-Path -LiteralPath $correctionAuthorizationPath -PathType Leaf)) {
        Assert-Sprint8ANoReadinessSuccessorCollision `
            -AttemptNumber ($AttemptNumber + 1) `
            -EvidenceRootPath $evidenceRootPath
    }

    $harvestRelative = [IO.Path]::GetRelativePath($repoRoot, $harvestPath).Replace("\", "/")
    $harvest = [ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "validation-readiness-harvest"
        attempt = $AttemptNumber
        authoritative = $false
        state = "harvest_complete"
        completed_at = $terminalEndedAt
        receipt_path = $harvestRelative
        mutable_source_identity = $AttemptDocument.mutable_source_identity
        environment_fingerprint = [string]$AttemptDocument.environment_fingerprint
        attempt_receipt = [ordered]@{ path = $relativeAttemptPath; sha256 = $AttemptSha }
        checks = $terminal
        failed_count = $failed.Count
        blocked_count = $blocked.Count
        passed_count = @($terminal | Where-Object state -CEQ "passed").Count
    }
    $harvestSha = Publish-Sprint8AReadinessCreateOnceOrVerify -Document $harvest -Path $harvestPath
    $defects = @($failed | ForEach-Object {
        $retainedFailureMessage = if ($_.PSObject.Properties.Name -contains "failure_message") {
            [string]$_.failure_message
        } else { $null }
        $summary = if ([string]::IsNullOrWhiteSpace($retainedFailureMessage)) {
            "Validation Readiness check '$([string]$_.name)' failed; inspect its authenticated raw evidence."
        } else { $retainedFailureMessage }
        [ordered]@{
            id = "8A-VR$AttemptNumber-$('{0:d2}' -f ([array]::IndexOf($failed, $_) + 1))"
            classification = [string]$_.classification
            summary = $summary
            check_names = @([string]$_.name)
            raw_evidence = @([ordered]@{
                path = [string]$_.evidence_path
                sha256 = [string]$_.evidence_sha256
            }) + @($_.produced_evidence | Where-Object { $null -ne $_ })
            state = "open"
        }
    })
    $batch = [ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "validation-readiness-defect-batch"
        attempt = $AttemptNumber
        authoritative = $false
        batch = 1
        state = "open"
        generated_at = $terminalEndedAt
        mutable_source_identity = $AttemptDocument.mutable_source_identity
        environment_fingerprint = [string]$AttemptDocument.environment_fingerprint
        harvest_receipt = [ordered]@{ path = $harvestRelative; sha256 = $harvestSha }
        defect_count = $defects.Count
        defects = $defects
        blocked_checks = @($blocked | ForEach-Object {
            [ordered]@{
                scope = "check"
                name = [string]$_.name
                dependency_reason = [string]$_.dependency_reason
            }
        })
    }
    $batchSha = Publish-Sprint8AReadinessCreateOnceOrVerify -Document $batch -Path $batchPath
    if (-not (Test-Path -LiteralPath $correctionAuthorizationPath -PathType Leaf)) {
        & (Join-Path $PSScriptRoot "test-sprint-validation-harvest.ps1") `
            -AttemptPath $attemptPath `
            -HarvestPath $harvestPath `
            -DefectBatchPath $batchPath `
            -CorrectionAuthorizationPath $correctionAuthorizationPath `
            -EvidenceRoot $evidenceRootPath
        if (-not $?) { throw "Failed-Readiness harvest guard did not authorize its consolidated batch." }
    }
    $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $correctionAuthorizationPath
    $authorization = Get-Content -LiteralPath $correctionAuthorizationPath -Raw | ConvertFrom-Json
    if ([string]$authorization.phase -cne "validation-readiness-correction-authorization" -or
        [int]$authorization.allowed_successor_attempt -ne ($AttemptNumber + 1) -or
        [string]$authorization.predecessor_attempt_receipt.sha256 -cne $AttemptSha -or
        [string]$authorization.harvest_receipt.sha256 -cne $harvestSha -or
        [string]$authorization.defect_batch.sha256 -cne $batchSha) {
        throw "Failed-Readiness correction authorization does not bind its exact terminal batch."
    }

    $links = @()
    if ($null -ne $lineage) { $links = @($lineage.links) }
    $existing = @($links | Where-Object {
        [string]$_.predecessor.phase -ceq "validation-readiness" -and
        [int]$_.predecessor.attempt -eq $AttemptNumber
    })
    if ($existing.Count -eq 0) {
        $lineage = Add-Sprint8ACorrectionLineageLink `
            -Lineage $lineage `
            -Predecessor ([ordered]@{
                phase = "validation-readiness"
                attempt = $AttemptNumber
                receipt = [ordered]@{ path = $relativeAttemptPath; sha256 = $AttemptSha }
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
    } elseif ($existing.Count -ne 1 -or $null -ne $existing[0].consumed_by_readiness -or
        [string]$existing[0].authorization.sha256 -cne $authorizationSha) {
        throw "Failed-Readiness lineage already contains a conflicting correction link."
    } else {
        $lineage = [ordered]@{ schema_version = 1; links = @($links) }
    }
    [void](Assert-Sprint8ACorrectionLineage `
        -Lineage $lineage `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath)
    Publish-Sprint7AEvidence -Document ([ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        updated_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_identity = $AttemptDocument.mutable_source_identity
        source_identity_verification_state = [string]$AttemptDocument.source_identity_verification_state
        environment_fingerprint = [string]$AttemptDocument.environment_fingerprint
        readiness = [ordered]@{ attempt = $AttemptNumber; state = "failed"; receipt = $relativeAttemptPath; sha256 = $AttemptSha }
        rehearsal = [ordered]@{ state = "ineligible"; reason = "no rehearsal is eligible until a successor readiness result passes" }
        correction_lineage = $lineage
        preflight_eligible = $false
    }) -OutputPath $statePath -Overwrite | Out-Null
    $lineage
}

function Test-Sprint8AFailedReadinessFinalization {
    $temporaryParent = [IO.Path]::GetFullPath((Join-Path $repoRoot "tmp"))
    $temporaryRoot = [IO.Path]::GetFullPath((Join-Path $temporaryParent "sprint-8a-readiness-finalizer-$([guid]::NewGuid().ToString('N'))"))
    if (-not $temporaryRoot.StartsWith("$temporaryParent$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Readiness finalizer self-test temporary path escaped the repository tmp directory."
    }
    $saved = [ordered]@{
        evidenceRootPath = $script:evidenceRootPath
        attemptPath = $script:attemptPath
        statePath = $script:statePath
        startSnapshotPath = $script:startSnapshotPath
        harvestPath = $script:harvestPath
        batchPath = $script:batchPath
        correctionAuthorizationPath = $script:correctionAuthorizationPath
    }
    $successorReservationHandle = $null
    $failedReservationHandle = $null
    try {
        $script:evidenceRootPath = $temporaryRoot
        $script:attemptPath = Join-Path $temporaryRoot "attempts/readiness-1.json"
        $script:statePath = Join-Path $temporaryRoot "validation-state.json"
        $script:startSnapshotPath = Join-Path $temporaryRoot "attempts/readiness-1-start.json"
        $script:harvestPath = Join-Path $temporaryRoot "attempts/readiness-1-harvest.json"
        $script:batchPath = Join-Path $temporaryRoot "attempts/readiness-1-defect-batch.json"
        $script:correctionAuthorizationPath = Join-Path $temporaryRoot "attempts/readiness-1-correction-authorization.json"
        [IO.Directory]::CreateDirectory((Split-Path -Parent $script:attemptPath)) | Out-Null
        $rawPath = Join-Path $temporaryRoot "readiness-1/environment-contract.log"
        [IO.Directory]::CreateDirectory((Split-Path -Parent $rawPath)) | Out-Null
        [IO.File]::WriteAllText($rawPath, "isolated failed Readiness evidence`n", [Text.UTF8Encoding]::new($false))
        $rawSha = Get-Sprint8AFileSha256 -Path $rawPath
        [IO.File]::WriteAllText("$rawPath.sha256", "$rawSha`n", [Text.UTF8Encoding]::new($false))
        $rawRelative = [IO.Path]::GetRelativePath($repoRoot, $rawPath).Replace("\", "/")
        $attemptRelative = [IO.Path]::GetRelativePath($repoRoot, $script:attemptPath).Replace("\", "/")
        $source = [pscustomobject][ordered]@{
            commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
            acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
        }
        $terminalCheck = [pscustomobject][ordered]@{
            name = "environment-contract"; depends_on = @(); command = "isolated finalizer self-test"
            started_at = "2026-08-07T12:00:00Z"; ended_at = "2026-08-07T12:00:01Z"
            assertions_started = $true; assertions_started_at = "2026-08-07T12:00:00Z"; duration_ms = 1000
            exit_status = 1; state = "failed"; passed = $false; classification = "environment"
            dependency_reason = $null; failure_message = "isolated expected failure"
            evidence_path = $rawRelative; evidence_sha256 = $rawSha; produced_evidence = $null
        }
        $attemptDocument = [pscustomobject][ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 1
            authoritative = $false; state = "failed"; assertions_started = $true
            assertions_started_at = "2026-08-07T12:00:00Z"; started_at = "2026-08-07T12:00:00Z"
            ended_at = "2026-08-07T12:00:01Z"; mutable_source_identity = $source
            source_identity_verification_state = "unverified"
            environment_identity = [pscustomobject]@{ verification_state = "failed" }
            environment_fingerprint = "0" * 64
            declared_checks = @([pscustomobject]@{ name = "environment-contract"; depends_on = @(); classification = "environment" })
            checks = @($terminalCheck); assertion_count = 1; failure_count = 1; blocked_count = 0
        }
        $attemptSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $attemptDocument -Path $script:attemptPath
        $attemptDocument = Get-Content -LiteralPath $script:attemptPath -Raw | ConvertFrom-Json
        $stateDocument = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; updated_at = "2026-08-07T12:00:01Z"
            source_identity = $source; source_identity_verification_state = "unverified"
            environment_fingerprint = "0" * 64
            readiness = [pscustomobject]@{ attempt = 1; state = "failed"; receipt = $attemptRelative; sha256 = $attemptSha }
            rehearsal = [pscustomobject]@{ state = "ineligible" }; preflight_eligible = $false
        }
        [void](Publish-Sprint8AAppendOnlyJsonReceipt -Document $stateDocument -Path $script:statePath)
        $stateDocument = Get-Content -LiteralPath $script:statePath -Raw | ConvertFrom-Json
        $successorCollisionPath = Join-Path $temporaryRoot "attempts/readiness-2-start.json"
        [IO.File]::WriteAllText($successorCollisionPath, "occupied successor`n", [Text.UTF8Encoding]::new($false))
        $collisionRejected = $false
        try {
            [void](Complete-Sprint8AFailedReadinessHarvest `
                -AttemptNumber 1 `
                -AttemptDocument $attemptDocument `
                -AttemptSha $attemptSha `
                -StateDocument $stateDocument)
        } catch {
            if ($_.Exception.Message -like "Failed-Readiness correction cannot authorize occupied successor attempt 2:*") {
                $collisionRejected = $true
            } else { throw }
        }
        if (-not $collisionRejected -or (Test-Path -LiteralPath $script:correctionAuthorizationPath)) {
            throw "Failed-Readiness finalizer self-test authorized an occupied exact successor."
        }
        Remove-Item -LiteralPath $successorCollisionPath -Force
        $lineage = Complete-Sprint8AFailedReadinessHarvest `
            -AttemptNumber 1 `
            -AttemptDocument $attemptDocument `
            -AttemptSha $attemptSha `
            -StateDocument $stateDocument
        $authorizationSha = Assert-Sprint8AReceiptSidecar -Path $script:correctionAuthorizationPath
        $authorization = Get-Content -LiteralPath $script:correctionAuthorizationPath -Raw | ConvertFrom-Json
        if ($lineage.links -isnot [array] -or @($lineage.links).Count -ne 1 -or
            [string]$lineage.links[0].predecessor.phase -cne "validation-readiness" -or
            $null -ne $lineage.links[0].consumed_by_readiness -or
            [int]$authorization.allowed_successor_attempt -ne 2 -or
            [string]$lineage.links[0].authorization.sha256 -cne $authorizationSha -or
            (Test-Path -LiteralPath (Join-Path $temporaryRoot "attempts/readiness-2-start.json")) -or
            (Test-Path -LiteralPath (Join-Path $temporaryRoot "attempts/readiness-2.json"))) {
            throw "Failed-Readiness finalizer self-test did not retain one pending exact-successor authorization without launching Readiness 2."
        }
        $finalState = Get-Content -LiteralPath $script:statePath -Raw | ConvertFrom-Json
        [void](Assert-Sprint8ACorrectionLineage `
            -Lineage $finalState.correction_lineage `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $temporaryRoot)

        $outOfSequenceRejected = $false
        $reservationLockPath = Join-Path $temporaryRoot "validation-attempt.lock"
        $outOfSequenceAttemptPath = Join-Path $temporaryRoot "attempts/readiness-3.json"
        $outOfSequenceStartPath = Join-Path $temporaryRoot "attempts/readiness-3-start.json"
        $outOfSequenceLogRoot = Join-Path $temporaryRoot "readiness-3"
        try {
            [void](Open-Sprint8AReadinessAttemptReservation `
                -AttemptNumber 3 `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $temporaryRoot `
                -StatePath $script:statePath `
                -LockPath $reservationLockPath `
                -AttemptReceiptPath $outOfSequenceAttemptPath `
                -StartReceiptPath $outOfSequenceStartPath `
                -AttemptLogRoot $outOfSequenceLogRoot)
        } catch {
            if ($_.Exception.Message -like "Correction authorization permits only Readiness attempt 2, not attempt 3.*") {
                $outOfSequenceRejected = $true
            } else { throw }
        }
        $outOfSequenceTargets = @(
            $outOfSequenceAttemptPath,
            "$outOfSequenceAttemptPath.sha256",
            $outOfSequenceStartPath,
            "$outOfSequenceStartPath.sha256",
            $outOfSequenceLogRoot
        )
        if (-not $outOfSequenceRejected -or
            @($outOfSequenceTargets | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Readiness reservation self-test let an out-of-sequence probe occupy its canonical namespace."
        }
        $successorStartPath = Join-Path $temporaryRoot "attempts/readiness-2-start.json"
        $successorAttemptPath = Join-Path $temporaryRoot "attempts/readiness-2.json"
        $successorReservationHandle = Open-Sprint8AReadinessAttemptReservation `
            -AttemptNumber 2 `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $temporaryRoot `
            -StatePath $script:statePath `
            -LockPath $reservationLockPath `
            -AttemptReceiptPath $successorAttemptPath `
            -StartReceiptPath $successorStartPath `
            -AttemptLogRoot (Join-Path $temporaryRoot "readiness-2")

        $successorStartRelative = [IO.Path]::GetRelativePath($repoRoot, $successorStartPath).Replace("\", "/")
        $successorStart = [ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 2
            authoritative = $false; state = "preparing"; assertions_started = $false; assertions_started_at = $null
            started_at = "2026-08-07T12:01:00Z"; ended_at = $null; mutable_source_identity = $source
            source_identity_verification_state = "unverified"; environment_fingerprint = "0" * 64
            predecessor_correction_authorization = $null; correction_consumption_receipt = $null; checks = @()
        }
        $successorStartSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $successorStart -Path $successorStartPath
        $authorizationRelative = [IO.Path]::GetRelativePath($repoRoot, $script:correctionAuthorizationPath).Replace("\", "/")
        $consumptionPath = "$($script:correctionAuthorizationPath).consumption.json"
        $consumptionRelative = [IO.Path]::GetRelativePath($repoRoot, $consumptionPath).Replace("\", "/")
        $consumption = [ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "validation-readiness-correction-consumption"
            authoritative = $false; state = "consumed"; consumed_at = "2026-08-07T12:01:01Z"
            authorization = [ordered]@{ path = $authorizationRelative; sha256 = $authorizationSha }
            predecessor = $lineage.links[0].predecessor
            successor_readiness = [ordered]@{
                attempt = 2; start_receipt = $successorStartRelative; start_receipt_sha256 = $successorStartSha
            }
        }
        $consumptionSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $consumption -Path $consumptionPath
        $successorAttemptRelative = [IO.Path]::GetRelativePath($repoRoot, $successorAttemptPath).Replace("\", "/")
        $successorTerminal = [ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 2
            authoritative = $false; state = "passed"; assertions_started = $true
            assertions_started_at = "2026-08-07T12:01:01Z"; started_at = "2026-08-07T12:01:00Z"
            ended_at = "2026-08-07T12:01:02Z"; mutable_source_identity = $source
            source_identity_verification_state = "unverified"; environment_fingerprint = "0" * 64
            predecessor_correction_authorization = [ordered]@{ path = $authorizationRelative; sha256 = $authorizationSha }
            correction_consumption_receipt = [ordered]@{ path = $consumptionRelative; sha256 = $consumptionSha }
            checks = @([ordered]@{ name = "isolated"; state = "passed" })
        }
        $successorAttemptSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $successorTerminal -Path $successorAttemptPath
        $canonicalAliasPath = Join-Path $temporaryRoot "validation-readiness-result.json"
        $canonicalAliasRelative = [IO.Path]::GetRelativePath($repoRoot, $canonicalAliasPath).Replace("\", "/")
        $canonicalAliasSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $successorTerminal -Path $canonicalAliasPath
        if ($canonicalAliasSha -cne $successorAttemptSha) {
            throw "Readiness alias self-test did not publish one exact immutable counterpart."
        }
        $lineage.links[0].consumed_by_readiness = [ordered]@{
            attempt = 2
            start_receipt = [ordered]@{ path = $successorStartRelative; sha256 = $successorStartSha }
            consumption_receipt = [ordered]@{ path = $consumptionRelative; sha256 = $consumptionSha }
            receipt = [ordered]@{ path = $successorAttemptRelative; sha256 = $successorAttemptSha; state = "passed" }
        }
        [void](Assert-Sprint8ACorrectionLineage `
            -Lineage $lineage `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $temporaryRoot `
            -ExpectedCurrentReadiness ([pscustomobject]@{
                attempt = 2; state = "passed"; receipt = $canonicalAliasRelative; sha256 = $canonicalAliasSha
            }) `
            -RequireConsumedTip)
        $successorReservationHandle.lock_handle.Dispose()
        $successorReservationHandle = $null

        $aliasSubstitution = $lineage | ConvertTo-Json -Depth 100 | ConvertFrom-Json
        $aliasSubstitution.links[0].consumed_by_readiness.receipt.path = $canonicalAliasRelative
        $substitutionRejected = $false
        try {
            [void](Assert-Sprint8ACorrectionLineage `
                -Lineage $aliasSubstitution `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $temporaryRoot `
                -ExpectedCurrentReadiness ([pscustomobject]@{
                    attempt = 2; state = "passed"; receipt = $canonicalAliasRelative; sha256 = $canonicalAliasSha
                }) `
                -RequireConsumedTip)
        } catch { $substitutionRejected = $true }
        if (-not $substitutionRejected) {
            throw "Readiness lineage self-test accepted the mutable canonical alias as its terminal receipt."
        }

        # Exercise an actual alias rollover through a second correction epoch. The
        # first epoch must remain authenticated through immutable Readiness 2 after
        # the canonical alias advances to passing Readiness 3.
        $priorLineage = $lineage | ConvertTo-Json -Depth 100 | ConvertFrom-Json
        $candidateAttemptPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-1-attempt.json"
        $candidateAttemptRelative = [IO.Path]::GetRelativePath($repoRoot, $candidateAttemptPath).Replace("\", "/")
        $candidateAttempt = [ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal"; attempt = 1
            authoritative = $false; state = "failed"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
            prerequisite_receipts = @([ordered]@{ path = $successorAttemptRelative; sha256 = $successorAttemptSha })
            correction_lineage = $priorLineage
        }
        $candidateAttemptSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $candidateAttempt -Path $candidateAttemptPath
        $candidateHarvestPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-1-harvest.json"
        $candidateHarvestRelative = [IO.Path]::GetRelativePath($repoRoot, $candidateHarvestPath).Replace("\", "/")
        $candidateHarvestSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-harvest"; attempt = 1
            authoritative = $false; state = "harvest_complete"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
        }) -Path $candidateHarvestPath
        $candidateBatchPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-1-defect-batch.json"
        $candidateBatchRelative = [IO.Path]::GetRelativePath($repoRoot, $candidateBatchPath).Replace("\", "/")
        $candidateBatchSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-defect-batch"; attempt = 1
            authoritative = $false; state = "open"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
        }) -Path $candidateBatchPath
        $candidateAuthorizationPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-1-correction-authorization.json"
        $candidateAuthorizationRelative = [IO.Path]::GetRelativePath($repoRoot, $candidateAuthorizationPath).Replace("\", "/")
        $candidateAuthorization = [ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-correction-authorization"; attempt = 1
            authoritative = $false; state = "authorized"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
            predecessor_attempt_receipt = [ordered]@{ path = $candidateAttemptRelative; sha256 = $candidateAttemptSha }
            harvest_receipt = [ordered]@{ path = $candidateHarvestRelative; sha256 = $candidateHarvestSha }
            defect_batch = [ordered]@{ path = $candidateBatchRelative; sha256 = $candidateBatchSha }
            allowed_successor_count = 1; allowed_successor_phase = "validation-readiness"; allowed_successor_attempt = 3
        }
        $candidateAuthorizationSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $candidateAuthorization -Path $candidateAuthorizationPath
        $secondLineage = Add-Sprint8ACorrectionLineageLink `
            -Lineage $priorLineage `
            -Predecessor ([ordered]@{
                phase = "candidate-rehearsal"; attempt = 1
                receipt = [ordered]@{ path = $candidateAttemptRelative; sha256 = $candidateAttemptSha }
                harvest = [ordered]@{ path = $candidateHarvestRelative; sha256 = $candidateHarvestSha }
                defect_batch = [ordered]@{ path = $candidateBatchRelative; sha256 = $candidateBatchSha }
            }) `
            -Authorization ([ordered]@{ path = $candidateAuthorizationRelative; sha256 = $candidateAuthorizationSha }) `
            -ConsumedByReadiness $null

        $secondStartPath = Join-Path $temporaryRoot "attempts/readiness-3-start.json"
        $secondStartRelative = [IO.Path]::GetRelativePath($repoRoot, $secondStartPath).Replace("\", "/")
        $secondStartSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 3
            authoritative = $false; state = "preparing"; assertions_started = $false; assertions_started_at = $null
            started_at = "2026-08-07T12:02:00Z"; ended_at = $null; mutable_source_identity = $source
            source_identity_verification_state = "unverified"; environment_fingerprint = "0" * 64
            predecessor_correction_authorization = $null; correction_consumption_receipt = $null; checks = @()
        }) -Path $secondStartPath
        $secondConsumptionPath = "$candidateAuthorizationPath.consumption.json"
        $secondConsumptionRelative = [IO.Path]::GetRelativePath($repoRoot, $secondConsumptionPath).Replace("\", "/")
        $secondConsumptionSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-correction-consumption"
            authoritative = $false; state = "consumed"; consumed_at = "2026-08-07T12:02:01Z"
            authorization = [ordered]@{ path = $candidateAuthorizationRelative; sha256 = $candidateAuthorizationSha }
            predecessor = $secondLineage.links[1].predecessor
            successor_readiness = [ordered]@{
                attempt = 3; start_receipt = $secondStartRelative; start_receipt_sha256 = $secondStartSha
            }
        }) -Path $secondConsumptionPath
        $secondAttemptPath = Join-Path $temporaryRoot "attempts/readiness-3.json"
        $secondAttemptRelative = [IO.Path]::GetRelativePath($repoRoot, $secondAttemptPath).Replace("\", "/")
        $secondTerminal = [ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 3
            authoritative = $false; state = "passed"; assertions_started = $true
            assertions_started_at = "2026-08-07T12:02:01Z"; started_at = "2026-08-07T12:02:00Z"
            ended_at = "2026-08-07T12:02:02Z"; mutable_source_identity = $source
            source_identity_verification_state = "unverified"; environment_fingerprint = "0" * 64
            predecessor_correction_authorization = [ordered]@{ path = $candidateAuthorizationRelative; sha256 = $candidateAuthorizationSha }
            correction_consumption_receipt = [ordered]@{ path = $secondConsumptionRelative; sha256 = $secondConsumptionSha }
            checks = @([ordered]@{ name = "isolated-second-epoch"; state = "passed" })
        }
        $secondAttemptSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $secondTerminal -Path $secondAttemptPath
        Publish-Sprint7AEvidence -Document $secondTerminal -OutputPath $canonicalAliasPath -Overwrite | Out-Null
        $secondAliasSha = Assert-Sprint8AReceiptSidecar -Path $canonicalAliasPath
        if ($secondAliasSha -cne $secondAttemptSha -or $secondAliasSha -ceq $canonicalAliasSha) {
            throw "Readiness alias rollover self-test did not advance to the second immutable passing terminal."
        }
        $secondLineage.links[1].consumed_by_readiness = [ordered]@{
            attempt = 3
            start_receipt = [ordered]@{ path = $secondStartRelative; sha256 = $secondStartSha }
            consumption_receipt = [ordered]@{ path = $secondConsumptionRelative; sha256 = $secondConsumptionSha }
            receipt = [ordered]@{ path = $secondAttemptRelative; sha256 = $secondAttemptSha; state = "passed" }
        }
        [void](Assert-Sprint8ACorrectionLineage `
            -Lineage $secondLineage `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $temporaryRoot `
            -ExpectedCurrentReadiness ([pscustomobject]@{
                attempt = 3; state = "passed"; receipt = $canonicalAliasRelative; sha256 = $secondAliasSha
            }) `
            -RequireConsumedTip)
        if ([string]$secondLineage.links[0].consumed_by_readiness.receipt.path -cne $successorAttemptRelative -or
            [string]$secondLineage.links[0].consumed_by_readiness.receipt.sha256 -cne $successorAttemptSha) {
            throw "Readiness alias rollover self-test rewrote the first immutable terminal."
        }
        $supersededAliasRejected = $false
        try {
            [void](Assert-Sprint8ACorrectionLineage `
                -Lineage $priorLineage `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $temporaryRoot `
                -ExpectedCurrentReadiness ([pscustomobject]@{
                    attempt = 2; state = "passed"; receipt = $canonicalAliasRelative; sha256 = $canonicalAliasSha
                }) `
                -RequireConsumedTip)
        } catch { $supersededAliasRejected = $true }
        if (-not $supersededAliasRejected) {
            throw "Readiness alias rollover self-test accepted the superseded alias as the current tip."
        }

        # A later failed Readiness must be finalizable on top of the complete
        # already-consumed lineage. This exercises the live failed-current
        # receipt contract rather than only a root failed attempt.
        $thirdCandidatePath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-2-attempt.json"
        $thirdCandidateRelative = [IO.Path]::GetRelativePath($repoRoot, $thirdCandidatePath).Replace("\", "/")
        $thirdCandidateSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal"; attempt = 2
            authoritative = $false; state = "failed"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
            prerequisite_receipts = @([ordered]@{ path = $secondAttemptRelative; sha256 = $secondAttemptSha })
            correction_lineage = $secondLineage
        }) -Path $thirdCandidatePath
        $thirdHarvestPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-2-harvest.json"
        $thirdHarvestRelative = [IO.Path]::GetRelativePath($repoRoot, $thirdHarvestPath).Replace("\", "/")
        $thirdHarvestSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-harvest"; attempt = 2
            authoritative = $false; state = "harvest_complete"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
        }) -Path $thirdHarvestPath
        $thirdBatchPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-2-defect-batch.json"
        $thirdBatchRelative = [IO.Path]::GetRelativePath($repoRoot, $thirdBatchPath).Replace("\", "/")
        $thirdBatchSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-defect-batch"; attempt = 2
            authoritative = $false; state = "open"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
        }) -Path $thirdBatchPath
        $thirdAuthorizationPath = Join-Path $temporaryRoot "attempts/candidate-rehearsal-2-correction-authorization.json"
        $thirdAuthorizationRelative = [IO.Path]::GetRelativePath($repoRoot, $thirdAuthorizationPath).Replace("\", "/")
        $thirdAuthorizationSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-correction-authorization"; attempt = 2
            authoritative = $false; state = "authorized"; mutable_source_identity = $source
            environment_fingerprint = "0" * 64
            predecessor_attempt_receipt = [ordered]@{ path = $thirdCandidateRelative; sha256 = $thirdCandidateSha }
            harvest_receipt = [ordered]@{ path = $thirdHarvestRelative; sha256 = $thirdHarvestSha }
            defect_batch = [ordered]@{ path = $thirdBatchRelative; sha256 = $thirdBatchSha }
            allowed_successor_count = 1; allowed_successor_phase = "validation-readiness"; allowed_successor_attempt = 4
        }) -Path $thirdAuthorizationPath
        $failedSuccessorLineage = Add-Sprint8ACorrectionLineageLink `
            -Lineage $secondLineage `
            -Predecessor ([ordered]@{
                phase = "candidate-rehearsal"; attempt = 2
                receipt = [ordered]@{ path = $thirdCandidateRelative; sha256 = $thirdCandidateSha }
                harvest = [ordered]@{ path = $thirdHarvestRelative; sha256 = $thirdHarvestSha }
                defect_batch = [ordered]@{ path = $thirdBatchRelative; sha256 = $thirdBatchSha }
            }) `
            -Authorization ([ordered]@{ path = $thirdAuthorizationRelative; sha256 = $thirdAuthorizationSha }) `
            -ConsumedByReadiness $null
        $failedAttemptPath = Join-Path $temporaryRoot "attempts/readiness-4.json"
        $failedStartPath = Join-Path $temporaryRoot "attempts/readiness-4-start.json"
        $preFailedReadinessState = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; updated_at = "2026-08-07T12:02:03Z"
            source_identity = $source; source_identity_verification_state = "unverified"
            environment_fingerprint = "0" * 64
            readiness = [pscustomobject]@{
                attempt = 3; state = "passed"; receipt = $canonicalAliasRelative; sha256 = $secondAliasSha
            }
            rehearsal = [pscustomobject]@{
                attempt = 2; state = "failed"; receipt = $thirdCandidateRelative; sha256 = $thirdCandidateSha
            }
            correction_lineage = $failedSuccessorLineage
            preflight_eligible = $false
        }
        Publish-Sprint7AEvidence -Document $preFailedReadinessState -OutputPath $script:statePath -Overwrite | Out-Null

        $outOfSequenceFailedAttemptPath = Join-Path $temporaryRoot "attempts/readiness-5.json"
        $outOfSequenceFailedStartPath = Join-Path $temporaryRoot "attempts/readiness-5-start.json"
        $outOfSequenceFailedLogRoot = Join-Path $temporaryRoot "readiness-5"
        $outOfSequenceFailedRejected = $false
        try {
            [void](Open-Sprint8AReadinessAttemptReservation `
                -AttemptNumber 5 `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $temporaryRoot `
                -StatePath $script:statePath `
                -LockPath $reservationLockPath `
                -AttemptReceiptPath $outOfSequenceFailedAttemptPath `
                -StartReceiptPath $outOfSequenceFailedStartPath `
                -AttemptLogRoot $outOfSequenceFailedLogRoot)
        } catch {
            if ($_.Exception.Message -like "Correction authorization permits only Readiness attempt 4, not attempt 5.*") {
                $outOfSequenceFailedRejected = $true
            } else { throw }
        }
        $outOfSequenceFailedTargets = @(
            $outOfSequenceFailedAttemptPath,
            "$outOfSequenceFailedAttemptPath.sha256",
            $outOfSequenceFailedStartPath,
            "$outOfSequenceFailedStartPath.sha256",
            $outOfSequenceFailedLogRoot
        )
        if (-not $outOfSequenceFailedRejected -or
            @($outOfSequenceFailedTargets | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Failed-Readiness reservation self-test let an out-of-sequence probe occupy its canonical namespace."
        }
        $failedLogRoot = Join-Path $temporaryRoot "readiness-4"
        $failedReservationHandle = Open-Sprint8AReadinessAttemptReservation `
            -AttemptNumber 4 `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $temporaryRoot `
            -StatePath $script:statePath `
            -LockPath $reservationLockPath `
            -AttemptReceiptPath $failedAttemptPath `
            -StartReceiptPath $failedStartPath `
            -AttemptLogRoot $failedLogRoot
        if ([int]$failedReservationHandle.reservation.authorization.allowed_successor_attempt -ne 4 -or
            -not (Test-Path -LiteralPath $failedLogRoot -PathType Container) -or
            (Test-Path -LiteralPath $failedAttemptPath) -or
            (Test-Path -LiteralPath "$failedAttemptPath.sha256") -or
            (Test-Path -LiteralPath $failedStartPath) -or
            (Test-Path -LiteralPath "$failedStartPath.sha256")) {
            throw "Failed-Readiness reservation self-test did not authenticate and reserve exact Readiness successor 4 before publication."
        }

        $failedStartRelative = [IO.Path]::GetRelativePath($repoRoot, $failedStartPath).Replace("\", "/")
        $failedStartSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 4
            authoritative = $false; state = "preparing"; assertions_started = $false; assertions_started_at = $null
            started_at = "2026-08-07T12:03:00Z"; ended_at = $null; mutable_source_identity = $source
            source_identity_verification_state = "unverified"; environment_fingerprint = "0" * 64
            predecessor_correction_authorization = $null; correction_consumption_receipt = $null; checks = @()
        }) -Path $failedStartPath
        $thirdConsumptionPath = "$thirdAuthorizationPath.consumption.json"
        $thirdConsumptionRelative = [IO.Path]::GetRelativePath($repoRoot, $thirdConsumptionPath).Replace("\", "/")
        $thirdConsumptionSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document ([ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-correction-consumption"
            authoritative = $false; state = "consumed"; consumed_at = "2026-08-07T12:03:01Z"
            authorization = [ordered]@{ path = $thirdAuthorizationRelative; sha256 = $thirdAuthorizationSha }
            predecessor = $failedSuccessorLineage.links[2].predecessor
            successor_readiness = [ordered]@{
                attempt = 4; start_receipt = $failedStartRelative; start_receipt_sha256 = $failedStartSha
            }
        }) -Path $thirdConsumptionPath
        $failedAttemptRelative = [IO.Path]::GetRelativePath($repoRoot, $failedAttemptPath).Replace("\", "/")
        $failedAttempt = [ordered]@{
            schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; attempt = 4
            authoritative = $false; state = "failed"; assertions_started = $true
            assertions_started_at = "2026-08-07T12:03:01Z"; started_at = "2026-08-07T12:03:00Z"
            ended_at = "2026-08-07T12:03:02Z"; mutable_source_identity = $source
            source_identity_verification_state = "unverified"; environment_fingerprint = "0" * 64
            predecessor_correction_authorization = [ordered]@{ path = $thirdAuthorizationRelative; sha256 = $thirdAuthorizationSha }
            correction_consumption_receipt = [ordered]@{ path = $thirdConsumptionRelative; sha256 = $thirdConsumptionSha }
            declared_checks = @([ordered]@{ name = "environment-contract"; depends_on = @(); classification = "environment" })
            checks = @($terminalCheck); assertion_count = 1; failure_count = 1; blocked_count = 0
        }
        $failedAttemptSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $failedAttempt -Path $failedAttemptPath
        $failedSuccessorLineage.links[2].consumed_by_readiness = [ordered]@{
            attempt = 4
            start_receipt = [ordered]@{ path = $failedStartRelative; sha256 = $failedStartSha }
            consumption_receipt = [ordered]@{ path = $thirdConsumptionRelative; sha256 = $thirdConsumptionSha }
            receipt = [ordered]@{ path = $failedAttemptRelative; sha256 = $failedAttemptSha; state = "failed" }
        }
        $failedState = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; updated_at = "2026-08-07T12:03:02Z"
            source_identity = $source; source_identity_verification_state = "unverified"
            environment_fingerprint = "0" * 64
            readiness = [pscustomobject]@{ attempt = 4; state = "failed"; receipt = $failedAttemptRelative; sha256 = $failedAttemptSha }
            rehearsal = [pscustomobject]@{ state = "ineligible" }
            correction_lineage = $failedSuccessorLineage
            preflight_eligible = $false
        }
        Publish-Sprint7AEvidence -Document $failedState -OutputPath $script:statePath -Overwrite | Out-Null
        $script:attemptPath = $failedAttemptPath
        $script:startSnapshotPath = $failedStartPath
        $script:harvestPath = Join-Path $temporaryRoot "attempts/readiness-4-harvest.json"
        $script:batchPath = Join-Path $temporaryRoot "attempts/readiness-4-defect-batch.json"
        $script:correctionAuthorizationPath = Join-Path $temporaryRoot "attempts/readiness-4-correction-authorization.json"
        $finalizedConsumedLineage = Complete-Sprint8AFailedReadinessHarvest `
            -AttemptNumber 4 `
            -AttemptDocument (Get-Content -LiteralPath $failedAttemptPath -Raw | ConvertFrom-Json) `
            -AttemptSha $failedAttemptSha `
            -StateDocument $failedState
        $nextAuthorization = Get-Content -LiteralPath $script:correctionAuthorizationPath -Raw | ConvertFrom-Json
        $nextSuccessorTargets = @(
            (Join-Path $temporaryRoot "attempts/readiness-5.json"),
            (Join-Path $temporaryRoot "attempts/readiness-5.json.sha256"),
            (Join-Path $temporaryRoot "attempts/readiness-5-start.json"),
            (Join-Path $temporaryRoot "attempts/readiness-5-start.json.sha256"),
            (Join-Path $temporaryRoot "readiness-5")
        )
        if (@($finalizedConsumedLineage.links).Count -ne 4 -or
            [string]$finalizedConsumedLineage.links[3].predecessor.phase -cne "validation-readiness" -or
            [int]$finalizedConsumedLineage.links[3].predecessor.attempt -ne 4 -or
            $null -ne $finalizedConsumedLineage.links[3].consumed_by_readiness -or
            [int]$nextAuthorization.allowed_successor_attempt -ne 5 -or
            @($nextSuccessorTargets | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Failed-Readiness finalizer self-test did not append one pending exact-successor link without occupying Readiness 5."
        }
        $failedReservationHandle.lock_handle.Dispose()
        $failedReservationHandle = $null
    } finally {
        if ($null -ne $successorReservationHandle) {
            $successorReservationHandle.lock_handle.Dispose()
            $successorReservationHandle = $null
        }
        if ($null -ne $failedReservationHandle) {
            $failedReservationHandle.lock_handle.Dispose()
            $failedReservationHandle = $null
        }
        $script:evidenceRootPath = $saved.evidenceRootPath
        $script:attemptPath = $saved.attemptPath
        $script:statePath = $saved.statePath
        $script:startSnapshotPath = $saved.startSnapshotPath
        $script:harvestPath = $saved.harvestPath
        $script:batchPath = $saved.batchPath
        $script:correctionAuthorizationPath = $saved.correctionAuthorizationPath
        if (Test-Path -LiteralPath $temporaryRoot -PathType Container) {
            Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
        }
    }
}

Assert-Sprint8AReadinessFailLateGraph -Checks $declaredChecks
if ($SelfTest) {
    if ($FinalizeFailedAttempt) { throw "Readiness self-test and failed-attempt finalization are mutually exclusive." }
    Test-Sprint8AExclusiveValidationLock
    Test-Sprint8AAppendOnlyCorrectionConsumption
    Test-Sprint8AAlternatingCorrectionEpochs
    Test-Sprint8ACorrectionIdentityContinuity
    Test-Sprint8AFailedReadinessFinalization
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
    $cleanRerunState = [pscustomobject]@{
        readiness = [pscustomobject]@{ state = "passed" }
        rehearsal = [pscustomobject]@{ state = "ineligible" }
        preflight_eligible = $false
    }
    if (-not (Test-Sprint8ACleanReadinessRerunState -State $cleanRerunState)) {
        throw "Readiness self-test rejected the narrow clean pre-rehearsal rerun boundary."
    }
    $cleanRerunState.readiness.state = "failed"
    if (Test-Sprint8ACleanReadinessRerunState -State $cleanRerunState) {
        throw "Readiness self-test admitted a failed predecessor without a correction-lineage authorization."
    }
    Write-Host "Sprint 8A readiness fail-late dependency self-test passed."
    return
}
if ($Attempt -lt 1) { throw "Validation Readiness requires -Attempt with a positive, unused attempt number." }
if ($FinalizeFailedAttempt) {
    $finalizationLock = Open-Sprint8AValidationAttemptLock -Path $validationLockPath
    try {
        [void](Assert-Sprint8AReceiptSidecar -Path $statePath)
        $failedAttemptSha = Assert-Sprint8AReceiptSidecar -Path $attemptPath
        $failedAttempt = Get-Content -LiteralPath $attemptPath -Raw | ConvertFrom-Json
        $failedState = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        [void](Complete-Sprint8AFailedReadinessHarvest `
            -AttemptNumber $Attempt `
            -AttemptDocument $failedAttempt `
            -AttemptSha $failedAttemptSha `
            -StateDocument $failedState)
    } finally {
        $finalizationLock.Dispose()
    }
    Write-Host "Sprint 8A failed Readiness $Attempt has one complete harvest, batch, authorization, and pending successor lineage link."
    return
}
$validationLockHandle = $null
$priorState = $null
$launchReservation = $null
$reservationHandle = Open-Sprint8AReadinessAttemptReservation `
    -AttemptNumber $Attempt `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $evidenceRootPath `
    -StatePath $statePath `
    -LockPath $validationLockPath `
    -AttemptReceiptPath $attemptPath `
    -StartReceiptPath $startSnapshotPath `
    -AttemptLogRoot $logRoot
$validationLockHandle = $reservationHandle.lock_handle
$priorState = $reservationHandle.prior_state
$launchReservation = $reservationHandle.reservation

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
$correctionLineage = $null
$predecessorCorrectionAuthorization = $null
$correctionConsumptionReceipt = $null
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
try {
    Publish-Sprint7AEvidence -Document $startReceipt -OutputPath $attemptPath | Out-Null
    Publish-Sprint7AEvidence -Document $startReceipt -OutputPath $startSnapshotPath | Out-Null
} catch {
    if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
    throw
}

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
    if ($null -ne $correctionLineage) {
        $tip = @($correctionLineage.links)[-1]
        if ($null -ne $tip.consumed_by_readiness -and
            [int]$tip.consumed_by_readiness.attempt -eq $Attempt) {
            $tip.consumed_by_readiness.receipt = [ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                sha256 = $attemptSha
                state = [string]$startReceipt.state
            }
        }
    }
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
        correction_lineage = $correctionLineage
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
            failure_message = $null
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
    $failureMessage = $null
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
        $failureMessage = $_.Exception.Message
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
        failure_message = $failureMessage
        evidence_path = [IO.Path]::GetRelativePath($repoRoot, $log).Replace("\", "/")
        evidence_sha256 = Get-Sprint8AFileSha256 -Path $log
        produced_evidence = @($producedEvidence)
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
        if ($null -eq $launchReservation -or $null -eq $validationLockHandle -or
            -not $validationLockHandle.CanWrite) {
            throw "Readiness did not retain its pre-publication attempt reservation and exclusive lock."
        }
        $script:priorState = $launchReservation.prior_state
        if ($null -ne $script:priorState) {
            if ([bool]$launchReservation.clean_rerun) {
                $script:launchAuthorized = $true
                Write-Host "A clean pre-rehearsal Readiness rerun is authorized without correction lineage."
                return
            }
            $lineageValidation = $launchReservation.lineage_validation
            $tip = $lineageValidation.tip
            $resolvedAuthorization = $launchReservation.authorization_reference
            $authorizationPath = [string]$resolvedAuthorization.full_path
            $authorizationSha = [string]$launchReservation.authorization_sha256
            $relativeAuthorizationPath = [string]$resolvedAuthorization.path
            $script:predecessorCorrectionAuthorization = [ordered]@{
                path = $relativeAuthorizationPath
                sha256 = $authorizationSha
                predecessor_phase = [string]$tip.predecessor.phase
                predecessor_attempt = [int]$tip.predecessor.attempt
            }
            $consumptionPath = "$authorizationPath.consumption.json"
            $startSnapshotSha = Assert-Sprint8AReceiptSidecar -Path $startSnapshotPath
            $consumptionDocument = [ordered]@{
                schema_version = 1
                sprint = "sprint-8a"
                phase = "$([string]$tip.predecessor.phase)-correction-consumption"
                authoritative = $false
                state = "consumed"
                consumed_at = [DateTimeOffset]::UtcNow.ToString("o")
                authorization = [ordered]@{ path = $relativeAuthorizationPath; sha256 = $authorizationSha }
                predecessor = $tip.predecessor
                successor_readiness = [ordered]@{
                    attempt = $Attempt
                    start_receipt = [IO.Path]::GetRelativePath($repoRoot, $startSnapshotPath).Replace("\", "/")
                    start_receipt_sha256 = $startSnapshotSha
                }
            }
            $consumptionSha = Publish-Sprint8AAppendOnlyJsonReceipt -Document $consumptionDocument -Path $consumptionPath
            $relativeConsumptionPath = [IO.Path]::GetRelativePath($repoRoot, $consumptionPath).Replace("\", "/")
            $script:correctionConsumptionReceipt = [ordered]@{ path = $relativeConsumptionPath; sha256 = $consumptionSha }
            $links = @($lineageValidation.links)
            $links[-1].consumed_by_readiness = [ordered]@{
                attempt = $Attempt
                start_receipt = [ordered]@{
                    path = [IO.Path]::GetRelativePath($repoRoot, $startSnapshotPath).Replace("\", "/")
                    sha256 = $startSnapshotSha
                }
                consumption_receipt = $script:correctionConsumptionReceipt
                receipt = [ordered]@{
                    path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                    sha256 = Assert-Sprint8AReceiptSidecar -Path $attemptPath
                    state = "preparing"
                }
            }
            $script:correctionLineage = [ordered]@{ schema_version = 1; links = @($links) }
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
    [void](Assert-Sprint8ACurrentReadinessReference `
        -StateReadiness ([pscustomobject]@{
            attempt = $Attempt
            state = "passed"
            receipt = [IO.Path]::GetRelativePath($repoRoot, $resultPath).Replace("\", "/")
            sha256 = $currentReadinessSha
        }) `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $evidenceRootPath `
        -RequirePassed)
}
if ($launchAuthorized) {
    if ($null -ne $correctionLineage) {
        $tip = @($correctionLineage.links)[-1]
        if ($null -ne $tip.consumed_by_readiness -and
            [int]$tip.consumed_by_readiness.attempt -eq $Attempt) {
            $tip.consumed_by_readiness.receipt = [ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/")
                sha256 = $attemptSha
                state = [string]$receipt.state
            }
        }
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
        correction_lineage = $correctionLineage
        preflight_eligible = $false
    }) -OutputPath $statePath -Overwrite | Out-Null
}

if (-not $passed) {
    if ($launchAuthorized) {
        [void](Assert-Sprint8AReceiptSidecar -Path $statePath)
        $terminalState = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
        [void](Complete-Sprint8AFailedReadinessHarvest `
            -AttemptNumber $Attempt `
            -AttemptDocument ($receipt | ConvertTo-Json -Depth 100 | ConvertFrom-Json) `
            -AttemptSha $attemptSha `
            -StateDocument $terminalState)
    }
    if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
    throw "Sprint 8A readiness failed $($failures.Count) checks and blocked $($blocked.Count); inspect $attemptPath."
}
if ($null -ne $validationLockHandle) { $validationLockHandle.Dispose(); $validationLockHandle = $null }
Write-Host "Sprint 8A Validation Readiness passed for source $($source.commit) and environment $($environment.fingerprint)."

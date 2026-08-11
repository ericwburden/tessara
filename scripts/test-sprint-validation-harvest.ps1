[CmdletBinding()]
param(
    [string]$AttemptPath,
    [string]$HarvestPath,
    [string]$DefectBatchPath,
    [string]$CorrectionAuthorizationPath,
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [switch]$HarvestOnly,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-rehearsal-scheduler.ps1")
$allowedClassifications = @(
    "preflight/setup",
    "product",
    "harness",
    "environment",
    "flaky",
    "evidence-finalization",
    "product-decision"
)
$script:StrictCanonicalEvidencePaths = $false

function Assert-EqualIdentity {
    param($Expected, $Actual, [string]$Label)
    $expectedJson = $Expected | ConvertTo-Json -Depth 30 -Compress
    $actualJson = $Actual | ConvertTo-Json -Depth 30 -Compress
    if ($actualJson -cne $expectedJson) { throw "$Label identity does not match the attempt." }
}

function Assert-DiagnosticReceiptHeader {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][int[]]$SchemaVersions,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Label
    )
    $schema = $Document.schema_version
    $authoritative = $Document.authoritative
    if ($Document.PSObject.Properties.Name -notcontains "schema_version" -or
        $Document.PSObject.Properties.Name -notcontains "sprint" -or
        $Document.PSObject.Properties.Name -notcontains "phase" -or
        $Document.PSObject.Properties.Name -notcontains "authoritative" -or
        -not ($schema -is [int] -or $schema -is [long]) -or
        $SchemaVersions -notcontains [long]$schema -or
        $Document.sprint -isnot [string] -or
        [string]$Document.sprint -cne "sprint-8a" -or
        $Document.phase -isnot [string] -or
        [string]$Document.phase -cne $Phase -or
        $authoritative -isnot [bool] -or
        $authoritative -ne $false) {
        throw "$Label is not an accepted exact non-authoritative Sprint 8A receipt type."
    }
}

function Assert-MutableSourceIdentity {
    param(
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][string]$VerificationState
    )
    $expectedProperties = @(
        "commit", "tree", "dirty", "branch",
        "acceptance_inventory_sha256", "deployment_inputs_sha256"
    )
    $actualProperties = @($Source.PSObject.Properties.Name | Sort-Object)
    if (($actualProperties | ConvertTo-Json -Compress) -cne
        (@($expectedProperties | Sort-Object) | ConvertTo-Json -Compress) -or
        $Source.commit -isnot [string] -or
        [string]$Source.commit -notmatch '^[0-9a-f]{40}$' -or
        $Source.tree -isnot [string] -or
        [string]$Source.tree -notmatch '^[0-9a-f]{40}$' -or
        $Source.dirty -isnot [bool] -or
        $Source.branch -isnot [string] -or
        [string]::IsNullOrWhiteSpace([string]$Source.branch) -or
        $Source.acceptance_inventory_sha256 -isnot [string] -or
        [string]$Source.acceptance_inventory_sha256 -notmatch '^[0-9a-f]{64}$' -or
        $Source.deployment_inputs_sha256 -isnot [string] -or
        [string]$Source.deployment_inputs_sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "The attempt mutable source identity claim is malformed."
    }
    if ($VerificationState -ceq "verified") {
        if ([string]$Source.branch -ceq "unverified" -or
            [string]$Source.commit -ceq ("0" * 40) -or [string]$Source.tree -ceq ("0" * 40)) {
            throw "A verified mutable source identity still carries placeholder claims."
        }
    } elseif ($VerificationState -in @("unverified", "failed")) {
        if ($Source.dirty -ne $false -or [string]$Source.branch -cne "unverified" -or
            [string]$Source.commit -cne ("0" * 40) -or [string]$Source.tree -cne ("0" * 40) -or
            [string]$Source.acceptance_inventory_sha256 -cne ("0" * 64) -or
            [string]$Source.deployment_inputs_sha256 -cne ("0" * 64)) {
            throw "An unverified mutable source claim must remain the exact explicit placeholder identity."
        }
    } else {
        throw "The attempt omits an explicit source identity verification state."
    }
}

function Assert-EnvironmentFingerprint {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Document.PSObject.Properties.Name -notcontains "environment_fingerprint" -or
        $Document.environment_fingerprint -isnot [string] -or
        [string]$Document.environment_fingerprint -notmatch '^[0-9a-f]{64}$') {
        throw "$Label environment fingerprint must be exactly 64 lowercase hexadecimal characters."
    }
}

function Assert-HashedFileEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Label,
        [switch]$SkipFileEvidence
    )

    if ([string]::IsNullOrWhiteSpace([string]$Evidence.path) -or
        [string]$Evidence.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "$Label lacks an exact path and SHA-256 digest."
    }
    if ($script:StrictCanonicalEvidencePaths) {
        [void](Assert-Sprint8ACanonicalEvidencePath `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$Evidence.path) `
            -Label $Label)
    }
    if (-not $SkipFileEvidence) {
        $reference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$Evidence.path) `
            -AllowLegacyAbsolute:(-not $script:StrictCanonicalEvidencePaths)
        if ((Get-Sprint8AFileSha256 -Path ([string]$reference.full_path)) -cne [string]$Evidence.sha256) {
            throw "$Label digest does not match its retained file."
        }
    }
}

function Assert-DeferredLanePriorPassingReceipt {
    param(
        [Parameter(Mandatory)]$Deferred,
        [Parameter(Mandatory)][int]$CurrentAttempt,
        [switch]$SkipFileEvidence
    )

    Assert-HashedFileEvidence `
        -Evidence $Deferred.prior_passing_receipt `
        -Label "Deferred lane '$($Deferred.name)' prior passing receipt" `
        -SkipFileEvidence:$SkipFileEvidence
    Assert-MutableSourceIdentity `
        -Source $Deferred.prior_source_identity `
        -VerificationState "verified"
    if ($Deferred.PSObject.Properties.Name -notcontains "prior_environment_identity" -or
        @($Deferred.prior_environment_identity.PSObject.Properties.Name).Count -ne 1 -or
        $Deferred.prior_environment_identity.PSObject.Properties.Name -cnotcontains "fingerprint" -or
        $Deferred.prior_environment_identity.fingerprint -isnot [string] -or
        [string]$Deferred.prior_environment_identity.fingerprint -notmatch '^[0-9a-f]{64}$') {
        throw "Deferred lane '$($Deferred.name)' has a malformed prior environment identity."
    }
    if ($SkipFileEvidence) { return }

    $reference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Deferred.prior_passing_receipt.path) `
        -AllowLegacyAbsolute:(-not $script:StrictCanonicalEvidencePaths)
    $sidecarSha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$reference.full_path)
    if ([string]$sidecarSha256 -cne [string]$Deferred.prior_passing_receipt.sha256) {
        throw "Deferred lane '$($Deferred.name)' prior passing receipt differs from its sidecar or embedded SHA-256."
    }
    $receipt = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
    $schema = $receipt.schema_version
    if (($schema -isnot [int] -and $schema -isnot [long]) -or
        @(1, 2) -notcontains [int]$schema -or
        $receipt.sprint -isnot [string] -or [string]$receipt.sprint -cne "sprint-8a" -or
        $receipt.phase -isnot [string] -or [string]$receipt.phase -cne "candidate-rehearsal-lane" -or
        $receipt.authoritative -isnot [bool] -or [bool]$receipt.authoritative -or
        ($receipt.attempt -isnot [int] -and $receipt.attempt -isnot [long]) -or
        [int]$receipt.attempt -lt 1 -or [int]$receipt.attempt -ge $CurrentAttempt -or
        $receipt.PSObject.Properties.Name -notcontains "result" -or $null -eq $receipt.result) {
        throw "Deferred lane '$($Deferred.name)' does not bind an exact earlier non-authoritative Candidate Rehearsal lane receipt."
    }
    Assert-EqualIdentity `
        -Expected $Deferred.prior_source_identity `
        -Actual $receipt.mutable_source_identity `
        -Label "Deferred lane '$($Deferred.name)' prior source"
    if ([string]$receipt.environment_fingerprint -cne
        [string]$Deferred.prior_environment_identity.fingerprint) {
        throw "Deferred lane '$($Deferred.name)' prior environment identity differs from its retained lane receipt."
    }
    if ([string]$receipt.result.name -cne [string]$Deferred.name -or
        [string]$receipt.result.state -cne "passed" -or
        $receipt.result.PSObject.Properties.Name -notcontains "assertions_started" -or
        $receipt.result.assertions_started -isnot [bool] -or
        -not [bool]$receipt.result.assertions_started) {
        throw "Deferred lane '$($Deferred.name)' prior receipt does not prove that exact lane previously executed and passed."
    }
}

function Assert-CandidateRehearsalScheduleBinding {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Harvest,
        [Parameter(Mandatory)][object[]]$DeclaredChecks,
        [switch]$SkipFileEvidence
    )

    foreach ($entry in @(
        [pscustomobject]@{ label = "attempt"; document = $Attempt },
        [pscustomobject]@{ label = "harvest"; document = $Harvest }
    )) {
        if ($entry.document.PSObject.Properties.Name -notcontains "immutable_start_receipt" -or
            $null -eq $entry.document.immutable_start_receipt -or
            [string]$entry.document.immutable_start_receipt.path -notmatch
                '(^|/)attempts/candidate-rehearsal-[0-9]+-start\.json$' -or
            [string]$entry.document.immutable_start_receipt.sha256 -notmatch '^[0-9a-f]{64}$' -or
            $entry.document.PSObject.Properties.Name -notcontains "schedule_sha256" -or
            [string]$entry.document.schedule_sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "Schema-3 Candidate Rehearsal $($entry.label) omits its immutable start and schedule binding."
        }
    }
    if ((($Attempt.immutable_start_receipt | ConvertTo-Json -Depth 10 -Compress) -cne
            ($Harvest.immutable_start_receipt | ConvertTo-Json -Depth 10 -Compress)) -or
        [string]$Attempt.schedule_sha256 -cne [string]$Harvest.schedule_sha256) {
        throw "Candidate Rehearsal attempt and harvest bind different immutable starts or schedules."
    }
    if ([int]$Attempt.attempt -gt 32) {
        foreach ($entry in @(
            [pscustomobject]@{ label = "Candidate Rehearsal attempt immutable start"; value = $Attempt.immutable_start_receipt },
            [pscustomobject]@{ label = "Candidate Rehearsal harvest immutable start"; value = $Harvest.immutable_start_receipt }
        )) {
            [void](Assert-Sprint8ACanonicalEvidencePath `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$entry.value.path) `
                -Label ([string]$entry.label))
        }
    }
    if ($SkipFileEvidence) { return }

    $startReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Attempt.immutable_start_receipt.path) `
        -AllowLegacyAbsolute:([int]$Attempt.attempt -le 32)
    $expectedEvidenceRoot = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
    }
    $expectedStartPath = [IO.Path]::GetFullPath((Join-Path `
        $expectedEvidenceRoot `
        "attempts/candidate-rehearsal-$([int]$Attempt.attempt)-start.json"))
    if ([IO.Path]::GetFullPath([string]$startReference.full_path) -cne $expectedStartPath -or
        (Assert-Sprint8AReceiptSidecar -Path ([string]$startReference.full_path)) -cne
            [string]$Attempt.immutable_start_receipt.sha256) {
        throw "Candidate Rehearsal immutable start path or sidecar is not exact."
    }
    $start = Get-Content -LiteralPath ([string]$startReference.full_path) -Raw | ConvertFrom-Json
    $declaredNames = @($DeclaredChecks | ForEach-Object { [string]$_.name })
    if (($start.schema_version -isnot [int] -and $start.schema_version -isnot [long]) -or
        [int]$start.schema_version -ne 3 -or
        [string]$start.sprint -cne "sprint-8a" -or
        [string]$start.phase -cne "candidate-rehearsal-start" -or
        [int]$start.attempt -ne [int]$Attempt.attempt -or
        $start.authoritative -isnot [bool] -or [bool]$start.authoritative -or
        [string]$start.schedule_sha256 -cne [string]$Attempt.schedule_sha256 -or
        [string]$start.schedule_sha256 -cne (Get-Sprint8ARehearsalJsonSha256 -Document $start.schedule) -or
        (@($start.declared_lanes | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($declaredNames -join "`n") -or
        [string]::IsNullOrWhiteSpace([string]$start.schedule_selection.source) -or
        [string]::IsNullOrWhiteSpace([string]$start.schedule_selection.reason)) {
        throw "Candidate Rehearsal immutable start does not retain its exact deterministic schedule declaration."
    }
    $startDeclaredChecks = Get-Sprint8AOptionalObjectPropertyValue `
        -InputObject $start `
        -Name "declared_checks"
    if ($null -eq $startDeclaredChecks) {
        if ([int]$Attempt.attempt -gt 32) {
            throw "New Candidate Rehearsal immutable starts must retain the complete exact declared-check graph, not names alone."
        }
    } else {
        if ((@($startDeclaredChecks) | ConvertTo-Json -Depth 50 -Compress) -cne
            ($DeclaredChecks | ConvertTo-Json -Depth 50 -Compress)) {
            throw "Candidate Rehearsal immutable start declared checks differ from the terminal attempt graph."
        }
        Assert-Sprint8ADeclaredEvidencePaths `
            -Checks @($startDeclaredChecks) `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Label "Candidate Rehearsal immutable start declarations"
    }
    Assert-Sprint8ARehearsalScheduleContract `
        -Schedule $start.schedule `
        -Checks $DeclaredChecks `
        -ExpectedAttempt ([int]$Attempt.attempt) | Out-Null
    Assert-Sprint8ARehearsalDeferredCounterBinding `
        -Schedule $start.schedule `
        -TerminalChecks @($Attempt.checks)
    Assert-HashedFileEvidence `
        -Evidence $start.readiness_receipt `
        -Label "Candidate Rehearsal immutable Readiness prerequisite" `
        -SkipFileEvidence:$SkipFileEvidence
    Assert-HashedFileEvidence `
        -Evidence $start.validation_state_receipt `
        -Label "Candidate Rehearsal immutable validation-state capture" `
        -SkipFileEvidence:$SkipFileEvidence
    $stateCapture = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$start.validation_state_receipt.path) `
        -AllowLegacyAbsolute:([int]$Attempt.attempt -le 32)
    if ((Assert-Sprint8AReceiptSidecar -Path ([string]$stateCapture.full_path)) -cne
        [string]$start.validation_state_receipt.sha256) {
        throw "Candidate Rehearsal immutable validation-state capture sidecar is stale."
    }
}

function Assert-CandidateRehearsalLaneReceipt {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Result,
        [switch]$SkipFileEvidence
    )

    if ($Result.PSObject.Properties.Name -notcontains "lane_receipt" -or
        $null -eq $Result.lane_receipt) {
        throw "Candidate Rehearsal lane '$($Result.name)' omits its exact lane receipt."
    }
    Assert-HashedFileEvidence `
        -Evidence $Result.lane_receipt `
        -Label "Candidate Rehearsal lane '$($Result.name)' receipt" `
        -SkipFileEvidence:$SkipFileEvidence
    if ($SkipFileEvidence) { return }

    $reference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Result.lane_receipt.path)
    $sidecar = Assert-Sprint8AReceiptSidecar -Path ([string]$reference.full_path)
    $lane = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
    $expectedResult = $Result | ConvertTo-Json -Depth 100 | ConvertFrom-Json
    $expectedResult.PSObject.Properties.Remove("lane_receipt")
    $identityMatches = (($lane.mutable_source_identity | ConvertTo-Json -Depth 30 -Compress) -ceq
            ($Attempt.mutable_source_identity | ConvertTo-Json -Depth 30 -Compress)) -and
        [string]$lane.environment_fingerprint -ceq [string]$Attempt.environment_fingerprint
    $preAuthenticationLifecycleIdentity = [string]$Result.name -ceq "attempt-state-prerequisite" -and
        [string]$lane.identity_binding -ceq "pre_authentication_lifecycle_placeholder" -and
        [string]$lane.mutable_source_identity.commit -ceq ("0" * 40) -and
        [string]$lane.mutable_source_identity.tree -ceq ("0" * 40) -and
        [bool]$lane.mutable_source_identity.dirty -eq $false -and
        [string]$lane.mutable_source_identity.branch -ceq "unverified" -and
        [string]$lane.mutable_source_identity.acceptance_inventory_sha256 -ceq ("0" * 64) -and
        [string]$lane.mutable_source_identity.deployment_inputs_sha256 -ceq ("0" * 64) -and
        [string]$lane.environment_fingerprint -ceq ("0" * 64) -and
        @("passed", "failed") -ccontains [string]$lane.result.state -and
        $lane.result.assertions_started -is [bool] -and
        [bool]$lane.result.assertions_started

    if ($sidecar -cne [string]$Result.lane_receipt.sha256 -or
        ($lane.schema_version -isnot [int] -and $lane.schema_version -isnot [long]) -or
        [int]$lane.schema_version -ne 2 -or
        [string]$lane.sprint -cne "sprint-8a" -or
        [string]$lane.phase -cne "candidate-rehearsal-lane" -or
        [int]$lane.attempt -ne [int]$Attempt.attempt -or
        [bool]$lane.authoritative -or
        (-not $identityMatches -and -not $preAuthenticationLifecycleIdentity) -or
        (($lane.result | ConvertTo-Json -Depth 100 -Compress) -cne
            ($expectedResult | ConvertTo-Json -Depth 100 -Compress))) {
        throw "Candidate Rehearsal lane '$($Result.name)' receipt is stale or differs from terminal accounting."
    }
}

function Assert-DeferredTerminalCheckEvidence {
    param(
        [Parameter(Mandatory)]$Declared,
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)][object[]]$TerminalChecks,
        [Parameter(Mandatory)][int]$CurrentAttempt,
        [switch]$SkipFileEvidence
    )

    foreach ($property in @(
        "wave", "scheduling_reason", "prior_passing_receipt", "prior_source_identity",
        "prior_environment_identity", "current_correction_impact", "non_impact_rationale",
        "consecutive_deferral_count", "mandatory_by_attempt", "prerequisite_state",
        "diagnostic_history_notice", "produced_evidence", "nested_blocked_checks",
        "nested_failed_checks"
    )) {
        if ($Result.PSObject.Properties.Name -notcontains $property) {
            throw "Deferred lane '$($Declared.name)' omits exact '$property' accounting."
        }
    }
    if ([string]$Result.wave -cne "B" -or
        [string]::IsNullOrWhiteSpace([string]$Result.scheduling_reason) -or
        [string]$Result.dependency_reason -cne
            "Wave B was deferred after Wave A failed; this lane executed no assertions." -or
        $null -ne $Result.started_at -or $null -ne $Result.ended_at -or
        $null -ne $Result.duration_ms -or $null -ne $Result.exit_status -or
        [bool]$Result.assertions_started -or $null -ne $Result.assertions_started_at -or
        $null -ne $Result.classification -or $null -ne $Result.classification_source -or
        $null -ne $Result.failure_message -or
        $null -ne $Result.evidence_path -or $null -ne $Result.evidence_sha256 -or
        @($Result.produced_evidence).Count -ne 0 -or
        @($Result.nested_blocked_checks).Count -ne 0 -or
        @($Result.nested_failed_checks).Count -ne 0 -or
        [string]$Result.diagnostic_history_notice -cne $script:Sprint8ADiagnosticHistoryNotice) {
        throw "Deferred lane '$($Declared.name)' claims execution, evidence, failure semantics, or authoritative prior proof."
    }
    if ($Result.current_correction_impact.PSObject.Properties.Name -notcontains "affected" -or
        $Result.current_correction_impact.affected -isnot [bool] -or
        [bool]$Result.current_correction_impact.affected -or
        [string]$Result.current_correction_impact.decision -cne "outside_correction_impact" -or
        [string]::IsNullOrWhiteSpace([string]$Result.current_correction_impact.rationale) -or
        [string]$Result.non_impact_rationale -cne [string]$Result.current_correction_impact.rationale) {
        throw "Deferred lane '$($Declared.name)' is not proven outside the current correction impact cone."
    }
    $count = $Result.consecutive_deferral_count
    $mandatory = $Result.mandatory_by_attempt
    if (($count -isnot [int] -and $count -isnot [long]) -or
        [int]$count -lt 1 -or [int]$count -gt 3 -or
        ($mandatory -isnot [int] -and $mandatory -isnot [long]) -or
        [int]$mandatory -ne ($CurrentAttempt + (4 - [int]$count))) {
        throw "Deferred lane '$($Declared.name)' has invalid bounded-deferral accounting."
    }
    $expectedPrerequisites = @($Declared.depends_on | ForEach-Object { [string]$_ })
    $prerequisites = @($Result.prerequisite_state)
    $prerequisiteNames = @($prerequisites | ForEach-Object { [string]$_.name })
    if ($prerequisites.Count -ne $expectedPrerequisites.Count -or
        @($prerequisiteNames | Sort-Object -Unique).Count -ne $prerequisiteNames.Count -or
        ($prerequisiteNames -join "`n") -cne ($expectedPrerequisites -join "`n")) {
        throw "Deferred lane '$($Declared.name)' does not retain its exact prerequisite-state inventory."
    }
    foreach ($prerequisite in $prerequisites) {
        if (@("passed", "failed", "blocked", "deferred", "not_terminal") -cnotcontains
            [string]$prerequisite.state) {
            throw "Deferred lane '$($Declared.name)' has invalid prerequisite state '$($prerequisite.state)'."
        }
        $terminal = @($TerminalChecks | Where-Object name -CEQ ([string]$prerequisite.name))
        if ($terminal.Count -ne 1 -or
            ([string]$prerequisite.state -cne "not_terminal" -and
                [string]$prerequisite.state -cne [string]$terminal[0].state) -or
            ([string]$prerequisite.state -ceq "not_terminal" -and
                [string]$terminal[0].state -cne "deferred")) {
            throw "Deferred lane '$($Declared.name)' prerequisite state is inconsistent with current terminal accounting."
        }
    }
    Assert-DeferredLanePriorPassingReceipt `
        -Deferred $Result `
        -CurrentAttempt $CurrentAttempt `
        -SkipFileEvidence:$SkipFileEvidence
}

function Assert-AcyclicCheckGraph {
    param([Parameter(Mandatory)][object[]]$Checks)

    $names = @($Checks | ForEach-Object { [string]$_.name })
    if ($names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count -or
        @($names | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0) {
        throw "The attempt must declare one nonempty unique name for every check."
    }
    foreach ($check in $Checks) {
        foreach ($dependency in @($check.depends_on)) {
            if ([string]$dependency -ceq [string]$check.name -or $names -cnotcontains [string]$dependency) {
                throw "Check '$($check.name)' contains an invalid dependency '$dependency'."
            }
        }
    }
    $visiting = @{}
    $visited = @{}
    function Visit-Check([string]$Name) {
        if ($visiting.ContainsKey($Name)) { throw "The declared check graph contains a cycle through '$Name'." }
        if ($visited.ContainsKey($Name)) { return }
        $visiting[$Name] = $true
        $check = @($Checks | Where-Object name -CEQ $Name)[0]
        foreach ($dependency in @($check.depends_on)) { Visit-Check -Name ([string]$dependency) }
        $visiting.Remove($Name)
        $visited[$Name] = $true
    }
    foreach ($name in $names) { Visit-Check -Name $name }
}

function Resolve-HarvestExecutionStart {
    param(
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)][int]$CurrentAttempt,
        [switch]$SkipFileEvidence
    )

    try {
        return ConvertTo-Sprint8ADateTimeOffset -Value $Result.started_at -Label "harvest check start"
    } catch {
        if ($_.Exception.Message -notmatch "has no UTC offset" -or
            [string]$Result.state -cne "failed" -or
            [string]$Result.classification -cne "harness" -or
            [string]$Result.classification_source -cne "process_loss_recovery" -or
            [string]$Result.started_at -cne [string]$Result.assertions_started_at -or
            [string]$Result.started_at -notmatch '^\d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2}$') {
            throw
        }
    }

    $captures = @($Result.produced_evidence | Where-Object {
        [string]$_.path -match '-process-loss\.json$'
    })
    if ($captures.Count -ne 1 -or $SkipFileEvidence) {
        throw "Process-loss check '$($Result.name)' cannot recover its timestamp without one authenticated capture."
    }
    Assert-HashedFileEvidence -Evidence $captures[0] -Label "Process-loss timestamp capture"
    $resolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$captures[0].path)
    $capture = Get-Content -LiteralPath $resolved.full_path -Raw | ConvertFrom-Json
    if ([string]$capture.phase -cne "candidate-rehearsal-lane" -or
        [int]$capture.attempt -ne $CurrentAttempt -or
        [string]$capture.result.name -cne [string]$Result.name -or
        [string]$capture.result.state -cne "executing" -or
        -not [bool]$capture.result.assertions_started) {
        throw "Process-loss check '$($Result.name)' has an invalid executing capture."
    }
    $capturedStart = ConvertTo-Sprint8ADateTimeOffset `
        -Value $capture.result.started_at `
        -Label "process-loss captured start"
    $capturedAssertionStart = ConvertTo-Sprint8ADateTimeOffset `
        -Value $capture.result.assertions_started_at `
        -Label "process-loss captured assertion start"
    if ($capturedStart -ne $capturedAssertionStart -or
        $capturedStart.ToString("MM/dd/yyyy HH:mm:ss", [Globalization.CultureInfo]::InvariantCulture) -cne
            [string]$Result.started_at) {
        throw "Process-loss check '$($Result.name)' timestamp does not match its authenticated executing capture."
    }
    $capturedStart
}

function Assert-TerminalCheckEvidence {
    param(
        [Parameter(Mandatory)]$Declared,
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)][object[]]$TerminalChecks,
        [Parameter(Mandatory)][int]$CurrentAttempt,
        [switch]$SkipFileEvidence
    )

    if ($Declared.PSObject.Properties.Name -contains "command" -and
        [string]$Result.command -cne [string]$Declared.command) {
        throw "Check '$($Declared.name)' terminal command differs from its declaration."
    }
    $state = [string]$Result.state
    if (@("passed", "failed", "blocked", "deferred") -cnotcontains $state) {
        throw "Check '$($Declared.name)' is not passed, failed, blocked, or deferred."
    }
    if ($Result.PSObject.Properties.Name -notcontains "assertions_started" -or
        $Result.assertions_started -isnot [bool] -or
        $Result.PSObject.Properties.Name -notcontains "assertions_started_at") {
        throw "Check '$($Declared.name)' omits its exact assertion-start boundary."
    }
    if ($state -ceq "blocked") {
        $failedDependencies = @($Declared.depends_on | Where-Object {
            $dependencyName = [string]$_
            $dependency = @($TerminalChecks | Where-Object name -CEQ $dependencyName)
            $dependency.Count -ne 1 -or [string]$dependency[0].state -cne "passed"
        })
        if ($failedDependencies.Count -eq 0) {
            throw "Blocked check '$($Declared.name)' has no failed or blocked declared prerequisite."
        }
        $reason = [string]$Result.dependency_reason
        if ([string]::IsNullOrWhiteSpace($reason) -or
            @($failedDependencies | Where-Object { -not $reason.Contains([string]$_) }).Count -gt 0) {
            throw "Blocked check '$($Declared.name)' lacks its exact failed prerequisite names."
        }
        if ($null -ne $Result.exit_status -or
            -not [string]::IsNullOrWhiteSpace([string]$Result.started_at) -or
            [bool]$Result.assertions_started -or
            -not [string]::IsNullOrWhiteSpace([string]$Result.assertions_started_at)) {
            throw "Blocked check '$($Declared.name)' must not claim command execution."
        }
        return
    }
    if ($state -ceq "deferred") {
        Assert-DeferredTerminalCheckEvidence `
            -Declared $Declared `
            -Result $Result `
            -TerminalChecks $TerminalChecks `
            -CurrentAttempt $CurrentAttempt `
            -SkipFileEvidence:$SkipFileEvidence
        return
    }

    $nonpassingPrerequisites = @($Declared.depends_on | Where-Object {
        $dependencyName = [string]$_
        $dependency = @($TerminalChecks | Where-Object {
            [string]$_.name -ceq $dependencyName
        })
        $dependency.Count -ne 1 -or [string]$dependency[0].state -cne "passed"
    })
    if ($nonpassingPrerequisites.Count -gt 0) {
        throw "Executed check '$($Declared.name)' has nonpassing declared prerequisite(s): $($nonpassingPrerequisites -join ', ')."
    }
    if ([string]::IsNullOrWhiteSpace([string]$Result.command) -or
        [string]::IsNullOrWhiteSpace([string]$Result.started_at) -or
        [string]::IsNullOrWhiteSpace([string]$Result.ended_at)) {
        throw "Executed check '$($Declared.name)' lacks command or timestamps."
    }
    $started = Resolve-HarvestExecutionStart `
        -Result $Result `
        -CurrentAttempt $CurrentAttempt `
        -SkipFileEvidence:$SkipFileEvidence
    $ended = ConvertTo-Sprint8ADateTimeOffset -Value $Result.ended_at -Label "harvest check end"
    if (-not [bool]$Result.assertions_started -or
        [string]::IsNullOrWhiteSpace([string]$Result.assertions_started_at)) {
        throw "Executed check '$($Declared.name)' does not prove that assertions started."
    }
    $assertionsStarted = if ([string]$Result.assertions_started_at -match '(?:Z|[+-]\d{2}:\d{2})$') {
        ConvertTo-Sprint8ADateTimeOffset `
            -Value $Result.assertions_started_at `
            -Label "harvest assertion start"
    } else {
        $started
    }
    if ($ended -lt $started -or
        $assertionsStarted -lt $started -or
        $assertionsStarted -gt $ended -or
        [double]$Result.duration_ms -lt 0) {
        throw "Executed check '$($Declared.name)' has invalid chronology."
    }
    if (($state -ceq "passed" -and [int]$Result.exit_status -ne 0) -or
        ($state -ceq "failed" -and [int]$Result.exit_status -eq 0)) {
        throw "Executed check '$($Declared.name)' has an exit status inconsistent with '$state'."
    }
    if ($state -ceq "failed" -and $allowedClassifications -cnotcontains [string]$Result.classification) {
        throw "Failed check '$($Declared.name)' has an unsupported classification."
    }
    if ($state -ceq "passed" -and -not [string]::IsNullOrWhiteSpace([string]$Result.classification)) {
        throw "Passed check '$($Declared.name)' must not retain a failure classification."
    }
    Assert-HashedFileEvidence -Evidence ([pscustomobject]@{
        path = [string]$Result.evidence_path
        sha256 = [string]$Result.evidence_sha256
    }) -Label "Executed check '$($Declared.name)' raw evidence" -SkipFileEvidence:$SkipFileEvidence
    if ($Result.PSObject.Properties.Name -contains "produced_evidence") {
        foreach ($evidence in @($Result.produced_evidence | Where-Object { $null -ne $_ })) {
            Assert-HashedFileEvidence -Evidence $evidence `
                -Label "Executed check '$($Declared.name)' produced evidence" `
                -SkipFileEvidence:$SkipFileEvidence
        }
    }
}

function Assert-CandidateRehearsalHarvestComplete {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Harvest,
        [Parameter(Mandatory)]$Batch,
        [switch]$SkipFileEvidence
    )

    Assert-DiagnosticReceiptHeader -Document $Attempt -SchemaVersions @(2, 3) -Phase "candidate-rehearsal" -Label "Attempt"
    $attemptSchema = [int]$Attempt.schema_version
    $script:StrictCanonicalEvidencePaths = $attemptSchema -eq 3 -and [int]$Attempt.attempt -gt 32
    Assert-DiagnosticReceiptHeader `
        -Document $Harvest `
        -SchemaVersions $(if ($attemptSchema -eq 3) { @(2) } else { @(1) }) `
        -Phase "candidate-rehearsal-harvest" `
        -Label "Harvest"
    Assert-DiagnosticReceiptHeader `
        -Document $Batch `
        -SchemaVersions $(if ($attemptSchema -eq 3) { @(2) } else { @(1) }) `
        -Phase "candidate-rehearsal-defect-batch" `
        -Label "Defect batch"
    if ($Attempt.PSObject.Properties.Name -notcontains "source_identity_verification_state") {
        throw "Attempt omits explicit mutable-source verification state."
    }
    if ($Attempt.PSObject.Properties.Name -notcontains "environment_identity" -or
        $Attempt.environment_identity.PSObject.Properties.Name -notcontains "verification_state" -or
        @("verified", "unverified", "failed") -cnotcontains [string]$Attempt.environment_identity.verification_state) {
        throw "Attempt omits explicit environment verification state."
    }
    Assert-MutableSourceIdentity -Source $Attempt.mutable_source_identity -VerificationState ([string]$Attempt.source_identity_verification_state)
    Assert-EnvironmentFingerprint -Document $Attempt -Label "Attempt"
    Assert-EnvironmentFingerprint -Document $Harvest -Label "Harvest"
    Assert-EnvironmentFingerprint -Document $Batch -Label "Defect batch"
    $acceptedAttemptStates = if ($attemptSchema -eq 3) { @("harvesting", "failed", "incomplete") } else { @("harvesting", "failed") }
    if ([string]$Attempt.state -cnotin $acceptedAttemptStates -or
        -not [bool]$Attempt.assertions_started -or
        [string]::IsNullOrWhiteSpace([string]$Attempt.assertions_started_at)) {
        throw "Only an assertion-bearing failed/harvesting attempt may authorize correction."
    }
    if ([string]$Harvest.state -cne "harvest_complete") {
        throw "Correction/restart is forbidden until the attempt reaches harvest_complete."
    }
    if ([string]::IsNullOrWhiteSpace([string]$Harvest.attempt_receipt.path) -or
        [string]$Harvest.attempt_receipt.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Harvest does not bind the exact failed attempt receipt and digest."
    }
    Assert-HashedFileEvidence -Evidence $Harvest.attempt_receipt -Label "Harvest attempt receipt" -SkipFileEvidence:$SkipFileEvidence
    foreach ($document in @($Harvest, $Batch)) {
        if ([int]$document.attempt -ne [int]$Attempt.attempt -or [string]$document.sprint -cne [string]$Attempt.sprint) {
            throw "Harvest and batch must bind the exact sprint and attempt."
        }
        Assert-EqualIdentity -Expected $Attempt.mutable_source_identity -Actual $document.mutable_source_identity -Label "Harvest/batch source"
        if ([string]$document.environment_fingerprint -cne [string]$Attempt.environment_fingerprint) {
            throw "Harvest/batch environment fingerprint does not match the attempt."
        }
    }

    if ($Attempt.PSObject.Properties.Name -notcontains "declared_checks") {
        throw "Attempt receipt omits its immutable declared check graph."
    }
    $declared = @($Attempt.declared_checks)
    Assert-AcyclicCheckGraph -Checks $declared
    if ($attemptSchema -eq 3 -and [int]$Attempt.attempt -gt 32) {
        Assert-Sprint8ADeclaredEvidencePaths `
            -Checks $declared `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Label "Candidate Rehearsal attempt declarations"
        foreach ($prerequisite in @($Attempt.prerequisite_receipts)) {
            Assert-HashedFileEvidence `
                -Evidence $prerequisite `
                -Label "Candidate Rehearsal prerequisite receipt" `
                -SkipFileEvidence:$SkipFileEvidence
        }
        [void](Assert-Sprint8ACanonicalEvidencePath `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$Harvest.receipt_path) `
            -Label "Candidate Rehearsal harvest receipt path")
    }
    if ($attemptSchema -eq 3) {
        Assert-CandidateRehearsalScheduleBinding `
            -Attempt $Attempt `
            -Harvest $Harvest `
            -DeclaredChecks $declared `
            -SkipFileEvidence:$SkipFileEvidence
    }
    $terminal = @($Harvest.checks)
    $attemptTerminal = @($Attempt.checks)
    foreach ($check in $declared) {
        $result = @($terminal | Where-Object name -CEQ ([string]$check.name))
        if ($result.Count -ne 1) { throw "Check '$($check.name)' does not have exactly one terminal result." }
        Assert-TerminalCheckEvidence `
            -Declared $check `
            -Result $result[0] `
            -TerminalChecks $terminal `
            -CurrentAttempt ([int]$Attempt.attempt) `
            -SkipFileEvidence:$SkipFileEvidence
        if ($attemptSchema -eq 3 -and [int]$Attempt.attempt -gt 32) {
            Assert-CandidateRehearsalLaneReceipt `
                -Attempt $Attempt `
                -Result $result[0] `
                -SkipFileEvidence:$SkipFileEvidence
        }
    }
    if ($terminal.Count -ne $declared.Count) { throw "Harvest contains undeclared or duplicate check results." }
    if ($attemptTerminal.Count -ne $terminal.Count -or
        (($attemptTerminal | ConvertTo-Json -Depth 30 -Compress) -cne
            ($terminal | ConvertTo-Json -Depth 30 -Compress))) {
        throw "Terminal attempt receipt does not retain the exact harvested terminal check results."
    }

    $failedTerminal = @($terminal | Where-Object state -CEQ "failed")
    $blockedTerminal = @($terminal | Where-Object state -CEQ "blocked")
    $deferredTerminal = @($terminal | Where-Object state -CEQ "deferred")
    $passedTerminal = @($terminal | Where-Object state -CEQ "passed")
    $assertionBearingTerminal = @($terminal | Where-Object assertions_started -EQ $true)
    if (([string]$Attempt.source_identity_verification_state -cne "verified" -or
            [bool]$Attempt.mutable_source_identity.dirty) -and
        @($failedTerminal | Where-Object name -CEQ "validation-readiness-prerequisite").Count -ne 1) {
        throw "Unverified or dirty source claims are harvestable only when the declared source/readiness collection lane failed."
    }
    if (([string]$Attempt.environment_identity.verification_state -cne "verified" -or
            [string]$Attempt.environment_fingerprint -ceq ("0" * 64)) -and
        @($failedTerminal | Where-Object name -CEQ "validation-readiness-prerequisite").Count -ne 1) {
        throw "Unverified environment claims are harvestable only when the readiness/environment authentication lane failed."
    }
    $nestedBlocked = [Collections.Generic.List[object]]::new()
    $nestedFailed = [Collections.Generic.List[object]]::new()
    foreach ($result in $terminal) {
        if ($result.PSObject.Properties.Name -notcontains "nested_blocked_checks" -or
            $result.PSObject.Properties.Name -notcontains "nested_failed_checks") {
            throw "Terminal check '$($result.name)' omits its nested UAT terminal inventories."
        }
        $nested = @($result.nested_blocked_checks)
        if ($nested.Count -gt 0 -and [string]$result.name -cne "uat-diagnostics") {
            throw "Only the UAT diagnostic projection may report nested blocked checks."
        }
        foreach ($blockedScenario in $nested) {
            if ([string]$blockedScenario.name -notmatch '^uat-diagnostics/UAT-8A-[0-9]{2}$' -or
                [string]$blockedScenario.parent_check -cne "uat-diagnostics" -or
                [string]::IsNullOrWhiteSpace([string]$blockedScenario.dependency_reason)) {
                throw "Nested blocked UAT scenarios require exact identity, parent, and dependency reason."
            }
            $blockedBy = @($blockedScenario.blocked_by | ForEach-Object { [string]$_ })
            if ($blockedBy.Count -eq 0 -or
                @($blockedBy | Sort-Object -Unique).Count -ne $blockedBy.Count -or
                @($blockedBy | Where-Object {
                    $dependencyName = [string]$_
                    $dependency = @($terminal | Where-Object { [string]$_.name -ceq $dependencyName })
                    $dependency.Count -ne 1 -or [string]$dependency[0].state -ceq "passed" -or
                        -not ([string]$blockedScenario.dependency_reason).Contains($dependencyName)
                }).Count -gt 0) {
                throw "Nested blocked UAT scenario '$($blockedScenario.name)' does not bind every exact nonpassing prerequisite."
            }
            $nestedBlocked.Add($blockedScenario)
        }
        $nestedFailures = @($result.nested_failed_checks)
        if ($nestedFailures.Count -gt 0 -and [string]$result.name -cne "uat-diagnostics") {
            throw "Only the UAT diagnostic projection may report nested failed semantic assertions."
        }
        if ($nestedFailures.Count -gt 0 -and [string]$result.state -cne "passed") {
            throw "Nested semantic assertion failures must not double count the outer UAT projection lane as failed."
        }
        foreach ($semanticFailure in $nestedFailures) {
            $expectedName = "uat-diagnostics/$([string]$semanticFailure.scenario)/$([string]$semanticFailure.assertion_id)"
            if ([string]$semanticFailure.name -cne $expectedName -or
                [string]$semanticFailure.scenario -notmatch '^UAT-8A-[0-9]{2}$' -or
                [string]$semanticFailure.assertion_id -notmatch '^[a-z0-9-]+$' -or
                [string]$semanticFailure.parent_check -cne "uat-diagnostics" -or
                $allowedClassifications -cnotcontains [string]$semanticFailure.classification -or
                [string]::IsNullOrWhiteSpace([string]$semanticFailure.failure_reason) -or
                @($semanticFailure.raw_evidence).Count -lt 1) {
                throw "Nested failed UAT semantic assertions require exact identity, reason, evidence, parent, and allowed classification."
            }
            foreach ($evidence in @($semanticFailure.raw_evidence)) {
                Assert-HashedFileEvidence -Evidence $evidence -Label "Nested semantic failure '$expectedName' raw evidence" -SkipFileEvidence:$SkipFileEvidence
            }
            $nestedFailed.Add($semanticFailure)
        }
    }
    $nestedNames = @($nestedBlocked | ForEach-Object { [string]$_.name })
    if (@($nestedNames | Sort-Object -Unique).Count -ne $nestedNames.Count) {
        throw "Nested blocked UAT scenario identities must be unique."
    }
    $nestedFailedNames = @($nestedFailed | ForEach-Object { [string]$_.name })
    if (@($nestedFailedNames | Sort-Object -Unique).Count -ne $nestedFailedNames.Count) {
        throw "Nested failed UAT semantic assertion identities must be unique."
    }
    foreach ($field in @("assertion_count", "failure_count", "blocked_count", "nested_blocked_count", "nested_failure_count")) {
        if ($Attempt.PSObject.Properties.Name -notcontains $field) {
            throw "Attempt receipt omits exact '$field' accounting."
        }
    }
    foreach ($field in @("failed_count", "blocked_count", "passed_count", "nested_blocked_count", "nested_failed_count")) {
        if ($Harvest.PSObject.Properties.Name -notcontains $field) {
            throw "Harvest receipt omits exact '$field' accounting."
        }
    }
    if ($attemptSchema -eq 3) {
        foreach ($document in @($Attempt, $Harvest, $Batch)) {
            if ($document.PSObject.Properties.Name -notcontains "deferred_count" -or
                ($document.deferred_count -isnot [int] -and $document.deferred_count -isnot [long])) {
                throw "Schema-3 Candidate Rehearsal accounting requires one exact integer deferred_count in attempt, harvest, and batch."
            }
        }
    } elseif ($deferredTerminal.Count -ne 0) {
        throw "Historical schema-2 Candidate Rehearsal attempts cannot acquire deferred lane semantics."
    }
    if ([int]$Attempt.assertion_count -ne $assertionBearingTerminal.Count -or
        [int]$Attempt.failure_count -ne $failedTerminal.Count -or
        [int]$Attempt.blocked_count -ne $blockedTerminal.Count -or
        [int]$Attempt.nested_blocked_count -ne $nestedBlocked.Count -or
        [int]$Attempt.nested_failure_count -ne $nestedFailed.Count -or
        [int]$Harvest.failed_count -ne $failedTerminal.Count -or
        [int]$Harvest.blocked_count -ne $blockedTerminal.Count -or
        [int]$Harvest.passed_count -ne $passedTerminal.Count -or
        [int]$Harvest.nested_blocked_count -ne $nestedBlocked.Count -or
        [int]$Harvest.nested_failed_count -ne $nestedFailed.Count -or
        ($attemptSchema -eq 3 -and (
            [int]$Attempt.deferred_count -ne $deferredTerminal.Count -or
            [int]$Harvest.deferred_count -ne $deferredTerminal.Count -or
            [int]$Batch.deferred_count -ne $deferredTerminal.Count))) {
        throw "Attempt/harvest pass, fail, block, deferred, or nested counts do not match terminal evidence."
    }

    if ([int]$Batch.batch -ne 1 -or [string]$Batch.state -cne "open" -or
        [string]$Batch.harvest_receipt.path -cne [string]$Harvest.receipt_path -or
        [string]$Batch.harvest_receipt.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "The diagnostic pass must produce exactly one open batch bound to its harvest receipt and digest."
    }
    Assert-HashedFileEvidence -Evidence $Batch.harvest_receipt -Label "Defect-batch harvest receipt" -SkipFileEvidence:$SkipFileEvidence
    if ($attemptSchema -eq 3) {
        if ($Batch.PSObject.Properties.Name -notcontains "deferred_checks") {
            throw "The schema-2 consolidated batch omits its separate deferred-check inventory."
        }
        $batchDeferred = @($Batch.deferred_checks)
        if ($batchDeferred.Count -ne $deferredTerminal.Count -or
            (($batchDeferred | ConvertTo-Json -Depth 50 -Compress) -cne
                ($deferredTerminal | ConvertTo-Json -Depth 50 -Compress))) {
            throw "The consolidated batch deferred inventory is not exact or is mixed into defect evidence."
        }
    }
    if ($Batch.PSObject.Properties.Name -notcontains "blocked_checks") {
        throw "The consolidated batch omits its exact blocked-check inventory."
    }
    $expectedBlockedKeys = @(
        @($blockedTerminal | ForEach-Object {
            "lane|$([string]$_.name)||$([string]$_.dependency_reason)"
        }) + @($nestedBlocked | ForEach-Object {
            $blockedBy = @($_.blocked_by | ForEach-Object { [string]$_ } | Sort-Object) -join ','
            "scenario|$([string]$_.name)|$([string]$_.parent_check)|$blockedBy|$([string]$_.dependency_reason)"
        }) | Sort-Object
    )
    $actualBlockedKeys = @($Batch.blocked_checks | ForEach-Object {
        $scope = [string]$_.scope
        $name = [string]$_.name
        $parent = [string]$_.parent_check
        $blockedBy = @(if ($_.PSObject.Properties.Name -contains "blocked_by") {
            $_.blocked_by | ForEach-Object { [string]$_ } | Sort-Object
        })
        $reason = [string]$_.dependency_reason
        if (@("lane", "scenario") -cnotcontains $scope -or
            [string]::IsNullOrWhiteSpace($name) -or
            [string]::IsNullOrWhiteSpace($reason) -or
            ($scope -ceq "lane" -and -not [string]::IsNullOrWhiteSpace($parent)) -or
            ($scope -ceq "scenario" -and ($parent -cne "uat-diagnostics" -or $blockedBy.Count -eq 0))) {
            throw "The consolidated batch contains an invalid blocked-check record."
        }
        if ($scope -ceq "scenario") {
            "$scope|$name|$parent|$($blockedBy -join ',')|$reason"
        } else {
            "$scope|$name|$parent|$reason"
        }
    } | Sort-Object)
    if (($expectedBlockedKeys -join "`n") -cne ($actualBlockedKeys -join "`n")) {
        throw "The consolidated batch blocked-check inventory does not match terminal lane and nested UAT evidence."
    }
    $defects = @($Batch.defects)
    if ([int]$Batch.defect_count -ne $defects.Count -or $defects.Count -eq 0) {
        throw "The consolidated defect count must equal its nonempty defect inventory."
    }
    $defectIds = @($defects | ForEach-Object { [string]$_.id })
    if (@($defectIds | Sort-Object -Unique).Count -ne $defectIds.Count -or
        @($defectIds | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0) {
        throw "The consolidated batch contains missing or duplicate defect identities."
    }
    foreach ($defect in $defects) {
        $checkNames = @($defect.check_names)
        $nestedAssertion = if ($defect.PSObject.Properties.Name -contains "nested_assertion") { [string]$defect.nested_assertion } else { "" }
        if ($allowedClassifications -cnotcontains [string]$defect.classification -or
            [string]::IsNullOrWhiteSpace([string]$defect.summary) -or
            (($checkNames.Count -eq 0) -eq [string]::IsNullOrWhiteSpace($nestedAssertion))) {
            throw "Defect '$($defect.id)' must bind exactly one failed lane set or one nested semantic assertion."
        }
        foreach ($checkName in $checkNames) {
            $result = @($terminal | Where-Object name -CEQ ([string]$checkName))
            if ($result.Count -ne 1 -or [string]$result[0].state -cne "failed") {
                throw "Defect '$($defect.id)' binds nonfailed or unknown check '$checkName'."
            }
        }
        $expectedRawEvidence = @($checkNames | ForEach-Object {
            $result = @($terminal | Where-Object name -CEQ ([string]$_))[0]
            [pscustomobject]@{
                path = [string]$result.evidence_path
                sha256 = [string]$result.evidence_sha256
            }
            if ($result.PSObject.Properties.Name -contains "produced_evidence") {
                @($result.produced_evidence | Where-Object { $null -ne $_ })
            }
        })
        if (-not [string]::IsNullOrWhiteSpace($nestedAssertion)) {
            $nestedResult = @($nestedFailed | Where-Object name -CEQ $nestedAssertion)
            if ($nestedResult.Count -ne 1 -or
                [string]$nestedResult[0].classification -cne [string]$defect.classification -or
                [string]$nestedResult[0].failure_reason -cne [string]$defect.summary) {
                throw "Defect '$($defect.id)' does not bind the exact nested failed semantic assertion."
            }
            $expectedRawEvidence = @($nestedResult[0].raw_evidence)
        }
        $actualRawEvidence = @($defect.raw_evidence)
        $expectedKeys = @($expectedRawEvidence | ForEach-Object { "$([string]$_.path)|$([string]$_.sha256)" } | Sort-Object -Unique)
        $actualKeys = @($actualRawEvidence | ForEach-Object {
            Assert-HashedFileEvidence -Evidence $_ -Label "Defect '$($defect.id)' raw evidence" -SkipFileEvidence:$SkipFileEvidence
            "$([string]$_.path)|$([string]$_.sha256)"
        } | Sort-Object -Unique)
        if (($expectedKeys | ConvertTo-Json -Compress) -cne ($actualKeys | ConvertTo-Json -Compress)) {
            throw "Defect '$($defect.id)' does not bind every retained raw artifact from its failed checks."
        }
    }
    foreach ($failed in @($terminal | Where-Object state -CEQ "failed")) {
        if (@($defects | Where-Object { @($_.check_names) -ccontains [string]$failed.name }).Count -eq 0) {
            throw "Failed check '$($failed.name)' is absent from the consolidated defect batch."
        }
    }
    foreach ($nestedFailure in $nestedFailed) {
        if (@($defects | Where-Object { [string]$_.nested_assertion -ceq [string]$nestedFailure.name }).Count -ne 1) {
            throw "Nested semantic failure '$($nestedFailure.name)' is absent from or duplicated in the consolidated defect batch."
        }
    }
    if ($defects.Count -ne ($failedTerminal.Count + $nestedFailed.Count)) {
        throw "The consolidated batch double counts or omits lane and nested semantic failures."
    }
}

function Assert-ReadinessHarvestComplete {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Harvest,
        [Parameter(Mandatory)]$Batch,
        [switch]$SkipFileEvidence
    )

    $script:StrictCanonicalEvidencePaths = $false
    Assert-DiagnosticReceiptHeader -Document $Attempt -SchemaVersions @(2, 3) -Phase "validation-readiness" -Label "Readiness attempt"
    Assert-DiagnosticReceiptHeader -Document $Harvest -SchemaVersions @(1) -Phase "validation-readiness-harvest" -Label "Readiness harvest"
    Assert-DiagnosticReceiptHeader -Document $Batch -SchemaVersions @(1) -Phase "validation-readiness-defect-batch" -Label "Readiness defect batch"
    Assert-MutableSourceIdentity `
        -Source $Attempt.mutable_source_identity `
        -VerificationState ([string]$Attempt.source_identity_verification_state)
    Assert-EnvironmentFingerprint -Document $Attempt -Label "Readiness attempt"
    Assert-EnvironmentFingerprint -Document $Harvest -Label "Readiness harvest"
    Assert-EnvironmentFingerprint -Document $Batch -Label "Readiness defect batch"
    if ([string]$Attempt.state -cne "failed" -or -not [bool]$Attempt.assertions_started -or
        [string]::IsNullOrWhiteSpace([string]$Attempt.assertions_started_at) -or
        [string]$Harvest.state -cne "harvest_complete") {
        throw "Only one assertion-bearing terminal failed Readiness may authorize correction."
    }
    foreach ($document in @($Harvest, $Batch)) {
        if ([int]$document.attempt -ne [int]$Attempt.attempt -or
            [string]$document.sprint -cne [string]$Attempt.sprint -or
            (($document.mutable_source_identity | ConvertTo-Json -Depth 30 -Compress) -cne
                ($Attempt.mutable_source_identity | ConvertTo-Json -Depth 30 -Compress)) -or
            [string]$document.environment_fingerprint -cne [string]$Attempt.environment_fingerprint) {
            throw "Readiness harvest and batch must bind the exact failed attempt identity."
        }
    }
    Assert-HashedFileEvidence -Evidence $Harvest.attempt_receipt `
        -Label "Readiness harvest attempt receipt" `
        -SkipFileEvidence:$SkipFileEvidence
    Assert-HashedFileEvidence -Evidence $Batch.harvest_receipt `
        -Label "Readiness batch harvest receipt" `
        -SkipFileEvidence:$SkipFileEvidence

    $declared = @($Attempt.declared_checks)
    $terminal = @($Attempt.checks)
    $harvestTerminal = @($Harvest.checks)
    Assert-AcyclicCheckGraph -Checks $declared
    Assert-Sprint8ADeclaredEvidencePaths `
        -Checks $declared `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Label "Validation Readiness attempt declarations"
    if ($terminal.Count -ne $declared.Count -or $harvestTerminal.Count -ne $declared.Count -or
        (($terminal | ConvertTo-Json -Depth 30 -Compress) -cne
            ($harvestTerminal | ConvertTo-Json -Depth 30 -Compress))) {
        throw "Readiness attempt and harvest do not retain the exact complete terminal check inventory."
    }
    foreach ($check in $declared) {
        $result = @($terminal | Where-Object name -CEQ ([string]$check.name))
        if ($result.Count -ne 1) { throw "Readiness check '$($check.name)' is not terminal exactly once." }
        Assert-TerminalCheckEvidence `
            -Declared $check `
            -Result $result[0] `
            -TerminalChecks $terminal `
            -CurrentAttempt ([int]$Attempt.attempt) `
            -SkipFileEvidence:$SkipFileEvidence
    }
    $failed = @($terminal | Where-Object state -CEQ "failed")
    $blocked = @($terminal | Where-Object state -CEQ "blocked")
    $passed = @($terminal | Where-Object state -CEQ "passed")
    $assertionBearing = @($terminal | Where-Object assertions_started -EQ $true)
    if ($failed.Count -eq 0 -or
        [int]$Attempt.assertion_count -ne $assertionBearing.Count -or
        [int]$Attempt.failure_count -ne $failed.Count -or
        [int]$Attempt.blocked_count -ne $blocked.Count -or
        [int]$Harvest.failed_count -ne $failed.Count -or
        [int]$Harvest.blocked_count -ne $blocked.Count -or
        [int]$Harvest.passed_count -ne $passed.Count) {
        throw "Readiness attempt/harvest pass, fail, block, or assertion counts are not exact."
    }
    if ([string]$Attempt.environment_fingerprint -ceq ("0" * 64)) {
        $environmentResult = @($terminal | Where-Object name -CEQ "environment-contract")
        if ($environmentResult.Count -ne 1 -or
            @("failed", "blocked") -cnotcontains [string]$environmentResult[0].state) {
            throw "A zero Readiness environment fingerprint requires one exact nonpassing environment-contract result."
        }
    }

    if ([int]$Batch.batch -ne 1 -or [string]$Batch.state -cne "open" -or
        [string]$Batch.harvest_receipt.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Failed Readiness must produce exactly one open defect batch bound to its harvest."
    }
    $defects = @($Batch.defects)
    if ([int]$Batch.defect_count -ne $defects.Count -or $defects.Count -ne $failed.Count) {
        throw "Readiness defect batch must record every failed check exactly once."
    }
    foreach ($failedCheck in $failed) {
        $matching = @($defects | Where-Object {
            @($_.check_names).Count -eq 1 -and
            [string]$_.check_names[0] -ceq [string]$failedCheck.name
        })
        if ($matching.Count -ne 1 -or
            [string]$matching[0].classification -cne [string]$failedCheck.classification -or
            [string]::IsNullOrWhiteSpace([string]$matching[0].summary)) {
            throw "Failed Readiness check '$($failedCheck.name)' is absent from or duplicated in its batch."
        }
        $expectedRaw = @([pscustomobject]@{
            path = [string]$failedCheck.evidence_path
            sha256 = [string]$failedCheck.evidence_sha256
        }) + @($failedCheck.produced_evidence | Where-Object { $null -ne $_ })
        $expectedKeys = @($expectedRaw | ForEach-Object { "$([string]$_.path)|$([string]$_.sha256)" } | Sort-Object -Unique)
        $actualKeys = @($matching[0].raw_evidence | ForEach-Object {
            Assert-HashedFileEvidence -Evidence $_ -Label "Readiness defect raw evidence" -SkipFileEvidence:$SkipFileEvidence
            "$([string]$_.path)|$([string]$_.sha256)"
        } | Sort-Object -Unique)
        if (($expectedKeys -join "`n") -cne ($actualKeys -join "`n")) {
            throw "Readiness defect '$($matching[0].id)' drops or adds raw evidence."
        }
    }
    $expectedBlocked = @($blocked | ForEach-Object {
        "$([string]$_.name)|$([string]$_.dependency_reason)"
    } | Sort-Object)
    $actualBlocked = @($Batch.blocked_checks | ForEach-Object {
        if ([string]$_.scope -cne "check" -or
            [string]::IsNullOrWhiteSpace([string]$_.dependency_reason)) {
            throw "Readiness defect batch contains an invalid blocked-check record."
        }
        "$([string]$_.name)|$([string]$_.dependency_reason)"
    } | Sort-Object)
    if (($expectedBlocked -join "`n") -cne ($actualBlocked -join "`n")) {
        throw "Readiness defect batch does not retain every exact blocked-check reason."
    }
}

function Assert-HarvestComplete {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Harvest,
        [Parameter(Mandatory)]$Batch,
        [switch]$SkipFileEvidence
    )

    switch ([string]$Attempt.phase) {
        "candidate-rehearsal" {
            Assert-CandidateRehearsalHarvestComplete @PSBoundParameters
        }
        "validation-readiness" {
            Assert-ReadinessHarvestComplete @PSBoundParameters
        }
        default { throw "Unsupported validation harvest predecessor phase '$($Attempt.phase)'." }
    }
}

function Assert-CorrectionAuthorizationEligibility {
    param(
        [Parameter(Mandatory)]$Attempt,
        [switch]$SkipFileEvidence
    )

    if ([string]$Attempt.phase -ceq "validation-readiness") {
        if ($Attempt.PSObject.Properties.Name -notcontains "cleanup_restoration" -or
            $null -eq $Attempt.cleanup_restoration -or
            $Attempt.cleanup_restoration.required -ne $false -or
            [string]$Attempt.cleanup_restoration.result -cne "not_applicable") {
            throw "Validation Readiness correction authority requires exact not-applicable cleanup semantics."
        }
        return [pscustomobject][ordered]@{
            required = $false
            result = "not_applicable"
            evidence = @()
        }
    }
    if ([string]$Attempt.phase -cne "candidate-rehearsal") {
        throw "Correction authorization eligibility does not support phase '$([string]$Attempt.phase)'."
    }
    if ($Attempt.PSObject.Properties.Name -notcontains "cleanup_restoration" -or
        $null -eq $Attempt.cleanup_restoration -or
        $Attempt.cleanup_restoration.required -isnot [bool] -or
        $Attempt.cleanup_restoration.required -ne $true -or
        [string]$Attempt.cleanup_restoration.result -cne "canonical_successor_healthy") {
        throw "Candidate Rehearsal harvest remains retainable, but correction authority is forbidden until canonical cleanup/restoration is proven."
    }

    $requiredLanes = @("final-successor-health", "final-environment-identity")
    $evidence = [Collections.Generic.List[object]]::new()
    foreach ($laneName in $requiredLanes) {
        $result = @($Attempt.checks | Where-Object name -CEQ $laneName)
        if ($result.Count -ne 1 -or
            [string]$result[0].state -cne "passed" -or
            $result[0].assertions_started -isnot [bool] -or
            $result[0].assertions_started -ne $true -or
            $result[0].PSObject.Properties.Name -notcontains "lane_receipt" -or
            $null -eq $result[0].lane_receipt) {
            throw "Candidate Rehearsal correction authority requires exact passing current-attempt proof for cleanup lane '$laneName'."
        }
        [void](Assert-Sprint8ACanonicalEvidencePath `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$result[0].lane_receipt.path) `
            -Label "Cleanup lane '$laneName' receipt")
        Assert-HashedFileEvidence `
            -Evidence $result[0].lane_receipt `
            -Label "Cleanup lane '$laneName' receipt" `
            -SkipFileEvidence:$SkipFileEvidence
        if (-not $SkipFileEvidence) {
            $reference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$result[0].lane_receipt.path)
            $sidecarSha = Assert-Sprint8AReceiptSidecar -Path ([string]$reference.full_path)
            $laneDocument = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
            if ($sidecarSha -cne [string]$result[0].lane_receipt.sha256 -or
                [string]$laneDocument.sprint -cne "sprint-8a" -or
                [string]$laneDocument.phase -cne "candidate-rehearsal-lane" -or
                [int]$laneDocument.attempt -ne [int]$Attempt.attempt -or
                $laneDocument.authoritative -isnot [bool] -or
                $laneDocument.authoritative -ne $false -or
                [string]$laneDocument.result.name -cne $laneName -or
                [string]$laneDocument.result.state -cne "passed" -or
                $laneDocument.result.assertions_started -ne $true) {
                throw "Cleanup lane '$laneName' receipt does not prove that exact current-attempt lane passed."
            }
        }
        $evidence.Add([ordered]@{
            lane = $laneName
            path = [string]$result[0].lane_receipt.path
            sha256 = [string]$result[0].lane_receipt.sha256
        })
    }

    [pscustomobject][ordered]@{
        required = $true
        result = "canonical_successor_healthy"
        evidence = @($evidence)
    }
}

function Invoke-ExpectedGuardFailure {
    param([scriptblock]$Action, [string]$Label)
    try {
        & $Action
        throw "Self-test failed: $Label was accepted."
    } catch {
        if ($_.Exception.Message -like "Self-test failed:*") { throw }
    }
}

if ($SelfTest) {
    Test-Sprint8AEvidenceReferenceResolution | Out-Null
    $source = [pscustomobject]@{ commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"; acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64 }
    $attempt = [pscustomobject]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal"; authoritative = $false
        attempt = 1; state = "harvesting"; assertions_started = $true
        assertions_started_at = "2026-01-01T00:00:00Z"; mutable_source_identity = $source; environment_fingerprint = "e" * 64
        source_identity_verification_state = "verified"
        environment_identity = [pscustomobject]@{ verification_state = "verified" }
        assertion_count = 2; failure_count = 1; blocked_count = 1; nested_blocked_count = 1; nested_failure_count = 1
        declared_checks = @(
            [pscustomobject]@{ name = "independent"; depends_on = @(); command = "fail" },
            [pscustomobject]@{ name = "uat-diagnostics"; depends_on = @(); command = "pass" },
            [pscustomobject]@{ name = "dependent"; depends_on = @("independent"); command = "blocked" }
        )
        checks = @()
        cleanup_restoration = [pscustomobject]@{ required = $true; result = "not_proven" }
    }
    $harvest = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-harvest"; authoritative = $false
        attempt = 1; state = "harvest_complete"; receipt_path = "attempts/harvest.json"
        mutable_source_identity = $source; environment_fingerprint = "e" * 64
        attempt_receipt = [pscustomobject]@{ path = "attempts/attempt.json"; sha256 = "d" * 64 }
        failed_count = 1; blocked_count = 1; passed_count = 1; nested_blocked_count = 1; nested_failed_count = 1
        checks = @(
            [pscustomobject]@{ name = "independent"; command = "fail"; state = "failed"; classification = "harness"; dependency_reason = $null; started_at = "2026-01-01T00:00:01Z"; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:01Z"; duration_ms = 1000; exit_status = 1; evidence_path = "raw/fail.log"; evidence_sha256 = "f" * 64; produced_evidence = @(); nested_blocked_checks = @(); nested_failed_checks = @() },
            [pscustomobject]@{ name = "uat-diagnostics"; command = "pass"; state = "passed"; classification = $null; dependency_reason = $null; started_at = "2026-01-01T00:00:01Z"; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:01Z"; duration_ms = 1000; exit_status = 0; evidence_path = "raw/pass.log"; evidence_sha256 = "a" * 64; produced_evidence = @(); nested_blocked_checks = @([pscustomobject]@{ name = "uat-diagnostics/UAT-8A-01"; parent_check = "uat-diagnostics"; blocked_by = @("independent"); dependency_reason = "blocked by invalid prerequisite(s): independent" }); nested_failed_checks = @([pscustomobject]@{ name = "uat-diagnostics/UAT-8A-02/semantic-proof"; parent_check = "uat-diagnostics"; scenario = "UAT-8A-02"; assertion_id = "semantic-proof"; classification = "product"; failure_reason = "semantic mismatch"; raw_evidence = @([pscustomobject]@{ path = "raw/semantic.json"; sha256 = "b" * 64 }) }) },
            [pscustomobject]@{ name = "dependent"; command = "blocked"; state = "blocked"; classification = "harness"; dependency_reason = "independent failed"; started_at = $null; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $false; assertions_started_at = $null; duration_ms = 0; exit_status = $null; evidence_path = $null; evidence_sha256 = $null; nested_blocked_checks = @(); nested_failed_checks = @() }
        )
    }
    $batch = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-defect-batch"; authoritative = $false
        attempt = 1; batch = 1; state = "open"; mutable_source_identity = $source; environment_fingerprint = "e" * 64
        harvest_receipt = [pscustomobject]@{ path = "attempts/harvest.json"; sha256 = "e" * 64 }
        defect_count = 2; defects = @(
            [pscustomobject]@{ id = "R1"; classification = "harness"; summary = "failure"; check_names = @("independent"); nested_assertion = $null; raw_evidence = @([pscustomobject]@{ path = "raw/fail.log"; sha256 = "f" * 64 }) },
            [pscustomobject]@{ id = "R2"; classification = "product"; summary = "semantic mismatch"; check_names = @(); nested_assertion = "uat-diagnostics/UAT-8A-02/semantic-proof"; raw_evidence = @([pscustomobject]@{ path = "raw/semantic.json"; sha256 = "b" * 64 }) }
        )
        blocked_checks = @(
            [pscustomobject]@{ scope = "lane"; name = "dependent"; parent_check = $null; dependency_reason = "independent failed" },
            [pscustomobject]@{ scope = "scenario"; name = "uat-diagnostics/UAT-8A-01"; parent_check = "uat-diagnostics"; blocked_by = @("independent"); dependency_reason = "blocked by invalid prerequisite(s): independent" }
        )
    }
    $attempt.checks = @($harvest.checks | ConvertTo-Json -Depth 30 | ConvertFrom-Json)
    Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence
    $invalidExecutedDependency = $harvest.checks[1] | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $invalidExecutedDependency.name = "dependent"
    $invalidExecutedDependency.command = "blocked"
    $invalidExecutedDependency.state = "passed"
    $invalidExecutedDependency.classification = $null
    $invalidExecutedDependency.dependency_reason = $null
    $invalidExecutedDependency.started_at = "2026-01-01T00:00:01Z"
    $invalidExecutedDependency.assertions_started = $true
    $invalidExecutedDependency.assertions_started_at = "2026-01-01T00:00:01Z"
    $invalidExecutedDependency.duration_ms = 1000
    $invalidExecutedDependency.exit_status = 0
    $invalidExecutedDependency.evidence_path = "raw/dependent.log"
    $invalidExecutedDependency.evidence_sha256 = "7" * 64
    $invalidExecutedDependency.produced_evidence = @()
    $invalidExecutedDependency.nested_blocked_checks = @()
    $invalidExecutedDependency.nested_failed_checks = @()
    $invalidDependencyTerminal = @($harvest.checks[0], $harvest.checks[1], $invalidExecutedDependency)
    try {
        Assert-TerminalCheckEvidence `
            -Declared $attempt.declared_checks[2] `
            -Result $invalidExecutedDependency `
            -TerminalChecks $invalidDependencyTerminal `
            -CurrentAttempt 1 `
            -SkipFileEvidence
        throw "Self-test failed: an executed lane with a nonpassing prerequisite was accepted."
    } catch {
        if ($_.Exception.Message -ceq
            "Self-test failed: an executed lane with a nonpassing prerequisite was accepted.") {
            throw
        }
        if ($_.Exception.Message -notlike
            "Executed check 'dependent' has nonpassing declared prerequisite(s): independent.*") {
            throw "Harvest guard rejected impossible terminal dependency state for the wrong reason: $($_.Exception.Message)"
        }
    }
    Invoke-ExpectedGuardFailure {
        Assert-CorrectionAuthorizationEligibility -Attempt $attempt -SkipFileEvidence
    } "a complete harvest without canonical cleanup proof authorizing correction"

    $eligibleAttempt = [pscustomobject]@{
        phase = "candidate-rehearsal"
        attempt = 33
        cleanup_restoration = [pscustomobject]@{
            required = $true
            result = "canonical_successor_healthy"
        }
        checks = @(
            [pscustomobject]@{
                name = "final-successor-health"; state = "passed"; assertions_started = $true
                lane_receipt = [pscustomobject]@{
                    path = "artifacts/sprint-8a-closeout/rehearsal/attempt-33/lanes/final-successor-health.json"
                    sha256 = "1" * 64
                }
            },
            [pscustomobject]@{
                name = "final-environment-identity"; state = "passed"; assertions_started = $true
                lane_receipt = [pscustomobject]@{
                    path = "artifacts/sprint-8a-closeout/rehearsal/attempt-33/lanes/final-environment-identity.json"
                    sha256 = "2" * 64
                }
            }
        )
    }
    $cleanupEligibility = Assert-CorrectionAuthorizationEligibility `
        -Attempt $eligibleAttempt `
        -SkipFileEvidence
    if ($cleanupEligibility.result -cne "canonical_successor_healthy" -or
        @($cleanupEligibility.evidence).Count -ne 2) {
        throw "Correction cleanup eligibility self-test did not return both exact mandatory lane bindings."
    }
    $missingCleanupLane = $eligibleAttempt | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $missingCleanupLane.checks = @($missingCleanupLane.checks | Where-Object name -CNE "final-successor-health")
    Invoke-ExpectedGuardFailure {
        Assert-CorrectionAuthorizationEligibility -Attempt $missingCleanupLane -SkipFileEvidence
    } "cleanup authority with one mandatory lane missing"
    $rootedCleanupLane = $eligibleAttempt | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $rootedCleanupLane.checks[0].lane_receipt.path = Join-Path $repoRoot "artifacts/sprint-8a-closeout/rehearsal/attempt-33/lanes/final-successor-health.json"
    Invoke-ExpectedGuardFailure {
        Assert-CorrectionAuthorizationEligibility -Attempt $rootedCleanupLane -SkipFileEvidence
    } "cleanup authority with a rooted emitted lane receipt"

    $cleanupAuthorizationSelfTestRoot = Join-Path $repoRoot "artifacts/sprint-8a-cleanup-authorization-selftest-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory((Join-Path $cleanupAuthorizationSelfTestRoot "lanes")) | Out-Null
    try {
        $cleanupBindings = @(
            "final-successor-health",
            "final-environment-identity"
        ) | ForEach-Object {
            $laneName = [string]$_
            $lanePath = Join-Path $cleanupAuthorizationSelfTestRoot "lanes/$laneName.json"
            $laneReference = Publish-Sprint7AEvidence -Document ([ordered]@{
                schema_version = 2
                sprint = "sprint-8a"
                phase = "candidate-rehearsal-lane"
                attempt = 33
                authoritative = $false
                result = [ordered]@{
                    name = $laneName
                    state = "passed"
                    assertions_started = $true
                }
            }) -OutputPath $lanePath
            [ordered]@{
                lane = $laneName
                path = [IO.Path]::GetRelativePath($repoRoot, $lanePath).Replace("\", "/")
                sha256 = [string]$laneReference.sha256
            }
        }
        $schema3Authorization = [pscustomobject][ordered]@{
            schema_version = 3
            cleanup_restoration = [pscustomobject][ordered]@{
                required = $true
                result = "canonical_successor_healthy"
                evidence = @($cleanupBindings)
            }
        }
        $cleanupValidation = Assert-Sprint8ACandidateCorrectionAuthorizationCleanup `
            -AuthorizationDocument $schema3Authorization `
            -Attempt 33 `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $cleanupAuthorizationSelfTestRoot
        if (@($cleanupValidation.references).Count -ne 2) {
            throw "Schema-3 cleanup authorization self-test did not authenticate both lane receipts."
        }
        $tamperedAuthorization = $schema3Authorization | ConvertTo-Json -Depth 20 | ConvertFrom-Json
        $tamperedAuthorization.cleanup_restoration.evidence[0].sha256 = "0" * 64
        Invoke-ExpectedGuardFailure {
            Assert-Sprint8ACandidateCorrectionAuthorizationCleanup `
                -AuthorizationDocument $tamperedAuthorization `
                -Attempt 33 `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $cleanupAuthorizationSelfTestRoot
        } "a schema-3 cleanup authorization with stale lane evidence"
    } finally {
        $artifactsRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot "artifacts"))
        $resolvedSelfTestRoot = [IO.Path]::GetFullPath($cleanupAuthorizationSelfTestRoot)
        if (-not $resolvedSelfTestRoot.StartsWith("$artifactsRoot$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::OrdinalIgnoreCase)) {
            throw "Cleanup authorization self-test root escaped the repository artifacts directory."
        }
        if (Test-Path -LiteralPath $resolvedSelfTestRoot -PathType Container) {
            [IO.Directory]::Delete($resolvedSelfTestRoot, $true)
        }
    }

    $deferredAttempt = $attempt | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $deferredHarvest = $harvest | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $deferredBatch = $batch | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $deferredAttempt.schema_version = 3
    $deferredAttempt.state = "incomplete"
    $deferredHarvest.schema_version = 2
    $deferredBatch.schema_version = 2
    $syntheticStartReference = [pscustomobject][ordered]@{
        path = "attempts/candidate-rehearsal-1-start.json"
        sha256 = "5" * 64
    }
    $deferredAttempt | Add-Member -NotePropertyName immutable_start_receipt -NotePropertyValue $syntheticStartReference
    $deferredAttempt | Add-Member -NotePropertyName schedule_sha256 -NotePropertyValue ("7" * 64)
    $deferredHarvest | Add-Member -NotePropertyName immutable_start_receipt -NotePropertyValue $syntheticStartReference
    $deferredHarvest | Add-Member -NotePropertyName schedule_sha256 -NotePropertyValue ("7" * 64)
    $deferredDeclaration = [pscustomobject]@{
        name = "prior-pass"; depends_on = @(); command = "deferred prior passing lane"
    }
    $deferredResult = [pscustomobject][ordered]@{
        name = "prior-pass"
        depends_on = @()
        command = "deferred prior passing lane"
        wave = "B"
        scheduling_reason = "prior_passing_outside_impact"
        started_at = $null
        ended_at = $null
        duration_ms = $null
        exit_status = $null
        assertions_started = $false
        assertions_started_at = $null
        state = "deferred"
        classification = $null
        classification_source = $null
        dependency_reason = "Wave B was deferred after Wave A failed; this lane executed no assertions."
        prior_passing_receipt = [pscustomobject]@{
            path = "attempts/candidate-rehearsal-0-prior-pass-lane.json"
            sha256 = "6" * 64
        }
        prior_source_identity = $source
        prior_environment_identity = [pscustomobject]@{ fingerprint = "e" * 64 }
        current_correction_impact = [pscustomobject]@{
            affected = $false
            decision = "outside_correction_impact"
            matched_paths = @()
            matched_identity_sources = @()
            rationale = "No authenticated changed input intersects this lane."
        }
        non_impact_rationale = "No authenticated changed input intersects this lane."
        consecutive_deferral_count = 1
        mandatory_by_attempt = 4
        prerequisite_state = @()
        diagnostic_history_notice = $script:Sprint8ADiagnosticHistoryNotice
        evidence_path = $null
        evidence_sha256 = $null
        produced_evidence = @()
        failure_message = $null
        nested_blocked_checks = @()
        nested_failed_checks = @()
    }
    $deferredAttempt.declared_checks = @($deferredAttempt.declared_checks) + @($deferredDeclaration)
    $deferredAttempt.checks = @($deferredAttempt.checks) + @($deferredResult)
    $deferredHarvest.checks = @($deferredHarvest.checks) + @($deferredResult)
    $deferredAttempt | Add-Member -NotePropertyName deferred_count -NotePropertyValue 1
    $deferredHarvest | Add-Member -NotePropertyName deferred_count -NotePropertyValue 1
    $deferredBatch | Add-Member -NotePropertyName deferred_count -NotePropertyValue 1
    $deferredBatch | Add-Member -NotePropertyName deferred_checks -NotePropertyValue @($deferredResult)
    Assert-HarvestComplete `
        -Attempt $deferredAttempt `
        -Harvest $deferredHarvest `
        -Batch $deferredBatch `
        -SkipFileEvidence
    $deferredSchedule = [pscustomobject][ordered]@{
        decisions = @([pscustomobject][ordered]@{
            name = "prior-pass"
            segment = "wave_b"
            consecutive_deferrals_before = 0
        })
    }
    Assert-Sprint8ARehearsalDeferredCounterBinding `
        -Schedule $deferredSchedule `
        -TerminalChecks @($deferredResult)
    $detachedDeferredCounter = $deferredResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $detachedDeferredCounter.consecutive_deferral_count = 2
    $detachedDeferredCounter.mandatory_by_attempt = 3
    Invoke-ExpectedGuardFailure {
        Assert-Sprint8ARehearsalDeferredCounterBinding `
            -Schedule $deferredSchedule `
            -TerminalChecks @($detachedDeferredCounter)
    } "a deferred counter detached from its immutable-start counter"
    $deferredAuthorization = [pscustomobject][ordered]@{ deferred_count = 1 }
    Assert-Sprint8ACandidateDeferredCountLineage `
        -PredecessorDocument $deferredAttempt `
        -HarvestDocument $deferredHarvest `
        -BatchDocument $deferredBatch `
        -AuthorizationDocument $deferredAuthorization `
        -Attempt 33
    $pendingDeferredState = [pscustomobject][ordered]@{ deferred_count = 1 }
    Assert-Sprint8APendingCandidateDeferredCountBinding `
        -PredecessorState $pendingDeferredState `
        -AuthorizationDocument $deferredAuthorization `
        -Attempt 33
    $stalePendingDeferredState = [pscustomobject][ordered]@{ deferred_count = 0 }
    Invoke-ExpectedGuardFailure {
        Assert-Sprint8APendingCandidateDeferredCountBinding `
            -PredecessorState $stalePendingDeferredState `
            -AuthorizationDocument $deferredAuthorization `
            -Attempt 33
    } "validation state whose deferred count differs from its pending correction authorization"
    $staleDeferredAuthorization = $deferredAuthorization | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $staleDeferredAuthorization.deferred_count = 0
    Invoke-ExpectedGuardFailure {
        Assert-Sprint8ACandidateDeferredCountLineage `
            -PredecessorDocument $deferredAttempt `
            -HarvestDocument $deferredHarvest `
            -BatchDocument $deferredBatch `
            -AuthorizationDocument $staleDeferredAuthorization `
            -Attempt 33
    } "a correction authorization whose deferred count differs from its predecessor batch"
    $invalidMandatoryDeferred = $deferredResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $invalidMandatoryDeferred.mandatory_by_attempt = 5
    Invoke-ExpectedGuardFailure {
        Assert-DeferredTerminalCheckEvidence `
            -Declared $deferredDeclaration `
            -Result $invalidMandatoryDeferred `
            -TerminalChecks @($invalidMandatoryDeferred) `
            -CurrentAttempt 1 `
            -SkipFileEvidence
    } "a deferred lane with a non-exact mandatory-by-attempt value"

    $staleScheduleHarvest = $deferredHarvest | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $staleScheduleHarvest.schedule_sha256 = "0" * 64
    Invoke-ExpectedGuardFailure {
        Assert-HarvestComplete `
            -Attempt $deferredAttempt `
            -Harvest $staleScheduleHarvest `
            -Batch $deferredBatch `
            -SkipFileEvidence
    } "a schema-3 harvest detached from its immutable start schedule"

    $missingDeferredHarvest = $deferredHarvest | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $missingDeferredHarvest.checks = @($missingDeferredHarvest.checks | Where-Object name -CNE "prior-pass")
    Invoke-ExpectedGuardFailure {
        Assert-HarvestComplete `
            -Attempt $deferredAttempt `
            -Harvest $missingDeferredHarvest `
            -Batch $deferredBatch `
            -SkipFileEvidence
    } "a schema-3 harvest with one declared deferred lane missing"
    $mixedDeferredBatch = $deferredBatch | ConvertTo-Json -Depth 50 | ConvertFrom-Json
    $mixedDeferredBatch.deferred_checks = @()
    Invoke-ExpectedGuardFailure {
        Assert-HarvestComplete `
            -Attempt $deferredAttempt `
            -Harvest $deferredHarvest `
            -Batch $mixedDeferredBatch `
            -SkipFileEvidence
    } "a consolidated batch that omits its separate deferred inventory"

    $priorReceiptSelfTestRoot = Join-Path $repoRoot "artifacts/sprint-8a-harvest-prior-lane-selftest-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($priorReceiptSelfTestRoot) | Out-Null
    $savedEvidenceRoot = $EvidenceRoot
    try {
        $EvidenceRoot = [IO.Path]::GetRelativePath($repoRoot, $priorReceiptSelfTestRoot).Replace("\", "/")
        $priorReceiptPath = Join-Path $priorReceiptSelfTestRoot "candidate-rehearsal-1-prior-pass-lane.json"
        $priorReceiptReference = Publish-Sprint7AEvidence -Document ([pscustomobject][ordered]@{
            schema_version = 2
            sprint = "sprint-8a"
            phase = "candidate-rehearsal-lane"
            attempt = 1
            authoritative = $false
            mutable_source_identity = $source
            environment_fingerprint = "e" * 64
            result = [pscustomobject][ordered]@{
                name = "prior-pass"
                state = "passed"
                assertions_started = $true
            }
        }) -OutputPath $priorReceiptPath
        $deferredResult.prior_passing_receipt = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $priorReceiptPath).Replace("\", "/")
            sha256 = [string]$priorReceiptReference.sha256
        }
        Assert-DeferredLanePriorPassingReceipt -Deferred $deferredResult -CurrentAttempt 2
        $savedPriorSha256 = [string]$deferredResult.prior_passing_receipt.sha256
        $deferredResult.prior_passing_receipt.sha256 = "0" * 64
        Invoke-ExpectedGuardFailure {
            Assert-DeferredLanePriorPassingReceipt -Deferred $deferredResult -CurrentAttempt 2
        } "a deferred lane with a stale prior passing receipt digest"
        $deferredResult.prior_passing_receipt.sha256 = $savedPriorSha256
        $savedPriorEnvironment = [string]$deferredResult.prior_environment_identity.fingerprint
        $deferredResult.prior_environment_identity.fingerprint = "9" * 64
        Invoke-ExpectedGuardFailure {
            Assert-DeferredLanePriorPassingReceipt -Deferred $deferredResult -CurrentAttempt 2
        } "a deferred lane whose prior environment differs from its lane receipt"
        $deferredResult.prior_environment_identity.fingerprint = $savedPriorEnvironment
    } finally {
        $EvidenceRoot = $savedEvidenceRoot
        $artifactsRoot = Join-Path $repoRoot "artifacts"
        $relativeSelfTestRoot = [IO.Path]::GetRelativePath($artifactsRoot, $priorReceiptSelfTestRoot)
        if (-not [IO.Path]::IsPathRooted($relativeSelfTestRoot) -and
            $relativeSelfTestRoot -ne ".." -and
            -not $relativeSelfTestRoot.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal)) {
            [IO.Directory]::Delete($priorReceiptSelfTestRoot, $true)
        }
    }

    $readinessAttempt = [pscustomobject]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "validation-readiness"; authoritative = $false
        attempt = 38; state = "failed"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:00Z"
        mutable_source_identity = $source; source_identity_verification_state = "verified"
        environment_identity = [pscustomobject]@{ verification_state = "unverified" }
        environment_fingerprint = "0" * 64; assertion_count = 2; failure_count = 1; blocked_count = 1
        declared_checks = @(
            [pscustomobject]@{ name = "compose-database-contract"; depends_on = @(); classification = "environment" },
            [pscustomobject]@{ name = "runner-self-tests"; depends_on = @(); classification = "harness" },
            [pscustomobject]@{ name = "environment-contract"; depends_on = @("compose-database-contract"); classification = "environment" }
        )
        checks = @(
            [pscustomobject]@{ name = "compose-database-contract"; depends_on = @(); command = "probe"; state = "failed"; classification = "environment"; dependency_reason = $null; started_at = "2026-01-01T00:00:01Z"; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:01Z"; duration_ms = 1000; exit_status = 1; evidence_path = "raw/readiness-environment.log"; evidence_sha256 = "1" * 64; produced_evidence = $null },
            [pscustomobject]@{ name = "runner-self-tests"; depends_on = @(); command = "selftest"; state = "passed"; classification = $null; dependency_reason = $null; started_at = "2026-01-01T00:00:01Z"; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:01Z"; duration_ms = 1000; exit_status = 0; evidence_path = "raw/readiness-harness.log"; evidence_sha256 = "2" * 64; produced_evidence = @() },
            [pscustomobject]@{ name = "environment-contract"; depends_on = @("compose-database-contract"); command = "finalize"; state = "blocked"; classification = "environment"; dependency_reason = "blocked by failed prerequisite(s): compose-database-contract"; started_at = $null; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $false; assertions_started_at = $null; duration_ms = 0; exit_status = $null; evidence_path = $null; evidence_sha256 = $null; produced_evidence = @() }
        )
    }
    $readinessHarvest = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "validation-readiness-harvest"; authoritative = $false
        attempt = 38; state = "harvest_complete"; mutable_source_identity = $source; environment_fingerprint = "0" * 64
        attempt_receipt = [pscustomobject]@{ path = "attempts/readiness-38.json"; sha256 = "3" * 64 }
        failed_count = 1; blocked_count = 1; passed_count = 1; checks = $readinessAttempt.checks
    }
    $readinessBatch = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "validation-readiness-defect-batch"; authoritative = $false
        attempt = 38; batch = 1; state = "open"; mutable_source_identity = $source; environment_fingerprint = "0" * 64
        harvest_receipt = [pscustomobject]@{ path = "attempts/readiness-38-harvest.json"; sha256 = "4" * 64 }
        defect_count = 1
        defects = @([pscustomobject]@{ id = "8A-VR38-01"; classification = "environment"; summary = "missing binding"; check_names = @("compose-database-contract"); raw_evidence = @([pscustomobject]@{ path = "raw/readiness-environment.log"; sha256 = "1" * 64 }) })
        blocked_checks = @([pscustomobject]@{ scope = "check"; name = "environment-contract"; dependency_reason = "blocked by failed prerequisite(s): compose-database-contract" })
    }
    Assert-HarvestComplete -Attempt $readinessAttempt -Harvest $readinessHarvest -Batch $readinessBatch -SkipFileEvidence
    $schema3ReadinessAttempt = $readinessAttempt | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $schema3ReadinessAttempt.schema_version = 3
    Assert-HarvestComplete `
        -Attempt $schema3ReadinessAttempt `
        -Harvest $readinessHarvest `
        -Batch $readinessBatch `
        -SkipFileEvidence
    $extraReadinessEvidence = [pscustomobject]@{ path = "raw/readiness-extra.json"; sha256 = "9" * 64 }
    $readinessAttempt.checks[0].produced_evidence = @($extraReadinessEvidence)
    $readinessHarvest.checks[0].produced_evidence = @($extraReadinessEvidence)
    $readinessBatch.defects[0].raw_evidence = @(
        [pscustomobject]@{ path = "raw/readiness-environment.log"; sha256 = "1" * 64 },
        $extraReadinessEvidence
    )
    Assert-HarvestComplete -Attempt $readinessAttempt -Harvest $readinessHarvest -Batch $readinessBatch -SkipFileEvidence
    $readinessAttempt.checks[0].produced_evidence = $null
    $readinessHarvest.checks[0].produced_evidence = $null
    $readinessBatch.defects[0].raw_evidence = @([pscustomobject]@{ path = "raw/readiness-environment.log"; sha256 = "1" * 64 })
    $readinessBatch.blocked_checks = @()
    Invoke-ExpectedGuardFailure {
        Assert-HarvestComplete -Attempt $readinessAttempt -Harvest $readinessHarvest -Batch $readinessBatch -SkipFileEvidence
    } "a Readiness batch that drops its blocked dependency"
    $readinessBatch.blocked_checks = @([pscustomobject]@{ scope = "check"; name = "environment-contract"; dependency_reason = "blocked by failed prerequisite(s): compose-database-contract" })
    $placeholderSource = [pscustomobject]@{
        commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
        acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
    }
    Assert-MutableSourceIdentity -Source $placeholderSource -VerificationState "failed"
    Invoke-ExpectedGuardFailure {
        Assert-MutableSourceIdentity -Source $placeholderSource -VerificationState "verified"
    } "a placeholder source claim marked verified"

    $attempt.schema_version = "2"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a string-coerced attempt schema"
    $attempt.schema_version = 2
    $harvest.phase = "candidate-rehearsal"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an inexact harvest phase"
    $harvest.phase = "candidate-rehearsal-harvest"
    $batch.authoritative = $true
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an authoritative diagnostic batch"
    $batch.authoritative = $false
    $harvest.authoritative = 0
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a numerically coerced diagnostic authority flag"
    $harvest.authoritative = $false
    $savedEnvironmentFingerprint = $attempt.environment_fingerprint
    $attempt.environment_fingerprint = "E" * 64
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an uppercase environment fingerprint"
    $attempt.environment_fingerprint = $savedEnvironmentFingerprint
    $harvest.environment_fingerprint = "e" * 63
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a malformed nested environment fingerprint"
    $harvest.environment_fingerprint = $savedEnvironmentFingerprint
    $source.dirty = 0
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a numerically coerced clean-source flag"
    $source.dirty = $false

    $harvest.checks[2].dependency_reason = ""
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a blocked check without its dependency"
    $harvest.checks[2].dependency_reason = "independent failed"
    $batch.defects[0].classification = "test"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an unsupported classification"
    $batch.defects[0].classification = "harness"
    $batch.defect_count = 3
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a false defect count"
    $batch.defect_count = 2
    $batch.defects[0].raw_evidence = @()
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a defect that drops retained raw evidence"
    $batch.defects[0].raw_evidence = @([pscustomobject]@{ path = "raw/fail.log"; sha256 = "f" * 64 })
    $attempt.declared_checks[2].depends_on = @("uat-diagnostics")
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a block attributed to a passing dependency"
    $attempt.declared_checks[2].depends_on = @("independent")
    $harvest.failed_count = 2
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a false harvest failure count"
    $harvest.failed_count = 1
    $attempt.blocked_count = 2
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a false attempt blocked count"
    $attempt.blocked_count = 1
    $savedBlockedChecks = @($batch.blocked_checks)
    $batch.blocked_checks = @($batch.blocked_checks | Where-Object scope -CNE "scenario")
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a batch that drops a nested blocked scenario"
    $batch.blocked_checks = $savedBlockedChecks
    $harvest.checks[1].nested_blocked_checks[0].dependency_reason = ""
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested block without its dependency reason"
    $harvest.checks[1].nested_blocked_checks[0].dependency_reason = "blocked by invalid prerequisite(s): independent"
    $harvest.checks[1].nested_blocked_checks[0].blocked_by = @("uat-diagnostics")
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested block attributed to a passing prerequisite"
    $harvest.checks[1].nested_blocked_checks[0].blocked_by = @("independent")
    $harvest.checks[1].nested_failed_checks[0].classification = "test"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested semantic failure with an unsupported classification"
    $harvest.checks[1].nested_failed_checks[0].classification = "product"
    $harvest.checks[1].nested_failed_checks[0].raw_evidence = @()
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested semantic failure without raw evidence"
    $harvest.checks[1].nested_failed_checks[0].raw_evidence = @([pscustomobject]@{ path = "raw/semantic.json"; sha256 = "b" * 64 })
    $savedNestedDefect = $batch.defects[1]
    $batch.defects = @($batch.defects[0]); $batch.defect_count = 1
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a batch that drops a nested semantic failure"
    $batch.defects = @($batch.defects[0], $savedNestedDefect); $batch.defect_count = 2
    $harvest.checks[1].state = "failed"; $harvest.checks[1].classification = "harness"; $harvest.checks[1].exit_status = 1
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "double counting a nested semantic failure as the outer UAT lane"
    $harvest.checks[1].state = "passed"; $harvest.checks[1].classification = $null; $harvest.checks[1].exit_status = 0
    $outsideEvidencePath = Join-Path ([IO.Path]::GetTempPath()) "tessara-outside-evidence-$([guid]::NewGuid().ToString('N')).json"
    Invoke-ExpectedGuardFailure {
        Assert-HashedFileEvidence `
            -Evidence ([pscustomobject]@{ path = $outsideEvidencePath; sha256 = "a" * 64 }) `
            -Label "outside evidence"
    } "raw evidence outside the repository evidence root"
    $harvest.mutable_source_identity = [pscustomobject]@{ commit = "wrong" }
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a mismatched source identity"
    Write-Host "Sprint validation harvest guard adversarial self-test passed."
    return
}

foreach ($path in @($AttemptPath, $HarvestPath, $DefectBatchPath)) {
    if ([string]::IsNullOrWhiteSpace($path) -or -not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Attempt, harvest, and defect-batch paths are required and must exist."
    }
}
if (-not $HarvestOnly -and [string]::IsNullOrWhiteSpace($CorrectionAuthorizationPath)) {
    throw "CorrectionAuthorizationPath is required; correction cannot be authorized implicitly."
}
$attemptReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $AttemptPath `
    -AllowLegacyAbsolute
$harvestReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $HarvestPath `
    -AllowLegacyAbsolute
$batchReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $DefectBatchPath `
    -AllowLegacyAbsolute
$authorizationReference = if ($HarvestOnly) { $null } else {
    Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $CorrectionAuthorizationPath `
        -AllowLegacyAbsolute
}
$attemptSha = Assert-Sprint8AReceiptSidecar -Path $AttemptPath
$harvestSha = Assert-Sprint8AReceiptSidecar -Path $HarvestPath
$batchSha = Assert-Sprint8AReceiptSidecar -Path $DefectBatchPath
$attempt = Get-Content -LiteralPath $AttemptPath -Raw | ConvertFrom-Json
$harvest = Get-Content -LiteralPath $HarvestPath -Raw | ConvertFrom-Json
$batch = Get-Content -LiteralPath $DefectBatchPath -Raw | ConvertFrom-Json
Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch

$boundAttemptReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path ([string]$harvest.attempt_receipt.path) `
    -AllowLegacyAbsolute
$boundHarvestReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path ([string]$batch.harvest_receipt.path) `
    -AllowLegacyAbsolute
if ([string]$boundAttemptReference.path -cne [string]$attemptReference.path -or
    $attemptSha -cne [string]$harvest.attempt_receipt.sha256) {
    throw "The harvest does not bind the retained failed-attempt digest."
}
if ([string]$boundHarvestReference.path -cne [string]$harvestReference.path -or
    $harvestSha -cne [string]$batch.harvest_receipt.sha256) {
    throw "The consolidated batch does not bind the retained harvest digest."
}
if ($HarvestOnly) {
    Write-Host "Validation harvest is complete; no correction authorization was requested or issued."
    return
}
$authorizationEligibility = Assert-CorrectionAuthorizationEligibility -Attempt $attempt
$predecessorPhase = [string]$attempt.phase
$authorizationSchema = if ($predecessorPhase -ceq "validation-readiness") {
    2
} else {
    3
}
$authorization = [ordered]@{
    schema_version = $authorizationSchema
    sprint = [string]$attempt.sprint
    phase = "$predecessorPhase-correction-authorization"
    attempt = [int]$attempt.attempt
    authoritative = $false
    state = "authorized"
    consumption_state = "unconsumed"
    allowed_successor_phase = "validation-readiness"
    allowed_successor_count = 1
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    mutable_source_identity = $attempt.mutable_source_identity
    environment_fingerprint = [string]$attempt.environment_fingerprint
    predecessor_attempt_receipt = [ordered]@{ path = [string]$attemptReference.path; sha256 = $attemptSha }
    harvest_receipt = [ordered]@{ path = [string]$harvestReference.path; sha256 = $harvestSha }
    defect_batch = [ordered]@{ path = [string]$batchReference.path; sha256 = $batchSha }
    cleanup_restoration = $authorizationEligibility
    deferred_count = if ($predecessorPhase -ceq "candidate-rehearsal" -and
        $attempt.PSObject.Properties.Name -contains "deferred_count") {
        [int]$attempt.deferred_count
    } else { 0 }
    authorization = "tracked correction and one successor readiness attempt are permitted for this consolidated batch"
}
if ($predecessorPhase -ceq "validation-readiness") {
    $authorization["allowed_successor_attempt"] = [int]$attempt.attempt + 1
}
Publish-Sprint7AEvidence -Document $authorization -OutputPath ([string]$authorizationReference.full_path) | Out-Null
Write-Host "Validation harvest is complete; one consolidated correction/restart is authorized."

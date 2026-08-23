[CmdletBinding()]
param([switch]$SelfTest)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if (-not $SelfTest) {
    throw "This script is an adversarial policy self-test. Invoke it with -SelfTest."
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $PSScriptRoot "tessara-validation-policy.psm1") -Force
$testHeadCommit = (@(& git -C $repositoryRoot rev-parse HEAD 2>$null) -join "").Trim()
$testHeadTree = (@(& git -C $repositoryRoot rev-parse "HEAD^{tree}" 2>$null) -join "").Trim()
if ($LASTEXITCODE -ne 0 -or $testHeadCommit -cnotmatch '^[0-9a-f]{40,64}$' -or
    $testHeadTree -cnotmatch '^[0-9a-f]{40,64}$') {
    throw "The validation-policy self-test could not authenticate the repository HEAD commit/tree."
}

function Assert-Throws {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        & $Action
    } catch {
        return
    }
    throw "Expected rejection did not occur: $Label"
}

function Assert-ThrowsMatching {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$MessagePattern
    )

    try {
        & $Action
    } catch {
        if ($_.Exception.Message -notmatch $MessagePattern) {
            throw "Rejection '$Label' did not match '$MessagePattern': $($_.Exception.Message)"
        }
        return
    }
    throw "Expected rejection did not occur: $Label"
}

function New-Reference {
    param([string]$Path, [string]$Sha = ("a" * 64))
    return [pscustomobject]@{ path = $Path; sha256 = $Sha }
}

function ConvertTo-TestDateTimeOffset {
    param([Parameter(Mandatory)][string]$Value)

    return [DateTimeOffset]::Parse(
        $Value,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind
    )
}

function New-TestDefectProvenanceRecord {
    param(
        [Parameter(Mandatory)][string]$RecordId,
        [ValidateSet("open", "classified", "corrected", "verified", "blocked", "superseded")]
        [string]$Status = "verified",
        [string]$Sprint = "sprint-9a",
        [datetimeoffset]$CreatedAt = (ConvertTo-TestDateTimeOffset "2026-08-10T10:00:00Z"),
        [Nullable[datetimeoffset]]$UpdatedAt = $null,
        [string]$SourceCommit = $testHeadCommit,
        [string]$SourceTree = $testHeadTree,
        [object[]]$Supersedes = @()
    )

    $findingStatus = switch ($Status) {
        "verified" { "verified" }
        "corrected" { "corrected" }
        "blocked" { "blocked" }
        "superseded" { "superseded" }
        default { "open" }
    }
    $sourceIdentity = [pscustomobject]@{
        commit = $SourceCommit
        tree = $SourceTree
        dirty = $false
    }
    $effectiveUpdatedAt = if ($null -eq $UpdatedAt) { $CreatedAt } else { [datetimeoffset]$UpdatedAt }
    $record = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.defect-provenance"
        policy_version = "tessara-validation-v2"
        sprint = $Sprint
        record_id = $RecordId
        status = $Status
        trigger = [pscustomobject]@{
            phase = "implementation"
            subject = "policy-selftest"
            attempt = 1
            state = "failed"
            assertions_started = $true
            product_actions_started = $false
            consecutive_failure_count = 1
            failed_receipt = New-Reference -Path "artifacts/$Sprint-closeout/selftest/failure.json"
        }
        source_identity = $sourceIdentity
        environment_fingerprint = ("3" * 64)
        inputs = [pscustomobject]@{
            validation_contract = New-Reference -Path "docs/sprints/$Sprint-validation-contract.json"
            implementation_readiness = $null
            fixture_identity = $null
            acceptance_inventory = $null
            test_change_log = $null
        }
        findings = @(
            [pscustomobject]@{
                id = "finding-$RecordId"
                classification = "evidence-finalization"
                confidence = "high"
                symptom = "Synthetic chronology finding."
                root_cause = "Synthetic chronology cause."
                governing_authority = @("validation-policy-v2")
                affected_subjects = @("policy-selftest")
                evidence = @(New-Reference -Path "artifacts/$Sprint-closeout/selftest/failure.json")
                status = $findingStatus
            }
        )
        drift_assessment = [pscustomobject]@{
            origin_boundary = "process"
            implementation_exit_gap = $false
            process_drift = $true
            affected_domains = @("evidence-publication")
            rationale = "Synthetic chronology coverage."
        }
        routing = [pscustomobject]@{
            owner = "validation-platform"
            action = "repair-process"
            invalidate = @()
            focused_reproducers = @()
            full_rerun_blocked = ($Status -cne "verified")
            next_boundary = if ($Status -ceq "verified") { "validation-readiness" } else { "focused-diagnosis" }
            rationale = "Synthetic chronology routing."
        }
        expectation_changes = @()
        supersedes = @($Supersedes)
        created_at = $CreatedAt.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffffffZ")
        updated_at = $effectiveUpdatedAt.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffffffZ")
    }
    if ($Status -ceq "verified") {
        $record | Add-Member -NotePropertyName correction -NotePropertyValue ([pscustomobject]@{
            changed_paths = @("scripts/test-tessara-validation-policy.ps1")
            changed_domains = @("evidence-publication")
            focused_results = @(
                [pscustomobject]@{
                    id = "focused-$RecordId"
                    command = "synthetic focused chronology proof"
                    state = "passed"
                    evidence = New-Reference -Path "artifacts/$Sprint-closeout/selftest/focused.json"
                }
            )
            implementation_target_results = @(
                [pscustomobject]@{
                    id = "target-$RecordId"
                    command = "synthetic chronology implementation target"
                    state = "passed"
                    evidence = New-Reference -Path "artifacts/$Sprint-closeout/selftest/target.json"
                }
            )
            clean_source = $sourceIdentity | ConvertTo-Json -Depth 5 | ConvertFrom-Json
            authorized_next_boundary = "validation-readiness"
        })
        $record.routing.focused_reproducers = @(
            $record.correction.focused_results[0] | ConvertTo-Json -Depth 10 | ConvertFrom-Json
        )
    }
    return $record
}

function New-TestChronologyRoot {
    param(
        [Parameter(Mandatory)][string]$Parent,
        [Parameter(Mandatory)][string]$Name
    )

    $path = Join-Path $Parent $Name
    New-Item -ItemType Directory -Path $path -Force | Out-Null
    return $path
}

function Write-TestEvidenceArtifact {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceDirectory,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Content
    )

    New-Item -ItemType Directory -Path $EvidenceDirectory -Force | Out-Null
    $path = Join-Path $EvidenceDirectory $Name
    Set-Content -LiteralPath $path -Value $Content -Encoding utf8NoBOM -NoNewline
    return New-Reference `
        -Path ([IO.Path]::GetRelativePath($RepositoryRoot, $path).Replace("\", "/")) `
        -Sha (Get-TessaraValidationSha256 -Path $path)
}

function Get-TestGitBlobReference {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Commit,
        [Parameter(Mandatory)][string]$Path
    )

    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = "git"
    $startInfo.WorkingDirectory = $RepositoryRoot
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.ArgumentList.Add("cat-file")
    $startInfo.ArgumentList.Add("blob")
    $startInfo.ArgumentList.Add("$Commit`:$Path")
    $process = [Diagnostics.Process]::Start($startInfo)
    try {
        $sha = [Security.Cryptography.SHA256]::Create()
        try {
            $hash = $sha.ComputeHash($process.StandardOutput.BaseStream)
        } finally {
            $sha.Dispose()
        }
        $errorText = $process.StandardError.ReadToEnd()
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) {
            throw "Unable to read synthetic Git blob '$Commit`:$Path': $errorText"
        }
    } finally {
        $process.Dispose()
    }
    return New-Reference -Path $Path `
        -Sha ([Convert]::ToHexString($hash).ToLowerInvariant())
}

function New-TestSourceRepository {
    param([Parameter(Mandatory)][string]$Path)

    New-Item -ItemType Directory -Path (Join-Path $Path "tracked") -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $Path "tracked/validation-contract.json") `
        -Value '{"tracked":"validation-contract"}' -Encoding utf8NoBOM -NoNewline
    Set-Content -LiteralPath (Join-Path $Path "tracked/test-change-log.md") `
        -Value "tracked test change log" -Encoding utf8NoBOM -NoNewline
    Set-Content -LiteralPath (Join-Path $Path "tracked/acceptance-inventory.json") `
        -Value '{"tracked":"acceptance-inventory"}' -Encoding utf8NoBOM -NoNewline
    Set-Content -LiteralPath (Join-Path $Path "tracked/source-only-evidence.json") `
        -Value '{"tracked":"source-only"}' -Encoding utf8NoBOM -NoNewline
    & git -C $Path init --quiet
    if ($LASTEXITCODE -ne 0) { throw "Synthetic chronology Git init failed." }
    & git -C $Path config user.name "Tessara Validation Self-Test"
    & git -C $Path config user.email "validation-selftest@tessara.invalid"
    & git -C $Path add --all
    & git -C $Path commit --quiet -m "synthetic chronology source"
    if ($LASTEXITCODE -ne 0) { throw "Synthetic chronology Git commit failed." }
    $commit = (@(& git -C $Path rev-parse HEAD) -join "").Trim()
    $tree = (@(& git -C $Path rev-parse "HEAD^{tree}") -join "").Trim()
    $references = [pscustomobject]@{
        validation_contract = Get-TestGitBlobReference -RepositoryRoot $Path `
            -Commit $commit -Path "tracked/validation-contract.json"
        test_change_log = Get-TestGitBlobReference -RepositoryRoot $Path `
            -Commit $commit -Path "tracked/test-change-log.md"
        acceptance_inventory = Get-TestGitBlobReference -RepositoryRoot $Path `
            -Commit $commit -Path "tracked/acceptance-inventory.json"
        source_only_evidence = Get-TestGitBlobReference -RepositoryRoot $Path `
            -Commit $commit -Path "tracked/source-only-evidence.json"
    }
    Remove-Item -LiteralPath (Join-Path $Path "tracked/validation-contract.json") -Force
    Remove-Item -LiteralPath (Join-Path $Path "tracked/test-change-log.md") -Force
    Remove-Item -LiteralPath (Join-Path $Path "tracked/acceptance-inventory.json") -Force
    Remove-Item -LiteralPath (Join-Path $Path "tracked/source-only-evidence.json") -Force
    return [pscustomobject]@{
        root = $Path
        commit = $commit
        tree = $tree
        references = $references
    }
}

function Write-TestDefectProvenanceRecord {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Directory,
        [Parameter(Mandatory)][object]$Record,
        [switch]$AuthenticateSidecar,
        [switch]$NamedSidecar,
        [switch]$PreserveEvidenceReferences
    )

    $recordDirectory = Join-Path $EvidenceRoot $Directory
    New-Item -ItemType Directory -Path $recordDirectory -Force | Out-Null
    if (-not $PreserveEvidenceReferences) {
        $evidenceDirectory = Join-Path $recordDirectory "evidence"
        $Record.trigger.failed_receipt = Write-TestEvidenceArtifact `
            -RepositoryRoot $RepositoryRoot -EvidenceDirectory $evidenceDirectory `
            -Name "failed-receipt.json" -Content "{`"state`":`"failed`"}"
        foreach ($inputName in @(
            "validation_contract",
            "implementation_readiness",
            "fixture_identity",
            "acceptance_inventory",
            "test_change_log"
        )) {
            if ($inputName -ceq "validation_contract" -or $null -ne $Record.inputs.$inputName) {
                $Record.inputs.$inputName = Write-TestEvidenceArtifact `
                    -RepositoryRoot $RepositoryRoot -EvidenceDirectory $evidenceDirectory `
                    -Name "input-$inputName.json" -Content "{`"input`":`"$inputName`"}"
            }
        }
        for ($findingIndex = 0; $findingIndex -lt @($Record.findings).Count; $findingIndex += 1) {
            $findingReferences = [Collections.Generic.List[object]]::new()
            for ($evidenceIndex = 0; `
                $evidenceIndex -lt @($Record.findings[$findingIndex].evidence).Count; `
                $evidenceIndex += 1) {
                $findingReferences.Add((Write-TestEvidenceArtifact `
                    -RepositoryRoot $RepositoryRoot -EvidenceDirectory $evidenceDirectory `
                    -Name "finding-$findingIndex-evidence-$evidenceIndex.json" `
                    -Content "{`"finding`":$findingIndex,`"evidence`":$evidenceIndex}"))
            }
            $Record.findings[$findingIndex].evidence = @($findingReferences)
        }
        if ($Record.PSObject.Properties.Name -contains "correction") {
            foreach ($resultCollection in @("focused_results", "implementation_target_results")) {
                for ($resultIndex = 0; `
                    $resultIndex -lt @($Record.correction.$resultCollection).Count; `
                    $resultIndex += 1) {
                    if ($null -ne $Record.correction.$resultCollection[$resultIndex].evidence) {
                        $Record.correction.$resultCollection[$resultIndex].evidence = `
                            Write-TestEvidenceArtifact -RepositoryRoot $RepositoryRoot `
                                -EvidenceDirectory $evidenceDirectory `
                                -Name "$resultCollection-$resultIndex.json" `
                                -Content "{`"collection`":`"$resultCollection`",`"index`":$resultIndex}"
                    }
                }
            }
        }
        for ($reproducerIndex = 0; `
            $reproducerIndex -lt @($Record.routing.focused_reproducers).Count; `
            $reproducerIndex += 1) {
            $reproducer = $Record.routing.focused_reproducers[$reproducerIndex]
            $matchingResult = @(
                if ($Record.PSObject.Properties.Name -contains "correction") {
                    $Record.correction.focused_results | Where-Object {
                        [string]$_.id -ceq [string]$reproducer.id
                    }
                }
            )
            if ($matchingResult.Count -eq 1 -and $null -ne $matchingResult[0].evidence) {
                $reproducer.evidence = $matchingResult[0].evidence
            } elseif ($null -ne $reproducer.evidence) {
                $reproducer.evidence = Write-TestEvidenceArtifact `
                    -RepositoryRoot $RepositoryRoot -EvidenceDirectory $evidenceDirectory `
                    -Name "focused-reproducer-$reproducerIndex.json" `
                    -Content "{`"focused_reproducer`":$reproducerIndex}"
            }
        }
        for ($changeIndex = 0; `
            $changeIndex -lt @($Record.expectation_changes).Count; `
            $changeIndex += 1) {
            $Record.expectation_changes[$changeIndex].test_change_log = Write-TestEvidenceArtifact `
                -RepositoryRoot $RepositoryRoot -EvidenceDirectory $evidenceDirectory `
                -Name "expectation-change-$changeIndex.json" `
                -Content "{`"expectation_change`":$changeIndex}"
        }
    }
    $path = Join-Path $recordDirectory "defect-provenance.json"
    $Record | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $path -Encoding utf8NoBOM
    $sha256 = Get-TessaraValidationSha256 -Path $path
    if ($AuthenticateSidecar) {
        $sidecarText = if ($NamedSidecar) {
            "$sha256  defect-provenance.json"
        } else { $sha256 }
        Set-Content -LiteralPath "$path.sha256" -Value $sidecarText -Encoding ascii -NoNewline
    }
    return [pscustomobject]@{
        path = [IO.Path]::GetRelativePath($RepositoryRoot, $path).Replace("\", "/")
        full_path = $path
        sha256 = $sha256
    }
}

function Assert-TestChronologyPassed {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [int]$ExpectedRecordCount = -1
    )

    $audit = Assert-TessaraDefectProvenanceChronology -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot -Sprint "sprint-9a"
    if ([string]$audit.state -cne "passed") {
        throw "Chronology unexpectedly failed for '$EvidenceRoot'."
    }
    if ($ExpectedRecordCount -ge 0 -and [int]$audit.record_count -ne $ExpectedRecordCount) {
        throw "Chronology record count for '$EvidenceRoot' was $($audit.record_count), expected $ExpectedRecordCount."
    }
    return $audit
}

function Assert-TestChronologyRejected {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Label,
        [string]$IssuePattern
    )

    $audit = Get-TessaraDefectProvenanceChronology -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot -Sprint "sprint-9a"
    if ([string]$audit.state -cne "failed") {
        throw "Chronology unexpectedly passed: $Label"
    }
    if (-not [string]::IsNullOrWhiteSpace($IssuePattern) -and
        -not (@($audit.issues) -match $IssuePattern)) {
        throw "Chronology rejection '$Label' did not report issue pattern '$IssuePattern'."
    }
    Assert-Throws -Label $Label -Action {
        Assert-TessaraDefectProvenanceChronology -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot -Sprint "sprint-9a"
    }
    return $audit
}

$temporaryRoot = Join-Path $repositoryRoot ("tmp/validation-policy-v2-selftest-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temporaryRoot -Force | Out-Null

try {
    $chronologyRoot = New-TestChronologyRoot -Parent $temporaryRoot -Name "defect-provenance-chronology"

    $emptyChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "empty"
    $emptyAudit = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $emptyChronology -ExpectedRecordCount 0
    if ([int]$emptyAudit.unresolved_count -ne 0) {
        throw "An empty chronology reported unresolved records."
    }

    $verifiedChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "all-verified"
    $authenticatedRecord = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedChronology -Directory "authenticated" `
        -Record (New-TestDefectProvenanceRecord -RecordId "verified-authenticated") `
        -AuthenticateSidecar
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedChronology -Directory "without-sidecar" `
        -Record (New-TestDefectProvenanceRecord -RecordId "verified-without-sidecar" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T10:01:00Z"))
    $namedSidecarRecord = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedChronology -Directory "named-sidecar" `
        -Record (New-TestDefectProvenanceRecord -RecordId "verified-named-sidecar" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T10:02:00Z")) `
        -AuthenticateSidecar -NamedSidecar
    $verifiedAudit = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedChronology -ExpectedRecordCount 3
    $authenticatedAuditRecords = @($verifiedAudit.records | Where-Object {
        [string]$_.path -in @(
            [string]$authenticatedRecord.path,
            [string]$namedSidecarRecord.path
        )
    })
    if ($authenticatedAuditRecords.Count -ne 2 -or
        @($authenticatedAuditRecords | Where-Object {
            [string]$_.sidecar_state -cne "authenticated"
        }).Count -ne 0 -or [int]$verifiedAudit.verified_count -ne 3 -or
        @($verifiedAudit.records | Where-Object {
            [int]$_.evidence_reference_count -ne 5 -or
            [int]$_.authenticated_evidence_reference_count -ne 5
        }).Count -ne 0) {
        throw "A valid digest-only or named sidecar did not authenticate its provenance record."
    }

    $fakeCommitChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "fake-source-commit"
    $fakeCommitRecord = New-TestDefectProvenanceRecord -RecordId "fake-source-commit"
    $fakeCommitRecord.source_identity.commit = "f" * 40
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $fakeCommitChronology -Directory "record" -Record $fakeCommitRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $fakeCommitChronology -Label "fake source commit"

    $mismatchedTreeChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "mismatched-source-tree"
    $mismatchedTreeRecord = New-TestDefectProvenanceRecord -RecordId "mismatched-source-tree"
    $mismatchedTreeRecord.source_identity.tree = "e" * 40
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mismatchedTreeChronology -Directory "record" `
        -Record $mismatchedTreeRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mismatchedTreeChronology -Label "mismatched source tree"

    $fakeCorrectionCommitChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "fake-correction-commit"
    $fakeCorrectionCommitRecord = New-TestDefectProvenanceRecord `
        -RecordId "fake-correction-commit"
    $fakeCorrectionCommitRecord.correction.clean_source.commit = "d" * 40
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $fakeCorrectionCommitChronology -Directory "record" `
        -Record $fakeCorrectionCommitRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $fakeCorrectionCommitChronology -Label "fake correction commit"

    $mismatchedCorrectionTreeChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "mismatched-correction-tree"
    $mismatchedCorrectionTreeRecord = New-TestDefectProvenanceRecord `
        -RecordId "mismatched-correction-tree"
    $mismatchedCorrectionTreeRecord.correction.clean_source.tree = "c" * 40
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mismatchedCorrectionTreeChronology -Directory "record" `
        -Record $mismatchedCorrectionTreeRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mismatchedCorrectionTreeChronology `
        -Label "mismatched correction tree"

    $missingBoundaryChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "missing-authorized-boundary"
    $missingBoundaryRecord = New-TestDefectProvenanceRecord `
        -RecordId "missing-authorized-boundary"
    $missingBoundaryRecord.correction.PSObject.Properties.Remove("authorized_next_boundary")
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingBoundaryChronology -Directory "record" `
        -Record $missingBoundaryRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingBoundaryChronology -Label "missing authorized next boundary"

    $mismatchedBoundaryChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "mismatched-authorized-boundary"
    $mismatchedBoundaryRecord = New-TestDefectProvenanceRecord `
        -RecordId "mismatched-authorized-boundary"
    $mismatchedBoundaryRecord.correction.authorized_next_boundary = "candidate-rehearsal"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mismatchedBoundaryChronology -Directory "record" `
        -Record $mismatchedBoundaryRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mismatchedBoundaryChronology -Label "mismatched authorized next boundary"

    $blockedVerifiedChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "blocked-verified-record"
    $blockedVerifiedRecord = New-TestDefectProvenanceRecord -RecordId "blocked-verified-record"
    $blockedVerifiedRecord.routing.full_rerun_blocked = $true
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $blockedVerifiedChronology -Directory "record" `
        -Record $blockedVerifiedRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $blockedVerifiedChronology -Label "verified record with blocked rerun"

    foreach ($unresolvedStatus in @("open", "classified", "corrected", "blocked", "superseded")) {
        $statusChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "status-$unresolvedStatus"
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $statusChronology -Directory "record" `
            -Record (New-TestDefectProvenanceRecord -RecordId "status-$unresolvedStatus" `
                -Status $unresolvedStatus)
        $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $statusChronology -Label "$unresolvedStatus provenance status" `
            -IssuePattern "status-$unresolvedStatus"
    }

    $invalidChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "invalid-unsuperseded"
    $invalidRecord = New-TestDefectProvenanceRecord -RecordId "invalid-unsuperseded"
    $invalidRecord | Add-Member -NotePropertyName unsupported_property -NotePropertyValue $true
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidChronology -Directory "record" -Record $invalidRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidChronology -Label "invalid unsuperseded provenance" `
        -IssuePattern "schema-invalid"

    $missingTimestampChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "schema-invalid-missing-timestamps"
    $missingTimestampRecord = New-TestDefectProvenanceRecord `
        -RecordId "schema-invalid-missing-timestamps"
    $missingTimestampRecord.PSObject.Properties.Remove("created_at")
    $missingTimestampRecord.PSObject.Properties.Remove("updated_at")
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingTimestampChronology -Directory "record" `
        -Record $missingTimestampRecord
    $missingTimestampAudit = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingTimestampChronology `
        -Label "schema-invalid record missing timestamps" -IssuePattern "schema-invalid"
    if ([int]$missingTimestampAudit.record_count -ne 1 -or
        [int]$missingTimestampAudit.schema_valid_count -ne 0 -or
        [int]$missingTimestampAudit.schema_invalid_count -ne 1 -or
        [int]$missingTimestampAudit.verified_count -ne 0 -or
        [int]$missingTimestampAudit.unresolved_count -ne 1 -or
        $null -ne $missingTimestampAudit.records[0].created_at -or
        $null -ne $missingTimestampAudit.records[0].updated_at) {
        throw "A timestamp-missing schema-invalid record did not return exact failed audit counts."
    }

    $supersededChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "exact-supersession"
    $supersededRecord = New-TestDefectProvenanceRecord -RecordId "superseded-invalid"
    $supersededRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $supersededTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $supersededChronology -Directory "original" -Record $supersededRecord
    $exactReference = New-Reference -Path $supersededTarget.path -Sha $supersededTarget.sha256
    $supersederInfo = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $supersededChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "verified-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($exactReference))
    $supersededAudit = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $supersededChronology -ExpectedRecordCount 2
    $supersededAuditTarget = @($supersededAudit.records | Where-Object {
        [string]$_.path -ceq [string]$supersededTarget.path
    })
    if ([int]$supersededAudit.record_count -ne 2 -or
        [int]$supersededAudit.schema_valid_count -ne 1 -or
        [int]$supersededAudit.schema_invalid_count -ne 1 -or
        [int]$supersededAudit.verified_count -ne 1 -or
        [int]$supersededAudit.validly_superseded_count -ne 1 -or
        [int]$supersededAudit.unresolved_count -ne 0 -or
        $supersededAuditTarget.Count -ne 1 -or
        [string]$supersededAuditTarget[0].superseded_by.path -cne [string]$supersederInfo.path -or
        [string]$supersededAuditTarget[0].superseded_by.sha256 -cne [string]$supersederInfo.sha256) {
        throw "An exact later verified supersession did not resolve its immutable invalid record."
    }

    $duplicateReferenceChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "duplicate-supersession-reference"
    $duplicateReferenceTargetRecord = New-TestDefectProvenanceRecord `
        -RecordId "duplicate-reference-target"
    $duplicateReferenceTargetRecord | Add-Member -NotePropertyName obsolete_shape `
        -NotePropertyValue $true
    $duplicateReferenceTarget = Write-TestDefectProvenanceRecord `
        -RepositoryRoot $repositoryRoot -EvidenceRoot $duplicateReferenceChronology `
        -Directory "original" -Record $duplicateReferenceTargetRecord
    $duplicateReference = New-Reference -Path $duplicateReferenceTarget.path `
        -Sha $duplicateReferenceTarget.sha256
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateReferenceChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "duplicate-reference-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($duplicateReference, $duplicateReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateReferenceChronology -Label "duplicate supersession reference" `
        -IssuePattern "duplicates supersession"

    $notLaterChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "superseder-not-later"
    $notLaterTargetRecord = New-TestDefectProvenanceRecord -RecordId "not-later-target" `
        -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z")
    $notLaterTargetRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $notLaterTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $notLaterChronology -Directory "original" -Record $notLaterTargetRecord
    $notLaterReference = New-Reference -Path $notLaterTarget.path -Sha $notLaterTarget.sha256
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $notLaterChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "not-later-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T10:00:00Z") `
            -Supersedes @($notLaterReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $notLaterChronology -Label "superseder is not later" `
        -IssuePattern "is not later"

    $wrongHashChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "wrong-hash"
    $wrongHashRecord = New-TestDefectProvenanceRecord -RecordId "wrong-hash-target"
    $wrongHashRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $wrongHashTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $wrongHashChronology -Directory "original" -Record $wrongHashRecord
    $wrongHashReference = New-Reference -Path $wrongHashTarget.path -Sha ("9" * 64)
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $wrongHashChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "wrong-hash-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($wrongHashReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $wrongHashChronology -Label "wrong supersession hash" `
        -IssuePattern "wrong hash"

    $danglingChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "wrong-path"
    $danglingReference = New-Reference `
        -Path ([IO.Path]::GetRelativePath($repositoryRoot, (Join-Path $danglingChronology "missing/defect-provenance.json")).Replace("\", "/"))
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $danglingChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "dangling-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($danglingReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $danglingChronology -Label "dangling supersession path" `
        -IssuePattern "dangling supersession"

    $duplicateSupersedersChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "duplicate-superseders"
    $duplicateTargetRecord = New-TestDefectProvenanceRecord -RecordId "duplicate-superseders-target"
    $duplicateTargetRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $duplicateTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateSupersedersChronology -Directory "original" -Record $duplicateTargetRecord
    $duplicateReference = New-Reference -Path $duplicateTarget.path -Sha $duplicateTarget.sha256
    foreach ($supersederNumber in 1..2) {
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $duplicateSupersedersChronology -Directory "replacement-$supersederNumber" `
            -Record (New-TestDefectProvenanceRecord -RecordId "duplicate-superseder-$supersederNumber" `
                -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T1$supersederNumber`:00:00Z") `
                -Supersedes @($duplicateReference))
    }
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateSupersedersChronology -Label "multiple verified superseders" `
        -IssuePattern "ambiguous multiple supersession"

    $invalidSupersederChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "invalid-superseder"
    $invalidSupersederTargetRecord = New-TestDefectProvenanceRecord `
        -RecordId "invalid-superseder-target"
    $invalidSupersederTargetRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $invalidSupersederTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidSupersederChronology -Directory "original" `
        -Record $invalidSupersederTargetRecord
    $invalidSupersederReference = New-Reference `
        -Path $invalidSupersederTarget.path -Sha $invalidSupersederTarget.sha256
    $invalidSupersederRecord = New-TestDefectProvenanceRecord -RecordId "invalid-superseder" `
        -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
        -Supersedes @($invalidSupersederReference)
    $invalidSupersederRecord | Add-Member -NotePropertyName unsupported_property -NotePropertyValue $true
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidSupersederChronology -Directory "replacement" `
        -Record $invalidSupersederRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidSupersederChronology -Label "invalid superseder" `
        -IssuePattern "schema-invalid"

    $nonverifiedSupersederChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "nonverified-superseder"
    $nonverifiedTargetRecord = New-TestDefectProvenanceRecord -RecordId "nonverified-target"
    $nonverifiedTargetRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $nonverifiedTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $nonverifiedSupersederChronology -Directory "original" -Record $nonverifiedTargetRecord
    $nonverifiedReference = New-Reference -Path $nonverifiedTarget.path -Sha $nonverifiedTarget.sha256
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $nonverifiedSupersederChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "nonverified-superseder" `
            -Status "corrected" -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($nonverifiedReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $nonverifiedSupersederChronology -Label "nonverified superseder" `
        -IssuePattern "not (completely )?verified"

    $selfSupersessionChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "self-supersession"
    $selfPath = [IO.Path]::GetRelativePath(
        $repositoryRoot,
        (Join-Path $selfSupersessionChronology "self/defect-provenance.json")
    ).Replace("\", "/")
    $selfReference = New-Reference -Path $selfPath
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $selfSupersessionChronology -Directory "self" `
        -Record (New-TestDefectProvenanceRecord -RecordId "self-superseder" `
            -Supersedes @($selfReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $selfSupersessionChronology -Label "self supersession" `
        -IssuePattern "cannot supersede itself"

    $foreignSprintChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "foreign-sprint"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $foreignSprintChronology -Directory "record" `
        -Record (New-TestDefectProvenanceRecord -RecordId "foreign-sprint" -Sprint "sprint-9b")
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $foreignSprintChronology -Label "foreign sprint provenance" `
        -IssuePattern "schema-invalid"

    $duplicateRecordIdChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "duplicate-record-id"
    foreach ($recordNumber in 1..2) {
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $duplicateRecordIdChronology -Directory "record-$recordNumber" `
            -Record (New-TestDefectProvenanceRecord -RecordId "duplicate-record-id" `
                -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T1$recordNumber`:00:00Z"))
    }
    $duplicateRecordIdAudit = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateRecordIdChronology -Label "duplicate provenance record ids" `
        -IssuePattern "duplicates record_id"
    if ([int]$duplicateRecordIdAudit.record_count -ne 2 -or
        [int]$duplicateRecordIdAudit.schema_valid_count -ne 2 -or
        [int]$duplicateRecordIdAudit.verified_count -ne 0 -or
        [int]$duplicateRecordIdAudit.validly_superseded_count -ne 0 -or
        [int]$duplicateRecordIdAudit.unresolved_count -ne 2 -or
        @($duplicateRecordIdAudit.records | Where-Object {
            [string]$_.resolution -cne "unresolved"
        }).Count -ne 0) {
        throw "Duplicate record IDs were not retained as two unresolved chronology records."
    }

    $mutatedChronology = New-TestChronologyRoot -Parent $chronologyRoot -Name "byte-mutation"
    $mutatedRecord = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mutatedChronology -Directory "record" `
        -Record (New-TestDefectProvenanceRecord -RecordId "byte-mutation") `
        -AuthenticateSidecar
    $null = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mutatedChronology -ExpectedRecordCount 1
    Add-Content -LiteralPath $mutatedRecord.full_path -Value " " -NoNewline
    $mutatedAudit = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $mutatedChronology -Label "authenticated provenance byte mutation" `
        -IssuePattern "sidecar does not authenticate"
    if ([int]$mutatedAudit.record_count -ne 1 -or
        [int]$mutatedAudit.schema_valid_count -ne 1 -or
        [int]$mutatedAudit.verified_count -ne 0 -or
        [int]$mutatedAudit.unresolved_count -ne 1 -or
        [string]$mutatedAudit.records[0].resolution -cne "unresolved" -or
        [string]$mutatedAudit.records[0].sidecar_state -cne "invalid") {
        throw "A byte-mutated authenticated record was not retained as unresolved."
    }

    $invalidSidecarChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "invalid-sidecar"
    $invalidSidecarRecord = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidSidecarChronology -Directory "record" `
        -Record (New-TestDefectProvenanceRecord -RecordId "invalid-sidecar") `
        -AuthenticateSidecar
    Set-Content -LiteralPath "$($invalidSidecarRecord.full_path).sha256" `
        -Value "not-a-valid-sidecar" -Encoding ascii -NoNewline
    $invalidSidecarAudit = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $invalidSidecarChronology -Label "invalid provenance sidecar" `
        -IssuePattern "sidecar does not authenticate"
    if ([int]$invalidSidecarAudit.record_count -ne 1 -or
        [int]$invalidSidecarAudit.schema_valid_count -ne 1 -or
        [int]$invalidSidecarAudit.verified_count -ne 0 -or
        [int]$invalidSidecarAudit.validly_superseded_count -ne 0 -or
        [int]$invalidSidecarAudit.unresolved_count -ne 1 -or
        [string]$invalidSidecarAudit.records[0].resolution -cne "unresolved" -or
        [string]$invalidSidecarAudit.records[0].sidecar_state -cne "invalid") {
        throw "An invalid sidecar did not leave its record unresolved with exact counts."
    }

    $verifiedOpenFindingChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "verified-open-finding"
    $verifiedOpenFindingRecord = New-TestDefectProvenanceRecord `
        -RecordId "verified-open-finding"
    $verifiedOpenFindingRecord.findings[0].status = "open"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedOpenFindingChronology -Directory "record" `
        -Record $verifiedOpenFindingRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedOpenFindingChronology -Label "verified record with open finding"

    $dirtyCorrectionChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "dirty-correction-source"
    $dirtyCorrectionRecord = New-TestDefectProvenanceRecord -RecordId "dirty-correction-source"
    $dirtyCorrectionRecord.correction.clean_source.dirty = $true
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $dirtyCorrectionChronology -Directory "record" -Record $dirtyCorrectionRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $dirtyCorrectionChronology -Label "verified record with dirty correction source"

    foreach ($failedCollection in @("focused_results", "implementation_target_results")) {
        $failedResultChronology = New-TestChronologyRoot -Parent $chronologyRoot `
            -Name "failed-$failedCollection"
        $failedResultRecord = New-TestDefectProvenanceRecord `
            -RecordId "failed-$($failedCollection.Replace('_', '-'))"
        $failedResultRecord.correction.$failedCollection[0].state = "failed"
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $failedResultChronology -Directory "record" -Record $failedResultRecord
        $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $failedResultChronology `
            -Label "verified record with failed $failedCollection result"
    }

    foreach ($missingCollection in @("focused_results", "implementation_target_results")) {
        $missingResultChronology = New-TestChronologyRoot -Parent $chronologyRoot `
            -Name "missing-$missingCollection"
        $missingResultRecord = New-TestDefectProvenanceRecord `
            -RecordId "missing-$($missingCollection.Replace('_', '-'))"
        $missingResultRecord.correction.$missingCollection = @()
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $missingResultChronology -Directory "record" -Record $missingResultRecord
        $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $missingResultChronology `
            -Label "verified record with missing $missingCollection proof"
    }

    $focusedSetMismatchChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "focused-set-mismatch"
    $focusedSetMismatchRecord = New-TestDefectProvenanceRecord `
        -RecordId "focused-set-mismatch"
    $focusedSetMismatchRecord.correction.focused_results[0].id = "different-focused-result"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $focusedSetMismatchChronology -Directory "record" `
        -Record $focusedSetMismatchRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $focusedSetMismatchChronology `
        -Label "focused reproducer and result set mismatch"

    $conflictingCheckIdChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "conflicting-check-id"
    $conflictingCheckIdRecord = New-TestDefectProvenanceRecord -RecordId "conflicting-check-id"
    $conflictingCheckIdRecord.correction.implementation_target_results[0].id = `
        $conflictingCheckIdRecord.correction.focused_results[0].id
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $conflictingCheckIdChronology -Directory "record" `
        -Record $conflictingCheckIdRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $conflictingCheckIdChronology -Label "conflicting correction check id"

    $duplicateFindingIdChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "duplicate-finding-id"
    $duplicateFindingIdRecord = New-TestDefectProvenanceRecord -RecordId "duplicate-finding-id"
    $duplicateFinding = $duplicateFindingIdRecord.findings[0] |
        ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $duplicateFindingIdRecord.findings = @($duplicateFindingIdRecord.findings[0], $duplicateFinding)
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateFindingIdChronology -Directory "record" `
        -Record $duplicateFindingIdRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateFindingIdChronology -Label "duplicate finding ids"

    $duplicateCheckIdChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "duplicate-check-id"
    $duplicateCheckIdRecord = New-TestDefectProvenanceRecord -RecordId "duplicate-check-id"
    $duplicateCheck = $duplicateCheckIdRecord.correction.focused_results[0] |
        ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $duplicateCheckIdRecord.correction.focused_results = @(
        $duplicateCheckIdRecord.correction.focused_results[0],
        $duplicateCheck
    )
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateCheckIdChronology -Directory "record" `
        -Record $duplicateCheckIdRecord
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateCheckIdChronology -Label "duplicate correction check ids"

    $duplicateEvidenceChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "duplicate-evidence-reference"
    $duplicateEvidenceRecord = New-TestDefectProvenanceRecord `
        -RecordId "duplicate-evidence-reference"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateEvidenceChronology -Directory "record" `
        -Record $duplicateEvidenceRecord
    $authenticatedEvidence = $duplicateEvidenceRecord.findings[0].evidence[0]
    $duplicateEvidenceRecord.findings[0].evidence = @(
        $authenticatedEvidence,
        $authenticatedEvidence
    )
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateEvidenceChronology -Directory "record" `
        -Record $duplicateEvidenceRecord -PreserveEvidenceReferences
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $duplicateEvidenceChronology -Label "duplicate evidence references"

    $conflictingEvidenceChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "conflicting-evidence-reference"
    $conflictingEvidenceRecord = New-TestDefectProvenanceRecord `
        -RecordId "conflicting-evidence-reference"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $conflictingEvidenceChronology -Directory "record" `
        -Record $conflictingEvidenceRecord
    $originalEvidence = $conflictingEvidenceRecord.findings[0].evidence[0]
    $conflictingEvidenceRecord.findings[0].evidence = @(
        $originalEvidence,
        (New-Reference -Path $originalEvidence.path -Sha ("9" * 64))
    )
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $conflictingEvidenceChronology -Directory "record" `
        -Record $conflictingEvidenceRecord -PreserveEvidenceReferences
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $conflictingEvidenceChronology -Label "conflicting evidence references"

    $missingEvidenceChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "missing-evidence"
    $missingEvidenceRecord = New-TestDefectProvenanceRecord -RecordId "missing-evidence"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingEvidenceChronology -Directory "record" `
        -Record $missingEvidenceRecord
    $missingEvidenceRecord.trigger.failed_receipt = New-Reference `
        -Path ([IO.Path]::GetRelativePath(
            $repositoryRoot,
            (Join-Path $missingEvidenceChronology "record/evidence/absent.json")
        ).Replace("\", "/"))
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingEvidenceChronology -Directory "record" `
        -Record $missingEvidenceRecord -PreserveEvidenceReferences
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $missingEvidenceChronology -Label "missing hashed evidence"

    $wrongEvidenceHashChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "wrong-evidence-hash"
    $wrongEvidenceHashRecord = New-TestDefectProvenanceRecord -RecordId "wrong-evidence-hash"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $wrongEvidenceHashChronology -Directory "record" `
        -Record $wrongEvidenceHashRecord
    $wrongEvidenceHashRecord.inputs.validation_contract.sha256 = "9" * 64
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $wrongEvidenceHashChronology -Directory "record" `
        -Record $wrongEvidenceHashRecord -PreserveEvidenceReferences
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $wrongEvidenceHashChronology -Label "wrong hashed-evidence digest"

    $dotAliasChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "dot-path-alias"
    $dotAliasRecord = New-TestDefectProvenanceRecord -RecordId "dot-path-alias"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $dotAliasChronology -Directory "record" -Record $dotAliasRecord
    $dotAliasRecord.trigger.failed_receipt.path = `
        $dotAliasRecord.trigger.failed_receipt.path.Replace("/evidence/", "/evidence/./")
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $dotAliasChronology -Directory "record" `
        -Record $dotAliasRecord -PreserveEvidenceReferences
    Assert-ThrowsMatching -Label "dot-segment evidence alias" `
        -MessagePattern "repository-relative|traversal-free" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $dotAliasChronology -Sprint "sprint-9a"
    }

    $adsAliasChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "ads-path-alias"
    $adsAliasRecord = New-TestDefectProvenanceRecord -RecordId "ads-path-alias"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $adsAliasChronology -Directory "record" -Record $adsAliasRecord
    $adsAliasRecord.trigger.failed_receipt.path = `
        "$($adsAliasRecord.trigger.failed_receipt.path):alternate"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $adsAliasChronology -Directory "record" `
        -Record $adsAliasRecord -PreserveEvidenceReferences
    Assert-ThrowsMatching -Label "alternate-data-stream evidence alias" `
        -MessagePattern "repository-relative|traversal-free" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $adsAliasChronology -Sprint "sprint-9a"
    }

    $separatorAliasChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "separator-path-alias"
    $separatorAliasRecord = New-TestDefectProvenanceRecord `
        -RecordId "separator-path-alias"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $separatorAliasChronology -Directory "record" `
        -Record $separatorAliasRecord
    $separatorAliasRecord.trigger.failed_receipt.path = `
        $separatorAliasRecord.trigger.failed_receipt.path.Replace("/evidence/", "/evidence//")
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $separatorAliasChronology -Directory "record" `
        -Record $separatorAliasRecord -PreserveEvidenceReferences
    Assert-ThrowsMatching -Label "duplicate-separator evidence alias" `
        -MessagePattern "repository-relative|traversal-free" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $separatorAliasChronology -Sprint "sprint-9a"
    }

    $caseAliasChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "case-path-alias"
    $caseAliasRecord = New-TestDefectProvenanceRecord -RecordId "case-path-alias"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $caseAliasChronology -Directory "record" -Record $caseAliasRecord
    $caseEvidenceReference = $caseAliasRecord.findings[0].evidence[0]
    $caseAliasRecord.findings[0].evidence = @(
        $caseEvidenceReference,
        (New-Reference -Path $caseEvidenceReference.path.ToUpperInvariant() `
            -Sha $caseEvidenceReference.sha256)
    )
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $caseAliasChronology -Directory "record" `
        -Record $caseAliasRecord -PreserveEvidenceReferences
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $caseAliasChronology -Label "case-variant evidence alias"

    $updatedTargetChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "target-updated-after-superseder"
    $updatedTargetRecord = New-TestDefectProvenanceRecord -RecordId "updated-target" `
        -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T10:00:00Z") `
        -UpdatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T12:00:00Z")
    $updatedTargetRecord | Add-Member -NotePropertyName obsolete_shape -NotePropertyValue $true
    $updatedTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $updatedTargetChronology -Directory "original" -Record $updatedTargetRecord
    $updatedTargetReference = New-Reference -Path $updatedTarget.path -Sha $updatedTarget.sha256
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $updatedTargetChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "premature-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -UpdatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($updatedTargetReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $updatedTargetChronology `
        -Label "superseder predates target update"

    $verifiedTargetChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "supersede-verified-target"
    $verifiedTarget = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedTargetChronology -Directory "original" `
        -Record (New-TestDefectProvenanceRecord -RecordId "verified-target")
    $verifiedTargetReference = New-Reference -Path $verifiedTarget.path -Sha $verifiedTarget.sha256
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedTargetChronology -Directory "replacement" `
        -Record (New-TestDefectProvenanceRecord -RecordId "verified-target-superseder" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z") `
            -Supersedes @($verifiedTargetReference))
    $null = Assert-TestChronologyRejected -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $verifiedTargetChronology -Label "superseding an already verified target"

    $orderedChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "chronological-output"
    $lateRecord = New-TestDefectProvenanceRecord -RecordId "chronology-late" `
        -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T09:00:00Z") `
        -UpdatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T09:30:00Z")
    $earlyRecord = New-TestDefectProvenanceRecord -RecordId "chronology-early" `
        -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T08:00:00Z") `
        -UpdatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T08:30:00Z")
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $orderedChronology -Directory "a-late" -Record $lateRecord
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $orderedChronology -Directory "z-early" -Record $earlyRecord
    $orderedAudit = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $orderedChronology -ExpectedRecordCount 2
    $orderedIds = @($orderedAudit.records | ForEach-Object { [string]$_.record_id })
    if (($orderedIds -join ",") -cne "chronology-early,chronology-late" -or
        [string]$orderedAudit.records[0].created_at -cne "2026-08-10T08:00:00.0000000+00:00" -or
        [string]$orderedAudit.records[0].updated_at -cne "2026-08-10T08:30:00.0000000+00:00" -or
        [string]$orderedAudit.records[1].created_at -cne "2026-08-10T09:00:00.0000000+00:00" -or
        [string]$orderedAudit.records[1].updated_at -cne "2026-08-10T09:30:00.0000000+00:00" -or
        [int]$orderedAudit.schema_valid_count -ne 2 -or
        [int]$orderedAudit.schema_invalid_count -ne 0 -or
        [int]$orderedAudit.verified_count -ne 2 -or
        [int]$orderedAudit.validly_superseded_count -ne 0 -or
        [int]$orderedAudit.unresolved_count -ne 0) {
        throw "Chronology output did not retain exact timestamp order and counts (early fixture $($earlyRecord.created_at)/$($earlyRecord.updated_at); late fixture $($lateRecord.created_at)/$($lateRecord.updated_at)): $($orderedAudit | ConvertTo-Json -Depth 12 -Compress)"
    }

    $inventoryAdditionChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "inventory-addition-count"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $inventoryAdditionChronology -Directory "first" `
        -Record (New-TestDefectProvenanceRecord -RecordId "inventory-first")
    $initialInventoryAudit = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $inventoryAdditionChronology -ExpectedRecordCount 1
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $inventoryAdditionChronology -Directory "second" `
        -Record (New-TestDefectProvenanceRecord -RecordId "inventory-second" `
            -CreatedAt (ConvertTo-TestDateTimeOffset "2026-08-10T11:00:00Z"))
    $expandedInventoryAudit = Assert-TestChronologyPassed -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $inventoryAdditionChronology -ExpectedRecordCount 2
    if ([int]$initialInventoryAudit.verified_count -ne 1 -or
        [int]$initialInventoryAudit.unresolved_count -ne 0 -or
        [int]$expandedInventoryAudit.verified_count -ne 2 -or
        [int]$expandedInventoryAudit.unresolved_count -ne 0) {
        throw "A deterministic retained inventory addition did not update exact chronology counts."
    }

    $outsideRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-validation-policy-outside"
    Assert-ThrowsMatching -Label "outside evidence root" -MessagePattern "escape|outside" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $outsideRoot -Sprint "sprint-9a"
    }
    $siblingRoot = "$repositoryRoot-sibling-probe"
    Assert-ThrowsMatching -Label "sibling-prefix evidence root" -MessagePattern "escape|outside" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $siblingRoot -Sprint "sprint-9a"
    }
    $traversalRoot = Join-Path $chronologyRoot "path-segment/../empty"
    Assert-ThrowsMatching -Label "traversal-bearing evidence root" `
        -MessagePattern "traversal|relative" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $traversalRoot -Sprint "sprint-9a"
    }
    $missingRoot = Join-Path $chronologyRoot "missing-root"
    Assert-ThrowsMatching -Label "missing evidence root" -MessagePattern "missing" -Action {
        Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $missingRoot -Sprint "sprint-9a"
    }

    $reparseTarget = New-TestChronologyRoot -Parent $chronologyRoot -Name "reparse-target"
    $reparseRoot = Join-Path $chronologyRoot "reparse-root"
    $reparseCreated = $false
    try {
        if ($IsWindows) {
            $null = New-Item -ItemType Junction -Path $reparseRoot -Target $reparseTarget
        } else {
            $null = New-Item -ItemType SymbolicLink -Path $reparseRoot -Target $reparseTarget
        }
        $reparseCreated = $true
        Assert-ThrowsMatching -Label "reparse evidence root" `
            -MessagePattern "reparse|symbolic|link" -Action {
            Get-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
                -EvidenceRoot $reparseRoot -Sprint "sprint-9a"
        }
    } finally {
        if ($reparseCreated -and (Test-Path -LiteralPath $reparseRoot)) {
            Remove-Item -LiteralPath $reparseRoot -Force
        }
    }

    $linkedEvidenceChronology = New-TestChronologyRoot -Parent $chronologyRoot `
        -Name "linked-evidence-reference"
    $linkedEvidenceRecord = New-TestDefectProvenanceRecord `
        -RecordId "linked-evidence-reference"
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
        -EvidenceRoot $linkedEvidenceChronology -Directory "record" `
        -Record $linkedEvidenceRecord
    $linkedEvidenceTarget = New-TestChronologyRoot -Parent $linkedEvidenceChronology `
        -Name "evidence-target"
    $linkedEvidenceReference = Write-TestEvidenceArtifact -RepositoryRoot $repositoryRoot `
        -EvidenceDirectory $linkedEvidenceTarget -Name "linked.json" `
        -Content "{`"linked`":true}"
    $linkedEvidencePath = Join-Path $linkedEvidenceChronology "record/evidence-link"
    $linkedEvidenceCreated = $false
    try {
        if ($IsWindows) {
            $null = New-Item -ItemType Junction -Path $linkedEvidencePath `
                -Target $linkedEvidenceTarget
        } else {
            $null = New-Item -ItemType SymbolicLink -Path $linkedEvidencePath `
                -Target $linkedEvidenceTarget
        }
        $linkedEvidenceCreated = $true
        $linkedEvidenceRecord.trigger.failed_receipt = New-Reference `
            -Path ([IO.Path]::GetRelativePath(
                $repositoryRoot,
                (Join-Path $linkedEvidencePath "linked.json")
            ).Replace("\", "/")) `
            -Sha $linkedEvidenceReference.sha256
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $repositoryRoot `
            -EvidenceRoot $linkedEvidenceChronology -Directory "record" `
            -Record $linkedEvidenceRecord -PreserveEvidenceReferences
        Assert-ThrowsMatching -Label "linked evidence reference" `
            -MessagePattern "reparse|symbolic|link" -Action {
            Assert-TessaraDefectProvenanceChronology -RepositoryRoot $repositoryRoot `
                -EvidenceRoot $linkedEvidenceChronology -Sprint "sprint-9a"
        }
    } finally {
        if ($linkedEvidenceCreated -and (Test-Path -LiteralPath $linkedEvidencePath)) {
            Remove-Item -LiteralPath $linkedEvidencePath -Force
        }
    }

    $sourceRepository = New-TestSourceRepository `
        -Path (Join-Path $temporaryRoot "source-auth-repository")
    $sourceCommitChronology = New-TestChronologyRoot -Parent $sourceRepository.root `
        -Name "evidence/source-commit-positive"
    $sourceCommitRecord = New-TestDefectProvenanceRecord `
        -RecordId "source-commit-positive" -SourceCommit $sourceRepository.commit `
        -SourceTree $sourceRepository.tree
    $sourceCommitRecord.inputs.acceptance_inventory = New-Reference `
        -Path "placeholder/acceptance-inventory.json"
    $sourceCommitRecord.inputs.test_change_log = New-Reference `
        -Path "placeholder/test-change-log.md"
    $sourceCommitRecord.expectation_changes = @(
        [pscustomobject]@{
            test_path = "synthetic/acceptance-test.ps1"
            old_assertion = "synthetic old assertion"
            new_assertion = "synthetic stronger assertion"
            authority = @("synthetic approved contract")
            supersession_rationale = "Synthetic source-commit authentication coverage."
            replacement_coverage = "Synthetic equal-or-stronger coverage."
            test_change_log = New-Reference -Path "placeholder/expectation-change-log.md"
        }
    )
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $sourceRepository.root `
        -EvidenceRoot $sourceCommitChronology -Directory "record" `
        -Record $sourceCommitRecord
    $sourceCommitRecord.inputs.validation_contract = `
        $sourceRepository.references.validation_contract
    $sourceCommitRecord.inputs.acceptance_inventory = `
        $sourceRepository.references.acceptance_inventory
    $sourceCommitRecord.inputs.test_change_log = $sourceRepository.references.test_change_log
    $sourceCommitRecord.expectation_changes[0].test_change_log = `
        $sourceRepository.references.test_change_log
    $null = Write-TestDefectProvenanceRecord -RepositoryRoot $sourceRepository.root `
        -EvidenceRoot $sourceCommitChronology -Directory "record" `
        -Record $sourceCommitRecord -PreserveEvidenceReferences
    $sourceCommitAudit = Assert-TessaraDefectProvenanceChronology `
        -RepositoryRoot $sourceRepository.root -EvidenceRoot $sourceCommitChronology `
        -Sprint "sprint-9a"
    if ([string]$sourceCommitAudit.state -cne "passed" -or
        [int]$sourceCommitAudit.record_count -ne 1 -or
        [int]$sourceCommitAudit.verified_count -ne 1 -or
        [int]$sourceCommitAudit.records[0].evidence_reference_count -ne 7 -or
        [int]$sourceCommitAudit.records[0].authenticated_evidence_reference_count -ne 7) {
        throw "Tracked input/test-log source-commit authentication did not pass exactly."
    }

    $sourceCommitOnlyCases = @(
        [pscustomobject]@{
            name = "failed-receipt"
            mutate = { param($Record, $Reference) $Record.trigger.failed_receipt = $Reference }
        },
        [pscustomobject]@{
            name = "finding-evidence"
            mutate = { param($Record, $Reference) $Record.findings[0].evidence = @($Reference) }
        },
        [pscustomobject]@{
            name = "routed-check-evidence"
            mutate = { param($Record, $Reference) $Record.routing.focused_reproducers[0].evidence = $Reference }
        },
        [pscustomobject]@{
            name = "focused-result-evidence"
            mutate = { param($Record, $Reference) $Record.correction.focused_results[0].evidence = $Reference }
        },
        [pscustomobject]@{
            name = "target-result-evidence"
            mutate = { param($Record, $Reference) $Record.correction.implementation_target_results[0].evidence = $Reference }
        },
        [pscustomobject]@{
            name = "implementation-readiness-input"
            mutate = { param($Record, $Reference) $Record.inputs.implementation_readiness = $Reference }
        },
        [pscustomobject]@{
            name = "fixture-input"
            mutate = { param($Record, $Reference) $Record.inputs.fixture_identity = $Reference }
        }
    )
    foreach ($sourceCommitOnlyCase in $sourceCommitOnlyCases) {
        $caseRoot = New-TestChronologyRoot -Parent $sourceRepository.root `
            -Name "evidence/source-commit-rejected-$($sourceCommitOnlyCase.name)"
        $caseRecord = New-TestDefectProvenanceRecord `
            -RecordId "source-commit-rejected-$($sourceCommitOnlyCase.name)" `
            -SourceCommit $sourceRepository.commit -SourceTree $sourceRepository.tree
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $sourceRepository.root `
            -EvidenceRoot $caseRoot -Directory "record" -Record $caseRecord
        & $sourceCommitOnlyCase.mutate $caseRecord `
            $sourceRepository.references.source_only_evidence
        $null = Write-TestDefectProvenanceRecord -RepositoryRoot $sourceRepository.root `
            -EvidenceRoot $caseRoot -Directory "record" -Record $caseRecord `
            -PreserveEvidenceReferences
        $null = Assert-TestChronologyRejected -RepositoryRoot $sourceRepository.root `
            -EvidenceRoot $caseRoot `
            -Label "source-commit fallback for $($sourceCommitOnlyCase.name)"
    }

    $templatePath = Join-Path $repositoryRoot ".codex/skills/tessara-sprint-validation/assets/sprint-validation-contract.json"
    $templateContract = Get-Content -LiteralPath $templatePath -Raw | ConvertFrom-Json
    $null = Assert-TessaraValidationContract -Contract $templateContract
    $playbookImpact = Get-TessaraValidationImpact -Contract $templateContract -ChangedPaths @("docs/architecture/module-extraction-playbook.md")
    if (($playbookImpact.phase_decisions | Where-Object phase -eq "validation-readiness").action -cne "recertify_affected_lanes" -or
        ($playbookImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").action -cne "recertify_affected_lanes" -or
        @($playbookImpact.phase_decisions | Where-Object {
            $_.phase -in @("validation-preflight", "sit", "uat") -and $_.action -ne "rerun_full_phase"
        }).Count -ne 0) {
        throw "The Phase 8 playbook is not bound as shared executable validation policy."
    }

    $contract = [pscustomobject]@{
        schema_version = 2
        contract = "tessara.validation-contract"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        implementation_profile = [pscustomobject]@{ kind = "standard" }
        requirements = @(
            [pscustomobject]@{
                id = "req-product"
                implementation_targets = @("target-static", "target-materialize")
                validation_lanes = @("readiness-contract", "rehearsal-product", "sit-product", "uat-product")
            }
        )
        dependency_domains = @(
            [pscustomobject]@{ name = "product-source"; tracked_inputs = @("crates/**"); environment_sections = @() },
            [pscustomobject]@{ name = "deployment-materialization"; tracked_inputs = @("deploy/**"); environment_sections = @("compose") },
            [pscustomobject]@{ name = "readiness-runner"; tracked_inputs = @("scripts/readiness*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "rehearsal-runner"; tracked_inputs = @("scripts/rehearsal*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "preflight-runner"; tracked_inputs = @("scripts/preflight*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "sit-runner"; tracked_inputs = @("scripts/sit*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "uat-runner"; tracked_inputs = @("scripts/uat*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "evidence-publication"; tracked_inputs = @("scripts/evidence*.ps1"); environment_sections = @() }
        )
        implementation_targets = @(
            [pscustomobject]@{
                id = "target-static"
                command = "cargo check --workspace --locked"
                dependency_domains = @("product-source")
                proof_classes = @("static-quality")
                required = $true
                clean_environment = $false
            },
            [pscustomobject]@{
                id = "target-materialize"
                command = ".\scripts\materialize.ps1 -Clean"
                dependency_domains = @("deployment-materialization")
                proof_classes = @("clean-materialization", "semantic-noop", "failure-recovery")
                required = $false
                clean_environment = $true
            }
        )
        lanes = @(
            [pscustomobject]@{ id = "readiness-contract"; phase = "validation-readiness"; dependency_domains = @("readiness-runner"); prerequisites = @(); touches_live_state = $false },
            [pscustomobject]@{ id = "rehearsal-product"; phase = "candidate-rehearsal"; dependency_domains = @("product-source", "rehearsal-runner"); prerequisites = @(); touches_live_state = $false },
            [pscustomobject]@{ id = "rehearsal-materialize"; phase = "candidate-rehearsal"; dependency_domains = @("deployment-materialization", "rehearsal-runner"); prerequisites = @("rehearsal-product"); touches_live_state = $true },
            [pscustomobject]@{ id = "preflight-freeze"; phase = "validation-preflight"; dependency_domains = @("preflight-runner"); prerequisites = @(); touches_live_state = $false },
            [pscustomobject]@{ id = "sit-product"; phase = "sit"; dependency_domains = @("product-source", "sit-runner"); prerequisites = @(); touches_live_state = $true },
            [pscustomobject]@{ id = "uat-product"; phase = "uat"; dependency_domains = @("product-source", "uat-runner"); prerequisites = @(); touches_live_state = $true }
        )
        evidence_policy = [pscustomobject]@{
            root = "artifacts/sprint-9a-closeout"
            tracked = $false
            successful_raw = "retained_cold"
            phase_local_indexes = $true
            final_full_integrity_audit = $true
        }
    }

    $null = Assert-TessaraValidationContract -Contract $contract
    $fingerprintPassOne = @(Get-TessaraDependencyFingerprints -Contract $contract -RepositoryRoot $repositoryRoot)
    $fingerprintPassTwo = @(Get-TessaraDependencyFingerprints -Contract $contract -RepositoryRoot $repositoryRoot)
    if ($fingerprintPassOne.Count -ne @($contract.dependency_domains).Count -or
        (($fingerprintPassOne | ConvertTo-Json -Depth 10 -Compress) -cne ($fingerprintPassTwo | ConvertTo-Json -Depth 10 -Compress))) {
        throw "Dependency fingerprints are incomplete or nondeterministic."
    }
    $duplicateContract = $contract | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $duplicateContract.dependency_domains += $duplicateContract.dependency_domains[0]
    Assert-Throws -Label "duplicate dependency domain" -Action { Assert-TessaraValidationContract -Contract $duplicateContract }

    $extractionContract = $contract | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $extractionContract.implementation_profile = [pscustomobject]@{
        kind = "phase8-module-extraction"
        playbook = "docs/architecture/module-extraction-playbook.md"
        module_definition = "tessara.datasets"
        transition_identity = "tessara.datasets"
    }
    Assert-Throws -Label "incomplete Phase 8 extraction proof profile" -Action {
        Assert-TessaraValidationContract -Contract $extractionContract
    }

    $extractionProofClasses = @(
        "static-quality",
        "contract-boundary",
        "owner-product",
        "ui-sdk-conformance",
        "consumer-cutover",
        "core-subtraction",
        "inventory-navigation",
        "migration-seed",
        "clean-materialization",
        "semantic-noop",
        "failure-recovery",
        "fixture-acceptance",
        "runner-selftest",
        "deployed-smoke",
        "independent-upgrade-rollback",
        "uat-readiness"
    )
    $extractionContract.implementation_targets[0].proof_classes = $extractionProofClasses
    $extractionContract.implementation_targets[0].clean_environment = $true
    $null = Assert-TessaraValidationContract -Contract $extractionContract

    $nonCleanExtractionContract = $extractionContract | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $nonCleanExtractionContract.implementation_targets[0].clean_environment = $false
    Assert-Throws -Label "Phase 8 extraction clean-environment proof" -Action {
        Assert-TessaraValidationContract -Contract $nonCleanExtractionContract
    }

    $preflightImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("scripts/preflight-freeze.ps1")
    if (($preflightImpact.phase_decisions | Where-Object phase -eq "validation-readiness").action -cne "reuse_certificate" -or
        ($preflightImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").action -cne "reuse_certificate" -or
        ($preflightImpact.phase_decisions | Where-Object phase -eq "validation-preflight").action -cne "rerun_full_phase") {
        throw "A Preflight-only change did not preserve Readiness and Rehearsal certificates."
    }

    $productImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("crates/example/src/lib.rs") -CandidateChanged
    if (($productImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").action -cne "recertify_affected_lanes" -or
        ($productImpact.phase_decisions | Where-Object phase -eq "sit").action -cne "rerun_full_phase" -or
        ($productImpact.phase_decisions | Where-Object phase -eq "uat").action -cne "rerun_full_phase" -or
        -not $productImpact.require_complete_sit -or -not $productImpact.require_complete_uat) {
        throw "Candidate-changing product impact did not require pre-freeze recertification plus complete SIT/UAT."
    }

    $materializationImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("deploy/sprint-9a/compose.yaml")
    $materializationLanes = @(($materializationImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").affected_lanes)
    if ("rehearsal-materialize" -notin $materializationLanes -or "rehearsal-product" -notin $materializationLanes) {
        throw "Affected-lane selection did not include the safe prerequisite closure."
    }

    $unknownImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("unknown/new-input.txt")
    if (@($unknownImpact.phase_decisions | Where-Object action -ne "rerun_full_phase").Count -ne 0) {
        throw "An unknown tracked path did not select the conservative full-phase fallback."
    }

    $contractPath = Join-Path $temporaryRoot "validation-contract.json"
    $contract | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $contractPath -Encoding utf8NoBOM
    $contractSha = Get-TessaraValidationSha256 -Path $contractPath
    $source = [pscustomobject]@{ commit = ("1" * 40); tree = ("2" * 40); dirty = $false }
    $impactDocument = [pscustomobject]@{
        schema_version = 2
        contract = "tessara.validation.correction-impact-assessment"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        baseline_source = [pscustomobject]@{ commit = ("7" * 40); tree = ("8" * 40); dirty = $false }
        current_source = $source
        changed_paths = $productImpact.changed_paths
        changed_domains = $productImpact.changed_domains
        unknown_paths = $productImpact.unknown_paths
        phase_decisions = $productImpact.phase_decisions
        candidate_changed = $productImpact.candidate_changed
        require_complete_sit = $productImpact.require_complete_sit
        require_complete_uat = $productImpact.require_complete_uat
        authorized_at = "2026-08-10T10:00:00Z"
    }
    $null = Assert-TessaraCorrectionImpactAssessment -Assessment $impactDocument -Contract $contract
    $unsafeImpact = $impactDocument | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $unsafeImpact.require_complete_uat = $false
    Assert-Throws -Label "candidate correction without complete UAT" -Action {
        Assert-TessaraCorrectionImpactAssessment -Assessment $unsafeImpact -Contract $contract
    }
    $implementationResult = [pscustomobject]@{
        schema_version = 1
        contract = "tessara.implementation-readiness-result"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        state = "passed"
        authoritative = $false
        source_identity = $source
        validation_contract = New-Reference -Path "docs/sprints/sprint-9a-validation-contract.json" -Sha $contractSha
        affected_domains = @("product-source")
        targets = @(
            [pscustomobject]@{
                id = "target-static"; state = "passed"; command = "cargo check --workspace --locked"
                clean_environment = $false; evidence = New-Reference -Path "artifacts/sprint-9a-closeout/implementation/static.json"
            }
        )
        known_failure_count = 0
        materialization = [pscustomobject]@{
            required = $false
            first_apply = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
            semantic_no_op = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
            recovery = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
        }
        cleanup_restoration = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
        evidence_index = New-Reference -Path "artifacts/sprint-9a-closeout/implementation/evidence-index.json"
    }
    $null = Assert-TessaraImplementationReadinessResult -Result $implementationResult -Contract $contract -ContractPath $contractPath

    $failedImplementation = $implementationResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $failedImplementation.targets[0].state = "failed"
    $failedImplementation.state = "failed"
    $failedImplementation.known_failure_count = 1
    Assert-Throws -Label "known implementation failure" -Action {
        Assert-TessaraImplementationReadinessResult -Result $failedImplementation -Contract $contract -ContractPath $contractPath
    }

    $missingMaterialization = $implementationResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $missingMaterialization.affected_domains = @("deployment-materialization")
    Assert-Throws -Label "missing selected clean materialization target" -Action {
        Assert-TessaraImplementationReadinessResult -Result $missingMaterialization -Contract $contract -ContractPath $contractPath
    }

    $extractionContractPath = Join-Path $temporaryRoot "extraction-validation-contract.json"
    $extractionContract | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $extractionContractPath -Encoding utf8NoBOM
    $extractionContractSha = Get-TessaraValidationSha256 -Path $extractionContractPath
    $extractionImplementation = $implementationResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $extractionImplementation.validation_contract.sha256 = $extractionContractSha
    $extractionImplementation.targets[0].clean_environment = $true
    $extractionImplementation.materialization.required = $true
    foreach ($proofName in @("first_apply", "semantic_no_op", "recovery")) {
        $extractionImplementation.materialization.$proofName.required = $true
        $extractionImplementation.materialization.$proofName.state = "passed"
        $extractionImplementation.materialization.$proofName.evidence = New-Reference -Path "artifacts/sprint-9a-closeout/implementation/$proofName.json"
    }
    $extractionImplementation.cleanup_restoration.required = $true
    $extractionImplementation.cleanup_restoration.state = "passed"
    $extractionImplementation.cleanup_restoration.evidence = New-Reference -Path "artifacts/sprint-9a-closeout/implementation/restoration.json"
    $null = Assert-TessaraImplementationReadinessResult -Result $extractionImplementation -Contract $extractionContract -ContractPath $extractionContractPath

    $extractionMissingRecovery = $extractionImplementation | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $extractionMissingRecovery.materialization.recovery.required = $false
    $extractionMissingRecovery.materialization.recovery.state = "not_applicable"
    $extractionMissingRecovery.materialization.recovery.evidence = $null
    Assert-Throws -Label "Phase 8 extraction missing recovery result" -Action {
        Assert-TessaraImplementationReadinessResult -Result $extractionMissingRecovery -Contract $extractionContract -ContractPath $extractionContractPath
    }

    $fingerprints = @(
        [pscustomobject]@{ domain = "product-source"; sha256 = ("3" * 64) },
        [pscustomobject]@{ domain = "rehearsal-runner"; sha256 = ("4" * 64) }
    )
    $phaseCertificate = [pscustomobject]@{
        schema_version = 1
        contract = "tessara.validation.phase-certificate"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        phase = "candidate-rehearsal"
        attempt = 2
        state = "passed"
        authoritative = $false
        certified_at = "2026-08-10T12:00:00Z"
        source_identity = $source
        environment_fingerprint = ("5" * 64)
        candidate_fingerprint = $null
        prerequisite_certificates = @(New-Reference -Path "artifacts/sprint-9a-closeout/validation-readiness-result.json")
        dependency_fingerprints = $fingerprints
        coverage = [pscustomobject]@{ declared_lanes_sha256 = ("6" * 64); lane_count = 2; executed_count = 1; inherited_count = 1 }
        lanes = @(
            [pscustomobject]@{
                name = "rehearsal-product"; state = "passed"; certification_basis = "executed"
                dependency_domains = @("product-source"); receipt = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-2/product.json"
                started_at = "2026-08-10T11:00:00Z"; ended_at = "2026-08-10T11:01:00Z"; duration_ms = 60000; inheritance = $null
            },
            [pscustomobject]@{
                name = "rehearsal-runner-contract"; state = "passed"; certification_basis = "inherited_nonimpact"
                dependency_domains = @("rehearsal-runner"); receipt = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-1/runner.json"
                started_at = $null; ended_at = $null; duration_ms = $null
                inheritance = [pscustomobject]@{
                    prior_receipt = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-1/runner.json"
                    prior_source_identity = [pscustomobject]@{ commit = ("7" * 40); tree = ("8" * 40); dirty = $false }
                    prior_environment_fingerprint = ("5" * 64)
                    prior_dependency_fingerprints = @([pscustomobject]@{ domain = "rehearsal-runner"; sha256 = ("4" * 64) })
                    nonimpact_rationale = "The runner contract fingerprint is unchanged."
                }
            }
        )
        open_defect_count = 0
        cleanup_restoration = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
        evidence_index = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-2/evidence-index.json"
    }
    $null = Assert-TessaraPhaseCertificate -Certificate $phaseCertificate

    $phaseCertificateV2 = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $phaseCertificateV2.schema_version = 2
    $phaseCertificateV2 | Add-Member -NotePropertyName compatibility_plan `
        -NotePropertyValue (New-Reference `
            -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-2/compatibility-plan.json")
    foreach ($lane in @($phaseCertificateV2.lanes)) {
        $lane | Add-Member -NotePropertyName compatibility_fingerprint -NotePropertyValue ("c" * 64)
    }
    $phaseCertificateV2.lanes[1].inheritance | Add-Member -NotePropertyName prior_compatibility_fingerprint -NotePropertyValue ("c" * 64)
    $phaseCertificateV2.lanes[1].inheritance | Add-Member `
        -NotePropertyName prior_certificate -NotePropertyValue (New-Reference `
            -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-1/candidate-rehearsal-result.json")
    $null = Assert-TessaraJsonSchema -Document $phaseCertificateV2 `
        -Kind phase_certificate_v2 -Label "Phase certificate v2 structure"
    Assert-ThrowsMatching -Label "unauthenticated phase certificate v2" -Action {
        Assert-TessaraPhaseCertificate -Certificate $phaseCertificateV2
    } -MessagePattern "structural validation alone cannot authorize reuse"

    $changedCompatibility = $phaseCertificateV2 | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $changedCompatibility.lanes[1].inheritance.prior_compatibility_fingerprint = ("d" * 64)
    $null = Assert-TessaraJsonSchema -Document $changedCompatibility `
        -Kind phase_certificate_v2 -Label "Changed compatibility v2 structure"

    $changedInheritance = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $changedInheritance.lanes[1].inheritance.prior_dependency_fingerprints[0].sha256 = ("9" * 64)
    Assert-Throws -Label "changed inherited dependency" -Action { Assert-TessaraPhaseCertificate -Certificate $changedInheritance }

    $sitInheritance = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $sitInheritance.phase = "sit"
    $sitInheritance.authoritative = $true
    $sitInheritance.candidate_fingerprint = ("a" * 64)
    Assert-Throws -Label "candidate-bound inherited lane" -Action { Assert-TessaraPhaseCertificate -Certificate $sitInheritance }

    $coldIndex = [pscustomobject]@{
        schema_version = 1; contract = "tessara.validation.phase-evidence-index"; policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"; phase = "candidate-rehearsal"; attempt = 2
        evidence_root = "artifacts/sprint-9a-closeout/rehearsal/attempt-2"
        sealed_at = "2026-08-10T12:00:00Z"
        entry_count = 1
        entries = @([pscustomobject]@{ path = "artifacts/sprint-9a-closeout/rehearsal/attempt-2/cold.log"; sha256 = ("b" * 64); size = 12; kind = "log" })
    }
    $null = Assert-TessaraPhaseEvidenceIndex -Index $coldIndex
    Assert-Throws -Label "explicit audit detects missing cold evidence" -Action {
        Assert-TessaraPhaseEvidenceIndex -Index $coldIndex -RepositoryRoot $repositoryRoot -AuditFiles
    }

    $rawPath = Join-Path $temporaryRoot "raw.log"
    "passing raw evidence" | Set-Content -LiteralPath $rawPath -Encoding utf8NoBOM -NoNewline
    $relativeRaw = [IO.Path]::GetRelativePath($repositoryRoot, $rawPath).Replace("\", "/")
    $realIndex = $coldIndex | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $realIndex.evidence_root = [IO.Path]::GetRelativePath($repositoryRoot, $temporaryRoot).Replace("\", "/")
    $realIndex.entries[0].path = $relativeRaw
    $realIndex.entries[0].sha256 = Get-TessaraValidationSha256 -Path $rawPath
    $realIndex.entries[0].size = (Get-Item -LiteralPath $rawPath).Length
    $null = Assert-TessaraPhaseEvidenceIndex -Index $realIndex -RepositoryRoot $repositoryRoot -AuditFiles
    Add-Content -LiteralPath $rawPath -Value "tamper" -NoNewline
    Assert-Throws -Label "final audit detects raw evidence tamper" -Action {
        Assert-TessaraPhaseEvidenceIndex -Index $realIndex -RepositoryRoot $repositoryRoot -AuditFiles
    }

    $chainIndexPath = Join-Path $temporaryRoot "evidence-index.json"
    $coldIndex | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $chainIndexPath -Encoding utf8NoBOM
    $phaseCertificate.evidence_index = New-Reference `
        -Path ([IO.Path]::GetRelativePath($repositoryRoot, $chainIndexPath).Replace("\", "/")) `
        -Sha (Get-TessaraValidationSha256 -Path $chainIndexPath)
    $certificatePath = Join-Path $temporaryRoot "candidate-rehearsal-result.json"
    $phaseCertificate | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $certificatePath -Encoding utf8NoBOM
    $relativeCertificate = [IO.Path]::GetRelativePath($repositoryRoot, $certificatePath).Replace("\", "/")
    $chain = [pscustomobject]@{
        schema_version = 1; contract = "tessara.validation.evidence-chain"; policy_version = "tessara-validation-v2"; sprint = "sprint-9a"
        certificates = @([pscustomobject]@{ phase = "candidate-rehearsal"; path = $relativeCertificate; sha256 = (Get-TessaraValidationSha256 -Path $certificatePath) })
        corrections = @()
        final_integrity_audit = [pscustomobject]@{ state = "pending"; audited_at = $null; phase_index_count = 0; artifact_count = 0 }
    }
    $null = Assert-TessaraEvidenceChain -Chain $chain -RepositoryRoot $repositoryRoot
    Assert-Throws -Label "closeout rejects pending final audit" -Action {
        Assert-TessaraEvidenceChain -Chain $chain -RepositoryRoot $repositoryRoot -FinalAudit
    }

    "passing raw evidence" | Set-Content -LiteralPath $rawPath -Encoding utf8NoBOM -NoNewline
    $realIndex.entries[0].sha256 = Get-TessaraValidationSha256 -Path $rawPath
    $realIndex.entries[0].size = (Get-Item -LiteralPath $rawPath).Length
    $auditedIndexPath = Join-Path $temporaryRoot "audited-evidence-index.json"
    $realIndex | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $auditedIndexPath -Encoding utf8NoBOM
    $auditedCertificate = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $auditedCertificate.evidence_index = New-Reference `
        -Path ([IO.Path]::GetRelativePath($repositoryRoot, $auditedIndexPath).Replace("\", "/")) `
        -Sha (Get-TessaraValidationSha256 -Path $auditedIndexPath)
    $auditedCertificatePath = Join-Path $temporaryRoot "audited-candidate-rehearsal-result.json"
    $auditedCertificate | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $auditedCertificatePath -Encoding utf8NoBOM
    $auditedChain = [pscustomobject]@{
        schema_version = 1; contract = "tessara.validation.evidence-chain"; policy_version = "tessara-validation-v2"; sprint = "sprint-9a"
        certificates = @([pscustomobject]@{
            phase = "candidate-rehearsal"
            path = [IO.Path]::GetRelativePath($repositoryRoot, $auditedCertificatePath).Replace("\", "/")
            sha256 = Get-TessaraValidationSha256 -Path $auditedCertificatePath
        })
        corrections = @()
        final_integrity_audit = [pscustomobject]@{
            state = "passed"; audited_at = "2026-08-10T13:00:00Z"; phase_index_count = 1; artifact_count = 1
        }
    }
    $null = Assert-TessaraEvidenceChain -Chain $auditedChain -RepositoryRoot $repositoryRoot -FinalAudit
    Add-Content -LiteralPath $rawPath -Value "tamper" -NoNewline
    Assert-Throws -Label "evidence-chain final audit detects raw tamper" -Action {
        Assert-TessaraEvidenceChain -Chain $auditedChain -RepositoryRoot $repositoryRoot -FinalAudit
    }

    $ignoreOutput = & git -C $repositoryRoot check-ignore "artifacts/validation-policy-v2-probe.json" 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace(($ignoreOutput | Out-String))) {
        throw "Generated validation evidence is no longer ignored by Git."
    }

    Write-Output "Tessara validation policy v2 self-tests passed."
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

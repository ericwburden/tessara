Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:Sprint8CRepositoryRoot = Split-Path -Parent $PSScriptRoot
$script:Sprint8CContractPath = Join-Path $script:Sprint8CRepositoryRoot "docs/sprints/sprint-8c-validation-contract.json"
$script:Sprint8CScenarioContractPath = Join-Path $script:Sprint8CRepositoryRoot "docs/sprints/sprint-8c-uat/scenario-contract.json"
$script:Sprint8CPolicyPath = Join-Path $PSScriptRoot "tessara-validation-policy.psm1"
$script:Sprint8CResetAuthorization = "I_AUTHORIZE_THE_EXACT_SPRINT_8C_LANE_DISPOSABLE_TOPOLOGY_RESET"

Import-Module $script:Sprint8CPolicyPath -Force

function Get-Sprint8CContract {
    if (-not (Test-Path -LiteralPath $script:Sprint8CContractPath -PathType Leaf)) {
        throw "Sprint 8C validation contract is missing: $script:Sprint8CContractPath"
    }
    $contract = Get-Content -Raw -LiteralPath $script:Sprint8CContractPath | ConvertFrom-Json -Depth 100
    $null = Assert-TessaraValidationContract -Contract $contract
    if ([string]$contract.sprint -cne "sprint-8c" -or
        [string]$contract.policy_version -cne "tessara-validation-v2") {
        throw "The formal runner loaded a foreign validation contract."
    }
    return $contract
}

function Get-Sprint8CScenarioContract {
    if (-not (Test-Path -LiteralPath $script:Sprint8CScenarioContractPath -PathType Leaf)) {
        throw "Sprint 8C UAT scenario contract is missing: $script:Sprint8CScenarioContractPath"
    }
    $document = Get-Content -Raw -LiteralPath $script:Sprint8CScenarioContractPath | ConvertFrom-Json -Depth 100
    if ([int]$document.schema_version -ne 1 -or
        [string]$document.contract -cne "tessara.sprint-8c.uat-scenarios") {
        throw "Sprint 8C UAT scenario contract has the wrong identity."
    }
    $ids = @($document.scenarios | ForEach-Object { [string]$_.id })
    $expected = 1..11 | ForEach-Object { "UAT-8C-{0:D2}" -f $_ }
    Assert-Sprint8CExactSequence -Expected $expected -Actual $ids -Label "UAT scenario inventory"
    return $document
}

function Assert-Sprint8CExactSequence {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Expected,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Actual,
        [Parameter(Mandatory)][string]$Label
    )
    $expectedText = @($Expected | ForEach-Object { [string]$_ })
    $actualText = @($Actual | ForEach-Object { [string]$_ })
    if ($expectedText.Count -ne $actualText.Count) {
        throw "$Label count mismatch: expected $($expectedText.Count), found $($actualText.Count)."
    }
    for ($index = 0; $index -lt $expectedText.Count; $index++) {
        if ($expectedText[$index] -cne $actualText[$index]) {
            throw "$Label mismatch at index ${index}: expected '$($expectedText[$index])', found '$($actualText[$index])'."
        }
    }
}

function Get-Sprint8CPhaseRunnerName {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { "validate-sprint-8c-readiness.ps1" }
        "candidate-rehearsal" { "run-sprint-8c-candidate-rehearsal.ps1" }
        "validation-preflight" { "run-sprint-8c-validation-preflight.ps1" }
        "sit" { "run-sprint-8c-sit.ps1" }
        "uat" { "run-sprint-8c-formal-uat.ps1" }
        default { throw "Unknown Sprint 8C phase '$Phase'." }
    }
}

function Get-Sprint8CPhaseResultName {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { "validation-readiness-result.json" }
        "candidate-rehearsal" { "candidate-rehearsal-result.json" }
        "validation-preflight" { "preflight-result.json" }
        "sit" { "sit-result.json" }
        "uat" { "uat-result.json" }
        default { throw "Unknown Sprint 8C phase '$Phase'." }
    }
}

function Get-Sprint8CEvidenceRoot {
    param([Parameter(Mandatory)]$Contract)

    $canonicalRelative = ([string]$Contract.evidence_policy.root).TrimEnd("/", "\")
    $override = [Environment]::GetEnvironmentVariable(
        "TESSARA_SPRINT_8C_RUN_EVIDENCE_ROOT",
        "Process"
    )
    $relative = if ([string]::IsNullOrWhiteSpace($override)) {
        $canonicalRelative
    } else {
        $override.Replace("\", "/").TrimEnd("/")
    }
    if ([IO.Path]::IsPathRooted($relative) -or $relative -match '(^|/)\.\.(/|$)') {
        throw "Sprint 8C run evidence root must be a repository-relative path without traversal."
    }

    $full = [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot $relative))
    $repository = [IO.Path]::GetFullPath($script:Sprint8CRepositoryRoot)
    if (-not $full.StartsWith(
            $repository + [IO.Path]::DirectorySeparatorChar,
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Sprint 8C run evidence root escapes the repository."
    }

    $actualRelative = [IO.Path]::GetRelativePath($repository, $full).Replace("\", "/")
    if ($actualRelative -cne $canonicalRelative.Replace("\", "/") -and
        $actualRelative -cnotmatch ('^' + [regex]::Escape($canonicalRelative.Replace("\", "/")) + '/runs/[a-z0-9][a-z0-9._-]{0,79}$')) {
        throw "Sprint 8C run evidence root must be the canonical root or one bounded canonical runs/<identity> child."
    }

    $cursor = $repository
    foreach ($segment in @($actualRelative -split '/')) {
        $cursor = Join-Path $cursor $segment
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Sprint 8C run evidence root traverses a reparse point: $cursor"
            }
        }
    }
    return $full
}

function Get-Sprint8CPhaseRoot {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )
    return Join-Path (Get-Sprint8CEvidenceRoot -Contract $Contract) $Phase
}

function Assert-Sprint8CDefectProvenanceChronology {
    param(
        [Parameter(Mandatory)]$Contract,
        [string]$EvidenceRootPath
    )

    $root = if ([string]::IsNullOrWhiteSpace($EvidenceRootPath)) {
        [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot (
            [string]$Contract.evidence_policy.root
        )))
    } elseif ([IO.Path]::IsPathRooted($EvidenceRootPath)) {
        [IO.Path]::GetFullPath($EvidenceRootPath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot $EvidenceRootPath))
    }
    $null = Get-Sprint8CRepositoryRelativePath -Path $root
    return Assert-TessaraDefectProvenanceChronology `
        -RepositoryRoot $script:Sprint8CRepositoryRoot -EvidenceRoot $root -Sprint "sprint-8c"
}

function Get-Sprint8CLaneEvidencePaths {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $phaseRoot = Get-Sprint8CPhaseRoot -Contract $Contract -Phase $Phase
    $laneRoot = Join-Path $phaseRoot "lanes/$Lane"
    [pscustomobject][ordered]@{
        phase_root = $phaseRoot
        lane_root = $laneRoot
        result = Join-Path $laneRoot "result.json"
        command_log = Join-Path $laneRoot "command.log"
        evidence_references = Join-Path $laneRoot "evidence-references.json"
    }
}

function Get-Sprint8CRepositoryRelativePath {
    param([Parameter(Mandatory)][string]$Path)
    $fullRoot = [IO.Path]::GetFullPath($script:Sprint8CRepositoryRoot)
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith($fullRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Path escapes the Sprint 8C repository: $Path"
    }
    return [IO.Path]::GetRelativePath($fullRoot, $fullPath).Replace("\", "/")
}

function Get-Sprint8CSha256Text {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
    $hash = [Security.Cryptography.SHA256]::HashData($bytes)
    return [Convert]::ToHexString($hash).ToLowerInvariant()
}

function Get-Sprint8CFileReference {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Evidence file is missing: $Path"
    }
    [pscustomobject][ordered]@{
        path = Get-Sprint8CRepositoryRelativePath -Path $Path
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

function Write-Sprint8CNewUtf8File {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [scriptblock]$AfterCreateHook
    )
    $parent = Split-Path -Parent $Path
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    $stream = $null
    $created = $false
    $writeFailure = $null
    try {
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
        $created = $true
        if ($null -ne $AfterCreateHook) { & $AfterCreateHook }
        $stream.Write($bytes, 0, $bytes.Length)
    } catch {
        $writeFailure = $_
    } finally {
        if ($null -ne $stream) { $stream.Dispose() }
    }
    if ($null -ne $writeFailure) {
        if ($created -and (Test-Path -LiteralPath $Path -PathType Leaf)) {
            Remove-Item -LiteralPath $Path -Force
        }
        throw $writeFailure
    }
}

function Write-Sprint8CNewJsonFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document
    )
    $json = $Document | ConvertTo-Json -Depth 100
    Write-Sprint8CNewUtf8File -Path $Path -Text "$json`n"
}

function Publish-Sprint8CJsonAndSidecar {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document,
        [scriptblock]$BeforeSidecarPublicationHook
    )
    foreach ($candidate in @($Path, "$Path.sha256")) {
        if (Test-Path -LiteralPath $candidate) {
            throw "Formal evidence publication will not overwrite an existing file: $candidate"
        }
    }
    $jsonCreated = $false
    $sidecarCreated = $false
    try {
        Write-Sprint8CNewJsonFile -Path $Path -Document $Document
        $jsonCreated = $true
        $sha = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($null -ne $BeforeSidecarPublicationHook) {
            & $BeforeSidecarPublicationHook
        }
        Write-Sprint8CNewUtf8File -Path "$Path.sha256" -Text "$sha`n"
        $sidecarCreated = $true
        return $sha
    } catch {
        if ($sidecarCreated -and (Test-Path -LiteralPath "$Path.sha256" -PathType Leaf)) {
            Remove-Item -LiteralPath "$Path.sha256" -Force
        }
        if ($jsonCreated -and (Test-Path -LiteralPath $Path -PathType Leaf)) {
            Remove-Item -LiteralPath $Path -Force
        }
        throw
    }
}

function Get-Sprint8CJsonDocumentSha256 {
    param([Parameter(Mandatory)]$Document)

    $json = $Document | ConvertTo-Json -Depth 100
    return Get-Sprint8CSha256Text -Text "$json`n"
}

function Add-Sprint8CCreatedPublicationFile {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()]
        [Collections.Generic.List[object]]$CreatedFiles,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedSha256,
        [scriptblock]$AfterRegistrationHook
    )

    $fullPath = [IO.Path]::GetFullPath($Path)
    $CreatedFiles.Add([pscustomobject][ordered]@{
        path = $fullPath
        sha256 = $ExpectedSha256
    })
    if ($null -ne $AfterRegistrationHook) { & $AfterRegistrationHook }
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Newly published formal evidence file is missing: $fullPath"
    }
    $actualSha = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualSha -cne $ExpectedSha256) {
        throw "Newly published formal evidence file does not match its owned bytes: $fullPath"
    }
}

function Remove-Sprint8CCreatedPublicationFilesExact {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()]
        [object[]]$CreatedFiles
    )

    $cleanupFailures = [Collections.Generic.List[string]]::new()
    for ($index = $CreatedFiles.Count - 1; $index -ge 0; $index--) {
        $created = $CreatedFiles[$index]
        $path = [string]$created.path
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
        $actualSha = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actualSha -cne [string]$created.sha256) {
            $cleanupFailures.Add("Refused to remove changed formal publication file '$path'.")
            continue
        }
        Remove-Item -LiteralPath $path -Force
    }
    if ($cleanupFailures.Count -ne 0) {
        throw (@($cleanupFailures) -join " ")
    }
}

function Copy-Sprint8CNewFile {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination,
        [scriptblock]$AfterDestinationCreateHook
    )
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        throw "Publication source is missing: $Source"
    }
    $parent = Split-Path -Parent $Destination
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    if (Test-Path -LiteralPath $Destination) {
        throw "Formal evidence copy will not overwrite an existing file: $Destination"
    }
    $input = [IO.File]::OpenRead($Source)
    $output = $null
    $destinationCreated = $false
    $copyFailure = $null
    try {
        $output = [IO.File]::Open($Destination, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
        $destinationCreated = $true
        if ($null -ne $AfterDestinationCreateHook) {
            & $AfterDestinationCreateHook
        }
        $input.CopyTo($output)
    } catch {
        $copyFailure = $_
    } finally {
        if ($null -ne $output) { $output.Dispose() }
        $input.Dispose()
    }
    if ($null -ne $copyFailure) {
        if ($destinationCreated -and (Test-Path -LiteralPath $Destination -PathType Leaf)) {
            Remove-Item -LiteralPath $Destination -Force
        }
        throw $copyFailure
    }
}

function Assert-Sprint8CPhaseEvidenceSnapshot {
    param(
        [Parameter(Mandatory)]$Index,
        [Parameter(Mandatory)][string[]]$PublicationPaths
    )

    $null = Assert-TessaraPhaseEvidenceIndex -Index $Index `
        -RepositoryRoot $script:Sprint8CRepositoryRoot -AuditFiles
    $evidenceRoot = [IO.Path]::GetFullPath((Join-Path `
        $script:Sprint8CRepositoryRoot ([string]$Index.evidence_root)))
    $null = Get-Sprint8CRepositoryRelativePath -Path $evidenceRoot
    $excluded = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($path in $PublicationPaths) {
        $null = $excluded.Add([IO.Path]::GetFullPath($path))
    }
    $expected = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($entry in @($Index.entries)) {
        $null = $expected.Add([IO.Path]::GetFullPath((Join-Path `
            $script:Sprint8CRepositoryRoot ([string]$entry.path))))
    }
    $actual = @(
        Get-ChildItem -LiteralPath $evidenceRoot -File -Recurse | Where-Object {
            -not $excluded.Contains([IO.Path]::GetFullPath($_.FullName))
        } | ForEach-Object { [IO.Path]::GetFullPath($_.FullName) }
    )
    if ($actual.Count -ne $expected.Count -or
        @($actual | Where-Object { -not $expected.Contains($_) }).Count -ne 0) {
        throw "Formal phase evidence inventory changed during certificate sealing."
    }
}

function Publish-Sprint8CFormalLanePassResult {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Paths,
        [Parameter(Mandatory)][string]$AttemptLog,
        [Parameter(Mandatory)][string]$AttemptReferences,
        [Parameter(Mandatory)]$Result,
        [string]$ChronologyEvidenceRootPath,
        [scriptblock]$BeforeResultPublicationHook,
        [scriptblock]$AfterResultPublicationHook
    )

    $canonicalPaths = @(
        [string]$Paths.command_log,
        [string]$Paths.evidence_references,
        [string]$Paths.result,
        "$([string]$Paths.result).sha256"
    )
    foreach ($path in $canonicalPaths) {
        if (Test-Path -LiteralPath $path) {
            throw "Formal lane pass publication will not overwrite existing evidence: $path"
        }
    }

    $createdFiles = [Collections.Generic.List[object]]::new()
    try {
        $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
            -EvidenceRootPath $ChronologyEvidenceRootPath
        $attemptLogSha = (Get-FileHash -LiteralPath $AttemptLog -Algorithm SHA256).Hash.ToLowerInvariant()
        Copy-Sprint8CNewFile -Source $AttemptLog -Destination $Paths.command_log
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles -Path $Paths.command_log `
            -ExpectedSha256 $attemptLogSha
        $attemptReferencesSha = (
            Get-FileHash -LiteralPath $AttemptReferences -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        Copy-Sprint8CNewFile -Source $AttemptReferences -Destination $Paths.evidence_references
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles `
            -Path $Paths.evidence_references -ExpectedSha256 $attemptReferencesSha
        $Result.command_log = Get-Sprint8CFileReference -Path $Paths.command_log
        $Result.evidence_references = Get-Sprint8CFileReference -Path $Paths.evidence_references
        if ($null -ne $BeforeResultPublicationHook) { & $BeforeResultPublicationHook }
        $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
            -EvidenceRootPath $ChronologyEvidenceRootPath
        $resultSha = Publish-Sprint8CJsonAndSidecar -Path $Paths.result -Document $Result
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles -Path $Paths.result `
            -ExpectedSha256 $resultSha
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles `
            -Path "$($Paths.result).sha256" `
            -ExpectedSha256 (Get-Sprint8CSha256Text -Text "$resultSha`n")
        if ($null -ne $AfterResultPublicationHook) { & $AfterResultPublicationHook }
        $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
            -EvidenceRootPath $ChronologyEvidenceRootPath
        return [pscustomobject][ordered]@{
            result = $Result
            created_files = @($createdFiles)
        }
    } catch {
        $publicationFailure = $_
        try {
            Remove-Sprint8CCreatedPublicationFilesExact -CreatedFiles $createdFiles
        } catch {
            throw "Formal lane pass publication failed: $($publicationFailure.Exception.Message) Exact cleanup also failed: $($_.Exception.Message)"
        }
        throw $publicationFailure
    }
}

function Publish-Sprint8CPhasePassCertificate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$IndexPath,
        [Parameter(Mandatory)]$IndexDocument,
        [Parameter(Mandatory)][string]$ExpectedIndexSha256,
        [Parameter(Mandatory)][string]$CertificatePath,
        [Parameter(Mandatory)]$CertificateDocument,
        [string]$ChronologyEvidenceRootPath,
        [scriptblock]$BeforeCertificatePublicationHook,
        [scriptblock]$AfterCertificatePublicationHook
    )

    $publicationPaths = @(
        $IndexPath, "$IndexPath.sha256", $CertificatePath, "$CertificatePath.sha256"
    )
    foreach ($path in $publicationPaths) {
        if (Test-Path -LiteralPath $path) {
            throw "Formal phase pass publication will not overwrite existing evidence: $path"
        }
    }

    $createdFiles = [Collections.Generic.List[object]]::new()
    try {
        $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
            -EvidenceRootPath $ChronologyEvidenceRootPath
        Assert-Sprint8CPhaseEvidenceSnapshot -Index $IndexDocument `
            -PublicationPaths $publicationPaths
        $publishedIndexSha = Publish-Sprint8CJsonAndSidecar `
            -Path $IndexPath -Document $IndexDocument
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles -Path $IndexPath `
            -ExpectedSha256 $publishedIndexSha
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles -Path "$IndexPath.sha256" `
            -ExpectedSha256 (Get-Sprint8CSha256Text -Text "$publishedIndexSha`n")
        if ($publishedIndexSha -cne $ExpectedIndexSha256) {
            throw "Published formal phase evidence index changed during finalization."
        }
        if ($null -ne $BeforeCertificatePublicationHook) {
            & $BeforeCertificatePublicationHook
        }
        $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
            -EvidenceRootPath $ChronologyEvidenceRootPath
        Assert-Sprint8CPhaseEvidenceSnapshot -Index $IndexDocument `
            -PublicationPaths $publicationPaths
        $certificateSha = Publish-Sprint8CJsonAndSidecar `
            -Path $CertificatePath -Document $CertificateDocument
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles -Path $CertificatePath `
            -ExpectedSha256 $certificateSha
        Add-Sprint8CCreatedPublicationFile -CreatedFiles $createdFiles `
            -Path "$CertificatePath.sha256" `
            -ExpectedSha256 (Get-Sprint8CSha256Text -Text "$certificateSha`n")
        if ($null -ne $AfterCertificatePublicationHook) {
            & $AfterCertificatePublicationHook
        }
        $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
            -EvidenceRootPath $ChronologyEvidenceRootPath
        Assert-Sprint8CPhaseEvidenceSnapshot -Index $IndexDocument `
            -PublicationPaths $publicationPaths
        return $CertificateDocument
    } catch {
        $publicationFailure = $_
        try {
            Remove-Sprint8CCreatedPublicationFilesExact -CreatedFiles $createdFiles
        } catch {
            throw "Formal phase pass publication failed: $($publicationFailure.Exception.Message) Exact cleanup also failed: $($_.Exception.Message)"
        }
        throw $publicationFailure
    }
}

function Invoke-Sprint8CLanePassTransaction {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Paths,
        [Parameter(Mandatory)][string]$AttemptLog,
        [Parameter(Mandatory)][string]$AttemptReferences,
        [Parameter(Mandatory)]$Result,
        [AllowNull()][scriptblock]$PhaseCompletionAction,
        [string]$ChronologyEvidenceRootPath,
        [scriptblock]$BeforeResultPublicationHook,
        [scriptblock]$AfterResultPublicationHook
    )

    $publication = $null
    try {
        $publication = Publish-Sprint8CFormalLanePassResult -Contract $Contract `
            -Paths $Paths -AttemptLog $AttemptLog -AttemptReferences $AttemptReferences `
            -Result $Result -ChronologyEvidenceRootPath $ChronologyEvidenceRootPath `
            -BeforeResultPublicationHook $BeforeResultPublicationHook `
            -AfterResultPublicationHook $AfterResultPublicationHook
        if ($null -ne $PhaseCompletionAction) {
            $null = & $PhaseCompletionAction
        }
        return $publication.result
    } catch {
        $transactionFailure = $_
        if ($null -ne $publication) {
            try {
                Remove-Sprint8CCreatedPublicationFilesExact `
                    -CreatedFiles @($publication.created_files)
            } catch {
                throw "Formal lane pass transaction failed: $($transactionFailure.Exception.Message) Exact lane-result rollback also failed: $($_.Exception.Message)"
            }
        }
        throw $transactionFailure
    }
}

function Complete-Sprint8CFailedLaneAttempt {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)]$LaneError,
        [Parameter(Mandatory)]$LaneContract,
        [AllowNull()]$TopologyContext,
        [AllowNull()][string]$Project,
        [Parameter(Mandatory)]$Prerequisites,
        [Parameter(Mandatory)]$EnvironmentBefore,
        [scriptblock]$TopologyEnvironmentAction,
        [scriptblock]$TeardownAction,
        [scriptblock]$RestoreEnvironmentAction
    )

    $emergencyCleanup = $null
    if ([bool]$LaneContract.touches_live_state -and $null -ne $TopologyContext) {
        try {
            if ($null -ne $TopologyEnvironmentAction) {
                & $TopologyEnvironmentAction
            } else {
                Set-Sprint8CTopologyEnvironment -Context $TopologyContext `
                    -ExpectedProject $Project `
                    -CandidateFingerprint ([string]$Prerequisites.candidate_fingerprint)
            }
            $emergencyPath = Join-Path $AttemptRoot "emergency-teardown.json"
            if ($null -ne $TeardownAction) {
                & $TeardownAction $emergencyPath
            } else {
                $null = Invoke-Sprint8CComposeTeardown -Project $Project `
                    -EvidencePath $emergencyPath
            }
            $emergencyCleanup = [ordered]@{
                state = "passed"
                evidence = Get-Sprint8CFileReference -Path $emergencyPath
            }
        } catch {
            $emergencyCleanup = [ordered]@{
                state = "failed"
                message = $_.Exception.Message
            }
        } finally {
            if ($null -ne $RestoreEnvironmentAction) {
                & $RestoreEnvironmentAction
            } else {
                Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $EnvironmentBefore
            }
        }
    }
    $failure = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-attempt-failure"
        sprint = "sprint-8c"
        phase = $Phase
        lane = $Lane
        state = "failed"
        failed_at = [DateTimeOffset]::UtcNow.ToString("O")
        message = $LaneError.Exception.Message
        emergency_cleanup = $emergencyCleanup
    }
    $failurePath = Join-Path $AttemptRoot "failure.json"
    Write-Sprint8CNewJsonFile -Path $failurePath -Document $failure
    return [pscustomobject][ordered]@{
        failure = $failure
        failure_reference = Get-Sprint8CFileReference -Path $failurePath
        emergency_cleanup = $emergencyCleanup
    }
}

function Get-Sprint8CSourceIdentity {
    $commit = (& git -C $script:Sprint8CRepositoryRoot rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Unable to resolve the Sprint 8C source commit."
    }
    $tree = (& git -C $script:Sprint8CRepositoryRoot rev-parse 'HEAD^{tree}').Trim()
    if ($LASTEXITCODE -ne 0 -or $tree -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Unable to resolve the Sprint 8C source tree."
    }
    $branch = (& git -C $script:Sprint8CRepositoryRoot branch --show-current).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($branch)) {
        throw "Unable to resolve the Sprint 8C branch."
    }
    $status = @(& git -C $script:Sprint8CRepositoryRoot status --short --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "Unable to resolve the Sprint 8C source status." }
    [pscustomobject][ordered]@{
        commit = $commit
        tree = $tree
        dirty = @($status).Count -gt 0
        branch = $branch
    }
}

function Assert-Sprint8CCleanSource {
    param([Parameter(Mandatory)]$Source)
    if ([bool]$Source.dirty) {
        throw "Formal Sprint 8C validation refuses a dirty source tree."
    }
}

function Get-Sprint8CProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][string[]]$Names)
    $snapshot = [ordered]@{}
    foreach ($name in $Names) {
        $snapshot[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
    }
    return $snapshot
}

function Restore-Sprint8CProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)]$Snapshot)
    foreach ($name in @($Snapshot.Keys)) {
        [Environment]::SetEnvironmentVariable([string]$name, $Snapshot[$name], "Process")
    }
}

function Assert-Sprint8CSha256Sidecar {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedSha256
    )
    $sidecar = "$Path.sha256"
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecar -PathType Leaf)) {
        throw "Authenticated evidence pair is missing: $Path"
    }
    $sidecarSha = (Get-Content -LiteralPath $sidecar -Raw).Trim()
    $actualSha = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sidecarSha -cnotmatch '^[0-9a-f]{64}$' -or $sidecarSha -cne $actualSha -or
        (-not [string]::IsNullOrWhiteSpace($ExpectedSha256) -and $actualSha -cne $ExpectedSha256)) {
        throw "Evidence sidecar authentication failed: $Path"
    }
    return $actualSha
}

function Resolve-Sprint8CRepositoryPath {
    param([Parameter(Mandatory)][string]$Path)
    $full = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot $Path))
    }
    $null = Get-Sprint8CRepositoryRelativePath -Path $full
    return $full
}

function Get-Sprint8CEnvironmentFingerprintFromDocument {
    param([Parameter(Mandatory)]$Document)
    if ($Document.PSObject.Properties.Name -contains "environment_fingerprint_sha256") {
        return [string]$Document.environment_fingerprint_sha256
    }
    if ($Document.PSObject.Properties.Name -contains "environment_fingerprint") {
        return [string]$Document.environment_fingerprint
    }
    if ($Document.PSObject.Properties.Name -contains "environment" -and $null -ne $Document.environment) {
        if ($Document.environment.PSObject.Properties.Name -contains "fingerprint_sha256") {
            return [string]$Document.environment.fingerprint_sha256
        }
        if ($Document.environment.PSObject.Properties.Name -contains "fingerprint") {
            return [string]$Document.environment.fingerprint
        }
    }
    return ""
}

function Assert-Sprint8CTopologyContext {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$ExpectedProject,
        [AllowNull()][string]$CandidateFingerprint
    )
    if ([string]$Context.compose_project -cne $ExpectedProject -or
        [string]$Context.environment.COMPOSE_PROJECT_NAME -cne $ExpectedProject) {
        throw "Retained topology context does not bind Compose project '$ExpectedProject'."
    }
    if (-not [string]::IsNullOrWhiteSpace($CandidateFingerprint) -and
        [string]$Context.candidate_fingerprint -cne $CandidateFingerprint) {
        throw "Retained topology context is not bound to the frozen candidate."
    }
    $fingerprint = [string]$Context.environment_fingerprint_sha256
    if ($fingerprint -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Context.environment.fingerprint_sha256 -cne $fingerprint) {
        throw "Retained topology context omitted its secret-free environment fingerprint."
    }
    $ports = @(
        [string]$Context.environment.TESSARA_GATEWAY_PORT,
        [string]$Context.environment.TESSARA_CORE_CONTROL_PORT,
        [string]$Context.environment.TESSARA_SUPERVISOR_PORT
    )
    foreach ($port in $ports) {
        $parsed = 0
        if (-not [int]::TryParse($port, [ref]$parsed) -or $parsed -lt 1 -or $parsed -gt 65535) {
            throw "Retained topology context contains an invalid loopback port."
        }
    }
    if (@($ports | Sort-Object -Unique).Count -ne $ports.Count) {
        throw "Retained topology context reuses a loopback port."
    }
    $fixturePath = Resolve-Sprint8CRepositoryPath -Path ([string]$Context.fixture_receipt.path)
    $fixtureSha = Assert-Sprint8CSha256Sidecar -Path $fixturePath `
        -ExpectedSha256 ([string]$Context.fixture_receipt.sha256)
    $fixture = Read-Sprint8CAuthenticatedJson -Path $fixturePath -ExpectedSha256 $fixtureSha
    if ([string]$fixture.sprint -cne "sprint-8c" -or [string]$fixture.state -cne "passed" -or
        [string]$fixture.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$fixture.compose_project -cne $ExpectedProject -or
        [string]$fixture.restoration.state -cne "passed") {
        throw "Retained topology context has an invalid fixture/restoration receipt."
    }
    return $Context
}

function Get-Sprint8CTopologyContextFromHarness {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$ExpectedProject,
        [AllowNull()][string]$CandidateFingerprint
    )
    if ($null -eq $Document.environment -or
        [string]$Document.cleanup_restoration.mode -cne "retained-for-caller") {
        return $null
    }
    $fixturePath = Resolve-Sprint8CRepositoryPath -Path ([string]$Document.fixture_receipt_path)
    $fixtureSha = Assert-Sprint8CSha256Sidecar -Path $fixturePath `
        -ExpectedSha256 ([string]$Document.fixture_receipt_sha256)
    $fingerprint = Get-Sprint8CEnvironmentFingerprintFromDocument -Document $Document
    $context = [pscustomobject][ordered]@{
        compose_project = $ExpectedProject
        candidate_fingerprint = if ([string]::IsNullOrWhiteSpace($CandidateFingerprint)) { $null } else { $CandidateFingerprint }
        environment_fingerprint_sha256 = $fingerprint
        environment = [pscustomobject][ordered]@{
            COMPOSE_PROJECT_NAME = [string]$Document.environment.COMPOSE_PROJECT_NAME
            TESSARA_GATEWAY_PORT = [string]$Document.environment.TESSARA_GATEWAY_PORT
            TESSARA_CORE_CONTROL_PORT = [string]$Document.environment.TESSARA_CORE_CONTROL_PORT
            TESSARA_SUPERVISOR_PORT = [string]$Document.environment.TESSARA_SUPERVISOR_PORT
            fingerprint_sha256 = [string]$Document.environment.fingerprint_sha256
        }
        fixture_receipt = [pscustomobject][ordered]@{
            path = Get-Sprint8CRepositoryRelativePath -Path $fixturePath
            sha256 = $fixtureSha
        }
    }
    return Assert-Sprint8CTopologyContext -Context $context -ExpectedProject $ExpectedProject `
        -CandidateFingerprint $CandidateFingerprint
}

function Set-Sprint8CTopologyEnvironment {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$ExpectedProject,
        [AllowNull()][string]$CandidateFingerprint
    )
    $null = Assert-Sprint8CTopologyContext -Context $Context -ExpectedProject $ExpectedProject `
        -CandidateFingerprint $CandidateFingerprint
    foreach ($name in @(
        "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
        "TESSARA_SUPERVISOR_PORT"
    )) {
        [Environment]::SetEnvironmentVariable($name, [string]$Context.environment.$name, "Process")
    }
    [Environment]::SetEnvironmentVariable(
        "PLAYWRIGHT_BASE_URL", "http://127.0.0.1:$([string]$Context.environment.TESSARA_GATEWAY_PORT)", "Process"
    )
    [Environment]::SetEnvironmentVariable("TESSARA_PLAYWRIGHT_ACCEPTANCE", "1", "Process")

    $container = @(& docker compose -f deploy/sprint-8c/compose.yaml -p $ExpectedProject ps -q postgres)
    if ($LASTEXITCODE -ne 0 -or $container.Count -ne 1 -or
        [string]$container[0] -cnotmatch '^[0-9a-f]{12,64}$') {
        throw "Retained topology '$ExpectedProject' does not expose one exact PostgreSQL container."
    }
    $containerId = [string](& docker inspect --format "{{.Id}}" ([string]$container[0]))
    if ($LASTEXITCODE -ne 0 -or $containerId -cnotmatch '^[0-9a-f]{64}$') {
        throw "Retained topology '$ExpectedProject' PostgreSQL identity is not a full immutable container ID."
    }
    [Environment]::SetEnvironmentVariable("PLAYWRIGHT_POSTGRES_CONTAINER", $containerId, "Process")
    [Environment]::SetEnvironmentVariable("PLAYWRIGHT_POSTGRES_DATABASE", "tessara_core", "Process")
    [Environment]::SetEnvironmentVariable("PLAYWRIGHT_POSTGRES_USER", "tessara_bootstrap", "Process")
}

function New-Sprint8CAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [ValidateSet("program", "pwsh", "internal", "teardown", "manual")][string]$Kind,
        [string]$Command,
        [string[]]$Arguments = @(),
        [switch]$ProducesEvidence,
        [switch]$ProvidesEnvironment,
        [switch]$ProvidesRestoration,
        [ValidateSet("", "fresh", "upgraded")][string]$PlaywrightDataState = "",
        [string[]]$Scenarios = @()
    )
    [pscustomobject][ordered]@{
        id = $Id
        kind = $Kind
        command = $Command
        arguments = @($Arguments)
        produces_evidence = [bool]$ProducesEvidence
        provides_environment = [bool]$ProvidesEnvironment
        provides_restoration = [bool]$ProvidesRestoration
        playwright_data_state = $PlaywrightDataState
        scenarios = @($Scenarios)
    }
}

function New-Sprint8CPowerShellAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Script,
        [string[]]$Arguments = @(),
        [switch]$ProducesEvidence,
        [switch]$ProvidesEnvironment,
        [switch]$ProvidesRestoration,
        [ValidateSet("", "fresh", "upgraded")][string]$PlaywrightDataState = ""
    )
    New-Sprint8CAction -Id $Id -Kind pwsh -Command $Script -Arguments $Arguments `
        -ProducesEvidence:$ProducesEvidence -ProvidesEnvironment:$ProvidesEnvironment `
        -ProvidesRestoration:$ProvidesRestoration -PlaywrightDataState $PlaywrightDataState
}

function New-Sprint8CProgramAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][string[]]$Arguments,
        [ValidateSet("", "fresh", "upgraded")][string]$PlaywrightDataState = ""
    )
    New-Sprint8CAction -Id $Id -Kind program -Command $Program -Arguments $Arguments `
        -PlaywrightDataState $PlaywrightDataState
}

function New-Sprint8CBrowserAcceptanceActions {
    param([ValidateSet("fresh", "upgraded")][string]$PlaywrightDataState = "fresh")
    @(
        New-Sprint8CPowerShellAction -Id "browser-acceptance-batches" `
            -Script "scripts/tessara-validation-evidence-browser-batches.ps1" `
            -Arguments @(
                "-ManifestPath", "end2end/acceptance-manifest.json",
                "-ExpectedDataState", $PlaywrightDataState,
                "-EvidencePath", "{evidence}"
            ) `
            -ProducesEvidence -PlaywrightDataState $PlaywrightDataState
    )
}

function New-Sprint8CMaterializeAction {
    param(
        [string]$Id = "materialize",
        [ValidateSet("CoreFresh", "DatasetBootstrap", "Reference", "ReferenceNoOp", "All")][string]$Target = "Reference",
        [switch]$KeepTopology
    )
    if ($Target -in @("CoreFresh", "DatasetBootstrap", "All")) {
        if ($KeepTopology) { throw "Focused materialization target '$Target' cannot retain a Compose topology." }
        return New-Sprint8CPowerShellAction -Id $Id -Script "scripts/materialize-sprint-8c.ps1" `
            -Arguments @("-Target", $Target, "-EvidencePath", "{evidence}") `
            -ProducesEvidence -ProvidesRestoration
    }
    $arguments = @(
        "-Target", $Target,
        "-ComposeProject", "{project}",
        "-EvidencePath", "{evidence}",
        "-AuthorizeDisposableReset"
    )
    if ($KeepTopology) { $arguments += "-KeepTopology" }
    New-Sprint8CPowerShellAction -Id $Id -Script "scripts/materialize-sprint-8c.ps1" `
        -Arguments $arguments -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration:(-not $KeepTopology)
}

function New-Sprint8CSmokeAction {
    param([string]$Id = "deployed-smoke", [switch]$UseExistingTopology)
    $arguments = @(
        "-ComposeProject", "{project}",
        "-EvidencePath", "{evidence}"
    )
    if ($UseExistingTopology) {
        $arguments += @("-UseExistingTopology", "-FixtureReceiptPath", "{fixture}")
    } else {
        $arguments += "-AuthorizeDisposableReset"
    }
    New-Sprint8CPowerShellAction -Id $Id -Script "scripts/run-sprint-8c-deployed-smoke.ps1" `
        -Arguments $arguments -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function New-Sprint8CUatAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Scenario,
        [switch]$UseExistingTopology
    )
    $arguments = @(
        "-Scenario", $Scenario,
        "-ComposeProject", "{project}",
        "-EvidencePath", "{evidence}"
    )
    if ($UseExistingTopology) {
        $arguments += @("-UseExistingTopology", "-FixtureReceiptPath", "{fixture}")
    } else {
        $arguments += "-AuthorizeDisposableReset"
    }
    New-Sprint8CPowerShellAction -Id $Id -Script "scripts/uat-sprint-8c.ps1" `
        -Arguments $arguments -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function New-Sprint8CFailureAction {
    param([string]$Id = "failure-containment")
    New-Sprint8CPowerShellAction -Id $Id -Script "scripts/run-sprint-8c-failure-containment.ps1" `
        -Arguments @("-ComposeProject", "{project}", "-EvidencePath", "{evidence}", "-AuthorizeDisposableReset") `
        -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function New-Sprint8CUpgradeAction {
    param([string]$Id = "response-upgrade")
    New-Sprint8CPowerShellAction -Id $Id -Script "scripts/run-sprint-8c-response-upgrade.ps1" `
        -Arguments @("-ComposeProject", "{project}", "-EvidencePath", "{evidence}", "-AuthorizeDisposableReset") `
        -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function Get-Sprint8CStaticActions {
    @(
        New-Sprint8CProgramAction -Id "cargo-fmt" -Program "cargo" -Arguments @("fmt", "--all", "--", "--check")
        New-Sprint8CProgramAction -Id "cargo-check" -Program "cargo" -Arguments @("check", "--workspace", "--all-targets", "--all-features", "--locked", "--offline", "--jobs", "1")
        New-Sprint8CProgramAction -Id "cargo-clippy" -Program "cargo" -Arguments @("clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--offline", "--jobs", "1", "--", "-D", "warnings")
        New-Sprint8CPowerShellAction -Id "web-boundaries" -Script "scripts/check-web-crate-boundaries.ps1"
        New-Sprint8CPowerShellAction -Id "sdk-boundaries" -Script "scripts/verify-module-sdk-boundaries.ps1"
        New-Sprint8CPowerShellAction -Id "ui-conformance" -Script "scripts/ui-sdk-conformance.ps1"
        New-Sprint8CPowerShellAction -Id "asset-identity" -Script "scripts/build-module-ui-browser-assets.ps1" -Arguments @("-Module", "all", "-Check")
    )
}

function Get-Sprint8CFormalActionMap {
    $map = [ordered]@{}

    $map["readiness-contract"] = @(
        New-Sprint8CAction -Id "implementation-gate" -Kind internal -Command "implementation-gate"
        New-Sprint8CPowerShellAction -Id "planning-alignment" -Script "scripts/assert-sprint-8c-planning-contract.ps1"
        New-Sprint8CPowerShellAction -Id "policy-selftest" -Script "scripts/test-tessara-validation-policy.ps1" -Arguments @("-SelfTest")
        New-Sprint8CPowerShellAction -Id "implementation-runner-selftest" -Script "scripts/run-sprint-8c-implementation-readiness.ps1" -Arguments @("-SelfTest")
        New-Sprint8CPowerShellAction -Id "acceptance-contract-selftest" -Script "scripts/sprint-8c-acceptance-contract.ps1" -Arguments @("-SelfTest")
    )
    $map["readiness-materialization"] = @(
        New-Sprint8CMaterializeAction -Id "focused-owner-materialization" -Target All
        New-Sprint8CMaterializeAction -Id "reference-materialization" -Target ReferenceNoOp
    )
    $map["readiness-acceptance"] = @(
        New-Sprint8CPowerShellAction -Id "acceptance-contract" -Script "scripts/sprint-8c-acceptance-contract.ps1" -Arguments @("-EvidencePath", "{evidence}") -ProducesEvidence
        New-Sprint8CPowerShellAction -Id "uat-runner-selftest" -Script "scripts/uat-sprint-8c.ps1" -Arguments @("-SelfTest")
        New-Sprint8CPowerShellAction -Id "browser-inventory" -Script "scripts/validate-e2e.ps1" -Arguments @("-InventoryOnly", "-EvidencePath", "{evidence}") -ProducesEvidence
    )

    $map["rehearsal-static"] = @(Get-Sprint8CStaticActions) + @(
        New-Sprint8CPowerShellAction -Id "planning-alignment" -Script "scripts/assert-sprint-8c-planning-contract.ps1"
    )
    $map["rehearsal-rust"] = @(
        New-Sprint8CMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8CPowerShellAction -Id "workspace-rust" `
            -Script "scripts/run-sprint-8c-implementation-readiness.ps1" `
            -Arguments @("-WorkspaceTestOnly")
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-materialization"] = @(
        New-Sprint8CMaterializeAction -Id "focused-owner-materialization" -Target All
        New-Sprint8CMaterializeAction -Id "reference-materialization" -Target ReferenceNoOp
    )
    $map["rehearsal-browser"] = @(
        New-Sprint8CMaterializeAction -Id "setup" -Target Reference -KeepTopology
        @(New-Sprint8CBrowserAcceptanceActions -PlaywrightDataState fresh)
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-conformance"] = @(
        New-Sprint8CMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8CPowerShellAction -Id "response-boundary" -Script "scripts/check-sprint-8c-response-boundaries.ps1" -Arguments @("-Mode", "RequireClean")
        New-Sprint8CPowerShellAction -Id "ui-conformance" -Script "scripts/ui-sdk-conformance.ps1"
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-providers"] = @(
        New-Sprint8CMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8CPowerShellAction -Id "response-provider-contracts" -Script "scripts/test-sprint-8c-response-module.ps1" -Arguments @("-Suite", "Provider")
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-events"] = @(
        New-Sprint8CMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8CPowerShellAction -Id "workflow-events" -Script "scripts/test-sprint-8c-workflow-events.ps1"
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-consumers"] = @(
        New-Sprint8CMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8CPowerShellAction -Id "dataset-export" -Script "scripts/test-sprint-8c-dataset-export-contract.ps1"
        New-Sprint8CPowerShellAction -Id "component-consumer" -Script "scripts/test-sprint-8c-component-consumer.ps1"
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-smoke"] = @(New-Sprint8CSmokeAction -Id "rehearsal-smoke")
    $map["rehearsal-recovery"] = @(New-Sprint8CFailureAction -Id "rehearsal-recovery")
    $map["rehearsal-upgrade"] = @(New-Sprint8CUpgradeAction -Id "rehearsal-upgrade")
    $map["rehearsal-uat"] = @(New-Sprint8CUatAction -Id "rehearsal-uat" -Scenario All)

    $map["preflight-freeze"] = @(New-Sprint8CAction -Id "freeze-candidate" -Kind internal -Command "freeze-candidate")

    $map["sit-static"] = @(Get-Sprint8CStaticActions) + @(
        New-Sprint8CPowerShellAction -Id "response-boundary" -Script "scripts/check-sprint-8c-response-boundaries.ps1" -Arguments @("-Mode", "RequireClean")
    )
    $map["sit-rust"] = @(
        New-Sprint8CMaterializeAction -Id "frozen-sit-setup" -Target Reference -KeepTopology
        New-Sprint8CPowerShellAction -Id "workspace-rust" `
            -Script "scripts/run-sprint-8c-implementation-readiness.ps1" `
            -Arguments @("-WorkspaceTestOnly")
        New-Sprint8CSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8CAction -Id "post-smoke-teardown" -Kind teardown -Command "compose-down"
        New-Sprint8CMaterializeAction -Id "fresh-browser-handoff" -Target Reference -KeepTopology
    )
    $map["sit-browser"] = @(
        @(New-Sprint8CBrowserAcceptanceActions -PlaywrightDataState fresh)
        New-Sprint8CSmokeAction -Id "browser-restoration" -UseExistingTopology
    )
    $map["sit-smoke"] = @(
        New-Sprint8CSmokeAction -Id "sit-smoke" -UseExistingTopology
        New-Sprint8CAction -Id "teardown" -Kind teardown -Command "compose-down"
    )

    $map["uat-scripted"] = @(New-Sprint8CUatAction -Id "uat-scripted" -Scenario All)
    $map["uat-product-ui"] = @(
        New-Sprint8CUatAction -Id "uat-8c-01" -Scenario "UAT-8C-01"
        New-Sprint8CAction -Id "manual-product-ui" -Kind manual -Scenarios @("UAT-8C-01")
    )
    $map["uat-assignment-authorization"] = @(
        New-Sprint8CUatAction -Id "uat-8c-02" -Scenario "UAT-8C-02"
        New-Sprint8CUatAction -Id "uat-8c-03" -Scenario "UAT-8C-03"
        New-Sprint8CAction -Id "manual-assignment-authorization" -Kind manual -Scenarios @("UAT-8C-02", "UAT-8C-03")
    )
    $map["uat-providers"] = @(
        New-Sprint8CUatAction -Id "uat-8c-06" -Scenario "UAT-8C-06"
        New-Sprint8CAction -Id "manual-providers" -Kind manual -Scenarios @("UAT-8C-06")
    )
    $map["uat-consumers"] = @(
        New-Sprint8CUatAction -Id "uat-8c-04" -Scenario "UAT-8C-04"
        New-Sprint8CAction -Id "manual-consumers" -Kind manual -Scenarios @("UAT-8C-04")
    )
    $map["uat-materialization"] = @(
        New-Sprint8CMaterializeAction -Id "uat-materialization-proof" -Target ReferenceNoOp
        New-Sprint8CUatAction -Id "uat-8c-07" -Scenario "UAT-8C-07"
        New-Sprint8CAction -Id "manual-materialization" -Kind manual -Scenarios @("UAT-8C-07")
    )
    $map["uat-owner-isolation"] = @(
        New-Sprint8CUatAction -Id "uat-8c-08" -Scenario "UAT-8C-08"
        New-Sprint8CAction -Id "manual-owner-isolation" -Kind manual -Scenarios @("UAT-8C-08")
    )
    $map["uat-replay-outage"] = @(
        New-Sprint8CUatAction -Id "uat-8c-05" -Scenario "UAT-8C-05"
        New-Sprint8CAction -Id "manual-replay-outage" -Kind manual -Scenarios @("UAT-8C-05")
    )
    $map["uat-recovery"] = @(
        New-Sprint8CFailureAction -Id "uat-recovery-proof"
        New-Sprint8CUatAction -Id "uat-8c-09" -Scenario "UAT-8C-09"
        New-Sprint8CAction -Id "manual-recovery" -Kind manual -Scenarios @("UAT-8C-09")
    )
    $map["uat-upgrade"] = @(
        New-Sprint8CUpgradeAction -Id "uat-upgrade-proof"
        New-Sprint8CUatAction -Id "uat-8c-10" -Scenario "UAT-8C-10"
        New-Sprint8CAction -Id "manual-upgrade" -Kind manual -Scenarios @("UAT-8C-10")
    )
    $map["uat-roadmap-exit"] = @(
        New-Sprint8CUatAction -Id "uat-8c-11" -Scenario "UAT-8C-11"
        New-Sprint8CAction -Id "manual-roadmap-exit" -Kind manual -Scenarios @("UAT-8C-11")
    )

    return $map
}

function Get-Sprint8CExpectedLaneIds {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { @("readiness-contract", "readiness-materialization", "readiness-acceptance") }
        "candidate-rehearsal" { @("rehearsal-static", "rehearsal-rust", "rehearsal-materialization", "rehearsal-browser", "rehearsal-conformance", "rehearsal-providers", "rehearsal-events", "rehearsal-consumers", "rehearsal-smoke", "rehearsal-recovery", "rehearsal-upgrade", "rehearsal-uat") }
        "validation-preflight" { @("preflight-freeze") }
        "sit" { @("sit-static", "sit-rust", "sit-browser", "sit-smoke") }
        "uat" { @("uat-scripted", "uat-product-ui", "uat-assignment-authorization", "uat-providers", "uat-consumers", "uat-materialization", "uat-owner-isolation", "uat-replay-outage", "uat-recovery", "uat-upgrade", "uat-roadmap-exit") }
        default { throw "Unknown Sprint 8C phase '$Phase'." }
    }
}

function Get-Sprint8CExpectedLiveLaneIds {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { @("readiness-materialization") }
        "candidate-rehearsal" { @(
            "rehearsal-rust", "rehearsal-materialization", "rehearsal-browser",
            "rehearsal-conformance", "rehearsal-providers", "rehearsal-events",
            "rehearsal-consumers", "rehearsal-smoke", "rehearsal-recovery",
            "rehearsal-upgrade", "rehearsal-uat"
        ) }
        "validation-preflight" { @() }
        "sit" { @("sit-rust", "sit-browser", "sit-smoke") }
        "uat" { @(Get-Sprint8CExpectedLaneIds -Phase "uat") }
        default { throw "Unknown Sprint 8C phase '$Phase'." }
    }
}

function Get-Sprint8CExpectedProject {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$LaneContract
    )
    if (-not [bool]$LaneContract.touches_live_state) { return $null }
    if ($Phase -ceq "sit") { return "tessara-s8c-sit" }
    return "tessara-s8c-$([string]$LaneContract.id)"
}

function Get-Sprint8CSelector {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $runner = Get-Sprint8CPhaseRunnerName -Phase $Phase
    if ($Phase -ceq "validation-preflight") {
        return "pwsh -NoProfile -File .\scripts\$runner"
    }
    return "pwsh -NoProfile -File .\scripts\$runner -Lane $Lane"
}

function Assert-Sprint8CFormalProfile {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$ActionMap
    )
    if ([string]$Contract.evidence_policy.root -cne "artifacts/sprint-8c-closeout") {
        throw "Sprint 8C formal evidence root is not canonical."
    }
    $expectedIds = @(Get-Sprint8CExpectedLaneIds -Phase $Phase)
    $expectedLiveIds = @(Get-Sprint8CExpectedLiveLaneIds -Phase $Phase)
    $phaseLanes = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase })
    Assert-Sprint8CExactSequence -Expected $expectedIds -Actual @($phaseLanes.id) -Label "$Phase lane order"

    $seenActions = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    for ($index = 0; $index -lt $phaseLanes.Count; $index++) {
        $lane = $phaseLanes[$index]
        $id = [string]$lane.id
        $expectedPrerequisite = @(if ($index -eq 0) {
            switch ($Phase) {
                "validation-readiness" { @() }
                "candidate-rehearsal" { @("readiness-acceptance") }
                "validation-preflight" { @("rehearsal-uat") }
                "sit" { @("preflight-freeze") }
                "uat" { @("sit-smoke") }
            }
        } else { [string]$phaseLanes[$index - 1].id })
        Assert-Sprint8CExactSequence -Expected $expectedPrerequisite -Actual @($lane.prerequisites) -Label "$id prerequisites"

        $project = Get-Sprint8CExpectedProject -Phase $Phase -LaneContract $lane
        $expectedLive = $expectedLiveIds -ccontains $id
        if ([bool]$lane.touches_live_state -ne $expectedLive) {
            throw "Lane '$id' live-state identity does not match the approved Sprint 8C profile."
        }
        if ([bool]$lane.touches_live_state) {
            if ($project -cnotmatch '^tessara-s8c-[a-z0-9-]+$') {
                throw "Lane '$id' has an unsafe Compose project identity '$project'."
            }
        } elseif ($null -ne $project) {
            throw "Offline lane '$id' unexpectedly has a Compose project."
        }

        if (-not $ActionMap.Contains($id)) { throw "Formal profile omitted action mapping for '$id'." }
        $actions = @($ActionMap[$id])
        if ($actions.Count -eq 0) { throw "Formal lane '$id' has no assertion actions." }
        $localActions = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $environmentProviders = 0
        $restorationProviders = 0
        foreach ($action in $actions) {
            if (-not $localActions.Add([string]$action.id)) {
                throw "Formal lane '$id' contains duplicate action '$($action.id)'."
            }
            $null = $seenActions.Add("$id/$([string]$action.id)")
            if ([string]$action.kind -ceq "manual" -and $Phase -cne "uat") {
                throw "Only formal UAT may consume manual scenario evidence."
            }
            if ([bool]$action.produces_evidence -and @($action.arguments) -cnotcontains "{evidence}") {
                throw "Evidence-producing action '$id/$($action.id)' omits the attempt-scoped evidence path."
            }
            if ([bool]$action.provides_environment) {
                $environmentProviders++
                if (@($action.arguments) -cnotcontains "{project}") {
                    throw "Environment action '$id/$($action.id)' omits the exact Compose project binding."
                }
            }
            if ([bool]$action.provides_restoration -or [string]$action.kind -ceq "teardown") {
                $restorationProviders++
            }
            if (@($action.arguments) -ccontains "-UseExistingTopology" -and
                @($action.arguments) -cnotcontains "{fixture}") {
                throw "Existing-topology action '$id/$($action.id)' omits its authenticated fixture receipt."
            }
            $actionArguments = @($action.arguments)
            $isRawPlaywrightAcceptance = [string]$action.kind -ceq "program" -and
                [string]$action.command -ceq "npm" -and
                $actionArguments.Count -ge 3 -and
                [string]$actionArguments[0] -ceq "--prefix" -and
                [string]$actionArguments[1] -ceq ".\end2end" -and
                [string]$actionArguments[2] -ceq "test"
            $isBatchedPlaywrightAcceptance = [string]$action.kind -ceq "pwsh" -and
                [string]$action.command -ceq "scripts/tessara-validation-evidence-browser-batches.ps1"
            $isPlaywrightAcceptance = $isRawPlaywrightAcceptance -or $isBatchedPlaywrightAcceptance
            if ($isPlaywrightAcceptance -and
                [string]$action.playwright_data_state -notin @("fresh", "upgraded")) {
                throw "Playwright action '$id/$($action.id)' omits its exact data-state identity."
            }
            if (-not $isPlaywrightAcceptance -and
                -not [string]::IsNullOrWhiteSpace([string]$action.playwright_data_state)) {
                throw "Non-Playwright action '$id/$($action.id)' declares a Playwright data state."
            }
            if ($isPlaywrightAcceptance -and $id -in @("rehearsal-browser", "sit-browser") -and
                [string]$action.playwright_data_state -cne "fresh") {
                throw "Reference browser lane '$id' must bind Playwright to the fresh data state."
            }
        }
        if ([bool]$lane.touches_live_state -and ($environmentProviders -eq 0 -or $restorationProviders -eq 0)) {
            throw "Live lane '$id' lacks an environment or restoration evidence provider."
        }
        if (-not [bool]$lane.touches_live_state -and $environmentProviders -ne 0) {
            throw "Offline lane '$id' maps a live environment provider."
        }

        $paths = Get-Sprint8CLaneEvidencePaths -Contract $Contract -Phase $Phase -Lane $id
        $expectedSuffix = (Get-Sprint8CRepositoryRelativePath -Path (
            Get-Sprint8CEvidenceRoot -Contract $Contract
        )).TrimEnd("/", "\") + "/$Phase/lanes/$id"
        $actualSuffix = (Get-Sprint8CRepositoryRelativePath -Path $paths.lane_root)
        if ($actualSuffix -cne $expectedSuffix.Replace("\", "/")) {
            throw "Lane '$id' evidence mapping is not canonical."
        }
        if ((Split-Path -Leaf $paths.result) -cne "result.json" -or
            (Split-Path -Leaf $paths.command_log) -cne "command.log" -or
            (Split-Path -Leaf $paths.evidence_references) -cne "evidence-references.json") {
            throw "Lane '$id' evidence filenames are not canonical."
        }
        $selector = Get-Sprint8CSelector -Phase $Phase -Lane $id
        if (-not $selector.Contains($id) -and $Phase -cne "validation-preflight") {
            throw "Lane '$id' selector does not bind its exact identity."
        }
    }

    if ($Phase -ceq "uat") {
        $scenarioContract = Get-Sprint8CScenarioContract
        $mapped = @($phaseLanes | ForEach-Object {
            @($ActionMap[[string]$_.id] | Where-Object { [string]$_.kind -ceq "manual" } | ForEach-Object { @($_.scenarios) })
        } | ForEach-Object { [string]$_ } | Sort-Object -Unique)
        Assert-Sprint8CExactSequence -Expected @($scenarioContract.scenarios.id | Sort-Object) -Actual $mapped -Label "formal UAT manual scenario coverage"
    }

    return $true
}

function Get-Sprint8CLaneContract {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $matches = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase -and [string]$_.id -ceq $Lane })
    if ($matches.Count -ne 1) { throw "Unknown $Phase lane '$Lane'." }
    return $matches[0]
}

function Get-Sprint8CLaneResultPath {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$LaneId
    )
    $matches = @($Contract.lanes | Where-Object { [string]$_.id -ceq $LaneId })
    if ($matches.Count -ne 1) { throw "Prerequisite lane '$LaneId' is not unique in the contract." }
    return (Get-Sprint8CLaneEvidencePaths -Contract $Contract -Phase ([string]$matches[0].phase) -Lane $LaneId).result
}

function Read-Sprint8CAuthenticatedJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedSha256,
        [switch]$RequireSidecar
    )
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required evidence is missing: $Path"
    }
    if ($RequireSidecar) {
        $null = Assert-Sprint8CSha256Sidecar -Path $Path -ExpectedSha256 $ExpectedSha256
    } elseif (-not [string]::IsNullOrWhiteSpace($ExpectedSha256)) {
        $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -cne $ExpectedSha256) { throw "Evidence authentication failed: $Path" }
    }
    try { return Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json -Depth 100 } catch {
        throw "Evidence is not valid JSON: $Path. $($_.Exception.Message)"
    }
}

function Assert-Sprint8CSourceMatches {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual,
        [Parameter(Mandatory)][string]$Label
    )
    if ([string]$Expected.commit -cne [string]$Actual.commit -or
        [string]$Expected.tree -cne [string]$Actual.tree -or
        [bool]$Expected.dirty -or [bool]$Actual.dirty) {
        throw "$Label is not bound to the current clean source."
    }
}

function Get-Sprint8CImplementationResultPath {
    param([Parameter(Mandatory)]$Contract)
    $root = Get-Sprint8CEvidenceRoot -Contract $Contract
    return Join-Path $root "implementation/implementation-readiness-result.json"
}

function Get-Sprint8CImpactPath {
    param([Parameter(Mandatory)]$Contract)
    $root = Get-Sprint8CEvidenceRoot -Contract $Contract
    return Join-Path $root "validation-impact.json"
}

function Assert-Sprint8CImplementationGate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Source
    )
    $path = Get-Sprint8CImplementationResultPath -Contract $Contract
    $result = Read-Sprint8CAuthenticatedJson -Path $path -RequireSidecar
    $null = Assert-TessaraImplementationReadinessResult -Result $result -Contract $Contract -ContractPath $script:Sprint8CContractPath
    Assert-Sprint8CSourceMatches -Expected $Source -Actual $result.source_identity -Label "Implementation readiness"

    $impactPath = Get-Sprint8CImpactPath -Contract $Contract
    $impact = Read-Sprint8CAuthenticatedJson -Path $impactPath -RequireSidecar
    $null = Assert-TessaraCorrectionImpactAssessment -Assessment $impact -Contract $Contract
    Assert-Sprint8CSourceMatches -Expected $Source -Actual $impact.current_source -Label "Validation impact assessment"
    $decision = @($impact.phase_decisions | Where-Object { [string]$_.phase -ceq "validation-readiness" })
    if ($decision.Count -ne 1 -or [string]$decision[0].action -eq "reuse_certificate") {
        throw "Validation impact does not authorize a current Readiness execution plan."
    }
    return @(
        Get-Sprint8CFileReference -Path $path
        Get-Sprint8CFileReference -Path $impactPath
    )
}

function Get-Sprint8CCertificatePath {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )
    return Join-Path (Get-Sprint8CPhaseRoot -Contract $Contract -Phase $Phase) (Get-Sprint8CPhaseResultName -Phase $Phase)
}

function Get-Sprint8CCandidatePath {
    param([Parameter(Mandatory)]$Contract)
    $root = Get-Sprint8CEvidenceRoot -Contract $Contract
    return Join-Path $root "candidate.json"
}

function Read-Sprint8CCandidate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Source
    )
    $path = Get-Sprint8CCandidatePath -Contract $Contract
    $candidate = Read-Sprint8CAuthenticatedJson -Path $path -RequireSidecar
    if ([int]$candidate.schema_version -ne 1 -or
        [string]$candidate.contract -cne "tessara.validation.candidate" -or
        [string]$candidate.sprint -cne "sprint-8c" -or
        [string]$candidate.candidate_fingerprint -cnotmatch '^[0-9a-f]{64}$') {
        throw "Frozen Sprint 8C candidate has the wrong identity."
    }
    Assert-Sprint8CSourceMatches -Expected $Source -Actual $candidate.source_identity -Label "Frozen candidate"
    return $candidate
}

function Assert-Sprint8CPhaseCertificateForContract {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$Source,
        [string]$CandidateFingerprint
    )
    $path = Get-Sprint8CCertificatePath -Contract $Contract -Phase $Phase
    $certificate = Read-Sprint8CAuthenticatedJson -Path $path -RequireSidecar
    $null = Assert-TessaraPhaseCertificate -Certificate $certificate
    if ([string]$certificate.sprint -cne "sprint-8c" -or [string]$certificate.phase -cne $Phase) {
        throw "Certificate '$path' has the wrong Sprint 8C phase identity."
    }
    Assert-Sprint8CSourceMatches -Expected $Source -Actual $certificate.source_identity -Label "$Phase certificate"
    $expectedLanes = @(Get-Sprint8CExpectedLaneIds -Phase $Phase)
    Assert-Sprint8CExactSequence -Expected $expectedLanes -Actual @($certificate.lanes.name) -Label "$Phase certificate lane coverage"
    if (-not [string]::IsNullOrWhiteSpace($CandidateFingerprint) -and
        [string]$certificate.candidate_fingerprint -cne $CandidateFingerprint) {
        throw "$Phase certificate is not bound to the frozen candidate."
    }
    $indexPath = Join-Path $script:Sprint8CRepositoryRoot ([string]$certificate.evidence_index.path)
    $index = Read-Sprint8CAuthenticatedJson -Path $indexPath `
        -ExpectedSha256 ([string]$certificate.evidence_index.sha256) -RequireSidecar
    $null = Assert-TessaraPhaseEvidenceIndex -Index $index -RepositoryRoot $script:Sprint8CRepositoryRoot -AuditFiles
    return [pscustomobject]@{
        certificate = $certificate
        reference = Get-Sprint8CFileReference -Path $path
    }
}

function Assert-Sprint8CLanePrerequisites {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$LaneContract,
        [Parameter(Mandatory)]$Source
    )
    $references = [Collections.Generic.List[object]]::new()
    $phase = [string]$LaneContract.phase
    $candidateFingerprint = $null
    $topologyContext = $null

    if ([string]$LaneContract.id -ceq "readiness-contract") {
        foreach ($reference in @(Assert-Sprint8CImplementationGate -Contract $Contract -Source $Source)) {
            $references.Add($reference)
        }
    }

    foreach ($prerequisite in @($LaneContract.prerequisites)) {
        $path = Get-Sprint8CLaneResultPath -Contract $Contract -LaneId ([string]$prerequisite)
        $result = Read-Sprint8CAuthenticatedJson -Path $path -RequireSidecar
        if ([string]$result.contract -cne "tessara.validation.lane-result" -or
            [string]$result.lane -cne [string]$prerequisite -or
            [string]$result.state -cne "passed") {
            throw "Prerequisite lane '$prerequisite' is not a passing authenticated result."
        }
        Assert-Sprint8CSourceMatches -Expected $Source -Actual $result.source_identity -Label "Prerequisite lane '$prerequisite'"
        $references.Add((Get-Sprint8CFileReference -Path $path))
        if ($phase -ceq "sit" -and $result.PSObject.Properties.Name -contains "topology_context" -and
            $null -ne $result.topology_context) {
            $topologyContext = $result.topology_context
        }
    }

    if ([string]$LaneContract.id -ceq (Get-Sprint8CExpectedLaneIds -Phase $phase)[0]) {
        switch ($phase) {
            "candidate-rehearsal" {
                $prior = Assert-Sprint8CPhaseCertificateForContract -Contract $Contract -Phase "validation-readiness" -Source $Source
                $references.Add($prior.reference)
            }
            "validation-preflight" {
                foreach ($priorPhase in @("validation-readiness", "candidate-rehearsal")) {
                    $prior = Assert-Sprint8CPhaseCertificateForContract -Contract $Contract -Phase $priorPhase -Source $Source
                    $references.Add($prior.reference)
                }
            }
            "sit" {
                $candidate = Read-Sprint8CCandidate -Contract $Contract -Source $Source
                $candidateFingerprint = [string]$candidate.candidate_fingerprint
                $prior = Assert-Sprint8CPhaseCertificateForContract -Contract $Contract -Phase "validation-preflight" -Source $Source -CandidateFingerprint $candidateFingerprint
                $references.Add($prior.reference)
                $references.Add((Get-Sprint8CFileReference -Path (Get-Sprint8CCandidatePath -Contract $Contract)))
            }
            "uat" {
                $candidate = Read-Sprint8CCandidate -Contract $Contract -Source $Source
                $candidateFingerprint = [string]$candidate.candidate_fingerprint
                $prior = Assert-Sprint8CPhaseCertificateForContract -Contract $Contract -Phase "sit" -Source $Source -CandidateFingerprint $candidateFingerprint
                $references.Add($prior.reference)
                $references.Add((Get-Sprint8CFileReference -Path (Get-Sprint8CCandidatePath -Contract $Contract)))
            }
        }
    }

    if ($phase -in @("sit", "uat") -and [string]::IsNullOrWhiteSpace($candidateFingerprint)) {
        $candidate = Read-Sprint8CCandidate -Contract $Contract -Source $Source
        $candidateFingerprint = [string]$candidate.candidate_fingerprint
    }

    if ($phase -ceq "sit" -and [string]$LaneContract.id -in @("sit-browser", "sit-smoke")) {
        if ($null -eq $topologyContext) {
            throw "SIT lane '$($LaneContract.id)' requires the authenticated retained tessara-s8c-sit topology context."
        }
        $topologyContext = Assert-Sprint8CTopologyContext -Context $topologyContext `
            -ExpectedProject "tessara-s8c-sit" -CandidateFingerprint $candidateFingerprint
    }

    return [pscustomobject][ordered]@{
        references = @($references)
        candidate_fingerprint = $candidateFingerprint
        topology_context = $topologyContext
    }
}

function Assert-Sprint8CActionAvailability {
    param([Parameter(Mandatory)][object[]]$Actions)
    foreach ($action in $Actions) {
        switch ([string]$action.kind) {
            "pwsh" {
                $path = Join-Path $script:Sprint8CRepositoryRoot ([string]$action.command)
                if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                    throw "Required Sprint 8C lane harness is missing: $($action.command)"
                }
            }
            "program" {
                if ($null -eq (Get-Command ([string]$action.command) -ErrorAction SilentlyContinue)) {
                    throw "Required Sprint 8C lane program is unavailable: $($action.command)"
                }
            }
            "teardown" {
                if ($null -eq (Get-Command "docker" -ErrorAction SilentlyContinue)) {
                    throw "Docker is required for exact Sprint 8C lane teardown."
                }
            }
            "internal" { }
            "manual" { }
            default { throw "Lane action '$($action.id)' has unknown kind '$($action.kind)'." }
        }
    }
}

function Expand-Sprint8CActionArguments {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Arguments,
        [AllowNull()][string]$Project,
        [Parameter(Mandatory)][string]$EvidencePath,
        [AllowNull()][string]$CandidateFingerprint,
        [AllowNull()]$TopologyContext
    )
    $fixturePath = if ($null -eq $TopologyContext) { "" } else { [string]$TopologyContext.fixture_receipt.path }
    @($Arguments | ForEach-Object {
        ([string]$_).Replace("{project}", [string]$Project).
            Replace("{evidence}", $EvidencePath).
            Replace("{repo}", $script:Sprint8CRepositoryRoot).
            Replace("{candidate}", [string]$CandidateFingerprint).
            Replace("{fixture}", $fixturePath)
    })
}

function Assert-Sprint8CHarnessEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Action,
        [AllowNull()][string]$ExpectedProject,
        [Parameter(Mandatory)]$ExpectedSource
    )
    $document = Read-Sprint8CAuthenticatedJson -Path $Path `
        -RequireSidecar:([bool]$Action.provides_environment -or [bool]$Action.provides_restoration)
    if ([bool]$Action.provides_environment -or [bool]$Action.provides_restoration) {
        if ([string]$document.sprint -cne "sprint-8c" -or [string]$document.state -cne "passed") {
            throw "Harness '$($action.id)' did not publish a passing Sprint 8C result."
        }
    }
    $expectedProof = $null
    $expectedContract = $null
    $expectedStatus = $null
    switch ([IO.Path]::GetFileName([string]$Action.command)) {
        "materialize-sprint-8c.ps1" {
            $targetIndex = [Array]::IndexOf([object[]]@($Action.arguments), "-Target")
            if ($targetIndex -lt 0 -or $targetIndex + 1 -ge @($Action.arguments).Count) {
                throw "Materialization action '$($Action.id)' has no exact target selector."
            }
            $target = [string]$Action.arguments[$targetIndex + 1]
            $expectedProof = if ($target -ceq "All") {
                "focused-owner-materialization"
            } elseif ($target -ceq "ReferenceNoOp") {
                "clean-owner-materialization-and-semantic-noop"
            } else { "clean-owner-materialization" }
            if ([string]$document.target -cne $target) {
                throw "Harness '$($Action.id)' substituted materialization target '$($document.target)'."
            }
        }
        "run-sprint-8c-deployed-smoke.ps1" { $expectedProof = "deployed-acceptance-smoke" }
        "run-sprint-8c-failure-containment.ps1" {
            $expectedProof = "deterministic-failure-containment-retry-and-restoration"
        }
        "run-sprint-8c-response-upgrade.ps1" {
            $expectedProof = "independent-response-upgrade-rollback-restoration"
        }
        "uat-sprint-8c.ps1" { $expectedProof = "uat-automated-predicates" }
        "sprint-8c-acceptance-contract.ps1" {
            $expectedContract = "tessara.sprint-8c.acceptance-contract-result"
            $expectedStatus = "passed"
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($expectedProof) -and (
            [string]$document.proof -cne $expectedProof -or
            $null -eq $document.PSObject.Properties['source'] -or
            [string]$document.source.commit -cne [string]$ExpectedSource.commit -or
            [string]$document.source.tree -cne [string]$ExpectedSource.tree -or
            [bool]$document.source.dirty -ne [bool]$ExpectedSource.dirty
        )) {
        throw "Harness '$($Action.id)' did not bind its action-specific proof to the exact source."
    }
    if (-not [string]::IsNullOrWhiteSpace($expectedContract) -and (
            [string]$document.contract -cne $expectedContract -or
            [string]$document.status -cne $expectedStatus
        )) {
        throw "Harness '$($Action.id)' published a substituted action-specific contract."
    }
    if ([bool]$Action.provides_environment) {
        if ([string]$document.compose_project -cne $ExpectedProject) {
            throw "Harness '$($action.id)' used Compose project '$($document.compose_project)' instead of '$ExpectedProject'."
        }
        $fingerprint = Get-Sprint8CEnvironmentFingerprintFromDocument -Document $document
        if ($fingerprint -cnotmatch '^[0-9a-f]{64}$') {
            throw "Harness '$($action.id)' omitted its secret-free environment fingerprint."
        }
    }
    if ([bool]$Action.provides_restoration) {
        if (-not ($document.PSObject.Properties.Name -contains "cleanup_restoration") -or
            [string]$document.cleanup_restoration.state -cne "passed") {
            throw "Harness '$($action.id)' did not prove cleanup/canonical restoration."
        }
    }
    return $document
}

function Invoke-Sprint8CComposeTeardown {
    param(
        [Parameter(Mandatory)][string]$Project,
        [Parameter(Mandatory)][string]$EvidencePath
    )
    if ($Project -cnotmatch '^tessara-s8c-[a-z0-9-]+$') {
        throw "Refusing to tear down unsafe Compose project '$Project'."
    }
    $composePath = Join-Path $script:Sprint8CRepositoryRoot "deploy/sprint-8c/compose.yaml"
    if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) {
        throw "Sprint 8C Compose profile is missing."
    }
    & docker compose -f $composePath -p $Project --profile reference down --volumes --remove-orphans
    if ($LASTEXITCODE -ne 0) { throw "Exact Compose teardown failed for '$Project'." }
    $remainingContainers = @(& docker ps -a --filter "label=com.docker.compose.project=$Project" --format "{{.ID}}")
    if ($LASTEXITCODE -ne 0 -or @($remainingContainers | Where-Object { $_ }).Count -ne 0) {
        throw "Compose teardown left containers for '$Project'."
    }
    $environmentFingerprint = Get-Sprint8CSha256Text -Text "sprint-8c`n$Project`nreference`n"
    $receipt = [ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8c.compose-teardown"
        sprint = "sprint-8c"
        state = "passed"
        compose_project = $Project
        environment_fingerprint = $environmentFingerprint
        cleanup_restoration = [ordered]@{ state = "passed"; mode = "exact_project_removed" }
    }
    Write-Sprint8CNewJsonFile -Path $EvidencePath -Document $receipt
    return $receipt
}

function Get-Sprint8CManualScenarioPath {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Scenario
    )
    $root = Get-Sprint8CEvidenceRoot -Contract $Contract
    return Join-Path $root "uat/scenarios/$Scenario/result.json"
}

function Assert-Sprint8CManualScenarioEvidence {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string[]]$Scenarios,
        [Parameter(Mandatory)][string]$CandidateFingerprint
    )
    $scenarioContract = Get-Sprint8CScenarioContract
    $references = [Collections.Generic.List[object]]::new()
    foreach ($scenario in $Scenarios) {
        $expected = @($scenarioContract.scenarios | Where-Object { [string]$_.id -ceq $scenario })
        if ($expected.Count -ne 1) { throw "Unknown manual UAT scenario '$scenario'." }
        $path = Get-Sprint8CManualScenarioPath -Contract $Contract -Scenario $scenario
        $result = Read-Sprint8CAuthenticatedJson -Path $path -RequireSidecar
        if ([string]$result.scenario_id -cne $scenario -or
            [string]$result.state -cne "passed" -or
            -not [bool]$result.manual_acceptance_claimed -or
            [string]$result.candidate_fingerprint -cne $CandidateFingerprint) {
            throw "Manual evidence for '$scenario' is missing a passing, candidate-bound human acceptance claim."
        }
        Assert-Sprint8CExactSequence -Expected @($expected[0].assertions) -Actual @($result.assertions) -Label "$scenario manual assertions"
        if (-not ($result.PSObject.Properties.Name -contains "evidence_references") -or
            @($result.evidence_references).Count -eq 0) {
            throw "Manual evidence for '$scenario' has no retained evidence references."
        }
        $references.Add((Get-Sprint8CFileReference -Path $path))
    }
    return @($references)
}

function Get-Sprint8CTrackedInventory {
    param([Parameter(Mandatory)][string[]]$Paths)
    $items = [Collections.Generic.List[object]]::new()
    foreach ($relative in @($Paths | Sort-Object -Unique)) {
        $full = Join-Path $script:Sprint8CRepositoryRoot $relative
        if (-not (Test-Path -LiteralPath $full -PathType Leaf)) {
            throw "Candidate input is missing: $relative"
        }
        $items.Add([ordered]@{
            path = $relative.Replace("\", "/")
            sha256 = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLowerInvariant()
        })
    }
    return @($items)
}

function New-Sprint8CCandidate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Source
    )
    $readiness = Assert-Sprint8CPhaseCertificateForContract -Contract $Contract -Phase "validation-readiness" -Source $Source
    $rehearsal = Assert-Sprint8CPhaseCertificateForContract -Contract $Contract -Phase "candidate-rehearsal" -Source $Source
    $implementationPath = Get-Sprint8CImplementationResultPath -Contract $Contract
    $implementation = Read-Sprint8CAuthenticatedJson -Path $implementationPath -RequireSidecar
    $null = Assert-TessaraImplementationReadinessResult -Result $implementation -Contract $Contract -ContractPath $script:Sprint8CContractPath
    Assert-Sprint8CSourceMatches -Expected $Source -Actual $implementation.source_identity -Label "Frozen implementation prerequisite"

    $acceptanceInputs = Get-Sprint8CTrackedInventory -Paths @(
        "end2end/acceptance-manifest.json",
        "docs/sprints/sprint-8c-test-change-log.md",
        "docs/sprints/sprint-8c-uat/scenario-contract.json",
        "docs/audits/sprint-8c-response-ui-baseline/baseline-index.json"
    )
    $deploymentPaths = @(
        "deploy/sprint-8c/compose.yaml",
        "deploy/sprint-8c/compose.override.yaml",
        "deploy/sprint-8c/catalogs/local-release-catalog.json",
        "deploy/sprint-8c/fixtures/reference-fixture-contract.json",
        "deploy/sprint-8c/fixtures/provider-fault-contract.json",
        "deploy/sprint-8c/fixtures/upgrade-fixture-contract.json",
        "crates/tessara-response-module/manifest.json"
    )
    $deploymentInputs = Get-Sprint8CTrackedInventory -Paths $deploymentPaths
    $payload = [ordered]@{
        source_identity = $Source
        validation_contract = Get-Sprint8CFileReference -Path $script:Sprint8CContractPath
        implementation_readiness = Get-Sprint8CFileReference -Path $implementationPath
        readiness_certificate = $readiness.reference
        rehearsal_certificate = $rehearsal.reference
        acceptance_inputs = $acceptanceInputs
        deployment_inputs = $deploymentInputs
    }
    $fingerprint = Get-Sprint8CSha256Text -Text (($payload | ConvertTo-Json -Depth 100 -Compress) + "`n")
    $candidate = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.candidate"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8c"
        frozen_at = [DateTimeOffset]::UtcNow.ToString("O")
        source_identity = $Source
        validation_contract = $payload.validation_contract
        implementation_readiness = $payload.implementation_readiness
        readiness_certificate = $payload.readiness_certificate
        rehearsal_certificate = $payload.rehearsal_certificate
        acceptance_inputs = $acceptanceInputs
        deployment_inputs = $deploymentInputs
        candidate_fingerprint = $fingerprint
    }
    $path = Get-Sprint8CCandidatePath -Contract $Contract
    $null = Publish-Sprint8CJsonAndSidecar -Path $path -Document $candidate
    return [pscustomobject]@{ document = $candidate; reference = Get-Sprint8CFileReference -Path $path }
}

function Invoke-Sprint8CAction {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Action,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [AllowNull()][string]$Project,
        [AllowNull()][string]$CandidateFingerprint,
        [AllowNull()]$TopologyContext,
        [Parameter(Mandatory)]$Source,
        [switch]$SuppressChildOutput
    )
    $actionsRoot = Join-Path $AttemptRoot "actions"
    [IO.Directory]::CreateDirectory($actionsRoot) | Out-Null
    $actionResultPath = Join-Path $actionsRoot "$($Action.id).json"
    $harnessEvidencePath = Join-Path $actionsRoot "$($Action.id)-evidence.json"
    $started = [DateTimeOffset]::UtcNow
    $references = [Collections.Generic.List[object]]::new()
    $harnessDocument = $null
    $harnessEvidenceReference = $null
    $commandText = $null
    $resultTopologyContext = $TopologyContext
    $topologyRemoved = $false

    switch ([string]$Action.kind) {
        "pwsh" {
            $scriptPath = Join-Path $script:Sprint8CRepositoryRoot ([string]$Action.command)
            $arguments = @(Expand-Sprint8CActionArguments -Arguments @($Action.arguments) -Project $Project `
                -EvidencePath $harnessEvidencePath -CandidateFingerprint $CandidateFingerprint `
                -TopologyContext $TopologyContext)
            $commandText = "pwsh -NoProfile -File $($Action.command) $($arguments -join ' ')"
            $expectedDataState = [string]$Action.playwright_data_state
            $dataStateBefore = [Environment]::GetEnvironmentVariable(
                "TESSARA_PLAYWRIGHT_DATA_STATE", "Process"
            )
            try {
                if (-not [string]::IsNullOrWhiteSpace($expectedDataState)) {
                    if ($null -eq $TopologyContext -or
                        [Environment]::GetEnvironmentVariable(
                            "TESSARA_PLAYWRIGHT_ACCEPTANCE", "Process"
                        ) -cne "1") {
                        throw "Playwright action '$($Action.id)' lacks an authenticated retained topology/acceptance binding."
                    }
                    [Environment]::SetEnvironmentVariable(
                        "TESSARA_PLAYWRIGHT_DATA_STATE", $expectedDataState, "Process"
                    )
                }
                if ($SuppressChildOutput) {
                    & pwsh -NoProfile -File $scriptPath @arguments | Out-Null
                } else {
                    & pwsh -NoProfile -File $scriptPath @arguments | Out-Host
                }
                $exitCode = $LASTEXITCODE
            } finally {
                [Environment]::SetEnvironmentVariable(
                    "TESSARA_PLAYWRIGHT_DATA_STATE", $dataStateBefore, "Process"
                )
            }
            if ($exitCode -ne 0) { throw "Action '$($Action.id)' exited $exitCode." }
            if ([bool]$Action.produces_evidence) {
                $harnessDocument = Assert-Sprint8CHarnessEvidence -Path $harnessEvidencePath `
                    -Action $Action -ExpectedProject $Project -ExpectedSource $Source
                $harnessEvidenceReference = Get-Sprint8CFileReference -Path $harnessEvidencePath
                $references.Add($harnessEvidenceReference)
                if ($harnessDocument.PSObject.Properties.Name -contains "cleanup_restoration" -and
                    $harnessDocument.cleanup_restoration.PSObject.Properties.Name -contains "mode" -and
                    [string]$harnessDocument.cleanup_restoration.mode -ceq "retained-for-caller") {
                    $resultTopologyContext = Get-Sprint8CTopologyContextFromHarness -Document $harnessDocument `
                        -ExpectedProject $Project -CandidateFingerprint $CandidateFingerprint
                }
            }
        }
        "program" {
            $arguments = @(Expand-Sprint8CActionArguments -Arguments @($Action.arguments) -Project $Project `
                -EvidencePath $harnessEvidencePath -CandidateFingerprint $CandidateFingerprint `
                -TopologyContext $TopologyContext)
            $commandText = "$($Action.command) $($arguments -join ' ')"
            $expectedDataState = [string]$Action.playwright_data_state
            $dataStateBefore = [Environment]::GetEnvironmentVariable(
                "TESSARA_PLAYWRIGHT_DATA_STATE", "Process"
            )
            try {
                if (-not [string]::IsNullOrWhiteSpace($expectedDataState)) {
                    if ($null -eq $TopologyContext -or
                        [Environment]::GetEnvironmentVariable(
                            "TESSARA_PLAYWRIGHT_ACCEPTANCE", "Process"
                        ) -cne "1") {
                        throw "Playwright action '$($Action.id)' lacks an authenticated retained topology/acceptance binding."
                    }
                    [Environment]::SetEnvironmentVariable(
                        "TESSARA_PLAYWRIGHT_DATA_STATE", $expectedDataState, "Process"
                    )
                }
                if ($SuppressChildOutput) {
                    & ([string]$Action.command) @arguments | Out-Null
                } else {
                    & ([string]$Action.command) @arguments | Out-Host
                }
                $exitCode = $LASTEXITCODE
            } finally {
                [Environment]::SetEnvironmentVariable(
                    "TESSARA_PLAYWRIGHT_DATA_STATE", $dataStateBefore, "Process"
                )
            }
            if ($exitCode -ne 0) { throw "Action '$($Action.id)' exited $exitCode." }
        }
        "teardown" {
            $commandText = "docker compose -f deploy/sprint-8c/compose.yaml -p $Project --profile reference down --volumes --remove-orphans"
            $harnessDocument = Invoke-Sprint8CComposeTeardown -Project $Project -EvidencePath $harnessEvidencePath
            $harnessEvidenceReference = Get-Sprint8CFileReference -Path $harnessEvidencePath
            $references.Add($harnessEvidenceReference)
            $resultTopologyContext = $null
            $topologyRemoved = $true
        }
        "manual" {
            $commandText = "authenticate manual UAT evidence: $(@($Action.scenarios) -join ', ')"
            foreach ($reference in @(Assert-Sprint8CManualScenarioEvidence -Contract $Contract -Scenarios @($Action.scenarios) -CandidateFingerprint $CandidateFingerprint)) {
                $references.Add($reference)
            }
        }
        "internal" {
            $commandText = [string]$Action.command
            switch ([string]$Action.command) {
                "implementation-gate" {
                    foreach ($reference in @(Assert-Sprint8CImplementationGate -Contract $Contract -Source $Source)) {
                        $references.Add($reference)
                    }
                }
                "freeze-candidate" {
                    $candidate = New-Sprint8CCandidate -Contract $Contract -Source $Source
                    $references.Add($candidate.reference)
                    $harnessDocument = $candidate.document
                }
                default { throw "Unknown internal action '$($Action.command)'." }
            }
        }
        default { throw "Unknown action kind '$($Action.kind)'." }
    }

    $completed = [DateTimeOffset]::UtcNow
    $result = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-action-result"
        sprint = "sprint-8c"
        action = [string]$Action.id
        state = "passed"
        command = $commandText
        started_at = $started.ToString("O")
        completed_at = $completed.ToString("O")
        duration_ms = [long]($completed - $started).TotalMilliseconds
        playwright_data_state = if ([string]::IsNullOrWhiteSpace(
            [string]$Action.playwright_data_state
        )) { $null } else { [string]$Action.playwright_data_state }
        evidence_references = @($references)
    }
    Write-Sprint8CNewJsonFile -Path $actionResultPath -Document $result
    $references.Add((Get-Sprint8CFileReference -Path $actionResultPath))

    $environmentFingerprint = $null
    $restorationPassed = $false
    $restorationReference = $null
    if ($null -ne $harnessDocument) {
        $environmentFingerprint = Get-Sprint8CEnvironmentFingerprintFromDocument -Document $harnessDocument
        if (($Action.kind -ceq "teardown" -or [bool]$Action.provides_restoration) -and
            $harnessDocument.PSObject.Properties.Name -contains "cleanup_restoration") {
            $restorationPassed = [string]$harnessDocument.cleanup_restoration.state -ceq "passed"
            if ($restorationPassed) { $restorationReference = $harnessEvidenceReference }
        }
    }

    return [pscustomobject][ordered]@{
        result_reference = Get-Sprint8CFileReference -Path $actionResultPath
        evidence_references = @($references)
        environment_fingerprint = $environmentFingerprint
        restoration_passed = $restorationPassed
        restoration_reference = $restorationReference
        topology_context = $resultTopologyContext
        topology_removed = $topologyRemoved
    }
}

function Get-Sprint8CPhasePrerequisiteReferences {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )
    switch ($Phase) {
        "validation-readiness" {
            @(
                Get-Sprint8CFileReference -Path (Get-Sprint8CImplementationResultPath -Contract $Contract)
                Get-Sprint8CFileReference -Path (Get-Sprint8CImpactPath -Contract $Contract)
            )
        }
        "candidate-rehearsal" { @((Get-Sprint8CFileReference -Path (Get-Sprint8CCertificatePath -Contract $Contract -Phase "validation-readiness"))) }
        "validation-preflight" {
            @(
                Get-Sprint8CFileReference -Path (Get-Sprint8CCertificatePath -Contract $Contract -Phase "validation-readiness")
                Get-Sprint8CFileReference -Path (Get-Sprint8CCertificatePath -Contract $Contract -Phase "candidate-rehearsal")
            )
        }
        "sit" { @((Get-Sprint8CFileReference -Path (Get-Sprint8CCertificatePath -Contract $Contract -Phase "validation-preflight"))) }
        "uat" { @((Get-Sprint8CFileReference -Path (Get-Sprint8CCertificatePath -Contract $Contract -Phase "sit"))) }
    }
}

function Complete-Sprint8CPhase {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$Source,
        [AllowNull()][string]$CandidateFingerprint
    )
    $phaseRoot = Get-Sprint8CPhaseRoot -Contract $Contract -Phase $Phase
    $null = Assert-Sprint8CDefectProvenanceChronology -Contract $Contract
    $certificatePath = Get-Sprint8CCertificatePath -Contract $Contract -Phase $Phase
    if (Test-Path -LiteralPath $certificatePath) {
        throw "$Phase certificate already exists and will not be overwritten."
    }
    $laneIds = @(Get-Sprint8CExpectedLaneIds -Phase $Phase)
    $laneDocuments = [Collections.Generic.List[object]]::new()
    $environmentFingerprints = [Collections.Generic.List[string]]::new()
    foreach ($laneId in $laneIds) {
        $path = (Get-Sprint8CLaneEvidencePaths -Contract $Contract -Phase $Phase -Lane $laneId).result
        $document = Read-Sprint8CAuthenticatedJson -Path $path -RequireSidecar
        if ([string]$document.contract -cne "tessara.validation.lane-result" -or
            [string]$document.lane -cne $laneId -or [string]$document.state -cne "passed") {
            throw "$Phase cannot finalize because lane '$laneId' is not passed."
        }
        Assert-Sprint8CSourceMatches -Expected $Source -Actual $document.source_identity -Label "$laneId result"
        if (-not [string]::IsNullOrWhiteSpace($CandidateFingerprint) -and
            [string]$document.candidate_fingerprint -cne $CandidateFingerprint) {
            throw "$laneId is not bound to the frozen candidate."
        }
        if ([string]$document.environment.fingerprint -cnotmatch '^[0-9a-f]{64}$') {
            throw "$laneId has no authenticated environment fingerprint."
        }
        $environmentFingerprints.Add([string]$document.environment.fingerprint)
        $laneDocuments.Add($document)
    }

    $indexPath = Join-Path $phaseRoot "evidence-index.json"
    $excluded = @(
        [IO.Path]::GetFullPath($indexPath),
        [IO.Path]::GetFullPath("$indexPath.sha256"),
        [IO.Path]::GetFullPath($certificatePath),
        [IO.Path]::GetFullPath("$certificatePath.sha256")
    )
    $entries = @(
        Get-ChildItem -LiteralPath $phaseRoot -File -Recurse | Where-Object {
            [IO.Path]::GetFullPath($_.FullName) -notin $excluded
        } | Sort-Object FullName | ForEach-Object {
            [ordered]@{
                path = Get-Sprint8CRepositoryRelativePath -Path $_.FullName
                sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                size = [long]$_.Length
                kind = if ($_.Name -eq "command.log") { "log" } elseif ($_.Name -match 'result|receipt') { "receipt" } else { "structured" }
            }
        }
    )
    $index = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.phase-evidence-index"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8c"
        phase = $Phase
        attempt = 1
        evidence_root = Get-Sprint8CRepositoryRelativePath -Path $phaseRoot
        sealed_at = [DateTimeOffset]::UtcNow.ToString("O")
        entry_count = $entries.Count
        entries = $entries
    }
    $null = Assert-TessaraPhaseEvidenceIndex -Index $index
    $indexSha = Get-Sprint8CJsonDocumentSha256 -Document $index

    $phaseDomains = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase } | ForEach-Object { @($_.dependency_domains) } | ForEach-Object { [string]$_ } | Sort-Object -Unique)
    $allFingerprints = @(Get-TessaraDependencyFingerprints -Contract $Contract -RepositoryRoot $script:Sprint8CRepositoryRoot)
    $fingerprints = @($allFingerprints | Where-Object { [string]$_.domain -in $phaseDomains } | Sort-Object domain)
    $declaredDigest = Get-Sprint8CSha256Text -Text (($laneIds -join "`n") + "`n")
    $environmentDigest = Get-Sprint8CSha256Text -Text ((@($environmentFingerprints) -join "`n") + "`n")
    $touchesLive = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase -and [bool]$_.touches_live_state }).Count -gt 0
    $lastLive = @($laneDocuments | Where-Object { [bool]$_.environment.touches_live_state }) | Select-Object -Last 1
    $cleanupReference = if ($touchesLive) {
        if ($null -eq $lastLive -or [string]$lastLive.cleanup_restoration.state -cne "passed") {
            throw "$Phase cannot finalize without passing canonical restoration."
        }
        Get-Sprint8CFileReference -Path (Get-Sprint8CLaneResultPath -Contract $Contract -LaneId ([string]$lastLive.lane))
    } else { $null }
    $authoritative = $Phase -in @("validation-preflight", "sit", "uat")
    $laneSummaries = @($laneDocuments | ForEach-Object {
        [ordered]@{
            name = [string]$_.lane
            state = "passed"
            certification_basis = "executed"
            dependency_domains = @($_.dependency_domains)
            receipt = Get-Sprint8CFileReference -Path (Get-Sprint8CLaneResultPath -Contract $Contract -LaneId ([string]$_.lane))
            started_at = [string]$_.started_at
            ended_at = [string]$_.completed_at
            duration_ms = [long]$_.duration_ms
            inheritance = $null
        }
    })
    $certificate = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.phase-certificate"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8c"
        phase = $Phase
        attempt = 1
        state = "passed"
        authoritative = $authoritative
        certified_at = [DateTimeOffset]::UtcNow.ToString("O")
        source_identity = $Source
        environment_fingerprint = $environmentDigest
        candidate_fingerprint = if ($authoritative) { $CandidateFingerprint } else { $null }
        prerequisite_certificates = @(Get-Sprint8CPhasePrerequisiteReferences -Contract $Contract -Phase $Phase)
        dependency_fingerprints = $fingerprints
        coverage = [ordered]@{
            declared_lanes_sha256 = $declaredDigest
            lane_count = $laneIds.Count
            executed_count = $laneIds.Count
            inherited_count = 0
        }
        lanes = $laneSummaries
        open_defect_count = 0
        cleanup_restoration = [ordered]@{
            required = $touchesLive
            state = if ($touchesLive) { "passed" } else { "not_applicable" }
            evidence = $cleanupReference
        }
        evidence_index = [ordered]@{
            path = Get-Sprint8CRepositoryRelativePath -Path $indexPath
            sha256 = $indexSha
        }
    }
    $null = Assert-TessaraPhaseCertificate -Certificate $certificate
    return Publish-Sprint8CPhasePassCertificate -Contract $Contract `
        -IndexPath $indexPath -IndexDocument $index -ExpectedIndexSha256 $indexSha `
        -CertificatePath $certificatePath -CertificateDocument $certificate
}

function Invoke-Sprint8CFormalLane {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $contract = Get-Sprint8CContract
    $actionMap = Get-Sprint8CFormalActionMap
    $null = Assert-Sprint8CFormalProfile -Contract $contract -Phase $Phase -ActionMap $actionMap
    $laneContract = Get-Sprint8CLaneContract -Contract $contract -Phase $Phase -Lane $Lane
    $actions = @($actionMap[$Lane])
    Assert-Sprint8CActionAvailability -Actions $actions

    $sourceBefore = Get-Sprint8CSourceIdentity
    Assert-Sprint8CCleanSource -Source $sourceBefore
    $null = Assert-Sprint8CDefectProvenanceChronology -Contract $contract
    $prerequisites = Assert-Sprint8CLanePrerequisites -Contract $contract -LaneContract $laneContract -Source $sourceBefore
    $project = Get-Sprint8CExpectedProject -Phase $Phase -LaneContract $laneContract
    $topologyContext = $prerequisites.topology_context
    if ([bool]$laneContract.touches_live_state -and
        [Environment]::GetEnvironmentVariable("TESSARA_SPRINT_8C_DISPOSABLE_RESET_AUTHORIZATION", "Process") -cne $script:Sprint8CResetAuthorization) {
        throw "Live lane '$Lane' requires the exact process-scoped TESSARA_SPRINT_8C_DISPOSABLE_RESET_AUTHORIZATION acknowledgement."
    }

    $paths = Get-Sprint8CLaneEvidencePaths -Contract $contract -Phase $Phase -Lane $Lane
    if (Test-Path -LiteralPath $paths.result -PathType Leaf) {
        throw "Lane '$Lane' already has a canonical result and will not be overwritten."
    }
    [IO.Directory]::CreateDirectory($paths.lane_root) | Out-Null
    $attemptId = "{0}-{1}" -f [DateTimeOffset]::UtcNow.ToString("yyyyMMddTHHmmssfffZ"), ([guid]::NewGuid().ToString("N").Substring(0, 8))
    $attemptRoot = Join-Path $paths.lane_root "attempts/$attemptId"
    [IO.Directory]::CreateDirectory($attemptRoot) | Out-Null
    $attemptLog = Join-Path $attemptRoot "command.log"
    $started = [DateTimeOffset]::UtcNow
    $actionResults = [Collections.Generic.List[object]]::new()
    $references = [Collections.Generic.List[object]]::new()
    foreach ($reference in @($prerequisites.references)) { $references.Add($reference) }
    $environmentFingerprints = [Collections.Generic.List[string]]::new()
    $restorationPassed = -not [bool]$laneContract.touches_live_state
    $restorationReference = $null
    $environmentNames = @(
        "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
        "TESSARA_SUPERVISOR_PORT", "PLAYWRIGHT_BASE_URL", "TESSARA_PLAYWRIGHT_ACCEPTANCE",
        "TESSARA_PLAYWRIGHT_DATA_STATE", "PLAYWRIGHT_POSTGRES_CONTAINER",
        "PLAYWRIGHT_POSTGRES_DATABASE", "PLAYWRIGHT_POSTGRES_USER",
        "TESSARA_SPRINT_8C_RUN_EVIDENCE_ROOT"
    )
    $environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $environmentNames
    $transcribing = $false
    $pushed = $false
    try {
        if ($null -ne $topologyContext) {
            Set-Sprint8CTopologyEnvironment -Context $topologyContext -ExpectedProject $project `
                -CandidateFingerprint ([string]$prerequisites.candidate_fingerprint)
        }
        Start-Transcript -LiteralPath $attemptLog -Force | Out-Null
        $transcribing = $true
        Push-Location $script:Sprint8CRepositoryRoot
        $pushed = $true
        foreach ($action in $actions) {
            $actionResult = Invoke-Sprint8CAction -Contract $contract -Action $action -AttemptRoot $attemptRoot `
                -Project $project -CandidateFingerprint ([string]$prerequisites.candidate_fingerprint) `
                -TopologyContext $topologyContext -Source $sourceBefore
            $actionResults.Add($actionResult.result_reference)
            foreach ($reference in @($actionResult.evidence_references)) { $references.Add($reference) }
            if ([string]$actionResult.environment_fingerprint -match '^[0-9a-f]{64}$') {
                $environmentFingerprints.Add([string]$actionResult.environment_fingerprint)
            }
            if ([bool]$actionResult.restoration_passed) { $restorationPassed = $true }
            if ($null -ne $actionResult.restoration_reference) {
                $restorationReference = $actionResult.restoration_reference
            }
            $topologyContext = $actionResult.topology_context
            if ($null -ne $topologyContext) {
                Set-Sprint8CTopologyEnvironment -Context $topologyContext -ExpectedProject $project `
                    -CandidateFingerprint ([string]$prerequisites.candidate_fingerprint)
            }
        }
    } catch {
        $laneError = $_
        $failureOutcome = Complete-Sprint8CFailedLaneAttempt -Phase $Phase -Lane $Lane `
            -AttemptRoot $attemptRoot -LaneError $laneError -LaneContract $laneContract `
            -TopologyContext $topologyContext -Project $project `
            -Prerequisites $prerequisites -EnvironmentBefore $environmentBefore
        $emergencyCleanup = $failureOutcome.emergency_cleanup
        if ($null -ne $emergencyCleanup -and [string]$emergencyCleanup.state -ceq "failed") {
            throw "Formal lane '$Lane' failed: $($laneError.Exception.Message) Emergency cleanup also failed: $([string]$emergencyCleanup.message)"
        }
        throw $laneError
    } finally {
        if ($pushed) { Pop-Location }
        if ($transcribing) { Stop-Transcript | Out-Null }
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }

    try {
    $sourceAfter = Get-Sprint8CSourceIdentity
    Assert-Sprint8CCleanSource -Source $sourceAfter
    Assert-Sprint8CSourceMatches -Expected $sourceBefore -Actual $sourceAfter -Label "Formal lane '$Lane'"
    if ([bool]$laneContract.touches_live_state -and $environmentFingerprints.Count -eq 0) {
        throw "Live lane '$Lane' did not retain an authenticated environment fingerprint."
    }
    if ([bool]$laneContract.touches_live_state -and -not $restorationPassed) {
        throw "Live lane '$Lane' did not prove cleanup/canonical restoration."
    }
    if ($Phase -ceq "sit" -and $Lane -in @("sit-rust", "sit-browser") -and $null -eq $topologyContext) {
        throw "SIT lane '$Lane' did not retain its authenticated frozen topology context."
    }
    if ($Phase -ceq "sit" -and $Lane -ceq "sit-smoke" -and $null -ne $topologyContext) {
        throw "Final SIT smoke did not remove its retained frozen topology."
    }
    $completed = [DateTimeOffset]::UtcNow
    $environmentFingerprint = if ([bool]$laneContract.touches_live_state) {
        Get-Sprint8CSha256Text -Text ((@($environmentFingerprints) -join "`n") + "`n")
    } else {
        Get-Sprint8CSha256Text -Text "offline`n$Phase`n$Lane`n$($sourceAfter.tree)`n"
    }
    $referenceDocument = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-evidence-references"
        sprint = "sprint-8c"
        phase = $Phase
        lane = $Lane
        references = @($references | Sort-Object path, sha256 -Unique)
    }
    $attemptReferences = Join-Path $attemptRoot "evidence-references.json"
    Write-Sprint8CNewJsonFile -Path $attemptReferences -Document $referenceDocument
    $result = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-result"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8c"
        phase = $Phase
        lane = $Lane
        state = "passed"
        selector = Get-Sprint8CSelector -Phase $Phase -Lane $Lane
        source_identity = $sourceAfter
        candidate_fingerprint = if ($Phase -in @("sit", "uat")) { [string]$prerequisites.candidate_fingerprint } elseif ($Phase -ceq "validation-preflight") {
            $candidate = Read-Sprint8CCandidate -Contract $contract -Source $sourceAfter
            [string]$candidate.candidate_fingerprint
        } else { $null }
        dependency_domains = @($laneContract.dependency_domains)
        prerequisites = @($prerequisites.references)
        environment = [ordered]@{
            kind = if ([bool]$laneContract.touches_live_state) { if ($Phase -ceq "sit") { "frozen-sit" } else { "isolated-live" } } else { "offline" }
            touches_live_state = [bool]$laneContract.touches_live_state
            compose_project = $project
            fingerprint = $environmentFingerprint
        }
        assertions = @($actions.id)
        action_results = @($actionResults)
        started_at = $started.ToString("O")
        completed_at = $completed.ToString("O")
        duration_ms = [long]($completed - $started).TotalMilliseconds
        cleanup_restoration = [ordered]@{
            required = [bool]$laneContract.touches_live_state
            state = if ([bool]$laneContract.touches_live_state) { "passed" } else { "not_applicable" }
            evidence = if ([bool]$laneContract.touches_live_state) { $restorationReference } else { $null }
        }
        topology_context = if ($Phase -ceq "sit" -and $Lane -in @("sit-rust", "sit-browser")) { $topologyContext } else { $null }
        command_log = $null
        evidence_references = $null
    }

    $phaseLaneIds = @(Get-Sprint8CExpectedLaneIds -Phase $Phase)
    $phaseCompletionAction = $null
    if ($Lane -ceq $phaseLaneIds[-1]) {
        $candidateFingerprint = if ($Phase -in @("validation-preflight", "sit", "uat")) {
            [string]$result.candidate_fingerprint
        } else { $null }
        $phaseCompletionAction = {
            Complete-Sprint8CPhase -Contract $contract -Phase $Phase `
                -Source $sourceAfter -CandidateFingerprint $candidateFingerprint
        }.GetNewClosure()
    }

    $result = Invoke-Sprint8CLanePassTransaction -Contract $contract -Paths $paths `
        -AttemptLog $attemptLog -AttemptReferences $attemptReferences -Result $result `
        -PhaseCompletionAction $phaseCompletionAction
    } catch {
        $lateLaneError = $_
        $failureOutcome = Complete-Sprint8CFailedLaneAttempt -Phase $Phase -Lane $Lane `
            -AttemptRoot $attemptRoot -LaneError $lateLaneError -LaneContract $laneContract `
            -TopologyContext $topologyContext -Project $project `
            -Prerequisites $prerequisites -EnvironmentBefore $environmentBefore
        $emergencyCleanup = $failureOutcome.emergency_cleanup
        if ($null -ne $emergencyCleanup -and
            [string]$emergencyCleanup.state -ceq "failed") {
            throw "Formal lane '$Lane' failed after action execution: $($lateLaneError.Exception.Message) Emergency cleanup also failed: $([string]$emergencyCleanup.message)"
        }
        throw $lateLaneError
    }

    return $result
}

function Assert-Sprint8CExpectedFailure {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$Label
    )
    try { & $Action } catch { return }
    throw "Formal runner self-test expected rejection: $Label"
}

function Test-Sprint8CChronologyPublicationLifecycle {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )

    $root = [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot (
        "tmp/formal-publication-selftest-$([guid]::NewGuid().ToString('N'))"
    )))
    $allowedRoot = [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot "tmp"))
    if (-not $root.StartsWith(
            $allowedRoot + [IO.Path]::DirectorySeparatorChar,
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Formal publication self-test root escaped the repository tmp directory."
    }

    $invalidProvenancePath = Join-Path $root "provenance/defect-provenance.json"
    $insertInvalidProvenance = {
        Write-Sprint8CNewUtf8File -Path $invalidProvenancePath -Text "{ invalid chronology"
    }.GetNewClosure()
    try {
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $attemptLog = Join-Path $root "attempt/command.log"
        $attemptReferences = Join-Path $root "attempt/evidence-references.json"
        Write-Sprint8CNewUtf8File -Path $attemptLog -Text "synthetic formal transcript`n"
        Write-Sprint8CNewJsonFile -Path $attemptReferences -Document ([ordered]@{
            schema_version = 1
            references = @()
        })
        $foreignPairPath = Join-Path $root "foreign/result.json"
        $foreignSidecarPath = "$foreignPairPath.sha256"
        Write-Sprint8CNewUtf8File -Path $foreignSidecarPath -Text "foreign-owned`n"
        $foreignSidecarSha = (
            Get-FileHash -LiteralPath $foreignSidecarPath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        $foreignPublicationRejected = $false
        try {
            Publish-Sprint8CJsonAndSidecar -Path $foreignPairPath `
                -Document ([ordered]@{ state = "passed" }) | Out-Null
        } catch {
            $foreignPublicationRejected = $true
        }
        if (-not $foreignPublicationRejected -or
            (Test-Path -LiteralPath $foreignPairPath -PathType Leaf) -or
            -not (Test-Path -LiteralPath $foreignSidecarPath -PathType Leaf) -or
            (Get-FileHash -LiteralPath $foreignSidecarPath -Algorithm SHA256).Hash.ToLowerInvariant() `
                -cne $foreignSidecarSha) {
            throw "$Phase formal publisher did not preserve a foreign publication member exactly."
        }
        Remove-Item -LiteralPath $foreignSidecarPath -Force

        $partialWritePath = Join-Path $root "faults/partial-write.txt"
        $partialWriteRejected = $false
        try {
            Write-Sprint8CNewUtf8File -Path $partialWritePath -Text "owned`n" `
                -AfterCreateHook { throw "synthetic write failure" }
        } catch {
            if ($_.Exception.Message -notmatch 'synthetic write failure') { throw }
            $partialWriteRejected = $true
        }
        if (-not $partialWriteRejected -or
            (Test-Path -LiteralPath $partialWritePath -PathType Leaf)) {
            throw "$Phase formal publisher left a partial newly-created evidence file."
        }

        $racingPairPath = Join-Path $root "faults/sidecar-race.json"
        $racingSidecarPath = "$racingPairPath.sha256"
        $sidecarRaceHook = {
            [IO.File]::WriteAllText(
                $racingSidecarPath,
                "foreign-race`n",
                [Text.UTF8Encoding]::new($false)
            )
        }.GetNewClosure()
        $sidecarRaceRejected = $false
        try {
            Publish-Sprint8CJsonAndSidecar -Path $racingPairPath `
                -Document ([ordered]@{ state = "passed" }) `
                -BeforeSidecarPublicationHook $sidecarRaceHook | Out-Null
        } catch { $sidecarRaceRejected = $true }
        if (-not $sidecarRaceRejected -or
            (Test-Path -LiteralPath $racingPairPath -PathType Leaf) -or
            (Get-Content -LiteralPath $racingSidecarPath -Raw) -cne "foreign-race`n") {
            throw "$Phase formal publisher did not preserve a sidecar won by a concurrent writer."
        }
        Remove-Item -LiteralPath $racingSidecarPath -Force

        $copySourcePath = Join-Path $root "faults/copy-source.txt"
        $copyDestinationPath = Join-Path $root "faults/copy-destination.txt"
        Write-Sprint8CNewUtf8File -Path $copySourcePath -Text "copy source`n"
        $partialCopyRejected = $false
        try {
            Copy-Sprint8CNewFile -Source $copySourcePath -Destination $copyDestinationPath `
                -AfterDestinationCreateHook { throw "synthetic copy failure" }
        } catch {
            if ($_.Exception.Message -notmatch 'synthetic copy failure') { throw }
            $partialCopyRejected = $true
        }
        if (-not $partialCopyRejected -or
            (Test-Path -LiteralPath $copyDestinationPath -PathType Leaf)) {
            throw "$Phase formal publisher left a partial newly-created copy."
        }
        Remove-Item -LiteralPath $copySourcePath -Force

        $registrationPath = Join-Path $root "faults/registration-failure.txt"
        Write-Sprint8CNewUtf8File -Path $registrationPath -Text "owned-registration`n"
        $registrationSha = (
            Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        $registrationFiles = [Collections.Generic.List[object]]::new()
        $registrationRejected = $false
        try {
            Add-Sprint8CCreatedPublicationFile -CreatedFiles $registrationFiles `
                -Path $registrationPath -ExpectedSha256 $registrationSha `
                -AfterRegistrationHook { throw "synthetic registration failure" }
        } catch {
            if ($_.Exception.Message -notmatch 'synthetic registration failure') { throw }
            $registrationRejected = $true
        }
        Remove-Sprint8CCreatedPublicationFilesExact -CreatedFiles $registrationFiles
        if (-not $registrationRejected -or
            (Test-Path -LiteralPath $registrationPath -PathType Leaf)) {
            throw "$Phase formal publisher did not retain ownership across registration failure."
        }
        $lanePaths = [pscustomobject][ordered]@{
            command_log = Join-Path $root "lane/command.log"
            evidence_references = Join-Path $root "lane/evidence-references.json"
            result = Join-Path $root "lane/result.json"
        }
        $laneResult = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8c"
            phase = $Phase
            lane = "synthetic-lane"
            state = "passed"
            command_log = $null
            evidence_references = $null
        }

        $laneInsertionRejected = $false
        try {
            Publish-Sprint8CFormalLanePassResult -Contract $Contract -Paths $lanePaths `
                -AttemptLog $attemptLog -AttemptReferences $attemptReferences `
                -Result $laneResult -ChronologyEvidenceRootPath $root `
                -BeforeResultPublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $laneInsertionRejected = $true
        }
        $laneCanonicalPaths = @(
            $lanePaths.command_log,
            $lanePaths.evidence_references,
            $lanePaths.result,
            "$($lanePaths.result).sha256"
        )
        if (-not $laneInsertionRejected -or
            @($laneCanonicalPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0 -or
            -not (Test-Path -LiteralPath $attemptLog -PathType Leaf) -or
            -not (Test-Path -LiteralPath $attemptReferences -PathType Leaf)) {
            throw "$Phase formal lane chronology insertion did not preserve only retained attempt evidence."
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force

        $lanePostPublicationRejected = $false
        try {
            Publish-Sprint8CFormalLanePassResult -Contract $Contract -Paths $lanePaths `
                -AttemptLog $attemptLog -AttemptReferences $attemptReferences `
                -Result $laneResult -ChronologyEvidenceRootPath $root `
                -AfterResultPublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $lanePostPublicationRejected = $true
        }
        if (-not $lanePostPublicationRejected -or
            @($laneCanonicalPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "$Phase formal lane post-publication chronology race left a sealed pass result."
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force

        $phaseCompletionInsertion = {
            Write-Sprint8CNewUtf8File -Path $invalidProvenancePath -Text "{ invalid chronology"
            Assert-Sprint8CDefectProvenanceChronology -Contract $Contract `
                -EvidenceRootPath $root | Out-Null
        }.GetNewClosure()
        $phaseCompletionRollbackPassed = $false
        try {
            Invoke-Sprint8CLanePassTransaction -Contract $Contract -Paths $lanePaths `
                -AttemptLog $attemptLog -AttemptReferences $attemptReferences `
                -Result $laneResult -ChronologyEvidenceRootPath $root `
                -PhaseCompletionAction $phaseCompletionInsertion | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $phaseCompletionRollbackPassed = $true
        }
        if (-not $phaseCompletionRollbackPassed -or
            @($laneCanonicalPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "$Phase phase-completion failure stranded a non-overwritable lane result."
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force

        $failureLifecycleState = [pscustomobject]@{
            setup = 0
            teardown = 0
            restore = 0
        }
        $topologySetup = {
            $failureLifecycleState.setup++
        }.GetNewClosure()
        $syntheticTeardown = {
            param($evidencePath)
            $failureLifecycleState.teardown++
            Write-Sprint8CNewJsonFile -Path $evidencePath -Document ([ordered]@{
                schema_version = 1
                state = "passed"
            })
        }.GetNewClosure()
        $syntheticRestore = {
            $failureLifecycleState.restore++
        }.GetNewClosure()
        $syntheticLaneError = $null
        try { throw "synthetic late publication failure" } catch { $syntheticLaneError = $_ }
        $failureOutcome = Complete-Sprint8CFailedLaneAttempt -Phase $Phase `
            -Lane "synthetic-late-lane" -AttemptRoot (Split-Path -Parent $attemptLog) `
            -LaneError $syntheticLaneError `
            -LaneContract ([pscustomobject]@{ touches_live_state = $true }) `
            -TopologyContext ([pscustomobject]@{ state = "retained" }) `
            -Project "synthetic-project" `
            -Prerequisites ([pscustomobject]@{ candidate_fingerprint = ('a' * 64) }) `
            -EnvironmentBefore @{} -TopologyEnvironmentAction $topologySetup `
            -TeardownAction $syntheticTeardown -RestoreEnvironmentAction $syntheticRestore
        $failureReceiptPath = Join-Path (Split-Path -Parent $attemptLog) "failure.json"
        $emergencyReceiptPath = Join-Path (Split-Path -Parent $attemptLog) `
            "emergency-teardown.json"
        $failureDocument = Get-Content -LiteralPath $failureReceiptPath -Raw | ConvertFrom-Json
        $failureLifecyclePassed = (
            $failureLifecycleState.setup -eq 1 -and
            $failureLifecycleState.teardown -eq 1 -and
            $failureLifecycleState.restore -eq 1 -and
            [string]$failureDocument.state -ceq "failed" -and
            [string]$failureDocument.message -ceq "synthetic late publication failure" -and
            [string]$failureOutcome.emergency_cleanup.state -ceq "passed" -and
            (Test-Path -LiteralPath $attemptLog -PathType Leaf) -and
            (Test-Path -LiteralPath $attemptReferences -PathType Leaf)
        )
        if (-not $failureLifecyclePassed) {
            throw "$Phase formal failure finalizer did not prove teardown, restoration, retained attempt evidence, and failure receipt."
        }
        Remove-Item -LiteralPath $failureReceiptPath, $emergencyReceiptPath -Force

        $phaseEvidenceRoot = Join-Path $root "phase"
        $phaseSeedPath = Join-Path $phaseEvidenceRoot "lane-evidence.json"
        Write-Sprint8CNewJsonFile -Path $phaseSeedPath -Document ([ordered]@{
            schema_version = 1
            state = "passed"
        })
        $phaseSeedItem = Get-Item -LiteralPath $phaseSeedPath
        $indexPath = Join-Path $phaseEvidenceRoot "evidence-index.json"
        $certificatePath = Join-Path $phaseEvidenceRoot "result.json"
        $indexDocument = [pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.phase-evidence-index"
            policy_version = "tessara-validation-v2"
            sprint = "sprint-8c"
            phase = $Phase
            attempt = 1
            evidence_root = Get-Sprint8CRepositoryRelativePath -Path $phaseEvidenceRoot
            sealed_at = [DateTimeOffset]::UtcNow.ToString("O")
            entry_count = 1
            entries = @([pscustomobject][ordered]@{
                path = Get-Sprint8CRepositoryRelativePath -Path $phaseSeedPath
                sha256 = (Get-FileHash -LiteralPath $phaseSeedPath -Algorithm SHA256).Hash.ToLowerInvariant()
                size = [long]$phaseSeedItem.Length
                kind = "receipt"
            })
        }
        $certificateDocument = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8c"
            phase = $Phase
            state = "passed"
        }
        $phaseInsertionRejected = $false
        try {
            Publish-Sprint8CPhasePassCertificate -Contract $Contract `
                -IndexPath $indexPath -IndexDocument $indexDocument `
                -ExpectedIndexSha256 (Get-Sprint8CJsonDocumentSha256 -Document $indexDocument) `
                -CertificatePath $certificatePath -CertificateDocument $certificateDocument `
                -ChronologyEvidenceRootPath $root `
                -BeforeCertificatePublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $phaseInsertionRejected = $true
        }
        $phasePassPaths = @(
            $indexPath, "$indexPath.sha256", $certificatePath, "$certificatePath.sha256"
        )
        if (-not $phaseInsertionRejected -or
            @($phasePassPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "$Phase formal phase chronology insertion left a sealed index or pass certificate."
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force

        $phasePostPublicationRejected = $false
        try {
            Publish-Sprint8CPhasePassCertificate -Contract $Contract `
                -IndexPath $indexPath -IndexDocument $indexDocument `
                -ExpectedIndexSha256 (Get-Sprint8CJsonDocumentSha256 -Document $indexDocument) `
                -CertificatePath $certificatePath -CertificateDocument $certificateDocument `
                -ChronologyEvidenceRootPath $root `
                -AfterCertificatePublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $phasePostPublicationRejected = $true
        }
        if (-not $phasePostPublicationRejected -or
            @($phasePassPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "$Phase formal phase post-publication chronology race left a sealed certificate."
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force

        $latePhaseEvidencePath = Join-Path $phaseEvidenceRoot "late-unindexed-evidence.txt"
        $insertLatePhaseEvidence = {
            Write-Sprint8CNewUtf8File -Path $latePhaseEvidencePath -Text "late evidence`n"
        }.GetNewClosure()
        $phaseInventoryRejected = $false
        try {
            Publish-Sprint8CPhasePassCertificate -Contract $Contract `
                -IndexPath $indexPath -IndexDocument $indexDocument `
                -ExpectedIndexSha256 (Get-Sprint8CJsonDocumentSha256 -Document $indexDocument) `
                -CertificatePath $certificatePath -CertificateDocument $certificateDocument `
                -ChronologyEvidenceRootPath $root `
                -BeforeCertificatePublicationHook $insertLatePhaseEvidence | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'evidence inventory changed') { throw }
            $phaseInventoryRejected = $true
        }
        Remove-Item -LiteralPath $latePhaseEvidencePath -Force
        if (-not $phaseInventoryRejected -or
            @($phasePassPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "$Phase formal phase evidence drift left a sealed index."
        }

        $phasePostPublicationInventoryRejected = $false
        try {
            Publish-Sprint8CPhasePassCertificate -Contract $Contract `
                -IndexPath $indexPath -IndexDocument $indexDocument `
                -ExpectedIndexSha256 (Get-Sprint8CJsonDocumentSha256 -Document $indexDocument) `
                -CertificatePath $certificatePath -CertificateDocument $certificateDocument `
                -ChronologyEvidenceRootPath $root `
                -AfterCertificatePublicationHook $insertLatePhaseEvidence | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'evidence inventory changed') { throw }
            $phasePostPublicationInventoryRejected = $true
        }
        Remove-Item -LiteralPath $latePhaseEvidencePath -Force
        if (-not $phasePostPublicationInventoryRejected -or
            @($phasePassPaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "$Phase formal phase post-publication evidence drift left a sealed certificate."
        }

        return [pscustomobject][ordered]@{
            state = "passed"
            foreign_publication_preserved = $foreignPublicationRejected
            partial_write_cleanup_exact = $partialWriteRejected
            partial_copy_cleanup_exact = $partialCopyRejected
            sidecar_create_race_preserved = $sidecarRaceRejected
            registration_failure_cleanup_exact = $registrationRejected
            lane_pass_insertion_rejected = $laneInsertionRejected
            lane_canonical_cleanup_exact = $true
            lane_post_publication_insertion_rolled_back = $lanePostPublicationRejected
            phase_completion_lane_result_rolled_back = $phaseCompletionRollbackPassed
            failure_finalizer_runtime_proven = $failureLifecyclePassed
            phase_pass_insertion_rejected = $phaseInsertionRejected
            phase_index_cleanup_exact = $true
            phase_post_publication_insertion_rolled_back = $phasePostPublicationRejected
            phase_inventory_drift_rolled_back = $phaseInventoryRejected
            phase_post_publication_inventory_drift_rolled_back = `
                $phasePostPublicationInventoryRejected
            sealed_pass_absent = $true
        }
    } finally {
        if (Test-Path -LiteralPath $root) {
            Remove-Item -LiteralPath $root -Recurse -Force
        }
        if (Test-Path -LiteralPath $root) {
            throw "Formal publication self-test cleanup was not exact."
        }
    }
}

function Assert-Sprint8CPublicationBoundaryPlacement {
    $laneSource = ${function:Invoke-Sprint8CFormalLane}.ToString()
    $cleanSourceIndex = $laneSource.IndexOf("Assert-Sprint8CCleanSource", [StringComparison]::Ordinal)
    $initialChronologyIndex = $laneSource.IndexOf(
        "Assert-Sprint8CDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    $prerequisiteIndex = $laneSource.IndexOf(
        "Assert-Sprint8CLanePrerequisites",
        [StringComparison]::Ordinal
    )
    $lanePublicationIndex = $laneSource.IndexOf(
        "Invoke-Sprint8CLanePassTransaction",
        [StringComparison]::Ordinal
    )
    $lateFailureFinalizerIndex = $laneSource.LastIndexOf(
        "Complete-Sprint8CFailedLaneAttempt",
        [StringComparison]::Ordinal
    )
    $failureFinalizerCount = [regex]::Matches(
        $laneSource,
        [regex]::Escape("Complete-Sprint8CFailedLaneAttempt")
    ).Count
    $failureFinalizerSource = ${function:Complete-Sprint8CFailedLaneAttempt}.ToString()
    $canonicalFailureLifecycle = @(@(
        "Invoke-Sprint8CComposeTeardown",
        "Restore-Sprint8CProcessEnvironmentSnapshot",
        'Join-Path $AttemptRoot "failure.json"'
    ) | Where-Object {
        $failureFinalizerSource.IndexOf($_, [StringComparison]::Ordinal) -lt 0
    })
    if ($cleanSourceIndex -lt 0 -or $initialChronologyIndex -le $cleanSourceIndex -or
        $prerequisiteIndex -le $initialChronologyIndex -or
        $lanePublicationIndex -le $prerequisiteIndex -or
        $lateFailureFinalizerIndex -le $lanePublicationIndex -or
        $failureFinalizerCount -ne 2 -or $canonicalFailureLifecycle.Count -ne 0) {
        throw "Formal lane chronology/authentication/publication placement is not fail-closed."
    }

    $lanePublisherSource = ${function:Publish-Sprint8CFormalLanePassResult}.ToString()
    $laneResultPublication = $lanePublisherSource.IndexOf(
        "Publish-Sprint8CJsonAndSidecar",
        [StringComparison]::Ordinal
    )
    $laneFirstChronology = $lanePublisherSource.IndexOf(
        "Assert-Sprint8CDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    $laneFinalChronology = $lanePublisherSource.LastIndexOf(
        "Assert-Sprint8CDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    if ($laneFirstChronology -lt 0 -or
        $laneResultPublication -le $laneFirstChronology -or
        $laneFinalChronology -le $laneResultPublication) {
        throw "Formal lane pass result is not guarded at its canonical publication boundary."
    }

    $phaseSource = ${function:Complete-Sprint8CPhase}.ToString()
    if ($phaseSource.IndexOf(
            "Publish-Sprint8CPhasePassCertificate",
            [StringComparison]::Ordinal
        ) -lt 0) {
        throw "Formal phase completion bypasses the transactional pass publisher."
    }
    $phasePublisherSource = ${function:Publish-Sprint8CPhasePassCertificate}.ToString()
    $phaseIndexPublication = $phasePublisherSource.IndexOf(
        "Publish-Sprint8CJsonAndSidecar",
        [StringComparison]::Ordinal
    )
    $phaseFinalChronology = $phasePublisherSource.LastIndexOf(
        "Assert-Sprint8CDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    $phaseCertificatePublication = $phasePublisherSource.LastIndexOf(
        "Publish-Sprint8CJsonAndSidecar",
        [StringComparison]::Ordinal
    )
    $phaseFinalEvidenceAudit = $phasePublisherSource.LastIndexOf(
        "Assert-Sprint8CPhaseEvidenceSnapshot",
        [StringComparison]::Ordinal
    )
    $betweenPhasePublications = if ($phaseIndexPublication -ge 0 -and
        $phaseCertificatePublication -gt $phaseIndexPublication) {
        $phasePublisherSource.Substring(
            $phaseIndexPublication,
            $phaseCertificatePublication - $phaseIndexPublication
        )
    } else { "" }
    if ($phaseIndexPublication -lt 0 -or
        $betweenPhasePublications.IndexOf(
            "Assert-Sprint8CDefectProvenanceChronology",
            [StringComparison]::Ordinal
        ) -lt 0 -or
        $phaseFinalChronology -le $phaseCertificatePublication -or
        $phaseFinalEvidenceAudit -le $phaseCertificatePublication) {
        throw "Formal phase index/certificate chronology placement is not fail-closed."
    }
}

function Test-Sprint8CFormalRunner {
    param([Parameter(Mandatory)][string]$Phase)
    $contract = Get-Sprint8CContract
    $actionMap = Get-Sprint8CFormalActionMap
    $null = Assert-Sprint8CFormalProfile -Contract $contract -Phase $Phase -ActionMap $actionMap
    Assert-Sprint8CPublicationBoundaryPlacement
    $expectedIds = @(Get-Sprint8CExpectedLaneIds -Phase $Phase)

    $chronologyRoot = Join-Path $script:Sprint8CRepositoryRoot `
        "tmp/formal-chronology-selftest-$([guid]::NewGuid().ToString('N'))"
    $chronologyRejected = $false
    try {
        [IO.Directory]::CreateDirectory($chronologyRoot) | Out-Null
        $chronology = Assert-Sprint8CDefectProvenanceChronology -Contract $contract `
            -EvidenceRootPath $chronologyRoot
        if ([string]$chronology.state -cne "passed") {
            throw "$Phase formal runner self-test did not authenticate empty synthetic provenance."
        }
        $invalidChronologyPath = Join-Path $chronologyRoot `
            "attempts/invalid/defect-provenance.json"
        Write-Sprint8CNewUtf8File -Path $invalidChronologyPath -Text "{ invalid chronology"
        try {
            Assert-Sprint8CDefectProvenanceChronology -Contract $contract `
                -EvidenceRootPath $chronologyRoot | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $chronologyRejected = $true
        }
        if (-not $chronologyRejected) {
            throw "$Phase formal runner self-test admitted unresolved defect provenance."
        }
    } finally {
        if (Test-Path -LiteralPath $chronologyRoot) {
            Remove-Item -LiteralPath $chronologyRoot -Recurse -Force
        }
    }
    $publicationLifecycle = Test-Sprint8CChronologyPublicationLifecycle `
        -Contract $contract -Phase $Phase

    $mutatedIdentity = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $identityLane = @($mutatedIdentity.lanes | Where-Object { [string]$_.phase -ceq $Phase })[0]
    $identityLane.id = "$([string]$identityLane.id)-wrong"
    Assert-Sprint8CExpectedFailure -Label "$Phase lane identity" -Action {
        Assert-Sprint8CFormalProfile -Contract $mutatedIdentity -Phase $Phase -ActionMap $actionMap
    }

    $mutatedOrder = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $phaseIndexes = @(for ($index = 0; $index -lt $mutatedOrder.lanes.Count; $index++) {
        if ([string]$mutatedOrder.lanes[$index].phase -ceq $Phase) { $index }
    })
    if ($phaseIndexes.Count -gt 1) {
        $first = $mutatedOrder.lanes[$phaseIndexes[0]]
        $mutatedOrder.lanes[$phaseIndexes[0]] = $mutatedOrder.lanes[$phaseIndexes[1]]
        $mutatedOrder.lanes[$phaseIndexes[1]] = $first
        Assert-Sprint8CExpectedFailure -Label "$Phase reordered lane" -Action {
            Assert-Sprint8CFormalProfile -Contract $mutatedOrder -Phase $Phase -ActionMap $actionMap
        }
    }

    $mutatedPrerequisite = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $targetLane = @($mutatedPrerequisite.lanes | Where-Object { [string]$_.id -ceq $expectedIds[-1] })[0]
    $targetLane.prerequisites = @()
    Assert-Sprint8CExpectedFailure -Label "$Phase prerequisite removal" -Action {
        Assert-Sprint8CFormalProfile -Contract $mutatedPrerequisite -Phase $Phase -ActionMap $actionMap
    }

    $mutatedEnvironment = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $environmentLane = @($mutatedEnvironment.lanes | Where-Object { [string]$_.phase -ceq $Phase })[0]
    $environmentLane.touches_live_state = -not [bool]$environmentLane.touches_live_state
    Assert-Sprint8CExpectedFailure -Label "$Phase environment identity" -Action {
        Assert-Sprint8CFormalProfile -Contract $mutatedEnvironment -Phase $Phase -ActionMap $actionMap
    }

    $mutatedEvidence = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $mutatedEvidence.evidence_policy.root = "artifacts/not-sprint-8c-closeout"
    Assert-Sprint8CExpectedFailure -Label "$Phase evidence root" -Action {
        Assert-Sprint8CFormalProfile -Contract $mutatedEvidence -Phase $Phase -ActionMap $actionMap
    }

    $priorRunRoot = [Environment]::GetEnvironmentVariable(
        "TESSARA_SPRINT_8C_RUN_EVIDENCE_ROOT",
        "Process"
    )
    try {
        $runIdentity = "selftest-$([guid]::NewGuid().ToString('N'))"
        $runRelative = "$([string]$contract.evidence_policy.root)/runs/$runIdentity"
        [Environment]::SetEnvironmentVariable(
            "TESSARA_SPRINT_8C_RUN_EVIDENCE_ROOT",
            $runRelative,
            "Process"
        )
        $runPaths = Get-Sprint8CLaneEvidencePaths -Contract $contract -Phase $Phase -Lane $expectedIds[0]
        if ((Get-Sprint8CRepositoryRelativePath -Path $runPaths.result) -cne
            "$runRelative/$Phase/lanes/$($expectedIds[0])/result.json") {
            throw "$Phase did not isolate its canonical result inside the selected run evidence root."
        }
        [Environment]::SetEnvironmentVariable(
            "TESSARA_SPRINT_8C_RUN_EVIDENCE_ROOT",
            "artifacts/sprint-8c-outside-canonical-runs",
            "Process"
        )
        Assert-Sprint8CExpectedFailure -Label "$Phase unbounded run evidence root" -Action {
            Get-Sprint8CEvidenceRoot -Contract $contract | Out-Null
        }
    } finally {
        [Environment]::SetEnvironmentVariable(
            "TESSARA_SPRINT_8C_RUN_EVIDENCE_ROOT",
            $priorRunRoot,
            "Process"
        )
    }

    $browserLaneId = if ($Phase -ceq "candidate-rehearsal") {
        "rehearsal-browser"
    } elseif ($Phase -ceq "sit") {
        "sit-browser"
    } else { $null }
    if ($null -ne $browserLaneId) {
        $browserActions = @($actionMap[$browserLaneId] | Where-Object {
            [string]$_.id -ceq "browser-acceptance-batches"
        })
        if ($browserActions.Count -ne 1 -or
            @($browserActions | Where-Object {
                [string]$_.playwright_data_state -cne "fresh"
            }).Count -ne 0) {
            throw "$browserLaneId does not bind its canonical browser-batch action to fresh reference data."
        }
        $browserAction = $browserActions[0]
        $expectedArguments = @(
            "-ManifestPath", "end2end/acceptance-manifest.json",
            "-ExpectedDataState", "fresh",
            "-EvidencePath", "{evidence}"
        )
        if ([string]$browserAction.kind -cne "pwsh" -or
            [string]$browserAction.command -cne "scripts/tessara-validation-evidence-browser-batches.ps1" -or
            -not [bool]$browserAction.produces_evidence -or
            (@($browserAction.arguments) -join "`n") -cne ($expectedArguments -join "`n")) {
            throw "$browserLaneId does not use the exact fresh-process browser acceptance boundary."
        }
        $manifest = Get-Content -LiteralPath (
            Join-Path $script:Sprint8CRepositoryRoot "end2end/acceptance-manifest.json"
        ) -Raw | ConvertFrom-Json
        if ([int]$manifest.expected_total -ne 105 -or @($manifest.files).Count -ne 12) {
            throw "$browserLaneId is not bound to the exact 105-test, 12-file acceptance inventory."
        }
        if ($Phase -ceq "candidate-rehearsal") {
            $platformSelfTestOutput = @(& pwsh -NoProfile -File (
                Join-Path $script:Sprint8CRepositoryRoot `
                    "scripts/tessara-validation-evidence-browser-batches.ps1"
            ) -SelfTest 2>&1 | ForEach-Object { [string]$_ })
            if ($LASTEXITCODE -ne 0 -or
                @($platformSelfTestOutput | Where-Object {
                    $_ -ceq "Synthetic browser lifecycle certification passed: 12 fresh processes."
                }).Count -ne 1) {
                throw "Candidate Rehearsal browser platform certification did not pass exactly."
            }
        }
        $browserAction.playwright_data_state = ""
        try {
            Assert-Sprint8CExpectedFailure -Label "$Phase missing Playwright data state" -Action {
                Assert-Sprint8CFormalProfile -Contract $contract -Phase $Phase -ActionMap $actionMap
            }
        } finally {
            $browserAction.playwright_data_state = "fresh"
        }
    }

    if ($Phase -ceq "sit") {
        $sitRustActions = @($actionMap["sit-rust"])
        $expectedSitRustActionIds = @(
            "frozen-sit-setup",
            "workspace-rust",
            "restoration-checkpoint",
            "post-smoke-teardown",
            "fresh-browser-handoff"
        )
        Assert-Sprint8CExactSequence -Expected $expectedSitRustActionIds `
            -Actual @($sitRustActions.id) -Label "SIT fresh browser handoff"
        $handoff = $sitRustActions[-1]
        if ([string]$sitRustActions[-2].kind -cne "teardown" -or
            [string]$handoff.kind -cne "pwsh" -or
            [string]$handoff.command -cne "scripts/materialize-sprint-8c.ps1" -or
            @($handoff.arguments) -cnotcontains "Reference" -or
            @($handoff.arguments) -cnotcontains "-KeepTopology") {
            throw "SIT Rust must remove the mutation-bearing smoke topology and retain a newly materialized fresh Reference topology for SIT browser."
        }
    }

    $firstLane = Get-Sprint8CLaneContract -Contract $contract -Phase $Phase -Lane $expectedIds[0]
    $contractCopy = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $contractCopy.evidence_policy.root = "tmp/formal-selftest-$([guid]::NewGuid().ToString('N'))"
    $source = [pscustomobject]@{ commit = "1" * 40; tree = "2" * 40; dirty = $false; branch = "codex/selftest" }
    Assert-Sprint8CExpectedFailure -Label "$Phase missing prerequisite" -Action {
        Assert-Sprint8CLanePrerequisites -Contract $contractCopy -LaneContract $firstLane -Source $source
    }

    $missingHarness = @(New-Sprint8CPowerShellAction -Id "missing" -Script "scripts/definitely-missing-sprint-8c-harness.ps1")
    Assert-Sprint8CExpectedFailure -Label "$Phase missing harness" -Action {
        Assert-Sprint8CActionAvailability -Actions $missingHarness
    }

    $emptyArguments = @(Expand-Sprint8CActionArguments -Arguments @() -Project $null `
        -EvidencePath "tmp/formal-selftest-evidence.json" -CandidateFingerprint $null -TopologyContext $null)
    if ($emptyArguments.Count -ne 0) {
        throw "$Phase zero-argument action expansion produced unexpected arguments."
    }

    $stdoutAttemptRoot = Join-Path $script:Sprint8CRepositoryRoot `
        "tmp/formal-action-stdout-$([guid]::NewGuid().ToString('N'))"
    try {
        $stdoutAction = New-Sprint8CProgramAction -Id "stdout" -Program "pwsh" `
            -Arguments @("-NoProfile", "-Command", "Write-Output 'child-output'")
        $stdoutResult = @(Invoke-Sprint8CAction -Contract $contract -Action $stdoutAction `
            -AttemptRoot $stdoutAttemptRoot -Project $null -CandidateFingerprint $null `
            -TopologyContext $null -Source $source -SuppressChildOutput)
        if ($stdoutResult.Count -ne 1 -or
            -not ($stdoutResult[0].PSObject.Properties.Name -contains "result_reference")) {
            throw "$Phase child stdout contaminated the typed action result."
        }
    } finally {
        if (Test-Path -LiteralPath $stdoutAttemptRoot) {
            Remove-Item -LiteralPath $stdoutAttemptRoot -Recurse -Force
        }
    }

    Assert-Sprint8CExpectedFailure -Label "$Phase unmapped selector" -Action {
        Invoke-Sprint8CPhaseRunner -Phase $Phase -Lane "not-a-sprint-8c-lane"
    }
    Assert-Sprint8CExpectedFailure -Label "$Phase ambiguous selector mode" -Action {
        Invoke-Sprint8CPhaseRunner -Phase $Phase -ListLanes -SelfTest
    }

    $phaseLanes = @($contract.lanes | Where-Object { [string]$_.phase -ceq $Phase })
    foreach ($lane in $phaseLanes) {
        $expectedProject = Get-Sprint8CExpectedProject -Phase $Phase -LaneContract $lane
        if ([bool]$lane.touches_live_state -and $expectedProject -cnotmatch '^tessara-s8c-[a-z0-9-]+$') {
            throw "Self-test found unsafe project identity for '$($lane.id)'."
        }
        $paths = Get-Sprint8CLaneEvidencePaths -Contract $contract -Phase $Phase -Lane ([string]$lane.id)
        $effectiveRoot = Get-Sprint8CRepositoryRelativePath -Path (
            Get-Sprint8CEvidenceRoot -Contract $contract
        )
        if ((Get-Sprint8CRepositoryRelativePath -Path $paths.result) -cne
            "$effectiveRoot/$Phase/lanes/$([string]$lane.id)/result.json") {
            throw "Self-test found an incorrect result path for '$($lane.id)'."
        }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8c.formal-runner-selftest"
        sprint = "sprint-8c"
        phase = $Phase
        state = "passed"
        lane_count = $expectedIds.Count
        lanes = $expectedIds
        verified = @(
            "identity", "order", "prerequisites", "environment", "evidence-mapping",
            "playwright-data-state", "missing-prerequisite", "missing-harness",
            "zero-argument-action", "typed-action-result", "unmapped-selector", "exclusive-mode",
            "sit-fresh-browser-handoff", "defect-provenance-chronology",
            "chronology-publication-rollback"
        )
        unresolved_provenance_rejected = $chronologyRejected
        chronology_publication_lifecycle = $publicationLifecycle
    }
}

function Invoke-Sprint8CPhaseRunner {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [string]$Lane,
        [switch]$ListLanes,
        [switch]$SelfTest
    )
    $expected = @(Get-Sprint8CExpectedLaneIds -Phase $Phase)
    if ($ListLanes -and $SelfTest) {
        throw "-ListLanes and -SelfTest are mutually exclusive."
    }
    if (($ListLanes -or $SelfTest) -and -not [string]::IsNullOrWhiteSpace($Lane)) {
        throw "-Lane cannot be combined with -ListLanes or -SelfTest."
    }
    if ($ListLanes) { return $expected }
    if ($SelfTest) { return Test-Sprint8CFormalRunner -Phase $Phase }
    if ($Phase -ceq "validation-preflight") {
        if (-not [string]::IsNullOrWhiteSpace($Lane) -and $Lane -cne "preflight-freeze") {
            throw "Preflight has exactly one implicit lane: preflight-freeze."
        }
        $Lane = "preflight-freeze"
    } elseif ([string]::IsNullOrWhiteSpace($Lane)) {
        throw "-Lane is required. Select exactly one of: $($expected -join ', ')."
    }
    if ($Lane -cnotin $expected) { throw "Unknown $Phase lane '$Lane'." }
    return Invoke-Sprint8CFormalLane -Phase $Phase -Lane $Lane
}

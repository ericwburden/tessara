Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:PolicyVersion = "tessara-validation-v2"
$script:SchemaRoot = Join-Path (Split-Path -Parent $PSScriptRoot) ".codex/skills/tessara-sprint-validation/references"
$script:SchemaFiles = @{
    validation_contract = "validation-contract.schema.json"
    implementation_readiness = "implementation-readiness.schema.json"
    phase_certificate = "phase-certificate.schema.json"
    correction_impact = "correction-impact-assessment-v2.schema.json"
    phase_evidence_index = "phase-evidence-index.schema.json"
    evidence_chain = "evidence-chain.schema.json"
    defect_provenance = "defect-provenance.schema.json"
}

function Get-TessaraValidationPolicyVersion {
    return $script:PolicyVersion
}

function Get-TessaraValidationSchemaPath {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [ValidateSet(
            "validation_contract",
            "implementation_readiness",
            "phase_certificate",
            "correction_impact",
            "phase_evidence_index",
            "evidence_chain",
            "defect_provenance"
        )]
        [string]$Kind
    )

    $path = Join-Path $script:SchemaRoot $script:SchemaFiles[$Kind]
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Tessara validation schema is missing: $path"
    }
    return $path
}

function Get-TessaraValidationSha256 {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$Path)

    $resolved = (Resolve-Path -LiteralPath $Path -ErrorAction Stop).Path
    return (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TessaraValidationChangedPaths {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$BaselineCommit,
        [Parameter(Mandatory)][string]$CurrentCommit
    )

    $output = @(& git -C $RepositoryRoot diff --name-only --diff-filter=ACDMRTUXB $BaselineCommit $CurrentCommit)
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to derive validation impact paths from '$BaselineCommit' to '$CurrentCommit'."
    }
    return @($output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        ForEach-Object { ConvertTo-TessaraRepositoryPath -Path ([string]$_) } |
        Sort-Object -Unique)
}

function Get-TessaraDependencyFingerprints {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [string]$Commit = "HEAD"
    )

    $null = Assert-TessaraValidationContract -Contract $Contract
    $treeLines = @(& git -C $RepositoryRoot ls-tree -r $Commit)
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to read Git tree '$Commit' for validation dependency fingerprints."
    }
    $tracked = [Collections.Generic.List[object]]::new()
    foreach ($line in $treeLines) {
        if ([string]$line -notmatch '^[0-7]{6}\s+\S+\s+([0-9a-f]+)\t(.+)$') {
            throw "Unexpected Git tree entry while fingerprinting validation dependencies: '$line'."
        }
        $tracked.Add([pscustomobject]@{
            path = ConvertTo-TessaraRepositoryPath -Path $Matches[2]
            object_id = $Matches[1]
        })
    }

    $fingerprints = [Collections.Generic.List[object]]::new()
    foreach ($domain in @($Contract.dependency_domains)) {
        $matches = @($tracked | Where-Object {
            $candidate = [string]$_.path
            @($domain.tracked_inputs | Where-Object {
                $candidate -clike (ConvertTo-TessaraRepositoryPath -Path ([string]$_))
            }).Count -gt 0
        } | Sort-Object path)
        $canonical = @($matches | ForEach-Object { "$($_.path)`0$($_.object_id)" }) -join "`n"
        $bytes = [Text.Encoding]::UTF8.GetBytes($canonical)
        $hashBytes = [Security.Cryptography.SHA256]::HashData($bytes)
        $fingerprints.Add([pscustomobject]@{
            domain = [string]$domain.name
            sha256 = [Convert]::ToHexString($hashBytes).ToLowerInvariant()
        })
    }
    return @($fingerprints | Sort-Object domain)
}

function Assert-TessaraJsonSchema {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]
        [ValidateSet(
            "validation_contract",
            "implementation_readiness",
            "phase_certificate",
            "correction_impact",
            "phase_evidence_index",
            "evidence_chain",
            "defect_provenance"
        )]
        [string]$Kind,
        [string]$Label = $Kind
    )

    $json = $Document | ConvertTo-Json -Depth 100 -Compress
    $schema = Get-TessaraValidationSchemaPath -Kind $Kind
    $errors = $null
    if (-not (Test-Json -Json $json -SchemaFile $schema -ErrorVariable errors)) {
        $message = @($errors | ForEach-Object { $_.Exception.Message }) -join "; "
        throw "$Label does not satisfy the Tessara $Kind schema. $message"
    }
}

function ConvertTo-TessaraRepositoryPath {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$Path)

    $normalized = $Path.Replace("\", "/").Trim()
    if ([string]::IsNullOrWhiteSpace($normalized) -or
        [IO.Path]::IsPathRooted($normalized) -or
        $normalized -match "(^|/)\.\.(/|$)" -or
        $normalized -match "(^|/)\.(/|$)" -or
        $normalized.Contains("//") -or
        $normalized.StartsWith("/") -or
        $normalized.EndsWith("/") -or
        $normalized.Contains(":")) {
        throw "Validation evidence and contract paths must be repository-relative and traversal-free: '$Path'."
    }
    return $normalized
}

function Test-TessaraContainedFileSystemPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path,
        [switch]$AllowRoot
    )

    $relative = [IO.Path]::GetRelativePath(
        [IO.Path]::GetFullPath($Root),
        [IO.Path]::GetFullPath($Path)
    )
    if ([IO.Path]::IsPathRooted($relative) -or
        $relative -eq ".." -or
        $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)") -or
        $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)")) {
        return $false
    }
    return $AllowRoot -or $relative -ne "."
}

function Assert-TessaraFileSystemPathHasNoLinks {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    $rootPath = [IO.Path]::GetFullPath($Root)
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not (Test-TessaraContainedFileSystemPath -Root $rootPath -Path $fullPath -AllowRoot)) {
        throw "$Label escapes the repository: $fullPath"
    }
    $relative = [IO.Path]::GetRelativePath($rootPath, $fullPath)
    $segments = if ($relative -eq ".") {
        @()
    } else {
        @($relative.Split(
            @([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar),
            [StringSplitOptions]::RemoveEmptyEntries
        ))
    }
    $current = $rootPath
    foreach ($segment in $segments) {
        $current = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $current)) { break }
        $item = Get-Item -LiteralPath $current -Force
        $linkType = if ($item.PSObject.Properties.Name -contains "LinkType") {
            [string]$item.LinkType
        } else { "" }
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            -not [string]::IsNullOrWhiteSpace($linkType)) {
            throw "$Label traverses a symbolic link or reparse point: $current"
        }
    }
}

function Read-TessaraValidationFileSnapshot {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::Open(
        $Path,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        [IO.FileShare]::Read
    )
    try {
        $memory = [IO.MemoryStream]::new()
        try {
            $stream.CopyTo($memory)
            $bytes = $memory.ToArray()
        } finally {
            $memory.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
    [pscustomobject][ordered]@{
        bytes = $bytes
        sha256 = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($bytes)
        ).ToLowerInvariant()
    }
}

function Get-TessaraGitBlobSha256 {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Commit,
        [Parameter(Mandatory)][string]$Path
    )

    if ($Commit -cnotmatch '^[0-9a-f]{40,64}$') { return $null }
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
        if ($process.ExitCode -ne 0) { return $null }
        return [Convert]::ToHexString($hash).ToLowerInvariant()
    } finally {
        $process.Dispose()
    }
}

function Get-TessaraGitSourceIdentityAuthentication {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)]$Identity
    )

    $commit = [string]$Identity.commit
    $tree = [string]$Identity.tree
    if ($commit -cnotmatch '^[0-9a-f]{40,64}$' -or
        $tree -cnotmatch '^[0-9a-f]{40,64}$') {
        return [pscustomobject]@{ state = "failed"; reason = "invalid-identity-shape" }
    }
    $objectType = (@(& git -C $RepositoryRoot cat-file -t $commit 2>$null) -join "").Trim()
    if ($LASTEXITCODE -ne 0 -or $objectType -cne "commit") {
        return [pscustomobject]@{ state = "failed"; reason = "commit-not-found" }
    }
    $actualTree = (@(& git -C $RepositoryRoot rev-parse "$commit`^{tree}" 2>$null) -join "").Trim()
    if ($LASTEXITCODE -ne 0 -or $actualTree -cne $tree) {
        return [pscustomobject]@{ state = "failed"; reason = "tree-mismatch" }
    }
    return [pscustomobject][ordered]@{
        state = "authenticated"
        commit = $commit
        tree = $tree
    }
}

function Get-TessaraEvidenceReferenceAuthentication {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)]$Reference,
        [string[]]$SourceCommits = @(),
        [switch]$AllowSourceCommit
    )

    $relativePath = ConvertTo-TessaraRepositoryPath -Path ([string]$Reference.path)
    $expectedSha = [string]$Reference.sha256
    $fullPath = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $relativePath))
    if (-not (Test-TessaraContainedFileSystemPath -Root $RepositoryRoot -Path $fullPath)) {
        throw "Defect-provenance evidence reference escapes the repository: $relativePath"
    }
    if (Test-Path -LiteralPath $fullPath -PathType Leaf) {
        Assert-TessaraFileSystemPathHasNoLinks -Root $RepositoryRoot -Path $fullPath `
            -Label "Defect-provenance evidence reference '$relativePath'"
        $snapshot = Read-TessaraValidationFileSnapshot -Path $fullPath
        if ([string]$snapshot.sha256 -ceq $expectedSha) {
            return [pscustomobject][ordered]@{
                state = "authenticated"
                basis = "retained-file"
                path = $relativePath
                sha256 = $expectedSha
                full_path = $fullPath
            }
        }
    }
    if (-not $AllowSourceCommit) {
        return [pscustomobject][ordered]@{
            state = "failed"
            basis = "retained-file-required"
            path = $relativePath
            sha256 = $expectedSha
        }
    }
    foreach ($commit in @($SourceCommits | Sort-Object -Unique)) {
        $blobSha = Get-TessaraGitBlobSha256 -RepositoryRoot $RepositoryRoot `
            -Commit $commit -Path $relativePath
        if (-not [string]::IsNullOrWhiteSpace($blobSha) -and $blobSha -ceq $expectedSha) {
            return [pscustomobject][ordered]@{
                state = "authenticated"
                basis = "source-commit"
                path = $relativePath
                sha256 = $expectedSha
                commit = $commit
            }
        }
    }
    [pscustomobject][ordered]@{
        state = "failed"
        basis = "missing-or-hash-mismatch"
        path = $relativePath
        sha256 = $expectedSha
    }
}

function Get-TessaraDefectProvenanceChronology {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Sprint
    )

    if (-not (Test-Path -LiteralPath $RepositoryRoot -PathType Container)) {
        throw "Defect-provenance repository root is missing: $RepositoryRoot"
    }
    $repositoryPath = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $RepositoryRoot).Path).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    )
    $evidenceRootSegments = @($EvidenceRoot -split '[\\/]' | Where-Object {
        -not [string]::IsNullOrEmpty($_)
    })
    if (@($evidenceRootSegments | Where-Object { $_ -ceq ".." }).Count -ne 0) {
        throw "Defect-provenance evidence root contains traversal: $EvidenceRoot"
    }
    $evidenceCandidate = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repositoryPath $EvidenceRoot))
    }
    if (-not (Test-TessaraContainedFileSystemPath -Root $repositoryPath `
            -Path $evidenceCandidate)) {
        throw "Defect-provenance evidence root escapes the repository: $EvidenceRoot"
    }
    if (-not (Test-Path -LiteralPath $evidenceCandidate -PathType Container)) {
        throw "Defect-provenance evidence root is missing: $evidenceCandidate"
    }
    Assert-TessaraFileSystemPathHasNoLinks -Root $repositoryPath -Path $evidenceCandidate `
        -Label "Defect-provenance evidence root"
    $evidencePath = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $evidenceCandidate).Path)

    $schemaPath = Get-TessaraValidationSchemaPath -Kind defect_provenance
    $pathComparer = if ([OperatingSystem]::IsWindows()) {
        [StringComparer]::OrdinalIgnoreCase
    } else {
        [StringComparer]::Ordinal
    }
    $records = [Collections.Generic.List[object]]::new()
    $structuralIssues = [Collections.Generic.List[string]]::new()
    $initialEntries = @(Get-ChildItem -LiteralPath $evidencePath -Recurse -Force)
    foreach ($entry in @($initialEntries | Where-Object {
            ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            ($_.PSObject.Properties.Name -contains "LinkType" -and
                -not [string]::IsNullOrWhiteSpace([string]$_.LinkType))
        })) {
        throw "Defect-provenance evidence root contains a symbolic link or reparse point: $($entry.FullName)"
    }
    $initialFiles = @($initialEntries | Where-Object {
        -not $_.PSIsContainer -and $_.Name -ceq "defect-provenance.json"
    } | Sort-Object FullName)
    foreach ($file in $initialFiles) {
        $fullPath = [IO.Path]::GetFullPath($file.FullName)
        if (-not (Test-TessaraContainedFileSystemPath -Root $evidencePath -Path $fullPath)) {
            throw "Defect-provenance record escapes its evidence root: $fullPath"
        }
        Assert-TessaraFileSystemPathHasNoLinks -Root $repositoryPath -Path $fullPath `
            -Label "Defect-provenance record"
        $relativePath = ConvertTo-TessaraRepositoryPath -Path (
            [IO.Path]::GetRelativePath($repositoryPath, $fullPath)
        )
        $snapshot = Read-TessaraValidationFileSnapshot -Path $fullPath
        $sha256 = [string]$snapshot.sha256
        $localIssues = [Collections.Generic.List[string]]::new()
        $sidecarPath = "$fullPath.sha256"
        $sidecarState = "not_present"
        $sidecarSnapshotSha = $null
        if (Test-Path -LiteralPath $sidecarPath -PathType Leaf) {
            Assert-TessaraFileSystemPathHasNoLinks -Root $repositoryPath -Path $sidecarPath `
                -Label "Defect-provenance sidecar"
            $sidecarSnapshot = Read-TessaraValidationFileSnapshot -Path $sidecarPath
            $sidecarSnapshotSha = [string]$sidecarSnapshot.sha256
            $sidecarText = [Text.UTF8Encoding]::new($false, $true).GetString(
                [byte[]]$sidecarSnapshot.bytes
            ).Trim()
            $sidecarMatch = [regex]::Match(
                $sidecarText,
                '^([0-9a-f]{64})(?:[\t ]+\*?defect-provenance\.json)?$',
                [Text.RegularExpressions.RegexOptions]::CultureInvariant
            )
            $declaredSha = if ($sidecarMatch.Success) {
                [string]$sidecarMatch.Groups[1].Value
            } else { "" }
            if (-not $sidecarMatch.Success -or $declaredSha -cne $sha256) {
                $localIssues.Add("sidecar does not authenticate the retained record")
                $sidecarState = "invalid"
            } else {
                $sidecarState = "authenticated"
            }
        }

        $recordBytes = [byte[]]$snapshot.bytes
        $recordOffset = if ($recordBytes.Length -ge 3 -and
            $recordBytes[0] -eq 0xef -and $recordBytes[1] -eq 0xbb -and
            $recordBytes[2] -eq 0xbf) { 3 } else { 0 }
        $raw = $null
        $document = $null
        $parseError = $null
        try {
            $raw = [Text.UTF8Encoding]::new($false, $true).GetString(
                $recordBytes,
                $recordOffset,
                $recordBytes.Length - $recordOffset
            )
            # Keep RFC 3339 values as their exact wire strings. The default JSON
            # conversion materializes them as local DateTime values and loses the
            # original offset before chronology ordering is evaluated.
            $document = $raw | ConvertFrom-Json -Depth 100 -DateKind String
        } catch {
            $parseError = $_.Exception.Message
        }
        $schemaErrors = @()
        $schemaValid = $false
        if ($null -ne $document -and
            $document.PSObject.Properties.Name -contains "created_at") {
            $validationErrors = $null
            $schemaValid = Test-Json -Json $raw -SchemaFile $schemaPath `
                -ErrorAction SilentlyContinue -ErrorVariable validationErrors
            $schemaErrors = @($validationErrors | ForEach-Object { $_.Exception.Message })
        }
        $recordSprint = if ($null -ne $document -and
            $document.PSObject.Properties.Name -contains "sprint") {
            [string]$document.sprint
        } else { "" }
        if (-not [string]::IsNullOrWhiteSpace($recordSprint) -and
            $recordSprint -cne $Sprint) {
            $schemaValid = $false
            $schemaErrors = @($schemaErrors) + "Record belongs to sprint '$recordSprint', not '$Sprint'."
            $structuralIssues.Add("Defect-provenance record '$relativePath' belongs to sprint '$recordSprint', not '$Sprint'.")
        }
        $recordId = if ($null -ne $document -and
            $document.PSObject.Properties.Name -contains "record_id") {
            [string]$document.record_id
        } else { "" }
        $status = if ($null -ne $document -and
            $document.PSObject.Properties.Name -contains "status") {
            [string]$document.status
        } else { "" }
        $createdAt = $null
        $updatedAt = $null
        if ($null -ne $document) {
        }
        if ($null -ne $document -and
            $document.PSObject.Properties.Name -contains "updated_at") {
            try {
                $createdAt = [DateTimeOffset]::Parse(
                    [string]$document.created_at,
                    [Globalization.CultureInfo]::InvariantCulture,
                    [Globalization.DateTimeStyles]::RoundtripKind
                )
            } catch {
                $createdAt = $null
            }
            try {
                $updatedAt = [DateTimeOffset]::Parse(
                    [string]$document.updated_at,
                    [Globalization.CultureInfo]::InvariantCulture,
                    [Globalization.DateTimeStyles]::RoundtripKind
                )
            } catch {
                $updatedAt = $null
            }
        }
        if ($schemaValid -and ($null -eq $createdAt -or $null -eq $updatedAt)) {
            $localIssues.Add("created_at and updated_at must be parseable date-time values")
        } elseif ($schemaValid -and $updatedAt -lt $createdAt) {
            $localIssues.Add("updated_at precedes created_at")
        }

        $referenceCount = 0
        $authenticatedReferenceCount = 0
        $retainedReferenceSnapshots = [Collections.Generic.List[object]]::new()
        if ($schemaValid) {
            $referenceEntries = [Collections.Generic.List[object]]::new()
            $referenceEntries.Add([pscustomobject]@{
                label = "trigger.failed_receipt"
                reference = $document.trigger.failed_receipt
                allow_source_commit = $false
            })
            foreach ($inputName in @(
                    "validation_contract", "implementation_readiness", "fixture_identity",
                    "acceptance_inventory", "test_change_log"
                )) {
                $reference = $document.inputs.$inputName
                if ($null -ne $reference) {
                    $referenceEntries.Add([pscustomobject]@{
                        label = "inputs.$inputName"
                        reference = $reference
                        allow_source_commit = $inputName -cin @(
                            "validation_contract", "acceptance_inventory", "test_change_log"
                        )
                    })
                }
            }
            foreach ($finding in @($document.findings)) {
                foreach ($reference in @($finding.evidence)) {
                    $referenceEntries.Add([pscustomobject]@{
                        label = "findings.$([string]$finding.id).evidence"
                        reference = $reference
                        allow_source_commit = $false
                    })
                }
            }
            foreach ($check in @($document.routing.focused_reproducers)) {
                if ($null -ne $check.evidence) {
                    $referenceEntries.Add([pscustomobject]@{
                        label = "routing.focused_reproducers.$([string]$check.id)"
                        reference = $check.evidence
                        allow_source_commit = $false
                    })
                }
            }
            foreach ($change in @($document.expectation_changes)) {
                $referenceEntries.Add([pscustomobject]@{
                    label = "expectation_changes.$([string]$change.test_path)"
                    reference = $change.test_change_log
                    allow_source_commit = $true
                })
            }
            if ($document.PSObject.Properties.Name -contains "correction") {
                foreach ($check in @($document.correction.focused_results)) {
                    if ($null -ne $check.evidence) {
                        $referenceEntries.Add([pscustomobject]@{
                            label = "correction.focused_results.$([string]$check.id)"
                            reference = $check.evidence
                            allow_source_commit = $false
                        })
                    }
                }
                foreach ($check in @($document.correction.implementation_target_results)) {
                    if ($null -ne $check.evidence) {
                        $referenceEntries.Add([pscustomobject]@{
                            label = "correction.implementation_target_results.$([string]$check.id)"
                            reference = $check.evidence
                            allow_source_commit = $false
                        })
                    }
                }
            }

            $referenceHashes = [Collections.Generic.Dictionary[string, string]]::new(
                $pathComparer
            )
            $referenceClaims = [Collections.Generic.HashSet[string]]::new(
                $pathComparer
            )
            $sourceCommits = [Collections.Generic.List[string]]::new()
            $sourceIdentityAuthentication = Get-TessaraGitSourceIdentityAuthentication `
                -RepositoryRoot $repositoryPath -Identity $document.source_identity
            if ([string]$sourceIdentityAuthentication.state -ceq "authenticated") {
                $sourceCommits.Add([string]$document.source_identity.commit)
            } else {
                $localIssues.Add("source_identity does not authenticate a Git commit and exact tree")
            }
            if ($document.PSObject.Properties.Name -contains "correction") {
                $cleanSourceAuthentication = Get-TessaraGitSourceIdentityAuthentication `
                    -RepositoryRoot $repositoryPath -Identity $document.correction.clean_source
                if ([string]$cleanSourceAuthentication.state -ceq "authenticated") {
                    $sourceCommits.Add([string]$document.correction.clean_source.commit)
                } else {
                    $localIssues.Add("correction.clean_source does not authenticate a Git commit and exact tree")
                }
            }
            foreach ($entry in $referenceEntries) {
                $referencePath = ConvertTo-TessaraRepositoryPath -Path ([string]$entry.reference.path)
                $referenceSha = [string]$entry.reference.sha256
                $claimKey = "{0}`0{1}`0{2}" -f @(
                    [string]$entry.label,
                    $referencePath,
                    $referenceSha
                )
                if (-not $referenceClaims.Add($claimKey)) {
                    $localIssues.Add("$([string]$entry.label) contains duplicate evidence '$referencePath'")
                }
                if ($referenceHashes.ContainsKey($referencePath) -and
                    [string]$referenceHashes[$referencePath] -cne $referenceSha) {
                    $localIssues.Add("conflicting hashes are claimed for evidence '$referencePath'")
                    continue
                }
                if ($referenceHashes.ContainsKey($referencePath)) { continue }
                $referenceHashes.Add($referencePath, $referenceSha)
                $referenceCount++
                $authenticationParameters = @{
                    RepositoryRoot = $repositoryPath
                    Reference = $entry.reference
                    SourceCommits = @($sourceCommits)
                }
                if ([bool]$entry.allow_source_commit) {
                    $authenticationParameters.AllowSourceCommit = $true
                }
                $authentication = Get-TessaraEvidenceReferenceAuthentication `
                    @authenticationParameters
                if ([string]$authentication.state -ceq "authenticated") {
                    $authenticatedReferenceCount++
                    if ([string]$authentication.basis -ceq "retained-file") {
                        $retainedReferenceSnapshots.Add([pscustomobject][ordered]@{
                            path = $referencePath
                            full_path = [string]$authentication.full_path
                            sha256 = $referenceSha
                        })
                    }
                } else {
                    $localIssues.Add("evidence '$referencePath' is missing or does not match its claimed hash")
                }
            }

            foreach ($collection in @(
                    [pscustomobject]@{ label = "findings"; items = @($document.findings) },
                    [pscustomobject]@{ label = "routing.focused_reproducers"; items = @($document.routing.focused_reproducers) }
                )) {
                $ids = @($collection.items | ForEach-Object { [string]$_.id })
                if (@($ids | Sort-Object -Unique).Count -ne $ids.Count) {
                    $localIssues.Add("$([string]$collection.label) contains duplicate IDs")
                }
            }
            if ($document.PSObject.Properties.Name -contains "correction") {
                foreach ($collection in @(
                        [pscustomobject]@{ label = "correction.focused_results"; items = @($document.correction.focused_results) },
                        [pscustomobject]@{ label = "correction.implementation_target_results"; items = @($document.correction.implementation_target_results) }
                    )) {
                    $ids = @($collection.items | ForEach-Object { [string]$_.id })
                    if (@($ids | Sort-Object -Unique).Count -ne $ids.Count) {
                        $localIssues.Add("$([string]$collection.label) contains duplicate IDs")
                    }
                }
                $correctionCheckIds = @(
                    @($document.correction.focused_results) +
                    @($document.correction.implementation_target_results) |
                        ForEach-Object { [string]$_.id }
                )
                if (@($correctionCheckIds | Sort-Object -Unique).Count -ne
                    $correctionCheckIds.Count) {
                    $localIssues.Add("correction check IDs are not globally unique")
                }
            }

            if ($status -ceq "verified") {
                if (@($document.findings | Where-Object {
                        [string]$_.status -cne "verified"
                    }).Count -ne 0) {
                    $localIssues.Add("verified record contains a finding that is not verified")
                }
                if ([bool]$document.correction.clean_source.dirty) {
                    $localIssues.Add("verified record correction source is dirty")
                }
                if ([bool]$document.routing.full_rerun_blocked) {
                    $localIssues.Add("verified record still blocks its full rerun")
                }
                $authorizedNextBoundary = if (
                    $document.correction.PSObject.Properties.Name -contains
                        "authorized_next_boundary"
                ) {
                    [string]$document.correction.authorized_next_boundary
                } else { "" }
                if ([string]::IsNullOrWhiteSpace($authorizedNextBoundary)) {
                    $localIssues.Add("verified record has no coordinator-authorized next boundary")
                } elseif ([string]$document.routing.next_boundary -cne $authorizedNextBoundary) {
                    $localIssues.Add("verified record routing does not match its authorized next boundary")
                }
                $routingChecks = @($document.routing.focused_reproducers)
                $focusedChecks = @($document.correction.focused_results)
                $targetChecks = @($document.correction.implementation_target_results)
                if ($routingChecks.Count -eq 0 -or $focusedChecks.Count -eq 0 -or
                    $targetChecks.Count -eq 0) {
                    $localIssues.Add("verified record has no complete focused and implementation proof inventory")
                }
                foreach ($collection in @($routingChecks, $focusedChecks, $targetChecks)) {
                    if (@($collection | Where-Object {
                            [string]$_.state -cne "passed" -or $null -eq $_.evidence
                        }).Count -ne 0) {
                        $localIssues.Add("verified record contains a non-passing or evidence-free required check")
                        break
                    }
                }
                $routingIds = @($routingChecks | ForEach-Object { [string]$_.id } | Sort-Object)
                $focusedIds = @($focusedChecks | ForEach-Object { [string]$_.id } | Sort-Object)
                if (($routingIds -join "`n") -cne ($focusedIds -join "`n")) {
                    $localIssues.Add("verified record focused correction results are not set-equal to routed reproducers")
                }
                if (@($document.correction.changed_paths).Count -eq 0 -or
                    @($document.correction.changed_domains).Count -eq 0) {
                    $localIssues.Add("verified record does not identify changed paths and dependency domains")
                }
            }
        }
        $records.Add([pscustomobject][ordered]@{
            path = $relativePath
            full_path = $fullPath
            sha256 = $sha256
            sidecar_state = $sidecarState
            document = $document
            parse_error = $parseError
            schema_valid = [bool]$schemaValid
            schema_errors = @($schemaErrors)
            sprint = $recordSprint
            record_id = $recordId
            status = $status
            created_at = $createdAt
            updated_at = $updatedAt
            local_issues = $localIssues
            reference_count = $referenceCount
            authenticated_reference_count = $authenticatedReferenceCount
            retained_reference_snapshots = $retainedReferenceSnapshots
            sidecar_path = $sidecarPath
            sidecar_snapshot_sha256 = $sidecarSnapshotSha
            superseded_by = $null
            resolution = "unresolved"
        })
    }

    $recordPathMap = [Collections.Generic.Dictionary[string, object]]::new(
        $pathComparer
    )
    $recordIdMap = [Collections.Generic.Dictionary[string, object]]::new(
        [StringComparer]::Ordinal
    )
    foreach ($record in @($records)) {
        if ($recordPathMap.ContainsKey([string]$record.path)) {
            $structuralIssues.Add("Defect-provenance chronology duplicates path '$($record.path)'.")
        } else {
            $recordPathMap.Add([string]$record.path, $record)
        }
        if (-not [string]::IsNullOrWhiteSpace([string]$record.record_id)) {
            if ($recordIdMap.ContainsKey([string]$record.record_id)) {
                $first = $recordIdMap[[string]$record.record_id]
                $first.local_issues.Add("record_id is duplicated by another retained record")
                $record.local_issues.Add("record_id is duplicated by another retained record")
                $structuralIssues.Add("Defect-provenance chronology duplicates record_id '$($record.record_id)'.")
            } else {
                $recordIdMap.Add([string]$record.record_id, $record)
            }
        }
    }

    $supersedersByTarget = @{}
    foreach ($record in @($records | Where-Object { [bool]$_.schema_valid })) {
        $supersedes = if ($record.document.PSObject.Properties.Name -contains "supersedes") {
            @($record.document.supersedes)
        } else { @() }
        $seenReferences = [Collections.Generic.HashSet[string]]::new($pathComparer)
        foreach ($reference in $supersedes) {
            $targetPath = ConvertTo-TessaraRepositoryPath -Path ([string]$reference.path)
            if (-not $seenReferences.Add($targetPath)) {
                $record.local_issues.Add("duplicates supersession '$targetPath'")
                continue
            }
            if ([string]$record.path -ceq $targetPath) {
                $record.local_issues.Add("cannot supersede itself")
                continue
            }
            if ([IO.Path]::GetFileName($targetPath) -cne "defect-provenance.json" -or
                -not $recordPathMap.ContainsKey($targetPath)) {
                $record.local_issues.Add("has a dangling supersession '$targetPath'")
                continue
            }
            $target = $recordPathMap[$targetPath]
            if ([string]$reference.sha256 -cne [string]$target.sha256) {
                $record.local_issues.Add("has the wrong hash for '$targetPath'")
                continue
            }
            $recordDirectVerified = [bool]$record.schema_valid -and
                [string]$record.status -ceq "verified" -and
                @($record.local_issues).Count -eq 0
            if (-not $recordDirectVerified) {
                $record.local_issues.Add("is not completely verified and cannot supersede '$targetPath'")
                continue
            }
            $targetDirectVerified = [bool]$target.schema_valid -and
                [string]$target.status -ceq "verified" -and
                @($target.local_issues).Count -eq 0
            if ($targetDirectVerified) {
                $record.local_issues.Add("cannot supersede already verified record '$targetPath'")
                continue
            }
            if ($null -eq $record.created_at -or $null -eq $target.updated_at -or
                $record.created_at -le $target.updated_at) {
                $record.local_issues.Add("is not later than the final update of '$targetPath'")
                continue
            }
            if (-not $supersedersByTarget.ContainsKey($targetPath)) {
                $supersedersByTarget[$targetPath] = [Collections.Generic.List[object]]::new()
            }
            $supersedersByTarget[$targetPath].Add($record)
        }
    }

    foreach ($targetPath in @($supersedersByTarget.Keys)) {
        $superseders = @($supersedersByTarget[$targetPath])
        if ($superseders.Count -gt 1) {
            $recordPathMap[$targetPath].local_issues.Add("has multiple verified superseders")
            foreach ($superseder in $superseders) {
                $superseder.local_issues.Add("participates in ambiguous multiple supersession of '$targetPath'")
            }
        }
    }

    $finalEntries = @(Get-ChildItem -LiteralPath $evidencePath -Recurse -Force)
    foreach ($entry in @($finalEntries | Where-Object {
            ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            ($_.PSObject.Properties.Name -contains "LinkType" -and
                -not [string]::IsNullOrWhiteSpace([string]$_.LinkType))
        })) {
        throw "Defect-provenance evidence root contains a symbolic link or reparse point: $($entry.FullName)"
    }
    $finalFiles = @($finalEntries | Where-Object {
        -not $_.PSIsContainer -and $_.Name -ceq "defect-provenance.json"
    } | ForEach-Object { [IO.Path]::GetFullPath($_.FullName) } | Sort-Object)
    $initialFilePaths = @($initialFiles | ForEach-Object {
        [IO.Path]::GetFullPath($_.FullName)
    } | Sort-Object)
    $initialFileSet = [Collections.Generic.HashSet[string]]::new($pathComparer)
    $finalFileSet = [Collections.Generic.HashSet[string]]::new($pathComparer)
    foreach ($path in $initialFilePaths) { $null = $initialFileSet.Add($path) }
    foreach ($path in $finalFiles) { $null = $finalFileSet.Add($path) }
    $inventoryAdditions = @($finalFiles | Where-Object { -not $initialFileSet.Contains($_) })
    $inventoryRemovals = @($initialFilePaths | Where-Object { -not $finalFileSet.Contains($_) })
    if ($inventoryAdditions.Count -ne 0 -or $inventoryRemovals.Count -ne 0) {
        $structuralIssues.Add("Defect-provenance record inventory changed during chronology audit.")
    }
    foreach ($record in @($records)) {
        if (-not (Test-Path -LiteralPath $record.full_path -PathType Leaf)) {
            $record.local_issues.Add("record disappeared during chronology audit")
            continue
        }
        $finalSnapshot = Read-TessaraValidationFileSnapshot -Path $record.full_path
        if ([string]$finalSnapshot.sha256 -cne [string]$record.sha256) {
            $record.local_issues.Add("record bytes changed during chronology audit")
        }
        $sidecarExists = Test-Path -LiteralPath $record.sidecar_path -PathType Leaf
        if ($null -eq $record.sidecar_snapshot_sha256) {
            if ($sidecarExists) {
                $record.local_issues.Add("record sidecar appeared during chronology audit")
            }
        } elseif (-not $sidecarExists) {
            $record.local_issues.Add("record sidecar disappeared during chronology audit")
        } else {
            $finalSidecarSnapshot = Read-TessaraValidationFileSnapshot -Path $record.sidecar_path
            if ([string]$finalSidecarSnapshot.sha256 -cne
                [string]$record.sidecar_snapshot_sha256) {
                $record.local_issues.Add("record sidecar changed during chronology audit")
            }
        }
        foreach ($referenceSnapshot in @($record.retained_reference_snapshots)) {
            if (-not (Test-Path -LiteralPath $referenceSnapshot.full_path -PathType Leaf)) {
                $record.local_issues.Add(
                    "retained evidence '$([string]$referenceSnapshot.path)' disappeared during chronology audit"
                )
                continue
            }
            Assert-TessaraFileSystemPathHasNoLinks -Root $repositoryPath `
                -Path $referenceSnapshot.full_path `
                -Label "Defect-provenance evidence reference '$([string]$referenceSnapshot.path)'"
            $finalReferenceSnapshot = Read-TessaraValidationFileSnapshot `
                -Path $referenceSnapshot.full_path
            if ([string]$finalReferenceSnapshot.sha256 -cne
                [string]$referenceSnapshot.sha256) {
                $record.local_issues.Add(
                    "retained evidence '$([string]$referenceSnapshot.path)' changed during chronology audit"
                )
            }
        }
    }

    $orderedRecords = @($records | Sort-Object `
        @{ Expression = { if ($null -eq $_.created_at) { [long]::MinValue } else { $_.created_at.UtcTicks } } }, `
        @{ Expression = { if ($null -eq $_.updated_at) { [long]::MinValue } else { $_.updated_at.UtcTicks } } }, `
        @{ Expression = { [string]$_.path } })
    foreach ($record in $orderedRecords) {
        $superseders = @(if ($supersedersByTarget.ContainsKey([string]$record.path)) {
            @($supersedersByTarget[[string]$record.path] | Where-Object {
                [bool]$_.schema_valid -and [string]$_.status -ceq "verified" -and
                    @($_.local_issues).Count -eq 0
            })
        })
        if ($superseders.Count -gt 1) {
            continue
        }
        if ([bool]$record.schema_valid -and [string]$record.status -ceq "verified" -and
            @($record.local_issues).Count -eq 0) {
            $record.resolution = "verified"
            continue
        }
        if ($superseders.Count -eq 1) {
            $record.superseded_by = [pscustomobject][ordered]@{
                path = [string]$superseders[0].path
                sha256 = [string]$superseders[0].sha256
            }
            $record.resolution = "validly_superseded"
            continue
        }
    }

    $issues = [Collections.Generic.List[string]]::new()
    foreach ($issue in $structuralIssues) { $issues.Add($issue) }
    foreach ($record in $orderedRecords | Where-Object {
            [string]$_.resolution -ceq "unresolved"
        }) {
        $reason = if (-not [bool]$record.schema_valid) {
            "schema-invalid"
        } elseif (@($record.local_issues).Count -gt 0) {
            @($record.local_issues | Sort-Object -Unique) -join "; "
        } else {
            "status-$([string]$record.status)"
        }
        $issues.Add("Defect-provenance record '$($record.path)' is unresolved ($reason).")
    }

    $publicRecords = @($orderedRecords | ForEach-Object {
        [pscustomobject][ordered]@{
            path = [string]$_.path
            sha256 = [string]$_.sha256
            record_id = [string]$_.record_id
            claimed_status = [string]$_.status
            schema_state = if ([bool]$_.schema_valid) { "valid" } else { "invalid" }
            resolution = [string]$_.resolution
            superseded_by = $_.superseded_by
            sidecar_state = [string]$_.sidecar_state
            created_at = if ($null -eq $_.created_at) { $null } else {
                $_.created_at.ToUniversalTime().ToString("O")
            }
            updated_at = if ($null -eq $_.updated_at) { $null } else {
                $_.updated_at.ToUniversalTime().ToString("O")
            }
            evidence_reference_count = [int]$_.reference_count
            authenticated_evidence_reference_count = [int]$_.authenticated_reference_count
            validation_issues = @($_.local_issues | Sort-Object -Unique)
        }
    })
    $inventoryAdditionRecords = @($inventoryAdditions | ForEach-Object {
        [pscustomobject][ordered]@{
            path = ConvertTo-TessaraRepositoryPath -Path (
                [IO.Path]::GetRelativePath($repositoryPath, $_)
            )
            sha256 = $null
            record_id = ""
            claimed_status = ""
            schema_state = "not_audited"
            resolution = "unresolved"
            superseded_by = $null
            sidecar_state = "not_audited"
            created_at = $null
            updated_at = $null
            evidence_reference_count = 0
            authenticated_evidence_reference_count = 0
            validation_issues = @("record appeared during chronology audit and was not audited")
        }
    })
    $publicRecords = @($publicRecords) + @($inventoryAdditionRecords)
    $uniqueIssues = @($issues | Sort-Object -Unique)
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.defect-provenance-chronology"
        policy_version = $script:PolicyVersion
        sprint = $Sprint
        state = if ($uniqueIssues.Count -eq 0) { "passed" } else { "failed" }
        record_count = $publicRecords.Count
        schema_valid_count = @($records | Where-Object { [bool]$_.schema_valid }).Count
        schema_invalid_count = @($records | Where-Object { -not [bool]$_.schema_valid }).Count +
            $inventoryAdditionRecords.Count
        verified_count = @($records | Where-Object { [string]$_.resolution -ceq "verified" }).Count
        validly_superseded_count = @($records | Where-Object {
            [string]$_.resolution -ceq "validly_superseded"
        }).Count
        unresolved_count = @($records | Where-Object { [string]$_.resolution -ceq "unresolved" }).Count +
            $inventoryAdditionRecords.Count
        inventory_added_count = $inventoryAdditions.Count
        inventory_removed_count = $inventoryRemovals.Count
        records = $publicRecords
        issues = $uniqueIssues
    }
}

function Assert-TessaraDefectProvenanceChronology {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Sprint
    )

    $audit = Get-TessaraDefectProvenanceChronology -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot -Sprint $Sprint
    if ([string]$audit.state -cne "passed") {
        $message = @($audit.issues) -join "; "
        throw "Defect-provenance chronology is unresolved. $message"
    }
    return $audit
}

function Get-TessaraNamedItems {
    param(
        [Parameter(Mandatory)][object[]]$Items,
        [Parameter(Mandatory)][string]$Property,
        [Parameter(Mandatory)][string]$Label
    )

    $map = @{}
    foreach ($item in @($Items)) {
        $name = [string]$item.$Property
        if ([string]::IsNullOrWhiteSpace($name)) {
            throw "$Label contains an item without '$Property'."
        }
        if ($map.ContainsKey($name)) {
            throw "$Label contains duplicate '$name'."
        }
        $map[$name] = $item
    }
    return $map
}

function Assert-TessaraValidationContract {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Contract)

    Assert-TessaraJsonSchema -Document $Contract -Kind validation_contract -Label "Validation contract"

    $domains = Get-TessaraNamedItems -Items @($Contract.dependency_domains) -Property name -Label "Dependency domains"
    $targets = Get-TessaraNamedItems -Items @($Contract.implementation_targets) -Property id -Label "Implementation targets"
    $lanes = Get-TessaraNamedItems -Items @($Contract.lanes) -Property id -Label "Validation lanes"
    $null = Get-TessaraNamedItems -Items @($Contract.requirements) -Property id -Label "Requirements"

    $profileKind = [string]$Contract.implementation_profile.kind
    $requiredExtractionProofClasses = @(
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
    $cleanEnvironmentProofClasses = @(
        "clean-materialization",
        "semantic-noop",
        "failure-recovery"
    )
    $proofTargets = @{}

    foreach ($domain in @($Contract.dependency_domains)) {
        foreach ($pattern in @($domain.tracked_inputs)) {
            $null = ConvertTo-TessaraRepositoryPath -Path ([string]$pattern)
        }
    }

    foreach ($target in @($Contract.implementation_targets)) {
        foreach ($domainName in @($target.dependency_domains)) {
            if (-not $domains.ContainsKey([string]$domainName)) {
                throw "Implementation target '$($target.id)' references unknown domain '$domainName'."
            }
        }
        foreach ($proofClass in @($target.proof_classes)) {
            $proofClassName = [string]$proofClass
            if (-not $proofTargets.ContainsKey($proofClassName)) {
                $proofTargets[$proofClassName] = [Collections.Generic.List[object]]::new()
            }
            $proofTargets[$proofClassName].Add($target)
        }
    }

    if ($profileKind -ceq "phase8-module-extraction") {
        foreach ($proofClass in $requiredExtractionProofClasses) {
            if (-not $proofTargets.ContainsKey($proofClass)) {
                throw "Phase 8 module extraction is missing implementation proof class '$proofClass'."
            }
            $requiredTargets = @($proofTargets[$proofClass] | Where-Object { $_.required -eq $true })
            if ($requiredTargets.Count -eq 0) {
                throw "Phase 8 module extraction proof class '$proofClass' is not bound to a required target."
            }
            if ($proofClass -in $cleanEnvironmentProofClasses -and
                @($requiredTargets | Where-Object { $_.clean_environment -eq $true }).Count -eq 0) {
                throw "Phase 8 module extraction proof class '$proofClass' is not bound to a clean-environment target."
            }
        }
    }

    foreach ($lane in @($Contract.lanes)) {
        foreach ($domainName in @($lane.dependency_domains)) {
            if (-not $domains.ContainsKey([string]$domainName)) {
                throw "Validation lane '$($lane.id)' references unknown domain '$domainName'."
            }
        }
        foreach ($prerequisite in @($lane.prerequisites)) {
            if (-not $lanes.ContainsKey([string]$prerequisite)) {
                throw "Validation lane '$($lane.id)' references unknown prerequisite '$prerequisite'."
            }
            if ([string]$prerequisite -ceq [string]$lane.id) {
                throw "Validation lane '$($lane.id)' cannot depend on itself."
            }
        }
    }

    foreach ($phase in @("validation-readiness", "candidate-rehearsal", "validation-preflight", "sit", "uat")) {
        if (@($Contract.lanes | Where-Object { [string]$_.phase -ceq $phase }).Count -eq 0) {
            throw "Validation contract does not declare a '$phase' lane."
        }
    }

    function Visit-Lane {
        param([string]$LaneId, [hashtable]$Visiting, [hashtable]$Visited)
        if ($Visiting.ContainsKey($LaneId)) {
            throw "Validation lane prerequisites contain a cycle at '$LaneId'."
        }
        if ($Visited.ContainsKey($LaneId)) { return }
        $Visiting[$LaneId] = $true
        foreach ($prerequisite in @($lanes[$LaneId].prerequisites)) {
            Visit-Lane -LaneId ([string]$prerequisite) -Visiting $Visiting -Visited $Visited
        }
        $Visiting.Remove($LaneId)
        $Visited[$LaneId] = $true
    }

    $visitedLanes = @{}
    foreach ($laneId in @($lanes.Keys)) {
        Visit-Lane -LaneId ([string]$laneId) -Visiting @{} -Visited $visitedLanes
    }

    foreach ($requirement in @($Contract.requirements)) {
        foreach ($targetId in @($requirement.implementation_targets)) {
            if (-not $targets.ContainsKey([string]$targetId)) {
                throw "Requirement '$($requirement.id)' references unknown implementation target '$targetId'."
            }
        }
        foreach ($laneId in @($requirement.validation_lanes)) {
            if (-not $lanes.ContainsKey([string]$laneId)) {
                throw "Requirement '$($requirement.id)' references unknown validation lane '$laneId'."
            }
        }
    }

    return $true
}

function Get-TessaraValidationImpact {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ChangedPaths,
        [switch]$CandidateChanged
    )

    $null = Assert-TessaraValidationContract -Contract $Contract
    $changed = [Collections.Generic.List[object]]::new()
    $unknown = [Collections.Generic.List[string]]::new()
    $changedDomains = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)

    foreach ($rawPath in @($ChangedPaths)) {
        $path = ConvertTo-TessaraRepositoryPath -Path $rawPath
        $matchedDomains = [Collections.Generic.List[string]]::new()
        foreach ($domain in @($Contract.dependency_domains)) {
            foreach ($rawPattern in @($domain.tracked_inputs)) {
                $pattern = ConvertTo-TessaraRepositoryPath -Path ([string]$rawPattern)
                if ($path -like $pattern) {
                    $name = [string]$domain.name
                    if ($name -notin $matchedDomains) {
                        $matchedDomains.Add($name)
                        $null = $changedDomains.Add($name)
                    }
                    break
                }
            }
        }
        if ($matchedDomains.Count -eq 0) {
            $unknown.Add($path)
        }
        $changed.Add([pscustomobject]@{
            path = $path
            domains = @($matchedDomains | Sort-Object)
        })
    }

    $decisions = [Collections.Generic.List[object]]::new()
    foreach ($phase in @("validation-readiness", "candidate-rehearsal", "validation-preflight", "sit", "uat")) {
        $phaseLanes = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $phase })
        if ($phaseLanes.Count -eq 0) {
            continue
        }

        $directlyAffected = @($phaseLanes | Where-Object {
            @($_.dependency_domains | Where-Object { $changedDomains.Contains([string]$_) }).Count -gt 0
        } | ForEach-Object { [string]$_.id } | Sort-Object -Unique)
        $affectedSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($laneId in $directlyAffected) { $null = $affectedSet.Add($laneId) }
        $phaseLaneMap = @{}
        foreach ($lane in $phaseLanes) { $phaseLaneMap[[string]$lane.id] = $lane }
        $pending = [Collections.Generic.Queue[string]]::new()
        foreach ($laneId in $directlyAffected) { $pending.Enqueue($laneId) }
        while ($pending.Count -gt 0) {
            $laneId = $pending.Dequeue()
            foreach ($prerequisite in @($phaseLaneMap[$laneId].prerequisites)) {
                $name = [string]$prerequisite
                if ($phaseLaneMap.ContainsKey($name) -and $affectedSet.Add($name)) {
                    $pending.Enqueue($name)
                }
            }
        }
        $affected = @($affectedSet | Sort-Object)

        $action = "reuse_certificate"
        $rationale = "No declared dependency for this phase changed."
        if ($unknown.Count -gt 0) {
            $action = "rerun_full_phase"
            $affected = @($phaseLanes | ForEach-Object { [string]$_.id } | Sort-Object)
            $rationale = "At least one changed path could not be authenticated against the validation contract."
        } elseif ($phase -in @("sit", "uat") -and $CandidateChanged) {
            $action = "rerun_full_phase"
            $affected = @($phaseLanes | ForEach-Object { [string]$_.id } | Sort-Object)
            $rationale = "A successor candidate requires complete candidate-bound $phase."
        } elseif ($affected.Count -gt 0 -and
            @($changedDomains).Count -eq 1 -and $changedDomains.Contains("evidence-publication")) {
            $action = "finalization_only"
            $rationale = "Only evidence publication changed and immutable assertion results remain outside the impact cone."
        } elseif ($affected.Count -gt 0 -and $phase -in @("validation-readiness", "candidate-rehearsal")) {
            $action = "recertify_affected_lanes"
            $rationale = "Only lanes intersecting changed dependency domains require pre-freeze recertification."
        } elseif ($affected.Count -gt 0) {
            $action = "rerun_full_phase"
            $rationale = "The phase consumes a changed dependency and is not eligible for pre-freeze lane inheritance."
        }

        $decisions.Add([pscustomobject]@{
            phase = $phase
            action = $action
            affected_lanes = $affected
            rationale = $rationale
        })
    }

    return [pscustomobject]@{
        changed_paths = @($changed)
        changed_domains = @($changedDomains | Sort-Object)
        unknown_paths = @($unknown | Sort-Object)
        phase_decisions = @($decisions)
        candidate_changed = [bool]$CandidateChanged
        require_complete_sit = [bool]$CandidateChanged
        require_complete_uat = [bool]$CandidateChanged
    }
}

function Assert-TessaraImplementationReadinessResult {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$ContractPath
    )

    $null = Assert-TessaraValidationContract -Contract $Contract
    Assert-TessaraJsonSchema -Document $Result -Kind implementation_readiness -Label "Implementation readiness result"

    if ([string]$Result.sprint -cne [string]$Contract.sprint) {
        throw "Implementation readiness sprint does not match the validation contract."
    }
    if ([bool]$Result.source_identity.dirty) {
        throw "A dirty source cannot pass implementation readiness."
    }

    $contractSha = Get-TessaraValidationSha256 -Path $ContractPath
    if ([string]$Result.validation_contract.sha256 -cne $contractSha) {
        throw "Implementation readiness does not bind the current validation contract hash."
    }

    $resultTargets = Get-TessaraNamedItems -Items @($Result.targets) -Property id -Label "Implementation readiness targets"
    $affected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($domain in @($Result.affected_domains)) { $null = $affected.Add([string]$domain) }

    foreach ($target in @($Contract.implementation_targets)) {
        $selected = [bool]$target.required -or
            @($target.dependency_domains | Where-Object { $affected.Contains([string]$_) }).Count -gt 0
        if (-not $selected) { continue }
        if (-not $resultTargets.ContainsKey([string]$target.id)) {
            throw "Implementation readiness omitted selected target '$($target.id)'."
        }
        $actual = $resultTargets[[string]$target.id]
        if ([string]$actual.state -cne "passed") {
            throw "Implementation target '$($target.id)' did not pass."
        }
        if ([string]$actual.command -cne [string]$target.command) {
            throw "Implementation target '$($target.id)' did not use the contract command."
        }
        if ([bool]$target.clean_environment -and -not [bool]$actual.clean_environment) {
            throw "Implementation target '$($target.id)' lacks its required clean-environment proof."
        }
    }

    if ([string]$Result.state -cne "passed" -or [int]$Result.known_failure_count -ne 0) {
        throw "Implementation readiness cannot pass with a failed state or known failures."
    }

    foreach ($proofName in @("first_apply", "semantic_no_op", "recovery")) {
        $proof = $Result.materialization.$proofName
        if ([bool]$proof.required -and [string]$proof.state -cne "passed") {
            throw "Required implementation proof '$proofName' did not pass."
        }
    }
    if ([bool]$Result.cleanup_restoration.required -and [string]$Result.cleanup_restoration.state -cne "passed") {
        throw "Required implementation cleanup/restoration did not pass."
    }
    if ([string]$Contract.implementation_profile.kind -ceq "phase8-module-extraction") {
        foreach ($proofName in @("first_apply", "semantic_no_op", "recovery")) {
            $proof = $Result.materialization.$proofName
            if (-not [bool]$proof.required -or [string]$proof.state -cne "passed") {
                throw "Phase 8 module extraction requires passing implementation proof '$proofName'."
            }
        }
        if (-not [bool]$Result.cleanup_restoration.required -or
            [string]$Result.cleanup_restoration.state -cne "passed") {
            throw "Phase 8 module extraction requires passing cleanup/restoration proof."
        }
    }
    return $true
}

function Assert-TessaraCorrectionImpactAssessment {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Assessment,
        [Parameter(Mandatory)]$Contract
    )

    $null = Assert-TessaraValidationContract -Contract $Contract
    Assert-TessaraJsonSchema -Document $Assessment -Kind correction_impact -Label "Correction impact assessment"
    if ([string]$Assessment.sprint -cne [string]$Contract.sprint) {
        throw "Correction impact sprint does not match the validation contract."
    }

    $decisions = Get-TessaraNamedItems -Items @($Assessment.phase_decisions) -Property phase -Label "Impact phase decisions"
    foreach ($phase in @("validation-readiness", "candidate-rehearsal", "validation-preflight", "sit", "uat")) {
        if (-not $decisions.ContainsKey($phase)) {
            throw "Correction impact assessment omitted phase '$phase'."
        }
        $decision = $decisions[$phase]
        if ([string]$decision.action -ceq "reuse_certificate" -and @($decision.affected_lanes).Count -ne 0) {
            throw "A reused '$phase' certificate cannot declare affected lanes."
        }
        if ([string]$decision.action -ceq "recertify_affected_lanes" -and @($decision.affected_lanes).Count -eq 0) {
            throw "Affected-lane recertification for '$phase' requires at least one lane."
        }
    }

    if (@($Assessment.unknown_paths).Count -gt 0 -and
        @($Assessment.phase_decisions | Where-Object { [string]$_.action -cne "rerun_full_phase" }).Count -gt 0) {
        throw "Unknown changed paths require the conservative full affected-phase fallback."
    }
    if ([bool]$Assessment.candidate_changed) {
        if (-not [bool]$Assessment.require_complete_sit -or -not [bool]$Assessment.require_complete_uat -or
            [string]$decisions["sit"].action -cne "rerun_full_phase" -or
            [string]$decisions["uat"].action -cne "rerun_full_phase") {
            throw "A successor candidate requires complete SIT and complete UAT."
        }
    }
    return $true
}

function Get-TessaraFingerprintMap {
    param([Parameter(Mandatory)][object[]]$Fingerprints, [string]$Label = "Fingerprints")
    return Get-TessaraNamedItems -Items $Fingerprints -Property domain -Label $Label
}

function Assert-TessaraPhaseCertificate {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Certificate)

    Assert-TessaraJsonSchema -Document $Certificate -Kind phase_certificate -Label "Phase certificate"
    $phase = [string]$Certificate.phase
    $preFreeze = $phase -in @("validation-readiness", "candidate-rehearsal")
    if ($preFreeze -and [bool]$Certificate.authoritative) {
        throw "$phase must remain non-authoritative."
    }
    if (-not $preFreeze -and -not [bool]$Certificate.authoritative) {
        throw "$phase must be authoritative."
    }
    if ($preFreeze -and $null -ne $Certificate.candidate_fingerprint) {
        throw "$phase cannot claim a frozen candidate fingerprint."
    }
    if (-not $preFreeze -and $null -eq $Certificate.candidate_fingerprint) {
        throw "$phase requires a candidate fingerprint."
    }

    $currentFingerprints = Get-TessaraFingerprintMap -Fingerprints @($Certificate.dependency_fingerprints)
    $laneNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $executedCount = 0
    $inheritedCount = 0
    foreach ($lane in @($Certificate.lanes)) {
        if (-not $laneNames.Add([string]$lane.name)) {
            throw "Phase certificate contains duplicate lane '$($lane.name)'."
        }
        if ([string]$lane.certification_basis -ceq "executed") {
            $executedCount++
            if ($null -eq $lane.started_at -or $null -eq $lane.ended_at -or $null -eq $lane.duration_ms) {
                throw "Executed lane '$($lane.name)' requires execution timestamps and duration."
            }
            if ($null -ne $lane.inheritance) {
                throw "Executed lane '$($lane.name)' cannot carry inheritance evidence."
            }
        } else {
            $inheritedCount++
            if (-not $preFreeze) {
                throw "Candidate-bound phase '$phase' cannot inherit a lane from another candidate."
            }
            if ($null -ne $lane.started_at -or $null -ne $lane.ended_at -or $null -ne $lane.duration_ms) {
                throw "Inherited lane '$($lane.name)' must not fabricate current execution timing."
            }
            if ($null -eq $lane.inheritance) {
                throw "Inherited lane '$($lane.name)' is missing its non-impact evidence."
            }
            $priorFingerprints = Get-TessaraFingerprintMap -Fingerprints @($lane.inheritance.prior_dependency_fingerprints) -Label "Prior lane fingerprints"
            foreach ($domain in @($lane.dependency_domains)) {
                $name = [string]$domain
                if (-not $currentFingerprints.ContainsKey($name) -or -not $priorFingerprints.ContainsKey($name) -or
                    [string]$currentFingerprints[$name].sha256 -cne [string]$priorFingerprints[$name].sha256) {
                    throw "Inherited lane '$($lane.name)' has a changed or missing '$name' dependency fingerprint."
                }
            }
        }
    }

    if ([int]$Certificate.coverage.lane_count -ne @($Certificate.lanes).Count -or
        [int]$Certificate.coverage.executed_count -ne $executedCount -or
        [int]$Certificate.coverage.inherited_count -ne $inheritedCount -or
        ($executedCount + $inheritedCount) -ne [int]$Certificate.coverage.lane_count) {
        throw "Phase certificate coverage counts do not match its lane inventory."
    }
    if ([string]$Certificate.state -ceq "passed") {
        if (@($Certificate.lanes | Where-Object { [string]$_.state -cne "passed" }).Count -gt 0 -or
            [int]$Certificate.open_defect_count -ne 0 -or
            ([bool]$Certificate.cleanup_restoration.required -and [string]$Certificate.cleanup_restoration.state -cne "passed")) {
            throw "A passing phase certificate requires complete passing coverage, no open defects, and required restoration."
        }
    }
    return $true
}

function Assert-TessaraPhaseEvidenceIndex {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Index,
        [string]$RepositoryRoot,
        [switch]$AuditFiles
    )

    Assert-TessaraJsonSchema -Document $Index -Kind phase_evidence_index -Label "Phase evidence index"
    if ([int]$Index.entry_count -ne @($Index.entries).Count) {
        throw "Phase evidence-index entry count does not match its inventory."
    }
    $evidenceRoot = (ConvertTo-TessaraRepositoryPath -Path ([string]$Index.evidence_root)).TrimEnd("/")
    $paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($Index.entries)) {
        $path = ConvertTo-TessaraRepositoryPath -Path ([string]$entry.path)
        if (-not $path.StartsWith($evidenceRoot + "/", [StringComparison]::Ordinal)) {
            throw "Phase evidence '$path' is outside its declared evidence root '$evidenceRoot'."
        }
        if (-not $paths.Add($path)) {
            throw "Phase evidence index contains duplicate '$path'."
        }
        if ($AuditFiles) {
            if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
                throw "RepositoryRoot is required for a full evidence audit."
            }
            $fullRoot = [IO.Path]::GetFullPath($RepositoryRoot)
            $fullPath = [IO.Path]::GetFullPath((Join-Path $fullRoot $path))
            if (-not $fullPath.StartsWith($fullRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Evidence path '$path' escapes the repository root."
            }
            if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
                throw "Evidence artifact is missing: $path"
            }
            $item = Get-Item -LiteralPath $fullPath
            if ([long]$item.Length -ne [long]$entry.size -or
                (Get-TessaraValidationSha256 -Path $fullPath) -cne [string]$entry.sha256) {
                throw "Evidence artifact changed after phase sealing: $path"
            }
        }
    }
    return $true
}

function Assert-TessaraEvidenceChain {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Chain,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [switch]$FinalAudit
    )

    Assert-TessaraJsonSchema -Document $Chain -Kind evidence_chain -Label "Evidence chain"
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $indexReferences = [Collections.Generic.List[object]]::new()
    foreach ($reference in @($Chain.certificates) + @($Chain.corrections)) {
        $relative = ConvertTo-TessaraRepositoryPath -Path ([string]$reference.path)
        $full = [IO.Path]::GetFullPath((Join-Path $root $relative))
        if (-not $full.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
            -not (Test-Path -LiteralPath $full -PathType Leaf) -or
            (Get-TessaraValidationSha256 -Path $full) -cne [string]$reference.sha256) {
            throw "Evidence-chain reference failed authentication: $relative"
        }

        if ($reference.PSObject.Properties.Name -contains "phase") {
            $document = Get-Content -LiteralPath $full -Raw | ConvertFrom-Json
            if ([string]$reference.phase -ceq "implementation-readiness") {
                Assert-TessaraJsonSchema -Document $document -Kind implementation_readiness -Label "Implementation readiness certificate"
            } else {
                $null = Assert-TessaraPhaseCertificate -Certificate $document
            }
            $indexReferences.Add($document.evidence_index)
        }
    }

    $artifactCount = 0
    foreach ($indexReference in $indexReferences) {
        $relative = ConvertTo-TessaraRepositoryPath -Path ([string]$indexReference.path)
        $full = [IO.Path]::GetFullPath((Join-Path $root $relative))
        if (-not $full.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
            -not (Test-Path -LiteralPath $full -PathType Leaf) -or
            (Get-TessaraValidationSha256 -Path $full) -cne [string]$indexReference.sha256) {
            throw "Phase evidence-index reference failed authentication: $relative"
        }
        $index = Get-Content -LiteralPath $full -Raw | ConvertFrom-Json
        $null = Assert-TessaraPhaseEvidenceIndex -Index $index -RepositoryRoot $root -AuditFiles:$FinalAudit
        $artifactCount += [int]$index.entry_count
    }

    if ($FinalAudit) {
        if ([string]$Chain.final_integrity_audit.state -cne "passed" -or
            $null -eq $Chain.final_integrity_audit.audited_at -or
            [int]$Chain.final_integrity_audit.phase_index_count -ne $indexReferences.Count -or
            [int]$Chain.final_integrity_audit.artifact_count -ne $artifactCount) {
            throw "Closeout requires a passing final full integrity audit with exact phase-index and artifact counts."
        }
    }
    return $true
}

Export-ModuleMember -Function @(
    "Get-TessaraValidationPolicyVersion",
    "Get-TessaraValidationSchemaPath",
    "Get-TessaraValidationSha256",
    "Get-TessaraValidationChangedPaths",
    "Get-TessaraDependencyFingerprints",
    "Assert-TessaraJsonSchema",
    "Get-TessaraDefectProvenanceChronology",
    "Assert-TessaraDefectProvenanceChronology",
    "Assert-TessaraValidationContract",
    "Get-TessaraValidationImpact",
    "Assert-TessaraImplementationReadinessResult",
    "Assert-TessaraCorrectionImpactAssessment",
    "Assert-TessaraPhaseCertificate",
    "Assert-TessaraPhaseEvidenceIndex",
    "Assert-TessaraEvidenceChain"
)

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
            "evidence_chain"
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
            "evidence_chain"
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
        $normalized -match "(^|/)\.\.(/|$)") {
        throw "Validation evidence and contract paths must be repository-relative and traversal-free: '$Path'."
    }
    return $normalized
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
    "Assert-TessaraValidationContract",
    "Get-TessaraValidationImpact",
    "Assert-TessaraImplementationReadinessResult",
    "Assert-TessaraCorrectionImpactAssessment",
    "Assert-TessaraPhaseCertificate",
    "Assert-TessaraPhaseEvidenceIndex",
    "Assert-TessaraEvidenceChain"
)

Set-StrictMode -Version Latest

function Get-TessaraCoordinatorReference {
    param([string]$RepositoryRoot, $Reference, [string]$Label)
    $full = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $RepositoryRoot `
        -Path ([string]$Reference.path) -Label $Label
    if (-not (Test-Path -LiteralPath $full -PathType Leaf) -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $full).Hash.ToLowerInvariant() -cne
            [string]$Reference.sha256) {
        throw "$Label cannot be authenticated."
    }
    $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $RepositoryRoot `
        -Path $full -Label $Label
    $full
}

function ConvertTo-TessaraCoordinatorReference {
    param([string]$RepositoryRoot, [string]$Path)
    [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath(
            [IO.Path]::GetFullPath($RepositoryRoot), [IO.Path]::GetFullPath($Path)
        ).Replace('\', '/')
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
    }
}

function Publish-TessaraCoordinatorImmutableJson {
    param([string]$RepositoryRoot, [string]$Path, $Document, [string]$SchemaKind)
    if (-not [string]::IsNullOrWhiteSpace($SchemaKind)) {
        Assert-TessaraJsonSchema -Document $Document -Kind $SchemaKind `
            -Label "Coordinator artifact '$([IO.Path]::GetFileName($Path))'"
    }
    $full = [IO.Path]::GetFullPath($Path)
    $null = New-Item -ItemType Directory -Force -Path (Split-Path -Parent $full)
    $text = ($Document | ConvertTo-Json -Depth 100) + "`n"
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($text)
    try {
        $stream = [IO.File]::Open($full, [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write, [IO.FileShare]::None)
        try { $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) }
        finally { $stream.Dispose() }
    } catch [IO.IOException] {
        if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { throw }
        $existing = [IO.File]::ReadAllBytes($full)
        if (-not [Linq.Enumerable]::SequenceEqual([byte[]]$existing, [byte[]]$bytes)) {
            throw "Immutable coordinator artifact already exists with different bytes: $full"
        }
    }
    ConvertTo-TessaraCoordinatorReference -RepositoryRoot $RepositoryRoot -Path $full
}

function Set-TessaraCoordinatorCheckpoint {
    param([string]$RepositoryRoot, [string]$Path, $Document)
    Assert-TessaraJsonSchema -Document $Document -Kind implementation_coordinator_checkpoint `
        -Label "Implementation coordinator checkpoint"
    $full = [IO.Path]::GetFullPath($Path)
    $null = New-Item -ItemType Directory -Force -Path (Split-Path -Parent $full)
    $temporary = "$full.$([guid]::NewGuid().ToString('N')).tmp"
    try {
        [IO.File]::WriteAllText($temporary, (($Document | ConvertTo-Json -Depth 100) + "`n"),
            [Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $temporary -Destination $full -Force
    } finally {
        if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
    }
    ConvertTo-TessaraCoordinatorReference -RepositoryRoot $RepositoryRoot -Path $full
}

function Get-TessaraCoordinatorChangedPaths {
    param([string]$RepositoryRoot)
    $gitPath = Assert-TessaraPlatformGitExecutableCurrent
    $changed = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $RepositoryRoot `
        -Arguments @('-C', $RepositoryRoot, 'diff', '--name-only', '-z', 'HEAD', '--')
    $untracked = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $RepositoryRoot `
        -Arguments @('-C', $RepositoryRoot, 'ls-files', '-z', '--others', '--exclude-per-directory=.gitignore')
    if ([int]$changed.exit_code -ne 0 -or [int]$untracked.exit_code -ne 0) {
        throw "Implementation coordinator could not enumerate changed paths."
    }
    @(
        @(ConvertFrom-TessaraPlatformGitNulRecords -Text ([string]$changed.stdout) `
            -Label 'Coordinator changed-path inventory') +
        @(ConvertFrom-TessaraPlatformGitNulRecords -Text ([string]$untracked.stdout) `
            -Label 'Coordinator untracked-path inventory') |
        ForEach-Object { ([string]$_).Replace('\', '/') } | Sort-Object -Unique
    )
}

function Get-TessaraCoordinatorTargetMap {
    param($Contract)
    $map = @{}
    foreach ($target in @($Contract.implementation_targets)) {
        $map[[string]$target.id] = $target
    }
    $map
}

function Get-TessaraCoordinatorLaneMap {
    param($Validated)
    $map = @{}
    foreach ($target in @($Validated.validation_contract.implementation_targets)) {
        $lanes = @($Validated.validation_contract.requirements | Where-Object {
                @($_.implementation_targets) -ccontains [string]$target.id
            } | ForEach-Object { @($_.validation_lanes) } | Where-Object {
                $candidate = [string]$_
                @($Validated.validation_contract.lanes | Where-Object {
                    [string]$_.id -ceq $candidate -and [string]$_.phase -ceq 'implementation'
                }).Count -eq 1
            } | Sort-Object -Unique)
        if ($lanes.Count -ne 1) { throw "Target '$($target.id)' lacks one implementation lane." }
        $map[[string]$target.id] = [string]$lanes[0]
    }
    $map
}

function Get-TessaraCoordinatorPrerequisiteClosure {
    param([hashtable]$Targets, [string]$TargetId)
    $result = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    function Visit-CoordinatorPrerequisite([string]$Id) {
        foreach ($prerequisite in @($Targets[$Id].prerequisites | Sort-Object)) {
            $prerequisiteId = [string]$prerequisite
            if ($result.Add($prerequisiteId)) { Visit-CoordinatorPrerequisite $prerequisiteId }
        }
    }
    Visit-CoordinatorPrerequisite $TargetId
    @($result | Sort-Object)
}

function Get-TessaraCoordinatorContext {
    param([string]$AdapterPath, [string]$RepositoryRoot, $Validated, $Platform)
    $candidate = Get-TessaraPlatformCandidateIdentityFromValidation `
        -ValidatedAdapter $Validated -RepositoryRoot $RepositoryRoot
    $firstLane = [string]@($Validated.adapter.lanes)[0].id
    $plan = Invoke-TessaraValidationLane -AdapterPath $AdapterPath -LaneId $firstLane `
        -CandidateFingerprint ([string]$candidate.candidate_fingerprint) `
        -EvidenceRoot ([string]$Validated.validation_contract.evidence_policy.root) `
        -RepositoryRoot $RepositoryRoot -PlanOnly
    $body = [pscustomobject][ordered]@{
        candidate_fingerprint = [string]$candidate.candidate_fingerprint
        compatibility_plan_fingerprint = [string]$plan.fingerprint
        validation_contract_sha256 = [string]$Validated.validation_contract_sha256
        adapter_sha256 = [string]$Validated.adapter_fingerprint
        platform_fingerprint = [string]$Platform.platform_fingerprint
        platform_execution_fingerprint = [string]$Platform.execution_fingerprint
    }
    [pscustomobject][ordered]@{
        candidate = $candidate
        compatibility_plan = $plan
        context_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value $body
    }
}

function Read-TessaraCoordinatorPriorReceipt {
    param([string]$RepositoryRoot, $Reference)
    if ($null -eq $Reference) { return $null }
    try {
        $path = Get-TessaraCoordinatorReference $RepositoryRoot $Reference 'Prior target receipt'
        $document = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -Depth 100
        Assert-TessaraJsonSchema -Document $document -Kind implementation_target_completion `
            -Label 'Prior target receipt'
        [pscustomobject]@{ document = $document; reference = $Reference; valid = $true; error = $null }
    } catch {
        [pscustomobject]@{ document = $null; reference = $Reference; valid = $false; error = $_.Exception.Message }
    }
}

function Read-TessaraCoordinatorState {
    param([string]$RepositoryRoot, [string]$StatePath, $Validated, $Platform)
    $map = @{}
    $reference = $null
    if (-not [string]::IsNullOrWhiteSpace($StatePath)) {
        $full = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $RepositoryRoot `
            -Path $StatePath -Label 'Implementation target state'
        $document = Get-Content -LiteralPath $full -Raw | ConvertFrom-Json -Depth 100
        Assert-TessaraJsonSchema -Document $document -Kind implementation_target_state `
            -Label 'Implementation target state'
        if ([string]$document.sprint -cne [string]$Validated.validation_contract.sprint -or
            [string]$document.validation_contract.sha256 -cne [string]$Validated.validation_contract_sha256 -or
            [string]$document.validation_adapter.sha256 -cne [string]$Validated.adapter_fingerprint -or
            [string]$document.platform_identity.platform_fingerprint -cne [string]$Platform.platform_fingerprint) {
            throw 'Implementation target state is stale for the current sprint contract, adapter, or platform.'
        }
        foreach ($item in @($document.targets)) {
            if ($map.ContainsKey([string]$item.id)) { throw "Target state duplicates '$($item.id)'." }
            $map[[string]$item.id] = $item
        }
        $reference = ConvertTo-TessaraCoordinatorReference -RepositoryRoot $RepositoryRoot -Path $full
    }
    [pscustomobject]@{ items = $map; reference = $reference }
}

function Test-TessaraCoordinatorFingerprintSetEqual {
    param($Left, $Right)
    (Get-TessaraPlatformCanonicalJsonSha256 -Value @($Left | Sort-Object domain)) -ceq
        (Get-TessaraPlatformCanonicalJsonSha256 -Value @($Right | Sort-Object domain))
}

function New-TessaraCoordinatorPlan {
    param([string]$AdapterPath, [string]$RepositoryRoot, [string]$EvidenceRoot,
        [string]$StatePath, [string]$ExpectedCandidateFingerprint)
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $validated = Assert-TessaraValidationAdapter -AdapterPath $AdapterPath -RepositoryRoot $root
    if ([int]$validated.validation_contract.schema_version -ne 3 -or
        [string]$validated.validation_contract.validation_platform.implementation_coordinator.entrypoint -cne
            'Invoke-TessaraImplementationHarvest') {
        throw 'The shared implementation coordinator requires validation contract v3 activation.'
    }
    $platform = Get-TessaraValidationPlatformIdentity
    $context = Get-TessaraCoordinatorContext $AdapterPath $root $validated $platform
    if (-not [string]::IsNullOrWhiteSpace($ExpectedCandidateFingerprint) -and
        $ExpectedCandidateFingerprint -cne [string]$context.candidate.candidate_fingerprint) {
        throw 'Caller candidate fingerprint does not match the coordinator plan.'
    }
    $targets = Get-TessaraCoordinatorTargetMap $validated.validation_contract
    $laneByTarget = Get-TessaraCoordinatorLaneMap $validated
    $planLanes = @{}; foreach ($lane in @($context.compatibility_plan.body.lanes)) {
        $planLanes[[string]$lane.lane_id] = $lane
    }
    $state = Read-TessaraCoordinatorState $root $StatePath $validated $platform
    foreach ($stateId in $state.items.Keys) {
        if (-not $targets.ContainsKey([string]$stateId)) { throw "Target state contains unknown target '$stateId'." }
    }
    $changedPaths = Get-TessaraCoordinatorChangedPaths $root
    $unknownImpact = @($changedPaths | Where-Object {
        $changedPath = [string]$_
        @($validated.validation_contract.dependency_domains | Where-Object {
            $patterns = if ($_.PSObject.Properties.Name -contains 'inputs') {
                @($_.inputs | ForEach-Object { [string]$_.path })
            } else { @($_.tracked_inputs) }
            @($patterns | Where-Object { $changedPath -clike ([string]$_).Replace('\', '/') }).Count -gt 0
        }).Count -eq 0
    }).Count -gt 0
    $preliminary = @{}
    foreach ($id in @($targets.Keys | Sort-Object)) {
        $target = $targets[$id]
        $lane = $planLanes[$laneByTarget[$id]]
        $stateItem = if ($state.items.ContainsKey($id)) { $state.items[$id] } else { $null }
        $prior = Read-TessaraCoordinatorPriorReceipt $root $(if ($null -ne $stateItem) { $stateItem.previous_receipt } else { $null })
        $provenance = if ($null -ne $stateItem) { $stateItem.provenance } else { $null }
        $provenanceRef = if ($null -ne $provenance) { $provenance.record } else { $null }
        $provenanceValid = $true
        if ($null -ne $provenance) {
            try {
                $provenancePath = Get-TessaraCoordinatorReference $root $provenance.record 'Target provenance'
                $provenanceDocument = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json -Depth 100
                Assert-TessaraJsonSchema -Document $provenanceDocument -Kind defect_provenance_v2 -Label 'Target provenance'
                if ([string]$provenanceDocument.status -cne [string]$provenance.status -or
                    [string]$provenanceDocument.classification -cne [string]$provenance.classification -or
                    [string]$provenanceDocument.correction.identity -cne [string]$provenance.correction_identity -or
                    [string]$provenanceDocument.target -cne [string]$id -or
                    [string]$provenanceDocument.lane -cne [string]$laneByTarget[$id] -or
                    [string]$provenanceDocument.validation_contract.sha256 -cne [string]$validated.validation_contract_sha256 -or
                    [string]$provenanceDocument.validation_adapter.sha256 -cne [string]$validated.adapter_fingerprint -or
                    [string]$provenanceDocument.platform_identity.platform_fingerprint -cne [string]$platform.platform_fingerprint -or
                    $null -eq $prior -or -not $prior.valid -or
                    [string]$provenanceDocument.failed_receipt.sha256 -cne [string]$prior.reference.sha256 -or
                    (Get-TessaraPlatformCanonicalJsonSha256 -Value @($provenanceDocument.focused_reproducers)) -cne
                        (Get-TessaraPlatformCanonicalJsonSha256 -Value @($provenance.focused_reproducers))) {
                    throw 'Target provenance summary disagrees with its record.'
                }
                foreach ($reproducer in @($provenanceDocument.focused_reproducers | Where-Object {
                            [string]$_.state -ceq 'passed'
                        })) {
                    $null = Get-TessaraCoordinatorReference $root $reproducer.evidence `
                        "Focused reproducer '$([string]$reproducer.id)'"
                }
            } catch { $provenanceValid = $false }
        }
        $priorFailed = $null -ne $prior -and $prior.valid -and
            [string]$prior.document.state -in @('failed', 'blocked')
        $corrected = $priorFailed -and $provenanceValid -and $null -ne $provenance -and
            [string]$provenance.status -in @('corrected', 'verified') -and
            -not [string]::IsNullOrWhiteSpace([string]$provenance.correction_identity) -and
            @($provenance.focused_reproducers).Count -gt 0 -and
            @($provenance.focused_reproducers | Where-Object {
                [string]$_.state -cne 'passed' -or $null -eq $_.evidence
            }).Count -eq 0
        $blockedByProvenance = $priorFailed -and -not $corrected
        $domains = @($target.dependency_domains)
        $pathAffected = $unknownImpact -or @($changedPaths | Where-Object {
            $changedPath = [string]$_
            @($validated.validation_contract.dependency_domains | Where-Object {
                [string]$_.name -in $domains -and
                @($(if ($_.PSObject.Properties.Name -contains 'inputs') {
                            @($_.inputs | ForEach-Object { [string]$_.path })
                        } else { @($_.tracked_inputs) }) | Where-Object {
                        $changedPath -clike ([string]$_).Replace('\', '/')
                    }).Count -gt 0
            }).Count -gt 0
        }).Count -gt 0
        $preliminary[$id] = [pscustomobject][ordered]@{
            target = $target; lane_id = $laneByTarget[$id]; lane = $lane; prior = $prior
            provenance = $provenance; provenance_reference = $provenanceRef
            corrected = $corrected; blocked_by_provenance = $blockedByProvenance
            path_affected = $pathAffected
            command_identity = Get-TessaraPlatformCanonicalJsonSha256 -Value $target.command
            prerequisite_closure = @(Get-TessaraCoordinatorPrerequisiteClosure $targets $id)
        }
    }
    $reuseMemo = @{}; $reuseVisiting = @{}
    function Test-CoordinatorReuse([string]$Id) {
        if ($reuseMemo.ContainsKey($Id)) { return [bool]$reuseMemo[$Id] }
        if ($reuseVisiting.ContainsKey($Id)) { throw "Coordinator reuse closure contains a cycle at '$Id'." }
        $reuseVisiting[$Id] = $true
        $entry = $preliminary[$Id]; $prior = $entry.prior
        $eligible = [string]$validated.validation_contract.validation_platform.implementation_coordinator.reuse_policy -ceq 'authenticated-unchanged' -and
            $null -ne $prior -and $prior.valid -and [string]$prior.document.state -ceq 'passed' -and
            -not [bool]$entry.path_affected -and
            [string]$prior.document.validation_contract.sha256 -ceq [string]$validated.validation_contract_sha256 -and
            [string]$prior.document.validation_adapter.sha256 -ceq [string]$validated.adapter_fingerprint -and
            [string]$prior.document.platform_identity.platform_fingerprint -ceq [string]$platform.platform_fingerprint -and
            [string]$prior.document.platform_identity.execution_fingerprint -ceq [string]$platform.execution_fingerprint -and
            [string]$prior.document.compatibility_fingerprint -ceq [string]$entry.lane.compatibility_fingerprint -and
            [string]$prior.document.command_identity -ceq [string]$entry.command_identity -and
            [string]$prior.document.environment_fingerprint -ceq [string]$entry.lane.environment_fingerprint -and
            (Test-TessaraCoordinatorFingerprintSetEqual $prior.document.dependency_fingerprints $entry.lane.dependency_fingerprints) -and
            $null -ne $prior.document.lane_result
        if ($eligible) {
            try { $null = Get-TessaraCoordinatorReference $root $prior.document.lane_result 'Reusable lane result' }
            catch { $eligible = $false }
        }
        if ($eligible) {
            foreach ($prerequisite in @($entry.target.prerequisites)) {
                if (-not (Test-CoordinatorReuse ([string]$prerequisite))) { $eligible = $false; break }
                $priorClosure = @($prior.document.prerequisite_closure | Where-Object {
                    [string]$_.target -ceq [string]$prerequisite -and
                    [string]$_.compatibility_fingerprint -ceq
                        [string]$preliminary[[string]$prerequisite].lane.compatibility_fingerprint
                })
                if ($priorClosure.Count -ne 1) { $eligible = $false; break }
                try { $null = Get-TessaraCoordinatorReference $root $priorClosure[0].receipt 'Reusable prerequisite receipt' }
                catch { $eligible = $false; break }
            }
        }
        $reuseVisiting.Remove($Id); $reuseMemo[$Id] = $eligible; return $eligible
    }
    foreach ($id in @($targets.Keys)) { $null = Test-CoordinatorReuse ([string]$id) }
    $entries = @{}
    foreach ($id in @($targets.Keys)) {
        $item = $preliminary[$id]
        if ($item.corrected) { $group = 1; $disposition = 'execute'; $rationale = 'corrected-failure-with-passing-focused-reproducers' }
        elseif ($item.blocked_by_provenance) { $group = 1; $disposition = 'blocked'; $rationale = 'provenance-gate-unclassified-open-blocked-or-uncorrected' }
        elseif ($null -eq $item.prior) { $group = 2; $disposition = 'execute'; $rationale = 'never-run-required-target' }
        elseif ([bool]$reuseMemo[$id]) { $group = 4; $disposition = 'reuse'; $rationale = 'authenticated-unchanged-target' }
        else { $group = 3; $disposition = 'execute'; $rationale = if ($unknownImpact) { 'unknown-dependency-impact-conservative-execution' } else { 'affected-or-reuse-ineligible-target' } }
        $entries[$id] = [pscustomobject]@{ group=$group; disposition=$disposition; rationale=$rationale }
    }
    do {
        $blockedChanged = $false
        foreach ($id in @($targets.Keys)) {
            if ([string]$entries[$id].disposition -cne 'blocked' -and
                @($targets[$id].prerequisites | Where-Object {
                        [string]$entries[[string]$_].disposition -ceq 'blocked'
                    }).Count -gt 0) {
                $entries[$id].disposition = 'blocked'
                $entries[$id].rationale = 'blocked-prerequisite-provenance-gate'
                $blockedChanged = $true
            }
        }
    } while ($blockedChanged)
    $ordered = [Collections.Generic.List[string]]::new()
    $orderedSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    function Add-CoordinatorOrdered([string]$Id) {
        foreach ($pre in @($targets[$Id].prerequisites | Sort-Object {
                    [int]$entries[[string]$_].group }, { [string]$_ })) {
            Add-CoordinatorOrdered ([string]$pre)
        }
        if ($orderedSet.Add($Id)) { $ordered.Add($Id) }
    }
    foreach ($id in @($targets.Keys | Sort-Object { [int]$entries[[string]$_].group }, { [string]$_ })) {
        Add-CoordinatorOrdered ([string]$id)
    }
    $schedule = [Collections.Generic.List[object]]::new(); $ordinal = 0
    foreach ($id in $ordered) {
        $ordinal++
        $item = $preliminary[$id]; $decision = $entries[$id]
        $schedule.Add([pscustomobject][ordered]@{
            ordinal=$ordinal; target=$id; lane=[string]$item.lane_id
            priority_group=[int]$decision.group; disposition=[string]$decision.disposition
            rationale=[string]$decision.rationale; prerequisite_closure=@($item.prerequisite_closure)
            dependency_fingerprints=@($item.lane.dependency_fingerprints)
            compatibility_fingerprint=[string]$item.lane.compatibility_fingerprint
            command_identity=[string]$item.command_identity
            environment_fingerprint=[string]$item.lane.environment_fingerprint
            previous_receipt=$(if ($null -ne $item.prior) { $item.prior.reference } else { $null })
            provenance=$item.provenance_reference; resource_claims=@($item.target.resource_claims)
        })
    }
    $contractRelative = [IO.Path]::GetRelativePath($root, [string]$validated.validation_contract_path).Replace('\','/')
    $adapterRelative = [IO.Path]::GetRelativePath($root, [string]$validated.adapter_path).Replace('\','/')
    $body = [pscustomobject][ordered]@{
        schema_version=1; contract='tessara.validation.implementation-coordinator-start'; policy_version='tessara-validation-v3'
        sprint=[string]$validated.validation_contract.sprint; context_fingerprint=[string]$context.context_fingerprint
        source_identity=$context.candidate.source_identity
        validation_contract=[pscustomobject][ordered]@{path=$contractRelative;sha256=[string]$validated.validation_contract_sha256}
        validation_adapter=[pscustomobject][ordered]@{path=$adapterRelative;sha256=[string]$validated.adapter_fingerprint}
        platform_identity=[pscustomobject][ordered]@{release_version=[string]$platform.release_version;platform_fingerprint=[string]$platform.platform_fingerprint;execution_fingerprint=[string]$platform.execution_fingerprint}
        state_input=$state.reference; ordering='evidentiary-priority-v1'; execution_mode='serial-resource-safe'
        schedule=@($schedule); finalization=[pscustomobject][ordered]@{priority_group=5;disposition='finalize';rationale='all-required-targets-terminal-and-authenticated'}
    }
    $scheduleDigest = Get-TessaraPlatformCanonicalJsonSha256 -Value $body
    $start = [ordered]@{}; foreach ($property in $body.PSObject.Properties) { $start[$property.Name]=$property.Value }
    $start.schedule_digest = $scheduleDigest
    $startDocument = [pscustomobject]$start
    $evidenceFull = [IO.Path]::GetFullPath($EvidenceRoot)
    $planRoot = Join-Path $evidenceFull "implementation-coordinator/plans/$scheduleDigest"
    $startRef = Publish-TessaraCoordinatorImmutableJson $root (Join-Path $planRoot 'coordinator-start.json') $startDocument implementation_coordinator_start
    [pscustomobject]@{ root=$root; validated=$validated; platform=$platform; context=$context; targets=$targets; lane_by_target=$laneByTarget; start=$startDocument; start_reference=$startRef; plan_root=$planRoot }
}

function Assert-TessaraCoordinatorContextCurrent {
    param($Plan, $Scheduled)
    $platform = Get-TessaraValidationPlatformIdentity
    $candidate = Get-TessaraPlatformCandidateIdentityFromValidation `
        -ValidatedAdapter $Plan.validated -RepositoryRoot $Plan.root
    $contractHash = (Get-FileHash -Algorithm SHA256 -LiteralPath `
        ([string]$Plan.validated.validation_contract_path)).Hash.ToLowerInvariant()
    $adapterHash = (Get-FileHash -Algorithm SHA256 -LiteralPath `
        ([string]$Plan.validated.adapter_path)).Hash.ToLowerInvariant()
    $contractCurrent = $contractHash -ceq [string]$Plan.start.validation_contract.sha256
    $adapterCurrent = $adapterHash -ceq [string]$Plan.start.validation_adapter.sha256
    $platformCurrent = [string]$platform.platform_fingerprint -ceq [string]$Plan.start.platform_identity.platform_fingerprint
    $candidateCurrent = [string]$candidate.candidate_fingerprint -ceq [string]$Plan.context.candidate.candidate_fingerprint
    $current = $contractCurrent -and $adapterCurrent -and $platformCurrent -and $candidateCurrent
    if ($current -and $null -ne $Plan.start.state_input) {
        try { $null = Get-TessaraCoordinatorReference $Plan.root $Plan.start.state_input 'Coordinator state input' }
        catch { $current = $false }
    }
    if ($current -and $null -ne $Scheduled) {
        $adapterLane = @($Plan.validated.adapter.lanes | Where-Object {
            [string]$_.id -ceq [string]$Scheduled.lane
        })[0]
        $current = (Get-TessaraPlatformLaneEnvironmentObservationFingerprint $adapterLane) -ceq
            [string]$Scheduled.environment_fingerprint
    }
    if (-not $current) {
        throw "Coordinator source, fixture, contract, environment, command, adapter, or dependency context changed; a new plan is required. contract=$contractCurrent adapter=$adapterCurrent platform=$platformCurrent candidate=$candidateCurrent"
    }
}

function Invoke-TessaraImplementationHarvest {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$AdapterPath,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [string]$RepositoryRoot = $script:RepositoryRoot,
        [string]$TargetStatePath,
        [Parameter(DontShow)][scriptblock]$LaneInvoker,
        [Parameter(DontShow)][scriptblock]$AfterCheckpoint
    )
    $plan = New-TessaraCoordinatorPlan $AdapterPath $RepositoryRoot $EvidenceRoot `
        $TargetStatePath $CandidateFingerprint
    $checkpointPath = Join-Path $plan.plan_root 'checkpoint.json'
    $completed = [Collections.Generic.List[object]]::new(); $completedMap = @{}
    $findings = [Collections.Generic.List[object]]::new()
    if (Test-Path -LiteralPath $checkpointPath -PathType Leaf) {
        $checkpoint = Get-Content -LiteralPath $checkpointPath -Raw | ConvertFrom-Json -Depth 100
        Assert-TessaraJsonSchema $checkpoint implementation_coordinator_checkpoint 'Implementation coordinator checkpoint'
        if ([string]$checkpoint.schedule_digest -cne [string]$plan.start.schedule_digest -or
            [string]$checkpoint.context_fingerprint -cne [string]$plan.start.context_fingerprint) {
            throw 'Coordinator checkpoint does not authenticate the immutable schedule.'
        }
        foreach ($item in @($checkpoint.completed | Sort-Object ordinal)) {
            $receiptPath = Get-TessaraCoordinatorReference $plan.root $item.receipt 'Completed target receipt'
            $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json -Depth 100
            Assert-TessaraJsonSchema $receipt implementation_target_completion 'Completed target receipt'
            if ([string]$receipt.schedule_digest -cne [string]$plan.start.schedule_digest -or
                [string]$receipt.target -cne [string]$item.target) { throw 'Completed target receipt does not bind the schedule.' }
            $completed.Add($item); $completedMap[[string]$item.target] = [pscustomobject]@{document=$receipt;reference=$item.receipt}
            if ([string]$receipt.state -ceq 'failed') {
                $targetRoot = Join-Path $plan.plan_root "targets/$('{0:d4}' -f [int]$item.ordinal)-$([string]$item.target)"
                $provenancePath = Join-Path $targetRoot 'defect-provenance.json'
                if (-not (Test-Path -LiteralPath $provenancePath -PathType Leaf)) {
                    throw "Recovered failed target '$([string]$item.target)' lacks defect provenance."
                }
                $provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json -Depth 100
                Assert-TessaraJsonSchema $provenance defect_provenance_v2 'Recovered target defect provenance'
                $provenanceRef = ConvertTo-TessaraCoordinatorReference $plan.root $provenancePath
                if ([string]$provenance.target -cne [string]$item.target -or
                    [string]$provenance.failed_receipt.sha256 -cne [string]$item.receipt.sha256) {
                    throw "Recovered failed target '$([string]$item.target)' has mismatched defect provenance."
                }
                $findings.Add([pscustomobject][ordered]@{
                    id=[string]$provenance.record_id;target=[string]$item.target
                    classification=[string]$provenance.classification;receipt=$item.receipt
                    provenance=$provenanceRef;reason=[string]$provenance.reason
                })
            }
        }
    }
    $runtimeFailure = $false; $unsafeHalt = $false
    foreach ($scheduled in @($plan.start.schedule | Sort-Object ordinal)) {
        $id = [string]$scheduled.target
        if ($completedMap.ContainsKey($id)) {
            if ([string]$completedMap[$id].document.state -eq 'failed') { $runtimeFailure = $true }
            continue
        }
        Assert-TessaraCoordinatorContextCurrent $plan $scheduled
        $target = $plan.targets[$id]
        $laneContract = @($plan.validated.validation_contract.lanes | Where-Object { [string]$_.id -ceq [string]$scheduled.lane })[0]
        $liveState = [bool]$laneContract.touches_live_state -or @($target.resource_claims | Where-Object { [string]$_.kind -cne 'evidence-path' }).Count -gt 0
        $startDocument = [pscustomobject][ordered]@{schema_version=1;contract='tessara.validation.implementation-target-start';policy_version='tessara-validation-v3';sprint=[string]$plan.start.sprint;schedule_digest=[string]$plan.start.schedule_digest;ordinal=[int]$scheduled.ordinal;target=$id;lane=[string]$scheduled.lane;priority_group=[int]$scheduled.priority_group;planned_disposition=[string]$scheduled.disposition;new_execution=([string]$scheduled.disposition -ceq 'execute');resource_claims=@($scheduled.resource_claims)}
        $targetRoot = Join-Path $plan.plan_root "targets/$('{0:d4}' -f [int]$scheduled.ordinal)-$id"
        $null = Publish-TessaraCoordinatorImmutableJson $plan.root (Join-Path $targetRoot 'start.json') $startDocument implementation_target_start
        $runtimeDisposition = [string]$scheduled.disposition; $state='blocked'; $reason=[string]$scheduled.rationale
        $laneResultRef=$null; $reusedReceipt=$null; $newlyExecuted=$false; $cleanupState=if($liveState){'blocked'}else{'not_applicable'}
        $failedPrerequisites = @($target.prerequisites | Where-Object {
            -not $completedMap.ContainsKey([string]$_) -or [string]$completedMap[[string]$_].document.state -cne 'passed'
        })
        if ($failedPrerequisites.Count -gt 0) { $runtimeDisposition='blocked'; $reason='prerequisite-failed-or-blocked' }
        elseif ($unsafeHalt -or ($runtimeFailure -and [string]$target.continuation -ceq 'unsafe-live-state')) { $runtimeDisposition='blocked'; $reason='unsafe-after-prior-failure' }
        elseif ($runtimeDisposition -ceq 'reuse') {
            $prior = Read-TessaraCoordinatorPriorReceipt $plan.root $scheduled.previous_receipt
            if ($null -eq $prior -or -not $prior.valid) { throw "Planned reused target '$id' lost its authenticated receipt; a new plan is required." }
            $state='passed'; $reason='authenticated-reuse'; $reusedReceipt=$scheduled.previous_receipt; $laneResultRef=$prior.document.lane_result; $cleanupState=[string]$prior.document.cleanup_restoration.state
        } elseif ($runtimeDisposition -ceq 'execute') {
            $newlyExecuted=$true
            try {
                $prerequisiteLanePaths = @($target.prerequisites | ForEach-Object {
                    $laneRef = $completedMap[[string]$_].document.lane_result
                    if ($null -eq $laneRef) { throw "Prerequisite target '$_' lacks authenticated lane evidence." }
                    Get-TessaraCoordinatorReference $plan.root $laneRef 'Prerequisite lane result'
                })
                $laneResult = if ($null -ne $LaneInvoker) {
                    $claimArray = @($scheduled.resource_claims)
                    & $LaneInvoker $id ([string]$scheduled.lane) $claimArray
                }
                else { Invoke-TessaraValidationLane -AdapterPath $AdapterPath -LaneId ([string]$scheduled.lane) -CandidateFingerprint $CandidateFingerprint -EvidenceRoot $EvidenceRoot -RepositoryRoot $plan.root -PrerequisiteResultPaths $prerequisiteLanePaths }
                if ($null -eq $laneResult) { throw 'Lane returned no result.' }
                if ($laneResult.PSObject.Properties.Name -contains 'evidence_path' -and -not [string]::IsNullOrWhiteSpace([string]$laneResult.evidence_path)) {
                    $laneFull = [IO.Path]::GetFullPath([string]$laneResult.evidence_path)
                    $laneResultRef = ConvertTo-TessaraCoordinatorReference $plan.root $laneFull
                } else {
                    $laneResultRef = Publish-TessaraCoordinatorImmutableJson $plan.root (Join-Path $targetRoot 'lane-result.json') $laneResult $null
                }
                $state = if ([string]$laneResult.state -ceq 'passed') {'passed'} else {'failed'}
                $reason = if ($state -eq 'passed') {$null} else { if($laneResult.PSObject.Properties.Name -contains 'failure_stage'){"lane-$([string]$laneResult.failure_stage)"}else{'lane-failed'} }
                if ($laneResult.PSObject.Properties.Name -contains 'cleanup_restoration') { $cleanupState=[string]$laneResult.cleanup_restoration.state }
                elseif ($liveState) { $cleanupState='failed' }
                if ($liveState -and $cleanupState -cne 'passed') { $state='failed'; $reason='cleanup-restoration-failed'; $unsafeHalt=$true }
            } catch {
                $state='failed'; $reason="lane-exception: $($_.Exception.Message)"; if($liveState){$cleanupState='failed';$unsafeHalt=$true}
            }
        }
        if ($state -eq 'failed') { $runtimeFailure=$true }
        $prerequisiteClosure = @($target.prerequisites | ForEach-Object {
            $prerequisiteId = [string]$_
            $prerequisiteSchedule = @($plan.start.schedule | Where-Object {
                [string]$_.target -ceq $prerequisiteId
            })[0]
            [pscustomobject][ordered]@{
                target = $prerequisiteId
                compatibility_fingerprint = [string]$prerequisiteSchedule.compatibility_fingerprint
                receipt = $completedMap[$prerequisiteId].reference
            }
        })
        $completion = [pscustomobject][ordered]@{schema_version=2;contract='tessara.validation.implementation-target-receipt';policy_version='tessara-validation-v3';sprint=[string]$plan.start.sprint;schedule_digest=[string]$plan.start.schedule_digest;ordinal=[int]$scheduled.ordinal;target=$id;lane=[string]$scheduled.lane;priority_group=[int]$scheduled.priority_group;disposition=$(if($runtimeDisposition -eq 'execute'){'executed'}elseif($runtimeDisposition -eq 'reuse'){'reused'}else{'blocked'});state=$state;reason=$(if([string]::IsNullOrWhiteSpace($reason)){$null}else{$reason});newly_executed=$newlyExecuted;source_identity=$plan.context.candidate.source_identity;validation_contract=$plan.start.validation_contract;validation_adapter=$plan.start.validation_adapter;platform_identity=$plan.start.platform_identity;compatibility_fingerprint=[string]$scheduled.compatibility_fingerprint;command_identity=[string]$scheduled.command_identity;dependency_fingerprints=@($scheduled.dependency_fingerprints);environment_fingerprint=[string]$scheduled.environment_fingerprint;prerequisite_closure=$prerequisiteClosure;resource_claims=@($scheduled.resource_claims);lane_result=$laneResultRef;reused_receipt=$reusedReceipt;cleanup_restoration=[pscustomobject][ordered]@{required=$liveState;state=$cleanupState}}
        $completionRef = Publish-TessaraCoordinatorImmutableJson $plan.root (Join-Path $targetRoot 'completion.json') $completion implementation_target_completion
        $completedItem=[pscustomobject][ordered]@{ordinal=[int]$scheduled.ordinal;target=$id;receipt=$completionRef}; $completed.Add($completedItem); $completedMap[$id]=[pscustomobject]@{document=$completion;reference=$completionRef}
        if ($state -eq 'failed') {
            $provenance=[pscustomobject][ordered]@{schema_version=2;contract='tessara.validation.defect-provenance';policy_version='tessara-validation-v3';sprint=[string]$plan.start.sprint;record_id="implementation-$id-$([string]$plan.start.schedule_digest.Substring(0,12))";status='open';target=$id;lane=[string]$scheduled.lane;classification='unresolved';reason=$reason;source_identity=$plan.context.candidate.source_identity;validation_contract=$plan.start.validation_contract;validation_adapter=$plan.start.validation_adapter;platform_identity=[pscustomobject][ordered]@{release_version=[string]$plan.platform.release_version;platform_fingerprint=[string]$plan.platform.platform_fingerprint};failed_receipt=$completionRef;correction=$null;focused_reproducers=@()}
            $provenanceRef=Publish-TessaraCoordinatorImmutableJson $plan.root (Join-Path $targetRoot 'defect-provenance.json') $provenance defect_provenance_v2
            $findings.Add([pscustomobject][ordered]@{id=[string]$provenance.record_id;target=$id;classification='unresolved';receipt=$completionRef;provenance=$provenanceRef;reason=$reason})
        }
        $checkpoint=[pscustomobject][ordered]@{schema_version=1;contract='tessara.validation.implementation-coordinator-checkpoint';policy_version='tessara-validation-v3';sprint=[string]$plan.start.sprint;schedule_digest=[string]$plan.start.schedule_digest;context_fingerprint=[string]$plan.start.context_fingerprint;completed=@($completed | Sort-Object ordinal)}
        $null=Set-TessaraCoordinatorCheckpoint $plan.root $checkpointPath $checkpoint
        if ($null -ne $AfterCheckpoint) { & $AfterCheckpoint $id ([int]$scheduled.ordinal) }
    }
    Assert-TessaraCoordinatorContextCurrent $plan $null
    $targetRecords=@($plan.start.schedule | Sort-Object ordinal | ForEach-Object { $record=$completedMap[[string]$_.target]; [pscustomobject][ordered]@{id=[string]$_.target;lane=[string]$_.lane;priority_group=[int]$_.priority_group;state=[string]$record.document.state;disposition=[string]$record.document.disposition;receipt=$record.reference} })
    $batchPath = Join-Path $plan.plan_root 'defect-batch.json'
    if (Test-Path -LiteralPath $batchPath -PathType Leaf) {
        $batch = Get-Content -LiteralPath $batchPath -Raw | ConvertFrom-Json -Depth 100
        Assert-TessaraJsonSchema $batch implementation_defect_batch 'Recovered implementation defect batch'
        if ([string]$batch.schedule_digest -cne [string]$plan.start.schedule_digest -or
            (Get-TessaraPlatformCanonicalJsonSha256 -Value @($batch.targets)) -cne
                (Get-TessaraPlatformCanonicalJsonSha256 -Value @($targetRecords))) {
            throw 'Recovered defect batch does not authenticate the immutable schedule and completions.'
        }
        $batchRef = ConvertTo-TessaraCoordinatorReference $plan.root $batchPath
    } else {
        $batch=[pscustomobject][ordered]@{schema_version=1;contract='tessara.validation.implementation-defect-batch';policy_version='tessara-validation-v3';sprint=[string]$plan.start.sprint;schedule_digest=[string]$plan.start.schedule_digest;coordinator_start=$plan.start_reference;source_identity=$plan.context.candidate.source_identity;validation_contract=$plan.start.validation_contract;validation_adapter=$plan.start.validation_adapter;platform_identity=[pscustomobject][ordered]@{release_version=[string]$plan.platform.release_version;platform_fingerprint=[string]$plan.platform.platform_fingerprint};targets=$targetRecords;findings=@($findings | Sort-Object id)}
        $batchRef=Publish-TessaraCoordinatorImmutableJson $plan.root $batchPath $batch implementation_defect_batch
    }
    $checkpointRef=ConvertTo-TessaraCoordinatorReference $plan.root $checkpointPath
    $failedCount=@($targetRecords | Where-Object {$_.state -eq 'failed'}).Count; $blockedCount=@($targetRecords | Where-Object {$_.state -eq 'blocked'}).Count
    $liveResults=@($completedMap.Values | ForEach-Object {$_.document} | Where-Object {[bool]$_.cleanup_restoration.required})
    $topologyRestoration=if($liveResults.Count -eq 0){'not_applicable'}elseif(@($liveResults | Where-Object {$_.cleanup_restoration.state -ne 'passed'}).Count -eq 0){'passed'}else{'failed'}
    $openDefectCount = @($batch.findings).Count
    $finalization=[pscustomobject][ordered]@{schema_version=1;contract='tessara.validation.implementation-coordinator-finalization';policy_version='tessara-validation-v3';sprint=[string]$plan.start.sprint;state=$(if($failedCount -eq 0 -and $blockedCount -eq 0 -and $openDefectCount -eq 0 -and $topologyRestoration -ne 'failed'){'passed'}else{'failed'});schedule_digest=[string]$plan.start.schedule_digest;context_fingerprint=[string]$plan.start.context_fingerprint;start=$plan.start_reference;checkpoint=$checkpointRef;defect_batch=$batchRef;targets=@($targetRecords | ForEach-Object {[pscustomobject][ordered]@{target=$_.id;state=$_.state;disposition=$_.disposition;receipt=$_.receipt}});executed_count=@($targetRecords | Where-Object {$_.disposition -eq 'executed'}).Count;reused_count=@($targetRecords | Where-Object {$_.disposition -eq 'reused'}).Count;blocked_count=$blockedCount;failed_count=$failedCount;open_defect_count=$openDefectCount;topology_restoration=$topologyRestoration;source_identity=$plan.context.candidate.source_identity;validation_contract=$plan.start.validation_contract;validation_adapter=$plan.start.validation_adapter;platform_identity=$plan.start.platform_identity}
    $finalizationRef=Publish-TessaraCoordinatorImmutableJson $plan.root (Join-Path $plan.plan_root 'finalization.json') $finalization implementation_coordinator_finalization
    [pscustomobject][ordered]@{state=[string]$finalization.state;plan=$plan.start;plan_path=(Join-Path $plan.plan_root 'coordinator-start.json');plan_sha256=[string]$plan.start_reference.sha256;batch=$batch;batch_path=$batchPath;batch_sha256=[string]$batchRef.sha256;finalization=$finalization;finalization_path=(Join-Path $plan.plan_root 'finalization.json');finalization_sha256=[string]$finalizationRef.sha256}
}

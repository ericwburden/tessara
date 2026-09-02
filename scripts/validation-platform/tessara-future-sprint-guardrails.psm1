Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-TessaraGuardrailMap {
    param([object[]]$Items, [string]$Property, [string]$Label)
    $map = @{}
    foreach ($item in @($Items)) {
        $id = [string]$item.$Property
        if ([string]::IsNullOrWhiteSpace($id) -or $map.ContainsKey($id)) {
            throw "$Label contains a missing or duplicate '$id'."
        }
        $map[$id] = $item
    }
    $map
}

function Test-TessaraGuardrailDependency {
    param([hashtable]$Targets, [string]$TargetId, [string]$RequiredId)
    $pending = [Collections.Generic.Queue[string]]::new()
    foreach ($id in @($Targets[$TargetId].prerequisites)) { $pending.Enqueue([string]$id) }
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    while ($pending.Count -gt 0) {
        $id = $pending.Dequeue()
        if (-not $seen.Add($id)) { continue }
        if ($id -ceq $RequiredId) { return $true }
        if ($Targets.ContainsKey($id)) {
            foreach ($next in @($Targets[$id].prerequisites)) { $pending.Enqueue([string]$next) }
        }
    }
    return $false
}

function Assert-TessaraGuardrailAcyclic {
    param([hashtable]$Items, [string]$Property, [string]$Label)
    $visiting = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $visited = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    function Visit([string]$Id) {
        if ($visited.Contains($Id)) { return }
        if (-not $visiting.Add($Id)) { throw "$Label contains a cycle at '$Id'." }
        foreach ($next in @($Items[$Id].$Property)) { Visit ([string]$next) }
        $null = $visiting.Remove($Id)
        $null = $visited.Add($Id)
    }
    foreach ($id in @($Items.Keys)) { Visit ([string]$id) }
}

function Read-TessaraGuardrailReference {
    param([string]$RepositoryRoot, $Reference, [string]$Label)
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $full = [IO.Path]::GetFullPath((Join-Path $root ([string]$Reference.path)))
    if (-not $full.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath $full -PathType Leaf) -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $full).Hash.ToLowerInvariant() -cne [string]$Reference.sha256) {
        throw "$Label reference cannot be authenticated."
    }
    $full
}

function Assert-TessaraValidationContractV3Semantics {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Contract)

    if ([int]$Contract.schema_version -ne 3 -or
        [string]$Contract.policy_version -cne "tessara-validation-v3") {
        throw "Future-sprint guardrails require validation contract schema 3 and tessara-validation-v3."
    }
    $domains = Get-TessaraGuardrailMap @($Contract.dependency_domains) name "Dependency domains"
    $targets = Get-TessaraGuardrailMap @($Contract.implementation_targets) id "Implementation targets"
    $lanes = Get-TessaraGuardrailMap @($Contract.lanes) id "Validation lanes"
    $edges = Get-TessaraGuardrailMap @($Contract.controlled_artifact_edges) id "Controlled-artifact edges"
    $slices = Get-TessaraGuardrailMap @($Contract.implementation_slices) id "Implementation slices"
    $null = Get-TessaraGuardrailMap @($Contract.requirements) id "Requirements"

    if (-not [bool]$Contract.successor_certification.enabled -or
        [string]$Contract.successor_certification.planner_entrypoint -cne "New-TessaraSuccessorImpactPlan" -or
        [string]$Contract.successor_certification.validator_entrypoint -cne "Assert-TessaraSuccessorImpactPlan") {
        throw "Validation contract v3 must activate the shared successor-impact planner and validator."
    }

    foreach ($domain in @($Contract.dependency_domains)) {
        $inputKeys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $roles = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($input in @($domain.inputs)) {
            $key = "$([string]$input.role)/$([string]$input.path)"
            if (-not $inputKeys.Add($key)) { throw "Dependency domain '$($domain.name)' repeats input '$key'." }
            $null = $roles.Add([string]$input.role)
        }
        if (-not $roles.Contains("producer")) {
            throw "Dependency domain '$($domain.name)' must identify an actual producer input."
        }
        if ([string]$domain.default_impact -ceq "bounded") {
            if ([string]::IsNullOrWhiteSpace([string]$domain.bounded_rationale) -or
                -not $roles.Contains("test")) {
                throw "Bounded dependency domain '$($domain.name)' requires a rationale and tracked test input proving its closed cone."
            }
        } elseif ($null -ne $domain.bounded_rationale) {
            throw "Full-replay dependency domain '$($domain.name)' cannot claim a bounded-cone rationale."
        }
    }

    foreach ($phase in @("implementation", "validation-readiness", "candidate-rehearsal", "validation-preflight", "sit", "uat")) {
        if (@($Contract.lanes | Where-Object { [string]$_.phase -ceq $phase }).Count -eq 0) {
            throw "Validation contract does not declare a '$phase' lane."
        }
    }
    foreach ($lane in @($Contract.lanes)) {
        foreach ($domain in @($lane.dependency_domains)) {
            if (-not $domains.ContainsKey([string]$domain)) { throw "Lane '$($lane.id)' references unknown domain '$domain'." }
        }
        foreach ($prerequisite in @($lane.prerequisites)) {
            if (-not $lanes.ContainsKey([string]$prerequisite) -or [string]$prerequisite -ceq [string]$lane.id) {
                throw "Lane '$($lane.id)' has an invalid prerequisite '$prerequisite'."
            }
        }
    }
    Assert-TessaraGuardrailAcyclic $lanes prerequisites "Validation lane prerequisites"

    $implementationLaneByTarget = @{}
    $targetByImplementationLane = @{}
    foreach ($target in @($Contract.implementation_targets)) {
        foreach ($domain in @($target.dependency_domains)) {
            if (-not $domains.ContainsKey([string]$domain)) { throw "Target '$($target.id)' references unknown domain '$domain'." }
        }
        foreach ($prerequisite in @($target.prerequisites)) {
            if (-not $targets.ContainsKey([string]$prerequisite) -or [string]$prerequisite -ceq [string]$target.id) {
                throw "Target '$($target.id)' has an invalid prerequisite '$prerequisite'."
            }
        }
        if (-not $slices.ContainsKey([string]$target.slice)) { throw "Target '$($target.id)' references unknown slice '$($target.slice)'." }
        foreach ($edge in @($target.fanout_edges)) {
            if (-not $edges.ContainsKey([string]$edge)) { throw "Target '$($target.id)' references unknown fanout edge '$edge'." }
        }
        $claimKeys = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $hasEvidenceClaim = $false
        foreach ($claim in @($target.resource_claims)) {
            $claimKey = "$([string]$claim.kind)/$([string]$claim.identity)"
            if (-not $claimKeys.Add($claimKey)) { throw "Target '$($target.id)' has duplicate resource claim '$claimKey'." }
            if ([string]$claim.kind -ceq "evidence-path") { $hasEvidenceClaim = $true }
        }
        if (-not $hasEvidenceClaim) { throw "Target '$($target.id)' must claim its exclusive evidence path." }
        $mappedImplementationLanes = @($Contract.requirements | Where-Object {
                @($_.implementation_targets) -ccontains [string]$target.id
            } | ForEach-Object { @($_.validation_lanes) } | Where-Object {
                $lanes.ContainsKey([string]$_) -and [string]$lanes[[string]$_].phase -ceq "implementation"
            } | Sort-Object -Unique)
        if ($mappedImplementationLanes.Count -ne 1) {
            throw "Implementation target '$($target.id)' must map to exactly one implementation adapter lane."
        }
        $implementationLane = [string]$mappedImplementationLanes[0]
        if ($targetByImplementationLane.ContainsKey($implementationLane)) {
            throw "Implementation lane '$implementationLane' maps more than one focused target."
        }
        $implementationLaneByTarget[[string]$target.id] = $implementationLane
        $targetByImplementationLane[$implementationLane] = [string]$target.id
    }

    foreach ($domain in @($Contract.dependency_domains)) {
        $name = [string]$domain.name
        $expectedTargets = @($Contract.implementation_targets | Where-Object {
                @($_.dependency_domains) -ccontains $name
            } | ForEach-Object { [string]$_.id } | Sort-Object)
        $expectedLanes = @($Contract.lanes | Where-Object {
                @($_.dependency_domains) -ccontains $name
            } | ForEach-Object { [string]$_.id } | Sort-Object)
        if (($expectedTargets -join "`n") -cne
                (@($domain.consumers.implementation_targets | Sort-Object) -join "`n") -or
            ($expectedLanes -join "`n") -cne
                (@($domain.consumers.validation_lanes | Sort-Object) -join "`n")) {
            throw "Dependency domain '$name' consumer inventory does not exactly match its target and lane relationships."
        }
    }
    foreach ($lane in @($Contract.lanes | Where-Object { [string]$_.phase -ceq "implementation" })) {
        if (-not $targetByImplementationLane.ContainsKey([string]$lane.id)) {
            throw "Implementation lane '$($lane.id)' does not map exactly one focused target."
        }
        $target = $targets[$targetByImplementationLane[[string]$lane.id]]
        $expectedLanePrerequisites = @($target.prerequisites | ForEach-Object {
                [string]$implementationLaneByTarget[[string]$_]
            } | Sort-Object)
        if (($expectedLanePrerequisites -join "`n") -cne (@($lane.prerequisites | Sort-Object) -join "`n")) {
            throw "Implementation target '$($target.id)' prerequisites disagree with lane '$($lane.id)'."
        }
        if (([bool]$lane.touches_live_state -or [string]$target.continuation -ceq "unsafe-live-state") -and
            @($target.resource_claims | Where-Object { [string]$_.kind -cne "evidence-path" }).Count -eq 0) {
            throw "Live-state target '$($target.id)' must declare its exclusive topology, port, database, Docker, process, or service claims."
        }
    }
    Assert-TessaraGuardrailAcyclic $targets prerequisites "Implementation target prerequisites"

    foreach ($requirement in @($Contract.requirements)) {
        foreach ($target in @($requirement.implementation_targets)) {
            if (-not $targets.ContainsKey([string]$target)) { throw "Requirement '$($requirement.id)' references unknown target '$target'." }
        }
        foreach ($lane in @($requirement.validation_lanes)) {
            if (-not $lanes.ContainsKey([string]$lane)) { throw "Requirement '$($requirement.id)' references unknown lane '$lane'." }
        }
    }
    foreach ($edge in @($Contract.controlled_artifact_edges)) {
        if (-not $targets.ContainsKey([string]$edge.reconciliation_target)) {
            throw "Fanout edge '$($edge.id)' references unknown reconciliation target '$($edge.reconciliation_target)'."
        }
        $projectionPaths = @($edge.projections | ForEach-Object { [string]$_.path })
        if ($projectionPaths -ccontains [string]$edge.producer) { throw "Fanout edge '$($edge.id)' projects to its producer." }
    }
    foreach ($slice in @($Contract.implementation_slices)) {
        foreach ($target in @($slice.exit_targets)) {
            if (-not $targets.ContainsKey([string]$target) -or [string]$targets[[string]$target].slice -cne [string]$slice.id) {
                throw "Slice '$($slice.id)' has an invalid exit target '$target'."
            }
        }
        foreach ($edge in @($slice.fanout_edges)) {
            if (-not $edges.ContainsKey([string]$edge)) { throw "Slice '$($slice.id)' references unknown fanout edge '$edge'." }
        }
    }
    foreach ($target in @($Contract.implementation_targets)) {
        if (-not (@($slices[[string]$target.slice].exit_targets) -ccontains [string]$target.id)) {
            throw "Target '$($target.id)' is not an exit target of its owning slice."
        }
    }

    foreach ($visual in @($Contract.visual_contracts)) {
        if ([string]$visual.fixture_content -ceq "mutable-owner-content") {
            if ([string]$visual.comparison -cne "stable-regions" -or @($visual.stable_regions).Count -eq 0 -or
                @($visual.semantic_assertion_targets).Count -eq 0) {
                throw "Visual contract '$($visual.id)' uses mutable fixture content without stable regions and semantic assertions."
            }
        }
        if ([string]$visual.comparison -ceq "whole-frame" -and [string]$visual.fixture_content -cne "invariant") {
            throw "Visual contract '$($visual.id)' may use whole-frame comparison only with invariant fixture content."
        }
        foreach ($target in @($visual.semantic_assertion_targets)) {
            if (-not $targets.ContainsKey([string]$target)) { throw "Visual contract '$($visual.id)' references unknown semantic target '$target'." }
        }
    }

    if ([string]$Contract.implementation_profile.kind -ceq "phase8-module-extraction") {
        $auth = [string]$Contract.implementation_profile.authorization_target
        $gate = [string]$Contract.implementation_profile.ui_ownership_gate.target
        if (-not $targets.ContainsKey($auth) -or -not $targets.ContainsKey($gate)) {
            throw "Phase 8 authorization and UI ownership targets must be declared implementation targets."
        }
        foreach ($target in @($Contract.implementation_targets | Where-Object {
                    @($_.proof_classes) | Where-Object { $_ -in @("deployed-smoke", "fixture-acceptance", "uat-readiness") }
                })) {
            if ([string]$target.id -cne $auth -and -not (Test-TessaraGuardrailDependency $targets ([string]$target.id) $auth)) {
                throw "Target '$($target.id)' must depend on the early authorization target '$auth'."
            }
        }
        foreach ($cutover in @($Contract.implementation_profile.ui_ownership_gate.consumer_cutover_targets)) {
            if (-not $targets.ContainsKey([string]$cutover) -or -not (Test-TessaraGuardrailDependency $targets ([string]$cutover) $gate)) {
                throw "Consumer cutover target '$cutover' must depend on the independent UI ownership target '$gate'."
            }
        }
    }
    return $true
}

function Assert-TessaraAuthorizationMatrixSemantics {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Matrix)
    $expectedBoundaries = @("bootstrap", "configuration", "consumer", "diagnostics", "gateway", "private", "provider", "public")
    if ((@($Matrix.boundaries | Sort-Object) -join "`n") -cne ($expectedBoundaries -join "`n")) {
        throw "Authorization matrix must declare the exact eight Phase 8 boundaries."
    }
    $covered = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($Matrix.entries)) {
        foreach ($boundary in @($entry.boundaries)) { $null = $covered.Add([string]$boundary) }
        $cases = @($entry.cases | Sort-Object)
        $expected = @("negative", "nondisclosure", "outage", "positive", "replay", "revision")
        if (($cases -join "`n") -cne ($expected -join "`n")) { throw "Authorization entry '$($entry.id)' lacks the exact required cases." }
    }
    foreach ($boundary in $expectedBoundaries) {
        if (-not $covered.Contains($boundary)) { throw "Authorization matrix does not exercise '$boundary'." }
    }
    return $true
}

function Assert-TessaraControlledArtifactFanout {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [AllowEmptyCollection()][string[]]$ChangedPaths = @()
    )
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $changed = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($path in $ChangedPaths) { $null = $changed.Add(([string]$path).Replace('\', '/')) }
    $known = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $reconciled = [Collections.Generic.List[string]]::new()
    foreach ($edge in @($Contract.controlled_artifact_edges)) {
        $producer = ([string]$edge.producer).Replace('\', '/')
        $null = $known.Add($producer)
        foreach ($projection in @($edge.projections)) { $null = $known.Add(([string]$projection.path).Replace('\', '/')) }
        $producerChanged = $changed.Contains($producer)
        $producerFull = Join-Path $root $producer
        if (Test-Path -LiteralPath $producerFull -PathType Leaf) {
            $producerChanged = $producerChanged -or ((Get-FileHash -Algorithm SHA256 -LiteralPath $producerFull).Hash.ToLowerInvariant() -cne [string]$edge.producer_baseline_sha256)
        }
        if (-not $producerChanged) { continue }
        foreach ($projection in @($edge.projections)) {
            $projectionPath = ([string]$projection.path).Replace('\', '/')
            $full = Join-Path $root $projectionPath
            if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { throw "Controlled projection '$projectionPath' is missing for changed producer '$producer'." }
            $current = (Get-FileHash -Algorithm SHA256 -LiteralPath $full).Hash.ToLowerInvariant()
            if ($current -ceq [string]$projection.baseline_sha256) {
                throw "Controlled projection '$projectionPath' is stale for changed producer '$producer'."
            }
        }
        $reconciled.Add([string]$edge.id)
    }
    $unknown = @($ChangedPaths | ForEach-Object { ([string]$_).Replace('\', '/') } | Where-Object { -not $known.Contains($_) } | Sort-Object -Unique)
    [pscustomobject][ordered]@{
        reconciled_edges = @($reconciled | Sort-Object)
        unknown_paths = $unknown
        expanded_verification_targets = if ($unknown.Count -gt 0) { @($Contract.implementation_targets.id | Sort-Object) } else { @() }
    }
}

function Assert-TessaraImplementationReadinessV2Semantics {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$ContractPath,
        [Parameter(Mandatory)][string]$AdapterPath
    )
    $contractSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $ContractPath).Hash.ToLowerInvariant()
    $adapterSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $AdapterPath).Hash.ToLowerInvariant()
    if ([string]$Result.validation_contract.sha256 -cne $contractSha -or [string]$Result.validation_adapter.sha256 -cne $adapterSha) {
        throw "Implementation readiness does not bind the current contract and adapter hashes."
    }
    if ([string]$Result.validation_adapter.path -cne [string]$Contract.validation_platform.adapter_path) {
        throw "Implementation readiness does not bind the canonical adapter path."
    }
    if ([string]$Result.platform_identity.release_version -cne [string]$Contract.validation_platform.supported_release) {
        throw "Implementation readiness uses an unsupported platform release."
    }
    $platformCommand = Get-Command Get-TessaraValidationPlatformIdentity -CommandType Function -ErrorAction SilentlyContinue
    if ($null -eq $platformCommand) { throw "Implementation readiness requires the loaded validation-platform identity API." }
    $identity = & $platformCommand
    if ([string]$Result.platform_identity.platform_fingerprint -cne [string]$identity.platform_fingerprint) { throw "Implementation readiness platform provenance cannot be authenticated." }
    if ([bool]$Result.source_identity.dirty -or [string]$Result.state -cne "passed" -or [int]$Result.known_failure_count -ne 0) {
        throw "Implementation readiness cannot pass for dirty, failed, or known-failing source."
    }
    $contractFull = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $ContractPath).Path)
    $repositoryRoot = (& git -C (Split-Path -Parent $contractFull) rev-parse --show-toplevel).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($repositoryRoot)) {
        throw "Implementation readiness could not resolve its repository root."
    }
    $harvestPath = Read-TessaraGuardrailReference $repositoryRoot $Result.harvested_defects "Harvested defect batch"
    $harvest = Get-Content -Raw -LiteralPath $harvestPath | ConvertFrom-Json -Depth 100
    if ([string]$harvest.contract -cne "tessara.validation.implementation-defect-batch" -or
        [string]$harvest.policy_version -cne "tessara-validation-v3" -or
        [string]$harvest.sprint -cne [string]$Contract.sprint -or
        [string]$harvest.validation_contract.sha256 -cne $contractSha -or
        [string]$harvest.validation_adapter.sha256 -cne $adapterSha -or
        [string]$harvest.platform_identity.platform_fingerprint -cne [string]$identity.platform_fingerprint) {
        throw "Harvested defect batch does not bind the current contract, adapter, and platform."
    }
    Assert-TessaraJsonSchema -Document $harvest -Kind implementation_defect_batch `
        -Label "Harvested implementation defect batch"
    $finalizationPath = Read-TessaraGuardrailReference $repositoryRoot `
        $Result.coordinator_finalization "Implementation coordinator finalization"
    $finalization = Get-Content -Raw -LiteralPath $finalizationPath | ConvertFrom-Json -Depth 100
    Assert-TessaraJsonSchema -Document $finalization -Kind implementation_coordinator_finalization `
        -Label "Implementation coordinator finalization"
    if ([string]$finalization.state -cne "passed" -or
        [int]$finalization.failed_count -ne 0 -or [int]$finalization.blocked_count -ne 0 -or
        [int]$finalization.open_defect_count -ne 0 -or
        [string]$finalization.validation_contract.sha256 -cne $contractSha -or
        [string]$finalization.validation_adapter.sha256 -cne $adapterSha -or
        [string]$finalization.platform_identity.platform_fingerprint -cne [string]$identity.platform_fingerprint -or
        [string]$finalization.schedule_digest -cne [string]$harvest.schedule_digest -or
        [string]$finalization.defect_batch.sha256 -cne [string]$Result.harvested_defects.sha256) {
        throw "Implementation coordinator finalization is failed, stale, blocked, or unauthenticated."
    }
    $targets = Get-TessaraGuardrailMap @($Result.targets) id "Implementation readiness targets"
    if ($targets.Count -ne @($Contract.implementation_targets).Count) { throw "Implementation readiness target inventory is not exact." }
    foreach ($target in @($Contract.implementation_targets)) {
        if (-not $targets.ContainsKey([string]$target.id)) { throw "Implementation readiness omitted target '$($target.id)'." }
        $receipt = $targets[[string]$target.id]
        if ([string]$receipt.state -cne "passed" -or
            [string]$receipt.source_identity.commit -cne [string]$Result.source_identity.commit -or
            [string]$receipt.source_identity.tree -cne [string]$Result.source_identity.tree -or
            [string]$receipt.validation_contract_sha256 -cne $contractSha -or
            [string]$receipt.adapter_sha256 -cne $adapterSha) {
            throw "Implementation target '$($target.id)' receipt is failed, stale, or bound to older source/contract."
        }
        $targetReceiptPath = Read-TessaraGuardrailReference $repositoryRoot $receipt.evidence "Implementation target '$($target.id)'"
        $targetReceipt = Get-Content -Raw -LiteralPath $targetReceiptPath | ConvertFrom-Json -Depth 100
        Assert-TessaraJsonSchema -Document $targetReceipt -Kind implementation_target_completion `
            -Label "Implementation target '$($target.id)' completion"
        if ([string]$targetReceipt.target -cne [string]$target.id -or
            [string]$targetReceipt.state -cne "passed" -or
            [string]$targetReceipt.schedule_digest -cne [string]$finalization.schedule_digest -or
            ([string]$targetReceipt.disposition -ceq "reused" -and [bool]$targetReceipt.newly_executed) -or
            ([string]$targetReceipt.disposition -ceq "executed" -and -not [bool]$targetReceipt.newly_executed)) {
            throw "Implementation target '$($target.id)' completion misstates execution or reuse provenance."
        }
    }
    $fanout = Get-TessaraGuardrailMap @($Result.fanout) edge "Fanout readiness results"
    if ($fanout.Count -ne @($Contract.controlled_artifact_edges).Count) { throw "Implementation readiness fanout inventory is not exact." }
    foreach ($edge in @($Contract.controlled_artifact_edges)) {
        if (-not $fanout.ContainsKey([string]$edge.id) -or [string]$fanout[[string]$edge.id].state -cne "passed") {
            throw "Implementation readiness lacks passing fanout proof '$($edge.id)'."
        }
        $null = Read-TessaraGuardrailReference $repositoryRoot $fanout[[string]$edge.id].receipt "Fanout '$($edge.id)'"
    }
    $slices = Get-TessaraGuardrailMap @($Result.slices) id "Implementation readiness slices"
    if ($slices.Count -ne @($Contract.implementation_slices).Count) { throw "Implementation readiness slice inventory is not exact." }
    foreach ($slice in @($Contract.implementation_slices)) {
        if (-not $slices.ContainsKey([string]$slice.id) -or [string]$slices[[string]$slice.id].state -cne "passed" -or
            ((@($slices[[string]$slice.id].exit_targets | Sort-Object) -join "`n") -cne (@($slice.exit_targets | Sort-Object) -join "`n"))) {
            throw "Implementation slice '$($slice.id)' is incomplete or has stale exit coverage."
        }
        if ((@($slices[[string]$slice.id].fanout_edges | Sort-Object) -join "`n") -cne
            (@($slice.fanout_edges | Sort-Object) -join "`n")) {
            throw "Implementation slice '$($slice.id)' has stale fanout coverage."
        }
    }
    return $true
}

function Assert-TessaraCloseoutEfficiencySemantics {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Report)
    if ([int]$Report.attempt_count -lt [int]$Report.target_count) {
        throw "Closeout attempt count cannot be lower than target count."
    }
    if ([int]$Report.first_pass_pass_count -gt [int]$Report.target_count -or
        [Math]::Abs(([double]$Report.first_pass_pass_count / [double]$Report.target_count) - [double]$Report.first_pass_pass_rate) -gt 0.000000001) {
        throw "Closeout first-pass pass rate does not match its target counts."
    }
    $null = Get-TessaraGuardrailMap @($Report.findings_by_classification) name "Closeout finding classifications"
    $null = Get-TessaraGuardrailMap @($Report.findings_by_target) name "Closeout finding targets"
    return $true
}

Export-ModuleMember -Function @(
    "Assert-TessaraValidationContractV3Semantics",
    "Assert-TessaraAuthorizationMatrixSemantics",
    "Assert-TessaraControlledArtifactFanout",
    "Assert-TessaraImplementationReadinessV2Semantics",
    "Assert-TessaraCloseoutEfficiencySemantics"
)

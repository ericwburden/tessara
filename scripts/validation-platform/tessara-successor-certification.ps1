Set-StrictMode -Version Latest

function Get-TessaraSuccessorMap {
    param([object[]]$Items, [string]$Property, [string]$Label)
    $map = @{}
    foreach ($item in @($Items)) {
        $key = [string]$item.$Property
        if ([string]::IsNullOrWhiteSpace($key) -or $map.ContainsKey($key)) {
            throw "$Label contains a missing or duplicate '$key'."
        }
        $map[$key] = $item
    }
    $map
}

function Get-TessaraSuccessorDomainPatterns {
    param($Domain)
    if ($Domain.PSObject.Properties.Name -contains "inputs") {
        return @($Domain.inputs | ForEach-Object { [string]$_.path } | Sort-Object -Unique)
    }
    @($Domain.tracked_inputs | ForEach-Object { [string]$_ } | Sort-Object -Unique)
}

function Get-TessaraSuccessorPrerequisiteClosure {
    param([hashtable]$Lanes, [string]$LaneId)
    $result = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $pending = [Collections.Generic.Queue[string]]::new()
    foreach ($id in @($Lanes[$LaneId].prerequisites)) { $pending.Enqueue([string]$id) }
    while ($pending.Count -gt 0) {
        $id = $pending.Dequeue()
        if (-not $result.Add($id)) { continue }
        foreach ($next in @($Lanes[$id].prerequisites)) { $pending.Enqueue([string]$next) }
    }
    @($result | Sort-Object)
}

function Get-TessaraSuccessorAffectedClosure {
    param([hashtable]$Lanes, [string[]]$DirectIds)
    $selected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $pending = [Collections.Generic.Queue[string]]::new()
    foreach ($id in @($DirectIds)) {
        if ($Lanes.ContainsKey($id) -and $selected.Add($id)) { $pending.Enqueue($id) }
    }
    while ($pending.Count -gt 0) {
        $id = $pending.Dequeue()
        foreach ($prerequisite in @($Lanes[$id].prerequisites)) {
            $name = [string]$prerequisite
            if ($selected.Add($name)) { $pending.Enqueue($name) }
        }
    }
    @($selected | Sort-Object)
}

function Get-TessaraSuccessorPlanIdentity {
    param($CompatibilityPlan)
    if ([string]$CompatibilityPlan.contract -cne "tessara.validation.compatibility-plan") {
        throw "Successor planning requires authenticated compatibility-plan documents."
    }
    $body = $CompatibilityPlan.body
    $sourceFingerprint = if ($body.PSObject.Properties.Name -contains "source_fingerprint") {
        [string]$body.source_fingerprint
    } else {
        Get-TessaraPlatformCanonicalJsonSha256 -Value $body.source_identity
    }
    $dependencyMap = @{}
    foreach ($fingerprint in @($body.lanes.dependency_fingerprints)) {
        $domain = [string]$fingerprint.domain
        if ($dependencyMap.ContainsKey($domain) -and
            [string]$dependencyMap[$domain] -cne [string]$fingerprint.sha256) {
            throw "Compatibility plan contains conflicting '$domain' fingerprints."
        }
        $dependencyMap[$domain] = [string]$fingerprint.sha256
    }
    [pscustomobject][ordered]@{
        source_fingerprint = $sourceFingerprint
        candidate_fingerprint = [string]$body.candidate_fingerprint
        compatibility_plan_fingerprint = [string]$CompatibilityPlan.fingerprint
        dependency_fingerprints = @($dependencyMap.Keys | Sort-Object | ForEach-Object {
                [pscustomobject][ordered]@{ domain = $_; sha256 = $dependencyMap[$_] }
            })
    }
}

function New-TessaraSuccessorImpactPlan {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$PredecessorCompatibilityPlan,
        [Parameter(Mandatory)]$SuccessorCompatibilityPlan,
        [Parameter(Mandatory)]
        [ValidateSet(
            "human-execution-mistake", "evidence-publication-defect",
            "phase-local-runner-defect", "candidate-product-bounded",
            "unknown-shared-or-unauthenticated"
        )]
        [string]$CorrectionClass,
        [Parameter(Mandatory)][ValidatePattern("^[0-9a-f]{64}$")][string]$CorrectionBatchDiffSha256,
        [AllowEmptyCollection()][string[]]$ChangedPaths = @(),
        [AllowEmptyCollection()][string[]]$AffectedItemIds = @(),
        [AllowEmptyCollection()][string[]]$PreviouslyFailedItemIds = @(),
        [datetimeoffset]$GeneratedAt = [datetimeoffset]::UtcNow
    )
    $null = Assert-TessaraValidationContract -Contract $Contract
    if ([int]$Contract.schema_version -ne 3 -or
        [string]$Contract.policy_version -cne "tessara-validation-v3") {
        throw "Successor impact selection is available only to validation policy v3."
    }
    $domains = Get-TessaraSuccessorMap @($Contract.dependency_domains) name "Dependency domains"
    $lanes = Get-TessaraSuccessorMap @($Contract.lanes) id "Validation lanes"
    $targets = Get-TessaraSuccessorMap @($Contract.implementation_targets) id "Implementation targets"
    $predecessorLanes = Get-TessaraSuccessorMap @($PredecessorCompatibilityPlan.body.lanes) lane_id "Predecessor lanes"
    $successorLanes = Get-TessaraSuccessorMap @($SuccessorCompatibilityPlan.body.lanes) lane_id "Successor lanes"
    $predecessorIdentity = Get-TessaraSuccessorPlanIdentity $PredecessorCompatibilityPlan
    $successorIdentity = Get-TessaraSuccessorPlanIdentity $SuccessorCompatibilityPlan

    $changed = [Collections.Generic.List[object]]::new()
    $unknown = [Collections.Generic.List[string]]::new()
    $changedDomains = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($rawPath in @($ChangedPaths | Sort-Object -Unique)) {
        $path = ([string]$rawPath).Replace('\', '/').Trim()
        $matches = @($Contract.dependency_domains | Where-Object {
                $patterns = @(Get-TessaraSuccessorDomainPatterns $_)
                @($patterns | Where-Object { $path -clike ([string]$_).Replace('\', '/') }).Count -gt 0
            } | ForEach-Object { [string]$_.name } | Sort-Object -Unique)
        if ($matches.Count -eq 0) { $unknown.Add($path) }
        foreach ($name in $matches) { $null = $changedDomains.Add($name) }
        $changed.Add([pscustomobject][ordered]@{ path = $path; domains = $matches })
    }

    $fallbackReasons = [Collections.Generic.List[string]]::new()
    if ($unknown.Count -gt 0) { $fallbackReasons.Add("At least one changed path has no authenticated dependency-domain mapping.") }
    foreach ($name in @($changedDomains)) {
        $domainClass = [string]$domains[$name].class
        $classScopedException =
            ($CorrectionClass -ceq "evidence-publication-defect" -and $domainClass -ceq "evidence-publication") -or
            ($CorrectionClass -ceq "phase-local-runner-defect" -and $domainClass -in @("phase-runner", "evidence-publication"))
        if ([string]$domains[$name].default_impact -ceq "full-replay" -and
            -not $classScopedException) {
            $fallbackReasons.Add("Dependency domain '$name' defaults to complete replay because no finer closed cone is authenticated.")
        }
    }
    if ($CorrectionClass -ceq "unknown-shared-or-unauthenticated") {
        $fallbackReasons.Add("The correction was classified as unknown, shared-risk, or unauthenticated.")
    }

    $sameCandidate = [string]$predecessorIdentity.candidate_fingerprint -ceq
        [string]$successorIdentity.candidate_fingerprint
    switch ($CorrectionClass) {
        "human-execution-mistake" {
            if (-not $sameCandidate -or [string]$predecessorIdentity.source_fingerprint -cne
                [string]$successorIdentity.source_fingerprint -or $ChangedPaths.Count -ne 0) {
                $fallbackReasons.Add("A human-execution classification cannot authenticate unchanged source and candidate identity.")
            }
            foreach ($id in $AffectedItemIds) {
                if (-not $lanes.ContainsKey($id) -or
                    [string]$lanes[$id].coverage_kind -cne "manual-scenario") {
                    $fallbackReasons.Add("Human-execution repair item '$id' is not a declared manual scenario.")
                }
            }
        }
        "evidence-publication-defect" {
            if (-not $sameCandidate -or @($changedDomains | Where-Object {
                        [string]$domains[[string]$_].class -cne "evidence-publication"
                    }).Count -gt 0) {
                $fallbackReasons.Add("Evidence-finalization-only reuse could not authenticate an unchanged candidate and evidence-publication-only diff.")
            }
        }
        "phase-local-runner-defect" {
            if (-not $sameCandidate -or @($changedDomains | Where-Object {
                        [string]$domains[[string]$_].class -notin @("phase-runner", "evidence-publication")
                    }).Count -gt 0) {
                $fallbackReasons.Add("Runner-local reuse could not authenticate an unchanged candidate and phase-local runner-only diff.")
            }
        }
        "candidate-product-bounded" {
            if ($sameCandidate) { $fallbackReasons.Add("A candidate-changing product correction did not produce a successor candidate fingerprint.") }
            if ($changedDomains.Count -eq 0) { $fallbackReasons.Add("A bounded product correction has no authenticated changed domain.") }
        }
    }

    $fullReplay = $fallbackReasons.Count -gt 0
    $directLaneIds = @()
    if ($CorrectionClass -ceq "human-execution-mistake" -and -not $fullReplay) {
        $directLaneIds = @($AffectedItemIds | Sort-Object -Unique)
    } elseif ($CorrectionClass -ne "evidence-publication-defect") {
        $directLaneIds = @($Contract.lanes | Where-Object {
                @($_.dependency_domains | Where-Object { $changedDomains.Contains([string]$_) }).Count -gt 0
            } | ForEach-Object { [string]$_.id } | Sort-Object -Unique)
    }
    $selectedLaneIds = if ($fullReplay) {
        @($Contract.lanes | Where-Object { [string]$_.phase -ne "implementation" } |
            ForEach-Object { [string]$_.id } | Sort-Object -Unique)
    } elseif ($CorrectionClass -ceq "evidence-publication-defect") {
        @()
    } elseif ($CorrectionClass -ceq "human-execution-mistake") {
        @($directLaneIds)
    } else {
        @(Get-TessaraSuccessorAffectedClosure -Lanes $lanes -DirectIds $directLaneIds)
    }
    if (-not $fullReplay -and $CorrectionClass -ceq "candidate-product-bounded") {
        $selectedLaneIds = @($selectedLaneIds + @($Contract.lanes | Where-Object {
                    [string]$_.phase -ceq "validation-preflight"
                } | ForEach-Object { [string]$_.id }) | Sort-Object -Unique)
    }

    $affectedTargetIds = @(if ($fullReplay) {
        @($targets.Keys | Sort-Object)
    } else {
        @($Contract.implementation_targets | Where-Object {
                @($_.dependency_domains | Where-Object { $changedDomains.Contains([string]$_) }).Count -gt 0
            } | ForEach-Object { [string]$_.id } | Sort-Object -Unique)
    })
    $coverage = [Collections.Generic.List[object]]::new()
    foreach ($lane in @($Contract.lanes)) {
        $id = [string]$lane.id
        $phase = [string]$lane.phase
        $execute = $id -in $selectedLaneIds -and $phase -ne "implementation"
        $disposition = if ($phase -ceq "implementation") { "not-required" } elseif ($execute) { "execute" } else { "inherit" }
        $reason = if ($phase -ceq "implementation") { "implementation-only" } elseif ($id -in $PreviouslyFailedItemIds -and $execute) {
            "previously-failed"
        } elseif ($id -in $directLaneIds -and $execute) { "direct-impact" } elseif ($execute -and $fullReplay) {
            "conservative-fallback"
        } elseif ($execute) { "dependency-closure" } elseif ($sameCandidate) {
            "same-candidate-retained"
        } else { "authenticated-nonimpact" }
        $prior = if ($predecessorLanes.ContainsKey($id)) { $predecessorLanes[$id] } else { $null }
        $current = if ($successorLanes.ContainsKey($id)) { $successorLanes[$id] } else { $null }
        if ($null -eq $prior -or $null -eq $current) {
            if (-not $fullReplay) { throw "Impact selection requires exact predecessor/successor lane parity; '$id' is missing." }
            $zero = "0" * 64
            $priorCompatibility = $zero; $priorInheritance = $zero
            $currentCompatibility = if ($null -eq $current) { $zero } else { [string]$current.compatibility_fingerprint }
            $currentInheritance = if ($null -eq $current) { $zero } else { [string]$current.inheritance_fingerprint }
            $dependencies = @([pscustomobject][ordered]@{ domain = "unknown"; sha256 = $zero })
        } else {
            $priorCompatibility = [string]$prior.compatibility_fingerprint
            $currentCompatibility = [string]$current.compatibility_fingerprint
            $priorInheritance = [string]$prior.inheritance_fingerprint
            $currentInheritance = [string]$current.inheritance_fingerprint
            $dependencies = @($current.dependency_fingerprints | Sort-Object domain)
        }
        $nonImpact = if ($disposition -ceq "inherit") {
            "No changed domain intersects '$id'; its complete dependency, fixture, acceptance, environment, runner, platform, and recursive prerequisite inheritance fingerprint is unchanged."
        } else { $null }
        $coverage.Add([pscustomobject][ordered]@{
            id = $id
            phase = $phase
            coverage_kind = [string]$lane.coverage_kind
            disposition = $disposition
            selection_reason = $reason
            risk_rank = [int]$lane.risk_rank
            dependency_domains = @($lane.dependency_domains | Sort-Object)
            prerequisite_closure = @(Get-TessaraSuccessorPrerequisiteClosure -Lanes $lanes -LaneId $id)
            predecessor_compatibility_fingerprint = $priorCompatibility
            successor_compatibility_fingerprint = $currentCompatibility
            predecessor_inheritance_fingerprint = $priorInheritance
            successor_inheritance_fingerprint = $currentInheritance
            dependency_fingerprints = $dependencies
            non_impact_rationale = $nonImpact
        })
    }
    $reasonOrder = @{
        "previously-failed" = 0; "direct-impact" = 1; "dependency-closure" = 2
        "conservative-fallback" = 3; "same-candidate-retained" = 4
        "authenticated-nonimpact" = 5; "implementation-only" = 6
    }
    $orderedCoverage = @($coverage | Sort-Object `
        @{ Expression = { $reasonOrder[[string]$_.selection_reason] } },
        @{ Expression = { [int]$_.risk_rank } },
        @{ Expression = { [string]$_.phase } },
        @{ Expression = { [string]$_.id } })
    $mode = if ($fullReplay) { "full-replay" } else {
        switch ($CorrectionClass) {
            "human-execution-mistake" { "scenario-only" }
            "evidence-publication-defect" { "finalization-only" }
            default { "impact-selected" }
        }
    }
    $planBody = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.successor-impact-plan"
        policy_version = "tessara-validation-v3"
        sprint = [string]$Contract.sprint
        correction_class = $CorrectionClass
        certification_mode = $mode
        predecessor = $predecessorIdentity
        successor = $successorIdentity
        correction_batch = [pscustomobject][ordered]@{
            diff_sha256 = $CorrectionBatchDiffSha256
            changed_paths = @($changed)
            changed_domains = @($changedDomains | Sort-Object)
            unknown_paths = @($unknown | Sort-Object)
        }
        affected_implementation_targets = $affectedTargetIds
        affected_readiness_lanes = @($orderedCoverage | Where-Object { $_.phase -ceq "validation-readiness" -and $_.disposition -ceq "execute" } | ForEach-Object id)
        affected_rehearsal_lanes = @($orderedCoverage | Where-Object { $_.phase -ceq "candidate-rehearsal" -and $_.disposition -ceq "execute" } | ForEach-Object id)
        affected_preflight_lanes = @($orderedCoverage | Where-Object { $_.phase -ceq "validation-preflight" -and $_.disposition -ceq "execute" } | ForEach-Object id)
        affected_sit_lanes = @($orderedCoverage | Where-Object { $_.phase -ceq "sit" -and $_.disposition -ceq "execute" } | ForEach-Object id)
        affected_uat_scripted_scenarios = @($orderedCoverage | Where-Object { $_.phase -ceq "uat" -and $_.coverage_kind -ceq "scripted-scenario" -and $_.disposition -ceq "execute" } | ForEach-Object id)
        affected_uat_manual_scenarios = @($orderedCoverage | Where-Object { $_.phase -ceq "uat" -and $_.coverage_kind -ceq "manual-scenario" -and $_.disposition -ceq "execute" } | ForEach-Object id)
        coverage = $orderedCoverage
        cleanup_restoration = [pscustomobject][ordered]@{
            required = @($orderedCoverage | Where-Object { $_.disposition -ceq "execute" }).Count -gt 0
            scopes = @($orderedCoverage | Where-Object { $_.disposition -ceq "execute" -and [bool]$lanes[[string]$_.id].touches_live_state } | ForEach-Object id | Sort-Object -Unique)
        }
        conservative_fallback_reasons = @($fallbackReasons | Sort-Object -Unique)
        open_defect_count = 0
        expectation_changes_authenticated = $true
        generated_at = $GeneratedAt.ToUniversalTime().ToString("o")
    }
    $planBody.plan_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value ([pscustomobject]$planBody)
    $plan = [pscustomobject]$planBody
    $null = Assert-TessaraSuccessorImpactPlan -Plan $plan -Contract $Contract
    $plan
}

function Assert-TessaraSuccessorImpactPlan {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Plan,
        [Parameter(Mandatory)]$Contract
    )
    $null = Assert-TessaraValidationContract -Contract $Contract
    Assert-TessaraJsonSchema -Document $Plan -Kind successor_impact_plan `
        -Label "Successor impact plan"
    if ([string]$Plan.sprint -cne [string]$Contract.sprint) {
        throw "Successor impact plan sprint does not match its validation contract."
    }
    $copy = [ordered]@{}
    foreach ($property in $Plan.PSObject.Properties | Where-Object { $_.Name -cne "plan_fingerprint" }) {
        $copy[$property.Name] = $property.Value
    }
    if ([string]$Plan.plan_fingerprint -cne
        (Get-TessaraPlatformCanonicalJsonSha256 -Value ([pscustomobject]$copy))) {
        throw "Successor impact plan fingerprint does not authenticate its contents."
    }
    $contractLanes = Get-TessaraSuccessorMap @($Contract.lanes) id "Validation lanes"
    $coverage = Get-TessaraSuccessorMap @($Plan.coverage) id "Successor coverage"
    if ($coverage.Count -ne $contractLanes.Count) {
        throw "Successor impact plan does not cover the exact contract lane inventory."
    }
    $reasonOrder = @{
        "previously-failed" = 0; "direct-impact" = 1; "dependency-closure" = 2
        "conservative-fallback" = 3; "same-candidate-retained" = 4
        "authenticated-nonimpact" = 5; "implementation-only" = 6
    }
    $priorKey = $null
    foreach ($item in @($Plan.coverage)) {
        if (-not $contractLanes.ContainsKey([string]$item.id) -or
            [string]$contractLanes[[string]$item.id].phase -cne [string]$item.phase -or
            [string]$contractLanes[[string]$item.id].coverage_kind -cne [string]$item.coverage_kind) {
            throw "Successor coverage item '$($item.id)' does not match the contract lane."
        }
        $key = "{0:D2}/{1:D4}/{2}/{3}" -f $reasonOrder[[string]$item.selection_reason], [int]$item.risk_rank, [string]$item.phase, [string]$item.id
        if ($null -ne $priorKey -and [string]::CompareOrdinal($priorKey, $key) -gt 0) {
            throw "Successor impact coverage is not in deterministic failed/direct/closure/risk order."
        }
        $priorKey = $key
        if ([string]$item.disposition -ceq "inherit") {
            if ([string]::IsNullOrWhiteSpace([string]$item.non_impact_rationale) -or
                [string]$item.predecessor_inheritance_fingerprint -cne
                    [string]$item.successor_inheritance_fingerprint) {
                throw "Inherited item '$($item.id)' lacks unchanged authenticated inheritance compatibility and non-impact rationale."
            }
            foreach ($domain in @($item.dependency_domains)) {
                $prior = @($Plan.predecessor.dependency_fingerprints | Where-Object { [string]$_.domain -ceq [string]$domain })
                $current = @($Plan.successor.dependency_fingerprints | Where-Object { [string]$_.domain -ceq [string]$domain })
                if ($prior.Count -ne 1 -or $current.Count -ne 1 -or
                    [string]$prior[0].sha256 -cne [string]$current[0].sha256) {
                    throw "Inherited item '$($item.id)' has changed or missing '$domain' dependency evidence."
                }
            }
        }
    }
    if (@($Plan.correction_batch.unknown_paths).Count -gt 0 -and
        [string]$Plan.certification_mode -cne "full-replay") {
        throw "Unknown correction paths require complete replay."
    }
    if ([string]$Plan.certification_mode -ceq "full-replay") {
        if (@($Plan.coverage | Where-Object {
                    [string]$_.phase -ne "implementation" -and [string]$_.disposition -cne "execute"
                }).Count -gt 0 -or @($Plan.conservative_fallback_reasons).Count -eq 0) {
            throw "Conservative fallback must execute every formal lane and record its reasons."
        }
    } elseif ([int]$Plan.open_defect_count -ne 0 -or -not [bool]$Plan.expectation_changes_authenticated) {
        throw "Impact-selected certification cannot inherit with open defects or unauthenticated expectation changes."
    }
    if ([string]$Plan.correction_class -ceq "candidate-product-bounded" -and
        [string]$Plan.certification_mode -ne "full-replay" -and
        [string]$Plan.predecessor.candidate_fingerprint -ceq [string]$Plan.successor.candidate_fingerprint) {
        throw "Bounded product correction requires a distinct successor candidate."
    }
    if ([string]$Plan.correction_class -in @("human-execution-mistake", "evidence-publication-defect", "phase-local-runner-defect") -and
        [string]$Plan.certification_mode -ne "full-replay" -and
        [string]$Plan.predecessor.candidate_fingerprint -cne [string]$Plan.successor.candidate_fingerprint) {
        throw "Non-product correction classification cannot cross candidate fingerprints."
    }
    return $true
}

function Assert-TessaraSuccessorPredecessorCertificate {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$ImpactPlan,
        [Parameter(Mandatory)]$PriorCertificate,
        [Parameter(Mandatory)][string]$Phase
    )
    $candidateBound = $Phase -in @("validation-preflight", "sit", "uat")
    if ([int]$PriorCertificate.schema_version -ne 3 -or
        [string]$PriorCertificate.policy_version -cne "tessara-validation-v3" -or
        [string]$PriorCertificate.phase -cne $Phase -or
        [string]$PriorCertificate.state -cne "passed" -or
        [int]$PriorCertificate.open_defect_count -ne 0 -or
        ($candidateBound -and [string]$PriorCertificate.candidate_fingerprint -cne
            [string]$ImpactPlan.predecessor.candidate_fingerprint)) {
        throw "Prior phase certificate is not the immediate authenticated predecessor selected by the successor impact plan."
    }
    $priorLanes = Get-TessaraSuccessorMap @($PriorCertificate.lanes) name `
        "Prior phase certificate lanes"
    foreach ($item in @($ImpactPlan.coverage | Where-Object {
                [string]$_.phase -ceq $Phase -and [string]$_.disposition -ceq "inherit"
            })) {
        if (-not $priorLanes.ContainsKey([string]$item.id) -or
            [string]$priorLanes[[string]$item.id].state -cne "passed") {
            throw "Immediate predecessor certificate does not own passing inherited item '$($item.id)'."
        }
    }
    return $true
}

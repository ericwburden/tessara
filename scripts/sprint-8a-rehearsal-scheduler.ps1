Set-StrictMode -Version Latest

$script:Sprint8AMaxConsecutiveDeferrals = 3
$script:Sprint8ADiagnosticHistoryNotice =
    "Prior evidence is diagnostic history only; it is not reused evidence or authoritative proof for this attempt."

function Get-Sprint8ARehearsalJsonSha256 {
    param([Parameter(Mandatory)]$Document)

    $json = $Document | ConvertTo-Json -Depth 100 -Compress
    $bytes = [Text.Encoding]::UTF8.GetBytes($json)
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        ([Convert]::ToHexString($sha.ComputeHash($bytes))).ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

function Get-Sprint8ARehearsalLanePolicies {
    function New-Policy {
        param(
            [Parameter(Mandatory)][string]$Name,
            [string[]]$DependsOn = @(),
            [Parameter(Mandatory)][string]$Role,
            [string[]]$ImpactPaths = @(),
            [string[]]$ImpactSources = @()
        )
        [pscustomobject][ordered]@{
            name = $Name
            depends_on = @($DependsOn)
            scheduler_role = $Role
            impact_paths = @($ImpactPaths)
            impact_sources = @($ImpactSources)
        }
    }

    @(
        New-Policy -Name "attempt-state-prerequisite" -Role "lifecycle"
        New-Policy -Name "validation-readiness-prerequisite" -Role "lifecycle"
        New-Policy -Name "formatting" -Role "ordinary" -ImpactPaths @("Cargo.*", ".cargo/*", "crates/*", "build.rs")
        New-Policy -Name "workspace-check" -Role "ordinary" -ImpactPaths @("Cargo.*", ".cargo/*", "crates/*", "build.rs") -ImpactSources @("deployment_inputs")
        New-Policy -Name "workspace-clippy" -Role "ordinary" -ImpactPaths @("Cargo.*", ".cargo/*", "crates/*", "build.rs") -ImpactSources @("deployment_inputs")
        New-Policy -Name "compose-manifest-schema-contract" -Role "ordinary" -ImpactPaths @("deploy/*", "Dockerfile*", "crates/*/manifest.json", "scripts/*", "docs/sprints/sprint-8a-*", ".codex/skills/tessara-*/*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "web-native-wasm-source-boundaries" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/*", "scripts/check-web-crate-boundaries.ps1") -ImpactSources @("deployment_inputs")
        New-Policy -Name "module-sdk-boundaries" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-module-*/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "scripts/verify-module-sdk-*.ps1") -ImpactSources @("deployment_inputs")
        New-Policy -Name "dashboard-source-boundaries" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-dashboard-*/*", "crates/tessara-components-contract/*", "scripts/verify-sprint-6e-boundaries.ps1") -ImpactSources @("deployment_inputs")
        New-Policy -Name "markdown-links" -Role "ordinary" -ImpactPaths @("*.md", "docs/*", ".codex/skills/*") -ImpactSources @("acceptance_inventory")
        New-Policy -Name "workspace-tests" -Role "ordinary" -ImpactPaths @("Cargo.*", ".cargo/*", "crates/*", "build.rs", "scripts/sprint-8a-validation-environment.ps1") -ImpactSources @("deployment_inputs", "environment_contract")
        New-Policy -Name "optimized-resource-reference-timing" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-api/*", "crates/tessara-module-contract/*") -ImpactSources @("environment_contract")
        New-Policy -Name "components-contract-tests" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-components-contract/*")
        New-Policy -Name "dashboard-module-tests" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-dashboard-*/*", "crates/tessara-components-contract/*")
        New-Policy -Name "component-conformance-nondisclosure" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-component-*/*", "crates/tessara-components-contract/*", "crates/tessara-datasets-contract/*") -ImpactSources @("environment_contract")
        New-Policy -Name "module-testkit-conformance" -Role "ordinary" -ImpactPaths @("Cargo.*", "crates/tessara-module-contract/*", "crates/tessara-module-testkit/*")
        New-Policy -Name "playwright-discovery" -Role "ordinary" -ImpactPaths @("end2end/*", "scripts/validate-e2e.ps1", "scripts/sprint-8a-acceptance-contract.ps1", "docs/sprints/sprint-8a-uat/*") -ImpactSources @("acceptance_inventory")
        New-Policy -Name "source-exact-materialization-no-op" -DependsOn @("attempt-state-prerequisite", "validation-readiness-prerequisite") -Role "ordinary" -ImpactPaths @("Cargo.*", "Dockerfile*", "deploy/*", "crates/tessara-composition/*", "crates/tessara-supervisor/*", "crates/tessara-api/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "crates/tessara-reference-*/*", "scripts/bootstrap-sprint-7a-composition.ps1", "scripts/materialize-sprint-8a.ps1", "scripts/sprint-8a-validation-environment.ps1") -ImpactSources @("deployment_inputs", "environment_contract")
        New-Policy -Name "deployed-inventory-navigation-audit" -DependsOn @("source-exact-materialization-no-op") -Role "ordinary" -ImpactPaths @("scripts/audit-sprint-8a-deployed-inventory.ps1", "crates/tessara-api/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "deploy/*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "deployment-evidence" -DependsOn @("source-exact-materialization-no-op") -Role "ordinary" -ImpactPaths @("scripts/run-sprint-8a-deployed-smoke.ps1", "scripts/capture-sprint-6a-deployment-evidence.ps1", "deploy/*", "Dockerfile*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "product-smoke" -DependsOn @("source-exact-materialization-no-op") -Role "ordinary" -ImpactPaths @("scripts/smoke-sprint-8a.ps1", "crates/tessara-api/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "deploy/*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "playwright-execution" -DependsOn @("source-exact-materialization-no-op", "deployment-evidence") -Role "ordinary" -ImpactPaths @("end2end/*", "scripts/validate-e2e.ps1", "crates/tessara-web/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "deploy/*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "component-upgrade-rollback" -DependsOn @("source-exact-materialization-no-op") -Role "ordinary" -ImpactPaths @("Dockerfile.component", "deploy/*", "crates/tessara-component-*/*", "scripts/*sprint-8a-component*") -ImpactSources @("deployment_inputs", "environment_contract")
        New-Policy -Name "failure-containment-successor-health" -DependsOn @("attempt-state-prerequisite", "validation-readiness-prerequisite") -Role "cleanup" -ImpactPaths @("Dockerfile*", "deploy/*", "crates/tessara-supervisor/*", "crates/tessara-composition/*", "crates/tessara-api/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "scripts/bootstrap-sprint-7a-composition.ps1", "scripts/materialize-sprint-8a.ps1", "scripts/run-sprint-8a-failure-containment.ps1") -ImpactSources @("deployment_inputs", "environment_contract")
        New-Policy -Name "successor-inventory-navigation-audit" -DependsOn @("failure-containment-successor-health") -Role "ordinary" -ImpactPaths @("scripts/audit-sprint-8a-deployed-inventory.ps1", "crates/tessara-api/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "deploy/*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "successor-deployment-evidence" -DependsOn @("failure-containment-successor-health") -Role "ordinary" -ImpactPaths @("scripts/run-sprint-8a-deployed-smoke.ps1", "scripts/capture-sprint-6a-deployment-evidence.ps1", "deploy/*", "Dockerfile*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "successor-product-smoke" -DependsOn @("failure-containment-successor-health") -Role "ordinary" -ImpactPaths @("scripts/smoke-sprint-8a.ps1", "crates/tessara-api/*", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*", "deploy/*") -ImpactSources @("acceptance_inventory", "deployment_inputs", "environment_contract")
        New-Policy -Name "live-product-diagnostics" -DependsOn @("deployment-evidence", "product-smoke") -Role "ordinary" -ImpactPaths @("scripts/diagnose-sprint-8a-product.ps1", "scripts/diagnose-sprint-8a-dashboard-dependencies.ps1", "crates/tessara-component-*/*", "crates/tessara-dashboard-*/*") -ImpactSources @("acceptance_inventory", "environment_contract")
        New-Policy -Name "uat-diagnostics" -DependsOn @("source-exact-materialization-no-op", "successor-inventory-navigation-audit", "successor-deployment-evidence", "successor-product-smoke", "failure-containment-successor-health", "component-upgrade-rollback", "components-contract-tests", "component-conformance-nondisclosure", "playwright-execution", "compose-manifest-schema-contract", "web-native-wasm-source-boundaries", "dashboard-source-boundaries", "live-product-diagnostics") -Role "aggregate_sink" -ImpactPaths @("scripts/uat-sprint-8a.ps1", "docs/sprints/sprint-8a-uat/*", "scripts/sprint-8a-acceptance-contract.ps1") -ImpactSources @("acceptance_inventory", "environment_contract")
        New-Policy -Name "final-successor-health" -DependsOn @("failure-containment-successor-health") -Role "cleanup_sink" -ImpactPaths @("deploy/*", "scripts/materialize-sprint-8a.ps1", "scripts/run-sprint-8a-failure-containment.ps1") -ImpactSources @("deployment_inputs", "environment_contract")
        New-Policy -Name "final-clean-source" -Role "safety_finalizer" -ImpactPaths @("*") -ImpactSources @("acceptance_inventory", "deployment_inputs")
        New-Policy -Name "final-environment-identity" -Role "safety_finalizer" -ImpactPaths @("deploy/*", "scripts/sprint-8a-validation-environment.ps1") -ImpactSources @("environment_contract", "deployment_inputs")
    )
}

function Test-Sprint8ARehearsalPathPattern {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Pattern
    )

    $normalizedPath = $Path.Replace("\", "/")
    $normalizedPattern = $Pattern.Replace("\", "/")
    $normalizedPath -like $normalizedPattern
}

function Test-Sprint8ARehearsalDeclarationMember {
    param(
        [Parameter(Mandatory)]$Declaration,
        [Parameter(Mandatory)][string]$Name
    )

    if ($Declaration -is [Collections.IDictionary]) {
        return ([Collections.IDictionary]$Declaration).Contains($Name)
    }
    return $Declaration.PSObject.Properties.Name -contains $Name
}

function Assert-Sprint8ARehearsalSchedulerDeclarations {
    param([Parameter(Mandatory)][object[]]$Checks)

    $allowedRoles = @("lifecycle", "ordinary", "cleanup", "safety_finalizer", "aggregate_sink", "cleanup_sink")
    $allowedImpactSources = @("acceptance_inventory", "deployment_inputs", "environment_contract")
    $names = @($Checks | ForEach-Object { [string]$_.name })
    if ($names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count) {
        throw "Candidate Rehearsal scheduler requires nonempty, unique lane names."
    }

    foreach ($check in $Checks) {
        if (-not (Test-Sprint8ARehearsalDeclarationMember -Declaration $check -Name "scheduler_role") -or
            $allowedRoles -cnotcontains [string]$check.scheduler_role) {
            throw "Candidate Rehearsal lane '$($check.name)' has an unsupported scheduler role."
        }
        if (-not (Test-Sprint8ARehearsalDeclarationMember -Declaration $check -Name "impact_paths") -or
            -not (Test-Sprint8ARehearsalDeclarationMember -Declaration $check -Name "impact_sources")) {
            throw "Candidate Rehearsal lane '$($check.name)' omits its correction-impact contract."
        }
        $impactSources = @($check.impact_sources | ForEach-Object { [string]$_ })
        if (@($impactSources | Sort-Object -Unique).Count -ne $impactSources.Count -or
            @($impactSources | Where-Object { $allowedImpactSources -cnotcontains $_ }).Count -gt 0) {
            throw "Candidate Rehearsal lane '$($check.name)' has an invalid impact source."
        }
        if ([string]$check.scheduler_role -in @("aggregate_sink", "cleanup_sink") -and
            @($check.depends_on).Count -eq 0) {
            throw "Candidate Rehearsal aggregate sink '$($check.name)' must declare its real prerequisites."
        }
        foreach ($dependency in @($check.depends_on)) {
            if ([string]$dependency -ceq [string]$check.name -or
                $names -cnotcontains [string]$dependency) {
                throw "Candidate Rehearsal lane '$($check.name)' has invalid scheduler dependency '$dependency'."
            }
        }
    }
    $visiting = @{}
    $visited = @{}
    function Visit-Sprint8ARehearsalSchedulerLane([string]$Name) {
        if ($visiting.ContainsKey($Name)) {
            throw "Candidate Rehearsal scheduler graph contains a cycle through '$Name'."
        }
        if ($visited.ContainsKey($Name)) { return }
        $visiting[$Name] = $true
        $check = @($Checks | Where-Object name -CEQ $Name)[0]
        foreach ($dependency in @($check.depends_on)) {
            Visit-Sprint8ARehearsalSchedulerLane -Name ([string]$dependency)
        }
        [void]$visiting.Remove($Name)
        $visited[$Name] = $true
    }
    foreach ($name in $names) { Visit-Sprint8ARehearsalSchedulerLane -Name $name }
}

function Get-Sprint8ARehearsalDirectImpactDecision {
    param(
        [Parameter(Mandatory)]$Check,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ChangedPaths,
        [Parameter(Mandatory)][bool]$AcceptanceInventoryChanged,
        [Parameter(Mandatory)][bool]$DeploymentInputsChanged,
        [Parameter(Mandatory)][bool]$EnvironmentContractChanged
    )

    $matchedPaths = @($ChangedPaths | Where-Object {
        $candidate = [string]$_
        @($Check.impact_paths | Where-Object {
            Test-Sprint8ARehearsalPathPattern -Path $candidate -Pattern ([string]$_)
        }).Count -gt 0
    } | Sort-Object -Unique)
    $matchedSources = [Collections.Generic.List[string]]::new()
    $declaredSources = @($Check.impact_sources | ForEach-Object { [string]$_ })
    if ($AcceptanceInventoryChanged -and $declaredSources -ccontains "acceptance_inventory") {
        $matchedSources.Add("acceptance_inventory")
    }
    if ($DeploymentInputsChanged -and $declaredSources -ccontains "deployment_inputs") {
        $matchedSources.Add("deployment_inputs")
    }
    if ($EnvironmentContractChanged -and $declaredSources -ccontains "environment_contract") {
        $matchedSources.Add("environment_contract")
    }
    $globalHarnessPatterns = @(
        "scripts/run-sprint-8a-candidate-rehearsal.ps1",
        "scripts/sprint-8a-rehearsal-scheduler.ps1",
        "scripts/validate-sprint-8a-readiness.ps1",
        "scripts/test-sprint-validation-harvest.ps1",
        "scripts/run-sprint-8a-validation-preflight.ps1",
        "scripts/sprint-8a-validation-environment.ps1",
        "scripts/fixtures/*",
        ".codex/skills/tessara-sprint-validation/*"
    )
    $globalHarnessPaths = @($ChangedPaths | Where-Object {
        $candidate = [string]$_
        @($globalHarnessPatterns | Where-Object {
            Test-Sprint8ARehearsalPathPattern -Path $candidate -Pattern ([string]$_)
        }).Count -gt 0
    } | Sort-Object -Unique)
    $affected = $matchedPaths.Count -gt 0 -or $matchedSources.Count -gt 0 -or $globalHarnessPaths.Count -gt 0
    [pscustomobject][ordered]@{
        affected = $affected
        decision = if ($affected) { "inside_correction_impact" } else { "outside_correction_impact" }
        matched_paths = $matchedPaths
        global_harness_paths = $globalHarnessPaths
        matched_identity_sources = @($matchedSources)
        rationale = if ($affected) {
            "The current correction changes a declared lane input, validation harness, fixture, or identity source."
        } else {
            "No authenticated changed path or identity source intersects this lane's declared impact contract."
        }
    }
}

function Get-Sprint8ARehearsalTopologicalOrder {
    param(
        [Parameter(Mandatory)][object[]]$Checks,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Names,
        [Parameter(Mandatory)]$Decisions
    )

    $remaining = [Collections.Generic.List[string]]::new()
    foreach ($name in $Names) { $remaining.Add([string]$name) }
    $ordered = [Collections.Generic.List[string]]::new()
    $indices = @{}
    for ($index = 0; $index -lt $Checks.Count; $index++) {
        $indices[[string]$Checks[$index].name] = $index
    }
    while ($remaining.Count -gt 0) {
        $eligible = @($remaining | Where-Object {
            $name = [string]$_
            $check = @($Checks | Where-Object name -CEQ $name)[0]
            @($check.depends_on | Where-Object {
                $dependency = [string]$_
                $Names -ccontains $dependency -and $ordered -cnotcontains $dependency
            }).Count -eq 0
        } | Sort-Object `
            @{ Expression = { [int]$Decisions[[string]$_].priority } },
            @{ Expression = { [int]$indices[[string]$_] } })
        if ($eligible.Count -eq 0) {
            throw "Candidate Rehearsal scheduler cannot produce a dependency-safe deterministic order."
        }
        $selected = [string]$eligible[0]
        $ordered.Add($selected)
        [void]$remaining.Remove($selected)
    }
    @($ordered)
}

function Resolve-Sprint8ARehearsalSchedule {
    param(
        [Parameter(Mandatory)][object[]]$Checks,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)]$LaneHistory,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ChangedPaths,
        [Parameter(Mandatory)][bool]$AcceptanceInventoryChanged,
        [Parameter(Mandatory)][bool]$DeploymentInputsChanged,
        [Parameter(Mandatory)][bool]$EnvironmentContractChanged,
        [Parameter(Mandatory)][bool]$HistoryAuthenticated,
        [AllowNull()][string]$FallbackReason
    )

    Assert-Sprint8ARehearsalSchedulerDeclarations -Checks $Checks
    $decisions = [ordered]@{}
    foreach ($check in $Checks) {
        $name = [string]$check.name
        $role = [string]$check.scheduler_role
        $impact = Get-Sprint8ARehearsalDirectImpactDecision `
            -Check $check `
            -ChangedPaths $ChangedPaths `
            -AcceptanceInventoryChanged $AcceptanceInventoryChanged `
            -DeploymentInputsChanged $DeploymentInputsChanged `
            -EnvironmentContractChanged $EnvironmentContractChanged
        $history = if ($LaneHistory.Contains($name)) { $LaneHistory[$name] } else { $null }
        $priorState = if ($null -eq $history) { $null } else { [string]$history.preceding_state }
        $priorDeferrals = if ($null -eq $history) { 0 } else { [int]$history.consecutive_deferrals }
        $hasPriorPass = $null -ne $history -and $null -ne $history.prior_passing_receipt

        $wave = "B"
        $segment = "wave_b"
        $reason = "prior_passing_outside_impact"
        $priority = 80
        if ($role -eq "aggregate_sink") {
            $wave = "sink"
            $segment = "aggregate_sinks"
            $reason = "aggregate_sink_waits_for_current_attempt_prerequisites"
            $priority = 100
        } elseif ($role -eq "cleanup_sink") {
            $wave = "A"
            $segment = "cleanup_sinks"
            $reason = "mandatory_cleanup_sink"
            $priority = 70
        } elseif ($role -eq "safety_finalizer") {
            $wave = "A"
            $segment = "finalizers"
            $reason = "mandatory_safety_finalizer"
            $priority = 90
        } elseif (-not $HistoryAuthenticated) {
            $wave = "A"
            $segment = "wave_a"
            $reason = "conservative_full_harvest_fallback"
            $priority = 60
        } elseif ($role -eq "lifecycle") {
            $wave = "A"
            $segment = "wave_a"
            $reason = "mandatory_lifecycle_prerequisite"
            $priority = 0
        } elseif ($role -eq "cleanup") {
            $wave = "A"
            $segment = "wave_a"
            $reason = "mandatory_cleanup_and_restoration"
            $priority = 50
        } elseif ($priorState -ceq "failed") {
            $wave = "A"
            $segment = "wave_a"
            $reason = "failed_in_preceding_rehearsal"
            $priority = 10
        } elseif ($priorState -ceq "blocked") {
            $wave = "A"
            $segment = "wave_a"
            $reason = "blocked_or_newly_reachable_in_preceding_rehearsal"
            $priority = 20
        } elseif ([bool]$impact.affected) {
            $wave = "A"
            $segment = "wave_a"
            $reason = "inside_current_correction_impact"
            $priority = 30
        } elseif ($priorDeferrals -ge $script:Sprint8AMaxConsecutiveDeferrals) {
            $wave = "A"
            $segment = "wave_a"
            $reason = "maximum_consecutive_deferrals_reached"
            $priority = 40
        } elseif ($null -eq $history -or (-not [bool]$history.ever_executed -and -not $hasPriorPass)) {
            $wave = "A"
            $segment = "wave_a"
            $reason = "never_executed_or_newly_reachable"
            $priority = 20
        } elseif (-not $hasPriorPass) {
            $wave = "A"
            $segment = "wave_a"
            $reason = "no_authenticated_prior_passing_receipt"
            $priority = 20
        }
        $decisions[$name] = [pscustomobject][ordered]@{
            name = $name
            wave = $wave
            segment = $segment
            reason = $reason
            priority = $priority
            preceding_state = $priorState
            consecutive_deferrals_before = $priorDeferrals
            ever_executed_before = if ($null -eq $history) { $false } else { [bool]$history.ever_executed }
            prior_passing_receipt = if ($hasPriorPass) { $history.prior_passing_receipt } else { $null }
            prior_source_identity = if ($null -eq $history) { $null } else { $history.prior_source_identity }
            prior_environment_fingerprint = if ($null -eq $history) { $null } else { [string]$history.prior_environment_fingerprint }
            mandatory_by_attempt = if ($priorDeferrals -gt 0) {
                $Attempt + ($script:Sprint8AMaxConsecutiveDeferrals - $priorDeferrals)
            } else { $null }
            current_correction_impact = $impact
        }
    }

    if ($HistoryAuthenticated) {
        $changed = $true
        while ($changed) {
            $changed = $false
            foreach ($check in $Checks) {
                $name = [string]$check.name
                $decision = $decisions[$name]
                if ([string]$decision.segment -in @("aggregate_sinks", "cleanup_sinks", "finalizers")) { continue }
                $affectedDependencies = @($check.depends_on | Where-Object {
                    $dependencyDecision = $decisions[[string]$_]
                    [bool]$dependencyDecision.current_correction_impact.affected -or
                        [string]$dependencyDecision.reason -eq "failed_in_preceding_rehearsal"
                })
                if ($affectedDependencies.Count -gt 0 -and [string]$decision.wave -cne "A") {
                    $decision.wave = "A"
                    $decision.segment = "wave_a"
                    $decision.reason = "relevant_prerequisite_changed"
                    $decision.priority = 35
                    $decision.current_correction_impact.affected = $true
                    $decision.current_correction_impact.decision = "inside_correction_impact"
                    $decision.current_correction_impact.rationale =
                        "A relevant prerequisite failed previously or is inside the current correction impact: $($affectedDependencies -join ', ')."
                    $changed = $true
                }
            }
        }

        $closureChanged = $true
        while ($closureChanged) {
            $closureChanged = $false
            foreach ($check in $Checks) {
                $decision = $decisions[[string]$check.name]
                if ([string]$decision.segment -cne "wave_a") { continue }
                foreach ($dependency in @($check.depends_on)) {
                    $dependencyDecision = $decisions[[string]$dependency]
                    if ([string]$dependencyDecision.segment -eq "wave_b") {
                        $dependencyDecision.wave = "A"
                        $dependencyDecision.segment = "wave_a"
                        $dependencyDecision.reason = "wave_a_prerequisite_closure"
                        $dependencyDecision.priority = 55
                        $closureChanged = $true
                    }
                }
            }
        }
    }

    $waveANames = @($Checks | Where-Object {
        [string]$decisions[[string]$_.name].segment -ceq "wave_a"
    } | ForEach-Object { [string]$_.name })
    $waveBNames = @($Checks | Where-Object {
        [string]$decisions[[string]$_.name].segment -ceq "wave_b"
    } | ForEach-Object { [string]$_.name })
    $finalizers = @($Checks | Where-Object {
        [string]$decisions[[string]$_.name].segment -ceq "finalizers"
    } | ForEach-Object { [string]$_.name })
    $aggregateSinks = @($Checks | Where-Object {
        [string]$decisions[[string]$_.name].segment -ceq "aggregate_sinks"
    } | ForEach-Object { [string]$_.name })
    $cleanupSinks = @($Checks | Where-Object {
        [string]$decisions[[string]$_.name].segment -ceq "cleanup_sinks"
    } | ForEach-Object { [string]$_.name })

    $waveA = @(Get-Sprint8ARehearsalTopologicalOrder -Checks $Checks -Names $waveANames -Decisions $decisions)
    $waveB = @(Get-Sprint8ARehearsalTopologicalOrder -Checks $Checks -Names $waveBNames -Decisions $decisions)
    [pscustomobject][ordered]@{
        schema_version = 1
        strategy = "bounded_failure_first_two_wave"
        attempt = $Attempt
        max_consecutive_deferrals = $script:Sprint8AMaxConsecutiveDeferrals
        history_authentication = [ordered]@{
            authenticated = $HistoryAuthenticated
            fallback = -not $HistoryAuthenticated
            fallback_reason = if ($HistoryAuthenticated) { $null } else { $FallbackReason }
        }
        changed_paths = @($ChangedPaths | Sort-Object -Unique)
        identity_changes = [ordered]@{
            acceptance_inventory = $AcceptanceInventoryChanged
            deployment_inputs = $DeploymentInputsChanged
            environment_contract = $EnvironmentContractChanged
        }
        wave_a = $waveA
        cleanup_sinks = $cleanupSinks
        wave_b = $waveB
        aggregate_sinks = $aggregateSinks
        finalizers = $finalizers
        decisions = @($Checks | ForEach-Object { $decisions[[string]$_.name] })
    }
}

function Assert-Sprint8ARehearsalScheduleContract {
    param(
        [Parameter(Mandatory)]$Schedule,
        [Parameter(Mandatory)][object[]]$Checks,
        [Parameter(Mandatory)][int]$ExpectedAttempt
    )

    Assert-Sprint8ARehearsalSchedulerDeclarations -Checks $Checks
    if (($Schedule.schema_version -isnot [int] -and $Schedule.schema_version -isnot [long]) -or
        [int]$Schedule.schema_version -ne 1 -or
        [string]$Schedule.strategy -cne "bounded_failure_first_two_wave" -or
        ($Schedule.attempt -isnot [int] -and $Schedule.attempt -isnot [long]) -or
        [int]$Schedule.attempt -ne $ExpectedAttempt -or
        ($Schedule.max_consecutive_deferrals -isnot [int] -and $Schedule.max_consecutive_deferrals -isnot [long]) -or
        [int]$Schedule.max_consecutive_deferrals -ne $script:Sprint8AMaxConsecutiveDeferrals) {
        throw "Candidate Rehearsal schedule has an invalid identity or bounded-deferral contract."
    }
    foreach ($property in @("wave_a", "cleanup_sinks", "wave_b", "aggregate_sinks", "finalizers", "decisions")) {
        if ($Schedule.PSObject.Properties.Name -notcontains $property) {
            throw "Candidate Rehearsal schedule omits '$property'."
        }
    }
    $declaredNames = @($Checks | ForEach-Object { [string]$_.name })
    $segments = @(
        @($Schedule.wave_a), @($Schedule.cleanup_sinks), @($Schedule.wave_b),
        @($Schedule.aggregate_sinks), @($Schedule.finalizers)
    )
    $scheduledNames = @($segments | ForEach-Object { @($_) } | ForEach-Object { [string]$_ })
    if ($scheduledNames.Count -ne $declaredNames.Count -or
        @($scheduledNames | Sort-Object -Unique).Count -ne $scheduledNames.Count -or
        (($scheduledNames | Sort-Object) -join "`n") -cne (($declaredNames | Sort-Object) -join "`n")) {
        throw "Candidate Rehearsal schedule does not cover every declared lane exactly once."
    }
    $decisions = @($Schedule.decisions)
    if ($decisions.Count -ne $declaredNames.Count -or
        @($decisions.name | ForEach-Object { [string]$_ } | Sort-Object -Unique).Count -ne $declaredNames.Count) {
        throw "Candidate Rehearsal schedule decisions do not identify every declared lane exactly once."
    }
    $segmentProperties = [ordered]@{
        wave_a = @($Schedule.wave_a)
        cleanup_sinks = @($Schedule.cleanup_sinks)
        wave_b = @($Schedule.wave_b)
        aggregate_sinks = @($Schedule.aggregate_sinks)
        finalizers = @($Schedule.finalizers)
    }
    foreach ($segmentName in $segmentProperties.Keys) {
        foreach ($name in @($segmentProperties[$segmentName])) {
            $decision = @($decisions | Where-Object name -CEQ ([string]$name))
            if ($decision.Count -ne 1 -or [string]$decision[0].segment -cne [string]$segmentName) {
                throw "Candidate Rehearsal lane '$name' is inconsistent with its schedule decision."
            }
        }
    }
    foreach ($name in @($Schedule.wave_b)) {
        $decision = @($decisions | Where-Object name -CEQ ([string]$name))[0]
        if ([bool]$decision.current_correction_impact.affected -or
            [int]$decision.consecutive_deferrals_before -ge $script:Sprint8AMaxConsecutiveDeferrals -or
            @("passed", "deferred") -cnotcontains [string]$decision.preceding_state -or
            [string]$decision.prior_passing_receipt.path -notmatch '\.json$' -or
            [string]$decision.prior_passing_receipt.sha256 -notmatch '^[0-9a-f]{64}$' -or
            [string]$decision.prior_source_identity.commit -notmatch '^[0-9a-f]{40}$' -or
            [string]$decision.prior_environment_fingerprint -notmatch '^[0-9a-f]{64}$') {
            throw "Candidate Rehearsal Wave B lane '$name' is not an authenticated prior pass outside the correction impact cone."
        }
    }
    if (-not [bool]$Schedule.history_authentication.authenticated -and @($Schedule.wave_b).Count -ne 0) {
        throw "Unauthenticated Candidate Rehearsal history must select conservative full harvest with no Wave B lanes."
    }
    foreach ($segmentName in @("wave_a", "cleanup_sinks", "wave_b")) {
        $ordered = @($segmentProperties[$segmentName])
        for ($index = 0; $index -lt $ordered.Count; $index++) {
            $name = [string]$ordered[$index]
            $check = @($Checks | Where-Object name -CEQ $name)[0]
            foreach ($dependency in @($check.depends_on)) {
                if ($ordered -ccontains [string]$dependency -and
                    [Array]::IndexOf([object[]]$ordered, [string]$dependency) -gt $index) {
                    throw "Candidate Rehearsal segment '$segmentName' orders '$name' before prerequisite '$dependency'."
                }
            }
        }
    }
    $Schedule
}

function Get-Sprint8ARehearsalWaveBDisposition {
    param(
        [Parameter(Mandatory)]$Schedule,
        [Parameter(Mandatory)]$TerminalByName
    )

    $waveAAndCleanup = @(@($Schedule.wave_a) + @($Schedule.cleanup_sinks))
    $unterminated = @($waveAAndCleanup | Where-Object {
        -not $TerminalByName.ContainsKey([string]$_) -or
            @("passed", "failed", "blocked") -cnotcontains [string]$TerminalByName[[string]$_].state
    })
    if ($unterminated.Count -gt 0) {
        throw "Candidate Rehearsal cannot decide Wave B before every Wave A and cleanup-sink lane is terminal: $($unterminated -join ', ')."
    }
    if (@($waveAAndCleanup | Where-Object {
        [string]$TerminalByName[[string]$_].state -cne "passed"
    }).Count -gt 0) { "defer" } else { "execute" }
}

function Assert-Sprint8ARehearsalRecoveryScheduleBinding {
    param(
        [Parameter(Mandatory)]$StartDocument,
        [Parameter(Mandatory)]$AttemptDocument,
        [Parameter(Mandatory)][object[]]$Checks,
        [Parameter(Mandatory)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$ExpectedStartPath,
        [Parameter(Mandatory)][string]$ExpectedStartSha256
    )

    $computedScheduleSha = Get-Sprint8ARehearsalJsonSha256 -Document $StartDocument.schedule
    if (($StartDocument.schema_version -isnot [int] -and $StartDocument.schema_version -isnot [long]) -or
        [int]$StartDocument.schema_version -ne 3 -or
        [string]$StartDocument.phase -cne "candidate-rehearsal-start" -or
        [int]$StartDocument.attempt -ne $ExpectedAttempt -or
        [string]$StartDocument.schedule_sha256 -cne $computedScheduleSha -or
        ($AttemptDocument.schema_version -isnot [int] -and $AttemptDocument.schema_version -isnot [long]) -or
        [int]$AttemptDocument.schema_version -ne 3 -or
        [string]$AttemptDocument.phase -cne "candidate-rehearsal" -or
        [int]$AttemptDocument.attempt -ne $ExpectedAttempt -or
        @("preparing", "executing", "harvesting") -cnotcontains [string]$AttemptDocument.state -or
        [string]$AttemptDocument.schedule_sha256 -cne $computedScheduleSha -or
        [string]$AttemptDocument.immutable_start_receipt.path -cne $ExpectedStartPath -or
        [string]$AttemptDocument.immutable_start_receipt.sha256 -cne $ExpectedStartSha256) {
        throw "Candidate Rehearsal recovery rejected changed immutable ordering, deferral counters, or attempt binding."
    }
    [void](Assert-Sprint8ARehearsalScheduleContract `
        -Schedule $StartDocument.schedule -Checks $Checks -ExpectedAttempt $ExpectedAttempt)
    $StartDocument.schedule
}

function Resolve-Sprint8ARehearsalEvidenceReference {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$Reference
    )

    if ([string]$Reference.path -notmatch '\.json$' -or
        [string]$Reference.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Candidate Rehearsal history reference is malformed."
    }
    $fullPath = if ([IO.Path]::IsPathRooted([string]$Reference.path)) {
        [IO.Path]::GetFullPath([string]$Reference.path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot ([string]$Reference.path)))
    }
    $repoPrefix = [IO.Path]::GetFullPath($RepositoryRoot).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    $evidencePrefix = [IO.Path]::GetFullPath($EvidenceRoot).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if (-not $fullPath.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase) -and
        -not $fullPath.StartsWith($evidencePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Candidate Rehearsal history reference escapes the repository/evidence roots."
    }
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath "$fullPath.sha256" -PathType Leaf)) {
        throw "Candidate Rehearsal history reference is missing its receipt or sidecar."
    }
    $actual = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $sidecar = (Get-Content -LiteralPath "$fullPath.sha256" -Raw).Trim().ToLowerInvariant()
    if ($actual -cne [string]$Reference.sha256 -or $sidecar -cne $actual) {
        throw "Candidate Rehearsal history reference SHA-256 authentication failed."
    }
    [pscustomobject][ordered]@{
        path = [string]$Reference.path
        sha256 = $actual
        full_path = $fullPath
        document = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json
    }
}

function Get-Sprint8AAuthenticatedRehearsalHistory {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][object[]]$Checks,
        [Parameter(Mandatory)]$PriorAttemptReference
    )

    $attemptRef = Resolve-Sprint8ARehearsalEvidenceReference `
        -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Reference $PriorAttemptReference
    $attempt = $attemptRef.document
    if (($attempt.schema_version -isnot [int] -and $attempt.schema_version -isnot [long]) -or
        [int]$attempt.schema_version -ne 3 -or
        [string]$attempt.sprint -cne "sprint-8a" -or
        [string]$attempt.phase -cne "candidate-rehearsal" -or
        @("passed", "failed") -cnotcontains [string]$attempt.state -or
        $attempt.PSObject.Properties.Name -notcontains "immutable_start_receipt") {
        throw "Preceding Candidate Rehearsal is not a terminal schema-3 attempt with an immutable start receipt."
    }
    $startRef = Resolve-Sprint8ARehearsalEvidenceReference `
        -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Reference $attempt.immutable_start_receipt
    $start = $startRef.document
    if (($start.schema_version -isnot [int] -and $start.schema_version -isnot [long]) -or
        [int]$start.schema_version -ne 3 -or
        [string]$start.phase -cne "candidate-rehearsal-start" -or
        [int]$start.attempt -ne [int]$attempt.attempt -or
        [string]$start.schedule_sha256 -cne (Get-Sprint8ARehearsalJsonSha256 -Document $start.schedule) -or
        [string]$attempt.schedule_sha256 -cne [string]$start.schedule_sha256) {
        throw "Preceding Candidate Rehearsal immutable schedule authentication failed."
    }
    [void](Assert-Sprint8ARehearsalScheduleContract `
        -Schedule $start.schedule -Checks $Checks -ExpectedAttempt ([int]$attempt.attempt))
    $terminalChecks = @($attempt.checks)
    Assert-Sprint8ARehearsalTerminalAccounting `
        -DeclaredChecks $Checks -TerminalChecks $terminalChecks `
        -Attempt ([int]$attempt.attempt) -AttemptState ([string]$attempt.state)
    $history = @{}
    foreach ($declaration in $Checks) {
        $name = [string]$declaration.name
        $result = @($terminalChecks | Where-Object name -CEQ $name)[0]
        if ($result.PSObject.Properties.Name -notcontains "lane_receipt") {
            throw "Preceding Candidate Rehearsal lane '$name' omits its authenticated lane receipt."
        }
        $laneRef = Resolve-Sprint8ARehearsalEvidenceReference `
            -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Reference $result.lane_receipt
        $lane = $laneRef.document
        if (($lane.schema_version -isnot [int] -and $lane.schema_version -isnot [long]) -or
            [int]$lane.schema_version -notin @(1, 2) -or
            [string]$lane.phase -cne "candidate-rehearsal-lane" -or
            [int]$lane.attempt -ne [int]$attempt.attempt -or
            [string]$lane.result.name -cne $name -or
            [string]$lane.result.state -cne [string]$result.state -or
            [string]$lane.environment_fingerprint -cne [string]$attempt.environment_fingerprint -or
            ($lane.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress) -cne
                ($attempt.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress)) {
            throw "Preceding Candidate Rehearsal lane '$name' does not bind the attempt source/environment identity."
        }
        $priorPassingReceipt = $null
        $deferrals = 0
        if ([string]$result.state -ceq "passed") {
            $priorPassingReceipt = [pscustomobject][ordered]@{ path = $laneRef.path; sha256 = $laneRef.sha256 }
        } elseif ([string]$result.state -ceq "deferred") {
            $deferrals = [int]$result.consecutive_deferral_count
            $priorPassRef = Resolve-Sprint8ARehearsalEvidenceReference `
                -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Reference $result.prior_passing_receipt
            $priorPass = $priorPassRef.document
            if ([string]$priorPass.phase -cne "candidate-rehearsal-lane" -or
                [string]$priorPass.result.name -cne $name -or
                [string]$priorPass.result.state -cne "passed" -or
                [string]$priorPass.environment_fingerprint -cne [string]$result.prior_environment_identity.fingerprint -or
                ($priorPass.mutable_source_identity | ConvertTo-Json -Depth 20 -Compress) -cne
                    ($result.prior_source_identity | ConvertTo-Json -Depth 20 -Compress)) {
                throw "Deferred lane '$name' does not authenticate its diagnostic prior passing lane receipt."
            }
            $priorPassingReceipt = [pscustomobject][ordered]@{ path = $priorPassRef.path; sha256 = $priorPassRef.sha256 }
        }
        $history[$name] = [pscustomobject][ordered]@{
            preceding_state = [string]$result.state
            consecutive_deferrals = $deferrals
            ever_executed = [bool]$result.assertions_started -or [string]$result.state -in @("passed", "failed")
            prior_passing_receipt = $priorPassingReceipt
            prior_source_identity = if ([string]$result.state -ceq "deferred") {
                $result.prior_source_identity
            } else { $attempt.mutable_source_identity }
            prior_environment_fingerprint = if ([string]$result.state -ceq "deferred") {
                [string]$result.prior_environment_identity.fingerprint
            } else { [string]$attempt.environment_fingerprint }
        }
    }
    [pscustomobject][ordered]@{
        authenticated = $true
        prior_attempt = [pscustomobject][ordered]@{
            attempt = [int]$attempt.attempt
            receipt = [pscustomobject][ordered]@{ path = $attemptRef.path; sha256 = $attemptRef.sha256 }
            immutable_start_receipt = [pscustomobject][ordered]@{ path = $startRef.path; sha256 = $startRef.sha256 }
            mutable_source_identity = $attempt.mutable_source_identity
            environment_fingerprint = [string]$attempt.environment_fingerprint
        }
        lane_history = $history
    }
}

function Get-Sprint8ARehearsalChangedPaths {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$PriorCommit,
        [Parameter(Mandatory)][string]$CurrentCommit
    )

    foreach ($commit in @($PriorCommit, $CurrentCommit)) {
        & git -C $RepositoryRoot cat-file -e "$commit^{commit}" 2>$null
        if ($LASTEXITCODE -ne 0) { throw "Cannot authenticate correction source commit '$commit'." }
    }
    $paths = @(& git -C $RepositoryRoot diff --name-only --no-renames $PriorCommit $CurrentCommit --)
    if ($LASTEXITCODE -ne 0) { throw "Cannot derive the authenticated Candidate Rehearsal correction impact cone." }
    @($paths | ForEach-Object { ([string]$_).Replace("\", "/") } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Sort-Object -Unique)
}

function Test-Sprint8ARehearsalHistoryRegressionFixtures {
    $fixturePath = Join-Path $PSScriptRoot "fixtures/sprint-8a-rehearsal-history-regressions.json"
    if (-not (Test-Path -LiteralPath $fixturePath -PathType Leaf)) {
        throw "Candidate Rehearsal history regression fixture is missing."
    }
    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json
    $expectedNames = @(
        "r9-r11-materialization",
        "r19-r22-playwright-aggregate-sink",
        "r29-process-loss",
        "r31-first-use-fallback"
    )
    $actualNames = @($fixture.fixtures | ForEach-Object { [string]$_.name })
    if (($fixture.schema_version -isnot [int] -and $fixture.schema_version -isnot [long]) -or
        [int]$fixture.schema_version -ne 1 -or
        [string]$fixture.authority -cne "diagnostic_history_only" -or
        $fixture.authenticated_for_deferral -isnot [bool] -or [bool]$fixture.authenticated_for_deferral -or
        [string]$fixture.expected_scheduler_behavior -cne "conservative_full_harvest" -or
        (($actualNames | Sort-Object) -join "`n") -cne (($expectedNames | Sort-Object) -join "`n") -or
        @($fixture.fixtures | Where-Object {
            [string]$_.expected_scheduler_behavior -cne "conservative_full_harvest" -or
                [string]::IsNullOrWhiteSpace([string]$_.authentication_reason)
        }).Count -ne 0) {
        throw "Candidate Rehearsal retained-history fixture could be misused as authenticated deferral evidence."
    }
    $materialization = @($fixture.fixtures | Where-Object name -CEQ "r9-r11-materialization")[0]
    $playwright = @($fixture.fixtures | Where-Object name -CEQ "r19-r22-playwright-aggregate-sink")[0]
    if ((@($materialization.attempts.attempt | ForEach-Object { [int]$_ }) -join ",") -cne "9,10,11" -or
        @($materialization.attempts | Where-Object failed_lane -CNE "source-exact-materialization-no-op").Count -ne 0 -or
        (@($playwright.attempts.attempt | ForEach-Object { [int]$_ }) -join ",") -cne "19,20,21,22" -or
        [string]$playwright.aggregate_sink -cne "final-health-and-clean-source") {
        throw "Candidate Rehearsal retained-history regression identities drifted from R9-R11 or R19-R22."
    }
    $fallback = Resolve-Sprint8ARehearsalSchedule `
        -Checks @(Get-Sprint8ARehearsalLanePolicies) `
        -Attempt 32 `
        -LaneHistory @{} `
        -ChangedPaths @() `
        -AcceptanceInventoryChanged:$false `
        -DeploymentInputsChanged:$false `
        -EnvironmentContractChanged:$false `
        -HistoryAuthenticated:$false `
        -FallbackReason "retained regression history is diagnostic only"
    if (@($fallback.wave_b).Count -ne 0 -or -not [bool]$fallback.history_authentication.fallback) {
        throw "Candidate Rehearsal retained history bypassed conservative full-harvest fallback."
    }
}

function New-Sprint8ADeferredLaneResult {
    param(
        [Parameter(Mandatory)]$Declaration,
        [Parameter(Mandatory)]$Decision,
        [Parameter(Mandatory)]$History,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)]$TerminalByName
    )

    if ([string]$Decision.wave -cne "B" -or $null -eq $History.prior_passing_receipt -or
        [string]$History.prior_passing_receipt.path -notmatch '\.json$' -or
        [string]$History.prior_passing_receipt.sha256 -notmatch '^[0-9a-f]{64}$' -or
        [bool]$Decision.current_correction_impact.affected) {
        throw "Lane '$($Declaration.name)' is not eligible for a deferred Wave B result."
    }
    $count = [int]$History.consecutive_deferrals + 1
    if ($count -lt 1 -or $count -gt $script:Sprint8AMaxConsecutiveDeferrals) {
        throw "Lane '$($Declaration.name)' exceeded the consecutive-deferral limit."
    }
    $prerequisiteState = @($Declaration.depends_on | ForEach-Object {
        $name = [string]$_
        [ordered]@{
            name = $name
            state = if ($TerminalByName.ContainsKey($name)) { [string]$TerminalByName[$name].state } else { "not_terminal" }
        }
    })
    [pscustomobject][ordered]@{
        name = [string]$Declaration.name
        depends_on = @($Declaration.depends_on)
        command = [string]$Declaration.command
        wave = "B"
        scheduling_reason = [string]$Decision.reason
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
        prior_passing_receipt = $History.prior_passing_receipt
        prior_source_identity = $History.prior_source_identity
        prior_environment_identity = [ordered]@{ fingerprint = [string]$History.prior_environment_fingerprint }
        current_correction_impact = $Decision.current_correction_impact
        non_impact_rationale = [string]$Decision.current_correction_impact.rationale
        consecutive_deferral_count = $count
        mandatory_by_attempt = $Attempt + ($script:Sprint8AMaxConsecutiveDeferrals - $count + 1)
        prerequisite_state = $prerequisiteState
        diagnostic_history_notice = $script:Sprint8ADiagnosticHistoryNotice
        evidence_path = $null
        evidence_sha256 = $null
        produced_evidence = @()
        failure_message = $null
        nested_blocked_checks = @()
        nested_failed_checks = @()
    }
}

function Assert-Sprint8ARehearsalTerminalAccounting {
    param(
        [Parameter(Mandatory)][object[]]$DeclaredChecks,
        [Parameter(Mandatory)][object[]]$TerminalChecks,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)][string]$AttemptState
    )

    $declaredNames = @($DeclaredChecks | ForEach-Object { [string]$_.name })
    $terminalNames = @($TerminalChecks | ForEach-Object { [string]$_.name })
    if ($TerminalChecks.Count -ne $DeclaredChecks.Count -or
        @($terminalNames | Sort-Object -Unique).Count -ne $terminalNames.Count -or
        (($declaredNames | Sort-Object) -join "`n") -cne (($terminalNames | Sort-Object) -join "`n")) {
        throw "Candidate Rehearsal terminal accounting is missing or duplicates declared lanes."
    }
    foreach ($result in $TerminalChecks) {
        if (@("passed", "failed", "blocked", "deferred") -cnotcontains [string]$result.state) {
            throw "Candidate Rehearsal lane '$($result.name)' has unsupported terminal state '$($result.state)'."
        }
        if ([string]$result.state -ceq "deferred") {
            if ([bool]$result.assertions_started -or $null -ne $result.started_at -or $null -ne $result.ended_at -or
                $null -ne $result.duration_ms -or $null -ne $result.assertions_started_at -or
                [string]$result.prior_passing_receipt.sha256 -notmatch '^[0-9a-f]{64}$' -or
                [string]$result.prior_source_identity.commit -notmatch '^[0-9a-f]{40}$' -or
                [string]$result.prior_environment_identity.fingerprint -notmatch '^[0-9a-f]{64}$' -or
                [bool]$result.current_correction_impact.affected -or
                [string]::IsNullOrWhiteSpace([string]$result.non_impact_rationale) -or
                [int]$result.consecutive_deferral_count -lt 1 -or
                [int]$result.consecutive_deferral_count -gt $script:Sprint8AMaxConsecutiveDeferrals -or
                [int]$result.mandatory_by_attempt -le $Attempt -or
                [string]$result.diagnostic_history_notice -cne $script:Sprint8ADiagnosticHistoryNotice) {
                throw "Deferred lane '$($result.name)' does not satisfy the exact diagnostic-only receipt contract."
            }
        }
    }
    $deferred = @($TerminalChecks | Where-Object state -CEQ "deferred")
    if ($deferred.Count -gt 0 -and $AttemptState -ceq "passed") {
        throw "A Candidate Rehearsal with deferred lanes cannot pass or authorize preflight."
    }
    if ($AttemptState -ceq "passed" -and @($TerminalChecks | Where-Object state -CNE "passed").Count -gt 0) {
        throw "A passing Candidate Rehearsal must execute and pass every declared lane."
    }
}

function Test-Sprint8ARehearsalTwoWaveScheduler {
    $checks = @(
        [ordered]@{ name = "lock"; depends_on = @(); command = "lock"; scheduler_role = "lifecycle"; impact_paths = @(); impact_sources = @() },
        [ordered]@{ name = "prior-pass"; depends_on = @(); command = "pass"; scheduler_role = "ordinary"; impact_paths = @("src/pass/**"); impact_sources = @() },
        [ordered]@{ name = "prior-failure"; depends_on = @("lock"); command = "fail"; scheduler_role = "ordinary"; impact_paths = @("src/fail/**"); impact_sources = @() },
        [ordered]@{ name = "impacted-dependent"; depends_on = @("prior-pass"); command = "impact"; scheduler_role = "ordinary"; impact_paths = @("src/dependent/**"); impact_sources = @() },
        [ordered]@{ name = "cleanup"; depends_on = @("lock"); command = "cleanup"; scheduler_role = "cleanup"; impact_paths = @(); impact_sources = @("environment_contract") },
        [ordered]@{ name = "cleanup-sink"; depends_on = @("cleanup"); command = "cleanup sink"; scheduler_role = "cleanup_sink"; impact_paths = @(); impact_sources = @("environment_contract") },
        [ordered]@{ name = "sink"; depends_on = @("prior-failure", "prior-pass"); command = "sink"; scheduler_role = "aggregate_sink"; impact_paths = @("src/sink/**"); impact_sources = @() }
    )
    if (@($checks | Where-Object { $_ -isnot [Collections.Specialized.OrderedDictionary] }).Count -gt 0) {
        throw "Candidate Rehearsal scheduler self-test must exercise live ordered-dictionary declarations."
    }
    $genericDictionary = [Collections.Generic.Dictionary[string, object]]::new()
    $genericDictionary["scheduler_role"] = "ordinary"
    if (-not (Test-Sprint8ARehearsalDeclarationMember -Declaration $genericDictionary -Name "scheduler_role") -or
        (Test-Sprint8ARehearsalDeclarationMember -Declaration $genericDictionary -Name "impact_paths")) {
        throw "Candidate Rehearsal scheduler self-test must dispatch member lookup through the IDictionary interface."
    }
    foreach ($requiredMember in @("scheduler_role", "impact_paths", "impact_sources")) {
        $invalidDeclaration = [ordered]@{}
        foreach ($entry in $checks[0].GetEnumerator()) {
            if ([string]$entry.Key -cne $requiredMember) {
                $invalidDeclaration[[string]$entry.Key] = $entry.Value
            }
        }
        $invalidChecks = @($invalidDeclaration) + @($checks | Select-Object -Skip 1)
        $expectedFailure = if ($requiredMember -ceq "scheduler_role") {
            "Candidate Rehearsal lane 'lock' has an unsupported scheduler role."
        } else {
            "Candidate Rehearsal lane 'lock' omits its correction-impact contract."
        }
        try {
            Assert-Sprint8ARehearsalSchedulerDeclarations -Checks $invalidChecks
            throw "Candidate Rehearsal scheduler self-test accepted an ordered declaration missing '$requiredMember'."
        } catch {
            if ($_.Exception.Message -ceq "Candidate Rehearsal scheduler self-test accepted an ordered declaration missing '$requiredMember'.") {
                throw
            }
            if ($_.Exception.Message -cne $expectedFailure) {
                throw "Candidate Rehearsal scheduler self-test rejected missing '$requiredMember' for the wrong reason: $($_.Exception.Message)"
            }
        }
    }
    $priorPass = [pscustomobject]@{ path = "evidence/lanes/prior-pass.json"; sha256 = "a" * 64 }
    $source = [pscustomobject]@{
        commit = "b" * 40; tree = "c" * 40; dirty = $false; branch = "test"
        acceptance_inventory_sha256 = "d" * 64; deployment_inputs_sha256 = "e" * 64
    }
    $history = @{
        "lock" = [pscustomobject]@{ preceding_state = "passed"; consecutive_deferrals = 0; ever_executed = $true; prior_passing_receipt = $priorPass; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
        "prior-pass" = [pscustomobject]@{ preceding_state = "passed"; consecutive_deferrals = 0; ever_executed = $true; prior_passing_receipt = $priorPass; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
        "prior-failure" = [pscustomobject]@{ preceding_state = "failed"; consecutive_deferrals = 0; ever_executed = $true; prior_passing_receipt = $null; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
        "impacted-dependent" = [pscustomobject]@{ preceding_state = "passed"; consecutive_deferrals = 0; ever_executed = $true; prior_passing_receipt = $priorPass; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
        "cleanup" = [pscustomobject]@{ preceding_state = "passed"; consecutive_deferrals = 0; ever_executed = $true; prior_passing_receipt = $priorPass; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
        "cleanup-sink" = [pscustomobject]@{ preceding_state = "passed"; consecutive_deferrals = 0; ever_executed = $true; prior_passing_receipt = $priorPass; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
        "sink" = [pscustomobject]@{ preceding_state = "blocked"; consecutive_deferrals = 0; ever_executed = $false; prior_passing_receipt = $null; prior_source_identity = $source; prior_environment_fingerprint = "f" * 64 }
    }
    $schedule = Resolve-Sprint8ARehearsalSchedule `
        -Checks $checks -Attempt 8 -LaneHistory $history -ChangedPaths @("src/dependent/value.rs") `
        -AcceptanceInventoryChanged:$false -DeploymentInputsChanged:$false -EnvironmentContractChanged:$false `
        -HistoryAuthenticated:$true -FallbackReason $null
    $repeat = Resolve-Sprint8ARehearsalSchedule `
        -Checks $checks -Attempt 8 -LaneHistory $history -ChangedPaths @("src/dependent/value.rs") `
        -AcceptanceInventoryChanged:$false -DeploymentInputsChanged:$false -EnvironmentContractChanged:$false `
        -HistoryAuthenticated:$true -FallbackReason $null
    [void](Assert-Sprint8ARehearsalScheduleContract -Schedule $schedule -Checks $checks -ExpectedAttempt 8)
    if (($schedule | ConvertTo-Json -Depth 30 -Compress) -cne ($repeat | ConvertTo-Json -Depth 30 -Compress)) {
        throw "Candidate Rehearsal ordering is not deterministic for the immutable start receipt."
    }
    if ($schedule.wave_a -cnotcontains "prior-failure" -or
        $schedule.wave_a -cnotcontains "impacted-dependent" -or
        $schedule.wave_a -cnotcontains "prior-pass" -or
        $schedule.wave_a -cnotcontains "cleanup" -or
        $schedule.cleanup_sinks -cnotcontains "cleanup-sink" -or
        $schedule.aggregate_sinks -cnotcontains "sink") {
        throw "Candidate Rehearsal Wave A selection or prerequisite closure is incomplete."
    }
    $sinkDecision = @($schedule.decisions | Where-Object name -CEQ "sink")[0]
    if ([string]$sinkDecision.segment -cne "aggregate_sinks") {
        throw "Aggregate sink incorrectly expanded Wave A to the whole graph."
    }
    $dispositionStates = @{}
    foreach ($name in @(@($schedule.wave_a) + @($schedule.cleanup_sinks))) {
        $dispositionStates[[string]$name] = [pscustomobject]@{ state = "passed" }
    }
    if ((Get-Sprint8ARehearsalWaveBDisposition -Schedule $schedule -TerminalByName $dispositionStates) -cne "execute") {
        throw "Passing Wave A did not continue into Wave B in the same attempt."
    }
    $dispositionStates["prior-failure"].state = "failed"
    if ((Get-Sprint8ARehearsalWaveBDisposition -Schedule $schedule -TerminalByName $dispositionStates) -cne "defer") {
        throw "Failed Wave A did not defer eligible Wave B lanes after safe harvesting."
    }
    $dispositionStates["prior-failure"].state = "passed"
    [void]$dispositionStates.Remove("cleanup-sink")
    try {
        Get-Sprint8ARehearsalWaveBDisposition -Schedule $schedule -TerminalByName $dispositionStates | Out-Null
        throw "Wave B disposition self-test ignored mandatory cleanup terminalization."
    } catch {
        if ($_.Exception.Message -ceq "Wave B disposition self-test ignored mandatory cleanup terminalization.") { throw }
    }
    $dispositionStates["cleanup-sink"] = [pscustomobject]@{ state = "passed" }

    $deferHistory = $history["prior-pass"]
    $decision = [pscustomobject][ordered]@{
        wave = "B"; reason = "prior_passing_outside_impact"
        current_correction_impact = [pscustomobject][ordered]@{
            affected = $false; decision = "outside_correction_impact"; matched_paths = @(); matched_identity_sources = @()
            rationale = "No authenticated changed input intersects this lane."
        }
    }
    $deferred = New-Sprint8ADeferredLaneResult -Declaration $checks[1] -Decision $decision -History $deferHistory -Attempt 8 -TerminalByName @{}
    if ([string]$deferred.state -cne "deferred" -or [bool]$deferred.assertions_started -or
        [int]$deferred.consecutive_deferral_count -ne 1 -or [int]$deferred.mandatory_by_attempt -ne 11) {
        throw "Wave A failure did not create the exact eligible Wave B deferred result."
    }

    $history["prior-pass"].consecutive_deferrals = 3
    $forced = Resolve-Sprint8ARehearsalSchedule `
        -Checks $checks -Attempt 11 -LaneHistory $history -ChangedPaths @() `
        -AcceptanceInventoryChanged:$false -DeploymentInputsChanged:$false -EnvironmentContractChanged:$false `
        -HistoryAuthenticated:$true -FallbackReason $null
    if ($forced.wave_a -cnotcontains "prior-pass") {
        throw "Three consecutive deferrals did not force execution on the next attempt."
    }
    $history["prior-pass"].consecutive_deferrals = 0

    $impactOverride = Resolve-Sprint8ARehearsalSchedule `
        -Checks $checks -Attempt 9 -LaneHistory $history -ChangedPaths @("src/pass/change.rs") `
        -AcceptanceInventoryChanged:$false -DeploymentInputsChanged:$false -EnvironmentContractChanged:$false `
        -HistoryAuthenticated:$true -FallbackReason $null
    if ($impactOverride.wave_a -cnotcontains "prior-pass") {
        throw "Correction impact did not override Wave B eligibility."
    }
    $fallback = Resolve-Sprint8ARehearsalSchedule `
        -Checks $checks -Attempt 9 -LaneHistory @{} -ChangedPaths @() `
        -AcceptanceInventoryChanged:$false -DeploymentInputsChanged:$false -EnvironmentContractChanged:$false `
        -HistoryAuthenticated:$false -FallbackReason "unauthenticated fixture"
    if ($fallback.wave_b.Count -ne 0 -or @($fallback.wave_a).Count -ne 5 -or
        @($fallback.cleanup_sinks).Count -ne 1) {
        throw "Unauthenticated prior evidence did not select conservative full-harvest diagnostics."
    }

    $terminal = @(
        [pscustomobject]@{ name = "lock"; state = "passed" },
        $deferred,
        [pscustomobject]@{ name = "prior-failure"; state = "failed" },
        [pscustomobject]@{ name = "impacted-dependent"; state = "blocked" },
        [pscustomobject]@{ name = "cleanup"; state = "passed" },
        [pscustomobject]@{ name = "cleanup-sink"; state = "passed" },
        [pscustomobject]@{ name = "sink"; state = "blocked" }
    )
    try {
        Assert-Sprint8ARehearsalTerminalAccounting -DeclaredChecks $checks -TerminalChecks $terminal -Attempt 8 -AttemptState "passed"
        throw "Deferred Candidate Rehearsal self-test incorrectly authorized a pass."
    } catch {
        if ($_.Exception.Message -ceq "Deferred Candidate Rehearsal self-test incorrectly authorized a pass.") { throw }
    }

    $recovered = $schedule | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    if ((@($recovered.wave_a) -join "`n") -cne (@($schedule.wave_a) -join "`n") -or
        [int](@($recovered.decisions | Where-Object name -CEQ "prior-pass")[0].consecutive_deferrals_before) -ne
            [int](@($schedule.decisions | Where-Object name -CEQ "prior-pass")[0].consecutive_deferrals_before)) {
        throw "Process-loss recovery did not preserve immutable ordering and counters."
    }
    $recoveryScheduleSha = Get-Sprint8ARehearsalJsonSha256 -Document $recovered
    $recoveryStart = [pscustomobject][ordered]@{
        schema_version = 3
        phase = "candidate-rehearsal-start"
        attempt = 8
        schedule = $recovered
        schedule_sha256 = $recoveryScheduleSha
    }
    $recoveryAttempt = [pscustomobject][ordered]@{
        schema_version = 3
        phase = "candidate-rehearsal"
        attempt = 8
        state = "executing"
        schedule_sha256 = $recoveryScheduleSha
        immutable_start_receipt = [pscustomobject][ordered]@{
            path = "artifacts/sprint-8a-closeout/attempts/candidate-rehearsal-8-start.json"
            sha256 = "9" * 64
        }
    }
    $restored = Assert-Sprint8ARehearsalRecoveryScheduleBinding `
        -StartDocument $recoveryStart `
        -AttemptDocument $recoveryAttempt `
        -Checks $checks `
        -ExpectedAttempt 8 `
        -ExpectedStartPath ([string]$recoveryAttempt.immutable_start_receipt.path) `
        -ExpectedStartSha256 ([string]$recoveryAttempt.immutable_start_receipt.sha256)
    if ((@($restored.wave_a) -join "`n") -cne (@($schedule.wave_a) -join "`n")) {
        throw "Process-loss recovery changed the immutable execution order."
    }
    $tamperedStart = $recoveryStart | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    @($tamperedStart.schedule.decisions | Where-Object name -CEQ "prior-pass")[0].consecutive_deferrals_before = 2
    try {
        Assert-Sprint8ARehearsalRecoveryScheduleBinding `
            -StartDocument $tamperedStart `
            -AttemptDocument $recoveryAttempt `
            -Checks $checks `
            -ExpectedAttempt 8 `
            -ExpectedStartPath ([string]$recoveryAttempt.immutable_start_receipt.path) `
            -ExpectedStartSha256 ([string]$recoveryAttempt.immutable_start_receipt.sha256) | Out-Null
        throw "Process-loss recovery self-test accepted a changed deferral counter."
    } catch {
        if ($_.Exception.Message -ceq "Process-loss recovery self-test accepted a changed deferral counter.") { throw }
    }
    Test-Sprint8ARehearsalHistoryRegressionFixtures
}

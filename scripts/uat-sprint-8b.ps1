[CmdletBinding()]
param(
    [ValidateSet(
        "All",
        "UAT-8B-01", "UAT-8B-02", "UAT-8B-03", "UAT-8B-04",
        "UAT-8B-05", "UAT-8B-06", "UAT-8B-07", "UAT-8B-08",
        "UAT-8B-09", "UAT-8B-10", "UAT-8B-11"
    )]
    [string[]]$Scenario = @("All"),
    [string]$ComposeProject = "tessara-s8b-uat-scripted",
    [switch]$UseExistingTopology,
    [string]$FixtureReceiptPath,
    [string]$EvidencePath = "target/sprint-8b-uat/predicate-result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidencePathWasExplicit = $PSBoundParameters.ContainsKey("EvidencePath")
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8b-harness-isolation.ps1")
$SelfTest = $requestedSelfTest
$scenarioContractPath = Join-Path $repoRoot "docs/sprints/sprint-8b-uat/scenario-contract.json"
$script:Sprint8BRefreshTestIdentities = @(
    "unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state",
    "ordered_fixed_bound_pages_promote_each_response_change_once",
    "interrupted_page_attempt_retry_converges_once_from_published_cursor",
    "concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay",
    "expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement",
    "refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding",
    "derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure",
    "refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing"
)
$script:Sprint8BDagTestIdentities = @(
    "candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt",
    "rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact",
    "downstream_materialization_failure_rolls_back_every_rebuilt_table"
)

function Get-Sprint8BUatPredicateCatalog {
    @(
        [pscustomobject][ordered]@{
            id = "acceptance-contract"; kind = "script"
            path = "scripts/sprint-8b-acceptance-contract.ps1"; arguments = @()
            topology = "none"
        },
        [pscustomobject][ordered]@{
            id = "dataset-authoring"; kind = "script"
            path = "scripts/test-sprint-8b-dataset-module.ps1"; arguments = @("-Suite", "Authoring")
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "dataset-product"; kind = "script"
            path = "scripts/test-sprint-8b-dataset-module.ps1"; arguments = @("-Suite", "Product")
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "dataset-provider"; kind = "script"
            path = "scripts/test-sprint-8b-dataset-module.ps1"; arguments = @("-Suite", "Provider")
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "dataset-refresh-orchestration"; kind = "script"
            path = "scripts/test-sprint-8b-dataset-module.ps1"; arguments = @("-Suite", "Refresh")
            topology = "isolated-database"
            evidence_contract = [pscustomobject][ordered]@{
                proof = "dataset-module-test-suite"
                suite = "Refresh"
                test_binary = "refresh_integration"
                expected_test_identities = @($script:Sprint8BRefreshTestIdentities)
            }
        },
        [pscustomobject][ordered]@{
            id = "dataset-dag"; kind = "script"
            path = "scripts/test-sprint-8b-dataset-module.ps1"; arguments = @("-Suite", "Dag")
            topology = "isolated-database"
            evidence_contract = [pscustomobject][ordered]@{
                proof = "dataset-module-test-suite"
                suite = "Dag"
                test_binary = "dependency_dag_integration"
                expected_test_identities = @($script:Sprint8BDagTestIdentities)
            }
        },
        [pscustomobject][ordered]@{
            id = "component-consumer"; kind = "script"
            path = "scripts/test-sprint-8b-component-consumer.ps1"; arguments = @()
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "core-subtraction"; kind = "script"
            path = "scripts/check-sprint-8b-dataset-boundaries.ps1"; arguments = @("-Mode", "RequireClean")
            topology = "none"
        },
        [pscustomobject][ordered]@{
            id = "provider-boundaries"; kind = "readiness-target"
            target = "ui-provider-boundaries"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "reverse-consumers"; kind = "readiness-target"
            target = "reverse-consumers"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "resource-resolution"; kind = "readiness-target"
            target = "resource-resolution"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "api-idempotency"; kind = "readiness-target"
            target = "api-idempotency"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "browser-datasets"; kind = "program"; program = "npm"
            arguments = @("--prefix", "end2end", "test", "--", "tests/datasets-module.spec.ts")
            topology = "existing-reference"
        },
        [pscustomobject][ordered]@{
            id = "browser-dataset-visual"; kind = "program"; program = "npm"
            arguments = @("--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Datasets")
            topology = "existing-reference"
        },
        [pscustomobject][ordered]@{
            id = "materialization-noop"; kind = "materialization"
            topology = "owned-clean-lane"
        },
        [pscustomobject][ordered]@{
            id = "deployed-smoke"; kind = "deployed-smoke"
            topology = "existing-or-owned-reference"
        },
        [pscustomobject][ordered]@{
            id = "failure-recovery"; kind = "failure-recovery"
            topology = "owned-clean-lane"
        },
        [pscustomobject][ordered]@{
            id = "independent-upgrade-rollback"; kind = "independent-upgrade-rollback"
            topology = "owned-clean-lane"
        }
    )
}

function Get-Sprint8BUatAssertionMap {
    [ordered]@{
        "UAT-8B-01" = [ordered]@{
            author = @("dataset-authoring", "browser-datasets")
            preview = @("dataset-product", "browser-datasets")
            revise = @("dataset-authoring", "browser-datasets")
            publish = @("dataset-authoring", "browser-datasets")
            catalog = @("dataset-product", "browser-datasets")
            responsive = @("browser-dataset-visual")
            accessible = @("browser-dataset-visual")
        }
        "UAT-8B-02" = [ordered]@{
            owner_order = @("materialization-noop")
            exact_releases = @("materialization-noop")
            owner_receipts = @("materialization-noop")
            semantic_noop = @("materialization-noop")
        }
        "UAT-8B-03" = [ordered]@{
            validate_configuration = @("deployed-smoke")
            apply_configuration = @("deployed-smoke")
            exact_health = @("deployed-smoke")
            sanitized_diagnostics = @("deployed-smoke")
        }
        "UAT-8B-04" = [ordered]@{
            unchanged_head_no_page = @("dataset-refresh-orchestration")
            ordered_changes_once = @("dataset-refresh-orchestration")
            interrupt_retry = @("dataset-refresh-orchestration")
            concurrent_refresh = @("dataset-refresh-orchestration")
            expired_cursor_rebase = @("dataset-refresh-orchestration")
            atomic_dataset_closure = @("dataset-refresh-orchestration")
            independent_binding_unchanged = @("dataset-refresh-orchestration")
            cycle_rejected = @("dataset-dag")
            derived_failure_rolls_back = @("dataset-refresh-orchestration")
            nondisclosure = @("dataset-refresh-orchestration")
        }
        "UAT-8B-05" = [ordered]@{
            component_executes_dataset = @("component-consumer", "deployed-smoke")
            dashboard_renders_component = @("deployed-smoke")
            dataset_outage_degrades = @("deployed-smoke")
            recovery_converges = @("deployed-smoke")
        }
        "UAT-8B-06" = [ordered]@{
            four_core_transitions = @("core-subtraction")
            one_dataset_enrollment = @("core-subtraction", "deployed-smoke")
            no_core_dataset_schema = @("core-subtraction")
            no_core_dataset_adapter = @("core-subtraction")
            pairwise_database_denial = @("core-subtraction", "deployed-smoke")
        }
        "UAT-8B-07" = [ordered]@{
            failed_apply_retained = @("failure-recovery")
            partial_topology_removed = @("failure-recovery")
            clean_successor = @("failure-recovery")
            retry_noop = @("failure-recovery")
            healthy_without_manual_repair = @("failure-recovery")
        }
        "UAT-8B-08" = [ordered]@{
            upgrade = @("independent-upgrade-rollback")
            rollback = @("independent-upgrade-rollback")
            restore = @("independent-upgrade-rollback")
            dataset_state_preserved = @("independent-upgrade-rollback")
            unrelated_owners_unchanged = @("independent-upgrade-rollback")
        }
        "UAT-8B-09" = [ordered]@{
            form_picker = @("dataset-provider", "browser-datasets")
            schema_fields = @("dataset-provider", "browser-datasets")
            scope_tree = @("dataset-provider", "browser-datasets")
            principal_labels = @("dataset-provider", "browser-datasets")
            direct_and_lifecycle_hydration = @("browser-datasets")
            isolated_provider_faults = @("provider-boundaries", "browser-datasets")
            browser_calls_dataset_only = @("provider-boundaries", "browser-datasets")
            unsaved_input_retained = @("browser-datasets")
        }
        "UAT-8B-10" = [ordered]@{
            form_dataset_sources = @("reverse-consumers", "browser-datasets")
            operations_readiness = @("reverse-consumers", "browser-datasets")
            app_summary = @("reverse-consumers", "browser-datasets")
            outage_is_unavailable_not_empty = @("reverse-consumers", "deployed-smoke")
            unrelated_content_usable = @("reverse-consumers", "deployed-smoke")
            recovery_exact = @("reverse-consumers", "deployed-smoke")
        }
        "UAT-8B-11" = [ordered]@{
            three_v2_resource_types = @("resource-resolution")
            known_random_nondisclosure = @("resource-resolution", "dataset-provider")
            mutation_replay = @("api-idempotency")
            changed_input_rejected = @("api-idempotency")
            private_nonce_one_use = @("api-idempotency")
            static_routes_precede_ids = @("resource-resolution", "browser-datasets")
        }
    }
}

function Get-Sprint8BUatAssertionEvidenceClaims {
    [ordered]@{
        "UAT-8B-04" = [ordered]@{
            unchanged_head_no_page = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[0]
            })
            ordered_changes_once = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[1]
            })
            interrupt_retry = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[2]
            })
            concurrent_refresh = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[3]
            })
            expired_cursor_rebase = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[4]
            })
            atomic_dataset_closure = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[5]
            })
            independent_binding_unchanged = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[5]
            })
            cycle_rejected = @([pscustomobject][ordered]@{
                predicate_id = "dataset-dag"
                test_identity = $script:Sprint8BDagTestIdentities[0]
            })
            derived_failure_rolls_back = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[6]
            })
            nondisclosure = @([pscustomobject][ordered]@{
                predicate_id = "dataset-refresh-orchestration"
                test_identity = $script:Sprint8BRefreshTestIdentities[7]
            })
        }
    }
}

function Get-Sprint8BUatAssertionProofContract {
    param(
        [Parameter(Mandatory)]$ScenarioContract,
        [Parameter(Mandatory)]$AssertionMap,
        [Parameter(Mandatory)]$EvidenceClaims
    )

    @($ScenarioContract.scenarios | ForEach-Object {
        $scenario = $_
        $scenarioClaims = $EvidenceClaims[[string]$scenario.id]
        @($scenario.assertions | ForEach-Object {
            $assertion = [string]$_
            [object[]]$claims = @(
                if ($null -ne $scenarioClaims) {
                    @($scenarioClaims[$assertion])
                }
            )
            [pscustomobject][ordered]@{
                scenario_id = [string]$scenario.id
                assertion = $assertion
                predicate_ids = @($AssertionMap[[string]$scenario.id][$assertion])
                automated_claim_kind = if ($claims.Count -eq 0) {
                    "prerequisite_only"
                } else {
                    "exact_test_evidence"
                }
                exact_evidence_claims = @($claims)
                manual_acceptance_required = $true
            }
        })
    })
}

function Assert-Sprint8BUatPredicateContract {
    param(
        [Parameter(Mandatory)]$ScenarioContract,
        [Parameter(Mandatory)][object[]]$PredicateCatalog,
        [Parameter(Mandatory)]$AssertionMap,
        [Parameter(Mandatory)]$EvidenceClaims
    )

    $expectedScenarioIds = @(1..11 | ForEach-Object { "UAT-8B-{0:d2}" -f $_ })
    $actualScenarioIds = @($ScenarioContract.scenarios.id)
    if ([int]$ScenarioContract.schema_version -ne 1 -or
        [string]$ScenarioContract.contract -cne "tessara.sprint-8b.uat-scenarios" -or
        ($actualScenarioIds -join "`n") -cne ($expectedScenarioIds -join "`n") -or
        (@($AssertionMap.Keys) -join "`n") -cne ($expectedScenarioIds -join "`n")) {
        throw "Sprint 8B UAT predicate contract is not set-equal to the eleven canonical scenarios."
    }
    $predicateIds = @($PredicateCatalog.id)
    if (@($predicateIds | Sort-Object -Unique).Count -ne $predicateIds.Count) {
        throw "Sprint 8B UAT predicate IDs are not unique."
    }
    foreach ($predicate in $PredicateCatalog) {
        if ([string]::IsNullOrWhiteSpace([string]$predicate.id) -or
            [string]$predicate.kind -notin @(
                "script", "program", "readiness-target", "materialization",
                "deployed-smoke", "failure-recovery", "independent-upgrade-rollback"
            ) -or
            [string]$predicate.topology -notin @(
                "none", "isolated-database", "existing-reference",
                "existing-or-owned-reference", "owned-clean-lane"
            )) {
            throw "Sprint 8B UAT predicate '$($predicate.id)' has no executable command/topology contract."
        }
        $evidenceProperty = $predicate.PSObject.Properties['evidence_contract']
        if ($null -ne $evidenceProperty) {
            $evidenceContract = $evidenceProperty.Value
            $expectedTestIdentities = @($evidenceContract.expected_test_identities | ForEach-Object {
                [string]$_
            })
            if ([string]$predicate.kind -cne "script" -or
                [string]$evidenceContract.proof -cne "dataset-module-test-suite" -or
                [string]::IsNullOrWhiteSpace([string]$evidenceContract.suite) -or
                [string]::IsNullOrWhiteSpace([string]$evidenceContract.test_binary) -or
                $expectedTestIdentities.Count -eq 0 -or
                @($expectedTestIdentities | Sort-Object -Unique).Count -ne $expectedTestIdentities.Count -or
                @($expectedTestIdentities | Where-Object { $_ -cnotmatch '^[a-z0-9_]+$' }).Count -ne 0) {
                throw "Sprint 8B UAT predicate '$($predicate.id)' has an invalid exact test-evidence contract."
            }
        }
    }
    foreach ($scenario in @($ScenarioContract.scenarios)) {
        if ([string]::IsNullOrWhiteSpace([string]$scenario.start_state) -or
            [string]::IsNullOrWhiteSpace([string]$scenario.cleanup)) {
            throw "Sprint 8B UAT scenario '$($scenario.id)' lacks an executable precondition or cleanup identity."
        }
        $mapping = $AssertionMap[[string]$scenario.id]
        if ($null -eq $mapping -or
            (@($mapping.Keys | Sort-Object) -join "`n") -cne
                (@($scenario.assertions | Sort-Object) -join "`n")) {
            throw "Sprint 8B UAT scenario '$($scenario.id)' assertion predicates are not set-equal to its contract."
        }
        foreach ($assertion in @($scenario.assertions)) {
            $mappedPredicates = @($mapping[[string]$assertion])
            if ($mappedPredicates.Count -eq 0) {
                throw "Sprint 8B UAT assertion '$($scenario.id)/$assertion' has no executable predicate."
            }
            foreach ($predicateId in $mappedPredicates) {
                if ($predicateIds -cnotcontains [string]$predicateId) {
                    throw "Sprint 8B UAT assertion '$($scenario.id)/$assertion' names unknown predicate '$predicateId'."
                }
            }
        }
        $scenarioClaims = $EvidenceClaims[[string]$scenario.id]
        if ($null -ne $scenarioClaims) {
            if ((@($scenarioClaims.Keys | Sort-Object) -join "`n") -cne
                (@($scenario.assertions | Sort-Object) -join "`n")) {
                throw "Sprint 8B UAT scenario '$($scenario.id)' exact evidence claims are not set-equal to its assertions."
            }
            foreach ($assertion in @($scenario.assertions)) {
                $claims = @($scenarioClaims[[string]$assertion])
                $mappedPredicates = @($mapping[[string]$assertion])
                $claimPredicates = @($claims.predicate_id | ForEach-Object { [string]$_ })
                if ($claims.Count -eq 0 -or
                    (@($claimPredicates | Sort-Object -Unique) -join "`n") -cne
                        (@($mappedPredicates | Sort-Object -Unique) -join "`n")) {
                    throw "Sprint 8B UAT assertion '$($scenario.id)/$assertion' does not bind every mapped predicate to exact evidence."
                }
                foreach ($claim in $claims) {
                    $predicate = @($PredicateCatalog | Where-Object {
                        [string]$_.id -ceq [string]$claim.predicate_id
                    })
                    if ($predicate.Count -ne 1 -or
                        $null -eq $predicate[0].PSObject.Properties['evidence_contract'] -or
                        @($predicate[0].evidence_contract.expected_test_identities) -cnotcontains
                            [string]$claim.test_identity) {
                        throw "Sprint 8B UAT assertion '$($scenario.id)/$assertion' names an unproven exact test identity."
                    }
                }
            }
        }
    }
}

function Get-Sprint8BSelectedScenarios {
    param(
        [Parameter(Mandatory)]$ScenarioContract,
        [Parameter(Mandatory)][string[]]$Selections
    )

    if ($Selections -ccontains "All") {
        if ($Selections.Count -ne 1) { throw "UAT scenario selector 'All' cannot be combined with explicit IDs." }
        return @($ScenarioContract.scenarios)
    }
    $unique = @($Selections | Sort-Object -Unique)
    if ($unique.Count -ne $Selections.Count) { throw "UAT scenario selectors must be unique." }
    @($ScenarioContract.scenarios | Where-Object { $unique -ccontains [string]$_.id })
}

function Get-Sprint8BPredicateSelection {
    param(
        [Parameter(Mandatory)][object[]]$SelectedScenarios,
        [Parameter(Mandatory)]$AssertionMap,
        [Parameter(Mandatory)][object[]]$PredicateCatalog
    )

    $selectedIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($scenario in $SelectedScenarios) {
        $mapping = $AssertionMap[[string]$scenario.id]
        foreach ($assertion in @($scenario.assertions)) {
            foreach ($predicateId in @($mapping[[string]$assertion])) {
                [void]$selectedIds.Add([string]$predicateId)
            }
        }
    }
    @($PredicateCatalog | Where-Object { $selectedIds.Contains([string]$_.id) })
}

function Invoke-Sprint8BProgramPredicate {
    param(
        [Parameter(Mandatory)][string]$Program,
        [AllowEmptyCollection()][string[]]$Arguments = @(),
        [switch]$AllowFailure
    )

    $output = @(& $Program @Arguments 2>&1 | ForEach-Object { [string]$_ })
    $exitCode = $LASTEXITCODE
    if (-not $AllowFailure -and $exitCode -ne 0) {
        throw "Predicate command '$Program $($Arguments -join ' ')' exited $exitCode.`n$($output -join "`n")"
    }
    [pscustomobject][ordered]@{
        program = $Program
        arguments = @($Arguments)
        exit_code = $exitCode
        output = @($output)
    }
}

function Assert-Sprint8BExactTestPredicateEvidence {
    param(
        [Parameter(Mandatory)]$Predicate,
        [Parameter(Mandatory)][string]$EvidencePath
    )

    $contractProperty = $Predicate.PSObject.Properties['evidence_contract']
    if ($null -eq $contractProperty) {
        throw "Predicate '$($Predicate.id)' has no exact test-evidence contract."
    }
    $resolved = Resolve-Sprint8BRepositoryPath -Path $EvidencePath
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $resolved -SidecarPath "$resolved.sha256")) {
        throw "Predicate '$($Predicate.id)' did not publish an authenticated evidence pair."
    }
    $document = Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json -Depth 100
    $contract = $contractProperty.Value
    $expected = @($contract.expected_test_identities | ForEach-Object { [string]$_ })
    $declared = @($document.expected_test_identities | ForEach-Object { [string]$_ })
    $executed = @($document.executed_test_identities | ForEach-Object { [string]$_ })
    if ([int]$document.schema_version -ne 1 -or
        [string]$document.sprint -cne "sprint-8b" -or
        [string]$document.proof -cne [string]$contract.proof -or
        [string]$document.state -cne "passed" -or
        [string]$document.suite -cne [string]$contract.suite -or
        [string]$document.test_binary -cne [string]$contract.test_binary -or
        [int]$document.executed_test_count -ne $expected.Count -or
        (@($declared | Sort-Object -Unique) -join "`n") -cne
            (@($expected | Sort-Object -Unique) -join "`n") -or
        (@($executed | Sort-Object -Unique) -join "`n") -cne
            (@($expected | Sort-Object -Unique) -join "`n") -or
        $declared.Count -ne $expected.Count -or
        $executed.Count -ne $expected.Count -or
        [string]$document.database.mode -cne "disposable-postgres" -or
        [string]$document.database.cleanup_restoration.state -cne "passed") {
        throw "Predicate '$($Predicate.id)' evidence does not prove its exact test identity and cleanup contract."
    }
    $arguments = @($document.command.arguments | ForEach-Object { [string]$_ })
    $packageIndex = [Array]::IndexOf($arguments, "-p")
    $binaryIndex = [Array]::IndexOf($arguments, "--test")
    $jobsIndex = [Array]::IndexOf($arguments, "--jobs")
    if ($arguments.Count -eq 0 -or $arguments[0] -cne "test" -or
        $packageIndex -lt 0 -or $packageIndex + 1 -ge $arguments.Count -or
        $arguments[$packageIndex + 1] -cne "tessara-dataset-module" -or
        $binaryIndex -lt 0 -or $binaryIndex + 1 -ge $arguments.Count -or
        $arguments[$binaryIndex + 1] -cne [string]$contract.test_binary -or
        $arguments -cnotcontains "--locked" -or
        $arguments -cnotcontains "--offline" -or
        $jobsIndex -lt 0 -or $jobsIndex + 1 -ge $arguments.Count -or
        $arguments[$jobsIndex + 1] -cne "1") {
        throw "Predicate '$($Predicate.id)' evidence does not retain its exact Cargo test selector."
    }
    [pscustomobject][ordered]@{
        path = $resolved
        sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
        proof = [string]$document.proof
        suite = [string]$document.suite
        test_binary = [string]$document.test_binary
        test_identities = @($executed)
        executed_test_count = [int]$document.executed_test_count
        cleanup_restoration = $document.database.cleanup_restoration
    }
}

function Get-Sprint8BUatChildEvidenceRoot {
    param(
        [Parameter(Mandatory)][string]$EvidencePath
    )

    $resolvedEvidencePath = Resolve-Sprint8BRepositoryPath -Path $EvidencePath
    $evidenceStem = [IO.Path]::GetFileNameWithoutExtension($resolvedEvidencePath)
    if ([string]::IsNullOrWhiteSpace($evidenceStem) -or
        $evidenceStem -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
        throw "Sprint 8B UAT evidence path must have a bounded action-specific JSON filename."
    }
    Join-Path (Join-Path (Split-Path -Parent $resolvedEvidencePath) "predicates") $evidenceStem
}

function Assert-Sprint8BSelectedAssertionEvidence {
    param(
        [Parameter(Mandatory)][object[]]$SelectedScenarios,
        [Parameter(Mandatory)]$EvidenceClaims,
        [Parameter(Mandatory)][object[]]$PredicateResults
    )

    $verified = [Collections.Generic.List[object]]::new()
    foreach ($scenario in $SelectedScenarios) {
        $scenarioClaims = $EvidenceClaims[[string]$scenario.id]
        if ($null -eq $scenarioClaims) { continue }
        foreach ($assertion in @($scenario.assertions)) {
            foreach ($claim in @($scenarioClaims[[string]$assertion])) {
                $result = @($PredicateResults | Where-Object {
                    [string]$_.predicate_id -ceq [string]$claim.predicate_id
                })
                if ($result.Count -ne 1 -or [string]$result[0].state -cne "passed" -or
                    $null -eq $result[0].PSObject.Properties['exact_test_evidence'] -or
                    @($result[0].exact_test_evidence.test_identities) -cnotcontains
                        [string]$claim.test_identity) {
                    throw "Assertion '$($scenario.id)/$assertion' lacks its exact executed test evidence."
                }
                $verified.Add([pscustomobject][ordered]@{
                    scenario_id = [string]$scenario.id
                    assertion = [string]$assertion
                    predicate_id = [string]$claim.predicate_id
                    test_identity = [string]$claim.test_identity
                    evidence_sha256 = [string]$result[0].exact_test_evidence.sha256
                })
            }
        }
    }
    @($verified)
}

function Get-Sprint8BUatAssertionProofState {
    param(
        [Parameter(Mandatory)][string]$ScenarioId,
        [Parameter(Mandatory)][string]$Assertion,
        [Parameter(Mandatory)]$EvidenceClaims,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$VerifiedAssertionEvidence,
        [Parameter(Mandatory)][bool]$PredicateExecutionPassed
    )

    if (-not $PredicateExecutionPassed) {
        return [pscustomobject][ordered]@{
            state = "not_proven"
            automated_claim_kind = "none"
            manual_acceptance_required = $true
        }
    }

    $scenarioClaims = $EvidenceClaims[$ScenarioId]
    [object[]]$claims = @(
        if ($null -ne $scenarioClaims) {
            @($scenarioClaims[$Assertion])
        }
    )
    if ($claims.Count -eq 0) {
        return [pscustomobject][ordered]@{
            state = "predicate_prerequisites_passed"
            automated_claim_kind = "prerequisite_only"
            manual_acceptance_required = $true
        }
    }

    $verified = @($VerifiedAssertionEvidence | Where-Object {
        [string]$_.scenario_id -ceq $ScenarioId -and
        [string]$_.assertion -ceq $Assertion
    })
    if ($verified.Count -ne $claims.Count) {
        throw "Assertion '$ScenarioId/$Assertion' cannot claim exact automated evidence without every authenticated claim."
    }
    [pscustomobject][ordered]@{
        state = "exact_automated_evidence_passed"
        automated_claim_kind = "exact_test_evidence"
        manual_acceptance_required = $true
    }
}

function ConvertTo-Sprint8BUatScenarioAssertionResults {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$SelectedScenarios,
        [Parameter(Mandatory)]$AssertionMap,
        [Parameter(Mandatory)]$EvidenceClaims,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$VerifiedAssertionEvidence,
        [Parameter(Mandatory)][bool]$PredicateExecutionPassed
    )

    @($SelectedScenarios | ForEach-Object {
        $scenarioDefinition = $_
        [pscustomobject][ordered]@{
            scenario_id = [string]$scenarioDefinition.id
            start_state = [string]$scenarioDefinition.start_state
            cleanup = [string]$scenarioDefinition.cleanup
            assertions = @($scenarioDefinition.assertions | ForEach-Object {
                $assertion = [string]$_
                $proofState = Get-Sprint8BUatAssertionProofState `
                    -ScenarioId ([string]$scenarioDefinition.id) -Assertion $assertion `
                    -EvidenceClaims $EvidenceClaims `
                    -VerifiedAssertionEvidence @($VerifiedAssertionEvidence) `
                    -PredicateExecutionPassed $PredicateExecutionPassed
                [pscustomobject][ordered]@{
                    assertion = $assertion
                    predicate_ids = @($AssertionMap[[string]$scenarioDefinition.id][$assertion])
                    exact_evidence_claims = @(
                        if ($null -ne $EvidenceClaims[[string]$scenarioDefinition.id]) {
                            @($EvidenceClaims[[string]$scenarioDefinition.id][$assertion])
                        }
                    )
                    state = [string]$proofState.state
                    automated_claim_kind = [string]$proofState.automated_claim_kind
                    manual_acceptance_required = [bool]$proofState.manual_acceptance_required
                }
            })
        }
    })
}

function Test-Sprint8BUatPredicateReadiness {
    $scenarioContract = Get-Content -LiteralPath $scenarioContractPath -Raw | ConvertFrom-Json -Depth 100
    $catalog = @(Get-Sprint8BUatPredicateCatalog)
    $mapping = Get-Sprint8BUatAssertionMap
    $evidenceClaims = Get-Sprint8BUatAssertionEvidenceClaims
    $assertionProofContract = @(Get-Sprint8BUatAssertionProofContract `
        -ScenarioContract $scenarioContract -AssertionMap $mapping `
        -EvidenceClaims $evidenceClaims)
    Assert-Sprint8BUatPredicateContract -ScenarioContract $scenarioContract `
        -PredicateCatalog $catalog -AssertionMap $mapping -EvidenceClaims $evidenceClaims
    $exactAssertionContracts = @($assertionProofContract | Where-Object {
        [string]$_.automated_claim_kind -ceq "exact_test_evidence"
    })
    $prerequisiteOnlyContracts = @($assertionProofContract | Where-Object {
        [string]$_.automated_claim_kind -ceq "prerequisite_only"
    })
    if ($assertionProofContract.Count -ne 64 -or
        $exactAssertionContracts.Count -ne 10 -or
        $prerequisiteOnlyContracts.Count -ne 54 -or
        @($assertionProofContract | Where-Object {
            -not [bool]$_.manual_acceptance_required
        }).Count -ne 0) {
        throw "Sprint 8B UAT assertion proof classification is not exact and fail-closed."
    }

    $selected = @(Get-Sprint8BSelectedScenarios -ScenarioContract $scenarioContract -Selections @("UAT-8B-04", "UAT-8B-11"))
    $predicates = @(Get-Sprint8BPredicateSelection -SelectedScenarios $selected `
        -AssertionMap $mapping -PredicateCatalog $catalog)
    if (($selected.id -join "`n") -cne (@("UAT-8B-04", "UAT-8B-11") -join "`n") -or
        $predicates.id -cnotcontains "dataset-refresh-orchestration" -or
        $predicates.id -cnotcontains "resource-resolution") {
        throw "Sprint 8B UAT exact scenario/predicate selection self-test failed."
    }

    $allSelected = @(Get-Sprint8BSelectedScenarios `
        -ScenarioContract $scenarioContract -Selections @("All"))
    $allScenarioAssertions = @(ConvertTo-Sprint8BUatScenarioAssertionResults `
        -SelectedScenarios $allSelected -AssertionMap $mapping `
        -EvidenceClaims $evidenceClaims -VerifiedAssertionEvidence @() `
        -PredicateExecutionPassed $false)
    if ($allScenarioAssertions.Count -ne 11 -or
        @($allScenarioAssertions.assertions).Count -ne 64 -or
        @($allScenarioAssertions.assertions | Where-Object {
            [string]$_.state -cne "not_proven" -or
            -not [bool]$_.manual_acceptance_required
        }).Count -ne 0) {
        throw "Sprint 8B UAT All-scenario result assembly self-test failed."
    }

    $allPredicates = @(Get-Sprint8BPredicateSelection -SelectedScenarios $allSelected `
        -AssertionMap $mapping -PredicateCatalog $catalog)
    $parentProject = "tessara-s8b-uat-selftest"
    $topologyPlans = @($allPredicates | ForEach-Object {
        Get-Sprint8BUatPredicateTopologyPlan -Predicate $_ -ParentComposeProject $parentProject
    })
    $ownedCleanPlans = @($topologyPlans | Where-Object { [bool]$_.owns_clean_lane })
    $referencePlans = @($topologyPlans | Where-Object { [bool]$_.requires_reference })
    if ($ownedCleanPlans.Count -ne 3 -or
        @($ownedCleanPlans.compose_project | Sort-Object -Unique).Count -ne $ownedCleanPlans.Count -or
        @($ownedCleanPlans | Where-Object {
            [string]$_.compose_project -ceq $parentProject -or [bool]$_.requires_reference
        }).Count -ne 0 -or
        $referencePlans.Count -eq 0 -or
        @($referencePlans | Where-Object {
            [string]$_.compose_project -cne $parentProject -or
            -not [bool]$_.materialize_reference_on_demand -or
            -not [bool]$_.reference_preflight_required
        }).Count -ne 0 -or
        @($topologyPlans | Where-Object {
            -not [bool]$_.requires_reference -and -not [bool]$_.owns_clean_lane -and
            [string]$_.compose_project -cne $parentProject
        }).Count -ne 0) {
        throw "Sprint 8B UAT topology scheduling self-test failed."
    }

    $materializedPorts = Set-Sprint8BUatMaterializedTopologyEnvironment `
        -ExpectedComposeProject $parentProject -MaterializationReceipt ([pscustomobject]@{
            compose_project = $parentProject
            environment = [pscustomobject]@{
                COMPOSE_PROJECT_NAME = $parentProject
                TESSARA_GATEWAY_PORT = "45101"
                TESSARA_CORE_CONTROL_PORT = "45102"
                TESSARA_SUPERVISOR_PORT = "45103"
            }
        })
    if ([string]$materializedPorts.gateway_url -cne "http://127.0.0.1:45101") {
        throw "Sprint 8B UAT materialized-port handoff self-test failed."
    }
    try {
        Set-Sprint8BUatMaterializedTopologyEnvironment `
            -ExpectedComposeProject $parentProject -MaterializationReceipt ([pscustomobject]@{
                compose_project = "tessara-s8b-substituted"
                environment = [pscustomobject]@{
                    COMPOSE_PROJECT_NAME = "tessara-s8b-substituted"
                    TESSARA_GATEWAY_PORT = "45101"
                    TESSARA_CORE_CONTROL_PORT = "45102"
                    TESSARA_SUPERVISOR_PORT = "45103"
                }
            }) | Out-Null
        throw "Sprint 8B UAT materialized-port handoff accepted a substituted project."
    } catch {
        if ($_.Exception.Message -notmatch 'substituted the Compose project') { throw }
    }

    $tampered = Get-Sprint8BUatAssertionMap
    [void]$tampered["UAT-8B-04"].Remove("unchanged_head_no_page")
    try {
        Assert-Sprint8BUatPredicateContract -ScenarioContract $scenarioContract `
            -PredicateCatalog $catalog -AssertionMap $tampered -EvidenceClaims $evidenceClaims
        throw "Sprint 8B UAT predicate self-test accepted a missing assertion mapping."
    } catch {
        if ($_.Exception.Message -notmatch 'not set-equal') { throw }
    }

    $tamperedClaims = Get-Sprint8BUatAssertionEvidenceClaims
    $tamperedClaims["UAT-8B-04"]["unchanged_head_no_page"][0].test_identity =
        "unrelated_store_level_test"
    try {
        Assert-Sprint8BUatPredicateContract -ScenarioContract $scenarioContract `
            -PredicateCatalog $catalog -AssertionMap $mapping -EvidenceClaims $tamperedClaims
        throw "Sprint 8B UAT predicate self-test accepted a substituted exact test identity."
    } catch {
        if ($_.Exception.Message -notmatch 'unproven exact test identity') { throw }
    }

    $selfTestRoot = Join-Path ([IO.Path]::GetTempPath()) `
        "tessara-s8b-uat-predicate-$([Guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($selfTestRoot) | Out-Null
        $firstScenarioEvidence = Join-Path $selfTestRoot "actions/uat-8b-01-evidence.json"
        $secondScenarioEvidence = Join-Path $selfTestRoot "actions/uat-8b-09-evidence.json"
        $firstChildRoot = Get-Sprint8BUatChildEvidenceRoot -EvidencePath $firstScenarioEvidence
        $secondChildRoot = Get-Sprint8BUatChildEvidenceRoot -EvidencePath $secondScenarioEvidence
        if ($firstChildRoot -ceq $secondChildRoot -or
            [IO.Path]::GetFileName($firstChildRoot) -cne "uat-8b-01-evidence" -or
            [IO.Path]::GetFileName($secondChildRoot) -cne "uat-8b-09-evidence") {
            throw "Sprint 8B UAT self-test did not isolate child evidence by formal action identity."
        }
        [IO.Directory]::CreateDirectory($firstChildRoot) | Out-Null
        [IO.Directory]::CreateDirectory($secondChildRoot) | Out-Null
        $firstMaterialization = Join-Path $firstChildRoot "reference-materialization.json"
        $secondMaterialization = Join-Path $secondChildRoot "reference-materialization.json"
        $syntheticMaterialization = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8b"
            proof = "synthetic-uat-materialization"
            state = "passed"
        }
        Publish-Sprint8BHarnessEvidence -Document $syntheticMaterialization `
            -OutputPath $firstMaterialization | Out-Null
        Publish-Sprint8BHarnessEvidence -Document $syntheticMaterialization `
            -OutputPath $secondMaterialization | Out-Null
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $firstMaterialization `
                -SidecarPath "$firstMaterialization.sha256") -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $secondMaterialization `
                -SidecarPath "$secondMaterialization.sha256")) {
            throw "Sprint 8B UAT self-test did not publish both action-scoped evidence pairs."
        }
        try {
            Publish-Sprint8BHarnessEvidence -Document $syntheticMaterialization `
                -OutputPath $firstMaterialization | Out-Null
            throw "Sprint 8B UAT self-test allowed an overwrite within one action evidence scope."
        } catch {
            if ($_.Exception.Message -notmatch 'Retained evidence exists') { throw }
        }

        $refreshPredicate = @($catalog | Where-Object {
            [string]$_.id -ceq "dataset-refresh-orchestration"
        })[0]
        $exactEvidencePath = Join-Path $selfTestRoot "refresh.json"
        Publish-Sprint8BHarnessEvidence -Document ([pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8b"
            proof = "dataset-module-test-suite"
            state = "passed"
            suite = "Refresh"
            test_binary = "refresh_integration"
            expected_test_identities = @($script:Sprint8BRefreshTestIdentities)
            executed_test_identities = @($script:Sprint8BRefreshTestIdentities)
            executed_test_count = $script:Sprint8BRefreshTestIdentities.Count
            database = [pscustomobject][ordered]@{
                mode = "disposable-postgres"
                cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
            }
            command = [pscustomobject][ordered]@{
                arguments = @(
                    "test", "-p", "tessara-dataset-module", "--test", "refresh_integration",
                    "--locked", "--offline", "--jobs", "1"
                )
            }
        }) -OutputPath $exactEvidencePath | Out-Null
        Assert-Sprint8BExactTestPredicateEvidence -Predicate $refreshPredicate `
            -EvidencePath $exactEvidencePath | Out-Null
        $tamperedEvidencePath = Join-Path $selfTestRoot "refresh-tampered.json"
        Publish-Sprint8BHarnessEvidence -Document ([pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8b"
            proof = "dataset-module-test-suite"
            state = "passed"
            suite = "Refresh"
            test_binary = "refresh_integration"
            expected_test_identities = @($script:Sprint8BRefreshTestIdentities)
            executed_test_identities = @($script:Sprint8BRefreshTestIdentities[0..6])
            executed_test_count = 7
            database = [pscustomobject][ordered]@{
                mode = "disposable-postgres"
                cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
            }
            command = [pscustomobject][ordered]@{
                arguments = @(
                    "test", "-p", "tessara-dataset-module", "--test", "refresh_integration",
                    "--locked", "--offline", "--jobs", "1"
                )
            }
        }) -OutputPath $tamperedEvidencePath | Out-Null
        try {
            Assert-Sprint8BExactTestPredicateEvidence -Predicate $refreshPredicate `
                -EvidencePath $tamperedEvidencePath | Out-Null
            throw "Sprint 8B UAT predicate self-test accepted incomplete exact test evidence."
        } catch {
            if ($_.Exception.Message -notmatch 'exact test identity') { throw }
        }

        $prerequisiteOnly = Get-Sprint8BUatAssertionProofState `
            -ScenarioId "UAT-8B-01" -Assertion "author" -EvidenceClaims $evidenceClaims `
            -VerifiedAssertionEvidence @() -PredicateExecutionPassed $true
        if ([string]$prerequisiteOnly.state -cne "predicate_prerequisites_passed" -or
            [string]$prerequisiteOnly.automated_claim_kind -cne "prerequisite_only" -or
            -not [bool]$prerequisiteOnly.manual_acceptance_required) {
            throw "Sprint 8B UAT self-test mislabeled association-only predicate evidence as an assertion pass."
        }

        $exactState = Get-Sprint8BUatAssertionProofState `
            -ScenarioId "UAT-8B-04" -Assertion "unchanged_head_no_page" `
            -EvidenceClaims $evidenceClaims -VerifiedAssertionEvidence @(
                [pscustomobject]@{
                    scenario_id = "UAT-8B-04"
                    assertion = "unchanged_head_no_page"
                }
            ) -PredicateExecutionPassed $true
        if ([string]$exactState.state -cne "exact_automated_evidence_passed" -or
            [string]$exactState.automated_claim_kind -cne "exact_test_evidence" -or
            -not [bool]$exactState.manual_acceptance_required) {
            throw "Sprint 8B UAT self-test did not retain the exact automated/manual proof boundary."
        }
    } finally {
        if (Test-Path -LiteralPath $selfTestRoot) {
            Remove-Item -LiteralPath $selfTestRoot -Recurse -Force
        }
    }

    $failed = Invoke-Sprint8BProgramPredicate -Program "pwsh" `
        -Arguments @("-NoProfile", "-Command", "exit 19") -AllowFailure
    if ($failed.exit_code -ne 19) {
        throw "Sprint 8B UAT predicate child failure self-test did not retain the exact exit code."
    }
    try {
        Invoke-Sprint8BProgramPredicate -Program "pwsh" `
            -Arguments @("-NoProfile", "-Command", "exit 23") | Out-Null
        throw "Sprint 8B UAT predicate child failure self-test did not fail closed."
    } catch {
        if ($_.Exception.Message -notmatch 'exited 23') { throw }
    }

    $contractHash = (Get-FileHash -LiteralPath $scenarioContractPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $assertionProofContractHash = Get-Sprint7ASha256 -Text (
        ($assertionProofContract | ConvertTo-Json -Depth 100 -Compress) + "`n"
    )
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "uat-automated-predicate-readiness"
        state = "passed"
        database_free = $true
        manual_acceptance_claimed = $false
        manual_acceptance_required = $true
        compose_project = $null
        selected_scenarios = @($scenarioContract.scenarios.id)
        assertion_count = @($scenarioContract.scenarios.assertions).Count
        predicate_ids = @($catalog.id)
        exact_assertion_evidence = "passed"
        association_only_assertion_pass_forbidden = "passed"
        assertion_proof_contract = [pscustomobject][ordered]@{
            sha256 = $assertionProofContractHash
            assertion_count = $assertionProofContract.Count
            exact_test_evidence_count = $exactAssertionContracts.Count
            prerequisite_only_count = $prerequisiteOnlyContracts.Count
            manual_acceptance_required_count = $assertionProofContract.Count
        }
        scenario_contract_sha256 = $contractHash
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text (
            "uat-readiness`n$contractHash`n$(@($catalog.id) -join "`n")"
        )
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"
            mode = "database-free-self-test"
        }
    }
}

function Assert-Sprint8BFixtureReceipt {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ExpectedComposeProject
    )

    $resolved = Resolve-Sprint8BRepositoryPath -Path $Path
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $resolved -SidecarPath "$resolved.sha256")) {
        throw "Sprint 8B live UAT requires an authenticated fixture receipt pair."
    }
    $receipt = Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json -Depth 100
    if ([string]$receipt.sprint -cne "sprint-8b" -or
        [string]$receipt.state -cne "passed" -or
        [string]$receipt.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$receipt.compose_project -cne $ExpectedComposeProject -or
        [string]$receipt.restoration.state -cne "passed") {
        throw "Sprint 8B fixture receipt does not bind the expected healthy '$ExpectedComposeProject' topology."
    }
    $responseFixtureProof = Assert-Sprint8BPreparedResponseFixtures -FixtureReceipt $receipt
    [pscustomobject][ordered]@{
        path = $resolved
        sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
        document = $receipt
        response_fixtures = $responseFixtureProof
    }
}

function Get-Sprint8BUatChildComposeProject {
    param(
        [Parameter(Mandatory)][string]$ParentComposeProject,
        [Parameter(Mandatory)][string]$PredicateId
    )

    Assert-Sprint8BComposeProject -ComposeProject $ParentComposeProject | Out-Null
    $digest = (Get-Sprint7ASha256 -Text "$ParentComposeProject`n$PredicateId`n").Substring(0, 12)
    $child = "tessara-s8b-uat-$digest"
    Assert-Sprint8BComposeProject -ComposeProject $child | Out-Null
    $child
}

function Get-Sprint8BUatPredicateTopologyPlan {
    param(
        [Parameter(Mandatory)]$Predicate,
        [Parameter(Mandatory)][string]$ParentComposeProject
    )

    $topology = [string]$Predicate.topology
    $requiresReference = $topology -in @("existing-reference", "existing-or-owned-reference")
    $ownsCleanLane = $topology -ceq "owned-clean-lane"
    [pscustomobject][ordered]@{
        predicate_id = [string]$Predicate.id
        topology = $topology
        compose_project = if ($ownsCleanLane) {
            Get-Sprint8BUatChildComposeProject -ParentComposeProject $ParentComposeProject `
                -PredicateId ([string]$Predicate.id)
        } else {
            $ParentComposeProject
        }
        requires_reference = $requiresReference
        materialize_reference_on_demand = $requiresReference
        reference_preflight_required = $requiresReference
        owns_clean_lane = $ownsCleanLane
    }
}

function Assert-Sprint8BUatReferenceTopologyReady {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject,
        [Parameter(Mandatory)][string]$GatewayUrl,
        [Parameter(Mandatory)]$Fixture
    )

    Assert-Sprint8BExistingTopology -ComposePath $ComposePath `
        -ComposeProject $ComposeProject | Out-Null
    $probe = Invoke-Sprint8BHttpProbe -Uri "$($GatewayUrl.TrimEnd('/'))/health" `
        -ExpectedStatus @(200)
    Assert-Sprint8BFixtureReceipt -Path ([string]$Fixture.path) `
        -ExpectedComposeProject $ComposeProject | Out-Null
    [pscustomobject][ordered]@{
        state = "passed"
        compose_project = $ComposeProject
        gateway = [pscustomobject][ordered]@{
            status = [int]$probe.status
            content_type = [string]$probe.content_type
            body_sha256 = [string]$probe.body_sha256
        }
        fixture_sha256 = [string]$Fixture.sha256
    }
}

function Set-Sprint8BUatMaterializedTopologyEnvironment {
    param(
        [Parameter(Mandatory)]$MaterializationReceipt,
        [Parameter(Mandatory)][string]$ExpectedComposeProject
    )

    if ([string]$MaterializationReceipt.compose_project -cne $ExpectedComposeProject -or
        $null -eq $MaterializationReceipt.environment -or
        [string]$MaterializationReceipt.environment.COMPOSE_PROJECT_NAME -cne $ExpectedComposeProject) {
        throw "Owned UAT reference materialization substituted the Compose project identity."
    }
    $portValues = @(
        [string]$MaterializationReceipt.environment.TESSARA_GATEWAY_PORT,
        [string]$MaterializationReceipt.environment.TESSARA_CORE_CONTROL_PORT,
        [string]$MaterializationReceipt.environment.TESSARA_SUPERVISOR_PORT
    )
    if (@($portValues | Where-Object { $_ -cnotmatch '^[0-9]{4,5}$' }).Count -ne 0) {
        throw "Owned UAT reference materialization did not publish exact port identities."
    }

    Set-Sprint8BComposeEnvironment -ComposeProject $ExpectedComposeProject `
        -GatewayPort ([int]$portValues[0]) -CorePort ([int]$portValues[1]) `
        -SupervisorPort ([int]$portValues[2])
}

function Invoke-Sprint8BUatPredicate {
    param(
        [Parameter(Mandatory)]$Predicate,
        [Parameter(Mandatory)][string]$ResolvedComposeProject,
        [Parameter(Mandatory)][string]$PredicateComposeProject,
        [Parameter(Mandatory)][string]$ChildEvidenceRoot,
        [Parameter(Mandatory)][AllowEmptyString()][string]$ResolvedFixtureReceiptPath,
        [Parameter(Mandatory)][bool]$ExistingTopology,
        [Parameter(Mandatory)][bool]$ResetAuthorized,
        [Parameter(Mandatory)][bool]$BuildSkipped
    )

    $started = [DateTimeOffset]::UtcNow
    $childEvidence = Join-Path $ChildEvidenceRoot "$($Predicate.id).json"
    $exactTestEvidence = $null
    $commandResult = switch ([string]$Predicate.kind) {
        "script" {
            $arguments = @($Predicate.arguments)
            if ([string]$Predicate.id -ceq "acceptance-contract") {
                $arguments += @("-EvidencePath", $childEvidence)
            }
            if ($null -ne $Predicate.PSObject.Properties['evidence_contract']) {
                $arguments += @("-EvidencePath", $childEvidence)
            }
            Invoke-Sprint8BChildScript -ScriptPath ([string]$Predicate.path) -Arguments $arguments
            break
        }
        "program" {
            Invoke-Sprint8BProgramPredicate -Program ([string]$Predicate.program) `
                -Arguments @($Predicate.arguments)
            break
        }
        "readiness-target" {
            Invoke-Sprint8BChildScript -ScriptPath "scripts/run-sprint-8b-implementation-readiness.ps1" `
                -Arguments @("-Target", [string]$Predicate.target, "-EvidenceRoot", (Join-Path $ChildEvidenceRoot "readiness"))
            break
        }
        "materialization" {
            $arguments = @(
                "-Target", "ReferenceNoOp",
                "-ComposeProject", $PredicateComposeProject,
                "-EvidencePath", $childEvidence,
                "-AuthorizeDisposableReset"
            )
            if ($BuildSkipped) { $arguments += "-SkipBuild" }
            Invoke-Sprint8BChildScript -ScriptPath "scripts/materialize-sprint-8b.ps1" -Arguments $arguments
            break
        }
        "deployed-smoke" {
            $arguments = @(
                "-ComposeProject", $ResolvedComposeProject,
                "-EvidencePath", $childEvidence,
                "-FixtureReceiptPath", $ResolvedFixtureReceiptPath
            )
            if ($ExistingTopology) { $arguments += "-UseExistingTopology" } else { $arguments += "-AuthorizeDisposableReset" }
            if ($BuildSkipped) { $arguments += "-SkipBuild" }
            Invoke-Sprint8BChildScript -ScriptPath "scripts/run-sprint-8b-deployed-smoke.ps1" -Arguments $arguments
            break
        }
        "failure-recovery" {
            if (-not $ResetAuthorized) { throw "Failure-recovery UAT predicate requires reset authorization." }
            $arguments = @(
                "-ComposeProject", $PredicateComposeProject,
                "-EvidencePath", $childEvidence,
                "-AuthorizeDisposableReset"
            )
            if ($BuildSkipped) { $arguments += "-SkipBuild" }
            Invoke-Sprint8BChildScript -ScriptPath "scripts/run-sprint-8b-failure-containment.ps1" -Arguments $arguments
            break
        }
        "independent-upgrade-rollback" {
            if (-not $ResetAuthorized) { throw "Upgrade UAT predicate requires reset authorization." }
            $arguments = @(
                "-ComposeProject", $PredicateComposeProject,
                "-EvidencePath", $childEvidence,
                "-AuthorizeDisposableReset"
            )
            if ($BuildSkipped) { $arguments += "-SkipBuild" }
            Invoke-Sprint8BChildScript -ScriptPath "scripts/run-sprint-8b-dataset-upgrade.ps1" -Arguments $arguments
            break
        }
        default { throw "Unknown Sprint 8B UAT predicate kind '$($Predicate.kind)'." }
    }
    if ($null -ne $Predicate.PSObject.Properties['evidence_contract']) {
        $exactTestEvidence = Assert-Sprint8BExactTestPredicateEvidence `
            -Predicate $Predicate -EvidencePath $childEvidence
    }
    [pscustomobject][ordered]@{
        predicate_id = [string]$Predicate.id
        command_kind = [string]$Predicate.kind
        topology = [string]$Predicate.topology
        compose_project = $PredicateComposeProject
        started_at = $started.ToString("o")
        finished_at = [DateTimeOffset]::UtcNow.ToString("o")
        exit_code = [int]$commandResult.exit_code
        state = "passed"
        command = [pscustomobject][ordered]@{
            program = if ($null -ne $commandResult.PSObject.Properties['program']) {
                [string]$commandResult.program
            } else {
                [string]$commandResult.script
            }
            arguments = @($commandResult.arguments)
        }
        output_sha256 = Get-Sprint7ASha256 -Text ((@($commandResult.output) -join "`n") + "`n")
        evidence_path = if (Test-Path -LiteralPath $childEvidence -PathType Leaf) { $childEvidence } else { $null }
        exact_test_evidence = $exactTestEvidence
    }
}

if ($SelfTest) {
    $result = Test-Sprint8BUatPredicateReadiness
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8BHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result | ConvertTo-Json -Depth 50
    return
}

$scenarioContract = Get-Content -LiteralPath $scenarioContractPath -Raw | ConvertFrom-Json -Depth 100
$predicateCatalog = @(Get-Sprint8BUatPredicateCatalog)
$assertionMap = Get-Sprint8BUatAssertionMap
$evidenceClaims = Get-Sprint8BUatAssertionEvidenceClaims
$assertionProofContract = @(Get-Sprint8BUatAssertionProofContract `
    -ScenarioContract $scenarioContract -AssertionMap $assertionMap `
    -EvidenceClaims $evidenceClaims)
$assertionProofContractHash = Get-Sprint7ASha256 -Text (
    ($assertionProofContract | ConvertTo-Json -Depth 100 -Compress) + "`n"
)
Assert-Sprint8BUatPredicateContract -ScenarioContract $scenarioContract `
    -PredicateCatalog $predicateCatalog -AssertionMap $assertionMap -EvidenceClaims $evidenceClaims
$selectedScenarios = @(Get-Sprint8BSelectedScenarios -ScenarioContract $scenarioContract -Selections $Scenario)
$selectedPredicates = @(Get-Sprint8BPredicateSelection -SelectedScenarios $selectedScenarios `
    -AssertionMap $assertionMap -PredicateCatalog $predicateCatalog)
Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
$source = Get-Sprint8BSourceIdentity -RequireClean

$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT", "PLAYWRIGHT_BASE_URL", "TESSARA_PLAYWRIGHT_ACCEPTANCE"
)
$environmentBefore = Get-Sprint8BProcessEnvironmentSnapshot -Names $environmentNames
$ownedTopology = $false
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$predicateResults = [Collections.Generic.List[object]]::new()
$verifiedAssertionEvidence = @()
$resolvedFixture = $null
$failure = $null
$composePath = Resolve-Sprint8BRepositoryPath -Path "deploy/sprint-8b/compose.yaml"
$evidenceFullPath = Resolve-Sprint8BRepositoryPath -Path $EvidencePath
$childEvidenceRoot = Get-Sprint8BUatChildEvidenceRoot -EvidencePath $evidenceFullPath
[IO.Directory]::CreateDirectory($childEvidenceRoot) | Out-Null

try {
    if ($UseExistingTopology) {
        foreach ($name in @("TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT")) {
            if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
                throw "Existing-topology UAT requires inherited '$name'."
            }
        }
        $ports = Set-Sprint8BComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$env:TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$env:TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$env:TESSARA_SUPERVISOR_PORT)
        Assert-Sprint8BExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
        if ([string]::IsNullOrWhiteSpace($FixtureReceiptPath)) {
            throw "Existing-topology UAT requires -FixtureReceiptPath."
        }
        $resolvedFixture = Assert-Sprint8BFixtureReceipt `
            -Path $FixtureReceiptPath -ExpectedComposeProject $ComposeProject
    } else {
        Assert-Sprint8BResetAuthorization -ComposeProject $ComposeProject `
            -Authorized ([bool]$AuthorizeDisposableReset)
        $ports = Set-Sprint8BComposeEnvironment -ComposeProject $ComposeProject
    }
    $env:PLAYWRIGHT_BASE_URL = $ports.gateway_url
    $env:TESSARA_PLAYWRIGHT_ACCEPTANCE = "1"

    foreach ($predicate in $selectedPredicates) {
        $topologyPlan = Get-Sprint8BUatPredicateTopologyPlan -Predicate $predicate `
            -ParentComposeProject $ComposeProject
        $requiresFixture = [bool]$topologyPlan.requires_reference
        if ($requiresFixture -and $null -eq $resolvedFixture -and -not $UseExistingTopology) {
            $materializationEvidence = Join-Path $childEvidenceRoot "reference-materialization.json"
            if (Test-Path -LiteralPath $materializationEvidence) {
                throw "Owned UAT reference materialization evidence already exists: $materializationEvidence"
            }
            $arguments = @(
                "-Target", "ReferenceNoOp", "-ComposeProject", $ComposeProject,
                "-EvidencePath", $materializationEvidence, "-AuthorizeDisposableReset", "-KeepTopology"
            )
            if ($SkipBuild) { $arguments += "-SkipBuild" }
            # Ownership begins before launch because the child may create resources and then fail.
            $ownedTopology = $true
            Invoke-Sprint8BChildScript -ScriptPath "scripts/materialize-sprint-8b.ps1" `
                -Arguments $arguments | Out-Null
            $materialized = Get-Content -LiteralPath $materializationEvidence -Raw | ConvertFrom-Json -Depth 100
            if ([string]$materialized.state -cne "passed" -or
                [string]::IsNullOrWhiteSpace([string]$materialized.fixture_receipt_path)) {
                throw "Owned UAT reference materialization did not publish a fixture receipt."
            }
            $resolvedFixture = Assert-Sprint8BFixtureReceipt `
                -Path ([string]$materialized.fixture_receipt_path) -ExpectedComposeProject $ComposeProject
            $ports = Set-Sprint8BUatMaterializedTopologyEnvironment `
                -MaterializationReceipt $materialized -ExpectedComposeProject $ComposeProject
            $env:PLAYWRIGHT_BASE_URL = $ports.gateway_url
        }
        if ($requiresFixture -and $null -eq $resolvedFixture) {
            throw "Predicate '$($predicate.id)' requires an authenticated reference fixture receipt."
        }
        if ($requiresFixture) {
            Assert-Sprint8BUatReferenceTopologyReady -ComposePath $composePath `
                -ComposeProject $ComposeProject -GatewayUrl $ports.gateway_url `
                -Fixture $resolvedFixture | Out-Null
        }
        $predicateComposeProject = [string]$topologyPlan.compose_project
        $fixturePath = if ($null -eq $resolvedFixture) { "" } else { [string]$resolvedFixture.path }
        $predicateResults.Add((Invoke-Sprint8BUatPredicate -Predicate $predicate `
            -ResolvedComposeProject $ComposeProject -PredicateComposeProject $predicateComposeProject `
            -ChildEvidenceRoot $childEvidenceRoot `
            -ResolvedFixtureReceiptPath $fixturePath -ExistingTopology ([bool]$UseExistingTopology -or $ownedTopology) `
            -ResetAuthorized ([bool]$AuthorizeDisposableReset) -BuildSkipped ([bool]$SkipBuild)))
    }
    $verifiedAssertionEvidence = @(Assert-Sprint8BSelectedAssertionEvidence `
        -SelectedScenarios $selectedScenarios -EvidenceClaims $evidenceClaims `
        -PredicateResults @($predicateResults))
    if ($UseExistingTopology -or $ownedTopology) {
        Assert-Sprint8BExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
        if ($null -ne $resolvedFixture) {
            Assert-Sprint8BFixtureReceipt -Path $resolvedFixture.path `
                -ExpectedComposeProject $ComposeProject | Out-Null
        }
    }
} catch {
    $failure = $_
} finally {
    try {
        if ($ownedTopology) {
            $cleanup = Remove-Sprint8BProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized ([bool]$AuthorizeDisposableReset)
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
        } elseif ($UseExistingTopology) {
            Assert-Sprint8BExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"
                mode = "existing-topology-restored-and-retained"
            }
        } else {
            $resources = Assert-Sprint8BProjectAbsent -ComposeProject $ComposeProject
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"
                mode = "child-owned-clean-lane"
                remaining = $resources
            }
        }
    } catch {
        if ($null -eq $failure) { $failure = $_ }
        $cleanup = [pscustomobject][ordered]@{
            state = "failed"
            error = $_.Exception.Message
        }
    } finally {
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$scenarioContractHash = (Get-FileHash -LiteralPath $scenarioContractPath -Algorithm SHA256).Hash.ToLowerInvariant()
$fixtureHash = if ($null -eq $resolvedFixture) { "none" } else { [string]$resolvedFixture.sha256 }
$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n$scenarioContractHash`n$fixtureHash`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    proof = "uat-automated-predicates"
    state = if ($null -eq $failure -and [string]$cleanup.state -ceq "passed") { "passed" } else { "failed" }
    manual_acceptance_claimed = $false
    manual_acceptance_required = $true
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    scenario_contract_sha256 = $scenarioContractHash
    assertion_proof_contract_sha256 = $assertionProofContractHash
    fixture_receipt_sha256 = if ($fixtureHash -ceq "none") { $null } else { $fixtureHash }
    selected_scenarios = @($selectedScenarios.id)
    predicates = @($predicateResults)
    scenario_assertions = @(ConvertTo-Sprint8BUatScenarioAssertionResults `
        -SelectedScenarios $selectedScenarios -AssertionMap $assertionMap `
        -EvidenceClaims $evidenceClaims `
        -VerifiedAssertionEvidence @($verifiedAssertionEvidence) `
        -PredicateExecutionPassed ($null -eq $failure))
    verified_assertion_evidence = @($verifiedAssertionEvidence)
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
        message = $failure.Exception.Message
        category = [string]$failure.CategoryInfo.Category
    } }
}
Publish-Sprint8BHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8B automated UAT predicates failed; retained evidence: $evidenceFullPath"
}

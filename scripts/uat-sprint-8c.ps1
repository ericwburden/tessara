[CmdletBinding()]
param(
    [ValidateSet(
        "All",
        "UAT-8C-01", "UAT-8C-02", "UAT-8C-03", "UAT-8C-04",
        "UAT-8C-05", "UAT-8C-06", "UAT-8C-07", "UAT-8C-08",
        "UAT-8C-09", "UAT-8C-10", "UAT-8C-11"
    )]
    [string[]]$Scenario = @("All"),
    [string]$ComposeProject = "tessara-s8c-uat-scripted",
    [switch]$UseExistingTopology,
    [string]$FixtureReceiptPath,
    [string]$EvidencePath = "target/sprint-8c-uat/predicate-result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidencePathWasExplicit = $PSBoundParameters.ContainsKey("EvidencePath")
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")
. (Join-Path $PSScriptRoot "sprint-8c-uat-predicate-inventory.ps1")
$SelfTest = $requestedSelfTest
$scenarioContractPath = Join-Path $repoRoot "docs/sprints/sprint-8c-uat/scenario-contract.json"
$script:Sprint8CRefreshTestIdentities = @(
    "unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state",
    "ordered_fixed_bound_pages_promote_each_response_change_once",
    "interrupted_page_attempt_retry_converges_once_from_published_cursor",
    "concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay",
    "expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement",
    "refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding",
    "derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure",
    "refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing"
)
$script:Sprint8CDagTestIdentities = @(
    "candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt",
    "rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact",
    "downstream_materialization_failure_rolls_back_every_rebuilt_table"
)
$script:Sprint8CResponseOwnerInventory = Get-Sprint8CResponseOwnerTestInventory
$script:Sprint8CWorkflowEventInventory = @(Get-Sprint8CWorkflowEventTestInventory)
$script:Sprint8CWorkflowEventTestIdentities = @(
    $script:Sprint8CWorkflowEventInventory | ForEach-Object { @($_.identities) }
)

function Get-Sprint8CUatPredicateCatalog {
    @(
        [pscustomobject][ordered]@{
            id = "acceptance-contract"; kind = "script"
            path = "scripts/sprint-8c-acceptance-contract.ps1"; arguments = @()
            topology = "none"
        },
        [pscustomobject][ordered]@{
            id = "dataset-refresh-orchestration"; kind = "script"
            path = "scripts/test-sprint-8c-dataset-module.ps1"; arguments = @("-Suite", "Refresh")
            topology = "isolated-database"
            evidence_contract = [pscustomobject][ordered]@{
                proof = "dataset-module-test-suite"
                suite = "Refresh"
                test_binary = "refresh_integration"
                expected_test_identities = @($script:Sprint8CRefreshTestIdentities)
            }
        },
        [pscustomobject][ordered]@{
            id = "dataset-dag"; kind = "script"
            path = "scripts/test-sprint-8c-dataset-module.ps1"; arguments = @("-Suite", "Dag")
            topology = "isolated-database"
            evidence_contract = [pscustomobject][ordered]@{
                proof = "dataset-module-test-suite"
                suite = "Dag"
                test_binary = "dependency_dag_integration"
                expected_test_identities = @($script:Sprint8CDagTestIdentities)
            }
        },
        [pscustomobject][ordered]@{
            id = "component-consumer"; kind = "script"
            path = "scripts/test-sprint-8c-component-consumer.ps1"; arguments = @()
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "core-subtraction"; kind = "readiness-target"
            target = "core-subtraction"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "provider-boundaries"; kind = "readiness-target"
            target = "provider-contracts"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "api-idempotency"; kind = "readiness-target"
            target = "api-idempotency"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "response-owner"; kind = "script"
            path = "scripts/test-sprint-8c-response-module.ps1"; arguments = @("-Suite", "Owner")
            topology = "isolated-database"
            evidence_contract = [pscustomobject][ordered]@{
                kind = "exact-identities"; proof = "response-module-test-suite"
                package = "tessara-response-module"; test_binary = "owner_persistence"
                expected_test_identities = @($script:Sprint8CResponseOwnerInventory.identities)
            }
        },
        [pscustomobject][ordered]@{
            id = "response-bootstrap"; kind = "script"
            path = "scripts/test-sprint-8c-response-module.ps1"; arguments = @("-Suite", "Bootstrap")
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "response-provider"; kind = "script"
            path = "scripts/test-sprint-8c-response-module.ps1"; arguments = @("-Suite", "Provider")
            topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "workflow-events"; kind = "script"
            path = "scripts/test-sprint-8c-workflow-events.ps1"; arguments = @()
            topology = "isolated-database"
            evidence_contract = [pscustomobject][ordered]@{
                kind = "exact-runs"; proof = "workflow-response-event-consumption"
                expected_runs = @($script:Sprint8CWorkflowEventInventory)
                expected_test_identities = @($script:Sprint8CWorkflowEventTestIdentities)
            }
        },
        [pscustomobject][ordered]@{
            id = "dataset-export"; kind = "script"
            path = "scripts/test-sprint-8c-dataset-export-contract.ps1"; arguments = @()
            topology = "isolated-database"
            child_evidence_root_argument = "-EvidenceRoot"
            evidence_contract = [pscustomobject][ordered]@{
                kind = "summary"; proof = "response-owner-to-dataset-export-boundary"
                expected_counts = [pscustomobject][ordered]@{
                    response_owner_tests = @($script:Sprint8CResponseOwnerInventory.identities).Count
                    dataset_sync_tests = 7; dataset_refresh_tests = 8
                }
            }
        },
        [pscustomobject][ordered]@{
            id = "response-boundaries"; kind = "script"
            path = "scripts/check-sprint-8c-response-boundaries.ps1"; arguments = @("-Mode", "RequireClean")
            topology = "none"
        },
        [pscustomobject][ordered]@{
            id = "assignment-only"; kind = "readiness-target"
            target = "assignment-only"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "scoped-review"; kind = "readiness-target"
            target = "scoped-review"; topology = "isolated-database"
        },
        [pscustomobject][ordered]@{
            id = "response-ui-conformance"; kind = "readiness-target"
            target = "ui-sdk-conformance"; topology = "existing-reference"
        },
        [pscustomobject][ordered]@{
            id = "browser-responses"; kind = "program"; program = "npm"
            arguments = @("--prefix", "end2end", "test", "--", "tests/permissions.spec.ts", "tests/workflow-mediated-assignments.spec.ts", "--grep", "response|submission")
            topology = "existing-reference"
        },
        [pscustomobject][ordered]@{
            id = "materialization-noop"; kind = "materialization"
            topology = "owned-clean-lane"
            evidence_contract = [pscustomobject][ordered]@{
                kind = "structured-receipt"
                receipt_type = "materialization-noop"
                proof = "clean-owner-materialization-and-semantic-noop"
                target = "ReferenceNoOp"
                owner_order = @(
                    "core", "tessara.responses", "tessara.datasets",
                    "tessara.components", "tessara.dashboards",
                    "tessara.reference.scoped-records"
                )
            }
        },
        [pscustomobject][ordered]@{
            id = "deployed-smoke"; kind = "deployed-smoke"
            topology = "existing-or-owned-reference"
        },
        [pscustomobject][ordered]@{
            id = "failure-recovery"; kind = "failure-recovery"
            topology = "owned-clean-lane"
            evidence_contract = [pscustomobject][ordered]@{
                kind = "structured-receipt"
                receipt_type = "failure-recovery"
                proof = "deterministic-failure-containment-retry-and-restoration"
                expected_fault_keys = @(
                    "response.bootstrap.mid-apply",
                    "response.incompatible",
                    "dataset.derived-rebuild"
                )
            }
        },
        [pscustomobject][ordered]@{
            id = "independent-upgrade-rollback"; kind = "independent-upgrade-rollback"
            topology = "owned-clean-lane"
            evidence_contract = [pscustomobject][ordered]@{
                kind = "structured-receipt"
                receipt_type = "response-upgrade-rollback"
                proof = "independent-response-upgrade-rollback-restoration"
                module_definition = "tessara.responses"
                intended_release = "1.0.0"
                release_sequence = @("0.9.0", "1.0.0", "0.9.0", "1.0.0")
                stage_sequence = @(
                    "establish-compatible-baseline",
                    "upgrade-to-candidate",
                    "rollback-to-compatible-baseline",
                    "restore-intended-candidate"
                )
            }
        }
    )
}

function Get-Sprint8CUatAssertionMap {
    return [ordered]@{
        "UAT-8C-01" = [ordered]@{
            "assigned start, save, resume, submit, and review preserve accepted UI behavior" =
                @("acceptance-contract", "response-owner", "response-bootstrap", "browser-responses", "response-ui-conformance")
        }
        "UAT-8C-02" = [ordered]@{
            "only an exact active assignment starts a Response" = @("assignment-only", "browser-responses")
            "typed FormVersion snapshot renders without provider storage access" = @("response-provider", "response-bootstrap")
        }
        "UAT-8C-03" = [ordered]@{
            "owner, delegate, and scoped manager visibility is exact" = @("scoped-review", "browser-responses")
            "known and random forbidden identities are indistinguishable" = @("scoped-review", "response-owner")
        }
        "UAT-8C-04" = [ordered]@{
            response_owner_exact = @("response-owner")
            workflow_events_exact = @("workflow-events")
            response_export_boundary_exact = @("dataset-export")
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
        "UAT-8C-05" = [ordered]@{
            "identical mutation replay is exact" = @("api-idempotency", "response-owner")
            "binding faults retain last-good state and recover" =
                @("provider-boundaries", "failure-recovery", "deployed-smoke")
        }
        "UAT-8C-06" = [ordered]@{
            "administrator Module Management validates and applies Response configuration" = @("deployed-smoke")
            "administrator diagnostics are sanitized and owner-authentic while the operator status projection remains available without Module Management access" = @("deployed-smoke", "response-provider")
        }
        "UAT-8C-07" = [ordered]@{
            "from-empty owner order returns signed read-back" = @("materialization-noop", "response-bootstrap")
            "unchanged apply is a semantic no-op" = @("materialization-noop")
        }
        "UAT-8C-08" = [ordered]@{
            "Core has no Response schema, routes, adapters, or seed" = @("response-boundaries", "core-subtraction")
            "pairwise database credentials are denied" = @("core-subtraction", "deployed-smoke")
        }
        "UAT-8C-09" = [ordered]@{
            "failed apply is retained" = @("failure-recovery")
            "exact teardown permits a clean successor from-empty apply" = @("failure-recovery")
        }
        "UAT-8C-10" = [ordered]@{
            "Response-only upgrade, rollback, and restore preserve state and unrelated owners" =
                @("independent-upgrade-rollback")
        }
        "UAT-8C-11" = [ordered]@{
            "complete and review a Response" = @("response-owner", "browser-responses")
            "consume submitted output through Dataset, Component, and Dashboard without shared database access" =
                @("workflow-events", "dataset-export", "component-consumer", "deployed-smoke")
        }
    }
}

function Get-Sprint8CUatAssertionEvidenceClaims {
    [ordered]@{
        "UAT-8C-04" = [ordered]@{
            response_owner_exact = @(
                [pscustomobject][ordered]@{
                    predicate_id = "response-owner"
                    test_identity = "create_is_atomic_audited_evented_and_idempotent"
                }
            )
            workflow_events_exact = @(
                [pscustomobject][ordered]@{
                    predicate_id = "workflow-events"
                    test_identity = "workflow_response_consumer::tests::autonomous_consumer_recovers_owner_start_save_submit_backlog_while_unready"
                }
            )
            response_export_boundary_exact = @(
                [pscustomobject][ordered]@{
                    predicate_id = "dataset-export"
                    test_identity = "summary"
                }
            )
            unchanged_head_no_page = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[0] })
            ordered_changes_once = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[1] })
            interrupt_retry = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[2] })
            concurrent_refresh = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[3] })
            expired_cursor_rebase = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[4] })
            atomic_dataset_closure = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[5] })
            independent_binding_unchanged = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[5] })
            cycle_rejected = @([pscustomobject][ordered]@{ predicate_id = "dataset-dag"; test_identity = $script:Sprint8CDagTestIdentities[0] })
            derived_failure_rolls_back = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[6] })
            nondisclosure = @([pscustomobject][ordered]@{ predicate_id = "dataset-refresh-orchestration"; test_identity = $script:Sprint8CRefreshTestIdentities[7] })
        }
    }
}

function Get-Sprint8CUatAssertionProofContract {
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

function Assert-Sprint8CUatPredicateContract {
    param(
        [Parameter(Mandatory)]$ScenarioContract,
        [Parameter(Mandatory)][object[]]$PredicateCatalog,
        [Parameter(Mandatory)]$AssertionMap,
        [Parameter(Mandatory)]$EvidenceClaims
    )

    $fixtureContract = Get-Content -LiteralPath (
        Join-Path $repoRoot "deploy/sprint-8c/fixtures/reference-fixture-contract.json"
    ) -Raw | ConvertFrom-Json -Depth 100
    if ([int]$fixtureContract.schema_version -ne 1 -or
        [string]$fixtureContract.contract -cne "tessara.sprint-8c.reference-fixture") {
        throw "Sprint 8C UAT cannot authenticate the canonical logical fixture-key inventory."
    }
    $knownActorKeys = @($fixtureContract.actors.key | ForEach-Object { [string]$_ })
    $knownFixtureKeys = @(
        @($fixtureContract.form_versions.key)
        @($fixtureContract.response_changes.key)
        @($fixtureContract.source_bindings.key)
        @($fixtureContract.datasets.key)
        @($fixtureContract.downstream.components.key)
        [string]$fixtureContract.downstream.dashboard.key
    ) | ForEach-Object { [string]$_ }
    $allowedCleanupModes = @(
        "restore-reference-fixture", "restore-provider-proxies",
        "restore-canonical-topology", "clear-faults-and-restore-canonical-topology",
        "restore-reference-configuration", "restore-response-1.0.0"
    )
    $expectedScenarioIds = @(1..11 | ForEach-Object { "UAT-8C-{0:d2}" -f $_ })
    $actualScenarioIds = @($ScenarioContract.scenarios.id)
    if ([int]$ScenarioContract.schema_version -ne 1 -or
        [string]$ScenarioContract.contract -cne "tessara.sprint-8c.uat-scenarios" -or
        ($actualScenarioIds -join "`n") -cne ($expectedScenarioIds -join "`n") -or
        (@($AssertionMap.Keys) -join "`n") -cne ($expectedScenarioIds -join "`n")) {
        throw "Sprint 8C UAT predicate contract is not set-equal to the eleven canonical scenarios."
    }
    $predicateIds = @($PredicateCatalog.id)
    if (@($predicateIds | Sort-Object -Unique).Count -ne $predicateIds.Count) {
        throw "Sprint 8C UAT predicate IDs are not unique."
    }
    $referencedPredicateIds = @(
        foreach ($scenarioId in @($AssertionMap.Keys)) {
            foreach ($assertion in @($AssertionMap[$scenarioId].Keys)) {
                @($AssertionMap[$scenarioId][$assertion])
            }
        }
    ) | ForEach-Object { [string]$_ } | Sort-Object -Unique
    if ((@($predicateIds | Sort-Object) -join "`n") -cne
        (@($referencedPredicateIds) -join "`n")) {
        throw "Sprint 8C UAT predicate catalog is not set-equal to the predicates referenced by canonical assertions."
    }
    $readinessTargets = @(& pwsh -NoProfile -File (
        Join-Path $repoRoot "scripts/run-sprint-8c-implementation-readiness.ps1"
    ) -ListTargets 2>&1 | ForEach-Object { [string]$_ })
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 8C UAT could not discover implementation-readiness targets."
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
            throw "Sprint 8C UAT predicate '$($predicate.id)' has no executable command/topology contract."
        }
        if ([string]$predicate.kind -ceq "script" -and
            -not (Test-Path -LiteralPath (Join-Path $repoRoot ([string]$predicate.path)) -PathType Leaf)) {
            throw "Sprint 8C UAT predicate '$($predicate.id)' references a missing script."
        }
        if ([string]$predicate.kind -ceq "readiness-target" -and
            $readinessTargets -cnotcontains [string]$predicate.target) {
            throw "Sprint 8C UAT predicate '$($predicate.id)' references an unknown readiness target."
        }
        $childEvidenceRootProperty = $predicate.PSObject.Properties['child_evidence_root_argument']
        if ($null -ne $childEvidenceRootProperty -and (
                [string]$predicate.kind -cne "script" -or
                [string]$childEvidenceRootProperty.Value -cne "-EvidenceRoot")) {
            throw "Sprint 8C UAT predicate '$($predicate.id)' has an invalid child evidence-root contract."
        }
        $evidenceProperty = $predicate.PSObject.Properties['evidence_contract']
        if ($null -ne $evidenceProperty) {
            $evidenceContract = $evidenceProperty.Value
            $kindProperty = $evidenceContract.PSObject.Properties['kind']
            $evidenceKind = if ($null -eq $kindProperty) { "legacy-exact-identities" } else {
                [string]$kindProperty.Value
            }
            $identityProperty = $evidenceContract.PSObject.Properties['expected_test_identities']
            $expectedTestIdentities = if ($null -eq $identityProperty) { @() } else {
                @($identityProperty.Value | ForEach-Object { [string]$_ })
            }
            $legacyValid = $evidenceKind -ceq "legacy-exact-identities" -and
                [string]$predicate.kind -ceq "script" -and
                [string]$evidenceContract.proof -ceq "dataset-module-test-suite" -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.suite) -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.test_binary)
            $exactValid = $evidenceKind -ceq "exact-identities" -and
                [string]$predicate.kind -ceq "script" -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.proof) -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.package) -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.test_binary)
            $exactRunsValid = $false
            if ($evidenceKind -ceq "exact-runs" -and
                [string]$predicate.kind -ceq "script" -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.proof)) {
                $expectedRuns = @($evidenceContract.expected_runs)
                $runIdentities = @($expectedRuns | ForEach-Object { @($_.identities) })
                $exactRunsValid = $expectedRuns.Count -gt 0 -and
                    $runIdentities.Count -eq $expectedTestIdentities.Count -and
                    ($runIdentities -join "`n") -ceq ($expectedTestIdentities -join "`n") -and
                    @($expectedRuns.label | Sort-Object -Unique).Count -eq $expectedRuns.Count -and
                    @($expectedRuns | Where-Object {
                        [string]::IsNullOrWhiteSpace([string]$_.label) -or
                        [string]::IsNullOrWhiteSpace([string]$_.package) -or
                        [string]::IsNullOrWhiteSpace([string]$_.test_binary) -or
                        @($_.arguments).Count -eq 0 -or @($_.identities).Count -eq 0 -or
                        @($_.identities | Sort-Object -Unique).Count -ne @($_.identities).Count
                    }).Count -eq 0
            }
            $summaryValid = $evidenceKind -ceq "summary" -and
                [string]$predicate.kind -ceq "script" -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.proof) -and
                $null -ne $evidenceContract.PSObject.Properties['expected_counts'] -and
                @($evidenceContract.expected_counts.PSObject.Properties).Count -gt 0
            $structuredValid = $false
            if ($evidenceKind -ceq "structured-receipt" -and
                -not [string]::IsNullOrWhiteSpace([string]$evidenceContract.proof)) {
                $structuredValid = switch ([string]$evidenceContract.receipt_type) {
                    "materialization-noop" {
                        $owners = @($evidenceContract.owner_order | ForEach-Object { [string]$_ })
                        [string]$predicate.kind -ceq "materialization" -and
                            [string]$evidenceContract.target -ceq "ReferenceNoOp" -and
                            $owners.Count -gt 0 -and
                            @($owners | Sort-Object -Unique).Count -eq $owners.Count
                        break
                    }
                    "failure-recovery" {
                        $faultKeys = @($evidenceContract.expected_fault_keys | ForEach-Object { [string]$_ })
                        [string]$predicate.kind -ceq "failure-recovery" -and
                            $faultKeys.Count -gt 0 -and
                            @($faultKeys | Sort-Object -Unique).Count -eq $faultKeys.Count
                        break
                    }
                    "response-upgrade-rollback" {
                        $sequence = @($evidenceContract.release_sequence | ForEach-Object { [string]$_ })
                        [string]$predicate.kind -ceq "independent-upgrade-rollback" -and
                            [string]$evidenceContract.module_definition -ceq "tessara.responses" -and
                            [string]$evidenceContract.intended_release -ceq "1.0.0" -and
                            ($sequence -join "`n") -ceq (@("0.9.0", "1.0.0", "0.9.0", "1.0.0") -join "`n")
                        break
                    }
                    default { $false }
                }
            }
            if ((-not $legacyValid -and -not $exactValid -and -not $exactRunsValid -and
                    -not $summaryValid -and -not $structuredValid) -or
                (($legacyValid -or $exactValid -or $exactRunsValid) -and (
                    $expectedTestIdentities.Count -eq 0 -or
                    @($expectedTestIdentities | Sort-Object -Unique).Count -ne $expectedTestIdentities.Count
                ))) {
                throw "Sprint 8C UAT predicate '$($predicate.id)' has an invalid exact test-evidence contract."
            }
        }
    }
    $datasetExportPredicate = @($PredicateCatalog | Where-Object {
        [string]$_.id -ceq "dataset-export"
    })
    if ($datasetExportPredicate.Count -ne 1 -or
        $null -eq $datasetExportPredicate[0].PSObject.Properties['child_evidence_root_argument'] -or
        [string]$datasetExportPredicate[0].child_evidence_root_argument -cne "-EvidenceRoot") {
        throw "Sprint 8C UAT Dataset-export must bind nested receipts to its run-scoped child evidence root."
    }
    foreach ($scenario in @($ScenarioContract.scenarios)) {
        $actorKeys = @($scenario.actor_keys | ForEach-Object { [string]$_ })
        $fixtureKeys = @($scenario.fixture_keys | ForEach-Object { [string]$_ })
        if ([string]::IsNullOrWhiteSpace([string]$scenario.start_state) -or
            [string]::IsNullOrWhiteSpace([string]$scenario.cleanup) -or
            [string]$scenario.cleanup_mode -notin $allowedCleanupModes -or
            $actorKeys.Count -eq 0 -or $fixtureKeys.Count -eq 0 -or
            @($actorKeys | Sort-Object -Unique).Count -ne $actorKeys.Count -or
            @($fixtureKeys | Sort-Object -Unique).Count -ne $fixtureKeys.Count -or
            @($actorKeys | Where-Object { $knownActorKeys -cnotcontains $_ }).Count -ne 0 -or
            @($fixtureKeys | Where-Object { $knownFixtureKeys -cnotcontains $_ }).Count -ne 0) {
            throw "Sprint 8C UAT scenario '$($scenario.id)' lacks an executable precondition or cleanup identity."
        }
        $mapping = $AssertionMap[[string]$scenario.id]
        if ($null -eq $mapping -or
            (@($mapping.Keys | Sort-Object) -join "`n") -cne
                (@($scenario.assertions | Sort-Object) -join "`n")) {
            throw "Sprint 8C UAT scenario '$($scenario.id)' assertion predicates are not set-equal to its contract."
        }
        foreach ($assertion in @($scenario.assertions)) {
            $mappedPredicates = @($mapping[[string]$assertion])
            if ($mappedPredicates.Count -eq 0) {
                throw "Sprint 8C UAT assertion '$($scenario.id)/$assertion' has no executable predicate."
            }
            foreach ($predicateId in $mappedPredicates) {
                if ($predicateIds -cnotcontains [string]$predicateId) {
                    throw "Sprint 8C UAT assertion '$($scenario.id)/$assertion' names unknown predicate '$predicateId'."
                }
            }
        }
        $scenarioClaims = $EvidenceClaims[[string]$scenario.id]
        if ($null -ne $scenarioClaims) {
            if ((@($scenarioClaims.Keys | Sort-Object) -join "`n") -cne
                (@($scenario.assertions | Sort-Object) -join "`n")) {
                throw "Sprint 8C UAT scenario '$($scenario.id)' exact evidence claims are not set-equal to its assertions."
            }
            foreach ($assertion in @($scenario.assertions)) {
                $claims = @($scenarioClaims[[string]$assertion])
                $mappedPredicates = @($mapping[[string]$assertion])
                $claimPredicates = @($claims.predicate_id | ForEach-Object { [string]$_ })
                if ($claims.Count -eq 0 -or
                    (@($claimPredicates | Sort-Object -Unique) -join "`n") -cne
                        (@($mappedPredicates | Sort-Object -Unique) -join "`n")) {
                    throw "Sprint 8C UAT assertion '$($scenario.id)/$assertion' does not bind every mapped predicate to exact evidence."
                }
                foreach ($claim in $claims) {
                    $predicate = @($PredicateCatalog | Where-Object {
                        [string]$_.id -ceq [string]$claim.predicate_id
                    })
                    $claimContract = if ($predicate.Count -eq 1) { $predicate[0].evidence_contract } else { $null }
                    $claimKind = if ($null -ne $claimContract -and
                        $null -ne $claimContract.PSObject.Properties['kind']) {
                        [string]$claimContract.kind
                    } else { "legacy-exact-identities" }
                    $identityAccepted = switch ($claimKind) {
                        "summary" { [string]$claim.test_identity -ceq "summary"; break }
                        "structured-receipt" {
                            [string]$claim.test_identity -ceq "structured-receipt"
                            break
                        }
                        default {
                            @($claimContract.expected_test_identities) -ccontains [string]$claim.test_identity
                        }
                    }
                    if ($predicate.Count -ne 1 -or $null -eq $claimContract -or -not $identityAccepted) {
                        throw "Sprint 8C UAT assertion '$($scenario.id)/$assertion' names an unproven exact test identity."
                    }
                }
            }
        }
    }
}

function Get-Sprint8CSelectedScenarios {
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

function Get-Sprint8CPredicateSelection {
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

function Invoke-Sprint8CProgramPredicate {
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

function Assert-Sprint8CExactCargoSelector {
    param(
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$Package,
        [Parameter(Mandatory)][string]$TestBinary,
        [Parameter(Mandatory)][string]$PredicateId
    )

    $packageIndex = [Array]::IndexOf($Arguments, "-p")
    $binaryIndex = [Array]::IndexOf($Arguments, "--test")
    $libIndex = [Array]::IndexOf($Arguments, "--lib")
    $jobsIndex = [Array]::IndexOf($Arguments, "--jobs")
    $binarySelectorValid = if ($TestBinary -ceq "lib") {
        $libIndex -ge 0 -and $binaryIndex -lt 0
    } else {
        $binaryIndex -ge 0 -and $binaryIndex + 1 -lt $Arguments.Count -and
            $Arguments[$binaryIndex + 1] -ceq $TestBinary
    }
    if ($Arguments.Count -eq 0 -or $Arguments[0] -cne "test" -or
        $packageIndex -lt 0 -or $packageIndex + 1 -ge $Arguments.Count -or
        $Arguments[$packageIndex + 1] -cne $Package -or
        -not $binarySelectorValid -or
        $Arguments -cnotcontains "--locked" -or
        $Arguments -cnotcontains "--offline" -or
        $jobsIndex -lt 0 -or $jobsIndex + 1 -ge $Arguments.Count -or
        $Arguments[$jobsIndex + 1] -cne "1") {
        throw "Predicate '$PredicateId' evidence does not retain its exact Cargo test selector."
    }
}

function Assert-Sprint8CUatResponseHealthEvidence {
    param(
        [Parameter(Mandatory)]$Health,
        [Parameter(Mandatory)][string]$PredicateId
    )

    $liveProperty = $Health.PSObject.Properties['response_live']
    $readyProperty = $Health.PSObject.Properties['response_ready']
    if ($null -eq $liveProperty -or $null -eq $readyProperty) {
        throw "Predicate '$PredicateId' evidence does not prove the required Response health documents."
    }
    $live = $liveProperty.Value
    $ready = $readyProperty.Value
    $expectedEnvelopeProperties = @("schema_version", "status", "checks")
    foreach ($document in @($live, $ready)) {
        $actualProperties = @($document.PSObject.Properties.Name | Sort-Object)
        if (($actualProperties -join "`n") -cne
                (($expectedEnvelopeProperties | Sort-Object) -join "`n") -or
            [int]$document.schema_version -ne 1 -or
            [string]$document.status -cne "passing") {
            throw "Predicate '$PredicateId' evidence does not prove the exact passing shared Response health envelope."
        }
    }
    if (@($live.checks).Count -ne 0) {
        throw "Predicate '$PredicateId' evidence does not prove empty Response liveness checks."
    }

    $expectedReadinessCodes = @(
        "response.database",
        "response.configuration",
        "response.security_state",
        "response.provider.forms",
        "response.provider.workflow",
        "response.events.publication",
        "response.export.publication"
    )
    $readyChecks = @($ready.checks)
    if ($readyChecks.Count -ne $expectedReadinessCodes.Count -or
        (@($readyChecks.code) -join "`n") -cne ($expectedReadinessCodes -join "`n")) {
        throw "Predicate '$PredicateId' evidence does not prove the exact ordered Response readiness inventory."
    }
    foreach ($check in $readyChecks) {
        $actualProperties = @($check.PSObject.Properties.Name | Sort-Object)
        if (($actualProperties -join "`n") -cne
                ((@("code", "passing", "message") | Sort-Object) -join "`n") -or
            -not [bool]$check.passing -or
            [string]::IsNullOrWhiteSpace([string]$check.message)) {
            throw "Predicate '$PredicateId' evidence does not prove the exact passing Response readiness check shape."
        }
    }
}

function Assert-Sprint8CExactTestPredicateEvidence {
    param(
        [Parameter(Mandatory)]$Predicate,
        [Parameter(Mandatory)][string]$EvidencePath,
        [AllowEmptyString()][string]$ExpectedComposeProject = "",
        [AllowNull()]$ExpectedSource = $null
    )

    $contractProperty = $Predicate.PSObject.Properties['evidence_contract']
    if ($null -eq $contractProperty) {
        throw "Predicate '$($Predicate.id)' has no exact test-evidence contract."
    }
    $resolved = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $resolved -SidecarPath "$resolved.sha256")) {
        throw "Predicate '$($Predicate.id)' did not publish an authenticated evidence pair."
    }
    $document = Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json -Depth 100
    $contract = $contractProperty.Value
    $kind = if ($null -eq $contract.PSObject.Properties['kind']) {
        "legacy-exact-identities"
    } else { [string]$contract.kind }
    if ([int]$document.schema_version -ne 1 -or
        [string]$document.sprint -cne "sprint-8c" -or
        [string]$document.proof -cne [string]$contract.proof -or
        [string]$document.state -cne "passed") {
        throw "Predicate '$($Predicate.id)' evidence has a substituted identity or non-passing state."
    }
    if ($kind -ceq "structured-receipt") {
        if (-not [string]::IsNullOrWhiteSpace($ExpectedComposeProject) -and (
                [string]$document.compose_project -cne $ExpectedComposeProject -or
                [string]$document.source.commit -cne [string]$ExpectedSource.commit -or
                [string]$document.source.tree -cne [string]$ExpectedSource.tree -or
                [bool]$document.source.dirty -ne [bool]$ExpectedSource.dirty -or
                $null -ne $document.failure -or
                [string]$document.cleanup_restoration.state -cne "passed"
            )) {
            throw "Predicate '$($Predicate.id)' structured evidence substituted source, Compose project, cleanup, or failure state."
        }
        switch ([string]$contract.receipt_type) {
            "materialization-noop" {
                Assert-Sprint8CUatResponseHealthEvidence -Health $document.health `
                    -PredicateId ([string]$Predicate.id)
                $expectedOwners = @($contract.owner_order | ForEach-Object { [string]$_ })
                $firstOwners = @($document.first_apply.owner_order | ForEach-Object { [string]$_ })
                $noOpOwners = @($document.semantic_noop.owner_order | ForEach-Object { [string]$_ })
                $firstReceipts = @($document.first_apply.owner_receipts)
                $noOpReceipts = @($document.semantic_noop.owner_receipts)
                if ([string]$document.target -cne [string]$contract.target -or
                    [string]$document.compose_configuration_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
                    [string]$document.blueprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
                    [string]$document.first_apply.receipt_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                    [string]$document.first_apply.lockfile_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                    [string]$document.first_apply.plan_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                    [string]$document.first_apply.operation_state -cne "succeeded" -or
                    -not [bool]$document.first_apply.changed -or
                    [bool]$document.first_apply.no_op -or
                    [string]$document.semantic_noop.operation_state -cne "succeeded" -or
                    [string]$document.semantic_noop.previous_receipt_digest -cne
                        [string]$document.first_apply.receipt_digest -or
                    [uint64]$document.semantic_noop.revision -ne
                        ([uint64]$document.first_apply.revision + 1) -or
                    [bool]$document.semantic_noop.changed -or
                    -not [bool]$document.semantic_noop.no_op -or
                    ($firstOwners -join "`n") -cne ($expectedOwners -join "`n") -or
                    ($noOpOwners -join "`n") -cne ($expectedOwners -join "`n") -or
                    $firstReceipts.Count -ne $expectedOwners.Count -or
                    @($firstReceipts | Where-Object {
                        -not [bool]$_.changed -or
                        [string]::IsNullOrWhiteSpace([string]$_.schema_version) -or
                        [string]$_.input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        [string]$_.result_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        @($_.resource_ids.PSObject.Properties).Count -eq 0
                    }).Count -ne 0 -or
                    $noOpReceipts.Count -ne $expectedOwners.Count -or
                    @($noOpReceipts | Where-Object {
                        [bool]$_.changed -or
                        [string]$_.input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        [string]$_.result_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        @($_.resource_ids.PSObject.Properties).Count -eq 0
                    }).Count -ne 0 -or
                    [string]$document.semantic_noop_proof.state -cne "passed" -or
                    -not [bool]$document.semantic_noop_proof.stable_owner_receipts -or
                    -not [bool]$document.semantic_noop_proof.stable_enablement_artifacts_configuration -or
                    -not [bool]$document.semantic_noop_proof.stable_container_topology -or
                    -not [bool]$document.gateway_start_boundary.owner_apply_completed_before_start -or
                    [string]$document.gateway_start_boundary.post_start_health -cne "passed" -or
                    [string]$document.cleanup_restoration.state -cne "passed" -or
                    [string]$document.cleanup_restoration.mode -cne "exact-project-teardown" -or
                    [string]::IsNullOrWhiteSpace([string]$document.fixture_receipt_path) -or
                    [string]$document.fixture_receipt_sha256 -cnotmatch '^[0-9a-f]{64}$') {
                    throw "Predicate '$($Predicate.id)' evidence does not prove exact materialization and semantic no-op restoration."
                }
                break
            }
            "failure-recovery" {
                $expectedFaultKeys = @($contract.expected_fault_keys | ForEach-Object { [string]$_ })
                $attempts = @($document.failure_attempts)
                $actualFaultKeys = @($attempts | ForEach-Object { [string]$_.fault.fault_key })
                $expectedAttemptFields = @(
                    "child_exit_code", "containment", "correlation_id", "fault",
                    "materialization_evidence", "materialization_evidence_sha256"
                )
                $expectedFaultFields = @(
                    "attempt", "attempt_limit", "expected_failure_code", "expected_outcome",
                    "fault_key", "no_cross_owner_write", "no_unauthorized_state", "phase",
                    "receipt_contract", "target_service", "transaction_field", "transaction_value"
                )
                $expectedContainmentFields = @(
                    "fault_key", "fixture_published", "gateway_started", "materialization_state",
                    "partial_topology_teardown"
                )
                $expectedMaterializationEvidenceFields = @("path", "sha256")
                if (($actualFaultKeys -join "`n") -cne ($expectedFaultKeys -join "`n") -or
                    $attempts.Count -ne $expectedFaultKeys.Count -or
                    @($attempts | Where-Object {
                        (@($_.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                            ($expectedAttemptFields -join "`n") -or
                        (@($_.fault.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                            ($expectedFaultFields -join "`n") -or
                        (@($_.containment.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                            ($expectedContainmentFields -join "`n") -or
                        (@($_.materialization_evidence.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                            ($expectedMaterializationEvidenceFields -join "`n") -or
                        [int]$_.child_exit_code -eq 0 -or
                        [string]$_.materialization_evidence_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
                        [string]$_.materialization_evidence.sha256 -cne
                            [string]$_.materialization_evidence_sha256 -or
                        [string]::IsNullOrWhiteSpace([string]$_.materialization_evidence.path) -or
                        [string]$_.correlation_id -cnotmatch
                            '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' -or
                        [string]$_.fault.fault_key -cne [string]$_.containment.fault_key -or
                        [string]$_.fault.receipt_contract -cnotmatch
                            '^tessara\.sprint-8[bc]\.failure-control/v1$' -or
                        [string]::IsNullOrWhiteSpace([string]$_.fault.target_service) -or
                        [string]::IsNullOrWhiteSpace([string]$_.fault.phase) -or
                        [string]::IsNullOrWhiteSpace([string]$_.fault.expected_failure_code) -or
                        [string]$_.fault.expected_outcome -notin @("rejected_pre_write", "rolled_back") -or
                        [string]$_.fault.transaction_field -notin @(
                            "dataset_transaction", "response_transaction"
                        ) -or
                        [string]$_.fault.transaction_value -notin @("not_started", "rolled_back") -or
                        [string]$_.containment.materialization_state -cne "failed" -or
                        [bool]$_.containment.gateway_started -or
                        [bool]$_.containment.fixture_published -or
                        [string]$_.containment.partial_topology_teardown -cne "passed" -or
                        [int]$_.fault.attempt -ne 1 -or
                        [int]$_.fault.attempt_limit -ne 1 -or
                        -not [bool]$_.fault.no_unauthorized_state -or
                        -not [bool]$_.fault.no_cross_owner_write
                    }).Count -ne 0 -or
                    [string]$document.successor_evidence.state -cne "passed" -or
                    [string]$document.successor_evidence.target -cne "ReferenceNoOp" -or
                    [string]$document.restoration_proof.empty_start -cne "passed" -or
                    [string]$document.restoration_proof.canonical_first_apply -cne "passed" -or
                    [string]$document.restoration_proof.semantic_noop -cne "passed" -or
                    [string]$document.restoration_proof.canonical_health -cne "passed" -or
                    [string]$document.restoration_proof.final_teardown -cne "passed" -or
                    [string]$document.cleanup_restoration.state -cne "passed" -or
                    [string]$document.cleanup_restoration.mode -cne
                        "three-exact-partial-teardowns-plus-restored-successor-teardown" -or
                    [string]$document.cleanup_restoration.fault_controls -cne "cleared-to-none" -or
                    [string]$document.cleanup_restoration.empty_successor_start -cne "passed") {
                    throw "Predicate '$($Predicate.id)' evidence does not prove every bounded fault, exact teardown, and from-empty restoration."
                }
                break
            }
            "response-upgrade-rollback" {
                $expectedSequence = @($contract.release_sequence | ForEach-Object { [string]$_ })
                $expectedStages = @($contract.stage_sequence | ForEach-Object { [string]$_ })
                $contractSequence = @($document.release_contract.sequence | ForEach-Object { [string]$_ })
                $contractStages = @($document.release_contract.stages | ForEach-Object { [string]$_ })
                $transitions = @($document.transitions)
                $actualSequence = @($transitions.target_release | ForEach-Object { [string]$_ })
                $stages = @($document.stage_snapshots)
                $preservation = @($document.preservation_proofs)
                if ([string]$document.release_contract.transition.owner -cne [string]$contract.module_definition -or
                    [string]$document.release_contract.transition.intended_release -cne [string]$contract.intended_release -or
                    ($contractSequence -join "`n") -cne ($expectedSequence -join "`n") -or
                    ($contractStages -join "`n") -cne ($expectedStages -join "`n") -or
                    ($actualSequence -join "`n") -cne ($expectedSequence -join "`n") -or
                    ((@($transitions.stage | ForEach-Object { [string]$_ }) -join "`n") -cne
                        ($expectedStages -join "`n")) -or
                    $stages.Count -ne $expectedSequence.Count -or
                    $preservation.Count -ne $expectedSequence.Count -or
                    [string]$document.pre_exercise_snapshot.stage -cne "pre-exercise-candidate" -or
                    [string]$document.pre_exercise_snapshot.response.release -cne
                        [string]$contract.intended_release -or
                    ((@($stages.stage | ForEach-Object { [string]$_ }) -join "`n") -cne
                        ($expectedStages -join "`n")) -or
                    ((@($stages.response.release | ForEach-Object { [string]$_ }) -join "`n") -cne
                        ($expectedSequence -join "`n")) -or
                    ((@($preservation.stage | ForEach-Object { [string]$_ }) -join "`n") -cne
                        ($expectedStages -join "`n")) -or
                    @($transitions | Where-Object {
                        [string]$_.fixed_owner_lockfile -cne "passed" -or
                        [string]$_.target_image -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        [string]$_.target_manifest_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        [string]$_.plan_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        [string]$_.lockfile_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
                        [string]$_.receipt_lockfile_digest -cne [string]$_.lockfile_digest -or
                        [string]$_.exact_delta.owner -cne "tessara.responses" -or
                        [int]$_.exact_delta.action_count -ne 5 -or
                        -not [bool]$_.exact_delta.exact
                    }).Count -ne 0 -or
                    @($preservation | Where-Object {
                        [string]$_.state -cne "passed" -or
                        [string]$_.response_state -cne "passed" -or
                        [string]$_.module_instance_identity -cne "passed" -or
                        [string]$_.typed_resource_identity -cne "passed" -or
                        [string]$_.navigation_identity -cne "passed" -or
                        [string]$_.outbox_positions -cne "passed" -or
                        [string]$_.unrelated_owners -cne "passed"
                    }).Count -ne 0 -or
                    [string]$document.release_fixture.baseline.version -cne "0.9.0" -or
                    [string]$document.release_fixture.candidate.version -cne
                        [string]$contract.intended_release -or
                    [string]$document.release_fixture.baseline.runtime_image -ceq
                        [string]$document.release_fixture.candidate.runtime_image -or
                    [string]$document.release_fixture.baseline.executable_sha256 -ceq
                        [string]$document.release_fixture.candidate.executable_sha256 -or
                    [string]$document.release_restoration.state -cne "passed" -or
                    [string]$document.release_restoration.final_release -cne [string]$contract.intended_release -or
                    [string]$document.cleanup_restoration.state -cne "passed") {
                    throw "Predicate '$($Predicate.id)' evidence does not prove the exact Response upgrade, rollback, preservation, and restoration sequence."
                }
                break
            }
            default {
                throw "Predicate '$($Predicate.id)' evidence declares an unknown structured receipt type."
            }
        }
        return [pscustomobject][ordered]@{
            path = $resolved
            sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
            proof = [string]$document.proof
            test_identities = @("structured-receipt")
            executed_test_count = 1
            cleanup_restoration = $document.cleanup_restoration
        }
    }
    if ($kind -ceq "summary") {
        foreach ($expectedCount in @($contract.expected_counts.PSObject.Properties)) {
            $actualProperty = $document.PSObject.Properties[[string]$expectedCount.Name]
            if ($null -eq $actualProperty -or
                [uint64]$actualProperty.Value -ne [uint64]$expectedCount.Value) {
                throw "Predicate '$($Predicate.id)' evidence does not prove exact summary count '$($expectedCount.Name)'."
            }
        }
        return [pscustomobject][ordered]@{
            path = $resolved
            sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
            proof = [string]$document.proof
            test_identities = @("summary")
            executed_test_count = [uint64](($contract.expected_counts.PSObject.Properties.Value |
                Measure-Object -Sum).Sum)
            cleanup_restoration = $document.cleanup_restoration
        }
    }
    if ($kind -ceq "exact-runs") {
        $expectedRuns = @($contract.expected_runs)
        $actualRuns = @($document.runs)
        $expected = @($contract.expected_test_identities | ForEach-Object { [string]$_ })
        $declared = @($document.expected_test_identities | ForEach-Object { [string]$_ })
        $executed = @($document.executed_test_identities | ForEach-Object { [string]$_ })
        if ($expectedRuns.Count -eq 0 -or $actualRuns.Count -ne $expectedRuns.Count -or
            [int]$document.executed_test_count -ne $expected.Count -or
            (@($declared | Sort-Object -Unique) -join "`n") -cne
                (@($expected | Sort-Object -Unique) -join "`n") -or
            (@($executed | Sort-Object -Unique) -join "`n") -cne
                (@($expected | Sort-Object -Unique) -join "`n") -or
            $declared.Count -ne $expected.Count -or $executed.Count -ne $expected.Count -or
            [string]$document.database.mode -cne "disposable-postgres" -or
            [string]$document.database.cleanup_restoration.state -cne "passed") {
            throw "Predicate '$($Predicate.id)' evidence does not prove its exact multi-run test identity and cleanup contract."
        }
        for ($index = 0; $index -lt $expectedRuns.Count; $index++) {
            $expectedRun = $expectedRuns[$index]
            $actualRun = $actualRuns[$index]
            $expectedRunIdentities = @($expectedRun.identities | ForEach-Object { [string]$_ })
            $declaredRunIdentities = @($actualRun.expected_test_identities | ForEach-Object { [string]$_ })
            $executedRunIdentities = @($actualRun.executed_test_identities | ForEach-Object { [string]$_ })
            $expectedArguments = @($expectedRun.arguments | ForEach-Object { [string]$_ }) +
                @("--", "--format", "terse")
            $actualArguments = @($actualRun.arguments | ForEach-Object { [string]$_ })
            if ([string]$actualRun.label -cne [string]$expectedRun.label -or
                [int]$actualRun.executed_test_count -ne $expectedRunIdentities.Count -or
                ($declaredRunIdentities -join "`n") -cne ($expectedRunIdentities -join "`n") -or
                ($executedRunIdentities -join "`n") -cne ($expectedRunIdentities -join "`n") -or
                ($actualArguments -join "`n") -cne ($expectedArguments -join "`n")) {
                throw "Predicate '$($Predicate.id)' evidence does not prove exact Workflow run '$([string]$expectedRun.label)'."
            }
            Assert-Sprint8CExactCargoSelector -Arguments $actualArguments `
                -Package ([string]$expectedRun.package) -TestBinary ([string]$expectedRun.test_binary) `
                -PredicateId ([string]$Predicate.id)
        }
        return [pscustomobject][ordered]@{
            path = $resolved
            sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
            proof = [string]$document.proof
            suite = $null
            test_binary = "multiple"
            test_identities = @($executed)
            executed_test_count = [int]$document.executed_test_count
            cleanup_restoration = $document.database.cleanup_restoration
        }
    }
    $evidence = if ($kind -ceq "exact-identities" -and
        $null -ne $document.PSObject.Properties['suites']) {
        $matches = @($document.suites | Where-Object {
            [string]$_.test_binary -ceq [string]$contract.test_binary
        })
        if ($matches.Count -ne 1) {
            throw "Predicate '$($Predicate.id)' evidence omits its exact test binary."
        }
        $matches[0]
    } else { $document }
    $expected = @($contract.expected_test_identities | ForEach-Object { [string]$_ })
    $declared = @($evidence.expected_test_identities | ForEach-Object { [string]$_ })
    $executed = @($evidence.executed_test_identities | ForEach-Object { [string]$_ })
    if (($kind -ceq "legacy-exact-identities" -and
            ([string]$document.suite -cne [string]$contract.suite -or
             [string]$document.test_binary -cne [string]$contract.test_binary)) -or
        [int]$evidence.executed_test_count -ne $expected.Count -or
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
    $evidenceArgumentsProperty = $evidence.PSObject.Properties['arguments']
    [string[]]$arguments = @(if ($null -ne $evidenceArgumentsProperty) {
        @($evidenceArgumentsProperty.Value | ForEach-Object { [string]$_ })
    })
    if ($arguments.Count -eq 0) {
        $commandProperty = $document.PSObject.Properties['command']
        $commandArgumentsProperty = if ($null -eq $commandProperty) { $null } else {
            $commandProperty.Value.PSObject.Properties['arguments']
        }
        [string[]]$arguments = @(if ($null -ne $commandArgumentsProperty) {
            @($commandArgumentsProperty.Value | ForEach-Object { [string]$_ })
        })
    }
    Assert-Sprint8CExactCargoSelector -Arguments $arguments `
        -Package $(if ($kind -ceq "legacy-exact-identities") {
            "tessara-dataset-module"
        } else { [string]$contract.package }) `
        -TestBinary ([string]$contract.test_binary) -PredicateId ([string]$Predicate.id)
    [pscustomobject][ordered]@{
        path = $resolved
        sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
        proof = [string]$document.proof
        suite = if ($null -eq $evidence.PSObject.Properties['suite']) {
            $null
        } else { [string]$evidence.suite }
        test_binary = [string]$contract.test_binary
        test_identities = @($executed)
        executed_test_count = [int]$evidence.executed_test_count
        cleanup_restoration = $document.database.cleanup_restoration
    }
}

function Get-Sprint8CUatChildEvidenceRoot {
    param(
        [Parameter(Mandatory)][string]$EvidencePath
    )

    $resolvedEvidencePath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
    $evidenceStem = [IO.Path]::GetFileNameWithoutExtension($resolvedEvidencePath)
    if ([string]::IsNullOrWhiteSpace($evidenceStem) -or
        $evidenceStem -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
        throw "Sprint 8C UAT evidence path must have a bounded action-specific JSON filename."
    }
    Join-Path (Join-Path (Split-Path -Parent $resolvedEvidencePath) "predicates") $evidenceStem
}

function Assert-Sprint8CSelectedAssertionEvidence {
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

function Get-Sprint8CUatAssertionProofState {
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

function ConvertTo-Sprint8CUatScenarioAssertionResults {
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
                $proofState = Get-Sprint8CUatAssertionProofState `
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

function Test-Sprint8CUatPredicateReadiness {
    $scenarioContract = Get-Content -LiteralPath $scenarioContractPath -Raw | ConvertFrom-Json -Depth 100
    $catalog = @(Get-Sprint8CUatPredicateCatalog)
    $mapping = Get-Sprint8CUatAssertionMap
    $evidenceClaims = Get-Sprint8CUatAssertionEvidenceClaims
    $assertionProofContract = @(Get-Sprint8CUatAssertionProofContract `
        -ScenarioContract $scenarioContract -AssertionMap $mapping `
        -EvidenceClaims $evidenceClaims)
    Assert-Sprint8CUatPredicateContract -ScenarioContract $scenarioContract `
        -PredicateCatalog $catalog -AssertionMap $mapping -EvidenceClaims $evidenceClaims
    $exactAssertionContracts = @($assertionProofContract | Where-Object {
        [string]$_.automated_claim_kind -ceq "exact_test_evidence"
    })
    $prerequisiteOnlyContracts = @($assertionProofContract | Where-Object {
        [string]$_.automated_claim_kind -ceq "prerequisite_only"
    })
    if ($assertionProofContract.Count -ne 31 -or
        $exactAssertionContracts.Count -ne 13 -or
        $prerequisiteOnlyContracts.Count -ne 18 -or
        @($assertionProofContract | Where-Object {
            -not [bool]$_.manual_acceptance_required
        }).Count -ne 0) {
        throw "Sprint 8C UAT assertion proof classification is not exact and fail-closed."
    }

    $selected = @(Get-Sprint8CSelectedScenarios -ScenarioContract $scenarioContract -Selections @("UAT-8C-04", "UAT-8C-11"))
    $predicates = @(Get-Sprint8CPredicateSelection -SelectedScenarios $selected `
        -AssertionMap $mapping -PredicateCatalog $catalog)
    if (($selected.id -join "`n") -cne (@("UAT-8C-04", "UAT-8C-11") -join "`n") -or
        $predicates.id -cnotcontains "dataset-refresh-orchestration" -or
        $predicates.id -cnotcontains "dataset-dag" -or
        $predicates.id -cnotcontains "workflow-events" -or
        $predicates.id -cnotcontains "dataset-export") {
        throw "Sprint 8C UAT exact scenario/predicate selection self-test failed."
    }
    $datasetExportPredicate = @($catalog | Where-Object {
        [string]$_.id -ceq "dataset-export"
    })[0]

    $allSelected = @(Get-Sprint8CSelectedScenarios `
        -ScenarioContract $scenarioContract -Selections @("All"))
    $allScenarioAssertions = @(ConvertTo-Sprint8CUatScenarioAssertionResults `
        -SelectedScenarios $allSelected -AssertionMap $mapping `
        -EvidenceClaims $evidenceClaims -VerifiedAssertionEvidence @() `
        -PredicateExecutionPassed $false)
    if ($allScenarioAssertions.Count -ne 11 -or
        @($allScenarioAssertions.assertions).Count -ne 31 -or
        @($allScenarioAssertions.assertions | Where-Object {
            [string]$_.state -cne "not_proven" -or
            -not [bool]$_.manual_acceptance_required
        }).Count -ne 0) {
        throw "Sprint 8C UAT All-scenario result assembly self-test failed."
    }

    $allPredicates = @(Get-Sprint8CPredicateSelection -SelectedScenarios $allSelected `
        -AssertionMap $mapping -PredicateCatalog $catalog)
    $parentProject = "tessara-s8c-uat-selftest"
    $topologyPlans = @($allPredicates | ForEach-Object {
        Get-Sprint8CUatPredicateTopologyPlan -Predicate $_ -ParentComposeProject $parentProject
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
        throw "Sprint 8C UAT topology scheduling self-test failed."
    }

    $materializedPorts = Set-Sprint8CUatMaterializedTopologyEnvironment `
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
        throw "Sprint 8C UAT materialized-port handoff self-test failed."
    }
    try {
        Set-Sprint8CUatMaterializedTopologyEnvironment `
            -ExpectedComposeProject $parentProject -MaterializationReceipt ([pscustomobject]@{
                compose_project = "tessara-s8c-substituted"
                environment = [pscustomobject]@{
                    COMPOSE_PROJECT_NAME = "tessara-s8c-substituted"
                    TESSARA_GATEWAY_PORT = "45101"
                    TESSARA_CORE_CONTROL_PORT = "45102"
                    TESSARA_SUPERVISOR_PORT = "45103"
                }
            }) | Out-Null
        throw "Sprint 8C UAT materialized-port handoff accepted a substituted project."
    } catch {
        if ($_.Exception.Message -notmatch 'substituted the Compose project') { throw }
    }

    $tampered = Get-Sprint8CUatAssertionMap
    [void]$tampered["UAT-8C-04"].Remove("unchanged_head_no_page")
    try {
        Assert-Sprint8CUatPredicateContract -ScenarioContract $scenarioContract `
            -PredicateCatalog $catalog -AssertionMap $tampered -EvidenceClaims $evidenceClaims
        throw "Sprint 8C UAT predicate self-test accepted a missing assertion mapping."
    } catch {
        if ($_.Exception.Message -notmatch 'not set-equal') { throw }
    }

    $tamperedCatalog = $catalog | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $tamperedDatasetExport = @($tamperedCatalog | Where-Object {
        [string]$_.id -ceq "dataset-export"
    })[0]
    $tamperedDatasetExport.PSObject.Properties.Remove('child_evidence_root_argument')
    try {
        Assert-Sprint8CUatPredicateContract -ScenarioContract $scenarioContract `
            -PredicateCatalog @($tamperedCatalog) -AssertionMap $mapping -EvidenceClaims $evidenceClaims
        throw "Sprint 8C UAT predicate self-test accepted Dataset-export without run-scoped nested evidence."
    } catch {
        if ($_.Exception.Message -notmatch 'must bind nested receipts') { throw }
    }

    $tamperedScenarioContract = $scenarioContract | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedScenarioContract.scenarios[0].actor_keys[0] = "actor.predicted"
    try {
        Assert-Sprint8CUatPredicateContract -ScenarioContract $tamperedScenarioContract `
            -PredicateCatalog $catalog -AssertionMap $mapping -EvidenceClaims $evidenceClaims
        throw "Sprint 8C UAT predicate self-test accepted an undeclared logical actor key."
    } catch {
        if ($_.Exception.Message -notmatch 'precondition or cleanup identity') { throw }
    }

    $tamperedClaims = Get-Sprint8CUatAssertionEvidenceClaims
    $tamperedClaims["UAT-8C-04"]["unchanged_head_no_page"][0].test_identity =
        "unrelated_store_level_test"
    try {
        Assert-Sprint8CUatPredicateContract -ScenarioContract $scenarioContract `
            -PredicateCatalog $catalog -AssertionMap $mapping -EvidenceClaims $tamperedClaims
        throw "Sprint 8C UAT predicate self-test accepted a substituted exact test identity."
    } catch {
        if ($_.Exception.Message -notmatch 'unproven exact test identity') { throw }
    }

    $selfTestRoot = Join-Path ([IO.Path]::GetTempPath()) `
        "tessara-s8c-uat-predicate-$([Guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($selfTestRoot) | Out-Null
        $firstScenarioEvidence = Join-Path $selfTestRoot "actions/uat-8c-01-evidence.json"
        $secondScenarioEvidence = Join-Path $selfTestRoot "actions/uat-8c-09-evidence.json"
        $firstChildRoot = Get-Sprint8CUatChildEvidenceRoot -EvidencePath $firstScenarioEvidence
        $secondChildRoot = Get-Sprint8CUatChildEvidenceRoot -EvidencePath $secondScenarioEvidence
        if ($firstChildRoot -ceq $secondChildRoot -or
            [IO.Path]::GetFileName($firstChildRoot) -cne "uat-8c-01-evidence" -or
            [IO.Path]::GetFileName($secondChildRoot) -cne "uat-8c-09-evidence") {
            throw "Sprint 8C UAT self-test did not isolate child evidence by formal action identity."
        }
        [IO.Directory]::CreateDirectory($firstChildRoot) | Out-Null
        [IO.Directory]::CreateDirectory($secondChildRoot) | Out-Null
        $datasetExportEvidence = Join-Path $firstChildRoot "dataset-export.json"
        $datasetExportArguments = @(Get-Sprint8CUatScriptPredicateArguments `
            -Predicate $datasetExportPredicate -ChildEvidenceRoot $firstChildRoot `
            -ChildEvidence $datasetExportEvidence)
        $expectedDatasetExportArguments = @(
            "-EvidencePath", $datasetExportEvidence,
            "-EvidenceRoot", (Join-Path $firstChildRoot "dataset-export-children")
        )
        if (($datasetExportArguments -join "`n") -cne
            ($expectedDatasetExportArguments -join "`n")) {
            throw "Sprint 8C UAT self-test did not bind Dataset-export nested receipts to its action root."
        }
        $firstMaterialization = Join-Path $firstChildRoot "reference-materialization.json"
        $secondMaterialization = Join-Path $secondChildRoot "reference-materialization.json"
        $syntheticMaterialization = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8c"
            proof = "synthetic-uat-materialization"
            state = "passed"
        }
        Publish-Sprint8CHarnessEvidence -Document $syntheticMaterialization `
            -OutputPath $firstMaterialization | Out-Null
        Publish-Sprint8CHarnessEvidence -Document $syntheticMaterialization `
            -OutputPath $secondMaterialization | Out-Null
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $firstMaterialization `
                -SidecarPath "$firstMaterialization.sha256") -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $secondMaterialization `
                -SidecarPath "$secondMaterialization.sha256")) {
            throw "Sprint 8C UAT self-test did not publish both action-scoped evidence pairs."
        }
        try {
            Publish-Sprint8CHarnessEvidence -Document $syntheticMaterialization `
                -OutputPath $firstMaterialization | Out-Null
            throw "Sprint 8C UAT self-test allowed an overwrite within one action evidence scope."
        } catch {
            if ($_.Exception.Message -notmatch 'Retained evidence exists') { throw }
        }

        $refreshPredicate = @($catalog | Where-Object {
            [string]$_.id -ceq "dataset-refresh-orchestration"
        })[0]
        $exactEvidencePath = Join-Path $selfTestRoot "refresh.json"
        Publish-Sprint8CHarnessEvidence -Document ([pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8c"
            proof = "dataset-module-test-suite"
            state = "passed"
            suite = "Refresh"
            test_binary = "refresh_integration"
            expected_test_identities = @($script:Sprint8CRefreshTestIdentities)
            executed_test_identities = @($script:Sprint8CRefreshTestIdentities)
            executed_test_count = $script:Sprint8CRefreshTestIdentities.Count
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
        Assert-Sprint8CExactTestPredicateEvidence -Predicate $refreshPredicate `
            -EvidencePath $exactEvidencePath | Out-Null
        $tamperedEvidencePath = Join-Path $selfTestRoot "refresh-tampered.json"
        Publish-Sprint8CHarnessEvidence -Document ([pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8c"
            proof = "dataset-module-test-suite"
            state = "passed"
            suite = "Refresh"
            test_binary = "refresh_integration"
            expected_test_identities = @($script:Sprint8CRefreshTestIdentities)
            executed_test_identities = @($script:Sprint8CRefreshTestIdentities[0..6])
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
            Assert-Sprint8CExactTestPredicateEvidence -Predicate $refreshPredicate `
                -EvidencePath $tamperedEvidencePath | Out-Null
            throw "Sprint 8C UAT predicate self-test accepted incomplete exact test evidence."
        } catch {
            if ($_.Exception.Message -notmatch 'exact test identity') { throw }
        }

        $digest = "sha256:$('a' * 64)"
        $ownerOrder = @(
            "core", "tessara.responses", "tessara.datasets", "tessara.components",
            "tessara.dashboards", "tessara.reference.scoped-records"
        )
        $firstOwnerReceipts = @($ownerOrder | ForEach-Object {
            [pscustomobject][ordered]@{
                owner = $_; schema_version = "tessara.io/self-test/v1"
                input_digest = $digest; result_digest = $digest; changed = $true
                resource_ids = [pscustomobject]@{ logical_key = "typed-read-back" }
            }
        })
        $noOpOwnerReceipts = @($firstOwnerReceipts | ForEach-Object {
            $copy = $_ | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
            $copy.changed = $false
            $copy
        })
        $materializationDocument = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8c"
            proof = "clean-owner-materialization-and-semantic-noop"; state = "passed"
            target = "ReferenceNoOp"; compose_configuration_sha256 = ('b' * 64)
            blueprint_sha256 = ('c' * 64)
            first_apply = [pscustomobject]@{
                operation_state = "succeeded"; changed = $true; no_op = $false
                receipt_digest = $digest; lockfile_digest = $digest; plan_digest = $digest
                revision = 1; owner_order = $ownerOrder; owner_receipts = $firstOwnerReceipts
            }
            semantic_noop = [pscustomobject]@{
                operation_state = "succeeded"; changed = $false; no_op = $true
                previous_receipt_digest = $digest; revision = 2
                owner_order = $ownerOrder; owner_receipts = $noOpOwnerReceipts
            }
            semantic_noop_proof = [pscustomobject]@{
                state = "passed"; stable_owner_receipts = $true
                stable_enablement_artifacts_configuration = $true
                stable_container_topology = $true
            }
            gateway_start_boundary = [pscustomobject]@{
                owner_apply_completed_before_start = $true; post_start_health = "passed"
            }
            health = [pscustomobject]@{
                response_live = [pscustomobject]@{
                    schema_version = 1; status = "passing"; checks = @()
                }
                response_ready = [pscustomobject]@{
                    schema_version = 1; status = "passing"
                    checks = @(
                        "response.database",
                        "response.configuration",
                        "response.security_state",
                        "response.provider.forms",
                        "response.provider.workflow",
                        "response.events.publication",
                        "response.export.publication" | ForEach-Object {
                            [pscustomobject][ordered]@{
                                code = $_; passing = $true; message = "self-test passing check"
                            }
                        }
                    )
                }
            }
            fixture_receipt_path = "target/self-test/fixture.json"
            fixture_receipt_sha256 = ('d' * 64)
            cleanup_restoration = [pscustomobject]@{
                state = "passed"; mode = "exact-project-teardown"
            }
        }
        $failureSpecifications = @(
            [pscustomobject]@{
                fault_key = "response.bootstrap.mid-apply"
                receipt_contract = "tessara.sprint-8c.failure-control/v1"
                target_service = "responses"; phase = "response_bootstrap_transaction"
                expected_outcome = "rolled_back"
                expected_failure_code = "response.bootstrap.injected_failure"
                transaction_field = "response_transaction"; transaction_value = "rolled_back"
            },
            [pscustomobject]@{
                fault_key = "response.incompatible"
                receipt_contract = "tessara.sprint-8b.failure-control/v1"
                target_service = "response-provider-proxy"
                phase = "dataset_bootstrap_provider_validation"
                expected_outcome = "rejected_pre_write"
                expected_failure_code = "dataset.dependency_incompatible"
                transaction_field = "dataset_transaction"; transaction_value = "not_started"
            },
            [pscustomobject]@{
                fault_key = "dataset.derived-rebuild"
                receipt_contract = "tessara.sprint-8b.failure-control/v1"
                target_service = "datasets"; phase = "dataset_bootstrap_transaction"
                expected_outcome = "rolled_back"
                expected_failure_code = "dataset.dependency_unavailable"
                transaction_field = "dataset_transaction"; transaction_value = "rolled_back"
            }
        )
        $failureAttemptIndex = 0
        $failureAttempts = @($failureSpecifications | ForEach-Object {
            $failureAttemptIndex++
            $materializationSha256 = ([string]$failureAttemptIndex) * 64
            [pscustomobject][ordered]@{
                fault = [pscustomobject][ordered]@{
                    fault_key = [string]$_.fault_key
                    receipt_contract = [string]$_.receipt_contract
                    target_service = [string]$_.target_service; phase = [string]$_.phase
                    expected_outcome = [string]$_.expected_outcome
                    expected_failure_code = [string]$_.expected_failure_code
                    attempt = 1; attempt_limit = 1
                    no_unauthorized_state = $true; no_cross_owner_write = $true
                    transaction_field = [string]$_.transaction_field
                    transaction_value = [string]$_.transaction_value
                }
                correlation_id = "01980000-00f0-7000-8000-{0:d12}" -f $failureAttemptIndex
                materialization_evidence_sha256 = $materializationSha256
                materialization_evidence = [pscustomobject][ordered]@{
                    path = "target/self-test/fault-$failureAttemptIndex.json"
                    sha256 = $materializationSha256
                }
                containment = [pscustomobject][ordered]@{
                    fault_key = [string]$_.fault_key
                    materialization_state = "failed"; gateway_started = $false
                    fixture_published = $false; partial_topology_teardown = "passed"
                }
                child_exit_code = 1
            }
        })
        $failureDocument = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8c"
            proof = "deterministic-failure-containment-retry-and-restoration"; state = "passed"
            failure_attempts = $failureAttempts
            successor_evidence = [pscustomobject]@{ state = "passed"; target = "ReferenceNoOp" }
            restoration_proof = [pscustomobject]@{
                empty_start = "passed"; canonical_first_apply = "passed"
                semantic_noop = "passed"; canonical_health = "passed"; final_teardown = "passed"
            }
            cleanup_restoration = [pscustomobject]@{
                state = "passed"
                mode = "three-exact-partial-teardowns-plus-restored-successor-teardown"
                fault_controls = "cleared-to-none"; empty_successor_start = "passed"
            }
        }
        $upgradeSequence = @("0.9.0", "1.0.0", "0.9.0", "1.0.0")
        $upgradeStages = @(
            "establish-compatible-baseline", "upgrade-to-candidate",
            "rollback-to-compatible-baseline", "restore-intended-candidate"
        )
        $upgradeDocument = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8c"
            proof = "independent-response-upgrade-rollback-restoration"; state = "passed"
            release_contract = [pscustomobject]@{
                transition = [pscustomobject]@{
                    owner = "tessara.responses"; intended_release = "1.0.0"
                }
                sequence = $upgradeSequence
                stages = $upgradeStages
            }
            release_fixture = [pscustomobject]@{
                baseline = [pscustomobject]@{
                    version = "0.9.0"; runtime_image = "sha256:$('b' * 64)"
                    executable_sha256 = ('b' * 64)
                }
                candidate = [pscustomobject]@{
                    version = "1.0.0"; runtime_image = "sha256:$('a' * 64)"
                    executable_sha256 = ('a' * 64)
                }
            }
            pre_exercise_snapshot = [pscustomobject]@{
                stage = "pre-exercise-candidate"
                response = [pscustomobject]@{ release = "1.0.0" }
            }
            transitions = @(for ($index = 0; $index -lt $upgradeSequence.Count; $index++) {
                $imageToken = if ($upgradeSequence[$index] -ceq "0.9.0") { 'b' } else { 'a' }
                [pscustomobject]@{
                    stage = $upgradeStages[$index]
                    target_release = $upgradeSequence[$index]
                    target_image = "sha256:$($imageToken * 64)"
                    target_manifest_digest = "sha256:$('c' * 64)"
                    plan_digest = "sha256:$('d' * 64)"
                    lockfile_digest = "sha256:$('e' * 64)"
                    receipt_lockfile_digest = "sha256:$('e' * 64)"
                    fixed_owner_lockfile = "passed"
                    exact_delta = [pscustomobject]@{
                        owner = "tessara.responses"; action_count = 5; exact = $true
                    }
                }
            })
            stage_snapshots = @(for ($index = 0; $index -lt $upgradeSequence.Count; $index++) {
                [pscustomobject]@{
                    stage = $upgradeStages[$index]
                    response = [pscustomobject]@{ release = $upgradeSequence[$index] }
                }
            })
            preservation_proofs = @($upgradeStages | ForEach-Object {
                [pscustomobject]@{
                    stage = $_; state = "passed"; response_state = "passed"
                    module_instance_identity = "passed"; typed_resource_identity = "passed"
                    navigation_identity = "passed"; outbox_positions = "passed"
                    unrelated_owners = "passed"
                }
            })
            release_restoration = [pscustomobject]@{ state = "passed"; final_release = "1.0.0" }
            cleanup_restoration = [pscustomobject]@{ state = "passed" }
            failure = $null
        }
        $structuredCases = @(
            [pscustomobject]@{
                id = "materialization-noop"; document = $materializationDocument
                tamper = {
                    param($value)
                    $value.health.response_live = [pscustomobject]@{
                        module_definition_id = "tessara.responses"
                        module_release_version = "1.0.0"; status = "live"
                    }
                }
            },
            [pscustomobject]@{
                id = "failure-recovery"; document = $failureDocument
                tamper = { param($value) $value.failure_attempts[0].fault.attempt_limit = 2 }
            },
            [pscustomobject]@{
                id = "independent-upgrade-rollback"; document = $upgradeDocument
                tamper = { param($value) $value.transitions[2].target_release = "1.0.0" }
            }
        )
        foreach ($case in $structuredCases) {
            $predicate = @($catalog | Where-Object { [string]$_.id -ceq [string]$case.id })[0]
            $path = Join-Path $selfTestRoot "$($case.id)-structured.json"
            Publish-Sprint8CHarnessEvidence -Document $case.document -OutputPath $path | Out-Null
            Assert-Sprint8CExactTestPredicateEvidence -Predicate $predicate -EvidencePath $path | Out-Null

            $tamperedDocument = $case.document | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
            & $case.tamper $tamperedDocument
            $tamperedPath = Join-Path $selfTestRoot "$($case.id)-structured-tampered.json"
            Publish-Sprint8CHarnessEvidence -Document $tamperedDocument -OutputPath $tamperedPath | Out-Null
            try {
                Assert-Sprint8CExactTestPredicateEvidence -Predicate $predicate `
                    -EvidencePath $tamperedPath | Out-Null
                throw "Sprint 8C UAT self-test accepted tampered structured '$($case.id)' evidence."
            } catch {
                if ($_.Exception.Message -notmatch 'does not prove') { throw }
            }
        }

        $duplicateFailureDocument = $failureDocument | ConvertTo-Json -Depth 100 |
            ConvertFrom-Json -Depth 100
        $duplicateFailureDocument.failure_attempts[0].containment | Add-Member `
            -NotePropertyName containment -NotePropertyValue ([pscustomobject]@{
                attempt = 1; attempt_limit = 1
            })
        $duplicateFailurePath = Join-Path $selfTestRoot "failure-recovery-duplicate.json"
        Publish-Sprint8CHarnessEvidence -Document $duplicateFailureDocument `
            -OutputPath $duplicateFailurePath | Out-Null
        $duplicateFailureRejected = $false
        try {
            $failurePredicate = @($catalog | Where-Object {
                [string]$_.id -ceq "failure-recovery"
            })[0]
            Assert-Sprint8CExactTestPredicateEvidence -Predicate $failurePredicate `
                -EvidencePath $duplicateFailurePath | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'does not prove') { throw }
            $duplicateFailureRejected = $true
        }
        if (-not $duplicateFailureRejected) {
            throw "Sprint 8C UAT self-test accepted a duplicate nested failure projection."
        }

        $workflowPredicate = @($catalog | Where-Object { [string]$_.id -ceq "workflow-events" })[0]
        $workflowEvidence = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8c"
            proof = "workflow-response-event-consumption"; state = "passed"
            expected_test_identities = @($workflowPredicate.evidence_contract.expected_test_identities)
            executed_test_identities = @($workflowPredicate.evidence_contract.expected_test_identities)
            executed_test_count = @($workflowPredicate.evidence_contract.expected_test_identities).Count
            database = [pscustomobject]@{
                mode = "disposable-postgres"
                cleanup_restoration = [pscustomobject]@{ state = "passed" }
            }
            runs = @($workflowPredicate.evidence_contract.expected_runs | ForEach-Object {
                [pscustomobject][ordered]@{
                    label = [string]$_.label
                    arguments = @($_.arguments) + @("--", "--format", "terse")
                    expected_test_identities = @($_.identities)
                    executed_test_identities = @($_.identities)
                    executed_test_count = @($_.identities).Count
                }
            })
        }
        $workflowPath = Join-Path $selfTestRoot "workflow-lib.json"
        Publish-Sprint8CHarnessEvidence -Document $workflowEvidence -OutputPath $workflowPath | Out-Null
        Assert-Sprint8CExactTestPredicateEvidence -Predicate $workflowPredicate `
            -EvidencePath $workflowPath | Out-Null
        $workflowTampered = $workflowEvidence | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
        $workflowTampered.runs[0].arguments = @(
            "test", "-p", "tessara-api", "--test", "lib",
            "workflow_response_consumer::tests::", "--locked", "--offline", "--jobs", "1",
            "--", "--format", "terse"
        )
        $workflowTamperedPath = Join-Path $selfTestRoot "workflow-lib-tampered.json"
        Publish-Sprint8CHarnessEvidence -Document $workflowTampered `
            -OutputPath $workflowTamperedPath | Out-Null
        try {
            Assert-Sprint8CExactTestPredicateEvidence -Predicate $workflowPredicate `
                -EvidencePath $workflowTamperedPath | Out-Null
            throw "Sprint 8C UAT self-test accepted a substituted Workflow run selector."
        } catch {
            if ($_.Exception.Message -notmatch 'exact Workflow run|exact Cargo test selector') { throw }
        }

        $prerequisiteOnly = Get-Sprint8CUatAssertionProofState `
            -ScenarioId "UAT-8C-01" `
            -Assertion "assigned start, save, resume, submit, and review preserve accepted UI behavior" `
            -EvidenceClaims $evidenceClaims `
            -VerifiedAssertionEvidence @() -PredicateExecutionPassed $true
        if ([string]$prerequisiteOnly.state -cne "predicate_prerequisites_passed" -or
            [string]$prerequisiteOnly.automated_claim_kind -cne "prerequisite_only" -or
            -not [bool]$prerequisiteOnly.manual_acceptance_required) {
            throw "Sprint 8C UAT self-test mislabeled association-only predicate evidence as an assertion pass."
        }

        $exactState = Get-Sprint8CUatAssertionProofState `
            -ScenarioId "UAT-8C-04" -Assertion "unchanged_head_no_page" `
            -EvidenceClaims $evidenceClaims -VerifiedAssertionEvidence @(
                [pscustomobject]@{
                    scenario_id = "UAT-8C-04"
                    assertion = "unchanged_head_no_page"
                }
            ) -PredicateExecutionPassed $true
        if ([string]$exactState.state -cne "exact_automated_evidence_passed" -or
            [string]$exactState.automated_claim_kind -cne "exact_test_evidence" -or
            -not [bool]$exactState.manual_acceptance_required) {
            throw "Sprint 8C UAT self-test did not retain the exact automated/manual proof boundary."
        }
    } finally {
        if (Test-Path -LiteralPath $selfTestRoot) {
            Remove-Item -LiteralPath $selfTestRoot -Recurse -Force
        }
    }

    $failed = Invoke-Sprint8CProgramPredicate -Program "pwsh" `
        -Arguments @("-NoProfile", "-Command", "exit 19") -AllowFailure
    if ($failed.exit_code -ne 19) {
        throw "Sprint 8C UAT predicate child failure self-test did not retain the exact exit code."
    }
    try {
        Invoke-Sprint8CProgramPredicate -Program "pwsh" `
            -Arguments @("-NoProfile", "-Command", "exit 23") | Out-Null
        throw "Sprint 8C UAT predicate child failure self-test did not fail closed."
    } catch {
        if ($_.Exception.Message -notmatch 'exited 23') { throw }
    }

    $contractHash = (Get-FileHash -LiteralPath $scenarioContractPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $assertionProofContractHash = Get-Sprint7ASha256 -Text (
        ($assertionProofContract | ConvertTo-Json -Depth 100 -Compress) + "`n"
    )
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
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
        duplicate_failure_projection_rejected = $duplicateFailureRejected
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

function Assert-Sprint8CFixtureReceipt {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ExpectedComposeProject
    )

    $resolved = Resolve-Sprint8CRepositoryPath -Path $Path
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $resolved -SidecarPath "$resolved.sha256")) {
        throw "Sprint 8C live UAT requires an authenticated fixture receipt pair."
    }
    $receipt = Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json -Depth 100
    if ([string]$receipt.sprint -cne "sprint-8c" -or
        [string]$receipt.state -cne "passed" -or
        [string]$receipt.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$receipt.compose_project -cne $ExpectedComposeProject -or
        [string]$receipt.restoration.state -cne "passed") {
        throw "Sprint 8C fixture receipt does not bind the expected healthy '$ExpectedComposeProject' topology."
    }
    $responseFixtureProof = Assert-Sprint8CPreparedResponseFixtures -FixtureReceipt $receipt
    [pscustomobject][ordered]@{
        path = $resolved
        sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
        document = $receipt
        response_fixtures = $responseFixtureProof
    }
}

function Get-Sprint8CUatChildComposeProject {
    param(
        [Parameter(Mandatory)][string]$ParentComposeProject,
        [Parameter(Mandatory)][string]$PredicateId
    )

    Assert-Sprint8CComposeProject -ComposeProject $ParentComposeProject | Out-Null
    $digest = (Get-Sprint7ASha256 -Text "$ParentComposeProject`n$PredicateId`n").Substring(0, 12)
    $child = "tessara-s8c-uat-$digest"
    Assert-Sprint8CComposeProject -ComposeProject $child | Out-Null
    $child
}

function Get-Sprint8CUatPredicateTopologyPlan {
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
            Get-Sprint8CUatChildComposeProject -ParentComposeProject $ParentComposeProject `
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

function Assert-Sprint8CUatReferenceTopologyReady {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject,
        [Parameter(Mandatory)][string]$GatewayUrl,
        [Parameter(Mandatory)]$Fixture
    )

    Assert-Sprint8CExistingTopology -ComposePath $ComposePath `
        -ComposeProject $ComposeProject | Out-Null
    $probe = Invoke-Sprint8CHttpProbe -Uri "$($GatewayUrl.TrimEnd('/'))/health" `
        -ExpectedStatus @(200)
    Assert-Sprint8CFixtureReceipt -Path ([string]$Fixture.path) `
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

function Set-Sprint8CUatMaterializedTopologyEnvironment {
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

    Set-Sprint8CComposeEnvironment -ComposeProject $ExpectedComposeProject `
        -GatewayPort ([int]$portValues[0]) -CorePort ([int]$portValues[1]) `
        -SupervisorPort ([int]$portValues[2])
}

function Get-Sprint8CUatScriptPredicateArguments {
    param(
        [Parameter(Mandatory)]$Predicate,
        [Parameter(Mandatory)][string]$ChildEvidenceRoot,
        [Parameter(Mandatory)][string]$ChildEvidence
    )

    $arguments = @($Predicate.arguments)
    if ([string]$Predicate.id -ceq "acceptance-contract" -or
        $null -ne $Predicate.PSObject.Properties['evidence_contract']) {
        $arguments += @("-EvidencePath", $ChildEvidence)
    }
    $childEvidenceRootProperty = $Predicate.PSObject.Properties['child_evidence_root_argument']
    if ($null -ne $childEvidenceRootProperty) {
        $arguments += @(
            [string]$childEvidenceRootProperty.Value,
            (Join-Path $ChildEvidenceRoot "$($Predicate.id)-children")
        )
    }
    @($arguments)
}

function Invoke-Sprint8CUatPredicate {
    param(
        [Parameter(Mandatory)]$Predicate,
        [Parameter(Mandatory)][string]$ResolvedComposeProject,
        [Parameter(Mandatory)][string]$PredicateComposeProject,
        [Parameter(Mandatory)][string]$ChildEvidenceRoot,
        [Parameter(Mandatory)]$ExpectedSource,
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
            $arguments = @(Get-Sprint8CUatScriptPredicateArguments -Predicate $Predicate `
                -ChildEvidenceRoot $ChildEvidenceRoot -ChildEvidence $childEvidence)
            Invoke-Sprint8CChildScript -ScriptPath ([string]$Predicate.path) -Arguments $arguments
            break
        }
        "program" {
            Invoke-Sprint8CProgramPredicate -Program ([string]$Predicate.program) `
                -Arguments @($Predicate.arguments)
            break
        }
        "readiness-target" {
            Invoke-Sprint8CChildScript -ScriptPath "scripts/run-sprint-8c-implementation-readiness.ps1" `
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
            Invoke-Sprint8CChildScript -ScriptPath "scripts/materialize-sprint-8c.ps1" -Arguments $arguments
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
            Invoke-Sprint8CChildScript -ScriptPath "scripts/run-sprint-8c-deployed-smoke.ps1" -Arguments $arguments
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
            Invoke-Sprint8CChildScript -ScriptPath "scripts/run-sprint-8c-failure-containment.ps1" -Arguments $arguments
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
            Invoke-Sprint8CChildScript -ScriptPath "scripts/run-sprint-8c-response-upgrade.ps1" -Arguments $arguments
            break
        }
        default { throw "Unknown Sprint 8C UAT predicate kind '$($Predicate.kind)'." }
    }
    if ($null -ne $Predicate.PSObject.Properties['evidence_contract']) {
        $exactTestEvidence = Assert-Sprint8CExactTestPredicateEvidence `
            -Predicate $Predicate -EvidencePath $childEvidence `
            -ExpectedComposeProject $PredicateComposeProject -ExpectedSource $ExpectedSource
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
    $result = Test-Sprint8CUatPredicateReadiness
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8CHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result | ConvertTo-Json -Depth 50
    return
}

$scenarioContract = Get-Content -LiteralPath $scenarioContractPath -Raw | ConvertFrom-Json -Depth 100
$predicateCatalog = @(Get-Sprint8CUatPredicateCatalog)
$assertionMap = Get-Sprint8CUatAssertionMap
$evidenceClaims = Get-Sprint8CUatAssertionEvidenceClaims
$assertionProofContract = @(Get-Sprint8CUatAssertionProofContract `
    -ScenarioContract $scenarioContract -AssertionMap $assertionMap `
    -EvidenceClaims $evidenceClaims)
$assertionProofContractHash = Get-Sprint7ASha256 -Text (
    ($assertionProofContract | ConvertTo-Json -Depth 100 -Compress) + "`n"
)
Assert-Sprint8CUatPredicateContract -ScenarioContract $scenarioContract `
    -PredicateCatalog $predicateCatalog -AssertionMap $assertionMap -EvidenceClaims $evidenceClaims
$selectedScenarios = @(Get-Sprint8CSelectedScenarios -ScenarioContract $scenarioContract -Selections $Scenario)
$selectedPredicates = @(Get-Sprint8CPredicateSelection -SelectedScenarios $selectedScenarios `
    -AssertionMap $assertionMap -PredicateCatalog $predicateCatalog)
Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
$source = Get-Sprint8CSourceIdentity -RequireClean

$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT", "PLAYWRIGHT_BASE_URL", "TESSARA_PLAYWRIGHT_ACCEPTANCE"
)
$environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $environmentNames
$ownedTopology = $false
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$predicateResults = [Collections.Generic.List[object]]::new()
$verifiedAssertionEvidence = @()
$resolvedFixture = $null
$failure = $null
$composePath = Resolve-Sprint8CRepositoryPath -Path "deploy/sprint-8c/compose.yaml"
$evidenceFullPath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
$childEvidenceRoot = Get-Sprint8CUatChildEvidenceRoot -EvidencePath $evidenceFullPath
[IO.Directory]::CreateDirectory($childEvidenceRoot) | Out-Null

try {
    if ($UseExistingTopology) {
        foreach ($name in @("TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT")) {
            if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
                throw "Existing-topology UAT requires inherited '$name'."
            }
        }
        $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$env:TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$env:TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$env:TESSARA_SUPERVISOR_PORT)
        Assert-Sprint8CExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
        if ([string]::IsNullOrWhiteSpace($FixtureReceiptPath)) {
            throw "Existing-topology UAT requires -FixtureReceiptPath."
        }
        $resolvedFixture = Assert-Sprint8CFixtureReceipt `
            -Path $FixtureReceiptPath -ExpectedComposeProject $ComposeProject
    } else {
        Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject `
            -Authorized ([bool]$AuthorizeDisposableReset)
        $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject
    }
    $env:PLAYWRIGHT_BASE_URL = $ports.gateway_url
    $env:TESSARA_PLAYWRIGHT_ACCEPTANCE = "1"

    foreach ($predicate in $selectedPredicates) {
        $topologyPlan = Get-Sprint8CUatPredicateTopologyPlan -Predicate $predicate `
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
            Invoke-Sprint8CChildScript -ScriptPath "scripts/materialize-sprint-8c.ps1" `
                -Arguments $arguments | Out-Null
            $materialized = Get-Content -LiteralPath $materializationEvidence -Raw | ConvertFrom-Json -Depth 100
            if ([string]$materialized.state -cne "passed" -or
                [string]::IsNullOrWhiteSpace([string]$materialized.fixture_receipt_path)) {
                throw "Owned UAT reference materialization did not publish a fixture receipt."
            }
            $resolvedFixture = Assert-Sprint8CFixtureReceipt `
                -Path ([string]$materialized.fixture_receipt_path) -ExpectedComposeProject $ComposeProject
            $ports = Set-Sprint8CUatMaterializedTopologyEnvironment `
                -MaterializationReceipt $materialized -ExpectedComposeProject $ComposeProject
            $env:PLAYWRIGHT_BASE_URL = $ports.gateway_url
        }
        if ($requiresFixture -and $null -eq $resolvedFixture) {
            throw "Predicate '$($predicate.id)' requires an authenticated reference fixture receipt."
        }
        if ($requiresFixture) {
            Assert-Sprint8CUatReferenceTopologyReady -ComposePath $composePath `
                -ComposeProject $ComposeProject -GatewayUrl $ports.gateway_url `
                -Fixture $resolvedFixture | Out-Null
        }
        $predicateComposeProject = [string]$topologyPlan.compose_project
        $fixturePath = if ($null -eq $resolvedFixture) { "" } else { [string]$resolvedFixture.path }
        $predicateResults.Add((Invoke-Sprint8CUatPredicate -Predicate $predicate `
            -ResolvedComposeProject $ComposeProject -PredicateComposeProject $predicateComposeProject `
            -ChildEvidenceRoot $childEvidenceRoot -ExpectedSource $source `
            -ResolvedFixtureReceiptPath $fixturePath -ExistingTopology ([bool]$UseExistingTopology -or $ownedTopology) `
            -ResetAuthorized ([bool]$AuthorizeDisposableReset) -BuildSkipped ([bool]$SkipBuild)))
    }
    $verifiedAssertionEvidence = @(Assert-Sprint8CSelectedAssertionEvidence `
        -SelectedScenarios $selectedScenarios -EvidenceClaims $evidenceClaims `
        -PredicateResults @($predicateResults))
    if ($UseExistingTopology -or $ownedTopology) {
        Assert-Sprint8CExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
        if ($null -ne $resolvedFixture) {
            Assert-Sprint8CFixtureReceipt -Path $resolvedFixture.path `
                -ExpectedComposeProject $ComposeProject | Out-Null
        }
    }
} catch {
    $failure = $_
} finally {
    try {
        if ($ownedTopology) {
            $cleanup = Remove-Sprint8CProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized ([bool]$AuthorizeDisposableReset)
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
        } elseif ($UseExistingTopology) {
            Assert-Sprint8CExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"
                mode = "existing-topology-restored-and-retained"
            }
        } else {
            $resources = Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject
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
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$scenarioContractHash = (Get-FileHash -LiteralPath $scenarioContractPath -Algorithm SHA256).Hash.ToLowerInvariant()
$fixtureHash = if ($null -eq $resolvedFixture) { "none" } else { [string]$resolvedFixture.sha256 }
$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n$scenarioContractHash`n$fixtureHash`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
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
    scenario_assertions = @(ConvertTo-Sprint8CUatScenarioAssertionResults `
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
Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8C automated UAT predicates failed; retained evidence: $evidenceFullPath"
}

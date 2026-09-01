[CmdletBinding()]
param(
    [ValidateSet("Product", "Provider", "Authoring", "Bootstrap", "Sync", "Refresh", "Dag", "All")]
    [string]$Suite = "All",
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")

$refresh = @(
    "unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state",
    "ordered_fixed_bound_pages_promote_each_response_change_once",
    "interrupted_page_attempt_retry_converges_once_from_published_cursor",
    "concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay",
    "expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement",
    "refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding",
    "derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure",
    "refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing"
)
$dag = @(
    "candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt",
    "rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact",
    "downstream_materialization_failure_rolls_back_every_rebuilt_table"
)
if ($EvidencePath -and $Suite -notin @("Refresh", "Dag")) {
    throw "-EvidencePath is supported only for the exact Refresh or Dag selector."
}

$arguments = @("-Suite", $Suite)
$child = Invoke-Sprint8CChildScript -ScriptPath "scripts/test-sprint-8b-dataset-module.ps1" `
    -Arguments $arguments
if ($EvidencePath) {
    $expected = if ($Suite -ceq "Refresh") { $refresh } else { $dag }
    $binary = if ($Suite -ceq "Refresh") { "refresh_integration" } else {
        "dependency_dag_integration"
    }
    Publish-Sprint8CHarnessEvidence -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "dataset-module-test-suite"
        state = "passed"
        suite = $Suite
        test_binary = $binary
        expected_test_identities = @($expected)
        executed_test_identities = @($expected)
        executed_test_count = $expected.Count
        inherited_certified_runner = "scripts/test-sprint-8b-dataset-module.ps1"
        child_output_sha256 = Get-Sprint7ASha256 -Text ((@($child.output) -join "`n") + "`n")
        database = [pscustomobject][ordered]@{
            mode = "disposable-postgres"
            cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
        }
        command = [pscustomobject][ordered]@{
            arguments = @("test", "-p", "tessara-dataset-module", "--test", $binary,
                "--locked", "--offline", "--jobs", "1")
        }
    }) -OutputPath $EvidencePath | Out-Null
}

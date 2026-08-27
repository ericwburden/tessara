[CmdletBinding()]
param(
    [ValidateSet("Owner", "Bootstrap", "Provider", "Contract", "Gateway", "Failure", "All")]
    [string]$Suite = "All",
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$containerName = "tessara-s8c-response-tests-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
$databaseUrlWasPresent = Test-Path Env:DATABASE_URL
$databaseUrlBefore = $env:DATABASE_URL
. (Join-Path $PSScriptRoot "sprint-8c-cargo-test-integrity.ps1")
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")

$suiteContracts = [ordered]@{
    Owner = [pscustomobject][ordered]@{
        kind = "integration"; binary = "owner_persistence"; identities = @(
            "concurrent_different_save_input_with_one_key_applies_once_and_conflicts",
            "concurrent_identical_delete_is_one_apply_and_one_replay",
            "concurrent_identical_save_is_one_apply_and_one_replay",
            "concurrent_identical_start_is_one_apply_and_one_replay",
            "concurrent_identical_submit_is_one_apply_and_one_replay",
            "create_is_atomic_audited_evented_and_idempotent",
            "create_requires_the_exact_live_start_claim",
            "delete_authority_separates_respond_ownership_from_manage_scope",
            "expired_start_claim_cannot_create_a_response",
            "fresh_gateway_grant_replays_stable_request_and_preserves_initial_binding",
            "inaccessible_response_is_nondisclosing",
            "pinned_draft_saves_submits_and_exports_without_live_providers"
        )
    }
    Bootstrap = [pscustomobject][ordered]@{
        kind = "integration"; binary = "bootstrap_integration"; identities = @(
            "owner_bootstrap_authorizes_dataset_export_checkpoint",
            "signed_response_bootstrap_materializes_submits_and_replays"
        )
    }
    Provider = [pscustomobject][ordered]@{
        kind = "library"; binary = "lib"; identities = @(
            "bootstrap::tests::bootstrap_schema_is_owner_specific_and_rejects_empty_input",
            "event_provider::tests::workflow_consumer_checkpoint_ack_is_monotonic_and_rejects_unpublished_heads",
            "operational::tests::provider_observations_and_consumer_lag_fail_closed",
            "operational::tests::operational_projection_shares_provider_and_consumer_truth",
            "owner::tests::canonical_digest_is_order_independent_for_json_object_keys",
            "owner::tests::create_validation_rejects_duplicate_fields_and_noncanonical_digests",
            "product_api::tests::idempotency_header_is_bounded_and_required",
            "product_api::tests::fresh_gateway_grants_share_stable_mutation_identity_and_authority_changes_conflict",
            "product_api::tests::mutation_identity_binds_exact_wire_route_and_grant_identity",
            "product_api::tests::public_mutation_decode_is_strict_and_bounded",
            "product_api::tests::read_authority_preserves_scoped_and_global_manage_bindings",
            "product_api::tests::response_start_accepts_only_respond_or_manage_authority",
            "product_store::tests::access_is_exact_to_owner_delegation_or_managed_scope",
            "product_store::tests::field_validation_is_typed_and_option_bound",
            "provider_client::tests::provider_observation_is_sanitized_and_binding_specific",
            "provider_client::tests::provider_response_bound_rejects_declared_and_streamed_overflow",
            "reconciliation_provider::tests::reconciliation_retains_live_claim_and_abandons_only_after_expiry",
            "reverse_provider::tests::operations_status_reads_the_owned_export_sequence_column",
            "reverse_provider::tests::reverse_scope_uses_only_the_required_capability_binding",
            "tests::configuration_normalizes_and_rejects_every_bound",
            "tests::fresh_baseline_is_response_owned_and_has_no_cross_database_constraints",
            "tests::lifecycle_bootstrap_requires_the_versioned_accept_media_type",
            "tests::lifecycle_projection_matches_response_manifest_routes_and_assets",
            "tests::manifest_is_semantically_valid_and_declares_independent_ownership",
            "validation_fault::tests::active_fault_is_unavailable_outside_validation_profile",
            "validation_fault::tests::disabled_and_unknown_faults_are_exact"
        )
    }
    Contract = [pscustomobject][ordered]@{
        package = "tessara-module-contract"; kind = "library"; binary = "lib"
        filter = "tests::public_api_routes_require_a_nonempty_unique_declared_capability_set"
        identities = @(
            "tests::public_api_routes_require_a_nonempty_unique_declared_capability_set"
        )
    }
    Gateway = [pscustomobject][ordered]@{
        package = "tessara-api"; kind = "library"; binary = "lib"
        filter = "module_gateway::tests::public_api_"
        identities = @(
            "module_gateway::tests::public_api_capability_alternatives_authorize_either_declared_binding",
            "module_gateway::tests::public_api_grants_exclude_undeclared_actor_bindings"
        )
    }
    FailureUnit = [pscustomobject][ordered]@{
        kind = "library"; binary = "lib"; features = @("sprint-8c-validation-faults")
        filter = "validation_fault::tests::"; identities = @(
            "validation_fault::tests::disabled_and_unknown_faults_are_exact",
            "validation_fault::tests::mid_apply_fault_requires_exact_projection_and_owner_write"
        )
    }
    FailureBootstrap = [pscustomobject][ordered]@{
        kind = "integration"; binary = "bootstrap_integration"
        features = @("sprint-8c-validation-faults")
        filter = "mid_apply_validation_fault_rolls_back_complete_bootstrap_transaction"
        identities = @(
            "mid_apply_validation_fault_rolls_back_complete_bootstrap_transaction"
        )
    }
}

function Invoke-ExactResponseSuite {
    param([string]$SuiteName, $Contract)
    $selector = if ([string]$Contract.kind -ceq "library") { @("--lib") }
        else { @("--test", [string]$Contract.binary) }
    $packageProperty = $Contract.PSObject.Properties["package"]
    $package = if ($null -eq $packageProperty) { "tessara-response-module" }
        else { [string]$packageProperty.Value }
    $common = @("test", "-p", $package) + $selector +
        @("--locked", "--offline", "--jobs", "1")
    $featuresProperty = $Contract.PSObject.Properties["features"]
    if ($null -ne $featuresProperty -and @($featuresProperty.Value).Count -gt 0) {
        $common += @("--features", (@($featuresProperty.Value) -join ","))
    }
    $filterProperty = $Contract.PSObject.Properties["filter"]
    if ($null -ne $filterProperty -and
        -not [string]::IsNullOrWhiteSpace([string]$filterProperty.Value)) {
        $common += [string]$filterProperty.Value
    }
    $discoveryArguments = $common + @("--", "--list", "--format", "terse")
    $discoveryLines = [Collections.Generic.List[string]]::new()
    & cargo @discoveryArguments 2>&1 | ForEach-Object {
        $line = [string]$_; $discoveryLines.Add($line); Write-Host $line
    }
    if ($LASTEXITCODE -ne 0) { throw "Response $SuiteName test discovery failed." }
    $discovered = @($discoveryLines | ForEach-Object {
        if ([string]$_ -cmatch '^(?<identity>[A-Za-z0-9_:]+): test$') { $Matches['identity'] }
    })
    $expected = @($Contract.identities)
    if ((@($discovered | Sort-Object) -join "`n") -cne
            (@($expected | Sort-Object) -join "`n") -or
        @($discovered | Sort-Object -Unique).Count -ne $expected.Count) {
        throw "Response $SuiteName discovery was not set-equal to its frozen test identities."
    }
    $arguments = $common + @("--", "--format", "terse")
    $lines = [Collections.Generic.List[string]]::new()
    & cargo @arguments 2>&1 | ForEach-Object {
        $line = [string]$_; $lines.Add($line); Write-Host $line
    }
    if ($LASTEXITCODE -ne 0) { throw "Response $SuiteName tests failed." }
    Assert-Sprint8CCargoTestTranscript -OutputLines @($lines) `
        -ExpectedExecutedTestCount $expected.Count | Out-Null
    [pscustomobject][ordered]@{
        suite = $SuiteName; package = $package; test_binary = [string]$Contract.binary
        arguments = @($arguments); expected_test_identities = @($expected)
        executed_test_identities = @($discovered); executed_test_count = $discovered.Count
    }
}

$results = [Collections.Generic.List[object]]::new()
$cleanupSucceeded = $false
try {
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        throw "Disposable Response test container '$containerName' already exists."
    }
    docker run --name $containerName -e POSTGRES_USER=tessara_test `
        -e POSTGRES_PASSWORD=tessara_test -e POSTGRES_DB=tessara_test `
        -p "127.0.0.1::5432" -d postgres:16-alpine | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Could not start the Response test database." }
    $ready = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        docker exec $containerName pg_isready -U tessara_test -d tessara_test *> $null
        if ($LASTEXITCODE -eq 0) { $ready = $true; break }
        Start-Sleep -Seconds 1
    }
    if (-not $ready) { throw "Response test database did not become ready." }
    $portLine = docker port $containerName 5432/tcp
    if ($portLine -notmatch '127\.0\.0\.1:(\d+)$') {
        throw "Could not resolve the isolated Response test database port."
    }
    $env:DATABASE_URL = "postgres://tessara_test:tessara_test@127.0.0.1:$($Matches[1])/tessara_test"
    Push-Location $repoRoot
    try {
        $selected = if ($Suite -ceq "All") { @($suiteContracts.Keys) }
            elseif ($Suite -ceq "Failure") { @("FailureUnit", "FailureBootstrap") }
            else { @($Suite) }
        foreach ($name in $selected) {
            $results.Add((Invoke-ExactResponseSuite -SuiteName $name -Contract $suiteContracts[$name]))
        }
    } finally { Pop-Location }
} finally {
    if ($databaseUrlWasPresent) { $env:DATABASE_URL = $databaseUrlBefore }
    else { Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue }
    if ((docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") -ceq $containerName) {
        docker rm -f $containerName | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Could not remove the Response test database." }
    }
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        throw "Response test database teardown was not exact."
    }
    if ((Test-Path Env:DATABASE_URL) -ne $databaseUrlWasPresent -or
        ($databaseUrlWasPresent -and $env:DATABASE_URL -cne $databaseUrlBefore)) {
        throw "Response test database environment restoration was not exact."
    }
    $cleanupSucceeded = $true
}

if ($EvidencePath) {
    if (-not $cleanupSucceeded -or $results.Count -eq 0) {
        throw "Response test evidence cannot publish before successful execution and cleanup."
    }
    Publish-Sprint8CHarnessEvidence -Document ([pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8c"; proof = "response-module-test-suite"
        state = "passed"; requested_suite = $Suite; suites = @($results)
        executed_test_count = [int](($results | Measure-Object -Property executed_test_count -Sum).Sum)
        database = [pscustomobject][ordered]@{
            mode = "disposable-postgres"
            cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
        }
    }) -OutputPath $EvidencePath | Out-Null
}

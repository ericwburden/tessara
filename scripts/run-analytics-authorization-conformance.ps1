[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest,
    [switch]$SourceOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

$requiredProof = [ordered]@{
    "crates/tessara-module-contract/src/protocol.rs" = @(
        "authorization_validation_rejects_wrong_context_and_stale_revisions",
        "v2_resource_assertion_is_exact_and_provider_fresh",
        "capability_scope_bindings_never_form_a_cross_product"
    )
    "crates/tessara-api/src/dashboard_components_adapter.rs" = @(
        "validate_downstream_grant",
        "consumed_module_service_nonces"
    )
    "crates/tessara-api/src/analytics_authorization.rs" = @(
        "boundaries_intersect_on_governing_node",
        "disjoint_scopes_do_not_cross_product"
    )
}

function Assert-ConformanceSources {
    foreach ($entry in $requiredProof.GetEnumerator()) {
        $path = Join-Path $script:Sprint7ARepositoryRoot $entry.Key
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Required authorization proof source is missing: $($entry.Key)"
        }
        $source = Get-Content -LiteralPath $path -Raw
        foreach ($needle in $entry.Value) {
            if (-not $source.Contains($needle)) {
                throw "Required authorization proof '$needle' is missing from $($entry.Key)."
            }
        }
    }
}

if ($SelfTest) {
    Test-Sprint7AAcceptanceContract
    Assert-ConformanceSources
    Write-Host "Sprint 7A analytics authorization conformance self-test passed."
    return
}

Assert-ConformanceSources
$checks = [Collections.Generic.List[object]]::new()
$commands = @(
    @("cargo", @("test", "--locked", "-p", "tessara-module-contract", "protocol::tests::authorization_validation_rejects_wrong_context_and_stale_revisions", "--", "--exact")),
    @("cargo", @("test", "--locked", "-p", "tessara-module-contract", "protocol::tests::v2_resource_assertion_is_exact_and_provider_fresh", "--", "--exact")),
    @("cargo", @("test", "--locked", "-p", "tessara-components-contract"))
)
foreach ($command in $commands) {
    & $command[0] @($command[1])
    Assert-Sprint7A ($LASTEXITCODE -eq 0) "source_conformance" "$($command[0]) $($command[1] -join ' ')" $checks
}

if (-not $SourceOnly) {
    $ready = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/health/ready"
    Assert-Sprint7A ($ready.status -in 200, 204) "live_core_ready" "HTTP $($ready.status)" $checks
    foreach ($path in @(
        "/api/private/dashboard-components/catalog",
        "/api/private/dashboard-components/resolve",
        "/api/private/dashboard-components/render"
    )) {
        $response = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $path -Method POST -Body @{}
        Assert-Sprint7A ($response.status -in 401, 403, 422) "unauthenticated_service_denial" "$path rejected the browser request with HTTP $($response.status) without disclosing a service grant" $checks
    }
}

$result = [ordered]@{ schema_version = 1; evidence_kind = "tessara.sprint-7a.analytics-authorization-conformance"; generated_at = [DateTimeOffset]::UtcNow.ToString("o"); provider_boundary = "tessara.components.component-version/1.0.0"; source_only = [bool]$SourceOnly; checks = $checks; passed = $true }
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
}
$result | ConvertTo-Json -Depth 20

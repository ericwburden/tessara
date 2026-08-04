[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$SupervisorUrl = "http://127.0.0.1:8096",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7b-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint7BAcceptanceContract
    Write-Host "Sprint 7B smoke self-test passed."
    return
}

$checks = [Collections.Generic.List[object]]::new()
$ready = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/health/ready"
Assert-Sprint7A ($ready.status -in 200, 204) "core_ready" "HTTP $($ready.status)" $checks
$supervisorReady = Invoke-Sprint7ARequest -BaseUrl $SupervisorUrl -Path "/health/ready"
Assert-Sprint7A ($supervisorReady.status -in 200, 204) "supervisor_ready" "HTTP $($supervisorReady.status)" $checks

$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
try {
    $component = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/components/$($script:Sprint7BFixture.component_id)" -Token $token
    Assert-Sprint7A ($component.status -eq 200) "component_detail" "Reference Component detail is readable" $checks
    $componentDocument = $component.body | ConvertFrom-Json
    $version = @($componentDocument.versions | Where-Object { $_.id -eq $script:Sprint7BFixture.component_version_id }) | Select-Object -First 1
    Assert-Sprint7A ($null -ne $version -and $version.status -eq "published" -and $version.lifecycle_state -eq "active" -and [int64]$version.resource_revision -gt 0) "component_lifecycle_contract" "Published reference version exposes separate publication, lifecycle, and positive revision fields" $checks

    $dashboard = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint7BFixture.dashboard_id)" -Token $token
    Assert-Sprint7A ($dashboard.status -eq 200 -and $dashboard.body.Contains($script:Sprint7BFixture.placement_id) -and $dashboard.body.Contains($script:Sprint7BFixture.component_version_id)) "dashboard_pinned_reference" "Reference Dashboard retains the exact ComponentVersion reference" $checks

    $health = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/dashboards/$($script:Sprint7BFixture.dashboard_id)/dependencies" -Token $token
    Assert-Sprint7A ($health.status -eq 200) "dependency_health" "Dashboard-owned dependency health is readable" $checks
    $healthDocument = $health.body | ConvertFrom-Json
    foreach ($property in @("health", "open_count", "deferred_count", "findings")) {
        Assert-Sprint7BJsonProperty -Object $healthDocument -Name $property -Context "dependency health"
    }
    Assert-Sprint7A ($healthDocument.health -in "healthy", "degraded") "dependency_health_shape" "Health is a typed healthy/degraded projection" $checks
} finally {
    if (-not [string]::IsNullOrWhiteSpace($token)) {
        $logout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token
        Assert-Sprint7A ($logout.status -in 200, 204) "session_cleanup" "Acceptance session was explicitly logged out" $checks
    }
}

$result = [ordered]@{
    schema_version = 1
    evidence_kind = "tessara.sprint-7b.smoke"
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    base_url = $BaseUrl.TrimEnd('/')
    checks = $checks
    passed = $true
}
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
}
$result | ConvertTo-Json -Depth 20

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
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint7AAcceptanceContract
    foreach ($name in @("dataset_id", "metric_component_id", "table_component_id", "chart_component_id", "dashboard_id", "blocked_dashboard_id")) {
        try {
            $null = [guid]::ParseExact([string]$script:Sprint7AFixture[$name], "D")
        } catch {
            throw "Sprint 7A smoke fixture '$name' is not a UUID."
        }
    }
    Write-Host "Sprint 7A smoke self-test passed."
    return
}

$checks = [Collections.Generic.List[object]]::new()
$ready = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/health/ready"
Assert-Sprint7A ($ready.status -in 200, 204) "core_ready" "HTTP $($ready.status)" $checks
$supervisorReady = Invoke-Sprint7ARequest -BaseUrl $SupervisorUrl -Path "/health/ready"
Assert-Sprint7A ($supervisorReady.status -in 200, 204) "supervisor_ready" "HTTP $($supervisorReady.status)" $checks

$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
$datasets = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/datasets" -Token $token
Assert-Sprint7A ($datasets.status -eq 200 -and $datasets.body.Contains($script:Sprint7AFixture.dataset_id)) "dataset_inventory" "Reference Dataset is present" $checks
$components = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/components" -Token $token
Assert-Sprint7A ($components.status -eq 200 -and $components.body.Contains($script:Sprint7AFixture.metric_component_id) -and $components.body.Contains($script:Sprint7AFixture.table_component_id) -and $components.body.Contains($script:Sprint7AFixture.chart_component_id)) "component_inventory" "Reference table, chart, and stat Components are present" $checks
$dashboard = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint7AFixture.dashboard_id)" -Token $token
Assert-Sprint7A ($dashboard.status -eq 200 -and $dashboard.body.Contains($script:Sprint7AFixture.metric_placement_id) -and $dashboard.body.Contains($script:Sprint7AFixture.table_placement_id) -and $dashboard.body.Contains($script:Sprint7AFixture.chart_placement_id) -and $dashboard.body.Contains($script:Sprint7AFixture.blocked_placement_id)) "dashboard_inventory" "Reference Dashboard has the exact mixed-placement inventory" $checks
$stat = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint7AFixture.dashboard_id)/placements/$($script:Sprint7AFixture.metric_placement_id)/render/stat-card" -Token $token
Assert-Sprint7A ($stat.status -eq 200 -and $stat.body.Contains('"display_value":"4"') -and $stat.body.Contains('"materialization_state":"ready"')) "dashboard_stat_render" "Administrator-mediated stat-card render is ready and includes all four tiers" $checks
$table = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint7AFixture.dashboard_id)/placements/$($script:Sprint7AFixture.table_placement_id)/render/table" -Token $token
Assert-Sprint7A ($table.status -eq 200 -and $table.body.Contains("UAT7A-PUBLIC") -and $table.body.Contains("UAT7A-CONFIDENTIAL-BLOCKED") -and $table.body.Contains('"materialization_state":"ready"')) "dashboard_table_render" "Administrator-mediated table render contains the complete four-tier fixture" $checks

$result = [ordered]@{ schema_version = 1; evidence_kind = "tessara.sprint-7a.smoke"; generated_at = [DateTimeOffset]::UtcNow.ToString("o"); base_url = $BaseUrl.TrimEnd('/'); checks = $checks; passed = $true }
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
}
$result | ConvertTo-Json -Depth 20

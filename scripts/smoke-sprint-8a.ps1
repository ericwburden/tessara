[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint8AAcceptanceContract
    foreach ($path in @(
        "deploy/sprint-8a/blueprints/reference.json",
        "deploy/sprint-8a/catalogs/local-release-catalog.json",
        "crates/tessara-component-module/manifest.json"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $script:Sprint7ARepositoryRoot $path) -PathType Leaf)) {
            throw "Sprint 8A smoke input is missing: $path"
        }
    }
    Write-Host "Sprint 8A smoke self-test passed."
    return
}

Test-Sprint8AAcceptanceContract
$checks = [Collections.Generic.List[object]]::new()
$ready = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/health/ready"
Assert-Sprint7A ($ready.status -in 200, 204) "core_ready" "HTTP $($ready.status)" $checks
$supervisorReady = Invoke-Sprint7ARequest -BaseUrl $SupervisorUrl -Path "/health/ready"
Assert-Sprint7A ($supervisorReady.status -in 200, 204) "supervisor_ready" "HTTP $($supervisorReady.status)" $checks

$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
$componentsResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/components" -Token $token
Assert-Sprint7A ($componentsResponse.status -eq 200) "component_inventory_status" "HTTP $($componentsResponse.status)" $checks
$components = @($componentsResponse.body | ConvertFrom-Json)
$actualKinds = @($components | ForEach-Object { $_.current_version.component_type } | Sort-Object -Unique)
$expectedKinds = @($script:Sprint8AFixture.component_versions.Keys | Sort-Object)
Assert-Sprint7A (($actualKinds -join ",") -ceq ($expectedKinds -join ",")) "component_kind_inventory" "All six canonical Component kinds are present" $checks
foreach ($component in $components) {
    $reference = $component.current_version.reference.reference
    Assert-Sprint7A (
        $reference.installation_id -ceq $script:Sprint8AFixture.installation_id -and
        $reference.owner.kind -ceq "module_instance" -and
        $reference.owner.module_instance_id -ceq $script:Sprint8AFixture.component_module_instance_id -and
        $reference.resource_type -ceq $script:Sprint8AFixture.component_resource_type
    ) "component_reference_$($component.slug)" "Component uses the selected v3 module owner/type" $checks
}

$navigation = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/shell/navigation" -Token $token
Assert-Sprint7A ($navigation.status -eq 200 -and $navigation.body.Contains('"href":"/components"') -and $navigation.body.Contains('"label":"Components"')) "manifest_navigation" "Manifest-owned Component navigation is visible" $checks
$componentDocument = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/components"
Assert-Sprint7A ($componentDocument.status -eq 200 -and $componentDocument.body.Contains('content="tessara.components"') -and $componentDocument.body.Contains('id="module-content"')) "component_document" "Complete document is Component-owned" $checks

$dashboardResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint8AFixture.dashboard_id)" -Token $token
Assert-Sprint7A ($dashboardResponse.status -eq 200) "dashboard_status" "HTTP $($dashboardResponse.status)" $checks
$dashboard = $dashboardResponse.body | ConvertFrom-Json
foreach ($placement in @($dashboard.placements)) {
    $reference = $placement.component.reference.reference
    Assert-Sprint7A (
        $reference.owner.kind -ceq "module_instance" -and
        $reference.owner.module_instance_id -ceq $script:Sprint8AFixture.component_module_instance_id -and
        $reference.resource_type -ceq $script:Sprint8AFixture.component_resource_type
    ) "dashboard_reference_$($placement.placement_id)" "Dashboard placement uses Components v3" $checks
}

foreach ($render in @(
    [ordered]@{ placement = $script:Sprint8AFixture.stat_placement_id; kind = "stat-card" },
    [ordered]@{ placement = $script:Sprint8AFixture.table_placement_id; kind = "table" }
)) {
    $response = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint8AFixture.dashboard_id)/placements/$($render.placement)/render/$($render.kind)" -Token $token
    Assert-Sprint7A ($response.status -eq 200 -and $response.body.Contains('"materialization_state":"ready"')) "dashboard_render_$($render.kind)" "Dashboard-mediated Component render is ready" $checks
}

$componentExecution = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/components/sprint-8a-record-table/table" -Token $token
Assert-Sprint7A (
    $componentExecution.status -eq 200 -and
    $componentExecution.body.Contains('"dataset_reference":') -and
    -not $componentExecution.body.Contains('"dataset_id":') -and
    $componentExecution.body.Contains('"materialization_state":"ready"')
) "component_execution_contract" "Component-owned stable execution route returns the typed Dataset reference contract" $checks
$oldPayload = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/components" -Method POST -Token $token -Body @{
    name = "Obsolete payload"
    slug = "obsolete-payload"
    dataset_id = $script:Sprint8AFixture.dataset_id
    component_type = "table"
    config = @{ visible_columns = @("label") }
}
Assert-Sprint7A ($oldPayload.status -in 400, 422) "old_core_payload_rejected" "Old Core request shape is rejected" $checks

$logout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token
Assert-Sprint7A ($logout.status -in 200, 204) "session_cleanup" "Acceptance session was explicitly logged out" $checks

$result = [ordered]@{
    schema_version = 1
    evidence_kind = "tessara.sprint-8a.smoke"
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    base_url = $BaseUrl.TrimEnd('/')
    checks = $checks
    passed = $true
}
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
}
$result | ConvertTo-Json -Depth 20

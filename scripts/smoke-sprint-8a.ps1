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
. (Join-Path $PSScriptRoot "sprint-8a-health-contract.ps1")

function Test-Sprint8ADashboardPlacementProjection {
    param(
        [Parameter(Mandatory)]$Placement,
        [Parameter(Mandatory)]$Expected
    )

    $propertyNames = @($Placement.PSObject.Properties.Name)
    $geometryMatches = [int]$Placement.grid_row -eq [int]$Expected.grid_row -and
        [int]$Placement.grid_column -eq [int]$Expected.grid_column -and
        [int]$Placement.grid_width -eq [int]$Expected.grid_width -and
        [int]$Placement.grid_height -eq [int]$Expected.grid_height
    $baseMatches = $propertyNames -cnotcontains "placement_key" -and
        $geometryMatches -and
        [string]$Placement.resolution_state -ceq [string]$Expected.resolution_state -and
        [string]$Placement.availability -ceq [string]$Expected.availability

    if ([string]$Expected.disclosure -ceq "restricted") {
        $encoded = $Placement | ConvertTo-Json -Depth 20 -Compress
        $passed = $baseMatches -and
            $propertyNames -cnotcontains "component" -and
            $propertyNames -cnotcontains "title" -and
            -not $encoded.Contains('"resource_id":') -and
            [string]$Placement.resolution.access_state -ceq "unauthorized" -and
            [string]$Placement.resolution.owner_state.kind -ceq "undisclosed" -and
            [string]$Placement.resolution.resource_identity_state -ceq "undisclosed"
        return [pscustomobject][ordered]@{
            passed = $passed
            detail = "restricted placement preserves exact geometry and discloses no bootstrap key, title, or Component identity"
        }
    }

    $reference = if ($propertyNames -ccontains "component") {
        $Placement.component.reference.reference
    } else {
        $null
    }
    $passed = $baseMatches -and $null -ne $reference -and
        [string]$reference.installation_id -ceq $script:Sprint8AFixture.installation_id -and
        [string]$reference.owner.kind -ceq "module_instance" -and
        [string]$reference.owner.installation_id -ceq $script:Sprint8AFixture.installation_id -and
        [string]$reference.owner.module_instance_id -ceq $script:Sprint8AFixture.component_module_instance_id -and
        [string]$reference.resource_type -ceq $script:Sprint8AFixture.component_resource_type -and
        [string]$reference.resource_id -ceq [string]$Expected.component_version_id
    [pscustomobject][ordered]@{
        passed = $passed
        detail = "placement preserves exact geometry and receipt-bound Components v3 identity without exposing bootstrap-only placement_key"
    }
}

function Test-Sprint8ADashboardPlacementProjectionContract {
    $authorizedExpected = $script:Sprint8AFixture.dashboard_placements[$script:Sprint8AFixture.stat_placement_id]
    $authorized = [pscustomobject][ordered]@{
        placement_id = $script:Sprint8AFixture.stat_placement_id
        grid_row = 1; grid_column = 1; grid_width = 4; grid_height = 2
        availability = "available"; resolution_state = "available"
        resolution = [pscustomobject]@{ access_state = "authorized"; owner_state = [pscustomobject]@{ kind = "module_instance" }; resource_identity_state = "resolved" }
        component = [pscustomobject]@{ reference = [pscustomobject]@{ reference = [pscustomobject]@{
            installation_id = $script:Sprint8AFixture.installation_id
            owner = [pscustomobject]@{ kind = "module_instance"; installation_id = $script:Sprint8AFixture.installation_id; module_instance_id = $script:Sprint8AFixture.component_module_instance_id }
            resource_type = $script:Sprint8AFixture.component_resource_type
            resource_id = $authorizedExpected.component_version_id
        } } }
    }
    if (-not (Test-Sprint8ADashboardPlacementProjection -Placement $authorized -Expected $authorizedExpected).passed) {
        throw "Sprint 8A smoke projection rejected an exact authorized placement without placement_key."
    }
    $authorized | Add-Member -NotePropertyName placement_key -NotePropertyValue "row-count"
    if ((Test-Sprint8ADashboardPlacementProjection -Placement $authorized -Expected $authorizedExpected).passed) {
        throw "Sprint 8A smoke projection accepted a bootstrap-only placement_key in the public response."
    }
    $authorized.PSObject.Properties.Remove("placement_key")
    $authorized.component.reference.reference.resource_id = $script:Sprint8AFixture.component_versions.table
    if ((Test-Sprint8ADashboardPlacementProjection -Placement $authorized -Expected $authorizedExpected).passed) {
        throw "Sprint 8A smoke projection accepted a same-count Component identity swap."
    }
    $authorized.component.reference.reference.resource_id = $authorizedExpected.component_version_id
    $authorized.grid_width = 12
    if ((Test-Sprint8ADashboardPlacementProjection -Placement $authorized -Expected $authorizedExpected).passed) {
        throw "Sprint 8A smoke projection accepted fallback rather than canonical seed geometry."
    }

    $restrictedExpected = $script:Sprint8AFixture.dashboard_placements["01980000-0003-7000-8000-000000000005"]
    $restricted = [pscustomobject][ordered]@{
        placement_id = "01980000-0003-7000-8000-000000000005"
        grid_row = 9; grid_column = 7; grid_width = 6; grid_height = 4
        availability = "unavailable"; resolution_state = "restricted"
        resolution = [pscustomobject]@{ access_state = "unauthorized"; owner_state = [pscustomobject]@{ kind = "undisclosed" }; resource_identity_state = "undisclosed" }
    }
    if (-not (Test-Sprint8ADashboardPlacementProjection -Placement $restricted -Expected $restrictedExpected).passed) {
        throw "Sprint 8A smoke projection rejected the exact restricted nondisclosure response."
    }
    $restricted | Add-Member -NotePropertyName component -NotePropertyValue $authorized.component
    if ((Test-Sprint8ADashboardPlacementProjection -Placement $restricted -Expected $restrictedExpected).passed) {
        throw "Sprint 8A smoke projection accepted restricted Component identity disclosure."
    }
}

if ($SelfTest) {
    Test-Sprint8AAcceptanceContract
    Test-Sprint8AHealthContract
    Test-Sprint8ADashboardPlacementProjectionContract
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
$ready = Invoke-Sprint8AHealthProbe -Target gateway_core -BaseUrl $BaseUrl
Assert-Sprint7A ([bool]$ready.passed) "core_ready" "HTTP $($ready.response.status); exact /health text/plain ok contract" $checks
$supervisorReady = Invoke-Sprint8AHealthProbe -Target supervisor -BaseUrl $SupervisorUrl
Assert-Sprint7A ([bool]$supervisorReady.passed) "supervisor_ready" "HTTP $($supervisorReady.response.status); exact /health/ready empty 204 contract" $checks

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
        $reference.owner.installation_id -ceq $script:Sprint8AFixture.installation_id -and
        $reference.owner.module_instance_id -ceq $script:Sprint8AFixture.component_module_instance_id -and
        $reference.resource_type -ceq $script:Sprint8AFixture.component_resource_type
    ) "component_reference_$($component.slug)" "Component uses the selected v3 module owner/type" $checks
}

$navigation = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/shell/navigation" -Token $token
Assert-Sprint7A ($navigation.status -eq 200 -and $navigation.body.Contains('"href":"/components"') -and $navigation.body.Contains('"label":"Components"')) "manifest_navigation" "Manifest-owned Component navigation is visible" $checks
$componentDocument = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/components" -Token $token
Assert-Sprint7A ($componentDocument.status -eq 200 -and $componentDocument.body.Contains('content="tessara.components"') -and $componentDocument.body.Contains('id="module-content"')) "component_document" "Complete document is Component-owned" $checks

$dashboardResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint8AFixture.dashboard_id)" -Token $token
Assert-Sprint7A ($dashboardResponse.status -eq 200) "dashboard_status" "HTTP $($dashboardResponse.status)" $checks
$dashboard = $dashboardResponse.body | ConvertFrom-Json
$expectedPlacementIds = @($script:Sprint8AFixture.dashboard_placements.Keys | Sort-Object)
$actualPlacementIds = @($dashboard.placements | ForEach-Object { [string]$_.placement_id } | Sort-Object)
Assert-Sprint7A (
    ($actualPlacementIds -join ",") -ceq ($expectedPlacementIds -join ",")
) "dashboard_placement_inventory" "Dashboard owner returned the exact seven receipt-bound placements" $checks
foreach ($placement in @($dashboard.placements)) {
    $expectedPlacement = $script:Sprint8AFixture.dashboard_placements[[string]$placement.placement_id]
    $projection = if ($null -eq $expectedPlacement) {
        [pscustomobject][ordered]@{ passed = $false; detail = "Dashboard returned an undeclared placement identity" }
    } else {
        Test-Sprint8ADashboardPlacementProjection -Placement $placement -Expected $expectedPlacement
    }
    Assert-Sprint7A ([bool]$projection.passed) "dashboard_reference_$($placement.placement_id)" ([string]$projection.detail) $checks
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
    -not $componentExecution.body.Contains('"dataset_reference":') -and
    -not $componentExecution.body.Contains('"dataset_id":') -and
    $componentExecution.body.Contains('"materialization_state":"ready"')
) "component_execution_contract" "Component-owned stable execution route returns only the Component render contract" $checks
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
    health = [ordered]@{
        core = $ready
        supervisor = $supervisorReady
    }
    checks = $checks
    passed = $true
}
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
}
$result | ConvertTo-Json -Depth 20

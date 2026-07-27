[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function ConvertTo-Sprint6CArray {
    param([AllowNull()][object]$Value)

    $items = @()
    foreach ($item in $Value) {
        $items += $item
    }
    return $items
}

if ($SelfTest) {
    if (@(ConvertTo-Sprint6CArray -Value @()).Count -ne 0) {
        throw "Empty JSON arrays must remain empty."
    }
    $fixture = [object[]]@(
        [pscustomobject]@{ id = "first" },
        [pscustomobject]@{ id = "second" }
    )
    $normalized = @(ConvertTo-Sprint6CArray -Value $fixture)
    if ($normalized.Count -ne 2 -or
        $normalized[0].id -cne "first" -or
        $normalized[1].id -cne "second") {
        throw "Non-empty JSON arrays must retain their item boundaries."
    }
    Write-Host "Sprint 6C seed JSON-array normalization self-test passed."
    exit 0
}

$login = $null
for ($attempt = 1; $attempt -le 30 -and $null -eq $login; $attempt++) {
    try {
        $login = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/auth/login" `
            -ContentType "application/json" `
            -Body (@{
                email = "admin@tessara.local"
                password = "tessara-dev-admin"
            } | ConvertTo-Json)
    } catch {
        if ($attempt -eq 30) {
            throw
        }
        Start-Sleep -Milliseconds 500
    }
}
$headers = @{
    Authorization = "Bearer $($login.token)"
}

$componentResponse = Invoke-RestMethod -Uri "$BaseUrl/api/components" -Headers $headers
$components = @(ConvertTo-Sprint6CArray -Value $componentResponse)
if ($components.Count -eq 0) {
    Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/demo/seed" -Headers $headers | Out-Null
}

$dashboardResponse = Invoke-RestMethod -Uri "$BaseUrl/api/dashboards" -Headers $headers
$dashboards = @(ConvertTo-Sprint6CArray -Value $dashboardResponse)
$dashboard = $dashboards |
    Where-Object {
        $null -ne $_ -and
        $null -ne $_.PSObject.Properties["name"] -and
        $_.name -eq "Demo Operations Dashboard"
    } |
    Select-Object -First 1
if ($null -eq $dashboard) {
    $visibilityResponse = Invoke-RestMethod `
        -Uri "$BaseUrl/api/admin/dashboards/visibility-nodes" -Headers $headers
    $visibility = @(ConvertTo-Sprint6CArray -Value $visibilityResponse)
    if ($visibility.Count -eq 0) {
        throw "Sprint 6C demo seed requires projected Organization nodes."
    }
    $createHeaders = $headers.Clone()
    $createHeaders["x-idempotency-key"] = "sprint-6c-demo-dashboard-v1"
    $created = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/admin/dashboards" `
        -Headers $createHeaders -ContentType "application/json" `
        -Body (@{
            name = "Demo Operations Dashboard"
            description = "Operational view of partner, program, activity, and session data."
            visibility_node_ids = @($visibility | ForEach-Object { $_.id })
        } | ConvertTo-Json -Depth 8)
    $dashboardId = $created.id
} else {
    $dashboardId = $dashboard.id
}

$composition = Invoke-RestMethod `
    -Uri "$BaseUrl/api/admin/dashboards/$dashboardId/composition" `
    -Headers $headers
$byName = @{}
foreach ($component in $composition.available_component_versions) {
    $byName[$component.component_name] = $component.component_version_id
}

$layout = @(
    @{ name = "Demo Partner Profile Table"; title = "Partner Profile"; row = 1; column = 1; width = 6; height = 4 },
    @{ name = "Demo Program Snapshot Table"; title = "Program Snapshot"; row = 1; column = 7; width = 6; height = 4 },
    @{ name = "Demo Activity Plan Table"; title = "Activity Plan"; row = 5; column = 1; width = 6; height = 4 },
    @{ name = "Demo Session Log Table"; title = "Session Log Table"; row = 9; column = 1; width = 12; height = 6 },
    @{ name = "Demo Session Participants Bar"; title = "Participants by Completion"; row = 15; column = 1; width = 6; height = 3 },
    @{ name = "Demo Session Participants Line"; title = "Participants Over Time"; row = 15; column = 7; width = 6; height = 2 },
    @{ name = "Demo Session Completion Pie"; title = "Completion Share"; row = 18; column = 1; width = 3; height = 3 },
    @{ name = "Demo Session Completion Donut"; title = "Completion Donut"; row = 18; column = 4; width = 3; height = 3 },
    @{ name = "Demo Session Total Participants StatCard"; title = "Total Participants"; row = 18; column = 7; width = 3; height = 2 }
)
$missing = @($layout | Where-Object { -not $byName.ContainsKey($_.name) })
if ($missing.Count -gt 0) {
    throw "Sprint 6C demo seed could not resolve: $($missing.name -join ', ')."
}

$alreadySeeded = $composition.dashboard.placements.Count -eq $layout.Count
if ($alreadySeeded) {
    foreach ($item in $layout) {
        $matching = @($composition.dashboard.placements | Where-Object {
            $_.component.component_name -eq $item.name -and
            $_.grid_row -eq $item.row -and
            $_.grid_column -eq $item.column -and
            $_.grid_width -eq $item.width -and
            $_.grid_height -eq $item.height -and
            $_.title -eq $item.title
        })
        if ($matching.Count -ne 1) {
            $alreadySeeded = $false
            break
        }
    }
}
if ($alreadySeeded) {
    [pscustomobject][ordered]@{
        seed_version = "sprint-6c-demo-v1"
        dashboard_id = $dashboardId
        dashboard_placements = $composition.dashboard.placements.Count
        dashboard_database = "tessara_module_dashboards"
    } | ConvertTo-Json
    exit 0
}

$commands = @()
foreach ($placement in $composition.dashboard.placements) {
    $commands += @{
        operation = "remove"
        placement_id = $placement.placement_id
    }
}
for ($index = 0; $index -lt $layout.Count; $index++) {
    $item = $layout[$index]
    $commands += @{
        operation = "bind"
        client_key = "sprint-6c-demo-$index"
        component_version_id = $byName[$item.name]
        geometry = @{
            grid_row = $item.row
            grid_column = $item.column
            grid_width = $item.width
            grid_height = $item.height
        }
        title = $item.title
    }
}

$saveHeaders = $headers.Clone()
$saveHeaders["x-idempotency-key"] = "sprint-6c-demo-composition-v1"
$saved = Invoke-RestMethod -Method Put `
    -Uri "$BaseUrl/api/admin/dashboards/$dashboardId/composition" `
    -Headers $saveHeaders -ContentType "application/json" `
    -Body (@{ commands = $commands } | ConvertTo-Json -Depth 12)
if ($saved.dashboard.placements.Count -ne 9) {
    throw "Sprint 6C demo Dashboard contains $($saved.dashboard.placements.Count) placements instead of 9."
}

[pscustomobject][ordered]@{
    seed_version = "sprint-6c-demo-v1"
    dashboard_id = $dashboardId
    dashboard_placements = $saved.dashboard.placements.Count
    dashboard_database = "tessara_module_dashboards"
} | ConvertTo-Json

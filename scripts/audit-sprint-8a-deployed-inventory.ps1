[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

$expectedTransitions = @(
    "tessara.datasets",
    "tessara.forms",
    "tessara.migration",
    "tessara.responses",
    "tessara.workflows"
)
$expectedNavigation = @(
    "Home", "Organization", "Forms", "Workflows", "Responses", "Operations",
    "Datasets", "Scoped Records", "Components", "Dashboards", "User Management",
    "Roles & Access", "Node Types", "Module Management", "Application Composition"
)

function Assert-Sprint8ADeployedInventory {
    param(
        [Parameter(Mandatory)][object]$Inventory,
        [Parameter(Mandatory)][object]$Navigation
    )

    if ([int]$Inventory.schema_version -ne 1) {
        throw "Sprint 8A deployed inventory must use schema version 1."
    }
    $transitions = @($Inventory.entries | Where-Object kind -CEQ "transitional_in_process" | ForEach-Object {
        [string]$_.descriptor.reserved_definition_id
    } | Sort-Object)
    if (($transitions -join "`n") -cne ($expectedTransitions -join "`n")) {
        throw "Sprint 8A deployed inventory must contain the exact five Core transition identities."
    }
    $dashboardInventory = @($Inventory.entries | Where-Object {
        $_.kind -ceq "independently_deployed" -and $_.definition.id -ceq "tessara.dashboards"
    })
    $anyDashboard = @($Inventory.entries | Where-Object {
        ($_.kind -ceq "independently_deployed" -and $_.definition.id -ceq "tessara.dashboards") -or
        ($_.kind -ceq "transitional_in_process" -and $_.descriptor.reserved_definition_id -ceq "tessara.dashboards")
    })
    if ($dashboardInventory.Count -ne 1 -or $anyDashboard.Count -ne 1 -or
        [string]$dashboardInventory[0].release.version -cne "3.0.0" -or
        [string]::IsNullOrWhiteSpace([string]$dashboardInventory[0].instance.id)) {
        throw "Dashboard must appear exactly once through its real 3.0.0 Module Release and live Module Instance."
    }

    if ([int]$Navigation.schema_version -ne 3 -or [string]$Navigation.state -cne "available") {
        throw "Sprint 8A shell navigation must be an available schema-v3 document."
    }
    $items = @($Navigation.groups | ForEach-Object { $_.items })
    $dashboardNavigation = @($items | Where-Object {
        $_.key -ceq "tessara.dashboards.navigation" -or
        $_.contribution_id -ceq "tessara.dashboards.navigation" -or
        $_.href -ceq "/dashboards"
    })
    if ($dashboardNavigation.Count -ne 1 -or
        [string]$dashboardNavigation[0].owner -cne "contribution" -or
        [string]$dashboardNavigation[0].contribution_id -cne "tessara.dashboards.navigation") {
        throw "Dashboard navigation must appear exactly once through its manifest contribution."
    }
    $labels = @($items | ForEach-Object { [string]$_.label })
    if (($labels -join "`n") -cne ($expectedNavigation -join "`n")) {
        throw "Sprint 8A deployed navigation ordering differs from the exact accepted identity order."
    }

    [ordered]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-8a.deployed-inventory-navigation"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        transition_identities = $transitions
        dashboard_inventory = @($dashboardInventory | ForEach-Object {
            [ordered]@{ definition_id = [string]$_.definition.id; release_version = [string]$_.release.version; instance_id = [string]$_.instance.id }
        })
        dashboard_navigation = @($dashboardNavigation)
        navigation_order = $labels
        passed = $true
    }
}

if ($SelfTest) {
    $entries = @($expectedTransitions | ForEach-Object {
        [pscustomobject]@{ kind = "transitional_in_process"; descriptor = [pscustomobject]@{ reserved_definition_id = $_ }; definition = $null }
    }) + @([pscustomobject]@{
        kind = "independently_deployed"
        descriptor = [pscustomobject]@{ reserved_definition_id = $null }
        definition = [pscustomobject]@{ id = "tessara.dashboards" }
        release = [pscustomobject]@{ version = "3.0.0" }
        instance = [pscustomobject]@{ id = "a6339e9f-1131-870e-aac6-18a8a01e4bbd" }
    })
    $items = @($expectedNavigation | ForEach-Object {
        if ($_ -ceq "Dashboards") {
            [pscustomobject]@{ key = "tessara.dashboards.navigation"; label = $_; href = "/dashboards"; owner = "contribution"; contribution_id = "tessara.dashboards.navigation" }
        } else {
            [pscustomobject]@{ key = $_.ToLowerInvariant(); label = $_; href = "/fixture"; owner = "core"; contribution_id = $null }
        }
    })
    $result = Assert-Sprint8ADeployedInventory `
        -Inventory ([pscustomobject]@{ schema_version = 1; entries = $entries }) `
        -Navigation ([pscustomobject]@{ schema_version = 3; state = "available"; groups = @([pscustomobject]@{ items = $items }) })
    if (-not $result.passed) { throw "Sprint 8A deployed inventory audit self-test failed." }
    Write-Host "Sprint 8A deployed inventory/navigation audit self-test passed."
    return
}

$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
try {
    $inventoryResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/modules" -Token $token
    if ($inventoryResponse.status -ne 200) { throw "Module inventory returned HTTP $($inventoryResponse.status)." }
    $navigationResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/shell/navigation" -Token $token
    if ($navigationResponse.status -ne 200) { throw "Shell navigation returned HTTP $($navigationResponse.status)." }
    $result = Assert-Sprint8ADeployedInventory `
        -Inventory ($inventoryResponse.body | ConvertFrom-Json) `
        -Navigation ($navigationResponse.body | ConvertFrom-Json)
    if ([string]::IsNullOrWhiteSpace($OutputPath)) {
        $result | ConvertTo-Json -Depth 12
    } else {
        Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
        Write-Host "Retained Sprint 8A deployed inventory/navigation evidence: $OutputPath"
    }
} finally {
    [void](Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token)
}

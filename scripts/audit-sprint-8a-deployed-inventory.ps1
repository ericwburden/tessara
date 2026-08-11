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
$expectedModules = [ordered]@{
    "tessara.components" = [ordered]@{ version = "1.0.1"; instance_id = "142a1ece-f74b-85f6-8ca0-92f4a02e9409"; contribution_id = "tessara.components.navigation"; href = "/components" }
    "tessara.dashboards" = [ordered]@{ version = "3.0.1"; instance_id = "a6339e9f-1131-870e-aac6-18a8a01e4bbd"; contribution_id = "tessara.dashboards.navigation"; href = "/dashboards" }
}

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
    $moduleInventory = [ordered]@{}
    foreach ($definitionId in $expectedModules.Keys) {
        $expected = $expectedModules[$definitionId]
        $real = @($Inventory.entries | Where-Object {
            $_.kind -ceq "independently_deployed" -and $_.definition.id -ceq $definitionId
        })
        $allRepresentations = @($Inventory.entries | Where-Object {
            ($_.kind -ceq "independently_deployed" -and $_.definition.id -ceq $definitionId) -or
            ($_.kind -ceq "transitional_in_process" -and $_.descriptor.reserved_definition_id -ceq $definitionId)
        })
        if ($real.Count -ne 1 -or $allRepresentations.Count -ne 1 -or
            [string]$real[0].release.version -cne [string]$expected.version -or
            [string]$real[0].instance.id -cne [string]$expected.instance_id) {
            throw "$definitionId must appear exactly once through its exact Module Release and Module Instance identity."
        }
        $moduleInventory[$definitionId] = $real[0]
    }

    if ([int]$Navigation.schema_version -ne 3 -or [string]$Navigation.state -cne "available") {
        throw "Sprint 8A shell navigation must be an available schema-v3 document."
    }
    $items = @($Navigation.groups | ForEach-Object { $_.items })
    $moduleNavigation = [ordered]@{}
    foreach ($definitionId in $expectedModules.Keys) {
        $expected = $expectedModules[$definitionId]
        $matches = @($items | Where-Object {
            $_.key -ceq [string]$expected.contribution_id -or
            $_.contribution_id -ceq [string]$expected.contribution_id -or
            $_.href -ceq [string]$expected.href
        })
        if ($matches.Count -ne 1 -or
            [string]$matches[0].owner -cne "contribution" -or
            [string]$matches[0].contribution_id -cne [string]$expected.contribution_id) {
            throw "$definitionId navigation must appear exactly once through its manifest contribution."
        }
        $moduleNavigation[$definitionId] = $matches[0]
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
        module_inventory = @($moduleInventory.GetEnumerator() | ForEach-Object {
            [ordered]@{ definition_id = [string]$_.Key; release_version = [string]$_.Value.release.version; instance_id = [string]$_.Value.instance.id }
        })
        module_navigation = @($moduleNavigation.GetEnumerator() | ForEach-Object { $_.Value })
        navigation_order = $labels
        passed = $true
    }
}

if ($SelfTest) {
    $entries = @($expectedTransitions | ForEach-Object {
        [pscustomobject]@{ kind = "transitional_in_process"; descriptor = [pscustomobject]@{ reserved_definition_id = $_ }; definition = $null }
    }) + @($expectedModules.GetEnumerator() | ForEach-Object {
        [pscustomobject]@{
            kind = "independently_deployed"
            descriptor = [pscustomobject]@{ reserved_definition_id = $null }
            definition = [pscustomobject]@{ id = [string]$_.Key }
            release = [pscustomobject]@{ version = [string]$_.Value.version }
            instance = [pscustomobject]@{ id = [string]$_.Value.instance_id }
        }
    })
    $items = @($expectedNavigation | ForEach-Object {
        if ($_ -ceq "Dashboards") {
            [pscustomobject]@{ key = "dashboards"; label = $_; href = "/dashboards"; owner = "contribution"; contribution_id = "tessara.dashboards.navigation" }
        } elseif ($_ -ceq "Components") {
            [pscustomobject]@{ key = "components"; label = $_; href = "/components"; owner = "contribution"; contribution_id = "tessara.components.navigation" }
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

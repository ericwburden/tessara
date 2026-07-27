[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6c/compose.yaml",
    [string]$ComposeOverrideFile = "deploy/sprint-6c/compose.override.yaml",
    [string]$DashboardName = "Demo Operations Dashboard",
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = Join-Path $repoRoot $ComposeFile
$composeOverridePath = Join-Path $repoRoot $ComposeOverrideFile
$dashboardDefinitionId = "tessara.dashboards"

function Get-AdminHeaders {
    $login = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/auth/login" `
        -ContentType "application/json" `
        -Body (@{
            email = "admin@tessara.local"
            password = "tessara-dev-admin"
        } | ConvertTo-Json)
    return @{ Authorization = "Bearer $($login.token)" }
}

function Wait-Core {
    for ($attempt = 1; $attempt -le 40; $attempt++) {
        try {
            $response = Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/health" `
                -TimeoutSec 3
            if ($response.StatusCode -eq 200) {
                return
            }
        } catch {
            if ($attempt -eq 40) {
                throw
            }
        }
        Start-Sleep -Milliseconds 500
    }
    throw "Core did not become healthy."
}

function Set-ProviderState {
    param([Parameter(Mandatory)][string]$State)

    $env:TESSARA_COMPONENTS_PROVIDER_STATE = $State
    & docker compose -f $composePath -f $composeOverridePath up -d --no-deps `
        --force-recreate core | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Core could not be recreated for Components provider state '$State'."
    }
    Wait-Core
}

function Set-DashboardEnablement {
    param(
        [Parameter(Mandatory)][string]$InstanceId,
        [Parameter(Mandatory)][bool]$Enabled
    )

    $headers = Get-AdminHeaders
    $response = Invoke-WebRequest -UseBasicParsing -Method Post `
        -Uri "$BaseUrl/api/modules/instances/$InstanceId/enablement/form" `
        -Headers $headers -ContentType "application/x-www-form-urlencoded" `
        -Body "enabled=$($Enabled.ToString().ToLowerInvariant())"
    if ($response.StatusCode -ne 200) {
        throw "Dashboard enablement could not be set to '$Enabled'."
    }
}

function Get-DashboardComposition {
    param([Parameter(Mandatory)][string]$DashboardId)

    for ($attempt = 1; $attempt -le 40; $attempt++) {
        try {
            $headers = Get-AdminHeaders
            return Invoke-RestMethod `
                -Uri "$BaseUrl/api/admin/dashboards/$DashboardId/composition" `
                -Headers $headers
        } catch {
            if ($attempt -eq 40) {
                throw
            }
        }
        Start-Sleep -Milliseconds 500
    }
    throw "Dashboard composition route did not become available."
}

Push-Location $repoRoot
try {
    $headers = Get-AdminHeaders
    $inventory = Invoke-RestMethod -Uri "$BaseUrl/api/admin/modules" -Headers $headers
    $module = @($inventory.entries | Where-Object {
        $_.kind -eq "independently_deployed" -and
        $_.definition.id -eq $dashboardDefinitionId
    })
    if ($module.Count -ne 1) {
        throw "Expected exactly one independently deployed Dashboard module."
    }
    $instanceId = [string]$module[0].instance.id
    $originalEnablement = [bool]$module[0].instance.enabled
    if (-not $originalEnablement) {
        Set-DashboardEnablement -InstanceId $instanceId -Enabled $true
    }

    $headers = Get-AdminHeaders
    $dashboardResponse = Invoke-RestMethod -Uri "$BaseUrl/api/dashboards" -Headers $headers
    $dashboard = $null
    $dashboardMatches = 0
    foreach ($candidate in $dashboardResponse) {
        if ($candidate.name -eq $DashboardName) {
            $dashboard = $candidate
            $dashboardMatches++
        }
    }
    if ($dashboardMatches -ne 1) {
        throw "Expected exactly one '$DashboardName' Dashboard."
    }
    $dashboardId = [string]$dashboard.id

    $states = [ordered]@{
        available            = "available"
        unavailable          = "provider_unavailable"
        incompatible         = "incompatible"
        inactive             = "inactive"
        superseded           = "superseded"
        tombstoned           = "tombstoned"
        owner_tombstoned     = "owner_tombstoned"
        owner_data_destroyed = "owner_data_destroyed"
        missing              = "missing"
        not_evaluated        = "not_evaluated"
    }
    $results = @()
    foreach ($state in $states.Keys) {
        Set-ProviderState -State $state
        $composition = Get-DashboardComposition -DashboardId $dashboardId
        $placements = @($composition.dashboard.placements)
        if ($placements.Count -ne 9) {
            throw "State '$state' returned $($placements.Count) placements instead of 9."
        }
        $expected = $states[$state]
        $unexpected = @($placements | Where-Object {
            [string]$_.resolution_state -cne $expected
        })
        if ($unexpected.Count -gt 0) {
            $observed = @($placements.resolution_state | Sort-Object -Unique) -join ", "
            throw "State '$state' expected '$expected' but observed: $observed."
        }
        $titles = @($placements | Where-Object {
            [string]::IsNullOrWhiteSpace([string]$_.title)
        })
        if ($titles.Count -gt 0) {
            throw "State '$state' did not retain every saved placement title."
        }
        $results += [pscustomobject][ordered]@{
            provider_state = $state
            resolution_state = $expected
            placement_count = $placements.Count
            saved_titles_retained = $true
        }
    }

    $evidence = [pscustomobject][ordered]@{
        schema_version = 1
        verified_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_commit = (& git rev-parse HEAD).Trim()
        source_tree_clean = @(& git status --porcelain).Count -eq 0
        dashboard_id = $dashboardId
        module_instance_id = $instanceId
        original_enablement = $originalEnablement
        scenarios = $results
    }
    $json = $evidence | ConvertTo-Json -Depth 8
    if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        $resolvedEvidencePath = if ([IO.Path]::IsPathRooted($EvidencePath)) {
            $EvidencePath
        } else {
            Join-Path $repoRoot $EvidencePath
        }
        $evidenceDirectory = Split-Path -Parent $resolvedEvidencePath
        [IO.Directory]::CreateDirectory($evidenceDirectory) | Out-Null
        [IO.File]::WriteAllText(
            $resolvedEvidencePath,
            $json + [Environment]::NewLine,
            [Text.UTF8Encoding]::new($false)
        )
        $digest = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData(
                [IO.File]::ReadAllBytes($resolvedEvidencePath)
            )
        ).ToLowerInvariant()
        [IO.File]::WriteAllText(
            "$resolvedEvidencePath.sha256",
            "$digest  $([IO.Path]::GetFileName($resolvedEvidencePath))" +
                [Environment]::NewLine,
            [Text.UTF8Encoding]::new($false)
        )
    }
    $json
} finally {
    try {
        Set-ProviderState -State "available"
    } finally {
        if ($null -ne (Get-Variable -Name originalEnablement -ErrorAction SilentlyContinue) `
            -and $null -ne (Get-Variable -Name instanceId -ErrorAction SilentlyContinue)) {
            Set-DashboardEnablement -InstanceId $instanceId `
                -Enabled $originalEnablement
        }
        Pop-Location
    }
}

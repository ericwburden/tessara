[CmdletBinding()]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-dashboard-dependency-contract.ps1")
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-8a"
$fixture = [ordered]@{
    dashboard_id = "01980000-0003-7000-8000-000000000001"
    blocked_placement_id = "01980000-0003-7000-8000-000000000005"
    upgrade_placement_id = "01980000-0003-7000-8000-000000000006"
    replace_placement_id = "01980000-0003-7000-8000-000000000007"
    remove_placement_id = "01980000-0003-7000-8000-000000000008"
    outage_placement_ids = @(
        "01980000-0003-7000-8000-000000000002",
        "01980000-0003-7000-8000-000000000003",
        "01980000-0003-7000-8000-000000000004",
        "01980000-0003-7000-8000-000000000006",
        "01980000-0003-7000-8000-000000000007"
    )
}

function Assert-Condition {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Code,
        [Parameter(Mandatory)][string]$Detail,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Checks
    )
    $Checks.Add([ordered]@{
        code = $Code
        state = if ($Condition) { "passed" } else { "failed" }
        passed = $Condition
        detail = $Detail
        dependency_reason = $null
    })
}

function Add-FailedCheck {
    param(
        [Parameter(Mandatory)][string]$Code,
        [Parameter(Mandatory)][string]$Detail,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Checks
    )
    $Checks.Add([ordered]@{
        code = $Code
        state = "failed"
        passed = $false
        detail = $Detail
        dependency_reason = $null
    })
}

function Add-BlockedCheck {
    param(
        [Parameter(Mandatory)][string]$Code,
        [Parameter(Mandatory)][string]$DependencyReason,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Checks
    )
    $Checks.Add([ordered]@{
        code = $Code
        state = "blocked"
        passed = $false
        detail = "Check was not executed because a required prerequisite did not pass."
        dependency_reason = $DependencyReason
    })
}

function Publish-DashboardDependencyEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [string]$Path
    )
    if ([string]::IsNullOrWhiteSpace($Path)) { return $null }
    Publish-Sprint7AEvidence -Document $Document -OutputPath $Path
}

function Invoke-JsonRequest {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Token,
        [ValidateSet("GET", "POST", "DELETE")][string]$Method = "GET",
        [object]$Body,
        [string]$IdempotencyKey
    )
    $headers = @{
        Authorization = "Bearer $Token"
        "x-tessara-correlation-id" = [Guid]::NewGuid().ToString("D")
    }
    if (-not [string]::IsNullOrWhiteSpace($IdempotencyKey)) {
        $headers["x-idempotency-key"] = $IdempotencyKey
    }
    $parameters = @{
        Uri = "$($BaseUrl.TrimEnd('/'))$Path"
        Method = $Method
        Headers = $headers
        UseBasicParsing = $true
    }
    if ((Get-Command Invoke-WebRequest).Parameters.ContainsKey("SkipHttpErrorCheck")) {
        $parameters.SkipHttpErrorCheck = $true
    }
    if ($null -ne $Body) {
        $parameters.ContentType = "application/json"
        $parameters.Body = $Body | ConvertTo-Json -Depth 30 -Compress
    }
    $response = Invoke-WebRequest @parameters
    if ([int]$response.StatusCode -notin 200, 201, 202, 204) {
        $bodyText = [string]$response.Content
        if ($bodyText.Length -gt 500) { $bodyText = $bodyText.Substring(0, 500) }
        throw "$Method $Path returned HTTP $([int]$response.StatusCode): $bodyText"
    }
    if ([int]$response.StatusCode -eq 204 -or [string]::IsNullOrWhiteSpace([string]$response.Content)) {
        return $null
    }
    return [string]$response.Content | ConvertFrom-Json
}

function Get-Token {
    $parameters = @{
        Uri = "$($BaseUrl.TrimEnd('/'))/api/auth/login"
        Method = "POST"
        ContentType = "application/json"
        Body = (@{ email = $AdminEmail; password = $AdminPassword } | ConvertTo-Json -Compress)
        UseBasicParsing = $true
    }
    $response = Invoke-WebRequest @parameters
    $document = [string]$response.Content | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace([string]$document.token)) {
        throw "Sprint 8A Dashboard dependency diagnostic login omitted its bearer token."
    }
    return [string]$document.token
}

function Get-FindingByPlacement {
    param(
        [Parameter(Mandatory)]$Health,
        [Parameter(Mandatory)][string]$PlacementId
    )
    $matches = @($Health.findings | Where-Object { [string]$_.placement_id -ceq $PlacementId })
    if ($matches.Count -ne 1) {
        throw "Expected one visible dependency finding for placement '$PlacementId', found $($matches.Count)."
    }
    return $matches[0]
}

function Invoke-DependencyAction {
    param(
        [Parameter(Mandatory)][string]$Token,
        [Parameter(Mandatory)]$Finding,
        [Parameter(Mandatory)][ValidateSet("defer", "upgrade", "replace", "remove")][string]$Action,
        [object]$Replacement
    )
    $body = [ordered]@{
        action = $Action
        expected_finding_revision = [long]$Finding.finding_revision
        replacement_component_reference = $Replacement
    }
    Invoke-JsonRequest `
        -Path "/api/admin/dashboards/$($fixture.dashboard_id)/dependencies/$($Finding.id)/actions" `
        -Method POST `
        -Token $Token `
        -IdempotencyKey "sprint-8a-dashboard-$Action-$([Guid]::NewGuid().ToString('N'))" `
        -Body $body
}

function Wait-ComponentHealthy {
    for ($attempt = 1; $attempt -le 60; $attempt++) {
        $containerId = [string](& docker compose -f $composePath --profile reference ps -q components)
        $containerId = $containerId.Trim()
        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($containerId)) {
            $status = [string](& docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' $containerId)
            $status = $status.Trim()
            if ($LASTEXITCODE -eq 0 -and $status -ceq "healthy") { return }
            if ($status -in "dead", "exited") { throw "Component container entered '$status'." }
        }
        Start-Sleep -Seconds 1
    }
    throw "Component did not become healthy within 60 seconds."
}

foreach ($name in @("dashboard_id", "blocked_placement_id", "upgrade_placement_id", "replace_placement_id", "remove_placement_id")) {
    $null = [Guid]::ParseExact([string]$fixture[$name], "D")
}
$expectedActionPlacements = @($fixture.upgrade_placement_id, $fixture.replace_placement_id, $fixture.remove_placement_id)
if (@($expectedActionPlacements | Sort-Object -Unique).Count -ne 3 -or
    @($fixture.outage_placement_ids | Sort-Object -Unique).Count -ne 5 -or
    $fixture.outage_placement_ids -contains $fixture.blocked_placement_id) {
    throw "Sprint 8A Dashboard dependency diagnostic fixture identities are inconsistent."
}
if ($SelfTest) {
    $probeChecks = [Collections.Generic.List[object]]::new()
    Assert-Condition -Condition $false -Code "probe_failure" -Detail "self-test failure" -Checks $probeChecks
    Add-BlockedCheck -Code "probe_blocked" -DependencyReason "probe_failure did not pass" -Checks $probeChecks
    if ($probeChecks.Count -ne 2 -or
        [string]$probeChecks[0].state -cne "failed" -or
        $probeChecks[0].passed -ne $false -or
        [string]$probeChecks[1].state -cne "blocked" -or
        [string]$probeChecks[1].dependency_reason -cne "probe_failure did not pass") {
        throw "Sprint 8A Dashboard dependency diagnostic does not retain fail-late failure and blocked-check evidence."
    }
    $probeRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-dashboard-diagnostic-$([Guid]::NewGuid().ToString('N'))"
    try {
        $probePath = Join-Path $probeRoot "raw-evidence.json"
        $publication = Publish-DashboardDependencyEvidence `
            -Document ([ordered]@{ checks = $probeChecks; harvesting_complete = $true; passed = $false }) `
            -Path $probePath
        $retained = Get-Content -LiteralPath $publication.path -Raw | ConvertFrom-Json
        if (-not (Test-Path -LiteralPath "$($publication.path).sha256" -PathType Leaf) -or
            @($retained.checks).Count -ne 2 -or
            [string]$retained.checks[0].state -cne "failed" -or
            [string]$retained.checks[1].state -cne "blocked") {
            throw "Sprint 8A Dashboard dependency diagnostic did not retain raw fail-late evidence and its digest sidecar."
        }
    } finally {
        Remove-Item -LiteralPath $probeRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
    Write-Host "Sprint 8A Dashboard dependency semantic diagnostic self-test passed."
    return
}

if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) { throw "Compose file not found: $composePath" }
$configuration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or [string]$configuration.name -cne $expectedProject) {
    throw "Dashboard dependency diagnostics are restricted to the exact $expectedProject Compose project."
}
$baseUri = [Uri]$BaseUrl
if ($baseUri.Scheme -notin @("http", "https") -or $baseUri.Host -notin @("127.0.0.1", "localhost", "::1")) {
    throw "Dashboard dependency diagnostics are restricted to an explicit loopback deployment."
}

$checks = [Collections.Generic.List[object]]::new()
$actions = [Collections.Generic.List[object]]::new()
$fatalErrors = [Collections.Generic.List[string]]::new()
$token = $null
$componentStopped = $false
$visibleInitialPlacements = @()
$outagePlacements = @()
$recovered = $null
Push-Location $repoRoot
try {
    $token = Get-Token
    $refreshPath = "/api/admin/dashboards/$($fixture.dashboard_id)/dependencies/refresh"
    $readPath = "/api/admin/dashboards/$($fixture.dashboard_id)/dependencies"
    $initial = $null
    try {
        $initial = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
        $visibleInitialPlacements = @($initial.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        Assert-Condition `
            -Condition (($visibleInitialPlacements -join ',') -ceq ((@($expectedActionPlacements) | Sort-Object) -join ',')) `
            -Code "exact_lifecycle_finding_placements" `
            -Detail "Only the three authorized action-fixture placements expose findings" `
            -Checks $checks
    } catch {
        Add-FailedCheck -Code "exact_lifecycle_finding_placements" -Detail $_.Exception.Message -Checks $checks
    }
    if ($null -eq $initial) {
        Add-BlockedCheck -Code "inactive_successor_findings" -DependencyReason "initial dependency refresh did not complete" -Checks $checks
        Add-BlockedCheck -Code "blocked_scope_nondisclosure" -DependencyReason "initial dependency refresh did not complete" -Checks $checks
    } else {
        Assert-Condition `
            -Condition (@($initial.findings | Where-Object { [string]$_.finding_code -cne "lifecycle_unrenderable" }).Count -eq 0) `
            -Code "inactive_successor_findings" `
            -Detail "All action fixtures expose lifecycle_unrenderable without leaking blocked scope" `
            -Checks $checks
        Assert-Condition `
            -Condition (@($initial.findings | Where-Object { [string]$_.placement_id -ceq $fixture.blocked_placement_id }).Count -eq 0) `
            -Code "blocked_scope_nondisclosure" `
            -Detail "The blocked placement is absent from the manager-visible finding response" `
            -Checks $checks
    }

    $upgradeFinding = $null
    if ($null -eq $initial) {
        Add-BlockedCheck -Code "successor_disclosed_for_authorized_finding" -DependencyReason "initial lifecycle findings are unavailable" -Checks $checks
    } else {
        try {
            $upgradeFinding = Get-FindingByPlacement -Health $initial -PlacementId $fixture.upgrade_placement_id
            Assert-Condition `
                -Condition ([bool]$upgradeFinding.successor_available) `
                -Code "successor_disclosed_for_authorized_finding" `
                -Detail "The inactive authorized version declares a provider-owned successor" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "successor_disclosed_for_authorized_finding" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $deferred = $null
    if ($null -eq $upgradeFinding) {
        Add-BlockedCheck -Code "defer_advances_finding" -DependencyReason "the upgrade fixture finding was not available" -Checks $checks
    } else {
        try {
            $deferred = Invoke-DependencyAction -Token $token -Finding $upgradeFinding -Action "defer" -Replacement $null
            Assert-Condition `
                -Condition ([string]$deferred.disposition -ceq "deferred" -and [long]$deferred.finding_revision -eq ([long]$upgradeFinding.finding_revision + 1)) `
                -Code "defer_advances_finding" `
                -Detail "Defer retains the finding and advances its exact revision" `
                -Checks $checks
            $actions.Add($deferred)
        } catch {
            Add-FailedCheck -Code "defer_advances_finding" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $deferredFinding = $null
    if ($null -eq $deferred) {
        Add-BlockedCheck -Code "deferred_finding_remains_actionable" -DependencyReason "defer did not produce a current finding revision" -Checks $checks
    } else {
        try {
            $afterDefer = Invoke-JsonRequest -Path $readPath -Token $token
            $deferredFinding = Get-FindingByPlacement -Health $afterDefer -PlacementId $fixture.upgrade_placement_id
            Assert-Condition `
                -Condition ([string]$deferredFinding.disposition -ceq "deferred" -and [long]$deferredFinding.finding_revision -eq [long]$deferred.finding_revision) `
                -Code "deferred_finding_remains_actionable" `
                -Detail "The deferred finding is retained at the returned revision" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "deferred_finding_remains_actionable" -Detail $_.Exception.Message -Checks $checks
        }
    }

    if ($null -eq $deferredFinding) {
        Add-BlockedCheck -Code "upgrade_uses_declared_successor" -DependencyReason "the deferred finding is not available at a verified revision" -Checks $checks
    } else {
        try {
            $upgraded = Invoke-DependencyAction -Token $token -Finding $deferredFinding -Action "upgrade" -Replacement $null
            Assert-Condition `
                -Condition ([string]$upgraded.disposition -ceq "resolved") `
                -Code "upgrade_uses_declared_successor" `
                -Detail "Upgrade resolves the deferred finding through the disclosed successor" `
                -Checks $checks
            $actions.Add($upgraded)
        } catch {
            Add-FailedCheck -Code "upgrade_uses_declared_successor" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $replacementComponent = @()
    try {
        $components = @(Invoke-JsonRequest -Path "/api/components" -Token $token)
        $replacementComponent = @($components | Where-Object { [string]$_.slug -ceq "sprint-8a-record-table" })
        Assert-Condition `
            -Condition ($replacementComponent.Count -eq 1 -and $null -ne $replacementComponent[0].current_version.reference) `
            -Code "replacement_reference_available" `
            -Detail "The exact authorized table replacement reference is available" `
            -Checks $checks
    } catch {
        Add-FailedCheck -Code "replacement_reference_available" -Detail $_.Exception.Message -Checks $checks
    }
    if ($null -eq $initial) {
        Add-BlockedCheck -Code "replace_uses_authorized_renderable_reference" -DependencyReason "initial lifecycle findings are unavailable" -Checks $checks
    } elseif ($replacementComponent.Count -ne 1 -or $null -eq $replacementComponent[0].current_version.reference) {
        Add-BlockedCheck -Code "replace_uses_authorized_renderable_reference" -DependencyReason "an exact authorized replacement reference was not available" -Checks $checks
    } else {
        try {
            $replaceFinding = Get-FindingByPlacement -Health $initial -PlacementId $fixture.replace_placement_id
            $replaced = Invoke-DependencyAction `
                -Token $token `
                -Finding $replaceFinding `
                -Action "replace" `
                -Replacement $replacementComponent[0].current_version.reference
            Assert-Condition `
                -Condition ([string]$replaced.disposition -ceq "resolved") `
                -Code "replace_uses_authorized_renderable_reference" `
                -Detail "Replace resolves its independent finding" `
                -Checks $checks
            $actions.Add($replaced)
        } catch {
            Add-FailedCheck -Code "replace_uses_authorized_renderable_reference" -Detail $_.Exception.Message -Checks $checks
        }
    }

    if ($null -eq $initial) {
        Add-BlockedCheck -Code "remove_consumes_independent_placement" -DependencyReason "initial lifecycle findings are unavailable" -Checks $checks
    } else {
        try {
            $removeFinding = Get-FindingByPlacement -Health $initial -PlacementId $fixture.remove_placement_id
            $removed = Invoke-DependencyAction -Token $token -Finding $removeFinding -Action "remove" -Replacement $null
            Assert-Condition `
                -Condition ([string]$removed.disposition -ceq "resolved") `
                -Code "remove_consumes_independent_placement" `
                -Detail "Remove resolves its independent finding and deletes its placement" `
                -Checks $checks
            $actions.Add($removed)
        } catch {
            Add-FailedCheck -Code "remove_consumes_independent_placement" -Detail $_.Exception.Message -Checks $checks
        }
    }

    try {
        & docker compose -f $composePath --profile reference stop components | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Could not stop the Component provider for the outage diagnostic." }
        $componentStopped = $true
        $outage = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
        $outagePlacements = @($outage.findings | Where-Object { [string]$_.finding_code -ceq "provider_unavailable" } | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        Assert-Condition `
            -Condition (($outagePlacements -join ',') -ceq ((@($fixture.outage_placement_ids) | Sort-Object) -join ',')) `
            -Code "provider_outage_is_contained" `
            -Detail "Exactly the five remaining authorized placements degrade without exposing blocked scope" `
            -Checks $checks
    } catch {
        Add-FailedCheck -Code "provider_outage_is_contained" -Detail $_.Exception.Message -Checks $checks
    }

    if (-not $componentStopped) {
        Add-BlockedCheck -Code "provider_recovery_converges" -DependencyReason "the provider outage was not established safely" -Checks $checks
    } else {
        try {
            & docker compose -f $composePath --profile reference start components | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "Could not restart the Component provider after the outage diagnostic." }
            Wait-ComponentHealthy
            $componentStopped = $false
            $recovered = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
            Assert-Condition `
                -Condition ([string]$recovered.health -ceq "healthy" -and [long]$recovered.open_count -eq 0 -and [long]$recovered.deferred_count -eq 0 -and @($recovered.findings).Count -eq 0) `
                -Code "provider_recovery_converges" `
                -Detail "A healthy provider refresh resolves every visible outage finding" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "provider_recovery_converges" -Detail $_.Exception.Message -Checks $checks
        }
    }
} catch {
    $fatalErrors.Add($_.Exception.Message)
} finally {
    if ($componentStopped) {
        & docker compose -f $composePath --profile reference start components | Out-Null
        if ($LASTEXITCODE -eq 0) {
            try { Wait-ComponentHealthy } catch { Write-Warning $_.Exception.Message }
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($token)) {
        try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $token | Out-Null } catch { }
    }
    Pop-Location
}

$recordedCodes = @($checks | ForEach-Object { [string]$_.code })
foreach ($missingCode in @($script:Sprint8ADashboardDependencyCheckCodes | Where-Object { $recordedCodes -cnotcontains $_ })) {
    $reason = if ($fatalErrors.Count -gt 0) {
        "unhandled prerequisite failure: $($fatalErrors -join '; ')"
    } else {
        "a failed prerequisite prevented safe dependent execution"
    }
    Add-BlockedCheck -Code $missingCode -DependencyReason $reason -Checks $checks
}
$duplicateCodes = @($checks | Group-Object code | Where-Object Count -ne 1 | ForEach-Object Name)
$failedChecks = @($checks | Where-Object { [string]$_.state -ceq "failed" })
$blockedChecks = @($checks | Where-Object { [string]$_.state -ceq "blocked" })
$passed = $fatalErrors.Count -eq 0 -and $duplicateCodes.Count -eq 0 -and
    $failedChecks.Count -eq 0 -and $blockedChecks.Count -eq 0 -and
    $checks.Count -eq $script:Sprint8ADashboardDependencyCheckCodes.Count
$evidence = [ordered]@{
    schema_version = 1
    evidence_kind = "tessara.sprint-8a.dashboard-dependency-semantic-diagnostic"
    dashboard_id = $fixture.dashboard_id
    checks = $checks
    actions = $actions
    initial_finding_placements = $visibleInitialPlacements
    outage_finding_placements = $outagePlacements
    final_health = [ordered]@{
        health = if ($null -eq $recovered) { "not_proven" } else { [string]$recovered.health }
        open_count = if ($null -eq $recovered) { -1 } else { [long]$recovered.open_count }
        deferred_count = if ($null -eq $recovered) { -1 } else { [long]$recovered.deferred_count }
    }
    failure_count = $failedChecks.Count + $fatalErrors.Count
    blocked_count = $blockedChecks.Count
    duplicate_check_codes = $duplicateCodes
    fatal_errors = $fatalErrors
    harvesting_complete = $true
    canonical_reset_required = $true
    passed = $passed
}
if ($passed) {
    Assert-Sprint8ADashboardDependencyEvidence -Evidence ([pscustomobject]$evidence)
}
$null = Publish-DashboardDependencyEvidence -Document $evidence -Path $OutputPath
$evidence | ConvertTo-Json -Depth 30
if (-not $passed) {
    throw "Sprint 8A Dashboard dependency diagnostic harvested $($failedChecks.Count + $fatalErrors.Count) failure(s) and $($blockedChecks.Count) blocked check(s)."
}

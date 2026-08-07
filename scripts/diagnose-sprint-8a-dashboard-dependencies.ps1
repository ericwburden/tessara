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
$fixture = $script:Sprint8ADashboardDependencyFixture

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
        [ValidateSet("GET", "POST", "PUT", "DELETE")][string]$Method = "GET",
        [object]$Body,
        [string]$IdempotencyKey,
        [switch]$IncludeStatus
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
    $statusCode = [int]$response.StatusCode
    if ($statusCode -notin 200, 201, 202, 204) {
        $bodyText = [string]$response.Content
        if ($bodyText.Length -gt 500) { $bodyText = $bodyText.Substring(0, 500) }
        throw "$Method $Path returned HTTP $statusCode`: $bodyText"
    }
    $document = if ($statusCode -eq 204 -or [string]::IsNullOrWhiteSpace([string]$response.Content)) {
        $null
    } else {
        [string]$response.Content | ConvertFrom-Json
    }
    if ($IncludeStatus) {
        return [pscustomobject][ordered]@{ status_code = $statusCode; body = $document }
    }
    return $document
}

function Get-Token {
    param(
        [string]$Email = $AdminEmail,
        [string]$Password = $AdminPassword
    )
    $parameters = @{
        Uri = "$($BaseUrl.TrimEnd('/'))/api/auth/login"
        Method = "POST"
        ContentType = "application/json"
        Body = (@{ email = $Email; password = $Password } | ConvertTo-Json -Compress)
        UseBasicParsing = $true
    }
    $response = Invoke-WebRequest @parameters
    $document = [string]$response.Content | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace([string]$document.token)) {
        throw "Sprint 8A Dashboard dependency diagnostic login omitted its bearer token."
    }
    return [string]$document.token
}

function Initialize-ComponentAccessBasis {
    param([Parameter(Mandatory)][string]$Token)

    $components = @(Invoke-JsonRequest -Path "/api/components" -Token $Token)
    if ($components.Count -lt 1) {
        throw "The Component provider did not return an access-basis catalog."
    }
    return $components
}

function Initialize-ContextIsolationActor {
    param([Parameter(Mandatory)][string]$AdminToken)

    $roles = @(Invoke-JsonRequest -Path "/api/admin/roles" -Token $AdminToken)
    $roleMatches = @($roles | Where-Object { [string]$_.name -ceq "reference-operator" })
    if ($roleMatches.Count -ne 1) {
        throw "The exact reference-operator role was not enrolled."
    }
    $roleId = [string]$roleMatches[0].id
    $users = @(Invoke-JsonRequest -Path "/api/admin/users" -Token $AdminToken)
    $matches = @($users | Where-Object { [string]$_.email -ceq $fixture.isolation_actor_email })
    if ($matches.Count -gt 1) {
        throw "The Dashboard context-isolation actor is not unique."
    }
    $password = "S8a!$([Guid]::NewGuid().ToString('N'))"
    $payload = [ordered]@{
        email = $fixture.isolation_actor_email
        display_name = "Sprint 8A Dashboard Context"
        password = $password
        is_active = $true
        role_ids = @($roleId)
    }
    if ($matches.Count -eq 0) {
        $actor = Invoke-JsonRequest -Path "/api/admin/users" -Method POST -Token $AdminToken -Body $payload
        $actorId = [string]$actor.id
    } else {
        $actorId = [string]$matches[0].id
        $null = Invoke-JsonRequest -Path "/api/admin/users/$actorId" -Method PUT -Token $AdminToken -Body $payload
    }
    $null = Invoke-JsonRequest `
        -Path "/api/admin/users/$actorId/access" `
        -Method PUT `
        -Token $AdminToken `
        -Body @{ scope_node_ids = @($fixture.scope_node_id); delegate_account_ids = @() }
    [pscustomobject][ordered]@{ actor_id = $actorId; password = $password }
}

function Get-OptionalPropertyValue {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) { return $null }
    return $property.Value
}

function Invoke-DashboardProjectionConvergence {
    param(
        [Parameter(Mandatory)][scriptblock]$Operation,
        [Parameter(Mandatory)][scriptblock]$Assertion,
        [Parameter(Mandatory)][string]$Label,
        [ValidateRange(1, 120)][int]$MaximumAttempts = 30,
        [ValidateRange(0, 30)][int]$DelaySeconds = 1
    )

    $lastFailure = "no observation"
    for ($attempt = 1; $attempt -le $MaximumAttempts; $attempt++) {
        try {
            $value = & $Operation
            & $Assertion $value
            return $value
        } catch {
            $lastFailure = $_.Exception.Message
        }
        if ($attempt -lt $MaximumAttempts -and $DelaySeconds -gt 0) {
            Start-Sleep -Seconds $DelaySeconds
        }
    }
    throw "$Label did not converge after $MaximumAttempts attempt(s). Last observation: $lastFailure"
}

function ConvertTo-ComponentReferenceIdentity {
    param([Parameter(Mandatory)]$Reference)

    $current = $Reference
    for ($depth = 0; $depth -lt 4; $depth++) {
        $nested = Get-OptionalPropertyValue -Value $current -Name "reference"
        if ($null -eq $nested) { break }
        $current = $nested
    }
    $owner = Get-OptionalPropertyValue -Value $current -Name "owner"
    if ($null -eq $owner) { throw "Component reference omitted its typed owner." }
    $identity = [ordered]@{
        installation_id = [string](Get-OptionalPropertyValue -Value $current -Name "installation_id")
        owner_kind = [string](Get-OptionalPropertyValue -Value $owner -Name "kind")
        owner_installation_id = [string](Get-OptionalPropertyValue -Value $owner -Name "installation_id")
        module_instance_id = [string](Get-OptionalPropertyValue -Value $owner -Name "module_instance_id")
        resource_type = [string](Get-OptionalPropertyValue -Value $current -Name "resource_type")
        resource_id = [string](Get-OptionalPropertyValue -Value $current -Name "resource_id")
    }
    foreach ($field in $identity.Keys) {
        if ([string]::IsNullOrWhiteSpace([string]$identity[$field])) {
            throw "Component reference omitted its '$field' identity."
        }
    }
    return [pscustomobject]$identity
}

function Get-DashboardCompositionSnapshot {
    param(
        [Parameter(Mandatory)][string]$Token,
        [Parameter(Mandatory)][string]$Stage
    )

    $composition = Invoke-JsonRequest -Path "/api/admin/dashboards/$($fixture.dashboard_id)/composition" -Token $Token
    $placements = @($composition.dashboard.placements)
    $referenceRows = [Collections.Generic.List[object]]::new()
    $nondisclosed = [Collections.Generic.List[string]]::new()
    foreach ($placement in @($placements | Sort-Object { [string]$_.placement_id })) {
        $placementId = [string]$placement.placement_id
        $component = Get-OptionalPropertyValue -Value $placement -Name "component"
        if ($null -eq $component) {
            $nondisclosed.Add($placementId)
            continue
        }
        $componentReference = Get-OptionalPropertyValue -Value $component -Name "reference"
        if ($null -eq $componentReference) {
            throw "Dashboard placement '$placementId' disclosed Component metadata without its typed reference."
        }
        $referenceRows.Add([ordered]@{
            placement_id = $placementId
            reference = ConvertTo-ComponentReferenceIdentity -Reference $componentReference
        })
    }
    $snapshot = [ordered]@{
        stage = $Stage
        placement_ids = @($placements | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        references = @($referenceRows.ToArray())
        nondisclosed_placement_ids = @($nondisclosed.ToArray() | Sort-Object)
    }
    ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $snapshot
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

function ConvertTo-OutageFindingIdentities {
    param([Parameter(Mandatory)]$Health)

    @($Health.findings | Sort-Object { [string]$_.placement_id } | ForEach-Object {
        [ordered]@{
            placement_id = [string]$_.placement_id
            finding_id = [string]$_.id
            finding_revision = [long]$_.finding_revision
        }
    })
}

function Get-OutageCompositionProjection {
    param([Parameter(Mandatory)][string]$Token)

    $composition = Invoke-JsonRequest -Path "/api/admin/dashboards/$($fixture.dashboard_id)/composition" -Token $Token
    $placements = @($composition.dashboard.placements)
    [ordered]@{
        placement_ids = @($placements | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        provider_unavailable_placement_ids = @($placements | Where-Object { [string]$_.resolution_state -ceq "provider_unavailable" } | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        restricted_placement_ids = @($placements | Where-Object { [string]$_.resolution_state -ceq "restricted" } | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        component_metadata_disclosed_placement_ids = @($placements | Where-Object { $null -ne (Get-OptionalPropertyValue -Value $_ -Name "component") } | ForEach-Object { [string]$_.placement_id } | Sort-Object)
    }
}

function ConvertTo-FindingEvidence {
    param([Parameter(Mandatory)]$Finding)

    [ordered]@{
        placement_id = [string]$Finding.placement_id
        finding_id = [string]$Finding.id
        finding_code = [string]$Finding.finding_code
        disposition = [string]$Finding.disposition
        finding_revision = [long]$Finding.finding_revision
        observed_lifecycle = [string]$Finding.observed_lifecycle
        publication_state = [string]$Finding.publication_state
        successor_available = [bool]$Finding.successor_available
        saved_reference = ConvertTo-ComponentReferenceIdentity -Reference $Finding.saved_reference
    }
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

function New-ActionEvidence {
    param(
        [Parameter(Mandatory)][string]$Action,
        [Parameter(Mandatory)]$Finding,
        [Parameter(Mandatory)]$Response,
        [Parameter(Mandatory)]$BeforeReference,
        [AllowNull()]$RequestedReference,
        [AllowNull()]$AfterReference,
        [AllowNull()]$PlacementPresentAfter
    )

    [ordered]@{
        action = $Action
        placement_id = [string]$Response.placement_id
        finding_id = [string]$Response.finding_id
        request_finding_revision = [long]$Finding.finding_revision
        result_finding_revision = [long]$Response.finding_revision
        disposition = [string]$Response.disposition
        before_reference = $BeforeReference
        requested_reference = $RequestedReference
        after_reference = $AfterReference
        placement_present_after = $PlacementPresentAfter
    }
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
    @($fixture.initial_placement_ids | Sort-Object -Unique).Count -ne 7 -or
    @($fixture.outage_placement_ids | Sort-Object -Unique).Count -ne 5 -or
    $fixture.outage_placement_ids -contains $fixture.blocked_placement_id) {
    throw "Sprint 8A Dashboard dependency diagnostic fixture identities are inconsistent."
}
if ($SelfTest) {
    Test-Sprint8ADashboardDependencyEvidenceContract
    if (-not (Get-Command Initialize-ComponentAccessBasis).Definition.Contains('"/api/components"')) {
        throw "Sprint 8A Dashboard dependency diagnostic no longer primes the provider-evaluated access basis."
    }
    $retryState = [pscustomobject]@{ attempt = 0 }
    $retryOperation = {
        $retryState.attempt++
        [pscustomobject]@{ ready = $retryState.attempt -ge 2 }
    }
    $retryAssertion = {
        param($value)
        if ($value.ready -ne $true) { throw "projection pending" }
    }
    $retryResult = Invoke-DashboardProjectionConvergence `
        -Operation $retryOperation `
        -Assertion $retryAssertion `
        -Label "self-test projection" `
        -MaximumAttempts 3 `
        -DelaySeconds 0
    if ($retryState.attempt -ne 2 -or $retryResult.ready -ne $true) {
        throw "Sprint 8A Dashboard dependency diagnostic did not retry a pending projection deterministically."
    }
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
$snapshots = [Collections.Generic.List[object]]::new()
$fatalErrors = [Collections.Generic.List[string]]::new()
$bootstrapToken = $null
$token = $null
$isolationToken = $null
$isolationActor = $null
$componentStopped = $false
$initial = $null
$initialFindingEvidence = @()
$initialBlockedDisclosed = $true
$initialHealthEvidence = [ordered]@{ health = "not_proven"; open_count = -1; deferred_count = -1 }
$referenceCatalog = [ordered]@{ declared_successor = $null; replacement = $null }
$outageEvidence = [ordered]@{
    health = "not_proven"
    open_count = -1
    deferred_count = -1
    finding_placements = @()
    finding_codes = @()
    finding_identities = @()
    blocked_placement_disclosed = $true
    composition = [ordered]@{ placement_ids = @(); provider_unavailable_placement_ids = @(); restricted_placement_ids = @(); component_metadata_disclosed_placement_ids = @() }
    unrelated_route = [ordered]@{ path = $fixture.unrelated_route_path; status_code = -1; schema_version = -1; state = "not_proven" }
}
$contextIsolationEvidence = [ordered]@{
    actor_id = "00000000-0000-0000-0000-000000000000"
    health = "not_proven"
    open_count = -1
    deferred_count = -1
    finding_placements = @()
    finding_identities = @()
    blocked_placement_disclosed = $true
    distinct_finding_ids = $false
}
$repeatOutageEvidence = [ordered]@{
    finding_placements = @()
    reopened_findings = @()
    stable_open_findings = @()
    reopened_revision_advanced = $false
    open_refresh_idempotent = $false
}
$recovered = $null
$isolatedRecovered = $null
$recoveredSnapshot = $null
Push-Location $repoRoot
try {
    $bootstrapToken = Get-Token
    $isolationActor = Initialize-ContextIsolationActor -AdminToken $bootstrapToken
    $contextIsolationEvidence.actor_id = [string]$isolationActor.actor_id
    $refreshPath = "/api/admin/dashboards/$($fixture.dashboard_id)/dependencies/refresh"
    $readPath = "/api/admin/dashboards/$($fixture.dashboard_id)/dependencies"

    try {
        $initialSnapshotOperation = {
            $candidateToken = Get-Token
            try {
                $null = @(Initialize-ComponentAccessBasis -Token $candidateToken)
                [pscustomobject]@{
                    token = $candidateToken
                    snapshot = Get-DashboardCompositionSnapshot -Token $candidateToken -Stage "initial"
                }
            } catch {
                try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $candidateToken | Out-Null } catch { }
                throw
            }
        }
        $initialSnapshotAssertion = {
            param($candidate)
            try {
                Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $candidate.snapshot -Stage "initial"
            } catch {
                try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $candidate.token | Out-Null } catch { }
                throw
            }
        }
        $initialSnapshotContext = Invoke-DashboardProjectionConvergence `
            -Operation $initialSnapshotOperation `
            -Assertion $initialSnapshotAssertion `
            -Label "Initial Dashboard composition access-basis projection" `
            -MaximumAttempts 60
        $token = [string]$initialSnapshotContext.token
        $initialSnapshot = $initialSnapshotContext.snapshot
        $snapshots.Add($initialSnapshot)
        Assert-Condition -Condition $true -Code "exact_initial_composition" -Detail "Dashboard composition starts with the exact seven receipt-bound placement identities and typed references" -Checks $checks
    } catch {
        Add-FailedCheck -Code "exact_initial_composition" -Detail $_.Exception.Message -Checks $checks
    }

    try {
        $snapshotToken = $token
        $initialRefreshOperation = {
            $candidateToken = Get-Token
            try {
                $null = @(Initialize-ComponentAccessBasis -Token $candidateToken)
                [pscustomobject]@{
                    token = $candidateToken
                    result = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $candidateToken -Body @{}
                }
            } catch {
                try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $candidateToken | Out-Null } catch { }
                throw
            }
        }
        $initialRefreshAssertion = {
            param($candidate)
            try {
                $placements = @($candidate.result.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object)
                if ([string]$candidate.result.health -cne "degraded" -or
                    [long]$candidate.result.open_count -ne 3 -or
                    [long]$candidate.result.deferred_count -ne 0 -or
                    ($placements -join ',') -cne ((@($expectedActionPlacements) | Sort-Object) -join ',')) {
                    throw "the exact three lifecycle findings are not visible yet"
                }
            } catch {
                try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $candidate.token | Out-Null } catch { }
                throw
            }
        }
        $initialRefreshContext = Invoke-DashboardProjectionConvergence `
            -Operation $initialRefreshOperation `
            -Assertion $initialRefreshAssertion `
            -Label "Initial Dashboard dependency access-basis projection" `
            -MaximumAttempts 60
        $token = [string]$initialRefreshContext.token
        $initial = $initialRefreshContext.result
        if (-not [string]::IsNullOrWhiteSpace($snapshotToken) -and $snapshotToken -cne $token) {
            try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $snapshotToken | Out-Null } catch { }
        }
        $initialHealthEvidence = [ordered]@{
            health = [string]$initial.health
            open_count = [long]$initial.open_count
            deferred_count = [long]$initial.deferred_count
        }
        $visibleInitialPlacements = @($initial.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object)
        Assert-Condition `
            -Condition (($visibleInitialPlacements -join ',') -ceq ((@($expectedActionPlacements) | Sort-Object) -join ',')) `
            -Code "exact_lifecycle_finding_placements" `
            -Detail "Only the three authorized action-fixture placements expose findings" `
            -Checks $checks
    } catch {
        Add-FailedCheck -Code "exact_lifecycle_finding_placements" -Detail $_.Exception.Message -Checks $checks
    }
    $isolationToken = Get-Token -Email $fixture.isolation_actor_email -Password $isolationActor.password
    $null = @(Initialize-ComponentAccessBasis -Token $isolationToken)
    if ($null -eq $initial) {
        Add-BlockedCheck -Code "inactive_successor_findings" -DependencyReason "initial dependency refresh did not complete" -Checks $checks
        Add-BlockedCheck -Code "blocked_scope_nondisclosure" -DependencyReason "initial dependency refresh did not complete" -Checks $checks
    } else {
        try {
            $initialFindingEvidence = @($initial.findings | Sort-Object { [string]$_.placement_id } | ForEach-Object { ConvertTo-FindingEvidence -Finding $_ })
            if ($initialFindingEvidence.Count -ne 3) {
                throw "The exact three authorized lifecycle findings were not retained."
            }
            foreach ($finding in $initialFindingEvidence) {
                if ([string]$finding.finding_code -cne "lifecycle_unrenderable" -or
                    [string]$finding.disposition -cne "open" -or
                    [string]$finding.observed_lifecycle -cne "inactive" -or
                    [string]$finding.publication_state -cne "superseded" -or
                    $finding.successor_available -ne $true) {
                    throw "Placement '$($finding.placement_id)' did not expose the exact inactive successor finding."
                }
                Assert-Sprint8AComponentReferenceIdentity -Reference $finding.saved_reference -ExpectedResourceId $script:Sprint8AFixture.inactive_stat_card_version_id -Label "Initial finding '$($finding.placement_id)' saved reference"
            }
            Assert-Condition -Condition $true -Code "inactive_successor_findings" -Detail "All three action fixtures expose exact lifecycle_unrenderable inactive-predecessor findings" -Checks $checks
        } catch {
            Add-FailedCheck -Code "inactive_successor_findings" -Detail $_.Exception.Message -Checks $checks
        }
        try {
            $initialBlockedDisclosed = @($initial.findings | Where-Object { [string]$_.placement_id -ceq $fixture.blocked_placement_id }).Count -ne 0
            Assert-Condition `
                -Condition (-not $initialBlockedDisclosed -and [long]$initial.open_count -eq 3 -and [long]$initial.deferred_count -eq 0) `
                -Code "blocked_scope_nondisclosure" `
                -Detail "The blocked placement affects neither visible findings nor visible aggregate counts" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "blocked_scope_nondisclosure" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $upgradeFinding = $null
    if ($null -ne $initial) {
        try { $upgradeFinding = Get-FindingByPlacement -Health $initial -PlacementId $fixture.upgrade_placement_id } catch { }
    }
    $componentCatalogError = $null
    try {
        $components = @(Invoke-JsonRequest -Path "/api/components" -Token $token)
        $successorComponent = @($components | Where-Object { [string]$_.slug -ceq "sprint-8a-row-count" })
        $replacementComponent = @($components | Where-Object { [string]$_.slug -ceq "sprint-8a-record-table" })
        if ($successorComponent.Count -ne 1 -or $null -eq $successorComponent[0].current_version.reference) {
            throw "The exact declared row-count successor reference was not available."
        }
        if ($replacementComponent.Count -ne 1 -or $null -eq $replacementComponent[0].current_version.reference) {
            throw "The exact authorized table replacement reference was not available."
        }
        $referenceCatalog.declared_successor = ConvertTo-ComponentReferenceIdentity -Reference $successorComponent[0].current_version.reference
        $referenceCatalog.replacement = ConvertTo-ComponentReferenceIdentity -Reference $replacementComponent[0].current_version.reference
    } catch {
        $componentCatalogError = $_.Exception.Message
        $successorComponent = @()
        $replacementComponent = @()
    }
    if ($null -eq $upgradeFinding) {
        Add-BlockedCheck -Code "successor_disclosed_for_authorized_finding" -DependencyReason "the exact Upgrade fixture finding was not available" -Checks $checks
    } elseif ($null -ne $componentCatalogError) {
        Add-FailedCheck -Code "successor_disclosed_for_authorized_finding" -Detail $componentCatalogError -Checks $checks
    } else {
        try {
            Assert-Sprint8AComponentReferenceIdentity -Reference $referenceCatalog.declared_successor -ExpectedResourceId $script:Sprint8AFixture.component_versions.stat_card -Label "Declared Upgrade successor"
            Assert-Condition -Condition ([bool]$upgradeFinding.successor_available) -Code "successor_disclosed_for_authorized_finding" -Detail "The provider declares the exact current stat-card successor for the authorized inactive finding" -Checks $checks
        } catch {
            Add-FailedCheck -Code "successor_disclosed_for_authorized_finding" -Detail $_.Exception.Message -Checks $checks
        }
    }
    if ($null -ne $componentCatalogError) {
        Add-FailedCheck -Code "replacement_reference_available" -Detail $componentCatalogError -Checks $checks
    } else {
        try {
            Assert-Sprint8AComponentReferenceIdentity -Reference $referenceCatalog.replacement -ExpectedResourceId $script:Sprint8AFixture.component_versions.table -Label "Authorized Replace reference"
            Assert-Condition -Condition $true -Code "replacement_reference_available" -Detail "The exact authorized renderable table reference is available for Replace" -Checks $checks
        } catch {
            Add-FailedCheck -Code "replacement_reference_available" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $deferred = $null
    if ($null -eq $upgradeFinding) {
        Add-BlockedCheck -Code "defer_advances_finding" -DependencyReason "the Upgrade fixture finding was not available" -Checks $checks
        Add-BlockedCheck -Code "defer_preserves_composition" -DependencyReason "Defer could not execute without its exact finding" -Checks $checks
    } else {
        try {
            $beforeReference = ConvertTo-ComponentReferenceIdentity -Reference $upgradeFinding.saved_reference
            $deferred = Invoke-DependencyAction -Token $token -Finding $upgradeFinding -Action "defer" -Replacement $null
            $deferActionEvidence = New-ActionEvidence -Action "defer" -Finding $upgradeFinding -Response $deferred -BeforeReference $beforeReference -RequestedReference $null -AfterReference $null -PlacementPresentAfter $null
            $actions.Add($deferActionEvidence)
            Assert-Condition `
                -Condition ([string]$deferred.action -ceq "defer" -and [string]$deferred.placement_id -ceq $fixture.upgrade_placement_id -and [string]$deferred.disposition -ceq "deferred" -and [long]$deferred.finding_revision -eq ([long]$upgradeFinding.finding_revision + 1)) `
                -Code "defer_advances_finding" `
                -Detail "Defer retains the exact Upgrade finding and advances its revision" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "defer_advances_finding" -Detail $_.Exception.Message -Checks $checks
        }
        if ($null -eq $deferred) {
            Add-BlockedCheck -Code "defer_preserves_composition" -DependencyReason "Defer did not return a successful action response" -Checks $checks
        } else {
            try {
                $afterDeferSnapshot = Get-DashboardCompositionSnapshot -Token $token -Stage "after_defer"
                $snapshots.Add($afterDeferSnapshot)
                $deferActionEvidence["after_reference"] = Get-Sprint8ADashboardSnapshotReference -Snapshot $afterDeferSnapshot -PlacementId $fixture.upgrade_placement_id
                $deferActionEvidence["placement_present_after"] = $true
                Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $afterDeferSnapshot -Stage "after_defer"
                Assert-Condition -Condition $true -Code "defer_preserves_composition" -Detail "Composition readback proves Defer changed no placement identity or reference" -Checks $checks
            } catch {
                Add-FailedCheck -Code "defer_preserves_composition" -Detail $_.Exception.Message -Checks $checks
            }
        }
    }

    $deferredFinding = $null
    if ($null -eq $deferred) {
        Add-BlockedCheck -Code "deferred_finding_remains_actionable" -DependencyReason "Defer did not produce a current finding revision" -Checks $checks
    } else {
        try {
            $afterDefer = Invoke-JsonRequest -Path $readPath -Token $token
            $deferredFinding = Get-FindingByPlacement -Health $afterDefer -PlacementId $fixture.upgrade_placement_id
            Assert-Condition `
                -Condition ([string]$deferredFinding.disposition -ceq "deferred" -and [long]$deferredFinding.finding_revision -eq [long]$deferred.finding_revision) `
                -Code "deferred_finding_remains_actionable" `
                -Detail "The deferred finding remains actionable at the exact returned revision" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "deferred_finding_remains_actionable" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $upgraded = $null
    if ($null -eq $deferredFinding) {
        Add-BlockedCheck -Code "upgrade_uses_declared_successor" -DependencyReason "the deferred finding is unavailable at a verified revision" -Checks $checks
    } else {
        try {
            $beforeReference = ConvertTo-ComponentReferenceIdentity -Reference $deferredFinding.saved_reference
            $upgraded = Invoke-DependencyAction -Token $token -Finding $deferredFinding -Action "upgrade" -Replacement $null
            $upgradeActionEvidence = New-ActionEvidence -Action "upgrade" -Finding $deferredFinding -Response $upgraded -BeforeReference $beforeReference -RequestedReference $null -AfterReference $null -PlacementPresentAfter $null
            $actions.Add($upgradeActionEvidence)
            $afterUpgradeSnapshot = Get-DashboardCompositionSnapshot -Token $token -Stage "after_upgrade"
            $snapshots.Add($afterUpgradeSnapshot)
            $afterReference = Get-Sprint8ADashboardSnapshotReference -Snapshot $afterUpgradeSnapshot -PlacementId $fixture.upgrade_placement_id
            $upgradeActionEvidence["after_reference"] = $afterReference
            $upgradeActionEvidence["placement_present_after"] = $true
            Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $afterUpgradeSnapshot -Stage "after_upgrade"
            Assert-Sprint8AComponentReferenceIdentity -Reference $afterReference -ExpectedResourceId $referenceCatalog.declared_successor.resource_id -Label "Upgrade composition readback"
            Assert-Condition `
                -Condition ([string]$upgraded.action -ceq "upgrade" -and [string]$upgraded.placement_id -ceq $fixture.upgrade_placement_id -and [string]$upgraded.disposition -ceq "resolved" -and [long]$upgraded.finding_revision -eq ([long]$deferredFinding.finding_revision + 1)) `
                -Code "upgrade_uses_declared_successor" `
                -Detail "Composition readback proves Upgrade selected the exact provider-declared successor and changed no sibling fixture" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "upgrade_uses_declared_successor" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $replaceFinding = $null
    if ($null -ne $initial) {
        try { $replaceFinding = Get-FindingByPlacement -Health $initial -PlacementId $fixture.replace_placement_id } catch { }
    }
    $replaced = $null
    if ($null -eq $replaceFinding) {
        Add-BlockedCheck -Code "replace_uses_authorized_renderable_reference" -DependencyReason "the exact Replace fixture finding was not available" -Checks $checks
    } elseif ($null -eq $referenceCatalog.replacement -or $replacementComponent.Count -ne 1) {
        Add-BlockedCheck -Code "replace_uses_authorized_renderable_reference" -DependencyReason "the exact authorized replacement reference was not available" -Checks $checks
    } else {
        try {
            $beforeReference = ConvertTo-ComponentReferenceIdentity -Reference $replaceFinding.saved_reference
            $requestedReference = ConvertTo-ComponentReferenceIdentity -Reference $replacementComponent[0].current_version.reference
            $replaced = Invoke-DependencyAction -Token $token -Finding $replaceFinding -Action "replace" -Replacement $replacementComponent[0].current_version.reference
            $replaceActionEvidence = New-ActionEvidence -Action "replace" -Finding $replaceFinding -Response $replaced -BeforeReference $beforeReference -RequestedReference $requestedReference -AfterReference $null -PlacementPresentAfter $null
            $actions.Add($replaceActionEvidence)
            $afterReplaceSnapshot = Get-DashboardCompositionSnapshot -Token $token -Stage "after_replace"
            $snapshots.Add($afterReplaceSnapshot)
            $afterReference = Get-Sprint8ADashboardSnapshotReference -Snapshot $afterReplaceSnapshot -PlacementId $fixture.replace_placement_id
            $replaceActionEvidence["after_reference"] = $afterReference
            $replaceActionEvidence["placement_present_after"] = $true
            Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $afterReplaceSnapshot -Stage "after_replace"
            Assert-Sprint8AComponentReferenceIdentity -Reference $afterReference -ExpectedResourceId $referenceCatalog.replacement.resource_id -Label "Replace composition readback"
            Assert-Condition `
                -Condition ([string]$replaced.action -ceq "replace" -and [string]$replaced.placement_id -ceq $fixture.replace_placement_id -and [string]$replaced.disposition -ceq "resolved" -and [long]$replaced.finding_revision -eq ([long]$replaceFinding.finding_revision + 1)) `
                -Code "replace_uses_authorized_renderable_reference" `
                -Detail "Composition readback proves Replace selected the exact authorized table reference and changed no sibling fixture" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "replace_uses_authorized_renderable_reference" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $removeFinding = $null
    if ($null -ne $initial) {
        try { $removeFinding = Get-FindingByPlacement -Health $initial -PlacementId $fixture.remove_placement_id } catch { }
    }
    $removed = $null
    if ($null -eq $removeFinding) {
        Add-BlockedCheck -Code "remove_consumes_independent_placement" -DependencyReason "the exact Remove fixture finding was not available" -Checks $checks
    } else {
        try {
            $beforeReference = ConvertTo-ComponentReferenceIdentity -Reference $removeFinding.saved_reference
            $removed = Invoke-DependencyAction -Token $token -Finding $removeFinding -Action "remove" -Replacement $null
            $removeActionEvidence = New-ActionEvidence -Action "remove" -Finding $removeFinding -Response $removed -BeforeReference $beforeReference -RequestedReference $null -AfterReference $null -PlacementPresentAfter $null
            $actions.Add($removeActionEvidence)
            $afterRemoveSnapshot = Get-DashboardCompositionSnapshot -Token $token -Stage "after_remove"
            $snapshots.Add($afterRemoveSnapshot)
            $removedStillPresent = @($afterRemoveSnapshot.placement_ids | Where-Object { [string]$_ -ceq $fixture.remove_placement_id }).Count -ne 0
            $removeActionEvidence["placement_present_after"] = $removedStillPresent
            Assert-Condition `
                -Condition ([string]$removed.action -ceq "remove" -and [string]$removed.placement_id -ceq $fixture.remove_placement_id -and [string]$removed.disposition -ceq "resolved" -and [long]$removed.finding_revision -eq ([long]$removeFinding.finding_revision + 1) -and -not $removedStillPresent) `
                -Code "remove_consumes_independent_placement" `
                -Detail "Composition readback proves Remove deleted the exact independent placement" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "remove_consumes_independent_placement" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $preOutageStages = @("initial", "after_defer", "after_upgrade", "after_replace", "after_remove")
    $missingPreOutageStages = @(
        foreach ($stage in $preOutageStages) {
            if (@($snapshots | Where-Object { [string]$_.stage -ceq $stage }).Count -ne 1) { $stage }
        }
    )
    if ($missingPreOutageStages.Count -ne 0) {
        Add-BlockedCheck -Code "action_fixtures_remain_independent" -DependencyReason "composition readback is incomplete for stage(s): $($missingPreOutageStages -join ', ')" -Checks $checks
    } else {
        try {
            foreach ($stage in $preOutageStages) {
                $snapshot = @($snapshots | Where-Object { [string]$_.stage -ceq $stage })[0]
                Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $snapshot -Stage $stage
            }
            Assert-Condition -Condition $true -Code "action_fixtures_remain_independent" -Detail "Exact stage-by-stage composition identities prove Defer, Upgrade, Replace, and Remove affected only their declared placements" -Checks $checks
        } catch {
            Add-FailedCheck -Code "action_fixtures_remain_independent" -Detail $_.Exception.Message -Checks $checks
        }
    }

    $preOutagePrimeError = $null
    if ($missingPreOutageStages.Count -ne 0) {
        Add-BlockedCheck `
            -Code "provider_outage_is_contained" `
            -DependencyReason "the action-fixture composition stages did not complete" `
            -Checks $checks
    } else {
        try {
            $primeOperation = {
                [pscustomobject]@{
                    primary = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
                    isolated = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $isolationToken -Body @{}
                }
            }
            $primeAssertion = {
                param($candidate)
                if ([string]$candidate.primary.health -cne "healthy" -or @($candidate.primary.findings).Count -ne 0 -or
                    [string]$candidate.isolated.health -cne "healthy" -or @($candidate.isolated.findings).Count -ne 0) {
                    throw "both authorization contexts are not healthy at zero findings yet"
                }
            }
            $primeState = Invoke-DashboardProjectionConvergence `
                -Operation $primeOperation `
                -Assertion $primeAssertion `
                -Label "Pre-outage access-basis refresh"
            $primaryPrime = $primeState.primary
            $isolatedPrime = $primeState.isolated
        } catch {
            $preOutagePrimeError = $_.Exception.Message
        }
        if ($null -ne $preOutagePrimeError) {
            Add-FailedCheck -Code "provider_outage_is_contained" -Detail $preOutagePrimeError -Checks $checks
        } else {
            try {
                & docker compose -f $composePath --profile reference stop components | Out-Null
                if ($LASTEXITCODE -ne 0) { throw "Could not stop the Component provider for the outage diagnostic." }
                $componentStopped = $true
            } catch {
                Add-FailedCheck -Code "provider_outage_is_contained" -Detail $_.Exception.Message -Checks $checks
            }
        }
    }
    if (-not $componentStopped) {
        Add-BlockedCheck -Code "outage_blocked_scope_nondisclosure" -DependencyReason "the Component provider outage was not established safely" -Checks $checks
        Add-BlockedCheck -Code "outage_composition_projection_is_safe" -DependencyReason "the Component provider outage was not established safely" -Checks $checks
        Add-BlockedCheck -Code "authorization_contexts_are_isolated" -DependencyReason "the Component provider outage was not established safely" -Checks $checks
        Add-BlockedCheck -Code "unrelated_route_remains_healthy" -DependencyReason "the Component provider outage was not established safely" -Checks $checks
    } else {
        $outage = $null
        try {
            $outage = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
            $outageEvidence.health = [string]$outage.health
            $outageEvidence.open_count = [long]$outage.open_count
            $outageEvidence.deferred_count = [long]$outage.deferred_count
            $visibleOutageFindings = @($outage.findings | Sort-Object { [string]$_.placement_id })
            $outageEvidence.finding_placements = @($visibleOutageFindings | ForEach-Object { [string]$_.placement_id })
            $outageEvidence.finding_codes = @($visibleOutageFindings | ForEach-Object { [string]$_.finding_code })
            $outageEvidence.finding_identities = ConvertTo-OutageFindingIdentities -Health $outage
            $outageEvidence.blocked_placement_disclosed = @($visibleOutageFindings | Where-Object { [string]$_.placement_id -ceq $fixture.blocked_placement_id }).Count -ne 0
            Assert-Condition `
                -Condition (($outageEvidence.finding_placements -join ',') -ceq ((@($fixture.outage_placement_ids) | Sort-Object) -join ',') -and @($outageEvidence.finding_codes | Where-Object { $_ -cne "provider_unavailable" }).Count -eq 0) `
                -Code "provider_outage_is_contained" `
                -Detail "Exactly the five remaining authorized placements report provider_unavailable" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "provider_outage_is_contained" -Detail $_.Exception.Message -Checks $checks
        }
        if ($null -eq $outage) {
            Add-BlockedCheck -Code "outage_blocked_scope_nondisclosure" -DependencyReason "the exact provider-outage finding response was unavailable" -Checks $checks
        } else {
            try {
            Assert-Condition `
                -Condition (-not $outageEvidence.blocked_placement_disclosed -and [long]$outage.open_count -eq 5 -and [long]$outage.deferred_count -eq 0) `
                -Code "outage_blocked_scope_nondisclosure" `
                -Detail "The blocked placement affects neither outage findings nor visible aggregate counts" `
                -Checks $checks
            } catch {
                Add-FailedCheck -Code "outage_blocked_scope_nondisclosure" -Detail $_.Exception.Message -Checks $checks
            }
        }
        try {
            $outageEvidence.composition = Get-OutageCompositionProjection -Token $token
            Assert-Sprint8AOutageCompositionProjection -Composition ([pscustomobject]$outageEvidence.composition)
            Assert-Condition `
                -Condition $true `
                -Code "outage_composition_projection_is_safe" `
                -Detail "Editor composition uses the same exact access basis and discloses no provider metadata during outage" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "outage_composition_projection_is_safe" -Detail $_.Exception.Message -Checks $checks
        }
        if ($null -eq $outage) {
            Add-BlockedCheck -Code "authorization_contexts_are_isolated" -DependencyReason "the primary-context outage findings were unavailable for identity comparison" -Checks $checks
        } else {
            try {
                $isolatedOutage = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $isolationToken -Body @{}
                $isolatedIdentities = ConvertTo-OutageFindingIdentities -Health $isolatedOutage
                $primaryIds = @($outageEvidence.finding_identities | ForEach-Object { [string]$_.finding_id })
                $isolatedIds = @($isolatedIdentities | ForEach-Object { [string]$_.finding_id })
                $distinctFindingIds = @($primaryIds | Where-Object { $isolatedIds -ccontains $_ }).Count -eq 0
                $contextIsolationEvidence.health = [string]$isolatedOutage.health
                $contextIsolationEvidence.open_count = [long]$isolatedOutage.open_count
                $contextIsolationEvidence.deferred_count = [long]$isolatedOutage.deferred_count
                $contextIsolationEvidence.finding_placements = @($isolatedOutage.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object)
                $contextIsolationEvidence.finding_identities = $isolatedIdentities
                $contextIsolationEvidence.blocked_placement_disclosed = @($isolatedOutage.findings | Where-Object { [string]$_.placement_id -ceq $fixture.blocked_placement_id }).Count -ne 0
                $contextIsolationEvidence.distinct_finding_ids = $distinctFindingIds
                $isolatedCodesValid = @($isolatedOutage.findings | Where-Object { [string]$_.finding_code -cne "provider_unavailable" }).Count -eq 0
                $isolatedPlacementsValid = (($contextIsolationEvidence.finding_placements -join ',') -ceq ((@($fixture.outage_placement_ids) | Sort-Object) -join ','))
                Assert-Condition `
                    -Condition ([string]$isolatedOutage.health -ceq "degraded" -and [long]$isolatedOutage.open_count -eq 5 -and [long]$isolatedOutage.deferred_count -eq 0 -and $isolatedPlacementsValid -and $isolatedCodesValid -and -not $contextIsolationEvidence.blocked_placement_disclosed -and $distinctFindingIds) `
                    -Code "authorization_contexts_are_isolated" `
                    -Detail "A second actor with equivalent scope receives distinct context-owned finding identities and no blocked placement" `
                    -Checks $checks
            } catch {
                Add-FailedCheck -Code "authorization_contexts_are_isolated" -Detail $_.Exception.Message -Checks $checks
            }
        }
        try {
            $navigation = Invoke-JsonRequest -Path $fixture.unrelated_route_path -Token $token -IncludeStatus
            $outageEvidence.unrelated_route = [ordered]@{
                path = $fixture.unrelated_route_path
                status_code = [int]$navigation.status_code
                schema_version = [int]$navigation.body.schema_version
                state = [string]$navigation.body.state
            }
            Assert-Condition `
                -Condition ([int]$navigation.status_code -eq 200 -and [int]$navigation.body.schema_version -eq 3 -and [string]$navigation.body.state -ceq "available") `
                -Code "unrelated_route_remains_healthy" `
                -Detail "Core-owned shell navigation remains available while Components is stopped" `
                -Checks $checks
        } catch {
            Add-FailedCheck -Code "unrelated_route_remains_healthy" -Detail $_.Exception.Message -Checks $checks
        }
    }

    if (-not $componentStopped) {
        Add-BlockedCheck -Code "identical_outage_reopens" -DependencyReason "the first provider outage was not established safely" -Checks $checks
        Add-BlockedCheck -Code "open_outage_refresh_is_idempotent" -DependencyReason "the first provider outage was not established safely" -Checks $checks
        Add-BlockedCheck -Code "provider_recovery_converges" -DependencyReason "the provider outage was not established safely" -Checks $checks
    } else {
        $recoveryErrors = [Collections.Generic.List[string]]::new()
        try {
            & docker compose -f $composePath --profile reference start components | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "Could not restart the Component provider after the outage diagnostic." }
            Wait-ComponentHealthy
            $componentStopped = $false
            $firstRecoveryOperation = {
                $null = @(Initialize-ComponentAccessBasis -Token $token)
                $null = @(Initialize-ComponentAccessBasis -Token $isolationToken)
                [pscustomobject]@{
                    primary = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
                    isolated = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $isolationToken -Body @{}
                }
            }
            $exactRecoveryAssertion = {
                param($candidate)
                if ([string]$candidate.primary.health -cne "healthy" -or
                    [long]$candidate.primary.open_count -ne 0 -or
                    [long]$candidate.primary.deferred_count -ne 0 -or
                    @($candidate.primary.findings).Count -ne 0 -or
                    [string]$candidate.isolated.health -cne "healthy" -or
                    [long]$candidate.isolated.open_count -ne 0 -or
                    [long]$candidate.isolated.deferred_count -ne 0 -or
                    @($candidate.isolated.findings).Count -ne 0) {
                    throw "both authorization contexts are not at exact zero-finding health yet"
                }
            }
            $firstRecovery = Invoke-DashboardProjectionConvergence `
                -Operation $firstRecoveryOperation `
                -Assertion $exactRecoveryAssertion `
                -Label "First provider recovery" `
                -MaximumAttempts 60
            $recovered = $firstRecovery.primary
            $isolatedRecovered = $firstRecovery.isolated
        } catch {
            $recoveryErrors.Add($_.Exception.Message)
        }

        $repeat = $null
        $repeatStable = $null
        if ($recoveryErrors.Count -ne 0) {
            Add-BlockedCheck -Code "identical_outage_reopens" -DependencyReason "the first outage did not recover cleanly" -Checks $checks
            Add-BlockedCheck -Code "open_outage_refresh_is_idempotent" -DependencyReason "the identical outage could not be re-established" -Checks $checks
        } else {
            try {
                & docker compose -f $composePath --profile reference stop components | Out-Null
                if ($LASTEXITCODE -ne 0) { throw "Could not stop Components for the repeated outage." }
                $componentStopped = $true
                $repeat = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
                $repeatOutageEvidence.finding_placements = @($repeat.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object)
                $repeatOutageEvidence.reopened_findings = ConvertTo-OutageFindingIdentities -Health $repeat
                $advanced = @($outageEvidence.finding_identities).Count -eq 5 -and @($repeatOutageEvidence.reopened_findings).Count -eq 5
                foreach ($prior in @($outageEvidence.finding_identities)) {
                    $current = @($repeatOutageEvidence.reopened_findings | Where-Object { [string]$_.placement_id -ceq [string]$prior.placement_id })
                    if ($current.Count -ne 1 -or [string]$current[0].finding_id -cne [string]$prior.finding_id -or [long]$current[0].finding_revision -le [long]$prior.finding_revision) { $advanced = $false }
                }
                $repeatOutageEvidence.reopened_revision_advanced = $advanced
                Assert-Condition -Condition $advanced -Code "identical_outage_reopens" -Detail "The identical resolved outage reopens each exact finding identity with an advanced revision" -Checks $checks
            } catch {
                Add-FailedCheck -Code "identical_outage_reopens" -Detail $_.Exception.Message -Checks $checks
            }
            if ($null -eq $repeat) {
                Add-BlockedCheck -Code "open_outage_refresh_is_idempotent" -DependencyReason "the repeated outage refresh did not return findings" -Checks $checks
            } else {
                try {
                    $repeatStable = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
                    $repeatOutageEvidence.stable_open_findings = ConvertTo-OutageFindingIdentities -Health $repeatStable
                    $stable = (($repeatOutageEvidence.reopened_findings | ConvertTo-Json -Depth 5 -Compress) -ceq ($repeatOutageEvidence.stable_open_findings | ConvertTo-Json -Depth 5 -Compress))
                    $repeatOutageEvidence.open_refresh_idempotent = $stable
                    Assert-Condition -Condition $stable -Code "open_outage_refresh_is_idempotent" -Detail "Refreshing an unchanged open outage preserves exact finding identities and revisions" -Checks $checks
                } catch {
                    Add-FailedCheck -Code "open_outage_refresh_is_idempotent" -Detail $_.Exception.Message -Checks $checks
                }
            }
        }

        try {
            if ($componentStopped) {
                & docker compose -f $composePath --profile reference start components | Out-Null
                if ($LASTEXITCODE -ne 0) { throw "Could not restart Components after the repeated outage." }
                Wait-ComponentHealthy
                $componentStopped = $false
            }
            $finalRecoveryOperation = {
                $null = @(Initialize-ComponentAccessBasis -Token $token)
                $null = @(Initialize-ComponentAccessBasis -Token $isolationToken)
                [pscustomobject]@{
                    primary = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $token -Body @{}
                    isolated = Invoke-JsonRequest -Path $refreshPath -Method POST -Token $isolationToken -Body @{}
                    snapshot = Get-DashboardCompositionSnapshot -Token $token -Stage "recovered"
                }
            }
            $finalRecoveryAssertion = {
                param($candidate)
                & $exactRecoveryAssertion $candidate
                Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $candidate.snapshot -Stage "recovered"
            }
            $finalRecovery = Invoke-DashboardProjectionConvergence `
                -Operation $finalRecoveryOperation `
                -Assertion $finalRecoveryAssertion `
                -Label "Final provider recovery" `
                -MaximumAttempts 60
            $recovered = $finalRecovery.primary
            $isolatedRecovered = $finalRecovery.isolated
            $recoveredSnapshot = $finalRecovery.snapshot
            $snapshots.Add($recoveredSnapshot)
        } catch {
            $recoveryErrors.Add($_.Exception.Message)
        }
        if ($recoveryErrors.Count -ne 0 -or $null -eq $recovered -or $null -eq $isolatedRecovered -or $null -eq $recoveredSnapshot) {
            Add-FailedCheck -Code "provider_recovery_converges" -Detail ($recoveryErrors -join "; ") -Checks $checks
        } else {
            Assert-Condition `
                -Condition ([string]$recovered.health -ceq "healthy" -and @($recovered.findings).Count -eq 0 -and [string]$isolatedRecovered.health -ceq "healthy" -and @($isolatedRecovered.findings).Count -eq 0) `
                -Code "provider_recovery_converges" `
                -Detail "Final recovery independently resolves both context-owned finding sets and preserves action identities" `
                -Checks $checks
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
    if (-not [string]::IsNullOrWhiteSpace($isolationToken)) {
        try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $isolationToken | Out-Null } catch { }
    }
    if (-not [string]::IsNullOrWhiteSpace($bootstrapToken) -and $bootstrapToken -cne $token) {
        try { Invoke-JsonRequest -Path "/api/auth/logout" -Method DELETE -Token $bootstrapToken | Out-Null } catch { }
    }
    Pop-Location
}

$checkRows = @($checks.ToArray())
$recordedCodes = @($checkRows | ForEach-Object { [string]$_.code })
foreach ($missingCode in @($script:Sprint8ADashboardDependencyCheckCodes | Where-Object { $recordedCodes -cnotcontains $_ })) {
    $reason = if ($fatalErrors.Count -gt 0) {
        "unhandled prerequisite failure: $($fatalErrors -join '; ')"
    } else {
        "a failed prerequisite prevented safe dependent execution"
    }
    Add-BlockedCheck -Code $missingCode -DependencyReason $reason -Checks $checks
}
$checkRows = @($checks.ToArray())
$duplicateCodes = @($recordedCodes | Group-Object | Where-Object Count -ne 1 | ForEach-Object { [string]$_.Name })
$failedChecks = @($checkRows | Where-Object { [string]$_.state -ceq "failed" })
$blockedChecks = @($checkRows | Where-Object { [string]$_.state -ceq "blocked" })
$passed = $fatalErrors.Count -eq 0 -and $duplicateCodes.Count -eq 0 -and
    $failedChecks.Count -eq 0 -and $blockedChecks.Count -eq 0 -and
    $checks.Count -eq $script:Sprint8ADashboardDependencyCheckCodes.Count
$evidence = [ordered]@{
    schema_version = 3
    evidence_kind = "tessara.sprint-8a.dashboard-dependency-semantic-diagnostic"
    dashboard_id = $fixture.dashboard_id
    checks = @($checkRows)
    actions = @($actions.ToArray())
    reference_catalog = $referenceCatalog
    initial_findings = $initialFindingEvidence
    initial_blocked_placement_disclosed = $initialBlockedDisclosed
    initial_health = $initialHealthEvidence
    composition_snapshots = @($snapshots.ToArray())
    outage = $outageEvidence
    context_isolation = $contextIsolationEvidence
    repeat_outage = $repeatOutageEvidence
    final_health = [ordered]@{
        health = if ($null -eq $recovered) { "not_proven" } else { [string]$recovered.health }
        open_count = if ($null -eq $recovered) { -1 } else { [long]$recovered.open_count }
        deferred_count = if ($null -eq $recovered) { -1 } else { [long]$recovered.deferred_count }
        visible_finding_placements = if ($null -eq $recovered) { @() } else { @($recovered.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object) }
        isolated_health = if ($null -eq $isolatedRecovered) { "not_proven" } else { [string]$isolatedRecovered.health }
        isolated_open_count = if ($null -eq $isolatedRecovered) { -1 } else { [long]$isolatedRecovered.open_count }
        isolated_deferred_count = if ($null -eq $isolatedRecovered) { -1 } else { [long]$isolatedRecovered.deferred_count }
        isolated_visible_finding_placements = if ($null -eq $isolatedRecovered) { @() } else { @($isolatedRecovered.findings | ForEach-Object { [string]$_.placement_id } | Sort-Object) }
    }
    failure_count = $failedChecks.Count + $fatalErrors.Count
    blocked_count = $blockedChecks.Count
    duplicate_check_codes = $duplicateCodes
    fatal_errors = @($fatalErrors.ToArray())
    harvesting_complete = $true
    canonical_reset_required = $true
    passed = $passed
}
if ($passed) {
    try {
        Assert-Sprint8ADashboardDependencyEvidence -Evidence ([pscustomobject]$evidence)
    } catch {
        $fatalErrors.Add("Evidence contract rejected the harvested diagnostic: $($_.Exception.Message)")
        $passed = $false
        $evidence.fatal_errors = @($fatalErrors.ToArray())
        $evidence.failure_count = $failedChecks.Count + $fatalErrors.Count
        $evidence.passed = $false
    }
}
$null = Publish-DashboardDependencyEvidence -Document $evidence -Path $OutputPath
$evidence | ConvertTo-Json -Depth 50
if (-not $passed) {
    throw "Sprint 8A Dashboard dependency diagnostic harvested $($failedChecks.Count + $fatalErrors.Count) failure(s) and $($blockedChecks.Count) blocked check(s)."
}

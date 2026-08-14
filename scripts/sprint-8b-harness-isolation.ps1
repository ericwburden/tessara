[CmdletBinding()]
param(
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$script:Sprint8BRepositoryRoot = Split-Path -Parent $PSScriptRoot

. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

function Resolve-Sprint8BRepositoryPath {
    param([Parameter(Mandatory)][string]$Path)

    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot $Path))
}

function Assert-Sprint8BComposeProject {
    param([Parameter(Mandatory)][string]$ComposeProject)

    if ($ComposeProject -cnotmatch '^tessara-s8b-[a-z0-9](?:[a-z0-9-]{0,46}[a-z0-9])?$') {
        throw "Compose project '$ComposeProject' is outside the exact Sprint 8B isolated-lane namespace."
    }
    $ComposeProject
}

function Assert-Sprint8BResetAuthorization {
    param(
        [Parameter(Mandatory)][string]$ComposeProject,
        [Parameter(Mandatory)][bool]$Authorized
    )

    Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
    if (-not $Authorized) {
        throw "Reset of disposable project '$ComposeProject' requires -AuthorizeDisposableReset."
    }
}

function Get-Sprint8BProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][string[]]$Names)

    $environment = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Process)
    $snapshot = [ordered]@{}
    foreach ($name in $Names) {
        $snapshot[$name] = [pscustomobject][ordered]@{
            present = $environment.Contains($name)
            value = [Environment]::GetEnvironmentVariable($name, [EnvironmentVariableTarget]::Process)
        }
    }
    $snapshot
}

function Restore-Sprint8BProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)]$Snapshot)

    foreach ($name in @($Snapshot.Keys)) {
        $entry = $Snapshot[$name]
        if ([bool]$entry.present) {
            [Environment]::SetEnvironmentVariable(
                [string]$name,
                [string]$entry.value,
                [EnvironmentVariableTarget]::Process
            )
        } else {
            [Environment]::SetEnvironmentVariable(
                [string]$name,
                $null,
                [EnvironmentVariableTarget]::Process
            )
        }
    }
}

function Get-Sprint8BFreeTcpPort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    try {
        $listener.Start()
        ([Net.IPEndPoint]$listener.LocalEndpoint).Port
    } finally {
        $listener.Stop()
    }
}

function Set-Sprint8BComposeEnvironment {
    param(
        [Parameter(Mandatory)][string]$ComposeProject,
        [Nullable[int]]$GatewayPort,
        [Nullable[int]]$CorePort,
        [Nullable[int]]$SupervisorPort
    )

    Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
    $resolvedGatewayPort = if ($null -eq $GatewayPort) { Get-Sprint8BFreeTcpPort } else { [int]$GatewayPort }
    $resolvedCorePort = if ($null -eq $CorePort) { Get-Sprint8BFreeTcpPort } else { [int]$CorePort }
    $resolvedSupervisorPort = if ($null -eq $SupervisorPort) { Get-Sprint8BFreeTcpPort } else { [int]$SupervisorPort }
    $ports = @($resolvedGatewayPort, $resolvedCorePort, $resolvedSupervisorPort)
    if (@($ports | Sort-Object -Unique).Count -ne 3 -or @($ports | Where-Object { $_ -lt 1024 -or $_ -gt 65535 }).Count -ne 0) {
        throw "Sprint 8B gateway, Core, and Supervisor ports must be three distinct non-privileged TCP ports."
    }

    $env:COMPOSE_PROJECT_NAME = $ComposeProject
    $env:TESSARA_GATEWAY_PORT = [string]$resolvedGatewayPort
    $env:TESSARA_CORE_CONTROL_PORT = [string]$resolvedCorePort
    $env:TESSARA_SUPERVISOR_PORT = [string]$resolvedSupervisorPort

    [pscustomobject][ordered]@{
        compose_project = $ComposeProject
        gateway_port = $resolvedGatewayPort
        core_port = $resolvedCorePort
        supervisor_port = $resolvedSupervisorPort
        gateway_url = "http://127.0.0.1:$resolvedGatewayPort"
        core_url = "http://127.0.0.1:$resolvedCorePort"
        supervisor_url = "http://127.0.0.1:$resolvedSupervisorPort"
    }
}

function Invoke-Sprint8BDockerCompose {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string[]]$Arguments,
        [switch]$AllowFailure
    )

    $output = @(& docker compose -f $ComposePath --profile reference @Arguments 2>&1 |
        ForEach-Object { [string]$_ })
    $exitCode = $LASTEXITCODE
    if (-not $AllowFailure -and $exitCode -ne 0) {
        throw "docker compose $($Arguments -join ' ') exited $exitCode.`n$($output -join "`n")"
    }
    [pscustomobject][ordered]@{
        exit_code = $exitCode
        output = $output
    }
}

function Get-Sprint8BComposeConfiguration {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject
    )

    Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
    if ([string]$env:COMPOSE_PROJECT_NAME -cne $ComposeProject) {
        throw "COMPOSE_PROJECT_NAME must equal '$ComposeProject' before resolving Sprint 8B Compose."
    }
    $result = Invoke-Sprint8BDockerCompose -ComposePath $ComposePath -Arguments @("config", "--format", "json")
    $configuration = ($result.output -join "`n") | ConvertFrom-Json -Depth 100
    if ([string]$configuration.name -cne $ComposeProject) {
        throw "Normalized Compose project '$($configuration.name)' does not equal '$ComposeProject'."
    }
    $configuration
}

function Assert-Sprint8BDatabaseIsolationConfiguration {
    param([Parameter(Mandatory)]$ComposeConfiguration)

    $expected = [ordered]@{
        core = "tessara_core"
        datasets = "tessara_module_datasets"
        components = "tessara_module_components"
        dashboards = "tessara_module_dashboards"
        "scoped-records" = "tessara_module_scoped_records"
    }
    $databaseNames = [Collections.Generic.List[string]]::new()
    $runtimeUsers = [Collections.Generic.List[string]]::new()
    foreach ($serviceName in $expected.Keys) {
        $serviceProperty = $ComposeConfiguration.services.PSObject.Properties[$serviceName]
        if ($null -eq $serviceProperty) {
            throw "Normalized Sprint 8B Compose omits owner service '$serviceName'."
        }
        $databaseUrl = [string]$serviceProperty.Value.environment.DATABASE_URL
        $uri = [Uri]$databaseUrl
        $databaseName = $uri.AbsolutePath.TrimStart('/')
        $runtimeUser = $uri.UserInfo.Split(':', 2)[0]
        if ($databaseName -cne [string]$expected[$serviceName]) {
            throw "Owner service '$serviceName' targets unexpected database '$databaseName'."
        }
        if ([string]::IsNullOrWhiteSpace($runtimeUser)) {
            throw "Owner service '$serviceName' has no isolated runtime database principal."
        }
        $databaseNames.Add($databaseName)
        $runtimeUsers.Add($runtimeUser)
    }
    if (@($databaseNames | Sort-Object -Unique).Count -ne $expected.Count -or
        @($runtimeUsers | Sort-Object -Unique).Count -ne $expected.Count) {
        throw "Sprint 8B owner databases or runtime principals are not pairwise distinct."
    }
    [pscustomobject][ordered]@{
        pairwise_distinct_databases = @($databaseNames)
        pairwise_distinct_runtime_users = @($runtimeUsers)
    }
}

function Get-Sprint8BProjectResources {
    param([Parameter(Mandatory)][string]$ComposeProject)

    Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
    $containers = @(& docker ps -a --filter "label=com.docker.compose.project=$ComposeProject" --format "{{.ID}}")
    if ($LASTEXITCODE -ne 0) { throw "Could not enumerate containers for '$ComposeProject'." }
    $volumes = @(& docker volume ls --filter "label=com.docker.compose.project=$ComposeProject" --format "{{.Name}}")
    if ($LASTEXITCODE -ne 0) { throw "Could not enumerate volumes for '$ComposeProject'." }
    $networks = @(& docker network ls --filter "label=com.docker.compose.project=$ComposeProject" --format "{{.Name}}")
    if ($LASTEXITCODE -ne 0) { throw "Could not enumerate networks for '$ComposeProject'." }
    [pscustomobject][ordered]@{
        containers = @($containers | ForEach-Object { ([string]$_).Trim() } | Where-Object { $_ })
        volumes = @($volumes | ForEach-Object { ([string]$_).Trim() } | Where-Object { $_ })
        networks = @($networks | ForEach-Object { ([string]$_).Trim() } | Where-Object { $_ })
    }
}

function Assert-Sprint8BProjectAbsent {
    param([Parameter(Mandatory)][string]$ComposeProject)

    $resources = Get-Sprint8BProjectResources -ComposeProject $ComposeProject
    if ($resources.containers.Count -ne 0 -or $resources.volumes.Count -ne 0 -or $resources.networks.Count -ne 0) {
        throw "Disposable project '$ComposeProject' is not empty (containers=$($resources.containers.Count), volumes=$($resources.volumes.Count), networks=$($resources.networks.Count))."
    }
    $resources
}

function Remove-Sprint8BProjectTopology {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject,
        [Parameter(Mandatory)][bool]$Authorized
    )

    Assert-Sprint8BResetAuthorization -ComposeProject $ComposeProject -Authorized $Authorized
    if ([string]$env:COMPOSE_PROJECT_NAME -cne $ComposeProject) {
        throw "Refusing teardown because COMPOSE_PROJECT_NAME is not '$ComposeProject'."
    }
    $teardown = Invoke-Sprint8BDockerCompose -ComposePath $ComposePath `
        -Arguments @("down", "--volumes", "--remove-orphans") -AllowFailure
    $remaining = Get-Sprint8BProjectResources -ComposeProject $ComposeProject
    $passed = $teardown.exit_code -eq 0 -and
        $remaining.containers.Count -eq 0 -and
        $remaining.volumes.Count -eq 0 -and
        $remaining.networks.Count -eq 0
    $result = [pscustomobject][ordered]@{
        attempted = $true
        authorized = $Authorized
        compose_project = $ComposeProject
        exit_code = $teardown.exit_code
        remaining = $remaining
        passed = $passed
    }
    if (-not $passed) {
        throw "Exact teardown for '$ComposeProject' failed or retained project resources."
    }
    $result
}

function Get-Sprint8BSourceIdentity {
    param([switch]$RequireClean)

    Push-Location $script:Sprint8BRepositoryRoot
    try {
        $commit = (& git rev-parse HEAD).Trim()
        if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[0-9a-f]{40}$') {
            throw "Could not resolve the Sprint 8B source commit."
        }
        $tree = (& git rev-parse 'HEAD^{tree}').Trim()
        if ($LASTEXITCODE -ne 0 -or $tree -cnotmatch '^[0-9a-f]{40}$') {
            throw "Could not resolve the Sprint 8B source tree."
        }
        $status = @(& git status --porcelain=v1 --untracked-files=all)
        if ($LASTEXITCODE -ne 0) { throw "Could not inspect Sprint 8B source cleanliness." }
        $dirty = @($status | Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) }).Count -ne 0
        if ($RequireClean -and $dirty) {
            throw "Live Sprint 8B implementation proof requires a clean source worktree."
        }
        [pscustomobject][ordered]@{
            commit = $commit
            tree = $tree
            dirty = $dirty
        }
    } finally {
        Pop-Location
    }
}

function Get-Sprint8BComposeServiceState {
    param([Parameter(Mandatory)][string]$ComposePath)

    $result = Invoke-Sprint8BDockerCompose -ComposePath $ComposePath -Arguments @("ps", "--all", "--format", "json")
    @($result.output | Where-Object { $_ } | ForEach-Object { $_ | ConvertFrom-Json })
}

function Assert-Sprint8BExistingTopology {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject,
        [string[]]$RequiredServices = @(
            "core", "datasets", "components", "dashboards", "scoped-records", "supervisor", "gateway"
        )
    )

    Get-Sprint8BComposeConfiguration -ComposePath $ComposePath -ComposeProject $ComposeProject | Out-Null
    $states = @(Get-Sprint8BComposeServiceState -ComposePath $ComposePath)
    foreach ($serviceName in $RequiredServices) {
        $service = @($states | Where-Object { [string]$_.Service -ceq $serviceName })
        if ($service.Count -ne 1 -or [string]$service[0].State -cne "running" -or
            (-not [string]::IsNullOrWhiteSpace([string]$service[0].Health) -and
                [string]$service[0].Health -cne "healthy")) {
            throw "Existing '$ComposeProject' topology does not have exactly one healthy running '$serviceName' service."
        }
    }
    $states
}

function Invoke-Sprint8BHttpProbe {
    param(
        [Parameter(Mandatory)][string]$Uri,
        [int[]]$ExpectedStatus = @(200),
        [hashtable]$Headers = @{},
        [string]$Method = "GET"
    )

    $parameters = @{
        Uri = $Uri
        Method = $Method
        Headers = $Headers
        UseBasicParsing = $true
        SkipHttpErrorCheck = $true
        TimeoutSec = 15
    }
    $response = Invoke-WebRequest @parameters
    $status = [int]$response.StatusCode
    $content = [string]$response.Content
    $result = [pscustomobject][ordered]@{
        uri = $Uri
        method = $Method
        status = $status
        content_type = [string]$response.Headers.'Content-Type'
        body_sha256 = Get-Sprint7ASha256 -Text $content
        body = $content
        passed = $ExpectedStatus -contains $status
    }
    if (-not $result.passed) {
        throw "HTTP probe '$Method $Uri' returned $status; expected one of $($ExpectedStatus -join ', ')."
    }
    $result
}

function Wait-Sprint8BHttpProbe {
    param(
        [Parameter(Mandatory)][string]$Uri,
        [int[]]$ExpectedStatus = @(200),
        [int]$Attempts = 60
    )

    $lastError = $null
    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        try {
            return Invoke-Sprint8BHttpProbe -Uri $Uri -ExpectedStatus $ExpectedStatus
        } catch {
            $lastError = $_
            if ($attempt -lt $Attempts) { Start-Sleep -Seconds 1 }
        }
    }
    throw "HTTP endpoint '$Uri' did not reach the expected status: $($lastError.Exception.Message)"
}

function Publish-Sprint8BHarnessEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$OutputPath
    )

    $resolved = Resolve-Sprint8BRepositoryPath -Path $OutputPath
    Publish-Sprint7AEvidence -Document $Document -OutputPath $resolved
}

function Assert-Sprint8BPreparedResponseFixtures {
    param([Parameter(Mandatory)]$FixtureReceipt)

    $expectedKeys = @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.new",
        "response.corrected", "response.status-out", "response.status-in", "response.redacted",
        "response.deleted", "response.outside-scope"
    ) | Sort-Object
    $responses = $FixtureReceipt.logical_identities.core.responses
    $actualKeys = @($responses.PSObject.Properties.Name | Sort-Object)
    if (($actualKeys -join "`n") -cne ($expectedKeys -join "`n")) {
        throw "Prepared Response fixture keys are not the exact Sprint 8B scenario set."
    }
    $preInitial = @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.outside-scope"
    )
    foreach ($key in $expectedKeys) {
        $identity = $responses.PSObject.Properties[$key].Value
        if ([string]$identity.response_id -cnotmatch
            '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
            throw "Prepared Response fixture '$key' has a non-canonical identity."
        }
        $expectedProvenance = if ($preInitial -ccontains $key) {
            "signed-core-bootstrap-receipt"
        } else { "signed-core-response-owner-action-receipt" }
        if ([string]$identity.provenance -cne $expectedProvenance) {
            throw "Prepared Response fixture '$key' has substituted provenance."
        }
    }

    $ownerActions = $FixtureReceipt.response_owner_actions
    if ([int]$ownerActions.schema_version -ne 1 -or
        [string]$ownerActions.endpoint -cne "/api/admin/responses/owner-actions" -or
        [string]$ownerActions.media_type -cne
            "application/vnd.tessara.responses.owner-action+json;version=1" -or
        [string]$ownerActions.authentication -cne "bearer-owner-capability" -or
        [string]$ownerActions.signature -cne "ed25519-purpose-bound" -or
        [string]$ownerActions.replay -cne "exact-signed-receipt" -or
        [int]$ownerActions.action_count -ne 12 -or
        [string]$ownerActions.strictly_monotonic_change_sequence -cne "passed") {
        throw "Prepared Response owner-action contract is incomplete or substituted."
    }
    $expectedActions = @(
        [pscustomobject]@{ logical_key = "response.new"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.corrected"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.corrected"; action = "correct"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.status-out"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.status-out"; action = "status_out"; kind = "tombstone"; reason = "status_excluded" }
        [pscustomobject]@{ logical_key = "response.status-in"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.status-in"; action = "status_out"; kind = "tombstone"; reason = "status_excluded" }
        [pscustomobject]@{ logical_key = "response.status-in"; action = "status_in"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.redacted"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.redacted"; action = "redact"; kind = "tombstone"; reason = "redacted" }
        [pscustomobject]@{ logical_key = "response.deleted"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.deleted"; action = "delete"; kind = "tombstone"; reason = "deleted" }
    )
    $receipts = @($ownerActions.receipts)
    if ($receipts.Count -ne $expectedActions.Count) {
        throw "Prepared Response owner-action receipt count is not exact."
    }
    [uint64]$previousSequence = 0
    for ($index = 0; $index -lt $receipts.Count; $index++) {
        $receipt = $receipts[$index]
        $expected = $expectedActions[$index]
        $expectedResponse = $responses.PSObject.Properties[[string]$expected.logical_key].Value
        if ([string]$receipt.logical_key -cne [string]$expected.logical_key -or
            [string]$receipt.action -cne [string]$expected.action -or
            [string]$receipt.response_id -cne [string]$expectedResponse.response_id -or
            [string]$receipt.change_kind -cne [string]$expected.kind -or
            [string]$receipt.tombstone_reason -cne [string]$expected.reason -or
            [uint64]$receipt.change_sequence -le $previousSequence -or
            -not [bool]$receipt.replay_verified -or
            [string]$receipt.signature_verification.state -cne "passed" -or
            [string]$receipt.signed_receipt.issuer -cne "tessara.core" -or
            [string]$receipt.signed_receipt.key_id -cne "core-development-v1" -or
            [string]$receipt.signed_receipt.purpose -cne "response_owner_action_receipt") {
            throw "Prepared Response owner-action receipt at index $index is not exact and monotonic."
        }
        $previousSequence = [uint64]$receipt.change_sequence
    }
    [pscustomobject][ordered]@{
        state = "passed"
        response_count = $expectedKeys.Count
        post_bootstrap_response_count = $expectedKeys.Count - $preInitial.Count
        owner_action_count = $receipts.Count
        terminal_change_sequence = $previousSequence
    }
}

function New-Sprint8BPreparedResponseFixturesSelfTestProjection {
    $keys = @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.new",
        "response.corrected", "response.status-out", "response.status-in", "response.redacted",
        "response.deleted", "response.outside-scope"
    )
    $preInitial = @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.outside-scope"
    )
    $responses = [ordered]@{}
    $ordinal = 0
    foreach ($key in $keys) {
        $ordinal++
        $responses[$key] = [pscustomobject][ordered]@{
            response_id = "01980000-0088-7000-8000-{0:d12}" -f $ordinal
            provenance = if ($preInitial -ccontains $key) {
                "signed-core-bootstrap-receipt"
            } else { "signed-core-response-owner-action-receipt" }
        }
    }
    $actions = @(
        [pscustomobject]@{ logical_key = "response.new"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.corrected"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.corrected"; action = "correct"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.status-out"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.status-out"; action = "status_out"; kind = "tombstone"; reason = "status_excluded" }
        [pscustomobject]@{ logical_key = "response.status-in"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.status-in"; action = "status_out"; kind = "tombstone"; reason = "status_excluded" }
        [pscustomobject]@{ logical_key = "response.status-in"; action = "status_in"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.redacted"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.redacted"; action = "redact"; kind = "tombstone"; reason = "redacted" }
        [pscustomobject]@{ logical_key = "response.deleted"; action = "create"; kind = "upsert"; reason = $null }
        [pscustomobject]@{ logical_key = "response.deleted"; action = "delete"; kind = "tombstone"; reason = "deleted" }
    )
    $receipts = [Collections.Generic.List[object]]::new()
    $sequence = 40
    foreach ($action in $actions) {
        $sequence++
        $receipts.Add([pscustomobject][ordered]@{
            logical_key = $action.logical_key
            action = $action.action
            response_id = [string]$responses[$action.logical_key].response_id
            idempotency_key_digest = "sha256:$('a' * 64)"
            raw_body_digest = "sha256:$('b' * 64)"
            change_sequence = $sequence
            change_kind = $action.kind
            tombstone_reason = $action.reason
            replay_verified = $true
            signed_receipt = [pscustomobject][ordered]@{
                schema_version = 1
                issuer = "tessara.core"
                key_id = "core-development-v1"
                purpose = "response_owner_action_receipt"
                payload = [pscustomobject]@{ logical_key = $action.logical_key }
                signature = "self-test"
            }
            signature_verification = [pscustomobject]@{
                state = "passed"; signing_input_sha256 = "c" * 64
            }
        })
    }
    [pscustomobject][ordered]@{
        responses = [pscustomobject]$responses
        response_owner_actions = [pscustomobject][ordered]@{
            schema_version = 1
            endpoint = "/api/admin/responses/owner-actions"
            media_type = "application/vnd.tessara.responses.owner-action+json;version=1"
            authentication = "bearer-owner-capability"
            signature = "ed25519-purpose-bound"
            replay = "exact-signed-receipt"
            action_count = $receipts.Count
            strictly_monotonic_change_sequence = "passed"
            receipts = @($receipts)
        }
    }
}

function Invoke-Sprint8BChildScript {
    param(
        [Parameter(Mandatory)][string]$ScriptPath,
        [AllowEmptyCollection()][string[]]$Arguments = @(),
        [switch]$AllowFailure
    )

    $resolved = Resolve-Sprint8BRepositoryPath -Path $ScriptPath
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "Required Sprint 8B child script is missing: $resolved"
    }
    $output = @(& pwsh -NoProfile -File $resolved @Arguments 2>&1 | ForEach-Object { [string]$_ })
    $exitCode = $LASTEXITCODE
    if (-not $AllowFailure -and $exitCode -ne 0) {
        throw "Child script '$resolved' exited $exitCode.`n$($output -join "`n")"
    }
    [pscustomobject][ordered]@{
        script = $resolved
        arguments = @($Arguments)
        exit_code = $exitCode
        output = @($output)
    }
}

function Test-Sprint8BHarnessIsolation {
    foreach ($valid in @(
        "tessara-s8b-readiness-materialization",
        "tessara-s8b-rehearsal-smoke",
        "tessara-s8b-sit",
        "tessara-s8b-uat-upgrade"
    )) {
        if ((Assert-Sprint8BComposeProject -ComposeProject $valid) -cne $valid) {
            throw "Sprint 8B Compose project validation did not round-trip '$valid'."
        }
    }
    foreach ($invalid in @(
        "tessara-sprint-8b", "tessara-s8b-", "TESSARA-S8B-SIT", "tessara-s8b_bad", "other"
    )) {
        try {
            Assert-Sprint8BComposeProject -ComposeProject $invalid | Out-Null
            throw "Sprint 8B Compose project validation accepted '$invalid'."
        } catch {
            if ($_.Exception.Message -notmatch 'outside the exact Sprint 8B') { throw }
        }
    }
    try {
        Assert-Sprint8BResetAuthorization -ComposeProject "tessara-s8b-selftest" -Authorized $false
        throw "Sprint 8B reset authorization self-test accepted an unauthorized reset."
    } catch {
        if ($_.Exception.Message -notmatch 'requires -AuthorizeDisposableReset') { throw }
    }

    $responseProjection = New-Sprint8BPreparedResponseFixturesSelfTestProjection
    $responseFixture = [pscustomobject][ordered]@{
        logical_identities = [pscustomobject][ordered]@{
            core = [pscustomobject][ordered]@{ responses = $responseProjection.responses }
        }
        response_owner_actions = $responseProjection.response_owner_actions
    }
    $responseProof = Assert-Sprint8BPreparedResponseFixtures -FixtureReceipt $responseFixture
    if ($responseProof.response_count -ne 10 -or $responseProof.owner_action_count -ne 12) {
        throw "Prepared Response fixture self-test did not prove its exact scenario/action set."
    }
    $tamperedResponseFixture = $responseFixture | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedResponseFixture.response_owner_actions.receipts[2].change_sequence = 41
    $rejected = $false
    try {
        Assert-Sprint8BPreparedResponseFixtures -FixtureReceipt $tamperedResponseFixture | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Prepared Response fixture self-test accepted a non-monotonic action receipt."
    }

    $names = @("COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT")
    $original = Get-Sprint8BProcessEnvironmentSnapshot -Names $names
    try {
        $caller = Get-Sprint8BProcessEnvironmentSnapshot -Names $names
        $ports = Set-Sprint8BComposeEnvironment -ComposeProject "tessara-s8b-selftest" `
            -GatewayPort 18001 -CorePort 18002 -SupervisorPort 18003
        if ($ports.compose_project -cne "tessara-s8b-selftest" -or
            [string]$env:COMPOSE_PROJECT_NAME -cne "tessara-s8b-selftest" -or
            @($ports.gateway_port, $ports.core_port, $ports.supervisor_port | Sort-Object -Unique).Count -ne 3) {
            throw "Sprint 8B Compose environment projection self-test failed."
        }
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $caller
    } finally {
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $original
    }

    $mockConfiguration = [pscustomobject]@{
        services = [pscustomobject][ordered]@{
            core = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://core:secret@postgres:5432/tessara_core" } }
            datasets = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://datasets:secret@postgres:5432/tessara_module_datasets" } }
            components = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://components:secret@postgres:5432/tessara_module_components" } }
            dashboards = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://dashboards:secret@postgres:5432/tessara_module_dashboards" } }
            "scoped-records" = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://scoped:secret@postgres:5432/tessara_module_scoped_records" } }
        }
    }
    $isolation = Assert-Sprint8BDatabaseIsolationConfiguration -ComposeConfiguration $mockConfiguration
    if ($isolation.pairwise_distinct_databases.Count -ne 5 -or
        $isolation.pairwise_distinct_runtime_users.Count -ne 5) {
        throw "Sprint 8B database isolation self-test did not prove five owner boundaries."
    }
    $tampered = $mockConfiguration | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
    $tampered.services.datasets.environment.DATABASE_URL = $tampered.services.core.environment.DATABASE_URL
    try {
        Assert-Sprint8BDatabaseIsolationConfiguration -ComposeConfiguration $tampered | Out-Null
        throw "Sprint 8B database isolation self-test accepted a foreign owner database."
    } catch {
        if ($_.Exception.Message -notmatch "unexpected database") { throw }
    }

    $root = Join-Path ([IO.Path]::GetTempPath()) "tessara-s8b-harness-$([Guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $path = Join-Path $root "result.json"
        $published = Publish-Sprint8BHarnessEvidence -Document ([ordered]@{
            schema_version = 1
            proof = "harness-isolation-self-test"
            state = "passed"
        }) -OutputPath $path
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $published.path -SidecarPath "$($published.path).sha256")) {
            throw "Sprint 8B harness evidence publication self-test failed."
        }
    } finally {
        if (Test-Path -LiteralPath $root) {
            Remove-Item -LiteralPath $root -Recurse -Force
        }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "harness-isolation-self-test"
        database_free = $true
        state = "passed"
    }
}

if ($SelfTest) {
    Test-Sprint8BHarnessIsolation | ConvertTo-Json -Depth 20
}

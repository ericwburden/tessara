[CmdletBinding()]
param(
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$script:Sprint8CRepositoryRoot = Split-Path -Parent $PSScriptRoot

. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

function Resolve-Sprint8CRepositoryPath {
    param([Parameter(Mandatory)][string]$Path)

    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $script:Sprint8CRepositoryRoot $Path))
}

function Assert-Sprint8CComposeProject {
    param([Parameter(Mandatory)][string]$ComposeProject)

    if ($ComposeProject -cnotmatch '^tessara-s8c-[a-z0-9](?:[a-z0-9-]{0,46}[a-z0-9])?$') {
        throw "Compose project '$ComposeProject' is outside the exact Sprint 8C isolated-lane namespace."
    }
    $ComposeProject
}

function Assert-Sprint8CResetAuthorization {
    param(
        [Parameter(Mandatory)][string]$ComposeProject,
        [Parameter(Mandatory)][bool]$Authorized
    )

    Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
    if (-not $Authorized) {
        throw "Reset of disposable project '$ComposeProject' requires -AuthorizeDisposableReset."
    }
}

function Get-Sprint8CProcessEnvironmentSnapshot {
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

function Restore-Sprint8CProcessEnvironmentSnapshot {
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

function Get-Sprint8CFreeTcpPort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    try {
        $listener.Start()
        ([Net.IPEndPoint]$listener.LocalEndpoint).Port
    } finally {
        $listener.Stop()
    }
}

function Set-Sprint8CComposeEnvironment {
    param(
        [Parameter(Mandatory)][string]$ComposeProject,
        [Nullable[int]]$GatewayPort,
        [Nullable[int]]$CorePort,
        [Nullable[int]]$SupervisorPort
    )

    Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
    $resolvedGatewayPort = if ($null -eq $GatewayPort) { Get-Sprint8CFreeTcpPort } else { [int]$GatewayPort }
    $resolvedCorePort = if ($null -eq $CorePort) { Get-Sprint8CFreeTcpPort } else { [int]$CorePort }
    $resolvedSupervisorPort = if ($null -eq $SupervisorPort) { Get-Sprint8CFreeTcpPort } else { [int]$SupervisorPort }
    $ports = @($resolvedGatewayPort, $resolvedCorePort, $resolvedSupervisorPort)
    if (@($ports | Sort-Object -Unique).Count -ne 3 -or @($ports | Where-Object { $_ -lt 1024 -or $_ -gt 65535 }).Count -ne 0) {
        throw "Sprint 8C gateway, Core, and Supervisor ports must be three distinct non-privileged TCP ports."
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

function Invoke-Sprint8CDockerCompose {
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

function Get-Sprint8CComposeConfiguration {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject
    )

    Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
    if ([string]$env:COMPOSE_PROJECT_NAME -cne $ComposeProject) {
        throw "COMPOSE_PROJECT_NAME must equal '$ComposeProject' before resolving Sprint 8C Compose."
    }
    $result = Invoke-Sprint8CDockerCompose -ComposePath $ComposePath -Arguments @("config", "--format", "json")
    $configuration = ($result.output -join "`n") | ConvertFrom-Json -Depth 100
    if ([string]$configuration.name -cne $ComposeProject) {
        throw "Normalized Compose project '$($configuration.name)' does not equal '$ComposeProject'."
    }
    $configuration
}

function Assert-Sprint8CDatabaseIsolationConfiguration {
    param([Parameter(Mandatory)]$ComposeConfiguration)

    $expected = [ordered]@{
        core = "tessara_core"
        responses = "tessara_module_responses"
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
            throw "Normalized Sprint 8C Compose omits owner service '$serviceName'."
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
        throw "Sprint 8C owner databases or runtime principals are not pairwise distinct."
    }
    [pscustomobject][ordered]@{
        pairwise_distinct_databases = @($databaseNames)
        pairwise_distinct_runtime_users = @($runtimeUsers)
    }
}

function Get-Sprint8CProjectResources {
    param([Parameter(Mandatory)][string]$ComposeProject)

    Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
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

function Assert-Sprint8CProjectAbsent {
    param([Parameter(Mandatory)][string]$ComposeProject)

    $resources = Get-Sprint8CProjectResources -ComposeProject $ComposeProject
    if ($resources.containers.Count -ne 0 -or $resources.volumes.Count -ne 0 -or $resources.networks.Count -ne 0) {
        throw "Disposable project '$ComposeProject' is not empty (containers=$($resources.containers.Count), volumes=$($resources.volumes.Count), networks=$($resources.networks.Count))."
    }
    $resources
}

function Remove-Sprint8CProjectTopology {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject,
        [Parameter(Mandatory)][bool]$Authorized
    )

    Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject -Authorized $Authorized
    if ([string]$env:COMPOSE_PROJECT_NAME -cne $ComposeProject) {
        throw "Refusing teardown because COMPOSE_PROJECT_NAME is not '$ComposeProject'."
    }
    $teardown = Invoke-Sprint8CDockerCompose -ComposePath $ComposePath `
        -Arguments @("down", "--volumes", "--remove-orphans") -AllowFailure
    $remaining = Get-Sprint8CProjectResources -ComposeProject $ComposeProject
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

function Get-Sprint8CSourceIdentity {
    param([switch]$RequireClean)

    Push-Location $script:Sprint8CRepositoryRoot
    try {
        $commit = (& git rev-parse HEAD).Trim()
        if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[0-9a-f]{40}$') {
            throw "Could not resolve the Sprint 8C source commit."
        }
        $tree = (& git rev-parse 'HEAD^{tree}').Trim()
        if ($LASTEXITCODE -ne 0 -or $tree -cnotmatch '^[0-9a-f]{40}$') {
            throw "Could not resolve the Sprint 8C source tree."
        }
        $status = @(& git status --porcelain=v1 --untracked-files=all)
        if ($LASTEXITCODE -ne 0) { throw "Could not inspect Sprint 8C source cleanliness." }
        $dirty = @($status | Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) }).Count -ne 0
        if ($RequireClean -and $dirty) {
            throw "Live Sprint 8C implementation proof requires a clean source worktree."
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

function Get-Sprint8CComposeServiceState {
    param([Parameter(Mandatory)][string]$ComposePath)

    $result = Invoke-Sprint8CDockerCompose -ComposePath $ComposePath -Arguments @("ps", "--all", "--format", "json")
    @($result.output | Where-Object { $_ } | ForEach-Object { $_ | ConvertFrom-Json })
}

function Assert-Sprint8CExistingTopology {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ComposeProject,
        [string[]]$RequiredServices = @(
            "core", "responses", "datasets", "components", "dashboards", "scoped-records",
            "supervisor", "gateway"
        )
    )

    Get-Sprint8CComposeConfiguration -ComposePath $ComposePath -ComposeProject $ComposeProject | Out-Null
    $states = @(Get-Sprint8CComposeServiceState -ComposePath $ComposePath)
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

function Invoke-Sprint8CHttpProbe {
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
    $contentTypeProperty = $response.Headers.PSObject.Properties['Content-Type']
    $contentType = if ($null -eq $contentTypeProperty) {
        ''
    } else {
        [string]$contentTypeProperty.Value
    }
    $result = [pscustomobject][ordered]@{
        uri = $Uri
        method = $Method
        status = $status
        content_type = $contentType
        body_sha256 = Get-Sprint7ASha256 -Text $content
        body = $content
        passed = $ExpectedStatus -contains $status
    }
    if (-not $result.passed) {
        throw "HTTP probe '$Method $Uri' returned $status; expected one of $($ExpectedStatus -join ', ')."
    }
    $result
}

function Wait-Sprint8CHttpProbe {
    param(
        [Parameter(Mandatory)][string]$Uri,
        [int[]]$ExpectedStatus = @(200),
        [int]$Attempts = 60
    )

    $lastError = $null
    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        try {
            return Invoke-Sprint8CHttpProbe -Uri $Uri -ExpectedStatus $ExpectedStatus
        } catch {
            $lastError = $_
            if ($attempt -lt $Attempts) { Start-Sleep -Seconds 1 }
        }
    }
    throw "HTTP endpoint '$Uri' did not reach the expected status: $($lastError.Exception.Message)"
}

function Publish-Sprint8CHarnessEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$OutputPath
    )

    $resolved = Resolve-Sprint8CRepositoryPath -Path $OutputPath
    Publish-Sprint7AEvidence -Document $Document -OutputPath $resolved
}

function Assert-Sprint8CPreparedResponseFixtures {
    param([Parameter(Mandatory)]$FixtureReceipt)

    $expectedStates = [ordered]@{
        "response.draft.owner" = "draft"
        "response.submitted.owner" = "submitted"
        "response.submitted.delegated" = "submitted"
        "response.submitted.restricted" = "submitted"
    }
    $expectedKeys = @($expectedStates.Keys)
    $expectedKeySet = @($expectedKeys | Sort-Object)
    $responses = $FixtureReceipt.logical_identities.responses
    $actualKeys = @($responses.PSObject.Properties.Name | Sort-Object)
    if (($actualKeys -join "`n") -cne ($expectedKeySet -join "`n")) {
        throw "Prepared Response fixture keys are not the exact Response-owner bootstrap set."
    }
    $responseOwnerDigests = @($FixtureReceipt.owner_receipt_digests | Where-Object {
        [string]$_.owner -ceq "tessara.responses"
    })
    if ($responseOwnerDigests.Count -ne 1 -or
        [string]$responseOwnerDigests[0].input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
        [string]$responseOwnerDigests[0].result_digest -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Prepared Response fixtures lack one exact authenticated Response owner receipt digest."
    }

    $moduleInstanceIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $responseIds = [ordered]@{}
    $lifecycleStates = [ordered]@{}
    $previousWorkflowEventSequence = [uint64]0
    foreach ($key in $expectedKeys) {
        $identity = $responses.PSObject.Properties[$key].Value
        if ([string]$identity.response_id -cnotmatch
            '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
            throw "Prepared Response fixture '$key' has a non-canonical identity."
        }
        $reference = $identity.reference.reference
        $readBack = $identity.read_back
        $expectedRevision = if ([string]$expectedStates[$key] -ceq "draft") { 1 } else { 2 }
        if ([string]$identity.provenance -cne "signed-tessara.responses-bootstrap-receipt" -or
            [string]$reference.resource_type -cne "tessara.responses.response" -or
            [string]$reference.resource_id -cne [string]$identity.response_id -or
            [string]$reference.installation_id -cne [string]$FixtureReceipt.installation_id -or
            [string]$reference.owner.kind -cne "module_instance" -or
            [string]$reference.owner.installation_id -cne [string]$reference.installation_id -or
            [string]::IsNullOrWhiteSpace([string]$reference.owner.module_instance_id) -or
            [int]$readBack.schema_version -ne 1 -or
            (($readBack.response | ConvertTo-Json -Depth 30 -Compress) -cne
                ($identity.reference | ConvertTo-Json -Depth 30 -Compress)) -or
            [string]$readBack.lifecycle_state -cne [string]$expectedStates[$key] -or
            [uint64]$readBack.revision -ne $expectedRevision -or
            [uint64]$readBack.workflow_event_sequence -le $previousWorkflowEventSequence) {
            throw "Prepared Response fixture '$key' lacks exact typed Response-owner read-back."
        }
        $previousWorkflowEventSequence = [uint64]$readBack.workflow_event_sequence
        foreach ($idName in @(
            "workflow_assignment_id", "workflow_instance_id", "workflow_step_instance_id"
        )) {
            if ([string]$readBack.$idName -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
                throw "Prepared Response fixture '$key' has a non-canonical '$idName'."
            }
        }
        [void]$moduleInstanceIds.Add([string]$reference.owner.module_instance_id)
        $responseIds[$key] = [string]$identity.response_id
        $lifecycleStates[$key] = [string]$readBack.lifecycle_state
    }
    if ($moduleInstanceIds.Count -ne 1) {
        throw "Prepared Response fixtures do not identify exactly one Response Module Instance."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        response_count = $expectedKeys.Count
        owner = "tessara.responses"
        installation_id = [string]$FixtureReceipt.installation_id
        module_instance_id = @($moduleInstanceIds)[0]
        response_ids = [pscustomobject]$responseIds
        lifecycle_states = [pscustomobject]$lifecycleStates
        input_digest = [string]$responseOwnerDigests[0].input_digest
        result_digest = [string]$responseOwnerDigests[0].result_digest
    }
}

function New-Sprint8CPreparedResponseFixturesSelfTestProjection {
    $installationId = "01980000-0000-7000-8000-00000000008c"
    $moduleInstanceId = "01980000-0000-7000-8000-000000000083"
    $states = [ordered]@{
        "response.draft.owner" = "draft"
        "response.submitted.owner" = "submitted"
        "response.submitted.delegated" = "submitted"
        "response.submitted.restricted" = "submitted"
    }
    $responses = [ordered]@{}
    $ordinal = 0
    foreach ($key in $states.Keys) {
        $ordinal++
        $responseId = "01980000-0088-7000-8000-{0:d12}" -f $ordinal
        $revision = if ([string]$states[$key] -ceq "draft") { 1 } else { 2 }
        $reference = [pscustomobject][ordered]@{
            reference = [pscustomobject][ordered]@{
                installation_id = $installationId
                owner = [pscustomobject][ordered]@{
                    kind = "module_instance"
                    installation_id = $installationId
                    module_instance_id = $moduleInstanceId
                }
                resource_type = "tessara.responses.response"
                resource_id = $responseId
            }
        }
        $responses[$key] = [pscustomobject][ordered]@{
            response_id = $responseId
            reference = $reference
            read_back = [pscustomobject][ordered]@{
                schema_version = 1
                response = $reference
                lifecycle_state = [string]$states[$key]
                revision = $revision
                workflow_assignment_id = "01980000-0090-7000-8000-{0:d12}" -f $ordinal
                workflow_instance_id = "01980000-0091-7000-8000-{0:d12}" -f $ordinal
                workflow_step_instance_id = "01980000-0092-7000-8000-{0:d12}" -f $ordinal
                workflow_event_sequence = (($ordinal * 2) - 1)
            }
            provenance = "signed-tessara.responses-bootstrap-receipt"
        }
    }
    [pscustomobject][ordered]@{
        installation_id = $installationId
        responses = [pscustomobject]$responses
        owner_receipt_digest = [pscustomobject][ordered]@{
            owner = "tessara.responses"
            input_digest = "sha256:$('a' * 64)"
            result_digest = "sha256:$('b' * 64)"
        }
    }
}

function Invoke-Sprint8CChildScript {
    param(
        [Parameter(Mandatory)][string]$ScriptPath,
        [AllowEmptyCollection()][string[]]$Arguments = @(),
        [switch]$AllowFailure
    )

    $resolved = Resolve-Sprint8CRepositoryPath -Path $ScriptPath
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "Required Sprint 8C child script is missing: $resolved"
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

function Test-Sprint8CHarnessIsolation {
    foreach ($valid in @(
        "tessara-s8c-readiness-materialization",
        "tessara-s8c-rehearsal-smoke",
        "tessara-s8c-sit",
        "tessara-s8c-uat-upgrade"
    )) {
        if ((Assert-Sprint8CComposeProject -ComposeProject $valid) -cne $valid) {
            throw "Sprint 8C Compose project validation did not round-trip '$valid'."
        }
    }
    foreach ($invalid in @(
        "tessara-sprint-8c", "tessara-s8c-", "TESSARA-S8B-SIT", "tessara-s8c_bad", "other"
    )) {
        try {
            Assert-Sprint8CComposeProject -ComposeProject $invalid | Out-Null
            throw "Sprint 8C Compose project validation accepted '$invalid'."
        } catch {
            if ($_.Exception.Message -notmatch 'outside the exact Sprint 8C') { throw }
        }
    }
    try {
        Assert-Sprint8CResetAuthorization -ComposeProject "tessara-s8c-selftest" -Authorized $false
        throw "Sprint 8C reset authorization self-test accepted an unauthorized reset."
    } catch {
        if ($_.Exception.Message -notmatch 'requires -AuthorizeDisposableReset') { throw }
    }

    $responseProjection = New-Sprint8CPreparedResponseFixturesSelfTestProjection
    $responseFixture = [pscustomobject][ordered]@{
        installation_id = $responseProjection.installation_id
        logical_identities = [pscustomobject][ordered]@{
            responses = $responseProjection.responses
        }
        owner_receipt_digests = @($responseProjection.owner_receipt_digest)
    }
    $responseProof = Assert-Sprint8CPreparedResponseFixtures -FixtureReceipt $responseFixture
    if ($responseProof.response_count -ne 4 -or
        [string]$responseProof.owner -cne "tessara.responses") {
        throw "Prepared Response fixture self-test did not prove its exact owner bootstrap set."
    }
    $tamperedResponseFixture = $responseFixture | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedResponseFixture.logical_identities.responses.'response.submitted.owner'.read_back.lifecycle_state = "draft"
    $rejected = $false
    try {
        Assert-Sprint8CPreparedResponseFixtures -FixtureReceipt $tamperedResponseFixture | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Prepared Response fixture self-test accepted substituted lifecycle read-back."
    }
    $tamperedResponseFixture = $responseFixture | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedResponseFixture.logical_identities.responses.'response.submitted.delegated'.read_back.workflow_event_sequence = 3
    $rejected = $false
    try {
        Assert-Sprint8CPreparedResponseFixtures -FixtureReceipt $tamperedResponseFixture | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Prepared Response fixture self-test accepted a non-monotonic workflow event cursor."
    }

    $names = @("COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT")
    $original = Get-Sprint8CProcessEnvironmentSnapshot -Names $names
    try {
        $caller = Get-Sprint8CProcessEnvironmentSnapshot -Names $names
        $ports = Set-Sprint8CComposeEnvironment -ComposeProject "tessara-s8c-selftest" `
            -GatewayPort 18001 -CorePort 18002 -SupervisorPort 18003
        if ($ports.compose_project -cne "tessara-s8c-selftest" -or
            [string]$env:COMPOSE_PROJECT_NAME -cne "tessara-s8c-selftest" -or
            @($ports.gateway_port, $ports.core_port, $ports.supervisor_port | Sort-Object -Unique).Count -ne 3) {
            throw "Sprint 8C Compose environment projection self-test failed."
        }
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $caller
    } finally {
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $original
    }

    $mockConfiguration = [pscustomobject]@{
        services = [pscustomobject][ordered]@{
            core = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://core:secret@postgres:5432/tessara_core" } }
            responses = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://responses:secret@postgres:5432/tessara_module_responses" } }
            datasets = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://datasets:secret@postgres:5432/tessara_module_datasets" } }
            components = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://components:secret@postgres:5432/tessara_module_components" } }
            dashboards = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://dashboards:secret@postgres:5432/tessara_module_dashboards" } }
            "scoped-records" = [pscustomobject]@{ environment = [pscustomobject]@{ DATABASE_URL = "postgres://scoped:secret@postgres:5432/tessara_module_scoped_records" } }
        }
    }
    $isolation = Assert-Sprint8CDatabaseIsolationConfiguration -ComposeConfiguration $mockConfiguration
    if ($isolation.pairwise_distinct_databases.Count -ne 6 -or
        $isolation.pairwise_distinct_runtime_users.Count -ne 6) {
        throw "Sprint 8C database isolation self-test did not prove six owner boundaries."
    }
    $tampered = $mockConfiguration | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
    $tampered.services.datasets.environment.DATABASE_URL = $tampered.services.core.environment.DATABASE_URL
    try {
        Assert-Sprint8CDatabaseIsolationConfiguration -ComposeConfiguration $tampered | Out-Null
        throw "Sprint 8C database isolation self-test accepted a foreign owner database."
    } catch {
        if ($_.Exception.Message -notmatch "unexpected database") { throw }
    }

    $root = Join-Path ([IO.Path]::GetTempPath()) "tessara-s8c-harness-$([Guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $path = Join-Path $root "result.json"
        $published = Publish-Sprint8CHarnessEvidence -Document ([ordered]@{
            schema_version = 1
            proof = "harness-isolation-self-test"
            state = "passed"
        }) -OutputPath $path
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $published.path -SidecarPath "$($published.path).sha256")) {
            throw "Sprint 8C harness evidence publication self-test failed."
        }
    } finally {
        if (Test-Path -LiteralPath $root) {
            Remove-Item -LiteralPath $root -Recurse -Force
        }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "harness-isolation-self-test"
        database_free = $true
        state = "passed"
    }
}

if ($SelfTest) {
    Test-Sprint8CHarnessIsolation | ConvertTo-Json -Depth 20
}

[CmdletBinding(SupportsShouldProcess, ConfirmImpact = "High")]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BlueprintPath = "deploy/sprint-8a/blueprints/reference.json",
    [string]$CoreUrl = "http://127.0.0.1:8088",
    [string]$ControlUrl = "http://127.0.0.1:18088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [ValidateRange(1, 2147483647)]
    [int]$Attempt = 1,
    [string]$EvidenceRoot = "target/sprint-8a-bootstrap",
    [string]$EnvironmentFingerprint,
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$VerifyNoOp,
    [string]$ExpectedFaultId,
    [string]$ExpectedFailurePattern,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-health-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")

$expectedProject = "tessara-sprint-8a"
$installationId = "01980000-0000-7000-8000-00000000008a"
$componentOwner = "142a1ece-f74b-85f6-8ca0-92f4a02e9409"
$expectedOwnerOrder = @(
    "core",
    "tessara.components",
    "tessara.dashboards",
    "tessara.reference.scoped-records"
)

function Assert-Sprint8ADestructiveEndpoint {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][int]$ExpectedPort
    )
    $uri = $null
    $parsed = -not [string]::IsNullOrWhiteSpace($Value) -and
        [Uri]::TryCreate($Value, [UriKind]::Absolute, [ref]$uri)
    $exactRoot = $false
    if ($parsed) {
        $canonicalRoot = "http://$($uri.Authority)/"
        $canonicalWithoutSlash = $canonicalRoot.Substring(0, $canonicalRoot.Length - 1)
        $exactRoot = [string]::Equals($Value, $canonicalRoot, [StringComparison]::OrdinalIgnoreCase) -or
            [string]::Equals($Value, $canonicalWithoutSlash, [StringComparison]::OrdinalIgnoreCase)
    }
    if (-not $parsed -or
        $uri.Scheme -cne [Uri]::UriSchemeHttp -or
        -not $uri.IsLoopback -or
        $uri.Port -ne $ExpectedPort -or
        -not [string]::IsNullOrEmpty($uri.UserInfo) -or
        -not [string]::IsNullOrEmpty($uri.Query) -or
        -not [string]::IsNullOrEmpty($uri.Fragment) -or
        $uri.AbsolutePath -cne "/" -or
        -not $exactRoot) {
        throw "Destructive Sprint 8A execution requires $Name to be an exact HTTP loopback root endpoint on port $ExpectedPort with no credentials, query, or fragment."
    }
}

function Resolve-Sprint8ARepositoryPath {
    param([Parameter(Mandatory)][string]$Path)
    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Get-Sprint8ADisplayPath {
    param([Parameter(Mandatory)][string]$Path)
    $fullPath = [IO.Path]::GetFullPath($Path)
    $rootPrefix = [IO.Path]::GetFullPath($repoRoot).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if ($fullPath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        return $fullPath.Substring($rootPrefix.Length).Replace("\", "/")
    }
    $fullPath.Replace("\", "/")
}

function Test-Sprint8AComposeResourceExternal {
    param([Parameter(Mandatory)]$Resource)
    $externalProperty = $Resource.PSObject.Properties["external"]
    return $null -ne $externalProperty -and [bool]$externalProperty.Value
}

function Get-Sprint8AArtifact {
    param([Parameter(Mandatory)][string]$Path)
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Required evidence artifact is missing: $fullPath"
    }
    [pscustomobject][ordered]@{
        path = Get-Sprint8ADisplayPath -Path $fullPath
        sha256 = Get-Sprint7AFileSha256 -Path $fullPath
        bytes = [IO.FileInfo]::new($fullPath).Length
    }
}

function Copy-Sprint8ARawEvidence {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination
    )
    $sourcePath = [IO.Path]::GetFullPath($Source)
    $destinationPath = [IO.Path]::GetFullPath($Destination)
    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "Raw evidence source is missing: $sourcePath"
    }
    if ((Test-Path -LiteralPath $destinationPath) -or (Test-Path -LiteralPath "$destinationPath.sha256")) {
        throw "Refusing to replace retained raw evidence: $destinationPath"
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $destinationPath)) | Out-Null
    [IO.File]::WriteAllBytes($destinationPath, [IO.File]::ReadAllBytes($sourcePath))
    $digest = Get-Sprint7AFileSha256 -Path $destinationPath
    [IO.File]::WriteAllText("$destinationPath.sha256", "$digest`n", [Text.UTF8Encoding]::new($false))
    Get-Sprint8AArtifact -Path $destinationPath
}

function Write-Sprint8ARawLines {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Lines,
        [Parameter(Mandatory)][string]$Path
    )
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (Test-Path -LiteralPath $fullPath) {
        throw "Refusing to replace retained raw evidence: $fullPath"
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $fullPath)) | Out-Null
    [IO.File]::WriteAllLines(
        $fullPath,
        @($Lines | ForEach-Object { [string]$_ }),
        [Text.UTF8Encoding]::new($false)
    )
    $digest = Get-Sprint7AFileSha256 -Path $fullPath
    [IO.File]::WriteAllText("$fullPath.sha256", "$digest`n", [Text.UTF8Encoding]::new($false))
    Get-Sprint8AArtifact -Path $fullPath
}

function Get-Sprint8AMaterializationSourceIdentity {
    $commit = (& git -C $repoRoot rev-parse HEAD).Trim()
    $tree = (& git -C $repoRoot rev-parse "HEAD^{tree}").Trim()
    $branch = (& git -C $repoRoot branch --show-current).Trim()
    $status = @(& git -C $repoRoot status --porcelain=v1 --untracked-files=all | ForEach-Object { [string]$_ })
    if ($LASTEXITCODE -ne 0 -or $commit -notmatch '^[0-9a-f]{40}$' -or $tree -notmatch '^[0-9a-f]{40}$') {
        throw "Could not resolve the Sprint 8A source identity."
    }
    [pscustomobject][ordered]@{
        commit = $commit
        tree = $tree
        branch = $branch
        dirty = $status.Count -ne 0
        dirty_paths = $status
    }
}

function Get-Sprint8ADockerState {
    param(
        [Parameter(Mandatory)][string[]]$VolumeNames,
        [Parameter(Mandatory)][string[]]$NetworkNames
    )
    $containerOutput = @(& docker ps -aq --filter "label=com.docker.compose.project=$expectedProject" 2>&1)
    $containerExitCode = $LASTEXITCODE
    if ($containerExitCode -ne 0) { throw "Docker container inventory failed." }
    $containerIds = @($containerOutput | ForEach-Object { ([string]$_).Trim() } | Where-Object { $_ })
    $containers = @($containerIds | ForEach-Object {
        $containerId = $_
        $inspectOutput = @(& docker inspect --type container $containerId 2>&1)
        if ($LASTEXITCODE -ne 0) { throw "Could not inspect Sprint 8A container '$containerId'." }
        $inspect = @(($inspectOutput -join "`n") | ConvertFrom-Json)[0]
        $serviceProperty = $inspect.Config.Labels.PSObject.Properties['com.docker.compose.service']
        [pscustomobject][ordered]@{
            id = [string]$inspect.Id
            name = ([string]$inspect.Name).TrimStart('/')
            service = if ($null -eq $serviceProperty) { $null } else { [string]$serviceProperty.Value }
            image_id = [string]$inspect.Image
            status = [string]$inspect.State.Status
        }
    } | Sort-Object service, name)

    $volumeOutput = @(& docker volume ls --format '{{.Name}}' 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "Docker volume inventory failed." }
    $existingVolumes = @($volumeOutput | ForEach-Object { [string]$_ })
    $presentVolumes = @($VolumeNames | Where-Object { $existingVolumes -ccontains $_ } | Sort-Object)

    $networkOutput = @(& docker network ls --format '{{.Name}}' 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "Docker network inventory failed." }
    $existingNetworks = @($networkOutput | ForEach-Object { [string]$_ })
    $presentNetworks = @($NetworkNames | Where-Object { $existingNetworks -ccontains $_ } | Sort-Object)

    [pscustomobject][ordered]@{
        containers = $containers
        present_volumes = $presentVolumes
        present_networks = $presentNetworks
        empty = $containers.Count -eq 0 -and $presentVolumes.Count -eq 0 -and $presentNetworks.Count -eq 0
    }
}

function Invoke-Sprint8AComposeDown {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string[]]$VolumeNames,
        [Parameter(Mandatory)][string[]]$NetworkNames,
        [Parameter(Mandatory)][string]$RawLogPath,
        [Parameter(Mandatory)][string]$Reason
    )
    $before = Get-Sprint8ADockerState -VolumeNames $VolumeNames -NetworkNames $NetworkNames
    $output = @(& docker compose -f $ComposePath --profile reference down --volumes --remove-orphans 2>&1)
    $exitCode = $LASTEXITCODE
    $rawLog = Write-Sprint8ARawLines -Lines $output -Path $RawLogPath
    $after = Get-Sprint8ADockerState -VolumeNames $VolumeNames -NetworkNames $NetworkNames
    [pscustomobject][ordered]@{
        reason = $Reason
        command = "docker compose -f <resolved-compose> --profile reference down --volumes --remove-orphans"
        exit_code = $exitCode
        raw_log = $rawLog
        before = $before
        after = $after
        passed = $exitCode -eq 0 -and [bool]$after.empty
    }
}

function New-Sprint8AComposeJsonException {
    param(
        [Parameter(Mandatory)][string]$Message,
        [AllowNull()][Exception]$InnerException
    )
    $exception = if ($null -eq $InnerException) {
        [IO.InvalidDataException]::new($Message)
    } else {
        [IO.InvalidDataException]::new($Message, $InnerException)
    }
    $exception.Data["TessaraFailureClassification"] = "harness"
    $exception
}

function ConvertFrom-Sprint8AComposeServiceJson {
    param([AllowNull()][AllowEmptyCollection()][object[]]$Lines = @())

    $text = (@($Lines | ForEach-Object { [string]$_ }) -join "`n").Trim()
    if ([string]::IsNullOrWhiteSpace($text)) { return @() }
    $leadingCharacter = $text.Substring(0, 1)
    if ($leadingCharacter -cne "{" -and $leadingCharacter -cne "[") {
        throw (New-Sprint8AComposeJsonException `
            -Message "Sprint 8A Compose service-state output begins with non-JSON content." `
            -InnerException $null)
    }

    $wholeDocument = $null
    $wholeDocumentError = $null
    $wholeDocumentParsed = $false
    try {
        $wholeDocument = ConvertFrom-Json -InputObject $text -NoEnumerate -ErrorAction Stop
        $wholeDocumentParsed = $true
    } catch {
        $wholeDocumentError = $_
    }
    if ($wholeDocumentParsed) {
        if ($leadingCharacter -ceq "{") {
            if ($wholeDocument -isnot [pscustomobject]) {
                throw (New-Sprint8AComposeJsonException `
                    -Message "Sprint 8A Compose service-state JSON object did not decode as an object." `
                    -InnerException $null)
            }
            return $wholeDocument
        }
        if ($wholeDocument -isnot [Array]) {
            throw (New-Sprint8AComposeJsonException `
                -Message "Sprint 8A Compose service-state JSON array did not decode as an array." `
                -InnerException $null)
        }
        foreach ($record in @($wholeDocument)) {
            if ($record -isnot [pscustomobject]) {
                throw (New-Sprint8AComposeJsonException `
                    -Message "Sprint 8A Compose service-state JSON array contains a non-object value." `
                    -InnerException $null)
            }
            $record
        }
        return
    }

    if ($leadingCharacter -ceq "[") {
        throw (New-Sprint8AComposeJsonException `
            -Message "Sprint 8A Compose service-state JSON array is malformed or followed by additional content." `
            -InnerException $wholeDocumentError.Exception)
    }
    $jsonLines = @($text -split '\r?\n')
    if ($jsonLines.Count -lt 2) {
        throw (New-Sprint8AComposeJsonException `
            -Message "Sprint 8A Compose service-state JSON object is malformed." `
            -InnerException $wholeDocumentError.Exception)
    }
    for ($index = 0; $index -lt $jsonLines.Count; $index++) {
        $line = ([string]$jsonLines[$index]).Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or
            -not $line.StartsWith("{", [StringComparison]::Ordinal)) {
            throw (New-Sprint8AComposeJsonException `
                -Message "Sprint 8A Compose service-state NDJSON line $($index + 1) is not one JSON object." `
                -InnerException $wholeDocumentError.Exception)
        }
        try {
            $record = ConvertFrom-Json -InputObject $line -NoEnumerate -ErrorAction Stop
        } catch {
            throw (New-Sprint8AComposeJsonException `
                -Message "Sprint 8A Compose service-state NDJSON line $($index + 1) is malformed." `
                -InnerException $_.Exception)
        }
        if ($record -isnot [pscustomobject]) {
            throw (New-Sprint8AComposeJsonException `
                -Message "Sprint 8A Compose service-state NDJSON line $($index + 1) is not one JSON object." `
                -InnerException $null)
        }
        $record
    }
}

function ConvertTo-Sprint8AComposeServiceState {
    param([AllowEmptyCollection()][object[]]$Services = @())

    @($Services | ForEach-Object {
        [pscustomobject][ordered]@{
            service = [string]$_.Service
            id = [string]$_.ID
            name = [string]$_.Name
            image = [string]$_.Image
            state = ([string]$_.State).ToLowerInvariant()
            health = ([string]$_.Health).ToLowerInvariant()
            exit_code = if ($null -eq $_.ExitCode) { $null } else { [int]$_.ExitCode }
        }
    } | Sort-Object service, name)
}

function Get-Sprint8AComposeServiceState {
    param([Parameter(Mandatory)][string]$ComposePath)
    $output = @(& docker compose -f $ComposePath --profile reference ps --all --format json 2>&1)
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) { throw "Sprint 8A Compose service-state query failed with exit code $exitCode." }
    $services = @(ConvertFrom-Sprint8AComposeServiceJson -Lines $output)
    @(ConvertTo-Sprint8AComposeServiceState -Services $services)
}

function Get-Sprint8AMaterializationFailureClassification {
    param(
        [Parameter(Mandatory)][Management.Automation.ErrorRecord]$ErrorRecord,
        [AllowNull()]$ExpectedFaultObserved
    )
    if ($ExpectedFaultObserved -eq $true) { return "harness" }

    $exception = $ErrorRecord.Exception
    while ($null -ne $exception) {
        if ($exception.Data.Contains("TessaraFailureClassification")) {
            $declared = [string]$exception.Data["TessaraFailureClassification"]
            if ($declared -in @("product", "harness", "environment")) { return $declared }
        }
        $exception = $exception.InnerException
    }
    $exceptionType = $ErrorRecord.Exception.GetType().FullName
    if ($exceptionType -match '^System\.Management\.Automation\.(PropertyNotFoundException|ParameterBindingException|CommandNotFoundException|ParseException)$') {
        return "harness"
    }
    if ($ErrorRecord.Exception.Message -match '(?i)Docker daemon is not running|Cannot connect to the Docker daemon|connection refused|timed out while waiting for .* health') {
        return "environment"
    }
    "product"
}

function Assert-Sprint8AReceipt {
    param(
        [Parameter(Mandatory)]$ApplyResponse,
        [Parameter(Mandatory)][bool]$ExpectedNoOp,
        [Parameter(Mandatory)][bool]$ExpectedChanged,
        [Parameter(Mandatory)]$Blueprint
    )
    if ($null -eq $ApplyResponse.receipt -or $null -eq $ApplyResponse.operation) {
        throw "Sprint 8A apply response is missing its operation or installation receipt."
    }
    if ([string]$ApplyResponse.operation.state -cne "succeeded") {
        throw "Sprint 8A apply operation did not report succeeded."
    }
    $receipt = $ApplyResponse.receipt
    if ([bool]$receipt.no_op -ne $ExpectedNoOp) {
        throw "Sprint 8A apply response reported an unexpected no-op state."
    }
    $ownerReceipts = @($receipt.bootstrap_receipts)
    $actualOwnerOrder = @($ownerReceipts | ForEach-Object { [string]$_.owner })
    if (($actualOwnerOrder -join "`n") -cne ($expectedOwnerOrder -join "`n")) {
        throw "Sprint 8A owner bootstrap order differs from the exact canonical owner order."
    }
    if (@($ownerReceipts | Where-Object { [bool]$_.changed -ne $ExpectedChanged }).Count -ne 0) {
        throw "Sprint 8A owner bootstrap changed flags differ from the expected apply semantics."
    }

    $componentReceipt = @($ownerReceipts | Where-Object owner -CEQ "tessara.components")
    if ($componentReceipt.Count -ne 1) { throw "Component owner bootstrap receipt is missing or duplicated." }
    $expectedComponentKeys = @(
        "sprint-8a-record-table", "sprint-8a-label-bar", "sprint-8a-label-line",
        "sprint-8a-label-pie", "sprint-8a-label-donut", "sprint-8a-row-count-inactive",
        "sprint-8a-row-count", "sprint-8a-blocked-component"
    ) | Sort-Object
    $actualComponentKeys = @($componentReceipt[0].resource_ids.PSObject.Properties.Name | Sort-Object)
    if (($actualComponentKeys -join "`n") -cne ($expectedComponentKeys -join "`n")) {
        throw "Component owner bootstrap did not return the exact canonical Component identities."
    }
    foreach ($resource in $componentReceipt[0].resource_ids.PSObject.Properties) {
        $reference = ([string]$resource.Value) | ConvertFrom-Json
        if ([string]$reference.reference.installation_id -cne $installationId -or
            [string]$reference.reference.owner.kind -cne "module_instance" -or
            [string]$reference.reference.owner.installation_id -cne $installationId -or
            [string]$reference.reference.owner.module_instance_id -cne $componentOwner -or
            [string]$reference.reference.resource_type -cne "tessara.components.component_version") {
            throw "Component read-back '$($resource.Name)' is not a Sprint 8A v3 module-instance reference."
        }
    }

    $dashboardReceipt = @($ownerReceipts | Where-Object owner -CEQ "tessara.dashboards")
    if ($dashboardReceipt.Count -ne 1) { throw "Dashboard owner bootstrap receipt is missing or duplicated." }
    $dashboardModules = @($Blueprint.modules | Where-Object definition_id -CEQ "tessara.dashboards")
    if ($dashboardModules.Count -ne 1 -or $null -eq $dashboardModules[0].bootstrap.value) {
        throw "Sprint 8A Blueprint must contain one inline Dashboard bootstrap."
    }
    $dashboardBootstrap = $dashboardModules[0].bootstrap
    $dashboardPlacements = @($dashboardBootstrap.value.placements)
    if (@($dashboardPlacements | Where-Object { $null -ne $_.component_reference }).Count -ne 0) {
        throw "Dashboard canonical bootstrap must leave Component references for receipt binding."
    }
    $expectedBindings = [ordered]@{
        "/placements/0/component_reference" = "sprint-8a-row-count"
        "/placements/1/component_reference" = "sprint-8a-record-table"
        "/placements/2/component_reference" = "sprint-8a-label-bar"
        "/placements/3/component_reference" = "sprint-8a-blocked-component"
        "/placements/4/component_reference" = "sprint-8a-row-count-inactive"
        "/placements/5/component_reference" = "sprint-8a-row-count-inactive"
        "/placements/6/component_reference" = "sprint-8a-row-count-inactive"
    }
    $receiptBindings = @($dashboardBootstrap.receipt_bindings)
    if ($receiptBindings.Count -ne $expectedBindings.Count) {
        throw "Dashboard canonical bootstrap does not declare the exact Component receipt-binding set."
    }
    foreach ($targetPointer in $expectedBindings.Keys) {
        $binding = @($receiptBindings | Where-Object target_pointer -CEQ $targetPointer)
        $resourceKey = [string]$expectedBindings[$targetPointer]
        if ($binding.Count -ne 1 -or
            [string]$binding[0].source_owner -cne "tessara.components" -or
            [string]$binding[0].resource_key -cne $resourceKey -or
            [string]$binding[0].value_encoding -cne "json" -or
            -not ($actualComponentKeys -ccontains $resourceKey)) {
            throw "Dashboard receipt binding '$targetPointer' does not use exact Component owner read-back."
        }
    }

    [pscustomobject][ordered]@{
        operation_id = [string]$ApplyResponse.operation.operation_id
        operation_state = [string]$ApplyResponse.operation.state
        receipt_digest = [string]$ApplyResponse.operation.receipt_digest
        lockfile_digest = [string]$receipt.lockfile_digest
        plan_digest = [string]$receipt.plan_digest
        revision = [uint64]$receipt.revision
        previous_receipt_digest = if ($null -eq $receipt.previous_receipt_digest) {
            $null
        } else {
            [string]$receipt.previous_receipt_digest
        }
        no_op = [bool]$receipt.no_op
        owner_order = $actualOwnerOrder
        owner_receipts = $ownerReceipts
        desired_enablement = $receipt.desired_enablement
        observed_enablement = $receipt.observed_enablement
        observed_artifacts = $receipt.observed_artifacts
        configuration_digests = $receipt.configuration_digests
    }
}

function Assert-Sprint8ANoOpMatchesFirst {
    param(
        [Parameter(Mandatory)]$First,
        [Parameter(Mandatory)]$NoOp
    )
    if ($null -ne $First.previous_receipt_digest -or
        [string]$NoOp.previous_receipt_digest -cne [string]$First.receipt_digest -or
        [uint64]$NoOp.revision -ne ([uint64]$First.revision + 1)) {
        throw "Sprint 8A no-op receipt chain does not bind the exact from-empty first apply."
    }
    foreach ($field in @('desired_enablement', 'observed_enablement', 'observed_artifacts', 'configuration_digests')) {
        if (($First.$field | ConvertTo-Json -Depth 30 -Compress) -cne
            ($NoOp.$field | ConvertTo-Json -Depth 30 -Compress)) {
            throw "Sprint 8A semantic no-op changed carried-forward receipt field '$field'."
        }
    }
    $firstByOwner = @{}
    foreach ($ownerReceipt in @($First.owner_receipts)) {
        $firstByOwner[[string]$ownerReceipt.owner] = [ordered]@{
            input_digest = [string]$ownerReceipt.input_digest
            result_digest = [string]$ownerReceipt.result_digest
            resource_ids = ($ownerReceipt.resource_ids | ConvertTo-Json -Depth 30 -Compress)
        }
    }
    foreach ($ownerReceipt in @($NoOp.owner_receipts)) {
        $firstOwner = $firstByOwner[[string]$ownerReceipt.owner]
        if ($null -eq $firstOwner -or
            [string]$ownerReceipt.input_digest -cne $firstOwner.input_digest -or
            [string]$ownerReceipt.result_digest -cne $firstOwner.result_digest -or
            ($ownerReceipt.resource_ids | ConvertTo-Json -Depth 30 -Compress) -cne $firstOwner.resource_ids) {
            throw "Sprint 8A no-op owner receipt '$($ownerReceipt.owner)' differs from first-apply read-back."
        }
    }
}

function Get-Sprint8AFinalHealth {
    param([Parameter(Mandatory)][string]$ComposePath)
    $gatewayReady = Invoke-Sprint8AHealthProbe -Target gateway_core -BaseUrl $CoreUrl
    $supervisorReady = Invoke-Sprint8AHealthProbe -Target supervisor -BaseUrl $SupervisorUrl
    $services = @(Get-Sprint8AComposeServiceState -ComposePath $ComposePath)
    $expectedRuntimeServices = @(
        "components", "core", "dashboards", "gateway", "postgres", "scoped-records", "supervisor"
    )
    $failures = [Collections.Generic.List[string]]::new()
    if (-not [bool]$gatewayReady.passed) {
        $failures.Add("gateway_core_health_contract_failed")
    }
    if (-not [bool]$supervisorReady.passed) {
        $failures.Add("supervisor_health_contract_failed")
    }
    foreach ($serviceName in $expectedRuntimeServices) {
        $service = @($services | Where-Object service -CEQ $serviceName)
        if ($service.Count -ne 1 -or $service[0].state -cne "running" -or
            ($service[0].health -and $service[0].health -cne "healthy")) {
            $failures.Add("runtime_service_not_healthy:$serviceName")
        }
    }
    [pscustomobject][ordered]@{
        captured_at = [DateTimeOffset]::UtcNow.ToString("o")
        health_contract = "tessara.sprint-8a.health-observation/v1"
        gateway_core = $gatewayReady
        supervisor = $supervisorReady
        expected_runtime_services = $expectedRuntimeServices
        compose_services = $services
        failures = @($failures)
        passed = $failures.Count -eq 0
    }
}

function Start-Sprint8APublicGateway {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$RawLogPath
    )

    $servicesBefore = @(Get-Sprint8AComposeServiceState -ComposePath $ComposePath)
    $gatewayBefore = @($servicesBefore | Where-Object service -CEQ "gateway")
    if (@($gatewayBefore | Where-Object state -CEQ "running").Count -ne 0) {
        throw "Sprint 8A public gateway became available before owner materialization completed."
    }
    $probeBeforeObservation = Invoke-Sprint8AHealthProbe -Target gateway_core -BaseUrl $CoreUrl
    $probeBefore = [ordered]@{
        response_received = [bool]$probeBeforeObservation.response.received
        unavailable_proven = -not [bool]$probeBeforeObservation.response.received
        observed_at = [DateTimeOffset]::UtcNow.ToString("o")
        observation = $probeBeforeObservation
    }
    if ([bool]$probeBefore.response_received) {
        throw "Sprint 8A public gateway endpoint responded before owner materialization completed."
    }

    $offlineUntil = [DateTimeOffset]::UtcNow
    $output = @(& docker compose -f $ComposePath --profile reference up -d --no-build gateway 2>&1)
    $exitCode = $LASTEXITCODE
    $rawLog = Write-Sprint8ARawLines -Lines $output -Path $RawLogPath
    if ($exitCode -ne 0) {
        throw "Sprint 8A public gateway startup failed after owner materialization."
    }

    $ready = Wait-Sprint8AHealthProbe `
        -Target gateway_core `
        -BaseUrl $CoreUrl `
        -MaximumAttempts 60 `
        -DelaySeconds 1
    $servicesAfter = @(Get-Sprint8AComposeServiceState -ComposePath $ComposePath)
    $gatewayAfter = @($servicesAfter | Where-Object service -CEQ "gateway")
    $failures = [Collections.Generic.List[string]]::new()
    if (-not [bool]$ready.passed) {
        $failures.Add("public_gateway_health_contract_failed")
    }
    if ($gatewayAfter.Count -ne 1 -or $gatewayAfter[0].state -cne "running") {
        $failures.Add("gateway_service_not_uniquely_running")
    }

    [ordered]@{
        schema_version = 2
        contract = "tessara.sprint-8a.public-gateway-boundary"
        public_gateway_url = $CoreUrl
        materialization_control_url = $ControlUrl
        gateway_service_before = $gatewayBefore
        public_probe_before = $probeBefore
        owner_materialization_completed_at = $offlineUntil.ToString("o")
        start = [ordered]@{
            command = "docker compose -f <resolved-compose> --profile reference up -d --no-build gateway"
            exit_code = $exitCode
            raw_log = $rawLog
        }
        public_ready_at = if ([bool]$ready.passed) { [DateTimeOffset]::UtcNow.ToString("o") } else { $null }
        public_ready_attempts = [int]$ready.attempts
        public_ready = $ready.observation
        gateway_service_after = $gatewayAfter
        failures = @($failures)
        passed = $failures.Count -eq 0
    }
}

function Assert-Sprint8AComposeParserRejects {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Lines,
        [Parameter(Mandatory)][string]$ExpectedFragment
    )
    $rejected = $false
    try {
        ConvertFrom-Sprint8AComposeServiceJson -Lines $Lines | Out-Null
    } catch {
        $rejected = $true
        if (-not $_.Exception.Message.Contains($ExpectedFragment)) {
            throw "Compose parser self-test caught the wrong failure: $($_.Exception.Message)"
        }
        if ((Get-Sprint8AMaterializationFailureClassification -ErrorRecord $_ -ExpectedFaultObserved $null) -cne "harness") {
            throw "Compose parser self-test did not classify malformed Compose output as a harness defect."
        }
    }
    if (-not $rejected) {
        throw "Compose parser self-test accepted invalid output expected to contain '$ExpectedFragment'."
    }
}

function Invoke-Sprint8AMaterializationSelfTest {
    Test-Sprint8AHealthContract | Out-Null
    if (@(ConvertFrom-Sprint8AComposeServiceJson -Lines @()).Count -ne 0 -or
        @(ConvertFrom-Sprint8AComposeServiceJson -Lines @('[]')).Count -ne 0) {
        throw "Compose parser self-test did not preserve empty output and an empty JSON array."
    }

    $singleObject = @(ConvertFrom-Sprint8AComposeServiceJson -Lines @(
        '{"Service":"core","ID":"1","Name":"core-1","Image":"core","State":"running","Health":"healthy","ExitCode":0}'
    ))
    if ($singleObject.Count -ne 1 -or [string]$singleObject[0].Service -cne "core") {
        throw "Compose parser self-test did not accept one JSON object."
    }

    $arrayRecords = @(ConvertFrom-Sprint8AComposeServiceJson -Lines @(
        '[{"Service":"zeta","ID":"3","Name":"zeta-1","Image":"zeta","State":"running","Health":"healthy","ExitCode":0},{"Service":"alpha","ID":"2","Name":"alpha-z","Image":"alpha","State":"running","Health":"healthy","ExitCode":0}]'
    ))
    if ($arrayRecords.Count -ne 2) {
        throw "Compose parser self-test did not accept a JSON object array."
    }

    $ndjsonRecords = @(ConvertFrom-Sprint8AComposeServiceJson -Lines @(
        '{"Service":"alpha","ID":"2","Name":"alpha-z","Image":"alpha","State":"running","Health":"healthy","ExitCode":0}',
        '{"Service":"alpha","ID":"1","Name":"alpha-a","Image":"alpha","State":"running","Health":"healthy","ExitCode":0}',
        '{"Service":"zeta","ID":"3","Name":"zeta-1","Image":"zeta","State":"running","Health":"healthy","ExitCode":0}'
    ))
    $sortedState = @(ConvertTo-Sprint8AComposeServiceState -Services $ndjsonRecords)
    if ($ndjsonRecords.Count -ne 3 -or
        (@($sortedState | ForEach-Object { "$($_.service)/$($_.name)" }) -join "`n") -cne
        "alpha/alpha-a`nalpha/alpha-z`nzeta/zeta-1") {
        throw "Compose parser self-test did not accept NDJSON or produce deterministic service ordering."
    }

    Assert-Sprint8AComposeParserRejects `
        -Lines @('{"Service":"alpha"}', '{not-json}') `
        -ExpectedFragment "NDJSON line 2 is malformed"
    Assert-Sprint8AComposeParserRejects `
        -Lines @('{"Service":"alpha"}', 'warning: daemon state changed') `
        -ExpectedFragment "NDJSON line 2 is not one JSON object"
    Assert-Sprint8AComposeParserRejects `
        -Lines @('warning: daemon state changed', '{"Service":"alpha"}') `
        -ExpectedFragment "begins with non-JSON content"
    Assert-Sprint8AComposeParserRejects `
        -Lines @('[{"Service":"alpha"},"warning"]') `
        -ExpectedFragment "contains a non-object value"

    $productError = $null
    try { throw [InvalidOperationException]::new("ordinary product failure") } catch { $productError = $_ }
    if ((Get-Sprint8AMaterializationFailureClassification -ErrorRecord $productError -ExpectedFaultObserved $false) -cne "product" -or
        (Get-Sprint8AMaterializationFailureClassification -ErrorRecord $productError -ExpectedFaultObserved $true) -cne "harness") {
        throw "Materialization failure-classification self-test did not distinguish product and injected-fault failures."
    }
    $environmentError = $null
    try { throw [IO.IOException]::new("Cannot connect to the Docker daemon") } catch { $environmentError = $_ }
    if ((Get-Sprint8AMaterializationFailureClassification -ErrorRecord $environmentError -ExpectedFaultObserved $false) -cne "environment") {
        throw "Materialization failure-classification self-test did not retain an environment failure."
    }

    Write-Host "Sprint 8A strict Compose service-state parser and failure-classification self-test passed."
    [pscustomobject][ordered]@{
        contract = "tessara.sprint-8a.materialization-self-test"
        passed = $true
    }
}

if ($SelfTest) {
    Invoke-Sprint8AMaterializationSelfTest
    return
}

if (-not $AuthorizeDisposableReset) {
    throw "Sprint 8A materialization is destructive. Re-run with -AuthorizeDisposableReset only for the disposable tessara-sprint-8a project."
}
Assert-Sprint8ADestructiveEndpoint -Value $CoreUrl -Name "CoreUrl" -ExpectedPort 8088
Assert-Sprint8ADestructiveEndpoint -Value $ControlUrl -Name "ControlUrl" -ExpectedPort 18088
Assert-Sprint8ADestructiveEndpoint -Value $SupervisorUrl -Name "SupervisorUrl" -ExpectedPort 8098
if (-not [string]::IsNullOrWhiteSpace($EnvironmentFingerprint) -and
    $EnvironmentFingerprint -notmatch '^[0-9a-fA-F]{64}$') {
    throw "EnvironmentFingerprint must be one SHA-256 value."
}
if ([string]::IsNullOrWhiteSpace($ExpectedFaultId) -xor [string]::IsNullOrWhiteSpace($ExpectedFailurePattern)) {
    throw "ExpectedFaultId and ExpectedFailurePattern must be supplied together."
}

$composePath = Resolve-Sprint8ARepositoryPath -Path $ComposeFile
$resolvedBlueprintPath = Resolve-Sprint8ARepositoryPath -Path $BlueprintPath
$resolvedEvidenceRoot = Resolve-Sprint8ARepositoryPath -Path $EvidenceRoot
$attemptDirectory = Join-Path $resolvedEvidenceRoot "materialization/attempt-$Attempt"
$runtimeDirectory = Join-Path $attemptDirectory "runtime"
$summaryPath = Join-Path $attemptDirectory "materialization-result.json"
$failurePath = Join-Path $attemptDirectory "materialization-failure.json"
$receiptPath = Join-Path $runtimeDirectory "apply-response.json"

if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) { throw "Compose file not found: $composePath" }
if (-not (Test-Path -LiteralPath $resolvedBlueprintPath -PathType Leaf)) { throw "Blueprint not found: $resolvedBlueprintPath" }

Push-Location $repoRoot
try {
    $configurationOutput = @(& docker compose -f $composePath --profile reference config --format json 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "Sprint 8A Compose configuration is invalid." }
    $configurationText = ($configurationOutput -join "`n")
    $configuration = $configurationText | ConvertFrom-Json
    if ([string]$configuration.name -cne $expectedProject) {
        throw "Refusing destructive reset for unexpected Compose project '$($configuration.name)'."
    }

    $expectedPostgresInit = [IO.Path]::GetFullPath((Join-Path $repoRoot "deploy/sprint-8a/postgres-init.sh"))
    $postgresInitMounts = @($configuration.services.postgres.volumes | Where-Object {
        $_.target -ceq "/docker-entrypoint-initdb.d/10-tessara-databases.sh"
    })
    if ($postgresInitMounts.Count -ne 1 -or
        [IO.Path]::GetFullPath([string]$postgresInitMounts[0].source) -cne $expectedPostgresInit) {
        throw "Refusing Sprint 8A materialization because PostgreSQL is not bound to the exact Sprint 8A initialization contract."
    }

    $unexpectedVolumes = @($configuration.volumes.PSObject.Properties | Where-Object {
        $configuredName = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_.Value -Name "name"
        $name = if ([string]::IsNullOrWhiteSpace([string]$configuredName)) { "${expectedProject}_$($_.Name)" } else { [string]$configuredName }
        -not $name.StartsWith("$expectedProject`_", [StringComparison]::Ordinal)
    })
    if ($unexpectedVolumes.Count -gt 0) {
        throw "Refusing reset because Compose resolves a named volume outside the $expectedProject namespace."
    }
    $unexpectedNetworks = @($configuration.networks.PSObject.Properties | Where-Object {
        $configuredName = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_.Value -Name "name"
        $name = if ([string]::IsNullOrWhiteSpace([string]$configuredName)) { "${expectedProject}_$($_.Name)" } else { [string]$configuredName }
        -not (Test-Sprint8AComposeResourceExternal -Resource $_.Value) -and
            -not $name.StartsWith("$expectedProject`_", [StringComparison]::Ordinal)
    })
    if ($unexpectedNetworks.Count -gt 0) {
        throw "Refusing reset because Compose resolves a non-external named network outside the $expectedProject namespace."
    }
    $resolvedVolumeNames = @($configuration.volumes.PSObject.Properties | ForEach-Object {
        $configuredName = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_.Value -Name "name"
        if ([string]::IsNullOrWhiteSpace([string]$configuredName)) { "${expectedProject}_$($_.Name)" } else { [string]$configuredName }
    } | Sort-Object)
    $resolvedNetworkNames = @($configuration.networks.PSObject.Properties | Where-Object {
        -not (Test-Sprint8AComposeResourceExternal -Resource $_.Value)
    } | ForEach-Object {
        $configuredName = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_.Value -Name "name"
        if ([string]::IsNullOrWhiteSpace([string]$configuredName)) { "${expectedProject}_$($_.Name)" } else { [string]$configuredName }
    } | Sort-Object)

    $source = Get-Sprint8AMaterializationSourceIdentity
    if ($source.dirty) {
        throw "Sprint 8A source-exact materialization requires a clean Git worktree."
    }

    $serviceProjection = Get-Sprint8AComposeServiceProjection -Services $configuration.services
    $ports = @($serviceProjection.ports)
    $databaseBindings = @($serviceProjection.database_bindings)
    $expectedDatabaseNames = @(
        "tessara_core",
        "tessara_deployment",
        "tessara_module_components",
        "tessara_module_dashboards",
        "tessara_module_scoped_records"
    )
    $postgresInitText = Get-Content -LiteralPath $expectedPostgresInit -Raw
    $createdDatabaseNames = @([regex]::Matches(
        $postgresInitText,
        '(?m)^\s*CREATE DATABASE ([a-z][a-z0-9_]*) OWNER [a-z][a-z0-9_]*;\s*$'
    ) | ForEach-Object { [string]$_.Groups[1].Value } | Sort-Object -Unique)
    $runtimeDatabaseNames = @($databaseBindings.database | Sort-Object -Unique)
    $expectedRuntimeDatabaseNames = @($expectedDatabaseNames | Where-Object { $_ -cne "tessara_deployment" } | Sort-Object)
    if (($createdDatabaseNames -join "`n") -cne (($expectedDatabaseNames | Sort-Object) -join "`n") -or
        ($runtimeDatabaseNames -join "`n") -cne ($expectedRuntimeDatabaseNames -join "`n") -or
        @($databaseBindings | Where-Object host -CNE "postgres").Count -ne 0) {
        throw "Sprint 8A initialization and runtime bindings do not resolve to the exact disposable PostgreSQL database set."
    }
    $postgresDataMount = @($configuration.services.postgres.volumes | Where-Object {
        $_.type -ceq "volume" -and $_.target -ceq "/var/lib/postgresql/data"
    })
    if ($postgresDataMount.Count -ne 1) {
        throw "Sprint 8A PostgreSQL data does not resolve to one named-volume target."
    }
    $postgresVolumeProperty = $configuration.volumes.PSObject.Properties[[string]$postgresDataMount[0].source]
    if ($null -eq $postgresVolumeProperty) {
        throw "Sprint 8A PostgreSQL data volume is absent from normalized Compose targets."
    }
    $configuredPostgresVolumeName = Get-Sprint8AOptionalObjectPropertyValue `
        -InputObject $postgresVolumeProperty.Value `
        -Name "name"
    $postgresVolumeName = if (-not [string]::IsNullOrWhiteSpace([string]$configuredPostgresVolumeName)) {
        [string]$configuredPostgresVolumeName
    } else {
        "${expectedProject}_$([string]$postgresDataMount[0].source)"
    }
    if ($resolvedVolumeNames -cnotcontains $postgresVolumeName) {
        throw "Sprint 8A PostgreSQL data volume is outside the exact destructive target set."
    }
    $serviceTargets = @($configuration.services.PSObject.Properties.Name | Sort-Object)
    $stateBeforeReset = Get-Sprint8ADockerState -VolumeNames $resolvedVolumeNames -NetworkNames $resolvedNetworkNames
    $materializationEnvironment = [ordered]@{
        project = $expectedProject
        profile = "reference"
        installation_id = $installationId
        compose_sha256 = Get-Sprint7AFileSha256 -Path $composePath
        normalized_compose_sha256 = Get-Sprint7ASha256 -Text $configurationText
        postgres_init_sha256 = Get-Sprint7AFileSha256 -Path $expectedPostgresInit
        core_url = $CoreUrl
        control_url = $ControlUrl
        supervisor_url = $SupervisorUrl
        services = $serviceTargets
        volumes = $resolvedVolumeNames
        networks = $resolvedNetworkNames
        ports = $ports
        databases = $createdDatabaseNames
        postgres_data_volume = $postgresVolumeName
    }
    $observedEnvironmentFingerprint = Get-Sprint7ASha256 -Text ($materializationEnvironment | ConvertTo-Json -Depth 20 -Compress)
    $targets = [ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.materialization-targets"
        attempt = $Attempt
        source = $source
        environment = [ordered]@{
            declared_fingerprint = if ($EnvironmentFingerprint) { $EnvironmentFingerprint.ToLowerInvariant() } else { $null }
            materialization_fingerprint = $observedEnvironmentFingerprint
            contract = $materializationEnvironment
        }
        invocation = [ordered]@{
            compose = Get-Sprint8AArtifact -Path $composePath
            blueprint = Get-Sprint8AArtifact -Path $resolvedBlueprintPath
            evidence_root = Get-Sprint8ADisplayPath -Path $resolvedEvidenceRoot
            attempt_directory = Get-Sprint8ADisplayPath -Path $attemptDirectory
            verify_no_op = [bool]$VerifyNoOp
            skip_build = [bool]$SkipBuild
            expected_fault_id = if ($ExpectedFaultId) { $ExpectedFaultId } else { $null }
        }
        destructive_targets = [ordered]@{
            project = $expectedProject
            services = $serviceTargets
            containers = $stateBeforeReset.containers
            named_volumes = $resolvedVolumeNames
            named_networks = $resolvedNetworkNames
            database_bindings = $databaseBindings
            ports = $ports
        }
    }

    if (-not $PSCmdlet.ShouldProcess(
        "$expectedProject containers and exact named volumes: $($resolvedVolumeNames -join ', ')",
        "Fresh-materialize Sprint 8A from an independently proven empty baseline"
    )) {
        Write-Host "Planned Sprint 8A evidence directory: $attemptDirectory"
        Write-Output ([pscustomobject][ordered]@{
            contract = "tessara.sprint-8a.materialization-plan"
            attempt = $Attempt
            evidence_directory = $attemptDirectory
            summary_path = $summaryPath
        })
        return
    }
    if ([string]::IsNullOrWhiteSpace($EnvironmentFingerprint)) {
        throw "Live Sprint 8A materialization requires -EnvironmentFingerprint from Validation Readiness."
    }
    if (Test-Path -LiteralPath $attemptDirectory) {
        throw "Refusing to replace retained Sprint 8A attempt evidence: $attemptDirectory"
    }
    [IO.Directory]::CreateDirectory($attemptDirectory) | Out-Null
    $targetPublication = Publish-Sprint7AEvidence -Document $targets -OutputPath (Join-Path $attemptDirectory "resolved-targets.json")

    try {
        $baselineTeardown = Invoke-Sprint8AComposeDown `
            -ComposePath $composePath `
            -VolumeNames $resolvedVolumeNames `
            -NetworkNames $resolvedNetworkNames `
            -RawLogPath (Join-Path $attemptDirectory "initial-reset.log") `
            -Reason "authorized_preproduction_from_empty_reset"
        if (-not $baselineTeardown.passed) {
            throw "Sprint 8A authorized reset did not establish an empty topology baseline."
        }
        $emptyBaseline = [ordered]@{
            schema_version = 1
            contract = "tessara.sprint-8a.empty-baseline"
            attempt = $Attempt
            source = $source
            environment_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
            captured_at = [DateTimeOffset]::UtcNow.ToString("o")
            resolved_targets_sha256 = [string]$targetPublication.sha256
            teardown = $baselineTeardown
            database_baseline = [ordered]@{
                names = $createdDatabaseNames
                postgres_data_volume = $postgresVolumeName
                proof = "all database storage is absent because every resolved database-bearing named volume is absent"
                named_volumes_absent = $baselineTeardown.after.present_volumes.Count -eq 0
            }
            empty = [bool]$baselineTeardown.after.empty
        }
        $emptyBaselinePublication = Publish-Sprint7AEvidence `
            -Document $emptyBaseline `
            -OutputPath (Join-Path $attemptDirectory "empty-baseline.json")

        & (Join-Path $PSScriptRoot "bootstrap-sprint-7a-composition.ps1") `
            -Composition reference `
            -ComposeFile $composePath `
            -BlueprintPath $resolvedBlueprintPath `
            -RuntimeDirectory $runtimeDirectory `
            -CoreUrl $ControlUrl `
            -SupervisorUrl $SupervisorUrl `
            -DeploymentDirectory "sprint-8a" `
            -ExpectedProject $expectedProject `
            -InstallationId $installationId `
            -RuntimeLabel "sprint-8a" `
            -AdditionalBuildServices @("components") `
            -AdditionalExpectedNavigationHrefs @("/components") `
            -ExcludePublicGateway `
            -SkipBuild:$SkipBuild `
            -Confirm:$false

        $firstRaw = Copy-Sprint8ARawEvidence `
            -Source $receiptPath `
            -Destination (Join-Path $attemptDirectory "first-apply-response.json")
        $blueprint = Get-Content -LiteralPath $resolvedBlueprintPath -Raw | ConvertFrom-Json
        $firstApplyResponse = Get-Content -LiteralPath $firstRaw.path -Raw | ConvertFrom-Json
        $first = Assert-Sprint8AReceipt `
            -ApplyResponse $firstApplyResponse `
            -ExpectedNoOp $false `
            -ExpectedChanged $true `
            -Blueprint $blueprint

        $noOp = $null
        $noOpRaw = $null
        if ($VerifyNoOp) {
            $firstCatalogEnvelope = Join-Path $runtimeDirectory "release-catalog.signed.json"
            if (-not (Test-Path -LiteralPath $firstCatalogEnvelope -PathType Leaf)) {
                throw "Sprint 8A first apply did not retain its exact signed release catalog for no-op verification."
            }
            & (Join-Path $PSScriptRoot "bootstrap-sprint-7a-composition.ps1") `
                -Composition reference `
                -ComposeFile $composePath `
                -BlueprintPath $resolvedBlueprintPath `
                -RuntimeDirectory $runtimeDirectory `
                -ReleaseCatalogEnvelope $firstCatalogEnvelope `
                -CoreUrl $ControlUrl `
                -SupervisorUrl $SupervisorUrl `
                -DeploymentDirectory "sprint-8a" `
                -ExpectedProject $expectedProject `
                -InstallationId $installationId `
                -RuntimeLabel "sprint-8a" `
                -AdditionalBuildServices @("components") `
                -AdditionalExpectedNavigationHrefs @("/components") `
                -ExcludePublicGateway `
                -SemanticNoOp `
                -SkipBuild `
                -Confirm:$false
            $noOpRaw = Copy-Sprint8ARawEvidence `
                -Source $receiptPath `
                -Destination (Join-Path $attemptDirectory "no-op-apply-response.json")
            $noOpLockfile = Get-Content -LiteralPath (Join-Path $runtimeDirectory "lockfile.json") -Raw | ConvertFrom-Json
            $noOpAuthorization = Get-Content -LiteralPath (Join-Path $runtimeDirectory "authorization.json") -Raw | ConvertFrom-Json
            $noOpActions = @($noOpLockfile.materialization_plan.actions)
            if ($noOpActions.Count -ne 1 -or [string]$noOpActions[0].action -cne "verify_read_back" -or
                @($noOpAuthorization.approved_effects).Count -ne 0) {
                throw "Sprint 8A unchanged desired state must use only read-back with zero approved effects."
            }
            $noOpApplyResponse = Get-Content -LiteralPath $noOpRaw.path -Raw | ConvertFrom-Json
            $noOp = Assert-Sprint8AReceipt `
                -ApplyResponse $noOpApplyResponse `
                -ExpectedNoOp $true `
                -ExpectedChanged $false `
                -Blueprint $blueprint
            Assert-Sprint8ANoOpMatchesFirst -First $first -NoOp $noOp
        }

        $gatewayBoundary = Start-Sprint8APublicGateway `
            -ComposePath $composePath `
            -RawLogPath (Join-Path $attemptDirectory "public-gateway-start.log")
        $gatewayBoundaryPublication = Publish-Sprint7AEvidence `
            -Document $gatewayBoundary `
            -OutputPath (Join-Path $attemptDirectory "public-gateway-boundary.json")
        if (-not [bool]$gatewayBoundary.passed) {
            if (-not [bool]$gatewayBoundary.public_ready.passed) {
                Assert-Sprint8AHealthPassed `
                    -Observation $gatewayBoundary.public_ready `
                    -Context "Sprint 8A public gateway/Core health contract"
            }
            throw "Sprint 8A public gateway is not uniquely running after the offline boundary opened."
        }

        $finalHealth = Get-Sprint8AFinalHealth -ComposePath $composePath
        $finalHealthPublication = Publish-Sprint7AEvidence `
            -Document ([ordered]@{
                schema_version = 2
                contract = "tessara.sprint-8a.final-health"
                attempt = $Attempt
                source = $source
                environment_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
                preceding_apply_response = if ($null -ne $noOpRaw) { $noOpRaw } else { $firstRaw }
                preceding_apply_no_op = $null -ne $noOp -and [bool]$noOp.no_op
                health = $finalHealth
            }) `
            -OutputPath (Join-Path $attemptDirectory "final-health.json")
        if (-not [bool]$finalHealth.passed) {
            if (-not [bool]$finalHealth.gateway_core.passed) {
                Assert-Sprint8AHealthPassed `
                    -Observation $finalHealth.gateway_core `
                    -Context "Sprint 8A final gateway/Core health contract"
            }
            if (-not [bool]$finalHealth.supervisor.passed) {
                Assert-Sprint8AHealthPassed `
                    -Observation $finalHealth.supervisor `
                    -Context "Sprint 8A final Supervisor health contract"
            }
            throw "Sprint 8A final runtime services are not uniquely running and healthy: $(@($finalHealth.failures) -join ', ')."
        }

        $result = [ordered]@{
            schema_version = 1
            contract = "tessara.sprint-8a.materialization-result"
            attempt = $Attempt
            passed = $true
            completed_at = [DateTimeOffset]::UtcNow.ToString("o")
            source = $source
            environment = [ordered]@{
                declared_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
                materialization_fingerprint = $observedEnvironmentFingerprint
            }
            inputs = [ordered]@{
                compose = Get-Sprint8AArtifact -Path $composePath
                blueprint = Get-Sprint8AArtifact -Path $resolvedBlueprintPath
            }
            evidence = [ordered]@{
                resolved_targets = Get-Sprint8AArtifact -Path $targetPublication.path
                empty_baseline = Get-Sprint8AArtifact -Path $emptyBaselinePublication.path
                signed_release_catalog = Get-Sprint8AArtifact -Path (Join-Path $runtimeDirectory "release-catalog.signed.json")
                lockfile = Get-Sprint8AArtifact -Path (Join-Path $runtimeDirectory "lockfile.json")
                first_apply_response = $firstRaw
                no_op_apply_response = $noOpRaw
                public_gateway_boundary = Get-Sprint8AArtifact -Path $gatewayBoundaryPublication.path
                final_health = Get-Sprint8AArtifact -Path $finalHealthPublication.path
            }
            first_apply = $first
            no_op_apply = $noOp
            final_health_passed = [bool]$finalHealth.passed
        }
        $resultPublication = Publish-Sprint7AEvidence -Document $result -OutputPath $summaryPath
        Write-Host "Sprint 8A fresh materialization completed with exact owner read-back verified."
        Write-Host "Materialization summary: $($resultPublication.path)"
        Write-Output ([pscustomobject][ordered]@{
            contract = "tessara.sprint-8a.materialization-result-location"
            attempt = $Attempt
            evidence_directory = $attemptDirectory
            summary_path = $resultPublication.path
            summary_sha256 = $resultPublication.sha256
        })
    } catch {
        $materializationError = $_
        $materializationMessage = $materializationError.Exception.Message
        $rawArtifacts = [Collections.Generic.List[object]]::new()

        $bootstrapFailurePath = Join-Path $runtimeDirectory "apply-failure-response.log"
        if (Test-Path -LiteralPath $bootstrapFailurePath -PathType Leaf) {
            $rawArtifacts.Add((Copy-Sprint8ARawEvidence `
                -Source $bootstrapFailurePath `
                -Destination (Join-Path $attemptDirectory "failed-apply-response.log")))
        }
        foreach ($healthEvidenceName in @("public-gateway-boundary.json", "final-health.json")) {
            $healthEvidencePath = Join-Path $attemptDirectory $healthEvidenceName
            if (Test-Path -LiteralPath $healthEvidencePath -PathType Leaf) {
                $rawArtifacts.Add((Get-Sprint8AArtifact -Path $healthEvidencePath))
            }
        }
        try {
            $serviceLogs = @(& docker compose -f $composePath --profile reference logs --no-color --timestamps 2>&1)
            $serviceLogExitCode = $LASTEXITCODE
            $serviceLogArtifact = Write-Sprint8ARawLines `
                -Lines $serviceLogs `
                -Path (Join-Path $attemptDirectory "failed-topology-services.log")
            $rawArtifacts.Add($serviceLogArtifact)
        } catch {
            $serviceLogExitCode = -1
            $serviceLogArtifact = $null
        }

        try {
            $failureTeardown = Invoke-Sprint8AComposeDown `
                -ComposePath $composePath `
                -VolumeNames $resolvedVolumeNames `
                -NetworkNames $resolvedNetworkNames `
                -RawLogPath (Join-Path $attemptDirectory "failure-teardown.log") `
                -Reason "failed_attempt_exact_topology_removal"
        } catch {
            $failureTeardown = [pscustomobject][ordered]@{
                reason = "failed_attempt_exact_topology_removal"
                passed = $false
                internal_error = $_.Exception.Message
            }
        }
        $teardownPublication = Publish-Sprint7AEvidence `
            -Document ([ordered]@{
                schema_version = 1
                contract = "tessara.sprint-8a.failure-teardown"
                attempt = $Attempt
                source = $source
                environment_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
                resolved_targets_sha256 = [string]$targetPublication.sha256
                teardown = $failureTeardown
            }) `
            -OutputPath (Join-Path $attemptDirectory "failure-teardown.json")

        $faultEvidenceText = @($materializationMessage)
        foreach ($artifact in $rawArtifacts) {
            $artifactPath = Resolve-Sprint8ARepositoryPath -Path ([string]$artifact.path)
            if (Test-Path -LiteralPath $artifactPath -PathType Leaf) {
                $faultEvidenceText += Get-Content -LiteralPath $artifactPath -Raw
            }
        }
        $expectedFaultObserved = if ([string]::IsNullOrWhiteSpace($ExpectedFailurePattern)) {
            $null
        } else {
            ($faultEvidenceText -join "`n") -match $ExpectedFailurePattern
        }
        $runtimeArtifacts = @(
            "release-catalog.signed.json",
            "lockfile.json",
            "authorization.signed.json"
        ) | ForEach-Object {
            $runtimeArtifactPath = Join-Path $runtimeDirectory $_
            if (Test-Path -LiteralPath $runtimeArtifactPath -PathType Leaf) {
                Get-Sprint8AArtifact -Path $runtimeArtifactPath
            }
        }
        $exceptionType = $materializationError.Exception.GetType().FullName
        $failureClassification = Get-Sprint8AMaterializationFailureClassification `
            -ErrorRecord $materializationError `
            -ExpectedFaultObserved $expectedFaultObserved
        $failure = [ordered]@{
            schema_version = 2
            contract = "tessara.sprint-8a.materialization-failure"
            attempt = $Attempt
            passed = $false
            failed_at = [DateTimeOffset]::UtcNow.ToString("o")
            source = $source
            environment = [ordered]@{
                declared_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
                materialization_fingerprint = $observedEnvironmentFingerprint
            }
            inputs = [ordered]@{
                compose = Get-Sprint8AArtifact -Path $composePath
                blueprint = Get-Sprint8AArtifact -Path $resolvedBlueprintPath
            }
            expected_fault = if ($ExpectedFaultId) {
                [ordered]@{
                    id = $ExpectedFaultId
                    classification = "validation_only_fault_injection"
                    pattern = $ExpectedFailurePattern
                    observed = $expectedFaultObserved
                }
            } else { $null }
            failure = [ordered]@{
                classification = $failureClassification
                message = $materializationMessage
                exception_type = $exceptionType
                script_stack_trace = $materializationError.ScriptStackTrace
                service_log_exit_code = $serviceLogExitCode
            }
            raw_artifacts = @($rawArtifacts)
            runtime_artifacts = @($runtimeArtifacts)
            teardown = Get-Sprint8AArtifact -Path $teardownPublication.path
            teardown_passed = [bool]$failureTeardown.passed
            retained_partial_topology = -not [bool]$failureTeardown.passed
        }
        $failurePublication = Publish-Sprint7AEvidence -Document $failure -OutputPath $failurePath
        $exception = [InvalidOperationException]::new(
            "Sprint 8A materialization failed; evidence: $($failurePublication.path); exact teardown passed: $([bool]$failureTeardown.passed). $materializationMessage",
            $materializationError.Exception
        )
        $exception.Data["Sprint8AEvidenceDirectory"] = $attemptDirectory
        $exception.Data["Sprint8AFailureReceipt"] = $failurePublication.path
        throw $exception
    }
} finally {
    Pop-Location
}

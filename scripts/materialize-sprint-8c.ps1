[CmdletBinding()]
param(
    [Alias("Suite")]
    [ValidateSet("CoreFresh", "ResponseBootstrap", "Reference", "ReferenceNoOp", "All")]
    [string]$Target = "All",
    [string]$ComposeFile = "deploy/sprint-8c/compose.yaml",
    [string]$BlueprintPath = "deploy/sprint-8c/blueprints/reference.json",
    [string]$ComposeProject = "tessara-s8c-implementation-materialization",
    [string]$EvidencePath = "target/sprint-8c-materialization/result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$KeepTopology,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidencePathWasExplicit = $PSBoundParameters.ContainsKey("EvidencePath")
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")
$SelfTest = $requestedSelfTest
. (Join-Path $PSScriptRoot "sprint-8c-cargo-test-integrity.ps1")
$installationId = "01980000-0000-7000-8000-00000000008c"
$expectedOwnerOrder = @(
    "core",
    "tessara.responses",
    "tessara.datasets",
    "tessara.components",
    "tessara.dashboards",
    "tessara.reference.scoped-records"
)
$expectedResponseReadinessCodes = @(
    "response.database",
    "response.configuration",
    "response.security_state",
    "response.provider.forms",
    "response.provider.workflow",
    "response.events.publication",
    "response.export.publication"
)
$compositionBuildServices = @("responses", "datasets", "components")
$compositionNavigationHrefs = @("/responses", "/datasets", "/components")

function Invoke-Sprint8CFocusedCoreFreshProof {
    $containerName = "tessara-s8c-core-fresh-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
    $databaseName = "tessara_sprint_8c_core_fresh_test"
    $databaseUrlBefore = $env:TEST_API_FRESH_DATABASE_URL
    $sqlxDatabaseUrlBefore = $env:DATABASE_URL
    $ackBefore = $env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET
    try {
        $containerId = (& docker run --detach --rm --name $containerName `
            -e POSTGRES_USER=tessara_materialize `
            -e POSTGRES_PASSWORD=tessara_materialize `
            -e POSTGRES_DB=$databaseName `
            -p "127.0.0.1::5432" postgres:16-alpine).Trim()
        if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($containerId)) {
            throw "Could not start the isolated Sprint 8C Core materialization database."
        }
        $ready = $false
        for ($attempt = 1; $attempt -le 30; $attempt++) {
            & docker exec $containerName pg_isready -U tessara_materialize -d $databaseName *> $null
            if ($LASTEXITCODE -eq 0) { $ready = $true; break }
            Start-Sleep -Seconds 1
        }
        if (-not $ready) { throw "Sprint 8C Core materialization database did not become ready." }
        $portLine = (& docker port $containerName 5432/tcp).Trim()
        if ($LASTEXITCODE -ne 0 -or $portLine -notmatch '^127\.0\.0\.1:(\d+)$') {
            throw "Could not resolve the isolated Sprint 8C Core materialization port."
        }
        $env:TEST_API_FRESH_DATABASE_URL =
            "postgres://tessara_materialize:tessara_materialize@127.0.0.1:$($Matches[1])/$databaseName"
        $env:DATABASE_URL = $env:TEST_API_FRESH_DATABASE_URL
        $env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET =
            "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"
        Push-Location $repoRoot
        try {
            Invoke-Sprint8CCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "--test", "sprint_6a_populated_upgrade",
                "fresh_startup_and_seed_assignment_lock_order_use_a_separate_database",
                "--locked", "--offline", "--jobs", "1"
            ) | Out-Null
            Invoke-Sprint8CCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "--lib",
                "composition::tests::fresh_baseline_enrolls_dataset_security_before_core_actor_bootstrap",
                "--locked", "--offline", "--jobs", "1", "--", "--exact"
            ) | Out-Null
        } finally {
            Pop-Location
        }
        [pscustomobject][ordered]@{
            proof = "core-fresh-database"
            state = "passed"
            isolated_database = $databaseName
        }
    } finally {
        $env:TEST_API_FRESH_DATABASE_URL = $databaseUrlBefore
        $env:DATABASE_URL = $sqlxDatabaseUrlBefore
        $env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = $ackBefore
        $existing = @(& docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}")
        if ($LASTEXITCODE -eq 0 -and $existing -ccontains $containerName) {
            & docker rm -f $containerName | Out-Null
            if ($LASTEXITCODE -ne 0) {
                throw "Could not remove the Core fresh materialization database."
            }
        }
        if (& docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
            throw "Core fresh materialization database teardown was not exact."
        }
    }
}

function Invoke-Sprint8CFocusedResponseBootstrapProof {
    $result = Invoke-Sprint8CChildScript -ScriptPath "scripts/test-sprint-8c-response-module.ps1" `
        -Arguments @("-Suite", "Bootstrap")
    [pscustomobject][ordered]@{
        proof = "response-owner-bootstrap-and-migrations"
        state = "passed"
        output_sha256 = Get-Sprint7ASha256 -Text ((@($result.output) -join "`n") + "`n")
    }
}

function Assert-Sprint8CMaterializationReceipt {
    param(
        [Parameter(Mandatory)]$ApplyResponse,
        [Parameter(Mandatory)][bool]$ExpectedNoOp,
        [Parameter(Mandatory)][bool]$ExpectedChanged
    )

    if ($null -eq $ApplyResponse.operation -or $null -eq $ApplyResponse.receipt -or
        [string]$ApplyResponse.operation.state -cne "succeeded" -or
        [string]$ApplyResponse.operation.receipt_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
        [bool]$ApplyResponse.receipt.no_op -ne $ExpectedNoOp) {
        throw "Sprint 8C materialization apply response has an invalid operation/receipt contract."
    }
    $receipts = @($ApplyResponse.receipt.bootstrap_receipts)
    $actualOwnerOrder = @($receipts | ForEach-Object { [string]$_.owner })
    if (($actualOwnerOrder -join "`n") -cne ($expectedOwnerOrder -join "`n")) {
        throw "Sprint 8C owner bootstrap order is not the exact Core -> Response -> Dataset -> Component -> Dashboard -> reference order."
    }
    foreach ($receipt in $receipts) {
        if ([bool]$receipt.changed -ne $ExpectedChanged -or
            [string]$receipt.input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$receipt.result_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            @($receipt.resource_ids.PSObject.Properties).Count -eq 0) {
            throw "Owner receipt '$($receipt.owner)' does not match expected changed/read-back semantics."
        }
    }
    [pscustomobject][ordered]@{
        operation_state = [string]$ApplyResponse.operation.state
        changed = $ExpectedChanged
        operation_id = [string]$ApplyResponse.operation.operation_id
        receipt_digest = [string]$ApplyResponse.operation.receipt_digest
        revision = [uint64]$ApplyResponse.receipt.revision
        previous_receipt_digest = if ($null -eq $ApplyResponse.receipt.previous_receipt_digest) {
            $null
        } else {
            [string]$ApplyResponse.receipt.previous_receipt_digest
        }
        lockfile_digest = [string]$ApplyResponse.receipt.lockfile_digest
        plan_digest = [string]$ApplyResponse.receipt.plan_digest
        no_op = [bool]$ApplyResponse.receipt.no_op
        owner_order = $actualOwnerOrder
        owner_receipts = $receipts
        desired_enablement = $ApplyResponse.receipt.desired_enablement
        observed_enablement = $ApplyResponse.receipt.observed_enablement
        observed_artifacts = $ApplyResponse.receipt.observed_artifacts
        configuration_digests = $ApplyResponse.receipt.configuration_digests
    }
}

function Assert-Sprint8CSemanticNoOp {
    param(
        [Parameter(Mandatory)]$First,
        [Parameter(Mandatory)]$NoOp,
        [Parameter(Mandatory)]$FirstTopology,
        [Parameter(Mandatory)]$NoOpTopology
    )

    if ($null -ne $First.previous_receipt_digest -or
        [string]$NoOp.previous_receipt_digest -cne [string]$First.receipt_digest -or
        [uint64]$NoOp.revision -ne ([uint64]$First.revision + 1)) {
        throw "Sprint 8C semantic no-op receipt chain does not bind the exact first apply."
    }
    foreach ($field in @(
        "desired_enablement", "observed_enablement", "observed_artifacts", "configuration_digests"
    )) {
        if (($First.$field | ConvertTo-Json -Depth 100 -Compress) -cne
            ($NoOp.$field | ConvertTo-Json -Depth 100 -Compress)) {
            throw "Sprint 8C semantic no-op changed receipt field '$field'."
        }
    }
    $firstByOwner = @{}
    foreach ($receipt in @($First.owner_receipts)) {
        $firstByOwner[[string]$receipt.owner] = $receipt
    }
    foreach ($receipt in @($NoOp.owner_receipts)) {
        $firstReceipt = $firstByOwner[[string]$receipt.owner]
        if ($null -eq $firstReceipt -or
            [string]$receipt.input_digest -cne [string]$firstReceipt.input_digest -or
            [string]$receipt.result_digest -cne [string]$firstReceipt.result_digest -or
            ($receipt.resource_ids | ConvertTo-Json -Depth 100 -Compress) -cne
                ($firstReceipt.resource_ids | ConvertTo-Json -Depth 100 -Compress)) {
            throw "Sprint 8C semantic no-op changed owner read-back '$($receipt.owner)'."
        }
    }
    if (($FirstTopology | ConvertTo-Json -Depth 30 -Compress) -cne
        ($NoOpTopology | ConvertTo-Json -Depth 30 -Compress)) {
        throw "Sprint 8C semantic no-op changed container image/identity/restart topology."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        previous_receipt_chain = "exact"
        stable_owner_receipts = $true
        stable_enablement_artifacts_configuration = $true
        stable_container_topology = $true
    }
}

function Get-Sprint8CRuntimeTopologySnapshot {
    param([Parameter(Mandatory)][string]$ComposePath)

    $services = @(
        "postgres", "core", "supervisor", "responses", "datasets", "components", "dashboards",
        "scoped-records", "response-provider-proxy", "form-provider-proxy",
        "scope-provider-proxy", "principal-provider-proxy", "gateway"
    )
    @($services | ForEach-Object {
        $service = $_
        $containerId = ((Invoke-Sprint8CDockerCompose -ComposePath $ComposePath `
            -Arguments @("ps", "-q", $service)).output -join "").Trim()
        if ([string]::IsNullOrWhiteSpace($containerId)) {
            throw "Sprint 8C topology omits running service '$service'."
        }
        $inspectionOutput = @(& docker inspect $containerId 2>&1)
        if ($LASTEXITCODE -ne 0) {
            throw "Could not inspect Sprint 8C service '$service'."
        }
        try {
            $inspection = @(($inspectionOutput -join "`n") | ConvertFrom-Json -Depth 100)
        } catch {
            throw "Could not decode the Sprint 8C service '$service' inspection."
        }
        if ($inspection.Count -ne 1 -or $null -eq $inspection[0].State) {
            throw "Could not inspect exactly one Sprint 8C service '$service'."
        }
        $container = $inspection[0]
        $healthProperty = $container.State.PSObject.Properties['Health']
        $health = if ($null -eq $healthProperty -or $null -eq $healthProperty.Value) {
            ''
        } else {
            [string]$healthProperty.Value.Status
        }
        if ([string]$container.State.Status -cne "running" -or
            (-not [string]::IsNullOrWhiteSpace($health) -and $health -cne "healthy")) {
            throw "Sprint 8C service '$service' is not exactly running/healthy."
        }
        [pscustomobject][ordered]@{
            service = $service
            container_id = [string]$container.Id
            image_id = [string]$container.Image
            restart_count = [int]$container.RestartCount
            state = [string]$container.State.Status
            health = $health
        }
    })
}

function Invoke-Sprint8CServiceProbe {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$Service,
        [Parameter(Mandatory)][string]$Uri,
        [int]$ExpectedStatus = 200
    )

    $result = Invoke-Sprint8CDockerCompose -ComposePath $ComposePath -Arguments @(
        "exec", "-T", $Service, "curl", "-sS", "-w", "`n%{http_code}", $Uri
    ) -AllowFailure
    if ($result.exit_code -ne 0 -or $result.output.Count -lt 1) {
        throw "Internal service probe '$Service $Uri' could not execute."
    }
    $status = [int]$result.output[-1]
    $body = @($result.output[0..($result.output.Count - 2)]) -join "`n"
    if ($status -ne $ExpectedStatus) {
        throw "Internal service probe '$Service $Uri' returned $status instead of $ExpectedStatus."
    }
    [pscustomobject][ordered]@{
        service = $Service
        uri = $Uri
        status = $status
        body = $body
        body_sha256 = Get-Sprint7ASha256 -Text $body
    }
}

function Assert-Sprint8CResponseHealthDocuments {
    param(
        [Parameter(Mandatory)]$Live,
        [Parameter(Mandatory)]$Ready
    )

    $expectedEnvelopeProperties = @("schema_version", "status", "checks")
    foreach ($document in @($Live, $Ready)) {
        $actualProperties = @($document.PSObject.Properties.Name | Sort-Object)
        if (($actualProperties -join "`n") -cne
            (($expectedEnvelopeProperties | Sort-Object) -join "`n")) {
            throw "Response health must use the exact shared runtime envelope."
        }
        if ([int]$document.schema_version -ne 1 -or
            [string]$document.status -cne "passing") {
            throw "Response health must report the passing shared runtime contract."
        }
    }
    if (@($Live.checks).Count -ne 0) {
        throw "Response liveness must not contain readiness checks."
    }

    $readyChecks = @($Ready.checks)
    if ($readyChecks.Count -ne $expectedResponseReadinessCodes.Count -or
        (@($readyChecks.code) -join "`n") -cne ($expectedResponseReadinessCodes -join "`n")) {
        throw "Response readiness must contain the exact ordered owner check inventory."
    }
    foreach ($check in $readyChecks) {
        $actualCheckProperties = @($check.PSObject.Properties.Name | Sort-Object)
        if (($actualCheckProperties -join "`n") -cne
                ((@("code", "passing", "message") | Sort-Object) -join "`n") -or
            -not [bool]$check.passing -or
            [string]::IsNullOrWhiteSpace([string]$check.message)) {
            throw "Response readiness checks must use the exact passing shared runtime shape."
        }
    }
}

function Get-Sprint8CMaterializationHealth {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)]$Ports
    )

    $gateway = Wait-Sprint8CHttpProbe -Uri "$($Ports.gateway_url)/health" -ExpectedStatus @(200)
    $supervisor = Wait-Sprint8CHttpProbe -Uri "$($Ports.supervisor_url)/health/ready" -ExpectedStatus @(204)
    if (-not [string]::IsNullOrWhiteSpace([string]$supervisor.body) -or
        -not [string]::IsNullOrWhiteSpace([string]$supervisor.content_type)) {
        throw "Supervisor readiness must return an exact bodyless 204 response without a content type."
    }
    $responseLive = Invoke-Sprint8CServiceProbe -ComposePath $ComposePath -Service "responses" `
        -Uri "http://127.0.0.1:8094/health/live"
    $responseReady = Invoke-Sprint8CServiceProbe -ComposePath $ComposePath -Service "responses" `
        -Uri "http://127.0.0.1:8094/health/ready"
    $responseLiveDocument = $responseLive.body | ConvertFrom-Json -Depth 20
    $responseReadyDocument = $responseReady.body | ConvertFrom-Json -Depth 20
    Assert-Sprint8CResponseHealthDocuments -Live $responseLiveDocument -Ready $responseReadyDocument

    $datasetLive = Invoke-Sprint8CServiceProbe -ComposePath $ComposePath -Service "datasets" `
        -Uri "http://127.0.0.1:8093/health/live"
    $datasetReady = Invoke-Sprint8CServiceProbe -ComposePath $ComposePath -Service "datasets" `
        -Uri "http://127.0.0.1:8093/health/ready"
    $datasetLiveDocument = $datasetLive.body | ConvertFrom-Json -Depth 20
    $datasetReadyDocument = $datasetReady.body | ConvertFrom-Json -Depth 20
    if ([int]$datasetLiveDocument.schema_version -ne 1 -or
        [string]$datasetLiveDocument.module_definition_id -cne "tessara.datasets" -or
        [string]$datasetLiveDocument.module_release_version -cne "1.0.0" -or
        [string]$datasetLiveDocument.status -cne "live" -or
        [string]$datasetReadyDocument.status -cne "ready") {
        throw "Dataset exact liveness/readiness identity contract failed after materialization."
    }
    [pscustomobject][ordered]@{
        gateway_core = $gateway
        supervisor = $supervisor
        response_live = $responseLiveDocument
        response_ready = $responseReadyDocument
        dataset_live = $datasetLiveDocument
        dataset_ready = $datasetReadyDocument
    }
}

function Invoke-Sprint8CCompositionBootstrap {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$ResolvedBlueprint,
        [Parameter(Mandatory)][string]$ResolvedComposeProject,
        [Parameter(Mandatory)]$Ports,
        [Parameter(Mandatory)][string]$RuntimeDirectory,
        [Parameter(Mandatory)][bool]$BuildSkipped,
        [switch]$NoOp,
        [switch]$ExcludeGateway
    )

    $parameters = @{
        Composition = "reference"
        ComposeFile = $ComposePath
        CoreUrl = $Ports.core_url
        SupervisorUrl = $Ports.supervisor_url
        BlueprintPath = $ResolvedBlueprint
        RuntimeDirectory = $RuntimeDirectory
        ReplaceExisting = -not $NoOp
        DeploymentDirectory = "sprint-8c"
        ExpectedProject = $ResolvedComposeProject
        InstallationId = $installationId
        RuntimeLabel = "sprint-8c"
        AdditionalBuildServices = @($compositionBuildServices)
        AdditionalExpectedNavigationHrefs = @($compositionNavigationHrefs)
        SkipLegacySeed = $true
        SemanticNoOp = [bool]$NoOp
        ExcludePublicGateway = [bool]$ExcludeGateway
        SkipBuild = $BuildSkipped
        Confirm = $false
    }
    & (Join-Path $PSScriptRoot "bootstrap-sprint-7a-composition.ps1") @parameters | Out-Host
    $receiptPath = Join-Path $RuntimeDirectory "apply-response.json"
    if (-not (Test-Path -LiteralPath $receiptPath -PathType Leaf)) {
        throw "Sprint 8C composition bootstrap did not retain apply response '$receiptPath'."
    }
    $receiptPath
}

function Get-Sprint8CRetainedMaterializationDiagnostics {
    param([Parameter(Mandatory)][string]$RuntimeDirectory)

    if (-not (Test-Path -LiteralPath $RuntimeDirectory -PathType Container)) { return @() }
    @(
        Get-ChildItem -LiteralPath $RuntimeDirectory -File -Recurse | Where-Object {
            $_.Name -match '(?i)(failure|operation|receipt|lockfile)'
        } | Sort-Object FullName | ForEach-Object {
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($repoRoot, $_.FullName).Replace("\", "/")
                sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                size = [uint64]$_.Length
            }
        }
    )
}

function New-Sprint8CMaterializationSelfTestReceipt {
    param(
        [Parameter(Mandatory)][uint64]$Revision,
        [Parameter(Mandatory)][bool]$NoOp,
        [string]$PreviousReceiptDigest
    )

    $digest = "sha256:$('a' * 64)"
    $ownerReceipts = @($expectedOwnerOrder | ForEach-Object {
        [pscustomobject][ordered]@{
            owner = $_
            schema_version = "tessara.io/mock-bootstrap/v1"
            input_digest = $digest
            result_digest = $digest
            changed = -not $NoOp
            resource_ids = [pscustomobject][ordered]@{ fixture = "typed-owner-read-back" }
        }
    })
    [pscustomobject][ordered]@{
        operation = [pscustomobject][ordered]@{
            operation_id = "01980000-0000-7000-8000-000000000001"
            state = "succeeded"
            receipt_digest = $digest
        }
        receipt = [pscustomobject][ordered]@{
            revision = $Revision
            previous_receipt_digest = if ([string]::IsNullOrEmpty($PreviousReceiptDigest)) {
                $null
            } else {
                $PreviousReceiptDigest
            }
            lockfile_digest = $digest
            plan_digest = $digest
            no_op = $NoOp
            bootstrap_receipts = $ownerReceipts
            desired_enablement = [pscustomobject]@{ "tessara.responses" = $true; "tessara.datasets" = $true }
            observed_enablement = [pscustomobject]@{ "tessara.responses" = $true; "tessara.datasets" = $true }
            observed_artifacts = [pscustomobject]@{ "tessara.responses" = $digest; "tessara.datasets" = $digest }
            configuration_digests = [pscustomobject]@{ "tessara.responses" = $digest; "tessara.datasets" = $digest }
        }
    }
}

function Test-Sprint8CMaterializationHarness {
    $materializerSource = Get-Content -Raw -LiteralPath $PSCommandPath
    $compositionBootstrapSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "scripts/bootstrap-sprint-7a-composition.ps1"
    )
    if ($compositionBootstrapSource -cnotmatch '-Uri "\$CoreUrl/api/me"' -or
        $compositionBootstrapSource -cnotmatch 'actor_id = \$authenticatedAccountId\.ToString\(\)' -or
        $compositionBootstrapSource -cmatch 'actor_id = "local:\$RuntimeLabel-(?:bootstrap|approver)"') {
        throw "Sprint 8C materialization must bind its signed apply authorization to the authenticated Core account UUID."
    }
    $expectedCompositionBuildServices = @("responses", "datasets", "components")
    $expectedCompositionNavigationHrefs = @("/responses", "/datasets", "/components")
    if (($compositionBuildServices -join "`n") -cne
            ($expectedCompositionBuildServices -join "`n") -or
        @($compositionBuildServices | Select-Object -Unique).Count -ne
            $compositionBuildServices.Count -or
        ($compositionNavigationHrefs -join "`n") -cne
            ($expectedCompositionNavigationHrefs -join "`n") -or
        @($compositionNavigationHrefs | Select-Object -Unique).Count -ne
            $compositionNavigationHrefs.Count) {
        throw "Sprint 8C materialization does not build and navigate the exact Response, Dataset, and Component module projection."
    }
    $datasetMigration = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "crates/tessara-dataset-module/migrations/001_dataset_module.sql"
    )
    $postgresBootstrap = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "deploy/sprint-8b/postgres-init.sh"
    )
    $responsePostgresBootstrap = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "deploy/sprint-8c/postgres-response-init.sh"
    )
    $composeOverride = @(
        Get-Content -Raw -LiteralPath (Join-Path $repoRoot "deploy/sprint-8b/compose.override.yaml")
        Get-Content -Raw -LiteralPath (Join-Path $repoRoot "deploy/sprint-8c/compose.override.yaml")
    ) -join "`n"
    if ($composeOverride -cnotmatch '(?m)^\s{6}TESSARA_CORE_CONTROL_PORT: \$\{TESSARA_CORE_CONTROL_PORT:-18089\}$') {
        throw "Sprint 8C Supervisor must preserve the harness-selected Core control port across runtime switches."
    }
    foreach ($requiredProjection in @(
        [pscustomobject]@{
            Label = "Response owner database"
            Source = $responsePostgresBootstrap
            Pattern = '(?m)^  CREATE DATABASE tessara_module_responses OWNER tessara_response_owner;$'
        },
        [pscustomobject]@{
            Label = "Response runtime default table privileges"
            Source = $responsePostgresBootstrap
            Pattern = '(?m)^    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_response_runtime;$'
        },
        [pscustomobject]@{
            Label = "Dataset migration materialized schema"
            Source = $datasetMigration
            Pattern = '(?m)^CREATE SCHEMA IF NOT EXISTS dataset_materialized;$'
        },
        [pscustomobject]@{
            Label = "Dataset migration materialized schema owner guard"
            Source = $datasetMigration
            Pattern = "nspowner FROM pg_namespace WHERE nspname = 'dataset_materialized'"
        },
        [pscustomobject]@{
            Label = "Dataset materialized schema owner"
            Source = $postgresBootstrap
            Pattern = '(?m)^  CREATE SCHEMA dataset_materialized AUTHORIZATION tessara_dataset_owner;$'
        },
        [pscustomobject]@{
            Label = "Dataset runtime materialized DDL grant"
            Source = $postgresBootstrap
            Pattern = '(?m)^  GRANT USAGE, CREATE ON SCHEMA dataset_materialized TO tessara_dataset_runtime;$'
        },
        [pscustomobject]@{
            Label = "Dataset materialized table default privileges"
            Source = $postgresBootstrap
            Pattern = '(?m)^  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_dataset_owner IN SCHEMA dataset_materialized GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_dataset_runtime;$'
        },
        [pscustomobject]@{
            Label = "Sprint 8C final PostgreSQL TCP health gate"
            Source = $composeOverride
            Pattern = 'pg_isready -h 127\.0\.0\.1 -U tessara_bootstrap -d postgres'
        },
        [pscustomobject]@{
            Label = "Response declared graceful shutdown window"
            Source = $composeOverride
            Pattern = '(?m)^    stop_grace_period: 30s$'
        }
    )) {
        if ($requiredProjection.Source -notmatch $requiredProjection.Pattern) {
            throw "$($requiredProjection.Label) is missing from the fresh deployment contract."
        }
    }

    $firstResponse = New-Sprint8CMaterializationSelfTestReceipt -Revision 1 -NoOp $false
    $first = Assert-Sprint8CMaterializationReceipt -ApplyResponse $firstResponse `
        -ExpectedNoOp $false -ExpectedChanged $true
    $noOpResponse = New-Sprint8CMaterializationSelfTestReceipt -Revision 2 -NoOp $true `
        -PreviousReceiptDigest $first.receipt_digest
    $noOp = Assert-Sprint8CMaterializationReceipt -ApplyResponse $noOpResponse `
        -ExpectedNoOp $true -ExpectedChanged $false
    $topology = @([pscustomobject][ordered]@{
        service = "datasets"; container_id = "container"; image_id = "sha256:$('b' * 64)"
        restart_count = 0; state = "running"; health = "healthy"
    })
    Assert-Sprint8CSemanticNoOp -First $first -NoOp $noOp `
        -FirstTopology $topology -NoOpTopology $topology | Out-Null

    $responseLive = [pscustomobject][ordered]@{
        schema_version = 1
        status = "passing"
        checks = @()
    }
    $responseReady = [pscustomobject][ordered]@{
        schema_version = 1
        status = "passing"
        checks = @(
            $expectedResponseReadinessCodes | ForEach-Object {
                [pscustomobject][ordered]@{
                    code = $_
                    passing = $true
                    message = "self-test passing check"
                }
            }
        )
    }
    Assert-Sprint8CResponseHealthDocuments -Live $responseLive -Ready $responseReady

    $datasetShapedResponseLive = [pscustomobject][ordered]@{
        schema_version = 1
        module_definition_id = "tessara.responses"
        module_release_version = "1.0.0"
        status = "live"
    }
    try {
        Assert-Sprint8CResponseHealthDocuments -Live $datasetShapedResponseLive -Ready $responseReady
        throw "Sprint 8C materialization self-test accepted a Dataset-shaped Response health document."
    } catch {
        if ($_.Exception.Message -notmatch 'exact shared runtime envelope') { throw }
    }

    $tampered = $noOpResponse | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $tampered.receipt.bootstrap_receipts[1].changed = $true
    try {
        Assert-Sprint8CMaterializationReceipt -ApplyResponse $tampered `
            -ExpectedNoOp $true -ExpectedChanged $false | Out-Null
        throw "Sprint 8C materialization self-test accepted a mutating no-op receipt."
    } catch {
        if ($_.Exception.Message -notmatch 'changed/read-back semantics') { throw }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "materialization-harness-self-test"
        state = "passed"
        database_free = $true
        compose_project = $null
        composition_bootstrap = [pscustomobject][ordered]@{
            build_services = @($compositionBuildServices)
            navigation_hrefs = @($compositionNavigationHrefs)
        }
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text "sprint-8c-materialization-self-test`n"
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"
            mode = "database-free-self-test"
        }
    }
}

if ($SelfTest) {
    $result = Test-Sprint8CMaterializationHarness
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8CHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result | ConvertTo-Json -Depth 30
    return
}

if ($Target -in @("CoreFresh", "ResponseBootstrap", "All")) {
    $focused = [Collections.Generic.List[object]]::new()
    if ($Target -in @("CoreFresh", "All")) {
        $focused.Add((Invoke-Sprint8CFocusedCoreFreshProof))
    }
    if ($Target -in @("ResponseBootstrap", "All")) {
        $focused.Add((Invoke-Sprint8CFocusedResponseBootstrapProof))
    }
    $source = Get-Sprint8CSourceIdentity
    $document = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "focused-owner-materialization"
        state = "passed"
        target = $Target
        compose_project = $null
        source = $source
        results = @($focused)
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text (
            "$($source.commit)`n$($source.tree)`n$Target`n"
        )
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"
            mode = "isolated-database-containers-removed"
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $EvidencePath | Out-Null
    }
    $document | ConvertTo-Json -Depth 50
    return
}

Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject `
    -Authorized ([bool]$AuthorizeDisposableReset)
$source = Get-Sprint8CSourceIdentity -RequireClean
$composePath = Resolve-Sprint8CRepositoryPath -Path $ComposeFile
$resolvedBlueprintPath = Resolve-Sprint8CRepositoryPath -Path $BlueprintPath
if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) {
    throw "Sprint 8C Compose file is missing: $composePath"
}
if (-not (Test-Path -LiteralPath $resolvedBlueprintPath -PathType Leaf)) {
    throw "Sprint 8C reference Blueprint is missing: $resolvedBlueprintPath"
}

$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT"
)
$environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $environmentNames
$ports = $null
$configuration = $null
$isolation = $null
$first = $null
$noOp = $null
$noOpProof = $null
$health = $null
$firstTopology = $null
$noOpTopology = $null
$fixtureReceiptPath = $null
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$failure = $null
$runtimeRoot = Resolve-Sprint8CRepositoryPath -Path (
    "target/sprint-8c-materialization/$ComposeProject-$([Guid]::NewGuid().ToString('N'))"
)
[IO.Directory]::CreateDirectory($runtimeRoot) | Out-Null

try {
    $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject
    $preexisting = Get-Sprint8CProjectResources -ComposeProject $ComposeProject
    if ($preexisting.containers.Count -ne 0 -or $preexisting.volumes.Count -ne 0 -or
        $preexisting.networks.Count -ne 0) {
        Remove-Sprint8CProjectTopology -ComposePath $composePath `
            -ComposeProject $ComposeProject -Authorized $true | Out-Null
    }
    Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject | Out-Null
    $configuration = Get-Sprint8CComposeConfiguration -ComposePath $composePath `
        -ComposeProject $ComposeProject
    $isolation = Assert-Sprint8CDatabaseIsolationConfiguration -ComposeConfiguration $configuration

    $firstRuntime = Join-Path $runtimeRoot "first"
    [IO.Directory]::CreateDirectory($firstRuntime) | Out-Null
    $firstPath = Invoke-Sprint8CCompositionBootstrap -ComposePath $composePath `
        -ResolvedBlueprint $resolvedBlueprintPath -ResolvedComposeProject $ComposeProject `
        -Ports $ports -RuntimeDirectory $firstRuntime -BuildSkipped ([bool]$SkipBuild) `
        -ExcludeGateway
    $runningGateway = ((Invoke-Sprint8CDockerCompose -ComposePath $composePath `
        -Arguments @("ps", "--status", "running", "-q", "gateway")).output | Where-Object { $_ })
    if (@($runningGateway).Count -ne 0) {
        throw "Public gateway started before owner materialization and read-back completed."
    }
    $firstResponse = Get-Content -LiteralPath $firstPath -Raw | ConvertFrom-Json -Depth 100
    $first = Assert-Sprint8CMaterializationReceipt -ApplyResponse $firstResponse `
        -ExpectedNoOp $false -ExpectedChanged $true

    Invoke-Sprint8CDockerCompose -ComposePath $composePath `
        -Arguments @("up", "-d", "--no-build", "gateway") | Out-Null
    $health = Get-Sprint8CMaterializationHealth -ComposePath $composePath -Ports $ports
    $firstTopology = @(Get-Sprint8CRuntimeTopologySnapshot -ComposePath $composePath)

    if ($Target -ceq "ReferenceNoOp") {
        $noOpRuntime = Join-Path $runtimeRoot "noop"
        [IO.Directory]::CreateDirectory($noOpRuntime) | Out-Null
        $noOpPath = Invoke-Sprint8CCompositionBootstrap -ComposePath $composePath `
            -ResolvedBlueprint $resolvedBlueprintPath -ResolvedComposeProject $ComposeProject `
            -Ports $ports -RuntimeDirectory $noOpRuntime -BuildSkipped $true -NoOp
        $noOpResponse = Get-Content -LiteralPath $noOpPath -Raw | ConvertFrom-Json -Depth 100
        $noOp = Assert-Sprint8CMaterializationReceipt -ApplyResponse $noOpResponse `
            -ExpectedNoOp $true -ExpectedChanged $false
        $noOpTopology = @(Get-Sprint8CRuntimeTopologySnapshot -ComposePath $composePath)
        $noOpProof = Assert-Sprint8CSemanticNoOp -First $first -NoOp $noOp `
            -FirstTopology $firstTopology -NoOpTopology $noOpTopology
        $health = Get-Sprint8CMaterializationHealth -ComposePath $composePath -Ports $ports
    }

    # Post-initial Response actions must occur only after Dataset's first bootstrap sync.
    # For the semantic no-op target they also occur after the repeated owner apply, so
    # fixture-only Response changes cannot turn the Blueprint repeat into a mutation.
    $fixtureReceiptPath = Join-Path $runtimeRoot "fixture-receipt.json"
    & (Join-Path $PSScriptRoot "prepare-sprint-8c-uat-fixtures.ps1") `
        -ApplyResponsePath $firstPath -OutputPath $fixtureReceiptPath `
        -ComposeProject $ComposeProject -CoreBaseUrl $ports.core_url | Out-Host
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $fixtureReceiptPath `
        -SidecarPath "$fixtureReceiptPath.sha256")) {
        throw "Sprint 8C owner-controlled fixture receipt was not published atomically."
    }
} catch {
    $failure = $_
    try {
        $diagnosticServices = @(
            "core", "gateway", "responses", "datasets", "components", "dashboards",
            "scoped-records", "supervisor", "response-provider-proxy",
            "form-provider-proxy", "scope-provider-proxy", "principal-provider-proxy"
        )
        $serviceLogs = @(Invoke-Sprint8CDockerCompose -ComposePath $composePath `
            -Arguments (@("logs", "--no-color", "--timestamps") + $diagnosticServices) `
            -AllowFailure).output
        if ($serviceLogs.Count -gt 0) {
            [IO.File]::WriteAllLines(
                (Join-Path $runtimeRoot "materialization-failure-service-logs.log"),
                $serviceLogs,
                [Text.UTF8Encoding]::new($false)
            )
        }
    } catch {
        # The original materialization failure remains authoritative when
        # bounded service-log capture is itself unavailable.
    }
} finally {
    try {
        if ($null -eq $failure -and $KeepTopology) {
            Assert-Sprint8CExistingTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject | Out-Null
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"
                mode = "retained-for-caller"
                topology_health = "passed"
            }
        } elseif ($null -ne $ports) {
            $cleanup = Remove-Sprint8CProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized $true
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
            $cleanup | Add-Member -NotePropertyName mode -NotePropertyValue "exact-project-teardown" -Force
        }
    } catch {
        if ($null -eq $failure) { $failure = $_ }
        $cleanup = [pscustomobject][ordered]@{
            state = "failed"
            mode = "exact-project-teardown"
            error = $_.Exception.Message
        }
    } finally {
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$composeConfigurationHash = if ($null -eq $configuration) { $null } else {
    Get-Sprint7ASha256 -Text (($configuration | ConvertTo-Json -Depth 100 -Compress) + "`n")
}
$blueprintHash = (Get-FileHash -LiteralPath $resolvedBlueprintPath -Algorithm SHA256).Hash.ToLowerInvariant()
$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n$composeConfigurationHash`n$blueprintHash`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    proof = if ($Target -ceq "ReferenceNoOp") {
        "clean-owner-materialization-and-semantic-noop"
    } else {
        "clean-owner-materialization"
    }
    state = if ($null -eq $failure -and [string]$cleanup.state -ceq "passed") { "passed" } else { "failed" }
    target = $Target
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    environment = if ($null -eq $ports) { $null } else { [pscustomobject][ordered]@{
        COMPOSE_PROJECT_NAME = $ComposeProject
        TESSARA_GATEWAY_PORT = [string]$ports.gateway_port
        TESSARA_CORE_CONTROL_PORT = [string]$ports.core_port
        TESSARA_SUPERVISOR_PORT = [string]$ports.supervisor_port
        fingerprint_sha256 = $environmentFingerprint
    } }
    compose_configuration_sha256 = $composeConfigurationHash
    blueprint_sha256 = $blueprintHash
    database_isolation = $isolation
    gateway_start_boundary = [pscustomobject][ordered]@{
        owner_apply_completed_before_start = $null -ne $first
        post_start_health = if ($null -eq $health) { "not_proven" } else { "passed" }
    }
    first_apply = $first
    semantic_noop = $noOp
    semantic_noop_proof = $noOpProof
    health = $health
    fixture_receipt_path = $fixtureReceiptPath
    fixture_receipt_sha256 = if ($null -ne $fixtureReceiptPath -and
        (Test-Path -LiteralPath $fixtureReceiptPath -PathType Leaf)) {
        (Get-FileHash -LiteralPath $fixtureReceiptPath -Algorithm SHA256).Hash.ToLowerInvariant()
    } else { $null }
    retained_diagnostics = @(Get-Sprint8CRetainedMaterializationDiagnostics `
        -RuntimeDirectory $runtimeRoot)
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
        message = $failure.Exception.Message
        category = [string]$failure.CategoryInfo.Category
    } }
}
$evidenceFullPath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8C materialization failed; retained evidence: $evidenceFullPath"
}

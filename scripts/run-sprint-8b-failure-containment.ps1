[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8b-implementation-recovery",
    [string]$ComposeFile = "deploy/sprint-8b/compose.yaml",
    [string]$FaultFixturePath = "deploy/sprint-8b/fixtures/provider-fault-contract.json",
    [string]$EvidencePath = "target/sprint-8b-failure-containment/result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidencePathWasExplicit = $PSBoundParameters.ContainsKey("EvidencePath")
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8b-harness-isolation.ps1")
$SelfTest = $requestedSelfTest

$faultControlContract = "tessara.sprint-8b.failure-control/v1"
$faultControlScope = "disposable-sprint-8b-only"
$faultEnvironmentNames = @(
    "TESSARA_SPRINT_8B_RESPONSE_FAULT_KEY",
    "TESSARA_SPRINT_8B_DATASET_FAULT_KEY",
    "TESSARA_SPRINT_8B_FAULT_CORRELATION_ID",
    "TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT"
)

function Copy-Sprint8BFailureJsonValue {
    param([Parameter(Mandatory)]$Value)
    $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
}

function Get-Sprint8BFailureControlContract {
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = $faultControlContract
        scope = $faultControlScope
        default_state = "off"
        attempt_limit = 1
        controls = @(
            [pscustomobject][ordered]@{
                fault_key = "response.incompatible"
                target_service = "response-provider-proxy"
                host_environment = "TESSARA_SPRINT_8B_RESPONSE_FAULT_KEY"
                service_environment = "TESSARA_SPRINT_8B_FAULT_KEY"
                mode = "incompatible_version"
                phase = "dataset_bootstrap_provider_validation"
                expected_outcome = "rejected_pre_write"
                expected_failure_code = "dataset.dependency_incompatible"
                dataset_transaction = "not_started"
            },
            [pscustomobject][ordered]@{
                fault_key = "dataset.derived-rebuild"
                target_service = "datasets"
                host_environment = "TESSARA_SPRINT_8B_DATASET_FAULT_KEY"
                service_environment = "TESSARA_SPRINT_8B_FAULT_KEY"
                mode = "deterministic_rebuild_failure"
                phase = "dataset_bootstrap_transaction"
                expected_outcome = "rolled_back"
                expected_failure_code = "dataset.dependency_unavailable"
                dataset_transaction = "rolled_back"
            }
        )
        common_service_environment = [pscustomobject][ordered]@{
            TESSARA_SPRINT_8B_FAULT_CONTRACT = $faultControlContract
            TESSARA_SPRINT_8B_FAULT_SCOPE = $faultControlScope
            TESSARA_SPRINT_8B_FAULT_CORRELATION_ID = "caller-generated-uuid"
            TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT = "1"
        }
        required_fault_receipt_fields = @(
            "schema_version", "contract", "fault_key", "correlation_id", "target_service",
            "phase", "attempt", "attempt_limit", "outcome", "failure_code",
            "dataset_transaction", "unauthorized_state", "cross_owner_write"
        )
        restoration = [pscustomobject][ordered]@{
            fault_environment = "cleared-to-none"
            partial_topology = "exact-project-teardown"
            successor_start = "verified-empty"
            successor_apply = "canonical-first-apply"
            successor_noop = "exact-semantic-noop"
            canonical_health = "passed"
        }
    }
}

function Read-Sprint8BFailureFixture {
    $path = Resolve-Sprint8BRepositoryPath -Path $FaultFixturePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Sprint 8B fault fixture is missing: $path"
    }
    $fixture = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -Depth 100
    if ([int]$fixture.schema_version -ne 1 -or
        [string]$fixture.contract -cne "tessara.sprint-8b.provider-faults" -or
        [string]$fixture.isolation -cne "one-binding-per-proxy" -or
        [bool]$fixture.product_bypass_forbidden -ne $true -or
        [bool]$fixture.database_bypass_forbidden -ne $true) {
        throw "Sprint 8B fault fixture does not enforce isolated owner-boundary failure injection."
    }
    foreach ($key in @("response.incompatible", "dataset.derived-rebuild")) {
        if (@($fixture.faults | Where-Object { [string]$_.key -ceq $key }).Count -ne 1) {
            throw "Sprint 8B fault fixture omits exact required fault '$key'."
        }
    }
    $fixture
}

function Get-Sprint8BServiceEnvironment {
    param(
        [Parameter(Mandatory)]$ComposeConfiguration,
        [Parameter(Mandatory)][string]$Service
    )

    $property = $ComposeConfiguration.services.PSObject.Properties[$Service]
    if ($null -eq $property) {
        throw "Failure-control discovery cannot find Compose service '$Service'."
    }
    $property.Value.environment
}

function Assert-Sprint8BFailureControlProjection {
    param(
        [Parameter(Mandatory)]$ComposeConfiguration,
        [Parameter(Mandatory)][string]$ActiveFault,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $contract = Get-Sprint8BFailureControlContract
    $active = @($contract.controls | Where-Object { [string]$_.fault_key -ceq $ActiveFault })
    if ($active.Count -ne 1) { throw "Unknown Sprint 8B failure control '$ActiveFault'." }
    foreach ($control in @($contract.controls)) {
        $environment = Get-Sprint8BServiceEnvironment -ComposeConfiguration $ComposeConfiguration `
            -Service ([string]$control.target_service)
        $expectedKey = if ([string]$control.fault_key -ceq $ActiveFault) {
            [string]$control.fault_key
        } else { "none" }
        if ([string]$environment.TESSARA_SPRINT_8B_FAULT_CONTRACT -cne $faultControlContract -or
            [string]$environment.TESSARA_SPRINT_8B_FAULT_SCOPE -cne $faultControlScope -or
            [string]$environment.TESSARA_SPRINT_8B_FAULT_KEY -cne $expectedKey -or
            [string]$environment.TESSARA_SPRINT_8B_FAULT_CORRELATION_ID -cne $CorrelationId -or
            [string]$environment.TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT -cne "1") {
            throw "Compose service '$($control.target_service)' does not expose exact off-by-default, one-shot '$ActiveFault' failure control projection."
        }
    }
    [pscustomobject][ordered]@{
        contract = $faultControlContract
        active_fault = $ActiveFault
        correlation_id = $CorrelationId
        attempt_limit = 1
        isolated_other_control = "none"
        state = "armed"
    }
}

function Set-Sprint8BFailureEnvironment {
    param(
        [Parameter(Mandatory)][string]$FaultKey,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $env:TESSARA_SPRINT_8B_RESPONSE_FAULT_KEY = if ($FaultKey -ceq "response.incompatible") {
        $FaultKey
    } else { "none" }
    $env:TESSARA_SPRINT_8B_DATASET_FAULT_KEY = if ($FaultKey -ceq "dataset.derived-rebuild") {
        $FaultKey
    } else { "none" }
    $env:TESSARA_SPRINT_8B_FAULT_CORRELATION_ID = $CorrelationId
    $env:TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT = "1"
}

function Clear-Sprint8BFailureEnvironment {
    $env:TESSARA_SPRINT_8B_RESPONSE_FAULT_KEY = "none"
    $env:TESSARA_SPRINT_8B_DATASET_FAULT_KEY = "none"
    $env:TESSARA_SPRINT_8B_FAULT_CORRELATION_ID = "00000000-0000-0000-0000-000000000000"
    $env:TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT = "0"
}

function Assert-Sprint8BFaultReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $actualFields = @($Receipt.PSObject.Properties.Name | Sort-Object)
    $expectedFields = @((Get-Sprint8BFailureControlContract).required_fault_receipt_fields | Sort-Object)
    if (($actualFields -join "`n") -cne ($expectedFields -join "`n") -or
        [int]$Receipt.schema_version -ne 1 -or
        [string]$Receipt.contract -cne $faultControlContract -or
        [string]$Receipt.fault_key -cne [string]$Control.fault_key -or
        [string]$Receipt.correlation_id -cne $CorrelationId -or
        [string]$Receipt.target_service -cne [string]$Control.target_service -or
        [string]$Receipt.phase -cne [string]$Control.phase -or
        [uint64]$Receipt.attempt -ne 1 -or [uint64]$Receipt.attempt_limit -ne 1 -or
        [string]$Receipt.outcome -cne [string]$Control.expected_outcome -or
        [string]$Receipt.failure_code -cne [string]$Control.expected_failure_code -or
        [string]$Receipt.dataset_transaction -cne [string]$Control.dataset_transaction -or
        [string]$Receipt.unauthorized_state -cne "none" -or
        [string]$Receipt.cross_owner_write -cne "none") {
        throw "Failure receipt for '$($Control.fault_key)' is not the exact bounded containment proof."
    }
    [pscustomobject][ordered]@{
        fault_key = [string]$Control.fault_key
        outcome = [string]$Receipt.outcome
        no_unauthorized_state = $true
        no_cross_owner_write = $true
        transaction = [string]$Receipt.dataset_transaction
    }
}

function Find-Sprint8BFaultReceiptInDiagnostics {
    param(
        [Parameter(Mandatory)]$MaterializationEvidence,
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $diagnostics = @($MaterializationEvidence.retained_diagnostics)
    if ($diagnostics.Count -eq 0) {
        throw "Failed materialization retained no diagnostic artifacts."
    }
    foreach ($entry in $diagnostics) {
        $path = Resolve-Sprint8BRepositoryPath -Path ([string]$entry.path)
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne
                [string]$entry.sha256) {
            throw "Retained materialization diagnostic '$($entry.path)' is missing or changed."
        }
        $text = Get-Content -LiteralPath $path -Raw
        $jsonCandidates = @()
        try { $jsonCandidates += ($text | ConvertFrom-Json -Depth 100) } catch {}
        foreach ($match in [regex]::Matches($text, '\{[^\r\n]+\}')) {
            try { $jsonCandidates += ([string]$match.Value | ConvertFrom-Json -Depth 100) } catch {}
        }
        foreach ($candidate in $jsonCandidates) {
            $possible = @($candidate)
            foreach ($pointer in @("fault_receipt", "error", "details")) {
                $expanded = [Collections.Generic.List[object]]::new()
                foreach ($value in $possible) {
                    $property = $value.PSObject.Properties[$pointer]
                    if ($null -ne $property) { $expanded.Add($property.Value) }
                }
                $possible += @($expanded)
            }
            foreach ($value in $possible) {
                if ($null -ne $value.PSObject.Properties['contract'] -and
                    [string]$value.contract -ceq $faultControlContract) {
                    return Assert-Sprint8BFaultReceipt -Receipt $value -Control $Control `
                        -CorrelationId $CorrelationId
                }
            }
        }
    }
    throw "Retained diagnostics do not contain the exact '$($Control.fault_key)' fault receipt."
}

function Assert-Sprint8BFailedMaterialization {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId,
        [switch]$SkipDiagnosticFileRead
    )

    if ([string]$Evidence.sprint -cne "sprint-8b" -or
        [string]$Evidence.state -cne "failed" -or
        [string]$Evidence.target -cne "Reference" -or
        [string]$Evidence.compose_project -cne $ComposeProject -or
        [string]$Evidence.cleanup_restoration.state -cne "passed" -or
        [string]$Evidence.cleanup_restoration.mode -cne "exact-project-teardown" -or
        [bool]$Evidence.gateway_start_boundary.owner_apply_completed_before_start -or
        $null -ne $Evidence.fixture_receipt_path -or
        $null -eq $Evidence.failure) {
        throw "Fault '$($Control.fault_key)' did not fail inside the private owner-apply boundary with exact teardown."
    }
    $containment = if ($SkipDiagnosticFileRead) {
        Assert-Sprint8BFaultReceipt -Receipt $Evidence.self_test_fault_receipt `
            -Control $Control -CorrelationId $CorrelationId
    } else {
        Find-Sprint8BFaultReceiptInDiagnostics -MaterializationEvidence $Evidence `
            -Control $Control -CorrelationId $CorrelationId
    }
    [pscustomobject][ordered]@{
        fault_key = [string]$Control.fault_key
        materialization_state = "failed"
        gateway_started = $false
        fixture_published = $false
        partial_topology_teardown = "passed"
        containment = $containment
    }
}

function Assert-Sprint8BSuccessorRestoration {
    param([Parameter(Mandatory)]$Evidence)

    if ([string]$Evidence.sprint -cne "sprint-8b" -or
        [string]$Evidence.state -cne "passed" -or
        [string]$Evidence.target -cne "ReferenceNoOp" -or
        [string]$Evidence.compose_project -cne $ComposeProject -or
        [string]$Evidence.first_apply.operation_state -cne "succeeded" -or
        [bool]$Evidence.first_apply.no_op -or -not [bool]$Evidence.first_apply.changed -or
        [string]$Evidence.semantic_noop.operation_state -cne "succeeded" -or
        -not [bool]$Evidence.semantic_noop.no_op -or [bool]$Evidence.semantic_noop.changed -or
        [string]$Evidence.semantic_noop_proof.state -cne "passed" -or
        [string]$Evidence.gateway_start_boundary.post_start_health -cne "passed" -or
        [string]$Evidence.cleanup_restoration.state -cne "passed" -or
        [string]$Evidence.cleanup_restoration.mode -cne "exact-project-teardown" -or
        [string]::IsNullOrWhiteSpace([string]$Evidence.fixture_receipt_path)) {
        throw "Failure successor did not prove from-empty first apply, exact no-op, canonical health, fixture identity, and teardown."
    }
    [pscustomobject][ordered]@{
        empty_start = "passed"
        canonical_first_apply = "passed"
        semantic_noop = "passed"
        fixture_receipt_path = [string]$Evidence.fixture_receipt_path
        canonical_health = "passed"
        final_teardown = "passed"
    }
}

function New-Sprint8BFaultSelfTestReceipt {
    param(
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId
    )
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = $faultControlContract
        fault_key = [string]$Control.fault_key
        correlation_id = $CorrelationId
        target_service = [string]$Control.target_service
        phase = [string]$Control.phase
        attempt = 1
        attempt_limit = 1
        outcome = [string]$Control.expected_outcome
        failure_code = [string]$Control.expected_failure_code
        dataset_transaction = [string]$Control.dataset_transaction
        unauthorized_state = "none"
        cross_owner_write = "none"
    }
}

function Test-Sprint8BFailureContainmentHarness {
    Read-Sprint8BFailureFixture | Out-Null
    $contract = Get-Sprint8BFailureControlContract
    $correlation = "01980000-00f0-7000-8000-000000000001"
    $mockServices = [ordered]@{}
    foreach ($control in @($contract.controls)) {
        $mockServices[[string]$control.target_service] = [pscustomobject]@{
            environment = [pscustomobject][ordered]@{
                TESSARA_SPRINT_8B_FAULT_CONTRACT = $faultControlContract
                TESSARA_SPRINT_8B_FAULT_SCOPE = $faultControlScope
                TESSARA_SPRINT_8B_FAULT_KEY = if ([string]$control.fault_key -ceq
                    "response.incompatible") { "response.incompatible" } else { "none" }
                TESSARA_SPRINT_8B_FAULT_CORRELATION_ID = $correlation
                TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT = "1"
            }
        }
    }
    $configuration = [pscustomobject]@{ services = [pscustomobject]$mockServices }
    Assert-Sprint8BFailureControlProjection -ComposeConfiguration $configuration `
        -ActiveFault "response.incompatible" -CorrelationId $correlation | Out-Null
    $tamperedConfiguration = $configuration | ConvertTo-Json -Depth 50 | ConvertFrom-Json -Depth 50
    $tamperedConfiguration.services.datasets.environment.TESSARA_SPRINT_8B_FAULT_KEY =
        "dataset.derived-rebuild"
    $rejected = $false
    try {
        Assert-Sprint8BFailureControlProjection -ComposeConfiguration $tamperedConfiguration `
            -ActiveFault "response.incompatible" -CorrelationId $correlation | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Failure-control self-test accepted two simultaneously armed faults." }

    foreach ($control in @($contract.controls)) {
        $receipt = New-Sprint8BFaultSelfTestReceipt -Control $control -CorrelationId $correlation
        $mockEvidence = [pscustomobject][ordered]@{
            sprint = "sprint-8b"
            state = "failed"
            target = "Reference"
            compose_project = $ComposeProject
            cleanup_restoration = [pscustomobject]@{
                state = "passed"; mode = "exact-project-teardown"
            }
            gateway_start_boundary = [pscustomobject]@{
                owner_apply_completed_before_start = $false
            }
            fixture_receipt_path = $null
            failure = [pscustomobject]@{ message = [string]$control.expected_failure_code }
            self_test_fault_receipt = $receipt
        }
        Assert-Sprint8BFailedMaterialization -Evidence $mockEvidence -Control $control `
            -CorrelationId $correlation -SkipDiagnosticFileRead | Out-Null
        $tampered = Copy-Sprint8BFailureJsonValue -Value $mockEvidence
        $tampered.self_test_fault_receipt.cross_owner_write = "detected"
        $rejected = $false
        try {
            Assert-Sprint8BFailedMaterialization -Evidence $tampered -Control $control `
                -CorrelationId $correlation -SkipDiagnosticFileRead | Out-Null
        } catch { $rejected = $true }
        if (-not $rejected) {
            throw "Failure-containment self-test accepted a cross-owner write."
        }
    }

    $successor = [pscustomobject][ordered]@{
        sprint = "sprint-8b"
        state = "passed"
        target = "ReferenceNoOp"
        compose_project = $ComposeProject
        first_apply = [pscustomobject]@{
            operation_state = "succeeded"; no_op = $false; changed = $true
        }
        semantic_noop = [pscustomobject]@{
            operation_state = "succeeded"; no_op = $true; changed = $false
        }
        semantic_noop_proof = [pscustomobject]@{ state = "passed" }
        gateway_start_boundary = [pscustomobject]@{ post_start_health = "passed" }
        cleanup_restoration = [pscustomobject]@{
            state = "passed"; mode = "exact-project-teardown"
        }
        fixture_receipt_path = "target/self-test/fixture.json"
    }
    Assert-Sprint8BSuccessorRestoration -Evidence $successor | Out-Null
    $successor.semantic_noop.no_op = $false
    $rejected = $false
    try { Assert-Sprint8BSuccessorRestoration -Evidence $successor | Out-Null } catch {
        $rejected = $true
    }
    if (-not $rejected) { throw "Failure-containment self-test accepted a non-no-op successor." }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "failure-containment-and-recovery-self-test"
        state = "passed"
        self_test = $true
        database_free = $true
        live_proof_claimed = $false
        compose_project = "tessara-s8b-failure-selftest"
        fault_control_contract = $contract
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text `
            "sprint-8b-failure-containment-self-test`n"
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"
            mode = "database-free-self-test"
        }
    }
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8BHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result
}

if ($SelfTest) {
    Test-Sprint8BFailureContainmentHarness | ConvertTo-Json -Depth 100
    return
}

Assert-Sprint8BResetAuthorization -ComposeProject $ComposeProject `
    -Authorized ([bool]$AuthorizeDisposableReset)
$source = Get-Sprint8BSourceIdentity -RequireClean
$fixture = Read-Sprint8BFailureFixture
$controlContract = Get-Sprint8BFailureControlContract
$composePath = Resolve-Sprint8BRepositoryPath -Path $ComposeFile
if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) {
    throw "Sprint 8B failure-containment Compose file is missing: $composePath"
}

$allEnvironmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT"
) + $faultEnvironmentNames
$environmentBefore = Get-Sprint8BProcessEnvironmentSnapshot -Names $allEnvironmentNames
$runtimeRoot = Resolve-Sprint8BRepositoryPath -Path (
    "target/sprint-8b-failure-containment/$ComposeProject-$([Guid]::NewGuid().ToString('N'))"
)
[IO.Directory]::CreateDirectory($runtimeRoot) | Out-Null
$ports = $null
$configurationProofs = [Collections.Generic.List[object]]::new()
$attempts = [Collections.Generic.List[object]]::new()
$successorEvidence = $null
$successorProof = $null
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$failure = $null

try {
    $ports = Set-Sprint8BComposeEnvironment -ComposeProject $ComposeProject
    $preexisting = Get-Sprint8BProjectResources -ComposeProject $ComposeProject
    if ($preexisting.containers.Count -ne 0 -or $preexisting.volumes.Count -ne 0 -or
        $preexisting.networks.Count -ne 0) {
        Remove-Sprint8BProjectTopology -ComposePath $composePath `
            -ComposeProject $ComposeProject -Authorized $true | Out-Null
    }
    Assert-Sprint8BProjectAbsent -ComposeProject $ComposeProject | Out-Null

    $index = 0
    foreach ($control in @($controlContract.controls)) {
        $index++
        $correlation = [Guid]::NewGuid().ToString()
        Set-Sprint8BFailureEnvironment -FaultKey ([string]$control.fault_key) `
            -CorrelationId $correlation
        $configuration = Get-Sprint8BComposeConfiguration -ComposePath $composePath `
            -ComposeProject $ComposeProject
        Assert-Sprint8BDatabaseIsolationConfiguration -ComposeConfiguration $configuration | Out-Null
        $configurationProofs.Add((Assert-Sprint8BFailureControlProjection `
            -ComposeConfiguration $configuration -ActiveFault ([string]$control.fault_key) `
            -CorrelationId $correlation))

        $faultName = ([string]$control.fault_key).Replace('.', '-')
        $materializationPath = Join-Path $runtimeRoot "$faultName-materialization.json"
        $arguments = @(
            "-Target", "Reference", "-ComposeProject", $ComposeProject,
            "-EvidencePath", $materializationPath, "-AuthorizeDisposableReset"
        )
        if ($SkipBuild -or $index -gt 1) { $arguments += "-SkipBuild" }
        $child = Invoke-Sprint8BChildScript -ScriptPath "scripts/materialize-sprint-8b.ps1" `
            -Arguments $arguments -AllowFailure
        if ($child.exit_code -eq 0) {
            throw "Armed fault '$($control.fault_key)' did not fail materialization."
        }
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $materializationPath `
            -SidecarPath "$materializationPath.sha256")) {
            throw "Fault '$($control.fault_key)' did not retain authenticated materialization evidence."
        }
        $materialization = Get-Content -LiteralPath $materializationPath -Raw | ConvertFrom-Json -Depth 100
        $containment = Assert-Sprint8BFailedMaterialization -Evidence $materialization `
            -Control $control -CorrelationId $correlation
        Assert-Sprint8BProjectAbsent -ComposeProject $ComposeProject | Out-Null
        $attempts.Add([pscustomobject][ordered]@{
            fault = $control
            correlation_id = $correlation
            materialization_evidence = [pscustomobject][ordered]@{
                path = $materializationPath
                sha256 = (Get-FileHash -LiteralPath $materializationPath -Algorithm SHA256).Hash.ToLowerInvariant()
            }
            containment = $containment
            child_exit_code = $child.exit_code
        })
    }

    Clear-Sprint8BFailureEnvironment
    $clearedConfiguration = Get-Sprint8BComposeConfiguration -ComposePath $composePath `
        -ComposeProject $ComposeProject
    foreach ($control in @($controlContract.controls)) {
        $environment = Get-Sprint8BServiceEnvironment -ComposeConfiguration $clearedConfiguration `
            -Service ([string]$control.target_service)
        if ([string]$environment.TESSARA_SPRINT_8B_FAULT_KEY -cne "none" -or
            [string]$environment.TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT -cne "0") {
            throw "Failure control '$($control.fault_key)' did not restore to off before successor apply."
        }
    }
    Assert-Sprint8BProjectAbsent -ComposeProject $ComposeProject | Out-Null
    $successorPath = Join-Path $runtimeRoot "successor-materialization.json"
    $successorArguments = @(
        "-Target", "ReferenceNoOp", "-ComposeProject", $ComposeProject,
        "-EvidencePath", $successorPath, "-AuthorizeDisposableReset", "-SkipBuild"
    )
    Invoke-Sprint8BChildScript -ScriptPath "scripts/materialize-sprint-8b.ps1" `
        -Arguments $successorArguments | Out-Null
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $successorPath `
        -SidecarPath "$successorPath.sha256")) {
        throw "Failure successor did not publish authenticated materialization evidence."
    }
    $successorEvidence = Get-Content -LiteralPath $successorPath -Raw | ConvertFrom-Json -Depth 100
    $successorProof = Assert-Sprint8BSuccessorRestoration -Evidence $successorEvidence
    Assert-Sprint8BProjectAbsent -ComposeProject $ComposeProject | Out-Null
    $cleanup = [pscustomobject][ordered]@{
        state = "passed"
        mode = "two-exact-partial-teardowns-plus-restored-successor-teardown"
        fault_controls = "cleared-to-none"
        empty_successor_start = "passed"
    }
} catch {
    $failure = $_
} finally {
    try {
        if ($null -ne $ports) {
            $remaining = Get-Sprint8BProjectResources -ComposeProject $ComposeProject
            if ($remaining.containers.Count -ne 0 -or $remaining.volumes.Count -ne 0 -or
                $remaining.networks.Count -ne 0) {
                Remove-Sprint8BProjectTopology -ComposePath $composePath `
                    -ComposeProject $ComposeProject -Authorized $true | Out-Null
            }
            Assert-Sprint8BProjectAbsent -ComposeProject $ComposeProject | Out-Null
            if ([string]$cleanup.state -cne "passed") {
                $cleanup = [pscustomobject][ordered]@{
                    state = "passed"
                    mode = "failure-path-exact-project-teardown"
                    fault_controls = "process-environment-restored"
                }
            }
        }
    } catch {
        if ($null -eq $failure) { $failure = $_ }
        $cleanup = [pscustomobject][ordered]@{
            state = "failed"
            error = $_.Exception.Message
        }
    } finally {
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n" +
    "$(Get-Sprint7ASha256 -Text (($controlContract | ConvertTo-Json -Depth 100 -Compress) + "`n"))`n" +
    "$(Get-Sprint7ASha256 -Text (($fixture | ConvertTo-Json -Depth 100 -Compress) + "`n"))`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    proof = "deterministic-failure-containment-retry-and-restoration"
    state = if ($null -eq $failure -and $attempts.Count -eq 2 -and
        $null -ne $successorProof -and [string]$cleanup.state -ceq "passed") {
        "passed"
    } else { "failed" }
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    fault_control_contract = $controlContract
    fault_fixture_sha256 = (Get-FileHash -LiteralPath `
        (Resolve-Sprint8BRepositoryPath -Path $FaultFixturePath) -Algorithm SHA256).Hash.ToLowerInvariant()
    control_projection_proofs = @($configurationProofs)
    failure_attempts = @($attempts)
    successor_evidence = if ($null -eq $successorEvidence) { $null } else {
        [pscustomobject][ordered]@{
            state = [string]$successorEvidence.state
            target = [string]$successorEvidence.target
            environment_fingerprint_sha256 = [string]$successorEvidence.environment_fingerprint_sha256
            fixture_receipt_path = [string]$successorEvidence.fixture_receipt_path
        }
    }
    restoration_proof = $successorProof
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
        message = $failure.Exception.Message
        category = [string]$failure.CategoryInfo.Category
        live_proof_claimed = $false
    } }
}
$evidenceFullPath = Resolve-Sprint8BRepositoryPath -Path $EvidencePath
Publish-Sprint8BHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8B failure containment is not live-ready; retained evidence: $evidenceFullPath"
}

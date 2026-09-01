[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8c-implementation-recovery",
    [string]$ComposeFile = "deploy/sprint-8c/compose.yaml",
    [string]$FaultFixturePath = "deploy/sprint-8c/fixtures/provider-fault-contract.json",
    [string]$EvidencePath = "target/sprint-8c-failure-containment/result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidencePathWasExplicit = $PSBoundParameters.ContainsKey("EvidencePath")
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")
$SelfTest = $requestedSelfTest

$faultControlContract = "tessara.sprint-8c.failure-control/v1"
$faultControlScope = "disposable-sprint-8c-only"
$inheritedFaultControlContract = "tessara.sprint-8b.failure-control/v1"
$inheritedFaultControlScope = "disposable-sprint-8b-only"
$faultEnvironmentNames = @(
    "TESSARA_SPRINT_8C_RESPONSE_OWNER_FAULT_KEY",
    "TESSARA_SPRINT_8C_FAULT_CORRELATION_ID",
    "TESSARA_SPRINT_8C_FAULT_ATTEMPT_LIMIT",
    "TESSARA_SPRINT_8B_RESPONSE_FAULT_KEY",
    "TESSARA_SPRINT_8B_DATASET_FAULT_KEY",
    "TESSARA_SPRINT_8B_FAULT_CORRELATION_ID",
    "TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT"
)

function Copy-Sprint8CFailureJsonValue {
    param([Parameter(Mandatory)]$Value)
    $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
}

function Get-Sprint8CFailureControlContract {
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = $faultControlContract
        scope = $faultControlScope
        default_state = "off"
        attempt_limit = 1
        controls = @(
            [pscustomobject][ordered]@{
                fault_key = "response.bootstrap.mid-apply"
                target_service = "responses"
                host_environment = "TESSARA_SPRINT_8C_RESPONSE_OWNER_FAULT_KEY"
                service_contract_environment = "TESSARA_SPRINT_8C_FAULT_CONTRACT"
                service_scope_environment = "TESSARA_SPRINT_8C_FAULT_SCOPE"
                service_environment = "TESSARA_SPRINT_8C_FAULT_KEY"
                service_correlation_environment = "TESSARA_SPRINT_8C_FAULT_CORRELATION_ID"
                service_attempt_limit_environment = "TESSARA_SPRINT_8C_FAULT_ATTEMPT_LIMIT"
                receipt_contract = $faultControlContract
                receipt_scope = $faultControlScope
                mode = "deterministic_mid_apply_failure"
                phase = "response_bootstrap_transaction"
                expected_outcome = "rolled_back"
                expected_failure_code = "response.bootstrap.injected_failure"
                transaction_field = "response_transaction"
                transaction_value = "rolled_back"
            },
            [pscustomobject][ordered]@{
                fault_key = "response.incompatible"
                target_service = "response-provider-proxy"
                host_environment = "TESSARA_SPRINT_8B_RESPONSE_FAULT_KEY"
                service_contract_environment = "TESSARA_SPRINT_8B_FAULT_CONTRACT"
                service_scope_environment = "TESSARA_SPRINT_8B_FAULT_SCOPE"
                service_environment = "TESSARA_SPRINT_8B_FAULT_KEY"
                service_correlation_environment = "TESSARA_SPRINT_8B_FAULT_CORRELATION_ID"
                service_attempt_limit_environment = "TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT"
                receipt_contract = $inheritedFaultControlContract
                receipt_scope = $inheritedFaultControlScope
                mode = "incompatible_version"
                phase = "dataset_bootstrap_provider_validation"
                expected_outcome = "rejected_pre_write"
                expected_failure_code = "dataset.dependency_incompatible"
                transaction_field = "dataset_transaction"
                transaction_value = "not_started"
            },
            [pscustomobject][ordered]@{
                fault_key = "dataset.derived-rebuild"
                target_service = "datasets"
                host_environment = "TESSARA_SPRINT_8B_DATASET_FAULT_KEY"
                service_contract_environment = "TESSARA_SPRINT_8B_FAULT_CONTRACT"
                service_scope_environment = "TESSARA_SPRINT_8B_FAULT_SCOPE"
                service_environment = "TESSARA_SPRINT_8B_FAULT_KEY"
                service_correlation_environment = "TESSARA_SPRINT_8B_FAULT_CORRELATION_ID"
                service_attempt_limit_environment = "TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT"
                receipt_contract = $inheritedFaultControlContract
                receipt_scope = $inheritedFaultControlScope
                mode = "deterministic_rebuild_failure"
                phase = "dataset_bootstrap_transaction"
                expected_outcome = "rolled_back"
                expected_failure_code = "dataset.dependency_unavailable"
                transaction_field = "dataset_transaction"
                transaction_value = "rolled_back"
            }
        )
        receipt_contracts = @(
            [pscustomobject][ordered]@{
                contract = $faultControlContract
                scope = $faultControlScope
                owner = "tessara.responses"
            },
            [pscustomobject][ordered]@{
                contract = $inheritedFaultControlContract
                scope = $inheritedFaultControlScope
                owner = "retained-sprint-8b-provider-and-dataset-controls"
            }
        )
        required_common_fault_receipt_fields = @(
            "schema_version", "contract", "fault_key", "correlation_id", "target_service",
            "phase", "attempt", "attempt_limit", "outcome", "failure_code",
            "unauthorized_state", "cross_owner_write"
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

function Read-Sprint8CFailureFixture {
    $path = Resolve-Sprint8CRepositoryPath -Path $FaultFixturePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Sprint 8C fault fixture is missing: $path"
    }
    $fixture = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -Depth 100
    if ([int]$fixture.schema_version -ne 1 -or
        [string]$fixture.contract -cne "tessara.sprint-8c.response-provider-faults" -or
        [string]$fixture.isolation -cne "one-binding-per-proxy" -or
        [bool]$fixture.product_bypass_forbidden -ne $true -or
        [bool]$fixture.database_bypass_forbidden -ne $true) {
        throw "Sprint 8C fault fixture does not enforce isolated owner-boundary failure injection."
    }
    foreach ($key in @(
        "response.bootstrap.mid-apply", "response.incompatible", "dataset.derived-rebuild"
    )) {
        if (@($fixture.faults | Where-Object { [string]$_.key -ceq $key }).Count -ne 1) {
            throw "Sprint 8C fault fixture omits exact required fault '$key'."
        }
    }
    $fixture
}

function Get-Sprint8CServiceEnvironment {
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

function Get-Sprint8CFailurePropertyValue {
    param(
        [Parameter(Mandatory)]$Object,
        [Parameter(Mandatory)][string]$Name
    )

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        throw "Failure-control projection is missing required property '$Name'."
    }
    $property.Value
}

function Assert-Sprint8CFailureControlProjection {
    param(
        [Parameter(Mandatory)]$ComposeConfiguration,
        [Parameter(Mandatory)][string]$ActiveFault,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $contract = Get-Sprint8CFailureControlContract
    $active = @($contract.controls | Where-Object { [string]$_.fault_key -ceq $ActiveFault })
    if ($active.Count -ne 1) { throw "Unknown Sprint 8C failure control '$ActiveFault'." }
    foreach ($control in @($contract.controls)) {
        $environment = Get-Sprint8CServiceEnvironment -ComposeConfiguration $ComposeConfiguration `
            -Service ([string]$control.target_service)
        $expectedKey = if ([string]$control.fault_key -ceq $ActiveFault) {
            [string]$control.fault_key
        } else { "none" }
        $contractValue = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_contract_environment)
        $scopeValue = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_scope_environment)
        $keyValue = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_environment)
        $correlationValue = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_correlation_environment)
        $attemptValue = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_attempt_limit_environment)
        if ([string]$contractValue -cne [string]$control.receipt_contract -or
            [string]$scopeValue -cne [string]$control.receipt_scope -or
            [string]$keyValue -cne $expectedKey -or
            [string]$correlationValue -cne $CorrelationId -or
            [string]$attemptValue -cne "1") {
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

function Set-Sprint8CFailureEnvironment {
    param(
        [Parameter(Mandatory)][string]$FaultKey,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $contract = Get-Sprint8CFailureControlContract
    foreach ($control in @($contract.controls)) {
        [Environment]::SetEnvironmentVariable(
            [string]$control.host_environment,
            $(if ([string]$control.fault_key -ceq $FaultKey) { $FaultKey } else { "none" }),
            "Process"
        )
    }
    $env:TESSARA_SPRINT_8C_FAULT_CORRELATION_ID = $CorrelationId
    $env:TESSARA_SPRINT_8C_FAULT_ATTEMPT_LIMIT = "1"
    $env:TESSARA_SPRINT_8B_FAULT_CORRELATION_ID = $CorrelationId
    $env:TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT = "1"
}

function Clear-Sprint8CFailureEnvironment {
    foreach ($control in @((Get-Sprint8CFailureControlContract).controls)) {
        [Environment]::SetEnvironmentVariable(
            [string]$control.host_environment,
            "none",
            "Process"
        )
    }
    $env:TESSARA_SPRINT_8C_FAULT_CORRELATION_ID = "00000000-0000-0000-0000-000000000000"
    $env:TESSARA_SPRINT_8C_FAULT_ATTEMPT_LIMIT = "0"
    $env:TESSARA_SPRINT_8B_FAULT_CORRELATION_ID = "00000000-0000-0000-0000-000000000000"
    $env:TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT = "0"
}

function Assert-Sprint8CFaultReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId
    )

    $actualFields = @($Receipt.PSObject.Properties.Name | Sort-Object)
    $expectedFields = @(
        @((Get-Sprint8CFailureControlContract).required_common_fault_receipt_fields) +
        [string]$Control.transaction_field
    ) | Sort-Object
    $transactionValue = Get-Sprint8CFailurePropertyValue -Object $Receipt `
        -Name ([string]$Control.transaction_field)
    if (($actualFields -join "`n") -cne ($expectedFields -join "`n") -or
        [int]$Receipt.schema_version -ne 1 -or
        [string]$Receipt.contract -cne [string]$Control.receipt_contract -or
        [string]$Receipt.fault_key -cne [string]$Control.fault_key -or
        [string]$Receipt.correlation_id -cne $CorrelationId -or
        [string]$Receipt.target_service -cne [string]$Control.target_service -or
        [string]$Receipt.phase -cne [string]$Control.phase -or
        [uint64]$Receipt.attempt -ne 1 -or [uint64]$Receipt.attempt_limit -ne 1 -or
        [string]$Receipt.outcome -cne [string]$Control.expected_outcome -or
        [string]$Receipt.failure_code -cne [string]$Control.expected_failure_code -or
        [string]$transactionValue -cne [string]$Control.transaction_value -or
        [string]$Receipt.unauthorized_state -cne "none" -or
        [string]$Receipt.cross_owner_write -cne "none") {
        throw "Failure receipt for '$($Control.fault_key)' is not the exact bounded containment proof."
    }
    [pscustomobject][ordered]@{
        fault_key = [string]$Control.fault_key
        receipt_contract = [string]$Receipt.contract
        attempt = [uint64]$Receipt.attempt
        attempt_limit = [uint64]$Receipt.attempt_limit
        outcome = [string]$Receipt.outcome
        no_unauthorized_state = $true
        no_cross_owner_write = $true
        transaction_field = [string]$Control.transaction_field
        transaction_value = [string]$transactionValue
    }
}

function Find-Sprint8CFaultReceiptInDiagnostics {
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
        $path = Resolve-Sprint8CRepositoryPath -Path ([string]$entry.path)
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
                    $null -ne $value.PSObject.Properties['fault_key'] -and
                    [string]$value.contract -ceq [string]$Control.receipt_contract -and
                    [string]$value.fault_key -ceq [string]$Control.fault_key) {
                    return Assert-Sprint8CFaultReceipt -Receipt $value -Control $Control `
                        -CorrelationId $CorrelationId
                }
            }
        }
    }
    throw "Retained diagnostics do not contain the exact '$($Control.fault_key)' fault receipt."
}

function Assert-Sprint8CFailedMaterialization {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId,
        [switch]$SkipDiagnosticFileRead
    )

    if ([string]$Evidence.sprint -cne "sprint-8c" -or
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
    $faultReceipt = if ($SkipDiagnosticFileRead) {
        Assert-Sprint8CFaultReceipt -Receipt $Evidence.self_test_fault_receipt `
            -Control $Control -CorrelationId $CorrelationId
    } else {
        Find-Sprint8CFaultReceiptInDiagnostics -MaterializationEvidence $Evidence `
            -Control $Control -CorrelationId $CorrelationId
    }
    [pscustomobject][ordered]@{
        fault = Get-Sprint8CFaultSemanticProjection -Control $Control `
            -Containment $faultReceipt
        containment = [pscustomobject][ordered]@{
            fault_key = [string]$Control.fault_key
            materialization_state = "failed"
            gateway_started = $false
            fixture_published = $false
            partial_topology_teardown = "passed"
        }
    }
}

function Assert-Sprint8CSuccessorRestoration {
    param([Parameter(Mandatory)]$Evidence)

    if ([string]$Evidence.sprint -cne "sprint-8c" -or
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

function New-Sprint8CFaultSelfTestReceipt {
    param(
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)][string]$CorrelationId
    )
    $receipt = [ordered]@{
        schema_version = 1
        contract = [string]$Control.receipt_contract
        fault_key = [string]$Control.fault_key
        correlation_id = $CorrelationId
        target_service = [string]$Control.target_service
        phase = [string]$Control.phase
        attempt = 1
        attempt_limit = 1
        outcome = [string]$Control.expected_outcome
        failure_code = [string]$Control.expected_failure_code
        unauthorized_state = "none"
        cross_owner_write = "none"
    }
    $receipt[[string]$Control.transaction_field] = [string]$Control.transaction_value
    [pscustomobject]$receipt
}

function Get-Sprint8CFaultSemanticProjection {
    param(
        [Parameter(Mandatory)]$Control,
        [Parameter(Mandatory)]$Containment
    )

    [pscustomobject][ordered]@{
        fault_key = [string]$Control.fault_key
        receipt_contract = [string]$Control.receipt_contract
        target_service = [string]$Control.target_service
        phase = [string]$Control.phase
        expected_outcome = [string]$Control.expected_outcome
        expected_failure_code = [string]$Control.expected_failure_code
        attempt = [uint64]$Containment.attempt
        attempt_limit = [uint64]$Containment.attempt_limit
        no_unauthorized_state = [bool]$Containment.no_unauthorized_state
        no_cross_owner_write = [bool]$Containment.no_cross_owner_write
        transaction_field = [string]$Containment.transaction_field
        transaction_value = [string]$Containment.transaction_value
    }
}

function Test-Sprint8CFailureContainmentHarness {
    Read-Sprint8CFailureFixture | Out-Null
    $contract = Get-Sprint8CFailureControlContract
    $correlation = "01980000-00f0-7000-8000-000000000001"
    $mockServices = [ordered]@{}
    foreach ($control in @($contract.controls)) {
        $environment = [ordered]@{}
        $environment[[string]$control.service_contract_environment] =
            [string]$control.receipt_contract
        $environment[[string]$control.service_scope_environment] = [string]$control.receipt_scope
        $environment[[string]$control.service_environment] = if (
            [string]$control.fault_key -ceq "response.bootstrap.mid-apply"
        ) { [string]$control.fault_key } else { "none" }
        $environment[[string]$control.service_correlation_environment] = $correlation
        $environment[[string]$control.service_attempt_limit_environment] = "1"
        $mockServices[[string]$control.target_service] = [pscustomobject]@{
            environment = [pscustomobject]$environment
        }
    }
    $configuration = [pscustomobject]@{ services = [pscustomobject]$mockServices }
    Assert-Sprint8CFailureControlProjection -ComposeConfiguration $configuration `
        -ActiveFault "response.bootstrap.mid-apply" -CorrelationId $correlation | Out-Null
    $tamperedConfiguration = $configuration | ConvertTo-Json -Depth 50 | ConvertFrom-Json -Depth 50
    $datasetControl = @($contract.controls | Where-Object {
        [string]$_.fault_key -ceq "dataset.derived-rebuild"
    })[0]
    $tamperedConfiguration.services.datasets.environment.PSObject.Properties[
        [string]$datasetControl.service_environment
    ].Value = "dataset.derived-rebuild"
    $rejected = $false
    try {
        Assert-Sprint8CFailureControlProjection -ComposeConfiguration $tamperedConfiguration `
            -ActiveFault "response.bootstrap.mid-apply" -CorrelationId $correlation | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Failure-control self-test accepted two simultaneously armed faults." }

    foreach ($control in @($contract.controls)) {
        $receipt = New-Sprint8CFaultSelfTestReceipt -Control $control -CorrelationId $correlation
        $mockEvidence = [pscustomobject][ordered]@{
            sprint = "sprint-8c"
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
        $attemptProof = Assert-Sprint8CFailedMaterialization -Evidence $mockEvidence `
            -Control $control -CorrelationId $correlation -SkipDiagnosticFileRead
        $expectedFaultFields = @(
            "attempt", "attempt_limit", "expected_failure_code", "expected_outcome",
            "fault_key", "no_cross_owner_write", "no_unauthorized_state", "phase",
            "receipt_contract", "target_service", "transaction_field", "transaction_value"
        )
        $expectedContainmentFields = @(
            "fault_key", "fixture_published", "gateway_started", "materialization_state",
            "partial_topology_teardown"
        )
        if ((@($attemptProof.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                (@("containment", "fault") -join "`n") -or
            (@($attemptProof.fault.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                ($expectedFaultFields -join "`n") -or
            (@($attemptProof.containment.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                ($expectedContainmentFields -join "`n") -or
            [string]$attemptProof.fault.fault_key -cne [string]$control.fault_key -or
            [string]$attemptProof.containment.fault_key -cne [string]$control.fault_key -or
            [uint64]$attemptProof.fault.attempt -ne 1 -or
            [uint64]$attemptProof.fault.attempt_limit -ne 1) {
            throw "Failure-containment self-test did not produce the exact sibling fault and topology-only containment receipt."
        }
        $tampered = Copy-Sprint8CFailureJsonValue -Value $mockEvidence
        $tampered.self_test_fault_receipt.cross_owner_write = "detected"
        $rejected = $false
        try {
            Assert-Sprint8CFailedMaterialization -Evidence $tampered -Control $control `
                -CorrelationId $correlation -SkipDiagnosticFileRead | Out-Null
        } catch { $rejected = $true }
        if (-not $rejected) {
            throw "Failure-containment self-test accepted a cross-owner write."
        }
    }

    $successor = [pscustomobject][ordered]@{
        sprint = "sprint-8c"
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
    Assert-Sprint8CSuccessorRestoration -Evidence $successor | Out-Null
    $successor.semantic_noop.no_op = $false
    $rejected = $false
    try { Assert-Sprint8CSuccessorRestoration -Evidence $successor | Out-Null } catch {
        $rejected = $true
    }
    if (-not $rejected) { throw "Failure-containment self-test accepted a non-no-op successor." }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "failure-containment-and-recovery-self-test"
        state = "passed"
        self_test = $true
        database_free = $true
        live_proof_claimed = $false
        compose_project = "tessara-s8c-failure-selftest"
        fault_control_contract = $contract
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text `
            "sprint-8c-failure-containment-self-test`n"
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"
            mode = "database-free-self-test"
        }
    }
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8CHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result
}

if ($SelfTest) {
    Test-Sprint8CFailureContainmentHarness | ConvertTo-Json -Depth 100
    return
}

Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject `
    -Authorized ([bool]$AuthorizeDisposableReset)
$source = Get-Sprint8CSourceIdentity -RequireClean
$fixture = Read-Sprint8CFailureFixture
$controlContract = Get-Sprint8CFailureControlContract
$composePath = Resolve-Sprint8CRepositoryPath -Path $ComposeFile
if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) {
    throw "Sprint 8C failure-containment Compose file is missing: $composePath"
}

$allEnvironmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT"
) + $faultEnvironmentNames
$environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $allEnvironmentNames
$runtimeRoot = Resolve-Sprint8CRepositoryPath -Path (
    "target/sprint-8c-failure-containment/$ComposeProject-$([Guid]::NewGuid().ToString('N'))"
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
    $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject
    $preexisting = Get-Sprint8CProjectResources -ComposeProject $ComposeProject
    if ($preexisting.containers.Count -ne 0 -or $preexisting.volumes.Count -ne 0 -or
        $preexisting.networks.Count -ne 0) {
        Remove-Sprint8CProjectTopology -ComposePath $composePath `
            -ComposeProject $ComposeProject -Authorized $true | Out-Null
    }
    Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject | Out-Null

    $index = 0
    foreach ($control in @($controlContract.controls)) {
        $index++
        $correlation = [Guid]::NewGuid().ToString()
        Set-Sprint8CFailureEnvironment -FaultKey ([string]$control.fault_key) `
            -CorrelationId $correlation
        $configuration = Get-Sprint8CComposeConfiguration -ComposePath $composePath `
            -ComposeProject $ComposeProject
        Assert-Sprint8CDatabaseIsolationConfiguration -ComposeConfiguration $configuration | Out-Null
        $configurationProofs.Add((Assert-Sprint8CFailureControlProjection `
            -ComposeConfiguration $configuration -ActiveFault ([string]$control.fault_key) `
            -CorrelationId $correlation))

        $faultName = ([string]$control.fault_key).Replace('.', '-')
        $materializationPath = Join-Path $runtimeRoot "$faultName-materialization.json"
        $arguments = @(
            "-Target", "Reference", "-ComposeProject", $ComposeProject,
            "-EvidencePath", $materializationPath, "-AuthorizeDisposableReset"
        )
        if ($SkipBuild -or $index -gt 1) { $arguments += "-SkipBuild" }
        $child = Invoke-Sprint8CChildScript -ScriptPath "scripts/materialize-sprint-8c.ps1" `
            -Arguments $arguments -AllowFailure
        if ($child.exit_code -eq 0) {
            throw "Armed fault '$($control.fault_key)' did not fail materialization."
        }
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $materializationPath `
            -SidecarPath "$materializationPath.sha256")) {
            throw "Fault '$($control.fault_key)' did not retain authenticated materialization evidence."
        }
        $materialization = Get-Content -LiteralPath $materializationPath -Raw | ConvertFrom-Json -Depth 100
        $attemptProof = Assert-Sprint8CFailedMaterialization -Evidence $materialization `
            -Control $control -CorrelationId $correlation
        $materializationEvidenceSha256 = (Get-FileHash -LiteralPath $materializationPath `
            -Algorithm SHA256).Hash.ToLowerInvariant()
        Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject | Out-Null
        $attempts.Add([pscustomobject][ordered]@{
            fault = $attemptProof.fault
            correlation_id = $correlation
            materialization_evidence_sha256 = $materializationEvidenceSha256
            materialization_evidence = [pscustomobject][ordered]@{
                path = $materializationPath
                sha256 = $materializationEvidenceSha256
            }
            containment = $attemptProof.containment
            child_exit_code = $child.exit_code
        })
    }

    Clear-Sprint8CFailureEnvironment
    $clearedConfiguration = Get-Sprint8CComposeConfiguration -ComposePath $composePath `
        -ComposeProject $ComposeProject
    foreach ($control in @($controlContract.controls)) {
        $environment = Get-Sprint8CServiceEnvironment -ComposeConfiguration $clearedConfiguration `
            -Service ([string]$control.target_service)
        $clearedKey = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_environment)
        $clearedLimit = Get-Sprint8CFailurePropertyValue -Object $environment `
            -Name ([string]$control.service_attempt_limit_environment)
        if ([string]$clearedKey -cne "none" -or [string]$clearedLimit -cne "0") {
            throw "Failure control '$($control.fault_key)' did not restore to off before successor apply."
        }
    }
    Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject | Out-Null
    $successorPath = Join-Path $runtimeRoot "successor-materialization.json"
    $successorArguments = @(
        "-Target", "ReferenceNoOp", "-ComposeProject", $ComposeProject,
        "-EvidencePath", $successorPath, "-AuthorizeDisposableReset", "-SkipBuild"
    )
    Invoke-Sprint8CChildScript -ScriptPath "scripts/materialize-sprint-8c.ps1" `
        -Arguments $successorArguments | Out-Null
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $successorPath `
        -SidecarPath "$successorPath.sha256")) {
        throw "Failure successor did not publish authenticated materialization evidence."
    }
    $successorEvidence = Get-Content -LiteralPath $successorPath -Raw | ConvertFrom-Json -Depth 100
    $successorProof = Assert-Sprint8CSuccessorRestoration -Evidence $successorEvidence
    Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject | Out-Null
    $cleanup = [pscustomobject][ordered]@{
        state = "passed"
        mode = "three-exact-partial-teardowns-plus-restored-successor-teardown"
        fault_controls = "cleared-to-none"
        empty_successor_start = "passed"
    }
} catch {
    $failure = $_
} finally {
    try {
        if ($null -ne $ports) {
            $remaining = Get-Sprint8CProjectResources -ComposeProject $ComposeProject
            if ($remaining.containers.Count -ne 0 -or $remaining.volumes.Count -ne 0 -or
                $remaining.networks.Count -ne 0) {
                Remove-Sprint8CProjectTopology -ComposePath $composePath `
                    -ComposeProject $ComposeProject -Authorized $true | Out-Null
            }
            Assert-Sprint8CProjectAbsent -ComposeProject $ComposeProject | Out-Null
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
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n" +
    "$(Get-Sprint7ASha256 -Text (($controlContract | ConvertTo-Json -Depth 100 -Compress) + "`n"))`n" +
    "$(Get-Sprint7ASha256 -Text (($fixture | ConvertTo-Json -Depth 100 -Compress) + "`n"))`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    proof = "deterministic-failure-containment-retry-and-restoration"
    state = if ($null -eq $failure -and $attempts.Count -eq 3 -and
        $null -ne $successorProof -and [string]$cleanup.state -ceq "passed") {
        "passed"
    } else { "failed" }
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    fault_control_contract = $controlContract
    fault_fixture_sha256 = (Get-FileHash -LiteralPath `
        (Resolve-Sprint8CRepositoryPath -Path $FaultFixturePath) -Algorithm SHA256).Hash.ToLowerInvariant()
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
$evidenceFullPath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8C failure containment is not live-ready; retained evidence: $evidenceFullPath"
}

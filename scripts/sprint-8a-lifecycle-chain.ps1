Set-StrictMode -Version Latest

if ($PSVersionTable.PSEdition -cne "Core" -or $PSVersionTable.PSVersion.Major -lt 7) {
    throw "Sprint 8A lifecycle commands require PowerShell 7 or newer."
}

. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")

function Open-Sprint8AValidationAttemptLock {
    param([Parameter(Mandatory)][string]$Path)

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    [IO.File]::Open($Path, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
}

function Test-Sprint8ALifecycleExclusiveLock {
    $path = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-lifecycle-lock-$([guid]::NewGuid().ToString('N')).lock"
    $first = Open-Sprint8AValidationAttemptLock -Path $path
    try {
        $rejected = $false
        try {
            $second = Open-Sprint8AValidationAttemptLock -Path $path
            $second.Dispose()
        } catch [IO.IOException] { $rejected = $true }
        if (-not $rejected) { throw "Sprint 8A lifecycle lock admitted a concurrent process." }
    } finally {
        $first.Dispose()
    }
    $reopened = Open-Sprint8AValidationAttemptLock -Path $path
    $reopened.Dispose()
    Remove-Item -LiteralPath $path -Force
}

function Get-Sprint8APreflightCheckNames {
    @(
        "receipt-chain",
        "repository-scope",
        "clean-source",
        "acceptance-traceability",
        "environment-contract",
        "database-contract",
        "deployment-contract",
        "downstream-command-contract",
        "evidence-path-contract",
        "evidence-inventory"
    )
}

function Get-Sprint8ASitLaneNames {
    @("static-and-boundaries", "rust-workspace", "playwright", "deployed-acceptance-smoke")
}

function Get-Sprint8AManualUatScenarioNames {
    @(1..8 | ForEach-Object { "UAT-8A-{0:d2}" -f $_ })
}

function Assert-Sprint8AManualUatEvidenceCardinality {
    param(
        [Parameter(Mandatory)]$Requirement,
        [Parameter(Mandatory)][string]$Label
    )

    $minimum = 0
    $maximum = 0
    if (-not [int]::TryParse([string]$Requirement.minimum, [ref]$minimum) -or
        -not [int]::TryParse([string]$Requirement.maximum, [ref]$maximum) -or
        $minimum -ne 1 -or $maximum -ne 1) {
        throw "$Label must declare exactly one evidence artifact (minimum=1 and maximum=1)."
    }
}

function Test-Sprint8AExactPropertyInventory {
    param(
        [AllowNull()]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedProperties
    )

    if ($null -eq $Value) { return $false }
    $actualProperties = @($Value.PSObject.Properties.Name)
    if ($actualProperties.Count -ne $ExpectedProperties.Count) { return $false }
    foreach ($property in $ExpectedProperties) {
        if ($actualProperties -cnotcontains $property) { return $false }
    }
    return $true
}

function Get-Sprint8AManualUatContractManifest {
    $repositoryRoot = Split-Path -Parent $PSScriptRoot
    $manifestPath = "docs/sprints/sprint-8a-uat/scenario-contract.json"
    $manifestFullPath = Join-Path $repositoryRoot $manifestPath
    try {
        $manifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    } catch {
        throw "Sprint 8A manual UAT contract manifest is missing or malformed: $($_.Exception.Message)"
    }
    $expectedScenarios = @(Get-Sprint8AManualUatScenarioNames)
    $actualScenarios = @($manifest.scenarios | ForEach-Object { [string]$_.id })
    $expectedPreconditions = @(
        "candidate-fingerprint", "environment-fingerprint", "preflight-receipt",
        "sit-result-receipt", "evidence-root", "execution-start"
    )
    $expectedAcceptanceCriteria = [ordered]@{
        "UAT-8A-01" = @("AC-01", "AC-07", "AC-15", "AC-19")
        "UAT-8A-02" = @("AC-03", "AC-04", "AC-05", "AC-16")
        "UAT-8A-03" = @("AC-08")
        "UAT-8A-04" = @("AC-09", "AC-10", "AC-18", "AC-19")
        "UAT-8A-05" = @("AC-11", "AC-18")
        "UAT-8A-06" = @("AC-01", "AC-02", "AC-06", "AC-12", "AC-16", "AC-18", "AC-19")
        "UAT-8A-07" = @("AC-13")
        "UAT-8A-08" = @("AC-14")
    }
    $expectedSemanticPredicates = [ordered]@{
        "UAT-8A-01" = @("component-module-live-script", "complete-browser-inventory", "module-owned-documents-and-assets", "exact-v3-first-party-inputs")
        "UAT-8A-02" = @("empty-first-apply", "semantic-no-op", "exact-five-core-transitions", "receipt-bound-dashboard-references")
        "UAT-8A-03" = @("configuration-schema-authority", "label-navigation-projection", "sanitized-diagnostics")
        "UAT-8A-04" = @("dataset-contract-execution", "joint-dashboard-component-scope", "known-random-nondisclosure", "timeout-outage-recovery", "components-owned-exact-render-contract", "exact-v3-first-party-inputs")
        "UAT-8A-05" = @("dashboard-lifecycle-findings", "consumer-actions", "provider-outage-containment", "components-owned-exact-render-contract")
        "UAT-8A-06" = @("core-component-absence", "native-wasm-source-boundaries", "old-input-rejection", "exact-real-module-inventory", "components-owned-exact-render-contract", "exact-v3-first-party-inputs", "retired-missing-policy-rejection")
        "UAT-8A-07" = @("induced-owner-failure", "exact-teardown", "empty-successor", "successor-no-op-health")
        "UAT-8A-08" = @("component-only-upgrade", "rollback", "unrelated-identity-stability", "intended-release-restoration")
    }
    $manifestPreconditions = @($manifest.receipt_contract.preconditions | ForEach-Object { [string]$_.id })
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.manual-uat-scenarios" -or
        ($manifest.scenario_count -isnot [int] -and $manifest.scenario_count -isnot [long]) -or
        [int]$manifest.scenario_count -ne $expectedScenarios.Count -or
        ($actualScenarios -join "`n") -cne ($expectedScenarios -join "`n") -or
        @($actualScenarios | Sort-Object -Unique).Count -ne $actualScenarios.Count -or
        ($manifestPreconditions -join "`n") -cne ($expectedPreconditions -join "`n") -or
        (@($manifest.receipt_contract.tester_identity_fields) -join "`n") -cne "tester_id`ndisplay_name`nactor_bindings" -or
        (@($manifest.receipt_contract.actor_binding_fields) -join "`n") -cne "id`nactor_id" -or
        (@($manifest.receipt_contract.precondition_fields) -join "`n") -cne "id`nstate`nvalue`nreference" -or
        (@($manifest.receipt_contract.starting_state_fields) -join "`n") -cne "id`nobserved_value" -or
        (@($manifest.receipt_contract.evidence_fields) -join "`n") -cne "step`nrequirement_id`nkind`ncapture`npath`nsha256" -or
        [int]$manifest.receipt_contract.authenticated_evidence.schema_version -ne 1 -or
        [string]$manifest.receipt_contract.authenticated_evidence.phase -cne "uat-manual-structured-evidence" -or
        $manifest.receipt_contract.authenticated_evidence.sidecar_required -isnot [bool] -or
        -not [bool]$manifest.receipt_contract.authenticated_evidence.sidecar_required -or
        (@($manifest.receipt_contract.authenticated_evidence.required_fields) -join "`n") -cne
            "schema_version`nsprint`nphase`nscenario`nattempt`nevidence_type`nevidence_id`nauthoritative`ndiagnostic`nstate`ncandidate_fingerprint`nenvironment_fingerprint`nstart_checkpoint`nexecution_lease`nproducer_receipt`nassertions") {
        throw "Sprint 8A manual UAT contract manifest has a non-canonical schema or scenario inventory."
    }
    $expectedEvidenceExtensions = [ordered]@{
        "operator-record" = @(".json"); screenshot = @(".png"); "browser-trace" = @(".zip")
        "browser-console" = @(".json"); "http-transcript" = @(".json"); "authenticated-json" = @(".json")
    }
    $expectedEvidenceContentContracts = [ordered]@{
        "operator-record" = [ordered]@{
            schema_version = 1; phase = "uat-manual-operator-record"
            identity_fields = @("scenario", "attempt", "candidate_fingerprint", "environment_fingerprint", "evidence_id")
            payload_field = "observations"; minimum_payload_items = 1
        }
        screenshot = [ordered]@{ format = "png"; required_chunk = "IHDR" }
        "browser-trace" = [ordered]@{
            format = "zip"; required_entry_suffixes = @("trace.trace", "trace.network")
        }
        "browser-console" = [ordered]@{
            schema_version = 1; phase = "uat-manual-browser-console"
            identity_fields = @("scenario", "attempt", "candidate_fingerprint", "environment_fingerprint", "evidence_id")
            payload_field = "entries"; minimum_payload_items = 0
        }
        "http-transcript" = [ordered]@{
            schema_version = 1; phase = "uat-manual-http-transcript"
            identity_fields = @("scenario", "attempt", "candidate_fingerprint", "environment_fingerprint", "evidence_id")
            payload_field = "exchanges"; minimum_payload_items = 1
        }
        "authenticated-json" = [ordered]@{ contract = "authenticated_evidence" }
    }
    if ((@($manifest.receipt_contract.evidence_kind_extensions.PSObject.Properties.Name) -join "`n") -cne
            (@($expectedEvidenceExtensions.Keys) -join "`n") -or
        (@($manifest.receipt_contract.evidence_kind_content_contracts.PSObject.Properties.Name) -join "`n") -cne
            (@($expectedEvidenceContentContracts.Keys) -join "`n")) {
        throw "Sprint 8A manual UAT contract manifest has a non-canonical evidence-kind inventory."
    }
    foreach ($kind in $expectedEvidenceExtensions.Keys) {
        if ((@($manifest.receipt_contract.evidence_kind_extensions.$kind) -join "`n") -cne
                (@($expectedEvidenceExtensions[$kind]) -join "`n") -or
            (ConvertTo-Json -InputObject $manifest.receipt_contract.evidence_kind_content_contracts.PSObject.Properties[$kind].Value -Depth 10 -Compress) -cne
                (ConvertTo-Json -InputObject $expectedEvidenceContentContracts[$kind] -Depth 10 -Compress)) {
            throw "Sprint 8A manual UAT evidence kind '$kind' has a non-canonical extension contract."
        }
    }

    foreach ($scenario in @($manifest.scenarios)) {
        $scenarioId = [string]$scenario.id
        $documentPath = [string]$scenario.document.path
        $expectedDocumentPath = "docs/sprints/sprint-8a-uat/$($scenarioId.ToLowerInvariant()).md"
        $documentFullPath = Join-Path $repositoryRoot $documentPath
        $documentLines = @(Get-Content -LiteralPath $documentFullPath)
        $roleLines = @($documentLines | Where-Object { $_ -match '^- User role:\s*(.+?)\s*$' })
        $documentRole = if ($roleLines.Count -eq 1) {
            ([regex]::Match($roleLines[0], '^- User role:\s*(.+?)\s*$')).Groups[1].Value
        } else { $null }
        $documentSteps = @($documentLines | ForEach-Object {
            $match = [regex]::Match($_, '^\|\s*(\d+)\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|')
            if ($match.Success) {
                [pscustomobject][ordered]@{
                    step = [int]$match.Groups[1].Value
                    action = [string]$match.Groups[2].Value
                    expected_result = [string]$match.Groups[3].Value
                }
            }
        })
        $actorIds = @($scenario.actor_bindings | ForEach-Object { [string]$_.id })
        $startingIds = @($scenario.required_starting_state | ForEach-Object { [string]$_.id })
        $stepNumbers = @($scenario.steps | ForEach-Object { [int]$_.step })
        $evidenceIds = @($scenario.steps | ForEach-Object {
            @($_.evidence_requirements | ForEach-Object { [string]$_.id })
        })
        if ($documentPath -cne $expectedDocumentPath -or
            [string]$scenario.document.sha256 -notmatch '^[0-9a-f]{64}$' -or
            (Get-Sprint8AFileSha256 -Path $documentFullPath) -cne [string]$scenario.document.sha256 -or
            [string]::IsNullOrWhiteSpace([string]$scenario.role) -or $documentRole -cne [string]$scenario.role -or
            (@($scenario.acceptance_criteria) -join "`n") -cne (@($expectedAcceptanceCriteria[$scenarioId]) -join "`n") -or
            (@($scenario.semantic_predicate_ids) -join "`n") -cne (@($expectedSemanticPredicates[$scenarioId]) -join "`n") -or
            $actorIds.Count -lt 1 -or @($actorIds | Sort-Object -Unique).Count -ne $actorIds.Count -or
            @($scenario.actor_bindings | Where-Object {
                [string]::IsNullOrWhiteSpace([string]$_.id) -or [string]::IsNullOrWhiteSpace([string]$_.role)
            }).Count -ne 0 -or
            (@($scenario.required_precondition_ids) -join "`n") -cne ($expectedPreconditions -join "`n") -or
            $startingIds.Count -lt 1 -or @($startingIds | Sort-Object -Unique).Count -ne $startingIds.Count -or
            @($scenario.required_starting_state | Where-Object {
                [string]::IsNullOrWhiteSpace([string]$_.id) -or [string]::IsNullOrWhiteSpace([string]$_.description)
            }).Count -ne 0 -or
            $documentSteps.Count -ne @($scenario.steps).Count -or
            ($stepNumbers -join ",") -cne ((1..@($scenario.steps).Count) -join ",") -or
            @($evidenceIds | Sort-Object -Unique).Count -ne $evidenceIds.Count) {
            throw "Manual UAT scenario '$scenarioId' differs from the canonical manifest or bound document."
        }
        for ($index = 0; $index -lt @($scenario.steps).Count; $index++) {
            $step = $scenario.steps[$index]
            $documentStep = $documentSteps[$index]
            if ([int]$step.step -ne [int]$documentStep.step -or
                [string]$step.action -cne [string]$documentStep.action -or
                [string]$step.expected_result -cne [string]$documentStep.expected_result -or
                @($step.evidence_requirements).Count -lt 1) {
                throw "Manual UAT scenario '$scenarioId' step $([int]$step.step) differs from its bound document."
            }
            foreach ($requirement in @($step.evidence_requirements)) {
                $hasAuthenticatedContract = $requirement.PSObject.Properties.Name -contains "authenticated_contract"
                $authenticatedContract = if ($hasAuthenticatedContract) { $requirement.authenticated_contract } else { $null }
                Assert-Sprint8AManualUatEvidenceCardinality `
                    -Requirement $requirement `
                    -Label "Manual UAT scenario '$scenarioId' evidence requirement '$([string]$requirement.id)'"
                if ([string]$requirement.id -notmatch '^[a-z0-9]+(?:-[a-z0-9]+)*$' -or
                    [string]$requirement.kind -notin @("operator-record", "screenshot", "browser-trace", "browser-console", "http-transcript", "authenticated-json") -or
                    -not (($documentLines -join "`n").Contains("``$([string]$requirement.id)``")) -or
                    ([string]$requirement.kind -ceq "authenticated-json" -and
                        ($null -eq $authenticatedContract -or
                            [string]::IsNullOrWhiteSpace([string]$authenticatedContract.evidence_type) -or
                            [string]::IsNullOrWhiteSpace([string]$authenticatedContract.producer_phase) -or
                            @($authenticatedContract.assertion_ids).Count -lt 1 -or
                            @($authenticatedContract.assertion_ids | Sort-Object -Unique).Count -ne
                                @($authenticatedContract.assertion_ids).Count)) -or
                    ([string]$requirement.kind -cne "authenticated-json" -and $hasAuthenticatedContract)) {
                    throw "Manual UAT scenario '$scenarioId' has an invalid evidence requirement '$([string]$requirement.id)'."
                }
            }
        }
    }
    [pscustomobject][ordered]@{
        path = $manifestPath
        sha256 = Get-Sprint8AFileSha256 -Path $manifestFullPath
        document = $manifest
    }
}

function Get-Sprint8AManualUatScenarioContract {
    param([Parameter(Mandatory)][string]$Scenario)

    $manifest = Get-Sprint8AManualUatContractManifest
    $matches = @($manifest.document.scenarios | Where-Object { [string]$_.id -ceq $Scenario })
    if ($matches.Count -ne 1) {
        throw "Unknown Sprint 8A manual UAT scenario '$Scenario'."
    }
    $contract = $matches[0]
    [pscustomobject][ordered]@{
        scenario = $Scenario
        role = [string]$contract.role
        actor_bindings = @($contract.actor_bindings)
        acceptance_criteria = @($contract.acceptance_criteria)
        semantic_predicate_ids = @($contract.semantic_predicate_ids)
        required_precondition_ids = @($contract.required_precondition_ids)
        required_starting_state = @($contract.required_starting_state)
        step_count = @($contract.steps).Count
        steps = @($contract.steps)
        document = $contract.document
        manifest = [pscustomobject][ordered]@{ path = [string]$manifest.path; sha256 = [string]$manifest.sha256 }
        receipt_contract = $manifest.document.receipt_contract
    }
}

function Get-Sprint8AManualUatEvidencePlan {
    param(
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot)) }
    $evidenceRelative = [IO.Path]::GetRelativePath($repository, $evidence).Replace("\", "/").TrimEnd("/")
    if ($evidenceRelative -eq "." -or $evidenceRelative.StartsWith("../", [StringComparison]::Ordinal) -or
        [IO.Path]::IsPathRooted($evidenceRelative)) {
        throw "Sprint 8A manual UAT evidence root must be a repository-owned path."
    }
    $contract = Get-Sprint8AManualUatScenarioContract -Scenario $Scenario
    $prefix = "$evidenceRelative/uat/attempt-$Attempt/raw/$($Scenario.ToLowerInvariant())"
    $planned = @($contract.steps | ForEach-Object {
        $step = $_
        @($step.evidence_requirements | ForEach-Object {
            $requirement = $_
            $extensions = @($contract.receipt_contract.evidence_kind_extensions.PSObject.Properties[[string]$requirement.kind].Value)
            if ($extensions.Count -ne 1) {
                throw "Manual UAT scenario '$Scenario' evidence '$([string]$requirement.id)' lacks one canonical extension."
            }
            $path = "$prefix/$([string]$requirement.id)$([string]$extensions[0])"
            [pscustomobject][ordered]@{
                step = [int]$step.step
                requirement_id = [string]$requirement.id
                kind = [string]$requirement.kind
                capture = if ($requirement.PSObject.Properties.Name -contains "capture") { $requirement.capture } else { $null }
                minimum = [int]$requirement.minimum
                maximum = [int]$requirement.maximum
                path = $path
                sidecar_path = if ([string]$requirement.kind -ceq "authenticated-json") { "$path.sha256" } else { $null }
                producer_path = if ([string]$requirement.kind -ceq "authenticated-json") {
                    "$prefix/$([string]$requirement.id)-producer.json"
                } else { $null }
                producer_sidecar_path = if ([string]$requirement.kind -ceq "authenticated-json") {
                    "$prefix/$([string]$requirement.id)-producer.json.sha256"
                } else { $null }
                assertion_raw_evidence = if ([string]$requirement.kind -ceq "authenticated-json") {
                    @($requirement.authenticated_contract.assertion_ids | ForEach-Object {
                        [pscustomobject][ordered]@{
                            assertion_id = [string]$_
                            path = "$prefix/$([string]$requirement.id)-$([string]$_)-raw.json"
                        }
                    })
                } else { @() }
            }
        })
    })
    [pscustomobject][ordered]@{
        scenario = $Scenario
        attempt = $Attempt
        raw_prefix = "$prefix/"
        evidence = $planned
        cleanup = [pscustomobject][ordered]@{
            kind = "canonical-restoration"
            path = "$prefix/canonical-restoration.json"
        }
    }
}

function Assert-Sprint8AManualUatJsonEvidenceContent {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$PayloadField,
        [Parameter(Mandatory)][ValidateRange(0, 9999)][int]$MinimumPayloadItems,
        [AllowNull()][string]$RequirementId,
        [switch]$CanonicalRestoration
    )

    $json = $null
    try {
        $json = [Text.Json.JsonDocument]::Parse([IO.File]::ReadAllText($Path))
        $root = $json.RootElement
        if ($root.ValueKind -ne [Text.Json.JsonValueKind]::Object) {
            throw "top-level JSON value is not an object"
        }
        $expectedProperties = if ($CanonicalRestoration) {
            @(
                "schema_version", "sprint", "phase", "scenario", "attempt", "result",
                "candidate_fingerprint", "environment_fingerprint", "restored_at", $PayloadField
            )
        } else {
            @(
                "schema_version", "sprint", "phase", "scenario", "attempt", "evidence_id",
                "candidate_fingerprint", "environment_fingerprint", "captured_at", $PayloadField
            )
        }
        $actualProperties = @($root.EnumerateObject() | ForEach-Object { [string]$_.Name })
        $timestampField = if ($CanonicalRestoration) { "restored_at" } else { "captured_at" }
        $timestamp = ConvertTo-Sprint8ADateTimeOffset `
            -Value $root.GetProperty($timestampField).GetString() `
            -Label "manual UAT $Phase timestamp"
        $payload = $root.GetProperty($PayloadField)
        if (($actualProperties -join "`n") -cne ($expectedProperties -join "`n") -or
            $root.GetProperty("schema_version").GetInt32() -ne 1 -or
            $root.GetProperty("sprint").GetString() -cne "sprint-8a" -or
            $root.GetProperty("phase").GetString() -cne $Phase -or
            $root.GetProperty("scenario").GetString() -cne $Scenario -or
            $root.GetProperty("attempt").GetInt32() -ne $Attempt -or
            $root.GetProperty("candidate_fingerprint").GetString() -cne $CandidateFingerprint -or
            $root.GetProperty("environment_fingerprint").GetString() -cne $EnvironmentFingerprint -or
            $timestamp -lt $StartedAt -or
            $timestamp -gt [DateTimeOffset]::UtcNow.AddMinutes(5) -or
            $payload.ValueKind -ne [Text.Json.JsonValueKind]::Array -or
            $payload.GetArrayLength() -lt $MinimumPayloadItems -or
            ($CanonicalRestoration -and $root.GetProperty("result").GetString() -cne "canonical_topology_verified") -or
            (-not $CanonicalRestoration -and $root.GetProperty("evidence_id").GetString() -cne $RequirementId)) {
            throw "document differs from its exact content schema"
        }
    } catch {
        $label = if ($CanonicalRestoration) { "canonical-restoration" } else { [string]$RequirementId }
        throw "Manual UAT scenario '$Scenario' JSON evidence '$label' is malformed: $($_.Exception.Message)"
    } finally {
        if ($null -ne $json) { $json.Dispose() }
    }
}

function Assert-Sprint8AManualUatEvidenceKindContent {
    param(
        [Parameter(Mandatory)][string]$Kind,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RequirementId,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)]$ContentContract
    )

    switch -CaseSensitive ($Kind) {
        "operator-record" {
            Assert-Sprint8AManualUatJsonEvidenceContent `
                -Path $Path -Scenario $Scenario -Attempt $Attempt `
                -CandidateFingerprint $CandidateFingerprint -EnvironmentFingerprint $EnvironmentFingerprint `
                -StartedAt $StartedAt `
                -Phase ([string]$ContentContract.phase) `
                -PayloadField ([string]$ContentContract.payload_field) `
                -MinimumPayloadItems ([int]$ContentContract.minimum_payload_items) `
                -RequirementId $RequirementId
        }
        "browser-console" {
            Assert-Sprint8AManualUatJsonEvidenceContent `
                -Path $Path -Scenario $Scenario -Attempt $Attempt `
                -CandidateFingerprint $CandidateFingerprint -EnvironmentFingerprint $EnvironmentFingerprint `
                -StartedAt $StartedAt `
                -Phase ([string]$ContentContract.phase) `
                -PayloadField ([string]$ContentContract.payload_field) `
                -MinimumPayloadItems ([int]$ContentContract.minimum_payload_items) `
                -RequirementId $RequirementId
        }
        "http-transcript" {
            Assert-Sprint8AManualUatJsonEvidenceContent `
                -Path $Path -Scenario $Scenario -Attempt $Attempt `
                -CandidateFingerprint $CandidateFingerprint -EnvironmentFingerprint $EnvironmentFingerprint `
                -StartedAt $StartedAt `
                -Phase ([string]$ContentContract.phase) `
                -PayloadField ([string]$ContentContract.payload_field) `
                -MinimumPayloadItems ([int]$ContentContract.minimum_payload_items) `
                -RequirementId $RequirementId
        }
        "screenshot" {
            $bytes = [IO.File]::ReadAllBytes($Path)
            $signature = [byte[]](0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a)
            $signatureValid = $bytes.Length -ge 33
            for ($index = 0; $signatureValid -and $index -lt $signature.Length; $index++) {
                $signatureValid = $bytes[$index] -eq $signature[$index]
            }
            $chunk = if ($bytes.Length -ge 16) { [Text.Encoding]::ASCII.GetString($bytes, 12, 4) } else { "" }
            if (-not $signatureValid -or $chunk -cne [string]$ContentContract.required_chunk) {
                throw "Manual UAT scenario '$Scenario' screenshot '$RequirementId' is not a PNG with the required IHDR chunk."
            }
        }
        "browser-trace" {
            $archive = $null
            try {
                $archive = [IO.Compression.ZipFile]::OpenRead($Path)
                $entries = @($archive.Entries)
                if ($entries.Count -lt 2) { throw "archive is empty or incomplete" }
                foreach ($suffix in @($ContentContract.required_entry_suffixes | ForEach-Object { [string]$_ })) {
                    $matches = @($entries | Where-Object {
                        [string]$_.FullName.EndsWith($suffix, [StringComparison]::Ordinal) -and $_.Length -gt 0
                    })
                    if ($matches.Count -lt 1) { throw "missing non-empty Playwright entry '*$suffix'" }
                }
            } catch {
                throw "Manual UAT scenario '$Scenario' browser trace '$RequirementId' is not an authenticated Playwright trace ZIP: $($_.Exception.Message)"
            } finally {
                if ($null -ne $archive) { $archive.Dispose() }
            }
        }
        "authenticated-json" {
            if ([string]$ContentContract.contract -cne "authenticated_evidence") {
                throw "Manual UAT scenario '$Scenario' authenticated evidence '$RequirementId' has no exact content contract."
            }
        }
        default { throw "Manual UAT scenario '$Scenario' evidence '$RequirementId' has unknown kind '$Kind'." }
    }
}

function Assert-Sprint8AManualUatCleanupEvidenceContent {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt
    )

    Assert-Sprint8AManualUatJsonEvidenceContent `
        -Path $Path -Scenario $Scenario -Attempt $Attempt `
        -CandidateFingerprint $CandidateFingerprint -EnvironmentFingerprint $EnvironmentFingerprint `
        -StartedAt $StartedAt `
        -Phase "uat-manual-canonical-restoration" -PayloadField "observations" `
        -MinimumPayloadItems 1 -CanonicalRestoration
}

function Test-Sprint8ASourceIdentityMatch {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual
    )

    Assert-Sprint8ASourceIdentityObject -Source $Expected | Out-Null
    Assert-Sprint8ASourceIdentityObject -Source $Actual | Out-Null
    foreach ($name in @(
        "commit", "tree", "dirty", "branch",
        "acceptance_inventory_sha256", "deployment_inputs_sha256"
    )) {
        if ($Expected.$name -cne $Actual.$name) {
            return $false
        }
    }
    $true
}

function Test-Sprint8AReceiptReferenceMatch {
    param(
        [AllowEmptyCollection()][object[]]$References = @(),
        [Parameter(Mandatory)]$ExpectedReference
    )

    @($References | Where-Object {
        [string]$_.path -ceq [string]$ExpectedReference.path -and
            [string]$_.sha256 -ceq [string]$ExpectedReference.sha256
    }).Count -eq 1
}

function Assert-Sprint8AExactTerminalIdentities {
    param(
        [Parameter(Mandatory)][object[]]$Results,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    $names = @($Results | ForEach-Object { [string]$_.name })
    if ($names.Count -ne $ExpectedNames.Count -or
        @($names | Sort-Object -Unique).Count -ne $names.Count -or
        (($names | Sort-Object) -join "`n") -cne (($ExpectedNames | Sort-Object) -join "`n")) {
        throw "$Label does not contain its exact terminal identity inventory."
    }
    foreach ($result in $Results) {
        if (@("passed", "failed", "blocked") -cnotcontains [string]$result.state -or
            $result.PSObject.Properties.Name -notcontains "assertions_started" -or
            $result.assertions_started -isnot [bool]) {
            throw "$Label result '$($result.name)' has malformed state or assertion-start evidence."
        }
        if ([string]$result.state -ceq "blocked" -and [bool]$result.assertions_started) {
            throw "$Label blocked result '$($result.name)' cannot claim assertions."
        }
        if ([string]$result.state -ceq "blocked" -and
            [string]::IsNullOrWhiteSpace([string]$result.blocked_reason)) {
            throw "$Label blocked result '$($result.name)' lacks its exact dependency reason."
        }
        if ([string]$result.state -ceq "blocked" -and
            -not [string]::IsNullOrWhiteSpace([string]$result.classification) -and
            [string]$result.classification -cne "product-decision") {
            throw "$Label blocked result '$($result.name)' has an invalid non-failure classification."
        }
        if ([string]$result.state -in @("passed", "failed") -and -not [bool]$result.assertions_started) {
            throw "$Label executed result '$($result.name)' must prove assertions started."
        }
        if ([string]$result.state -in @("passed", "failed")) {
            try {
                $started = ConvertTo-Sprint8ADateTimeOffset -Value $result.started_at -Label "$Label '$($result.name)' start"
                $ended = ConvertTo-Sprint8ADateTimeOffset -Value $result.ended_at -Label "$Label '$($result.name)' end"
            } catch {
                throw "$Label executed result '$($result.name)' has malformed chronology: $($_.Exception.Message)"
            }
            $exitStatus = 0
            $duration = 0L
            $hasRawEvidence = if ($null -ne $result.PSObject.Properties['evidence']) {
                @($result.evidence).Count -ge 1
            } else {
                $null -ne $result.PSObject.Properties['evidence_path'] -and
                    -not [string]::IsNullOrWhiteSpace([string]$result.evidence_path) -and
                    $null -ne $result.PSObject.Properties['evidence_sha256'] -and
                    [string]$result.evidence_sha256 -cmatch '^[0-9a-f]{64}$'
            }
            if ([string]::IsNullOrWhiteSpace([string]$result.command) -or
                -not [int]::TryParse([string]$result.exit_status, [ref]$exitStatus) -or
                -not [long]::TryParse([string]$result.duration_ms, [ref]$duration) -or
                $duration -ne [long][Math]::Max(0, ($ended - $started).TotalMilliseconds) -or
                $ended -lt $started -or
                -not $hasRawEvidence) {
                throw "$Label executed result '$($result.name)' lacks its command, chronology, exit status, or raw evidence."
            }
            if (([string]$result.state -ceq "passed" -and $exitStatus -ne 0) -or
                ([string]$result.state -ceq "failed" -and $exitStatus -eq 0)) {
                throw "$Label result '$($result.name)' has an exit status inconsistent with its terminal state."
            }
            if (([string]$result.state -ceq "passed" -and
                    -not [string]::IsNullOrWhiteSpace([string]$result.classification)) -or
                ([string]$result.state -ceq "failed" -and
                    ([string]$result.classification -notin @("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization") -or
                        [string]::IsNullOrWhiteSpace([string]$result.failure_message)))) {
                throw "$Label result '$($result.name)' has inconsistent terminal failure classification."
            }
        }
    }
    $Results
}

function Get-Sprint8ACandidateIdentity {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')]
        [string]$NormalizedDeploymentConfigurationSha256
    )

    Assert-Sprint8ASourceIdentityObject -Source $Source -RequireClean | Out-Null
    $canonicalSource = [pscustomobject][ordered]@{
        commit = [string]$Source.commit
        tree = [string]$Source.tree
        dirty = [bool]$Source.dirty
        branch = [string]$Source.branch
        acceptance_inventory_sha256 = [string]$Source.acceptance_inventory_sha256
        deployment_inputs_sha256 = [string]$Source.deployment_inputs_sha256
    }
    $deploymentProfile = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "deploy/sprint-8a/**",
        "crates/tessara-component-module/manifest.json",
        "crates/tessara-dashboard-module/manifest.json",
        "crates/tessara-reference-module-sdk/manifest.json",
        "crates/tessara-reference-scoped-records/manifest.json"
    )
    $migrationBaseline = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*/migrations/*.sql"
    )
    $moduleManifests = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*/manifest.json"
    )
    $moduleAssets = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*/assets/**"
    )
    $contractInputs = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*contract*/**",
        "crates/tessara-module-sdk/**"
    )
    $fixtureAndSeedInputs = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "deploy/sprint-7a/uat-fixture-contract.json",
        "deploy/sprint-8a/blueprints/**",
        "deploy/sprint-8a/catalogs/**",
        "crates/**/fixtures/**"
    )
    $moduleReleaseInventory = @(Get-ChildItem -LiteralPath (Join-Path $RepositoryRoot "crates") -Recurse -Filter "manifest.json" -File |
        Sort-Object FullName | ForEach-Object {
            $manifest = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($RepositoryRoot, $_.FullName).Replace("\", "/")
                definition_id = [string]$manifest.definition_id
                release_version = [string]$manifest.release_version
                sha256 = Get-Sprint8AFileSha256 -Path $_.FullName
            }
        })
    $contract = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.candidate-identity"
        source = $canonicalSource
        acceptance_inventory_sha256 = [string]$canonicalSource.acceptance_inventory_sha256
        deployment_inputs_sha256 = [string]$canonicalSource.deployment_inputs_sha256
        deployment_profile_sha256 = [string]$deploymentProfile.sha256
        normalized_deployment_configuration_sha256 = $NormalizedDeploymentConfigurationSha256
        migration_baseline_sha256 = [string]$migrationBaseline.sha256
        module_manifests_sha256 = [string]$moduleManifests.sha256
        module_assets_sha256 = [string]$moduleAssets.sha256
        contract_inputs_sha256 = [string]$contractInputs.sha256
        fixture_seed_inputs_sha256 = [string]$fixtureAndSeedInputs.sha256
        module_release_inventory = $moduleReleaseInventory
        expected_provenance = [pscustomobject][ordered]@{
            "org.opencontainers.image.revision" = [string]$canonicalSource.commit
            "com.tessara.source-tree" = [string]$canonicalSource.tree
            "com.tessara.source-dirty" = "false"
            "com.tessara.build-profile" = "release"
            acceptance_inventory_sha256 = [string]$canonicalSource.acceptance_inventory_sha256
            deployment_inputs_sha256 = [string]$canonicalSource.deployment_inputs_sha256
            module_manifests_sha256 = [string]$moduleManifests.sha256
            module_assets_sha256 = [string]$moduleAssets.sha256
            schema_migrations_sha256 = [string]$migrationBaseline.sha256
            contract_inputs_sha256 = [string]$contractInputs.sha256
            fixture_seed_inputs_sha256 = [string]$fixtureAndSeedInputs.sha256
        }
    }
    [pscustomobject][ordered]@{
        contract = $contract
        fingerprint = Get-Sprint8AStringSha256 -Text ($contract | ConvertTo-Json -Depth 30 -Compress)
    }
}

function Assert-Sprint8ALifecyclePrerequisite {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$Reference,
        [AllowNull()][string]$ExpectedPhase,
        [AllowNull()][string]$ExpectedCandidateFingerprint,
        [AllowNull()][string]$ExpectedEnvironmentFingerprint
    )

    $resolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Reference.path)

    $sha = Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
    if ($sha -cne [string]$Reference.sha256) {
        throw "Lifecycle prerequisite '$ExpectedPhase' SHA-256 differs from its reference."
    }
    $receipt = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
    $mutableGateReceipt = [string]$receipt.phase -in @("validation-readiness", "candidate-rehearsal")
    $schemaVersionAccepted = if ($mutableGateReceipt) {
        [int]$receipt.schema_version -in @(2, 3)
    } else {
        [int]$receipt.schema_version -eq 1
    }
    if (($receipt.schema_version -isnot [int] -and $receipt.schema_version -isnot [long]) -or
        -not $schemaVersionAccepted -or
        [string]$receipt.sprint -cne "sprint-8a" -or
        (-not [string]::IsNullOrWhiteSpace($ExpectedPhase) -and [string]$receipt.phase -cne $ExpectedPhase) -or
        [string]$receipt.state -cne "passed") {
        throw "Lifecycle prerequisite '$($Reference.path)' is not one passing Sprint 8A receipt."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedCandidateFingerprint) -and
        [string]$receipt.candidate_fingerprint -cne $ExpectedCandidateFingerprint) {
        throw "Lifecycle prerequisite '$ExpectedPhase' carries a different candidate fingerprint."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedEnvironmentFingerprint) -and
        [string]$receipt.environment_fingerprint -cne $ExpectedEnvironmentFingerprint) {
        throw "Lifecycle prerequisite '$ExpectedPhase' carries a different environment fingerprint."
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$resolved.path; sha256 = $sha }
        receipt = $receipt
    }
}

function Assert-Sprint8ALifecyclePrerequisiteSet {
    param(
        [Parameter(Mandatory)][ValidateSet("validation-preflight", "candidate-freeze", "sit", "uat")][string]$Phase,
        [AllowEmptyCollection()][object[]]$References = @(),
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')]
        [string]$NormalizedDeploymentConfigurationSha256,
        [AllowNull()][string]$CandidateFingerprint,
        [switch]$AllowPreflightHarnessOnlySourceAdvance,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $expectedPhases = switch ($Phase) {
        "validation-preflight" { @("validation-readiness", "candidate-rehearsal") }
        "candidate-freeze" { @("validation-preflight") }
        "sit" { @("validation-preflight", "candidate-freeze") }
        "uat" { @("validation-preflight", "candidate-freeze", "sit") }
    }
    if (@($References).Count -ne $expectedPhases.Count) {
        throw "Lifecycle phase '$Phase' requires exactly: $($expectedPhases -join ', ')."
    }
    if ($Phase -ceq "validation-preflight") {
        if (-not [string]::IsNullOrWhiteSpace([string]$CandidateFingerprint)) {
            throw "Validation preflight cannot receive a candidate fingerprint before freeze."
        }
    } else {
        if ($AllowPreflightHarnessOnlySourceAdvance) {
            throw "Preflight harness-only source advance cannot authorize a frozen-candidate downstream phase."
        }
        $expectedCandidateIdentity = Get-Sprint8ACandidateIdentity `
            -RepositoryRoot $RepositoryRoot `
            -Source $Source `
            -NormalizedDeploymentConfigurationSha256 $NormalizedDeploymentConfigurationSha256
        if ([string]$expectedCandidateIdentity.fingerprint -cne $CandidateFingerprint) {
            throw "Lifecycle phase '$Phase' carries a non-canonical candidate fingerprint."
        }
    }

    $resolvedInputs = @($References | ForEach-Object {
        Assert-Sprint8ALifecyclePrerequisite `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Reference $_ `
            -ExpectedEnvironmentFingerprint $EnvironmentFingerprint
    })
    $resolvedPrerequisites = [Collections.Generic.List[object]]::new()
    foreach ($expectedPhase in $expectedPhases) {
        $matches = @($resolvedInputs | Where-Object { [string]$_.receipt.phase -ceq $expectedPhase })
        if ($matches.Count -ne 1) {
            throw "Lifecycle phase '$Phase' does not bind exactly one '$expectedPhase' prerequisite."
        }
        $resolvedPrerequisites.Add($matches[0])
    }
    if (@($resolvedPrerequisites | ForEach-Object { [string]$_.reference.path } | Sort-Object -Unique).Count -ne
        $resolvedPrerequisites.Count) {
        throw "Lifecycle phase '$Phase' repeats a prerequisite receipt path."
    }

    foreach ($prerequisite in $resolvedPrerequisites) {
        $receiptSource = $prerequisite.receipt.PSObject.Properties["source_identity"]
        if ($null -eq $receiptSource) {
            $receiptSource = $prerequisite.receipt.PSObject.Properties["mutable_source_identity"]
        }
        $sourceMatches = $null -ne $receiptSource -and
            (Test-Sprint8ASourceIdentityMatch -Expected $Source -Actual $receiptSource.Value)
        if (-not $sourceMatches) {
            if (-not $AllowPreflightHarnessOnlySourceAdvance -or $Phase -cne "validation-preflight" -or $null -eq $receiptSource) {
                throw "Lifecycle prerequisite '$($prerequisite.receipt.phase)' carries another source identity."
            }
            $allowedPaths = @(
                "docs/sprints/sprint-8a-verification.md",
                "scripts/run-sprint-8a-validation-preflight.ps1",
                "scripts/sprint-8a-lifecycle-chain.ps1"
            )
            & git -C $RepositoryRoot merge-base --is-ancestor ([string]$receiptSource.Value.commit) ([string]$Source.commit)
            $ancestorExit = $LASTEXITCODE
            $changedPaths = @(& git -C $RepositoryRoot diff --name-only "$([string]$receiptSource.Value.commit)..$([string]$Source.commit)" |
                ForEach-Object { $_.Replace("\", "/") })
            if ($ancestorExit -ne 0 -or $LASTEXITCODE -ne 0 -or
                (($changedPaths | Sort-Object) -join "`n") -cne (($allowedPaths | Sort-Object) -join "`n")) {
                throw "Lifecycle preflight harness-only advance contains a path outside the exact authorized correction set."
            }
        }
        if ([string]$prerequisite.receipt.phase -in @("candidate-freeze", "sit") -and
            [string]$prerequisite.receipt.candidate_fingerprint -cne $CandidateFingerprint) {
            throw "Lifecycle prerequisite '$($prerequisite.receipt.phase)' carries another candidate fingerprint."
        }
    }

    $byPhase = @{}
    foreach ($prerequisite in $resolvedPrerequisites) {
        $byPhase[[string]$prerequisite.receipt.phase] = $prerequisite
    }
    if ($Phase -ceq "validation-preflight") {
        if ($byPhase["validation-readiness"].receipt.authoritative -isnot [bool] -or
            $byPhase["candidate-rehearsal"].receipt.authoritative -isnot [bool] -or
            $byPhase["validation-readiness"].receipt.authoritative -ne $false -or
            $byPhase["candidate-rehearsal"].receipt.authoritative -ne $false -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["candidate-rehearsal"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["validation-readiness"].reference)) {
            throw "Preflight prerequisites do not form the exact non-authoritative Readiness -> Rehearsal chain."
        }
    }
    if ($Phase -in @("candidate-freeze", "sit", "uat") -and
        ($byPhase["validation-preflight"].receipt.authoritative -isnot [bool] -or
            $byPhase["validation-preflight"].receipt.authoritative -ne $true)) {
        throw "Frozen lifecycle phase '$Phase' requires one authoritative passing preflight receipt."
    }
    if ($Phase -in @("sit", "uat")) {
        if ($byPhase["candidate-freeze"].receipt.authoritative -isnot [bool] -or
            $byPhase["candidate-freeze"].receipt.authoritative -ne $true -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["candidate-freeze"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["validation-preflight"].reference) -or
            @($byPhase["candidate-freeze"].receipt.prerequisite_receipts).Count -ne 1 -or
            ($byPhase["candidate-freeze"].receipt.details.candidate_identity | ConvertTo-Json -Depth 30 -Compress) -cne
                ($expectedCandidateIdentity.contract | ConvertTo-Json -Depth 30 -Compress)) {
            throw "Lifecycle phase '$Phase' requires the exact preflight-bound candidate receipt."
        }
    }
    if ($Phase -ceq "uat") {
        if ($byPhase["sit"].receipt.authoritative -isnot [bool] -or
            $byPhase["sit"].receipt.authoritative -ne $true -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["sit"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["candidate-freeze"].reference) -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["sit"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["validation-preflight"].reference) -or
            @($byPhase["sit"].receipt.prerequisite_receipts).Count -ne 2) {
            throw "Formal UAT requires the exact candidate-bound SIT receipt."
        }
    }

    @($resolvedPrerequisites)
}

function Publish-Sprint8ALifecycleReceipt {
    param(
        [Parameter(Mandatory)][ValidateSet("validation-preflight", "candidate-freeze", "sit", "uat")][string]$Phase,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')]
        [string]$NormalizedDeploymentConfigurationSha256,
        [AllowNull()][string]$CandidateFingerprint,
        [AllowEmptyCollection()][object[]]$PrerequisiteReceipts = @(),
        [AllowEmptyCollection()][object[]]$Checks = @(),
        [AllowNull()]$Details,
        [AllowNull()]$CleanupRestoration,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$OutputPath,
        [switch]$PrepareOnly
    )

    Assert-Sprint8ASourceIdentityObject -Source $Source -RequireClean | Out-Null
    if ($Phase -cne "validation-preflight" -and
        [string]$CandidateFingerprint -notmatch '^[0-9a-f]{64}$') {
        throw "Lifecycle phase '$Phase' requires one lowercase candidate fingerprint."
    }
    $expectedNames = switch ($Phase) {
        "validation-preflight" { Get-Sprint8APreflightCheckNames }
        "candidate-freeze" { @() }
        "sit" { Get-Sprint8ASitLaneNames }
        "uat" { @("scripted-uat") + @(Get-Sprint8AManualUatScenarioNames) }
    }
    Assert-Sprint8AExactTerminalIdentities -Results @($Checks) -ExpectedNames @($expectedNames) -Label $Phase | Out-Null
    $nonpassing = @($Checks | Where-Object state -CNE "passed")
    if ($nonpassing.Count -gt 0) {
        throw "A passing lifecycle receipt cannot be published with failed or blocked '$Phase' results."
    }
    $resolvedPrerequisites = Assert-Sprint8ALifecyclePrerequisiteSet `
        -Phase $Phase `
        -References @($PrerequisiteReceipts) `
        -Source $Source `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -NormalizedDeploymentConfigurationSha256 $NormalizedDeploymentConfigurationSha256 `
        -CandidateFingerprint $CandidateFingerprint `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot
    $canonicalPrerequisites = @($resolvedPrerequisites | ForEach-Object { $_.reference })
    $phaseDetails = $Details
    if ($Phase -ceq "candidate-freeze") {
        $candidateIdentity = Get-Sprint8ACandidateIdentity `
            -RepositoryRoot $RepositoryRoot `
            -Source $Source `
            -NormalizedDeploymentConfigurationSha256 $NormalizedDeploymentConfigurationSha256
        if ([string]$candidateIdentity.fingerprint -cne $CandidateFingerprint) {
            throw "Candidate-freeze fingerprint does not match the canonical candidate identity."
        }
        $phaseDetails = [pscustomobject][ordered]@{
            candidate_identity = $candidateIdentity.contract
            phase_details = $Details
        }
    }
    $canonicalChecks = @($Checks | ForEach-Object {
        $check = $_
        $canonicalCheck = $check | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        $canonicalEvidence = @()
        foreach ($evidence in @($check.evidence)) {
            $evidenceReference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$evidence.path)
            if ((Get-Sprint8AFileSha256 -Path ([string]$evidenceReference.full_path)) -cne [string]$evidence.sha256) {
                throw "Lifecycle check '$($check.name)' has stale evidence '$($evidence.path)'."
            }
            $canonicalEvidence += [pscustomobject][ordered]@{
                path = [string]$evidenceReference.path
                sha256 = [string]$evidence.sha256
            }
        }
        $canonicalCheck.evidence = $canonicalEvidence
        $canonicalCheck
    })
    $canonicalCleanupRestoration = $null
    if ($Phase -in @("sit", "uat")) {
        if ($null -eq $CleanupRestoration -or
            [string]$CleanupRestoration.result -cne "canonical_topology_verified" -or
            @($CleanupRestoration.evidence).Count -lt 1) {
            throw "Lifecycle phase '$Phase' requires retained canonical-topology restoration evidence."
        }
        $canonicalRestorationEvidence = @()
        foreach ($evidence in @($CleanupRestoration.evidence)) {
            $evidenceReference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$evidence.path)
            if ((Get-Sprint8AFileSha256 -Path ([string]$evidenceReference.full_path)) -cne [string]$evidence.sha256) {
                throw "Lifecycle phase '$Phase' has stale restoration evidence '$($evidence.path)'."
            }
            $canonicalRestorationEvidence += [pscustomobject][ordered]@{
                path = [string]$evidenceReference.path
                sha256 = [string]$evidence.sha256
            }
        }
        $canonicalCleanupRestoration = [pscustomobject][ordered]@{
            required = $true
            result = "canonical_topology_verified"
            evidence = $canonicalRestorationEvidence
        }
    } elseif ($null -ne $CleanupRestoration) {
        throw "Lifecycle phase '$Phase' cannot claim SIT/UAT restoration evidence."
    }
    $target = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $OutputPath
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $evidenceRootRelative = [IO.Path]::GetRelativePath($RepositoryRoot, $evidenceRootFullPath).Replace("\", "/").TrimEnd("/")
    $expectedOutputPath = switch ($Phase) {
        "validation-preflight" { "$evidenceRootRelative/preflight-result.json" }
        "candidate-freeze" { "$evidenceRootRelative/candidate.json" }
        "sit" { "$evidenceRootRelative/sit-result.json" }
        "uat" { "$evidenceRootRelative/uat-result.json" }
    }
    if ([string]$target.path -cne $expectedOutputPath) {
        throw "Lifecycle phase '$Phase' must publish to canonical path '$expectedOutputPath'."
    }
    if ($PrepareOnly -and $Phase -cne "uat") {
        throw "Only formal UAT may prepare its canonical result for manifest commitment before publication."
    }
    $now = [DateTimeOffset]::UtcNow
    $startedAt = if (@($canonicalChecks).Count -gt 0) {
        @($canonicalChecks | ForEach-Object { ConvertTo-Sprint8ADateTimeOffset -Value $_.started_at -Label "lifecycle check start" } | Sort-Object | Select-Object -First 1)[0]
    } else { $now }
    $endedAt = if (@($canonicalChecks).Count -gt 0) {
        @($canonicalChecks | ForEach-Object { ConvertTo-Sprint8ADateTimeOffset -Value $_.ended_at -Label "lifecycle check end" } | Sort-Object | Select-Object -Last 1)[0]
    } else { $now }
    $receipt = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = $Phase
        attempt = $Attempt
        authoritative = $true
        state = "passed"
        assertions_started = @($Checks).Count -gt 0
        started_at = $startedAt.ToString("o")
        ended_at = $endedAt.ToString("o")
        duration_ms = [long][Math]::Max(0, ($endedAt - $startedAt).TotalMilliseconds)
        source_identity = $Source
        environment_fingerprint = $EnvironmentFingerprint
        candidate_fingerprint = if ($Phase -ceq "validation-preflight") { $null } else { $CandidateFingerprint }
        prerequisite_receipts = $canonicalPrerequisites
        checks = $canonicalChecks
        assertion_count = @($canonicalChecks | Where-Object assertions_started -EQ $true).Count
        failure_count = 0
        blocked_count = 0
        classification = $null
        correction = $null
        narrow_proof = $null
        invalidation_decision = "none_required"
        details = $phaseDetails
        cleanup_restoration = if ($Phase -in @("sit", "uat")) {
            $canonicalCleanupRestoration
        } else {
            [pscustomobject][ordered]@{ required = $false; result = "not_applicable"; evidence = @() }
        }
    }
    $expectedSha256 = Get-Sprint8AStringSha256 -Text (($receipt | ConvertTo-Json -Depth 30) + "`n")
    if (-not $PrepareOnly) {
        Publish-Sprint7AEvidence -Document $receipt -OutputPath ([string]$target.full_path) | Out-Null
        if ((Assert-Sprint8AReceiptSidecar -Path ([string]$target.full_path)) -cne $expectedSha256) {
            throw "Lifecycle phase '$Phase' publication differs from its prepared canonical bytes."
        }
    }
    [pscustomobject][ordered]@{
        path = [string]$target.path
        sha256 = $expectedSha256
        receipt = $receipt
        prepared_only = [bool]$PrepareOnly
    }
}

function Publish-Sprint8AManualUatStructuredEvidence {
    param(
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)][string]$RequirementId,
        [Parameter(Mandatory)][object[]]$AssertionEvidence,
        [Parameter(Mandatory)]$ExecutionLease,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot)) }
    $contract = Get-Sprint8AManualUatScenarioContract -Scenario $Scenario
    $requirementMatches = @($contract.steps | ForEach-Object {
        $step = $_
        @($step.evidence_requirements | Where-Object { [string]$_.id -ceq $RequirementId } | ForEach-Object {
            [pscustomobject][ordered]@{ step = [int]$step.step; definition = $_ }
        })
    })
    if ($requirementMatches.Count -ne 1 -or
        [string]$requirementMatches[0].definition.kind -cne "authenticated-json") {
        throw "Manual UAT scenario '$Scenario' requirement '$RequirementId' is not one exact authenticated-JSON contract."
    }
    $requirement = $requirementMatches[0].definition
    $plan = Get-Sprint8AManualUatEvidencePlan `
        -Scenario $Scenario -Attempt $Attempt -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath
    $plannedMatches = @($plan.evidence | Where-Object { [string]$_.requirement_id -ceq $RequirementId })
    if ($plannedMatches.Count -ne 1) {
        throw "Manual UAT scenario '$Scenario' requirement '$RequirementId' has no canonical publication plan."
    }
    $planned = $plannedMatches[0]
    $checkpoint = Assert-Sprint8AManualUatStartCheckpoint `
        -Attempt $Attempt `
        -Scenario $Scenario `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -ScenarioStartedAt $StartedAt `
        -RepositoryRoot $repository `
        -EvidenceRoot $evidenceRootFullPath
    Assert-Sprint8AManualUatAttemptOpen `
        -Attempt $Attempt -Checkpoint $checkpoint -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath | Out-Null
    $expectedLockPath = [IO.Path]::GetFullPath((Join-Path $evidenceRootFullPath "validation-attempt.lock"))
    if ($ExecutionLease.PSObject.Properties.Name -notcontains "stream" -or
        $ExecutionLease.stream -isnot [IO.FileStream] -or -not $ExecutionLease.stream.CanWrite -or
        [IO.Path]::GetFullPath([string]$ExecutionLease.stream.Name) -cne $expectedLockPath -or
        $ExecutionLease.authoritative -isnot [bool] -or
        $ExecutionLease.diagnostic -isnot [bool] -or
        [bool]$ExecutionLease.authoritative -eq [bool]$ExecutionLease.diagnostic -or
        [int]$ExecutionLease.current_process_id -ne $PID -or
        [int]$ExecutionLease.attempt -ne $Attempt -or
        [string]$ExecutionLease.scenario -cne $Scenario -or
        [string]$ExecutionLease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$ExecutionLease.environment_fingerprint -cne $EnvironmentFingerprint -or
        [string]$ExecutionLease.started_at -cne $StartedAt.ToString("o") -or
        (ConvertTo-Json -InputObject $ExecutionLease.checkpoint -Depth 10 -Compress) -cne
            (ConvertTo-Json -InputObject $checkpoint.reference -Depth 10 -Compress)) {
        throw "Manual UAT scenario '$Scenario' structured publication requires its exact live execution lease."
    }
    $lease = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath -Path ([string]$ExecutionLease.lease.path)
    if ((Assert-Sprint8AReceiptSidecar -Path ([string]$lease.full_path)) -cne [string]$ExecutionLease.lease.sha256) {
        throw "Manual UAT scenario '$Scenario' structured publication has a stale execution lease."
    }

    $expectedAssertionIds = @($requirement.authenticated_contract.assertion_ids | ForEach-Object { [string]$_ })
    $actualAssertionIds = @($AssertionEvidence | ForEach-Object { [string]$_.assertion_id })
    if (($actualAssertionIds -join "`n") -cne ($expectedAssertionIds -join "`n") -or
        @($actualAssertionIds | Sort-Object -Unique).Count -ne $actualAssertionIds.Count) {
        throw "Manual UAT scenario '$Scenario' structured publication must supply every exact assertion identity in contract order."
    }
    $plannedAssertionRaw = @{}
    foreach ($plannedRaw in @($planned.assertion_raw_evidence)) {
        $plannedAssertionRaw[[string]$plannedRaw.assertion_id] = [string]$plannedRaw.path
    }
    $canonicalAssertions = @($AssertionEvidence | ForEach-Object {
        $assertion = $_
        if ((@($assertion.PSObject.Properties.Name) -join "`n") -cne "assertion_id`nraw_evidence" -or
            @($assertion.raw_evidence).Count -ne 1 -or
            -not $plannedAssertionRaw.ContainsKey([string]$assertion.assertion_id)) {
            throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.assertion_id)' is malformed."
        }
        $rawReferences = @($assertion.raw_evidence | ForEach-Object {
            if ((@($_.PSObject.Properties.Name) -join "`n") -cne "path`nsha256") {
                throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.assertion_id)' has a malformed raw reference."
            }
            $raw = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath -Path ([string]$_.path)
            if ([string]$raw.path -cne [string]$plannedAssertionRaw[[string]$assertion.assertion_id] -or
                -not [string]$raw.path.StartsWith([string]$plan.raw_prefix, [StringComparison]::Ordinal) -or
                [string]$raw.path -in @([string]$planned.path, [string]$planned.producer_path) -or
                (Get-Sprint8AFileSha256 -Path ([string]$raw.full_path)) -cne [string]$_.sha256 -or
                (Get-Item -LiteralPath ([string]$raw.full_path)).LastWriteTimeUtc -lt $StartedAt.UtcDateTime) {
                throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.assertion_id)' has stale, non-canonical, or self-referential raw evidence."
            }
            [pscustomobject][ordered]@{ path = [string]$raw.path; sha256 = [string]$_.sha256 }
        })
        if (@($rawReferences.path | Sort-Object -Unique).Count -ne $rawReferences.Count) {
            throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.assertion_id)' repeats raw evidence."
        }
        [pscustomobject][ordered]@{
            id = [string]$assertion.assertion_id
            state = "passed"
            producer_receipt = $null
            raw_evidence = $rawReferences
        }
    })

    $producerDocument = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = [string]$requirement.authenticated_contract.producer_phase
        scenario = $Scenario
        attempt = $Attempt
        authoritative = [bool]$ExecutionLease.authoritative
        diagnostic = [bool]$ExecutionLease.diagnostic
        state = "passed"
        candidate_fingerprint = $CandidateFingerprint
        environment_fingerprint = $EnvironmentFingerprint
        start_checkpoint = $checkpoint.reference
        execution_lease = $ExecutionLease.lease
        assertion_ids = $expectedAssertionIds
    }
    $producerDocument = $producerDocument | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $producerTarget = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath -Path ([string]$planned.producer_path)
    $producerSha = Get-Sprint8AStringSha256 -Text ((ConvertTo-Json -InputObject $producerDocument -Depth 30) + "`n")
    Repair-Sprint7AEvidencePublication -Path ([string]$producerTarget.full_path)
    $producerExists = Test-Path -LiteralPath ([string]$producerTarget.full_path) -PathType Leaf
    $producerSidecarExists = Test-Path -LiteralPath "$([string]$producerTarget.full_path).sha256" -PathType Leaf
    if ($producerExists -or $producerSidecarExists) {
        if (-not $producerExists -or -not $producerSidecarExists -or
            (Assert-Sprint8AReceiptSidecar -Path ([string]$producerTarget.full_path)) -cne $producerSha) {
            throw "Manual UAT scenario '$Scenario' already has a different or incomplete structured producer publication."
        }
    } else {
        Publish-Sprint7AEvidence -Document $producerDocument -OutputPath ([string]$producerTarget.full_path) | Out-Null
    }
    $producerReference = [pscustomobject][ordered]@{ path = [string]$producerTarget.path; sha256 = $producerSha }
    foreach ($assertion in $canonicalAssertions) { $assertion.producer_receipt = $producerReference }
    $wrapperDocument = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "uat-manual-structured-evidence"
        scenario = $Scenario
        attempt = $Attempt
        evidence_type = [string]$requirement.authenticated_contract.evidence_type
        evidence_id = $RequirementId
        authoritative = [bool]$ExecutionLease.authoritative
        diagnostic = [bool]$ExecutionLease.diagnostic
        state = "passed"
        candidate_fingerprint = $CandidateFingerprint
        environment_fingerprint = $EnvironmentFingerprint
        start_checkpoint = $checkpoint.reference
        execution_lease = $ExecutionLease.lease
        producer_receipt = $producerReference
        assertions = $canonicalAssertions
    }
    $wrapperDocument = $wrapperDocument | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $wrapperTarget = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath -Path ([string]$planned.path)
    $wrapperSha = Get-Sprint8AStringSha256 -Text ((ConvertTo-Json -InputObject $wrapperDocument -Depth 30) + "`n")
    Repair-Sprint7AEvidencePublication -Path ([string]$wrapperTarget.full_path)
    $wrapperExists = Test-Path -LiteralPath ([string]$wrapperTarget.full_path) -PathType Leaf
    $wrapperSidecarExists = Test-Path -LiteralPath "$([string]$wrapperTarget.full_path).sha256" -PathType Leaf
    if ($wrapperExists -or $wrapperSidecarExists) {
        if (-not $wrapperExists -or -not $wrapperSidecarExists -or
            (Assert-Sprint8AReceiptSidecar -Path ([string]$wrapperTarget.full_path)) -cne $wrapperSha) {
            throw "Manual UAT scenario '$Scenario' already has a different or incomplete structured wrapper publication."
        }
    } else {
        Publish-Sprint7AEvidence -Document $wrapperDocument -OutputPath ([string]$wrapperTarget.full_path) | Out-Null
    }
    $result = [pscustomobject][ordered]@{
        step = [int]$requirementMatches[0].step
        requirement_id = $RequirementId
        kind = "authenticated-json"
        capture = if ($requirement.PSObject.Properties.Name -contains "capture") { $requirement.capture } else { $null }
        path = [string]$wrapperTarget.path
        sha256 = $wrapperSha
    }
    Assert-Sprint8AManualUatStructuredEvidence `
        -Evidence $result -Requirement $requirement -Scenario $Scenario -Attempt $Attempt `
        -CandidateFingerprint $CandidateFingerprint -EnvironmentFingerprint $EnvironmentFingerprint `
        -ExpectedAuthoritative ([bool]$ExecutionLease.authoritative) `
        -ExpectedDiagnostic ([bool]$ExecutionLease.diagnostic) `
        -StartedAt $StartedAt -StartCheckpoint $checkpoint.reference -ExecutionLease $ExecutionLease.lease `
        -RepositoryRoot $repository -EvidenceRoot $evidenceRootFullPath | Out-Null
    $result
}

function Assert-Sprint8AManualUatStructuredEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)]$Requirement,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][bool]$ExpectedAuthoritative,
        [Parameter(Mandatory)][bool]$ExpectedDiagnostic,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)]$StartCheckpoint,
        [Parameter(Mandatory)]$ExecutionLease,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $resolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Evidence.path)
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $plan = Get-Sprint8AManualUatEvidencePlan `
        -Scenario $Scenario -Attempt $Attempt -RepositoryRoot $RepositoryRoot -EvidenceRoot $evidenceRootFullPath
    $plannedMatches = @($plan.evidence | Where-Object { [string]$_.requirement_id -ceq [string]$Requirement.id })
    if ($plannedMatches.Count -ne 1 -or [string]$resolved.path -cne [string]$plannedMatches[0].path) {
        throw "Manual UAT scenario '$Scenario' authenticated evidence '$([string]$Evidence.requirement_id)' has a non-canonical wrapper path."
    }
    $planned = $plannedMatches[0]
    $expectedRawPrefix = [string]$plan.raw_prefix
    $plannedAssertionRaw = @{}
    foreach ($plannedRaw in @($planned.assertion_raw_evidence)) {
        $plannedAssertionRaw[[string]$plannedRaw.assertion_id] = [string]$plannedRaw.path
    }
    $sidecarSha = Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
    if ($sidecarSha -cne [string]$Evidence.sha256) {
        throw "Manual UAT scenario '$Scenario' authenticated evidence '$([string]$Evidence.requirement_id)' has a stale sidecar."
    }
    try {
        $document = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
    } catch {
        throw "Manual UAT scenario '$Scenario' authenticated evidence '$([string]$Evidence.requirement_id)' is not valid JSON."
    }
    $expectedAssertions = @($Requirement.authenticated_contract.assertion_ids | ForEach-Object { [string]$_ })
    $actualAssertions = @($document.assertions)
    $actualAssertionIds = @($actualAssertions | ForEach-Object { [string]$_.id })
    if ((@($document.PSObject.Properties.Name) -join "`n") -cne
            "schema_version`nsprint`nphase`nscenario`nattempt`nevidence_type`nevidence_id`nauthoritative`ndiagnostic`nstate`ncandidate_fingerprint`nenvironment_fingerprint`nstart_checkpoint`nexecution_lease`nproducer_receipt`nassertions" -or
        ($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 1 -or [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "uat-manual-structured-evidence" -or
        [string]$document.scenario -cne $Scenario -or
        [int]$document.attempt -ne $Attempt -or
        [string]$document.evidence_type -cne [string]$Requirement.authenticated_contract.evidence_type -or
        [string]$document.evidence_id -cne [string]$Requirement.id -or
        $document.authoritative -isnot [bool] -or [bool]$document.authoritative -ne $ExpectedAuthoritative -or
        $document.diagnostic -isnot [bool] -or [bool]$document.diagnostic -ne $ExpectedDiagnostic -or
        [bool]$document.authoritative -eq [bool]$document.diagnostic -or
        [string]$document.state -cne "passed" -or
        [string]$document.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$document.environment_fingerprint -cne $EnvironmentFingerprint -or
        (@($document.start_checkpoint.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
        (@($document.execution_lease.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
        (@($document.producer_receipt.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
        (ConvertTo-Json $document.start_checkpoint -Compress) -cne (ConvertTo-Json $StartCheckpoint -Compress) -or
        (ConvertTo-Json $document.execution_lease -Compress) -cne (ConvertTo-Json $ExecutionLease -Compress) -or
        ($actualAssertionIds -join "`n") -cne ($expectedAssertions -join "`n") -or
        @($actualAssertionIds | Sort-Object -Unique).Count -ne $actualAssertionIds.Count) {
        throw "Manual UAT scenario '$Scenario' authenticated evidence '$([string]$Evidence.requirement_id)' differs from its exact structured contract."
    }
    $producer = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$document.producer_receipt.path)
    $producerSha = Assert-Sprint8AReceiptSidecar -Path ([string]$producer.full_path)
    if ($producerSha -cne [string]$document.producer_receipt.sha256 -or
        [string]$producer.path -cne [string]$planned.producer_path -or
        -not [string]$producer.path.StartsWith($expectedRawPrefix, [StringComparison]::Ordinal) -or
        (Get-Item -LiteralPath ([string]$producer.full_path)).LastWriteTimeUtc -lt $StartedAt.UtcDateTime -or
        [IO.Path]::GetFullPath([string]$producer.full_path) -ceq [IO.Path]::GetFullPath([string]$resolved.full_path)) {
        throw "Manual UAT scenario '$Scenario' structured evidence does not bind a distinct authenticated producer receipt."
    }
    try {
        $producerDocument = Get-Content -LiteralPath ([string]$producer.full_path) -Raw | ConvertFrom-Json
    } catch {
        throw "Manual UAT scenario '$Scenario' structured-evidence producer receipt is malformed."
    }
    if ((@($producerDocument.PSObject.Properties.Name) -join "`n") -cne
            "schema_version`nsprint`nphase`nscenario`nattempt`nauthoritative`ndiagnostic`nstate`ncandidate_fingerprint`nenvironment_fingerprint`nstart_checkpoint`nexecution_lease`nassertion_ids" -or
        ($producerDocument.schema_version -isnot [int] -and $producerDocument.schema_version -isnot [long]) -or
        [int]$producerDocument.schema_version -ne 1 -or [string]$producerDocument.sprint -cne "sprint-8a" -or
        [string]$producerDocument.phase -cne [string]$Requirement.authenticated_contract.producer_phase -or
        [string]$producerDocument.scenario -cne $Scenario -or
        [int]$producerDocument.attempt -ne $Attempt -or
        $producerDocument.authoritative -isnot [bool] -or [bool]$producerDocument.authoritative -ne $ExpectedAuthoritative -or
        $producerDocument.diagnostic -isnot [bool] -or [bool]$producerDocument.diagnostic -ne $ExpectedDiagnostic -or
        [bool]$producerDocument.authoritative -eq [bool]$producerDocument.diagnostic -or
        [string]$producerDocument.state -cne "passed" -or
        [string]$producerDocument.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$producerDocument.environment_fingerprint -cne $EnvironmentFingerprint -or
        (@($producerDocument.start_checkpoint.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
        (@($producerDocument.execution_lease.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
        (ConvertTo-Json $producerDocument.start_checkpoint -Compress) -cne (ConvertTo-Json $StartCheckpoint -Compress) -or
        (ConvertTo-Json $producerDocument.execution_lease -Compress) -cne (ConvertTo-Json $ExecutionLease -Compress) -or
        (@($producerDocument.assertion_ids) -join "`n") -cne ($expectedAssertions -join "`n")) {
        throw "Manual UAT scenario '$Scenario' structured-evidence producer receipt differs from its exact producer contract."
    }
    foreach ($assertion in $actualAssertions) {
        if ((@($assertion.PSObject.Properties.Name) -join "`n") -cne "id`nstate`nproducer_receipt`nraw_evidence" -or
            (@($assertion.producer_receipt.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
            [string]$assertion.state -cne "passed" -or
            [string]$assertion.producer_receipt.path -cne [string]$document.producer_receipt.path -or
            [string]$assertion.producer_receipt.sha256 -cne [string]$document.producer_receipt.sha256 -or
            @($assertion.raw_evidence).Count -ne 1 -or
            -not $plannedAssertionRaw.ContainsKey([string]$assertion.id)) {
            throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.id)' lacks a passing producer/raw-evidence binding."
        }
        $rawPaths = @()
        foreach ($rawEvidence in @($assertion.raw_evidence)) {
            if ((@($rawEvidence.PSObject.Properties.Name) -join "`n") -cne "path`nsha256") {
                throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.id)' has a malformed raw-evidence reference."
            }
            $raw = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$rawEvidence.path)
            if ([string]$raw.path -cne [string]$plannedAssertionRaw[[string]$assertion.id] -or
                -not [string]$raw.path.StartsWith($expectedRawPrefix, [StringComparison]::Ordinal) -or
                (Get-Sprint8AFileSha256 -Path ([string]$raw.full_path)) -cne [string]$rawEvidence.sha256 -or
                (Get-Item -LiteralPath ([string]$raw.full_path)).LastWriteTimeUtc -lt $StartedAt.UtcDateTime -or
                [IO.Path]::GetFullPath([string]$raw.full_path) -in @(
                    [IO.Path]::GetFullPath([string]$resolved.full_path),
                    [IO.Path]::GetFullPath([string]$producer.full_path)
                )) {
                throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.id)' has stale or self-referential raw evidence."
            }
            $rawPaths += [string]$raw.path
        }
        if (@($rawPaths | Sort-Object -Unique).Count -ne $rawPaths.Count) {
            throw "Manual UAT scenario '$Scenario' structured assertion '$([string]$assertion.id)' repeats raw evidence."
        }
    }
    $document
}

function Assert-Sprint8AManualUatReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][string]$ExpectedScenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [AllowNull()][string]$RepositoryRoot,
        [AllowNull()][string]$EvidenceRoot
    )

    $contract = Get-Sprint8AManualUatScenarioContract -Scenario $ExpectedScenario
    $shapeContractValid = $true
    try {
        $expectedReceiptProperties = @(
            "schema_version", "sprint", "phase", "attempt", "authoritative", "diagnostic", "scenario", "state",
            "candidate_fingerprint", "environment_fingerprint", "assertions_started", "assertions_started_at",
            "started_at", "ended_at", "duration_ms", "role", "tester_identity", "preconditions", "starting_state",
            "actions", "expected_result", "actual_result", "classification", "classification_source",
            "failure_message", "blocked_reason", "scenario_contract", "start_checkpoint", "execution_lease",
            "resumed", "execution_resume", "evidence", "cleanup_restoration"
        )
        if (-not (Test-Sprint8AExactPropertyInventory -Value $Receipt -ExpectedProperties $expectedReceiptProperties) -or
            $Receipt.tester_identity.actor_bindings -isnot [array] -or
            $Receipt.preconditions -isnot [array] -or
            $Receipt.starting_state -isnot [array] -or
            $Receipt.actions -isnot [array] -or
            $Receipt.evidence -isnot [array] -or
            -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.scenario_contract -ExpectedProperties @(
                    "manifest", "document", "acceptance_criteria", "semantic_predicate_ids"
                )) -or
            $Receipt.scenario_contract.acceptance_criteria -isnot [array] -or
            $Receipt.scenario_contract.semantic_predicate_ids -isnot [array] -or
            -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.scenario_contract.manifest -ExpectedProperties @("path", "sha256")) -or
            -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.scenario_contract.document -ExpectedProperties @("path", "sha256")) -or
            -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.start_checkpoint -ExpectedProperties @("path", "sha256")) -or
            -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.execution_lease -ExpectedProperties @("path", "sha256")) -or
            ($null -ne $Receipt.execution_resume -and
                -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.execution_resume -ExpectedProperties @("path", "sha256"))) -or
            -not (Test-Sprint8AExactPropertyInventory -Value $Receipt.cleanup_restoration -ExpectedProperties @(
                    "required", "result", "evidence"
                )) -or
            $Receipt.cleanup_restoration.evidence -isnot [array] -or
            @($Receipt.cleanup_restoration.evidence).Count -ne 1) {
            $shapeContractValid = $false
        }
        foreach ($action in @($Receipt.actions)) {
            if (-not (Test-Sprint8AExactPropertyInventory -Value $action -ExpectedProperties @(
                        "step", "action", "expected_result", "actual_result", "state"
                    ))) {
                $shapeContractValid = $false
            }
        }
        foreach ($cleanupEvidence in @($Receipt.cleanup_restoration.evidence)) {
            if (-not (Test-Sprint8AExactPropertyInventory -Value $cleanupEvidence -ExpectedProperties @(
                        "kind", "path", "sha256"
                    ))) {
                $shapeContractValid = $false
            }
        }
    } catch {
        $shapeContractValid = $false
    }
    try {
        $started = ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.started_at -Label "manual UAT start"
        $ended = ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.ended_at -Label "manual UAT end"
    } catch {
        throw "Manual UAT receipt '$ExpectedScenario' has malformed chronology: $($_.Exception.Message)"
    }
    $duration = 0L
    $expectedSummary = "Every canonical step expectation in $ExpectedScenario is satisfied."
    $expectedLeaseSuffix = "uat/attempt-$ExpectedAttempt/manual-leases/$($ExpectedScenario.ToLowerInvariant())-start.json"
    $expectedResumeSuffix = "uat/attempt-$ExpectedAttempt/manual-leases/$($ExpectedScenario.ToLowerInvariant())-resume.json"
    $resumeContractValid = $Receipt.PSObject.Properties.Name -contains "resumed" -and
        $Receipt.resumed -is [bool] -and
        $Receipt.PSObject.Properties.Name -contains "execution_resume"
    if ($resumeContractValid) {
        if ([bool]$Receipt.resumed) {
            $resumeContractValid = $null -ne $Receipt.execution_resume -and
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.execution_resume.path) -and
                [string]$Receipt.execution_resume.path -notmatch '(^|/)\.\.(/|$)|\\' -and
                [string]$Receipt.execution_resume.path.EndsWith($expectedResumeSuffix, [StringComparison]::Ordinal) -and
                [string]$Receipt.execution_resume.sha256 -match '^[0-9a-f]{64}$'
        } else {
            $resumeContractValid = $null -eq $Receipt.execution_resume
        }
    }
    $actionContractValid = $true
    $actionStates = @()
    try {
        if (@($Receipt.actions).Count -ne [int]$contract.step_count) {
            $actionContractValid = $false
        } else {
            for ($index = 0; $index -lt $contract.steps.Count; $index++) {
                $actualAction = $Receipt.actions[$index]
                $expectedAction = $contract.steps[$index]
                if (-not (Test-Sprint8AExactPropertyInventory -Value $actualAction -ExpectedProperties @(
                            "step", "action", "expected_result", "actual_result", "state"
                        )) -or
                    ($actualAction.step -isnot [int] -and $actualAction.step -isnot [long]) -or
                    [int]$actualAction.step -ne [int]$expectedAction.step -or
                    [string]$actualAction.action -cne [string]$expectedAction.action -or
                    [string]$actualAction.expected_result -cne [string]$expectedAction.expected_result -or
                    [string]::IsNullOrWhiteSpace([string]$actualAction.actual_result) -or
                    [string]$actualAction.state -notin @("passed", "failed", "blocked")) {
                    $actionContractValid = $false
                    break
                }
                $actionStates += [string]$actualAction.state
            }
        }
    } catch {
        $actionContractValid = $false
    }
    $identityContractValid = $true
    try {
        $expectedActorIds = @($contract.actor_bindings | ForEach-Object { [string]$_.id })
        $actualActorBindings = @($Receipt.tester_identity.actor_bindings)
        $actualActorIds = @($actualActorBindings | ForEach-Object { [string]$_.id })
        $actualAccountIds = @($actualActorBindings | ForEach-Object { [string]$_.actor_id })
        if ((@($Receipt.tester_identity.PSObject.Properties.Name) -join "`n") -cne "tester_id`ndisplay_name`nactor_bindings" -or
            [string]::IsNullOrWhiteSpace([string]$Receipt.tester_identity.tester_id) -or
            [string]::IsNullOrWhiteSpace([string]$Receipt.tester_identity.display_name) -or
            ($actualActorIds -join "`n") -cne ($expectedActorIds -join "`n") -or
            @($actualActorIds | Sort-Object -Unique).Count -ne $actualActorIds.Count -or
            @($actualAccountIds | Sort-Object -Unique).Count -ne $actualAccountIds.Count -or
            @($actualActorBindings | Where-Object {
                (@($_.PSObject.Properties.Name) -join "`n") -cne "id`nactor_id" -or
                [string]::IsNullOrWhiteSpace([string]$_.actor_id)
            }).Count -ne 0) {
            $identityContractValid = $false
        }
        $preconditions = @($Receipt.preconditions)
        $preconditionIds = @($preconditions | ForEach-Object { [string]$_.id })
        if (($preconditionIds -join "`n") -cne (@($contract.required_precondition_ids) -join "`n") -or
            @($preconditions | Where-Object {
                (@($_.PSObject.Properties.Name) -join "`n") -cne "id`nstate`nvalue`nreference" -or
                [string]$_.state -notin @("satisfied", "failed", "blocked") -or
                (([string]::IsNullOrWhiteSpace([string]$_.value)) -eq ($null -eq $_.reference))
            }).Count -ne 0 -or
            ([string]$Receipt.state -ceq "passed" -and
                @($preconditions | Where-Object { [string]$_.state -cne "satisfied" }).Count -ne 0)) {
            $identityContractValid = $false
        }
        $preconditionById = @{}
        foreach ($precondition in $preconditions) { $preconditionById[[string]$precondition.id] = $precondition }
        $executionStart = ConvertTo-Sprint8ADateTimeOffset `
            -Value $preconditionById["execution-start"].value `
            -Label "manual UAT execution-start precondition"
        if ([string]$preconditionById["candidate-fingerprint"].value -cne $CandidateFingerprint -or
            $null -ne $preconditionById["candidate-fingerprint"].reference -or
            [string]$preconditionById["environment-fingerprint"].value -cne $EnvironmentFingerprint -or
            $null -ne $preconditionById["environment-fingerprint"].reference -or
            $executionStart -ne $started -or
            $null -ne $preconditionById["execution-start"].reference -or
            -not [string]::IsNullOrWhiteSpace([string]$preconditionById["preflight-receipt"].value) -or
            -not [string]::IsNullOrWhiteSpace([string]$preconditionById["sit-result-receipt"].value)) {
            $identityContractValid = $false
        }
        foreach ($referencePreconditionId in @("preflight-receipt", "sit-result-receipt")) {
            $reference = $preconditionById[$referencePreconditionId].reference
            if ((@($reference.PSObject.Properties.Name) -join "`n") -cne "path`nsha256" -or
                [string]::IsNullOrWhiteSpace([string]$reference.path) -or
                [string]$reference.sha256 -notmatch '^[0-9a-f]{64}$') {
                $identityContractValid = $false
            }
        }
        if (-not [string]::IsNullOrWhiteSpace($RepositoryRoot) -and -not [string]::IsNullOrWhiteSpace($EvidenceRoot)) {
            $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
                [IO.Path]::GetFullPath($EvidenceRoot)
            } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
            $expectedEvidenceRoot = [IO.Path]::GetRelativePath($RepositoryRoot, $evidenceRootFullPath).Replace("\", "/").TrimEnd("/")
            if ([string]$preconditionById["evidence-root"].value -cne $expectedEvidenceRoot -or
                $null -ne $preconditionById["evidence-root"].reference) {
                $identityContractValid = $false
            }
            $checkpoint = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -Path ([string]$Receipt.start_checkpoint.path)
            $checkpointSha = Assert-Sprint8AReceiptSidecar -Path ([string]$checkpoint.full_path)
            $checkpointDocument = Get-Content -LiteralPath ([string]$checkpoint.full_path) -Raw | ConvertFrom-Json
            if ($checkpointSha -cne [string]$Receipt.start_checkpoint.sha256 -or
                @($checkpointDocument.prerequisite_receipts).Count -ne 3 -or
                (ConvertTo-Json $preconditionById["preflight-receipt"].reference -Compress) -cne
                    (ConvertTo-Json $checkpointDocument.prerequisite_receipts[0] -Compress) -or
                (ConvertTo-Json $preconditionById["sit-result-receipt"].reference -Compress) -cne
                    (ConvertTo-Json $checkpointDocument.prerequisite_receipts[2] -Compress)) {
                $identityContractValid = $false
            }
        } elseif ([string]::IsNullOrWhiteSpace([string]$preconditionById["evidence-root"].value) -or
            $null -ne $preconditionById["evidence-root"].reference) {
            $identityContractValid = $false
        }
        $startingState = @($Receipt.starting_state)
        $startingIds = @($startingState | ForEach-Object { [string]$_.id })
        $expectedStartingIds = @($contract.required_starting_state | ForEach-Object { [string]$_.id })
        if (($startingIds -join "`n") -cne ($expectedStartingIds -join "`n") -or
            @($startingState | Where-Object {
                (@($_.PSObject.Properties.Name) -join "`n") -cne "id`nobserved_value" -or
                [string]::IsNullOrWhiteSpace([string]$_.observed_value)
            }).Count -ne 0) {
            $identityContractValid = $false
        }
    } catch {
        $identityContractValid = $false
    }
    $evidenceContractValid = $true
    $evidenceContractFailure = $null
    try {
        $requirementMap = @{}
        $plannedEvidenceMap = @{}
        $evidencePlan = $null
        if (-not [string]::IsNullOrWhiteSpace($RepositoryRoot) -and -not [string]::IsNullOrWhiteSpace($EvidenceRoot)) {
            $evidencePlan = Get-Sprint8AManualUatEvidencePlan `
                -Scenario $ExpectedScenario `
                -Attempt $ExpectedAttempt `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot
            foreach ($plannedEvidence in @($evidencePlan.evidence)) {
                $plannedEvidenceMap[[string]$plannedEvidence.requirement_id] = $plannedEvidence
            }
        }
        foreach ($step in @($contract.steps)) {
            foreach ($requirement in @($step.evidence_requirements)) {
                $requirementMap[[string]$requirement.id] = [pscustomobject]@{
                    step = [int]$step.step
                    definition = $requirement
                }
            }
        }
        $evidenceCounts = @{}
        foreach ($evidence in @($Receipt.evidence)) {
            $evidenceProperties = @($evidence.PSObject.Properties.Name)
            $requirementId = [string]$evidence.requirement_id
            if (($evidenceProperties -join "`n") -cne "step`nrequirement_id`nkind`ncapture`npath`nsha256" -or
                -not $requirementMap.ContainsKey($requirementId)) {
                $evidenceContractValid = $false
                continue
            }
            $requirementBinding = $requirementMap[$requirementId]
            $expectedCapture = if ($requirementBinding.definition.PSObject.Properties.Name -contains "capture") {
                $requirementBinding.definition.capture | ConvertTo-Json -Depth 10 -Compress
            } else { "null" }
            $actualCapture = $evidence.capture | ConvertTo-Json -Depth 10 -Compress
            $allowedExtensions = @($contract.receipt_contract.evidence_kind_extensions.([string]$evidence.kind))
            if (($evidence.step -isnot [int] -and $evidence.step -isnot [long]) -or
                [int]$evidence.step -ne [int]$requirementBinding.step -or
                [string]$evidence.kind -cne [string]$requirementBinding.definition.kind -or
                $actualCapture -cne $expectedCapture -or
                [string]::IsNullOrWhiteSpace([string]$evidence.path) -or
                [string]$evidence.path -match '(^|/)\.\.(/|$)|\\' -or
                [string]$evidence.sha256 -notmatch '^[0-9a-f]{64}$' -or
                @($allowedExtensions | Where-Object {
                    [string]$evidence.path.EndsWith([string]$_, [StringComparison]::OrdinalIgnoreCase)
                }).Count -ne 1) {
                $evidenceContractValid = $false
            }
            if (-not [string]::IsNullOrWhiteSpace($RepositoryRoot) -and -not [string]::IsNullOrWhiteSpace($EvidenceRoot)) {
                $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
                    [IO.Path]::GetFullPath($EvidenceRoot)
                } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
                $evidenceRootRelative = [IO.Path]::GetRelativePath($RepositoryRoot, $evidenceRootFullPath).Replace("\", "/").TrimEnd("/")
                $expectedRawPrefix = "$evidenceRootRelative/uat/attempt-$ExpectedAttempt/raw/$($ExpectedScenario.ToLowerInvariant())/"
                $resolvedEvidence = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $RepositoryRoot `
                    -EvidenceRoot $evidenceRootFullPath `
                    -Path ([string]$evidence.path)
                $plannedEvidence = $plannedEvidenceMap[$requirementId]
                if ($null -eq $plannedEvidence -or
                    [string]$resolvedEvidence.path -cne [string]$plannedEvidence.path -or
                    -not [string]$resolvedEvidence.path.StartsWith($expectedRawPrefix, [StringComparison]::Ordinal) -or
                    (Get-Sprint8AFileSha256 -Path ([string]$resolvedEvidence.full_path)) -cne [string]$evidence.sha256 -or
                    (Get-Item -LiteralPath ([string]$resolvedEvidence.full_path)).LastWriteTimeUtc -lt $started.UtcDateTime) {
                    $evidenceContractValid = $false
                }
                $contentContract = $contract.receipt_contract.evidence_kind_content_contracts.PSObject.Properties[[string]$evidence.kind].Value
                Assert-Sprint8AManualUatEvidenceKindContent `
                    -Kind ([string]$evidence.kind) `
                    -Path ([string]$resolvedEvidence.full_path) `
                    -Scenario $ExpectedScenario `
                    -Attempt $ExpectedAttempt `
                    -RequirementId $requirementId `
                    -CandidateFingerprint $CandidateFingerprint `
                    -EnvironmentFingerprint $EnvironmentFingerprint `
                    -StartedAt $started `
                    -ContentContract $contentContract
            }
            if (-not $evidenceCounts.ContainsKey($requirementId)) { $evidenceCounts[$requirementId] = 0 }
            $evidenceCounts[$requirementId] = [int]$evidenceCounts[$requirementId] + 1
            if ([string]$evidence.kind -ceq "authenticated-json") {
                if ([string]::IsNullOrWhiteSpace($RepositoryRoot) -or [string]::IsNullOrWhiteSpace($EvidenceRoot)) {
                    $evidenceContractValid = $false
                } else {
                    Assert-Sprint8AManualUatStructuredEvidence `
                        -Evidence $evidence `
                        -Requirement $requirementBinding.definition `
                        -Scenario $ExpectedScenario `
                        -Attempt $ExpectedAttempt `
                        -CandidateFingerprint $CandidateFingerprint `
                        -EnvironmentFingerprint $EnvironmentFingerprint `
                        -ExpectedAuthoritative ([bool]$Receipt.authoritative) `
                        -ExpectedDiagnostic ([bool]$Receipt.diagnostic) `
                        -StartedAt $started `
                        -StartCheckpoint $Receipt.start_checkpoint `
                        -ExecutionLease $Receipt.execution_lease `
                        -RepositoryRoot $RepositoryRoot `
                        -EvidenceRoot $EvidenceRoot | Out-Null
                }
            }
        }
        foreach ($requirementId in $requirementMap.Keys) {
            $count = if ($evidenceCounts.ContainsKey($requirementId)) { [int]$evidenceCounts[$requirementId] } else { 0 }
            $definition = $requirementMap[$requirementId].definition
            if ($count -gt [int]$definition.maximum -or
                ([string]$Receipt.state -ceq "passed" -and $count -lt [int]$definition.minimum)) {
                $evidenceContractValid = $false
            }
        }
        foreach ($cleanupEvidence in @($Receipt.cleanup_restoration.evidence)) {
            if ([string]$cleanupEvidence.kind -cne "canonical-restoration" -or
                [string]::IsNullOrWhiteSpace([string]$cleanupEvidence.path) -or
                [string]$cleanupEvidence.sha256 -notmatch '^[0-9a-f]{64}$' -or
                -not [string]$cleanupEvidence.path.EndsWith(".json", [StringComparison]::OrdinalIgnoreCase)) {
                $evidenceContractValid = $false
            }
            if (-not [string]::IsNullOrWhiteSpace($RepositoryRoot) -and -not [string]::IsNullOrWhiteSpace($EvidenceRoot)) {
                $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
                    [IO.Path]::GetFullPath($EvidenceRoot)
                } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
                $evidenceRootRelative = [IO.Path]::GetRelativePath($RepositoryRoot, $evidenceRootFullPath).Replace("\", "/").TrimEnd("/")
                $expectedRawPrefix = "$evidenceRootRelative/uat/attempt-$ExpectedAttempt/raw/$($ExpectedScenario.ToLowerInvariant())/"
                $resolvedCleanup = Resolve-Sprint8AEvidenceReference `
                    -RepositoryRoot $RepositoryRoot `
                    -EvidenceRoot $evidenceRootFullPath `
                    -Path ([string]$cleanupEvidence.path)
                if ([string]$resolvedCleanup.path -cne [string]$evidencePlan.cleanup.path -or
                    -not [string]$resolvedCleanup.path.StartsWith($expectedRawPrefix, [StringComparison]::Ordinal) -or
                    (Get-Sprint8AFileSha256 -Path ([string]$resolvedCleanup.full_path)) -cne [string]$cleanupEvidence.sha256 -or
                    (Get-Item -LiteralPath ([string]$resolvedCleanup.full_path)).LastWriteTimeUtc -lt $started.UtcDateTime) {
                    $evidenceContractValid = $false
                }
                Assert-Sprint8AManualUatCleanupEvidenceContent `
                    -Path ([string]$resolvedCleanup.full_path) `
                    -Scenario $ExpectedScenario `
                    -Attempt $ExpectedAttempt `
                    -CandidateFingerprint $CandidateFingerprint `
                    -EnvironmentFingerprint $EnvironmentFingerprint `
                    -StartedAt $started
            }
        }
    } catch {
        $evidenceContractValid = $false
        $evidenceContractFailure = $_.Exception.Message
    }
    if (($Receipt.schema_version -isnot [int] -and $Receipt.schema_version -isnot [long]) -or
        [int]$Receipt.schema_version -ne 2 -or
        (Get-Sprint8AManualUatScenarioNames) -cnotcontains $ExpectedScenario -or
        [string]$Receipt.sprint -cne "sprint-8a" -or
        [string]$Receipt.phase -cne "uat-manual-scenario" -or
        ($Receipt.attempt -isnot [int] -and $Receipt.attempt -isnot [long]) -or
        [int]$Receipt.attempt -ne $ExpectedAttempt -or
        $Receipt.authoritative -isnot [bool] -or
        $Receipt.diagnostic -isnot [bool] -or
        [bool]$Receipt.authoritative -eq [bool]$Receipt.diagnostic -or
        [string]$Receipt.scenario -cne $ExpectedScenario -or
        [string]$Receipt.state -notin @("passed", "failed", "blocked") -or
        [string]$Receipt.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$Receipt.environment_fingerprint -cne $EnvironmentFingerprint -or
        $Receipt.assertions_started -isnot [bool] -or
        [string]$Receipt.role -cne [string]$contract.role -or
        -not $shapeContractValid -or
        -not $identityContractValid -or
        -not $actionContractValid -or
        -not $evidenceContractValid -or
        [string]$Receipt.expected_result -cne $expectedSummary -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.actual_result) -or
        [string]$Receipt.scenario_contract.manifest.path -cne [string]$contract.manifest.path -or
        [string]$Receipt.scenario_contract.manifest.sha256 -cne [string]$contract.manifest.sha256 -or
        [string]$Receipt.scenario_contract.document.path -cne [string]$contract.document.path -or
        [string]$Receipt.scenario_contract.document.sha256 -cne [string]$contract.document.sha256 -or
        (@($Receipt.scenario_contract.acceptance_criteria) -join "`n") -cne (@($contract.acceptance_criteria) -join "`n") -or
        (@($Receipt.scenario_contract.semantic_predicate_ids) -join "`n") -cne (@($contract.semantic_predicate_ids) -join "`n") -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.start_checkpoint.path) -or
        [string]$Receipt.start_checkpoint.sha256 -notmatch '^[0-9a-f]{64}$' -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.execution_lease.path) -or
        [string]$Receipt.execution_lease.path -match '(^|/)\.\.(/|$)|\\' -or
        -not [string]$Receipt.execution_lease.path.EndsWith($expectedLeaseSuffix, [StringComparison]::Ordinal) -or
        [string]$Receipt.execution_lease.sha256 -notmatch '^[0-9a-f]{64}$' -or
        -not $resumeContractValid -or
        -not [long]::TryParse([string]$Receipt.duration_ms, [ref]$duration) -or
        $duration -ne [long][Math]::Max(0, ($ended - $started).TotalMilliseconds) -or
        $ended -lt $started -or
        $Receipt.cleanup_restoration.required -isnot [bool] -or
        $Receipt.cleanup_restoration.required -ne $true -or
        [string]$Receipt.cleanup_restoration.result -cne "canonical_topology_verified" -or
        @($Receipt.cleanup_restoration.evidence).Count -lt 1) {
        $evidenceFailureDetail = if ([string]::IsNullOrWhiteSpace([string]$evidenceContractFailure)) {
            ""
        } else { " Evidence validation failed: $evidenceContractFailure" }
        throw "Manual UAT receipt '$ExpectedScenario' is malformed, incomplete, or bound to another candidate/environment.$evidenceFailureDetail"
    }
    if (([string]$Receipt.state -ceq "passed" -and
            ((@($actionStates | Where-Object { $_ -cne "passed" }).Count -ne 0) -or
                ([bool]$Receipt.authoritative -and [bool]$Receipt.resumed) -or
                -not [bool]$Receipt.assertions_started -or
                [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.classification) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.classification_source) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.failure_message) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.blocked_reason))) -or
        ([string]$Receipt.state -ceq "failed" -and
            ((@($actionStates | Where-Object { $_ -ceq "failed" }).Count -lt 1 -and
                    -not (-not [bool]$Receipt.assertions_started -and
                        [string]$Receipt.classification -ceq "preflight/setup" -and
                        @($actionStates | Where-Object { $_ -ceq "blocked" }).Count -gt 0)) -or
                [string]$Receipt.classification -notin @("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization") -or
                [string]$Receipt.classification_source -cne "manual_operator" -or
                [string]::IsNullOrWhiteSpace([string]$Receipt.failure_message) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.blocked_reason) -or
                ([bool]$Receipt.assertions_started -and [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at)) -or
                (-not [bool]$Receipt.assertions_started -and -not [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at)))) -or
        ([string]$Receipt.state -ceq "blocked" -and
            ((@($actionStates | Where-Object { $_ -ceq "failed" }).Count -ne 0) -or
                (@($actionStates | Where-Object { $_ -ceq "blocked" }).Count -lt 1) -or
                [bool]$Receipt.assertions_started -or
                ([bool]$Receipt.authoritative -and -not [bool]$Receipt.resumed) -or
                [string]::IsNullOrWhiteSpace([string]$Receipt.blocked_reason) -or
                (([string]::IsNullOrWhiteSpace([string]$Receipt.classification) -and
                        -not [string]::IsNullOrWhiteSpace([string]$Receipt.classification_source)) -or
                    (-not [string]::IsNullOrWhiteSpace([string]$Receipt.classification) -and
                        ([string]$Receipt.classification -cne "product-decision" -or
                            [string]$Receipt.classification_source -cne "manual_operator"))) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.failure_message) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at)))) {
        throw "Manual UAT receipt '$ExpectedScenario' has inconsistent authority, assertion, or failure disposition."
    }
    $Receipt
}

function Assert-Sprint8AManualUatExecutionLeasePair {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$ReceiptReference,
        [Parameter(Mandatory)][string]$ExpectedScenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $start = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Receipt.execution_lease.path)
    $startSha = Assert-Sprint8AReceiptSidecar -Path ([string]$start.full_path)
    if ($startSha -cne [string]$Receipt.execution_lease.sha256) {
        throw "Manual UAT scenario '$ExpectedScenario' execution-lease digest differs from its receipt."
    }
    $lease = Get-Content -LiteralPath ([string]$start.full_path) -Raw | ConvertFrom-Json
    $leaseStartedAt = ConvertTo-Sprint8ADateTimeOffset -Value $lease.started_at -Label "manual UAT execution-lease start"
    $receiptStartedAt = ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.started_at -Label "manual UAT receipt start"
    if (($lease.schema_version -isnot [int] -and $lease.schema_version -isnot [long]) -or
        [int]$lease.schema_version -ne 1 -or [string]$lease.sprint -cne "sprint-8a" -or
        [string]$lease.phase -cne "uat-manual-execution-lease" -or [string]$lease.state -cne "executing" -or
        [int]$lease.attempt -ne $ExpectedAttempt -or [string]$lease.scenario -cne $ExpectedScenario -or
        [string]$lease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$lease.environment_fingerprint -cne $EnvironmentFingerprint -or
        $leaseStartedAt -ne $receiptStartedAt -or
        $lease.authoritative -isnot [bool] -or [bool]$lease.authoritative -ne [bool]$Receipt.authoritative -or
        $lease.diagnostic -isnot [bool] -or [bool]$lease.diagnostic -ne [bool]$Receipt.diagnostic -or
        ($lease.process_id -isnot [int] -and $lease.process_id -isnot [long]) -or
        [int]$lease.process_id -lt 1 -or
        [string]$lease.checkpoint.path -cne [string]$Receipt.start_checkpoint.path -or
        [string]$lease.checkpoint.sha256 -cne [string]$Receipt.start_checkpoint.sha256) {
        throw "Manual UAT scenario '$ExpectedScenario' execution lease is malformed or bound to another execution."
    }
    if ([bool]$Receipt.resumed) {
        Assert-Sprint8AManualUatResumeMarker `
            -MarkerReference $Receipt.execution_resume `
            -LeaseReference ([pscustomobject]@{ path = [string]$start.path; sha256 = $startSha }) `
            -LeaseDocument $lease `
            -Scenario $ExpectedScenario `
            -Attempt $ExpectedAttempt `
            -CandidateFingerprint $CandidateFingerprint `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -StartedAt $receiptStartedAt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot | Out-Null
    } elseif ($null -ne $Receipt.execution_resume) {
        throw "Manual UAT scenario '$ExpectedScenario' has an unexpected execution-resume binding."
    }

    $completionPath = [string]$start.path
    if (-not $completionPath.EndsWith("-start.json", [StringComparison]::Ordinal)) {
        throw "Manual UAT scenario '$ExpectedScenario' execution lease path is non-canonical."
    }
    $completionPath = $completionPath.Substring(0, $completionPath.Length - "-start.json".Length) + "-complete.json"
    $completion = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $completionPath
    $completionSha = Assert-Sprint8AReceiptSidecar -Path ([string]$completion.full_path)
    $document = Get-Content -LiteralPath ([string]$completion.full_path) -Raw | ConvertFrom-Json
    try {
        $completionStarted = ConvertTo-Sprint8ADateTimeOffset -Value $document.started_at -Label "manual UAT lease start"
        $completionEnded = ConvertTo-Sprint8ADateTimeOffset -Value $document.ended_at -Label "manual UAT lease completion"
    } catch {
        throw "Manual UAT scenario '$ExpectedScenario' execution-lease completion has malformed chronology: $($_.Exception.Message)"
    }
    if (($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 1 -or [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "uat-manual-execution-lease" -or [string]$document.state -cne "completed" -or
        $document.authoritative -isnot [bool] -or [bool]$document.authoritative -or
        [int]$document.attempt -ne $ExpectedAttempt -or [string]$document.scenario -cne $ExpectedScenario -or
        [string]$document.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$document.environment_fingerprint -cne $EnvironmentFingerprint -or
        $completionStarted -ne $receiptStartedAt -or
        $completionEnded -ne (ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.ended_at -Label "manual UAT receipt end") -or
        $completionEnded -lt $completionStarted -or
        [string]$document.lease.path -cne [string]$start.path -or
        [string]$document.lease.sha256 -cne $startSha -or
        $document.resumed -isnot [bool] -or [bool]$document.resumed -ne [bool]$Receipt.resumed -or
        (ConvertTo-Json -InputObject $document.resume -Depth 10 -Compress) -cne
            (ConvertTo-Json -InputObject $Receipt.execution_resume -Depth 10 -Compress) -or
        [string]$document.receipt.path -cne [string]$ReceiptReference.path -or
        [string]$document.receipt.sha256 -cne [string]$ReceiptReference.sha256) {
        throw "Manual UAT scenario '$ExpectedScenario' execution-lease completion is malformed or not bound to its exact receipt."
    }
    [pscustomobject][ordered]@{
        start = [pscustomobject][ordered]@{ path = [string]$start.path; sha256 = $startSha }
        completion = [pscustomobject][ordered]@{ path = [string]$completion.path; sha256 = $completionSha }
    }
}

function Assert-Sprint8AManualUatStartCheckpoint {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$ScenarioStartedAt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $checkpointPath = Join-Path $evidenceRootFullPath "attempts/uat-$Attempt-manual-checkpoint.json"
    $checkpointRepositoryPath = [IO.Path]::GetRelativePath($RepositoryRoot, $checkpointPath).Replace("\", "/")
    $reference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path $checkpointRepositoryPath
    $sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$reference.full_path)
    $checkpoint = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
    try {
        $scriptedCompletedAt = ConvertTo-Sprint8ADateTimeOffset `
            -Value $checkpoint.scripted_completed_at `
            -Label "manual UAT scripted completion"
    } catch {
        throw "Manual UAT scenario '$Scenario' Start checkpoint has malformed chronology: $($_.Exception.Message)"
    }
    if (($checkpoint.schema_version -isnot [int] -and $checkpoint.schema_version -isnot [long]) -or
        [int]$checkpoint.schema_version -ne 1 -or
        [string]$checkpoint.sprint -cne "sprint-8a" -or
        [string]$checkpoint.phase -cne "uat" -or
        ($checkpoint.attempt -isnot [int] -and $checkpoint.attempt -isnot [long]) -or
        [int]$checkpoint.attempt -ne $Attempt -or
        $checkpoint.authoritative -isnot [bool] -or [bool]$checkpoint.authoritative -or
        [string]$checkpoint.state -cne "executing" -or
        [string]$checkpoint.stage -notin @("manual", "diagnostic-manual") -or
        [string]$checkpoint.source_verification_state -cne "verified" -or
        [string]$checkpoint.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$checkpoint.environment_fingerprint -cne $EnvironmentFingerprint -or
        @($checkpoint.prerequisite_receipts).Count -ne 3 -or
        @($checkpoint.manual_scenarios_pending | Where-Object { [string]$_ -ceq $Scenario }).Count -ne 1 -or
        $ScenarioStartedAt -lt $scriptedCompletedAt) {
        throw "Manual UAT scenario '$Scenario' is not bound to its exact authenticated Start checkpoint."
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = $sha256 }
        receipt = $checkpoint
    }
}

function Assert-Sprint8AManualUatAttemptOpen {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)]$Checkpoint,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $attemptPath = Join-Path $evidenceRootFullPath "attempts/uat-$Attempt.json"
    $attemptRepositoryPath = [IO.Path]::GetRelativePath($RepositoryRoot, $attemptPath).Replace("\", "/")
    $attemptReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path $attemptRepositoryPath
    $attemptSha = Assert-Sprint8AReceiptSidecar -Path ([string]$attemptReference.full_path)
    $attemptReceipt = Get-Content -LiteralPath ([string]$attemptReference.full_path) -Raw | ConvertFrom-Json
    if (($attemptReceipt | ConvertTo-Json -Depth 50 -Compress) -cne
        ($Checkpoint.receipt | ConvertTo-Json -Depth 50 -Compress)) {
        throw "Manual UAT publication requires the mutable attempt to remain at its exact awaiting-manual checkpoint."
    }
    $canonicalResultPath = Join-Path $evidenceRootFullPath "uat-result.json"
    if ((Test-Path -LiteralPath $canonicalResultPath -PathType Leaf) -or
        (Test-Path -LiteralPath "$canonicalResultPath.sha256" -PathType Leaf)) {
        throw "Manual UAT publication is forbidden after canonical UAT result publication begins."
    }
    $manifestPath = Join-Path $evidenceRootFullPath "evidence-manifest.json"
    Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or
        [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
        throw "Manual UAT publication requires a valid lifecycle evidence manifest."
    }
    $checkpointResolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$Checkpoint.reference.path)
    $requiredManifestReferences = @(
        [pscustomobject][ordered]@{ path = [string]$Checkpoint.reference.path; sha256 = [string]$Checkpoint.reference.sha256 },
        [pscustomobject][ordered]@{ path = [string]$attemptReference.path; sha256 = $attemptSha },
        [pscustomobject][ordered]@{
            path = "$([string]$Checkpoint.reference.path).sha256"
            sha256 = Get-Sprint8AFileSha256 -Path "$([string]$checkpointResolved.full_path).sha256"
        },
        [pscustomobject][ordered]@{
            path = "$([string]$attemptReference.path).sha256"
            sha256 = Get-Sprint8AFileSha256 -Path "$([string]$attemptReference.full_path).sha256"
        }
    )
    foreach ($required in $requiredManifestReferences) {
        $matches = @($manifest.entries | Where-Object {
            [string]$_.path -ceq [string]$required.path -and
                [string]$_.sha256 -ceq [string]$required.sha256
        })
        if ($matches.Count -ne 1) {
            throw "Manual UAT publication cannot authenticate open-attempt manifest entry '$($required.path)'."
        }
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$attemptReference.path; sha256 = $attemptSha }
        receipt = $attemptReceipt
    }
}

function Assert-Sprint8AManualUatResumeMarker {
    param(
        [Parameter(Mandatory)]$MarkerReference,
        [Parameter(Mandatory)]$LeaseReference,
        [Parameter(Mandatory)]$LeaseDocument,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [ValidateRange(0, [int]::MaxValue)][int]$ExpectedCurrentProcessId = 0
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $marker = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$MarkerReference.path)
    $expectedPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-resume.json")
    ).Replace("\", "/")
    $markerSha = Assert-Sprint8AReceiptSidecar -Path ([string]$marker.full_path)
    $document = Get-Content -LiteralPath ([string]$marker.full_path) -Raw | ConvertFrom-Json
    $resumedAt = ConvertTo-Sprint8ADateTimeOffset `
        -Value $document.resumed_at `
        -Label "manual UAT execution-resume time"
    if ([string]$marker.path -cne $expectedPath -or
        $markerSha -cne [string]$MarkerReference.sha256 -or
        ($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 1 -or [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "uat-manual-execution-resume" -or
        [string]$document.state -cne "resumed" -or $document.authoritative -isnot [bool] -or
        [bool]$document.authoritative -or [int]$document.attempt -ne $Attempt -or
        [string]$document.scenario -cne $Scenario -or
        [string]$document.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$document.environment_fingerprint -cne $EnvironmentFingerprint -or
        ($document.original_process_id -isnot [int] -and $document.original_process_id -isnot [long]) -or
        [int]$document.original_process_id -lt 1 -or
        ($document.current_process_id -isnot [int] -and $document.current_process_id -isnot [long]) -or
        [int]$document.current_process_id -lt 1 -or
        ($LeaseDocument.process_id -isnot [int] -and $LeaseDocument.process_id -isnot [long]) -or
        [int]$document.original_process_id -ne [int]$LeaseDocument.process_id -or
        ($ExpectedCurrentProcessId -gt 0 -and [int]$document.current_process_id -ne $ExpectedCurrentProcessId) -or
        $resumedAt -lt $StartedAt -or
        [string]$document.lease.path -cne [string]$LeaseReference.path -or
        [string]$document.lease.sha256 -cne [string]$LeaseReference.sha256 -or
        [string]$document.checkpoint.path -cne [string]$LeaseDocument.checkpoint.path -or
        [string]$document.checkpoint.sha256 -cne [string]$LeaseDocument.checkpoint.sha256) {
        throw "Manual UAT scenario '$Scenario' execution-resume marker is malformed or bound to another process lineage."
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$marker.path; sha256 = $markerSha }
        document = $document
    }
}

function Open-Sprint8AManualUatScenarioLease {
    param(
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [bool]$Authoritative = $true,
        [bool]$Diagnostic = $false,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$Resume
    )

    if ((Get-Sprint8AManualUatScenarioNames) -cnotcontains $Scenario) {
        throw "Unknown Sprint 8A manual UAT scenario '$Scenario'."
    }
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $stream = Open-Sprint8AValidationAttemptLock -Path (Join-Path $evidenceRootFullPath "validation-attempt.lock")
    try {
        $checkpoint = Assert-Sprint8AManualUatStartCheckpoint `
            -Attempt $Attempt `
            -Scenario $Scenario `
            -CandidateFingerprint $CandidateFingerprint `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -ScenarioStartedAt $StartedAt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath
        Assert-Sprint8AManualUatAttemptOpen `
            -Attempt $Attempt `
            -Checkpoint $checkpoint `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath | Out-Null
        if ([string]$checkpoint.receipt.stage -ceq "diagnostic-manual" -and ($Authoritative -or -not $Diagnostic)) {
            throw "Manual UAT scenario '$Scenario' execution must be diagnostic after scripted invalidation."
        }
        $manualRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual"
        foreach ($priorPath in @(Get-ChildItem -LiteralPath $manualRoot -Filter "uat-8a-*.json" -File -ErrorAction SilentlyContinue)) {
            Assert-Sprint8AReceiptSidecar -Path $priorPath.FullName | Out-Null
            $prior = Get-Content -LiteralPath $priorPath.FullName -Raw | ConvertFrom-Json
            if ((ConvertTo-Sprint8ADateTimeOffset -Value $prior.ended_at -Label "prior manual UAT end") -ge $StartedAt) { continue }
            if ([string]$prior.state -ceq "blocked" -and [string]$prior.classification -ceq "product-decision") {
                throw "Manual UAT scenario '$Scenario' execution is paused by unresolved product decision '$($prior.scenario)'."
            }
            if ([string]$prior.state -ceq "failed" -and [bool]$prior.authoritative -and
                ($Authoritative -or -not $Diagnostic)) {
                throw "Manual UAT scenario '$Scenario' execution must be diagnostic after '$($prior.scenario)' invalidated the candidate."
            }
        }
        $leaseRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases"
        [IO.Directory]::CreateDirectory($leaseRoot) | Out-Null
        $leasePath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-start.json"
        $resumePath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-resume.json"
        $completionPath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-complete.json"
        $preparedPublicationPath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-publication-prepared.json"
        Repair-Sprint7AEvidencePublication -Path $preparedPublicationPath
        if ((Test-Path -LiteralPath $preparedPublicationPath -PathType Leaf) -or
            (Test-Path -LiteralPath "$preparedPublicationPath.sha256" -PathType Leaf)) {
            throw "Manual UAT scenario '$Scenario' already has a prepared publication; repair or finalize it without rerunning the scenario."
        }
        if (Test-Path -LiteralPath $completionPath -PathType Leaf) {
            throw "Manual UAT scenario '$Scenario' already has a completed execution lease."
        }
        $resumeReference = $null
        $resumed = $false
        $originalProcessId = 0
        $currentProcessId = $PID
        if (Test-Path -LiteralPath $leasePath -PathType Leaf) {
            if (-not $Resume) {
                throw "Manual UAT scenario '$Scenario' has an interrupted execution lease; resume it explicitly."
            }
            $leaseSha = Assert-Sprint8AReceiptSidecar -Path $leasePath
            $leaseDocument = Get-Content -LiteralPath $leasePath -Raw | ConvertFrom-Json
            $leaseStartedAt = ConvertTo-Sprint8ADateTimeOffset `
                -Value $leaseDocument.started_at `
                -Label "interrupted manual UAT execution-lease start"
            if (($leaseDocument.schema_version -isnot [int] -and $leaseDocument.schema_version -isnot [long]) -or
                [int]$leaseDocument.schema_version -ne 1 -or [string]$leaseDocument.sprint -cne "sprint-8a" -or
                [string]$leaseDocument.phase -cne "uat-manual-execution-lease" -or
                [string]$leaseDocument.state -cne "executing" -or
                [string]$leaseDocument.scenario -cne $Scenario -or [int]$leaseDocument.attempt -ne $Attempt -or
                [string]$leaseDocument.candidate_fingerprint -cne $CandidateFingerprint -or
                [string]$leaseDocument.environment_fingerprint -cne $EnvironmentFingerprint -or
                $leaseStartedAt -ne $StartedAt -or
                $leaseDocument.authoritative -isnot [bool] -or [bool]$leaseDocument.authoritative -ne $Authoritative -or
                $leaseDocument.diagnostic -isnot [bool] -or [bool]$leaseDocument.diagnostic -ne $Diagnostic -or
                ($leaseDocument.process_id -isnot [int] -and $leaseDocument.process_id -isnot [long]) -or
                [int]$leaseDocument.process_id -lt 1 -or
                [string]$leaseDocument.checkpoint.path -cne [string]$checkpoint.reference.path -or
                [string]$leaseDocument.checkpoint.sha256 -cne [string]$checkpoint.reference.sha256) {
                throw "Manual UAT scenario '$Scenario' interrupted lease is bound to another execution."
            }
            $originalProcessId = [int]$leaseDocument.process_id
            Repair-Sprint7AEvidencePublication -Path $resumePath
            if (Test-Path -LiteralPath $resumePath -PathType Leaf) {
                $resumeReference = [pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($RepositoryRoot, $resumePath).Replace("\", "/")
                    sha256 = Assert-Sprint8AReceiptSidecar -Path $resumePath
                }
                $resumeMarker = Assert-Sprint8AManualUatResumeMarker `
                    -MarkerReference $resumeReference `
                    -LeaseReference ([pscustomobject]@{
                        path = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
                        sha256 = $leaseSha
                    }) `
                    -LeaseDocument $leaseDocument `
                    -Scenario $Scenario `
                    -Attempt $Attempt `
                    -CandidateFingerprint $CandidateFingerprint `
                    -EnvironmentFingerprint $EnvironmentFingerprint `
                    -StartedAt $StartedAt `
                    -RepositoryRoot $RepositoryRoot `
                    -EvidenceRoot $evidenceRootFullPath `
                    -ExpectedCurrentProcessId $PID
                $resumeReference = $resumeMarker.reference
            } else {
                $resumeDocument = [pscustomobject][ordered]@{
                    schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-execution-resume"
                    authoritative = $false; state = "resumed"; attempt = $Attempt; scenario = $Scenario
                    candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
                    original_process_id = $originalProcessId; current_process_id = $PID
                    resumed_at = [DateTimeOffset]::UtcNow.ToString("o")
                    lease = [pscustomobject][ordered]@{
                        path = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
                        sha256 = $leaseSha
                    }
                    checkpoint = $checkpoint.reference
                }
                Publish-Sprint7AEvidence -Document $resumeDocument -OutputPath $resumePath | Out-Null
                $resumeReference = [pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($RepositoryRoot, $resumePath).Replace("\", "/")
                    sha256 = Assert-Sprint8AReceiptSidecar -Path $resumePath
                }
            }
            $resumed = $true
            $manifestFullPath = Join-Path $evidenceRootFullPath "evidence-manifest.json"
            Assert-Sprint8AReceiptSidecar -Path $manifestFullPath | Out-Null
            $existingManifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
            $entries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -Overrides @([pscustomobject]@{
                    path = [string]$resumeReference.path; sha256 = [string]$resumeReference.sha256
                    phase = "uat-manual-resume"; authoritative = $false; status = "resumed"
                })
            $authorizedReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
                -ExistingEntries @($existingManifest.entries) `
                -UpdatedEntries $entries)
            Publish-Sprint8AEvidenceManifest `
                -Entries $entries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -OutputPath ([IO.Path]::GetRelativePath($RepositoryRoot, $manifestFullPath).Replace("\", "/")) `
                -Merge `
                -AuthorizedReplacementPaths $authorizedReplacementPaths | Out-Null
        } else {
            if ($Resume) { throw "Manual UAT scenario '$Scenario' has no interrupted lease to resume." }
            $leaseDocument = [pscustomobject][ordered]@{
                schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-execution-lease"
                authoritative = $Authoritative; diagnostic = $Diagnostic; state = "executing"
                attempt = $Attempt; scenario = $Scenario
                candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
                started_at = $StartedAt.ToString("o"); process_id = $PID; checkpoint = $checkpoint.reference
            }
            Publish-Sprint7AEvidence -Document $leaseDocument -OutputPath $leasePath | Out-Null
            $leaseSha = Assert-Sprint8AReceiptSidecar -Path $leasePath
            $originalProcessId = $PID
            $leaseRelative = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
            $entries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -Overrides @([pscustomobject]@{
                    path = $leaseRelative; sha256 = $leaseSha; phase = "uat-manual-lease"
                    authoritative = $false; status = "executing"
                })
            Publish-Sprint8AEvidenceManifest `
                -Entries $entries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -OutputPath ([IO.Path]::GetRelativePath($RepositoryRoot, (Join-Path $evidenceRootFullPath "evidence-manifest.json")).Replace("\", "/")) `
                -Merge | Out-Null
        }
        [pscustomobject][ordered]@{
            schema_version = 1; stream = $stream; attempt = $Attempt; scenario = $Scenario
            candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
            started_at = $StartedAt.ToString("o"); authoritative = $Authoritative; diagnostic = $Diagnostic
            resumed = $resumed; original_process_id = $originalProcessId; current_process_id = $currentProcessId
            checkpoint = $checkpoint.reference
            lease = [pscustomobject]@{
                path = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
                sha256 = $leaseSha
            }
            resume = $resumeReference
            completion_path = [IO.Path]::GetRelativePath($RepositoryRoot, $completionPath).Replace("\", "/")
        }
    } catch {
        $stream.Dispose()
        throw
    }
}

function Assert-Sprint8ANoOpenManualUatLeases {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $leaseRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases"
    if (-not (Test-Path -LiteralPath $leaseRoot -PathType Container)) { return }
    $open = @()
    foreach ($start in @(Get-ChildItem -LiteralPath $leaseRoot -Filter "*-start.json" -File)) {
        Assert-Sprint8AReceiptSidecar -Path $start.FullName | Out-Null
        $completion = $start.FullName.Substring(0, $start.FullName.Length - "-start.json".Length) + "-complete.json"
        if (-not (Test-Path -LiteralPath $completion -PathType Leaf)) {
            $open += $start.Name
        } else {
            Assert-Sprint8AReceiptSidecar -Path $completion | Out-Null
        }
    }
    if ($open.Count -gt 0) {
        throw "Formal UAT finalization is blocked by open or interrupted manual execution lease(s): $($open -join ', ')."
    }
}

function Assert-Sprint8AManualUatPreparedPublicationCheckpoint {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$CheckpointReference,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $scenario = [string]$Document.scenario
    if ((Get-Sprint8AManualUatScenarioNames) -cnotcontains $scenario) {
        throw "Manual UAT prepared publication names unknown scenario '$scenario'."
    }
    $checkpoint = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$CheckpointReference.path)
    $scenarioName = $scenario.ToLowerInvariant()
    $expectedCheckpointPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$ExpectedAttempt/manual-leases/$scenarioName-publication-prepared.json")
    ).Replace("\", "/")
    $expectedReceiptPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$ExpectedAttempt/manual/$scenarioName.json")
    ).Replace("\", "/")
    $expectedCompletionPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$ExpectedAttempt/manual-leases/$scenarioName-complete.json")
    ).Replace("\", "/")
    $checkpointSha = Assert-Sprint8AReceiptSidecar -Path ([string]$checkpoint.full_path)
    $preparedAt = ConvertTo-Sprint8ADateTimeOffset `
        -Value $Document.prepared_at `
        -Label "manual UAT prepared-publication time"
    $receipt = $Document.receipt.document
    Assert-Sprint8AManualUatReceipt `
        -Receipt $receipt `
        -ExpectedScenario $scenario `
        -ExpectedAttempt $ExpectedAttempt `
        -CandidateFingerprint ([string]$Document.candidate_fingerprint) `
        -EnvironmentFingerprint ([string]$Document.environment_fingerprint) `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    $receiptSha = Get-Sprint8AStringSha256 -Text (($receipt | ConvertTo-Json -Depth 30) + "`n")
    $completionSha = Get-Sprint8AStringSha256 -Text (($Document.completion.document | ConvertTo-Json -Depth 30) + "`n")
    $receiptEndedAt = ConvertTo-Sprint8ADateTimeOffset -Value $receipt.ended_at -Label "manual UAT receipt end"
    $leaseBinding = ConvertTo-Json -InputObject $receipt.execution_lease -Depth 10 -Compress
    $resumeBinding = ConvertTo-Json -InputObject $receipt.execution_resume -Depth 10 -Compress
    if (($Document.schema_version -isnot [int] -and $Document.schema_version -isnot [long]) -or
        [int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8a" -or
        [string]$Document.phase -cne "uat-manual-publication-checkpoint" -or
        [string]$Document.state -cne "prepared" -or $Document.authoritative -isnot [bool] -or
        [bool]$Document.authoritative -or [int]$Document.attempt -ne $ExpectedAttempt -or
        [string]$Document.candidate_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$Document.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$checkpoint.path -cne $expectedCheckpointPath -or
        $checkpointSha -cne [string]$CheckpointReference.sha256 -or
        $preparedAt -lt $receiptEndedAt -or
        [string]$Document.receipt.path -cne $expectedReceiptPath -or
        [string]$Document.receipt.sha256 -cne $receiptSha -or
        [string]$Document.completion.path -cne $expectedCompletionPath -or
        [string]$Document.completion.sha256 -cne $completionSha -or
        (ConvertTo-Json -InputObject $Document.lease -Depth 10 -Compress) -cne $leaseBinding -or
        (ConvertTo-Json -InputObject $Document.resume -Depth 10 -Compress) -cne $resumeBinding -or
        [string]$Document.completion.document.phase -cne "uat-manual-execution-lease" -or
        [string]$Document.completion.document.state -cne "completed" -or
        [int]$Document.completion.document.attempt -ne $ExpectedAttempt -or
        [string]$Document.completion.document.scenario -cne $scenario -or
        (ConvertTo-Json -InputObject $Document.completion.document.lease -Depth 10 -Compress) -cne $leaseBinding -or
        $Document.completion.document.resumed -isnot [bool] -or
        [bool]$Document.completion.document.resumed -ne [bool]$receipt.resumed -or
        (ConvertTo-Json -InputObject $Document.completion.document.resume -Depth 10 -Compress) -cne $resumeBinding -or
        [string]$Document.completion.document.receipt.path -cne $expectedReceiptPath -or
        [string]$Document.completion.document.receipt.sha256 -cne $receiptSha) {
        throw "Manual UAT prepared publication for '$scenario' is malformed or not bound to its exact receipt, completion, and lease lineage (checkpoint '$([string]$checkpoint.path)'/'$expectedCheckpointPath'; receipt '$([string]$Document.receipt.path)'/'$expectedReceiptPath' $([string]$Document.receipt.sha256)/$receiptSha; completion '$([string]$Document.completion.path)'/'$expectedCompletionPath' $([string]$Document.completion.sha256)/$completionSha)."
    }
    [pscustomobject][ordered]@{
        checkpoint = [pscustomobject][ordered]@{ path = [string]$checkpoint.path; sha256 = $checkpointSha }
        receipt = [pscustomobject][ordered]@{
            path = $expectedReceiptPath; full_path = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expectedReceiptPath))
            sha256 = $receiptSha; document = $receipt
        }
        completion = [pscustomobject][ordered]@{
            path = $expectedCompletionPath; full_path = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expectedCompletionPath))
            sha256 = $completionSha; document = $Document.completion.document
        }
    }
}

function Complete-Sprint8AManualUatPreparedEvidencePair {
    param(
        [Parameter(Mandatory)]$PreparedEvidence,
        [Parameter(Mandatory)][string]$Label
    )

    $path = [string]$PreparedEvidence.full_path
    Repair-Sprint7AEvidencePublication -Path $path
    $artifactExists = Test-Path -LiteralPath $path -PathType Leaf
    $sidecarExists = Test-Path -LiteralPath "$path.sha256" -PathType Leaf
    if ($artifactExists -or $sidecarExists) {
        if (-not $artifactExists -or -not $sidecarExists -or
            (Assert-Sprint8AReceiptSidecar -Path $path) -cne [string]$PreparedEvidence.sha256) {
            throw "Manual UAT $Label prepared publication differs from its immutable checkpoint."
        }
    } else {
        Publish-Sprint7AEvidence -Document $PreparedEvidence.document -OutputPath $path | Out-Null
        if ((Assert-Sprint8AReceiptSidecar -Path $path) -cne [string]$PreparedEvidence.sha256) {
            throw "Manual UAT $Label publication differs from its immutable prepared bytes."
        }
    }
    [pscustomobject][ordered]@{ path = [string]$PreparedEvidence.path; sha256 = [string]$PreparedEvidence.sha256 }
}

function Repair-Sprint8AManualUatPreparedPublication {
    param(
        [Parameter(Mandatory)][string]$CheckpointPath,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidenceRootFullPath
    Repair-Sprint7AEvidencePublication -Path $CheckpointPath
    $checkpointRepositoryPath = if ([IO.Path]::IsPathRooted($CheckpointPath)) {
        [IO.Path]::GetRelativePath($RepositoryRoot, [IO.Path]::GetFullPath($CheckpointPath)).Replace("\", "/")
    } else { $CheckpointPath.Replace("\", "/") }
    $checkpoint = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path $checkpointRepositoryPath
    $checkpointReference = [pscustomobject][ordered]@{
        path = [string]$checkpoint.path
        sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$checkpoint.full_path)
    }
    $document = Get-Content -LiteralPath ([string]$checkpoint.full_path) -Raw | ConvertFrom-Json
    $prepared = Assert-Sprint8AManualUatPreparedPublicationCheckpoint `
        -Document $document `
        -CheckpointReference $checkpointReference `
        -ExpectedAttempt $Attempt `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath
    $receiptReference = Complete-Sprint8AManualUatPreparedEvidencePair `
        -PreparedEvidence $prepared.receipt `
        -Label "scenario receipt"
    $completionReference = Complete-Sprint8AManualUatPreparedEvidencePair `
        -PreparedEvidence $prepared.completion `
        -Label "execution-lease completion"
    Assert-Sprint8AManualUatExecutionLeasePair `
        -Receipt $prepared.receipt.document `
        -ReceiptReference $receiptReference `
        -ExpectedScenario ([string]$document.scenario) `
        -ExpectedAttempt $Attempt `
        -CandidateFingerprint ([string]$document.candidate_fingerprint) `
        -EnvironmentFingerprint ([string]$document.environment_fingerprint) `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    $manifestFullPath = Join-Path $evidenceRootFullPath "evidence-manifest.json"
    Assert-Sprint8AReceiptSidecar -Path $manifestFullPath | Out-Null
    $existingManifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    $overrides = @(
        [pscustomobject][ordered]@{
            path = [string]$prepared.checkpoint.path; sha256 = [string]$prepared.checkpoint.sha256
            phase = "uat-manual-publication"; authoritative = $false; status = "prepared"
        },
        [pscustomobject][ordered]@{
            path = [string]$receiptReference.path; sha256 = [string]$receiptReference.sha256
            phase = "uat-manual"; authoritative = [bool]$prepared.receipt.document.authoritative
            status = if ([bool]$prepared.receipt.document.diagnostic) {
                "diagnostic-$([string]$prepared.receipt.document.state)"
            } else { [string]$prepared.receipt.document.state }
        },
        [pscustomobject][ordered]@{
            path = [string]$completionReference.path; sha256 = [string]$completionReference.sha256
            phase = "uat-manual-lease"; authoritative = $false; status = "completed"
        }
    )
    if ([bool]$prepared.receipt.document.resumed) {
        $overrides += [pscustomobject][ordered]@{
            path = [string]$prepared.receipt.document.execution_resume.path
            sha256 = [string]$prepared.receipt.document.execution_resume.sha256
            phase = "uat-manual-resume"; authoritative = $false; status = "resumed"
        }
    }
    $manifestEntries = Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Overrides $overrides
    $authorizedReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
        -ExistingEntries @($existingManifest.entries) `
        -UpdatedEntries $manifestEntries)
    Publish-Sprint8AEvidenceManifest `
        -Entries $manifestEntries `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -OutputPath ([IO.Path]::GetRelativePath($RepositoryRoot, $manifestFullPath).Replace("\", "/")) `
        -Merge `
        -AuthorizedReplacementPaths $authorizedReplacementPaths | Out-Null
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -ManifestPath $manifestFullPath `
        -AllowMissingCanonicalUatCommitment:$AllowMissingCanonicalUatCommitment | Out-Null
    [pscustomobject][ordered]@{
        scenario = [string]$document.scenario
        checkpoint = $prepared.checkpoint
        receipt = $receiptReference
        completion = $completionReference
    }
}

function Repair-Sprint8AManualUatPreparedPublications {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidenceRootFullPath
    $leaseRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases"
    if (-not (Test-Path -LiteralPath $leaseRoot -PathType Container)) { return @() }
    $checkpointPaths = @(Get-ChildItem -LiteralPath $leaseRoot -Force -File | Where-Object {
        $_.Name -cmatch '^uat-8a-[0-9]{2}-publication-prepared\.json(?:\.sha256)?$'
    } | ForEach-Object {
        if ($_.Name.EndsWith(".sha256", [StringComparison]::Ordinal)) {
            $_.FullName.Substring(0, $_.FullName.Length - ".sha256".Length)
        } else { $_.FullName }
    } | Sort-Object -CaseSensitive -Unique)
    @($checkpointPaths | ForEach-Object {
        Repair-Sprint8AManualUatPreparedPublication `
            -CheckpointPath $_ `
            -Attempt $Attempt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath `
            -AllowMissingCanonicalUatCommitment:$AllowMissingCanonicalUatCommitment
    })
}

function Publish-Sprint8AManualUatReceipt {
    param(
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [ValidateSet("passed", "failed", "blocked")][string]$State = "passed",
        [bool]$Authoritative = $true,
        [bool]$Diagnostic = $false,
        [bool]$AssertionsStarted = $true,
        [AllowNull()][string]$Classification,
        [AllowNull()][string]$FailureMessage,
        [AllowNull()][string]$BlockedReason,
        [Parameter(Mandatory)][string]$Role,
        [Parameter(Mandatory)]$TesterIdentity,
        [Parameter(Mandatory)][object[]]$Preconditions,
        [Parameter(Mandatory)][object[]]$StartingState,
        [Parameter(Mandatory)][object[]]$Actions,
        [Parameter(Mandatory)][string]$ExpectedResult,
        [Parameter(Mandatory)][string]$ActualResult,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Evidence,
        [Parameter(Mandatory)][object[]]$CleanupEvidence,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)][DateTimeOffset]$EndedAt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$OutputPath,
        [Parameter(Mandatory)]$ExecutionLease
    )

    if ($EndedAt -lt $StartedAt) { throw "Manual UAT scenario '$Scenario' has invalid chronology." }
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    if ($ExecutionLease.PSObject.Properties.Name -notcontains "stream" -or
        $ExecutionLease.stream -isnot [IO.FileStream] -or -not $ExecutionLease.stream.CanWrite -or
        $ExecutionLease.PSObject.Properties.Name -notcontains "resumed" -or
        $ExecutionLease.resumed -isnot [bool] -or
        $ExecutionLease.PSObject.Properties.Name -notcontains "resume" -or
        ($ExecutionLease.original_process_id -isnot [int] -and $ExecutionLease.original_process_id -isnot [long]) -or
        [int]$ExecutionLease.original_process_id -lt 1 -or
        ($ExecutionLease.current_process_id -isnot [int] -and $ExecutionLease.current_process_id -isnot [long]) -or
        [int]$ExecutionLease.current_process_id -ne $PID -or
        [int]$ExecutionLease.attempt -ne $Attempt -or [string]$ExecutionLease.scenario -cne $Scenario -or
        [string]$ExecutionLease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$ExecutionLease.environment_fingerprint -cne $EnvironmentFingerprint -or
        [string]$ExecutionLease.started_at -cne $StartedAt.ToString("o") -or
        [bool]$ExecutionLease.authoritative -ne $Authoritative -or [bool]$ExecutionLease.diagnostic -ne $Diagnostic) {
        throw "Manual UAT scenario '$Scenario' publication requires its exact live execution lease."
    }
    if ([bool]$ExecutionLease.resumed -and $Authoritative -and $State -ceq "passed") {
        throw "Manual UAT scenario '$Scenario' cannot publish an authoritative pass after execution resume; retain a failed or blocked result."
    }
    $expectedLockPath = [IO.Path]::GetFullPath((Join-Path $evidenceRootFullPath "validation-attempt.lock"))
    if ([IO.Path]::GetFullPath([string]$ExecutionLease.stream.Name) -cne $expectedLockPath) {
        throw "Manual UAT scenario '$Scenario' execution lease does not hold the canonical validation lock."
    }
    $lockIsExclusive = $false
    try {
        $concurrentLock = Open-Sprint8AValidationAttemptLock -Path $expectedLockPath
        $concurrentLock.Dispose()
    } catch [IO.IOException] {
        $lockIsExclusive = $true
    }
    if (-not $lockIsExclusive) {
        throw "Manual UAT scenario '$Scenario' execution lease does not exclude concurrent finalization."
    }
    $leaseReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$ExecutionLease.lease.path)
    $expectedLeasePath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-start.json")
    ).Replace("\", "/")
    $retainedLeaseSha = Assert-Sprint8AReceiptSidecar -Path ([string]$leaseReference.full_path)
    $retainedLease = Get-Content -LiteralPath ([string]$leaseReference.full_path) -Raw | ConvertFrom-Json
    $retainedLeaseStartedAt = ConvertTo-Sprint8ADateTimeOffset `
        -Value $retainedLease.started_at `
        -Label "retained manual UAT execution-lease start"
    if ([string]$leaseReference.path -cne $expectedLeasePath -or
        $retainedLeaseSha -cne [string]$ExecutionLease.lease.sha256 -or
        ($retainedLease.schema_version -isnot [int] -and $retainedLease.schema_version -isnot [long]) -or
        [int]$retainedLease.schema_version -ne 1 -or [string]$retainedLease.sprint -cne "sprint-8a" -or
        [string]$retainedLease.phase -cne "uat-manual-execution-lease" -or [string]$retainedLease.state -cne "executing" -or
        [int]$retainedLease.attempt -ne $Attempt -or [string]$retainedLease.scenario -cne $Scenario -or
        [string]$retainedLease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$retainedLease.environment_fingerprint -cne $EnvironmentFingerprint -or
        $retainedLeaseStartedAt -ne $StartedAt -or
        $retainedLease.authoritative -isnot [bool] -or [bool]$retainedLease.authoritative -ne $Authoritative -or
        $retainedLease.diagnostic -isnot [bool] -or [bool]$retainedLease.diagnostic -ne $Diagnostic -or
        ($retainedLease.process_id -isnot [int] -and $retainedLease.process_id -isnot [long]) -or
        [int]$retainedLease.process_id -ne [int]$ExecutionLease.original_process_id -or
        [string]$retainedLease.checkpoint.path -cne [string]$ExecutionLease.checkpoint.path -or
        [string]$retainedLease.checkpoint.sha256 -cne [string]$ExecutionLease.checkpoint.sha256) {
        throw "Manual UAT scenario '$Scenario' retained execution lease is malformed or bound to another execution."
    }
    if ([bool]$ExecutionLease.resumed) {
        if ($null -eq $ExecutionLease.resume) {
            throw "Manual UAT scenario '$Scenario' resumed execution omits its authenticated marker."
        }
        Assert-Sprint8AManualUatResumeMarker `
            -MarkerReference $ExecutionLease.resume `
            -LeaseReference $ExecutionLease.lease `
            -LeaseDocument $retainedLease `
            -Scenario $Scenario `
            -Attempt $Attempt `
            -CandidateFingerprint $CandidateFingerprint `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -StartedAt $StartedAt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath `
            -ExpectedCurrentProcessId $PID | Out-Null
    } else {
        $canonicalResumePath = [string]$leaseReference.full_path
        if (-not $canonicalResumePath.EndsWith("-start.json", [StringComparison]::Ordinal)) {
            throw "Manual UAT scenario '$Scenario' retained execution lease path is non-canonical."
        }
        $canonicalResumePath = $canonicalResumePath.Substring(
            0,
            $canonicalResumePath.Length - "-start.json".Length
        ) + "-resume.json"
        Repair-Sprint7AEvidencePublication -Path $canonicalResumePath
        if ($null -ne $ExecutionLease.resume -or
            [int]$ExecutionLease.current_process_id -ne [int]$ExecutionLease.original_process_id -or
            (Test-Path -LiteralPath $canonicalResumePath -PathType Leaf) -or
            (Test-Path -LiteralPath "$canonicalResumePath.sha256" -PathType Leaf)) {
            throw "Manual UAT scenario '$Scenario' non-resumed execution has inconsistent process lineage."
        }
    }
    $attemptLock = $ExecutionLease.stream
    try {
    $resolveEvidence = {
        param(
            [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$References,
            [Parameter(Mandatory)][ValidateSet("scenario", "cleanup")][string]$Kind
        )
        @($References | ForEach-Object {
        $reference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$_.path)
        if ((Get-Sprint8AFileSha256 -Path ([string]$reference.full_path)) -cne [string]$_.sha256) {
            throw "Manual UAT scenario '$Scenario' evidence digest does not match '$($_.path)'."
        }
        if ($Kind -ceq "scenario") {
            if (($_.step -isnot [int] -and $_.step -isnot [long])) {
                throw "Manual UAT scenario '$Scenario' evidence requires a typed step number."
            }
            [pscustomobject][ordered]@{
                step = [int]$_.step
                requirement_id = [string]$_.requirement_id
                kind = [string]$_.kind
                capture = if ($_.PSObject.Properties.Name -contains "capture") { $_.capture } else { $null }
                path = [string]$reference.path
                sha256 = [string]$_.sha256
            }
        } else {
            if ([string]$_.kind -cne "canonical-restoration") {
                throw "Manual UAT scenario '$Scenario' cleanup evidence requires kind 'canonical-restoration'."
            }
            [pscustomobject][ordered]@{
                kind = "canonical-restoration"
                path = [string]$reference.path
                sha256 = [string]$_.sha256
            }
        }
        })
    }
    $evidenceReferences = @(& $resolveEvidence -References $Evidence -Kind "scenario")
    $cleanupEvidenceReferences = @(& $resolveEvidence -References $CleanupEvidence -Kind "cleanup")
    $target = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $OutputPath
    $expectedOutputPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual/$($Scenario.ToLowerInvariant()).json")
    ).Replace("\", "/")
    if ([string]$target.path -cne $expectedOutputPath) {
        throw "Manual UAT scenario '$Scenario' must publish to '$expectedOutputPath'."
    }
    $checkpoint = Assert-Sprint8AManualUatStartCheckpoint `
        -Attempt $Attempt `
        -Scenario $Scenario `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -ScenarioStartedAt $StartedAt `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath
    Assert-Sprint8AManualUatAttemptOpen `
        -Attempt $Attempt `
        -Checkpoint $checkpoint `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    if ([string]$checkpoint.receipt.stage -ceq "diagnostic-manual" -and
        ($Authoritative -or -not $Diagnostic)) {
        throw "Manual UAT scenario '$Scenario' must be non-authoritative diagnostic harvest after scripted invalidation."
    }
    foreach ($priorPath in @(Get-ChildItem -LiteralPath (Split-Path -Parent ([string]$target.full_path)) -Filter "uat-8a-*.json" -File -ErrorAction SilentlyContinue)) {
        if ([IO.Path]::GetFullPath($priorPath.FullName) -ceq [IO.Path]::GetFullPath([string]$target.full_path)) { continue }
        try {
            Assert-Sprint8AReceiptSidecar -Path $priorPath.FullName | Out-Null
            $prior = Get-Content -LiteralPath $priorPath.FullName -Raw | ConvertFrom-Json
            if ([int]$prior.attempt -eq $Attempt -and
                [string]$prior.candidate_fingerprint -ceq $CandidateFingerprint -and
                [string]$prior.environment_fingerprint -ceq $EnvironmentFingerprint -and
                [string]$prior.state -ceq "blocked" -and
                [string]$prior.classification -ceq "product-decision" -and
                (ConvertTo-Sprint8ADateTimeOffset -Value $prior.ended_at -Label "prior manual UAT end") -lt $StartedAt) {
                throw "Manual UAT scenario '$Scenario' is paused by unresolved product decision '$($prior.scenario)'."
            }
            if ([int]$prior.attempt -eq $Attempt -and
                [string]$prior.candidate_fingerprint -ceq $CandidateFingerprint -and
                [string]$prior.environment_fingerprint -ceq $EnvironmentFingerprint -and
                [string]$prior.state -ceq "failed" -and [bool]$prior.authoritative -and
                (ConvertTo-Sprint8ADateTimeOffset -Value $prior.ended_at -Label "prior manual UAT end") -lt $StartedAt -and
                ($Authoritative -or -not $Diagnostic)) {
                throw "Manual UAT scenario '$Scenario' must be diagnostic after '$($prior.scenario)' invalidated the candidate."
            }
        } catch {
            if ($_.Exception.Message -like "Manual UAT scenario '$Scenario' must be diagnostic*" -or
                $_.Exception.Message -like "Manual UAT scenario '$Scenario' is paused by unresolved product decision*") { throw }
            throw "Manual UAT scenario '$Scenario' cannot authenticate prior receipt '$($priorPath.FullName)': $($_.Exception.Message)"
        }
    }
    $scenarioContract = Get-Sprint8AManualUatScenarioContract -Scenario $Scenario
    $receipt = [pscustomobject][ordered]@{
        schema_version = 2
        sprint = "sprint-8a"
        phase = "uat-manual-scenario"
        attempt = $Attempt
        authoritative = $Authoritative
        diagnostic = $Diagnostic
        scenario = $Scenario
        state = $State
        candidate_fingerprint = $CandidateFingerprint
        environment_fingerprint = $EnvironmentFingerprint
        assertions_started = $AssertionsStarted
        assertions_started_at = if ($AssertionsStarted) { $StartedAt.ToString("o") } else { $null }
        started_at = $StartedAt.ToString("o")
        ended_at = $EndedAt.ToString("o")
        duration_ms = [long][Math]::Max(0, ($EndedAt - $StartedAt).TotalMilliseconds)
        role = $Role
        tester_identity = $TesterIdentity
        preconditions = @($Preconditions)
        starting_state = @($StartingState)
        actions = @($Actions)
        expected_result = $ExpectedResult
        actual_result = $ActualResult
        classification = $Classification
        classification_source = if (-not [string]::IsNullOrWhiteSpace($Classification)) { "manual_operator" } else { $null }
        failure_message = $FailureMessage
        blocked_reason = $BlockedReason
        scenario_contract = [pscustomobject][ordered]@{
            manifest = $scenarioContract.manifest
            document = $scenarioContract.document
            acceptance_criteria = @($scenarioContract.acceptance_criteria)
            semantic_predicate_ids = @($scenarioContract.semantic_predicate_ids)
        }
        start_checkpoint = $checkpoint.reference
        execution_lease = $ExecutionLease.lease
        resumed = [bool]$ExecutionLease.resumed
        execution_resume = $ExecutionLease.resume
        evidence = $evidenceReferences
        cleanup_restoration = [pscustomobject][ordered]@{
            required = $true
            result = "canonical_topology_verified"
            evidence = $cleanupEvidenceReferences
        }
    }
    Assert-Sprint8AManualUatReceipt `
        -Receipt $receipt `
        -ExpectedScenario $Scenario `
        -ExpectedAttempt $Attempt `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    $receipt = $receipt | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    Assert-Sprint8AManualUatReceipt `
        -Receipt $receipt `
        -ExpectedScenario $Scenario `
        -ExpectedAttempt $Attempt `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    $expectedReceiptSha = Get-Sprint8AStringSha256 -Text (($receipt | ConvertTo-Json -Depth 30) + "`n")
    $receiptReference = [pscustomobject][ordered]@{
        path = [string]$target.path
        sha256 = $expectedReceiptSha
    }
    $completionTarget = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$ExecutionLease.completion_path)
    $expectedCompletionPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-complete.json")
    ).Replace("\", "/")
    if ([string]$completionTarget.path -cne $expectedCompletionPath) {
        throw "Manual UAT scenario '$Scenario' execution lease has a non-canonical completion path."
    }
    $completionDocument = [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-execution-lease"
        authoritative = $false; state = "completed"; attempt = $Attempt; scenario = $Scenario
        candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
        started_at = $StartedAt.ToString("o"); ended_at = $EndedAt.ToString("o")
        lease = $ExecutionLease.lease; resumed = [bool]$ExecutionLease.resumed
        resume = $ExecutionLease.resume; receipt = $receiptReference
    }
    $completionDocument = $completionDocument | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $completionReference = [pscustomobject][ordered]@{
        path = [string]$completionTarget.path
        sha256 = Get-Sprint8AStringSha256 -Text (($completionDocument | ConvertTo-Json -Depth 30) + "`n")
    }
    $preparedTarget = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([IO.Path]::GetRelativePath(
            $RepositoryRoot,
            (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-publication-prepared.json")
        ).Replace("\", "/"))
    $preparedDocument = [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-publication-checkpoint"
        authoritative = $false; state = "prepared"; attempt = $Attempt; scenario = $Scenario
        candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
        prepared_at = [DateTimeOffset]::UtcNow.ToString("o")
        lease = $ExecutionLease.lease; resume = $ExecutionLease.resume
        receipt = [pscustomobject][ordered]@{
            path = [string]$receiptReference.path; sha256 = [string]$receiptReference.sha256; document = $receipt
        }
        completion = [pscustomobject][ordered]@{
            path = [string]$completionReference.path; sha256 = [string]$completionReference.sha256
            document = $completionDocument
        }
    }
    $expectedPreparedSha = Get-Sprint8AStringSha256 -Text (($preparedDocument | ConvertTo-Json -Depth 30) + "`n")
    Repair-Sprint7AEvidencePublication -Path ([string]$preparedTarget.full_path)
    $preparedArtifactExists = Test-Path -LiteralPath ([string]$preparedTarget.full_path) -PathType Leaf
    $preparedSidecarExists = Test-Path -LiteralPath "$([string]$preparedTarget.full_path).sha256" -PathType Leaf
    if ($preparedArtifactExists -or $preparedSidecarExists) {
        if (-not $preparedArtifactExists -or -not $preparedSidecarExists -or
            (Assert-Sprint8AReceiptSidecar -Path ([string]$preparedTarget.full_path)) -cne $expectedPreparedSha) {
            throw "Manual UAT scenario '$Scenario' already has a different or incomplete prepared publication."
        }
    } else {
        Publish-Sprint7AEvidence -Document $preparedDocument -OutputPath ([string]$preparedTarget.full_path) | Out-Null
    }
    $publication = Repair-Sprint8AManualUatPreparedPublication `
        -CheckpointPath ([string]$preparedTarget.full_path) `
        -Attempt $Attempt `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath
    $publication.receipt
    } finally {
        $attemptLock.Dispose()
    }
}

function Repair-Sprint8AEvidenceRootPublications {
    param([Parameter(Mandatory)][string]$EvidenceRoot)

    $evidence = [IO.Path]::GetFullPath($EvidenceRoot)
    if (-not (Test-Path -LiteralPath $evidence -PathType Container)) {
        throw "Sprint 8A evidence publication recovery root does not exist: '$evidence'."
    }
    $journalSuffix = ".publish-journal.json"
    foreach ($journal in @(Get-ChildItem -LiteralPath $evidence -Recurse -Force -File | Where-Object {
        $_.Name.EndsWith($journalSuffix, [StringComparison]::Ordinal)
    })) {
        $targetPath = $journal.FullName.Substring(0, $journal.FullName.Length - $journalSuffix.Length)
        Repair-Sprint7AEvidencePublication -Path $targetPath
    }
    $remainingControls = @(Get-ChildItem -LiteralPath $evidence -Recurse -Force -File | Where-Object {
        $_.Name.EndsWith($journalSuffix, [StringComparison]::Ordinal) -or
            $_.Name.EndsWith(".rollback", [StringComparison]::Ordinal) -or
            $_.Name -cmatch '^\..+\.[0-9a-f]{32}\.tmp(?:\.sha256)?$' -or
            $_.Name -cmatch '^\..+\.[0-9a-f]{32}\.(?:json|sha256)\.tmp$' -or
            $_.Name -cmatch '^\..+\.publish-journal\.json\.[0-9a-f]{32}\.tmp$'
    } | ForEach-Object {
        [IO.Path]::GetRelativePath($evidence, $_.FullName).Replace("\", "/")
    } | Sort-Object -CaseSensitive)
    if ($remainingControls.Count -gt 0) {
        throw "Sprint 8A evidence inventory found unresolved publisher control file(s): $($remainingControls -join ', ')."
    }
}

function Get-Sprint8AEvidenceFileManifestEntries {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [AllowEmptyCollection()][object[]]$Overrides = @(),
        [switch]$IgnoreExistingManifest
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot))
    }
    if (-not (Test-Path -LiteralPath $evidence -PathType Container)) {
        throw "Sprint 8A evidence root does not exist: '$evidence'."
    }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidence
    $excluded = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($path in @(
        (Join-Path $evidence "evidence-manifest.json"),
        (Join-Path $evidence "evidence-manifest.json.sha256"),
        (Join-Path $evidence "validation-attempt.lock")
    )) {
        $excluded.Add([IO.Path]::GetFullPath($path)) | Out-Null
    }
    $entries = [ordered]@{}
    foreach ($file in @(Get-ChildItem -LiteralPath $evidence -Recurse -Force -File)) {
        if ($excluded.Contains([IO.Path]::GetFullPath($file.FullName))) { continue }
        $path = [IO.Path]::GetRelativePath($repository, $file.FullName).Replace("\", "/")
        $entries[$path] = [pscustomobject][ordered]@{
            path = $path
            sha256 = Get-Sprint8AFileSha256 -Path $file.FullName
            phase = "retained-evidence"
            authoritative = $false
            status = "retained"
        }
    }
    $overridePaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $authorizedMutablePaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $normalizedOverrides = [Collections.Generic.List[object]]::new()
    foreach ($override in @($Overrides)) {
        if ([string]::IsNullOrWhiteSpace([string]$override.path)) {
            throw "Sprint 8A evidence manifest overrides require canonical paths."
        }
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Path ([string]$override.path)
        if ($excluded.Contains([IO.Path]::GetFullPath([string]$resolved.full_path))) {
            throw "Sprint 8A evidence manifest cannot inventory its manifest pair or live attempt lock."
        }
        if (-not $overridePaths.Add([string]$resolved.path)) {
            throw "Sprint 8A evidence manifest overrides contain duplicate path '$($resolved.path)'."
        }
        $authorizedMutablePaths.Add([string]$resolved.path) | Out-Null
        $authorizedMutablePaths.Add("$([string]$resolved.path).sha256") | Out-Null
        $normalizedOverrides.Add([pscustomobject][ordered]@{ override = $override; resolved = $resolved })
    }
    $manifestPath = Join-Path $evidence "evidence-manifest.json"
    if (-not $IgnoreExistingManifest -and (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
        $existingManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        if (($existingManifest.schema_version -isnot [int] -and $existingManifest.schema_version -isnot [long]) -or
            [int]$existingManifest.schema_version -ne 1 -or
            [string]$existingManifest.sprint -cne "sprint-8a" -or
            [string]$existingManifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
            throw "The existing Sprint 8A evidence manifest is malformed."
        }
        $existingManifestEntries = @($existingManifest.entries)
        $existingManifestPaths = @($existingManifestEntries | ForEach-Object { [string]$_.path })
        if (@($existingManifestPaths | Sort-Object -Unique).Count -ne $existingManifestPaths.Count) {
            throw "The existing Sprint 8A evidence manifest contains duplicate paths."
        }
        $canonicalUatPath = [IO.Path]::GetRelativePath($repository, (Join-Path $evidence "uat-result.json")).Replace("\", "/")
        $canonicalUatSidecarPath = "$canonicalUatPath.sha256"
        foreach ($existing in $existingManifestEntries) {
            $resolved = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -Path ([string]$existing.path)
            if ($excluded.Contains([IO.Path]::GetFullPath([string]$resolved.full_path))) {
                throw "The existing Sprint 8A evidence manifest inventories an excluded control file."
            }
            $declaredSha = [string]$existing.sha256
            $exists = Test-Path -LiteralPath ([string]$resolved.full_path) -PathType Leaf
            $allowedMissingCommitment = -not $exists -and
                [string]$resolved.path -in @($canonicalUatPath, $canonicalUatSidecarPath) -and
                $declaredSha -match '^[0-9a-f]{64}$' -and
                [string]$existing.phase -ceq "uat" -and
                $existing.authoritative -is [bool] -and [bool]$existing.authoritative -and
                [string]$existing.status -ceq "committed"
            if ($declaredSha -notmatch '^[0-9a-f]{64}$' -or
                $existing.authoritative -isnot [bool] -or
                [string]::IsNullOrWhiteSpace([string]$existing.phase) -or
                [string]::IsNullOrWhiteSpace([string]$existing.status) -or
                (-not $exists -and -not $allowedMissingCommitment) -or
                ($exists -and -not $authorizedMutablePaths.Contains([string]$resolved.path) -and
                    (Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)) -cne $declaredSha)) {
                throw "The existing Sprint 8A evidence manifest has stale or malformed entry '$($existing.path)'."
            }
            $entries[[string]$resolved.path] = $existing
        }
    }
    foreach ($normalizedOverride in $normalizedOverrides) {
        $override = $normalizedOverride.override
        $resolved = $normalizedOverride.resolved
        $declaredOverrideSha = if ($override.PSObject.Properties.Name -contains "sha256") {
            [string]$override.sha256
        } else { "" }
        $sha = if (Test-Path -LiteralPath ([string]$resolved.full_path) -PathType Leaf) {
            $actualSha = Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)
            if (-not [string]::IsNullOrWhiteSpace($declaredOverrideSha) -and $declaredOverrideSha -cne $actualSha) {
                throw "Sprint 8A evidence manifest override for '$($resolved.path)' differs from retained bytes."
            }
            $actualSha
        } else {
            $declaredOverrideSha
        }
        $entries[[string]$resolved.path] = [pscustomobject][ordered]@{
            path = [string]$resolved.path
            sha256 = $sha
            phase = [string]$override.phase
            authoritative = $override.authoritative
            status = [string]$override.status
        }
        $sidecarPath = "$([string]$resolved.path).sha256"
        if (-not $overridePaths.Contains($sidecarPath) -and $entries.Contains($sidecarPath) -and
            (Test-Path -LiteralPath "$([string]$resolved.full_path).sha256" -PathType Leaf)) {
            $entries[$sidecarPath] = [pscustomobject][ordered]@{
                path = $sidecarPath
                sha256 = Get-Sprint8AFileSha256 -Path "$([string]$resolved.full_path).sha256"
                phase = "retained-evidence"
                authoritative = $false
                status = "retained"
            }
        }
    }
    @($entries.Values | Sort-Object path)
}

function Assert-Sprint8AEvidenceManifestCompleteness {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [string]$ManifestPath,
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot))
    }
    $manifestFullPath = if ([string]::IsNullOrWhiteSpace($ManifestPath)) {
        Join-Path $evidence "evidence-manifest.json"
    } elseif ([IO.Path]::IsPathRooted($ManifestPath)) {
        [IO.Path]::GetFullPath($ManifestPath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $ManifestPath))
    }
    $expectedManifestPath = [IO.Path]::GetFullPath((Join-Path $evidence "evidence-manifest.json"))
    if ($manifestFullPath -cne $expectedManifestPath) {
        throw "Sprint 8A evidence-manifest completeness must authenticate the canonical manifest."
    }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidence
    Assert-Sprint8AReceiptSidecar -Path $manifestFullPath | Out-Null
    $manifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or
        [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
        throw "The Sprint 8A evidence manifest is malformed."
    }
    $manifestEntries = @($manifest.entries)
    $manifestPaths = @($manifestEntries | ForEach-Object { [string]$_.path })
    if (@($manifestPaths | Sort-Object -Unique).Count -ne $manifestPaths.Count) {
        throw "The Sprint 8A evidence manifest contains duplicate paths."
    }
    $actualEntries = @(Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $repository `
        -EvidenceRoot $evidence `
        -IgnoreExistingManifest)
    $actual = [ordered]@{}
    foreach ($entry in $actualEntries) { $actual[[string]$entry.path] = $entry }
    $canonicalUatPath = [IO.Path]::GetRelativePath(
        $repository,
        (Join-Path $evidence "uat-result.json")
    ).Replace("\", "/")
    $canonicalUatSidecarPath = "$canonicalUatPath.sha256"
    $canonicalCommitments = @($manifestEntries | Where-Object {
        [string]$_.path -in @($canonicalUatPath, $canonicalUatSidecarPath)
    })
    foreach ($entry in $manifestEntries) {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Path ([string]$entry.path)
        $declaredSha = [string]$entry.sha256
        if ($declaredSha -notmatch '^[0-9a-f]{64}$' -or
            $entry.authoritative -isnot [bool] -or
            [string]::IsNullOrWhiteSpace([string]$entry.phase) -or
            [string]::IsNullOrWhiteSpace([string]$entry.status)) {
            throw "Lifecycle evidence '$($entry.path)' has malformed digest, phase, authority, or status metadata."
        }
        if ($actual.Contains([string]$resolved.path)) {
            if ([string]$actual[[string]$resolved.path].sha256 -cne $declaredSha) {
                throw "Lifecycle evidence '$($entry.path)' differs from its manifest SHA-256."
            }
            $actual.Remove([string]$resolved.path)
            continue
        }
        $allowedMissing = $AllowMissingCanonicalUatCommitment -and
            [string]$resolved.path -in @($canonicalUatPath, $canonicalUatSidecarPath) -and
            [string]$entry.phase -ceq "uat" -and
            [bool]$entry.authoritative -and
            [string]$entry.status -ceq "committed"
        if (-not $allowedMissing) {
            throw "Manifested Sprint 8A evidence '$($entry.path)' is missing."
        }
    }
    if ($actual.Count -ne 0) {
        throw "The Sprint 8A evidence manifest omits retained file(s): $(@($actual.Keys) -join ', ')."
    }
    if ($AllowMissingCanonicalUatCommitment -and $canonicalCommitments.Count -gt 0) {
        if ($canonicalCommitments.Count -ne 2 -or
            @($canonicalCommitments | Where-Object {
                [string]$_.phase -cne "uat" -or $_.authoritative -isnot [bool] -or
                    -not [bool]$_.authoritative -or [string]$_.status -cne "committed"
            }).Count -ne 0) {
            throw "The canonical UAT result commitment must include its exact JSON and SHA-256 sidecar pair."
        }
        $jsonCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatPath)[0]
        $sidecarCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatSidecarPath)[0]
        if ([string]$sidecarCommitment.sha256 -cne
            (Get-Sprint8AStringSha256 -Text "$([string]$jsonCommitment.sha256)`n")) {
            throw "The canonical UAT result sidecar commitment does not authenticate its JSON digest."
        }
    }
    $manifest
}

function Get-Sprint8AEvidenceManifestReplacementPaths {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$ExistingEntries,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$UpdatedEntries
    )

    $existingByPath = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($ExistingEntries)) {
        $path = [string]$entry.path
        if ([string]::IsNullOrWhiteSpace($path) -or -not $existingByPath.TryAdd($path, $entry)) {
            throw "Sprint 8A evidence manifest replacement comparison requires unique nonempty existing paths."
        }
    }
    $updatedPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $replacementPaths = [Collections.Generic.List[string]]::new()
    foreach ($entry in @($UpdatedEntries)) {
        $path = [string]$entry.path
        if ([string]::IsNullOrWhiteSpace($path) -or -not $updatedPaths.Add($path)) {
            throw "Sprint 8A evidence manifest replacement comparison requires unique nonempty updated paths."
        }
        if (-not $existingByPath.ContainsKey($path)) { continue }
        $existing = $existingByPath[$path]
        $authorityChanged = $existing.authoritative -isnot [bool] -or
            $entry.authoritative -isnot [bool] -or
            [bool]$existing.authoritative -ne [bool]$entry.authoritative
        if ([string]$existing.sha256 -cne [string]$entry.sha256 -or
            [string]$existing.phase -cne [string]$entry.phase -or
            $authorityChanged -or
            [string]$existing.status -cne [string]$entry.status) {
            $replacementPaths.Add($path)
        }
    }
    @($replacementPaths)
}

function Publish-Sprint8AEvidenceManifest {
    param(
        [Parameter(Mandatory)][object[]]$Entries,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$OutputPath,
        [switch]$Merge,
        [AllowEmptyCollection()][string[]]$AuthorizedReplacementPaths = @()
    )

    $target = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $OutputPath
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $expectedManifestPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "evidence-manifest.json")
    ).Replace("\", "/")
    if ([string]$target.path -cne $expectedManifestPath) {
        throw "Sprint 8A evidence manifest must use canonical path '$expectedManifestPath'."
    }
    $existingEntries = @()
    $hasExistingManifest = Test-Path -LiteralPath ([string]$target.full_path) -PathType Leaf
    $replacementPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    if (-not $hasExistingManifest -and @($AuthorizedReplacementPaths).Count -gt 0) {
        $unusedInitialReplacementPaths = @($AuthorizedReplacementPaths | Sort-Object -CaseSensitive -Unique)
        throw "Sprint 8A evidence manifest replacement authorization was not consumed by an exact metadata or digest change: $($unusedInitialReplacementPaths -join ', ')."
    }
    if ($hasExistingManifest) {
        if (-not $Merge) {
            throw "The Sprint 8A evidence manifest already exists; use -Merge to preserve and extend it."
        }
        Assert-Sprint8AReceiptSidecar -Path ([string]$target.full_path) | Out-Null
        $existingManifest = Get-Content -LiteralPath ([string]$target.full_path) -Raw | ConvertFrom-Json
        if (($existingManifest.schema_version -isnot [int] -and $existingManifest.schema_version -isnot [long]) -or
            [int]$existingManifest.schema_version -ne 1 -or
            [string]$existingManifest.sprint -cne "sprint-8a" -or
            [string]$existingManifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
            throw "The existing Sprint 8A evidence manifest is malformed."
        }
        $existingPaths = @($existingManifest.entries | ForEach-Object { [string]$_.path })
        if (@($existingPaths | Sort-Object -Unique).Count -ne $existingPaths.Count) {
            throw "The existing Sprint 8A evidence manifest contains duplicate paths."
        }
        $replacementEntries = [Collections.Generic.List[object]]::new()
        foreach ($replacementPath in @($AuthorizedReplacementPaths)) {
            $resolvedReplacement = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path $replacementPath
            if (-not $replacementPaths.Add([string]$resolvedReplacement.path)) {
                throw "Sprint 8A evidence manifest replacement authorization contains duplicate paths."
            }
            $matches = @($Entries | Where-Object { [string]$_.path -ceq [string]$resolvedReplacement.path })
            if ($matches.Count -ne 1) {
                throw "Sprint 8A evidence manifest replacement '$($resolvedReplacement.path)' lacks one exact update entry."
            }
            $replacementEntries.Add($matches[0])
        }
        Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Overrides @($replacementEntries) | Out-Null
        $existingEntries = @($existingManifest.entries)
    }
    $newEntryPaths = @($Entries | ForEach-Object { [string]$_.path })
    if (@($newEntryPaths | Sort-Object -Unique).Count -ne $newEntryPaths.Count) {
        throw "Sprint 8A evidence manifest update contains duplicate paths."
    }
    $entryMap = [ordered]@{}
    foreach ($entry in @($existingEntries) + @($Entries)) {
        if ([string]::IsNullOrWhiteSpace([string]$entry.path)) {
            throw "Sprint 8A evidence manifest entries require canonical paths."
        }
        $entryMap[[string]$entry.path] = $entry
    }
    $canonicalUatResultPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat-result.json")
    ).Replace("\", "/")
    $canonicalUatResultSidecarPath = "$canonicalUatResultPath.sha256"
    $validated = @($entryMap.Values | ForEach-Object {
        $reference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$_.path)
        $exists = Test-Path -LiteralPath ([string]$reference.full_path) -PathType Leaf
        $declaredSha = [string]$_.sha256
        $status = [string]$_.status
        if (-not $exists -and
            ($status -cne "committed" -or
                $declaredSha -notmatch '^[0-9a-f]{64}$' -or
                [string]$reference.path -notin @($canonicalUatResultPath, $canonicalUatResultSidecarPath) -or
                [string]$_.phase -cne "uat" -or
                $_.authoritative -isnot [bool] -or -not [bool]$_.authoritative)) {
            throw "Required lifecycle evidence '$($_.path)' is missing and has no content-addressed commitment."
        }
        $sha = if ($exists) {
            Get-Sprint8AFileSha256 -Path ([string]$reference.full_path)
        } else {
            $declaredSha
        }
        if (-not [string]::IsNullOrWhiteSpace($declaredSha) -and $declaredSha -cne $sha) {
            throw "Lifecycle evidence '$($_.path)' differs from its declared SHA-256."
        }
        if ($_.authoritative -isnot [bool] -or
            [string]::IsNullOrWhiteSpace([string]$_.phase) -or
            [string]::IsNullOrWhiteSpace($status)) {
            throw "Lifecycle evidence '$($_.path)' has malformed phase, authority, or status metadata."
        }
        [pscustomobject][ordered]@{
            path = [string]$reference.path
            sha256 = $sha
            phase = [string]$_.phase
            authoritative = $_.authoritative
            status = $status
        }
    } | Sort-Object path)
    $canonicalCommitments = @($validated | Where-Object {
        [string]$_.path -in @($canonicalUatResultPath, $canonicalUatResultSidecarPath)
    })
    if ($canonicalCommitments.Count -gt 0) {
        if ($canonicalCommitments.Count -ne 2 -or
            @($canonicalCommitments | Where-Object {
                [string]$_.phase -cne "uat" -or -not [bool]$_.authoritative -or
                    [string]$_.status -cne "committed"
            }).Count -ne 0) {
            throw "The canonical UAT result commitment must include its exact JSON and SHA-256 sidecar pair."
        }
        $jsonCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatResultPath)[0]
        $sidecarCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatResultSidecarPath)[0]
        if ([string]$sidecarCommitment.sha256 -cne
            (Get-Sprint8AStringSha256 -Text "$([string]$jsonCommitment.sha256)`n")) {
            throw "The canonical UAT result sidecar commitment does not authenticate its JSON digest."
        }
    }
    if ($hasExistingManifest) {
        $normalizedExistingEntries = @($existingEntries | ForEach-Object {
            $reference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$_.path)
            [pscustomobject][ordered]@{
                path = [string]$reference.path
                sha256 = [string]$_.sha256
                phase = [string]$_.phase
                authoritative = $_.authoritative
                status = [string]$_.status
            }
        })
        $requiredReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
            -ExistingEntries $normalizedExistingEntries `
            -UpdatedEntries $validated)
        $unauthorizedReplacementPaths = @($requiredReplacementPaths | Where-Object {
            -not $replacementPaths.Contains([string]$_)
        })
        if ($unauthorizedReplacementPaths.Count -gt 0) {
            throw "Sprint 8A evidence manifest entry metadata or digest changed without exact replacement authorization: $($unauthorizedReplacementPaths -join ', ')."
        }
        $requiredReplacementPathSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($requiredReplacementPath in $requiredReplacementPaths) {
            [void]$requiredReplacementPathSet.Add([string]$requiredReplacementPath)
        }
        $unusedReplacementPaths = @($replacementPaths | Where-Object {
            -not $requiredReplacementPathSet.Contains([string]$_)
        } | Sort-Object -CaseSensitive)
        if ($unusedReplacementPaths.Count -gt 0) {
            throw "Sprint 8A evidence manifest replacement authorization was not consumed by an exact metadata or digest change: $($unusedReplacementPaths -join ', ')."
        }
    }
    $manifest = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.evidence-manifest"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        entries = $validated
    }
    Publish-Sprint7AEvidence -Document $manifest -OutputPath ([string]$target.full_path) -Overwrite:$Merge | Out-Null
    [pscustomobject][ordered]@{
        path = [string]$target.path
        sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$target.full_path)
    }
}

function Test-Sprint8AEvidenceManifestContract {
    $repository = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
    $relativeRoot = "artifacts/.sprint-8a-manifest-selftest-$([guid]::NewGuid().ToString('N'))"
    $evidence = [IO.Path]::GetFullPath((Join-Path $repository $relativeRoot))
    $artifactsRoot = [IO.Path]::GetFullPath((Join-Path $repository "artifacts"))
    if (-not $evidence.StartsWith("$($artifactsRoot.TrimEnd('\'))\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Sprint 8A evidence-manifest self-test target escaped the repository artifacts root."
    }
    [IO.Directory]::CreateDirectory($evidence) | Out-Null
    try {
        $rawPath = Join-Path $evidence "raw.txt"
        [IO.File]::WriteAllText($rawPath, "retained`n", [Text.UTF8Encoding]::new($false))
        $stablePath = Join-Path $evidence "stable.txt"
        [IO.File]::WriteAllText($stablePath, "stable`n", [Text.UTF8Encoding]::new($false))
        $journalTargetPath = Join-Path $evidence "journal-target.json"
        Publish-Sprint7AEvidence `
            -Document ([pscustomobject][ordered]@{ schema_version = 1; state = "retained" }) `
            -OutputPath $journalTargetPath | Out-Null
        $journalPath = "$journalTargetPath.publish-journal.json"
        [IO.File]::WriteAllText($journalPath, "{", [Text.UTF8Encoding]::new($false))
        $journalRecoveryEntries = @(Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence)
        if ((Test-Path -LiteralPath $journalPath) -or
            @($journalRecoveryEntries | Where-Object { [string]$_.path -like "*.publish-journal.json" }).Count -ne 0) {
            throw "Sprint 8A evidence-manifest self-test retained or inventoried a repaired publisher journal."
        }
        foreach ($transientPath in @(
            "$journalTargetPath.rollback",
            (Join-Path $evidence ".orphan.$([guid]::NewGuid().ToString('N')).tmp")
        )) {
            [IO.File]::WriteAllText($transientPath, "publisher-control", [Text.UTF8Encoding]::new($false))
            $controlRejected = $false
            try {
                Get-Sprint8AEvidenceFileManifestEntries `
                    -RepositoryRoot $repository `
                    -EvidenceRoot $evidence | Out-Null
            } catch {
                if (-not $_.Exception.Message.Contains("unresolved publisher control file")) { throw }
                $controlRejected = $true
            }
            Remove-Item -LiteralPath $transientPath -Force
            if (-not $controlRejected) {
                throw "Sprint 8A evidence-manifest self-test inventoried unresolved publisher control '$transientPath'."
            }
        }
        $manifestPath = Join-Path $evidence "evidence-manifest.json"
        $manifestOutput = "$relativeRoot/evidence-manifest.json"
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput | Out-Null
        [IO.File]::WriteAllText("$manifestPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath | Out-Null
        if (Test-Path -LiteralPath "$manifestPath.publish-journal.json") {
            throw "Sprint 8A evidence-manifest completeness did not repair its target-bound publisher journal."
        }
        [IO.File]::WriteAllText($rawPath, "tampered`n", [Text.UTF8Encoding]::new($false))
        $tamperRejected = $false
        try {
            Assert-Sprint8AEvidenceManifestCompleteness `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -ManifestPath $manifestPath | Out-Null
        } catch { $tamperRejected = $true }
        if (-not $tamperRejected) {
            throw "Sprint 8A evidence-manifest self-test accepted changed retained evidence."
        }
        [IO.File]::WriteAllText($rawPath, "retained`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($rawPath, "intentional-update`n", [Text.UTF8Encoding]::new($false))
        $rawRelative = "$relativeRoot/raw.txt"
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @([pscustomobject]@{
                path = $rawRelative; sha256 = Get-Sprint8AFileSha256 -Path $rawPath
                phase = "self-test-mutable"; authoritative = $false; status = "updated"
            })
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput `
            -Merge `
            -AuthorizedReplacementPaths @($rawRelative) | Out-Null
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath | Out-Null
        $rawSha = Get-Sprint8AFileSha256 -Path $rawPath
        foreach ($metadataCase in @(
            [pscustomobject]@{
                label = "phase"; phase = "self-test-metadata"; authoritative = $false; status = "updated"
            },
            [pscustomobject]@{
                label = "authority"; phase = "self-test-mutable"; authoritative = $true; status = "updated"
            },
            [pscustomobject]@{
                label = "status"; phase = "self-test-mutable"; authoritative = $false; status = "metadata-updated"
            }
        )) {
            $metadataEntries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -Overrides @([pscustomobject]@{
                    path = $rawRelative; sha256 = $rawSha
                    phase = [string]$metadataCase.phase
                    authoritative = [bool]$metadataCase.authoritative
                    status = [string]$metadataCase.status
                })
            $currentManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
            $requiredReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
                -ExistingEntries @($currentManifest.entries) `
                -UpdatedEntries $metadataEntries)
            if ($requiredReplacementPaths.Count -ne 1 -or
                [string]$requiredReplacementPaths[0] -cne $rawRelative) {
                throw "Sprint 8A evidence-manifest self-test did not classify the $($metadataCase.label)-only update as one replacement."
            }
            $metadataRejected = $false
            try {
                Publish-Sprint8AEvidenceManifest `
                    -Entries $metadataEntries `
                    -RepositoryRoot $repository `
                    -EvidenceRoot $evidence `
                    -OutputPath $manifestOutput `
                    -Merge | Out-Null
            } catch {
                if (-not $_.Exception.Message.Contains("without exact replacement authorization")) { throw }
                $metadataRejected = $true
            }
            if (-not $metadataRejected) {
                throw "Sprint 8A evidence-manifest self-test accepted an unauthorized $($metadataCase.label)-only replacement."
            }
        }
        $authorizedMetadataEntries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @([pscustomobject]@{
                path = $rawRelative; sha256 = $rawSha
                phase = "self-test-metadata"; authoritative = $true; status = "metadata-updated"
            })
        Publish-Sprint8AEvidenceManifest `
            -Entries $authorizedMetadataEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput `
            -Merge `
            -AuthorizedReplacementPaths @($rawRelative) | Out-Null
        $metadataManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $metadataEntry = @($metadataManifest.entries | Where-Object path -CEQ $rawRelative)
        if ($metadataEntry.Count -ne 1 -or [string]$metadataEntry[0].sha256 -cne $rawSha -or
            [string]$metadataEntry[0].phase -cne "self-test-metadata" -or
            $metadataEntry[0].authoritative -isnot [bool] -or -not [bool]$metadataEntry[0].authoritative -or
            [string]$metadataEntry[0].status -cne "metadata-updated") {
            throw "Sprint 8A evidence-manifest self-test did not retain an authorized full-metadata replacement."
        }
        $unusedAuthorizationRejected = $false
        try {
            Publish-Sprint8AEvidenceManifest `
                -Entries $authorizedMetadataEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -OutputPath $manifestOutput `
                -Merge `
                -AuthorizedReplacementPaths @($rawRelative) | Out-Null
        } catch {
            if (-not $_.Exception.Message.Contains("replacement authorization was not consumed")) { throw }
            $unusedAuthorizationRejected = $true
        }
        if (-not $unusedAuthorizationRejected) {
            throw "Sprint 8A evidence-manifest self-test accepted an unused replacement authorization."
        }
        [IO.File]::WriteAllText($stablePath, "unrelated-tamper`n", [Text.UTF8Encoding]::new($false))
        $unrelatedTamperRejected = $false
        try {
            Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -Overrides @([pscustomobject]@{
                    path = $rawRelative; sha256 = Get-Sprint8AFileSha256 -Path $rawPath
                    phase = "self-test-mutable"; authoritative = $false; status = "updated"
                }) | Out-Null
        } catch { $unrelatedTamperRejected = $true }
        if (-not $unrelatedTamperRejected) {
            throw "Sprint 8A evidence-manifest self-test allowed an unrelated tamper beside an authorized replacement."
        }
        [IO.File]::WriteAllText($stablePath, "stable`n", [Text.UTF8Encoding]::new($false))
        $canonicalDocument = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "uat"; state = "passed"
        }
        $canonicalSha = Get-Sprint8AStringSha256 -Text (($canonicalDocument | ConvertTo-Json -Depth 30) + "`n")
        $canonicalPath = "$relativeRoot/uat-result.json"
        $commitments = @(
            [pscustomobject][ordered]@{
                path = $canonicalPath; sha256 = $canonicalSha
                phase = "uat"; authoritative = $true; status = "committed"
            },
            [pscustomobject][ordered]@{
                path = "$canonicalPath.sha256"
                sha256 = Get-Sprint8AStringSha256 -Text "$canonicalSha`n"
                phase = "uat"; authoritative = $true; status = "committed"
            }
        )
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides $commitments
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput `
            -Merge | Out-Null
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath `
            -AllowMissingCanonicalUatCommitment | Out-Null
        $missingRejected = $false
        try {
            Assert-Sprint8AEvidenceManifestCompleteness `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -ManifestPath $manifestPath | Out-Null
        } catch { $missingRejected = $true }
        if (-not $missingRejected) {
            throw "Sprint 8A evidence-manifest self-test accepted unpublished canonical evidence."
        }
        Publish-Sprint7AEvidence -Document $canonicalDocument -OutputPath (Join-Path $evidence "uat-result.json") | Out-Null
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath | Out-Null
    } finally {
        if (Test-Path -LiteralPath $evidence -PathType Container) {
            Remove-Item -LiteralPath $evidence -Recurse -Force
        }
    }
}

function Write-Sprint8AManualUatSelfTestEvidenceFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Kind,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)][string]$RequirementId,
        [Parameter(Mandatory)][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$CapturedAt
    )

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    switch -CaseSensitive ($Kind) {
        "screenshot" {
            $png = [Convert]::FromBase64String(
                "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="
            )
            [IO.File]::WriteAllBytes($Path, $png)
        }
        "browser-trace" {
            $archive = [IO.Compression.ZipFile]::Open($Path, [IO.Compression.ZipArchiveMode]::Create)
            try {
                foreach ($name in @("trace.trace", "trace.network")) {
                    $entry = $archive.CreateEntry($name)
                    $writer = [IO.StreamWriter]::new($entry.Open(), [Text.UTF8Encoding]::new($false))
                    try { $writer.Write("self-test $name") } finally { $writer.Dispose() }
                }
            } finally { $archive.Dispose() }
        }
        { $_ -in @("operator-record", "browser-console", "http-transcript") } {
            $phase = switch ($Kind) {
                "operator-record" { "uat-manual-operator-record" }
                "browser-console" { "uat-manual-browser-console" }
                "http-transcript" { "uat-manual-http-transcript" }
            }
            $payloadField = switch ($Kind) {
                "operator-record" { "observations" }
                "browser-console" { "entries" }
                "http-transcript" { "exchanges" }
            }
            [object[]]$payload = @()
            if ($Kind -cne "browser-console") {
                $payload = @([pscustomobject][ordered]@{ state = "observed" })
            }
            $document = [ordered]@{
                schema_version = 1
                sprint = "sprint-8a"
                phase = $phase
                scenario = $Scenario
                attempt = $Attempt
                evidence_id = $RequirementId
                candidate_fingerprint = $CandidateFingerprint
                environment_fingerprint = $EnvironmentFingerprint
                captured_at = $CapturedAt.ToString("o")
            }
            $document[$payloadField] = $payload
            [IO.File]::WriteAllText(
                $Path,
                ((ConvertTo-Json -InputObject $document -Depth 10) + "`n"),
                [Text.UTF8Encoding]::new($false)
            )
        }
        default { throw "Unsupported manual UAT self-test evidence kind '$Kind'." }
    }
}

function Write-Sprint8AManualUatSelfTestCleanupFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][int]$Attempt,
        [Parameter(Mandatory)][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$RestoredAt
    )

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    $document = [ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "uat-manual-canonical-restoration"
        scenario = $Scenario
        attempt = $Attempt
        result = "canonical_topology_verified"
        candidate_fingerprint = $CandidateFingerprint
        environment_fingerprint = $EnvironmentFingerprint
        restored_at = $RestoredAt.ToString("o")
        observations = @([pscustomobject][ordered]@{ state = "restored" })
    }
    [IO.File]::WriteAllText(
        $Path,
        ((ConvertTo-Json -InputObject $document -Depth 10) + "`n"),
        [Text.UTF8Encoding]::new($false)
    )
}

function Test-Sprint8AManualUatEvidenceContentContracts {
    $root = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-content-selftest-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($root) | Out-Null
    try {
        $contract = Get-Sprint8AManualUatScenarioContract -Scenario "UAT-8A-01"
        $contentContracts = $contract.receipt_contract.evidence_kind_content_contracts
        $started = [DateTimeOffset]::UtcNow.AddSeconds(-1)
        $common = @{
            Scenario = "UAT-8A-01"; Attempt = 1; CandidateFingerprint = "e" * 64
            EnvironmentFingerprint = "f" * 64; StartedAt = $started
        }
        $capturedAt = [DateTimeOffset]::UtcNow
        foreach ($validKind in @("operator-record", "browser-console", "http-transcript")) {
            $validPath = Join-Path $root "valid-$validKind.json"
            Write-Sprint8AManualUatSelfTestEvidenceFile `
                -Path $validPath -Kind $validKind -Scenario "UAT-8A-01" -Attempt 1 `
                -RequirementId "valid-$validKind" -CandidateFingerprint ("e" * 64) `
                -EnvironmentFingerprint ("f" * 64) -CapturedAt $capturedAt
            Assert-Sprint8AManualUatEvidenceKindContent @common `
                -Kind $validKind -Path $validPath -RequirementId "valid-$validKind" `
                -ContentContract $contentContracts.PSObject.Properties[$validKind].Value
        }
        $lookalikes = @(
            [pscustomobject]@{
                kind = "screenshot"; id = "light-1280-screenshot"; name = "lookalike.png"
                content = "not a png"
            },
            [pscustomobject]@{
                kind = "browser-trace"; id = "component-lifecycle-trace"; name = "lookalike.zip"
                content = "not a zip"
            },
            [pscustomobject]@{
                kind = "operator-record"; id = "seeded-kind-route-record"; name = "lookalike.json"
                content = '{"schema_version":1}'
            }
        )
        foreach ($lookalike in $lookalikes) {
            $path = Join-Path $root $lookalike.name
            [IO.File]::WriteAllText($path, $lookalike.content, [Text.UTF8Encoding]::new($false))
            $rejected = $false
            try {
                Assert-Sprint8AManualUatEvidenceKindContent @common `
                    -Kind $lookalike.kind -Path $path -RequirementId $lookalike.id `
                    -ContentContract $contentContracts.PSObject.Properties[[string]$lookalike.kind].Value
            } catch { $rejected = $true }
            if (-not $rejected) {
                throw "Sprint 8A manual UAT content self-test accepted relabeled '$($lookalike.kind)' bytes."
            }
        }
    } finally {
        if (Test-Path -LiteralPath $root -PathType Container) { Remove-Item -LiteralPath $root -Recurse -Force }
    }
}

function Test-Sprint8AManualUatAttemptAuthority {
    $repository = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
    $relativeRoot = "artifacts/.sprint-8a-manual-authority-selftest-$([guid]::NewGuid().ToString('N'))"
    $evidence = [IO.Path]::GetFullPath((Join-Path $repository $relativeRoot))
    $artifactsRoot = [IO.Path]::GetFullPath((Join-Path $repository "artifacts"))
    if (-not $evidence.StartsWith("$($artifactsRoot.TrimEnd('\'))\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Sprint 8A manual-authority self-test target escaped the repository artifacts root."
    }
    $lease = $null
    $structuredLease = $null
    [IO.Directory]::CreateDirectory((Join-Path $evidence "attempts")) | Out-Null
    try {
        $receipt = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "uat"; attempt = 1
            authoritative = $false; state = "executing"; stage = "manual"
            source_verification_state = "verified"
            candidate_fingerprint = "e" * 64; environment_fingerprint = "f" * 64
            prerequisite_receipts = @(
                [pscustomobject]@{ path = "preflight"; sha256 = "1" * 64 },
                [pscustomobject]@{ path = "candidate"; sha256 = "2" * 64 },
                [pscustomobject]@{ path = "sit"; sha256 = "3" * 64 }
            )
            manual_scenarios_pending = @(Get-Sprint8AManualUatScenarioNames)
            scripted_completed_at = "2026-01-01T00:00:00Z"
        }
        $attemptPath = Join-Path $evidence "attempts/uat-1.json"
        $checkpointPath = Join-Path $evidence "attempts/uat-1-manual-checkpoint.json"
        Publish-Sprint7AEvidence -Document $receipt -OutputPath $attemptPath | Out-Null
        Publish-Sprint7AEvidence -Document $receipt -OutputPath $checkpointPath | Out-Null
        $attemptRelative = "$relativeRoot/attempts/uat-1.json"
        $checkpointRelative = "$relativeRoot/attempts/uat-1-manual-checkpoint.json"
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @(
                [pscustomobject]@{ path = $attemptRelative; phase = "uat-attempt"; authoritative = $false; status = "awaiting-manual" },
                [pscustomobject]@{ path = $checkpointRelative; phase = "uat-checkpoint"; authoritative = $false; status = "awaiting-manual" }
            )
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath "$relativeRoot/evidence-manifest.json" | Out-Null
        $checkpoint = [pscustomobject][ordered]@{
            reference = [pscustomobject][ordered]@{
                path = $checkpointRelative
                sha256 = Assert-Sprint8AReceiptSidecar -Path $checkpointPath
            }
            receipt = $receipt
        }
        Assert-Sprint8AManualUatAttemptOpen `
            -Attempt 1 `
            -Checkpoint $checkpoint `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence | Out-Null
        $structuredCases = @(
            [pscustomobject][ordered]@{
                scenario = "UAT-8A-05"; requirement_id = "dependency-semantic-receipt"
                started_at = [DateTimeOffset]::Parse("2026-01-01T00:00:05Z")
                authoritative = $true; diagnostic = $false
            },
            [pscustomobject][ordered]@{
                scenario = "UAT-8A-07"; requirement_id = "failure-containment-receipt"
                started_at = [DateTimeOffset]::Parse("2026-01-01T00:00:07Z")
                authoritative = $true; diagnostic = $false
            },
            [pscustomobject][ordered]@{
                scenario = "UAT-8A-08"; requirement_id = "upgrade-rollback-receipt"
                started_at = [DateTimeOffset]::Parse("2026-01-01T00:00:08Z")
                authoritative = $false; diagnostic = $true
            }
        )
        foreach ($structuredCase in $structuredCases) {
            $structuredLease = Open-Sprint8AManualUatScenarioLease `
                -Scenario ([string]$structuredCase.scenario) `
                -Attempt 1 `
                -CandidateFingerprint ("e" * 64) `
                -EnvironmentFingerprint ("f" * 64) `
                -StartedAt $structuredCase.started_at `
                -Authoritative ([bool]$structuredCase.authoritative) `
                -Diagnostic ([bool]$structuredCase.diagnostic) `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence
            $structuredContract = Get-Sprint8AManualUatScenarioContract -Scenario $structuredCase.scenario
            $structuredRequirement = @($structuredContract.steps | ForEach-Object {
                @($_.evidence_requirements | Where-Object { [string]$_.id -ceq [string]$structuredCase.requirement_id })
            })[0]
            $structuredPlan = Get-Sprint8AManualUatEvidencePlan `
                -Scenario $structuredCase.scenario -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
            $assertionEvidence = @($structuredRequirement.authenticated_contract.assertion_ids | ForEach-Object {
                $assertionId = [string]$_
                $rawRelative = "$([string]$structuredPlan.raw_prefix)$([string]$structuredCase.requirement_id)-$assertionId-raw.json"
                $rawPath = Join-Path $repository $rawRelative
                [IO.Directory]::CreateDirectory((Split-Path -Parent $rawPath)) | Out-Null
                [IO.File]::WriteAllText(
                    $rawPath,
                    "{`"assertion_id`":`"$assertionId`",`"state`":`"observed`"}`n",
                    [Text.UTF8Encoding]::new($false)
                )
                [pscustomobject][ordered]@{
                    assertion_id = $assertionId
                    raw_evidence = @([pscustomobject][ordered]@{
                        path = $rawRelative
                        sha256 = Get-Sprint8AFileSha256 -Path $rawPath
                    })
                }
            })
            $structuredArguments = @{
                Scenario = [string]$structuredCase.scenario
                Attempt = 1
                CandidateFingerprint = "e" * 64
                EnvironmentFingerprint = "f" * 64
                StartedAt = $structuredCase.started_at
                RequirementId = [string]$structuredCase.requirement_id
                AssertionEvidence = $assertionEvidence
                ExecutionLease = $structuredLease
                RepositoryRoot = $repository
                EvidenceRoot = $evidence
            }
            $structuredReference = Publish-Sprint8AManualUatStructuredEvidence @structuredArguments
            $structuredReplay = Publish-Sprint8AManualUatStructuredEvidence @structuredArguments
            if ([string]$structuredReference.sha256 -cne [string]$structuredReplay.sha256) {
                throw "Sprint 8A structured-evidence publisher is not idempotent for '$([string]$structuredCase.scenario)'."
            }
            $structuredDocument = Get-Content -LiteralPath (Join-Path $repository ([string]$structuredReference.path)) -Raw | ConvertFrom-Json
            if ([bool]$structuredDocument.authoritative -ne [bool]$structuredCase.authoritative -or
                [bool]$structuredDocument.diagnostic -ne [bool]$structuredCase.diagnostic) {
                throw "Sprint 8A structured-evidence publisher lost the execution authority binding."
            }
            $missingAssertionRejected = $false
            try {
                $structuredArguments.AssertionEvidence = @($assertionEvidence | Select-Object -SkipLast 1)
                Publish-Sprint8AManualUatStructuredEvidence @structuredArguments | Out-Null
            } catch { $missingAssertionRejected = $true }
            if (-not $missingAssertionRejected) {
                throw "Sprint 8A structured-evidence publisher accepted an incomplete assertion inventory."
            }
            if ([string]$structuredCase.scenario -ceq "UAT-8A-07") {
                foreach ($requiredRawIdentity in @("failed-apply-response-retained", "service-logs-retained")) {
                    $missingRawRejected = $false
                    try {
                        $structuredArguments.AssertionEvidence = @(
                            $assertionEvidence | Where-Object { [string]$_.assertion_id -cne $requiredRawIdentity }
                        )
                        Publish-Sprint8AManualUatStructuredEvidence @structuredArguments | Out-Null
                    } catch { $missingRawRejected = $true }
                    if (-not $missingRawRejected) {
                        throw "Sprint 8A UAT-8A-07 structured evidence accepted missing '$requiredRawIdentity' raw evidence."
                    }
                }
            }
            $structuredArguments.AssertionEvidence = $assertionEvidence
            $structuredLeasePath = Join-Path $repository ([string]$structuredLease.lease.path)
            $structuredLease.stream.Dispose()
            $structuredLease = $null
            foreach ($path in @($structuredLeasePath, "$structuredLeasePath.sha256")) {
                if (Test-Path -LiteralPath $path -PathType Leaf) { Remove-Item -LiteralPath $path -Force }
            }
            $structuredRawRoot = Join-Path $evidence "uat/attempt-1/raw/$([string]$structuredCase.scenario.ToLowerInvariant())"
            if (Test-Path -LiteralPath $structuredRawRoot -PathType Container) {
                Remove-Item -LiteralPath $structuredRawRoot -Recurse -Force
            }
            $manifestPath = Join-Path $evidence "evidence-manifest.json"
            foreach ($path in @($manifestPath, "$manifestPath.sha256")) {
                if (Test-Path -LiteralPath $path -PathType Leaf) { Remove-Item -LiteralPath $path -Force }
            }
            $entries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -IgnoreExistingManifest `
                -Overrides @(
                    [pscustomobject]@{ path = $attemptRelative; phase = "uat-attempt"; authoritative = $false; status = "awaiting-manual" },
                    [pscustomobject]@{ path = $checkpointRelative; phase = "uat-checkpoint"; authoritative = $false; status = "awaiting-manual" }
                )
            Publish-Sprint8AEvidenceManifest `
                -Entries $entries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -OutputPath "$relativeRoot/evidence-manifest.json" | Out-Null
        }
        $blockedScenario = "UAT-8A-02"
        $blockedStarted = [DateTimeOffset]::Parse("2026-01-01T00:00:00.100Z")
        $blockedContract = Get-Sprint8AManualUatScenarioContract -Scenario $blockedScenario
        $blockedPlan = Get-Sprint8AManualUatEvidencePlan `
            -Scenario $blockedScenario -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
        $blockedCleanupPath = Join-Path $repository ([string]$blockedPlan.cleanup.path)
        Write-Sprint8AManualUatSelfTestCleanupFile `
            -Path $blockedCleanupPath -Scenario $blockedScenario -Attempt 1 `
            -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) `
            -RestoredAt $blockedStarted.AddMilliseconds(1)
        $structuredLease = Open-Sprint8AManualUatScenarioLease `
            -Scenario $blockedScenario `
            -Attempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -StartedAt $blockedStarted `
            -Authoritative $false `
            -Diagnostic $true `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence
        $blockedReference = Publish-Sprint8AManualUatReceipt `
            -Scenario $blockedScenario `
            -Attempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -State blocked `
            -Authoritative $false `
            -Diagnostic $true `
            -AssertionsStarted $false `
            -BlockedReason "self-test independent prerequisite failed" `
            -Role ([string]$blockedContract.role) `
            -TesterIdentity ([pscustomobject][ordered]@{
                tester_id = "self-test-operator"; display_name = "Self-test Operator"
                actor_bindings = @($blockedContract.actor_bindings | ForEach-Object {
                    [pscustomobject][ordered]@{ id = [string]$_.id; actor_id = "self-test-$([string]$_.id)" }
                })
            }) `
            -Preconditions @($blockedContract.required_precondition_ids | ForEach-Object {
                $id = [string]$_
                [pscustomobject][ordered]@{
                    id = $id; state = "satisfied"
                    value = switch ($id) {
                        "candidate-fingerprint" { "e" * 64 }
                        "environment-fingerprint" { "f" * 64 }
                        "evidence-root" { $relativeRoot }
                        "execution-start" { $blockedStarted.ToString("o") }
                        default { $null }
                    }
                    reference = switch ($id) {
                        "preflight-receipt" { $receipt.prerequisite_receipts[0] }
                        "sit-result-receipt" { $receipt.prerequisite_receipts[2] }
                        default { $null }
                    }
                }
            }) `
            -StartingState @($blockedContract.required_starting_state | ForEach-Object {
                [pscustomobject][ordered]@{ id = [string]$_.id; observed_value = "self-test observed" }
            }) `
            -Actions @($blockedContract.steps | ForEach-Object {
                [pscustomobject][ordered]@{
                    step = [int]$_.step; action = [string]$_.action
                    expected_result = [string]$_.expected_result
                    actual_result = "blocked before dependent product action"; state = "blocked"
                }
            }) `
            -ExpectedResult "Every canonical step expectation in $blockedScenario is satisfied." `
            -ActualResult "Scenario retained as a true dependent block without fabricated observations." `
            -Evidence @() `
            -CleanupEvidence @([pscustomobject][ordered]@{
                kind = "canonical-restoration"; path = [string]$blockedPlan.cleanup.path
                sha256 = Get-Sprint8AFileSha256 -Path $blockedCleanupPath
            }) `
            -StartedAt $blockedStarted `
            -EndedAt $blockedStarted.AddMilliseconds(2) `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath "$relativeRoot/uat/attempt-1/manual/$($blockedScenario.ToLowerInvariant()).json" `
            -ExecutionLease $structuredLease
        $structuredLease = $null
        $blockedDocument = Get-Content -LiteralPath (Join-Path $repository ([string]$blockedReference.path)) -Raw | ConvertFrom-Json
        if ([string]$blockedDocument.state -cne "blocked" -or
            $blockedDocument.evidence -isnot [array] -or @($blockedDocument.evidence).Count -ne 0) {
            throw "Sprint 8A manual-authority self-test did not retain a typed zero-evidence dependent block."
        }
        $scenario = "UAT-8A-01"
        $scenarioStarted = [DateTimeOffset]::Parse("2026-01-01T00:00:01Z")
        $lease = Open-Sprint8AManualUatScenarioLease `
            -Scenario $scenario `
            -Attempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -StartedAt $scenarioStarted `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence
        $concurrentRejected = $false
        try {
            $concurrent = Open-Sprint8AValidationAttemptLock -Path (Join-Path $evidence "validation-attempt.lock")
            $concurrent.Dispose()
        } catch { $concurrentRejected = $true }
        if (-not $concurrentRejected) {
            $lease.stream.Dispose()
            throw "Sprint 8A manual-lease self-test allowed concurrent finalization."
        }
        $openLeaseRejected = $false
        try {
            Assert-Sprint8ANoOpenManualUatLeases -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
        } catch { $openLeaseRejected = $true }
        if (-not $openLeaseRejected) {
            $lease.stream.Dispose()
            throw "Sprint 8A manual-lease self-test did not detect an open execution lease."
        }
        $lease.stream.Dispose()
        $lease = Open-Sprint8AManualUatScenarioLease `
            -Scenario $scenario `
            -Attempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -StartedAt $scenarioStarted `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Resume
        $scenarioContract = Get-Sprint8AManualUatScenarioContract -Scenario $scenario
        $scenarioPlan = Get-Sprint8AManualUatEvidencePlan `
            -Scenario $scenario -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
        $scenarioEvidence = [Collections.Generic.List[object]]::new()
        foreach ($plannedEvidence in @($scenarioPlan.evidence)) {
                $rawPath = Join-Path $repository ([string]$plannedEvidence.path)
                Write-Sprint8AManualUatSelfTestEvidenceFile `
                    -Path $rawPath `
                    -Kind ([string]$plannedEvidence.kind) `
                    -Scenario $scenario `
                    -Attempt 1 `
                    -RequirementId ([string]$plannedEvidence.requirement_id) `
                    -CandidateFingerprint ("e" * 64) `
                    -EnvironmentFingerprint ("f" * 64) `
                    -CapturedAt $scenarioStarted.AddMilliseconds(1)
                $scenarioEvidence.Add([pscustomobject][ordered]@{
                    step = [int]$plannedEvidence.step
                    requirement_id = [string]$plannedEvidence.requirement_id
                    kind = [string]$plannedEvidence.kind
                    capture = $plannedEvidence.capture
                    path = [string]$plannedEvidence.path
                    sha256 = Get-Sprint8AFileSha256 -Path $rawPath
                })
        }
        $cleanupPath = Join-Path $repository ([string]$scenarioPlan.cleanup.path)
        Write-Sprint8AManualUatSelfTestCleanupFile `
            -Path $cleanupPath -Scenario $scenario -Attempt 1 `
            -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) `
            -RestoredAt $scenarioStarted.AddMilliseconds(2)
        $cleanupEvidence = @([pscustomobject][ordered]@{
            kind = "canonical-restoration"
            path = [string]$scenarioPlan.cleanup.path
            sha256 = Get-Sprint8AFileSha256 -Path $cleanupPath
        })
        $passingActions = @($scenarioContract.steps | ForEach-Object {
            [pscustomobject][ordered]@{
                step = [int]$_.step; action = [string]$_.action
                expected_result = [string]$_.expected_result
                actual_result = "observed step $([int]$_.step)"; state = "passed"
            }
        })
        $publicationArguments = @{
            Scenario = $scenario
            Attempt = 1
            CandidateFingerprint = "e" * 64
            EnvironmentFingerprint = "f" * 64
            State = "passed"
            Authoritative = $true
            Diagnostic = $false
            AssertionsStarted = $true
            Role = [string]$scenarioContract.role
            TesterIdentity = [pscustomobject][ordered]@{
                tester_id = "self-test-operator"
                display_name = "Self-test Operator"
                actor_bindings = @($scenarioContract.actor_bindings | ForEach-Object {
                    [pscustomobject][ordered]@{ id = [string]$_.id; actor_id = "self-test-$([string]$_.id)" }
                })
            }
            Preconditions = @($scenarioContract.required_precondition_ids | ForEach-Object {
                $id = [string]$_
                [pscustomobject][ordered]@{
                    id = $id
                    state = "satisfied"
                    value = switch ($id) {
                        "candidate-fingerprint" { "e" * 64 }
                        "environment-fingerprint" { "f" * 64 }
                        "evidence-root" { $relativeRoot }
                        "execution-start" { $scenarioStarted.ToString("o") }
                        default { $null }
                    }
                    reference = switch ($id) {
                        "preflight-receipt" { $receipt.prerequisite_receipts[0] }
                        "sit-result-receipt" { $receipt.prerequisite_receipts[2] }
                        default { $null }
                    }
                }
            })
            StartingState = @($scenarioContract.required_starting_state | ForEach-Object {
                [pscustomobject][ordered]@{ id = [string]$_.id; observed_value = "self-test observed" }
            })
            Actions = $passingActions
            ExpectedResult = "Every canonical step expectation in $scenario is satisfied."
            ActualResult = "All canonical observations passed."
            Evidence = @($scenarioEvidence)
            CleanupEvidence = $cleanupEvidence
            StartedAt = $scenarioStarted
            EndedAt = $scenarioStarted.AddSeconds(1)
            RepositoryRoot = $repository
            EvidenceRoot = $evidence
            OutputPath = "$relativeRoot/uat/attempt-1/manual/$($scenario.ToLowerInvariant()).json"
            ExecutionLease = $lease
        }
        $resumedPassRejected = $false
        try {
            Publish-Sprint8AManualUatReceipt @publicationArguments | Out-Null
        } catch {
            if (-not $_.Exception.Message.Contains("cannot publish an authoritative pass after execution resume")) { throw }
            $resumedPassRejected = $true
        }
        if (-not $resumedPassRejected) {
            throw "Sprint 8A manual-lease self-test accepted an authoritative pass after resume."
        }
        $downgradedLease = $lease | Select-Object *
        $downgradedLease.resumed = $false
        $downgradedLease.resume = $null
        $publicationArguments.ExecutionLease = $downgradedLease
        $resumeDowngradeRejected = $false
        try {
            Publish-Sprint8AManualUatReceipt @publicationArguments | Out-Null
        } catch {
            if (-not $_.Exception.Message.Contains("non-resumed execution has inconsistent process lineage")) { throw }
            $resumeDowngradeRejected = $true
        }
        $publicationArguments.ExecutionLease = $lease
        if (-not $resumeDowngradeRejected) {
            throw "Sprint 8A manual-lease self-test allowed a retained resume marker to be downgraded."
        }
        $preparedPath = Join-Path $evidence "uat/attempt-1/manual-leases/$($scenario.ToLowerInvariant())-publication-prepared.json"
        if (Test-Path -LiteralPath $preparedPath) {
            throw "Sprint 8A manual-lease self-test prepared publication for a forbidden resumed pass."
        }
        $failedActions = @($scenarioContract.steps | ForEach-Object {
                [pscustomobject][ordered]@{
                    step = [int]$_.step; action = [string]$_.action
                    expected_result = [string]$_.expected_result
                    actual_result = "observed step $([int]$_.step)"
                    state = if ([int]$_.step -eq 1) { "failed" } else { "passed" }
                }
            })
        $manifestPath = Join-Path $evidence "evidence-manifest.json"
        $manifestBeforePublication = Get-Content -LiteralPath $manifestPath -Raw
        $manifestSidecarBeforePublication = Get-Content -LiteralPath "$manifestPath.sha256" -Raw
        $publicationArguments.State = "failed"
        $publicationArguments.Actions = $failedActions
        $publicationArguments.ActualResult = "The resumed scenario retained an observed product failure."
        $publicationArguments.Classification = "product"
        $publicationArguments.FailureMessage = "Resumed scenario cannot establish a fresh authoritative pass."
        $manualReference = Publish-Sprint8AManualUatReceipt @publicationArguments
        if (-not (Test-Path -LiteralPath $preparedPath -PathType Leaf)) {
            throw "Sprint 8A manual-lease self-test omitted the immutable prepared publication checkpoint."
        }
        $resumePath = Join-Path $evidence "uat/attempt-1/manual-leases/$($scenario.ToLowerInvariant())-resume.json"
        $resumeDocument = Get-Content -LiteralPath $resumePath -Raw | ConvertFrom-Json
        if ([int]$resumeDocument.original_process_id -ne $PID -or
            [int]$resumeDocument.current_process_id -ne $PID -or
            [string]$resumeDocument.lease.path -cne [string]$lease.lease.path) {
            throw "Sprint 8A manual-lease self-test did not retain exact original/current process lineage."
        }
        $completionPath = Join-Path $evidence "uat/attempt-1/manual-leases/$($scenario.ToLowerInvariant())-complete.json"
        $manualReceiptPath = Join-Path $repository ([string]$manualReference.path)
        foreach ($publishedPath in @($manualReceiptPath, "$manualReceiptPath.sha256", $completionPath, "$completionPath.sha256")) {
            Remove-Item -LiteralPath $publishedPath -Force
        }
        [IO.File]::WriteAllText($manifestPath, $manifestBeforePublication, [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText("$manifestPath.sha256", $manifestSidecarBeforePublication, [Text.UTF8Encoding]::new($false))
        $repairedPublications = @(Repair-Sprint8AManualUatPreparedPublications `
            -Attempt 1 `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence)
        $repairedPublication = @($repairedPublications | Where-Object scenario -CEQ $scenario)
        if ($repairedPublication.Count -ne 1) {
            throw "Sprint 8A manual-lease self-test did not idempotently repair one prepared human result."
        }
        $manualReference = $repairedPublication[0].receipt
        Assert-Sprint8ANoOpenManualUatLeases -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
        $manualReceipt = Get-Content -LiteralPath $manualReceiptPath -Raw | ConvertFrom-Json
        Assert-Sprint8AManualUatExecutionLeasePair `
            -Receipt $manualReceipt `
            -ReceiptReference $manualReference `
            -ExpectedScenario $scenario `
            -ExpectedAttempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence | Out-Null
        $blockedReceipt = $manualReceipt | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        $blockedReceipt.state = "blocked"
        $blockedReceipt.assertions_started = $false
        $blockedReceipt.assertions_started_at = $null
        $blockedReceipt.classification = "product-decision"
        $blockedReceipt.classification_source = "manual_operator"
        $blockedReceipt.failure_message = $null
        $blockedReceipt.blocked_reason = "A product decision is required after the resumed execution."
        foreach ($action in @($blockedReceipt.actions)) { $action.state = "blocked" }
        Assert-Sprint8AManualUatReceipt `
            -Receipt $blockedReceipt `
            -ExpectedScenario $scenario `
            -ExpectedAttempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) | Out-Null
        $terminal = $receipt | ConvertTo-Json -Depth 20 | ConvertFrom-Json
        $terminal.state = "passed"
        $terminal.stage = "result-committed"
        Publish-Sprint7AEvidence -Document $terminal -OutputPath $attemptPath -Overwrite | Out-Null
        $terminalRejected = $false
        try {
            Assert-Sprint8AManualUatAttemptOpen `
                -Attempt 1 `
                -Checkpoint $checkpoint `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence | Out-Null
        } catch { $terminalRejected = $true }
        if (-not $terminalRejected) {
            throw "Sprint 8A manual-authority self-test accepted a terminal mutable attempt."
        }
        Publish-Sprint7AEvidence -Document $receipt -OutputPath $attemptPath -Overwrite | Out-Null
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @([pscustomobject]@{
                path = $attemptRelative; phase = "uat-attempt"; authoritative = $false; status = "awaiting-manual"
            })
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath "$relativeRoot/evidence-manifest.json" `
            -Merge | Out-Null
        [IO.File]::WriteAllText((Join-Path $evidence "uat-result.json"), "publication-started", [Text.UTF8Encoding]::new($false))
        $canonicalRejected = $false
        try {
            Assert-Sprint8AManualUatAttemptOpen `
                -Attempt 1 `
                -Checkpoint $checkpoint `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence | Out-Null
        } catch { $canonicalRejected = $true }
        if (-not $canonicalRejected) {
            throw "Sprint 8A manual-authority self-test accepted publication after the canonical result boundary."
        }
    } finally {
        foreach ($heldLease in @($lease, $structuredLease)) {
            if ($null -ne $heldLease -and
                $heldLease.PSObject.Properties.Name -contains "stream" -and
                $heldLease.stream -is [IO.FileStream]) {
                $heldLease.stream.Dispose()
            }
        }
        if (Test-Path -LiteralPath $evidence -PathType Container) {
            Remove-Item -LiteralPath $evidence -Recurse -Force
        }
    }
}

function Test-Sprint8ALifecycleChain {
    $offsetlessRejected = $false
    try {
        ConvertTo-Sprint8ADateTimeOffset `
            -Value "2026-01-01T00:00:00" `
            -Label "lifecycle offsetless timestamp self-test" | Out-Null
    } catch {
        if ($_.Exception.Message -notmatch "has no UTC offset") { throw }
        $offsetlessRejected = $true
    }
    if (-not $offsetlessRejected) {
        throw "Sprint 8A lifecycle self-test accepted an offsetless timestamp."
    }
    $nonExactCardinalityRejected = $false
    try {
        Assert-Sprint8AManualUatEvidenceCardinality `
            -Requirement ([pscustomobject]@{ minimum = 1; maximum = 2 }) `
            -Label "Sprint 8A lifecycle cardinality self-test"
    } catch { $nonExactCardinalityRejected = $true }
    if (-not $nonExactCardinalityRejected) {
        throw "Sprint 8A lifecycle self-test accepted non-exact manual evidence cardinality."
    }
    Test-Sprint8ALifecycleExclusiveLock
    Test-Sprint8AEvidenceManifestContract
    Test-Sprint8AManualUatEvidenceContentContracts
    Test-Sprint8AManualUatAttemptAuthority
    $source = [pscustomobject][ordered]@{
        commit = "a" * 40
        tree = "b" * 40
        dirty = $false
        branch = "codex/sprint-8a"
        acceptance_inventory_sha256 = "c" * 64
        deployment_inputs_sha256 = "d" * 64
    }
    Assert-Sprint8ASourceIdentityObject -Source $source -RequireClean | Out-Null
    $sit = @(Get-Sprint8ASitLaneNames | ForEach-Object {
        [pscustomobject]@{
            name = $_; state = "passed"; assertions_started = $true
            classification = $null; failure_message = $null; blocked_reason = $null
            command = "self-test $_"; exit_status = 0
            started_at = "2026-01-01T00:00:00Z"; ended_at = "2026-01-01T00:00:01Z"
            duration_ms = 1000L
            evidence = @([pscustomobject]@{ path = "artifacts/sprint-8a-closeout/self-test.log"; sha256 = "a" * 64 })
        }
    })
    Assert-Sprint8AExactTerminalIdentities -Results $sit -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "SIT self-test" | Out-Null
    $sitRoundTrip = $sit | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    Assert-Sprint8AExactTerminalIdentities -Results @($sitRoundTrip) -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "SIT JSON round-trip self-test" | Out-Null
    $currentRunnerResults = $sit | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    foreach ($result in $currentRunnerResults) {
        $result.PSObject.Properties.Remove("evidence")
        $result | Add-Member -NotePropertyName evidence_path -NotePropertyValue "artifacts/sprint-8a-closeout/self-test-$([string]$result.name).log"
        $result | Add-Member -NotePropertyName evidence_sha256 -NotePropertyValue ("b" * 64)
    }
    Assert-Sprint8AExactTerminalIdentities -Results @($currentRunnerResults) -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "current runner evidence self-test" | Out-Null
    $currentRunnerResults[0].evidence_sha256 = $null
    $missingCurrentEvidenceRejected = $false
    try {
        Assert-Sprint8AExactTerminalIdentities -Results @($currentRunnerResults) -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "missing current runner evidence self-test" | Out-Null
    } catch { $missingCurrentEvidenceRejected = $true }
    if (-not $missingCurrentEvidenceRejected) {
        throw "Sprint 8A lifecycle-chain self-test accepted a current runner result without authenticated raw evidence."
    }
    $manualContract = Get-Sprint8AManualUatScenarioContract -Scenario "UAT-8A-01"
    $manual = [pscustomobject]@{
        schema_version = 2
        sprint = "sprint-8a"; phase = "uat-manual-scenario"; attempt = 1; authoritative = $true; diagnostic = $false
        scenario = "UAT-8A-01"; state = "passed"; candidate_fingerprint = "e" * 64
        environment_fingerprint = "f" * 64; assertions_started = $true; role = $manualContract.role
        assertions_started_at = "2026-01-01T00:00:00Z"
        started_at = "2026-01-01T00:00:00Z"; ended_at = "2026-01-01T00:00:01Z"
        duration_ms = 1000L
        tester_identity = [pscustomobject][ordered]@{
            tester_id = "self-test-operator"; display_name = "Self-test Operator"
            actor_bindings = @($manualContract.actor_bindings | ForEach-Object {
                [pscustomobject][ordered]@{ id = [string]$_.id; actor_id = "self-test-$([string]$_.id)" }
            })
        }
        preconditions = @($manualContract.required_precondition_ids | ForEach-Object {
            $id = [string]$_
            [pscustomobject][ordered]@{
                id = $id
                state = "satisfied"
                value = switch ($id) {
                    "candidate-fingerprint" { "e" * 64 }
                    "environment-fingerprint" { "f" * 64 }
                    "evidence-root" { "artifacts/sprint-8a-closeout" }
                    "execution-start" { "2026-01-01T00:00:00.0000000+00:00" }
                    default { $null }
                }
                reference = switch ($id) {
                    "preflight-receipt" { [pscustomobject][ordered]@{ path = "preflight"; sha256 = "1" * 64 } }
                    "sit-result-receipt" { [pscustomobject][ordered]@{ path = "sit"; sha256 = "3" * 64 } }
                    default { $null }
                }
            }
        })
        starting_state = @($manualContract.required_starting_state | ForEach-Object {
            [pscustomobject][ordered]@{ id = [string]$_.id; observed_value = "self-test observed" }
        })
        actions = @($manualContract.steps | ForEach-Object {
            [pscustomobject]@{
                step = [int]$_.step
                action = [string]$_.action
                expected_result = [string]$_.expected_result
                actual_result = "observed step $($_.step)"
                state = "passed"
            }
        })
        expected_result = "Every canonical step expectation in UAT-8A-01 is satisfied."
        actual_result = "observed"
        evidence = @($manualContract.steps | ForEach-Object {
            $step = $_
            @($step.evidence_requirements | ForEach-Object {
                [pscustomobject][ordered]@{
                    step = [int]$step.step; requirement_id = [string]$_.id; kind = [string]$_.kind
                    capture = if ($_.PSObject.Properties.Name -contains "capture") { $_.capture } else { $null }
                    path = "evidence-$([string]$_.id)$([string]@($manualContract.receipt_contract.evidence_kind_extensions.([string]$_.kind))[0])"
                    sha256 = "a" * 64
                }
            })
        })
        classification = $null; classification_source = $null; failure_message = $null; blocked_reason = $null
        scenario_contract = [pscustomobject][ordered]@{
            manifest = $manualContract.manifest
            document = $manualContract.document
            acceptance_criteria = @($manualContract.acceptance_criteria)
            semantic_predicate_ids = @($manualContract.semantic_predicate_ids)
        }
        start_checkpoint = [pscustomobject]@{ path = "checkpoint"; sha256 = "c" * 64 }
        execution_lease = [pscustomobject]@{
            path = "artifacts/sprint-8a-closeout/uat/attempt-1/manual-leases/uat-8a-01-start.json"
            sha256 = "d" * 64
        }
        resumed = $false
        execution_resume = $null
        cleanup_restoration = [pscustomobject]@{
            required = $true; result = "canonical_topology_verified"
            evidence = @([pscustomobject]@{ kind = "canonical-restoration"; path = "cleanup.json"; sha256 = "b" * 64 })
        }
    }
    Assert-Sprint8AManualUatReceipt -Receipt $manual -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $manualRoundTrip = $manual | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $extraActionPropertyReceipt = $manual | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $extraActionPropertyReceipt.actions[0] | Add-Member -NotePropertyName defects -NotePropertyValue @()
    $extraActionPropertyRejected = $false
    try {
        Assert-Sprint8AManualUatReceipt -Receipt $extraActionPropertyReceipt -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    } catch { $extraActionPropertyRejected = $true }
    if (-not $extraActionPropertyRejected) {
        throw "Sprint 8A lifecycle self-test accepted an undeclared manual action property."
    }
    $authorityRejected = $false
    try {
        $manualRoundTrip.authoritative = "true"
        Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    } catch { $authorityRejected = $true }
    if (-not $authorityRejected) { throw "Sprint 8A lifecycle-chain self-test accepted string manual authority." }
    $manualRoundTrip.authoritative = $false
    $manualRoundTrip.diagnostic = $true
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $manualRoundTrip.state = "failed"
    $manualRoundTrip.authoritative = $true
    $manualRoundTrip.diagnostic = $false
    $manualRoundTrip.classification = "product"
    $manualRoundTrip.classification_source = "manual_operator"
    $manualRoundTrip.failure_message = "self-test product failure"
    $manualRoundTrip.actions[0].state = "failed"
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $manualRoundTrip.state = "blocked"
    $manualRoundTrip.authoritative = $false
    $manualRoundTrip.diagnostic = $true
    $manualRoundTrip.assertions_started = $false
    $manualRoundTrip.assertions_started_at = $null
    $manualRoundTrip.classification = $null
    $manualRoundTrip.classification_source = $null
    $manualRoundTrip.failure_message = $null
    $manualRoundTrip.blocked_reason = "self-test prerequisite failed"
    $manualRoundTrip.actions[0].state = "blocked"
    $manualRoundTrip.evidence = @()
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $rejected = $false
    try {
        Assert-Sprint8AExactTerminalIdentities -Results @($sit | Select-Object -First 3) -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "SIT self-test" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Sprint 8A lifecycle-chain self-test accepted an incomplete SIT inventory." }
    "Sprint 8A lifecycle-chain schema and identity self-test passed."
}

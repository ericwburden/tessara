[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8c-implementation-smoke",
    [string]$ComposeFile = "deploy/sprint-8c/compose.yaml",
    [switch]$UseExistingTopology,
    [string]$FixtureReceiptPath,
    [string]$EvidencePath = "target/sprint-8c-deployed-smoke/result.json",
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

function Get-Sprint8CSmokeFixtureIdentity {
    param([Parameter(Mandatory)]$FixtureReceipt)

    if ([string]$FixtureReceipt.sprint -cne "sprint-8c" -or
        [string]$FixtureReceipt.state -cne "passed" -or
        [string]$FixtureReceipt.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$FixtureReceipt.restoration.state -cne "passed") {
        throw "Deployed smoke fixture receipt is not a healthy Sprint 8C owner-controlled fixture."
    }
    $responseFixtureProof = Assert-Sprint8CPreparedResponseFixtures `
        -FixtureReceipt $FixtureReceipt
    $datasetProperty = $FixtureReceipt.logical_identities.datasets.PSObject.Properties['dataset.base']
    $derivedFirstProperty = $FixtureReceipt.logical_identities.datasets.PSObject.Properties['dataset.derived']
    $derivedProperty = $FixtureReceipt.logical_identities.datasets.PSObject.Properties['dataset.derived-second-hop']
    $independentProperty = $FixtureReceipt.logical_identities.datasets.PSObject.Properties['dataset.independent-binding']
    $componentProperty = $FixtureReceipt.logical_identities.components.PSObject.Properties['component.dataset-table']
    $formProperty = $FixtureReceipt.logical_identities.core.forms.PSObject.Properties['form.primary/v1']
    if ($null -eq $datasetProperty -or $null -eq $derivedFirstProperty -or
        $null -eq $derivedProperty -or $null -eq $independentProperty -or
        $null -eq $componentProperty -or $null -eq $formProperty) {
        throw "Deployed smoke fixture omits the complete Dataset closure, independent binding, table Component, or primary Form identity."
    }
    $base = $datasetProperty.Value
    $derivedFirst = $derivedFirstProperty.Value
    $derived = $derivedProperty.Value
    $independent = $independentProperty.Value
    $component = $componentProperty.Value
    if ([string]$base.dataset.reference.resource_type -cne "tessara.datasets.dataset" -or
        [string]$derivedFirst.major_line.reference.resource_type -cne "tessara.datasets.dataset_major_line" -or
        [string]$derived.major_line.reference.resource_type -cne "tessara.datasets.dataset_major_line" -or
        [string]$independent.dataset.reference.resource_type -cne "tessara.datasets.dataset" -or
        [string]$component.reference.resource_type -cne "tessara.components.component_version" -or
        [string]$base.dataset.reference.owner.kind -cne "module_instance" -or
        [string]$component.reference.owner.kind -cne "module_instance") {
        throw "Deployed smoke fixture identities are not exact typed module-owned references."
    }
    $datasetId = [string]$base.dataset.reference.resource_id
    $derivedFirstDatasetId = [string]$derivedFirst.dataset.reference.resource_id
    $derivedDatasetId = [string]$derived.dataset.reference.resource_id
    $independentDatasetId = [string]$independent.dataset.reference.resource_id
    $componentVersionId = [string]$component.reference.resource_id
    $dashboardId = [string]$FixtureReceipt.logical_identities.dashboard.id
    $formId = [string]$formProperty.Value.form_id
    foreach ($identity in @(
        $datasetId, $derivedFirstDatasetId, $derivedDatasetId, $independentDatasetId,
        $componentVersionId, $dashboardId, $formId
    )) {
        if ($identity -cnotmatch '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
            throw "Deployed smoke fixture carries a non-canonical physical identity."
        }
    }
    [pscustomobject][ordered]@{
        response_ids = $responseFixtureProof.response_ids
        draft_response_id = [string]$responseFixtureProof.response_ids.'response.draft.owner'
        submitted_response_id = [string]$responseFixtureProof.response_ids.'response.submitted.owner'
        dataset_id = $datasetId
        derived_first_dataset_id = $derivedFirstDatasetId
        derived_dataset_id = $derivedDatasetId
        derived_major_line = [string]$derived.major_line.reference.resource_id
        independent_dataset_id = $independentDatasetId
        component_version_id = $componentVersionId
        dashboard_id = $dashboardId
        form_id = $formId
        installation_id = [string]$FixtureReceipt.installation_id
        response_fixtures = $responseFixtureProof
    }
}

function Assert-Sprint8CSmokeAvailabilityTransition {
    param(
        [Parameter(Mandatory)]$InitialOperations,
        [Parameter(Mandatory)]$InitialSummary,
        [Parameter(Mandatory)]$OutageOperations,
        [Parameter(Mandatory)]$OutageSummary,
        [Parameter(Mandatory)]$RecoveredOperations,
        [Parameter(Mandatory)]$RecoveredSummary
    )

    if ([string]$InitialOperations.dataset_readiness.state -notin @("available", "empty") -or
        [string]$InitialSummary.dataset_state -notin @("available", "empty")) {
        throw "Initial reverse-consumer Dataset state is not available/empty owner truth."
    }
    if ([string]$OutageOperations.dataset_readiness.state -cne "unavailable" -or
        @($OutageOperations.dataset_readiness.datasets).Count -ne 0 -or
        $null -ne $OutageOperations.summary.dataset_attention_count -or
        [string]$OutageSummary.dataset_state -cne "unavailable" -or
        $null -ne $OutageSummary.datasets -or $null -ne $OutageSummary.dataset_revisions) {
        throw "Dataset outage was misreported as empty/zero rather than explicitly unavailable."
    }
    if (($InitialOperations | ConvertTo-Json -Depth 100 -Compress) -cne
        ($RecoveredOperations | ConvertTo-Json -Depth 100 -Compress) -or
        ($InitialSummary | ConvertTo-Json -Depth 100 -Compress) -cne
        ($RecoveredSummary | ConvertTo-Json -Depth 100 -Compress)) {
        throw "Reverse consumers did not converge exactly to their pre-outage Dataset state."
    }
    [pscustomobject][ordered]@{
        initial = [string]$InitialOperations.dataset_readiness.state
        outage = "unavailable"
        recovered = [string]$RecoveredOperations.dataset_readiness.state
        false_zero_rejected = $true
        exact_recovery = $true
    }
}

function ConvertTo-Sprint8CStableFormJson {
    param([Parameter(Mandatory)]$FormDocument)

    $copy = $FormDocument | ConvertTo-Json -Depth 100 -Compress | ConvertFrom-Json -Depth 100
    foreach ($propertyName in @('dataset_sources_state', 'dataset_sources')) {
        if ($null -eq $copy.PSObject.Properties[$propertyName]) {
            throw "Form detail does not expose the canonical Dataset source-usage projection."
        }
        [void]$copy.PSObject.Properties.Remove($propertyName)
    }
    $copy | ConvertTo-Json -Depth 100 -Compress
}

function Assert-Sprint8CFormSourceUsageTransition {
    param(
        [Parameter(Mandatory)]$InitialForm,
        [Parameter(Mandatory)]$OutageForm,
        [Parameter(Mandatory)]$RecoveredForm
    )

    if ([string]$InitialForm.dataset_sources_state -cne "available" -or
        @($InitialForm.dataset_sources).Count -lt 1 -or
        [string]$OutageForm.dataset_sources_state -cne "unavailable" -or
        @($OutageForm.dataset_sources).Count -ne 0 -or
        (ConvertTo-Sprint8CStableFormJson -FormDocument $InitialForm) -cne
            (ConvertTo-Sprint8CStableFormJson -FormDocument $OutageForm) -or
        ($InitialForm | ConvertTo-Json -Depth 100 -Compress) -cne
            ($RecoveredForm | ConvertTo-Json -Depth 100 -Compress)) {
        throw "Form Dataset Sources did not distinguish available, unavailable, and exact recovery while preserving unrelated Form content."
    }
    [pscustomobject][ordered]@{
        initial = "available"
        outage = "unavailable"
        recovered = "available"
        initial_source_count = @($InitialForm.dataset_sources).Count
        unrelated_form_content_preserved = $true
        false_zero_rejected = $true
        exact_recovery = $true
    }
}

function ConvertTo-Sprint8CStableDatasetJson {
    param([Parameter(Mandatory)]$DatasetDocument)

    $copy = $DatasetDocument | ConvertTo-Json -Depth 100 -Compress | ConvertFrom-Json -Depth 100
    if ($null -eq $copy.PSObject.Properties['freshness']) {
        throw "Dataset detail does not expose the canonical freshness contract."
    }
    [void]$copy.PSObject.Properties.Remove('freshness')
    $copy | ConvertTo-Json -Depth 100 -Compress
}

function Assert-Sprint8CLastGoodDegradation {
    param(
        [Parameter(Mandatory)]$BeforeDetail,
        [Parameter(Mandatory)]$AfterDetail,
        [Parameter(Mandatory)][string]$BeforeTableSha256,
        [Parameter(Mandatory)][string]$AfterTableSha256,
        [Parameter(Mandatory)][string]$BeforeComponentSha256,
        [Parameter(Mandatory)][string]$AfterComponentSha256,
        [Parameter(Mandatory)][string]$BeforeDashboardSha256,
        [Parameter(Mandatory)][string]$AfterDashboardSha256
    )

    if ([string]$AfterDetail.freshness.state -cne "degraded" -or
        [string]$AfterDetail.freshness.sanitized_failure_code -cne
            "dataset.dependency_unavailable" -or
        (ConvertTo-Sprint8CStableDatasetJson -DatasetDocument $BeforeDetail) -cne
            (ConvertTo-Sprint8CStableDatasetJson -DatasetDocument $AfterDetail) -or
        $BeforeTableSha256 -cne $AfterTableSha256 -or
        $BeforeComponentSha256 -cne $AfterComponentSha256 -or
        $BeforeDashboardSha256 -cne $AfterDashboardSha256) {
        throw "Response provider outage did not retain exact last-good Dataset, Component, and Dashboard data with degraded freshness."
    }
    [pscustomobject][ordered]@{
        state = "degraded"
        failure_code = "dataset.dependency_unavailable"
        definition_preserved = $true
        table_preserved = $true
        downstream_preserved = $true
    }
}

function Assert-Sprint8CRefreshClosureTransition {
    param(
        [Parameter(Mandatory)]$Refresh,
        [Parameter(Mandatory)][string[]]$BeforeClosureSha256,
        [Parameter(Mandatory)][string[]]$AfterClosureSha256,
        [Parameter(Mandatory)][string]$BeforeIndependentSha256,
        [Parameter(Mandatory)][string]$AfterIndependentSha256,
        [Parameter(Mandatory)][string]$BeforeComponentSha256,
        [Parameter(Mandatory)][string]$AfterComponentSha256,
        [Parameter(Mandatory)][string]$BeforeDashboardSha256,
        [Parameter(Mandatory)][string]$AfterDashboardSha256
    )

    if ($BeforeClosureSha256.Count -ne 3 -or $AfterClosureSha256.Count -ne 3 -or
        [string]$Refresh.freshness.state -cne "current" -or
        $BeforeIndependentSha256 -cne $AfterIndependentSha256) {
        throw "Dataset refresh did not expose one current three-Dataset closure with an unchanged independent binding."
    }
    $receiptCount = @($Refresh.materialization_receipt_ids).Count
    $closureChanged = @(
        for ($index = 0; $index -lt 3; $index++) {
            $BeforeClosureSha256[$index] -cne $AfterClosureSha256[$index]
        }
    )
    if ([bool]$Refresh.changed) {
        if ($closureChanged -contains $false -or $receiptCount -lt 1 -or
            $BeforeComponentSha256 -ceq $AfterComponentSha256 -or
            $BeforeDashboardSha256 -ceq $AfterDashboardSha256) {
            throw "Changed Dataset refresh did not advance the complete closure and downstream Component/Dashboard result."
        }
    } elseif ($closureChanged -contains $true -or $receiptCount -ne 0 -or
        $BeforeComponentSha256 -cne $AfterComponentSha256 -or
        $BeforeDashboardSha256 -cne $AfterDashboardSha256) {
        throw "No-op Dataset refresh changed closure, receipt, or downstream Component/Dashboard state."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        changed = [bool]$Refresh.changed
        closure_datasets = 3
        independent_binding_unchanged = $true
        component_observation_coherent = $true
        dashboard_observation_coherent = $true
        materialization_receipt_count = $receiptCount
    }
}

function Assert-Sprint8CDashboardDocumentHealthy {
    param(
        [Parameter(Mandatory)]$DashboardResponse,
        [Parameter(Mandatory)][string]$AvailablePlacementId,
        [Parameter(Mandatory)][string]$ExpectedPresentation
    )

    $availablePattern = '<article\b(?=[^>]*\bdata-placement-id="' +
        [regex]::Escape($AvailablePlacementId) +
        '")(?=[^>]*\bdata-placement-presentation="' +
        [regex]::Escape($ExpectedPresentation) + '")[^>]*>'
    $availableMatches = [regex]::Matches([string]$DashboardResponse.body, $availablePattern)
    $unavailableMatches = [regex]::Matches(
        [string]$DashboardResponse.body,
        'data-placement-presentation="unavailable"'
    )
    if ([string]$DashboardResponse.body -notmatch 'Dashboard' -or
        $availableMatches.Count -ne 1 -or $unavailableMatches.Count -ne 1) {
        throw "Dashboard document does not expose the expected available and redacted placement states."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        available_placement_id = $AvailablePlacementId
        expected_presentation = $ExpectedPresentation
        unavailable_placement_count = $unavailableMatches.Count
    }
}

function Invoke-Sprint8CSmokeRequest {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][Microsoft.PowerShell.Commands.WebRequestSession]$Session,
        [string]$Method = "GET",
        [hashtable]$Headers = @{},
        [AllowNull()]$Body,
        [int[]]$ExpectedStatus = @(200)
    )

    $parameters = @{
        Uri = "$BaseUrl$Path"
        Method = $Method
        WebSession = $Session
        Headers = $Headers
        UseBasicParsing = $true
        SkipHttpErrorCheck = $true
        TimeoutSec = 20
    }
    if ($null -ne $Body) {
        $parameters.ContentType = "application/json"
        $parameters.Body = if ($Body -is [string]) { $Body } else { $Body | ConvertTo-Json -Depth 100 -Compress }
    }
    $response = Invoke-WebRequest @parameters
    $status = [int]$response.StatusCode
    $content = [string]$response.Content
    if ($ExpectedStatus -notcontains $status) {
        throw "Smoke request '$Method $Path' returned $status; expected $($ExpectedStatus -join ', ')."
    }
    $document = $null
    if (-not [string]::IsNullOrWhiteSpace($content) -and
        [string]$response.Headers.'Content-Type' -match 'application/json') {
        $document = $content | ConvertFrom-Json -Depth 100
    }
    [pscustomobject][ordered]@{
        method = $Method
        path = $Path
        status = $status
        content_type = [string]$response.Headers.'Content-Type'
        body = $content
        body_sha256 = Get-Sprint7ASha256 -Text $content
        document = $document
    }
}

function Invoke-Sprint8CInternalModuleRequest {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$Service,
        [Parameter(Mandatory)][ValidateRange(1, 65535)][int]$Port,
        [Parameter(Mandatory)][string]$ModuleLabel,
        [Parameter(Mandatory)][string]$Path,
        [string]$Method = "GET",
        [AllowNull()][string]$Body,
        [switch]$ControlKey,
        [int]$ExpectedStatus = 200
    )

    $arguments = @("exec", "-T", $Service, "curl", "-sS", "-X", $Method)
    if ($ControlKey) {
        $arguments += @("-H", "x-tessara-module-control-key: development-module-control-only")
    }
    if ($PSBoundParameters.ContainsKey('Body')) {
        if ([string]::IsNullOrEmpty($Body)) {
            throw "Internal $ModuleLabel request body cannot be null or empty when supplied."
        }
        $arguments += @("-H", "content-type: application/json", "--data-binary", $Body)
    }
    $arguments += @("-w", "`n%{http_code}", "http://127.0.0.1:$Port$Path")
    $result = Invoke-Sprint8CDockerCompose -ComposePath $ComposePath -Arguments $arguments -AllowFailure
    if ($result.exit_code -ne 0 -or $result.output.Count -lt 2) {
        throw "Internal $ModuleLabel request '$Method $Path' could not execute."
    }
    $status = [int]$result.output[-1]
    $content = @($result.output[0..($result.output.Count - 2)]) -join "`n"
    if ($status -ne $ExpectedStatus) {
        throw "Internal $ModuleLabel request '$Method $Path' returned $status instead of $ExpectedStatus."
    }
    [pscustomobject][ordered]@{
        method = $Method
        path = $Path
        status = $status
        body_sha256 = Get-Sprint7ASha256 -Text $content
        document = if ([string]::IsNullOrWhiteSpace($content)) { $null } else {
            $content | ConvertFrom-Json -Depth 100
        }
    }
}

function Assert-Sprint8CResponseProductProjection {
    param(
        [Parameter(Mandatory)]$DirectoryResponse,
        [Parameter(Mandatory)][object[]]$ListDocument,
        [Parameter(Mandatory)]$DraftDetail,
        [Parameter(Mandatory)]$ResponseFixtureProof
    )

    if ([int]$DirectoryResponse.status -ne 200 -or
        [string]$DirectoryResponse.body -notmatch 'Responses') {
        throw "Response directory document did not render through the canonical module route."
    }
    $expectedProperties = @($ResponseFixtureProof.response_ids.PSObject.Properties)
    $expectedIds = @($expectedProperties.Value | ForEach-Object { [string]$_ } | Sort-Object)
    $actualIds = @($ListDocument | ForEach-Object { [string]$_.id } | Sort-Object)
    if (($actualIds -join "`n") -cne ($expectedIds -join "`n")) {
        throw "Response list is not set-equal to the exact owner-bootstrap fixture identities."
    }
    foreach ($property in $expectedProperties) {
        $logicalKey = [string]$property.Name
        $identity = [string]$property.Value
        $entry = @($ListDocument | Where-Object { [string]$_.id -ceq $identity })
        $expectedState = [string]$ResponseFixtureProof.lifecycle_states.PSObject.Properties[$logicalKey].Value
        if ($entry.Count -ne 1 -or [string]$entry[0].status -cne $expectedState) {
            throw "Response list substituted lifecycle state for '$logicalKey'."
        }
    }
    $draftId = [string]$ResponseFixtureProof.response_ids.'response.draft.owner'
    if ([string]$DraftDetail.id -cne $draftId -or
        [string]$DraftDetail.status -cne "draft" -or
        [uint64]$DraftDetail.revision -ne 1 -or
        $null -eq $DraftDetail.form -or $null -eq $DraftDetail.values -or
        $null -eq $DraftDetail.audit_events) {
        throw "Response detail did not resolve exact pinned draft owner read-back."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        response_count = $expectedIds.Count
        draft_response_id = $draftId
        directory_sha256 = [string]$DirectoryResponse.body_sha256
        list_identities = @($actualIds)
        pinned_draft_read_back = "passed"
    }
}

function Assert-Sprint8CResponseDiagnosticsEnvelope {
    param([Parameter(Mandatory)]$Diagnostics)

    $expectedFields = @(
        "contract_version", "facts", "findings", "health", "release",
        "runtime_version", "schema_version", "ui_version"
    )
    $actualFields = @($Diagnostics.PSObject.Properties.Name | Sort-Object)
    if (($actualFields -join "`n") -cne ($expectedFields -join "`n") -or
        [uint16]$Diagnostics.schema_version -ne 1 -or
        [string]$Diagnostics.release -cne "1.0.0" -or
        [string]$Diagnostics.contract_version -cne "0.4.0" -or
        [string]$Diagnostics.runtime_version -cne "0.3.0" -or
        [string]$Diagnostics.ui_version -cne "0.3.0" -or
        [string]$Diagnostics.health -cne "passing" -or
        $null -eq $Diagnostics.facts -or
        @($Diagnostics.findings).Count -ne 0) {
        throw "Response diagnostics do not match the exact shared module-runtime envelope."
    }

    [pscustomobject][ordered]@{
        state = "passed"
        release = [string]$Diagnostics.release
        contract_version = [string]$Diagnostics.contract_version
        runtime_version = [string]$Diagnostics.runtime_version
        ui_version = [string]$Diagnostics.ui_version
        health = [string]$Diagnostics.health
        finding_count = @($Diagnostics.findings).Count
    }
}

function Get-Sprint8CSmokeTopologySnapshot {
    param([Parameter(Mandatory)][string]$ComposePath)

    $services = @(
        "postgres", "core", "supervisor", "responses", "datasets", "components", "dashboards",
        "scoped-records", "response-provider-proxy", "form-provider-proxy",
        "scope-provider-proxy", "principal-provider-proxy", "gateway"
    )
    $composeStates = @(Get-Sprint8CComposeServiceState -ComposePath $ComposePath)
    @($services | ForEach-Object {
        $service = $_
        $state = @($composeStates | Where-Object { [string]$_.Service -ceq $service })
        if ($state.Count -ne 1 -or [string]::IsNullOrWhiteSpace([string]$state[0].ID)) {
            throw "Smoke topology does not expose exactly one '$service' container."
        }
        $inspectionOutput = @(& docker inspect --format '{{json .}}' -- ([string]$state[0].ID) 2>&1 |
            ForEach-Object { [string]$_ })
        $inspectionExitCode = $LASTEXITCODE
        $inspectionJson = @($inspectionOutput | Where-Object { $_.TrimStart().StartsWith('{') })
        if ($inspectionExitCode -ne 0 -or $inspectionJson.Count -ne 1) {
            throw "Could not inspect exactly one '$service' container."
        }
        try {
            $inspection = $inspectionJson[0] | ConvertFrom-Json -Depth 30
        } catch {
            throw "Could not decode the '$service' container inspection."
        }
        $healthProperty = $inspection.State.PSObject.Properties['Health']
        $health = if ($null -eq $healthProperty -or $null -eq $healthProperty.Value) { "" } else {
            [string]$healthProperty.Value.Status
        }
        if ([string]$inspection.State.Status -cne "running" -or
            (-not [string]::IsNullOrWhiteSpace($health) -and $health -cne "healthy")) {
            throw "Smoke topology service '$service' is not running/healthy."
        }
        [pscustomobject][ordered]@{
            service = $service; container_id = [string]$inspection.Id
            image_id = [string]$inspection.Image
            restart_count = [int]$inspection.RestartCount
            state = [string]$inspection.State.Status; health = $health
        }
    })
}

function Assert-Sprint8CFreshDatasetReadiness {
    param([Parameter(Mandatory)]$ReadinessResponse)

    if ([int]$ReadinessResponse.status -ne 200 -or
        [string]$ReadinessResponse.document.status -cne "ready") {
        throw "Dataset service did not expose a fresh ready response from the restarted process."
    }

    [pscustomobject][ordered]@{
        state = "passed"
        status = [int]$ReadinessResponse.status
        readiness = [string]$ReadinessResponse.document.status
        body_sha256 = [string]$ReadinessResponse.body_sha256
    }
}

function Assert-Sprint8CPairwiseDatabaseDenial {
    param(
        [Parameter(Mandatory)]$ComposeConfiguration,
        [Parameter(Mandatory)][string]$ComposePath
    )

    $owners = @("core", "responses", "datasets", "components", "dashboards", "scoped-records")
    $databaseByOwner = [ordered]@{}
    $urlByOwner = [ordered]@{}
    foreach ($owner in $owners) {
        $url = [Uri][string]$ComposeConfiguration.services.$owner.environment.DATABASE_URL
        $databaseByOwner[$owner] = $url.AbsolutePath.TrimStart('/')
        $urlByOwner[$owner] = $url
    }
    $attempts = [Collections.Generic.List[object]]::new()
    foreach ($owner in $owners) {
        foreach ($foreignOwner in $owners) {
            if ($owner -ceq $foreignOwner) { continue }
            $builder = [UriBuilder]$urlByOwner[$owner]
            $builder.Host = "127.0.0.1"
            $builder.Path = "/$($databaseByOwner[$foreignOwner])"
            $result = Invoke-Sprint8CDockerCompose -ComposePath $ComposePath -Arguments @(
                "exec", "-T", "postgres", "psql", $builder.Uri.AbsoluteUri,
                "--no-psqlrc", "-v", "ON_ERROR_STOP=1", "-Atc", "SELECT current_database()"
            ) -AllowFailure
            if ($result.exit_code -eq 0) {
                throw "Runtime principal '$owner' connected to foreign owner database '$foreignOwner'."
            }
            $attempts.Add([pscustomobject][ordered]@{
                runtime_owner = $owner
                denied_database_owner = $foreignOwner
                state = "denied"
                diagnostic_sha256 = Get-Sprint7ASha256 -Text ((@($result.output) -join "`n") + "`n")
            })
        }
    }
    if ($attempts.Count -ne 30) { throw "Pairwise database denial did not execute all 30 foreign-owner pairs." }
    @($attempts)
}

function Test-Sprint8CDeployedSmokeHarness {
    $installationId = "01980000-0000-7000-8000-00000000008c"
    $moduleOwner = "01980000-0000-7000-8000-000000000081"
    $reference = {
        param($Type, $Id)
        [pscustomobject][ordered]@{
            installation_id = $installationId
            owner = [pscustomobject][ordered]@{
                kind = "module_instance"; installation_id = $installationId
                module_instance_id = $moduleOwner
            }
            resource_type = $Type
            resource_id = $Id
        }
    }
    $responseProjection = New-Sprint8CPreparedResponseFixturesSelfTestProjection
    $fixture = [pscustomobject][ordered]@{
        sprint = "sprint-8c"; state = "passed"; proof = "owner-controlled-uat-fixture-preparation"
        installation_id = $installationId
        restoration = [pscustomobject]@{ state = "passed" }
        owner_receipt_digests = @($responseProjection.owner_receipt_digest)
        logical_identities = [pscustomobject][ordered]@{
            responses = $responseProjection.responses
            core = [pscustomobject][ordered]@{
                forms = [pscustomobject][ordered]@{
                    "form.primary/v1" = [pscustomobject]@{
                        form_id = "01980000-0004-7000-8000-000000000001"
                    }
                }
            }
            datasets = [pscustomobject][ordered]@{
                "dataset.base" = [pscustomobject][ordered]@{
                    dataset = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset" "01980000-0001-7000-8000-000000000001" }
                    major_line = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset_major_line" "01980000-0001-7000-8000-000000000001@1" }
                }
                "dataset.derived" = [pscustomobject][ordered]@{
                    dataset = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset" "01980000-0001-7000-8000-000000000002" }
                    major_line = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset_major_line" "01980000-0001-7000-8000-000000000002@1" }
                }
                "dataset.derived-second-hop" = [pscustomobject][ordered]@{
                    dataset = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset" "01980000-0001-7000-8000-000000000003" }
                    major_line = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset_major_line" "01980000-0001-7000-8000-000000000003@1" }
                }
                "dataset.independent-binding" = [pscustomobject][ordered]@{
                    dataset = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset" "01980000-0001-7000-8000-000000000004" }
                    major_line = [pscustomobject]@{ reference = & $reference "tessara.datasets.dataset_major_line" "01980000-0001-7000-8000-000000000004@1" }
                }
            }
            components = [pscustomobject][ordered]@{
                "component.dataset-table" = [pscustomobject]@{
                    reference = & $reference "tessara.components.component_version" "01980000-0002-7000-8000-000000000001"
                }
            }
            dashboard = [pscustomobject]@{
                id = "01980000-0003-7000-8000-000000000001"
                key = "dashboard.dataset-components"
            }
        }
    }
    $identity = Get-Sprint8CSmokeFixtureIdentity -FixtureReceipt $fixture
    if ($identity.dataset_id -cne "01980000-0001-7000-8000-000000000001" -or
        $identity.draft_response_id -cne "01980000-0088-7000-8000-000000000001" -or
        $identity.response_fixtures.response_count -ne 4) {
        throw "Deployed smoke fixture identity self-test failed."
    }
    $canonicalDiagnostics = [pscustomobject][ordered]@{
        schema_version = 1
        release = "1.0.0"
        contract_version = "0.4.0"
        runtime_version = "0.3.0"
        ui_version = "0.3.0"
        health = "passing"
        facts = [pscustomobject]@{}
        findings = @()
    }
    Assert-Sprint8CResponseDiagnosticsEnvelope -Diagnostics $canonicalDiagnostics | Out-Null
    $datasetShapedDiagnostics = $canonicalDiagnostics | ConvertTo-Json -Depth 10 | ConvertFrom-Json -Depth 10
    $datasetShapedDiagnostics.PSObject.Properties.Remove("contract_version")
    $datasetShapedDiagnostics | Add-Member -NotePropertyName "module" -NotePropertyValue "tessara.responses"
    try {
        Assert-Sprint8CResponseDiagnosticsEnvelope -Diagnostics $datasetShapedDiagnostics | Out-Null
        throw "Deployed smoke self-test accepted Dataset-shaped Response diagnostics."
    } catch {
        if ($_.Exception.Message -notmatch 'shared module-runtime envelope') { throw }
    }
    $responseList = @($identity.response_fixtures.response_ids.PSObject.Properties |
        ForEach-Object {
            [pscustomobject]@{
                id = [string]$_.Value
                status = [string]$identity.response_fixtures.lifecycle_states.PSObject.Properties[$_.Name].Value
            }
        })
    $draftDetail = [pscustomobject]@{
        id = $identity.draft_response_id
        status = "draft"
        revision = 1
        form = [pscustomobject]@{ name = "Primary Responses" }
        values = @()
        audit_events = @()
    }
    $responseDirectory = [pscustomobject]@{
        status = 200; body = "<main><h1>Responses</h1></main>"; body_sha256 = "response-directory"
    }
    Assert-Sprint8CResponseProductProjection -DirectoryResponse $responseDirectory `
        -ListDocument $responseList -DraftDetail $draftDetail `
        -ResponseFixtureProof $identity.response_fixtures | Out-Null
    $tamperedResponseList = @($responseList | ForEach-Object {
        $_ | ConvertTo-Json -Depth 10 | ConvertFrom-Json -Depth 10
    })
    $tamperedResponseList[0].status = "deleted"
    try {
        Assert-Sprint8CResponseProductProjection -DirectoryResponse $responseDirectory `
            -ListDocument $tamperedResponseList -DraftDetail $draftDetail `
            -ResponseFixtureProof $identity.response_fixtures | Out-Null
        throw "Deployed smoke self-test accepted substituted Response lifecycle state."
    } catch {
        if ($_.Exception.Message -notmatch 'substituted lifecycle state') { throw }
    }
    $initialOperations = [pscustomobject]@{
        summary = [pscustomobject]@{ dataset_attention_count = 1 }
        dataset_readiness = [pscustomobject]@{ state = "available"; datasets = @([pscustomobject]@{ id = 1 }) }
    }
    $initialSummary = [pscustomobject]@{ dataset_state = "available"; datasets = 4; dataset_revisions = 4 }
    $outageOperations = [pscustomobject]@{
        summary = [pscustomobject]@{ dataset_attention_count = $null }
        dataset_readiness = [pscustomobject]@{ state = "unavailable"; datasets = @() }
    }
    $outageSummary = [pscustomobject]@{ dataset_state = "unavailable"; datasets = $null; dataset_revisions = $null }
    Assert-Sprint8CSmokeAvailabilityTransition -InitialOperations $initialOperations `
        -InitialSummary $initialSummary -OutageOperations $outageOperations `
        -OutageSummary $outageSummary -RecoveredOperations $initialOperations `
        -RecoveredSummary $initialSummary | Out-Null
    $tampered = $outageSummary | ConvertTo-Json | ConvertFrom-Json
    $tampered.datasets = 0
    try {
        Assert-Sprint8CSmokeAvailabilityTransition -InitialOperations $initialOperations `
            -InitialSummary $initialSummary -OutageOperations $outageOperations `
            -OutageSummary $tampered -RecoveredOperations $initialOperations `
            -RecoveredSummary $initialSummary | Out-Null
        throw "Deployed smoke self-test accepted a false-zero outage."
    } catch {
        if ($_.Exception.Message -notmatch 'misreported as empty/zero') { throw }
    }
    $initialForm = [pscustomobject][ordered]@{
        id = $identity.form_id
        name = "Primary Responses"
        versions = @([pscustomobject]@{ id = "version-1" })
        dataset_sources_state = "available"
        dataset_sources = @([pscustomobject]@{ dataset_id = $identity.dataset_id })
    }
    $outageForm = $initialForm | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
    $outageForm.dataset_sources_state = "unavailable"
    $outageForm.dataset_sources = @()
    Assert-Sprint8CFormSourceUsageTransition -InitialForm $initialForm `
        -OutageForm $outageForm -RecoveredForm $initialForm | Out-Null
    $falseEmptyForm = $outageForm | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
    $falseEmptyForm.dataset_sources_state = "empty"
    try {
        Assert-Sprint8CFormSourceUsageTransition -InitialForm $initialForm `
            -OutageForm $falseEmptyForm -RecoveredForm $initialForm | Out-Null
        throw "Deployed smoke self-test accepted a false-empty Form source projection."
    } catch {
        if ($_.Exception.Message -notmatch 'did not distinguish available, unavailable') { throw }
    }
    $beforeDetail = [pscustomobject]@{
        id = $identity.dataset_id
        name = "Base Responses"
        freshness = [pscustomobject]@{
            state = "current"; sanitized_failure_code = $null
        }
    }
    $degradedDetail = $beforeDetail | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
    $degradedDetail.freshness.state = "degraded"
    $degradedDetail.freshness.sanitized_failure_code = "dataset.dependency_unavailable"
    Assert-Sprint8CLastGoodDegradation -BeforeDetail $beforeDetail `
        -AfterDetail $degradedDetail -BeforeTableSha256 "table-a" `
        -AfterTableSha256 "table-a" -BeforeComponentSha256 "component-a" `
        -AfterComponentSha256 "component-a" -BeforeDashboardSha256 "dashboard-a" `
        -AfterDashboardSha256 "dashboard-a" | Out-Null
    try {
        Assert-Sprint8CLastGoodDegradation -BeforeDetail $beforeDetail `
            -AfterDetail $degradedDetail -BeforeTableSha256 "table-a" `
            -AfterTableSha256 "table-a" -BeforeComponentSha256 "component-a" `
            -AfterComponentSha256 "component-b" -BeforeDashboardSha256 "dashboard-a" `
            -AfterDashboardSha256 "dashboard-a" | Out-Null
        throw "Deployed smoke self-test accepted changed downstream last-good data."
    } catch {
        if ($_.Exception.Message -notmatch 'retain exact last-good') { throw }
    }
    $changedRefresh = [pscustomobject]@{
        changed = $true
        freshness = [pscustomobject]@{ state = "current" }
        materialization_receipt_ids = @("receipt-1")
    }
    Assert-Sprint8CRefreshClosureTransition -Refresh $changedRefresh `
        -BeforeClosureSha256 @("base-a", "derived-a", "second-a") `
        -AfterClosureSha256 @("base-b", "derived-b", "second-b") `
        -BeforeIndependentSha256 "independent-a" -AfterIndependentSha256 "independent-a" `
        -BeforeComponentSha256 "component-a" -AfterComponentSha256 "component-b" `
        -BeforeDashboardSha256 "dashboard-a" -AfterDashboardSha256 "dashboard-b" | Out-Null
    $noOpRefresh = [pscustomobject]@{
        changed = $false
        freshness = [pscustomobject]@{ state = "current" }
        materialization_receipt_ids = @()
    }
    Assert-Sprint8CRefreshClosureTransition -Refresh $noOpRefresh `
        -BeforeClosureSha256 @("base-a", "derived-a", "second-a") `
        -AfterClosureSha256 @("base-a", "derived-a", "second-a") `
        -BeforeIndependentSha256 "independent-a" -AfterIndependentSha256 "independent-a" `
        -BeforeComponentSha256 "component-a" -AfterComponentSha256 "component-a" `
        -BeforeDashboardSha256 "dashboard-a" -AfterDashboardSha256 "dashboard-a" | Out-Null
    try {
        Assert-Sprint8CRefreshClosureTransition -Refresh $changedRefresh `
            -BeforeClosureSha256 @("base-a", "derived-a", "second-a") `
            -AfterClosureSha256 @("base-b", "derived-b", "second-b") `
            -BeforeIndependentSha256 "independent-a" -AfterIndependentSha256 "independent-b" `
            -BeforeComponentSha256 "component-a" -AfterComponentSha256 "component-b" `
            -BeforeDashboardSha256 "dashboard-a" -AfterDashboardSha256 "dashboard-b" | Out-Null
        throw "Deployed smoke self-test accepted an independently changed binding."
    } catch {
        if ($_.Exception.Message -notmatch 'unchanged independent binding') { throw }
    }
    try {
        Assert-Sprint8CRefreshClosureTransition -Refresh $changedRefresh `
            -BeforeClosureSha256 @("base-a", "derived-a", "second-a") `
            -AfterClosureSha256 @("base-b", "derived-b", "second-b") `
            -BeforeIndependentSha256 "independent-a" -AfterIndependentSha256 "independent-a" `
            -BeforeComponentSha256 "component-a" -AfterComponentSha256 "component-b" `
            -BeforeDashboardSha256 "dashboard-a" -AfterDashboardSha256 "dashboard-a" | Out-Null
        throw "Deployed smoke self-test accepted a stale Dashboard observation."
    } catch {
        if ($_.Exception.Message -notmatch 'complete closure and downstream') { throw }
    }
    $dashboardProjection = [pscustomobject]@{
        body = '<main>Dashboard<article data-placement-presentation="table" data-placement-id="table-placement"></article><article data-placement-id="redacted" data-placement-presentation="unavailable"></article></main>'
    }
    Assert-Sprint8CDashboardDocumentHealthy -DashboardResponse $dashboardProjection `
        -AvailablePlacementId "table-placement" -ExpectedPresentation "table" | Out-Null
    try {
        Assert-Sprint8CDashboardDocumentHealthy -DashboardResponse $dashboardProjection `
            -AvailablePlacementId "substituted-placement" -ExpectedPresentation "table" | Out-Null
        throw "Deployed smoke self-test accepted a substituted available Dashboard placement."
    } catch {
        if ($_.Exception.Message -notmatch 'expected available and redacted') { throw }
    }
    $freshReadiness = [pscustomobject]@{
        status = 200
        body_sha256 = "ready-body"
        document = [pscustomobject]@{ status = "ready" }
    }
    Assert-Sprint8CFreshDatasetReadiness -ReadinessResponse $freshReadiness | Out-Null
    $staleReadiness = $freshReadiness | ConvertTo-Json -Depth 10 | ConvertFrom-Json -Depth 10
    $staleReadiness.document.status = "not_ready"
    try {
        Assert-Sprint8CFreshDatasetReadiness -ReadinessResponse $staleReadiness | Out-Null
        throw "Deployed smoke self-test accepted a non-ready restarted Dataset process."
    } catch {
        if ($_.Exception.Message -notmatch 'fresh ready response') { throw }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "deployed-smoke-harness-self-test"
        state = "passed"
        database_free = $true
        compose_project = $null
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text "sprint-8c-deployed-smoke-self-test`n"
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"; mode = "database-free-self-test"
        }
    }
}

if ($SelfTest) {
    $result = Test-Sprint8CDeployedSmokeHarness
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8CHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result | ConvertTo-Json -Depth 40
    return
}

Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
$source = Get-Sprint8CSourceIdentity -RequireClean
$composePath = Resolve-Sprint8CRepositoryPath -Path $ComposeFile
$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT"
)
$environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $environmentNames
$ownedTopology = $false
$datasetStopped = $false
$responseProxyStopped = $false
$ports = $null
$fixturePair = $null
$fixtureIdentity = $null
$configuration = $null
$initialTopology = $null
$restoredTopology = $null
$checks = [ordered]@{}
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$failure = $null
$evidenceFullPath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
$childRoot = Join-Path (Split-Path -Parent $evidenceFullPath) "materialization"
[IO.Directory]::CreateDirectory($childRoot) | Out-Null

try {
    if ($UseExistingTopology) {
        foreach ($name in @("TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT")) {
            if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
                throw "Existing deployed smoke requires inherited '$name'."
            }
        }
        $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$env:TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$env:TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$env:TESSARA_SUPERVISOR_PORT)
        Assert-Sprint8CExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
        if ([string]::IsNullOrWhiteSpace($FixtureReceiptPath)) {
            throw "Existing deployed smoke requires -FixtureReceiptPath."
        }
    } else {
        Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject `
            -Authorized ([bool]$AuthorizeDisposableReset)
        $materializationEvidence = Join-Path $childRoot "result.json"
        $arguments = @(
            "-Target", "ReferenceNoOp", "-ComposeProject", $ComposeProject,
            "-EvidencePath", $materializationEvidence, "-AuthorizeDisposableReset", "-KeepTopology"
        )
        if ($SkipBuild) { $arguments += "-SkipBuild" }
        Invoke-Sprint8CChildScript -ScriptPath "scripts/materialize-sprint-8c.ps1" `
            -Arguments $arguments | Out-Null
        $materialized = Get-Content -LiteralPath $materializationEvidence -Raw | ConvertFrom-Json -Depth 100
        if ([string]$materialized.state -cne "passed" -or $null -eq $materialized.environment -or
            [string]::IsNullOrWhiteSpace([string]$materialized.fixture_receipt_path)) {
            throw "Deployed smoke materialization did not publish topology context and fixture identity."
        }
        $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$materialized.environment.TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$materialized.environment.TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$materialized.environment.TESSARA_SUPERVISOR_PORT)
        $FixtureReceiptPath = [string]$materialized.fixture_receipt_path
        $ownedTopology = $true
    }
    $resolvedFixturePath = Resolve-Sprint8CRepositoryPath -Path $FixtureReceiptPath
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $resolvedFixturePath `
        -SidecarPath "$resolvedFixturePath.sha256")) {
        throw "Deployed smoke requires an authenticated fixture receipt pair."
    }
    $fixtureDocument = Get-Content -LiteralPath $resolvedFixturePath -Raw | ConvertFrom-Json -Depth 100
    if ([string]$fixtureDocument.compose_project -cne $ComposeProject) {
        throw "Deployed smoke fixture receipt belongs to a different Compose project."
    }
    $fixturePair = [pscustomobject][ordered]@{
        path = $resolvedFixturePath
        sha256 = (Get-FileHash -LiteralPath $resolvedFixturePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $fixtureIdentity = Get-Sprint8CSmokeFixtureIdentity -FixtureReceipt $fixtureDocument
    $configuration = Get-Sprint8CComposeConfiguration -ComposePath $composePath `
        -ComposeProject $ComposeProject
    Assert-Sprint8CDatabaseIsolationConfiguration -ComposeConfiguration $configuration | Out-Null
    $initialTopology = @(Get-Sprint8CSmokeTopologySnapshot -ComposePath $composePath)

    $session = [Microsoft.PowerShell.Commands.WebRequestSession]::new()
    Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url -Path "/api/auth/login" `
        -Session $session -Method POST -Body ([ordered]@{
            email = "admin@tessara.local"; password = "tessara-dev-admin"
        }) -ExpectedStatus @(200) | Out-Null

    $inventory = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/admin/modules" -Session $session).document
    $expectedReleases = [ordered]@{
        "tessara.responses" = "1.0.0"
        "tessara.datasets" = "1.0.0"
        "tessara.components" = "1.1.0"
        "tessara.dashboards" = "3.0.2"
    }
    foreach ($definitionId in $expectedReleases.Keys) {
        $entry = @($inventory.entries | Where-Object {
            [string]$_.kind -ceq "independently_deployed" -and
            [string]$_.definition.id -ceq $definitionId
        })
        if ($entry.Count -ne 1 -or [string]$entry[0].release.version -cne $expectedReleases[$definitionId] -or
            -not [bool]$entry[0].instance.ready -or -not [bool]$entry[0].instance.healthy) {
            throw "Module inventory does not expose exact healthy release '$definitionId@$($expectedReleases[$definitionId])'."
        }
    }
    $navigation = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/shell/navigation" -Session $session).document
    $navigationItems = @($navigation.groups | ForEach-Object { $_.items })
    $datasetNavigation = @($navigationItems | Where-Object {
        [string]$_.contribution_id -ceq "tessara.datasets.navigation"
    })
    $responseNavigation = @($navigationItems | Where-Object {
        [string]$_.contribution_id -ceq "tessara.responses.navigation"
    })
    if ([string]$navigation.state -cne "available" -or $datasetNavigation.Count -ne 1 -or
        [string]$datasetNavigation[0].key -cne "datasets" -or
        [string]$datasetNavigation[0].label -cne "Datasets" -or
        [string]$datasetNavigation[0].href -cne "/datasets" -or
        [string]$datasetNavigation[0].owner -cne "contribution") {
        throw "Dataset navigation identity is missing, duplicated, or substituted."
    }
    if ($responseNavigation.Count -ne 1 -or
        [string]$responseNavigation[0].key -cne "responses" -or
        [string]$responseNavigation[0].label -cne "Responses" -or
        [string]$responseNavigation[0].href -cne "/responses" -or
        [string]$responseNavigation[0].owner -cne "contribution") {
        throw "Response navigation identity is missing, duplicated, or substituted."
    }
    $checks.inventory_navigation = [pscustomobject][ordered]@{
        state = "passed"
        releases = $expectedReleases
        response_navigation = $responseNavigation[0]
        dataset_navigation = $datasetNavigation[0]
    }

    $manifest = (Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "datasets" -Port 8093 -ModuleLabel "Dataset" `
        -Path "/api/manifest" -ControlKey).document
    $currentConfiguration = Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "datasets" -Port 8093 -ModuleLabel "Dataset" -Path "/api/configuration"
    $validatedConfiguration = Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "datasets" -Port 8093 -ModuleLabel "Dataset" `
        -Path "/api/configuration/validate" -Method POST `
        -Body ($currentConfiguration.document | ConvertTo-Json -Depth 30 -Compress)
    $appliedConfiguration = Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "datasets" -Port 8093 -ModuleLabel "Dataset" `
        -Path "/api/configuration" -Method PUT -ControlKey `
        -Body ($currentConfiguration.document | ConvertTo-Json -Depth 30 -Compress)
    $diagnostics = Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "datasets" -Port 8093 -ModuleLabel "Dataset" `
        -Path "/api/diagnostics" -ControlKey
    $responseManifest = (Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "responses" -Port 8094 -ModuleLabel "Response" `
        -Path "/api/manifest" -ControlKey).document
    $responseCurrentConfiguration = Invoke-Sprint8CInternalModuleRequest `
        -ComposePath $composePath -Service "responses" -Port 8094 `
        -ModuleLabel "Response" -Path "/api/configuration" -ControlKey
    $responseValidatedConfiguration = Invoke-Sprint8CInternalModuleRequest `
        -ComposePath $composePath -Service "responses" -Port 8094 `
        -ModuleLabel "Response" -Path "/api/configuration/validate" -Method POST `
        -ControlKey `
        -Body ($responseCurrentConfiguration.document | ConvertTo-Json -Depth 30 -Compress)
    $responseAppliedConfiguration = Invoke-Sprint8CInternalModuleRequest `
        -ComposePath $composePath -Service "responses" -Port 8094 `
        -ModuleLabel "Response" -Path "/api/configuration" -Method PUT -ControlKey `
        -Body ($responseCurrentConfiguration.document | ConvertTo-Json -Depth 30 -Compress)
    $responseDiagnostics = Invoke-Sprint8CInternalModuleRequest -ComposePath $composePath `
        -Service "responses" -Port 8094 -ModuleLabel "Response" `
        -Path "/api/diagnostics" -ControlKey
    if ([string]$manifest.definition_id -cne "tessara.datasets" -or
        [string]$manifest.release_version -cne "1.0.0" -or
        @($validatedConfiguration.document.findings).Count -ne 0 -or
        $null -eq $validatedConfiguration.document.normalized -or
        @($appliedConfiguration.document.findings).Count -ne 0 -or
        [string]$diagnostics.document.module -cne "tessara.datasets" -or
        [string]$diagnostics.document.release -cne "1.0.0") {
        throw "Dataset manifest/configuration/diagnostics generic control contract failed."
    }
    $diagnosticsText = $diagnostics.document | ConvertTo-Json -Depth 100 -Compress
    if ($diagnosticsText -match '(?i)postgres(?:ql)?://|password|secret|signing[_-]?key|response\.initial') {
        throw "Dataset diagnostics disclosed a credential or product-row identity."
    }
    $responseDiagnosticsContract = Assert-Sprint8CResponseDiagnosticsEnvelope `
        -Diagnostics $responseDiagnostics.document
    if ([string]$responseManifest.definition_id -cne "tessara.responses" -or
        [string]$responseManifest.release_version -cne "1.0.0" -or
        @($responseValidatedConfiguration.document.findings).Count -ne 0 -or
        $null -eq $responseValidatedConfiguration.document.normalized -or
        @($responseAppliedConfiguration.document.findings).Count -ne 0) {
        throw "Response manifest/configuration/diagnostics generic control contract failed."
    }
    $responseDiagnosticsText = $responseDiagnostics.document | ConvertTo-Json -Depth 100 -Compress
    if ($responseDiagnosticsText -match
        '(?i)postgres(?:ql)?://|password|secret|signing[_-]?key|response\.draft\.owner') {
        throw "Response diagnostics disclosed a credential or product-row identity."
    }
    foreach ($responseId in @($fixtureIdentity.response_fixtures.response_ids.PSObject.Properties.Value)) {
        if ($responseDiagnosticsText.Contains([string]$responseId, [StringComparison]::Ordinal)) {
            throw "Response diagnostics disclosed a prepared product-row identity."
        }
    }
    $checks.control_contract = [pscustomobject][ordered]@{
        state = "passed"
        response = [pscustomobject][ordered]@{
            manifest = "tessara.responses@1.0.0"
            configuration_round_trip = "exact"
            diagnostics = $responseDiagnosticsContract
        }
        dataset = [pscustomobject][ordered]@{
            manifest = "tessara.datasets@1.0.0"
            configuration_round_trip = "exact"
            diagnostics = "sanitized"
        }
    }

    $responseDirectory = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/responses" -Session $session
    $responseDraftDocument = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/responses/$($fixtureIdentity.draft_response_id)" -Session $session
    $responseList = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/responses" -Session $session
    $responseDraftDetail = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/responses/$($fixtureIdentity.draft_response_id)" -Session $session
    if ($responseDraftDocument.body -notmatch 'Response') {
        throw "Response detail document did not render through the canonical module route."
    }
    $responseProduct = Assert-Sprint8CResponseProductProjection `
        -DirectoryResponse $responseDirectory -ListDocument @($responseList.document) `
        -DraftDetail $responseDraftDetail.document `
        -ResponseFixtureProof $fixtureIdentity.response_fixtures
    $checks.response_product = [pscustomobject][ordered]@{
        state = "passed"
        owner = "tessara.responses"
        release = "1.0.0"
        projection = $responseProduct
        detail_document_sha256 = $responseDraftDocument.body_sha256
        list_api_sha256 = $responseList.body_sha256
        detail_api_sha256 = $responseDraftDetail.body_sha256
    }

    $datasetDirectory = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/datasets" -Session $session
    $datasetNew = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/datasets/new" -Session $session
    $datasetList = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets" -Session $session
    $baseDetail = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)" -Session $session
    $baseTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)/table" -Session $session
    $derivedFirstTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.derived_first_dataset_id)/table" -Session $session
    $derivedTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.derived_dataset_id)/table" -Session $session
    $independentTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.independent_dataset_id)/table" -Session $session
    if ($datasetDirectory.body -notmatch 'Datasets' -or $datasetNew.body -notmatch 'Dataset' -or
        @($datasetList.document).Count -lt 4 -or
        [string]$baseDetail.document.id -cne $fixtureIdentity.dataset_id) {
        throw "Dataset direct documents/API did not resolve canonical owner read-back."
    }

    $componentList = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/components" -Session $session
    $tableComponent = @($componentList.document | Where-Object {
        [string]$_.current_version.component_version_id -ceq $fixtureIdentity.component_version_id
    })
    if ($tableComponent.Count -ne 1) {
        throw "Component list did not resolve the owner-read-back table Component version."
    }
    $componentId = [string]$tableComponent[0].component_id
    $componentKind = [string]$tableComponent[0].current_version.component_type
    if ($componentKind -cne "table") {
        throw "Component fixture identity did not resolve the canonical table render kind."
    }
    $componentExecution = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/components/$componentId/$componentKind" -Session $session
    $dashboardList = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/dashboards" -Session $session
    $dashboardDetail = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/dashboards/$($fixtureIdentity.dashboard_id)" -Session $session
    $dashboardPlacements = @($dashboardDetail.document.placements | Where-Object {
        $componentProperty = $_.PSObject.Properties['component']
        $null -ne $componentProperty -and $null -ne $componentProperty.Value -and
        [string]$componentProperty.Value.component_version_id -ceq $fixtureIdentity.component_version_id
    })
    if ($dashboardPlacements.Count -ne 1 -or
        [string]$dashboardPlacements[0].availability -cne "available" -or
        [string]$dashboardPlacements[0].component.component_type -cne "table") {
        throw "Dashboard detail did not resolve one available canonical table Component placement."
    }
    $dashboardPlacementId = [string]$dashboardPlacements[0].placement_id
    $dashboardRenderPath = "/api/dashboards/$($fixtureIdentity.dashboard_id)/placements/$dashboardPlacementId/render/table"
    $dashboardExecution = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path $dashboardRenderPath -Session $session
    $dashboardView = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/dashboards/$($fixtureIdentity.dashboard_id)/view" -Session $session
    $dashboardDocument = Assert-Sprint8CDashboardDocumentHealthy `
        -DashboardResponse $dashboardView -AvailablePlacementId $dashboardPlacementId `
        -ExpectedPresentation $componentKind
    if (@($dashboardList.document | Where-Object { [string]$_.id -ceq $fixtureIdentity.dashboard_id }).Count -ne 1 -or
        [string]$dashboardDocument.state -cne "passed") {
        throw "Dashboard did not resolve the canonical Component-backed fixture."
    }
    $checks.cross_module = [pscustomobject][ordered]@{
        state = "passed"
        response_directory_sha256 = $responseDirectory.body_sha256
        response_detail_sha256 = $responseDraftDocument.body_sha256
        dataset_directory_sha256 = $datasetDirectory.body_sha256
        base_table_sha256 = $baseTable.body_sha256
        derived_first_table_sha256 = $derivedFirstTable.body_sha256
        derived_table_sha256 = $derivedTable.body_sha256
        independent_table_sha256 = $independentTable.body_sha256
        component_execution_sha256 = $componentExecution.body_sha256
        dashboard_execution_sha256 = $dashboardExecution.body_sha256
        dashboard_view_sha256 = $dashboardView.body_sha256
    }

    Invoke-Sprint8CDockerCompose -ComposePath $composePath `
        -Arguments @("stop", "response-provider-proxy") | Out-Null
    $responseProxyStopped = $true
    $refreshKey = "sprint-8c-smoke-response-outage-$([Guid]::NewGuid().ToString('N'))"
    $failedRefresh = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/admin/datasets/$($fixtureIdentity.dataset_id)/refresh" `
        -Session $session -Method POST -Headers @{ "x-idempotency-key" = $refreshKey } `
        -ExpectedStatus @(503)
    if ([int]$failedRefresh.document.schema_version -ne 1 -or
        [string]$failedRefresh.document.error.code -cne "dataset.dependency_unavailable" -or
        [string]$failedRefresh.document.correlation_id -cnotmatch
            '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
        throw "Response provider outage did not return the exact sanitized Dataset error envelope."
    }
    $afterFailedRefresh = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)" -Session $session
    $afterFailedTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)/table" -Session $session
    $afterFailedComponent = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/components/$componentId/$componentKind" -Session $session
    $afterFailedDashboard = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path $dashboardRenderPath -Session $session
    $degradation = Assert-Sprint8CLastGoodDegradation -BeforeDetail $baseDetail.document `
        -AfterDetail $afterFailedRefresh.document -BeforeTableSha256 $baseTable.body_sha256 `
        -AfterTableSha256 $afterFailedTable.body_sha256 `
        -BeforeComponentSha256 $componentExecution.body_sha256 `
        -AfterComponentSha256 $afterFailedComponent.body_sha256 `
        -BeforeDashboardSha256 $dashboardExecution.body_sha256 `
        -AfterDashboardSha256 $afterFailedDashboard.body_sha256
    Invoke-Sprint8CDockerCompose -ComposePath $composePath `
        -Arguments @("start", "response-provider-proxy") | Out-Null
    $responseProxyStopped = $false
    for ($attempt = 1; $attempt -le 30; $attempt++) {
        $proxyState = @(Get-Sprint8CComposeServiceState -ComposePath $composePath | Where-Object {
            [string]$_.Service -ceq "response-provider-proxy"
        })
        if ($proxyState.Count -eq 1 -and [string]$proxyState[0].State -ceq "running" -and
            [string]$proxyState[0].Health -ceq "healthy") { break }
        if ($attempt -eq 30) { throw "Response provider proxy did not recover healthy." }
        Start-Sleep -Seconds 1
    }
    $recoveryRefresh = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/admin/datasets/$($fixtureIdentity.dataset_id)/refresh" `
        -Session $session -Method POST -Headers @{ "x-idempotency-key" = $refreshKey } `
        -ExpectedStatus @(200)
    $promotedBaseDetail = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)" -Session $session
    $promotedBaseTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)/table" -Session $session
    $promotedDerivedFirstTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.derived_first_dataset_id)/table" -Session $session
    $promotedDerivedTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.derived_dataset_id)/table" -Session $session
    $promotedIndependentTable = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.independent_dataset_id)/table" -Session $session
    $promotedComponent = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/components/$componentId/$componentKind" -Session $session
    $promotedDashboardExecution = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path $dashboardRenderPath -Session $session
    $promotedDashboard = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/dashboards/$($fixtureIdentity.dashboard_id)/view" -Session $session
    $closure = Assert-Sprint8CRefreshClosureTransition -Refresh $recoveryRefresh.document `
        -BeforeClosureSha256 @(
            $baseTable.body_sha256, $derivedFirstTable.body_sha256, $derivedTable.body_sha256
        ) -AfterClosureSha256 @(
            $promotedBaseTable.body_sha256, $promotedDerivedFirstTable.body_sha256,
            $promotedDerivedTable.body_sha256
        ) -BeforeIndependentSha256 $independentTable.body_sha256 `
        -AfterIndependentSha256 $promotedIndependentTable.body_sha256 `
        -BeforeComponentSha256 $componentExecution.body_sha256 `
        -AfterComponentSha256 $promotedComponent.body_sha256 `
        -BeforeDashboardSha256 $dashboardExecution.body_sha256 `
        -AfterDashboardSha256 $promotedDashboardExecution.body_sha256
    $promotedDashboardDocument = Assert-Sprint8CDashboardDocumentHealthy `
        -DashboardResponse $promotedDashboard -AvailablePlacementId $dashboardPlacementId `
        -ExpectedPresentation $componentKind
    if ([string]$promotedBaseDetail.document.freshness.state -cne "current" -or
        [string]$promotedDashboardDocument.state -cne "passed") {
        throw "Response provider recovery did not restore a current Dataset and healthy downstream view."
    }
    $checks.provider_outage = [pscustomobject][ordered]@{
        state = "passed"; failure_status = 503
        failure_code = [string]$failedRefresh.document.error.code
        last_good = $degradation
        same_retry_converged = $true
        closure = $closure
    }

    $initialOperations = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/operations/status" -Session $session).document
    $initialSummary = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/summary" -Session $session).document
    $initialForm = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/forms/$($fixtureIdentity.form_id)" -Session $session).document

    Invoke-Sprint8CDockerCompose -ComposePath $composePath -Arguments @("stop", "datasets") | Out-Null
    $datasetStopped = $true
    Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url -Path "/health" `
        -Session $session -ExpectedStatus @(200) | Out-Null
    Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url -Path "/api/datasets" `
        -Session $session -ExpectedStatus @(503) | Out-Null
    Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/components/$componentId/$componentKind" -Session $session `
        -ExpectedStatus @(503) | Out-Null
    $degradedDashboardExecution = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path $dashboardRenderPath -Session $session -ExpectedStatus @(503)
    $degradedDashboard = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/dashboards/$($fixtureIdentity.dashboard_id)/view" -Session $session `
        -ExpectedStatus @(200)
    $unrelatedForms = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/forms" -Session $session -ExpectedStatus @(200)
    $outageForm = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/forms/$($fixtureIdentity.form_id)" -Session $session `
        -ExpectedStatus @(200)).document
    $outageOperations = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/operations/status" -Session $session).document
    $outageSummary = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/summary" -Session $session).document
    if ($degradedDashboard.body -notmatch 'Dashboard' -or
        [int]$degradedDashboardExecution.status -ne 503 -or
        $null -eq $unrelatedForms.document) {
        throw "Dataset outage did not preserve usable unrelated content with coherent Dashboard degradation."
    }

    Invoke-Sprint8CDockerCompose -ComposePath $composePath -Arguments @("start", "datasets") | Out-Null
    $datasetStopped = $false
    $freshDatasetReadiness = $null
    for ($attempt = 1; $attempt -le 60; $attempt++) {
        $datasetState = @(Get-Sprint8CComposeServiceState -ComposePath $composePath | Where-Object {
            [string]$_.Service -ceq "datasets"
        })
        if ($datasetState.Count -eq 1 -and [string]$datasetState[0].State -ceq "running" -and
            [string]$datasetState[0].Health -ceq "healthy") {
            try {
                $candidateReadiness = Invoke-Sprint8CInternalModuleRequest `
                    -ComposePath $composePath -Service "datasets" -Port 8093 `
                    -ModuleLabel "Dataset" -Path "/health/ready"
                $freshDatasetReadiness = Assert-Sprint8CFreshDatasetReadiness `
                    -ReadinessResponse $candidateReadiness
                break
            } catch {
                if ($attempt -eq 60) { throw }
            }
        }
        if ($attempt -eq 60) {
            throw "Dataset service did not recover with a fresh ready response."
        }
        Start-Sleep -Seconds 1
    }
    $recoveredOperations = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/operations/status" -Session $session).document
    $recoveredSummary = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/summary" -Session $session).document
    $recoveredForm = (Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/forms/$($fixtureIdentity.form_id)" -Session $session).document
    $availability = Assert-Sprint8CSmokeAvailabilityTransition `
        -InitialOperations $initialOperations -InitialSummary $initialSummary `
        -OutageOperations $outageOperations -OutageSummary $outageSummary `
        -RecoveredOperations $recoveredOperations -RecoveredSummary $recoveredSummary
    $formAvailability = Assert-Sprint8CFormSourceUsageTransition `
        -InitialForm $initialForm -OutageForm $outageForm -RecoveredForm $recoveredForm
    $recoveredDataset = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/datasets/$($fixtureIdentity.dataset_id)" -Session $session
    $recoveredComponent = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/components/$componentId/$componentKind" -Session $session
    $recoveredDashboardExecution = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path $dashboardRenderPath -Session $session
    $recoveredDashboard = Invoke-Sprint8CSmokeRequest -BaseUrl $ports.gateway_url `
        -Path "/dashboards/$($fixtureIdentity.dashboard_id)/view" -Session $session
    $recoveredDashboardDocument = Assert-Sprint8CDashboardDocumentHealthy `
        -DashboardResponse $recoveredDashboard -AvailablePlacementId $dashboardPlacementId `
        -ExpectedPresentation $componentKind
    if (($recoveredDataset.document | ConvertTo-Json -Depth 100 -Compress) -cne
            ($promotedBaseDetail.document | ConvertTo-Json -Depth 100 -Compress) -or
        $recoveredComponent.body_sha256 -cne $promotedComponent.body_sha256 -or
        $recoveredDashboardExecution.body_sha256 -cne $promotedDashboardExecution.body_sha256 -or
        [string]$recoveredDashboardDocument.state -cne "passed") {
        throw "Dataset/Component/Dashboard did not recover exact last-good behavior."
    }
    $checks.outage_recovery = [pscustomobject][ordered]@{
        state = "passed"; reverse_consumers = $availability
        form_dataset_sources = $formAvailability
        core_and_unrelated_content_usable = $true
        fresh_dataset_readiness = $freshDatasetReadiness
        dataset_component_dashboard_exact_recovery = $true
    }

    $databaseDenials = @(Assert-Sprint8CPairwiseDatabaseDenial `
        -ComposeConfiguration $configuration -ComposePath $composePath)
    $checks.database_isolation = [pscustomobject][ordered]@{
        state = "passed"; attempted_pairs = $databaseDenials.Count; denials = $databaseDenials
    }
    $restoredTopology = @(Get-Sprint8CSmokeTopologySnapshot -ComposePath $composePath)
    if (($initialTopology | ConvertTo-Json -Depth 30 -Compress) -cne
        ($restoredTopology | ConvertTo-Json -Depth 30 -Compress)) {
        throw "Smoke outage/recovery changed container/image/restart topology."
    }
} catch {
    $failure = $_
} finally {
    try {
        if ($null -ne $ports) {
            if ($responseProxyStopped) {
                Invoke-Sprint8CDockerCompose -ComposePath $composePath `
                    -Arguments @("start", "response-provider-proxy") | Out-Null
                $responseProxyStopped = $false
            }
            if ($datasetStopped) {
                Invoke-Sprint8CDockerCompose -ComposePath $composePath `
                    -Arguments @("start", "datasets") | Out-Null
                $datasetStopped = $false
            }
        }
        if ($ownedTopology) {
            $cleanup = Remove-Sprint8CProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized ([bool]$AuthorizeDisposableReset)
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
            $cleanup | Add-Member -NotePropertyName mode -NotePropertyValue "exact-project-teardown" -Force
        } elseif ($UseExistingTopology) {
            $restored = $false
            for ($attempt = 1; $attempt -le 60; $attempt++) {
                try {
                    Assert-Sprint8CExistingTopology -ComposePath $composePath `
                        -ComposeProject $ComposeProject | Out-Null
                    $restored = $true
                    break
                } catch {
                    if ($attempt -eq 60) { throw }
                    Start-Sleep -Seconds 1
                }
            }
            if (-not $restored) { throw "Existing Sprint 8C smoke topology did not restore healthy." }
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"; mode = "existing-topology-restored-and-retained"
            }
        }
    } catch {
        if ($null -eq $failure) { $failure = $_ }
        $cleanup = [pscustomobject][ordered]@{
            state = "failed"; error = $_.Exception.Message
        }
    } finally {
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$configurationHash = if ($null -eq $configuration) { $null } else {
    Get-Sprint7ASha256 -Text (($configuration | ConvertTo-Json -Depth 100 -Compress) + "`n")
}
$fixtureHash = if ($null -eq $fixturePair) { "none" } else { [string]$fixturePair.sha256 }
$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n$configurationHash`n$fixtureHash`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    proof = "deployed-acceptance-smoke"
    state = if ($null -eq $failure -and [string]$cleanup.state -ceq "passed") { "passed" } else { "failed" }
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    fixture_receipt = $fixturePair
    checks = [pscustomobject]$checks
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
        message = $failure.Exception.Message
        category = [string]$failure.CategoryInfo.Category
    } }
}
Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8C deployed smoke failed; retained evidence: $evidenceFullPath"
}

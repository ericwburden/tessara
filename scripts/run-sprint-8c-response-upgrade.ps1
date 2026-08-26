[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8c-implementation-upgrade",
    [string]$ComposeFile = "deploy/sprint-8c/compose.yaml",
    [switch]$UseExistingTopology,
    [string]$FixtureReceiptPath,
    [string]$UpgradeContractPath = "deploy/sprint-8c/fixtures/upgrade-fixture-contract.json",
    [string]$CandidateManifestPath = "crates/tessara-response-module/manifest.json",
    [string]$BaselineImageTag = "tessara-sprint-8c-responses-upgrade-baseline:latest",
    [string]$EvidencePath = "target/sprint-8c-response-upgrade/result.json",
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

$responseDefinition = "tessara.responses"
$baselineRelease = "0.9.0"
$candidateRelease = "1.0.0"
$baselineReceiptContract = "tessara.sprint-8c.response-prior-compatible-release"
$responseExecutable = "/usr/local/bin/response-module"
$baselineSourceCommit = "13eb6ffaa9479fa71f4270edb049d079495d1a79"
$baselineSourceTree = "c99cf7e156dd157a705b8354d706f9b8be7baeb9"
$baselineSourceModel = "authenticated-git-snapshot-plus-dedicated-patch"
$baselineFixtureKind = "authenticated-independent-source-release-v2"
$baselineManifestProjection = "independent-source-behavioral-compatibility"
$allowedBaselineManifestVariations = @("release_version", "browser_lifecycle", "assets")
$expectedStages = @(
    "establish-compatible-baseline",
    "upgrade-to-candidate",
    "rollback-to-compatible-baseline",
    "restore-intended-candidate"
)

function Copy-Sprint8CJsonValue {
    param([Parameter(Mandatory)]$Value)
    $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
}

function ConvertTo-Sprint8CCanonicalJsonValue {
    param([AllowNull()]$Value)

    if ($null -eq $Value) { return $null }
    if ($Value -is [System.Collections.IDictionary]) {
        $names = [string[]]@($Value.Keys | ForEach-Object { [string]$_ })
        [Array]::Sort($names, [StringComparer]::Ordinal)
        $canonical = [ordered]@{}
        foreach ($name in $names) {
            $canonical[$name] = ConvertTo-Sprint8CCanonicalJsonValue -Value $Value[$name]
        }
        return [pscustomobject]$canonical
    }
    if ($Value -is [pscustomobject]) {
        $names = [string[]]@($Value.PSObject.Properties | ForEach-Object { $_.Name })
        [Array]::Sort($names, [StringComparer]::Ordinal)
        $canonical = [ordered]@{}
        foreach ($name in $names) {
            $canonical[$name] = ConvertTo-Sprint8CCanonicalJsonValue `
                -Value $Value.PSObject.Properties[$name].Value
        }
        return [pscustomobject]$canonical
    }
    if ($Value -is [System.Collections.IEnumerable] -and $Value -isnot [string]) {
        $canonical = [System.Collections.Generic.List[object]]::new()
        foreach ($item in $Value) {
            $canonical.Add((ConvertTo-Sprint8CCanonicalJsonValue -Value $item))
        }
        Write-Output -NoEnumerate $canonical.ToArray()
        return
    }
    $Value
}

function ConvertTo-Sprint8CStableJson {
    param([AllowNull()]$Value)
    if ($null -eq $Value) { return "null" }
    ConvertTo-Sprint8CCanonicalJsonValue -Value $Value |
        ConvertTo-Json -Depth 100 -Compress
}

function Get-Sprint8CManifestBehaviorProjection {
    param([Parameter(Mandatory)]$Manifest)
    $projection = Copy-Sprint8CJsonValue -Value $Manifest
    foreach ($property in $allowedBaselineManifestVariations) {
        $projection.PSObject.Properties.Remove($property)
    }
    $projection
}

function Assert-Sprint8CExactSequence {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Expected,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Actual,
        [Parameter(Mandatory)][string]$Label
    )
    $actualText = @($Actual | ForEach-Object { [string]$_ })
    if ($Expected.Count -ne $actualText.Count) {
        throw "$Label count mismatch: expected $($Expected.Count), found $($actualText.Count)."
    }
    for ($index = 0; $index -lt $Expected.Count; $index++) {
        if ($Expected[$index] -cne $actualText[$index]) {
            throw "$Label mismatch at index ${index}: expected '$($Expected[$index])', found '$($actualText[$index])'."
        }
    }
}

function Read-Sprint8CUpgradeContract {
    $path = Resolve-Sprint8CRepositoryPath -Path $UpgradeContractPath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Response upgrade fixture contract is missing: $path"
    }
    $contract = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -Depth 100
    if ([int]$contract.schema_version -ne 1 -or
        [string]$contract.contract -cne "tessara.sprint-8c.response-upgrade-fixture" -or
        [string]$contract.transition.owner -cne $responseDefinition -or
        [string]$contract.transition.mechanism -cne "resolved_one_owner_blueprint_delta" -or
        [string]$contract.transition.intended_release -cne $candidateRelease) {
        throw "Response upgrade fixture contract does not declare the exact one-owner Sprint 8C transition."
    }
    Assert-Sprint8CExactSequence -Expected @(
        $baselineRelease, $candidateRelease, $baselineRelease, $candidateRelease
    ) -Actual @($contract.sequence) -Label "Response upgrade release sequence"
    Assert-Sprint8CExactSequence -Expected $expectedStages -Actual @($contract.stages) `
        -Label "Response upgrade stage sequence"
    $artifact = $contract.prior_compatible_artifact
    if ([string]$artifact.release -cne $baselineRelease -or
        [string]$artifact.source_model -cne $baselineSourceModel -or
        [string]$artifact.base_commit -cne $baselineSourceCommit -or
        [string]$artifact.base_tree -cne $baselineSourceTree -or
        [string]$artifact.patch_path -cne "deploy/sprint-8c/baselines/response-0.9.0.patch" -or
        [string]$artifact.patch_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$artifact.fixture_kind -cne $baselineFixtureKind -or
        [string]$artifact.manifest_projection -cne $baselineManifestProjection) {
        throw "Response upgrade fixture does not authenticate an independent 0.9.0 source release."
    }
    $baselinePatchPath = Resolve-Sprint8CRepositoryPath -Path ([string]$artifact.patch_path)
    if (-not (Test-Path -LiteralPath $baselinePatchPath -PathType Leaf) -or
        (Get-FileHash -LiteralPath $baselinePatchPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne
            [string]$artifact.patch_sha256) {
        throw "Response upgrade fixture baseline source patch failed SHA-256 authentication."
    }
    Assert-Sprint8CExactSequence -Expected @() -Actual @($artifact.cargo_features) `
        -Label "Response baseline Cargo features"
    Assert-Sprint8CExactSequence -Expected $allowedBaselineManifestVariations `
        -Actual @($artifact.allowed_manifest_variations) `
        -Label "Response baseline allowed Manifest variations"
    if ([string]$contract.schema_compatibility.mode -cne
            "reads_current_1_0_0_response_baseline_after_rollback" -or
        [string]$contract.schema_compatibility.source_independence -cne
            "authenticated_pinned_git_tree_plus_dedicated_patch" -or
        [string]$contract.schema_compatibility.candidate_source_dependency -cne "none" -or
        [string]$contract.schema_compatibility.rollback_reader_contract -cne
            "same_owned_schema_and_product_read_paths") {
        throw "Response baseline source/schema compatibility declaration is not exact."
    }
    $expectedFixed = [ordered]@{
        core = "0.1.0"
        "tessara.datasets" = "1.0.0"
        "tessara.components" = "1.1.0"
        "tessara.dashboards" = "3.0.2"
        "tessara.reference.scoped-records" = "1.0.2"
    }
    Assert-Sprint8CExactSequence -Expected @($expectedFixed.Keys) `
        -Actual @($contract.fixed_dependencies.PSObject.Properties.Name) `
        -Label "Response upgrade fixed owners"
    foreach ($owner in $expectedFixed.Keys) {
        if ([string]$contract.fixed_dependencies.$owner -cne $expectedFixed[$owner]) {
            throw "Response upgrade fixed owner '$owner' has a substituted release."
        }
    }
    Assert-Sprint8CExactSequence -Expected @(
        "response_state", "module_instance_identity", "typed_resource_identity",
        "navigation_identity", "outbox_positions"
    ) -Actual @($contract.preserved) -Label "Response upgrade preserved state"
    Assert-Sprint8CExactSequence -Expected @(
        "images", "containers", "restart_counts", "owner_data", "navigation", "availability"
    ) -Actual @($contract.unrelated_unchanged) -Label "Response upgrade unrelated state"
    $contract
}

function Assert-Sprint8CExactResponseDeltaPlan {
    param(
        [Parameter(Mandatory)]$Lockfile,
        [Parameter(Mandatory)][string]$ExpectedImageDigest
    )
    $expected = @(
        [ordered]@{ action = "acquire_image"; component = $responseDefinition; digest = $ExpectedImageDigest },
        [ordered]@{ action = "migrate"; owner = $responseDefinition; image = $ExpectedImageDigest },
        [ordered]@{ action = "health_gate"; owner = $responseDefinition },
        [ordered]@{ action = "switch_traffic"; owner = $responseDefinition },
        [ordered]@{ action = "verify_read_back" }
    )
    $actual = @($Lockfile.materialization_plan.actions)
    if ((ConvertTo-Sprint8CStableJson $actual) -cne
        (ConvertTo-Sprint8CStableJson $expected)) {
        throw "Response transition is not the exact one-owner semantic delta: $(ConvertTo-Sprint8CStableJson $actual)"
    }
    [pscustomobject][ordered]@{
        owner = $responseDefinition
        action_count = $actual.Count
        actions = $actual
        exact = $true
    }
}

function Get-Sprint8CFixedLockfileProjection {
    param([Parameter(Mandatory)]$Lockfile)

    [pscustomobject][ordered]@{
        core = $Lockfile.core
        modules = @($Lockfile.modules | Where-Object {
            [string]$_.definition_id -cne $responseDefinition
        } | Sort-Object definition_id)
        navigation = @($Lockfile.navigation)
        roles = @($Lockfile.roles)
        administrator_enrollment_role = [string]$Lockfile.administrator_enrollment_role
        capability_floor_version = [string]$Lockfile.capability_floor_version
        secret_references = $Lockfile.secret_references
    }
}

function Assert-Sprint8CFixedReleaseVersions {
    param(
        [Parameter(Mandatory)]$Lockfile,
        [Parameter(Mandatory)]$Contract
    )

    if ([string]$Lockfile.core.version -cne [string]$Contract.fixed_dependencies.core) {
        throw "Response upgrade topology has a substituted Core release."
    }
    $expectedDefinitions = @(
        $responseDefinition,
        "tessara.datasets",
        "tessara.components",
        "tessara.dashboards",
        "tessara.reference.scoped-records"
    ) | Sort-Object
    Assert-Sprint8CExactSequence -Expected $expectedDefinitions `
        -Actual @($Lockfile.modules.definition_id | Sort-Object) `
        -Label "Response upgrade resolved module owners"
    foreach ($owner in @($Contract.fixed_dependencies.PSObject.Properties.Name | Where-Object {
        $_ -cne "core"
    })) {
        $release = @($Lockfile.modules | Where-Object {
            [string]$_.definition_id -ceq $owner
        })
        if ($release.Count -ne 1 -or
            [string]$release[0].version -cne [string]$Contract.fixed_dependencies.$owner) {
            throw "Response upgrade topology has a substituted '$owner' release."
        }
    }
    $true
}

function Get-Sprint8CUpgradeAuthorizationProjection {
    param([Parameter(Mandatory)]$Module)
    $diagnostics = $Module.PSObject.Properties["diagnostics"]
    if ($null -eq $diagnostics -or $null -eq $diagnostics.Value) { return $null }
    $details = $diagnostics.Value.PSObject.Properties["details"]
    if ($null -eq $details -or $null -eq $details.Value) { return $null }
    $authorization = $details.Value.PSObject.Properties["authorization"]
    if ($null -eq $authorization) { return $null }
    $authorization.Value
}

function Normalize-Sprint8CResponseObservationTimestamps {
    param([Parameter(Mandatory)]$Module)

    $diagnostics = $Module.PSObject.Properties["diagnostics"]
    if ($null -eq $diagnostics -or $null -eq $diagnostics.Value) { return }
    $details = $diagnostics.Value.PSObject.Properties["details"]
    if ($null -eq $details -or $null -eq $details.Value) { return }
    $facts = $details.Value.PSObject.Properties["facts"]
    if ($null -eq $facts -or $null -eq $facts.Value) { return }
    foreach ($property in @($facts.Value.PSObject.Properties)) {
        if ([string]$property.Name -clike "*_last_stable_at") {
            $property.Value = "<observational-last-stable-at>"
        }
    }
}

function Assert-Sprint8CUpgradePreservation {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual,
        [Parameter(Mandatory)][string]$Stage
    )

    $expectedResponse = Copy-Sprint8CJsonValue -Value $Expected.response
    $actualResponse = Copy-Sprint8CJsonValue -Value $Actual.response
    foreach ($snapshot in @($expectedResponse, $actualResponse)) {
        $snapshot.release = "<transition-release>"
        $snapshot.runtime_image = "<transition-image>"
        $snapshot.manifest_digest = "<transition-manifest>"
        $snapshot.executable_sha256 = "<transition-executable>"
        $snapshot.runtime_identity = "<transition-runtime>"
        $snapshot.module.release.id = "<transition-release-id>"
        $snapshot.module.release.version = "<transition-release>"
        $snapshot.module.release.runtime_image = "<transition-image>"
        $snapshot.module.release.manifest_digest = "<transition-manifest>"
        $snapshot.module.manifest = "<transition-manifest-document>"
        $snapshot.module.diagnostics.details.release = "<transition-release>"
        $authorization = Get-Sprint8CUpgradeAuthorizationProjection -Module $snapshot.module
        if ($null -ne $authorization) {
            $authorization.authorization_revision = "<transition-authorization-revision>"
            $authorization.organization_revision = "<transition-organization-revision>"
            $authorization.updated_at = "<transition-authorization-updated-at>"
        }
        Normalize-Sprint8CResponseObservationTimestamps -Module $snapshot.module
    }
    if ((ConvertTo-Sprint8CStableJson $expectedResponse) -cne
        (ConvertTo-Sprint8CStableJson $actualResponse)) {
        throw "$Stage changed Response state, Module Instance identity, typed references, navigation, outbox positions, configuration, or behavior."
    }

    $expectedUnrelated = Copy-Sprint8CJsonValue -Value $Expected.unrelated
    $actualUnrelated = Copy-Sprint8CJsonValue -Value $Actual.unrelated
    foreach ($snapshot in @($expectedUnrelated, $actualUnrelated)) {
        foreach ($module in @($snapshot.modules)) {
            $authorization = Get-Sprint8CUpgradeAuthorizationProjection -Module $module
            if ($null -ne $authorization) {
                $authorization.updated_at = "<transition-authorization-updated-at>"
            }
        }
    }
    if ((ConvertTo-Sprint8CStableJson $expectedUnrelated) -cne
        (ConvertTo-Sprint8CStableJson $actualUnrelated)) {
        throw "$Stage changed an unrelated owner image, container, restart count, data, navigation, or availability."
    }
    [pscustomobject][ordered]@{
        stage = $Stage
        state = "passed"
        response_state = "passed"
        module_instance_identity = "passed"
        typed_resource_identity = "passed"
        navigation_identity = "passed"
        outbox_positions = "passed"
        unrelated_owners = "passed"
    }
}

function ConvertFrom-Sprint8CUpgradeContainerInspection {
    param(
        [Parameter(Mandatory)][string[]]$InspectionOutput,
        [Parameter(Mandatory)][int]$InspectionExitCode,
        [Parameter(Mandatory)][string]$ExpectedId,
        [Parameter(Mandatory)][string]$Service,
        [switch]$AllowStarting
    )
    $inspectionJson = @($InspectionOutput | Where-Object { $_.TrimStart().StartsWith('{') })
    if ($InspectionExitCode -ne 0 -or $inspectionJson.Count -ne 1) {
        throw "Could not inspect exactly one Response upgrade service '$Service'."
    }
    try { $inspection = $inspectionJson[0] | ConvertFrom-Json -Depth 30 } catch {
        throw "Could not decode Response upgrade service '$Service' inspection."
    }
    $healthProperty = $inspection.State.PSObject.Properties['Health']
    $health = if ($null -eq $healthProperty -or $null -eq $healthProperty.Value) { "" } else {
        [string]$healthProperty.Value.Status
    }
    if ([string]$inspection.Id -cne $ExpectedId -or
        [string]$inspection.State.Status -cne "running" -or
        (-not [string]::IsNullOrWhiteSpace($health) -and $health -cne "healthy" -and
            (-not $AllowStarting -or $health -cne "starting"))) {
        throw "Response upgrade service '$Service' is not running and healthy."
    }
    [pscustomobject][ordered]@{
        service = $Service
        container_id = [string]$inspection.Id
        image_id = [string]$inspection.Image
        restart_count = [uint64]$inspection.RestartCount
        state = [string]$inspection.State.Status
        health = $health
    }
}

function Get-Sprint8CUpgradeContainerIdentity {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$Service
    )
    $id = ((Invoke-Sprint8CDockerCompose -ComposePath $ComposePath `
        -Arguments @("ps", "--status", "running", "-q", $Service)).output -join "").Trim()
    if ($id -cnotmatch '^[0-9a-f]{64}$') {
        throw "Response upgrade requires exactly one running '$Service' container."
    }
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds(60)
    do {
        $inspectionOutput = @(& docker inspect --format '{{json .}}' -- $id 2>&1 |
            ForEach-Object { [string]$_ })
        $inspectionExitCode = $LASTEXITCODE
        $identity = ConvertFrom-Sprint8CUpgradeContainerInspection `
            -InspectionOutput $inspectionOutput -InspectionExitCode $inspectionExitCode `
            -ExpectedId $id -Service $Service -AllowStarting
        if ([string]$identity.health -cne "starting") { return $identity }
        if ([DateTimeOffset]::UtcNow -ge $deadline) {
            throw "Response upgrade service '$Service' did not become healthy within 60 seconds."
        }
        Start-Sleep -Seconds 1
    } while ($true)
}

function Invoke-Sprint8CUpgradeRequest {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][Microsoft.PowerShell.Commands.WebRequestSession]$Session,
        [string]$Method = "GET",
        [AllowNull()]$Body,
        [int[]]$ExpectedStatus = @(200)
    )
    $parameters = @{
        Uri = "$BaseUrl$Path"
        Method = $Method
        WebSession = $Session
        UseBasicParsing = $true
        SkipHttpErrorCheck = $true
        TimeoutSec = 60
    }
    if ($null -ne $Body) {
        $parameters.ContentType = "application/json"
        $parameters.Body = if ($Body -is [string]) {
            $Body
        } else {
            $Body | ConvertTo-Json -Depth 100 -Compress
        }
    }
    $response = Invoke-WebRequest @parameters
    if ($ExpectedStatus -notcontains [int]$response.StatusCode) {
        throw "Response upgrade request '$Method $Path' returned HTTP $($response.StatusCode): $($response.Content)"
    }
    $document = if ([string]::IsNullOrWhiteSpace([string]$response.Content)) {
        $null
    } else {
        [string]$response.Content | ConvertFrom-Json -Depth 100
    }
    [pscustomobject][ordered]@{
        status = [int]$response.StatusCode
        content_type = [string]$response.Headers.'Content-Type'
        document = $document
    }
}

function Get-Sprint8CUpgradeFixtureIdentity {
    param([Parameter(Mandatory)]$Fixture)
    if ([int]$Fixture.schema_version -ne 1 -or
        [string]$Fixture.sprint -cne "sprint-8c" -or
        [string]$Fixture.state -cne "passed" -or
        [string]$Fixture.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$Fixture.apply_response_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Fixture.receipt_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
        [string]$Fixture.installation_id -cnotmatch
            '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$' -or
        [string]$Fixture.mutation_policy -cne "owner-apis-and-owner-bootstrap-only" -or
        [string]$Fixture.identity_policy -cne
            "logical-keys-resolve-only-from-signed-owner-receipts-and-typed-read-back" -or
        [string]$Fixture.forbidden_proofs.predicted_uuid -cne "not_used" -or
        [string]$Fixture.forbidden_proofs.foreign_owner_database_write -cne "not_used" -or
        [string]$Fixture.forbidden_proofs.response_sql_mutation -cne "not_used" -or
        [string]$Fixture.forbidden_proofs.copied_count_as_identity -cne "not_used" -or
        [string]$Fixture.restoration.state -cne "passed" -or
        [string]$Fixture.restoration.basis -cne "successful_from_empty_owner_receipts") {
        throw "Response upgrade fixture receipt is not a healthy owner-controlled Sprint 8C receipt."
    }
    $ownerReceiptDigests = @($Fixture.owner_receipt_digests)
    Assert-Sprint8CExactSequence -Expected @(
        "core", "tessara.components", "tessara.dashboards", "tessara.datasets",
        "tessara.reference.scoped-records", "tessara.responses"
    ) -Actual @($ownerReceiptDigests.owner | Sort-Object) `
        -Label "Response upgrade authenticated owner receipts"
    if (@($ownerReceiptDigests | Where-Object {
        [string]$_.input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
        [string]$_.result_digest -cnotmatch '^sha256:[0-9a-f]{64}$'
    }).Count -ne 0) {
        throw "Response upgrade fixture has an invalid owner receipt digest."
    }

    $expectedResponseKeys = @(
        "response.draft.owner", "response.submitted.owner", "response.submitted.delegated",
        "response.submitted.restricted"
    )
    $responseProperties = @($Fixture.logical_identities.responses.PSObject.Properties)
    Assert-Sprint8CExactSequence -Expected @($expectedResponseKeys | Sort-Object) `
        -Actual @($responseProperties.Name | Sort-Object) -Label "Response fixture logical keys"
    $moduleInstanceIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $installationIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $responses = [ordered]@{}
    $submittedCount = 0
    foreach ($key in $expectedResponseKeys) {
        $identity = $Fixture.logical_identities.responses.PSObject.Properties[$key].Value
        $reference = $identity.reference.reference
        $readBack = $identity.read_back
        $expectedLifecycle = if ($key -ceq "response.draft.owner") { "draft" } else {
            "submitted"
        }
        if ([string]$identity.response_id -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$' -or
            [string]$identity.provenance -cne "signed-tessara.responses-bootstrap-receipt" -or
            [string]$reference.installation_id -cne [string]$Fixture.installation_id -or
            [string]$reference.resource_type -cne "tessara.responses.response" -or
            [string]$reference.resource_id -cne [string]$identity.response_id -or
            [string]$reference.owner.kind -cne "module_instance" -or
            [string]$reference.owner.installation_id -cne [string]$reference.installation_id -or
            [string]::IsNullOrWhiteSpace([string]$reference.owner.module_instance_id) -or
            [int]$readBack.schema_version -ne 1 -or
            (ConvertTo-Sprint8CStableJson $readBack.response) -cne
                (ConvertTo-Sprint8CStableJson $identity.reference) -or
            [string]$readBack.lifecycle_state -cne $expectedLifecycle -or
            [uint64]$readBack.revision -lt 1 -or
            [uint64]$readBack.workflow_event_sequence -lt 1) {
            throw "Response bootstrap fixture '$key' lacks exact signed typed owner read-back."
        }
        foreach ($idName in @(
            "workflow_assignment_id", "workflow_instance_id", "workflow_step_instance_id"
        )) {
            if ([string]$readBack.$idName -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
                throw "Response bootstrap fixture '$key' has an invalid '$idName'."
            }
        }
        [void]$moduleInstanceIds.Add([string]$reference.owner.module_instance_id)
        [void]$installationIds.Add([string]$reference.installation_id)
        if ($expectedLifecycle -ceq "submitted") { $submittedCount++ }
        $responses[$key] = $identity
    }
    if ($moduleInstanceIds.Count -ne 1 -or $installationIds.Count -ne 1) {
        throw "Response bootstrap fixtures do not identify exactly one installation and Module Instance."
    }

    $datasets = [ordered]@{}
    foreach ($key in @(
        "dataset.base", "dataset.derived", "dataset.derived-second-hop",
        "dataset.independent-binding", "dataset.disjoint-binding"
    )) {
        $property = $Fixture.logical_identities.datasets.PSObject.Properties[$key]
        if ($null -eq $property -or
            [string]$property.Value.dataset.reference.resource_type -cne
                "tessara.datasets.dataset" -or
            [string]$property.Value.revision.reference.resource_type -cne
                "tessara.datasets.dataset_revision" -or
            [string]$property.Value.dataset.reference.owner.kind -cne "module_instance") {
            throw "Response upgrade fixture omits unrelated Dataset typed read-back for '$key'."
        }
        $datasets[$key] = $property.Value
    }
    [pscustomobject][ordered]@{
        installation_id = [string]$Fixture.installation_id
        response_module_instance_id = @($moduleInstanceIds)[0]
        responses = [pscustomobject]$responses
        response_export_minimum = [uint64]$submittedCount
        datasets = [pscustomobject]$datasets
        components = $Fixture.logical_identities.components
        dashboard = $Fixture.logical_identities.dashboard
    }
}

function Get-Sprint8CUpgradeExecutableSha256 {
    param([Parameter(Mandatory)][string]$ContainerId)
    $output = @(& docker exec $ContainerId sha256sum $responseExecutable 2>&1)
    if ($LASTEXITCODE -ne 0 -or $output.Count -ne 1 -or
        [string]$output[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
        throw "Could not capture the live Response executable identity."
    }
    $Matches.hash
}

function Get-Sprint8CUpgradeOwnerHealth {
    param([Parameter(Mandatory)][string]$ContainerId)

    $marker = "__TESSARA_RESPONSE_HEALTH_META__"
    $output = @(& docker exec $ContainerId curl --silent --show-error `
        --max-time 10 --header "Accept: application/json" `
        --write-out "$marker%{http_code}|%{content_type}|%{num_redirects}" `
        http://127.0.0.1:8094/health/ready 2>&1)
    $joined = $output -join "`n"
    $parts = @($joined -split [regex]::Escape($marker))
    if ($LASTEXITCODE -ne 0 -or $parts.Count -ne 2) {
        throw "Could not capture the live Response owner readiness contract."
    }
    $metadata = @([string]$parts[1] -split '\|')
    if ($metadata.Count -ne 3 -or [string]$metadata[0] -cne "200" -or
        [string]$metadata[1] -cne "application/json" -or
        [string]$metadata[2] -cne "0") {
        throw "Response owner readiness did not return exact 200/application-json/no-redirect semantics."
    }
    try { $health = [string]$parts[0] | ConvertFrom-Json -Depth 30 } catch {
        throw "Response owner readiness returned invalid JSON."
    }
    Assert-Sprint8CExactSequence -Expected @(
        "response.configuration", "response.database", "response.events.publication",
        "response.export.publication", "response.provider.forms",
        "response.provider.workflow", "response.security_state"
    ) -Actual @($health.checks.code | Sort-Object) -Label "Response owner readiness checks"
    if ([int]$health.schema_version -ne 1 -or [string]$health.status -cne "passing" -or
        @($health.checks | Where-Object { -not [bool]$_.passing }).Count -ne 0) {
        throw "Response owner readiness is not exactly passing."
    }
    $health
}

function Get-Sprint8CUpgradeServiceSnapshot {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string[]]$Services
    )
    @($Services | ForEach-Object {
        Get-Sprint8CUpgradeContainerIdentity -ComposePath $ComposePath -Service $_
    })
}

function Get-Sprint8CUpgradeImageIdentity {
    param(
        [Parameter(Mandatory)][string]$Image,
        [Parameter(Mandatory)]$ExpectedSource,
        [Parameter(Mandatory)][string]$ExpectedRelease,
        [AllowEmptyString()][string]$ExpectedFixtureKind = "",
        [AllowNull()]$ExpectedBaselineSource = $null,
        [AllowNull()]$ExpectedBuilderSource = $null
    )
    $inspection = @(& docker image inspect $Image | ConvertFrom-Json -Depth 100)
    if ($LASTEXITCODE -ne 0 -or $inspection.Count -ne 1) {
        throw "Could not inspect exact Response release image '$Image'."
    }
    $identity = $inspection[0]
    $labels = $identity.Config.Labels
    if ([string]$identity.Id -cne $Image -or
        [string]$labels.'com.tessara.module-definition' -cne $responseDefinition -or
        [string]$labels.'com.tessara.module-release' -cne $ExpectedRelease -or
        [string]$labels.'org.opencontainers.image.revision' -cne [string]$ExpectedSource.commit -or
        [string]$labels.'com.tessara.source-tree' -cne [string]$ExpectedSource.tree -or
        [string]$labels.'com.tessara.source-dirty' -cne 'false') {
        throw "Response release image '$Image' is not source-exact for '$ExpectedRelease'."
    }
    $fixtureProperty = $labels.PSObject.Properties['com.tessara.upgrade.fixture']
    $fixtureLabel = if ($null -eq $fixtureProperty) { "" } else {
        [string]$fixtureProperty.Value
    }
    if ([string]::IsNullOrEmpty($ExpectedFixtureKind)) {
        if (-not [string]::IsNullOrEmpty($fixtureLabel)) {
            throw "Response candidate image is mislabeled as an upgrade fixture."
        }
    } elseif ($fixtureLabel -cne $ExpectedFixtureKind) {
        throw "Response baseline image omits its exact source-built fixture identity."
    }
    if ($null -ne $ExpectedBaselineSource) {
        if ([string]$labels.'com.tessara.baseline-patch-sha256' -cne
                [string]$ExpectedBaselineSource.patch_sha256 -or
            [string]$labels.'com.tessara.materialized-source-sha256' -cne
                [string]$ExpectedBaselineSource.materialized_source_sha256 -or
            $null -eq $ExpectedBuilderSource -or
            [string]$labels.'com.tessara.builder-source-commit' -cne
                [string]$ExpectedBuilderSource.commit -or
            [string]$labels.'com.tessara.builder-source-tree' -cne
                [string]$ExpectedBuilderSource.tree) {
            throw "Response baseline image omits authenticated materialized-source provenance."
        }
    }
    [pscustomobject][ordered]@{
        image_id = [string]$identity.Id
        definition_id = [string]$labels.'com.tessara.module-definition'
        release = [string]$labels.'com.tessara.module-release'
        source_commit = [string]$labels.'org.opencontainers.image.revision'
        source_tree = [string]$labels.'com.tessara.source-tree'
        source_dirty = [string]$labels.'com.tessara.source-dirty'
        fixture_kind = if ([string]::IsNullOrEmpty($fixtureLabel)) { $null } else { $fixtureLabel }
        baseline_patch_sha256 = if ($null -eq $ExpectedBaselineSource) { $null } else {
            [string]$labels.'com.tessara.baseline-patch-sha256'
        }
        materialized_source_sha256 = if ($null -eq $ExpectedBaselineSource) { $null } else {
            [string]$labels.'com.tessara.materialized-source-sha256'
        }
    }
}

function Get-Sprint8CUpgradeModuleProjection {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][Microsoft.PowerShell.Commands.WebRequestSession]$Session,
        [Parameter(Mandatory)][string]$DefinitionId
    )
    $module = (Invoke-Sprint8CUpgradeRequest -BaseUrl $BaseUrl `
        -Path "/api/admin/modules/$DefinitionId" -Session $Session).document
    [pscustomobject][ordered]@{
        kind = [string]$module.entry.kind
        definition = $module.entry.definition
        release = $module.entry.release
        instance = [pscustomobject][ordered]@{
            id = [string]$module.entry.instance.id
            identity = [string]$module.entry.instance.identity
            data = [string]$module.entry.instance.data
            database_name = [string]$module.entry.instance.database_name
            installed = [bool]$module.entry.instance.installed
            deployed = [bool]$module.entry.instance.deployed
            configured = [bool]$module.entry.instance.configured
            ready = [bool]$module.entry.instance.ready
            enabled = [bool]$module.entry.instance.enabled
            healthy = [bool]$module.entry.instance.healthy
        }
        configuration = $module.entry.configuration
        diagnostics = $module.entry.diagnostics
        manifest = $module.entry.manifest
        findings = @($module.entry.findings)
    }
}

function Get-Sprint8CUpgradeTypedResponseObservations {
    $observations = [Collections.Generic.List[object]]::new()
    $authorizedCount = 0
    foreach ($property in @($script:upgradeFixture.responses.PSObject.Properties | Sort-Object Name)) {
        $key = [string]$property.Name
        $identity = $property.Value
        $referenceProperty = $identity.PSObject.Properties['reference']
        $reference = if ($null -ne $referenceProperty -and $null -ne $referenceProperty.Value) {
            $referenceProperty.Value.reference
        } else {
            [pscustomobject][ordered]@{
                installation_id = [string]$script:upgradeFixture.installation_id
                owner = [pscustomobject][ordered]@{
                    kind = "module_instance"
                    installation_id = [string]$script:upgradeFixture.installation_id
                    module_instance_id = [string]$script:upgradeFixture.response_module_instance_id
                }
                resource_type = "tessara.responses.response"
                resource_id = [string]$identity.response_id
            }
        }
        if ([string]$reference.installation_id -cne [string]$script:upgradeFixture.installation_id -or
            [string]$reference.owner.kind -cne "module_instance" -or
            [string]$reference.owner.installation_id -cne [string]$reference.installation_id -or
            [string]$reference.owner.module_instance_id -cne
                [string]$script:upgradeFixture.response_module_instance_id -or
            [string]$reference.resource_type -cne "tessara.responses.response" -or
            [string]$reference.resource_id -cne [string]$identity.response_id) {
            throw "Response fixture '$key' does not identify the exact independent owner."
        }
        $result = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
            -Path "/api/platform/resource-observations/resolve" `
            -Session $script:upgradeSession -Method POST -Body ([ordered]@{
                schema_version = 1
                reference = $reference
            })).document
        if ([int]$result.schema_version -ne 1 -or $null -eq $result.resolution) {
            throw "Response typed observation '$key' returned an invalid platform envelope."
        }
        $resolution = $result.resolution
        $observationProperty = $result.PSObject.Properties['observation']
        $observation = if ($null -eq $observationProperty) { $null } else {
            $observationProperty.Value
        }
        if ([string]$resolution.access_state -ceq "authorized") {
            if ([string]$resolution.owner_state.kind -cne "module_instance" -or
                [string]$resolution.owner_state.instance_state -cne "live" -or
                [string]$resolution.owner_state.data_state -cne "retained" -or
                [string]$resolution.resource_identity_state -cne "resolved" -or
                [string]$resolution.resource_lifecycle_state.kind -cne "provider_defined" -or
                [string]$resolution.compatibility_state -cne "compatible" -or
                [string]$resolution.availability_state -cne "available" -or
                $null -eq $observation -or
                [string]$observation.reference.installation_id -cne [string]$reference.installation_id -or
                [string]$observation.reference.owner.kind -cne "module_instance" -or
                [string]$observation.reference.owner.module_instance_id -cne
                    [string]$reference.owner.module_instance_id -or
                [string]$observation.reference.resource_type -cne [string]$reference.resource_type -or
                [string]$observation.reference.resource_id -cne [string]$reference.resource_id -or
                [string]$observation.provider_contract.contract_id -cne
                    "tessara.responses.response" -or
                [string]$observation.provider_contract.contract_version -cne "2.0.0" -or
                [string]$observation.strategy -cne "live_resolution_with_revision" -or
                [uint64]$observation.resource_revision -lt 1) {
                throw "Response typed observation '$key' did not resolve through the live independent owner."
            }
            $authorizedCount++
        } elseif ([string]$resolution.access_state -notin @("unauthorized", "not_evaluated") -or
            $null -ne $observation) {
            throw "Response typed observation '$key' did not fail closed."
        }
        $observations.Add([pscustomobject][ordered]@{
            key = $key
            reference = $reference
            resolution = $resolution
            observation = $observation
        })
    }
    if ($authorizedCount -lt 1) {
        throw "Response typed identity proof did not resolve any live Response resource."
    }
    @($observations)
}

function Get-Sprint8CUpgradeStageSnapshot {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$ExpectedRelease,
        [Parameter(Mandatory)][string]$ExpectedImage,
        [Parameter(Mandatory)][string]$ExpectedManifestDigest
    )
    Wait-Sprint8CHttpProbe -Uri "$($script:upgradePorts.gateway_url)/health" | Out-Null
    Wait-Sprint8CHttpProbe -Uri "$($script:upgradePorts.supervisor_url)/health/ready" `
        -ExpectedStatus @(204) | Out-Null
    $runtime = Get-Sprint8CUpgradeContainerIdentity -ComposePath $script:upgradeComposePath `
        -Service "responses"
    if ([string]$runtime.image_id -cne $ExpectedImage) {
        throw "$Stage Response runtime image '$($runtime.image_id)' is not '$ExpectedImage'."
    }
    $module = Get-Sprint8CUpgradeModuleProjection -BaseUrl $script:upgradePorts.gateway_url `
        -Session $script:upgradeSession -DefinitionId $responseDefinition
    if ([string]$module.kind -cne "independently_deployed" -or
        [string]$module.definition.id -cne $responseDefinition -or
        [string]$module.release.version -cne $ExpectedRelease -or
        [string]$module.release.runtime_image -cne $ExpectedImage -or
        [string]$module.release.manifest_digest -cne $ExpectedManifestDigest -or
        [string]$module.instance.id -cne $script:upgradeFixture.response_module_instance_id -or
        [string]$module.diagnostics.public_route -cne "/responses" -or
        [string]$module.diagnostics.details.release -cne $ExpectedRelease -or
        [string]$module.manifest.release_version -cne $ExpectedRelease -or
        -not [bool]$module.instance.ready -or -not [bool]$module.instance.healthy) {
        throw "$Stage Module Management read-back does not identify the expected independent healthy Response release."
    }

    $responseList = @((Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/responses" -Session $script:upgradeSession).document | Sort-Object id)
    $responseDetails = @($script:upgradeFixture.responses.PSObject.Properties | ForEach-Object {
        $key = [string]$_.Name
        $id = [string]$_.Value.response_id
        $result = Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
            -Path "/api/responses/$id" -Session $script:upgradeSession `
            -ExpectedStatus @(200, 404)
        [pscustomobject][ordered]@{
            key = $key
            response_id = $id
            status = [int]$result.status
            document = $result.document
        }
    } | Sort-Object key)
    $navigation = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/shell/navigation" -Session $script:upgradeSession).document
    $responseNavigation = @($navigation.groups | ForEach-Object { $_.items } | Where-Object {
        [string]$_.contribution_id -ceq "tessara.responses.navigation"
    })
    if ($responseNavigation.Count -ne 1 -or
        [string]$responseNavigation[0].href -cne "/responses") {
        throw "$Stage Response navigation identity is missing or substituted."
    }
    $summary = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/summary" -Session $script:upgradeSession).document
    $operations = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/operations/status" -Session $script:upgradeSession).document
    if ([string]$summary.response_state -notin @("available", "empty") -or
        [string]$operations.response_owner.state -cne "available" -or
        $null -eq $operations.response_owner.status -or
        [uint64]$operations.response_owner.status.export_head_sequence -lt
            [uint64]$script:upgradeFixture.response_export_minimum) {
        throw "$Stage Response summary or outbox status is unavailable or behind the prepared fixture."
    }

    $datasetDetails = @($script:upgradeFixture.datasets.PSObject.Properties | ForEach-Object {
        $key = [string]$_.Name
        $id = [string]$_.Value.dataset.reference.resource_id
        [pscustomobject][ordered]@{
            key = $key
            detail = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
                -Path "/api/datasets/$id" -Session $script:upgradeSession).document
            table = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
                -Path "/api/datasets/$id/table" -Session $script:upgradeSession).document
        }
    } | Sort-Object key)
    $datasetList = @((Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/datasets" -Session $script:upgradeSession).document | Sort-Object dataset_id)
    $componentList = @((Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/components" -Session $script:upgradeSession).document | Sort-Object component_id)
    $dashboardList = @((Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/dashboards" -Session $script:upgradeSession).document | Sort-Object id)
    $dashboardDetail = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/dashboards/$([string]$script:upgradeFixture.dashboard.id)" `
        -Session $script:upgradeSession).document
    $forms = @((Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/forms" -Session $script:upgradeSession).document | Sort-Object id)
    $workflows = @((Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/workflows" -Session $script:upgradeSession).document | Sort-Object id)
    $unrelatedModules = @(
        "tessara.datasets", "tessara.components", "tessara.dashboards",
        "tessara.reference.scoped-records" | ForEach-Object {
            Get-Sprint8CUpgradeModuleProjection -BaseUrl $script:upgradePorts.gateway_url `
                -Session $script:upgradeSession -DefinitionId $_
        }
    )

    [pscustomobject][ordered]@{
        stage = $Stage
        captured_at = [DateTimeOffset]::UtcNow.ToString("o")
        response = [pscustomobject][ordered]@{
            release = $ExpectedRelease
            runtime_image = $ExpectedImage
            manifest_digest = $ExpectedManifestDigest
            executable_sha256 = Get-Sprint8CUpgradeExecutableSha256 `
                -ContainerId ([string]$runtime.container_id)
            owner_health = Get-Sprint8CUpgradeOwnerHealth `
                -ContainerId ([string]$runtime.container_id)
            runtime_identity = $runtime
            module = $module
            typed_resource_identity = Get-Sprint8CUpgradeTypedResponseObservations
            navigation = $responseNavigation[0]
            product_list = $responseList
            product_details = $responseDetails
            summary = [pscustomobject][ordered]@{
                state = [string]$summary.response_state
                draft_count = $summary.draft_submissions
                submitted_count = $summary.submitted_submissions
            }
            outbox_positions = $operations.response_owner.status
        }
        unrelated = [pscustomobject][ordered]@{
            services = Get-Sprint8CUpgradeServiceSnapshot -ComposePath $script:upgradeComposePath `
                -Services @(
                    "postgres", "core", "supervisor", "datasets", "components", "dashboards",
                    "scoped-records", "response-provider-proxy", "form-provider-proxy",
                    "scope-provider-proxy", "principal-provider-proxy", "gateway"
                )
            modules = $unrelatedModules
            dataset_product = [pscustomobject][ordered]@{
                list = $datasetList
                details_and_tables = $datasetDetails
            }
            component_product = $componentList
            dashboard_product = [pscustomobject][ordered]@{
                list = $dashboardList
                detail = $dashboardDetail
            }
            forms = $forms
            workflows = $workflows
            navigation = $navigation
            operations = $operations
            summary = $summary
            availability = [pscustomobject][ordered]@{
                gateway = "healthy"
                supervisor = "healthy"
            }
        }
    }
}

function New-Sprint8CUpgradeExerciseCatalog {
    param(
        [Parameter(Mandatory)]$RuntimeCatalog,
        [Parameter(Mandatory)]$InitialLockfile,
        [Parameter(Mandatory)]$BaselineMetadata,
        [Parameter(Mandatory)][string]$CandidateManifestDigest,
        [Parameter(Mandatory)][string]$CandidateImageDigest
    )
    $catalog = Copy-Sprint8CJsonValue -Value $RuntimeCatalog
    $catalog.revision = [uint64]$InitialLockfile.blueprint_revision + 1000
    $catalog.issued_at = [DateTimeOffset]::UtcNow.ToString("o")
    $responseTemplates = @($catalog.module_releases | Where-Object {
        [string]$_.definition_id -ceq $responseDefinition
    })
    if ($responseTemplates.Count -ne 1) {
        throw "Runtime release catalog must contain exactly one candidate Response release."
    }
    $lockedResponse = @($InitialLockfile.modules | Where-Object {
        [string]$_.definition_id -ceq $responseDefinition
    })
    if ($lockedResponse.Count -ne 1 -or
        [string]$responseTemplates[0].version -cne [string]$lockedResponse[0].version -or
        [string]$responseTemplates[0].manifest_digest -cne
            [string]$lockedResponse[0].manifest_digest -or
        [string]$responseTemplates[0].runtime_image -cne
            [string]$lockedResponse[0].runtime_image) {
        throw "Signed runtime catalog Response candidate is not bound to the initial lockfile."
    }
    $candidate = Copy-Sprint8CJsonValue -Value $responseTemplates[0]
    $candidate.version = $candidateRelease
    $candidate.manifest_digest = $CandidateManifestDigest
    $candidate.runtime_image = $CandidateImageDigest
    $baseline = Copy-Sprint8CJsonValue -Value $candidate
    $baseline.version = $baselineRelease
    $baseline.manifest_digest = [string]$BaselineMetadata.release_identity.manifest_digest
    $baseline.runtime_image = [string]$BaselineMetadata.release_identity.runtime_image

    $otherReleases = @($catalog.module_releases | Where-Object {
        [string]$_.definition_id -cne $responseDefinition
    })
    foreach ($release in $otherReleases) {
        $locked = @($InitialLockfile.modules | Where-Object {
            [string]$_.definition_id -ceq [string]$release.definition_id
        })
        if ($locked.Count -ne 1 -or
            [string]$release.version -cne [string]$locked[0].version -or
            [string]$release.manifest_digest -cne [string]$locked[0].manifest_digest -or
            [string]$release.runtime_image -cne [string]$locked[0].runtime_image) {
            throw "Runtime catalog owner '$($release.definition_id)' is not fixed to the initial lockfile."
        }
    }
    $lockedDefinitions = @($InitialLockfile.modules.definition_id | Sort-Object)
    $catalogDefinitions = @($catalog.module_releases.definition_id | Sort-Object)
    if (($lockedDefinitions -join "`n") -cne ($catalogDefinitions -join "`n")) {
        throw "Runtime catalog and initial lockfile do not contain the same module owner set."
    }
    $catalog.module_releases = @($candidate, $baseline) + $otherReleases
    $catalog
}

function Select-Sprint8CBoundRuntimeCatalog {
    param(
        [Parameter(Mandatory)][object[]]$Candidates,
        [Parameter(Mandatory)][string]$ExpectedCatalogDigest
    )
    $matches = @($Candidates | Where-Object {
        [string]$_.catalog_digest -ceq $ExpectedCatalogDigest
    })
    if ($matches.Count -eq 0) {
        throw "Materialization runtime catalog is not bound to the current resolved composition."
    }
    $expectedPayload = ConvertTo-Sprint8CStableJson $matches[0].payload
    foreach ($match in @($matches | Select-Object -Skip 1)) {
        if ((ConvertTo-Sprint8CStableJson $match.payload) -cne $expectedPayload) {
            throw "Materialization runtime catalogs collide on one digest with different payloads."
        }
    }
    $matches[0]
}

function Resolve-Sprint8CBoundRuntimeCatalog {
    param(
        [Parameter(Mandatory)][string]$MaterializationRoot,
        [Parameter(Mandatory)][string]$ExpectedCatalogDigest
    )
    $catalogKeyPath = Resolve-Sprint8CRepositoryPath -Path `
        "deploy/sprint-8c/catalogs/catalog-dev-v1.public.hex"
    $candidates = [Collections.Generic.List[object]]::new()
    foreach ($stage in @("noop", "first")) {
        $catalogPath = Join-Path $MaterializationRoot "$stage/release-catalog.signed.json"
        if (-not (Test-Path -LiteralPath $catalogPath -PathType Leaf)) { continue }
        $digestOutput = @(& cargo run --quiet --locked --offline `
            -p tessara-supervisor --bin tessara-compose -- `
            catalog-verify $catalogPath $catalogKeyPath 2>&1)
        if ($LASTEXITCODE -ne 0) {
            throw "Materialization runtime release catalog '$stage' failed signature and contract verification."
        }
        $digest = [string]($digestOutput | Select-Object -Last 1)
        if ($digest -cnotmatch '^sha256:[0-9a-f]{64}$') {
            throw "Materialization runtime release catalog '$stage' did not emit a canonical digest."
        }
        $envelope = Get-Content -LiteralPath $catalogPath -Raw | ConvertFrom-Json -Depth 100
        $candidates.Add([pscustomobject][ordered]@{
            stage = $stage
            path = $catalogPath
            catalog_digest = $digest
            payload = $envelope.payload
        })
    }
    if ($candidates.Count -eq 0) {
        throw "Response upgrade cannot locate a signed materialization runtime release catalog."
    }
    Select-Sprint8CBoundRuntimeCatalog -Candidates @($candidates) `
        -ExpectedCatalogDigest $ExpectedCatalogDigest
}

function Invoke-Sprint8CResponseTransition {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$TargetRelease,
        [Parameter(Mandatory)][string]$TargetImage,
        [Parameter(Mandatory)][string]$TargetManifestDigest
    )
    $summary = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition" -Session $script:upgradeSession).document
    $blueprint = Copy-Sprint8CJsonValue -Value $summary.latest_blueprint
    $blueprint.revision = [uint64]$summary.latest_blueprint.revision + 1
    $selection = @($blueprint.modules | Where-Object {
        [string]$_.definition_id -ceq $responseDefinition
    })
    if ($selection.Count -ne 1) {
        throw "$Stage Blueprint does not contain exactly one Response selection."
    }
    $selection[0].version_requirement = "=$TargetRelease"
    Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints" -Session $script:upgradeSession `
        -Method POST -Body $blueprint -ExpectedStatus @(200, 201) | Out-Null
    $resolved = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints/$($blueprint.revision)/resolve" `
        -Session $script:upgradeSession -Method POST `
        -Body ([ordered]@{ catalog = $script:upgradeCatalog })).document
    $resolvedResponse = @($resolved.lockfile.modules | Where-Object {
        [string]$_.definition_id -ceq $responseDefinition
    })
    if ($resolvedResponse.Count -ne 1 -or
        [string]$resolvedResponse[0].version -cne $TargetRelease -or
        [string]$resolvedResponse[0].runtime_image -cne $TargetImage -or
        [string]$resolvedResponse[0].manifest_digest -cne $TargetManifestDigest) {
        throw "$Stage resolver substituted the intended source-authenticated Response release."
    }
    $fixedProjection = Get-Sprint8CFixedLockfileProjection -Lockfile $resolved.lockfile
    if ((ConvertTo-Sprint8CStableJson $fixedProjection) -cne
        $script:upgradeInitialFixedProjection) {
        throw "$Stage changed a fixed Core or unrelated Module lockfile projection."
    }
    $delta = Assert-Sprint8CExactResponseDeltaPlan -Lockfile $resolved.lockfile `
        -ExpectedImageDigest $TargetImage
    Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints/$($blueprint.revision)/approve" `
        -Session $script:upgradeSession -Method POST -Body ([ordered]@{
            approved_effects = @("install", "upgrade")
            reason = "Sprint 8C Response-only $Stage"
        }) | Out-Null
    $applied = (Invoke-Sprint8CUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints/$($blueprint.revision)/apply" `
        -Session $script:upgradeSession -Method POST -Body ([ordered]@{}) `
        -ExpectedStatus @(200, 202)).document
    $artifact = $applied.receipt.observed_artifacts.PSObject.Properties[$responseDefinition]
    if ([string]$applied.operation.state -cne "succeeded" -or
        $null -eq $artifact -or [string]$artifact.Value -cne $TargetImage -or
        [string]$applied.receipt.lockfile_digest -cne [string]$resolved.lockfile_digest -or
        (ConvertTo-Sprint8CStableJson @($applied.receipt.bootstrap_receipts | Sort-Object owner)) -cne
            $script:upgradeInitialBootstrapReceipts) {
        throw "$Stage apply did not bind the intended Response artifact and preserve bootstrap receipts."
    }
    [pscustomobject][ordered]@{
        stage = $Stage
        blueprint_revision = [uint64]$blueprint.revision
        target_release = $TargetRelease
        target_image = $TargetImage
        target_manifest_digest = $TargetManifestDigest
        plan_digest = [string]$resolved.plan_digest
        lockfile_digest = [string]$resolved.lockfile_digest
        exact_delta = $delta
        operation_id = [string]$applied.operation.operation_id
        receipt_revision = [uint64]$applied.receipt.revision
        receipt_lockfile_digest = [string]$applied.receipt.lockfile_digest
        fixed_owner_lockfile = "passed"
    }
}

function Assert-Sprint8CResponseUpgradeReceipt {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$Contract
    )
    $transitions = @($Document.transitions)
    $snapshots = @($Document.stage_snapshots)
    $proofs = @($Document.preservation_proofs)
    if ([int]$Document.schema_version -ne 1 -or
        [string]$Document.sprint -cne "sprint-8c" -or
        [string]$Document.proof -cne "independent-response-upgrade-rollback-restoration" -or
        [string]$Document.state -cne "passed" -or
        [string]$Document.compose_project -cnotmatch '^[a-z0-9][a-z0-9_-]*$' -or
        [string]$Document.source.commit -cnotmatch '^[0-9a-f]{40,64}$' -or
        [string]$Document.source.tree -cnotmatch '^[0-9a-f]{40,64}$' -or
        [bool]$Document.source.dirty -or
        $null -ne $Document.failure -or
        [string]$Document.pre_exercise_snapshot.stage -cne "pre-exercise-candidate" -or
        [string]$Document.pre_exercise_snapshot.response.release -cne $candidateRelease -or
        $transitions.Count -ne 4 -or $snapshots.Count -ne 4 -or $proofs.Count -ne 4) {
        throw "Response upgrade receipt does not contain one pre-exercise snapshot and four passing transition records."
    }
    Assert-Sprint8CExactSequence -Expected @($Contract.sequence) `
        -Actual @($transitions.target_release) -Label "receipt transition releases"
    Assert-Sprint8CExactSequence -Expected @($Contract.stages) `
        -Actual @($transitions.stage) -Label "receipt transition stages"
    Assert-Sprint8CExactSequence -Expected @($Contract.stages) `
        -Actual @($snapshots.stage) -Label "receipt snapshot stages"
    Assert-Sprint8CExactSequence -Expected @($Contract.stages) `
        -Actual @($proofs.stage) -Label "receipt preservation stages"
    Assert-Sprint8CExactSequence -Expected @($Contract.sequence) `
        -Actual @($snapshots.response.release) -Label "receipt snapshot releases"
    Assert-Sprint8CExactSequence -Expected $allowedBaselineManifestVariations `
        -Actual @($Document.release_fixture.baseline.compatibility.allowed_manifest_variations) `
        -Label "receipt baseline allowed Manifest variations"
    $baselineSource = $Document.release_fixture.baseline.source_identity
    if (@($transitions | Where-Object {
        [string]$_.fixed_owner_lockfile -cne "passed"
    }).Count -ne 0 -or @($proofs | Where-Object {
        [string]$_.state -cne "passed" -or
        [string]$_.response_state -cne "passed" -or
        [string]$_.module_instance_identity -cne "passed" -or
        [string]$_.typed_resource_identity -cne "passed" -or
        [string]$_.navigation_identity -cne "passed" -or
        [string]$_.outbox_positions -cne "passed" -or
        [string]$_.unrelated_owners -cne "passed"
    }).Count -ne 0 -or
        [string]$Document.release_restoration.state -cne "passed" -or
        [string]$Document.release_restoration.final_release -cne $candidateRelease -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.release_fixture.baseline.version -cne $baselineRelease -or
        [string]$Document.release_fixture.candidate.version -cne $candidateRelease -or
        [string]$Document.release_fixture.baseline.runtime_image -ceq
            [string]$Document.release_fixture.candidate.runtime_image -or
        [string]$Document.release_fixture.baseline.executable_sha256 -ceq
            [string]$Document.release_fixture.candidate.executable_sha256 -or
        [string]$baselineSource.source_model -cne $baselineSourceModel -or
        [string]$baselineSource.candidate_source_dependency -cne "none" -or
        [string]$baselineSource.commit -cne $baselineSourceCommit -or
        [string]$baselineSource.commit -ceq [string]$Document.source.commit -or
        [string]$baselineSource.tree -cne $baselineSourceTree -or
        [string]$baselineSource.patch_sha256 -cne
            [string]$Contract.prior_compatible_artifact.patch_sha256 -or
        [string]$baselineSource.materialized_source_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [int]$baselineSource.materialized_file_count -le 0 -or
        [string]$baselineSource.marker_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Document.release_fixture.baseline.compatibility.state -cne "passed" -or
        [string]$Document.release_fixture.baseline.compatibility.manifest_projection -cne
            $baselineManifestProjection) {
        throw "Response upgrade receipt does not prove exact preservation, distinct artifacts, and intended restoration."
    }
    $true
}

function New-Sprint8CSelfTestFixtureReceipt {
    $installationId = "01980000-0000-7000-8000-000000000001"
    $responseOwner = "01980000-0000-7000-8000-000000000083"
    $responseKeys = @(
        "response.draft.owner", "response.submitted.owner", "response.submitted.delegated",
        "response.submitted.restricted"
    )
    $responses = [ordered]@{}
    for ($index = 0; $index -lt $responseKeys.Count; $index++) {
        $key = $responseKeys[$index]
        $responseId = "01980000-0019-7000-8000-{0:d12}" -f ($index + 1)
        $reference = [pscustomobject][ordered]@{
            reference = [pscustomobject][ordered]@{
                installation_id = $installationId
                owner = [pscustomobject][ordered]@{
                    kind = "module_instance"
                    installation_id = $installationId
                    module_instance_id = $responseOwner
                }
                resource_type = "tessara.responses.response"
                resource_id = $responseId
            }
        }
        $lifecycle = if ($index -eq 0) { "draft" } else { "submitted" }
        $responses[$key] = [pscustomobject][ordered]@{
            response_id = $responseId
            reference = $reference
            read_back = [pscustomobject][ordered]@{
                schema_version = 1
                response = $reference
                lifecycle_state = $lifecycle
                revision = if ($lifecycle -ceq "draft") { 1 } else { 2 }
                workflow_assignment_id = "01980000-001a-7000-8000-{0:d12}" -f ($index + 1)
                workflow_instance_id = "01980000-001b-7000-8000-{0:d12}" -f ($index + 1)
                workflow_step_instance_id = "01980000-001c-7000-8000-{0:d12}" -f ($index + 1)
                workflow_event_sequence = if ($lifecycle -ceq "draft") { 1 } else { 2 }
            }
            provenance = "signed-tessara.responses-bootstrap-receipt"
        }
    }
    $datasetOwner = "01980000-0000-7000-8000-000000000084"
    $datasets = [ordered]@{}
    $datasetKeys = @(
        "dataset.base", "dataset.derived", "dataset.derived-second-hop",
        "dataset.independent-binding", "dataset.disjoint-binding"
    )
    for ($index = 0; $index -lt $datasetKeys.Count; $index++) {
        $owner = [pscustomobject][ordered]@{
            kind = "module_instance"
            installation_id = $installationId
            module_instance_id = $datasetOwner
        }
        $datasets[$datasetKeys[$index]] = [pscustomobject][ordered]@{
            dataset = [pscustomobject][ordered]@{
                reference = [pscustomobject][ordered]@{
                    installation_id = $installationId; owner = $owner
                    resource_type = "tessara.datasets.dataset"
                    resource_id = "01980000-0001-7000-8000-{0:d12}" -f ($index + 1)
                }
            }
            revision = [pscustomobject][ordered]@{
                reference = [pscustomobject][ordered]@{
                    installation_id = $installationId; owner = $owner
                    resource_type = "tessara.datasets.dataset_revision"
                    resource_id = "01980000-0002-7000-8000-{0:d12}" -f ($index + 1)
                }
            }
        }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "owner-controlled-uat-fixture-preparation"
        state = "passed"
        apply_response_sha256 = "a" * 64
        receipt_digest = "sha256:$('b' * 64)"
        installation_id = $installationId
        mutation_policy = "owner-apis-and-owner-bootstrap-only"
        identity_policy =
            "logical-keys-resolve-only-from-signed-owner-receipts-and-typed-read-back"
        owner_receipt_digests = @(
            "core", "tessara.responses", "tessara.datasets", "tessara.components",
            "tessara.dashboards", "tessara.reference.scoped-records" | ForEach-Object {
                [pscustomobject][ordered]@{
                    owner = $_
                    input_digest = "sha256:$('c' * 64)"
                    result_digest = "sha256:$('d' * 64)"
                }
            }
        )
        logical_identities = [pscustomobject][ordered]@{
            responses = [pscustomobject]$responses
            datasets = [pscustomobject]$datasets
            components = [pscustomobject]@{}
            dashboard = [pscustomobject]@{ id = "01980000-0005-7000-8000-000000000001" }
        }
        forbidden_proofs = [pscustomobject][ordered]@{
            predicted_uuid = "not_used"
            foreign_owner_database_write = "not_used"
            response_sql_mutation = "not_used"
            copied_count_as_identity = "not_used"
        }
        restoration = [pscustomobject][ordered]@{
            state = "passed"
            basis = "successful_from_empty_owner_receipts"
        }
    }
}

function New-Sprint8CSelfTestSnapshot {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$Release,
        [Parameter(Mandatory)][string]$ImageToken
    )
    $digest = "sha256:$ImageToken"
    [pscustomobject][ordered]@{
        stage = $Stage
        captured_at = "2026-08-24T00:00:00Z"
        response = [pscustomobject][ordered]@{
            release = $Release
            runtime_image = $digest
            manifest_digest = $digest
            executable_sha256 = $ImageToken
            runtime_identity = [pscustomobject]@{
                service = "responses"; container_id = $ImageToken; restart_count = 0
            }
            owner_health = [pscustomobject]@{
                schema_version = 1
                status = "passing"
                checks = @(
                    [pscustomobject]@{ code = "response.database"; passing = $true },
                    [pscustomobject]@{ code = "response.configuration"; passing = $true },
                    [pscustomobject]@{ code = "response.security_state"; passing = $true },
                    [pscustomobject]@{ code = "response.provider.forms"; passing = $true },
                    [pscustomobject]@{ code = "response.provider.workflow"; passing = $true },
                    [pscustomobject]@{ code = "response.events.publication"; passing = $true },
                    [pscustomobject]@{ code = "response.export.publication"; passing = $true }
                )
            }
            module = [pscustomobject]@{
                kind = "independently_deployed"
                definition = [pscustomobject]@{ id = "tessara.responses" }
                release = [pscustomobject]@{
                    id = "release-$Release"; version = $Release
                    runtime_image = $digest; manifest_digest = $digest
                }
                instance = [pscustomobject]@{ id = "response-instance"; ready = $true; healthy = $true }
                manifest = [pscustomobject]@{ release_version = $Release }
                diagnostics = [pscustomobject]@{
                    details = [pscustomobject]@{
                        release = $Release
                        facts = [pscustomobject]@{
                            workflow_event_consumer_last_stable_at =
                                "2026-08-24T00:00:00Z"
                            workflow_event_pending_count = "0"
                            export_head_sequence = "12"
                        }
                        authorization = [pscustomobject]@{
                            authorization_revision = 1
                            organization_revision = 1
                            updated_at = "2026-08-24T00:00:00Z"
                            enabled = $true
                            document_state = "enabled"
                        }
                    }
                }
            }
            typed_resource_identity = [pscustomobject]@{
                response = [pscustomobject]@{ owner = "response-instance"; id = "response-1" }
            }
            navigation = [pscustomobject]@{
                contribution_id = "tessara.responses.navigation"; href = "/responses"
            }
            product_list = @([pscustomobject]@{ id = "response-1"; status = "submitted" })
            product_details = @([pscustomobject]@{ response_id = "response-1"; status = 200 })
            summary = [pscustomobject]@{ state = "available"; draft_count = 1; submitted_count = 3 }
            outbox_positions = [pscustomobject]@{
                pending_workflow_event_count = 4; export_head_sequence = 12
            }
        }
        unrelated = [pscustomobject][ordered]@{
            services = @([pscustomobject]@{
                service = "datasets"; container_id = "c" * 64; restart_count = 0
            })
            modules = @([pscustomobject]@{
                definition = [pscustomobject]@{ id = "tessara.datasets" }
                diagnostics = [pscustomobject]@{
                    details = [pscustomobject]@{
                        authorization = [pscustomobject]@{
                            updated_at = "2026-08-24T00:00:00Z"; enabled = $true
                        }
                    }
                }
            })
            dataset_product = [pscustomobject]@{ list = @([pscustomobject]@{ id = "dataset-1" }) }
            component_product = @([pscustomobject]@{ id = "component-1" })
            dashboard_product = [pscustomobject]@{ id = "dashboard-1" }
            forms = @([pscustomobject]@{ id = "form-1" })
            workflows = @([pscustomobject]@{ id = "workflow-1" })
            navigation = [pscustomobject]@{ revision = 1 }
            operations = [pscustomobject]@{ state = "available" }
            summary = [pscustomobject]@{ state = "available" }
            availability = [pscustomobject]@{ gateway = "healthy"; supervisor = "healthy" }
        }
    }
}

function Test-Sprint8CResponseUpgradeHarness {
    $contract = Read-Sprint8CUpgradeContract
    $fixtureReceipt = New-Sprint8CSelfTestFixtureReceipt
    $fixtureIdentity = Get-Sprint8CUpgradeFixtureIdentity -Fixture $fixtureReceipt
    if (@($fixtureIdentity.responses.PSObject.Properties).Count -ne 4 -or
        [uint64]$fixtureIdentity.response_export_minimum -ne 3 -or
        [string]$fixtureIdentity.response_module_instance_id -cne
            "01980000-0000-7000-8000-000000000083") {
        throw "Response upgrade self-test did not accept the four signed owner read-backs."
    }
    $substitutedFixtureSurface = Copy-Sprint8CJsonValue -Value $fixtureReceipt
    $substitutedFixtureSurface.logical_identities.responses | Add-Member `
        -NotePropertyName "response.unexpected" -NotePropertyValue `
        $substitutedFixtureSurface.logical_identities.responses.'response.draft.owner'
    $rejected = $false
    try {
        Get-Sprint8CUpgradeFixtureIdentity -Fixture $substitutedFixtureSurface | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Response upgrade self-test accepted an unexpected fifth Response fixture."
    }
    $digest = "sha256:$('a' * 64)"
    $fixedLockfile = [pscustomobject]@{
        core = [pscustomobject]@{ version = "0.1.0"; core_image = $digest }
        modules = @(
            [pscustomobject]@{
                definition_id = $responseDefinition; version = $candidateRelease
                manifest_digest = $digest; runtime_image = $digest
            },
            [pscustomobject]@{
                definition_id = "tessara.datasets"; version = "1.0.0"
                manifest_digest = $digest; runtime_image = $digest
            },
            [pscustomobject]@{
                definition_id = "tessara.components"; version = "1.1.0"
                manifest_digest = $digest; runtime_image = $digest
            },
            [pscustomobject]@{
                definition_id = "tessara.dashboards"; version = "3.0.2"
                manifest_digest = $digest; runtime_image = $digest
            },
            [pscustomobject]@{
                definition_id = "tessara.reference.scoped-records"; version = "1.0.2"
                manifest_digest = $digest; runtime_image = $digest
            }
        )
        blueprint_revision = 1
        navigation = @()
        roles = @()
        administrator_enrollment_role = "administrator"
        capability_floor_version = "sprint-8b-v1"
        secret_references = [pscustomobject]@{}
    }
    Assert-Sprint8CFixedReleaseVersions -Lockfile $fixedLockfile `
        -Contract $contract | Out-Null
    $fixedProjection = ConvertTo-Sprint8CStableJson `
        (Get-Sprint8CFixedLockfileProjection -Lockfile $fixedLockfile)
    $reorderedFixedLockfile = Copy-Sprint8CJsonValue -Value $fixedLockfile
    $reorderedFixedLockfile.core = [pscustomobject][ordered]@{
        core_image = $digest
        version = "0.1.0"
    }
    if ((ConvertTo-Sprint8CStableJson `
        (Get-Sprint8CFixedLockfileProjection -Lockfile $reorderedFixedLockfile)) -cne
        $fixedProjection) {
        throw "Response upgrade self-test treated fixed projection property order as drift."
    }
    $tamperedFixedLockfile = Copy-Sprint8CJsonValue -Value $fixedLockfile
    $tamperedFixedLockfile.modules[1].version = "9.9.9"
    $rejected = $false
    try {
        Assert-Sprint8CFixedReleaseVersions -Lockfile $tamperedFixedLockfile `
            -Contract $contract | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected -or (ConvertTo-Sprint8CStableJson `
        (Get-Sprint8CFixedLockfileProjection -Lockfile $tamperedFixedLockfile)) -ceq
        $fixedProjection) {
        throw "Response upgrade self-test accepted a substituted fixed owner release."
    }
    $runtimeCatalog = [pscustomobject]@{
        revision = 1
        issued_at = "2026-08-24T00:00:00Z"
        module_releases = Copy-Sprint8CJsonValue -Value $fixedLockfile.modules
    }
    $exerciseCatalog = New-Sprint8CUpgradeExerciseCatalog `
        -RuntimeCatalog $runtimeCatalog -InitialLockfile $fixedLockfile `
        -BaselineMetadata ([pscustomobject]@{
            release_identity = [pscustomobject]@{
                manifest_digest = "sha256:$('b' * 64)"
                runtime_image = "sha256:$('b' * 64)"
            }
        }) -CandidateManifestDigest $digest -CandidateImageDigest $digest
    $exerciseResponses = @($exerciseCatalog.module_releases | Where-Object {
        [string]$_.definition_id -ceq $responseDefinition
    })
    if (($exerciseResponses.version -join "`n") -cne
        (@($candidateRelease, $baselineRelease) -join "`n")) {
        throw "Response upgrade self-test did not derive the exact two-release exercise catalog."
    }
    $tamperedRuntimeCatalog = Copy-Sprint8CJsonValue -Value $runtimeCatalog
    $tamperedRuntimeCatalog.module_releases[0].runtime_image = "sha256:$('c' * 64)"
    $rejected = $false
    try {
        New-Sprint8CUpgradeExerciseCatalog -RuntimeCatalog $tamperedRuntimeCatalog `
            -InitialLockfile $fixedLockfile -BaselineMetadata ([pscustomobject]@{
                release_identity = [pscustomobject]@{
                    manifest_digest = "sha256:$('b' * 64)"
                    runtime_image = "sha256:$('b' * 64)"
                }
            }) -CandidateManifestDigest $digest -CandidateImageDigest $digest | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Response upgrade self-test accepted a substituted signed-catalog candidate."
    }
    $mock = [pscustomobject]@{
        materialization_plan = [pscustomobject]@{
            actions = @(
                [pscustomobject]@{ action = "acquire_image"; component = $responseDefinition; digest = $digest },
                [pscustomobject]@{ action = "migrate"; owner = $responseDefinition; image = $digest },
                [pscustomobject]@{ action = "health_gate"; owner = $responseDefinition },
                [pscustomobject]@{ action = "switch_traffic"; owner = $responseDefinition },
                [pscustomobject]@{ action = "verify_read_back" }
            )
        }
    }
    Assert-Sprint8CExactResponseDeltaPlan -Lockfile $mock -ExpectedImageDigest $digest | Out-Null
    $tamperedPlan = Copy-Sprint8CJsonValue -Value $mock
    $tamperedPlan.materialization_plan.actions = @(
        $tamperedPlan.materialization_plan.actions +
            [pscustomobject]@{ action = "configure"; owner = "tessara.datasets" }
    )
    $rejected = $false
    try {
        Assert-Sprint8CExactResponseDeltaPlan -Lockfile $tamperedPlan `
            -ExpectedImageDigest $digest | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Response upgrade self-test accepted an unrelated owner action." }

    $boundCatalog = Select-Sprint8CBoundRuntimeCatalog -ExpectedCatalogDigest $digest `
        -Candidates @(
            [pscustomobject]@{
                stage = "noop"; catalog_digest = $digest
                payload = [pscustomobject]@{ api_version = "tessara.io/release-catalog/v1" }
            },
            [pscustomobject]@{
                stage = "first"; catalog_digest = "sha256:$('b' * 64)"
                payload = [pscustomobject]@{ api_version = "wrong" }
            }
        )
    if ([string]$boundCatalog.stage -cne "noop") {
        throw "Response upgrade self-test did not select the current resolved catalog."
    }
    foreach ($catalogCase in @(
        [pscustomobject]@{
            digest = "sha256:$('c' * 64)"; candidates = @($boundCatalog)
        },
        [pscustomobject]@{
            digest = $digest
            candidates = @($boundCatalog, [pscustomobject]@{
                stage = "first"; catalog_digest = $digest
                payload = [pscustomobject]@{ api_version = "digest-collision" }
            })
        }
    )) {
        $rejected = $false
        try {
            Select-Sprint8CBoundRuntimeCatalog -ExpectedCatalogDigest $catalogCase.digest `
                -Candidates @($catalogCase.candidates) | Out-Null
        } catch { $rejected = $true }
        if (-not $rejected) {
            throw "Response upgrade self-test accepted a missing or colliding runtime catalog."
        }
    }

    $containerId = "a" * 64
    $inspection = ConvertFrom-Sprint8CUpgradeContainerInspection -InspectionOutput @(
        "informational Docker output",
        (@{
            Id = $containerId; Image = "sha256:$('b' * 64)"; RestartCount = 0
            State = @{ Status = "running"; Health = @{ Status = "healthy" } }
        } | ConvertTo-Json -Compress)
    ) -InspectionExitCode 0 -ExpectedId $containerId -Service "core"
    if ([string]$inspection.container_id -cne $containerId -or
        [string]$inspection.health -cne "healthy") {
        throw "Response upgrade self-test did not normalize exact Docker inspection identity."
    }
    $startingInspection = @((@{
        Id = $containerId; Image = "sha256:$('b' * 64)"; RestartCount = 0
        State = @{ Status = "running"; Health = @{ Status = "starting" } }
    } | ConvertTo-Json -Compress))
    $rejected = $false
    try {
        ConvertFrom-Sprint8CUpgradeContainerInspection -InspectionOutput $startingInspection `
            -InspectionExitCode 0 -ExpectedId $containerId -Service "responses" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Response upgrade self-test accepted a starting service as final evidence."
    }
    $starting = ConvertFrom-Sprint8CUpgradeContainerInspection `
        -InspectionOutput $startingInspection -InspectionExitCode 0 `
        -ExpectedId $containerId -Service "responses" -AllowStarting
    if ([string]$starting.health -cne "starting") {
        throw "Response upgrade self-test did not classify the bounded starting state."
    }
    $rejected = $false
    try {
        ConvertFrom-Sprint8CUpgradeContainerInspection -InspectionOutput @((@{
            Id = "c" * 64; Image = "sha256:$('b' * 64)"; RestartCount = 0
            State = @{ Status = "running"; Health = @{ Status = "healthy" } }
        } | ConvertTo-Json -Compress)) -InspectionExitCode 0 `
            -ExpectedId $containerId -Service "core" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Response upgrade self-test accepted a substituted Docker object identity."
    }

    $pre = New-Sprint8CSelfTestSnapshot -Stage "pre-exercise-candidate" `
        -Release $candidateRelease -ImageToken ("a" * 64)
    $snapshots = [Collections.Generic.List[object]]::new()
    $proofs = [Collections.Generic.List[object]]::new()
    for ($index = 0; $index -lt $contract.sequence.Count; $index++) {
        $release = [string]$contract.sequence[$index]
        $token = if ($release -ceq $baselineRelease) { "b" * 64 } else { "a" * 64 }
        $snapshot = New-Sprint8CSelfTestSnapshot -Stage ([string]$contract.stages[$index]) `
            -Release $release -ImageToken $token
        $snapshot.response.module.diagnostics.details.authorization.authorization_revision = 2
        $snapshot.response.module.diagnostics.details.authorization.organization_revision = 2
        $snapshot.response.module.diagnostics.details.authorization.updated_at =
            "2026-08-24T00:01:00Z"
        $snapshot.response.module.diagnostics.details.facts.workflow_event_consumer_last_stable_at =
            "2026-08-24T00:01:00Z"
        $snapshot.unrelated.modules[0].diagnostics.details.authorization.updated_at =
            "2026-08-24T00:01:00Z"
        $snapshots.Add($snapshot)
        $proofs.Add((Assert-Sprint8CUpgradePreservation -Expected $pre -Actual $snapshot `
            -Stage ([string]$contract.stages[$index])))
    }

    foreach ($tamper in @(
        "response", "health", "instance", "typed", "navigation", "outbox",
        "diagnostic-fact", "unrelated"
    )) {
        $actual = Copy-Sprint8CJsonValue -Value $snapshots[0]
        switch ($tamper) {
            "response" { $actual.response.product_list[0].status = "draft" }
            "health" { $actual.response.owner_health.status = "failing" }
            "instance" { $actual.response.module.instance.id = "substituted-instance" }
            "typed" { $actual.response.typed_resource_identity.response.id = "substituted-response" }
            "navigation" { $actual.response.navigation.href = "/core-responses" }
            "outbox" { $actual.response.outbox_positions.export_head_sequence = 13 }
            "diagnostic-fact" {
                $actual.response.module.diagnostics.details.facts.workflow_event_pending_count = "1"
            }
            "unrelated" { $actual.unrelated.services[0].restart_count = 1 }
        }
        $rejected = $false
        try {
            Assert-Sprint8CUpgradePreservation -Expected $pre -Actual $actual `
                -Stage "self-test-$tamper-tamper" | Out-Null
        } catch { $rejected = $true }
        if (-not $rejected) {
            throw "Response upgrade self-test accepted '$tamper' preservation drift."
        }
    }

    $transitions = @(for ($index = 0; $index -lt 4; $index++) {
        [pscustomobject][ordered]@{
            stage = [string]$contract.stages[$index]
            target_release = [string]$contract.sequence[$index]
            fixed_owner_lockfile = "passed"
        }
    })
    $result = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "independent-response-upgrade-rollback-restoration"
        state = "passed"
        self_test = $true
        database_free = $true
        compose_project = "tessara-s8c-upgrade-selftest"
        source = [pscustomobject]@{ commit = "a" * 40; tree = "b" * 40; dirty = $false }
        environment_fingerprint_sha256 = Get-Sprint7ASha256 `
            -Text "sprint-8c-response-upgrade-self-test`n"
        release_contract = $contract
        release_fixture = [pscustomobject][ordered]@{
            baseline = [pscustomobject]@{
                version = $baselineRelease; runtime_image = "sha256:$('b' * 64)"
                executable_sha256 = "b" * 64
                source_identity = [pscustomobject]@{
                    source_model = $baselineSourceModel
                    candidate_source_dependency = "none"
                    commit = $baselineSourceCommit
                    tree = $baselineSourceTree
                    patch_sha256 = [string]$contract.prior_compatible_artifact.patch_sha256
                    materialized_source_sha256 = "d" * 64
                    materialized_file_count = 100
                    marker_sha256 = "e" * 64
                }
                compatibility = [pscustomobject]@{
                    state = "passed"
                    manifest_projection = $baselineManifestProjection
                    allowed_manifest_variations = $allowedBaselineManifestVariations
                }
            }
            candidate = [pscustomobject]@{
                version = $candidateRelease; runtime_image = "sha256:$('a' * 64)"
                executable_sha256 = "a" * 64
            }
        }
        fixture_receipt = [pscustomobject]@{ path = "self-test"; sha256 = "c" * 64 }
        pre_exercise_snapshot = $pre
        transitions = $transitions
        stage_snapshots = @($snapshots)
        preservation_proofs = @($proofs)
        release_restoration = [pscustomobject]@{
            attempted = $false; state = "passed"; final_release = $candidateRelease
        }
        cleanup_restoration = [pscustomobject]@{
            state = "passed"; mode = "database-free-self-test"
        }
        failure = $null
    }
    Assert-Sprint8CResponseUpgradeReceipt -Document $result -Contract $contract | Out-Null
    $tamperedReceipt = Copy-Sprint8CJsonValue -Value $result
    $tamperedReceipt.stage_snapshots = @($tamperedReceipt.stage_snapshots | Select-Object -Skip 1)
    $rejected = $false
    try {
        Assert-Sprint8CResponseUpgradeReceipt -Document $tamperedReceipt `
            -Contract $contract | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Response upgrade self-test accepted a truncated four-stage receipt."
    }
    $receiptTamperCases = @(
        [pscustomobject]@{
            name = "dirty-source"
            mutate = { param($value) $value.source.dirty = $true }
        },
        [pscustomobject]@{
            name = "retained-failure"
            mutate = { param($value) $value.failure = [pscustomobject]@{ message = "tampered" } }
        },
        [pscustomobject]@{
            name = "fixed-owner-proof"
            mutate = { param($value) $value.transitions[0].fixed_owner_lockfile = "failed" }
        },
        [pscustomobject]@{
            name = "typed-preservation-proof"
            mutate = { param($value) $value.preservation_proofs[0].typed_resource_identity = "failed" }
        },
        [pscustomobject]@{
            name = "artifact-substitution"
            mutate = {
                param($value)
                $value.release_fixture.baseline.runtime_image =
                    $value.release_fixture.candidate.runtime_image
            }
        },
        [pscustomobject]@{
            name = "baseline-source-provenance"
            mutate = {
                param($value)
                $value.release_fixture.baseline.source_identity.patch_sha256 = "0" * 64
            }
        }
    )
    foreach ($case in $receiptTamperCases) {
        $tamperedReceipt = Copy-Sprint8CJsonValue -Value $result
        & $case.mutate $tamperedReceipt
        $rejected = $false
        try {
            Assert-Sprint8CResponseUpgradeReceipt -Document $tamperedReceipt `
                -Contract $contract | Out-Null
        } catch { $rejected = $true }
        if (-not $rejected) {
            throw "Response upgrade self-test accepted '$($case.name)' receipt tampering."
        }
    }
    if ($evidencePathWasExplicit -and -not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        Publish-Sprint8CHarnessEvidence -Document $result -OutputPath $EvidencePath | Out-Null
    }
    $result
}

if ($SelfTest) {
    Test-Sprint8CResponseUpgradeHarness | ConvertTo-Json -Depth 100
    return
}

Assert-Sprint8CComposeProject -ComposeProject $ComposeProject | Out-Null
if (-not $UseExistingTopology) {
    Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject `
        -Authorized ([bool]$AuthorizeDisposableReset)
}
$source = Get-Sprint8CSourceIdentity -RequireClean
$contract = Read-Sprint8CUpgradeContract
$composePath = Resolve-Sprint8CRepositoryPath -Path $ComposeFile
$candidateManifestFullPath = Resolve-Sprint8CRepositoryPath -Path $CandidateManifestPath
if (-not (Test-Path -LiteralPath $composePath -PathType Leaf) -or
    -not (Test-Path -LiteralPath $candidateManifestFullPath -PathType Leaf)) {
    throw "Response upgrade Compose or candidate Manifest input is missing."
}

$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT"
)
$environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $environmentNames
$childRoot = Resolve-Sprint8CRepositoryPath -Path (
    "target/sprint-8c-response-upgrade/$ComposeProject-$([Guid]::NewGuid().ToString('N'))"
)
[IO.Directory]::CreateDirectory($childRoot) | Out-Null
$ownedTopology = $false
$ports = $null
$configuration = $null
$fixturePair = $null
$baselineMetadata = $null
$candidateIdentity = $null
$preExercise = $null
$transitions = [Collections.Generic.List[object]]::new()
$stages = [Collections.Generic.List[object]]::new()
$preservation = [Collections.Generic.List[object]]::new()
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$releaseRestoration = [pscustomobject][ordered]@{
    attempted = $false
    state = "not_started"
    final_release = $null
}
$failure = $null
$script:upgradeFixture = $null
$script:upgradeComposePath = $null
$script:upgradePorts = $null
$script:upgradeSession = $null
$script:upgradeCatalog = $null
$script:upgradeInitialBootstrapReceipts = $null
$script:upgradeInitialFixedProjection = $null

try {
    if ($UseExistingTopology) {
        foreach ($name in @(
            "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT"
        )) {
            if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
                throw "Existing Response upgrade topology requires inherited $name."
            }
        }
        $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$env:TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$env:TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$env:TESSARA_SUPERVISOR_PORT)
        Assert-Sprint8CExistingTopology -ComposePath $composePath `
            -ComposeProject $ComposeProject | Out-Null
        if ([string]::IsNullOrWhiteSpace($FixtureReceiptPath)) {
            throw "Existing Response upgrade topology requires -FixtureReceiptPath."
        }
    } else {
        $materializationEvidence = Join-Path $childRoot "materialization.json"
        $arguments = @(
            "-Target", "ReferenceNoOp", "-ComposeProject", $ComposeProject,
            "-EvidencePath", $materializationEvidence, "-AuthorizeDisposableReset", "-KeepTopology"
        )
        if ($SkipBuild) { $arguments += "-SkipBuild" }
        Invoke-Sprint8CChildScript -ScriptPath "scripts/materialize-sprint-8c.ps1" `
            -Arguments $arguments | Out-Null
        $materialized = Get-Content -LiteralPath $materializationEvidence -Raw |
            ConvertFrom-Json -Depth 100
        if ([string]$materialized.state -cne "passed" -or
            $null -eq $materialized.environment -or
            [string]::IsNullOrWhiteSpace([string]$materialized.fixture_receipt_path)) {
            throw "Response upgrade materialization did not publish topology and fixture context."
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
        throw "Response upgrade requires an authenticated fixture receipt pair."
    }
    $fixtureDocument = Get-Content -LiteralPath $resolvedFixturePath -Raw |
        ConvertFrom-Json -Depth 100
    if ([string]$fixtureDocument.compose_project -cne $ComposeProject) {
        throw "Response upgrade fixture receipt belongs to a different Compose project."
    }
    $script:upgradeFixture = Get-Sprint8CUpgradeFixtureIdentity -Fixture $fixtureDocument
    $fixturePair = [pscustomobject][ordered]@{
        path = $resolvedFixturePath
        sha256 = (Get-FileHash -LiteralPath $resolvedFixturePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $configuration = Get-Sprint8CComposeConfiguration -ComposePath $composePath `
        -ComposeProject $ComposeProject
    Assert-Sprint8CDatabaseIsolationConfiguration -ComposeConfiguration $configuration | Out-Null
    $script:upgradeComposePath = $composePath
    $script:upgradePorts = $ports
    $script:upgradeSession = [Microsoft.PowerShell.Commands.WebRequestSession]::new()
    Invoke-Sprint8CUpgradeRequest -BaseUrl $ports.gateway_url -Path "/api/auth/login" `
        -Session $script:upgradeSession -Method POST -Body ([ordered]@{
            email = "admin@tessara.local"
            password = "tessara-dev-admin"
        }) | Out-Null

    $summary = (Invoke-Sprint8CUpgradeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/admin/composition" -Session $script:upgradeSession).document
    if ($null -eq $summary.latest_blueprint -or $null -eq $summary.latest_lockfile -or
        $null -eq $summary.latest_receipt) {
        throw "Response upgrade requires an applied candidate composition."
    }
    $initialResponse = @($summary.latest_lockfile.modules | Where-Object {
        [string]$_.definition_id -ceq $responseDefinition
    })
    if ($initialResponse.Count -ne 1 -or
        [string]$initialResponse[0].version -cne $candidateRelease) {
        throw "Response upgrade topology does not begin at candidate release $candidateRelease."
    }
    Assert-Sprint8CFixedReleaseVersions -Lockfile $summary.latest_lockfile `
        -Contract $contract | Out-Null
    $script:upgradeInitialFixedProjection = ConvertTo-Sprint8CStableJson `
        (Get-Sprint8CFixedLockfileProjection -Lockfile $summary.latest_lockfile)
    $script:upgradeInitialBootstrapReceipts = ConvertTo-Sprint8CStableJson `
        @($summary.latest_receipt.bootstrap_receipts | Sort-Object owner)

    $candidateRuntime = Get-Sprint8CUpgradeContainerIdentity -ComposePath $composePath `
        -Service "responses"
    $candidateImage = [string]$candidateRuntime.image_id
    if ($candidateImage -cne [string]$initialResponse[0].runtime_image) {
        throw "Live Response image differs from the initial resolved candidate lockfile."
    }
    $candidateImageIdentity = Get-Sprint8CUpgradeImageIdentity -Image $candidateImage `
        -ExpectedSource $source -ExpectedRelease $candidateRelease
    $candidateManifestDigestOutput = @(& cargo run --quiet --locked --offline `
        -p tessara-supervisor --bin tessara-compose -- `
        manifest-digest $candidateManifestFullPath 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "Could not compute the candidate Response Manifest digest."
    }
    $candidateManifestDigest = [string]($candidateManifestDigestOutput | Select-Object -Last 1)
    if ($candidateManifestDigest -cne [string]$initialResponse[0].manifest_digest) {
        throw "Candidate Response Manifest differs from the initial resolved lockfile."
    }
    $candidateExecutable = Get-Sprint8CUpgradeExecutableSha256 `
        -ContainerId ([string]$candidateRuntime.container_id)
    $candidateIdentity = [pscustomobject][ordered]@{
        definition_id = $responseDefinition
        version = $candidateRelease
        manifest_digest = $candidateManifestDigest
        runtime_image = $candidateImage
        executable_sha256 = $candidateExecutable
        image_provenance = $candidateImageIdentity
    }

    $baselineMetadataPath = Join-Path $childRoot "baseline-metadata.json"
    Invoke-Sprint8CChildScript `
        -ScriptPath "scripts/build-sprint-8c-response-upgrade-baseline.ps1" `
        -Arguments @(
            "-OutputTag", $BaselineImageTag,
            "-BaselineRelease", $baselineRelease,
            "-MetadataOutputPath", $baselineMetadataPath
        ) | Out-Null
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $baselineMetadataPath `
        -SidecarPath "$baselineMetadataPath.sha256")) {
        throw "Response baseline builder did not publish authenticated release metadata."
    }
    $baselineMetadata = Get-Content -LiteralPath $baselineMetadataPath -Raw |
        ConvertFrom-Json -Depth 100
    $baselineSourceMetadata = $baselineMetadata.baseline_source_identity
    if ([string]$baselineMetadata.receipt_contract -cne $baselineReceiptContract -or
        [string]$baselineMetadata.release_identity.definition_id -cne $responseDefinition -or
        [string]$baselineMetadata.release_identity.version -cne $baselineRelease -or
        [string]$baselineMetadata.source_identity.commit -cne [string]$source.commit -or
        [string]$baselineMetadata.source_identity.tree -cne [string]$source.tree -or
        [bool]$baselineMetadata.source_identity.dirty -or
        [string]$baselineMetadata.source_identity.builder -cne
            "scripts/build-sprint-8c-response-upgrade-baseline.ps1" -or
        [string]$baselineSourceMetadata.source_model -cne $baselineSourceModel -or
        [string]$baselineSourceMetadata.candidate_source_dependency -cne "none" -or
        [string]$baselineSourceMetadata.commit -cne $baselineSourceCommit -or
        [string]$baselineSourceMetadata.commit -ceq [string]$source.commit -or
        [string]$baselineSourceMetadata.tree -cne $baselineSourceTree -or
        [string]$baselineSourceMetadata.patch_path -cne
            "deploy/sprint-8c/baselines/response-0.9.0.patch" -or
        [string]$baselineSourceMetadata.patch_sha256 -cne
            [string]$contract.prior_compatible_artifact.patch_sha256 -or
        [string]$baselineSourceMetadata.materialized_source_sha256 -cnotmatch
            '^[0-9a-f]{64}$' -or
        [int]$baselineSourceMetadata.materialized_file_count -le 0 -or
        [string]$baselineSourceMetadata.marker_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$baselineSourceMetadata.cargo_package -cne "tessara-response-module" -or
        [string]$baselineSourceMetadata.cargo_lock_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$baselineSourceMetadata.manifest_source_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$baselineSourceMetadata.migration_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$baselineSourceMetadata.migration_sha256 -cne
            [string]$baselineMetadata.compatibility.candidate_migration_sha256 -or
        [string]$baselineMetadata.compatibility.state -cne "passed" -or
        [string]$baselineMetadata.compatibility.manifest_projection -cne
            $baselineManifestProjection -or
        [string]$baselineMetadata.compatibility.projection_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$baselineMetadata.compatibility.tracked_candidate_release -cne
            $candidateRelease -or
        [string]$baselineMetadata.compatibility.candidate_manifest_digest -cne
            $candidateManifestDigest -or
        [string]$baselineMetadata.compatibility.resource_contract_id -cne
            "tessara.responses.response" -or
        [string]$baselineMetadata.compatibility.resource_contract_version -cne "2.0.0" -or
        [string]$baselineMetadata.compatibility.lifecycle_contract_id -cne
            "tessara.responses.response-lifecycle" -or
        [string]$baselineMetadata.compatibility.lifecycle_contract_version -cne "2.0.0" -or
        [string]$baselineMetadata.compatibility.rollback_schema_expectation -cne
            "reads_current_1_0_0_response_baseline_after_rollback" -or
        [string]$baselineMetadata.compatibility.rollback_reader_contract -cne
            "same_owned_schema_and_product_read_paths" -or
        [string]$baselineMetadata.compatibility.migration_compatibility -cne
            "required-owned-tables-and-live-rollback-readback" -or
        [string]$baselineMetadata.compatibility.migration_path -cne
            "crates/tessara-response-module/migrations/001_response_module.sql" -or
        [string]$baselineMetadata.fixture_identity.contract_path -cne
            "deploy/sprint-8c/fixtures/upgrade-fixture-contract.json" -or
        [string]$baselineMetadata.fixture_identity.fixture_kind -cne
            $baselineFixtureKind -or
        [string]$baselineMetadata.release_identity.runtime_image -ceq $candidateImage -or
        [string]$baselineMetadata.release_identity.executable_sha256 -ceq $candidateExecutable) {
        throw "Response baseline metadata is not a distinct authenticated independent compatible 0.9.0 release."
    }
    Assert-Sprint8CExactSequence -Expected @() `
        -Actual @($baselineSourceMetadata.cargo_features) `
        -Label "Response baseline Cargo features"
    Assert-Sprint8CExactSequence -Expected $allowedBaselineManifestVariations `
        -Actual @($baselineMetadata.compatibility.allowed_manifest_variations) `
        -Label "Response baseline compatible Manifest variations"
    $baselineManifestDocumentPath = Resolve-Sprint8CRepositoryPath `
        -Path ([string]$baselineMetadata.manifest_path)
    $baselineSourcePaths = [ordered]@{
        $baselineManifestDocumentPath = [string]$baselineMetadata.release_identity.manifest_document_sha256
        (Resolve-Sprint8CRepositoryPath -Path `
            "deploy/sprint-8c/Dockerfile.response-upgrade-baseline") =
                [string]$baselineMetadata.source_identity.dockerfile_sha256
        (Resolve-Sprint8CRepositoryPath -Path "deploy/sprint-8c/Dockerfile.response") =
                [string]$baselineMetadata.source_identity.candidate_dockerfile_sha256
        (Resolve-Sprint8CRepositoryPath -Path `
            "scripts/build-sprint-8c-response-upgrade-baseline.ps1") =
                [string]$baselineMetadata.source_identity.builder_sha256
        (Resolve-Sprint8CRepositoryPath -Path `
            "deploy/sprint-8c/baselines/response-0.9.0.patch") =
                [string]$baselineSourceMetadata.patch_sha256
        (Resolve-Sprint8CRepositoryPath -Path `
            "crates/tessara-response-module/migrations/001_response_module.sql") =
                [string]$baselineMetadata.compatibility.candidate_migration_sha256
        (Resolve-Sprint8CRepositoryPath -Path $UpgradeContractPath) =
                [string]$baselineMetadata.fixture_identity.contract_sha256
    }
    foreach ($path in $baselineSourcePaths.Keys) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne
                $baselineSourcePaths[$path]) {
            throw "Response baseline metadata does not authenticate '$path'."
        }
    }
    if ([string]$baselineSourceMetadata.manifest_source_sha256 -cne
            [string]$baselineMetadata.release_identity.manifest_document_sha256) {
        throw "Response baseline materialized source hashes are incompatible with retained artifacts."
    }
    $baselineManifestDocument = Get-Content -LiteralPath $baselineManifestDocumentPath -Raw |
        ConvertFrom-Json -Depth 100
    $candidateManifestDocument = Get-Content -LiteralPath $candidateManifestFullPath -Raw |
        ConvertFrom-Json -Depth 100
    $baselineBehavior = Get-Sprint8CManifestBehaviorProjection -Manifest $baselineManifestDocument
    $candidateBehavior = Get-Sprint8CManifestBehaviorProjection -Manifest $candidateManifestDocument
    if ((ConvertTo-Sprint8CStableJson $baselineBehavior) -cne
            (ConvertTo-Sprint8CStableJson $candidateBehavior) -or
        (Get-Sprint7ASha256 -Text (
            (ConvertTo-Sprint8CStableJson $candidateBehavior) + "`n"
        )) -cne [string]$baselineMetadata.compatibility.projection_sha256) {
        throw "Response baseline retained Manifest is not behaviorally compatible with the candidate."
    }
    Get-Sprint8CUpgradeImageIdentity `
        -Image ([string]$baselineMetadata.release_identity.runtime_image) `
        -ExpectedSource $baselineSourceMetadata -ExpectedRelease $baselineRelease `
        -ExpectedFixtureKind $baselineFixtureKind `
        -ExpectedBaselineSource $baselineSourceMetadata `
        -ExpectedBuilderSource $baselineMetadata.source_identity | Out-Null

    $boundRuntimeCatalog = Resolve-Sprint8CBoundRuntimeCatalog `
        -MaterializationRoot (Split-Path -Parent $resolvedFixturePath) `
        -ExpectedCatalogDigest ([string]$summary.latest_lockfile.catalog_digest)
    $script:upgradeCatalog = New-Sprint8CUpgradeExerciseCatalog `
        -RuntimeCatalog $boundRuntimeCatalog.payload -InitialLockfile $summary.latest_lockfile `
        -BaselineMetadata $baselineMetadata -CandidateManifestDigest $candidateManifestDigest `
        -CandidateImageDigest $candidateImage

    $preExercise = Get-Sprint8CUpgradeStageSnapshot -Stage "pre-exercise-candidate" `
        -ExpectedRelease $candidateRelease -ExpectedImage $candidateImage `
        -ExpectedManifestDigest $candidateManifestDigest
    for ($index = 0; $index -lt $contract.sequence.Count; $index++) {
        $release = [string]$contract.sequence[$index]
        $target = [pscustomobject]@{
            stage = [string]$contract.stages[$index]
            release = $release
            image = if ($release -ceq $baselineRelease) {
                [string]$baselineMetadata.release_identity.runtime_image
            } else { $candidateImage }
            manifest = if ($release -ceq $baselineRelease) {
                [string]$baselineMetadata.release_identity.manifest_digest
            } else { $candidateManifestDigest }
        }
        $transitions.Add((Invoke-Sprint8CResponseTransition -Stage $target.stage `
            -TargetRelease $target.release -TargetImage $target.image `
            -TargetManifestDigest $target.manifest))
        $snapshot = Get-Sprint8CUpgradeStageSnapshot -Stage $target.stage `
            -ExpectedRelease $target.release -ExpectedImage $target.image `
            -ExpectedManifestDigest $target.manifest
        $stages.Add($snapshot)
        $preservation.Add((Assert-Sprint8CUpgradePreservation -Expected $preExercise `
            -Actual $snapshot -Stage $target.stage))
    }
    Assert-Sprint8CExactSequence -Expected @($contract.sequence) `
        -Actual @($transitions.target_release) -Label "executed Response release sequence"
    Assert-Sprint8CExactSequence -Expected @($contract.stages) `
        -Actual @($transitions.stage) -Label "executed Response stage sequence"
    $releaseRestoration.state = "passed"
    $releaseRestoration.final_release = $candidateRelease
} catch {
    $failure = $_
} finally {
    if ($null -ne $ports -and $null -ne $script:upgradeSession -and
        $null -ne $candidateIdentity -and $null -ne $script:upgradeCatalog) {
        try {
            $projected = Get-Sprint8CUpgradeModuleProjection -BaseUrl $ports.gateway_url `
                -Session $script:upgradeSession -DefinitionId $responseDefinition
            if ([string]$projected.release.version -cne $candidateRelease) {
                $releaseRestoration.attempted = $true
                Invoke-Sprint8CResponseTransition -Stage "failure-restoration" `
                    -TargetRelease $candidateRelease `
                    -TargetImage ([string]$candidateIdentity.runtime_image) `
                    -TargetManifestDigest ([string]$candidateIdentity.manifest_digest) | Out-Null
            }
            $restored = Get-Sprint8CUpgradeModuleProjection -BaseUrl $ports.gateway_url `
                -Session $script:upgradeSession -DefinitionId $responseDefinition
            if ([string]$restored.kind -cne "independently_deployed" -or
                [string]$restored.release.version -cne $candidateRelease -or
                [string]$restored.instance.id -cne
                    [string]$script:upgradeFixture.response_module_instance_id -or
                -not [bool]$restored.instance.ready -or -not [bool]$restored.instance.healthy) {
                throw "Response release did not restore to the healthy intended independent candidate."
            }
            $releaseRestoration.state = "passed"
            $releaseRestoration.final_release = $candidateRelease
        } catch {
            if ($null -eq $failure) { $failure = $_ }
            $releaseRestoration.state = "failed"
            $releaseRestoration | Add-Member -NotePropertyName error `
                -NotePropertyValue $_.Exception.Message -Force
        }
    }
    try {
        if ($ownedTopology) {
            $cleanup = Remove-Sprint8CProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized ([bool]$AuthorizeDisposableReset)
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
            $cleanup | Add-Member -NotePropertyName mode `
                -NotePropertyValue "exact-project-teardown" -Force
        } elseif ($UseExistingTopology) {
            Assert-Sprint8CExistingTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject | Out-Null
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"
                mode = "existing-topology-restored-to-candidate-and-retained"
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

$configurationHash = if ($null -eq $configuration) { "none" } else {
    Get-Sprint7ASha256 -Text ((ConvertTo-Sprint8CStableJson $configuration) + "`n")
}
$fixtureHash = if ($null -eq $fixturePair) { "none" } else { [string]$fixturePair.sha256 }
$baselineHash = if ($null -eq $baselineMetadata) { "none" } else {
    Get-Sprint7ASha256 -Text ((ConvertTo-Sprint8CStableJson $baselineMetadata) + "`n")
}
$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n$configurationHash`n$fixtureHash`n$baselineHash`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    proof = "independent-response-upgrade-rollback-restoration"
    state = if ($null -eq $failure -and [string]$releaseRestoration.state -ceq "passed" -and
        [string]$cleanup.state -ceq "passed" -and $transitions.Count -eq 4 -and
        $stages.Count -eq 4 -and $preservation.Count -eq 4) { "passed" } else { "failed" }
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    release_contract = $contract
    release_fixture = [pscustomobject][ordered]@{
        baseline = if ($null -eq $baselineMetadata) { $null } else {
            [pscustomobject][ordered]@{
                definition_id = [string]$baselineMetadata.release_identity.definition_id
                version = [string]$baselineMetadata.release_identity.version
                manifest_digest = [string]$baselineMetadata.release_identity.manifest_digest
                runtime_image = [string]$baselineMetadata.release_identity.runtime_image
                migration_image = [string]$baselineMetadata.release_identity.migration_image
                image_reference = [string]$baselineMetadata.release_identity.image_reference
                executable_sha256 = [string]$baselineMetadata.release_identity.executable_sha256
                manifest_document_sha256 =
                    [string]$baselineMetadata.release_identity.manifest_document_sha256
                source_identity = $baselineMetadata.baseline_source_identity
                builder_source_identity = $baselineMetadata.source_identity
                compatibility = $baselineMetadata.compatibility
            }
        }
        candidate = $candidateIdentity
    }
    fixture_receipt = $fixturePair
    pre_exercise_snapshot = $preExercise
    transitions = @($transitions)
    stage_snapshots = @($stages)
    preservation_proofs = @($preservation)
    release_restoration = $releaseRestoration
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else {
        [pscustomobject][ordered]@{
            message = $failure.Exception.Message
            category = [string]$failure.CategoryInfo.Category
        }
    }
}
if ([string]$document.state -ceq "passed") {
    try {
        Assert-Sprint8CResponseUpgradeReceipt -Document $document -Contract $contract | Out-Null
    } catch {
        $document.state = "failed"
        $document.failure = [pscustomobject][ordered]@{
            message = $_.Exception.Message
            category = [string]$_.CategoryInfo.Category
        }
    }
}
$evidenceFullPath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8C Response upgrade/rollback failed; retained evidence: $evidenceFullPath"
}

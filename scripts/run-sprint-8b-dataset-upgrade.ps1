[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8b-implementation-upgrade",
    [string]$ComposeFile = "deploy/sprint-8b/compose.yaml",
    [switch]$UseExistingTopology,
    [string]$FixtureReceiptPath,
    [string]$UpgradeContractPath = "deploy/sprint-8b/fixtures/upgrade-fixture-contract.json",
    [string]$CandidateManifestPath = "crates/tessara-dataset-module/manifest.json",
    [string]$BaselineImageTag = "tessara-sprint-8b-datasets-upgrade-baseline:latest",
    [string]$EvidencePath = "target/sprint-8b-dataset-upgrade/result.json",
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

$datasetDefinition = "tessara.datasets"
$baselineRelease = "0.9.0"
$candidateRelease = "1.0.0"

function Copy-Sprint8BJsonValue {
    param([Parameter(Mandatory)]$Value)
    $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
}

function ConvertTo-Sprint8BStableJson {
    param([AllowNull()]$Value)
    if ($null -eq $Value) { return "null" }
    $Value | ConvertTo-Json -Depth 100 -Compress
}

function Assert-Sprint8BExactSequence {
    param(
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][object[]]$Actual,
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

function Read-Sprint8BUpgradeContract {
    $path = Resolve-Sprint8BRepositoryPath -Path $UpgradeContractPath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Dataset upgrade fixture contract is missing: $path"
    }
    $contract = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -Depth 100
    if ([int]$contract.schema_version -ne 1 -or
        [string]$contract.contract -cne "tessara.sprint-8b.dataset-upgrade-fixture" -or
        [string]$contract.transition.owner -cne $datasetDefinition -or
        [string]$contract.transition.mechanism -cne "resolved_one_owner_blueprint_delta" -or
        [string]$contract.transition.intended_release -cne $candidateRelease) {
        throw "Dataset upgrade fixture contract does not declare the exact one-owner Sprint 8B transition."
    }
    Assert-Sprint8BExactSequence -Expected @(
        $baselineRelease, $candidateRelease, $baselineRelease, $candidateRelease
    ) -Actual @($contract.sequence) -Label "Dataset upgrade sequence"
    if ([string]$contract.fixed_dependencies.'tessara.components' -cne "1.1.0" -or
        [string]$contract.fixed_dependencies.'tessara.dashboards' -cne "3.0.1") {
        throw "Dataset upgrade fixture does not keep Component 1.1.0 and Dashboard 3.0.1 fixed."
    }
    foreach ($required in @(
        "dataset_state", "provider_route", "typed_resource_identity", "navigation_identity"
    )) {
        if (@($contract.preserved) -cnotcontains $required) {
            throw "Dataset upgrade fixture omits preserved invariant '$required'."
        }
    }
    foreach ($required in @(
        "images", "containers", "restart_counts", "owner_data", "navigation", "availability"
    )) {
        if (@($contract.unrelated_unchanged) -cnotcontains $required) {
            throw "Dataset upgrade fixture omits unrelated invariant '$required'."
        }
    }
    $contract
}

function Assert-Sprint8BExactDatasetDeltaPlan {
    param(
        [Parameter(Mandatory)]$Lockfile,
        [Parameter(Mandatory)][string]$ExpectedImageDigest
    )

    $expected = @(
        [ordered]@{ action = "acquire_image"; component = $datasetDefinition; digest = $ExpectedImageDigest },
        [ordered]@{ action = "migrate"; owner = $datasetDefinition; image = $ExpectedImageDigest },
        [ordered]@{ action = "health_gate"; owner = $datasetDefinition },
        [ordered]@{ action = "switch_traffic"; owner = $datasetDefinition },
        [ordered]@{ action = "verify_read_back" }
    )
    $actual = @($Lockfile.materialization_plan.actions)
    if ((ConvertTo-Sprint8BStableJson $actual) -cne (ConvertTo-Sprint8BStableJson $expected)) {
        throw "Dataset transition is not the exact one-owner semantic delta: $(ConvertTo-Sprint8BStableJson $actual)"
    }
    [pscustomobject][ordered]@{
        owner = $datasetDefinition
        action_count = $actual.Count
        actions = $actual
        exact = $true
    }
}

function Assert-Sprint8BUpgradePreservation {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual,
        [Parameter(Mandatory)][string]$Stage
    )

    $expectedDataset = Copy-Sprint8BJsonValue -Value $Expected.dataset
    $actualDataset = Copy-Sprint8BJsonValue -Value $Actual.dataset
    foreach ($snapshot in @($expectedDataset, $actualDataset)) {
        $snapshot.release = "<transition-release>"
        $snapshot.runtime_image = "<transition-image>"
        $snapshot.manifest_digest = "<transition-manifest>"
        $snapshot.executable_sha256 = "<transition-executable>"
        $snapshot.runtime_identity = "<transition-runtime>"
    }
    if ((ConvertTo-Sprint8BStableJson $expectedDataset) -cne
        (ConvertTo-Sprint8BStableJson $actualDataset)) {
        throw "$Stage changed Dataset state, provider route, typed identity, configuration, navigation, or behavior."
    }
    if ((ConvertTo-Sprint8BStableJson $Expected.unrelated) -cne
        (ConvertTo-Sprint8BStableJson $Actual.unrelated)) {
        throw "$Stage changed an unrelated owner image, container, restart count, data, navigation, or availability."
    }
    [pscustomobject][ordered]@{
        stage = $Stage
        dataset_preserved = $true
        unrelated_preserved = $true
    }
}

function ConvertFrom-Sprint8BUpgradeContainerInspection {
    param(
        [Parameter(Mandatory)][string[]]$InspectionOutput,
        [Parameter(Mandatory)][int]$InspectionExitCode,
        [Parameter(Mandatory)][string]$ExpectedId,
        [Parameter(Mandatory)][string]$Service,
        [switch]$AllowStarting
    )

    $inspectionJson = @($InspectionOutput | Where-Object { $_.TrimStart().StartsWith('{') })
    if ($InspectionExitCode -ne 0 -or $inspectionJson.Count -ne 1) {
        throw "Could not inspect exactly one Dataset upgrade service '$Service'."
    }
    try {
        $inspection = $inspectionJson[0] | ConvertFrom-Json -Depth 30
    } catch {
        throw "Could not decode Dataset upgrade service '$Service' inspection."
    }
    $healthProperty = $inspection.State.PSObject.Properties['Health']
    $health = if ($null -eq $healthProperty -or $null -eq $healthProperty.Value) { "" } else {
        [string]$healthProperty.Value.Status
    }
    if ([string]$inspection.Id -cne $ExpectedId -or
        [string]$inspection.State.Status -cne "running" -or
        (-not [string]::IsNullOrWhiteSpace($health) -and $health -cne "healthy" -and
            (-not $AllowStarting -or $health -cne "starting"))) {
        throw "Dataset upgrade service '$Service' is not running and healthy."
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

function Get-Sprint8BUpgradeContainerIdentity {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$Service
    )

    $id = ((Invoke-Sprint8BDockerCompose -ComposePath $ComposePath `
        -Arguments @("ps", "--status", "running", "-q", $Service)).output -join "").Trim()
    if ($id -cnotmatch '^[0-9a-f]{64}$') {
        throw "Dataset upgrade requires exactly one running '$Service' container."
    }
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds(60)
    do {
        $inspectionOutput = @(& docker inspect --format '{{json .}}' -- $id 2>&1 |
            ForEach-Object { [string]$_ })
        $inspectionExitCode = $LASTEXITCODE
        $identity = ConvertFrom-Sprint8BUpgradeContainerInspection `
            -InspectionOutput $inspectionOutput -InspectionExitCode $inspectionExitCode `
            -ExpectedId $id -Service $Service -AllowStarting
        if ([string]$identity.health -cne "starting") { return $identity }
        if ([DateTimeOffset]::UtcNow -ge $deadline) {
            throw "Dataset upgrade service '$Service' did not become healthy within 60 seconds."
        }
        Start-Sleep -Seconds 1
    } while ($true)
}

function Invoke-Sprint8BUpgradeRequest {
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
        throw "Dataset upgrade request '$Method $Path' returned HTTP $($response.StatusCode): $($response.Content)"
    }
    $document = if ([string]::IsNullOrWhiteSpace([string]$response.Content)) {
        $null
    } else {
        [string]$response.Content | ConvertFrom-Json -Depth 100
    }
    [pscustomobject][ordered]@{
        status = [int]$response.StatusCode
        document = $document
    }
}

function Get-Sprint8BUpgradeFixtureIdentity {
    param([Parameter(Mandatory)]$Fixture)

    if ([string]$Fixture.sprint -cne "sprint-8b" -or
        [string]$Fixture.state -cne "passed" -or
        [string]$Fixture.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$Fixture.restoration.state -cne "passed") {
        throw "Dataset upgrade fixture receipt is not a healthy owner-controlled Sprint 8B receipt."
    }
    $responseFixtureProof = Assert-Sprint8BPreparedResponseFixtures -FixtureReceipt $Fixture
    $datasets = [ordered]@{}
    foreach ($key in @(
        "dataset.base", "dataset.derived", "dataset.derived-second-hop",
        "dataset.independent-binding", "dataset.disjoint-binding"
    )) {
        $property = $Fixture.logical_identities.datasets.PSObject.Properties[$key]
        if ($null -eq $property -or
            [string]$property.Value.dataset.reference.resource_type -cne "tessara.datasets.dataset" -or
            [string]$property.Value.revision.reference.resource_type -cne "tessara.datasets.dataset_revision" -or
            [string]$property.Value.major_line.reference.resource_type -cne "tessara.datasets.dataset_major_line" -or
            [string]$property.Value.dataset.reference.owner.kind -cne "module_instance") {
            throw "Dataset upgrade fixture omits exact typed read-back for '$key'."
        }
        $datasets[$key] = $property.Value
    }
    [pscustomobject][ordered]@{
        installation_id = [string]$Fixture.installation_id
        datasets = [pscustomobject]$datasets
        components = $Fixture.logical_identities.components
        dashboard = $Fixture.logical_identities.dashboard
        response_fixtures = $responseFixtureProof
    }
}

function Get-Sprint8BUpgradeExecutableSha256 {
    param(
        [Parameter(Mandatory)][string]$ContainerId,
        [string]$Executable = "/usr/local/bin/dataset-module"
    )

    $output = @(& docker exec $ContainerId sha256sum $Executable 2>&1)
    if ($LASTEXITCODE -ne 0 -or $output.Count -ne 1 -or
        [string]$output[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
        throw "Could not capture the live Dataset executable identity."
    }
    $Matches.hash
}

function Get-Sprint8BUpgradeServiceSnapshot {
    param(
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string[]]$Services
    )

    @($Services | ForEach-Object {
        Get-Sprint8BUpgradeContainerIdentity -ComposePath $ComposePath -Service $_
    })
}

function Get-Sprint8BUpgradeModuleProjection {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][Microsoft.PowerShell.Commands.WebRequestSession]$Session,
        [Parameter(Mandatory)][string]$DefinitionId
    )

    $module = (Invoke-Sprint8BUpgradeRequest -BaseUrl $BaseUrl `
        -Path "/api/admin/modules/$DefinitionId" -Session $Session).document
    [pscustomobject][ordered]@{
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

function Get-Sprint8BUpgradeStageSnapshot {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$ExpectedRelease,
        [Parameter(Mandatory)][string]$ExpectedImage,
        [Parameter(Mandatory)][string]$ExpectedManifestDigest
    )

    Wait-Sprint8BHttpProbe -Uri "$($script:upgradePorts.gateway_url)/health" | Out-Null
    Wait-Sprint8BHttpProbe -Uri "$($script:upgradePorts.supervisor_url)/health/ready" `
        -ExpectedStatus @(204) | Out-Null
    $runtime = Get-Sprint8BUpgradeContainerIdentity -ComposePath $script:upgradeComposePath `
        -Service "datasets"
    if ([string]$runtime.image_id -cne $ExpectedImage) {
        throw "$Stage Dataset runtime image '$($runtime.image_id)' is not '$ExpectedImage'."
    }
    $module = Get-Sprint8BUpgradeModuleProjection -BaseUrl $script:upgradePorts.gateway_url `
        -Session $script:upgradeSession -DefinitionId $datasetDefinition
    if ([string]$module.release.version -cne $ExpectedRelease -or
        [string]$module.release.runtime_image -cne $ExpectedImage -or
        [string]$module.release.manifest_digest -cne $ExpectedManifestDigest -or
        [string]$module.diagnostics.public_route -cne "/datasets" -or
        -not [bool]$module.instance.ready -or -not [bool]$module.instance.healthy) {
        throw "$Stage Module Management read-back does not identify the expected healthy Dataset release."
    }
    $normalizedModule = Copy-Sprint8BJsonValue -Value $module
    $normalizedModule.release.version = "<transition-release>"
    $normalizedModule.release.runtime_image = "<transition-image>"
    $normalizedModule.release.manifest_digest = "<transition-manifest>"
    $normalizedModule.release.id = "<transition-release-id>"
    $normalizedModule.manifest.release_version = "<transition-release>"

    $datasetDetails = @($script:upgradeFixture.datasets.PSObject.Properties | ForEach-Object {
        $key = [string]$_.Name
        $id = [string]$_.Value.dataset.reference.resource_id
        [pscustomobject][ordered]@{
            key = $key
            detail = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
                -Path "/api/datasets/$id" -Session $script:upgradeSession).document
            table = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
                -Path "/api/datasets/$id/table" -Session $script:upgradeSession).document
        }
    } | Sort-Object key)
    $datasetList = @((Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/datasets" -Session $script:upgradeSession).document | Sort-Object dataset_id)
    $navigation = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/shell/navigation" -Session $script:upgradeSession).document
    $datasetNavigation = @($navigation.groups | ForEach-Object { $_.items } | Where-Object {
        [string]$_.contribution_id -ceq "tessara.datasets.navigation"
    })
    if ($datasetNavigation.Count -ne 1 -or [string]$datasetNavigation[0].href -cne "/datasets") {
        throw "$Stage Dataset navigation identity is missing or substituted."
    }

    $componentList = @((Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/components" -Session $script:upgradeSession).document | Sort-Object component_id)
    $dashboardList = @((Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/dashboards" -Session $script:upgradeSession).document | Sort-Object id)
    $dashboardId = [string]$script:upgradeFixture.dashboard.id
    $dashboardDetail = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/dashboards/$dashboardId" -Session $script:upgradeSession).document
    $forms = @((Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/forms" -Session $script:upgradeSession).document | Sort-Object id)
    $operations = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/operations/status" -Session $script:upgradeSession).document
    $summary = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/summary" -Session $script:upgradeSession).document
    $unrelatedModules = @(
        "tessara.components", "tessara.dashboards", "tessara.reference.scoped-records" |
            ForEach-Object {
                Get-Sprint8BUpgradeModuleProjection -BaseUrl $script:upgradePorts.gateway_url `
                    -Session $script:upgradeSession -DefinitionId $_
            }
    )
    [pscustomobject][ordered]@{
        stage = $Stage
        captured_at = [DateTimeOffset]::UtcNow.ToString("o")
        dataset = [pscustomobject][ordered]@{
            release = $ExpectedRelease
            runtime_image = $ExpectedImage
            manifest_digest = $ExpectedManifestDigest
            executable_sha256 = Get-Sprint8BUpgradeExecutableSha256 `
                -ContainerId ([string]$runtime.container_id)
            runtime_identity = $runtime
            module = $normalizedModule
            typed_resource_identity = $script:upgradeFixture.datasets
            navigation = $datasetNavigation[0]
            product_list = $datasetList
            product_details_and_tables = $datasetDetails
        }
        unrelated = [pscustomobject][ordered]@{
            services = Get-Sprint8BUpgradeServiceSnapshot -ComposePath $script:upgradeComposePath `
                -Services @(
                    "postgres", "core", "supervisor", "components", "dashboards", "scoped-records",
                    "response-provider-proxy", "form-provider-proxy", "scope-provider-proxy",
                    "principal-provider-proxy", "gateway"
                )
            modules = $unrelatedModules
            component_product = $componentList
            dashboard_product = [pscustomobject][ordered]@{
                list = $dashboardList
                detail = $dashboardDetail
            }
            forms = $forms
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

function New-Sprint8BUpgradeExerciseCatalog {
    param(
        [Parameter(Mandatory)]$RuntimeCatalog,
        [Parameter(Mandatory)]$InitialLockfile,
        [Parameter(Mandatory)]$BaselineMetadata,
        [Parameter(Mandatory)][string]$CandidateManifestDigest,
        [Parameter(Mandatory)][string]$CandidateImageDigest
    )

    $catalog = Copy-Sprint8BJsonValue -Value $RuntimeCatalog
    $catalog.revision = [uint64]$InitialLockfile.blueprint_revision + 1000
    $catalog.issued_at = [DateTimeOffset]::UtcNow.ToString("o")
    $datasetTemplates = @($catalog.module_releases | Where-Object {
        [string]$_.definition_id -ceq $datasetDefinition
    })
    if ($datasetTemplates.Count -ne 1) {
        throw "Runtime release catalog must contain exactly one candidate Dataset release."
    }
    $candidate = Copy-Sprint8BJsonValue -Value $datasetTemplates[0]
    $candidate.version = $candidateRelease
    $candidate.manifest_digest = $CandidateManifestDigest
    $candidate.runtime_image = $CandidateImageDigest
    $baseline = Copy-Sprint8BJsonValue -Value $candidate
    $baseline.version = $baselineRelease
    $baseline.manifest_digest = [string]$BaselineMetadata.release_identity.manifest_digest
    $baseline.runtime_image = [string]$BaselineMetadata.release_identity.runtime_image

    $otherReleases = @($catalog.module_releases | Where-Object {
        [string]$_.definition_id -cne $datasetDefinition
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

function Select-Sprint8BBoundRuntimeCatalog {
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
    $expectedPayload = ConvertTo-Sprint8BStableJson $matches[0].payload
    foreach ($match in @($matches | Select-Object -Skip 1)) {
        if ((ConvertTo-Sprint8BStableJson $match.payload) -cne $expectedPayload) {
            throw "Materialization runtime catalogs collide on one digest with different payloads."
        }
    }
    $matches[0]
}

function Resolve-Sprint8BBoundRuntimeCatalog {
    param(
        [Parameter(Mandatory)][string]$MaterializationRoot,
        [Parameter(Mandatory)][string]$ExpectedCatalogDigest
    )

    $catalogKeyPath = Resolve-Sprint8BRepositoryPath -Path `
        "deploy/sprint-8b/catalogs/catalog-dev-v1.public.hex"
    $candidates = [Collections.Generic.List[object]]::new()
    foreach ($stage in @("noop", "first")) {
        $catalogPath = Join-Path $MaterializationRoot "$stage/release-catalog.signed.json"
        if (-not (Test-Path -LiteralPath $catalogPath -PathType Leaf)) { continue }
        $digestOutput = @(& cargo run -q -p tessara-supervisor --bin tessara-compose -- `
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
        throw "Dataset upgrade cannot locate a signed materialization runtime release catalog."
    }
    Select-Sprint8BBoundRuntimeCatalog -Candidates @($candidates) `
        -ExpectedCatalogDigest $ExpectedCatalogDigest
}

function Invoke-Sprint8BDatasetTransition {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$TargetRelease,
        [Parameter(Mandatory)][string]$TargetImage,
        [Parameter(Mandatory)][string]$TargetManifestDigest
    )

    $summary = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition" -Session $script:upgradeSession).document
    $blueprint = Copy-Sprint8BJsonValue -Value $summary.latest_blueprint
    $blueprint.revision = [uint64]$summary.latest_blueprint.revision + 1
    $selection = @($blueprint.modules | Where-Object {
        [string]$_.definition_id -ceq $datasetDefinition
    })
    if ($selection.Count -ne 1) {
        throw "$Stage Blueprint does not contain exactly one Dataset selection."
    }
    $selection[0].version_requirement = "=$TargetRelease"
    Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints" -Session $script:upgradeSession `
        -Method POST -Body $blueprint -ExpectedStatus @(200, 201) | Out-Null
    $resolved = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints/$($blueprint.revision)/resolve" `
        -Session $script:upgradeSession -Method POST `
        -Body ([ordered]@{ catalog = $script:upgradeCatalog })).document
    $delta = Assert-Sprint8BExactDatasetDeltaPlan -Lockfile $resolved.lockfile `
        -ExpectedImageDigest $TargetImage
    Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints/$($blueprint.revision)/approve" `
        -Session $script:upgradeSession -Method POST -Body ([ordered]@{
            approved_effects = @("install", "upgrade")
            reason = "Sprint 8B Dataset-only $Stage"
        }) | Out-Null
    $applied = (Invoke-Sprint8BUpgradeRequest -BaseUrl $script:upgradePorts.gateway_url `
        -Path "/api/admin/composition/blueprints/$($blueprint.revision)/apply" `
        -Session $script:upgradeSession -Method POST -Body ([ordered]@{}) `
        -ExpectedStatus @(200, 202)).document
    $artifact = $applied.receipt.observed_artifacts.PSObject.Properties[$datasetDefinition]
    if ([string]$applied.operation.state -cne "succeeded" -or
        $null -eq $artifact -or [string]$artifact.Value -cne $TargetImage -or
        (ConvertTo-Sprint8BStableJson @($applied.receipt.bootstrap_receipts | Sort-Object owner)) -cne
            $script:upgradeInitialBootstrapReceipts) {
        throw "$Stage apply did not bind the intended Dataset artifact and preserve bootstrap receipts."
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
    }
}

function Test-Sprint8BDatasetUpgradeHarness {
    $contract = Read-Sprint8BUpgradeContract
    $digest = "sha256:$('a' * 64)"
    $mock = [pscustomobject]@{
        materialization_plan = [pscustomobject]@{
            actions = @(
                [pscustomobject]@{ action = "acquire_image"; component = $datasetDefinition; digest = $digest },
                [pscustomobject]@{ action = "migrate"; owner = $datasetDefinition; image = $digest },
                [pscustomobject]@{ action = "health_gate"; owner = $datasetDefinition },
                [pscustomobject]@{ action = "switch_traffic"; owner = $datasetDefinition },
                [pscustomobject]@{ action = "verify_read_back" }
            )
        }
    }
    Assert-Sprint8BExactDatasetDeltaPlan -Lockfile $mock -ExpectedImageDigest $digest | Out-Null
    $tamperedPlan = Copy-Sprint8BJsonValue -Value $mock
    $tamperedPlan.materialization_plan.actions = @(
        $tamperedPlan.materialization_plan.actions +
            [pscustomobject]@{ action = "configure"; owner = "tessara.components"; digest = "sha256:$('b' * 64)" }
    )
    $rejected = $false
    try {
        Assert-Sprint8BExactDatasetDeltaPlan -Lockfile $tamperedPlan `
            -ExpectedImageDigest $digest | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Dataset upgrade self-test accepted an unrelated owner action." }

    $boundCatalog = Select-Sprint8BBoundRuntimeCatalog -ExpectedCatalogDigest $digest `
        -Candidates @(
            [pscustomobject]@{
                stage = "noop"
                catalog_digest = $digest
                payload = [pscustomobject]@{ api_version = "tessara.io/release-catalog/v1" }
            },
            [pscustomobject]@{
                stage = "first"
                catalog_digest = "sha256:$('b' * 64)"
                payload = [pscustomobject]@{ api_version = "wrong" }
            }
        )
    if ([string]$boundCatalog.stage -cne "noop") {
        throw "Dataset upgrade self-test did not select the current resolved catalog."
    }
    $rejected = $false
    try {
        Select-Sprint8BBoundRuntimeCatalog -ExpectedCatalogDigest "sha256:$('c' * 64)" `
            -Candidates @($boundCatalog) | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Dataset upgrade self-test accepted a catalog outside the current resolved composition."
    }
    $rejected = $false
    try {
        Select-Sprint8BBoundRuntimeCatalog -ExpectedCatalogDigest $digest -Candidates @(
            $boundCatalog,
            [pscustomobject]@{
                stage = "first"
                catalog_digest = $digest
                payload = [pscustomobject]@{ api_version = "digest-collision" }
            }
        ) | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Dataset upgrade self-test accepted conflicting catalog payloads under one digest."
    }

    $containerId = "a" * 64
    $inspection = ConvertFrom-Sprint8BUpgradeContainerInspection -InspectionOutput @(
        "informational Docker output",
        (@{
            Id = $containerId
            Image = "sha256:$('b' * 64)"
            RestartCount = 0
            State = @{ Status = "running"; Health = @{ Status = "healthy" } }
        } | ConvertTo-Json -Compress)
    ) -InspectionExitCode 0 -ExpectedId $containerId -Service "core"
    if ([string]$inspection.container_id -cne $containerId -or
        [string]$inspection.health -cne "healthy") {
        throw "Dataset upgrade self-test did not normalize exact Docker inspection identity."
    }
    $startingInspection = @((@{
        Id = $containerId
        Image = "sha256:$('b' * 64)"
        RestartCount = 0
        State = @{ Status = "running"; Health = @{ Status = "starting" } }
    } | ConvertTo-Json -Compress))
    $rejected = $false
    try {
        ConvertFrom-Sprint8BUpgradeContainerInspection -InspectionOutput $startingInspection `
            -InspectionExitCode 0 -ExpectedId $containerId -Service "datasets" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Dataset upgrade self-test accepted a starting service as final evidence."
    }
    $starting = ConvertFrom-Sprint8BUpgradeContainerInspection `
        -InspectionOutput $startingInspection -InspectionExitCode 0 `
        -ExpectedId $containerId -Service "datasets" -AllowStarting
    if ([string]$starting.health -cne "starting") {
        throw "Dataset upgrade self-test did not classify the bounded starting state."
    }
    $rejected = $false
    try {
        ConvertFrom-Sprint8BUpgradeContainerInspection -InspectionOutput @(
            (@{
                Id = "c" * 64
                Image = "sha256:$('b' * 64)"
                RestartCount = 0
                State = @{ Status = "running"; Health = @{ Status = "healthy" } }
            } | ConvertTo-Json -Compress)
        ) -InspectionExitCode 0 -ExpectedId $containerId -Service "core" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Dataset upgrade self-test accepted a substituted Docker object identity."
    }

    $dataset = [pscustomobject][ordered]@{
        release = $candidateRelease
        runtime_image = $digest
        manifest_digest = $digest
        executable_sha256 = "a" * 64
        runtime_identity = [pscustomobject]@{ container_id = "a" * 64 }
        module = [pscustomobject]@{ route = "/datasets" }
        typed_resource_identity = [pscustomobject]@{ key = "dataset.base" }
        navigation = [pscustomobject]@{ contribution_id = "tessara.datasets.navigation" }
        product_list = @([pscustomobject]@{ dataset_id = "dataset-1" })
        product_details_and_tables = @([pscustomobject]@{ row_count = 1 })
    }
    $unrelated = [pscustomobject][ordered]@{
        services = @([pscustomobject]@{ service = "components"; container_id = "c" * 64; restart_count = 0 })
        modules = @([pscustomobject]@{ definition = "tessara.components"; release = "1.1.0" })
        component_product = @([pscustomobject]@{ component_id = "component-1" })
        dashboard_product = [pscustomobject]@{ id = "dashboard-1" }
        forms = @([pscustomobject]@{ id = "form-1" })
        navigation = [pscustomobject]@{ state = "available" }
        operations = [pscustomobject]@{ dataset = "available" }
        summary = [pscustomobject]@{ datasets = 5 }
        availability = [pscustomobject]@{ gateway = "healthy"; supervisor = "healthy" }
    }
    $before = [pscustomobject][ordered]@{ dataset = $dataset; unrelated = $unrelated }
    $after = Copy-Sprint8BJsonValue -Value $before
    $after.dataset.release = $baselineRelease
    $after.dataset.runtime_image = "sha256:$('d' * 64)"
    $after.dataset.manifest_digest = "sha256:$('e' * 64)"
    $after.dataset.executable_sha256 = "f" * 64
    $after.dataset.runtime_identity = [pscustomobject]@{ container_id = "f" * 64 }
    Assert-Sprint8BUpgradePreservation -Expected $before -Actual $after `
        -Stage "self-test-transition" | Out-Null
    $after.unrelated.services[0].restart_count = 1
    $rejected = $false
    try {
        Assert-Sprint8BUpgradePreservation -Expected $before -Actual $after `
            -Stage "self-test-tamper" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Dataset upgrade self-test accepted unrelated restart drift." }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "independent-dataset-upgrade-rollback-self-test"
        state = "passed"
        self_test = $true
        database_free = $true
        compose_project = "tessara-s8b-upgrade-selftest"
        release_sequence = @($contract.sequence)
        exact_one_owner_delta = "passed"
        runtime_catalog_binding = "passed"
        preservation_rejection = "passed"
        environment_fingerprint_sha256 = Get-Sprint7ASha256 -Text "sprint-8b-dataset-upgrade-self-test`n"
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
    Test-Sprint8BDatasetUpgradeHarness | ConvertTo-Json -Depth 100
    return
}

Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
if (-not $UseExistingTopology) {
    Assert-Sprint8BResetAuthorization -ComposeProject $ComposeProject `
        -Authorized ([bool]$AuthorizeDisposableReset)
}
$source = Get-Sprint8BSourceIdentity -RequireClean
$contract = Read-Sprint8BUpgradeContract
$composePath = Resolve-Sprint8BRepositoryPath -Path $ComposeFile
$candidateManifestFullPath = Resolve-Sprint8BRepositoryPath -Path $CandidateManifestPath
if (-not (Test-Path -LiteralPath $composePath -PathType Leaf) -or
    -not (Test-Path -LiteralPath $candidateManifestFullPath -PathType Leaf)) {
    throw "Dataset upgrade Compose or candidate Manifest input is missing."
}

$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT"
)
$environmentBefore = Get-Sprint8BProcessEnvironmentSnapshot -Names $environmentNames
$childRoot = Resolve-Sprint8BRepositoryPath -Path (
    "target/sprint-8b-dataset-upgrade/$ComposeProject-$([Guid]::NewGuid().ToString('N'))"
)
[IO.Directory]::CreateDirectory($childRoot) | Out-Null
$ownedTopology = $false
$ports = $null
$configuration = $null
$fixturePair = $null
$baselineMetadata = $null
$candidateIdentity = $null
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

try {
    if ($UseExistingTopology) {
        foreach ($name in @(
            "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT", "TESSARA_SUPERVISOR_PORT"
        )) {
            if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
                throw "Existing Dataset upgrade topology requires inherited $name."
            }
        }
        $ports = Set-Sprint8BComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$env:TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$env:TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$env:TESSARA_SUPERVISOR_PORT)
        Assert-Sprint8BExistingTopology -ComposePath $composePath `
            -ComposeProject $ComposeProject | Out-Null
        if ([string]::IsNullOrWhiteSpace($FixtureReceiptPath)) {
            throw "Existing Dataset upgrade topology requires -FixtureReceiptPath."
        }
    } else {
        $materializationEvidence = Join-Path $childRoot "materialization.json"
        $arguments = @(
            "-Target", "ReferenceNoOp", "-ComposeProject", $ComposeProject,
            "-EvidencePath", $materializationEvidence, "-AuthorizeDisposableReset", "-KeepTopology"
        )
        if ($SkipBuild) { $arguments += "-SkipBuild" }
        Invoke-Sprint8BChildScript -ScriptPath "scripts/materialize-sprint-8b.ps1" `
            -Arguments $arguments | Out-Null
        $materialized = Get-Content -LiteralPath $materializationEvidence -Raw | ConvertFrom-Json -Depth 100
        if ([string]$materialized.state -cne "passed" -or $null -eq $materialized.environment -or
            [string]::IsNullOrWhiteSpace([string]$materialized.fixture_receipt_path)) {
            throw "Dataset upgrade materialization did not publish topology and fixture context."
        }
        $ports = Set-Sprint8BComposeEnvironment -ComposeProject $ComposeProject `
            -GatewayPort ([int]$materialized.environment.TESSARA_GATEWAY_PORT) `
            -CorePort ([int]$materialized.environment.TESSARA_CORE_CONTROL_PORT) `
            -SupervisorPort ([int]$materialized.environment.TESSARA_SUPERVISOR_PORT)
        $FixtureReceiptPath = [string]$materialized.fixture_receipt_path
        $ownedTopology = $true
    }

    $resolvedFixturePath = Resolve-Sprint8BRepositoryPath -Path $FixtureReceiptPath
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $resolvedFixturePath `
        -SidecarPath "$resolvedFixturePath.sha256")) {
        throw "Dataset upgrade requires an authenticated fixture receipt pair."
    }
    $fixtureDocument = Get-Content -LiteralPath $resolvedFixturePath -Raw | ConvertFrom-Json -Depth 100
    if ([string]$fixtureDocument.compose_project -cne $ComposeProject) {
        throw "Dataset upgrade fixture receipt belongs to a different Compose project."
    }
    $script:upgradeFixture = Get-Sprint8BUpgradeFixtureIdentity -Fixture $fixtureDocument
    $fixturePair = [pscustomobject][ordered]@{
        path = $resolvedFixturePath
        sha256 = (Get-FileHash -LiteralPath $resolvedFixturePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $configuration = Get-Sprint8BComposeConfiguration -ComposePath $composePath `
        -ComposeProject $ComposeProject
    Assert-Sprint8BDatabaseIsolationConfiguration -ComposeConfiguration $configuration | Out-Null
    $script:upgradeComposePath = $composePath
    $script:upgradePorts = $ports
    $script:upgradeSession = [Microsoft.PowerShell.Commands.WebRequestSession]::new()
    Invoke-Sprint8BUpgradeRequest -BaseUrl $ports.gateway_url -Path "/api/auth/login" `
        -Session $script:upgradeSession -Method POST -Body ([ordered]@{
            email = "admin@tessara.local"
            password = "tessara-dev-admin"
        }) | Out-Null

    $summary = (Invoke-Sprint8BUpgradeRequest -BaseUrl $ports.gateway_url `
        -Path "/api/admin/composition" -Session $script:upgradeSession).document
    if ($null -eq $summary.latest_blueprint -or $null -eq $summary.latest_lockfile -or
        $null -eq $summary.latest_receipt) {
        throw "Dataset upgrade requires an applied candidate composition."
    }
    $initialDataset = @($summary.latest_lockfile.modules | Where-Object {
        [string]$_.definition_id -ceq $datasetDefinition
    })
    if ($initialDataset.Count -ne 1 -or [string]$initialDataset[0].version -cne $candidateRelease) {
        throw "Dataset upgrade topology does not begin at candidate release $candidateRelease."
    }
    $script:upgradeInitialBootstrapReceipts = ConvertTo-Sprint8BStableJson `
        @($summary.latest_receipt.bootstrap_receipts | Sort-Object owner)

    $candidateRuntime = Get-Sprint8BUpgradeContainerIdentity -ComposePath $composePath `
        -Service "datasets"
    $candidateImage = [string]$candidateRuntime.image_id
    if ($candidateImage -cne [string]$initialDataset[0].runtime_image) {
        throw "Live Dataset image differs from the initial resolved candidate lockfile."
    }
    $candidateManifestDigestOutput = @(& cargo run -q -p tessara-supervisor `
        --bin tessara-compose -- manifest-digest $candidateManifestFullPath 2>&1)
    if ($LASTEXITCODE -ne 0) { throw "Could not compute the candidate Dataset Manifest digest." }
    $candidateManifestDigest = [string]($candidateManifestDigestOutput | Select-Object -Last 1)
    if ($candidateManifestDigest -cne [string]$initialDataset[0].manifest_digest) {
        throw "Candidate Dataset Manifest differs from the initial resolved lockfile."
    }
    $candidateExecutable = Get-Sprint8BUpgradeExecutableSha256 `
        -ContainerId ([string]$candidateRuntime.container_id)
    $candidateIdentity = [pscustomobject][ordered]@{
        definition_id = $datasetDefinition
        version = $candidateRelease
        manifest_digest = $candidateManifestDigest
        runtime_image = $candidateImage
        executable_sha256 = $candidateExecutable
    }

    $baselineMetadataPath = Join-Path $childRoot "baseline-metadata.json"
    Invoke-Sprint8BChildScript -ScriptPath "scripts/build-sprint-8b-dataset-upgrade-baseline.ps1" `
        -Arguments @(
            "-OutputTag", $BaselineImageTag,
            "-BaselineRelease", $baselineRelease,
            "-MetadataOutputPath", $baselineMetadataPath
        ) | Out-Null
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $baselineMetadataPath `
        -SidecarPath "$baselineMetadataPath.sha256")) {
        throw "Dataset baseline builder did not publish authenticated release metadata."
    }
    $baselineMetadata = Get-Content -LiteralPath $baselineMetadataPath -Raw | ConvertFrom-Json -Depth 100
    if ([string]$baselineMetadata.receipt_contract -cne
            "tessara.sprint-8b.dataset-prior-compatible-release" -or
        [string]$baselineMetadata.release_identity.definition_id -cne $datasetDefinition -or
        [string]$baselineMetadata.release_identity.version -cne $baselineRelease -or
        [string]$baselineMetadata.source_identity.commit -cne [string]$source.commit -or
        [string]$baselineMetadata.source_identity.tree -cne [string]$source.tree -or
        [bool]$baselineMetadata.source_identity.dirty -or
        [string]$baselineMetadata.release_identity.runtime_image -ceq $candidateImage -or
        [string]$baselineMetadata.release_identity.executable_sha256 -ceq $candidateExecutable) {
        throw "Dataset baseline metadata is not a distinct source-bound compatible 0.9.0 release."
    }

    $boundRuntimeCatalog = Resolve-Sprint8BBoundRuntimeCatalog `
        -MaterializationRoot (Split-Path -Parent $resolvedFixturePath) `
        -ExpectedCatalogDigest ([string]$summary.latest_lockfile.catalog_digest)
    $runtimeCatalog = $boundRuntimeCatalog.payload
    $script:upgradeCatalog = New-Sprint8BUpgradeExerciseCatalog `
        -RuntimeCatalog $runtimeCatalog -InitialLockfile $summary.latest_lockfile `
        -BaselineMetadata $baselineMetadata -CandidateManifestDigest $candidateManifestDigest `
        -CandidateImageDigest $candidateImage

    $preExercise = Get-Sprint8BUpgradeStageSnapshot -Stage "pre-exercise-candidate" `
        -ExpectedRelease $candidateRelease -ExpectedImage $candidateImage `
        -ExpectedManifestDigest $candidateManifestDigest
    $stages.Add($preExercise)
    $sequence = @(
        [pscustomobject]@{
            stage = "establish-compatible-baseline"
            release = $baselineRelease
            image = [string]$baselineMetadata.release_identity.runtime_image
            manifest = [string]$baselineMetadata.release_identity.manifest_digest
        },
        [pscustomobject]@{
            stage = "upgrade-to-candidate"
            release = $candidateRelease
            image = $candidateImage
            manifest = $candidateManifestDigest
        },
        [pscustomobject]@{
            stage = "rollback-to-compatible-baseline"
            release = $baselineRelease
            image = [string]$baselineMetadata.release_identity.runtime_image
            manifest = [string]$baselineMetadata.release_identity.manifest_digest
        },
        [pscustomobject]@{
            stage = "restore-intended-candidate"
            release = $candidateRelease
            image = $candidateImage
            manifest = $candidateManifestDigest
        }
    )
    foreach ($target in $sequence) {
        $transitions.Add((Invoke-Sprint8BDatasetTransition -Stage ([string]$target.stage) `
            -TargetRelease ([string]$target.release) -TargetImage ([string]$target.image) `
            -TargetManifestDigest ([string]$target.manifest)))
        $snapshot = Get-Sprint8BUpgradeStageSnapshot -Stage ([string]$target.stage) `
            -ExpectedRelease ([string]$target.release) -ExpectedImage ([string]$target.image) `
            -ExpectedManifestDigest ([string]$target.manifest)
        $preservation.Add((Assert-Sprint8BUpgradePreservation -Expected $preExercise `
            -Actual $snapshot -Stage ([string]$target.stage)))
        $stages.Add($snapshot)
    }
    Assert-Sprint8BExactSequence -Expected @($contract.sequence) `
        -Actual @($transitions.target_release) -Label "Executed Dataset release sequence"
    $releaseRestoration.state = "passed"
    $releaseRestoration.final_release = $candidateRelease
} catch {
    $failure = $_
} finally {
    if ($null -ne $ports -and $null -ne $script:upgradeSession -and
        $null -ne $candidateIdentity -and $null -ne $script:upgradeCatalog) {
        try {
            $projected = Get-Sprint8BUpgradeModuleProjection -BaseUrl $ports.gateway_url `
                -Session $script:upgradeSession -DefinitionId $datasetDefinition
            if ([string]$projected.release.version -cne $candidateRelease) {
                $releaseRestoration.attempted = $true
                Invoke-Sprint8BDatasetTransition -Stage "failure-restoration" `
                    -TargetRelease $candidateRelease -TargetImage ([string]$candidateIdentity.runtime_image) `
                    -TargetManifestDigest ([string]$candidateIdentity.manifest_digest) | Out-Null
            }
            $restored = Get-Sprint8BUpgradeModuleProjection -BaseUrl $ports.gateway_url `
                -Session $script:upgradeSession -DefinitionId $datasetDefinition
            if ([string]$restored.release.version -cne $candidateRelease -or
                -not [bool]$restored.instance.ready -or -not [bool]$restored.instance.healthy) {
                throw "Dataset release did not restore to the healthy intended candidate."
            }
            $releaseRestoration.state = "passed"
            $releaseRestoration.final_release = $candidateRelease
        } catch {
            if ($null -eq $failure) { $failure = $_ }
            $releaseRestoration.state = "failed"
            $releaseRestoration.error = $_.Exception.Message
        }
    }
    try {
        if ($ownedTopology) {
            $cleanup = Remove-Sprint8BProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized ([bool]$AuthorizeDisposableReset)
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
            $cleanup | Add-Member -NotePropertyName mode -NotePropertyValue "exact-project-teardown" -Force
        } elseif ($UseExistingTopology) {
            Assert-Sprint8BExistingTopology -ComposePath $composePath `
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
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$configurationHash = if ($null -eq $configuration) { "none" } else {
    Get-Sprint7ASha256 -Text ((ConvertTo-Sprint8BStableJson $configuration) + "`n")
}
$fixtureHash = if ($null -eq $fixturePair) { "none" } else { [string]$fixturePair.sha256 }
$baselineHash = if ($null -eq $baselineMetadata) { "none" } else {
    Get-Sprint7ASha256 -Text ((ConvertTo-Sprint8BStableJson $baselineMetadata) + "`n")
}
$environmentFingerprint = Get-Sprint7ASha256 -Text (
    "$($source.commit)`n$($source.tree)`n$ComposeProject`n$configurationHash`n$fixtureHash`n$baselineHash`n"
)
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    proof = "independent-dataset-upgrade-rollback-restoration"
    state = if ($null -eq $failure -and [string]$releaseRestoration.state -ceq "passed" -and
        [string]$cleanup.state -ceq "passed") { "passed" } else { "failed" }
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    release_contract = $contract
    release_fixture = [pscustomobject][ordered]@{
        baseline = if ($null -eq $baselineMetadata) { $null } else { $baselineMetadata.release_identity }
        candidate = $candidateIdentity
    }
    fixture_receipt = $fixturePair
    transitions = @($transitions)
    stage_snapshots = @($stages)
    preservation_proofs = @($preservation)
    release_restoration = $releaseRestoration
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
        message = $failure.Exception.Message
        category = [string]$failure.CategoryInfo.Category
    } }
}
$evidenceFullPath = Resolve-Sprint8BRepositoryPath -Path $EvidencePath
Publish-Sprint8BHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Sprint 8B Dataset upgrade/rollback failed; retained evidence: $evidenceFullPath"
}

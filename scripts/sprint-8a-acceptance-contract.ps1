Set-StrictMode -Version Latest

$script:Sprint8AFixture = [ordered]@{
    installation_id = "01980000-0000-7000-8000-00000000008a"
    component_module_instance_id = "142a1ece-f74b-85f6-8ca0-92f4a02e9409"
    component_resource_type = "tessara.components.component_version"
    dataset_id = "01980000-0002-7000-8000-000000000003"
    dashboard_id = "01980000-0003-7000-8000-000000000001"
    table_placement_id = "01980000-0003-7000-8000-000000000003"
    stat_placement_id = "01980000-0003-7000-8000-000000000002"
    inactive_stat_card_version_id = "01980000-0001-7000-8000-000000000001"
    blocked_component_version_id = "01980000-0001-7000-8000-000000000004"
    component_versions = [ordered]@{
        table = "01980000-0001-7000-8000-000000000002"
        bar = "01980000-0001-7000-8000-000000000003"
        line = "01980000-0001-7000-8000-000000000012"
        pie = "01980000-0001-7000-8000-000000000013"
        donut = "01980000-0001-7000-8000-000000000014"
        stat_card = "01980000-0001-7000-8000-000000000011"
    }
    dashboard_placements = [ordered]@{
        "01980000-0003-7000-8000-000000000002" = [ordered]@{ placement_key = "row-count"; resource_key = "sprint-8a-row-count"; component_version_id = "01980000-0001-7000-8000-000000000011"; grid_row = 1; grid_column = 1; grid_width = 4; grid_height = 2; disclosure = "authorized"; resolution_state = "available"; availability = "available" }
        "01980000-0003-7000-8000-000000000003" = [ordered]@{ placement_key = "records"; resource_key = "sprint-8a-record-table"; component_version_id = "01980000-0001-7000-8000-000000000002"; grid_row = 3; grid_column = 1; grid_width = 12; grid_height = 6; disclosure = "authorized"; resolution_state = "available"; availability = "available" }
        "01980000-0003-7000-8000-000000000004" = [ordered]@{ placement_key = "tier-chart"; resource_key = "sprint-8a-label-bar"; component_version_id = "01980000-0001-7000-8000-000000000003"; grid_row = 9; grid_column = 1; grid_width = 6; grid_height = 4; disclosure = "authorized"; resolution_state = "available"; availability = "available" }
        "01980000-0003-7000-8000-000000000005" = [ordered]@{ placement_key = "blocked-scope"; resource_key = "sprint-8a-blocked-component"; component_version_id = "01980000-0001-7000-8000-000000000004"; grid_row = 9; grid_column = 7; grid_width = 6; grid_height = 4; disclosure = "restricted"; resolution_state = "restricted"; availability = "unavailable" }
        "01980000-0003-7000-8000-000000000006" = [ordered]@{ placement_key = "lifecycle-upgrade"; resource_key = "sprint-8a-row-count-inactive"; component_version_id = "01980000-0001-7000-8000-000000000001"; grid_row = 13; grid_column = 1; grid_width = 4; grid_height = 2; disclosure = "authorized"; resolution_state = "inactive"; availability = "unavailable" }
        "01980000-0003-7000-8000-000000000007" = [ordered]@{ placement_key = "lifecycle-replace"; resource_key = "sprint-8a-row-count-inactive"; component_version_id = "01980000-0001-7000-8000-000000000001"; grid_row = 13; grid_column = 5; grid_width = 4; grid_height = 2; disclosure = "authorized"; resolution_state = "inactive"; availability = "unavailable" }
        "01980000-0003-7000-8000-000000000008" = [ordered]@{ placement_key = "lifecycle-remove"; resource_key = "sprint-8a-row-count-inactive"; component_version_id = "01980000-0001-7000-8000-000000000001"; grid_row = 13; grid_column = 9; grid_width = 4; grid_height = 2; disclosure = "authorized"; resolution_state = "inactive"; availability = "unavailable" }
    }
}

function Test-Sprint8AFirstPartyComponentContractSources {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot
    )

    $sourceContracts = [ordered]@{
        "end2end/tests/permissions.spec.ts" = [ordered]@{
            forbidden = @(
                'function\s+canonicalComponentVersionInput',
                'function\s+canonicalComponentRequest',
                'function\s+componentResponseAliases',
                '(?m)^\s*missing_policy\s*:',
                '(?m)^\s*dataset_version_major\s*:',
                'versions\[0\]\s+as\s+\{\s*id:\s*string\s*\}'
            )
            required = @(
                '/api/admin/components/datasets',
                'dataset_reference: datasetReference',
                'component_id: string;',
                'component_version_id: string;'
            )
        }
        "scripts/smoke.ps1" = [ordered]@{
            forbidden = @(
                '(?m)^\s*missing_policy\s*=',
                '(?m)^\s*dataset_version_major\s*=',
                '\$visualComponent\.id',
                '\$visualVersion\.id'
            )
            required = @(
                'dataset_reference = $visualDatasetReference',
                '$visualComponent.component_id',
                '$visualVersion.component_version_id'
            )
        }
        "scripts/uat-sprint.ps1" = [ordered]@{
            forbidden = @(
                '(?m)^\s*missing_policy\s*=',
                '(?m)^\s*dataset_version_major\s*=',
                '\$visualCreated\.id',
                '\$visualVersion\.id'
            )
            required = @(
                'dataset_reference = $visualDatasetReference',
                '$visualCreated.component_id',
                '$visualVersion.component_version_id'
            )
        }
        "crates/tessara-component-module/assets/component.js" = [ordered]@{
            forbidden = @(
                '(?m)^\s*missing_policy\s*:',
                'config\.missing_policy'
            )
            required = @(
                'value_missing_policy: configControl(form, "value_missing_policy").value'
            )
        }
        "end2end/tests/components.spec.ts" = [ordered]@{
            forbidden = @(
                '(?m)^\s*missing_policy\s*:',
                '(?m)^\s*dataset_version_major\s*:'
            )
            required = @(
                'dataset_reference: DatasetReference;',
                'component_id: string;',
                'component_version_id: string;',
                'value_missing_policy: "omit"'
            )
        }
        "end2end/tests/analytics-sprint-7a.spec.ts" = [ordered]@{
            forbidden = @(
                '(?m)^\s*missing_policy\s*:',
                '(?m)^\s*dataset_version_major\s*:'
            )
            required = @(
                'component_id: string;',
                'component_version_id: string;',
                'component_version_id: fixture.metricComponentVersionId'
            )
        }
        "end2end/tests/dashboards.spec.ts" = [ordered]@{
            forbidden = @(
                '(?m)^\s*missing_policy\s*:',
                '(?m)^\s*dataset_version_major\s*:'
            )
            required = @(
                'component_version_id: string;',
                'placement.component?.component_version_id ===',
                'editorOption!.component_version_id'
            )
        }
        "deploy/sprint-8a/blueprints/reference.json" = [ordered]@{
            forbidden = @(
                '"missing_policy"\s*:'
            )
            required = @()
        }
    }
    foreach ($sourcePath in $sourceContracts.Keys) {
        $sourceText = Get-Content -LiteralPath (Join-Path $RepoRoot $sourcePath) -Raw
        foreach ($pattern in $sourceContracts[$sourcePath].forbidden) {
            if ($sourceText -match $pattern) {
                throw "Sprint 8A first-party acceptance source '$sourcePath' retains a legacy Component payload or response alias matching '$pattern'."
            }
        }
        foreach ($fragment in $sourceContracts[$sourcePath].required) {
            if (-not $sourceText.Contains($fragment)) {
                throw "Sprint 8A first-party acceptance source '$sourcePath' omits canonical Component contract fragment '$fragment'."
            }
        }
    }
}

function Get-Sprint8AExpectedDeploymentTargets {
    [ordered]@{
        "core" = [ordered]@{
            image_environment = "TESSARA_CORE_IMAGE"
            migration_service = "core-migrate"
            runtime_service = "core"
            image_template = '${TESSARA_CORE_IMAGE:-tessara-sprint-8a-core}'
        }
        "tessara.components" = [ordered]@{
            image_environment = "TESSARA_COMPONENT_IMAGE"
            migration_service = "components-migrate"
            runtime_service = "components"
            image_template = '${TESSARA_COMPONENT_IMAGE:-tessara-sprint-8a-components}'
        }
        "tessara.dashboards" = [ordered]@{
            image_environment = "TESSARA_DASHBOARD_IMAGE"
            migration_service = "dashboards-migrate"
            runtime_service = "dashboards"
            image_template = '${TESSARA_DASHBOARD_IMAGE:-tessara-sprint-8a-dashboards}'
        }
        "tessara.reference.scoped-records" = [ordered]@{
            image_environment = "TESSARA_REFERENCE_MODULE_IMAGE"
            migration_service = "scoped-records-migrate"
            runtime_service = "scoped-records"
            image_template = '${TESSARA_REFERENCE_MODULE_IMAGE:-tessara-sprint-7a-scoped-records}'
        }
    }
}

function Get-Sprint8AExpectedDeploymentTargetMap {
    $targets = Get-Sprint8AExpectedDeploymentTargets
    $map = [ordered]@{}
    foreach ($owner in $targets.Keys) {
        $target = $targets[$owner]
        $map[$owner] = [ordered]@{
            image_environment = [string]$target.image_environment
            migration_service = [string]$target.migration_service
            runtime_service = [string]$target.runtime_service
        }
    }
    $map
}

function Assert-Sprint8ADeploymentTargetContract {
    param([Parameter(Mandatory)][string]$ComposeText)

    $targetPattern = "(?m)^      TESSARA_DEPLOYMENT_TARGETS:\s+'(?<json>\{[^\r\n]+\})'\s*$"
    $targetMatches = @([regex]::Matches($ComposeText, $targetPattern))
    if ($targetMatches.Count -ne 1) {
        throw "Sprint 8A Compose must declare exactly one single-line deployment-target map."
    }
    try {
        $actualTargets = ConvertFrom-Json -InputObject $targetMatches[0].Groups["json"].Value -ErrorAction Stop
    } catch {
        throw "Sprint 8A deployment-target map is not valid JSON: $($_.Exception.Message)"
    }
    $expectedTargets = Get-Sprint8AExpectedDeploymentTargets
    $actualOwners = @($actualTargets.PSObject.Properties.Name | Sort-Object)
    $expectedOwners = @($expectedTargets.Keys | Sort-Object)
    if (($actualOwners -join "`n") -cne ($expectedOwners -join "`n")) {
        throw "Sprint 8A deployment-target map does not contain the exact four canonical owners."
    }

    foreach ($owner in $expectedTargets.Keys) {
        $actualTargetProperty = $actualTargets.PSObject.Properties[[string]$owner]
        if ($null -eq $actualTargetProperty) {
            throw "Sprint 8A deployment target '$owner' is missing."
        }
        $actualTarget = $actualTargetProperty.Value
        $expectedTarget = $expectedTargets[$owner]
        $actualFields = @($actualTarget.PSObject.Properties.Name | Sort-Object)
        $expectedFields = @("image_environment", "migration_service", "runtime_service") | Sort-Object
        if (($actualFields -join "`n") -cne ($expectedFields -join "`n") -or
            [string]$actualTarget.image_environment -cne [string]$expectedTarget.image_environment -or
            [string]$actualTarget.migration_service -cne [string]$expectedTarget.migration_service -or
            [string]$actualTarget.runtime_service -cne [string]$expectedTarget.runtime_service) {
            throw "Sprint 8A deployment target '$owner' differs from its exact image and service mapping."
        }

        foreach ($service in @([string]$expectedTarget.migration_service, [string]$expectedTarget.runtime_service)) {
            $servicePattern = "(?ms)^  $([regex]::Escape($service)):\r?\n(?<body>.*?)(?=^  [a-zA-Z0-9][a-zA-Z0-9-]*:\r?\n|\z)"
            $serviceMatch = [regex]::Match($ComposeText, $servicePattern)
            if (-not $serviceMatch.Success) {
                throw "Sprint 8A deployment target '$owner' references missing service '$service'."
            }
            $imagePattern = "(?m)^    image:\s+$([regex]::Escape([string]$expectedTarget.image_template))\s*$"
            if (@([regex]::Matches($serviceMatch.Groups["body"].Value, $imagePattern)).Count -ne 1) {
                throw "Sprint 8A service '$service' is not wired to '$($expectedTarget.image_environment)'."
            }
        }
    }
}

function Set-Sprint8ADeploymentTargetFixture {
    param(
        [Parameter(Mandatory)][string]$ComposeText,
        [Parameter(Mandatory)]$Targets
    )
    $pattern = [regex]::new("(?m)^      TESSARA_DEPLOYMENT_TARGETS:\s+'\{[^\r\n]+\}'\s*$")
    $replacement = "      TESSARA_DEPLOYMENT_TARGETS: '$($Targets | ConvertTo-Json -Depth 10 -Compress)'"
    $pattern.Replace($ComposeText, $replacement, 1)
}

function Assert-Sprint8ADeploymentTargetFixtureRejected {
    param(
        [Parameter(Mandatory)][string]$ComposeText,
        [Parameter(Mandatory)][string]$Label
    )
    $rejected = $false
    try {
        Assert-Sprint8ADeploymentTargetContract -ComposeText $ComposeText
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Sprint 8A deployment-target adversarial self-test accepted $Label."
    }
}

function Test-Sprint8ADeploymentTargetContract {
    param([Parameter(Mandatory)][string]$ComposeText)

    Assert-Sprint8ADeploymentTargetContract -ComposeText $ComposeText
    $expectedTargets = Get-Sprint8AExpectedDeploymentTargetMap

    $missingOwner = $expectedTargets | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $missingOwner.PSObject.Properties.Remove("core")
    Assert-Sprint8ADeploymentTargetFixtureRejected `
        -ComposeText (Set-Sprint8ADeploymentTargetFixture -ComposeText $ComposeText -Targets $missingOwner) `
        -Label "a missing Core target"

    $extraOwner = $expectedTargets | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $extraOwner | Add-Member -NotePropertyName "tessara.unexpected" -NotePropertyValue ([pscustomobject]@{
        image_environment = "TESSARA_UNEXPECTED_IMAGE"
        migration_service = "unexpected-migrate"
        runtime_service = "unexpected"
    })
    Assert-Sprint8ADeploymentTargetFixtureRejected `
        -ComposeText (Set-Sprint8ADeploymentTargetFixture -ComposeText $ComposeText -Targets $extraOwner) `
        -Label "an extra owner target"

    $wrongTarget = $expectedTargets | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $wrongTarget.'tessara.dashboards'.image_environment = "TESSARA_COMPONENT_IMAGE"
    Assert-Sprint8ADeploymentTargetFixtureRejected `
        -ComposeText (Set-Sprint8ADeploymentTargetFixture -ComposeText $ComposeText -Targets $wrongTarget) `
        -Label "a Dashboard target using the Component image environment"

    $hardCodedImage = $ComposeText.Replace(
        '    image: ${TESSARA_DASHBOARD_IMAGE:-tessara-sprint-8a-dashboards}',
        '    image: tessara-sprint-8a-dashboards'
    )
    Assert-Sprint8ADeploymentTargetFixtureRejected `
        -ComposeText $hardCodedImage `
        -Label "hard-coded Dashboard migration and runtime images"
}

function Test-Sprint8AAcceptanceContract {
    $repoRoot = Split-Path -Parent $PSScriptRoot
    foreach ($name in @(
        "installation_id",
        "component_module_instance_id",
        "dataset_id",
        "dashboard_id",
        "table_placement_id",
        "stat_placement_id",
        "inactive_stat_card_version_id",
        "blocked_component_version_id"
    )) {
        try {
            $null = [guid]::ParseExact([string]$script:Sprint8AFixture[$name], "D")
        } catch {
            throw "Sprint 8A acceptance fixture '$name' is not a canonical UUID."
        }
    }
    $expectedKinds = @("bar", "donut", "line", "pie", "stat_card", "table")
    $actualKinds = @($script:Sprint8AFixture.component_versions.Keys | Sort-Object)
    if (($actualKinds -join ",") -cne ($expectedKinds -join ",")) {
        throw "Sprint 8A acceptance fixture does not define every canonical Component kind."
    }
    foreach ($versionId in $script:Sprint8AFixture.component_versions.Values) {
        try {
            $null = [guid]::ParseExact([string]$versionId, "D")
        } catch {
            throw "Sprint 8A ComponentVersion fixture '$versionId' is not a canonical UUID."
        }
    }
    if ($script:Sprint8AFixture.dashboard_placements.Count -ne 7) {
        throw "Sprint 8A acceptance fixture must define the exact seven Dashboard placement identities."
    }
    foreach ($placementId in $script:Sprint8AFixture.dashboard_placements.Keys) {
        $placement = $script:Sprint8AFixture.dashboard_placements[$placementId]
        try {
            $null = [guid]::ParseExact([string]$placementId, "D")
            $null = [guid]::ParseExact([string]$placement.component_version_id, "D")
        } catch {
            throw "Sprint 8A Dashboard placement fixture '$placementId' is not canonical."
        }
        if ([string]::IsNullOrWhiteSpace([string]$placement.placement_key) -or
            [string]::IsNullOrWhiteSpace([string]$placement.resource_key) -or
            [int]$placement.grid_row -lt 1 -or [int]$placement.grid_column -lt 1 -or
            [int]$placement.grid_width -lt 1 -or [int]$placement.grid_height -lt 1 -or
            @("authorized", "restricted") -cnotcontains [string]$placement.disclosure -or
            @("available", "inactive", "restricted") -cnotcontains [string]$placement.resolution_state -or
            @("available", "unavailable") -cnotcontains [string]$placement.availability) {
            throw "Sprint 8A Dashboard placement fixture '$placementId' lacks exact owner binding identities."
        }
    }
    if ($script:Sprint8AFixture.component_resource_type -cne "tessara.components.component_version") {
        throw "Sprint 8A Component resource type drifted."
    }

    $configuration = & docker compose -f (Join-Path $repoRoot "deploy/sprint-8a/compose.yaml") --profile reference config --format json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw "Sprint 8A Compose configuration could not be normalized." }
    $artifactImages = [string]$configuration.services.supervisor.environment.TESSARA_ARTIFACT_IMAGE_REFERENCES | ConvertFrom-Json
    $moduleImages = [ordered]@{
        "tessara.reference.scoped-records" = [string]$configuration.services.'scoped-records'.image
        "tessara.components" = [string]$configuration.services.components.image
        "tessara.dashboards" = [string]$configuration.services.dashboards.image
    }
    foreach ($moduleId in $moduleImages.Keys) {
        $artifactProperty = $artifactImages.PSObject.Properties[$moduleId]
        $references = if ($null -eq $artifactProperty) { @() } else { @($artifactProperty.Value) }
        $expectedReferences = if ($moduleId -ceq "tessara.components") {
            @($moduleImages[$moduleId], "tessara-sprint-8a-components-rehearsal-baseline:latest") | Sort-Object
        } else {
            @($moduleImages[$moduleId])
        }
        if ((@($references | Sort-Object) -join "`n") -cne ($expectedReferences -join "`n")) {
            throw "Sprint 8A Supervisor image reference for '$moduleId' differs from normalized Compose."
        }
    }
    $componentHealthcheck = @($configuration.services.components.healthcheck.test)
    if (($componentHealthcheck -join " ") -cne "CMD curl -fsS http://127.0.0.1:8092/health/ready") {
        throw "Sprint 8A Component runtime must declare the exact readiness healthcheck used by upgrade/rollback."
    }

    $componentManifest = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-component-module/manifest.json") -Raw | ConvertFrom-Json
    $expectedComponentProvidedActions = @(
        "components.catalog|POST|/api/private/components/catalog|read|components:read",
        "components.render|POST|/api/private/components/render|read|components:read",
        "components.resolve|POST|/api/private/components/resolve|read|components:read"
    ) | Sort-Object
    $actualComponentProvidedActions = @(
        $componentManifest.provided_service_actions | ForEach-Object {
            "$($_.authorization_action)|$($_.method)|$($_.path)|$($_.operation)|$($_.required_capability)"
        } | Sort-Object
    )
    $expectedComponentConsumedActions = @(
        "datasets.bootstrap_validate", "datasets.catalog", "datasets.compatibility",
        "datasets.distinct_values", "datasets.execute", "datasets.schema"
    ) | Sort-Object
    $actualComponentConsumedActions = @(
        $componentManifest.consumed_service_actions | ForEach-Object {
            if ([string]$_.dependency_binding -cne "tessara.components.dataset-major-line" -or
                [string]$_.functional_contract -cne "tessara.datasets.dataset-major-line") {
                throw "Component consumed service action is not bound to its exact Dataset dependency."
            }
            [string]$_.authorization_action
        } | Sort-Object
    )
    if (($actualComponentProvidedActions -join ",") -cne ($expectedComponentProvidedActions -join ",") -or
        ($actualComponentConsumedActions -join ",") -cne ($expectedComponentConsumedActions -join ",")) {
        throw "Component Manifest service-action declarations differ from the exact Sprint 8A boundary."
    }
    $bootstrapValidation = $componentManifest.bootstrap_dependency_validation
    if ($null -eq $bootstrapValidation -or
        [string]$bootstrapValidation.dependency_binding -cne "tessara.components.dataset-major-line" -or
        [string]$bootstrapValidation.functional_contract -cne "tessara.datasets.dataset-major-line" -or
        [string]$bootstrapValidation.contract_version -cne "1.0.0" -or
        [string]$bootstrapValidation.authorization_action -cne "datasets.bootstrap_validate" -or
        [string]$bootstrapValidation.method -cne "POST" -or
        [string]$bootstrapValidation.path -cne "/api/private/datasets/bootstrap-validation" -or
        [string]$bootstrapValidation.audience -cne "resolved_dependency_provider" -or
        [string]$bootstrapValidation.payload_pointer -cne "/dependency_validation") {
        throw "Component Manifest does not declare the exact lockfile-owned Dataset bootstrap validation target."
    }
    foreach ($manifestPath in @(
        "crates/tessara-dashboard-module/manifest.json",
        "crates/tessara-reference-scoped-records/manifest.json"
    )) {
        $manifest = Get-Content -LiteralPath (Join-Path $repoRoot $manifestPath) -Raw | ConvertFrom-Json
        if ($null -ne $manifest.PSObject.Properties["bootstrap_dependency_validation"]) {
            throw "Module Manifest '$manifestPath' opts into bootstrap dependency validation without a Sprint 8A requirement."
        }
    }
    $componentAssets = [ordered]@{
        "/component.css" = @(
            "crates/tessara-module-ui/assets/module-shell.css",
            "crates/tessara-component-module/assets/component.css"
        )
        "/component-lifecycle.css" = @(
            "crates/tessara-component-module/assets/component.css",
            "crates/tessara-component-module/assets/component-lifecycle.css"
        )
        "/component.js" = @("crates/tessara-component-module/assets/component.js")
    }
    foreach ($assetPath in $componentAssets.Keys) {
        $declaration = @($componentManifest.assets | Where-Object path -CEQ $assetPath)
        if ($declaration.Count -ne 1) {
            throw "Component manifest must declare '$assetPath' exactly once."
        }
        $assetStream = [IO.MemoryStream]::new()
        try {
            $sourcePaths = @($componentAssets[$assetPath])
            for ($index = 0; $index -lt $sourcePaths.Count; $index++) {
                $sourceBytes = [IO.File]::ReadAllBytes((Join-Path $repoRoot $sourcePaths[$index]))
                $assetStream.Write($sourceBytes, 0, $sourceBytes.Length)
                if ($index -lt ($sourcePaths.Count - 1)) {
                    # These assets are served from Rust concat!(include_str!(...), "\n", ...).
                    $assetStream.WriteByte(10)
                }
            }
            $sha256 = [Security.Cryptography.SHA256]::Create()
            try {
                $actualDigest = "sha256:$(-join ($sha256.ComputeHash($assetStream.ToArray()) | ForEach-Object { $_.ToString('x2') }))"
            } finally {
                $sha256.Dispose()
            }
        } finally {
            $assetStream.Dispose()
        }
        if ([string]$declaration[0].digest -cne $actualDigest) {
            throw "Component asset '$assetPath' differs from its manifest digest."
        }
    }
    $componentDockerfile = Get-Content -LiteralPath (Join-Path $repoRoot "Dockerfile.component") -Raw
    foreach ($fragment in @(
        "cargo build --release -p tessara-component-module",
        'org.opencontainers.image.revision="$TESSARA_SOURCE_COMMIT"',
        'com.tessara.source-tree="$TESSARA_SOURCE_TREE"',
        'com.tessara.source-dirty="$TESSARA_SOURCE_DIRTY"',
        'com.tessara.module-definition="tessara.components"',
        'COPY --from=builder /tmp/component-module /usr/local/bin/component-module'
    )) {
        if (-not $componentDockerfile.Contains($fragment)) {
            throw "Component image contract omits '$fragment'."
        }
    }
    $componentValidationText = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-component-module/src/validation.rs") -Raw
    $componentProviderText = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-component-module/src/provider.rs") -Raw
    $componentJavaScriptText = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-component-module/assets/component.js") -Raw
    foreach ($retiredReader in @(
        [pscustomobject]@{ source = "validation"; text = $componentValidationText; fragment = '#[serde(alias = "field")]' },
        [pscustomobject]@{ source = "validation"; text = $componentValidationText; fragment = "enum ComponentFieldRef" },
        [pscustomobject]@{ source = "provider"; text = $componentProviderText; fragment = '.get("field")' },
        [pscustomobject]@{ source = "browser"; text = $componentJavaScriptText; fragment = "item.field_key || item.key || item.field" },
        [pscustomobject]@{ source = "browser"; text = $componentJavaScriptText; fragment = "filter.field_key || filter.field" },
        [pscustomobject]@{ source = "validation"; text = $componentValidationText; fragment = "missing_policy: String"; pattern = '(?m)^\s*missing_policy:\s*String' },
        [pscustomobject]@{ source = "provider"; text = $componentProviderText; fragment = '.get("missing_policy")' },
        [pscustomobject]@{ source = "browser"; text = $componentJavaScriptText; fragment = "`n    missing_policy:" },
        [pscustomobject]@{ source = "browser"; text = $componentJavaScriptText; fragment = "config.missing_policy" }
    )) {
        $retiredPresent = if ($retiredReader.PSObject.Properties.Name -contains "pattern") {
            $retiredReader.text -match [string]$retiredReader.pattern
        } else {
            $retiredReader.text.Contains($retiredReader.fragment)
        }
        if ($retiredPresent) {
            throw "Component $($retiredReader.source) source retains retired configuration reader '$($retiredReader.fragment)'."
        }
    }
    foreach ($canonicalReader in @(
        [pscustomobject]@{ source = "validation"; text = $componentValidationText; fragment = "visible_columns: Vec<String>" },
        [pscustomobject]@{ source = "provider"; text = $componentProviderText; fragment = '.get("field_key")' },
        [pscustomobject]@{ source = "browser"; text = $componentJavaScriptText; fragment = 'const storedField = filter.field_key || "";' },
        [pscustomobject]@{ source = "validation"; text = $componentValidationText; fragment = "value_missing_policy: String" },
        [pscustomobject]@{ source = "provider"; text = $componentProviderText; fragment = '.get("value_missing_policy")' },
        [pscustomobject]@{ source = "browser"; text = $componentJavaScriptText; fragment = 'value_missing_policy: configControl(form, "value_missing_policy").value' }
    )) {
        if (-not $canonicalReader.text.Contains($canonicalReader.fragment)) {
            throw "Component $($canonicalReader.source) source omits canonical configuration reader '$($canonicalReader.fragment)'."
        }
    }

    $dashboardManifest = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-dashboard-module/manifest.json") -Raw | ConvertFrom-Json
    if ((@($dashboardManifest.deployment.declaration.runtime_image.command) -join "|") -cne
            "/usr/local/bin/dashboard-module|serve" -or
        (@($dashboardManifest.deployment.declaration.migration_image.command) -join "|") -cne
            "/usr/local/bin/dashboard-module|migrate") {
        throw "Dashboard Manifest runtime and migration commands must name the exact image-installed executable."
    }
    $dashboardDockerfile = Get-Content -LiteralPath (Join-Path $repoRoot "Dockerfile.dashboard") -Raw
    foreach ($fragment in @(
        "cargo build --release -p tessara-dashboard-module",
        'COPY --from=builder /tmp/dashboard-module /usr/local/bin/dashboard-module',
        'ENTRYPOINT ["dashboard-module"]'
    )) {
        if (-not $dashboardDockerfile.Contains($fragment)) {
            throw "Dashboard image contract omits '$fragment'."
        }
    }
    $expectedDashboardConsumedActions = @("components.catalog", "components.render", "components.resolve")
    $actualDashboardConsumedActions = @(
        $dashboardManifest.consumed_service_actions | ForEach-Object {
            if ([string]$_.dependency_binding -cne "tessara.dashboards.component-version" -or
                [string]$_.functional_contract -cne "tessara.components.component-version") {
                throw "Dashboard consumed service action is not bound to its exact Component dependency."
            }
            [string]$_.authorization_action
        } | Sort-Object
    )
    if (@($dashboardManifest.provided_service_actions).Count -ne 0 -or
        ($actualDashboardConsumedActions -join ",") -cne (($expectedDashboardConsumedActions | Sort-Object) -join ",")) {
        throw "Dashboard Manifest service-action declarations differ from the exact Sprint 8A boundary."
    }
    $dashboardAssets = [ordered]@{
        "/dashboard.js" = "crates/tessara-dashboard-ui/assets/dashboard.js"
        "/dashboard-bindings.js" = "crates/tessara-dashboard-ui/assets/dashboard-bindings.js"
        "/dashboard.wasm" = "crates/tessara-dashboard-ui/assets/dashboard.wasm"
    }
    foreach ($assetPath in $dashboardAssets.Keys) {
        $declaration = @($dashboardManifest.assets | Where-Object path -CEQ $assetPath)
        if ($declaration.Count -ne 1) {
            throw "Dashboard manifest must declare '$assetPath' exactly once."
        }
        $assetFile = Join-Path $repoRoot $dashboardAssets[$assetPath]
        $actualDigest = "sha256:$((Get-FileHash -Algorithm SHA256 -LiteralPath $assetFile).Hash.ToLowerInvariant())"
        if ([string]$declaration[0].digest -cne $actualDigest) {
            throw "Dashboard asset '$assetPath' differs from its manifest digest."
        }
    }
    $dashboardWasmText = [Text.Encoding]::ASCII.GetString(
        [IO.File]::ReadAllBytes((Join-Path $repoRoot $dashboardAssets["/dashboard.wasm"]))
    )
    if ($dashboardWasmText.Contains("dataset_reference") -or
        $dashboardWasmText.Contains("dataset_version_major")) {
        throw "Dashboard embedded WASM retains a Dataset identity outside the canonical Component render response."
    }
    $coreIdentityRegistry = [string]$configuration.services.core.environment.TESSARA_MODULE_SERVICE_IDENTITIES | ConvertFrom-Json
    $componentIdentityRegistry = [string]$configuration.services.components.environment.TESSARA_MODULE_SERVICE_IDENTITIES | ConvertFrom-Json
    $expectedServiceDefinitions = @("tessara.components", "tessara.dashboards")
    foreach ($registry in @($coreIdentityRegistry, $componentIdentityRegistry)) {
        if ([int]$registry.schema_version -ne 1 -or
            (@($registry.identities.PSObject.Properties.Name | Sort-Object) -join ",") -cne
            (($expectedServiceDefinitions | Sort-Object) -join ",")) {
            throw "Sprint 8A service identities must use one exact definition-keyed registry."
        }
    }
    $expectedEndpointDefinitions = @(
        "tessara.components", "tessara.dashboards", "tessara.reference.scoped-records"
    ) | Sort-Object
    foreach ($service in @("core", "dashboards")) {
        $endpoints = [string]$configuration.services.$service.environment.TESSARA_MODULE_SERVICE_ENDPOINTS | ConvertFrom-Json
        if ((@($endpoints.PSObject.Properties.Name | Sort-Object) -join ",") -cne
            ($expectedEndpointDefinitions -join ",")) {
            throw "Sprint 8A '$service' service endpoints are not keyed by exact Module Definition."
        }
    }
    $exchangeSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/module_authorization_exchange.rs") -Raw
    $serviceRequestSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/module_service_requests.rs") -Raw
    $datasetProviderSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/dataset_provider.rs") -Raw
    $componentProductSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-component-module/src/product.rs") -Raw
    $componentBootstrapSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-component-module/src/lib.rs") -Raw
    $componentIntegrationSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-component-module/tests/product_integration.rs") -Raw
    foreach ($forbidden in @("tessara.components", "tessara.dashboards", "TESSARA_COMPONENT_", "TESSARA_DASHBOARD_")) {
        if ($exchangeSource.Contains($forbidden) -or $serviceRequestSource.Contains($forbidden)) {
            throw "Generic Core authorization source contains module-specific identity '$forbidden'."
        }
    }
    foreach ($genericSourcePath in @(
        "crates/tessara-api/src/composition/mod.rs",
        "crates/tessara-supervisor/src/main.rs"
    )) {
        $genericSource = Get-Content -LiteralPath (Join-Path $repoRoot $genericSourcePath) -Raw
        $testBoundary = $genericSource.IndexOf("#[cfg(test)]", [StringComparison]::Ordinal)
        $productionSource = if ($testBoundary -ge 0) {
            $genericSource.Substring(0, $testBoundary)
        } else {
            $genericSource
        }
        foreach ($forbidden in @(
            "tessara.components", "tessara.dashboards", "tessara.datasets",
            "tessara_datasets_contract", "DATASET_BOOTSTRAP_VALIDATION"
        )) {
            if ($productionSource.Contains($forbidden)) {
                throw "Generic composition source '$genericSourcePath' contains product-specific identity '$forbidden'."
            }
        }
    }
    $supervisorManifest = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-supervisor/Cargo.toml") -Raw
    if ($supervisorManifest.Contains("tessara-datasets-contract")) {
        throw "Generic Supervisor depends directly on the Dataset product contract."
    }
    $coreBaselineSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/migrations/001_baseline.sql") -Raw
    if ($coreBaselineSource -notmatch '(?s)CREATE TABLE consumed_module_service_nonces \(.*?authorization_jti UUID UNIQUE') {
        throw "Core service replay storage does not uniquely consume the authorization grant JTI."
    }
    if ($serviceRequestSource -notmatch '(?s)INSERT INTO consumed_module_service_nonces\s+\(module_instance_id,nonce,authorization_jti,correlation_id,issued_at\).*?\.bind\(\s*(?:[A-Za-z_][A-Za-z0-9_]*\.)*grant_consumption\.authorization_jti\(\)\s*\)') {
        throw "Core service request verification does not atomically bind nonce consumption to its optional provider authorization grant JTI."
    }
    if ($exchangeSource -notmatch 'AuthorizationGrantConsumption::ReusableExchange' -or
        $datasetProviderSource -match 'AuthorizationGrantConsumption::ReusableExchange' -or
        $datasetProviderSource -notmatch 'AuthorizationGrantConsumption::OneTimeProviderAudience') {
        throw "Core exchange and Dataset provider grant-consumption modes are not explicitly separated."
    }
    foreach ($requiredFragment in @(
        'DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY',
        'compatibility_marks_non_ready_materialization_before_field_evaluation'
    )) {
        if (-not $datasetProviderSource.Contains($requiredFragment)) {
            throw "Dataset provider omits non-ready compatibility proof '$requiredFragment'."
        }
    }
    if (-not $componentProductSource.Contains('require_ready_dataset_metadata(&metadata, &input.dataset_reference)') -or
        -not $componentBootstrapSource.Contains('bootstrap_rejects_non_ready_and_mismatched_dataset_metadata')) {
        throw "Component pre-write validation does not enforce exact ready Dataset metadata for mutation and bootstrap."
    }
    foreach ($requiredFragment in @(
        'component-create-non-ready-metadata',
        'component-create-mismatched-metadata',
        'assert_component_product_empty(&pool).await'
    )) {
        if (-not $componentIntegrationSource.Contains($requiredFragment)) {
            throw "Component integration coverage omits zero-write Dataset guard '$requiredFragment'."
        }
    }
    if (Test-Path -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/dataset_components_adapter.rs")) {
        throw "Core retains the consumer-named Component Dataset adapter."
    }
    if ($datasetProviderSource -match '(?i)component-datasets|COMPONENT_DEFINITION_ID|TESSARA_COMPONENT_SERVICE') {
        throw "Core Dataset provider retains a Component-specific route or identity branch."
    }
    $rootStyleText = Get-Content -LiteralPath (Join-Path $repoRoot "style/main.css") -Raw
    $forbiddenRootProductSelector = [regex]::Match(
        $rootStyleText,
        '(?im)\.(?:dashboards?[A-Za-z0-9_-]*|components?-[A-Za-z0-9_-]*)'
    )
    if ($forbiddenRootProductSelector.Success) {
        throw "Core root CSS retains extracted Dashboard/Component product selector text '$($forbiddenRootProductSelector.Value)'."
    }

    foreach ($runner in @("scripts/capture-sprint-6a-deployment-evidence.ps1", "scripts/validate-e2e.ps1", "scripts/smoke.ps1")) {
        $runnerText = Get-Content -LiteralPath (Join-Path $repoRoot $runner) -Raw
        if ($runnerText -notmatch 'TransitionCatalogProfile') {
            throw "Sprint 8A deployment runner '$runner' cannot bind the exact transition-catalog profile."
        }
    }
    Test-Sprint8AFirstPartyComponentContractSources -RepoRoot $repoRoot
    $deploymentEvidenceCommonText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/sprint-6a-deployment-evidence-common.ps1") -Raw
    foreach ($requiredFragment in @(
        '$script:Sprint8ABuiltInSeedVersion = "sprint-8a-role-capabilities-v1+sha256.4f607b6f428c"',
        '$script:Sprint8ABuiltInSeedSha256 = "4f607b6f428c0de70901dd119f7026b4c700c9e86309e76a3f5085a4da366609"',
        'function Get-Sprint6ABuiltInSeedContract',
        '$expectedSeed = $contract.expected_seed',
        '[string]$Evidence.snapshot.built_in_seed.version -cne [string]$seedContract.version',
        '[string]$Evidence.snapshot.built_in_seed.canonical_sha256 -cne [string]$seedContract.canonical_sha256'
    )) {
        if (-not $deploymentEvidenceCommonText.Contains($requiredFragment)) {
            throw "Sprint 8A deployment evidence omits profile-specific built-in seed contract fragment '$requiredFragment'."
        }
    }
    $deploymentEvidenceCaptureText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/capture-sprint-6a-deployment-evidence.ps1") -Raw
    foreach ($requiredFragment in @(
        '$sprint8SeedRows',
        '-TransitionCatalogProfile sprint-8a',
        '$crossProfileSeedRejected',
        '$crossProfileEvidenceRejected',
        '$missingSprint8ProfileRejected'
    )) {
        if (-not $deploymentEvidenceCaptureText.Contains($requiredFragment)) {
            throw "Sprint 8A deployment-evidence self-test omits '$requiredFragment'."
        }
    }
    $materializationText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/materialize-sprint-8a.ps1") -Raw
    if ($materializationText -match '(?m)^\s*-SkipLegacySeed(?:\s|`|$)') {
        throw "Sprint 8A reference materialization must rebuild the established acceptance fixtures."
    }
    foreach ($requiredFragment in @(
        '[int]$Attempt',
        '[string]$EvidenceRoot',
        '[string]$EnvironmentFingerprint',
        '[string]$BlueprintPath',
        '[string]$ControlUrl',
        '[switch]$SelfTest',
        '"materialization/attempt-$Attempt"',
        '"resolved-targets.json"',
        '"empty-baseline.json"',
        '"first-apply-response.json"',
        '"no-op-apply-response.json"',
        '"failure-teardown.json"',
        '"public-gateway-boundary.json"',
        '"final-health.json"',
        '"materialization-result.json"',
        '. (Join-Path $PSScriptRoot "sprint-8a-health-contract.ps1")',
        'Invoke-Sprint8AHealthProbe -Target gateway_core',
        'Invoke-Sprint8AHealthProbe -Target supervisor',
        'schema_version = 2',
        'tessara.sprint-8a.health-observation/v1',
        'Assert-Sprint8ADestructiveEndpoint',
        'ExpectedPort 8088',
        'ExpectedPort 18088',
        'ExpectedPort 8098',
        '$unexpectedNetworks',
        '. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")',
        'Get-Sprint8AComposeServiceProjection -Services $configuration.services',
        'ConvertFrom-Sprint8AComposeServiceJson',
        'Get-Sprint8AMaterializationFailureClassification',
        '-ComposeFile $composePath',
        'Start-Sprint8APublicGateway',
        '-ExcludePublicGateway',
        '-SemanticNoOp',
        'verify_read_back',
        'zero approved effects',
        "foreach (`$field in @('desired_enablement', 'observed_enablement', 'observed_artifacts', 'configuration_digests'))"
    )) {
        if (-not $materializationText.Contains($requiredFragment)) {
            throw "Sprint 8A materialization omits canonical evidence contract fragment '$requiredFragment'."
        }
    }
    & (Join-Path $repoRoot "scripts/materialize-sprint-8a.ps1") -SelfTest | Out-Null
    $compositionBootstrapText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/bootstrap-sprint-7a-composition.ps1") -Raw
    foreach ($requiredFragment in @(
        '[switch]$ExcludePublicGateway',
        '$startupServices',
        '[string]$_ -cne "gateway"',
        'ps --status running -q gateway',
        'public gateway must remain stopped during owner materialization'
    )) {
        if (-not $compositionBootstrapText.Contains($requiredFragment)) {
            throw "Sprint 8A owner materialization does not enforce its offline public-gateway boundary fragment '$requiredFragment'."
        }
    }
    $failureContainmentPath = Join-Path $repoRoot "scripts/run-sprint-8a-failure-containment.ps1"
    if (-not (Test-Path -LiteralPath $failureContainmentPath -PathType Leaf)) {
        throw "Sprint 8A repository-owned failure-containment runner is missing."
    }
    $failureContainmentText = Get-Content -LiteralPath $failureContainmentPath -Raw
    foreach ($requiredFragment in @(
        '[int]$Attempt',
        '[string]$EvidenceRoot',
        '[string]$EnvironmentFingerprint',
        '[string]$OutputPath',
        '[switch]$SelfTest',
        'sprint-8a.dashboard-owner-bootstrap-invalid-layout',
        'Dashboard bootstrap layout is invalid',
        '"fault"',
        '"successor"',
        '"recovery"',
        'original_defect',
        'tessara.sprint-8a.required-recovery',
        'tessara.sprint-8a.canonical-restoration',
        'unexpected_fault_acceptance',
        'unexpected_precondition_failure',
        'Get-Sprint8AFailureReceiptDisposition',
        'Assert-Sprint8ADestructiveEndpoint',
        'Assert-Sprint8AContainmentReferencedArtifact',
        '-RequireSidecar',
        'sidecar does not exactly bind',
        'verified_evidence.raw_artifacts',
        'verified_evidence.teardown',
        'verified_evidence.empty_baseline',
        'verified_evidence.first_apply_response',
        'verified_evidence.no_op_apply_response',
        'verified_evidence.final_health',
        'verified_evidence.final_health_preceding_apply_response',
        'Test-Sprint8AHealthObservation -Observation $finalHealth.health.gateway_core',
        'Test-Sprint8AHealthObservation -Observation $finalHealth.health.supervisor',
        'fault_raw_artifacts',
        'evidence-finalization',
        '"failure-containment-result.json"'
    )) {
        if (-not $failureContainmentText.Contains($requiredFragment)) {
            throw "Sprint 8A failure-containment runner omits canonical contract fragment '$requiredFragment'."
        }
    }
    $receiptRetentionIndex = $failureContainmentText.IndexOf(
        '$faultReceiptArtifact = Get-Sprint8AContainmentArtifact -Path $faultFailureReceiptPath -RequireSidecar',
        [StringComparison]::Ordinal
    )
    $receiptAuthenticationIndex = $failureContainmentText.IndexOf(
        '$faultReceipt = Assert-Sprint8AFailureReceipt',
        [StringComparison]::Ordinal
    )
    $expectedFaultDispositionIndex = $failureContainmentText.IndexOf(
        '$faultDisposition = Get-Sprint8AFailureReceiptDisposition -Receipt $faultReceipt',
        [StringComparison]::Ordinal
    )
    if ($receiptRetentionIndex -lt 0 -or
        $receiptAuthenticationIndex -le $receiptRetentionIndex -or
        $expectedFaultDispositionIndex -le $receiptAuthenticationIndex) {
        throw "Sprint 8A failure containment must retain the failure receipt, authenticate all referenced evidence, and only then classify expected-fault semantics."
    }
    & $failureContainmentPath -SelfTest | Out-Null
    if ($failureContainmentText.Contains('raw_artifacts = $faultReceipt.raw_artifacts') -or
        $failureContainmentText.Contains('teardown = $faultReceipt.teardown') -or
        $failureContainmentText.Contains('empty_baseline = $successorReceipt.evidence.empty_baseline') -or
        $failureContainmentText.Contains('final_health = $successorReceipt.evidence.final_health')) {
        throw "Sprint 8A containment result must publish verified actual-file and sidecar identities, not unverified receipt declarations."
    }
    $semanticFixturePath = Join-Path $repoRoot "scripts/prepare-sprint-7a-uat-fixtures.ps1"
    $semanticFixtureText = Get-Content -LiteralPath $semanticFixturePath -Raw
    foreach ($requiredFragment in @(
        'tessara_module_components',
        'SplitComponentOwnership',
        "kind='module_instance'",
        "resource_type='tessara.components.component_version'"
    )) {
        if (-not $semanticFixtureText.Contains($requiredFragment)) {
            throw "Sprint 8A semantic fixtures do not preserve the extracted Component ownership contract ('$requiredFragment')."
        }
    }
    # The fixture self-test parses its AST and rejects product mutations outside
    # the Sprint 7A-only exclusion branch.
    & $semanticFixturePath -SelfTest | Out-Null
    $compositionBootstrapText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/bootstrap-sprint-7a-composition.ps1") -Raw
    if (-not $compositionBootstrapText.Contains('-OwnerControlledSeed:($RuntimeLabel -ceq "sprint-8a")')) {
        throw "Sprint 8A composition must keep product seed writes inside each owning bootstrap API."
    }
    foreach ($requiredFragment in @(
        '[string]$BlueprintPath',
        '[string]$RuntimeDirectory',
        '$resolvedBlueprintPath',
        '[switch]$SemanticNoOp',
        'Unchanged desired state did not resolve to the exact semantic no-op read-back plan.',
        '[switch]$SelfTest',
        'function Get-Sprint7AApprovedEffects',
        '$action = $_',
        '[bool]$action.enabled',
        'function Get-Sprint7AProcessEnvironmentSnapshot',
        'function Restore-Sprint7AProcessEnvironmentSnapshot',
        '$composePath = Resolve-RepositoryPath -Path $ComposeFile',
        'Restore-Sprint7AProcessEnvironmentSnapshot -Snapshot $processEnvironmentSnapshot',
        'Get-Sprint7AApprovedEffects -Actions @($lockfile.materialization_plan.actions)'
    )) {
        if (-not $compositionBootstrapText.Contains($requiredFragment)) {
            throw "Sprint 8A composition bootstrap omits harness override '$requiredFragment'."
        }
    }
    & (Join-Path $repoRoot "scripts/bootstrap-sprint-7a-composition.ps1") -SelfTest | Out-Null
    $bootstrapTokens = $null
    $bootstrapParseErrors = $null
    $bootstrapAst = [Management.Automation.Language.Parser]::ParseFile(
        (Join-Path $repoRoot "scripts/bootstrap-sprint-7a-composition.ps1"),
        [ref]$bootstrapTokens,
        [ref]$bootstrapParseErrors
    )
    $bootstrapExecutionTries = @($bootstrapAst.FindAll({
        param($node)
        $node -is [Management.Automation.Language.TryStatementAst] -and
            $node.Body.Extent.Text.Contains('$env:TESSARA_SOURCE_COMMIT =') -and
            $node.Body.Extent.Text.Contains('$env:TESSARA_SIGNING_ISSUER =')
    }, $true))
    $bootstrapCleanupTries = @()
    if ($bootstrapExecutionTries.Count -eq 1 -and
        $null -ne $bootstrapExecutionTries[0].Finally) {
        $bootstrapCleanupTries = @($bootstrapExecutionTries[0].Finally.FindAll({
            param($node)
            $node -is [Management.Automation.Language.TryStatementAst] -and
                $node.Body.Extent.Text.Contains('Pop-Location') -and
                $null -ne $node.Finally -and
                $node.Finally.Extent.Text.Contains('Restore-Sprint7AProcessEnvironmentSnapshot -Snapshot $processEnvironmentSnapshot')
        }, $true))
    }
    if (@($bootstrapParseErrors).Count -ne 0 -or
        $bootstrapExecutionTries.Count -ne 1 -or
        $bootstrapCleanupTries.Count -ne 1) {
        throw "Sprint 8A composition bootstrap must restore the caller process environment in a nested finally even when location cleanup fails."
    }
    $dashboardAcceptanceText = Get-Content -LiteralPath (Join-Path $repoRoot "end2end/tests/dashboards.spec.ts") -Raw
    if (-not $dashboardAcceptanceText.Contains('option.component_slug === "sprint-8a-record-table"')) {
        throw "Sprint 8A Dashboard paging acceptance is not bound to the exact module-owned record Table."
    }
    $baselineDockerfile = Get-Content -LiteralPath (Join-Path $repoRoot "deploy/sprint-8a/Dockerfile.component-rehearsal-baseline") -Raw
    foreach ($requiredFragment in @(
        'cargo build --release -p tessara-component-module',
        '--features sprint-8a-rehearsal-baseline',
        'com.tessara.rehearsal.fixture="source-built-compatible-release-v1"',
        'COPY --from=builder /tmp/component-module /usr/local/bin/component-module'
    )) {
        if (-not $baselineDockerfile.Contains($requiredFragment)) {
            throw "Sprint 8A source-built Component baseline omits '$requiredFragment'."
        }
    }
    if ($baselineDockerfile -match '(?m)^\s*FROM\s+\$\{?COMPONENT_BASE_IMAGE') {
        throw "Sprint 8A Component rehearsal baseline must be source-built, not a relabeled candidate image."
    }
    $composeOverride = Get-Content -LiteralPath (Join-Path $repoRoot "deploy/sprint-8a/compose.override.yaml") -Raw
    foreach ($requiredFragment in @(
        'TESSARA_DEPLOYMENT_COMPOSE_FILE:',
        '/var/run/docker.sock:/var/run/docker.sock'
    )) {
        if (-not $composeOverride.Contains($requiredFragment)) {
            throw "Sprint 8A Supervisor-owned release transition wiring omits '$requiredFragment'."
        }
    }
    Test-Sprint8ADeploymentTargetContract -ComposeText $composeOverride
    $inventoryAudit = Join-Path $repoRoot "scripts/audit-sprint-8a-deployed-inventory.ps1"
    if (-not (Test-Path -LiteralPath $inventoryAudit -PathType Leaf)) {
        throw "Sprint 8A exact deployed inventory/navigation audit runner is missing."
    }
    $rehearsalRunners = [ordered]@{
        "scripts/smoke-sprint-8a.ps1" = @(
            "Test-Sprint8ADashboardPlacementProjection",
            '$propertyNames -cnotcontains "placement_key"',
            '$Placement.grid_row', '$Placement.grid_column',
            '$Placement.grid_width', '$Placement.grid_height',
            'Invoke-Sprint8AHealthProbe -Target gateway_core',
            'Invoke-Sprint8AHealthProbe -Target supervisor'
        )
        "scripts/run-sprint-8a-deployed-smoke.ps1" = @(
            "ApiContainerId", "GatewayContainerId", "DatabaseContainerId",
            "ExpectedDataState fresh", "TransitionCatalogProfile sprint-8a",
            "DeploymentEvidencePath"
        )
        "scripts/run-sprint-8a-component-upgrade.ps1" = @(
            "build-sprint-8a-component-rehearsal-baseline.ps1", "OutputTag",
            "verify-sprint-8a-component-upgrade.ps1", "BaselineMetadataPath",
            '-OutputPath $OutputPath'
        )
    }
    foreach ($runner in $rehearsalRunners.Keys) {
        $runnerPath = Join-Path $repoRoot $runner
        if (-not (Test-Path -LiteralPath $runnerPath -PathType Leaf)) {
            throw "Sprint 8A repository-owned rehearsal runner '$runner' is missing."
        }
        $runnerText = Get-Content -LiteralPath $runnerPath -Raw
        foreach ($fragment in $rehearsalRunners[$runner]) {
            if (-not $runnerText.Contains($fragment)) {
                throw "Sprint 8A rehearsal runner '$runner' omits canonical orchestration fragment '$fragment'."
            }
        }
    }
    $upgradeContracts = [ordered]@{
        "crates/tessara-composition/src/lib.rs" = @(
            "pub fn resolve_against(", "delta_materialization_actions(",
            "release_update_materializes_only_the_changed_module",
            "unchanged_owner_state_is_not_reconfigured_or_bootstrapped"
        )
        "crates/tessara-api/migrations/001_baseline.sql" = @(
            "lockfile JSONB NOT NULL"
        )
        "crates/tessara-api/src/composition/mod.rs" = @(
            "current_applied_lockfile(", "resolve_against(", "previous_lockfile.as_ref()"
        )
        "crates/tessara-api/src/modules/service.rs" = @(
            "previous_lockfile: Option<&ApplicationLockfileV1>",
            "changed_definitions", "action_owners",
            "INSERT INTO composition_receipt_projections"
        )
        "crates/tessara-supervisor/src/lib.rs" = @(
            "delta_receipt_carries_forward_unchanged_bootstrap_receipts",
            "failed_materialization_is_terminal_and_does_not_block_a_successor_apply",
            "failed_core_projection_retires_current_receipt_and_allows_same_sequence_retry",
            "failed_emergency_override_finalization_does_not_publish_receipt_or_block_retry",
            "rollback_projection_failure(",
            "SELECT MAX(apply_sequence) FROM operations WHERE state != 'failed'",
            "[MaterializationActionV1::VerifyReadBack]",
            "receipt.changed = false;",
            "ledger_finalization_failed",
            "INSERT INTO emergency_overrides"
        )
        "crates/tessara-supervisor/src/main.rs" = @(
            "TESSARA_DEPLOYMENT_COMPOSE_FILE", "run_compose_migration(",
            "run_compose_runtime_switch(", "available_bootstrap_receipts",
            "owner_adapter_prepare_failed", "core_projection_failed",
            "rollback_projection_failure(", "build_health_client()",
            "Policy::none()", 'owner_health_path("core")',
            'validate_owner_health_response(', 'b"ok"'
        )
        "scripts/build-sprint-8a-component-rehearsal-baseline.ps1" = @(
            "source-built-compatible-release-v1", "executable_sha256",
            "Publish-Sprint7AEvidence"
        )
        "scripts/verify-sprint-8a-component-upgrade.ps1" = @(
            "pre-exercise-current", "establish-compatible-baseline",
            "upgrade-to-candidate", "rollback-to-baseline", "restore-intended-candidate",
            "Assert-ExactDeltaPlan", "Assert-Preservation",
            "unrelated_container_image_restart_data_availability",
            "x-tessara-module-control-key: `$moduleControlKey", '$smokeSucceeded = $?',
            'Invoke-Sprint8AHealthProbe -Target gateway_core',
            'Invoke-Sprint8AHealthProbe -Target supervisor'
        )
        "scripts/sprint-8a-health-contract.ps1" = @(
            'function Get-Sprint8AHealthContract',
            'function Test-Sprint8AHealthObservation',
            'function Invoke-Sprint8AHealthProbe',
            '$handler.AllowAutoRedirect = $false',
            'path = "/health"',
            'path = "/health/ready"',
            'status = 204',
            'sprint-8a-health-contract-regressions.json',
            'diagnostic_history_only'
        )
    }
    foreach ($path in $upgradeContracts.Keys) {
        $text = Get-Content -LiteralPath (Join-Path $repoRoot $path) -Raw
        foreach ($fragment in $upgradeContracts[$path]) {
            if (-not $text.Contains($fragment)) {
                throw "Sprint 8A Component transition contract '$path' omits '$fragment'."
            }
        }
    }
    $supervisorMainSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-supervisor/src/main.rs") -Raw
    if ($supervisorMainSource.Contains('.record_emergency_override(')) {
        throw "Sprint 8A emergency override and successful receipt are not finalized in one Supervisor ledger transaction."
    }
    $moduleProjectionSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/modules/service.rs") -Raw
    $receiptProjectionIndex = $moduleProjectionSource.IndexOf('INSERT INTO composition_receipt_projections', [StringComparison]::Ordinal)
    $projectionCommitIndex = if ($receiptProjectionIndex -lt 0) { -1 } else {
        $moduleProjectionSource.IndexOf('tx.commit().await?', $receiptProjectionIndex, [StringComparison]::Ordinal)
    }
    $compositionProjectionSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/composition/mod.rs") -Raw
    if ($receiptProjectionIndex -lt 0 -or $projectionCommitIndex -lt $receiptProjectionIndex -or
        $compositionProjectionSource -match '(?s)INSERT INTO composition_receipt_projections.*?execute\(&state\.pool\)') {
        throw "Sprint 8A module inventory and applied lockfile/receipt projection must commit in one Core transaction."
    }
    $deployedSmokeRunner = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/run-sprint-8a-deployed-smoke.ps1") -Raw
    if ($deployedSmokeRunner.Contains("AcceptanceEvidencePath") -or
        $deployedSmokeRunner.Contains("OverwriteAcceptanceEvidence")) {
        throw "Sprint 8A rehearsal smoke must not publish the Sprint 6A authoritative acceptance-evidence schema."
    }
    $validationContracts = [ordered]@{
        "scripts/sprint-8a-validation-environment.ps1" = @(
            "TEST_API_DATABASE_URL", "TEST_API_FRESH_DATABASE_URL",
            "TEST_REFERENCE_MODULE_DATABASE_URL", "TEST_COMPONENT_MODULE_DATABASE_URL",
            "TEST_API_ENROLLMENT_DATABASE_URL",
            "TEST_INSTALLATION_CONTROL_DATABASE_URL", "Invoke-Sprint8ADatabaseProbe",
            "Get-Sprint8ADeploymentEnvironmentProbe", "Get-Sprint8AToolchainEnvironmentContract",
            "function Resolve-Sprint8AEvidenceReference",
            "function Test-Sprint8AEvidenceReferenceResolution",
            "function Compare-Sprint8AEnvironmentContracts",
            "function Test-Sprint8AEnvironmentContractComparison",
            "function Get-Sprint8AOptionalObjectPropertyValue",
            "function Get-Sprint8AComposeServiceProjection",
            "function Test-Sprint8AComposeServiceProjection",
            "function Get-Sprint8AResultClassifications",
            "function Test-Sprint8AResultClassificationProjection",
            "function Assert-Sprint8ASourceIdentityObject",
            "function ConvertTo-Sprint8ACorrectionLineage",
            "function Get-Sprint8AEvidenceRelativePath",
            "function Assert-Sprint8ACurrentReadinessReference",
            "function Assert-Sprint8AReadinessCorrectionLineagePresence",
            "function Add-Sprint8ACorrectionLineageLink",
            "function Assert-Sprint8ACorrectionIdentityContinuity",
            "function Assert-Sprint8ACorrectionLineageTopology",
            "function Assert-Sprint8ACorrectionLineage",
            "function Assert-Sprint8AR33CorrectionAuthorizationQualification",
            "approved_historical_evidence_correction",
            "allowed_successor_attempt = 44",
            "five Core transitions and one real Dashboard presentation",
            "function Assert-Sprint8AReadinessSupersessionChain",
            "current_readiness_binding", "direct_correction_consumption",
            "clean_pre_rehearsal_supersession",
            '($currentDocument.schema_version -isnot [int] -and $currentDocument.schema_version -isnot [long])',
            '($immutableDocument.schema_version -isnot [int] -and $immutableDocument.schema_version -isnot [long])',
            '@(2, 3) -notcontains [int]$currentDocument.schema_version',
            '@(2, 3) -notcontains [int]$immutableDocument.schema_version',
            '[int]$startRef.document.schema_version -ne 2',
            '($startRef.document.schema_version -isnot [int] -and $startRef.document.schema_version -isnot [long])',
            '@(2, 3) -notcontains [int]$terminalRef.document.schema_version',
            '($terminalRef.document.schema_version -isnot [int] -and $terminalRef.document.schema_version -isnot [long])',
            '$terminalRef.document.assertions_started -ne $true',
            "Root failed-Readiness correction link cannot be a truncated consumed suffix",
            "Historical R30 immutable Readiness prerequisite",
            'RelativePath "attempts/readiness-37.json"',
            "noncanonical start, consumption, or terminal path",
            "one exact immutable attempt counterpart",
            "complete preceding lineage prefix",
            '"docs/sprints/sprint-8a-uat/scenario-contract.json"',
            '[pscustomobject][ordered]@{',
            '".codex/skills/tessara-sprint-validation/**"',
            '"docs/sprints/sprint-8a-*.md"',
            '"end2end/**"',
            '"scripts/fixtures/**"',
            '"scripts/*sprint-8a*.ps1"',
            '"scripts/test-sprint-validation-harvest.ps1"',
            '"deploy/sprint-7a/**"',
            '"deploy/sprint-8a/**"',
            '"crates/*/manifest.json"',
            '"crates/*/migrations/*.sql"',
            "tessara.sprint-8a.deployment-environment-probe", "DeploymentProbe",
            "materialization_control",
            "transaction_round_trip", "canonical_server", 'identity = "$canonicalServer/',
            "environment", "fingerprint",
            'function Assert-Sprint8ACanonicalEvidencePath', '$strictLineagePaths',
            '$strictConsumptionPaths', '-RequireCanonical:$strictLineagePaths'
        )
        "scripts/sprint-8a-rehearsal-scheduler.ps1" = @(
            "Get-Sprint8ARehearsalLanePolicies", "Resolve-Sprint8ARehearsalSchedule",
            "Test-Sprint8ARehearsalDeclarationMember", "[Collections.IDictionary]",
            "Assert-Sprint8ARehearsalScheduleContract", "Get-Sprint8AAuthenticatedRehearsalHistory",
            "Get-Sprint8ARehearsalChangedPaths", "New-Sprint8ADeferredLaneResult",
            "Get-Sprint8ARehearsalWaveBDisposition",
            "Assert-Sprint8ARehearsalTerminalAccounting", "Test-Sprint8ARehearsalTwoWaveScheduler",
            "Test-Sprint8ARehearsalHistoryRegressionFixtures", "sprint-8a-rehearsal-history-regressions.json",
            "bounded_failure_first_two_wave", "conservative_full_harvest_fallback",
            "maximum_consecutive_deferrals_reached", "aggregate_sink_waits_for_current_attempt_prerequisites",
            "Prior evidence is diagnostic history only",
            "Assert-Sprint8ARehearsalCanonicalReferencePath", "depends on later segment",
            "relevant_prerequisite_changed", "pre_authentication_lifecycle_placeholder",
            "scheduler self-test must exercise live ordered-dictionary declarations",
            "scheduler self-test must dispatch member lookup through the IDictionary interface"
        )
        "scripts/materialize-sprint-8a.ps1" = @(
            '$exceptionType = $materializationError.Exception.GetType().FullName',
            'Get-Sprint8AMaterializationFailureClassification',
            'TessaraFailureClassification',
            'PropertyNotFoundException|ParameterBindingException|CommandNotFoundException|ParseException',
            'Docker daemon is not running|Cannot connect to the Docker daemon|connection refused|timed out while waiting for .* health',
            'failure = [ordered]@{',
            'classification = $failureClassification',
            'exception_type = $exceptionType'
        )
        "scripts/run-sprint-8a-candidate-rehearsal.ps1" = @(
            "attempt-state-prerequisite", "validation-readiness-prerequisite", "formatting", "workspace-check",
            "workspace-clippy", "compose-manifest-schema-contract",
            "web-native-wasm-source-boundaries", "module-sdk-boundaries",
            "dashboard-source-boundaries", "markdown-links", "workspace-tests",
            "optimized-resource-reference-timing",
            "resource_reference_restricted_known_random_latency_profile",
            "components-contract-tests", "dashboard-module-tests",
            "component-conformance-nondisclosure", "module-testkit-conformance",
            "playwright-discovery", "source-exact-materialization-no-op",
            "deployed-inventory-navigation-audit", "deployment-evidence", "product-smoke",
            "playwright-execution", "component-upgrade-rollback",
            "failure-containment-successor-health", "successor-inventory-navigation-audit",
            "successor-deployment-evidence", "successor-product-smoke", "live-product-diagnostics", "uat-diagnostics",
            "final-clean-source", "final-environment-identity", "final-successor-health",
            "harvest_complete", "candidate-rehearsal-result.json",
            "test-sprint-validation-harvest.ps1", "validation-state.json",
            "nested_results_path", "nested_blocked_checks", "nested_blocked_count",
            "nested_failed_checks", "nested_failure_count", "harness_failure_count",
            "Assert-Sprint8ANestedUatReceiptIdentity", '[long]$schema -ne 2',
            '$Receipt.authoritative -isnot [bool]', "numerically coerced nested UAT authority",
            "nested semantic assertion", "does not retain its exact semantic assertion inventory",
            "playwright-acceptance.discovery.json", "playwright-acceptance.xml", "playwright-acceptance.summary.json",
            "productDiagnosticRawEvidence", "acceptance-manifest inventory",
            "evidence_roots", "produced_evidence", "raw_evidence", 'state = "executing"', '[switch]$SelfTest'
            "Assert-RehearsalIndependentChecks", "correction_lineage",
            "Test-RehearsalReadinessLaneIsolation",
            "Independent Readiness lane retains state-lane dependency",
            "Resolve-Sprint8AEvidenceReference",
            "state/readiness failure cannot suppress useful safe evidence",
            "validation-attempt.lock", "Open-Sprint8AValidationAttemptLock", '[IO.FileShare]::None',
            "Invoke-RehearsalPowerShellCheck", "Test-RehearsalPowerShellCheck",
            "A stale native exit code falsely failed a successful PowerShell child.",
            "Assert-Sprint8ACorrectionLineage",
            "current_readiness_binding", "direct_correction_consumption",
            "clean_pre_rehearsal_supersession",
            "Assert-Sprint8ACurrentReadinessReference",
            "Assert-Sprint8AReceiptSidecar -Path `$statePath",
            "ExpectedCurrentReadiness", "RequireConsumedTip",
            'receipt = [string]$stateIndex.readiness.receipt',
            "readiness_immutable_reference",
            'path = [string]$runtimeContext.readiness_immutable_reference.path',
            'sha256 = [string]$runtimeContext.readiness_immutable_reference.sha256',
            "Test-Sprint8AFirstRehearsalCorrectionLink",
            '-ComponentsContractLaneReceipt (Join-Path $laneRoot "components-contract-tests.json")',
            'Assert-Sprint8AReadinessCorrectionLineagePresence',
            '$attemptReceipt.correction_lineage = $stateCorrectionLineage',
            "function Resolve-LaneClassification",
            '[AllowNull()][string]$StructuredClassification',
            'source = "structured_evidence"',
            "function Get-RehearsalStructuredClassification",
            "function Test-RehearsalClassificationResolution",
            "classification_source", "assertions_started", "assertions_started_at",
            "final-environment-identity.json", "Compare-Sprint8AEnvironmentContracts",
            "changed_sections",
            'Test-Sprint8AResultClassificationProjection',
            'Get-Sprint8AResultClassifications -Results (@($failedChecks) + @($nestedFailedChecks))',
            'candidate-rehearsal-$Attempt-start.json', "immutable_start_receipt", "schedule_sha256",
            "next_candidate_rehearsal", "deferred_count", "deferred_checks",
            "New-Sprint8ADeferredLaneResult", "Complete-RehearsalOrphanedLane",
            "Test-Sprint8ACandidateTwoWaveRunnerContract", "ResumeInterruptedAttempt",
            "Resolve-RehearsalPreAttemptRecoveryState", "Resolve-RehearsalAttemptRecoveryState",
            "Test-Sprint8AProcessLossRecoveryContract", "Publish-OrAuthenticateRehearsalImmutableEvidence",
            "Assert-RehearsalLaneIdentityBinding", "Set-RehearsalRecoveredAttemptIdentity",
            "Test-RehearsalRecoveredTerminalSourceBinding",
            "Assert-RehearsalRestorationMaterializationReceipt", '$null -ne $restorationEvidence',
            '-HarvestOnly', "correction_authorization_withheld_cleanup_not_proven",
            '$restorationRequired = $true', "AuthorizeApprovedR33Correction",
            "Invoke-ApprovedR33CorrectionAuthorization",
            "The immutable R33 failure remains unchanged",
            "exactly Readiness 44 may start next"
        )
        "scripts/test-sprint-validation-harvest.ps1" = @(
            "Assert-DiagnosticReceiptHeader", "Assert-MutableSourceIdentity", "Assert-EnvironmentFingerprint",
            '$authoritative -isnot [bool]', "a string-coerced attempt schema", "an uppercase environment fingerprint",
            "Assert-HashedFileEvidence", "produced_evidence", "raw_evidence",
            "harvest_complete", "exactly one open batch", "CorrectionAuthorizationPath",
            "nested_blocked_count", "nested_failed_count", "nested_assertion",
            "consumption_state", "allowed_successor_count", "blocked-check inventory", "passed_count",
            '$allowedClassifications = @(', "assertions_started", "assertions_started_at",
            '$assertionBearingTerminal = @($terminal | Where-Object assertions_started -EQ $true)',
            'Passed check', 'must not retain a failure classification',
            "Resolve-Sprint8AEvidenceReference"
            "raw evidence outside the repository evidence root",
            'Where-Object { $null -ne $_ }',
            "validation-readiness-harvest", "validation-readiness-defect-batch",
            '$predecessorPhase-correction-authorization', "allowed_successor_attempt",
            "Assert-ReadinessHarvestComplete", '$script:StrictCanonicalEvidencePaths',
            "Assert-CandidateRehearsalLaneReceipt", '[switch]$HarvestOnly'
        )
        "scripts/validate-e2e.ps1" = @(
            "InventoryOnly", "ActualIdentities", "Independent Playwright discovery",
            "FailureEvidenceDirectory", "Publish-PlaywrightFailureEvidence",
            "failure-summary.json", "test-results", "Refusing to overwrite retained Playwright failure evidence"
        )
        "scripts/validate-sprint-8a-readiness.ps1" = @(
            "attempt-state-prerequisite", "compose-database-contract", "Get-Sprint8ADeploymentEnvironmentProbe",
            '-DeploymentProbe $script:deploymentProbe', "Assert-Sprint8AReadinessFailLateGraph",
            "authenticated six-database transaction probes", '$databaseProbes.Count -ne 6', '[switch]$SelfTest',
            "TEST_ALIAS_A_DATABASE_URL", "equivalent loopback host aliases",
            "validate-e2e.ps1 -InventoryOnly exact acceptance-manifest identity",
            "-InventoryOnly -EvidencePath `$playwrightInventoryPath",
            "playwright-inventory.json", "produced_evidence", "Checkpoint-ReadinessAttempt",
            "source_identity_verification_state", "predecessor_correction_authorization",
            "correction_lineage", "consumed_by_readiness",
            "duplicate or alternate Readiness consumption is forbidden",
            "FinalizeFailedAttempt", "Complete-Sprint8AFailedReadinessHarvest",
            "Test-Sprint8AReadinessSupersessionContract", "Test-Sprint8AFailedReadinessFinalization",
            "Test-Sprint8ACorrectionIdentityContinuity", "Assert-Sprint8ANoReadinessSuccessorCollision",
            "Test-Sprint8ACleanReadinessRerunState", "Add-Sprint8ACorrectionLineageLink",
            "Get-Sprint8AReadinessReceiptCorrectionLineage",
            "clean_rerun", "lineage_validation", "preserved_correction_lineage", "prerequisite_receipts",
            '$readinessPrerequisiteReceipts',
            '$cleanRerunReservation = Get-Sprint8AReadinessAttemptReservation',
            '$cleanRerunPrerequisites.Count -ne 1',
            "without a second consumption", "noncontiguous successor attempt",
            "orphaned consumed correction transition",
            "approved_historical_evidence_correction",
            "source differs from the exact clean correction source approved by the R33 evidence-correction record",
            'receipt = $relativeAttemptPath',
            "CurrentReadinessReference", 'path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath)',
            "New-Sprint8ANextCandidateRehearsalPlan", "next_candidate_rehearsal",
            "schedule_sha256", "conservative_fallback", "RequireFreshDatabases",
            'readiness-$Attempt-harvest.json', 'readiness-$Attempt-defect-batch.json',
            'readiness-$Attempt-correction-authorization.json',
            "validation-attempt.lock", "Open-Sprint8AValidationAttemptLock", '[IO.FileShare]::None',
            "Publish-Sprint8AAppendOnlyJsonReceipt", '[IO.FileMode]::CreateNew',
            "-correction-consumption", "correction_consumption_receipt",
            '[pscustomobject]@{ name = "compose-optional-properties"; action = { Test-Sprint8AComposeServiceProjection } }',
            '[pscustomobject]@{ name = "result-classification-projection"; action = { Test-Sprint8AResultClassificationProjection } }',
            '[pscustomobject]@{ name = "environment-comparison"; action = { Test-Sprint8AEnvironmentContractComparison } }',
            '[pscustomobject]@{ name = "evidence-reference-containment"; action = { Test-Sprint8AEvidenceReferenceResolution } }',
            '[pscustomobject]@{ name = "composition-bootstrap"; action = { & ./scripts/bootstrap-sprint-7a-composition.ps1 -SelfTest',
            "assertions_started", "assertions_started_at",
            '$startReceipt.assertion_count = @($checks | Where-Object assertions_started -EQ $true).Count',
            "Resolve-Sprint8AEvidenceReference",
            'Get-Sprint8AResultClassifications -Results $failures'
        )
        "scripts/uat-sprint-8a.ps1" = @(
            "MaterializationLaneReceipt", "InventoryLaneReceipt",
            "DeploymentEvidenceLaneReceipt", "ProductSmokeLaneReceipt",
            "FailureContainmentLaneReceipt", "UpgradeLaneReceipt", "ComponentsContractLaneReceipt",
            "ComponentConformanceLaneReceipt", "PlaywrightLaneReceipt",
            "ManifestContractLaneReceipt", "WebBoundaryLaneReceipt",
            "DashboardBoundaryLaneReceipt", "ProductDiagnosticLaneReceipt",
            '$Receipt.authoritative -isnot [bool]', "numerically coerced prerequisite authority flag",
            "string-coerced prerequisite schema", "malformed environment identity",
            "failure-containment-successor-health", "semantic_assertions",
            "sprint-8a-dashboard-dependency-contract.ps1",
            "Assert-Sprint8ADashboardDependencyEvidence",
            "prerequisite_receipts", "semanticPredicateRegistry",
            "Invoke-Sprint8AUatPredicate", "Read-UatLaneJsonEvidence",
            "Read-UatReferencedJsonEvidence", "Assert-UatDashboardDependencyEvidenceMatches",
            "canonicalProductDiagnostic", "Get-UatLaneEvidence",
            "diverge from the authenticated raw evidence",
            "playwright-acceptance.json", "assertion_ids",
            "semantic_predicate_registry_version", "semantic_failure_count",
            "harness_failure_count", 'classification = "product"', "Get-UatSemanticFailureEvidence",
            "Assert-Sprint8ASourceIdentityObject", "assertions_started", "assertions_started_at",
            "an assertion label without an executable semantic predicate",
            "exact viewport and theme matrix preserves directory editor detail and viewer usability",
            "Dataset provider outage retains unsaved editor state and one retry mutation",
            "Components configuration enforces exact schema authority projection and sanitized diagnostics"
        )
        "scripts/sprint-8a-lifecycle-chain.ps1" = @(
            "function Get-Sprint8APreflightCheckNames",
            '"receipt-chain"', '"repository-scope"', '"clean-source"',
            '"acceptance-traceability"', '"environment-contract"', '"database-contract"',
            '"deployment-contract"', '"downstream-command-contract"',
            '"evidence-path-contract"', '"evidence-inventory"',
            "function Get-Sprint8ASitLaneNames",
            '"static-and-boundaries"', '"rust-workspace"', '"playwright"', '"deployed-acceptance-smoke"',
            "function Get-Sprint8AManualUatScenarioNames", '"UAT-8A-{0:d2}"',
            "function Get-Sprint8AManualUatScenarioContract", "step_count", "scenario_contract",
            "function Assert-Sprint8AExactTerminalIdentities",
            "assertions_started", "command", "exit_status", "evidence",
            "function Get-Sprint8ACandidateIdentity",
            '[string]$NormalizedDeploymentConfigurationSha256',
            "acceptance_inventory_sha256", "deployment_inputs_sha256",
            "deployment_profile_sha256", "normalized_deployment_configuration_sha256", "migration_baseline_sha256", "expected_provenance",
            '"crates/tessara-component-module/manifest.json"',
            '"crates/tessara-dashboard-module/manifest.json"',
            '"crates/tessara-reference-module-sdk/manifest.json"',
            '"crates/tessara-reference-scoped-records/manifest.json"',
            "function Assert-Sprint8ALifecyclePrerequisite",
            "Resolve-Sprint8AEvidenceReference", "Assert-Sprint8AReceiptSidecar",
            "function Assert-Sprint8ALifecyclePrerequisiteSet",
            "function Publish-Sprint8ALifecycleReceipt",
            "function Assert-Sprint8AManualUatReceipt",
            "function Assert-Sprint8AManualUatExecutionLeasePair",
            "function Assert-Sprint8AManualUatResumeMarker",
            "function Open-Sprint8AManualUatScenarioLease",
            "function Assert-Sprint8ANoOpenManualUatLeases",
            "function Assert-Sprint8AManualUatPreparedPublicationCheckpoint",
            "function Repair-Sprint8AManualUatPreparedPublication",
            "function Repair-Sprint8AManualUatPreparedPublications",
            '[switch]$AllowMissingCanonicalUatCommitment',
            "function Publish-Sprint8AManualUatReceipt",
            '[Parameter(Mandatory)]$ExecutionLease', "uat-manual-execution-lease",
            'uat/attempt-$Attempt/manual', 'uat/attempt-$Attempt/manual-leases',
            'uat-manual-execution-resume', 'publication-prepared.json',
            "diagnostic", "duration_ms", "execution_lease", "execution_resume",
            "original_process_id", "current_process_id",
            "cannot publish an authoritative pass after execution resume",
            "function Get-Sprint8AEvidenceFileManifestEntries",
            "function Assert-Sprint8AEvidenceManifestCompleteness",
            "function Repair-Sprint8AEvidenceRootPublications",
            "unresolved publisher control file(s)",
            "function Get-Sprint8AEvidenceManifestReplacementPaths",
            '[string]$existing.sha256 -cne [string]$entry.sha256',
            '[string]$existing.phase -cne [string]$entry.phase',
            '$existing.authoritative -isnot [bool]',
            '[string]$existing.status -cne [string]$entry.status',
            "function Publish-Sprint8AEvidenceManifest", 'status -cne "committed"', '[switch]$Merge',
            '[AllowEmptyCollection()][string[]]$AuthorizedReplacementPaths = @()',
            '-AuthorizedReplacementPaths', 'without exact replacement authorization',
            'replacement authorization was not consumed by an exact metadata or digest change',
            '[switch]$PrepareOnly', '"$evidenceRootRelative/uat-result.json"',
            "function Test-Sprint8AEvidenceManifestContract",
            "function Test-Sprint8AManualUatAttemptAuthority",
            "function Test-Sprint8ALifecycleChain"
        )
        "scripts/run-sprint-8a-validation-preflight.ps1" = @(
            '[ValidateRange(1, 9999)][int]$Attempt', '[switch]$SelfTest',
            "function Get-Sprint8APreflightDeclarations",
            '"receipt-chain"', '"repository-scope"', '"clean-source"',
            '"acceptance-traceability"', '"environment-contract"', '"database-contract"',
            '"deployment-contract"', '"downstream-command-contract"',
            '"evidence-path-contract"', '"evidence-inventory"',
            "Open-Sprint8AValidationAttemptLock", "validation-attempt.lock",
            "Assert-Sprint8ACorrectionLineage", "correction_lineage",
            "Assert-Sprint8AReadinessCorrectionLineagePresence",
            "current_readiness_binding", "direct_correction_consumption",
            "clean_pre_rehearsal_supersession",
            "Assert-Sprint8ACurrentReadinessReference",
            "function Assert-Sprint8APreflightRehearsalAttemptReceipt",
            '"attempts/candidate-rehearsal-$attempt-attempt.json"',
            "attempt, result, and validation-state correction_lineage values differ",
            "same immutable Readiness prerequisite",
            "readiness_immutable_reference",
            "rehearsal_attempt_reference",
            '[int]$state.readiness.attempt -ne [int]$readiness.attempt',
            '[int]$state.rehearsal.attempt -ne [int]$rehearsal.attempt',
            "ExpectedCurrentReadiness", "RequireConsumedTip", 'receipt = [string]$readinessReference.path',
            "canonical_current_correction_lineage_authority",
            "function Invoke-Sprint8APreflightCheck", 'state = "harvesting"',
            "blocked by failed prerequisite(s)",
            "function Get-Sprint8APlannedEvidenceInventory",
            'attempts/sit-{attempt}-start.json',
            "Get-Sprint8AManualUatContractManifest", "Get-Sprint8AManualUatEvidencePlan",
            'foreach ($scenario in @(Get-Sprint8AManualUatScenarioNames))',
            "manual_contract_sources", '"uat/attempt-{attempt}/raw/"',
            '} else { "authenticated-producer" }', 'kind = "authenticated-assertion-raw"',
            'kind = "canonical-restoration"',
            "Sort-Object -Unique).Count -ne @(`$inventory.required).Count",
            "-cmatch '\{01\.\.08\}'",
            'uat/attempt-{attempt}/finalizations/run-{n}/finalization-completion-checkpoint.json',
            'uat/attempt-{attempt}/finalizations/run-{n}/result-commit.json',
            'uat/attempt-{attempt}/finalizations/run-{n}/publication-retry-checkpoint.json',
            'uat/attempt-{attempt}/publication-retries/run-{n}/result-commit.json',
            "run-sprint-8a-sit.ps1", "run-sprint-8a-formal-uat.ps1",
            "Get-Sprint8ACandidateIdentity", "Publish-Sprint8ALifecycleReceipt",
            "-NormalizedDeploymentConfigurationSha256",
            "Publish-Sprint8AEvidenceManifest",
            'mutable_attempt_checkpoints_are_overwritten_only_by_the_owning_runner',
            'immutable_snapshots_and_terminal_receipts_are_never_overwritten',
            'attempt lock was not acquired before evidence-root mutation',
            "complete exact declared-check graph",
            "declared graph differs from immutable start",
            "function Test-Sprint8AValidationPreflightRunner"
        )
        "scripts/run-sprint-8a-sit.ps1" = @(
            '[ValidateSet("Run", "Finalize")]', '[switch]$SelfTest',
            "function Get-Sprint8ASitLaneContracts", "Get-Sprint8ASitLaneNames",
            '"static-and-boundaries"', '"rust-workspace"', '"playwright"', '"deployed-acceptance-smoke"',
            "Open-Sprint8AValidationAttemptLock", "validation-attempt.lock",
            "exclusive evidence-root lock acquired before mutation",
            "function Set-Sprint8ASitAttemptState", '"harvesting"',
            "function Get-Sprint8ASitDefectBatch", "harvest_complete = `$true",
            "function Assert-Sprint8ASitRawResultsForFinalization",
            "finalization_only_retry_permitted_if_identity_remains_exact",
            "Publish-Sprint8ALifecycleReceipt", "-NormalizedDeploymentConfigurationSha256",
            "Get-Sprint8AEvidenceManifestReplacementPaths",
            "Publish-Sprint8AEvidenceManifest", "-AuthorizedReplacementPaths",
            "function Test-Sprint8ASitRunner"
        )
        "scripts/run-sprint-8a-formal-uat.ps1" = @(
            '[ValidateSet("Start", "Finalize")]', "[switch]`$SelfTest",
            'sprint-8a-lifecycle-chain.ps1',
            "function Get-Sprint8AFormalUatReference",
            "function Set-Sprint8AFormalUatAttemptState",
            "function Resolve-Sprint8AFormalUatFailureClassification",
            "function Publish-Sprint8AFormalUatResultCommit",
            "function Assert-Sprint8AFormalUatResultCommit",
            "function Publish-Sprint8AFormalUatFinalizationFailure",
            "function Invoke-Sprint8AFormalUatCatchHarvest",
            "function Assert-Sprint8AFormalUatReceiptBinding",
            "function Invoke-Sprint8AFormalUatCheck",
            "assertions_started", "assertions_started_at",
            "Test-Sprint8ALifecycleChain",
            'ExpectedPhase "validation-preflight"', 'ExpectedPhase "candidate-freeze"', 'ExpectedPhase "sit"',
            "Assert-Sprint8AExactTerminalIdentities", "Get-Sprint8ASitLaneNames",
            "Assert-Sprint8ASourceIdentityObject", "Get-Sprint8ACandidateIdentity",
            "Get-Sprint8AEnvironmentContract", '-State "executing" -Stage "manual"',
            '-State "executing" -Stage "diagnostic-manual"',
            '-State "finalizing" -Stage "canonical-restoration"',
            '-State "passed" -Stage "result-committed"',
            "Get-Sprint8AManualUatScenarioNames", "Assert-Sprint8AManualUatReceipt",
            "Assert-Sprint8ANoOpenManualUatLeases",
            "Repair-Sprint8AManualUatPreparedPublications",
            "Resolve-Sprint8AEvidenceReference", "Publish-Sprint8ALifecycleReceipt",
            "Get-Sprint8AEvidenceManifestReplacementPaths",
            "Publish-Sprint8AEvidenceManifest", "-AuthorizedReplacementPaths",
            '-NormalizedDeploymentConfigurationSha256', 'result = "canonical_topology_verified"',
            '[switch]$AuthorizeDisposableReset', 'attempts/uat-$Attempt.json',
            'uat/attempt-$Attempt', '-PrepareOnly', '-Merge',
            'materialize-sprint-8a.ps1', 'diagnostic-harvest-complete',
            "function Get-Sprint8AFormalUatCanonicalPairState",
            '"complete"', '"json-only"', '"sidecar-only"', '"absent"',
            "function Complete-Sprint8AFormalUatCanonicalPair",
            "function Assert-Sprint8AFormalUatFinalizationCompletionCheckpoint",
            "uat-finalization-completion", "finalization_completion_checkpoint",
            "finalization-completion-checkpoint.json", "publication-retry-checkpoint.json",
            'Join-Path $attemptRoot "publication-retries"',
            "function Invoke-Sprint8AFormalUatPublicationTail",
            "uat-finalization-retry", "finalization_retry", "consumed_at",
            "finalization-only publication retry passed without re-running manual validation or restoration"
        )
        "scripts/sprint-8a-dashboard-dependency-contract.ps1" = @(
            "Sprint8ADashboardDependencyCheckCodes", "Sprint8ADashboardDependencyActions",
            "Assert-Sprint8ADashboardDependencyEvidence", "canonical_reset_required"
        )
        "scripts/verify-sprint-6e-boundaries.ps1" = @(
            "tessara-dashboard-placement-renderer", "tessara-datasets-contract",
            "pub enum ComponentRenderResponse", "pub struct ComponentTableResponse",
            "pub struct ComponentVisualResponse", "Result<Json<ComponentRenderResponse>",
            "serde_json::from_slice", "validate_for", "body: Bytes",
            "require_json_content_type", "exact inbound body",
            "ProviderResourceAssertion::Required", "ResourceAuthorizationAssertionV2",
            "component_resource_assertion", "render_authorized_on_same_governing_node",
            "resource_assertion_is_authorized",
            "downstream_exchange_accepts_only_scope_authorized_resource_assertions",
            "authorized_dashboard_scope", "dashboard_scope_node_ids: authorized_dashboard_scope",
            "restrict_component_attempt_for_dashboard_projection",
            "dashboard_projection_redacts_disjoint_component_metadata",
            "editor_and_viewer_projection_use_their_independent_dashboard_capabilities",
            "replacement_scope_must_be_nonempty_canonical_and_contained_by_dashboard_scope",
            "Dashboard dependency refresh must apply MANAGE-capability joint-scope redaction",
            "altered_body.body.push(b' ');",
            "authorization must consume exact request bytes before typed JSON decoding"
        )
        "scripts/diagnose-sprint-8a-dashboard-dependencies.ps1" = @(
            "sprint-8a-dashboard-dependency-contract.ps1",
            "Assert-Sprint8ADashboardDependencyEvidence", "harvesting_complete",
            "exact_lifecycle_finding_placements", "provider_recovery_converges",
            '[string]$OutputPath', "Publish-DashboardDependencyEvidence",
            "raw fail-late evidence and its digest sidecar"
        )
    }
    foreach ($runner in $validationContracts.Keys) {
        $runnerPath = Join-Path $repoRoot $runner
        if (-not (Test-Path -LiteralPath $runnerPath -PathType Leaf)) {
            throw "Sprint 8A validation contract runner '$runner' is missing."
        }
        $runnerText = Get-Content -LiteralPath $runnerPath -Raw
        foreach ($fragment in $validationContracts[$runner]) {
            if (-not $runnerText.Contains($fragment)) {
                throw "Sprint 8A validation contract '$runner' omits '$fragment'."
            }
        }
        $runnerTokens = $null
        $runnerParseErrors = $null
        [void][Management.Automation.Language.Parser]::ParseFile(
            $runnerPath,
            [ref]$runnerTokens,
            [ref]$runnerParseErrors
        )
        if (@($runnerParseErrors).Count -ne 0) {
            throw "Sprint 8A validation contract runner '$runner' does not parse: $($runnerParseErrors.Message -join '; ')."
        }
    }
    $validationEnvironmentText = Get-Content -LiteralPath (
        Join-Path $repoRoot "scripts/sprint-8a-validation-environment.ps1"
    ) -Raw
    $supersessionHelperIndex = $validationEnvironmentText.IndexOf(
        'function Assert-Sprint8AReadinessSupersessionChain',
        [StringComparison]::Ordinal
    )
    $supersessionHelperEndIndex = $validationEnvironmentText.IndexOf(
        'function ConvertTo-Sprint8ACorrectionLineage',
        $supersessionHelperIndex,
        [StringComparison]::Ordinal
    )
    if ($supersessionHelperIndex -lt 0 -or $supersessionHelperEndIndex -le $supersessionHelperIndex) {
        throw "Sprint 8A validation must have one central Readiness supersession verifier."
    }
    $supersessionHelper = $validationEnvironmentText.Substring(
        $supersessionHelperIndex,
        $supersessionHelperEndIndex - $supersessionHelperIndex
    )
    foreach ($fragment in @(
        '($document.schema_version -isnot [int] -and $document.schema_version -isnot [long])',
        '@(2, 3) -notcontains [int]$document.schema_version',
        '[string]$document.sprint -cne "sprint-8a"',
        '$document.authoritative -ne $false',
        '$document.assertions_started -ne $true',
        '($startDocument.schema_version -isnot [int] -and $startDocument.schema_version -isnot [long])',
        '[int]$startDocument.schema_version -ne 2',
        '$prerequisites.Count -ne 1',
        '$startPrerequisites.Count -ne 1',
        '$cursorDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization"',
        '$cursorDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt"',
        '$startDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization"',
        '$startDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt"',
        'predecessor_correction_authorization',
        'correction_consumption_receipt',
        'direct_correction_consumption',
        'clean_pre_rehearsal_supersession',
        '[int]$cursorDocument.attempt -ne ([int]$predecessor.document.attempt + 1)',
        'changed correction lineage between sequential clean edges'
    )) {
        if (-not $supersessionHelper.Contains($fragment)) {
            throw "Central Readiness supersession verification omits '$fragment'."
        }
    }
    foreach ($runner in @(
        "scripts/run-sprint-8a-candidate-rehearsal.ps1",
        "scripts/run-sprint-8a-validation-preflight.ps1"
    )) {
        $runnerText = Get-Content -LiteralPath (Join-Path $repoRoot $runner) -Raw
        foreach ($fragment in @(
            'Assert-Sprint8AReadinessCorrectionLineagePresence',
            '-ExpectedCurrentReadiness',
            '$currentReadinessBinding = $lineageValidation.current_readiness_binding',
            'if ([string]$currentReadinessBinding.kind -ceq "direct_correction_consumption")',
            '} elseif ([string]$currentReadinessBinding.kind -ceq "clean_pre_rehearsal_supersession")'
        )) {
            if (-not $runnerText.Contains($fragment)) {
                throw "Sprint 8A runner '$runner' does not consume the central current-Readiness lineage binding through '$fragment'."
            }
        }
    }
    . (Join-Path $repoRoot "scripts/sprint-8a-lifecycle-chain.ps1")
    foreach ($contract in @(
        [pscustomobject]@{ command = "Get-Sprint8ACandidateIdentity"; parameter = "NormalizedDeploymentConfigurationSha256" },
        [pscustomobject]@{ command = "Assert-Sprint8ALifecyclePrerequisiteSet"; parameter = "NormalizedDeploymentConfigurationSha256" },
        [pscustomobject]@{ command = "Publish-Sprint8ALifecycleReceipt"; parameter = "NormalizedDeploymentConfigurationSha256" },
        [pscustomobject]@{ command = "Publish-Sprint8AEvidenceManifest"; parameter = "AuthorizedReplacementPaths" },
        [pscustomobject]@{ command = "Publish-Sprint8AManualUatReceipt"; parameter = "ExecutionLease" },
        [pscustomobject]@{ command = "Repair-Sprint8AManualUatPreparedPublications"; parameter = "Attempt" },
        [pscustomobject]@{ command = "Repair-Sprint8AManualUatPreparedPublications"; parameter = "AllowMissingCanonicalUatCommitment" }
    )) {
        if (-not (Get-Command $contract.command).Parameters.ContainsKey($contract.parameter)) {
            throw "Sprint 8A lifecycle command '$($contract.command)' omits required parameter '$($contract.parameter)'."
        }
    }
    Test-Sprint8ALifecycleChain | Out-Null
    & (Join-Path $repoRoot "scripts/run-sprint-8a-validation-preflight.ps1") -SelfTest | Out-Null
    & (Join-Path $repoRoot "scripts/run-sprint-8a-sit.ps1") -SelfTest | Out-Null
    $lifecycleChainText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/sprint-8a-lifecycle-chain.ps1") -Raw
    if ($lifecycleChainText.Contains('"crates/tessara-reference-module/manifest.json"')) {
        throw "Sprint 8A candidate identity must bind the complete real tracked Module Manifest inventory."
    }
    $formalUatText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/run-sprint-8a-formal-uat.ps1") -Raw
    if ($formalUatText.Contains('$candidateReceiptObject.mutable_source_identity') -or
        -not $formalUatText.Contains('$candidateReceiptObject.source_identity')) {
        throw "Formal UAT must compare source against the canonical candidate lifecycle source_identity."
    }
    $rehearsalUatPhase = "candidate-rehearsal-" + "uat-diagnostics"
    $diagnosticUatRunner = "uat-sprint-" + "8a.ps1"
    if ($formalUatText.Contains($rehearsalUatPhase) -or $formalUatText.Contains($diagnosticUatRunner)) {
        throw "Formal UAT must not consume or relabel Candidate Rehearsal diagnostic UAT evidence."
    }
    foreach ($requiredCall in @(
        "Assert-Sprint8AFormalUatFinalizationCompletionCheckpoint",
        "Invoke-Sprint8AFormalUatPublicationTail",
        "Invoke-Sprint8AFormalUatCatchHarvest"
    )) {
        if ([regex]::Matches($formalUatText, [regex]::Escape($requiredCall)).Count -lt 2) {
            throw "Formal UAT declares '$requiredCall' but does not consume it from executable orchestration."
        }
    }
    foreach ($pairState in @("complete", "json-only", "sidecar-only", "absent")) {
        if (-not $formalUatText.Contains('"' + $pairState + '"')) {
            throw "Formal UAT canonical result recovery omits '$pairState'."
        }
    }
    & (Join-Path $repoRoot "scripts/run-sprint-8a-formal-uat.ps1") -SelfTest | Out-Null
    $rehearsalRunnerText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/run-sprint-8a-candidate-rehearsal.ps1") -Raw
    foreach ($powerShellChildRunner in @(
        "scripts/run-sprint-8a-candidate-rehearsal.ps1",
        "scripts/verify-sprint-8a-component-upgrade.ps1"
    )) {
        $powerShellChildRunnerText = Get-Content -LiteralPath (Join-Path $repoRoot $powerShellChildRunner) -Raw
        if ([regex]::IsMatch(
            $powerShellChildRunnerText,
            '(?m)&[^\r\n]*\.ps1[^\r\n]*(?:\r?\n[ \t]*)?if[ \t]*\(\$LASTEXITCODE'
        )) {
            throw "Sprint 8A runner '$powerShellChildRunner' uses native LASTEXITCODE to classify a PowerShell child script."
        }
    }
    $immutableStartFragment = 'Publish-Sprint7AEvidence -Document $startDocument -OutputPath $startPath'
    $reservationLockFragment = '$validationLockHandle = Open-Sprint8AValidationAttemptLock -Path $validationLockPath'
    $attemptStartFragment = 'Publish-Sprint7AEvidence -Document $attemptReceipt -OutputPath $attemptPath'
    $stateCaptureFragment = 'Publish-Sprint7AEvidence -Document $prelaunchStateCapture -OutputPath $stateSnapshotPath'
    $stateLaneFragment = 'Invoke-RehearsalLane "attempt-state-prerequisite"'
    $prerequisiteLaneFragment = 'Invoke-RehearsalLane "validation-readiness-prerequisite"'
    $immutableStartIndex = $rehearsalRunnerText.LastIndexOf($immutableStartFragment, [StringComparison]::Ordinal)
    $reservationLockIndex = $rehearsalRunnerText.LastIndexOf($reservationLockFragment, [StringComparison]::Ordinal)
    $executionStartIndex = $rehearsalRunnerText.LastIndexOf('if ($Attempt -lt 1)', [StringComparison]::Ordinal)
    $attemptStartIndex = $rehearsalRunnerText.IndexOf($attemptStartFragment, $immutableStartIndex, [StringComparison]::Ordinal)
    $stateCaptureIndex = $rehearsalRunnerText.IndexOf($stateCaptureFragment, [StringComparison]::Ordinal)
    $stateLaneIndex = $rehearsalRunnerText.LastIndexOf($stateLaneFragment, [StringComparison]::Ordinal)
    $prerequisiteLaneIndex = $rehearsalRunnerText.LastIndexOf($prerequisiteLaneFragment, [StringComparison]::Ordinal)
    if ([regex]::Matches($rehearsalRunnerText, '(?m)^\$declaredChecks\s*=\s*@\(').Count -ne 1 -or
        $executionStartIndex -lt 0 -or $reservationLockIndex -le $executionStartIndex -or
        $reservationLockIndex -ge $stateCaptureIndex -or
        $stateCaptureIndex -lt 0 -or $immutableStartIndex -le $stateCaptureIndex -or
        $attemptStartIndex -le $immutableStartIndex -or $stateLaneIndex -le $attemptStartIndex -or
        $prerequisiteLaneIndex -le $attemptStartIndex) {
        throw "Candidate Rehearsal must reserve the attempt namespace, declare one check graph, retain prelaunch state, and publish immutable schedule plus mutable attempt before executing lifecycle prerequisites."
    }
    foreach ($independentLane in @(
        "attempt-state-prerequisite", "validation-readiness-prerequisite",
        "formatting", "workspace-check", "workspace-clippy", "compose-manifest-schema-contract",
        "web-native-wasm-source-boundaries", "module-sdk-boundaries", "dashboard-source-boundaries",
        "markdown-links", "workspace-tests", "optimized-resource-reference-timing",
        "components-contract-tests", "dashboard-module-tests", "component-conformance-nondisclosure",
        "module-testkit-conformance", "playwright-discovery"
    )) {
        $independentPattern = '(?s)name\s*=\s*"' + [regex]::Escape($independentLane) + '";\s*depends_on\s*=\s*@\(\)'
        if ($rehearsalRunnerText -notmatch $independentPattern) {
            throw "Candidate Rehearsal must keep safe diagnostic lane '$independentLane' independent of fallible readiness prerequisites."
        }
    }
    if ($executionStartIndex -lt 0 -or $executionStartIndex -ge $immutableStartIndex) {
        throw "Candidate Rehearsal does not expose a distinct post-self-test execution boundary."
    }
    $preStartText = $rehearsalRunnerText.Substring($executionStartIndex, $immutableStartIndex - $executionStartIndex)
    if ($preStartText.Contains('-ProbeDatabases') -or
        $preStartText.Contains('Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot')) {
        throw "Candidate Rehearsal probes mutable source or environment before retaining its immutable scheduled start receipt."
    }
    foreach ($fragment in @(
        '$scheduleSha = Get-Sprint8ARehearsalJsonSha256',
        'Assert-Sprint8ARehearsalScheduleContract',
        'source = "conservative_full_harvest"',
        'foreach ($name in @($selectedSchedule.wave_a))',
        'foreach ($name in @($selectedSchedule.cleanup_sinks))',
        'Add-RehearsalDeferredLane',
        'foreach ($name in @($selectedSchedule.aggregate_sinks))',
        'foreach ($name in @($selectedSchedule.finalizers))',
        'Complete-RehearsalOrphanedLane',
        'Assert-Sprint8ARehearsalTerminalAccounting'
    )) {
        if (-not $rehearsalRunnerText.Contains($fragment)) {
            throw "Candidate Rehearsal bounded failure-first enforcement omits '$fragment'."
        }
    }
    $readinessLaneExtent = $rehearsalRunnerText.Substring(
        $prerequisiteLaneIndex,
        $rehearsalRunnerText.IndexOf('Invoke-RehearsalLane "formatting"', $prerequisiteLaneIndex, [StringComparison]::Ordinal) - $prerequisiteLaneIndex
    )
    foreach ($stateLaneDependency in @(
        'runtimeContext.validation_state',
        'runtimeContext.readiness_immutable_reference',
        '$stateIndex',
        '$statePath'
    )) {
        if ($readinessLaneExtent.Contains($stateLaneDependency)) {
            throw "The independent readiness/source/environment lane must not consume state-lane dependency '$stateLaneDependency'."
        }
    }
    if (-not $readinessLaneExtent.Contains('Resolve-Sprint8AEvidenceReference')) {
        throw "The independent readiness/source/environment lane must resolve corrected-consumption evidence inside the evidence root."
    }
    foreach ($destructiveLane in @("source-exact-materialization-no-op", "failure-containment-successor-health")) {
        $destructivePattern = '(?s)name\s*=\s*"' + [regex]::Escape($destructiveLane) + '";\s*depends_on\s*=\s*@\("attempt-state-prerequisite",\s*"validation-readiness-prerequisite"\)'
        if ($rehearsalRunnerText -notmatch $destructivePattern) {
            throw "Candidate Rehearsal destructive lane '$destructiveLane' must depend on both state-lock and readiness authentication."
        }
    }
    $productDiagnosticRunner = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/diagnose-sprint-8a-product.ps1") -Raw
    foreach ($fragment in @(
        "candidate-product-diagnostic", "authoritative = `$false", "formal_uat = `$false",
        "acceptance_evidence_published = `$false", "smoke-sprint-8a.ps1",
        "required_by_later_failure-containment-successor-materialization",
        "sprint-8a-dashboard-dependency-contract.ps1",
        "diagnose-sprint-8a-dashboard-dependencies.ps1",
        "Assert-Sprint8ADashboardDependencyEvidence", "dashboard_dependency_semantics",
        "dashboard_dependency_semantics_evidence", "Get-RetainedDiagnosticArtifact",
        "raw_evidence = `$dashboardDependencySemanticsEvidence"
    )) {
        if (-not $productDiagnosticRunner.Contains($fragment)) {
            throw "Sprint 8A non-acceptance product diagnostic omits '$fragment'."
        }
    }
    if ($productDiagnosticRunner.Contains("./scripts/uat-sprint.ps1") -or
        $productDiagnosticRunner.Contains("-DevelopmentMode")) {
        throw "Sprint 8A product diagnostics must not invoke the legacy demo-seed UAT runner."
    }
    $candidateRunner = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/run-sprint-8a-candidate-rehearsal.ps1") -Raw
    if ($candidateRunner.Contains("AcceptanceEvidencePath") -or
        $candidateRunner.Contains("./scripts/uat-sprint.ps1")) {
        throw "Candidate rehearsal must not invoke formal UAT or publish acceptance evidence."
    }
    if (-not $candidateRunner.Contains("--profile reference config --quiet")) {
        throw "Candidate rehearsal must validate Compose quietly so normalized runtime secrets never enter retained logs."
    }
    $readinessRunner = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/validate-sprint-8a-readiness.ps1") -Raw
    $readinessSelfTestIndex = $readinessRunner.IndexOf('if ($SelfTest)', [StringComparison]::Ordinal)
    $readinessSelfTestEndIndex = $readinessRunner.IndexOf(
        'if ($Attempt -lt 1)',
        $readinessSelfTestIndex,
        [StringComparison]::Ordinal
    )
    if ($readinessSelfTestIndex -lt 0 -or $readinessSelfTestEndIndex -le $readinessSelfTestIndex) {
        throw "Validation Readiness must retain one bounded no-attempt self-test branch."
    }
    $readinessSelfTestBlock = $readinessRunner.Substring(
        $readinessSelfTestIndex,
        $readinessSelfTestEndIndex - $readinessSelfTestIndex
    )
    foreach ($requiredSelfTestCall in @(
        'Test-Sprint8AReadinessSupersessionContract',
        'Test-Sprint8AFailedReadinessFinalization'
    )) {
        if (-not $readinessSelfTestBlock.Contains($requiredSelfTestCall)) {
            throw "Validation Readiness self-test branch does not invoke '$requiredSelfTestCall'."
        }
    }
    $readinessStartReceiptIndex = $readinessRunner.IndexOf('$startReceipt = [ordered]@{', [StringComparison]::Ordinal)
    $readinessTerminalReceiptIndex = $readinessRunner.IndexOf('$receipt = [ordered]@{', [StringComparison]::Ordinal)
    if ($readinessStartReceiptIndex -lt 0 -or $readinessTerminalReceiptIndex -le $readinessStartReceiptIndex) {
        throw "Validation Readiness must retain distinct immutable-start and terminal receipt producers."
    }
    $readinessStartReceiptBlock = $readinessRunner.Substring(
        $readinessStartReceiptIndex,
        [Math]::Min(320, $readinessRunner.Length - $readinessStartReceiptIndex)
    )
    $readinessTerminalReceiptBlock = $readinessRunner.Substring(
        $readinessTerminalReceiptIndex,
        [Math]::Min(320, $readinessRunner.Length - $readinessTerminalReceiptIndex)
    )
    if ($readinessStartReceiptBlock -notmatch '(?m)^\s*schema_version\s*=\s*2\s*$' -or
        $readinessTerminalReceiptBlock -notmatch '(?m)^\s*schema_version\s*=\s*3\s*$') {
        throw "Validation Readiness must publish an exact schema-2 immutable start and the current schema-3 terminal receipt."
    }
    if ([regex]::Matches(
            $readinessRunner,
            [regex]::Escape('prerequisite_receipts = @($readinessPrerequisiteReceipts)')
        ).Count -lt 2) {
        throw "Validation Readiness clean reruns must retain the same reserved immutable predecessor in both start and terminal receipts."
    }
    if (-not $readinessRunner.Contains('$startReceipt.prerequisite_receipts = @($readinessPrerequisiteReceipts)')) {
        throw "Validation Readiness checkpoints must retain the reserved immutable predecessor."
    }
    if ([regex]::Matches(
            $readinessRunner,
            [regex]::Escape('correction_lineage = $receiptCorrectionLineage')
        ).Count -lt 2 -or
        -not $readinessRunner.Contains('$startReceipt.correction_lineage = $receiptCorrectionLineage')) {
        throw "Validation Readiness clean start, checkpoint, and terminal receipts must retain only the reserved correction lineage."
    }
    foreach ($fragment in @(
        '$correctionLineage = $launchReservation.preserved_correction_lineage',
        '$receiptCorrectionLineage = Get-Sprint8AReadinessReceiptCorrectionLineage -Reservation $launchReservation',
        '$readinessPrerequisiteReceipts = @($launchReservation.prerequisite_receipts)'
    )) {
        if (-not $readinessRunner.Contains($fragment)) {
            throw "Validation Readiness does not initialize clean-rerun receipt state from its locked reservation through '$fragment'."
        }
    }
    $attemptStateCheckIndex = $readinessRunner.IndexOf(
        'Invoke-ReadinessCheck "attempt-state-prerequisite"',
        [StringComparison]::Ordinal
    )
    $cleanRerunBranchIndex = $readinessRunner.IndexOf(
        'if ([bool]$launchReservation.clean_rerun)',
        $attemptStateCheckIndex,
        [StringComparison]::Ordinal
    )
    $cleanRerunBranchEndIndex = $readinessRunner.IndexOf(
        '$lineageValidation = $launchReservation.lineage_validation',
        $cleanRerunBranchIndex,
        [StringComparison]::Ordinal
    )
    if ($attemptStateCheckIndex -lt 0 -or $cleanRerunBranchIndex -lt 0 -or
        $cleanRerunBranchEndIndex -le $cleanRerunBranchIndex) {
        throw "Validation Readiness must expose a bounded clean-rerun branch before correction consumption."
    }
    $cleanRerunBranch = $readinessRunner.Substring(
        $cleanRerunBranchIndex,
        $cleanRerunBranchEndIndex - $cleanRerunBranchIndex
    )
    foreach ($fragment in @(
        '@($script:readinessPrerequisiteReceipts).Count -ne 1',
        '$null -ne $script:predecessorCorrectionAuthorization',
        '$null -ne $script:correctionConsumptionReceipt',
        '$launchReservation.preserved_correction_lineage',
        'return'
    )) {
        if (-not $cleanRerunBranch.Contains($fragment)) {
            throw "Validation Readiness clean-rerun branch omits '$fragment'."
        }
    }
    if ($cleanRerunBranch.Contains('Publish-Sprint8AAppendOnlyJsonReceipt') -or
        $cleanRerunBranch.Contains('consumptionDocument =') -or
        $cleanRerunBranch.Contains('consumed_by_readiness =')) {
        throw "A clean Readiness supersession must not create another correction consumption."
    }
    $readinessReservationHelperIndex = $readinessRunner.IndexOf(
        'function Open-Sprint8AReadinessAttemptReservation',
        [StringComparison]::Ordinal
    )
    $readinessReservationHelperEndIndex = $readinessRunner.IndexOf(
        'function Publish-Sprint8AReadinessCreateOnceOrVerify',
        $readinessReservationHelperIndex,
        [StringComparison]::Ordinal
    )
    if ($readinessReservationHelperIndex -lt 0 -or
        $readinessReservationHelperEndIndex -le $readinessReservationHelperIndex) {
        throw "Validation Readiness must expose one testable locked reservation helper."
    }
    $readinessReservationHelper = $readinessRunner.Substring(
        $readinessReservationHelperIndex,
        $readinessReservationHelperEndIndex - $readinessReservationHelperIndex
    )
    $readinessHelperLockIndex = $readinessReservationHelper.IndexOf(
        '$lockHandle = Open-Sprint8AValidationAttemptLock -Path $LockPath',
        [StringComparison]::Ordinal
    )
    $readinessHelperStateIndex = $readinessReservationHelper.IndexOf(
        'Get-Content -LiteralPath $StatePath',
        $readinessHelperLockIndex,
        [StringComparison]::Ordinal
    )
    $readinessHelperAuthorizationIndex = $readinessReservationHelper.IndexOf(
        '$reservation = Get-Sprint8AReadinessAttemptReservation',
        $readinessHelperStateIndex,
        [StringComparison]::Ordinal
    )
    $readinessHelperNamespaceIndex = $readinessReservationHelper.IndexOf(
        '$namespaceCollisions = @(',
        $readinessHelperAuthorizationIndex,
        [StringComparison]::Ordinal
    )
    $readinessHelperLogIndex = $readinessReservationHelper.IndexOf(
        '[IO.Directory]::CreateDirectory($AttemptLogRoot)',
        $readinessHelperNamespaceIndex,
        [StringComparison]::Ordinal
    )
    if ($readinessHelperLockIndex -lt 0 -or
        $readinessHelperStateIndex -le $readinessHelperLockIndex -or
        $readinessHelperAuthorizationIndex -le $readinessHelperStateIndex -or
        $readinessHelperNamespaceIndex -le $readinessHelperAuthorizationIndex -or
        $readinessHelperLogIndex -le $readinessHelperNamespaceIndex) {
        throw "Validation Readiness reservation must lock, authenticate state/authorization, reject all namespace collisions, and only then create attempt storage."
    }
    $readinessMainBoundaryIndex = $readinessRunner.IndexOf('if ($Attempt -lt 1)', [StringComparison]::Ordinal)
    $readinessFinalizerIndex = $readinessRunner.IndexOf('if ($FinalizeFailedAttempt)', $readinessMainBoundaryIndex, [StringComparison]::Ordinal)
    $readinessFinalizerReturnIndex = $readinessRunner.IndexOf('    return', $readinessFinalizerIndex, [StringComparison]::Ordinal)
    $readinessLiveReservationIndex = $readinessRunner.IndexOf(
        '$reservationHandle = Open-Sprint8AReadinessAttemptReservation',
        $readinessFinalizerIndex,
        [StringComparison]::Ordinal
    )
    if ($readinessMainBoundaryIndex -lt 0 -or $readinessFinalizerIndex -le $readinessMainBoundaryIndex -or
        $readinessFinalizerReturnIndex -le $readinessFinalizerIndex -or
        $readinessLiveReservationIndex -le $readinessFinalizerReturnIndex) {
        throw "Validation Readiness failed-attempt finalization must complete before a normal attempt reserves its namespace."
    }
    $readinessStartIndex = $readinessRunner.IndexOf(
        'Publish-Sprint7AEvidence -Document $startReceipt -OutputPath $attemptPath',
        $readinessLiveReservationIndex,
        [StringComparison]::Ordinal
    )
    $readinessStateLaneIndex = $readinessRunner.IndexOf('Invoke-ReadinessCheck "attempt-state-prerequisite"', [StringComparison]::Ordinal)
    $readinessSourceIndex = $readinessRunner.IndexOf(
        'Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot',
        $readinessStartIndex,
        [StringComparison]::Ordinal
    )
    if ($readinessStartIndex -le $readinessLiveReservationIndex -or
        $readinessStateLaneIndex -le $readinessStartIndex -or
        $readinessSourceIndex -le $readinessStartIndex -or
        [regex]::Matches($readinessRunner, 'Checkpoint-ReadinessAttempt').Count -lt 3) {
        throw "Validation Readiness must reserve an authorized namespace under lock, publish its unverified start before source/assertion work, and hash-checkpoint every terminal result."
    }
    $readinessPreStartText = $readinessRunner.Substring(
        $readinessMainBoundaryIndex,
        $readinessStartIndex - $readinessMainBoundaryIndex
    )
    if ($readinessPreStartText.Contains('Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot') -or
        -not $readinessPreStartText.Contains('$reservationHandle = Open-Sprint8AReadinessAttemptReservation') -or
        -not $readinessPreStartText.Contains('-StatePath $statePath') -or
        -not $readinessPreStartText.Contains('-AttemptReceiptPath $attemptPath') -or
        -not $readinessPreStartText.Contains('-StartReceiptPath $startSnapshotPath') -or
        -not $readinessPreStartText.Contains('-AttemptLogRoot $logRoot')) {
        throw "Validation Readiness must authenticate state and all five namespace targets before start publication without probing mutable source."
    }
    $composeProbeInvocationIndex = $readinessRunner.IndexOf('Invoke-ReadinessCheck "compose-database-contract"', [StringComparison]::Ordinal)
    $toolchainInvocationIndex = $readinessRunner.IndexOf('Invoke-ReadinessCheck "toolchain"', [StringComparison]::Ordinal)
    $npmInvocationIndex = $readinessRunner.IndexOf('Invoke-ReadinessCheck "playwright-locked-install"', [StringComparison]::Ordinal)
    if ($readinessRunner -notmatch '(?s)name\s*=\s*"compose-database-contract";\s*depends_on\s*=\s*@\(\)' -or
        $readinessRunner -notmatch '(?s)name\s*=\s*"environment-contract";\s*depends_on\s*=\s*@\("toolchain",\s*"playwright-locked-install",\s*"compose-database-contract"\)' -or
        $readinessRunner -match 'Get-Sprint8AEnvironmentContract[^\r\n]*-ProbeDatabases' -or
        $composeProbeInvocationIndex -lt 0 -or
        $toolchainInvocationIndex -lt 0 -or
        $npmInvocationIndex -lt 0 -or
        $composeProbeInvocationIndex -gt $toolchainInvocationIndex -or
        $composeProbeInvocationIndex -gt $npmInvocationIndex) {
        throw "Validation Readiness must harvest Compose and authenticated six-database evidence independently before toolchain/npm-dependent environment finalization."
    }
    $environmentContractPath = Join-Path $repoRoot "scripts/sprint-8a-validation-environment.ps1"
    $environmentTokens = $null
    $environmentParseErrors = $null
    $environmentAst = [Management.Automation.Language.Parser]::ParseFile(
        $environmentContractPath,
        [ref]$environmentTokens,
        [ref]$environmentParseErrors
    )
    $deploymentProbeFunctions = @($environmentAst.FindAll({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -ceq "Get-Sprint8ADeploymentEnvironmentProbe"
    }, $true))
    if ($environmentParseErrors.Count -ne 0 -or $deploymentProbeFunctions.Count -ne 1 -or
        $deploymentProbeFunctions[0].Extent.Text -match '(?i)Get-Sprint8AToolVersion|Get-Sprint8AToolchainEnvironmentContract|["''](?:npm|node|cargo|rustc)["'']') {
        throw "The independent Compose/six-database probe must not execute Rust/Node/npm/Playwright toolchain discovery."
    }
    $protocolClassifications = @(
        "preflight/setup", "product", "harness", "environment", "flaky",
        "evidence-finalization", "product-decision"
    )
    $assignedContainmentClassifications = @(
        [regex]::Matches($failureContainmentText, '\$defectClassification\s*=\s*"(?<classification>[^"]+)"') |
            ForEach-Object { $_.Groups['classification'].Value }
    )
    if ($assignedContainmentClassifications.Count -eq 0 -or
        @($assignedContainmentClassifications | Where-Object { $protocolClassifications -cnotcontains $_ }).Count -gt 0) {
        throw "Failure-containment defect classification assignments must use only the canonical validation-protocol vocabulary."
    }
    $coreDatabaseSource = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/src/db.rs") -Raw
    $startupCapabilityCatalog = [regex]::Match(
        $coreDatabaseSource,
        '(?s)let capabilities = \[(?<catalog>.*?)\];'
    )
    $builtInRoleCatalog = [regex]::Match(
        $coreDatabaseSource,
        '(?s)BUILT_IN_ROLE_CAPABILITY_SEED:\s*&\[\(&str,\s*&\[&str\]\)\]\s*=\s*&\[(?<catalog>.*?)\];'
    )
    if (-not $startupCapabilityCatalog.Success -or -not $builtInRoleCatalog.Success) {
        throw "Sprint 8A cannot locate the exact Core startup capability and built-in-role seed contracts."
    }
    foreach ($catalog in @($startupCapabilityCatalog.Groups['catalog'].Value, $builtInRoleCatalog.Groups['catalog'].Value)) {
        if ($catalog -match '(?i)components:(?:read|manage)|dashboards:(?:read|manage)') {
            throw "Core startup must not seed independent Component or Dashboard capabilities or built-in grants; manifests and Blueprint role policy own them."
        }
    }
    $coreBaselineText = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-api/migrations/001_baseline.sql") -Raw
    if (-not $coreBaselineText.Contains("CREATE TABLE core_module_action_declarations") -or
        $coreBaselineText -match '(?im)^\s*INSERT\s+INTO\s+core_module_action_declarations') {
        throw "Core baseline must retain the generic action-declaration table without statically seeding module-owned actions."
    }
    foreach ($forbiddenIdentity in @(
        'tessara.reference.scoped-records',
        'tessara.dashboards',
        'tessara.components.component-version'
    )) {
        if ($coreBaselineText.Contains($forbiddenIdentity)) {
            throw "Core baseline contains module-owned action declaration identity '$forbiddenIdentity'; Manifest enrollment must project it."
        }
    }
    $testChangeLogText = Get-Content -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8a-test-change-log.md") -Raw
    foreach ($requiredFragment in @(
        'sprint-8a-role-capabilities-v1+sha256.4f607b6f428c',
        '4f607b6f428c0de70901dd119f7026b4c700c9e86309e76a3f5085a4da366609',
        'demo_seed_uses_capability_scope_ownership_and_components',
        'tessara-component-module/tests/product_integration.rs'
    )) {
        if (-not $testChangeLogText.Contains($requiredFragment)) {
            throw "Sprint 8A test-change log omits owner-cutover rationale fragment '$requiredFragment'."
        }
    }
    foreach ($orchestrator in @(
        "scripts/validate-sprint-8a-readiness.ps1",
        "scripts/run-sprint-8a-candidate-rehearsal.ps1"
    )) {
        $orchestratorText = Get-Content -LiteralPath (Join-Path $repoRoot $orchestrator) -Raw
        $invokedRunners = @(
            [regex]::Matches($orchestratorText, '(?m)&\s+(?:\.\/)?(?<path>scripts\/[A-Za-z0-9._-]+\.ps1)') |
                ForEach-Object { $_.Groups['path'].Value } |
                Sort-Object -Unique
        )
        if ($orchestrator -in @(
            "scripts/validate-sprint-8a-readiness.ps1",
            "scripts/run-sprint-8a-candidate-rehearsal.ps1"
        )) {
            # The harvest guard is invoked through its absolute PSScriptRoot
            # path after Pop-Location, so it is not discoverable by the static
            # relative-call pattern above. Keep it inside the same no-exit
            # enforcement boundary explicitly.
            $invokedRunners = @($invokedRunners + "scripts/test-sprint-validation-harvest.ps1" | Sort-Object -Unique)
        }
        foreach ($invokedRunner in $invokedRunners) {
            $tokens = $null
            $parseErrors = $null
            $runnerAst = [Management.Automation.Language.Parser]::ParseFile(
                (Join-Path $repoRoot $invokedRunner),
                [ref]$tokens,
                [ref]$parseErrors
            )
            if ($parseErrors.Count -ne 0) {
                throw "Fail-late child runner '$invokedRunner' does not parse."
            }
            $exitStatements = @($runnerAst.FindAll({
                param($node)
                $node -is [Management.Automation.Language.ExitStatementAst]
            }, $true))
            if ($exitStatements.Count -gt 0) {
                throw "Fail-late orchestrator '$orchestrator' invokes '$invokedRunner', which can terminate the parent host with exit. Use a catchable failure instead."
            }
        }
    }

    $blueprint = Get-Content -LiteralPath (Join-Path $repoRoot "deploy/sprint-8a/blueprints/reference.json") -Raw | ConvertFrom-Json
    $coreBootstrap = $blueprint.core.bootstrap.value
    if (@($coreBootstrap.dataset_rows).Count -ne 30 -or
        @($coreBootstrap.dataset_rows | Where-Object row_id -CLike 'uat7a-page-*').Count -ne 26 -or
        @($coreBootstrap.additional_datasets).Count -ne 1) {
        throw "Sprint 8A Core bootstrap must own the exact 30-row primary Dataset and blocked Dataset seed."
    }
    $componentBootstrap = @($blueprint.modules | Where-Object definition_id -CEQ 'tessara.components')[0].bootstrap.value
    $expectedComponentKeys = @(
        'sprint-8a-blocked-component', 'sprint-8a-label-bar', 'sprint-8a-label-donut',
        'sprint-8a-label-line', 'sprint-8a-label-pie', 'sprint-8a-record-table', 'sprint-8a-row-count'
    ) | Sort-Object
    $actualComponentKeys = @($componentBootstrap.components.external_key | Sort-Object)
    if (@($componentBootstrap.components).Count -ne 7 -or
        @($componentBootstrap.components.versions).Count -ne 8 -or
        ($actualComponentKeys -join ',') -cne ($expectedComponentKeys -join ',')) {
        throw "Sprint 8A Component bootstrap does not own the exact canonical Component seed inventory."
    }
    $componentVersionKeys = @(
        $componentBootstrap.components | ForEach-Object { @($_.versions).resource_key }
    ) | Sort-Object
    $validationItems = @($componentBootstrap.dependency_validation.items)
    $validationKeys = @($validationItems.validation_key | Sort-Object)
    if ([int]$componentBootstrap.dependency_validation.schema_version -ne 1 -or
        $validationItems.Count -ne 8 -or
        ($validationKeys -join ',') -cne ($componentVersionKeys -join ',')) {
        throw "Component bootstrap dependency validation must name every exact ComponentVersion seed once."
    }
    foreach ($version in @($componentBootstrap.components.versions)) {
        $validationItem = @($validationItems | Where-Object validation_key -CEQ $version.resource_key)
        if ($validationItem.Count -ne 1 -or
            ($validationItem[0].reference | ConvertTo-Json -Compress -Depth 20) -cne
            ($version.dataset_reference | ConvertTo-Json -Compress -Depth 20)) {
            throw "Component bootstrap validation '$($version.resource_key)' is not bound to its exact locked Dataset reference."
        }
    }
    $expectedVersionByKey = [ordered]@{
        'sprint-8a-record-table' = $script:Sprint8AFixture.component_versions.table
        'sprint-8a-label-bar' = $script:Sprint8AFixture.component_versions.bar
        'sprint-8a-label-line' = $script:Sprint8AFixture.component_versions.line
        'sprint-8a-label-pie' = $script:Sprint8AFixture.component_versions.pie
        'sprint-8a-label-donut' = $script:Sprint8AFixture.component_versions.donut
        'sprint-8a-row-count' = $script:Sprint8AFixture.component_versions.stat_card
    }
    foreach ($key in $expectedVersionByKey.Keys) {
        $item = @($componentBootstrap.components | Where-Object external_key -CEQ $key)
        $version = @($item.versions | Where-Object resource_key -CEQ $key)
        if ($item.Count -ne 1 -or $version.Count -ne 1 -or
            [string]$version[0].component_version_id -cne [string]$expectedVersionByKey[$key]) {
            throw "Sprint 8A acceptance identity '$key' differs from its owner bootstrap identity."
        }
    }
    $rowCountShell = @($componentBootstrap.components | Where-Object external_key -CEQ 'sprint-8a-row-count')
    $inactiveVersion = @($rowCountShell.versions | Where-Object resource_key -CEQ 'sprint-8a-row-count-inactive')
    $currentVersion = @($rowCountShell.versions | Where-Object resource_key -CEQ 'sprint-8a-row-count')
    if ($rowCountShell.Count -ne 1 -or $inactiveVersion.Count -ne 1 -or $currentVersion.Count -ne 1 -or
        [string]$inactiveVersion[0].component_version_id -cne $script:Sprint8AFixture.inactive_stat_card_version_id -or
        [string]$inactiveVersion[0].status -cne 'superseded' -or
        [string]$inactiveVersion[0].lifecycle_state -cne 'inactive' -or
        [uint64]$inactiveVersion[0].resource_revision -ne 3 -or
        [string]$inactiveVersion[0].successor_version_id -cne [string]$currentVersion[0].component_version_id -or
        [string]$currentVersion[0].component_version_id -cne $script:Sprint8AFixture.component_versions.stat_card -or
        [string]$currentVersion[0].status -cne 'published' -or
        [string]$currentVersion[0].lifecycle_state -cne 'active') {
        throw "Sprint 8A Component lifecycle seed must retain the exact inactive predecessor and current successor identities."
    }
    $dashboardBootstrap = @($blueprint.modules | Where-Object definition_id -CEQ 'tessara.dashboards')[0].bootstrap.value
    $expectedPlacementIds = @($script:Sprint8AFixture.dashboard_placements.Keys | Sort-Object)
    if (@($dashboardBootstrap.placements).Count -ne 7 -or
        (@($dashboardBootstrap.placements.placement_id | Sort-Object) -join ',') -cne
        ($expectedPlacementIds -join ',') -or
        @($dashboardBootstrap.placements.placement_key | Sort-Object -Unique).Count -ne 7) {
        throw "Sprint 8A Dashboard bootstrap must own the exact seven-placement acceptance inventory."
    }
    if (@($dashboardBootstrap.placements | Where-Object { $null -ne $_.component_reference }).Count -ne 0) {
        throw "Sprint 8A Dashboard source input must not hardcode Component ModuleInstance references."
    }
    $dashboardModuleBootstrap = @($blueprint.modules | Where-Object definition_id -CEQ 'tessara.dashboards')[0].bootstrap
    $expectedReceiptBindings = [ordered]@{
        "/placements/0/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[0].placement_id].resource_key
        "/placements/1/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[1].placement_id].resource_key
        "/placements/2/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[2].placement_id].resource_key
        "/placements/3/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[3].placement_id].resource_key
        "/placements/4/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[4].placement_id].resource_key
        "/placements/5/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[5].placement_id].resource_key
        "/placements/6/component_reference" = [string]$script:Sprint8AFixture.dashboard_placements[[string]$dashboardBootstrap.placements[6].placement_id].resource_key
    }
    if (@($dashboardModuleBootstrap.receipt_bindings).Count -ne $expectedReceiptBindings.Count) {
        throw "Sprint 8A Dashboard bootstrap must declare exactly seven Component receipt bindings."
    }
    foreach ($targetPointer in $expectedReceiptBindings.Keys) {
        $placementIndex = [int]([regex]::Match($targetPointer, '^/placements/(?<index>\d+)/component_reference$').Groups['index'].Value)
        $placementInput = $dashboardBootstrap.placements[$placementIndex]
        $expectedPlacement = $script:Sprint8AFixture.dashboard_placements[[string]$placementInput.placement_id]
        $binding = @($dashboardModuleBootstrap.receipt_bindings | Where-Object target_pointer -CEQ $targetPointer)
        if ($binding.Count -ne 1 -or
            [string]$placementInput.placement_key -cne [string]$expectedPlacement.placement_key -or
            [int]$placementInput.row + 1 -ne [int]$expectedPlacement.grid_row -or
            [int]$placementInput.column + 1 -ne [int]$expectedPlacement.grid_column -or
            [int]$placementInput.width -ne [int]$expectedPlacement.grid_width -or
            [int]$placementInput.height -ne [int]$expectedPlacement.grid_height -or
            [string]$binding[0].source_owner -cne "tessara.components" -or
            [string]$binding[0].resource_key -cne [string]$expectedReceiptBindings[$targetPointer] -or
            [string]$binding[0].value_encoding -cne "json") {
            throw "Sprint 8A Dashboard receipt binding '$targetPointer' differs from the exact owner read-back contract."
        }
        $ownerVersions = @($componentBootstrap.components | ForEach-Object { @($_.versions) } | Where-Object {
            [string]$_.resource_key -ceq [string]$expectedPlacement.resource_key
        })
        if ($ownerVersions.Count -ne 1 -or
            [string]$ownerVersions[0].component_version_id -cne [string]$expectedPlacement.component_version_id) {
            throw "Sprint 8A Dashboard placement '$($placementInput.placement_id)' is not bound to its exact owner-produced ComponentVersion."
        }
    }
    $expectedNavigation = [ordered]@{
        "tessara.reference.scoped-records.navigation" = 7
        "tessara.components.navigation" = 8
        "tessara.dashboards.navigation" = 9
        "core.admin.composition" = 4
    }
    foreach ($destinationId in $expectedNavigation.Keys) {
        $placement = @($blueprint.navigation | Where-Object destination_id -CEQ $destinationId)
        if ($placement.Count -ne 1 -or [int]$placement[0].order -ne $expectedNavigation[$destinationId]) {
            throw "Sprint 8A navigation placement '$destinationId' does not match the recalculated canonical order."
        }
    }

    $expectedUatRequirements = [ordered]@{
        "01" = "Sprint 8A AC-01, AC-07, AC-15, and AC-19"
        "02" = "Sprint 8A AC-03, AC-04, AC-05, and AC-16"
        "03" = "Sprint 8A AC-08"
        "04" = "Sprint 8A AC-09, AC-10, AC-18, and AC-19"
        "05" = "Sprint 8A AC-11 and AC-18"
        "06" = "Sprint 8A AC-01, AC-02, AC-06, AC-12, AC-16, AC-18, and AC-19"
        "07" = "Sprint 8A AC-13"
        "08" = "Sprint 8A AC-14"
    }
    foreach ($scenario in 1..8) {
        $scriptPath = Join-Path $repoRoot ("docs/sprints/sprint-8a-uat/uat-8a-{0:d2}.md" -f $scenario)
        if (-not (Test-Path -LiteralPath $scriptPath)) { throw "Missing Sprint 8A UAT script '$scriptPath'." }
        $scenarioKey = "{0:d2}" -f $scenario
        $scriptText = Get-Content -LiteralPath $scriptPath -Raw
        if (-not $scriptText.Contains("- Requirement: $($expectedUatRequirements[$scenarioKey])")) {
            throw "Sprint 8A UAT-$scenarioKey requirement mapping is stale."
        }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $repoRoot "scripts/uat-sprint-8a.ps1"))) {
        throw "Missing Sprint 8A automated UAT diagnostic runner."
    }
}

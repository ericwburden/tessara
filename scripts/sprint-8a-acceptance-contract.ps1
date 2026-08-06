Set-StrictMode -Version Latest

$script:Sprint8AFixture = [ordered]@{
    installation_id = "01980000-0000-7000-8000-00000000008a"
    component_module_instance_id = "142a1ece-f74b-85f6-8ca0-92f4a02e9409"
    component_resource_type = "tessara.components.component_version"
    dataset_id = "01980000-0002-7000-8000-000000000003"
    dashboard_id = "01980000-0003-7000-8000-000000000001"
    table_placement_id = "01980000-0003-7000-8000-000000000003"
    stat_placement_id = "01980000-0003-7000-8000-000000000002"
    component_versions = [ordered]@{
        table = "01980000-0001-7000-8000-000000000010"
        bar = "01980000-0001-7000-8000-000000000011"
        line = "01980000-0001-7000-8000-000000000012"
        pie = "01980000-0001-7000-8000-000000000013"
        donut = "01980000-0001-7000-8000-000000000014"
        stat_card = "01980000-0001-7000-8000-000000000015"
    }
}

function Test-Sprint8AAcceptanceContract {
    $repoRoot = Split-Path -Parent $PSScriptRoot
    foreach ($name in @(
        "installation_id",
        "component_module_instance_id",
        "dataset_id",
        "dashboard_id",
        "table_placement_id",
        "stat_placement_id"
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
        if ([string]$artifactImages.$moduleId -cne $moduleImages[$moduleId]) {
            throw "Sprint 8A Supervisor image reference for '$moduleId' differs from normalized Compose."
        }
    }
    $componentHealthcheck = @($configuration.services.components.healthcheck.test)
    if (($componentHealthcheck -join " ") -cne "CMD curl -fsS http://127.0.0.1:8092/health/ready") {
        throw "Sprint 8A Component runtime must declare the exact readiness healthcheck used by upgrade/rollback."
    }

    $dashboardManifest = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-dashboard-module/manifest.json") -Raw | ConvertFrom-Json
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
    if (-not $dashboardWasmText.Contains("dataset_reference") -or
        $dashboardWasmText.Contains("dataset_version_major")) {
        throw "Dashboard embedded WASM does not implement the canonical Components V3 Dataset reference wire contract."
    }

    foreach ($runner in @("scripts/capture-sprint-6a-deployment-evidence.ps1", "scripts/validate-e2e.ps1", "scripts/smoke.ps1")) {
        $runnerText = Get-Content -LiteralPath (Join-Path $repoRoot $runner) -Raw
        if ($runnerText -notmatch 'TransitionCatalogProfile') {
            throw "Sprint 8A deployment runner '$runner' cannot bind the exact transition-catalog profile."
        }
    }
    $materializationText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/materialize-sprint-8a.ps1") -Raw
    if ($materializationText -match '(?m)^\s*-SkipLegacySeed(?:\s|`|$)') {
        throw "Sprint 8A reference materialization must rebuild the established acceptance fixtures."
    }
    $semanticFixtureText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/prepare-sprint-7a-uat-fixtures.ps1") -Raw
    foreach ($requiredFragment in @(
        'tessara_module_components',
        'SplitComponentOwnership',
        "kind='module_instance'",
        "resource_type='tessara.components.component_version'",
        'generate_series(1,26)',
        'materialized_row_count=30'
    )) {
        if (-not $semanticFixtureText.Contains($requiredFragment)) {
            throw "Sprint 8A semantic fixtures do not preserve the extracted Component ownership contract ('$requiredFragment')."
        }
    }
    $dashboardAcceptanceText = Get-Content -LiteralPath (Join-Path $repoRoot "end2end/tests/dashboards.spec.ts") -Raw
    if (-not $dashboardAcceptanceText.Contains('option.component_slug === "sprint-8a-record-table"')) {
        throw "Sprint 8A Dashboard paging acceptance is not bound to the exact module-owned record Table."
    }
    $baselineDockerfile = Get-Content -LiteralPath (Join-Path $repoRoot "deploy/sprint-8a/Dockerfile.component-rehearsal-baseline") -Raw
    if ($baselineDockerfile -notmatch '(?m)^ARG COMPONENT_BASE_IMAGE=[^\r\n]+$') {
        throw "Sprint 8A Component rehearsal baseline must declare a valid default base image without Docker warnings."
    }
    $inventoryAudit = Join-Path $repoRoot "scripts/audit-sprint-8a-deployed-inventory.ps1"
    if (-not (Test-Path -LiteralPath $inventoryAudit -PathType Leaf)) {
        throw "Sprint 8A exact deployed inventory/navigation audit runner is missing."
    }
    $rehearsalRunners = [ordered]@{
        "scripts/run-sprint-8a-deployed-smoke.ps1" = @(
            "ApiContainerId", "GatewayContainerId", "DatabaseContainerId",
            "ExpectedDataState fresh", "TransitionCatalogProfile sprint-8a",
            "DeploymentEvidencePath"
        )
        "scripts/run-sprint-8a-component-upgrade.ps1" = @(
            "build-sprint-8a-component-rehearsal-baseline.ps1", "OutputTag",
            "verify-sprint-8a-component-upgrade.ps1", "CandidateImage", "CurrentImage",
            "target/sprint-8a-upgrade/component-upgrade-rollback.json",
            "Publish-Sprint7AEvidence"
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
    $deployedSmokeRunner = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/run-sprint-8a-deployed-smoke.ps1") -Raw
    if ($deployedSmokeRunner.Contains("AcceptanceEvidencePath") -or
        $deployedSmokeRunner.Contains("OverwriteAcceptanceEvidence")) {
        throw "Sprint 8A rehearsal smoke must not publish the Sprint 6A authoritative acceptance-evidence schema."
    }

    $blueprint = Get-Content -LiteralPath (Join-Path $repoRoot "deploy/sprint-8a/blueprints/reference.json") -Raw | ConvertFrom-Json
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

    foreach ($scenario in 1..8) {
        $scriptPath = Join-Path $repoRoot ("docs/sprints/sprint-8a-uat/uat-8a-{0:d2}.md" -f $scenario)
        if (-not (Test-Path -LiteralPath $scriptPath)) { throw "Missing Sprint 8A UAT script '$scriptPath'." }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $repoRoot "scripts/uat-sprint-8a.ps1"))) {
        throw "Missing Sprint 8A automated UAT diagnostic runner."
    }
}

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

    foreach ($runner in @("scripts/capture-sprint-6a-deployment-evidence.ps1", "scripts/validate-e2e.ps1", "scripts/smoke.ps1")) {
        $runnerText = Get-Content -LiteralPath (Join-Path $repoRoot $runner) -Raw
        if ($runnerText -notmatch 'TransitionCatalogProfile') {
            throw "Sprint 8A deployment runner '$runner' cannot bind the exact transition-catalog profile."
        }
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

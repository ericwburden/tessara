[CmdletBinding()]
param(
    [switch]$SelfTest,
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

function Read-TrackedJson {
    param([Parameter(Mandatory)][string]$RelativePath)
    $path = Join-Path $repoRoot $RelativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Tracked Sprint 8B contract '$RelativePath' is missing."
    }
    try {
        Get-Content -Raw -LiteralPath $path | ConvertFrom-Json -Depth 100
    } catch {
        throw "Tracked Sprint 8B contract '$RelativePath' is invalid JSON: $($_.Exception.Message)"
    }
}

function Assert-ExactSequence {
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

function Assert-UniqueKeys {
    param(
        [Parameter(Mandatory)][object[]]$Items,
        [Parameter(Mandatory)][string]$Label
    )
    $keys = @($Items | ForEach-Object { [string]$_.key })
    if (@($keys | Sort-Object -Unique).Count -ne $keys.Count) {
        throw "$Label contains duplicate logical keys."
    }
    $keys
}

function Assert-ReferenceFixture {
    param([Parameter(Mandatory)][object]$Fixture)
    if ([int]$Fixture.schema_version -ne 1 -or
        [string]$Fixture.contract -cne "tessara.sprint-8b.reference-fixture" -or
        [string]$Fixture.mutation_policy -cne "owner-apis-and-owner-bootstrap-only" -or
        [string]$Fixture.identity_policy -cne "logical-keys-resolve-only-from-signed-owner-receipts-and-typed-read-back") {
        throw "Sprint 8B reference fixture identity or ownership policy is invalid."
    }

    $canonicalJson = $Fixture | ConvertTo-Json -Depth 100 -Compress
    if ($canonicalJson -match '(?i)[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}') {
        throw "Sprint 8B reference fixture predicts a physical UUID instead of using owner read-back."
    }

    Assert-ExactSequence -Expected @(
        "actor.admin", "actor.dataset-manager", "actor.operations", "actor.full",
        "actor.restricted", "actor.confidential", "actor.disjoint", "actor.delegate"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.actors) -Label "reference actors") -Label "reference actors"
    Assert-ExactSequence -Expected @(
        "form.primary/v1", "form.secondary/v1", "form.disjoint/v1"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.form_versions) -Label "reference FormVersions") -Label "reference FormVersions"
    Assert-ExactSequence -Expected @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.new",
        "response.corrected", "response.status-out", "response.status-in", "response.redacted",
        "response.deleted", "response.outside-scope"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.response_changes) -Label "reference Responses") -Label "reference Responses"
    Assert-ExactSequence -Expected @("binding.primary", "binding.independent") `
        -Actual @(Assert-UniqueKeys -Items @($Fixture.source_bindings) -Label "reference bindings") `
        -Label "reference bindings"
    if ([string]$Fixture.source_bindings[0].cursor_partition -ceq [string]$Fixture.source_bindings[1].cursor_partition) {
        throw "Sprint 8B reference bindings must use independent cursor partitions."
    }
    Assert-ExactSequence -Expected @(
        "dataset.base", "dataset.derived", "dataset.derived-second-hop",
        "dataset.independent-binding", "dataset.disjoint-binding", "dataset.incompatible",
        "dataset.cycle-candidate"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.datasets) -Label "reference Datasets") -Label "reference Datasets"
    Assert-ExactSequence -Expected @(
        "component.dataset-table", "component.dataset-chart", "component.dataset-stat",
        "component.dataset-disjoint", "component.dataset-incompatible"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.downstream.components) -Label "reference Components") -Label "reference Components"
    if ([string]$Fixture.downstream.dashboard.key -cne "dashboard.dataset-components" -or
        [string]$Fixture.downstream.dashboard.release -cne "3.0.2" -or
        [string]$Fixture.downstream.dashboard.viewer_actor -cne "actor.full" -or
        @($Fixture.downstream.components | Where-Object { [string]$_.release -cne "1.1.0" }).Count -ne 0) {
        throw "Sprint 8B downstream fixtures do not pin Component 1.1.0 and Dashboard 3.0.2 exactly."
    }
    Assert-ExactSequence -Expected @(
        "component.dataset-table", "component.dataset-chart", "component.dataset-stat",
        "component.dataset-disjoint"
    ) -Actual @($Fixture.downstream.dashboard.components) -Label "reference Dashboard Components"
    $expectedPlacements = @(
        [pscustomobject]@{ key = "dataset-stat"; component = "component.dataset-stat"; column = 0; row = 0; width = 4; height = 2; expected = "available" },
        [pscustomobject]@{ key = "dataset-table"; component = "component.dataset-table"; column = 0; row = 2; width = 12; height = 6; expected = "available" },
        [pscustomobject]@{ key = "dataset-chart"; component = "component.dataset-chart"; column = 0; row = 8; width = 8; height = 4; expected = "available" },
        [pscustomobject]@{ key = "dataset-disjoint"; component = "component.dataset-disjoint"; column = 8; row = 8; width = 4; height = 4; expected = "redacted" }
    )
    if ((@($Fixture.downstream.dashboard.placements) | ConvertTo-Json -Depth 10 -Compress) -cne
        ($expectedPlacements | ConvertTo-Json -Depth 10 -Compress) -or
        [string]$Fixture.downstream.dashboard.expected -cne
            "healthy-outage-recovery-and-scoped-redaction") {
        throw "Sprint 8B Dashboard fixture does not declare the exact non-overlapping scoped-redaction layout."
    }
    Assert-ExactSequence -Expected @(
        "forms.source-usage", "operations.dataset-readiness", "app-summary.dataset-counts"
    ) -Actual @($Fixture.reverse_consumers) -Label "reverse-consumer fixtures"
    Assert-ExactSequence -Expected @(
        "tessara.datasets.dataset", "tessara.datasets.dataset_revision", "tessara.datasets.dataset_major_line"
    ) -Actual @($Fixture.resource_observation_types) -Label "Dataset resource-observation fixtures"
    Assert-ExactSequence -Expected @(
        "predicted-uuid", "foreign-owner-database-write", "response-sql-mutation", "copied-count-as-identity"
    ) -Actual @($Fixture.forbidden) -Label "reference fixture forbidden operations"
}

function Assert-ReferenceActorBlueprintAlignment {
    param(
        [Parameter(Mandatory)][object]$Fixture,
        [Parameter(Mandatory)][object]$Blueprint
    )

    $fixtureActors = @($Fixture.actors | Where-Object { [string]$_.key -cne "actor.admin" })
    $blueprintActors = @($Blueprint.core.bootstrap.value.actors)
    if ((@($fixtureActors.key | Sort-Object) -join "`n") -cne
        (@($blueprintActors.resource_key | Sort-Object) -join "`n")) {
        throw "Reference Blueprint actors are not set-equal to the non-admin fixture actors."
    }
    foreach ($fixtureActor in $fixtureActors) {
        $actorKey = [string]$fixtureActor.key
        $blueprintActor = @($blueprintActors | Where-Object {
            [string]$_.resource_key -ceq $actorKey
        })
        if ($blueprintActor.Count -ne 1) {
            throw "Reference Blueprint actor '$actorKey' is missing or duplicated."
        }
        $fixtureCapabilities = @($fixtureActor.capabilities | ForEach-Object { [string]$_ })
        $blueprintCapabilities = @($blueprintActor[0].capabilities | ForEach-Object { [string]$_ })
        if (@($fixtureCapabilities | Sort-Object -Unique).Count -ne $fixtureCapabilities.Count -or
            @($blueprintCapabilities | Sort-Object -Unique).Count -ne $blueprintCapabilities.Count -or
            ((@($fixtureCapabilities | Sort-Object) -join "`n") -cne
                (@($blueprintCapabilities | Sort-Object) -join "`n")) -or
            @($blueprintActor[0].scope_node_keys).Count -ne 1 -or
            [string]$blueprintActor[0].scope_node_keys[0] -cne [string]$fixtureActor.scope) {
            throw "Reference actor '$actorKey' capability/scope tuple differs between fixture and Blueprint."
        }
    }
}

function Assert-ReferenceDownstreamBlueprintAlignment {
    param(
        [Parameter(Mandatory)][object]$Fixture,
        [Parameter(Mandatory)][object]$Blueprint
    )

    $datasetModule = @($Blueprint.modules | Where-Object {
        [string]$_.definition_id -ceq "tessara.datasets"
    })
    $componentModule = @($Blueprint.modules | Where-Object {
        [string]$_.definition_id -ceq "tessara.components"
    })
    $dashboardModule = @($Blueprint.modules | Where-Object {
        [string]$_.definition_id -ceq "tessara.dashboards"
    })
    if ($datasetModule.Count -ne 1 -or $componentModule.Count -ne 1 -or
        $dashboardModule.Count -ne 1) {
        throw "Reference Blueprint must select Dataset, Component, and Dashboard exactly once."
    }

    $datasetKeys = @($datasetModule[0].bootstrap.value.datasets | ForEach-Object {
        [string]$_.resource_key
    })
    Assert-ExactSequence -Expected @(
        "dataset.base", "dataset.derived", "dataset.derived-second-hop",
        "dataset.independent-binding", "dataset.disjoint-binding"
    ) -Actual $datasetKeys -Label "Reference Blueprint accepted Datasets"
    $datasetBindings = @($datasetModule[0].bootstrap.receipt_bindings)
    foreach ($binding in @(
        @("/datasets/4/definition/visibility_node_ids/0", "scope.disjoint", "string"),
        @("/datasets/4/definition/initial_source", "form.disjoint/v1.dataset_source", "json")
    )) {
        $matches = @($datasetBindings | Where-Object {
            [string]$_.target_pointer -ceq $binding[0] -and
            [string]$_.source_owner -ceq "core" -and
            [string]$_.resource_key -ceq $binding[1] -and
            [string]$_.value_encoding -ceq $binding[2]
        })
        if ($matches.Count -ne 1) {
            throw "Reference Blueprint lacks exact disjoint Dataset receipt binding '$($binding[0])'."
        }
    }

    $componentBindings = @($componentModule[0].bootstrap.receipt_bindings)
    foreach ($binding in @(
        @("/dependency_validation/items/3/reference", "tessara.datasets", "dataset.disjoint-binding", "json"),
        @("/components/3/versions/0/dataset_reference", "tessara.datasets", "dataset.disjoint-binding", "json"),
        @("/components/3/versions/0/dataset_scope_node_ids/0", "core", "scope.disjoint", "string")
    )) {
        $matches = @($componentBindings | Where-Object {
            [string]$_.target_pointer -ceq $binding[0] -and
            [string]$_.source_owner -ceq $binding[1] -and
            [string]$_.resource_key -ceq $binding[2] -and
            [string]$_.value_encoding -ceq $binding[3]
        })
        if ($matches.Count -ne 1) {
            throw "Reference Blueprint lacks exact disjoint Component receipt binding '$($binding[0])'."
        }
    }

    $fixturePlacements = @($Fixture.downstream.dashboard.placements)
    $blueprintPlacements = @($dashboardModule[0].bootstrap.value.placements)
    if ($fixturePlacements.Count -ne $blueprintPlacements.Count) {
        throw "Reference Dashboard placement count differs between fixture and Blueprint."
    }
    $dashboardBindings = @($dashboardModule[0].bootstrap.receipt_bindings)
    for ($index = 0; $index -lt $fixturePlacements.Count; $index++) {
        $fixturePlacement = $fixturePlacements[$index]
        $blueprintPlacement = $blueprintPlacements[$index]
        foreach ($property in @("key", "column", "row", "width", "height")) {
            $blueprintProperty = if ($property -ceq "key") { "placement_key" } else { $property }
            if ([string]$fixturePlacement.$property -cne [string]$blueprintPlacement.$blueprintProperty) {
                throw "Reference Dashboard placement '$index' differs from its logical fixture geometry."
            }
        }
        if ($null -ne $blueprintPlacement.component_reference) {
            throw "Reference Dashboard placement '$index' hardcodes a Component identity."
        }
        $targetPointer = "/placements/$index/component_reference"
        $matches = @($dashboardBindings | Where-Object {
            [string]$_.target_pointer -ceq $targetPointer -and
            [string]$_.source_owner -ceq "tessara.components" -and
            [string]$_.resource_key -ceq [string]$fixturePlacement.component -and
            [string]$_.value_encoding -ceq "json"
        })
        if ($matches.Count -ne 1) {
            throw "Reference Dashboard placement '$index' lacks its exact Component owner receipt binding."
        }
    }
    if ($dashboardBindings.Count -ne ($fixturePlacements.Count + 1) -or
        [string]$dashboardModule[0].bootstrap.value.scope_node_id -ne "" -or
        @($dashboardBindings | Where-Object {
            [string]$_.target_pointer -ceq "/scope_node_id" -and
            [string]$_.source_owner -ceq "core" -and
            [string]$_.resource_key -ceq "scope.full" -and
            [string]$_.value_encoding -ceq "string"
        }).Count -ne 1) {
        throw "Reference Dashboard does not bind its exact full-scope identity plus four Component placements."
    }
}

function Assert-ProviderFaultFixture {
    param([Parameter(Mandatory)][object]$Fixture)
    if ([int]$Fixture.schema_version -ne 1 -or
        [string]$Fixture.contract -cne "tessara.sprint-8b.provider-faults" -or
        [string]$Fixture.isolation -cne "one-binding-per-proxy" -or
        [bool]$Fixture.product_bypass_forbidden -ne $true -or
        [bool]$Fixture.database_bypass_forbidden -ne $true) {
        throw "Sprint 8B provider-fault contract identity or isolation policy is invalid."
    }
    Assert-ExactSequence -Expected @(
        "tessara.datasets.response-export", "tessara.datasets.form-version-schema",
        "tessara.datasets.scope-catalog", "tessara.datasets.principal-display-catalog"
    ) -Actual @($Fixture.runtime_configuration.bindings.binding) -Label "provider fault bindings"
    if (@($Fixture.runtime_configuration.bindings.service | Sort-Object -Unique).Count -ne 4 -or
        @($Fixture.runtime_configuration.bindings.endpoint | Sort-Object -Unique).Count -ne 4) {
        throw "Every Sprint 8B provider binding must use a separately addressable proxy."
    }
    Assert-ExactSequence -Expected @(
        "response.timeout", "response.malformed", "response.incompatible", "response.expired-cursor",
        "response.epoch-change", "response.scope-substitution", "forms.unavailable", "scope.unavailable",
        "principal.unavailable", "dataset.derived-rebuild"
    ) -Actual @($Fixture.faults.key) -Label "provider fault cases"
}

function Assert-UpgradeFixture {
    param([Parameter(Mandatory)][object]$Fixture)
    if ([int]$Fixture.schema_version -ne 1 -or
        [string]$Fixture.contract -cne "tessara.sprint-8b.dataset-upgrade-fixture") {
        throw "Sprint 8B upgrade fixture identity is invalid."
    }
    Assert-ExactSequence -Expected @("0.9.0", "1.0.0", "0.9.0", "1.0.0") `
        -Actual @($Fixture.sequence) -Label "Dataset upgrade sequence"
    foreach ($release in @("0.9.0", "1.0.0")) {
        $declaration = $Fixture.dataset_releases.PSObject.Properties[$release].Value
        if ([string]$declaration.contract -cne "tessara.datasets.dataset-major-line" -or
            [string]$declaration.contract_version -cne "2.0.0" -or
            [bool]$declaration.source_built -ne $true) {
            throw "Dataset release '$release' is not a real source-built Dataset v2 fixture."
        }
    }
    if ([string]$Fixture.fixed_dependencies.'tessara.components' -cne "1.1.0" -or
        [string]$Fixture.fixed_dependencies.'tessara.dashboards' -cne "3.0.2") {
        throw "Upgrade fixture must keep Component 1.1.0 and Dashboard 3.0.2 fixed."
    }
    foreach ($required in @("dataset_state", "provider_route", "typed_resource_identity", "navigation_identity")) {
        if (@($Fixture.preserved) -cnotcontains $required) { throw "Upgrade fixture omits preserved identity '$required'." }
    }
    foreach ($required in @("images", "containers", "restart_counts", "owner_data", "availability")) {
        if (@($Fixture.unrelated_unchanged) -cnotcontains $required) { throw "Upgrade fixture omits unrelated invariant '$required'." }
    }
}

function Assert-UatScenarioContract {
    param([Parameter(Mandatory)][object]$Contract)
    if ([int]$Contract.schema_version -ne 1 -or [string]$Contract.contract -cne "tessara.sprint-8b.uat-scenarios") {
        throw "Sprint 8B UAT scenario contract identity is invalid."
    }
    Assert-ExactSequence -Expected @(1..11 | ForEach-Object { "UAT-8B-{0:d2}" -f $_ }) `
        -Actual @($Contract.scenarios.id) -Label "UAT scenario IDs"
    Assert-ExactSequence -Expected @(
        "Product parity and UI", "Fresh materialization", "Configuration and diagnostics",
        "Provider contracts, cursor, scope, and Dataset DAG", "Cross-module exit and outage",
        "Core subtraction and isolation", "Failure retry and recovery", "Independent upgrade and rollback",
        "Editor provider boundaries", "Reverse consumers and Operations", "Resource, replay, and routing"
    ) -Actual @($Contract.scenarios.name) -Label "UAT scenario names"
    foreach ($scenario in @($Contract.scenarios)) {
        if (@($scenario.assertions).Count -eq 0 -or [string]::IsNullOrWhiteSpace([string]$scenario.cleanup)) {
            throw "UAT scenario '$($scenario.id)' lacks assertions or cleanup."
        }
    }
}

function Assert-UiBaseline {
    param([Parameter(Mandatory)][object]$Baseline)
    if ([int]$Baseline.schema_version -ne 1 -or
        [string]$Baseline.contract -cne "tessara.sprint-8b.dataset-ui-baseline" -or
        [string]$Baseline.capture_state -cne "captured-current-main" -or
        [string]$Baseline.source_commit -cnotmatch '^[0-9a-f]{40}$') {
        throw "Sprint 8B UI baseline identity is invalid."
    }
    Assert-ExactSequence -Expected @(
        "dataset-directory", "dataset-create", "dataset-detail", "dataset-preview", "dataset-edit",
        "dataset-revisions", "dataset-revision-detail", "dataset-revision-edit",
        "operations-dataset-readiness", "module-management-dataset", "form-dataset-sources",
        "dataset-directory-javascript-disabled"
    ) -Actual @($Baseline.cases.key) -Label "UI baseline cases"
    foreach ($case in @($Baseline.cases)) {
        if (@($case.console.errors).Count -ne 0) { throw "UI baseline '$($case.key)' contains console errors." }
        $screenshotPath = Join-Path $repoRoot "docs/audits/sprint-8b-dataset-ui-baseline/$($case.screenshot)"
        if (-not (Test-Path -LiteralPath $screenshotPath -PathType Leaf)) {
            throw "UI baseline screenshot '$($case.screenshot)' is missing."
        }
        $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $screenshotPath).Hash.ToLowerInvariant()
        if ($actualHash -cne [string]$case.screenshot_sha256) {
            throw "UI baseline screenshot '$($case.screenshot)' does not match its recorded SHA-256."
        }
        foreach ($external in @($case.external_requests)) {
            if ([string]$external -cne "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.2/css/all.min.css") {
                throw "UI baseline '$($case.key)' contains an unclassified external request."
            }
        }
    }
    foreach ($value in @("reader", "manager", "restricted", "administrator")) {
        if (@($Baseline.required_matrix.roles) -cnotcontains $value) { throw "UI baseline role matrix omits '$value'." }
    }
    foreach ($value in @("light", "dark", "stored_theme", "system_theme")) {
        if (@($Baseline.required_matrix.themes) -cnotcontains $value) { throw "UI baseline theme matrix omits '$value'." }
    }
    foreach ($value in @("javascript_disabled_ssr", "direct_refresh", "hydrated", "no_external_assets")) {
        if (@($Baseline.required_matrix.runtime) -cnotcontains $value) { throw "UI baseline runtime matrix omits '$value'." }
    }
    $coreHead = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web/src/document/assets.rs")
    $datasetHead = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web-datasets/src/document.rs")
    if ($coreHead.Contains("cdnjs.cloudflare.com") -or $datasetHead -match 'https?://') {
        throw "Current Dataset/Core document source still depends on an external browser asset."
    }
}

function Assert-AcceptanceManifest {
    param([Parameter(Mandatory)][object]$Manifest)
    if ([int]$Manifest.schema_version -ne 2 -or @($Manifest.files).Count -eq 0) {
        throw "Sprint 8B browser acceptance manifest identity is invalid."
    }
    $listedTotal = @($Manifest.files | ForEach-Object { @($_.tests).Count } | Measure-Object -Sum).Sum
    if ([int]$Manifest.expected_total -ne [int]$listedTotal) {
        throw "Acceptance manifest expected_total does not equal its literal test inventory."
    }
    $required = @(
        "Sprint 8B independent Dataset module › editor options use only Dataset-owned browser routes",
        "Sprint 8B independent Dataset module › synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure",
        "Sprint 8B independent Dataset module › reverse consumers distinguish authorized empty unavailable and undisclosed states",
        "Sprint 8B independent Dataset module › mutation replay and static route precedence remain exact",
        "canonical module UI visual baselines › Datasets directory at 1440 px (light)",
        "canonical module UI visual baselines › Datasets editor at 390 px (light)",
        "canonical module UI visual baselines › Datasets directory at 1440 px (dark)",
        "canonical module UI visual baselines › Datasets editor at 390 px (dark)",
        "canonical module UI visual baselines › Datasets revisions at 1024 px",
        "canonical module UI visual baselines › Datasets preview at 1440 px",
        "canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas"
    )
    $identities = @($Manifest.files | ForEach-Object {
        $path = [string]$_.path
        foreach ($test in @($_.tests)) { "$path::$test" }
    })
    if (@($identities | Sort-Object -Unique).Count -ne $identities.Count) {
        throw "Acceptance manifest contains duplicate file/test identities."
    }
    $titles = @($Manifest.files.tests)
    foreach ($identity in $required) {
        if ($titles -cnotcontains $identity) { throw "Acceptance manifest omits '$identity'." }
    }
    foreach ($file in @($Manifest.files)) {
        $path = Join-Path $repoRoot "end2end/tests/$($file.path)"
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Acceptance test '$($file.path)' is missing." }
        foreach ($line in Get-Content -LiteralPath $path) {
            $trimmed = $line.TrimStart()
            if (-not $trimmed.StartsWith("//") -and $trimmed -match '\btest\.(skip|fixme|only)\s*\(') {
                throw "Acceptance test '$($file.path)' contains prohibited '$($Matches[1])' selection."
            }
        }
    }
    $demoSeedSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "end2end/tests/support/demo-seed.ts"
    )
    if ($demoSeedSource.Contains('return dataState === "fresh";') -or
        -not $demoSeedSource.Contains("return false;") -or
        -not $demoSeedSource.Contains("TESSARA_PLAYWRIGHT_ACCEPTANCE")) {
        throw "Browser acceptance may not invoke the legacy demo seed endpoint on a materialized topology."
    }
    $componentSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "end2end/tests/components.spec.ts"
    )
    foreach ($retired in @("tessara.transition.dataset_major_line", 'kind: "core_installation"')) {
        if ($componentSource.Contains($retired)) {
            throw "Component browser acceptance still contains retired Dataset reference identity '$retired'."
        }
    }
    foreach ($requiredComponentIdentity in @(
        "tessara.datasets.dataset_major_line", 'kind: "module_instance"',
        "module_instance_id"
    )) {
        if (-not $componentSource.Contains($requiredComponentIdentity)) {
            throw "Component browser acceptance omits current Dataset v2 identity '$requiredComponentIdentity'."
        }
    }
    $moduleSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "end2end/tests/modules.spec.ts"
    )
    foreach ($requiredModuleIdentity in @(
        'const DATASETS_DEFINITION = "tessara.datasets"', 'release: "1.1.0"',
        'dataset_dependency: "2.0.0"', 'version: "2.0.0"',
        'kind: "module_instance"', "module_instance_id: datasetModule.entry.instance.id"
    )) {
        if (-not $moduleSource.Contains($requiredModuleIdentity)) {
            throw "Module browser acceptance omits current Component/Dataset diagnostic identity '$requiredModuleIdentity'."
        }
    }
    foreach ($retiredModuleIdentity in @(
        'release: "1.0.1"', 'dataset_dependency: "1.0.0"',
        'kind: "core_installation"', "tessara.transition.dataset"
    )) {
        if ($moduleSource.Contains($retiredModuleIdentity)) {
            throw "Module browser acceptance still contains retired Component/Dataset identity '$retiredModuleIdentity'."
        }
    }
    $dashboardSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "end2end/tests/dashboards.spec.ts"
    )
    foreach ($requiredDashboardIdentity in @(
        'REFERENCE_DASHBOARD_NAME = "Dataset Components"',
        'REFERENCE_FULL_READER_EMAIL = "full-reader@tessara.local"',
        "expect(dashboard!.placement_count).toBe(4)",
        "const unavailablePlacements = operatorDefinition.placements.filter(",
        "const redactedPlacements = unavailablePlacements.filter(",
        "adminPlacementsForRedacted",
        "expect(viewerHtml).not.toContain(REFERENCE_HIDDEN_COMPONENT_SLUG)"
    )) {
        if (-not $dashboardSource.Contains($requiredDashboardIdentity)) {
            throw "Dashboard browser acceptance omits Reference fixture predicate '$requiredDashboardIdentity'."
        }
    }
    if ($dashboardSource -notmatch '(?s)expect\(\s*redactedPlacements,.*?\)\.toHaveLength\(1\);') {
        throw "Dashboard browser acceptance does not require exactly one opaque redacted placement."
    }
    if ($dashboardSource -notmatch '(?s)expect\(\s*unavailablePlacements\.length,.*?\)\.toBe\(1\);') {
        throw "Dashboard browser acceptance does not require exactly one unavailable placement."
    }
    $changeLog = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8b-test-change-log.md")
    foreach ($requiredLogIdentity in @(
        "end2end/tests/datasets-module.spec.ts",
        "end2end/tests/module-ui-visual.spec.ts",
        "95 exact identities",
        "eleven frozen Sprint 8B scenarios",
        "unchanged browser identities requiring Dataset v2 ModuleInstance references",
        "demo-seed-free fresh/upgraded acceptance",
        "receipt-bound Reference fourth Dashboard placement"
    )) {
        if (-not $changeLog.Contains($requiredLogIdentity)) {
            throw "Sprint 8B test-change log does not account for '$requiredLogIdentity'."
        }
    }
}

$reference = Read-TrackedJson "deploy/sprint-8b/fixtures/reference-fixture-contract.json"
$blueprint = Read-TrackedJson "deploy/sprint-8b/blueprints/reference.json"
$faults = Read-TrackedJson "deploy/sprint-8b/fixtures/provider-fault-contract.json"
$upgrade = Read-TrackedJson "deploy/sprint-8b/fixtures/upgrade-fixture-contract.json"
$scenarios = Read-TrackedJson "docs/sprints/sprint-8b-uat/scenario-contract.json"
$baseline = Read-TrackedJson "docs/audits/sprint-8b-dataset-ui-baseline/baseline-index.json"
$acceptance = Read-TrackedJson "end2end/acceptance-manifest.json"

Assert-ReferenceFixture $reference
Assert-ReferenceActorBlueprintAlignment -Fixture $reference -Blueprint $blueprint
Assert-ReferenceDownstreamBlueprintAlignment -Fixture $reference -Blueprint $blueprint
Assert-ProviderFaultFixture $faults
Assert-UpgradeFixture $upgrade
Assert-UatScenarioContract $scenarios
Assert-UiBaseline $baseline
Assert-AcceptanceManifest $acceptance

if ($SelfTest) {
    $tampered = ($reference | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100)
    $tampered.actors = @($tampered.actors | Select-Object -SkipLast 1)
    $rejected = $false
    try { Assert-ReferenceFixture $tampered } catch { $rejected = $true }
    if (-not $rejected) { throw "Acceptance contract self-test admitted an incomplete actor fixture." }

    $tamperedBlueprint = $blueprint | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $restricted = @($tamperedBlueprint.core.bootstrap.value.actors | Where-Object {
        [string]$_.resource_key -ceq "actor.restricted"
    })[0]
    $restricted.capabilities = @("datasets:read")
    $rejected = $false
    try {
        Assert-ReferenceActorBlueprintAlignment -Fixture $reference -Blueprint $tamperedBlueprint
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Acceptance contract self-test admitted an actor capability/scope mismatch."
    }

    $tamperedDownstreamBlueprint = $blueprint | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedDownstreamBlueprint.modules[2].bootstrap.receipt_bindings[4].resource_key =
        "component.dataset-table"
    $rejected = $false
    try {
        Assert-ReferenceDownstreamBlueprintAlignment -Fixture $reference `
            -Blueprint $tamperedDownstreamBlueprint
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Acceptance contract self-test admitted a substituted redacted Dashboard placement."
    }

    $tamperedManifest = ($acceptance | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100)
    $tamperedManifest.expected_total = [int]$tamperedManifest.expected_total + 1
    $rejected = $false
    try { Assert-AcceptanceManifest $tamperedManifest } catch { $rejected = $true }
    if (-not $rejected) { throw "Acceptance contract self-test admitted a copied test count." }
}

$result = [ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    contract = "tessara.sprint-8b.acceptance-contract-result"
    status = "passed"
    reference_actors = @($reference.actors).Count
    reference_responses = @($reference.response_changes).Count
    reference_datasets = @($reference.datasets).Count
    provider_faults = @($faults.faults).Count
    uat_scenarios = @($scenarios.scenarios).Count
    ui_baseline_cases = @($baseline.cases).Count
    browser_tests = [int]$acceptance.expected_total
    self_test = [bool]$SelfTest
}

if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
    $fullEvidencePath = if ([IO.Path]::IsPathRooted($EvidencePath)) {
        [IO.Path]::GetFullPath($EvidencePath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidencePath))
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $fullEvidencePath)) | Out-Null
    $result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $fullEvidencePath -Encoding utf8NoBOM
}

$result | ConvertTo-Json -Depth 10

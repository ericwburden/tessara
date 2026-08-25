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
        throw "Tracked Sprint 8C contract '$RelativePath' is missing."
    }
    try {
        Get-Content -Raw -LiteralPath $path | ConvertFrom-Json -Depth 100
    } catch {
        throw "Tracked Sprint 8C contract '$RelativePath' is invalid JSON: $($_.Exception.Message)"
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
        [string]$Fixture.contract -cne "tessara.sprint-8c.reference-fixture" -or
        [string]$Fixture.mutation_policy -cne "owner-apis-and-owner-bootstrap-only" -or
        [string]$Fixture.identity_policy -cne "logical-keys-resolve-only-from-signed-owner-receipts-and-typed-read-back") {
        throw "Sprint 8C reference fixture identity or ownership policy is invalid."
    }

    $canonicalJson = $Fixture | ConvertTo-Json -Depth 100 -Compress
    if ($canonicalJson -match '(?i)[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}') {
        throw "Sprint 8C reference fixture predicts a physical UUID instead of using owner read-back."
    }

    Assert-ExactSequence -Expected @(
        "actor.admin", "actor.dataset-manager", "actor.operations", "actor.full",
        "actor.restricted", "actor.confidential", "actor.disjoint", "actor.delegate",
        "actor.response-owner", "actor.response-manager", "actor.response-outsider"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.actors) -Label "reference actors") -Label "reference actors"
    Assert-ExactSequence -Expected @(
        "form.primary/v1", "form.secondary/v1", "form.disjoint/v1"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.form_versions) -Label "reference FormVersions") -Label "reference FormVersions"
    Assert-ExactSequence -Expected @(
        "response.draft.owner", "response.submitted.owner",
        "response.submitted.delegated", "response.submitted.restricted"
    ) -Actual @(Assert-UniqueKeys -Items @($Fixture.response_changes) -Label "reference Responses") -Label "reference Responses"
    Assert-ExactSequence -Expected @("binding.primary", "binding.independent") `
        -Actual @(Assert-UniqueKeys -Items @($Fixture.source_bindings) -Label "reference bindings") `
        -Label "reference bindings"
    if ([string]$Fixture.source_bindings[0].cursor_partition -ceq [string]$Fixture.source_bindings[1].cursor_partition) {
        throw "Sprint 8C reference bindings must use independent cursor partitions."
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
        throw "Sprint 8C downstream fixtures do not pin Component 1.1.0 and Dashboard 3.0.2 exactly."
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
        throw "Sprint 8C Dashboard fixture does not declare the exact non-overlapping scoped-redaction layout."
    }
    Assert-ExactSequence -Expected @(
        "forms.source-usage", "operations.response-status", "app-summary.response-counts"
    ) -Actual @($Fixture.reverse_consumers) -Label "reverse-consumer fixtures"
    Assert-ExactSequence -Expected @(
        "tessara.responses.response", "tessara.datasets.dataset",
        "tessara.datasets.dataset_revision", "tessara.datasets.dataset_major_line"
    ) -Actual @($Fixture.resource_observation_types) -Label "resource-observation fixtures"
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

function Assert-ReferenceFormWorkflowBlueprintAlignment {
    param(
        [Parameter(Mandatory)][object]$Fixture,
        [Parameter(Mandatory)][object]$Blueprint
    )

    $fixtureForms = @($Fixture.form_versions)
    $blueprintForms = @($Blueprint.core.bootstrap.value.forms)
    if ((@($fixtureForms.key | Sort-Object) -join "`n") -cne
        (@($blueprintForms.resource_key | Sort-Object) -join "`n")) {
        throw "Reference Blueprint Forms are not set-equal to the fixture FormVersions."
    }
    foreach ($fixtureForm in $fixtureForms) {
        $formKey = [string]$fixtureForm.key
        if ($fixtureForm.PSObject.Properties.Name -contains "scope") {
            throw "Reference FormVersion '$formKey' retains the retired singular scope shape."
        }
        $blueprintForm = @($blueprintForms | Where-Object {
            [string]$_.resource_key -ceq $formKey
        })
        if ($blueprintForm.Count -ne 1) {
            throw "Reference Blueprint Form '$formKey' is missing or duplicated."
        }
        $fixtureScopes = @($fixtureForm.scopes | ForEach-Object { [string]$_ })
        $blueprintScopes = @($blueprintForm[0].scope_node_keys | ForEach-Object { [string]$_ })
        if ($fixtureScopes.Count -eq 0 -or
            @($fixtureScopes | Sort-Object -Unique).Count -ne $fixtureScopes.Count -or
            @($blueprintScopes | Sort-Object -Unique).Count -ne $blueprintScopes.Count -or
            (($fixtureScopes -join "`n") -cne ($blueprintScopes -join "`n"))) {
            throw "Reference Form '$formKey' source scopes differ between fixture and Blueprint."
        }
    }

    foreach ($assignment in @($Blueprint.core.bootstrap.value.workflow_assignments)) {
        $assignmentKey = [string]$assignment.resource_key
        $formKey = [string]$assignment.form_resource_key
        $nodeKey = [string]$assignment.node_key
        $referencedForm = @($blueprintForms | Where-Object {
            [string]$_.resource_key -ceq $formKey
        })
        if ($referencedForm.Count -ne 1 -or
            -not (@($referencedForm[0].scope_node_keys | ForEach-Object { [string]$_ }) -ccontains $nodeKey)) {
            throw "Reference workflow assignment '$assignmentKey' node '$nodeKey' is not a canonical source scope of Form '$formKey'."
        }
    }
}

function Assert-ReferenceFormDatasetBlueprintAlignment {
    param(
        [Parameter(Mandatory)][object]$Fixture,
        [Parameter(Mandatory)][object]$Blueprint
    )

    $fixtureForms = @($Fixture.form_versions)
    foreach ($binding in @($Fixture.source_bindings)) {
        $bindingKey = [string]$binding.key
        if ($binding.PSObject.Properties.Name -contains "scope") {
            throw "Reference source binding '$bindingKey' retains the retired singular scope shape."
        }
        $formKey = [string]$binding.form_version
        $fixtureForm = @($fixtureForms | Where-Object { [string]$_.key -ceq $formKey })
        if ($fixtureForm.Count -ne 1) {
            throw "Reference source binding '$bindingKey' uses missing or duplicated Form '$formKey'."
        }
        $bindingScopes = @($binding.scopes | ForEach-Object { [string]$_ })
        $formScopes = @($fixtureForm[0].scopes | ForEach-Object { [string]$_ })
        if ($bindingScopes.Count -eq 0 -or
            @($bindingScopes | Sort-Object -Unique).Count -ne $bindingScopes.Count -or
            (($bindingScopes -join "`n") -cne ($formScopes -join "`n"))) {
            throw "Reference source binding '$bindingKey' scopes differ from Form '$formKey'."
        }
    }

    $datasetModules = @($Blueprint.modules | Where-Object {
        [string]$_.definition_id -ceq "tessara.datasets"
    })
    if ($datasetModules.Count -ne 1) {
        throw "Reference Blueprint must select Dataset exactly once."
    }
    $datasets = @($datasetModules[0].bootstrap.value.datasets)
    $receiptBindings = @($datasetModules[0].bootstrap.receipt_bindings)
    $formSourceBindings = @($receiptBindings | Where-Object {
        [string]$_.source_owner -ceq "core" -and
        [string]$_.resource_key -clike "form.*/v*.dataset_source"
    })
    foreach ($sourceBinding in $formSourceBindings) {
        $targetPointer = [string]$sourceBinding.target_pointer
        if ($targetPointer -cnotmatch '^/datasets/([0-9]+)/definition/initial_source$') {
            throw "Reference Form-backed Dataset binding has invalid target '$targetPointer'."
        }
        $datasetIndex = [int]$Matches[1]
        if ($datasetIndex -ge $datasets.Count) {
            throw "Reference Form-backed Dataset binding targets missing Dataset index $datasetIndex."
        }
        $formKey = ([string]$sourceBinding.resource_key) -replace '\.dataset_source$', ''
        $fixtureForm = @($fixtureForms | Where-Object { [string]$_.key -ceq $formKey })
        if ($fixtureForm.Count -ne 1) {
            throw "Reference Form-backed Dataset uses unknown Form '$formKey'."
        }
        $formScopes = @($fixtureForm[0].scopes | ForEach-Object { [string]$_ })
        $visibilityPrefix = "/datasets/$datasetIndex/definition/visibility_node_ids/"
        $visibilityBindings = @($receiptBindings | Where-Object {
            [string]$_.target_pointer -clike "$visibilityPrefix*"
        } | Sort-Object { [int](([string]$_.target_pointer).Substring($visibilityPrefix.Length)) })
        $visibilityScopes = @($visibilityBindings | ForEach-Object { [string]$_.resource_key })
        $declaredVisibility = @($datasets[$datasetIndex].definition.visibility_node_ids)
        if ($declaredVisibility.Count -ne $formScopes.Count -or
            $visibilityBindings.Count -ne $formScopes.Count -or
            @($visibilityBindings | Where-Object {
                [string]$_.source_owner -cne "core" -or
                [string]$_.value_encoding -cne "string"
            }).Count -ne 0 -or
            (($visibilityScopes -join "`n") -cne ($formScopes -join "`n"))) {
            $datasetKey = [string]$datasets[$datasetIndex].resource_key
            throw "Reference Dataset '$datasetKey' visibility does not contain its Form '$formKey' source scopes exactly."
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
        [string]$Fixture.contract -cne "tessara.sprint-8c.response-provider-faults" -or
        [string]$Fixture.isolation -cne "one-binding-per-proxy" -or
        [bool]$Fixture.product_bypass_forbidden -ne $true -or
        [bool]$Fixture.database_bypass_forbidden -ne $true) {
        throw "Sprint 8C provider-fault contract identity or isolation policy is invalid."
    }
    Assert-ExactSequence -Expected @(
        "tessara.responses.form-version", "tessara.responses.workflow-context",
        "tessara.responses.workflow-assignments", "tessara.datasets.response-export",
        "core.response-reverse-providers"
    ) -Actual @($Fixture.runtime_configuration.bindings.binding) -Label "provider fault bindings"
    if (@($Fixture.runtime_configuration.bindings.service | Sort-Object -Unique).Count -ne 5 -or
        @($Fixture.runtime_configuration.bindings.endpoint | Sort-Object -Unique).Count -ne 5) {
        throw "Every Sprint 8C provider binding must use a separately addressable proxy."
    }
    Assert-ExactSequence -Expected @(
        "response.bootstrap.mid-apply", "response.incompatible", "dataset.derived-rebuild",
        "forms.timeout", "forms.malformed", "workflow.timeout", "workflow.stale-context",
        "workflow.event-interruption", "response-export.timeout", "response-export.malformed",
        "response-export.expired-cursor", "response-export.epoch-change",
        "response-reverse.unavailable"
    ) -Actual @($Fixture.faults.key) -Label "provider fault cases"
}

function Assert-UpgradeFixture {
    param([Parameter(Mandatory)][object]$Fixture)
    if ([int]$Fixture.schema_version -ne 1 -or
        [string]$Fixture.contract -cne "tessara.sprint-8c.response-upgrade-fixture") {
        throw "Sprint 8C upgrade fixture identity is invalid."
    }
    Assert-ExactSequence -Expected @("0.9.0", "1.0.0", "0.9.0", "1.0.0") `
        -Actual @($Fixture.sequence) -Label "Response upgrade sequence"
    foreach ($release in @("0.9.0", "1.0.0")) {
        $declaration = $Fixture.response_releases.PSObject.Properties[$release].Value
        if ([string]$declaration.contract -cne "tessara.responses.product" -or
            [string]$declaration.contract_version -cne "1.0.0" -or
            [bool]$declaration.source_built -ne $true) {
            throw "Response release '$release' is not a real source-built fixture."
        }
    }
    if ([string]$Fixture.fixed_dependencies.'tessara.components' -cne "1.1.0" -or
        [string]$Fixture.fixed_dependencies.'tessara.dashboards' -cne "3.0.2") {
        throw "Upgrade fixture must keep Component 1.1.0 and Dashboard 3.0.2 fixed."
    }
    foreach ($required in @("response_state", "module_instance_identity", "typed_resource_identity", "navigation_identity", "outbox_positions")) {
        if (@($Fixture.preserved) -cnotcontains $required) { throw "Upgrade fixture omits preserved identity '$required'." }
    }
    foreach ($required in @("images", "containers", "restart_counts", "owner_data", "availability")) {
        if (@($Fixture.unrelated_unchanged) -cnotcontains $required) { throw "Upgrade fixture omits unrelated invariant '$required'." }
    }
}

function Assert-UatScenarioContract {
    param([Parameter(Mandatory)][object]$Contract)
    if ([int]$Contract.schema_version -ne 1 -or [string]$Contract.contract -cne "tessara.sprint-8c.uat-scenarios") {
        throw "Sprint 8C UAT scenario contract identity is invalid."
    }
    Assert-ExactSequence -Expected @(1..11 | ForEach-Object { "UAT-8C-{0:d2}" -f $_ }) `
        -Actual @($Contract.scenarios.id) -Label "UAT scenario IDs"
    Assert-ExactSequence -Expected @(
        "Product and UI parity", "Assignment and providers", "Scoped review",
        "Dataset and Workflow consumers", "Replay, compatibility, and outage",
        "Configuration and diagnostics", "Fresh materialization and no-op",
        "Core subtraction and isolation", "Failure recovery", "Upgrade and rollback",
        "Roadmap exit"
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
        [string]$Baseline.contract -cne "tessara.sprint-8c.response-ui-baseline" -or
        [string]$Baseline.capture_state -cne "captured-pre-extraction" -or
        [string]$Baseline.source_commit -cnotmatch '^[0-9a-f]{40}$') {
        throw "Sprint 8C UI baseline identity is invalid."
    }
    Assert-ExactSequence -Expected @(
        "response-directory", "response-start", "response-draft-detail", "response-draft-edit",
        "response-submitted-detail", "operations-response-status",
        "module-management-response", "response-directory-javascript-disabled"
    ) -Actual @($Baseline.cases.key) -Label "UI baseline cases"
    foreach ($case in @($Baseline.cases)) {
        if (@($case.console.errors).Count -ne 0) { throw "UI baseline '$($case.key)' contains console errors." }
        $screenshotPath = Join-Path $repoRoot "docs/audits/sprint-8c-response-ui-baseline/$($case.screenshot)"
        if (-not (Test-Path -LiteralPath $screenshotPath -PathType Leaf)) {
            throw "UI baseline screenshot '$($case.screenshot)' is missing."
        }
        $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $screenshotPath).Hash.ToLowerInvariant()
        if ($actualHash -cne [string]$case.screenshot_sha256) {
            throw "UI baseline screenshot '$($case.screenshot)' does not match its recorded SHA-256."
        }
        if (@($case.external_requests).Count -ne 0) {
            throw "UI baseline '$($case.key)' contains external browser asset traffic."
        }
    }
    Assert-ExactSequence -Expected @(
        "/responses", "/responses/new", "/responses/{response_id}",
        "/responses/{response_id}/edit", "/operations",
        "/administration/modules/tessara.responses"
    ) -Actual @($Baseline.required_matrix.routes) -Label "UI baseline route matrix"
    Assert-ExactSequence -Expected @(
        "owner", "delegate", "manager", "restricted", "administrator"
    ) -Actual @($Baseline.required_matrix.roles) -Label "UI baseline role matrix"
    Assert-ExactSequence -Expected @(
        "populated", "empty", "loading", "draft", "submitted", "delegated",
        "restricted", "provider_degraded", "validation_error", "unsaved_dirty"
    ) -Actual @($Baseline.required_matrix.states) -Label "UI baseline state matrix"
    Assert-ExactSequence -Expected @(
        "light", "dark", "stored_theme", "system_theme"
    ) -Actual @($Baseline.required_matrix.themes) -Label "UI baseline theme matrix"
    Assert-ExactSequence -Expected @(
        "1440x1000", "1024x1366", "390x844", "200_percent_zoom"
    ) -Actual @($Baseline.required_matrix.viewports) -Label "UI baseline viewport matrix"
    Assert-ExactSequence -Expected @(
        "javascript_disabled_ssr", "direct_refresh", "hydrated",
        "lifecycle_navigation", "no_external_assets"
    ) -Actual @($Baseline.required_matrix.runtime) -Label "UI baseline runtime matrix"
    $coreHead = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web/src/document/assets.rs")
    $responseEntry = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web-responses/assets/response.js")
    if ($coreHead.Contains("cdnjs.cloudflare.com") -or $responseEntry -match 'https?://') {
        throw "Current Response/Core document source still depends on an external browser asset."
    }
}

function Assert-AcceptanceManifest {
    param([Parameter(Mandatory)][object]$Manifest)
    if ([int]$Manifest.schema_version -ne 2 -or @($Manifest.files).Count -eq 0) {
        throw "Sprint 8C browser acceptance manifest identity is invalid."
    }
    if ([int]$Manifest.expected_total -ne 105) {
        throw "Sprint 8C browser acceptance must retain the discovered exact 105-test inventory."
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
        "canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas",
        "Sprint 8C independent Response module › direct documents use only Response-owned public browser routes",
        "Sprint 8C independent Response module › assignment-only start options reject retired Core start routes",
        "Sprint 8C independent Response module › lifecycle navigation preserves unsaved draft state when discard is declined",
        "Sprint 8C independent Response module › scoped review and module diagnostics remain explicit and nondisclosing",
        "canonical module UI visual baselines › Responses directory at 1440 px (light)",
        "canonical module UI visual baselines › Responses start at 390 px (dark)",
        "canonical module UI visual baselines › Responses draft detail at 1024 px (light)",
        "canonical module UI visual baselines › Responses draft editor at 1440 px (dark)",
        "canonical module UI visual baselines › Responses submitted detail at 390 px (light)",
        "canonical module UI visual baselines › Responses Module Management at 1440 px (light)",
        "root route renders the native Home shell without a Core-owned Response projection",
        "capability + scope + ownership permissions › response edit route follows ownership and delegation permissions",
        "capability + scope + ownership permissions › submission management combines scope with response ownership",
        "capability + scope + ownership permissions › JavaScript-disabled Response and Dataset routes preserve native SSR ownership",
        "workflow-mediated form shortcuts › Assign Form uses workflow assignments and the Response-owned start contract",
        "workflow-mediated form shortcuts › response start options are assignment-only"
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
    $responseSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "end2end/tests/responses-module.spec.ts"
    )
    foreach ($requiredResponseIdentity in @(
        '"/api/responses"', '"/api/responses/start-options"',
        'RETIRED_RESPONSE_BROWSER_PATHS',
        'isAllowedResponseDocumentApiPath',
        'responseRouteParity',
        'expect(lifecycle).toEqual(direct)',
        'RESPONSE_DIAGNOSTIC_SECTIONS',
        'forbiddenDiagnosticValues',
        'expect(knownForbidden.status()).toBe(404)',
        'expect(randomForbidden.status()).toBe(404)',
        'Discard unsaved Response changes?'
    )) {
        if (-not $responseSource.Contains($requiredResponseIdentity)) {
            throw "Response browser acceptance omits current owner predicate '$requiredResponseIdentity'."
        }
    }
    foreach ($retiredPositiveUse in @(
        'page.request.post("/api/workflow-assignments/',
        'page.request.get("/api/responses/options")',
        'page.request.get("/api/submissions")'
    )) {
        if ($responseSource.Contains($retiredPositiveUse)) {
            throw "Response browser acceptance uses retired positive route '$retiredPositiveUse'."
        }
    }
    $responseVisualSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "end2end/tests/module-ui-visual.spec.ts"
    )
    foreach ($acceptedVisualKey in @(
        'expectAcceptedResponseVisualFrame(page, "response-directory"',
        'expectAcceptedResponseVisualFrame(page, "response-start"',
        'expectAcceptedResponseVisualFrame(page, "response-draft-detail"',
        'expectAcceptedResponseVisualFrame(page, "response-draft-edit"',
        'expectAcceptedResponseVisualFrame(page, "response-submitted-detail"',
        'expectAcceptedResponseVisualFrame(page, "module-management-response"',
        'baseline-index.json', 'createImageBitmap', 'diffPixelRatio',
        'meanColorDelta', 'externalRequests'
    )) {
        if (-not $responseVisualSource.Contains($acceptedVisualKey)) {
            throw "Response visual acceptance omits accepted continuity predicate '$acceptedVisualKey'."
        }
    }
    $responseEntrypoint = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "crates/tessara-web-responses/assets/response.js"
    )
    foreach ($lifecycleIdentity in @(
        "export async function createModule(host)", "mount_response", "navigate_response",
        "can_deactivate_response", "suspend_response", "resume_response", "unmount_response",
        "Discard unsaved Response changes?"
    )) {
        if (-not $responseEntrypoint.Contains($lifecycleIdentity)) {
            throw "Response browser lifecycle entrypoint omits '$lifecycleIdentity'."
        }
    }
    $responseManifest = Read-TrackedJson "crates/tessara-response-module/manifest.json"
    $expectedResponseLifecycle = [pscustomobject][ordered]@{
        lifecycle_abi = "1.0.0"
        entry_asset = "/response.js"
        stylesheet_assets = @("/response.css")
        complete_document_fallback = $true
        capabilities = [pscustomobject][ordered]@{
            navigation_guard = $true
            suspend_resume = $true
        }
    }
    if (($responseManifest.browser_lifecycle | ConvertTo-Json -Depth 10 -Compress) -cne
        ($expectedResponseLifecycle | ConvertTo-Json -Depth 10 -Compress)) {
        throw "Response manifest does not declare the exact lifecycle-v1 browser contract."
    }
    $responseDirtySource = @(
        Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web-responses/src/components/edit_form.rs")
        Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web-responses/src/actions.rs")
        Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-web-responses/src/edit.rs")
    ) -join "`n"
    foreach ($dirtyIdentity in @(
        "set_lifecycle_dirty(true)", "set_lifecycle_dirty(false)",
        "on:input", "on:change", "on_cleanup"
    )) {
        if (-not $responseDirtySource.Contains($dirtyIdentity)) {
            throw "Response lifecycle dirty-state source omits '$dirtyIdentity'."
        }
    }
    $responseApiSource = Get-Content -Raw -LiteralPath (
        Join-Path $repoRoot "crates/tessara-web-responses/src/api.rs"
    )
    foreach ($mutationIdentity in @(
        "RESPONSE_IDEMPOTENCY_HEADER", "Uuid::new_v4()", "mutation_request(",
        'Request::post("/api/responses")', '/values', '/submit'
    )) {
        if (-not $responseApiSource.Contains($mutationIdentity)) {
            throw "Response browser mutation transport omits '$mutationIdentity'."
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
        'kind: "module_instance"', "module_instance_id: datasetModule.entry.instance.id",
        'const RESPONSES_DEFINITION = "tessara.responses"',
        'const responseDetail = await independentModuleDetail(',
        'provider_request_timeout_seconds: 5', 'workflow_event_page_size: 250',
        'tessara.forms.form-version-schema', 'tessara.workflows.response-context',
        'tessara.workflows.response-assignment-catalog'
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
    $changeLog = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8c-test-change-log.md")
    foreach ($requiredLogIdentity in @(
        "Core `/api/submissions` and Workflow Response adapters",
        "Response owner contract, persistence, bootstrap, product/provider suites",
        "Home Assigned to Me and Organization Related Responses acceptance",
        "Response directory/start/detail/edit, assignment-only, scoped-review",
        "root route renders the native Home shell without a Core-owned Response projection",
        "Assign Form uses workflow assignments and the Response-owned start contract",
        "four Response-owner browser identities and six Response visual identities",
        "Sprint 8B Dataset fixture and lifecycle scaffolds copied for derivation",
        "Response-only upgrade/rollback",
        "browser acceptance manifest remains literal and zero-skip guarded"
    )) {
        if (-not $changeLog.Contains($requiredLogIdentity)) {
            throw "Sprint 8C test-change log does not account for '$requiredLogIdentity'."
        }
    }
}

$reference = Read-TrackedJson "deploy/sprint-8c/fixtures/reference-fixture-contract.json"
$blueprint = Read-TrackedJson "deploy/sprint-8c/blueprints/reference.json"
$faults = Read-TrackedJson "deploy/sprint-8c/fixtures/provider-fault-contract.json"
$upgrade = Read-TrackedJson "deploy/sprint-8c/fixtures/upgrade-fixture-contract.json"
$scenarios = Read-TrackedJson "docs/sprints/sprint-8c-uat/scenario-contract.json"
$baseline = Read-TrackedJson "docs/audits/sprint-8c-response-ui-baseline/baseline-index.json"
$acceptance = Read-TrackedJson "end2end/acceptance-manifest.json"

Assert-ReferenceFixture $reference
Assert-ReferenceActorBlueprintAlignment -Fixture $reference -Blueprint $blueprint
Assert-ReferenceFormWorkflowBlueprintAlignment -Fixture $reference -Blueprint $blueprint
Assert-ReferenceFormDatasetBlueprintAlignment -Fixture $reference -Blueprint $blueprint
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

    $tamperedFormFixture = $reference | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedPrimaryFixtureForm = @($tamperedFormFixture.form_versions |
        Where-Object { [string]$_.key -ceq "form.primary/v1" })[0]
    $tamperedPrimaryFixtureForm.scopes = @("scope.full")
    $tamperedFormBlueprint = $blueprint | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedPrimaryForm = @($tamperedFormBlueprint.core.bootstrap.value.forms |
        Where-Object { [string]$_.resource_key -ceq "form.primary/v1" })[0]
    $tamperedPrimaryForm.scope_node_keys = @("scope.full")
    $rejected = $false
    try {
        Assert-ReferenceFormWorkflowBlueprintAlignment -Fixture $tamperedFormFixture `
            -Blueprint $tamperedFormBlueprint
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Acceptance contract self-test admitted a workflow node outside its Form source scopes."
    }

    $tamperedDatasetBlueprint = $blueprint | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedDatasetModule = @($tamperedDatasetBlueprint.modules | Where-Object {
        [string]$_.definition_id -ceq "tessara.datasets"
    })[0]
    $tamperedDatasetModule.bootstrap.value.datasets[0].definition.visibility_node_ids = @($null)
    $tamperedDatasetModule.bootstrap.receipt_bindings = @(
        $tamperedDatasetModule.bootstrap.receipt_bindings | Where-Object {
            [string]$_.target_pointer -cne "/datasets/0/definition/visibility_node_ids/1"
        }
    )
    $rejected = $false
    try {
        Assert-ReferenceFormDatasetBlueprintAlignment -Fixture $reference `
            -Blueprint $tamperedDatasetBlueprint
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Acceptance contract self-test admitted Dataset visibility narrower than its Form source."
    }

    $tamperedDownstreamBlueprint = $blueprint | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $tamperedDashboard = @($tamperedDownstreamBlueprint.modules | Where-Object {
        [string]$_.definition_id -ceq "tessara.dashboards"
    })[0]
    $tamperedDashboard.bootstrap.receipt_bindings[4].resource_key =
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

    $tamperedResponseManifest = $acceptance | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $responseManifestFile = @($tamperedResponseManifest.files | Where-Object {
        [string]$_.path -ceq "responses-module.spec.ts"
    })[0]
    $responseManifestFile.tests[0] =
        "Sprint 8C independent Response module › substituted weaker predicate"
    $rejected = $false
    try {
        Assert-AcceptanceManifest $tamperedResponseManifest
    } catch { $rejected = $true }
    if (-not $rejected) {
        throw "Acceptance contract self-test admitted a missing Response-owner identity."
    }
}

$result = [ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    contract = "tessara.sprint-8c.acceptance-contract-result"
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

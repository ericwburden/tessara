Set-StrictMode -Version Latest

if ($null -eq (Get-Variable -Name Sprint8AFixture -Scope Script -ErrorAction SilentlyContinue)) {
    . (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")
}

$script:Sprint8ADashboardDependencyFixture = [ordered]@{
    dashboard_id = [string]$script:Sprint8AFixture.dashboard_id
    blocked_placement_id = "01980000-0003-7000-8000-000000000005"
    upgrade_placement_id = "01980000-0003-7000-8000-000000000006"
    replace_placement_id = "01980000-0003-7000-8000-000000000007"
    remove_placement_id = "01980000-0003-7000-8000-000000000008"
    scope_node_id = "01980000-0002-7000-8000-000000000002"
    isolation_actor_email = "sprint-8a-dashboard-context@tessara.local"
    initial_placement_ids = @(
        "01980000-0003-7000-8000-000000000002",
        "01980000-0003-7000-8000-000000000003",
        "01980000-0003-7000-8000-000000000004",
        "01980000-0003-7000-8000-000000000005",
        "01980000-0003-7000-8000-000000000006",
        "01980000-0003-7000-8000-000000000007",
        "01980000-0003-7000-8000-000000000008"
    )
    outage_placement_ids = @(
        "01980000-0003-7000-8000-000000000002",
        "01980000-0003-7000-8000-000000000003",
        "01980000-0003-7000-8000-000000000004",
        "01980000-0003-7000-8000-000000000006",
        "01980000-0003-7000-8000-000000000007"
    )
    unrelated_route_path = "/api/shell/navigation"
}

$script:Sprint8ADashboardDependencyCheckCodes = @(
    "exact_initial_composition",
    "exact_lifecycle_finding_placements",
    "inactive_successor_findings",
    "blocked_scope_nondisclosure",
    "successor_disclosed_for_authorized_finding",
    "defer_advances_finding",
    "defer_preserves_composition",
    "deferred_finding_remains_actionable",
    "upgrade_uses_declared_successor",
    "replacement_reference_available",
    "replace_uses_authorized_renderable_reference",
    "remove_consumes_independent_placement",
    "action_fixtures_remain_independent",
    "provider_outage_is_contained",
    "outage_blocked_scope_nondisclosure",
    "outage_composition_projection_is_safe",
    "authorization_contexts_are_isolated",
    "unrelated_route_remains_healthy",
    "identical_outage_reopens",
    "open_outage_refresh_is_idempotent",
    "provider_recovery_converges"
)
$script:Sprint8ADashboardDependencyActions = @("defer", "remove", "replace", "upgrade")
$script:Sprint8ADashboardDependencySnapshotStages = @(
    "initial", "after_defer", "after_upgrade", "after_replace", "after_remove", "recovered"
)

function ConvertTo-Sprint8ADashboardDependencyContractDocument {
    param([Parameter(Mandatory)]$Value)

    $Value | ConvertTo-Json -Depth 50 -Compress | ConvertFrom-Json
}

function Assert-Sprint8AExactStringArray {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Actual,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Expected,
        [Parameter(Mandatory)][string]$Label,
        [switch]$PreserveOrder
    )

    $actualStrings = @($Actual | ForEach-Object { [string]$_ })
    $expectedStrings = @($Expected | ForEach-Object { [string]$_ })
    if (-not $PreserveOrder) {
        $actualStrings = @($actualStrings | Sort-Object)
        $expectedStrings = @($expectedStrings | Sort-Object)
    }
    if (($actualStrings -join "`n") -cne ($expectedStrings -join "`n")) {
        throw "$Label differs from the exact Sprint 8A identity set."
    }
}

function Assert-Sprint8AExactPropertySet {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Label
    )

    $actual = @($Value.PSObject.Properties.Name | Sort-Object)
    $expectedSorted = @($Expected | Sort-Object)
    if (($actual -join "`n") -cne ($expectedSorted -join "`n")) {
        throw "$Label does not contain the exact semantic field set."
    }
}

function Assert-Sprint8ACanonicalUuid {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        $parsed = [guid]::ParseExact($Value, "D")
    } catch {
        throw "$Label is not a canonical UUID."
    }
    if ($parsed.ToString("D") -cne $Value) {
        throw "$Label is not a lowercase canonical UUID."
    }
}

function New-Sprint8AComponentReferenceIdentity {
    param([Parameter(Mandatory)][string]$ResourceId)

    [ordered]@{
        installation_id = [string]$script:Sprint8AFixture.installation_id
        owner_kind = "module_instance"
        owner_installation_id = [string]$script:Sprint8AFixture.installation_id
        module_instance_id = [string]$script:Sprint8AFixture.component_module_instance_id
        resource_type = [string]$script:Sprint8AFixture.component_resource_type
        resource_id = $ResourceId
    }
}

function Assert-Sprint8AComponentReferenceIdentity {
    param(
        [Parameter(Mandatory)]$Reference,
        [Parameter(Mandatory)][string]$ExpectedResourceId,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-Sprint8AExactPropertySet -Value $Reference -Expected @(
        "installation_id", "owner_kind", "owner_installation_id",
        "module_instance_id", "resource_type", "resource_id"
    ) -Label $Label
    $expected = New-Sprint8AComponentReferenceIdentity -ResourceId $ExpectedResourceId
    foreach ($field in $expected.Keys) {
        if ([string]$Reference.$field -cne [string]$expected[$field]) {
            throw "$Label has the wrong '$field' identity."
        }
    }
}

function Get-Sprint8AExpectedDashboardSnapshot {
    param([Parameter(Mandatory)][string]$Stage)

    $references = [ordered]@{
        "01980000-0003-7000-8000-000000000002" = [string]$script:Sprint8AFixture.component_versions.stat_card
        "01980000-0003-7000-8000-000000000003" = [string]$script:Sprint8AFixture.component_versions.table
        "01980000-0003-7000-8000-000000000004" = [string]$script:Sprint8AFixture.component_versions.bar
        "01980000-0003-7000-8000-000000000006" = [string]$script:Sprint8AFixture.inactive_stat_card_version_id
        "01980000-0003-7000-8000-000000000007" = [string]$script:Sprint8AFixture.inactive_stat_card_version_id
        "01980000-0003-7000-8000-000000000008" = [string]$script:Sprint8AFixture.inactive_stat_card_version_id
    }
    if ($Stage -in @("after_upgrade", "after_replace", "after_remove", "recovered")) {
        $references[$script:Sprint8ADashboardDependencyFixture.upgrade_placement_id] = [string]$script:Sprint8AFixture.component_versions.stat_card
    }
    if ($Stage -in @("after_replace", "after_remove", "recovered")) {
        $references[$script:Sprint8ADashboardDependencyFixture.replace_placement_id] = [string]$script:Sprint8AFixture.component_versions.table
    }
    $placementIds = @($script:Sprint8ADashboardDependencyFixture.initial_placement_ids)
    if ($Stage -in @("after_remove", "recovered")) {
        $references.Remove($script:Sprint8ADashboardDependencyFixture.remove_placement_id)
        $placementIds = @($placementIds | Where-Object { $_ -cne $script:Sprint8ADashboardDependencyFixture.remove_placement_id })
    }
    [ordered]@{
        placement_ids = @($placementIds | Sort-Object)
        references = $references
        nondisclosed_placement_ids = @($script:Sprint8ADashboardDependencyFixture.blocked_placement_id)
    }
}

function Assert-Sprint8ADashboardCompositionSnapshot {
    param(
        [Parameter(Mandatory)]$Snapshot,
        [Parameter(Mandatory)][string]$Stage
    )

    Assert-Sprint8AExactPropertySet -Value $Snapshot -Expected @(
        "stage", "placement_ids", "references", "nondisclosed_placement_ids"
    ) -Label "Dashboard composition snapshot '$Stage'"
    if ([string]$Snapshot.stage -cne $Stage) {
        throw "Dashboard composition snapshot stage '$($Snapshot.stage)' does not match '$Stage'."
    }
    $expected = Get-Sprint8AExpectedDashboardSnapshot -Stage $Stage
    Assert-Sprint8AExactStringArray -Actual @($Snapshot.placement_ids) -Expected @($expected.placement_ids) -Label "Dashboard placement inventory at '$Stage'"
    Assert-Sprint8AExactStringArray -Actual @($Snapshot.nondisclosed_placement_ids) -Expected @($expected.nondisclosed_placement_ids) -Label "Dashboard nondisclosure inventory at '$Stage'"

    $referenceRows = @($Snapshot.references)
    $actualReferencePlacements = @($referenceRows | ForEach-Object { [string]$_.placement_id })
    Assert-Sprint8AExactStringArray -Actual $actualReferencePlacements -Expected @($expected.references.Keys) -Label "Dashboard disclosed reference placements at '$Stage'"
    if (@($actualReferencePlacements | Sort-Object -Unique).Count -ne $expected.references.Count) {
        throw "Dashboard composition snapshot '$Stage' contains duplicate reference placements."
    }
    foreach ($placementId in $expected.references.Keys) {
        $row = @($referenceRows | Where-Object { [string]$_.placement_id -ceq $placementId })[0]
        Assert-Sprint8AExactPropertySet -Value $row -Expected @("placement_id", "reference") -Label "Dashboard reference row '$placementId' at '$Stage'"
        Assert-Sprint8AComponentReferenceIdentity -Reference $row.reference -ExpectedResourceId $expected.references[$placementId] -Label "Dashboard reference '$placementId' at '$Stage'"
    }
}

function Assert-Sprint8ADashboardCompositionSnapshots {
    param([Parameter(Mandatory)][object[]]$Snapshots)

    $stages = @($Snapshots | ForEach-Object { [string]$_.stage })
    Assert-Sprint8AExactStringArray -Actual $stages -Expected $script:Sprint8ADashboardDependencySnapshotStages -Label "Dashboard composition snapshot stages"
    if (@($stages | Sort-Object -Unique).Count -ne $script:Sprint8ADashboardDependencySnapshotStages.Count) {
        throw "Dashboard composition snapshots contain duplicate stages."
    }
    foreach ($stage in $script:Sprint8ADashboardDependencySnapshotStages) {
        $snapshot = @($Snapshots | Where-Object { [string]$_.stage -ceq $stage })[0]
        Assert-Sprint8ADashboardCompositionSnapshot -Snapshot $snapshot -Stage $stage
    }
}

function Get-Sprint8ADashboardSnapshotReference {
    param(
        [Parameter(Mandatory)]$Snapshot,
        [Parameter(Mandatory)][string]$PlacementId
    )

    $matches = @($Snapshot.references | Where-Object { [string]$_.placement_id -ceq $PlacementId })
    if ($matches.Count -eq 0) { return $null }
    if ($matches.Count -ne 1) { throw "Dashboard composition snapshot contains duplicate placement '$PlacementId'." }
    return $matches[0].reference
}

function Assert-Sprint8AOutageFindingIdentities {
    param(
        [Parameter(Mandatory)][object[]]$Findings,
        [Parameter(Mandatory)][string]$Label
    )

    $placements = @($Findings | ForEach-Object { [string]$_.placement_id })
    Assert-Sprint8AExactStringArray -Actual $placements -Expected @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids) -Label "$Label placements"
    if (@($placements | Sort-Object -Unique).Count -ne $script:Sprint8ADashboardDependencyFixture.outage_placement_ids.Count) {
        throw "$Label contains duplicate placements."
    }
    foreach ($finding in $Findings) {
        Assert-Sprint8AExactPropertySet -Value $finding -Expected @(
            "placement_id", "finding_id", "finding_revision"
        ) -Label "$Label finding '$($finding.placement_id)'"
        Assert-Sprint8ACanonicalUuid -Value ([string]$finding.finding_id) -Label "$Label finding id"
        if ([long]$finding.finding_revision -lt 1) {
            throw "$Label contains a non-positive finding revision."
        }
    }
}

function Assert-Sprint8AOutageCompositionProjection {
    param([Parameter(Mandatory)]$Composition)

    Assert-Sprint8AExactPropertySet -Value $Composition -Expected @(
        "placement_ids", "provider_unavailable_placement_ids", "restricted_placement_ids",
        "component_metadata_disclosed_placement_ids"
    ) -Label "Dashboard outage composition projection"
    $expectedPlacements = @($script:Sprint8ADashboardDependencyFixture.initial_placement_ids |
        Where-Object { $_ -cne $script:Sprint8ADashboardDependencyFixture.remove_placement_id })
    Assert-Sprint8AExactStringArray -Actual @($Composition.placement_ids) -Expected $expectedPlacements -Label "Dashboard outage composition placement inventory"
    Assert-Sprint8AExactStringArray -Actual @($Composition.provider_unavailable_placement_ids) -Expected @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids) -Label "Dashboard outage composition unavailable placements"
    Assert-Sprint8AExactStringArray -Actual @($Composition.restricted_placement_ids) -Expected @($script:Sprint8ADashboardDependencyFixture.blocked_placement_id) -Label "Dashboard outage composition restricted placements"
    if (@($Composition.component_metadata_disclosed_placement_ids).Count -ne 0) {
        throw "Dashboard outage composition disclosed provider metadata."
    }
}

function Assert-Sprint8ADashboardDependencyEvidence {
    param([Parameter(Mandatory)]$Evidence)

    $document = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $Evidence
    Assert-Sprint8AExactPropertySet -Value $document -Expected @(
        "schema_version", "evidence_kind", "dashboard_id", "checks", "actions",
        "reference_catalog", "initial_findings", "initial_blocked_placement_disclosed",
        "initial_health", "composition_snapshots", "outage", "context_isolation", "repeat_outage",
        "final_health", "failure_count", "blocked_count",
        "duplicate_check_codes", "fatal_errors", "harvesting_complete", "canonical_reset_required", "passed"
    ) -Label "Sprint 8A Dashboard dependency evidence"
    if ([int]$document.schema_version -ne 3 -or
        [string]$document.evidence_kind -cne "tessara.sprint-8a.dashboard-dependency-semantic-diagnostic" -or
        [string]$document.dashboard_id -cne $script:Sprint8ADashboardDependencyFixture.dashboard_id -or
        $document.harvesting_complete -ne $true -or
        $document.canonical_reset_required -ne $true -or
        $document.passed -ne $true -or
        [long]$document.failure_count -ne 0 -or
        [long]$document.blocked_count -ne 0 -or
        @($document.duplicate_check_codes).Count -ne 0 -or
        @($document.fatal_errors).Count -ne 0) {
        throw "Sprint 8A Dashboard dependency evidence is not a complete passing schema-v3 diagnostic."
    }

    $actualChecks = @($document.checks)
    $actualCodes = @($actualChecks | ForEach-Object { [string]$_.code })
    Assert-Sprint8AExactStringArray -Actual $actualCodes -Expected $script:Sprint8ADashboardDependencyCheckCodes -Label "Dashboard dependency check codes"
    if (@($actualCodes | Sort-Object -Unique).Count -ne $script:Sprint8ADashboardDependencyCheckCodes.Count) {
        throw "Dashboard dependency evidence contains duplicate check codes."
    }
    foreach ($check in $actualChecks) {
        Assert-Sprint8AExactPropertySet -Value $check -Expected @(
            "code", "state", "passed", "detail", "dependency_reason"
        ) -Label "Dashboard dependency check '$($check.code)'"
        if ([string]$check.state -cne "passed" -or $check.passed -ne $true -or $null -ne $check.dependency_reason) {
            throw "Dashboard dependency check '$($check.code)' is not a complete passing check."
        }
    }

    Assert-Sprint8AExactPropertySet -Value $document.reference_catalog -Expected @(
        "declared_successor", "replacement"
    ) -Label "Dashboard dependency reference catalog"
    Assert-Sprint8AComponentReferenceIdentity -Reference $document.reference_catalog.declared_successor -ExpectedResourceId $script:Sprint8AFixture.component_versions.stat_card -Label "Declared Upgrade successor"
    Assert-Sprint8AComponentReferenceIdentity -Reference $document.reference_catalog.replacement -ExpectedResourceId $script:Sprint8AFixture.component_versions.table -Label "Authorized Replace reference"

    $initialFindings = @($document.initial_findings)
    $initialFindingPlacements = @($initialFindings | ForEach-Object { [string]$_.placement_id })
    $actionPlacements = @(
        $script:Sprint8ADashboardDependencyFixture.upgrade_placement_id,
        $script:Sprint8ADashboardDependencyFixture.replace_placement_id,
        $script:Sprint8ADashboardDependencyFixture.remove_placement_id
    )
    Assert-Sprint8AExactStringArray -Actual $initialFindingPlacements -Expected $actionPlacements -Label "Initial Dashboard lifecycle findings"
    if ($document.initial_blocked_placement_disclosed -ne $false) {
        throw "The blocked Dashboard placement was disclosed by initial dependency evidence."
    }
    $initialFindingIds = @{}
    $initialFindingRevisions = @{}
    foreach ($finding in $initialFindings) {
        Assert-Sprint8AExactPropertySet -Value $finding -Expected @(
            "placement_id", "finding_id", "finding_code", "disposition", "finding_revision",
            "observed_lifecycle", "publication_state", "successor_available", "saved_reference"
        ) -Label "Initial Dashboard finding '$($finding.placement_id)'"
        Assert-Sprint8ACanonicalUuid -Value ([string]$finding.finding_id) -Label "Initial Dashboard finding id"
        if ([string]$finding.finding_code -cne "lifecycle_unrenderable" -or
            [string]$finding.disposition -cne "open" -or
            [long]$finding.finding_revision -lt 1 -or
            [string]$finding.observed_lifecycle -cne "inactive" -or
            [string]$finding.publication_state -cne "superseded" -or
            $finding.successor_available -ne $true) {
            throw "Initial Dashboard finding '$($finding.placement_id)' does not prove the exact inactive-successor semantics."
        }
        Assert-Sprint8AComponentReferenceIdentity -Reference $finding.saved_reference -ExpectedResourceId $script:Sprint8AFixture.inactive_stat_card_version_id -Label "Initial saved reference '$($finding.placement_id)'"
        $initialFindingIds[[string]$finding.placement_id] = [string]$finding.finding_id
        $initialFindingRevisions[[string]$finding.placement_id] = [long]$finding.finding_revision
    }
    Assert-Sprint8AExactPropertySet -Value $document.initial_health -Expected @(
        "health", "open_count", "deferred_count"
    ) -Label "Initial Dashboard dependency health"
    if ([string]$document.initial_health.health -cne "degraded" -or
        [long]$document.initial_health.open_count -ne 3 -or
        [long]$document.initial_health.deferred_count -ne 0) {
        throw "Initial Dashboard dependency health leaks a hidden finding through aggregate state."
    }

    $actions = @($document.actions)
    $actualActions = @($actions | ForEach-Object { [string]$_.action })
    Assert-Sprint8AExactStringArray -Actual $actualActions -Expected $script:Sprint8ADashboardDependencyActions -Label "Dashboard dependency actions"
    if (@($actualActions | Sort-Object -Unique).Count -ne $script:Sprint8ADashboardDependencyActions.Count) {
        throw "Dashboard dependency evidence contains duplicate actions."
    }
    foreach ($action in $actions) {
        Assert-Sprint8AExactPropertySet -Value $action -Expected @(
            "action", "placement_id", "finding_id", "request_finding_revision",
            "result_finding_revision", "disposition", "before_reference", "requested_reference",
            "after_reference", "placement_present_after"
        ) -Label "Dashboard dependency action '$($action.action)'"
        Assert-Sprint8ACanonicalUuid -Value ([string]$action.finding_id) -Label "Dashboard dependency action finding id"
        if ([long]$action.result_finding_revision -ne ([long]$action.request_finding_revision + 1)) {
            throw "Dashboard dependency action '$($action.action)' did not advance the exact finding revision."
        }
        $expectedPlacement = switch ([string]$action.action) {
            "defer" { $script:Sprint8ADashboardDependencyFixture.upgrade_placement_id }
            "upgrade" { $script:Sprint8ADashboardDependencyFixture.upgrade_placement_id }
            "replace" { $script:Sprint8ADashboardDependencyFixture.replace_placement_id }
            "remove" { $script:Sprint8ADashboardDependencyFixture.remove_placement_id }
        }
        if ([string]$action.placement_id -cne $expectedPlacement -or
            [string]$action.finding_id -cne $initialFindingIds[$expectedPlacement]) {
            throw "Dashboard dependency action '$($action.action)' did not consume its exact independent finding."
        }
        Assert-Sprint8AComponentReferenceIdentity -Reference $action.before_reference -ExpectedResourceId $script:Sprint8AFixture.inactive_stat_card_version_id -Label "Dashboard action '$($action.action)' before reference"
        switch ([string]$action.action) {
            "defer" {
                if ([string]$action.disposition -cne "deferred" -or $action.placement_present_after -ne $true -or $null -ne $action.requested_reference) {
                    throw "Defer does not retain the exact actionable placement semantics."
                }
                Assert-Sprint8AComponentReferenceIdentity -Reference $action.after_reference -ExpectedResourceId $script:Sprint8AFixture.inactive_stat_card_version_id -Label "Defer after reference"
            }
            "upgrade" {
                if ([string]$action.disposition -cne "resolved" -or $action.placement_present_after -ne $true -or $null -ne $action.requested_reference) {
                    throw "Upgrade does not retain the exact successor action semantics."
                }
                Assert-Sprint8AComponentReferenceIdentity -Reference $action.after_reference -ExpectedResourceId $script:Sprint8AFixture.component_versions.stat_card -Label "Upgrade after reference"
            }
            "replace" {
                if ([string]$action.disposition -cne "resolved" -or $action.placement_present_after -ne $true) {
                    throw "Replace does not retain the exact replacement action semantics."
                }
                Assert-Sprint8AComponentReferenceIdentity -Reference $action.requested_reference -ExpectedResourceId $script:Sprint8AFixture.component_versions.table -Label "Replace requested reference"
                Assert-Sprint8AComponentReferenceIdentity -Reference $action.after_reference -ExpectedResourceId $script:Sprint8AFixture.component_versions.table -Label "Replace after reference"
            }
            "remove" {
                if ([string]$action.disposition -cne "resolved" -or $action.placement_present_after -ne $false -or
                    $null -ne $action.requested_reference -or $null -ne $action.after_reference) {
                    throw "Remove did not delete only its exact independent placement."
                }
            }
        }
    }
    $defer = @($actions | Where-Object action -CEQ "defer")[0]
    $upgrade = @($actions | Where-Object action -CEQ "upgrade")[0]
    if ([long]$defer.request_finding_revision -ne $initialFindingRevisions[$script:Sprint8ADashboardDependencyFixture.upgrade_placement_id] -or
        [long]$upgrade.request_finding_revision -ne [long]$defer.result_finding_revision) {
        throw "Upgrade did not consume the exact revision returned by Defer."
    }
    foreach ($actionName in @("replace", "remove")) {
        $action = @($actions | Where-Object action -CEQ $actionName)[0]
        if ([long]$action.request_finding_revision -ne $initialFindingRevisions[[string]$action.placement_id]) {
            throw "Dashboard action '$actionName' did not consume its initial independent finding revision."
        }
    }

    Assert-Sprint8ADashboardCompositionSnapshots -Snapshots @($document.composition_snapshots)

    Assert-Sprint8AExactPropertySet -Value $document.outage -Expected @(
        "health", "open_count", "deferred_count", "finding_placements", "finding_codes",
        "finding_identities", "blocked_placement_disclosed", "composition", "unrelated_route"
    ) -Label "Dashboard dependency outage evidence"
    Assert-Sprint8AExactStringArray -Actual @($document.outage.finding_placements) -Expected @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids) -Label "Dashboard provider-outage findings"
    Assert-Sprint8AExactStringArray -Actual @($document.outage.finding_codes) -Expected @(1..5 | ForEach-Object { "provider_unavailable" }) -Label "Dashboard provider-outage finding codes" -PreserveOrder
    if ($document.outage.blocked_placement_disclosed -ne $false) {
        throw "The blocked Dashboard placement was disclosed during provider outage."
    }
    if ([string]$document.outage.health -cne "degraded" -or
        [long]$document.outage.open_count -ne 5 -or
        [long]$document.outage.deferred_count -ne 0) {
        throw "Dashboard provider-outage health leaks a hidden finding through aggregate state."
    }
    Assert-Sprint8AOutageFindingIdentities -Findings @($document.outage.finding_identities) -Label "Primary Dashboard outage"
    Assert-Sprint8AOutageCompositionProjection -Composition $document.outage.composition
    Assert-Sprint8AExactPropertySet -Value $document.outage.unrelated_route -Expected @(
        "path", "status_code", "schema_version", "state"
    ) -Label "Dashboard outage unrelated-route observation"
    if ([string]$document.outage.unrelated_route.path -cne $script:Sprint8ADashboardDependencyFixture.unrelated_route_path -or
        [int]$document.outage.unrelated_route.status_code -ne 200 -or
        [int]$document.outage.unrelated_route.schema_version -ne 3 -or
        [string]$document.outage.unrelated_route.state -cne "available") {
        throw "The Core-owned shell navigation route was not healthy throughout the Component outage."
    }

    Assert-Sprint8AExactPropertySet -Value $document.context_isolation -Expected @(
        "actor_id", "health", "open_count", "deferred_count", "finding_placements",
        "finding_identities", "blocked_placement_disclosed", "distinct_finding_ids"
    ) -Label "Dashboard authorization-context isolation evidence"
    Assert-Sprint8ACanonicalUuid -Value ([string]$document.context_isolation.actor_id) -Label "Dashboard isolation actor id"
    Assert-Sprint8AExactStringArray -Actual @($document.context_isolation.finding_placements) -Expected @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids) -Label "Isolated-context outage findings"
    Assert-Sprint8AOutageFindingIdentities -Findings @($document.context_isolation.finding_identities) -Label "Isolated Dashboard outage"
    if ([string]$document.context_isolation.health -cne "degraded" -or
        [long]$document.context_isolation.open_count -ne 5 -or
        [long]$document.context_isolation.deferred_count -ne 0 -or
        $document.context_isolation.blocked_placement_disclosed -ne $false -or
        $document.context_isolation.distinct_finding_ids -ne $true) {
        throw "Dashboard findings were not isolated by exact semantic authorization context."
    }
    $primaryFindingIds = @($document.outage.finding_identities | ForEach-Object { [string]$_.finding_id })
    $isolatedFindingIds = @($document.context_isolation.finding_identities | ForEach-Object { [string]$_.finding_id })
    if (@($primaryFindingIds | Where-Object { $isolatedFindingIds -ccontains $_ }).Count -ne 0) {
        throw "Dashboard outage findings were reused across actor authorization contexts."
    }

    Assert-Sprint8AExactPropertySet -Value $document.repeat_outage -Expected @(
        "finding_placements", "reopened_findings", "stable_open_findings",
        "reopened_revision_advanced", "open_refresh_idempotent"
    ) -Label "Dashboard repeated-outage evidence"
    Assert-Sprint8AExactStringArray -Actual @($document.repeat_outage.finding_placements) -Expected @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids) -Label "Repeated Dashboard outage findings"
    Assert-Sprint8AOutageFindingIdentities -Findings @($document.repeat_outage.reopened_findings) -Label "Reopened Dashboard outage"
    Assert-Sprint8AOutageFindingIdentities -Findings @($document.repeat_outage.stable_open_findings) -Label "Idempotent open Dashboard outage"
    if ($document.repeat_outage.reopened_revision_advanced -ne $true -or
        $document.repeat_outage.open_refresh_idempotent -ne $true) {
        throw "Dashboard repeated-outage lifecycle was not reopen-then-idempotent."
    }
    foreach ($primary in @($document.outage.finding_identities)) {
        $reopened = @($document.repeat_outage.reopened_findings | Where-Object { [string]$_.placement_id -ceq [string]$primary.placement_id })[0]
        $stable = @($document.repeat_outage.stable_open_findings | Where-Object { [string]$_.placement_id -ceq [string]$primary.placement_id })[0]
        if ([string]$reopened.finding_id -cne [string]$primary.finding_id -or
            [long]$reopened.finding_revision -le [long]$primary.finding_revision -or
            [string]$stable.finding_id -cne [string]$reopened.finding_id -or
            [long]$stable.finding_revision -ne [long]$reopened.finding_revision) {
            throw "Dashboard repeated outage did not reuse and advance the exact resolved episode identity."
        }
    }

    Assert-Sprint8AExactPropertySet -Value $document.final_health -Expected @(
        "health", "open_count", "deferred_count", "visible_finding_placements",
        "isolated_health", "isolated_open_count", "isolated_deferred_count",
        "isolated_visible_finding_placements"
    ) -Label "Dashboard dependency recovery health"
    if ([string]$document.final_health.health -cne "healthy" -or
        [long]$document.final_health.open_count -ne 0 -or
        [long]$document.final_health.deferred_count -ne 0 -or
        @($document.final_health.visible_finding_placements).Count -ne 0 -or
        [string]$document.final_health.isolated_health -cne "healthy" -or
        [long]$document.final_health.isolated_open_count -ne 0 -or
        [long]$document.final_health.isolated_deferred_count -ne 0 -or
        @($document.final_health.isolated_visible_finding_placements).Count -ne 0) {
        throw "Dashboard dependency recovery did not converge to exact zero-finding visible health."
    }
}

function New-Sprint8ADashboardDependencySelfTestSnapshot {
    param([Parameter(Mandatory)][string]$Stage)

    $expected = Get-Sprint8AExpectedDashboardSnapshot -Stage $Stage
    $rows = @($expected.references.Keys | Sort-Object | ForEach-Object {
        [ordered]@{
            placement_id = $_
            reference = New-Sprint8AComponentReferenceIdentity -ResourceId $expected.references[$_]
        }
    })
    [ordered]@{
        stage = $Stage
        placement_ids = @($expected.placement_ids)
        references = $rows
        nondisclosed_placement_ids = @($expected.nondisclosed_placement_ids)
    }
}

function New-Sprint8ADashboardDependencySelfTestEvidence {
    $findingIds = [ordered]@{
        $script:Sprint8ADashboardDependencyFixture.upgrade_placement_id = "01980000-0004-7000-8000-000000000006"
        $script:Sprint8ADashboardDependencyFixture.replace_placement_id = "01980000-0004-7000-8000-000000000007"
        $script:Sprint8ADashboardDependencyFixture.remove_placement_id = "01980000-0004-7000-8000-000000000008"
    }
    $findings = @($findingIds.Keys | ForEach-Object {
        [ordered]@{
            placement_id = $_
            finding_id = $findingIds[$_]
            finding_code = "lifecycle_unrenderable"
            disposition = "open"
            finding_revision = 1
            observed_lifecycle = "inactive"
            publication_state = "superseded"
            successor_available = $true
            saved_reference = New-Sprint8AComponentReferenceIdentity -ResourceId $script:Sprint8AFixture.inactive_stat_card_version_id
        }
    })
    $predecessor = New-Sprint8AComponentReferenceIdentity -ResourceId $script:Sprint8AFixture.inactive_stat_card_version_id
    $successor = New-Sprint8AComponentReferenceIdentity -ResourceId $script:Sprint8AFixture.component_versions.stat_card
    $replacement = New-Sprint8AComponentReferenceIdentity -ResourceId $script:Sprint8AFixture.component_versions.table
    $actions = @(
        [ordered]@{ action = "defer"; placement_id = $script:Sprint8ADashboardDependencyFixture.upgrade_placement_id; finding_id = $findingIds[$script:Sprint8ADashboardDependencyFixture.upgrade_placement_id]; request_finding_revision = 1; result_finding_revision = 2; disposition = "deferred"; before_reference = $predecessor; requested_reference = $null; after_reference = $predecessor; placement_present_after = $true },
        [ordered]@{ action = "upgrade"; placement_id = $script:Sprint8ADashboardDependencyFixture.upgrade_placement_id; finding_id = $findingIds[$script:Sprint8ADashboardDependencyFixture.upgrade_placement_id]; request_finding_revision = 2; result_finding_revision = 3; disposition = "resolved"; before_reference = $predecessor; requested_reference = $null; after_reference = $successor; placement_present_after = $true },
        [ordered]@{ action = "replace"; placement_id = $script:Sprint8ADashboardDependencyFixture.replace_placement_id; finding_id = $findingIds[$script:Sprint8ADashboardDependencyFixture.replace_placement_id]; request_finding_revision = 1; result_finding_revision = 2; disposition = "resolved"; before_reference = $predecessor; requested_reference = $replacement; after_reference = $replacement; placement_present_after = $true },
        [ordered]@{ action = "remove"; placement_id = $script:Sprint8ADashboardDependencyFixture.remove_placement_id; finding_id = $findingIds[$script:Sprint8ADashboardDependencyFixture.remove_placement_id]; request_finding_revision = 1; result_finding_revision = 2; disposition = "resolved"; before_reference = $predecessor; requested_reference = $null; after_reference = $null; placement_present_after = $false }
    )
    $outageFindingIdentities = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids | Sort-Object | ForEach-Object -Begin { $index = 0 } -Process {
        $index++
        [ordered]@{ placement_id = $_; finding_id = ("01980000-0010-7000-8000-{0:D12}" -f $index); finding_revision = 1 }
    })
    $isolatedFindingIdentities = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids | Sort-Object | ForEach-Object -Begin { $index = 0 } -Process {
        $index++
        [ordered]@{ placement_id = $_; finding_id = ("01980000-0020-7000-8000-{0:D12}" -f $index); finding_revision = 1 }
    })
    $reopenedFindingIdentities = @($outageFindingIdentities | ForEach-Object {
        [ordered]@{ placement_id = $_.placement_id; finding_id = $_.finding_id; finding_revision = 3 }
    })
    $outageComposition = [ordered]@{
        placement_ids = @($script:Sprint8ADashboardDependencyFixture.initial_placement_ids | Where-Object { $_ -cne $script:Sprint8ADashboardDependencyFixture.remove_placement_id } | Sort-Object)
        provider_unavailable_placement_ids = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids | Sort-Object)
        restricted_placement_ids = @($script:Sprint8ADashboardDependencyFixture.blocked_placement_id)
        component_metadata_disclosed_placement_ids = @()
    }
    $evidence = [ordered]@{
        schema_version = 3
        evidence_kind = "tessara.sprint-8a.dashboard-dependency-semantic-diagnostic"
        dashboard_id = $script:Sprint8ADashboardDependencyFixture.dashboard_id
        checks = @($script:Sprint8ADashboardDependencyCheckCodes | ForEach-Object { [ordered]@{ code = $_; state = "passed"; passed = $true; detail = "semantic self-test"; dependency_reason = $null } })
        actions = $actions
        reference_catalog = [ordered]@{ declared_successor = $successor; replacement = $replacement }
        initial_findings = $findings
        initial_blocked_placement_disclosed = $false
        initial_health = [ordered]@{ health = "degraded"; open_count = 3; deferred_count = 0 }
        composition_snapshots = @($script:Sprint8ADashboardDependencySnapshotStages | ForEach-Object { New-Sprint8ADashboardDependencySelfTestSnapshot -Stage $_ })
        outage = [ordered]@{
            health = "degraded"
            open_count = 5
            deferred_count = 0
            finding_placements = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids | Sort-Object)
            finding_codes = @(1..5 | ForEach-Object { "provider_unavailable" })
            finding_identities = $outageFindingIdentities
            blocked_placement_disclosed = $false
            composition = $outageComposition
            unrelated_route = [ordered]@{ path = $script:Sprint8ADashboardDependencyFixture.unrelated_route_path; status_code = 200; schema_version = 3; state = "available" }
        }
        context_isolation = [ordered]@{
            actor_id = "01980000-0030-7000-8000-000000000001"
            health = "degraded"
            open_count = 5
            deferred_count = 0
            finding_placements = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids | Sort-Object)
            finding_identities = $isolatedFindingIdentities
            blocked_placement_disclosed = $false
            distinct_finding_ids = $true
        }
        repeat_outage = [ordered]@{
            finding_placements = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids | Sort-Object)
            reopened_findings = $reopenedFindingIdentities
            stable_open_findings = $reopenedFindingIdentities
            reopened_revision_advanced = $true
            open_refresh_idempotent = $true
        }
        final_health = [ordered]@{
            health = "healthy"; open_count = 0; deferred_count = 0; visible_finding_placements = @()
            isolated_health = "healthy"; isolated_open_count = 0; isolated_deferred_count = 0
            isolated_visible_finding_placements = @()
        }
        failure_count = 0
        blocked_count = 0
        duplicate_check_codes = @()
        fatal_errors = @()
        harvesting_complete = $true
        canonical_reset_required = $true
        passed = $true
    }
    ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $evidence
}

function Assert-Sprint8ADashboardDependencyEvidenceRejected {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Case
    )

    try {
        Assert-Sprint8ADashboardDependencyEvidence -Evidence $Evidence
    } catch {
        return
    }
    throw "Sprint 8A Dashboard dependency contract accepted invalid evidence: $Case"
}

function Test-Sprint8ADashboardDependencyEvidenceContract {
    $valid = New-Sprint8ADashboardDependencySelfTestEvidence
    Assert-Sprint8ADashboardDependencyEvidence -Evidence $valid

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.reference_catalog.declared_successor.resource_id = [string]$script:Sprint8AFixture.inactive_stat_card_version_id
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "forged Upgrade successor"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    (@($invalid.actions | Where-Object action -CEQ "replace")[0]).after_reference.resource_id = [string]$script:Sprint8AFixture.component_versions.bar
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "forged Replace readback"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    (@($invalid.actions | Where-Object action -CEQ "remove")[0]).placement_present_after = $true
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "Remove without deletion"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $afterUpgrade = @($invalid.composition_snapshots | Where-Object stage -CEQ "after_upgrade")[0]
    (@($afterUpgrade.references | Where-Object placement_id -CEQ $script:Sprint8ADashboardDependencyFixture.replace_placement_id)[0]).reference.resource_id = [string]$script:Sprint8AFixture.component_versions.table
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "Upgrade mutating the independent Replace fixture"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.initial_blocked_placement_disclosed = $true
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "blocked placement disclosure"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.initial_health.open_count = 4
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "blocked finding leaking through initial aggregate counts"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.outage.finding_placements += $script:Sprint8ADashboardDependencyFixture.blocked_placement_id
    $invalid.outage.finding_codes += "provider_unavailable"
    $invalid.outage.blocked_placement_disclosed = $true
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "blocked placement outage disclosure"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.outage.open_count = 6
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "blocked finding leaking through outage aggregate counts"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.outage.unrelated_route.status_code = 503
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "unrelated route outage"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.outage.composition.component_metadata_disclosed_placement_ids = @($script:Sprint8ADashboardDependencyFixture.outage_placement_ids[0])
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "outage composition metadata disclosure"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.context_isolation.finding_identities[0].finding_id = $invalid.outage.finding_identities[0].finding_id
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "finding identity shared across contexts"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.repeat_outage.reopened_findings[0].finding_revision = $invalid.outage.finding_identities[0].finding_revision
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "identical outage not reopened"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.repeat_outage.stable_open_findings[0].finding_revision++
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "open outage refresh advanced revision"

    $invalid = ConvertTo-Sprint8ADashboardDependencyContractDocument -Value $valid
    $invalid.final_health.open_count = 1
    Assert-Sprint8ADashboardDependencyEvidenceRejected -Evidence $invalid -Case "non-zero recovery finding count"
}

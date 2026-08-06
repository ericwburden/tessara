Set-StrictMode -Version Latest

$script:Sprint8ADashboardDependencyCheckCodes = @(
    "exact_lifecycle_finding_placements",
    "inactive_successor_findings",
    "blocked_scope_nondisclosure",
    "successor_disclosed_for_authorized_finding",
    "defer_advances_finding",
    "deferred_finding_remains_actionable",
    "upgrade_uses_declared_successor",
    "replacement_reference_available",
    "replace_uses_authorized_renderable_reference",
    "remove_consumes_independent_placement",
    "provider_outage_is_contained",
    "provider_recovery_converges"
)
$script:Sprint8ADashboardDependencyActions = @("defer", "remove", "replace", "upgrade")

function Assert-Sprint8ADashboardDependencyEvidence {
    param([Parameter(Mandatory)]$Evidence)

    $actualChecks = @($Evidence.checks)
    $actualCodes = @($actualChecks | Where-Object { $_.passed -eq $true } | ForEach-Object { [string]$_.code } | Sort-Object -Unique)
    $expectedCodes = @($script:Sprint8ADashboardDependencyCheckCodes | Sort-Object -Unique)
    $actualActions = @($Evidence.actions | ForEach-Object { [string]$_.action } | Sort-Object -Unique)
    if ($Evidence.schema_version -ne 1 -or
        [string]$Evidence.evidence_kind -cne "tessara.sprint-8a.dashboard-dependency-semantic-diagnostic" -or
        $Evidence.passed -ne $true -or
        $actualChecks.Count -ne $expectedCodes.Count -or
        ($actualCodes -join ",") -cne ($expectedCodes -join ",") -or
        ($actualActions -join ",") -cne ($script:Sprint8ADashboardDependencyActions -join ",") -or
        [string]$Evidence.final_health.health -cne "healthy" -or
        [long]$Evidence.final_health.open_count -ne 0 -or
        [long]$Evidence.final_health.deferred_count -ne 0 -or
        $Evidence.canonical_reset_required -ne $true) {
        throw "Sprint 8A Dashboard dependency evidence does not prove the exact lifecycle/action/outage semantics."
    }
}

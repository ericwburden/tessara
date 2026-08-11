[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [string]$OutputPath = "target/sprint-8a-product-diagnostic.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-dashboard-dependency-contract.ps1")

$assertions = @(
    "exact-real-module-enrollment",
    "exact-five-core-transitions",
    "module-owned-component-documents-and-assets",
    "owner-produced-fresh-seed",
    "receipt-bound-dashboard-references",
    "navigation-and-module-management-projection",
    "component-dataset-contract-execution",
    "all-six-component-kinds",
    "dashboard-placement-execution",
    "ordinary-404-for-removed-routes",
    "authorization-boundaries"
)
function New-Sprint8AProductDiagnosticReceipt {
    param(
        [Parameter(Mandatory)]$SourceIdentity,
        [Parameter(Mandatory)][string]$Fingerprint,
        [Parameter(Mandatory)][int]$AttemptNumber,
        [Parameter(Mandatory)][string]$Endpoint,
        [Parameter(Mandatory)][string]$StartedAt,
        [Parameter(Mandatory)][string]$EndedAt,
        [Parameter(Mandatory)]$DashboardDependencySemantics,
        [Parameter(Mandatory)]$DashboardDependencySemanticsEvidence,
        [Parameter(Mandatory)]$DiagnosticChecks
    )

    [ordered]@{
        schema_version = 2
        sprint = "sprint-8a"
        phase = "candidate-product-diagnostic"
        attempt = $AttemptNumber
        authoritative = $false
        state = "passed"
        started_at = $StartedAt
        ended_at = $EndedAt
        mutable_source_identity = $SourceIdentity
        environment_fingerprint = $Fingerprint
        base_url = $Endpoint
        assertions = $assertions
        diagnostic_checks = $DiagnosticChecks
        dashboard_dependency_semantics = $DashboardDependencySemantics
        dashboard_dependency_semantics_evidence = $DashboardDependencySemanticsEvidence
        acceptance_authority = [ordered]@{
            formal_uat = $false
            acceptance_evidence_published = $false
            mode = "non_acceptance_development_diagnostic"
        }
        state_cleanup = "required_by_later_failure-containment-successor-materialization"
    }
}

function Get-RetainedDiagnosticArtifact {
    param([Parameter(Mandatory)][string]$Path)
    $fullPath = [IO.Path]::GetFullPath($Path)
    $sidecar = "$fullPath.sha256"
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecar -PathType Leaf)) {
        throw "Diagnostic artifact or digest sidecar is missing: $fullPath"
    }
    $digest = Get-Sprint7AFileSha256 -Path $fullPath
    $declaredDigest = (Get-Content -LiteralPath $sidecar -Raw).Trim()
    if ($declaredDigest -cne $digest) {
        throw "Diagnostic artifact digest sidecar does not bind its raw evidence: $fullPath"
    }
    [ordered]@{ path = $fullPath; sha256 = $digest; sidecar = $sidecar }
}

if ($SelfTest) {
    Test-Sprint8ADashboardDependencyEvidenceContract
    $semanticFixture = New-Sprint8ADashboardDependencySelfTestEvidence
    Assert-Sprint8ADashboardDependencyEvidence -Evidence $semanticFixture
    $diagnosticFixture = @(
        [pscustomobject]@{ name = "broad-product-behavior"; state = "passed"; reason = $null },
        [pscustomobject]@{ name = "dashboard-dependency-semantics"; state = "passed"; reason = $null }
    )
    $selfTestRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-product-diagnostic-$([Guid]::NewGuid().ToString('N'))"
    try {
        $rawPath = Join-Path $selfTestRoot "dashboard-raw.json"
        Publish-Sprint7AEvidence -Document $semanticFixture -OutputPath $rawPath | Out-Null
        $rawEvidence = Get-RetainedDiagnosticArtifact -Path $rawPath
        $receipt = New-Sprint8AProductDiagnosticReceipt `
            -SourceIdentity ([ordered]@{ commit = "self-test"; tree = "self-test"; dirty = $false }) `
            -Fingerprint ("a" * 64) `
            -AttemptNumber 1 `
            -Endpoint "http://127.0.0.1:8088" `
            -StartedAt "2026-01-01T00:00:00Z" `
            -EndedAt "2026-01-01T00:00:01Z" `
            -DashboardDependencySemantics $semanticFixture `
            -DashboardDependencySemanticsEvidence $rawEvidence `
            -DiagnosticChecks $diagnosticFixture
        if ($receipt.authoritative -or
            $receipt.acceptance_authority.formal_uat -or
            $receipt.acceptance_authority.acceptance_evidence_published -or
            [string]$receipt.phase -cne "candidate-product-diagnostic" -or
            @($receipt.assertions).Count -lt 1 -or
            @($receipt.diagnostic_checks | Where-Object state -CNE "passed").Count -ne 0 -or
            $receipt.dashboard_dependency_semantics.passed -ne $true -or
            [string]$receipt.dashboard_dependency_semantics_evidence.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "Sprint 8A product-diagnostic self-test found acceptance authority, raw-evidence loss, or an empty semantic inventory."
        }
    } finally {
        Remove-Item -LiteralPath $selfTestRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
    Write-Host "Sprint 8A non-acceptance product-diagnostic self-test passed."
    return
}

if ($Attempt -lt 1 -or [string]::IsNullOrWhiteSpace($EnvironmentFingerprint)) {
    throw "Sprint 8A product diagnostics require the exact rehearsal attempt and environment fingerprint."
}
$baseUri = [Uri]$BaseUrl
if ($baseUri.Scheme -notin @("http", "https") -or $baseUri.Host -notin @("127.0.0.1", "localhost", "::1")) {
    throw "Sprint 8A product diagnostics are restricted to an explicit loopback deployment."
}
$outputFullPath = if ([IO.Path]::IsPathRooted($OutputPath)) {
    [IO.Path]::GetFullPath($OutputPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $OutputPath))
}
if ((Test-Path -LiteralPath $outputFullPath) -or (Test-Path -LiteralPath "$outputFullPath.sha256")) {
    throw "Sprint 8A product-diagnostic evidence already exists and cannot be overwritten: $outputFullPath"
}
$semanticEvidencePath = "$outputFullPath.dashboard-dependencies.json"
if ((Test-Path -LiteralPath $semanticEvidencePath) -or
    (Test-Path -LiteralPath "$semanticEvidencePath.sha256")) {
    throw "Sprint 8A raw Dashboard diagnostic evidence already exists and cannot be overwritten: $semanticEvidencePath"
}
$source = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
if ($source.dirty) { throw "Sprint 8A product diagnostics require clean source." }

$startedAt = [DateTimeOffset]::UtcNow
$dashboardDependencySemantics = $null
$dashboardDependencySemanticsEvidence = $null
$diagnosticChecks = [Collections.Generic.List[object]]::new()
Push-Location $repoRoot
try {
    # The Sprint 8A smoke runner is product/ownership diagnostic coverage. With
    # no OutputPath it publishes no acceptance artifact; the later containment
    # lane destroys semantic-diagnostic writes and proves a canonical successor.
    try {
        $broadJson = (& ./scripts/smoke-sprint-8a.ps1 -BaseUrl $BaseUrl -SupervisorUrl $SupervisorUrl | Out-String).Trim()
        if (-not $?) { throw "Sprint 8A product behavior diagnostic returned a failing exit status." }
        $broadResult = $broadJson | ConvertFrom-Json
        if ($broadResult.passed -ne $true -or @($broadResult.checks).Count -lt 1) {
            throw "Sprint 8A product behavior diagnostic did not return a complete passing check inventory."
        }
        $diagnosticChecks.Add([ordered]@{ name = "broad-product-behavior"; state = "passed"; reason = $null; check_count = @($broadResult.checks).Count })
    } catch {
        $diagnosticChecks.Add([ordered]@{ name = "broad-product-behavior"; state = "failed"; reason = $_.Exception.Message; check_count = 0 })
    }
    try {
        $semanticJson = (& ./scripts/diagnose-sprint-8a-dashboard-dependencies.ps1 `
            -BaseUrl $BaseUrl `
            -OutputPath $semanticEvidencePath | Out-String).Trim()
        if (-not $?) { throw "Sprint 8A Dashboard dependency semantic diagnostic returned a failing exit status." }
        $dashboardDependencySemanticsEvidence = Get-RetainedDiagnosticArtifact -Path $semanticEvidencePath
        $dashboardDependencySemantics = Get-Content -LiteralPath $semanticEvidencePath -Raw | ConvertFrom-Json
        Assert-Sprint8ADashboardDependencyEvidence -Evidence $dashboardDependencySemantics
        $diagnosticChecks.Add([ordered]@{ name = "dashboard-dependency-semantics"; state = "passed"; reason = $null; raw_evidence = $dashboardDependencySemanticsEvidence })
    } catch {
        $semanticFailure = $_.Exception.Message
        if (Test-Path -LiteralPath $semanticEvidencePath -PathType Leaf) {
            try {
                $dashboardDependencySemanticsEvidence = Get-RetainedDiagnosticArtifact -Path $semanticEvidencePath
                $dashboardDependencySemantics = Get-Content -LiteralPath $semanticEvidencePath -Raw | ConvertFrom-Json
            } catch {
                $semanticFailure = "$semanticFailure Raw evidence integrity error: $($_.Exception.Message)"
            }
        }
        $diagnosticChecks.Add([ordered]@{ name = "dashboard-dependency-semantics"; state = "failed"; reason = $semanticFailure; raw_evidence = $dashboardDependencySemanticsEvidence })
    }
} finally {
    Pop-Location
}
$endedAt = [DateTimeOffset]::UtcNow
$diagnosticFailures = @($diagnosticChecks | Where-Object state -CEQ "failed")
if ($diagnosticFailures.Count -gt 0) {
    [ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "candidate-product-diagnostic-harvest"
        attempt = $Attempt
        authoritative = $false
        state = "failed"
        started_at = $startedAt.ToString("o")
        ended_at = $endedAt.ToString("o")
        mutable_source_identity = $source
        environment_fingerprint = $EnvironmentFingerprint
        checks = $diagnosticChecks
        dashboard_dependency_semantics = $dashboardDependencySemantics
        dashboard_dependency_semantics_evidence = $dashboardDependencySemanticsEvidence
        failure_count = $diagnosticFailures.Count
        harvesting_complete = $true
    } | ConvertTo-Json -Depth 30
    throw "Sprint 8A product diagnostics harvested $($diagnosticFailures.Count) failure(s); no passing receipt was published."
}
$receipt = New-Sprint8AProductDiagnosticReceipt `
    -SourceIdentity $source `
    -Fingerprint $EnvironmentFingerprint `
    -AttemptNumber $Attempt `
    -Endpoint $BaseUrl `
    -StartedAt $startedAt.ToString("o") `
    -EndedAt $endedAt.ToString("o") `
    -DashboardDependencySemantics $dashboardDependencySemantics `
    -DashboardDependencySemanticsEvidence $dashboardDependencySemanticsEvidence `
    -DiagnosticChecks $diagnosticChecks
Publish-Sprint7AEvidence -Document $receipt -OutputPath $outputFullPath | Out-Null
Write-Host "Sprint 8A non-acceptance product diagnostics passed; formal UAT was not started."

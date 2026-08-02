[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$SupervisorUrl = "http://127.0.0.1:8096",
    [string]$ScopedEmail = "scoped-sprint7a@tessara.local",
    [string]$ScopedPassword = "tessara-sprint-7a-scoped",
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint7AAcceptanceContract
    $manualRoot = Join-Path $script:Sprint7ARepositoryRoot "docs/sprints/sprint-7a-uat"
    foreach ($index in 1..11) {
        $name = "uat-7a-{0:D2}.md" -f $index
        if (-not (Test-Path -LiteralPath (Join-Path $manualRoot $name) -PathType Leaf)) {
            throw "Manual Sprint 7A UAT script is missing: $name"
        }
    }
    Write-Host "Sprint 7A UAT harness self-test passed."
    return
}
$checks = [Collections.Generic.List[object]]::new()
$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $ScopedEmail -Password $ScopedPassword
foreach ($surface in @(
    [ordered]@{ code = "scoped_dataset"; path = "/api/datasets"; marker = $script:Sprint7AFixture.dataset_id },
    [ordered]@{ code = "scoped_components"; path = "/api/components"; marker = $script:Sprint7AFixture.metric_component_id },
    [ordered]@{ code = "scoped_dashboard"; path = "/api/dashboards/$($script:Sprint7AFixture.dashboard_id)"; marker = $script:Sprint7AFixture.metric_placement_id }
)) {
    $response = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $surface.path -Token $token
    Assert-Sprint7A ($response.status -eq 200 -and $response.body.Contains($surface.marker)) $surface.code "$($surface.path) exposes the authorized reference fixture" $checks
}
$stat = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/dashboards/$($script:Sprint7AFixture.dashboard_id)/placements/$($script:Sprint7AFixture.metric_placement_id)/render/stat-card" -Token $token
Assert-Sprint7A ($stat.status -eq 200 -and $stat.body.Contains('"display_value":"1"')) "scoped_dashboard_execution" "Authorized Dashboard mediation returns the exact stat result" $checks
$logout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token
Assert-Sprint7A ($logout.status -in 200, 204) "session_cleanup" "Scoped acceptance session was explicitly logged out" $checks

$result = [ordered]@{ schema_version = 1; evidence_kind = "tessara.sprint-7a.scripted-uat"; generated_at = [DateTimeOffset]::UtcNow.ToString("o"); base_url = $BaseUrl.TrimEnd('/'); actor = $ScopedEmail; checks = $checks; manual_scripts = 11; passed = $true }
if ([string]::IsNullOrWhiteSpace($OutputPath)) { throw "-OutputPath is required for retained Sprint 7A scripted UAT evidence." }
Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
$result | ConvertTo-Json -Depth 20

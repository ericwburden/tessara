[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composeFile = Join-Path $repoRoot "docker-compose.yml"

if (-not (Test-Path $composeFile)) {
    throw "Could not find docker-compose.yml at $composeFile"
}

Push-Location $repoRoot
try {
    Write-Host "`n==> Seeding UAT demo dataset" -ForegroundColor Cyan
    $json = docker compose exec -T api tessara-api seed-demo | Out-String

    if ($LASTEXITCODE -ne 0) {
        throw "Demo seed failed with exit code $LASTEXITCODE"
    }

    $summary = $json | ConvertFrom-Json

    Write-Host "Seed version: $($summary.seed_version)" -ForegroundColor Green
    Write-Host ("Nodes: {0} partners, {1} programs, {2} activities, {3} sessions" -f `
        $summary.node_counts.partners, `
        $summary.node_counts.programs, `
        $summary.node_counts.activities, `
        $summary.node_counts.sessions)
    Write-Host ("Forms: {0}" -f $summary.form_count)
    Write-Host ("Responses: {0} drafts, {1} submitted" -f `
        $summary.draft_submission_count, `
        $summary.submitted_submission_count)
    Write-Host ("Reports: {0}" -f $summary.report_count)
    Write-Host ("Dashboards: {0}" -f $summary.dashboard_count)
    Write-Host ("Primary demo node:      {0}" -f $summary.organization_node_id)
    Write-Host ("Primary form version:   {0}" -f $summary.form_version_id)
    Write-Host ("Primary submission:     {0}" -f $summary.submission_id)
    Write-Host ("Primary report:         {0}" -f $summary.report_id)
    Write-Host ("Primary dashboard:      {0}" -f $summary.dashboard_id)
} finally {
    Pop-Location
}

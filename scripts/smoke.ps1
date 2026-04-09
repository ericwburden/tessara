param(
    [switch]$KeepServices,
    [switch]$ComposeApi,
    [int]$ApiTimeoutSeconds = 600
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tmpDir = Join-Path $repoRoot "tmp"
$apiOut = Join-Path $tmpDir "tessara-api.out.log"
$apiErr = Join-Path $tmpDir "tessara-api.err.log"
$baseUrl = "http://127.0.0.1:8080"
$apiProcess = $null

function Invoke-Json {
    param(
        [string]$Method,
        [string]$Uri,
        [hashtable]$Headers = @{},
        [object]$Body = $null
    )

    $params = @{
        Method = $Method
        Uri = $Uri
        Headers = $Headers
        TimeoutSec = 30
    }

    if ($null -ne $Body) {
        $params.ContentType = "application/json"
        $params.Body = ($Body | ConvertTo-Json -Depth 20)
    }

    Invoke-RestMethod @params
}

function Assert-LastExitCode {
    param([string]$CommandName)

    if ($LASTEXITCODE -ne 0) {
        throw "$CommandName failed with exit code $LASTEXITCODE"
    }
}

try {
    Set-Location $repoRoot
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
    Remove-Item -LiteralPath $apiOut, $apiErr -ErrorAction SilentlyContinue

    if ($ComposeApi) {
        docker compose up -d --build | Out-Host
        Assert-LastExitCode "docker compose up"
    } else {
        docker compose up -d --wait postgres | Out-Host
        Assert-LastExitCode "docker compose up"
        $composeApiContainer = docker compose ps -q api
        if ($composeApiContainer) {
            docker compose stop api | Out-Null
            docker compose rm -f api | Out-Null
        }
    }

    $postgresDeadline = (Get-Date).AddSeconds(120)
    do {
        docker compose exec -T postgres pg_isready -U tessara -d postgres | Out-Null
        if ($LASTEXITCODE -eq 0) {
            break
        }
        Start-Sleep -Seconds 2
    } while ((Get-Date) -lt $postgresDeadline)

    if ((Get-Date) -ge $postgresDeadline) {
        throw "Timed out waiting for Postgres readiness"
    }

    $env:DATABASE_URL = "postgres://tessara:tessara@localhost:5432/tessara"
    $env:TEST_DATABASE_URL = "postgres://tessara:tessara@localhost:5432/tessara_test"
    $env:TESSARA_BIND_ADDR = "127.0.0.1:8080"
    $env:RUST_LOG = "tessara_api=debug,sqlx=warn"

    foreach ($databaseName in @("tessara", "tessara_test")) {
        $dbExists = docker compose exec -T postgres psql -U tessara -d postgres -tc "SELECT 1 FROM pg_database WHERE datname = '$databaseName'"
        Assert-LastExitCode "checking $databaseName database"
        if (-not ($dbExists | Select-String "1" -Quiet)) {
            docker compose exec -T postgres psql -U tessara -d postgres -c "CREATE DATABASE $databaseName" | Out-Host
            Assert-LastExitCode "creating $databaseName database"
        }
    }

    if (-not $ComposeApi) {
        cargo test -p tessara-api --test demo_flow | Out-Host
        Assert-LastExitCode "cargo test -p tessara-api --test demo_flow"

        $apiProcess = Start-Process `
            -FilePath "cargo" `
            -ArgumentList @("run", "-p", "tessara-api") `
            -WorkingDirectory $repoRoot `
            -NoNewWindow `
            -PassThru `
            -RedirectStandardOutput $apiOut `
            -RedirectStandardError $apiErr
    }

    $deadline = (Get-Date).AddSeconds($ApiTimeoutSeconds)
    do {
        Start-Sleep -Seconds 2
        try {
            $health = Invoke-RestMethod -Uri "$baseUrl/health" -TimeoutSec 3
            if ($health -eq "ok") {
                break
            }
        } catch {
            if ($null -ne $apiProcess -and $apiProcess.HasExited) {
                throw "API exited before becoming healthy. stderr:`n$(Get-Content -Raw -LiteralPath $apiErr -ErrorAction SilentlyContinue)"
            }
        }
    } while ((Get-Date) -lt $deadline)

    if ((Get-Date) -ge $deadline) {
        throw "Timed out waiting for API health. stderr:`n$(Get-Content -Tail 80 -LiteralPath $apiErr -ErrorAction SilentlyContinue)"
    }

    $shell = Invoke-RestMethod -Uri "$baseUrl/" -TimeoutSec 30
    if (-not ($shell -like "*Admin Shell*") -or -not ($shell -like "*Create Draft*") -or -not ($shell -like "*Validate Legacy Fixture*")) {
        throw "Expected local shell HTML to include admin and submission controls"
    }
    if (-not ($shell -like "*Open Application Shell*")) {
        throw "Expected local shell HTML to link to application shell"
    }

    $appShell = Invoke-RestMethod -Uri "$baseUrl/app" -TimeoutSec 30
    if (-not ($appShell -like "*Application Overview*") -or -not ($appShell -like "*Welcome to Tessara*") -or -not ($appShell -like "*Product Areas*") -or -not ($appShell -like "*Internal Areas*") -or -not ($appShell -like "*Start Demo Response*") -or -not ($appShell -like "*Browse Organization*") -or -not ($appShell -like "*Browse Forms*") -or -not ($appShell -like "*Go to Responses*") -or -not ($appShell -like "*Go to Dashboards*")) {
        throw "Expected application home HTML to include overview and split-area navigation"
    }
    $organizationShell = Invoke-RestMethod -Uri "$baseUrl/app/organization" -TimeoutSec 30
    if (-not ($organizationShell -like "*Organization*") -or -not ($organizationShell -like "*Organization Areas*") -or -not ($organizationShell -like "*Organization Workspace*") -or -not ($organizationShell -like "*Load Nodes*")) {
        throw "Expected organization application shell HTML to include organization route controls"
    }
    $formsShell = Invoke-RestMethod -Uri "$baseUrl/app/forms" -TimeoutSec 30
    if (-not ($formsShell -like "*Forms*") -or -not ($formsShell -like "*Forms Areas*") -or -not ($formsShell -like "*Forms Workspace*") -or -not ($formsShell -like "*Load Forms*")) {
        throw "Expected forms application shell HTML to include forms route controls"
    }
    $responsesShell = Invoke-RestMethod -Uri "$baseUrl/app/responses" -TimeoutSec 30
    if (-not ($responsesShell -like "*Responses*") -or -not ($responsesShell -like "*Responses Workspace*") -or -not ($responsesShell -like "*Response Directory*") -or -not ($responsesShell -like "*Refresh Responses*")) {
        throw "Expected responses application shell HTML to include responses route controls"
    }
    $submissionAppShell = Invoke-RestMethod -Uri "$baseUrl/app/submissions" -TimeoutSec 30
    if (-not ($submissionAppShell -like "*Responses*") -or -not ($submissionAppShell -like "*Response Stages*") -or -not ($submissionAppShell -like "*Response Directory*") -or -not ($submissionAppShell -like "*Browse Published Forms*") -or -not ($submissionAppShell -like "*Response Review*") -or -not ($submissionAppShell -like "*Refresh Summary*") -or -not ($submissionAppShell -like "*Refresh Responses*") -or -not ($submissionAppShell -like "*Session Status*") -or -not ($submissionAppShell -like "*Sign Out*")) {
        throw "Expected responses compatibility shell HTML to include response workflow controls"
    }
    $administrationShell = Invoke-RestMethod -Uri "$baseUrl/app/administration" -TimeoutSec 30
    if (-not ($administrationShell -like "*Administration*") -or -not ($administrationShell -like "*Management Areas*") -or -not ($administrationShell -like "*Entity Directory*") -or -not ($administrationShell -like "*Inspect Form*") -or -not ($administrationShell -like "*Inspect Node Type*")) {
        throw "Expected administration application shell HTML to include setup workflow controls"
    }
    $adminAppShell = Invoke-RestMethod -Uri "$baseUrl/app/admin" -TimeoutSec 30
    if (-not ($adminAppShell -like "*Administration*") -or -not ($adminAppShell -like "*Management Areas*") -or -not ($adminAppShell -like "*Entity Directory*") -or -not ($adminAppShell -like "*Organization Setup*") -or -not ($adminAppShell -like "*Forms Configuration*") -or -not ($adminAppShell -like "*Inspect Form*") -or -not ($adminAppShell -like "*Inspect Node Type*")) {
        throw "Expected admin application shell HTML to include setup workflow controls"
    }
    $reportingAppShell = Invoke-RestMethod -Uri "$baseUrl/app/reports" -TimeoutSec 30
    if (-not ($reportingAppShell -like "*Reports*") -or -not ($reportingAppShell -like "*Report Areas*") -or -not ($reportingAppShell -like "*Reporting Directory*") -or -not ($reportingAppShell -like "*Reports Workspace*") -or -not ($reportingAppShell -like "*Browse Datasets*") -or -not ($reportingAppShell -like "*Inspect Dataset*") -or -not ($reportingAppShell -like "*View Dataset Rows*") -or -not ($reportingAppShell -like "*Dashboard Preview*") -or -not ($reportingAppShell -like "*Inspect Chart*") -or -not ($reportingAppShell -like "*Review Reports*") -or -not ($reportingAppShell -like "*Refresh Reports*")) {
        throw "Expected reporting application shell HTML to include report and dashboard workflow controls"
    }
    if ($reportingAppShell -like "*Create Shortcuts*") {
        throw "Expected reports application shell HTML to keep create shortcuts in internal areas"
    }
    $dashboardsShell = Invoke-RestMethod -Uri "$baseUrl/app/dashboards" -TimeoutSec 30
    if (-not ($dashboardsShell -like "*Dashboards*") -or -not ($dashboardsShell -like "*Dashboard Areas*") -or -not ($dashboardsShell -like "*Dashboards Workspace*") -or -not ($dashboardsShell -like "*Dashboard Preview*")) {
        throw "Expected dashboards application shell HTML to include dashboard route controls"
    }
    $migrationAppShell = Invoke-RestMethod -Uri "$baseUrl/app/migration" -TimeoutSec 30
    if (-not ($migrationAppShell -like "*Migration Workbench*") -or -not ($migrationAppShell -like "*Migration Stages*") -or -not ($migrationAppShell -like "*Migration Directory*") -or -not ($migrationAppShell -like "*Fixture Intake and Validation*") -or -not ($migrationAppShell -like "*Load Fixture Examples*") -or -not ($migrationAppShell -like "*Import Fixture*")) {
        throw "Expected migration application shell HTML to include fixture workflow controls"
    }
    if ($migrationAppShell -like "*Create Shortcuts*") {
        throw "Expected migration application shell HTML to keep create shortcuts in internal areas"
    }

    $login = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "admin@tessara.local"; password = "tessara-dev-admin" }
    $headers = @{ Authorization = "Bearer $($login.token)" }

    $seed = Invoke-Json -Method "Post" -Uri "$baseUrl/api/demo/seed" -Headers $headers
    $summary = Invoke-Json -Method "Get" -Uri "$baseUrl/api/app/summary" -Headers $headers
    if ($summary.published_form_versions -lt 1 -or $summary.submitted_submissions -lt 1 -or $summary.reports -lt 1 -or $summary.dashboards -lt 1) {
        throw "Expected application summary to include seeded published forms, submissions, reports, and dashboards"
    }
    $nodes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/nodes"
    $dashboard = Invoke-Json -Method "Get" -Uri "$baseUrl/api/dashboards/$($seed.dashboard_id)"
    $report = Invoke-Json -Method "Get" -Uri "$baseUrl/api/reports/$($seed.report_id)/table" -Headers $headers

    if ($seed.analytics_values -lt 1) {
        throw "Expected at least one analytics value, got $($seed.analytics_values)"
    }
    if ($nodes.Count -lt 1) {
        throw "Expected at least one node, got $($nodes.Count)"
    }
    if ($dashboard.components.Count -lt 1) {
        throw "Expected at least one dashboard component, got $($dashboard.components.Count)"
    }
    if ($report.rows.Count -lt 1 -or $report.rows[0].field_value -ne "42") {
        throw "Expected report value 42, got: $($report | ConvertTo-Json -Depth 20)"
    }

    [pscustomobject]@{
        status = "passed"
        organization_node_id = $seed.organization_node_id
        dashboard_id = $seed.dashboard_id
        report_rows = $report.rows.Count
        first_report_value = $report.rows[0].field_value
    } | ConvertTo-Json -Depth 10
}
finally {
    if ($null -ne $apiProcess -and -not $apiProcess.HasExited) {
        Stop-Process -Id $apiProcess.Id -Force
    }

    if (-not $KeepServices) {
        docker compose down -v | Out-Host
        Assert-LastExitCode "docker compose down"
    }
}

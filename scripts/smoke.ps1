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
        docker compose down --remove-orphans | Out-Host
        Assert-LastExitCode "docker compose down"
        docker compose up -d --build | Out-Host
        Assert-LastExitCode "docker compose up"
    } else {
        docker compose down --remove-orphans | Out-Host
        Assert-LastExitCode "docker compose down"
        docker compose up -d --wait postgres | Out-Host
        Assert-LastExitCode "docker compose up"
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
    if (-not ($appShell -like "*Application Overview*") -or -not ($appShell -like "*Welcome to Tessara*") -or -not ($appShell -like "*Role-Ready Home Modules*") -or -not ($appShell -like "*Product Areas*") -or -not ($appShell -like "*Current Deployment Readiness*") -or -not ($appShell -like "*Current Workflow Context*") -or -not ($appShell -like "*Internal Areas*") -or -not ($appShell -like "*Go to Organization*") -or -not ($appShell -like "*Go to Forms*") -or -not ($appShell -like "*Go to Responses*")) {
        throw "Expected application home HTML to include overview and split-area navigation"
    }
    $loginShell = Invoke-RestMethod -Uri "$baseUrl/app/login" -TimeoutSec 30
    if (-not ($loginShell -like "*Sign In*") -or -not ($loginShell -like "*operator@tessara.local*") -or -not ($loginShell -like "*delegator@tessara.local*") -or -not ($loginShell -like "*delegate@tessara.local*")) {
        throw "Expected login HTML to include dedicated sign-in controls and demo credentials"
    }
    $organizationShell = Invoke-RestMethod -Uri "$baseUrl/app/organization" -TimeoutSec 30
    if (-not ($organizationShell -like "*Organizations*") -or -not ($organizationShell -like "*Create Organization*") -or -not ($organizationShell -like "*organization-list*")) {
        throw "Expected organization application shell HTML to include organization route controls"
    }
    $formsShell = Invoke-RestMethod -Uri "$baseUrl/app/forms" -TimeoutSec 30
    if (-not ($formsShell -like "*Forms*") -or -not ($formsShell -like "*Create Form*") -or -not ($formsShell -like "*form-list*")) {
        throw "Expected forms application shell HTML to include forms route controls"
    }
    $responsesShell = Invoke-RestMethod -Uri "$baseUrl/app/responses" -TimeoutSec 30
    if (-not ($responsesShell -like "*Responses*") -or -not ($responsesShell -like "*Start Response*") -or -not ($responsesShell -like "*Start New Response*") -or -not ($responsesShell -like "*Draft Responses*") -or -not ($responsesShell -like "*Submitted Responses*")) {
        throw "Expected responses application shell HTML to include responses route controls"
    }
    $submissionAppShell = Invoke-RestMethod -Uri "$baseUrl/app/submissions" -TimeoutSec 30
    if (-not ($submissionAppShell -like "*Responses*") -or -not ($submissionAppShell -like "*Start Response*") -or -not ($submissionAppShell -like "*Draft Responses*") -or -not ($submissionAppShell -like "*Submitted Responses*")) {
        throw "Expected responses compatibility shell HTML to include response workflow controls"
    }
    $administrationShell = Invoke-RestMethod -Uri "$baseUrl/app/administration" -TimeoutSec 30
    if (-not ($administrationShell -like "*Administration*") -or -not ($administrationShell -like "*Advanced Configuration*") -or -not ($administrationShell -like "*Open Legacy Builder*")) {
        throw "Expected administration application shell HTML to include setup workflow controls"
    }
    $adminAppShell = Invoke-RestMethod -Uri "$baseUrl/app/admin" -TimeoutSec 30
    if (-not ($adminAppShell -like "*Admin Shell*") -or -not ($adminAppShell -like "*Create Form*") -or -not ($adminAppShell -like "*Validate Legacy Fixture*")) {
        throw "Expected admin application shell HTML to include setup workflow controls"
    }
    $reportingAppShell = Invoke-RestMethod -Uri "$baseUrl/app/reports" -TimeoutSec 30
    if (-not ($reportingAppShell -like "*Reports*") -or -not ($reportingAppShell -like "*Create Report*") -or -not ($reportingAppShell -like "*report-list*")) {
        throw "Expected reporting application shell HTML to include report and dashboard workflow controls"
    }
    $dashboardsShell = Invoke-RestMethod -Uri "$baseUrl/app/dashboards" -TimeoutSec 30
    if (-not ($dashboardsShell -like "*Dashboards*") -or -not ($dashboardsShell -like "*Create Dashboard*") -or -not ($dashboardsShell -like "*dashboard-list*")) {
        throw "Expected dashboards application shell HTML to include dashboard route controls"
    }
    $migrationAppShell = Invoke-RestMethod -Uri "$baseUrl/app/migration" -TimeoutSec 30
    if (-not ($migrationAppShell -like "*Migration Workbench*") -or -not ($migrationAppShell -like "*Fixture Intake*") -or -not ($migrationAppShell -like "*Load Fixture Examples*") -or -not ($migrationAppShell -like "*Import Fixture*")) {
        throw "Expected migration application shell HTML to include fixture workflow controls"
    }

    $login = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "admin@tessara.local"; password = "tessara-dev-admin" }
    $headers = @{ Authorization = "Bearer $($login.token)" }
    $seed = Invoke-Json -Method "Post" -Uri "$baseUrl/api/demo/seed" -Headers $headers
    $summary = Invoke-Json -Method "Get" -Uri "$baseUrl/api/app/summary" -Headers $headers
    $operatorLogin = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "operator@tessara.local"; password = "tessara-dev-operator" }
    $operatorHeaders = @{ Authorization = "Bearer $($operatorLogin.token)" }
    $respondentLogin = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "respondent@tessara.local"; password = "tessara-dev-respondent" }
    $respondentHeaders = @{ Authorization = "Bearer $($respondentLogin.token)" }
    $delegatorLogin = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "delegator@tessara.local"; password = "tessara-dev-delegator" }
    $delegatorHeaders = @{ Authorization = "Bearer $($delegatorLogin.token)" }
    if ($summary.published_form_versions -lt 1 -or $summary.submitted_submissions -lt 1 -or $summary.reports -lt 1 -or $summary.dashboards -lt 1) {
        throw "Expected application summary to include seeded published forms, submissions, reports, and dashboards"
    }
    $nodes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/nodes" -Headers $headers
    $dashboard = Invoke-Json -Method "Get" -Uri "$baseUrl/api/dashboards/$($seed.dashboard_id)" -Headers $headers
    $report = Invoke-Json -Method "Get" -Uri "$baseUrl/api/reports/$($seed.report_id)/table" -Headers $headers
    $operatorMe = Invoke-Json -Method "Get" -Uri "$baseUrl/api/me" -Headers $operatorHeaders
    $operatorNodes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/nodes?q=Demo" -Headers $operatorHeaders
    $respondentOptions = Invoke-Json -Method "Get" -Uri "$baseUrl/api/responses/options" -Headers $respondentHeaders
    $delegatorMe = Invoke-Json -Method "Get" -Uri "$baseUrl/api/me" -Headers $delegatorHeaders
    $delegateAccountId = $delegatorMe.delegations[0].account_id
    $delegatedOptions = Invoke-Json -Method "Get" -Uri "$baseUrl/api/responses/options?delegate_account_id=$delegateAccountId" -Headers $delegatorHeaders
    $respondentFormsDenied = $false
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/forms" -Headers $respondentHeaders | Out-Null
    } catch {
        $respondentFormsDenied = $_.Exception.Message -like "*403*"
    }

    if ($seed.analytics_values -lt 1) {
        throw "Expected at least one analytics value, got $($seed.analytics_values)"
    }
    if ($nodes.Count -lt 1) {
        throw "Expected at least one node, got $($nodes.Count)"
    }
    if ($dashboard.components.Count -lt 1) {
        throw "Expected at least one dashboard component, got $($dashboard.components.Count)"
    }
    if ($report.rows.Count -lt 1 -or -not ($report.rows | Where-Object { $_.logical_key -eq "participants" -and $_.field_value -eq "42" })) {
        throw "Expected report value 42, got: $($report | ConvertTo-Json -Depth 20)"
    }
    if ($operatorMe.ui_access_profile -ne "operator" -or $operatorMe.scope_nodes.Count -lt 1) {
        throw "Expected operator account context to include operator UI access profile and scoped nodes"
    }
    if (-not ($operatorNodes | Where-Object { $_.name -eq "Demo Program Family Outreach" }) -or ($operatorNodes | Where-Object { $_.name -eq "Demo Partner Community Bridge" })) {
        throw "Expected operator node list to stay within assigned scope"
    }
    if ($respondentOptions.mode -ne "assignment" -or $respondentOptions.assignments.Count -lt 1) {
        throw "Expected respondent response options to return assigned response starts"
    }
    if ($delegatedOptions.mode -ne "assignment" -or $delegatedOptions.assignments.Count -lt 1) {
        throw "Expected delegated response options to support delegated response context"
    }
    if (-not $respondentFormsDenied) {
        throw "Expected respondent access to /api/forms to be forbidden"
    }

    $organizationDetail = Invoke-RestMethod -Uri "$baseUrl/app/organization/$($seed.organization_node_id)" -TimeoutSec 30
    if (-not ($organizationDetail -like "*Organization Detail*") -or -not ($organizationDetail -like "*Back to List*") -or -not ($organizationDetail -like "*organization-detail*")) {
        throw "Expected organization detail HTML to include dedicated detail framing"
    }
    $organizationNew = Invoke-RestMethod -Uri "$baseUrl/app/organization/new" -TimeoutSec 30
    if (-not ($organizationNew -like "*Create Organization*") -or -not ($organizationNew -like "*Submit*") -or -not ($organizationNew -like "*Cancel*")) {
        throw "Expected organization create HTML to include dedicated form controls"
    }
    $formDetail = Invoke-RestMethod -Uri "$baseUrl/app/forms/$($seed.form_id)" -TimeoutSec 30
    if (-not ($formDetail -like "*Form Detail*") -or -not ($formDetail -like "*Back to List*")) {
        throw "Expected form detail HTML to include dedicated detail framing"
    }
    $formNew = Invoke-RestMethod -Uri "$baseUrl/app/forms/new" -TimeoutSec 30
    if (-not ($formNew -like "*Create Form*") -or -not ($formNew -like "*Submit*") -or -not ($formNew -like "*Cancel*")) {
        throw "Expected form create HTML to include dedicated form controls"
    }
    $responseDetail = Invoke-RestMethod -Uri "$baseUrl/app/responses/$($seed.submission_id)" -TimeoutSec 30
    if (-not ($responseDetail -like "*Response Detail*") -or -not ($responseDetail -like "*Back to List*")) {
        throw "Expected response detail HTML to include dedicated detail framing"
    }
    $responseNew = Invoke-RestMethod -Uri "$baseUrl/app/responses/new" -TimeoutSec 30
    if (-not ($responseNew -like "*Start Response*") -or -not ($responseNew -like "*Submit*") -or -not ($responseNew -like "*Cancel*")) {
        throw "Expected response create HTML to include dedicated form controls"
    }
    $reportDetailPage = Invoke-RestMethod -Uri "$baseUrl/app/reports/$($seed.report_id)" -TimeoutSec 30
    if (-not ($reportDetailPage -like "*Report Detail*") -or -not ($reportDetailPage -like "*Run*")) {
        throw "Expected report detail HTML to include dedicated detail framing"
    }
    $reportNew = Invoke-RestMethod -Uri "$baseUrl/app/reports/new" -TimeoutSec 30
    if (-not ($reportNew -like "*Create Report*") -or -not ($reportNew -like "*Bindings*") -or -not ($reportNew -like "*Submit*")) {
        throw "Expected report create HTML to include binding editor controls"
    }
    $dashboardDetailPage = Invoke-RestMethod -Uri "$baseUrl/app/dashboards/$($seed.dashboard_id)" -TimeoutSec 30
    if (-not ($dashboardDetailPage -like "*Dashboard Detail*") -or -not ($dashboardDetailPage -like "*View*")) {
        throw "Expected dashboard detail HTML to include dedicated detail framing"
    }
    $dashboardNew = Invoke-RestMethod -Uri "$baseUrl/app/dashboards/new" -TimeoutSec 30
    if (-not ($dashboardNew -like "*Create Dashboard*") -or -not ($dashboardNew -like "*Submit*") -or -not ($dashboardNew -like "*Cancel*")) {
        throw "Expected dashboard create HTML to include dedicated form controls"
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

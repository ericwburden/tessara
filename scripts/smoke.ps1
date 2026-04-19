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
$cargoCommand = $null

function Resolve-CargoCommand {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargo) {
        return $cargo.Source
    }

    $defaultCargo = Join-Path $HOME ".cargo\bin\cargo.exe"
    if (Test-Path $defaultCargo) {
        return $defaultCargo
    }

    throw "Unable to locate cargo. Add cargo to PATH or install it under $HOME\\.cargo\\bin."
}

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

function Invoke-Html {
    param(
        [string]$Uri,
        [string]$CookieJarPath = $null
    )

    $arguments = @("-sS", "-f")
    if ($null -ne $CookieJarPath) {
        $arguments += @("-b", $CookieJarPath)
    }
    $arguments += $Uri

    $content = & curl.exe @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "curl failed while fetching $Uri with exit code $LASTEXITCODE"
    }

    $content
}

function Assert-ProtectedShell {
    param(
        [string]$Content,
        [string[]]$Needles,
        [string]$Context
    )

    foreach ($needle in @(
        "top-app-bar",
        "global-search",
        "Loading Session",
        "Loading session state..."
    ) + $Needles) {
        if ($Content -notlike "*$needle*") {
            throw "Smoke failure in $Context. Missing marker: $needle"
        }
    }
}

function New-BrowserSession {
    param(
        [string]$Email,
        [string]$Password
    )

    $cookieJar = Join-Path $tmpDir ("browser-" + [guid]::NewGuid().ToString() + ".txt")
    $payloadPath = Join-Path $tmpDir ("browser-login-" + [guid]::NewGuid().ToString() + ".json")
    $loginBody = @{
        email = $Email
        password = $Password
    } | ConvertTo-Json

    [System.IO.File]::WriteAllText($payloadPath, $loginBody, [System.Text.UTF8Encoding]::new($false))

    $response = & curl.exe `
        -sS `
        -f `
        -c $cookieJar `
        -H "Content-Type: application/json" `
        --data-binary ("@" + $payloadPath) `
        "$baseUrl/api/auth/login"
    if ($LASTEXITCODE -ne 0) {
        throw "curl login failed for $Email with exit code $LASTEXITCODE"
    }

    if (-not $response) {
        throw "Login response for $Email was empty."
    }

    return $cookieJar
}

function Assert-LastExitCode {
    param([string]$CommandName)

    if ($LASTEXITCODE -ne 0) {
        throw "$CommandName failed with exit code $LASTEXITCODE"
    }
}

function Start-ComposeWithRetry {
    param(
        [string[]]$Arguments,
        [string]$CommandName,
        [int]$Attempts = 3
    )

    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        docker compose @Arguments | Out-Host
        if ($LASTEXITCODE -eq 0) {
            return
        }

        if ($attempt -eq $Attempts) {
            throw "$CommandName failed with exit code $LASTEXITCODE"
        }

        Start-Sleep -Seconds 3
        docker compose down -v | Out-Null
    }
}

function Invoke-PostgresSqlWithRetry {
    param(
        [string]$Sql,
        [string]$Database = "postgres",
        [int]$Attempts = 10
    )

    $previousNativePreference = $PSNativeCommandUseErrorActionPreference
    $PSNativeCommandUseErrorActionPreference = $false

    try {
    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        $result = docker compose exec -T postgres psql -U tessara -d $Database -tc $Sql 2>&1
        if ($LASTEXITCODE -eq 0) {
            return $result
        }

        if ($attempt -eq $Attempts) {
            throw "psql command failed after $Attempts attempts: $result"
        }

        Start-Sleep -Seconds 2
    }
    } finally {
        $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    }
}

try {
    Set-Location $repoRoot
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
    Remove-Item -LiteralPath $apiOut, $apiErr -ErrorAction SilentlyContinue
    $cargoCommand = Resolve-CargoCommand

    if ($ComposeApi) {
        docker compose down --remove-orphans | Out-Host
        Assert-LastExitCode "docker compose down"
        Start-ComposeWithRetry -Arguments @("up", "-d", "--build") -CommandName "docker compose up"
    } else {
        docker compose down --remove-orphans | Out-Host
        Assert-LastExitCode "docker compose down"
        Start-ComposeWithRetry -Arguments @("up", "-d", "--wait", "postgres") -CommandName "docker compose up"
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
        $dbExists = Invoke-PostgresSqlWithRetry "SELECT 1 FROM pg_database WHERE datname = '$databaseName'"
        if (-not ($dbExists | Select-String "1" -Quiet)) {
            $null = Invoke-PostgresSqlWithRetry "CREATE DATABASE $databaseName"
        }
    }

    if (-not $ComposeApi) {
        & $cargoCommand test -p tessara-api --test demo_flow | Out-Host
        Assert-LastExitCode "cargo test -p tessara-api --test demo_flow"

        $apiProcess = Start-Process `
            -FilePath $cargoCommand `
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

    $adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"

    $shell = Invoke-Html -Uri "$baseUrl/"
    if (-not ($shell -like "*Admin Shell*") -or -not ($shell -like "*Create Draft*") -or -not ($shell -like "*Validate Legacy Fixture*")) {
        throw "Expected local shell HTML to include admin and submission controls"
    }
    if (-not ($shell -like "*Open Application Shell*")) {
        throw "Expected local shell HTML to link to application shell"
    }

    $appShell = Invoke-Html -Uri "$baseUrl/app" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $appShell -Needles @("Tessara Home", "Product Areas", "Product navigation") -Context "application home shell"
    $loginShell = Invoke-Html -Uri "$baseUrl/app/login"
    if (
        -not ($loginShell -like "*Sign In*") `
        -or -not ($loginShell -like "*login-form*") `
        -or -not ($loginShell -like "*login-email*") `
        -or -not ($loginShell -like "*login-password*") `
        -or -not ($loginShell -like "*Cookie session contract*") `
        -or ($loginShell -like "*operator@tessara.local*")
    ) {
        throw "Expected login HTML to expose native sign-in controls without public demo credentials"
    }
    $organizationShell = Invoke-Html -Uri "$baseUrl/app/organization" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $organizationShell -Needles @("Organization") -Context "organization list shell"
    $formsShell = Invoke-Html -Uri "$baseUrl/app/forms" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formsShell -Needles @("Forms") -Context "forms list shell"
    $workflowsShell = Invoke-Html -Uri "$baseUrl/app/workflows" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $workflowsShell -Needles @("Workflows") -Context "workflows list shell"
    $responsesShell = Invoke-Html -Uri "$baseUrl/app/responses" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $responsesShell -Needles @("Responses") -Context "responses list shell"
    $submissionAppShell = Invoke-Html -Uri "$baseUrl/app/submissions" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $submissionAppShell -Needles @("Responses") -Context "submissions compatibility shell"
    $administrationShell = Invoke-Html -Uri "$baseUrl/app/administration" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $administrationShell -Needles @("Administration") -Context "administration shell"
    $nodeTypesShell = Invoke-Html -Uri "$baseUrl/app/administration/node-types" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $nodeTypesShell -Needles @("Organization Node Types") -Context "node type list shell"
    $nodeTypeCreateShell = Invoke-Html -Uri "$baseUrl/app/administration/node-types/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $nodeTypeCreateShell -Needles @("Create Organization Node Type") -Context "node type create shell"
    $adminAppShell = Invoke-Html -Uri "$baseUrl/app/admin" -CookieJarPath $adminBrowserSession
    if (-not ($adminAppShell -like "*<!doctype html>*") -or -not ($adminAppShell -like "*<body*")) {
        throw "Expected /app/admin to remain a reachable HTML surface"
    }
    $reportingAppShell = Invoke-Html -Uri "$baseUrl/app/reports" -CookieJarPath $adminBrowserSession
    if (-not ($reportingAppShell -like "*Reports*") -or -not ($reportingAppShell -like "*Create Report*") -or -not ($reportingAppShell -like "*report-list*")) {
        throw "Expected reporting application shell HTML to include report and dashboard workflow controls"
    }
    $dashboardsShell = Invoke-Html -Uri "$baseUrl/app/dashboards" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardsShell -Needles @("Dashboards") -Context "dashboards shell"
    $migrationAppShell = Invoke-Html -Uri "$baseUrl/app/migration" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $migrationAppShell -Needles @("Migration Workbench") -Context "migration shell"

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
    $respondentPending = Invoke-Json -Method "Get" -Uri "$baseUrl/api/workflow-assignments/pending" -Headers $respondentHeaders
    $delegatorMe = Invoke-Json -Method "Get" -Uri "$baseUrl/api/me" -Headers $delegatorHeaders
    $delegateAccountId = $delegatorMe.delegations[0].account_id
    $delegatedOptions = Invoke-Json -Method "Get" -Uri "$baseUrl/api/responses/options?delegate_account_id=$delegateAccountId" -Headers $delegatorHeaders
    $respondentFormsDenied = $false
    $operatorNodeTypeAdminDenied = $false
    $respondentNodeTypeAdminDenied = $false
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/forms" -Headers $respondentHeaders | Out-Null
    } catch {
        $respondentFormsDenied = $_.Exception.Message -like "*403*"
    }
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/node-types" -Headers $operatorHeaders | Out-Null
    } catch {
        $operatorNodeTypeAdminDenied = $_.Exception.Message -like "*403*"
    }
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/node-types" -Headers $respondentHeaders | Out-Null
    } catch {
        $respondentNodeTypeAdminDenied = $_.Exception.Message -like "*403*"
    }
    $readableNodeTypes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/node-types" -Headers $headers

    if ($seed.analytics_values -lt 1) {
        throw "Expected at least one analytics value, got $($seed.analytics_values)"
    }
    $adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"
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
    if ($respondentPending.Count -lt 1) {
        throw "Expected respondent pending workflow assignments to return assigned start work"
    }
    if ($delegatedOptions.mode -ne "assignment" -or $delegatedOptions.assignments.Count -lt 1) {
        throw "Expected delegated response options to support delegated response context"
    }
    if (-not $respondentFormsDenied) {
        throw "Expected respondent access to /api/forms to be forbidden"
    }
    if (-not $operatorNodeTypeAdminDenied -or -not $respondentNodeTypeAdminDenied) {
        throw "Expected operator and respondent access to /api/admin/node-types to be forbidden"
    }
    if ($readableNodeTypes.Count -lt 1 -or -not ($readableNodeTypes | Where-Object { $_.singular_label -and $_.plural_label })) {
        throw "Expected readable node-type catalog to include singular/plural labels"
    }

    $organizationDetail = Invoke-Html -Uri "$baseUrl/app/organization/$($seed.organization_node_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $organizationDetail -Needles @("Organization Detail") -Context "organization detail shell"
    $organizationNew = Invoke-Html -Uri "$baseUrl/app/organization/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $organizationNew -Needles @("Create Organization") -Context "organization create shell"
    $formDetail = Invoke-Html -Uri "$baseUrl/app/forms/$($seed.form_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formDetail -Needles @("Form Detail") -Context "form detail shell"
    $formNew = Invoke-Html -Uri "$baseUrl/app/forms/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formNew -Needles @("Create Form") -Context "form create shell"
    $formEdit = Invoke-Html -Uri "$baseUrl/app/forms/$($seed.form_id)/edit" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formEdit -Needles @("Edit Form") -Context "form edit shell"
    $responseDetail = Invoke-Html -Uri "$baseUrl/app/responses/$($seed.submission_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $responseDetail -Needles @("Response Detail") -Context "response detail shell"
    $responseNew = Invoke-Html -Uri "$baseUrl/app/responses/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $responseNew -Needles @("New Response") -Context "response create shell"
    $reportDetailPage = Invoke-Html -Uri "$baseUrl/app/reports/$($seed.report_id)" -CookieJarPath $adminBrowserSession
    if (-not ($reportDetailPage -like "*Report Detail*") -or -not ($reportDetailPage -like "*Run*")) {
        throw "Expected report detail HTML to include dedicated detail framing"
    }
    $reportNew = Invoke-Html -Uri "$baseUrl/app/reports/new" -CookieJarPath $adminBrowserSession
    if (-not ($reportNew -like "*Create Report*") -or -not ($reportNew -like "*Bindings*") -or -not ($reportNew -like "*Submit*")) {
        throw "Expected report create HTML to include binding editor controls"
    }
    $dashboardDetailPage = Invoke-Html -Uri "$baseUrl/app/dashboards/$($seed.dashboard_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardDetailPage -Needles @("Dashboard Detail") -Context "dashboard detail shell"
    $dashboardNew = Invoke-Html -Uri "$baseUrl/app/dashboards/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardNew -Needles @("Create Dashboard") -Context "dashboard create shell"

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

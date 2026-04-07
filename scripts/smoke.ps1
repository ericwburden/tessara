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
    if (-not ($appShell -like "*Submission Workspace*") -or -not ($appShell -like "*Choose Published Form*") -or -not ($appShell -like "*Review Submissions*")) {
        throw "Expected application shell HTML to include submission workflow controls"
    }

    $login = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "admin@tessara.local"; password = "tessara-dev-admin" }
    $headers = @{ Authorization = "Bearer $($login.token)" }

    $seed = Invoke-Json -Method "Post" -Uri "$baseUrl/api/demo/seed" -Headers $headers
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

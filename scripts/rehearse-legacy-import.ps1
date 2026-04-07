param(
    [string]$FixturePath = ".\fixtures\legacy-rehearsal.json",
    [switch]$KeepServices,
    [int]$ApiTimeoutSeconds = 600
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$repoRoot = Split-Path -Parent $PSScriptRoot
$tmpDir = Join-Path $repoRoot "tmp"
$apiOut = Join-Path $tmpDir "tessara-import-api.out.log"
$apiErr = Join-Path $tmpDir "tessara-import-api.err.log"
$baseUrl = "http://127.0.0.1:8081"
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

    $resolvedFixture = Resolve-Path -LiteralPath $FixturePath
    $validationJson = cargo run -q -p tessara-api -- validate-legacy-fixture $resolvedFixture.Path
    Assert-LastExitCode "cargo run validate-legacy-fixture"
    $validation = $validationJson | ConvertFrom-Json

    if ($validation.issue_count -ne 0) {
        throw "Legacy fixture validation failed before import: $validationJson"
    }

    $dryRunJson = cargo run -q -p tessara-api -- dry-run-legacy-fixture $resolvedFixture.Path
    Assert-LastExitCode "cargo run dry-run-legacy-fixture"
    $dryRun = $dryRunJson | ConvertFrom-Json

    if (-not $dryRun.would_import) {
        throw "Legacy fixture dry run reported it would not import: $dryRunJson"
    }

    docker compose up -d --wait postgres | Out-Host
    Assert-LastExitCode "docker compose up"

    $databaseName = "tessara_import_rehearsal"
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

    $dbExists = docker compose exec -T postgres psql -U tessara -d postgres -tc "SELECT 1 FROM pg_database WHERE datname = '$databaseName'"
    Assert-LastExitCode "checking $databaseName database"

    if (-not ($dbExists | Select-String "1" -Quiet)) {
        docker compose exec -T postgres psql -U tessara -d postgres -c "CREATE DATABASE $databaseName" | Out-Host
        Assert-LastExitCode "creating $databaseName database"
    }

    $env:DATABASE_URL = "postgres://tessara:tessara@localhost:5432/$databaseName"
    $env:TESSARA_BIND_ADDR = "127.0.0.1:8081"
    $env:RUST_LOG = "error"

    $summaryJson = cargo run -q -p tessara-api -- import-legacy-fixture $resolvedFixture.Path
    Assert-LastExitCode "cargo run import-legacy-fixture"
    $summary = $summaryJson | ConvertFrom-Json

    if ($summary.analytics_values -lt 1) {
        throw "Expected imported analytics values, got $($summary.analytics_values)"
    }

    $env:RUST_LOG = "tessara_api=debug,sqlx=warn"

    $apiProcess = Start-Process `
        -FilePath "cargo" `
        -ArgumentList @("run", "-p", "tessara-api") `
        -WorkingDirectory $repoRoot `
        -NoNewWindow `
        -PassThru `
        -RedirectStandardOutput $apiOut `
        -RedirectStandardError $apiErr

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

    $login = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "admin@tessara.local"; password = "tessara-dev-admin" }
    $headers = @{ Authorization = "Bearer $($login.token)" }

    $dashboard = Invoke-Json -Method "Get" -Uri "$baseUrl/api/dashboards/$($summary.dashboard_id)"
    $report = Invoke-Json -Method "Get" -Uri "$baseUrl/api/reports/$($summary.report_id)/table" -Headers $headers

    if ($dashboard.components.Count -lt 1) {
        throw "Expected at least one imported dashboard component"
    }
    if ($report.rows.Count -lt 1 -or $report.rows[0].field_value -ne "42") {
        throw "Expected imported report value 42, got: $($report | ConvertTo-Json -Depth 20)"
    }

    [pscustomobject]@{
        status = "passed"
        fixture_name = $summary.fixture_name
        database = $databaseName
        submission_id = $summary.submission_id
        dashboard_id = $summary.dashboard_id
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

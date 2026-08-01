[CmdletBinding()]
param(
    [switch]$FreshData,
    [switch]$FollowLogs,
    [switch]$ApiOnly,
    [switch]$SkipBuild,
    [switch]$SkipSeed,
    [string]$ExternalDatabaseUrl,
    [string]$ExternalDatabaseContainerId,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composeFile = Join-Path $repoRoot "docker-compose.yml"
$seedScript = Join-Path $PSScriptRoot "seed-demo-data.ps1"
$refreshScript = Join-Path $PSScriptRoot "local-refresh-api.ps1"
$hadCallerDatabaseOverride = Test-Path Env:TESSARA_DATABASE_URL
$callerDatabaseOverride = if ($hadCallerDatabaseOverride) { $env:TESSARA_DATABASE_URL } else { $null }
$callerImageBuildEnvironment = @{}
foreach ($name in @("TESSARA_SOURCE_COMMIT", "TESSARA_SOURCE_TREE", "TESSARA_SOURCE_DIRTY")) {
    $present = Test-Path "Env:$name"
    $callerImageBuildEnvironment[$name] = [pscustomobject]@{
        Present = $present
        Value = if ($present) { [Environment]::GetEnvironmentVariable($name) } else { $null }
    }
}

if (-not (Test-Path $composeFile)) {
    throw "Could not find docker-compose.yml at $composeFile"
}

if (-not (Test-Path $seedScript)) {
    throw "Could not find seed helper at $seedScript"
}

if (-not (Test-Path $refreshScript)) {
    throw "Could not find API refresh helper at $refreshScript"
}

if ($FreshData -and $ApiOnly) {
    throw "-FreshData and -ApiOnly cannot be used together. Use .\\scripts\\local-refresh-api.ps1 for API-only refreshes."
}

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory)]
        [string]$Label,
        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    & $Command

    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Wait-ForHttpOk {
    param(
        [Parameter(Mandatory)]
        [string]$Uri,
        [int]$TimeoutSeconds = 180
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        try {
            $response = Invoke-WebRequest -Uri $Uri -TimeoutSec 5 -UseBasicParsing
            if ($response.StatusCode -eq 200) {
                return
            }
        } catch {
            Start-Sleep -Seconds 2
            continue
        }

        Start-Sleep -Seconds 2
    }

    Write-Host "`nCurrent docker compose status:" -ForegroundColor Yellow
    docker compose ps
    Write-Host "`nRecent API logs:" -ForegroundColor Yellow
    docker compose logs --tail 80 api
    throw "Timed out waiting for $Uri to return HTTP 200"
}

function Test-DemoSeedTargetEmpty {
    $sql = @"
SELECT
    (SELECT COUNT(*) FROM accounts WHERE email <> 'admin@tessara.local')
  + (SELECT COUNT(*) FROM node_types)
  + (SELECT COUNT(*) FROM nodes)
  + (SELECT COUNT(*) FROM forms)
  + (SELECT COUNT(*) FROM form_versions)
  + (SELECT COUNT(*) FROM submissions)
  + (SELECT COUNT(*) FROM workflows)
  + (SELECT COUNT(*) FROM workflow_versions)
  + (SELECT COUNT(*) FROM datasets)
  + (SELECT COUNT(*) FROM dataset_revisions)
  + (SELECT COUNT(*) FROM components)
  + (SELECT COUNT(*) FROM component_versions);
"@

    $rawCount = docker compose exec -T postgres psql -U tessara -d tessara -Atc $sql
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect database before demo seeding."
    }

    return ([int64]$rawCount.Trim()) -eq 0
}
if ([string]::IsNullOrWhiteSpace($ExternalDatabaseUrl) -xor [string]::IsNullOrWhiteSpace($ExternalDatabaseContainerId)) {
    throw "-ExternalDatabaseUrl and -ExternalDatabaseContainerId must be supplied together."
}
if (-not [string]::IsNullOrWhiteSpace($ExternalDatabaseUrl) -and ($FreshData -or $ApiOnly -or -not $SkipSeed)) {
    throw "An external upgraded database requires -SkipSeed and cannot be combined with -FreshData or -ApiOnly."
}

function Set-TessaraImageBuildProvenance {
    $commit = (& git -C $repoRoot rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -notmatch "^[0-9a-f]{40}$") {
        throw "Could not resolve the source commit for the release image."
    }
    $tree = (& git -C $repoRoot rev-parse "HEAD^{tree}").Trim()
    if ($LASTEXITCODE -ne 0 -or $tree -notmatch "^[0-9a-f]{40}$") {
        throw "Could not resolve the source tree for the release image."
    }
    $status = (& git -C $repoRoot status --porcelain=v1 --untracked-files=all) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw "Could not determine whether the source tree is clean."
    }

    $env:TESSARA_SOURCE_COMMIT = $commit
    $env:TESSARA_SOURCE_TREE = $tree
    $env:TESSARA_SOURCE_DIRTY = if ([string]::IsNullOrWhiteSpace($status)) { "false" } else { "true" }
    Write-Host (
        "Release image provenance: commit {0}, tree {1}, dirty={2}" -f `
            $commit,
            $tree,
            $env:TESSARA_SOURCE_DIRTY
    )
}

function Restore-TessaraImageBuildEnvironment {
    param([Parameter(Mandatory)][hashtable]$Snapshot)

    foreach ($name in @("TESSARA_SOURCE_COMMIT", "TESSARA_SOURCE_TREE", "TESSARA_SOURCE_DIRTY")) {
        $entry = $Snapshot[$name]
        if ($null -eq $entry) {
            throw "Image-build environment snapshot is missing '$name'."
        }
        if ([bool]$entry.Present) {
            [Environment]::SetEnvironmentVariable($name, [string]$entry.Value)
        } else {
            Remove-Item "Env:$name" -ErrorAction SilentlyContinue
        }
    }
}

function Set-TessaraDatabaseOverride {
    param([string]$DatabaseUrl)

    if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
        Remove-Item Env:TESSARA_DATABASE_URL -ErrorAction SilentlyContinue
    } else {
        $env:TESSARA_DATABASE_URL = $DatabaseUrl
    }
}

function Restore-TessaraDatabaseOverride {
    param(
        [Parameter(Mandatory)][bool]$HadCallerValue,
        [AllowNull()][string]$CallerValue
    )

    if ($HadCallerValue) {
        $env:TESSARA_DATABASE_URL = $CallerValue
    } else {
        Remove-Item Env:TESSARA_DATABASE_URL -ErrorAction SilentlyContinue
    }
}

function Assert-ExternalDatabaseTarget {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$ContainerId
    )

    $uri = [Uri]$DatabaseUrl
    $databaseName = $uri.AbsolutePath.TrimStart("/")
    $databaseUser = ($uri.UserInfo -split ":", 2)[0]
    if (-not $uri.IsAbsoluteUri -or
        $uri.Scheme -notin @("postgres", "postgresql") -or
        $uri.Host -notin @("host.docker.internal", "gateway.docker.internal") -or
        $uri.Port -lt 1 -or
        $databaseName -notmatch "^[A-Za-z_][A-Za-z0-9_-]*$" -or
        $databaseUser -notmatch "^[A-Za-z_][A-Za-z0-9_-]*$") {
        throw "The external database URL must use postgres://host.docker.internal:<published-port>/<disposable-database>."
    }
    $nameTokens = @($databaseName.ToLowerInvariant() -split "[^a-z0-9]+" | Where-Object { $_ })
    $hasDisposableToken = @($nameTokens | Where-Object {
        $_ -in @("test", "tests", "testing", "upgrade", "clone", "rollback", "sprint6a")
    }).Count -gt 0
    for ($index = 0; $index -lt ($nameTokens.Count - 1); $index++) {
        if ($nameTokens[$index] -eq "sprint" -and $nameTokens[$index + 1] -eq "6a") {
            $hasDisposableToken = $true
        }
    }
    if (-not $hasDisposableToken) {
        throw "External closing-build deployment refuses database '$databaseName': its name lacks a token-bounded disposable marker."
    }

    $inspectRaw = docker container inspect $ContainerId
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect external database container '$ContainerId'."
    }
    $inspectRows = @(ConvertFrom-Json -InputObject ($inspectRaw -join "`n"))
    if ($inspectRows.Count -ne 1 -or -not [bool]$inspectRows[0].State.Running) {
        throw "The external database container must identify exactly one running container."
    }
    $publishedPorts = @(
        $inspectRows[0].NetworkSettings.Ports."5432/tcp".HostPort |
            Sort-Object -Unique
    )
    if ($publishedPorts.Count -ne 1 -or [int]$publishedPorts[0] -ne $uri.Port) {
        throw "The external database URL port does not uniquely match the inspected container's published PostgreSQL port."
    }
    $currentDatabase = docker exec -i $ContainerId psql -X -v ON_ERROR_STOP=1 -U $databaseUser -d $databaseName -Atc "SELECT current_database();"
    if ($LASTEXITCODE -ne 0 -or ($currentDatabase -join "`n").Trim() -cne $databaseName) {
        throw "The inspected external container did not return the database named by ExternalDatabaseUrl."
    }
}

if ($SelfTest) {
    if (-not [string]::IsNullOrWhiteSpace($ExternalDatabaseUrl)) {
        Assert-ExternalDatabaseTarget `
            -DatabaseUrl $ExternalDatabaseUrl `
            -ContainerId $ExternalDatabaseContainerId
    }
    $hadOriginalValue = Test-Path Env:TESSARA_DATABASE_URL
    $originalValue = if ($hadOriginalValue) { $env:TESSARA_DATABASE_URL } else { $null }
    $originalImageBuildEnvironment = @{}
    foreach ($name in @("TESSARA_SOURCE_COMMIT", "TESSARA_SOURCE_TREE", "TESSARA_SOURCE_DIRTY")) {
        $present = Test-Path "Env:$name"
        $originalImageBuildEnvironment[$name] = [pscustomobject]@{
            Present = $present
            Value = if ($present) { [Environment]::GetEnvironmentVariable($name) } else { $null }
        }
    }
    try {
        $env:TESSARA_DATABASE_URL = "caller-value"
        Set-TessaraDatabaseOverride
        if (Test-Path Env:TESSARA_DATABASE_URL) {
            throw "Self-test failed: a normal/fresh launch inherited the caller's database override."
        }
        Restore-TessaraDatabaseOverride -HadCallerValue $true -CallerValue "caller-value"
        if ($env:TESSARA_DATABASE_URL -cne "caller-value") {
            throw "Self-test failed: caller database override was not restored."
        }
        Set-TessaraDatabaseOverride -DatabaseUrl "external-value"
        if ($env:TESSARA_DATABASE_URL -cne "external-value") {
            throw "Self-test failed: explicit external database override was not scoped."
        }
        Restore-TessaraDatabaseOverride -HadCallerValue $false
        if (Test-Path Env:TESSARA_DATABASE_URL) {
            throw "Self-test failed: an absent caller database override was not restored as absent."
        }
        $imageFixture = @{
            TESSARA_SOURCE_COMMIT = [pscustomobject]@{ Present = $true; Value = "caller-commit" }
            TESSARA_SOURCE_TREE = [pscustomobject]@{ Present = $false; Value = $null }
            TESSARA_SOURCE_DIRTY = [pscustomobject]@{ Present = $true; Value = "caller-dirty" }
        }
        $env:TESSARA_SOURCE_COMMIT = "temporary"
        $env:TESSARA_SOURCE_TREE = "temporary"
        $env:TESSARA_SOURCE_DIRTY = "temporary"
        Restore-TessaraImageBuildEnvironment -Snapshot $imageFixture
        if ($env:TESSARA_SOURCE_COMMIT -cne "caller-commit" -or
            (Test-Path Env:TESSARA_SOURCE_TREE) -or
            $env:TESSARA_SOURCE_DIRTY -cne "caller-dirty") {
            throw "Self-test failed: image-build provenance environment was not restored exactly."
        }
    } finally {
        Restore-TessaraDatabaseOverride `
            -HadCallerValue $hadOriginalValue `
            -CallerValue $originalValue
        Restore-TessaraImageBuildEnvironment -Snapshot $originalImageBuildEnvironment
    }
    Write-Host "local-launch database override scope self-test passed." -ForegroundColor Green
    exit 0
}

if (-not [string]::IsNullOrWhiteSpace($ExternalDatabaseUrl)) {
    Assert-ExternalDatabaseTarget `
        -DatabaseUrl $ExternalDatabaseUrl `
        -ContainerId $ExternalDatabaseContainerId
}

Push-Location $repoRoot
try {
    Set-TessaraDatabaseOverride -DatabaseUrl $ExternalDatabaseUrl

    if ($ApiOnly) {
        $refreshArgs = @{}
        if ($SkipBuild) {
            $refreshArgs.SkipBuild = $true
        }
        if ($SkipSeed) {
            $refreshArgs.SkipSeed = $true
        }
        if ($FollowLogs) {
            $refreshArgs.FollowLogs = $true
        }

        & $refreshScript @refreshArgs
        return
    }

    $downArgs = @("compose", "down")
    if ($FreshData) {
        $downArgs += "-v"
    }

    Invoke-CheckedStep -Label "Stopping existing Compose stack" -Command {
        docker @downArgs
    }

    if (-not $SkipBuild) {
        Set-TessaraImageBuildProvenance
        Invoke-CheckedStep -Label "Rebuilding Tessara API image" -Command {
            docker compose build api
        }
    } else {
        Write-Host "`n==> Reusing existing API image" -ForegroundColor Cyan
    }

    Invoke-CheckedStep -Label "Starting refreshed Compose stack" -Command {
        docker compose up -d --force-recreate
    }

    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/health"
    Wait-ForHttpOk -Uri "http://127.0.0.1:8080/"

    if (-not $SkipSeed -and (Test-DemoSeedTargetEmpty)) {
        Invoke-CheckedStep -Label "Ensuring UAT demo data" -Command {
            & $seedScript
        }
    } elseif (-not $SkipSeed) {
        Write-Host "`n==> Skipping demo seed because the database already contains app data" -ForegroundColor Cyan
        Write-Host "Use .\scripts\local-launch.ps1 -FreshData to recreate and reseed the local database." -ForegroundColor Yellow
    } else {
        Write-Host "`n==> Skipping demo seed" -ForegroundColor Cyan
    }

    Write-Host "`nTessara is ready." -ForegroundColor Green
    Write-Host "Application shell: http://localhost:8080/"
    Write-Host "Administration:   http://localhost:8080/administration"
    Write-Host "Node Types:       http://localhost:8080/administration/node-types"
    Write-Host "Roles:            http://localhost:8080/administration/roles"
    Write-Host ""
    Write-Host "Demo accounts:" -ForegroundColor Green
    Write-Host "  admin@tessara.local       / tessara-dev-admin"
    Write-Host "  operator@tessara.local    / tessara-dev-operator"
    Write-Host "  delegator@tessara.local   / tessara-dev-delegator"
    Write-Host "  respondent@tessara.local  / tessara-dev-respondent"
    Write-Host "  delegate@tessara.local    / tessara-dev-delegate"
    if ($FreshData) {
        Write-Host ""
        Write-Host "Postgres volume was refreshed because -FreshData was supplied." -ForegroundColor Yellow
    }
    if ($SkipBuild) {
        Write-Host "API image rebuild was skipped because -SkipBuild was supplied." -ForegroundColor Yellow
    }
    if ($SkipSeed) {
        Write-Host "Demo seeding was skipped because -SkipSeed was supplied." -ForegroundColor Yellow
    }
    if (-not [string]::IsNullOrWhiteSpace($ExternalDatabaseUrl)) {
        Write-Host "API is using the verified external disposable upgraded database container '$ExternalDatabaseContainerId'." -ForegroundColor Yellow
    }

    if ($FollowLogs) {
        Write-Host "`nFollowing Compose logs. Press Ctrl+C to stop log streaming." -ForegroundColor Cyan
        docker compose logs -f postgres api
    }
} finally {
    Pop-Location
    Restore-TessaraDatabaseOverride `
        -HadCallerValue $hadCallerDatabaseOverride `
        -CallerValue $callerDatabaseOverride
    Restore-TessaraImageBuildEnvironment -Snapshot $callerImageBuildEnvironment
}

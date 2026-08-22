[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$containerName = "tessara-s8b-component-tests-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
$databaseUrlBefore = $env:TEST_COMPONENT_MODULE_DATABASE_URL
. (Join-Path $PSScriptRoot "sprint-8b-cargo-test-integrity.ps1")

try {
    $containerId = docker run --detach --rm --name $containerName `
        -e POSTGRES_USER=tessara_component_test `
        -e POSTGRES_PASSWORD=tessara_component_test `
        -e POSTGRES_DB=tessara_component_test `
        -p "127.0.0.1::5432" `
        postgres:16-alpine
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($containerId)) {
        throw "Could not start the isolated Sprint 8B Component consumer database."
    }

    $ready = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        docker exec $containerName pg_isready `
            -U tessara_component_test -d tessara_component_test *> $null
        if ($LASTEXITCODE -eq 0) { $ready = $true; break }
        Start-Sleep -Seconds 1
    }
    if (-not $ready) { throw "Sprint 8B Component consumer database did not become ready." }
    $port = docker port $containerName 5432/tcp
    if ($port -notmatch '127\.0\.0\.1:(\d+)$') {
        throw "Could not resolve the isolated Sprint 8B Component consumer database port."
    }
    $env:TEST_COMPONENT_MODULE_DATABASE_URL =
        "postgres://tessara_component_test:tessara_component_test@127.0.0.1:$($Matches[1])/tessara_component_test"

    Push-Location $repoRoot
    try {
        Invoke-Sprint8BCheckedCargoTest -Arguments @(
            "test", "-p", "tessara-component-module", "--test", "product_integration",
            "--locked", "--offline", "--jobs", "1"
        ) | Out-Null
    } finally {
        Pop-Location
    }
} finally {
    $env:TEST_COMPONENT_MODULE_DATABASE_URL = $databaseUrlBefore
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        docker rm -f $containerName | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Could not remove the Component consumer database." }
    }
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        throw "Component consumer database teardown was not exact."
    }
}

[CmdletBinding()]
param(
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$checks = [Collections.Generic.List[object]]::new()

function Invoke-ConformanceCheck {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][scriptblock]$Command
    )
    $started = [DateTimeOffset]::UtcNow
    & $Command
    $exitCode = $LASTEXITCODE
    $checks.Add([pscustomobject][ordered]@{
        name = $Name
        started_at = $started.ToString("o")
        completed_at = [DateTimeOffset]::UtcNow.ToString("o")
        exit_code = $exitCode
        passed = $exitCode -eq 0
    })
    if ($exitCode -ne 0) {
        throw "Module SDK conformance check '$Name' failed with exit code $exitCode."
    }
}

Push-Location $repoRoot
try {
    Invoke-ConformanceCheck "contract-runtime-ui-testkit-reference-tests" {
        & cargo test -p tessara-module-contract -p tessara-module-runtime `
            -p tessara-module-ui -p tessara-module-testkit `
            -p tessara-reference-module-sdk --features tessara-reference-module-sdk/ssr --locked
    }
    Invoke-ConformanceCheck "contract-wasm" {
        & cargo check -p tessara-module-contract --target wasm32-unknown-unknown --locked
    }
    Invoke-ConformanceCheck "ui-wasm" {
        & cargo check -p tessara-module-ui --no-default-features --features hydrate `
            --target wasm32-unknown-unknown --locked
    }
    Invoke-ConformanceCheck "reference-wasm" {
        & cargo check -p tessara-reference-module-sdk --no-default-features --features hydrate `
            --target wasm32-unknown-unknown --locked
    }
    Invoke-ConformanceCheck "package-source-boundaries" {
        & .\scripts\verify-module-sdk-boundaries.ps1
    }
    Invoke-ConformanceCheck "compatibility-inventory" {
        & .\scripts\verify-module-sdk-compatibility.ps1
    }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_commit = (& git rev-parse HEAD).Trim()
        source_tree = (& git rev-parse "HEAD^{tree}").Trim()
        source_dirty = -not [string]::IsNullOrWhiteSpace((& git status --porcelain) -join "")
        checks = @($checks)
        passed = @($checks | Where-Object { -not $_.passed }).Count -eq 0
    }
    if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        $fullPath = if ([IO.Path]::IsPathRooted($EvidencePath)) {
            [IO.Path]::GetFullPath($EvidencePath)
        } else {
            [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidencePath))
        }
        [IO.Directory]::CreateDirectory((Split-Path -Parent $fullPath)) | Out-Null
        [IO.File]::WriteAllText(
            $fullPath,
            ($result | ConvertTo-Json -Depth 20) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    $result | ConvertTo-Json -Depth 20
} finally {
    Pop-Location
}

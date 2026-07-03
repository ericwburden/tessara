[CmdletBinding()]
param(
    [string]$Phase = "commit-0",
    [string]$OutputRoot,
    [string]$PilotTargetRoot,
    [switch]$Inventory,
    [switch]$BaselineChecks,
    [switch]$BundleReport,
    [switch]$AllowNormalCache
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $repoRoot "tmp\web-refactor-pilot"
}
if ([string]::IsNullOrWhiteSpace($PilotTargetRoot)) {
    $PilotTargetRoot = Join-Path $repoRoot "tmp\pilot-targets"
}

if (-not $Inventory -and -not $BaselineChecks -and -not $BundleReport) {
    $Inventory = $true
}

function Resolve-UnderRepo {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $fullRepo = [System.IO.Path]::GetFullPath($repoRoot)

    if (-not $fullPath.StartsWith($fullRepo, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to write outside repository: $fullPath"
    }

    $fullPath
}

function Resolve-UnderPath {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter(Mandatory)]
        [string]$AllowedRoot
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $fullRoot = [System.IO.Path]::GetFullPath($AllowedRoot)

    if (-not $fullPath.StartsWith($fullRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing path outside allowed root. Path: $fullPath Root: $fullRoot"
    }

    $fullPath
}

function Remove-PilotDirectory {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter(Mandatory)]
        [string]$AllowedRoot
    )

    $resolved = Resolve-UnderPath -Path $Path -AllowedRoot $AllowedRoot
    Remove-Item -LiteralPath $resolved -Recurse -Force -ErrorAction SilentlyContinue
}

function New-PhaseDirectory {
    $safePhase = $Phase -replace '[^A-Za-z0-9_.-]', '-'
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $root = Resolve-UnderRepo -Path $OutputRoot
    $dir = Join-Path $root "$safePhase-$stamp"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $dir
}

function Format-CommandLine {
    param(
        [Parameter(Mandatory)]
        [string]$Command,

        [string[]]$Arguments = @()
    )

    $parts = @($Command) + $Arguments
    ($parts | ForEach-Object {
        if ($_ -match '\s') {
            '"' + ($_ -replace '"', '\"') + '"'
        } else {
            $_
        }
    }) -join " "
}

function Invoke-LoggedCommand {
    param(
        [Parameter(Mandatory)]
        [string]$Label,

        [Parameter(Mandatory)]
        [string]$Command,

        [string[]]$Arguments = @(),

        [Parameter(Mandatory)]
        [string]$OutputDirectory,

        [string]$CargoTargetDirectory,

        [switch]$CleanCargoTarget,

        [switch]$CleanLeptosSite
    )

    $safeLabel = $Label.ToLowerInvariant() -replace '[^a-z0-9]+', '-'
    $safeLabel = $safeLabel.Trim("-")
    $logPath = Join-Path $OutputDirectory "$safeLabel.txt"
    $startedAt = Get-Date

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    Write-Host (Format-CommandLine -Command $Command -Arguments $Arguments) -ForegroundColor DarkGray

    $previousTarget = $env:CARGO_TARGET_DIR
    try {
        if (-not [string]::IsNullOrWhiteSpace($CargoTargetDirectory)) {
            $targetRoot = Resolve-UnderRepo -Path $PilotTargetRoot
            $target = Resolve-UnderPath -Path $CargoTargetDirectory -AllowedRoot $targetRoot

            if ($CleanCargoTarget) {
                Remove-PilotDirectory -Path $target -AllowedRoot $targetRoot
            }

            New-Item -ItemType Directory -Force -Path $target | Out-Null
            $env:CARGO_TARGET_DIR = $target
        }

        if ($CleanLeptosSite) {
            $siteRoot = Join-Path $repoRoot "target\site"
            Remove-PilotDirectory -Path $siteRoot -AllowedRoot (Join-Path $repoRoot "target")
        }

        $startedAt = Get-Date
        $output = & $Command @Arguments 2>&1
        $exitCode = if ($null -eq $LASTEXITCODE) { 0 } else { $LASTEXITCODE }
        $output | ForEach-Object { $_.ToString() } | Set-Content -LiteralPath $logPath -Encoding utf8
        $finishedAt = Get-Date
    } finally {
        if ($null -eq $previousTarget) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        } else {
            $env:CARGO_TARGET_DIR = $previousTarget
        }
    }

    $elapsed = $finishedAt - $startedAt

    [pscustomobject]@{
        label = $Label
        command = Format-CommandLine -Command $Command -Arguments $Arguments
        exitCode = $exitCode
        startedAtUtc = $startedAt.ToUniversalTime().ToString("o")
        finishedAtUtc = $finishedAt.ToUniversalTime().ToString("o")
        elapsedSeconds = [math]::Round($elapsed.TotalSeconds, 3)
        cargoTargetDir = if ([string]::IsNullOrWhiteSpace($CargoTargetDirectory)) { $null } else { [System.IO.Path]::GetFullPath($CargoTargetDirectory) }
        log = $logPath
    }
}

function Assert-NoTessaraBuildProcesses {
    $processes = Get-Process | Where-Object {
        $_.ProcessName -match '^(cargo|rustc|cargo-leptos)$' -or
        $_.ProcessName -like '*cargo*' -or
        $_.ProcessName -like '*rustc*'
    } | Select-Object Id,ProcessName,Path

    if ($processes) {
        $processes | Format-Table | Out-String | Write-Host
        throw "Unrelated Cargo/Rust build processes are active. Stop them before decision-grade measurements."
    }
}

function Set-PrimaryCachePolicy {
    if ($AllowNormalCache) {
        Write-Host "Using normal cache configuration because -AllowNormalCache was supplied." -ForegroundColor Yellow
        return
    }

    Remove-Item Env:RUSTC_WRAPPER -ErrorAction SilentlyContinue
    $env:SCCACHE_DISABLE = "1"
    $env:CARGO_INCREMENTAL = "1"
}

function Write-BundleReport {
    param(
        [Parameter(Mandatory)]
        [string]$OutputDirectory
    )

    $pkg = Join-Path $repoRoot "target\site\pkg"
    $reportPath = Join-Path $OutputDirectory "bundle-report.json"

    if (-not (Test-Path $pkg)) {
        throw "No cargo-leptos package directory exists at $pkg"
    }

    $files = Get-ChildItem $pkg -File |
        Where-Object { $_.Extension -in ".wasm", ".js", ".css" } |
        Sort-Object Name |
        Select-Object Name,Length,LastWriteTimeUtc

    $clientBytes = ($files | Where-Object { $_.Name -match '\.(wasm|js)$' } | Measure-Object Length -Sum).Sum
    $report = [pscustomobject]@{
        worktree = $repoRoot
        gitSha = (& git rev-parse HEAD).Trim()
        cargoTargetDir = $env:CARGO_TARGET_DIR
        siteRoot = (Join-Path $repoRoot "target\site")
        reportedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
        clientJsWasmBytes = $clientBytes
        files = $files
    }

    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
    Write-Host "Bundle report written to $reportPath" -ForegroundColor Green
}

function Write-EnvironmentReport {
    param(
        [Parameter(Mandatory)]
        [string]$OutputDirectory
    )

    $computer = Get-CimInstance Win32_ComputerSystem -ErrorAction SilentlyContinue
    $processor = Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1
    $os = Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue

    $powerPlan = $null
    $powerCfg = Get-Command powercfg.exe -ErrorAction SilentlyContinue
    if ($powerCfg) {
        $powerPlan = (& powercfg.exe /GETACTIVESCHEME 2>$null) -join "`n"
    }

    $report = [pscustomobject]@{
        worktree = $repoRoot
        gitSha = (& git rev-parse HEAD).Trim()
        recordedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
        rustc = (& rustc -Vv) -join "`n"
        cargo = (& cargo -V) -join "`n"
        cargoLeptos = (& cargo leptos --version) -join "`n"
        os = $os | Select-Object Caption,Version,BuildNumber,OSArchitecture
        cpu = $processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors
        memoryBytes = $computer.TotalPhysicalMemory
        powerPlan = $powerPlan
        rustcWrapper = $env:RUSTC_WRAPPER
        sccacheDisable = $env:SCCACHE_DISABLE
        cargoIncremental = $env:CARGO_INCREMENTAL
    }

    $reportPath = Join-Path $OutputDirectory "environment.json"
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding utf8
}

$results = @()
$phaseDir = New-PhaseDirectory

Push-Location $repoRoot
try {
    Assert-NoTessaraBuildProcesses
    Set-PrimaryCachePolicy
    Write-EnvironmentReport -OutputDirectory $phaseDir

    if ($Inventory) {
        $results += Invoke-LoggedCommand -Label "git rev-parse HEAD" -Command "git" -Arguments @("rev-parse", "HEAD") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "git status short" -Command "git" -Arguments @("status", "--short") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "rustc version verbose" -Command "rustc" -Arguments @("-Vv") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "cargo version" -Command "cargo" -Arguments @("-V") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "cargo leptos version" -Command "cargo" -Arguments @("leptos", "--version") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "cargo metadata no deps" -Command "cargo" -Arguments @("metadata", "--format-version", "1", "--no-deps") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "tessara web ssr feature tree depth 1" -Command "cargo" -Arguments @("tree", "-p", "tessara-web", "-e", "features", "--depth", "1", "--features", "ssr", "--color", "never") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "tessara web hydrate feature tree depth 1" -Command "cargo" -Arguments @("tree", "-p", "tessara-web", "-e", "features", "--depth", "1", "--no-default-features", "--features", "hydrate", "--target", "wasm32-unknown-unknown", "--color", "never") -OutputDirectory $phaseDir
        $results += Invoke-LoggedCommand -Label "tessara api ssr feature tree depth 1" -Command "cargo" -Arguments @("tree", "-p", "tessara-api", "-e", "features", "--depth", "1", "--features", "ssr", "--color", "never") -OutputDirectory $phaseDir
    }

    if ($BaselineChecks) {
        $baselineTargetRoot = Join-Path (Resolve-UnderRepo -Path $PilotTargetRoot) $Phase
        $results += Invoke-LoggedCommand -Label "web hydrate check" -Command "cargo" -Arguments @("check", "-p", "tessara-web", "--no-default-features", "--features", "hydrate", "--target", "wasm32-unknown-unknown") -OutputDirectory $phaseDir -CargoTargetDirectory (Join-Path $baselineTargetRoot "web-hydrate-check") -CleanCargoTarget
        $results += Invoke-LoggedCommand -Label "api ssr check" -Command "cargo" -Arguments @("check", "-p", "tessara-api", "--features", "ssr") -OutputDirectory $phaseDir -CargoTargetDirectory (Join-Path $baselineTargetRoot "api-ssr-check") -CleanCargoTarget
        $results += Invoke-LoggedCommand -Label "cargo leptos build" -Command "cargo" -Arguments @("leptos", "build") -OutputDirectory $phaseDir -CargoTargetDirectory (Join-Path $baselineTargetRoot "cargo-leptos-build") -CleanCargoTarget -CleanLeptosSite
        $results += Invoke-LoggedCommand -Label "web test compile no run" -Command "cargo" -Arguments @("test", "-p", "tessara-web", "--lib", "--no-run", "-j", "1") -OutputDirectory $phaseDir -CargoTargetDirectory (Join-Path $baselineTargetRoot "web-test-compile-no-run") -CleanCargoTarget
    }

    if ($BundleReport) {
        Write-BundleReport -OutputDirectory $phaseDir
    }
} finally {
    $summaryPath = Join-Path $phaseDir "summary.json"
    $results | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $summaryPath -Encoding utf8
    Write-Host "`nPilot output directory: $phaseDir" -ForegroundColor Green
    Pop-Location
}

[CmdletBinding()]
param(
    [string]$Sprint5ACommit = "3625d4de52c5856e4ac3bc642a9422a029e9f375",
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9a-fA-F]{40}$')]
    [string]$ClosingSprint6ACommit,
    [string]$OutputPath = "artifacts/sprint-6a/compatibility-rollback",
    [switch]$Force
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$outputFullPath = if ([IO.Path]::IsPathRooted($OutputPath)) {
    [IO.Path]::GetFullPath($OutputPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repositoryRoot $OutputPath))
}

function Invoke-Git {
    param([Parameter(Mandatory)][string[]]$Arguments)

    $output = & git -C $repositoryRoot @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "git -C $repositoryRoot $($Arguments -join ' ') failed:`n$($output -join [Environment]::NewLine)"
    }
    return (($output | ForEach-Object { $_.ToString() }) -join "`n").Trim()
}

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)

    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-RelativePackagePath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )

    return [IO.Path]::GetRelativePath($Root, $Path).Replace('\', '/')
}

function Get-StringSha256 {
    param([Parameter(Mandatory)][string]$Value)

    $utf8 = [Text.UTF8Encoding]::new($false)
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $hex = [BitConverter]::ToString($algorithm.ComputeHash($utf8.GetBytes($Value)))
        return $hex.Replace("-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Assert-SafeRemovalTarget {
    param([Parameter(Mandatory)][string]$Path)

    $resolved = [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $repository = $repositoryRoot.TrimEnd([IO.Path]::DirectorySeparatorChar)
    if ($resolved -eq $repository -or $resolved -eq [IO.Path]::GetPathRoot($resolved).TrimEnd([IO.Path]::DirectorySeparatorChar)) {
        throw "Refusing recursive removal of unsafe path '$resolved'."
    }
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw "git is required to export and identify the exact Sprint 5A source."
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required to build the exact Sprint 5A executable."
}
if (-not (Get-Command cargo-leptos -ErrorAction SilentlyContinue)) {
    throw "cargo-leptos is required to build the exact Sprint 5A SSR executable and hydration assets."
}
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    throw "rustc is required to record the builder version."
}
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "npm is required to build the exact Sprint 5A stylesheet and hydration assets."
}
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    throw "node is required to record the web builder version."
}
if (-not (Get-Command tar -ErrorAction SilentlyContinue)) {
    throw "tar is required to extract the exact git archive."
}

$resolvedSprint5ACommit = Invoke-Git @("rev-parse", "--verify", "$Sprint5ACommit`^{commit}")
$sprint6ARepositoryCommit = Invoke-Git @("rev-parse", "HEAD")
$resolvedClosingSprint6ACommit = Invoke-Git @(
    "rev-parse",
    "--verify",
    "$ClosingSprint6ACommit`^{commit}"
)
if ($resolvedClosingSprint6ACommit -ne $sprint6ARepositoryCommit) {
    throw "Closing Sprint 6A commit '$resolvedClosingSprint6ACommit' is not current HEAD '$sprint6ARepositoryCommit'. Build the retained package from the exact reviewed closing commit."
}
$workingTreeStatus = Invoke-Git @("status", "--porcelain=v1", "--untracked-files=all")
if (-not [string]::IsNullOrWhiteSpace($workingTreeStatus)) {
    throw "The complete working tree must be clean before building the retained rollback package for closing commit $sprint6ARepositoryCommit.`n$workingTreeStatus"
}

$migrationPaths = @(
    "crates/tessara-api/migrations/001_baseline.sql",
    "crates/tessara-api/migrations/002_dashboard_placement_capacity.sql",
    "crates/tessara-api/migrations/003_module_control_plane.sql"
)

foreach ($relativePath in $migrationPaths) {
    $absolutePath = Join-Path $repositoryRoot $relativePath
    if (-not (Test-Path -LiteralPath $absolutePath -PathType Leaf)) {
        throw "Required migration '$relativePath' does not exist."
    }
    & git -C $repositoryRoot ls-files --error-unmatch -- $relativePath *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "Required migration '$relativePath' must be committed before building a retained rollback package."
    }
    & git -C $repositoryRoot diff --quiet HEAD -- $relativePath
    if ($LASTEXITCODE -eq 1) {
        throw "Required migration '$relativePath' differs from HEAD; commit it before building a retained rollback package."
    }
    if ($LASTEXITCODE -gt 1) {
        throw "Could not verify the working-tree state of '$relativePath'."
    }
}

foreach ($relativePath in $migrationPaths[0..1]) {
    $historicalBlob = Invoke-Git @("rev-parse", "${resolvedSprint5ACommit}:$relativePath")
    $currentBlob = Invoke-Git @("hash-object", "--", (Join-Path $repositoryRoot $relativePath))
    if ($historicalBlob -ne $currentBlob) {
        throw "Migration '$relativePath' is not byte-identical to Sprint 5A commit $resolvedSprint5ACommit."
    }
}

if (Test-Path -LiteralPath $outputFullPath) {
    if (-not $Force) {
        throw "Output '$outputFullPath' already exists. Pass -Force to replace this explicit package target."
    }
    Assert-SafeRemovalTarget $outputFullPath
}

$stagingParent = [IO.Path]::GetFullPath((Join-Path $repositoryRoot "tmp"))
New-Item -ItemType Directory -Path $stagingParent -Force | Out-Null
$stagingRoot = Join-Path $stagingParent "sprint-6a-rollback-build-$PID-$([Guid]::NewGuid().ToString('N'))"
$sourceRoot = Join-Path $stagingRoot "source"
$buildTarget = Join-Path $sourceRoot "target"
$sourceArchive = Join-Path $stagingRoot "sprint-5a-source.tar"
$preservedStylesheet = Join-Path $stagingRoot "tessara-web.css"
$packageBuildPath = Join-Path $stagingRoot "package"
New-Item -ItemType Directory -Path $sourceRoot -Force | Out-Null

try {
    Invoke-Git @(
        "archive",
        "--format=tar",
        "--output=$sourceArchive",
        $resolvedSprint5ACommit
    ) | Out-Null
    & tar -xf $sourceArchive -C $sourceRoot
    if ($LASTEXITCODE -ne 0) {
        throw "Could not extract the Sprint 5A source archive."
    }

    $buildEnvironment = [ordered]@{
        CARGO_TARGET_DIR = $buildTarget
        LEPTOS_SITE_ROOT = (Join-Path $buildTarget "site")
        LEPTOS_SITE_PKG_DIR = "pkg"
    }
    $previousBuildEnvironment = @{}
    foreach ($name in $buildEnvironment.Keys) {
        $previousBuildEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
        [Environment]::SetEnvironmentVariable($name, $buildEnvironment[$name], "Process")
    }
    $locationPushed = $false
    try {
        Push-Location $sourceRoot
        $locationPushed = $true
        & npm ci
        if ($LASTEXITCODE -ne 0) {
            throw "The exact Sprint 5A web dependency installation failed."
        }
        & npm run tailwind:build
        if ($LASTEXITCODE -ne 0) {
            throw "The exact Sprint 5A stylesheet build failed."
        }
        $stylesheet = Join-Path $buildTarget "site/pkg/tessara-web.css"
        if (-not (Test-Path -LiteralPath $stylesheet -PathType Leaf)) {
            throw "The Sprint 5A stylesheet build did not produce '$stylesheet'."
        }
        Copy-Item -LiteralPath $stylesheet -Destination $preservedStylesheet

        & cargo leptos build --release --split
        if ($LASTEXITCODE -ne 0) {
            throw "The exact Sprint 5A SSR executable/hydration build failed."
        }
        $stylesheetDirectory = Split-Path -Parent $stylesheet
        New-Item -ItemType Directory -Path $stylesheetDirectory -Force | Out-Null
        Copy-Item -LiteralPath $preservedStylesheet -Destination $stylesheet
    } finally {
        if ($locationPushed) {
            Pop-Location
        }
        foreach ($name in $buildEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($name, $previousBuildEnvironment[$name], "Process")
        }
    }

    $binaryName = if ($env:OS -eq "Windows_NT") { "tessara-api.exe" } else { "tessara-api" }
    $builtBinary = Join-Path $buildTarget "release/$binaryName"
    if (-not (Test-Path -LiteralPath $builtBinary -PathType Leaf)) {
        throw "Expected Sprint 5A executable '$builtBinary' was not produced."
    }
    $builtSite = Join-Path $buildTarget "site"
    if (-not (Test-Path -LiteralPath $builtSite -PathType Container)) {
        throw "Expected Sprint 5A site bundle '$builtSite' was not produced."
    }
    foreach ($assetName in @("tessara-web.css", "tessara-web.js", "tessara-web.wasm")) {
        $assetPath = Join-Path $builtSite "pkg/$assetName"
        if (-not (Test-Path -LiteralPath $assetPath -PathType Leaf)) {
            throw "Expected Sprint 5A hydration asset '$assetPath' was not produced."
        }
    }

    $appDirectory = Join-Path $packageBuildPath "app"
    $siteDirectory = Join-Path $packageBuildPath "site"
    $compatibilityMigrationsDirectory = Join-Path $packageBuildPath "migrations"
    $originalMigrationsDirectory = Join-Path $packageBuildPath "original-migrations"
    $sourceDirectory = Join-Path $packageBuildPath "source"
    foreach ($directory in @(
        $appDirectory,
        $siteDirectory,
        $compatibilityMigrationsDirectory,
        $originalMigrationsDirectory,
        $sourceDirectory
    )) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }

    $packagedBinary = Join-Path $appDirectory $binaryName
    Copy-Item -LiteralPath $builtBinary -Destination $packagedBinary
    Get-ChildItem -LiteralPath $builtSite -Force | Copy-Item -Destination $siteDirectory -Recurse -Force
    Copy-Item -LiteralPath $sourceArchive -Destination (Join-Path $sourceDirectory "sprint-5a-source.tar")
    foreach ($relativePath in $migrationPaths) {
        $copyArguments = @{
            LiteralPath = Join-Path $repositoryRoot $relativePath
            Destination = Join-Path $compatibilityMigrationsDirectory ([IO.Path]::GetFileName($relativePath))
        }
        Copy-Item @copyArguments
    }
    foreach ($relativePath in $migrationPaths[0..1]) {
        $copyArguments = @{
            LiteralPath = Join-Path $sourceRoot $relativePath
            Destination = Join-Path $originalMigrationsDirectory ([IO.Path]::GetFileName($relativePath))
        }
        Copy-Item @copyArguments
    }

    $payloadFiles = @(Get-ChildItem -LiteralPath $packageBuildPath -File -Recurse | Sort-Object FullName)
    $fileEntries = @($payloadFiles | ForEach-Object {
        [pscustomobject][ordered]@{
            path = Get-RelativePackagePath $packageBuildPath $_.FullName
            sha256 = Get-Sha256 $_.FullName
            length_bytes = $_.Length
        }
    } | Sort-Object path)
    $digestLines = @($fileEntries | ForEach-Object { "$($_.path)=$($_.sha256)" })
    $packageContentDigest = Get-StringSha256 (($digestLines -join "`n") + "`n")

    $migrationEntries = @()
    for ($index = 0; $index -lt $migrationPaths.Count; $index++) {
        $fileName = [IO.Path]::GetFileName($migrationPaths[$index])
        $packagedPath = Join-Path $compatibilityMigrationsDirectory $fileName
        $migrationEntries += [ordered]@{
            version = $index + 1
            file_name = $fileName
            path = "migrations/$fileName"
            sha256 = Get-Sha256 $packagedPath
        }
    }

    $manifest = [ordered]@{
        schema_version = 1
        package_kind = "tessara_sprint_5a_code_sprint_6a_migration_compatibility_rollback"
        created_at_utc = (Get-Date).ToUniversalTime().ToString("o")
        application = [ordered]@{
            sprint_5a_source_commit = $resolvedSprint5ACommit
            binary_path = "app/$binaryName"
            binary_sha256 = Get-Sha256 $packagedBinary
            site_path = "site"
            source_archive_path = "source/sprint-5a-source.tar"
            source_archive_sha256 = Get-Sha256 (Join-Path $sourceDirectory "sprint-5a-source.tar")
        }
        closing_sprint_6a_repository_commit = $sprint6ARepositoryCommit
        migrations = $migrationEntries
        builder = [ordered]@{
            cargo_version = (& cargo --version).Trim()
            rustc_version = (& rustc --version).Trim()
            cargo_leptos_version = (& cargo leptos --version).Trim()
            node_version = (& node --version).Trim()
            npm_version = (& npm --version).Trim()
            operating_system = [Environment]::OSVersion.VersionString
            packaging_script_sha256 = Get-Sha256 $PSCommandPath
            build_command = 'set isolated CARGO_TARGET_DIR/LEPTOS_SITE_ROOT; npm ci; npm run tailwind:build; preserve target/site/pkg/tessara-web.css; cargo leptos build --release --split; restore target/site/pkg/tessara-web.css'
        }
        compatibility_contract = [ordered]@{
            upgraded_database_ledger = @(1, 2, 3)
            compatibility_migrations_path = "migrations"
            control_plane_behavior = "ignored_by_exact_sprint_5a_code"
            destructive_down_migration = $false
            applied_checksum_edit = $false
            original_historical_migrations_path = "original-migrations"
            original_historical_usage = "only_after_pre_upgrade_backup_restore_to_ledger_1_2"
        }
        package_content = [ordered]@{
            digest_algorithm = "sha256_of_sorted_utf8_path_equals_sha256_lines_excluding_manifest"
            sha256 = $packageContentDigest
            files = $fileEntries
        }
    }

    $manifestPath = Join-Path $packageBuildPath "manifest.json"
    $manifestJson = $manifest | ConvertTo-Json -Depth 12
    [IO.File]::WriteAllText($manifestPath, $manifestJson + "`n", [Text.UTF8Encoding]::new($false))

    if (Test-Path -LiteralPath $outputFullPath) {
        Assert-SafeRemovalTarget $outputFullPath
        Remove-Item -LiteralPath $outputFullPath -Recurse -Force
    }
    $outputParent = Split-Path -Parent $outputFullPath
    New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
    Move-Item -LiteralPath $packageBuildPath -Destination $outputFullPath

    Write-Host "Sprint 6A compatibility rollback package created: $outputFullPath"
    Write-Host "Manifest: $(Join-Path $outputFullPath 'manifest.json')"
    Write-Host "Payload digest: $packageContentDigest"
} finally {
    if (Test-Path -LiteralPath $stagingRoot) {
        $resolvedStaging = [IO.Path]::GetFullPath($stagingRoot)
        $requiredPrefix = $stagingParent.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
        if (-not $resolvedStaging.StartsWith($requiredPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove unexpected staging path '$resolvedStaging'."
        }
        Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
    }
}

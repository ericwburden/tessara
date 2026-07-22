[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [ValidateSet("fresh")][string]$ExpectedDataState,
    [string]$OutputPath,
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$ApiContainerId,
    [string]$DatabaseContainerId,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$artifactRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot "artifacts/sprint-6a-ui"))
$artifactPrefix = "$artifactRoot$([IO.Path]::DirectorySeparatorChar)"
. (Join-Path $PSScriptRoot "sprint-6a-deployment-evidence-common.ps1")

function Assert-Sprint6AUiArtifactPath {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = Resolve-Sprint6ARepositoryPath -RepositoryRoot $repoRoot -Path $Path
    if (-not $fullPath.StartsWith($artifactPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Sprint 6A-UI deployment evidence must be written under '$artifactRoot'."
    }
    return $fullPath
}

# Sprint closeout resets the prior stack and captures only a freshly seeded
# database using the one squashed baseline migration.
function Get-Sprint6AExpectedMigrationLedger {
    param([Parameter(Mandatory)][string]$RepositoryRoot)

    $migrationDirectory = Join-Path $RepositoryRoot "crates/tessara-api/migrations"
    $rows = @()
    foreach ($file in @(Get-ChildItem -LiteralPath $migrationDirectory -Filter "*.sql" -File | Sort-Object Name)) {
        if ($file.Name -notmatch "^(?<version>[0-9]+)_(?<description>.+)\.sql$") {
            throw "Migration filename '$($file.Name)' does not follow the version_description.sql contract."
        }
        $rows += [pscustomobject][ordered]@{
            version = [int64]$Matches.version
            file = $file.Name
            checksum_sha384 = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA384).Hash.ToLowerInvariant()
        }
    }
    if (($rows.version -join ",") -cne "1") {
        throw "Sprint 6A-UI deployment evidence requires repository migration exactly 1; found '$($rows.version -join ",")'."
    }
    return $rows
}

function Assert-Sprint6AMigrationLedger {
    param(
        [Parameter(Mandatory)][object[]]$DatabaseMigrations,
        [Parameter(Mandatory)][object[]]$ExpectedMigrations
    )

    if ($DatabaseMigrations.Count -ne 1 -or ($DatabaseMigrations.version -join ",") -cne "1") {
        throw "The live database migration ledger must contain exactly successful version 1."
    }
    for ($index = 0; $index -lt 1; $index++) {
        $database = $DatabaseMigrations[$index]
        $expected = $ExpectedMigrations[$index]
        if (-not [bool]$database.success -or
            [int64]$database.version -ne [int64]$expected.version -or
            [string]$database.checksum_sha384 -cne [string]$expected.checksum_sha384) {
            throw "Migration $($expected.version) in the live database does not match '$($expected.file)' or was not successful."
        }
    }
}

function Publish-Sprint6AUiEvidencePair {
    param(
        [Parameter(Mandatory)][string]$TemporaryEvidencePath,
        [Parameter(Mandatory)][string]$TemporaryDigestPath,
        [Parameter(Mandatory)][string]$FinalEvidencePath,
        [Parameter(Mandatory)][string]$FinalDigestPath,
        [switch]$AllowOverwrite
    )

    $finalExists = Test-Path -LiteralPath $FinalEvidencePath -PathType Leaf
    $digestExists = Test-Path -LiteralPath $FinalDigestPath -PathType Leaf
    if (($finalExists -or $digestExists) -and -not $AllowOverwrite) {
        throw "Retained Sprint 6A-UI deployment evidence already exists. Refusing to overwrite '$FinalEvidencePath' or its sidecar without -Overwrite."
    }
    if ($TemporaryEvidencePath -ceq $TemporaryDigestPath) {
        throw "Sprint 6A-UI deployment evidence and digest temporary paths must be distinct."
    }
    [IO.File]::Move($TemporaryEvidencePath, $FinalEvidencePath, [bool]$AllowOverwrite)
    try {
        [IO.File]::Move($TemporaryDigestPath, $FinalDigestPath, [bool]$AllowOverwrite)
    } catch {
        Remove-Item -LiteralPath $FinalEvidencePath -Force -ErrorAction SilentlyContinue
        throw
    }
}

if ($SelfTest) {
    $expected = @([pscustomobject]@{ version = 1; file = "001_test.sql"; checksum_sha384 = "d" * 96 })
    $database = @([pscustomobject]@{ version = 1; success = $true; checksum_sha384 = "d" * 96 })
    Assert-Sprint6AMigrationLedger -DatabaseMigrations $database -ExpectedMigrations $expected
    $database[0].checksum_sha384 = "e" * 96
    $rejected = $false
    try {
        Assert-Sprint6AMigrationLedger -DatabaseMigrations $database -ExpectedMigrations $expected
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Sprint 6A-UI deployment-evidence self-test failed: a baseline checksum mismatch was accepted."
    }
    $rejected = $false
    try {
        Assert-Sprint6AUiArtifactPath -Path "artifacts/sprint-6a/deployment-other-state.json" | Out-Null
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Sprint 6A-UI deployment-evidence self-test failed: a non-UI artifact path was accepted."
    }
    Write-Host "Sprint 6A-UI deployment-evidence publisher self-test passed." -ForegroundColor Green
    exit 0
}

if ([string]::IsNullOrWhiteSpace($ExpectedDataState)) {
    throw "-ExpectedDataState fresh is required. Sprint closeout accepts only a freshly seeded database."
}
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = "artifacts/sprint-6a-ui/deployment-$ExpectedDataState.json"
}
$fullOutputPath = Assert-Sprint6AUiArtifactPath -Path $OutputPath
$finalDigestPath = "$fullOutputPath.sha256"
if (((Test-Path -LiteralPath $fullOutputPath -PathType Leaf) -or
    (Test-Path -LiteralPath $finalDigestPath -PathType Leaf)) -and -not $Overwrite) {
    throw "Retained Sprint 6A-UI deployment evidence already exists. Refusing to overwrite '$fullOutputPath' or its sidecar without -Overwrite."
}

$snapshot = Get-Sprint6ADeploymentSnapshot `
    -RepositoryRoot $repoRoot `
    -BaseUrl $BaseUrl `
    -AdminEmail $AdminEmail `
    -AdminPassword $AdminPassword `
    -ApiContainerId $ApiContainerId `
    -DatabaseContainerId $DatabaseContainerId
if ([string]$snapshot.data.state -cne $ExpectedDataState) {
    throw "The database-derived data state is '$($snapshot.data.state)', not requested '$ExpectedDataState'."
}

$directory = Split-Path -Parent $fullOutputPath
[IO.Directory]::CreateDirectory($directory) | Out-Null
$temporaryPath = Join-Path $directory ".$([IO.Path]::GetFileName($fullOutputPath)).$([guid]::NewGuid().ToString('N')).tmp"
$temporaryDigestPath = "$temporaryPath.sha256"
try {
    $evidence = [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-6a.deployment-evidence"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        snapshot = $snapshot
    }
    $json = $evidence | ConvertTo-Json -Depth 30
    [IO.File]::WriteAllText($temporaryPath, $json + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
    $digest = (Get-FileHash -LiteralPath $temporaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText($temporaryDigestPath, $digest + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
    Publish-Sprint6AUiEvidencePair `
        -TemporaryEvidencePath $temporaryPath `
        -TemporaryDigestPath $temporaryDigestPath `
        -FinalEvidencePath $fullOutputPath `
        -FinalDigestPath $finalDigestPath `
        -AllowOverwrite:$Overwrite
    Assert-Sprint6ADeploymentEvidence `
        -RepositoryRoot $repoRoot `
        -EvidencePath $fullOutputPath `
        -BaseUrl $BaseUrl `
        -ExpectedDataState $ExpectedDataState `
        -AdminEmail $AdminEmail `
        -AdminPassword $AdminPassword | Out-Null
} finally {
    Remove-Item -LiteralPath $temporaryPath, $temporaryDigestPath -Force -ErrorAction SilentlyContinue
}

Write-Host "Retained Sprint 6A-UI deployment evidence: $fullOutputPath" -ForegroundColor Green
Write-Host "SHA-256 sidecar: $finalDigestPath"

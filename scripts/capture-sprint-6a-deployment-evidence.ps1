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

$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-6a-deployment-evidence-common.ps1")

function Publish-Sprint6AEvidencePair {
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
        throw "Retained deployment evidence already exists. Refusing to overwrite '$FinalEvidencePath' or its sidecar without -Overwrite."
    }

    $backupSuffix = ".backup-$([guid]::NewGuid().ToString('N'))"
    $evidenceBackup = "$FinalEvidencePath$backupSuffix"
    $digestBackup = "$FinalDigestPath$backupSuffix"
    $publishedEvidence = $false
    $publishedDigest = $false
    $backedUpEvidence = $false
    $backedUpDigest = $false
    $publishCompleted = $false
    try {
        if ($finalExists) {
            [IO.File]::Move($FinalEvidencePath, $evidenceBackup)
            $backedUpEvidence = $true
        }
        if ($digestExists) {
            [IO.File]::Move($FinalDigestPath, $digestBackup)
            $backedUpDigest = $true
        }
        [IO.File]::Move($TemporaryEvidencePath, $FinalEvidencePath)
        $publishedEvidence = $true
        [IO.File]::Move($TemporaryDigestPath, $FinalDigestPath)
        $publishedDigest = $true
        $publishCompleted = $true
    } catch {
        if ($publishedDigest -and (Test-Path -LiteralPath $FinalDigestPath)) {
            Remove-Item -LiteralPath $FinalDigestPath -Force
        }
        if ($publishedEvidence -and (Test-Path -LiteralPath $FinalEvidencePath)) {
            Remove-Item -LiteralPath $FinalEvidencePath -Force
        }
        if ($backedUpDigest -and (Test-Path -LiteralPath $digestBackup)) {
            [IO.File]::Move($digestBackup, $FinalDigestPath)
            $backedUpDigest = $false
        }
        if ($backedUpEvidence -and (Test-Path -LiteralPath $evidenceBackup)) {
            [IO.File]::Move($evidenceBackup, $FinalEvidencePath)
            $backedUpEvidence = $false
        }
        throw
    } finally {
        if ($publishCompleted) {
            foreach ($backup in @($evidenceBackup, $digestBackup)) {
                if (Test-Path -LiteralPath $backup) {
                    Remove-Item -LiteralPath $backup -Force
                }
            }
        }
    }
}

if ($SelfTest) {
    $missingConfig = [pscustomobject][ordered]@{ Cmd = "tessara-api" }
    $nullConfig = [pscustomobject][ordered]@{
        Cmd = "tessara-api"
        Entrypoint = $null
        User = $null
        WorkingDir = $null
    }
    $emptyConfig = [pscustomobject][ordered]@{
        Cmd = "tessara-api"
        Entrypoint = @()
        User = ""
        WorkingDir = ""
    }
    $explicitConfig = [pscustomobject][ordered]@{
        Cmd = @("tessara-api", "--serve")
        Entrypoint = @("/usr/bin/env")
        User = "10001"
        WorkingDir = "/app"
    }
    foreach ($config in @($missingConfig, $nullConfig, $emptyConfig)) {
        $entrypoint = ConvertTo-Sprint6AConfigSequenceJson -Config $config -PropertyName Entrypoint
        $user = ConvertTo-Sprint6AConfigScalar -Config $config -PropertyName User
        $workingDirectory = ConvertTo-Sprint6AConfigScalar -Config $config -PropertyName WorkingDir
        if ($entrypoint -cne "[]" -or $user -cne "" -or $workingDirectory -cne "") {
            throw "Self-test failed: missing, null, and empty Docker runtime defaults must normalize exactly."
        }
    }
    if ((ConvertTo-Sprint6AConfigSequenceJson -Config $missingConfig -PropertyName Cmd) -cne '"tessara-api"' -or
        (ConvertTo-Sprint6AConfigSequenceJson -Config $explicitConfig -PropertyName Cmd) -cne '["tessara-api","--serve"]' -or
        (ConvertTo-Sprint6AConfigSequenceJson -Config $explicitConfig -PropertyName Entrypoint) -cne '"/usr/bin/env"' -or
        (ConvertTo-Sprint6AConfigScalar -Config $explicitConfig -PropertyName User) -cne "10001" -or
        (ConvertTo-Sprint6AConfigScalar -Config $explicitConfig -PropertyName WorkingDir) -cne "/app") {
        throw "Self-test failed: non-empty Docker runtime identity was not preserved exactly."
    }
    $invalidScalarRejected = $false
    try {
        ConvertTo-Sprint6AConfigScalar `
            -Config ([pscustomobject]@{ User = @("10001") }) `
            -PropertyName User | Out-Null
    } catch {
        if ($_.Exception.Message -notmatch "must be a string") {
            throw
        }
        $invalidScalarRejected = $true
    }
    if (-not $invalidScalarRejected) {
        throw "Self-test failed: a non-string Docker scalar config value was accepted."
    }

    $timestampJson = '{"created_at":"2026-07-15T18:28:31.2182510+00:00","observed_at":"2026-07-15T18:28:31.8063730+00:00"}'
    $timestampDocument = ConvertFrom-Sprint6ADeploymentEvidenceJson -Json $timestampJson
    if ($timestampDocument.created_at -isnot [string] -or
        $timestampDocument.observed_at -isnot [string] -or
        [string]$timestampDocument.created_at -cne "2026-07-15T18:28:31.2182510+00:00" -or
        [string]$timestampDocument.observed_at -cne "2026-07-15T18:28:31.8063730+00:00" -or
        ($timestampDocument | ConvertTo-Json -Depth 5 -Compress) -cne $timestampJson) {
        throw "Self-test failed: deployment timestamps changed type, offset, precision, or bytes during JSON parsing."
    }

    $publishTestRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-deployment-evidence-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($publishTestRoot) | Out-Null
    try {
        $finalTestEvidence = Join-Path $publishTestRoot "evidence.json"
        $finalTestDigest = "$finalTestEvidence.sha256"
        $temporaryTestEvidence = Join-Path $publishTestRoot "new.json"
        $temporaryTestDigest = "$temporaryTestEvidence.sha256"
        [IO.File]::WriteAllText($finalTestEvidence, "old-evidence")
        [IO.File]::WriteAllText($finalTestDigest, "old-digest")
        [IO.File]::WriteAllText($temporaryTestEvidence, "new-evidence")
        [IO.File]::WriteAllText($temporaryTestDigest, "new-digest")
        $overwriteRejected = $false
        try {
            Publish-Sprint6AEvidencePair `
                -TemporaryEvidencePath $temporaryTestEvidence `
                -TemporaryDigestPath $temporaryTestDigest `
                -FinalEvidencePath $finalTestEvidence `
                -FinalDigestPath $finalTestDigest
        } catch {
            $overwriteRejected = $true
        }
        if (-not $overwriteRejected -or
            (Get-Content -LiteralPath $finalTestEvidence -Raw) -cne "old-evidence" -or
            (Get-Content -LiteralPath $finalTestDigest -Raw) -cne "old-digest") {
            throw "Self-test failed: retained evidence was overwritten without explicit authorization."
        }
        Publish-Sprint6AEvidencePair `
            -TemporaryEvidencePath $temporaryTestEvidence `
            -TemporaryDigestPath $temporaryTestDigest `
            -FinalEvidencePath $finalTestEvidence `
            -FinalDigestPath $finalTestDigest `
            -AllowOverwrite
        if ((Get-Content -LiteralPath $finalTestEvidence -Raw) -cne "new-evidence" -or
            (Get-Content -LiteralPath $finalTestDigest -Raw) -cne "new-digest") {
            throw "Self-test failed: validated evidence pair was not published together."
        }

        $rollbackSharedPath = Join-Path $publishTestRoot "rollback-shared"
        [IO.File]::WriteAllText($rollbackSharedPath, "replacement")
        $rollbackFailed = $false
        try {
            Publish-Sprint6AEvidencePair `
                -TemporaryEvidencePath $rollbackSharedPath `
                -TemporaryDigestPath $rollbackSharedPath `
                -FinalEvidencePath $finalTestEvidence `
                -FinalDigestPath $finalTestDigest `
                -AllowOverwrite
        } catch {
            $rollbackFailed = $true
        }
        if (-not $rollbackFailed -or
            (Get-Content -LiteralPath $finalTestEvidence -Raw) -cne "new-evidence" -or
            (Get-Content -LiteralPath $finalTestDigest -Raw) -cne "new-digest") {
            throw "Self-test failed: partial publication did not preserve the retained evidence pair."
        }
    } finally {
        Remove-Item -LiteralPath $publishTestRoot -Recurse -Force -ErrorAction SilentlyContinue
    }

    $seedRows = @(
        [pscustomobject]@{ name = "admin"; capabilities = @("admin:all") },
        [pscustomobject]@{
            name = "operator"
            capabilities = @(
                "components:read", "dashboards:read", "datasets:read", "forms:read",
                "hierarchy:read", "operations:view", "submissions:manage",
                "submissions:respond", "workflows:manage", "workflows:read"
            )
        },
        [pscustomobject]@{
            name = "respondent"
            capabilities = @("submissions:read_own", "submissions:respond")
        }
    )
    $seedContract = Get-Sprint6ASeedContract -SeedRoles $seedRows
    if ($seedContract.canonical_sha256 -cne "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600") {
        throw "Self-test failed: built-in seed canonical digest changed."
    }

    $expectedMigrations = @(
        [pscustomobject]@{ version = 1; file = "001_test.sql"; checksum_sha384 = "d" * 96 }
    )
    $databaseMigrations = @(
        [pscustomobject]@{ version = 1; success = $true; checksum_sha384 = "d" * 96 }
    )
    Assert-Sprint6AMigrationLedger `
        -DatabaseMigrations $databaseMigrations `
        -ExpectedMigrations $expectedMigrations
    $databaseMigrations[0].checksum_sha384 = "e" * 96
    $migrationMismatchRejected = $false
    try {
        Assert-Sprint6AMigrationLedger `
            -DatabaseMigrations $databaseMigrations `
            -ExpectedMigrations $expectedMigrations
    } catch {
        $migrationMismatchRejected = $true
    }
    if (-not $migrationMismatchRejected) {
        throw "Self-test failed: a changed database migration checksum was accepted."
    }

    $catalogEntries = @(
        "tessara.components", "tessara.dashboards", "tessara.datasets",
        "tessara.forms", "tessara.migration", "tessara.responses", "tessara.workflows"
    ) | ForEach-Object {
        [pscustomobject]@{ definition_id = $_; source_digest = "sha256:" + ("f" * 64) }
    }
    $immutableExpectedEntries = @($catalogEntries | ForEach-Object {
        [pscustomobject]@{ definition_id = $_.definition_id; source_digest = $_.source_digest }
    })
    $catalog = [pscustomobject]@{
        definition_count = 7
        source_count = 7
        projection_count = 7
        current_count = 7
        navigation_contribution_count = 6
        navigation_policy_count = 1
        policy_entries = @(1..6 | ForEach-Object {
            [pscustomobject]@{ contribution_id = "test.$_"; visible = $true; policy_order = $_ - 1 }
        })
        release_table_absent = $true
        instance_table_absent = $true
        entries = $catalogEntries
    }
    $inventory = [pscustomobject]@{
        schema_version = 1
        entries = @($catalogEntries | ForEach-Object {
            [pscustomobject]@{
                descriptor = [pscustomobject]@{ reserved_definition_id = $_.definition_id }
                source_digest = $_.source_digest
            }
        })
    }
    [void](Assert-Sprint6ACatalog `
        -DatabaseCatalog $catalog `
        -Inventory $inventory `
        -ExpectedEntries $immutableExpectedEntries)
    $inventory.schema_version = 999
    $invalidInventorySchemaRejected = $false
    try {
        [void](Assert-Sprint6ACatalog `
            -DatabaseCatalog $catalog `
            -Inventory $inventory `
            -ExpectedEntries $immutableExpectedEntries)
    } catch {
        $invalidInventorySchemaRejected = $true
    }
    if (-not $invalidInventorySchemaRejected) {
        throw "Self-test failed: a non-v1 inventory API response was accepted."
    }
    $inventory.schema_version = 1
    $catalog.entries[0].source_digest = "sha256:" + ("0" * 64)
    $inventory.entries[0].source_digest = "sha256:" + ("0" * 64)
    $changedCatalogDigestRejected = $false
    try {
        [void](Assert-Sprint6ACatalog `
            -DatabaseCatalog $catalog `
            -Inventory $inventory `
            -ExpectedEntries $immutableExpectedEntries)
    } catch {
        $changedCatalogDigestRejected = $true
    }
    if (-not $changedCatalogDigestRejected) {
        throw "Self-test failed: catalog provenance outside the immutable fixture digest was accepted."
    }
    $immutableCatalog = @(Get-Sprint6AExpectedCatalogEntries -RepositoryRoot $repoRoot)
    if ($immutableCatalog.Count -ne 7) {
        throw "Self-test failed: immutable transition fixture inventory changed."
    }
    $fixtureProofRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-transition-fixture-pins-$([guid]::NewGuid().ToString('N'))"
    try {
        $fixtureProofDirectory = Join-Path $fixtureProofRoot "crates/tessara-module-contract/tests/fixtures"
        [IO.Directory]::CreateDirectory($fixtureProofDirectory) | Out-Null
        $repositoryFixtureDirectory = Join-Path $repoRoot "crates/tessara-module-contract/tests/fixtures"
        foreach ($fixtureFile in Get-ChildItem -LiteralPath $repositoryFixtureDirectory -Filter "transition-*-v1.json*" -File) {
            Copy-Item -LiteralPath $fixtureFile.FullName -Destination $fixtureProofDirectory
        }

        $mutatedSourcePath = Join-Path $fixtureProofDirectory "transition-forms-v1.json"
        $mutatedSidecarPath = "$mutatedSourcePath.sha256"
        $originalSourceBytes = [IO.File]::ReadAllBytes($mutatedSourcePath)
        $mutatedSourceBytes = [byte[]]::new($originalSourceBytes.Length + 1)
        [Array]::Copy($originalSourceBytes, 0, $mutatedSourceBytes, 0, $originalSourceBytes.Length - 1)
        $mutatedSourceBytes[$mutatedSourceBytes.Length - 2] = 32
        $mutatedSourceBytes[$mutatedSourceBytes.Length - 1] = 10
        [IO.File]::WriteAllBytes($mutatedSourcePath, $mutatedSourceBytes)
        $mutatedDigest = "sha256:" + (Get-FileHash -LiteralPath $mutatedSourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::WriteAllText($mutatedSidecarPath, "$mutatedDigest`n", [Text.UTF8Encoding]::new($false))
        $coordinatedMutationRejected = $false
        try {
            [void](Get-Sprint6AExpectedCatalogEntries -RepositoryRoot $fixtureProofRoot)
        } catch {
            $coordinatedMutationRejected = $true
        }
        if (-not $coordinatedMutationRejected) {
            throw "Self-test failed: changing a transition fixture and its sidecar together bypassed the independent digest pin."
        }

        Copy-Item `
            -LiteralPath (Join-Path $repositoryFixtureDirectory "transition-forms-v1.json") `
            -Destination $mutatedSourcePath `
            -Force
        $formsDigest = [string]$script:Sprint6AExpectedTransitionFixtureDigests["transition-forms-v1.json"]
        [IO.File]::WriteAllText($mutatedSidecarPath, "$formsDigest`r`n", [Text.UTF8Encoding]::new($false))
        $nonCanonicalSidecarRejected = $false
        try {
            [void](Get-Sprint6AExpectedCatalogEntries -RepositoryRoot $fixtureProofRoot)
        } catch {
            $nonCanonicalSidecarRejected = $true
        }
        if (-not $nonCanonicalSidecarRejected) {
            throw "Self-test failed: a CRLF transition digest sidecar bypassed the exact-byte check."
        }
    } finally {
        Remove-Item -LiteralPath $fixtureProofRoot -Recurse -Force -ErrorAction SilentlyContinue
    }

    $sample = [pscustomobject]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-6a.deployment-evidence"
        snapshot = [pscustomobject]@{
            base_url = "http://127.0.0.1:8080"
            source = [pscustomobject]@{ commit = "a" * 40; tree = "b" * 40 }
            release_image = [pscustomobject]@{ image_id = "sha256:" + ("c" * 64) }
            database_runtime = [pscustomobject]@{ database_user = "tessara" }
            built_in_seed = [pscustomobject]@{
                version = "sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68"
                canonical_sha256 = "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600"
            }
            data = [pscustomobject]@{ state = "fresh" }
        }
    }
    Assert-Sprint6ADeploymentEvidenceDocument `
        -Evidence $sample `
        -ExpectedDataState fresh `
        -BaseUrl "http://127.0.0.1:8080"
    $rejected = $false
    try {
        Assert-Sprint6ADeploymentEvidenceDocument `
            -Evidence $sample `
            -ExpectedDataState upgraded `
            -BaseUrl "http://127.0.0.1:8080"
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Self-test failed: fresh evidence was accepted as the retired upgraded state."
    }
    $sample.snapshot.database_runtime.database_user = 7
    $rejected = $false
    try {
        Assert-Sprint6ADeploymentEvidenceDocument `
            -Evidence $sample `
            -ExpectedDataState fresh `
            -BaseUrl "http://127.0.0.1:8080"
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Self-test failed: a non-string deployment database user was accepted."
    }
    $sample.snapshot.database_runtime.database_user = "tessara user"
    $rejected = $false
    try {
        Assert-Sprint6ADeploymentEvidenceDocument `
            -Evidence $sample `
            -ExpectedDataState fresh `
            -BaseUrl "http://127.0.0.1:8080"
    } catch {
        $rejected = $true
    }
    if (-not $rejected) {
        throw "Self-test failed: an invalid deployment database user was accepted."
    }
    Write-Host "Sprint 6A deployment-evidence seed/migration/catalog/schema/interchange self-test passed." -ForegroundColor Green
    exit 0
}

if ([string]::IsNullOrWhiteSpace($ExpectedDataState)) {
    throw "-ExpectedDataState fresh is required. The value is verified against database history; it is not self-attested."
}
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = "artifacts/sprint-6a/deployment-$ExpectedDataState.json"
}
$fullPath = Resolve-Sprint6ARepositoryPath -RepositoryRoot $repoRoot -Path $OutputPath
$finalDigestPath = "$fullPath.sha256"
if (((Test-Path -LiteralPath $fullPath -PathType Leaf) -or
    (Test-Path -LiteralPath $finalDigestPath -PathType Leaf)) -and -not $Overwrite) {
    throw "Retained deployment evidence already exists. Refusing to overwrite '$fullPath' or its sidecar without -Overwrite."
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
$evidence = [pscustomobject][ordered]@{
    schema_version = 1
    evidence_kind = "tessara.sprint-6a.deployment-evidence"
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    snapshot = $snapshot
}
$directory = Split-Path -Parent $fullPath
[IO.Directory]::CreateDirectory($directory) | Out-Null
$temporaryPath = Join-Path $directory ".$(Split-Path -Leaf $fullPath).$([guid]::NewGuid().ToString('N')).tmp"
$temporaryDigestPath = "$temporaryPath.sha256"
try {
    $json = $evidence | ConvertTo-Json -Depth 30
    [IO.File]::WriteAllText($temporaryPath, $json + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
    $digest = (Get-FileHash -LiteralPath $temporaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText($temporaryDigestPath, $digest + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))

    Assert-Sprint6ADeploymentEvidence `
        -RepositoryRoot $repoRoot `
        -EvidencePath $temporaryPath `
        -BaseUrl $BaseUrl `
        -ExpectedDataState $ExpectedDataState `
        -AdminEmail $AdminEmail `
        -AdminPassword $AdminPassword | Out-Null
    Publish-Sprint6AEvidencePair `
        -TemporaryEvidencePath $temporaryPath `
        -TemporaryDigestPath $temporaryDigestPath `
        -FinalEvidencePath $fullPath `
        -FinalDigestPath $finalDigestPath `
        -AllowOverwrite:$Overwrite
} finally {
    Remove-Item -LiteralPath $temporaryPath, $temporaryDigestPath -Force -ErrorAction SilentlyContinue
}
Write-Host "Retained deployment evidence: $fullPath" -ForegroundColor Green
Write-Host "SHA-256 sidecar: $finalDigestPath"

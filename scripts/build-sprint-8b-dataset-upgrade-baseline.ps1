[CmdletBinding()]
param(
    [string]$OutputTag = "tessara-sprint-8b-datasets-upgrade-baseline:latest",
    [ValidatePattern('^0\.9\.0$')][string]$BaselineRelease = "0.9.0",
    [string]$MetadataOutputPath = "target/sprint-8b-upgrade/dataset-baseline-release.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$dockerfile = Join-Path $repoRoot "deploy/sprint-8b/Dockerfile.dataset-upgrade-baseline"
$fixturePath = Join-Path $repoRoot "deploy/sprint-8b/fixtures/upgrade-fixture-contract.json"
$manifestPath = Join-Path $repoRoot "crates/tessara-dataset-module/manifest.json"
$migrationPath = Join-Path $repoRoot "crates/tessara-dataset-module/migrations/001_dataset_module.sql"
$datasetCargo = Join-Path $repoRoot "crates/tessara-dataset-module/Cargo.toml"
$datasetSource = Join-Path $repoRoot "crates/tessara-dataset-module/src/lib.rs"
$datasetSourceRoot = Join-Path $repoRoot "crates/tessara-dataset-module/src"
$baselineFeature = "sprint-8b-upgrade-baseline"
$candidateRelease = "1.0.0"
$datasetContractId = "tessara.datasets.dataset-major-line"
$datasetContractVersion = "2.0.0"
$fixtureKind = "source-built-prior-compatible-release-v1"
$receiptContract = "tessara.sprint-8b.dataset-prior-compatible-release"

function Resolve-OutputPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    return [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Get-RelativeRepositoryPath([string]$Path) {
    return [IO.Path]::GetRelativePath($repoRoot, $Path).Replace('\', '/')
}

function Get-FileSha256([string]$Path) {
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Assert-ExactSequence {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Actual,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Expected,
        [Parameter(Mandatory)][string]$Label
    )
    if ($Actual.Count -ne $Expected.Count) {
        throw "$Label expected $($Expected.Count) values, found $($Actual.Count)."
    }
    for ($index = 0; $index -lt $Expected.Count; $index++) {
        if ([string]$Actual[$index] -cne $Expected[$index]) {
            throw "$Label[$index] expected '$($Expected[$index])', found '$($Actual[$index])'."
        }
    }
}

function Assert-SourceFixture {
    foreach ($requiredPath in @(
        $dockerfile,
        $fixturePath,
        $manifestPath,
        $migrationPath,
        $datasetCargo,
        $datasetSource
    )) {
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Dataset upgrade fixture path is missing: $requiredPath"
        }
    }

    $dockerfileText = Get-Content -LiteralPath $dockerfile -Raw
    $cargoText = Get-Content -LiteralPath $datasetCargo -Raw
    $sourceText = Get-Content -LiteralPath $datasetSource -Raw
    foreach ($fragment in @(
        'test "$TESSARA_DATASET_RELEASE" = "0.9.0"',
        'cargo build --release --locked -p tessara-dataset-module',
        '--features sprint-8b-upgrade-baseline',
        'com.tessara.module-release="$TESSARA_DATASET_RELEASE"',
        'com.tessara.upgrade.fixture="source-built-prior-compatible-release-v1"',
        'COPY --from=builder /tmp/dataset-module /usr/local/bin/dataset-module',
        'ENTRYPOINT ["/usr/local/bin/dataset-module"]'
    )) {
        if (-not $dockerfileText.Contains($fragment)) {
            throw "Source-built Dataset baseline Dockerfile omits '$fragment'."
        }
    }
    if ($dockerfileText -match '(?im)^\s*FROM\s+.*(?:TESSARA_DATASET_IMAGE|tessara-sprint-8b-datasets)(?:\s|$)') {
        throw "Dataset baseline Dockerfile must build from source instead of relabeling the 1.0.0 candidate image."
    }
    foreach ($fragment in @(
        '[features]',
        'default = []',
        'sprint-8b-upgrade-baseline = []'
    )) {
        if (-not $cargoText.Contains($fragment)) {
            throw "Dataset Cargo feature declaration omits '$fragment'."
        }
    }
    foreach ($fragment in @(
        'pub const CURRENT_MODULE_RELEASE_VERSION: &str = "1.0.0";',
        'pub const PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION: &str = "0.9.0";',
        '#[cfg(feature = "sprint-8b-upgrade-baseline")]',
        'pub const MODULE_RELEASE_VERSION: &str = PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION;',
        'manifest.release_version = MODULE_RELEASE_VERSION'
    )) {
        if (-not $sourceText.Contains($fragment)) {
            throw "Dataset source-built release projection omits '$fragment'."
        }
    }

    $featureReferences = @(
        Get-ChildItem -LiteralPath $datasetSourceRoot -Recurse -File -Filter "*.rs" |
            Select-String -SimpleMatch $baselineFeature
    )
    if ($featureReferences.Count -ne 3 -or
        @($featureReferences | Where-Object { $_.Path -cne $datasetSource }).Count -ne 0) {
        throw "The Dataset baseline feature must be isolated to the release constant and its exact lib.rs test."
    }

    $candidateManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 100
    if ([string]$candidateManifest.definition_id -cne "tessara.datasets" -or
        [string]$candidateManifest.release_version -cne $candidateRelease) {
        throw "The tracked Dataset manifest must remain the real tessara.datasets 1.0.0 candidate."
    }
    $majorLineContracts = @(
        $candidateManifest.provided_contracts |
            Where-Object { [string]$_.id -ceq $datasetContractId }
    )
    if ($majorLineContracts.Count -ne 1 -or
        [string]$majorLineContracts[0].version -cne $datasetContractVersion) {
        throw "The Dataset candidate and prior-compatible projection must expose exactly Dataset v2."
    }

    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json -Depth 100
    if ([int]$fixture.schema_version -ne 1 -or
        [string]$fixture.contract -cne "tessara.sprint-8b.dataset-upgrade-fixture") {
        throw "Dataset upgrade fixture identity is invalid."
    }
    Assert-ExactSequence -Actual @($fixture.sequence) `
        -Expected @("0.9.0", "1.0.0", "0.9.0", "1.0.0") -Label "upgrade sequence"
    if ([string]$fixture.transition.owner -cne "tessara.datasets" -or
        [string]$fixture.transition.mechanism -cne "resolved_one_owner_blueprint_delta" -or
        [string]$fixture.transition.intended_release -cne $candidateRelease) {
        throw "Dataset upgrade transition must be an exact one-owner delta restored to 1.0.0."
    }
    $artifact = $fixture.prior_compatible_artifact
    $expectedArtifact = [ordered]@{
        release = $BaselineRelease
        cargo_package = "tessara-dataset-module"
        cargo_feature = $baselineFeature
        dockerfile = "deploy/sprint-8b/Dockerfile.dataset-upgrade-baseline"
        builder = "scripts/build-sprint-8b-dataset-upgrade-baseline.ps1"
        executable = "/usr/local/bin/dataset-module"
        fixture_kind = $fixtureKind
        manifest_source = "crates/tessara-dataset-module/manifest.json"
        manifest_projection = "release_version_only"
        metadata_receipt_contract = $receiptContract
    }
    Assert-ExactSequence -Actual @($artifact.PSObject.Properties.Name) `
        -Expected @($expectedArtifact.Keys) -Label "prior-compatible artifact fields"
    foreach ($field in $expectedArtifact.Keys) {
        if ([string]$artifact.$field -cne [string]$expectedArtifact[$field]) {
            throw "Dataset prior-compatible artifact '$field' is not exact."
        }
    }

    Assert-ExactSequence -Actual @($fixture.dataset_releases.PSObject.Properties.Name) `
        -Expected @("0.9.0", "1.0.0") -Label "Dataset release inventory"
    foreach ($release in @("0.9.0", "1.0.0")) {
        $releaseDeclaration = $fixture.dataset_releases.$release
        if ([string]$releaseDeclaration.contract -cne $datasetContractId -or
            [string]$releaseDeclaration.contract_version -cne $datasetContractVersion -or
            $releaseDeclaration.source_built -ne $true) {
            throw "Dataset $release must be a real source-built Dataset v2 release."
        }
    }

    $schema = $fixture.schema_compatibility
    if ([string]$schema.mode -cne "reads_additive_1_0_0_schema_after_rollback" -or
        [string]$schema.migration_path -cne "crates/tessara-dataset-module/migrations/001_dataset_module.sql" -or
        [string]$schema.baseline_feature_scope -cne "release_identity_only" -or
        [string]$schema.rollback_reader_contract -cne "same_source_and_storage_read_paths") {
        throw "Dataset rollback schema compatibility is not exact."
    }
    $expectedAdditiveObjects = @(
        "dataset_sync_partitions",
        "dataset_sync_attempts",
        "dataset_sync_staged_changes",
        "dataset_imported_responses",
        "dataset_imported_response_values",
        "dataset_materialization_receipts"
    )
    Assert-ExactSequence -Actual @($schema.required_additive_objects) `
        -Expected $expectedAdditiveObjects -Label "additive Dataset schema objects"
    $migrationText = Get-Content -LiteralPath $migrationPath -Raw
    foreach ($object in $expectedAdditiveObjects) {
        if ($migrationText -cnotmatch "(?m)^CREATE TABLE $([regex]::Escape($object))\s*\(") {
            throw "Dataset additive schema object '$object' is absent from the compiled migration."
        }
    }

    Assert-ExactSequence -Actual @($fixture.fixed_dependencies.PSObject.Properties.Name) `
        -Expected @("tessara.components", "tessara.dashboards") -Label "fixed dependency owners"
    if ([string]$fixture.fixed_dependencies.'tessara.components' -cne "1.1.0" -or
        [string]$fixture.fixed_dependencies.'tessara.dashboards' -cne "3.0.2") {
        throw "Component 1.1.0 and Dashboard 3.0.2 must remain fixed."
    }
    Assert-ExactSequence -Actual @($fixture.preserved) `
        -Expected @("dataset_state", "provider_route", "typed_resource_identity", "navigation_identity") `
        -Label "preserved Dataset state"
    Assert-ExactSequence -Actual @($fixture.unrelated_unchanged) `
        -Expected @("images", "containers", "restart_counts", "owner_data", "navigation", "availability") `
        -Label "unrelated unchanged state"
}

function Publish-MetadataReceipt {
    param(
        [Parameter(Mandatory)][object]$Document,
        [Parameter(Mandatory)][string]$OutputPath
    )
    $sidecarPath = "$OutputPath.sha256"
    if ((Test-Path -LiteralPath $OutputPath) -or (Test-Path -LiteralPath $sidecarPath)) {
        throw "Dataset baseline metadata already exists and cannot be overwritten: $OutputPath"
    }
    $directory = Split-Path -Parent $OutputPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    $temporary = Join-Path $directory ".$([IO.Path]::GetFileName($OutputPath)).$([guid]::NewGuid().ToString('N')).tmp"
    $temporarySidecar = "$temporary.sha256"
    $publishedDocument = $false
    try {
        [IO.File]::WriteAllText(
            $temporary,
            (($Document | ConvertTo-Json -Depth 100) + "`n"),
            [Text.UTF8Encoding]::new($false)
        )
        $digest = Get-FileSha256 -Path $temporary
        [IO.File]::WriteAllText(
            $temporarySidecar,
            "$digest`n",
            [Text.UTF8Encoding]::new($false)
        )
        Move-Item -LiteralPath $temporary -Destination $OutputPath
        $publishedDocument = $true
        Move-Item -LiteralPath $temporarySidecar -Destination $sidecarPath
        if ((Get-FileSha256 -Path $OutputPath) -cne $digest -or
            (Get-Content -LiteralPath $sidecarPath -Raw).Trim() -cne $digest) {
            throw "Published Dataset baseline metadata receipt failed SHA-256 verification."
        }
        return [pscustomobject]@{ path = $OutputPath; sha256 = $digest }
    } catch {
        if ($publishedDocument) {
            Remove-Item -LiteralPath $OutputPath -Force -ErrorAction SilentlyContinue
        }
        Remove-Item -LiteralPath $sidecarPath -Force -ErrorAction SilentlyContinue
        throw
    } finally {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temporarySidecar -Force -ErrorAction SilentlyContinue
    }
}

function Test-MetadataReceiptPublisher {
    $selfTestPath = Join-Path $repoRoot "target/.dataset-baseline-receipt-self-test-$([guid]::NewGuid().ToString('N')).json"
    try {
        $published = Publish-MetadataReceipt -Document ([ordered]@{
            schema_version = 1
            receipt_contract = $receiptContract
            self_test = $true
        }) -OutputPath $selfTestPath
        if ([string]$published.path -cne $selfTestPath -or
            [string]$published.sha256 -cnotmatch '^[0-9a-f]{64}$') {
            throw "Dataset baseline metadata publisher returned an invalid identity."
        }
        $overwriteRejected = $false
        try {
            Publish-MetadataReceipt -Document ([ordered]@{
                schema_version = 1
                receipt_contract = $receiptContract
                self_test = $false
            }) -OutputPath $selfTestPath | Out-Null
        } catch {
            $overwriteRejected = $true
        }
        if (-not $overwriteRejected) {
            throw "Dataset baseline metadata publisher admitted an overwrite."
        }
    } finally {
        Remove-Item -LiteralPath $selfTestPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath "$selfTestPath.sha256" -Force -ErrorAction SilentlyContinue
    }
}

Assert-SourceFixture
if ($SelfTest) {
    Test-MetadataReceiptPublisher
    Write-Host "Sprint 8B source-built prior-compatible Dataset release self-test passed."
    return
}

if ($OutputTag -notmatch '^[^\s@]+:[^\s@]+$') {
    throw "OutputTag must be one local mutable image tag; the receipt records its immutable image ID."
}
$metadataPath = Resolve-OutputPath $MetadataOutputPath
$generatedManifestPath = [IO.Path]::ChangeExtension($metadataPath, "manifest.json")
foreach ($retainedPath in @($metadataPath, "$metadataPath.sha256", $generatedManifestPath)) {
    if (Test-Path -LiteralPath $retainedPath) {
        throw "Dataset baseline retained output already exists and cannot be overwritten: $retainedPath"
    }
}

$generatedManifestCreated = $false
Push-Location $repoRoot
try {
    $sourceStatus = @(& git status --porcelain=v1 --untracked-files=all)
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect the Dataset baseline source worktree."
    }
    if ($sourceStatus.Count -ne 0) {
        throw "Dataset baseline release freezing requires a clean source worktree."
    }
    $sourceCommit = (& git rev-parse HEAD).Trim()
    $sourceTree = (& git rev-parse 'HEAD^{tree}').Trim()
    if ($LASTEXITCODE -ne 0 -or
        $sourceCommit -cnotmatch '^[0-9a-f]{40,64}$' -or
        $sourceTree -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Could not resolve exact Dataset baseline source identity."
    }

    & docker build `
        --file $dockerfile `
        --build-arg "TESSARA_DATASET_RELEASE=$BaselineRelease" `
        --build-arg "TESSARA_SOURCE_COMMIT=$sourceCommit" `
        --build-arg "TESSARA_SOURCE_TREE=$sourceTree" `
        --build-arg "TESSARA_SOURCE_DIRTY=false" `
        --tag $OutputTag `
        .
    if ($LASTEXITCODE -ne 0) {
        throw "Source-built prior-compatible Dataset image build failed."
    }

    $inspection = @(& docker image inspect $OutputTag | ConvertFrom-Json -Depth 100)
    if ($LASTEXITCODE -ne 0 -or $inspection.Count -ne 1) {
        throw "Could not inspect the source-built prior-compatible Dataset image."
    }
    $imageId = [string]$inspection[0].Id
    if ($imageId -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Dataset baseline image does not have one immutable image ID."
    }
    $labels = $inspection[0].Config.Labels
    if ([string]$labels.'com.tessara.module-definition' -cne 'tessara.datasets' -or
        [string]$labels.'com.tessara.module-release' -cne $BaselineRelease -or
        [string]$labels.'com.tessara.upgrade.fixture' -cne $fixtureKind -or
        [string]$labels.'org.opencontainers.image.revision' -cne $sourceCommit -or
        [string]$labels.'com.tessara.source-tree' -cne $sourceTree -or
        [string]$labels.'com.tessara.source-dirty' -cne 'false') {
        throw "Dataset baseline OCI release/source identity is incomplete."
    }
    Assert-ExactSequence -Actual @($inspection[0].Config.Entrypoint) `
        -Expected @('/usr/local/bin/dataset-module') -Label "Dataset baseline entrypoint"

    $binaryHashOutput = @(
        & docker run --rm --entrypoint sha256sum $OutputTag /usr/local/bin/dataset-module
    )
    if ($LASTEXITCODE -ne 0 -or $binaryHashOutput.Count -ne 1 -or
        [string]$binaryHashOutput[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
        throw "Could not capture the prior-compatible Dataset executable identity."
    }
    $executableSha256 = $Matches.hash

    $candidateManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 100
    $baselineManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 100
    $baselineManifest.release_version = $BaselineRelease
    $normalizedBaseline = $baselineManifest | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $normalizedBaseline.release_version = $candidateManifest.release_version
    if (($normalizedBaseline | ConvertTo-Json -Depth 100 -Compress) -cne
        ($candidateManifest | ConvertTo-Json -Depth 100 -Compress)) {
        throw "The generated Dataset 0.9.0 manifest may differ from 1.0.0 only by release_version."
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $generatedManifestPath)) | Out-Null
    [IO.File]::WriteAllText(
        $generatedManifestPath,
        (($baselineManifest | ConvertTo-Json -Depth 100) + "`n"),
        [Text.UTF8Encoding]::new($false)
    )
    $generatedManifestCreated = $true

    $manifestDigestOutput = @(
        & cargo run --quiet --locked --offline -p tessara-supervisor --bin tessara-compose -- `
            manifest-digest $generatedManifestPath
    )
    if ($LASTEXITCODE -ne 0) {
        throw "Could not compute the canonical Dataset baseline manifest digest."
    }
    $manifestDigest = [string]($manifestDigestOutput | Select-Object -Last 1)
    if ($manifestDigest -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Dataset baseline manifest digest is invalid: '$manifestDigest'."
    }

    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json -Depth 100
    $metadata = [ordered]@{
        schema_version = 1
        receipt_contract = $receiptContract
        generated_at = [DateTimeOffset]::UtcNow.ToString("O")
        release_identity = [ordered]@{
            definition_id = "tessara.datasets"
            version = $BaselineRelease
            manifest_digest = $manifestDigest
            runtime_image = $imageId
            migration_image = $imageId
            image_reference = $OutputTag
            executable_sha256 = $executableSha256
        }
        source_identity = [ordered]@{
            commit = $sourceCommit
            tree = $sourceTree
            dirty = $false
            cargo_package = "tessara-dataset-module"
            feature = $baselineFeature
            dockerfile_sha256 = Get-FileSha256 -Path $dockerfile
            cargo_lock_sha256 = Get-FileSha256 -Path (Join-Path $repoRoot "Cargo.lock")
        }
        compatibility = [ordered]@{
            feature_scope = "release_identity_only"
            tracked_candidate_release = $candidateRelease
            dataset_contract_id = $datasetContractId
            dataset_contract_version = $datasetContractVersion
            rollback_schema_expectation = "reads_additive_1_0_0_schema_after_rollback"
            rollback_reader_contract = "same_source_and_storage_read_paths"
            migration_path = Get-RelativeRepositoryPath -Path $migrationPath
            migration_sha256 = Get-FileSha256 -Path $migrationPath
            required_additive_objects = @($fixture.schema_compatibility.required_additive_objects)
        }
        fixture_identity = [ordered]@{
            contract_path = Get-RelativeRepositoryPath -Path $fixturePath
            contract_sha256 = Get-FileSha256 -Path $fixturePath
            fixture_kind = $fixtureKind
        }
        manifest_path = Get-RelativeRepositoryPath -Path $generatedManifestPath
    }
    $published = Publish-MetadataReceipt -Document $metadata -OutputPath $metadataPath
    $published.path
} catch {
    if ($generatedManifestCreated -and -not (Test-Path -LiteralPath $metadataPath)) {
        Remove-Item -LiteralPath $generatedManifestPath -Force -ErrorAction SilentlyContinue
    }
    throw
} finally {
    Pop-Location
}

[CmdletBinding()]
param(
    [string]$OutputTag = "tessara-sprint-8c-responses-upgrade-baseline:latest",
    [ValidatePattern('^0\.9\.0$')][string]$BaselineRelease = "0.9.0",
    [string]$MetadataOutputPath = "target/sprint-8c-response-upgrade/response-baseline-release.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$dockerfile = Join-Path $repoRoot "deploy/sprint-8c/Dockerfile.response-upgrade-baseline"
$candidateDockerfile = Join-Path $repoRoot "deploy/sprint-8c/Dockerfile.response"
$fixturePath = Join-Path $repoRoot "deploy/sprint-8c/fixtures/upgrade-fixture-contract.json"
$patchPath = Join-Path $repoRoot "deploy/sprint-8c/baselines/response-0.9.0.patch"
$candidateManifestPath = Join-Path $repoRoot "crates/tessara-response-module/manifest.json"
$candidateCargoPath = Join-Path $repoRoot "crates/tessara-response-module/Cargo.toml"
$candidateSourcePath = Join-Path $repoRoot "crates/tessara-response-module/src/lib.rs"
$candidateMigrationPath = Join-Path $repoRoot "crates/tessara-response-module/migrations/001_response_module.sql"
$candidateMainPath = Join-Path $repoRoot "crates/tessara-response-module/src/main.rs"
$candidateOperationalPath = Join-Path $repoRoot "crates/tessara-response-module/src/operational.rs"
$candidateProviderClientPath = Join-Path $repoRoot "crates/tessara-response-module/src/provider_client.rs"
$candidateReversePath = Join-Path $repoRoot "crates/tessara-response-module/src/reverse_provider.rs"
$baselineCommit = "13eb6ffaa9479fa71f4270edb049d079495d1a79"
$baselineTree = "c99cf7e156dd157a705b8354d706f9b8be7baeb9"
$sourceModel = "authenticated-git-snapshot-plus-dedicated-patch"
$candidateRelease = "1.0.0"
$fixtureKind = "authenticated-independent-source-release-v2"
$receiptContract = "tessara.sprint-8c.response-prior-compatible-release"
$manifestProjection = "independent-source-behavioral-compatibility"
$allowedManifestVariations = @("release_version", "browser_lifecycle", "assets")
$resourceContractId = "tessara.responses.response"
$resourceContractVersion = "2.0.0"
$lifecycleContractId = "tessara.responses.response-lifecycle"
$lifecycleContractVersion = "2.0.0"
$responseMigrationSha256 = "958c7928d90950f588fa943992430e81fabcfbe23ac52c1ec453291b3b811dbd"

function Resolve-OutputPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) { return [IO.Path]::GetFullPath($Path) }
    [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Get-RelativeRepositoryPath([string]$Path) {
    [IO.Path]::GetRelativePath($repoRoot, $Path).Replace('\', '/')
}

function Get-FileSha256([string]$Path) {
    (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Get-TextSha256([string]$Text) {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
        ([BitConverter]::ToString($algorithm.ComputeHash($bytes))).Replace("-", "").ToLowerInvariant()
    } finally { $algorithm.Dispose() }
}

function ConvertTo-StableJson([AllowNull()]$Value) {
    if ($null -eq $Value) { return "null" }
    $Value | ConvertTo-Json -Depth 100 -Compress
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

function Get-SourceProjectionIdentity([string]$SourceRoot) {
    $records = @(
        Get-ChildItem -LiteralPath $SourceRoot -Recurse -File | ForEach-Object {
            [pscustomobject]@{
                path = [IO.Path]::GetRelativePath($SourceRoot, $_.FullName).Replace('\', '/')
                sha256 = Get-FileSha256 -Path $_.FullName
            }
        } | Sort-Object path
    )
    $projection = (($records | ForEach-Object { "$($_.path)`t$($_.sha256)" }) -join "`n") + "`n"
    [pscustomobject]@{
        sha256 = Get-TextSha256 -Text $projection
        file_count = $records.Count
    }
}

function Get-ManifestBehaviorProjection([object]$Manifest) {
    $copy = $Manifest | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    foreach ($property in $allowedManifestVariations) {
        $copy.PSObject.Properties.Remove($property)
    }
    $copy
}

function Assert-ManifestBehaviorCompatibility {
    param([Parameter(Mandatory)]$Baseline, [Parameter(Mandatory)]$Candidate)
    $baselineProjection = Get-ManifestBehaviorProjection -Manifest $Baseline
    $candidateProjection = Get-ManifestBehaviorProjection -Manifest $Candidate
    if ((ConvertTo-StableJson $baselineProjection) -cne
        (ConvertTo-StableJson $candidateProjection)) {
        throw "Independent Response 0.9.0 source is not behaviorally Manifest-compatible with 1.0.0."
    }
    Get-TextSha256 -Text ((ConvertTo-StableJson $candidateProjection) + "`n")
}

function Assert-BaselineProvenance([object]$Identity) {
    if ([string]$Identity.source_model -cne $sourceModel -or
        [string]$Identity.candidate_source_dependency -cne "none" -or
        [string]$Identity.commit -cne $baselineCommit -or
        [string]$Identity.tree -cne $baselineTree -or
        [string]$Identity.patch_sha256 -cne (Get-FileSha256 -Path $patchPath) -or
        [string]$Identity.materialized_source_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [int]$Identity.materialized_file_count -le 0 -or
        [string]$Identity.marker_sha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw "Response 0.9.0 baseline source provenance is not exact and authenticated."
    }
}

function Assert-SourceFixture {
    foreach ($requiredPath in @(
        $dockerfile, $candidateDockerfile, $fixturePath, $patchPath, $candidateManifestPath,
        $candidateMigrationPath, $candidateCargoPath, $candidateSourcePath, $candidateMainPath,
        $candidateOperationalPath, $candidateProviderClientPath, $candidateReversePath
    )) {
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Response upgrade fixture path is missing: $requiredPath"
        }
    }
    $dockerfileText = Get-Content -LiteralPath $dockerfile -Raw
    foreach ($fragment in @(
        'test "$TESSARA_RESPONSE_RELEASE" = "0.9.0"',
        'cargo build --release --locked -p tessara-response-module',
        'ARG TESSARA_BASELINE_SOURCE_COMMIT',
        'ARG TESSARA_BASELINE_SOURCE_TREE',
        'ARG TESSARA_BASELINE_PATCH_SHA256',
        'ARG TESSARA_BASELINE_MATERIALIZED_SOURCE_SHA256',
        'com.tessara.upgrade.fixture="authenticated-independent-source-release-v2"',
        'COPY --from=builder /tmp/response-module /usr/local/bin/response-module',
        'ENTRYPOINT ["/usr/local/bin/response-module"]'
    )) {
        if (-not $dockerfileText.Contains($fragment)) {
            throw "Independent Response baseline Dockerfile omits '$fragment'."
        }
    }
    if ($dockerfileText.Contains('sprint-8c-upgrade-baseline') -or
        $dockerfileText.Contains('sprint-8c-validation-faults') -or
        $dockerfileText -match '(?im)^\s*FROM\s+.*(?:TESSARA_RESPONSE_IMAGE|tessara-sprint-8c-responses)(?:\s|$)') {
        throw "Response baseline Dockerfile must build only the independently materialized source."
    }
    $candidateCargoText = Get-Content -LiteralPath $candidateCargoPath -Raw
    $candidateSourceText = Get-Content -LiteralPath $candidateSourcePath -Raw
    if ($candidateCargoText.Contains('sprint-8c-upgrade-baseline') -or
        $candidateSourceText.Contains('sprint-8c-upgrade-baseline') -or
        -not $candidateSourceText.Contains('pub const MODULE_RELEASE_VERSION: &str = "1.0.0";')) {
        throw "The current Response candidate retains a forbidden baseline relabeling shortcut."
    }
    & git cat-file -e "${baselineCommit}^{commit}"
    if ($LASTEXITCODE -ne 0) { throw "Pinned Response 0.9.0 base commit is unavailable." }
    $resolvedTree = (& git rev-parse "${baselineCommit}^{tree}").Trim()
    if ($LASTEXITCODE -ne 0 -or $resolvedTree -cne $baselineTree) {
        throw "Pinned Response 0.9.0 base tree does not match its fixture identity."
    }
    $currentCommit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $currentCommit -ceq $baselineCommit) {
        throw "Response 0.9.0 source must be structurally independent from the candidate commit."
    }
    $candidateManifest = Get-Content -LiteralPath $candidateManifestPath -Raw |
        ConvertFrom-Json -Depth 100
    if ([int]$candidateManifest.schema_version -ne 4 -or
        [string]$candidateManifest.definition_id -cne "tessara.responses" -or
        [string]$candidateManifest.release_version -cne $candidateRelease -or
        [string]$candidateManifest.platform_versions.module_contract -cne "0.4.0" -or
        [string]$candidateManifest.linked_packages.module_contract -cne "0.4.0") {
        throw "The tracked Response Manifest must remain the real tessara.responses 1.0.0 candidate."
    }
    if ((Get-FileSha256 -Path $candidateMigrationPath) -cne $responseMigrationSha256) {
        throw "The tracked Response squashed migration is not the settled Sprint 8C baseline."
    }

    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json -Depth 100
    if ([int]$fixture.schema_version -ne 1 -or
        [string]$fixture.contract -cne "tessara.sprint-8c.response-upgrade-fixture" -or
        [string]$fixture.transition.owner -cne "tessara.responses" -or
        [string]$fixture.transition.mechanism -cne "resolved_one_owner_blueprint_delta" -or
        [string]$fixture.transition.intended_release -cne $candidateRelease) {
        throw "Response upgrade fixture does not declare the exact one-owner transition."
    }
    Assert-ExactSequence -Actual @($fixture.sequence) `
        -Expected @("0.9.0", "1.0.0", "0.9.0", "1.0.0") -Label "release sequence"
    Assert-ExactSequence -Actual @($fixture.stages) -Expected @(
        "establish-compatible-baseline", "upgrade-to-candidate",
        "rollback-to-compatible-baseline", "restore-intended-candidate"
    ) -Label "transition stages"
    $artifact = $fixture.prior_compatible_artifact
    Assert-ExactSequence -Actual @($artifact.PSObject.Properties.Name) -Expected @(
        "release", "source_model", "base_commit", "base_tree", "patch_path", "patch_sha256",
        "cargo_package", "cargo_features", "dockerfile", "builder", "executable", "fixture_kind",
        "manifest_source", "manifest_projection", "allowed_manifest_variations",
        "metadata_receipt_contract"
    ) -Label "prior-compatible artifact fields"
    $expectedArtifact = [ordered]@{
        release = $BaselineRelease
        source_model = $sourceModel
        base_commit = $baselineCommit
        base_tree = $baselineTree
        patch_path = "deploy/sprint-8c/baselines/response-0.9.0.patch"
        patch_sha256 = Get-FileSha256 -Path $patchPath
        cargo_package = "tessara-response-module"
        dockerfile = "deploy/sprint-8c/Dockerfile.response-upgrade-baseline"
        builder = "scripts/build-sprint-8c-response-upgrade-baseline.ps1"
        executable = "/usr/local/bin/response-module"
        fixture_kind = $fixtureKind
        manifest_source = "crates/tessara-response-module/manifest.json"
        manifest_projection = $manifestProjection
        metadata_receipt_contract = $receiptContract
    }
    foreach ($field in $expectedArtifact.Keys) {
        if ([string]$artifact.$field -cne [string]$expectedArtifact[$field]) {
            throw "Response prior-compatible artifact '$field' is not exact."
        }
    }
    Assert-ExactSequence -Actual @($artifact.cargo_features) -Expected @() -Label "baseline cargo features"
    Assert-ExactSequence -Actual @($artifact.allowed_manifest_variations) `
        -Expected $allowedManifestVariations -Label "allowed Manifest variations"

    Assert-ExactSequence -Actual @($fixture.response_releases.PSObject.Properties.Name) `
        -Expected @("0.9.0", "1.0.0") -Label "Response release inventory"
    foreach ($release in @("0.9.0", "1.0.0")) {
        $declaration = $fixture.response_releases.PSObject.Properties[$release].Value
        if ([string]$declaration.contract -cne "tessara.responses.product" -or
            [string]$declaration.contract_version -cne "1.0.0" -or
            [string]$declaration.resource_contract -cne $resourceContractId -or
            [string]$declaration.resource_contract_version -cne $resourceContractVersion -or
            [string]$declaration.lifecycle_contract -cne $lifecycleContractId -or
            [string]$declaration.lifecycle_contract_version -cne $lifecycleContractVersion -or
            [bool]$declaration.source_built -ne $true) {
            throw "Response release '$release' is not an exact source-built compatible fixture."
        }
    }
    $schema = $fixture.schema_compatibility
    if ([string]$schema.mode -cne "reads_current_1_0_0_response_baseline_after_rollback" -or
        [string]$schema.migration_path -cne "crates/tessara-response-module/migrations/001_response_module.sql" -or
        [string]$schema.source_independence -cne "authenticated_pinned_git_tree_plus_dedicated_patch" -or
        [string]$schema.candidate_source_dependency -cne "none" -or
        [string]$schema.rollback_reader_contract -cne "same_owned_schema_and_product_read_paths") {
        throw "Response rollback source/schema compatibility is not exact."
    }
    Assert-ExactSequence -Actual @($schema.required_owned_tables) -Expected @(
        "response_module_configuration", "response_module_security_state",
        "response_provider_observations", "response_consumed_service_nonces",
        "response_consumed_core_service_nonces", "responses", "response_start_claims",
        "response_values", "response_audit_events", "response_idempotency_receipts",
        "response_workflow_event_state", "response_workflow_events", "response_export_state",
        "response_export_changes", "response_bootstrap_receipts"
    ) -Label "required Response owned tables"
    Assert-ExactSequence -Actual @($fixture.fixed_dependencies.PSObject.Properties.Name) `
        -Expected @(
            "core", "tessara.datasets", "tessara.components", "tessara.dashboards",
            "tessara.reference.scoped-records"
        ) -Label "fixed owner inventory"
    Assert-ExactSequence -Actual @($fixture.preserved) -Expected @(
        "response_state", "module_instance_identity", "typed_resource_identity",
        "navigation_identity", "outbox_positions"
    ) -Label "preserved Response state"
    Assert-ExactSequence -Actual @($fixture.unrelated_unchanged) -Expected @(
        "images", "containers", "restart_counts", "owner_data", "navigation", "availability"
    ) -Label "unrelated unchanged state"
}

function Assert-MaterializedBaselineSource {
    param([Parameter(Mandatory)][string]$SourceRoot, [string]$ExpectedMaterializedDigest = "")
    $responseRoot = Join-Path $SourceRoot "crates/tessara-response-module"
    $markerPath = Join-Path $responseRoot "BASELINE-SOURCE.json"
    $cargoPath = Join-Path $responseRoot "Cargo.toml"
    $sourcePath = Join-Path $responseRoot "src/lib.rs"
    $manifestPath = Join-Path $responseRoot "manifest.json"
    $migrationPath = Join-Path $responseRoot "migrations/001_response_module.sql"
    $mainPath = Join-Path $responseRoot "src/main.rs"
    $operationalPath = Join-Path $responseRoot "src/operational.rs"
    $providerClientPath = Join-Path $responseRoot "src/provider_client.rs"
    $reversePath = Join-Path $responseRoot "src/reverse_provider.rs"
    $moduleContractCargoPath = Join-Path $SourceRoot "crates/tessara-module-contract/Cargo.toml"
    $moduleContractSourcePath = Join-Path $SourceRoot "crates/tessara-module-contract/src/lib.rs"
    $moduleProtocolPath = Join-Path $SourceRoot "crates/tessara-module-contract/src/protocol.rs"
    $responseContractSourcePath = Join-Path $SourceRoot "crates/tessara-responses-contract/src/lib.rs"
    $webCargoPath = Join-Path $SourceRoot "crates/tessara-web-responses/Cargo.toml"
    $lockPath = Join-Path $SourceRoot "Cargo.lock"
    foreach ($path in @(
        $markerPath, $cargoPath, $sourcePath, $manifestPath, $migrationPath,
        $mainPath, $operationalPath, $providerClientPath, $reversePath,
        $moduleContractCargoPath, $moduleContractSourcePath, $moduleProtocolPath,
        $responseContractSourcePath, $webCargoPath, $lockPath
    )) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Materialized Response baseline source omits '$path'."
        }
    }
    $marker = Get-Content -LiteralPath $markerPath -Raw | ConvertFrom-Json -Depth 20
    if ([int]$marker.schema_version -ne 1 -or
        [string]$marker.contract -cne "tessara.sprint-8c.response-baseline-source" -or
        [string]$marker.release -cne $BaselineRelease -or
        [string]$marker.base_commit -cne $baselineCommit -or
        [string]$marker.base_tree -cne $baselineTree -or
        [string]$marker.source_model -cne $sourceModel -or
        [string]$marker.candidate_source_dependency -cne "none") {
        throw "Materialized Response baseline source marker is not exact."
    }
    $cargoText = Get-Content -LiteralPath $cargoPath -Raw
    $sourceText = Get-Content -LiteralPath $sourcePath -Raw
    $mainText = Get-Content -LiteralPath $mainPath -Raw
    $operationalText = Get-Content -LiteralPath $operationalPath -Raw
    $providerClientText = Get-Content -LiteralPath $providerClientPath -Raw
    $reverseText = Get-Content -LiteralPath $reversePath -Raw
    $moduleContractCargoText = Get-Content -LiteralPath $moduleContractCargoPath -Raw
    $moduleContractSourceText = Get-Content -LiteralPath $moduleContractSourcePath -Raw
    $moduleProtocolText = Get-Content -LiteralPath $moduleProtocolPath -Raw
    $responseContractSourceText = Get-Content -LiteralPath $responseContractSourcePath -Raw
    $webCargoText = Get-Content -LiteralPath $webCargoPath -Raw
    $lockText = Get-Content -LiteralPath $lockPath -Raw
    if ($cargoText -cnotmatch '(?ms)^\[package\].*?^name = "tessara-response-module"\s*$.*?^version = "0\.9\.0"\s*$' -or
        $cargoText.Contains('sprint-8c-upgrade-baseline') -or
        -not $sourceText.Contains('pub const MODULE_RELEASE_VERSION: &str = "0.9.0";') -or
        $sourceText.Contains('sprint-8c-upgrade-baseline') -or
        -not $sourceText.Contains('mod operational;') -or
        -not $sourceText.Contains('ResponseOperationalProjection::load_cached') -or
        -not $reverseText.Contains('ResponseOperationalProjection::load_cached') -or
        -not $operationalText.Contains('MODULE_PROVIDER_COMPATIBILITY_PATH') -or
        -not $operationalText.Contains('SignedEnvelopeV1<ModuleProviderCompatibilityResponseV1>') -or
        -not $operationalText.Contains('ProtocolSignaturePurposeV1::ProviderCompatibilityResponse') -or
        -not $providerClientText.Contains('MODULE_PROVIDER_COMPATIBILITY_SERVICE_CONTEXT') -or
        -not $mainText.Contains('ProtocolSignaturePurposeV1::ProviderCompatibilityResponse') -or
        -not $moduleContractCargoText.Contains('version = "0.4.0"') -or
        -not $moduleContractSourceText.Contains('ModuleProviderCompatibilityRequestV1') -or
        -not $moduleProtocolText.Contains('ModuleProviderCompatibilityResponseV1') -or
        -not $moduleProtocolText.Contains('ProviderCompatibilityResponse') -or
        $responseContractSourceText.Contains('ResponseProviderCompatibility') -or
        -not $webCargoText.Contains('features = ["components"]') -or
        $lockText -cnotmatch '(?ms)^name = "tessara-response-module"\s*^version = "0\.9\.0"\s*$') {
        throw "Materialized Response baseline source is not the dedicated 0.9.0 implementation."
    }
    foreach ($readinessCode in @(
        "response.configuration", "response.database", "response.events.publication",
        "response.export.publication", "response.provider.forms",
        "response.provider.workflow", "response.security_state"
    )) {
        if (-not $operationalText.Contains(('"' + $readinessCode + '"'))) {
            throw "Materialized Response baseline omits readiness check '$readinessCode'."
        }
    }
    $normalizedBaselineSource = $sourceText.Replace(
        'pub const MODULE_RELEASE_VERSION: &str = "0.9.0";',
        'pub const MODULE_RELEASE_VERSION: &str = "1.0.0";'
    )
    if ($normalizedBaselineSource -cne (Get-Content -LiteralPath $candidateSourcePath -Raw) -or
        (Get-FileSha256 -Path $mainPath) -cne (Get-FileSha256 -Path $candidateMainPath) -or
        (Get-FileSha256 -Path $operationalPath) -cne (Get-FileSha256 -Path $candidateOperationalPath) -or
        (Get-FileSha256 -Path $providerClientPath) -cne (Get-FileSha256 -Path $candidateProviderClientPath) -or
        (Get-FileSha256 -Path $reversePath) -cne (Get-FileSha256 -Path $candidateReversePath)) {
        throw "Response 0.9.0 operational/readiness behavior drifted from the current candidate."
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 100
    if ([int]$manifest.schema_version -ne 4 -or
        [string]$manifest.definition_id -cne "tessara.responses" -or
        [string]$manifest.release_version -cne $BaselineRelease -or
        [string]$manifest.platform_versions.module_contract -cne "0.4.0" -or
        [string]$manifest.linked_packages.module_contract -cne "0.4.0") {
        throw "Materialized Response baseline Manifest identity is invalid."
    }
    foreach ($contract in @(
        [pscustomobject]@{ id = $resourceContractId; version = $resourceContractVersion },
        [pscustomobject]@{ id = $lifecycleContractId; version = $lifecycleContractVersion }
    )) {
        $matches = @($manifest.provided_contracts | Where-Object { [string]$_.id -ceq $contract.id })
        if ($matches.Count -ne 1 -or [string]$matches[0].version -cne $contract.version) {
            throw "Materialized Response baseline omits contract '$($contract.id)' '$($contract.version)'."
        }
    }
    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json -Depth 100
    $migrationText = Get-Content -LiteralPath $migrationPath -Raw
    if ($migrationText.Contains('response_export_consumer_checkpoints')) {
        throw "Materialized Response baseline retained the removed export-consumer checkpoint table."
    }
    foreach ($table in @($fixture.schema_compatibility.required_owned_tables)) {
        if ($migrationText -cnotmatch "(?m)^CREATE TABLE $([regex]::Escape([string]$table))\s*\(") {
            throw "Materialized Response baseline migration omits owned table '$table'."
        }
    }
    $baselineMigrationSha256 = Get-FileSha256 -Path $migrationPath
    $candidateMigrationSha256 = Get-FileSha256 -Path $candidateMigrationPath
    if ($baselineMigrationSha256 -cne $responseMigrationSha256 -or
        $candidateMigrationSha256 -cne $responseMigrationSha256) {
        throw "Materialized Response baseline must embed the exact candidate squashed migration checksum."
    }
    $projection = Get-SourceProjectionIdentity -SourceRoot $SourceRoot
    if (-not [string]::IsNullOrEmpty($ExpectedMaterializedDigest) -and
        [string]$projection.sha256 -cne $ExpectedMaterializedDigest) {
        throw "Materialized Response baseline source digest changed after authentication."
    }
    $identity = [ordered]@{
        source_model = $sourceModel
        candidate_source_dependency = "none"
        commit = $baselineCommit
        tree = $baselineTree
        patch_sha256 = Get-FileSha256 -Path $patchPath
        materialized_source_sha256 = [string]$projection.sha256
        materialized_file_count = [int]$projection.file_count
        marker_sha256 = Get-FileSha256 -Path $markerPath
    }
    Assert-BaselineProvenance -Identity $identity
    [pscustomobject]$identity
}

function New-BaselineMaterialization {
    $temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) `
        ("tessara-response-baseline-" + [guid]::NewGuid().ToString("N"))
    [IO.Directory]::CreateDirectory($temporaryRoot) | Out-Null
    $archivePath = Join-Path $temporaryRoot "source.tar"
    $sourceRoot = Join-Path $temporaryRoot "source"
    [IO.Directory]::CreateDirectory($sourceRoot) | Out-Null
    & git -C $repoRoot archive --format=tar --output=$archivePath $baselineCommit
    if ($LASTEXITCODE -ne 0) { throw "Could not archive the pinned Response 0.9.0 source commit." }
    & tar -xf $archivePath -C $sourceRoot
    if ($LASTEXITCODE -ne 0) { throw "Could not extract the pinned Response 0.9.0 source archive." }
    Push-Location $sourceRoot
    try {
        & git -c core.autocrlf=false apply --check $patchPath
        if ($LASTEXITCODE -ne 0) { throw "Dedicated Response 0.9.0 source patch does not apply exactly." }
        & git -c core.autocrlf=false apply $patchPath
        if ($LASTEXITCODE -ne 0) { throw "Dedicated Response 0.9.0 source patch failed to apply." }
    } finally { Pop-Location }
    $identity = Assert-MaterializedBaselineSource -SourceRoot $sourceRoot
    [pscustomobject]@{ temporary_root = $temporaryRoot; source_root = $sourceRoot; identity = $identity }
}

function Remove-BaselineMaterialization([string]$Path) {
    $resolved = [IO.Path]::GetFullPath($Path).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd(
        [IO.Path]::DirectorySeparatorChar
    )
    $leaf = [IO.Path]::GetFileName($resolved)
    if (-not $resolved.StartsWith("$temporaryBase$([IO.Path]::DirectorySeparatorChar)",
            [StringComparison]::OrdinalIgnoreCase) -or
        -not $leaf.StartsWith("tessara-response-baseline-", [StringComparison]::Ordinal)) {
        throw "Refusing to remove non-baseline temporary path '$resolved'."
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force -ErrorAction SilentlyContinue
}

function Assert-MetadataReceiptPair {
    param([Parameter(Mandatory)][string]$ArtifactPath, [Parameter(Mandatory)][string]$SidecarPath)
    if (-not (Test-Path -LiteralPath $ArtifactPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $SidecarPath -PathType Leaf)) {
        throw "Response baseline metadata receipt pair is incomplete."
    }
    $expectedDigest = (Get-Content -LiteralPath $SidecarPath -Raw).Trim()
    $actualDigest = Get-FileSha256 -Path $ArtifactPath
    if ($expectedDigest -cnotmatch '^[0-9a-f]{64}$' -or $actualDigest -cne $expectedDigest) {
        throw "Response baseline metadata receipt pair failed SHA-256 authentication."
    }
    $document = Get-Content -LiteralPath $ArtifactPath -Raw | ConvertFrom-Json -Depth 100
    if ([int]$document.schema_version -ne 1 -or
        [string]$document.receipt_contract -cne $receiptContract) {
        throw "Response baseline metadata receipt contract is invalid."
    }
    $document
}

function Publish-MetadataReceipt {
    param([Parameter(Mandatory)][object]$Document, [Parameter(Mandatory)][string]$OutputPath)
    $sidecarPath = "$OutputPath.sha256"
    if ((Test-Path -LiteralPath $OutputPath) -or (Test-Path -LiteralPath $sidecarPath)) {
        throw "Response baseline metadata already exists and cannot be overwritten: $OutputPath"
    }
    $directory = Split-Path -Parent $OutputPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    $temporary = Join-Path $directory `
        ".$([IO.Path]::GetFileName($OutputPath)).$([guid]::NewGuid().ToString('N')).tmp"
    $temporarySidecar = "$temporary.sha256"
    $publishedDocument = $false
    try {
        [IO.File]::WriteAllText(
            $temporary, (($Document | ConvertTo-Json -Depth 100) + "`n"),
            [Text.UTF8Encoding]::new($false)
        )
        $digest = Get-FileSha256 -Path $temporary
        [IO.File]::WriteAllText($temporarySidecar, "$digest`n", [Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $temporary -Destination $OutputPath
        $publishedDocument = $true
        Move-Item -LiteralPath $temporarySidecar -Destination $sidecarPath
        Assert-MetadataReceiptPair -ArtifactPath $OutputPath -SidecarPath $sidecarPath | Out-Null
        [pscustomobject]@{ path = $OutputPath; sha256 = $digest }
    } catch {
        if ($publishedDocument) { Remove-Item -LiteralPath $OutputPath -Force -ErrorAction SilentlyContinue }
        Remove-Item -LiteralPath $sidecarPath -Force -ErrorAction SilentlyContinue
        throw
    } finally {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temporarySidecar -Force -ErrorAction SilentlyContinue
    }
}

function Test-MetadataReceiptPublisher {
    $path = Join-Path $repoRoot `
        "target/.response-baseline-receipt-self-test-$([guid]::NewGuid().ToString('N')).json"
    try {
        $published = Publish-MetadataReceipt -Document ([ordered]@{
            schema_version = 1; receipt_contract = $receiptContract; self_test = $true
        }) -OutputPath $path
        if ([string]$published.path -cne $path -or
            [string]$published.sha256 -cnotmatch '^[0-9a-f]{64}$') {
            throw "Response baseline metadata publisher returned an invalid identity."
        }
        $overwriteRejected = $false
        try {
            Publish-MetadataReceipt -Document ([ordered]@{
                schema_version = 1; receipt_contract = $receiptContract; self_test = $false
            }) -OutputPath $path | Out-Null
        } catch { $overwriteRejected = $true }
        if (-not $overwriteRejected) { throw "Response baseline metadata publisher admitted an overwrite." }
        Assert-MetadataReceiptPair -ArtifactPath $path -SidecarPath "$path.sha256" | Out-Null
        [IO.File]::AppendAllText($path, " ", [Text.UTF8Encoding]::new($false))
        $tamperRejected = $false
        try { Assert-MetadataReceiptPair -ArtifactPath $path -SidecarPath "$path.sha256" | Out-Null } catch {
            $tamperRejected = $true
        }
        if (-not $tamperRejected) {
            throw "Response baseline metadata self-test accepted a tampered receipt pair."
        }
    } finally {
        Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath "$path.sha256" -Force -ErrorAction SilentlyContinue
    }
}

function Test-IndependentSourceFixture {
    $materialization = $null
    try {
        $materialization = New-BaselineMaterialization
        Assert-BaselineProvenance -Identity $materialization.identity
        $baselineManifest = Get-Content -LiteralPath (Join-Path $materialization.source_root `
            "crates/tessara-response-module/manifest.json") -Raw | ConvertFrom-Json -Depth 100
        $candidateManifest = Get-Content -LiteralPath $candidateManifestPath -Raw |
            ConvertFrom-Json -Depth 100
        Assert-ManifestBehaviorCompatibility -Baseline $baselineManifest `
            -Candidate $candidateManifest | Out-Null
        $behaviorTamper = $baselineManifest | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
        $behaviorTamper.definition_id = "tampered.responses"
        $behaviorRejected = $false
        try {
            Assert-ManifestBehaviorCompatibility -Baseline $behaviorTamper `
                -Candidate $candidateManifest | Out-Null
        } catch { $behaviorRejected = $true }
        if (-not $behaviorRejected) {
            throw "Response baseline self-test accepted incompatible Manifest behavior."
        }
        $provenanceTamper = $materialization.identity | ConvertTo-Json -Depth 20 | ConvertFrom-Json -Depth 20
        $provenanceTamper.patch_sha256 = "0" * 64
        $provenanceRejected = $false
        try { Assert-BaselineProvenance -Identity $provenanceTamper } catch { $provenanceRejected = $true }
        if (-not $provenanceRejected) {
            throw "Response baseline self-test accepted tampered pinned-source provenance."
        }
        $markerPath = Join-Path $materialization.source_root `
            "crates/tessara-response-module/BASELINE-SOURCE.json"
        [IO.File]::AppendAllText($markerPath, " ", [Text.UTF8Encoding]::new($false))
        $sourceTamperRejected = $false
        try {
            Assert-MaterializedBaselineSource -SourceRoot $materialization.source_root `
                -ExpectedMaterializedDigest ([string]$materialization.identity.materialized_source_sha256) |
                Out-Null
        } catch { $sourceTamperRejected = $true }
        if (-not $sourceTamperRejected) {
            throw "Response baseline self-test accepted tampered materialized source."
        }
    } finally {
        if ($null -ne $materialization) {
            Remove-BaselineMaterialization -Path $materialization.temporary_root
        }
    }
}

Assert-SourceFixture
if ($SelfTest) {
    Test-IndependentSourceFixture
    Test-MetadataReceiptPublisher
    Write-Host "Sprint 8C authenticated independent Response 0.9.0 source self-test passed."
    return
}

if ($OutputTag -notmatch '^[^\s@]+:[^\s@]+$') {
    throw "OutputTag must be one local mutable image tag; the receipt records its immutable image ID."
}
$metadataPath = Resolve-OutputPath $MetadataOutputPath
$generatedManifestPath = [IO.Path]::ChangeExtension($metadataPath, "manifest.json")
foreach ($retainedPath in @($metadataPath, "$metadataPath.sha256", $generatedManifestPath)) {
    if (Test-Path -LiteralPath $retainedPath) {
        throw "Response baseline retained output already exists and cannot be overwritten: $retainedPath"
    }
}

$generatedManifestCreated = $false
$materialization = $null
Push-Location $repoRoot
try {
    $sourceStatus = @(& git status --porcelain=v1 --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "Could not inspect the Response baseline builder worktree." }
    if ($sourceStatus.Count -ne 0) {
        throw "Response baseline release freezing requires a clean governing builder worktree."
    }
    $builderCommit = (& git rev-parse HEAD).Trim()
    $builderTree = (& git rev-parse 'HEAD^{tree}').Trim()
    if ($LASTEXITCODE -ne 0 -or
        $builderCommit -cnotmatch '^[0-9a-f]{40,64}$' -or $builderTree -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Could not resolve exact Response baseline builder source identity."
    }
    if ($builderCommit -ceq $baselineCommit -or $builderTree -ceq $baselineTree) {
        throw "Response baseline builder and candidate may not reuse the pinned 0.9.0 source identity."
    }
    $materialization = New-BaselineMaterialization
    $baselineIdentity = $materialization.identity
    Assert-BaselineProvenance -Identity $baselineIdentity

    & docker build `
        --file $dockerfile `
        --build-arg "TESSARA_RESPONSE_RELEASE=$BaselineRelease" `
        --build-arg "TESSARA_BASELINE_SOURCE_COMMIT=$baselineCommit" `
        --build-arg "TESSARA_BASELINE_SOURCE_TREE=$baselineTree" `
        --build-arg "TESSARA_BASELINE_PATCH_SHA256=$($baselineIdentity.patch_sha256)" `
        --build-arg "TESSARA_BASELINE_MATERIALIZED_SOURCE_SHA256=$($baselineIdentity.materialized_source_sha256)" `
        --build-arg "TESSARA_BUILDER_SOURCE_COMMIT=$builderCommit" `
        --build-arg "TESSARA_BUILDER_SOURCE_TREE=$builderTree" `
        --tag $OutputTag `
        $materialization.source_root
    if ($LASTEXITCODE -ne 0) {
        throw "Authenticated independent prior-compatible Response image build failed."
    }
    $inspection = @(& docker image inspect $OutputTag | ConvertFrom-Json -Depth 100)
    if ($LASTEXITCODE -ne 0 -or $inspection.Count -ne 1) {
        throw "Could not inspect the independent prior-compatible Response image."
    }
    $imageId = [string]$inspection[0].Id
    if ($imageId -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Response baseline image does not have one immutable image ID."
    }
    $labels = $inspection[0].Config.Labels
    if ([string]$labels.'com.tessara.module-definition' -cne 'tessara.responses' -or
        [string]$labels.'com.tessara.module-release' -cne $BaselineRelease -or
        [string]$labels.'com.tessara.upgrade.fixture' -cne $fixtureKind -or
        [string]$labels.'org.opencontainers.image.revision' -cne $baselineCommit -or
        [string]$labels.'com.tessara.source-tree' -cne $baselineTree -or
        [string]$labels.'com.tessara.source-dirty' -cne 'false' -or
        [string]$labels.'com.tessara.baseline-patch-sha256' -cne $baselineIdentity.patch_sha256 -or
        [string]$labels.'com.tessara.materialized-source-sha256' -cne $baselineIdentity.materialized_source_sha256 -or
        [string]$labels.'com.tessara.builder-source-commit' -cne $builderCommit -or
        [string]$labels.'com.tessara.builder-source-tree' -cne $builderTree) {
        throw "Response baseline OCI independent-source identity is incomplete."
    }
    Assert-ExactSequence -Actual @($inspection[0].Config.Entrypoint) `
        -Expected @('/usr/local/bin/response-module') -Label "Response baseline entrypoint"
    $binaryHashOutput = @(
        & docker run --rm --entrypoint sha256sum $OutputTag /usr/local/bin/response-module
    )
    if ($LASTEXITCODE -ne 0 -or $binaryHashOutput.Count -ne 1 -or
        [string]$binaryHashOutput[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
        throw "Could not capture the independent Response 0.9.0 executable identity."
    }
    $executableSha256 = $Matches.hash

    $materializedManifestPath = Join-Path $materialization.source_root `
        "crates/tessara-response-module/manifest.json"
    $baselineManifest = Get-Content -LiteralPath $materializedManifestPath -Raw | ConvertFrom-Json -Depth 100
    $candidateManifest = Get-Content -LiteralPath $candidateManifestPath -Raw | ConvertFrom-Json -Depth 100
    $projectionSha256 = Assert-ManifestBehaviorCompatibility -Baseline $baselineManifest `
        -Candidate $candidateManifest

    [IO.Directory]::CreateDirectory((Split-Path -Parent $generatedManifestPath)) | Out-Null
    Copy-Item -LiteralPath $materializedManifestPath -Destination $generatedManifestPath
    $generatedManifestCreated = $true
    $manifestDigestOutput = @(
        & cargo run --quiet --locked --offline -p tessara-supervisor --bin tessara-compose -- `
            manifest-digest $generatedManifestPath
    )
    if ($LASTEXITCODE -ne 0) { throw "Could not compute the canonical Response baseline Manifest digest." }
    $manifestDigest = [string]($manifestDigestOutput | Select-Object -Last 1)
    if ($manifestDigest -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Response baseline Manifest digest is invalid: '$manifestDigest'."
    }
    $candidateManifestDigestOutput = @(
        & cargo run --quiet --locked --offline -p tessara-supervisor --bin tessara-compose -- `
            manifest-digest $candidateManifestPath
    )
    if ($LASTEXITCODE -ne 0) { throw "Could not compute the canonical Response candidate Manifest digest." }
    $candidateManifestDigest = [string]($candidateManifestDigestOutput | Select-Object -Last 1)
    if ($candidateManifestDigest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
        $candidateManifestDigest -ceq $manifestDigest) {
        throw "Response candidate and baseline Manifest identities are not distinct and canonical."
    }

    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json -Depth 100
    $baselineMigrationPath = Join-Path $materialization.source_root `
        "crates/tessara-response-module/migrations/001_response_module.sql"
    $baselineLockPath = Join-Path $materialization.source_root "Cargo.lock"
    $markerPath = Join-Path $materialization.source_root `
        "crates/tessara-response-module/BASELINE-SOURCE.json"
    $metadata = [ordered]@{
        schema_version = 1
        receipt_contract = $receiptContract
        generated_at = [DateTimeOffset]::UtcNow.ToString("O")
        release_identity = [ordered]@{
            definition_id = "tessara.responses"; version = $BaselineRelease
            manifest_digest = $manifestDigest; runtime_image = $imageId; migration_image = $imageId
            image_reference = $OutputTag; executable_sha256 = $executableSha256
            manifest_document_sha256 = Get-FileSha256 -Path $generatedManifestPath
        }
        source_identity = [ordered]@{
            commit = $builderCommit; tree = $builderTree; dirty = $false
            builder = "scripts/build-sprint-8c-response-upgrade-baseline.ps1"
            builder_sha256 = Get-FileSha256 -Path $PSCommandPath
            dockerfile_sha256 = Get-FileSha256 -Path $dockerfile
            candidate_dockerfile_sha256 = Get-FileSha256 -Path $candidateDockerfile
        }
        baseline_source_identity = [ordered]@{
            source_model = $sourceModel; candidate_source_dependency = "none"
            commit = $baselineCommit; tree = $baselineTree
            patch_path = Get-RelativeRepositoryPath -Path $patchPath
            patch_sha256 = [string]$baselineIdentity.patch_sha256
            materialized_source_sha256 = [string]$baselineIdentity.materialized_source_sha256
            materialized_file_count = [int]$baselineIdentity.materialized_file_count
            marker_sha256 = Get-FileSha256 -Path $markerPath
            cargo_package = "tessara-response-module"; cargo_features = @()
            cargo_lock_sha256 = Get-FileSha256 -Path $baselineLockPath
            manifest_source_sha256 = Get-FileSha256 -Path $materializedManifestPath
            migration_sha256 = Get-FileSha256 -Path $baselineMigrationPath
        }
        compatibility = [ordered]@{
            state = "passed"; manifest_projection = $manifestProjection
            allowed_manifest_variations = $allowedManifestVariations
            projection_sha256 = $projectionSha256; tracked_candidate_release = $candidateRelease
            candidate_manifest_digest = $candidateManifestDigest
            resource_contract_id = $resourceContractId; resource_contract_version = $resourceContractVersion
            lifecycle_contract_id = $lifecycleContractId; lifecycle_contract_version = $lifecycleContractVersion
            rollback_schema_expectation = "reads_current_1_0_0_response_baseline_after_rollback"
            rollback_reader_contract = "same_owned_schema_and_product_read_paths"
            migration_compatibility = "required-owned-tables-and-live-rollback-readback"
            migration_path = "crates/tessara-response-module/migrations/001_response_module.sql"
            candidate_migration_sha256 = Get-FileSha256 -Path $candidateMigrationPath
            required_owned_tables = @($fixture.schema_compatibility.required_owned_tables)
        }
        fixture_identity = [ordered]@{
            contract_path = Get-RelativeRepositoryPath -Path $fixturePath
            contract_sha256 = Get-FileSha256 -Path $fixturePath; fixture_kind = $fixtureKind
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
    if ($null -ne $materialization) {
        Remove-BaselineMaterialization -Path $materialization.temporary_root
    }
}

[CmdletBinding()]
param(
    [string]$OutputTag = "tessara-sprint-8a-components-rehearsal-baseline:latest",
    [ValidatePattern('^0\.9\.0$')][string]$BaselineRelease = "0.9.0",
    [string]$MetadataOutputPath = "target/sprint-8a-upgrade/component-baseline-release.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
$dockerfile = Join-Path $repoRoot "deploy/sprint-8a/Dockerfile.component-rehearsal-baseline"
$manifestPath = Join-Path $repoRoot "crates/tessara-component-module/manifest.json"
$componentCargo = Join-Path $repoRoot "crates/tessara-component-module/Cargo.toml"
$componentSource = Join-Path $repoRoot "crates/tessara-component-module/src/lib.rs"

function Resolve-OutputPath([string]$Path) {
    if ([IO.Path]::IsPathRooted($Path)) { return [IO.Path]::GetFullPath($Path) }
    return [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

if ($SelfTest) {
    $dockerfileText = Get-Content -LiteralPath $dockerfile -Raw
    $cargoText = Get-Content -LiteralPath $componentCargo -Raw
    $sourceText = Get-Content -LiteralPath $componentSource -Raw
    foreach ($fragment in @(
        'cargo build --release -p tessara-component-module',
        '--features sprint-8a-rehearsal-baseline',
        'com.tessara.module-release="$TESSARA_COMPONENT_RELEASE"',
        'com.tessara.rehearsal.fixture="source-built-compatible-release-v1"',
        'COPY --from=builder /tmp/component-module /usr/local/bin/component-module'
    )) {
        if (-not $dockerfileText.Contains($fragment)) {
            throw "Source-built Component baseline Dockerfile omits '$fragment'."
        }
    }
    if ($dockerfileText -match '(?m)^\s*FROM\s+\$\{?COMPONENT_BASE_IMAGE' -or
        -not $cargoText.Contains('sprint-8a-rehearsal-baseline = []') -or
        -not $sourceText.Contains('#[cfg(feature = "sprint-8a-rehearsal-baseline")]') -or
        -not $sourceText.Contains('pub const MODULE_RELEASE_VERSION: &str = "0.9.0";')) {
        throw "Component rehearsal baseline must compile the distinct 0.9.0 source feature, not relabel the candidate image."
    }
    Write-Host "Sprint 8A source-built compatible Component release self-test passed."
    return
}

if ($OutputTag -notmatch '^[^\s@]+:[^\s@]+$') {
    throw "OutputTag must be one local mutable image tag; the receipt records its immutable image ID."
}
$metadataPath = Resolve-OutputPath $MetadataOutputPath
if ((Test-Path -LiteralPath $metadataPath) -or (Test-Path -LiteralPath "$metadataPath.sha256")) {
    throw "Component baseline release metadata already exists and cannot be overwritten: $metadataPath"
}
[IO.Directory]::CreateDirectory((Split-Path -Parent $metadataPath)) | Out-Null
$generatedManifestPath = [IO.Path]::ChangeExtension($metadataPath, "manifest.json")
if (Test-Path -LiteralPath $generatedManifestPath) {
    throw "Generated Component baseline manifest already exists and cannot be overwritten: $generatedManifestPath"
}

Push-Location $repoRoot
try {
    $sourceCommit = (& git rev-parse HEAD).Trim()
    $sourceTree = (& git write-tree).Trim()
    $sourceDirty = -not [string]::IsNullOrWhiteSpace((& git status --porcelain=v1))
    & docker build `
        --file $dockerfile `
        --build-arg "TESSARA_COMPONENT_RELEASE=$BaselineRelease" `
        --build-arg "TESSARA_SOURCE_COMMIT=$sourceCommit" `
        --build-arg "TESSARA_SOURCE_TREE=$sourceTree" `
        --build-arg "TESSARA_SOURCE_DIRTY=$($sourceDirty.ToString().ToLowerInvariant())" `
        --tag $OutputTag `
        .
    if ($LASTEXITCODE -ne 0) { throw "Source-built Component baseline image build failed." }

    $inspection = @(& docker image inspect $OutputTag | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0 -or $inspection.Count -ne 1) {
        throw "Could not inspect the source-built Component baseline image."
    }
    $imageId = [string]$inspection[0].Id
    if ($imageId -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Component baseline image does not have one immutable image ID."
    }
    $labels = $inspection[0].Config.Labels
    if ([string]$labels.'com.tessara.module-definition' -cne 'tessara.components' -or
        [string]$labels.'com.tessara.module-release' -cne $BaselineRelease -or
        [string]$labels.'com.tessara.rehearsal.fixture' -cne 'source-built-compatible-release-v1' -or
        [string]$labels.'org.opencontainers.image.revision' -cne $sourceCommit -or
        [string]$labels.'com.tessara.source-tree' -cne $sourceTree) {
        throw "Component baseline OCI release/source identity is incomplete."
    }
    $binaryHashOutput = @(& docker run --rm --entrypoint sha256sum $OutputTag /usr/local/bin/component-module)
    if ($LASTEXITCODE -ne 0 -or $binaryHashOutput.Count -ne 1 -or
        [string]$binaryHashOutput[0] -cnotmatch '^(?<hash>[0-9a-f]{64})\s+') {
        throw "Could not capture the Component baseline executable identity."
    }
    $executableSha256 = $Matches.hash

    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $manifest.release_version = $BaselineRelease
    [IO.File]::WriteAllText(
        $generatedManifestPath,
        ($manifest | ConvertTo-Json -Depth 100) + "`n",
        [Text.UTF8Encoding]::new($false)
    )
    $manifestDigestOutput = @(& cargo run -q -p tessara-supervisor --bin tessara-compose -- digest $generatedManifestPath)
    if ($LASTEXITCODE -ne 0) { throw "Could not compute the canonical Component baseline Manifest digest." }
    $manifestDigest = [string]($manifestDigestOutput | Select-Object -Last 1)
    if ($manifestDigest -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Component baseline Manifest digest is invalid: '$manifestDigest'."
    }

    $metadata = [ordered]@{
        schema_version = 1
        release_identity = [ordered]@{
            definition_id = "tessara.components"
            version = $BaselineRelease
            manifest_digest = $manifestDigest
            runtime_image = $imageId
            image_reference = $OutputTag
            executable_sha256 = $executableSha256
        }
        source_identity = [ordered]@{
            commit = $sourceCommit
            tree = $sourceTree
            dirty = $sourceDirty
            feature = "sprint-8a-rehearsal-baseline"
        }
        manifest_path = [IO.Path]::GetRelativePath($repoRoot, $generatedManifestPath).Replace('\', '/')
    }
    Publish-Sprint7AEvidence -Document $metadata -OutputPath $metadataPath | Out-Null
    $metadataPath
} finally {
    Pop-Location
}

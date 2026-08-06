[CmdletBinding()]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [string]$BaselineTag = "tessara-sprint-8a-components-rehearsal-baseline:latest",
    [string]$OutputPath = "target/sprint-8a-upgrade/component-upgrade-rollback.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-8a"
$immutableImagePattern = '^[^\s@]+@sha256:[0-9a-f]{64}$'
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

$configuration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or [string]$configuration.name -cne $expectedProject) {
    throw "Sprint 8A Component upgrade orchestration is restricted to the exact $expectedProject Compose project."
}
$artifactImages = [string]$configuration.services.supervisor.environment.TESSARA_ARTIFACT_IMAGE_REFERENCES | ConvertFrom-Json
$componentArtifactReferences = @($artifactImages.PSObject.Properties['tessara.components'].Value)
if ($componentArtifactReferences -cnotcontains [string]$configuration.services.components.image -or
    $componentArtifactReferences -cnotcontains $BaselineTag) {
    throw "Supervisor artifact enrollment does not contain both the candidate and requested Component baseline references."
}

if ($SelfTest) {
    if (-not $configuration.services.components -or -not $configuration.services.'components-migrate') {
        throw "Sprint 8A Compose does not declare both Component runtime and migration services."
    }
    $builder = Get-Command (Join-Path $PSScriptRoot 'build-sprint-8a-component-rehearsal-baseline.ps1')
    if (-not $builder.Parameters.ContainsKey('OutputTag') -or $builder.Parameters.ContainsKey('OutputImage')) {
        throw "Component rehearsal baseline builder must expose the canonical -OutputTag contract only."
    }
    $verifier = Get-Command (Join-Path $PSScriptRoot 'verify-sprint-8a-component-upgrade.ps1')
    foreach ($parameter in @('BaselineMetadataPath', 'CandidateManifestPath', 'OutputPath')) {
        if (-not $verifier.Parameters.ContainsKey($parameter)) {
            throw "Component upgrade verifier no longer declares -$parameter."
        }
    }
    & (Join-Path $PSScriptRoot 'build-sprint-8a-component-rehearsal-baseline.ps1') -SelfTest
    & (Join-Path $PSScriptRoot 'verify-sprint-8a-component-upgrade.ps1') `
        -BaselineMetadataPath 'self-test-not-read.json' `
        -SelfTest
    Write-Host "Sprint 8A Component upgrade orchestration self-test passed."
    return
}

$componentIds = @(@(& docker compose -f $composePath --profile reference ps --status running -q components) |
    ForEach-Object { ([string]$_).Trim() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
if ($LASTEXITCODE -ne 0 -or $componentIds.Count -ne 1 -or $componentIds[0] -cnotmatch '^[0-9a-f]{64}$') {
    throw "Expected exactly one running $expectedProject 'components' container, found '$($componentIds -join ',')'."
}
$inspection = @(& docker inspect $componentIds[0] | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0 -or $inspection.Count -ne 1) {
    throw "Could not inspect the exact running Sprint 8A Component container."
}
$repository = [regex]::Replace([string]$inspection[0].Config.Image, ':[^/:]+$', '')
$candidateImage = "$repository@$([string]$inspection[0].Image)"
if ($candidateImage -cnotmatch $immutableImagePattern) {
    throw "Running Component image did not resolve to one immutable name@sha256 reference: '$candidateImage'."
}

$outputFullPath = if ([IO.Path]::IsPathRooted($OutputPath)) {
    [IO.Path]::GetFullPath($OutputPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $OutputPath))
}
$metadataPath = [IO.Path]::ChangeExtension($outputFullPath, "baseline-release.json")
$baselineOutput = @(& (Join-Path $PSScriptRoot 'build-sprint-8a-component-rehearsal-baseline.ps1') `
    -OutputTag $BaselineTag `
    -MetadataOutputPath $metadataPath)
$baselineMetadataPath = [string]($baselineOutput | Select-Object -Last 1)
$baselineMetadataPath = $baselineMetadataPath.Trim()
if (-not (Test-Path -LiteralPath $baselineMetadataPath -PathType Leaf)) {
    throw "Component rehearsal baseline builder did not emit release metadata: '$baselineMetadataPath'."
}

& (Join-Path $PSScriptRoot 'verify-sprint-8a-component-upgrade.ps1') `
    -ComposeFile $ComposeFile `
    -BaseUrl $BaseUrl `
    -BaselineMetadataPath $baselineMetadataPath `
    -OutputPath $OutputPath

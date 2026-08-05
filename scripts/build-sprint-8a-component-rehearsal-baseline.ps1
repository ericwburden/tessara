[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$CurrentImage,
    [string]$OutputTag = "tessara-sprint-8a-components-rehearsal-baseline:latest",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$dockerfile = Join-Path $repoRoot "deploy/sprint-8a/Dockerfile.component-rehearsal-baseline"
$immutableImagePattern = '^[^\s@]+@sha256:[0-9a-f]{64}$'

if (-not (Test-Path -LiteralPath $dockerfile -PathType Leaf)) {
    throw "Sprint 8A Component rehearsal baseline Dockerfile is missing."
}
if ($SelfTest) {
    $text = Get-Content -LiteralPath $dockerfile -Raw
    if ($text -notmatch 'ARG COMPONENT_BASE_IMAGE' -or
        $text -notmatch 'FROM \$\{COMPONENT_BASE_IMAGE\}' -or
        $text -notmatch 'com\.tessara\.rehearsal\.compatible-baseline') {
        throw "Sprint 8A Component rehearsal baseline Dockerfile is not a source-preserving derivative contract."
    }
    Write-Host "Sprint 8A Component rehearsal baseline builder self-test passed."
    return
}
if ($CurrentImage -cnotmatch $immutableImagePattern) {
    throw "CurrentImage must be an immutable image reference in name@sha256:<64 lowercase hex> form."
}
if ($OutputTag -notmatch '^[^\s@]+:[^\s@]+$') {
    throw "OutputTag must be a mutable local build tag; the emitted result is the immutable reference."
}

$baselineId = "source-preserving-$((Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ'))"
Push-Location $repoRoot
try {
    & docker build `
        --file $dockerfile `
        --build-arg "COMPONENT_BASE_IMAGE=$CurrentImage" `
        --build-arg "BASELINE_ID=$baselineId" `
        --tag $OutputTag `
        .
    if ($LASTEXITCODE -ne 0) { throw "Component rehearsal baseline image build failed." }
    $inspection = & docker image inspect $OutputTag | ConvertFrom-Json
    $imageId = [string]$inspection[0].Id
    if ($imageId -notmatch '^sha256:[0-9a-f]{64}$') {
        throw "Component rehearsal baseline did not resolve to one immutable local image ID."
    }
    $repository = ($OutputTag -split ':', 2)[0]
    "$repository@$imageId"
} finally {
    Pop-Location
}

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$modulePath = Join-Path $PSScriptRoot "tessara-validation-platform.psm1"
$adapterTemplatePath = Join-Path $PSScriptRoot `
    "validation-platform/fixtures/synthetic-adapter.json"
$liveComposeRelative = "scripts/validation-platform/fixtures/live-docker-compose.yaml"
$liveComposePath = Join-Path $repositoryRoot $liveComposeRelative
$image = "alpine/socat@sha256:4e625a62c9ea40ccbce93b9a4fcc6b41740a9f308389c216f34c88ce3abb275b"

Import-Module $modulePath -Force

$callerContext = (& docker context show).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($callerContext)) {
    throw "The live-Docker certification could not inspect the caller Docker context."
}
# The platform intentionally supplies an empty DOCKER_CONFIG, which selects
# Docker's built-in default context instead of caller-owned client state.
$context = "default"
$provider = & docker info --format json | ConvertFrom-Json -Depth 40
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace([string]$provider.ID) -or
    [string]::IsNullOrWhiteSpace([string]$provider.ServerVersion)) {
    throw "The live-Docker certification could not authenticate the Docker daemon."
}
$imageId = (& docker image inspect $image --format "{{.Id}}").Trim()
if ($LASTEXITCODE -ne 0 -or $imageId -cne ($image -replace '^.*@', '')) {
    throw "The required digest-pinned live-Docker certification image is not present exactly."
}

$evidenceRoot = Join-Path $repositoryRoot (
    "artifacts/validation-platform-certification/live-docker-" +
    [DateTimeOffset]::UtcNow.ToString("yyyyMMddTHHmmssfffZ") + "-" +
    [guid]::NewGuid().ToString("N").Substring(0, 8)
)
[IO.Directory]::CreateDirectory($evidenceRoot) | Out-Null
$adapter = Get-Content -Raw -LiteralPath $adapterTemplatePath | ConvertFrom-Json -Depth 100
foreach ($lane in @($adapter.lanes | Where-Object {
            [string]$_.topology.provider -ceq "docker-compose"
        })) {
    $lane.environment = @()
    $lane.topology.compose_file = $liveComposeRelative
    $lane.topology.provider_identity.context = $context
    $lane.topology.provider_identity.daemon_id = [string]$provider.ID
    $lane.topology.provider_identity.server_version = [string]$provider.ServerVersion
    $lane.topology.provider_identity.allowed_images = @($image)
}
$composeInput = @($adapter.execution_inputs | Where-Object {
        [string]$_.path -ceq "scripts/validation-platform/fixtures/synthetic-compose.yaml"
    })[0]
$composeInput.path = $liveComposeRelative
$adapterPath = Join-Path $evidenceRoot "live-docker-adapter.json"
[IO.File]::WriteAllText(
    $adapterPath,
    (($adapter | ConvertTo-Json -Depth 100 -Compress) + "`n"),
    [Text.UTF8Encoding]::new($false)
)

$candidate = [string](Get-TessaraValidationCandidateIdentity `
    -AdapterPath $adapterPath -RepositoryRoot $repositoryRoot).candidate_fingerprint
try {
    $producer = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -RepositoryRoot $repositoryRoot -LaneId "compose-create" `
        -CandidateFingerprint $candidate -EvidenceRoot $evidenceRoot
} catch {
    throw "Live-Docker producer failed. $($_.Exception.Message)`n$($_.ScriptStackTrace)"
}
$receiptPath = [string]$producer.cleanup_restoration.receipt.path
$consumer = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
    -RepositoryRoot $repositoryRoot -LaneId "compose-consume" `
    -CandidateFingerprint $candidate -EvidenceRoot $evidenceRoot `
    -TopologyReceiptPath $receiptPath

$project = [string]$consumer.compose_project
$residue = @(@(
    & docker container ls --all --filter "label=com.docker.compose.project=$project" `
        --format "{{.ID}}"
    & docker volume ls --filter "label=com.docker.compose.project=$project" `
        --format "{{.Name}}"
    & docker network ls --filter "label=com.docker.compose.project=$project" `
        --format "{{.ID}}"
) | Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) })
if ($LASTEXITCODE -ne 0 -or $residue.Count -ne 0) {
    throw "The live-Docker certification left project-owned engine residue."
}

[pscustomobject][ordered]@{
    schema_version = 1
    contract = "tessara.validation.live-docker-certification"
    state = "passed"
    context = $context
    daemon_id = [string]$provider.ID
    server_version = [string]$provider.ServerVersion
    image = $image
    producer_result = [string]$producer.evidence_path
    consumer_result = [string]$consumer.evidence_path
    evidence_root = $evidenceRoot
    residue_count = $residue.Count
} | ConvertTo-Json -Depth 20

[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6d/compose.yaml",
    [string]$ImportToken = "local-deploy-import-token",
    [string]$EvidenceDirectory = "artifacts/sprint-6d-closeout"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = Join-Path $repoRoot $ComposeFile
$workingDirectory = Join-Path $repoRoot "target/sprint-6d-bootstrap"
[IO.Directory]::CreateDirectory($workingDirectory) | Out-Null

function Write-Evidence {
    param(
        [Parameter(Mandatory)][string]$Filename,
        [Parameter(Mandatory)][object]$Value
    )
    if ([string]::IsNullOrWhiteSpace($EvidenceDirectory)) { return }
    $directory = if ([IO.Path]::IsPathRooted($EvidenceDirectory)) {
        [IO.Path]::GetFullPath($EvidenceDirectory)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceDirectory))
    }
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    $path = Join-Path $directory $Filename
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
        ($Value | ConvertTo-Json -Depth 100) + "`n"
    )
    [IO.File]::WriteAllBytes($path, $bytes)
    $digest = [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($bytes)
    ).ToLowerInvariant()
    [IO.File]::WriteAllText(
        "$path.sha256",
        "$digest`n",
        [Text.UTF8Encoding]::new($false)
    )
}

Push-Location $repoRoot
try {
    & .\scripts\bootstrap-sprint-6c-deployment.ps1 `
        -BaseUrl $BaseUrl `
        -ComposeFile $ComposeFile `
        -ImportToken $ImportToken
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6C prerequisite bootstrap failed."
    }

    $databaseContainer = (& docker compose -f $composePath ps -q postgres).Trim()
    if ($LASTEXITCODE -ne 0 -or $databaseContainer -notmatch "^[0-9a-f]{64}$") {
        throw "The Sprint 6D PostgreSQL container is not running."
    }

    $installationId = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
        "SELECT id FROM application_installations WHERE singleton=true").Trim()
    if ($LASTEXITCODE -ne 0 -or $installationId -notmatch "^[0-9a-f-]{36}$") {
        throw "The Sprint 6D installation identity is unavailable."
    }

    $referenceCount = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
        "SELECT count(*) FROM module_instances WHERE definition_id='tessara.reference.module-sdk' AND identity_state='live'").Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "The Sprint 6D reference inventory could not be read."
    }
    if ([int]$referenceCount -eq 1) {
        $snapshotJson = (& docker exec $databaseContainer psql -X -U tessara_bootstrap `
            -d tessara_core -Atc `
            "SELECT json_build_object('definition_id',instances.definition_id,'instance_id',instances.id,'installation_id',instances.installation_id,'enabled',instances.enabled,'installed',instances.installed,'deployed',instances.deployed,'configured',instances.configured,'ready',instances.ready,'release_version',releases.version,'manifest',releases.manifest,'configuration',instances.configuration)::text FROM module_instances instances JOIN module_releases releases ON releases.id=instances.release_id WHERE instances.definition_id='tessara.reference.module-sdk' AND instances.identity_state='live'").Trim()
        if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($snapshotJson)) {
            throw "Sprint 6D no-op state could not be captured."
        }
        $snapshot = $snapshotJson | ConvertFrom-Json
        Write-Evidence -Filename "bootstrap-second-noop.json" -Value ([pscustomobject][ordered]@{
            schema_version = 1
            checked_at = [DateTimeOffset]::UtcNow.ToString("o")
            result = "exact_noop"
            material_changes = 0
            reference = $snapshot
        })
        Write-Host "Sprint 6D bootstrap is an exact no-op; the current reference instance already exists."
        exit 0
    }
    if ([int]$referenceCount -ne 0) {
        throw "Sprint 6D reference inventory is ambiguous."
    }

    $desired = Get-Content -LiteralPath `
        (Join-Path $repoRoot "deploy/sprint-6b1/fixtures/deployment-v1.json") -Raw |
        ConvertFrom-Json
    $dashboardManifest = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-dashboard-module/manifest.json") -Raw |
        ConvertFrom-Json
    $referenceManifest = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-reference-module-sdk/manifest.json") -Raw |
        ConvertFrom-Json

    function Get-ComposeImageDigest {
        param([Parameter(Mandatory)][string]$Service)
        $imageId = (& docker compose -f $composePath images -q $Service).Trim()
        if ($imageId -match "^[0-9a-f]{64}$") {
            $imageId = "sha256:$imageId"
        }
        if ($LASTEXITCODE -ne 0 -or $imageId -notmatch "^sha256:[0-9a-f]{64}$") {
            throw "The Sprint 6D $Service image is unavailable as an immutable image ID."
        }
        $imageId
    }

    function Set-ManifestRuntimeImage {
        param(
            [Parameter(Mandatory)][object]$Manifest,
            [Parameter(Mandatory)][string]$Digest,
            [Parameter(Mandatory)][string]$Repository
        )
        $Manifest.deployment.declaration.runtime_image.digest = $Digest
        $Manifest.deployment.declaration.runtime_image.image_reference = "$Repository@$Digest"
        if ($null -ne $Manifest.deployment.declaration.migration_image) {
            $Manifest.deployment.declaration.migration_image.digest = $Digest
            $Manifest.deployment.declaration.migration_image.image_reference = "$Repository@$Digest"
        }
    }

    function Get-ManifestDigest {
        param([Parameter(Mandatory)][object]$Manifest)
        $bytes = [Text.Encoding]::UTF8.GetBytes(
            ($Manifest | ConvertTo-Json -Depth 100 -Compress)
        )
        "sha256:$([Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($bytes)
        ).ToLowerInvariant())"
    }

    $dashboardDigest = Get-ComposeImageDigest -Service "dashboards"
    $referenceDigest = Get-ComposeImageDigest -Service "reference-module-sdk"
    Set-ManifestRuntimeImage -Manifest $dashboardManifest -Digest $dashboardDigest `
        -Repository "local/tessara-dashboards"
    Set-ManifestRuntimeImage -Manifest $referenceManifest -Digest $referenceDigest `
        -Repository "local/tessara-reference-module-sdk"

    $dashboardModule = [pscustomobject]@{
        definition_id = $dashboardManifest.definition_id
        version = $dashboardManifest.release_version
        manifest = $dashboardManifest
        manifest_digest = Get-ManifestDigest -Manifest $dashboardManifest
        runtime_image = $dashboardDigest
        publisher = $dashboardManifest.publisher
        database_name = "tessara_module_dashboards"
        route_prefix = "/dashboards"
        configuration = [pscustomobject]@{
            display_label = "Dashboards"
            default_page_size = "25"
        }
    }
    $referenceModule = [pscustomobject]@{
        definition_id = $referenceManifest.definition_id
        version = $referenceManifest.release_version
        manifest = $referenceManifest
        manifest_digest = Get-ManifestDigest -Manifest $referenceManifest
        runtime_image = $referenceDigest
        publisher = $referenceManifest.publisher
        database_name = $null
        route_prefix = "/reference/module-sdk"
        configuration = [pscustomobject]@{
            display_label = "Module SDK Reference"
        }
    }

    $existingRevision = (& docker exec $databaseContainer psql -X -U tessara_bootstrap -d tessara_core -Atc `
        "SELECT COALESCE(max(revision),0) FROM deployment_receipts").Trim()
    if ($LASTEXITCODE -ne 0 -or $existingRevision -notmatch "^\d+$") {
        throw "The Sprint 6D deployment revision is unavailable."
    }
    $desired.installation_id = $installationId
    $desired.revision = [int64]$existingRevision + 1
    $desired.modules = @($desired.modules) + @($dashboardModule, $referenceModule)

    $desiredPath = Join-Path $workingDirectory "deployment-current.json"
    $planPath = Join-Path $workingDirectory "plan-current.json"
    $receiptPath = Join-Path $workingDirectory "receipt-current.json"
    [IO.File]::WriteAllText(
        $desiredPath,
        ($desired | ConvertTo-Json -Depth 100) + "`n",
        [Text.UTF8Encoding]::new($false)
    )
    foreach ($path in @($planPath, $receiptPath)) {
        if (Test-Path -LiteralPath $path) {
            Remove-Item -LiteralPath $path -Force
        }
    }

    & cargo run -q -p tessara-deploy -- plan $desiredPath $planPath
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6D deployment planning failed."
    }
    & cargo run -q -p tessara-deploy -- apply $desiredPath $planPath $receiptPath `
        "local:sprint-6d-bootstrap" ([DateTimeOffset]::UtcNow.ToString("o")) `
        $BaseUrl $ImportToken
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 6D deployment receipt import failed."
    }

    Write-Evidence -Filename "bootstrap-first.json" -Value ([pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        result = "applied"
        desired = Get-Content -LiteralPath $desiredPath -Raw | ConvertFrom-Json
        plan = Get-Content -LiteralPath $planPath -Raw | ConvertFrom-Json
        receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
    })
    Write-Host "Sprint 6D reference module registered for installation $installationId."
} finally {
    Pop-Location
}

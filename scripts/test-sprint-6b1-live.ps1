param([string]$BaseUrl = "http://127.0.0.1:8080")

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$composeFile = Join-Path $repoRoot "deploy\sprint-6b1\compose.yaml"
$fixtureRoot = Join-Path $repoRoot "deploy\sprint-6b1\fixtures"
$working = Join-Path $repoRoot "target\sprint-6b1-live"
$token = "local-deploy-import-token"
New-Item -ItemType Directory -Path $working -Force | Out-Null

function Invoke-Compose([string[]]$Arguments) {
    & docker compose -f $composeFile @Arguments
    if ($LASTEXITCODE -ne 0) { throw "docker compose $($Arguments -join ' ') failed" }
}

function Write-Deployment([int]$Revision, [string]$Version, [string]$ManifestCharacter, [string]$RuntimeCharacter, [string]$InstallationId) {
    $desired = Get-Content -Raw (Join-Path $fixtureRoot "deployment-v1.json") | ConvertFrom-Json
    $desired.installation_id = $InstallationId
    $desired.revision = $Revision
    $desired.modules[0].version = $Version
    $desired.modules[0].manifest_digest = "sha256:" + ($ManifestCharacter * 64)
    $desired.modules[0].runtime_image = "sha256:" + ($RuntimeCharacter * 64)
    $desired.modules[0].manifest.release_version = $Version
    $desired.modules[0].manifest.deployment.declaration.runtime_image.digest = $desired.modules[0].runtime_image
    $desired.modules[0].manifest.deployment.declaration.runtime_image.image_reference =
        "registry.example/tessara/scoped-records@$($desired.modules[0].runtime_image)"
    $path = Join-Path $working "deployment-$Revision.json"
    $desired | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $path -Encoding utf8

    return $path
}

Push-Location $repoRoot
try {
    $coreHealth = Invoke-WebRequest -UseBasicParsing "$BaseUrl/health"
    if ($coreHealth.StatusCode -ne 200) { throw "Core health failed" }
    $moduleReady = Invoke-WebRequest -UseBasicParsing "$BaseUrl/reference/scoped-records/health/ready" -SkipHttpErrorCheck
    if ($moduleReady.StatusCode -ne 204) { throw "Scoped Records readiness failed" }
    $modulePage = Invoke-WebRequest -UseBasicParsing "$BaseUrl/reference/scoped-records/"
    if ($modulePage.Content -notmatch "Scoped Records") { throw "Scoped Records product page failed" }

    $created = Invoke-RestMethod -Method Post -Uri "$BaseUrl/reference/scoped-records/api/records" -ContentType "application/json" -Body '{"label":"Acceptance record","scope":"sprint-6b1"}'
    if (-not $created.id) { throw "Scoped Records create failed" }

    $installationId = (& docker compose -f $composeFile exec -T postgres psql -U tessara_bootstrap -d tessara_core -Atc "SELECT id FROM application_installations WHERE singleton").Trim()
    if ($LASTEXITCODE -ne 0 -or -not $installationId) { throw "installation identity lookup failed" }

    $v1 = Write-Deployment 1 "1.0.0" "c" "d" $installationId
    $plan1 = Join-Path $working "plan-1.json"
    $receipt1 = Join-Path $working "receipt-1.json"
    foreach ($path in @($plan1, $receipt1)) { if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force } }
    cargo run -q -p tessara-deploy -- plan $v1 $plan1
    cargo run -q -p tessara-deploy -- apply $v1 $plan1 $receipt1 "local:acceptance" "2026-07-22T19:20:00Z" $BaseUrl $token
    if ($LASTEXITCODE -ne 0) { throw "initial live apply failed" }
    $first = Get-Content -Raw $receipt1 | ConvertFrom-Json

    $v2 = Write-Deployment 2 "1.1.0" "e" "f" $installationId
    $plan2 = Join-Path $working "plan-2.json"
    $receipt2 = Join-Path $working "receipt-2.json"
    Copy-Item -LiteralPath $receipt1 -Destination $receipt2 -Force
    cargo run -q -p tessara-deploy -- plan $v2 $plan2
    cargo run -q -p tessara-deploy -- apply $v2 $plan2 $receipt2 "local:acceptance" "2026-07-22T19:25:00Z" $BaseUrl $token
    if ($LASTEXITCODE -ne 0) { throw "upgrade live apply failed" }
    $second = Get-Content -Raw $receipt2 | ConvertFrom-Json
    if ($first.modules[0].instance_id -ne $second.modules[0].instance_id) { throw "upgrade changed durable instance identity" }
    if ($first.modules[0].release_id -eq $second.modules[0].release_id) { throw "upgrade did not change release identity" }

    $rollbackReceipt = Join-Path $working "receipt-rollback.json"
    cargo run -q -p tessara-deploy -- rollback $receipt2 $receipt1 $rollbackReceipt "local:acceptance" "2026-07-22T19:30:00Z" $BaseUrl $token
    if ($LASTEXITCODE -ne 0) { throw "rollback failed" }

    $projection = (& docker compose -f $composeFile exec -T postgres psql -U tessara_bootstrap -d tessara_core -Atc "SELECT count(*) || ':' || min(revision) || ':' || max(revision) FROM deployment_receipts").Trim()
    if ($projection -ne "3:1:3") { throw "unexpected receipt projection '$projection'" }

    & docker compose -f $composeFile exec -T -e PGPASSWORD=local-scoped-runtime postgres psql -U tessara_scoped_runtime -d tessara_core -c "SELECT 1" 2>$null
    if ($LASTEXITCODE -eq 0) { throw "module runtime role crossed into the Core database" }

    Invoke-Compose @("stop", "scoped-records")
    $coreDuringFailure = Invoke-WebRequest -UseBasicParsing "$BaseUrl/health"
    if ($coreDuringFailure.StatusCode -ne 200) { throw "Core failed with module unavailable" }
    $moduleFallback = Invoke-WebRequest -UseBasicParsing "$BaseUrl/reference/scoped-records/" -SkipHttpErrorCheck
    if ($moduleFallback.StatusCode -notin @(502, 503) -or $moduleFallback.Content -notmatch "Module temporarily unavailable") {
        throw "Core-rendered module fallback failed"
    }
    Invoke-Compose @("start", "scoped-records")
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        $ready = Invoke-WebRequest -UseBasicParsing "$BaseUrl/reference/scoped-records/health/ready" -SkipHttpErrorCheck
        if ($ready.StatusCode -eq 204) { break }
        Start-Sleep -Seconds 1
    }
    $records = Invoke-RestMethod "$BaseUrl/reference/scoped-records/api/records"
    if (-not ($records | Where-Object id -eq $created.id)) { throw "module data did not survive restart" }
    Write-Host "Sprint 6B1 live acceptance passed."
}
finally {
    Pop-Location
}

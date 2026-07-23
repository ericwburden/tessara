param(
    [string]$WorkingDirectory = (Join-Path $PSScriptRoot "..\target\sprint-6b1-contract")
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$fixtureRoot = Join-Path $repoRoot "deploy\sprint-6b1\fixtures"
$working = [System.IO.Path]::GetFullPath($WorkingDirectory)

New-Item -ItemType Directory -Path $working -Force | Out-Null
$plan = Join-Path $working "plan.json"
$receipt = Join-Path $working "receipt.json"
foreach ($artifact in @($plan, $receipt)) {
    if (Test-Path -LiteralPath $artifact) {
        Remove-Item -LiteralPath $artifact -Force
    }
}

Push-Location $repoRoot
try {
    cargo run -q -p tessara-deploy -- validate (Join-Path $fixtureRoot "deployment-v1.json")
    if ($LASTEXITCODE -ne 0) { throw "validate failed" }
    cargo run -q -p tessara-deploy -- plan (Join-Path $fixtureRoot "deployment-v1.json") $plan
    if ($LASTEXITCODE -ne 0) { throw "plan failed" }
    cargo run -q -p tessara-deploy -- apply (Join-Path $fixtureRoot "deployment-v1.json") $plan $receipt "local:acceptance" "2026-07-22T18:30:00Z"
    if ($LASTEXITCODE -ne 0) { throw "apply failed" }
    cargo run -q -p tessara-deploy -- status $receipt | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "status failed" }

    $receiptJson = Get-Content -Raw $receipt | ConvertFrom-Json
    if ($receiptJson.revision -ne 1) { throw "unexpected receipt revision" }
    if ($receiptJson.modules[0].definition_id -ne "tessara.reference.scoped-records") { throw "reference module missing from receipt" }
    if (-not $receiptJson.modules[0].instance_id) { throw "durable instance identity missing" }
    Write-Host "Sprint 6B1 contract acceptance passed."
}
finally {
    Pop-Location
}

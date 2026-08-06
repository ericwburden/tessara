[CmdletBinding()]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [Parameter(Mandatory = $true)][string]$BaselineImage,
    [Parameter(Mandatory = $true)][string]$CandidateImage,
    [Parameter(Mandatory = $true)][string]$CurrentImage,
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath = "target/sprint-8a-upgrade/component-upgrade-rollback.json",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-8a"
$immutableImagePattern = '^[^\s@]+@sha256:[0-9a-f]{64}$'
$unrelatedServices = @("gateway", "postgres", "core", "supervisor", "scoped-records", "dashboards")
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

function Assert-ImmutableImage([string]$Image, [string]$Name) {
    if ($Image -cnotmatch $immutableImagePattern) {
        throw "$Name must be an immutable image reference in name@sha256:<64 lowercase hex> form."
    }
}

function Assert-DistinctUpgradeImages([string]$Baseline, [string]$Candidate) {
    $baselineDigest = ($Baseline -split "@", 2)[1]
    $candidateDigest = ($Candidate -split "@", 2)[1]
    if ($baselineDigest -ceq $candidateDigest) {
        throw "BaselineImage and CandidateImage must identify distinct immutable image digests."
    }
}

function Get-ServiceIdentity([string[]]$Services) {
    $identity = [ordered]@{}
    foreach ($service in $Services) {
        $containerId = (& docker compose -f $composePath --profile reference ps -q $service).Trim()
        if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($containerId)) {
            throw "Sprint 8A service '$service' is not running."
        }
        $inspection = & docker inspect $containerId | ConvertFrom-Json
        $identity[$service] = [ordered]@{
            container_id = [string]$inspection[0].Id
            image_id = [string]$inspection[0].Image
            restart_count = [uint64]$inspection[0].RestartCount
        }
    }
    return $identity
}

function Get-ComponentInventory {
    $token = $null
    $token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
    try {
        $response = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/components" -Token $token
        if ($response.status -ne 200) { throw "Component inventory returned HTTP $($response.status)." }
        return @($response.body | ConvertFrom-Json | Sort-Object slug | ForEach-Object {
            [ordered]@{
                id = [string]$_.component_id
                slug = [string]$_.slug
                current_version_id = [string]$_.current_version.component_version_id
                reference = $_.current_version.reference
            }
        })
    } finally {
        if ($token) {
            [void](Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token)
        }
    }
}

function Wait-ComponentHealthy {
    for ($attempt = 1; $attempt -le 60; $attempt++) {
        $containerId = (& docker compose -f $composePath --profile reference ps -q components).Trim()
        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($containerId)) {
            $status = (& docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' $containerId).Trim()
            if ($LASTEXITCODE -eq 0 -and $status -ceq "healthy") { return }
            if ($status -in "dead", "exited") { throw "Component container entered '$status'." }
        }
        Start-Sleep -Seconds 1
    }
    throw "Component did not become healthy within 60 seconds."
}

function Set-ComponentImage([string]$Image, [string]$Stage) {
    $stagePreviousImage = $env:TESSARA_COMPONENT_IMAGE
    try {
        $env:TESSARA_COMPONENT_IMAGE = $Image
        & docker compose -f $composePath --profile reference run --rm components-migrate
        if ($LASTEXITCODE -ne 0) { throw "$Stage Component migration failed." }
        & docker compose -f $composePath --profile reference up -d --no-deps components
        if ($LASTEXITCODE -ne 0) { throw "$Stage Component switch failed." }
    } finally {
        if ($null -eq $stagePreviousImage) {
            Remove-Item Env:TESSARA_COMPONENT_IMAGE -ErrorAction SilentlyContinue
        } else {
            $env:TESSARA_COMPONENT_IMAGE = $stagePreviousImage
        }
    }
    Wait-ComponentHealthy
    & (Join-Path $PSScriptRoot "smoke-sprint-8a.ps1") -BaseUrl $BaseUrl | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "$Stage Sprint 8A smoke failed." }
}

Assert-ImmutableImage $BaselineImage "BaselineImage"
Assert-ImmutableImage $CandidateImage "CandidateImage"
Assert-ImmutableImage $CurrentImage "CurrentImage"
Assert-DistinctUpgradeImages -Baseline $BaselineImage -Candidate $CandidateImage
if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) { throw "Compose file not found: $composePath" }

Push-Location $repoRoot
$previousImage = $env:TESSARA_COMPONENT_IMAGE
try {
    $configuration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or $configuration.name -cne $expectedProject) {
        throw "Upgrade/rollback runner is restricted to the exact $expectedProject Compose project."
    }
    if ($SelfTest) {
        if (-not $configuration.services.components -or -not $configuration.services.'components-migrate') {
            throw "Sprint 8A Compose does not declare both Component runtime and migration services."
        }
        $sameDigestRejected = $false
        try {
            Assert-DistinctUpgradeImages `
                -Baseline "local/baseline@sha256:$('a' * 64)" `
                -Candidate "local/candidate@sha256:$('a' * 64)"
        } catch {
            $sameDigestRejected = $true
        }
        if (-not $sameDigestRejected) {
            throw "Sprint 8A upgrade self-test accepted two names for the same image digest."
        }
        $env:TESSARA_COMPONENT_IMAGE = "temporary-component-image@sha256:$('b' * 64)"
        try {
            $defaultConfiguration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
            if ([string]$defaultConfiguration.services.components.image -cne [string]$env:TESSARA_COMPONENT_IMAGE) {
                throw "Sprint 8A upgrade self-test could not establish the transient Compose image override."
            }
        } finally {
            Remove-Item Env:TESSARA_COMPONENT_IMAGE -ErrorAction SilentlyContinue
        }
        Write-Host "Sprint 8A Component upgrade/rollback self-test passed."
        return
    }

    $baselineUnrelated = Get-ServiceIdentity $unrelatedServices
    Set-ComponentImage $BaselineImage "baseline"
    $baselineInventory = Get-ComponentInventory

    Set-ComponentImage $CandidateImage "candidate upgrade"
    $candidateInventory = Get-ComponentInventory
    Set-ComponentImage $BaselineImage "baseline rollback"
    $rollbackInventory = Get-ComponentInventory
    Set-ComponentImage $CurrentImage "current-release restoration"
    $restoredInventory = Get-ComponentInventory
    $finalUnrelated = Get-ServiceIdentity $unrelatedServices

    $baselineInventoryJson = $baselineInventory | ConvertTo-Json -Depth 30 -Compress
    foreach ($observed in @($candidateInventory, $rollbackInventory, $restoredInventory)) {
        if (($observed | ConvertTo-Json -Depth 30 -Compress) -cne $baselineInventoryJson) {
            throw "Component resource identity changed during upgrade, rollback, or restoration."
        }
    }
    if (($finalUnrelated | ConvertTo-Json -Depth 10 -Compress) -cne ($baselineUnrelated | ConvertTo-Json -Depth 10 -Compress)) {
        throw "An unrelated service image, container identity, or restart count changed during the Component-only exercise."
    }

    $result = [ordered]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-8a.component-upgrade-rollback"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_identity = [ordered]@{
            commit = (& git -C $repoRoot rev-parse HEAD).Trim()
            tree = (& git -C $repoRoot rev-parse "HEAD^{tree}").Trim()
            dirty = @(& git -C $repoRoot status --porcelain=v1).Count -ne 0
        }
        project = $expectedProject
        baseline_image = $BaselineImage
        candidate_image = $CandidateImage
        rollback_image = $BaselineImage
        restored_current_image = $CurrentImage
        component_inventory = $restoredInventory
        unrelated_services_before = $baselineUnrelated
        unrelated_services_after = $finalUnrelated
        passed = $true
    }
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite | Out-Null
    $result | ConvertTo-Json -Depth 30
} catch {
    $exerciseFailure = $_.Exception.Message
    if (-not $SelfTest) {
        try {
            Set-ComponentImage $CurrentImage "failure-path current-release restoration"
        } catch {
            throw "Component upgrade/rollback failed: $exerciseFailure Current-release restoration also failed: $($_.Exception.Message)"
        }
    }
    throw $exerciseFailure
} finally {
    if ($null -eq $previousImage) {
        Remove-Item Env:TESSARA_COMPONENT_IMAGE -ErrorAction SilentlyContinue
    } else {
        $env:TESSARA_COMPONENT_IMAGE = $previousImage
    }
    Pop-Location
}

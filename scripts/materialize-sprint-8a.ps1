[CmdletBinding(SupportsShouldProcess, ConfirmImpact = "High")]
param(
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$CoreUrl = "http://127.0.0.1:8088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$VerifyNoOp
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$expectedProject = "tessara-sprint-8a"
$installationId = "01980000-0000-7000-8000-00000000008a"
$componentOwner = "142a1ece-f74b-85f6-8ca0-92f4a02e9409"
$evidenceDirectory = Join-Path $repoRoot "target/sprint-8a-bootstrap/reference"
$receiptPath = Join-Path $evidenceDirectory "apply-response.json"

if (-not $AuthorizeDisposableReset) {
    throw "Sprint 8A materialization is destructive. Re-run with -AuthorizeDisposableReset only for the disposable tessara-sprint-8a project."
}
if (-not (Test-Path -LiteralPath $composePath)) { throw "Compose file not found: $composePath" }

Push-Location $repoRoot
try {
    $configuration = & docker compose -f $composePath --profile reference config --format json | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw "Sprint 8A Compose configuration is invalid." }
    if ($configuration.name -cne $expectedProject) {
        throw "Refusing destructive reset for unexpected Compose project '$($configuration.name)'."
    }
    $expectedPostgresInit = [IO.Path]::GetFullPath((Join-Path $repoRoot "deploy/sprint-8a/postgres-init.sh"))
    $postgresInitMounts = @($configuration.services.postgres.volumes | Where-Object {
        $_.target -ceq "/docker-entrypoint-initdb.d/10-tessara-databases.sh"
    })
    if ($postgresInitMounts.Count -ne 1 -or
        [IO.Path]::GetFullPath([string]$postgresInitMounts[0].source) -cne $expectedPostgresInit) {
        throw "Refusing Sprint 8A materialization because PostgreSQL is not bound to the exact Sprint 8A initialization contract."
    }
    $unexpectedVolumes = @($configuration.volumes.PSObject.Properties | Where-Object {
        $name = if ($_.Value.name) { [string]$_.Value.name } else { "${expectedProject}_$($_.Name)" }
        -not $name.StartsWith("$expectedProject`_", [StringComparison]::Ordinal)
    })
    if ($unexpectedVolumes.Count -gt 0) {
        throw "Refusing reset because Compose resolves a named volume outside the $expectedProject namespace."
    }
    $resolvedVolumeNames = @($configuration.volumes.PSObject.Properties | ForEach-Object {
        if ($_.Value.name) { [string]$_.Value.name } else { "${expectedProject}_$($_.Name)" }
    })
    $resolvedNetworkNames = @($configuration.networks.PSObject.Properties | ForEach-Object {
        if ($_.Value.name) { [string]$_.Value.name } else { "${expectedProject}_$($_.Name)" }
    })
    if (-not $PSCmdlet.ShouldProcess("$expectedProject containers and named volumes", "Fresh-materialize Sprint 8A from empty owner databases")) { return }

    try {
        & (Join-Path $PSScriptRoot "bootstrap-sprint-7a-composition.ps1") `
            -Composition reference `
            -ComposeFile $ComposeFile `
            -CoreUrl $CoreUrl `
            -SupervisorUrl $SupervisorUrl `
            -ReplaceExisting `
            -Confirm:$false `
            -DeploymentDirectory "sprint-8a" `
            -ExpectedProject $expectedProject `
            -InstallationId $installationId `
            -RuntimeLabel "sprint-8a" `
            -AdditionalBuildServices @("components") `
            -AdditionalExpectedNavigationHrefs @("/components") `
            -SkipLegacySeed `
            -SkipBuild:$SkipBuild
        if ($LASTEXITCODE -ne 0) { throw "Sprint 8A composition materialization failed." }

        $applyResponse = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
        $receipt = $applyResponse.receipt
        if (-not $receipt) { throw "Sprint 8A apply response is missing its installation receipt." }
        $componentReceipt = @($receipt.bootstrap_receipts | Where-Object owner -eq "tessara.components") | Select-Object -First 1
        if (-not $componentReceipt) { throw "Component owner bootstrap receipt is missing." }
        if (@($componentReceipt.resource_ids.PSObject.Properties).Count -ne 6) {
            throw "Component owner bootstrap did not return all six canonical Component kinds."
        }
        foreach ($resource in $componentReceipt.resource_ids.PSObject.Properties) {
            $reference = $resource.Value | ConvertFrom-Json
            if ($reference.reference.owner.kind -cne "module_instance" `
                -or $reference.reference.owner.module_instance_id -cne $componentOwner `
                -or $reference.reference.resource_type -cne "tessara.components.component_version") {
                throw "Component read-back '$($resource.Name)' is not a Sprint 8A v3 module-instance reference."
            }
        }
        $dashboardReceipt = @($receipt.bootstrap_receipts | Where-Object owner -eq "tessara.dashboards") | Select-Object -First 1
        if (-not $dashboardReceipt) { throw "Dashboard owner bootstrap receipt is missing." }

        if ($VerifyNoOp) {
            $firstBootstrapResults = @{}
            foreach ($ownerReceipt in @($receipt.bootstrap_receipts)) {
                $firstBootstrapResults[[string]$ownerReceipt.owner] = [ordered]@{
                    result_digest = [string]$ownerReceipt.result_digest
                    resource_ids = ($ownerReceipt.resource_ids | ConvertTo-Json -Depth 20 -Compress)
                }
            }
            & (Join-Path $PSScriptRoot "bootstrap-sprint-7a-composition.ps1") `
                -Composition reference -ComposeFile $ComposeFile -CoreUrl $CoreUrl -SupervisorUrl $SupervisorUrl `
                -DeploymentDirectory "sprint-8a" -ExpectedProject $expectedProject -InstallationId $installationId `
                -RuntimeLabel "sprint-8a" -AdditionalBuildServices @("components") `
                -AdditionalExpectedNavigationHrefs @("/components") `
                -SkipLegacySeed -SkipBuild -Confirm:$false
            if ($LASTEXITCODE -ne 0) { throw "Sprint 8A no-op materialization verification failed." }
            $secondApplyResponse = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
            $secondReceipt = $secondApplyResponse.receipt
            if (-not $secondReceipt) { throw "Sprint 8A no-op apply response is missing its installation receipt." }
            if (-not $secondReceipt.no_op) {
                throw "Sprint 8A unchanged second apply did not report a semantic no-op."
            }
            foreach ($ownerReceipt in @($secondReceipt.bootstrap_receipts)) {
                $first = $firstBootstrapResults[[string]$ownerReceipt.owner]
                if (-not $first `
                    -or $ownerReceipt.changed `
                    -or [string]$ownerReceipt.result_digest -cne $first.result_digest `
                    -or ($ownerReceipt.resource_ids | ConvertTo-Json -Depth 20 -Compress) -cne $first.resource_ids) {
                    throw "Sprint 8A owner bootstrap '$($ownerReceipt.owner)' was not an exact semantic no-op."
                }
            }
            if (@($secondReceipt.bootstrap_receipts).Count -ne $firstBootstrapResults.Count) {
                throw "Sprint 8A no-op apply returned a different owner receipt set."
            }
        }
    } catch {
        $materializationMessage = $_.Exception.Message
        [IO.Directory]::CreateDirectory($evidenceDirectory) | Out-Null
        $serviceLogPath = Join-Path $evidenceDirectory "failed-topology-services.log"
        $serviceLogs = & docker compose -f $composePath --profile reference logs --no-color --timestamps 2>&1
        $serviceLogExitCode = $LASTEXITCODE
        [IO.File]::WriteAllLines($serviceLogPath, @($serviceLogs | ForEach-Object { [string]$_ }), [Text.UTF8Encoding]::new($false))
        $teardownCompleted = $false
        $teardownMessage = $null
        try {
            & docker compose -f $composePath --profile reference down --volumes --remove-orphans
            if ($LASTEXITCODE -ne 0) { throw "Docker Compose teardown returned exit code $LASTEXITCODE." }
            $remainingContainers = @(& docker ps -aq --filter "label=com.docker.compose.project=$expectedProject" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
            if ($LASTEXITCODE -ne 0 -or $remainingContainers.Count -ne 0) {
                throw "Sprint 8A containers remain after failed-attempt teardown."
            }
            $existingVolumes = @(& docker volume ls --format '{{.Name}}')
            if ($LASTEXITCODE -ne 0) { throw "Docker volume inventory failed after teardown." }
            $remainingVolumes = @($resolvedVolumeNames | Where-Object { $existingVolumes -contains $_ })
            if ($remainingVolumes.Count -ne 0) {
                throw "Sprint 8A volumes remain after teardown: $($remainingVolumes -join ', ')."
            }
            $existingNetworks = @(& docker network ls --format '{{.Name}}')
            if ($LASTEXITCODE -ne 0) { throw "Docker network inventory failed after teardown." }
            $remainingNetworks = @($resolvedNetworkNames | Where-Object { $existingNetworks -contains $_ })
            if ($remainingNetworks.Count -ne 0) {
                throw "Sprint 8A networks remain after teardown: $($remainingNetworks -join ', ')."
            }
            $teardownCompleted = $true
        } catch {
            $teardownMessage = $_.Exception.Message
        }
        $failure = [ordered]@{
            schema_version = 1
            project = $expectedProject
            failed_at = [DateTimeOffset]::UtcNow.ToString("o")
            message = $materializationMessage
            failed_topology_service_log = [IO.Path]::GetRelativePath($repoRoot, $serviceLogPath).Replace("\", "/")
            failed_topology_service_log_exit_code = $serviceLogExitCode
            teardown_completed = $teardownCompleted
            teardown_message = $teardownMessage
            retained_partial_topology = -not $teardownCompleted
        }
        [IO.File]::WriteAllText((Join-Path $evidenceDirectory "materialization-failure.json"), ($failure | ConvertTo-Json -Depth 10) + "`n", [Text.UTF8Encoding]::new($false))
        if (-not $teardownCompleted) {
            throw "Sprint 8A materialization failed: $materializationMessage Teardown also failed: $teardownMessage"
        }
        throw "Sprint 8A materialization failed and its exact disposable topology was removed: $materializationMessage"
    }

    Write-Host "Sprint 8A fresh materialization completed with Component owner read-back verified."
    Write-Host "Receipt: $receiptPath"
} finally {
    Pop-Location
}

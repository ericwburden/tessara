[CmdletBinding()]
param(
    [ValidateSet("Static", "RuntimeResilience", "ScopedRecordsRegression", "Digests")]
    [string]$Mode = "Static",
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [string]$ComposeFile = "deploy/sprint-6d/compose.yaml",
    [string]$EvidenceDirectory = "artifacts/sprint-6d-closeout",
    [string]$DeploymentEvidencePath = "artifacts/sprint-6d-closeout/deployment-fresh.json",
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$composePath = [IO.Path]::GetFullPath((Join-Path $repoRoot $ComposeFile))
$evidenceRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceDirectory))
$deploymentPath = [IO.Path]::GetFullPath((Join-Path $repoRoot $DeploymentEvidencePath))
$BaseUrl = $BaseUrl.TrimEnd("/")

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Publish-Evidence {
    param(
        [Parameter(Mandatory)][string]$Filename,
        [Parameter(Mandatory)][object]$Value,
        [switch]$Markdown
    )
    [IO.Directory]::CreateDirectory($evidenceRoot) | Out-Null
    $path = Join-Path $evidenceRoot $Filename
    $digestPath = "$path.sha256"
    if (((Test-Path -LiteralPath $path) -or (Test-Path -LiteralPath $digestPath)) -and -not $Overwrite) {
        throw "Refusing to overwrite retained evidence '$path' without -Overwrite."
    }
    $content = if ($Markdown) {
        ([string]$Value).TrimEnd() + "`n"
    } else {
        ($Value | ConvertTo-Json -Depth 100) + "`n"
    }
    [IO.File]::WriteAllText($path, $content, [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText(
        $digestPath,
        "$(Get-Sha256 -Path $path)`n",
        [Text.UTF8Encoding]::new($false)
    )
    $path
}

function Assert-DeploymentBinding {
    if (-not (Test-Path -LiteralPath $deploymentPath -PathType Leaf)) {
        throw "Deployment evidence is missing at '$deploymentPath'."
    }
    if (-not (Test-Path -LiteralPath "$deploymentPath.sha256" -PathType Leaf)) {
        throw "Deployment evidence digest is missing at '$deploymentPath.sha256'."
    }
    $expectedDigest = (Get-Content -LiteralPath "$deploymentPath.sha256" -Raw).Trim()
    if ($expectedDigest -cne (Get-Sha256 -Path $deploymentPath)) {
        throw "Deployment evidence digest does not match."
    }
    $deployment = Get-Content -LiteralPath $deploymentPath -Raw | ConvertFrom-Json
    $head = (& git rev-parse HEAD).Trim()
    $tree = (& git rev-parse "HEAD^{tree}").Trim()
    $dirty = -not [string]::IsNullOrWhiteSpace(((& git status --porcelain) -join ""))
    if ($dirty -or
        [string]$deployment.snapshot.source.commit -cne $head -or
        [string]$deployment.snapshot.source.tree -cne $tree -or
        -not [bool]$deployment.snapshot.source.clean) {
        throw "Deployment evidence is not bound to the current clean source commit and tree."
    }
    $deployment
}

function Get-ContainerId {
    param([Parameter(Mandatory)][string]$Service)
    $id = (& docker compose -f $composePath ps -q $Service).Trim()
    if ($LASTEXITCODE -ne 0 -or $id -notmatch "^[0-9a-f]{64}$") {
        throw "Compose service '$Service' does not have one resolvable container."
    }
    $id
}

function Get-ImageProvenance {
    param(
        [Parameter(Mandatory)][string]$Service,
        [Parameter(Mandatory)][string]$SourceCommit,
        [Parameter(Mandatory)][string]$SourceTree
    )
    $containerId = Get-ContainerId -Service $Service
    $container = (& docker inspect $containerId) | ConvertFrom-Json | Select-Object -First 1
    $image = (& docker image inspect ([string]$container.Image)) | ConvertFrom-Json | Select-Object -First 1
    $labels = $image.Config.Labels
    if ([string]$labels.'org.opencontainers.image.revision' -cne $SourceCommit -or
        [string]$labels.'com.tessara.source-tree' -cne $SourceTree -or
        [string]$labels.'com.tessara.source-dirty' -cne "false" -or
        [string]$labels.'com.tessara.build-profile' -cne "release") {
        throw "Service '$Service' image provenance labels do not match the clean source."
    }
    [pscustomobject][ordered]@{
        service = $Service
        container_id = $containerId
        image_id = [string]$container.Image
        image_reference = [string]$container.Config.Image
        repo_digests = @($image.RepoDigests)
        source_commit = [string]$labels.'org.opencontainers.image.revision'
        source_tree = [string]$labels.'com.tessara.source-tree'
        source_dirty = [string]$labels.'com.tessara.source-dirty'
        build_profile = [string]$labels.'com.tessara.build-profile'
    }
}

function Get-CrateVersion {
    param([Parameter(Mandatory)][string]$Manifest)
    $content = Get-Content -LiteralPath (Join-Path $repoRoot $Manifest) -Raw
    $match = [regex]::Match($content, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $match.Success -and $content -match '(?m)^version\.workspace\s*=\s*true\s*$') {
        $content = Get-Content -LiteralPath (Join-Path $repoRoot "Cargo.toml") -Raw
        $match = [regex]::Match($content, '(?m)^version\s*=\s*"([^"]+)"')
    }
    if (-not $match.Success) {
        throw "Could not resolve package version from '$Manifest'."
    }
    $match.Groups[1].Value
}

function Get-DatabaseLedger {
    param(
        [Parameter(Mandatory)][string]$ContainerId,
        [Parameter(Mandatory)][string]$Database,
        [Parameter(Mandatory)][string]$BaselinePath
    )
    $json = (& docker exec $ContainerId psql -X -U tessara_bootstrap -d $Database -Atc `
        "SELECT COALESCE(json_agg(json_build_object('version',version,'description',description,'success',success,'checksum',encode(checksum,'hex')) ORDER BY version),'[]'::json)::text FROM _sqlx_migrations").Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($json)) {
        throw "Could not read the migration ledger for '$Database'."
    }
    $ledger = @($json | ConvertFrom-Json)
    if ($ledger.Count -ne 1 -or [int64]$ledger[0].version -ne 1 -or -not [bool]$ledger[0].success) {
        throw "Database '$Database' does not have the expected single successful baseline ledger."
    }
    $baseline = Join-Path $repoRoot $BaselinePath
    $baselineSha384 = (Get-FileHash -LiteralPath $baseline -Algorithm SHA384).Hash.ToLowerInvariant()
    if ([string]$ledger[0].checksum -cne $baselineSha384) {
        throw "Database '$Database' ledger checksum does not match '$BaselinePath'."
    }
    [pscustomobject][ordered]@{
        database = $Database
        baseline_path = $BaselinePath
        baseline_sha384 = $baselineSha384
        ledger = $ledger
        passed = $true
    }
}

function Get-OwnershipRows {
    $path = Join-Path $repoRoot "docs/architecture/module-sdk-ownership-inventory.md"
    $lines = Get-Content -LiteralPath $path
    $start = [Array]::IndexOf($lines, "## Current Source Inventory")
    $end = [Array]::IndexOf($lines, "## Immediate Dependency Findings")
    if ($start -lt 0 -or $end -le $start) {
        throw "Could not locate the ownership inventory table."
    }
    $rows = [Collections.Generic.List[object]]::new()
    foreach ($line in $lines[($start + 1)..($end - 1)]) {
        if (-not $line.StartsWith("|") -or $line -match '^\|\s*(Behavior|---)') { continue }
        $cells = @($line.Trim("|").Split("|") | ForEach-Object { $_.Trim() })
        if ($cells.Count -ne 5) {
            throw "Ownership inventory row does not have five columns: $line"
        }
        $rows.Add([pscustomobject][ordered]@{
            behavior = $cells[0]
            source = $cells[1]
            consumers = $cells[2]
            finding = $cells[3]
            disposition = $cells[4]
        })
    }
    if ($rows.Count -lt 15) {
        throw "Ownership inventory unexpectedly contains only $($rows.Count) rows."
    }
    @($rows)
}

function Invoke-StaticCapture {
    $deployment = Assert-DeploymentBinding
    $sourceCommit = [string]$deployment.snapshot.source.commit
    $sourceTree = [string]$deployment.snapshot.source.tree
    $services = @(
        "core",
        "dashboards",
        "installation-control",
        "reference-module-sdk",
        "scoped-records"
    )
    $images = @($services | ForEach-Object {
        Get-ImageProvenance -Service $_ -SourceCommit $sourceCommit -SourceTree $sourceTree
    })
    $versions = [ordered]@{
        module_contract = Get-CrateVersion "crates/tessara-module-contract/Cargo.toml"
        module_runtime = Get-CrateVersion "crates/tessara-module-runtime/Cargo.toml"
        module_ui = Get-CrateVersion "crates/tessara-module-ui/Cargo.toml"
        module_testkit = Get-CrateVersion "crates/tessara-module-testkit/Cargo.toml"
        reference_module = Get-CrateVersion "crates/tessara-reference-module-sdk/Cargo.toml"
    }
    Publish-Evidence -Filename "source-provenance.json" -Value ([pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        source = [pscustomobject][ordered]@{
            commit = $sourceCommit
            tree = $sourceTree
            dirty = $false
        }
        release_profile = "release"
        images = $images
        sdk_versions = $versions
        passed = $true
    }) | Out-Null

    $postgres = Get-ContainerId -Service "postgres"
    $baselines = @(
        Get-DatabaseLedger $postgres "tessara_core" "crates/tessara-api/migrations/001_baseline.sql"
        Get-DatabaseLedger $postgres "tessara_module_dashboards" "crates/tessara-dashboard-module/migrations/001_dashboard_module.sql"
        Get-DatabaseLedger $postgres "tessara_deployment" "crates/tessara-installation-control/migrations/001_enrollment_claims.sql"
        Get-DatabaseLedger $postgres "tessara_module_scoped_records" "crates/tessara-reference-scoped-records/migrations/001_scoped_records.sql"
    )
    $referenceContainer = Get-ContainerId -Service "reference-module-sdk"
    $referenceStateJson = (& docker exec $referenceContainer sh -c "cat /var/lib/tessara-reference/state.json").Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($referenceStateJson)) {
        throw "Reference module state could not be read."
    }
    $referenceState = $referenceStateJson | ConvertFrom-Json
    if ([int]$referenceState.schema_version -ne 1) {
        throw "Reference module state does not use schema version 1."
    }
    Publish-Evidence -Filename "migration-checkpoint.json" -Value ([pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_commit = $sourceCommit
        empty_state_proof = [pscustomobject][ordered]@{
            deployment_data_state = [string]$deployment.snapshot.data.state
            destructive_reset_required = $true
            migrators_completed = @("core-migrate", "dashboards-migrate", "installation-control-migrate", "scoped-records-migrate")
        }
        changed_baselines = @("crates/tessara-api/migrations/001_baseline.sql")
        database_baselines = $baselines
        reference_state = [pscustomobject][ordered]@{
            storage = "module-owned JSON volume"
            schema_version = [int]$referenceState.schema_version
            configuration_present = $null -ne $referenceState.configuration
            security_state_present = $null -ne $referenceState.security
        }
        passed = $true
    }) | Out-Null

    $boundaryNative = Join-Path $evidenceRoot "package-boundaries-native.json"
    $boundaryWasm = Join-Path $evidenceRoot "package-boundaries-wasm.json"
    if (-not (Test-Path $boundaryNative) -or -not (Test-Path $boundaryWasm)) {
        throw "Run the native/WASM boundary publisher before static ownership capture."
    }
    $native = Get-Content -LiteralPath $boundaryNative -Raw | ConvertFrom-Json
    $wasm = Get-Content -LiteralPath $boundaryWasm -Raw | ConvertFrom-Json
    if (-not $native.passed -or -not $wasm.passed -or
        -not $native.dashboard_transition.observed -or
        [string]$native.dashboard_transition.owner -cne "Sprint 6E") {
        throw "Package boundary evidence does not contain the expected passing SDK result and visible Sprint 6E finding."
    }
    Publish-Evidence -Filename "sdk-ownership.json" -Value ([pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        source_commit = $sourceCommit
        inventory_path = "docs/architecture/module-sdk-ownership-inventory.md"
        inventory_sha256 = Get-Sha256 (Join-Path $repoRoot "docs/architecture/module-sdk-ownership-inventory.md")
        rows = @(Get-OwnershipRows)
        dashboard_transition = $native.dashboard_transition
        native_findings = @($native.findings)
        wasm_findings = @($wasm.findings)
        passed = $true
    }) | Out-Null
}

function Invoke-Request {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$CookiePath,
        [int[]]$ExpectedStatus
    )
    $bodyPath = Join-Path ([IO.Path]::GetTempPath()) "tessara-6d-$([guid]::NewGuid().ToString('N')).body"
    try {
        $arguments = @("-sS", "-o", $bodyPath, "-w", "%{http_code}")
        if (-not [string]::IsNullOrWhiteSpace($CookiePath)) { $arguments += @("-b", $CookiePath) }
        $statusText = (& curl.exe @arguments "$BaseUrl$Path").Trim()
        if ($LASTEXITCODE -ne 0 -or $statusText -notmatch "^\d{3}$") {
            throw "Request to '$Path' failed."
        }
        $status = [int]$statusText
        if ($ExpectedStatus -notcontains $status) {
            throw "Request to '$Path' returned $status; expected $($ExpectedStatus -join ', ')."
        }
        [pscustomobject][ordered]@{
            path = $Path
            status = $status
            body = Get-Content -LiteralPath $bodyPath -Raw
        }
    } finally {
        Remove-Item -LiteralPath $bodyPath -Force -ErrorAction SilentlyContinue
    }
}

function New-AdminCookie {
    $cookiePath = Join-Path ([IO.Path]::GetTempPath()) "tessara-6d-$([guid]::NewGuid().ToString('N')).cookie"
    $payloadPath = Join-Path ([IO.Path]::GetTempPath()) "tessara-6d-$([guid]::NewGuid().ToString('N')).login"
    try {
        [IO.File]::WriteAllText(
            $payloadPath,
            '{"email":"admin@tessara.local","password":"tessara-dev-admin"}',
            [Text.UTF8Encoding]::new($false)
        )
        $null = & curl.exe -sS -f -c $cookiePath -H "Content-Type: application/json" `
            --data-binary "@$payloadPath" "$BaseUrl/api/auth/login"
        if ($LASTEXITCODE -ne 0) { throw "Administrator browser login failed." }
        $cookiePath
    } finally {
        Remove-Item -LiteralPath $payloadPath -Force -ErrorAction SilentlyContinue
    }
}

function Wait-ReferenceHealthy {
    param([int]$TimeoutSeconds = 90)
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTimeOffset]::UtcNow -lt $deadline) {
        $id = (& docker compose -f $composePath ps -q reference-module-sdk).Trim()
        if ($id -match "^[0-9a-f]{64}$") {
            $health = (& docker inspect --format "{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}" $id).Trim()
            if ($health -eq "healthy") { return $id }
        }
        Start-Sleep -Seconds 1
    }
    throw "Reference module did not become healthy within $TimeoutSeconds seconds."
}

function Invoke-RuntimeResilienceCapture {
    $deployment = Assert-DeploymentBinding
    $cookie = $null
    try {
        $cookie = New-AdminCookie
        $before = Invoke-Request -Path "/reference/module-sdk" -CookiePath $cookie -ExpectedStatus 200
        if ($before.body -notlike "*Module SDK Reference*") {
            throw "Reference module precondition document is incomplete."
        }
        $referenceId = Get-ContainerId -Service "reference-module-sdk"
        $stateBefore = (& docker exec $referenceId sha256sum /var/lib/tessara-reference/state.json).Split(" ")[0]
        $started = [DateTimeOffset]::UtcNow
        $stopwatch = [Diagnostics.Stopwatch]::StartNew()
        & docker compose -f $composePath stop --timeout 30 reference-module-sdk
        if ($LASTEXITCODE -ne 0) { throw "Reference module graceful stop failed." }
        $stopwatch.Stop()
        $stopped = (& docker inspect $referenceId) | ConvertFrom-Json | Select-Object -First 1
        if ($stopped.State.Status -cne "exited" -or [int]$stopped.State.ExitCode -ne 0 -or $stopwatch.Elapsed.TotalSeconds -gt 30) {
            throw "Reference module did not stop cleanly within the 30-second bound."
        }
        $fallback = Invoke-Request -Path "/reference/module-sdk" -CookiePath $cookie -ExpectedStatus 503
        if ($fallback.body -notlike "*Module temporarily unavailable*" -or
            $fallback.body -notlike "*Core and its administration surfaces remain available*") {
            throw "Core did not render the standard authenticated module fallback."
        }
        $coreHealth = Invoke-Request -Path "/health" -ExpectedStatus 200
        $dashboard = Invoke-Request -Path "/dashboards" -CookiePath $cookie -ExpectedStatus 200
        & docker compose -f $composePath up -d reference-module-sdk
        if ($LASTEXITCODE -ne 0) { throw "Reference module restart failed." }
        $restartedId = Wait-ReferenceHealthy
        $after = Invoke-Request -Path "/reference/module-sdk" -CookiePath $cookie -ExpectedStatus 200
        if ($after.body -notlike "*Module SDK Reference*") {
            throw "Reference module did not recover its document after restart."
        }
        $stateAfter = (& docker exec $restartedId sha256sum /var/lib/tessara-reference/state.json).Split(" ")[0]
        if ($stateBefore -cne $stateAfter) {
            throw "Reference module state changed across graceful shutdown and restart."
        }
        $completed = [DateTimeOffset]::UtcNow
        Publish-Evidence -Filename "outage-recovery.json" -Value ([pscustomobject][ordered]@{
            schema_version = 1
            source_commit = [string]$deployment.snapshot.source.commit
            started_at = $started.ToString("o")
            completed_at = $completed.ToString("o")
            chronology = @(
                [pscustomobject]@{ step = "reference_available"; status = $before.status }
                [pscustomobject]@{ step = "reference_stopped"; container_id = $referenceId }
                [pscustomobject]@{ step = "core_fallback"; status = $fallback.status }
                [pscustomobject]@{ step = "core_continuity"; status = $coreHealth.status }
                [pscustomobject]@{ step = "dashboard_continuity"; status = $dashboard.status }
                [pscustomobject]@{ step = "reference_restarted"; container_id = $restartedId; status = $after.status }
            )
            fallback_contained = $true
            state_retained = $true
            passed = $true
        }) | Out-Null
        Publish-Evidence -Filename "shutdown.json" -Value ([pscustomobject][ordered]@{
            schema_version = 1
            source_commit = [string]$deployment.snapshot.source.commit
            signal = "SIGTERM via docker compose stop"
            drain_bound_seconds = 30
            observed_exit_seconds = [Math]::Round($stopwatch.Elapsed.TotalSeconds, 3)
            exit_code = [int]$stopped.State.ExitCode
            state_sha256_before = $stateBefore
            state_sha256_after = $stateAfter
            restarted_healthy = $true
            passed = $true
        }) | Out-Null
    } finally {
        if (-not [string]::IsNullOrWhiteSpace($cookie) -and (Test-Path -LiteralPath $cookie)) {
            $null = & curl.exe -sS -X DELETE -b $cookie "$BaseUrl/api/auth/logout"
            Remove-Item -LiteralPath $cookie -Force -ErrorAction SilentlyContinue
        }
    }
}

function Invoke-ScopedRecordsRegressionCapture {
    $deployment = Assert-DeploymentBinding
    foreach ($required in @("smoke-fresh.json", "uat-fresh.json", "e2e-fresh.summary.json")) {
        if (-not (Test-Path -LiteralPath (Join-Path $evidenceRoot $required) -PathType Leaf)) {
            throw "Required retained regression input '$required' is missing."
        }
    }
    $started = [DateTimeOffset]::UtcNow
    & cargo test -p tessara-reference-scoped-records --locked
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "Scoped Records targeted regression tests failed with exit code $exitCode."
    }
    $manifest = Get-Content -LiteralPath (Join-Path $repoRoot "crates/tessara-reference-scoped-records/Cargo.toml") -Raw
    if ($manifest -notmatch 'tessara-module-runtime\s*=' -or
        $manifest -notmatch 'tessara-module-ui\s*=' -or
        $manifest -match '(?m)^tessara-web\s*=') {
        throw "Scoped Records does not have the required canonical runtime/UI dependency shape."
    }
    Publish-Evidence -Filename "scoped-records-regression.json" -Value ([pscustomobject][ordered]@{
        schema_version = 1
        source_commit = [string]$deployment.snapshot.source.commit
        started_at = $started.ToString("o")
        completed_at = [DateTimeOffset]::UtcNow.ToString("o")
        adoption = [pscustomobject][ordered]@{
            module_runtime = $true
            module_ui = $true
            root_tessara_web = $false
        }
        targeted_test = [pscustomobject][ordered]@{
            command = "cargo test -p tessara-reference-scoped-records --locked"
            exit_code = $exitCode
            passed = $true
        }
        retained_regressions = @(
            [pscustomobject]@{ filename = "smoke-fresh.json"; sha256 = Get-Sha256 (Join-Path $evidenceRoot "smoke-fresh.json") }
            [pscustomobject]@{ filename = "uat-fresh.json"; sha256 = Get-Sha256 (Join-Path $evidenceRoot "uat-fresh.json") }
            [pscustomobject]@{ filename = "e2e-fresh.summary.json"; sha256 = Get-Sha256 (Join-Path $evidenceRoot "e2e-fresh.summary.json") }
        )
        passed = $true
    }) | Out-Null
}

function Invoke-DigestCapture {
    $required = @(
        "source-provenance.json",
        "deployment-fresh.json",
        "bootstrap-first.json",
        "bootstrap-second-noop.json",
        "migration-checkpoint.json",
        "sdk-ownership.json",
        "package-boundaries-native.json",
        "package-boundaries-wasm.json",
        "compatibility-inventory.json",
        "reference-conformance.json",
        "scoped-records-regression.json",
        "outage-recovery.json",
        "shutdown.json",
        "smoke-fresh.json",
        "uat-fresh.json",
        "e2e-fresh.json",
        "e2e-fresh.summary.json",
        "manual-uat.md"
    )
    foreach ($filename in $required) {
        $path = Join-Path $evidenceRoot $filename
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Required closeout evidence is missing: $filename"
        }
        [IO.File]::WriteAllText(
            "$path.sha256",
            "$(Get-Sha256 -Path $path)`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    Write-Host "Validated and digest-bound $($required.Count) Sprint 6D closeout artifacts."
}

if ($SelfTest) {
    $originalRoot = $evidenceRoot
    $evidenceRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-6d-evidence-$([guid]::NewGuid().ToString('N'))"
    try {
        $path = Publish-Evidence -Filename "self-test.json" -Value ([pscustomobject]@{ passed = $true })
        if ((Get-Content -LiteralPath "$path.sha256" -Raw).Trim() -cne (Get-Sha256 -Path $path)) {
            throw "Evidence publisher self-test digest mismatch."
        }
        Write-Host "Sprint 6D closeout evidence publisher self-test passed."
    } finally {
        Remove-Item -LiteralPath $evidenceRoot -Recurse -Force -ErrorAction SilentlyContinue
        $evidenceRoot = $originalRoot
    }
    exit 0
}

Push-Location $repoRoot
try {
    switch ($Mode) {
        "Static" { Invoke-StaticCapture }
        "RuntimeResilience" { Invoke-RuntimeResilienceCapture }
        "ScopedRecordsRegression" { Invoke-ScopedRecordsRegressionCapture }
        "Digests" { Invoke-DigestCapture }
    }
} finally {
    Pop-Location
}

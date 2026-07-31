[CmdletBinding()]
param(
    [string]$EvidencePath,
    [string]$NativeEvidencePath,
    [string]$WasmEvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$findings = [Collections.Generic.List[object]]::new()

function Add-Finding {
    param(
        [Parameter(Mandatory)][string]$Code,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Message
    )
    $findings.Add([pscustomobject][ordered]@{
        code = $Code
        path = $Path
        message = $Message
    })
}

function Get-CargoTree {
    param(
        [Parameter(Mandatory)][string]$Package,
        [Parameter(Mandatory)][string]$Target,
        [string[]]$Arguments = @()
    )
    $output = & cargo tree -p $Package --target $Target --edges normal @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "cargo tree failed for $Package on $Target"
    }
    $output -join "`n"
}

Push-Location $repoRoot
try {
    $nativeTarget = if ($IsWindows) { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" }
    $canonical = @(
        "tessara-module-contract",
        "tessara-module-runtime",
        "tessara-module-ui",
        "tessara-module-testkit"
    )
    $forbiddenPackages = @(
        "tessara-api",
        "tessara-core",
        "tessara-web ",
        "tessara-web-ui",
        "tessara-dashboard-module",
        "tessara-reference-scoped-records",
        "sqlx "
    )
    $nativeTrees = [ordered]@{}
    foreach ($package in $canonical) {
        $tree = Get-CargoTree -Package $package -Target $nativeTarget
        $nativeTrees[$package] = $tree
        foreach ($forbidden in $forbiddenPackages) {
            if ($tree -match "(?m)^[^\r\n]*\b$([regex]::Escape($forbidden.Trim())) v") {
                Add-Finding -Code "forbidden_native_dependency" -Path $package `
                    -Message "$package reaches $($forbidden.Trim()) on $nativeTarget"
            }
        }
    }

    $wasmTrees = [ordered]@{
        "tessara-module-contract" = Get-CargoTree -Package "tessara-module-contract" `
            -Target "wasm32-unknown-unknown"
        "tessara-module-ui" = Get-CargoTree -Package "tessara-module-ui" `
            -Target "wasm32-unknown-unknown" -Arguments @("--no-default-features", "--features", "hydrate")
        "tessara-reference-module-sdk" = Get-CargoTree -Package "tessara-reference-module-sdk" `
            -Target "wasm32-unknown-unknown" -Arguments @("--no-default-features", "--features", "hydrate")
    }
    foreach ($entry in $wasmTrees.GetEnumerator()) {
        foreach ($forbidden in @("tessara-module-runtime", "tokio", "axum", "sqlx", "tessara-api", "tessara-web ")) {
            if ($entry.Value -match "(?m)^[^\r\n]*\b$([regex]::Escape($forbidden.Trim())) v") {
                Add-Finding -Code "forbidden_wasm_dependency" -Path $entry.Key `
                    -Message "$($entry.Key) reaches $($forbidden.Trim()) on wasm32-unknown-unknown"
            }
        }
    }

    $canonicalSourceRoots = @(
        "crates/tessara-module-contract/src",
        "crates/tessara-module-runtime/src",
        "crates/tessara-module-ui/src",
        "crates/tessara-module-testkit/src"
    )
    $sourcePattern = "tessara_(api|core|web|dashboard|dashboards|reference_scoped_records)|tessara\.(dashboards|reference\.scoped-records)|sqlx::"
    foreach ($root in $canonicalSourceRoots) {
        foreach ($sourceFile in Get-ChildItem -LiteralPath $root -Recurse -File -Filter "*.rs") {
            $productionSource = (Get-Content -LiteralPath $sourceFile.FullName -Raw) `
                -split '#\[cfg\(test\)\]', 2 |
                Select-Object -First 1
            $lineNumber = 0
            foreach ($line in $productionSource -split "`r?`n") {
                $lineNumber++
                if ($line -cmatch $sourcePattern) {
                    $relative = [IO.Path]::GetRelativePath($repoRoot, $sourceFile.FullName)
                    Add-Finding -Code "forbidden_canonical_source" -Path $relative `
                        -Message "$relative`:$lineNumber`:$line"
                }
            }
        }
    }

    $dashboardManifest = Get-Content -LiteralPath `
        (Join-Path $repoRoot "crates/tessara-dashboard-module/Cargo.toml") -Raw
    $dashboardTransition = $dashboardManifest -match 'tessara-web\s*='
    if ($dashboardTransition) {
        throw "Dashboard still depends on root tessara-web after the Sprint 6E lifecycle migration."
    }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        native_target = $nativeTarget
        wasm_target = "wasm32-unknown-unknown"
        canonical_packages = $canonical
        dashboard_transition = [pscustomobject][ordered]@{
            code = "dashboard_root_web_dependency"
            owner = "Sprint 6E"
            allowlisted = $false
            observed = $dashboardTransition
            resolved = -not $dashboardTransition
        }
        findings = @($findings)
        passed = $findings.Count -eq 0
    }

    $nativeResult = [pscustomobject][ordered]@{
        schema_version = 1
        checked_at = $result.checked_at
        target = $nativeTarget
        canonical_packages = $canonical
        dependency_trees = $nativeTrees
        dashboard_transition = $result.dashboard_transition
        findings = @($findings | Where-Object { $_.code -notlike "*wasm*" })
        passed = @($findings | Where-Object { $_.code -notlike "*wasm*" }).Count -eq 0
    }
    $wasmResult = [pscustomobject][ordered]@{
        schema_version = 1
        checked_at = $result.checked_at
        target = "wasm32-unknown-unknown"
        dependency_trees = $wasmTrees
        findings = @($findings | Where-Object { $_.code -like "*wasm*" })
        passed = @($findings | Where-Object { $_.code -like "*wasm*" }).Count -eq 0
    }

    function Write-Evidence {
        param([string]$Path, [object]$Value)
        if ([string]::IsNullOrWhiteSpace($Path)) { return }
        $fullPath = if ([IO.Path]::IsPathRooted($Path)) {
            [IO.Path]::GetFullPath($Path)
        } else {
            [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
        }
        $parent = Split-Path -Parent $fullPath
        [IO.Directory]::CreateDirectory($parent) | Out-Null
        [IO.File]::WriteAllText(
            $fullPath,
            ($Value | ConvertTo-Json -Depth 20) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    Write-Evidence -Path $EvidencePath -Value $result
    Write-Evidence -Path $NativeEvidencePath -Value $nativeResult
    Write-Evidence -Path $WasmEvidencePath -Value $wasmResult
    $result | ConvertTo-Json -Depth 20
    if (-not $result.passed) {
        exit 1
    }
} finally {
    Pop-Location
}

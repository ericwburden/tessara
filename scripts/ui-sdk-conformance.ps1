[CmdletBinding()]
param([string]$EvidencePath)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$findings = [Collections.Generic.List[object]]::new()

function Add-Finding([string]$Code, [string]$Path, [string]$Message) {
    $findings.Add([pscustomobject][ordered]@{ code = $Code; path = $Path; message = $Message })
}

Push-Location $repoRoot
try {
    $sdkDigest = (Get-FileHash -LiteralPath "crates/tessara-module-ui/assets/module-ui.css" -Algorithm SHA256).Hash.ToLowerInvariant()
    $manifests = @(Get-ChildItem -LiteralPath "crates" -Recurse -File -Filter "manifest.json" | Sort-Object FullName)
    if ($manifests.Count -eq 0) { throw "No first-party manifests were found" }
    $expectedTuple = [ordered]@{
        shell_context_schema = "2.0.0";
        module_contract = "0.3.0"; module_runtime = "0.3.0"; module_ui = "0.3.0";
        design_system_asset_abi = "2.0.0"; conformance_suite = "1.2.0"
    }
    foreach ($file in $manifests) {
        $path = [IO.Path]::GetRelativePath($repoRoot, $file.FullName)
        $manifest = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
        foreach ($entry in $expectedTuple.GetEnumerator()) {
            if ([string]$manifest.platform_versions.($entry.Key) -cne $entry.Value) {
                Add-Finding "ui_sdk_tuple_mismatch" $path "platform_versions.$($entry.Key) must be $($entry.Value)"
            }
        }
        $sdkAsset = @($manifest.assets | Where-Object { [string]$_.path -match "module-ui\.css$" })
        if ($sdkAsset.Count -ne 1 -or [string]$sdkAsset[0].digest -cne "sha256:$sdkDigest") {
            Add-Finding "canonical_sdk_asset_missing" $path "manifest must declare exactly one canonical module-ui.css digest"
        }
    }

    try {
        & "scripts/build-module-ui-browser-assets.ps1" -Module all -DeclarationsOnly | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Add-Finding "module_asset_identity_mismatch" "scripts/build-module-ui-browser-assets.ps1" "module asset declarations are not source exact"
        }
    } catch {
        Add-Finding "module_asset_identity_mismatch" "scripts/build-module-ui-browser-assets.ps1" $_.Exception.Message
    }

    $cssOwnershipJson = & node "scripts/check-product-css-ownership.mjs" 2>&1
    $cssOwnershipExit = $LASTEXITCODE
    $cssOwnership = $cssOwnershipJson | ConvertFrom-Json
    foreach ($finding in @($cssOwnership.findings)) {
        Add-Finding ([string]$finding.code) ([string]$finding.path) ([string]$finding.message)
    }
    if ($cssOwnershipExit -notin @(0, 1)) { throw "product CSS ownership runner failed: $cssOwnershipJson" }

    $productSources = @(
        "crates/tessara-component-module", "crates/tessara-component-ui",
        "crates/tessara-dashboard-module", "crates/tessara-dashboard-ui",
        "crates/tessara-dataset-module", "crates/tessara-web-datasets",
        "crates/tessara-reference-scoped-records", "crates/tessara-reference-module-sdk"
    )
    foreach ($sourceRoot in $productSources) {
        foreach ($source in Get-ChildItem -LiteralPath $sourceRoot -Recurse -File -Filter "*.rs") {
            $path = [IO.Path]::GetRelativePath($repoRoot, $source.FullName)
            $text = Get-Content -LiteralPath $source.FullName -Raw
            if ($text -match "\binner_html\s*=" -or $text -match "\.set_inner_html\(") {
                Add-Finding "raw_module_html" $path "product source must not inject structural HTML"
            }
            if ($text -match 'format!\s*\(\s*r?#+?"\s*<') {
                Add-Finding "raw_module_document" $path "product source must not build structural HTML strings"
            }
        }
    }

    $sdkCss = Get-Content -LiteralPath "crates/tessara-module-ui/assets/module-ui.css" -Raw
    foreach ($selector in @(
        ".app-shell", ".app-main", ".top-app-bar", ".page-header", ".button",
        ".data-table", ".searchable-data-table__search", ".breadcrumb", ".empty-state", ".status-badge",
        ".modal-dialog", ".sheet-panel", ".tabs-list"
    )) {
        if (-not $sdkCss.Contains($selector)) {
            Add-Finding "unstyled_sdk_primitive" "crates/tessara-module-ui/assets/module-ui.css" "missing canonical style for $selector"
        }
    }
    if ($sdkCss -notmatch 'body\.tessara-app[\s\S]*background:\s*var\(--color-bg\)') {
        Add-Finding "canonical_canvas_missing" "crates/tessara-module-ui/assets/module-ui.css" "SDK must own the application canvas background"
    }

    $coreCss = Get-Content -LiteralPath "style/core.css" -Raw
    if ($coreCss -match '(?m)^[^{]*\.dataset[-_A-Za-z0-9]*') {
        Add-Finding "dataset_css_owned_by_core" "style/core.css" "Dataset product selectors must be owned by the Dataset module asset"
    }
    $moduleDocumentSource = Get-Content -LiteralPath "crates/tessara-module-ui/src/lib.rs" -Raw
    $sharedSidebarSource = Get-Content -LiteralPath "crates/tessara-module-ui/src/shell_sidebar.rs" -Raw
    $coreNavigationSource = Get-Content -LiteralPath "crates/tessara-web/src/ui/shell/nav.rs" -Raw
    foreach ($required in @("ApplicationShell", "ShellSidebar")) {
        if (-not $moduleDocumentSource.Contains($required)) {
            Add-Finding "module_shell_not_shared" "crates/tessara-module-ui/src/lib.rs" "complete module documents must render the shared $required component"
        }
    }
    if (-not $sharedSidebarSource.Contains("ShellNavigationIcon") -or
        -not $coreNavigationSource.Contains("ShellNavigationIcon")) {
        Add-Finding "navigation_icon_mapping_not_shared" "crates/tessara-module-ui/src/shell_sidebar.rs" "Core and complete module documents must consume the canonical navigation icon component"
    }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = "tessara.ui-sdk-conformance"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        sdk_css_sha256 = $sdkDigest
        manifest_inventory = @($manifests | ForEach-Object { [IO.Path]::GetRelativePath($repoRoot, $_.FullName) })
        findings = @($findings)
        passed = $findings.Count -eq 0
    }
    if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        $fullPath = if ([IO.Path]::IsPathRooted($EvidencePath)) { $EvidencePath } else { Join-Path $repoRoot $EvidencePath }
        [IO.Directory]::CreateDirectory((Split-Path -Parent $fullPath)) | Out-Null
        [IO.File]::WriteAllText($fullPath, ($result | ConvertTo-Json -Depth 10) + "`n", [Text.UTF8Encoding]::new($false))
    }
    $result | ConvertTo-Json -Depth 10
    if (-not $result.passed) { exit 1 }
} finally { Pop-Location }

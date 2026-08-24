[CmdletBinding()]
param(
    [ValidateSet("all", "components", "dashboards", "datasets", "responses")]
    [string]$Module = "all",
    [switch]$Check,
    [switch]$DeclarationsOnly,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$modules = @(
    [pscustomobject]@{
        Name = "components"
        Definition = "tessara.components"
        Release = "1.1.0"
        Package = "tessara-component-ui"
        Wasm = "tessara_component_ui.wasm"
        OutputName = "component-bindings"
        EntryAsset = "crates/tessara-component-ui/assets/component.js"
        BindingsAsset = "crates/tessara-component-ui/assets/component-bindings.js"
        WasmAsset = "crates/tessara-component-ui/assets/component.wasm"
        BindingsName = "component-bindings.js"
        WasmName = "component.wasm"
        DigestContract = "crates/tessara-component-ui/src/document.rs"
        Manifest = "crates/tessara-component-module/manifest.json"
        ReleaseCatalog = "deploy/sprint-8b/catalogs/local-release-catalog.json"
        AssetSpecs = @(
            [pscustomobject]@{ Path = "/component.css"; Constant = "COMPONENT_CSS_SHA256"; Sources = @("crates/tessara-component-ui/assets/component.css") }
            [pscustomobject]@{ Path = "/component-lifecycle.css"; Constant = "COMPONENT_LIFECYCLE_CSS_SHA256"; Sources = @("crates/tessara-component-ui/assets/component.css", "crates/tessara-component-ui/assets/component-lifecycle.css") }
            [pscustomobject]@{ Path = "/component.js"; Constant = "COMPONENT_JS_SHA256"; Sources = @("crates/tessara-component-ui/assets/component.js") }
            [pscustomobject]@{ Path = "/component-bindings.js"; Constant = "COMPONENT_BINDINGS_JS_SHA256"; Sources = @("crates/tessara-component-ui/assets/component-bindings.js") }
            [pscustomobject]@{ Path = "/component.wasm"; Constant = "COMPONENT_WASM_SHA256"; Sources = @("crates/tessara-component-ui/assets/component.wasm") }
        )
    },
    [pscustomobject]@{
        Name = "dashboards"
        Definition = "tessara.dashboards"
        Release = "3.0.2"
        Package = "tessara-dashboard-ui"
        Wasm = "tessara_dashboard_ui.wasm"
        OutputName = "dashboard-bindings"
        EntryAsset = "crates/tessara-dashboard-ui/assets/dashboard.js"
        BindingsAsset = "crates/tessara-dashboard-ui/assets/dashboard-bindings.js"
        WasmAsset = "crates/tessara-dashboard-ui/assets/dashboard.wasm"
        BindingsName = "dashboard-bindings.js"
        WasmName = "dashboard.wasm"
        DigestContract = "crates/tessara-dashboard-ui/src/document.rs"
        Manifest = "crates/tessara-dashboard-module/manifest.json"
        ReleaseCatalog = "deploy/sprint-8b/catalogs/local-release-catalog.json"
        AssetSpecs = @(
            [pscustomobject]@{ Path = "/dashboard.css"; Constant = "DASHBOARD_CSS_SHA256"; Sources = @("crates/tessara-dashboard-ui/assets/dashboard.css") }
            [pscustomobject]@{ Path = "/dashboard-lifecycle.css"; Constant = "DASHBOARD_LIFECYCLE_CSS_SHA256"; Sources = @("crates/tessara-dashboard-ui/assets/dashboard.css", "crates/tessara-dashboard-ui/assets/dashboard-lifecycle.css") }
            [pscustomobject]@{ Path = "/dashboard.js"; Constant = "DASHBOARD_JS_SHA256"; Sources = @("crates/tessara-dashboard-ui/assets/dashboard.js") }
            [pscustomobject]@{ Path = "/dashboard-bindings.js"; Constant = "DASHBOARD_BINDINGS_JS_SHA256"; Sources = @("crates/tessara-dashboard-ui/assets/dashboard-bindings.js") }
            [pscustomobject]@{ Path = "/dashboard.wasm"; Constant = "DASHBOARD_WASM_SHA256"; Sources = @("crates/tessara-dashboard-ui/assets/dashboard.wasm") }
        )
    },
    [pscustomobject]@{
        Name = "datasets"
        Definition = "tessara.datasets"
        Release = "1.0.0"
        Package = "tessara-dataset-ui"
        Wasm = "tessara_dataset_ui.wasm"
        OutputName = "dataset-bindings"
        EntryAsset = "crates/tessara-web-datasets/assets/dataset.js"
        BindingsAsset = "crates/tessara-web-datasets/assets/dataset-bindings.js"
        WasmAsset = "crates/tessara-web-datasets/assets/dataset.wasm"
        BindingsName = "dataset-bindings.js"
        WasmName = "dataset.wasm"
        DigestContract = "crates/tessara-web-datasets/src/document.rs"
        Manifest = "crates/tessara-dataset-module/manifest.json"
        ReleaseCatalog = "deploy/sprint-8b/catalogs/local-release-catalog.json"
        AssetSpecs = @(
            [pscustomobject]@{ Path = "/dataset.css"; Constant = "DATASET_CSS_SHA256"; Sources = @("crates/tessara-web-datasets/assets/dataset.css") }
            [pscustomobject]@{ Path = "/dataset-lifecycle.css"; Constant = "DATASET_LIFECYCLE_CSS_SHA256"; Sources = @("crates/tessara-web-datasets/assets/dataset.css", "crates/tessara-web-datasets/assets/dataset-lifecycle.css") }
            [pscustomobject]@{ Path = "/dataset.js"; Constant = "DATASET_JS_SHA256"; Sources = @("crates/tessara-web-datasets/assets/dataset.js") }
            [pscustomobject]@{ Path = "/dataset-bindings.js"; Constant = "DATASET_BINDINGS_JS_SHA256"; Sources = @("crates/tessara-web-datasets/assets/dataset-bindings.js") }
            [pscustomobject]@{ Path = "/dataset.wasm"; Constant = "DATASET_WASM_SHA256"; Sources = @("crates/tessara-web-datasets/assets/dataset.wasm") }
        )
    },
    [pscustomobject]@{
        Name = "responses"
        Definition = "tessara.responses"
        Release = "1.0.0"
        Package = "tessara-response-ui"
        Wasm = "tessara_response_ui.wasm"
        OutputName = "response-bindings"
        EntryAsset = "crates/tessara-web-responses/assets/response.js"
        BindingsAsset = "crates/tessara-web-responses/assets/response-bindings.js"
        WasmAsset = "crates/tessara-web-responses/assets/response.wasm"
        BindingsName = "response-bindings.js"
        WasmName = "response.wasm"
        DigestContract = "crates/tessara-web-responses/src/document.rs"
        Manifest = "crates/tessara-response-module/manifest.json"
        ReleaseCatalog = "deploy/sprint-8c/catalogs/local-release-catalog.json"
        AssetSpecs = @(
            [pscustomobject]@{ Path = "/response.css"; Constant = "RESPONSE_CSS_SHA256"; Sources = @("crates/tessara-web-responses/assets/response.css") }
            [pscustomobject]@{ Path = "/response.js"; Constant = "RESPONSE_JS_SHA256"; Sources = @("crates/tessara-web-responses/assets/response.js") }
            [pscustomobject]@{ Path = "/response-bindings.js"; Constant = "RESPONSE_BINDINGS_JS_SHA256"; Sources = @("crates/tessara-web-responses/assets/response-bindings.js") }
            [pscustomobject]@{ Path = "/response.wasm"; Constant = "RESPONSE_WASM_SHA256"; Sources = @("crates/tessara-web-responses/assets/response.wasm") }
        )
    }
)
if ($Module -cne "all") {
    $modules = @($modules | Where-Object Name -CEQ $Module)
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-CompositeSha256([string[]]$Sources) {
    $stream = [IO.MemoryStream]::new()
    try {
        for ($index = 0; $index -lt $Sources.Count; $index++) {
            $sourcePath = Join-Path $repoRoot $Sources[$index]
            $bytes = [IO.File]::ReadAllBytes($sourcePath)
            $stream.Write($bytes, 0, $bytes.Length)
            if ($index -lt ($Sources.Count - 1)) {
                $stream.WriteByte(10)
            }
        }
        -join ([Security.Cryptography.SHA256]::HashData($stream.ToArray()) | ForEach-Object { $_.ToString("x2") })
    } finally {
        $stream.Dispose()
    }
}

function Sync-SingleDigest(
    [string]$Path,
    [string]$Pattern,
    [string]$ExpectedDigest,
    [string]$Description,
    [switch]$CheckOnly
) {
    $text = [IO.File]::ReadAllText($Path)
    $matches = [regex]::Matches($text, $Pattern)
    if ($matches.Count -ne 1) {
        throw "$Description must declare exactly one digest binding."
    }
    $digestGroup = $matches[0].Groups["digest"]
    if ($digestGroup.Value -ceq $ExpectedDigest) {
        return
    }
    if ($CheckOnly) {
        throw "$Description digest is stale: declared $($digestGroup.Value), source-exact $ExpectedDigest."
    }
    $updated = $text.Remove($digestGroup.Index, $digestGroup.Length).Insert($digestGroup.Index, $ExpectedDigest)
    [IO.File]::WriteAllText($Path, $updated, [Text.UTF8Encoding]::new($false))
}

function Sync-ModuleAssetDigests([pscustomobject]$ModuleDefinition, [switch]$CheckOnly) {
    $contractPath = Join-Path $repoRoot $ModuleDefinition.DigestContract
    $manifestPath = Join-Path $repoRoot $ModuleDefinition.Manifest
    $digests = [ordered]@{}
    foreach ($asset in $ModuleDefinition.AssetSpecs) {
        $digest = Get-CompositeSha256 @($asset.Sources)
        $digests[$asset.Path] = $digest
        $constantPattern = "(?s)pub const $([regex]::Escape($asset.Constant)): &str =\s*`"(?<digest>[0-9a-f]{64})`";"
        $manifestPattern = "(?s)`"path`"\s*:\s*`"$([regex]::Escape($asset.Path))`"\s*,\s*`"digest`"\s*:\s*`"sha256:(?<digest>[0-9a-f]{64})`""
        Sync-SingleDigest $contractPath $constantPattern $digest "$($ModuleDefinition.Name) Rust constant $($asset.Constant)" -CheckOnly:$CheckOnly
        Sync-SingleDigest $manifestPath $manifestPattern $digest "$($ModuleDefinition.Name) manifest asset $($asset.Path)" -CheckOnly:$CheckOnly
    }
    $digests
}

function Sync-ReleaseCatalogManifestDigest([pscustomobject]$ModuleDefinition, [switch]$CheckOnly) {
    $manifestPath = Join-Path $repoRoot $ModuleDefinition.Manifest
    $digestOutput = @(& cargo run --manifest-path (Join-Path $repoRoot "Cargo.toml") --locked --offline -q -p tessara-supervisor --bin tessara-compose -- manifest-digest $manifestPath)
    if ($LASTEXITCODE -ne 0) {
        throw "$($ModuleDefinition.Name) manifest canonical digest could not be computed."
    }
    $manifestDigest = [string]($digestOutput | Select-Object -Last 1)
    if ($manifestDigest -cnotmatch '^sha256:(?<value>[0-9a-f]{64})$') {
        throw "$($ModuleDefinition.Name) manifest canonical digest output is invalid: $manifestDigest"
    }
    $catalogPath = Join-Path $repoRoot $ModuleDefinition.ReleaseCatalog
    $catalogPattern = "(?s)`"definition_id`"\s*:\s*`"$([regex]::Escape($ModuleDefinition.Definition))`"\s*,\s*`"version`"\s*:\s*`"$([regex]::Escape($ModuleDefinition.Release))`"\s*,\s*`"manifest_digest`"\s*:\s*`"sha256:(?<digest>[0-9a-f]{64})`""
    Sync-SingleDigest $catalogPath $catalogPattern $manifestDigest.Substring(7) "$($ModuleDefinition.Name) release-catalog manifest identity" -CheckOnly:$CheckOnly
    $manifestDigest
}

function Assert-SourceExactEntryAssetReferences(
    [pscustomobject]$ModuleDefinition,
    [string]$EntryText,
    [string]$BindingsHash,
    [string]$WasmHash
) {
    foreach ($expectedReference in @(
        "/_tessara/modules/$($ModuleDefinition.Definition)/$($ModuleDefinition.Release)/sha256:$BindingsHash/$($ModuleDefinition.BindingsName)",
        "/_tessara/modules/$($ModuleDefinition.Definition)/$($ModuleDefinition.Release)/sha256:$WasmHash/$($ModuleDefinition.WasmName)"
    )) {
        if (-not $EntryText.Contains($expectedReference)) {
            throw "$($ModuleDefinition.Name) entry asset does not reference source-exact generated asset '$expectedReference'."
        }
    }
}

if ($SelfTest) {
    $fixture = $modules[0]
    $bindingsHash = "a" * 64
    $wasmHash = "b" * 64
    $validEntry = @"
import init from "/_tessara/modules/$($fixture.Definition)/$($fixture.Release)/sha256:$bindingsHash/$($fixture.BindingsName)";
await init("/_tessara/modules/$($fixture.Definition)/$($fixture.Release)/sha256:$wasmHash/$($fixture.WasmName)");
"@
    Assert-SourceExactEntryAssetReferences $fixture $validEntry $bindingsHash $wasmHash
    $rejectedMismatch = $false
    try {
        Assert-SourceExactEntryAssetReferences $fixture ($validEntry.Replace($bindingsHash, ("c" * 64))) $bindingsHash $wasmHash
    } catch {
        $rejectedMismatch = $_.Exception.Message.Contains("entry asset does not reference source-exact generated asset")
    }
    if (-not $rejectedMismatch) {
        throw "Entry asset source-exact reference self-test did not reject a stale bindings digest."
    }
    $declarationFixture = 'pub const SAMPLE_SHA256: &str = "' + ("c" * 64) + '";'
    $declarationFixturePath = Join-Path ([IO.Path]::GetTempPath()) "tessara-module-ui-digest-$([guid]::NewGuid().ToString('N')).rs"
    try {
        [IO.File]::WriteAllText($declarationFixturePath, $declarationFixture, [Text.UTF8Encoding]::new($false))
        $staleDeclarationRejected = $false
        try {
            Sync-SingleDigest $declarationFixturePath 'pub const SAMPLE_SHA256: &str = "(?<digest>[0-9a-f]{64})";' $bindingsHash "self-test digest" -CheckOnly
        } catch {
            $staleDeclarationRejected = $_.Exception.Message.Contains("digest is stale")
        }
        if (-not $staleDeclarationRejected) {
            throw "Declared asset digest self-test did not reject a stale source binding."
        }
        Sync-SingleDigest $declarationFixturePath 'pub const SAMPLE_SHA256: &str = "(?<digest>[0-9a-f]{64})";' $bindingsHash "self-test digest"
        if (-not ([IO.File]::ReadAllText($declarationFixturePath).Contains($bindingsHash))) {
            throw "Declared asset digest self-test did not update the exact digest group."
        }
    } finally {
        Remove-Item -LiteralPath $declarationFixturePath -Force -ErrorAction SilentlyContinue
    }
    [pscustomobject][ordered]@{
        contract = "tessara.module-ui.entry-asset-source-exact-self-test"
        state = "passed"
        stale_bindings_digest_rejected = $rejectedMismatch
        stale_declared_digest_rejected = $staleDeclarationRejected
    } | ConvertTo-Json -Compress
    exit 0
}

if ($DeclarationsOnly) {
    Push-Location $repoRoot
    try {
        foreach ($item in $modules) {
            $bindingsHash = Get-Sha256 (Join-Path $repoRoot $item.BindingsAsset)
            $wasmHash = Get-Sha256 (Join-Path $repoRoot $item.WasmAsset)
            $entryText = Get-Content -LiteralPath (Join-Path $repoRoot $item.EntryAsset) -Raw
            Assert-SourceExactEntryAssetReferences $item $entryText $bindingsHash $wasmHash
            $assetDigests = Sync-ModuleAssetDigests $item -CheckOnly
            $manifestDigest = Sync-ReleaseCatalogManifestDigest $item -CheckOnly
            [pscustomobject][ordered]@{
                module = $item.Name
                asset_digests = $assetDigests
                manifest_digest = $manifestDigest
                state = "declared-source-exact"
            } | ConvertTo-Json -Compress
        }
    } finally {
        Pop-Location
    }
    exit 0
}

$wasmBindgen = Get-Command "wasm-bindgen" -ErrorAction Stop

Push-Location $repoRoot
try {
    foreach ($item in $modules) {
        & cargo build -p $item.Package --target wasm32-unknown-unknown --release --features hydrate --locked --offline
        if ($LASTEXITCODE -ne 0) {
            throw "The $($item.Name) browser WASM build failed."
        }

        $outputDirectory = Join-Path $repoRoot "target/module-ui-assets/$($item.Name)"
        [IO.Directory]::CreateDirectory($outputDirectory) | Out-Null
        & $wasmBindgen.Source `
            (Join-Path $repoRoot "target/wasm32-unknown-unknown/release/$($item.Wasm)") `
            --target web `
            --out-dir $outputDirectory `
            --out-name $item.OutputName
        if ($LASTEXITCODE -ne 0) {
            throw "wasm-bindgen failed for $($item.Name)."
        }

        $generatedBindings = Join-Path $outputDirectory "$($item.OutputName).js"
        $generatedWasm = Join-Path $outputDirectory "$($item.OutputName)_bg.wasm"
        $bindingsAsset = Join-Path $repoRoot $item.BindingsAsset
        $wasmAsset = Join-Path $repoRoot $item.WasmAsset
        $entryAsset = Join-Path $repoRoot $item.EntryAsset
        $generatedBindingsHash = Get-Sha256 $generatedBindings
        $generatedWasmHash = Get-Sha256 $generatedWasm

        if ($Check) {
            foreach ($comparison in @(
                @($generatedBindings, $bindingsAsset, "bindings"),
                @($generatedWasm, $wasmAsset, "WASM")
            )) {
                $generatedHash = Get-Sha256 $comparison[0]
                $trackedHash = Get-Sha256 $comparison[1]
                if ($generatedHash -cne $trackedHash) {
                    throw "$($item.Name) $($comparison[2]) asset is stale: generated $generatedHash, tracked $trackedHash."
                }
            }
        } else {
            Copy-Item -LiteralPath $generatedBindings -Destination $bindingsAsset -Force
            Copy-Item -LiteralPath $generatedWasm -Destination $wasmAsset -Force
            $entryText = Get-Content -LiteralPath $entryAsset -Raw
            $entryText = [regex]::Replace(
                $entryText,
                "/_tessara/modules/$([regex]::Escape($item.Definition))/$([regex]::Escape($item.Release))/sha256:[0-9a-f]{64}/$([regex]::Escape($item.BindingsName))",
                "/_tessara/modules/$($item.Definition)/$($item.Release)/sha256:$generatedBindingsHash/$($item.BindingsName)"
            )
            $entryText = [regex]::Replace(
                $entryText,
                "/_tessara/modules/$([regex]::Escape($item.Definition))/$([regex]::Escape($item.Release))/sha256:[0-9a-f]{64}/$([regex]::Escape($item.WasmName))",
                "/_tessara/modules/$($item.Definition)/$($item.Release)/sha256:$generatedWasmHash/$($item.WasmName)"
            )
            [IO.File]::WriteAllText($entryAsset, $entryText, [Text.UTF8Encoding]::new($false))
        }

        $entryText = Get-Content -LiteralPath $entryAsset -Raw
        Assert-SourceExactEntryAssetReferences $item $entryText $generatedBindingsHash $generatedWasmHash
        $assetDigests = Sync-ModuleAssetDigests $item -CheckOnly:$Check
        $manifestDigest = Sync-ReleaseCatalogManifestDigest $item -CheckOnly:$Check

        [pscustomobject][ordered]@{
            module = $item.Name
            bindings_sha256 = $generatedBindingsHash
            wasm_sha256 = $generatedWasmHash
            asset_digests = $assetDigests
            manifest_digest = $manifestDigest
            state = if ($Check) { "source-exact" } else { "generated" }
        } | ConvertTo-Json -Compress
    }
} finally {
    Pop-Location
}

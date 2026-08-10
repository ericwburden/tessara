[CmdletBinding()]
param(
    [ValidateSet("all", "components", "dashboards")]
    [string]$Module = "all",
    [switch]$Check,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$modules = @(
    [pscustomobject]@{
        Name = "components"
        Definition = "tessara.components"
        Release = "1.0.1"
        Package = "tessara-component-ui"
        Wasm = "tessara_component_ui.wasm"
        OutputName = "component-bindings"
        EntryAsset = "crates/tessara-component-ui/assets/component.js"
        BindingsAsset = "crates/tessara-component-ui/assets/component-bindings.js"
        WasmAsset = "crates/tessara-component-ui/assets/component.wasm"
        BindingsName = "component-bindings.js"
        WasmName = "component.wasm"
    },
    [pscustomobject]@{
        Name = "dashboards"
        Definition = "tessara.dashboards"
        Release = "3.0.1"
        Package = "tessara-dashboard-ui"
        Wasm = "tessara_dashboard_ui.wasm"
        OutputName = "dashboard-bindings"
        EntryAsset = "crates/tessara-dashboard-ui/assets/dashboard.js"
        BindingsAsset = "crates/tessara-dashboard-ui/assets/dashboard-bindings.js"
        WasmAsset = "crates/tessara-dashboard-ui/assets/dashboard.wasm"
        BindingsName = "dashboard-bindings.js"
        WasmName = "dashboard.wasm"
    }
)
if ($Module -cne "all") {
    $modules = @($modules | Where-Object Name -CEQ $Module)
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
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
    [pscustomobject][ordered]@{
        contract = "tessara.module-ui.entry-asset-source-exact-self-test"
        state = "passed"
        stale_bindings_digest_rejected = $rejectedMismatch
    } | ConvertTo-Json -Compress
    exit 0
}

$wasmBindgen = Get-Command "wasm-bindgen" -ErrorAction Stop

Push-Location $repoRoot
try {
    foreach ($item in $modules) {
        & cargo build -p $item.Package --target wasm32-unknown-unknown --release --features hydrate
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

        [pscustomobject][ordered]@{
            module = $item.Name
            bindings_sha256 = $generatedBindingsHash
            wasm_sha256 = $generatedWasmHash
            state = if ($Check) { "source-exact" } else { "generated" }
        } | ConvertTo-Json -Compress
    }
} finally {
    Pop-Location
}

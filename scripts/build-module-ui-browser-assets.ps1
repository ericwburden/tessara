[CmdletBinding()]
param(
    [ValidateSet("all", "components", "dashboards")]
    [string]$Module = "all",
    [switch]$Check
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$wasmBindgen = Get-Command "wasm-bindgen" -ErrorAction Stop
$modules = @(
    [pscustomobject]@{
        Name = "components"
        Package = "tessara-component-ui"
        Wasm = "tessara_component_ui.wasm"
        OutputName = "component-bindings"
        BindingsAsset = "crates/tessara-component-ui/assets/component-bindings.js"
        WasmAsset = "crates/tessara-component-ui/assets/component.wasm"
    },
    [pscustomobject]@{
        Name = "dashboards"
        Package = "tessara-dashboard-ui"
        Wasm = "tessara_dashboard_ui.wasm"
        OutputName = "dashboard-bindings"
        BindingsAsset = "crates/tessara-dashboard-ui/assets/dashboard-bindings.js"
        WasmAsset = "crates/tessara-dashboard-ui/assets/dashboard.wasm"
    }
)
if ($Module -cne "all") {
    $modules = @($modules | Where-Object Name -CEQ $Module)
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

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
        }

        [pscustomobject][ordered]@{
            module = $item.Name
            bindings_sha256 = Get-Sha256 $generatedBindings
            wasm_sha256 = Get-Sha256 $generatedWasm
            state = if ($Check) { "source-exact" } else { "generated" }
        } | ConvertTo-Json -Compress
    }
} finally {
    Pop-Location
}

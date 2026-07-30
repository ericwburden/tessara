[CmdletBinding()]
param(
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$findings = [Collections.Generic.List[object]]::new()

function Add-Finding {
    param([string]$Code, [string]$Path, [string]$Message)
    $findings.Add([pscustomobject][ordered]@{
        code = $Code
        path = $Path
        message = $Message
    })
}

$supported = [ordered]@{
    core_release = "0.1.0"
    shell_context_schema = "1.0.0"
    module_control_protocol = "1.0.0"
    module_contract = "0.1.0"
    module_runtime = "0.1.0"
    module_ui = "0.1.0"
    design_system_asset_abi = "1.0.0"
    conformance_suite = "1.0.0"
}
$inventory = [Collections.Generic.List[object]]::new()

Push-Location $repoRoot
try {
    $manifestRecords = [Collections.Generic.List[object]]::new()
    foreach ($path in @(
        "crates/tessara-dashboard-module/manifest.json",
        "crates/tessara-reference-module-sdk/manifest.json",
        "crates/tessara-module-contract/tests/fixtures/valid-manifest.json"
    )) {
        $manifestRecords.Add([pscustomobject]@{
            source = $path
            manifest = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
        })
    }
    $deployment = Get-Content -LiteralPath `
        "deploy/sprint-6b1/fixtures/deployment-v1.json" -Raw | ConvertFrom-Json
    foreach ($module in $deployment.modules) {
        $manifestRecords.Add([pscustomobject]@{
            source = "deploy/sprint-6b1/fixtures/deployment-v1.json#$($module.definition_id)"
            manifest = $module.manifest
        })
    }
    foreach ($record in $manifestRecords) {
        $path = $record.source
        $manifest = $record.manifest
        $releaseFindings = [Collections.Generic.List[string]]::new()
        if ($manifest.schema_version -ne 2) {
            $releaseFindings.Add("unsupported manifest schema $($manifest.schema_version)")
        }
        foreach ($field in $supported.Keys) {
            if ([string]$manifest.platform_versions.$field -ne $supported[$field]) {
                $releaseFindings.Add(
                    "platform_versions.$field is $($manifest.platform_versions.$field); expected $($supported[$field])"
                )
            }
        }
        foreach ($field in @("module_contract", "module_runtime", "module_ui")) {
            $linked = $manifest.linked_packages.$field
            if ($null -ne $linked -and [string]$linked -ne $supported[$field]) {
                $releaseFindings.Add(
                    "linked_packages.$field is $linked; expected $($supported[$field])"
                )
            }
        }
        foreach ($message in $releaseFindings) {
            Add-Finding -Code "unsupported_module_sdk_release" -Path $path -Message $message
        }
        $inventory.Add([pscustomobject][ordered]@{
            source = $path
            definition_id = $manifest.definition_id
            release_version = $manifest.release_version
            schema_version = $manifest.schema_version
            platform_versions = $manifest.platform_versions
            linked_packages = $manifest.linked_packages
            supported = $releaseFindings.Count -eq 0
        })
    }

    $retired = @(Get-ChildItem -LiteralPath "crates" -Recurse -File |
        Where-Object { $_.Name -match "manifest-v1|valid-manifest-v1" })
    foreach ($file in $retired) {
        Add-Finding -Code "retired_manifest_artifact" `
            -Path ([IO.Path]::GetRelativePath($repoRoot, $file.FullName)) `
            -Message "pre-production manifest compatibility artifacts must be removed"
    }

    $result = [pscustomobject][ordered]@{
        schema_version = 1
        checked_at = [DateTimeOffset]::UtcNow.ToString("o")
        policy = [pscustomobject][ordered]@{
            versioning = "semantic-versioned canonical packages with an exact current platform tuple"
            supported_window = "current exact tuple only while pre-production"
            deprecation = "advance manifests, fixtures, and baselines in the same fast-forward change"
            unsupported_release_action = "reject installation and inventory the affected release"
            vulnerable_release_action = "block the exact package tuple and rebuild every affected module release"
        }
        supported_platform_versions = $supported
        blocked_package_versions = @()
        releases = @($inventory)
        findings = @($findings)
        passed = $findings.Count -eq 0
    }

    if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
        $fullPath = if ([IO.Path]::IsPathRooted($EvidencePath)) {
            [IO.Path]::GetFullPath($EvidencePath)
        } else {
            [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidencePath))
        }
        [IO.Directory]::CreateDirectory((Split-Path -Parent $fullPath)) | Out-Null
        [IO.File]::WriteAllText(
            $fullPath,
            ($result | ConvertTo-Json -Depth 20) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    $result | ConvertTo-Json -Depth 20
    if (-not $result.passed) { exit 1 }
} finally {
    Pop-Location
}

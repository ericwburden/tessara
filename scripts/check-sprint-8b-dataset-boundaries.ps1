[CmdletBinding()]
param(
    [ValidateSet("Inventory", "RequireClean")][string]$Mode = "RequireClean",
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$findings = [Collections.Generic.List[object]]::new()

function Add-Finding {
    param([string]$Code, [string]$Path, [int]$Line, [string]$Text)
    $findings.Add([pscustomobject][ordered]@{
        code = $Code
        path = $Path.Replace("\", "/")
        line = $Line
        text = $Text.Trim()
    })
}

function Find-SourcePattern {
    param(
        [string]$Code,
        [string[]]$Roots,
        [string]$Pattern,
        [string[]]$ExcludePaths = @()
    )
    foreach ($root in $Roots) {
        $fullRoot = Join-Path $repoRoot $root
        if (-not (Test-Path -LiteralPath $fullRoot)) { continue }
        foreach ($file in Get-ChildItem -LiteralPath $fullRoot -Recurse -File) {
            $relative = [IO.Path]::GetRelativePath($repoRoot, $file.FullName).Replace("\", "/")
            if ($ExcludePaths | Where-Object { $relative -like $_ }) { continue }
            if ($file.Extension -notin @(".rs", ".toml", ".sql", ".json", ".ts", ".tsx")) { continue }
            $lines = @(Get-Content -LiteralPath $file.FullName)
            if ($file.Extension -eq ".rs") {
                $testBoundary = [Array]::FindIndex($lines, [Predicate[string]]{ param($line) $line -cmatch '^#\[cfg\(test\)\]' })
                if ($testBoundary -ge 0) { $lines = @($lines[0..($testBoundary - 1)]) }
            }
            $lineNumber = 0
            foreach ($line in $lines) {
                $lineNumber++
                if ($line -match $Pattern) {
                    Add-Finding -Code $Code -Path $relative -Line $lineNumber -Text $line
                }
            }
        }
    }
}

# Core must not retain a Dataset product router/provider or storage implementation.
foreach ($relative in @(
    "crates/tessara-api/src/datasets",
    "crates/tessara-api/src/dataset_provider.rs",
    "crates/tessara-api/tests/dataset_native_routes.rs"
)) {
    if (Test-Path -LiteralPath (Join-Path $repoRoot $relative)) {
        Add-Finding -Code "core_dataset_owner_residue" -Path $relative -Line 0 `
            -Text "Core still contains a Dataset product owner implementation."
    }
}

# The pre-module domain crate duplicated Dataset validation policy and must not
# remain as an apparently canonical workspace member after the module cutover.
$legacyDatasetCrate = "crates/tessara-datasets"
if (Test-Path -LiteralPath (Join-Path $repoRoot "$legacyDatasetCrate/Cargo.toml")) {
    Add-Finding -Code "legacy_dataset_domain_crate" -Path $legacyDatasetCrate -Line 0 `
        -Text "The retired pre-module Dataset domain crate still exists."
}
$legacyWorkspacePatterns = [ordered]@{
    "Cargo.toml" = '^\s*"crates/tessara-datasets",\s*$'
    "Cargo.lock" = '^name = "tessara-datasets"$'
}
foreach ($workspaceFile in $legacyWorkspacePatterns.Keys) {
    $fullWorkspaceFile = Join-Path $repoRoot $workspaceFile
    $lineNumber = 0
    foreach ($line in Get-Content -LiteralPath $fullWorkspaceFile) {
        $lineNumber++
        if ($line -match $legacyWorkspacePatterns[$workspaceFile]) {
            Add-Finding -Code "legacy_dataset_workspace_identity" -Path $workspaceFile `
                -Line $lineNumber -Text $line
        }
    }
}

Find-SourcePattern -Code "core_dataset_storage_access" `
    -Roots @("crates/tessara-api/src") `
    -Pattern '\b(FROM|JOIN|INTO|UPDATE|DELETE FROM)\s+(datasets|dataset_(revisions|sources|fields|operations|scope_nodes|revision_scope_nodes|tags|provenance))\b' `
    -ExcludePaths @("crates/tessara-api/src/response_export_provider.rs")

Find-SourcePattern -Code "core_dataset_route_residue" `
    -Roots @("crates/tessara-api/src") `
    -Pattern '(^|[^a-zA-Z])(/api/(admin/)?datasets|/datasets(/|"))'

Find-SourcePattern -Code "core_transition_reference_residue" `
    -Roots @("crates/tessara-api/src", "crates/tessara-component-module/src", "crates/tessara-web/src") `
    -Pattern 'tessara\.transition\.dataset(_revision|_major_line)?'

Find-SourcePattern -Code "browser_direct_provider_call" `
    -Roots @("crates/tessara-web-datasets/src") `
    -Pattern '"/api/(me|forms|nodes|admin/users|form-versions/)'

Find-SourcePattern -Code "dataset_definition_branch" `
    -Roots @("crates/tessara-api/src/modules", "crates/tessara-api/src/module_gateway.rs") `
    -Pattern 'tessara\.datasets'

$datasetManifestPath = Join-Path $repoRoot "crates/tessara-dataset-module/manifest.json"
if (-not (Test-Path -LiteralPath $datasetManifestPath)) {
    Add-Finding -Code "dataset_manifest_missing" -Path "crates/tessara-dataset-module/manifest.json" -Line 0 `
        -Text "The independent Dataset manifest is missing."
} else {
    $manifest = Get-Content -Raw -LiteralPath $datasetManifestPath | ConvertFrom-Json -Depth 100
    $expectedConsumed = @(
        "responses.export_checkpoint", "responses.export_start", "responses.export_page",
        "forms.form_version_catalog", "forms.form_version_schema",
        "core.scope_catalog", "core.principal_display_catalog"
    )
    $actualConsumed = @($manifest.consumed_service_actions.authorization_action)
    if (($actualConsumed -join "`n") -cne ($expectedConsumed -join "`n")) {
        Add-Finding -Code "dataset_consumed_action_drift" -Path "crates/tessara-dataset-module/manifest.json" -Line 0 `
            -Text "Consumed service-action declarations do not match the frozen seven-action contract."
    }
}

$result = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    checked_at = [DateTimeOffset]::UtcNow.ToString("O")
    mode = $Mode
    finding_count = $findings.Count
    finding_codes = @($findings | ForEach-Object { $_.code } | Sort-Object -Unique)
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
if ($Mode -eq "RequireClean" -and -not $result.passed) {
    throw "Sprint 8B Dataset boundary check found $($findings.Count) forbidden ownership/coupling occurrence(s)."
}

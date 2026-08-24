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
                $testBoundary = [Array]::FindIndex(
                    $lines,
                    [Predicate[string]]{ param($line) $line -cmatch '^#\[cfg\(test\)\]' }
                )
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

foreach ($relative in @(
    "crates/tessara-api/src/submissions",
    "crates/tessara-api/src/response_owner_actions.rs",
    "crates/tessara-api/src/response_export_provider.rs",
    "crates/tessara-api/src/demo/responses.rs",
    "crates/tessara-web/src/routes/responses.rs"
)) {
    if (Test-Path -LiteralPath (Join-Path $repoRoot $relative)) {
        Add-Finding -Code "core_response_owner_residue" -Path $relative -Line 0 `
            -Text "Core or the root web application still contains a Response product owner implementation."
    }
}

$transitionFixture = "crates/tessara-module-contract/tests/fixtures/transition-responses-v1.json"
if (Test-Path -LiteralPath (Join-Path $repoRoot $transitionFixture)) {
    Add-Finding -Code "response_transition_fixture_residue" -Path $transitionFixture -Line 0 `
        -Text "The retired in-process Response transition fixture still exists."
}

Find-SourcePattern -Code "core_response_storage_access" `
    -Roots @("crates/tessara-api/src") `
    -Pattern '\b(FROM|JOIN|INTO|UPDATE|DELETE FROM)\s+(submissions|submission_values|submission_value_multi|submission_audit_events|response_export_state|response_export_changes|response_owner_action_receipts)\b'

Find-SourcePattern -Code "core_response_route_residue" `
    -Roots @("crates/tessara-api/src", "crates/tessara-web/src") `
    -Pattern '(^|[^a-zA-Z])(/api/(admin/)?(submissions|responses)|/responses(/|"))'

Find-SourcePattern -Code "core_transition_reference_residue" `
    -Roots @("crates/tessara-api/src", "crates/tessara-web/src", "crates/tessara-dataset-module/src") `
    -Pattern 'tessara\.transition\.response'

Find-SourcePattern -Code "workflow_direct_response_storage" `
    -Roots @("crates/tessara-api/src/workflows") `
    -Pattern '\b(submissions|submission_values|submission_value_multi|submission_audit_events)\b'

Find-SourcePattern -Code "response_definition_branch" `
    -Roots @("crates/tessara-api/src/modules", "crates/tessara-api/src/module_gateway.rs") `
    -Pattern 'tessara\.responses'

$manifestPath = Join-Path $repoRoot "crates/tessara-response-module/manifest.json"
if (-not (Test-Path -LiteralPath $manifestPath)) {
    Add-Finding -Code "response_manifest_missing" -Path "crates/tessara-response-module/manifest.json" -Line 0 `
        -Text "The independent Response manifest is missing."
}

$result = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
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
    throw "Sprint 8C Response boundary check found $($findings.Count) forbidden ownership/coupling occurrence(s)."
}

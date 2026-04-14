[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$sourceRoot = Join-Path $repoRoot ".codex\skills"

if (-not (Test-Path -LiteralPath $sourceRoot)) {
    throw "Source skill directory not found: $sourceRoot"
}

$globalRoot = if ($env:CODEX_HOME) {
    Join-Path $env:CODEX_HOME "skills"
} else {
    Join-Path $HOME ".codex\skills"
}

$skillNames = @(
    "tessara-sprint-kickoff",
    "tessara-sprint-closeout"
)

$legacySkillNames = @(
    "sprint-closeout"
)

New-Item -ItemType Directory -Path $globalRoot -Force | Out-Null

foreach ($legacyName in $legacySkillNames) {
    $legacyPath = Join-Path $globalRoot $legacyName
    if (Test-Path -LiteralPath $legacyPath) {
        Remove-Item -LiteralPath $legacyPath -Recurse -Force
    }
}

foreach ($skillName in $skillNames) {
    $sourcePath = Join-Path $sourceRoot $skillName
    if (-not (Test-Path -LiteralPath $sourcePath)) {
        throw "Missing source skill: $sourcePath"
    }

    $destinationPath = Join-Path $globalRoot $skillName
    if (Test-Path -LiteralPath $destinationPath) {
        Remove-Item -LiteralPath $destinationPath -Recurse -Force
    }

    Copy-Item -LiteralPath $sourcePath -Destination $destinationPath -Recurse
}

Write-Output "Synced Tessara skills to $globalRoot"

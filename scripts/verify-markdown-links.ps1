[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$broken = [Collections.Generic.List[string]]::new()
$pattern = '\[[^\]]+\]\((?<target><[^>]+>|[^)\s]+)(?:\s+"[^"]*")?\)'

Push-Location $repoRoot
try {
    foreach ($file in Get-ChildItem -LiteralPath $repoRoot -Recurse -File -Filter "*.md") {
        if ($file.FullName -like "*\target\*" -or $file.FullName -like "*\node_modules\*") {
            continue
        }
        $lineNumber = 0
        foreach ($line in Get-Content -LiteralPath $file.FullName) {
            $lineNumber++
            foreach ($match in [regex]::Matches($line, $pattern)) {
                $target = $match.Groups["target"].Value.Trim('<', '>')
                if (
                    $target.StartsWith("#") -or
                    $target -match "^(https?|mailto|data):"
                ) {
                    continue
                }
                $path = [Uri]::UnescapeDataString(($target -split "#", 2)[0])
                if ([string]::IsNullOrWhiteSpace($path)) { continue }
                $resolved = [IO.Path]::GetFullPath((Join-Path $file.DirectoryName $path))
                if (-not (Test-Path -LiteralPath $resolved)) {
                    $relative = [IO.Path]::GetRelativePath($repoRoot, $file.FullName)
                    $broken.Add("$relative`:$lineNumber -> $target")
                }
            }
        }
    }
    if ($broken.Count -gt 0) {
        $broken | ForEach-Object { Write-Error "Broken Markdown link: $_" }
        exit 1
    }
    Write-Host "Markdown local-link validation passed."
} finally {
    Pop-Location
}

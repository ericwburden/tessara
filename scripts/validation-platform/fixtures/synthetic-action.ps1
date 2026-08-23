[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet(
        "success", "require-port", "echo-secret", "noisy-secret", "mutate-file", "fail", "wait"
    )]
    [string]$Mode,
    [int]$ExitCode = 0,
    [int]$Seconds = 30,
    [string]$Path
)

$ErrorActionPreference = "Stop"
$attemptTempValues = @($env:TEMP, $env:TMP, $env:TMPDIR)
if (@($attemptTempValues | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0 -or
    @($attemptTempValues | Sort-Object -CaseSensitive -Unique).Count -ne 1 -or
    -not (Test-Path -LiteralPath $attemptTempValues[0] -PathType Container)) {
    throw "Synthetic validation action did not receive one attempt-owned temp directory."
}

switch ($Mode) {
    "success" { }
    "require-port" {
        if ([string]::IsNullOrWhiteSpace($env:TESSARA_VP_HTTP_PORT)) {
            throw "Synthetic validation port is absent."
        }
    }
    "echo-secret" {
        [Console]::Out.WriteLine([string]$env:TESSARA_VP_SECRET)
        [Console]::Error.WriteLine([string]$env:TESSARA_VP_SECRET)
    }
    "noisy-secret" {
        $secret = [string]$env:TESSARA_VP_SECRET
        if ([string]::IsNullOrEmpty($secret)) {
            throw "Synthetic validation secret is absent."
        }
        $chunk = (@($secret) * 4096) -join ""
        for ($write = 0; $write -lt 160; $write++) {
            [Console]::Out.Write($chunk)
            [Console]::Error.Write($chunk)
        }
    }
    "mutate-file" {
        if ([string]::IsNullOrWhiteSpace($Path)) {
            throw "Synthetic mutation path is absent."
        }
        [IO.File]::AppendAllText($Path, "changed-during-execution`n")
    }
    "fail" { exit $ExitCode }
    "wait" { Start-Sleep -Seconds $Seconds }
}

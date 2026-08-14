[CmdletBinding()]
param(
    [string]$Lane,
    [switch]$ListLanes,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-8b-evidence-runner.ps1")

Invoke-Sprint8BPhaseRunner -Phase "candidate-rehearsal" -Lane $Lane -ListLanes:$ListLanes -SelfTest:$SelfTest |
    ConvertTo-Json -Depth 100

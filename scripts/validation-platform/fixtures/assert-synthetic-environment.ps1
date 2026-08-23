[CmdletBinding()]
param([Parameter(Mandatory)][int]$ExpectedPort)

$ErrorActionPreference = "Stop"
if ([string]$env:TESSARA_VP_HTTP_PORT -cne [string]$ExpectedPort -or
    [string]$env:TESSARA_VP_SENTINEL -cnotmatch '^lane-[0-9a-f]{64}$') {
    throw "Synthetic validation action did not receive its exact scoped environment."
}
if (Test-Path Env:TESSARA_VP_UNDECLARED) {
    throw "Synthetic validation action inherited an undeclared caller variable."
}
$client = [Net.Sockets.TcpClient]::new()
try {
    $client.Connect([Net.IPAddress]::Loopback, $ExpectedPort)
} finally {
    $client.Dispose()
}

[CmdletBinding()]
param(
    [Parameter(Mandatory)][int]$Port,
    [string]$StopPath,
    [string]$StopCapability
)

$ErrorActionPreference = "Stop"
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, $Port)
$readinessCapability = [Environment]::GetEnvironmentVariable(
    "TESSARA_VALIDATION_READINESS_CAPABILITY", "Process"
)
if ([string]::IsNullOrWhiteSpace($readinessCapability)) {
    throw "Synthetic service requires the validation readiness capability."
}
try {
    $listener.Start()
    while ($true) {
        if (-not [string]::IsNullOrWhiteSpace($StopPath) -and
            (Test-Path -LiteralPath $StopPath -PathType Leaf) -and
            [IO.File]::ReadAllText($StopPath).Trim() -ceq $StopCapability) {
            break
        }
        if ($listener.Pending()) {
            $client = $listener.AcceptTcpClient()
            $reader = $null
            $writer = $null
            try {
                $client.ReceiveTimeout = 1000
                $client.SendTimeout = 1000
                $reader = [IO.StreamReader]::new(
                    $client.GetStream(), [Text.UTF8Encoding]::new($false, $true),
                    $false, 1024, $true
                )
                $writer = [IO.StreamWriter]::new(
                    $client.GetStream(), [Text.UTF8Encoding]::new($false), 1024, $true
                )
                $writer.NewLine = "`n"
                $writer.AutoFlush = $true
                $nonce = $reader.ReadLine()
                if (-not [string]::IsNullOrWhiteSpace($nonce)) {
                    $payload = [Text.UTF8Encoding]::new($false).GetBytes(
                        "$readinessCapability`n$nonce"
                    )
                    $response = [Convert]::ToHexString(
                        [Security.Cryptography.SHA256]::HashData($payload)
                    ).ToLowerInvariant()
                    $writer.WriteLine($response)
                }
            } finally {
                if ($null -ne $reader) { $reader.Dispose() }
                if ($null -ne $writer) { $writer.Dispose() }
                $client.Dispose()
            }
        } else {
            Start-Sleep -Milliseconds 25
        }
    }
} finally {
    $listener.Stop()
}

$ErrorActionPreference = "Stop"
$Arguments = @($args)
$root = [Environment]::GetEnvironmentVariable("TESSARA_VP_DOCKER_SHIM_ROOT", "Process")
if ([string]::IsNullOrWhiteSpace($root)) { throw "Docker shim root is not configured." }
[IO.Directory]::CreateDirectory($root) | Out-Null

function Assert-ExactArguments {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Actual,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Expected,
        [Parameter(Mandatory)][string]$Label
    )
    if ($Actual.Count -ne $Expected.Count) {
        throw "Docker shim $Label argument count is invalid."
    }
    for ($index = 0; $index -lt $Expected.Count; $index++) {
        if ([string]$Actual[$index] -cne [string]$Expected[$index]) {
            throw "Docker shim $Label argument $index is invalid."
        }
    }
}

$projectIndex = [Array]::IndexOf($Arguments, "-p")
$project = if ($projectIndex -ge 0) { $Arguments[$projectIndex + 1] } else { $null }
if ($null -ne $project -and $project -cnotmatch '^[A-Za-z0-9][A-Za-z0-9_-]{0,127}$') {
    throw "Docker shim received an unsafe project identity."
}
$statePath = if ($null -eq $project) { $null } else { Join-Path $root "$project.json" }
$stopPath = if ($null -eq $project) { $null } else { Join-Path $root "$project.stop" }

if (($Arguments -join "`n") -ceq ("info`n--format`njson")) {
    '{"ID":"synthetic-daemon-v1","ServerVersion":"1.0.0"}'
    exit 0
}
if (($Arguments -join "`n") -ceq ("context`nshow")) {
    "default"
    exit 0
}

if ($Arguments.Count -ge 2 -and $Arguments[0] -in @("container", "volume", "network") -and
    $Arguments[1] -ceq "ls") {
    $expectedPrefix = if ($Arguments[0] -ceq "container") {
        @([string]$Arguments[0], "ls", "--all")
    } else { @([string]$Arguments[0], "ls") }
    $filter = @($Arguments | Where-Object { $_ -like 'label=com.docker.compose.project=*' }) | Select-Object -First 1
    $listedProject = [string]$filter -replace '^label=com\.docker\.compose\.project=', ''
    $expectedArguments = @($expectedPrefix + @(
        "--filter", "label=com.docker.compose.project=$listedProject"
    ))
    if (@($Arguments | Where-Object {
            $_ -like 'label=tessara.validation.lease=*'
        }).Count -ne 0) {
        $expectedArguments += @(
            "--filter", "label=tessara.validation.lease=$($env:TESSARA_VALIDATION_TOPOLOGY_LEASE)",
            "--filter", "label=tessara.validation.attempt=$($env:TESSARA_VALIDATION_TOPOLOGY_ATTEMPT)"
        )
    }
    $expectedArguments += @("--format", "{{.ID}}")
    Assert-ExactArguments -Actual $Arguments -Expected $expectedArguments `
        -Label "$($Arguments[0]) inventory"
    $listedStatePath = Join-Path $root "$listedProject.json"
    if (Test-Path -LiteralPath $listedStatePath) {
        $state = Get-Content -Raw -LiteralPath $listedStatePath | ConvertFrom-Json
        $leaseFilter = @($Arguments | Where-Object {
            $_ -like 'label=tessara.validation.lease=*'
        }) | Select-Object -First 1
        $attemptFilter = @($Arguments | Where-Object {
            $_ -like 'label=tessara.validation.attempt=*'
        }) | Select-Object -First 1
        $requestedLease = [string]$leaseFilter -replace '^label=tessara\.validation\.lease=', ''
        $requestedAttempt = [string]$attemptFilter -replace '^label=tessara\.validation\.attempt=', ''
        $matchesOwnership = [string]::IsNullOrWhiteSpace($requestedLease) -or (
            [string]$state.lease -ceq $requestedLease -and
            [string]$state.attempt -ceq $requestedAttempt -and
            $env:TESSARA_VP_DOCKER_SHIM_FOREIGN_LEASE -cne "1"
        )
        if ($matchesOwnership) { "synthetic-$($Arguments[0])" }
    }
    exit 0
}
if ($Arguments[0] -cne "compose" -or [string]::IsNullOrWhiteSpace($project)) {
    throw "Docker shim received an unsupported command: $($Arguments -join ' ')"
}
$expectedComposeFile = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "synthetic-compose.yaml"))
$composeEnvironmentPath = if ($Arguments.Count -gt 2) {
    [IO.Path]::GetFullPath([string]$Arguments[2])
} else { $null }
if ($Arguments.Count -lt 3 -or [string]$Arguments[1] -cne "--env-file" -or
    [IO.Path]::GetFileName($composeEnvironmentPath) -cne "compose-environment.empty" -or
    -not (Test-Path -LiteralPath $composeEnvironmentPath -PathType Leaf) -or
    (Get-Item -LiteralPath $composeEnvironmentPath).Length -ne 0) {
    throw "Docker shim requires the platform-owned empty Compose environment file."
}
$composePrefix = @(
    "compose", "--env-file", $composeEnvironmentPath,
    "-f", $expectedComposeFile, "-p", $project, "--profile", "reference"
)
if ($Arguments.Count -lt ($composePrefix.Count + 1)) {
    throw "Docker shim received an incomplete Compose command."
}
for ($index = 0; $index -lt $composePrefix.Count; $index++) {
    if ([string]$Arguments[$index] -cne [string]$composePrefix[$index]) {
        throw "Docker shim Compose prefix argument $index is invalid."
    }
}
$composeTail = @($Arguments[$composePrefix.Count..($Arguments.Count - 1)])
$composeAction = [string]$composeTail[0]
$expectedTail = switch ($composeAction) {
    "config" { @("config", "--format", "json") }
    "up" { @("up", "-d") }
    "ps" { @("ps", "--all", "--format", "json") }
    "down" { @("down", "--volumes", "--remove-orphans") }
    default { throw "Docker shim received an unsupported Compose action '$composeAction'." }
}
Assert-ExactArguments -Actual $composeTail -Expected $expectedTail `
    -Label "Compose $composeAction"
[IO.File]::AppendAllText(
    (Join-Path $root "docker-transcript.jsonl"),
    (([pscustomobject][ordered]@{ arguments = $Arguments } |
        ConvertTo-Json -Depth 10 -Compress) + "`n"),
    [Text.UTF8Encoding]::new($false)
)

if ($composeAction -ceq "config") {
    $configuredProject = if ($env:TESSARA_VP_DOCKER_SHIM_FOREIGN_CONFIG -ceq "1") {
        "foreign-project"
    } else { $project }
    $labels = @{
        "tessara.validation.lease" = [string]$env:TESSARA_VALIDATION_TOPOLOGY_LEASE
        "tessara.validation.attempt" = [string]$env:TESSARA_VALIDATION_TOPOLOGY_ATTEMPT
    }
    @{
        name = $configuredProject
        services = @{ synthetic = @{
            image = "tessara-validation-platform-synthetic@sha256:$('0' * 64)"
            read_only = $true
            init = $true
            user = "65532:65532"
            environment = @{ TESSARA_VALIDATION_READINESS_CAPABILITY = [string]$env:TESSARA_VALIDATION_READINESS_CAPABILITY }
            labels = $labels
            networks = @("default")
        } }
        networks = @{ default = @{ labels = $labels } }
    } | ConvertTo-Json -Depth 10 -Compress
    exit 0
}
if ($composeAction -ceq "up") {
    $port = [int]$env:TESSARA_VP_HTTP_PORT
    $service = [string]$env:TESSARA_VP_SYNTHETIC_SERVICE
    $stopCapability = [Convert]::ToHexString(
        [Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
    ).ToLowerInvariant()
    [IO.File]::Delete($stopPath)
    $startParameters = @{
        FilePath = [Environment]::ProcessPath
        ArgumentList = @(
            "-NoProfile", "-File", $service, "-Port", [string]$port,
            "-StopPath", $stopPath, "-StopCapability", $stopCapability
        )
        PassThru = $true
    }
    if ($IsWindows) {
        $startParameters.WindowStyle = "Hidden"
    }
    $process = Start-Process @startParameters
    $ownership = [pscustomobject][ordered]@{
        project = $project
        pid = $process.Id
        process_started_at = $process.StartTime.ToUniversalTime().ToString("O")
        port = $port
        lease = [string]$env:TESSARA_VALIDATION_TOPOLOGY_LEASE
        attempt = [string]$env:TESSARA_VALIDATION_TOPOLOGY_ATTEMPT
        stop_path = [IO.Path]::GetFullPath($stopPath)
        stop_capability = $stopCapability
    }
    $ownershipJson = $ownership | ConvertTo-Json -Compress
    $ownershipJson | Set-Content -NoNewline -LiteralPath $statePath
    [IO.File]::AppendAllText(
        (Join-Path $root "synthetic-processes.jsonl"),
        "$ownershipJson`n",
        [Text.UTF8Encoding]::new($false)
    )
    exit 0
}
if ($composeAction -ceq "ps") {
    if (-not (Test-Path -LiteralPath $statePath)) { "[]"; exit 0 }
    @([pscustomobject][ordered]@{
        ID = "synthetic-container-id"
        Service = "synthetic"
        State = "running"
        Health = "healthy"
        Image = "tessara-validation-platform-synthetic@sha256:$('0' * 64)"
    }) |
        ConvertTo-Json -Compress
    exit 0
}
if ($composeAction -ceq "down") {
    if (Test-Path -LiteralPath $statePath) {
        $state = Get-Content -Raw -LiteralPath $statePath | ConvertFrom-Json
        if ([string]$state.project -cne $project -or
            [string]$state.lease -cne [string]$env:TESSARA_VALIDATION_TOPOLOGY_LEASE -or
            [string]$state.attempt -cne [string]$env:TESSARA_VALIDATION_TOPOLOGY_ATTEMPT -or
            [IO.Path]::GetFullPath([string]$state.stop_path) -cne
                [IO.Path]::GetFullPath($stopPath)) {
            throw "Docker shim refused teardown for mismatched synthetic ownership state."
        }
        [IO.File]::WriteAllText(
            $stopPath,
            ([string]$state.stop_capability) + "`n",
            [Text.UTF8Encoding]::new($false)
        )
        $deadline = [DateTimeOffset]::UtcNow.AddSeconds(5)
        while ([DateTimeOffset]::UtcNow -lt $deadline) {
            $process = Get-Process -Id ([int]$state.pid) -ErrorAction SilentlyContinue
            if ($null -eq $process -or
                $process.StartTime.ToUniversalTime().ToString("O") -cne
                    [string]$state.process_started_at) {
                break
            }
            Start-Sleep -Milliseconds 50
        }
        $remaining = Get-Process -Id ([int]$state.pid) -ErrorAction SilentlyContinue
        if ($null -ne $remaining -and
            $remaining.StartTime.ToUniversalTime().ToString("O") -ceq
                [string]$state.process_started_at) {
            throw "Synthetic service did not honor its cooperative stop capability."
        }
        if ($env:TESSARA_VP_DOCKER_SHIM_RETAIN -cne "1") {
            Remove-Item -LiteralPath $statePath -Force
            Remove-Item -LiteralPath $stopPath -Force -ErrorAction SilentlyContinue
        }
    }
    exit 0
}
throw "Docker shim received an unsupported Compose action: $($Arguments -join ' ')"

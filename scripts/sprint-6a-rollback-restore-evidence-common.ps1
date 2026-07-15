Set-StrictMode -Version Latest

$script:Sprint6ARestoreEvidenceSchemaVersion = 3
$script:Sprint6ARestoreEvidenceKind = "tessara_sprint_6a_pre_upgrade_backup_restore_proof"
$script:Sprint6ADatabaseFingerprintContract = "postgres_user_schema_and_logical_rows_v1"
$script:Sprint6ADatabaseFingerprintCanonicalFormat = "utf8_length_framed_schema_and_data_sections_lf_v1"
$script:Sprint6ARestoreEvidenceGeneratorName = "capture-sprint-6a-rollback-restore-evidence.ps1"
$script:Sprint6ARestoreEvidenceGeneratorPath = Join-Path $PSScriptRoot $script:Sprint6ARestoreEvidenceGeneratorName
$script:Sprint6ARestoreEvidenceCommonHelperName = "sprint-6a-rollback-restore-evidence-common.ps1"
$script:Sprint6ARestoreEvidenceCommonHelperPath = Join-Path $PSScriptRoot $script:Sprint6ARestoreEvidenceCommonHelperName

function Get-Sprint6AFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-Sprint6AStringSha256 {
    param([Parameter(Mandatory)][string]$Value)

    $utf8 = [Text.UTF8Encoding]::new($false)
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $hex = [BitConverter]::ToString($algorithm.ComputeHash($utf8.GetBytes($Value)))
        return $hex.Replace("-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Test-Sprint6ADisposableDatabaseName {
    param([Parameter(Mandatory)][string]$Name)

    return $Name -match '(?i)(^|[^a-z0-9])(test|tests|testing|upgrade|clone|rollback|sprint[-_]?6a)([^a-z0-9]|$)'
}

function Invoke-Sprint6AProcess {
    param(
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$Context,
        [hashtable]$Environment = @{},
        [string]$StandardInputPath
    )

    $resolvedCommands = @(Get-Command $Command -CommandType Application -ErrorAction Stop)
    if ($resolvedCommands.Count -ne 1) {
        throw "$Context requires exactly one executable command for '$Command'; found $($resolvedCommands.Count)."
    }
    $resolvedCommand = $resolvedCommands[0]
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $resolvedCommand.Source
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.StandardOutputEncoding = [Text.UTF8Encoding]::new($false)
    $startInfo.StandardErrorEncoding = [Text.UTF8Encoding]::new($false)
    $startInfo.Environment["PGCLIENTENCODING"] = "UTF8"
    foreach ($name in $Environment.Keys) {
        if ([string]::IsNullOrWhiteSpace([string]$name) -or $null -eq $Environment[$name]) {
            throw "$Context received an invalid process environment override."
        }
        $startInfo.Environment[[string]$name] = [string]$Environment[$name]
    }
    if (-not [string]::IsNullOrWhiteSpace($StandardInputPath)) {
        $standardInputFullPath = [IO.Path]::GetFullPath($StandardInputPath)
        if (-not (Test-Path -LiteralPath $standardInputFullPath -PathType Leaf)) {
            throw "$Context standard-input file does not exist."
        }
        $startInfo.RedirectStandardInput = $true
    }
    foreach ($argument in $Arguments) {
        $startInfo.ArgumentList.Add($argument)
    }

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $processStarted = $false
    try {
        if (-not $process.Start()) {
            throw "Could not start $Context."
        }
        $processStarted = $true
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if ($startInfo.RedirectStandardInput) {
            $inputStream = [IO.File]::OpenRead($standardInputFullPath)
            try {
                $inputStream.CopyTo($process.StandardInput.BaseStream)
            } finally {
                $inputStream.Dispose()
                $process.StandardInput.Close()
            }
        }
        $process.WaitForExit()
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0) {
            throw "$Context failed with exit code $($process.ExitCode):`n$stderr"
        }
        if (-not [string]::IsNullOrWhiteSpace($stderr)) {
            throw "$Context emitted an unexpected diagnostic on stderr:`n$stderr"
        }
        return $stdout
    } finally {
        if ($processStarted -and -not $process.HasExited) {
            $process.Kill($true)
            $process.WaitForExit()
        }
        $process.Dispose()
    }
}

function ConvertFrom-Sprint6APostgresDatabaseUrl {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$Context
    )

    try {
        $uri = [Uri]::new($DatabaseUrl, [UriKind]::Absolute)
    } catch {
        throw "$Context is not a valid absolute PostgreSQL URL."
    }
    if ($uri.Scheme -notin @("postgres", "postgresql")) {
        throw "$Context must use the postgres or postgresql URL scheme."
    }
    if (-not [string]::IsNullOrEmpty($uri.Query) -or -not [string]::IsNullOrEmpty($uri.Fragment)) {
        throw "$Context must not contain query or fragment components."
    }
    $separatorIndex = $uri.UserInfo.IndexOf(':')
    if ($separatorIndex -le 0 -or $separatorIndex -eq ($uri.UserInfo.Length - 1)) {
        throw "$Context must contain both a PostgreSQL user and password."
    }
    foreach ($encodedValue in @(
        $uri.UserInfo.Substring(0, $separatorIndex),
        $uri.UserInfo.Substring($separatorIndex + 1),
        $uri.AbsolutePath.TrimStart('/')
    )) {
        if ($encodedValue -match '%(?![0-9a-fA-F]{2})') {
            throw "$Context contains invalid percent encoding."
        }
    }
    try {
        $databaseUser = [Uri]::UnescapeDataString($uri.UserInfo.Substring(0, $separatorIndex))
        $databasePassword = [Uri]::UnescapeDataString($uri.UserInfo.Substring($separatorIndex + 1))
        $databaseName = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart('/'))
    } catch {
        throw "$Context contains invalid percent encoding."
    }
    if ([string]::IsNullOrWhiteSpace($uri.DnsSafeHost)) {
        throw "$Context must identify a PostgreSQL host."
    }
    if ([string]::IsNullOrWhiteSpace($databaseName) -or $databaseName.Contains('/')) {
        throw "$Context must identify exactly one non-empty PostgreSQL database name."
    }
    return [pscustomobject][ordered]@{
        scheme = $uri.Scheme
        host = $uri.DnsSafeHost
        host_port = if ($uri.IsDefaultPort -or $uri.Port -lt 0) { 5432 } else { $uri.Port }
        database_name = $databaseName
        database_user = $databaseUser
        database_password = $databasePassword
    }
}

function Assert-Sprint6AExactDatabaseName {
    param(
        [Parameter(Mandatory)][string]$Actual,
        [Parameter(Mandatory)][string]$Expected,
        [Parameter(Mandatory)][string]$Context
    )

    if ($Actual -cne $Expected) {
        throw "$Context resolved to database '$Actual', not exact case-sensitive database '$Expected'."
    }
}

function Get-Sprint6ADatabaseResetStatements {
    return [string[]]@(
        "SET standard_conforming_strings TO on;",
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = :'target_database' AND pid <> pg_backend_pid();",
        'DROP DATABASE IF EXISTS :"target_database";',
        'CREATE DATABASE :"target_database";'
    )
}

function ConvertTo-Sprint6AContainerPostgresEndpoint {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$ExpectedPostgresUser,
        [Parameter(Mandatory)][string]$ExpectedPostgresPassword,
        [Parameter(Mandatory)][object[]]$PublishedHostBindings,
        [Parameter(Mandatory)][string]$Context
    )

    $endpoint = ConvertFrom-Sprint6APostgresDatabaseUrl -DatabaseUrl $DatabaseUrl -Context $Context
    if ($endpoint.host -notin @("127.0.0.1", "::1")) {
        throw "$Context must use an unambiguous literal IPv4 or IPv6 loopback host bound to the supplied PostgreSQL container."
    }
    $matchingBindings = @($PublishedHostBindings | Where-Object {
        $bindingHostIp = [string]$_.HostIp
        $bindingHostPort = [string]$_.HostPort
        if ($bindingHostPort -cne [string]$endpoint.host_port) {
            return $false
        }
        if ($endpoint.host -ceq "127.0.0.1") {
            return $bindingHostIp -in @("127.0.0.1", "0.0.0.0")
        }
        return $bindingHostIp -in @("::1", "::")
    })
    if ($matchingBindings.Count -ne 1) {
        throw "$Context host '$($endpoint.host)' and port '$($endpoint.host_port)' must match exactly one family-compatible published binding for container port 5432; found $($matchingBindings.Count)."
    }
    $matchingBinding = $matchingBindings[0]
    if ($endpoint.database_user -cne $ExpectedPostgresUser -or $endpoint.database_password -cne $ExpectedPostgresPassword) {
        throw "$Context credentials do not match the supplied PostgreSQL container's configured credentials."
    }
    $safeUser = [Uri]::EscapeDataString($endpoint.database_user)
    $safeDatabaseName = [Uri]::EscapeDataString($endpoint.database_name)
    return [pscustomobject][ordered]@{
        database_name = $endpoint.database_name
        database_user = $endpoint.database_user
        requested_host = $endpoint.host
        requested_host_port = [int]$endpoint.host_port
        binding_host_ip = [string]$matchingBinding.HostIp
        binding_host_port = [int]$matchingBinding.HostPort
        container_database_url = "postgresql://${safeUser}@127.0.0.1:5432/${safeDatabaseName}"
    }
}

function New-Sprint6APostgresClientContext {
    param(
        [Parameter(Mandatory)][string[]]$DatabaseUrls,
        [string]$PostgresClientContainerId,
        [string]$PsqlCommand = "psql",
        [string]$PgDumpCommand = "pg_dump",
        [string]$PgRestoreCommand = "pg_restore",
        [ValidateSet("psql", "pg_dump", "pg_restore")]
        [string[]]$RequiredTools = @("psql")
    )

    if ($DatabaseUrls.Count -eq 0 -or @($DatabaseUrls | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -ne 0) {
        throw "PostgreSQL client context requires at least one non-empty database URL."
    }

    if ([string]::IsNullOrWhiteSpace($PostgresClientContainerId)) {
        $resolvedTools = [ordered]@{}
        $resolvedToolDigests = [ordered]@{}
        foreach ($tool in @("psql", "pg_dump", "pg_restore")) {
            $command = switch ($tool) {
                "psql" { $PsqlCommand }
                "pg_dump" { $PgDumpCommand }
                "pg_restore" { $PgRestoreCommand }
            }
            $matches = @(Get-Command $command -CommandType Application -ErrorAction SilentlyContinue)
            if ($matches.Count -ne 1) {
                throw "Required local PostgreSQL command '$command' for '$tool' is unavailable or ambiguous."
            }
            $resolvedPath = [IO.Path]::GetFullPath($matches[0].Source)
            $resolvedTools[$tool] = $resolvedPath
            $resolvedToolDigests[$tool] = Get-Sprint6AFileSha256 $resolvedPath
        }
        return [pscustomobject][ordered]@{
            mode = "local_executables"
            container_id = $null
            container_name = $null
            image_reference = $null
            image_id = $null
            psql_command = $resolvedTools.psql
            pg_dump_command = $resolvedTools.pg_dump
            pg_restore_command = $resolvedTools.pg_restore
            psql_sha256 = $resolvedToolDigests.psql
            pg_dump_sha256 = $resolvedToolDigests.pg_dump
            pg_restore_sha256 = $resolvedToolDigests.pg_restore
            validated_host_bindings = @()
            allowed_database_url_sha256 = @($DatabaseUrls | ForEach-Object { Get-Sprint6AStringSha256 $_ })
        }
    }

    if ($PsqlCommand -ne "psql" -or $PgDumpCommand -ne "pg_dump" -or $PgRestoreCommand -ne "pg_restore") {
        throw "Container PostgreSQL client mode uses the container's exact psql, pg_dump, and pg_restore commands; custom local command names are not allowed."
    }
    $dockerMatches = @(Get-Command docker -CommandType Application -ErrorAction SilentlyContinue)
    $dockerCommand = if ($dockerMatches.Count -eq 1) {
        $dockerMatches[0].Source
    } else {
        $dockerExeMatches = @($dockerMatches | Where-Object { $_.Name -eq "docker.exe" })
        if ($dockerExeMatches.Count -eq 1) { $dockerExeMatches[0].Source } else { $null }
    }
    if ([string]::IsNullOrWhiteSpace([string]$dockerCommand)) {
        throw "Container PostgreSQL client mode requires one unambiguous docker executable."
    }
    $inspectJson = Invoke-Sprint6AProcess `
        -Command $dockerCommand `
        -Arguments ([string[]]@("inspect", "--type", "container", $PostgresClientContainerId)) `
        -Context "PostgreSQL client container inspection"
    try {
        $inspection = @($inspectJson | ConvertFrom-Json)
    } catch {
        throw "Docker returned invalid inspection data for the supplied PostgreSQL client container."
    }
    if ($inspection.Count -ne 1) {
        throw "The supplied PostgreSQL client container identifier must resolve to exactly one container."
    }
    $container = $inspection[0]
    if ($container.State.Running -isnot [bool] -or -not $container.State.Running) {
        throw "The supplied PostgreSQL client container is not running."
    }
    if ([string]$container.Id -notmatch '^[0-9a-f]{64}$' -or [string]$container.Image -notmatch '^sha256:[0-9a-f]{64}$') {
        throw "The supplied PostgreSQL client container lacks immutable container or image identity."
    }
    $containerName = ([string]$container.Name).TrimStart('/')
    if ([string]::IsNullOrWhiteSpace($containerName) -or [string]::IsNullOrWhiteSpace([string]$container.Config.Image)) {
        throw "The supplied PostgreSQL client container lacks a stable name or image reference."
    }
    $environment = @{}
    foreach ($entry in @($container.Config.Env)) {
        $separator = ([string]$entry).IndexOf('=')
        if ($separator -gt 0) {
            $environment[([string]$entry).Substring(0, $separator)] = ([string]$entry).Substring($separator + 1)
        }
    }
    if ([string]::IsNullOrWhiteSpace([string]$environment.POSTGRES_USER) -or
        [string]::IsNullOrEmpty([string]$environment.POSTGRES_PASSWORD)) {
        throw "Container PostgreSQL client mode requires POSTGRES_USER and POSTGRES_PASSWORD in the supplied container configuration."
    }
    $portProperty = $container.NetworkSettings.Ports.PSObject.Properties["5432/tcp"]
    $portBindings = if ($null -eq $portProperty) { @() } else { @($portProperty.Value) }
    $publishedHostBindings = @($portBindings | ForEach-Object {
        $hostIp = [string]$_.HostIp
        $hostPort = [string]$_.HostPort
        $parsedHostIp = [Net.IPAddress]::None
        if ([string]::IsNullOrWhiteSpace($hostIp) -or
            -not [Net.IPAddress]::TryParse($hostIp, [ref]$parsedHostIp) -or
            $hostPort -notmatch '^\d+$' -or [int]$hostPort -lt 1 -or [int]$hostPort -gt 65535) {
            throw "The supplied PostgreSQL client container has a malformed published 5432/tcp host binding."
        }
        [pscustomobject][ordered]@{
            HostIp = $parsedHostIp.ToString()
            HostPort = $hostPort
        }
    })
    if ($publishedHostBindings.Count -eq 0) {
        throw "The supplied PostgreSQL client container does not publish container port 5432 to the host."
    }
    $validatedHostBindings = @()
    foreach ($index in 0..($DatabaseUrls.Count - 1)) {
        $validatedEndpoint = ConvertTo-Sprint6AContainerPostgresEndpoint `
            -DatabaseUrl $DatabaseUrls[$index] `
            -ExpectedPostgresUser ([string]$environment.POSTGRES_USER) `
            -ExpectedPostgresPassword ([string]$environment.POSTGRES_PASSWORD) `
            -PublishedHostBindings $publishedHostBindings `
            -Context "Database URL $($index + 1)"
        $validatedHostBindings += [pscustomobject][ordered]@{
            requested_host = $validatedEndpoint.requested_host
            requested_host_port = $validatedEndpoint.requested_host_port
            database_name = $validatedEndpoint.database_name
            database_user_sha256 = Get-Sprint6AStringSha256 $validatedEndpoint.database_user
            binding_host_ip = $validatedEndpoint.binding_host_ip
            binding_host_port = $validatedEndpoint.binding_host_port
        }
    }
    [void](Invoke-Sprint6AProcess `
        -Command $dockerCommand `
        -Arguments ([string[]]@(
            "exec", [string]$container.Id, "sh", "-c",
            'command -v psql >/dev/null && command -v pg_dump >/dev/null && command -v pg_restore >/dev/null'
        )) `
        -Context "PostgreSQL client tool discovery in the supplied container")

    return [pscustomobject][ordered]@{
        mode = "docker_container"
        container_id = [string]$container.Id
        container_name = $containerName
        image_reference = [string]$container.Config.Image
        image_id = [string]$container.Image
        docker_command = $dockerCommand
        postgres_user = [string]$environment.POSTGRES_USER
        psql_command = "psql"
        pg_dump_command = "pg_dump"
        pg_restore_command = "pg_restore"
        psql_sha256 = $null
        pg_dump_sha256 = $null
        pg_restore_sha256 = $null
        validated_host_bindings = @($validatedHostBindings)
        allowed_database_url_sha256 = @($DatabaseUrls | ForEach-Object { Get-Sprint6AStringSha256 $_ })
    }
}

function Get-Sprint6APostgresClientEvidence {
    param([Parameter(Mandatory)]$PostgresClientContext)

    return [pscustomobject][ordered]@{
        mode = $PostgresClientContext.mode
        container_id = $PostgresClientContext.container_id
        container_name = $PostgresClientContext.container_name
        image_reference = $PostgresClientContext.image_reference
        image_id = $PostgresClientContext.image_id
        tool_commands = [ordered]@{
            psql = $PostgresClientContext.psql_command
            pg_dump = $PostgresClientContext.pg_dump_command
            pg_restore = $PostgresClientContext.pg_restore_command
        }
        tool_sha256 = [ordered]@{
            psql = $PostgresClientContext.psql_sha256
            pg_dump = $PostgresClientContext.pg_dump_sha256
            pg_restore = $PostgresClientContext.pg_restore_sha256
        }
        validated_host_bindings = @($PostgresClientContext.validated_host_bindings)
    }
}

function Get-Sprint6ALocalPostgresEnvironment {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)]$PostgresClientContext
    )

    $urlSha256 = Get-Sprint6AStringSha256 $DatabaseUrl
    if ($urlSha256 -notin @($PostgresClientContext.allowed_database_url_sha256)) {
        throw "A local PostgreSQL client was asked to use a database URL that was not validated when its context was created."
    }
    $endpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
        -DatabaseUrl $DatabaseUrl `
        -Context "Previously validated local PostgreSQL URL"
    return @{
        PGHOST = $endpoint.host
        PGPORT = [string]$endpoint.host_port
        PGUSER = $endpoint.database_user
        PGPASSWORD = $endpoint.database_password
        PGDATABASE = $endpoint.database_name
    }
}

function Assert-Sprint6ALocalToolIdentity {
    param(
        [Parameter(Mandatory)][ValidateSet("psql", "pg_dump", "pg_restore")][string]$Tool,
        [Parameter(Mandatory)]$PostgresClientContext
    )

    $command = [string]$PostgresClientContext."${Tool}_command"
    $expectedSha256 = [string]$PostgresClientContext."${Tool}_sha256"
    if ([string]::IsNullOrWhiteSpace($command) -or
        -not [IO.Path]::IsPathRooted($command) -or
        -not (Test-Path -LiteralPath $command -PathType Leaf) -or
        $expectedSha256 -notmatch '^[0-9a-f]{64}$' -or
        (Get-Sprint6AFileSha256 $command) -cne $expectedSha256) {
        throw "Local PostgreSQL client executable identity for '$Tool' changed after validation."
    }
    return $command
}

function Get-Sprint6AContainerDatabaseUrl {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)]$PostgresClientContext
    )

    $urlSha256 = Get-Sprint6AStringSha256 $DatabaseUrl
    if ($urlSha256 -notin @($PostgresClientContext.allowed_database_url_sha256)) {
        throw "A PostgreSQL container client was asked to use a database URL that was not validated when its context was created."
    }
    $endpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
        -DatabaseUrl $DatabaseUrl `
        -Context "Previously validated container PostgreSQL URL"
    $safeUser = [Uri]::EscapeDataString($endpoint.database_user)
    $safeDatabaseName = [Uri]::EscapeDataString($endpoint.database_name)
    return "postgresql://${safeUser}@127.0.0.1:5432/${safeDatabaseName}"
}

function Invoke-Sprint6APostgresClientTool {
    param(
        [Parameter(Mandatory)][ValidateSet("psql", "pg_dump", "pg_restore")][string]$Tool,
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)]$PostgresClientContext,
        [Parameter(Mandatory)][string]$Context,
        [string]$StandardInputPath
    )

    if ($PostgresClientContext.mode -eq "local_executables") {
        $command = Assert-Sprint6ALocalToolIdentity `
            -Tool $Tool `
            -PostgresClientContext $PostgresClientContext
        $postgresEnvironment = Get-Sprint6ALocalPostgresEnvironment `
            -DatabaseUrl $DatabaseUrl `
            -PostgresClientContext $PostgresClientContext
        return Invoke-Sprint6AProcess `
            -Command $command `
            -Arguments $Arguments `
            -Context $Context `
            -Environment $postgresEnvironment `
            -StandardInputPath $StandardInputPath
    }
    if ($PostgresClientContext.mode -ne "docker_container") {
        throw "Unsupported PostgreSQL client mode '$($PostgresClientContext.mode)'."
    }
    $containerDatabaseUrl = Get-Sprint6AContainerDatabaseUrl `
        -DatabaseUrl $DatabaseUrl `
        -PostgresClientContext $PostgresClientContext
    [string[]]$containerDatabaseArguments = if ($Tool -eq "psql") {
        [string[]]@($containerDatabaseUrl)
    } else {
        [string[]]@("--dbname", $containerDatabaseUrl)
    }
    $dockerArguments = [string[]]@("exec")
    if (-not [string]::IsNullOrWhiteSpace($StandardInputPath)) {
        $dockerArguments += "--interactive"
    }
    $dockerArguments += [string[]]@(
        "--env", "PGCLIENTENCODING=UTF8",
        $PostgresClientContext.container_id,
        "sh", "-c",
        'export PGPASSWORD="$POSTGRES_PASSWORD"; exec "$@"',
        "tessara-postgres-client",
        $Tool
    )
    $dockerArguments += [string[]]@($containerDatabaseArguments + $Arguments)
    return Invoke-Sprint6AProcess `
        -Command $PostgresClientContext.docker_command `
        -Arguments $dockerArguments `
        -Context $Context `
        -StandardInputPath $StandardInputPath
}

function Remove-Sprint6AContainerTempFile {
    param(
        [Parameter(Mandatory)]$PostgresClientContext,
        [Parameter(Mandatory)][string]$ContainerPath
    )

    if ($PostgresClientContext.mode -ne "docker_container" -or $ContainerPath -notmatch '^/tmp/tessara-sprint-6a-[0-9a-f]{32}\.dump$') {
        throw "Refusing to remove an unrecognized PostgreSQL client container temporary path."
    }
    [void](Invoke-Sprint6AProcess `
        -Command $PostgresClientContext.docker_command `
        -Arguments ([string[]]@("exec", $PostgresClientContext.container_id, "rm", "-f", "--", $ContainerPath)) `
        -Context "PostgreSQL client container temporary archive cleanup")
}

function Invoke-Sprint6APgDumpCapture {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$OutputPath,
        [Parameter(Mandatory)]$PostgresClientContext
    )

    if ($PostgresClientContext.mode -eq "local_executables") {
        [void](Invoke-Sprint6APostgresClientTool `
            -Tool "pg_dump" `
            -DatabaseUrl $DatabaseUrl `
            -Arguments ([string[]]@("--format=custom", "--no-owner", "--no-privileges", "--file", $OutputPath)) `
            -PostgresClientContext $PostgresClientContext `
            -Context "pre-upgrade pg_dump capture")
        return
    }
    $containerPath = "/tmp/tessara-sprint-6a-$([Guid]::NewGuid().ToString('N')).dump"
    try {
        [void](Invoke-Sprint6APostgresClientTool `
            -Tool "pg_dump" `
            -DatabaseUrl $DatabaseUrl `
            -Arguments ([string[]]@("--format=custom", "--no-owner", "--no-privileges", "--file", $containerPath)) `
            -PostgresClientContext $PostgresClientContext `
            -Context "pre-upgrade pg_dump capture in PostgreSQL client container")
        $containerSource = "$($PostgresClientContext.container_id):$containerPath"
        [void](Invoke-Sprint6AProcess `
            -Command $PostgresClientContext.docker_command `
            -Arguments ([string[]]@("cp", $containerSource, $OutputPath)) `
            -Context "copying retained PostgreSQL custom archive from client container")
    } finally {
        Remove-Sprint6AContainerTempFile -PostgresClientContext $PostgresClientContext -ContainerPath $containerPath
    }
}

function Invoke-Sprint6APgRestoreVerification {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$InputPath,
        [Parameter(Mandatory)]$PostgresClientContext
    )

    if ($PostgresClientContext.mode -eq "local_executables") {
        [void](Invoke-Sprint6APostgresClientTool `
            -Tool "pg_restore" `
            -DatabaseUrl $DatabaseUrl `
            -Arguments ([string[]]@("--no-owner", "--no-privileges", "--exit-on-error", $InputPath)) `
            -PostgresClientContext $PostgresClientContext `
            -Context "pre-upgrade pg_restore verification")
        return
    }
    $containerPath = "/tmp/tessara-sprint-6a-$([Guid]::NewGuid().ToString('N')).dump"
    try {
        [void](Invoke-Sprint6AProcess `
            -Command $PostgresClientContext.docker_command `
            -Arguments ([string[]]@(
                "exec", "--interactive", $PostgresClientContext.container_id,
                "sh", "-c", 'umask 077; cat > "$1"',
                "tessara-postgres-client", $containerPath
            )) `
            -Context "streaming retained PostgreSQL custom archive into client container as its execution user" `
            -StandardInputPath $InputPath)
        [void](Invoke-Sprint6APostgresClientTool `
            -Tool "pg_restore" `
            -DatabaseUrl $DatabaseUrl `
            -Arguments ([string[]]@("--no-owner", "--no-privileges", "--exit-on-error", $containerPath)) `
            -PostgresClientContext $PostgresClientContext `
            -Context "pre-upgrade pg_restore verification in PostgreSQL client container")
    } finally {
        Remove-Sprint6AContainerTempFile -PostgresClientContext $PostgresClientContext -ContainerPath $containerPath
    }
}

function Invoke-Sprint6APsqlLines {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$PsqlCommand,
        $PostgresClientContext,
        [Parameter(Mandatory)][string]$Sql
    )

    if ($null -eq $PostgresClientContext) {
        throw "A validated PostgreSQL client context is required so credentials are never passed in process arguments."
    }
    $arguments = [string[]]@(
        "--no-psqlrc",
        "--no-password",
        "--set=ON_ERROR_STOP=1",
        "--set=VERBOSITY=terse",
        "--set=SHOW_CONTEXT=never",
        "--tuples-only",
        "--no-align",
        "--quiet",
        "--pset=footer=off",
        "--command",
        "SET client_min_messages TO warning; $Sql"
    )
    $stdout = Invoke-Sprint6APostgresClientTool `
        -Tool "psql" `
        -DatabaseUrl $DatabaseUrl `
        -Arguments $arguments `
        -PostgresClientContext $PostgresClientContext `
        -Context "deterministic psql query"
    $normalized = $stdout.Replace("`r`n", "`n").Replace("`r", "`n")
    if ($normalized.EndsWith("`n", [StringComparison]::Ordinal)) {
        $normalized = $normalized.Substring(0, $normalized.Length - 1)
    }
    if ($normalized.Length -eq 0) {
        return @()
    }
    return @($normalized.Split([string[]]@("`n"), [StringSplitOptions]::None))
}

function Invoke-Sprint6APsqlScalar {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$PsqlCommand,
        $PostgresClientContext,
        [Parameter(Mandatory)][string]$Sql
    )

    $lines = @(Invoke-Sprint6APsqlLines `
        -DatabaseUrl $DatabaseUrl `
        -PsqlCommand $PsqlCommand `
        -PostgresClientContext $PostgresClientContext `
        -Sql $Sql)
    if ($lines.Count -ne 1) {
        throw "psql scalar query returned $($lines.Count) lines instead of exactly one."
    }
    return $lines[0]
}

function Add-Sprint6ALengthFramedSection {
    param(
        [Parameter(Mandatory)][Text.StringBuilder]$Builder,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Value
    )

    $utf8 = [Text.UTF8Encoding]::new($false)
    [void]$Builder.Append("name_bytes=")
    [void]$Builder.Append($utf8.GetByteCount($Name))
    [void]$Builder.Append("`n")
    [void]$Builder.Append($Name)
    [void]$Builder.Append("`nvalue_bytes=")
    [void]$Builder.Append($utf8.GetByteCount($Value))
    [void]$Builder.Append("`n")
    [void]$Builder.Append($Value)
    [void]$Builder.Append("`n")
}

function Get-Sprint6ADatabaseFingerprint {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$PsqlCommand,
        $PostgresClientContext
    )

    $schemaQueries = [ordered]@{
        database_settings = @'
SELECT jsonb_build_object(
    'encoding', pg_encoding_to_char(d.encoding),
    'collation', d.datcollate,
    'character_classification', d.datctype
)::text
FROM pg_database AS d
WHERE d.datname = current_database();
'@
        namespaces = @'
SELECT COALESCE(
    jsonb_agg(jsonb_build_object('schema_name', n.nspname) ORDER BY n.nspname COLLATE "C"),
    '[]'::jsonb
)::text
FROM pg_namespace AS n
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_';
'@
        relations = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'relation_name', c.relname,
            'relation_kind', c.relkind,
            'persistence', c.relpersistence,
            'replica_identity', c.relreplident,
            'partitioned', c.relkind = 'p'
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_class AS c
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND c.relkind IN ('r', 'p', 'v', 'm', 'S');
'@
        columns = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'relation_name', c.relname,
            'ordinal', a.attnum,
            'column_name', a.attname,
            'data_type', pg_catalog.format_type(a.atttypid, a.atttypmod),
            'not_null', a.attnotnull,
            'identity_kind', a.attidentity,
            'generated_kind', a.attgenerated,
            'default_expression', pg_get_expr(d.adbin, d.adrelid),
            'collation', CASE WHEN a.attcollation = 0 THEN NULL ELSE coll.collname END
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C", a.attnum
    ),
    '[]'::jsonb
)::text
FROM pg_attribute AS a
JOIN pg_class AS c ON c.oid = a.attrelid
JOIN pg_namespace AS n ON n.oid = c.relnamespace
LEFT JOIN pg_attrdef AS d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
LEFT JOIN pg_collation AS coll ON coll.oid = a.attcollation
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND c.relkind IN ('r', 'p', 'v', 'm')
  AND a.attnum > 0
  AND NOT a.attisdropped;
'@
        constraints = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'relation_name', c.relname,
            'constraint_name', con.conname,
            'constraint_kind', con.contype,
            'definition', pg_get_constraintdef(con.oid, true)
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C", con.conname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_constraint AS con
JOIN pg_class AS c ON c.oid = con.conrelid
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_';
'@
        indexes = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'relation_name', c.relname,
            'index_name', i.relname,
            'definition', pg_get_indexdef(i.oid)
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C", i.relname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_index AS ix
JOIN pg_class AS c ON c.oid = ix.indrelid
JOIN pg_class AS i ON i.oid = ix.indexrelid
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_';
'@
        views = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'view_name', c.relname,
            'definition', pg_get_viewdef(c.oid, true)
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_class AS c
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND c.relkind IN ('v', 'm');
'@
        routines = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'routine_name', p.proname,
            'identity_arguments', pg_get_function_identity_arguments(p.oid),
            'routine_kind', p.prokind,
            'definition', pg_get_functiondef(p.oid)
        )
        ORDER BY n.nspname COLLATE "C", p.proname COLLATE "C", pg_get_function_identity_arguments(p.oid) COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_proc AS p
JOIN pg_namespace AS n ON n.oid = p.pronamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND p.prokind IN ('f', 'p');
'@
        triggers = @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'relation_name', c.relname,
            'trigger_name', t.tgname,
            'definition', pg_get_triggerdef(t.oid, true)
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C", t.tgname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_trigger AS t
JOIN pg_class AS c ON c.oid = t.tgrelid
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE NOT t.tgisinternal
  AND n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_';
'@
        user_types_and_extensions = @'
SELECT jsonb_build_object(
    'types', COALESCE((
        SELECT jsonb_agg(
            jsonb_build_object(
                'schema_name', n.nspname,
                'type_name', t.typname,
                'type_kind', t.typtype,
                'base_type', CASE WHEN t.typbasetype = 0 THEN NULL ELSE pg_catalog.format_type(t.typbasetype, t.typtypmod) END,
                'not_null', t.typnotnull,
                'default_expression', t.typdefault,
                'enum_labels', COALESCE((
                    SELECT jsonb_agg(e.enumlabel ORDER BY e.enumsortorder)
                    FROM pg_enum AS e
                    WHERE e.enumtypid = t.oid
                ), '[]'::jsonb)
            )
            ORDER BY n.nspname COLLATE "C", t.typname COLLATE "C"
        )
        FROM pg_type AS t
        JOIN pg_namespace AS n ON n.oid = t.typnamespace
        WHERE n.nspname <> 'information_schema'
          AND n.nspname !~ '^pg_'
          AND t.typtype IN ('d', 'e')
    ), '[]'::jsonb),
    'extensions', COALESCE((
        SELECT jsonb_agg(
            jsonb_build_object('extension_name', e.extname, 'version', e.extversion, 'schema_name', n.nspname)
            ORDER BY e.extname COLLATE "C"
        )
        FROM pg_extension AS e
        JOIN pg_namespace AS n ON n.oid = e.extnamespace
    ), '[]'::jsonb)
)::text;
'@
    }

    $schemaCanonical = [Text.StringBuilder]::new()
    foreach ($sectionName in $schemaQueries.Keys) {
        $sectionValue = Invoke-Sprint6APsqlScalar `
            -DatabaseUrl $DatabaseUrl `
            -PsqlCommand $PsqlCommand `
            -PostgresClientContext $PostgresClientContext `
            -Sql $schemaQueries[$sectionName]
        Add-Sprint6ALengthFramedSection -Builder $schemaCanonical -Name $sectionName -Value $sectionValue
    }

    $relationListJson = Invoke-Sprint6APsqlScalar `
        -DatabaseUrl $DatabaseUrl `
        -PsqlCommand $PsqlCommand `
        -PostgresClientContext $PostgresClientContext `
        -Sql @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'relation_name', c.relname,
            'qualified_name', format('%I.%I', n.nspname, c.relname)
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_class AS c
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND c.relkind = 'r';
'@
    $relations = @($relationListJson | ConvertFrom-Json)
    $dataCanonical = [Text.StringBuilder]::new()
    foreach ($relation in $relations) {
        $rows = Invoke-Sprint6APsqlScalar `
            -DatabaseUrl $DatabaseUrl `
            -PsqlCommand $PsqlCommand `
            -PostgresClientContext $PostgresClientContext `
            -Sql "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY (to_jsonb(t)::text) COLLATE `"C`"), '[]'::jsonb)::text FROM $($relation.qualified_name) AS t;"
        $sectionName = "relation:$($relation.schema_name).$($relation.relation_name)"
        Add-Sprint6ALengthFramedSection -Builder $dataCanonical -Name $sectionName -Value $rows
    }

    $sequenceListJson = Invoke-Sprint6APsqlScalar `
        -DatabaseUrl $DatabaseUrl `
        -PsqlCommand $PsqlCommand `
        -PostgresClientContext $PostgresClientContext `
        -Sql @'
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object(
            'schema_name', n.nspname,
            'sequence_name', c.relname,
            'qualified_name', format('%I.%I', n.nspname, c.relname)
        )
        ORDER BY n.nspname COLLATE "C", c.relname COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM pg_class AS c
JOIN pg_namespace AS n ON n.oid = c.relnamespace
WHERE n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND c.relkind = 'S';
'@
    $sequences = @($sequenceListJson | ConvertFrom-Json)
    foreach ($sequence in $sequences) {
        $state = Invoke-Sprint6APsqlScalar `
            -DatabaseUrl $DatabaseUrl `
            -PsqlCommand $PsqlCommand `
            -PostgresClientContext $PostgresClientContext `
            -Sql "SELECT jsonb_build_object('last_value', last_value, 'is_called', is_called)::text FROM $($sequence.qualified_name);"
        $sectionName = "sequence:$($sequence.schema_name).$($sequence.sequence_name)"
        Add-Sprint6ALengthFramedSection -Builder $dataCanonical -Name $sectionName -Value $state
    }

    $schemaSha256 = Get-Sprint6AStringSha256 $schemaCanonical.ToString()
    $dataSha256 = Get-Sprint6AStringSha256 $dataCanonical.ToString()
    $canonicalSha256 = Get-Sprint6AStringSha256 "schema_sha256=$schemaSha256`ndata_sha256=$dataSha256`n"
    return [pscustomobject][ordered]@{
        contract_id = $script:Sprint6ADatabaseFingerprintContract
        canonical_format = $script:Sprint6ADatabaseFingerprintCanonicalFormat
        schema_sha256 = $schemaSha256
        data_sha256 = $dataSha256
        canonical_sha256 = $canonicalSha256
        schema_section_count = $schemaQueries.Count
        relation_count = $relations.Count
        sequence_count = $sequences.Count
    }
}

function Assert-Sprint6AExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Context
    )

    $actualNames = @($Value.PSObject.Properties.Name | Sort-Object)
    $expectedNames = @($Expected | Sort-Object)
    if (($actualNames -join "`n") -ne ($expectedNames -join "`n")) {
        throw "$Context properties '$($actualNames -join ',')' do not equal required properties '$($expectedNames -join ',')'."
    }
}

function Assert-Sprint6AHash {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Context
    )

    if ($Value -notmatch '^[0-9a-f]{64}$') {
        throw "$Context must be a lowercase SHA-256 value."
    }
}

function Assert-Sprint6AMigrationLedger {
    param(
        [Parameter(Mandatory)][object[]]$Ledger,
        [Parameter(Mandatory)][string]$Context
    )

    if ($Ledger.Count -ne 2) {
        throw "$Context must contain exactly migrations 1 and 2."
    }
    for ($index = 0; $index -lt 2; $index++) {
        if (-not (Test-Sprint6AInteger $Ledger[$index]) -or [int64]$Ledger[$index] -ne ($index + 1)) {
            throw "$Context must contain numeric migrations 1 and 2 in order."
        }
    }
}

function Test-Sprint6AInteger {
    param($Value)

    return $Value -is [sbyte] -or
        $Value -is [byte] -or
        $Value -is [int16] -or
        $Value -is [uint16] -or
        $Value -is [int32] -or
        $Value -is [uint32] -or
        $Value -is [int64] -or
        $Value -is [uint64]
}

function Assert-Sprint6AFingerprintShape {
    param(
        [Parameter(Mandatory)]$Fingerprint,
        [Parameter(Mandatory)][string]$Context
    )

    Assert-Sprint6AExactProperties `
        -Value $Fingerprint `
        -Expected @(
            "contract_id",
            "canonical_format",
            "schema_sha256",
            "data_sha256",
            "canonical_sha256",
            "schema_section_count",
            "relation_count",
            "sequence_count"
        ) `
        -Context $Context
    if ($Fingerprint.contract_id -ne $script:Sprint6ADatabaseFingerprintContract -or
        $Fingerprint.canonical_format -ne $script:Sprint6ADatabaseFingerprintCanonicalFormat) {
        throw "$Context uses an unsupported fingerprint contract."
    }
    foreach ($field in @("schema_sha256", "data_sha256", "canonical_sha256")) {
        Assert-Sprint6AHash -Value ([string]$Fingerprint.$field) -Context "$Context.$field"
    }
    foreach ($field in @("schema_section_count", "relation_count", "sequence_count")) {
        if (-not (Test-Sprint6AInteger $Fingerprint.$field) -or [int64]$Fingerprint.$field -lt 0) {
            throw "$Context.$field must be a non-negative integer."
        }
    }
    if ([int64]$Fingerprint.schema_section_count -ne 10) {
        throw "$Context schema section count does not match the fingerprint contract."
    }
}

function Assert-Sprint6APostgresClientEvidence {
    param(
        [Parameter(Mandatory)]$PostgresClient,
        [Parameter(Mandatory)][string]$Context
    )

    Assert-Sprint6AExactProperties `
        -Value $PostgresClient `
        -Expected @("mode", "container_id", "container_name", "image_reference", "image_id", "tool_commands", "tool_sha256", "validated_host_bindings") `
        -Context $Context
    if ($PostgresClient.mode -notin @("local_executables", "docker_container")) {
        throw "$Context uses unsupported mode '$($PostgresClient.mode)'."
    }
    Assert-Sprint6AExactProperties `
        -Value $PostgresClient.tool_commands `
        -Expected @("psql", "pg_dump", "pg_restore") `
        -Context "$Context tool commands"
    Assert-Sprint6AExactProperties `
        -Value $PostgresClient.tool_sha256 `
        -Expected @("psql", "pg_dump", "pg_restore") `
        -Context "$Context tool digests"
    foreach ($tool in @("psql", "pg_dump", "pg_restore")) {
        $command = [string]$PostgresClient.tool_commands.$tool
        if ([string]::IsNullOrWhiteSpace($command) -or
            $command -match '(?i)(postgres(?:ql)?://|password\s*=)' -or
            $command.Contains("`r") -or $command.Contains("`n")) {
            throw "$Context tool command '$tool' is empty or contains credential-bearing data."
        }
    }
    if ($PostgresClient.mode -eq "local_executables") {
        foreach ($field in @("container_id", "container_name", "image_reference", "image_id")) {
            if ($null -ne $PostgresClient.$field) {
                throw "$Context local-executable mode must not claim container identity."
            }
        }
        if (@($PostgresClient.validated_host_bindings).Count -ne 0) {
            throw "$Context local-executable mode must not claim Docker host bindings."
        }
        foreach ($tool in @("psql", "pg_dump", "pg_restore")) {
            $command = [string]$PostgresClient.tool_commands.$tool
            $digest = [string]$PostgresClient.tool_sha256.$tool
            if (-not [IO.Path]::IsPathRooted($command) -or
                -not (Test-Path -LiteralPath $command -PathType Leaf)) {
                throw "$Context local-executable '$tool' path is not an available exact executable identity."
            }
            Assert-Sprint6AHash -Value $digest -Context "$Context local-executable '$tool' digest"
            if ((Get-Sprint6AFileSha256 $command) -cne $digest) {
                throw "$Context local-executable '$tool' digest does not match the exact available executable."
            }
        }
        return
    }
    if ([string]$PostgresClient.container_id -notmatch '^[0-9a-f]{64}$' -or
        [string]$PostgresClient.image_id -notmatch '^sha256:[0-9a-f]{64}$' -or
        [string]::IsNullOrWhiteSpace([string]$PostgresClient.container_name) -or
        [string]::IsNullOrWhiteSpace([string]$PostgresClient.image_reference)) {
        throw "$Context container mode lacks complete immutable container identity."
    }
    if ($PostgresClient.tool_commands.psql -ne "psql" -or
        $PostgresClient.tool_commands.pg_dump -ne "pg_dump" -or
        $PostgresClient.tool_commands.pg_restore -ne "pg_restore") {
        throw "$Context container mode must use the container's exact PostgreSQL client command names."
    }
    foreach ($tool in @("psql", "pg_dump", "pg_restore")) {
        if ($null -ne $PostgresClient.tool_sha256.$tool) {
            throw "$Context container mode must not claim host-executable digests."
        }
    }
    $validatedHostBindings = @($PostgresClient.validated_host_bindings)
    if ($validatedHostBindings.Count -eq 0) {
        throw "$Context container mode must record at least one validated host binding."
    }
    foreach ($binding in $validatedHostBindings) {
        Assert-Sprint6AExactProperties `
            -Value $binding `
            -Expected @("requested_host", "requested_host_port", "database_name", "database_user_sha256", "binding_host_ip", "binding_host_port") `
            -Context "$Context validated host binding"
        if ($binding.requested_host -notin @("127.0.0.1", "::1") -or
            [string]::IsNullOrWhiteSpace([string]$binding.database_name) -or
            -not (Test-Sprint6AInteger $binding.requested_host_port) -or
            -not (Test-Sprint6AInteger $binding.binding_host_port) -or
            [int]$binding.requested_host_port -ne [int]$binding.binding_host_port -or
            [int]$binding.binding_host_port -lt 1 -or [int]$binding.binding_host_port -gt 65535) {
            throw "$Context contains an invalid sanitized host-binding claim."
        }
        Assert-Sprint6AHash -Value ([string]$binding.database_user_sha256) -Context "$Context database-user digest"
        $allowedBindingIps = if ($binding.requested_host -ceq "127.0.0.1") {
            @("127.0.0.1", "0.0.0.0")
        } else {
            @("::1", "::")
        }
        if ([string]$binding.binding_host_ip -notin $allowedBindingIps) {
            throw "$Context host binding is not family-compatible with its requested loopback host."
        }
    }
}

function Assert-Sprint6ARestoreEvidenceDocument {
    param(
        [Parameter(Mandatory)][string]$EvidencePath,
        [Parameter(Mandatory)][string]$ExpectedTargetDatabaseName
    )

    $evidenceFullPath = [IO.Path]::GetFullPath($EvidencePath)
    if (-not (Test-Path -LiteralPath $evidenceFullPath -PathType Leaf)) {
        throw "Restore evidence '$evidenceFullPath' does not exist."
    }
    try {
        $rawJson = Get-Content -LiteralPath $evidenceFullPath -Raw -Encoding UTF8
        $convertArguments = @{ InputObject = $rawJson }
        if ((Get-Command ConvertFrom-Json).Parameters.ContainsKey("DateKind")) {
            $convertArguments.DateKind = "String"
        }
        $document = ConvertFrom-Json @convertArguments
    } catch {
        throw "Restore evidence '$evidenceFullPath' is not valid UTF-8 JSON: $($_.Exception.Message)"
    }

    Assert-Sprint6AExactProperties `
        -Value $document `
        -Expected @(
            "schema_version",
            "evidence_kind",
            "generated_at_utc",
            "generator",
            "postgres_client",
            "backup_artifact",
            "source_database",
            "restored_target_database",
            "restore_operation",
            "result"
        ) `
        -Context "Restore evidence"
    if (-not (Test-Sprint6AInteger $document.schema_version) -or
        [int64]$document.schema_version -ne $script:Sprint6ARestoreEvidenceSchemaVersion) {
        throw "Restore evidence uses an unsupported schema version."
    }
    if ($document.evidence_kind -ne $script:Sprint6ARestoreEvidenceKind) {
        throw "Restore evidence uses an unsupported evidence kind."
    }
    if ($document.result -ne "passed") {
        throw "Restore evidence does not record a passed capture-and-restore operation."
    }
    $parsedTimestamp = [DateTimeOffset]::MinValue
    $timestampParsed = [DateTimeOffset]::TryParse(
        [string]$document.generated_at_utc,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$parsedTimestamp
    )
    if (-not $timestampParsed -or $parsedTimestamp.Offset -ne [TimeSpan]::Zero) {
        throw "Restore evidence generated_at_utc must be an unambiguous UTC timestamp."
    }

    Assert-Sprint6AExactProperties `
        -Value $document.generator `
        -Expected @("script_name", "script_sha256", "common_helper_name", "common_helper_sha256", "powershell_version") `
        -Context "Restore evidence generator"
    if ($document.generator.script_name -ne $script:Sprint6ARestoreEvidenceGeneratorName) {
        throw "Restore evidence identifies an unsupported generator."
    }
    Assert-Sprint6AHash -Value ([string]$document.generator.script_sha256) -Context "Restore evidence generator script digest"
    if (-not (Test-Path -LiteralPath $script:Sprint6ARestoreEvidenceGeneratorPath -PathType Leaf)) {
        throw "Restore-evidence generator '$script:Sprint6ARestoreEvidenceGeneratorPath' is unavailable for digest verification."
    }
    $generatorSha256 = Get-Sprint6AFileSha256 $script:Sprint6ARestoreEvidenceGeneratorPath
    if ($document.generator.script_sha256 -ne $generatorSha256) {
        throw "Restore evidence generator digest does not equal the exact available capture helper."
    }
    if ($document.generator.common_helper_name -cne $script:Sprint6ARestoreEvidenceCommonHelperName) {
        throw "Restore evidence identifies an unsupported dot-sourced common helper."
    }
    Assert-Sprint6AHash `
        -Value ([string]$document.generator.common_helper_sha256) `
        -Context "Restore evidence common-helper digest"
    if (-not (Test-Path -LiteralPath $script:Sprint6ARestoreEvidenceCommonHelperPath -PathType Leaf) -or
        $document.generator.common_helper_sha256 -cne (Get-Sprint6AFileSha256 $script:Sprint6ARestoreEvidenceCommonHelperPath)) {
        throw "Restore evidence common-helper digest does not equal the exact available dot-sourced helper."
    }
    if ([string]::IsNullOrWhiteSpace([string]$document.generator.powershell_version)) {
        throw "Restore evidence does not identify its PowerShell version."
    }

    Assert-Sprint6APostgresClientEvidence `
        -PostgresClient $document.postgres_client `
        -Context "Restore evidence PostgreSQL client"

    Assert-Sprint6AExactProperties `
        -Value $document.backup_artifact `
        -Expected @("path_relative_to_evidence", "sha256", "length_bytes", "format", "source_database_name") `
        -Context "Restore evidence backup artifact"
    if ([string]::IsNullOrWhiteSpace([string]$document.backup_artifact.path_relative_to_evidence) -or
        [IO.Path]::IsPathRooted([string]$document.backup_artifact.path_relative_to_evidence)) {
        throw "Restore evidence backup artifact path must be a non-empty path relative to the evidence file."
    }
    if ($document.backup_artifact.format -ne "postgresql_custom_archive") {
        throw "Restore evidence backup artifact format is unsupported."
    }
    Assert-Sprint6AHash -Value ([string]$document.backup_artifact.sha256) -Context "Restore evidence backup artifact digest"
    if (-not (Test-Sprint6AInteger $document.backup_artifact.length_bytes) -or
        [int64]$document.backup_artifact.length_bytes -le 0) {
        throw "Restore evidence backup artifact length must be a positive integer."
    }
    $backupFullPath = [IO.Path]::GetFullPath((Join-Path (Split-Path -Parent $evidenceFullPath) ([string]$document.backup_artifact.path_relative_to_evidence)))
    if (-not (Test-Path -LiteralPath $backupFullPath -PathType Leaf)) {
        throw "Retained backup artifact '$backupFullPath' does not exist."
    }
    $backupFile = Get-Item -LiteralPath $backupFullPath
    if ($backupFile.Length -ne [int64]$document.backup_artifact.length_bytes) {
        throw "Retained backup artifact length does not equal restore evidence."
    }
    $backupSha256 = Get-Sprint6AFileSha256 $backupFullPath
    if ($backupSha256 -ne $document.backup_artifact.sha256) {
        throw "Retained backup artifact SHA-256 does not equal restore evidence."
    }
    $backupStream = [IO.File]::OpenRead($backupFullPath)
    try {
        $header = [byte[]]::new(5)
        if ($backupStream.Read($header, 0, $header.Length) -ne $header.Length -or
            [Text.Encoding]::ASCII.GetString($header) -ne "PGDMP") {
            throw "Retained backup artifact is not a PostgreSQL custom archive."
        }
    } finally {
        $backupStream.Dispose()
    }

    foreach ($databaseField in @("source_database", "restored_target_database")) {
        Assert-Sprint6AExactProperties `
            -Value $document.$databaseField `
            -Expected @("database_name", "migration_ledger", "fingerprint") `
            -Context "Restore evidence $databaseField"
        if ([string]::IsNullOrWhiteSpace([string]$document.$databaseField.database_name)) {
            throw "Restore evidence $databaseField database name must not be empty."
        }
        Assert-Sprint6AMigrationLedger `
            -Ledger @($document.$databaseField.migration_ledger) `
            -Context "Restore evidence $databaseField migration ledger"
        Assert-Sprint6AFingerprintShape `
            -Fingerprint $document.$databaseField.fingerprint `
            -Context "Restore evidence $databaseField fingerprint"
    }
    if ($document.backup_artifact.source_database_name -cne $document.source_database.database_name) {
        throw "Restore evidence backup source database does not equal the fingerprinted source database."
    }
    if ($document.source_database.database_name -ceq $document.restored_target_database.database_name) {
        throw "Restore evidence source and restored target database names must be distinct."
    }
    if ($document.restored_target_database.database_name -cne $ExpectedTargetDatabaseName) {
        throw "Restore evidence target database '$($document.restored_target_database.database_name)' does not equal expected database '$ExpectedTargetDatabaseName'."
    }
    if (-not (Test-Sprint6ADisposableDatabaseName ([string]$document.restored_target_database.database_name))) {
        throw "Restore evidence target database is not clearly disposable."
    }
    $sourceFingerprintJson = $document.source_database.fingerprint | ConvertTo-Json -Depth 4 -Compress
    $targetFingerprintJson = $document.restored_target_database.fingerprint | ConvertTo-Json -Depth 4 -Compress
    if ($sourceFingerprintJson -ne $targetFingerprintJson) {
        throw "Restore evidence source and target database fingerprints do not match exactly."
    }

    Assert-Sprint6AExactProperties `
        -Value $document.restore_operation `
        -Expected @("target_was_destructively_recreated", "backup_sha256_used", "credential_redaction", "archive_transfer", "commands") `
        -Context "Restore evidence operation"
    if ($document.restore_operation.target_was_destructively_recreated -isnot [bool] -or
        -not $document.restore_operation.target_was_destructively_recreated) {
        throw "Restore evidence must record that the disposable target was destructively recreated."
    }
    if ($document.restore_operation.backup_sha256_used -ne $document.backup_artifact.sha256) {
        throw "Restore operation backup digest does not equal the retained artifact digest."
    }
    if ($document.restore_operation.credential_redaction -ne "all_database_urls_replaced_with_named_placeholders") {
        throw "Restore evidence does not declare the required credential redaction contract."
    }
    $commands = @($document.restore_operation.commands)
    $expectedTools = if ($document.postgres_client.mode -eq "docker_container") {
        @("psql", "pg_dump", "docker", "psql", "docker", "pg_restore", "psql")
    } else {
        @("psql", "pg_dump", "psql", "pg_restore", "psql")
    }
    $expectedArchiveTransfer = if ($document.postgres_client.mode -eq "docker_container") {
        "docker_cp_out_and_execution_user_stdin_in_unique_container_temp_paths_with_finally_cleanup"
    } else {
        "direct_host_path"
    }
    if ($document.restore_operation.archive_transfer -ne $expectedArchiveTransfer) {
        throw "Restore evidence archive transfer contract does not match PostgreSQL client mode."
    }
    if ($commands.Count -ne $expectedTools.Count) {
        throw "Restore evidence contains $($commands.Count) sanitized command summaries; PostgreSQL client mode requires $($expectedTools.Count)."
    }
    $allArguments = @()
    for ($commandIndex = 0; $commandIndex -lt $commands.Count; $commandIndex++) {
        $command = $commands[$commandIndex]
        Assert-Sprint6AExactProperties -Value $command -Expected @("tool", "arguments") -Context "Restore evidence command"
        if ($command.tool -ne $expectedTools[$commandIndex]) {
            throw "Restore evidence command $($commandIndex + 1) uses tool '$($command.tool)' instead of required '$($expectedTools[$commandIndex])'."
        }
        $expectedArguments = if ($document.postgres_client.mode -eq "docker_container") {
            switch ($commandIndex) {
                0 { [string[]]@("<postgres-client-container-id>", "<source-database-url>", "<read-only-name-ledger-fingerprint-verification>") }
                1 { [string[]]@("<postgres-client-container-id>", "--dbname", "<source-database-url>", "--format=custom", "--no-owner", "--no-privileges", "--file", "<unique-container-dump-path>") }
                2 { [string[]]@("cp", "<postgres-client-container-id>:<unique-container-dump-path>", "<retained-backup-path>") }
                3 { [string[]]@("<postgres-client-container-id>", "<maintenance-database-url>", "<terminate-drop-create-disposable-target>", "<target-database-name>") }
                4 { [string[]]@("exec", "--interactive", "<postgres-client-container-id>", "<retained-backup-stdin>", "<unique-container-dump-path>") }
                5 { [string[]]@("<postgres-client-container-id>", "--dbname", "<target-database-url>", "--no-owner", "--no-privileges", "--exit-on-error", "<unique-container-dump-path>") }
                6 { [string[]]@("<postgres-client-container-id>", "<target-database-url>", "<read-only-name-ledger-fingerprint-verification>") }
            }
        } else {
            switch ($commandIndex) {
                0 { [string[]]@("<source-database-connection-environment>", "<read-only-name-ledger-fingerprint-verification>") }
                1 { [string[]]@("<source-database-connection-environment>", "--format=custom", "--no-owner", "--no-privileges", "--file", "<retained-backup-path>") }
                2 { [string[]]@("<maintenance-database-connection-environment>", "<terminate-drop-create-disposable-target>", "<target-database-name>") }
                3 { [string[]]@("<target-database-connection-environment>", "--no-owner", "--no-privileges", "--exit-on-error", "<retained-backup-path>") }
                4 { [string[]]@("<target-database-connection-environment>", "<read-only-name-ledger-fingerprint-verification>") }
            }
        }
        $actualArguments = @($command.arguments)
        if (($actualArguments -join "`n") -ne ($expectedArguments -join "`n")) {
            throw "Restore evidence command $($commandIndex + 1) arguments do not equal the required sanitized operation contract."
        }
        foreach ($argument in @($command.arguments)) {
            if ($argument -isnot [string] -or $argument -match '(?i)(postgres(?:ql)?://|password\s*=)' -or
                ($argument -match '@' -and $argument -notmatch '^<[^>]+>$')) {
                throw "Restore evidence command arguments contain a database URL or credential-bearing value."
            }
            $allArguments += $argument
        }
    }
    $requiredConnectionPlaceholders = if ($document.postgres_client.mode -eq "docker_container") {
        @("<source-database-url>", "<maintenance-database-url>", "<target-database-url>")
    } else {
        @("<source-database-connection-environment>", "<maintenance-database-connection-environment>", "<target-database-connection-environment>")
    }
    foreach ($placeholder in $requiredConnectionPlaceholders) {
        if ($placeholder -notin $allArguments) {
            throw "Restore evidence command summaries omit required credential placeholder '$placeholder'."
        }
    }
    if ($document.postgres_client.mode -eq "docker_container") {
        foreach ($placeholder in @("<postgres-client-container-id>:<unique-container-dump-path>", "<unique-container-dump-path>", "<postgres-client-container-id>")) {
            if ($placeholder -notin $allArguments) {
                throw "Container restore evidence command summaries omit required sanitized placeholder '$placeholder'."
            }
        }
    }

    return [pscustomobject][ordered]@{
        path = $evidenceFullPath
        sha256 = Get-Sprint6AFileSha256 $evidenceFullPath
        backup_path = $backupFullPath
        backup_sha256 = $backupSha256
        backup_length_bytes = $backupFile.Length
        document = $document
    }
}

function Write-Sprint6AUtf8Json {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Path
    )

    $json = $Value | ConvertTo-Json -Depth 12
    [IO.File]::WriteAllText($Path, $json + "`n", [Text.UTF8Encoding]::new($false))
}

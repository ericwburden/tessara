[CmdletBinding()]
param(
    [switch]$Fast,
    [switch]$SelfTest,
    [switch]$RetainCargoTarget
)

if ($PSVersionTable.PSEdition -cne 'Core' -or
    $PSVersionTable.PSVersion -lt [Version]'7.3') {
    throw "Tessara validation requires PowerShell Core 7.3 or newer. Invoke it with pwsh."
}

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$repoRoot = Split-Path -Parent $PSScriptRoot
$cargoPolicyModulePath = [IO.Path]::GetFullPath(
    (Join-Path $PSScriptRoot "tessara-cargo-build-policy.psm1")
)
$loadedCargoPolicy = @(Get-Module | Where-Object {
    -not [string]::IsNullOrWhiteSpace($_.Path) -and
    [IO.Path]::GetFullPath($_.Path) -eq $cargoPolicyModulePath
}) | Select-Object -First 1
if ($null -eq $loadedCargoPolicy) {
    Import-Module $cargoPolicyModulePath
}
$loadedCargoPolicyIdentity = Get-TessaraCargoBuildPolicyIdentity
$currentCargoPolicySha256 = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $cargoPolicyModulePath
).Hash.ToLowerInvariant()
if ([string]$loadedCargoPolicyIdentity.module_sha256 -cne $currentCargoPolicySha256) {
    throw "The loaded Cargo build policy does not match its current source bytes. Start a fresh PowerShell process."
}
$fullValidationDatabaseEnvironmentNames = @(
    "TEST_API_DATABASE_URL",
    "TEST_API_FRESH_DATABASE_URL",
    "TEST_SQLX_DATABASE_URL",
    "TEST_REFERENCE_MODULE_DATABASE_URL",
    "TEST_COMPONENT_MODULE_DATABASE_URL",
    "TEST_API_ENROLLMENT_DATABASE_URL",
    "TEST_INSTALLATION_CONTROL_DATABASE_URL"
)
$destructiveFreshResetAcknowledgement =
    "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"
$fullValidationProcessEnvironmentNames = @(
    $fullValidationDatabaseEnvironmentNames +
        "SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET"
)
$validationDatabaseOwnershipMarker = "tessara-validation-disposable-v1"
$blockedDatabaseUrl = "__TESSARA_VALIDATION_UNDECLARED_DATABASE_URL__"
$fullValidationDatabasePackageScopes = @(
    [pscustomobject][ordered]@{
        package = "tessara-reference-scoped-records"
        environment_name = "TEST_REFERENCE_MODULE_DATABASE_URL"
        label = "Reference Scoped Records database tests"
    },
    [pscustomobject][ordered]@{
        package = "tessara-component-module"
        environment_name = "TEST_COMPONENT_MODULE_DATABASE_URL"
        label = "Extracted Component module database tests"
    },
    [pscustomobject][ordered]@{
        package = "tessara-installation-control"
        environment_name = "TEST_INSTALLATION_CONTROL_DATABASE_URL"
        label = "Installation-control database tests"
    }
)
$fullValidationIsolatedCargoPackages = @(
    "tessara-api",
    "tessara-dataset-module"
) + @(
    $fullValidationDatabasePackageScopes | ForEach-Object { [string]$_.package }
)
$fullValidationApiGeneralSkipTests = @(
    "core_security::tests::local_enrollment_is_atomic_global_and_idempotent",
    "composition::tests::fresh_baseline_enrolls_dataset_security_before_core_actor_bootstrap",
    "fresh_startup_and_seed_assignment_lock_order_use_a_separate_database"
)
$fullValidationApiEnrollmentTest =
    "core_security::tests::local_enrollment_is_atomic_global_and_idempotent"
$fullValidationApiSqlxTest =
    "composition::tests::fresh_baseline_enrolls_dataset_security_before_core_actor_bootstrap"
$fullValidationApiFreshTest =
    "fresh_startup_and_seed_assignment_lock_order_use_a_separate_database"
$fullValidationApiReleaseTimingTest =
    "resource_reference_restricted_known_random_latency_profile"
$fastValidationApiDatabaseSkipTests = @(
    $fullValidationApiEnrollmentTest,
    $fullValidationApiSqlxTest,
        "modules::service::tests::catalog_sync_is_repeatable_concurrent_and_rolls_back_injected_failure"
)
$script:ActiveValidationCargoBinding = $null
$script:ActiveValidationPsqlBinding = $null
$script:ActiveValidationCargoPolicyState = $null
$script:InitialValidationCargoControlValues = [ordered]@{}

function Test-TessaraDisposableDatabaseName {
    param([Parameter(Mandatory)][string]$DatabaseName)

    return [bool]($DatabaseName -match
        '^(?i:tessara[-_](?:test|tests|testing)(?:[-_][A-Za-z0-9][A-Za-z0-9_-]*)?)$')
}

function ConvertFrom-TessaraValidationDatabaseUrl {
    param(
        [Parameter(Mandatory)][string]$EnvironmentName,
        [Parameter(Mandatory)][string]$DatabaseUrl
    )

    try {
        $uri = [Uri]::new($DatabaseUrl, [UriKind]::Absolute)
    } catch {
        throw "Full validation requires $EnvironmentName to be an absolute PostgreSQL URL."
    }
    if ($uri.Scheme -notin @("postgres", "postgresql") -or
        [string]::IsNullOrWhiteSpace($uri.Host)) {
        throw "Full validation requires $EnvironmentName to be an absolute postgres:// or postgresql:// URL with a host."
    }

    $databaseName = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart("/"))
    if ([string]::IsNullOrWhiteSpace($databaseName) -or
        $databaseName.Contains("/") -or
        $databaseName -notmatch "^[A-Za-z_][A-Za-z0-9_-]*$") {
        throw "Full validation requires $EnvironmentName to name one explicit PostgreSQL database."
    }
    if (-not (Test-TessaraDisposableDatabaseName -DatabaseName $databaseName)) {
        throw "Full validation refuses $EnvironmentName database '$databaseName': its name is outside the tessara_test/tessara_tests/tessara_testing disposable namespace."
    }

    $port = if ($uri.IsDefaultPort -or $uri.Port -lt 0) { 5432 } else { $uri.Port }
    $userInfo = [string]$uri.UserInfo
    if ([string]::IsNullOrWhiteSpace($userInfo)) {
        throw "Full validation requires $EnvironmentName to name an explicit PostgreSQL role."
    }
    $userParts = $userInfo.Split(':', 2)
    $role = [Uri]::UnescapeDataString($userParts[0])
    $password = if ($userParts.Count -eq 2) {
        [Uri]::UnescapeDataString($userParts[1])
    } else { $null }
    if ([string]::IsNullOrWhiteSpace($role)) {
        throw "Full validation requires $EnvironmentName to name an explicit PostgreSQL role."
    }
    if ($userParts.Count -ne 2 -or [string]::IsNullOrWhiteSpace($password)) {
        throw "Full validation requires $EnvironmentName to declare one explicit nonblank PostgreSQL password so neither psql nor SQLx can fall back to ambient credential files."
    }
    $normalizedHost = if ($uri.Host.ToLowerInvariant() -in @(
            "localhost", "127.0.0.1", "::1", "[::1]"
        )) { "loopback" } else { $uri.IdnHost.ToLowerInvariant() }
    if (-not [string]::IsNullOrEmpty($uri.Fragment)) {
        throw "Full validation rejects fragments in $EnvironmentName."
    }
    $sslMode = $null
    if (-not [string]::IsNullOrEmpty($uri.Query)) {
        $queryParts = @($uri.Query.TrimStart('?') -split '&')
        if ($queryParts.Count -ne 1 -or $queryParts[0] -notmatch '^([^=]+)=(.*)$') {
            throw "Full validation permits only one explicit sslmode query parameter in $EnvironmentName; connection-identity overrides are forbidden."
        }
        $queryName = [Uri]::UnescapeDataString([string]$Matches[1])
        $queryValue = [Uri]::UnescapeDataString([string]$Matches[2])
        if ($queryName -cne "sslmode" -or
            $queryValue -notin @(
                "disable", "allow", "prefer", "require", "verify-ca", "verify-full"
            )) {
            throw "Full validation permits only one explicit sslmode query parameter in $EnvironmentName; connection-identity overrides are forbidden."
        }
        $sslMode = $queryValue
    }
    [pscustomobject][ordered]@{
        EnvironmentName = $EnvironmentName
        DatabaseName = $databaseName
        Host = $uri.Host
        Port = $port
        Role = $role
        Password = $password
        SslMode = $sslMode
        Identity = "$normalizedHost`:$port/$($databaseName.ToLowerInvariant())"
        RoleIdentity = ConvertTo-Json -Compress -InputObject @(
            $normalizedHost, $port, $role
        )
    }
}

function New-TessaraAuthenticatedDatabaseIdentityRecord {
    param(
        [Parameter(Mandatory)][string]$ServerAddress,
        [Parameter(Mandatory)][int]$ServerPort,
        [Parameter(Mandatory)][string]$DatabaseName,
        [Parameter(Mandatory)][string]$RoleName
    )

    if ([string]::IsNullOrWhiteSpace($ServerAddress) -or
        $ServerPort -lt 1 -or $ServerPort -gt 65535 -or
        [string]::IsNullOrWhiteSpace($DatabaseName) -or
        [string]::IsNullOrWhiteSpace($RoleName)) {
        throw "Authenticated database identity contains an invalid nonsecret database or role field."
    }
    [pscustomobject][ordered]@{
        server_address = $ServerAddress
        server_port = $ServerPort
        database_name = $DatabaseName
        role_name = $RoleName
        database_identity = ConvertTo-Json -Compress -InputObject @(
            $ServerAddress, $ServerPort, $DatabaseName
        )
        server_role_identity = ConvertTo-Json -Compress -InputObject @(
            $ServerAddress, $ServerPort, $RoleName
        )
    }
}

function Assert-TessaraFullValidationDatabaseEnvironment {
    param(
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [scriptblock]$AuthenticatedProbe
    )

    if ($Environment["SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET"] -ne
        $destructiveFreshResetAcknowledgement) {
        throw "Full validation requires SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET=$destructiveFreshResetAcknowledgement because the fresh-baseline proof destroys and recreates its dedicated database."
    }

    $endpoints = @(
        foreach ($environmentName in $fullValidationDatabaseEnvironmentNames) {
            $value = $Environment[$environmentName]
            if ($value -isnot [string] -or [string]::IsNullOrWhiteSpace($value)) {
                throw "Full validation requires $environmentName so its database integration tests cannot silently skip."
            }
            ConvertFrom-TessaraValidationDatabaseUrl `
                -EnvironmentName $environmentName `
                -DatabaseUrl $value
        }
    )

    $duplicateIdentity = @(
        $endpoints |
            Group-Object Identity |
            Where-Object Count -gt 1
    ) | Select-Object -First 1
    if ($null -ne $duplicateIdentity) {
        $environmentNames = @(
            $duplicateIdentity.Group |
                ForEach-Object EnvironmentName |
                Sort-Object
        )
        throw "Full validation database URLs must resolve to pairwise-distinct host/port/database identities; duplicate: $($environmentNames -join ', ')."
    }
    $duplicateRoleIdentity = @(
        $endpoints | Group-Object RoleIdentity | Where-Object Count -gt 1
    ) | Select-Object -First 1
    if ($null -ne $duplicateRoleIdentity) {
        $environmentNames = @(
            $duplicateRoleIdentity.Group |
                ForEach-Object EnvironmentName |
                Sort-Object
        )
        throw "Full validation database URLs must use pairwise-distinct host/port/role identities; duplicate: $($environmentNames -join ', ')."
    }
    if (@($endpoints | Group-Object Password | Where-Object Count -gt 1).Count -ne 0) {
        throw "Full validation database URLs must use seven pairwise-distinct passwords; password values were not emitted."
    }

    $probe = if ($null -eq $AuthenticatedProbe) {
        ${function:Get-TessaraAuthenticatedDatabaseIdentity}
    } else { $AuthenticatedProbe }
    $authenticated = @($endpoints | ForEach-Object {
        $probeResult = & $probe $_
        $propertyNames = if ($null -eq $probeResult) {
            @()
        } else { @($probeResult.PSObject.Properties.Name) }
        if ($null -eq $probeResult -or
            "database_identity" -notin $propertyNames -or
            "server_role_identity" -notin $propertyNames -or
            [string]::IsNullOrWhiteSpace([string]$probeResult.database_identity) -or
            [string]::IsNullOrWhiteSpace([string]$probeResult.server_role_identity)) {
            throw "Authenticated database probe returned an invalid nonsecret database/role identity for $($_.EnvironmentName)."
        }
        [pscustomobject][ordered]@{
            endpoint = $_
            database_identity = [string]$probeResult.database_identity
            server_role_identity = [string]$probeResult.server_role_identity
        }
    })
    $duplicateAuthenticatedIdentity = @(
        $authenticated | Group-Object database_identity | Where-Object Count -gt 1
    ) | Select-Object -First 1
    if ($null -ne $duplicateAuthenticatedIdentity) {
        $names = @($duplicateAuthenticatedIdentity.Group.endpoint.EnvironmentName | Sort-Object)
        throw "Full validation database URLs resolve to one authenticated physical database: $($names -join ', ')."
    }
    $duplicateAuthenticatedRole = @(
        $authenticated | Group-Object server_role_identity | Where-Object Count -gt 1
    ) | Select-Object -First 1
    if ($null -ne $duplicateAuthenticatedRole) {
        $names = @($duplicateAuthenticatedRole.Group.endpoint.EnvironmentName | Sort-Object)
        throw "Full validation database URLs reuse one authenticated server role: $($names -join ', ')."
    }
    if (@($authenticated | Select-Object -ExpandProperty database_identity -Unique).Count -ne
            $fullValidationDatabaseEnvironmentNames.Count -or
        @($authenticated | Select-Object -ExpandProperty server_role_identity -Unique).Count -ne
            $fullValidationDatabaseEnvironmentNames.Count) {
        throw "Full validation requires seven distinct authenticated database and server-role identities."
    }

    return $endpoints
}

function Restore-TessaraProcessEnvironmentValue {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][bool]$WasPresent,
        [AllowNull()][string]$Value
    )

    if ($WasPresent) {
        [Environment]::SetEnvironmentVariable(
            $Name,
            $Value,
            [EnvironmentVariableTarget]::Process
        )
    } else {
        if (Test-Path -LiteralPath "Env:$Name") {
            Remove-Item -LiteralPath "Env:$Name" -ErrorAction Stop
        }
    }

    $processEnvironment = [Environment]::GetEnvironmentVariables(
        [EnvironmentVariableTarget]::Process
    )
    if ($WasPresent) {
        if (-not $processEnvironment.Contains($Name) -or
            -not [string]::Equals(
                [string]$processEnvironment[$Name],
                [string]$Value,
                [StringComparison]::Ordinal
            )) {
            throw "Failed to restore the exact caller value of process environment variable '$Name'."
        }
    } elseif ($processEnvironment.Contains($Name)) {
        throw "Failed to restore process environment variable '$Name' to its absent caller state."
    }
}

function Get-TessaraProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Names)

    $processEnvironment = [Environment]::GetEnvironmentVariables(
        [EnvironmentVariableTarget]::Process
    )
    $snapshot = [ordered]@{}
    foreach ($name in @($Names | Sort-Object -Unique)) {
        $snapshot[$name] = [pscustomobject][ordered]@{
            present = [bool]$processEnvironment.Contains($name)
            value = if ($processEnvironment.Contains($name)) {
                [string]$processEnvironment[$name]
            } else { $null }
        }
    }
    return $snapshot
}

function Get-TessaraPostgresProcessEnvironmentNames {
    $processEnvironment = [Environment]::GetEnvironmentVariables(
        [EnvironmentVariableTarget]::Process
    )
    return @(
        $processEnvironment.Keys |
            ForEach-Object { [string]$_ } |
            Where-Object { $_ -match '^(?i:PG)' } |
            Sort-Object -Unique
    )
}

function Clear-TessaraProcessEnvironmentValues {
    param([Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Names)

    $failures = [Collections.Generic.List[Exception]]::new()
    foreach ($name in @($Names | Sort-Object -Unique)) {
        try {
            Restore-TessaraProcessEnvironmentValue `
                -Name $name -WasPresent $false -Value $null
        } catch {
            $failures.Add($_.Exception)
        }
    }
    if ($failures.Count -ne 0) {
        throw [AggregateException]::new(
            "One or more process environment variables could not be cleared.",
            $failures.ToArray()
        )
    }
}

function Restore-TessaraProcessEnvironmentSnapshot {
    param(
        [Parameter(Mandatory)][Collections.IDictionary]$Snapshot,
        [string]$IncludeCurrentNamePattern
    )

    $failures = [Collections.Generic.List[Exception]]::new()
    $namesToClear = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($name in $Snapshot.Keys) {
        [void]$namesToClear.Add([string]$name)
    }
    if (-not [string]::IsNullOrWhiteSpace($IncludeCurrentNamePattern)) {
        $processEnvironment = [Environment]::GetEnvironmentVariables(
            [EnvironmentVariableTarget]::Process
        )
        foreach ($name in @($processEnvironment.Keys | ForEach-Object { [string]$_ })) {
            if ($name -match $IncludeCurrentNamePattern) {
                [void]$namesToClear.Add($name)
            }
        }
    }

    foreach ($name in @($namesToClear | Sort-Object)) {
        try {
            Restore-TessaraProcessEnvironmentValue `
                -Name $name -WasPresent $false -Value $null
        } catch {
            $failures.Add($_.Exception)
        }
    }
    foreach ($name in @($Snapshot.Keys | ForEach-Object { [string]$_ } | Sort-Object)) {
        try {
            Restore-TessaraProcessEnvironmentValue `
                -Name $name `
                -WasPresent ([bool]$Snapshot[$name].present) `
                -Value ([string]$Snapshot[$name].value)
        } catch {
            $failures.Add($_.Exception)
        }
    }
    if ($failures.Count -ne 0) {
        throw [AggregateException]::new(
            "One or more process environment variables could not be restored.",
            $failures.ToArray()
        )
    }
}

function Throw-TessaraPrimaryOrCleanupFailure {
    param(
        [AllowNull()]$PrimaryFailure,
        [Parameter(Mandatory)][AllowEmptyCollection()]
        [Collections.Generic.List[Exception]]$CleanupFailures,
        [Parameter(Mandatory)][string]$Boundary
    )

    if ($null -eq $PrimaryFailure -and $CleanupFailures.Count -eq 0) {
        return
    }
    if ($null -ne $PrimaryFailure -and $CleanupFailures.Count -eq 0) {
        throw $PrimaryFailure
    }

    $failures = [Collections.Generic.List[Exception]]::new()
    if ($null -ne $PrimaryFailure) {
        $primaryException = if ($PrimaryFailure -is [Management.Automation.ErrorRecord]) {
            $PrimaryFailure.Exception
        } elseif ($PrimaryFailure -is [Exception]) {
            $PrimaryFailure
        } else {
            [InvalidOperationException]::new([string]$PrimaryFailure)
        }
        $failures.Add($primaryException)
    }
    foreach ($cleanupFailure in $CleanupFailures) {
        $failures.Add($cleanupFailure)
    }
    $aggregateMessage = if ($null -eq $PrimaryFailure) {
        "$Boundary cleanup failed."
    } else {
        "$Boundary failed and one or more required cleanup operations also failed."
    }
    $aggregate = [AggregateException]::new(
        $aggregateMessage,
        $failures.ToArray()
    )
    if ($null -ne $PrimaryFailure) {
        $aggregate.Data['TessaraPrimaryFailure'] = [string]$PrimaryFailure
    }
    throw $aggregate
}

function Set-TessaraProcessEnvironmentBindings {
    param([Parameter(Mandatory)][Collections.IDictionary]$Bindings)

    foreach ($name in $Bindings.Keys) {
        $value = [string]$Bindings[$name]
        [Environment]::SetEnvironmentVariable(
            [string]$name,
            $value,
            [EnvironmentVariableTarget]::Process
        )
        $actual = [Environment]::GetEnvironmentVariable(
            [string]$name,
            [EnvironmentVariableTarget]::Process
        )
        if (-not [string]::Equals($actual, $value, [StringComparison]::Ordinal)) {
            throw "Failed to establish the exact process environment binding '$name'."
        }
    }
}

function Invoke-TessaraWithProcessEnvironmentBindings {
    param(
        [Parameter(Mandatory)][Collections.IDictionary]$Bindings,
        [Parameter(Mandatory)][scriptblock]$Action
    )

    $snapshot = Get-TessaraProcessEnvironmentSnapshot -Names @($Bindings.Keys)
    $primaryFailure = $null
    $cleanupFailures = [Collections.Generic.List[Exception]]::new()
    try {
        Set-TessaraProcessEnvironmentBindings -Bindings $Bindings
        & $Action
    } catch {
        $primaryFailure = $_
    } finally {
        try {
            Restore-TessaraProcessEnvironmentSnapshot -Snapshot $snapshot
        } catch {
            $cleanupFailures.Add($_.Exception)
        }
    }
    Throw-TessaraPrimaryOrCleanupFailure `
        -PrimaryFailure $primaryFailure `
        -CleanupFailures $cleanupFailures `
        -Boundary "Scoped process environment binding"
}

function Get-TessaraPsqlExecutableVersion {
    param(
        [Parameter(Mandatory)][string]$Path,
        [scriptblock]$VersionInvoker
    )

    try {
        $versionOutput = if ($null -eq $VersionInvoker) {
            $lines = @(& $Path --version 2>&1 | ForEach-Object { [string]$_ })
            if ($LASTEXITCODE -ne 0) {
                throw "psql --version failed with exit code $LASTEXITCODE."
            }
            $lines
        } else {
            @(& $VersionInvoker -Path $Path | ForEach-Object { [string]$_ })
        }
    } catch {
        throw [InvalidOperationException]::new(
            "Validation could not authenticate the resolved psql executable version: $($_.Exception.Message)",
            $_.Exception
        )
    }
    $version = $versionOutput -join "`n"
    if ([string]::IsNullOrWhiteSpace($version)) {
        throw "Validation requires the resolved psql executable to report a nonblank version."
    }
    return $version
}

function Get-TessaraPsqlExecutableBinding {
    param(
        [string]$ExecutablePath,
        [scriptblock]$VersionInvoker
    )

    if ([string]::IsNullOrWhiteSpace($ExecutablePath)) {
        $commands = @(Get-Command psql -CommandType Application -All `
            -ErrorAction SilentlyContinue)
        if ($commands.Count -eq 0 -or
            [string]::IsNullOrWhiteSpace([string]$commands[0].Source)) {
            throw "Full-validation database identity preflight requires one directly executable psql program."
        }
        $ExecutablePath = [string]$commands[0].Source
    }
    $path = [IO.Path]::GetFullPath($ExecutablePath)
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Full-validation database identity preflight requires psql to be one ordinary executable file."
    }
    $entry = Get-Item -Force -LiteralPath $path
    if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Full-validation database identity preflight requires psql to be one ordinary executable file."
    }
    [pscustomobject][ordered]@{
        path = $path
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
        version = Get-TessaraPsqlExecutableVersion -Path $path `
            -VersionInvoker $VersionInvoker
    }
}

function Assert-TessaraPsqlExecutableBindingCurrent {
    param(
        [Parameter(Mandatory)]$Binding,
        [scriptblock]$VersionInvoker
    )

    $path = [IO.Path]::GetFullPath([string]$Binding.path)
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "The resolved psql executable changed after validation preflight."
    }
    $entry = Get-Item -Force -LiteralPath $path
    if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant() -cne
            [string]$Binding.sha256) {
        throw "The resolved psql executable changed after validation preflight."
    }
    $version = Get-TessaraPsqlExecutableVersion -Path $path `
        -VersionInvoker $VersionInvoker
    if ($version -cne [string]$Binding.version) {
        throw "The resolved psql executable version changed after validation preflight."
    }
    return $path
}

function Get-TessaraAuthenticatedDatabaseIdentity {
    param(
        [Parameter(Mandatory)]$Endpoint,
        [scriptblock]$PsqlInvoker
    )
    if ($null -eq $PsqlInvoker -and
        $null -eq $script:ActiveValidationPsqlBinding) {
        throw "Full-validation database identity preflight has no authenticated psql executable binding."
    }
    $names = @(
        "PGHOST",
        "PGPORT",
        "PGDATABASE",
        "PGUSER",
        "PGPASSWORD",
        "PGSSLMODE",
        "PGCONNECT_TIMEOUT",
        "PGAPPNAME"
    )
    $postgresNames = @($names + (Get-TessaraPostgresProcessEnvironmentNames) | Sort-Object -Unique)
    $snapshot = Get-TessaraProcessEnvironmentSnapshot -Names $postgresNames
    $primaryFailure = $null
    $cleanupFailures = [Collections.Generic.List[Exception]]::new()
    $identity = $null
    try {
        Clear-TessaraProcessEnvironmentValues -Names $postgresNames
        $values = [ordered]@{
            PGHOST = [string]$Endpoint.Host
            PGPORT = [string]$Endpoint.Port
            PGDATABASE = [string]$Endpoint.DatabaseName
            PGUSER = [string]$Endpoint.Role
            PGPASSWORD = [string]$Endpoint.Password
            PGSSLMODE = [string]$Endpoint.SslMode
            PGCONNECT_TIMEOUT = "10"
            PGAPPNAME = "tessara-validation-preflight"
        }
        foreach ($name in $names) {
            if ([string]::IsNullOrWhiteSpace([string]$values[$name])) {
                Restore-TessaraProcessEnvironmentValue `
                    -Name $name -WasPresent $false -Value $null
            } else {
                Set-TessaraProcessEnvironmentBindings -Bindings ([ordered]@{
                    $name = [string]$values[$name]
                })
            }
        }
        $query = @"
SELECT
  COALESCE(inet_server_addr()::text, 'local'),
  inet_server_port()::text,
  current_database(),
  current_user,
  pg_get_userbyid(database_entry.datdba),
  CASE WHEN role_entry.rolsuper THEN 't' ELSE 'f' END,
  CASE WHEN role_entry.rolcreatedb THEN 't' ELSE 'f' END,
  CASE WHEN role_entry.rolcreaterole THEN 't' ELSE 'f' END,
  CASE WHEN role_entry.rolreplication THEN 't' ELSE 'f' END,
  CASE WHEN role_entry.rolbypassrls THEN 't' ELSE 'f' END,
  (SELECT count(*)
     FROM pg_auth_members
    WHERE member = role_entry.oid OR roleid = role_entry.oid)::text,
  (SELECT count(*)
     FROM pg_database AS other_database
    WHERE other_database.datdba = role_entry.oid
      AND other_database.oid <> database_entry.oid
      AND NOT other_database.datistemplate)::text,
  (SELECT count(*)
     FROM pg_database AS other_database
    WHERE other_database.oid <> database_entry.oid
      AND NOT other_database.datistemplate
      AND (
        has_database_privilege(current_user, other_database.oid, 'CONNECT')
        OR has_database_privilege(current_user, other_database.oid, 'CREATE')
        OR has_database_privilege(current_user, other_database.oid, 'TEMP')
      ))::text,
  CASE WHEN has_database_privilege(current_user, current_database(), 'CONNECT') THEN 't' ELSE 'f' END,
  CASE WHEN has_database_privilege(current_user, current_database(), 'CREATE') THEN 't' ELSE 'f' END,
  CASE WHEN has_database_privilege(current_user, current_database(), 'TEMP') THEN 't' ELSE 'f' END,
  CASE WHEN has_schema_privilege(current_user, 'public', 'CREATE') THEN 't' ELSE 'f' END,
  COALESCE(shobj_description(database_entry.oid, 'pg_database'), '')
FROM pg_database AS database_entry
JOIN pg_roles AS role_entry ON role_entry.rolname = current_user
WHERE database_entry.datname = current_database();
"@
        $psqlArguments = @(
            "-X", "-w", "-v", "ON_ERROR_STOP=1", "-At", "-F", "`t", "-c", $query
        )
        try {
            if ($null -eq $PsqlInvoker) {
                $psqlPath = Assert-TessaraPsqlExecutableBindingCurrent `
                    -Binding $script:ActiveValidationPsqlBinding
                $output = @(& $psqlPath @psqlArguments 2>&1 |
                    ForEach-Object { [string]$_ })
            } else {
                $output = @(& $PsqlInvoker $psqlArguments |
                    ForEach-Object { [string]$_ })
            }
        } catch {
            throw [InvalidOperationException]::new(
                "Authenticated database probe failed for $($Endpoint.EnvironmentName): $($_.Exception.Message)",
                $_.Exception
            )
        }
        if ($null -eq $PsqlInvoker -and $LASTEXITCODE -ne 0) {
            throw "Authenticated database probe failed for $($Endpoint.EnvironmentName)."
        }
        $lines = @($output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        $fields = if ($lines.Count -eq 1) {
            @(([string]$lines[0]).Split([char]"`t"))
        } else { @() }
        if ($fields.Count -ne 18 -or
            [string]::IsNullOrWhiteSpace([string]$fields[0]) -or
            [string]$fields[1] -notmatch '^\d+$') {
            throw "Authenticated database probe returned an invalid identity for $($Endpoint.EnvironmentName)."
        }
        if ([string]$fields[2] -cne [string]$Endpoint.DatabaseName -or
            [string]$fields[3] -cne [string]$Endpoint.Role) {
            throw "Authenticated database probe reached the wrong database or role for $($Endpoint.EnvironmentName)."
        }
        if ([string]$fields[4] -cne [string]$Endpoint.Role) {
            throw "Full validation requires $($Endpoint.EnvironmentName) to be owned by its declared validation role."
        }
        if ([string]$fields[5] -cne "f") {
            throw "Full validation rejects a superuser role for $($Endpoint.EnvironmentName)."
        }
        if (@($fields[7..9] | Where-Object { [string]$_ -cne "f" }).Count -ne 0) {
            throw "Full validation requires NOCREATEROLE, NOREPLICATION, and NOBYPASSRLS for $($Endpoint.EnvironmentName)."
        }
        if ([string]$fields[10] -cne "0") {
            throw "Full validation requires $($Endpoint.EnvironmentName) role to have no role memberships."
        }
        if ([string]$fields[11] -cne "0") {
            throw "Full validation requires $($Endpoint.EnvironmentName) role to own no other non-template database."
        }
        if ([string]$fields[12] -cne "0") {
            throw "Full validation requires $($Endpoint.EnvironmentName) role to have no CONNECT, CREATE, or TEMP capability on any other non-template database."
        }
        if (@($fields[13..16] | Where-Object { [string]$_ -cne "t" }).Count -ne 0) {
            throw "Full validation requires CONNECT, CREATE, TEMP, and public-schema CREATE capabilities for $($Endpoint.EnvironmentName)."
        }
        $expectedCreateDatabase = if (
            [string]$Endpoint.EnvironmentName -ceq "TEST_SQLX_DATABASE_URL"
        ) { "t" } else { "f" }
        if ([string]$fields[6] -cne $expectedCreateDatabase) {
            throw "Full validation requires CREATEDB only for TEST_SQLX_DATABASE_URL and NOCREATEDB for every other validation role."
        }
        if ([string]$fields[17] -cne $validationDatabaseOwnershipMarker) {
            throw "Full validation requires $($Endpoint.EnvironmentName) to carry the authenticated disposable ownership marker '$validationDatabaseOwnershipMarker'."
        }
        $identity = New-TessaraAuthenticatedDatabaseIdentityRecord `
            -ServerAddress ([string]$fields[0]) `
            -ServerPort ([int]$fields[1]) `
            -DatabaseName ([string]$fields[2]) `
            -RoleName ([string]$fields[3])
    } catch {
        $primaryFailure = $_
    } finally {
        try {
            Restore-TessaraProcessEnvironmentSnapshot `
                -Snapshot $snapshot `
                -IncludeCurrentNamePattern '^(?i:PG)'
        } catch {
            $cleanupFailures.Add($_.Exception)
        }
    }
    Throw-TessaraPrimaryOrCleanupFailure `
        -PrimaryFailure $primaryFailure `
        -CleanupFailures $cleanupFailures `
        -Boundary "Authenticated database probe for $($Endpoint.EnvironmentName)"
    return $identity
}

function Invoke-TessaraWithDatabaseUrlBinding {
    param(
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][scriptblock]$Action
    )
    Invoke-TessaraWithProcessEnvironmentBindings `
        -Bindings ([ordered]@{ DATABASE_URL = $DatabaseUrl }) `
        -Action $Action
}

function Assert-TessaraValidationAmbientDatabaseIsolation {
    $databaseUrl = [Environment]::GetEnvironmentVariable(
        "DATABASE_URL",
        [EnvironmentVariableTarget]::Process
    )
    if ($databaseUrl -cne $blockedDatabaseUrl) {
        throw "Validation process DATABASE_URL escaped its fail-closed sentinel."
    }
    $postgresNames = @(Get-TessaraPostgresProcessEnvironmentNames)
    if ($postgresNames.Count -ne 0) {
        throw "Validation process acquired undeclared PG* settings: $($postgresNames -join ', ')."
    }
    $processEnvironment = [Environment]::GetEnvironmentVariables(
        [EnvironmentVariableTarget]::Process
    )
    $leakedInputs = @($fullValidationProcessEnvironmentNames | Where-Object {
        $processEnvironment.Contains([string]$_)
    } | Sort-Object)
    if ($leakedInputs.Count -ne 0) {
        throw "Validation process retained scoped validation inputs outside their owned partition: $($leakedInputs -join ', ')."
    }
}

function Test-TessaraCargoExecutionControlEnvironmentName {
    param([Parameter(Mandatory)][string]$Name)

    $normalizedName = $Name.ToUpperInvariant()
    return [bool](
        $normalizedName -match '^RUSTC.*$' -or
        $normalizedName -match '^RUSTDOC.*$' -or
        $normalizedName -match '^CARGO_PROFILE_.*$' -or
        $normalizedName -in @(
            'RUSTFLAGS',
            'RUSTUP_TOOLCHAIN',
            'RUST_TEST_THREADS',
            'CARGO_ENCODED_RUSTFLAGS',
            'CARGO_TARGET_DIR',
            'CARGO_INCREMENTAL',
            'CARGO_BUILD_TARGET',
            'CARGO_BUILD_PROFILE',
            'CARGO_BUILD_RUSTC',
            'CARGO_BUILD_RUSTC_WRAPPER',
            'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER',
            'CARGO_BUILD_RUSTDOC',
            'CARGO_BUILD_RUSTFLAGS',
            'CARGO_BUILD_RUSTDOCFLAGS'
        ) -or
        $normalizedName -match
            '^CARGO_TARGET_[A-Z0-9_]+_(?:RUNNER|LINKER|RUSTFLAGS|RUSTDOCFLAGS)$'
    )
}

function Get-TessaraEnvironmentEntry {
    param(
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)][string]$Name
    )

    $matches = @($Environment.Keys | Where-Object {
        [string]::Equals([string]$_, $Name, [StringComparison]::OrdinalIgnoreCase)
    })
    if ($matches.Count -gt 1) {
        throw "Validation cannot safely classify duplicate case-variant process environment names for '$Name'."
    }
    if ($matches.Count -eq 0) {
        return [pscustomobject][ordered]@{ present = $false; value = $null }
    }
    return [pscustomobject][ordered]@{
        present = $true
        value = [string]$Environment[$matches[0]]
    }
}

function ConvertFrom-TessaraTomlKeyPath {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $segments = [Collections.Generic.List[string]]::new()
    $index = 0
    while ($index -lt $Text.Length) {
        while ($index -lt $Text.Length -and [char]::IsWhiteSpace($Text[$index])) {
            $index++
        }
        if ($index -ge $Text.Length) {
            break
        }

        $segment = $null
        if ($Text[$index] -eq [char]34 -or $Text[$index] -eq [char]39) {
            $quote = $Text[$index]
            $literal = $quote -eq [char]39
            $index++
            $builder = [Text.StringBuilder]::new()
            $closed = $false
            while ($index -lt $Text.Length) {
                $character = $Text[$index]
                if ($character -eq $quote) {
                    $closed = $true
                    $index++
                    break
                }
                if (-not $literal -and $character -eq [char]92) {
                    $index++
                    if ($index -ge $Text.Length) {
                        break
                    }
                    $escape = $Text[$index]
                    switch -CaseSensitive ([string]$escape) {
                        '"' { [void]$builder.Append([char]34); $index++ }
                        '\' { [void]$builder.Append([char]92); $index++ }
                        'b' { [void]$builder.Append([char]8); $index++ }
                        't' { [void]$builder.Append([char]9); $index++ }
                        'n' { [void]$builder.Append([char]10); $index++ }
                        'f' { [void]$builder.Append([char]12); $index++ }
                        'r' { [void]$builder.Append([char]13); $index++ }
                        'u' {
                            if ($index + 4 -ge $Text.Length) {
                                throw "Cargo configuration contains an incomplete TOML key escape."
                            }
                            $hex = $Text.Substring($index + 1, 4)
                            if ($hex -notmatch '^[0-9A-Fa-f]{4}$') {
                                throw "Cargo configuration contains an invalid TOML key escape."
                            }
                            $codePoint = [Convert]::ToInt32($hex, 16)
                            if ($codePoint -ge 0xD800 -and $codePoint -le 0xDFFF) {
                                throw "Cargo configuration contains an invalid TOML Unicode scalar."
                            }
                            [void]$builder.Append([char]::ConvertFromUtf32($codePoint))
                            $index += 5
                        }
                        'U' {
                            if ($index + 8 -ge $Text.Length) {
                                throw "Cargo configuration contains an incomplete TOML key escape."
                            }
                            $hex = $Text.Substring($index + 1, 8)
                            if ($hex -notmatch '^[0-9A-Fa-f]{8}$') {
                                throw "Cargo configuration contains an invalid TOML key escape."
                            }
                            $codePoint = [Convert]::ToInt32($hex, 16)
                            if ($codePoint -gt 0x10FFFF -or
                                ($codePoint -ge 0xD800 -and $codePoint -le 0xDFFF)) {
                                throw "Cargo configuration contains an invalid TOML Unicode scalar."
                            }
                            [void]$builder.Append([char]::ConvertFromUtf32($codePoint))
                            $index += 9
                        }
                        default {
                            throw "Cargo configuration contains an unsupported TOML key escape."
                        }
                    }
                    continue
                }
                if ([int]$character -lt 0x20 -or [int]$character -eq 0x7F) {
                    throw "Cargo configuration contains a control character in a TOML key."
                }
                [void]$builder.Append($character)
                $index++
            }
            if (-not $closed) {
                throw "Cargo configuration contains an unterminated quoted TOML key."
            }
            $segment = $builder.ToString()
        } else {
            $start = $index
            while ($index -lt $Text.Length -and
                -not [char]::IsWhiteSpace($Text[$index]) -and
                $Text[$index] -ne '.') {
                $index++
            }
            $segment = $Text.Substring($start, $index - $start)
            if ($segment -notmatch '^[A-Za-z0-9_-]+$') {
                throw "Cargo configuration contains a TOML key the validation preflight cannot classify safely."
            }
        }
        $segments.Add([string]$segment)

        while ($index -lt $Text.Length -and [char]::IsWhiteSpace($Text[$index])) {
            $index++
        }
        if ($index -ge $Text.Length) {
            break
        }
        if ($Text[$index] -ne '.') {
            throw "Cargo configuration contains a TOML key the validation preflight cannot classify safely."
        }
        $index++
        if ($index -ge $Text.Length) {
            throw "Cargo configuration contains an incomplete dotted TOML key."
        }
    }
    if ($segments.Count -eq 0) {
        throw "Cargo configuration contains an empty TOML key."
    }
    return @($segments)
}

function Get-TessaraTomlTablePath {
    param([Parameter(Mandatory)][string]$Line)

    $trimmed = $Line.TrimStart()
    $arrayTable = $trimmed.StartsWith('[[')
    $openLength = if ($arrayTable) { 2 } else { 1 }
    $closeToken = if ($arrayTable) { ']]' } else { ']' }
    $index = $openLength
    $quote = [char]0
    $escaped = $false
    while ($index -lt $trimmed.Length) {
        $character = $trimmed[$index]
        if ($quote -ne [char]0) {
            if ($quote -eq [char]34 -and -not $escaped -and $character -eq [char]92) {
                $escaped = $true
                $index++
                continue
            }
            if (-not $escaped -and $character -eq $quote) {
                $quote = [char]0
            }
            $escaped = $false
            $index++
            continue
        }
        if ($character -eq [char]34 -or $character -eq [char]39) {
            $quote = $character
            $index++
            continue
        }
        if ($index + $closeToken.Length -le $trimmed.Length -and
            $trimmed.Substring($index, $closeToken.Length) -ceq $closeToken) {
            $keyText = $trimmed.Substring($openLength, $index - $openLength)
            $trailing = $trimmed.Substring($index + $closeToken.Length).TrimStart()
            if (-not [string]::IsNullOrWhiteSpace($trailing) -and
                -not $trailing.StartsWith('#')) {
                throw "Cargo configuration contains trailing content after a TOML table header."
            }
            return @(ConvertFrom-TessaraTomlKeyPath -Text $keyText)
        }
        if ($character -eq '#') {
            break
        }
        $index++
    }
    throw "Cargo configuration contains a TOML table header the validation preflight cannot classify safely."
}

function Find-TessaraTomlAssignmentEquals {
    param([Parameter(Mandatory)][string]$Line)

    $quote = [char]0
    $escaped = $false
    for ($index = 0; $index -lt $Line.Length; $index++) {
        $character = $Line[$index]
        if ($quote -ne [char]0) {
            if ($quote -eq [char]34 -and -not $escaped -and $character -eq [char]92) {
                $escaped = $true
                continue
            }
            if (-not $escaped -and $character -eq $quote) {
                $quote = [char]0
            }
            $escaped = $false
            continue
        }
        if ($character -eq [char]34 -or $character -eq [char]39) {
            $quote = $character
            continue
        }
        if ($character -eq '#') {
            break
        }
        if ($character -eq '=') {
            return $index
        }
    }
    return -1
}

function Update-TessaraTomlValueState {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)]$State
    )

    $mode = [string]$State.mode
    $squareDepth = [int]$State.square_depth
    $curlyDepth = [int]$State.curly_depth
    $index = 0
    while ($index -lt $Text.Length) {
        if ($mode -eq 'basic-multiline') {
            if ($Text[$index] -eq [char]92) {
                $index += 2
                continue
            }
            if ($index + 3 -le $Text.Length -and
                $Text.Substring($index, 3) -ceq '"""') {
                $mode = 'none'
                $index += 3
                continue
            }
            $index++
            continue
        }
        if ($mode -eq 'literal-multiline') {
            if ($index + 3 -le $Text.Length -and
                $Text.Substring($index, 3) -ceq "'''") {
                $mode = 'none'
                $index += 3
                continue
            }
            $index++
            continue
        }
        if ($mode -eq 'basic') {
            if ($Text[$index] -eq [char]92) {
                $index += 2
                continue
            }
            if ($Text[$index] -eq [char]34) {
                $mode = 'none'
            }
            $index++
            continue
        }
        if ($mode -eq 'literal') {
            if ($Text[$index] -eq [char]39) {
                $mode = 'none'
            }
            $index++
            continue
        }

        if ($Text[$index] -eq '#') {
            break
        }
        if ($index + 3 -le $Text.Length -and
            $Text.Substring($index, 3) -ceq '"""') {
            $mode = 'basic-multiline'
            $index += 3
            continue
        }
        if ($index + 3 -le $Text.Length -and
            $Text.Substring($index, 3) -ceq "'''") {
            $mode = 'literal-multiline'
            $index += 3
            continue
        }
        switch ($Text[$index]) {
            ([char]34) { $mode = 'basic' }
            ([char]39) { $mode = 'literal' }
            '[' { $squareDepth++ }
            ']' {
                $squareDepth--
                if ($squareDepth -lt 0) {
                    throw "Cargo configuration contains an unmatched TOML array delimiter."
                }
            }
            '{' { $curlyDepth++ }
            '}' {
                $curlyDepth--
                if ($curlyDepth -lt 0) {
                    throw "Cargo configuration contains an unmatched TOML inline-table delimiter."
                }
            }
        }
        $index++
    }
    if ($mode -in @('basic', 'literal')) {
        throw "Cargo configuration contains an unterminated single-line TOML string."
    }
    return [pscustomobject][ordered]@{
        mode = $mode
        square_depth = $squareDepth
        curly_depth = $curlyDepth
    }
}

function Get-TessaraCargoConfigKeyPaths {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    if ($Text.Length -ne 0 -and $Text[0] -eq [char]0xFEFF) {
        $Text = $Text.Substring(1)
    }
    $currentTable = @()
    $continuationState = [pscustomobject][ordered]@{
        mode = 'none'
        square_depth = 0
        curly_depth = 0
    }
    $results = [Collections.Generic.List[object]]::new()
    foreach ($line in @($Text -split '\r?\n')) {
        $inContinuation =
            [string]$continuationState.mode -cne 'none' -or
            [int]$continuationState.square_depth -ne 0 -or
            [int]$continuationState.curly_depth -ne 0
        if ($inContinuation) {
            $continuationState = Update-TessaraTomlValueState `
                -Text $line -State $continuationState
            continue
        }

        $trimmed = $line.TrimStart()
        if ([string]::IsNullOrWhiteSpace($trimmed) -or $trimmed.StartsWith('#')) {
            continue
        }
        if ($trimmed.StartsWith('[')) {
            $currentTable = @(Get-TessaraTomlTablePath -Line $line)
            continue
        }

        $equalsIndex = Find-TessaraTomlAssignmentEquals -Line $line
        if ($equalsIndex -lt 0) {
            throw "Cargo configuration contains a statement the validation preflight cannot classify safely."
        }
        $keyPath = @(ConvertFrom-TessaraTomlKeyPath `
            -Text $line.Substring(0, $equalsIndex))
        $effectivePath = @($currentTable + $keyPath)
        $results.Add([pscustomobject][ordered]@{
            segments = $effectivePath
            canonical = ($effectivePath | ForEach-Object {
                ([string]$_).ToLowerInvariant()
            }) -join '.'
        })
        $continuationState = Update-TessaraTomlValueState `
            -Text $line.Substring($equalsIndex + 1) `
            -State $continuationState
    }
    if ([string]$continuationState.mode -cne 'none' -or
        [int]$continuationState.square_depth -ne 0 -or
        [int]$continuationState.curly_depth -ne 0) {
        throw "Cargo configuration ends inside a TOML multiline value the validation preflight cannot classify safely."
    }
    return @($results)
}

function Test-TessaraCargoConfigExecutionControlKey {
    param([Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Segments)

    $normalized = @($Segments | ForEach-Object { $_.ToLowerInvariant() })
    if ($normalized.Count -eq 0) {
        return $true
    }
    # The validation runner does not attempt to reproduce Cargo's evolving
    # configuration semantics. Only settings proven unable to select, wrap,
    # recompile, or reshape test code are accepted; every unknown key fails
    # closed before the next Cargo process starts.
    $canonical = $normalized -join '.'
    return $canonical -notin @(
        'build.jobs',
        'build.target-dir',
        'build.incremental',
        'net.retry'
    )
}

function Get-TessaraEffectiveCargoConfigPaths {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment
    )

    $paths = [Collections.Generic.List[string]]::new()
    $seen = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    $directory = [IO.DirectoryInfo]::new([IO.Path]::GetFullPath($RepositoryRoot))
    while ($null -ne $directory) {
        $cargoDirectory = Join-Path $directory.FullName '.cargo'
        $candidates = @(
            Join-Path $cargoDirectory 'config.toml'
            Join-Path $cargoDirectory 'config'
        )
        foreach ($candidate in $candidates) {
            if ((Test-Path -LiteralPath $candidate) -and
                -not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
                throw "Validation found a non-file Cargo config candidate in an effective search directory."
            }
        }
        $effectiveConfig = if (Test-Path -LiteralPath $candidates[1] -PathType Leaf) {
            $candidates[1]
        } elseif (Test-Path -LiteralPath $candidates[0] -PathType Leaf) {
            $candidates[0]
        } else { $null }
        if ($null -ne $effectiveConfig) {
            $fullPath = [IO.Path]::GetFullPath($effectiveConfig)
            if ($seen.Add($fullPath)) {
                $paths.Add($fullPath)
            }
        }
        $directory = $directory.Parent
    }

    $cargoHomeEntry = Get-TessaraEnvironmentEntry `
        -Environment $Environment -Name 'CARGO_HOME'
    if ([bool]$cargoHomeEntry.present) {
        if ([string]::IsNullOrWhiteSpace([string]$cargoHomeEntry.value)) {
            throw "Validation cannot safely resolve an empty process CARGO_HOME."
        }
        $cargoHome = [string]$cargoHomeEntry.value
        if (-not [IO.Path]::IsPathFullyQualified($cargoHome)) {
            $cargoHome = [IO.Path]::GetFullPath($cargoHome, $RepositoryRoot)
        }
    } else {
        $userProfile = [Environment]::GetFolderPath(
            [Environment+SpecialFolder]::UserProfile
        )
        if ([string]::IsNullOrWhiteSpace($userProfile)) {
            $homeEntry = Get-TessaraEnvironmentEntry `
                -Environment $Environment -Name 'HOME'
            if ([bool]$homeEntry.present) {
                $userProfile = [string]$homeEntry.value
            }
        }
        if ([string]::IsNullOrWhiteSpace($userProfile)) {
            throw "Validation could not resolve the effective user Cargo configuration directory."
        }
        $cargoHome = Join-Path $userProfile '.cargo'
    }

    $userCandidates = @(
        Join-Path $cargoHome 'config.toml'
        Join-Path $cargoHome 'config'
    )
    foreach ($candidate in $userCandidates) {
        if ((Test-Path -LiteralPath $candidate) -and
            -not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
            throw "Validation found a non-file user Cargo config candidate."
        }
    }
    $effectiveUserConfig = if (Test-Path -LiteralPath $userCandidates[1] -PathType Leaf) {
        $userCandidates[1]
    } elseif (Test-Path -LiteralPath $userCandidates[0] -PathType Leaf) {
        $userCandidates[0]
    } else { $null }
    if ($null -ne $effectiveUserConfig) {
        $fullPath = [IO.Path]::GetFullPath($effectiveUserConfig)
        if ($seen.Add($fullPath)) {
            $paths.Add($fullPath)
        }
    }
    return @($paths)
}

function Assert-TessaraCargoExecutionControlPreflight {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Collections.IDictionary]$ProcessEnvironment,
        [AllowEmptyCollection()][string[]]$CargoConfigPaths,
        [Collections.IDictionary]$AllowedExecutionControls
    )

    if ($null -eq $ProcessEnvironment) {
        $ProcessEnvironment = [Environment]::GetEnvironmentVariables(
            [EnvironmentVariableTarget]::Process
        )
    }
    $blockedNames = @($ProcessEnvironment.Keys | ForEach-Object { [string]$_ } |
        Where-Object {
            if (-not (Test-TessaraCargoExecutionControlEnvironmentName -Name $_)) {
                return $false
            }
            if ($null -eq $AllowedExecutionControls) { return $true }
            $allowedEntry = Get-TessaraEnvironmentEntry `
                -Environment $AllowedExecutionControls -Name $_
            if (-not [bool]$allowedEntry.present) { return $true }
            $actualEntry = Get-TessaraEnvironmentEntry `
                -Environment $ProcessEnvironment -Name $_
            return -not (
                [bool]$actualEntry.present -and
                [string]$actualEntry.value -ceq [string]$allowedEntry.value
            )
        } | Sort-Object -Unique)
    if ($blockedNames.Count -ne 0) {
        throw "Validation rejects process Cargo/Rust execution-control overrides before starting Cargo: $($blockedNames -join ', '). Override values were not emitted."
    }
    if ($null -ne $AllowedExecutionControls) {
        $missingOrChangedAllowedNames = @(
            $AllowedExecutionControls.Keys | ForEach-Object { [string]$_ } |
                Where-Object {
                    $actualEntry = Get-TessaraEnvironmentEntry `
                        -Environment $ProcessEnvironment -Name $_
                    -not ([bool]$actualEntry.present -and
                        [string]$actualEntry.value -ceq
                            [string]$AllowedExecutionControls[$_])
                } | Sort-Object -Unique
        )
        if ($missingOrChangedAllowedNames.Count -ne 0) {
            throw "Validation Cargo policy controls are missing or changed: $($missingOrChangedAllowedNames -join ', '). Values were not emitted."
        }
    }

    $configPaths = if ($PSBoundParameters.ContainsKey('CargoConfigPaths')) {
        @($CargoConfigPaths)
    } else {
        @(Get-TessaraEffectiveCargoConfigPaths `
            -RepositoryRoot $RepositoryRoot `
            -Environment $ProcessEnvironment)
    }
    $utf8 = [Text.UTF8Encoding]::new($false, $true)
    foreach ($configPath in @($configPaths | Sort-Object -Unique)) {
        $fullPath = [IO.Path]::GetFullPath($configPath)
        try {
            $text = $utf8.GetString([IO.File]::ReadAllBytes($fullPath))
            $keyPaths = @(Get-TessaraCargoConfigKeyPaths -Text $text)
        } catch {
            throw [InvalidOperationException]::new(
                "Validation could not safely classify effective Cargo config '$fullPath'; no Cargo command was started.",
                $_.Exception
            )
        }
        $blockedKeys = @($keyPaths | Where-Object {
            Test-TessaraCargoConfigExecutionControlKey -Segments @($_.segments)
        } | ForEach-Object { [string]$_.canonical } | Sort-Object -Unique)
        if ($blockedKeys.Count -ne 0) {
            throw "Validation rejects Cargo/Rust execution-control keys in effective Cargo config '$fullPath': $($blockedKeys -join ', '). Configuration values were not emitted."
        }
    }
}

function Get-TessaraCargoExecutableBinding {
    $commands = @(Get-Command cargo -CommandType Application -All -ErrorAction Stop)
    if ($commands.Count -eq 0 -or
        [string]::IsNullOrWhiteSpace([string]$commands[0].Source)) {
        throw "Validation requires one directly executable Cargo program."
    }
    $path = [IO.Path]::GetFullPath([string]$commands[0].Source)
    $entry = Get-Item -Force -LiteralPath $path
    if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
        ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Validation requires Cargo to be one ordinary executable file."
    }
    $rustcCommands = @(Get-Command rustc -CommandType Application -All -ErrorAction Stop)
    if ($rustcCommands.Count -eq 0 -or
        [string]::IsNullOrWhiteSpace([string]$rustcCommands[0].Source)) {
        throw "Validation requires one directly executable Rust compiler program."
    }
    $rustcPath = [IO.Path]::GetFullPath([string]$rustcCommands[0].Source)
    $rustcEntry = Get-Item -Force -LiteralPath $rustcPath
    if (-not (Test-Path -LiteralPath $rustcPath -PathType Leaf) -or
        ($rustcEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Validation requires rustc to be one ordinary executable file."
    }
    $cargoVersion = @(& $path --version --verbose) -join "`n"
    $rustcVersion = @(& $rustcPath --version --verbose) -join "`n"
    [pscustomobject][ordered]@{
        path = $path
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
        version = $cargoVersion
        rustc_path = $rustcPath
        rustc_sha256 = (
            Get-FileHash -Algorithm SHA256 -LiteralPath $rustcPath
        ).Hash.ToLowerInvariant()
        rustc_version = $rustcVersion
    }
}

function Assert-TessaraCargoExecutableBindingCurrent {
    param([Parameter(Mandatory)]$Binding)
    $path = [IO.Path]::GetFullPath([string]$Binding.path)
    $entry = Get-Item -Force -LiteralPath $path
    if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
        ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant() -cne
            [string]$Binding.sha256) {
        throw "The resolved Cargo executable changed after validation preflight."
    }
    $rustcPath = [IO.Path]::GetFullPath([string]$Binding.rustc_path)
    $rustcEntry = Get-Item -Force -LiteralPath $rustcPath
    if (-not (Test-Path -LiteralPath $rustcPath -PathType Leaf) -or
        ($rustcEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $rustcPath).Hash.ToLowerInvariant() -cne
            [string]$Binding.rustc_sha256 -or
        ((@(& $path --version --verbose) -join "`n") -cne [string]$Binding.version) -or
        ((@(& $rustcPath --version --verbose) -join "`n") -cne
            [string]$Binding.rustc_version)) {
        throw "The resolved Cargo/Rust toolchain changed after validation preflight."
    }
    $path
}

function Get-TessaraValidationAllowedCargoExecutionControls {
    $allowed = [ordered]@{}
    if ($null -ne $script:ActiveValidationCargoPolicyState) {
        $policy = Get-TessaraCargoBuildPolicy `
            -Mode ([string]$script:ActiveValidationCargoPolicyState.mode)
        $allowed.CARGO_TARGET_DIR =
            [string]$script:ActiveValidationCargoPolicyState.target_directory
        $allowed.CARGO_INCREMENTAL = if ([bool]$policy.incremental) { '1' } else { '0' }
        $allowed.CARGO_PROFILE_TEST_DEBUG = [string]$policy.test_debug
    } elseif ($SelfTest) {
        # The full gate's authenticated parent Cargo policy deliberately
        # carries its storage/debug settings into the fresh self-test process.
        # Self-test invokes Cargo only for read-only metadata inventory.
        foreach ($name in @(
                'CARGO_TARGET_DIR', 'CARGO_INCREMENTAL',
                'CARGO_PROFILE_TEST_DEBUG'
            )) {
            $value = [Environment]::GetEnvironmentVariable(
                $name, [EnvironmentVariableTarget]::Process
            )
            if ($null -ne $value) { $allowed[$name] = [string]$value }
        }
    } elseif ($Fast) {
        foreach ($name in $script:InitialValidationCargoControlValues.Keys) {
            $allowed[$name] = [string]$script:InitialValidationCargoControlValues[$name]
        }
    }
    $allowed
}

function Invoke-TessaraBoundCargoProcess {
    param(
        [Parameter(Mandatory)][string]$CargoPath,
        [Parameter(Mandatory)][string]$RustcPath,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [scriptblock]$NativeInvoker
    )

    Invoke-TessaraWithProcessEnvironmentBindings `
        -Bindings ([ordered]@{ RUSTC = [IO.Path]::GetFullPath($RustcPath) }) `
        -Action {
            try {
                if ($null -eq $NativeInvoker) {
                    & $CargoPath @Arguments
                    if ($LASTEXITCODE -ne 0) {
                        throw "Cargo command failed with exit code $LASTEXITCODE."
                    }
                } else {
                    & $NativeInvoker -CargoPath $CargoPath -Arguments $Arguments
                }
            } catch {
                throw [InvalidOperationException]::new(
                    "Cargo command failed before the validation boundary completed: $($_.Exception.Message)",
                    $_.Exception
                )
            }
        }
}

function Invoke-TessaraCargo {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [Collections.IDictionary]$ProcessEnvironment,
        [AllowEmptyCollection()][string[]]$CargoConfigPaths,
        [Collections.IDictionary]$AllowedExecutionControls,
        [scriptblock]$CargoInvoker
    )
    $preflightArguments = @{
        RepositoryRoot = $repoRoot
        ProcessEnvironment = $ProcessEnvironment
    }
    if ($PSBoundParameters.ContainsKey('AllowedExecutionControls')) {
        $preflightArguments.AllowedExecutionControls = $AllowedExecutionControls
    } elseif ($null -eq $ProcessEnvironment) {
        $preflightArguments.AllowedExecutionControls =
            Get-TessaraValidationAllowedCargoExecutionControls
    }
    if ($PSBoundParameters.ContainsKey('CargoConfigPaths')) {
        $preflightArguments.CargoConfigPaths = @($CargoConfigPaths)
    }
    Assert-TessaraCargoExecutionControlPreflight @preflightArguments

    if ($null -ne $CargoInvoker) {
        return & $CargoInvoker -Arguments $Arguments
    }
    if ($null -eq $script:ActiveValidationCargoBinding) {
        throw "Validation has no authenticated Cargo executable binding."
    }
    $cargoPath = Assert-TessaraCargoExecutableBindingCurrent `
        -Binding $script:ActiveValidationCargoBinding
    Invoke-TessaraBoundCargoProcess `
        -CargoPath $cargoPath `
        -RustcPath ([string]$script:ActiveValidationCargoBinding.rustc_path) `
        -Arguments $Arguments
}

function Get-TessaraFullStaticCargoCommandPlan {
    @(
        [pscustomobject][ordered]@{
            label = "Workspace check"
            arguments = @("check", "--workspace", "--all-features", "--locked")
        },
        [pscustomobject][ordered]@{
            label = "Clippy (warnings denied)"
            arguments = @(
                "clippy", "--workspace", "--all-targets", "--all-features",
                "--locked", "--", "-D", "warnings"
            )
        }
    )
}

function Assert-TessaraExactApiProofSourceInventory {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [scriptblock]$SourceReader
    )
    $proofs = @(
        [pscustomobject]@{
            path = 'crates/tessara-api/src/core_security.rs'
            name = $fullValidationApiEnrollmentTest
            function_name = 'local_enrollment_is_atomic_global_and_idempotent'
            attribute_patterns = @([regex]::Escape('#[tokio::test]'))
            required_body_text = 'TEST_API_ENROLLMENT_DATABASE_URL'
        },
        [pscustomobject]@{
            path = 'crates/tessara-api/src/composition/mod.rs'
            name = $fullValidationApiSqlxTest
            function_name = 'fresh_baseline_enrolls_dataset_security_before_core_actor_bootstrap'
            attribute_patterns = @(
                [regex]::Escape('#[sqlx::test(migrations = "./migrations")]')
            )
            required_body_text = 'dataset_bootstrap_manifest_fixture()'
        },
        [pscustomobject]@{
            path = 'crates/tessara-api/tests/modules.rs'
            name = $fullValidationApiReleaseTimingTest
            function_name = 'resource_reference_restricted_known_random_latency_profile'
            attribute_patterns = @(
                [regex]::Escape('#[cfg(not(debug_assertions))]'),
                [regex]::Escape('#[tokio::test]')
            )
            required_body_text = 'test_state().await'
        },
        [pscustomobject]@{
            path = 'crates/tessara-api/tests/sprint_6a_populated_upgrade.rs'
            name = $fullValidationApiFreshTest
            function_name = 'fresh_startup_and_seed_assignment_lock_order_use_a_separate_database'
            attribute_patterns = @([regex]::Escape('#[tokio::test]'))
            required_body_text = 'assert_destructive_fresh_reset_acknowledged()'
        }
    )
    foreach ($proof in $proofs) {
        $path = Join-Path $RepositoryRoot ([string]$proof.path)
        $text = if ($null -eq $SourceReader) {
            Get-Content -Raw -LiteralPath $path
        } else {
            [string](& $SourceReader ([string]$proof.path))
        }
        $escapedName = [regex]::Escape([string]$proof.function_name)
        $definitionPattern = "(?m)^\s*async\s+fn\s+$escapedName\s*\("
        if ([regex]::Matches($text, $definitionPattern).Count -ne 1) {
            throw "Full validation requires exactly one source definition for exact API proof '$($proof.name)'."
        }
        $attributePattern = @($proof.attribute_patterns) -join '\s*'
        $shapePattern = "(?ms)$attributePattern\s*async\s+fn\s+$escapedName\s*\([^)]*\)\s*\{.{0,2048}?" +
            [regex]::Escape([string]$proof.required_body_text)
        if (-not [regex]::IsMatch($text, $shapePattern)) {
            throw "Exact API proof '$($proof.name)' lost its required attribute or owned input shape."
        }
    }
}

function Assert-TessaraExactCargoTestList {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Output,
        [Parameter(Mandatory)][string]$ExpectedTestName
    )
    $listedTests = @($Output | Where-Object { $_ -cmatch '^(.+): test$' } |
        ForEach-Object { [string]$Matches[1] })
    if ($listedTests.Count -ne 1 -or
        [string]$listedTests[0] -cne $ExpectedTestName) {
        throw "Cargo exact-test selection for '$ExpectedTestName' did not resolve to exactly one test."
    }
}

function Assert-TessaraExactCargoTestSummary {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Output,
        [Parameter(Mandatory)][string]$ExpectedTestName
    )
    $summaries = @($Output | Where-Object {
        $_ -cmatch '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out; finished in .+$'
    })
    if ($summaries.Count -ne 1) {
        throw "Cargo exact-test execution for '$ExpectedTestName' did not report exactly one passing assertion."
    }
}

function Invoke-TessaraExactCargoTestProof {
    param(
        [Parameter(Mandatory)][string[]]$CargoArguments,
        [Parameter(Mandatory)][string]$ExpectedTestName,
        [switch]$NoCapture
    )
    $listArguments = @($CargoArguments + @($ExpectedTestName, '--', '--exact', '--list'))
    $listOutput = @(Invoke-TessaraCargo -Arguments $listArguments 2>&1 |
        ForEach-Object { [string]$_ })
    Assert-TessaraExactCargoTestList -Output $listOutput `
        -ExpectedTestName $ExpectedTestName

    $runArguments = @($CargoArguments + @($ExpectedTestName, '--', '--exact'))
    if ($NoCapture) { $runArguments += '--nocapture' }
    $runOutput = @(Invoke-TessaraCargo -Arguments $runArguments 2>&1 |
        ForEach-Object { [string]$_ })
    foreach ($line in $runOutput) { Write-Host $line }
    Assert-TessaraExactCargoTestSummary -Output $runOutput `
        -ExpectedTestName $ExpectedTestName
}

function Assert-TessaraCargoSkipFilterList {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ListedTests,
        [Parameter(Mandatory)][string[]]$SkipTests
    )
    foreach ($skipTest in $SkipTests) {
        $exactMatches = @($ListedTests | Where-Object { $_ -ceq $skipTest })
        $substringMatches = @($ListedTests | Where-Object {
            $_.Contains($skipTest, [StringComparison]::Ordinal)
        })
        if ($exactMatches.Count -ne 1 -or $substringMatches.Count -ne 1) {
            throw "Cargo skip selector '$skipTest' is not an exact one-test filter."
        }
    }
}

function Assert-TessaraCargoSkipFiltersExact {
    param(
        [Parameter(Mandatory)][string[]]$CargoArguments,
        [Parameter(Mandatory)][string[]]$SkipTests
    )
    $listOutput = @(Invoke-TessaraCargo `
        -Arguments @($CargoArguments + @('--', '--list')) 2>&1 |
        ForEach-Object { [string]$_ })
    $listedTests = @($listOutput | Where-Object { $_ -cmatch '^(.+): test$' } |
        ForEach-Object { [string]$Matches[1] })
    Assert-TessaraCargoSkipFilterList -ListedTests $listedTests -SkipTests $SkipTests
}

function Get-TessaraCargoWorkspaceMetadata {
    param(
        [Collections.IDictionary]$ProcessEnvironment,
        [AllowEmptyCollection()][string[]]$CargoConfigPaths,
        [scriptblock]$CargoInvoker
    )

    try {
        $invokeArguments = @{
            Arguments = @('metadata', '--locked', '--format-version', '1', '--no-deps')
            ProcessEnvironment = $ProcessEnvironment
            CargoInvoker = $CargoInvoker
        }
        if ($PSBoundParameters.ContainsKey('CargoConfigPaths')) {
            $invokeArguments.CargoConfigPaths = @($CargoConfigPaths)
        }
        $metadataText = @(Invoke-TessaraCargo @invokeArguments)
    } catch {
        throw [InvalidOperationException]::new(
            "Locked Cargo workspace metadata failed: $($_.Exception.Message)",
            $_.Exception
        )
    }
    try {
        return (($metadataText -join "`n") | ConvertFrom-Json -Depth 100)
    } catch {
        throw [InvalidOperationException]::new(
            "Locked Cargo workspace metadata returned invalid JSON.",
            $_.Exception
        )
    }
}

function Get-TessaraDotEnvPostgresDeclarationNames {
    param([Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Lines)

    $names = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($line in $Lines) {
        if ($line -match '^\s*(?:export\s+)?(?<name>[A-Za-z_][A-Za-z0-9_]*)\s*=') {
            $name = [string]$Matches.name
            if ($name -match '^(?i:PG)' -or
                $name -in $fullValidationProcessEnvironmentNames) {
                [void]$names.Add($name)
            }
        }
    }
    return @($names | Sort-Object)
}

function Assert-TessaraWorkspaceDotEnvPostgresIsolation {
    param([Parameter(Mandatory)]$Metadata)

    $directories = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    [void]$directories.Add([IO.Path]::GetFullPath($repoRoot))
    foreach ($package in @($Metadata.packages)) {
        $manifestDirectory = Split-Path -Parent ([string]$package.manifest_path)
        $currentDirectory = [IO.Path]::GetFullPath($manifestDirectory)
        while ($currentDirectory.StartsWith(
                [IO.Path]::GetFullPath($repoRoot),
                [StringComparison]::OrdinalIgnoreCase
            )) {
            [void]$directories.Add($currentDirectory)
            if ([string]::Equals(
                    $currentDirectory,
                    [IO.Path]::GetFullPath($repoRoot),
                    [StringComparison]::OrdinalIgnoreCase
                )) {
                break
            }
            $parent = Split-Path -Parent $currentDirectory
            if ([string]::IsNullOrWhiteSpace($parent) -or
                [string]::Equals($parent, $currentDirectory, [StringComparison]::OrdinalIgnoreCase)) {
                break
            }
            $currentDirectory = [IO.Path]::GetFullPath($parent)
        }
    }

    $violations = [Collections.Generic.List[string]]::new()
    foreach ($directory in @($directories | Sort-Object)) {
        $dotEnvPath = Join-Path $directory ".env"
        if (-not (Test-Path -LiteralPath $dotEnvPath -PathType Leaf)) {
            continue
        }
        $names = @(Get-TessaraDotEnvPostgresDeclarationNames `
            -Lines @(Get-Content -LiteralPath $dotEnvPath))
        if ($names.Count -ne 0) {
            $relativePath = [IO.Path]::GetRelativePath($repoRoot, $dotEnvPath).Replace('\', '/')
            $violations.Add("$relativePath ($($names -join ', '))")
        }
    }
    if ($violations.Count -ne 0) {
        throw "Validation rejects PG* and reserved validation-input declarations in workspace .env search paths because dotenv loading would bypass the authenticated process environment: $($violations -join '; ')."
    }
}

function Get-TessaraDatasetCargoTestPlan {
    param(
        [Parameter(Mandatory)]$Metadata,
        [Parameter(Mandatory)][object[]]$SqlxMatches
    )

    $datasetPackages = @($Metadata.packages | Where-Object {
        [string]$_.name -ceq "tessara-dataset-module"
    })
    if ($datasetPackages.Count -ne 1) {
        throw "Validation requires exactly one tessara-dataset-module Cargo package."
    }
    $datasetPackage = $datasetPackages[0]
    $testTargets = @($datasetPackage.targets | Where-Object {
        @($_.kind | ForEach-Object { [string]$_ }) -contains "test"
    } | ForEach-Object {
        [pscustomobject][ordered]@{
            name = [string]$_.name
            source_path = [IO.Path]::GetFullPath([string]$_.src_path)
        }
    } | Sort-Object name)
    if ($testTargets.Count -eq 0 -or
        @($testTargets | Group-Object name | Where-Object Count -ne 1).Count -ne 0 -or
        @($testTargets | Group-Object source_path | Where-Object Count -ne 1).Count -ne 0) {
        throw "Dataset Cargo integration targets must have unique names and root source paths."
    }

    $datasetSqlxMatches = @($SqlxMatches | Where-Object {
        [IO.Path]::GetRelativePath($repoRoot, $_.Path).Replace('\', '/') -like
            'crates/tessara-dataset-module/*'
    })
    if ($datasetSqlxMatches.Count -eq 0) {
        throw "Validation did not find the expected Dataset SQLx integration inventory."
    }
    $sqlxTargetNames = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::Ordinal
    )
    foreach ($match in $datasetSqlxMatches) {
        $matchPath = [IO.Path]::GetFullPath([string]$match.Path)
        $owners = @($testTargets | Where-Object {
            [string]::Equals(
                [string]$_.source_path,
                $matchPath,
                [StringComparison]::OrdinalIgnoreCase
            )
        })
        if ($owners.Count -ne 1) {
            $relativePath = [IO.Path]::GetRelativePath($repoRoot, $matchPath).Replace('\', '/')
            throw "Dataset SQLx source '$relativePath' is not the unique root source of one Cargo integration target; helper-source ownership must be made explicit before validation can classify it."
        }
        [void]$sqlxTargetNames.Add([string]$owners[0].name)
    }

    $kindSets = @($datasetPackage.targets | ForEach-Object {
        [pscustomobject]@{
            target = $_
            kinds = @($_.kind | ForEach-Object { [string]$_ })
        }
    })
    return [pscustomobject][ordered]@{
        test_targets = $testTargets
        sqlx_target_names = @($sqlxTargetNames | Sort-Object)
        independent_target_names = @($testTargets.name | Where-Object {
            [string]$_ -notin $sqlxTargetNames
        } | Sort-Object)
        has_lib_target = [bool](@($kindSets | Where-Object {
            $_.kinds -contains "lib" -and [bool]$_.target.test
        }).Count -ne 0)
        has_doctest_target = [bool](@($kindSets | Where-Object {
            [bool]$_.target.doctest
        }).Count -ne 0)
        has_bin_targets = [bool](@($kindSets | Where-Object {
            $_.kinds -contains "bin" -and [bool]$_.target.test
        }).Count -ne 0)
        has_example_targets = [bool](@($kindSets | Where-Object {
            $_.kinds -contains "example" -and [bool]$_.target.test
        }).Count -ne 0)
        has_bench_targets = [bool](@($kindSets | Where-Object {
            $_.kinds -contains "bench" -and [bool]$_.target.test
        }).Count -ne 0)
    }
}

function Assert-TessaraSqlxTestInventory {
    param([Parameter(Mandatory)]$Metadata)

    $sqlxMatches = @(Get-ChildItem -LiteralPath (Join-Path $repoRoot "crates") `
        -Filter "*.rs" -File -Recurse | Select-String -Pattern '#\[sqlx::test')
    $apiLibraryMatches = @($sqlxMatches | Where-Object {
        [IO.Path]::GetRelativePath($repoRoot, $_.Path).Replace('\', '/') -like
            'crates/tessara-api/src/*'
    })
    if ($apiLibraryMatches.Count -ne 1 -or
        [IO.Path]::GetRelativePath($repoRoot, $apiLibraryMatches[0].Path).Replace('\', '/') -cne
            'crates/tessara-api/src/composition/mod.rs') {
        throw "Fast validation SQLx exclusions no longer exactly cover the API library SQLx inventory."
    }
    $compositionSource = Get-Content -Raw -LiteralPath $apiLibraryMatches[0].Path
    if ($compositionSource -notmatch '(?s)#\[sqlx::test[^\]]*\]\s*async\s+fn\s+fresh_baseline_enrolls_dataset_security_before_core_actor_bootstrap\s*\(') {
        throw "Validation no longer owns the exact API SQLx test selected by the full and fast lanes."
    }
    $unowned = @($sqlxMatches | Where-Object {
        $relative = [IO.Path]::GetRelativePath($repoRoot, $_.Path).Replace('\', '/')
        $relative -cne 'crates/tessara-api/src/composition/mod.rs' -and
        $relative -notlike 'crates/tessara-dataset-module/*'
    })
    if ($unowned.Count -ne 0) {
        throw "Validation found SQLx tests outside the explicitly owned API and Dataset test lanes."
    }
    return Get-TessaraDatasetCargoTestPlan `
        -Metadata $Metadata `
        -SqlxMatches $sqlxMatches
}

function Assert-TessaraFullCargoTestPartition {
    param([Parameter(Mandatory)]$Metadata)

    $workspaceMemberIds = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::Ordinal
    )
    foreach ($member in @($Metadata.workspace_members)) {
        [void]$workspaceMemberIds.Add([string]$member)
    }
    $workspacePackages = @($Metadata.packages | Where-Object {
        $workspaceMemberIds.Contains([string]$_.id)
    })
    $workspaceNames = @($workspacePackages.name | Sort-Object)
    if ($workspaceNames.Count -eq 0 -or
        @($workspaceNames | Group-Object | Where-Object Count -ne 1).Count -ne 0) {
        throw "Full validation could not establish a unique Cargo workspace package inventory."
    }
    foreach ($isolatedPackage in $fullValidationIsolatedCargoPackages) {
        if (@($workspaceNames | Where-Object { [string]$_ -ceq $isolatedPackage }).Count -ne 1) {
            throw "Full validation isolated Cargo partition is missing '$isolatedPackage'."
        }
    }
    $workspacePartition = @($workspaceNames | Where-Object {
        [string]$_ -notin $fullValidationIsolatedCargoPackages
    })
    $partitionedNames = @($workspacePartition + $fullValidationIsolatedCargoPackages | Sort-Object)
    if (($partitionedNames -join "`n") -cne ($workspaceNames -join "`n")) {
        throw "Full validation Cargo partitions do not cover every workspace package exactly once."
    }
}

function Invoke-TessaraValidationPreflightSelfTest {
    function Assert-Rejected {
        param(
            [Parameter(Mandatory)][scriptblock]$Action,
            [Parameter(Mandatory)][string]$ExpectedMessage
        )

        try {
            & $Action
        } catch {
            if (-not $_.Exception.Message.Contains($ExpectedMessage)) {
                throw
            }
            return
        }
        throw "Expected validation preflight rejection containing '$ExpectedMessage'."
    }

    foreach ($accepted in @(
        "tessara_test",
        "tessara-test-api",
        "tessara-tests-01",
        "tessara_testing_01"
    )) {
        if (-not (Test-TessaraDisposableDatabaseName -DatabaseName $accepted)) {
            throw "Validation preflight self-test rejected disposable database name '$accepted'."
        }
    }
    foreach ($rejected in @(
        "latest",
        "contest",
        "attested",
        "production_upgradeable",
        "sprint6atest",
        "production",
        "production_clone",
        "prod_rollback",
        "live_upgrade",
        "tessara_sprint6a_prod"
    )) {
        if (Test-TessaraDisposableDatabaseName -DatabaseName $rejected) {
            throw "Validation preflight self-test accepted unsafe database name '$rejected'."
        }
    }

    $expectedIsolatedPackages = @(
        "tessara-api",
        "tessara-component-module",
        "tessara-dataset-module",
        "tessara-installation-control",
        "tessara-reference-scoped-records"
    ) | Sort-Object
    if ((@($fullValidationIsolatedCargoPackages | Sort-Object) -join "`n") -cne
        ($expectedIsolatedPackages -join "`n")) {
        throw "Full-validation database package isolation inventory drifted."
    }
    $expectedDatabasePackageEnvironmentNames = @(
        "TEST_COMPONENT_MODULE_DATABASE_URL",
        "TEST_INSTALLATION_CONTROL_DATABASE_URL",
        "TEST_REFERENCE_MODULE_DATABASE_URL"
    ) | Sort-Object
    if ((@($fullValidationDatabasePackageScopes.environment_name | Sort-Object) -join "`n") -cne
        ($expectedDatabasePackageEnvironmentNames -join "`n") -or
        @($fullValidationDatabasePackageScopes.package | Sort-Object -Unique).Count -ne 3) {
        throw "Full-validation exact package/database scope inventory drifted."
    }
    $expectedApiGeneralSkips = @(
        $fullValidationApiEnrollmentTest,
        $fullValidationApiFreshTest,
        $fullValidationApiSqlxTest
    ) | Sort-Object
    if ((@($fullValidationApiGeneralSkipTests | Sort-Object) -join "`n") -cne
        ($expectedApiGeneralSkips -join "`n")) {
        throw "Full-validation API database-scope skip inventory drifted."
    }
    $staticCargoPlan = @(Get-TessaraFullStaticCargoCommandPlan)
    $expectedStaticCargoPlan = @(
        [pscustomobject]@{
            label = "Workspace check"
            arguments = @("check", "--workspace", "--all-features", "--locked")
        },
        [pscustomobject]@{
            label = "Clippy (warnings denied)"
            arguments = @(
                "clippy", "--workspace", "--all-targets", "--all-features",
                "--locked", "--", "-D", "warnings"
            )
        }
    )
    if ($staticCargoPlan.Count -ne $expectedStaticCargoPlan.Count) {
        throw "Full-validation guarded static Cargo command inventory drifted."
    }
    for ($planIndex = 0; $planIndex -lt $expectedStaticCargoPlan.Count; $planIndex++) {
        if ([string]$staticCargoPlan[$planIndex].label -cne
                [string]$expectedStaticCargoPlan[$planIndex].label -or
            (@($staticCargoPlan[$planIndex].arguments) -join "`n") -cne
                (@($expectedStaticCargoPlan[$planIndex].arguments) -join "`n")) {
            throw "Full-validation guarded static Cargo command plan drifted."
        }
    }
    $validationScriptSource = Get-Content -Raw -LiteralPath $PSCommandPath
    $runtimeGuardIndex = $validationScriptSource.IndexOf(
        "if (`$PSVersionTable.PSEdition",
        [StringComparison]::Ordinal
    )
    $importIndex = $validationScriptSource.IndexOf(
        "Import-Module",
        [StringComparison]::Ordinal
    )
    if ($runtimeGuardIndex -lt 0 -or $importIndex -le $runtimeGuardIndex) {
        throw "PowerShell Core version preflight no longer precedes policy imports."
    }
    $datasetExecutionIndex = $validationScriptSource.LastIndexOf(
        'Invoke-CheckedStep -Label "Extracted Dataset module tests"',
        [StringComparison]::Ordinal
    )
    $freshExecutionIndex = $validationScriptSource.LastIndexOf(
        'Invoke-CheckedStep -Label "Destructive fresh API baseline proof"',
        [StringComparison]::Ordinal
    )
    if ($datasetExecutionIndex -lt 0 -or
        $freshExecutionIndex -le $datasetExecutionIndex) {
        throw "Destructive fresh API proof no longer executes last after Dataset."
    }

    Assert-TessaraExactApiProofSourceInventory -RepositoryRoot $repoRoot
    $proofSourcePaths = @(
        'crates/tessara-api/src/core_security.rs',
        'crates/tessara-api/src/composition/mod.rs',
        'crates/tessara-api/tests/modules.rs',
        'crates/tessara-api/tests/sprint_6a_populated_upgrade.rs'
    )
    $proofSources = [ordered]@{}
    foreach ($proofSourcePath in $proofSourcePaths) {
        $proofSources[$proofSourcePath] = Get-Content -Raw `
            -LiteralPath (Join-Path $repoRoot $proofSourcePath)
    }
    $renamedProofSources = [ordered]@{} + $proofSources
    $renamedProofSources['crates/tessara-api/src/core_security.rs'] =
        ([string]$renamedProofSources['crates/tessara-api/src/core_security.rs']).Replace(
            'async fn local_enrollment_is_atomic_global_and_idempotent(',
            'async fn renamed_local_enrollment_proof('
        )
    Assert-Rejected -Action {
        Assert-TessaraExactApiProofSourceInventory -RepositoryRoot $repoRoot `
            -SourceReader {
                param([string]$Path)
                [string]$renamedProofSources[$Path]
            }
    } -ExpectedMessage "requires exactly one source definition"
    $uncfgProofSources = [ordered]@{} + $proofSources
    $uncfgProofSources['crates/tessara-api/tests/modules.rs'] =
        [regex]::Replace(
            [string]$uncfgProofSources['crates/tessara-api/tests/modules.rs'],
            [regex]::Escape('#[cfg(not(debug_assertions))]') + '\s*' +
                [regex]::Escape('#[tokio::test]') + '\s*async fn ' +
                [regex]::Escape($fullValidationApiReleaseTimingTest),
            "#[tokio::test]`nasync fn $fullValidationApiReleaseTimingTest",
            [Text.RegularExpressions.RegexOptions]::None,
            [TimeSpan]::FromSeconds(1)
        )
    Assert-Rejected -Action {
        Assert-TessaraExactApiProofSourceInventory -RepositoryRoot $repoRoot `
            -SourceReader {
                param([string]$Path)
                [string]$uncfgProofSources[$Path]
            }
    } -ExpectedMessage "lost its required attribute"

    Assert-TessaraExactCargoTestList `
        -Output @("$fullValidationApiEnrollmentTest`: test") `
        -ExpectedTestName $fullValidationApiEnrollmentTest
    Assert-TessaraExactCargoTestSummary `
        -Output @('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 124 filtered out; finished in 0.01s') `
        -ExpectedTestName $fullValidationApiEnrollmentTest
    Assert-TessaraCargoSkipFilterList `
        -ListedTests @($fullValidationApiGeneralSkipTests + 'unrelated::proof') `
        -SkipTests $fullValidationApiGeneralSkipTests
    foreach ($invalidList in @(
            @(),
            @("$fullValidationApiEnrollmentTest`: test", "$fullValidationApiEnrollmentTest`: test"),
            @('wrong::test: test')
        )) {
        Assert-Rejected -Action {
            Assert-TessaraExactCargoTestList -Output @($invalidList) `
                -ExpectedTestName $fullValidationApiEnrollmentTest
        } -ExpectedMessage "did not resolve to exactly one test"
    }
    Assert-Rejected -Action {
        Assert-TessaraExactCargoTestSummary `
            -Output @('test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 1 filtered out; finished in 0.01s') `
            -ExpectedTestName $fullValidationApiEnrollmentTest
    } -ExpectedMessage "did not report exactly one passing assertion"
    Assert-Rejected -Action {
        Assert-TessaraCargoSkipFilterList `
            -ListedTests @(
                $fullValidationApiEnrollmentTest,
                "$fullValidationApiEnrollmentTest-extra"
            ) `
            -SkipTests @($fullValidationApiEnrollmentTest)
    } -ExpectedMessage "is not an exact one-test filter"

    $validEnvironment = [ordered]@{
        TEST_API_DATABASE_URL = "postgres://tester_api:validation-secret-api@127.0.0.1:55432/tessara_test_api"
        TEST_API_FRESH_DATABASE_URL = "postgres://tester_fresh:validation-secret-fresh@127.0.0.1:55432/tessara_test_api_fresh"
        TEST_SQLX_DATABASE_URL = "postgres://tester_sqlx:validation-secret-sqlx@127.0.0.1:55432/tessara_test_sqlx"
        TEST_REFERENCE_MODULE_DATABASE_URL = "postgres://tester_reference:validation-secret-reference@127.0.0.1:55432/tessara_test_reference_module"
        TEST_COMPONENT_MODULE_DATABASE_URL = "postgres://tester_component:validation-secret-component@127.0.0.1:55432/tessara_test_component_module"
        TEST_API_ENROLLMENT_DATABASE_URL = "postgres://tester_enrollment:validation-secret-enrollment@127.0.0.1:55432/tessara_test_api_enrollment"
        TEST_INSTALLATION_CONTROL_DATABASE_URL = "postgres://tester_installation:validation-secret-installation@127.0.0.1:55432/tessara_test_installation_control"
        SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = $destructiveFreshResetAcknowledgement
    }
    $fakeProbe = {
        param($Endpoint)
        New-TessaraAuthenticatedDatabaseIdentityRecord `
            -ServerAddress "192.0.2.10" `
            -ServerPort ([int]$Endpoint.Port) `
            -DatabaseName ([string]$Endpoint.DatabaseName) `
            -RoleName ([string]$Endpoint.Role)
    }
    $endpoints = @(
        Assert-TessaraFullValidationDatabaseEnvironment -Environment $validEnvironment `
            -AuthenticatedProbe $fakeProbe
    )
    if ($endpoints.Count -ne $fullValidationDatabaseEnvironmentNames.Count) {
        throw "Validation preflight self-test did not return every required database endpoint."
    }

    $missing = [ordered]@{} + $validEnvironment
    [void]$missing.Remove("TEST_API_ENROLLMENT_DATABASE_URL")
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $missing -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "requires TEST_API_ENROLLMENT_DATABASE_URL"

    $duplicate = [ordered]@{} + $validEnvironment
    $duplicate.TEST_API_ENROLLMENT_DATABASE_URL =
        "postgres://other:other-secret@127.0.0.1:55432/tessara_test_api"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $duplicate -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "pairwise-distinct"

    $aliasDuplicate = [ordered]@{} + $validEnvironment
    $aliasDuplicate.TEST_SQLX_DATABASE_URL =
        "postgres://tester:validation-secret@localhost:55432/tessara_test_api"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $aliasDuplicate -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "pairwise-distinct"

    $ipv6AliasDuplicate = [ordered]@{} + $validEnvironment
    $ipv6AliasDuplicate.TEST_SQLX_DATABASE_URL =
        "postgres://tester:validation-secret@[::1]:55432/tessara_test_api"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $ipv6AliasDuplicate -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "pairwise-distinct"

    $roleDuplicate = [ordered]@{} + $validEnvironment
    $roleDuplicate.TEST_SQLX_DATABASE_URL =
        "postgres://tester_api:validation-secret@127.0.0.1:55432/tessara_test_sqlx"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $roleDuplicate -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "host/port/role identities"

    $passwordDuplicate = [ordered]@{} + $validEnvironment
    $passwordDuplicate.TEST_API_ENROLLMENT_DATABASE_URL =
        "postgres://tester_enrollment:validation-secret-api@127.0.0.1:55432/tessara_test_api_enrollment"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $passwordDuplicate -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "pairwise-distinct passwords"

    Assert-Rejected `
        -Action {
            Assert-TessaraFullValidationDatabaseEnvironment -Environment $validEnvironment `
                -AuthenticatedProbe {
                    param($Endpoint)
                    $record = New-TessaraAuthenticatedDatabaseIdentityRecord `
                        -ServerAddress "198.51.100.5" `
                        -ServerPort 5432 `
                        -DatabaseName ([string]$Endpoint.DatabaseName) `
                        -RoleName ([string]$Endpoint.Role)
                    $record.database_identity = '["198.51.100.5",5432,"shared_test_database"]'
                    $record
                }
        } `
        -ExpectedMessage "authenticated physical database"

    Assert-Rejected `
        -Action {
            Assert-TessaraFullValidationDatabaseEnvironment -Environment $validEnvironment `
                -AuthenticatedProbe {
                    param($Endpoint)
                    $record = New-TessaraAuthenticatedDatabaseIdentityRecord `
                        -ServerAddress "198.51.100.5" `
                        -ServerPort 5432 `
                        -DatabaseName ([string]$Endpoint.DatabaseName) `
                        -RoleName ([string]$Endpoint.Role)
                    $record.server_role_identity = '["198.51.100.5",5432,"shared_validation_role"]'
                    $record
                }
        } `
        -ExpectedMessage "authenticated server role"

    Assert-Rejected `
        -Action {
            Assert-TessaraFullValidationDatabaseEnvironment -Environment $validEnvironment `
                -AuthenticatedProbe { param($Endpoint) "legacy-string-identity" }
        } `
        -ExpectedMessage "invalid nonsecret database/role identity"

    $unsafe = [ordered]@{} + $validEnvironment
    $unsafe.TEST_API_DATABASE_URL = "postgres://tester:validation-secret@127.0.0.1:55432/production"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $unsafe -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "disposable namespace"

    $wrongScheme = [ordered]@{} + $validEnvironment
    $wrongScheme.TEST_API_DATABASE_URL = "https://127.0.0.1/tessara_test_api"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $wrongScheme -AuthenticatedProbe $fakeProbe } `
        -ExpectedMessage "postgres:// or postgresql://"

    foreach ($passwordlessUrl in @(
        "postgres://tester@127.0.0.1:55432/tessara_test_api",
        "postgres://tester:@127.0.0.1:55432/tessara_test_api",
        "postgres://tester:%20@127.0.0.1:55432/tessara_test_api"
    )) {
        $passwordlessEnvironment = [ordered]@{} + $validEnvironment
        $passwordlessEnvironment.TEST_API_DATABASE_URL = $passwordlessUrl
        Assert-Rejected `
            -Action {
                Assert-TessaraFullValidationDatabaseEnvironment `
                    -Environment $passwordlessEnvironment `
                    -AuthenticatedProbe $fakeProbe
            } `
            -ExpectedMessage "explicit nonblank PostgreSQL password"
    }

    foreach ($identityOverride in @(
        "dbname=production",
        "host=production.example",
        "hostaddr=192.0.2.20",
        "port=5433",
        "user=production",
        "password=production",
        "service=production",
        "sslmode=require&dbname=production"
    )) {
        $overrideEnvironment = [ordered]@{} + $validEnvironment
        $overrideEnvironment.TEST_API_DATABASE_URL =
            "postgres://tester:validation-secret@127.0.0.1:55432/tessara_test_api?$identityOverride"
        Assert-Rejected `
            -Action {
                Assert-TessaraFullValidationDatabaseEnvironment `
                    -Environment $overrideEnvironment -AuthenticatedProbe $fakeProbe
            } `
            -ExpectedMessage "connection-identity overrides are forbidden"
    }

    $validSslMode = [ordered]@{} + $validEnvironment
    $validSslMode.TEST_API_DATABASE_URL =
        "postgres://tester:validation-secret@127.0.0.1:55432/tessara_test_api?sslmode=require"
    [void](Assert-TessaraFullValidationDatabaseEnvironment `
        -Environment $validSslMode -AuthenticatedProbe $fakeProbe)

    $missingAcknowledgement = [ordered]@{} + $validEnvironment
    $missingAcknowledgement.SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = ""
    $acknowledgementProbeCapture = [pscustomobject]@{ invoked = $false }
    Assert-Rejected `
        -Action {
            Assert-TessaraFullValidationDatabaseEnvironment `
                -Environment $missingAcknowledgement `
                -AuthenticatedProbe {
                    param($Endpoint)
                    $acknowledgementProbeCapture.invoked = $true
                    throw "database probe must not run"
                }
        } `
        -ExpectedMessage "SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET"
    if ($acknowledgementProbeCapture.invoked) {
        throw "Destructive acknowledgement rejection occurred after an external database probe."
    }

    function New-FakeAuthenticatedProbeOutput {
        param(
            [Parameter(Mandatory)]$Endpoint,
            [string]$Owner,
            [string]$Superuser = "f",
            [AllowNull()][string]$CreateDatabase,
            [string]$CreateRole = "f",
            [string]$Replication = "f",
            [string]$BypassRls = "f",
            [string]$MembershipCount = "0",
            [string]$OtherOwnedDatabaseCount = "0",
            [string]$OtherDatabasePrivilegeCount = "0",
            [string]$Connect = "t",
            [string]$Create = "t",
            [string]$Temporary = "t",
            [string]$PublicCreate = "t",
            [string]$OwnershipMarker = $validationDatabaseOwnershipMarker
        )
        if ([string]::IsNullOrWhiteSpace($Owner)) {
            $Owner = [string]$Endpoint.Role
        }
        if ([string]::IsNullOrEmpty($CreateDatabase)) {
            $CreateDatabase = if (
                [string]$Endpoint.EnvironmentName -ceq "TEST_SQLX_DATABASE_URL"
            ) { "t" } else { "f" }
        }
        return @(
            "192.0.2.10",
            [string]$Endpoint.Port,
            [string]$Endpoint.DatabaseName,
            [string]$Endpoint.Role,
            $Owner,
            $Superuser,
            $CreateDatabase,
            $CreateRole,
            $Replication,
            $BypassRls,
            $MembershipCount,
            $OtherOwnedDatabaseCount,
            $OtherDatabasePrivilegeCount,
            $Connect,
            $Create,
            $Temporary,
            $PublicCreate,
            $OwnershipMarker
        ) -join "`t"
    }

    $sqlxEndpoint = @($endpoints | Where-Object {
        [string]$_.EnvironmentName -ceq "TEST_SQLX_DATABASE_URL"
    })[0]
    $postgresSentinelBindings = [ordered]@{
        PGHOSTADDR = "203.0.113.41"
        PGPASSWORD = "ambient-password-must-not-leak"
        PGSSLMODE = "verify-full"
        PGOPTIONS = "-c search_path=ambient"
        PGSSLROOTCERT = "ambient-root.crt"
        PGSSLCERT = "ambient-client.crt"
        PGSSLKEY = "ambient-client.key"
    }
    $outerPostgresNames = @(
        (Get-TessaraPostgresProcessEnvironmentNames) +
            @($postgresSentinelBindings.Keys) |
            Sort-Object -Unique
    )
    $outerPostgresSnapshot = Get-TessaraProcessEnvironmentSnapshot `
        -Names $outerPostgresNames
    try {
        Set-TessaraProcessEnvironmentBindings -Bindings $postgresSentinelBindings
        $psqlCapture = [pscustomobject]@{
            arguments = @()
            environment = $null
            line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint
        }
        $fakePsqlInvoker = {
            param([string[]]$Arguments)
            $psqlCapture.arguments = @($Arguments)
            $psqlCapture.environment = Get-TessaraProcessEnvironmentSnapshot -Names @(
                "PGHOST", "PGPORT", "PGDATABASE", "PGUSER", "PGPASSWORD",
                "PGSSLMODE", "PGCONNECT_TIMEOUT", "PGAPPNAME", "PGHOSTADDR",
                "PGOPTIONS", "PGSSLROOTCERT", "PGSSLCERT", "PGSSLKEY"
            )
            return [string]$psqlCapture.line
        }
        $authenticatedIdentity = Get-TessaraAuthenticatedDatabaseIdentity `
            -Endpoint $sqlxEndpoint `
            -PsqlInvoker $fakePsqlInvoker
        $expectedAuthenticatedIdentity = New-TessaraAuthenticatedDatabaseIdentityRecord `
            -ServerAddress "192.0.2.10" `
            -ServerPort ([int]$sqlxEndpoint.Port) `
            -DatabaseName ([string]$sqlxEndpoint.DatabaseName) `
            -RoleName ([string]$sqlxEndpoint.Role)
        if ([string]$authenticatedIdentity.database_identity -cne
                [string]$expectedAuthenticatedIdentity.database_identity -or
            [string]$authenticatedIdentity.server_role_identity -cne
                [string]$expectedAuthenticatedIdentity.server_role_identity) {
            throw "Injected authenticated database probe returned the wrong normalized identity."
        }
        if ("-w" -notin $psqlCapture.arguments -or
            "ON_ERROR_STOP=1" -notin $psqlCapture.arguments -or
            "`t" -notin $psqlCapture.arguments) {
            throw "Authenticated database probe did not use noninteractive, fail-fast, tab-delimited psql arguments."
        }
        $probeCommandIndex = [Array]::IndexOf($psqlCapture.arguments, "-c")
        $probeCommand = if ($probeCommandIndex -ge 0 -and
            $probeCommandIndex + 1 -lt $psqlCapture.arguments.Count) {
            [string]$psqlCapture.arguments[$probeCommandIndex + 1]
        } else { "" }
        if ([regex]::Matches(
                $probeCommand,
                "THEN 't' ELSE 'f' END"
            ).Count -ne 9 -or
            $probeCommand -match 'role_entry\.rol(?:super|createdb|createrole|replication|bypassrls)::text') {
            throw "Authenticated database probe did not normalize all live PostgreSQL boolean values to the validator's t/f contract."
        }
        $expectedProbeEnvironment = [ordered]@{
            PGHOST = [string]$sqlxEndpoint.Host
            PGPORT = [string]$sqlxEndpoint.Port
            PGDATABASE = [string]$sqlxEndpoint.DatabaseName
            PGUSER = [string]$sqlxEndpoint.Role
            PGPASSWORD = [string]$sqlxEndpoint.Password
            PGCONNECT_TIMEOUT = "10"
            PGAPPNAME = "tessara-validation-preflight"
        }
        foreach ($name in $expectedProbeEnvironment.Keys) {
            if (-not [bool]$psqlCapture.environment[$name].present -or
                [string]$psqlCapture.environment[$name].value -cne
                    [string]$expectedProbeEnvironment[$name]) {
                throw "Authenticated database probe did not establish exact '$name' semantics."
            }
        }
        foreach ($name in @(
            "PGSSLMODE", "PGHOSTADDR", "PGOPTIONS", "PGSSLROOTCERT",
            "PGSSLCERT", "PGSSLKEY"
        )) {
            if ([bool]$psqlCapture.environment[$name].present) {
                throw "Authenticated database probe inherited forbidden caller setting '$name'."
            }
        }
        foreach ($name in $postgresSentinelBindings.Keys) {
            if ([Environment]::GetEnvironmentVariable([string]$name) -cne
                [string]$postgresSentinelBindings[$name]) {
                throw "Authenticated database probe did not restore caller setting '$name'."
            }
        }

        Assert-Rejected `
            -Action {
                Get-TessaraAuthenticatedDatabaseIdentity `
                    -Endpoint $sqlxEndpoint `
                    -PsqlInvoker { throw "injected psql failure" }
            } `
            -ExpectedMessage "Authenticated database probe failed for TEST_SQLX_DATABASE_URL"
        foreach ($name in $postgresSentinelBindings.Keys) {
            if ([Environment]::GetEnvironmentVariable([string]$name) -cne
                [string]$postgresSentinelBindings[$name]) {
                throw "Failing database probe did not restore caller setting '$name'."
            }
        }

        $apiEndpoint = @($endpoints | Where-Object {
            [string]$_.EnvironmentName -ceq "TEST_API_DATABASE_URL"
        })[0]
        foreach ($capabilityCase in @(
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -Owner "not-the-validation-role"
                message = "owned by its declared validation role"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -Superuser "t"
                message = "rejects a superuser role"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -Create "f"
                message = "CONNECT, CREATE, TEMP"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -CreateDatabase "f"
                message = "CREATEDB only for TEST_SQLX_DATABASE_URL"
            },
            [pscustomobject]@{
                endpoint = $apiEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $apiEndpoint `
                    -CreateDatabase "t"
                message = "NOCREATEDB for every other validation role"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -CreateRole "t"
                message = "NOCREATEROLE, NOREPLICATION, and NOBYPASSRLS"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -Replication "t"
                message = "NOCREATEROLE, NOREPLICATION, and NOBYPASSRLS"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -BypassRls "t"
                message = "NOCREATEROLE, NOREPLICATION, and NOBYPASSRLS"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -MembershipCount "1"
                message = "no role memberships"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -OtherOwnedDatabaseCount "1"
                message = "own no other non-template database"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -OtherDatabasePrivilegeCount "1"
                message = "no CONNECT, CREATE, or TEMP capability"
            },
            [pscustomobject]@{
                endpoint = $sqlxEndpoint
                line = New-FakeAuthenticatedProbeOutput -Endpoint $sqlxEndpoint `
                    -OwnershipMarker "not-owned"
                message = "authenticated disposable ownership marker"
            }
        )) {
            $capabilityCapture = [pscustomobject]@{ line = [string]$capabilityCase.line }
            Assert-Rejected `
                -Action {
                    Get-TessaraAuthenticatedDatabaseIdentity `
                        -Endpoint $capabilityCase.endpoint `
                        -PsqlInvoker {
                            param([string[]]$Arguments)
                            [string]$capabilityCapture.line
                        }
                } `
                -ExpectedMessage ([string]$capabilityCase.message)
        }
    } finally {
        Restore-TessaraProcessEnvironmentSnapshot `
            -Snapshot $outerPostgresSnapshot `
            -IncludeCurrentNamePattern '^(?i:PG)'
    }

    $databaseUrlWasPresent = Test-Path Env:DATABASE_URL
    $originalDatabaseUrl = [Environment]::GetEnvironmentVariable("DATABASE_URL")
    try {
        Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue
        Invoke-TessaraWithDatabaseUrlBinding `
            -DatabaseUrl $validEnvironment.TEST_SQLX_DATABASE_URL `
            -Action {
                if ($env:DATABASE_URL -cne $validEnvironment.TEST_SQLX_DATABASE_URL) {
                    throw "Scoped SQLx binding was not visible inside its action."
                }
            }
        if (Test-Path Env:DATABASE_URL) {
            throw "Scoped SQLx binding did not restore an absent caller value."
        }
        [Environment]::SetEnvironmentVariable(
            "DATABASE_URL", "postgres://ambient@127.0.0.1/ambient"
        )
        Assert-Rejected -Action {
            Invoke-TessaraWithDatabaseUrlBinding `
                -DatabaseUrl $validEnvironment.TEST_SQLX_DATABASE_URL `
                -Action {
                    if ($env:DATABASE_URL -cne $validEnvironment.TEST_SQLX_DATABASE_URL) {
                        throw "Scoped SQLx binding was not visible inside its failing action."
                    }
                    throw "injected scoped binding failure"
                }
        } -ExpectedMessage "injected scoped binding failure"
        if ($env:DATABASE_URL -cne "postgres://ambient@127.0.0.1/ambient") {
            throw "Scoped SQLx binding did not restore a present caller value after failure."
        }
    } finally {
        Restore-TessaraProcessEnvironmentValue -Name "DATABASE_URL" `
            -WasPresent $databaseUrlWasPresent -Value $originalDatabaseUrl
    }
    $dotEnvNames = @(Get-TessaraDotEnvPostgresDeclarationNames -Lines @(
        "# PGIGNORED=secret-value-must-not-appear",
        " export pghostaddr=secret-value-must-not-appear",
        "PGOPTIONS=-c search_path=secret-value-must-not-appear",
        "TEST_SQLX_DATABASE_URL=secret-value-must-not-appear",
        "DATABASE_URL=permitted-because-the-process-sentinel-blocks-dotenv-override"
    ))
    $normalizedDotEnvNames = @($dotEnvNames | ForEach-Object {
        ([string]$_).ToUpperInvariant()
    } | Sort-Object)
    if (($normalizedDotEnvNames -join "|") -cne
        ((@("PGHOSTADDR", "PGOPTIONS", "TEST_SQLX_DATABASE_URL") | Sort-Object) -join "|")) {
        throw "Workspace .env blocked-input parser did not classify exact PG*/validation declarations."
    }

    $executionOverrideSecret = 'execution-override-secret-must-not-appear'
    foreach ($blockedExecutionName in @(
        'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER',
        'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER',
        'CARGO_BUILD_TARGET',
        'CARGO_BUILD_RUSTC_WRAPPER',
        'CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS',
        'CARGO_TARGET_DIR',
        'CARGO_INCREMENTAL',
        'RUSTC_WRAPPER',
        'RUSTDOC',
        'RUSTFLAGS',
        'RUSTUP_TOOLCHAIN',
        'RUST_TEST_THREADS',
        'CARGO_ENCODED_RUSTFLAGS'
    )) {
        $cargoInvocationCapture = [pscustomobject]@{ count = 0 }
        $injectedEnvironment = [ordered]@{}
        $injectedEnvironment[$blockedExecutionName] = $executionOverrideSecret
        $overrideFailure = $null
        try {
            [void](Get-TessaraCargoWorkspaceMetadata `
                -ProcessEnvironment $injectedEnvironment `
                -CargoConfigPaths @() `
                -CargoInvoker {
                    $cargoInvocationCapture.count++
                    '{"packages":[],"workspace_members":[]}'
                })
        } catch {
            $overrideFailure = $_
        }
        if ($null -eq $overrideFailure -or
            -not $overrideFailure.Exception.Message.Contains($blockedExecutionName) -or
            $overrideFailure.Exception.Message.Contains($executionOverrideSecret) -or
            $cargoInvocationCapture.count -ne 0 -or
            [string]$injectedEnvironment[$blockedExecutionName] -cne
                $executionOverrideSecret) {
            throw "Cargo execution-control preflight did not reject '$blockedExecutionName' before Cargo without exposing or mutating its value."
        }
    }

    $policyControlEnvironment = [ordered]@{
        CARGO_TARGET_DIR = 'C:\tessara-validation-selftest-target'
        CARGO_INCREMENTAL = '0'
        CARGO_PROFILE_TEST_DEBUG = '0'
    }
    $policyControlInvocationCapture = [pscustomobject]@{ count = 0 }
    [void](Invoke-TessaraCargo -Arguments @('metadata') `
        -ProcessEnvironment $policyControlEnvironment -CargoConfigPaths @() `
        -AllowedExecutionControls $policyControlEnvironment `
        -CargoInvoker {
            param([string[]]$Arguments)
            $policyControlInvocationCapture.count++
            '{}'
        })
    foreach ($policyControlName in @($policyControlEnvironment.Keys)) {
        $mutatedControls = [ordered]@{} + $policyControlEnvironment
        [void]$mutatedControls.Remove([string]$policyControlName)
        Assert-Rejected -Action {
            Invoke-TessaraCargo -Arguments @('metadata') `
                -ProcessEnvironment $mutatedControls -CargoConfigPaths @() `
                -AllowedExecutionControls $policyControlEnvironment `
                -CargoInvoker {
                    param([string[]]$Arguments)
                    $policyControlInvocationCapture.count++
                }
        } -ExpectedMessage "missing or changed"
    }
    if ($policyControlInvocationCapture.count -ne 1) {
        throw "Cargo policy-control removal reached a later Cargo invocation."
    }

    $dynamicRunnerName = 'CARGO_TARGET_TESSARA_SELFTEST_RUNNER'
    $dynamicRunnerSnapshot = Get-TessaraProcessEnvironmentSnapshot `
        -Names @($dynamicRunnerName)
    try {
        Set-TessaraProcessEnvironmentBindings -Bindings ([ordered]@{
            $dynamicRunnerName = 'runner-value-must-not-survive'
        })
    } finally {
        Restore-TessaraProcessEnvironmentSnapshot `
            -Snapshot $dynamicRunnerSnapshot `
            -IncludeCurrentNamePattern '^CARGO_TARGET_TESSARA_SELFTEST_RUNNER$'
    }

    $temporaryCargoConfig = [IO.Path]::GetTempFileName()
    $configCargoInvocationCapture = [pscustomobject]@{ count = 0 }
    $emptyProcessEnvironment = [ordered]@{}
    try {
        [IO.File]::WriteAllText(
            $temporaryCargoConfig,
            @'
[build]
jobs = 2
target-dir = "validation-target"
incremental = false

[net]
retry = 2
'@,
            [Text.UTF8Encoding]::new($false)
        )
        [void](Get-TessaraCargoWorkspaceMetadata `
            -ProcessEnvironment $emptyProcessEnvironment `
            -CargoConfigPaths @($temporaryCargoConfig) `
            -CargoInvoker {
                $configCargoInvocationCapture.count++
                '{"packages":[],"workspace_members":[]}'
            })
        if ($configCargoInvocationCapture.count -ne 1) {
            throw "Harmless effective Cargo configuration did not reach the Cargo metadata boundary exactly once."
        }

        foreach ($configMutation in @(
            [pscustomobject]@{
                text = @'
[build]
target = "execution-override-secret-must-not-appear"
'@
                expected_key = 'build.target'
            },
            [pscustomobject]@{
                text = @'
[build]
rustc-wrapper = "execution-override-secret-must-not-appear"
'@
                expected_key = 'build.rustc-wrapper'
            },
            [pscustomobject]@{
                text = @'
build = { jobs = 2, target = "execution-override-secret-must-not-appear" }
'@
                expected_key = 'build'
            },
            [pscustomobject]@{
                text = @'
[target.'cfg(all())']
"run\u006eer" = "execution-override-secret-must-not-appear"
'@
                expected_key = 'target.cfg(all()).runner'
            },
            [pscustomobject]@{
                text = @'
[target.x86_64-pc-windows-msvc]
linker = "execution-override-secret-must-not-appear"
'@
                expected_key = 'target.x86_64-pc-windows-msvc.linker'
            },
            [pscustomobject]@{
                text = @'
[target.x86_64-unknown-linux-gnu]
rustflags = ["execution-override-secret-must-not-appear"]
'@
                expected_key = 'target.x86_64-unknown-linux-gnu.rustflags'
            },
            [pscustomobject]@{
                text = @'
[env]
PATH = { value = "execution-override-secret-must-not-appear", force = true }
'@
                expected_key = 'env.path'
            },
            [pscustomobject]@{
                text = @'
[profile.release]
debug-assertions = true
'@
                expected_key = 'profile.release.debug-assertions'
            },
            [pscustomobject]@{
                text = @'
paths = ["execution-override-secret-must-not-appear"]
'@
                expected_key = 'paths'
            },
            [pscustomobject]@{
                text = @'
[source.crates-io]
replace-with = "execution-override-secret-must-not-appear"
'@
                expected_key = 'source.crates-io.replace-with'
            },
            [pscustomobject]@{
                text = @'
[target]
x86_64-unknown-linux-gnu = { runner = "execution-override-secret-must-not-appear" }
'@
                expected_key = 'target.x86_64-unknown-linux-gnu'
            },
            [pscustomobject]@{
                text = @'
include = ["execution-override-secret-must-not-appear"]
'@
                expected_key = 'include'
            }
        )) {
            [IO.File]::WriteAllText(
                $temporaryCargoConfig,
                [string]$configMutation.text,
                [Text.UTF8Encoding]::new($false)
            )
            $configFailure = $null
            try {
                [void](Get-TessaraCargoWorkspaceMetadata `
                    -ProcessEnvironment $emptyProcessEnvironment `
                    -CargoConfigPaths @($temporaryCargoConfig) `
                    -CargoInvoker {
                        $configCargoInvocationCapture.count++
                        '{"packages":[],"workspace_members":[]}'
                    })
            } catch {
                $configFailure = $_
            }
            if ($null -eq $configFailure -or
                -not $configFailure.Exception.Message.Contains(
                    [string]$configMutation.expected_key
                ) -or
                $configFailure.Exception.Message.Contains($executionOverrideSecret) -or
                $configCargoInvocationCapture.count -ne 1) {
                throw "Cargo config mutation '$($configMutation.expected_key)' was not rejected before a second Cargo invocation without exposing its value."
            }
        }
    } finally {
        if (Test-Path -LiteralPath $temporaryCargoConfig) {
            Remove-Item -LiteralPath $temporaryCargoConfig -Force
        }
    }

    $temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    $configDiscoveryRoot = Join-Path $temporaryRoot `
        ("tessara-cargo-config-selftest-" + [Guid]::NewGuid().ToString('N'))
    $temporaryPrefix = $temporaryRoot.TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    ) + [IO.Path]::DirectorySeparatorChar
    if (-not $configDiscoveryRoot.StartsWith(
            $temporaryPrefix,
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Cargo config discovery self-test root escaped the temporary directory."
    }
    $syntheticRepository = Join-Path $configDiscoveryRoot 'repository/child'
    $ancestorConfig = Join-Path $configDiscoveryRoot `
        'repository/.cargo/config.toml'
    $syntheticCargoHome = Join-Path $configDiscoveryRoot 'cargo-home'
    $userConfig = Join-Path $syntheticCargoHome 'config.toml'
    try {
        [void][IO.Directory]::CreateDirectory($syntheticRepository)
        [void][IO.Directory]::CreateDirectory((Split-Path -Parent $ancestorConfig))
        [void][IO.Directory]::CreateDirectory($syntheticCargoHome)
        foreach ($configPath in @($ancestorConfig, $userConfig)) {
            [IO.File]::WriteAllText(
                $configPath,
                "[build]`njobs = 2`n",
                [Text.UTF8Encoding]::new($false)
            )
        }
        $syntheticDiscoveryEnvironment = [ordered]@{
            CARGO_HOME = $syntheticCargoHome
        }
        $discoveredConfigs = @(Get-TessaraEffectiveCargoConfigPaths `
            -RepositoryRoot $syntheticRepository `
            -Environment $syntheticDiscoveryEnvironment)
        foreach ($requiredConfig in @($ancestorConfig, $userConfig)) {
            $requiredFullPath = [IO.Path]::GetFullPath($requiredConfig)
            if (@($discoveredConfigs | Where-Object {
                    [string]::Equals(
                        [string]$_,
                        $requiredFullPath,
                        [StringComparison]::OrdinalIgnoreCase
                    )
                }).Count -ne 1) {
                throw "Cargo config discovery did not include one required ancestor/user config file."
            }
        }

        $ancestorLegacyConfig = Join-Path `
            (Split-Path -Parent $ancestorConfig) 'config'
        [IO.File]::WriteAllText(
            $ancestorLegacyConfig,
            "[build]`njobs = 2`n",
            [Text.UTF8Encoding]::new($false)
        )
        [IO.File]::WriteAllText(
            $ancestorConfig,
            "[build]`ntarget = 'execution-override-secret-must-not-appear'`n",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-TessaraCargoExecutionControlPreflight `
            -RepositoryRoot $syntheticRepository `
            -ProcessEnvironment $syntheticDiscoveryEnvironment
        $precedencePaths = @(Get-TessaraEffectiveCargoConfigPaths `
            -RepositoryRoot $syntheticRepository `
            -Environment $syntheticDiscoveryEnvironment)
        if (@($precedencePaths | Where-Object {
                [string]::Equals(
                    [string]$_,
                    [IO.Path]::GetFullPath($ancestorLegacyConfig),
                    [StringComparison]::OrdinalIgnoreCase
                )
            }).Count -ne 1 -or
            @($precedencePaths | Where-Object {
                [string]::Equals(
                    [string]$_,
                    [IO.Path]::GetFullPath($ancestorConfig),
                    [StringComparison]::OrdinalIgnoreCase
                )
            }).Count -ne 0) {
            throw "Cargo config discovery did not apply Cargo's extensionless-file precedence."
        }
        Remove-Item -LiteralPath $ancestorLegacyConfig -Force

        foreach ($discoveryMutation in @(
            [pscustomobject]@{
                path = $userConfig
                text = "[env]`nPATH = 'execution-override-secret-must-not-appear'`n"
                expected_key = 'env.path'
            },
            [pscustomobject]@{
                path = $ancestorConfig
                text = "[target.x86_64-pc-windows-msvc]`nrunner = 'execution-override-secret-must-not-appear'`n"
                expected_key = 'target.x86_64-pc-windows-msvc.runner'
            }
        )) {
            foreach ($configPath in @($ancestorConfig, $userConfig)) {
                [IO.File]::WriteAllText(
                    $configPath,
                    "[build]`njobs = 2`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
            [IO.File]::WriteAllText(
                [string]$discoveryMutation.path,
                [string]$discoveryMutation.text,
                [Text.UTF8Encoding]::new($false)
            )
            $discoveryFailure = $null
            try {
                Assert-TessaraCargoExecutionControlPreflight `
                    -RepositoryRoot $syntheticRepository `
                    -ProcessEnvironment $syntheticDiscoveryEnvironment
            } catch {
                $discoveryFailure = $_
            }
            if ($null -eq $discoveryFailure -or
                -not $discoveryFailure.Exception.Message.Contains(
                    [string]$discoveryMutation.expected_key
                ) -or
                $discoveryFailure.Exception.Message.Contains($executionOverrideSecret)) {
                throw "Cargo config discovery did not fail closed on '$($discoveryMutation.expected_key)' without exposing its value."
            }
        }
    } finally {
        if ([IO.Directory]::Exists($configDiscoveryRoot)) {
            [IO.Directory]::Delete($configDiscoveryRoot, $true)
        }
    }

    $metadataArgumentsCapture = [pscustomobject]@{ arguments = @() }
    [void](Get-TessaraCargoWorkspaceMetadata `
        -ProcessEnvironment ([ordered]@{}) -CargoConfigPaths @() `
        -CargoInvoker {
            param([string[]]$Arguments)
            $metadataArgumentsCapture.arguments = @($Arguments)
            '{"packages":[],"workspace_members":[]}'
        })
    if ((@($metadataArgumentsCapture.arguments) -join "`n") -cne
        (@('metadata', '--locked', '--format-version', '1', '--no-deps') -join "`n")) {
        throw "Cargo metadata inventory is not fail-closed against Cargo.lock changes."
    }
    $priorSelfTestCargoBinding = $script:ActiveValidationCargoBinding
    $selfTestCargoBinding = $null
    try {
        Assert-TessaraCargoExecutionControlPreflight `
            -RepositoryRoot $repoRoot `
            -AllowedExecutionControls (
                Get-TessaraValidationAllowedCargoExecutionControls
            )
        $selfTestCargoBinding = Get-TessaraCargoExecutableBinding
        $script:ActiveValidationCargoBinding = $selfTestCargoBinding
        $cargoWorkspaceMetadata = Get-TessaraCargoWorkspaceMetadata
    } finally {
        $script:ActiveValidationCargoBinding = $priorSelfTestCargoBinding
    }
    $temporaryCargoExecutable = [IO.Path]::GetTempFileName()
    try {
        [IO.File]::WriteAllText(
            $temporaryCargoExecutable,
            'before',
            [Text.UTF8Encoding]::new($false)
        )
        $substitutionBinding = [pscustomobject][ordered]@{
            path = $temporaryCargoExecutable
            sha256 = (
                Get-FileHash -Algorithm SHA256 -LiteralPath $temporaryCargoExecutable
            ).Hash.ToLowerInvariant()
            version = [string]$selfTestCargoBinding.version
            rustc_path = [string]$selfTestCargoBinding.rustc_path
            rustc_sha256 = [string]$selfTestCargoBinding.rustc_sha256
            rustc_version = [string]$selfTestCargoBinding.rustc_version
        }
        [IO.File]::WriteAllText(
            $temporaryCargoExecutable,
            'after',
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Assert-TessaraCargoExecutableBindingCurrent -Binding $substitutionBinding
        } -ExpectedMessage "Cargo executable changed"
    } finally {
        Remove-Item -LiteralPath $temporaryCargoExecutable -Force `
            -ErrorAction SilentlyContinue
    }

    $rustcEnvironmentSnapshot = Get-TessaraProcessEnvironmentSnapshot `
        -Names @('RUSTC')
    try {
        Restore-TessaraProcessEnvironmentValue -Name 'RUSTC' `
            -WasPresent $false -Value $null
        $boundCargoCapture = [pscustomobject]@{
            count = 0
            cargo_path = $null
            arguments = @()
            rustc = $null
        }
        [void](Invoke-TessaraBoundCargoProcess `
            -CargoPath ([string]$selfTestCargoBinding.path) `
            -RustcPath ([string]$selfTestCargoBinding.rustc_path) `
            -Arguments @('metadata', '--locked') `
            -NativeInvoker {
                param([string]$CargoPath, [string[]]$Arguments)
                $boundCargoCapture.count++
                $boundCargoCapture.cargo_path = $CargoPath
                $boundCargoCapture.arguments = @($Arguments)
                $boundCargoCapture.rustc = [Environment]::GetEnvironmentVariable(
                    'RUSTC', [EnvironmentVariableTarget]::Process
                )
                '{}'
            })
        if ($boundCargoCapture.count -ne 1 -or
            [string]$boundCargoCapture.cargo_path -cne
                [string]$selfTestCargoBinding.path -or
            (@($boundCargoCapture.arguments) -join "`n") -cne
                (@('metadata', '--locked') -join "`n") -or
            [string]$boundCargoCapture.rustc -cne
                [IO.Path]::GetFullPath([string]$selfTestCargoBinding.rustc_path) -or
            (Test-Path Env:RUSTC)) {
            throw "Central Cargo invocation did not bind exact RUSTC and restore an absent caller value."
        }

        $callerRustcSentinel = 'caller-rustc-value-must-be-restored'
        Set-TessaraProcessEnvironmentBindings -Bindings ([ordered]@{
            RUSTC = $callerRustcSentinel
        })
        Assert-Rejected -Action {
            Invoke-TessaraBoundCargoProcess `
                -CargoPath ([string]$selfTestCargoBinding.path) `
                -RustcPath ([string]$selfTestCargoBinding.rustc_path) `
                -Arguments @('metadata') `
                -NativeInvoker { throw 'injected bound Cargo failure' }
        } -ExpectedMessage 'injected bound Cargo failure'
        if ([Environment]::GetEnvironmentVariable('RUSTC') -cne
            $callerRustcSentinel) {
            throw "Failing central Cargo invocation did not restore the caller's RUSTC value."
        }
    } finally {
        Restore-TessaraProcessEnvironmentSnapshot -Snapshot $rustcEnvironmentSnapshot
    }

    $temporaryRustcExecutable = [IO.Path]::GetTempFileName()
    try {
        [IO.File]::WriteAllText(
            $temporaryRustcExecutable,
            'before',
            [Text.UTF8Encoding]::new($false)
        )
        $rustcSubstitutionBinding = [pscustomobject][ordered]@{
            path = [string]$selfTestCargoBinding.path
            sha256 = [string]$selfTestCargoBinding.sha256
            version = [string]$selfTestCargoBinding.version
            rustc_path = $temporaryRustcExecutable
            rustc_sha256 = (
                Get-FileHash -Algorithm SHA256 -LiteralPath $temporaryRustcExecutable
            ).Hash.ToLowerInvariant()
            rustc_version = [string]$selfTestCargoBinding.rustc_version
        }
        [IO.File]::WriteAllText(
            $temporaryRustcExecutable,
            'after',
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Assert-TessaraCargoExecutableBindingCurrent `
                -Binding $rustcSubstitutionBinding
        } -ExpectedMessage "Cargo/Rust toolchain changed"
    } finally {
        Remove-Item -LiteralPath $temporaryRustcExecutable -Force `
            -ErrorAction SilentlyContinue
    }

    $temporaryPsqlExecutable = [IO.Path]::GetTempFileName()
    try {
        [IO.File]::WriteAllText(
            $temporaryPsqlExecutable,
            'stable-psql-bytes',
            [Text.UTF8Encoding]::new($false)
        )
        $psqlVersion = 'psql (PostgreSQL) 17.1'
        $psqlBinding = Get-TessaraPsqlExecutableBinding `
            -ExecutablePath $temporaryPsqlExecutable `
            -VersionInvoker { param([string]$Path) $psqlVersion }
        $authenticatedPsqlPath = Assert-TessaraPsqlExecutableBindingCurrent `
            -Binding $psqlBinding `
            -VersionInvoker { param([string]$Path) $psqlVersion }
        if ([string]$authenticatedPsqlPath -cne
            [IO.Path]::GetFullPath($temporaryPsqlExecutable)) {
            throw "psql binding did not retain its exact literal executable path."
        }
        Assert-Rejected -Action {
            Assert-TessaraPsqlExecutableBindingCurrent `
                -Binding $psqlBinding `
                -VersionInvoker { param([string]$Path) 'psql (PostgreSQL) 17.2' }
        } -ExpectedMessage "psql executable version changed"
        [IO.File]::WriteAllText(
            $temporaryPsqlExecutable,
            'substituted-psql-bytes',
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Assert-TessaraPsqlExecutableBindingCurrent `
                -Binding $psqlBinding `
                -VersionInvoker { param([string]$Path) $psqlVersion }
        } -ExpectedMessage "psql executable changed"
    } finally {
        Remove-Item -LiteralPath $temporaryPsqlExecutable -Force `
            -ErrorAction SilentlyContinue
    }
    Assert-TessaraWorkspaceDotEnvPostgresIsolation -Metadata $cargoWorkspaceMetadata
    Assert-TessaraFullCargoTestPartition -Metadata $cargoWorkspaceMetadata
    $currentDatasetPlan = Assert-TessaraSqlxTestInventory `
        -Metadata $cargoWorkspaceMetadata
    if (-not [bool]$currentDatasetPlan.has_lib_target -or
        -not [bool]$currentDatasetPlan.has_bin_targets -or
        -not [bool]$currentDatasetPlan.has_doctest_target -or
        @($currentDatasetPlan.test_targets).Count -eq 0 -or
        @($currentDatasetPlan.sqlx_target_names).Count -eq 0) {
        throw "Current Dataset Cargo target plan omitted a required test target class."
    }

    $syntheticNestedSource = Join-Path $repoRoot `
        "crates/tessara-dataset-module/tests/nested/custom_entry.rs"
    $syntheticDatasetMetadata = [pscustomobject]@{
        packages = @([pscustomobject]@{
            name = "tessara-dataset-module"
            targets = @(
                [pscustomobject]@{
                    name = "tessara_dataset_module"
                    kind = @("lib")
                    src_path = Join-Path $repoRoot `
                        "crates/tessara-dataset-module/src/lib.rs"
                    test = $true
                    doctest = $true
                },
                [pscustomobject]@{
                    name = "custom-nested-target"
                    kind = @("test")
                    src_path = $syntheticNestedSource
                    test = $true
                    doctest = $false
                }
            )
        })
    }
    $syntheticDatasetPlan = Get-TessaraDatasetCargoTestPlan `
        -Metadata $syntheticDatasetMetadata `
        -SqlxMatches @([pscustomobject]@{ Path = $syntheticNestedSource })
    if (@($syntheticDatasetPlan.sqlx_target_names).Count -ne 1 -or
        [string]$syntheticDatasetPlan.sqlx_target_names[0] -cne
            "custom-nested-target") {
        throw "Dataset target planning derived a filesystem basename instead of the Cargo target name."
    }
    Assert-Rejected `
        -Action {
            Get-TessaraDatasetCargoTestPlan `
                -Metadata $syntheticDatasetMetadata `
                -SqlxMatches @([pscustomobject]@{
                    Path = Join-Path $repoRoot `
                        "crates/tessara-dataset-module/tests/nested/helper.rs"
                })
        } `
        -ExpectedMessage "not the unique root source"

    $primaryFailureRecord = $null
    try {
        throw "injected product failure"
    } catch {
        $primaryFailureRecord = $_
    }
    $simulatedCleanupFailures = [Collections.Generic.List[Exception]]::new()
    $cleanupAttempts = [Collections.Generic.List[string]]::new()
    foreach ($cleanupAction in @(
        [pscustomobject]@{
            name = "first-restore"
            action = { throw "injected first restoration failure" }
        },
        [pscustomobject]@{
            name = "later-restore"
            action = { $cleanupAttempts.Add("later-restore") }
        }
    )) {
        try {
            $cleanupAttempts.Add([string]$cleanupAction.name)
            & $cleanupAction.action
        } catch {
            $simulatedCleanupFailures.Add($_.Exception)
        }
    }
    $aggregateFailure = $null
    try {
        Throw-TessaraPrimaryOrCleanupFailure `
            -PrimaryFailure $primaryFailureRecord `
            -CleanupFailures $simulatedCleanupFailures `
            -Boundary "Injected cleanup boundary"
    } catch {
        $aggregateFailure = $_
    }
    if ($null -eq $aggregateFailure -or
        -not $aggregateFailure.Exception.Message.Contains("injected product failure") -or
        -not $aggregateFailure.Exception.Message.Contains("injected first restoration failure") -or
        "later-restore" -notin $cleanupAttempts) {
        throw "Cleanup aggregation did not retain the primary failure and continue later restoration attempts."
    }

    Assert-Rejected -Action {
        Invoke-CheckedStep -Label "Injected early native failure" -Command {
            & pwsh -NoProfile -Command "exit 23"
            & pwsh -NoProfile -Command "exit 0"
        }
    } -ExpectedMessage "Injected early native failure failed"

    Write-Host "Full-validation database preflight self-test passed." -ForegroundColor Green
}

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory)]
        [string]$Label,

        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    $startedAt = Get-Date

    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            throw "$Label failed with exit code $LASTEXITCODE"
        }
    } catch {
        throw [InvalidOperationException]::new(
            "$Label failed: $($_.Exception.Message)",
            $_.Exception
        )
    }

    $elapsed = (Get-Date) - $startedAt
    Write-Host ("Passed in {0:mm\:ss}" -f $elapsed) -ForegroundColor Green
}

function Clear-TessaraWebTestArtifacts {
    $isWindowsPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows
    )

    if (-not $isWindowsPlatform) {
        return
    }

    Write-Host "`n==> Cleaning tessara-web Windows PDB/test artifacts" -ForegroundColor Cyan
    $startedAt = Get-Date

    Invoke-TessaraCargo -Arguments @('clean', '-p', 'tessara-web')

    if ($LASTEXITCODE -ne 0) {
        throw "Cleaning tessara-web package failed with exit code $LASTEXITCODE"
    }

    $cargoTarget = if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        Join-Path $repoRoot "target"
    } else {
        $env:CARGO_TARGET_DIR
    }
    Remove-Item -Force (Join-Path $cargoTarget "debug\deps\*.pdb") -ErrorAction SilentlyContinue
    Remove-Item -Force (Join-Path $cargoTarget "debug\deps\*.exe") -ErrorAction SilentlyContinue

    $elapsed = (Get-Date) - $startedAt
    Write-Host ("Cleaned in {0:mm\:ss}" -f $elapsed) -ForegroundColor Green
}

if ($SelfTest) {
    if ($RetainCargoTarget) {
        throw "-RetainCargoTarget is not valid with -SelfTest."
    }
    Invoke-TessaraValidationPreflightSelfTest
    return
}

if ($Fast -and $RetainCargoTarget) {
    throw "-RetainCargoTarget applies only to full validation; fast validation uses the developer target."
}

$cargoPolicyState = $null
$locationPushed = $false
$primaryFailure = $null
$cleanupFailures = [Collections.Generic.List[Exception]]::new()
$cargoExecutionControlEnvironmentNames = @(
    [Environment]::GetEnvironmentVariables(
        [EnvironmentVariableTarget]::Process
    ).Keys | ForEach-Object { [string]$_ } | Where-Object {
        Test-TessaraCargoExecutionControlEnvironmentName -Name $_
    } | Sort-Object -Unique
)
$cargoExecutionControlEnvironmentSnapshot = Get-TessaraProcessEnvironmentSnapshot `
    -Names $cargoExecutionControlEnvironmentNames
if ($Fast) {
    foreach ($name in @('CARGO_TARGET_DIR', 'CARGO_INCREMENTAL')) {
        if ($cargoExecutionControlEnvironmentSnapshot.Contains($name) -and
            [bool]$cargoExecutionControlEnvironmentSnapshot[$name].present) {
            $script:InitialValidationCargoControlValues[$name] =
                [string]$cargoExecutionControlEnvironmentSnapshot[$name].value
        }
    }
}
$databaseUrlSnapshot = Get-TessaraProcessEnvironmentSnapshot -Names @("DATABASE_URL")
$postgresEnvironmentNames = @(Get-TessaraPostgresProcessEnvironmentNames)
$postgresEnvironmentSnapshot = Get-TessaraProcessEnvironmentSnapshot `
    -Names $postgresEnvironmentNames
$validationDatabaseEnvironmentSnapshot = Get-TessaraProcessEnvironmentSnapshot `
    -Names $fullValidationProcessEnvironmentNames
$validationEnvironment = $null
$datasetTestPlan = $null
try {
    Push-Location $repoRoot
    $locationPushed = $true
    Clear-TessaraProcessEnvironmentValues -Names $postgresEnvironmentNames
    Clear-TessaraProcessEnvironmentValues -Names $fullValidationProcessEnvironmentNames
    Set-TessaraProcessEnvironmentBindings -Bindings ([ordered]@{
        DATABASE_URL = $blockedDatabaseUrl
    })

    Assert-TessaraCargoExecutionControlPreflight `
        -RepositoryRoot $repoRoot `
        -AllowedExecutionControls (
            Get-TessaraValidationAllowedCargoExecutionControls
        )
    $script:ActiveValidationCargoBinding = Get-TessaraCargoExecutableBinding
    Assert-TessaraExactApiProofSourceInventory -RepositoryRoot $repoRoot
    $cargoWorkspaceMetadata = Get-TessaraCargoWorkspaceMetadata
    Assert-TessaraWorkspaceDotEnvPostgresIsolation -Metadata $cargoWorkspaceMetadata
    Assert-TessaraFullCargoTestPartition -Metadata $cargoWorkspaceMetadata
    $datasetTestPlan = Assert-TessaraSqlxTestInventory -Metadata $cargoWorkspaceMetadata

    if ($Fast) {
        Write-Host "Running fast Tessara validation. Use .\scripts\validate.ps1 for the full pre-commit matrix." -ForegroundColor Yellow
    } else {
        Write-Host "Running full Tessara validation sequentially. This avoids Cargo lock contention on Windows." -ForegroundColor Yellow
        $script:ActiveValidationPsqlBinding = Get-TessaraPsqlExecutableBinding
        $validationEnvironment = [ordered]@{
            SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET =
                if ([bool]$validationDatabaseEnvironmentSnapshot.SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET.present) {
                    [string]$validationDatabaseEnvironmentSnapshot.SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET.value
                } else { $null }
        }
        foreach ($environmentName in $fullValidationDatabaseEnvironmentNames) {
            $validationEnvironment[$environmentName] =
                if ([bool]$validationDatabaseEnvironmentSnapshot[$environmentName].present) {
                    [string]$validationDatabaseEnvironmentSnapshot[$environmentName].value
                } else { $null }
        }
        [void](Assert-TessaraFullValidationDatabaseEnvironment `
            -Environment $validationEnvironment)
        $cargoPolicyMode = if ($RetainCargoTarget) { "Diagnostic" } else { "Validation" }
        $cargoPolicyState = Enter-TessaraCargoBuildPolicy `
            -Lane "full-validation" `
            -RepositoryRoot $repoRoot `
            -Mode $cargoPolicyMode `
            -CargoExecutablePath ([string]$script:ActiveValidationCargoBinding.path) `
            -RetainTarget:$RetainCargoTarget
        $script:ActiveValidationCargoPolicyState = $cargoPolicyState
    }
    Assert-TessaraValidationAmbientDatabaseIsolation

    Invoke-CheckedStep -Label "Acceptance PowerShell contracts" -Command {
        foreach ($scriptFile in @(Get-ChildItem -Path (Join-Path $repoRoot "scripts") -Filter "*.ps1" -File | Sort-Object FullName)) {
            $relativePath = [IO.Path]::GetRelativePath($repoRoot, $scriptFile.FullName)
            $tokens = $null
            $parseErrors = $null
            [void][Management.Automation.Language.Parser]::ParseFile(
                $scriptFile.FullName,
                [ref]$tokens,
                [ref]$parseErrors
            )
            if ($parseErrors.Count -ne 0) {
                throw "PowerShell AST validation failed for '$relativePath': $($parseErrors[0].Message)"
            }
        }
        & pwsh -NoProfile -File .\scripts\validate.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) {
            throw "validation preflight self-test failed with exit code $LASTEXITCODE"
        }
        & pwsh `
            -NoProfile `
            -File .\scripts\test-tessara-cargo-build-policy.ps1 `
            -SelfTest
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build policy self-test failed with exit code $LASTEXITCODE"
        }
        & pwsh `
            -NoProfile `
            -File .\scripts\test-tessara-validation-policy.ps1 `
            -SelfTest
        if ($LASTEXITCODE -ne 0) {
            throw "Validation policy self-test failed with exit code $LASTEXITCODE"
        }
        & pwsh `
            -NoProfile `
            -File .\scripts\test-tessara-validation-platform.ps1 `
            -SelfTest
        if ($LASTEXITCODE -ne 0) {
            throw "Validation platform certification failed with exit code $LASTEXITCODE"
        }
        & .\scripts\local-launch.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "local-launch self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\capture-sprint-6a-deployment-evidence.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "deployment-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\validate-e2e.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Playwright-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\validate-resource-reference-nondisclosure.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "nondisclosure-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\test-sprint-6a-acceptance-evidence.ps1
        if ($LASTEXITCODE -ne 0) { throw "smoke/UAT acceptance-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\prepare-sprint-7a-uat-fixtures.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7A semantic fixture self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\smoke-sprint-7a.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7A smoke self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\uat-sprint-7a.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7A UAT self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\smoke-sprint-7b.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7B smoke self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\uat-sprint-7b.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7B UAT self-test failed with exit code $LASTEXITCODE" }
    }
    Assert-TessaraValidationAmbientDatabaseIsolation

    Invoke-CheckedStep -Label "Formatting check" -Command {
        Invoke-TessaraCargo -Arguments @('fmt', '--all', '--check')
    }

    if (-not $Fast) {
        foreach ($staticCargoStep in @(Get-TessaraFullStaticCargoCommandPlan)) {
            Invoke-CheckedStep -Label ([string]$staticCargoStep.label) -Command {
                Invoke-TessaraCargo -Arguments @($staticCargoStep.arguments)
            }
        }
    }

    Invoke-CheckedStep -Label "Module contract check" -Command {
        Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-module-contract', '--locked')
    }

    Invoke-CheckedStep -Label "Canonical module SDK boundary audit" -Command {
        & .\scripts\verify-module-sdk-boundaries.ps1
        if ($LASTEXITCODE -ne 0) { throw "module SDK boundary audit failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Canonical module SDK compatibility inventory" -Command {
        & .\scripts\verify-module-sdk-compatibility.ps1
        if ($LASTEXITCODE -ne 0) { throw "module SDK compatibility inventory failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Markdown local links" -Command {
        & .\scripts\verify-markdown-links.ps1
        if ($LASTEXITCODE -ne 0) { throw "Markdown link validation failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Canonical module SDK native checks" -Command {
        Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-module-runtime', '-p', 'tessara-module-ui', '-p', 'tessara-module-testkit', '--locked')
        Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-reference-module-sdk', '--features', 'ssr', '--locked')
    }

    Invoke-CheckedStep -Label "API check" -Command {
        Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-api', '--locked')
    }

    Invoke-CheckedStep -Label "Extracted Component module check" -Command {
        Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-components-contract', '-p', 'tessara-component-module', '--locked')
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "API SSR check" -Command {
            Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-api', '--features', 'ssr', '--locked')
        }
    }

    Invoke-CheckedStep -Label "Web check" -Command {
        Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-web', '--locked')
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "Web hydrate check" -Command {
            Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-web', '--no-default-features', '--features', 'hydrate', '--target', 'wasm32-unknown-unknown', '--locked')
        }

        Clear-TessaraWebTestArtifacts
    }

    Assert-TessaraValidationAmbientDatabaseIsolation
    if ($Fast) {
        Invoke-CheckedStep -Label "Module contract tests" -Command {
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-module-contract', '--locked')
        }

        Invoke-CheckedStep -Label "Canonical module SDK tests" -Command {
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-module-runtime', '-p', 'tessara-module-ui', '-p', 'tessara-module-testkit', '--locked')
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-reference-module-sdk', '--features', 'ssr', '--locked')
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-reference-scoped-records', '--lib', '--locked')
        }

        Invoke-CheckedStep -Label "Web tests" -Command {
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-web', '-j', '1', '--locked')
        }

        Invoke-CheckedStep -Label "Extracted Component module tests" -Command {
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-components-contract', '-p', 'tessara-component-module', '--lib', '--locked')
        }

        Invoke-CheckedStep -Label "Installation-control library tests" -Command {
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-installation-control', '--lib', '--locked')
        }
        Assert-TessaraValidationAmbientDatabaseIsolation
    } else {
        Assert-TessaraValidationAmbientDatabaseIsolation
        Invoke-CheckedStep -Label "Workspace tests excluding database-scoped packages" -Command {
            $arguments = @("test", "--workspace", "--all-features", "--locked", "-j", "1")
            foreach ($package in $fullValidationIsolatedCargoPackages) {
                $arguments += @("--exclude", $package)
            }
            Invoke-TessaraCargo -Arguments $arguments
        }
        Assert-TessaraValidationAmbientDatabaseIsolation

        foreach ($packageScope in $fullValidationDatabasePackageScopes) {
            $packageName = [string]$packageScope.package
            $packageBindings = [ordered]@{}
            $packageBindings[[string]$packageScope.environment_name] =
                [string]$validationEnvironment[[string]$packageScope.environment_name]
            Invoke-TessaraWithProcessEnvironmentBindings `
                -Bindings $packageBindings `
                -Action {
                    Invoke-CheckedStep -Label ([string]$packageScope.label) -Command {
                        Invoke-TessaraCargo -Arguments @('test', '-p', $packageName, '--all-features', '--locked')
                    }
                }
            Assert-TessaraValidationAmbientDatabaseIsolation
        }
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "Canonical module SDK WASM checks" -Command {
            Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-module-contract', '--target', 'wasm32-unknown-unknown', '--locked')
            Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-module-ui', '--no-default-features', '--features', 'hydrate', '--target', 'wasm32-unknown-unknown', '--locked')
            Invoke-TessaraCargo -Arguments @('check', '-p', 'tessara-reference-module-sdk', '--no-default-features', '--features', 'hydrate', '--target', 'wasm32-unknown-unknown', '--locked')
        }
    }

    if ($Fast) {
        Invoke-CheckedStep -Label "API tests" -Command {
            # Database proofs intentionally fail when their dedicated URLs are
            # absent. Fast names every database-backed library test plus the
            # destructive integration test; the full gate executes all four
            # under their exact scopes.
            $arguments = @("test", "-p", "tessara-api", "--lib", "--locked", "--")
            Assert-TessaraCargoSkipFiltersExact `
                -CargoArguments @('test', '-p', 'tessara-api', '--lib', '--locked') `
                -SkipTests $fastValidationApiDatabaseSkipTests
            foreach ($testName in $fastValidationApiDatabaseSkipTests) {
                $arguments += @("--skip", $testName)
            }
            Invoke-TessaraCargo -Arguments $arguments
        }
    } else {
        Assert-TessaraValidationAmbientDatabaseIsolation
        $apiGeneralDatabaseBindings = [ordered]@{
            TEST_API_DATABASE_URL = [string]$validationEnvironment.TEST_API_DATABASE_URL
        }
        Invoke-TessaraWithProcessEnvironmentBindings `
            -Bindings $apiGeneralDatabaseBindings `
            -Action {
                Invoke-CheckedStep -Label "General API tests" -Command {
                    $arguments = @(
                        "test", "-p", "tessara-api", "--all-features", "--locked", "--"
                    )
                    Assert-TessaraCargoSkipFiltersExact `
                        -CargoArguments @(
                            'test', '-p', 'tessara-api', '--all-features', '--locked'
                        ) `
                        -SkipTests $fullValidationApiGeneralSkipTests
                    foreach ($testName in $fullValidationApiGeneralSkipTests) {
                        $arguments += @("--skip", $testName)
                    }
                    Invoke-TessaraCargo -Arguments $arguments
                }
            }
        Assert-TessaraValidationAmbientDatabaseIsolation

        Invoke-TessaraWithProcessEnvironmentBindings `
            -Bindings ([ordered]@{
                TEST_API_ENROLLMENT_DATABASE_URL =
                    [string]$validationEnvironment.TEST_API_ENROLLMENT_DATABASE_URL
            }) `
            -Action {
                Invoke-CheckedStep -Label "API enrollment database test" -Command {
                    Invoke-TessaraExactCargoTestProof `
                        -CargoArguments @(
                            'test', '-p', 'tessara-api', '--all-features', '--lib', '--locked'
                        ) `
                        -ExpectedTestName $fullValidationApiEnrollmentTest
                }
            }
        Assert-TessaraValidationAmbientDatabaseIsolation

        Invoke-TessaraWithDatabaseUrlBinding `
            -DatabaseUrl ([string]$validationEnvironment.TEST_SQLX_DATABASE_URL) `
            -Action {
                Invoke-CheckedStep -Label "API SQLx database test" -Command {
                    Invoke-TessaraExactCargoTestProof `
                        -CargoArguments @(
                            'test', '-p', 'tessara-api', '--all-features', '--lib', '--locked'
                        ) `
                        -ExpectedTestName $fullValidationApiSqlxTest
                }
            }
        Assert-TessaraValidationAmbientDatabaseIsolation
    }

    if (-not $Fast) {
        Invoke-TessaraWithProcessEnvironmentBindings `
            -Bindings ([ordered]@{
                TEST_API_DATABASE_URL = [string]$validationEnvironment.TEST_API_DATABASE_URL
            }) `
            -Action {
                Invoke-CheckedStep -Label "Release resource-reference timing proof" -Command {
                    Invoke-TessaraExactCargoTestProof `
                        -CargoArguments @(
                            'test', '-p', 'tessara-api', '--test', 'modules', '--release', '--locked'
                        ) `
                        -ExpectedTestName $fullValidationApiReleaseTimingTest `
                        -NoCapture
                }
            }
        Assert-TessaraValidationAmbientDatabaseIsolation
    }

    Invoke-CheckedStep -Label "Extracted Dataset module tests" -Command {
        if ($Fast) {
            Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--lib', '--locked')
        } else {
            if ([bool]$datasetTestPlan.has_lib_target) {
                Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--lib', '--locked')
            }
            if ([bool]$datasetTestPlan.has_bin_targets) {
                Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--bins', '--locked')
            }
            if ([bool]$datasetTestPlan.has_example_targets) {
                Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--examples', '--locked')
            }
            if ([bool]$datasetTestPlan.has_bench_targets) {
                Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--benches', '--locked')
            }
            if ([bool]$datasetTestPlan.has_doctest_target) {
                Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--doc', '--locked')
            }
            foreach ($target in @($datasetTestPlan.independent_target_names)) {
                Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--test', $target, '--locked')
            }
            Invoke-TessaraWithDatabaseUrlBinding `
                -DatabaseUrl ([string]$validationEnvironment.TEST_SQLX_DATABASE_URL) `
                -Action {
                    foreach ($target in @($datasetTestPlan.sqlx_target_names)) {
                        Invoke-TessaraCargo -Arguments @('test', '-p', 'tessara-dataset-module', '--all-features', '--test', $target, '--locked')
                    }
                }
        }
    }
    Assert-TessaraValidationAmbientDatabaseIsolation

    if (-not $Fast) {
        Invoke-TessaraWithProcessEnvironmentBindings `
            -Bindings ([ordered]@{
                TEST_API_FRESH_DATABASE_URL =
                    [string]$validationEnvironment.TEST_API_FRESH_DATABASE_URL
                SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET =
                    [string]$validationEnvironment.SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET
            }) `
            -Action {
                Invoke-CheckedStep -Label "Destructive fresh API baseline proof" -Command {
                    Invoke-TessaraExactCargoTestProof `
                        -CargoArguments @(
                            'test', '-p', 'tessara-api', '--all-features',
                            '--test', 'sprint_6a_populated_upgrade', '--locked'
                        ) `
                        -ExpectedTestName $fullValidationApiFreshTest
                }
            }
        Assert-TessaraValidationAmbientDatabaseIsolation
    }

} catch {
    $primaryFailure = $_
} finally {
    try {
        if ($null -ne $cargoPolicyState) {
            Exit-TessaraCargoBuildPolicy -State $cargoPolicyState `
                -CargoInvoker {
                    param([string[]]$Arguments)
                    Invoke-TessaraCargo -Arguments $Arguments
                }
        }
    } catch {
        $cleanupFailures.Add($_.Exception)
    } finally {
        $script:ActiveValidationCargoPolicyState = $null
    }
    try {
        Restore-TessaraProcessEnvironmentSnapshot `
            -Snapshot $cargoExecutionControlEnvironmentSnapshot `
            -IncludeCurrentNamePattern '^(?i:(?:RUSTC.*|RUSTDOC.*|RUSTFLAGS|RUSTUP_TOOLCHAIN|RUST_TEST_THREADS|CARGO_ENCODED_RUSTFLAGS|CARGO_TARGET_DIR|CARGO_INCREMENTAL|CARGO_PROFILE_.*|CARGO_BUILD_(?:TARGET|PROFILE|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|RUSTDOC|RUSTFLAGS|RUSTDOCFLAGS)|CARGO_TARGET_[A-Z0-9_]+_(?:RUNNER|LINKER|RUSTFLAGS|RUSTDOCFLAGS)))$'
    } catch {
        $cleanupFailures.Add($_.Exception)
    }
    try {
        Restore-TessaraProcessEnvironmentSnapshot -Snapshot $databaseUrlSnapshot
    } catch {
        $cleanupFailures.Add($_.Exception)
    }
    try {
        Restore-TessaraProcessEnvironmentSnapshot `
            -Snapshot $postgresEnvironmentSnapshot `
            -IncludeCurrentNamePattern '^(?i:PG)'
    } catch {
        $cleanupFailures.Add($_.Exception)
    }
    try {
        Restore-TessaraProcessEnvironmentSnapshot `
            -Snapshot $validationDatabaseEnvironmentSnapshot
    } catch {
        $cleanupFailures.Add($_.Exception)
    }
    if ($locationPushed) {
        try {
            Pop-Location
        } catch {
            $cleanupFailures.Add($_.Exception)
        }
    }
    $script:ActiveValidationCargoBinding = $null
    $script:ActiveValidationPsqlBinding = $null
    $script:InitialValidationCargoControlValues = [ordered]@{}
}
Throw-TessaraPrimaryOrCleanupFailure `
    -PrimaryFailure $primaryFailure `
    -CleanupFailures $cleanupFailures `
    -Boundary "Tessara validation"
Write-Host "`nValidation passed." -ForegroundColor Green

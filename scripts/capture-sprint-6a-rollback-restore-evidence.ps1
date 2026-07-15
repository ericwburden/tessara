[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$SourceDatabaseUrl,
    [Parameter(Mandatory)][string]$ExpectedSourceDatabaseName,
    [Parameter(Mandatory)][string]$MaintenanceDatabaseUrl,
    [Parameter(Mandatory)][string]$TargetDatabaseUrl,
    [Parameter(Mandatory)][string]$ExpectedTargetDatabaseName,
    [Parameter(Mandatory)][string]$BackupPath,
    [Parameter(Mandatory)][string]$EvidencePath,
    [string]$PsqlCommand = "psql",
    [string]$PgDumpCommand = "pg_dump",
    [string]$PgRestoreCommand = "pg_restore",
    [string]$PostgresClientContainerId,
    [string]$DestructiveResetAcknowledgement = $env:SPRINT_6A_CONFIRM_DESTRUCTIVE_RESTORE_RESET
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

. (Join-Path $PSScriptRoot "sprint-6a-rollback-restore-evidence-common.ps1")

$requiredAcknowledgement = "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"
if ($DestructiveResetAcknowledgement -cne $requiredAcknowledgement) {
    throw "Set SPRINT_6A_CONFIRM_DESTRUCTIVE_RESTORE_RESET=$requiredAcknowledgement or pass the exact acknowledgement before recreating the restored target database."
}
if ([string]::IsNullOrWhiteSpace($ExpectedSourceDatabaseName)) {
    throw "Expected source database name must not be empty."
}
if ([string]::IsNullOrWhiteSpace($ExpectedTargetDatabaseName)) {
    throw "Expected target database name must not be empty."
}
if ($ExpectedSourceDatabaseName -ceq $ExpectedTargetDatabaseName) {
    throw "Source and restored target database names must be distinct."
}
if (-not (Test-Sprint6ADisposableDatabaseName $ExpectedTargetDatabaseName)) {
    throw "Restored target database '$ExpectedTargetDatabaseName' is not clearly disposable; include a standalone test, upgrade, clone, rollback, or sprint-6a token."
}
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$backupFullPath = if ([IO.Path]::IsPathRooted($BackupPath)) {
    [IO.Path]::GetFullPath($BackupPath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repositoryRoot $BackupPath))
}
$evidenceFullPath = if ([IO.Path]::IsPathRooted($EvidencePath)) {
    [IO.Path]::GetFullPath($EvidencePath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repositoryRoot $EvidencePath))
}
if ($backupFullPath -eq $evidenceFullPath) {
    throw "Backup artifact and restore evidence paths must be distinct."
}
if (Test-Path -LiteralPath $backupFullPath) {
    throw "Refusing to overwrite retained backup artifact '$backupFullPath'."
}
if (Test-Path -LiteralPath $evidenceFullPath) {
    throw "Refusing to overwrite retained restore evidence '$evidenceFullPath'."
}
$backupParent = Split-Path -Parent $backupFullPath
$evidenceParent = Split-Path -Parent $evidenceFullPath
New-Item -ItemType Directory -Path $backupParent -Force | Out-Null
New-Item -ItemType Directory -Path $evidenceParent -Force | Out-Null

$postgresClientContext = New-Sprint6APostgresClientContext `
    -DatabaseUrls ([string[]]@($SourceDatabaseUrl, $MaintenanceDatabaseUrl, $TargetDatabaseUrl)) `
    -PostgresClientContainerId $PostgresClientContainerId `
    -PsqlCommand $PsqlCommand `
    -PgDumpCommand $PgDumpCommand `
    -PgRestoreCommand $PgRestoreCommand `
    -RequiredTools ([string[]]@("psql", "pg_dump", "pg_restore"))

$sourceUrlEndpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
    -DatabaseUrl $SourceDatabaseUrl `
    -Context "Source database URL"
Assert-Sprint6AExactDatabaseName `
    -Actual $sourceUrlEndpoint.database_name `
    -Expected $ExpectedSourceDatabaseName `
    -Context "Source database URL"
$maintenanceUrlEndpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
    -DatabaseUrl $MaintenanceDatabaseUrl `
    -Context "Maintenance database URL"
if ($maintenanceUrlEndpoint.database_name -ceq $ExpectedTargetDatabaseName) {
    throw "Maintenance database URL must not name the exact restored target '$ExpectedTargetDatabaseName'."
}
$targetUrlEndpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
    -DatabaseUrl $TargetDatabaseUrl `
    -Context "Target database URL"
Assert-Sprint6AExactDatabaseName `
    -Actual $targetUrlEndpoint.database_name `
    -Expected $ExpectedTargetDatabaseName `
    -Context "Target database URL"

function Get-MigrationLedger {
    param([Parameter(Mandatory)][string]$DatabaseUrl)

    $lines = @(
        Invoke-Sprint6APsqlLines `
            -DatabaseUrl $DatabaseUrl `
            -PsqlCommand $PsqlCommand `
            -PostgresClientContext $postgresClientContext `
            -Sql "SELECT version::text FROM _sqlx_migrations WHERE success ORDER BY version;"
    )
    return @($lines | Where-Object { $_ -ne "" } | ForEach-Object { [int64]$_ })
}

function Assert-PreUpgradeLedger {
    param(
        [Parameter(Mandatory)][long[]]$Ledger,
        [Parameter(Mandatory)][string]$Context
    )

    if (($Ledger -join ",") -ne "1,2") {
        throw "$Context migration ledger '$($Ledger -join ',')' does not equal the required pre-upgrade ledger '1,2'."
    }
}

function Invoke-AdministrativePsql {
    param(
        [Parameter(Mandatory)][string[]]$Sql,
        [hashtable]$Variables = @{}
    )

    $arguments = [string[]]@(
        "--no-psqlrc",
        "--no-password",
        "--set=ON_ERROR_STOP=1",
        "--set=VERBOSITY=terse",
        "--set=SHOW_CONTEXT=never",
        "--quiet"
    )
    foreach ($name in @($Variables.Keys | Sort-Object)) {
        if ($name -notmatch '^[a-z_][a-z0-9_]*$' -or $null -eq $Variables[$name]) {
            throw "Administrative psql received an invalid variable."
        }
        $arguments += "--set=$name=$($Variables[$name])"
    }
    $sqlPath = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-6a-reset-$PID-$([Guid]::NewGuid().ToString('N')).sql"
    try {
        $scriptText = "SET client_min_messages TO warning;`n" + ($Sql -join "`n") + "`n"
        [IO.File]::WriteAllText($sqlPath, $scriptText, [Text.UTF8Encoding]::new($false))
        [void](Invoke-Sprint6APostgresClientTool `
            -Tool "psql" `
            -DatabaseUrl $MaintenanceDatabaseUrl `
            -Arguments $arguments `
            -PostgresClientContext $postgresClientContext `
            -Context "administrative psql command" `
            -StandardInputPath $sqlPath)
    } finally {
        if (Test-Path -LiteralPath $sqlPath -PathType Leaf) {
            $resolvedSqlPath = [IO.Path]::GetFullPath($sqlPath)
            $tempPrefix = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
            if (-not $resolvedSqlPath.StartsWith($tempPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Refusing to remove unexpected administrative SQL path '$resolvedSqlPath'."
            }
            Remove-Item -LiteralPath $resolvedSqlPath -Force
        }
    }
}

$sourceDatabaseName = Invoke-Sprint6APsqlScalar `
    -DatabaseUrl $SourceDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext `
    -Sql "SELECT current_database();"
Assert-Sprint6AExactDatabaseName `
    -Actual $sourceDatabaseName `
    -Expected $ExpectedSourceDatabaseName `
    -Context "Source database connection"
$maintenanceDatabaseName = Invoke-Sprint6APsqlScalar `
    -DatabaseUrl $MaintenanceDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext `
    -Sql "SELECT current_database();"
Assert-Sprint6AExactDatabaseName `
    -Actual $maintenanceDatabaseName `
    -Expected $maintenanceUrlEndpoint.database_name `
    -Context "Maintenance database connection"
if ($maintenanceDatabaseName -ceq $ExpectedTargetDatabaseName) {
    throw "Maintenance database URL must not connect to restored target '$ExpectedTargetDatabaseName'."
}
$maintenanceSystemIdentifier = Invoke-Sprint6APsqlScalar `
    -DatabaseUrl $MaintenanceDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext `
    -Sql "SELECT system_identifier::text FROM pg_control_system();"

$sourceLedgerBefore = @(Get-MigrationLedger $SourceDatabaseUrl)
Assert-PreUpgradeLedger -Ledger $sourceLedgerBefore -Context "Source database before backup"
$sourceFingerprintBefore = Get-Sprint6ADatabaseFingerprint `
    -DatabaseUrl $SourceDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext

Invoke-Sprint6APgDumpCapture `
    -DatabaseUrl $SourceDatabaseUrl `
    -OutputPath $backupFullPath `
    -PostgresClientContext $postgresClientContext
if (-not (Test-Path -LiteralPath $backupFullPath -PathType Leaf)) {
    throw "pg_dump did not create retained backup artifact '$backupFullPath'."
}
$backupFile = Get-Item -LiteralPath $backupFullPath
if ($backupFile.Length -le 0) {
    throw "pg_dump created an empty retained backup artifact."
}
$backupSha256 = Get-Sprint6AFileSha256 $backupFullPath

$sourceLedgerAfter = @(Get-MigrationLedger $SourceDatabaseUrl)
Assert-PreUpgradeLedger -Ledger $sourceLedgerAfter -Context "Source database after backup"
$sourceFingerprintAfter = Get-Sprint6ADatabaseFingerprint `
    -DatabaseUrl $SourceDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext
if (($sourceLedgerBefore -join ",") -ne ($sourceLedgerAfter -join ",") -or
    ($sourceFingerprintBefore | ConvertTo-Json -Depth 4 -Compress) -ne
        ($sourceFingerprintAfter | ConvertTo-Json -Depth 4 -Compress)) {
    throw "Source database changed while the retained backup artifact was captured; retry from a quiescent pre-upgrade source."
}

$resetVariables = @{ target_database = $ExpectedTargetDatabaseName }
$resetStatements = @(Get-Sprint6ADatabaseResetStatements)
Invoke-AdministrativePsql -Sql $resetStatements -Variables $resetVariables

$targetDatabaseName = Invoke-Sprint6APsqlScalar `
    -DatabaseUrl $TargetDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext `
    -Sql "SELECT current_database();"
Assert-Sprint6AExactDatabaseName `
    -Actual $targetDatabaseName `
    -Expected $ExpectedTargetDatabaseName `
    -Context "Restored target database connection"
$targetSystemIdentifier = Invoke-Sprint6APsqlScalar `
    -DatabaseUrl $TargetDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext `
    -Sql "SELECT system_identifier::text FROM pg_control_system();"
if ($targetSystemIdentifier -ne $maintenanceSystemIdentifier) {
    throw "Target database URL does not connect to the PostgreSQL cluster where the disposable target was recreated."
}

Invoke-Sprint6APgRestoreVerification `
    -DatabaseUrl $TargetDatabaseUrl `
    -InputPath $backupFullPath `
    -PostgresClientContext $postgresClientContext

$targetLedger = @(Get-MigrationLedger $TargetDatabaseUrl)
Assert-PreUpgradeLedger -Ledger $targetLedger -Context "Restored target database"
$targetFingerprint = Get-Sprint6ADatabaseFingerprint `
    -DatabaseUrl $TargetDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext
$sourceFingerprintJson = $sourceFingerprintAfter | ConvertTo-Json -Depth 4 -Compress
$targetFingerprintJson = $targetFingerprint | ConvertTo-Json -Depth 4 -Compress
if ($sourceFingerprintJson -ne $targetFingerprintJson) {
    throw "Restored target database fingerprint does not exactly match the pre-upgrade source database fingerprint."
}
$sourceLedgerFinal = @(Get-MigrationLedger $SourceDatabaseUrl)
Assert-PreUpgradeLedger -Ledger $sourceLedgerFinal -Context "Source database after restore verification"
$sourceFingerprintFinal = Get-Sprint6ADatabaseFingerprint `
    -DatabaseUrl $SourceDatabaseUrl `
    -PsqlCommand $PsqlCommand `
    -PostgresClientContext $postgresClientContext
if (($sourceLedgerAfter -join ",") -ne ($sourceLedgerFinal -join ",") -or
    $sourceFingerprintJson -ne ($sourceFingerprintFinal | ConvertTo-Json -Depth 4 -Compress)) {
    throw "Source database changed during target restore verification; retry from a quiescent pre-upgrade source."
}

$relativeBackupPath = [IO.Path]::GetRelativePath($evidenceParent, $backupFullPath).Replace('\', '/')
$archiveTransfer = if ($postgresClientContext.mode -eq "docker_container") {
    "docker_cp_out_and_execution_user_stdin_in_unique_container_temp_paths_with_finally_cleanup"
} else {
    "direct_host_path"
}
$sanitizedCommands = if ($postgresClientContext.mode -eq "docker_container") {
    @(
        [ordered]@{ tool = "psql"; arguments = @("<postgres-client-container-id>", "<source-database-url>", "<read-only-name-ledger-fingerprint-verification>") },
        [ordered]@{ tool = "pg_dump"; arguments = @("<postgres-client-container-id>", "--dbname", "<source-database-url>", "--format=custom", "--no-owner", "--no-privileges", "--file", "<unique-container-dump-path>") },
        [ordered]@{ tool = "docker"; arguments = @("cp", "<postgres-client-container-id>:<unique-container-dump-path>", "<retained-backup-path>") },
        [ordered]@{ tool = "psql"; arguments = @("<postgres-client-container-id>", "<maintenance-database-url>", "<terminate-drop-create-disposable-target>", "<target-database-name>") },
        [ordered]@{ tool = "docker"; arguments = @("exec", "--interactive", "<postgres-client-container-id>", "<retained-backup-stdin>", "<unique-container-dump-path>") },
        [ordered]@{ tool = "pg_restore"; arguments = @("<postgres-client-container-id>", "--dbname", "<target-database-url>", "--no-owner", "--no-privileges", "--exit-on-error", "<unique-container-dump-path>") },
        [ordered]@{ tool = "psql"; arguments = @("<postgres-client-container-id>", "<target-database-url>", "<read-only-name-ledger-fingerprint-verification>") }
    )
} else {
    @(
        [ordered]@{ tool = "psql"; arguments = @("<source-database-connection-environment>", "<read-only-name-ledger-fingerprint-verification>") },
        [ordered]@{ tool = "pg_dump"; arguments = @("<source-database-connection-environment>", "--format=custom", "--no-owner", "--no-privileges", "--file", "<retained-backup-path>") },
        [ordered]@{ tool = "psql"; arguments = @("<maintenance-database-connection-environment>", "<terminate-drop-create-disposable-target>", "<target-database-name>") },
        [ordered]@{ tool = "pg_restore"; arguments = @("<target-database-connection-environment>", "--no-owner", "--no-privileges", "--exit-on-error", "<retained-backup-path>") },
        [ordered]@{ tool = "psql"; arguments = @("<target-database-connection-environment>", "<read-only-name-ledger-fingerprint-verification>") }
    )
}
$evidence = [ordered]@{
    schema_version = $script:Sprint6ARestoreEvidenceSchemaVersion
    evidence_kind = "tessara_sprint_6a_pre_upgrade_backup_restore_proof"
    generated_at_utc = [DateTime]::UtcNow.ToString("o")
    generator = [ordered]@{
        script_name = [IO.Path]::GetFileName($PSCommandPath)
        script_sha256 = Get-Sprint6AFileSha256 $PSCommandPath
        common_helper_name = $script:Sprint6ARestoreEvidenceCommonHelperName
        common_helper_sha256 = Get-Sprint6AFileSha256 $script:Sprint6ARestoreEvidenceCommonHelperPath
        powershell_version = $PSVersionTable.PSVersion.ToString()
    }
    postgres_client = Get-Sprint6APostgresClientEvidence $postgresClientContext
    backup_artifact = [ordered]@{
        path_relative_to_evidence = $relativeBackupPath
        sha256 = $backupSha256
        length_bytes = $backupFile.Length
        format = "postgresql_custom_archive"
        source_database_name = $sourceDatabaseName
    }
    source_database = [ordered]@{
        database_name = $sourceDatabaseName
        migration_ledger = @($sourceLedgerFinal)
        fingerprint = $sourceFingerprintFinal
    }
    restored_target_database = [ordered]@{
        database_name = $targetDatabaseName
        migration_ledger = @($targetLedger)
        fingerprint = $targetFingerprint
    }
    restore_operation = [ordered]@{
        target_was_destructively_recreated = $true
        backup_sha256_used = $backupSha256
        credential_redaction = "all_database_urls_replaced_with_named_placeholders"
        archive_transfer = $archiveTransfer
        commands = @($sanitizedCommands)
    }
    result = "passed"
}
Write-Sprint6AUtf8Json -Value $evidence -Path $evidenceFullPath
[void](Assert-Sprint6ARestoreEvidenceDocument `
    -EvidencePath $evidenceFullPath `
    -ExpectedTargetDatabaseName $ExpectedTargetDatabaseName)

Write-Host "Retained pre-upgrade backup and restore evidence captured successfully."
Write-Host "Backup: $backupFullPath"
Write-Host "Evidence: $evidenceFullPath"

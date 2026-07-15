[CmdletBinding()]
param(
    [ValidateSet("PackageOnly", "CompatibilityOnUpgraded", "OriginalAfterRestore")]
    [string]$Mode = "PackageOnly",
    [string]$PackagePath = "artifacts/sprint-6a/compatibility-rollback",
    [ValidatePattern('^[0-9a-fA-F]{40}$')]
    [string]$ExpectedClosingSprint6ACommit,
    [string]$DatabaseUrl,
    [string]$ExpectedDatabaseName,
    [string]$RestoreEvidencePath,
    [string]$BindAddress = "127.0.0.1:18086",
    [string]$PsqlCommand = "psql",
    [string]$PgDumpCommand = "pg_dump",
    [string]$PgRestoreCommand = "pg_restore",
    [string]$PostgresClientContainerId,
    [string]$DevAdminEmail = "admin@tessara.local",
    [string]$DevAdminPassword = "sprint-6a-rollback-proof",
    [string]$EvidencePath,
    [switch]$SelfTest
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

. (Join-Path $PSScriptRoot "sprint-6a-rollback-restore-evidence-common.ps1")

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$packageFullPath = if ([IO.Path]::IsPathRooted($PackagePath)) {
    [IO.Path]::GetFullPath($PackagePath)
} else {
    [IO.Path]::GetFullPath((Join-Path $repositoryRoot $PackagePath))
}

function New-BuiltInSeedSnapshot {
    param([Parameter(Mandatory)][object[]]$Mappings)

    $grouped = [Collections.Generic.Dictionary[string, Collections.Generic.List[string]]]::new(
        [StringComparer]::Ordinal
    )
    foreach ($mapping in $Mappings) {
        $roleName = [string]$mapping.role_name
        $capabilityKey = [string]$mapping.capability_key
        if ([string]::IsNullOrWhiteSpace($roleName) -or [string]::IsNullOrWhiteSpace($capabilityKey)) {
            throw "Built-in seed snapshots require non-empty role and capability keys."
        }
        if (-not $grouped.ContainsKey($roleName)) {
            $grouped.Add($roleName, [Collections.Generic.List[string]]::new())
        }
        $grouped[$roleName].Add($capabilityKey)
    }

    $roleNames = [string[]]@($grouped.Keys)
    [Array]::Sort($roleNames, [StringComparer]::Ordinal)
    $orderedMappings = @()
    $canonicalText = ""
    foreach ($roleName in $roleNames) {
        $canonicalText += "role=$roleName`n"
        $capabilityKeys = [string[]]@($grouped[$roleName])
        [Array]::Sort($capabilityKeys, [StringComparer]::Ordinal)
        foreach ($capabilityKey in $capabilityKeys) {
            $orderedMappings += [ordered]@{
                role_name = $roleName
                capability_key = $capabilityKey
            }
            $canonicalText += "capability=$capabilityKey`n"
        }
    }

    return [pscustomobject][ordered]@{
        canonical_format = "utf8_ordinal_role_then_capability_role_header_lf_v1"
        canonical_sha256 = Get-StringSha256 $canonicalText
        mapping_count = $orderedMappings.Count
        mappings = @($orderedMappings)
    }
}

function New-DeclaredBuiltInSeedContract {
    param(
        [Parameter(Mandatory)][string]$ContractId,
        [string]$SourceCommit,
        [Parameter(Mandatory)]
        [ValidateSet("declaration_order", "sorted_mapping_set")]
        [string]$ContractCanonicalMode,
        [Parameter(Mandatory)][string]$ExpectedContractSha256,
        [Parameter(Mandatory)][Collections.Specialized.OrderedDictionary]$RoleCapabilities
    )

    $mappings = @()
    $declarationCanonical = ""
    foreach ($roleName in $RoleCapabilities.Keys) {
        $declarationCanonical += "role=$roleName`n"
        foreach ($capabilityKey in @($RoleCapabilities[$roleName])) {
            $declarationCanonical += "capability=$capabilityKey`n"
            $mappings += [ordered]@{
                role_name = [string]$roleName
                capability_key = [string]$capabilityKey
            }
        }
    }
    $snapshot = New-BuiltInSeedSnapshot $mappings
    $contractCanonicalFormat = if ($ContractCanonicalMode -eq "declaration_order") {
        "utf8_declared_role_then_ordered_capability_lf_v1"
    } else {
        $snapshot.canonical_format
    }
    $contractSha256 = if ($ContractCanonicalMode -eq "declaration_order") {
        Get-StringSha256 $declarationCanonical
    } else {
        $snapshot.canonical_sha256
    }
    if ($contractSha256 -ne $ExpectedContractSha256) {
        throw "Declared seed contract '$ContractId' does not match expected canonical digest '$ExpectedContractSha256'."
    }
    return [pscustomobject][ordered]@{
        contract_id = $ContractId
        source_commit = $SourceCommit
        contract_canonical_format = $contractCanonicalFormat
        contract_sha256 = $contractSha256
        snapshot = $snapshot
    }
}

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)

    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-StringSha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Value)

    $utf8 = [Text.UTF8Encoding]::new($false)
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $hex = [BitConverter]::ToString($algorithm.ComputeHash($utf8.GetBytes($Value)))
        return $hex.Replace("-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Test-DisposableDatabaseName {
    param([Parameter(Mandatory)][string]$Name)

    return Test-Sprint6ADisposableDatabaseName $Name
}

function Select-HistoricalFormScope {
    param([Parameter(Mandatory)][object[]]$Forms)

    foreach ($form in $Forms) {
        $formIdProperty = $form.PSObject.Properties["id"]
        $scopeNodeTypeIdProperty = $form.PSObject.Properties["scope_node_type_id"]
        $visibilityNodesProperty = $form.PSObject.Properties["visibility_nodes"]
        if ($null -eq $formIdProperty -or
            $null -eq $scopeNodeTypeIdProperty -or
            $null -eq $visibilityNodesProperty) {
            continue
        }

        $formId = [Guid]::Empty
        $scopeNodeTypeId = [Guid]::Empty
        if (-not [Guid]::TryParse([string]$formIdProperty.Value, [ref]$formId) -or
            -not [Guid]::TryParse([string]$scopeNodeTypeIdProperty.Value, [ref]$scopeNodeTypeId)) {
            continue
        }

        foreach ($visibilityNode in @($visibilityNodesProperty.Value)) {
            if ($null -eq $visibilityNode) {
                continue
            }
            $visibilityNodeIdProperty = $visibilityNode.PSObject.Properties["node_id"]
            if ($null -eq $visibilityNodeIdProperty) {
                continue
            }
            $visibilityNodeId = [Guid]::Empty
            if (-not [Guid]::TryParse([string]$visibilityNodeIdProperty.Value, [ref]$visibilityNodeId)) {
                continue
            }
            return [pscustomobject][ordered]@{
                source_form_id = $formId.ToString("D")
                scope_node_type_id = $scopeNodeTypeId.ToString("D")
                visibility_node_id = $visibilityNodeId.ToString("D")
            }
        }
    }

    throw "Historical product read did not return a form with a valid scope node type and visibility node."
}

function Assert-HistoricalFormScopeProjection {
    param(
        [Parameter(Mandatory)]$Form,
        [Parameter(Mandatory)][string]$ExpectedScopeNodeTypeId,
        [Parameter(Mandatory)][string]$ExpectedVisibilityNodeId,
        [Parameter(Mandatory)][string]$Context
    )

    $scopeNodeTypeIdProperty = $Form.PSObject.Properties["scope_node_type_id"]
    $visibilityNodesProperty = $Form.PSObject.Properties["visibility_nodes"]
    if ($null -eq $scopeNodeTypeIdProperty -or
        [string]$scopeNodeTypeIdProperty.Value -cne $ExpectedScopeNodeTypeId) {
        throw "$Context did not retain the selected scope node type exactly."
    }
    if ($null -eq $visibilityNodesProperty) {
        throw "$Context did not return visibility nodes."
    }
    $visibilityNodes = @($visibilityNodesProperty.Value)
    if ($visibilityNodes.Count -ne 1) {
        throw "$Context did not retain exactly one selected visibility node."
    }
    $visibilityNodeIdProperty = $visibilityNodes[0].PSObject.Properties["node_id"]
    if ($null -eq $visibilityNodeIdProperty -or
        [string]$visibilityNodeIdProperty.Value -cne $ExpectedVisibilityNodeId) {
        throw "$Context did not retain the selected visibility node exactly."
    }
}

function New-SanitizedLogEvidence {
    param(
        [Parameter(Mandatory)][string]$StdoutPath,
        [Parameter(Mandatory)][string]$StderrPath,
        [Parameter(Mandatory)][string]$DatabaseUrl,
        [Parameter(Mandatory)][string]$DevAdminPassword,
        [string[]]$AdditionalSecrets = @()
    )

    $databaseEndpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
        -DatabaseUrl $DatabaseUrl `
        -Context "Rollback-process log-redaction database URL"
    $declaredSecrets = [string[]]@(
        $DatabaseUrl,
        $databaseEndpoint.database_password,
        $DevAdminPassword
    ) + $AdditionalSecrets
    $passwordAssignmentPattern = '(?im)(?<prefix>["'']?(?:password|passwd|pwd|pgpassword)["'']?\s*[:=]\s*)(?<value>"[^"\r\n]*"|''[^''\r\n]*''|[^\s,;]+)'
    $passwordAssignmentRedactor = [Text.RegularExpressions.MatchEvaluator]{
        param([Text.RegularExpressions.Match]$Match)

        return $Match.Groups["prefix"].Value + "<redacted-secret>"
    }

    function New-SanitizedLogStreamEvidence {
        param([Parameter(Mandatory)][string]$Path)

        $content = if (Test-Path -LiteralPath $Path -PathType Leaf) {
            [IO.File]::ReadAllText($Path, [Text.UTF8Encoding]::new($false))
        } else {
            ""
        }
        foreach ($secret in $declaredSecrets) {
            if (-not [string]::IsNullOrEmpty($secret)) {
                $content = $content.Replace($secret, "<redacted-secret>")
            }
        }
        $content = [Text.RegularExpressions.Regex]::Replace(
            $content,
            '(?i)\bpostgres(?:ql)?://[^\s''"]+',
            '<redacted-postgres-url>'
        )
        $content = [Text.RegularExpressions.Regex]::Replace(
            $content,
            '(?i)\bbearer\s+[^\s''"]+',
            'Bearer <redacted-token>'
        )
        $content = [Text.RegularExpressions.Regex]::Replace(
            $content,
            $passwordAssignmentPattern,
            $passwordAssignmentRedactor
        )
        foreach ($secret in $declaredSecrets) {
            if (-not [string]::IsNullOrEmpty($secret) -and $content.Contains($secret)) {
                throw "Sanitized rollback-process logs still contain a declared secret."
            }
        }
        if ($content -match '(?i)\bpostgres(?:ql)?://[^\s''"]+@') {
            throw "Sanitized rollback-process logs still contain a credential-bearing PostgreSQL URL."
        }
        foreach ($assignment in [Text.RegularExpressions.Regex]::Matches($content, $passwordAssignmentPattern)) {
            if ($assignment.Groups["value"].Value -cne "<redacted-secret>") {
                throw "Sanitized rollback-process logs still contain a normalized password assignment."
            }
        }
        return [ordered]@{
            encoding = "utf8"
            content = $content
            length_bytes = [Text.UTF8Encoding]::new($false).GetByteCount($content)
            sha256 = Get-StringSha256 $content
        }
    }

    return [ordered]@{
        lifecycle = "process_stopped_before_final_log_capture"
        stdout = New-SanitizedLogStreamEvidence $StdoutPath
        stderr = New-SanitizedLogStreamEvidence $StderrPath
    }
}

$sprint5ACapabilities = @(
    "admin:all",
    "hierarchy:read",
    "hierarchy:manage",
    "forms:read",
    "forms:manage",
    "workflows:read",
    "workflows:manage",
    "submissions:read_own",
    "submissions:respond",
    "submissions:manage",
    "analytics:refresh",
    "operations:view",
    "datasets:manage",
    "datasets:read",
    "datasets:read_restricted",
    "datasets:read_confidential",
    "components:manage",
    "components:read",
    "dashboards:manage",
    "dashboards:read"
)
$operatorCapabilities = @(
    "hierarchy:read",
    "forms:read",
    "workflows:read",
    "workflows:manage",
    "submissions:respond",
    "submissions:manage",
    "operations:view",
    "datasets:read",
    "components:read",
    "dashboards:read"
)
$respondentCapabilities = @("submissions:read_own", "submissions:respond")
$sprint6ASeedContract = New-DeclaredBuiltInSeedContract `
    -ContractId "sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68" `
    -ContractCanonicalMode "declaration_order" `
    -ExpectedContractSha256 "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600" `
    -RoleCapabilities ([ordered]@{
        admin = @("admin:all")
        operator = $operatorCapabilities
        respondent = $respondentCapabilities
    })
$sprint5ASeedContract = New-DeclaredBuiltInSeedContract `
    -ContractId "sprint-5a-role-capabilities-v1+sha256.7725e889996a" `
    -SourceCommit "3625d4de52c5856e4ac3bc642a9422a029e9f375" `
    -ContractCanonicalMode "sorted_mapping_set" `
    -ExpectedContractSha256 "7725e889996a73a5655c57106aca6e12d9a5f95e9103f14d7b0fd50fbac96988" `
    -RoleCapabilities ([ordered]@{
        admin = $sprint5ACapabilities
        operator = $operatorCapabilities
        respondent = $respondentCapabilities
    })

if ($SelfTest) {
    if ($sprint6ASeedContract.snapshot.mapping_count -ne 13) {
        throw "Self-test expected 13 Sprint 6A built-in seed mappings."
    }
    if ($sprint5ASeedContract.snapshot.mapping_count -ne 32) {
        throw "Self-test expected 32 Sprint 5A built-in seed mappings."
    }
    if ($sprint5ASeedContract.source_commit -notmatch '^[0-9a-f]{40}$') {
        throw "Self-test found an invalid Sprint 5A contract source commit."
    }

    $reversedMappings = @($sprint5ASeedContract.snapshot.mappings)
    [Array]::Reverse($reversedMappings)
    $reversedSnapshot = New-BuiltInSeedSnapshot $reversedMappings
    if ($reversedSnapshot.canonical_sha256 -ne $sprint5ASeedContract.snapshot.canonical_sha256) {
        throw "Self-test found order-dependent built-in seed canonicalization."
    }

    $tamperedMappings = @($sprint6ASeedContract.snapshot.mappings | ForEach-Object {
        [ordered]@{ role_name = $_.role_name; capability_key = $_.capability_key }
    })
    $tamperedMappings[0].capability_key = "$($tamperedMappings[0].capability_key):tampered"
    $tamperedSnapshot = New-BuiltInSeedSnapshot $tamperedMappings
    if ($tamperedSnapshot.canonical_sha256 -eq $sprint6ASeedContract.snapshot.canonical_sha256) {
        throw "Self-test did not detect a changed built-in seed mapping."
    }

    $historicalFormId = "10000000-0000-0000-0000-000000000001"
    $historicalScopeNodeTypeId = "20000000-0000-0000-0000-000000000002"
    $historicalVisibilityNodeId = "30000000-0000-0000-0000-000000000003"
    $historicalScope = Select-HistoricalFormScope ([object[]]@(
        [pscustomobject][ordered]@{
            id = "not-a-uuid"
            scope_node_type_id = $historicalScopeNodeTypeId
            visibility_nodes = @([pscustomobject]@{ node_id = $historicalVisibilityNodeId })
        },
        [pscustomobject][ordered]@{
            id = $historicalFormId
            scope_node_type_id = $historicalScopeNodeTypeId
            visibility_nodes = @([pscustomobject]@{ node_id = $historicalVisibilityNodeId })
        }
    ))
    if ($historicalScope.source_form_id -cne $historicalFormId -or
        $historicalScope.scope_node_type_id -cne $historicalScopeNodeTypeId -or
        $historicalScope.visibility_node_id -cne $historicalVisibilityNodeId) {
        throw "Self-test did not select the valid historical form scope exactly."
    }
    $scopePayload = @{
        scope_node_type_id = $historicalScope.scope_node_type_id
        visibility_node_ids = @($historicalScope.visibility_node_id)
    } | ConvertTo-Json | ConvertFrom-Json
    $roundTrippedVisibilityNodeIds = @($scopePayload.visibility_node_ids)
    if ([string]$scopePayload.scope_node_type_id -cne $historicalScopeNodeTypeId -or
        $roundTrippedVisibilityNodeIds.Count -ne 1 -or
        [string]$roundTrippedVisibilityNodeIds[0] -cne $historicalVisibilityNodeId) {
        throw "Self-test did not retain the historical scope as one explicit visibility-node JSON value."
    }
    Assert-HistoricalFormScopeProjection `
        -Form ([pscustomobject][ordered]@{
            scope_node_type_id = $historicalScopeNodeTypeId
            visibility_nodes = @([pscustomobject]@{ node_id = $historicalVisibilityNodeId })
        }) `
        -ExpectedScopeNodeTypeId $historicalScopeNodeTypeId `
        -ExpectedVisibilityNodeId $historicalVisibilityNodeId `
        -Context "Historical form-scope self-test"
    $invalidHistoricalScopeRejected = $false
    try {
        Select-HistoricalFormScope ([object[]]@(
            [pscustomobject][ordered]@{
                id = $historicalFormId
                scope_node_type_id = $historicalScopeNodeTypeId
                visibility_nodes = @([pscustomobject]@{ node_id = "not-a-uuid" })
            }
        )) | Out-Null
    } catch {
        if ($_.Exception.Message -notmatch "valid scope node type and visibility node") {
            throw
        }
        $invalidHistoricalScopeRejected = $true
    }
    if (-not $invalidHistoricalScopeRejected) {
        throw "Self-test accepted a malformed historical visibility node."
    }
    $invalidHistoricalProjectionRejected = $false
    try {
        Assert-HistoricalFormScopeProjection `
            -Form ([pscustomobject][ordered]@{
                scope_node_type_id = $historicalScopeNodeTypeId
                visibility_nodes = @()
            }) `
            -ExpectedScopeNodeTypeId $historicalScopeNodeTypeId `
            -ExpectedVisibilityNodeId $historicalVisibilityNodeId `
            -Context "Invalid historical form-scope self-test"
    } catch {
        if ($_.Exception.Message -notmatch "exactly one selected visibility node") {
            throw
        }
        $invalidHistoricalProjectionRejected = $true
    }
    if (-not $invalidHistoricalProjectionRejected) {
        throw "Self-test accepted a historical read-after-write projection without a visibility node."
    }

    foreach ($acceptedName in @(
        "tessara_sprint6a_test",
        "tessara_sprint6a_upgrade_test",
        "tessara-rollback-clone"
    )) {
        if (-not (Test-DisposableDatabaseName $acceptedName)) {
            throw "Self-test rejected disposable database name '$acceptedName'."
        }
    }
    foreach ($rejectedName in @("tessara", "tessara_latest", "contest", "production_upgradeable")) {
        if (Test-DisposableDatabaseName $rejectedName) {
            throw "Self-test accepted unsafe database name '$rejectedName'."
        }
    }

    $ipv4Binding = [pscustomobject][ordered]@{ HostIp = "127.0.0.1"; HostPort = "55432" }
    $containerEndpoint = ConvertTo-Sprint6AContainerPostgresEndpoint `
        -DatabaseUrl "postgres://tessara:p%40ssword@127.0.0.1:55432/tessara_rollback_test" `
        -ExpectedPostgresUser "tessara" `
        -ExpectedPostgresPassword "p@ssword" `
        -PublishedHostBindings ([object[]]@($ipv4Binding)) `
        -Context "Self-test database URL"
    if ($containerEndpoint.database_name -cne "tessara_rollback_test" -or
        $containerEndpoint.binding_host_ip -cne "127.0.0.1" -or
        $containerEndpoint.container_database_url -ne "postgresql://tessara@127.0.0.1:5432/tessara_rollback_test" -or
        $containerEndpoint.container_database_url -match "p%40ssword|p@ssword") {
        throw "Self-test did not safely derive the credential-free container-local PostgreSQL URL."
    }
    $ipv6Endpoint = ConvertTo-Sprint6AContainerPostgresEndpoint `
        -DatabaseUrl "postgres://tessara:p%40ssword@[::1]:55432/tessara_rollback_test" `
        -ExpectedPostgresUser "tessara" `
        -ExpectedPostgresPassword "p@ssword" `
        -PublishedHostBindings ([object[]]@([pscustomobject][ordered]@{ HostIp = "::1"; HostPort = "55432" })) `
        -Context "Self-test IPv6 database URL"
    if ($ipv6Endpoint.requested_host -cne "::1" -or $ipv6Endpoint.binding_host_ip -cne "::1") {
        throw "Self-test did not accept and exactly bind an IPv6 loopback URL."
    }
    foreach ($invalidContainerUrl in @(
        "postgres://tessara:p%40ssword@database.example:55432/tessara_rollback_test",
        "postgres://tessara:p%40ssword@localhost:55432/tessara_rollback_test",
        "postgres://tessara:p%40ssword@127.0.0.1:55433/tessara_rollback_test",
        "postgres://tessara:p%40ssword@127.0.0.1:55432/tessara_rollback_test?sslmode=disable",
        "postgres://tessara@127.0.0.1:55432/tessara_rollback_test",
        "postgres://tessara:wrong@127.0.0.1:55432/tessara_rollback_test"
    )) {
        $rejected = $false
        try {
            [void](ConvertTo-Sprint6AContainerPostgresEndpoint `
                -DatabaseUrl $invalidContainerUrl `
                -ExpectedPostgresUser "tessara" `
                -ExpectedPostgresPassword "p@ssword" `
                -PublishedHostBindings ([object[]]@($ipv4Binding)) `
                -Context "Invalid self-test database URL")
        } catch {
            $rejected = $true
        }
        if (-not $rejected) {
            throw "Self-test accepted an unsafe or mismatched container PostgreSQL URL."
        }
    }
    foreach ($unsafeBindings in @(
        [object[]]@([pscustomobject][ordered]@{ HostIp = "192.0.2.10"; HostPort = "55432" }),
        [object[]]@(
            [pscustomobject][ordered]@{ HostIp = "127.0.0.1"; HostPort = "55432" },
            [pscustomobject][ordered]@{ HostIp = "0.0.0.0"; HostPort = "55432" }
        )
    )) {
        $rejected = $false
        try {
            [void](ConvertTo-Sprint6AContainerPostgresEndpoint `
                -DatabaseUrl "postgres://tessara:p%40ssword@127.0.0.1:55432/tessara_rollback_test" `
                -ExpectedPostgresUser "tessara" `
                -ExpectedPostgresPassword "p@ssword" `
                -PublishedHostBindings ([object[]]$unsafeBindings) `
                -Context "Unsafe host-binding self-test")
        } catch {
            $rejected = $true
        }
        if (-not $rejected) {
            throw "Self-test accepted a wrong-family or ambiguous Docker host binding."
        }
    }

    $caseRejected = $false
    try {
        Assert-Sprint6AExactDatabaseName `
            -Actual "Tessara_Rollback_Test" `
            -Expected "tessara_rollback_test" `
            -Context "Case-sensitive identity self-test"
    } catch {
        $caseRejected = $true
    }
    if (-not $caseRejected) {
        throw "Self-test accepted a case-insensitive database identity match."
    }
    $canonicalLocalEndpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
        -DatabaseUrl "postgresql://local_user:p%40ssword@db.example:5433/tessara_rollback_test" `
        -Context "Canonical local-client URL self-test"
    if ($canonicalLocalEndpoint.database_user -cne "local_user" -or
        $canonicalLocalEndpoint.database_password -cne "p@ssword" -or
        $canonicalLocalEndpoint.database_name -cne "tessara_rollback_test" -or
        $canonicalLocalEndpoint.host -cne "db.example" -or
        $canonicalLocalEndpoint.host_port -ne 5433) {
        throw "Self-test did not preserve the documented canonical local-client PostgreSQL URL subset."
    }
    foreach ($unsupportedLocalUrl in @(
        "postgres://local_user@db.example/tessara_rollback_test",
        "postgres://local_user:@db.example/tessara_rollback_test",
        "postgres://local_user:password@db.example/tessara_rollback_test?sslmode=require",
        "postgres://local_user:password@db.example/tessara_rollback_test#fragment"
    )) {
        $rejected = $false
        try {
            [void](ConvertFrom-Sprint6APostgresDatabaseUrl `
                -DatabaseUrl $unsupportedLocalUrl `
                -Context "Unsupported local-client URL self-test")
        } catch {
            $rejected = $true
        }
        if (-not $rejected) {
            throw "Self-test accepted a local-client PostgreSQL URL outside the documented canonical subset."
        }
    }
    $resetStatements = @(Get-Sprint6ADatabaseResetStatements)
    if ($resetStatements[0] -cne "SET standard_conforming_strings TO on;" -or
        ($resetStatements -join "`n") -notmatch ":'target_database'" -or
        ($resetStatements -join "`n") -notmatch ':"target_database"' -or
        ($resetStatements -join "`n") -match "tessara_rollback_test") {
        throw "Self-test found an unsafe interpolated target name in destructive database reset SQL."
    }

    $restoreEvidenceTestRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-restore-evidence-self-test-$PID-$([Guid]::NewGuid().ToString('N'))"
    New-Item -ItemType Directory -Path $restoreEvidenceTestRoot -Force | Out-Null
    try {
        $backupFixturePath = Join-Path $restoreEvidenceTestRoot "pre-upgrade.dump"
        [IO.File]::WriteAllBytes($backupFixturePath, [byte[]]@(80, 71, 68, 77, 80, 1, 2, 3))
        $backupFixture = Get-Item -LiteralPath $backupFixturePath
        $backupFixtureSha256 = Get-Sprint6AFileSha256 $backupFixturePath
        $selfTestExecutablePath = [IO.Path]::GetFullPath((Get-Process -Id $PID).Path)
        $selfTestExecutableSha256 = Get-Sprint6AFileSha256 $selfTestExecutablePath
        $fingerprintFixture = [ordered]@{
            contract_id = "postgres_user_schema_and_logical_rows_v1"
            canonical_format = "utf8_length_framed_schema_and_data_sections_lf_v1"
            schema_sha256 = (("a" * 64) -join "")
            data_sha256 = (("b" * 64) -join "")
            canonical_sha256 = (("c" * 64) -join "")
            schema_section_count = 10
            relation_count = 12
            sequence_count = 0
        }
        $positiveEvidence = [ordered]@{
            schema_version = $script:Sprint6ARestoreEvidenceSchemaVersion
            evidence_kind = "tessara_sprint_6a_pre_upgrade_backup_restore_proof"
            generated_at_utc = [DateTime]::UtcNow.ToString("o")
            generator = [ordered]@{
                script_name = "capture-sprint-6a-rollback-restore-evidence.ps1"
                script_sha256 = Get-Sprint6AFileSha256 (Join-Path $PSScriptRoot "capture-sprint-6a-rollback-restore-evidence.ps1")
                common_helper_name = $script:Sprint6ARestoreEvidenceCommonHelperName
                common_helper_sha256 = Get-Sprint6AFileSha256 $script:Sprint6ARestoreEvidenceCommonHelperPath
                powershell_version = $PSVersionTable.PSVersion.ToString()
            }
            postgres_client = [ordered]@{
                mode = "local_executables"
                container_id = $null
                container_name = $null
                image_reference = $null
                image_id = $null
                tool_commands = [ordered]@{
                    psql = $selfTestExecutablePath
                    pg_dump = $selfTestExecutablePath
                    pg_restore = $selfTestExecutablePath
                }
                tool_sha256 = [ordered]@{
                    psql = $selfTestExecutableSha256
                    pg_dump = $selfTestExecutableSha256
                    pg_restore = $selfTestExecutableSha256
                }
                validated_host_bindings = @()
            }
            backup_artifact = [ordered]@{
                path_relative_to_evidence = "pre-upgrade.dump"
                sha256 = $backupFixtureSha256
                length_bytes = $backupFixture.Length
                format = "postgresql_custom_archive"
                source_database_name = "tessara_sprint5a_retained"
            }
            source_database = [ordered]@{
                database_name = "tessara_sprint5a_retained"
                migration_ledger = @(1, 2)
                fingerprint = $fingerprintFixture
            }
            restored_target_database = [ordered]@{
                database_name = "tessara-rollback-restore-test"
                migration_ledger = @(1, 2)
                fingerprint = $fingerprintFixture
            }
            restore_operation = [ordered]@{
                target_was_destructively_recreated = $true
                backup_sha256_used = $backupFixtureSha256
                credential_redaction = "all_database_urls_replaced_with_named_placeholders"
                archive_transfer = "direct_host_path"
                commands = @(
                    [ordered]@{ tool = "psql"; arguments = @("<source-database-connection-environment>", "<read-only-name-ledger-fingerprint-verification>") },
                    [ordered]@{ tool = "pg_dump"; arguments = @("<source-database-connection-environment>", "--format=custom", "--no-owner", "--no-privileges", "--file", "<retained-backup-path>") },
                    [ordered]@{ tool = "psql"; arguments = @("<maintenance-database-connection-environment>", "<terminate-drop-create-disposable-target>", "<target-database-name>") },
                    [ordered]@{ tool = "pg_restore"; arguments = @("<target-database-connection-environment>", "--no-owner", "--no-privileges", "--exit-on-error", "<retained-backup-path>") },
                    [ordered]@{ tool = "psql"; arguments = @("<target-database-connection-environment>", "<read-only-name-ledger-fingerprint-verification>") }
                )
            }
            result = "passed"
        }
        $positiveEvidencePath = Join-Path $restoreEvidenceTestRoot "positive.json"
        Write-Sprint6AUtf8Json -Value $positiveEvidence -Path $positiveEvidencePath
        [void](Assert-Sprint6ARestoreEvidenceDocument `
            -EvidencePath $positiveEvidencePath `
            -ExpectedTargetDatabaseName "tessara-rollback-restore-test")

        function Copy-RestoreEvidenceFixture {
            return ($positiveEvidence | ConvertTo-Json -Depth 12 | ConvertFrom-Json)
        }
        function Assert-RestoreEvidenceFixtureRejected {
            param(
                [Parameter(Mandatory)]$Fixture,
                [Parameter(Mandatory)][string]$Name,
                [string]$ExpectedTargetDatabaseName = "tessara-rollback-restore-test"
            )

            $fixturePath = Join-Path $restoreEvidenceTestRoot "$Name.json"
            Write-Sprint6AUtf8Json -Value $Fixture -Path $fixturePath
            $rejected = $false
            try {
                [void](Assert-Sprint6ARestoreEvidenceDocument `
                    -EvidencePath $fixturePath `
                    -ExpectedTargetDatabaseName $ExpectedTargetDatabaseName)
            } catch {
                $rejected = $true
            }
            if (-not $rejected) {
                throw "Self-test did not reject tampered restore evidence '$Name'."
            }
        }

        $positiveContainerEvidence = Copy-RestoreEvidenceFixture
        $positiveContainerEvidence.postgres_client = [pscustomobject][ordered]@{
            mode = "docker_container"
            container_id = ("a" * 64)
            container_name = "tessara-postgres-self-test"
            image_reference = "postgres:16-alpine"
            image_id = "sha256:$('b' * 64)"
            tool_commands = [pscustomobject][ordered]@{
                psql = "psql"
                pg_dump = "pg_dump"
                pg_restore = "pg_restore"
            }
            tool_sha256 = [pscustomobject][ordered]@{
                psql = $null
                pg_dump = $null
                pg_restore = $null
            }
            validated_host_bindings = @(
                [pscustomobject][ordered]@{
                    requested_host = "127.0.0.1"
                    requested_host_port = 55432
                    database_name = "tessara-rollback-restore-test"
                    database_user_sha256 = Get-Sprint6AStringSha256 "tessara"
                    binding_host_ip = "127.0.0.1"
                    binding_host_port = 55432
                }
            )
        }
        $positiveContainerEvidence.restore_operation.archive_transfer = "docker_cp_out_and_execution_user_stdin_in_unique_container_temp_paths_with_finally_cleanup"
        $positiveContainerEvidence.restore_operation.commands = @(
            [pscustomobject][ordered]@{ tool = "psql"; arguments = @("<postgres-client-container-id>", "<source-database-url>", "<read-only-name-ledger-fingerprint-verification>") },
            [pscustomobject][ordered]@{ tool = "pg_dump"; arguments = @("<postgres-client-container-id>", "--dbname", "<source-database-url>", "--format=custom", "--no-owner", "--no-privileges", "--file", "<unique-container-dump-path>") },
            [pscustomobject][ordered]@{ tool = "docker"; arguments = @("cp", "<postgres-client-container-id>:<unique-container-dump-path>", "<retained-backup-path>") },
            [pscustomobject][ordered]@{ tool = "psql"; arguments = @("<postgres-client-container-id>", "<maintenance-database-url>", "<terminate-drop-create-disposable-target>", "<target-database-name>") },
            [pscustomobject][ordered]@{ tool = "docker"; arguments = @("exec", "--interactive", "<postgres-client-container-id>", "<retained-backup-stdin>", "<unique-container-dump-path>") },
            [pscustomobject][ordered]@{ tool = "pg_restore"; arguments = @("<postgres-client-container-id>", "--dbname", "<target-database-url>", "--no-owner", "--no-privileges", "--exit-on-error", "<unique-container-dump-path>") },
            [pscustomobject][ordered]@{ tool = "psql"; arguments = @("<postgres-client-container-id>", "<target-database-url>", "<read-only-name-ledger-fingerprint-verification>") }
        )
        $positiveContainerEvidencePath = Join-Path $restoreEvidenceTestRoot "positive-container.json"
        Write-Sprint6AUtf8Json -Value $positiveContainerEvidence -Path $positiveContainerEvidencePath
        [void](Assert-Sprint6ARestoreEvidenceDocument `
            -EvidencePath $positiveContainerEvidencePath `
            -ExpectedTargetDatabaseName "tessara-rollback-restore-test")

        $badSchema = Copy-RestoreEvidenceFixture
        $badSchema.schema_version = 2
        Assert-RestoreEvidenceFixtureRejected -Fixture $badSchema -Name "bad-schema"

        $badKind = Copy-RestoreEvidenceFixture
        $badKind.evidence_kind = "unrelated_restore_claim"
        Assert-RestoreEvidenceFixtureRejected -Fixture $badKind -Name "bad-kind"

        $badGeneratorDigest = Copy-RestoreEvidenceFixture
        $replacementPrefix = if ($badGeneratorDigest.generator.script_sha256.StartsWith("a")) { "b" } else { "a" }
        $badGeneratorDigest.generator.script_sha256 = $replacementPrefix + $badGeneratorDigest.generator.script_sha256.Substring(1)
        Assert-RestoreEvidenceFixtureRejected -Fixture $badGeneratorDigest -Name "bad-generator-digest"

        $badCommonHelperDigest = Copy-RestoreEvidenceFixture
        $replacementPrefix = if ($badCommonHelperDigest.generator.common_helper_sha256.StartsWith("a")) { "b" } else { "a" }
        $badCommonHelperDigest.generator.common_helper_sha256 = $replacementPrefix + $badCommonHelperDigest.generator.common_helper_sha256.Substring(1)
        Assert-RestoreEvidenceFixtureRejected -Fixture $badCommonHelperDigest -Name "bad-common-helper-digest"

        $badLocalToolDigest = Copy-RestoreEvidenceFixture
        $badLocalToolDigest.postgres_client.tool_sha256.psql = ("d" * 64)
        Assert-RestoreEvidenceFixtureRejected -Fixture $badLocalToolDigest -Name "bad-local-tool-digest"

        $badArtifactDigest = Copy-RestoreEvidenceFixture
        $badArtifactDigest.backup_artifact.sha256 = (("e" * 64) -join "")
        Assert-RestoreEvidenceFixtureRejected -Fixture $badArtifactDigest -Name "bad-artifact-digest"

        $badLedger = Copy-RestoreEvidenceFixture
        $badLedger.restored_target_database.migration_ledger = @(1, 3)
        Assert-RestoreEvidenceFixtureRejected -Fixture $badLedger -Name "bad-ledger"

        $badFingerprint = Copy-RestoreEvidenceFixture
        $badFingerprint.restored_target_database.fingerprint.canonical_sha256 = (("f" * 64) -join "")
        Assert-RestoreEvidenceFixtureRejected -Fixture $badFingerprint -Name "bad-fingerprint"

        $badTarget = Copy-RestoreEvidenceFixture
        $badTarget.restored_target_database.database_name = "a-different-rollback-test"
        Assert-RestoreEvidenceFixtureRejected -Fixture $badTarget -Name "bad-target"

        Assert-RestoreEvidenceFixtureRejected `
            -Fixture (Copy-RestoreEvidenceFixture) `
            -Name "case-mismatched-expected-target" `
            -ExpectedTargetDatabaseName "Tessara-Rollback-Restore-Test"

        $leakedCredential = Copy-RestoreEvidenceFixture
        $leakedCredential.restore_operation.commands[1].arguments[1] = "postgres://user:password@localhost/source"
        Assert-RestoreEvidenceFixtureRejected -Fixture $leakedCredential -Name "leaked-credential"

        $extraProperty = Copy-RestoreEvidenceFixture
        $extraProperty | Add-Member -NotePropertyName unsupported_claim -NotePropertyValue "ignored"
        Assert-RestoreEvidenceFixtureRejected -Fixture $extraProperty -Name "extra-property"

        $badContainerIdentity = $positiveContainerEvidence | ConvertTo-Json -Depth 12 | ConvertFrom-Json
        $badContainerIdentity.postgres_client.container_id = "not-an-immutable-container-id"
        Assert-RestoreEvidenceFixtureRejected -Fixture $badContainerIdentity -Name "bad-container-identity"

        $badContainerBinding = $positiveContainerEvidence | ConvertTo-Json -Depth 12 | ConvertFrom-Json
        $badContainerBinding.postgres_client.validated_host_bindings[0].binding_host_ip = "192.0.2.10"
        Assert-RestoreEvidenceFixtureRejected -Fixture $badContainerBinding -Name "bad-container-binding"

        $badContainerTransfer = $positiveContainerEvidence | ConvertTo-Json -Depth 12 | ConvertFrom-Json
        $badContainerTransfer.restore_operation.archive_transfer = "direct_host_path"
        Assert-RestoreEvidenceFixtureRejected -Fixture $badContainerTransfer -Name "bad-container-transfer"

        $badContainerCommand = $positiveContainerEvidence | ConvertTo-Json -Depth 12 | ConvertFrom-Json
        $badContainerCommand.restore_operation.commands[2].arguments[1] = "actual-container-id:/tmp/archive.dump"
        Assert-RestoreEvidenceFixtureRejected -Fixture $badContainerCommand -Name "unsanitized-container-command"

        $stdoutFixturePath = Join-Path $restoreEvidenceTestRoot "stdout.log"
        $stderrFixturePath = Join-Path $restoreEvidenceTestRoot "stderr.log"
        $logPassword = "never retain this p@ssword"
        $logDatabaseUrl = "postgres://user:never%20retain%20this%20p%40ssword@127.0.0.1/database"
        $logDevAdminPassword = "never-retain-this-admin-password"
        $logToken = "never-retain-this-bearer-token"
        [IO.File]::WriteAllText(
            $stdoutFixturePath,
            "ready $logDatabaseUrl Bearer $logToken password=$logPassword admin=$logDevAdminPassword`n",
            [Text.UTF8Encoding]::new($false)
        )
        [IO.File]::WriteAllText(
            $stderrFixturePath,
            "final diagnostic PGPASSWORD=not-a-declared-secret`n",
            [Text.UTF8Encoding]::new($false)
        )
        $logEvidence = New-SanitizedLogEvidence `
            -StdoutPath $stdoutFixturePath `
            -StderrPath $stderrFixturePath `
            -DatabaseUrl $logDatabaseUrl `
            -DevAdminPassword $logDevAdminPassword `
            -AdditionalSecrets ([string[]]@($logToken))
        if ($logEvidence.lifecycle -cne "process_stopped_before_final_log_capture" -or
            $logEvidence.stdout.content -match "never.retain|postgres://user:" -or
            $logEvidence.stderr.content -match "not-a-declared-secret" -or
            $logEvidence.stdout.content -notmatch "password=<redacted-secret>" -or
            $logEvidence.stderr.content -notmatch "PGPASSWORD=<redacted-secret>" -or
            $logEvidence.stdout.sha256 -cne (Get-StringSha256 $logEvidence.stdout.content) -or
            $logEvidence.stderr.sha256 -cne (Get-StringSha256 $logEvidence.stderr.content)) {
            throw "Self-test did not produce durable, sanitized, recomputable final-log evidence."
        }

        $emptyLogPath = Join-Path $restoreEvidenceTestRoot "empty.log"
        [IO.File]::WriteAllText($emptyLogPath, "", [Text.UTF8Encoding]::new($false))
        $emptyLogEvidence = New-SanitizedLogEvidence `
            -StdoutPath $emptyLogPath `
            -StderrPath $emptyLogPath `
            -DatabaseUrl $logDatabaseUrl `
            -DevAdminPassword $logDevAdminPassword
        foreach ($stream in @($emptyLogEvidence.stdout, $emptyLogEvidence.stderr)) {
            if ($stream.content -cne "" -or
                $stream.length_bytes -ne 0 -or
                $stream.sha256 -cne "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855") {
                throw "Self-test did not retain deterministic SHA-256 evidence for an empty final log stream."
            }
        }

        $originalProcessInvoker = (Get-Item Function:\Invoke-Sprint6AProcess).ScriptBlock
        $script:selfTestProcessCalls = [Collections.Generic.List[object]]::new()
        Set-Item Function:\Invoke-Sprint6AProcess -Value {
            param(
                [Parameter(Mandatory)][string]$Command,
                [Parameter(Mandatory)][string[]]$Arguments,
                [Parameter(Mandatory)][string]$Context,
                [hashtable]$Environment = @{},
                [string]$StandardInputPath
            )

            [void]$script:selfTestProcessCalls.Add([pscustomobject][ordered]@{
                command = $Command
                arguments = @($Arguments)
                context = $Context
                environment = $Environment
                standard_input_path = $StandardInputPath
            })
            if ($Context -eq "pre-upgrade pg_restore verification in PostgreSQL client container") {
                throw "intentional pg_restore self-test failure"
            }
            return ""
        }
        try {
            $localDatabaseUrl = "postgres://tessara:p%40ssword@127.0.0.1:55432/tessara_rollback_test"
            $localContext = [pscustomobject][ordered]@{
                mode = "local_executables"
                psql_command = $selfTestExecutablePath
                psql_sha256 = $selfTestExecutableSha256
                allowed_database_url_sha256 = @((Get-Sprint6AStringSha256 $localDatabaseUrl))
            }
            [void](Invoke-Sprint6APostgresClientTool `
                -Tool "psql" `
                -DatabaseUrl $localDatabaseUrl `
                -Arguments ([string[]]@("--version")) `
                -PostgresClientContext $localContext `
                -Context "local argv credential self-test")
            $localCall = $script:selfTestProcessCalls[0]
            if (($localCall.arguments -join "`n") -match "p%40ssword|p@ssword|postgres://" -or
                $localCall.environment.PGPASSWORD -cne "p@ssword" -or
                $localCall.command -cne $selfTestExecutablePath) {
                throw "Self-test found a database URL or password in local PostgreSQL process arguments, or lost exact executable identity."
            }

            $script:selfTestProcessCalls.Clear()
            $dockerDatabaseUrl = "postgres://tessara:p%40ssword@127.0.0.1:55432/tessara_rollback_test"
            $dockerContext = [pscustomobject][ordered]@{
                mode = "docker_container"
                container_id = ("a" * 64)
                docker_command = "docker"
                allowed_database_url_sha256 = @((Get-Sprint6AStringSha256 $dockerDatabaseUrl))
            }
            $restoreFailed = $false
            try {
                Invoke-Sprint6APgRestoreVerification `
                    -DatabaseUrl $dockerDatabaseUrl `
                    -InputPath $backupFixturePath `
                    -PostgresClientContext $dockerContext
            } catch {
                if ($_.Exception.Message -notmatch "intentional pg_restore self-test failure") {
                    throw
                }
                $restoreFailed = $true
            }
            if (-not $restoreFailed -or $script:selfTestProcessCalls.Count -ne 3) {
                throw "Self-test did not exercise failed container restore plus guaranteed cleanup."
            }
            $streamCall = $script:selfTestProcessCalls[0]
            $cleanupCall = $script:selfTestProcessCalls[2]
            $allDockerArguments = @($script:selfTestProcessCalls | ForEach-Object { $_.arguments }) -join "`n"
            if ($streamCall.arguments[0] -cne "exec" -or
                "--interactive" -notin $streamCall.arguments -or
                $streamCall.standard_input_path -cne $backupFixturePath -or
                $allDockerArguments -match "p%40ssword|p@ssword" -or
                $cleanupCall.arguments[2] -cne "rm" -or
                "-f" -notin $cleanupCall.arguments) {
                throw "Self-test did not stream as the container execution user, redact Docker argv, or clean up after restore failure."
            }
        } finally {
            Set-Item Function:\Invoke-Sprint6AProcess -Value $originalProcessInvoker
            Remove-Variable -Scope Script -Name selfTestProcessCalls -ErrorAction SilentlyContinue
        }
    } finally {
        $resolvedTestRoot = [IO.Path]::GetFullPath($restoreEvidenceTestRoot)
        $tempPrefix = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
        if (-not $resolvedTestRoot.StartsWith($tempPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove unexpected restore-evidence self-test path '$resolvedTestRoot'."
        }
        Remove-Item -LiteralPath $resolvedTestRoot -Recurse -Force
    }

    Write-Host "Rollback validator static seed, historical product mutation, and structured restore-evidence self-test passed."
    return
}

if ([string]::IsNullOrWhiteSpace($ExpectedClosingSprint6ACommit)) {
    throw "ExpectedClosingSprint6ACommit is required for package validation and must identify the exact reviewed closing Sprint 6A commit."
}

if (-not (Test-Path -LiteralPath $packageFullPath -PathType Container)) {
    throw "Rollback package '$packageFullPath' does not exist."
}

function Get-RelativePackagePath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )

    return [IO.Path]::GetRelativePath($Root, $Path).Replace('\', '/')
}

function Resolve-PackageFile {
    param([Parameter(Mandatory)][string]$RelativePath)

    if ([IO.Path]::IsPathRooted($RelativePath)) {
        throw "Manifest path '$RelativePath' must be relative."
    }
    $resolved = [IO.Path]::GetFullPath((Join-Path $packageFullPath $RelativePath))
    $prefix = $packageFullPath.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Manifest path '$RelativePath' escapes the package root."
    }
    return $resolved
}

function Assert-PackageManifest {
    $manifestPath = Join-Path $packageFullPath "manifest.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Package manifest '$manifestPath' does not exist."
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.schema_version -ne 1) {
        throw "Unsupported rollback package manifest schema '$($manifest.schema_version)'."
    }
    if ($manifest.package_kind -ne "tessara_sprint_5a_code_sprint_6a_migration_compatibility_rollback") {
        throw "Unexpected rollback package kind '$($manifest.package_kind)'."
    }
    if ($manifest.application.sprint_5a_source_commit -notmatch '^[0-9a-f]{40}$') {
        throw "Manifest does not identify an exact Sprint 5A source commit."
    }
    if ($manifest.application.sprint_5a_source_commit -ne $sprint5ASeedContract.source_commit) {
        throw "Manifest Sprint 5A source commit does not match declared seed contract '$($sprint5ASeedContract.contract_id)'."
    }
    if ($manifest.closing_sprint_6a_repository_commit -notmatch '^[0-9a-f]{40}$') {
        throw "Manifest does not identify an exact closing Sprint 6A repository commit."
    }
    if ($manifest.closing_sprint_6a_repository_commit -ne $ExpectedClosingSprint6ACommit.ToLowerInvariant()) {
        throw "Manifest closing Sprint 6A commit '$($manifest.closing_sprint_6a_repository_commit)' does not equal explicitly expected commit '$ExpectedClosingSprint6ACommit'."
    }
    if (($manifest.compatibility_contract.upgraded_database_ledger -join ",") -ne "1,2,3" -or
        $manifest.compatibility_contract.control_plane_behavior -ne "ignored_by_exact_sprint_5a_code" -or
        $manifest.compatibility_contract.compatibility_migrations_path -ne "migrations" -or
        $manifest.compatibility_contract.original_historical_migrations_path -ne "original-migrations" -or
        $manifest.compatibility_contract.original_historical_usage -ne "only_after_pre_upgrade_backup_restore_to_ledger_1_2" -or
        $manifest.compatibility_contract.destructive_down_migration -ne $false -or
        $manifest.compatibility_contract.applied_checksum_edit -ne $false) {
        throw "Manifest compatibility contract permits behavior outside the approved additive rollback path."
    }
    foreach ($builderField in @("cargo_version", "rustc_version", "cargo_leptos_version", "node_version", "npm_version", "packaging_script_sha256", "build_command")) {
        if ([string]::IsNullOrWhiteSpace($manifest.builder.$builderField)) {
            throw "Manifest is missing builder evidence '$builderField'."
        }
    }
    if ($manifest.builder.packaging_script_sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Manifest packaging script digest is not a SHA-256 value."
    }

    $declaredFiles = @($manifest.package_content.files)
    if ($manifest.package_content.digest_algorithm -ne "sha256_of_sorted_utf8_path_equals_sha256_lines_excluding_manifest") {
        throw "Manifest uses an unsupported package content digest algorithm."
    }
    $declaredPaths = @($declaredFiles | ForEach-Object { $_.path } | Sort-Object)
    $actualPaths = @(Get-ChildItem -LiteralPath $packageFullPath -File -Recurse |
        Where-Object { $_.FullName -ne $manifestPath } |
        ForEach-Object { Get-RelativePackagePath $packageFullPath $_.FullName } |
        Sort-Object)
    if (($declaredPaths -join "`n") -ne ($actualPaths -join "`n")) {
        throw "Package payload file set does not match manifest.json."
    }

    foreach ($entry in $declaredFiles) {
        $filePath = Resolve-PackageFile $entry.path
        if (-not (Test-Path -LiteralPath $filePath -PathType Leaf)) {
            throw "Manifest payload '$($entry.path)' does not exist."
        }
        $file = Get-Item -LiteralPath $filePath
        if ($file.Length -ne [long]$entry.length_bytes) {
            throw "Manifest length mismatch for '$($entry.path)'."
        }
        $actualHash = Get-Sha256 $filePath
        if ($actualHash -ne $entry.sha256) {
            throw "Manifest SHA-256 mismatch for '$($entry.path)'."
        }
    }

    $digestLines = @($declaredFiles |
        Sort-Object path |
        ForEach-Object { "$($_.path)=$($_.sha256)" })
    $actualPackageDigest = Get-StringSha256 (($digestLines -join "`n") + "`n")
    if ($actualPackageDigest -ne $manifest.package_content.sha256) {
        throw "Package content digest does not match the declared digest."
    }

    $expectedMigrations = @(
        "001_baseline.sql",
        "002_dashboard_placement_capacity.sql",
        "003_module_control_plane.sql"
    )
    $migrations = @($manifest.migrations)
    if ($migrations.Count -ne 3) {
        throw "Compatibility package must contain exactly migrations 1 through 3."
    }
    $compatibilityMigrationFiles = @(Get-ChildItem -LiteralPath (Join-Path $packageFullPath "migrations") -File |
        Select-Object -ExpandProperty Name |
        Sort-Object)
    if (($compatibilityMigrationFiles -join ",") -ne ($expectedMigrations -join ",")) {
        throw "Compatibility migration directory must contain only immutable migrations 1 through 3."
    }
    $originalMigrationFiles = @(Get-ChildItem -LiteralPath (Join-Path $packageFullPath "original-migrations") -File |
        Select-Object -ExpandProperty Name |
        Sort-Object)
    if (($originalMigrationFiles -join ",") -ne ($expectedMigrations[0..1] -join ",")) {
        throw "Original historical migration directory must contain only migrations 1 and 2."
    }
    for ($index = 0; $index -lt $expectedMigrations.Count; $index++) {
        $entry = $migrations[$index]
        if ($entry.version -ne ($index + 1) -or $entry.file_name -ne $expectedMigrations[$index]) {
            throw "Compatibility migration manifest is not the immutable ordered 1-3 set."
        }
        $migrationPath = Resolve-PackageFile $entry.path
        if ((Get-Sha256 $migrationPath) -ne $entry.sha256) {
            throw "Compatibility migration digest mismatch for '$($entry.file_name)'."
        }
    }
    foreach ($fileName in $expectedMigrations[0..1]) {
        $compatibilityHash = Get-Sha256 (Join-Path $packageFullPath "migrations/$fileName")
        $historicalHash = Get-Sha256 (Join-Path $packageFullPath "original-migrations/$fileName")
        if ($compatibilityHash -ne $historicalHash) {
            throw "Historical and compatibility copies of '$fileName' are not byte-identical."
        }
    }

    $binaryPath = Resolve-PackageFile $manifest.application.binary_path
    if ((Get-Sha256 $binaryPath) -ne $manifest.application.binary_sha256) {
        throw "Sprint 5A binary digest does not match the manifest."
    }
    $sourceArchivePath = Resolve-PackageFile $manifest.application.source_archive_path
    if ((Get-Sha256 $sourceArchivePath) -ne $manifest.application.source_archive_sha256) {
        throw "Sprint 5A source archive digest does not match the manifest."
    }
    $sitePath = Resolve-PackageFile $manifest.application.site_path
    if (-not (Test-Path -LiteralPath $sitePath -PathType Container)) {
        throw "Sprint 5A SSR site bundle does not exist at '$($manifest.application.site_path)'."
    }
    foreach ($assetName in @("tessara-web.css", "tessara-web.js", "tessara-web.wasm")) {
        $assetPath = Join-Path $sitePath "pkg/$assetName"
        if (-not (Test-Path -LiteralPath $assetPath -PathType Leaf)) {
            throw "Sprint 5A SSR site bundle is missing '$assetName'."
        }
    }

    return $manifest
}

function Invoke-PsqlLines {
    param([Parameter(Mandatory)][string]$Sql)

    return @(
        Invoke-Sprint6APsqlLines `
            -DatabaseUrl $DatabaseUrl `
            -PsqlCommand $PsqlCommand `
            -PostgresClientContext $postgresClientContext `
            -Sql $Sql
    )
}

function Invoke-PsqlScalar {
    param([Parameter(Mandatory)][string]$Sql)

    $lines = @(Invoke-PsqlLines $Sql)
    if ($lines.Count -ne 1) {
        throw "psql scalar query returned $($lines.Count) lines instead of exactly one."
    }
    return $lines[0]
}

function Get-AppliedMigrations {
    $lines = @(Invoke-PsqlLines "SELECT version::text FROM _sqlx_migrations WHERE success ORDER BY version;")
    return @($lines | Where-Object { $_ -ne "" } | ForEach-Object { [int64]$_ })
}

function Assert-Ledger {
    param([Parameter(Mandatory)][long[]]$Expected)

    $actual = @(Get-AppliedMigrations)
    if (($actual -join ",") -ne ($Expected -join ",")) {
        throw "Database migration ledger '$($actual -join ',')' does not equal required ledger '$($Expected -join ',')'."
    }
    return $actual
}

function Get-BuiltInSeedSnapshot {
    $json = Invoke-PsqlScalar @"
SELECT COALESCE(
    jsonb_agg(
        jsonb_build_object('role_name', r.name, 'capability_key', c.key)
        ORDER BY r.name COLLATE "C", c.key COLLATE "C"
    ),
    '[]'::jsonb
)::text
FROM role_capabilities AS rc
JOIN roles AS r ON r.id = rc.role_id
JOIN capabilities AS c ON c.id = rc.capability_id
WHERE r.name IN ('admin', 'operator', 'respondent');
"@
    $mappings = @($json | ConvertFrom-Json)
    return New-BuiltInSeedSnapshot $mappings
}

function Assert-BuiltInSeedContract {
    param(
        [Parameter(Mandatory)]$Actual,
        [Parameter(Mandatory)]$ExpectedContract,
        [Parameter(Mandatory)][string]$Context
    )

    $expected = $ExpectedContract.snapshot
    $actualJson = $Actual.mappings | ConvertTo-Json -Depth 4 -Compress
    $expectedJson = $expected.mappings | ConvertTo-Json -Depth 4 -Compress
    if ($Actual.canonical_format -ne $expected.canonical_format -or
        $Actual.canonical_sha256 -ne $expected.canonical_sha256 -or
        $Actual.mapping_count -ne $expected.mapping_count -or
        $actualJson -ne $expectedJson) {
        throw "$Context built-in seed mappings do not equal declared contract '$($ExpectedContract.contract_id)'; actual=$actualJson expected=$expectedJson"
    }

    return [pscustomobject][ordered]@{
        expected_contract_id = $ExpectedContract.contract_id
        expected_contract_canonical_format = $ExpectedContract.contract_canonical_format
        expected_contract_sha256 = $ExpectedContract.contract_sha256
        matched = $true
        observed_mapping_set_canonical_format = $Actual.canonical_format
        observed_mapping_set_sha256 = $Actual.canonical_sha256
        mapping_count = $Actual.mapping_count
        mappings = @($Actual.mappings)
    }
}

function Get-ControlPlaneFingerprint {
    $tables = @(
        "application_installations",
        "core_runtime_observations",
        "module_definition_reservations",
        "transition_descriptor_sources",
        "transition_catalog_projections",
        "transition_catalog_current",
        "module_catalog_findings",
        "capability_provenance",
        "module_navigation_contributions",
        "navigation_policies",
        "navigation_policy_entries",
        "core_control_plane_audit_events",
        "capabilities",
        "roles",
        "role_capabilities",
        "role_assignments",
        "_sqlx_migrations"
    )
    $lines = @()
    foreach ($table in $tables) {
        $exists = Invoke-PsqlScalar "SELECT CASE WHEN to_regclass('public.$table') IS NULL THEN 'false' ELSE 'true' END;"
        if ($exists -eq "false") {
            $lines += "$table|<absent>"
            continue
        }
        if ($exists -ne "true") {
            throw "Could not determine whether invariant table '$table' exists."
        }
        $query = if ($table -eq "role_capabilities") {
            # The three versioned built-in mappings have their own exact
            # snapshot/digest assertion. Everything else remains in this
            # invariant fingerprint.
            @"
SELECT COALESCE(
    jsonb_agg(to_jsonb(rc) ORDER BY (to_jsonb(rc)::text) COLLATE "C"),
    '[]'::jsonb
)::text
FROM role_capabilities AS rc
JOIN roles AS r ON r.id = rc.role_id
WHERE r.name NOT IN ('admin', 'operator', 'respondent');
"@
        } else {
            @"
SELECT COALESCE(
    jsonb_agg(to_jsonb(t) ORDER BY (to_jsonb(t)::text) COLLATE "C"),
    '[]'::jsonb
)::text
FROM $table AS t;
"@
        }
        $rows = Invoke-PsqlScalar $query
        $lines += "$table|$rows"
    }
    return Get-StringSha256 (($lines -join "`n") + "`n")
}

function Assert-UpgradedControlPlaneReady {
    $shape = Invoke-PsqlScalar @"
SELECT concat_ws('|',
    (SELECT COUNT(*) FROM application_installations),
    (SELECT COUNT(*) FROM module_definition_reservations),
    (SELECT COUNT(*) FROM transition_descriptor_sources),
    (SELECT COUNT(*) FROM transition_catalog_projections),
    (SELECT COUNT(*) FROM transition_catalog_current),
    (SELECT COUNT(*) FROM module_navigation_contributions),
    (SELECT COUNT(*) FROM navigation_policies),
    (SELECT COUNT(*) FROM navigation_policy_entries),
    (SELECT COUNT(*) FROM capabilities WHERE key IN ('modules:read', 'modules:manage_navigation') AND scope_mode = 'installation_global'),
    CASE WHEN to_regclass('public.module_releases') IS NULL THEN 'false' ELSE 'true' END,
    CASE WHEN to_regclass('public.module_instances') IS NULL THEN 'false' ELSE 'true' END
);
"@
    $expected = "1|7|7|7|7|6|1|6|2|false|false"
    if ($shape -ne $expected) {
        throw "Upgraded clone does not contain the complete Sprint 6A catalog/control-plane shape; got '$shape'."
    }
    return [ordered]@{
        application_installations = 1
        reserved_definitions = 7
        descriptor_sources = 7
        catalog_projections = 7
        current_catalog_entries = 7
        navigation_contributions = 6
        navigation_policies = 1
        navigation_policy_entries = 6
        global_module_capabilities = 2
        module_release_table_present = $false
        module_instance_table_present = $false
    }
}

function Assert-BindAddressAvailable {
    if ($BindAddress -notmatch '^([^:]+):(\d+)$') {
        throw "Bind address '$BindAddress' must use the IPv4/hostname form host:port."
    }
    $hostName = $Matches[1]
    $port = [int]$Matches[2]
    if ($hostName -notin @("127.0.0.1", "localhost")) {
        throw "Rollback validation binds only to loopback; got '$hostName'."
    }
    $ipAddress = if ($hostName -eq "127.0.0.1") {
        [Net.IPAddress]::Loopback
    } else {
        $resolvedAddresses = @(
            [Net.Dns]::GetHostAddresses($hostName) |
                Where-Object { $_.AddressFamily -eq [Net.Sockets.AddressFamily]::InterNetwork }
        )
        $resolvedAddresses[0]
    }
    if ($null -eq $ipAddress) {
        throw "Bind host '$hostName' did not resolve to an IPv4 address."
    }
    $listener = [Net.Sockets.TcpListener]::new($ipAddress, $port)
    try {
        $listener.Start()
    } catch {
        throw "Bind address '$BindAddress' is already in use or unavailable: $($_.Exception.Message)"
    } finally {
        $listener.Stop()
    }
}

function Set-ChildEnvironmentAndStart {
    param([Parameter(Mandatory)][string]$MigrationsDirectory)

    Assert-BindAddressAvailable
    $binaryPath = Resolve-PackageFile $manifest.application.binary_path
    $logDirectory = Join-Path ([IO.Path]::GetTempPath()) "tessara-rollback-validation-$PID-$([Guid]::NewGuid().ToString('N'))"
    New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
    $stdoutPath = Join-Path $logDirectory "stdout.log"
    $stderrPath = Join-Path $logDirectory "stderr.log"
    $childEnvironment = [ordered]@{
        DATABASE_URL = $DatabaseUrl
        TESSARA_BIND_ADDR = $BindAddress
        TESSARA_MIGRATIONS_DIR = $MigrationsDirectory
        TESSARA_DEV_ADMIN_EMAIL = $DevAdminEmail
        TESSARA_DEV_ADMIN_PASSWORD = $DevAdminPassword
        LEPTOS_SITE_ROOT = (Resolve-PackageFile $manifest.application.site_path)
        LEPTOS_SITE_PKG_DIR = "pkg"
        RUST_LOG = "tessara_api=info"
    }
    $previousEnvironment = @{}
    foreach ($name in $childEnvironment.Keys) {
        $previousEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
        [Environment]::SetEnvironmentVariable($name, $childEnvironment[$name], "Process")
    }
    try {
        $startArguments = @{
            FilePath = $binaryPath
            WorkingDirectory = $packageFullPath
            PassThru = $true
            WindowStyle = "Hidden"
            RedirectStandardOutput = $stdoutPath
            RedirectStandardError = $stderrPath
        }
        $process = Start-Process @startArguments
    } finally {
        foreach ($name in $childEnvironment.Keys) {
            [Environment]::SetEnvironmentVariable($name, $previousEnvironment[$name], "Process")
        }
    }
    return [pscustomobject]@{
        Process = $process
        LogDirectory = $logDirectory
        StdoutPath = $stdoutPath
        StderrPath = $stderrPath
    }
}

function Stop-ValidationProcess {
    param([Parameter(Mandatory)]$Run)

    if (-not $Run.Process.HasExited) {
        Stop-Process -Id $Run.Process.Id -Force
        $Run.Process.WaitForExit(10000) | Out-Null
    }
}

function Remove-ValidationLogs {
    param([Parameter(Mandatory)]$Run)

    if (-not (Test-Path -LiteralPath $Run.LogDirectory)) {
        return
    }
    $resolved = [IO.Path]::GetFullPath($Run.LogDirectory)
    $tempPrefix = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $resolved.StartsWith($tempPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove unexpected validation log path '$resolved'."
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}

function Get-FinalLogEvidence {
    param(
        [Parameter(Mandatory)]$Run,
        [string[]]$AdditionalSecrets = @()
    )

    Stop-ValidationProcess $Run
    return New-SanitizedLogEvidence `
        -StdoutPath $Run.StdoutPath `
        -StderrPath $Run.StderrPath `
        -DatabaseUrl $DatabaseUrl `
        -DevAdminPassword $DevAdminPassword `
        -AdditionalSecrets $AdditionalSecrets
}

function Assert-OriginalPackageRejectsUpgradedDatabase {
    $run = Set-ChildEnvironmentAndStart (Join-Path $packageFullPath "original-migrations")
    try {
        $exited = $false
        for ($attempt = 0; $attempt -lt 40; $attempt++) {
            if ($run.Process.HasExited) {
                $exited = $true
                break
            }
            Start-Sleep -Milliseconds 250
        }
        if (-not $exited) {
            throw "Original Sprint 5A package did not reject the upgraded migration-3 ledger."
        }
        $run.Process.WaitForExit()
        $run.Process.Refresh()
        if ($run.Process.ExitCode -eq 0) {
            throw "Original Sprint 5A package unexpectedly exited successfully against migration 3."
        }
        $stderr = if (Test-Path -LiteralPath $run.StderrPath) {
            Get-Content -LiteralPath $run.StderrPath -Raw
        } else {
            ""
        }
        if ($stderr -notmatch '(?is)(migration\s+3.*missing|missing.*migration\s+3)') {
            throw "Original Sprint 5A package failed for an unexpected reason instead of rejecting missing migration 3."
        }
        $logs = Get-FinalLogEvidence $run
        return [ordered]@{
            rejected = $true
            reason_code = "migration_3_missing_from_original_package"
            exit_code = $run.Process.ExitCode
            logs = $logs
        }
    } finally {
        Stop-ValidationProcess $run
        Remove-ValidationLogs $run
    }
}

function Wait-ForHealth {
    param([Parameter(Mandatory)]$Run)

    $healthUrl = "http://$BindAddress/health"
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($Run.Process.HasExited) {
            $errorTail = if (Test-Path -LiteralPath $Run.StderrPath) {
                (Get-Content -LiteralPath $Run.StderrPath -Tail 20) -join [Environment]::NewLine
            } else {
                "<no stderr log>"
            }
            throw "Rollback executable exited before health became ready:`n$errorTail"
        }
        try {
            $response = Invoke-WebRequest -Uri $healthUrl -UseBasicParsing -TimeoutSec 1
            if ($response.StatusCode -eq 200 -and $response.Content.Trim() -eq "ok") {
                Start-Sleep -Milliseconds 100
                if ($Run.Process.HasExited) {
                    throw "Rollback executable exited after another process answered the health check."
                }
                return
            }
        } catch {
            # Startup races are expected until the listener is bound.
        }
        Start-Sleep -Milliseconds 250
    }
    throw "Rollback executable did not become healthy at '$healthUrl' within 30 seconds."
}

function Invoke-ProductSmoke {
    param([Parameter(Mandatory)][string]$MigrationsDirectory)

    $formsCountBefore = [int64](Invoke-PsqlScalar "SELECT COUNT(*) FROM forms;")
    $run = Set-ChildEnvironmentAndStart $MigrationsDirectory
    try {
        Wait-ForHealth $run
        $baseUrl = "http://$BindAddress"
        $loginBody = @{
            email = $DevAdminEmail
            password = $DevAdminPassword
        } | ConvertTo-Json
        $loginArguments = @{
            Uri = "$baseUrl/api/auth/login"
            Method = "Post"
            ContentType = "application/json"
            Body = $loginBody
        }
        $login = Invoke-RestMethod @loginArguments
        if ([string]::IsNullOrWhiteSpace($login.token)) {
            throw "Historical product login did not return a bearer token."
        }
        $headers = @{ Authorization = "Bearer $($login.token)" }
        $existingFormsResponse = Invoke-RestMethod -Uri "$baseUrl/api/admin/forms" -Headers $headers
        $existingForms = [object[]]$existingFormsResponse
        if ($existingForms.Count -lt 1) {
            throw "Historical product read did not return the populated Sprint 5A form."
        }
        $historicalScope = Select-HistoricalFormScope $existingForms

        $suffix = [Guid]::NewGuid().ToString("N")
        $formName = "Compatibility rollback proof $suffix"
        $formSlug = "compatibility-rollback-proof-$suffix"
        $createBody = @{
            name = $formName
            slug = $formSlug
            scope_node_type_id = $historicalScope.scope_node_type_id
            visibility_node_ids = @($historicalScope.visibility_node_id)
        } | ConvertTo-Json
        $createArguments = @{
            Uri = "$baseUrl/api/admin/forms"
            Headers = $headers
            Method = "Post"
            ContentType = "application/json"
            Body = $createBody
        }
        $created = Invoke-RestMethod @createArguments
        if ([string]::IsNullOrWhiteSpace($created.id)) {
            throw "Historical product write did not return a form id."
        }
        $detail = Invoke-RestMethod -Uri "$baseUrl/api/admin/forms/$($created.id)" -Headers $headers
        if ($detail.id -ne $created.id -or $detail.name -ne $formName -or $detail.slug -ne $formSlug) {
            throw "Historical product read-after-write did not return the created form exactly."
        }
        Assert-HistoricalFormScopeProjection `
            -Form $detail `
            -ExpectedScopeNodeTypeId $historicalScope.scope_node_type_id `
            -ExpectedVisibilityNodeId $historicalScope.visibility_node_id `
            -Context "Historical product read-after-write"
        $logs = Get-FinalLogEvidence `
            -Run $run `
            -AdditionalSecrets ([string[]]@([string]$login.token))
        return [ordered]@{
            forms_before = $formsCountBefore
            forms_visible_before = $existingForms.Count
            created_form_id = $created.id
            created_form_slug = $formSlug
            source_form_id = $historicalScope.source_form_id
            scope_node_type_id = $historicalScope.scope_node_type_id
            visibility_node_id = $historicalScope.visibility_node_id
            logs = $logs
        }
    } finally {
        Stop-ValidationProcess $run
        Remove-ValidationLogs $run
    }
}

function Write-Evidence {
    param([Parameter(Mandatory)]$Evidence)

    $path = if ([string]::IsNullOrWhiteSpace($EvidencePath)) {
        $stamp = [DateTime]::UtcNow.ToString("yyyyMMddTHHmmssfffZ")
        Join-Path $repositoryRoot "artifacts/sprint-6a/evidence/rollback-$($Mode.ToLowerInvariant())-$stamp.json"
    } elseif ([IO.Path]::IsPathRooted($EvidencePath)) {
        [IO.Path]::GetFullPath($EvidencePath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repositoryRoot $EvidencePath))
    }
    $packagePrefix = $packageFullPath.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if ($path.StartsWith($packagePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Validation evidence must be written outside the immutable rollback package."
    }
    $parent = Split-Path -Parent $path
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    $json = $Evidence | ConvertTo-Json -Depth 12
    [IO.File]::WriteAllText($path, $json + "`n", [Text.UTF8Encoding]::new($false))
    return $path
}

$manifest = Assert-PackageManifest
$baseEvidence = [ordered]@{
    schema_version = 1
    validation_mode = $Mode
    validated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
    validator_script_sha256 = Get-Sha256 $PSCommandPath
    validator_common_helper_sha256 = Get-Sprint6AFileSha256 $script:Sprint6ARestoreEvidenceCommonHelperPath
    bind_address = $BindAddress
    package_content_sha256 = $manifest.package_content.sha256
    sprint_5a_source_commit = $manifest.application.sprint_5a_source_commit
    closing_sprint_6a_repository_commit = $manifest.closing_sprint_6a_repository_commit
    sprint_5a_binary_sha256 = $manifest.application.binary_sha256
    migration_sha256 = @($manifest.migrations | ForEach-Object {
        [ordered]@{ version = $_.version; sha256 = $_.sha256 }
    })
    invariant_fingerprint_contract = "all declared control-plane tables and all user-managed role mappings; admin/operator/respondent mappings are asserted separately as exact versioned seed contracts"
    declared_built_in_seed_contracts = @(
        [ordered]@{
            contract_id = $sprint6ASeedContract.contract_id
            source_commit = $sprint6ASeedContract.source_commit
            contract_canonical_format = $sprint6ASeedContract.contract_canonical_format
            contract_sha256 = $sprint6ASeedContract.contract_sha256
            mapping_set_canonical_format = $sprint6ASeedContract.snapshot.canonical_format
            mapping_set_sha256 = $sprint6ASeedContract.snapshot.canonical_sha256
            mapping_count = $sprint6ASeedContract.snapshot.mapping_count
        },
        [ordered]@{
            contract_id = $sprint5ASeedContract.contract_id
            source_commit = $sprint5ASeedContract.source_commit
            contract_canonical_format = $sprint5ASeedContract.contract_canonical_format
            contract_sha256 = $sprint5ASeedContract.contract_sha256
            mapping_set_canonical_format = $sprint5ASeedContract.snapshot.canonical_format
            mapping_set_sha256 = $sprint5ASeedContract.snapshot.canonical_sha256
            mapping_count = $sprint5ASeedContract.snapshot.mapping_count
        }
    )
}

if ($Mode -eq "PackageOnly") {
    $baseEvidence.result = "passed"
    $evidenceFile = Write-Evidence $baseEvidence
    Write-Host "Rollback package manifest and all payload digests are valid."
    Write-Host "Evidence: $evidenceFile"
    return
}

if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
    throw "-DatabaseUrl is required for '$Mode' validation."
}
if ([string]::IsNullOrWhiteSpace($ExpectedDatabaseName)) {
    throw "-ExpectedDatabaseName is required for '$Mode' validation so the writable clone is named explicitly."
}
if (-not (Test-DisposableDatabaseName $ExpectedDatabaseName)) {
    throw "Expected database name '$ExpectedDatabaseName' is not clearly disposable; include a standalone test, upgrade, clone, rollback, or sprint-6a token in the clone name."
}
$databaseUrlEndpoint = ConvertFrom-Sprint6APostgresDatabaseUrl `
    -DatabaseUrl $DatabaseUrl `
    -Context "Rollback validation database URL"
Assert-Sprint6AExactDatabaseName `
    -Actual $databaseUrlEndpoint.database_name `
    -Expected $ExpectedDatabaseName `
    -Context "Rollback validation database URL"
$postgresClientContext = New-Sprint6APostgresClientContext `
    -DatabaseUrls ([string[]]@($DatabaseUrl)) `
    -PostgresClientContainerId $PostgresClientContainerId `
    -PsqlCommand $PsqlCommand `
    -PgDumpCommand $PgDumpCommand `
    -PgRestoreCommand $PgRestoreCommand `
    -RequiredTools ([string[]]@("psql", "pg_dump", "pg_restore"))

$databaseName = Invoke-PsqlScalar "SELECT current_database();"
Assert-Sprint6AExactDatabaseName `
    -Actual $databaseName `
    -Expected $ExpectedDatabaseName `
    -Context "Rollback validation database connection"
$baseEvidence.database_name = $databaseName
$baseEvidence.postgres_client = Get-Sprint6APostgresClientEvidence $postgresClientContext

if ($Mode -eq "CompatibilityOnUpgraded") {
    $ledgerBefore = @(Assert-Ledger @(1, 2, 3))
    $controlShape = Assert-UpgradedControlPlaneReady
    $builtInSeedBefore = Assert-BuiltInSeedContract `
        -Actual (Get-BuiltInSeedSnapshot) `
        -ExpectedContract $sprint6ASeedContract `
        -Context "Upgraded clone before rollback-package startup"
    $controlBefore = Get-ControlPlaneFingerprint
    $originalRejection = Assert-OriginalPackageRejectsUpgradedDatabase
    $builtInSeedAfterRejection = Assert-BuiltInSeedContract `
        -Actual (Get-BuiltInSeedSnapshot) `
        -ExpectedContract $sprint6ASeedContract `
        -Context "Upgraded clone after rejected original-package startup"
    $controlAfterRejection = Get-ControlPlaneFingerprint
    if ($controlAfterRejection -ne $controlBefore) {
        throw "Rejected original-package startup changed migration-3 control-plane state."
    }

    $smoke = Invoke-ProductSmoke (Join-Path $packageFullPath "migrations")
    $ledgerAfter = @(Assert-Ledger @(1, 2, 3))
    $builtInSeedAfter = Assert-BuiltInSeedContract `
        -Actual (Get-BuiltInSeedSnapshot) `
        -ExpectedContract $sprint5ASeedContract `
        -Context "Upgraded clone after exact Sprint 5A-code compatibility-package startup"
    $controlAfter = Get-ControlPlaneFingerprint
    if ($controlAfter -ne $controlBefore) {
        throw "Sprint 5A compatibility package changed migration-3 control-plane or user-managed role state outside the separately asserted built-in seed contract."
    }
    $formsCountAfter = [int64](Invoke-PsqlScalar "SELECT COUNT(*) FROM forms;")
    if ($formsCountAfter -ne ([int64]$smoke.forms_before + 1)) {
        throw "Historical product write did not persist exactly one new form."
    }

    $baseEvidence.ledger_before = $ledgerBefore
    $baseEvidence.ledger_after = $ledgerAfter
    $baseEvidence.control_plane_shape = $controlShape
    $baseEvidence.built_in_seed_before = $builtInSeedBefore
    $baseEvidence.built_in_seed_after_original_rejection = $builtInSeedAfterRejection
    $baseEvidence.built_in_seed_after_compatibility_smoke = $builtInSeedAfter
    $baseEvidence.built_in_seed_transition = [ordered]@{
        transition = "$($sprint6ASeedContract.contract_id)_to_$($sprint5ASeedContract.contract_id)"
        allowed_difference = "exact Sprint 5A startup restores redundant direct product-capability rows only on admin; operator and respondent mappings are identical"
        effective_admin_authority_unchanged = "admin:all was present before and remains universally implying"
    }
    $baseEvidence.control_plane_sha256_before = $controlBefore
    $baseEvidence.control_plane_sha256_after_original_rejection = $controlAfterRejection
    $baseEvidence.control_plane_sha256_after_compatibility_smoke = $controlAfter
    $baseEvidence.original_package_on_upgraded_database = $originalRejection
    $baseEvidence.compatibility_product_smoke = $smoke
    $baseEvidence.forms_after = $formsCountAfter
    $baseEvidence.result = "passed"
} else {
    if ([string]::IsNullOrWhiteSpace($RestoreEvidencePath)) {
        throw "-RestoreEvidencePath is required for OriginalAfterRestore and must identify structured evidence generated from the retained backup and verified restore."
    }
    $restoreEvidenceFullPath = if ([IO.Path]::IsPathRooted($RestoreEvidencePath)) {
        [IO.Path]::GetFullPath($RestoreEvidencePath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repositoryRoot $RestoreEvidencePath))
    }
    $restoreEvidence = Assert-Sprint6ARestoreEvidenceDocument `
        -EvidencePath $restoreEvidenceFullPath `
        -ExpectedTargetDatabaseName $ExpectedDatabaseName
    $validationClientEvidence = Get-Sprint6APostgresClientEvidence $postgresClientContext
    if ($restoreEvidence.document.postgres_client.mode -ne $validationClientEvidence.mode) {
        throw "Restore evidence PostgreSQL client mode does not match OriginalAfterRestore validation client mode."
    }
    if ($validationClientEvidence.mode -eq "docker_container") {
        foreach ($identityField in @("container_id", "container_name", "image_reference", "image_id")) {
            if ($restoreEvidence.document.postgres_client.$identityField -ne $validationClientEvidence.$identityField) {
                throw "Restore evidence PostgreSQL client $identityField does not match the exact container used for OriginalAfterRestore validation."
            }
        }
        $restoreTargetBindings = @($restoreEvidence.document.postgres_client.validated_host_bindings | Where-Object {
            $_.database_name -ceq $ExpectedDatabaseName
        })
        $validationTargetBindings = @($validationClientEvidence.validated_host_bindings | Where-Object {
            $_.database_name -ceq $ExpectedDatabaseName
        })
        if ($restoreTargetBindings.Count -ne 1 -or $validationTargetBindings.Count -ne 1 -or
            ($restoreTargetBindings[0] | ConvertTo-Json -Compress) -cne
                ($validationTargetBindings[0] | ConvertTo-Json -Compress)) {
            throw "Restore evidence target host binding does not match the exact sanitized binding revalidated for OriginalAfterRestore."
        }
    } else {
        foreach ($tool in @("psql", "pg_dump", "pg_restore")) {
            if ($restoreEvidence.document.postgres_client.tool_commands.$tool -cne $validationClientEvidence.tool_commands.$tool -or
                $restoreEvidence.document.postgres_client.tool_sha256.$tool -cne $validationClientEvidence.tool_sha256.$tool) {
                throw "Restore evidence local PostgreSQL executable identity for '$tool' does not match OriginalAfterRestore validation."
            }
        }
    }
    $ledgerBefore = @(Assert-Ledger @(1, 2))
    if (($ledgerBefore -join ",") -ne (@($restoreEvidence.document.restored_target_database.migration_ledger) -join ",")) {
        throw "Validated database migration ledger does not equal the structured restore-evidence target ledger."
    }
    $restoredDatabaseFingerprint = Get-Sprint6ADatabaseFingerprint `
        -DatabaseUrl $DatabaseUrl `
        -PsqlCommand $PsqlCommand `
        -PostgresClientContext $postgresClientContext
    $restoredDatabaseFingerprintJson = $restoredDatabaseFingerprint | ConvertTo-Json -Depth 4 -Compress
    $evidenceTargetFingerprintJson = $restoreEvidence.document.restored_target_database.fingerprint | ConvertTo-Json -Depth 4 -Compress
    if ($restoredDatabaseFingerprintJson -ne $evidenceTargetFingerprintJson) {
        throw "Validated database fingerprint does not equal the structured restore-evidence target fingerprint."
    }
    $controlPlaneExists = Invoke-PsqlScalar "SELECT CASE WHEN to_regclass('public.application_installations') IS NULL THEN 'false' ELSE 'true' END;"
    if ($controlPlaneExists -ne "false") {
        throw "Original historical package may start only after restoring a pre-migration-3 backup."
    }
    $builtInSeedBefore = Assert-BuiltInSeedContract `
        -Actual (Get-BuiltInSeedSnapshot) `
        -ExpectedContract $sprint5ASeedContract `
        -Context "Restored Sprint 5A clone before original-package startup"
    $controlBefore = Get-ControlPlaneFingerprint
    $smoke = Invoke-ProductSmoke (Join-Path $packageFullPath "original-migrations")
    $ledgerAfter = @(Assert-Ledger @(1, 2))
    $builtInSeedAfter = Assert-BuiltInSeedContract `
        -Actual (Get-BuiltInSeedSnapshot) `
        -ExpectedContract $sprint5ASeedContract `
        -Context "Restored Sprint 5A clone after original-package startup"
    $controlAfter = Get-ControlPlaneFingerprint
    if ($controlAfter -ne $controlBefore) {
        throw "Original Sprint 5A package changed restored control-plane or user-managed role state outside the separately asserted built-in seed contract."
    }
    $formsCountAfter = [int64](Invoke-PsqlScalar "SELECT COUNT(*) FROM forms;")
    if ($formsCountAfter -ne ([int64]$smoke.forms_before + 1)) {
        throw "Original historical product write did not persist exactly one new form after restore."
    }

    $baseEvidence.ledger_before = $ledgerBefore
    $baseEvidence.ledger_after = $ledgerAfter
    $baseEvidence.restore_evidence = [ordered]@{
        schema_version = $restoreEvidence.document.schema_version
        evidence_kind = $restoreEvidence.document.evidence_kind
        postgres_client = $restoreEvidence.document.postgres_client
        path = $restoreEvidence.path
        sha256 = $restoreEvidence.sha256
        backup_artifact = [ordered]@{
            path = $restoreEvidence.backup_path
            sha256 = $restoreEvidence.backup_sha256
            length_bytes = $restoreEvidence.backup_length_bytes
        }
        source_database_name = $restoreEvidence.document.source_database.database_name
        restored_target_database_name = $restoreEvidence.document.restored_target_database.database_name
        source_migration_ledger = @($restoreEvidence.document.source_database.migration_ledger)
        restored_target_migration_ledger = @($restoreEvidence.document.restored_target_database.migration_ledger)
        source_database_fingerprint = $restoreEvidence.document.source_database.fingerprint
        restored_target_database_fingerprint = $restoreEvidence.document.restored_target_database.fingerprint
        independently_observed_target_database_fingerprint = $restoredDatabaseFingerprint
    }
    $baseEvidence.pre_upgrade_database_state_verified = $true
    $baseEvidence.built_in_seed_before = $builtInSeedBefore
    $baseEvidence.built_in_seed_after_original_smoke = $builtInSeedAfter
    $baseEvidence.built_in_seed_transition = [ordered]@{
        transition = "$($sprint5ASeedContract.contract_id)_to_$($sprint5ASeedContract.contract_id)"
        allowed_difference = "none"
    }
    $baseEvidence.control_plane_sha256_before = $controlBefore
    $baseEvidence.control_plane_sha256_after_original_smoke = $controlAfter
    $baseEvidence.original_product_smoke = $smoke
    $baseEvidence.forms_after = $formsCountAfter
    $baseEvidence.result = "passed"
}

$evidenceFile = Write-Evidence $baseEvidence
Write-Host "Sprint 6A rollback validation '$Mode' passed."
Write-Host "Evidence: $evidenceFile"

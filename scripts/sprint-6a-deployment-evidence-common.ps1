$script:Sprint6ADeploymentEvidenceKind = "tessara.sprint-6a.deployment-evidence"
$script:Sprint6ABuiltInSeedVersion = "sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68"
$script:Sprint6ABuiltInSeedSha256 = "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600"
$script:Sprint6AExpectedDefinitions = @(
    "tessara.components",
    "tessara.dashboards",
    "tessara.datasets",
    "tessara.forms",
    "tessara.migration",
    "tessara.responses",
    "tessara.workflows"
)
$script:Sprint6AExpectedTransitionFixtureDigests = [ordered]@{
    "transition-components-v1.json" = "sha256:344388304b015421ea71b5e303e7b9699264aef51c116b56d7f52e1b92443499"
    "transition-dashboards-v1.json" = "sha256:c82ecc7c3d121d1e1498c130133e487c8a68899b9255951e97955ce0de76bbe5"
    "transition-datasets-v1.json" = "sha256:ca301f4ac9a589d498bc25c77de4223b33de90569ecf54974976424c07fb4614"
    "transition-forms-v1.json" = "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e"
    "transition-migration-v1.json" = "sha256:de48eeb3edb4a432e5060b817ef50c34c5316879b44aef0ad3d6877c5895b42e"
    "transition-responses-v1.json" = "sha256:e491986ed43b0f290f0c2ee763e60afb03e5b7babc7117a11e280e37de7b91bc"
    "transition-workflows-v1.json" = "sha256:e9bdf51896700ffb982a00e4c80ea198bbdb98056705036a1a948347a71c04cf"
}
$script:Sprint6AExpectedSeed = [ordered]@{
    admin = @("admin:all")
    operator = @(
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
    respondent = @("submissions:read_own", "submissions:respond")
}

function Resolve-Sprint6ARepositoryPath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Path
    )

    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $Path))
}

function Get-Sprint6ASha256Text {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $algorithm.ComputeHash($bytes)
        ([BitConverter]::ToString($hash)).Replace("-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Invoke-Sprint6ANative {
    param(
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$Context
    )

    $previousPreference = $ErrorActionPreference
    $previousNativePreference = $PSNativeCommandUseErrorActionPreference
    $ErrorActionPreference = "Continue"
    $PSNativeCommandUseErrorActionPreference = $false
    try {
        $output = @(& $Command @Arguments 2>&1)
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousPreference
        $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    }
    if ($exitCode -ne 0) {
        throw "$Context failed with exit code $exitCode`: $($output -join [Environment]::NewLine)"
    }
    ($output -join [Environment]::NewLine).Trim()
}

function Get-Sprint6AComposeContainerId {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][ValidateSet("api", "postgres")][string]$Service
    )

    Push-Location $RepositoryRoot
    try {
        $raw = Invoke-Sprint6ANative `
            -Command "docker" `
            -Arguments @("compose", "ps", "-q", $Service) `
            -Context "locating the running Compose $Service container"
    } finally {
        Pop-Location
    }
    $ids = @($raw -split "\r?\n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($ids.Count -ne 1 -or $ids[0] -notmatch "^[0-9a-f]{12,64}$") {
        throw "Expected exactly one running Compose $Service container, found '$raw'."
    }
    $ids[0]
}

function Get-Sprint6AContainerInspect {
    param(
        [Parameter(Mandatory)][string]$ContainerId,
        [Parameter(Mandatory)][string]$Context
    )

    $raw = Invoke-Sprint6ANative `
        -Command "docker" `
        -Arguments @("container", "inspect", $ContainerId) `
        -Context "inspecting $Context container"
    $rows = @(ConvertFrom-Json -InputObject $raw)
    if ($rows.Count -ne 1) {
        throw "Docker returned $($rows.Count) inspect records for $Context container '$ContainerId'."
    }
    $rows[0]
}

function Get-Sprint6AImageInspect {
    param([Parameter(Mandatory)][string]$ImageId)

    $raw = Invoke-Sprint6ANative `
        -Command "docker" `
        -Arguments @("image", "inspect", $ImageId) `
        -Context "inspecting the running Tessara image"
    $rows = @(ConvertFrom-Json -InputObject $raw)
    if ($rows.Count -ne 1) {
        throw "Docker returned $($rows.Count) image records for '$ImageId'."
    }
    $rows[0]
}

function Get-Sprint6ADatabaseDocument {
    param(
        [Parameter(Mandatory)][string]$DatabaseContainerId,
        [Parameter(Mandatory)][string]$DatabaseName,
        [Parameter(Mandatory)][string]$DatabaseUser
    )

    if ($DatabaseName -notmatch "^[A-Za-z_][A-Za-z0-9_-]*$" -or
        $DatabaseUser -notmatch "^[A-Za-z_][A-Za-z0-9_-]*$") {
        throw "The live API DATABASE_URL contains an unsupported database name or user."
    }

    $sql = @'
WITH seed_roles AS (
    SELECT r.name,
           COALESCE(jsonb_agg(c.key ORDER BY c.key) FILTER (WHERE c.key IS NOT NULL), '[]'::jsonb) AS capabilities
    FROM roles r
    LEFT JOIN role_capabilities rc ON rc.role_id = r.id
    LEFT JOIN capabilities c ON c.id = rc.capability_id
    WHERE r.name IN ('admin', 'operator', 'respondent')
    GROUP BY r.id, r.name
),
catalog_rows AS (
    SELECT tc.definition_id, source.source_digest
    FROM transition_catalog_current tc
    JOIN transition_descriptor_sources source ON source.id = tc.source_id
)
SELECT jsonb_build_object(
    'current_database', current_database(),
    'migrations', (
        SELECT COALESCE(jsonb_agg(jsonb_build_object(
            'version', version,
            'description', description,
            'installed_on', installed_on,
            'success', success,
            'checksum_sha384', encode(checksum, 'hex')
        ) ORDER BY version), '[]'::jsonb)
        FROM _sqlx_migrations
    ),
    'installation', jsonb_build_object(
        'row_count', (SELECT count(*) FROM application_installations),
        'id', (SELECT id FROM application_installations WHERE singleton = true),
        'created_at', (SELECT created_at FROM application_installations WHERE singleton = true),
        'runtime_observation_count', (SELECT count(*) FROM core_runtime_observations),
        'runtime_observation', (
            SELECT jsonb_build_object(
                'provenance', provenance,
                'observed_version', observed_version,
                'finding_code', finding_code,
                'observed_at', observed_at
            )
            FROM core_runtime_observations
        )
    ),
    'seed_roles', (
        SELECT COALESCE(jsonb_agg(jsonb_build_object(
            'name', name,
            'capabilities', capabilities
        ) ORDER BY name), '[]'::jsonb)
        FROM seed_roles
    ),
    'catalog', jsonb_build_object(
        'definition_count', (SELECT count(*) FROM module_definition_reservations),
        'source_count', (SELECT count(*) FROM transition_descriptor_sources),
        'projection_count', (SELECT count(*) FROM transition_catalog_projections),
        'current_count', (SELECT count(*) FROM transition_catalog_current),
        'navigation_contribution_count', (SELECT count(*) FROM module_navigation_contributions),
        'navigation_policy_count', (SELECT count(*) FROM navigation_policies),
        'release_table_absent', to_regclass('public.module_releases') IS NULL,
        'instance_table_absent', to_regclass('public.module_instances') IS NULL,
        'policy_entries', (
            SELECT COALESCE(jsonb_agg(jsonb_build_object(
                'contribution_id', contribution_id,
                'visible', visible,
                'policy_order', policy_order
            ) ORDER BY contribution_id), '[]'::jsonb)
            FROM navigation_policy_entries
        ),
        'entries', (
            SELECT COALESCE(jsonb_agg(jsonb_build_object(
                'definition_id', definition_id,
                'source_digest', source_digest
            ) ORDER BY definition_id), '[]'::jsonb)
            FROM catalog_rows
        )
    )
)::text;
'@

    $raw = Invoke-Sprint6ANative `
        -Command "docker" `
        -Arguments @(
            "exec", "-i", $DatabaseContainerId,
            "psql", "-X", "-v", "ON_ERROR_STOP=1", "-U", $DatabaseUser,
            "-d", $DatabaseName, "-At", "-c", $sql
        ) `
        -Context "reading the live Sprint 6A database evidence"
    try {
        ConvertFrom-Json -InputObject $raw
    } catch {
        throw "The live database evidence was not valid JSON: $($_.Exception.Message)"
    }
}

function Remove-Sprint6AEvidenceSession {
    param(
        [Parameter(Mandatory)][string]$DatabaseContainerId,
        [Parameter(Mandatory)][string]$DatabaseName,
        [Parameter(Mandatory)][string]$DatabaseUser,
        [Parameter(Mandatory)][string]$Token
    )

    if ($Token -notmatch "^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$") {
        throw "The evidence-only login returned a non-UUID session token that cannot be safely removed."
    }
    $sql = "WITH removed AS (DELETE FROM auth_sessions WHERE token = '$Token'::uuid RETURNING 1) SELECT count(*) FROM removed;"
    $removed = Invoke-Sprint6ANative `
        -Command "docker" `
        -Arguments @(
            "exec", "-i", $DatabaseContainerId,
            "psql", "-X", "-v", "ON_ERROR_STOP=1", "-U", $DatabaseUser,
            "-d", $DatabaseName, "-At", "-c", $sql
        ) `
        -Context "removing the evidence-only authentication session"
    if ($removed.Trim() -cne "1") {
        throw "Evidence-session cleanup removed '$removed' rows instead of exactly one."
    }
}

function Get-Sprint6AExpectedMigrationLedger {
    param([Parameter(Mandatory)][string]$RepositoryRoot)

    $migrationDirectory = Join-Path $RepositoryRoot "crates/tessara-api/migrations"
    $rows = @()
    foreach ($file in @(Get-ChildItem -LiteralPath $migrationDirectory -Filter "*.sql" -File | Sort-Object Name)) {
        if ($file.Name -notmatch "^(?<version>[0-9]+)_(?<description>.+)\.sql$") {
            throw "Migration filename '$($file.Name)' does not follow the version_description.sql contract."
        }
        $rows += [pscustomobject][ordered]@{
            version = [int64]$Matches.version
            file = $file.Name
            checksum_sha384 = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA384).Hash.ToLowerInvariant()
        }
    }
    if (($rows.version -join ",") -cne "1") {
        throw "Fresh-sprint deployment evidence requires repository migration exactly 1; found '$($rows.version -join ",")'."
    }
    $rows
}

function Get-Sprint6AApiPackageContract {
    param([Parameter(Mandatory)][string]$RepositoryRoot)

    $manifestPath = Join-Path $RepositoryRoot "crates/tessara-api/Cargo.toml"
    $manifestText = Get-Content -LiteralPath $manifestPath -Raw
    $versionMatch = [regex]::Match($manifestText, '(?m)^version\s*=\s*"(?<version>[^"]+)"\s*$')
    if (-not $versionMatch.Success -or $versionMatch.Groups["version"].Value -notmatch "^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$") {
        throw "Could not derive the Tessara API package version from '$manifestPath'."
    }
    [pscustomobject][ordered]@{
        version = $versionMatch.Groups["version"].Value
        manifest = "crates/tessara-api/Cargo.toml"
        manifest_sha256 = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

function ConvertTo-Sprint6AUtcTimestamp {
    param([Parameter(Mandatory)]$Value)

    ([DateTimeOffset]$Value).ToUniversalTime().ToString("o")
}

function Assert-Sprint6AMigrationLedger {
    param(
        [Parameter(Mandatory)][object[]]$DatabaseMigrations,
        [Parameter(Mandatory)][object[]]$ExpectedMigrations
    )

    if ($DatabaseMigrations.Count -ne 1 -or ($DatabaseMigrations.version -join ",") -cne "1") {
        throw "The live database migration ledger must contain exactly successful version 1."
    }
    for ($index = 0; $index -lt 1; $index++) {
        $database = $DatabaseMigrations[$index]
        $expected = $ExpectedMigrations[$index]
        if (-not [bool]$database.success -or
            [int64]$database.version -ne [int64]$expected.version -or
            [string]$database.checksum_sha384 -cne [string]$expected.checksum_sha384) {
            throw "Migration $($expected.version) in the live database does not match '$($expected.file)' or was not successful."
        }
    }
}

function Get-Sprint6ASeedContract {
    param([Parameter(Mandatory)][object[]]$SeedRoles)

    if ($SeedRoles.Count -ne 3) {
        throw "The live database must contain exactly the three built-in roles admin, operator, and respondent."
    }
    $canonical = [Text.StringBuilder]::new()
    foreach ($roleName in $script:Sprint6AExpectedSeed.Keys) {
        $role = @($SeedRoles | Where-Object { [string]$_.name -ceq $roleName })
        if ($role.Count -ne 1) {
            throw "The live database does not contain exactly one built-in '$roleName' role."
        }
        $expectedCapabilities = @($script:Sprint6AExpectedSeed[$roleName])
        $actualCapabilities = @($role[0].capabilities | Sort-Object)
        $expectedSorted = @($expectedCapabilities | Sort-Object)
        if (($actualCapabilities -join "`n") -cne ($expectedSorted -join "`n")) {
            throw "The live '$roleName' membership differs from the versioned Sprint 6A built-in seed contract."
        }
        [void]$canonical.Append("role=$roleName`n")
        foreach ($capability in $expectedCapabilities) {
            [void]$canonical.Append("capability=$capability`n")
        }
    }
    $digest = Get-Sprint6ASha256Text -Text $canonical.ToString()
    if ($digest -cne $script:Sprint6ABuiltInSeedSha256) {
        throw "The computed live built-in seed digest '$digest' does not match the versioned Sprint 6A contract."
    }
    [pscustomobject][ordered]@{
        version = $script:Sprint6ABuiltInSeedVersion
        canonical_sha256 = $digest
        roles = $SeedRoles
    }
}

function Get-Sprint6AExpectedCatalogEntries {
    param([Parameter(Mandatory)][string]$RepositoryRoot)

    $fixtureDirectory = Join-Path $RepositoryRoot "crates/tessara-module-contract/tests/fixtures"
    $entries = @()
    $actualSidecars = @(
        Get-ChildItem -LiteralPath $fixtureDirectory -Filter "transition-*-v1.json.sha256" -File |
            Sort-Object Name
    )
    $expectedSidecars = @($script:Sprint6AExpectedTransitionFixtureDigests.Keys | ForEach-Object { "$_.sha256" })
    if (($actualSidecars.Name -join "`n") -cne ($expectedSidecars -join "`n")) {
        throw "The repository transition fixture sidecars differ from the independently pinned Sprint 6A inventory."
    }
    $strictUtf8 = [Text.UTF8Encoding]::new($false, $true)
    foreach ($fixtureName in $script:Sprint6AExpectedTransitionFixtureDigests.Keys) {
        $sourcePath = Join-Path $fixtureDirectory $fixtureName
        $sidecarPath = "$sourcePath.sha256"
        if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf) -or
            -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
            throw "Transition fixture '$fixtureName' and its digest sidecar are both required."
        }
        $sourceBytes = [IO.File]::ReadAllBytes($sourcePath)
        if ($sourceBytes.Length -lt 2 -or
            ($sourceBytes.Length -ge 3 -and
                $sourceBytes[0] -eq 0xef -and
                $sourceBytes[1] -eq 0xbb -and
                $sourceBytes[2] -eq 0xbf) -or
            $sourceBytes -contains 0 -or
            $sourceBytes -contains 13 -or
            $sourceBytes[$sourceBytes.Length - 1] -ne 10 -or
            $sourceBytes[$sourceBytes.Length - 2] -eq 10) {
            throw "Transition source fixture '$sourcePath' must be nonempty UTF-8 without BOM/NUL and use LF-only bytes with exactly one terminal LF."
        }
        try {
            $sourceText = $strictUtf8.GetString($sourceBytes)
            $source = $sourceText | ConvertFrom-Json
        } catch {
            throw "Transition source fixture '$sourcePath' is not valid JSON: $($_.Exception.Message)"
        }
        try {
            $sidecarText = $strictUtf8.GetString([IO.File]::ReadAllBytes($sidecarPath))
        } catch {
            throw "Transition digest sidecar '$sidecarPath' is not strict UTF-8: $($_.Exception.Message)"
        }
        $definitionId = [string]$source.reserved_definition_id
        $expectedDigest = [string]$script:Sprint6AExpectedTransitionFixtureDigests[$fixtureName]
        $declaredDigest = $sidecarText.TrimEnd("`n")
        $computedDigest = "sha256:" + (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ([int]$source.schema_version -ne 1 -or
            $definitionId -notmatch "^[a-z0-9]+([.:_-][a-z0-9]+)*$" -or
            $sidecarText -cne "$expectedDigest`n" -or
            $declaredDigest -cne $expectedDigest -or
            $computedDigest -cne $expectedDigest) {
            throw "Transition source fixture '$sourcePath' does not match its independently pinned schema-v1 digest and exact LF sidecar."
        }
        $entries += [pscustomobject][ordered]@{
            definition_id = $definitionId
            source_digest = $declaredDigest
            fixture = [IO.Path]::GetFileName($sourcePath)
            sidecar = [IO.Path]::GetFileName($sidecarPath)
        }
    }
    $entries = @($entries | Sort-Object definition_id)
    if ($entries.Count -ne 7 -or
        ($entries.definition_id -join ",") -cne ($script:Sprint6AExpectedDefinitions -join ",")) {
        throw "The repository must contain exactly one immutable schema-v1 source/digest fixture for every Sprint 6A transition definition."
    }
    $entries
}

function Assert-Sprint6ACatalog {
    param(
        [Parameter(Mandatory)][object]$DatabaseCatalog,
        [Parameter(Mandatory)][object]$Inventory,
        [Parameter(Mandatory)][object[]]$ExpectedEntries
    )

    if ([int]$DatabaseCatalog.definition_count -ne 7 -or
        [int]$DatabaseCatalog.source_count -ne 7 -or
        [int]$DatabaseCatalog.projection_count -ne 7 -or
        [int]$DatabaseCatalog.current_count -ne 7 -or
        [int]$DatabaseCatalog.navigation_contribution_count -ne 6 -or
        [int]$DatabaseCatalog.navigation_policy_count -ne 1 -or
        @($DatabaseCatalog.policy_entries).Count -ne 6) {
        throw "The live database does not expose the exact Sprint 6A transition catalog shape."
    }
    if ([int]$Inventory.schema_version -ne 1) {
        throw "The live module inventory API did not expose schema version 1."
    }
    $databaseEntries = @($DatabaseCatalog.entries | Sort-Object definition_id)
    $expectedEntriesSorted = @($ExpectedEntries | Sort-Object definition_id)
    $apiEntries = @($Inventory.entries | ForEach-Object {
        [pscustomobject][ordered]@{
            definition_id = [string]$_.descriptor.reserved_definition_id
            source_digest = [string]$_.source_digest
        }
    } | Sort-Object definition_id)
    if (($databaseEntries.definition_id -join ",") -cne ($script:Sprint6AExpectedDefinitions -join ",") -or
        ($apiEntries.definition_id -join ",") -cne ($script:Sprint6AExpectedDefinitions -join ",")) {
        throw "The live database/API catalog identities differ from the exact seven Sprint 6A transition definitions."
    }
    for ($index = 0; $index -lt 7; $index++) {
        if ([string]$databaseEntries[$index].source_digest -cne [string]$apiEntries[$index].source_digest -or
            [string]$databaseEntries[$index].source_digest -cne [string]$expectedEntriesSorted[$index].source_digest -or
            [string]$databaseEntries[$index].definition_id -cne [string]$expectedEntriesSorted[$index].definition_id) {
            throw "Catalog source provenance differs from the immutable repository fixture, live database, or API for '$($databaseEntries[$index].definition_id)'."
        }
    }
    $databaseEntries
}

function ConvertTo-Sprint6AConfigSequenceJson {
    param(
        [Parameter(Mandatory)]$Config,
        [Parameter(Mandatory)]
        [ValidateSet("Cmd", "Entrypoint")]
        [string]$PropertyName
    )

    $property = $Config.PSObject.Properties[$PropertyName]
    if ($null -eq $property -or $null -eq $property.Value) {
        return "[]"
    }
    $values = @($property.Value)
    if ($values.Count -eq 0) {
        return "[]"
    }
    return $values | ConvertTo-Json -Compress
}

function ConvertTo-Sprint6AConfigScalar {
    param(
        [Parameter(Mandatory)]$Config,
        [Parameter(Mandatory)]
        [ValidateSet("User", "WorkingDir")]
        [string]$PropertyName
    )

    $property = $Config.PSObject.Properties[$PropertyName]
    if ($null -eq $property -or $null -eq $property.Value) {
        return [string]::Empty
    }
    if ($property.Value -isnot [string]) {
        throw "Docker config property '$PropertyName' must be a string when present."
    }
    return [string]$property.Value
}

function ConvertFrom-Sprint6ADeploymentEvidenceJson {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Json
    )

    try {
        ConvertFrom-Json -InputObject $Json -DateKind String
    } catch {
        throw "Deployment evidence is not valid JSON: $($_.Exception.Message)"
    }
}

function Get-Sprint6ADeploymentSnapshot {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$BaseUrl,
        [string]$AdminEmail = "admin@tessara.local",
        [string]$AdminPassword = "tessara-dev-admin",
        [string]$ApiContainerId,
        [string]$GatewayContainerId,
        [string]$DatabaseContainerId
    )

    $baseUri = [Uri]$BaseUrl
    if (-not $baseUri.IsAbsoluteUri -or
        $baseUri.Scheme -notin @("http", "https") -or
        -not $baseUri.IsLoopback) {
        throw "Sprint 6A deployment evidence requires an absolute loopback HTTP(S) BaseUrl."
    }
    $normalizedBaseUrl = $BaseUrl.TrimEnd("/")

    $status = Invoke-Sprint6ANative -Command "git" -Arguments @("-C", $RepositoryRoot, "status", "--porcelain=v1", "--untracked-files=all") -Context "checking the closing source tree"
    if (-not [string]::IsNullOrWhiteSpace($status)) {
        throw "Deployment evidence can only be captured or revalidated from the clean closing source tree."
    }
    $sourceCommit = Invoke-Sprint6ANative -Command "git" -Arguments @("-C", $RepositoryRoot, "rev-parse", "HEAD") -Context "resolving the closing source commit"
    $sourceTree = Invoke-Sprint6ANative -Command "git" -Arguments @("-C", $RepositoryRoot, "rev-parse", "HEAD^{tree}") -Context "resolving the closing source tree"
    if ($sourceCommit -notmatch "^[0-9a-f]{40}$" -or $sourceTree -notmatch "^[0-9a-f]{40}$") {
        throw "Could not resolve an exact closing Git commit and tree."
    }

    if ([string]::IsNullOrWhiteSpace($ApiContainerId)) {
        $ApiContainerId = Get-Sprint6AComposeContainerId -RepositoryRoot $RepositoryRoot -Service api
    }
    if ([string]::IsNullOrWhiteSpace($DatabaseContainerId)) {
        $DatabaseContainerId = Get-Sprint6AComposeContainerId -RepositoryRoot $RepositoryRoot -Service postgres
    }
    $apiInspect = Get-Sprint6AContainerInspect -ContainerId $apiContainerId -Context "API"
    $databaseInspect = Get-Sprint6AContainerInspect -ContainerId $databaseContainerId -Context "database"
    if (-not [bool]$apiInspect.State.Running -or -not [bool]$databaseInspect.State.Running) {
        throw "Sprint 6A deployment evidence requires running API and database containers."
    }
    $apiPortBindings = @($apiInspect.NetworkSettings.Ports."8080/tcp")
    $apiPublishedPorts = @($apiPortBindings | ForEach-Object { $_.HostPort } | Sort-Object -Unique)
    $publishedBaseUrlContainerId = [string]$apiInspect.Id
    if ($apiPublishedPorts.Count -ne 1 -or [int]$apiPublishedPorts[0] -ne $baseUri.Port) {
        if ([string]::IsNullOrWhiteSpace($GatewayContainerId)) {
            throw "The live BaseUrl port is not published by the inspected API container and no gateway container was supplied."
        }
        $gatewayInspect = Get-Sprint6AContainerInspect -ContainerId $GatewayContainerId -Context "gateway"
        if (-not [bool]$gatewayInspect.State.Running) {
            throw "Sprint 6A deployment evidence requires the supplied gateway container to be running."
        }
        $gatewayPortBindings = @($gatewayInspect.NetworkSettings.Ports."8080/tcp")
        $gatewayPublishedPorts = @($gatewayPortBindings | ForEach-Object { $_.HostPort } | Sort-Object -Unique)
        $apiGatewayNetworks = @(
            $apiInspect.NetworkSettings.Networks.PSObject.Properties.Name |
                Where-Object { $gatewayInspect.NetworkSettings.Networks.PSObject.Properties.Name -contains $_ }
        )
        if ($gatewayPublishedPorts.Count -ne 1 -or
            [int]$gatewayPublishedPorts[0] -ne $baseUri.Port -or
            $apiGatewayNetworks.Count -lt 1) {
            throw "The supplied gateway does not uniquely publish the live BaseUrl port or share a network with the API container."
        }
        $publishedBaseUrlContainerId = [string]$gatewayInspect.Id
    }
    $imageInspect = Get-Sprint6AImageInspect -ImageId ([string]$apiInspect.Image)
    $labels = $imageInspect.Config.Labels
    if ([string]$labels."org.opencontainers.image.revision" -cne $sourceCommit -or
        [string]$labels."com.tessara.source-tree" -cne $sourceTree -or
        [string]$labels."com.tessara.build-profile" -cne "release" -or
        [string]$labels."com.tessara.source-dirty" -cne "false") {
        throw "The running API image labels do not identify this exact clean closing source commit/tree and release build."
    }
    if ([string]$imageInspect.Id -cne [string]$apiInspect.Image -or
        [string]$imageInspect.Id -notmatch "^sha256:[0-9a-f]{64}$") {
        throw "The running API container is not bound to one immutable Docker image ID."
    }
    if (@($apiInspect.Mounts).Count -ne 0) {
        throw "The running API container has mounts and therefore is not an unmodified instance of the retained release image."
    }
    $containerDiff = Invoke-Sprint6ANative `
        -Command "docker" `
        -Arguments @("container", "diff", $ApiContainerId) `
        -Context "checking the running API container writable layer"
    $writableLayerChanges = @($containerDiff -split "\r?\n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($writableLayerChanges.Count -ne 0) {
        throw "The running API container writable layer differs from its immutable release image: $($writableLayerChanges -join ', ')."
    }
    $imageCommand = ConvertTo-Sprint6AConfigSequenceJson -Config $imageInspect.Config -PropertyName Cmd
    $containerCommand = ConvertTo-Sprint6AConfigSequenceJson -Config $apiInspect.Config -PropertyName Cmd
    $imageEntrypoint = ConvertTo-Sprint6AConfigSequenceJson -Config $imageInspect.Config -PropertyName Entrypoint
    $containerEntrypoint = ConvertTo-Sprint6AConfigSequenceJson -Config $apiInspect.Config -PropertyName Entrypoint
    $imageWorkingDirectory = ConvertTo-Sprint6AConfigScalar -Config $imageInspect.Config -PropertyName WorkingDir
    $containerWorkingDirectory = ConvertTo-Sprint6AConfigScalar -Config $apiInspect.Config -PropertyName WorkingDir
    $imageUser = ConvertTo-Sprint6AConfigScalar -Config $imageInspect.Config -PropertyName User
    $containerUser = ConvertTo-Sprint6AConfigScalar -Config $apiInspect.Config -PropertyName User
    if ($imageCommand -cne $containerCommand -or
        $imageEntrypoint -cne $containerEntrypoint -or
        $imageWorkingDirectory -cne $containerWorkingDirectory -or
        $imageUser -cne $containerUser) {
        throw "The running API container overrides the immutable release image command, entrypoint, working directory, or user."
    }
    $criticalEnvironment = [ordered]@{
        LEPTOS_SITE_ROOT = "/app/site"
        LEPTOS_SITE_PKG_DIR = "pkg"
        TESSARA_MIGRATIONS_DIR = "/app/migrations"
    }
    foreach ($name in $criticalEnvironment.Keys) {
        $matches = @($apiInspect.Config.Env | Where-Object { $_ -like "$name=*" })
        if ($matches.Count -ne 1 -or $matches[0].Substring($name.Length + 1) -cne $criticalEnvironment[$name]) {
            throw "The running API container overrides critical release-image environment '$name'."
        }
    }

    $databaseUrlEntry = @($apiInspect.Config.Env | Where-Object { $_ -like "DATABASE_URL=*" })
    if ($databaseUrlEntry.Count -ne 1) {
        throw "The running API container must expose exactly one DATABASE_URL."
    }
    $databaseUri = [Uri]($databaseUrlEntry[0].Substring("DATABASE_URL=".Length))
    $databaseName = $databaseUri.AbsolutePath.TrimStart("/")
    $databaseUser = ($databaseUri.UserInfo -split ":", 2)[0]
    if ([string]::IsNullOrWhiteSpace($databaseName) -or [string]::IsNullOrWhiteSpace($databaseUser)) {
        throw "The running API DATABASE_URL did not identify its database and user."
    }
    $apiNetworkNames = @($apiInspect.NetworkSettings.Networks.PSObject.Properties.Name | Sort-Object)
    $databaseNetworkNames = @($databaseInspect.NetworkSettings.Networks.PSObject.Properties.Name | Sort-Object)
    $sharedNetworkNames = @($apiNetworkNames | Where-Object { $databaseNetworkNames -contains $_ })
    $databaseNetworkIps = @(
        $databaseInspect.NetworkSettings.Networks.PSObject.Properties |
            ForEach-Object { $_.Value.IPAddress } |
            Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) } |
            Sort-Object -Unique
    )
    $resolvedDatabaseHosts = Invoke-Sprint6ANative `
        -Command "docker" `
        -Arguments @("exec", $ApiContainerId, "getent", "ahostsv4", $databaseUri.Host) `
        -Context "resolving the live API DATABASE_URL host inside the API container"
    $resolvedDatabaseIps = @(
        $resolvedDatabaseHosts -split "\r?\n" |
            ForEach-Object { ($_ -split "\s+", 2)[0] } |
            Where-Object { $_ -match "^[0-9]+(?:\.[0-9]+){3}$" } |
            Sort-Object -Unique
    )
    if ($resolvedDatabaseIps.Count -lt 1) {
        throw "The API DATABASE_URL host did not resolve inside the API container."
    }
    $directContainerBinding = $databaseUri.Port -eq 5432 -and
        $sharedNetworkNames.Count -gt 0 -and
        @($resolvedDatabaseIps | Where-Object { $databaseNetworkIps -contains $_ }).Count -gt 0
    $databasePortBindings = @($databaseInspect.NetworkSettings.Ports."5432/tcp")
    $databasePublishedPorts = @($databasePortBindings.HostPort | Sort-Object -Unique)
    $publishedHostBinding = $databaseUri.Host -in @("host.docker.internal", "gateway.docker.internal") -and
        $databasePublishedPorts.Count -eq 1 -and
        [int]$databasePublishedPorts[0] -eq $databaseUri.Port
    if (-not $directContainerBinding -and -not $publishedHostBinding) {
        throw "The live API DATABASE_URL is not bound to the inspected database container by direct Docker DNS or its unique published PostgreSQL port."
    }
    $databaseBindingMode = if ($directContainerBinding) { "direct_container_dns" } else { "published_host_port" }

    $evidenceSessionToken = $null
    try {
        $health = Invoke-WebRequest -Uri "$normalizedBaseUrl/health" -TimeoutSec 15 -UseBasicParsing
        if ($health.StatusCode -ne 200 -or $health.Content.Trim() -cne "ok") {
            throw "unexpected health response"
        }
        $login = Invoke-RestMethod `
            -Method Post `
            -Uri "$normalizedBaseUrl/api/auth/login" `
            -ContentType "application/json" `
            -Body (@{ email = $AdminEmail; password = $AdminPassword } | ConvertTo-Json) `
            -TimeoutSec 30
        if ([string]::IsNullOrWhiteSpace([string]$login.token)) {
            throw "admin login did not return a token"
        }
        $evidenceSessionToken = ([guid]$login.token).ToString("D").ToLowerInvariant()
        $inventory = Invoke-RestMethod `
            -Uri "$normalizedBaseUrl/api/admin/modules" `
            -Headers @{ Authorization = "Bearer $($login.token)" } `
            -TimeoutSec 30
    } catch {
        throw "Could not derive deployment evidence from the live Tessara service: $($_.Exception.Message)"
    } finally {
        if (-not [string]::IsNullOrWhiteSpace($evidenceSessionToken)) {
            Remove-Sprint6AEvidenceSession `
                -DatabaseContainerId $DatabaseContainerId `
                -DatabaseName $databaseName `
                -DatabaseUser $databaseUser `
                -Token $evidenceSessionToken
        }
    }

    $database = Get-Sprint6ADatabaseDocument `
        -DatabaseContainerId $databaseContainerId `
        -DatabaseName $databaseName `
        -DatabaseUser $databaseUser
    if ([string]$database.current_database -cne $databaseName) {
        throw "The database queried for evidence is not the database named by the live API DATABASE_URL."
    }
    if ([int]$database.installation.row_count -ne 1 -or
        [int]$database.installation.runtime_observation_count -ne 1 -or
        [string]$database.installation.id -cne [string]$inventory.installation.id) {
        throw "The live API is not bound to the exact single Application Installation read from the database container."
    }

    $apiPackage = Get-Sprint6AApiPackageContract -RepositoryRoot $RepositoryRoot
    $databaseRuntime = $database.installation.runtime_observation
    $databaseInstallationCreatedAt = ConvertTo-Sprint6AUtcTimestamp -Value $database.installation.created_at
    $apiInstallationCreatedAt = ConvertTo-Sprint6AUtcTimestamp -Value $inventory.installation.created_at
    $databaseObservedAt = ConvertTo-Sprint6AUtcTimestamp -Value $databaseRuntime.observed_at
    $apiObservedAt = ConvertTo-Sprint6AUtcTimestamp -Value $inventory.core_runtime.observed_at
    if ($databaseInstallationCreatedAt -cne $apiInstallationCreatedAt -or
        [string]$databaseRuntime.provenance -cne "development_unresolved" -or
        [string]$databaseRuntime.finding_code -cne "core_release_provenance_unresolved" -or
        [string]$databaseRuntime.observed_version -cne [string]$apiPackage.version -or
        [string]$inventory.core_runtime.provenance -cne [string]$databaseRuntime.provenance -or
        [string]$inventory.core_runtime.finding_code -cne [string]$databaseRuntime.finding_code -or
        [string]$inventory.core_runtime.observed_version -cne [string]$databaseRuntime.observed_version -or
        $apiObservedAt -cne $databaseObservedAt) {
        throw "The live API, database Core runtime observation, and current Tessara API package version do not match exactly."
    }

    $expectedMigrations = @(Get-Sprint6AExpectedMigrationLedger -RepositoryRoot $RepositoryRoot)
    Assert-Sprint6AMigrationLedger -DatabaseMigrations @($database.migrations) -ExpectedMigrations $expectedMigrations
    $seedContract = Get-Sprint6ASeedContract -SeedRoles @($database.seed_roles)
    $expectedCatalogEntries = @(Get-Sprint6AExpectedCatalogEntries -RepositoryRoot $RepositoryRoot)
    $catalogEntries = @(
        Assert-Sprint6ACatalog `
            -DatabaseCatalog $database.catalog `
            -Inventory $inventory `
            -ExpectedEntries $expectedCatalogEntries
    )

    [pscustomobject][ordered]@{
        base_url = $normalizedBaseUrl
        source = [pscustomobject][ordered]@{
            commit = $sourceCommit
            tree = $sourceTree
            clean = $true
        }
        release_image = [pscustomobject][ordered]@{
            api_container_id = [string]$apiInspect.Id
            published_base_url_container_id = $publishedBaseUrlContainerId
            published_base_url_port = [int]$baseUri.Port
            image_id = [string]$imageInspect.Id
            image_reference = [string]$apiInspect.Config.Image
            image_created = [string]$imageInspect.Created
            source_commit_label = [string]$labels."org.opencontainers.image.revision"
            source_tree_label = [string]$labels."com.tessara.source-tree"
            build_profile_label = [string]$labels."com.tessara.build-profile"
            source_dirty_label = [string]$labels."com.tessara.source-dirty"
            repo_digests = @($imageInspect.RepoDigests | Sort-Object)
            runtime_integrity = [pscustomobject][ordered]@{
                mount_count = 0
                writable_layer_changes = $writableLayerChanges
                command = $containerCommand
                entrypoint = $containerEntrypoint
                working_directory = $containerWorkingDirectory
                user = $containerUser
                critical_environment = $criticalEnvironment
            }
        }
        database_runtime = [pscustomobject][ordered]@{
            container_id = [string]$databaseInspect.Id
            database_user = [string]$databaseUser
            database_url_host = [string]$databaseUri.Host
            database_url_port = [int]$databaseUri.Port
            binding_mode = $databaseBindingMode
            shared_networks = $sharedNetworkNames
            resolved_database_ips = $resolvedDatabaseIps
            current_database = [string]$database.current_database
        }
        migrations = @($database.migrations)
        migration_sources = $expectedMigrations
        installation = [pscustomobject][ordered]@{
            id = [string]$database.installation.id
            created_at = $databaseInstallationCreatedAt
            row_count = [int]$database.installation.row_count
            runtime_observation_count = [int]$database.installation.runtime_observation_count
            core_runtime = [pscustomobject][ordered]@{
                provenance = [string]$databaseRuntime.provenance
                observed_version = [string]$databaseRuntime.observed_version
                finding_code = [string]$databaseRuntime.finding_code
                observed_at = $databaseObservedAt
                package_contract = $apiPackage
            }
        }
        built_in_seed = $seedContract
        catalog = [pscustomobject][ordered]@{
            definition_count = [int]$database.catalog.definition_count
            source_count = [int]$database.catalog.source_count
            projection_count = [int]$database.catalog.projection_count
            current_count = [int]$database.catalog.current_count
            navigation_contribution_count = [int]$database.catalog.navigation_contribution_count
            navigation_policy_count = [int]$database.catalog.navigation_policy_count
            policy_entries = @($database.catalog.policy_entries)
            release_table_absent = [bool]$database.catalog.release_table_absent
            instance_table_absent = [bool]$database.catalog.instance_table_absent
            entries = $catalogEntries
            immutable_fixture_entries = $expectedCatalogEntries
        }
        data = [pscustomobject][ordered]@{
            state = "fresh"
            classification_rule = "Sprint closeout uses the single migration-1 baseline and a freshly seeded database; no upgrade state is accepted."
        }
        service = [pscustomobject][ordered]@{
            health_status = 200
            health_body = "ok"
            inventory_schema_version = [int]$inventory.schema_version
            installation_id = [string]$inventory.installation.id
            catalog_entries = @($inventory.entries).Count
        }
    }
}

function Assert-Sprint6ADeploymentEvidenceDocument {
    param(
        [Parameter(Mandatory)][object]$Evidence,
        [Parameter(Mandatory)][ValidateSet("fresh")][string]$ExpectedDataState,
        [Parameter(Mandatory)][string]$BaseUrl
    )

    $databaseRuntime = $Evidence.snapshot.database_runtime
    $databaseUserProperty = if ($null -eq $databaseRuntime) {
        $null
    } else {
        $databaseRuntime.PSObject.Properties['database_user']
    }
    if ($null -eq $databaseUserProperty -or
        $databaseUserProperty.Value -isnot [string] -or
        [string]$databaseUserProperty.Value -notmatch '^[A-Za-z_][A-Za-z0-9_-]*$') {
        throw "The deployment evidence database_runtime.database_user must be a non-secret PostgreSQL identifier string."
    }

    if ([int]$Evidence.schema_version -ne 1 -or
        [string]$Evidence.evidence_kind -cne $script:Sprint6ADeploymentEvidenceKind -or
        [string]$Evidence.snapshot.base_url -cne $BaseUrl.TrimEnd("/") -or
        [string]$Evidence.snapshot.data.state -cne $ExpectedDataState -or
        [string]$Evidence.snapshot.built_in_seed.version -cne $script:Sprint6ABuiltInSeedVersion -or
        [string]$Evidence.snapshot.built_in_seed.canonical_sha256 -cne $script:Sprint6ABuiltInSeedSha256 -or
        [string]$Evidence.snapshot.release_image.image_id -notmatch "^sha256:[0-9a-f]{64}$" -or
        [string]$Evidence.snapshot.source.commit -notmatch "^[0-9a-f]{40}$" -or
        [string]$Evidence.snapshot.source.tree -notmatch "^[0-9a-f]{40}$") {
        throw "The deployment evidence document is not the required Sprint 6A schema-v1 '$ExpectedDataState' record for '$BaseUrl'."
    }
}

function Assert-Sprint6ADeploymentEvidence {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidencePath,
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][ValidateSet("fresh")][string]$ExpectedDataState,
        [string]$AdminEmail = "admin@tessara.local",
        [string]$AdminPassword = "tessara-dev-admin"
    )

    $fullPath = Resolve-Sprint6ARepositoryPath -RepositoryRoot $RepositoryRoot -Path $EvidencePath
    $digestPath = "$fullPath.sha256"
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $digestPath -PathType Leaf)) {
        throw "Deployment evidence and its SHA-256 sidecar are required: '$fullPath' and '$digestPath'."
    }
    $expectedDigest = (Get-Content -LiteralPath $digestPath -Raw).Trim()
    $actualDigest = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expectedDigest -notmatch "^[0-9a-f]{64}$" -or $expectedDigest -cne $actualDigest) {
        throw "Deployment evidence SHA-256 verification failed for '$fullPath'."
    }
    $evidence = ConvertFrom-Sprint6ADeploymentEvidenceJson `
        -Json (Get-Content -LiteralPath $fullPath -Raw)
    Assert-Sprint6ADeploymentEvidenceDocument `
        -Evidence $evidence `
        -ExpectedDataState $ExpectedDataState `
        -BaseUrl $BaseUrl
    $liveSnapshot = Get-Sprint6ADeploymentSnapshot `
        -RepositoryRoot $RepositoryRoot `
        -BaseUrl $BaseUrl `
        -AdminEmail $AdminEmail `
        -AdminPassword $AdminPassword `
        -ApiContainerId ([string]$evidence.snapshot.release_image.api_container_id) `
        -GatewayContainerId ([string]$evidence.snapshot.release_image.published_base_url_container_id) `
        -DatabaseContainerId ([string]$evidence.snapshot.database_runtime.container_id)
    $retainedJson = $evidence.snapshot | ConvertTo-Json -Depth 30 -Compress
    $liveJson = $liveSnapshot | ConvertTo-Json -Depth 30 -Compress
    if ($retainedJson -cne $liveJson) {
        throw "The retained deployment evidence no longer matches the exact live source/image/service/database deployment. Capture new evidence; do not reuse this record."
    }
    Write-Host "Verified Sprint 6A $ExpectedDataState deployment evidence: $fullPath" -ForegroundColor Green
    $evidence
}

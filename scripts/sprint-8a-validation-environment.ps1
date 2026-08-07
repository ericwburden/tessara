Set-StrictMode -Version Latest

function ConvertTo-Sprint8ADateTimeOffset {
    param(
        [Parameter(Mandatory)]$Value,
        [string]$Label = "timestamp"
    )

    if ($Value -is [DateTimeOffset]) { return [DateTimeOffset]$Value }
    if ($Value -is [DateTime]) {
        if ($Value.Kind -eq [DateTimeKind]::Unspecified) {
            throw "Sprint 8A $Label has no UTC offset."
        }
        return [DateTimeOffset]$Value
    }
    $text = [string]$Value
    if ($text -notmatch '(?:Z|[+-]\d{2}:\d{2})$') {
        throw "Sprint 8A $Label has no UTC offset."
    }
    $parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse(
        $text,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind,
        [ref]$parsed
    )) {
        throw "Sprint 8A $Label is not an offset-qualified timestamp."
    }
    $parsed
}

function Get-Sprint8AStringSha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        ([BitConverter]::ToString(
            $algorithm.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))
        ) -replace "-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Get-Sprint8AFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Cannot hash missing Sprint 8A input '$fullPath'."
    }
    (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Resolve-Sprint8AEvidenceReference {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Path,
        [switch]$AllowLegacyAbsolute
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    )
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot))
    }
    $candidate = if ([IO.Path]::IsPathRooted($Path)) {
        if (-not $AllowLegacyAbsolute) {
            throw "Sprint 8A evidence references must be repository-relative."
        }
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $Path))
    }
    $relativeToRepository = [IO.Path]::GetRelativePath($repository, $candidate)
    $relativeToEvidence = [IO.Path]::GetRelativePath($evidence, $candidate)
    foreach ($relative in @($relativeToRepository, $relativeToEvidence)) {
        if ([IO.Path]::IsPathRooted($relative) -or
            $relative -eq ".." -or
            $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
            $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
            throw "Sprint 8A evidence reference escapes its repository evidence root."
        }
    }
    [pscustomobject][ordered]@{
        full_path = $candidate
        path = $relativeToRepository.Replace("\", "/")
    }
}

function Test-Sprint8AEvidenceReferenceResolution {
    $repository = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
    $evidenceRoot = Join-Path $repository "artifacts/sprint-8a-closeout"
    $relative = "artifacts/sprint-8a-closeout/attempts/example.json"
    $absolute = Join-Path $repository $relative
    $canonical = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repository `
        -EvidenceRoot $evidenceRoot `
        -Path $relative
    $legacy = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repository `
        -EvidenceRoot $evidenceRoot `
        -Path $absolute `
        -AllowLegacyAbsolute
    if ([string]$canonical.path -cne $relative -or
        [string]$legacy.path -cne $relative -or
        [string]$canonical.full_path -cne [string]$legacy.full_path) {
        throw "Sprint 8A canonical/legacy evidence reference resolution self-test failed."
    }
    foreach ($unsafe in @(
        "../outside.json",
        "artifacts/outside.json",
        (Join-Path (Split-Path -Parent $repository) "outside.json")
    )) {
        try {
            Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidenceRoot `
                -Path $unsafe `
                -AllowLegacyAbsolute | Out-Null
            throw "Evidence reference self-test accepted unsafe path '$unsafe'."
        } catch {
            if ($_.Exception.Message -ceq "Evidence reference self-test accepted unsafe path '$unsafe'.") { throw }
        }
    }
    "Sprint 8A evidence-reference containment self-test passed."
}

function Get-Sprint8AOptionalObjectPropertyValue {
    param(
        [AllowNull()]$InputObject,
        [Parameter(Mandatory)][string]$Name
    )

    if ($null -eq $InputObject) {
        return $null
    }
    $property = $InputObject.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    $property.Value
}

function Get-Sprint8AComposeServiceProjection {
    param([Parameter(Mandatory)]$Services)

    $ports = @($Services.PSObject.Properties | ForEach-Object {
        $serviceName = [string]$_.Name
        $servicePorts = @(Get-Sprint8AOptionalObjectPropertyValue -InputObject $_.Value -Name "ports")
        @($servicePorts | ForEach-Object {
            $published = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_ -Name "published"
            if ($null -ne $published) {
                $target = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_ -Name "target"
                if ($null -eq $target) {
                    throw "Normalized Compose service '$serviceName' publishes a port without a target."
                }
                $hostIp = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_ -Name "host_ip"
                $protocol = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_ -Name "protocol"
                [pscustomobject][ordered]@{
                    service = $serviceName
                    host_ip = if ($null -eq $hostIp) { "" } else { [string]$hostIp }
                    published = [int]$published
                    target = [int]$target
                    protocol = if ($null -eq $protocol) { "tcp" } else { [string]$protocol }
                }
            }
        })
    } | Sort-Object service, published)
    $databaseBindings = @($Services.PSObject.Properties | ForEach-Object {
        $serviceName = [string]$_.Name
        $environment = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_.Value -Name "environment"
        if ($null -ne $environment) {
            @($environment.PSObject.Properties | Where-Object { $_.Name -match 'DATABASE_URL$' } | ForEach-Object {
                $uri = [Uri][string]$_.Value
                [pscustomobject][ordered]@{
                    service = $serviceName
                    variable = [string]$_.Name
                    host = $uri.Host
                    port = if ($uri.IsDefaultPort) { 5432 } else { $uri.Port }
                    database = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart('/'))
                    role = [Uri]::UnescapeDataString(($uri.UserInfo.Split(':', 2))[0])
                }
            })
        }
    } | Sort-Object database, role, service)

    [pscustomobject][ordered]@{
        ports = $ports
        database_bindings = $databaseBindings
    }
}

function Test-Sprint8AComposeServiceProjection {
    $services = [pscustomobject][ordered]@{
        components = [pscustomobject][ordered]@{
            environment = [pscustomobject][ordered]@{
                DATABASE_URL = "postgres://components_runtime@postgres:5432/tessara_module_components"
            }
        }
        gateway = [pscustomobject][ordered]@{
            ports = @([pscustomobject][ordered]@{
                host_ip = "127.0.0.1"
                published = 8088
                target = 8080
                protocol = "tcp"
            })
        }
        worker = [pscustomobject][ordered]@{}
    }
    $projection = Get-Sprint8AComposeServiceProjection -Services $services
    if (@($projection.ports).Count -ne 1 -or
        [string]$projection.ports[0].service -cne "gateway" -or
        [int]$projection.ports[0].published -ne 8088 -or
        @($projection.database_bindings).Count -ne 1 -or
        [string]$projection.database_bindings[0].service -cne "components" -or
        [string]$projection.database_bindings[0].database -cne "tessara_module_components") {
        throw "Sprint 8A optional Compose service projection self-test failed."
    }
    "Sprint 8A optional Compose service projection self-test passed."
}

function Get-Sprint8AResultClassifications {
    param([AllowEmptyCollection()][object[]]$Results = @())

    @($Results | ForEach-Object {
        $classification = Get-Sprint8AOptionalObjectPropertyValue -InputObject $_ -Name "classification"
        if (-not [string]::IsNullOrWhiteSpace([string]$classification)) {
            [string]$classification
        }
    } | Sort-Object -Unique)
}

function Test-Sprint8AResultClassificationProjection {
    $empty = @(Get-Sprint8AResultClassifications -Results @())
    $single = @(Get-Sprint8AResultClassifications -Results @(
        [pscustomobject]@{ classification = "product" }
    ))
    $duplicates = @(Get-Sprint8AResultClassifications -Results @(
        [pscustomobject]@{ classification = "harness" }
        [pscustomobject]@{ classification = "harness" }
    ))
    $multiple = @(Get-Sprint8AResultClassifications -Results @(
        $null
        [pscustomobject]@{}
        [pscustomobject]@{ classification = $null }
        [pscustomobject]@{ classification = "product" }
        [pscustomobject]@{ classification = "environment" }
    ))
    if ($empty.Count -ne 0 -or
        ($single -join ",") -cne "product" -or
        ($duplicates -join ",") -cne "harness" -or
        ($multiple -join ",") -cne "environment,product") {
        throw "Sprint 8A result-classification projection self-test failed."
    }
    "Sprint 8A result-classification projection self-test passed."
}

function Get-Sprint8APathSetDigest {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string[]]$PathSpecs
    )

    $files = @(& git -C $RepositoryRoot ls-files -- @PathSpecs | Where-Object {
        -not [string]::IsNullOrWhiteSpace([string]$_)
    } | Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0 -or $files.Count -eq 0) {
        throw "Could not resolve the tracked Sprint 8A input set: $($PathSpecs -join ', ')."
    }
    $members = @($files | ForEach-Object {
        $relative = ([string]$_).Replace("\", "/")
        $fullPath = Join-Path $RepositoryRoot $relative
        [ordered]@{
            path = $relative
            sha256 = Get-Sprint8AFileSha256 -Path $fullPath
        }
    })
    $canonical = $members | ConvertTo-Json -Depth 5 -Compress
    [ordered]@{
        sha256 = Get-Sprint8AStringSha256 -Text $canonical
        members = $members
    }
}

function Get-Sprint8ASourceIdentity {
    param([Parameter(Mandatory)][string]$RepositoryRoot)

    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $commit = (& git -C $root rev-parse HEAD).Trim()
    $tree = (& git -C $root rev-parse "HEAD^{tree}").Trim()
    $branch = (& git -C $root branch --show-current).Trim()
    $status = @(& git -C $root status --porcelain=v1)
    if ($LASTEXITCODE -ne 0 -or $commit -notmatch '^[0-9a-f]{40}$' -or $tree -notmatch '^[0-9a-f]{40}$') {
        throw "Could not resolve the Sprint 8A Git source identity."
    }

    $acceptance = Get-Sprint8APathSetDigest -RepositoryRoot $root -PathSpecs @(
        ".codex/skills/tessara-sprint-validation/**",
        ".codex/skills/tessara-validation-preflight/**",
        ".codex/skills/tessara-sit/**",
        ".codex/skills/tessara-uat/**",
        "docs/sprints/sprint-8a-*.md",
        "docs/sprints/sprint-8a-uat/*.md",
        "docs/sprints/sprint-8a-uat/scenario-contract.json",
        "end2end/**",
        "crates/**/tests/**",
        "crates/**/fixtures/**",
        "scripts/*sprint-8a*.ps1",
        "scripts/fixtures/**",
        "scripts/bootstrap-sprint-7a-composition.ps1",
        "scripts/capture-sprint-6a-deployment-evidence.ps1",
        "scripts/check-web-crate-boundaries.ps1",
        "scripts/prepare-sprint-7a-uat-fixtures.ps1",
        "scripts/run-analytics-authorization-conformance.ps1",
        "scripts/run-module-sdk-conformance.ps1",
        "scripts/smoke.ps1",
        "scripts/sprint-6a-deployment-evidence-common.ps1",
        "scripts/sprint-7a-acceptance-contract.ps1",
        "scripts/test-sprint-validation-harvest.ps1",
        "scripts/uat-sprint.ps1",
        "scripts/validate-analytics-nondisclosure.ps1",
        "scripts/validate-e2e.ps1",
        "scripts/validate-resource-reference-nondisclosure.ps1",
        "scripts/validate.ps1",
        "scripts/verify-markdown-links.ps1",
        "scripts/verify-module-sdk-*.ps1",
        "scripts/verify-sprint-6e-boundaries.ps1"
    )
    $deployment = Get-Sprint8APathSetDigest -RepositoryRoot $root -PathSpecs @(
        ".dockerignore",
        "Dockerfile*",
        "Cargo.lock",
        "Cargo.toml",
        "package-lock.json",
        "package.json",
        "tailwind.config.js",
        "deploy/sprint-7a/**",
        "deploy/sprint-8a/**",
        "crates/*/Cargo.toml",
        "crates/*/manifest.json",
        "crates/*/migrations/*.sql"
    )

    [pscustomobject][ordered]@{
        commit = $commit
        tree = $tree
        dirty = $status.Count -ne 0
        branch = $branch
        acceptance_inventory_sha256 = [string]$acceptance.sha256
        deployment_inputs_sha256 = [string]$deployment.sha256
    }
}

function Assert-Sprint8ASourceIdentityObject {
    param(
        [Parameter(Mandatory)]$Source,
        [switch]$RequireClean
    )

    $expectedProperties = @(
        "commit", "tree", "dirty", "branch",
        "acceptance_inventory_sha256", "deployment_inputs_sha256"
    )
    $actualProperties = @($Source.PSObject.Properties.Name | Sort-Object)
    if (($actualProperties | ConvertTo-Json -Compress) -cne
        (@($expectedProperties | Sort-Object) | ConvertTo-Json -Compress) -or
        $Source.commit -isnot [string] -or [string]$Source.commit -notmatch '^[0-9a-f]{40}$' -or
        $Source.tree -isnot [string] -or [string]$Source.tree -notmatch '^[0-9a-f]{40}$' -or
        $Source.dirty -isnot [bool] -or
        $Source.branch -isnot [string] -or [string]::IsNullOrWhiteSpace([string]$Source.branch) -or
        $Source.acceptance_inventory_sha256 -isnot [string] -or
        [string]$Source.acceptance_inventory_sha256 -notmatch '^[0-9a-f]{64}$' -or
        $Source.deployment_inputs_sha256 -isnot [string] -or
        [string]$Source.deployment_inputs_sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Sprint 8A mutable source identity has a malformed shape or value."
    }
    if ($RequireClean -and [bool]$Source.dirty) {
        throw "Sprint 8A mutable source identity is dirty."
    }
    $Source
}

function Get-Sprint8AToolVersion {
    param(
        [Parameter(Mandatory)][string]$Command,
        [string[]]$Arguments = @()
    )

    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
        throw "Required Sprint 8A tool '$Command' is unavailable."
    }
    $output = @(& $Command @Arguments 2>&1 | ForEach-Object { [string]$_ })
    if ($LASTEXITCODE -ne 0 -or $output.Count -eq 0) {
        throw "Could not obtain the version for required Sprint 8A tool '$Command'."
    }
    ($output -join "`n").Trim()
}

function ConvertTo-Sprint8ADatabaseBinding {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Value
    )

    try {
        $uri = [Uri]$Value
    } catch {
        throw "$Name is not a valid PostgreSQL URL."
    }
    $database = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart("/"))
    $userInfo = $uri.UserInfo.Split(':', 2)
    $user = if ($userInfo.Count -gt 0) { [Uri]::UnescapeDataString($userInfo[0]) } else { "" }
    $password = if ($userInfo.Count -eq 2) { [Uri]::UnescapeDataString($userInfo[1]) } else { "" }
    $hostName = $uri.DnsSafeHost.ToLowerInvariant()
    $port = if ($uri.IsDefaultPort) { 5432 } else { $uri.Port }
    if ($uri.Scheme -notin @("postgres", "postgresql") -or
        $hostName -notin @("127.0.0.1", "localhost", "::1") -or
        [string]::IsNullOrWhiteSpace($database) -or
        $database -notmatch '(^|[_-])test([_-]|$)' -or
        [string]::IsNullOrWhiteSpace($user)) {
        throw "$Name must identify one explicit loopback, token-bounded disposable test database and role."
    }
    # All allowed names resolve the same local trust boundary. Canonicalize
    # them before pairwise comparison so aliases cannot disguise one physical
    # database as two readiness bindings.
    $canonicalServer = "loopback:$port"
    [pscustomobject][ordered]@{
        name = $Name
        value_sha256 = Get-Sprint8AStringSha256 -Text $Value
        host = $hostName
        port = $port
        database = $database
        role = $user
        password = $password
        canonical_server = $canonicalServer
        identity = "$canonicalServer/$($database.ToLowerInvariant())"
    }
}

function Invoke-Sprint8ADatabaseProbe {
    param(
        [Parameter(Mandatory)]$Binding,
        [string]$PostgresContainerId,
        [switch]$RequireFreshDatabase
    )

    $sql = @"
BEGIN;
CREATE TEMP TABLE tessara_readiness_probe(value integer);
INSERT INTO tessara_readiness_probe VALUES (8);
SELECT current_database() || '|' || current_user || '|' ||
       (SELECT value::text FROM tessara_readiness_probe) || '|' ||
       (SELECT oid::text FROM pg_database WHERE datname = current_database()) || '|' ||
       (SELECT count(*)::text FROM pg_catalog.pg_tables
          WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
            AND schemaname NOT LIKE 'pg_toast%'
            AND schemaname NOT LIKE 'pg_temp_%');
ROLLBACK;
"@
    $output = @()
    $client = $null
    if (-not [string]::IsNullOrWhiteSpace($PostgresContainerId)) {
        $inspectOutput = @(& docker inspect $PostgresContainerId 2>&1)
        if ($LASTEXITCODE -ne 0 -or $inspectOutput.Count -eq 0) {
            throw "TEST_POSTGRES_CLIENT_CONTAINER_ID does not identify an inspectable container."
        }
        $inspect = @($inspectOutput | ConvertFrom-Json)[0]
        if (-not [bool]$inspect.State.Running) {
            throw "The approved PostgreSQL probe container is not running."
        }
        $published = @(& docker port ([string]$inspect.Id) 5432/tcp 2>&1 | ForEach-Object { [string]$_ })
        if ($LASTEXITCODE -ne 0 -or -not ($published -match ":$($Binding.port)$")) {
            throw "The approved PostgreSQL probe container is not published on $($Binding.host):$($Binding.port)."
        }
        $arguments = @("exec")
        if (-not [string]::IsNullOrEmpty([string]$Binding.password)) {
            $arguments += @("-e", "PGPASSWORD=$($Binding.password)")
        }
        $arguments += @(
            [string]$inspect.Id, "psql", "-X", "-v", "ON_ERROR_STOP=1", "-At",
            "-h", "127.0.0.1", "-p", "5432", "-U", [string]$Binding.role,
            "-d", [string]$Binding.database, "-c", $sql
        )
        $output = @(& docker @arguments 2>&1 | ForEach-Object { [string]$_ })
        $client = [ordered]@{
            kind = "docker_postgres"
            container_id = [string]$inspect.Id
            container_name = ([string]$inspect.Name).TrimStart('/')
            configured_image = [string]$inspect.Config.Image
            image_id = [string]$inspect.Image
            published_binding = ($published -join ",")
        }
    } else {
        if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
            throw "Authenticated database readiness requires psql or TEST_POSTGRES_CLIENT_CONTAINER_ID."
        }
        $priorPassword = [Environment]::GetEnvironmentVariable("PGPASSWORD", "Process")
        try {
            [Environment]::SetEnvironmentVariable("PGPASSWORD", [string]$Binding.password, "Process")
            $output = @(& psql -X -v ON_ERROR_STOP=1 -At -h ([string]$Binding.host) -p ([string]$Binding.port) -U ([string]$Binding.role) -d ([string]$Binding.database) -c $sql 2>&1 | ForEach-Object { [string]$_ })
        } finally {
            [Environment]::SetEnvironmentVariable("PGPASSWORD", $priorPassword, "Process")
        }
        $client = [ordered]@{ kind = "local_psql"; version = Get-Sprint8AToolVersion -Command "psql" -Arguments @("--version") }
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Authenticated database probe failed for $($Binding.name)."
    }
    $probeLines = @($output | Where-Object { $_ -match '^.+\|.+\|8\|[0-9]+\|[0-9]+$' })
    if ($probeLines.Count -ne 1) {
        throw "Authenticated database probe returned the wrong database or role for $($Binding.name)."
    }
    $parts = @($probeLines[0].Split('|'))
    if ($parts.Count -ne 5 -or $parts[0] -cne [string]$Binding.database -or
        $parts[1] -cne [string]$Binding.role -or $parts[2] -cne "8" -or
        [long]$parts[3] -lt 1 -or [long]$parts[4] -lt 0) {
        throw "Authenticated database probe returned a malformed generation identity for $($Binding.name)."
    }
    $userTableCount = [long]$parts[4]
    if ($RequireFreshDatabase -and $userTableCount -ne 0) {
        throw "Fresh validation database prerequisite rejected $($Binding.name): its database generation already contains $userTableCount user table(s). Recreate all six disposable databases before Readiness."
    }
    [ordered]@{
        database = [string]$Binding.database
        role = [string]$Binding.role
        database_oid = [long]$parts[3]
        transaction_round_trip = $true
        client = $client
    }
}

function Get-Sprint8ADeploymentEnvironmentProbe {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$ProbeDatabases,
        [switch]$RequireFreshDatabases
    )

    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $composePath = Join-Path $root "deploy/sprint-8a/compose.yaml"
    $composeJson = @(& docker compose -f $composePath --profile reference config --format json 2>&1)
    if ($LASTEXITCODE -ne 0 -or $composeJson.Count -eq 0) {
        throw "Sprint 8A Compose configuration cannot be normalized."
    }
    $composeText = ($composeJson -join "`n")
    $compose = $composeText | ConvertFrom-Json
    if ([string]$compose.name -cne "tessara-sprint-8a") {
        throw "Sprint 8A environment resolves unexpected Compose project '$($compose.name)'."
    }

    $databaseNames = @(
        "TEST_API_DATABASE_URL",
        "TEST_API_FRESH_DATABASE_URL",
        "TEST_REFERENCE_MODULE_DATABASE_URL",
        "TEST_COMPONENT_MODULE_DATABASE_URL",
        "TEST_API_ENROLLMENT_DATABASE_URL",
        "TEST_INSTALLATION_CONTROL_DATABASE_URL"
    )
    $bindings = @()
    foreach ($name in $databaseNames) {
        $value = [Environment]::GetEnvironmentVariable($name)
        if ([string]::IsNullOrWhiteSpace($value)) {
            throw "Sprint 8A environment requires $name for complete non-skipping validation."
        }
        $bindings += ConvertTo-Sprint8ADatabaseBinding -Name $name -Value $value
    }
    if (@($bindings.identity | Sort-Object -Unique).Count -ne $databaseNames.Count) {
        throw "Sprint 8A validation database identities must be pairwise distinct."
    }
    $resetAuthorized = [Environment]::GetEnvironmentVariable("SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET") -ceq "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"
    if (-not $resetAuthorized) {
        throw "Sprint 8A environment requires the exact destructive fresh-reset acknowledgement."
    }
    $containerId = [Environment]::GetEnvironmentVariable("TEST_POSTGRES_CLIENT_CONTAINER_ID")
    if ($RequireFreshDatabases -and -not $ProbeDatabases) {
        throw "Fresh validation database authentication requires live database probes."
    }
    $freshnessEvidence = [Collections.Generic.List[object]]::new()
    $databaseContracts = @($bindings | ForEach-Object {
        $binding = $_
        $probe = if ($ProbeDatabases) {
            Invoke-Sprint8ADatabaseProbe `
                -Binding $binding `
                -PostgresContainerId $containerId `
                -RequireFreshDatabase:$RequireFreshDatabases
        } else { $null }
        if ($RequireFreshDatabases) {
            $freshnessEvidence.Add([pscustomobject][ordered]@{
                variable = [string]$binding.name
                database = [string]$binding.database
                database_oid = [long]$probe.database_oid
                user_table_count = 0
                fresh = $true
            })
        }
        [ordered]@{
            variable = [string]$binding.name
            value_sha256 = [string]$binding.value_sha256
            host = [string]$binding.host
            port = [int]$binding.port
            database = [string]$binding.database
            role = [string]$binding.role
            canonical_server = [string]$binding.canonical_server
            authenticated_probe = $probe
        }
    })

    $serviceTopology = @($compose.services.PSObject.Properties | Sort-Object Name | ForEach-Object {
        [ordered]@{
            service = [string]$_.Name
            image = [string]$_.Value.image
        }
    })
    $probe = [ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.deployment-environment-probe"
        operating_system = [ordered]@{
            description = [Runtime.InteropServices.RuntimeInformation]::OSDescription
            architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
        }
        databases = $databaseContracts
        reset_authorization_present = $resetAuthorized
        compose = [ordered]@{
            project = [string]$compose.name
            profile = "reference"
            normalized_config_sha256 = Get-Sprint8AStringSha256 -Text $composeText
            topology = $serviceTopology
        }
        endpoints = [ordered]@{
            gateway = "http://127.0.0.1:8088"
            materialization_control = "http://127.0.0.1:18088"
            supervisor = "http://127.0.0.1:8098"
        }
        fixture_identities = @(
            "administrator", "scoped_operator", "mixed_scope_operator",
            "no_analytics_actor", "undeclared_service", "wrong_service_instance"
        )
        evidence = [ordered]@{
            root = $EvidenceRoot.Replace("\", "/")
            output_mode = "attempt_scoped_append_only"
        }
    }
    $canonical = $probe | ConvertTo-Json -Depth 30 -Compress
    [ordered]@{
        contract = $probe
        fingerprint = Get-Sprint8AStringSha256 -Text $canonical
        freshness = if ($RequireFreshDatabases) { [ordered]@{
            verified = $true
            databases = @($freshnessEvidence)
        } } else { $null }
    }
}

function Get-Sprint8AToolchainEnvironmentContract {
    param([Parameter(Mandatory)][string]$RepositoryRoot)

    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $pwshVersion = "$($PSVersionTable.PSEdition) $($PSVersionTable.PSVersion)"
    $windowsPowerShell = $null
    if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT -and
        (Get-Command powershell.exe -ErrorAction SilentlyContinue)) {
        $windowsPowerShell = (@(& powershell.exe -NoProfile -NonInteractive -Command '$PSVersionTable.PSVersion.ToString()' 2>&1) -join "`n").Trim()
        if ($LASTEXITCODE -ne 0) { throw "Windows PowerShell runtime probe failed." }
    }
    [ordered]@{
        powershell = $pwshVersion
        windows_powershell = $windowsPowerShell
        rustc = Get-Sprint8AToolVersion -Command "rustc" -Arguments @("--version")
        cargo = Get-Sprint8AToolVersion -Command "cargo" -Arguments @("--version")
        node = Get-Sprint8AToolVersion -Command "node" -Arguments @("--version")
        npm = Get-Sprint8AToolVersion -Command "npm" -Arguments @("--version")
        playwright = Get-Sprint8AToolVersion -Command "npm" -Arguments @("--prefix", (Join-Path $root "end2end"), "exec", "playwright", "--", "--version")
        docker = Get-Sprint8AToolVersion -Command "docker" -Arguments @("--version")
        docker_compose = Get-Sprint8AToolVersion -Command "docker" -Arguments @("compose", "version")
        dotnet_runtime = [Runtime.InteropServices.RuntimeInformation]::FrameworkDescription
    }
}

function Get-Sprint8AEnvironmentContract {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$ProbeDatabases,
        [switch]$RequireFreshDatabases,
        [AllowNull()]$DeploymentProbe
    )

    $probeResult = if ($null -eq $DeploymentProbe) {
        Get-Sprint8ADeploymentEnvironmentProbe `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -ProbeDatabases:$ProbeDatabases `
            -RequireFreshDatabases:$RequireFreshDatabases
    } else {
        $DeploymentProbe
    }
    if ($null -eq $probeResult.contract -or
        [int]$probeResult.contract.schema_version -ne 1 -or
        [string]$probeResult.contract.contract -cne "tessara.sprint-8a.deployment-environment-probe" -or
        [string]$probeResult.fingerprint -notmatch '^[0-9a-f]{64}$' -or
        (Get-Sprint8AStringSha256 -Text ($probeResult.contract | ConvertTo-Json -Depth 30 -Compress)) -cne
            [string]$probeResult.fingerprint) {
        throw "Sprint 8A environment finalization received a malformed deployment/database probe."
    }
    $deployment = $probeResult.contract
    $contract = [ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.validation-environment"
        operating_system = $deployment.operating_system
        toolchain = Get-Sprint8AToolchainEnvironmentContract -RepositoryRoot $RepositoryRoot
        databases = $deployment.databases
        reset_authorization_present = [bool]$deployment.reset_authorization_present
        compose = $deployment.compose
        endpoints = $deployment.endpoints
        fixture_identities = @($deployment.fixture_identities)
        evidence = $deployment.evidence
    }
    $canonical = $contract | ConvertTo-Json -Depth 30 -Compress
    [ordered]@{
        contract = $contract
        fingerprint = Get-Sprint8AStringSha256 -Text $canonical
        deployment_probe_fingerprint = [string]$probeResult.fingerprint
    }
}

function Compare-Sprint8AEnvironmentContracts {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual
    )

    foreach ($candidate in @($Expected, $Actual)) {
        if ([string]$candidate.fingerprint -notmatch '^[0-9a-f]{64}$' -or $null -eq $candidate.contract) {
            throw "Sprint 8A environment comparison requires complete fingerprinted contracts."
        }
    }
    $expectedNames = if ($Expected.contract -is [Collections.IDictionary]) {
        @($Expected.contract.Keys | ForEach-Object { [string]$_ })
    } else {
        @($Expected.contract.PSObject.Properties.Name)
    }
    $actualNames = if ($Actual.contract -is [Collections.IDictionary]) {
        @($Actual.contract.Keys | ForEach-Object { [string]$_ })
    } else {
        @($Actual.contract.PSObject.Properties.Name)
    }
    $sectionNames = @(@($expectedNames) + @($actualNames) | Sort-Object -Unique)
    $sections = @($sectionNames | ForEach-Object {
        $name = [string]$_
        $expectedValue = if ($Expected.contract -is [Collections.IDictionary]) {
            $Expected.contract[$name]
        } else {
            Get-Sprint8AOptionalObjectPropertyValue -InputObject $Expected.contract -Name $name
        }
        $actualValue = if ($Actual.contract -is [Collections.IDictionary]) {
            $Actual.contract[$name]
        } else {
            Get-Sprint8AOptionalObjectPropertyValue -InputObject $Actual.contract -Name $name
        }
        $expectedSha = Get-Sprint8AStringSha256 -Text ($expectedValue | ConvertTo-Json -Depth 30 -Compress)
        $actualSha = Get-Sprint8AStringSha256 -Text ($actualValue | ConvertTo-Json -Depth 30 -Compress)
        [pscustomobject][ordered]@{
            name = $name
            expected_sha256 = $expectedSha
            actual_sha256 = $actualSha
            changed = $expectedSha -cne $actualSha
        }
    })
    [pscustomobject][ordered]@{
        expected_fingerprint = [string]$Expected.fingerprint
        actual_fingerprint = [string]$Actual.fingerprint
        matched = [string]$Expected.fingerprint -ceq [string]$Actual.fingerprint -and
            @($sections | Where-Object changed -EQ $true).Count -eq 0
        changed_sections = @($sections | Where-Object changed -EQ $true | ForEach-Object { [string]$_.name })
        section_digests = $sections
        expected_contract = $Expected.contract
        actual_contract = $Actual.contract
    }
}

function Test-Sprint8AEnvironmentContractComparison {
    $expectedContract = [ordered]@{
        schema_version = 1
        contract = "fixture"
        compose = [ordered]@{ normalized_config_sha256 = "a" * 64 }
        databases = @([ordered]@{ database = "fixture" })
    }
    $expected = [ordered]@{
        contract = $expectedContract
        fingerprint = Get-Sprint8AStringSha256 -Text ($expectedContract | ConvertTo-Json -Depth 30 -Compress)
    }
    $equal = Compare-Sprint8AEnvironmentContracts -Expected $expected -Actual $expected
    $changedContract = [ordered]@{
        schema_version = 1
        contract = "fixture"
        compose = [ordered]@{ normalized_config_sha256 = "b" * 64 }
        databases = @([ordered]@{ database = "fixture" })
    }
    $changed = [ordered]@{
        contract = $changedContract
        fingerprint = Get-Sprint8AStringSha256 -Text ($changedContract | ConvertTo-Json -Depth 30 -Compress)
    }
    $different = Compare-Sprint8AEnvironmentContracts -Expected $expected -Actual $changed
    if (-not [bool]$equal.matched -or @($equal.changed_sections).Count -ne 0 -or
        [bool]$different.matched -or (@($different.changed_sections) -join ",") -cne "compose") {
        throw "Sprint 8A environment contract comparison self-test failed."
    }
    "Sprint 8A environment contract comparison self-test passed."
}

function Assert-Sprint8AReceiptSidecar {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    if ($null -ne (Get-Command Repair-Sprint7AEvidencePublication -ErrorAction SilentlyContinue)) {
        Repair-Sprint7AEvidencePublication -Path $fullPath
    }
    $sidecarPath = "$fullPath.sha256"
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        throw "Receipt or SHA-256 sidecar is missing for '$fullPath'."
    }
    $actual = Get-Sprint8AFileSha256 -Path $fullPath
    $expected = (Get-Content -LiteralPath $sidecarPath -Raw).Trim()
    if ($expected -notmatch '^[0-9a-f]{64}$' -or $actual -cne $expected) {
        throw "Receipt SHA-256 sidecar does not match '$fullPath'."
    }
    $actual
}

function Get-Sprint8AEvidenceRelativePath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$RelativePath
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    )
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot))
    }
    $evidenceRelative = [IO.Path]::GetRelativePath($repository, $evidence)
    if ([IO.Path]::IsPathRooted($evidenceRelative) -or
        $evidenceRelative -eq ".." -or
        $evidenceRelative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
        $evidenceRelative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "Sprint 8A evidence root must remain inside its repository."
    }
    $joined = if ($evidenceRelative -eq ".") {
        $RelativePath
    } else {
        "$($evidenceRelative.Replace('\', '/').TrimEnd('/'))/$($RelativePath.Replace('\', '/').TrimStart('/'))"
    }
    $joined.Replace("\", "/")
}

function Assert-Sprint8ACurrentReadinessReference {
    param(
        [Parameter(Mandatory)]$StateReadiness,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$RequirePassed
    )

    if (($StateReadiness.attempt -isnot [int] -and $StateReadiness.attempt -isnot [long]) -or
        [int]$StateReadiness.attempt -lt 1 -or
        @("passed", "failed") -cnotcontains [string]$StateReadiness.state -or
        [string]$StateReadiness.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Current Readiness state reference is malformed or nonterminal."
    }
    if ($RequirePassed -and [string]$StateReadiness.state -cne "passed") {
        throw "Current Readiness state reference is not passing."
    }
    $attempt = [int]$StateReadiness.attempt
    $immutablePath = Get-Sprint8AEvidenceRelativePath `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -RelativePath "attempts/readiness-$attempt.json"
    $currentPath = if ([string]$StateReadiness.state -ceq "passed") {
        Get-Sprint8AEvidenceRelativePath `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -RelativePath "validation-readiness-result.json"
    } else {
        $immutablePath
    }
    if ([string]$StateReadiness.receipt -cne $currentPath) {
        throw "Current Readiness state does not name its exact canonical receipt path."
    }

    $currentReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $currentPath
    $currentSha = Assert-Sprint8AReceiptSidecar -Path ([string]$currentReference.full_path)
    if ($currentSha -cne [string]$StateReadiness.sha256) {
        throw "Current Readiness state digest differs from its canonical receipt."
    }
    $currentDocument = Get-Content -LiteralPath ([string]$currentReference.full_path) -Raw | ConvertFrom-Json
    if (($currentDocument.schema_version -isnot [int] -and $currentDocument.schema_version -isnot [long]) -or
        @(2, 3) -notcontains [int]$currentDocument.schema_version -or
        [string]$currentDocument.sprint -cne "sprint-8a" -or
        [string]$currentDocument.phase -cne "validation-readiness" -or
        ($currentDocument.attempt -isnot [int] -and $currentDocument.attempt -isnot [long]) -or
        [int]$currentDocument.attempt -ne $attempt -or
        [string]$currentDocument.state -cne [string]$StateReadiness.state -or
        $currentDocument.PSObject.Properties.Name -notcontains "prerequisite_receipts" -or
        $currentDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
        $currentDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
        $currentDocument.authoritative -isnot [bool] -or
        $currentDocument.authoritative -ne $false -or
        $currentDocument.assertions_started -isnot [bool] -or
        ([string]$currentDocument.state -ceq "passed" -and $currentDocument.assertions_started -ne $true)) {
        throw "Current Readiness state does not authenticate its exact attempt document."
    }

    $immutableReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $immutablePath
    $immutableSha = Assert-Sprint8AReceiptSidecar -Path ([string]$immutableReference.full_path)
    $immutableDocument = Get-Content -LiteralPath ([string]$immutableReference.full_path) -Raw | ConvertFrom-Json
    if ($immutableSha -cne $currentSha -or
        ($immutableDocument.schema_version -isnot [int] -and $immutableDocument.schema_version -isnot [long]) -or
        @(2, 3) -notcontains [int]$immutableDocument.schema_version -or
        [string]$immutableDocument.sprint -cne "sprint-8a" -or
        [string]$immutableDocument.phase -cne "validation-readiness" -or
        ($immutableDocument.attempt -isnot [int] -and $immutableDocument.attempt -isnot [long]) -or
        [int]$immutableDocument.attempt -ne $attempt -or
        [string]$immutableDocument.state -cne [string]$StateReadiness.state -or
        $immutableDocument.PSObject.Properties.Name -notcontains "prerequisite_receipts" -or
        $immutableDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
        $immutableDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
        $immutableDocument.authoritative -isnot [bool] -or
        $immutableDocument.authoritative -ne $false -or
        $immutableDocument.assertions_started -isnot [bool] -or
        ([string]$immutableDocument.state -ceq "passed" -and $immutableDocument.assertions_started -ne $true) -or
        (($immutableDocument | ConvertTo-Json -Depth 100 -Compress) -cne
            ($currentDocument | ConvertTo-Json -Depth 100 -Compress))) {
        throw "Current Readiness canonical alias does not have one exact immutable attempt counterpart."
    }

    [pscustomobject][ordered]@{
        attempt = $attempt
        state = [string]$StateReadiness.state
        current = [pscustomobject][ordered]@{
            path = [string]$currentReference.path
            full_path = [string]$currentReference.full_path
            sha256 = $currentSha
            document = $currentDocument
        }
        immutable = [pscustomobject][ordered]@{
            path = [string]$immutableReference.path
            full_path = [string]$immutableReference.full_path
            sha256 = $immutableSha
            document = $immutableDocument
        }
    }
}

function Assert-Sprint8AReadinessCorrectionLineagePresence {
    param(
        [AllowNull()]$StateLineage,
        [Parameter(Mandatory)]$ReadinessDocument
    )

    $documentAuthorization = Get-Sprint8AOptionalObjectPropertyValue `
        -InputObject $ReadinessDocument `
        -Name "predecessor_correction_authorization"
    $documentConsumption = Get-Sprint8AOptionalObjectPropertyValue `
        -InputObject $ReadinessDocument `
        -Name "correction_consumption_receipt"
    $documentLineage = Get-Sprint8AOptionalObjectPropertyValue `
        -InputObject $ReadinessDocument `
        -Name "correction_lineage"
    if ($null -eq $StateLineage) {
        if ($null -ne $documentAuthorization -or
            $null -ne $documentConsumption -or
            $null -ne $documentLineage) {
            throw "Readiness names a correction transition or preserved lineage without canonical validation-state lineage."
        }
        return
    }
    if ($null -ne $documentLineage -and
        (($documentLineage | ConvertTo-Json -Depth 100 -Compress) -cne
            ($StateLineage | ConvertTo-Json -Depth 100 -Compress))) {
        throw "Readiness and validation-state retain different correction lineage values."
    }
}

function Assert-Sprint8AReadinessSupersessionChain {
    param(
        [Parameter(Mandatory)]$DescendantReference,
        [Parameter(Mandatory)]$AncestorReference,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    function Resolve-ReadinessTerminalReference {
        param($Reference, [string]$Label)

        if ($null -eq $Reference -or
            [string]::IsNullOrWhiteSpace([string]$Reference.path) -or
            [string]$Reference.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "$Label lacks one exact immutable Readiness path and SHA-256."
        }
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$Reference.path)
        $sha = Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
        if ($sha -cne [string]$Reference.sha256) {
            throw "$Label digest differs from its retained immutable receipt."
        }
        $document = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
        if (($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
            @(2, 3) -notcontains [int]$document.schema_version -or
            [string]$document.sprint -cne "sprint-8a" -or
            [string]$document.phase -cne "validation-readiness" -or
            ($document.attempt -isnot [int] -and $document.attempt -isnot [long]) -or
            [int]$document.attempt -lt 1 -or
            @("passed", "failed") -cnotcontains [string]$document.state -or
            $document.authoritative -isnot [bool] -or
            $document.authoritative -ne $false -or
            $document.assertions_started -isnot [bool] -or
            ([string]$document.state -ceq "passed" -and $document.assertions_started -ne $true)) {
            throw "$Label is not one supported terminal Readiness receipt."
        }
        $expectedPath = Get-Sprint8AEvidenceRelativePath `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -RelativePath "attempts/readiness-$([int]$document.attempt).json"
        if ([string]$resolved.path -cne $expectedPath) {
            throw "$Label substitutes a noncanonical immutable Readiness path."
        }
        if ($Reference.PSObject.Properties.Name -contains "attempt" -and
            [int]$Reference.attempt -ne [int]$document.attempt) {
            throw "$Label attempt differs from its retained immutable receipt."
        }
        if ($Reference.PSObject.Properties.Name -contains "state" -and
            [string]$Reference.state -cne [string]$document.state) {
            throw "$Label state differs from its retained immutable receipt."
        }
        [pscustomobject][ordered]@{
            path = [string]$resolved.path
            full_path = [string]$resolved.full_path
            sha256 = $sha
            document = $document
        }
    }

    function Assert-ExactReadinessPrerequisite {
        param($Reference, $Expected, [string]$Label)

        $names = @($Reference.PSObject.Properties.Name | Sort-Object)
        if ($null -eq $Reference -or
            ($names | ConvertTo-Json -Compress) -cne (@("path", "sha256") | ConvertTo-Json -Compress) -or
            [string]$Reference.path -cne [string]$Expected.path -or
            [string]$Reference.sha256 -cne [string]$Expected.sha256) {
            throw "$Label does not name its exact immutable predecessor Readiness."
        }
    }

    $descendant = Resolve-ReadinessTerminalReference $DescendantReference "Readiness supersession descendant"
    $ancestor = Resolve-ReadinessTerminalReference $AncestorReference "Readiness supersession ancestor"
    $references = [Collections.Generic.List[object]]::new()
    $references.Add($descendant)
    if ([string]$descendant.path -ceq [string]$ancestor.path -and
        [string]$descendant.sha256 -ceq [string]$ancestor.sha256) {
        return [pscustomobject][ordered]@{
            kind = "direct_correction_consumption"
            depth = 0
            descendant = $descendant
            ancestor = $ancestor
            references = @($references)
        }
    }
    if ([string]$ancestor.document.state -cne "passed" -or
        [int]$descendant.document.attempt -le [int]$ancestor.document.attempt) {
        throw "Clean Readiness supersession requires a later descendant of one passing ancestor."
    }

    $seen = @{}
    $cursor = $descendant
    $depth = 0
    while ([string]$cursor.path -cne [string]$ancestor.path -or
        [string]$cursor.sha256 -cne [string]$ancestor.sha256) {
        if ($seen.ContainsKey([string]$cursor.path)) {
            throw "Clean Readiness supersession contains a loop."
        }
        $seen[[string]$cursor.path] = $true
        $depth++
        if ($depth -gt 9999) { throw "Clean Readiness supersession exceeds the supported attempt range." }

        $cursorDocument = $cursor.document
        $prerequisites = @($cursorDocument.prerequisite_receipts)
        if ([int]$cursorDocument.schema_version -ne 3 -or
            $prerequisites.Count -ne 1 -or
            $cursorDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
            $cursorDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
            $null -ne (Get-Sprint8AOptionalObjectPropertyValue -InputObject $cursorDocument -Name "predecessor_correction_authorization") -or
            $null -ne (Get-Sprint8AOptionalObjectPropertyValue -InputObject $cursorDocument -Name "correction_consumption_receipt")) {
            throw "Clean Readiness supersession terminal must be schema 3, name one predecessor, and carry no direct correction consumption."
        }

        $predecessor = Resolve-ReadinessTerminalReference $prerequisites[0] "Readiness supersession predecessor"
        Assert-ExactReadinessPrerequisite $prerequisites[0] $predecessor "Readiness supersession terminal prerequisite"
        if ([string]$predecessor.document.state -cne "passed" -or
            [int]$cursorDocument.attempt -ne ([int]$predecessor.document.attempt + 1)) {
            throw "Clean Readiness supersession must be contiguous and may supersede only a passing predecessor."
        }
        $predecessorIsAncestor = [string]$predecessor.path -ceq [string]$ancestor.path -and
            [string]$predecessor.sha256 -ceq [string]$ancestor.sha256
        if (-not $predecessorIsAncestor) {
            $cursorLineage = Get-Sprint8AOptionalObjectPropertyValue -InputObject $cursorDocument -Name "correction_lineage"
            $predecessorLineage = Get-Sprint8AOptionalObjectPropertyValue -InputObject $predecessor.document -Name "correction_lineage"
            if (($cursorLineage | ConvertTo-Json -Depth 100 -Compress) -cne
                ($predecessorLineage | ConvertTo-Json -Depth 100 -Compress)) {
                throw "Clean Readiness supersession changed correction lineage between sequential clean edges."
            }
        }

        $startPath = Get-Sprint8AEvidenceRelativePath `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -RelativePath "attempts/readiness-$([int]$cursorDocument.attempt)-start.json"
        $startResolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path $startPath
        $startSha = Assert-Sprint8AReceiptSidecar -Path ([string]$startResolved.full_path)
        $startDocument = Get-Content -LiteralPath ([string]$startResolved.full_path) -Raw | ConvertFrom-Json
        $startPrerequisites = @($startDocument.prerequisite_receipts)
        $startLineage = Get-Sprint8AOptionalObjectPropertyValue -InputObject $startDocument -Name "correction_lineage"
        $terminalLineage = Get-Sprint8AOptionalObjectPropertyValue -InputObject $cursorDocument -Name "correction_lineage"
        if (($startDocument.schema_version -isnot [int] -and $startDocument.schema_version -isnot [long]) -or
            [int]$startDocument.schema_version -ne 2 -or
            [string]$startDocument.sprint -cne "sprint-8a" -or
            [string]$startDocument.phase -cne "validation-readiness" -or
            ($startDocument.attempt -isnot [int] -and $startDocument.attempt -isnot [long]) -or
            [int]$startDocument.attempt -ne [int]$cursorDocument.attempt -or
            [string]$startDocument.state -cne "preparing" -or
            $startDocument.assertions_started -isnot [bool] -or
            $startDocument.assertions_started -ne $false -or
            $startDocument.authoritative -isnot [bool] -or
            $startDocument.authoritative -ne $false -or
            @($startDocument.checks).Count -ne 0 -or
            $startPrerequisites.Count -ne 1 -or
            $startDocument.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
            $startDocument.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
            $null -ne (Get-Sprint8AOptionalObjectPropertyValue -InputObject $startDocument -Name "predecessor_correction_authorization") -or
            $null -ne (Get-Sprint8AOptionalObjectPropertyValue -InputObject $startDocument -Name "correction_consumption_receipt") -or
            (($startLineage | ConvertTo-Json -Depth 100 -Compress) -cne
                ($terminalLineage | ConvertTo-Json -Depth 100 -Compress))) {
            throw "Clean Readiness supersession immutable start does not preserve the exact pre-assertion schema."
        }
        Assert-ExactReadinessPrerequisite $startPrerequisites[0] $predecessor "Readiness supersession start prerequisite"
        $references.Add([pscustomobject][ordered]@{
            path = [string]$startResolved.path
            full_path = [string]$startResolved.full_path
            sha256 = $startSha
            document = $startDocument
        })
        $references.Add($predecessor)
        $cursor = $predecessor
        if ([int]$cursor.document.attempt -lt [int]$ancestor.document.attempt) {
            throw "Clean Readiness supersession does not terminate at its authenticated ancestor."
        }
    }

    [pscustomobject][ordered]@{
        kind = "clean_pre_rehearsal_supersession"
        depth = $depth
        descendant = $descendant
        ancestor = $ancestor
        references = @($references)
    }
}

function ConvertTo-Sprint8ACorrectionLineage {
    param(
        [Parameter(Mandatory)]$LegacyTransition,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    if ($null -eq $LegacyTransition.predecessor_rehearsal -or
        $null -eq $LegacyTransition.authorization -or
        [int]$LegacyTransition.predecessor_rehearsal.attempt -ne 30) {
        throw "Only the exact historical candidate-rehearsal correction transition can be normalized."
    }
    $expectedPredecessorPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/candidate-rehearsal-30-attempt.json"
    $expectedHarvestPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/candidate-rehearsal-30-harvest.json"
    $expectedBatchPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/candidate-rehearsal-30-defect-batch.json"
    $expectedAuthorizationPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/candidate-rehearsal-30-correction-authorization.json"
    $authorizationReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$LegacyTransition.authorization.path) `
        -AllowLegacyAbsolute
    $authorizationSha = Assert-Sprint8AReceiptSidecar -Path ([string]$authorizationReference.full_path)
    if ([string]$authorizationReference.path -cne $expectedAuthorizationPath -or
        $authorizationSha -cne [string]$LegacyTransition.authorization.sha256) {
        throw "Historical correction authorization differs from validation-state."
    }
    $authorization = Get-Content -LiteralPath ([string]$authorizationReference.full_path) -Raw | ConvertFrom-Json
    if ([string]$authorization.phase -cne "candidate-rehearsal-correction-authorization" -or
        [string]$authorization.state -cne "authorized" -or
        [int]$authorization.attempt -ne [int]$LegacyTransition.predecessor_rehearsal.attempt) {
        throw "Historical correction authorization is not the exact R30-style transition."
    }
    $predecessorReference = Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$authorization.predecessor_attempt_receipt.path) -AllowLegacyAbsolute
    $harvestReference = Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$authorization.harvest_receipt.path) -AllowLegacyAbsolute
    $batchReference = Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$authorization.defect_batch.path) -AllowLegacyAbsolute
    if ([string]$predecessorReference.path -cne $expectedPredecessorPath -or
        [string]$harvestReference.path -cne $expectedHarvestPath -or
        [string]$batchReference.path -cne $expectedBatchPath -or
        (Assert-Sprint8AReceiptSidecar -Path ([string]$predecessorReference.full_path)) -cne [string]$authorization.predecessor_attempt_receipt.sha256 -or
        (Assert-Sprint8AReceiptSidecar -Path ([string]$harvestReference.full_path)) -cne [string]$authorization.harvest_receipt.sha256 -or
        (Assert-Sprint8AReceiptSidecar -Path ([string]$batchReference.full_path)) -cne [string]$authorization.defect_batch.sha256) {
        throw "Historical correction authorization prerequisite digests are invalid."
    }

    $consumed = $null
    if ($null -ne $LegacyTransition.consumed_by_readiness) {
        if ([int]$LegacyTransition.consumed_by_readiness.attempt -ne 38 -or
            [string]$LegacyTransition.consumed_by_readiness.state -cne "failed") {
            throw "Only the exact historical R30-to-R38 failed transition can be normalized."
        }
        $expectedConsumptionPath = "$expectedAuthorizationPath.consumption.json"
        $expectedStartPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/readiness-38-start.json"
        $expectedTerminalPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/readiness-38.json"
        $consumptionReference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$LegacyTransition.consumed_by_readiness.consumption_receipt.path) `
            -AllowLegacyAbsolute
        $consumptionSha = Assert-Sprint8AReceiptSidecar -Path ([string]$consumptionReference.full_path)
        if ([string]$consumptionReference.path -cne $expectedConsumptionPath -or
            $consumptionSha -cne [string]$LegacyTransition.consumed_by_readiness.consumption_receipt.sha256) {
            throw "Historical correction consumption differs from validation-state."
        }
        $consumption = Get-Content -LiteralPath ([string]$consumptionReference.full_path) -Raw | ConvertFrom-Json
        $startReference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$consumption.successor_readiness.start_receipt) `
            -AllowLegacyAbsolute
        $startSha = Assert-Sprint8AReceiptSidecar -Path ([string]$startReference.full_path)
        if ([string]$startReference.path -cne $expectedStartPath -or
            $startSha -cne [string]$consumption.successor_readiness.start_receipt_sha256) {
            throw "Historical correction consumption does not bind its immutable Readiness start."
        }
        $terminalReference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$LegacyTransition.consumed_by_readiness.receipt) `
            -AllowLegacyAbsolute
        $terminalSha = Assert-Sprint8AReceiptSidecar -Path ([string]$terminalReference.full_path)
        if ([string]$terminalReference.path -cne $expectedTerminalPath -or
            $terminalSha -cne [string]$LegacyTransition.consumed_by_readiness.receipt_sha256) {
            throw "Historical correction transition does not bind its terminal Readiness receipt."
        }
        $terminalDocument = Get-Content -LiteralPath ([string]$terminalReference.full_path) -Raw | ConvertFrom-Json
        if ([string]$consumption.phase -cne "candidate-rehearsal-correction-consumption" -or
            [string]$consumption.state -cne "consumed" -or
            [string]$consumption.authorization.path -cne $expectedAuthorizationPath -or
            [string]$consumption.authorization.sha256 -cne $authorizationSha -or
            [int]$consumption.predecessor_rehearsal.attempt -ne 30 -or
            [string]$consumption.predecessor_rehearsal.receipt -cne $expectedPredecessorPath -or
            [string]$consumption.predecessor_rehearsal.sha256 -cne [string]$authorization.predecessor_attempt_receipt.sha256 -or
            [string]$terminalDocument.phase -cne "validation-readiness" -or
            [int]$terminalDocument.attempt -ne 38 -or
            [string]$terminalDocument.state -cne "failed") {
            throw "Historical correction transition documents are not the exact retained R30-to-R38 evidence."
        }
        $consumed = [ordered]@{
            attempt = [int]$LegacyTransition.consumed_by_readiness.attempt
            start_receipt = [ordered]@{ path = [string]$startReference.path; sha256 = $startSha }
            consumption_receipt = [ordered]@{ path = [string]$consumptionReference.path; sha256 = $consumptionSha }
            receipt = [ordered]@{
                path = [string]$terminalReference.path
                sha256 = $terminalSha
                state = [string]$LegacyTransition.consumed_by_readiness.state
            }
        }
    }

    [ordered]@{
        schema_version = 1
        links = @([ordered]@{
            ordinal = 1
            predecessor = [ordered]@{
                phase = "candidate-rehearsal"
                attempt = [int]$LegacyTransition.predecessor_rehearsal.attempt
                receipt = [ordered]@{
                    path = [string]$predecessorReference.path
                    sha256 = [string]$authorization.predecessor_attempt_receipt.sha256
                }
                harvest = [ordered]@{
                    path = [string]$harvestReference.path
                    sha256 = [string]$authorization.harvest_receipt.sha256
                }
                defect_batch = [ordered]@{
                    path = [string]$batchReference.path
                    sha256 = [string]$authorization.defect_batch.sha256
                }
            }
            authorization = [ordered]@{ path = [string]$authorizationReference.path; sha256 = $authorizationSha }
            consumed_by_readiness = $consumed
        })
    }
}

function Add-Sprint8ACorrectionLineageLink {
    param(
        [AllowNull()]$Lineage,
        [Parameter(Mandatory)]$Predecessor,
        [Parameter(Mandatory)]$Authorization,
        [AllowNull()]$ConsumedByReadiness
    )

    $links = [Collections.Generic.List[object]]::new()
    if ($null -ne $Lineage) {
        if (($Lineage.schema_version -isnot [int] -and $Lineage.schema_version -isnot [long]) -or
            [int]$Lineage.schema_version -ne 1) {
            throw "Correction lineage append requires exact schema version 1."
        }
        foreach ($link in @($Lineage.links)) { $links.Add($link) }
    }
    $links.Add([ordered]@{
        ordinal = $links.Count + 1
        predecessor = $Predecessor
        authorization = $Authorization
        consumed_by_readiness = $ConsumedByReadiness
    })
    [ordered]@{ schema_version = 1; links = @($links) }
}

function Assert-Sprint8ACorrectionIdentityContinuity {
    param([Parameter(Mandatory)][object[]]$Documents)

    if ($Documents.Count -ne 4) {
        throw "Correction identity continuity requires predecessor, harvest, batch, and authorization documents."
    }
    $predecessor = $Documents[0]
    if ($predecessor.PSObject.Properties.Name -notcontains "mutable_source_identity" -or
        $predecessor.PSObject.Properties.Name -notcontains "environment_fingerprint") {
        throw "Correction lineage predecessor omits source/environment identity."
    }
    $predecessorSourceJson = $predecessor.mutable_source_identity | ConvertTo-Json -Depth 30 -Compress
    $predecessorEnvironment = [string]$predecessor.environment_fingerprint
    foreach ($document in $Documents) {
        if ($document.PSObject.Properties.Name -notcontains "mutable_source_identity" -or
            $document.PSObject.Properties.Name -notcontains "environment_fingerprint" -or
            [string]$document.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
            [string]$document.environment_fingerprint -cne $predecessorEnvironment -or
            (($document.mutable_source_identity | ConvertTo-Json -Depth 30 -Compress) -cne $predecessorSourceJson)) {
            throw "Correction lineage breaks predecessor source/environment identity continuity."
        }
    }
}

function Assert-Sprint8ACorrectionLineageTopology {
    param(
        [Parameter(Mandatory)][object[]]$Links,
        [Parameter(Mandatory)][object[]]$PredecessorDocuments,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    if ($Links.Count -ne $PredecessorDocuments.Count) {
        throw "Correction-lineage topology requires one authenticated predecessor document per link."
    }
    for ($index = 0; $index -lt $Links.Count; $index++) {
        $link = $Links[$index]
        if ([int]$link.ordinal -ne ($index + 1)) {
            throw "Correction lineage ordinals must be contiguous and one-based."
        }
        $phase = [string]$link.predecessor.phase
        if (@("candidate-rehearsal", "validation-readiness") -cnotcontains $phase) {
            throw "Correction lineage link $($index + 1) has an invalid predecessor phase."
        }
        $predecessorDocument = $PredecessorDocuments[$index]
        if ($index -eq 0) {
            $rootLineage = Get-Sprint8AOptionalObjectPropertyValue -InputObject $predecessorDocument -Name "correction_lineage"
            if ($phase -ceq "candidate-rehearsal") {
                if (@($predecessorDocument.prerequisite_receipts).Count -ne 1 -or $null -ne $rootLineage) {
                    throw "Root correction lineage candidate must retain one Readiness prerequisite and no prior lineage prefix."
                }
            } else {
                $rootAuthorization = Get-Sprint8AOptionalObjectPropertyValue -InputObject $predecessorDocument -Name "predecessor_correction_authorization"
                $rootConsumption = Get-Sprint8AOptionalObjectPropertyValue -InputObject $predecessorDocument -Name "correction_consumption_receipt"
                if ($null -ne $rootAuthorization -or $null -ne $rootConsumption -or $null -ne $rootLineage) {
                    throw "Root failed-Readiness correction link cannot be a truncated consumed suffix."
                }
            }
            continue
        }

        $priorConsumed = $Links[$index - 1].consumed_by_readiness
        if ($null -eq $priorConsumed) {
            throw "Correction lineage contains a gap after an unconsumed authorization."
        }
        if ($phase -ceq "validation-readiness") {
            $readinessBinding = Assert-Sprint8AReadinessSupersessionChain `
                -DescendantReference $link.predecessor.receipt `
                -AncestorReference $priorConsumed.receipt `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot
            $predecessorAuthorization = Get-Sprint8AOptionalObjectPropertyValue -InputObject $predecessorDocument -Name "predecessor_correction_authorization"
            $predecessorConsumption = Get-Sprint8AOptionalObjectPropertyValue -InputObject $predecessorDocument -Name "correction_consumption_receipt"
            $directBindingInvalid = [string]$readinessBinding.kind -ceq "direct_correction_consumption" -and (
                $null -eq $predecessorAuthorization -or $null -eq $predecessorConsumption -or
                [string]$predecessorAuthorization.path -cne [string]$Links[$index - 1].authorization.path -or
                [string]$predecessorAuthorization.sha256 -cne [string]$Links[$index - 1].authorization.sha256 -or
                [string]$predecessorConsumption.path -cne [string]$priorConsumed.consumption_receipt.path -or
                [string]$predecessorConsumption.sha256 -cne [string]$priorConsumed.consumption_receipt.sha256)
            $supersessionBindingInvalid = [string]$readinessBinding.kind -ceq "clean_pre_rehearsal_supersession" -and (
                $null -ne $predecessorAuthorization -or $null -ne $predecessorConsumption -or
                $predecessorDocument.PSObject.Properties.Name -notcontains "correction_lineage" -or
                (($predecessorDocument.correction_lineage | ConvertTo-Json -Depth 100 -Compress) -cne
                    ([ordered]@{ schema_version = 1; links = @($Links[0..($index - 1)]) } |
                        ConvertTo-Json -Depth 100 -Compress)))
            if ([string]$predecessorDocument.state -cne "failed" -or
                ($directBindingInvalid -or $supersessionBindingInvalid)) {
                throw "Failed Readiness suffix does not authenticate the complete preceding correction link."
            }
            continue
        }

        $prerequisites = @($predecessorDocument.prerequisite_receipts)
        if ($prerequisites.Count -ne 1) {
            throw "Correction lineage candidate rehearsal does not name the exact preceding passing Readiness as its sole prerequisite."
        }
        [void](Assert-Sprint8AReadinessSupersessionChain `
            -DescendantReference $prerequisites[0] `
            -AncestorReference $priorConsumed.receipt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot)
        if ($predecessorDocument.PSObject.Properties.Name -notcontains "correction_lineage" -or
            $null -eq $predecessorDocument.correction_lineage -or
            [int]$predecessorDocument.correction_lineage.schema_version -ne 1) {
            throw "Correction lineage candidate rehearsal does not retain its authenticated prior lineage."
        }
        $expectedPrefix = [ordered]@{
            schema_version = 1
            links = @($Links[0..($index - 1)])
        }
        if (($predecessorDocument.correction_lineage | ConvertTo-Json -Depth 100 -Compress) -cne
            ($expectedPrefix | ConvertTo-Json -Depth 100 -Compress)) {
            throw "Correction lineage candidate rehearsal does not authenticate the complete preceding lineage prefix."
        }
    }
}

function Assert-Sprint8ACorrectionLineage {
    param(
        [AllowNull()]$Lineage,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [AllowNull()]$ExpectedCurrentReadiness,
        [switch]$RequireConsumedTip
    )

    if ($null -eq $Lineage) {
        if ($RequireConsumedTip -or $null -ne $ExpectedCurrentReadiness) {
            throw "Validation-state omits the required correction lineage."
        }
        return [pscustomobject][ordered]@{
            links = @()
            tip = $null
            current_readiness_binding = $null
            references = @()
        }
    }
    if (($Lineage.schema_version -isnot [int] -and $Lineage.schema_version -isnot [long]) -or
        [int]$Lineage.schema_version -ne 1) {
        throw "Correction lineage schema must be exact version 1."
    }
    $links = @($Lineage.links)
    if ($links.Count -eq 0) { throw "Correction lineage must contain at least one link." }

    function Resolve-LineageReference {
        param($Reference, [string]$Label)
        if ($null -eq $Reference -or [string]::IsNullOrWhiteSpace([string]$Reference.path) -or
            [string]$Reference.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "$Label lacks an exact path and SHA-256."
        }
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$Reference.path) `
            -AllowLegacyAbsolute
        $sha = Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
        if ($sha -cne [string]$Reference.sha256) { throw "$Label digest differs from its retained receipt." }
        [pscustomobject][ordered]@{
            path = [string]$resolved.path
            full_path = [string]$resolved.full_path
            sha256 = $sha
            document = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
        }
    }

    $references = [Collections.Generic.List[object]]::new()
    $predecessorDocuments = [Collections.Generic.List[object]]::new()
    $seenAuthorizationPaths = @{}
    $seenConsumptionPaths = @{}
    for ($index = 0; $index -lt $links.Count; $index++) {
        $link = $links[$index]
        if ([int]$link.ordinal -ne ($index + 1)) { throw "Correction lineage ordinals must be contiguous and one-based." }
        $phase = [string]$link.predecessor.phase
        if (@("candidate-rehearsal", "validation-readiness") -cnotcontains $phase) {
            throw "Correction lineage link $($index + 1) has an invalid predecessor phase."
        }
        $attempt = [int]$link.predecessor.attempt
        if ($attempt -lt 1) { throw "Correction lineage predecessor attempt must be positive." }
        $receiptRef = Resolve-LineageReference $link.predecessor.receipt "Correction predecessor receipt"
        $harvestRef = Resolve-LineageReference $link.predecessor.harvest "Correction predecessor harvest"
        $batchRef = Resolve-LineageReference $link.predecessor.defect_batch "Correction predecessor defect batch"
        $authorizationRef = Resolve-LineageReference $link.authorization "Correction authorization"
        $predecessorStem = if ($phase -ceq "candidate-rehearsal") {
            "candidate-rehearsal-$attempt"
        } else {
            "readiness-$attempt"
        }
        $expectedReceiptPath = Get-Sprint8AEvidenceRelativePath `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -RelativePath $(if ($phase -ceq "candidate-rehearsal") { "attempts/$predecessorStem-attempt.json" } else { "attempts/$predecessorStem.json" })
        $expectedHarvestPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/$predecessorStem-harvest.json"
        $expectedBatchPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/$predecessorStem-defect-batch.json"
        $expectedAuthorizationPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/$predecessorStem-correction-authorization.json"
        if ([string]$receiptRef.path -cne $expectedReceiptPath -or
            [string]$harvestRef.path -cne $expectedHarvestPath -or
            [string]$batchRef.path -cne $expectedBatchPath -or
            [string]$authorizationRef.path -cne $expectedAuthorizationPath) {
            throw "Correction lineage link $($index + 1) substitutes a noncanonical predecessor artifact path."
        }
        foreach ($reference in @($receiptRef, $harvestRef, $batchRef, $authorizationRef)) { $references.Add($reference) }
        $predecessorDocuments.Add($receiptRef.document)
        if ($seenAuthorizationPaths.ContainsKey([string]$authorizationRef.path)) {
            throw "Correction lineage reuses an authorization path."
        }
        $seenAuthorizationPaths[[string]$authorizationRef.path] = $true

        $expectedHarvestPhase = "$phase-harvest"
        $expectedBatchPhase = "$phase-defect-batch"
        $expectedAuthorizationPhase = "$phase-correction-authorization"
        $predecessorReadinessSchemaInvalid = $phase -ceq "validation-readiness" -and (
            ($receiptRef.document.schema_version -isnot [int] -and $receiptRef.document.schema_version -isnot [long]) -or
            @(2, 3) -notcontains [int]$receiptRef.document.schema_version -or
            $receiptRef.document.assertions_started -isnot [bool] -or
            ([string]$receiptRef.document.state -ceq "passed" -and $receiptRef.document.assertions_started -ne $true))
        $authorizedAttemptPath = (Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$authorizationRef.document.predecessor_attempt_receipt.path) -AllowLegacyAbsolute).path
        $authorizedHarvestPath = (Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$authorizationRef.document.harvest_receipt.path) -AllowLegacyAbsolute).path
        $authorizedBatchPath = (Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$authorizationRef.document.defect_batch.path) -AllowLegacyAbsolute).path
        if ($predecessorReadinessSchemaInvalid -or
            [string]$receiptRef.document.sprint -cne "sprint-8a" -or
            $receiptRef.document.authoritative -isnot [bool] -or
            $receiptRef.document.authoritative -ne $false -or
            [string]$receiptRef.document.phase -cne $phase -or
            [string]$receiptRef.document.state -cne "failed" -or
            [int]$receiptRef.document.attempt -ne $attempt -or
            [string]$harvestRef.document.phase -cne $expectedHarvestPhase -or
            [string]$harvestRef.document.state -cne "harvest_complete" -or
            [int]$harvestRef.document.attempt -ne $attempt -or
            [string]$batchRef.document.phase -cne $expectedBatchPhase -or
            [string]$batchRef.document.state -cne "open" -or
            [int]$batchRef.document.attempt -ne $attempt -or
            [string]$authorizationRef.document.phase -cne $expectedAuthorizationPhase -or
            [string]$authorizationRef.document.state -cne "authorized" -or
            [int]$authorizationRef.document.attempt -ne $attempt -or
            [int]$authorizationRef.document.allowed_successor_count -ne 1 -or
            [string]$authorizationRef.document.allowed_successor_phase -cne "validation-readiness" -or
            [string]$authorizedAttemptPath -cne [string]$receiptRef.path -or
            [string]$authorizedHarvestPath -cne [string]$harvestRef.path -or
            [string]$authorizedBatchPath -cne [string]$batchRef.path -or
            [string]$authorizationRef.document.predecessor_attempt_receipt.sha256 -cne [string]$receiptRef.sha256 -or
            [string]$authorizationRef.document.harvest_receipt.sha256 -cne [string]$harvestRef.sha256 -or
            [string]$authorizationRef.document.defect_batch.sha256 -cne [string]$batchRef.sha256) {
            throw "Correction lineage link $($index + 1) does not authenticate its exact predecessor, harvest, batch, and authorization."
        }
        Assert-Sprint8ACorrectionIdentityContinuity `
            -Documents @($receiptRef.document, $harvestRef.document, $batchRef.document, $authorizationRef.document)
        if ($index -eq 0 -and $phase -ceq "candidate-rehearsal") {
            $rootPrerequisite = @($receiptRef.document.prerequisite_receipts)[0]
            if ($attempt -eq 30) {
                $legacyAliasPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "validation-readiness-result.json"
                $legacyImmutablePath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/readiness-37.json"
                if ([string]$rootPrerequisite.path -cne $legacyAliasPath -or
                    [string]$rootPrerequisite.sha256 -notmatch '^[0-9a-f]{64}$') {
                    throw "Historical R30 root does not retain its exact legacy passing-Readiness prerequisite."
                }
                $legacyImmutableRef = Resolve-LineageReference ([pscustomobject]@{
                    path = $legacyImmutablePath
                    sha256 = [string]$rootPrerequisite.sha256
                }) "Historical R30 immutable Readiness prerequisite"
                if ([string]$legacyImmutableRef.document.phase -cne "validation-readiness" -or
                    [string]$legacyImmutableRef.document.sprint -cne "sprint-8a" -or
                    [int]$legacyImmutableRef.document.attempt -ne 37 -or
                    [string]$legacyImmutableRef.document.state -cne "passed" -or
                    $legacyImmutableRef.document.authoritative -isnot [bool] -or
                    $legacyImmutableRef.document.authoritative -ne $false -or
                    $legacyImmutableRef.document.assertions_started -isnot [bool] -or
                    $legacyImmutableRef.document.assertions_started -ne $true) {
                    throw "Historical R30 prerequisite does not resolve to exact immutable passing Readiness 37."
                }
                $references.Add($legacyImmutableRef)
            } else {
                $rootPrerequisiteRef = Resolve-LineageReference $rootPrerequisite "Root candidate Readiness prerequisite"
                $rootPrerequisiteAttempt = [int]$rootPrerequisiteRef.document.attempt
                $expectedRootPrerequisitePath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/readiness-$rootPrerequisiteAttempt.json"
                if ([string]$rootPrerequisiteRef.path -cne $expectedRootPrerequisitePath -or
                    ($rootPrerequisiteRef.document.schema_version -isnot [int] -and $rootPrerequisiteRef.document.schema_version -isnot [long]) -or
                    @(2, 3) -notcontains [int]$rootPrerequisiteRef.document.schema_version -or
                    [string]$rootPrerequisiteRef.document.sprint -cne "sprint-8a" -or
                    [string]$rootPrerequisiteRef.document.phase -cne "validation-readiness" -or
                    [string]$rootPrerequisiteRef.document.state -cne "passed" -or
                    $rootPrerequisiteRef.document.authoritative -isnot [bool] -or
                    $rootPrerequisiteRef.document.authoritative -ne $false -or
                    $rootPrerequisiteRef.document.assertions_started -isnot [bool] -or
                    $rootPrerequisiteRef.document.assertions_started -ne $true -or
                    $rootPrerequisiteAttempt -lt 1) {
                    throw "Root candidate does not authenticate one immutable passing-Readiness prerequisite."
                }
                $references.Add($rootPrerequisiteRef)
            }
        }
        if ($phase -ceq "validation-readiness" -and
            (($authorizationRef.document.schema_version -isnot [int] -and $authorizationRef.document.schema_version -isnot [long]) -or
                [int]$authorizationRef.document.schema_version -ne 2 -or
                [int]$authorizationRef.document.allowed_successor_attempt -ne ($attempt + 1))) {
            throw "Failed-Readiness authorization must bind exactly its next Readiness attempt."
        }
        $consumed = $link.consumed_by_readiness
        if ($null -eq $consumed) {
            if ($index -ne ($links.Count - 1)) { throw "Only the final correction lineage link may be unconsumed." }
            continue
        }
        $successorAttempt = [int]$consumed.attempt
        if ($successorAttempt -lt 1 -or @("passed", "failed") -cnotcontains [string]$consumed.receipt.state) {
            throw "Correction lineage successor Readiness is not one positive terminal attempt."
        }
        if ($phase -ceq "validation-readiness" -and
            $successorAttempt -ne [int]$authorizationRef.document.allowed_successor_attempt) {
            throw "Correction authorization was consumed by a different Readiness attempt."
        }
        $startRef = Resolve-LineageReference $consumed.start_receipt "Successor Readiness immutable start"
        $consumptionRef = Resolve-LineageReference $consumed.consumption_receipt "Correction consumption"
        $terminalRef = Resolve-LineageReference $consumed.receipt "Successor Readiness receipt"
        $expectedStartPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/readiness-$successorAttempt-start.json"
        $expectedTerminalPath = Get-Sprint8AEvidenceRelativePath -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -RelativePath "attempts/readiness-$successorAttempt.json"
        $expectedConsumptionPath = "$expectedAuthorizationPath.consumption.json"
        if ([string]$startRef.path -cne $expectedStartPath -or
            [string]$terminalRef.path -cne $expectedTerminalPath -or
            [string]$consumptionRef.path -cne $expectedConsumptionPath) {
            throw "Correction consumption substitutes a noncanonical start, consumption, or terminal path."
        }
        foreach ($reference in @($startRef, $consumptionRef, $terminalRef)) { $references.Add($reference) }
        if ($seenConsumptionPaths.ContainsKey([string]$consumptionRef.path)) {
            throw "Correction lineage reuses a consumption path."
        }
        $seenConsumptionPaths[[string]$consumptionRef.path] = $true
        $expectedConsumptionPhase = "$phase-correction-consumption"
        $consumedAuthorizationPath = (Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$consumptionRef.document.authorization.path) -AllowLegacyAbsolute).path
        $consumedStartPath = (Resolve-Sprint8AEvidenceReference -RepositoryRoot $RepositoryRoot -EvidenceRoot $EvidenceRoot -Path ([string]$consumptionRef.document.successor_readiness.start_receipt) -AllowLegacyAbsolute).path
        $terminalAuthorization = Get-Sprint8AOptionalObjectPropertyValue -InputObject $terminalRef.document -Name "predecessor_correction_authorization"
        $terminalConsumption = Get-Sprint8AOptionalObjectPropertyValue -InputObject $terminalRef.document -Name "correction_consumption_receipt"
        $consumptionPredecessorMatches = if ($index -eq 0 -and $phase -ceq "candidate-rehearsal" -and $attempt -eq 30 -and
            $consumptionRef.document.PSObject.Properties.Name -contains "predecessor_rehearsal") {
            [int]$consumptionRef.document.predecessor_rehearsal.attempt -eq 30 -and
                [string]$consumptionRef.document.predecessor_rehearsal.receipt -ceq [string]$receiptRef.path -and
                [string]$consumptionRef.document.predecessor_rehearsal.sha256 -ceq [string]$receiptRef.sha256 -and
                [string]$consumptionRef.document.predecessor_rehearsal.harvest -ceq [string]$harvestRef.path -and
                [string]$consumptionRef.document.predecessor_rehearsal.defect_batch -ceq [string]$batchRef.path
        } else {
            $consumptionRef.document.PSObject.Properties.Name -contains "predecessor" -and
                (($consumptionRef.document.predecessor | ConvertTo-Json -Depth 100 -Compress) -ceq
                    ($link.predecessor | ConvertTo-Json -Depth 100 -Compress))
        }
        if ([string]$consumptionRef.document.phase -cne $expectedConsumptionPhase -or
            [string]$consumptionRef.document.state -cne "consumed" -or
            [string]$consumedAuthorizationPath -cne [string]$authorizationRef.path -or
            [string]$consumptionRef.document.authorization.sha256 -cne [string]$authorizationRef.sha256 -or
            -not $consumptionPredecessorMatches -or
            [int]$consumptionRef.document.successor_readiness.attempt -ne $successorAttempt -or
            [string]$consumedStartPath -cne [string]$startRef.path -or
            [string]$consumptionRef.document.successor_readiness.start_receipt_sha256 -cne [string]$startRef.sha256 -or
            ($startRef.document.schema_version -isnot [int] -and $startRef.document.schema_version -isnot [long]) -or
            [int]$startRef.document.schema_version -ne 2 -or
            [string]$startRef.document.sprint -cne "sprint-8a" -or
            [string]$startRef.document.phase -cne "validation-readiness" -or
            ($startRef.document.attempt -isnot [int] -and $startRef.document.attempt -isnot [long]) -or
            [int]$startRef.document.attempt -ne $successorAttempt -or
            [string]$startRef.document.state -cne "preparing" -or
            $startRef.document.assertions_started -isnot [bool] -or
            $startRef.document.assertions_started -ne $false -or
            $startRef.document.authoritative -isnot [bool] -or
            $startRef.document.authoritative -ne $false -or
            @($startRef.document.checks).Count -ne 0 -or
            @($startRef.document.prerequisite_receipts).Count -ne 0 -or
            $startRef.document.PSObject.Properties.Name -notcontains "predecessor_correction_authorization" -or
            $startRef.document.PSObject.Properties.Name -notcontains "correction_consumption_receipt" -or
            $null -ne (Get-Sprint8AOptionalObjectPropertyValue -InputObject $startRef.document -Name "predecessor_correction_authorization") -or
            $null -ne (Get-Sprint8AOptionalObjectPropertyValue -InputObject $startRef.document -Name "correction_consumption_receipt") -or
            ($terminalRef.document.schema_version -isnot [int] -and $terminalRef.document.schema_version -isnot [long]) -or
            @(2, 3) -notcontains [int]$terminalRef.document.schema_version -or
            [string]$terminalRef.document.sprint -cne "sprint-8a" -or
            [string]$terminalRef.document.phase -cne "validation-readiness" -or
            ($terminalRef.document.attempt -isnot [int] -and $terminalRef.document.attempt -isnot [long]) -or
            [int]$terminalRef.document.attempt -ne $successorAttempt -or
            [string]$terminalRef.document.state -cne [string]$consumed.receipt.state -or
            $terminalRef.document.authoritative -isnot [bool] -or
            $terminalRef.document.authoritative -ne $false -or
            $terminalRef.document.assertions_started -isnot [bool] -or
            ([string]$terminalRef.document.state -ceq "passed" -and $terminalRef.document.assertions_started -ne $true) -or
            @($terminalRef.document.prerequisite_receipts).Count -ne 0 -or
            $null -eq $terminalAuthorization -or
            [string]$terminalAuthorization.path -cne [string]$authorizationRef.path -or
            [string]$terminalAuthorization.sha256 -cne [string]$authorizationRef.sha256 -or
            $null -eq $terminalConsumption -or
            [string]$terminalConsumption.path -cne [string]$consumptionRef.path -or
            [string]$terminalConsumption.sha256 -cne [string]$consumptionRef.sha256) {
            throw "Correction consumption does not bind its authorization, immutable start, and terminal Readiness receipt."
        }
    }

    Assert-Sprint8ACorrectionLineageTopology `
        -Links $links `
        -PredecessorDocuments @($predecessorDocuments) `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot

    $tip = $links[-1]
    if ($RequireConsumedTip -and $null -eq $tip.consumed_by_readiness) {
        throw "Correction lineage tip has not been consumed by Readiness."
    }
    $currentReadinessBinding = $null
    if ($null -ne $ExpectedCurrentReadiness) {
        $currentReadinessValidation = Assert-Sprint8ACurrentReadinessReference `
            -StateReadiness $ExpectedCurrentReadiness `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot
        if ($null -eq $tip.consumed_by_readiness) {
            throw "Correction lineage tip has no consumed Readiness authority for the current Readiness."
        }
        $currentReadinessBinding = Assert-Sprint8AReadinessSupersessionChain `
            -DescendantReference ([pscustomobject]@{
                attempt = [int]$currentReadinessValidation.attempt
                path = [string]$currentReadinessValidation.immutable.path
                sha256 = [string]$currentReadinessValidation.immutable.sha256
                state = [string]$currentReadinessValidation.state
            }) `
            -AncestorReference $tip.consumed_by_readiness.receipt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot
        if ([string]$currentReadinessBinding.kind -ceq "clean_pre_rehearsal_supersession") {
            $retainedCurrentLineage = Get-Sprint8AOptionalObjectPropertyValue `
                -InputObject $currentReadinessValidation.immutable.document `
                -Name "correction_lineage"
            if (($retainedCurrentLineage | ConvertTo-Json -Depth 100 -Compress) -cne
                ($Lineage | ConvertTo-Json -Depth 100 -Compress)) {
                throw "Clean Readiness supersession does not preserve the exact current correction lineage."
            }
        }
        foreach ($reference in @($currentReadinessBinding.references)) { $references.Add($reference) }
        if ([string]$currentReadinessValidation.current.path -cne [string]$currentReadinessValidation.immutable.path) {
            $references.Add($currentReadinessValidation.current)
        }
    }
    $uniqueReferences = [Collections.Generic.List[object]]::new()
    $seenReferences = @{}
    foreach ($reference in @($references)) {
        $key = "$([string]$reference.path)|$([string]$reference.sha256)"
        if (-not $seenReferences.ContainsKey($key)) {
            $seenReferences[$key] = $true
            $uniqueReferences.Add($reference)
        }
    }
    [pscustomobject][ordered]@{
        links = @($links)
        tip = $tip
        current_readiness_binding = $currentReadinessBinding
        references = @($uniqueReferences)
    }
}

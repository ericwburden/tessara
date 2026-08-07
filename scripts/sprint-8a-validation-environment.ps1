Set-StrictMode -Version Latest

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
        "docs/sprints/sprint-8a-plan.md",
        "docs/sprints/sprint-8a-verification.md",
        "docs/sprints/sprint-8a-uat/*.md",
        "end2end/playwright.config.*",
        "end2end/tests/*.spec.ts",
        "scripts/sprint-8a-acceptance-contract.ps1",
        "scripts/smoke-sprint-8a.ps1",
        "scripts/uat-sprint-8a.ps1"
    )
    $deployment = Get-Sprint8APathSetDigest -RepositoryRoot $root -PathSpecs @(
        "deploy/sprint-8a/**",
        "deploy/sprint-7a/compose.yaml",
        "deploy/sprint-7a/blueprints/**",
        "crates/tessara-component-module/manifest.json",
        "crates/tessara-dashboard-module/manifest.json",
        "crates/tessara-reference-module/manifest.json"
    )

    [ordered]@{
        commit = $commit
        tree = $tree
        dirty = $status.Count -ne 0
        branch = $branch
        acceptance_inventory_sha256 = [string]$acceptance.sha256
        deployment_inputs_sha256 = [string]$deployment.sha256
    }
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
        [string]$PostgresContainerId
    )

    $sql = "BEGIN; CREATE TEMP TABLE tessara_readiness_probe(value integer); INSERT INTO tessara_readiness_probe VALUES (8); SELECT current_database() || '|' || current_user || '|' || (SELECT value::text FROM tessara_readiness_probe); ROLLBACK;"
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
    $expected = "$($Binding.database)|$($Binding.role)|8"
    if (@($output | Where-Object { $_ -ceq $expected }).Count -ne 1) {
        throw "Authenticated database probe returned the wrong database or role for $($Binding.name)."
    }
    [ordered]@{
        database = [string]$Binding.database
        role = [string]$Binding.role
        transaction_round_trip = $true
        client = $client
    }
}

function Get-Sprint8ADeploymentEnvironmentProbe {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$ProbeDatabases
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
    $databaseContracts = @($bindings | ForEach-Object {
        $binding = $_
        $probe = if ($ProbeDatabases) { Invoke-Sprint8ADatabaseProbe -Binding $binding -PostgresContainerId $containerId } else { $null }
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
        [AllowNull()]$DeploymentProbe
    )

    $probeResult = if ($null -eq $DeploymentProbe) {
        Get-Sprint8ADeploymentEnvironmentProbe `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -ProbeDatabases:$ProbeDatabases
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

function Assert-Sprint8AReceiptSidecar {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
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

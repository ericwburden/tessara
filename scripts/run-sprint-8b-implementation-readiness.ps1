[CmdletBinding()]
param(
    [string]$Target,
    [string]$EvidenceRoot = "artifacts/sprint-8b-closeout/implementation",
    [switch]$ListTargets,
    [switch]$SelfTest,
    [switch]$Finalize,
    [switch]$WorkspaceTestOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$contractPath = Join-Path $repoRoot "docs/sprints/sprint-8b-validation-contract.json"
$contract = Get-Content -Raw -LiteralPath $contractPath | ConvertFrom-Json -Depth 100
$targets = @($contract.implementation_targets)
$targetIds = @($targets.id)
. (Join-Path $PSScriptRoot "sprint-8b-cargo-test-integrity.ps1")
Import-Module (Join-Path $PSScriptRoot "tessara-validation-policy.psm1") -Force

function Assert-RunnerContract {
    if ($targets.Count -ne 24) { throw "Runner expected 24 targets, found $($targets.Count)." }
    if (@($targetIds | Sort-Object -Unique).Count -ne $targetIds.Count) {
        throw "Runner target identities are not unique."
    }
    foreach ($item in $targets) {
        $expected = ".\scripts\run-sprint-8b-implementation-readiness.ps1 -Target $($item.id)"
        if ($item.command -cne $expected) {
            throw "Target '$($item.id)' command does not select itself exactly."
        }
    }
    $tokens = $null
    $parseErrors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile(
        $PSCommandPath,
        [ref]$tokens,
        [ref]$parseErrors
    )
    if ($parseErrors.Count -ne 0) {
        throw "Implementation runner cannot audit mappings because its source does not parse."
    }
    $targetSwitch = @($ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.SwitchStatementAst] -and
            [string]$node.Condition.Extent.Text -ceq '$Target'
    }, $true))
    if ($targetSwitch.Count -ne 1) {
        throw "Implementation runner must contain one exact target dispatch switch."
    }
    $mappedTargets = @($targetSwitch[0].Clauses | ForEach-Object {
        [string]$_.Item1.Value
    })
    if ($mappedTargets.Count -ne @($mappedTargets | Sort-Object -Unique).Count -or
        (@($mappedTargets | Sort-Object) -join "`n") -cne
            (@($targetIds | Sort-Object) -join "`n")) {
        throw "Implementation runner dispatch is not set-equal to all 24 tracked targets."
    }
}

function Invoke-CheckedCommand {
    param([Parameter(Mandatory)][string]$Program, [Parameter(Mandatory)][string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "'$Program $($Arguments -join ' ')' exited $LASTEXITCODE." }
}

function Invoke-CheckedCargoTest {
    param([Parameter(Mandatory)][string[]]$Arguments)
    Invoke-Sprint8BCheckedCargoTest -Arguments $Arguments | Out-Null
}

function Assert-CargoTestGuardSelfTest {
    Test-Sprint8BCargoTestIntegrity | Out-Null
}

function Get-Sprint8BWorkspaceTestDatabaseEnvironments {
    [ordered]@{
        DATABASE_URL = "tessara_workspace_test_sqlx"
        TEST_API_DATABASE_URL = "tessara_workspace_test_api"
        TEST_API_FRESH_DATABASE_URL = "tessara_workspace_test_api_fresh"
        TEST_API_ENROLLMENT_DATABASE_URL = "tessara_workspace_test_api_enrollment"
        SPRINT_6A_UPGRADE_DATABASE_URL = "tessara_workspace_test_api_upgrade"
        TEST_COMPONENT_MODULE_DATABASE_URL = "tessara_workspace_test_component"
        TEST_INSTALLATION_CONTROL_DATABASE_URL = "tessara_workspace_test_installation_control"
        TEST_REFERENCE_MODULE_DATABASE_URL = "tessara_workspace_test_reference"
    }
}

function Get-Sprint8BWorkspaceCargoTestArguments {
    @("test", "--workspace", "--all-features", "--locked", "--offline", "--jobs", "1")
}

function Assert-Sprint8BWorkspaceTestDatabaseContract {
    param(
        [Collections.IDictionary]$DatabaseEnvironments =
            (Get-Sprint8BWorkspaceTestDatabaseEnvironments),
        [string[]]$CargoArguments = @(Get-Sprint8BWorkspaceCargoTestArguments)
    )

    $databaseEnvironments = $DatabaseEnvironments
    $expectedEnvironmentNames = @(
        "DATABASE_URL", "TEST_API_DATABASE_URL", "TEST_API_FRESH_DATABASE_URL",
        "TEST_API_ENROLLMENT_DATABASE_URL", "SPRINT_6A_UPGRADE_DATABASE_URL",
        "TEST_COMPONENT_MODULE_DATABASE_URL", "TEST_INSTALLATION_CONTROL_DATABASE_URL",
        "TEST_REFERENCE_MODULE_DATABASE_URL"
    )
    $actualEnvironmentNames = @($databaseEnvironments.Keys | ForEach-Object { [string]$_ })
    $databaseNames = @($databaseEnvironments.Values | ForEach-Object { [string]$_ })
    $arguments = @($CargoArguments)
    if (($actualEnvironmentNames -join "`n") -cne ($expectedEnvironmentNames -join "`n") -or
        @($databaseNames | Sort-Object -Unique).Count -ne $databaseNames.Count -or
        @($databaseNames | Where-Object { $_ -cnotmatch '(^|_)test(s|ing)?(_|$)' }).Count -ne 0 -or
        ($arguments -join "`n") -cne
            (@("test", "--workspace", "--all-features", "--locked", "--offline", "--jobs", "1") -join "`n")) {
        throw "Sprint 8B full-workspace test database/command contract is not exact."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        environment_names = @($actualEnvironmentNames)
        database_names = @($databaseNames)
        cargo_arguments = @($arguments)
    }
}

function Assert-Sprint8BWorkspaceTestDatabaseContractSelfTest {
    Assert-Sprint8BWorkspaceTestDatabaseContract | Out-Null

    $missingEnvironment = [ordered]@{}
    foreach ($entry in (Get-Sprint8BWorkspaceTestDatabaseEnvironments).GetEnumerator()) {
        if ([string]$entry.Key -cne "TEST_API_FRESH_DATABASE_URL") {
            $missingEnvironment[[string]$entry.Key] = [string]$entry.Value
        }
    }
    try {
        Assert-Sprint8BWorkspaceTestDatabaseContract `
            -DatabaseEnvironments $missingEnvironment | Out-Null
        throw "Workspace database contract self-test accepted a missing destructive fresh-test database."
    } catch {
        if ($_.Exception.Message -notmatch 'database/command contract is not exact') { throw }
    }

    try {
        Assert-Sprint8BWorkspaceTestDatabaseContract `
            -CargoArguments @("test", "--workspace", "--locked", "--offline", "--jobs", "1") | Out-Null
        throw "Workspace database contract self-test accepted a narrowed full-workspace Cargo command."
    } catch {
        if ($_.Exception.Message -notmatch 'database/command contract is not exact') { throw }
    }
}

function Invoke-Sprint8BWorkspaceTestsWithDatabase {
    $containerName =
        "tessara-s8b-workspace-tests-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
    $contract = Assert-Sprint8BWorkspaceTestDatabaseContract
    $databaseEnvironments = Get-Sprint8BWorkspaceTestDatabaseEnvironments
    $environmentNames = @($databaseEnvironments.Keys) +
        @("SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET")
    $environmentBefore = [ordered]@{}
    foreach ($name in $environmentNames) {
        $environmentBefore[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
    }
    $failure = $null
    $provisioningState = "not_started"
    $cargoState = "not_started"
    $cleanupState = "not_started"
    try {
        $existing = @(& docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}")
        if ($LASTEXITCODE -ne 0 -or $existing.Count -ne 0) {
            throw "Disposable workspace test database name is not empty."
        }
        $containerId = (& docker run --detach --name $containerName `
            -e POSTGRES_USER=tessara_workspace `
            -e POSTGRES_PASSWORD=tessara_workspace `
            -e POSTGRES_DB=tessara_workspace `
            -p "127.0.0.1::5432" postgres:16-alpine).Trim()
        if ($LASTEXITCODE -ne 0 -or $containerId -cnotmatch '^[0-9a-f]{64}$') {
            throw "Could not start the isolated workspace test database."
        }
        $ready = $false
        for ($attempt = 1; $attempt -le 30; $attempt++) {
            & docker exec $containerName pg_isready `
                -h 127.0.0.1 -U tessara_workspace -d tessara_workspace *> $null
            if ($LASTEXITCODE -eq 0) { $ready = $true; break }
            Start-Sleep -Seconds 1
        }
        if (-not $ready) { throw "Workspace test database did not become ready." }
        $portLine = (& docker port $containerName 5432/tcp).Trim()
        if ($LASTEXITCODE -ne 0 -or $portLine -cnotmatch '^127\.0\.0\.1:(\d+)$') {
            throw "Could not resolve the isolated workspace test database port."
        }
        $workspaceDatabaseUrl =
            "postgres://tessara_workspace:tessara_workspace@127.0.0.1:$($Matches[1])"
        foreach ($entry in $databaseEnvironments.GetEnumerator()) {
            & docker exec $containerName createdb -U tessara_workspace $entry.Value
            if ($LASTEXITCODE -ne 0) {
                throw "Could not create isolated workspace test database '$($entry.Value)'."
            }
            [Environment]::SetEnvironmentVariable(
                [string]$entry.Key,
                "$workspaceDatabaseUrl/$($entry.Value)",
                "Process"
            )
        }
        $provisioningState = "passed"
        $env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET =
            "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"
        Invoke-CheckedCargoTest -Arguments @(Get-Sprint8BWorkspaceCargoTestArguments)
        $cargoState = "passed"
    } catch {
        $failure = $_
    } finally {
        foreach ($name in $environmentNames) {
            [Environment]::SetEnvironmentVariable(
                $name,
                $environmentBefore[$name],
                "Process"
            )
        }
        $present = @(& docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}")
        if ($LASTEXITCODE -ne 0) {
            if ($null -eq $failure) {
                $failure = [Management.Automation.RuntimeException]::new(
                    "Could not inspect the isolated workspace test database during teardown."
                )
            }
        } elseif ($present.Count -gt 0) {
            & docker rm -f $containerName | Out-Null
            if ($LASTEXITCODE -ne 0 -and $null -eq $failure) {
                $failure = [Management.Automation.RuntimeException]::new(
                    "Could not remove the isolated workspace test database."
                )
            }
        }
        $remaining = @(& docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}")
        if ($LASTEXITCODE -ne 0 -or $remaining.Count -ne 0) {
            $cleanupState = "failed"
            if ($null -eq $failure) {
                $failure = [Management.Automation.RuntimeException]::new(
                    "Workspace test database teardown was not exact."
                )
            }
        } else {
            $cleanupState = "passed"
        }
    }
    $failureMessage = if ($null -eq $failure) {
        $null
    } elseif ($failure -is [Management.Automation.ErrorRecord]) {
        [string]$failure.Exception.Message
    } else {
        [string]$failure.Message
    }
    $lifecycleReceipt = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "full-workspace-disposable-postgres-lifecycle"
        state = if ($null -eq $failure -and $provisioningState -ceq "passed" -and
            $cargoState -ceq "passed" -and $cleanupState -ceq "passed") { "passed" } else { "failed" }
        image = "postgres:16-alpine"
        container_name = $containerName
        environment_names = @($contract.environment_names)
        database_names = @($contract.database_names)
        command = [pscustomobject][ordered]@{
            program = "cargo"
            arguments = @($contract.cargo_arguments)
        }
        provisioning = [pscustomobject][ordered]@{ state = $provisioningState }
        cargo_tests = [pscustomobject][ordered]@{ state = $cargoState }
        cleanup_restoration = [pscustomobject][ordered]@{ state = $cleanupState; remaining = @($remaining) }
        failure = $failureMessage
    }
    if ($null -ne $script:TargetAttemptRoot) {
        $lifecycleReceipt | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath (Join-Path $script:TargetAttemptRoot "workspace-test-database.json") `
                -Encoding utf8NoBOM
    }
    if ($null -ne $failure) { throw $failure }
}

function Invoke-CheckedPowerShellSelfTest {
    param(
        [Parameter(Mandatory)][string]$ScriptName,
        [Parameter(Mandatory)][string]$OutputPath,
        [AllowEmptyCollection()][string[]]$AdditionalArguments = @()
    )

    $scriptPath = Join-Path $PSScriptRoot $ScriptName
    if (-not (Test-Path -LiteralPath $scriptPath -PathType Leaf)) {
        throw "Required Sprint 8B self-test wrapper is missing: $scriptPath"
    }
    $arguments = @("-NoProfile", "-File", $scriptPath, "-SelfTest") + $AdditionalArguments
    $lines = [Collections.Generic.List[string]]::new()
    & pwsh @arguments 2>&1 | ForEach-Object {
        $line = [string]$_
        $lines.Add($line)
        Write-Host $line
    }
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "Sprint 8B self-test '$ScriptName' exited $exitCode."
    }
    $text = @($lines) -join "`n"
    try { $document = $text | ConvertFrom-Json -Depth 100 } catch {
        throw "Sprint 8B self-test '$ScriptName' did not emit one parseable JSON receipt."
    }
    $terminalState = if ($null -ne $document.PSObject.Properties['state']) {
        [string]$document.state
    } elseif ($null -ne $document.PSObject.Properties['status']) {
        [string]$document.status
    } else { "" }
    if ([string]$document.sprint -cne "sprint-8b" -or $terminalState -cne "passed") {
        throw "Sprint 8B self-test '$ScriptName' did not return an exact passed Sprint 8B receipt."
    }
    $resolvedOutput = [IO.Path]::GetFullPath($OutputPath)
    [IO.Directory]::CreateDirectory((Split-Path -Parent $resolvedOutput)) | Out-Null
    [IO.File]::WriteAllText($resolvedOutput, "$text`n", [Text.UTF8Encoding]::new($false))
    $document
}

function Get-Sha256Text {
    param([Parameter(Mandatory)][string]$Text)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
    $hash = [Security.Cryptography.SHA256]::HashData($bytes)
    [Convert]::ToHexString($hash).ToLowerInvariant()
}

function Convert-TrackedGlobToRegex {
    param([Parameter(Mandatory)][string]$Pattern)
    $normalized = $Pattern.Replace("\", "/").TrimStart("./")
    $escaped = [regex]::Escape($normalized)
    $escaped = $escaped.Replace("\*\*", ".*")
    $escaped = $escaped.Replace("\*", "[^/]*")
    $escaped = $escaped.Replace("\?", "[^/]")
    "^$escaped$"
}

function Get-TrackedRepositoryFiles {
    $files = @(& git ls-files --cached --others --exclude-standard --deleted)
    if ($LASTEXITCODE -ne 0) { throw "Could not enumerate current repository files." }
    @($files | ForEach-Object { ([string]$_).Replace("\", "/") } | Where-Object {
        $_ -and -not $_.StartsWith("artifacts/") -and -not $_.StartsWith("target/")
    } | Where-Object { Test-Path -LiteralPath (Join-Path $repoRoot $_) -PathType Leaf } | Sort-Object -Unique)
}

function Get-DomainFingerprint {
    param(
        [Parameter(Mandatory)][object]$Domain,
        [Parameter(Mandatory)][string[]]$RepositoryFiles
    )
    $patterns = @($Domain.tracked_inputs | ForEach-Object { Convert-TrackedGlobToRegex ([string]$_) })
    $paths = @($RepositoryFiles | Where-Object {
        $path = $_
        @($patterns | Where-Object { $path -cmatch $_ }).Count -gt 0
    })
    $entries = @($paths | ForEach-Object {
        $fullPath = Join-Path $repoRoot $_
        if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
            throw "Dependency domain '$($Domain.name)' names missing file '$_'."
        }
        "$_`0$((Get-FileHash -Algorithm SHA256 -LiteralPath $fullPath).Hash.ToLowerInvariant())"
    })
    [ordered]@{
        domain = [string]$Domain.name
        sha256 = Get-Sha256Text (($entries -join "`n") + "`n")
        file_count = $paths.Count
    }
}

function Get-SourceSnapshot {
    param([Parameter(Mandatory)][object]$TargetContract)
    $commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[0-9a-f]{40}$') {
        throw "Could not resolve the implementation source commit."
    }
    $tree = (& git rev-parse 'HEAD^{tree}').Trim()
    if ($LASTEXITCODE -ne 0 -or $tree -cnotmatch '^[0-9a-f]{40}$') {
        throw "Could not resolve the implementation source tree."
    }
    $branch = (& git branch --show-current).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($branch)) {
        throw "Could not resolve the implementation source branch."
    }
    $status = @(& git status --short --untracked-files=all) -join "`n"
    if ($LASTEXITCODE -ne 0) { throw "Could not resolve the implementation source status." }
    $repositoryFiles = @(Get-TrackedRepositoryFiles)
    $domainFingerprints = foreach ($domainName in @($TargetContract.dependency_domains)) {
        $domain = @($contract.dependency_domains | Where-Object { [string]$_.name -ceq [string]$domainName })
        if ($domain.Count -ne 1) { throw "Target '$Target' references unknown dependency domain '$domainName'." }
        Get-DomainFingerprint -Domain $domain[0] -RepositoryFiles $repositoryFiles
    }
    [ordered]@{
        commit = $commit
        tree = $tree
        branch = $branch
        dirty = -not [string]::IsNullOrWhiteSpace($status)
        status_sha256 = Get-Sha256Text ($status + "`n")
        dependency_fingerprints = @($domainFingerprints)
    }
}

function Get-Sprint8BRepositoryRelativePath {
    param([Parameter(Mandatory)][string]$Path)

    $fullRoot = [IO.Path]::GetFullPath($repoRoot)
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith(
            $fullRoot + [IO.Path]::DirectorySeparatorChar,
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Implementation evidence path escapes the repository: $Path"
    }
    [IO.Path]::GetRelativePath($fullRoot, $fullPath).Replace("\", "/")
}

function Resolve-Sprint8BImplementationEvidencePath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$RequiredRoot
    )

    $fullPath = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
    }
    $null = Get-Sprint8BRepositoryRelativePath -Path $fullPath
    if (-not [string]::IsNullOrWhiteSpace($RequiredRoot)) {
        $fullRequiredRoot = [IO.Path]::GetFullPath($RequiredRoot).TrimEnd(
            [IO.Path]::DirectorySeparatorChar,
            [IO.Path]::AltDirectorySeparatorChar
        )
        if (-not $fullPath.StartsWith(
                $fullRequiredRoot + [IO.Path]::DirectorySeparatorChar,
                [StringComparison]::OrdinalIgnoreCase
            )) {
            throw "Implementation evidence '$Path' is outside '$fullRequiredRoot'."
        }
    }
    $fullPath
}

function Assert-Sprint8BFileSidecar {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedSha256
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Implementation evidence is missing: $Path"
    }
    $sidecarPath = "$Path.sha256"
    if (-not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        throw "Implementation evidence sidecar is missing: $sidecarPath"
    }
    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    $sidecar = (Get-Content -LiteralPath $sidecarPath -Raw).Trim()
    if ($sidecar -cnotmatch '^[0-9a-f]{64}$' -or $sidecar -cne $actual -or
        (-not [string]::IsNullOrWhiteSpace($ExpectedSha256) -and
            $actual -cne $ExpectedSha256)) {
        throw "Implementation evidence authentication failed: $Path"
    }
    $actual
}

function Read-Sprint8BAuthenticatedJsonFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$RequiredRoot,
        [string]$ExpectedSha256
    )

    $fullPath = Resolve-Sprint8BImplementationEvidencePath -Path $Path -RequiredRoot $RequiredRoot
    $sha = Assert-Sprint8BFileSidecar -Path $fullPath -ExpectedSha256 $ExpectedSha256
    try {
        $document = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json -Depth 100
    } catch {
        throw "Implementation evidence is not valid JSON: $fullPath. $($_.Exception.Message)"
    }
    [pscustomobject][ordered]@{
        document = $document
        full_path = $fullPath
        relative_path = Get-Sprint8BRepositoryRelativePath -Path $fullPath
        sha256 = $sha
        size = [long](Get-Item -LiteralPath $fullPath).Length
    }
}

function Assert-Sprint8BExactSourceIdentity {
    param(
        [Parameter(Mandatory)]$Actual,
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)][string]$Label
    )

    if ([string]$Actual.commit -cne [string]$Expected.commit -or
        [string]$Actual.tree -cne [string]$Expected.tree -or
        [bool]$Actual.dirty -or [bool]$Expected.dirty) {
        throw "$Label is not bound to the exact current clean commit/tree."
    }
    if ($null -ne $Expected.PSObject.Properties['branch'] -and
        $null -ne $Actual.PSObject.Properties['branch'] -and
        [string]$Actual.branch -cne [string]$Expected.branch) {
        throw "$Label is not bound to the current branch."
    }
}

function Get-Sprint8BCurrentCleanSourceIdentity {
    Push-Location $repoRoot
    try {
        $commit = (& git rev-parse HEAD).Trim()
        $tree = (& git rev-parse 'HEAD^{tree}').Trim()
        $branch = (& git branch --show-current).Trim()
        $status = @(& git status --porcelain=v1 --untracked-files=all)
        if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[0-9a-f]{40}$' -or
            $tree -cnotmatch '^[0-9a-f]{40}$' -or [string]::IsNullOrWhiteSpace($branch)) {
            throw "Could not resolve the exact implementation source identity."
        }
        $dirty = @($status | Where-Object {
            -not [string]::IsNullOrWhiteSpace([string]$_)
        }).Count -ne 0
        if ($dirty) { throw "Implementation readiness cannot finalize a dirty source worktree." }
        [pscustomobject][ordered]@{
            commit = $commit
            tree = $tree
            dirty = $false
            branch = $branch
        }
    } finally {
        Pop-Location
    }
}

function New-Sprint8BEvidenceIndexEntry {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidateSet("receipt", "log", "structured", "screenshot", "trace", "report", "other")]
        [string]$Kind
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Indexed implementation evidence is missing: $Path"
    }
    [pscustomobject][ordered]@{
        path = Get-Sprint8BRepositoryRelativePath -Path $Path
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        size = [long](Get-Item -LiteralPath $Path).Length
        kind = $Kind
    }
}

function Get-Sprint8BIndexedEvidenceDocument {
    param(
        [Parameter(Mandatory)]$TargetAudit,
        [Parameter(Mandatory)][string]$FileName
    )

    $matches = @($TargetAudit.index_entries | Where-Object {
        [IO.Path]::GetFileName([string]$_.path) -ceq $FileName
    })
    if ($matches.Count -ne 1) {
        throw "Target '$($TargetAudit.id)' must index exactly one '$FileName'."
    }
    $info = Read-Sprint8BAuthenticatedJsonFile -Path ([string]$matches[0].path) `
        -RequiredRoot $TargetAudit.attempt_root -ExpectedSha256 ([string]$matches[0].sha256)
    [pscustomobject][ordered]@{
        info = $info
        reference = [pscustomobject][ordered]@{
            path = $info.relative_path
            sha256 = $info.sha256
        }
    }
}

function Assert-Sprint8BRefreshEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][ValidateSet("Refresh", "Dag")][string]$Suite
    )

    $expectedBinary = if ($Suite -ceq "Refresh") {
        "refresh_integration"
    } else {
        "dependency_dag_integration"
    }
    $expectedIdentities = if ($Suite -ceq "Refresh") { @(
        "unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state",
        "ordered_fixed_bound_pages_promote_each_response_change_once",
        "interrupted_page_attempt_retry_converges_once_from_published_cursor",
        "concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay",
        "expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement",
        "refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding",
        "derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure",
        "refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing"
    ) } else { @(
        "candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt",
        "rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact",
        "downstream_materialization_failure_rolls_back_every_rebuilt_table"
    ) }
    $expectedCount = $expectedIdentities.Count
    $declared = @($Document.expected_test_identities | ForEach-Object { [string]$_ })
    $executed = @($Document.executed_test_identities | ForEach-Object { [string]$_ })
    $arguments = @($Document.command.arguments | ForEach-Object { [string]$_ })
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8b" -or
        [string]$Document.proof -cne "dataset-module-test-suite" -or
        [string]$Document.state -cne "passed" -or [string]$Document.suite -cne $Suite -or
        [string]$Document.test_binary -cne $expectedBinary -or
        [int]$Document.executed_test_count -ne $expectedCount -or
        $declared.Count -ne $expectedCount -or $executed.Count -ne $expectedCount -or
        @($declared | Sort-Object -Unique).Count -ne $expectedCount -or
        (@($declared | Sort-Object) -join "`n") -cne
            (@($expectedIdentities | Sort-Object) -join "`n") -or
        (@($executed | Sort-Object) -join "`n") -cne
            (@($expectedIdentities | Sort-Object) -join "`n") -or
        [string]$Document.database.mode -cne "disposable-postgres" -or
        [string]$Document.database.cleanup_restoration.state -cne "passed" -or
        ($arguments -join "`n") -cne (@(
            "test", "-p", "tessara-dataset-module", "--test", $expectedBinary,
            "--locked", "--offline", "--jobs", "1", "--", "--format", "terse"
        ) -join "`n")) {
        throw "Dataset $Suite evidence does not prove the exact disposable test selector and cleanup."
    }
}

function Assert-Sprint8BMaterializationEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][ValidateSet("Reference", "ReferenceNoOp")][string]$ExpectedTarget,
        [Parameter(Mandatory)]$ExpectedSource
    )

    Assert-Sprint8BExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "$ExpectedTarget materialization evidence"
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8b" -or
        [string]$Document.state -cne "passed" -or [string]$Document.target -cne $ExpectedTarget -or
        [string]$Document.compose_project -cnotmatch '^tessara-s8b-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Document.first_apply.operation_state -cne "succeeded" -or
        -not [bool]$Document.first_apply.changed -or [bool]$Document.first_apply.no_op -or
        [string]$Document.gateway_start_boundary.post_start_health -cne "passed" -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.cleanup_restoration.mode -cne "exact-project-teardown" -or
        [string]::IsNullOrWhiteSpace([string]$Document.fixture_receipt_path) -or
        [string]$Document.fixture_receipt_sha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw "$ExpectedTarget evidence does not prove clean first apply, health, fixture identity, and teardown."
    }
    if ($ExpectedTarget -ceq "ReferenceNoOp" -and
        ([string]$Document.semantic_noop.operation_state -cne "succeeded" -or
            [bool]$Document.semantic_noop.changed -or -not [bool]$Document.semantic_noop.no_op -or
            [string]$Document.semantic_noop_proof.state -cne "passed")) {
        throw "ReferenceNoOp evidence does not prove an exact semantic no-op."
    }
}

function Assert-Sprint8BFailureRecoveryEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$ExpectedSource
    )

    Assert-Sprint8BExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "failure-recovery evidence"
    $faultKeys = @($Document.failure_attempts | ForEach-Object {
        [string]$_.fault.fault_key
    } | Sort-Object)
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8b" -or
        [string]$Document.proof -cne "deterministic-failure-containment-retry-and-restoration" -or
        [string]$Document.state -cne "passed" -or [string]$Document.compose_project -cnotmatch '^tessara-s8b-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        ($faultKeys -join "`n") -cne (@("dataset.derived-rebuild", "response.incompatible") -join "`n") -or
        @($Document.failure_attempts | Where-Object {
            [string]$_.containment.partial_topology_teardown -cne "passed" -or
            [int]$_.child_exit_code -eq 0
        }).Count -ne 0 -or
        [string]$Document.successor_evidence.state -cne "passed" -or
        [string]$Document.successor_evidence.target -cne "ReferenceNoOp" -or
        [string]$Document.restoration_proof.empty_start -cne "passed" -or
        [string]$Document.restoration_proof.canonical_first_apply -cne "passed" -or
        [string]$Document.restoration_proof.semantic_noop -cne "passed" -or
        [string]$Document.restoration_proof.canonical_health -cne "passed" -or
        [string]$Document.restoration_proof.final_teardown -cne "passed" -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.cleanup_restoration.mode -cne
            "two-exact-partial-teardowns-plus-restored-successor-teardown") {
        throw "Failure-recovery evidence does not prove two contained faults and canonical restored teardown."
    }
}

function Assert-Sprint8BDatasetBrowserProofEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$ExpectedSource
    )

    Assert-Sprint8BExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "Dataset browser proof evidence"
    $expectedSnapshots = @(
        "datasets-directory-light-1440-win32.png",
        "datasets-editor-light-390-win32.png",
        "datasets-directory-dark-1440-win32.png",
        "datasets-editor-dark-390-win32.png",
        "datasets-revisions-dark-1024-win32.png",
        "datasets-preview-light-1440-win32.png",
        "module-parity-with-datasets-datasets-dark-1440-win32.png",
        "module-parity-with-datasets-components-dark-1440-win32.png",
        "module-parity-with-datasets-dashboards-dark-1440-win32.png",
        "module-parity-with-datasets-scoped-records-dark-1440-win32.png"
    )
    $expectedIdentities = @(
        "Sprint 8B independent Dataset module › editor options use only Dataset-owned browser routes",
        "Sprint 8B independent Dataset module › synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure",
        "Sprint 8B independent Dataset module › reverse consumers distinguish authorized empty unavailable and undisclosed states",
        "Sprint 8B independent Dataset module › mutation replay and static route precedence remain exact",
        "canonical module UI visual baselines › Datasets directory at 1440 px (light)",
        "canonical module UI visual baselines › Datasets editor at 390 px (light)",
        "canonical module UI visual baselines › Datasets directory at 1440 px (dark)",
        "canonical module UI visual baselines › Datasets editor at 390 px (dark)",
        "canonical module UI visual baselines › Datasets revisions at 1024 px",
        "canonical module UI visual baselines › Datasets preview at 1440 px",
        "canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas"
    )
    $snapshotNames = @($Document.snapshot_preflight.files | ForEach-Object { [string]$_.name })
    $identities = @($Document.playwright.identities | ForEach-Object { [string]$_ })
    $commands = @($Document.commands)
    $firstArguments = if ($commands.Count -ge 1) {
        @($commands[0].arguments | ForEach-Object { [string]$_ })
    } else { @() }
    $secondArguments = if ($commands.Count -ge 2) {
        @($commands[1].arguments | ForEach-Object { [string]$_ })
    } else { @() }
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8b" -or
        [string]$Document.proof -cne "owned-reference-focused-dataset-browser-and-visual" -or
        [string]$Document.state -cne "passed" -or
        [string]$Document.compose_project -cnotmatch '^tessara-s8b-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Document.environment.TESSARA_PLAYWRIGHT_DATA_STATE -cne "fresh" -or
        [string]$Document.snapshot_preflight.state -cne "passed" -or
        [int]$Document.snapshot_preflight.source_derived_count -ne 10 -or
        $snapshotNames.Count -ne 10 -or
        (@($snapshotNames | Sort-Object) -join "`n") -cne
            (@($expectedSnapshots | Sort-Object) -join "`n") -or
        @($Document.snapshot_preflight.files | Where-Object {
            [string]$_.sha256 -cnotmatch '^[0-9a-f]{64}$' -or [long]$_.size -lt 8
        }).Count -ne 0 -or
        [string]$Document.materialization.target -cne "Reference" -or
        [string]$Document.materialization.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]::IsNullOrWhiteSpace([string]$Document.materialization.fixture_receipt_path) -or
        $commands.Count -ne 2 -or
        @($commands | Where-Object {
            [string]$_.program -cne "npm" -or [int]$_.exit_code -ne 0 -or
            [string]$_.report_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            [string]$_.junit_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            [string]$_.log_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            [string]::IsNullOrWhiteSpace([string]$_.report_path) -or
            [string]::IsNullOrWhiteSpace([string]$_.junit_path) -or
            [string]::IsNullOrWhiteSpace([string]$_.log_path)
        }).Count -ne 0 -or
        ($firstArguments -join "`n") -cne
            (@(
                "--prefix", "end2end", "test", "--", "tests/datasets-module.spec.ts",
                "--update-snapshots=none"
            ) -join "`n") -or
        ($secondArguments -join "`n") -cne (@(
            "--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Datasets",
            "--update-snapshots=none"
        ) -join "`n") -or
        [string]$Document.playwright.state -cne "passed" -or
        [string]$Document.playwright.data_state -cne "fresh" -or
        [int]$Document.playwright.expected -ne 11 -or [int]$Document.playwright.passed -ne 11 -or
        [int]$Document.playwright.skipped -ne 0 -or [int]$Document.playwright.workers -ne 1 -or
        [int]$Document.playwright.retries -ne 0 -or
        [string]$Document.playwright.update_snapshots -cne "none" -or
        -not [bool]$Document.playwright.forbid_only -or
        [int]$Document.playwright.module.passed -ne 4 -or
        [int]$Document.playwright.visual.passed -ne 7 -or
        $identities.Count -ne 11 -or @($identities | Sort-Object -Unique).Count -ne 11 -or
        (@($identities | Sort-Object) -join "`n") -cne
            (@($expectedIdentities | Sort-Object) -join "`n") -or
        @($Document.playwright.reports).Count -ne 4 -or
        @($Document.playwright.reports | Where-Object {
            [string]$_.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            [string]::IsNullOrWhiteSpace([string]$_.path)
        }).Count -ne 0 -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.cleanup_restoration.mode -cne "exact-owned-reference-teardown" -or
        $null -ne $Document.failure) {
        throw "Dataset browser evidence does not prove the exact source-derived 10-snapshot, 11/11 focused owned-topology result and teardown."
    }
}

function Assert-Sprint8BCleanEnvironmentTargetEvidence {
    param(
        [Parameter(Mandatory)]$TargetAudit,
        [Parameter(Mandatory)]$ExpectedSource
    )

    switch ([string]$TargetAudit.id) {
        "ui-sdk-conformance" {
            $child = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "dataset-browser-proof.json"
            Assert-Sprint8BDatasetBrowserProofEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "migration-seed" {
            $child = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "migration-seed.json"
            Assert-Sprint8BMaterializationEvidence -Document $child.info.document `
                -ExpectedTarget Reference -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "clean-materialization" {
            $child = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "clean-materialization.json"
            Assert-Sprint8BMaterializationEvidence -Document $child.info.document `
                -ExpectedTarget Reference -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "semantic-noop" {
            $child = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "semantic-noop.json"
            Assert-Sprint8BMaterializationEvidence -Document $child.info.document `
                -ExpectedTarget ReferenceNoOp -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "failure-recovery" {
            $child = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "failure-containment.json"
            Assert-Sprint8BFailureRecoveryEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "response-incremental-sync" {
            $child = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "refresh-integration.json"
            Assert-Sprint8BRefreshEvidence -Document $child.info.document -Suite Refresh
            return $child.reference
        }
        "dataset-refresh-dag" {
            $refresh = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "refresh-closure.json"
            $dag = Get-Sprint8BIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "dependency-dag.json"
            Assert-Sprint8BRefreshEvidence -Document $refresh.info.document -Suite Refresh
            Assert-Sprint8BRefreshEvidence -Document $dag.info.document -Suite Dag
            return [pscustomobject][ordered]@{
                refresh = $refresh.reference
                dag = $dag.reference
            }
        }
        default {
            throw "Clean-environment target '$($TargetAudit.id)' has no exact evidence validator."
        }
    }
}

function Assert-Sprint8BImplementationTargetReceipt {
    param(
        [Parameter(Mandatory)]$ContractTarget,
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)]$ExpectedSource
    )

    $targetId = [string]$ContractTarget.id
    $targetRoot = Join-Path $EvidenceRootPath "targets/$targetId"
    $resultInfo = Read-Sprint8BAuthenticatedJsonFile -Path (Join-Path $targetRoot "result.json") `
        -RequiredRoot $targetRoot
    $result = $resultInfo.document
    Assert-Sprint8BExactSourceIdentity -Actual $result.source -Expected $ExpectedSource `
        -Label "Implementation target '$targetId'"
    $expectedProofClasses = @($ContractTarget.proof_classes | ForEach-Object { [string]$_ })
    $actualProofClasses = @($result.proof_classes | ForEach-Object { [string]$_ })
    if ([int]$result.schema_version -ne 1 -or [string]$result.sprint -cne "sprint-8b" -or
        [string]$result.phase -cne "implementation-readiness" -or [bool]$result.authoritative -or
        [string]$result.target -cne $targetId -or [string]$result.state -cne "passed" -or
        $null -ne $result.failure -or [string]$result.command -cne [string]$ContractTarget.command -or
        [bool]$result.clean_environment -ne [bool]$ContractTarget.clean_environment -or
        ($actualProofClasses -join "`n") -cne ($expectedProofClasses -join "`n") -or
        [string]$result.contract_sha256 -cne
            (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash.ToLowerInvariant()) {
        throw "Implementation target '$targetId' receipt does not match its exact tracked contract."
    }

    $indexInfo = Read-Sprint8BAuthenticatedJsonFile -Path ([string]$result.evidence_index.path) `
        -RequiredRoot $targetRoot -ExpectedSha256 ([string]$result.evidence_index.sha256)
    $index = $indexInfo.document
    $attemptRoot = Split-Path -Parent $indexInfo.full_path
    if ([int]$index.schema_version -ne 1 -or [string]$index.sprint -cne "sprint-8b" -or
        [string]$index.phase -cne "implementation-readiness" -or
        [string]$index.target -cne $targetId -or
        [string]$index.attempt_id -cne [IO.Path]::GetFileName($attemptRoot) -or
        @($index.evidence).Count -eq 0) {
        throw "Implementation target '$targetId' evidence index has the wrong identity."
    }

    $entryPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $authenticatedEntries = [Collections.Generic.List[object]]::new()
    foreach ($entry in @($index.evidence)) {
        $entryPath = Resolve-Sprint8BImplementationEvidencePath -Path ([string]$entry.path) `
            -RequiredRoot $attemptRoot
        $relative = Get-Sprint8BRepositoryRelativePath -Path $entryPath
        if (-not $entryPaths.Add($relative)) {
            throw "Implementation target '$targetId' evidence index duplicates '$relative'."
        }
        if (-not (Test-Path -LiteralPath $entryPath -PathType Leaf)) {
            throw "Implementation target '$targetId' indexed evidence is missing: $relative"
        }
        $actualSha = (Get-FileHash -LiteralPath $entryPath -Algorithm SHA256).Hash.ToLowerInvariant()
        $actualSize = [long](Get-Item -LiteralPath $entryPath).Length
        if ([string]$entry.sha256 -cne $actualSha -or [long]$entry.size -ne $actualSize) {
            throw "Implementation target '$targetId' indexed evidence changed: $relative"
        }
        $authenticatedEntries.Add([pscustomobject][ordered]@{
            path = $relative
            sha256 = $actualSha
            size = $actualSize
        })
    }
    $commandPath = Resolve-Sprint8BImplementationEvidencePath -Path ([string]$result.command_log.path) `
        -RequiredRoot $attemptRoot
    $commandRelative = Get-Sprint8BRepositoryRelativePath -Path $commandPath
    $commandEntry = @($authenticatedEntries | Where-Object { [string]$_.path -ceq $commandRelative })
    if ($commandEntry.Count -ne 1 -or
        [string]$commandEntry[0].sha256 -cne [string]$result.command_log.sha256) {
        throw "Implementation target '$targetId' command transcript is not exactly indexed."
    }

    $aggregateFiles = [Collections.Generic.List[object]]::new()
    $aggregateFiles.Add((New-Sprint8BEvidenceIndexEntry -Path $resultInfo.full_path -Kind receipt))
    $aggregateFiles.Add((New-Sprint8BEvidenceIndexEntry -Path "$($resultInfo.full_path).sha256" -Kind structured))
    $aggregateFiles.Add((New-Sprint8BEvidenceIndexEntry -Path $indexInfo.full_path -Kind structured))
    $aggregateFiles.Add((New-Sprint8BEvidenceIndexEntry -Path "$($indexInfo.full_path).sha256" -Kind structured))
    foreach ($entry in @($authenticatedEntries)) {
        $fullEntry = Resolve-Sprint8BImplementationEvidencePath -Path ([string]$entry.path) `
            -RequiredRoot $attemptRoot
        $kind = if ([IO.Path]::GetFileName($fullEntry) -ceq "command.log") {
            "log"
        } elseif ([IO.Path]::GetExtension($fullEntry) -ceq ".json") {
            "receipt"
        } else {
            "structured"
        }
        $aggregateFiles.Add((New-Sprint8BEvidenceIndexEntry -Path $fullEntry -Kind $kind))
    }

    $audit = [pscustomobject][ordered]@{
        id = $targetId
        contract_target = $ContractTarget
        result = $result
        result_reference = [pscustomobject][ordered]@{
            path = $resultInfo.relative_path
            sha256 = $resultInfo.sha256
        }
        attempt_root = $attemptRoot
        index_entries = @($authenticatedEntries)
        aggregate_files = @($aggregateFiles)
        clean_environment_evidence = $null
    }
    if ([bool]$ContractTarget.clean_environment) {
        $audit.clean_environment_evidence = Assert-Sprint8BCleanEnvironmentTargetEvidence `
            -TargetAudit $audit -ExpectedSource $ExpectedSource
    }
    $audit
}

function Get-Sprint8BImplementationFinalizationInputs {
    param(
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)]$ExpectedSource
    )

    if ([bool]$ExpectedSource.dirty) {
        throw "Implementation readiness finalization requires a clean source identity."
    }
    $audits = [Collections.Generic.List[object]]::new()
    $byId = @{}
    foreach ($contractTarget in @($targets)) {
        $audit = Assert-Sprint8BImplementationTargetReceipt -ContractTarget $contractTarget `
            -EvidenceRootPath $EvidenceRootPath -ExpectedSource $ExpectedSource
        $audits.Add($audit)
        $byId[[string]$audit.id] = $audit
    }
    if ($audits.Count -ne 24 -or @($byId.Keys).Count -ne 24) {
        throw "Implementation readiness finalization did not authenticate exactly all 24 targets."
    }
    [pscustomobject][ordered]@{
        source = $ExpectedSource
        audits = @($audits)
        first_apply = $byId["clean-materialization"].clean_environment_evidence
        semantic_no_op = $byId["semantic-noop"].clean_environment_evidence
        recovery = $byId["failure-recovery"].clean_environment_evidence
    }
}

function ConvertTo-Sprint8BJsonText {
    param([Parameter(Mandatory)]$Document)
    ($Document | ConvertTo-Json -Depth 100) + "`n"
}

function Get-Sprint8BTextSha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}

function Write-Sprint8BNewEvidenceFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text
    )

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    $stream = [IO.File]::Open(
        $Path,
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::None
    )
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
}

function Publish-Sprint8BNewJsonPair {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document
    )

    $text = ConvertTo-Sprint8BJsonText -Document $Document
    $sha = Get-Sprint8BTextSha256 -Text $text
    Write-Sprint8BNewEvidenceFile -Path $Path -Text $text
    try {
        Write-Sprint8BNewEvidenceFile -Path "$Path.sha256" -Text "$sha`n"
    } catch {
        Remove-Item -LiteralPath $Path -Force
        throw
    }
    $sha
}

function Publish-Sprint8BImplementationReadinessResult {
    param(
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)]$FinalizationInputs
    )

    $fullEvidenceRoot = [IO.Path]::GetFullPath($EvidenceRootPath)
    $indexPath = Join-Path $fullEvidenceRoot "evidence-index.json"
    $resultPath = Join-Path $fullEvidenceRoot "implementation-readiness-result.json"
    foreach ($path in @($indexPath, "$indexPath.sha256", $resultPath, "$resultPath.sha256")) {
        if (Test-Path -LiteralPath $path) {
            throw "Implementation readiness finalization will not overwrite existing evidence: $path"
        }
    }

    $entryMap = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::Ordinal)
    foreach ($audit in @($FinalizationInputs.audits)) {
        foreach ($entry in @($audit.aggregate_files)) {
            $path = [string]$entry.path
            if ($entryMap.ContainsKey($path)) {
                $existing = $entryMap[$path]
                if ([string]$existing.sha256 -cne [string]$entry.sha256 -or
                    [long]$existing.size -ne [long]$entry.size) {
                    throw "Implementation aggregate evidence conflicts for '$path'."
                }
            } else {
                $entryMap.Add($path, $entry)
            }
        }
    }
    $entries = @($entryMap.Values | Sort-Object path)
    $index = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.phase-evidence-index"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8b"
        phase = "implementation-readiness"
        attempt = 1
        evidence_root = Get-Sprint8BRepositoryRelativePath -Path $fullEvidenceRoot
        sealed_at = [DateTimeOffset]::UtcNow.ToString("O")
        entry_count = $entries.Count
        entries = $entries
    }
    $null = Assert-TessaraPhaseEvidenceIndex -Index $index -RepositoryRoot $repoRoot -AuditFiles
    $indexText = ConvertTo-Sprint8BJsonText -Document $index
    $indexSha = Get-Sprint8BTextSha256 -Text $indexText
    $contractSha = (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $result = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.implementation-readiness-result"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8b"
        state = "passed"
        authoritative = $false
        source_identity = $FinalizationInputs.source
        validation_contract = [pscustomobject][ordered]@{
            path = Get-Sprint8BRepositoryRelativePath -Path $contractPath
            sha256 = $contractSha
        }
        affected_domains = @($contract.dependency_domains | ForEach-Object { [string]$_.name })
        targets = @($FinalizationInputs.audits | ForEach-Object {
            [pscustomobject][ordered]@{
                id = [string]$_.id
                state = "passed"
                command = [string]$_.contract_target.command
                clean_environment = [bool]$_.contract_target.clean_environment
                evidence = $_.result_reference
            }
        })
        known_failure_count = 0
        materialization = [pscustomobject][ordered]@{
            required = $true
            first_apply = [pscustomobject][ordered]@{
                required = $true; state = "passed"; evidence = $FinalizationInputs.first_apply
            }
            semantic_no_op = [pscustomobject][ordered]@{
                required = $true; state = "passed"; evidence = $FinalizationInputs.semantic_no_op
            }
            recovery = [pscustomobject][ordered]@{
                required = $true; state = "passed"; evidence = $FinalizationInputs.recovery
            }
        }
        cleanup_restoration = [pscustomobject][ordered]@{
            required = $true; state = "passed"; evidence = $FinalizationInputs.recovery
        }
        evidence_index = [pscustomobject][ordered]@{
            path = Get-Sprint8BRepositoryRelativePath -Path $indexPath
            sha256 = $indexSha
        }
    }
    $null = Assert-TessaraImplementationReadinessResult -Result $result `
        -Contract $contract -ContractPath $contractPath

    $indexPublished = $false
    $resultPublished = $false
    try {
        $publishedIndexSha = Publish-Sprint8BNewJsonPair -Path $indexPath -Document $index
        $indexPublished = $true
        if ($publishedIndexSha -cne $indexSha) {
            throw "Published implementation evidence index changed during finalization."
        }
        $null = Publish-Sprint8BNewJsonPair -Path $resultPath -Document $result
        $resultPublished = $true
        $published = Read-Sprint8BAuthenticatedJsonFile -Path $resultPath `
            -RequiredRoot $fullEvidenceRoot
        $null = Assert-TessaraImplementationReadinessResult -Result $published.document `
            -Contract $contract -ContractPath $contractPath
        $published.document
    } catch {
        if ($resultPublished) {
            Remove-Item -LiteralPath $resultPath, "$resultPath.sha256" -Force
        }
        if ($indexPublished) {
            Remove-Item -LiteralPath $indexPath, "$indexPath.sha256" -Force
        }
        throw
    }
}

function New-Sprint8BSyntheticRefreshEvidence {
    param([Parameter(Mandatory)][ValidateSet("Refresh", "Dag")][string]$Suite)

    $identities = if ($Suite -ceq "Refresh") { @(
        "unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state",
        "ordered_fixed_bound_pages_promote_each_response_change_once",
        "interrupted_page_attempt_retry_converges_once_from_published_cursor",
        "concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay",
        "expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement",
        "refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding",
        "derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure",
        "refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing"
    ) } else { @(
        "candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt",
        "rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact",
        "downstream_materialization_failure_rolls_back_every_rebuilt_table"
    ) }
    $binary = if ($Suite -ceq "Refresh") { "refresh_integration" } else { "dependency_dag_integration" }
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "dataset-module-test-suite"
        state = "passed"
        suite = $Suite
        test_binary = $binary
        expected_test_identities = $identities
        executed_test_identities = @($identities | Sort-Object)
        executed_test_count = $identities.Count
        database = [pscustomobject][ordered]@{
            mode = "disposable-postgres"
            cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
        }
        command = [pscustomobject][ordered]@{
            arguments = @(
                "test", "-p", "tessara-dataset-module", "--test", $binary,
                "--locked", "--offline", "--jobs", "1", "--", "--format", "terse"
            )
        }
    }
}

function New-Sprint8BSyntheticMaterializationEvidence {
    param(
        [Parameter(Mandatory)][ValidateSet("Reference", "ReferenceNoOp")][string]$TargetName,
        [Parameter(Mandatory)]$Source
    )

    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        state = "passed"
        target = $TargetName
        compose_project = "tessara-s8b-finalizer-selftest"
        source = $Source
        environment_fingerprint_sha256 = "c" * 64
        first_apply = [pscustomobject][ordered]@{
            operation_state = "succeeded"; changed = $true; no_op = $false
        }
        semantic_noop = if ($TargetName -ceq "ReferenceNoOp") {
            [pscustomobject][ordered]@{
                operation_state = "succeeded"; changed = $false; no_op = $true
            }
        } else { $null }
        semantic_noop_proof = if ($TargetName -ceq "ReferenceNoOp") {
            [pscustomobject][ordered]@{ state = "passed" }
        } else { $null }
        gateway_start_boundary = [pscustomobject][ordered]@{ post_start_health = "passed" }
        fixture_receipt_path = "target/sprint-8b-finalizer-selftest/fixture.json"
        fixture_receipt_sha256 = "d" * 64
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"; mode = "exact-project-teardown"
        }
    }
}

function New-Sprint8BSyntheticFailureEvidence {
    param([Parameter(Mandatory)]$Source)

    $attempts = @("response.incompatible", "dataset.derived-rebuild") | ForEach-Object {
        [pscustomobject][ordered]@{
            fault = [pscustomobject][ordered]@{ fault_key = $_ }
            containment = [pscustomobject][ordered]@{ partial_topology_teardown = "passed" }
            child_exit_code = 1
        }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "deterministic-failure-containment-retry-and-restoration"
        state = "passed"
        compose_project = "tessara-s8b-finalizer-selftest"
        source = $Source
        environment_fingerprint_sha256 = "e" * 64
        failure_attempts = $attempts
        successor_evidence = [pscustomobject][ordered]@{
            state = "passed"; target = "ReferenceNoOp"
        }
        restoration_proof = [pscustomobject][ordered]@{
            empty_start = "passed"
            canonical_first_apply = "passed"
            semantic_noop = "passed"
            canonical_health = "passed"
            final_teardown = "passed"
        }
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"
            mode = "two-exact-partial-teardowns-plus-restored-successor-teardown"
        }
    }
}

function New-Sprint8BSyntheticDatasetBrowserEvidence {
    param([Parameter(Mandatory)]$Source)

    $snapshotNames = @(
        "datasets-directory-light-1440-win32.png",
        "datasets-editor-light-390-win32.png",
        "datasets-directory-dark-1440-win32.png",
        "datasets-editor-dark-390-win32.png",
        "datasets-revisions-dark-1024-win32.png",
        "datasets-preview-light-1440-win32.png",
        "module-parity-with-datasets-datasets-dark-1440-win32.png",
        "module-parity-with-datasets-components-dark-1440-win32.png",
        "module-parity-with-datasets-dashboards-dark-1440-win32.png",
        "module-parity-with-datasets-scoped-records-dark-1440-win32.png"
    )
    $identities = @(
        "Sprint 8B independent Dataset module › editor options use only Dataset-owned browser routes",
        "Sprint 8B independent Dataset module › synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure",
        "Sprint 8B independent Dataset module › reverse consumers distinguish authorized empty unavailable and undisclosed states",
        "Sprint 8B independent Dataset module › mutation replay and static route precedence remain exact",
        "canonical module UI visual baselines › Datasets directory at 1440 px (light)",
        "canonical module UI visual baselines › Datasets editor at 390 px (light)",
        "canonical module UI visual baselines › Datasets directory at 1440 px (dark)",
        "canonical module UI visual baselines › Datasets editor at 390 px (dark)",
        "canonical module UI visual baselines › Datasets revisions at 1024 px",
        "canonical module UI visual baselines › Datasets preview at 1440 px",
        "canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas"
    )
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "owned-reference-focused-dataset-browser-and-visual"
        state = "passed"
        compose_project = "tessara-s8b-finalizer-selftest"
        source = $Source
        environment_fingerprint_sha256 = "d" * 64
        environment = [pscustomobject][ordered]@{ TESSARA_PLAYWRIGHT_DATA_STATE = "fresh" }
        snapshot_preflight = [pscustomobject][ordered]@{
            state = "passed"
            source_derived_count = 10
            files = @($snapshotNames | ForEach-Object {
                [pscustomobject][ordered]@{ name = $_; sha256 = "c" * 64; size = 9 }
            })
        }
        materialization = [pscustomobject][ordered]@{
            target = "Reference"
            sha256 = "b" * 64
            fixture_receipt_path = "target/selftest/fixture.json"
        }
        commands = @(
            [pscustomobject][ordered]@{
                program = "npm"; exit_code = 0
                report_path = "target/selftest/module.json"; report_sha256 = "a" * 64
                junit_path = "target/selftest/module.xml"; junit_sha256 = "b" * 64
                log_path = "target/selftest/module.log"; log_sha256 = "c" * 64
                arguments = @(
                    "--prefix", "end2end", "test", "--", "tests/datasets-module.spec.ts",
                    "--update-snapshots=none"
                )
            },
            [pscustomobject][ordered]@{
                program = "npm"; exit_code = 0
                report_path = "target/selftest/visual.json"; report_sha256 = "d" * 64
                junit_path = "target/selftest/visual.xml"; junit_sha256 = "e" * 64
                log_path = "target/selftest/visual.log"; log_sha256 = "f" * 64
                arguments = @(
                    "--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Datasets",
                    "--update-snapshots=none"
                )
            }
        )
        playwright = [pscustomobject][ordered]@{
            state = "passed"; data_state = "fresh"; expected = 11; passed = 11; skipped = 0
            workers = 1; retries = 0; update_snapshots = "none"; forbid_only = $true
            module = [pscustomobject][ordered]@{ passed = 4 }
            visual = [pscustomobject][ordered]@{ passed = 7 }
            identities = $identities
            reports = @(1..4 | ForEach-Object {
                [pscustomobject][ordered]@{ path = "target/selftest/report-$_.json"; sha256 = "a" * 64 }
            })
        }
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"; mode = "exact-owned-reference-teardown"
        }
        failure = $null
    }
}

function Test-Sprint8BImplementationFinalizationContract {
    $selfTestRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot (
        "target/sprint-8b-finalizer-selftest-$([Guid]::NewGuid().ToString('N'))"
    )))
    $allowedRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot "target"))
    if (-not $selfTestRoot.StartsWith(
            $allowedRoot + [IO.Path]::DirectorySeparatorChar,
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Finalizer self-test root is not confined to the repository target directory."
    }
    $source = [pscustomobject][ordered]@{
        commit = "a" * 40
        tree = "b" * 40
        dirty = $false
        branch = "codex/sprint-8b-selftest"
    }
    try {
        foreach ($contractTarget in @($targets)) {
            $targetId = [string]$contractTarget.id
            $targetRoot = Join-Path $selfTestRoot "targets/$targetId"
            $attemptRoot = Join-Path $targetRoot "attempts/selftest-attempt"
            [IO.Directory]::CreateDirectory($attemptRoot) | Out-Null
            $commandLogPath = Join-Path $attemptRoot "command.log"
            Write-Sprint8BNewEvidenceFile -Path $commandLogPath -Text "synthetic $targetId transcript`n"

            $childDocuments = [ordered]@{}
            switch ($targetId) {
                "ui-sdk-conformance" {
                    $childDocuments["dataset-browser-proof.json"] =
                        New-Sprint8BSyntheticDatasetBrowserEvidence -Source $source
                }
                "migration-seed" {
                    $childDocuments["migration-seed.json"] =
                        New-Sprint8BSyntheticMaterializationEvidence -TargetName Reference -Source $source
                }
                "clean-materialization" {
                    $childDocuments["clean-materialization.json"] =
                        New-Sprint8BSyntheticMaterializationEvidence -TargetName Reference -Source $source
                }
                "semantic-noop" {
                    $childDocuments["semantic-noop.json"] =
                        New-Sprint8BSyntheticMaterializationEvidence -TargetName ReferenceNoOp -Source $source
                }
                "failure-recovery" {
                    $childDocuments["failure-containment.json"] =
                        New-Sprint8BSyntheticFailureEvidence -Source $source
                }
                "response-incremental-sync" {
                    $childDocuments["refresh-integration.json"] =
                        New-Sprint8BSyntheticRefreshEvidence -Suite Refresh
                }
                "dataset-refresh-dag" {
                    $childDocuments["refresh-closure.json"] =
                        New-Sprint8BSyntheticRefreshEvidence -Suite Refresh
                    $childDocuments["dependency-dag.json"] =
                        New-Sprint8BSyntheticRefreshEvidence -Suite Dag
                }
            }
            foreach ($entry in $childDocuments.GetEnumerator()) {
                $null = Publish-Sprint8BNewJsonPair -Path (Join-Path $attemptRoot $entry.Key) `
                    -Document $entry.Value
            }

            $indexedFiles = @(Get-ChildItem -LiteralPath $attemptRoot -File | Sort-Object FullName)
            $targetIndex = [pscustomobject][ordered]@{
                schema_version = 1
                sprint = "sprint-8b"
                phase = "implementation-readiness"
                target = $targetId
                attempt_id = "selftest-attempt"
                evidence = @($indexedFiles | ForEach-Object {
                    [pscustomobject][ordered]@{
                        path = Get-Sprint8BRepositoryRelativePath -Path $_.FullName
                        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                        size = [long]$_.Length
                    }
                })
            }
            $targetIndexPath = Join-Path $attemptRoot "evidence-index.json"
            $targetIndexSha = Publish-Sprint8BNewJsonPair -Path $targetIndexPath -Document $targetIndex
            $targetResult = [pscustomobject][ordered]@{
                schema_version = 1
                sprint = "sprint-8b"
                phase = "implementation-readiness"
                authoritative = $false
                target = $targetId
                command = [string]$contractTarget.command
                proof_classes = @($contractTarget.proof_classes)
                clean_environment = [bool]$contractTarget.clean_environment
                state = "passed"
                started_at = [DateTimeOffset]::UtcNow.ToString("O")
                completed_at = [DateTimeOffset]::UtcNow.ToString("O")
                contract_sha256 = (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash.ToLowerInvariant()
                source = $source
                command_log = [pscustomobject][ordered]@{
                    path = Get-Sprint8BRepositoryRelativePath -Path $commandLogPath
                    sha256 = (Get-FileHash -LiteralPath $commandLogPath -Algorithm SHA256).Hash.ToLowerInvariant()
                }
                evidence_index = [pscustomobject][ordered]@{
                    path = Get-Sprint8BRepositoryRelativePath -Path $targetIndexPath
                    sha256 = $targetIndexSha
                }
                failure = $null
            }
            $null = Publish-Sprint8BNewJsonPair -Path (Join-Path $targetRoot "result.json") `
                -Document $targetResult
        }

        $inputs = Get-Sprint8BImplementationFinalizationInputs `
            -EvidenceRootPath $selfTestRoot -ExpectedSource $source
        if (@($inputs.audits).Count -ne 24) {
            throw "Finalizer self-test did not authenticate all 24 synthetic targets."
        }

        $missingResult = Join-Path $selfTestRoot "targets/static-quality/result.json"
        $movedResult = "$missingResult.missing"
        Move-Item -LiteralPath $missingResult -Destination $movedResult
        $missingRejected = $false
        try {
            Get-Sprint8BImplementationFinalizationInputs `
                -EvidenceRootPath $selfTestRoot -ExpectedSource $source | Out-Null
        } catch { $missingRejected = $true }
        Move-Item -LiteralPath $movedResult -Destination $missingResult
        if (-not $missingRejected) {
            throw "Finalizer self-test admitted a missing required target receipt."
        }

        $dirtyRejected = $false
        try {
            $dirtySource = $source | ConvertTo-Json -Depth 10 | ConvertFrom-Json
            $dirtySource.dirty = $true
            Assert-Sprint8BExactSourceIdentity -Actual $dirtySource -Expected $source `
                -Label "synthetic dirty target"
        } catch { $dirtyRejected = $true }
        if (-not $dirtyRejected) { throw "Finalizer self-test admitted dirty target evidence." }

        $tamperedNoOp = New-Sprint8BSyntheticMaterializationEvidence `
            -TargetName ReferenceNoOp -Source $source
        $tamperedNoOp.semantic_noop.no_op = $false
        $noOpRejected = $false
        try {
            Assert-Sprint8BMaterializationEvidence -Document $tamperedNoOp `
                -ExpectedTarget ReferenceNoOp -ExpectedSource $source
        } catch { $noOpRejected = $true }
        if (-not $noOpRejected) { throw "Finalizer self-test admitted a mutating semantic no-op." }

        $tamperPath = Join-Path $selfTestRoot "tamper.json"
        $null = Publish-Sprint8BNewJsonPair -Path $tamperPath -Document ([pscustomobject]@{ value = 1 })
        Add-Content -LiteralPath $tamperPath -Value " " -NoNewline
        $authenticationRejected = $false
        try { Read-Sprint8BAuthenticatedJsonFile -Path $tamperPath | Out-Null } catch {
            $authenticationRejected = $true
        }
        if (-not $authenticationRejected) {
            throw "Finalizer self-test admitted evidence changed after sidecar publication."
        }

        $published = Publish-Sprint8BImplementationReadinessResult `
            -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs
        if ([string]$published.state -cne "passed" -or @($published.targets).Count -ne 24) {
            throw "Finalizer self-test did not publish a valid aggregate readiness result."
        }
        $overwriteRejected = $false
        try {
            Publish-Sprint8BImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs | Out-Null
        } catch { $overwriteRejected = $true }
        if (-not $overwriteRejected) {
            throw "Finalizer self-test overwrote an existing sealed aggregate."
        }

        [pscustomobject][ordered]@{
            state = "passed"
            target_count = 24
            missing_target_rejected = $missingRejected
            dirty_source_rejected = $dirtyRejected
            mutating_noop_rejected = $noOpRejected
            evidence_tamper_rejected = $authenticationRejected
            sealed_overwrite_rejected = $overwriteRejected
        }
    } finally {
        if (Test-Path -LiteralPath $selfTestRoot) {
            $resolved = [IO.Path]::GetFullPath($selfTestRoot)
            if (-not $resolved.StartsWith(
                    $allowedRoot + [IO.Path]::DirectorySeparatorChar,
                    [StringComparison]::OrdinalIgnoreCase
                )) {
                throw "Finalizer self-test cleanup target escaped the repository target directory."
            }
            Remove-Item -LiteralPath $resolved -Recurse -Force
        }
        if (Test-Path -LiteralPath $selfTestRoot) {
            throw "Finalizer self-test cleanup was not exact."
        }
    }
}

Assert-RunnerContract
$modeCount = @(
    [bool]$ListTargets,
    [bool]$SelfTest,
    [bool]$Finalize,
    [bool]$WorkspaceTestOnly,
    -not [string]::IsNullOrWhiteSpace($Target)
    | Where-Object { $_ }
).Count
if ($modeCount -ne 1) {
    throw "Select exactly one of -Target, -ListTargets, -SelfTest, -Finalize, or -WorkspaceTestOnly."
}
if ($ListTargets) { $targetIds; exit 0 }
if ($WorkspaceTestOnly) {
    Push-Location $repoRoot
    try {
        Invoke-Sprint8BWorkspaceTestsWithDatabase
    } finally {
        Pop-Location
    }
    exit 0
}
if ($SelfTest) {
    $rejected = $false
    try {
        if ($targetIds -cnotcontains "not-a-target") { throw "unknown target" }
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Runner self-test admitted an unknown target." }
    Assert-CargoTestGuardSelfTest
    Assert-Sprint8BWorkspaceTestDatabaseContractSelfTest
    $finalizationSelfTest = Test-Sprint8BImplementationFinalizationContract
    [pscustomobject]@{
        sprint = "sprint-8b"
        target_count = $targetIds.Count
        cargo_zero_test_guard = "passed"
        full_workspace_database_contract = "passed"
        finalization_contract = $finalizationSelfTest
        status = "passed"
    } | ConvertTo-Json
    exit 0
}
if ($Finalize) {
    $finalEvidenceRoot = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
    }
    $null = Get-Sprint8BRepositoryRelativePath -Path $finalEvidenceRoot
    if (-not (Test-Path -LiteralPath $finalEvidenceRoot -PathType Container)) {
        throw "Implementation readiness evidence root is missing: $finalEvidenceRoot"
    }
    $source = Get-Sprint8BCurrentCleanSourceIdentity
    $inputs = Get-Sprint8BImplementationFinalizationInputs `
        -EvidenceRootPath $finalEvidenceRoot -ExpectedSource $source
    Publish-Sprint8BImplementationReadinessResult `
        -EvidenceRootPath $finalEvidenceRoot -FinalizationInputs $inputs | ConvertTo-Json -Depth 100
    exit 0
}
if ($targetIds -cnotcontains $Target) { throw "Unknown Sprint 8B implementation target '$Target'." }

$targetContract = @($targets | Where-Object { [string]$_.id -ceq $Target })[0]
$evidenceRootPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
    [IO.Path]::GetFullPath($EvidenceRoot)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
}
$targetRoot = Join-Path $evidenceRootPath "targets/$Target"
$attemptId = "{0}-{1}" -f [DateTimeOffset]::UtcNow.ToString("yyyyMMddTHHmmssfffZ"), ([guid]::NewGuid().ToString("N").Substring(0, 8))
$script:TargetAttemptRoot = Join-Path $targetRoot "attempts/$attemptId"
[IO.Directory]::CreateDirectory($script:TargetAttemptRoot) | Out-Null
$commandLogPath = Join-Path $script:TargetAttemptRoot "command.log"
$sourceBefore = Get-SourceSnapshot -TargetContract $targetContract
$startedAt = [DateTimeOffset]::UtcNow
$state = "failed"
$failure = $null
$pushed = $false
$transcribing = $false
try {
    Start-Transcript -LiteralPath $commandLogPath -Force | Out-Null
    $transcribing = $true
    Push-Location $repoRoot
    $pushed = $true
    switch ($Target) {
        "static-quality" {
            Invoke-CheckedCommand -Program "cargo" -Arguments @("fmt", "--all", "--", "--check")
            Invoke-CheckedCommand -Program "cargo" -Arguments @(
                "check", "--workspace", "--all-targets", "--all-features",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCommand -Program "cargo" -Arguments @(
                "clippy", "--workspace", "--all-targets", "--all-features",
                "--locked", "--offline", "--jobs", "1", "--", "-D", "warnings"
            )
            Invoke-Sprint8BWorkspaceTestsWithDatabase
            & (Join-Path $PSScriptRoot "validate-e2e.ps1") `
                -InventoryOnly `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "typescript-browser-inventory.json")
            if (-not $?) { throw "Affected TypeScript/Playwright graph discovery failed." }
        }
        "planning-contract-alignment" {
            & (Join-Path $PSScriptRoot "assert-sprint-8b-planning-contract.ps1")
            if (-not $?) { throw "Planning contract alignment failed." }
        }
        "response-export-contract" {
            Invoke-CheckedCargoTest -Arguments @("test", "-p", "tessara-responses-contract", "--locked", "--offline")
            & (Join-Path $PSScriptRoot "test-sprint-8b-response-export-contract.ps1")
            if (-not $?) { throw "Response export database contract failed." }
        }
        "contract-boundary" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-responses-contract", "-p", "tessara-forms-contract",
                "-p", "tessara-control-plane-contract", "-p", "tessara-datasets-contract",
                "--locked", "--offline"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "core_service_providers", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "control_plane_catalog_provider", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Provider
            if (-not $?) { throw "Dataset signed provider contract integration failed." }
            & (Join-Path $PSScriptRoot "check-sprint-8b-dataset-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Dataset package/source contract boundary validation failed." }
        }
        "owner-product" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-datasets-contract", "-p", "tessara-dataset-ui",
                "-p", "tessara-dataset-module", "--lib", "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite All
            if (-not $?) { throw "Complete Dataset owner integration suite failed." }
        }
        "ui-sdk-conformance" {
            & (Join-Path $PSScriptRoot "check-web-crate-boundaries.ps1")
            if (-not $?) { throw "Web crate boundary validation failed." }
            & (Join-Path $PSScriptRoot "verify-module-sdk-boundaries.ps1")
            if (-not $?) { throw "Module SDK boundary validation failed." }
            & (Join-Path $PSScriptRoot "ui-sdk-conformance.ps1")
            if (-not $?) { throw "Module UI SDK conformance failed." }
            & (Join-Path $PSScriptRoot "build-module-ui-browser-assets.ps1") -Module all -Check
            if (-not $?) { throw "Module browser asset source-identity validation failed." }
            & (Join-Path $PSScriptRoot "run-sprint-8b-dataset-browser-proof.ps1") `
                -ComposeProject "tessara-s8b-readiness-dataset-browser" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "dataset-browser-proof.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Focused Dataset browser/visual proof failed." }
        }
        "core-subtraction" {
            & (Join-Path $PSScriptRoot "check-sprint-8b-dataset-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Dataset Core-subtraction boundary validation failed." }
        }
        "inventory-navigation" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "manifest_is_valid_and_route_action_inventory_is_exact", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "catalog_collection_and_core_navigation_defaults_are_frozen", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "manifest_route_specificity_places_static_siblings_before_parameters", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "sprint-8b-acceptance-contract.ps1") -SelfTest
            if (-not $?) { throw "Sprint 8B inventory/navigation contract failed." }
        }
        "migration-seed" {
            & (Join-Path $PSScriptRoot "materialize-sprint-8b.ps1") `
                -Target Reference `
                -ComposeProject "tessara-s8b-readiness-migration-seed" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "migration-seed.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8B complete owner-ordered migration/seed materialization failed." }
            & (Join-Path $PSScriptRoot "check-sprint-8b-dataset-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Sprint 8B migration/seed boundary validation failed." }
        }
        "clean-materialization" {
            & (Join-Path $PSScriptRoot "materialize-sprint-8b.ps1") `
                -Target Reference `
                -ComposeProject "tessara-s8b-readiness-clean-materialization" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "clean-materialization.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8B clean reference materialization failed." }
        }
        "semantic-noop" {
            & (Join-Path $PSScriptRoot "materialize-sprint-8b.ps1") `
                -Target ReferenceNoOp `
                -ComposeProject "tessara-s8b-readiness-semantic-noop" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "semantic-noop.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8B semantic no-op materialization failed." }
        }
        "failure-recovery" {
            & (Join-Path $PSScriptRoot "run-sprint-8b-failure-containment.ps1") `
                -ComposeProject "tessara-s8b-readiness-recovery" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "failure-containment.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8B deterministic failure containment/recovery failed." }
        }
        "consumer-cutover" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-datasets-contract", "-p", "tessara-components-contract",
                "-p", "tessara-component-module", "-p", "tessara-dashboard-module", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-component-consumer.ps1")
            if (-not $?) { throw "Component Dataset-v2 consumer integration failed." }
        }
        "response-incremental-sync" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-responses-contract", "--locked", "--offline"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Sync
            if (-not $?) { throw "Dataset incremental sync integration tests failed." }
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") `
                -Suite Refresh `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "refresh-integration.json")
            if (-not $?) { throw "Dataset HTTP refresh orchestration tests failed." }
        }
        "reverse-consumers" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "app_summary", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "operations::tests", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "dataset_source_usage_projection", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "source_usage", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-web-forms", "--features", "ssr", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Provider
            if (-not $?) { throw "Dataset reverse-consumer provider integration failed." }
        }
        "resource-resolution" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-datasets-contract", "resource_observation",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "modules::reference::tests", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "resource_observation", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Provider
            if (-not $?) { throw "Dataset resource-observation provider integration failed." }
        }
        "api-idempotency" {
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Product
            if (-not $?) { throw "Dataset product mutation replay integration failed." }
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Authoring
            if (-not $?) { throw "Dataset authoring mutation replay integration failed." }
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") `
                -Suite Refresh `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "refresh-idempotency.json")
            if (-not $?) { throw "Dataset refresh mutation replay integration failed." }
        }
        "dataset-refresh-dag" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "dependency_dag", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") `
                -Suite Refresh `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "refresh-closure.json")
            if (-not $?) { throw "Dataset HTTP refresh closure integration failed." }
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") `
                -Suite Dag `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "dependency-dag.json")
            if (-not $?) { throw "Dataset dependency-DAG integration failed." }
        }
        "fixture-acceptance" {
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "prepare-sprint-8b-uat-fixtures.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "fixture-preparation-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "sprint-8b-acceptance-contract.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "acceptance-contract-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "uat-sprint-8b.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "uat-predicate-selftest.json") | Out-Null
            & (Join-Path $PSScriptRoot "validate-e2e.ps1") `
                -InventoryOnly `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "browser-inventory.json")
            if (-not $?) { throw "Sprint 8B exact browser acceptance discovery failed." }
        }
        "deployed-smoke" {
            & (Join-Path $PSScriptRoot "run-sprint-8b-deployed-smoke.ps1") `
                -ComposeProject "tessara-s8b-readiness-smoke" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "deployed-smoke.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8B deployed acceptance smoke failed." }
        }
        "independent-upgrade-rollback" {
            & (Join-Path $PSScriptRoot "run-sprint-8b-dataset-upgrade.ps1") `
                -ComposeProject "tessara-s8b-readiness-upgrade" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "dataset-upgrade.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8B independent Dataset upgrade/rollback failed." }
        }
        "uat-readiness" {
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "prepare-sprint-8b-uat-fixtures.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "fixture-preparation-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "uat-sprint-8b.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "uat-predicate-selftest-output.json") `
                -AdditionalArguments @(
                    "-EvidencePath", (Join-Path $script:TargetAttemptRoot "uat-predicate-readiness.json")
                ) | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "run-sprint-8b-formal-uat.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "formal-uat-wrapper-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "sprint-8b-acceptance-contract.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "acceptance-contract-selftest-output.json") `
                -AdditionalArguments @(
                    "-EvidencePath", (Join-Path $script:TargetAttemptRoot "acceptance-contract-readiness.json")
                ) | Out-Null
        }
        "ui-provider-boundaries" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-ui", "--all-features", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "authoring", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8b-dataset-module.ps1") -Suite Provider
            if (-not $?) { throw "Dataset signed provider-boundary integration failed." }
            & (Join-Path $PSScriptRoot "check-sprint-8b-dataset-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Dataset browser/provider source boundary validation failed." }
            & (Join-Path $PSScriptRoot "sprint-8b-acceptance-contract.ps1")
            if (-not $?) { throw "Dataset browser/provider acceptance contract failed." }
        }
        "runner-selftest" {
            Assert-RunnerContract
            Assert-CargoTestGuardSelfTest
            Assert-Sprint8BWorkspaceTestDatabaseContractSelfTest
            Test-Sprint8BImplementationFinalizationContract | Out-Null
            foreach ($implementationHarness in @(
                "sprint-8b-harness-isolation.ps1",
                "materialize-sprint-8b.ps1",
                "prepare-sprint-8b-uat-fixtures.ps1",
                "sprint-8b-acceptance-contract.ps1",
                "run-sprint-8b-failure-containment.ps1",
                "run-sprint-8b-deployed-smoke.ps1",
                "run-sprint-8b-dataset-upgrade.ps1",
                "uat-sprint-8b.ps1",
                "run-sprint-8b-dataset-browser-proof.ps1"
            )) {
                $receiptName = ([IO.Path]::GetFileNameWithoutExtension($implementationHarness)) +
                    "-selftest.json"
                Invoke-CheckedPowerShellSelfTest -ScriptName $implementationHarness `
                    -OutputPath (Join-Path $script:TargetAttemptRoot $receiptName) | Out-Null
            }
            foreach ($formalWrapper in @(
                "validate-sprint-8b-readiness.ps1",
                "run-sprint-8b-candidate-rehearsal.ps1",
                "run-sprint-8b-validation-preflight.ps1",
                "run-sprint-8b-sit.ps1",
                "run-sprint-8b-formal-uat.ps1"
            )) {
                $receiptName = ([IO.Path]::GetFileNameWithoutExtension($formalWrapper)) + "-selftest.json"
                Invoke-CheckedPowerShellSelfTest -ScriptName $formalWrapper `
                    -OutputPath (Join-Path $script:TargetAttemptRoot $receiptName) | Out-Null
            }
        }
        default {
            throw "Target '$Target' is selectable but is not implemented by the current Sprint 8B slice."
        }
    }
    $state = "passed"
} catch {
    $failure = $_.Exception.Message
} finally {
    if ($pushed) { Pop-Location }
    if ($transcribing) { Stop-Transcript | Out-Null }
}

$sourceAfter = Get-SourceSnapshot -TargetContract $targetContract
if (($sourceBefore | ConvertTo-Json -Depth 20 -Compress) -cne ($sourceAfter | ConvertTo-Json -Depth 20 -Compress)) {
    $state = "failed"
    $failure = "Implementation source or a declared dependency changed while target '$Target' was running."
}
$evidenceFiles = @(Get-ChildItem -LiteralPath $script:TargetAttemptRoot -File -Recurse | Sort-Object FullName | ForEach-Object {
    [ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $_.FullName).Replace("\", "/")
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
        size = $_.Length
    }
})
$evidenceIndexPath = Join-Path $script:TargetAttemptRoot "evidence-index.json"
$evidenceIndex = [ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    phase = "implementation-readiness"
    target = $Target
    attempt_id = $attemptId
    evidence = $evidenceFiles
}
$evidenceIndex | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $evidenceIndexPath -Encoding utf8NoBOM
$evidenceIndexSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $evidenceIndexPath).Hash.ToLowerInvariant()
$evidenceIndexSha | Set-Content -LiteralPath "$evidenceIndexPath.sha256" -Encoding ascii
$result = [ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    phase = "implementation-readiness"
    authoritative = $false
    target = $Target
    command = [string]$targetContract.command
    proof_classes = @($targetContract.proof_classes)
    clean_environment = [bool]$targetContract.clean_environment
    state = $state
    started_at = $startedAt.ToString("O")
    completed_at = [DateTimeOffset]::UtcNow.ToString("O")
    contract_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $contractPath).Hash.ToLowerInvariant()
    source = $sourceAfter
    command_log = [ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $commandLogPath).Replace("\", "/")
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $commandLogPath).Hash.ToLowerInvariant()
    }
    evidence_index = [ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $evidenceIndexPath).Replace("\", "/")
        sha256 = $evidenceIndexSha
    }
    failure = $failure
}
$resultPath = Join-Path $targetRoot "result.json"
$result | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $resultPath -Encoding utf8NoBOM
(Get-FileHash -Algorithm SHA256 -LiteralPath $resultPath).Hash.ToLowerInvariant() |
    Set-Content -LiteralPath "$resultPath.sha256" -Encoding ascii
if ($state -ne "passed") { throw $failure }
$result | ConvertTo-Json -Depth 10

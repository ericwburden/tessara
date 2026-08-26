[CmdletBinding()]
param(
    [string]$Target,
    [string]$EvidenceRoot = "artifacts/sprint-8c-closeout/implementation",
    [switch]$ListTargets,
    [switch]$SelfTest,
    [switch]$Finalize,
    [switch]$WorkspaceTestOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$contractPath = Join-Path $repoRoot "docs/sprints/sprint-8c-validation-contract.json"
$contract = Get-Content -Raw -LiteralPath $contractPath | ConvertFrom-Json -Depth 100
$targets = @($contract.implementation_targets)
$targetIds = @($targets.id)
. (Join-Path $PSScriptRoot "sprint-8c-cargo-test-integrity.ps1")
Import-Module (Join-Path $PSScriptRoot "tessara-validation-policy.psm1") -Force
$script:TargetAttemptRoot = $null

function Assert-RunnerContract {
    if ($targets.Count -ne 23) { throw "Runner expected 23 targets, found $($targets.Count)." }
    if (@($targetIds | Sort-Object -Unique).Count -ne $targetIds.Count) {
        throw "Runner target identities are not unique."
    }
    foreach ($item in $targets) {
        $expected = ".\scripts\run-sprint-8c-implementation-readiness.ps1 -Target $($item.id)"
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
        throw "Implementation runner dispatch is not set-equal to all 23 tracked targets."
    }
    Assert-Sprint8CImplementationPublicationBoundaryPlacement
}

function Invoke-CheckedCommand {
    param([Parameter(Mandatory)][string]$Program, [Parameter(Mandatory)][string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "'$Program $($Arguments -join ' ')' exited $LASTEXITCODE." }
}

function Invoke-CheckedCargoTest {
    param([Parameter(Mandatory)][string[]]$Arguments)
    Invoke-Sprint8CCheckedCargoTest -Arguments $Arguments | Out-Null
}

function Assert-CargoTestGuardSelfTest {
    Test-Sprint8CCargoTestIntegrity | Out-Null
}

function Get-Sprint8CWorkspaceTestDatabaseEnvironments {
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

function Get-Sprint8CWorkspaceCargoTestArguments {
    @("test", "--workspace", "--all-features", "--locked", "--offline", "--jobs", "1")
}

function Assert-Sprint8CWorkspaceTestDatabaseContract {
    param(
        [Collections.IDictionary]$DatabaseEnvironments =
            (Get-Sprint8CWorkspaceTestDatabaseEnvironments),
        [string[]]$CargoArguments = @(Get-Sprint8CWorkspaceCargoTestArguments)
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
        throw "Sprint 8C full-workspace test database/command contract is not exact."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        environment_names = @($actualEnvironmentNames)
        database_names = @($databaseNames)
        cargo_arguments = @($arguments)
    }
}

function Assert-Sprint8CWorkspaceTestDatabaseContractSelfTest {
    Assert-Sprint8CWorkspaceTestDatabaseContract | Out-Null

    $missingEnvironment = [ordered]@{}
    foreach ($entry in (Get-Sprint8CWorkspaceTestDatabaseEnvironments).GetEnumerator()) {
        if ([string]$entry.Key -cne "TEST_API_FRESH_DATABASE_URL") {
            $missingEnvironment[[string]$entry.Key] = [string]$entry.Value
        }
    }
    try {
        Assert-Sprint8CWorkspaceTestDatabaseContract `
            -DatabaseEnvironments $missingEnvironment | Out-Null
        throw "Workspace database contract self-test accepted a missing destructive fresh-test database."
    } catch {
        if ($_.Exception.Message -notmatch 'database/command contract is not exact') { throw }
    }

    try {
        Assert-Sprint8CWorkspaceTestDatabaseContract `
            -CargoArguments @("test", "--workspace", "--locked", "--offline", "--jobs", "1") | Out-Null
        throw "Workspace database contract self-test accepted a narrowed full-workspace Cargo command."
    } catch {
        if ($_.Exception.Message -notmatch 'database/command contract is not exact') { throw }
    }
}

function Invoke-Sprint8CWorkspaceTestsWithDatabase {
    $containerName =
        "tessara-s8c-workspace-tests-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
    $contract = Assert-Sprint8CWorkspaceTestDatabaseContract
    $databaseEnvironments = Get-Sprint8CWorkspaceTestDatabaseEnvironments
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
        Invoke-CheckedCargoTest -Arguments @(Get-Sprint8CWorkspaceCargoTestArguments)
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
        sprint = "sprint-8c"
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
        throw "Required Sprint 8C self-test wrapper is missing: $scriptPath"
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
        throw "Sprint 8C self-test '$ScriptName' exited $exitCode."
    }
    $text = @($lines) -join "`n"
    try { $document = $text | ConvertFrom-Json -Depth 100 } catch {
        throw "Sprint 8C self-test '$ScriptName' did not emit one parseable JSON receipt."
    }
    $terminalState = if ($null -ne $document.PSObject.Properties['state']) {
        [string]$document.state
    } elseif ($null -ne $document.PSObject.Properties['status']) {
        [string]$document.status
    } else { "" }
    if ([string]$document.sprint -cne "sprint-8c" -or $terminalState -cne "passed") {
        throw "Sprint 8C self-test '$ScriptName' did not return an exact passed Sprint 8C receipt."
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

function Invoke-CheckedValidationPolicySelfTest {
    param([Parameter(Mandatory)][string]$OutputPath)

    $scriptPath = Join-Path $PSScriptRoot "test-tessara-validation-policy.ps1"
    $policyPath = Join-Path $PSScriptRoot "tessara-validation-policy.psm1"
    foreach ($requiredPath in @($scriptPath, $policyPath)) {
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Required validation-policy self-test input is missing: $requiredPath"
        }
    }

    $startedAt = [DateTimeOffset]::UtcNow
    $lines = [Collections.Generic.List[string]]::new()
    & pwsh -NoProfile -File $scriptPath -SelfTest 2>&1 | ForEach-Object {
        $line = [string]$_
        $lines.Add($line)
    }
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "Validation-policy adversarial self-test exited $exitCode."
    }
    $nonEmptyLines = @($lines | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($nonEmptyLines.Count -eq 0 -or
        $nonEmptyLines[-1] -cne "Tessara validation policy v2 self-tests passed.") {
        throw "Validation-policy adversarial self-test did not emit its exact success marker."
    }

    $resolvedOutput = [IO.Path]::GetFullPath($OutputPath)
    $logPath = [IO.Path]::ChangeExtension($resolvedOutput, ".log")
    $transcript = (@($lines) -join "`n") + "`n"
    Write-Sprint8CNewEvidenceFile -Path $logPath -Text $transcript
    try {
        $receipt = [pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation-policy-selftest-receipt"
            policy_version = "tessara-validation-v2"
            sprint = "sprint-8c"
            state = "passed"
            started_at = $startedAt.ToString("O")
            completed_at = [DateTimeOffset]::UtcNow.ToString("O")
            command = [pscustomobject][ordered]@{
                program = "pwsh"
                arguments = @("-NoProfile", "-File", "scripts/test-tessara-validation-policy.ps1", "-SelfTest")
            }
            inputs = @(
                [pscustomobject][ordered]@{
                    path = Get-Sprint8CRepositoryRelativePath -Path $scriptPath
                    sha256 = (Get-FileHash -LiteralPath $scriptPath -Algorithm SHA256).Hash.ToLowerInvariant()
                },
                [pscustomobject][ordered]@{
                    path = Get-Sprint8CRepositoryRelativePath -Path $policyPath
                    sha256 = (Get-FileHash -LiteralPath $policyPath -Algorithm SHA256).Hash.ToLowerInvariant()
                }
            )
            transcript = [pscustomobject][ordered]@{
                path = Get-Sprint8CRepositoryRelativePath -Path $logPath
                sha256 = (Get-FileHash -LiteralPath $logPath -Algorithm SHA256).Hash.ToLowerInvariant()
            }
        }
        Write-Sprint8CNewEvidenceFile -Path $resolvedOutput `
            -Text (ConvertTo-Sprint8CJsonText -Document $receipt)
        return $receipt
    } catch {
        if (Test-Path -LiteralPath $resolvedOutput -PathType Leaf) {
            Remove-Item -LiteralPath $resolvedOutput -Force
        }
        if (Test-Path -LiteralPath $logPath -PathType Leaf) {
            Remove-Item -LiteralPath $logPath -Force
        }
        throw
    }
}

function Test-Sprint8CValidationPolicySelfTestContract {
    $root = [IO.Path]::GetFullPath((Join-Path $repoRoot (
        "target/sprint-8c-policy-selftest-$([guid]::NewGuid().ToString('N'))"
    )))
    $allowedRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot "target"))
    if (-not $root.StartsWith(
            $allowedRoot + [IO.Path]::DirectorySeparatorChar,
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Validation-policy self-test root is not confined to the repository target directory."
    }
    try {
        $receipt = Invoke-CheckedValidationPolicySelfTest `
            -OutputPath (Join-Path $root "validation-policy-selftest.json")
        if ([string]$receipt.state -cne "passed" -or
            [string]$receipt.contract -cne "tessara.validation-policy-selftest-receipt") {
            throw "Validation-policy self-test receipt did not pass its exact contract."
        }
        [pscustomobject][ordered]@{
            state = "passed"
            canonical_adversarial_suite = "passed"
            receipt_authenticated_before_cleanup = $true
        }
    } finally {
        if (Test-Path -LiteralPath $root) {
            Remove-Item -LiteralPath $root -Recurse -Force
        }
    }
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

function Get-Sprint8CRepositoryRelativePath {
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

function Resolve-Sprint8CImplementationEvidencePath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$RequiredRoot
    )

    $fullPath = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
    }
    $null = Get-Sprint8CRepositoryRelativePath -Path $fullPath
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

function Assert-Sprint8CFileSidecar {
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

function Read-Sprint8CAuthenticatedJsonFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$RequiredRoot,
        [string]$ExpectedSha256
    )

    $fullPath = Resolve-Sprint8CImplementationEvidencePath -Path $Path -RequiredRoot $RequiredRoot
    $sha = Assert-Sprint8CFileSidecar -Path $fullPath -ExpectedSha256 $ExpectedSha256
    try {
        $document = Get-Content -LiteralPath $fullPath -Raw | ConvertFrom-Json -Depth 100
    } catch {
        throw "Implementation evidence is not valid JSON: $fullPath. $($_.Exception.Message)"
    }
    [pscustomobject][ordered]@{
        document = $document
        full_path = $fullPath
        relative_path = Get-Sprint8CRepositoryRelativePath -Path $fullPath
        sha256 = $sha
        size = [long](Get-Item -LiteralPath $fullPath).Length
    }
}

function Assert-Sprint8CExactSourceIdentity {
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

function Get-Sprint8CCurrentCleanSourceIdentity {
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

function New-Sprint8CEvidenceIndexEntry {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidateSet("receipt", "log", "structured", "screenshot", "trace", "report", "other")]
        [string]$Kind
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Indexed implementation evidence is missing: $Path"
    }
    [pscustomobject][ordered]@{
        path = Get-Sprint8CRepositoryRelativePath -Path $Path
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        size = [long](Get-Item -LiteralPath $Path).Length
        kind = $Kind
    }
}

function Get-Sprint8CIndexedEvidenceDocument {
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
    $info = Read-Sprint8CAuthenticatedJsonFile -Path ([string]$matches[0].path) `
        -RequiredRoot $TargetAudit.attempt_root -ExpectedSha256 ([string]$matches[0].sha256)
    [pscustomobject][ordered]@{
        info = $info
        reference = [pscustomobject][ordered]@{
            path = $info.relative_path
            sha256 = $info.sha256
        }
    }
}

function Assert-Sprint8CRefreshEvidence {
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
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8c" -or
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

function Assert-Sprint8CMaterializationEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][ValidateSet("Reference", "ReferenceNoOp")][string]$ExpectedTarget,
        [Parameter(Mandatory)]$ExpectedSource
    )

    Assert-Sprint8CExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "$ExpectedTarget materialization evidence"
    $expectedProof = if ($ExpectedTarget -ceq "ReferenceNoOp") {
        "clean-owner-materialization-and-semantic-noop"
    } else { "clean-owner-materialization" }
    $expectedOwners = @(
        "core", "tessara.responses", "tessara.datasets", "tessara.components",
        "tessara.dashboards", "tessara.reference.scoped-records"
    )
    $expectedResponseHealthProperties = @("checks", "schema_version", "status")
    $expectedResponseReadinessCodes = @(
        "response.database",
        "response.configuration",
        "response.security_state",
        "response.provider.forms",
        "response.provider.workflow",
        "response.events.publication",
        "response.export.publication"
    )
    $firstOwners = @($Document.first_apply.owner_order | ForEach-Object { [string]$_ })
    $responseLive = $Document.health.response_live
    $responseReady = $Document.health.response_ready
    $responseLiveProperties = @($responseLive.PSObject.Properties.Name | Sort-Object)
    $responseReadyProperties = @($responseReady.PSObject.Properties.Name | Sort-Object)
    $responseReadyChecks = @($responseReady.checks)
    $invalidResponseReadyChecks = @($responseReadyChecks | Where-Object {
        (@($_.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
            ((@("code", "message", "passing") | Sort-Object) -join "`n") -or
        -not [bool]$_.passing -or
        [string]::IsNullOrWhiteSpace([string]$_.message)
    })
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8c" -or
        [string]$Document.proof -cne $expectedProof -or [string]$Document.state -cne "passed" -or
        [string]$Document.target -cne $ExpectedTarget -or
        [string]$Document.compose_project -cnotmatch '^tessara-s8c-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Document.first_apply.receipt_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
        ($firstOwners -join "`n") -cne ($expectedOwners -join "`n") -or
        @($Document.first_apply.owner_receipts).Count -ne $expectedOwners.Count -or
        @($Document.first_apply.owner_receipts | Where-Object {
            -not [bool]$_.changed -or [string]$_.input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$_.result_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            @($_.resource_ids.PSObject.Properties).Count -eq 0
        }).Count -ne 0 -or
        [string]$Document.first_apply.operation_state -cne "succeeded" -or
        -not [bool]$Document.first_apply.changed -or [bool]$Document.first_apply.no_op -or
        [string]$Document.gateway_start_boundary.post_start_health -cne "passed" -or
        ($responseLiveProperties -join "`n") -cne
            ($expectedResponseHealthProperties -join "`n") -or
        ($responseReadyProperties -join "`n") -cne
            ($expectedResponseHealthProperties -join "`n") -or
        [int]$responseLive.schema_version -ne 1 -or
        [string]$responseLive.status -cne "passing" -or
        @($responseLive.checks).Count -ne 0 -or
        [int]$responseReady.schema_version -ne 1 -or
        [string]$responseReady.status -cne "passing" -or
        $responseReadyChecks.Count -ne $expectedResponseReadinessCodes.Count -or
        (@($responseReadyChecks.code | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedResponseReadinessCodes -join "`n") -or
        $invalidResponseReadyChecks.Count -ne 0 -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.cleanup_restoration.mode -cne "exact-project-teardown" -or
        [string]::IsNullOrWhiteSpace([string]$Document.fixture_receipt_path) -or
        [string]$Document.fixture_receipt_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        $null -ne $Document.failure) {
        throw "$ExpectedTarget evidence does not prove clean first apply, health, fixture identity, and teardown."
    }
    if ($ExpectedTarget -ceq "ReferenceNoOp" -and
        ([string]$Document.semantic_noop.operation_state -cne "succeeded" -or
            [bool]$Document.semantic_noop.changed -or -not [bool]$Document.semantic_noop.no_op -or
            [string]$Document.semantic_noop.previous_receipt_digest -cne
                [string]$Document.first_apply.receipt_digest -or
            (@($Document.semantic_noop.owner_order | ForEach-Object { [string]$_ }) -join "`n") -cne
                ($expectedOwners -join "`n") -or
            @($Document.semantic_noop.owner_receipts | Where-Object { [bool]$_.changed }).Count -ne 0 -or
            [string]$Document.semantic_noop_proof.state -cne "passed" -or
            -not [bool]$Document.semantic_noop_proof.stable_owner_receipts -or
            -not [bool]$Document.semantic_noop_proof.stable_container_topology)) {
        throw "ReferenceNoOp evidence does not prove an exact semantic no-op."
    }
}

function Assert-Sprint8CFailureRecoveryEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$ExpectedSource
    )

    Assert-Sprint8CExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "failure-recovery evidence"
    $expectedAttemptFields = @(
        "child_exit_code", "containment", "correlation_id", "fault",
        "materialization_evidence", "materialization_evidence_sha256"
    )
    $expectedFaultFields = @(
        "attempt", "attempt_limit", "expected_failure_code", "expected_outcome",
        "fault_key", "no_cross_owner_write", "no_unauthorized_state", "phase",
        "receipt_contract", "target_service", "transaction_field", "transaction_value"
    )
    $expectedContainmentFields = @(
        "fault_key", "fixture_published", "gateway_started", "materialization_state",
        "partial_topology_teardown"
    )
    $expectedMaterializationEvidenceFields = @("path", "sha256")
    $faultKeys = @($Document.failure_attempts | ForEach-Object {
        [string]$_.fault.fault_key
    } | Sort-Object)
    $invalidAttempts = @($Document.failure_attempts | Where-Object {
        (@($_.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
            ($expectedAttemptFields -join "`n") -or
        (@($_.fault.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
            ($expectedFaultFields -join "`n") -or
        (@($_.containment.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
            ($expectedContainmentFields -join "`n") -or
        (@($_.materialization_evidence.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
            ($expectedMaterializationEvidenceFields -join "`n") -or
        [string]$_.correlation_id -cnotmatch
            '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' -or
        [string]$_.materialization_evidence_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$_.materialization_evidence.sha256 -cne
            [string]$_.materialization_evidence_sha256 -or
        [string]::IsNullOrWhiteSpace([string]$_.materialization_evidence.path) -or
        [string]$_.fault.fault_key -cne [string]$_.containment.fault_key -or
        [string]$_.fault.receipt_contract -cnotmatch
            '^tessara\.sprint-8[bc]\.failure-control/v1$' -or
        [string]::IsNullOrWhiteSpace([string]$_.fault.target_service) -or
        [string]::IsNullOrWhiteSpace([string]$_.fault.phase) -or
        [string]::IsNullOrWhiteSpace([string]$_.fault.expected_failure_code) -or
        [string]$_.fault.expected_outcome -notin @("rejected_pre_write", "rolled_back") -or
        [string]$_.fault.transaction_field -notin @(
            "dataset_transaction", "response_transaction"
        ) -or
        [string]$_.fault.transaction_value -notin @("not_started", "rolled_back") -or
        [string]$_.containment.partial_topology_teardown -cne "passed" -or
        [string]$_.containment.materialization_state -cne "failed" -or
        [bool]$_.containment.gateway_started -or [bool]$_.containment.fixture_published -or
        [int]$_.child_exit_code -eq 0 -or
        [uint64]$_.fault.attempt -ne 1 -or [uint64]$_.fault.attempt_limit -ne 1 -or
        -not [bool]$_.fault.no_unauthorized_state -or
        -not [bool]$_.fault.no_cross_owner_write
    })
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8c" -or
        [string]$Document.proof -cne "deterministic-failure-containment-retry-and-restoration" -or
        [string]$Document.state -cne "passed" -or [string]$Document.compose_project -cnotmatch '^tessara-s8c-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        ($faultKeys -join "`n") -cne (@(
            "dataset.derived-rebuild", "response.bootstrap.mid-apply", "response.incompatible"
        ) -join "`n") -or
        $invalidAttempts.Count -ne 0 -or
        [string]$Document.successor_evidence.state -cne "passed" -or
        [string]$Document.successor_evidence.target -cne "ReferenceNoOp" -or
        [string]$Document.restoration_proof.empty_start -cne "passed" -or
        [string]$Document.restoration_proof.canonical_first_apply -cne "passed" -or
        [string]$Document.restoration_proof.semantic_noop -cne "passed" -or
        [string]$Document.restoration_proof.canonical_health -cne "passed" -or
        [string]$Document.restoration_proof.final_teardown -cne "passed" -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.cleanup_restoration.mode -cne
            "three-exact-partial-teardowns-plus-restored-successor-teardown" -or
        [string]$Document.cleanup_restoration.fault_controls -cne "cleared-to-none" -or
        [string]$Document.cleanup_restoration.empty_successor_start -cne "passed" -or
        $null -ne $Document.failure) {
        throw "Failure-recovery evidence does not prove three contained faults and canonical restored teardown."
    }
}

function Assert-Sprint8CResponseBrowserProofEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$ExpectedSource
    )

    Assert-Sprint8CExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "Response browser proof evidence"
    $expectedBaseline = @(
        [pscustomobject]@{ key = "response-directory"; name = "response-directory-light-1440x1000.png"; sha256 = "bd5f99fbddca32ca4970069748e9f1856c5a3a364cfa9933855590684a46cf1d"; size = 118940; width = 1440; height = 1000 },
        [pscustomobject]@{ key = "response-start"; name = "response-start-dark-390x844.png"; sha256 = "3d9002307b44facb306595b5e1483b20fe1c0a94957789c16cc02e8a25090273"; size = 30129; width = 390; height = 844 },
        [pscustomobject]@{ key = "response-draft-detail"; name = "response-draft-detail-light-1024x1366.png"; sha256 = "0611c3a6994cade1bc0dc272958a6e59e3aeb8c6f14781da6890a21ad050c863"; size = 65960; width = 1024; height = 1366 },
        [pscustomobject]@{ key = "response-draft-edit"; name = "response-draft-edit-dark-1440x1000.png"; sha256 = "7d279b6f73de84c9ed126a84188e6b32948f8fcde25c4bcc74ce418b2d4336b5"; size = 67625; width = 1440; height = 1000 },
        [pscustomobject]@{ key = "response-submitted-detail"; name = "response-submitted-detail-light-390x844.png"; sha256 = "a9475a2e0ebb3994b74a444f118e09ee17df7c61c09123ae566d2281e9e1b5c3"; size = 32058; width = 390; height = 844 },
        [pscustomobject]@{ key = "operations-response-status"; name = "operations-response-status-dark-1024x1366.png"; sha256 = "c80e6a60085d81d4237e3dbe25945c97a83aef5e969c6f50a54878e47ddb9618"; size = 131036; width = 1024; height = 1366 },
        [pscustomobject]@{ key = "module-management-response"; name = "module-management-response-light-1440x1000.png"; sha256 = "deea6d795117506d277e838a983826825cfb4de872133a8b682b7d631f6cfbd8"; size = 108466; width = 1440; height = 1000 },
        [pscustomobject]@{ key = "response-directory-javascript-disabled"; name = "response-directory-javascript-disabled-1024x1366.png"; sha256 = "7619f0a9f3c41637d6c820376c6a4b374d42f68e5104619abcb8f32b0582e1d7"; size = 32915; width = 1024; height = 1366 }
    )
    $expectedMatrix = [ordered]@{
        routes = @(
            "/responses", "/responses/new", "/responses/{response_id}",
            "/responses/{response_id}/edit", "/operations",
            "/administration/modules/tessara.responses"
        )
        roles = @("owner", "delegate", "manager", "restricted", "administrator")
        states = @(
            "populated", "empty", "loading", "draft", "submitted", "delegated",
            "restricted", "provider_degraded", "validation_error", "unsaved_dirty"
        )
        themes = @("light", "dark", "stored_theme", "system_theme")
        viewports = @("1440x1000", "1024x1366", "390x844", "200_percent_zoom")
        runtime = @(
            "javascript_disabled_ssr", "direct_refresh", "hydrated",
            "lifecycle_navigation", "no_external_assets"
        )
    }
    $expectedIdentities = @(
        "Sprint 8C independent Response module › direct documents use only Response-owned public browser routes",
        "Sprint 8C independent Response module › assignment-only start options reject retired Core start routes",
        "Sprint 8C independent Response module › lifecycle navigation preserves unsaved draft state when discard is declined",
        "Sprint 8C independent Response module › scoped review and module diagnostics remain explicit and nondisclosing",
        "canonical module UI visual baselines › Responses directory at 1440 px (light)",
        "canonical module UI visual baselines › Responses start at 390 px (dark)",
        "canonical module UI visual baselines › Responses draft detail at 1024 px (light)",
        "canonical module UI visual baselines › Responses draft editor at 1440 px (dark)",
        "canonical module UI visual baselines › Responses submitted detail at 390 px (light)",
        "canonical module UI visual baselines › Responses Module Management at 1440 px (light)"
    )
    $baselineFiles = @($Document.accepted_baseline_preflight.files)
    $baselineKeys = @($Document.accepted_baseline_preflight.keys | ForEach-Object { [string]$_ })
    $actualMatrix = $Document.accepted_baseline_preflight.required_matrix
    $matrixMismatch = $null -eq $actualMatrix
    if (-not $matrixMismatch) {
        $actualMatrixFields = @($actualMatrix.PSObject.Properties.Name | Sort-Object)
        $expectedMatrixFields = @($expectedMatrix.Keys | Sort-Object)
        $matrixMismatch = ($actualMatrixFields -join "`n") -cne
            ($expectedMatrixFields -join "`n")
        if (-not $matrixMismatch) {
            foreach ($field in $expectedMatrix.Keys) {
                $actualProperty = $actualMatrix.PSObject.Properties[[string]$field]
                if ($null -eq $actualProperty -or
                    (@($actualProperty.Value | ForEach-Object { [string]$_ }) -join "`n") -cne
                        (@($expectedMatrix[$field] | ForEach-Object { [string]$_ }) -join "`n")) {
                    $matrixMismatch = $true
                    break
                }
            }
        }
    }
    $identities = @($Document.playwright.identities | ForEach-Object { [string]$_ })
    $commands = @($Document.commands)
    $firstArguments = if ($commands.Count -ge 1) {
        @($commands[0].arguments | ForEach-Object { [string]$_ })
    } else { @() }
    $secondArguments = if ($commands.Count -ge 2) {
        @($commands[1].arguments | ForEach-Object { [string]$_ })
    } else { @() }
    if ([int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8c" -or
        [string]$Document.proof -cne "owned-reference-focused-response-browser-and-visual" -or
        [string]$Document.state -cne "passed" -or
        [string]$Document.compose_project -cnotmatch '^tessara-s8c-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Document.environment.TESSARA_PLAYWRIGHT_DATA_STATE -cne "fresh" -or
        [string]$Document.accepted_baseline_preflight.state -cne "passed" -or
        [int]$Document.accepted_baseline_preflight.accepted_case_count -ne 8 -or
        [string]$Document.accepted_baseline_preflight.source_commit -cne
            "8f6244e8df3e67a25ec537c671016544c1e05c19" -or
        $matrixMismatch -or
        $baselineFiles.Count -ne $expectedBaseline.Count -or
        ($baselineKeys -join "`n") -cne
            (@($expectedBaseline.key | ForEach-Object { [string]$_ }) -join "`n") -or
        @($baselineFiles | Where-Object {
            $index = [array]::IndexOf([object[]]$baselineFiles, $_)
            $index -lt 0 -or
            [string]$_.key -cne [string]$expectedBaseline[$index].key -or
            [string]$_.name -cne [string]$expectedBaseline[$index].name -or
            [string]$_.sha256 -cne [string]$expectedBaseline[$index].sha256 -or
            [long]$_.size -ne [long]$expectedBaseline[$index].size -or
            [int]$_.width -ne [int]$expectedBaseline[$index].width -or
            [int]$_.height -ne [int]$expectedBaseline[$index].height
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
                "--prefix", "end2end", "test", "--", "tests/responses-module.spec.ts",
                "--update-snapshots=none"
            ) -join "`n") -or
        ($secondArguments -join "`n") -cne (@(
            "--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Responses",
            "--update-snapshots=none"
        ) -join "`n") -or
        [string]$Document.playwright.state -cne "passed" -or
        [string]$Document.playwright.data_state -cne "fresh" -or
        [int]$Document.playwright.expected -ne 10 -or [int]$Document.playwright.passed -ne 10 -or
        [int]$Document.playwright.skipped -ne 0 -or [int]$Document.playwright.workers -ne 1 -or
        [int]$Document.playwright.retries -ne 0 -or
        [string]$Document.playwright.update_snapshots -cne "none" -or
        -not [bool]$Document.playwright.forbid_only -or
        [int]$Document.playwright.module.passed -ne 4 -or
        [int]$Document.playwright.visual.passed -ne 6 -or
        ($identities -join "`n") -cne ($expectedIdentities -join "`n") -or
        @($Document.playwright.reports).Count -ne 4 -or
        @($Document.playwright.reports | Where-Object {
            [string]$_.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            [string]::IsNullOrWhiteSpace([string]$_.path)
        }).Count -ne 0 -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        [string]$Document.cleanup_restoration.mode -cne "exact-owned-reference-teardown" -or
        $null -ne $Document.failure) {
        throw "Response browser evidence does not prove the exact accepted 8-case baseline, 10/10 focused owned-topology result, and teardown."
    }
}

function Assert-Sprint8CFocusedCleanEvidence {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$ExpectedSource,
        [Parameter(Mandatory)][string]$ExpectedProof,
        [Parameter(Mandatory)][int]$MinimumExecutedTests
    )
    Assert-Sprint8CExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "$ExpectedProof evidence"
    $executed = if ($null -ne $Document.executed_test_count) {
        [int]$Document.executed_test_count
    } elseif ($null -ne $Document.response_owner_tests) {
        [int]$Document.response_owner_tests + [int]$Document.dataset_sync_tests +
            [int]$Document.dataset_refresh_tests
    } else { 0 }
    if ([int]$Document.schema_version -ne 1 -or
        [string]$Document.sprint -cne "sprint-8c" -or
        [string]$Document.proof -cne $ExpectedProof -or
        [string]$Document.state -cne "passed" -or
        $executed -lt $MinimumExecutedTests -or
        [string]$Document.cleanup_restoration.state -cne "passed") {
        throw "$ExpectedProof evidence is incomplete."
    }
}

function Assert-Sprint8CResponseUpgradeEvidence {
    param([Parameter(Mandatory)]$Document, [Parameter(Mandatory)]$ExpectedSource)
    Assert-Sprint8CExactSourceIdentity -Actual $Document.source -Expected $ExpectedSource `
        -Label "Response upgrade evidence"
    $expectedSequence = @("0.9.0", "1.0.0", "0.9.0", "1.0.0")
    $expectedStages = @(
        "establish-compatible-baseline", "upgrade-to-candidate",
        "rollback-to-compatible-baseline", "restore-intended-candidate"
    )
    $contractSequence = @($Document.release_contract.sequence | ForEach-Object { [string]$_ })
    $contractStages = @($Document.release_contract.stages | ForEach-Object { [string]$_ })
    $transitions = @($Document.transitions)
    $transitionSequence = @($transitions.target_release | ForEach-Object { [string]$_ })
    $stages = @($Document.stage_snapshots)
    $preservation = @($Document.preservation_proofs)
    if ([int]$Document.schema_version -ne 1 -or
        [string]$Document.sprint -cne "sprint-8c" -or
        [string]$Document.proof -cne "independent-response-upgrade-rollback-restoration" -or
        [string]$Document.state -cne "passed" -or
        [string]$Document.compose_project -cnotmatch '^tessara-s8c-' -or
        [string]$Document.environment_fingerprint_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Document.release_contract.transition.owner -cne "tessara.responses" -or
        [string]$Document.release_contract.transition.intended_release -cne "1.0.0" -or
        ($contractSequence -join "`n") -cne ($expectedSequence -join "`n") -or
        ($contractStages -join "`n") -cne ($expectedStages -join "`n") -or
        ($transitionSequence -join "`n") -cne ($expectedSequence -join "`n") -or
        ((@($transitions.stage | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedStages -join "`n")) -or
        $stages.Count -ne 4 -or $preservation.Count -ne 4 -or
        [string]$Document.pre_exercise_snapshot.stage -cne "pre-exercise-candidate" -or
        [string]$Document.pre_exercise_snapshot.response.release -cne "1.0.0" -or
        ((@($stages.stage | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedStages -join "`n")) -or
        ((@($stages.response.release | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedSequence -join "`n")) -or
        ((@($preservation.stage | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedStages -join "`n")) -or
        @($transitions | Where-Object {
            [string]$_.fixed_owner_lockfile -cne "passed" -or
            [string]$_.target_image -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$_.target_manifest_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$_.plan_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$_.lockfile_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$_.receipt_lockfile_digest -cne [string]$_.lockfile_digest -or
            [string]$_.exact_delta.owner -cne "tessara.responses" -or
            [int]$_.exact_delta.action_count -ne 5 -or
            -not [bool]$_.exact_delta.exact
        }).Count -ne 0 -or
        @($preservation | Where-Object {
            [string]$_.state -cne "passed" -or
            [string]$_.response_state -cne "passed" -or
            [string]$_.module_instance_identity -cne "passed" -or
            [string]$_.typed_resource_identity -cne "passed" -or
            [string]$_.navigation_identity -cne "passed" -or
            [string]$_.outbox_positions -cne "passed" -or
            [string]$_.unrelated_owners -cne "passed"
        }).Count -ne 0 -or
        [string]$Document.release_fixture.baseline.version -cne "0.9.0" -or
        [string]$Document.release_fixture.candidate.version -cne "1.0.0" -or
        [string]$Document.release_fixture.baseline.runtime_image -ceq
            [string]$Document.release_fixture.candidate.runtime_image -or
        [string]$Document.release_fixture.baseline.executable_sha256 -ceq
            [string]$Document.release_fixture.candidate.executable_sha256 -or
        [string]$Document.release_restoration.state -cne "passed" -or
        [string]$Document.release_restoration.final_release -cne "1.0.0" -or
        [string]$Document.cleanup_restoration.state -cne "passed" -or
        $null -ne $Document.failure) {
        throw "Response upgrade/rollback evidence is incomplete."
    }
}

function Assert-Sprint8CCleanEnvironmentTargetEvidence {
    param(
        [Parameter(Mandatory)]$TargetAudit,
        [Parameter(Mandatory)]$ExpectedSource
    )

    switch ([string]$TargetAudit.id) {
        "ui-sdk-conformance" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "response-browser-proof.json"
            Assert-Sprint8CResponseBrowserProofEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "migration-seed" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "migration-seed.json"
            Assert-Sprint8CMaterializationEvidence -Document $child.info.document `
                -ExpectedTarget Reference -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "clean-materialization" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "clean-materialization.json"
            Assert-Sprint8CMaterializationEvidence -Document $child.info.document `
                -ExpectedTarget Reference -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "semantic-noop" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "semantic-noop.json"
            Assert-Sprint8CMaterializationEvidence -Document $child.info.document `
                -ExpectedTarget ReferenceNoOp -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "failure-recovery" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "failure-containment.json"
            Assert-Sprint8CFailureRecoveryEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource
            return $child.reference
        }
        "workflow-events" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "workflow-events.json"
            Assert-Sprint8CFocusedCleanEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource `
                -ExpectedProof "workflow-response-event-consumption" `
                -MinimumExecutedTests 2
            return $child.reference
        }
        "dataset-export" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "dataset-export.json"
            Assert-Sprint8CFocusedCleanEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource `
                -ExpectedProof "response-owner-to-dataset-export-boundary" `
                -MinimumExecutedTests 20
            return $child.reference
        }
        "independent-upgrade-rollback" {
            $child = Get-Sprint8CIndexedEvidenceDocument -TargetAudit $TargetAudit `
                -FileName "response-upgrade.json"
            Assert-Sprint8CResponseUpgradeEvidence -Document $child.info.document `
                -ExpectedSource $ExpectedSource
            return $child.reference
        }
        default {
            throw "Clean-environment target '$($TargetAudit.id)' has no exact evidence validator."
        }
    }
}

function Assert-Sprint8CImplementationTargetReceipt {
    param(
        [Parameter(Mandatory)]$ContractTarget,
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)]$ExpectedSource
    )

    $targetId = [string]$ContractTarget.id
    $targetRoot = Join-Path $EvidenceRootPath "targets/$targetId"
    $resultInfo = Read-Sprint8CAuthenticatedJsonFile -Path (Join-Path $targetRoot "result.json") `
        -RequiredRoot $targetRoot
    $result = $resultInfo.document
    Assert-Sprint8CExactSourceIdentity -Actual $result.source -Expected $ExpectedSource `
        -Label "Implementation target '$targetId'"
    $expectedProofClasses = @($ContractTarget.proof_classes | ForEach-Object { [string]$_ })
    $actualProofClasses = @($result.proof_classes | ForEach-Object { [string]$_ })
    if ([int]$result.schema_version -ne 1 -or [string]$result.sprint -cne "sprint-8c" -or
        [string]$result.phase -cne "implementation-readiness" -or [bool]$result.authoritative -or
        [string]$result.target -cne $targetId -or [string]$result.state -cne "passed" -or
        $null -ne $result.failure -or [string]$result.command -cne [string]$ContractTarget.command -or
        [bool]$result.clean_environment -ne [bool]$ContractTarget.clean_environment -or
        ($actualProofClasses -join "`n") -cne ($expectedProofClasses -join "`n") -or
        [string]$result.contract_sha256 -cne
            (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash.ToLowerInvariant()) {
        throw "Implementation target '$targetId' receipt does not match its exact tracked contract."
    }

    $indexInfo = Read-Sprint8CAuthenticatedJsonFile -Path ([string]$result.evidence_index.path) `
        -RequiredRoot $targetRoot -ExpectedSha256 ([string]$result.evidence_index.sha256)
    $index = $indexInfo.document
    $attemptRoot = Split-Path -Parent $indexInfo.full_path
    if ([int]$index.schema_version -ne 1 -or [string]$index.sprint -cne "sprint-8c" -or
        [string]$index.phase -cne "implementation-readiness" -or
        [string]$index.target -cne $targetId -or
        [string]$index.attempt_id -cne [IO.Path]::GetFileName($attemptRoot) -or
        @($index.evidence).Count -eq 0) {
        throw "Implementation target '$targetId' evidence index has the wrong identity."
    }

    $entryPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $authenticatedEntries = [Collections.Generic.List[object]]::new()
    foreach ($entry in @($index.evidence)) {
        $entryPath = Resolve-Sprint8CImplementationEvidencePath -Path ([string]$entry.path) `
            -RequiredRoot $attemptRoot
        $relative = Get-Sprint8CRepositoryRelativePath -Path $entryPath
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
    $commandPath = Resolve-Sprint8CImplementationEvidencePath -Path ([string]$result.command_log.path) `
        -RequiredRoot $attemptRoot
    $commandRelative = Get-Sprint8CRepositoryRelativePath -Path $commandPath
    $commandEntry = @($authenticatedEntries | Where-Object { [string]$_.path -ceq $commandRelative })
    if ($commandEntry.Count -ne 1 -or
        [string]$commandEntry[0].sha256 -cne [string]$result.command_log.sha256) {
        throw "Implementation target '$targetId' command transcript is not exactly indexed."
    }

    $aggregateFiles = [Collections.Generic.List[object]]::new()
    $aggregateFiles.Add((New-Sprint8CEvidenceIndexEntry -Path $resultInfo.full_path -Kind receipt))
    $aggregateFiles.Add((New-Sprint8CEvidenceIndexEntry -Path "$($resultInfo.full_path).sha256" -Kind structured))
    $aggregateFiles.Add((New-Sprint8CEvidenceIndexEntry -Path $indexInfo.full_path -Kind structured))
    $aggregateFiles.Add((New-Sprint8CEvidenceIndexEntry -Path "$($indexInfo.full_path).sha256" -Kind structured))
    foreach ($entry in @($authenticatedEntries)) {
        $fullEntry = Resolve-Sprint8CImplementationEvidencePath -Path ([string]$entry.path) `
            -RequiredRoot $attemptRoot
        $kind = if ([IO.Path]::GetFileName($fullEntry) -ceq "command.log") {
            "log"
        } elseif ([IO.Path]::GetExtension($fullEntry) -ceq ".json") {
            "receipt"
        } else {
            "structured"
        }
        $aggregateFiles.Add((New-Sprint8CEvidenceIndexEntry -Path $fullEntry -Kind $kind))
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
        $audit.clean_environment_evidence = Assert-Sprint8CCleanEnvironmentTargetEvidence `
            -TargetAudit $audit -ExpectedSource $ExpectedSource
    }
    $audit
}

function Get-Sprint8CImplementationFinalizationInputs {
    param(
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)]$ExpectedSource,
        [string]$DefectProvenanceEvidenceRootPath
    )

    if ([bool]$ExpectedSource.dirty) {
        throw "Implementation readiness finalization requires a clean source identity."
    }
    $provenanceEvidenceRoot = if ([string]::IsNullOrWhiteSpace(
        $DefectProvenanceEvidenceRootPath
    )) {
        [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$contract.evidence_policy.root)))
    } elseif ([IO.Path]::IsPathRooted($DefectProvenanceEvidenceRootPath)) {
        [IO.Path]::GetFullPath($DefectProvenanceEvidenceRootPath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repoRoot $DefectProvenanceEvidenceRootPath))
    }
    $null = Get-Sprint8CRepositoryRelativePath -Path $provenanceEvidenceRoot
    $chronology = Assert-TessaraDefectProvenanceChronology -RepositoryRoot $repoRoot `
        -EvidenceRoot $provenanceEvidenceRoot -Sprint "sprint-8c"
    $audits = [Collections.Generic.List[object]]::new()
    $byId = @{}
    foreach ($contractTarget in @($targets)) {
        $audit = Assert-Sprint8CImplementationTargetReceipt -ContractTarget $contractTarget `
            -EvidenceRootPath $EvidenceRootPath -ExpectedSource $ExpectedSource
        $audits.Add($audit)
        $byId[[string]$audit.id] = $audit
    }
    if ($audits.Count -ne 23 -or @($byId.Keys).Count -ne 23) {
        throw "Implementation readiness finalization did not authenticate exactly all 23 targets."
    }
    [pscustomobject][ordered]@{
        source = $ExpectedSource
        defect_provenance_evidence_root = $provenanceEvidenceRoot
        defect_provenance_chronology = $chronology
        audits = @($audits)
        first_apply = $byId["clean-materialization"].clean_environment_evidence
        semantic_no_op = $byId["semantic-noop"].clean_environment_evidence
        recovery = $byId["failure-recovery"].clean_environment_evidence
    }
}

function ConvertTo-Sprint8CJsonText {
    param([Parameter(Mandatory)]$Document)
    ($Document | ConvertTo-Json -Depth 100) + "`n"
}

function Get-Sprint8CTextSha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}

function Write-Sprint8CNewEvidenceFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [scriptblock]$AfterCreateHook
    )

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    $stream = $null
    $created = $false
    $writeFailure = $null
    try {
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
        $created = $true
        if ($null -ne $AfterCreateHook) { & $AfterCreateHook }
        $stream.Write($bytes, 0, $bytes.Length)
    } catch {
        $writeFailure = $_
    } finally {
        if ($null -ne $stream) { $stream.Dispose() }
    }
    if ($null -ne $writeFailure) {
        if ($created -and (Test-Path -LiteralPath $Path -PathType Leaf)) {
            Remove-Item -LiteralPath $Path -Force
        }
        throw $writeFailure
    }
}

function Publish-Sprint8CNewJsonPair {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document,
        [scriptblock]$BeforeSidecarPublicationHook
    )

    foreach ($candidate in @($Path, "$Path.sha256")) {
        if (Test-Path -LiteralPath $candidate) {
            throw "Evidence publication will not overwrite an existing file: $candidate"
        }
    }
    $text = ConvertTo-Sprint8CJsonText -Document $Document
    $sha = Get-Sprint8CTextSha256 -Text $text
    $jsonCreated = $false
    $sidecarCreated = $false
    try {
        Write-Sprint8CNewEvidenceFile -Path $Path -Text $text
        $jsonCreated = $true
        if ($null -ne $BeforeSidecarPublicationHook) {
            & $BeforeSidecarPublicationHook
        }
        Write-Sprint8CNewEvidenceFile -Path "$Path.sha256" -Text "$sha`n"
        $sidecarCreated = $true
    } catch {
        if ($sidecarCreated -and (Test-Path -LiteralPath "$Path.sha256" -PathType Leaf)) {
            Remove-Item -LiteralPath "$Path.sha256" -Force
        }
        if ($jsonCreated -and (Test-Path -LiteralPath $Path -PathType Leaf)) {
            Remove-Item -LiteralPath $Path -Force
        }
        throw
    }
    $sha
}

function Add-Sprint8CCreatedEvidenceFile {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()]
        [Collections.Generic.List[object]]$CreatedFiles,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedSha256,
        [scriptblock]$AfterRegistrationHook
    )

    $fullPath = [IO.Path]::GetFullPath($Path)
    $CreatedFiles.Add([pscustomobject][ordered]@{
        path = $fullPath
        sha256 = $ExpectedSha256
    })
    if ($null -ne $AfterRegistrationHook) { & $AfterRegistrationHook }
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Newly published evidence file is missing: $fullPath"
    }
    $actualSha = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualSha -cne $ExpectedSha256) {
        throw "Newly published evidence file does not match its owned bytes: $fullPath"
    }
}

function Assert-Sprint8CImplementationEvidenceSnapshot {
    param(
        [Parameter(Mandatory)]$Index,
        [Parameter(Mandatory)]$FinalizationInputs
    )

    $null = Assert-TessaraPhaseEvidenceIndex -Index $Index -RepositoryRoot $repoRoot -AuditFiles
    foreach ($audit in @($FinalizationInputs.audits)) {
        $attemptRoot = [IO.Path]::GetFullPath([string]$audit.attempt_root)
        $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        foreach ($entry in @($audit.aggregate_files)) {
            $fullPath = [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$entry.path)))
            if ($fullPath.StartsWith(
                    $attemptRoot + [IO.Path]::DirectorySeparatorChar,
                    [StringComparison]::OrdinalIgnoreCase
                )) {
                $null = $expected.Add($fullPath)
            }
        }
        $actual = @(
            Get-ChildItem -LiteralPath $attemptRoot -File -Recurse | ForEach-Object {
                [IO.Path]::GetFullPath($_.FullName)
            }
        )
        if ($actual.Count -ne $expected.Count -or
            @($actual | Where-Object { -not $expected.Contains($_) }).Count -ne 0) {
            throw "Implementation target '$([string]$audit.id)' evidence inventory changed during aggregate sealing."
        }
    }
}

function Remove-Sprint8CCreatedEvidenceFilesExact {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()]
        [object[]]$CreatedFiles
    )

    $cleanupFailures = [Collections.Generic.List[string]]::new()
    for ($index = $CreatedFiles.Count - 1; $index -ge 0; $index--) {
        $created = $CreatedFiles[$index]
        $path = [string]$created.path
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
        $actualSha = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actualSha -cne [string]$created.sha256) {
            $cleanupFailures.Add("Refused to remove changed publication file '$path'.")
            continue
        }
        Remove-Item -LiteralPath $path -Force
    }
    if ($cleanupFailures.Count -ne 0) {
        throw (@($cleanupFailures) -join " ")
    }
}

function Assert-Sprint8CImplementationPublicationBoundaryPlacement {
    $inputSource = ${function:Get-Sprint8CImplementationFinalizationInputs}.ToString()
    $inputChronology = $inputSource.IndexOf(
        "Assert-TessaraDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    $targetAuthentication = $inputSource.IndexOf(
        'foreach ($contractTarget',
        [StringComparison]::Ordinal
    )
    if ($inputChronology -lt 0 -or $targetAuthentication -le $inputChronology) {
        throw "Implementation finalization does not authenticate chronology before target receipts."
    }

    $publisherSource = ${function:Publish-Sprint8CImplementationReadinessResult}.ToString()
    $firstPublication = $publisherSource.IndexOf(
        "Publish-Sprint8CNewJsonPair",
        [StringComparison]::Ordinal
    )
    $firstChronology = $publisherSource.IndexOf(
        "Assert-TessaraDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    $lastChronology = $publisherSource.LastIndexOf(
        "Assert-TessaraDefectProvenanceChronology",
        [StringComparison]::Ordinal
    )
    $lastPublication = $publisherSource.LastIndexOf(
        "Publish-Sprint8CNewJsonPair",
        [StringComparison]::Ordinal
    )
    $cleanup = $publisherSource.IndexOf(
        "Remove-Sprint8CCreatedEvidenceFilesExact",
        [StringComparison]::Ordinal
    )
    $lastEvidenceAudit = $publisherSource.LastIndexOf(
        "Assert-Sprint8CImplementationEvidenceSnapshot",
        [StringComparison]::Ordinal
    )
    $betweenPublications = if ($firstPublication -ge 0 -and
        $lastPublication -gt $firstPublication) {
        $publisherSource.Substring(
            $firstPublication,
            $lastPublication - $firstPublication
        )
    } else { "" }
    if ($firstChronology -lt 0 -or $firstPublication -le $firstChronology -or
        $betweenPublications.IndexOf(
            "Assert-TessaraDefectProvenanceChronology",
            [StringComparison]::Ordinal
        ) -lt 0 -or
        $lastChronology -le $lastPublication -or
        $lastEvidenceAudit -le $lastPublication -or $cleanup -le $lastChronology) {
        throw "Implementation aggregate chronology/publication/cleanup placement is not fail-closed."
    }
}

function Publish-Sprint8CImplementationReadinessResult {
    param(
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)]$FinalizationInputs,
        [scriptblock]$BeforeIndexPublicationHook,
        [scriptblock]$BeforeResultPublicationHook,
        [scriptblock]$AfterResultPublicationHook
    )

    $provenanceEvidenceRoot = [IO.Path]::GetFullPath(
        [string]$FinalizationInputs.defect_provenance_evidence_root
    )
    $null = Get-Sprint8CRepositoryRelativePath -Path $provenanceEvidenceRoot
    $null = Assert-TessaraDefectProvenanceChronology -RepositoryRoot $repoRoot `
        -EvidenceRoot $provenanceEvidenceRoot -Sprint "sprint-8c"
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
        sprint = "sprint-8c"
        phase = "implementation-readiness"
        attempt = 1
        evidence_root = Get-Sprint8CRepositoryRelativePath -Path $fullEvidenceRoot
        sealed_at = [DateTimeOffset]::UtcNow.ToString("O")
        entry_count = $entries.Count
        entries = $entries
    }
    Assert-Sprint8CImplementationEvidenceSnapshot -Index $index `
        -FinalizationInputs $FinalizationInputs
    $indexText = ConvertTo-Sprint8CJsonText -Document $index
    $indexSha = Get-Sprint8CTextSha256 -Text $indexText
    $contractSha = (Get-FileHash -LiteralPath $contractPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $result = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.implementation-readiness-result"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8c"
        state = "passed"
        authoritative = $false
        source_identity = $FinalizationInputs.source
        validation_contract = [pscustomobject][ordered]@{
            path = Get-Sprint8CRepositoryRelativePath -Path $contractPath
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
            path = Get-Sprint8CRepositoryRelativePath -Path $indexPath
            sha256 = $indexSha
        }
    }
    $null = Assert-TessaraImplementationReadinessResult -Result $result `
        -Contract $contract -ContractPath $contractPath

    $createdFiles = [Collections.Generic.List[object]]::new()
    try {
        if ($null -ne $BeforeIndexPublicationHook) { & $BeforeIndexPublicationHook }
        $null = Assert-TessaraDefectProvenanceChronology -RepositoryRoot $repoRoot `
            -EvidenceRoot $provenanceEvidenceRoot -Sprint "sprint-8c"
        Assert-Sprint8CImplementationEvidenceSnapshot -Index $index `
            -FinalizationInputs $FinalizationInputs
        $publishedIndexSha = Publish-Sprint8CNewJsonPair -Path $indexPath -Document $index
        Add-Sprint8CCreatedEvidenceFile -CreatedFiles $createdFiles -Path $indexPath `
            -ExpectedSha256 $publishedIndexSha
        Add-Sprint8CCreatedEvidenceFile -CreatedFiles $createdFiles -Path "$indexPath.sha256" `
            -ExpectedSha256 (Get-Sprint8CTextSha256 -Text "$publishedIndexSha`n")
        if ($publishedIndexSha -cne $indexSha) {
            throw "Published implementation evidence index changed during finalization."
        }
        if ($null -ne $BeforeResultPublicationHook) { & $BeforeResultPublicationHook }
        $null = Assert-TessaraDefectProvenanceChronology -RepositoryRoot $repoRoot `
            -EvidenceRoot $provenanceEvidenceRoot -Sprint "sprint-8c"
        Assert-Sprint8CImplementationEvidenceSnapshot -Index $index `
            -FinalizationInputs $FinalizationInputs
        $publishedResultSha = Publish-Sprint8CNewJsonPair -Path $resultPath -Document $result
        Add-Sprint8CCreatedEvidenceFile -CreatedFiles $createdFiles -Path $resultPath `
            -ExpectedSha256 $publishedResultSha
        Add-Sprint8CCreatedEvidenceFile -CreatedFiles $createdFiles -Path "$resultPath.sha256" `
            -ExpectedSha256 (Get-Sprint8CTextSha256 -Text "$publishedResultSha`n")
        if ($null -ne $AfterResultPublicationHook) { & $AfterResultPublicationHook }
        $null = Assert-TessaraDefectProvenanceChronology -RepositoryRoot $repoRoot `
            -EvidenceRoot $provenanceEvidenceRoot -Sprint "sprint-8c"
        Assert-Sprint8CImplementationEvidenceSnapshot -Index $index `
            -FinalizationInputs $FinalizationInputs
        $published = Read-Sprint8CAuthenticatedJsonFile -Path $resultPath `
            -RequiredRoot $fullEvidenceRoot
        $null = Assert-TessaraImplementationReadinessResult -Result $published.document `
            -Contract $contract -ContractPath $contractPath
        $published.document
    } catch {
        $publicationFailure = $_
        try {
            Remove-Sprint8CCreatedEvidenceFilesExact -CreatedFiles $createdFiles
        } catch {
            throw "Implementation aggregate publication failed: $($publicationFailure.Exception.Message) Exact cleanup also failed: $($_.Exception.Message)"
        }
        throw $publicationFailure
    }
}

function New-Sprint8CSyntheticRefreshEvidence {
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
        sprint = "sprint-8c"
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

function New-Sprint8CSyntheticMaterializationEvidence {
    param(
        [Parameter(Mandatory)][ValidateSet("Reference", "ReferenceNoOp")][string]$TargetName,
        [Parameter(Mandatory)]$Source
    )

    $digest = "sha256:$('a' * 64)"
    $owners = @(
        "core", "tessara.responses", "tessara.datasets", "tessara.components",
        "tessara.dashboards", "tessara.reference.scoped-records"
    )
    $firstReceipts = @($owners | ForEach-Object {
        [pscustomobject][ordered]@{
            owner = $_; input_digest = $digest; result_digest = $digest; changed = $true
            resource_ids = [pscustomobject][ordered]@{ logical_key = "typed-read-back" }
        }
    })
    $noOpReceipts = @($firstReceipts | ForEach-Object {
        [pscustomobject][ordered]@{
            owner = $_.owner; input_digest = $_.input_digest; result_digest = $_.result_digest
            changed = $false; resource_ids = $_.resource_ids
        }
    })
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = if ($TargetName -ceq "ReferenceNoOp") {
            "clean-owner-materialization-and-semantic-noop"
        } else { "clean-owner-materialization" }
        state = "passed"
        target = $TargetName
        compose_project = "tessara-s8c-finalizer-selftest"
        source = $Source
        environment_fingerprint_sha256 = "c" * 64
        first_apply = [pscustomobject][ordered]@{
            operation_state = "succeeded"; changed = $true; no_op = $false
            receipt_digest = $digest; owner_order = $owners; owner_receipts = $firstReceipts
        }
        semantic_noop = if ($TargetName -ceq "ReferenceNoOp") {
            [pscustomobject][ordered]@{
                operation_state = "succeeded"; changed = $false; no_op = $true
                previous_receipt_digest = $digest; owner_order = $owners
                owner_receipts = $noOpReceipts
            }
        } else { $null }
        semantic_noop_proof = if ($TargetName -ceq "ReferenceNoOp") {
            [pscustomobject][ordered]@{
                state = "passed"; stable_owner_receipts = $true
                stable_container_topology = $true
            }
        } else { $null }
        gateway_start_boundary = [pscustomobject][ordered]@{ post_start_health = "passed" }
        health = [pscustomobject][ordered]@{
            response_live = [pscustomobject][ordered]@{
                schema_version = 1
                status = "passing"
                checks = @()
            }
            response_ready = [pscustomobject][ordered]@{
                schema_version = 1
                status = "passing"
                checks = @(
                    @(
                        "response.database",
                        "response.configuration",
                        "response.security_state",
                        "response.provider.forms",
                        "response.provider.workflow",
                        "response.events.publication",
                        "response.export.publication"
                    ) | ForEach-Object {
                        [pscustomobject][ordered]@{
                            code = $_
                            passing = $true
                            message = "synthetic passing check"
                        }
                    }
                )
            }
        }
        fixture_receipt_path = "target/sprint-8c-finalizer-selftest/fixture.json"
        fixture_receipt_sha256 = "d" * 64
        cleanup_restoration = [pscustomobject][ordered]@{
            state = "passed"; mode = "exact-project-teardown"
        }
        failure = $null
    }
}

function New-Sprint8CSyntheticFailureEvidence {
    param([Parameter(Mandatory)]$Source)

    $attemptSpecifications = @(
        [pscustomobject][ordered]@{
            fault_key = "response.bootstrap.mid-apply"
            receipt_contract = "tessara.sprint-8c.failure-control/v1"
            target_service = "responses"
            phase = "response_bootstrap_transaction"
            expected_outcome = "rolled_back"
            expected_failure_code = "response.bootstrap.injected_failure"
            transaction_field = "response_transaction"
            transaction_value = "rolled_back"
        },
        [pscustomobject][ordered]@{
            fault_key = "response.incompatible"
            receipt_contract = "tessara.sprint-8b.failure-control/v1"
            target_service = "response-provider-proxy"
            phase = "dataset_bootstrap_provider_validation"
            expected_outcome = "rejected_pre_write"
            expected_failure_code = "dataset.dependency_incompatible"
            transaction_field = "dataset_transaction"
            transaction_value = "not_started"
        },
        [pscustomobject][ordered]@{
            fault_key = "dataset.derived-rebuild"
            receipt_contract = "tessara.sprint-8b.failure-control/v1"
            target_service = "datasets"
            phase = "dataset_bootstrap_transaction"
            expected_outcome = "rolled_back"
            expected_failure_code = "dataset.dependency_unavailable"
            transaction_field = "dataset_transaction"
            transaction_value = "rolled_back"
        }
    )
    $attemptIndex = 0
    $attempts = $attemptSpecifications | ForEach-Object {
        $attemptIndex++
        $materializationSha256 = ([string]$attemptIndex) * 64
        [pscustomobject][ordered]@{
            fault = [pscustomobject][ordered]@{
                fault_key = [string]$_.fault_key
                receipt_contract = [string]$_.receipt_contract
                target_service = [string]$_.target_service
                phase = [string]$_.phase
                expected_outcome = [string]$_.expected_outcome
                expected_failure_code = [string]$_.expected_failure_code
                attempt = 1
                attempt_limit = 1
                no_unauthorized_state = $true; no_cross_owner_write = $true
                transaction_field = [string]$_.transaction_field
                transaction_value = [string]$_.transaction_value
            }
            correlation_id = "01980000-00f0-7000-8000-{0:d12}" -f $attemptIndex
            materialization_evidence_sha256 = $materializationSha256
            materialization_evidence = [pscustomobject][ordered]@{
                path = "target/sprint-8c-finalizer-selftest/fault-$attemptIndex.json"
                sha256 = $materializationSha256
            }
            containment = [pscustomobject][ordered]@{
                fault_key = [string]$_.fault_key
                materialization_state = "failed"; gateway_started = $false
                fixture_published = $false; partial_topology_teardown = "passed"
            }
            child_exit_code = 1
        }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "deterministic-failure-containment-retry-and-restoration"
        state = "passed"
        compose_project = "tessara-s8c-finalizer-selftest"
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
            mode = "three-exact-partial-teardowns-plus-restored-successor-teardown"
            fault_controls = "cleared-to-none"; empty_successor_start = "passed"
        }
        failure = $null
    }
}

function New-Sprint8CSyntheticFocusedCleanEvidence {
    param(
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][string]$Proof,
        [Parameter(Mandatory)][int]$ExecutedTestCount
    )
    [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8c"; proof = $Proof; state = "passed"
        source = $Source; executed_test_count = $ExecutedTestCount
        cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
    }
}

function New-Sprint8CSyntheticResponseUpgradeEvidence {
    param([Parameter(Mandatory)]$Source)
    $sequence = @("0.9.0", "1.0.0", "0.9.0", "1.0.0")
    $stages = @(
        "establish-compatible-baseline", "upgrade-to-candidate",
        "rollback-to-compatible-baseline", "restore-intended-candidate"
    )
    [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8c"
        proof = "independent-response-upgrade-rollback-restoration"; state = "passed"
        source = $Source
        compose_project = "tessara-s8c-finalizer-selftest"
        environment_fingerprint_sha256 = "f" * 64
        release_contract = [pscustomobject][ordered]@{
            transition = [pscustomobject][ordered]@{
                owner = "tessara.responses"; intended_release = "1.0.0"
            }
            sequence = $sequence
            stages = $stages
        }
        release_fixture = [pscustomobject][ordered]@{
            baseline = [pscustomobject]@{
                version = "0.9.0"; runtime_image = "sha256:$('b' * 64)"
                executable_sha256 = "b" * 64
            }
            candidate = [pscustomobject]@{
                version = "1.0.0"; runtime_image = "sha256:$('a' * 64)"
                executable_sha256 = "a" * 64
            }
        }
        pre_exercise_snapshot = [pscustomobject][ordered]@{
            stage = "pre-exercise-candidate"
            response = [pscustomobject]@{ release = "1.0.0" }
        }
        transitions = @(for ($index = 0; $index -lt $sequence.Count; $index++) {
            $imageToken = if ($sequence[$index] -ceq "0.9.0") { 'b' } else { 'a' }
            [pscustomobject][ordered]@{
                stage = $stages[$index]
                target_release = $sequence[$index]
                target_image = "sha256:$($imageToken * 64)"
                target_manifest_digest = "sha256:$('c' * 64)"
                plan_digest = "sha256:$('d' * 64)"
                lockfile_digest = "sha256:$('e' * 64)"
                receipt_lockfile_digest = "sha256:$('e' * 64)"
                fixed_owner_lockfile = "passed"
                exact_delta = [pscustomobject]@{
                    owner = "tessara.responses"; action_count = 5; exact = $true
                }
            }
        })
        stage_snapshots = @(for ($index = 0; $index -lt $sequence.Count; $index++) {
            [pscustomobject][ordered]@{
                stage = $stages[$index]
                response = [pscustomobject]@{ release = $sequence[$index] }
            }
        })
        preservation_proofs = @($stages | ForEach-Object {
            [pscustomobject][ordered]@{
                stage = $_; state = "passed"; response_state = "passed"
                module_instance_identity = "passed"; typed_resource_identity = "passed"
                navigation_identity = "passed"; outbox_positions = "passed"
                unrelated_owners = "passed"
            }
        })
        release_restoration = [pscustomobject][ordered]@{
            state = "passed"; final_release = "1.0.0"
        }
        cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
        failure = $null
    }
}

function New-Sprint8CSyntheticResponseBrowserEvidence {
    param([Parameter(Mandatory)]$Source)

    $baseline = @(
        [pscustomobject]@{ key = "response-directory"; name = "response-directory-light-1440x1000.png"; sha256 = "bd5f99fbddca32ca4970069748e9f1856c5a3a364cfa9933855590684a46cf1d"; size = 118940; width = 1440; height = 1000 },
        [pscustomobject]@{ key = "response-start"; name = "response-start-dark-390x844.png"; sha256 = "3d9002307b44facb306595b5e1483b20fe1c0a94957789c16cc02e8a25090273"; size = 30129; width = 390; height = 844 },
        [pscustomobject]@{ key = "response-draft-detail"; name = "response-draft-detail-light-1024x1366.png"; sha256 = "0611c3a6994cade1bc0dc272958a6e59e3aeb8c6f14781da6890a21ad050c863"; size = 65960; width = 1024; height = 1366 },
        [pscustomobject]@{ key = "response-draft-edit"; name = "response-draft-edit-dark-1440x1000.png"; sha256 = "7d279b6f73de84c9ed126a84188e6b32948f8fcde25c4bcc74ce418b2d4336b5"; size = 67625; width = 1440; height = 1000 },
        [pscustomobject]@{ key = "response-submitted-detail"; name = "response-submitted-detail-light-390x844.png"; sha256 = "a9475a2e0ebb3994b74a444f118e09ee17df7c61c09123ae566d2281e9e1b5c3"; size = 32058; width = 390; height = 844 },
        [pscustomobject]@{ key = "operations-response-status"; name = "operations-response-status-dark-1024x1366.png"; sha256 = "c80e6a60085d81d4237e3dbe25945c97a83aef5e969c6f50a54878e47ddb9618"; size = 131036; width = 1024; height = 1366 },
        [pscustomobject]@{ key = "module-management-response"; name = "module-management-response-light-1440x1000.png"; sha256 = "deea6d795117506d277e838a983826825cfb4de872133a8b682b7d631f6cfbd8"; size = 108466; width = 1440; height = 1000 },
        [pscustomobject]@{ key = "response-directory-javascript-disabled"; name = "response-directory-javascript-disabled-1024x1366.png"; sha256 = "7619f0a9f3c41637d6c820376c6a4b374d42f68e5104619abcb8f32b0582e1d7"; size = 32915; width = 1024; height = 1366 }
    )
    $requiredMatrix = [pscustomobject][ordered]@{
        routes = @(
            "/responses", "/responses/new", "/responses/{response_id}",
            "/responses/{response_id}/edit", "/operations",
            "/administration/modules/tessara.responses"
        )
        roles = @("owner", "delegate", "manager", "restricted", "administrator")
        states = @(
            "populated", "empty", "loading", "draft", "submitted", "delegated",
            "restricted", "provider_degraded", "validation_error", "unsaved_dirty"
        )
        themes = @("light", "dark", "stored_theme", "system_theme")
        viewports = @("1440x1000", "1024x1366", "390x844", "200_percent_zoom")
        runtime = @(
            "javascript_disabled_ssr", "direct_refresh", "hydrated",
            "lifecycle_navigation", "no_external_assets"
        )
    }
    $identities = @(
        "Sprint 8C independent Response module › direct documents use only Response-owned public browser routes",
        "Sprint 8C independent Response module › assignment-only start options reject retired Core start routes",
        "Sprint 8C independent Response module › lifecycle navigation preserves unsaved draft state when discard is declined",
        "Sprint 8C independent Response module › scoped review and module diagnostics remain explicit and nondisclosing",
        "canonical module UI visual baselines › Responses directory at 1440 px (light)",
        "canonical module UI visual baselines › Responses start at 390 px (dark)",
        "canonical module UI visual baselines › Responses draft detail at 1024 px (light)",
        "canonical module UI visual baselines › Responses draft editor at 1440 px (dark)",
        "canonical module UI visual baselines › Responses submitted detail at 390 px (light)",
        "canonical module UI visual baselines › Responses Module Management at 1440 px (light)"
    )
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "owned-reference-focused-response-browser-and-visual"
        state = "passed"
        compose_project = "tessara-s8c-finalizer-selftest"
        source = $Source
        environment_fingerprint_sha256 = "d" * 64
        environment = [pscustomobject][ordered]@{ TESSARA_PLAYWRIGHT_DATA_STATE = "fresh" }
        accepted_baseline_preflight = [pscustomobject][ordered]@{
            state = "passed"
            accepted_case_count = 8
            keys = @($baseline.key)
            files = @($baseline | ForEach-Object {
                [pscustomobject][ordered]@{
                    key = $_.key; name = $_.name; sha256 = $_.sha256
                    size = $_.size; width = $_.width; height = $_.height
                }
            })
            required_matrix = $requiredMatrix
            source_commit = "8f6244e8df3e67a25ec537c671016544c1e05c19"
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
                    "--prefix", "end2end", "test", "--", "tests/responses-module.spec.ts",
                    "--update-snapshots=none"
                )
            },
            [pscustomobject][ordered]@{
                program = "npm"; exit_code = 0
                report_path = "target/selftest/visual.json"; report_sha256 = "d" * 64
                junit_path = "target/selftest/visual.xml"; junit_sha256 = "e" * 64
                log_path = "target/selftest/visual.log"; log_sha256 = "f" * 64
                arguments = @(
                    "--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Responses",
                    "--update-snapshots=none"
                )
            }
        )
        playwright = [pscustomobject][ordered]@{
            state = "passed"; data_state = "fresh"; expected = 10; passed = 10; skipped = 0
            workers = 1; retries = 0; update_snapshots = "none"; forbid_only = $true
            module = [pscustomobject][ordered]@{ passed = 4 }
            visual = [pscustomobject][ordered]@{ passed = 6 }
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

function Test-Sprint8CImplementationFinalizationContract {
    $selfTestRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot (
        "target/sprint-8c-finalizer-selftest-$([Guid]::NewGuid().ToString('N'))"
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
        branch = "codex/sprint-8c-selftest"
    }
    $provenanceEvidenceRoot = Join-Path $selfTestRoot "synthetic-provenance"
    try {
        [IO.Directory]::CreateDirectory($provenanceEvidenceRoot) | Out-Null
        foreach ($contractTarget in @($targets)) {
            $targetId = [string]$contractTarget.id
            $targetRoot = Join-Path $selfTestRoot "targets/$targetId"
            $attemptRoot = Join-Path $targetRoot "attempts/selftest-attempt"
            [IO.Directory]::CreateDirectory($attemptRoot) | Out-Null
            $commandLogPath = Join-Path $attemptRoot "command.log"
            Write-Sprint8CNewEvidenceFile -Path $commandLogPath -Text "synthetic $targetId transcript`n"

            $childDocuments = [ordered]@{}
            switch ($targetId) {
                "ui-sdk-conformance" {
                    $childDocuments["response-browser-proof.json"] =
                        New-Sprint8CSyntheticResponseBrowserEvidence -Source $source
                }
                "migration-seed" {
                    $childDocuments["migration-seed.json"] =
                        New-Sprint8CSyntheticMaterializationEvidence -TargetName Reference -Source $source
                }
                "clean-materialization" {
                    $childDocuments["clean-materialization.json"] =
                        New-Sprint8CSyntheticMaterializationEvidence -TargetName Reference -Source $source
                }
                "semantic-noop" {
                    $childDocuments["semantic-noop.json"] =
                        New-Sprint8CSyntheticMaterializationEvidence -TargetName ReferenceNoOp -Source $source
                }
                "failure-recovery" {
                    $childDocuments["failure-containment.json"] =
                        New-Sprint8CSyntheticFailureEvidence -Source $source
                }
                "workflow-events" {
                    $childDocuments["workflow-events.json"] =
                        New-Sprint8CSyntheticFocusedCleanEvidence -Source $source `
                            -Proof "workflow-response-event-consumption" -ExecutedTestCount 2
                }
                "dataset-export" {
                    $childDocuments["dataset-export.json"] =
                        New-Sprint8CSyntheticFocusedCleanEvidence -Source $source `
                            -Proof "response-owner-to-dataset-export-boundary" -ExecutedTestCount 20
                }
                "independent-upgrade-rollback" {
                    $childDocuments["response-upgrade.json"] =
                        New-Sprint8CSyntheticResponseUpgradeEvidence -Source $source
                }
            }
            foreach ($entry in $childDocuments.GetEnumerator()) {
                $null = Publish-Sprint8CNewJsonPair -Path (Join-Path $attemptRoot $entry.Key) `
                    -Document $entry.Value
            }

            $indexedFiles = @(Get-ChildItem -LiteralPath $attemptRoot -File | Sort-Object FullName)
            $targetIndex = [pscustomobject][ordered]@{
                schema_version = 1
                sprint = "sprint-8c"
                phase = "implementation-readiness"
                target = $targetId
                attempt_id = "selftest-attempt"
                evidence = @($indexedFiles | ForEach-Object {
                    [pscustomobject][ordered]@{
                        path = Get-Sprint8CRepositoryRelativePath -Path $_.FullName
                        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                        size = [long]$_.Length
                    }
                })
            }
            $targetIndexPath = Join-Path $attemptRoot "evidence-index.json"
            $targetIndexSha = Publish-Sprint8CNewJsonPair -Path $targetIndexPath -Document $targetIndex
            $targetResult = [pscustomobject][ordered]@{
                schema_version = 1
                sprint = "sprint-8c"
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
                    path = Get-Sprint8CRepositoryRelativePath -Path $commandLogPath
                    sha256 = (Get-FileHash -LiteralPath $commandLogPath -Algorithm SHA256).Hash.ToLowerInvariant()
                }
                evidence_index = [pscustomobject][ordered]@{
                    path = Get-Sprint8CRepositoryRelativePath -Path $targetIndexPath
                    sha256 = $targetIndexSha
                }
                failure = $null
            }
            $null = Publish-Sprint8CNewJsonPair -Path (Join-Path $targetRoot "result.json") `
                -Document $targetResult
        }

        $inputs = Get-Sprint8CImplementationFinalizationInputs `
            -EvidenceRootPath $selfTestRoot -ExpectedSource $source `
            -DefectProvenanceEvidenceRootPath $provenanceEvidenceRoot
        if (@($inputs.audits).Count -ne 23) {
            throw "Finalizer self-test did not authenticate all 23 synthetic targets."
        }
        if ([string]$inputs.defect_provenance_chronology.state -cne "passed") {
            throw "Finalizer self-test did not authenticate its synthetic provenance chronology."
        }

        $invalidProvenancePath = Join-Path $provenanceEvidenceRoot `
            "attempts/invalid/defect-provenance.json"
        Write-Sprint8CNewEvidenceFile -Path $invalidProvenancePath -Text "{ invalid chronology"
        $chronologyRejected = $false
        try {
            Get-Sprint8CImplementationFinalizationInputs `
                -EvidenceRootPath $selfTestRoot -ExpectedSource $source `
                -DefectProvenanceEvidenceRootPath $provenanceEvidenceRoot | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $chronologyRejected = $true
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force
        if (-not $chronologyRejected) {
            throw "Finalizer self-test admitted unresolved defect provenance."
        }

        $missingResult = Join-Path $selfTestRoot "targets/static-quality/result.json"
        $movedResult = "$missingResult.missing"
        Move-Item -LiteralPath $missingResult -Destination $movedResult
        $missingRejected = $false
        try {
            Get-Sprint8CImplementationFinalizationInputs `
                -EvidenceRootPath $selfTestRoot -ExpectedSource $source `
                -DefectProvenanceEvidenceRootPath $provenanceEvidenceRoot | Out-Null
        } catch { $missingRejected = $true }
        Move-Item -LiteralPath $movedResult -Destination $missingResult
        if (-not $missingRejected) {
            throw "Finalizer self-test admitted a missing required target receipt."
        }

        $dirtyRejected = $false
        try {
            $dirtySource = $source | ConvertTo-Json -Depth 10 | ConvertFrom-Json
            $dirtySource.dirty = $true
            Assert-Sprint8CExactSourceIdentity -Actual $dirtySource -Expected $source `
                -Label "synthetic dirty target"
        } catch { $dirtyRejected = $true }
        if (-not $dirtyRejected) { throw "Finalizer self-test admitted dirty target evidence." }

        $tamperedNoOp = New-Sprint8CSyntheticMaterializationEvidence `
            -TargetName ReferenceNoOp -Source $source
        $tamperedNoOp.semantic_noop.no_op = $false
        $noOpRejected = $false
        try {
            Assert-Sprint8CMaterializationEvidence -Document $tamperedNoOp `
                -ExpectedTarget ReferenceNoOp -ExpectedSource $source
        } catch { $noOpRejected = $true }
        if (-not $noOpRejected) { throw "Finalizer self-test admitted a mutating semantic no-op." }

        $legacyResponseHealth = New-Sprint8CSyntheticMaterializationEvidence `
            -TargetName Reference -Source $source
        $legacyResponseHealth.health.response_live = [pscustomobject][ordered]@{
            schema_version = 1
            module_definition_id = "tessara.responses"
            module_release_version = "1.0.0"
            status = "live"
        }
        $legacyResponseHealthRejected = $false
        try {
            Assert-Sprint8CMaterializationEvidence -Document $legacyResponseHealth `
                -ExpectedTarget Reference -ExpectedSource $source
        } catch { $legacyResponseHealthRejected = $true }
        if (-not $legacyResponseHealthRejected) {
            throw "Finalizer self-test admitted retired Dataset-shaped Response health evidence."
        }

        $duplicatedFailureProjection = New-Sprint8CSyntheticFailureEvidence -Source $source
        $duplicatedFailureProjection.failure_attempts[0].containment | Add-Member `
            -NotePropertyName containment -NotePropertyValue ([pscustomobject]@{
                attempt = 1; attempt_limit = 1
            })
        $duplicatedFailureProjectionRejected = $false
        try {
            Assert-Sprint8CFailureRecoveryEvidence -Document $duplicatedFailureProjection `
                -ExpectedSource $source
        } catch { $duplicatedFailureProjectionRejected = $true }
        if (-not $duplicatedFailureProjectionRejected) {
            throw "Finalizer self-test admitted a duplicate nested failure-containment projection."
        }

        $tamperPath = Join-Path $selfTestRoot "tamper.json"
        $null = Publish-Sprint8CNewJsonPair -Path $tamperPath -Document ([pscustomobject]@{ value = 1 })
        Add-Content -LiteralPath $tamperPath -Value " " -NoNewline
        $authenticationRejected = $false
        try { Read-Sprint8CAuthenticatedJsonFile -Path $tamperPath | Out-Null } catch {
            $authenticationRejected = $true
        }
        if (-not $authenticationRejected) {
            throw "Finalizer self-test admitted evidence changed after sidecar publication."
        }

        $foreignPairPath = Join-Path $selfTestRoot "foreign-publication.json"
        $foreignSidecarPath = "$foreignPairPath.sha256"
        Write-Sprint8CNewEvidenceFile -Path $foreignSidecarPath -Text "foreign-owned`n"
        $foreignSidecarSha = (
            Get-FileHash -LiteralPath $foreignSidecarPath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        $foreignPublicationRejected = $false
        try {
            Publish-Sprint8CNewJsonPair -Path $foreignPairPath `
                -Document ([pscustomobject]@{ state = "passed" }) | Out-Null
        } catch {
            $foreignPublicationRejected = $true
        }
        if (-not $foreignPublicationRejected -or
            (Test-Path -LiteralPath $foreignPairPath -PathType Leaf) -or
            -not (Test-Path -LiteralPath $foreignSidecarPath -PathType Leaf) -or
            (Get-FileHash -LiteralPath $foreignSidecarPath -Algorithm SHA256).Hash.ToLowerInvariant() `
                -cne $foreignSidecarSha) {
            throw "Finalizer self-test did not preserve a foreign publication member exactly."
        }
        Remove-Item -LiteralPath $foreignSidecarPath -Force

        $partialWritePath = Join-Path $selfTestRoot "partial-write.txt"
        $partialWriteRejected = $false
        try {
            Write-Sprint8CNewEvidenceFile -Path $partialWritePath -Text "owned`n" `
                -AfterCreateHook { throw "synthetic write failure" }
        } catch {
            if ($_.Exception.Message -notmatch 'synthetic write failure') { throw }
            $partialWriteRejected = $true
        }
        if (-not $partialWriteRejected -or
            (Test-Path -LiteralPath $partialWritePath -PathType Leaf)) {
            throw "Finalizer self-test left a partial newly-created evidence file."
        }

        $racingPairPath = Join-Path $selfTestRoot "sidecar-race.json"
        $racingSidecarPath = "$racingPairPath.sha256"
        $sidecarRaceHook = {
            [IO.File]::WriteAllText(
                $racingSidecarPath,
                "foreign-race`n",
                [Text.UTF8Encoding]::new($false)
            )
        }.GetNewClosure()
        $sidecarRaceRejected = $false
        try {
            Publish-Sprint8CNewJsonPair -Path $racingPairPath `
                -Document ([pscustomobject]@{ state = "passed" }) `
                -BeforeSidecarPublicationHook $sidecarRaceHook | Out-Null
        } catch { $sidecarRaceRejected = $true }
        if (-not $sidecarRaceRejected -or
            (Test-Path -LiteralPath $racingPairPath -PathType Leaf) -or
            (Get-Content -LiteralPath $racingSidecarPath -Raw) -cne "foreign-race`n") {
            throw "Finalizer self-test did not preserve a sidecar won by a concurrent writer."
        }
        Remove-Item -LiteralPath $racingSidecarPath -Force

        $registrationPath = Join-Path $selfTestRoot "registration-failure.txt"
        Write-Sprint8CNewEvidenceFile -Path $registrationPath -Text "owned-registration`n"
        $registrationSha = (
            Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        $registrationFiles = [Collections.Generic.List[object]]::new()
        $registrationRejected = $false
        try {
            Add-Sprint8CCreatedEvidenceFile -CreatedFiles $registrationFiles `
                -Path $registrationPath -ExpectedSha256 $registrationSha `
                -AfterRegistrationHook { throw "synthetic registration failure" }
        } catch {
            if ($_.Exception.Message -notmatch 'synthetic registration failure') { throw }
            $registrationRejected = $true
        }
        Remove-Sprint8CCreatedEvidenceFilesExact -CreatedFiles $registrationFiles
        if (-not $registrationRejected -or
            (Test-Path -LiteralPath $registrationPath -PathType Leaf)) {
            throw "Finalizer self-test did not retain ownership across registration failure."
        }

        $aggregatePaths = @(
            (Join-Path $selfTestRoot "evidence-index.json"),
            (Join-Path $selfTestRoot "evidence-index.json.sha256"),
            (Join-Path $selfTestRoot "implementation-readiness-result.json"),
            (Join-Path $selfTestRoot "implementation-readiness-result.json.sha256")
        )
        $insertInvalidProvenance = {
            [IO.File]::WriteAllText(
                $invalidProvenancePath,
                "{ invalid chronology",
                [Text.UTF8Encoding]::new($false)
            )
        }.GetNewClosure()

        $preIndexChronologyRejected = $false
        try {
            Publish-Sprint8CImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs `
                -BeforeIndexPublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $preIndexChronologyRejected = $true
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force
        if (-not $preIndexChronologyRejected -or
            @($aggregatePaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Finalizer self-test did not fail cleanly before aggregate index publication."
        }

        $postIndexChronologyRejected = $false
        try {
            Publish-Sprint8CImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs `
                -BeforeResultPublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $postIndexChronologyRejected = $true
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force
        if (-not $postIndexChronologyRejected -or
            @($aggregatePaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Finalizer self-test did not roll back its aggregate index after a later chronology failure."
        }

        $postResultChronologyRejected = $false
        try {
            Publish-Sprint8CImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs `
                -AfterResultPublicationHook $insertInvalidProvenance | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'Defect-provenance chronology is unresolved') {
                throw
            }
            $postResultChronologyRejected = $true
        }
        Remove-Item -LiteralPath $invalidProvenancePath -Force
        if (-not $postResultChronologyRejected -or
            @($aggregatePaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Finalizer self-test did not roll back a pass published during a chronology race."
        }

        $lateInventoryPath = Join-Path ([string]$inputs.audits[0].attempt_root) `
            "late-unindexed-evidence.txt"
        $insertLateEvidence = {
            [IO.File]::WriteAllText(
                $lateInventoryPath,
                "late evidence`n",
                [Text.UTF8Encoding]::new($false)
            )
        }.GetNewClosure()
        $preResultInventoryRejected = $false
        try {
            Publish-Sprint8CImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs `
                -BeforeResultPublicationHook $insertLateEvidence | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'evidence inventory changed') { throw }
            $preResultInventoryRejected = $true
        }
        Remove-Item -LiteralPath $lateInventoryPath -Force
        if (-not $preResultInventoryRejected -or
            @($aggregatePaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Finalizer self-test did not roll back an index after evidence inventory drift."
        }

        $postResultInventoryRejected = $false
        try {
            Publish-Sprint8CImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs `
                -AfterResultPublicationHook $insertLateEvidence | Out-Null
        } catch {
            if ($_.Exception.Message -notmatch 'evidence inventory changed') { throw }
            $postResultInventoryRejected = $true
        }
        Remove-Item -LiteralPath $lateInventoryPath -Force
        if (-not $postResultInventoryRejected -or
            @($aggregatePaths | Where-Object { Test-Path -LiteralPath $_ }).Count -ne 0) {
            throw "Finalizer self-test did not roll back a sealed pass after evidence inventory drift."
        }

        $published = Publish-Sprint8CImplementationReadinessResult `
            -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs
        if ([string]$published.state -cne "passed" -or @($published.targets).Count -ne 23) {
            throw "Finalizer self-test did not publish a valid aggregate readiness result."
        }
        $overwriteRejected = $false
        try {
            Publish-Sprint8CImplementationReadinessResult `
                -EvidenceRootPath $selfTestRoot -FinalizationInputs $inputs | Out-Null
        } catch { $overwriteRejected = $true }
        if (-not $overwriteRejected) {
            throw "Finalizer self-test overwrote an existing sealed aggregate."
        }

        [pscustomobject][ordered]@{
            state = "passed"
            target_count = 23
            missing_target_rejected = $missingRejected
            dirty_source_rejected = $dirtyRejected
            mutating_noop_rejected = $noOpRejected
            legacy_response_health_rejected = $legacyResponseHealthRejected
            duplicated_failure_projection_rejected = $duplicatedFailureProjectionRejected
            evidence_tamper_rejected = $authenticationRejected
            foreign_publication_preserved = $foreignPublicationRejected
            partial_write_cleanup_exact = $partialWriteRejected
            sidecar_create_race_preserved = $sidecarRaceRejected
            registration_failure_cleanup_exact = $registrationRejected
            unresolved_provenance_rejected = $chronologyRejected
            pre_index_provenance_insertion_rejected = $preIndexChronologyRejected
            post_index_provenance_insertion_rolled_back = $postIndexChronologyRejected
            post_result_provenance_insertion_rolled_back = $postResultChronologyRejected
            pre_result_inventory_drift_rolled_back = $preResultInventoryRejected
            post_result_inventory_drift_rolled_back = $postResultInventoryRejected
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
        Invoke-Sprint8CWorkspaceTestsWithDatabase
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
    Assert-Sprint8CWorkspaceTestDatabaseContractSelfTest
    $policySelfTest = Test-Sprint8CValidationPolicySelfTestContract
    $finalizationSelfTest = Test-Sprint8CImplementationFinalizationContract
    [pscustomobject]@{
        sprint = "sprint-8c"
        target_count = $targetIds.Count
        cargo_zero_test_guard = "passed"
        full_workspace_database_contract = "passed"
        validation_policy_selftest = $policySelfTest
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
    $null = Get-Sprint8CRepositoryRelativePath -Path $finalEvidenceRoot
    if (-not (Test-Path -LiteralPath $finalEvidenceRoot -PathType Container)) {
        throw "Implementation readiness evidence root is missing: $finalEvidenceRoot"
    }
    $source = Get-Sprint8CCurrentCleanSourceIdentity
    $inputs = Get-Sprint8CImplementationFinalizationInputs `
        -EvidenceRootPath $finalEvidenceRoot -ExpectedSource $source
    Publish-Sprint8CImplementationReadinessResult `
        -EvidenceRootPath $finalEvidenceRoot -FinalizationInputs $inputs | ConvertTo-Json -Depth 100
    exit 0
}
if ($targetIds -cnotcontains $Target) { throw "Unknown Sprint 8C implementation target '$Target'." }

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
            Invoke-Sprint8CWorkspaceTestsWithDatabase
            & (Join-Path $PSScriptRoot "validate-e2e.ps1") `
                -InventoryOnly `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "typescript-browser-inventory.json")
            if (-not $?) { throw "Affected TypeScript/Playwright graph discovery failed." }
        }
        "planning-contract-alignment" {
            & (Join-Path $PSScriptRoot "assert-sprint-8c-planning-contract.ps1")
            if (-not $?) { throw "Planning contract alignment failed." }
        }
        "contract-boundary" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-responses-contract", "-p", "tessara-workflows-contract",
                "-p", "tessara-forms-contract", "-p", "tessara-datasets-contract",
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
            & (Join-Path $PSScriptRoot "test-sprint-8c-response-module.ps1") -Suite Provider
            if (-not $?) { throw "Response owner/provider contract integration failed." }
            & (Join-Path $PSScriptRoot "check-sprint-8c-response-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Response package/source contract boundary validation failed." }
        }
        "owner-product" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-responses-contract", "-p", "tessara-response-ui",
                "--lib", "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8c-response-module.ps1") -Suite All
            if (-not $?) { throw "Complete Response owner integration suite failed." }
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
            & (Join-Path $PSScriptRoot "run-sprint-8c-response-browser-proof.ps1") `
                -ComposeProject "tessara-s8c-readiness-response-browser" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "response-browser-proof.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Focused Response browser/visual proof failed." }
        }
        "core-subtraction" {
            & (Join-Path $PSScriptRoot "check-sprint-8c-response-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Response Core-subtraction boundary validation failed." }
        }
        "inventory-navigation" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-response-module", "manifest_is_semantically_valid_and_declares_independent_ownership", "--lib",
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
            & (Join-Path $PSScriptRoot "sprint-8c-acceptance-contract.ps1") -SelfTest
            if (-not $?) { throw "Sprint 8C inventory/navigation contract failed." }
        }
        "migration-seed" {
            & (Join-Path $PSScriptRoot "materialize-sprint-8c.ps1") `
                -Target Reference `
                -ComposeProject "tessara-s8c-readiness-migration-seed" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "migration-seed.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8C complete owner-ordered migration/seed materialization failed." }
            & (Join-Path $PSScriptRoot "check-sprint-8c-response-boundaries.ps1") -Mode RequireClean
            if (-not $?) { throw "Sprint 8C migration/seed boundary validation failed." }
        }
        "clean-materialization" {
            & (Join-Path $PSScriptRoot "materialize-sprint-8c.ps1") `
                -Target Reference `
                -ComposeProject "tessara-s8c-readiness-clean-materialization" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "clean-materialization.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8C clean reference materialization failed." }
        }
        "semantic-noop" {
            & (Join-Path $PSScriptRoot "materialize-sprint-8c.ps1") `
                -Target ReferenceNoOp `
                -ComposeProject "tessara-s8c-readiness-semantic-noop" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "semantic-noop.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8C semantic no-op materialization failed." }
        }
        "failure-recovery" {
            & (Join-Path $PSScriptRoot "run-sprint-8c-failure-containment.ps1") `
                -ComposeProject "tessara-s8c-readiness-recovery" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "failure-containment.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8C deterministic failure containment/recovery failed." }
        }
        "consumer-cutover" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-responses-contract", "-p", "tessara-datasets-contract", "-p", "tessara-components-contract",
                "-p", "tessara-component-module", "-p", "tessara-dashboard-module", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8c-component-consumer.ps1")
            if (-not $?) { throw "Downstream Dataset/Component Response consumer integration failed." }
        }
        "workflow-events" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-workflows-contract", "-p", "tessara-responses-contract", "--locked", "--offline"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8c-workflow-events.ps1") `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "workflow-events.json")
            if (-not $?) { throw "Workflow consumption of Response owner events failed." }
        }
        "provider-contracts" {
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "workflow_response_provider", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            Invoke-CheckedCargoTest -Arguments @(
                "test", "-p", "tessara-api", "forms::tests", "--lib",
                "--locked", "--offline", "--jobs", "1"
            )
            & (Join-Path $PSScriptRoot "test-sprint-8c-response-module.ps1") -Suite Provider
            if (-not $?) { throw "Response reverse-provider contract integration failed." }
        }
        "dataset-export" {
            Invoke-CheckedCargoTest -Arguments @("test", "-p", "tessara-responses-contract", "-p", "tessara-datasets-contract", "--locked", "--offline")
            & (Join-Path $PSScriptRoot "test-sprint-8c-dataset-export-contract.ps1") `
                -EvidenceRoot (Join-Path $script:TargetAttemptRoot "dataset-export-contract") `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "dataset-export.json")
            if (-not $?) { throw "Response-owned Dataset export contract failed." }
        }
        "api-idempotency" {
            & (Join-Path $PSScriptRoot "test-sprint-8c-response-module.ps1") -Suite Owner
            if (-not $?) { throw "Response owner mutation replay integration failed." }
        }
        "assignment-only" {
            & (Join-Path $PSScriptRoot "test-sprint-8c-response-module.ps1") -Suite Owner
            if (-not $?) { throw "Response assignment-only start contract failed." }
        }
        "scoped-review" {
            & (Join-Path $PSScriptRoot "test-sprint-8c-response-module.ps1") -Suite Owner
            if (-not $?) { throw "Response scoped review and nondisclosure contract failed." }
        }
        "fixture-acceptance" {
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "prepare-sprint-8c-uat-fixtures.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "fixture-preparation-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "sprint-8c-acceptance-contract.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "acceptance-contract-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "uat-sprint-8c.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "uat-predicate-selftest.json") | Out-Null
            & (Join-Path $PSScriptRoot "validate-e2e.ps1") `
                -InventoryOnly `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "browser-inventory.json")
            if (-not $?) { throw "Sprint 8C exact browser acceptance discovery failed." }
        }
        "deployed-smoke" {
            & (Join-Path $PSScriptRoot "run-sprint-8c-deployed-smoke.ps1") `
                -ComposeProject "tessara-s8c-readiness-smoke" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "deployed-smoke.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8C deployed acceptance smoke failed." }
        }
        "independent-upgrade-rollback" {
            & (Join-Path $PSScriptRoot "run-sprint-8c-response-upgrade.ps1") `
                -ComposeProject "tessara-s8c-readiness-upgrade" `
                -EvidencePath (Join-Path $script:TargetAttemptRoot "response-upgrade.json") `
                -AuthorizeDisposableReset
            if (-not $?) { throw "Sprint 8C independent Response upgrade/rollback failed." }
        }
        "uat-readiness" {
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "prepare-sprint-8c-uat-fixtures.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "fixture-preparation-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "uat-sprint-8c.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "uat-predicate-selftest-output.json") `
                -AdditionalArguments @(
                    "-EvidencePath", (Join-Path $script:TargetAttemptRoot "uat-predicate-readiness.json")
                ) | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "run-sprint-8c-formal-uat.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "formal-uat-wrapper-selftest.json") | Out-Null
            Invoke-CheckedPowerShellSelfTest `
                -ScriptName "sprint-8c-acceptance-contract.ps1" `
                -OutputPath (Join-Path $script:TargetAttemptRoot "acceptance-contract-selftest-output.json") `
                -AdditionalArguments @(
                    "-EvidencePath", (Join-Path $script:TargetAttemptRoot "acceptance-contract-readiness.json")
                ) | Out-Null
        }
        "runner-selftest" {
            Assert-RunnerContract
            Assert-CargoTestGuardSelfTest
            Assert-Sprint8CWorkspaceTestDatabaseContractSelfTest
            Invoke-CheckedValidationPolicySelfTest `
                -OutputPath (Join-Path $script:TargetAttemptRoot `
                    "validation-policy-selftest.json") | Out-Null
            Test-Sprint8CImplementationFinalizationContract | Out-Null
            foreach ($implementationHarness in @(
                "sprint-8c-harness-isolation.ps1",
                "materialize-sprint-8c.ps1",
                "prepare-sprint-8c-uat-fixtures.ps1",
                "sprint-8c-acceptance-contract.ps1",
                "run-sprint-8c-failure-containment.ps1",
                "run-sprint-8c-deployed-smoke.ps1",
                "run-sprint-8c-response-upgrade.ps1",
                "uat-sprint-8c.ps1",
                "run-sprint-8c-response-browser-proof.ps1"
            )) {
                $receiptName = ([IO.Path]::GetFileNameWithoutExtension($implementationHarness)) +
                    "-selftest.json"
                Invoke-CheckedPowerShellSelfTest -ScriptName $implementationHarness `
                    -OutputPath (Join-Path $script:TargetAttemptRoot $receiptName) | Out-Null
            }
            foreach ($formalWrapper in @(
                "validate-sprint-8c-readiness.ps1",
                "run-sprint-8c-candidate-rehearsal.ps1",
                "run-sprint-8c-validation-preflight.ps1",
                "run-sprint-8c-sit.ps1",
                "run-sprint-8c-formal-uat.ps1"
            )) {
                $receiptName = ([IO.Path]::GetFileNameWithoutExtension($formalWrapper)) + "-selftest.json"
                Invoke-CheckedPowerShellSelfTest -ScriptName $formalWrapper `
                    -OutputPath (Join-Path $script:TargetAttemptRoot $receiptName) | Out-Null
            }
        }
        default {
            throw "Target '$Target' is selectable but is not implemented by the current Sprint 8C slice."
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
    sprint = "sprint-8c"
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
    sprint = "sprint-8c"
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

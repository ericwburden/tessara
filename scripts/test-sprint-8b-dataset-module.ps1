[CmdletBinding()]
param(
    [ValidateSet("Product", "Provider", "Authoring", "Bootstrap", "Sync", "Refresh", "Dag", "All")]
    [string]$Suite = "All",
    [string]$EvidencePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$containerName = "tessara-s8b-dataset-tests-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
$databaseUrlWasPresent = Test-Path Env:DATABASE_URL
$databaseUrlBefore = $env:DATABASE_URL
. (Join-Path $PSScriptRoot "sprint-8b-cargo-test-integrity.ps1")
. (Join-Path $PSScriptRoot "sprint-8b-harness-isolation.ps1")

$script:RefreshTestIdentities = @(
    "unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state",
    "ordered_fixed_bound_pages_promote_each_response_change_once",
    "interrupted_page_attempt_retry_converges_once_from_published_cursor",
    "concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay",
    "expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement",
    "refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding",
    "derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure",
    "refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing"
)
$script:DagTestIdentities = @(
    "candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt",
    "rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact",
    "downstream_materialization_failure_rolls_back_every_rebuilt_table"
)

if ($EvidencePath -and $Suite -notin @("Refresh", "Dag")) {
    throw "-EvidencePath is supported only for the exact Refresh or Dag selector."
}

function Invoke-ExactDatasetTestBinary {
    param(
        [Parameter(Mandatory)][string]$TestBinary,
        [Parameter(Mandatory)][string[]]$ExpectedTestIdentities
    )

    $discoveryArguments = @(
        "test", "-p", "tessara-dataset-module", "--test", $TestBinary,
        "--locked", "--offline", "--jobs", "1", "--", "--list", "--format", "terse"
    )
    $discoveryLines = [Collections.Generic.List[string]]::new()
    & cargo @discoveryArguments 2>&1 | ForEach-Object {
        $line = [string]$_
        $discoveryLines.Add($line)
        Write-Host $line
    }
    $discoveryExitCode = $LASTEXITCODE
    if ($discoveryExitCode -ne 0) {
        throw "'cargo $($discoveryArguments -join ' ')' exited $discoveryExitCode."
    }
    $discovered = @(
        @($discoveryLines) | ForEach-Object {
            if ([string]$_ -cmatch '^(?<identity>[A-Za-z0-9_]+): test$') {
                $Matches['identity']
            }
        }
    )
    $expectedSet = @($ExpectedTestIdentities | Sort-Object -Unique)
    $discoveredSet = @($discovered | Sort-Object -Unique)
    if ($expectedSet.Count -ne $ExpectedTestIdentities.Count -or
        $discovered.Count -ne $ExpectedTestIdentities.Count -or
        $discoveredSet.Count -ne $discovered.Count -or
        ($discoveredSet -join "`n") -cne ($expectedSet -join "`n")) {
        throw "Dataset '$TestBinary' discovery was not set-equal to its exact test identity contract."
    }

    $arguments = @(
        "test", "-p", "tessara-dataset-module", "--test", $TestBinary,
        "--locked", "--offline", "--jobs", "1", "--", "--format", "terse"
    )
    $lines = [Collections.Generic.List[string]]::new()
    & cargo @arguments 2>&1 | ForEach-Object {
        $line = [string]$_
        $lines.Add($line)
        Write-Host $line
    }
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) {
        throw "'cargo $($arguments -join ' ')' exited $exitCode."
    }
    Assert-Sprint8BCargoTestTranscript -OutputLines @($lines) `
        -ExpectedExecutedTestCount $ExpectedTestIdentities.Count | Out-Null
    [pscustomobject][ordered]@{
        test_binary = $TestBinary
        arguments = @($arguments)
        expected_test_identities = @($ExpectedTestIdentities)
        executed_test_identities = @($discovered)
        executed_test_count = $discovered.Count
    }
}

$exactResult = $null
$cleanupSucceeded = $false

try {
    $existing = docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}"
    if ($existing) { throw "Disposable Dataset test container '$containerName' already exists." }
    docker run --name $containerName `
        -e POSTGRES_USER=tessara_test `
        -e POSTGRES_PASSWORD=tessara_test `
        -e POSTGRES_DB=tessara_test `
        -p "127.0.0.1::5432" -d postgres:16-alpine | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Could not start the Dataset test database." }

    $ready = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        docker exec $containerName pg_isready -U tessara_test -d tessara_test *> $null
        if ($LASTEXITCODE -eq 0) { $ready = $true; break }
        Start-Sleep -Seconds 1
    }
    if (-not $ready) { throw "Dataset test database did not become ready." }
    $portLine = docker port $containerName 5432/tcp
    if ($portLine -notmatch '127\.0\.0\.1:(\d+)$') {
        throw "Could not resolve the isolated Dataset test database port."
    }
    $env:DATABASE_URL = "postgres://tessara_test:tessara_test@127.0.0.1:$($Matches[1])/tessara_test"
    Push-Location $repoRoot
    try {
        if ($Suite -in @("Authoring", "All")) {
            Invoke-Sprint8BCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "--test", "authoring_integration",
                "--locked", "--offline", "--jobs", "1"
            ) | Out-Null
        }
        if ($Suite -in @("Product", "All")) {
            Invoke-Sprint8BCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "--test", "product_integration",
                "--locked", "--offline", "--jobs", "1"
            ) | Out-Null
        }
        if ($Suite -in @("Bootstrap", "All")) {
            Invoke-Sprint8BCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "--test", "bootstrap_integration",
                "--locked", "--offline", "--jobs", "1"
            ) | Out-Null
        }
        if ($Suite -in @("Provider", "All")) {
            Invoke-Sprint8BCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "--test", "provider_integration",
                "--locked", "--offline", "--jobs", "1"
            ) | Out-Null
        }
        if ($Suite -in @("Sync", "All")) {
            Invoke-Sprint8BCheckedCargoTest -Arguments @(
                "test", "-p", "tessara-dataset-module", "--test", "sync_integration",
                "--locked", "--offline", "--jobs", "1"
            ) | Out-Null
        }
        if ($Suite -in @("Refresh", "All")) {
            $refreshResult = Invoke-ExactDatasetTestBinary `
                -TestBinary "refresh_integration" `
                -ExpectedTestIdentities $script:RefreshTestIdentities
            if ($Suite -ceq "Refresh") { $exactResult = $refreshResult }
        }
        if ($Suite -in @("Dag", "All")) {
            $dagResult = Invoke-ExactDatasetTestBinary `
                -TestBinary "dependency_dag_integration" `
                -ExpectedTestIdentities $script:DagTestIdentities
            if ($Suite -ceq "Dag") { $exactResult = $dagResult }
        }
    } finally {
        Pop-Location
    }
} finally {
    if ($databaseUrlWasPresent) {
        $env:DATABASE_URL = $databaseUrlBefore
    } else {
        Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue
    }
    $resolved = docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}"
    if ($resolved -ceq $containerName) {
        docker rm -f $containerName | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Could not remove the Dataset test database." }
    }
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        throw "Dataset test database teardown was not exact."
    }
    if ((Test-Path Env:DATABASE_URL) -ne $databaseUrlWasPresent -or
        ($databaseUrlWasPresent -and $env:DATABASE_URL -cne $databaseUrlBefore)) {
        throw "Dataset test database environment restoration was not exact."
    }
    $cleanupSucceeded = $true
}

if ($EvidencePath) {
    if (-not $cleanupSucceeded -or $null -eq $exactResult) {
        throw "Exact Dataset test evidence cannot publish before successful execution and cleanup."
    }
    Publish-Sprint8BHarnessEvidence -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "dataset-module-test-suite"
        state = "passed"
        suite = $Suite
        test_binary = [string]$exactResult.test_binary
        expected_test_identities = @($exactResult.expected_test_identities)
        executed_test_identities = @($exactResult.executed_test_identities)
        executed_test_count = [int]$exactResult.executed_test_count
        database = [pscustomobject][ordered]@{
            mode = "disposable-postgres"
            cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
        }
        command = [pscustomobject][ordered]@{
            arguments = @($exactResult.arguments)
        }
    }) -OutputPath $EvidencePath | Out-Null
}

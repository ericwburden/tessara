[CmdletBinding()]
param([string]$EvidencePath)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$containerName = "tessara-s8c-workflow-events-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
$databaseUrlWasPresent = Test-Path Env:DATABASE_URL
$databaseUrlBefore = $env:DATABASE_URL
. (Join-Path $PSScriptRoot "sprint-8c-cargo-test-integrity.ps1")
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")
$contracts = @(
    [pscustomobject][ordered]@{
        label = "Core autonomous consumer"
        arguments = @(
            "test", "-p", "tessara-api", "--lib", "workflow_response_consumer::tests::",
            "--locked", "--offline", "--jobs", "1"
        )
        identities = @(
            "workflow_response_consumer::tests::autonomous_consumer_recovers_owner_start_save_submit_backlog_while_unready",
            "workflow_response_consumer::tests::background_consumer_cancels_cleanly_without_losing_the_durable_cursor",
            "workflow_response_consumer::tests::projection_revision_policy_is_stale_safe_gap_intolerant_and_reconciliation_aware"
        )
    },
    [pscustomobject][ordered]@{
        label = "Response owner outbox"
        arguments = @(
            "test", "-p", "tessara-response-module", "--test", "owner_persistence",
            "--locked", "--offline", "--jobs", "1",
            "pinned_draft_saves_submits_and_exports_without_live_providers"
        )
        identities = @("pinned_draft_saves_submits_and_exports_without_live_providers")
    },
    [pscustomobject][ordered]@{
        label = "Response consumer ACK"
        arguments = @(
            "test", "-p", "tessara-response-module", "--lib",
            "--locked", "--offline", "--jobs", "1",
            "event_provider::tests::workflow_consumer_checkpoint_ack_is_monotonic_and_rejects_unpublished_heads"
        )
        identities = @("event_provider::tests::workflow_consumer_checkpoint_ack_is_monotonic_and_rejects_unpublished_heads")
    }
)
$expected = @($contracts | ForEach-Object { @($_.identities) })
$cleanupSucceeded = $false
$runs = [Collections.Generic.List[object]]::new()
try {
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        throw "Disposable Workflow event test container already exists."
    }
    docker run --name $containerName -e POSTGRES_USER=tessara_test `
        -e POSTGRES_PASSWORD=tessara_test -e POSTGRES_DB=tessara_test `
        -p "127.0.0.1::5432" -d postgres:16-alpine | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Could not start the Workflow event test database." }
    $ready = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        docker exec $containerName pg_isready -U tessara_test -d tessara_test *> $null
        if ($LASTEXITCODE -eq 0) { $ready = $true; break }
        Start-Sleep -Seconds 1
    }
    if (-not $ready) { throw "Workflow event test database did not become ready." }
    $portLine = docker port $containerName 5432/tcp
    if ($portLine -notmatch '127\.0\.0\.1:(\d+)$') {
        throw "Could not resolve the Workflow event test database port."
    }
    $env:DATABASE_URL = "postgres://tessara_test:tessara_test@127.0.0.1:$($Matches[1])/tessara_test"
    Push-Location $repoRoot
    try {
        foreach ($contract in $contracts) {
            $baseArguments = @($contract.arguments)
            $discoveryArguments = $baseArguments + @("--", "--list", "--format", "terse")
            $discovery = [Collections.Generic.List[string]]::new()
            & cargo @discoveryArguments 2>&1 | ForEach-Object {
                $line = [string]$_; $discovery.Add($line); Write-Host $line
            }
            if ($LASTEXITCODE -ne 0) {
                throw "Workflow event $($contract.label) test discovery failed."
            }
            $identities = @($discovery | ForEach-Object {
                if ([string]$_ -cmatch '^(?<identity>[A-Za-z0-9_:]+): test$') {
                    $Matches['identity']
                }
            })
            $contractExpected = @($contract.identities)
            if ((@($identities | Sort-Object) -join "`n") -cne
                    (@($contractExpected | Sort-Object) -join "`n") -or
                @($identities | Sort-Object -Unique).Count -ne $contractExpected.Count) {
                throw "Workflow event $($contract.label) discovery was not set-equal to the frozen identities."
            }
            $arguments = $baseArguments + @("--", "--format", "terse")
            $lines = [Collections.Generic.List[string]]::new()
            & cargo @arguments 2>&1 | ForEach-Object {
                $line = [string]$_; $lines.Add($line); Write-Host $line
            }
            if ($LASTEXITCODE -ne 0) {
                throw "Workflow event $($contract.label) tests failed."
            }
            Assert-Sprint8CCargoTestTranscript -OutputLines @($lines) `
                -ExpectedExecutedTestCount $contractExpected.Count | Out-Null
            $runs.Add([pscustomobject][ordered]@{
                label = [string]$contract.label
                arguments = @($arguments)
                expected_test_identities = @($contractExpected)
                executed_test_identities = @($identities)
                executed_test_count = $identities.Count
            })
        }
    } finally { Pop-Location }
} finally {
    if ($databaseUrlWasPresent) { $env:DATABASE_URL = $databaseUrlBefore }
    else { Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue }
    if ((docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") -ceq $containerName) {
        docker rm -f $containerName | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Could not remove the Workflow event test database." }
    }
    if (docker ps -a --filter "name=^/$containerName$" --format "{{.Names}}") {
        throw "Workflow event test database teardown was not exact."
    }
    if ((Test-Path Env:DATABASE_URL) -ne $databaseUrlWasPresent -or
        ($databaseUrlWasPresent -and $env:DATABASE_URL -cne $databaseUrlBefore)) {
        throw "Workflow event test environment restoration was not exact."
    }
    $cleanupSucceeded = $true
}
if ($EvidencePath) {
    if (-not $cleanupSucceeded) { throw "Workflow event evidence requires exact cleanup." }
    Publish-Sprint8CHarnessEvidence -Document ([pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8c"; proof = "workflow-response-event-consumption"
        state = "passed"; expected_test_identities = $expected
        executed_test_identities = @($runs | ForEach-Object { @($_.executed_test_identities) })
        executed_test_count = [int](($runs | Measure-Object -Property executed_test_count -Sum).Sum)
        source = Get-Sprint8CSourceIdentity
        database = [pscustomobject][ordered]@{
            mode = "disposable-postgres"
            cleanup_restoration = [pscustomobject][ordered]@{ state = "passed" }
        }
        runs = @($runs)
    }) -OutputPath $EvidencePath | Out-Null
}

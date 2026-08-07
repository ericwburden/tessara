[CmdletBinding()]
param(
    [ValidateSet("Run", "Finalize")][string]$Stage = "Run",
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$PreflightReceipt = "artifacts/sprint-8a-closeout/preflight-result.json",
    [string]$CandidateReceipt = "artifacts/sprint-8a-closeout/candidate.json",
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [string]$OutputPath = "artifacts/sprint-8a-closeout/sit-result.json",
    [string]$BaseUrl = "http://127.0.0.1:8088",
    [switch]$AuthorizeDisposableReset,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
if ($PSVersionTable.PSEdition -cne "Core" -or $PSVersionTable.PSVersion.Major -lt 7) {
    throw "The Sprint 8A SIT runner requires PowerShell 7 or newer."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-8a-lifecycle-chain.ps1")

$script:sitAllowedClassifications = @(
    "preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization"
)

function ConvertTo-Sprint8APowerShellLiteral {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Value)

    "'" + $Value.Replace("'", "''") + "'"
}

function Get-Sprint8ASitRelativePath {
    param([Parameter(Mandatory)][string]$Path)

    [IO.Path]::GetRelativePath($repoRoot, [IO.Path]::GetFullPath($Path)).Replace("\", "/")
}

function Get-Sprint8ASitEvidenceReference {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$RequireSidecar
    )

    $resolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Path $Path
    $sha = if ($RequireSidecar) {
        Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
    } else {
        if (-not (Test-Path -LiteralPath ([string]$resolved.full_path) -PathType Leaf)) {
            throw "Expected retained SIT evidence is missing: '$($resolved.path)'."
        }
        Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)
    }
    [pscustomobject][ordered]@{
        path = [string]$resolved.path
        sha256 = $sha
        full_path = [string]$resolved.full_path
    }
}

function Get-Sprint8ASitFileReferences {
    param([AllowEmptyCollection()][string[]]$Paths = @())

    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    @(
        foreach ($path in @($Paths)) {
            if ([string]::IsNullOrWhiteSpace($path)) { continue }
            $fullPath = [IO.Path]::GetFullPath($path)
            foreach ($candidate in @($fullPath, "$fullPath.sha256")) {
                if ((Test-Path -LiteralPath $candidate -PathType Leaf) -and $seen.Add($candidate)) {
                    $reference = Get-Sprint8ASitEvidenceReference -Path $candidate
                    [pscustomobject][ordered]@{
                        path = [string]$reference.path
                        sha256 = [string]$reference.sha256
                    }
                }
            }
        }
    )
}

function Publish-Sprint8ASitDocument {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$Path,
        [switch]$Overwrite
    )

    Publish-Sprint7AEvidence -Document $Document -OutputPath $Path -Overwrite:$Overwrite | Out-Null
    Get-Sprint8ASitEvidenceReference -Path $Path -RequireSidecar
}

function Set-Sprint8ASitAttemptState {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][ValidateSet("preparing", "executing", "harvesting", "finalizing", "passed", "failed")][string]$State,
        [Parameter(Mandatory)][string]$Stage,
        [switch]$FinalizationRetry
    )

    $current = "$([string]$Receipt.state)/$([string]$Receipt.stage)"
    $target = "$State/$Stage"
    $allowed = [ordered]@{
        "preparing/start" = @("preparing/prerequisites", "failed/prerequisites")
        "preparing/prerequisites" = @("executing/lanes", "failed/prerequisites")
        "executing/lanes" = @("harvesting/lanes", "finalizing/canonical-restoration", "failed/interrupted")
        "harvesting/lanes" = @("finalizing/canonical-restoration", "failed/interrupted")
        "finalizing/canonical-restoration" = @(
            "finalizing/publication", "failed/canonical-restoration", "failed/diagnostic-harvest-complete"
        )
        "finalizing/publication" = @("passed/complete", "failed/publication-failed")
    }
    $retryAllowed = $FinalizationRetry -and
        $current -ceq "failed/publication-failed" -and
        $target -ceq "finalizing/canonical-restoration"
    if (-not $retryAllowed -and
        (-not $allowed.Contains($current) -or $allowed[$current] -cnotcontains $target)) {
        throw "Illegal Sprint 8A SIT attempt transition '$current' -> '$target'."
    }
    $Receipt.state = $State
    $Receipt.stage = $Stage
    $Receipt.state_history = @($Receipt.state_history) + @([pscustomobject][ordered]@{
        state = $State
        stage = $Stage
        at = [DateTimeOffset]::UtcNow.ToString("o")
    })
}

function Set-Sprint8ASitLaneState {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][ValidateSet("preparing", "executing", "finalizing", "passed", "failed", "blocked")][string]$State,
        [Parameter(Mandatory)][string]$Stage
    )

    $current = "$([string]$Receipt.state)/$([string]$Receipt.stage)"
    $target = "$State/$Stage"
    $allowed = [ordered]@{
        "preparing/declared" = @("executing/checks", "blocked/prerequisite", "blocked/interrupted")
        "executing/checks" = @("finalizing/evidence", "failed/interrupted", "blocked/interrupted")
        "finalizing/evidence" = @(
            "passed/complete", "failed/harvest-complete", "failed/interrupted", "blocked/interrupted"
        )
    }
    if (-not $allowed.Contains($current) -or $allowed[$current] -cnotcontains $target) {
        throw "Illegal Sprint 8A SIT lane transition '$current' -> '$target'."
    }
    $Receipt.state = $State
    $Receipt.stage = $Stage
    $Receipt.state_history = @($Receipt.state_history) + @([pscustomobject][ordered]@{
        state = $State
        stage = $Stage
        at = [DateTimeOffset]::UtcNow.ToString("o")
    })
}

function Get-Sprint8ASitLaneContracts {
    @(
        [pscustomobject][ordered]@{
            name = "static-and-boundaries"
            depends_on = @("authenticated-prerequisites")
            topology_mutation_group = $null
        },
        [pscustomobject][ordered]@{
            name = "rust-workspace"
            depends_on = @("authenticated-prerequisites")
            topology_mutation_group = $null
        },
        [pscustomobject][ordered]@{
            name = "playwright"
            depends_on = @("authenticated-prerequisites")
            topology_mutation_group = "serialized-deployed-topology"
        },
        [pscustomobject][ordered]@{
            name = "deployed-acceptance-smoke"
            depends_on = @("authenticated-prerequisites")
            topology_mutation_group = "serialized-deployed-topology"
        }
    )
}

function Resolve-Sprint8ASitFailureClassification {
    param(
        [Parameter(Mandatory)][ValidateSet("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization")][string]$DefaultClassification,
        [AllowNull()]$ErrorRecord,
        [AllowEmptyCollection()][string[]]$EvidencePaths = @()
    )

    if ($null -ne $ErrorRecord -and
        $null -ne $ErrorRecord.Exception.Data["Sprint8AClassification"] -and
        $script:sitAllowedClassifications -ccontains [string]$ErrorRecord.Exception.Data["Sprint8AClassification"]) {
        return [pscustomobject][ordered]@{
            classification = [string]$ErrorRecord.Exception.Data["Sprint8AClassification"]
            source = "structured_exception"
        }
    }
    foreach ($path in @($EvidencePaths)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            [IO.Path]::GetExtension($path) -cne ".json") {
            continue
        }
        try {
            $document = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
            $candidates = [Collections.Generic.List[object]]::new()
            foreach ($container in @($document, $document.failure, $document.details, $document.result)) {
                if ($null -ne $container -and
                    $container.PSObject.Properties.Name -contains "classification") {
                    $candidates.Add($container.classification)
                }
            }
            foreach ($candidate in @($candidates)) {
                if ($script:sitAllowedClassifications -ccontains [string]$candidate) {
                    return [pscustomobject][ordered]@{
                        classification = [string]$candidate
                        source = "structured_evidence"
                    }
                }
            }
        } catch {
            # Malformed evidence remains retained; it cannot override the declared category.
        }
    }
    $message = if ($null -eq $ErrorRecord) { "" } else { [string]$ErrorRecord.Exception.Message }
    if ($message -match '(?i)(manifest|sidecar|sha-?256|stale evidence|publication|retained evidence)') {
        return [pscustomobject][ordered]@{
            classification = "evidence-finalization"
            source = "failure_detail"
        }
    }
    if ($message -match '(?i)(docker daemon|connection refused|timed out|tool .* unavailable|database probe|psql)') {
        return [pscustomobject][ordered]@{
            classification = "environment"
            source = "failure_detail"
        }
    }
    [pscustomobject][ordered]@{
        classification = $DefaultClassification
        source = "declared_check_category"
    }
}

function Get-Sprint8ASitAggregateClassification {
    param([Parameter(Mandatory)][object[]]$Failures)

    foreach ($classification in @("product", "harness", "environment", "preflight/setup", "evidence-finalization", "flaky")) {
        if (@($Failures | Where-Object { [string]$_.classification -ceq $classification }).Count -gt 0) {
            return $classification
        }
    }
    "harness"
}

function New-Sprint8ASitBlockedResult {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string]$Reason
    )

    [pscustomobject][ordered]@{
        name = $Name
        command = $Command
        state = "blocked"
        assertions_started = $false
        assertions_started_at = $null
        started_at = $null
        ended_at = $null
        duration_ms = 0L
        exit_status = $null
        classification = $null
        classification_source = $null
        failure_message = $null
        blocked_reason = $Reason
        evidence = @()
    }
}

function Invoke-Sprint8ASitProcessCheck {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$DisplayCommand,
        [Parameter(Mandatory)][string]$CommandBody,
        [Parameter(Mandatory)][string]$LogPath,
        [AllowEmptyCollection()][string[]]$EvidencePaths = @(),
        [Parameter(Mandatory)][ValidateSet("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization")][string]$DefaultClassification
    )

    [IO.Directory]::CreateDirectory((Split-Path -Parent $LogPath)) | Out-Null
    $started = [DateTimeOffset]::UtcNow
    [IO.File]::WriteAllText(
        $LogPath,
        "[$($started.ToString('o'))] assertion_started name=$Name`ncommand=$DisplayCommand`n",
        [Text.UTF8Encoding]::new($false)
    )
    $stdoutPath = "$LogPath.stdout.log"
    $stderrPath = "$LogPath.stderr.log"
    $process = $null
    $stdoutFile = $null
    $stderrFile = $null
    $stdoutCopy = $null
    $stderrCopy = $null
    $errorRecord = $null
    $failureMessage = $null
    $exitStatus = 1
    try {
        $shell = (Get-Process -Id $PID).Path
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = $shell
        $startInfo.WorkingDirectory = $repoRoot
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        foreach ($argument in @("-NoLogo", "-NoProfile", "-NonInteractive", "-Command", $CommandBody)) {
            $startInfo.ArgumentList.Add($argument)
        }
        $process = [Diagnostics.Process]::new()
        $process.StartInfo = $startInfo
        $stdoutFile = [IO.File]::Open(
            $stdoutPath, [IO.FileMode]::Create, [IO.FileAccess]::Write, [IO.FileShare]::Read
        )
        $stderrFile = [IO.File]::Open(
            $stderrPath, [IO.FileMode]::Create, [IO.FileAccess]::Write, [IO.FileShare]::Read
        )
        if (-not $process.Start()) { throw "Could not start the PowerShell child process." }
        $stdoutCopy = $process.StandardOutput.BaseStream.CopyToAsync($stdoutFile)
        $stderrCopy = $process.StandardError.BaseStream.CopyToAsync($stderrFile)
        while (-not $process.WaitForExit(30000)) {
            [IO.File]::AppendAllText(
                $LogPath,
                "[$([DateTimeOffset]::UtcNow.ToString('o'))] heartbeat name=$Name pid=$($process.Id)`n",
                [Text.UTF8Encoding]::new($false)
            )
        }
        $process.WaitForExit()
        $stdoutCopy.GetAwaiter().GetResult()
        $stdoutCopy = $null
        $stderrCopy.GetAwaiter().GetResult()
        $stderrCopy = $null
        $stdoutFile.Flush($true)
        $stderrFile.Flush($true)
        $exitStatus = [int]$process.ExitCode
        if ($exitStatus -ne 0) {
            throw "SIT check '$Name' exited with status $exitStatus."
        }
        $missing = @($EvidencePaths | Where-Object { -not (Test-Path -LiteralPath $_ -PathType Leaf) })
        if ($missing.Count -gt 0) {
            $exitStatus = 1
            throw "SIT check '$Name' did not produce: $($missing -join ', ')."
        }
    } catch {
        $errorRecord = $_
        $failureMessage = $_.Exception.Message
        if ($exitStatus -eq 0) { $exitStatus = 1 }
        [IO.File]::AppendAllText(
            $LogPath,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] failure`n$($_ | Out-String)`n",
            [Text.UTF8Encoding]::new($false)
        )
    } finally {
        if ($null -ne $process) {
            try {
                if (-not $process.HasExited) {
                    $process.Kill($true)
                    $process.WaitForExit()
                }
            } catch {
                [IO.File]::AppendAllText(
                    $LogPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] child_cleanup_failure=$($_.Exception.Message)`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        foreach ($copy in @($stdoutCopy, $stderrCopy)) {
            if ($null -eq $copy) { continue }
            try { $copy.GetAwaiter().GetResult() } catch {
                [IO.File]::AppendAllText(
                    $LogPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] raw_stream_copy_failure=$($_.Exception.Message)`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        if ($null -ne $stdoutFile) { $stdoutFile.Dispose() }
        if ($null -ne $stderrFile) { $stderrFile.Dispose() }
        if ($null -ne $process) { $process.Dispose() }
    }
    $ended = [DateTimeOffset]::UtcNow
    $passed = $null -eq $errorRecord
    $classification = if ($passed) {
        $null
    } else {
        Resolve-Sprint8ASitFailureClassification `
            -DefaultClassification $DefaultClassification `
            -ErrorRecord $errorRecord `
            -EvidencePaths $EvidencePaths
    }
    $evidence = Get-Sprint8ASitFileReferences `
        -Paths (@($LogPath, $stdoutPath, $stderrPath) + @($EvidencePaths))
    [pscustomobject][ordered]@{
        name = $Name
        command = $DisplayCommand
        state = if ($passed) { "passed" } else { "failed" }
        assertions_started = $true
        assertions_started_at = $started.ToString("o")
        started_at = $started.ToString("o")
        ended_at = $ended.ToString("o")
        duration_ms = [long][Math]::Max(0, ($ended - $started).TotalMilliseconds)
        exit_status = if ($passed) { 0 } else { [Math]::Max(1, $exitStatus) }
        classification = if ($passed) { $null } else { [string]$classification.classification }
        classification_source = if ($passed) { $null } else { [string]$classification.source }
        failure_message = if ($passed) { $null } else { $failureMessage }
        blocked_reason = $null
        evidence = @($evidence | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
    }
}

function Invoke-Sprint8ASitDatabaseResetBinding {
    param(
        [Parameter(Mandatory)]$Binding,
        [AllowNull()][string]$PostgresContainerId
    )

    $sql = "DROP SCHEMA IF EXISTS analytics CASCADE; DROP SCHEMA IF EXISTS dataset_materialized CASCADE; DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public; SELECT current_database() || '|' || current_user;"
    if (-not [string]::IsNullOrWhiteSpace($PostgresContainerId)) {
        $inspectOutput = @(& docker inspect $PostgresContainerId 2>&1)
        if ($LASTEXITCODE -ne 0 -or $inspectOutput.Count -eq 0) {
            throw "TEST_POSTGRES_CLIENT_CONTAINER_ID does not identify an inspectable container."
        }
        $inspect = @($inspectOutput | ConvertFrom-Json)[0]
        if (-not [bool]$inspect.State.Running) {
            throw "The approved PostgreSQL reset container is not running."
        }
        $published = @(& docker port ([string]$inspect.Id) 5432/tcp 2>&1 | ForEach-Object { [string]$_ })
        if ($LASTEXITCODE -ne 0 -or -not ($published -match ":$($Binding.port)$")) {
            throw "The approved PostgreSQL reset container is not published on the approved loopback port."
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
        if ($LASTEXITCODE -ne 0) { throw "Database reset failed for $($Binding.name)." }
        $client = "docker_postgres"
    } else {
        if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
            throw "Database reset requires psql or TEST_POSTGRES_CLIENT_CONTAINER_ID."
        }
        $priorPassword = [Environment]::GetEnvironmentVariable("PGPASSWORD", "Process")
        $psqlExit = 1
        try {
            [Environment]::SetEnvironmentVariable("PGPASSWORD", [string]$Binding.password, "Process")
            $output = @(& psql -X -v ON_ERROR_STOP=1 -At `
                -h ([string]$Binding.host) -p ([string]$Binding.port) `
                -U ([string]$Binding.role) -d ([string]$Binding.database) -c $sql 2>&1 |
                ForEach-Object { [string]$_ })
            $psqlExit = $LASTEXITCODE
        } finally {
            [Environment]::SetEnvironmentVariable("PGPASSWORD", $priorPassword, "Process")
        }
        if ($psqlExit -ne 0) { throw "Database reset failed for $($Binding.name)." }
        $client = "local_psql"
    }
    $expected = "$($Binding.database)|$($Binding.role)"
    if (@($output | Where-Object { $_ -ceq $expected }).Count -ne 1) {
        throw "Database reset authenticated the wrong database or role for $($Binding.name)."
    }
    [pscustomobject][ordered]@{
        variable = [string]$Binding.name
        identity = [string]$Binding.identity
        database = [string]$Binding.database
        role = [string]$Binding.role
        client = $client
        reset = $true
    }
}

function Invoke-Sprint8ASitDatabaseResetCheck {
    param(
        [Parameter(Mandatory)]$Environment,
        [Parameter(Mandatory)][string]$LogPath,
        [Parameter(Mandatory)][string]$EvidencePath
    )

    [IO.Directory]::CreateDirectory((Split-Path -Parent $LogPath)) | Out-Null
    $started = [DateTimeOffset]::UtcNow
    [IO.File]::WriteAllText(
        $LogPath,
        "[$($started.ToString('o'))] assertion_started name=reset-six-databases`n",
        [Text.UTF8Encoding]::new($false)
    )
    $failure = $null
    $failureRecord = $null
    try {
        if (-not $AuthorizeDisposableReset -or -not [bool]$Environment.contract.reset_authorization_present) {
            throw "The Rust lane requires both -AuthorizeDisposableReset and the frozen destructive-reset acknowledgement."
        }
        $results = [Collections.Generic.List[object]]::new()
        $resetFailures = [Collections.Generic.List[object]]::new()
        $containerId = [Environment]::GetEnvironmentVariable("TEST_POSTGRES_CLIENT_CONTAINER_ID")
        foreach ($expected in @($Environment.contract.databases)) {
            try {
                $value = [Environment]::GetEnvironmentVariable([string]$expected.variable)
                if ([string]::IsNullOrWhiteSpace($value)) {
                    throw "Database variable '$($expected.variable)' disappeared after environment authentication."
                }
                $binding = ConvertTo-Sprint8ADatabaseBinding -Name ([string]$expected.variable) -Value $value
                if ([string]$binding.value_sha256 -cne [string]$expected.value_sha256 -or
                    [string]$binding.host -cne [string]$expected.host -or
                    [int]$binding.port -ne [int]$expected.port -or
                    [string]$binding.database -cne [string]$expected.database -or
                    [string]$binding.role -cne [string]$expected.role -or
                    [string]$binding.canonical_server -cne [string]$expected.canonical_server) {
                    throw "Database binding '$($expected.variable)' changed after environment authentication."
                }
                $results.Add((Invoke-Sprint8ASitDatabaseResetBinding `
                    -Binding $binding `
                    -PostgresContainerId $containerId))
                [IO.File]::AppendAllText(
                    $LogPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] reset=$($binding.name) identity=$($binding.identity)`n",
                    [Text.UTF8Encoding]::new($false)
                )
            } catch {
                $resetFailures.Add([pscustomobject][ordered]@{
                    variable = [string]$expected.variable
                    identity = "$([string]$expected.canonical_server)/$([string]$expected.database)"
                    state = "failed"
                    classification = "environment"
                    message = $_.Exception.Message
                })
                $results.Add([pscustomobject][ordered]@{
                    variable = [string]$expected.variable
                    identity = "$([string]$expected.canonical_server)/$([string]$expected.database)"
                    database = [string]$expected.database
                    role = [string]$expected.role
                    client = $null
                    reset = $false
                })
                [IO.File]::AppendAllText(
                    $LogPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] reset_failure=$($expected.variable) message=$($_.Exception.Message)`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        if ($results.Count -ne 6) { throw "The Rust lane did not attempt exactly six database identities." }
        $document = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8a"
            phase = "sit-database-reset"
            attempt = $Attempt
            authoritative = $false
            state = if ($resetFailures.Count -eq 0) { "passed" } else { "failed" }
            classification = if ($resetFailures.Count -eq 0) { $null } else { "environment" }
            environment_fingerprint = [string]$Environment.fingerprint
            started_at = $started.ToString("o")
            ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            reset_count = @($results | Where-Object reset -EQ $true).Count
            failure_count = $resetFailures.Count
            databases = @($results)
            failures = @($resetFailures)
        }
        Publish-Sprint7AEvidence -Document $document -OutputPath $EvidencePath | Out-Null
        if ($resetFailures.Count -gt 0) {
            $resetException = [InvalidOperationException]::new(
                "The Rust lane retained $($resetFailures.Count) database reset failure(s)."
            )
            $resetException.Data["Sprint8AClassification"] = "environment"
            throw $resetException
        }
    } catch {
        $failureRecord = $_
        $failure = $_.Exception.Message
        [IO.File]::AppendAllText(
            $LogPath,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] failure`n$($_ | Out-String)`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    $ended = [DateTimeOffset]::UtcNow
    $passed = $null -eq $failureRecord
    $classification = if ($passed) { $null } else {
        Resolve-Sprint8ASitFailureClassification `
            -DefaultClassification "environment" `
            -ErrorRecord $failureRecord `
            -EvidencePaths @($EvidencePath)
    }
    $evidence = Get-Sprint8ASitFileReferences -Paths @($LogPath, $EvidencePath)
    [pscustomobject][ordered]@{
        name = "reset-six-databases"
        command = "reset the six exact preflight-approved disposable database identities"
        state = if ($passed) { "passed" } else { "failed" }
        assertions_started = $true
        assertions_started_at = $started.ToString("o")
        started_at = $started.ToString("o")
        ended_at = $ended.ToString("o")
        duration_ms = [long][Math]::Max(0, ($ended - $started).TotalMilliseconds)
        exit_status = if ($passed) { 0 } else { 1 }
        classification = if ($passed) { $null } else { [string]$classification.classification }
        classification_source = if ($passed) { $null } else { [string]$classification.source }
        failure_message = if ($passed) { $null } else { $failure }
        blocked_reason = $null
        evidence = @($evidence | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
    }
}

function New-Sprint8ASitCommandBody {
    param([Parameter(Mandatory)][string]$Statement)

    @(
        '$ErrorActionPreference = "Stop"'
        'Set-StrictMode -Version Latest'
        "Set-Location -LiteralPath $(ConvertTo-Sprint8APowerShellLiteral -Value $repoRoot)"
        $Statement
    ) -join "`n"
}

function New-Sprint8ASitSpec {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Command,
        [AllowNull()][string]$Statement,
        [AllowEmptyCollection()][string[]]$DependsOn = @(),
        [AllowEmptyCollection()][string[]]$RequiresFiles = @(),
        [AllowEmptyCollection()][string[]]$EvidencePaths = @(),
        [ValidateSet("process", "database-reset")][string]$Kind = "process",
        [ValidateSet("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization")][string]$Classification = "product"
    )

    [pscustomobject][ordered]@{
        name = $Name
        command = $Command
        statement = $Statement
        depends_on = @($DependsOn)
        requires_files = @($RequiresFiles)
        evidence_paths = @($EvidencePaths)
        kind = $Kind
        classification = $Classification
    }
}

function Get-Sprint8ASitLaneSpecifications {
    param(
        [Parameter(Mandatory)][string]$Lane,
        [Parameter(Mandatory)]$Environment,
        [Parameter(Mandatory)][string]$LogRoot
    )

    $nativeGuard = '; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }'
    $scriptGuard = '; if (-not $?) { exit 1 }'
    switch ($Lane) {
        "static-and-boundaries" {
            @(
                New-Sprint8ASitSpec -Name "formatting" -Command "cargo fmt --all -- --check" `
                    -Statement "& cargo fmt --all -- --check$nativeGuard"
                New-Sprint8ASitSpec -Name "workspace-check" -Command "cargo check --workspace --all-features --locked --offline" `
                    -Statement "& cargo check --workspace --all-features --locked --offline$nativeGuard"
                New-Sprint8ASitSpec -Name "workspace-clippy" -Command "cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings" `
                    -Statement "& cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings$nativeGuard"
                New-Sprint8ASitSpec -Name "compose-config" -Command "docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config --quiet" `
                    -Statement "& docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config --quiet$nativeGuard"
                New-Sprint8ASitSpec -Name "acceptance-contract" `
                    -Command ". .\scripts\sprint-8a-acceptance-contract.ps1; Test-Sprint8AAcceptanceContract" `
                    -Statement '. .\scripts\sprint-8a-acceptance-contract.ps1; Test-Sprint8AAcceptanceContract'
                New-Sprint8ASitSpec -Name "web-crate-boundaries" -Command ".\scripts\check-web-crate-boundaries.ps1" `
                    -Statement "& .\scripts\check-web-crate-boundaries.ps1$scriptGuard"
                New-Sprint8ASitSpec -Name "module-sdk-boundaries" -Command ".\scripts\verify-module-sdk-boundaries.ps1" `
                    -Statement "& .\scripts\verify-module-sdk-boundaries.ps1$scriptGuard"
                New-Sprint8ASitSpec -Name "dashboard-source-boundaries" -Command ".\scripts\verify-sprint-6e-boundaries.ps1" `
                    -Statement "& .\scripts\verify-sprint-6e-boundaries.ps1$scriptGuard"
                New-Sprint8ASitSpec -Name "markdown-links" -Command ".\scripts\verify-markdown-links.ps1" `
                    -Statement "& .\scripts\verify-markdown-links.ps1$scriptGuard"
            )
        }
        "rust-workspace" {
            $resetEvidence = Join-Path $script:attemptRootPath "database-reset.json"
            @(
                New-Sprint8ASitSpec -Name "reset-six-databases" `
                    -Command "reset the six exact preflight-approved disposable database identities" `
                    -Kind "database-reset" -Classification "environment" -EvidencePaths @($resetEvidence)
                New-Sprint8ASitSpec -Name "workspace-tests" `
                    -Command "cargo test --workspace --all-features --locked --offline" `
                    -Statement "& cargo test --workspace --all-features --locked --offline$nativeGuard" `
                    -DependsOn @("reset-six-databases")
                New-Sprint8ASitSpec -Name "optimized-resource-reference-timing" `
                    -Command "cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture" `
                    -Statement "& cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture$nativeGuard" `
                    -DependsOn @("reset-six-databases")
                New-Sprint8ASitSpec -Name "components-contract-tests" `
                    -Command "cargo test --locked --offline -p tessara-components-contract" `
                    -Statement "& cargo test --locked --offline -p tessara-components-contract$nativeGuard" `
                    -DependsOn @("reset-six-databases")
                New-Sprint8ASitSpec -Name "dashboard-module-tests" `
                    -Command "cargo test --locked --offline -p tessara-dashboard-module" `
                    -Statement "& cargo test --locked --offline -p tessara-dashboard-module$nativeGuard" `
                    -DependsOn @("reset-six-databases")
                New-Sprint8ASitSpec -Name "component-module-tests" `
                    -Command "cargo test --locked --offline -p tessara-component-module" `
                    -Statement "& cargo test --locked --offline -p tessara-component-module$nativeGuard" `
                    -DependsOn @("reset-six-databases")
                New-Sprint8ASitSpec -Name "module-testkit-tests" `
                    -Command "cargo test --locked --offline -p tessara-module-testkit" `
                    -Statement "& cargo test --locked --offline -p tessara-module-testkit$nativeGuard" `
                    -DependsOn @("reset-six-databases")
            )
        }
        "playwright" {
            $rootRelative = "$script:evidenceRootRelative/sit/playwright-$Attempt"
            $rootPath = Join-Path $script:evidenceRootPath "sit/playwright-$Attempt"
            $materialization = Join-Path $rootPath "materialization/attempt-$Attempt/materialization-result.json"
            $deployment = Join-Path $rootPath "deployment.json"
            $playwright = Join-Path $rootPath "playwright.json"
            $failures = Join-Path $rootPath "failures"
            @(
                New-Sprint8ASitSpec -Name "source-exact-materialization-no-op" `
                    -Command ".\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot `"$rootRelative`" -EnvironmentFingerprint $($Environment.fingerprint) -AuthorizeDisposableReset -Confirm:`$false -VerifyNoOp" `
                    -Statement "& .\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot $(ConvertTo-Sprint8APowerShellLiteral $rootRelative) -EnvironmentFingerprint $($Environment.fingerprint) -AuthorizeDisposableReset -Confirm:`$false -VerifyNoOp$scriptGuard" `
                    -EvidencePaths @($materialization)
                New-Sprint8ASitSpec -Name "deployment-evidence" `
                    -Command ".\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath `"$rootRelative/deployment.json`"" `
                    -Statement "& .\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/deployment.json")$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -EvidencePaths @($deployment)
                New-Sprint8ASitSpec -Name "playwright-execution" `
                    -Command ".\scripts\validate-e2e.ps1 -BaseUrl `"$BaseUrl`" -DeploymentEvidencePath `"$rootRelative/deployment.json`" -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -EvidencePath `"$rootRelative/playwright.json`" -FailureEvidenceDirectory `"$rootRelative/failures`"" `
                    -Statement "& .\scripts\validate-e2e.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -DeploymentEvidencePath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/deployment.json") -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -EvidencePath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/playwright.json") -FailureEvidenceDirectory $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/failures")$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -RequiresFiles @($deployment) `
                    -EvidencePaths @($playwright)
            )
        }
        "deployed-acceptance-smoke" {
            $rootRelative = "$script:evidenceRootRelative/sit/attempt-$Attempt"
            $rootPath = Join-Path $script:evidenceRootPath "sit/attempt-$Attempt"
            $materialization = Join-Path $rootPath "materialization/attempt-$Attempt/materialization-result.json"
            $initialInventory = Join-Path $rootPath "initial-inventory.json"
            $initialDeployment = Join-Path $rootPath "initial-deployment.json"
            $initialSmoke = Join-Path $rootPath "initial-smoke.json"
            $componentUpgrade = Join-Path $rootPath "component-upgrade.json"
            $failureContainment = Join-Path $rootPath "failure-containment.json"
            $restoredInventory = Join-Path $rootPath "restored-inventory.json"
            $restoredDeployment = Join-Path $rootPath "restored-deployment.json"
            $restoredSmoke = Join-Path $rootPath "restored-smoke.json"
            $supervisor = [string]$Environment.contract.endpoints.supervisor
            @(
                New-Sprint8ASitSpec -Name "source-exact-materialization-no-op" `
                    -Command ".\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot `"$rootRelative`" -EnvironmentFingerprint $($Environment.fingerprint) -AuthorizeDisposableReset -Confirm:`$false -VerifyNoOp" `
                    -Statement "& .\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot $(ConvertTo-Sprint8APowerShellLiteral $rootRelative) -EnvironmentFingerprint $($Environment.fingerprint) -AuthorizeDisposableReset -Confirm:`$false -VerifyNoOp$scriptGuard" `
                    -EvidencePaths @($materialization)
                New-Sprint8ASitSpec -Name "initial-inventory" `
                    -Command ".\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl `"$BaseUrl`" -OutputPath `"$rootRelative/initial-inventory.json`"" `
                    -Statement "& .\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/initial-inventory.json")$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -EvidencePaths @($initialInventory)
                New-Sprint8ASitSpec -Name "initial-deployment-evidence" `
                    -Command ".\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath `"$rootRelative/initial-deployment.json`"" `
                    -Statement "& .\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/initial-deployment.json")$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -EvidencePaths @($initialDeployment)
                New-Sprint8ASitSpec -Name "initial-product-smoke" `
                    -Command ".\scripts\smoke-sprint-8a.ps1 -BaseUrl `"$BaseUrl`" -SupervisorUrl `"$supervisor`" -OutputPath `"$rootRelative/initial-smoke.json`"" `
                    -Statement "& .\scripts\smoke-sprint-8a.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -SupervisorUrl $(ConvertTo-Sprint8APowerShellLiteral $supervisor) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/initial-smoke.json")$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -EvidencePaths @($initialSmoke)
                New-Sprint8ASitSpec -Name "component-upgrade-rollback" `
                    -Command ".\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath `"$rootRelative/component-upgrade.json`"" `
                    -Statement "& .\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/component-upgrade.json")$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -EvidencePaths @($componentUpgrade)
                New-Sprint8ASitSpec -Name "failure-containment-successor-health" `
                    -Command ".\scripts\run-sprint-8a-failure-containment.ps1 -Attempt $Attempt -EvidenceRoot `"$rootRelative`" -EnvironmentFingerprint $($Environment.fingerprint) -OutputPath `"$rootRelative/failure-containment.json`" -AuthorizeDisposableReset -SkipBuild" `
                    -Statement "& .\scripts\run-sprint-8a-failure-containment.ps1 -Attempt $Attempt -EvidenceRoot $(ConvertTo-Sprint8APowerShellLiteral $rootRelative) -EnvironmentFingerprint $($Environment.fingerprint) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/failure-containment.json") -AuthorizeDisposableReset -SkipBuild$scriptGuard" `
                    -DependsOn @("source-exact-materialization-no-op") -EvidencePaths @($failureContainment)
                New-Sprint8ASitSpec -Name "restored-inventory" `
                    -Command ".\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl `"$BaseUrl`" -OutputPath `"$rootRelative/restored-inventory.json`"" `
                    -Statement "& .\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/restored-inventory.json")$scriptGuard" `
                    -DependsOn @("failure-containment-successor-health") -EvidencePaths @($restoredInventory)
                New-Sprint8ASitSpec -Name "restored-deployment-evidence" `
                    -Command ".\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath `"$rootRelative/restored-deployment.json`"" `
                    -Statement "& .\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/restored-deployment.json")$scriptGuard" `
                    -DependsOn @("failure-containment-successor-health") -EvidencePaths @($restoredDeployment)
                New-Sprint8ASitSpec -Name "restored-product-smoke" `
                    -Command ".\scripts\smoke-sprint-8a.ps1 -BaseUrl `"$BaseUrl`" -SupervisorUrl `"$supervisor`" -OutputPath `"$rootRelative/restored-smoke.json`"" `
                    -Statement "& .\scripts\smoke-sprint-8a.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -SupervisorUrl $(ConvertTo-Sprint8APowerShellLiteral $supervisor) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/restored-smoke.json")$scriptGuard" `
                    -DependsOn @("failure-containment-successor-health") -EvidencePaths @($restoredSmoke)
            )
        }
        default { throw "Unknown Sprint 8A SIT lane '$Lane'." }
    }
}

function Write-Sprint8ASitCheckpoint {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$Name
    )

    $path = Join-Path $script:checkpointRootPath "$Name.json"
    $reference = Publish-Sprint8ASitDocument -Document $Document -Path $path
    [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = [string]$reference.sha256 }
}

function Invoke-Sprint8ASitLane {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Environment,
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)][string]$AttemptPath,
        [Parameter(Mandatory)][string]$LanePath
    )

    $lane = Get-Content -LiteralPath $LanePath -Raw | ConvertFrom-Json
    Set-Sprint8ASitLaneState -Receipt $lane -State "executing" -Stage "checks"
    $lane.started_at = [DateTimeOffset]::UtcNow.ToString("o")
    $lane.source_identity = $AttemptReceipt.source_identity
    $lane.source_verification_state = "verified"
    $lane.environment_fingerprint = [string]$AttemptReceipt.environment_fingerprint
    $lane.normalized_deployment_configuration_sha256 = `
        [string]$AttemptReceipt.normalized_deployment_configuration_sha256
    $lane.candidate_fingerprint = [string]$AttemptReceipt.candidate_fingerprint
    Publish-Sprint8ASitDocument -Document $lane -Path $LanePath -Overwrite | Out-Null

    $laneLogRoot = Join-Path $script:logRootPath ([string]$Contract.name)
    [IO.Directory]::CreateDirectory($laneLogRoot) | Out-Null
    $specifications = @(Get-Sprint8ASitLaneSpecifications `
        -Lane ([string]$Contract.name) `
        -Environment $Environment `
        -LogRoot $laneLogRoot)
    $resultMap = [ordered]@{}
    $results = [Collections.Generic.List[object]]::new()
    $index = 0
    foreach ($specification in $specifications) {
        $index++
        $failedDependencies = @($specification.depends_on | Where-Object {
            -not $resultMap.Contains([string]$_) -or [string]$resultMap[[string]$_].state -cne "passed"
        })
        $missingFiles = @($specification.requires_files | Where-Object {
            -not (Test-Path -LiteralPath $_ -PathType Leaf)
        })
        $result = if ($failedDependencies.Count -gt 0) {
            New-Sprint8ASitBlockedResult `
                -Name ([string]$specification.name) `
                -Command ([string]$specification.command) `
                -Reason "blocked because prerequisite check(s) were not passed: $($failedDependencies -join ', ')"
        } elseif ($missingFiles.Count -gt 0) {
            New-Sprint8ASitBlockedResult `
                -Name ([string]$specification.name) `
                -Command ([string]$specification.command) `
                -Reason "blocked because prerequisite evidence was not produced: $($missingFiles -join ', ')"
        } elseif ([string]$specification.kind -ceq "database-reset") {
            Invoke-Sprint8ASitDatabaseResetCheck `
                -Environment $Environment `
                -LogPath (Join-Path $laneLogRoot ("{0:d2}-{1}.log" -f $index, $specification.name)) `
                -EvidencePath ([string]$specification.evidence_paths[0])
        } else {
            if ([string]$Contract.topology_mutation_group -ceq "serialized-deployed-topology") {
                $script:topologyMutationStarted = $true
            }
            Invoke-Sprint8ASitProcessCheck `
                -Name ([string]$specification.name) `
                -DisplayCommand ([string]$specification.command) `
                -CommandBody (New-Sprint8ASitCommandBody -Statement ([string]$specification.statement)) `
                -LogPath (Join-Path $laneLogRoot ("{0:d2}-{1}.log" -f $index, $specification.name)) `
                -EvidencePaths @($specification.evidence_paths) `
                -DefaultClassification ([string]$specification.classification)
        }
        $results.Add($result)
        $resultMap[[string]$result.name] = $result
        $lane.assertions_started = @($results | Where-Object assertions_started -EQ $true).Count -gt 0
        $lane.command_results = @($results)
        $lane.assertion_count = @($results | Where-Object assertions_started -EQ $true).Count
        $lane.failure_count = @($results | Where-Object state -CEQ "failed").Count
        $lane.blocked_count = @($results | Where-Object state -CEQ "blocked").Count
        Publish-Sprint8ASitDocument -Document $lane -Path $LanePath -Overwrite | Out-Null
        $checkpoint = Write-Sprint8ASitCheckpoint `
            -Document $lane `
            -Name ("lane-{0}-{1:d2}-{2}" -f $Contract.name, $index, $result.state)
        $lane.checkpoints = @($lane.checkpoints) + @($checkpoint)
        Publish-Sprint8ASitDocument -Document $lane -Path $LanePath -Overwrite | Out-Null
        if ([string]$result.state -ceq "failed" -and [string]$AttemptReceipt.state -ceq "executing") {
            Set-Sprint8ASitAttemptState -Receipt $AttemptReceipt -State "harvesting" -Stage "lanes"
            Publish-Sprint8ASitDocument -Document $AttemptReceipt -Path $AttemptPath -Overwrite | Out-Null
        }
    }

    Set-Sprint8ASitLaneState -Receipt $lane -State "finalizing" -Stage "evidence"
    $failed = @($results | Where-Object state -CEQ "failed")
    $blocked = @($results | Where-Object state -CEQ "blocked")
    $ended = [DateTimeOffset]::UtcNow
    $started = ConvertTo-Sprint8ADateTimeOffset `
        -Value $lane.started_at `
        -Label "SIT lane '$($Contract.name)' start"
    $laneEvidence = @($results | ForEach-Object { @($_.evidence) } | Sort-Object path -Unique)
    $summary = [pscustomobject][ordered]@{
        name = [string]$Contract.name
        state = if ($failed.Count -eq 0 -and $blocked.Count -eq 0) { "passed" } else { "failed" }
        assertions_started = @($results | Where-Object assertions_started -EQ $true).Count -gt 0
        assertions_started_at = @($results | Where-Object assertions_started -EQ $true | Select-Object -First 1).assertions_started_at
        command = (@($specifications | ForEach-Object { [string]$_.command }) -join "`n")
        exit_status = if ($failed.Count -eq 0 -and $blocked.Count -eq 0) { 0 } else { 1 }
        started_at = $started.ToString("o")
        ended_at = $ended.ToString("o")
        duration_ms = [long][Math]::Max(0, ($ended - $started).TotalMilliseconds)
        classification = if ($failed.Count -eq 0 -and $blocked.Count -eq 0) {
            $null
        } else {
            Get-Sprint8ASitAggregateClassification -Failures $failed
        }
        classification_source = if ($failed.Count -eq 0 -and $blocked.Count -eq 0) { $null } else { "nested_results" }
        failure_message = if ($failed.Count -eq 0 -and $blocked.Count -eq 0) {
            $null
        } else {
            "Lane retained $($failed.Count) failed and $($blocked.Count) blocked check(s)."
        }
        blocked_reason = $null
        evidence = @($laneEvidence)
    }
    if ([string]$summary.state -ceq "passed") {
        Set-Sprint8ASitLaneState -Receipt $lane -State "passed" -Stage "complete"
    } else {
        Set-Sprint8ASitLaneState -Receipt $lane -State "failed" -Stage "harvest-complete"
    }
    $lane.ended_at = $ended.ToString("o")
    $lane.duration_ms = [long][Math]::Max(0, ($ended - $started).TotalMilliseconds)
    $lane.command_results = @($results)
    $lane.summary = $summary
    $lane.evidence = @($laneEvidence)
    $reference = Publish-Sprint8ASitDocument -Document $lane -Path $LanePath -Overwrite
    [pscustomobject][ordered]@{
        summary = $summary
        reference = [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = [string]$reference.sha256 }
        receipt = $lane
    }
}

function Complete-Sprint8ASitRemainingLanes {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)][string]$Reason
    )

    $existingSummaries = [ordered]@{}
    foreach ($summary in @($AttemptReceipt.checks)) {
        $existingSummaries[[string]$summary.name] = $summary
    }
    $existingReferences = [ordered]@{}
    foreach ($reference in @($AttemptReceipt.lane_receipts)) {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $script:evidenceRootPath `
            -Path ([string]$reference.path)
        if ((Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)) -cne [string]$reference.sha256) {
            throw "Interrupted SIT attempt found a changed retained lane receipt '$($reference.path)'."
        }
        $receipt = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
        $summary = @($AttemptReceipt.checks | Where-Object name -CEQ ([string]$receipt.name))
        if ($summary.Count -ne 1 -or
            ($summary[0] | ConvertTo-Json -Depth 30 -Compress) -cne
                ($receipt.summary | ConvertTo-Json -Depth 30 -Compress)) {
            throw "Interrupted SIT attempt found a changed retained lane summary '$($receipt.name)'."
        }
        $existingReferences[[string]$receipt.name] = $reference
    }

    @(
        foreach ($contract in Get-Sprint8ASitLaneContracts) {
            $name = [string]$contract.name
            $lanePath = $script:lanePaths[$name]
            $lane = Get-Content -LiteralPath $lanePath -Raw | ConvertFrom-Json
            if ($existingSummaries.Contains($name) -and $existingReferences.Contains($name)) {
                [pscustomobject][ordered]@{
                    summary = $existingSummaries[$name]
                    reference = $existingReferences[$name]
                    receipt = $lane
                }
                continue
            }
            if ([string]$lane.state -in @("passed", "failed", "blocked") -and $null -ne $lane.summary) {
                $terminalReference = Get-Sprint8ASitEvidenceReference -Path $lanePath -RequireSidecar
                [pscustomobject][ordered]@{
                    summary = $lane.summary
                    reference = [pscustomobject][ordered]@{
                        path = [string]$terminalReference.path
                        sha256 = [string]$terminalReference.sha256
                    }
                    receipt = $lane
                }
                continue
            }

            $ended = [DateTimeOffset]::UtcNow
            $hasStarted = $null -ne $lane.started_at -and
                -not [string]::IsNullOrWhiteSpace([string]$lane.started_at)
            $startedValue = if ($hasStarted) {
                ConvertTo-Sprint8ADateTimeOffset `
                    -Value $lane.started_at `
                    -Label "interrupted SIT lane '$name' start"
            } else {
                [DateTimeOffset]::MinValue
            }
            $assertionsStarted = [bool]$lane.assertions_started
            $interruptionLog = Join-Path $script:logRootPath "$name/interruption.log"
            [IO.Directory]::CreateDirectory((Split-Path -Parent $interruptionLog)) | Out-Null
            [IO.File]::WriteAllText(
                $interruptionLog,
                "[$($ended.ToString('o'))] $Reason`n",
                [Text.UTF8Encoding]::new($false)
            )
            $interruptionEvidence = @(Get-Sprint8ASitFileReferences -Paths @($interruptionLog) | ForEach-Object {
                [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
            })
            $nestedEvidence = @($lane.command_results | ForEach-Object { @($_.evidence) })
            $evidence = @(@($nestedEvidence) + @($interruptionEvidence) | Sort-Object path -Unique)
            if ($assertionsStarted) {
                $interruptionResult = [pscustomobject][ordered]@{
                    name = "runner-interruption"
                    command = "retain and terminalize interrupted lane '$name'"
                    state = "failed"
                    assertions_started = $true
                    assertions_started_at = $ended.ToString("o")
                    started_at = $ended.ToString("o")
                    ended_at = $ended.ToString("o")
                    duration_ms = 0L
                    exit_status = 1
                    classification = "harness"
                    classification_source = "runner_interruption"
                    failure_message = $Reason
                    blocked_reason = $null
                    evidence = $interruptionEvidence
                }
                $lane.command_results = @($lane.command_results) + @($interruptionResult)
                Set-Sprint8ASitLaneState -Receipt $lane -State "failed" -Stage "interrupted"
            } else {
                Set-Sprint8ASitLaneState -Receipt $lane -State "blocked" -Stage "interrupted"
                $lane.blocked_reason = $Reason
            }
            $summary = [pscustomobject][ordered]@{
                name = $name
                state = [string]$lane.state
                assertions_started = $assertionsStarted
                assertions_started_at = if ($assertionsStarted) { $ended.ToString("o") } else { $null }
                command = if ($assertionsStarted) { "retain and terminalize interrupted lane '$name'" } else { $null }
                exit_status = if ($assertionsStarted) { 1 } else { $null }
                started_at = if ($assertionsStarted -and $hasStarted) { $startedValue.ToString("o") } else { $null }
                ended_at = if ($assertionsStarted) { $ended.ToString("o") } else { $null }
                duration_ms = if ($assertionsStarted -and $hasStarted) {
                    [long][Math]::Max(0, ($ended - $startedValue).TotalMilliseconds)
                } else { 0L }
                classification = if ($assertionsStarted) { "harness" } else { $null }
                classification_source = if ($assertionsStarted) { "runner_interruption" } else { $null }
                failure_message = if ($assertionsStarted) { $Reason } else { $null }
                blocked_reason = if ($assertionsStarted) { $null } else { $Reason }
                evidence = if ($assertionsStarted) { $evidence } else { @() }
            }
            $lane.ended_at = $ended.ToString("o")
            $lane.duration_ms = if ($hasStarted) {
                [long][Math]::Max(0, ($ended - $startedValue).TotalMilliseconds)
            } else { 0L }
            $lane.summary = $summary
            $lane.evidence = $evidence
            $lane.failure_count = @($lane.command_results | Where-Object state -CEQ "failed").Count
            $lane.blocked_count = @($lane.command_results | Where-Object state -CEQ "blocked").Count
            $reference = Publish-Sprint8ASitDocument -Document $lane -Path $lanePath -Overwrite
            [pscustomobject][ordered]@{
                summary = $summary
                reference = [pscustomobject][ordered]@{
                    path = [string]$reference.path
                    sha256 = [string]$reference.sha256
                }
                receipt = $lane
            }
        }
    )
}

function Assert-Sprint8ASitManifestBaseline {
    param(
        [Parameter(Mandatory)]$PreflightReference,
        [Parameter(Mandatory)]$CandidateReference,
        [Parameter(Mandatory)][string[]]$CurrentAttemptPaths
    )

    $manifestPath = Join-Path $script:evidenceRootPath "evidence-manifest.json"
    Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or
        [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
        throw "The frozen Sprint 8A evidence manifest is malformed."
    }
    $excluded = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($path in $CurrentAttemptPaths) {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $script:evidenceRootPath `
            -Path $path
        $excluded.Add([string]$resolved.path) | Out-Null
    }
    $actual = [ordered]@{}
    foreach ($entry in Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $script:evidenceRootPath `
            -IgnoreExistingManifest) {
        if (-not $excluded.Contains([string]$entry.path)) { $actual[[string]$entry.path] = $entry }
    }
    $manifestPaths = @($manifest.entries | ForEach-Object { [string]$_.path })
    if (@($manifestPaths | Sort-Object -Unique).Count -ne $manifestPaths.Count) {
        throw "The frozen Sprint 8A evidence manifest repeats a path."
    }
    foreach ($entry in @($manifest.entries)) {
        if ($excluded.Contains([string]$entry.path)) { continue }
        if (-not $actual.Contains([string]$entry.path) -or
            [string]$actual[[string]$entry.path].sha256 -cne [string]$entry.sha256) {
            throw "Frozen manifested evidence is missing or stale: '$($entry.path)'."
        }
        $actual.Remove([string]$entry.path)
    }
    if ($actual.Count -ne 0) {
        throw "The frozen evidence manifest omits retained pre-SIT file(s): $(@($actual.Keys) -join ', ')."
    }
    foreach ($binding in @(
        [pscustomobject]@{ reference = $PreflightReference; phase = "validation-preflight" },
        [pscustomobject]@{ reference = $CandidateReference; phase = "candidate-freeze" }
    )) {
        $matches = @($manifest.entries | Where-Object {
            [string]$_.path -ceq [string]$binding.reference.path -and
                [string]$_.sha256 -ceq [string]$binding.reference.sha256 -and
                [string]$_.phase -ceq [string]$binding.phase -and
                $_.authoritative -is [bool] -and [bool]$_.authoritative -and
                [string]$_.status -ceq "passed"
        })
        if ($matches.Count -ne 1) {
            throw "The frozen evidence manifest lacks the exact authoritative '$($binding.phase)' receipt."
        }
    }
    $manifest
}

function Update-Sprint8ASitEvidenceManifest {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)][object[]]$PrerequisiteReferences,
        [AllowNull()]$ResultReference
    )

    $overrides = [Collections.Generic.List[object]]::new()
    foreach ($binding in @(
        [pscustomobject]@{ reference = $PrerequisiteReferences[0]; phase = "validation-preflight" },
        [pscustomobject]@{ reference = $PrerequisiteReferences[1]; phase = "candidate-freeze" }
    )) {
        $overrides.Add([pscustomobject][ordered]@{
            path = [string]$binding.reference.path
            sha256 = [string]$binding.reference.sha256
            phase = [string]$binding.phase
            authoritative = $true
            status = "passed"
        })
    }
    $attemptReference = Get-Sprint8ASitEvidenceReference -Path $script:attemptPath -RequireSidecar
    $overrides.Add([pscustomobject][ordered]@{
        path = [string]$attemptReference.path
        sha256 = [string]$attemptReference.sha256
        phase = "sit-attempt"
        authoritative = $false
        status = [string]$AttemptReceipt.state
    })
    foreach ($laneReference in @($AttemptReceipt.lane_receipts)) {
        $lane = Get-Content -LiteralPath ([string](Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot -EvidenceRoot $script:evidenceRootPath -Path ([string]$laneReference.path)).full_path) -Raw | ConvertFrom-Json
        $overrides.Add([pscustomobject][ordered]@{
            path = [string]$laneReference.path
            sha256 = [string]$laneReference.sha256
            phase = "sit-lane"
            authoritative = $false
            status = [string]$lane.state
        })
    }
    if ($null -ne $ResultReference) {
        $resultSidecar = Get-Sprint8ASitEvidenceReference -Path "$($ResultReference.path).sha256"
        foreach ($reference in @($ResultReference, $resultSidecar)) {
            $overrides.Add([pscustomobject][ordered]@{
                path = [string]$reference.path
                sha256 = [string]$reference.sha256
                phase = "sit"
                authoritative = $true
                status = "passed"
            })
        }
    }
    $existingManifestPath = Join-Path $script:evidenceRootPath "evidence-manifest.json"
    Assert-Sprint8AReceiptSidecar -Path $existingManifestPath | Out-Null
    $existingManifest = Get-Content -LiteralPath $existingManifestPath -Raw | ConvertFrom-Json
    $entries = Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Overrides @($overrides)
    $authorizedReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
        -ExistingEntries @($existingManifest.entries) `
        -UpdatedEntries $entries)
    $manifestPath = "$script:evidenceRootRelative/evidence-manifest.json"
    $reference = Publish-Sprint8AEvidenceManifest `
        -Entries $entries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -OutputPath $manifestPath `
        -Merge `
        -AuthorizedReplacementPaths @($authorizedReplacementPaths)
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath | Out-Null
    $reference
}

function Get-Sprint8ASitAuthenticatedContext {
    param(
        [Parameter(Mandatory)]$PreflightReference,
        [Parameter(Mandatory)]$CandidateReference
    )

    $expectedPreflightPath = "$script:evidenceRootRelative/preflight-result.json"
    $expectedCandidatePath = "$script:evidenceRootRelative/candidate.json"
    if ([string]$PreflightReference.path -cne $expectedPreflightPath -or
        [string]$CandidateReference.path -cne $expectedCandidatePath) {
        throw "SIT requires the canonical preflight and candidate receipt paths."
    }
    $preflight = Assert-Sprint8ALifecyclePrerequisite `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Reference $PreflightReference `
        -ExpectedPhase "validation-preflight"
    $candidate = Assert-Sprint8ALifecyclePrerequisite `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Reference $CandidateReference `
        -ExpectedPhase "candidate-freeze"
    $candidateFingerprint = [string]$candidate.receipt.candidate_fingerprint
    $environmentFingerprint = [string]$candidate.receipt.environment_fingerprint
    if ($candidateFingerprint -notmatch '^[0-9a-f]{64}$' -or
        $environmentFingerprint -notmatch '^[0-9a-f]{64}$') {
        throw "The candidate receipt omits its exact candidate/environment identity."
    }
    if (-not $AuthorizeDisposableReset) {
        throw "Authoritative Sprint 8A SIT requires -AuthorizeDisposableReset."
    }
    $source = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $source -RequireClean | Out-Null
    if (-not (Test-Sprint8ASourceIdentityMatch -Expected $candidate.receipt.source_identity -Actual $source)) {
        throw "The current source differs from the frozen Sprint 8A candidate."
    }
    $environment = Get-Sprint8AEnvironmentContract `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot `
        -ProbeDatabases
    if ([string]$environment.fingerprint -cne $environmentFingerprint) {
        throw "The live environment differs from the frozen candidate environment."
    }
    $normalizedDeploymentConfigurationSha256 = `
        [string]$environment.contract.compose.normalized_config_sha256
    if ($normalizedDeploymentConfigurationSha256 -notmatch '^[0-9a-f]{64}$') {
        throw "The live environment omits its normalized deployment-configuration identity."
    }
    $candidateIdentity = Get-Sprint8ACandidateIdentity `
        -RepositoryRoot $repoRoot `
        -Source $source `
        -NormalizedDeploymentConfigurationSha256 $normalizedDeploymentConfigurationSha256
    if ([string]$candidateIdentity.fingerprint -cne $candidateFingerprint) {
        throw "The recomputed candidate fingerprint differs from the frozen candidate."
    }
    $requestedBase = $null
    $frozenBase = $null
    if (-not [Uri]::TryCreate($BaseUrl, [UriKind]::Absolute, [ref]$requestedBase) -or
        -not [Uri]::TryCreate([string]$environment.contract.endpoints.gateway, [UriKind]::Absolute, [ref]$frozenBase) -or
        -not $requestedBase.IsLoopback -or
        $requestedBase.AbsoluteUri.TrimEnd('/') -cne $frozenBase.AbsoluteUri.TrimEnd('/')) {
        throw "SIT -BaseUrl differs from the frozen loopback gateway endpoint."
    }
    Assert-Sprint8ALifecyclePrerequisiteSet `
        -Phase "sit" `
        -References @($PreflightReference, $CandidateReference) `
        -Source $source `
        -EnvironmentFingerprint $environmentFingerprint `
        -NormalizedDeploymentConfigurationSha256 $normalizedDeploymentConfigurationSha256 `
        -CandidateFingerprint $candidateFingerprint `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath | Out-Null
    [pscustomobject][ordered]@{
        preflight = $preflight
        candidate = $candidate
        source = $source
        candidate_fingerprint = $candidateFingerprint
        environment_fingerprint = $environmentFingerprint
        normalized_deployment_configuration_sha256 = $normalizedDeploymentConfigurationSha256
        environment = $environment
    }
}

function Invoke-Sprint8ASitCanonicalRestoration {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][int]$Generation
    )

    $rootRelative = "$script:evidenceRootRelative/sit/attempt-$Attempt/final-restoration-$Generation"
    $rootPath = Join-Path $script:evidenceRootPath "sit/attempt-$Attempt/final-restoration-$Generation"
    $logRoot = Join-Path $script:logRootPath "final-restoration-$Generation"
    [IO.Directory]::CreateDirectory($logRoot) | Out-Null
    $materialization = Join-Path $rootPath "materialization/attempt-$Attempt/materialization-result.json"
    $inventory = Join-Path $rootPath "inventory.json"
    $deployment = Join-Path $rootPath "deployment.json"
    $smoke = Join-Path $rootPath "smoke.json"
    $supervisor = [string]$Context.environment.contract.endpoints.supervisor
    $scriptGuard = '; if (-not $?) { exit 1 }'
    $specifications = @(
        New-Sprint8ASitSpec -Name "canonical-materialization" `
            -Command ".\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot `"$rootRelative`" -EnvironmentFingerprint $($Context.environment_fingerprint) -AuthorizeDisposableReset -Confirm:`$false -VerifyNoOp" `
            -Statement "& .\scripts\materialize-sprint-8a.ps1 -Attempt $Attempt -EvidenceRoot $(ConvertTo-Sprint8APowerShellLiteral $rootRelative) -EnvironmentFingerprint $($Context.environment_fingerprint) -AuthorizeDisposableReset -Confirm:`$false -VerifyNoOp$scriptGuard" `
            -EvidencePaths @($materialization)
        New-Sprint8ASitSpec -Name "canonical-inventory" `
            -Command ".\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl `"$BaseUrl`" -OutputPath `"$rootRelative/inventory.json`"" `
            -Statement "& .\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/inventory.json")$scriptGuard" `
            -DependsOn @("canonical-materialization") -EvidencePaths @($inventory)
        New-Sprint8ASitSpec -Name "canonical-deployment-evidence" `
            -Command ".\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath `"$rootRelative/deployment.json`"" `
            -Statement "& .\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/deployment.json")$scriptGuard" `
            -DependsOn @("canonical-materialization") -EvidencePaths @($deployment)
        New-Sprint8ASitSpec -Name "canonical-product-smoke" `
            -Command ".\scripts\smoke-sprint-8a.ps1 -BaseUrl `"$BaseUrl`" -SupervisorUrl `"$supervisor`" -OutputPath `"$rootRelative/smoke.json`"" `
            -Statement "& .\scripts\smoke-sprint-8a.ps1 -BaseUrl $(ConvertTo-Sprint8APowerShellLiteral $BaseUrl) -SupervisorUrl $(ConvertTo-Sprint8APowerShellLiteral $supervisor) -OutputPath $(ConvertTo-Sprint8APowerShellLiteral "$rootRelative/smoke.json")$scriptGuard" `
            -DependsOn @("canonical-materialization") -EvidencePaths @($smoke)
    )
    $results = [Collections.Generic.List[object]]::new()
    $map = [ordered]@{}
    $index = 0
    foreach ($specification in $specifications) {
        $index++
        $failedDependencies = @($specification.depends_on | Where-Object {
            -not $map.Contains([string]$_) -or [string]$map[[string]$_].state -cne "passed"
        })
        $result = if ($failedDependencies.Count -gt 0) {
            New-Sprint8ASitBlockedResult `
                -Name ([string]$specification.name) `
                -Command ([string]$specification.command) `
                -Reason "blocked because canonical restoration prerequisite(s) were not passed: $($failedDependencies -join ', ')"
        } else {
            Invoke-Sprint8ASitProcessCheck `
                -Name ([string]$specification.name) `
                -DisplayCommand ([string]$specification.command) `
                -CommandBody (New-Sprint8ASitCommandBody -Statement ([string]$specification.statement)) `
                -LogPath (Join-Path $logRoot ("{0:d2}-{1}.log" -f $index, $specification.name)) `
                -EvidencePaths @($specification.evidence_paths) `
                -DefaultClassification "product"
        }
        $results.Add($result)
        $map[[string]$result.name] = $result
    }
    $passed = @($results | Where-Object state -CNE "passed").Count -eq 0
    $evidence = @($results | ForEach-Object { @($_.evidence) } | Sort-Object path -Unique)
    [pscustomobject][ordered]@{
        required = $true
        result = if ($passed) { "canonical_topology_verified" } else { "canonical_topology_not_proven" }
        generation = $Generation
        checks = @($results)
        evidence = $evidence
        passed = $passed
    }
}

function Get-Sprint8ASitDefectBatch {
    param(
        [Parameter(Mandatory)][object[]]$LaneReceipts,
        [AllowNull()]$Restoration,
        [AllowEmptyCollection()][object[]]$AdditionalDefects = @()
    )

    $defects = [Collections.Generic.List[object]]::new()
    foreach ($lane in $LaneReceipts) {
        foreach ($failure in @($lane.receipt.command_results | Where-Object state -CEQ "failed")) {
            $defects.Add([pscustomobject][ordered]@{
                lane = [string]$lane.receipt.name
                check = [string]$failure.name
                classification = [string]$failure.classification
                classification_source = [string]$failure.classification_source
                message = [string]$failure.failure_message
                evidence = @($failure.evidence)
            })
        }
    }
    if ($null -ne $Restoration) {
        foreach ($failure in @($Restoration.checks | Where-Object state -CEQ "failed")) {
            $defects.Add([pscustomobject][ordered]@{
                lane = "canonical-restoration"
                check = [string]$failure.name
                classification = [string]$failure.classification
                classification_source = [string]$failure.classification_source
                message = [string]$failure.failure_message
                evidence = @($failure.evidence)
            })
        }
    }
    foreach ($defect in $AdditionalDefects) { $defects.Add($defect) }
    $blocked = [Collections.Generic.List[object]]::new()
    foreach ($lane in $LaneReceipts) {
        foreach ($check in @($lane.receipt.command_results | Where-Object state -CEQ "blocked")) {
            $blocked.Add([pscustomobject][ordered]@{
                lane = [string]$lane.receipt.name
                check = [string]$check.name
                reason = [string]$check.blocked_reason
            })
        }
        if ($null -ne $lane.receipt.summary -and [string]$lane.receipt.summary.state -ceq "blocked") {
            $blocked.Add([pscustomobject][ordered]@{
                lane = [string]$lane.receipt.name
                check = [string]$lane.receipt.name
                reason = [string]$lane.receipt.summary.blocked_reason
            })
        }
    }
    if ($null -ne $Restoration) {
        foreach ($check in @($Restoration.checks | Where-Object state -CEQ "blocked")) {
            $blocked.Add([pscustomobject][ordered]@{
                lane = "canonical-restoration"
                check = [string]$check.name
                reason = [string]$check.blocked_reason
            })
        }
    }
    [pscustomobject][ordered]@{
        defect_count = $defects.Count
        defects = @($defects)
        blocked_count = $blocked.Count
        blocked_checks = @($blocked)
        consolidated = $true
        harvest_complete = $true
    }
}

function Assert-Sprint8ASitRawResultsForFinalization {
    param([Parameter(Mandatory)]$AttemptReceipt)

    Assert-Sprint8AExactTerminalIdentities `
        -Results @($AttemptReceipt.checks) `
        -ExpectedNames (Get-Sprint8ASitLaneNames) `
        -Label "SIT finalization retry" | Out-Null
    if (@($AttemptReceipt.checks | Where-Object state -CNE "passed").Count -ne 0 -or
        $AttemptReceipt.raw_results_complete -isnot [bool] -or
        -not [bool]$AttemptReceipt.raw_results_complete -or
        @($AttemptReceipt.lane_receipts).Count -ne 4) {
        throw "SIT finalization retry requires four immutable passing raw lane receipts."
    }
    if ($null -eq $AttemptReceipt.raw_terminal_checkpoint -or
        [string]::IsNullOrWhiteSpace([string]$AttemptReceipt.raw_terminal_checkpoint.path) -or
        [string]$AttemptReceipt.raw_terminal_checkpoint.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "SIT finalization retry requires the immutable raw-terminal checkpoint."
    }
    $checkpointResolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Path ([string]$AttemptReceipt.raw_terminal_checkpoint.path)
    if ((Assert-Sprint8AReceiptSidecar -Path ([string]$checkpointResolved.full_path)) -cne
        [string]$AttemptReceipt.raw_terminal_checkpoint.sha256) {
        throw "SIT finalization retry found a changed raw-terminal checkpoint."
    }
    $checkpoint = Get-Content -LiteralPath ([string]$checkpointResolved.full_path) -Raw | ConvertFrom-Json
    if (($checkpoint.checks | ConvertTo-Json -Depth 30 -Compress) -cne
            ($AttemptReceipt.checks | ConvertTo-Json -Depth 30 -Compress) -or
        ($checkpoint.lane_receipts | ConvertTo-Json -Depth 30 -Compress) -cne
            ($AttemptReceipt.lane_receipts | ConvertTo-Json -Depth 30 -Compress) -or
        [string]$checkpoint.candidate_fingerprint -cne [string]$AttemptReceipt.candidate_fingerprint -or
        [string]$checkpoint.environment_fingerprint -cne [string]$AttemptReceipt.environment_fingerprint -or
        [string]$checkpoint.normalized_deployment_configuration_sha256 -cne
            [string]$AttemptReceipt.normalized_deployment_configuration_sha256) {
        throw "SIT finalization retry raw results diverge from their terminal checkpoint."
    }
    foreach ($reference in @($AttemptReceipt.lane_receipts)) {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $script:evidenceRootPath `
            -Path ([string]$reference.path)
        if ((Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)) -cne [string]$reference.sha256) {
            throw "SIT finalization retry found a changed lane receipt '$($reference.path)'."
        }
        $lane = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
        $summary = @($AttemptReceipt.checks | Where-Object name -CEQ ([string]$lane.name))
        if ($summary.Count -ne 1 -or
            ($summary[0] | ConvertTo-Json -Depth 30 -Compress) -cne
                ($lane.summary | ConvertTo-Json -Depth 30 -Compress)) {
            throw "SIT finalization retry lane summary diverges from '$($lane.name)'."
        }
        foreach ($evidence in @($lane.summary.evidence)) {
            $evidenceResolved = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $script:evidenceRootPath `
                -Path ([string]$evidence.path)
            if ((Get-Sprint8AFileSha256 -Path ([string]$evidenceResolved.full_path)) -cne
                [string]$evidence.sha256) {
                throw "SIT finalization retry found changed raw evidence '$($evidence.path)'."
            }
        }
    }
}

function Assert-Sprint8AExistingSitResult {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)]$PreflightReference,
        [Parameter(Mandatory)]$CandidateReference
    )

    $resultReference = Get-Sprint8ASitEvidenceReference -Path $OutputPath -RequireSidecar
    $result = Assert-Sprint8ALifecyclePrerequisite `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Reference $resultReference `
        -ExpectedPhase "sit" `
        -ExpectedCandidateFingerprint ([string]$Context.candidate_fingerprint) `
        -ExpectedEnvironmentFingerprint ([string]$Context.environment_fingerprint)
    if ([int]$result.receipt.attempt -ne $Attempt -or
        -not (Test-Sprint8ASourceIdentityMatch -Expected $Context.source -Actual $result.receipt.source_identity) -or
        ($result.receipt.checks | ConvertTo-Json -Depth 30 -Compress) -cne
            ($AttemptReceipt.checks | ConvertTo-Json -Depth 30 -Compress) -or
        @($result.receipt.prerequisite_receipts).Count -ne 2 -or
        -not (Test-Sprint8AReceiptReferenceMatch -References @($result.receipt.prerequisite_receipts) -ExpectedReference $PreflightReference) -or
        -not (Test-Sprint8AReceiptReferenceMatch -References @($result.receipt.prerequisite_receipts) -ExpectedReference $CandidateReference) -or
        [string]$result.receipt.cleanup_restoration.result -cne "canonical_topology_verified") {
        throw "Existing canonical SIT result is not the exact result committed by this raw attempt."
    }
    foreach ($evidence in @($result.receipt.cleanup_restoration.evidence)) {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repoRoot -EvidenceRoot $script:evidenceRootPath -Path ([string]$evidence.path)
        if ((Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)) -cne [string]$evidence.sha256) {
            throw "Existing canonical SIT result has stale restoration evidence '$($evidence.path)'."
        }
    }
    [pscustomobject][ordered]@{
        path = [string]$result.reference.path
        sha256 = [string]$result.reference.sha256
        receipt = $result.receipt
    }
}

function Publish-Sprint8ASitFinalizationFailure {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$ErrorRecord
    )

    $path = Join-Path $script:attemptRootPath ("evidence-finalization-{0}.json" -f [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
    $document = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "sit-evidence-finalization"
        attempt = $Attempt
        authoritative = $false
        state = "failed"
        source_identity = $AttemptReceipt.source_identity
        environment_fingerprint = [string]$AttemptReceipt.environment_fingerprint
        candidate_fingerprint = [string]$AttemptReceipt.candidate_fingerprint
        classification = "evidence-finalization"
        classification_source = "final_publication_boundary"
        occurred_at = [DateTimeOffset]::UtcNow.ToString("o")
        failure_message = $ErrorRecord.Exception.Message
        raw_results_complete = [bool]$AttemptReceipt.raw_results_complete
        sit_result = $AttemptReceipt.sit_result
    }
    $reference = Publish-Sprint8ASitDocument -Document $document -Path $path
    [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = [string]$reference.sha256 }
}

function Complete-Sprint8ASitPublication {
    param(
        [Parameter(Mandatory)]$AttemptReceipt,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)]$PreflightReference,
        [Parameter(Mandatory)]$CandidateReference,
        [Parameter(Mandatory)]$Restoration
    )

    Set-Sprint8ASitAttemptState -Receipt $AttemptReceipt -State "finalizing" -Stage "publication"
    $AttemptReceipt.cleanup_restoration = $Restoration
    Publish-Sprint8ASitDocument -Document $AttemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
    try {
        $resultReference = if (Test-Path -LiteralPath ([string](Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repoRoot -EvidenceRoot $script:evidenceRootPath -Path $OutputPath).full_path) -PathType Leaf) {
            Assert-Sprint8AExistingSitResult `
                -AttemptReceipt $AttemptReceipt `
                -Context $Context `
                -PreflightReference $PreflightReference `
                -CandidateReference $CandidateReference
        } else {
            Publish-Sprint8ALifecycleReceipt `
                -Phase "sit" `
                -Attempt $Attempt `
                -Source $Context.source `
                -EnvironmentFingerprint ([string]$Context.environment_fingerprint) `
                -NormalizedDeploymentConfigurationSha256 `
                    ([string]$Context.normalized_deployment_configuration_sha256) `
                -CandidateFingerprint ([string]$Context.candidate_fingerprint) `
                -PrerequisiteReceipts @($PreflightReference, $CandidateReference) `
                -Checks @($AttemptReceipt.checks) `
                -Details ([pscustomobject][ordered]@{
                    lane_receipts = @($AttemptReceipt.lane_receipts)
                    start_checkpoint = $AttemptReceipt.start_checkpoint
                    raw_terminal_checkpoint = $AttemptReceipt.raw_terminal_checkpoint
                    finalization_generation = [int]$Restoration.generation
                }) `
                -CleanupRestoration ([pscustomobject][ordered]@{
                    result = "canonical_topology_verified"
                    evidence = @($Restoration.evidence)
                }) `
                -RepositoryRoot $repoRoot `
                -EvidenceRoot $script:evidenceRootPath `
                -OutputPath $OutputPath
        }
        $AttemptReceipt.sit_result = [pscustomobject][ordered]@{
            path = [string]$resultReference.path
            sha256 = [string]$resultReference.sha256
        }
        Set-Sprint8ASitAttemptState -Receipt $AttemptReceipt -State "passed" -Stage "complete"
        $AttemptReceipt.authoritative = $false
        $AttemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
        $AttemptReceipt.duration_ms = [long][Math]::Max(
            0,
            ((ConvertTo-Sprint8ADateTimeOffset `
                    -Value $AttemptReceipt.ended_at `
                    -Label "completed SIT attempt end") -
                (ConvertTo-Sprint8ADateTimeOffset `
                    -Value $AttemptReceipt.started_at `
                    -Label "completed SIT attempt start")).TotalMilliseconds
        )
        $AttemptReceipt.invalidation_decision = "none_required"
        Publish-Sprint8ASitDocument -Document $AttemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
        $manifest = Update-Sprint8ASitEvidenceManifest `
            -AttemptReceipt $AttemptReceipt `
            -PrerequisiteReferences @($PreflightReference, $CandidateReference) `
            -ResultReference $resultReference
        $AttemptReceipt.manifest = $manifest
        # The manifest intentionally inventories the immediately preceding passed-attempt bytes.
        # Do not rewrite the attempt after this point and invalidate that complete inventory.
        Write-Host "Sprint 8A SIT passed for candidate $($Context.candidate_fingerprint)." -ForegroundColor Green
        return $resultReference
    } catch {
        $finalizationError = $_
        $failureReference = Publish-Sprint8ASitFinalizationFailure `
            -AttemptReceipt $AttemptReceipt `
            -ErrorRecord $finalizationError
        if ([string]$AttemptReceipt.state -ceq "passed") {
            $AttemptReceipt.state = "finalizing"
            $AttemptReceipt.stage = "publication"
            $AttemptReceipt.state_history = @($AttemptReceipt.state_history) + @([pscustomobject][ordered]@{
                state = "finalizing"; stage = "publication"; at = [DateTimeOffset]::UtcNow.ToString("o")
            })
        }
        Set-Sprint8ASitAttemptState -Receipt $AttemptReceipt -State "failed" -Stage "publication-failed"
        $AttemptReceipt.classification = "evidence-finalization"
        $AttemptReceipt.invalidation_decision = "finalization_only_retry_permitted_if_identity_remains_exact"
        $AttemptReceipt.finalization_failures = @($AttemptReceipt.finalization_failures) + @($failureReference)
        Publish-Sprint8ASitDocument -Document $AttemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
        throw $finalizationError
    }
}

function Test-Sprint8ASitRunner {
    Test-Sprint8ALifecycleChain | Out-Null
    Test-Sprint8ALifecycleExclusiveLock
    $contracts = @(Get-Sprint8ASitLaneContracts)
    $expected = @(Get-Sprint8ASitLaneNames)
    if (($contracts.name -join "`n") -cne ($expected -join "`n") -or
        @($contracts | Where-Object { @($_.depends_on).Count -ne 1 -or $_.depends_on[0] -cne "authenticated-prerequisites" }).Count -ne 0 -or
        @($contracts | Where-Object topology_mutation_group -CEQ "serialized-deployed-topology").Count -ne 2) {
        throw "SIT runner self-test found a changed lane identity/dependency graph."
    }
    $attemptFixture = [pscustomobject]@{
        state = "preparing"; stage = "start"
        state_history = @([pscustomobject]@{ state = "preparing"; stage = "start"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "preparing" -Stage "prerequisites"
    Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "executing" -Stage "lanes"
    Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "harvesting" -Stage "lanes"
    Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "finalizing" -Stage "canonical-restoration"
    Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "finalizing" -Stage "publication"
    Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "passed" -Stage "complete"
    $illegalRejected = $false
    try { Set-Sprint8ASitAttemptState -Receipt $attemptFixture -State "executing" -Stage "lanes" } catch { $illegalRejected = $true }
    if (-not $illegalRejected) { throw "SIT runner self-test accepted an illegal terminal transition." }
    $retryFixture = [pscustomobject]@{
        state = "failed"; stage = "publication-failed"
        state_history = @([pscustomobject]@{ state = "failed"; stage = "publication-failed"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8ASitAttemptState `
        -Receipt $retryFixture `
        -State "finalizing" `
        -Stage "canonical-restoration" `
        -FinalizationRetry
    $laneFixture = [pscustomobject]@{
        state = "preparing"; stage = "declared"
        state_history = @([pscustomobject]@{ state = "preparing"; stage = "declared"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8ASitLaneState -Receipt $laneFixture -State "executing" -Stage "checks"
    Set-Sprint8ASitLaneState -Receipt $laneFixture -State "finalizing" -Stage "evidence"
    Set-Sprint8ASitLaneState -Receipt $laneFixture -State "passed" -Stage "complete"
    $laneTransitionRejected = $false
    try { Set-Sprint8ASitLaneState -Receipt $laneFixture -State "failed" -Stage "interrupted" } catch {
        $laneTransitionRejected = $true
    }
    if (-not $laneTransitionRejected) { throw "SIT runner self-test accepted an illegal terminal lane transition." }
    $blockedLaneFixture = [pscustomobject]@{
        state = "preparing"; stage = "declared"
        state_history = @([pscustomobject]@{ state = "preparing"; stage = "declared"; at = "2026-01-01T00:00:00Z" })
    }
    Set-Sprint8ASitLaneState -Receipt $blockedLaneFixture -State "blocked" -Stage "interrupted"
    $terminalFixtures = @($expected | ForEach-Object {
        [pscustomobject][ordered]@{
            name = $_
            state = "blocked"
            assertions_started = $false
            blocked_reason = "blocked by authenticated-prerequisites"
            classification = $null
        }
    })
    Assert-Sprint8AExactTerminalIdentities `
        -Results $terminalFixtures `
        -ExpectedNames $expected `
        -Label "SIT runner self-test terminal fixtures" | Out-Null
    $missingTerminalRejected = $false
    try {
        Assert-Sprint8AExactTerminalIdentities `
            -Results @($terminalFixtures | Select-Object -Skip 1) `
            -ExpectedNames $expected `
            -Label "SIT runner self-test missing terminal fixture" | Out-Null
    } catch { $missingTerminalRejected = $true }
    if (-not $missingTerminalRejected) {
        throw "SIT runner self-test accepted an incomplete terminal lane inventory."
    }
    $blockedBatch = Get-Sprint8ASitDefectBatch `
        -LaneReceipts @($terminalFixtures | ForEach-Object {
            [pscustomobject]@{
                receipt = [pscustomobject]@{ name = $_.name; summary = $_; command_results = @() }
            }
        }) `
        -Restoration $null
    if ([int]$blockedBatch.defect_count -ne 0 -or [int]$blockedBatch.blocked_count -ne 4) {
        throw "SIT runner self-test did not retain exact lane-level blocked reasons."
    }
    $failedSibling = [pscustomobject]@{ name = "static-and-boundaries"; state = "failed" }
    $eligibleSiblings = @($contracts | Where-Object name -CNE $failedSibling.name | Where-Object {
        @($_.depends_on) -notcontains $failedSibling.name
    })
    if ($eligibleSiblings.Count -ne 3) {
        throw "SIT runner self-test would suppress safe independent lanes after one lane failure."
    }
    $blocked = New-Sprint8ASitBlockedResult -Name "dependent" -Command "self-test" -Reason "blocked because prerequisite check failed"
    if ([string]$blocked.state -cne "blocked" -or [bool]$blocked.assertions_started -or
        [string]::IsNullOrWhiteSpace([string]$blocked.blocked_reason)) {
        throw "SIT runner self-test found malformed blocked-check evidence."
    }
    $literal = ConvertTo-Sprint8APowerShellLiteral -Value "a'b"
    if ($literal -cne "'a''b'") { throw "SIT runner self-test found unsafe PowerShell literal quoting." }
    $jsonTimestampFixture = '{"started_at":"2026-01-01T00:00:00+00:00"}' | ConvertFrom-Json
    $jsonTimestampInstant = ConvertTo-Sprint8ADateTimeOffset `
        -Value $jsonTimestampFixture.started_at `
        -Label "SIT JSON round-trip self-test start"
    if ($jsonTimestampInstant.ToUniversalTime().ToString("o") -cne "2026-01-01T00:00:00.0000000+00:00") {
        throw "SIT runner self-test changed the instant of a +00:00 JSON timestamp."
    }
    $sourceText = Get-Content -LiteralPath $PSCommandPath -Raw
    foreach ($fragment in @(
        'attempts/sit-$Attempt.json', 'sit/attempts', 'validation-attempt.lock',
        'Get-Sprint8AEvidenceFileManifestEntries', 'Assert-Sprint8AEvidenceManifestCompleteness',
        'Get-Sprint8AEvidenceManifestReplacementPaths',
        'Publish-Sprint8ALifecycleReceipt', 'canonical_topology_verified',
        'CopyToAsync',
        'cargo fmt --all -- --check',
        'cargo check --workspace --all-features --locked --offline',
        'cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings',
        'cargo test --workspace --all-features --locked --offline',
        'resource_reference_restricted_known_random_latency_profile',
        'tessara-components-contract', 'tessara-dashboard-module',
        'tessara-component-module', 'tessara-module-testkit',
        'validate-e2e.ps1', 'run-sprint-8a-failure-containment.ps1',
        'finalization_only_retry_permitted_if_identity_remains_exact'
    )) {
        if (-not $sourceText.Contains($fragment)) {
            throw "SIT runner self-test cannot find required contract fragment '$fragment'."
        }
    }
    foreach ($command in @(
        "materialize-sprint-8a.ps1", "audit-sprint-8a-deployed-inventory.ps1",
        "run-sprint-8a-deployed-smoke.ps1", "smoke-sprint-8a.ps1",
        "run-sprint-8a-component-upgrade.ps1", "run-sprint-8a-failure-containment.ps1",
        "validate-e2e.ps1"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot $command) -PathType Leaf)) {
            throw "SIT runner self-test cannot find repository command '$command'."
        }
    }
    $runBranchIndex = $sourceText.LastIndexOf('if ($Stage -ceq "Run")', [StringComparison]::Ordinal)
    $runLockFragment = '$lockHandle = Open-Sprint8AValidationAttemptLock -Path $lockPath'
    $runLockIndex = $sourceText.IndexOf($runLockFragment, $runBranchIndex, [StringComparison]::Ordinal)
    $runMutationFragment = '[IO.Directory]::CreateDirectory((Split-Path -Parent $script:attemptPath))'
    $runMutationIndex = $sourceText.IndexOf($runMutationFragment, $runBranchIndex, [StringComparison]::Ordinal)
    if ($runBranchIndex -lt 0 -or $runLockIndex -le $runBranchIndex -or
        $runMutationIndex -le $runLockIndex) {
        throw "SIT runner self-test found evidence-root mutation before exclusive Run-stage lock acquisition."
    }
    if (-not (Get-Command Get-Sprint8ACandidateIdentity).Parameters.ContainsKey("NormalizedDeploymentConfigurationSha256") -or
        -not (Get-Command Assert-Sprint8ALifecyclePrerequisiteSet).Parameters.ContainsKey("NormalizedDeploymentConfigurationSha256") -or
        -not (Get-Command Publish-Sprint8ALifecycleReceipt).Parameters.ContainsKey("NormalizedDeploymentConfigurationSha256") -or
        -not (Get-Command Publish-Sprint8ALifecycleReceipt).Parameters.ContainsKey("CleanupRestoration") -or
        -not (Get-Command Get-Sprint8AEvidenceManifestReplacementPaths -CommandType Function -ErrorAction SilentlyContinue) -or
        -not (Get-Command Publish-Sprint8AEvidenceManifest).Parameters.ContainsKey("Merge") -or
        -not (Get-Command Publish-Sprint8AEvidenceManifest).Parameters.ContainsKey("AuthorizedReplacementPaths") -or
        -not (Get-Command Assert-Sprint8AEvidenceManifestCompleteness).Parameters.ContainsKey("ManifestPath") -or
        -not (Get-Command Get-Sprint8AEvidenceFileManifestEntries).Parameters.ContainsKey("IgnoreExistingManifest")) {
        throw "SIT runner self-test found an incompatible lifecycle helper interface."
    }
    Write-Host "Sprint 8A SIT runner dependency, receipt, lock, and command-contract self-test passed."
}

if ($SelfTest) {
    Test-Sprint8ASitRunner
    return
}

if ($Attempt -lt 1) { throw "Sprint 8A SIT requires a positive -Attempt." }
if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
    throw "Sprint 8A SIT requires a repository-relative -EvidenceRoot."
}
$evidenceRootReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $EvidenceRoot
$script:evidenceRootPath = [string]$evidenceRootReference.full_path
$script:evidenceRootRelative = [string]$evidenceRootReference.path.TrimEnd('/')
$script:attemptPath = Join-Path $script:evidenceRootPath "attempts/sit-$Attempt.json"
$startSnapshotPath = Join-Path $script:evidenceRootPath "attempts/sit-$Attempt-start.json"
$lockPath = Join-Path $script:evidenceRootPath "validation-attempt.lock"
$script:attemptRootPath = Join-Path $script:evidenceRootPath "sit/attempt-$Attempt"
$script:logRootPath = Join-Path $script:attemptRootPath "logs"
$script:checkpointRootPath = Join-Path $script:attemptRootPath "checkpoints"
$script:lanePaths = [ordered]@{}
foreach ($lane in Get-Sprint8ASitLaneNames) {
    $script:lanePaths[$lane] = Join-Path $script:evidenceRootPath "sit/attempts/$lane-$Attempt.json"
}

$canonicalOutput = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $script:evidenceRootPath `
    -Path $OutputPath
if ([string]$canonicalOutput.path -cne "$script:evidenceRootRelative/sit-result.json") {
    throw "Sprint 8A SIT must publish only to '$script:evidenceRootRelative/sit-result.json'."
}

$preflightReference = $null
$candidateReference = $null
$lockHandle = $null
$context = $null
$attemptReceipt = $null
$launchEvidenceInitialized = $false
$script:topologyMutationStarted = $false

if ($Stage -ceq "Run") {
    $launchPaths = @($script:attemptPath, $startSnapshotPath) + @($script:lanePaths.Values)
    $collisions = @($launchPaths | Where-Object {
        (Test-Path -LiteralPath $_ -PathType Leaf) -or
            (Test-Path -LiteralPath "$_.sha256" -PathType Leaf)
    })
    foreach ($namespace in @(
        $script:attemptRootPath,
        (Join-Path $script:evidenceRootPath "sit/playwright-$Attempt")
    )) {
        if (Test-Path -LiteralPath $namespace) { $collisions += $namespace }
    }
    if ($collisions.Count -gt 0) {
        throw "Sprint 8A SIT attempt $Attempt cannot be reused; retained path(s) exist: $($collisions -join ', ')."
    }
    $lockHandle = Open-Sprint8AValidationAttemptLock -Path $lockPath
    try {
    [IO.Directory]::CreateDirectory((Split-Path -Parent $script:attemptPath)) | Out-Null
    [IO.Directory]::CreateDirectory((Split-Path -Parent $script:lanePaths[(Get-Sprint8ASitLaneNames)[0]])) | Out-Null
    [IO.Directory]::CreateDirectory($script:checkpointRootPath) | Out-Null
    [IO.Directory]::CreateDirectory($script:logRootPath) | Out-Null
    $placeholderSource = [pscustomobject][ordered]@{
        commit = "0" * 40
        tree = "0" * 40
        dirty = $false
        branch = "unverified"
        acceptance_inventory_sha256 = "0" * 64
        deployment_inputs_sha256 = "0" * 64
    }
    $startedAt = [DateTimeOffset]::UtcNow
    $attemptReceipt = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "sit"
        attempt = $Attempt
        authoritative = $false
        state = "preparing"
        stage = "start"
        assertions_started = $false
        started_at = $startedAt.ToString("o")
        ended_at = $null
        duration_ms = 0L
        state_history = @([pscustomobject][ordered]@{
            state = "preparing"; stage = "start"; at = $startedAt.ToString("o")
        })
        source_identity = $placeholderSource
        source_verification_state = "unverified"
        environment_fingerprint = "0" * 64
        normalized_deployment_configuration_sha256 = "0" * 64
        candidate_fingerprint = "0" * 64
        claimed_prerequisite_paths = @($PreflightReceipt, $CandidateReceipt)
        prerequisite_receipts = @()
        declared_prerequisites = @(
            [pscustomobject][ordered]@{ name = "attempt-state-prerequisite"; depends_on = @() },
            [pscustomobject][ordered]@{ name = "authenticated-prerequisites"; depends_on = @("attempt-state-prerequisite") }
        )
        declared_lanes = @(Get-Sprint8ASitLaneContracts)
        prerequisite_results = @()
        checks = @()
        lane_receipts = @()
        assertion_count = 0
        failure_count = 0
        blocked_count = 0
        raw_results_complete = $false
        classification = $null
        failure_batch = $null
        invalidation_decision = "pending_validation_coordinator"
        cleanup_restoration = [pscustomobject][ordered]@{
            required = $true; result = "pending"; generation = 0; checks = @(); evidence = @(); passed = $false
        }
        start_checkpoint = $null
        raw_terminal_checkpoint = $null
        sit_result = $null
        manifest = $null
        finalization_failures = @()
        checkpoint_references = @()
    }
    Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath | Out-Null
    $startReference = Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $startSnapshotPath
    $attemptReceipt.start_checkpoint = [pscustomobject][ordered]@{
        path = [string]$startReference.path; sha256 = [string]$startReference.sha256
    }
    Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
    foreach ($contract in Get-Sprint8ASitLaneContracts) {
        $laneStarted = [DateTimeOffset]::UtcNow
        $laneReceipt = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8a"
            phase = "sit-lane"
            attempt = $Attempt
            authoritative = $false
            name = [string]$contract.name
            state = "preparing"
            stage = "declared"
            state_history = @([pscustomobject][ordered]@{
                state = "preparing"; stage = "declared"; at = $laneStarted.ToString("o")
            })
            depends_on = @($contract.depends_on)
            topology_mutation_group = $contract.topology_mutation_group
            assertions_started = $false
            prepared_at = $laneStarted.ToString("o")
            started_at = $null
            ended_at = $null
            duration_ms = 0L
            source_identity = $placeholderSource
            source_verification_state = "unverified"
            environment_fingerprint = "0" * 64
            normalized_deployment_configuration_sha256 = "0" * 64
            candidate_fingerprint = "0" * 64
            command_results = @()
            checkpoints = @()
            assertion_count = 0
            failure_count = 0
            blocked_count = 0
            blocked_reason = $null
            evidence = @()
            summary = $null
        }
        Publish-Sprint8ASitDocument `
            -Document $laneReceipt `
            -Path $script:lanePaths[[string]$contract.name] | Out-Null
    }
    $launchEvidenceInitialized = $true

        $prerequisiteLog = Join-Path $script:logRootPath "authenticated-prerequisites.log"
        [IO.File]::WriteAllText(
            $prerequisiteLog,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] attempt-state-prerequisite exclusive evidence-root lock acquired before mutation`n",
            [Text.UTF8Encoding]::new($false)
        )
        Set-Sprint8ASitAttemptState -Receipt $attemptReceipt -State "preparing" -Stage "prerequisites"
        Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null

        $preflightReference = Get-Sprint8ASitEvidenceReference -Path $PreflightReceipt -RequireSidecar
        $candidateReference = Get-Sprint8ASitEvidenceReference -Path $CandidateReceipt -RequireSidecar
        $currentAttemptPaths = @(
            Get-Sprint8ASitRelativePath $script:attemptPath
            Get-Sprint8ASitRelativePath "$script:attemptPath.sha256"
            Get-Sprint8ASitRelativePath $startSnapshotPath
            Get-Sprint8ASitRelativePath "$startSnapshotPath.sha256"
            Get-Sprint8ASitRelativePath $prerequisiteLog
        ) + @($script:lanePaths.Values | ForEach-Object {
            Get-Sprint8ASitRelativePath $_
            Get-Sprint8ASitRelativePath "$_.sha256"
        })
        Assert-Sprint8ASitManifestBaseline `
            -PreflightReference $preflightReference `
            -CandidateReference $candidateReference `
            -CurrentAttemptPaths $currentAttemptPaths | Out-Null

        $context = Get-Sprint8ASitAuthenticatedContext `
            -PreflightReference $preflightReference `
            -CandidateReference $candidateReference
        [IO.File]::AppendAllText(
            $prerequisiteLog,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] exact prerequisite/source/environment chain authenticated`n",
            [Text.UTF8Encoding]::new($false)
        )
        $attemptReceipt.source_identity = $context.source
        $attemptReceipt.source_verification_state = "verified"
        $attemptReceipt.environment_fingerprint = [string]$context.environment_fingerprint
        $attemptReceipt.normalized_deployment_configuration_sha256 = `
            [string]$context.normalized_deployment_configuration_sha256
        $attemptReceipt.candidate_fingerprint = [string]$context.candidate_fingerprint
        $attemptReceipt.prerequisite_receipts = @(
            [pscustomobject][ordered]@{ path = [string]$preflightReference.path; sha256 = [string]$preflightReference.sha256 },
            [pscustomobject][ordered]@{ path = [string]$candidateReference.path; sha256 = [string]$candidateReference.sha256 }
        )
        $prerequisiteEnded = [DateTimeOffset]::UtcNow
        $attemptReceipt.prerequisite_results = @([pscustomobject][ordered]@{
            name = "authenticated-prerequisites"
            state = "passed"
            assertions_started = $true
            assertions_started_at = $startedAt.ToString("o")
            command = "exclusive lock; exact preflight/candidate/source/environment/manifest authentication"
            exit_status = 0
            started_at = $startedAt.ToString("o")
            ended_at = $prerequisiteEnded.ToString("o")
            duration_ms = [long][Math]::Max(0, ($prerequisiteEnded - $startedAt).TotalMilliseconds)
            classification = $null
            classification_source = $null
            failure_message = $null
            blocked_reason = $null
            evidence = @(Get-Sprint8ASitFileReferences -Paths @($prerequisiteLog) | ForEach-Object {
                [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
            })
        })
        Set-Sprint8ASitAttemptState -Receipt $attemptReceipt -State "executing" -Stage "lanes"
        Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
        Update-Sprint8ASitEvidenceManifest `
            -AttemptReceipt $attemptReceipt `
            -PrerequisiteReferences @($preflightReference, $candidateReference) `
            -ResultReference $null | Out-Null

        $laneResults = [Collections.Generic.List[object]]::new()
        foreach ($contract in Get-Sprint8ASitLaneContracts) {
            $laneResult = Invoke-Sprint8ASitLane `
                -Contract $contract `
                -Environment $context.environment `
                -AttemptReceipt $attemptReceipt `
                -AttemptPath $script:attemptPath `
                -LanePath $script:lanePaths[[string]$contract.name]
            $laneResults.Add($laneResult)
            $attemptReceipt.checks = @($laneResults | ForEach-Object { $_.summary })
            $attemptReceipt.lane_receipts = @($laneResults | ForEach-Object { $_.reference })
            $attemptReceipt.assertions_started = @($attemptReceipt.checks | Where-Object assertions_started -EQ $true).Count -gt 0
            $attemptReceipt.assertion_count = @($attemptReceipt.checks | Where-Object assertions_started -EQ $true).Count
            $attemptReceipt.failure_count = @($attemptReceipt.checks | Where-Object state -CEQ "failed").Count
            $attemptReceipt.blocked_count = @($attemptReceipt.checks | Where-Object state -CEQ "blocked").Count
            Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
            $laneCheckpoint = Write-Sprint8ASitCheckpoint `
                -Document $attemptReceipt `
                -Name ("attempt-after-{0}" -f $contract.name)
            $attemptReceipt.checkpoint_references = @($attemptReceipt.checkpoint_references) + @($laneCheckpoint)
            Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
        }
        Assert-Sprint8AExactTerminalIdentities `
            -Results @($attemptReceipt.checks) `
            -ExpectedNames (Get-Sprint8ASitLaneNames) `
            -Label "SIT attempt $Attempt" | Out-Null
        $attemptReceipt.raw_results_complete = $true
        $rawCheckpoint = Write-Sprint8ASitCheckpoint -Document $attemptReceipt -Name "attempt-raw-terminal"
        $attemptReceipt.raw_terminal_checkpoint = $rawCheckpoint
        Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null

        Set-Sprint8ASitAttemptState -Receipt $attemptReceipt -State "finalizing" -Stage "canonical-restoration"
        $attemptReceipt.cleanup_restoration.generation = 1
        Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
        $restoration = Invoke-Sprint8ASitCanonicalRestoration -Context $context -Generation 1
        $attemptReceipt.cleanup_restoration = $restoration
        $laneFailures = @($attemptReceipt.checks | Where-Object state -CNE "passed")
        if ($laneFailures.Count -gt 0 -or -not [bool]$restoration.passed) {
            $batch = Get-Sprint8ASitDefectBatch -LaneReceipts @($laneResults) -Restoration $restoration
            Set-Sprint8ASitAttemptState -Receipt $attemptReceipt -State "failed" -Stage $(if ($laneFailures.Count -gt 0) { "diagnostic-harvest-complete" } else { "canonical-restoration" })
            $attemptReceipt.failure_batch = $batch
            $attemptReceipt.failure_count = [int]$batch.defect_count
            $attemptReceipt.blocked_count = [int]$batch.blocked_count
            $attemptReceipt.classification = if ($batch.defect_count -gt 0) {
                Get-Sprint8ASitAggregateClassification -Failures @($batch.defects)
            } else { "harness" }
            $attemptReceipt.invalidation_decision = "withheld_for_validation_coordinator"
            $attemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            $attemptReceipt.duration_ms = [long][Math]::Max(
                0,
                ((ConvertTo-Sprint8ADateTimeOffset `
                        -Value $attemptReceipt.ended_at `
                        -Label "failed SIT attempt end") - $startedAt).TotalMilliseconds
            )
            Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
            Update-Sprint8ASitEvidenceManifest `
                -AttemptReceipt $attemptReceipt `
                -PrerequisiteReferences @($preflightReference, $candidateReference) `
                -ResultReference $null | Out-Null
            throw "Sprint 8A SIT retained $($batch.defect_count) consolidated defect(s) and $($batch.blocked_count) blocked check(s); sit-result.json was withheld."
        }
        Complete-Sprint8ASitPublication `
            -AttemptReceipt $attemptReceipt `
            -Context $context `
            -PreflightReference $preflightReference `
            -CandidateReference $candidateReference `
            -Restoration $restoration | Out-Null
    } catch {
        $runError = $_
        if (-not $launchEvidenceInitialized -or $null -eq $attemptReceipt) {
            throw $runError
        }
        if ($null -eq $lockHandle) {
            $attemptReceipt.classification = "preflight/setup"
        }
        if ([string]$attemptReceipt.state -in @("preparing", "executing", "harvesting", "finalizing")) {
            $authenticated = $null -ne $context
            $defaultClassification = if ($authenticated) { "harness" } else { "preflight/setup" }
            $classification = Resolve-Sprint8ASitFailureClassification `
                -DefaultClassification $defaultClassification `
                -ErrorRecord $runError
            $terminalReason = if ($authenticated) {
                "blocked or failed because the SIT runner was interrupted: $($runError.Exception.Message)"
            } else {
                "blocked because authenticated-prerequisites failed: $($runError.Exception.Message)"
            }
            $terminalLaneResults = Complete-Sprint8ASitRemainingLanes `
                -AttemptReceipt $attemptReceipt `
                -Reason $terminalReason
            $attemptReceipt.checks = @($terminalLaneResults | ForEach-Object { $_.summary })
            $attemptReceipt.lane_receipts = @($terminalLaneResults | ForEach-Object { $_.reference })
            $attemptReceipt.assertions_started = `
                @($attemptReceipt.checks | Where-Object assertions_started -EQ $true).Count -gt 0
            $attemptReceipt.assertion_count = `
                @($attemptReceipt.checks | Where-Object assertions_started -EQ $true).Count
            Assert-Sprint8AExactTerminalIdentities `
                -Results @($attemptReceipt.checks) `
                -ExpectedNames (Get-Sprint8ASitLaneNames) `
                -Label "Interrupted SIT attempt $Attempt" | Out-Null
            $attemptReceipt.raw_results_complete = $true
            $prerequisiteLog = Join-Path $script:logRootPath "authenticated-prerequisites.log"
            if (-not (Test-Path -LiteralPath $prerequisiteLog -PathType Leaf)) {
                [IO.File]::WriteAllText(
                    $prerequisiteLog,
                    ($runError | Out-String),
                    [Text.UTF8Encoding]::new($false)
                )
            }
            $failureEvidence = @(Get-Sprint8ASitFileReferences -Paths @($prerequisiteLog) | ForEach-Object {
                [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
            })
            if (-not $authenticated) {
                $prerequisiteEnded = [DateTimeOffset]::UtcNow
                $prerequisiteStarted = ConvertTo-Sprint8ADateTimeOffset `
                    -Value $attemptReceipt.started_at `
                    -Label "SIT prerequisite attempt start"
                $attemptReceipt.prerequisite_results = @([pscustomobject][ordered]@{
                    name = "authenticated-prerequisites"
                    state = "failed"
                    assertions_started = $true
                    assertions_started_at = $prerequisiteStarted.ToString("o")
                    command = "exclusive lock; exact preflight/candidate/source/environment/manifest authentication"
                    exit_status = 1
                    started_at = $prerequisiteStarted.ToString("o")
                    ended_at = $prerequisiteEnded.ToString("o")
                    duration_ms = [long][Math]::Max(0, ($prerequisiteEnded - $prerequisiteStarted).TotalMilliseconds)
                    classification = [string]$classification.classification
                    classification_source = [string]$classification.source
                    failure_message = $runError.Exception.Message
                    blocked_reason = $null
                    evidence = $failureEvidence
                })
            }

            $restoration = $null
            if ($authenticated -and $script:topologyMutationStarted -and $null -ne $lockHandle) {
                try {
                    if ([string]$attemptReceipt.state -in @("executing", "harvesting")) {
                        Set-Sprint8ASitAttemptState `
                            -Receipt $attemptReceipt `
                            -State "finalizing" `
                            -Stage "canonical-restoration"
                    }
                    $restorationGeneration = [int]$attemptReceipt.cleanup_restoration.generation + 1
                    $attemptReceipt.cleanup_restoration.generation = $restorationGeneration
                    Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
                    $restoration = Invoke-Sprint8ASitCanonicalRestoration `
                        -Context $context `
                        -Generation $restorationGeneration
                    $attemptReceipt.cleanup_restoration = $restoration
                } catch {
                    $restorationFailureLog = Join-Path $script:logRootPath "interruption-restoration-failure.log"
                    [IO.File]::WriteAllText(
                        $restorationFailureLog,
                        ($_ | Out-String),
                        [Text.UTF8Encoding]::new($false)
                    )
                    $restorationEvidence = @(Get-Sprint8ASitFileReferences -Paths @($restorationFailureLog) | ForEach-Object {
                        [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
                    })
                    $now = [DateTimeOffset]::UtcNow
                    $restoration = [pscustomobject][ordered]@{
                        required = $true
                        result = "canonical_topology_not_proven"
                        generation = [int]$attemptReceipt.cleanup_restoration.generation
                        checks = @([pscustomobject][ordered]@{
                            name = "interruption-restoration"
                            command = "independent canonical topology restoration after interruption"
                            state = "failed"
                            assertions_started = $true
                            assertions_started_at = $now.ToString("o")
                            started_at = $now.ToString("o")
                            ended_at = $now.ToString("o")
                            duration_ms = 0L
                            exit_status = 1
                            classification = "harness"
                            classification_source = "runner_interruption"
                            failure_message = $_.Exception.Message
                            blocked_reason = $null
                            evidence = $restorationEvidence
                        })
                        evidence = $restorationEvidence
                        passed = $false
                    }
                    $attemptReceipt.cleanup_restoration = $restoration
                }
            }

            $additionalDefect = [pscustomobject][ordered]@{
                lane = if ($authenticated) { "runner-interruption" } else { "authenticated-prerequisites" }
                check = if ($authenticated) { "runner-interruption" } else { "authenticated-prerequisites" }
                classification = [string]$classification.classification
                classification_source = [string]$classification.source
                message = $runError.Exception.Message
                evidence = $failureEvidence
            }
            $batch = Get-Sprint8ASitDefectBatch `
                -LaneReceipts @($terminalLaneResults) `
                -Restoration $restoration `
                -AdditionalDefects @($additionalDefect)
            $attemptReceipt.state = "failed"
            $attemptReceipt.stage = if ($authenticated) { "interrupted" } else { "prerequisites" }
            $attemptReceipt.state_history = @($attemptReceipt.state_history) + @([pscustomobject][ordered]@{
                state = "failed"; stage = [string]$attemptReceipt.stage; at = [DateTimeOffset]::UtcNow.ToString("o")
            })
            $attemptReceipt.failure_batch = $batch
            $attemptReceipt.failure_count = [int]$batch.defect_count
            $attemptReceipt.blocked_count = [int]$batch.blocked_count
            $attemptReceipt.classification = Get-Sprint8ASitAggregateClassification -Failures @($batch.defects)
            $attemptReceipt.invalidation_decision = "withheld_for_validation_coordinator"
            $attemptReceipt.ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            $attemptReceipt.duration_ms = [long][Math]::Max(
                0,
                ((ConvertTo-Sprint8ADateTimeOffset `
                        -Value $attemptReceipt.ended_at `
                        -Label "interrupted SIT attempt end") -
                    (ConvertTo-Sprint8ADateTimeOffset `
                        -Value $attemptReceipt.started_at `
                        -Label "interrupted SIT attempt start")).TotalMilliseconds
            )
            Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
            $interruptedCheckpoint = Write-Sprint8ASitCheckpoint `
                -Document $attemptReceipt `
                -Name "attempt-interrupted-terminal"
            $attemptReceipt.raw_terminal_checkpoint = $interruptedCheckpoint
            Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
            if ($authenticated -and $null -ne $lockHandle -and
                $null -ne $preflightReference -and $null -ne $candidateReference) {
                try {
                    Update-Sprint8ASitEvidenceManifest `
                        -AttemptReceipt $attemptReceipt `
                        -PrerequisiteReferences @($preflightReference, $candidateReference) `
                        -ResultReference $null | Out-Null
                } catch {
                    Write-Host "SIT failure-evidence manifest finalization also failed: $($_.Exception.Message)" -ForegroundColor Red
                }
            }
        }
        throw $runError
    } finally {
        if ($null -ne $lockHandle) { $lockHandle.Dispose() }
    }
    return
}

# Finalization-only retry. It never reruns a raw lane.
if (-not (Test-Path -LiteralPath $script:attemptPath -PathType Leaf)) {
    throw "SIT Finalize requires the retained failed/publication-failed attempt receipt."
}
$lockHandle = Open-Sprint8AValidationAttemptLock -Path $lockPath
try {
    Assert-Sprint8AReceiptSidecar -Path $script:attemptPath | Out-Null
    $attemptReceipt = Get-Content -LiteralPath $script:attemptPath -Raw | ConvertFrom-Json
    if ([string]$attemptReceipt.state -cne "failed" -or
        [string]$attemptReceipt.stage -cne "publication-failed" -or
        [string]$attemptReceipt.classification -cne "evidence-finalization") {
        throw "SIT Finalize is restricted to an exact evidence-finalization failure with immutable passing raw results."
    }
    Assert-Sprint8ASitRawResultsForFinalization -AttemptReceipt $attemptReceipt
    $preflightReference = Get-Sprint8ASitEvidenceReference -Path $PreflightReceipt -RequireSidecar
    $candidateReference = Get-Sprint8ASitEvidenceReference -Path $CandidateReceipt -RequireSidecar
    $context = Get-Sprint8ASitAuthenticatedContext `
        -PreflightReference $preflightReference `
        -CandidateReference $candidateReference
    if (-not (Test-Sprint8ASourceIdentityMatch -Expected $attemptReceipt.source_identity -Actual $context.source) -or
        [string]$attemptReceipt.environment_fingerprint -cne [string]$context.environment_fingerprint -or
        [string]$attemptReceipt.normalized_deployment_configuration_sha256 -cne
            [string]$context.normalized_deployment_configuration_sha256 -or
        [string]$attemptReceipt.candidate_fingerprint -cne [string]$context.candidate_fingerprint) {
        throw "SIT Finalize source, environment, or candidate identity changed; full readiness/rehearsal/SIT is required."
    }
    $generation = [int]$attemptReceipt.cleanup_restoration.generation + 1
    Set-Sprint8ASitAttemptState `
        -Receipt $attemptReceipt `
        -State "finalizing" `
        -Stage "canonical-restoration" `
        -FinalizationRetry
    $attemptReceipt.cleanup_restoration.generation = $generation
    Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
    $restoration = Invoke-Sprint8ASitCanonicalRestoration -Context $context -Generation $generation
    if (-not [bool]$restoration.passed) {
        $attemptReceipt.state = "failed"
        $attemptReceipt.stage = "canonical-restoration"
        $attemptReceipt.state_history = @($attemptReceipt.state_history) + @([pscustomobject][ordered]@{
            state = "failed"; stage = "canonical-restoration"; at = [DateTimeOffset]::UtcNow.ToString("o")
        })
        $attemptReceipt.cleanup_restoration = $restoration
        $attemptReceipt.classification = Get-Sprint8ASitAggregateClassification `
            -Failures @($restoration.checks | Where-Object state -CEQ "failed")
        $attemptReceipt.invalidation_decision = "withheld_for_validation_coordinator"
        Publish-Sprint8ASitDocument -Document $attemptReceipt -Path $script:attemptPath -Overwrite | Out-Null
        Update-Sprint8ASitEvidenceManifest `
            -AttemptReceipt $attemptReceipt `
            -PrerequisiteReferences @($preflightReference, $candidateReference) `
            -ResultReference $null | Out-Null
        throw "SIT finalization retry could not re-establish canonical topology."
    }
    Complete-Sprint8ASitPublication `
        -AttemptReceipt $attemptReceipt `
        -Context $context `
        -PreflightReference $preflightReference `
        -CandidateReference $candidateReference `
        -Restoration $restoration | Out-Null
} finally {
    $lockHandle.Dispose()
}

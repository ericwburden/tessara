[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$evidenceRootPath = [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
$attemptPath = Join-Path $evidenceRootPath "attempts/readiness-$Attempt.json"
$resultPath = Join-Path $evidenceRootPath "validation-readiness-result.json"
$logRoot = Join-Path $evidenceRootPath "readiness-$Attempt"
[IO.Directory]::CreateDirectory($logRoot) | Out-Null
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-acceptance-contract.ps1")

function Get-SourceIdentity {
    $status = @(& git -C $repoRoot status --porcelain=v1)
    [ordered]@{
        commit = (& git -C $repoRoot rev-parse HEAD).Trim()
        tree = (& git -C $repoRoot rev-parse "HEAD^{tree}").Trim()
        dirty = $status.Count -ne 0
        branch = (& git -C $repoRoot branch --show-current).Trim()
    }
}

$source = Get-SourceIdentity
$startedAt = [DateTimeOffset]::UtcNow
Publish-Sprint7AEvidence -Document ([ordered]@{
    schema_version=1; sprint="sprint-8a"; phase="validation-readiness"; attempt=$Attempt
    authoritative=$false; state="preparing"; assertions_started=$false
    started_at=$startedAt.ToString("o"); mutable_source_identity=$source; prerequisite_receipts=@()
}) -OutputPath $attemptPath | Out-Null

$checks = [Collections.Generic.List[object]]::new()
function Invoke-ReadinessCheck {
    param([string]$Name, [string]$Command, [scriptblock]$Action)
    $start = [DateTimeOffset]::UtcNow
    $passed = $false
    try { $detail = (& $Action | Out-String); $passed = $true }
    catch { $detail = $_ | Out-String }
    $end = [DateTimeOffset]::UtcNow
    $log = Join-Path $logRoot "$Name.log"
    [IO.File]::WriteAllText($log, $detail.TrimEnd() + "`n", [Text.UTF8Encoding]::new($false))
    $checks.Add([ordered]@{
        name=$Name; command=$Command; started_at=$start.ToString("o"); ended_at=$end.ToString("o")
        duration_ms=[math]::Round(($end-$start).TotalMilliseconds); exit_status=if($passed){0}else{1}
        passed=$passed; classification=if($passed){$null}else{"preflight/setup"}
        evidence_path=[IO.Path]::GetRelativePath($repoRoot,$log).Replace("\","/")
    })
}

Push-Location $repoRoot
try {
    Invoke-ReadinessCheck "clean-source" "git status --porcelain=v1" {
        if ((Get-SourceIdentity).dirty) { throw "Readiness requires clean tracked source." }; $source | ConvertTo-Json -Compress
    }
    Invoke-ReadinessCheck "toolchain" "rustc/cargo/docker/compose/node/npm --version" {
        $values=@(& rustc --version; & cargo --version; & docker --version; & docker compose version; & node --version; & npm --version)
        if($LASTEXITCODE -ne 0 -or $values.Count -ne 6){throw "Required toolchain unavailable."}; $values
    }
    Invoke-ReadinessCheck "database-environment" "five pairwise-distinct disposable database bindings" {
        $names = @(
            "TEST_API_DATABASE_URL",
            "TEST_API_FRESH_DATABASE_URL",
            "TEST_REFERENCE_MODULE_DATABASE_URL",
            "TEST_API_ENROLLMENT_DATABASE_URL",
            "TEST_INSTALLATION_CONTROL_DATABASE_URL"
        )
        $identities = @()
        foreach($name in $names) {
            $value = [Environment]::GetEnvironmentVariable($name)
            if([string]::IsNullOrWhiteSpace($value)){throw "Readiness requires $name for complete non-skipping validation."}
            $uri = [Uri]$value
            $database = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart("/"))
            if($uri.Scheme -notin @("postgres","postgresql") -or $database -notmatch "(^|[_-])test([_-]|$)"){
                throw "$name must identify one explicit token-bounded disposable test database."
            }
            $port = if($uri.IsDefaultPort){5432}else{$uri.Port}
            $identities += "$($uri.Host.ToLowerInvariant()):$port/$($database.ToLowerInvariant())"
        }
        if(@($identities | Sort-Object -Unique).Count -ne $names.Count){throw "Validation database identities must be pairwise distinct."}
        if([Environment]::GetEnvironmentVariable("SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET") -cne "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"){
            throw "Readiness requires the exact destructive fresh-reset acknowledgement."
        }
        $identities
    }
    Invoke-ReadinessCheck "playwright-locked-install" "npm ci --prefix end2end" {
        & npm ci --prefix end2end 2>&1; if($LASTEXITCODE -ne 0){throw "Locked Playwright dependency installation failed."}
    }
    Invoke-ReadinessCheck "playwright-discovery" "npm --prefix end2end test -- --list" {
        $list=& npm --prefix end2end test -- --list 2>&1
        if($LASTEXITCODE -ne 0){throw "Playwright discovery failed."}
        if(-not ($list -match "Total: 70 tests in 9 files")){throw "Playwright discovery did not return the exact 70-test/9-file inventory."}; $list
    }
    Invoke-ReadinessCheck "compose-and-fixture-contract" "Test-Sprint8AAcceptanceContract" {
        Test-Sprint7AAcceptanceContract; Test-Sprint8AAcceptanceContract; "Compose images, fixtures, and UAT inventory agree."
    }
    Invoke-ReadinessCheck "runner-parsing" "PowerShell parser for Sprint 8A validation runners" {
        foreach($file in @("scripts/materialize-sprint-8a.ps1","scripts/bootstrap-sprint-7a-composition.ps1","scripts/smoke-sprint-8a.ps1","scripts/uat-sprint-8a.ps1","scripts/test-sprint-validation-harvest.ps1","scripts/verify-sprint-8a-component-upgrade.ps1")){
            $tokens=$null;$errors=$null;[void][Management.Automation.Language.Parser]::ParseFile((Resolve-Path $file),[ref]$tokens,[ref]$errors)
            if($errors.Count){throw "$file parse failed: $($errors.Message -join '; ')"}
        }; "runner parsing passed"
    }
    Invoke-ReadinessCheck "runner-self-tests" "Sprint 8A smoke/UAT/harvest/upgrade self-tests" {
        & ./scripts/smoke-sprint-8a.ps1 -SelfTest; if($LASTEXITCODE -ne 0){throw "Smoke self-test failed."}
        & ./scripts/uat-sprint-8a.ps1 -SelfTest; if($LASTEXITCODE -ne 0){throw "UAT diagnostic self-test failed."}
        & ./scripts/test-sprint-validation-harvest.ps1 -SelfTest; if($LASTEXITCODE -ne 0){throw "Harvest self-test failed."}
        $a="a"*64;$b="b"*64
        & ./scripts/verify-sprint-8a-component-upgrade.ps1 -BaselineImage "local/components@sha256:$a" -CandidateImage "local/components@sha256:$b" -CurrentImage "local/components@sha256:$b" -SelfTest
        if($LASTEXITCODE -ne 0){throw "Upgrade self-test failed."}
    }
    Invoke-ReadinessCheck "reset-dry-run" "materialize-sprint-8a.ps1 -AuthorizeDisposableReset -WhatIf" {
        & ./scripts/materialize-sprint-8a.ps1 -AuthorizeDisposableReset -WhatIf; if($LASTEXITCODE -ne 0){throw "Reset dry-run failed."}
    }
    Invoke-ReadinessCheck "package-boundaries" "scripts/check-web-crate-boundaries.ps1" {
        & ./scripts/check-web-crate-boundaries.ps1; if($LASTEXITCODE -ne 0){throw "Package boundaries failed."}
    }
    Invoke-ReadinessCheck "cargo-metadata" "cargo metadata --locked --offline --no-deps --format-version 1" {
        $metadata=& cargo metadata --locked --offline --no-deps --format-version 1 | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw "Cargo metadata failed."}
        foreach($name in @("tessara-component-module","tessara-components-contract","tessara-datasets-contract","tessara-dashboard-placement-renderer")){
            if(@($metadata.packages.name) -notcontains $name){throw "Workspace omits '$name'."}
        }; @($metadata.packages.name | Sort-Object)
    }
    Invoke-ReadinessCheck "markdown-links" "scripts/verify-markdown-links.ps1" {
        & ./scripts/verify-markdown-links.ps1; if($LASTEXITCODE -ne 0){throw "Markdown links failed."}
    }
    Invoke-ReadinessCheck "final-clean-source" "git status --porcelain=v1" {
        if ((Get-SourceIdentity).dirty) { throw "Readiness probes changed tracked source." }; "clean"
    }
} finally { Pop-Location }

$endedAt=[DateTimeOffset]::UtcNow
$failures=@($checks | Where-Object {-not $_.passed})
$receipt=[ordered]@{
    schema_version=1;sprint="sprint-8a";phase="validation-readiness";attempt=$Attempt;authoritative=$false
    state=if($failures.Count -eq 0){"passed"}else{"failed"};assertions_started=$true
    started_at=$startedAt.ToString("o");ended_at=$endedAt.ToString("o");duration_ms=[math]::Round(($endedAt-$startedAt).TotalMilliseconds)
    mutable_source_identity=$source
    environment_identity=[ordered]@{os=[Environment]::OSVersion.VersionString;compose_project="tessara-sprint-8a";profile="reference";gateway_port=8088;supervisor_port=8098;evidence_root=$EvidenceRoot}
    prerequisite_receipts=@();checks=$checks;assertion_count=$checks.Count;failure_count=$failures.Count
    classification=if($failures.Count -eq 0){$null}else{"preflight/setup"}
    invalidation_decision=if($failures.Count -eq 0){"none"}else{"candidate freeze forbidden"}
    cleanup_restoration=[ordered]@{required=$false;result="not_applicable"}
}
Publish-Sprint7AEvidence -Document $receipt -OutputPath $attemptPath -Overwrite | Out-Null
if($failures.Count){throw "Sprint 8A readiness failed $($failures.Count) checks; inspect $attemptPath"}
Publish-Sprint7AEvidence -Document $receipt -OutputPath $resultPath -Overwrite | Out-Null
Write-Host "Sprint 8A Validation Readiness passed."

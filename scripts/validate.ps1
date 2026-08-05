[CmdletBinding()]
param(
    [switch]$Fast,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$fullValidationDatabaseEnvironmentNames = @(
    "TEST_API_DATABASE_URL",
    "TEST_API_FRESH_DATABASE_URL",
    "TEST_REFERENCE_MODULE_DATABASE_URL",
    "TEST_API_ENROLLMENT_DATABASE_URL"
)
$destructiveFreshResetAcknowledgement =
    "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET"

function Test-TessaraDisposableDatabaseName {
    param([Parameter(Mandatory)][string]$DatabaseName)

    $tokens = @(
        $DatabaseName.ToLowerInvariant() -split "[^a-z0-9]+" |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    )
    $acceptedTokens = @(
        "test",
        "tests",
        "testing",
        "upgrade",
        "clone",
        "rollback",
        "sprint6a"
    )
    if (@($tokens | Where-Object { $_ -in $acceptedTokens }).Count -gt 0) {
        return $true
    }

    for ($index = 0; $index -lt ($tokens.Count - 1); $index++) {
        if ($tokens[$index] -eq "sprint" -and $tokens[$index + 1] -eq "6a") {
            return $true
        }
    }

    return $false
}

function ConvertFrom-TessaraValidationDatabaseUrl {
    param(
        [Parameter(Mandatory)][string]$EnvironmentName,
        [Parameter(Mandatory)][string]$DatabaseUrl
    )

    try {
        $uri = [Uri]::new($DatabaseUrl, [UriKind]::Absolute)
    } catch {
        throw "Full validation requires $EnvironmentName to be an absolute PostgreSQL URL."
    }
    if ($uri.Scheme -notin @("postgres", "postgresql") -or
        [string]::IsNullOrWhiteSpace($uri.Host)) {
        throw "Full validation requires $EnvironmentName to be an absolute postgres:// or postgresql:// URL with a host."
    }

    $databaseName = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart("/"))
    if ([string]::IsNullOrWhiteSpace($databaseName) -or
        $databaseName.Contains("/") -or
        $databaseName -notmatch "^[A-Za-z_][A-Za-z0-9_-]*$") {
        throw "Full validation requires $EnvironmentName to name one explicit PostgreSQL database."
    }
    if (-not (Test-TessaraDisposableDatabaseName -DatabaseName $databaseName)) {
        throw "Full validation refuses $EnvironmentName database '$databaseName': its name lacks a token-bounded disposable marker."
    }

    $port = if ($uri.IsDefaultPort -or $uri.Port -lt 0) { 5432 } else { $uri.Port }
    [pscustomobject][ordered]@{
        EnvironmentName = $EnvironmentName
        DatabaseName = $databaseName
        Identity = "$($uri.Host.ToLowerInvariant()):$port/$($databaseName.ToLowerInvariant())"
    }
}

function Assert-TessaraFullValidationDatabaseEnvironment {
    param([Parameter(Mandatory)][Collections.IDictionary]$Environment)

    $endpoints = @(
        foreach ($environmentName in $fullValidationDatabaseEnvironmentNames) {
            $value = $Environment[$environmentName]
            if ($value -isnot [string] -or [string]::IsNullOrWhiteSpace($value)) {
                throw "Full validation requires $environmentName so its database integration tests cannot silently skip."
            }
            ConvertFrom-TessaraValidationDatabaseUrl `
                -EnvironmentName $environmentName `
                -DatabaseUrl $value
        }
    )

    $duplicateIdentity = @(
        $endpoints |
            Group-Object Identity |
            Where-Object Count -gt 1
    ) | Select-Object -First 1
    if ($null -ne $duplicateIdentity) {
        $environmentNames = @(
            $duplicateIdentity.Group |
                ForEach-Object EnvironmentName |
                Sort-Object
        )
        throw "Full validation database URLs must resolve to pairwise-distinct host/port/database identities; duplicate: $($environmentNames -join ', ')."
    }

    if ($Environment["SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET"] -ne
        $destructiveFreshResetAcknowledgement) {
        throw "Full validation requires SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET=$destructiveFreshResetAcknowledgement because the fresh-baseline proof destroys and recreates its dedicated database."
    }

    return $endpoints
}

function Invoke-TessaraValidationPreflightSelfTest {
    function Assert-Rejected {
        param(
            [Parameter(Mandatory)][scriptblock]$Action,
            [Parameter(Mandatory)][string]$ExpectedMessage
        )

        try {
            & $Action
            throw "Expected validation preflight rejection containing '$ExpectedMessage'."
        } catch {
            if (-not $_.Exception.Message.Contains($ExpectedMessage)) {
                throw
            }
        }
    }

    foreach ($accepted in @(
        "tessara_sprint6a_test",
        "tessara_sprint6a_upgrade_test",
        "tessara-clone-01",
        "ROLLBACK_snapshot",
        "tessara-tests-01",
        "tessara_testing_01",
        "tessara-sprint-6a-fresh"
    )) {
        if (-not (Test-TessaraDisposableDatabaseName -DatabaseName $accepted)) {
            throw "Validation preflight self-test rejected disposable database name '$accepted'."
        }
    }
    foreach ($rejected in @(
        "latest",
        "contest",
        "attested",
        "production_upgradeable",
        "sprint6atest",
        "production"
    )) {
        if (Test-TessaraDisposableDatabaseName -DatabaseName $rejected) {
            throw "Validation preflight self-test accepted unsafe database name '$rejected'."
        }
    }

    $validEnvironment = [ordered]@{
        TEST_API_DATABASE_URL = "postgres://tester@127.0.0.1:55432/tessara_test_api"
        TEST_API_FRESH_DATABASE_URL = "postgres://tester@127.0.0.1:55432/tessara_test_api_fresh"
        TEST_REFERENCE_MODULE_DATABASE_URL = "postgres://tester@127.0.0.1:55432/tessara_test_reference_module"
        TEST_API_ENROLLMENT_DATABASE_URL = "postgres://tester@127.0.0.1:55432/tessara_test_api_enrollment"
        SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = $destructiveFreshResetAcknowledgement
    }
    $endpoints = @(
        Assert-TessaraFullValidationDatabaseEnvironment -Environment $validEnvironment
    )
    if ($endpoints.Count -ne $fullValidationDatabaseEnvironmentNames.Count) {
        throw "Validation preflight self-test did not return every required database endpoint."
    }

    $missing = [ordered]@{} + $validEnvironment
    [void]$missing.Remove("TEST_API_ENROLLMENT_DATABASE_URL")
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $missing } `
        -ExpectedMessage "requires TEST_API_ENROLLMENT_DATABASE_URL"

    $duplicate = [ordered]@{} + $validEnvironment
    $duplicate.TEST_API_ENROLLMENT_DATABASE_URL =
        "postgres://other-credentials@127.0.0.1:55432/tessara_test_api"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $duplicate } `
        -ExpectedMessage "pairwise-distinct"

    $unsafe = [ordered]@{} + $validEnvironment
    $unsafe.TEST_API_DATABASE_URL = "postgres://tester@127.0.0.1:55432/production"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $unsafe } `
        -ExpectedMessage "token-bounded disposable marker"

    $wrongScheme = [ordered]@{} + $validEnvironment
    $wrongScheme.TEST_API_DATABASE_URL = "https://127.0.0.1/tessara_test_api"
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $wrongScheme } `
        -ExpectedMessage "postgres:// or postgresql://"

    $missingAcknowledgement = [ordered]@{} + $validEnvironment
    $missingAcknowledgement.SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET = ""
    Assert-Rejected `
        -Action { Assert-TessaraFullValidationDatabaseEnvironment -Environment $missingAcknowledgement } `
        -ExpectedMessage "SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET"

    Write-Host "Full-validation database preflight self-test passed." -ForegroundColor Green
}

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory)]
        [string]$Label,

        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    $startedAt = Get-Date

    & $Command

    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }

    $elapsed = (Get-Date) - $startedAt
    Write-Host ("Passed in {0:mm\:ss}" -f $elapsed) -ForegroundColor Green
}

function Clear-TessaraWebTestArtifacts {
    $isWindowsPlatform = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows
    )

    if (-not $isWindowsPlatform) {
        return
    }

    Write-Host "`n==> Cleaning tessara-web Windows PDB/test artifacts" -ForegroundColor Cyan
    $startedAt = Get-Date

    cargo clean -p tessara-web

    if ($LASTEXITCODE -ne 0) {
        throw "Cleaning tessara-web package failed with exit code $LASTEXITCODE"
    }

    Remove-Item -Force .\target\debug\deps\*.pdb -ErrorAction SilentlyContinue
    Remove-Item -Force .\target\debug\deps\*.exe -ErrorAction SilentlyContinue

    $elapsed = (Get-Date) - $startedAt
    Write-Host ("Cleaned in {0:mm\:ss}" -f $elapsed) -ForegroundColor Green
}

if ($SelfTest) {
    Invoke-TessaraValidationPreflightSelfTest
    return
}

Push-Location $repoRoot
try {
    if ($Fast) {
        Write-Host "Running fast Tessara validation. Use .\scripts\validate.ps1 for the full pre-commit matrix." -ForegroundColor Yellow
    } else {
        Write-Host "Running full Tessara validation sequentially. This avoids Cargo lock contention on Windows." -ForegroundColor Yellow
        $validationEnvironment = [ordered]@{
            SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET =
                $env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET
        }
        foreach ($environmentName in $fullValidationDatabaseEnvironmentNames) {
            $validationEnvironment[$environmentName] =
                [Environment]::GetEnvironmentVariable($environmentName)
        }
        [void](Assert-TessaraFullValidationDatabaseEnvironment `
            -Environment $validationEnvironment)
    }

    Invoke-CheckedStep -Label "Acceptance PowerShell contracts" -Command {
        foreach ($scriptFile in @(Get-ChildItem -Path (Join-Path $repoRoot "scripts") -Filter "*.ps1" -File | Sort-Object FullName)) {
            $relativePath = [IO.Path]::GetRelativePath($repoRoot, $scriptFile.FullName)
            $tokens = $null
            $parseErrors = $null
            [void][Management.Automation.Language.Parser]::ParseFile(
                $scriptFile.FullName,
                [ref]$tokens,
                [ref]$parseErrors
            )
            if ($parseErrors.Count -ne 0) {
                throw "PowerShell AST validation failed for '$relativePath': $($parseErrors[0].Message)"
            }
        }
        & .\scripts\validate.ps1 -SelfTest
        if (-not $?) { throw "validation preflight self-test failed" }
        & .\scripts\local-launch.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "local-launch self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\capture-sprint-6a-deployment-evidence.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "deployment-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\validate-e2e.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Playwright-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\validate-resource-reference-nondisclosure.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "nondisclosure-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\test-sprint-6a-acceptance-evidence.ps1
        if ($LASTEXITCODE -ne 0) { throw "smoke/UAT acceptance-evidence self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\prepare-sprint-7a-uat-fixtures.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7A semantic fixture self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\smoke-sprint-7a.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7A smoke self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\uat-sprint-7a.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7A UAT self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\smoke-sprint-7b.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7B smoke self-test failed with exit code $LASTEXITCODE" }
        & .\scripts\uat-sprint-7b.ps1 -SelfTest
        if ($LASTEXITCODE -ne 0) { throw "Sprint 7B UAT self-test failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Formatting check" -Command {
        cargo fmt --all --check
    }

    Invoke-CheckedStep -Label "Module contract check" -Command {
        cargo check -p tessara-module-contract --locked
    }

    Invoke-CheckedStep -Label "Canonical module SDK boundary audit" -Command {
        & .\scripts\verify-module-sdk-boundaries.ps1
        if ($LASTEXITCODE -ne 0) { throw "module SDK boundary audit failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Canonical module SDK compatibility inventory" -Command {
        & .\scripts\verify-module-sdk-compatibility.ps1
        if ($LASTEXITCODE -ne 0) { throw "module SDK compatibility inventory failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Markdown local links" -Command {
        & .\scripts\verify-markdown-links.ps1
        if ($LASTEXITCODE -ne 0) { throw "Markdown link validation failed with exit code $LASTEXITCODE" }
    }

    Invoke-CheckedStep -Label "Canonical module SDK native checks" -Command {
        cargo check -p tessara-module-runtime -p tessara-module-ui -p tessara-module-testkit --locked
        cargo check -p tessara-reference-module-sdk --features ssr --locked
    }

    Invoke-CheckedStep -Label "API check" -Command {
        cargo check -p tessara-api --locked
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "API SSR check" -Command {
            cargo check -p tessara-api --features ssr --locked
        }
    }

    Invoke-CheckedStep -Label "Web check" -Command {
        cargo check -p tessara-web --locked
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "Web hydrate check" -Command {
            cargo check -p tessara-web --no-default-features --features hydrate --target wasm32-unknown-unknown --locked
        }

        Clear-TessaraWebTestArtifacts
    }

    # Contract tests are database-independent and belong in both validation modes.
    Invoke-CheckedStep -Label "Module contract tests" -Command {
        cargo test -p tessara-module-contract --locked
    }

    Invoke-CheckedStep -Label "Canonical module SDK tests" -Command {
        cargo test -p tessara-module-runtime -p tessara-module-ui -p tessara-module-testkit --locked
        cargo test -p tessara-reference-module-sdk --features ssr --locked
        if ($Fast) {
            cargo test -p tessara-reference-scoped-records --lib --locked
        } else {
            cargo test -p tessara-reference-scoped-records --locked
        }
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "Canonical module SDK WASM checks" -Command {
            cargo check -p tessara-module-contract --target wasm32-unknown-unknown --locked
            cargo check -p tessara-module-ui --no-default-features --features hydrate --target wasm32-unknown-unknown --locked
            cargo check -p tessara-reference-module-sdk --no-default-features --features hydrate --target wasm32-unknown-unknown --locked
        }
    }

    Invoke-CheckedStep -Label "Web tests" -Command {
        cargo test -p tessara-web -j 1 --locked
    }

    Invoke-CheckedStep -Label "API tests" -Command {
        if ($Fast) {
            # Integration targets include database proofs that intentionally
            # fail when their dedicated URLs are absent. Two database proofs
            # live in the library target, so the fast loop names and skips
            # only those tests; the full gate still executes both.
            cargo test -p tessara-api --lib --locked -- `
                --skip core_security::tests::local_enrollment_is_atomic_global_and_idempotent `
                --skip modules::service::tests::catalog_sync_is_repeatable_concurrent_and_rolls_back_injected_failure
        } else {
            cargo test -p tessara-api --all-features --locked
        }
    }

    if (-not $Fast) {
        Invoke-CheckedStep -Label "Release resource-reference timing proof" -Command {
            cargo test -p tessara-api --test modules --release --locked resource_reference_restricted_known_random_latency_profile -- --exact --nocapture
        }
    }

    Write-Host "`nValidation passed." -ForegroundColor Green
} finally {
    Pop-Location
}

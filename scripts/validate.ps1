[CmdletBinding()]
param(
    [switch]$Fast
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

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

Push-Location $repoRoot
try {
    if ($Fast) {
        Write-Host "Running fast Tessara validation. Use .\scripts\validate.ps1 for the full pre-commit matrix." -ForegroundColor Yellow
    } else {
        Write-Host "Running full Tessara validation sequentially. This avoids Cargo lock contention on Windows." -ForegroundColor Yellow
        if ([string]::IsNullOrWhiteSpace($env:TEST_DATABASE_URL)) {
            throw "Full validation requires TEST_DATABASE_URL so database integration tests cannot silently skip. Use -Fast for a non-database development check."
        }
        if ([string]::IsNullOrWhiteSpace($env:SPRINT_6A_FRESH_DATABASE_URL)) {
            throw "Full validation requires SPRINT_6A_FRESH_DATABASE_URL pointing at a second dedicated disposable database. The fresh-baseline proof resets that database and must not share TEST_DATABASE_URL."
        }
        if ($env:SPRINT_6A_FRESH_DATABASE_URL -eq $env:TEST_DATABASE_URL) {
            throw "SPRINT_6A_FRESH_DATABASE_URL must differ from TEST_DATABASE_URL."
        }
        if ($env:SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET -ne "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET") {
            throw "Full validation requires SPRINT_6A_CONFIRM_DESTRUCTIVE_FRESH_RESET=I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET because the fresh-baseline proof destroys and recreates its dedicated database."
        }
    }

    Invoke-CheckedStep -Label "Sprint 6A evidence PowerShell contracts" -Command {
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
            # fail when their dedicated URLs are absent. Fast mode remains a
            # truthful non-database loop by selecting library tests only.
            cargo test -p tessara-api --lib --locked
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

Set-StrictMode -Version Latest

function Assert-Sprint8CCargoTestTranscript {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][AllowEmptyString()][string[]]$OutputLines,
        [ValidateRange(-1, [long]::MaxValue)][long]$ExpectedExecutedTestCount = -1
    )

    $text = ($OutputLines -join "`n") -replace "`e\[[0-9;]*[A-Za-z]", ""
    $matches = [regex]::Matches(
        $text,
        'test result:\s+(?:ok|FAILED)\.\s+(?<passed>\d+)\s+passed;\s+(?<failed>\d+)\s+failed;',
        [Text.RegularExpressions.RegexOptions]::CultureInvariant
    )
    if ($matches.Count -eq 0) {
        throw "Cargo test output contained no parseable test-result summaries."
    }
    [uint64]$executed = 0
    foreach ($match in $matches) {
        $executed += [uint64]$match.Groups['passed'].Value
        $executed += [uint64]$match.Groups['failed'].Value
    }
    if ($executed -eq 0) {
        throw "Cargo test invocation executed zero tests across all result summaries."
    }
    if (
        $ExpectedExecutedTestCount -ge 0 -and
        $executed -ne [uint64]$ExpectedExecutedTestCount
    ) {
        throw "Cargo test invocation executed $executed tests; expected exactly $ExpectedExecutedTestCount."
    }
    [pscustomobject][ordered]@{
        result_summary_count = $matches.Count
        executed_test_count = $executed
    }
}

function Invoke-Sprint8CCheckedCargoTest {
    param(
        [Parameter(Mandatory)][string[]]$Arguments,
        [ValidateRange(-1, [long]::MaxValue)][long]$ExpectedExecutedTestCount = -1
    )

    if ($Arguments.Count -eq 0 -or $Arguments[0] -cne "test") {
        throw "Invoke-Sprint8CCheckedCargoTest accepts only an exact cargo test invocation."
    }
    $lines = [Collections.Generic.List[string]]::new()
    & cargo @Arguments 2>&1 | ForEach-Object {
        $line = [string]$_
        $lines.Add($line)
        Write-Host $line
    }
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) { throw "'cargo $($Arguments -join ' ')' exited $exitCode." }
    Assert-Sprint8CCargoTestTranscript -OutputLines @($lines) `
        -ExpectedExecutedTestCount $ExpectedExecutedTestCount
}

function Test-Sprint8CCargoTestIntegrity {
    $positive = Assert-Sprint8CCargoTestTranscript -OutputLines @(
        "",
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 91 filtered out",
        "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    )
    foreach ($invalid in @(
        @( "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 91 filtered out" ),
        @( "Finished test profile target(s) in 0.01s" )
    )) {
        $rejected = $false
        try { Assert-Sprint8CCargoTestTranscript -OutputLines $invalid | Out-Null } catch {
            $rejected = $true
        }
        if (-not $rejected) {
            throw "Cargo test integrity self-test accepted a zero-test or malformed transcript."
        }
    }
    $exactCountRejected = $false
    try {
        Assert-Sprint8CCargoTestTranscript -OutputLines @(
            "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
        ) -ExpectedExecutedTestCount 4 | Out-Null
    } catch {
        $exactCountRejected = $true
    }
    if (-not $exactCountRejected) {
        throw "Cargo test integrity self-test accepted an incorrect exact test count."
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8c"
        proof = "cargo-test-integrity-self-test"
        state = "passed"
        aggregate_executed_test_count = [uint64]$positive.executed_test_count
        zero_test_rejected = $true
        malformed_transcript_rejected = $true
        incorrect_exact_count_rejected = $true
    }
}

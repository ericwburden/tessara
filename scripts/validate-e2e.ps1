[CmdletBinding()]
param(
    [string]$Spec,
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [switch]$Seed,
    [string[]]$PlaywrightArgs = @(),
    [switch]$DevelopmentMode,
    [string]$AcceptanceManifestPath = "end2end/acceptance-manifest.json",
    [string]$EvidencePath,
    [string]$DeploymentEvidencePath,
    [ValidateSet("upgraded", "fresh")][string]$ExpectedDataState,
    [switch]$OverwriteEvidence,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$endToEndDir = Join-Path $repoRoot "end2end"
$seedScript = Join-Path $PSScriptRoot "seed-demo-data.ps1"
$demoSeedSelfTest = Join-Path $endToEndDir "tests/support/demo-seed.self-test.mjs"
$deploymentEvidenceCommon = Join-Path $PSScriptRoot "sprint-6a-deployment-evidence-common.ps1"
$playwrightEnvironmentNames = @(
    "PLAYWRIGHT_BASE_URL",
    "TESSARA_PLAYWRIGHT_ACCEPTANCE",
    "TESSARA_PLAYWRIGHT_DATA_STATE",
    "PLAYWRIGHT_JSON_OUTPUT_FILE",
    "PLAYWRIGHT_JUNIT_OUTPUT_FILE",
    "PLAYWRIGHT_POSTGRES_CONTAINER",
    "PLAYWRIGHT_POSTGRES_DATABASE",
    "PLAYWRIGHT_POSTGRES_USER"
)
$processEnvironment = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Process)
$callerPlaywrightEnvironment = @{}
foreach ($name in $playwrightEnvironmentNames) {
    $callerPlaywrightEnvironment[$name] = [pscustomobject]@{
        Present = $processEnvironment.Contains($name)
        Value = [Environment]::GetEnvironmentVariable($name, [EnvironmentVariableTarget]::Process)
    }
}

function Remove-ProcessEnvironmentVariable {
    param([Parameter(Mandatory)][string]$Name)

    Remove-Item -LiteralPath "Env:\$Name" -Force -ErrorAction SilentlyContinue
}

function Restore-PlaywrightEnvironment {
    foreach ($name in $playwrightEnvironmentNames) {
        $saved = $callerPlaywrightEnvironment[$name]
        if ($saved.Present) {
            [Environment]::SetEnvironmentVariable(
                $name,
                [string]$saved.Value,
                [EnvironmentVariableTarget]::Process
            )
        } else {
            Remove-ProcessEnvironmentVariable -Name $name
        }
    }
}

function Get-PlaywrightPostgresAcceptanceBinding {
    param([Parameter(Mandatory)][object]$DeploymentEvidence)

    $databaseRuntime = $DeploymentEvidence.snapshot.database_runtime
    if ($null -eq $databaseRuntime) {
        throw "Validated deployment evidence is missing database_runtime."
    }
    $containerProperty = $databaseRuntime.PSObject.Properties['container_id']
    $databaseProperty = $databaseRuntime.PSObject.Properties['current_database']
    $userProperty = $databaseRuntime.PSObject.Properties['database_user']
    if ($null -eq $containerProperty -or
        $containerProperty.Value -isnot [string] -or
        [string]$containerProperty.Value -notmatch '^[0-9a-f]{64}$') {
        throw "Validated deployment evidence contains an invalid PostgreSQL container ID."
    }
    if ($null -eq $databaseProperty -or
        $databaseProperty.Value -isnot [string] -or
        [string]$databaseProperty.Value -notmatch '^[A-Za-z_][A-Za-z0-9_-]*$') {
        throw "Validated deployment evidence contains an invalid PostgreSQL database name."
    }
    if ($null -eq $userProperty -or
        $userProperty.Value -isnot [string] -or
        [string]$userProperty.Value -notmatch '^[A-Za-z_][A-Za-z0-9_-]*$') {
        throw "Validated deployment evidence contains an invalid PostgreSQL user."
    }

    [pscustomobject][ordered]@{
        Container = [string]$containerProperty.Value
        Database = [string]$databaseProperty.Value
        User = [string]$userProperty.Value
    }
}

function Set-PlaywrightPostgresAcceptanceBinding {
    param([Parameter(Mandatory)][object]$DeploymentEvidence)

    $binding = Get-PlaywrightPostgresAcceptanceBinding -DeploymentEvidence $DeploymentEvidence
    $env:PLAYWRIGHT_POSTGRES_CONTAINER = $binding.Container
    $env:PLAYWRIGHT_POSTGRES_DATABASE = $binding.Database
    $env:PLAYWRIGHT_POSTGRES_USER = $binding.User
    $binding
}

function Assert-PlaywrightDeploymentEvidenceDigestStable {
    param(
        [Parameter(Mandatory)][string]$InitialSha256,
        [Parameter(Mandatory)][string]$FinalSha256
    )

    if ($InitialSha256 -cnotmatch '^[0-9a-f]{64}$' -or
        $FinalSha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw "Playwright deployment-evidence digests must be exact lowercase SHA-256 values."
    }
    if ($FinalSha256 -cne $InitialSha256) {
        throw "Deployment evidence changed during Playwright acceptance; publication is forbidden."
    }
}

function Publish-PlaywrightArtifactSet {
    param(
        [Parameter(Mandatory)][object[]]$Artifacts,
        [switch]$AllowOverwrite
    )

    $states = @($Artifacts | ForEach-Object {
        [pscustomobject]@{
            TemporaryPath = [IO.Path]::GetFullPath([string]$_.TemporaryPath)
            FinalPath = [IO.Path]::GetFullPath([string]$_.FinalPath)
            BackupPath = "$([IO.Path]::GetFullPath([string]$_.FinalPath)).backup-$([guid]::NewGuid().ToString('N'))"
            FinalExisted = Test-Path -LiteralPath ([string]$_.FinalPath) -PathType Leaf
            BackedUp = $false
            Published = $false
        }
    })
    if ($states.Count -eq 0) {
        throw "At least one Playwright artifact is required for publication."
    }

    $finalPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($state in $states) {
        if (-not (Test-Path -LiteralPath $state.TemporaryPath -PathType Leaf)) {
            throw "Validated Playwright artifact '$($state.TemporaryPath)' is missing before publication."
        }
        if (-not $finalPaths.Add($state.FinalPath)) {
            throw "Playwright artifact publication repeats final path '$($state.FinalPath)'."
        }
    }
    if (@($states | Where-Object FinalExisted).Count -gt 0 -and -not $AllowOverwrite) {
        throw "Retained Playwright acceptance evidence already exists. Refusing to replace it without -OverwriteEvidence."
    }

    $publishCompleted = $false
    try {
        foreach ($state in $states) {
            if ($state.FinalExisted) {
                [IO.File]::Move($state.FinalPath, $state.BackupPath)
                $state.BackedUp = $true
            }
        }
        foreach ($state in $states) {
            [IO.File]::Move($state.TemporaryPath, $state.FinalPath)
            $state.Published = $true
        }
        $publishCompleted = $true
    } catch {
        $publicationFailure = $_.Exception.Message
        $restoreFailures = [Collections.Generic.List[string]]::new()
        for ($index = $states.Count - 1; $index -ge 0; $index--) {
            $state = $states[$index]
            if ($state.Published -and (Test-Path -LiteralPath $state.FinalPath)) {
                try {
                    Remove-Item -LiteralPath $state.FinalPath -Force
                } catch {
                    $restoreFailures.Add("could not remove new '$($state.FinalPath)': $($_.Exception.Message)")
                }
            }
        }
        for ($index = $states.Count - 1; $index -ge 0; $index--) {
            $state = $states[$index]
            if ($state.BackedUp -and (Test-Path -LiteralPath $state.BackupPath)) {
                try {
                    [IO.File]::Move($state.BackupPath, $state.FinalPath)
                    $state.BackedUp = $false
                } catch {
                    $restoreFailures.Add("could not restore '$($state.BackupPath)': $($_.Exception.Message)")
                }
            }
        }
        if ($restoreFailures.Count -gt 0) {
            throw "Playwright artifact publication failed ('$publicationFailure') and rollback was incomplete. Backup files were retained. $($restoreFailures -join '; ')"
        }
        throw
    } finally {
        if ($publishCompleted) {
            foreach ($state in $states) {
                if (Test-Path -LiteralPath $state.BackupPath) {
                    Remove-Item -LiteralPath $state.BackupPath -Force -ErrorAction SilentlyContinue
                }
            }
        }
    }
}

function Resolve-RepositoryPath {
    param([Parameter(Mandatory)][string]$Path)

    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    return [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Get-ManifestPlaywrightTestPaths {
    param(
        [Parameter(Mandatory)][object]$Manifest,
        [Parameter(Mandatory)][string]$TestsRoot
    )

    if ($Manifest.PSObject.Properties['files'] -eq $null -or
        $Manifest.files -isnot [array] -or
        @($Manifest.files).Count -lt 1) {
        throw "The Playwright acceptance manifest files must be a nonempty JSON array."
    }

    $testsRootFullPath = [IO.Path]::GetFullPath($TestsRoot).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $testsRootPrefix = "$testsRootFullPath$([IO.Path]::DirectorySeparatorChar)"
    $paths = [Collections.Generic.List[string]]::new()
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($Manifest.files)) {
        if ($null -eq $entry -or $entry.PSObject.Properties['path'] -eq $null -or
            $entry.path -isnot [string]) {
            throw "The Playwright acceptance manifest contains an invalid file entry."
        }
        $path = [string]$entry.path
        if ([string]::IsNullOrWhiteSpace($path) -or
            $path -cne $path.Trim() -or
            $path -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*\.spec\.ts$' -or
            $path.Contains('..') -or
            $path.Contains('\')) {
            throw "The Playwright acceptance manifest contains invalid file path '$path'."
        }

        $fullPath = [IO.Path]::GetFullPath((Join-Path $testsRootFullPath $path))
        if (-not $fullPath.StartsWith($testsRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "The Playwright acceptance manifest file path '$path' escapes the tests directory."
        }
        if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
            throw "The Playwright acceptance manifest names a missing test file '$path'."
        }
        if ($seen.Add($path)) {
            $paths.Add("tests/$path")
        }
    }
    return @($paths)
}

function Get-EvidenceSiblingPath {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Suffix
    )

    $directory = [IO.Path]::GetDirectoryName($Path)
    $stem = [IO.Path]::GetFileNameWithoutExtension($Path)
    return Join-Path $directory "$stem$Suffix"
}

function Add-PlaywrightSuiteCounts {
    param(
        [Parameter(Mandatory)][object]$Suite,
        [Parameter(Mandatory)][Collections.Generic.Dictionary[string, int]]$Counts,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.HashSet[string]]$Identities,
        [Parameter(Mandatory)][ref]$Total,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ParentTitles,
        [Parameter(Mandatory)][AllowEmptyString()][string]$InheritedFile,
        [switch]$RequireResults,
        [Parameter(Mandatory)][ref]$RetriedResults,
        [Parameter(Mandatory)][ref]$NonPassingResults,
        [Parameter(Mandatory)][ref]$NonPassingExpectations
    )

    $suiteFile = if ($null -eq $Suite.PSObject.Properties['file']) { '' } else { [string]$Suite.file }
    $file = if ([string]::IsNullOrWhiteSpace($suiteFile)) {
        $InheritedFile
    } else {
        $suiteFile
    }
    $suiteTitles = @($ParentTitles)
    $suiteTitle = if ($null -eq $Suite.PSObject.Properties['title']) { '' } else { [string]$Suite.title }
    if (-not [string]::IsNullOrWhiteSpace($suiteTitle) -and $suiteTitle -cne $file) {
        $suiteTitles += $suiteTitle
    }
    $suiteSpecs = if ($null -eq $Suite.PSObject.Properties['specs']) { @() } else { @($Suite.specs) }
    foreach ($spec in $suiteSpecs) {
        if ($null -eq $spec) {
            continue
        }
        $specTitle = if ($null -eq $spec.PSObject.Properties['title']) { '' } else { [string]$spec.title }
        if ([string]::IsNullOrWhiteSpace($specTitle)) {
            throw "Playwright report contains a test without a title."
        }
        $fullTitle = (@($suiteTitles) + @($specTitle)) -join " › "
        $testCount = 0
        $specTests = if ($null -eq $spec.PSObject.Properties['tests']) { @() } else { @($spec.tests) }
        foreach ($test in $specTests) {
            if ($null -eq $test) {
                continue
            }
            $testCount += 1
            $projectName = if ($null -eq $test.PSObject.Properties['projectName']) { '' } else { [string]$test.projectName }
            $identity = if ([string]::IsNullOrWhiteSpace($projectName)) {
                "$file :: $fullTitle"
            } else {
                "[$projectName] $file :: $fullTitle"
            }
            if (-not $Identities.Add($identity)) {
                throw "Playwright report repeats full test identity '$identity'."
            }

            $testResults = if ($null -eq $test.PSObject.Properties['results']) { @() } else { @($test.results) }
            if ($RequireResults -and @($testResults).Count -ne 1) {
                $NonPassingResults.Value += 1
            }
            $expectedStatus = if ($null -eq $test.PSObject.Properties['expectedStatus']) { '' } else { [string]$test.expectedStatus }
            if ($expectedStatus -ne "passed") {
                $NonPassingExpectations.Value += 1
            }
            foreach ($result in $testResults) {
                if ($null -eq $result) {
                    continue
                }
                $retry = if ($null -eq $result.PSObject.Properties['retry']) { -1 } else { [int]$result.retry }
                $status = if ($null -eq $result.PSObject.Properties['status']) { '' } else { [string]$result.status }
                if ($retry -ne 0) {
                    $RetriedResults.Value += 1
                }
                if ($RequireResults -and $status -ne "passed") {
                    $NonPassingResults.Value += 1
                }
            }
        }
        if ([string]::IsNullOrWhiteSpace($file) -and $testCount -gt 0) {
            throw "Playwright report contains tests without a source file."
        }
        if ($testCount -gt 0) {
            if (-not $Counts.ContainsKey($file)) {
                $Counts.Add($file, 0)
            }
            $Counts[$file] += $testCount
            $Total.Value += $testCount
        }
    }

    $childSuites = if ($null -eq $Suite.PSObject.Properties['suites']) { @() } else { @($Suite.suites) }
    foreach ($child in $childSuites) {
        if ($null -eq $child) {
            continue
        }
        Add-PlaywrightSuiteCounts `
            -Suite $child `
            -Counts $Counts `
            -Identities $Identities `
            -Total $Total `
            -ParentTitles $suiteTitles `
            -InheritedFile $file `
            -RequireResults:$RequireResults `
            -RetriedResults $RetriedResults `
            -NonPassingResults $NonPassingResults `
            -NonPassingExpectations $NonPassingExpectations
    }
}

function Read-PlaywrightReport {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$RequireResults
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Playwright did not produce the required report '$Path'."
    }
    $report = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    $counts = [Collections.Generic.Dictionary[string, int]]::new([StringComparer]::Ordinal)
    $identities = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $total = 0
    $retriedResults = 0
    $nonPassingResults = 0
    $nonPassingExpectations = 0
    foreach ($suite in @($report.suites)) {
        Add-PlaywrightSuiteCounts `
            -Suite $suite `
            -Counts $counts `
            -Identities $identities `
            -Total ([ref]$total) `
            -ParentTitles @() `
            -InheritedFile '' `
            -RequireResults:$RequireResults `
            -RetriedResults ([ref]$retriedResults) `
            -NonPassingResults ([ref]$nonPassingResults) `
            -NonPassingExpectations ([ref]$nonPassingExpectations)
    }
    return [pscustomobject]@{
        Report = $report
        Counts = $counts
        Identities = $identities
        Total = $total
        RetriedResults = $retriedResults
        NonPassingResults = $nonPassingResults
        NonPassingExpectations = $nonPassingExpectations
    }
}

function Assert-ExactJsonProperties {
    param(
        [Parameter(Mandatory)][object]$Object,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Context
    )

    $actual = @($Object.PSObject.Properties.Name | Sort-Object)
    $expectedSorted = @($Expected | Sort-Object)
    if (($actual -join "`n") -cne ($expectedSorted -join "`n")) {
        throw "$Context must contain exactly: $($Expected -join ', ')."
    }
}

function Assert-ExpectedInventory {
    param(
        [Parameter(Mandatory)][object]$Manifest,
        [Parameter(Mandatory)][Collections.Generic.Dictionary[string, int]]$ActualCounts,
        [Parameter(Mandatory)][Collections.Generic.HashSet[string]]$ActualIdentities,
        [Parameter(Mandatory)][int]$ActualTotal,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-ExactJsonProperties `
        -Object $Manifest `
        -Expected @('schema_version', 'expected_total', 'files') `
        -Context 'The Playwright acceptance manifest'
    if (($Manifest.schema_version -isnot [long] -and $Manifest.schema_version -isnot [int]) -or
        [int]$Manifest.schema_version -ne 2) {
        throw "Unsupported Playwright acceptance manifest schema '$($Manifest.schema_version)'."
    }
    if (($Manifest.expected_total -isnot [long] -and $Manifest.expected_total -isnot [int]) -or
        [int]$Manifest.expected_total -lt 1) {
        throw "The Playwright acceptance manifest expected_total must be a positive JSON integer."
    }
    if ($Manifest.files -isnot [array] -or @($Manifest.files).Count -lt 1) {
        throw "The Playwright acceptance manifest files must be a nonempty JSON array."
    }

    $expectedCounts = [Collections.Generic.Dictionary[string, int]]::new([StringComparer]::Ordinal)
    $expectedIdentities = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($Manifest.files)) {
        if ($null -eq $entry) {
            throw "The Playwright acceptance manifest contains an invalid file entry."
        }
        Assert-ExactJsonProperties `
            -Object $entry `
            -Expected @('path', 'tests') `
            -Context 'Each Playwright acceptance manifest file entry'
        if ($entry.path -isnot [string]) {
            throw "The Playwright acceptance manifest file path must be a JSON string."
        }
        $path = [string]$entry.path
        if ([string]::IsNullOrWhiteSpace($path) -or
            $path -cne $path.Trim() -or
            $path -notmatch '^[A-Za-z0-9][A-Za-z0-9._/-]*\.spec\.ts$' -or
            $path.Contains('..') -or
            $path.Contains('\')) {
            throw "The Playwright acceptance manifest contains invalid file path '$path'."
        }
        if ($entry.tests -isnot [array] -or @($entry.tests).Count -lt 1) {
            throw "The Playwright acceptance manifest file '$path' must list at least one full test title."
        }
        if (-not $expectedCounts.TryAdd($path, @($entry.tests).Count)) {
            throw "The Playwright acceptance manifest repeats '$path'."
        }
        foreach ($titleValue in @($entry.tests)) {
            if ($titleValue -isnot [string]) {
                throw "The Playwright acceptance manifest file '$path' contains a non-string test title."
            }
            $title = [string]$titleValue
            if ([string]::IsNullOrWhiteSpace($title) -or
                $title -cne $title.Trim() -or
                $title -match '[\x00-\x1f]' -or
                $title.Contains(' :: ')) {
                throw "The Playwright acceptance manifest file '$path' contains invalid full test title '$title'."
            }
            $identity = "$path :: $title"
            if (-not $expectedIdentities.Add($identity)) {
                throw "The Playwright acceptance manifest repeats full test identity '$identity'."
            }
        }
    }

    if ([int]$Manifest.expected_total -ne $expectedIdentities.Count) {
        throw "The Playwright acceptance manifest expected_total is $($Manifest.expected_total), but it lists $($expectedIdentities.Count) exact identities."
    }
    if ([int]$Manifest.expected_total -ne $ActualTotal) {
        throw "$Label discovered $ActualTotal tests; the durable acceptance manifest requires $($Manifest.expected_total)."
    }

    $expectedRows = @($expectedCounts.GetEnumerator() | Sort-Object Key | ForEach-Object { "$($_.Key)=$($_.Value)" })
    $actualRows = @($ActualCounts.GetEnumerator() | Sort-Object Key | ForEach-Object { "$($_.Key)=$($_.Value)" })
    if (($expectedRows -join "`n") -cne ($actualRows -join "`n")) {
        throw "$Label test-file inventory differs from the durable acceptance manifest.`nExpected:`n$($expectedRows -join "`n")`nActual:`n$($actualRows -join "`n")"
    }
    $expectedIdentityRows = @($expectedIdentities | Sort-Object)
    $actualIdentityRows = @($ActualIdentities | Sort-Object)
    if (($expectedIdentityRows -join "`n") -cne ($actualIdentityRows -join "`n")) {
        throw "$Label full test identities differ from the durable acceptance manifest.`nExpected:`n$($expectedIdentityRows -join "`n")`nActual:`n$($actualIdentityRows -join "`n")"
    }
}

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory)]
        [string]$Label,
        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "`n==> $Label" -ForegroundColor Cyan
    & $Command

    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Assert-AppIsReachable {
    param(
        [Parameter(Mandatory)]
        [string]$Uri
    )

    try {
        $response = Invoke-WebRequest -Uri "$Uri/health" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -ne 200) {
            throw "Expected HTTP 200 from $Uri/health, received $($response.StatusCode)."
        }
    } catch {
        throw "Tessara is not reachable at $Uri. Start it with .\scripts\local-launch.ps1 before running e2e validation."
    }
}

if ($SelfTest) {
    function Assert-SelfTestRejects {
        param(
            [Parameter(Mandatory)][scriptblock]$Action,
            [Parameter(Mandatory)][string]$Context
        )

        $rejected = $false
        try {
            & $Action
        } catch {
            $rejected = $true
        }
        if (-not $rejected) {
            throw "Self-test failed: $Context was accepted."
        }
    }

    $inventoryManifest = @'
{"schema_version":2,"expected_total":2,"files":[{"path":"alpha.spec.ts","tests":["outer › first","outer › second"]}]}
'@ | ConvertFrom-Json
    $inventoryCounts = [Collections.Generic.Dictionary[string, int]]::new([StringComparer]::Ordinal)
    $inventoryCounts.Add('alpha.spec.ts', 2)
    $inventoryIdentities = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    [void]$inventoryIdentities.Add('alpha.spec.ts :: outer › first')
    [void]$inventoryIdentities.Add('alpha.spec.ts :: outer › second')
    Assert-ExpectedInventory `
        -Manifest $inventoryManifest `
        -ActualCounts $inventoryCounts `
        -ActualIdentities $inventoryIdentities `
        -ActualTotal 2 `
        -Label 'Self-test discovery'

    $renamedManifest = @'
{"schema_version":2,"expected_total":2,"files":[{"path":"alpha.spec.ts","tests":["outer › first","outer › renamed"]}]}
'@ | ConvertFrom-Json
    Assert-SelfTestRejects `
        -Context 'a renamed test with unchanged per-file and total counts' `
        -Action {
            Assert-ExpectedInventory `
                -Manifest $renamedManifest `
                -ActualCounts $inventoryCounts `
                -ActualIdentities $inventoryIdentities `
                -ActualTotal 2 `
                -Label 'Self-test discovery'
        }
    $schemaOneManifest = @'
{"schema_version":1,"expected_total":2,"files":[{"path":"alpha.spec.ts","tests":["outer › first","outer › second"]}]}
'@ | ConvertFrom-Json
    Assert-SelfTestRejects `
        -Context 'the count-only manifest schema' `
        -Action {
            Assert-ExpectedInventory `
                -Manifest $schemaOneManifest `
                -ActualCounts $inventoryCounts `
                -ActualIdentities $inventoryIdentities `
                -ActualTotal 2 `
                -Label 'Self-test discovery'
        }
    $wrongPathTypeManifest = @'
{"schema_version":2,"expected_total":2,"files":[{"path":["alpha.spec.ts"],"tests":["outer › first","outer › second"]}]}
'@ | ConvertFrom-Json
    Assert-SelfTestRejects `
        -Context 'a non-string manifest file path' `
        -Action {
            Assert-ExpectedInventory `
                -Manifest $wrongPathTypeManifest `
                -ActualCounts $inventoryCounts `
                -ActualIdentities $inventoryIdentities `
                -ActualTotal 2 `
                -Label 'Self-test discovery'
        }
    $duplicateManifest = @'
{"schema_version":2,"expected_total":2,"files":[{"path":"alpha.spec.ts","tests":["outer › first","outer › first"]}]}
'@ | ConvertFrom-Json
    Assert-SelfTestRejects `
        -Context 'duplicate full test identities' `
        -Action {
            Assert-ExpectedInventory `
                -Manifest $duplicateManifest `
                -ActualCounts $inventoryCounts `
                -ActualIdentities $inventoryIdentities `
                -ActualTotal 2 `
                -Label 'Self-test discovery'
        }

    $publishTestRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-playwright-evidence-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($publishTestRoot) | Out-Null
    try {
        $testArtifacts = @(1..4 | ForEach-Object {
            $temporaryPath = Join-Path $publishTestRoot "temporary-$_.artifact"
            $finalPath = Join-Path $publishTestRoot "final-$_.artifact"
            [IO.File]::WriteAllText($temporaryPath, "new-$_")
            [IO.File]::WriteAllText($finalPath, "old-$_")
            [pscustomobject]@{ TemporaryPath = $temporaryPath; FinalPath = $finalPath }
        })
        $overwriteRejected = $false
        try {
            Publish-PlaywrightArtifactSet -Artifacts $testArtifacts
        } catch {
            $overwriteRejected = $true
        }
        if (-not $overwriteRejected) {
            throw "Self-test failed: retained Playwright evidence was replaced without explicit authorization."
        }
        foreach ($index in 1..4) {
            if ((Get-Content -LiteralPath (Join-Path $publishTestRoot "final-$index.artifact") -Raw) -cne "old-$index" -or
                (Get-Content -LiteralPath (Join-Path $publishTestRoot "temporary-$index.artifact") -Raw) -cne "new-$index") {
                throw "Self-test failed: overwrite refusal did not preserve artifact $index."
            }
        }

        Publish-PlaywrightArtifactSet -Artifacts $testArtifacts -AllowOverwrite
        foreach ($index in 1..4) {
            if ((Get-Content -LiteralPath (Join-Path $publishTestRoot "final-$index.artifact") -Raw) -cne "new-$index") {
                throw "Self-test failed: validated Playwright artifact $index was not published."
            }
        }

        $sharedRollbackPath = Join-Path $publishTestRoot "rollback-shared.artifact"
        [IO.File]::WriteAllText($sharedRollbackPath, "replacement")
        $rollbackThirdPath = Join-Path $publishTestRoot "rollback-third.artifact"
        $rollbackFourthPath = Join-Path $publishTestRoot "rollback-fourth.artifact"
        [IO.File]::WriteAllText($rollbackThirdPath, "replacement-3")
        [IO.File]::WriteAllText($rollbackFourthPath, "replacement-4")
        $rollbackArtifacts = @(
            [pscustomobject]@{ TemporaryPath = $sharedRollbackPath; FinalPath = (Join-Path $publishTestRoot "final-1.artifact") },
            [pscustomobject]@{ TemporaryPath = $sharedRollbackPath; FinalPath = (Join-Path $publishTestRoot "final-2.artifact") },
            [pscustomobject]@{ TemporaryPath = $rollbackThirdPath; FinalPath = (Join-Path $publishTestRoot "final-3.artifact") },
            [pscustomobject]@{ TemporaryPath = $rollbackFourthPath; FinalPath = (Join-Path $publishTestRoot "final-4.artifact") }
        )
        $publicationFailed = $false
        try {
            Publish-PlaywrightArtifactSet -Artifacts $rollbackArtifacts -AllowOverwrite
        } catch {
            $publicationFailed = $true
        }
        if (-not $publicationFailed) {
            throw "Self-test failed: the injected partial-publication failure did not fail."
        }
        foreach ($index in 1..4) {
            if ((Get-Content -LiteralPath (Join-Path $publishTestRoot "final-$index.artifact") -Raw) -cne "new-$index") {
                throw "Self-test failed: rollback did not preserve retained Playwright artifact $index."
            }
        }
        $prepublicationArtifacts = @()
        foreach ($index in 1..4) {
            $temporaryPath = Join-Path $publishTestRoot "prepublication-$index.artifact"
            [IO.File]::WriteAllText($temporaryPath, "forbidden-$index")
            $prepublicationArtifacts += [pscustomobject]@{
                TemporaryPath = $temporaryPath
                FinalPath = Join-Path $publishTestRoot "final-$index.artifact"
            }
        }
        Assert-SelfTestRejects `
            -Context 'changed deployment evidence before Playwright publication' `
            -Action {
                Assert-PlaywrightDeploymentEvidenceDigestStable `
                    -InitialSha256 ('1' * 64) `
                    -FinalSha256 ('2' * 64)
                Publish-PlaywrightArtifactSet -Artifacts $prepublicationArtifacts -AllowOverwrite
            }
        foreach ($index in 1..4) {
            if ((Get-Content -LiteralPath (Join-Path $publishTestRoot "final-$index.artifact") -Raw) -cne "new-$index") {
                throw "Self-test failed: final deployment revalidation failure changed retained Playwright artifact $index."
            }
        }

        foreach ($name in @(
            'PLAYWRIGHT_POSTGRES_CONTAINER',
            'PLAYWRIGHT_POSTGRES_DATABASE',
            'PLAYWRIGHT_POSTGRES_USER'
        )) {
            [Environment]::SetEnvironmentVariable(
                $name,
                "sprint-6a-stale-$name",
                [EnvironmentVariableTarget]::Process
            )
        }
        $bindingEvidence = [pscustomobject]@{
            snapshot = [pscustomobject]@{
                database_runtime = [pscustomobject]@{
                    container_id = 'a' * 64
                    current_database = 'tessara_sprint6a_upgrade_test'
                    database_user = 'tessara'
                }
            }
        }
        $binding = Set-PlaywrightPostgresAcceptanceBinding -DeploymentEvidence $bindingEvidence
        if ($binding.Container -cne ('a' * 64) -or
            $binding.Database -cne 'tessara_sprint6a_upgrade_test' -or
            $binding.User -cne 'tessara' -or
            $env:PLAYWRIGHT_POSTGRES_CONTAINER -cne ('a' * 64) -or
            $env:PLAYWRIGHT_POSTGRES_DATABASE -cne 'tessara_sprint6a_upgrade_test' -or
            $env:PLAYWRIGHT_POSTGRES_USER -cne 'tessara') {
            throw "Self-test failed: acceptance PostgreSQL binding was not derived exactly from deployment evidence."
        }
        foreach ($malformedRuntime in @(
            [pscustomobject]@{
                container_id = 'a' * 64
                current_database = 'tessara_sprint6a_upgrade_test'
            },
            [pscustomobject]@{
                container_id = @('a' * 64)
                current_database = 'tessara_sprint6a_upgrade_test'
                database_user = 'tessara'
            },
            [pscustomobject]@{
                container_id = 'a' * 64
                current_database = '../upgrade'
                database_user = 'tessara'
            },
            [pscustomobject]@{
                container_id = 'a' * 64
                current_database = 'tessara_sprint6a_upgrade_test'
                database_user = 7
            }
        )) {
            $malformedEvidence = [pscustomobject]@{
                snapshot = [pscustomobject]@{ database_runtime = $malformedRuntime }
            }
            Assert-SelfTestRejects `
                -Context 'a missing, incomplete, or malformed acceptance PostgreSQL binding' `
                -Action {
                    Set-PlaywrightPostgresAcceptanceBinding -DeploymentEvidence $malformedEvidence
                }
        }
        if ($env:PLAYWRIGHT_POSTGRES_CONTAINER -cne ('a' * 64) -or
            $env:PLAYWRIGHT_POSTGRES_DATABASE -cne 'tessara_sprint6a_upgrade_test' -or
            $env:PLAYWRIGHT_POSTGRES_USER -cne 'tessara') {
            throw "Self-test failed: malformed acceptance PostgreSQL binding changed the last valid environment."
        }

        foreach ($name in $playwrightEnvironmentNames) {
            [Environment]::SetEnvironmentVariable(
                $name,
                "sprint-6a-self-test-$name",
                [EnvironmentVariableTarget]::Process
            )
        }
        Restore-PlaywrightEnvironment
        $restoredEnvironment = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Process)
        foreach ($name in $playwrightEnvironmentNames) {
            $saved = $callerPlaywrightEnvironment[$name]
            if ($restoredEnvironment.Contains($name) -ne $saved.Present -or
                ($saved.Present -and
                    [string]$restoredEnvironment[$name] -cne [string]$saved.Value)) {
                throw "Self-test failed: process environment '$name' was not restored exactly."
            }
        }
        if (-not (Test-Path -LiteralPath $demoSeedSelfTest -PathType Leaf)) {
            throw "Self-test failed: Playwright demo seed guard self-test is missing at '$demoSeedSelfTest'."
        }
        $nodeCommands = @(Get-Command node -CommandType Application -ErrorAction Stop)
        if ($nodeCommands.Count -ne 1) {
            throw "Self-test requires one unambiguous Node.js executable; found $($nodeCommands.Count)."
        }
        Invoke-CheckedStep -Label "Validating Playwright upgraded demo-seed guard and endpoint inventory" -Command {
            & $nodeCommands[0].Source --no-warnings $demoSeedSelfTest
        }
    } finally {
        Restore-PlaywrightEnvironment
        Remove-Item -LiteralPath $publishTestRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
    Write-Host "Playwright evidence retention self-test passed." -ForegroundColor Green
    return
}

if (-not (Test-Path $endToEndDir)) {
    throw "Could not find end2end directory at $endToEndDir"
}

if ($Seed -and -not (Test-Path $seedScript)) {
    throw "Could not find seed helper at $seedScript"
}

if (-not $DevelopmentMode -and (-not [string]::IsNullOrWhiteSpace($Spec) -or $PlaywrightArgs.Count -gt 0)) {
    throw "Acceptance validation always runs the complete unfiltered suite. Use -DevelopmentMode for a targeted Spec or PlaywrightArgs run; that run is not acceptance evidence."
}
if (-not $DevelopmentMode -and $Seed) {
    throw "Playwright acceptance cannot use -Seed because seeding must finish before deployment evidence is captured. -Seed is DevelopmentMode-only."
}
if (-not $DevelopmentMode) {
    if ([string]::IsNullOrWhiteSpace($DeploymentEvidencePath) -or [string]::IsNullOrWhiteSpace($ExpectedDataState)) {
        throw "Playwright acceptance requires -DeploymentEvidencePath and -ExpectedDataState upgraded|fresh. Use -DevelopmentMode only for targeted non-acceptance diagnostics."
    }
    if (-not (Test-Path -LiteralPath $deploymentEvidenceCommon -PathType Leaf)) {
        throw "Could not find Sprint 6A deployment evidence validator at $deploymentEvidenceCommon"
    }
    . $deploymentEvidenceCommon
}

$resolvedEvidencePath = if ([string]::IsNullOrWhiteSpace($EvidencePath)) {
    if ($DevelopmentMode) {
        "artifacts/sprint-6a/playwright-development.json"
    } else {
        "artifacts/sprint-6a/playwright-acceptance-$ExpectedDataState.json"
    }
} else {
    $EvidencePath
}
$manifestFullPath = Resolve-RepositoryPath $AcceptanceManifestPath
$evidenceFullPath = Resolve-RepositoryPath $resolvedEvidencePath
$discoveryPath = Get-EvidenceSiblingPath -Path $evidenceFullPath -Suffix ".discovery.json"
$junitPath = Get-EvidenceSiblingPath -Path $evidenceFullPath -Suffix ".xml"
$summaryPath = Get-EvidenceSiblingPath -Path $evidenceFullPath -Suffix ".summary.json"
$deploymentEvidence = $null
$deploymentEvidenceSha256 = $null
$deploymentEvidenceFullPath = if ([string]::IsNullOrWhiteSpace($DeploymentEvidencePath)) {
    $null
} else {
    Resolve-RepositoryPath $DeploymentEvidencePath
}
if (-not $DevelopmentMode) {
    $protectedPaths = @(
        $manifestFullPath,
        $evidenceFullPath,
        $discoveryPath,
        $junitPath,
        $summaryPath,
        $deploymentEvidenceFullPath,
        "$deploymentEvidenceFullPath.sha256"
    )
    $distinctPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($path in $protectedPaths) {
        if (-not $distinctPaths.Add([IO.Path]::GetFullPath($path))) {
            throw "Playwright acceptance input/output paths must be pairwise distinct; collision at '$path'."
        }
    }

    $existingArtifacts = @(
        @($evidenceFullPath, $discoveryPath, $junitPath, $summaryPath) |
            Where-Object { Test-Path -LiteralPath $_ -PathType Leaf }
    )
    if ($existingArtifacts.Count -gt 0 -and -not $OverwriteEvidence) {
        throw "Retained Playwright acceptance evidence already exists. Refusing to replace it without -OverwriteEvidence: $($existingArtifacts -join ', ')"
    }
}

$temporaryArtifactPaths = @()
$temporaryArtifactDirectory = $null

Push-Location $repoRoot
try {
    Assert-AppIsReachable -Uri $BaseUrl.TrimEnd("/")

    if (-not $DevelopmentMode) {
        $deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
            -RepositoryRoot $repoRoot `
            -EvidencePath $deploymentEvidenceFullPath `
            -BaseUrl $BaseUrl `
            -ExpectedDataState $ExpectedDataState
        $null = Set-PlaywrightPostgresAcceptanceBinding -DeploymentEvidence $deploymentEvidence
        $env:TESSARA_PLAYWRIGHT_DATA_STATE = $ExpectedDataState
        $deploymentEvidenceSha256 = (Get-FileHash -LiteralPath $deploymentEvidenceFullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    if ($Seed) {
        Invoke-CheckedStep -Label "Ensuring UAT demo data" -Command {
            & $seedScript
        }
    }

    $env:PLAYWRIGHT_BASE_URL = $BaseUrl.TrimEnd("/")
    if ($DevelopmentMode) {
        Remove-ProcessEnvironmentVariable -Name "TESSARA_PLAYWRIGHT_ACCEPTANCE"
        Remove-ProcessEnvironmentVariable -Name "TESSARA_PLAYWRIGHT_DATA_STATE"
        Remove-ProcessEnvironmentVariable -Name "PLAYWRIGHT_JSON_OUTPUT_FILE"
        Remove-ProcessEnvironmentVariable -Name "PLAYWRIGHT_JUNIT_OUTPUT_FILE"
        Write-Warning "DevelopmentMode permits filtering and does not produce Sprint acceptance evidence."
        Invoke-CheckedStep -Label "Running targeted Playwright developer tests" -Command {
            if ([string]::IsNullOrWhiteSpace($Spec)) {
                npm --prefix $endToEndDir test -- @PlaywrightArgs
            } else {
                npm --prefix $endToEndDir test -- $Spec @PlaywrightArgs
            }
        }
        return
    }

    if (-not (Test-Path -LiteralPath $manifestFullPath -PathType Leaf)) {
        throw "Could not find durable Playwright acceptance manifest at $manifestFullPath"
    }
    $manifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    $manifestTestPaths = @(Get-ManifestPlaywrightTestPaths `
        -Manifest $manifest `
        -TestsRoot (Join-Path $endToEndDir "tests"))
    $evidenceDirectory = [IO.Path]::GetDirectoryName($evidenceFullPath)
    [IO.Directory]::CreateDirectory($evidenceDirectory) | Out-Null
    $temporaryArtifactDirectory = Join-Path `
        $evidenceDirectory `
        ".playwright-acceptance-$ExpectedDataState-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($temporaryArtifactDirectory) | Out-Null
    $temporaryEvidencePath = Join-Path $temporaryArtifactDirectory "execution.json"
    $temporaryDiscoveryPath = Join-Path $temporaryArtifactDirectory "discovery.json"
    $temporaryJunitPath = Join-Path $temporaryArtifactDirectory "execution.xml"
    $temporarySummaryPath = Join-Path $temporaryArtifactDirectory "summary.json"
    $temporaryArtifactPaths = @(
        $temporaryEvidencePath,
        $temporaryDiscoveryPath,
        $temporaryJunitPath,
        $temporarySummaryPath
    )

    $env:TESSARA_PLAYWRIGHT_ACCEPTANCE = "1"
    $env:PLAYWRIGHT_JSON_OUTPUT_FILE = $temporaryDiscoveryPath
    Remove-ProcessEnvironmentVariable -Name "PLAYWRIGHT_JUNIT_OUTPUT_FILE"
    Invoke-CheckedStep -Label "Discovering the complete Playwright acceptance inventory" -Command {
        npm --prefix $endToEndDir test -- @manifestTestPaths --list --reporter=json
    }
    $discovery = Read-PlaywrightReport -Path $temporaryDiscoveryPath
    Assert-ExpectedInventory `
        -Manifest $manifest `
        -ActualCounts $discovery.Counts `
        -ActualIdentities $discovery.Identities `
        -ActualTotal $discovery.Total `
        -Label "Playwright discovery"
    if ([int]$discovery.Report.config.workers -ne 1 -or -not [bool]$discovery.Report.config.forbidOnly) {
        throw "Acceptance discovery must use exactly one worker with forbidOnly enabled."
    }
    if (@($discovery.Report.config.projects | Where-Object { [int]$_.retries -ne 0 }).Count -gt 0) {
        throw "Acceptance discovery found a project with nonzero retries."
    }

    $env:PLAYWRIGHT_JSON_OUTPUT_FILE = $temporaryEvidencePath
    $env:PLAYWRIGHT_JUNIT_OUTPUT_FILE = $temporaryJunitPath
    Invoke-CheckedStep -Label "Running the complete Playwright acceptance suite with one worker and zero retries" -Command {
        npm --prefix $endToEndDir test -- @manifestTestPaths
    }

    $result = Read-PlaywrightReport -Path $temporaryEvidencePath -RequireResults
    Assert-ExpectedInventory `
        -Manifest $manifest `
        -ActualCounts $result.Counts `
        -ActualIdentities $result.Identities `
        -ActualTotal $result.Total `
        -Label "Playwright execution"
    $stats = $result.Report.stats
    if ([int]$stats.expected -ne $result.Total -or
        [int]$stats.skipped -ne 0 -or
        [int]$stats.unexpected -ne 0 -or
        [int]$stats.flaky -ne 0 -or
        $result.RetriedResults -ne 0 -or
        $result.NonPassingResults -ne 0 -or
        $result.NonPassingExpectations -ne 0) {
        throw "Playwright acceptance evidence is invalid: total=$($result.Total), expected=$($stats.expected), skipped=$($stats.skipped), unexpected=$($stats.unexpected), flaky=$($stats.flaky), retried_results=$($result.RetriedResults), non_passing_results=$($result.NonPassingResults), non_passing_expectations=$($result.NonPassingExpectations)."
    }
    if ([int]$result.Report.config.workers -ne 1 -or -not [bool]$result.Report.config.forbidOnly) {
        throw "Acceptance execution did not retain one-worker/forbidOnly configuration."
    }
    if (@($result.Report.config.projects | Where-Object { [int]$_.retries -ne 0 }).Count -gt 0) {
        throw "Acceptance execution found a project with nonzero retries."
    }
    if (-not (Test-Path -LiteralPath $temporaryJunitPath -PathType Leaf)) {
        throw "Playwright did not produce required JUnit evidence '$temporaryJunitPath'."
    }

    $deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
        -RepositoryRoot $repoRoot `
        -EvidencePath $deploymentEvidenceFullPath `
        -BaseUrl $BaseUrl `
        -ExpectedDataState $ExpectedDataState
    $finalDeploymentEvidenceSha256 = (Get-FileHash -LiteralPath $deploymentEvidenceFullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    Assert-PlaywrightDeploymentEvidenceDigestStable `
        -InitialSha256 $deploymentEvidenceSha256 `
        -FinalSha256 $finalDeploymentEvidenceSha256
    $finalBinding = Get-PlaywrightPostgresAcceptanceBinding -DeploymentEvidence $deploymentEvidence
    if ($env:PLAYWRIGHT_POSTGRES_CONTAINER -cne $finalBinding.Container -or
        $env:PLAYWRIGHT_POSTGRES_DATABASE -cne $finalBinding.Database -or
        $env:PLAYWRIGHT_POSTGRES_USER -cne $finalBinding.User -or
        $env:TESSARA_PLAYWRIGHT_DATA_STATE -cne $ExpectedDataState) {
        throw "Playwright acceptance binding changed before final deployment revalidation; publication is forbidden."
    }

    $manifestDigest = (Get-FileHash -LiteralPath $manifestFullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $discoveryDigest = (Get-FileHash -LiteralPath $temporaryDiscoveryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $resultDigest = (Get-FileHash -LiteralPath $temporaryEvidencePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $junitDigest = (Get-FileHash -LiteralPath $temporaryJunitPath -Algorithm SHA256).Hash.ToLowerInvariant()
    [ordered]@{
        schema_version = 1
        acceptance = $true
        base_url = $BaseUrl.TrimEnd("/")
        total = $result.Total
        expected = [int]$stats.expected
        skipped = [int]$stats.skipped
        unexpected = [int]$stats.unexpected
        flaky = [int]$stats.flaky
        retried_results = $result.RetriedResults
        workers = [int]$result.Report.config.workers
        retries = 0
        forbid_only = [bool]$result.Report.config.forbidOnly
        manifest = [ordered]@{ path = $manifestFullPath; sha256 = $manifestDigest }
        discovery = [ordered]@{ path = $discoveryPath; sha256 = $discoveryDigest }
        json_report = [ordered]@{ path = $evidenceFullPath; sha256 = $resultDigest }
        junit_report = [ordered]@{ path = $junitPath; sha256 = $junitDigest }
        deployment = [ordered]@{
            evidence_path = $deploymentEvidenceFullPath
            evidence_sha256 = $finalDeploymentEvidenceSha256
            data_state = [string]$deploymentEvidence.snapshot.data.state
            image_id = [string]$deploymentEvidence.snapshot.release_image.image_id
            source_commit = [string]$deploymentEvidence.snapshot.source.commit
            database_name = [string]$deploymentEvidence.snapshot.database_runtime.current_database
            database_user = [string]$deploymentEvidence.snapshot.database_runtime.database_user
        }
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $temporarySummaryPath -Encoding utf8

    Publish-PlaywrightArtifactSet `
        -Artifacts @(
            [pscustomobject]@{ TemporaryPath = $temporaryEvidencePath; FinalPath = $evidenceFullPath },
            [pscustomobject]@{ TemporaryPath = $temporaryDiscoveryPath; FinalPath = $discoveryPath },
            [pscustomobject]@{ TemporaryPath = $temporaryJunitPath; FinalPath = $junitPath },
            [pscustomobject]@{ TemporaryPath = $temporarySummaryPath; FinalPath = $summaryPath }
        ) `
        -AllowOverwrite:$OverwriteEvidence
    Write-Host "Playwright acceptance evidence passed: $($result.Total) passed; 0 skipped, unexpected, flaky, filtered, or retried." -ForegroundColor Green
    Write-Host "Acceptance summary: $summaryPath"
} finally {
    try {
        foreach ($temporaryPath in $temporaryArtifactPaths) {
            Remove-Item -LiteralPath $temporaryPath -Force -ErrorAction SilentlyContinue
        }
        if (-not [string]::IsNullOrWhiteSpace($temporaryArtifactDirectory)) {
            Remove-Item -LiteralPath $temporaryArtifactDirectory -Recurse -Force -ErrorAction SilentlyContinue
        }
    } finally {
        try {
            Restore-PlaywrightEnvironment
        } finally {
            Pop-Location
        }
    }
}

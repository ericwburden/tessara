[CmdletBinding()]
param(
    [string]$ManifestPath = "end2end/acceptance-manifest.json",
    [ValidateSet("fresh", "upgraded")]
    [string]$ExpectedDataState = "fresh",
    [string]$EvidencePath,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$endToEndRoot = Join-Path $repositoryRoot "end2end"
$manifestFullPath = [IO.Path]::GetFullPath($(if ([IO.Path]::IsPathRooted($ManifestPath)) {
    $ManifestPath
} else {
    Join-Path $repositoryRoot $ManifestPath
}))
$repositoryPrefix = $repositoryRoot.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
if (-not $manifestFullPath.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase) -or
    -not (Test-Path -LiteralPath $manifestFullPath -PathType Leaf)) {
    throw "Browser-batch manifest must be a tracked repository file: $ManifestPath"
}

function Get-Sha256File {
    param([Parameter(Mandatory)][string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Publish-JsonAndSidecar {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document
    )
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Browser-batch evidence must remain inside the repository: $Path"
    }
    if ((Test-Path -LiteralPath $fullPath) -or (Test-Path -LiteralPath "$fullPath.sha256")) {
        throw "Browser-batch evidence is non-overwritable: $fullPath"
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $fullPath)) | Out-Null
    $temporaryPath = "$fullPath.tmp-$([Guid]::NewGuid().ToString('N'))"
    try {
        ($Document | ConvertTo-Json -Depth 100) + "`n" |
            Set-Content -LiteralPath $temporaryPath -Encoding utf8NoBOM -NoNewline
        Move-Item -LiteralPath $temporaryPath -Destination $fullPath
        $sha256 = Get-Sha256File -Path $fullPath
        Set-Content -LiteralPath "$fullPath.sha256" -Value $sha256 -Encoding ascii -NoNewline
        return $sha256
    } finally {
        if (Test-Path -LiteralPath $temporaryPath) {
            Remove-Item -LiteralPath $temporaryPath -Force
        }
    }
}

function Read-AcceptanceManifest {
    $manifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    if ([int]$manifest.schema_version -ne 2 -or [int]$manifest.expected_total -le 0 -or
        @($manifest.files).Count -lt 1) {
        throw "Browser-batch manifest is not the exact schema-v2 acceptance inventory."
    }
    $paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $identities = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $total = 0
    foreach ($entry in @($manifest.files)) {
        $path = ([string]$entry.path).Replace("\", "/")
        if ($path -cnotmatch '^[A-Za-z0-9._/-]+\.spec\.ts$' -or $path.Contains("..") -or
            -not $paths.Add($path)) {
            throw "Browser-batch manifest contains an invalid or duplicate file '$path'."
        }
        $testPath = Join-Path (Join-Path $endToEndRoot "tests") $path
        if (-not (Test-Path -LiteralPath $testPath -PathType Leaf)) {
            throw "Browser-batch manifest names missing test file '$path'."
        }
        if (@($entry.tests).Count -lt 1) {
            throw "Browser-batch manifest file '$path' has no exact test identities."
        }
        foreach ($titleValue in @($entry.tests)) {
            $title = [string]$titleValue
            if ([string]::IsNullOrWhiteSpace($title) -or -not $identities.Add("$path :: $title")) {
                throw "Browser-batch manifest contains a blank or duplicate identity in '$path'."
            }
            $total++
        }
    }
    if ($total -ne [int]$manifest.expected_total) {
        throw "Browser-batch manifest lists $total identities, expected $($manifest.expected_total)."
    }
    return [pscustomobject]@{
        Document = $manifest
        Paths = @($manifest.files | ForEach-Object { ([string]$_.path).Replace("\", "/") })
        Identities = $identities
        Total = $total
    }
}

function Add-ReportIdentities {
    param(
        [Parameter(Mandatory)]$Suite,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.HashSet[string]]$Identities,
        [Parameter(Mandatory)][ref]$Retried,
        [Parameter(Mandatory)][ref]$NonPassing,
        [Parameter(Mandatory)][ref]$NonPassingExpectations,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ParentTitles,
        [string]$InheritedFile = ""
    )
    $file = if ($null -ne $Suite.PSObject.Properties['file'] -and
        -not [string]::IsNullOrWhiteSpace([string]$Suite.file)) {
        ([string]$Suite.file).Replace("\", "/")
    } else { $InheritedFile }
    $titles = @($ParentTitles)
    $suiteTitle = if ($null -eq $Suite.PSObject.Properties['title']) { "" } else { [string]$Suite.title }
    if (-not [string]::IsNullOrWhiteSpace($suiteTitle) -and $suiteTitle -cne $file) {
        $titles += $suiteTitle
    }
    $specs = if ($null -eq $Suite.PSObject.Properties['specs']) { @() } else { @($Suite.specs) }
    foreach ($spec in $specs) {
        if ($null -eq $spec) { continue }
        $specTitle = [string]$spec.title
        $fullTitle = (@($titles) + @($specTitle)) -join " › "
        foreach ($test in @($spec.tests)) {
            if ($null -eq $test) { continue }
            $identity = "$file :: $fullTitle"
            if (-not $Identities.Add($identity)) {
                throw "Browser batch report repeats exact identity '$identity'."
            }
            $results = @($test.results)
            if ($results.Count -ne 1) {
                $NonPassing.Value++
            }
            if ([string]$test.expectedStatus -cne "passed") { $NonPassingExpectations.Value++ }
            foreach ($result in $results) {
                if ([int]$result.retry -ne 0) { $Retried.Value++ }
                if ([string]$result.status -cne "passed") { $NonPassing.Value++ }
            }
        }
    }
    $childSuites = if ($null -eq $Suite.PSObject.Properties['suites']) { @() } else { @($Suite.suites) }
    foreach ($child in $childSuites) {
        if ($null -ne $child) {
            Add-ReportIdentities -Suite $child -Identities $Identities -Retried $Retried `
                -NonPassing $NonPassing -NonPassingExpectations $NonPassingExpectations `
                -ParentTitles $titles -InheritedFile $file
        }
    }
}

function Read-BatchReport {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Playwright batch omitted JSON report '$Path'."
    }
    $report = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    $identities = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $retried = 0
    $nonPassing = 0
    $nonPassingExpectations = 0
    foreach ($suite in @($report.suites)) {
        Add-ReportIdentities -Suite $suite -Identities $identities -Retried ([ref]$retried) `
            -NonPassing ([ref]$nonPassing) -NonPassingExpectations ([ref]$nonPassingExpectations) `
            -ParentTitles @()
    }
    $invalidConfig = if ([int]$report.config.workers -ne 1 -or -not [bool]$report.config.forbidOnly -or
        @($report.config.projects | Where-Object { [int]$_.retries -ne 0 }).Count -ne 0) { 1 } else { 0 }
    $stats = $report.stats
    if ([int]$stats.expected -ne $identities.Count -or [int]$stats.skipped -ne 0 -or
        [int]$stats.unexpected -ne 0 -or [int]$stats.flaky -ne 0) {
        $nonPassing++
    }
    return [pscustomobject]@{
        Identities = $identities
        Retried = $retried
        NonPassing = $nonPassing
        NonPassingExpectations = $nonPassingExpectations
        InvalidConfig = $invalidConfig
    }
}

function Invoke-SyntheticBrowserLifecycle {
    param(
        [Parameter(Mandatory)][string[]]$BatchPaths,
        [Parameter(Mandatory)][string]$OutputPath
    )
    $probeRoot = Split-Path -Parent $OutputPath
    [IO.Directory]::CreateDirectory($probeRoot) | Out-Null
    $probeScript = Join-Path $probeRoot "browser-lifecycle-probe.cjs"
    $playwrightPath = (Join-Path $endToEndRoot "node_modules/playwright").Replace("\", "\\")
    $batchJson = ConvertTo-Json -InputObject @($BatchPaths) -Compress
    $outputEscaped = $OutputPath.Replace("\", "\\")
    $sourceTemplate = @'
const fs = require('fs');
const http = require('http');
const { chromium } = require('__PLAYWRIGHT_PATH__');
const batches = __BATCHES_JSON__;
const outputPath = '__OUTPUT_PATH__';
(async () => {
  let requests = 0;
  const server = http.createServer((request, response) => {
    requests += 1;
    if (request.url.endsWith('.css')) {
      response.writeHead(200, {'content-type': 'text/css', 'cache-control': 'no-store'});
      return response.end('body{color:rgb(1,2,3)}');
    }
    if (request.url.endsWith('.js')) {
      response.writeHead(200, {'content-type': 'application/javascript', 'cache-control': 'no-store'});
      return response.end('document.documentElement.dataset.probe="ready";');
    }
    response.writeHead(200, {'content-type': 'text/html', 'cache-control': 'no-store'});
    response.end('<!doctype html><link rel="stylesheet" href="/probe.css"><script src="/probe.js"></script><main>ready</main>');
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  const address = server.address();
  const results = [];
  try {
    for (const batch of batches) {
      const browser = await chromium.launch();
      let disconnected = false;
      browser.once('disconnected', () => { disconnected = true; });
      const context = await browser.newContext();
      const page = await context.newPage();
      const failures = [];
      page.on('requestfailed', request => failures.push(`${request.url()} :: ${request.failure()?.errorText || 'unknown'}`));
      for (let index = 0; index < 8; index += 1) {
        await page.goto(`http://127.0.0.1:${address.port}/${encodeURIComponent(batch)}/${index}`, {waitUntil: 'networkidle'});
        if (await page.locator('main').textContent() !== 'ready') throw new Error(`probe document mismatch for ${batch}`);
      }
      await context.close();
      await browser.close();
      if (!disconnected) throw new Error(`browser process did not disconnect for ${batch}`);
      if (failures.length) throw new Error(`browser network failures for ${batch}: ${failures.join('; ')}`);
      results.push({batch, navigations: 8, browser_disconnected: true, network_failures: 0});
    }
  } finally {
    await new Promise(resolve => server.close(resolve));
  }
  fs.writeFileSync(outputPath, JSON.stringify({schema_version:1, state:'passed', batch_count:batches.length, requests, results}, null, 2) + '\n');
})().catch(error => { console.error(error); process.exit(1); });
'@
    $source = $sourceTemplate.
        Replace('__PLAYWRIGHT_PATH__', $playwrightPath).
        Replace('__BATCHES_JSON__', $batchJson).
        Replace('__OUTPUT_PATH__', $outputEscaped)
    Set-Content -LiteralPath $probeScript -Value $source -Encoding utf8NoBOM
    & node $probeScript
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $OutputPath -PathType Leaf)) {
        throw "Synthetic fresh-browser lifecycle certification failed."
    }
    $probe = Get-Content -LiteralPath $OutputPath -Raw | ConvertFrom-Json
    if ([string]$probe.state -cne "passed" -or [int]$probe.batch_count -ne $BatchPaths.Count -or
        @($probe.results | Where-Object { $_.browser_disconnected -ne $true -or [int]$_.network_failures -ne 0 }).Count -ne 0) {
        throw "Synthetic fresh-browser lifecycle evidence is incomplete."
    }
    return $probe
}

$inventory = Read-AcceptanceManifest
$temporaryRoot = Join-Path $repositoryRoot "target/sprint-8b-validation-platform/browser-batches-$([Guid]::NewGuid().ToString('N'))"
[IO.Directory]::CreateDirectory($temporaryRoot) | Out-Null
try {
    $probePath = Join-Path $temporaryRoot "platform-probe.json"
    $probe = Invoke-SyntheticBrowserLifecycle -BatchPaths $inventory.Paths -OutputPath $probePath
    if ($SelfTest) {
        if ($inventory.Total -ne 95 -or $inventory.Paths.Count -ne 11) {
            throw "Browser lifecycle self-test requires the exact current 95-test, 11-file inventory."
        }
        $syntheticReport = [ordered]@{
            config = [ordered]@{
                workers = 1
                forbidOnly = $true
                projects = @([ordered]@{ name = ""; retries = 0 })
            }
            suites = @([ordered]@{
                title = "alpha.spec.ts"
                file = "alpha.spec.ts"
                suites = @([ordered]@{
                    title = "synthetic lifecycle"
                    file = "alpha.spec.ts"
                    specs = @([ordered]@{
                        title = "retains one exact passing identity"
                        tests = @([ordered]@{
                            expectedStatus = "passed"
                            projectName = ""
                            results = @([ordered]@{ retry = 0; status = "passed" })
                        })
                    })
                })
            })
            stats = [ordered]@{ expected = 1; skipped = 0; unexpected = 0; flaky = 0 }
        }
        $syntheticReportPath = Join-Path $temporaryRoot "synthetic-report.json"
        ($syntheticReport | ConvertTo-Json -Depth 20) + "`n" |
            Set-Content -LiteralPath $syntheticReportPath -Encoding utf8NoBOM -NoNewline
        $parsedReport = Read-BatchReport -Path $syntheticReportPath
        if ($parsedReport.Identities.Count -ne 1 -or
            -not $parsedReport.Identities.Contains(
                "alpha.spec.ts :: synthetic lifecycle › retains one exact passing identity"
            ) -or $parsedReport.Retried -ne 0 -or $parsedReport.NonPassing -ne 0 -or
            $parsedReport.NonPassingExpectations -ne 0 -or $parsedReport.InvalidConfig -ne 0) {
            throw "Browser lifecycle self-test did not authenticate the exact synthetic report."
        }
        $syntheticReport.config.projects[0].retries = 1
        ($syntheticReport | ConvertTo-Json -Depth 20) + "`n" |
            Set-Content -LiteralPath $syntheticReportPath -Encoding utf8NoBOM -NoNewline
        if ((Read-BatchReport -Path $syntheticReportPath).InvalidConfig -ne 1) {
            throw "Browser lifecycle self-test accepted a nonzero retry policy."
        }
        Write-Host "Synthetic browser lifecycle certification passed: $($probe.batch_count) fresh processes." -ForegroundColor Green
        return
    }

    if ([string]::IsNullOrWhiteSpace($EvidencePath)) {
        throw "Live browser batches require -EvidencePath."
    }
    if ([Environment]::GetEnvironmentVariable("TESSARA_PLAYWRIGHT_ACCEPTANCE", "Process") -cne "1" -or
        [string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable("PLAYWRIGHT_BASE_URL", "Process"))) {
        throw "Live browser batches require an authenticated retained topology binding."
    }

    $evidenceFullPath = [IO.Path]::GetFullPath($(if ([IO.Path]::IsPathRooted($EvidencePath)) {
        $EvidencePath
    } else {
        Join-Path $repositoryRoot $EvidencePath
    }))
    if (-not $evidenceFullPath.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Live browser-batch evidence must remain inside the repository."
    }
    if ((Test-Path -LiteralPath $evidenceFullPath) -or
        (Test-Path -LiteralPath "$evidenceFullPath.sha256")) {
        throw "Browser-batch evidence is non-overwritable: $evidenceFullPath"
    }
    $artifactRoot = Join-Path (Split-Path -Parent $evidenceFullPath) `
        "$([IO.Path]::GetFileNameWithoutExtension($evidenceFullPath))-artifacts"
    if (Test-Path -LiteralPath $artifactRoot) {
        throw "Browser-batch artifact root is non-overwritable: $artifactRoot"
    }
    [IO.Directory]::CreateDirectory($artifactRoot) | Out-Null
    Copy-Item -LiteralPath $probePath -Destination (Join-Path $artifactRoot "platform-probe.json")

    $priorDataState = [Environment]::GetEnvironmentVariable("TESSARA_PLAYWRIGHT_DATA_STATE", "Process")
    $priorJsonOutput = [Environment]::GetEnvironmentVariable("PLAYWRIGHT_JSON_OUTPUT_FILE", "Process")
    $priorJunitOutput = [Environment]::GetEnvironmentVariable("PLAYWRIGHT_JUNIT_OUTPUT_FILE", "Process")
    $allIdentities = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $batchResults = [Collections.Generic.List[object]]::new()
    try {
        [Environment]::SetEnvironmentVariable("TESSARA_PLAYWRIGHT_DATA_STATE", $ExpectedDataState, "Process")
        for ($index = 0; $index -lt $inventory.Paths.Count; $index++) {
            $path = [string]$inventory.Paths[$index]
            $batchId = "{0:D2}-{1}" -f ($index + 1), ([IO.Path]::GetFileNameWithoutExtension($path) -replace '[^A-Za-z0-9.-]', '-')
            $batchRoot = Join-Path $artifactRoot $batchId
            [IO.Directory]::CreateDirectory($batchRoot) | Out-Null
            $jsonPath = Join-Path $batchRoot "report.json"
            $junitPath = Join-Path $batchRoot "junit.xml"
            $logPath = Join-Path $batchRoot "command.log"
            [Environment]::SetEnvironmentVariable("PLAYWRIGHT_JSON_OUTPUT_FILE", $jsonPath, "Process")
            [Environment]::SetEnvironmentVariable("PLAYWRIGHT_JUNIT_OUTPUT_FILE", $junitPath, "Process")
            & npm --prefix $endToEndRoot test -- "tests/$path" 2>&1 |
                Tee-Object -LiteralPath $logPath | Out-Host
            $exitCode = $LASTEXITCODE
            $testResultsPath = Join-Path $endToEndRoot "test-results"
            if (Test-Path -LiteralPath $testResultsPath -PathType Container) {
                Copy-Item -LiteralPath $testResultsPath -Destination (Join-Path $batchRoot "test-results") -Recurse
            }

            $reportState = "failed"
            $identityCount = 0
            $retried = -1
            $nonPassing = -1
            $nonPassingExpectations = -1
            $invalidConfig = -1
            $reportError = $null
            try {
                $report = Read-BatchReport -Path $jsonPath
                $identityCount = $report.Identities.Count
                $retried = $report.Retried
                $nonPassing = $report.NonPassing
                $nonPassingExpectations = $report.NonPassingExpectations
                $invalidConfig = $report.InvalidConfig
                foreach ($identity in $report.Identities) {
                    if (-not $allIdentities.Add($identity)) {
                        throw "Browser batches repeat exact identity '$identity'."
                    }
                }
                if ($exitCode -eq 0 -and $retried -eq 0 -and $nonPassing -eq 0 -and
                    $nonPassingExpectations -eq 0 -and $invalidConfig -eq 0) {
                    $reportState = "passed"
                }
            } catch {
                $reportError = $_.Exception.Message
            }
            $batchResults.Add([ordered]@{
                id = $batchId
                path = $path
                state = $reportState
                exit_code = $exitCode
                identity_count = $identityCount
                retried_results = $retried
                non_passing_results = $nonPassing
                non_passing_expectations = $nonPassingExpectations
                invalid_configuration = $invalidConfig
                report_error = $reportError
                report_sha256 = if (Test-Path -LiteralPath $jsonPath) { Get-Sha256File -Path $jsonPath } else { $null }
                junit_sha256 = if (Test-Path -LiteralPath $junitPath) { Get-Sha256File -Path $junitPath } else { $null }
                command_log_sha256 = Get-Sha256File -Path $logPath
            })
        }
    } finally {
        [Environment]::SetEnvironmentVariable("TESSARA_PLAYWRIGHT_DATA_STATE", $priorDataState, "Process")
        [Environment]::SetEnvironmentVariable("PLAYWRIGHT_JSON_OUTPUT_FILE", $priorJsonOutput, "Process")
        [Environment]::SetEnvironmentVariable("PLAYWRIGHT_JUNIT_OUTPUT_FILE", $priorJunitOutput, "Process")
    }

    $missing = @($inventory.Identities | Where-Object { -not $allIdentities.Contains($_) })
    $unexpected = @($allIdentities | Where-Object { -not $inventory.Identities.Contains($_) })
    $failed = @($batchResults | Where-Object { [string]$_.state -cne "passed" })
    $state = if ($failed.Count -eq 0 -and $missing.Count -eq 0 -and $unexpected.Count -eq 0 -and
        $allIdentities.Count -eq $inventory.Total) { "passed" } else { "failed" }
    $artifactInventory = @(Get-ChildItem -LiteralPath $artifactRoot -Recurse -File | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path = [IO.Path]::GetRelativePath($repositoryRoot, $_.FullName).Replace("\", "/")
            sha256 = Get-Sha256File -Path $_.FullName
        }
    })
    $document = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.browser-batch-result"
        sprint = "sprint-8b"
        state = $state
        execution_policy = [ordered]@{
            one_fresh_browser_process_per_manifest_file = true
            workers = 1
            retries = 0
            fail_late_across_batches = true
            snapshot_update = false
        }
        expected_data_state = $ExpectedDataState
        manifest = [ordered]@{ path = $ManifestPath.Replace("\", "/"); sha256 = Get-Sha256File -Path $manifestFullPath }
        expected_total = $inventory.Total
        executed_total = $allIdentities.Count
        missing_identities = $missing
        unexpected_identities = $unexpected
        platform_probe = [ordered]@{
            state = [string]$probe.state
            batch_count = [int]$probe.batch_count
            requests = [int]$probe.requests
            sha256 = Get-Sha256File -Path (Join-Path $artifactRoot "platform-probe.json")
        }
        batches = @($batchResults)
        artifacts = $artifactInventory
        cleanup_restoration = [ordered]@{ state = "passed"; mode = "browser-processes-closed-between-batches" }
    }
    $null = Publish-JsonAndSidecar -Path $evidenceFullPath -Document $document
    if ($state -cne "passed") {
        throw "Browser acceptance batches failed: failed=$($failed.Count), missing=$($missing.Count), unexpected=$($unexpected.Count)."
    }
    Write-Host "Browser acceptance batches passed: $($inventory.Total) exact tests in $($inventory.Paths.Count) fresh processes." -ForegroundColor Green
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

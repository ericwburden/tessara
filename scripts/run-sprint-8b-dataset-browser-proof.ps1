[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8b-implementation-dataset-browser",
    [string]$ComposeFile = "deploy/sprint-8b/compose.yaml",
    [string]$EvidencePath = "target/sprint-8b-dataset-browser-proof/result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8b-harness-isolation.ps1")
$SelfTest = $requestedSelfTest

function Get-Sprint8BDatasetBrowserIdentityContract {
    $manifestPath = Join-Path $repoRoot "end2end/acceptance-manifest.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Sprint 8B Dataset browser proof requires the tracked acceptance manifest."
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 100
    $moduleFile = @($manifest.files | Where-Object { [string]$_.path -ceq "datasets-module.spec.ts" })
    $visualFile = @($manifest.files | Where-Object { [string]$_.path -ceq "module-ui-visual.spec.ts" })
    if ($moduleFile.Count -ne 1 -or $visualFile.Count -ne 1) {
        throw "Sprint 8B Dataset browser proof requires one exact module and visual manifest entry."
    }
    $module = @($moduleFile[0].tests | ForEach-Object { [string]$_ })
    $visual = @($visualFile[0].tests | Where-Object {
        [string]$_ -clike "canonical module UI visual baselines › Datasets*"
    } | ForEach-Object { [string]$_ })
    $expectedModule = @(
        "Sprint 8B independent Dataset module › editor options use only Dataset-owned browser routes",
        "Sprint 8B independent Dataset module › synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure",
        "Sprint 8B independent Dataset module › reverse consumers distinguish authorized empty unavailable and undisclosed states",
        "Sprint 8B independent Dataset module › mutation replay and static route precedence remain exact"
    )
    $expectedVisual = @(
        "canonical module UI visual baselines › Datasets directory at 1440 px (light)",
        "canonical module UI visual baselines › Datasets editor at 390 px (light)",
        "canonical module UI visual baselines › Datasets directory at 1440 px (dark)",
        "canonical module UI visual baselines › Datasets editor at 390 px (dark)",
        "canonical module UI visual baselines › Datasets revisions at 1024 px",
        "canonical module UI visual baselines › Datasets preview at 1440 px",
        "canonical module UI visual baselines › Datasets, Components, Dashboards, and Scoped Records share one module canvas"
    )
    if (($module -join "`n") -cne ($expectedModule -join "`n") -or
        ($visual -join "`n") -cne ($expectedVisual -join "`n")) {
        throw "Sprint 8B Dataset browser manifest identities are not the exact frozen 4+7 set."
    }
    [pscustomobject][ordered]@{
        module = $module
        visual = $visual
        all = @($module + $visual)
    }
}

function Add-Sprint8BWindowsSnapshotSuffix {
    param([Parameter(Mandatory)][string]$Name)

    if ($Name -cnotmatch '^(?<stem>[a-z0-9-]+)\.png$') {
        throw "Dataset visual source produced invalid snapshot name '$Name'."
    }
    "$($Matches.stem)-win32.png"
}

function Get-Sprint8BDatasetSnapshotNamesFromSource {
    param([Parameter(Mandatory)][string]$SourcePath)

    if (-not (Test-Path -LiteralPath $SourcePath -PathType Leaf)) {
        throw "Dataset visual source is missing: $SourcePath"
    }
    $text = Get-Content -LiteralPath $SourcePath -Raw
    $sourcePatterns = [ordered]@{
        directory = 'toHaveScreenshot\(`datasets-directory-\$\{theme\}-1440\.png`'
        editor = 'toHaveScreenshot\(`datasets-editor-\$\{theme\}-390\.png`'
        revisions = 'toHaveScreenshot\("datasets-revisions-dark-1024\.png"'
        preview = 'toHaveScreenshot\("datasets-preview-light-1440\.png"'
        parity = '`module-parity-with-datasets-\$\{module\.name\}-dark-1440\.png`'
    }
    foreach ($entry in $sourcePatterns.GetEnumerator()) {
        if ([regex]::Matches($text, [string]$entry.Value).Count -ne 1) {
            throw "Dataset visual source does not contain one exact '$($entry.Key)' snapshot expression."
        }
    }

    $parityStart = $text.IndexOf(
        'test("Datasets, Components, Dashboards, and Scoped Records share one module canvas"',
        [StringComparison]::Ordinal
    )
    $parityEnd = $text.IndexOf("await expectDatasetZoomContainment(browser);", $parityStart,
        [StringComparison]::Ordinal)
    if ($parityStart -lt 0 -or $parityEnd -le $parityStart) {
        throw "Dataset visual source does not retain the exact shared-canvas/zoom predicate."
    }
    $parityBlock = $text.Substring($parityStart, $parityEnd - $parityStart)
    $moduleNames = @([regex]::Matches(
        $parityBlock,
        'name:\s*"(?<name>datasets|components|dashboards|scoped-records)"'
    ) | ForEach-Object { [string]$_.Groups['name'].Value })
    $expectedModules = @("datasets", "components", "dashboards", "scoped-records")
    if (($moduleNames -join "`n") -cne ($expectedModules -join "`n")) {
        throw "Dataset shared-canvas source does not capture the exact four-module order."
    }

    $logicalNames = [Collections.Generic.List[string]]::new()
    foreach ($theme in @("light", "dark")) {
        $logicalNames.Add("datasets-directory-$theme-1440.png")
        $logicalNames.Add("datasets-editor-$theme-390.png")
    }
    $logicalNames.Add("datasets-revisions-dark-1024.png")
    $logicalNames.Add("datasets-preview-light-1440.png")
    foreach ($moduleName in $moduleNames) {
        $logicalNames.Add("module-parity-with-datasets-$moduleName-dark-1440.png")
    }
    $derived = @($logicalNames | ForEach-Object { Add-Sprint8BWindowsSnapshotSuffix -Name $_ })
    $expected = @(
        "datasets-directory-light-1440-win32.png",
        "datasets-editor-light-390-win32.png",
        "datasets-directory-dark-1440-win32.png",
        "datasets-editor-dark-390-win32.png",
        "datasets-revisions-dark-1024-win32.png",
        "datasets-preview-light-1440-win32.png",
        "module-parity-with-datasets-datasets-dark-1440-win32.png",
        "module-parity-with-datasets-components-dark-1440-win32.png",
        "module-parity-with-datasets-dashboards-dark-1440-win32.png",
        "module-parity-with-datasets-scoped-records-dark-1440-win32.png"
    )
    if (($derived -join "`n") -cne ($expected -join "`n")) {
        throw "Dataset visual source did not derive the exact ten Windows snapshot baselines."
    }
    $derived
}

function Assert-Sprint8BDatasetSnapshotPreflight {
    param(
        [Parameter(Mandatory)][string]$VisualSourcePath,
        [Parameter(Mandatory)][string]$SnapshotDirectory,
        [switch]$RequireTracked
    )

    $expected = @(Get-Sprint8BDatasetSnapshotNamesFromSource -SourcePath $VisualSourcePath)
    $actual = if (Test-Path -LiteralPath $SnapshotDirectory -PathType Container) {
        @(Get-ChildItem -LiteralPath $SnapshotDirectory -File -Filter "*.png" | Where-Object {
            $_.Name -clike "datasets-*-win32.png" -or
                $_.Name -clike "module-parity-with-datasets-*-win32.png"
        } | Sort-Object Name | ForEach-Object { $_.Name })
    } else { @() }
    if (($actual -join "`n") -cne (@($expected | Sort-Object) -join "`n")) {
        $missing = @($expected | Where-Object { $actual -cnotcontains $_ })
        $unexpected = @($actual | Where-Object { $expected -cnotcontains $_ })
        throw "Dataset visual baseline preflight failed before topology creation (missing=$($missing -join ','), unexpected=$($unexpected -join ','))."
    }
    $entries = [Collections.Generic.List[object]]::new()
    foreach ($name in $expected) {
        $path = Join-Path $SnapshotDirectory $name
        $bytes = [IO.File]::ReadAllBytes($path)
        if ($bytes.Length -lt 8 -or
            ($bytes[0..7] -join ',') -cne '137,80,78,71,13,10,26,10') {
            throw "Dataset visual baseline '$name' is empty or is not a PNG."
        }
        if ($RequireTracked) {
            $relative = [IO.Path]::GetRelativePath($repoRoot, $path).Replace("\", "/")
            & git ls-files --error-unmatch -- $relative *> $null
            if ($LASTEXITCODE -ne 0) {
                throw "Dataset visual baseline '$relative' is not tracked source."
            }
        }
        $entries.Add([pscustomobject][ordered]@{
            name = $name
            sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
            size = $bytes.Length
        })
    }
    [pscustomobject][ordered]@{
        state = "passed"
        source_derived_count = $expected.Count
        expected = $expected
        actual = $actual
        files = @($entries)
    }
}

function Add-Sprint8BPlaywrightSuiteResults {
    param(
        [Parameter(Mandatory)]$Suite,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Parents,
        [Parameter(Mandatory)][AllowEmptyString()][string]$InheritedFile,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Tests
    )

    $suiteFile = if ($null -eq $Suite.PSObject.Properties['file']) { "" } else { [string]$Suite.file }
    $file = if ([string]::IsNullOrWhiteSpace($suiteFile)) { $InheritedFile } else { $suiteFile }
    $titles = @($Parents)
    $suiteTitle = if ($null -eq $Suite.PSObject.Properties['title']) { "" } else { [string]$Suite.title }
    if (-not [string]::IsNullOrWhiteSpace($suiteTitle) -and $suiteTitle -cne $file) {
        $titles += $suiteTitle
    }
    foreach ($spec in @($Suite.specs)) {
        $specTitle = [string]$spec.title
        if ([string]::IsNullOrWhiteSpace($specTitle)) {
            throw "Focused Dataset Playwright report contains an untitled test."
        }
        $identity = (@($titles) + @($specTitle)) -join " › "
        foreach ($test in @($spec.tests)) {
            $results = @($test.results)
            $Tests.Add([pscustomobject][ordered]@{
                file = $file
                identity = $identity
                expected_status = [string]$test.expectedStatus
                project_name = [string]$test.projectName
                results = @($results | ForEach-Object { [pscustomobject][ordered]@{
                    status = [string]$_.status
                    retry = [int]$_.retry
                } })
            })
        }
    }
    foreach ($child in @($Suite.suites)) {
        Add-Sprint8BPlaywrightSuiteResults -Suite $child -Parents $titles `
            -InheritedFile $file -Tests $Tests
    }
}

function Assert-Sprint8BFocusedPlaywrightReport {
    param(
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][string]$ExpectedFile,
        [Parameter(Mandatory)][string[]]$ExpectedIdentities
    )

    $tests = [Collections.Generic.List[object]]::new()
    foreach ($suite in @($Report.suites)) {
        Add-Sprint8BPlaywrightSuiteResults -Suite $suite -Parents @() `
            -InheritedFile "" -Tests $tests
    }
    $identities = @($tests | ForEach-Object { [string]$_.identity })
    if ($tests.Count -ne $ExpectedIdentities.Count -or
        @($identities | Sort-Object -Unique).Count -ne $ExpectedIdentities.Count -or
        (@($identities | Sort-Object) -join "`n") -cne
            (@($ExpectedIdentities | Sort-Object) -join "`n") -or
        @($tests | Where-Object {
            [string]$_.file -cne $ExpectedFile -or
            [string]$_.expected_status -cne "passed" -or
            @($_.results).Count -ne 1 -or
            [string]$_.results[0].status -cne "passed" -or
            [int]$_.results[0].retry -ne 0
        }).Count -ne 0 -or
        [int]$Report.config.workers -ne 1 -or -not [bool]$Report.config.forbidOnly -or
        @($Report.config.projects | Where-Object { [int]$_.retries -ne 0 }).Count -ne 0 -or
        [string]$Report.config.updateSnapshots -cne "none" -or
        [int]$Report.stats.expected -ne $ExpectedIdentities.Count -or
        [int]$Report.stats.skipped -ne 0 -or [int]$Report.stats.unexpected -ne 0 -or
        [int]$Report.stats.flaky -ne 0) {
        throw "Focused Dataset Playwright report does not prove the exact passing one-worker, zero-retry identity set for '$ExpectedFile'."
    }
    [pscustomobject][ordered]@{
        file = $ExpectedFile
        expected = $ExpectedIdentities.Count
        passed = $tests.Count
        skipped = 0
        retries = 0
        workers = 1
        update_snapshots = "none"
        forbid_only = $true
        identities = $identities
    }
}

function Invoke-Sprint8BFocusedPlaywrightCommand {
    param(
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$ReportPath,
        [Parameter(Mandatory)][string]$JunitPath,
        [Parameter(Mandatory)][string]$LogPath
    )

    foreach ($path in @($ReportPath, $JunitPath, $LogPath)) {
        if (Test-Path -LiteralPath $path) {
            throw "Focused Dataset browser proof refuses to overwrite retained output '$path'."
        }
    }
    $env:PLAYWRIGHT_JSON_OUTPUT_FILE = $ReportPath
    $env:PLAYWRIGHT_JUNIT_OUTPUT_FILE = $JunitPath
    $lines = [Collections.Generic.List[string]]::new()
    & npm @Arguments 2>&1 | ForEach-Object {
        $line = [string]$_
        $lines.Add($line)
        Write-Host $line
    }
    $exitCode = $LASTEXITCODE
    [IO.File]::WriteAllLines($LogPath, @($lines), [Text.UTF8Encoding]::new($false))
    if ($exitCode -ne 0) {
        throw "Focused Dataset Playwright command exited ${exitCode}: npm $($Arguments -join ' ')"
    }
    foreach ($path in @($ReportPath, $JunitPath, $LogPath)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            [long](Get-Item -LiteralPath $path).Length -eq 0) {
            throw "Focused Dataset Playwright command omitted retained output '$path'."
        }
    }
    [pscustomobject][ordered]@{
        program = "npm"
        arguments = $Arguments
        exit_code = $exitCode
        report_path = $ReportPath
        report_sha256 = (Get-FileHash -LiteralPath $ReportPath -Algorithm SHA256).Hash.ToLowerInvariant()
        junit_path = $JunitPath
        junit_sha256 = (Get-FileHash -LiteralPath $JunitPath -Algorithm SHA256).Hash.ToLowerInvariant()
        log_path = $LogPath
        log_sha256 = (Get-FileHash -LiteralPath $LogPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

function New-Sprint8BPlaywrightSelfTestReport {
    param(
        [Parameter(Mandatory)][string]$File,
        [Parameter(Mandatory)][string[]]$Identities
    )

    $specs = foreach ($identity in $Identities) {
        $parts = $identity.Split(" › ")
        [pscustomobject][ordered]@{
            title = $parts[-1]
            tests = @([pscustomobject][ordered]@{
                expectedStatus = "passed"
                projectName = ""
                results = @([pscustomobject][ordered]@{ status = "passed"; retry = 0 })
            })
        }
    }
    $describeTitle = $Identities[0].Split(" › ")[0]
    [pscustomobject][ordered]@{
        config = [pscustomobject][ordered]@{
            workers = 1
            forbidOnly = $true
            projects = @([pscustomobject][ordered]@{ retries = 0 })
            updateSnapshots = "none"
        }
        suites = @([pscustomobject][ordered]@{
            title = $describeTitle
            file = $File
            specs = @($specs)
            suites = @()
        })
        stats = [pscustomobject][ordered]@{
            expected = $Identities.Count
            skipped = 0
            unexpected = 0
            flaky = 0
        }
    }
}

function Test-Sprint8BDatasetBrowserProofContract {
    $identity = Get-Sprint8BDatasetBrowserIdentityContract
    $temporary = Join-Path $repoRoot "target/sprint-8b-dataset-browser-selftest-$([guid]::NewGuid().ToString('N'))"
    $sourcePath = Join-Path $temporary "module-ui-visual.spec.ts"
    $snapshotDirectory = Join-Path $temporary "module-ui-visual.spec.ts-snapshots"
    try {
        [IO.Directory]::CreateDirectory($snapshotDirectory) | Out-Null
        $source = @'
for (const theme of ["light", "dark"] as const) {
  await expect(page).toHaveScreenshot(`datasets-directory-${theme}-1440.png`, {});
  await expect(page).toHaveScreenshot(`datasets-editor-${theme}-390.png`, {});
}
await expect(page).toHaveScreenshot("datasets-revisions-dark-1024.png", {});
await expect(page).toHaveScreenshot("datasets-preview-light-1440.png", {});
test("Datasets, Components, Dashboards, and Scoped Records share one module canvas", async () => {
  const modules = [
    { path: "/datasets", name: "datasets", title: "Datasets" },
    { path: "/components", name: "components", title: "Components" },
    { path: "/dashboards", name: "dashboards", title: "Dashboards" },
    { path: "/reference/scoped-records", name: "scoped-records", title: "Scoped Records" },
  ];
  await expect(page).toHaveScreenshot(`module-parity-with-datasets-${module.name}-dark-1440.png`, {});
  await expectDatasetZoomContainment(browser);
});
'@
        [IO.File]::WriteAllText($sourcePath, $source, [Text.UTF8Encoding]::new($false))
        $derived = @(Get-Sprint8BDatasetSnapshotNamesFromSource -SourcePath $sourcePath)
        $pngHeader = [byte[]](137, 80, 78, 71, 13, 10, 26, 10, 0)
        foreach ($name in $derived) {
            [IO.File]::WriteAllBytes((Join-Path $snapshotDirectory $name), $pngHeader)
        }
        $preflight = Assert-Sprint8BDatasetSnapshotPreflight -VisualSourcePath $sourcePath `
            -SnapshotDirectory $snapshotDirectory
        if ($preflight.source_derived_count -ne 10) {
            throw "Dataset browser proof self-test did not derive ten exact snapshots."
        }
        $missingRejected = $false
        Remove-Item -LiteralPath (Join-Path $snapshotDirectory $derived[0]) -Force
        try {
            Assert-Sprint8BDatasetSnapshotPreflight -VisualSourcePath $sourcePath `
                -SnapshotDirectory $snapshotDirectory | Out-Null
        } catch { $missingRejected = $_.Exception.Message -match 'before topology creation' }
        if (-not $missingRejected) {
            throw "Dataset browser proof self-test accepted a missing visual baseline."
        }

        $moduleReport = New-Sprint8BPlaywrightSelfTestReport `
            -File "datasets-module.spec.ts" -Identities $identity.module
        $visualReport = New-Sprint8BPlaywrightSelfTestReport `
            -File "module-ui-visual.spec.ts" -Identities $identity.visual
        $moduleProof = Assert-Sprint8BFocusedPlaywrightReport -Report $moduleReport `
            -ExpectedFile "datasets-module.spec.ts" -ExpectedIdentities $identity.module
        $visualProof = Assert-Sprint8BFocusedPlaywrightReport -Report $visualReport `
            -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identity.visual
        $retryRejected = $false
        $visualReport.suites[0].specs[0].tests[0].results[0].retry = 1
        try {
            Assert-Sprint8BFocusedPlaywrightReport -Report $visualReport `
                -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identity.visual | Out-Null
        } catch { $retryRejected = $true }
        if (-not $retryRejected) {
            throw "Dataset browser proof self-test accepted a retried visual predicate."
        }
        $visualReport.suites[0].specs[0].tests[0].results[0].retry = 0
        $visualReport.config.updateSnapshots = "missing"
        $snapshotUpdateRejected = $false
        try {
            Assert-Sprint8BFocusedPlaywrightReport -Report $visualReport `
                -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identity.visual | Out-Null
        } catch { $snapshotUpdateRejected = $true }
        if (-not $snapshotUpdateRejected) {
            throw "Dataset browser proof self-test accepted snapshot generation mode."
        }

        $scriptText = Get-Content -LiteralPath $PSCommandPath -Raw
        $preflightPosition = $scriptText.LastIndexOf(
            '$snapshotPreflight = Assert-Sprint8BDatasetSnapshotPreflight',
            [StringComparison]::Ordinal
        )
        $authorizationPosition = $scriptText.LastIndexOf(
            'Assert-Sprint8BResetAuthorization -ComposeProject $ComposeProject',
            [StringComparison]::Ordinal
        )
        $materializationPosition = $scriptText.LastIndexOf(
            '$materializeArguments = @{',
            [StringComparison]::Ordinal
        )
        $ownedTeardownPosition = $scriptText.LastIndexOf(
            'if ($ownedTopology)',
            [StringComparison]::Ordinal
        )
        if ($preflightPosition -lt 0 -or $authorizationPosition -le $preflightPosition -or
            $materializationPosition -le $authorizationPosition -or
            $ownedTeardownPosition -le $materializationPosition) {
            throw "Dataset browser proof self-test found unsafe preflight/materialization/teardown ordering."
        }

        [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8b"
            proof = "dataset-browser-proof-contract-self-test"
            state = "passed"
            manifest_identity_count = $identity.all.Count
            module_identity_count = $moduleProof.passed
            visual_identity_count = $visualProof.passed
            source_derived_snapshot_count = $preflight.source_derived_count
            missing_baseline_rejected_before_topology = $missingRejected
            retry_tamper_rejected = $retryRejected
            snapshot_generation_mode_rejected = $snapshotUpdateRejected
            owned_topology_teardown_order = "passed"
            database_used = $false
            topology_created = $false
        }
    } finally {
        if (Test-Path -LiteralPath $temporary -PathType Container) {
            Remove-Item -LiteralPath $temporary -Recurse -Force
        }
        if (Test-Path -LiteralPath $temporary) {
            throw "Dataset browser proof self-test cleanup was not exact."
        }
    }
}

if ($SelfTest) {
    Test-Sprint8BDatasetBrowserProofContract | ConvertTo-Json -Depth 100
    return
}

$identityContract = Get-Sprint8BDatasetBrowserIdentityContract
$visualSourcePath = Join-Path $repoRoot "end2end/tests/module-ui-visual.spec.ts"
$snapshotDirectory = Join-Path $repoRoot "end2end/tests/module-ui-visual.spec.ts-snapshots"
# This preflight is intentionally before reset authorization, source cleanliness,
# materialization, or any Docker call. Missing/unreviewed visual baselines can
# never create a topology or be manufactured by this proof harness.
$snapshotPreflight = Assert-Sprint8BDatasetSnapshotPreflight `
    -VisualSourcePath $visualSourcePath -SnapshotDirectory $snapshotDirectory -RequireTracked

Assert-Sprint8BResetAuthorization -ComposeProject $ComposeProject `
    -Authorized ([bool]$AuthorizeDisposableReset)
$source = Get-Sprint8BSourceIdentity -RequireClean
$composePath = Resolve-Sprint8BRepositoryPath -Path $ComposeFile
$evidenceFullPath = Resolve-Sprint8BRepositoryPath -Path $EvidencePath
$evidenceDirectory = Split-Path -Parent $evidenceFullPath
[IO.Directory]::CreateDirectory($evidenceDirectory) | Out-Null
$baseName = [IO.Path]::GetFileNameWithoutExtension($evidenceFullPath)
$moduleReportPath = Join-Path $evidenceDirectory "$baseName.module.json"
$moduleJunitPath = Join-Path $evidenceDirectory "$baseName.module.xml"
$moduleLogPath = Join-Path $evidenceDirectory "$baseName.module.log"
$visualReportPath = Join-Path $evidenceDirectory "$baseName.visual.json"
$visualJunitPath = Join-Path $evidenceDirectory "$baseName.visual.xml"
$visualLogPath = Join-Path $evidenceDirectory "$baseName.visual.log"
$materializationPath = Join-Path $evidenceDirectory "$baseName.materialization.json"
$environmentNames = @(
    "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
    "TESSARA_SUPERVISOR_PORT", "PLAYWRIGHT_BASE_URL", "TESSARA_PLAYWRIGHT_ACCEPTANCE",
    "TESSARA_PLAYWRIGHT_DATA_STATE", "PLAYWRIGHT_JSON_OUTPUT_FILE",
    "PLAYWRIGHT_JUNIT_OUTPUT_FILE"
)
$environmentBefore = Get-Sprint8BProcessEnvironmentSnapshot -Names $environmentNames
$ownedTopology = $false
$ports = $null
$materialization = $null
$moduleCommand = $null
$visualCommand = $null
$moduleProof = $null
$visualProof = $null
$cleanup = [pscustomobject][ordered]@{ state = "not_started" }
$failure = $null
try {
    $materializeArguments = @{
        Target = "Reference"
        ComposeProject = $ComposeProject
        ComposeFile = $ComposeFile
        EvidencePath = $materializationPath
        AuthorizeDisposableReset = $true
        KeepTopology = $true
    }
    if ($SkipBuild) { $materializeArguments.SkipBuild = $true }
    & (Join-Path $PSScriptRoot "materialize-sprint-8b.ps1") @materializeArguments | Out-Host
    if (-not $?) { throw "Dataset browser proof Reference materialization failed." }
    # From this point the child has promised a retained topology. Teardown must
    # not depend on successfully parsing the child's receipt or port context.
    $ownedTopology = $true
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $materializationPath `
        -SidecarPath "$materializationPath.sha256")) {
        throw "Dataset browser proof materialization receipt is not authenticated."
    }
    $materialization = Get-Content -LiteralPath $materializationPath -Raw |
        ConvertFrom-Json -Depth 100
    if ([string]$materialization.state -cne "passed" -or
        [string]$materialization.target -cne "Reference" -or
        [string]$materialization.compose_project -cne $ComposeProject -or
        [string]$materialization.cleanup_restoration.mode -cne "retained-for-caller" -or
        [string]$materialization.cleanup_restoration.state -cne "passed" -or
        [string]$materialization.source.commit -cne [string]$source.commit -or
        [string]$materialization.source.tree -cne [string]$source.tree -or
        [bool]$materialization.source.dirty) {
        throw "Dataset browser proof materialization did not retain the exact healthy source topology."
    }
    $ports = Set-Sprint8BComposeEnvironment -ComposeProject $ComposeProject `
        -GatewayPort ([int]$materialization.environment.TESSARA_GATEWAY_PORT) `
        -CorePort ([int]$materialization.environment.TESSARA_CORE_CONTROL_PORT) `
        -SupervisorPort ([int]$materialization.environment.TESSARA_SUPERVISOR_PORT)
    Assert-Sprint8BExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
    $env:PLAYWRIGHT_BASE_URL = $ports.gateway_url
    $env:TESSARA_PLAYWRIGHT_ACCEPTANCE = "1"
    $env:TESSARA_PLAYWRIGHT_DATA_STATE = "fresh"

    $moduleCommand = Invoke-Sprint8BFocusedPlaywrightCommand `
        -Arguments @(
            "--prefix", "end2end", "test", "--", "tests/datasets-module.spec.ts",
            "--update-snapshots=none"
        ) `
        -ReportPath $moduleReportPath -JunitPath $moduleJunitPath -LogPath $moduleLogPath
    $moduleReport = Get-Content -LiteralPath $moduleReportPath -Raw | ConvertFrom-Json -Depth 100
    $moduleProof = Assert-Sprint8BFocusedPlaywrightReport -Report $moduleReport `
        -ExpectedFile "datasets-module.spec.ts" -ExpectedIdentities $identityContract.module

    $visualCommand = Invoke-Sprint8BFocusedPlaywrightCommand `
        -Arguments @(
            "--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Datasets",
            "--update-snapshots=none"
        ) `
        -ReportPath $visualReportPath -JunitPath $visualJunitPath -LogPath $visualLogPath
    $visualReport = Get-Content -LiteralPath $visualReportPath -Raw | ConvertFrom-Json -Depth 100
    $visualProof = Assert-Sprint8BFocusedPlaywrightReport -Report $visualReport `
        -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identityContract.visual
} catch {
    $failure = $_
} finally {
    try {
        if ($ownedTopology) {
            $env:COMPOSE_PROJECT_NAME = $ComposeProject
            $cleanup = Remove-Sprint8BProjectTopology -ComposePath $composePath `
                -ComposeProject $ComposeProject -Authorized $true
            $cleanup | Add-Member -NotePropertyName state -NotePropertyValue "passed" -Force
            $cleanup | Add-Member -NotePropertyName mode `
                -NotePropertyValue "exact-owned-reference-teardown" -Force
        }
    } catch {
        if ($null -eq $failure) { $failure = $_ }
        $cleanup = [pscustomobject][ordered]@{
            state = "failed"
            mode = "exact-owned-reference-teardown"
            error = $_.Exception.Message
        }
    } finally {
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }
}

$materializationSha = if (Test-Path -LiteralPath $materializationPath -PathType Leaf) {
    (Get-FileHash -LiteralPath $materializationPath -Algorithm SHA256).Hash.ToLowerInvariant()
} else { $null }
$fingerprintMaterial = @(
    [string]$source.commit,
    [string]$source.tree,
    $ComposeProject,
    [string]$materializationSha,
    (@($snapshotPreflight.files | ForEach-Object { "$($_.name):$($_.sha256)" }) -join "`n")
) -join "`n"
$environmentFingerprint = Get-Sprint7ASha256 -Text ($fingerprintMaterial + "`n")
$passedCount = if ($null -ne $moduleProof -and $null -ne $visualProof) {
    [int]$moduleProof.passed + [int]$visualProof.passed
} else { 0 }
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8b"
    proof = "owned-reference-focused-dataset-browser-and-visual"
    state = if ($null -eq $failure -and $passedCount -eq 11 -and
        [string]$cleanup.state -ceq "passed") { "passed" } else { "failed" }
    compose_project = $ComposeProject
    source = $source
    environment_fingerprint_sha256 = $environmentFingerprint
    environment = if ($null -eq $ports) { $null } else { [pscustomobject][ordered]@{
        COMPOSE_PROJECT_NAME = $ComposeProject
        TESSARA_GATEWAY_PORT = [string]$ports.gateway_port
        TESSARA_CORE_CONTROL_PORT = [string]$ports.core_port
        TESSARA_SUPERVISOR_PORT = [string]$ports.supervisor_port
        TESSARA_PLAYWRIGHT_DATA_STATE = "fresh"
        fingerprint_sha256 = $environmentFingerprint
    } }
    snapshot_preflight = $snapshotPreflight
    materialization = [pscustomobject][ordered]@{
        path = $materializationPath
        sha256 = $materializationSha
        target = if ($null -eq $materialization) { $null } else { [string]$materialization.target }
        fixture_receipt_path = if ($null -eq $materialization) { $null } else {
            [string]$materialization.fixture_receipt_path
        }
    }
    commands = @($moduleCommand, $visualCommand | Where-Object { $null -ne $_ })
    playwright = [pscustomobject][ordered]@{
        state = if ($passedCount -eq 11) { "passed" } else { "failed" }
        data_state = "fresh"
        expected = 11
        passed = $passedCount
        skipped = if ($passedCount -eq 11) { 0 } else { $null }
        workers = 1
        retries = 0
        update_snapshots = "none"
        forbid_only = $true
        module = $moduleProof
        visual = $visualProof
        identities = if ($passedCount -eq 11) {
            @($moduleProof.identities + $visualProof.identities)
        } else { @() }
        reports = @(
            foreach ($path in @($moduleReportPath, $visualReportPath, $moduleJunitPath, $visualJunitPath)) {
                if (Test-Path -LiteralPath $path -PathType Leaf) {
                    [pscustomobject][ordered]@{
                        path = $path
                        sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
                    }
                }
            }
        )
    }
    cleanup_restoration = $cleanup
    failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
        message = $failure.Exception.Message
        category = [string]$failure.CategoryInfo.Category
    } }
}
Publish-Sprint8BHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Focused Dataset browser proof failed; retained evidence: $evidenceFullPath"
}

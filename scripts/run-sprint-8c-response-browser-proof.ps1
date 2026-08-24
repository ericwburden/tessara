[CmdletBinding()]
param(
    [string]$ComposeProject = "tessara-s8c-implementation-response-browser",
    [string]$ComposeFile = "deploy/sprint-8c/compose.yaml",
    [string]$EvidencePath = "target/sprint-8c-response-browser-proof/result.json",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8c-harness-isolation.ps1")
$SelfTest = $requestedSelfTest

function Get-Sprint8CResponseBrowserIdentityContract {
    $manifestPath = Join-Path $repoRoot "end2end/acceptance-manifest.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Sprint 8C Response browser proof requires the tracked acceptance manifest."
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -Depth 100
    $moduleFile = @($manifest.files | Where-Object { [string]$_.path -ceq "responses-module.spec.ts" })
    $visualFile = @($manifest.files | Where-Object { [string]$_.path -ceq "module-ui-visual.spec.ts" })
    if ($moduleFile.Count -ne 1 -or $visualFile.Count -ne 1) {
        throw "Sprint 8C Response browser proof requires one exact module and visual manifest entry."
    }
    $module = @($moduleFile[0].tests | ForEach-Object { [string]$_ })
    $visual = @($visualFile[0].tests | Where-Object {
        [string]$_ -clike "canonical module UI visual baselines › Responses*"
    } | ForEach-Object { [string]$_ })
    $expectedModule = @(
        "Sprint 8C independent Response module › direct documents use only Response-owned public browser routes",
        "Sprint 8C independent Response module › assignment-only start options reject retired Core start routes",
        "Sprint 8C independent Response module › lifecycle navigation preserves unsaved draft state when discard is declined",
        "Sprint 8C independent Response module › scoped review and module diagnostics remain explicit and nondisclosing"
    )
    $expectedVisual = @(
        "canonical module UI visual baselines › Responses directory at 1440 px (light)",
        "canonical module UI visual baselines › Responses start at 390 px (dark)",
        "canonical module UI visual baselines › Responses draft detail at 1024 px (light)",
        "canonical module UI visual baselines › Responses draft editor at 1440 px (dark)",
        "canonical module UI visual baselines › Responses submitted detail at 390 px (light)",
        "canonical module UI visual baselines › Responses Module Management at 1440 px (light)"
    )
    if (($module -join "`n") -cne ($expectedModule -join "`n") -or
        ($visual -join "`n") -cne ($expectedVisual -join "`n")) {
        throw "Sprint 8C Response browser manifest identities are not the exact frozen 4+6 set."
    }
    [pscustomobject][ordered]@{
        module = $module
        visual = $visual
        all = @($module + $visual)
    }
}

function Assert-Sprint8CExactPropertySet {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Label
    )

    $actual = @($Value.PSObject.Properties.Name | ForEach-Object { [string]$_ } | Sort-Object)
    $expectedSorted = @($Expected | Sort-Object)
    if (($actual -join "`n") -cne ($expectedSorted -join "`n")) {
        throw "$Label does not retain its exact accepted schema."
    }
}

function Get-Sprint8CResponseBaselineContract {
    $cases = @(
        [pscustomobject][ordered]@{ key = "response-directory"; route = "/responses"; role = "administrator"; fixture_key = "populated"; theme = "light"; viewport = "1440x1000"; runtime = "hydrated_direct_load"; screenshot = "response-directory-light-1440x1000.png" },
        [pscustomobject][ordered]@{ key = "response-start"; route = "/responses/new"; role = "administrator"; fixture_key = "assignment_only"; theme = "dark"; viewport = "390x844"; runtime = "hydrated_direct_load"; screenshot = "response-start-dark-390x844.png" },
        [pscustomobject][ordered]@{ key = "response-draft-detail"; route = "/responses/f9f5392c-ffe6-4842-9c54-9f0225d6b7f4"; role = "administrator"; fixture_key = "draft"; theme = "light"; viewport = "1024x1366"; runtime = "hydrated_direct_load"; screenshot = "response-draft-detail-light-1024x1366.png" },
        [pscustomobject][ordered]@{ key = "response-draft-edit"; route = "/responses/f9f5392c-ffe6-4842-9c54-9f0225d6b7f4/edit"; role = "administrator"; fixture_key = "draft"; theme = "dark"; viewport = "1440x1000"; runtime = "hydrated_direct_load"; screenshot = "response-draft-edit-dark-1440x1000.png" },
        [pscustomobject][ordered]@{ key = "response-submitted-detail"; route = "/responses/09ed36ef-610a-4fdc-8201-4bdab23525e8"; role = "administrator"; fixture_key = "submitted"; theme = "light"; viewport = "390x844"; runtime = "hydrated_direct_load"; screenshot = "response-submitted-detail-light-390x844.png" },
        [pscustomobject][ordered]@{ key = "operations-response-status"; route = "/operations"; role = "administrator"; fixture_key = "provider_status"; theme = "dark"; viewport = "1024x1366"; runtime = "hydrated_direct_load"; screenshot = "operations-response-status-dark-1024x1366.png" },
        [pscustomobject][ordered]@{ key = "module-management-response"; route = "/administration/modules/tessara.responses"; role = "administrator"; fixture_key = "configuration_diagnostics"; theme = "light"; viewport = "1440x1000"; runtime = "hydrated_direct_load"; screenshot = "module-management-response-light-1440x1000.png" },
        [pscustomobject][ordered]@{ key = "response-directory-javascript-disabled"; route = "/responses"; role = "administrator"; fixture_key = "populated"; theme = "system_theme"; viewport = "1024x1366"; runtime = "javascript_disabled_ssr_direct_refresh"; screenshot = "response-directory-javascript-disabled-1024x1366.png" }
    )
    $semantics = [ordered]@{
        "response-directory" = '{"title":"Responses · Tessara","headings":["Responses","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log","Demo Session Log"],"landmarks":["main","aside","nav","header","aside","nav","header","nav"],"body_text_sha256":"212f1669554d6f258dbc560380a07b2c8371000115bf2064abe599a156a4a270"}' | ConvertFrom-Json -Depth 20
        "response-start" = '{"title":"Start Response · Tessara","headings":["Start Response","No assigned responses"],"landmarks":["main","aside","nav","header","aside","nav","nav","header"],"body_text_sha256":"cc03dcf5c62f31d062113fec98fabd31851af1927c2d8432ed0f65f150d88ba7"}' | ConvertFrom-Json -Depth 20
        "response-draft-detail" = '{"title":"Response Detail · Tessara","headings":["Response Detail","Demo Partner Profile","Summary","Workflow Runtime","Response Values","Audit Trail"],"landmarks":["main","aside","nav","header","aside","nav","nav","header","header","header","header"],"body_text_sha256":"05f1e1e7e8e7b031c3cf69a458d7c6d84e295e1b0d27d509c7e927f45b16fcee"}' | ConvertFrom-Json -Depth 20
        "response-draft-edit" = '{"title":"Edit Response · Tessara","headings":["Edit Response","Demo Partner Profile","Partner Profile"],"landmarks":["main","aside","nav","header","aside","nav","nav","header"],"body_text_sha256":"93d8c1f47d7573786f53582e5ec459ffa0a6f5f4274c7734e56f32bf7b7489fb"}' | ConvertFrom-Json -Depth 20
        "response-submitted-detail" = '{"title":"Response Detail · Tessara","headings":["Response Detail","Demo Partner Profile","Summary","Workflow Runtime","Response Values","Audit Trail"],"landmarks":["main","aside","nav","header","aside","nav","nav","header","header","header","header"],"body_text_sha256":"fd5cef8f301a59e0c22ce28e9558c326e4c752d12489d98103d7c7fa6eccccf3"}' | ConvertFrom-Json -Depth 20
        "operations-response-status" = '{"title":"Operations · Tessara","headings":["Operations","Workflow Assignments","Dataset Readiness","Dataset readiness unavailable"],"landmarks":["main","aside","nav","header","aside","nav","header","nav","header","header","header","header","header","header","header","header","header","header"],"body_text_sha256":"ccbd4d791fb92153446b500831d6ff1cc101aac9a91bbcc160de5190c924fa77"}' | ConvertFrom-Json -Depth 20
        "module-management-response" = '{"title":"Module Management · Tessara","headings":["Responses","Definition","Lifecycle assessment","Declaration summary","Current navigation","Configuration","Feature Declarations","Response start","Response draft","Response submission","Response review","Contracts","Capabilities","Dependency assessment","Declared dependencies","Catalog findings","Configuration","Readiness","Health","Findings","Resources/Destinations","Resource types","Semantic destinations","Navigation policy","Descriptor declarations","Source digest"],"landmarks":["main","aside","nav","header","aside","nav","nav","header"],"body_text_sha256":"345cf372157be339f1e0d143fec95439103abf12d12f771eae80ecbf30b1798d"}' | ConvertFrom-Json -Depth 20
        "response-directory-javascript-disabled" = '{"body_text_sha256":"189944dab13636afeac162ba1a4e26527397ee9867958855a94d3ac6ee8da23e","useful_content":true}' | ConvertFrom-Json -Depth 20
    }
    $accessibility = [ordered]@{
        "response-directory" = [pscustomobject][ordered]@{ named_heading_count = 11; landmark_count = 8 }
        "response-start" = [pscustomobject][ordered]@{ named_heading_count = 2; landmark_count = 8 }
        "response-draft-detail" = [pscustomobject][ordered]@{ named_heading_count = 6; landmark_count = 11 }
        "response-draft-edit" = [pscustomobject][ordered]@{ named_heading_count = 3; landmark_count = 8 }
        "response-submitted-detail" = [pscustomobject][ordered]@{ named_heading_count = 6; landmark_count = 11 }
        "operations-response-status" = [pscustomobject][ordered]@{ named_heading_count = 4; landmark_count = 18 }
        "module-management-response" = [pscustomobject][ordered]@{ named_heading_count = 26; landmark_count = 8 }
        "response-directory-javascript-disabled" = [pscustomobject][ordered]@{ body_text_length = 101 }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8c.response-ui-baseline"
        source_commit = "8f6244e8df3e67a25ec537c671016544c1e05c19"
        capture_state = "captured-pre-extraction"
        captured_from = "http://127.0.0.1:8080"
        cases = $cases
        semantics = $semantics
        accessibility = $accessibility
        source_evidence = @(
            [pscustomobject][ordered]@{ path = "crates/tessara-web/src/routes/responses.rs"; sha256 = "3726338b2460a33043548ab2dc7148a1a5b7a47c55a862248e664cb487a4dcf7" },
            [pscustomobject][ordered]@{ path = "crates/tessara-web-responses/src/api.rs"; sha256 = "14e5b498da519626c924301bbaba8885203f01b32a7f08744ddecfdfee817ff4" },
            [pscustomobject][ordered]@{ path = "crates/tessara-web-responses/src/list.rs"; sha256 = "9d3b09d58d8d6b45859ab3e3091b3ebde133cc8703f4e366d58137232d2b6c9e" },
            [pscustomobject][ordered]@{ path = "crates/tessara-web-responses/src/start.rs"; sha256 = "a618c0edd028f2fc175691d1b78757c198c1a713eb827723ec0faaddc3c8b965" },
            [pscustomobject][ordered]@{ path = "crates/tessara-web-responses/src/detail.rs"; sha256 = "fff608d71f09193edd2d2997313bd5c280c938427d0f6834da04938edb3fcb49" },
            [pscustomobject][ordered]@{ path = "crates/tessara-web-responses/src/edit.rs"; sha256 = "b2e45e62166ab2b9b7cb05a72666dc8312aad0198acfbb4ba014f0b56e5edefc" },
            [pscustomobject][ordered]@{ path = "end2end/tests/permissions.spec.ts"; sha256 = "62c09387bd8fd5c4370be000eb92213bdfba3989379fb21937a3feb708ab4538" },
            [pscustomobject][ordered]@{ path = "end2end/tests/workflow-mediated-assignments.spec.ts"; sha256 = "bfafb656e1b7ee89433b0c51e87c537d69fcef85bdc329bf1634e130fb797ae1" }
        )
        delegated_behavior_coverage = [pscustomobject][ordered]@{
            ownership_delegation_restricted_and_empty_states = "end2end/tests/permissions.spec.ts"
            assignment_only_loading_validation_and_submit_states = "end2end/tests/workflow-mediated-assignments.spec.ts"
            stored_and_system_theme_behavior = "end2end/tests/components.spec.ts"
            two_hundred_percent_zoom_and_overflow = "end2end/tests/components.spec.ts"
        }
        required_matrix = [pscustomobject][ordered]@{
            routes = @("/responses", "/responses/new", "/responses/{response_id}", "/responses/{response_id}/edit", "/operations", "/administration/modules/tessara.responses")
            roles = @("owner", "delegate", "manager", "restricted", "administrator")
            states = @("populated", "empty", "loading", "draft", "submitted", "delegated", "restricted", "provider_degraded", "validation_error", "unsaved_dirty")
            themes = @("light", "dark", "stored_theme", "system_theme")
            viewports = @("1440x1000", "1024x1366", "390x844", "200_percent_zoom")
            runtime = @("javascript_disabled_ssr", "direct_refresh", "hydrated", "lifecycle_navigation", "no_external_assets")
        }
    }
}

function Assert-Sprint8CResponseBaselinePreflight {
    param(
        [Parameter(Mandatory)][string]$IndexPath,
        [Parameter(Mandatory)][string]$ScreenshotDirectory,
        [switch]$RequireTracked
    )

    if (-not (Test-Path -LiteralPath $IndexPath -PathType Leaf)) {
        throw "Response visual baseline index is missing before topology creation."
    }
    $index = Get-Content -LiteralPath $IndexPath -Raw | ConvertFrom-Json -Depth 100
    $contract = Get-Sprint8CResponseBaselineContract
    Assert-Sprint8CExactPropertySet -Value $index -Expected @(
        "schema_version", "contract", "source_commit", "capture_state", "captured_from",
        "cases", "source_evidence", "delegated_behavior_coverage", "required_matrix"
    ) -Label "Response visual baseline index"
    if ([int]$index.schema_version -ne [int]$contract.schema_version -or
        [string]$index.contract -cne [string]$contract.contract -or
        [string]$index.source_commit -cne [string]$contract.source_commit -or
        [string]$index.capture_state -cne [string]$contract.capture_state -or
        [string]$index.captured_from -cne [string]$contract.captured_from) {
        throw "Response visual baseline identity is invalid before topology creation."
    }
    $expectedKeys = @($contract.cases.key | ForEach-Object { [string]$_ })
    $expectedScreenshots = @($contract.cases.screenshot | ForEach-Object { [string]$_ })
    if ((@($index.cases.key | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedKeys -join "`n") -or
        (@($index.cases.screenshot | ForEach-Object { [string]$_ }) -join "`n") -cne
            ($expectedScreenshots -join "`n")) {
        throw "Response visual baseline does not retain the exact accepted eight-case matrix."
    }
    if (@($index.cases).Count -ne @($contract.cases).Count) {
        throw "Response visual baseline does not retain exactly eight accepted cases."
    }
    $entries = [Collections.Generic.List[object]]::new()
    Add-Type -AssemblyName System.Drawing
    for ($caseIndex = 0; $caseIndex -lt @($index.cases).Count; $caseIndex++) {
        $case = @($index.cases)[$caseIndex]
        $expectedCase = @($contract.cases)[$caseIndex]
        Assert-Sprint8CExactPropertySet -Value $case -Expected @(
            "key", "route", "role", "fixture_key", "theme", "viewport", "runtime",
            "screenshot", "screenshot_sha256", "semantic_assertions", "console",
            "accessibility", "external_requests"
        ) -Label "Response visual baseline case '$($expectedCase.key)'"
        $actualMetadata = [pscustomobject][ordered]@{
            key = [string]$case.key
            route = [string]$case.route
            role = [string]$case.role
            fixture_key = [string]$case.fixture_key
            theme = [string]$case.theme
            viewport = [string]$case.viewport
            runtime = [string]$case.runtime
            screenshot = [string]$case.screenshot
        }
        if (($actualMetadata | ConvertTo-Json -Compress) -cne
            ($expectedCase | ConvertTo-Json -Compress)) {
            throw "Response visual baseline case '$($expectedCase.key)' metadata is not exact."
        }
        $expectedSemantics = $contract.semantics[[string]$case.key]
        $expectedAccessibility = $contract.accessibility[[string]$case.key]
        Assert-Sprint8CExactPropertySet -Value $case.semantic_assertions `
            -Expected @($expectedSemantics.PSObject.Properties.Name) `
            -Label "Response visual baseline case '$($case.key)' semantic assertions"
        Assert-Sprint8CExactPropertySet -Value $case.console -Expected @("errors") `
            -Label "Response visual baseline case '$($case.key)' console evidence"
        Assert-Sprint8CExactPropertySet -Value $case.accessibility `
            -Expected @($expectedAccessibility.PSObject.Properties.Name) `
            -Label "Response visual baseline case '$($case.key)' accessibility evidence"
        if (($case.semantic_assertions | ConvertTo-Json -Depth 20 -Compress) -cne
                ($expectedSemantics | ConvertTo-Json -Depth 20 -Compress) -or
            ($case.accessibility | ConvertTo-Json -Depth 20 -Compress) -cne
                ($expectedAccessibility | ConvertTo-Json -Depth 20 -Compress) -or
            @($case.console.errors).Count -ne 0 -or @($case.external_requests).Count -ne 0) {
            throw "Response visual baseline case '$($case.key)' semantic/runtime evidence is not exact."
        }
        if ([string]$case.screenshot_sha256 -cnotmatch '^[0-9a-f]{64}$') {
            throw "Response visual baseline case '$($case.key)' has an invalid screenshot SHA-256."
        }
        $name = [string]$case.screenshot
        $path = Join-Path $ScreenshotDirectory $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Response visual baseline '$name' is missing before topology creation."
        }
        $bytes = [IO.File]::ReadAllBytes($path)
        $stream = [IO.MemoryStream]::new($bytes, $false)
        try {
            $image = [Drawing.Image]::FromStream($stream, $true, $true)
            try {
                $expectedDimensions = ([string]$case.viewport).Split("x")
                if ($expectedDimensions.Count -ne 2 -or
                    [int]$image.Width -ne [int]$expectedDimensions[0] -or
                    [int]$image.Height -ne [int]$expectedDimensions[1] -or
                    [string]$image.RawFormat.Guid -cne [string][Drawing.Imaging.ImageFormat]::Png.Guid) {
                    throw "Response visual baseline '$name' does not decode to its exact PNG viewport."
                }
                # Force pixel decoding; FromStream alone can retain a lazy decoder.
                [void]$image.GetPixel(0, 0)
                $decodedWidth = [int]$image.Width
                $decodedHeight = [int]$image.Height
            } finally {
                $image.Dispose()
            }
        } catch {
            throw "Response visual baseline '$name' is not a decodable exact-dimension PNG: $($_.Exception.Message)"
        } finally {
            $stream.Dispose()
        }
        $sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($sha256 -cne [string]$case.screenshot_sha256) {
            throw "Response visual baseline '$name' does not match its accepted SHA-256."
        }
        if ($RequireTracked) {
            $relative = [IO.Path]::GetRelativePath($repoRoot, $path).Replace("\", "/")
            & git ls-files --error-unmatch -- $relative *> $null
            if ($LASTEXITCODE -ne 0) {
                throw "Response visual baseline '$relative' is not tracked source."
            }
        }
        $entries.Add([pscustomobject][ordered]@{
            key = [string]$case.key
            name = $name
            sha256 = $sha256
            size = $bytes.Length
            width = $decodedWidth
            height = $decodedHeight
        })
    }
    Assert-Sprint8CExactPropertySet -Value $index.delegated_behavior_coverage `
        -Expected @($contract.delegated_behavior_coverage.PSObject.Properties.Name) `
        -Label "Response visual delegated behavior coverage"
    Assert-Sprint8CExactPropertySet -Value $index.required_matrix `
        -Expected @("routes", "roles", "states", "themes", "viewports", "runtime") `
        -Label "Response visual required matrix"
    if (($index.source_evidence | ConvertTo-Json -Depth 20 -Compress) -cne
            ($contract.source_evidence | ConvertTo-Json -Depth 20 -Compress) -or
        ($index.delegated_behavior_coverage | ConvertTo-Json -Depth 20 -Compress) -cne
            ($contract.delegated_behavior_coverage | ConvertTo-Json -Depth 20 -Compress) -or
        ($index.required_matrix | ConvertTo-Json -Depth 20 -Compress) -cne
            ($contract.required_matrix | ConvertTo-Json -Depth 20 -Compress)) {
        throw "Response visual baseline source, delegated behavior, or required matrix is not exact."
    }
    if ($RequireTracked) {
        $relativeIndex = [IO.Path]::GetRelativePath($repoRoot, $IndexPath).Replace("\", "/")
        & git ls-files --error-unmatch -- $relativeIndex *> $null
        if ($LASTEXITCODE -ne 0) {
            throw "Response visual baseline index '$relativeIndex' is not tracked source."
        }
    }
    [pscustomobject][ordered]@{
        state = "passed"
        accepted_case_count = $expectedKeys.Count
        keys = $expectedKeys
        files = @($entries)
        required_matrix = $contract.required_matrix
        source_commit = $contract.source_commit
    }
}

function Add-Sprint8CPlaywrightSuiteResults {
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
    $specs = if ($null -eq $Suite.PSObject.Properties['specs']) {
        @()
    } else {
        @($Suite.specs)
    }
    foreach ($spec in $specs) {
        $specTitle = [string]$spec.title
        if ([string]::IsNullOrWhiteSpace($specTitle)) {
            throw "Focused Response Playwright report contains an untitled test."
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
    $children = if ($null -eq $Suite.PSObject.Properties['suites']) {
        @()
    } else {
        @($Suite.suites)
    }
    foreach ($child in $children) {
        Add-Sprint8CPlaywrightSuiteResults -Suite $child -Parents $titles `
            -InheritedFile $file -Tests $Tests
    }
}

function Assert-Sprint8CFocusedPlaywrightReport {
    param(
        [Parameter(Mandatory)]$Report,
        [Parameter(Mandatory)][string]$ExpectedFile,
        [Parameter(Mandatory)][string[]]$ExpectedIdentities
    )

    $tests = [Collections.Generic.List[object]]::new()
    foreach ($suite in @($Report.suites)) {
        Add-Sprint8CPlaywrightSuiteResults -Suite $suite -Parents @() `
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
        throw "Focused Response Playwright report does not prove the exact passing one-worker, zero-retry identity set for '$ExpectedFile'."
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

function Invoke-Sprint8CFocusedPlaywrightCommand {
    param(
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$ReportPath,
        [Parameter(Mandatory)][string]$JunitPath,
        [Parameter(Mandatory)][string]$LogPath
    )

    foreach ($path in @($ReportPath, $JunitPath, $LogPath)) {
        if (Test-Path -LiteralPath $path) {
            throw "Focused Response browser proof refuses to overwrite retained output '$path'."
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
        throw "Focused Response Playwright command exited ${exitCode}: npm $($Arguments -join ' ')"
    }
    foreach ($path in @($ReportPath, $JunitPath, $LogPath)) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
            [long](Get-Item -LiteralPath $path).Length -eq 0) {
            throw "Focused Response Playwright command omitted retained output '$path'."
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

function New-Sprint8CPlaywrightSelfTestReport {
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
            title = $File
            file = $File
            specs = @()
            suites = @([pscustomobject][ordered]@{
                title = $describeTitle
                file = $File
                specs = @($specs)
            })
        })
        stats = [pscustomobject][ordered]@{
            expected = $Identities.Count
            skipped = 0
            unexpected = 0
            flaky = 0
        }
    }
}

function Test-Sprint8CResponseBrowserProofContract {
    $identity = Get-Sprint8CResponseBrowserIdentityContract
    $temporary = Join-Path $repoRoot "target/sprint-8c-response-browser-selftest-$([guid]::NewGuid().ToString('N'))"
    $baselineDirectory = Join-Path $temporary "baseline"
    $indexPath = Join-Path $baselineDirectory "baseline-index.json"
    try {
        [IO.Directory]::CreateDirectory($baselineDirectory) | Out-Null
        $sourceBaselineDirectory = Join-Path $repoRoot "docs/audits/sprint-8c-response-ui-baseline"
        $sourceIndexPath = Join-Path $sourceBaselineDirectory "baseline-index.json"
        $indexDocument = Get-Content -LiteralPath $sourceIndexPath -Raw |
            ConvertFrom-Json -Depth 100
        $keys = @($indexDocument.cases.key | ForEach-Object { [string]$_ })
        $screenshots = @($indexDocument.cases.screenshot | ForEach-Object { [string]$_ })
        [IO.File]::Copy($sourceIndexPath, $indexPath)
        foreach ($screenshot in $screenshots) {
            [IO.File]::Copy(
                (Join-Path $sourceBaselineDirectory $screenshot),
                (Join-Path $baselineDirectory $screenshot)
            )
        }
        $preflight = Assert-Sprint8CResponseBaselinePreflight `
            -IndexPath $indexPath -ScreenshotDirectory $baselineDirectory
        if ($preflight.accepted_case_count -ne 8 -or
            @($preflight.files | Where-Object {
                [int]$_.width -le 0 -or [int]$_.height -le 0
            }).Count -ne 0) {
            throw "Response browser proof self-test did not decode eight accepted baseline cases."
        }
        $missingRejected = $false
        Remove-Item -LiteralPath (Join-Path $baselineDirectory $screenshots[0]) -Force
        try {
            Assert-Sprint8CResponseBaselinePreflight `
                -IndexPath $indexPath -ScreenshotDirectory $baselineDirectory | Out-Null
        } catch { $missingRejected = $_.Exception.Message -match 'before topology creation' }
        if (-not $missingRejected) {
            throw "Response browser proof self-test accepted a missing visual baseline."
        }
        [IO.File]::Copy(
            (Join-Path $sourceBaselineDirectory $screenshots[0]),
            (Join-Path $baselineDirectory $screenshots[0])
        )

        $runtimeTampered = $indexDocument | ConvertTo-Json -Depth 100 |
            ConvertFrom-Json -Depth 100
        $runtimeTampered.cases[0].runtime = "direct_refresh"
        [IO.File]::WriteAllText($indexPath, ($runtimeTampered | ConvertTo-Json -Depth 100),
            [Text.UTF8Encoding]::new($false))
        $runtimeTamperRejected = $false
        try {
            Assert-Sprint8CResponseBaselinePreflight `
                -IndexPath $indexPath -ScreenshotDirectory $baselineDirectory | Out-Null
        } catch { $runtimeTamperRejected = $true }
        if (-not $runtimeTamperRejected) {
            throw "Response browser proof self-test accepted tampered runtime metadata."
        }

        $viewportTampered = $indexDocument | ConvertTo-Json -Depth 100 |
            ConvertFrom-Json -Depth 100
        $viewportTampered.cases[0].viewport = "1024x1366"
        [IO.File]::WriteAllText($indexPath, ($viewportTampered | ConvertTo-Json -Depth 100),
            [Text.UTF8Encoding]::new($false))
        $viewportTamperRejected = $false
        try {
            Assert-Sprint8CResponseBaselinePreflight `
                -IndexPath $indexPath -ScreenshotDirectory $baselineDirectory | Out-Null
        } catch { $viewportTamperRejected = $true }
        if (-not $viewportTamperRejected) {
            throw "Response browser proof self-test accepted tampered viewport metadata."
        }

        $semanticTampered = $indexDocument | ConvertTo-Json -Depth 100 |
            ConvertFrom-Json -Depth 100
        $semanticTampered.cases[0].semantic_assertions.title = "Substituted · Tessara"
        [IO.File]::WriteAllText($indexPath, ($semanticTampered | ConvertTo-Json -Depth 100),
            [Text.UTF8Encoding]::new($false))
        $semanticTamperRejected = $false
        try {
            Assert-Sprint8CResponseBaselinePreflight `
                -IndexPath $indexPath -ScreenshotDirectory $baselineDirectory | Out-Null
        } catch { $semanticTamperRejected = $true }
        if (-not $semanticTamperRejected) {
            throw "Response browser proof self-test accepted tampered semantic metadata."
        }

        $invalidPngDocument = $indexDocument | ConvertTo-Json -Depth 100 |
            ConvertFrom-Json -Depth 100
        $invalidPngPath = Join-Path $baselineDirectory $screenshots[0]
        [IO.File]::WriteAllBytes(
            $invalidPngPath,
            [byte[]](137, 80, 78, 71, 13, 10, 26, 10, 0)
        )
        $invalidPngDocument.cases[0].screenshot_sha256 =
            (Get-FileHash -LiteralPath $invalidPngPath -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::WriteAllText($indexPath, ($invalidPngDocument | ConvertTo-Json -Depth 100),
            [Text.UTF8Encoding]::new($false))
        $invalidPngRejected = $false
        try {
            Assert-Sprint8CResponseBaselinePreflight `
                -IndexPath $indexPath -ScreenshotDirectory $baselineDirectory | Out-Null
        } catch { $invalidPngRejected = $true }
        if (-not $invalidPngRejected) {
            throw "Response browser proof self-test accepted a PNG-signature stub."
        }
        [IO.File]::Copy(
            (Join-Path $sourceBaselineDirectory $screenshots[0]),
            $invalidPngPath,
            $true
        )
        [IO.File]::Copy($sourceIndexPath, $indexPath, $true)

        $moduleReport = New-Sprint8CPlaywrightSelfTestReport `
            -File "responses-module.spec.ts" -Identities $identity.module
        $visualReport = New-Sprint8CPlaywrightSelfTestReport `
            -File "module-ui-visual.spec.ts" -Identities $identity.visual
        $moduleProof = Assert-Sprint8CFocusedPlaywrightReport -Report $moduleReport `
            -ExpectedFile "responses-module.spec.ts" -ExpectedIdentities $identity.module
        $visualProof = Assert-Sprint8CFocusedPlaywrightReport -Report $visualReport `
            -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identity.visual
        $retryRejected = $false
        $visualReport.suites[0].suites[0].specs[0].tests[0].results[0].retry = 1
        try {
            Assert-Sprint8CFocusedPlaywrightReport -Report $visualReport `
                -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identity.visual | Out-Null
        } catch { $retryRejected = $true }
        if (-not $retryRejected) {
            throw "Response browser proof self-test accepted a retried visual predicate."
        }
        $visualReport.suites[0].suites[0].specs[0].tests[0].results[0].retry = 0
        $visualReport.config.updateSnapshots = "missing"
        $snapshotUpdateRejected = $false
        try {
            Assert-Sprint8CFocusedPlaywrightReport -Report $visualReport `
                -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identity.visual | Out-Null
        } catch { $snapshotUpdateRejected = $true }
        if (-not $snapshotUpdateRejected) {
            throw "Response browser proof self-test accepted snapshot generation mode."
        }

        $scriptText = Get-Content -LiteralPath $PSCommandPath -Raw
        $preflightPosition = $scriptText.LastIndexOf(
            '$baselinePreflight = Assert-Sprint8CResponseBaselinePreflight',
            [StringComparison]::Ordinal
        )
        $authorizationPosition = $scriptText.LastIndexOf(
            'Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject',
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
            throw "Response browser proof self-test found unsafe preflight/materialization/teardown ordering."
        }

        [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8c"
            proof = "response-browser-proof-contract-self-test"
            state = "passed"
            manifest_identity_count = $identity.all.Count
            module_identity_count = $moduleProof.passed
            visual_identity_count = $visualProof.passed
            accepted_baseline_case_count = $preflight.accepted_case_count
            missing_baseline_rejected_before_topology = $missingRejected
            runtime_metadata_tamper_rejected = $runtimeTamperRejected
            viewport_metadata_tamper_rejected = $viewportTamperRejected
            semantic_metadata_tamper_rejected = $semanticTamperRejected
            png_signature_stub_rejected = $invalidPngRejected
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
            throw "Response browser proof self-test cleanup was not exact."
        }
    }
}

if ($SelfTest) {
    Test-Sprint8CResponseBrowserProofContract | ConvertTo-Json -Depth 100
    return
}

$identityContract = Get-Sprint8CResponseBrowserIdentityContract
$baselineDirectory = Join-Path $repoRoot "docs/audits/sprint-8c-response-ui-baseline"
$baselineIndexPath = Join-Path $baselineDirectory "baseline-index.json"
# This preflight is intentionally before reset authorization, source cleanliness,
# materialization, or any Docker call. Missing/unreviewed visual baselines can
# never create a topology or be manufactured by this proof harness.
$baselinePreflight = Assert-Sprint8CResponseBaselinePreflight `
    -IndexPath $baselineIndexPath -ScreenshotDirectory $baselineDirectory -RequireTracked

Assert-Sprint8CResetAuthorization -ComposeProject $ComposeProject `
    -Authorized ([bool]$AuthorizeDisposableReset)
$source = Get-Sprint8CSourceIdentity -RequireClean
$composePath = Resolve-Sprint8CRepositoryPath -Path $ComposeFile
$evidenceFullPath = Resolve-Sprint8CRepositoryPath -Path $EvidencePath
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
$environmentBefore = Get-Sprint8CProcessEnvironmentSnapshot -Names $environmentNames
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
    & (Join-Path $PSScriptRoot "materialize-sprint-8c.ps1") @materializeArguments | Out-Host
    if (-not $?) { throw "Response browser proof Reference materialization failed." }
    # From this point the child has promised a retained topology. Teardown must
    # not depend on successfully parsing the child's receipt or port context.
    $ownedTopology = $true
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $materializationPath `
        -SidecarPath "$materializationPath.sha256")) {
        throw "Response browser proof materialization receipt is not authenticated."
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
        throw "Response browser proof materialization did not retain the exact healthy source topology."
    }
    $ports = Set-Sprint8CComposeEnvironment -ComposeProject $ComposeProject `
        -GatewayPort ([int]$materialization.environment.TESSARA_GATEWAY_PORT) `
        -CorePort ([int]$materialization.environment.TESSARA_CORE_CONTROL_PORT) `
        -SupervisorPort ([int]$materialization.environment.TESSARA_SUPERVISOR_PORT)
    Assert-Sprint8CExistingTopology -ComposePath $composePath -ComposeProject $ComposeProject | Out-Null
    $env:PLAYWRIGHT_BASE_URL = $ports.gateway_url
    $env:TESSARA_PLAYWRIGHT_ACCEPTANCE = "1"
    $env:TESSARA_PLAYWRIGHT_DATA_STATE = "fresh"

    $moduleCommand = Invoke-Sprint8CFocusedPlaywrightCommand `
        -Arguments @(
            "--prefix", "end2end", "test", "--", "tests/responses-module.spec.ts",
            "--update-snapshots=none"
        ) `
        -ReportPath $moduleReportPath -JunitPath $moduleJunitPath -LogPath $moduleLogPath
    $moduleReport = Get-Content -LiteralPath $moduleReportPath -Raw | ConvertFrom-Json -Depth 100
    $moduleProof = Assert-Sprint8CFocusedPlaywrightReport -Report $moduleReport `
        -ExpectedFile "responses-module.spec.ts" -ExpectedIdentities $identityContract.module

    $visualCommand = Invoke-Sprint8CFocusedPlaywrightCommand `
        -Arguments @(
            "--prefix", "end2end", "test", "--", "tests/module-ui-visual.spec.ts", "--grep", "Responses",
            "--update-snapshots=none"
        ) `
        -ReportPath $visualReportPath -JunitPath $visualJunitPath -LogPath $visualLogPath
    $visualReport = Get-Content -LiteralPath $visualReportPath -Raw | ConvertFrom-Json -Depth 100
    $visualProof = Assert-Sprint8CFocusedPlaywrightReport -Report $visualReport `
        -ExpectedFile "module-ui-visual.spec.ts" -ExpectedIdentities $identityContract.visual
} catch {
    $failure = $_
} finally {
    try {
        if ($ownedTopology) {
            $env:COMPOSE_PROJECT_NAME = $ComposeProject
            $cleanup = Remove-Sprint8CProjectTopology -ComposePath $composePath `
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
        Restore-Sprint8CProcessEnvironmentSnapshot -Snapshot $environmentBefore
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
    (@($baselinePreflight.files | ForEach-Object { "$($_.key):$($_.name):$($_.sha256)" }) -join "`n")
) -join "`n"
$environmentFingerprint = Get-Sprint7ASha256 -Text ($fingerprintMaterial + "`n")
$passedCount = if ($null -ne $moduleProof -and $null -ne $visualProof) {
    [int]$moduleProof.passed + [int]$visualProof.passed
} else { 0 }
$document = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8c"
    proof = "owned-reference-focused-response-browser-and-visual"
    state = if ($null -eq $failure -and $passedCount -eq 10 -and
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
    accepted_baseline_preflight = $baselinePreflight
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
        state = if ($passedCount -eq 10) { "passed" } else { "failed" }
        data_state = "fresh"
        expected = 10
        passed = $passedCount
        skipped = if ($passedCount -eq 10) { 0 } else { $null }
        workers = 1
        retries = 0
        update_snapshots = "none"
        forbid_only = $true
        module = $moduleProof
        visual = $visualProof
        identities = if ($passedCount -eq 10) {
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
Publish-Sprint8CHarnessEvidence -Document $document -OutputPath $evidenceFullPath | Out-Null
$document | ConvertTo-Json -Depth 100
if ([string]$document.state -cne "passed") {
    throw "Focused Response browser proof failed; retained evidence: $evidenceFullPath"
}

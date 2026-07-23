[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$common = Join-Path $PSScriptRoot 'sprint-6a-acceptance-evidence-common.ps1'
if (-not (Test-Path -LiteralPath $common -PathType Leaf)) { throw "Missing acceptance evidence helper at $common" }
. $common

function Assert-True {
    param([Parameter(Mandatory)][bool]$Condition, [Parameter(Mandatory)][string]$Message)
    if (-not $Condition) { throw $Message }
}

function Assert-ThrowsLike {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$Pattern,
        [Parameter(Mandatory)][string]$Message
    )
    $matched = $false
    $actualMessage = '<no exception>'
    try { & $Action } catch { $actualMessage = $_.Exception.Message; $matched = $actualMessage -like $Pattern }
    Assert-True $matched "$Message Actual: $actualMessage"
}

function Assert-NoPublishTemps {
    param([Parameter(Mandatory)][string]$Directory)
    $temps = @(Get-ChildItem -LiteralPath $Directory -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like '.*.publish-*' -or $_.Name -like '.sprint-6a-acceptance-*.lock' })
    if ($temps.Count -gt 0) { throw "Publication left temporary/lock paths behind: $(@($temps | ForEach-Object FullName) -join ', ')" }
}

function Write-JsonPair {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][object]$Document)
    $json = $Document | ConvertTo-Json -Depth 30
    Write-Sprint6AFileExclusive -Path $Path -Content "$json`n"
    $digest = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Sprint6AFileExclusive -Path "$Path.sha256" -Content "$digest`n"
    $digest
}

function New-TestDeploymentPair {
    param(
        [Parameter(Mandatory)][string]$Path,
        [ValidateSet('fresh')][string]$DataState = 'fresh'
    )
    $document = [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = 'tessara.sprint-6a.deployment-evidence'
        captured_at_utc = [DateTime]::UtcNow.ToString('o')
        snapshot = [pscustomobject][ordered]@{
            base_url = 'http://127.0.0.1:8080'
            source = [pscustomobject][ordered]@{ commit = 'c' * 40; tree = 'd' * 40; clean = $true }
            release_image = [pscustomobject][ordered]@{ image_id = 'sha256:' + ('e' * 64) }
            database_runtime = [pscustomobject][ordered]@{ current_database = "self_test_$DataState" }
            installation = [pscustomobject][ordered]@{ id = '11111111-2222-3333-4444-555555555555' }
            catalog = [pscustomobject][ordered]@{ definition_count = 8 }
            built_in_seed = [pscustomobject][ordered]@{ canonical_sha256 = $script:Sprint6AAcceptanceEvidenceBuiltInSeedSha256 }
            data = [pscustomobject][ordered]@{ state = $DataState }
        }
    }
    $digest = Write-JsonPair -Path $Path -Document $document
    [pscustomobject][ordered]@{ path = $Path; digest = $digest; document = $document }
}

function New-TestEvidence {
    param(
        [Parameter(Mandatory)][object]$Deployment,
        [ValidateSet('smoke', 'uat')][string]$Kind = 'smoke',
        [string]$Marker = 'initial'
    )
    $runnerLeaf = if ($Kind -eq 'smoke') { 'smoke.ps1' } else { 'uat-sprint.ps1' }
    $runnerPath = Join-Path $PSScriptRoot $runnerLeaf
    $runnerName = if ($Kind -eq 'smoke') { 'scripts/smoke.ps1' } else { 'scripts/uat-sprint.ps1' }
    $snapshot = $Deployment.document.snapshot
    $result = if ($Kind -eq 'smoke') {
        [pscustomobject][ordered]@{
            dataset_rows = 52
            component_rows = 50
            seeded_visual_points = 26
            visual_points = if ($Marker -eq 'initial') { 20 } else { 21 }
        }
    } else {
        [pscustomobject][ordered]@{
            seed_version = 'uat-demo-v2'
            dashboard_placements = 9
            component_kinds = @($script:Sprint6AUatComponentKinds)
            authorization_roles_checked = @($script:Sprint6AUatAuthorizationRoles)
        }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-6a.$Kind"
        status = 'passed'
        completed_at_utc = [DateTime]::UtcNow.ToString('o')
        expected_data_state = [string]$snapshot.data.state
        base_url = [string]$snapshot.base_url
        runner = [pscustomobject][ordered]@{
            path = $runnerName
            sha256 = (Get-FileHash -LiteralPath $runnerPath -Algorithm SHA256).Hash.ToLowerInvariant()
        }
        checks = @(Get-Sprint6AAcceptanceExpectedChecks -Kind $Kind)
        deployment = [pscustomobject][ordered]@{
            evidence_sha256 = [string]$Deployment.digest
            data_state = [string]$snapshot.data.state
            source_commit = [string]$snapshot.source.commit
            source_tree = [string]$snapshot.source.tree
            image_id = [string]$snapshot.release_image.image_id
            database_name = [string]$snapshot.database_runtime.current_database
            installation_id = [string]$snapshot.installation.id
            catalog_entries = [int]$snapshot.catalog.definition_count
            built_in_seed_sha256 = [string]$snapshot.built_in_seed.canonical_sha256
        }
        result = $result
    }
}

function Get-TestRunner {
    param([ValidateSet('smoke', 'uat')][string]$Kind = 'smoke')
    $runnerLeaf = if ($Kind -eq 'smoke') { 'smoke.ps1' } else { 'uat-sprint.ps1' }
    Join-Path $PSScriptRoot $runnerLeaf
}

$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('tessara-acceptance-evidence-selftest-' + [guid]::NewGuid().ToString('N'))
$environmentNames = @('TESSARA_ACCEPTANCE_EVIDENCE_SELFTEST_EXISTING', 'TESSARA_ACCEPTANCE_EVIDENCE_SELFTEST_ABSENT')
$outerEnvironment = Get-Sprint6AProcessEnvironmentSnapshot -Names $environmentNames
$junctionPath = $null

try {
    [IO.Directory]::CreateDirectory($testRoot) | Out-Null
    Assert-True `
        (Test-Sprint6AShouldInvokeDemoSeed -ExpectedDataState fresh) `
        'Fresh acceptance must retain the demo seed path.'
    Assert-True `
        (Test-Sprint6AShouldInvokeDemoSeed -ExpectedDataState $null) `
        'Development diagnostics must retain the established demo seed path.'
    Assert-ThrowsLike {
        Test-Sprint6AShouldInvokeDemoSeed -ExpectedDataState upgraded | Out-Null
    } '*unsupported data state*' 'The retired upgraded acceptance state was not rejected.'
    Assert-ThrowsLike {
        Test-Sprint6AShouldInvokeDemoSeed -ExpectedDataState unexpected | Out-Null
    } '*unsupported data state*' 'An unknown demo seed strategy state was not rejected.'

    $demoSeedMessage = $script:Sprint6ADemoSeedRefusalMessage
    $demoSeedRefusal = [pscustomobject][ordered]@{
        code = 'bad_request'
        message = $demoSeedMessage
        error = $demoSeedMessage
    } | ConvertTo-Json -Compress
    Assert-Sprint6ADemoSeedRefusal `
        -StatusCode 400 `
        -ResponseBody $demoSeedRefusal `
        -Context 'Acceptance self-test demo seed'
    Assert-ThrowsLike {
        Assert-Sprint6ADemoSeedRefusal -StatusCode 500 -ResponseBody $demoSeedRefusal -Context 'Acceptance self-test demo seed'
    } '*HTTP 500*' 'A non-400 demo seed failure was accepted as the populated-database refusal.'
    Assert-ThrowsLike {
        Assert-Sprint6ADemoSeedRefusal -StatusCode 400 -ResponseBody '{' -Context 'Acceptance self-test demo seed'
    } '*non-JSON*' 'A malformed demo seed refusal was accepted.'
    $wrongDemoSeedRefusal = [pscustomobject][ordered]@{
        code = 'bad_request'
        message = 'different failure'
        error = 'different failure'
    } | ConvertTo-Json -Compress
    Assert-ThrowsLike {
        Assert-Sprint6ADemoSeedRefusal -StatusCode 400 -ResponseBody $wrongDemoSeedRefusal -Context 'Acceptance self-test demo seed'
    } '*exact populated-database*' 'A different HTTP 400 was accepted as the demo seed refusal.'

    $deployment = New-TestDeploymentPair -Path (Join-Path $testRoot 'deployment-fresh.json')
    $evidencePath = Join-Path $testRoot 'smoke-fresh.json'
    $firstEvidence = New-TestEvidence -Deployment $deployment
    $published = Publish-Sprint6AAcceptanceEvidence `
        -EvidencePath $evidencePath `
        -DeploymentEvidencePath $deployment.path `
        -RunnerFilePath (Get-TestRunner) `
        -Evidence $firstEvidence
    $verifiedDigest = Assert-Sprint6AAcceptanceEvidenceDigest `
        -EvidencePath $evidencePath `
        -DeploymentEvidencePath $deployment.path `
        -RunnerFilePath (Get-TestRunner)
    Assert-True ($verifiedDigest -ceq $published.sha256) 'Success evidence/sidecar validation did not match publication.'
    Assert-NoPublishTemps -Directory $testRoot

    $uatDeployment = New-TestDeploymentPair -Path (Join-Path $testRoot 'deployment-uat-fresh.json') -DataState fresh
    $uatPath = Join-Path $testRoot 'uat-fresh.json'
    Publish-Sprint6AAcceptanceEvidence `
        -EvidencePath $uatPath `
        -DeploymentEvidencePath $uatDeployment.path `
        -RunnerFilePath (Get-TestRunner -Kind uat) `
        -Evidence (New-TestEvidence -Deployment $uatDeployment -Kind uat) | Out-Null
    Assert-Sprint6AAcceptanceEvidenceDigest `
        -EvidencePath $uatPath `
        -DeploymentEvidencePath $uatDeployment.path `
        -RunnerFilePath (Get-TestRunner -Kind uat) | Out-Null

    $originalEvidence = Get-Content -LiteralPath $evidencePath -Raw
    $originalDigest = Get-Content -LiteralPath "$evidencePath.sha256" -Raw
    Assert-ThrowsLike {
        Publish-Sprint6AAcceptanceEvidence -EvidencePath $evidencePath -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner) -Evidence (New-TestEvidence -Deployment $deployment -Marker refused) | Out-Null
    } '*refuses to replace*' 'Existing pair was not refused without overwrite.'
    Assert-True ((Get-Content -LiteralPath $evidencePath -Raw) -ceq $originalEvidence) 'Overwrite refusal changed evidence.'
    Assert-True ((Get-Content -LiteralPath "$evidencePath.sha256" -Raw) -ceq $originalDigest) 'Overwrite refusal changed sidecar.'

    $orphanPath = Join-Path $testRoot 'orphan.json'
    Write-Sprint6AFileExclusive -Path "$orphanPath.sha256" -Content ('f' * 64)
    Assert-ThrowsLike {
        Publish-Sprint6AAcceptanceEvidence -EvidencePath $orphanPath -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner) -Evidence (New-TestEvidence -Deployment $deployment) | Out-Null
    } '*refuses to replace*' 'Orphan sidecar was not refused.'
    Assert-True (-not (Test-Path -LiteralPath $orphanPath)) 'Orphan refusal created evidence.'

    foreach ($aliasPath in @($deployment.path, "$($deployment.path).sha256", (Join-Path $testRoot '.\deployment-fresh.json'))) {
        Assert-ThrowsLike {
            Assert-Sprint6AAcceptanceEvidenceTargetAvailable -EvidencePath $aliasPath -DeploymentEvidencePath $deployment.path -Overwrite | Out-Null
        } '*distinct*' "Deployment/acceptance alias '$aliasPath' was not rejected before mutation."
    }
    Assert-True ((Get-FileHash -LiteralPath $deployment.path -Algorithm SHA256).Hash.ToLowerInvariant() -ceq $deployment.digest) 'Alias rejection mutated deployment evidence.'

    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceTargetAvailable -EvidencePath "$evidencePath`:secret" -DeploymentEvidencePath $deployment.path -Overwrite | Out-Null
    } '*Alternate data streams*' 'ADS output path was not rejected.'

    $hardlinkDeployment = New-TestDeploymentPair -Path (Join-Path $testRoot 'hardlink-deployment.json')
    $hardlinkAlias = Join-Path $testRoot 'hardlink-alias.json'
    New-Item -ItemType HardLink -Path $hardlinkAlias -Target $hardlinkDeployment.path | Out-Null
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceTargetAvailable -EvidencePath (Join-Path $testRoot 'hardlink-output.json') -DeploymentEvidencePath $hardlinkDeployment.path | Out-Null
    } '*Hard-linked*' 'Hard-linked deployment input was not rejected.'
    Remove-Item -LiteralPath $hardlinkAlias -Force

    $junctionTarget = Join-Path $testRoot 'junction-target'
    [IO.Directory]::CreateDirectory($junctionTarget) | Out-Null
    $junctionPath = Join-Path $testRoot 'junction-alias'
    New-Item -ItemType Junction -Path $junctionPath -Target $junctionTarget | Out-Null
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceTargetAvailable -EvidencePath (Join-Path $junctionPath 'output.json') -DeploymentEvidencePath $deployment.path | Out-Null
    } '*Reparse points*' 'Junction ancestor was not rejected.'
    Remove-Item -LiteralPath $junctionPath -Force
    $junctionPath = $null

    $concurrentPath = Join-Path $testRoot 'concurrent.json'
    $publicationLock = New-Sprint6AAcceptancePublicationLock -EvidencePath $concurrentPath
    try {
        Assert-ThrowsLike {
            Publish-Sprint6AAcceptanceEvidence -EvidencePath $concurrentPath -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner) -Evidence (New-TestEvidence -Deployment $deployment) | Out-Null
        } '*concurrent*' 'Exclusive publication reservation did not reject a concurrent publisher.'
    } finally { $publicationLock.Dispose() }
    Assert-True (-not (Test-Path -LiteralPath $concurrentPath)) 'Concurrent publication created a partial artifact.'
    Assert-NoPublishTemps -Directory $testRoot

    $unknown = New-TestEvidence -Deployment $deployment
    $unknown | Add-Member -NotePropertyName token -NotePropertyValue 'must-not-persist'
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDocument -Evidence $unknown -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner)
    } '*fields must be exactly*' 'Unknown/raw secret root field was accepted.'
    $bogusType = New-TestEvidence -Deployment $deployment
    $bogusType.schema_version = '1'
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDocument -Evidence $bogusType -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner)
    } '*integral number*' 'String schema_version was accepted.'
    $bogusRunner = New-TestEvidence -Deployment $deployment
    $bogusRunner.runner.sha256 = '0' * 64
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDocument -Evidence $bogusRunner -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner)
    } '*runner name or SHA*' 'Bogus runner digest was accepted.'
    $bogusChecks = New-TestEvidence -Deployment $deployment
    $bogusChecks.checks = @('self_test')
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDocument -Evidence $bogusChecks -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner)
    } '*checks must be exactly*' 'Non-production check list was accepted.'
    $bogusIdentity = New-TestEvidence -Deployment $deployment
    $bogusIdentity.deployment.installation_id = [guid]::Empty.ToString()
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDocument -Evidence $bogusIdentity -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner)
    } '*deployment identity*' 'Zero/bogus deployment installation identity was accepted.'
    $secretValue = New-TestEvidence -Deployment $deployment
    $secretValue.result = [pscustomobject][ordered]@{ dataset_rows = 1; component_rows = 1; seeded_visual_points = 1; visual_points = 1; password = 'raw' }
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDocument -Evidence $secretValue -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner)
    } '*fields must be exactly*' 'Raw secret result field was accepted.'

    foreach ($failurePoint in @('AfterEvidencePublish', 'FinalValidationFailure')) {
        Assert-ThrowsLike {
            Publish-Sprint6AAcceptanceEvidence -EvidencePath $evidencePath -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner) -Evidence (New-TestEvidence -Deployment $deployment -Marker $failurePoint) -Overwrite -FailurePoint $failurePoint | Out-Null
        } '*Injected*' "$failurePoint did not trigger."
        Assert-True ((Get-Content -LiteralPath $evidencePath -Raw) -ceq $originalEvidence) "$failurePoint did not restore prior evidence byte-for-byte."
        Assert-True ((Get-Content -LiteralPath "$evidencePath.sha256" -Raw) -ceq $originalDigest) "$failurePoint did not restore prior digest byte-for-byte."
        Assert-NoPublishTemps -Directory $testRoot
    }

    $replacement = Publish-Sprint6AAcceptanceEvidence -EvidencePath $evidencePath -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner) -Evidence (New-TestEvidence -Deployment $deployment -Marker replacement) -Overwrite
    Assert-True ($replacement.sha256 -cne $published.sha256) 'Authorized overwrite did not replace evidence.'
    Add-Content -LiteralPath $evidencePath -Value 'tampered'
    Assert-ThrowsLike {
        Assert-Sprint6AAcceptanceEvidenceDigest -EvidencePath $evidencePath -DeploymentEvidencePath $deployment.path -RunnerFilePath (Get-TestRunner) | Out-Null
    } '*SHA-256 verification failed*' 'Sidecar validation did not detect tampering.'

    [Environment]::SetEnvironmentVariable($environmentNames[0], 'original-value', [EnvironmentVariableTarget]::Process)
    Remove-Item -LiteralPath "Env:$($environmentNames[1])" -Force -ErrorAction SilentlyContinue
    $environmentSnapshot = Get-Sprint6AProcessEnvironmentSnapshot -Names $environmentNames
    [Environment]::SetEnvironmentVariable($environmentNames[0], 'mutated', [EnvironmentVariableTarget]::Process)
    [Environment]::SetEnvironmentVariable($environmentNames[1], 'created', [EnvironmentVariableTarget]::Process)
    $sessions = [Collections.Generic.List[object]]::new()
    $paths = [Collections.Generic.List[string]]::new()
    $bearer = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'
    $browser = 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb'
    $preExisting = 'cccccccc-cccc-4ccc-8ccc-cccccccccccc'
    $cookie = Register-Sprint6ASensitivePath -Paths $paths -Path (Join-Path $testRoot 'cleanup-cookie.txt')
    $payload = Register-Sprint6ASensitivePath -Paths $paths -Path (Join-Path $testRoot 'cleanup-login.json')
    Write-Sprint6AFileExclusive -Path $cookie -Content 'cookie-material'
    Write-Sprint6AFileExclusive -Path $payload -Content 'credential-material'
    Register-Sprint6ACurrentRunSession -Sessions $sessions -Source bearer -Token $bearer
    Register-Sprint6ACurrentRunSession -Sessions $sessions -Source browser -Token $browser -CookiePath $cookie
    $logoutCalls = [Collections.Generic.List[string]]::new()
    $logout = { param($session) $logoutCalls.Add([string]$session.token); [pscustomobject]@{ signed_out = $true } }
    Complete-Sprint6AAcceptanceRunCleanup -Sessions $sessions -SensitivePaths $paths -LogoutAction $logout -EnvironmentSnapshot $environmentSnapshot
    Assert-True ($logoutCalls.Count -eq 2 -and $logoutCalls -ccontains $bearer -and $logoutCalls -ccontains $browser) 'Cleanup did not revoke exactly every current-run session.'
    Assert-True ($logoutCalls -cnotcontains $preExisting) 'Cleanup revoked an unregistered pre-existing session.'
    Assert-True (-not (Test-Path -LiteralPath $cookie) -and -not (Test-Path -LiteralPath $payload)) 'Cleanup retained cookie/payload files.'
    Assert-Sprint6AProcessEnvironmentRestored -Snapshot $environmentSnapshot

    $shimSessions = [Collections.Generic.List[object]]::new()
    $shimPaths = [Collections.Generic.List[string]]::new()
    $shimToken = 'eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee'
    $shimCookie = Register-Sprint6ASensitivePath -Paths $shimPaths -Path (Join-Path $testRoot 'shim-cookie.txt')
    $shimPayload = Register-Sprint6ASensitivePath -Paths $shimPaths -Path (Join-Path $testRoot 'shim-login.json')
    $shimCommand = Join-Path $testRoot 'curl-login-shim.cmd'
    Write-Sprint6AFileExclusive -Path $shimPayload -Content '{"credential":"must-be-removed"}'
    New-Sprint6AExclusiveEmptyFile -Path $shimCookie
    $cookieLine = "127.0.0.1`tFALSE`t/`tFALSE`t0`ttessara_session`t$shimToken"
    Write-Sprint6AFileExclusive -Path $shimCommand -Content "@echo off`r`n> `"%~4`" echo $cookieLine`r`nexit /b 18`r`n"
    $shimPublishPath = Join-Path $testRoot 'must-not-publish-after-curl-18.json'
    $shimExitObserved = $false
    try {
        $previousNativePreference = $PSNativeCommandUseErrorActionPreference
        $PSNativeCommandUseErrorActionPreference = $false
        try {
            $shimResponse = & $shimCommand -sS -f -c $shimCookie -H 'Content-Type: application/json' --data-binary "@$shimPayload" 'http://127.0.0.1:8080/api/auth/login'
            $shimExitCode = $LASTEXITCODE
        } finally {
            $PSNativeCommandUseErrorActionPreference = $previousNativePreference
        }
        Complete-Sprint6ABrowserLoginObservation `
            -Sessions $shimSessions `
            -CookiePath $shimCookie `
            -Response $shimResponse `
            -CurlExitCode $shimExitCode `
            -Context 'Curl shim browser login' | Out-Null
        Publish-Sprint6AAcceptanceEvidence `
            -EvidencePath $shimPublishPath `
            -DeploymentEvidencePath $deployment.path `
            -RunnerFilePath (Get-TestRunner) `
            -Evidence (New-TestEvidence -Deployment $deployment) | Out-Null
    } catch {
        $shimExitObserved = $_.Exception.Message -like '*exit code 18 after committed-cookie inspection*'
    } finally {
        if (Test-Path -LiteralPath $shimPayload) {
            $null = Get-Sprint6ASecurePathInfo -Path $shimPayload -RequireLeaf
            Remove-Item -LiteralPath $shimPayload -Force
        }
    }
    Assert-True $shimExitObserved 'Curl exit 18 did not fail only after committed-cookie inspection.'
    Assert-True ($shimSessions.Count -eq 1 -and $shimSessions[0].token -ceq $shimToken) 'Curl exit 18 cookie UUID was not registered exactly once.'
    Assert-True (-not (Test-Path -LiteralPath $shimPublishPath) -and -not (Test-Path -LiteralPath "$shimPublishPath.sha256")) 'Curl exit 18 published acceptance evidence.'
    $shimLogoutTokens = [Collections.Generic.List[string]]::new()
    Complete-Sprint6AAcceptanceRunCleanup `
        -Sessions $shimSessions `
        -SensitivePaths $shimPaths `
        -LogoutAction { param($session) $shimLogoutTokens.Add([string]$session.token); [pscustomobject]@{ signed_out = $true } } `
        -FinalAttempt
    Assert-True ($shimLogoutTokens.Count -eq 1 -and $shimLogoutTokens[0] -ceq $shimToken) 'Curl exit 18 session was not logged out exactly once with signed_out=true.'
    Assert-True (-not (Test-Path -LiteralPath $shimCookie) -and -not (Test-Path -LiteralPath $shimPayload)) 'Curl exit 18 retained cookie or credential residue.'

    $protectedPath = Join-Path $testRoot 'cleanup-protected.json'
    Copy-Item -LiteralPath $uatPath -Destination $protectedPath
    Copy-Item -LiteralPath "$uatPath.sha256" -Destination "$protectedPath.sha256"
    $protectedEvidence = Get-Content -LiteralPath $protectedPath -Raw
    $protectedDigest = Get-Content -LiteralPath "$protectedPath.sha256" -Raw
    $failedSessions = [Collections.Generic.List[object]]::new()
    $failedPaths = [Collections.Generic.List[string]]::new()
    $failedCookie = Register-Sprint6ASensitivePath -Paths $failedPaths -Path (Join-Path $testRoot 'failed-cookie.txt')
    Write-Sprint6AFileExclusive -Path $failedCookie -Content 'retry-cookie'
    Register-Sprint6ACurrentRunSession -Sessions $failedSessions -Source browser -Token 'dddddddd-dddd-4ddd-8ddd-dddddddddddd' -CookiePath $failedCookie
    $cleanupFailed = $false
    try {
        Complete-Sprint6AAcceptanceRunCleanup -Sessions $failedSessions -SensitivePaths $failedPaths -LogoutAction { throw 'injected logout failure' }
        Publish-Sprint6AAcceptanceEvidence -EvidencePath $protectedPath -DeploymentEvidencePath $uatDeployment.path -RunnerFilePath (Get-TestRunner -Kind uat) -Evidence (New-TestEvidence -Deployment $uatDeployment -Kind uat) -Overwrite | Out-Null
    } catch { $cleanupFailed = $_.Exception.Message -like '*cleanup failed*' }
    Assert-True $cleanupFailed 'Injected cleanup failure did not block publication.'
    Assert-True ((Get-Content -LiteralPath $protectedPath -Raw) -ceq $protectedEvidence) 'Cleanup failure changed prior evidence.'
    Assert-True ((Get-Content -LiteralPath "$protectedPath.sha256" -Raw) -ceq $protectedDigest) 'Cleanup failure changed prior sidecar.'
    Assert-True (Test-Path -LiteralPath $failedCookie -PathType Leaf) 'Retryable browser cookie was removed before final logout retry.'
    Complete-Sprint6AAcceptanceRunCleanup -Sessions $failedSessions -SensitivePaths $failedPaths -LogoutAction { [pscustomobject]@{ signed_out = $true } } -FinalAttempt
    Assert-True (-not (Test-Path -LiteralPath $failedCookie)) 'Final cleanup retry retained browser cookie.'

    foreach ($acceptanceScript in @('smoke.ps1', 'uat-sprint.ps1')) {
        $forbiddenPath = Join-Path $testRoot "$acceptanceScript-development.json"
        Assert-ThrowsLike {
            & (Join-Path $PSScriptRoot $acceptanceScript) -DevelopmentMode -AcceptanceEvidencePath $forbiddenPath
        } '*DevelopmentMode cannot produce*' "$acceptanceScript did not preserve non-acceptance DevelopmentMode semantics."
        Assert-True (-not (Test-Path -LiteralPath $forbiddenPath) -and -not (Test-Path -LiteralPath "$forbiddenPath.sha256")) "$acceptanceScript emitted DevelopmentMode evidence."
    }

    Write-Host 'Sprint 6A acceptance evidence adversarial self-test passed.' -ForegroundColor Green
} finally {
    Restore-Sprint6AProcessEnvironmentSnapshot -Snapshot $outerEnvironment
    if ($null -ne $junctionPath -and (Test-Path -LiteralPath $junctionPath)) { Remove-Item -LiteralPath $junctionPath -Force }
    if (Test-Path -LiteralPath $testRoot) {
        $resolvedRoot = [IO.Path]::GetFullPath($testRoot)
        $resolvedTemp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
        if (-not $resolvedRoot.StartsWith($resolvedTemp, [StringComparison]::OrdinalIgnoreCase)) { throw 'Self-test cleanup target escaped the resolved temp root.' }
        Remove-Item -LiteralPath $resolvedRoot -Recurse -Force
    }
}

# The adversarial curl shim intentionally sets a nonzero native exit code. A
# successful self-test must not leak that expected failure into its caller.
$global:LASTEXITCODE = 0

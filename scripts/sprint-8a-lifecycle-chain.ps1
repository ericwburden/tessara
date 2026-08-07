Set-StrictMode -Version Latest

if ($PSVersionTable.PSEdition -cne "Core" -or $PSVersionTable.PSVersion.Major -lt 7) {
    throw "Sprint 8A lifecycle commands require PowerShell 7 or newer."
}

. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")

function Open-Sprint8AValidationAttemptLock {
    param([Parameter(Mandatory)][string]$Path)

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    [IO.File]::Open($Path, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
}

function Test-Sprint8ALifecycleExclusiveLock {
    $path = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-lifecycle-lock-$([guid]::NewGuid().ToString('N')).lock"
    $first = Open-Sprint8AValidationAttemptLock -Path $path
    try {
        $rejected = $false
        try {
            $second = Open-Sprint8AValidationAttemptLock -Path $path
            $second.Dispose()
        } catch [IO.IOException] { $rejected = $true }
        if (-not $rejected) { throw "Sprint 8A lifecycle lock admitted a concurrent process." }
    } finally {
        $first.Dispose()
    }
    $reopened = Open-Sprint8AValidationAttemptLock -Path $path
    $reopened.Dispose()
    Remove-Item -LiteralPath $path -Force
}

function Get-Sprint8APreflightCheckNames {
    @(
        "receipt-chain",
        "repository-scope",
        "clean-source",
        "acceptance-traceability",
        "environment-contract",
        "database-contract",
        "deployment-contract",
        "downstream-command-contract",
        "evidence-path-contract",
        "evidence-inventory"
    )
}

function Get-Sprint8ASitLaneNames {
    @("static-and-boundaries", "rust-workspace", "playwright", "deployed-acceptance-smoke")
}

function Get-Sprint8AManualUatScenarioNames {
    @(1..8 | ForEach-Object { "UAT-8A-{0:d2}" -f $_ })
}

function Get-Sprint8AManualUatScenarioContract {
    param([Parameter(Mandatory)][string]$Scenario)

    $contracts = [ordered]@{
        "UAT-8A-01" = [ordered]@{ role = "Component manager and reader"; step_count = 6 }
        "UAT-8A-02" = [ordered]@{ role = "Operator and Dashboard reader"; step_count = 5 }
        "UAT-8A-03" = [ordered]@{ role = "Global Module Management manager and reader"; step_count = 5 }
        "UAT-8A-04" = [ordered]@{ role = "Component manager plus scoped and out-of-scope actors"; step_count = 5 }
        "UAT-8A-05" = [ordered]@{ role = "Component manager, Dashboard manager, and reader"; step_count = 6 }
        "UAT-8A-06" = [ordered]@{ role = "Operator plus authorized and restricted users"; step_count = 3 }
        "UAT-8A-07" = [ordered]@{ role = "Operator"; step_count = 4 }
        "UAT-8A-08" = [ordered]@{ role = "Operator and reviewer"; step_count = 4 }
    }
    if (-not $contracts.Contains($Scenario)) {
        throw "Unknown Sprint 8A manual UAT scenario '$Scenario'."
    }
    $repositoryRoot = Split-Path -Parent $PSScriptRoot
    $documentPath = "docs/sprints/sprint-8a-uat/$($Scenario.ToLowerInvariant()).md"
    $documentFullPath = Join-Path $repositoryRoot $documentPath
    $documentLines = @(Get-Content -LiteralPath $documentFullPath)
    $roleLines = @($documentLines | Where-Object { $_ -match '^- User role:\s*(.+?)\s*$' })
    if ($roleLines.Count -ne 1) {
        throw "Manual UAT scenario '$Scenario' must declare exactly one user role."
    }
    $documentRole = ([regex]::Match($roleLines[0], '^- User role:\s*(.+?)\s*$')).Groups[1].Value
    $steps = @($documentLines | ForEach-Object {
        $match = [regex]::Match($_, '^\|\s*(\d+)\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|')
        if ($match.Success) {
            [pscustomobject][ordered]@{
                step = [int]$match.Groups[1].Value
                action = [string]$match.Groups[2].Value
                expected_result = [string]$match.Groups[3].Value
            }
        }
    })
    if ($documentRole -cne [string]$contracts[$Scenario].role -or
        $steps.Count -ne [int]$contracts[$Scenario].step_count -or
        (($steps.step | Sort-Object) -join ",") -cne ((1..$steps.Count) -join ",") -or
        @($steps | Where-Object {
            [string]::IsNullOrWhiteSpace([string]$_.action) -or
                [string]::IsNullOrWhiteSpace([string]$_.expected_result)
        }).Count -ne 0) {
        throw "Manual UAT scenario '$Scenario' role or executable step contract differs from its canonical inventory."
    }
    [pscustomobject][ordered]@{
        scenario = $Scenario
        role = $documentRole
        step_count = $steps.Count
        steps = $steps
        document = [pscustomobject][ordered]@{
            path = $documentPath
            sha256 = Get-Sprint8AFileSha256 -Path $documentFullPath
        }
    }
}

function Test-Sprint8ASourceIdentityMatch {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual
    )

    Assert-Sprint8ASourceIdentityObject -Source $Expected | Out-Null
    Assert-Sprint8ASourceIdentityObject -Source $Actual | Out-Null
    foreach ($name in @(
        "commit", "tree", "dirty", "branch",
        "acceptance_inventory_sha256", "deployment_inputs_sha256"
    )) {
        if ($Expected.$name -cne $Actual.$name) {
            return $false
        }
    }
    $true
}

function Test-Sprint8AReceiptReferenceMatch {
    param(
        [AllowEmptyCollection()][object[]]$References = @(),
        [Parameter(Mandatory)]$ExpectedReference
    )

    @($References | Where-Object {
        [string]$_.path -ceq [string]$ExpectedReference.path -and
            [string]$_.sha256 -ceq [string]$ExpectedReference.sha256
    }).Count -eq 1
}

function Assert-Sprint8AExactTerminalIdentities {
    param(
        [Parameter(Mandatory)][object[]]$Results,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    $names = @($Results | ForEach-Object { [string]$_.name })
    if ($names.Count -ne $ExpectedNames.Count -or
        @($names | Sort-Object -Unique).Count -ne $names.Count -or
        (($names | Sort-Object) -join "`n") -cne (($ExpectedNames | Sort-Object) -join "`n")) {
        throw "$Label does not contain its exact terminal identity inventory."
    }
    foreach ($result in $Results) {
        if (@("passed", "failed", "blocked") -cnotcontains [string]$result.state -or
            $result.PSObject.Properties.Name -notcontains "assertions_started" -or
            $result.assertions_started -isnot [bool]) {
            throw "$Label result '$($result.name)' has malformed state or assertion-start evidence."
        }
        if ([string]$result.state -ceq "blocked" -and [bool]$result.assertions_started) {
            throw "$Label blocked result '$($result.name)' cannot claim assertions."
        }
        if ([string]$result.state -ceq "blocked" -and
            [string]::IsNullOrWhiteSpace([string]$result.blocked_reason)) {
            throw "$Label blocked result '$($result.name)' lacks its exact dependency reason."
        }
        if ([string]$result.state -ceq "blocked" -and
            -not [string]::IsNullOrWhiteSpace([string]$result.classification) -and
            [string]$result.classification -cne "product-decision") {
            throw "$Label blocked result '$($result.name)' has an invalid non-failure classification."
        }
        if ([string]$result.state -in @("passed", "failed") -and -not [bool]$result.assertions_started) {
            throw "$Label executed result '$($result.name)' must prove assertions started."
        }
        if ([string]$result.state -in @("passed", "failed")) {
            try {
                $started = ConvertTo-Sprint8ADateTimeOffset -Value $result.started_at -Label "$Label '$($result.name)' start"
                $ended = ConvertTo-Sprint8ADateTimeOffset -Value $result.ended_at -Label "$Label '$($result.name)' end"
            } catch {
                throw "$Label executed result '$($result.name)' has malformed chronology: $($_.Exception.Message)"
            }
            $exitStatus = 0
            $duration = 0L
            if ([string]::IsNullOrWhiteSpace([string]$result.command) -or
                -not [int]::TryParse([string]$result.exit_status, [ref]$exitStatus) -or
                -not [long]::TryParse([string]$result.duration_ms, [ref]$duration) -or
                $duration -ne [long][Math]::Max(0, ($ended - $started).TotalMilliseconds) -or
                $ended -lt $started -or
                @($result.evidence).Count -lt 1) {
                throw "$Label executed result '$($result.name)' lacks its command, chronology, exit status, or raw evidence."
            }
            if (([string]$result.state -ceq "passed" -and $exitStatus -ne 0) -or
                ([string]$result.state -ceq "failed" -and $exitStatus -eq 0)) {
                throw "$Label result '$($result.name)' has an exit status inconsistent with its terminal state."
            }
            if (([string]$result.state -ceq "passed" -and
                    -not [string]::IsNullOrWhiteSpace([string]$result.classification)) -or
                ([string]$result.state -ceq "failed" -and
                    ([string]$result.classification -notin @("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization") -or
                        [string]::IsNullOrWhiteSpace([string]$result.failure_message)))) {
                throw "$Label result '$($result.name)' has inconsistent terminal failure classification."
            }
        }
    }
    $Results
}

function Get-Sprint8ACandidateIdentity {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')]
        [string]$NormalizedDeploymentConfigurationSha256
    )

    Assert-Sprint8ASourceIdentityObject -Source $Source -RequireClean | Out-Null
    $canonicalSource = [pscustomobject][ordered]@{
        commit = [string]$Source.commit
        tree = [string]$Source.tree
        dirty = [bool]$Source.dirty
        branch = [string]$Source.branch
        acceptance_inventory_sha256 = [string]$Source.acceptance_inventory_sha256
        deployment_inputs_sha256 = [string]$Source.deployment_inputs_sha256
    }
    $deploymentProfile = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "deploy/sprint-8a/**",
        "crates/tessara-component-module/manifest.json",
        "crates/tessara-dashboard-module/manifest.json",
        "crates/tessara-reference-module-sdk/manifest.json",
        "crates/tessara-reference-scoped-records/manifest.json"
    )
    $migrationBaseline = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*/migrations/*.sql"
    )
    $moduleManifests = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*/manifest.json"
    )
    $moduleAssets = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*/assets/**"
    )
    $contractInputs = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "crates/*contract*/**",
        "crates/tessara-module-sdk/**"
    )
    $fixtureAndSeedInputs = Get-Sprint8APathSetDigest -RepositoryRoot $RepositoryRoot -PathSpecs @(
        "deploy/sprint-7a/uat-fixture-contract.json",
        "deploy/sprint-8a/blueprints/**",
        "deploy/sprint-8a/catalogs/**",
        "crates/**/fixtures/**"
    )
    $moduleReleaseInventory = @(Get-ChildItem -LiteralPath (Join-Path $RepositoryRoot "crates") -Recurse -Filter "manifest.json" -File |
        Sort-Object FullName | ForEach-Object {
            $manifest = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
            [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($RepositoryRoot, $_.FullName).Replace("\", "/")
                definition_id = [string]$manifest.definition_id
                release_version = [string]$manifest.release_version
                sha256 = Get-Sprint8AFileSha256 -Path $_.FullName
            }
        })
    $contract = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.candidate-identity"
        source = $canonicalSource
        acceptance_inventory_sha256 = [string]$canonicalSource.acceptance_inventory_sha256
        deployment_inputs_sha256 = [string]$canonicalSource.deployment_inputs_sha256
        deployment_profile_sha256 = [string]$deploymentProfile.sha256
        normalized_deployment_configuration_sha256 = $NormalizedDeploymentConfigurationSha256
        migration_baseline_sha256 = [string]$migrationBaseline.sha256
        module_manifests_sha256 = [string]$moduleManifests.sha256
        module_assets_sha256 = [string]$moduleAssets.sha256
        contract_inputs_sha256 = [string]$contractInputs.sha256
        fixture_seed_inputs_sha256 = [string]$fixtureAndSeedInputs.sha256
        module_release_inventory = $moduleReleaseInventory
        expected_provenance = [pscustomobject][ordered]@{
            "org.opencontainers.image.revision" = [string]$canonicalSource.commit
            "com.tessara.source-tree" = [string]$canonicalSource.tree
            "com.tessara.source-dirty" = "false"
            "com.tessara.build-profile" = "release"
            acceptance_inventory_sha256 = [string]$canonicalSource.acceptance_inventory_sha256
            deployment_inputs_sha256 = [string]$canonicalSource.deployment_inputs_sha256
            module_manifests_sha256 = [string]$moduleManifests.sha256
            module_assets_sha256 = [string]$moduleAssets.sha256
            schema_migrations_sha256 = [string]$migrationBaseline.sha256
            contract_inputs_sha256 = [string]$contractInputs.sha256
            fixture_seed_inputs_sha256 = [string]$fixtureAndSeedInputs.sha256
        }
    }
    [pscustomobject][ordered]@{
        contract = $contract
        fingerprint = Get-Sprint8AStringSha256 -Text ($contract | ConvertTo-Json -Depth 30 -Compress)
    }
}

function Assert-Sprint8ALifecyclePrerequisite {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$Reference,
        [AllowNull()][string]$ExpectedPhase,
        [AllowNull()][string]$ExpectedCandidateFingerprint,
        [AllowNull()][string]$ExpectedEnvironmentFingerprint
    )

    $resolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Reference.path)

    $sha = Assert-Sprint8AReceiptSidecar -Path ([string]$resolved.full_path)
    if ($sha -cne [string]$Reference.sha256) {
        throw "Lifecycle prerequisite '$ExpectedPhase' SHA-256 differs from its reference."
    }
    $receipt = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
    $expectedSchemaVersion = if ([string]$receipt.phase -in @("validation-readiness", "candidate-rehearsal")) { 2 } else { 1 }
    if (($receipt.schema_version -isnot [int] -and $receipt.schema_version -isnot [long]) -or
        [int]$receipt.schema_version -ne $expectedSchemaVersion -or
        [string]$receipt.sprint -cne "sprint-8a" -or
        (-not [string]::IsNullOrWhiteSpace($ExpectedPhase) -and [string]$receipt.phase -cne $ExpectedPhase) -or
        [string]$receipt.state -cne "passed") {
        throw "Lifecycle prerequisite '$($Reference.path)' is not one passing Sprint 8A receipt."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedCandidateFingerprint) -and
        [string]$receipt.candidate_fingerprint -cne $ExpectedCandidateFingerprint) {
        throw "Lifecycle prerequisite '$ExpectedPhase' carries a different candidate fingerprint."
    }
    if (-not [string]::IsNullOrWhiteSpace($ExpectedEnvironmentFingerprint) -and
        [string]$receipt.environment_fingerprint -cne $ExpectedEnvironmentFingerprint) {
        throw "Lifecycle prerequisite '$ExpectedPhase' carries a different environment fingerprint."
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$resolved.path; sha256 = $sha }
        receipt = $receipt
    }
}

function Assert-Sprint8ALifecyclePrerequisiteSet {
    param(
        [Parameter(Mandatory)][ValidateSet("validation-preflight", "candidate-freeze", "sit", "uat")][string]$Phase,
        [AllowEmptyCollection()][object[]]$References = @(),
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')]
        [string]$NormalizedDeploymentConfigurationSha256,
        [AllowNull()][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $expectedPhases = switch ($Phase) {
        "validation-preflight" { @("validation-readiness", "candidate-rehearsal") }
        "candidate-freeze" { @("validation-preflight") }
        "sit" { @("validation-preflight", "candidate-freeze") }
        "uat" { @("validation-preflight", "candidate-freeze", "sit") }
    }
    if (@($References).Count -ne $expectedPhases.Count) {
        throw "Lifecycle phase '$Phase' requires exactly: $($expectedPhases -join ', ')."
    }
    if ($Phase -ceq "validation-preflight") {
        if (-not [string]::IsNullOrWhiteSpace([string]$CandidateFingerprint)) {
            throw "Validation preflight cannot receive a candidate fingerprint before freeze."
        }
    } else {
        $expectedCandidateIdentity = Get-Sprint8ACandidateIdentity `
            -RepositoryRoot $RepositoryRoot `
            -Source $Source `
            -NormalizedDeploymentConfigurationSha256 $NormalizedDeploymentConfigurationSha256
        if ([string]$expectedCandidateIdentity.fingerprint -cne $CandidateFingerprint) {
            throw "Lifecycle phase '$Phase' carries a non-canonical candidate fingerprint."
        }
    }

    $resolvedInputs = @($References | ForEach-Object {
        Assert-Sprint8ALifecyclePrerequisite `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Reference $_ `
            -ExpectedEnvironmentFingerprint $EnvironmentFingerprint
    })
    $resolvedPrerequisites = [Collections.Generic.List[object]]::new()
    foreach ($expectedPhase in $expectedPhases) {
        $matches = @($resolvedInputs | Where-Object { [string]$_.receipt.phase -ceq $expectedPhase })
        if ($matches.Count -ne 1) {
            throw "Lifecycle phase '$Phase' does not bind exactly one '$expectedPhase' prerequisite."
        }
        $resolvedPrerequisites.Add($matches[0])
    }
    if (@($resolvedPrerequisites | ForEach-Object { [string]$_.reference.path } | Sort-Object -Unique).Count -ne
        $resolvedPrerequisites.Count) {
        throw "Lifecycle phase '$Phase' repeats a prerequisite receipt path."
    }

    foreach ($prerequisite in $resolvedPrerequisites) {
        $receiptSource = $prerequisite.receipt.PSObject.Properties["source_identity"]
        if ($null -eq $receiptSource) {
            $receiptSource = $prerequisite.receipt.PSObject.Properties["mutable_source_identity"]
        }
        if ($null -eq $receiptSource -or
            -not (Test-Sprint8ASourceIdentityMatch -Expected $Source -Actual $receiptSource.Value)) {
            throw "Lifecycle prerequisite '$($prerequisite.receipt.phase)' carries another source identity."
        }
        if ([string]$prerequisite.receipt.phase -in @("candidate-freeze", "sit") -and
            [string]$prerequisite.receipt.candidate_fingerprint -cne $CandidateFingerprint) {
            throw "Lifecycle prerequisite '$($prerequisite.receipt.phase)' carries another candidate fingerprint."
        }
    }

    $byPhase = @{}
    foreach ($prerequisite in $resolvedPrerequisites) {
        $byPhase[[string]$prerequisite.receipt.phase] = $prerequisite
    }
    if ($Phase -ceq "validation-preflight") {
        if ($byPhase["validation-readiness"].receipt.authoritative -isnot [bool] -or
            $byPhase["candidate-rehearsal"].receipt.authoritative -isnot [bool] -or
            $byPhase["validation-readiness"].receipt.authoritative -ne $false -or
            $byPhase["candidate-rehearsal"].receipt.authoritative -ne $false -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["candidate-rehearsal"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["validation-readiness"].reference)) {
            throw "Preflight prerequisites do not form the exact non-authoritative Readiness -> Rehearsal chain."
        }
    }
    if ($Phase -in @("candidate-freeze", "sit", "uat") -and
        ($byPhase["validation-preflight"].receipt.authoritative -isnot [bool] -or
            $byPhase["validation-preflight"].receipt.authoritative -ne $true)) {
        throw "Frozen lifecycle phase '$Phase' requires one authoritative passing preflight receipt."
    }
    if ($Phase -in @("sit", "uat")) {
        if ($byPhase["candidate-freeze"].receipt.authoritative -isnot [bool] -or
            $byPhase["candidate-freeze"].receipt.authoritative -ne $true -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["candidate-freeze"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["validation-preflight"].reference) -or
            @($byPhase["candidate-freeze"].receipt.prerequisite_receipts).Count -ne 1 -or
            ($byPhase["candidate-freeze"].receipt.details.candidate_identity | ConvertTo-Json -Depth 30 -Compress) -cne
                ($expectedCandidateIdentity.contract | ConvertTo-Json -Depth 30 -Compress)) {
            throw "Lifecycle phase '$Phase' requires the exact preflight-bound candidate receipt."
        }
    }
    if ($Phase -ceq "uat") {
        if ($byPhase["sit"].receipt.authoritative -isnot [bool] -or
            $byPhase["sit"].receipt.authoritative -ne $true -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["sit"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["candidate-freeze"].reference) -or
            -not (Test-Sprint8AReceiptReferenceMatch `
                -References @($byPhase["sit"].receipt.prerequisite_receipts) `
                -ExpectedReference $byPhase["validation-preflight"].reference) -or
            @($byPhase["sit"].receipt.prerequisite_receipts).Count -ne 2) {
            throw "Formal UAT requires the exact candidate-bound SIT receipt."
        }
    }

    @($resolvedPrerequisites)
}

function Publish-Sprint8ALifecycleReceipt {
    param(
        [Parameter(Mandatory)][ValidateSet("validation-preflight", "candidate-freeze", "sit", "uat")][string]$Phase,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')]
        [string]$NormalizedDeploymentConfigurationSha256,
        [AllowNull()][string]$CandidateFingerprint,
        [AllowEmptyCollection()][object[]]$PrerequisiteReceipts = @(),
        [AllowEmptyCollection()][object[]]$Checks = @(),
        [AllowNull()]$Details,
        [AllowNull()]$CleanupRestoration,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$OutputPath,
        [switch]$PrepareOnly
    )

    Assert-Sprint8ASourceIdentityObject -Source $Source -RequireClean | Out-Null
    if ($Phase -cne "validation-preflight" -and
        [string]$CandidateFingerprint -notmatch '^[0-9a-f]{64}$') {
        throw "Lifecycle phase '$Phase' requires one lowercase candidate fingerprint."
    }
    $expectedNames = switch ($Phase) {
        "validation-preflight" { Get-Sprint8APreflightCheckNames }
        "candidate-freeze" { @() }
        "sit" { Get-Sprint8ASitLaneNames }
        "uat" { @("scripted-uat") + @(Get-Sprint8AManualUatScenarioNames) }
    }
    Assert-Sprint8AExactTerminalIdentities -Results @($Checks) -ExpectedNames @($expectedNames) -Label $Phase | Out-Null
    $nonpassing = @($Checks | Where-Object state -CNE "passed")
    if ($nonpassing.Count -gt 0) {
        throw "A passing lifecycle receipt cannot be published with failed or blocked '$Phase' results."
    }
    $resolvedPrerequisites = Assert-Sprint8ALifecyclePrerequisiteSet `
        -Phase $Phase `
        -References @($PrerequisiteReceipts) `
        -Source $Source `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -NormalizedDeploymentConfigurationSha256 $NormalizedDeploymentConfigurationSha256 `
        -CandidateFingerprint $CandidateFingerprint `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot
    $canonicalPrerequisites = @($resolvedPrerequisites | ForEach-Object { $_.reference })
    $phaseDetails = $Details
    if ($Phase -ceq "candidate-freeze") {
        $candidateIdentity = Get-Sprint8ACandidateIdentity `
            -RepositoryRoot $RepositoryRoot `
            -Source $Source `
            -NormalizedDeploymentConfigurationSha256 $NormalizedDeploymentConfigurationSha256
        if ([string]$candidateIdentity.fingerprint -cne $CandidateFingerprint) {
            throw "Candidate-freeze fingerprint does not match the canonical candidate identity."
        }
        $phaseDetails = [pscustomobject][ordered]@{
            candidate_identity = $candidateIdentity.contract
            phase_details = $Details
        }
    }
    $canonicalChecks = @($Checks | ForEach-Object {
        $check = $_
        $canonicalCheck = $check | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        $canonicalEvidence = @()
        foreach ($evidence in @($check.evidence)) {
            $evidenceReference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$evidence.path)
            if ((Get-Sprint8AFileSha256 -Path ([string]$evidenceReference.full_path)) -cne [string]$evidence.sha256) {
                throw "Lifecycle check '$($check.name)' has stale evidence '$($evidence.path)'."
            }
            $canonicalEvidence += [pscustomobject][ordered]@{
                path = [string]$evidenceReference.path
                sha256 = [string]$evidence.sha256
            }
        }
        $canonicalCheck.evidence = $canonicalEvidence
        $canonicalCheck
    })
    $canonicalCleanupRestoration = $null
    if ($Phase -in @("sit", "uat")) {
        if ($null -eq $CleanupRestoration -or
            [string]$CleanupRestoration.result -cne "canonical_topology_verified" -or
            @($CleanupRestoration.evidence).Count -lt 1) {
            throw "Lifecycle phase '$Phase' requires retained canonical-topology restoration evidence."
        }
        $canonicalRestorationEvidence = @()
        foreach ($evidence in @($CleanupRestoration.evidence)) {
            $evidenceReference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$evidence.path)
            if ((Get-Sprint8AFileSha256 -Path ([string]$evidenceReference.full_path)) -cne [string]$evidence.sha256) {
                throw "Lifecycle phase '$Phase' has stale restoration evidence '$($evidence.path)'."
            }
            $canonicalRestorationEvidence += [pscustomobject][ordered]@{
                path = [string]$evidenceReference.path
                sha256 = [string]$evidence.sha256
            }
        }
        $canonicalCleanupRestoration = [pscustomobject][ordered]@{
            required = $true
            result = "canonical_topology_verified"
            evidence = $canonicalRestorationEvidence
        }
    } elseif ($null -ne $CleanupRestoration) {
        throw "Lifecycle phase '$Phase' cannot claim SIT/UAT restoration evidence."
    }
    $target = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $OutputPath
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $evidenceRootRelative = [IO.Path]::GetRelativePath($RepositoryRoot, $evidenceRootFullPath).Replace("\", "/").TrimEnd("/")
    $expectedOutputPath = switch ($Phase) {
        "validation-preflight" { "$evidenceRootRelative/preflight-result.json" }
        "candidate-freeze" { "$evidenceRootRelative/candidate.json" }
        "sit" { "$evidenceRootRelative/sit-result.json" }
        "uat" { "$evidenceRootRelative/uat-result.json" }
    }
    if ([string]$target.path -cne $expectedOutputPath) {
        throw "Lifecycle phase '$Phase' must publish to canonical path '$expectedOutputPath'."
    }
    if ($PrepareOnly -and $Phase -cne "uat") {
        throw "Only formal UAT may prepare its canonical result for manifest commitment before publication."
    }
    $now = [DateTimeOffset]::UtcNow
    $startedAt = if (@($canonicalChecks).Count -gt 0) {
        @($canonicalChecks | ForEach-Object { ConvertTo-Sprint8ADateTimeOffset -Value $_.started_at -Label "lifecycle check start" } | Sort-Object | Select-Object -First 1)[0]
    } else { $now }
    $endedAt = if (@($canonicalChecks).Count -gt 0) {
        @($canonicalChecks | ForEach-Object { ConvertTo-Sprint8ADateTimeOffset -Value $_.ended_at -Label "lifecycle check end" } | Sort-Object | Select-Object -Last 1)[0]
    } else { $now }
    $receipt = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = $Phase
        attempt = $Attempt
        authoritative = $true
        state = "passed"
        assertions_started = @($Checks).Count -gt 0
        started_at = $startedAt.ToString("o")
        ended_at = $endedAt.ToString("o")
        duration_ms = [long][Math]::Max(0, ($endedAt - $startedAt).TotalMilliseconds)
        source_identity = $Source
        environment_fingerprint = $EnvironmentFingerprint
        candidate_fingerprint = if ($Phase -ceq "validation-preflight") { $null } else { $CandidateFingerprint }
        prerequisite_receipts = $canonicalPrerequisites
        checks = $canonicalChecks
        assertion_count = @($canonicalChecks | Where-Object assertions_started -EQ $true).Count
        failure_count = 0
        blocked_count = 0
        classification = $null
        correction = $null
        narrow_proof = $null
        invalidation_decision = "none_required"
        details = $phaseDetails
        cleanup_restoration = if ($Phase -in @("sit", "uat")) {
            $canonicalCleanupRestoration
        } else {
            [pscustomobject][ordered]@{ required = $false; result = "not_applicable"; evidence = @() }
        }
    }
    $expectedSha256 = Get-Sprint8AStringSha256 -Text (($receipt | ConvertTo-Json -Depth 30) + "`n")
    if (-not $PrepareOnly) {
        Publish-Sprint7AEvidence -Document $receipt -OutputPath ([string]$target.full_path) | Out-Null
        if ((Assert-Sprint8AReceiptSidecar -Path ([string]$target.full_path)) -cne $expectedSha256) {
            throw "Lifecycle phase '$Phase' publication differs from its prepared canonical bytes."
        }
    }
    [pscustomobject][ordered]@{
        path = [string]$target.path
        sha256 = $expectedSha256
        receipt = $receipt
        prepared_only = [bool]$PrepareOnly
    }
}

function Assert-Sprint8AManualUatReceipt {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][string]$ExpectedScenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint
    )

    $contract = Get-Sprint8AManualUatScenarioContract -Scenario $ExpectedScenario
    try {
        $started = ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.started_at -Label "manual UAT start"
        $ended = ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.ended_at -Label "manual UAT end"
    } catch {
        throw "Manual UAT receipt '$ExpectedScenario' has malformed chronology: $($_.Exception.Message)"
    }
    $duration = 0L
    $expectedSummary = "Every canonical step expectation in $ExpectedScenario is satisfied."
    $expectedLeaseSuffix = "uat/attempt-$ExpectedAttempt/manual-leases/$($ExpectedScenario.ToLowerInvariant())-start.json"
    $expectedResumeSuffix = "uat/attempt-$ExpectedAttempt/manual-leases/$($ExpectedScenario.ToLowerInvariant())-resume.json"
    $resumeContractValid = $Receipt.PSObject.Properties.Name -contains "resumed" -and
        $Receipt.resumed -is [bool] -and
        $Receipt.PSObject.Properties.Name -contains "execution_resume"
    if ($resumeContractValid) {
        if ([bool]$Receipt.resumed) {
            $resumeContractValid = $null -ne $Receipt.execution_resume -and
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.execution_resume.path) -and
                [string]$Receipt.execution_resume.path -notmatch '(^|/)\.\.(/|$)|\\' -and
                [string]$Receipt.execution_resume.path.EndsWith($expectedResumeSuffix, [StringComparison]::Ordinal) -and
                [string]$Receipt.execution_resume.sha256 -match '^[0-9a-f]{64}$'
        } else {
            $resumeContractValid = $null -eq $Receipt.execution_resume
        }
    }
    $actionContractValid = $true
    $actionStates = @()
    try {
        if (@($Receipt.actions).Count -ne [int]$contract.step_count) {
            $actionContractValid = $false
        } else {
            for ($index = 0; $index -lt $contract.steps.Count; $index++) {
                $actualAction = $Receipt.actions[$index]
                $expectedAction = $contract.steps[$index]
                if (($actualAction.step -isnot [int] -and $actualAction.step -isnot [long]) -or
                    [int]$actualAction.step -ne [int]$expectedAction.step -or
                    [string]$actualAction.action -cne [string]$expectedAction.action -or
                    [string]$actualAction.expected_result -cne [string]$expectedAction.expected_result -or
                    [string]::IsNullOrWhiteSpace([string]$actualAction.actual_result) -or
                    [string]$actualAction.state -notin @("passed", "failed", "blocked")) {
                    $actionContractValid = $false
                    break
                }
                $actionStates += [string]$actualAction.state
            }
        }
    } catch {
        $actionContractValid = $false
    }
    $evidenceContractValid = $true
    try {
        $evidenceSteps = @($Receipt.evidence | ForEach-Object {
            if (($_.step -isnot [int] -and $_.step -isnot [long]) -or
                [int]$_.step -lt 1 -or [int]$_.step -gt [int]$contract.step_count -or
                [string]::IsNullOrWhiteSpace([string]$_.path) -or
                [string]$_.sha256 -notmatch '^[0-9a-f]{64}$') {
                $evidenceContractValid = $false
            }
            [int]$_.step
        } | Sort-Object -Unique)
        if ([string]$Receipt.state -ceq "passed" -and
            ($evidenceSteps -join ",") -cne ((1..$contract.step_count) -join ",")) {
            $evidenceContractValid = $false
        }
        foreach ($cleanupEvidence in @($Receipt.cleanup_restoration.evidence)) {
            if ([string]$cleanupEvidence.kind -cne "canonical-restoration" -or
                [string]::IsNullOrWhiteSpace([string]$cleanupEvidence.path) -or
                [string]$cleanupEvidence.sha256 -notmatch '^[0-9a-f]{64}$') {
                $evidenceContractValid = $false
            }
        }
    } catch {
        $evidenceContractValid = $false
    }
    if (($Receipt.schema_version -isnot [int] -and $Receipt.schema_version -isnot [long]) -or
        [int]$Receipt.schema_version -ne 1 -or
        (Get-Sprint8AManualUatScenarioNames) -cnotcontains $ExpectedScenario -or
        [string]$Receipt.sprint -cne "sprint-8a" -or
        [string]$Receipt.phase -cne "uat-manual-scenario" -or
        ($Receipt.attempt -isnot [int] -and $Receipt.attempt -isnot [long]) -or
        [int]$Receipt.attempt -ne $ExpectedAttempt -or
        $Receipt.authoritative -isnot [bool] -or
        $Receipt.diagnostic -isnot [bool] -or
        [bool]$Receipt.authoritative -eq [bool]$Receipt.diagnostic -or
        [string]$Receipt.scenario -cne $ExpectedScenario -or
        [string]$Receipt.state -notin @("passed", "failed", "blocked") -or
        [string]$Receipt.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$Receipt.environment_fingerprint -cne $EnvironmentFingerprint -or
        $Receipt.assertions_started -isnot [bool] -or
        [string]$Receipt.role -cne [string]$contract.role -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.starting_state) -or
        -not $actionContractValid -or
        -not $evidenceContractValid -or
        [string]$Receipt.expected_result -cne $expectedSummary -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.actual_result) -or
        @($Receipt.evidence).Count -lt 1 -or
        [string]$Receipt.scenario_contract.path -cne [string]$contract.document.path -or
        [string]$Receipt.scenario_contract.sha256 -cne [string]$contract.document.sha256 -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.start_checkpoint.path) -or
        [string]$Receipt.start_checkpoint.sha256 -notmatch '^[0-9a-f]{64}$' -or
        [string]::IsNullOrWhiteSpace([string]$Receipt.execution_lease.path) -or
        [string]$Receipt.execution_lease.path -match '(^|/)\.\.(/|$)|\\' -or
        -not [string]$Receipt.execution_lease.path.EndsWith($expectedLeaseSuffix, [StringComparison]::Ordinal) -or
        [string]$Receipt.execution_lease.sha256 -notmatch '^[0-9a-f]{64}$' -or
        -not $resumeContractValid -or
        -not [long]::TryParse([string]$Receipt.duration_ms, [ref]$duration) -or
        $duration -ne [long][Math]::Max(0, ($ended - $started).TotalMilliseconds) -or
        $ended -lt $started -or
        $Receipt.cleanup_restoration.required -isnot [bool] -or
        $Receipt.cleanup_restoration.required -ne $true -or
        [string]$Receipt.cleanup_restoration.result -cne "canonical_topology_verified" -or
        @($Receipt.cleanup_restoration.evidence).Count -lt 1) {
        throw "Manual UAT receipt '$ExpectedScenario' is malformed, incomplete, or bound to another candidate/environment."
    }
    if (([string]$Receipt.state -ceq "passed" -and
            ((@($actionStates | Where-Object { $_ -cne "passed" }).Count -ne 0) -or
                ([bool]$Receipt.authoritative -and [bool]$Receipt.resumed) -or
                -not [bool]$Receipt.assertions_started -or
                [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.classification) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.classification_source) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.failure_message) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.blocked_reason))) -or
        ([string]$Receipt.state -ceq "failed" -and
            ((@($actionStates | Where-Object { $_ -ceq "failed" }).Count -lt 1 -and
                    -not (-not [bool]$Receipt.assertions_started -and
                        [string]$Receipt.classification -ceq "preflight/setup" -and
                        @($actionStates | Where-Object { $_ -ceq "blocked" }).Count -gt 0)) -or
                [string]$Receipt.classification -notin @("preflight/setup", "product", "harness", "environment", "flaky", "evidence-finalization") -or
                [string]$Receipt.classification_source -cne "manual_operator" -or
                [string]::IsNullOrWhiteSpace([string]$Receipt.failure_message) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.blocked_reason) -or
                ([bool]$Receipt.assertions_started -and [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at)) -or
                (-not [bool]$Receipt.assertions_started -and -not [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at)))) -or
        ([string]$Receipt.state -ceq "blocked" -and
            ((@($actionStates | Where-Object { $_ -ceq "failed" }).Count -ne 0) -or
                (@($actionStates | Where-Object { $_ -ceq "blocked" }).Count -lt 1) -or
                [bool]$Receipt.assertions_started -or
                ([bool]$Receipt.authoritative -and -not [bool]$Receipt.resumed) -or
                [string]::IsNullOrWhiteSpace([string]$Receipt.blocked_reason) -or
                (([string]::IsNullOrWhiteSpace([string]$Receipt.classification) -and
                        -not [string]::IsNullOrWhiteSpace([string]$Receipt.classification_source)) -or
                    (-not [string]::IsNullOrWhiteSpace([string]$Receipt.classification) -and
                        ([string]$Receipt.classification -cne "product-decision" -or
                            [string]$Receipt.classification_source -cne "manual_operator"))) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.failure_message) -or
                -not [string]::IsNullOrWhiteSpace([string]$Receipt.assertions_started_at)))) {
        throw "Manual UAT receipt '$ExpectedScenario' has inconsistent authority, assertion, or failure disposition."
    }
    $Receipt
}

function Assert-Sprint8AManualUatExecutionLeasePair {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)]$ReceiptReference,
        [Parameter(Mandatory)][string]$ExpectedScenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $start = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path ([string]$Receipt.execution_lease.path)
    $startSha = Assert-Sprint8AReceiptSidecar -Path ([string]$start.full_path)
    if ($startSha -cne [string]$Receipt.execution_lease.sha256) {
        throw "Manual UAT scenario '$ExpectedScenario' execution-lease digest differs from its receipt."
    }
    $lease = Get-Content -LiteralPath ([string]$start.full_path) -Raw | ConvertFrom-Json
    $leaseStartedAt = ConvertTo-Sprint8ADateTimeOffset -Value $lease.started_at -Label "manual UAT execution-lease start"
    $receiptStartedAt = ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.started_at -Label "manual UAT receipt start"
    if (($lease.schema_version -isnot [int] -and $lease.schema_version -isnot [long]) -or
        [int]$lease.schema_version -ne 1 -or [string]$lease.sprint -cne "sprint-8a" -or
        [string]$lease.phase -cne "uat-manual-execution-lease" -or [string]$lease.state -cne "executing" -or
        [int]$lease.attempt -ne $ExpectedAttempt -or [string]$lease.scenario -cne $ExpectedScenario -or
        [string]$lease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$lease.environment_fingerprint -cne $EnvironmentFingerprint -or
        $leaseStartedAt -ne $receiptStartedAt -or
        $lease.authoritative -isnot [bool] -or [bool]$lease.authoritative -ne [bool]$Receipt.authoritative -or
        $lease.diagnostic -isnot [bool] -or [bool]$lease.diagnostic -ne [bool]$Receipt.diagnostic -or
        ($lease.process_id -isnot [int] -and $lease.process_id -isnot [long]) -or
        [int]$lease.process_id -lt 1 -or
        [string]$lease.checkpoint.path -cne [string]$Receipt.start_checkpoint.path -or
        [string]$lease.checkpoint.sha256 -cne [string]$Receipt.start_checkpoint.sha256) {
        throw "Manual UAT scenario '$ExpectedScenario' execution lease is malformed or bound to another execution."
    }
    if ([bool]$Receipt.resumed) {
        Assert-Sprint8AManualUatResumeMarker `
            -MarkerReference $Receipt.execution_resume `
            -LeaseReference ([pscustomobject]@{ path = [string]$start.path; sha256 = $startSha }) `
            -LeaseDocument $lease `
            -Scenario $ExpectedScenario `
            -Attempt $ExpectedAttempt `
            -CandidateFingerprint $CandidateFingerprint `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -StartedAt $receiptStartedAt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot | Out-Null
    } elseif ($null -ne $Receipt.execution_resume) {
        throw "Manual UAT scenario '$ExpectedScenario' has an unexpected execution-resume binding."
    }

    $completionPath = [string]$start.path
    if (-not $completionPath.EndsWith("-start.json", [StringComparison]::Ordinal)) {
        throw "Manual UAT scenario '$ExpectedScenario' execution lease path is non-canonical."
    }
    $completionPath = $completionPath.Substring(0, $completionPath.Length - "-start.json".Length) + "-complete.json"
    $completion = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $completionPath
    $completionSha = Assert-Sprint8AReceiptSidecar -Path ([string]$completion.full_path)
    $document = Get-Content -LiteralPath ([string]$completion.full_path) -Raw | ConvertFrom-Json
    try {
        $completionStarted = ConvertTo-Sprint8ADateTimeOffset -Value $document.started_at -Label "manual UAT lease start"
        $completionEnded = ConvertTo-Sprint8ADateTimeOffset -Value $document.ended_at -Label "manual UAT lease completion"
    } catch {
        throw "Manual UAT scenario '$ExpectedScenario' execution-lease completion has malformed chronology: $($_.Exception.Message)"
    }
    if (($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 1 -or [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "uat-manual-execution-lease" -or [string]$document.state -cne "completed" -or
        $document.authoritative -isnot [bool] -or [bool]$document.authoritative -or
        [int]$document.attempt -ne $ExpectedAttempt -or [string]$document.scenario -cne $ExpectedScenario -or
        [string]$document.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$document.environment_fingerprint -cne $EnvironmentFingerprint -or
        $completionStarted -ne $receiptStartedAt -or
        $completionEnded -ne (ConvertTo-Sprint8ADateTimeOffset -Value $Receipt.ended_at -Label "manual UAT receipt end") -or
        $completionEnded -lt $completionStarted -or
        [string]$document.lease.path -cne [string]$start.path -or
        [string]$document.lease.sha256 -cne $startSha -or
        $document.resumed -isnot [bool] -or [bool]$document.resumed -ne [bool]$Receipt.resumed -or
        (ConvertTo-Json -InputObject $document.resume -Depth 10 -Compress) -cne
            (ConvertTo-Json -InputObject $Receipt.execution_resume -Depth 10 -Compress) -or
        [string]$document.receipt.path -cne [string]$ReceiptReference.path -or
        [string]$document.receipt.sha256 -cne [string]$ReceiptReference.sha256) {
        throw "Manual UAT scenario '$ExpectedScenario' execution-lease completion is malformed or not bound to its exact receipt."
    }
    [pscustomobject][ordered]@{
        start = [pscustomobject][ordered]@{ path = [string]$start.path; sha256 = $startSha }
        completion = [pscustomobject][ordered]@{ path = [string]$completion.path; sha256 = $completionSha }
    }
}

function Assert-Sprint8AManualUatStartCheckpoint {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$ScenarioStartedAt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $checkpointPath = Join-Path $evidenceRootFullPath "attempts/uat-$Attempt-manual-checkpoint.json"
    $checkpointRepositoryPath = [IO.Path]::GetRelativePath($RepositoryRoot, $checkpointPath).Replace("\", "/")
    $reference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path $checkpointRepositoryPath
    $sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$reference.full_path)
    $checkpoint = Get-Content -LiteralPath ([string]$reference.full_path) -Raw | ConvertFrom-Json
    try {
        $scriptedCompletedAt = ConvertTo-Sprint8ADateTimeOffset `
            -Value $checkpoint.scripted_completed_at `
            -Label "manual UAT scripted completion"
    } catch {
        throw "Manual UAT scenario '$Scenario' Start checkpoint has malformed chronology: $($_.Exception.Message)"
    }
    if (($checkpoint.schema_version -isnot [int] -and $checkpoint.schema_version -isnot [long]) -or
        [int]$checkpoint.schema_version -ne 1 -or
        [string]$checkpoint.sprint -cne "sprint-8a" -or
        [string]$checkpoint.phase -cne "uat" -or
        ($checkpoint.attempt -isnot [int] -and $checkpoint.attempt -isnot [long]) -or
        [int]$checkpoint.attempt -ne $Attempt -or
        $checkpoint.authoritative -isnot [bool] -or [bool]$checkpoint.authoritative -or
        [string]$checkpoint.state -cne "executing" -or
        [string]$checkpoint.stage -notin @("manual", "diagnostic-manual") -or
        [string]$checkpoint.source_verification_state -cne "verified" -or
        [string]$checkpoint.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$checkpoint.environment_fingerprint -cne $EnvironmentFingerprint -or
        @($checkpoint.prerequisite_receipts).Count -ne 3 -or
        @($checkpoint.manual_scenarios_pending | Where-Object { [string]$_ -ceq $Scenario }).Count -ne 1 -or
        $ScenarioStartedAt -lt $scriptedCompletedAt) {
        throw "Manual UAT scenario '$Scenario' is not bound to its exact authenticated Start checkpoint."
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$reference.path; sha256 = $sha256 }
        receipt = $checkpoint
    }
}

function Assert-Sprint8AManualUatAttemptOpen {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)]$Checkpoint,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $attemptPath = Join-Path $evidenceRootFullPath "attempts/uat-$Attempt.json"
    $attemptRepositoryPath = [IO.Path]::GetRelativePath($RepositoryRoot, $attemptPath).Replace("\", "/")
    $attemptReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path $attemptRepositoryPath
    $attemptSha = Assert-Sprint8AReceiptSidecar -Path ([string]$attemptReference.full_path)
    $attemptReceipt = Get-Content -LiteralPath ([string]$attemptReference.full_path) -Raw | ConvertFrom-Json
    if (($attemptReceipt | ConvertTo-Json -Depth 50 -Compress) -cne
        ($Checkpoint.receipt | ConvertTo-Json -Depth 50 -Compress)) {
        throw "Manual UAT publication requires the mutable attempt to remain at its exact awaiting-manual checkpoint."
    }
    $canonicalResultPath = Join-Path $evidenceRootFullPath "uat-result.json"
    if ((Test-Path -LiteralPath $canonicalResultPath -PathType Leaf) -or
        (Test-Path -LiteralPath "$canonicalResultPath.sha256" -PathType Leaf)) {
        throw "Manual UAT publication is forbidden after canonical UAT result publication begins."
    }
    $manifestPath = Join-Path $evidenceRootFullPath "evidence-manifest.json"
    Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or
        [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
        throw "Manual UAT publication requires a valid lifecycle evidence manifest."
    }
    $checkpointResolved = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$Checkpoint.reference.path)
    $requiredManifestReferences = @(
        [pscustomobject][ordered]@{ path = [string]$Checkpoint.reference.path; sha256 = [string]$Checkpoint.reference.sha256 },
        [pscustomobject][ordered]@{ path = [string]$attemptReference.path; sha256 = $attemptSha },
        [pscustomobject][ordered]@{
            path = "$([string]$Checkpoint.reference.path).sha256"
            sha256 = Get-Sprint8AFileSha256 -Path "$([string]$checkpointResolved.full_path).sha256"
        },
        [pscustomobject][ordered]@{
            path = "$([string]$attemptReference.path).sha256"
            sha256 = Get-Sprint8AFileSha256 -Path "$([string]$attemptReference.full_path).sha256"
        }
    )
    foreach ($required in $requiredManifestReferences) {
        $matches = @($manifest.entries | Where-Object {
            [string]$_.path -ceq [string]$required.path -and
                [string]$_.sha256 -ceq [string]$required.sha256
        })
        if ($matches.Count -ne 1) {
            throw "Manual UAT publication cannot authenticate open-attempt manifest entry '$($required.path)'."
        }
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$attemptReference.path; sha256 = $attemptSha }
        receipt = $attemptReceipt
    }
}

function Assert-Sprint8AManualUatResumeMarker {
    param(
        [Parameter(Mandatory)]$MarkerReference,
        [Parameter(Mandatory)]$LeaseReference,
        [Parameter(Mandatory)]$LeaseDocument,
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [ValidateRange(0, [int]::MaxValue)][int]$ExpectedCurrentProcessId = 0
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $marker = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$MarkerReference.path)
    $expectedPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-resume.json")
    ).Replace("\", "/")
    $markerSha = Assert-Sprint8AReceiptSidecar -Path ([string]$marker.full_path)
    $document = Get-Content -LiteralPath ([string]$marker.full_path) -Raw | ConvertFrom-Json
    $resumedAt = ConvertTo-Sprint8ADateTimeOffset `
        -Value $document.resumed_at `
        -Label "manual UAT execution-resume time"
    if ([string]$marker.path -cne $expectedPath -or
        $markerSha -cne [string]$MarkerReference.sha256 -or
        ($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 1 -or [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "uat-manual-execution-resume" -or
        [string]$document.state -cne "resumed" -or $document.authoritative -isnot [bool] -or
        [bool]$document.authoritative -or [int]$document.attempt -ne $Attempt -or
        [string]$document.scenario -cne $Scenario -or
        [string]$document.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$document.environment_fingerprint -cne $EnvironmentFingerprint -or
        ($document.original_process_id -isnot [int] -and $document.original_process_id -isnot [long]) -or
        [int]$document.original_process_id -lt 1 -or
        ($document.current_process_id -isnot [int] -and $document.current_process_id -isnot [long]) -or
        [int]$document.current_process_id -lt 1 -or
        ($LeaseDocument.process_id -isnot [int] -and $LeaseDocument.process_id -isnot [long]) -or
        [int]$document.original_process_id -ne [int]$LeaseDocument.process_id -or
        ($ExpectedCurrentProcessId -gt 0 -and [int]$document.current_process_id -ne $ExpectedCurrentProcessId) -or
        $resumedAt -lt $StartedAt -or
        [string]$document.lease.path -cne [string]$LeaseReference.path -or
        [string]$document.lease.sha256 -cne [string]$LeaseReference.sha256 -or
        [string]$document.checkpoint.path -cne [string]$LeaseDocument.checkpoint.path -or
        [string]$document.checkpoint.sha256 -cne [string]$LeaseDocument.checkpoint.sha256) {
        throw "Manual UAT scenario '$Scenario' execution-resume marker is malformed or bound to another process lineage."
    }
    [pscustomobject][ordered]@{
        reference = [pscustomobject][ordered]@{ path = [string]$marker.path; sha256 = $markerSha }
        document = $document
    }
}

function Open-Sprint8AManualUatScenarioLease {
    param(
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [bool]$Authoritative = $true,
        [bool]$Diagnostic = $false,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$Resume
    )

    if ((Get-Sprint8AManualUatScenarioNames) -cnotcontains $Scenario) {
        throw "Unknown Sprint 8A manual UAT scenario '$Scenario'."
    }
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $stream = Open-Sprint8AValidationAttemptLock -Path (Join-Path $evidenceRootFullPath "validation-attempt.lock")
    try {
        $checkpoint = Assert-Sprint8AManualUatStartCheckpoint `
            -Attempt $Attempt `
            -Scenario $Scenario `
            -CandidateFingerprint $CandidateFingerprint `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -ScenarioStartedAt $StartedAt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath
        Assert-Sprint8AManualUatAttemptOpen `
            -Attempt $Attempt `
            -Checkpoint $checkpoint `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath | Out-Null
        if ([string]$checkpoint.receipt.stage -ceq "diagnostic-manual" -and ($Authoritative -or -not $Diagnostic)) {
            throw "Manual UAT scenario '$Scenario' execution must be diagnostic after scripted invalidation."
        }
        $manualRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual"
        foreach ($priorPath in @(Get-ChildItem -LiteralPath $manualRoot -Filter "uat-8a-*.json" -File -ErrorAction SilentlyContinue)) {
            Assert-Sprint8AReceiptSidecar -Path $priorPath.FullName | Out-Null
            $prior = Get-Content -LiteralPath $priorPath.FullName -Raw | ConvertFrom-Json
            if ((ConvertTo-Sprint8ADateTimeOffset -Value $prior.ended_at -Label "prior manual UAT end") -ge $StartedAt) { continue }
            if ([string]$prior.state -ceq "blocked" -and [string]$prior.classification -ceq "product-decision") {
                throw "Manual UAT scenario '$Scenario' execution is paused by unresolved product decision '$($prior.scenario)'."
            }
            if ([string]$prior.state -ceq "failed" -and [bool]$prior.authoritative -and
                ($Authoritative -or -not $Diagnostic)) {
                throw "Manual UAT scenario '$Scenario' execution must be diagnostic after '$($prior.scenario)' invalidated the candidate."
            }
        }
        $leaseRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases"
        [IO.Directory]::CreateDirectory($leaseRoot) | Out-Null
        $leasePath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-start.json"
        $resumePath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-resume.json"
        $completionPath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-complete.json"
        $preparedPublicationPath = Join-Path $leaseRoot "$($Scenario.ToLowerInvariant())-publication-prepared.json"
        Repair-Sprint7AEvidencePublication -Path $preparedPublicationPath
        if ((Test-Path -LiteralPath $preparedPublicationPath -PathType Leaf) -or
            (Test-Path -LiteralPath "$preparedPublicationPath.sha256" -PathType Leaf)) {
            throw "Manual UAT scenario '$Scenario' already has a prepared publication; repair or finalize it without rerunning the scenario."
        }
        if (Test-Path -LiteralPath $completionPath -PathType Leaf) {
            throw "Manual UAT scenario '$Scenario' already has a completed execution lease."
        }
        $resumeReference = $null
        $resumed = $false
        $originalProcessId = 0
        $currentProcessId = $PID
        if (Test-Path -LiteralPath $leasePath -PathType Leaf) {
            if (-not $Resume) {
                throw "Manual UAT scenario '$Scenario' has an interrupted execution lease; resume it explicitly."
            }
            $leaseSha = Assert-Sprint8AReceiptSidecar -Path $leasePath
            $leaseDocument = Get-Content -LiteralPath $leasePath -Raw | ConvertFrom-Json
            $leaseStartedAt = ConvertTo-Sprint8ADateTimeOffset `
                -Value $leaseDocument.started_at `
                -Label "interrupted manual UAT execution-lease start"
            if (($leaseDocument.schema_version -isnot [int] -and $leaseDocument.schema_version -isnot [long]) -or
                [int]$leaseDocument.schema_version -ne 1 -or [string]$leaseDocument.sprint -cne "sprint-8a" -or
                [string]$leaseDocument.phase -cne "uat-manual-execution-lease" -or
                [string]$leaseDocument.state -cne "executing" -or
                [string]$leaseDocument.scenario -cne $Scenario -or [int]$leaseDocument.attempt -ne $Attempt -or
                [string]$leaseDocument.candidate_fingerprint -cne $CandidateFingerprint -or
                [string]$leaseDocument.environment_fingerprint -cne $EnvironmentFingerprint -or
                $leaseStartedAt -ne $StartedAt -or
                $leaseDocument.authoritative -isnot [bool] -or [bool]$leaseDocument.authoritative -ne $Authoritative -or
                $leaseDocument.diagnostic -isnot [bool] -or [bool]$leaseDocument.diagnostic -ne $Diagnostic -or
                ($leaseDocument.process_id -isnot [int] -and $leaseDocument.process_id -isnot [long]) -or
                [int]$leaseDocument.process_id -lt 1 -or
                [string]$leaseDocument.checkpoint.path -cne [string]$checkpoint.reference.path -or
                [string]$leaseDocument.checkpoint.sha256 -cne [string]$checkpoint.reference.sha256) {
                throw "Manual UAT scenario '$Scenario' interrupted lease is bound to another execution."
            }
            $originalProcessId = [int]$leaseDocument.process_id
            Repair-Sprint7AEvidencePublication -Path $resumePath
            if (Test-Path -LiteralPath $resumePath -PathType Leaf) {
                $resumeReference = [pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($RepositoryRoot, $resumePath).Replace("\", "/")
                    sha256 = Assert-Sprint8AReceiptSidecar -Path $resumePath
                }
                $resumeMarker = Assert-Sprint8AManualUatResumeMarker `
                    -MarkerReference $resumeReference `
                    -LeaseReference ([pscustomobject]@{
                        path = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
                        sha256 = $leaseSha
                    }) `
                    -LeaseDocument $leaseDocument `
                    -Scenario $Scenario `
                    -Attempt $Attempt `
                    -CandidateFingerprint $CandidateFingerprint `
                    -EnvironmentFingerprint $EnvironmentFingerprint `
                    -StartedAt $StartedAt `
                    -RepositoryRoot $RepositoryRoot `
                    -EvidenceRoot $evidenceRootFullPath `
                    -ExpectedCurrentProcessId $PID
                $resumeReference = $resumeMarker.reference
            } else {
                $resumeDocument = [pscustomobject][ordered]@{
                    schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-execution-resume"
                    authoritative = $false; state = "resumed"; attempt = $Attempt; scenario = $Scenario
                    candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
                    original_process_id = $originalProcessId; current_process_id = $PID
                    resumed_at = [DateTimeOffset]::UtcNow.ToString("o")
                    lease = [pscustomobject][ordered]@{
                        path = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
                        sha256 = $leaseSha
                    }
                    checkpoint = $checkpoint.reference
                }
                Publish-Sprint7AEvidence -Document $resumeDocument -OutputPath $resumePath | Out-Null
                $resumeReference = [pscustomobject][ordered]@{
                    path = [IO.Path]::GetRelativePath($RepositoryRoot, $resumePath).Replace("\", "/")
                    sha256 = Assert-Sprint8AReceiptSidecar -Path $resumePath
                }
            }
            $resumed = $true
            $manifestFullPath = Join-Path $evidenceRootFullPath "evidence-manifest.json"
            Assert-Sprint8AReceiptSidecar -Path $manifestFullPath | Out-Null
            $existingManifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
            $entries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -Overrides @([pscustomobject]@{
                    path = [string]$resumeReference.path; sha256 = [string]$resumeReference.sha256
                    phase = "uat-manual-resume"; authoritative = $false; status = "resumed"
                })
            $authorizedReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
                -ExistingEntries @($existingManifest.entries) `
                -UpdatedEntries $entries)
            Publish-Sprint8AEvidenceManifest `
                -Entries $entries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -OutputPath ([IO.Path]::GetRelativePath($RepositoryRoot, $manifestFullPath).Replace("\", "/")) `
                -Merge `
                -AuthorizedReplacementPaths $authorizedReplacementPaths | Out-Null
        } else {
            if ($Resume) { throw "Manual UAT scenario '$Scenario' has no interrupted lease to resume." }
            $leaseDocument = [pscustomobject][ordered]@{
                schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-execution-lease"
                authoritative = $Authoritative; diagnostic = $Diagnostic; state = "executing"
                attempt = $Attempt; scenario = $Scenario
                candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
                started_at = $StartedAt.ToString("o"); process_id = $PID; checkpoint = $checkpoint.reference
            }
            Publish-Sprint7AEvidence -Document $leaseDocument -OutputPath $leasePath | Out-Null
            $leaseSha = Assert-Sprint8AReceiptSidecar -Path $leasePath
            $originalProcessId = $PID
            $leaseRelative = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
            $entries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -Overrides @([pscustomobject]@{
                    path = $leaseRelative; sha256 = $leaseSha; phase = "uat-manual-lease"
                    authoritative = $false; status = "executing"
                })
            Publish-Sprint8AEvidenceManifest `
                -Entries $entries `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $evidenceRootFullPath `
                -OutputPath ([IO.Path]::GetRelativePath($RepositoryRoot, (Join-Path $evidenceRootFullPath "evidence-manifest.json")).Replace("\", "/")) `
                -Merge | Out-Null
        }
        [pscustomobject][ordered]@{
            schema_version = 1; stream = $stream; attempt = $Attempt; scenario = $Scenario
            candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
            started_at = $StartedAt.ToString("o"); authoritative = $Authoritative; diagnostic = $Diagnostic
            resumed = $resumed; original_process_id = $originalProcessId; current_process_id = $currentProcessId
            checkpoint = $checkpoint.reference
            lease = [pscustomobject]@{
                path = [IO.Path]::GetRelativePath($RepositoryRoot, $leasePath).Replace("\", "/")
                sha256 = $leaseSha
            }
            resume = $resumeReference
            completion_path = [IO.Path]::GetRelativePath($RepositoryRoot, $completionPath).Replace("\", "/")
        }
    } catch {
        $stream.Dispose()
        throw
    }
}

function Assert-Sprint8ANoOpenManualUatLeases {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $leaseRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases"
    if (-not (Test-Path -LiteralPath $leaseRoot -PathType Container)) { return }
    $open = @()
    foreach ($start in @(Get-ChildItem -LiteralPath $leaseRoot -Filter "*-start.json" -File)) {
        Assert-Sprint8AReceiptSidecar -Path $start.FullName | Out-Null
        $completion = $start.FullName.Substring(0, $start.FullName.Length - "-start.json".Length) + "-complete.json"
        if (-not (Test-Path -LiteralPath $completion -PathType Leaf)) {
            $open += $start.Name
        } else {
            Assert-Sprint8AReceiptSidecar -Path $completion | Out-Null
        }
    }
    if ($open.Count -gt 0) {
        throw "Formal UAT finalization is blocked by open or interrupted manual execution lease(s): $($open -join ', ')."
    }
}

function Assert-Sprint8AManualUatPreparedPublicationCheckpoint {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)]$CheckpointReference,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$ExpectedAttempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    $scenario = [string]$Document.scenario
    if ((Get-Sprint8AManualUatScenarioNames) -cnotcontains $scenario) {
        throw "Manual UAT prepared publication names unknown scenario '$scenario'."
    }
    $checkpoint = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$CheckpointReference.path)
    $scenarioName = $scenario.ToLowerInvariant()
    $expectedCheckpointPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$ExpectedAttempt/manual-leases/$scenarioName-publication-prepared.json")
    ).Replace("\", "/")
    $expectedReceiptPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$ExpectedAttempt/manual/$scenarioName.json")
    ).Replace("\", "/")
    $expectedCompletionPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$ExpectedAttempt/manual-leases/$scenarioName-complete.json")
    ).Replace("\", "/")
    $checkpointSha = Assert-Sprint8AReceiptSidecar -Path ([string]$checkpoint.full_path)
    $preparedAt = ConvertTo-Sprint8ADateTimeOffset `
        -Value $Document.prepared_at `
        -Label "manual UAT prepared-publication time"
    $receipt = $Document.receipt.document
    Assert-Sprint8AManualUatReceipt `
        -Receipt $receipt `
        -ExpectedScenario $scenario `
        -ExpectedAttempt $ExpectedAttempt `
        -CandidateFingerprint ([string]$Document.candidate_fingerprint) `
        -EnvironmentFingerprint ([string]$Document.environment_fingerprint) | Out-Null
    $receiptSha = Get-Sprint8AStringSha256 -Text (($receipt | ConvertTo-Json -Depth 30) + "`n")
    $completionSha = Get-Sprint8AStringSha256 -Text (($Document.completion.document | ConvertTo-Json -Depth 30) + "`n")
    $receiptEndedAt = ConvertTo-Sprint8ADateTimeOffset -Value $receipt.ended_at -Label "manual UAT receipt end"
    $leaseBinding = ConvertTo-Json -InputObject $receipt.execution_lease -Depth 10 -Compress
    $resumeBinding = ConvertTo-Json -InputObject $receipt.execution_resume -Depth 10 -Compress
    if (($Document.schema_version -isnot [int] -and $Document.schema_version -isnot [long]) -or
        [int]$Document.schema_version -ne 1 -or [string]$Document.sprint -cne "sprint-8a" -or
        [string]$Document.phase -cne "uat-manual-publication-checkpoint" -or
        [string]$Document.state -cne "prepared" -or $Document.authoritative -isnot [bool] -or
        [bool]$Document.authoritative -or [int]$Document.attempt -ne $ExpectedAttempt -or
        [string]$Document.candidate_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$Document.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$checkpoint.path -cne $expectedCheckpointPath -or
        $checkpointSha -cne [string]$CheckpointReference.sha256 -or
        $preparedAt -lt $receiptEndedAt -or
        [string]$Document.receipt.path -cne $expectedReceiptPath -or
        [string]$Document.receipt.sha256 -cne $receiptSha -or
        [string]$Document.completion.path -cne $expectedCompletionPath -or
        [string]$Document.completion.sha256 -cne $completionSha -or
        (ConvertTo-Json -InputObject $Document.lease -Depth 10 -Compress) -cne $leaseBinding -or
        (ConvertTo-Json -InputObject $Document.resume -Depth 10 -Compress) -cne $resumeBinding -or
        [string]$Document.completion.document.phase -cne "uat-manual-execution-lease" -or
        [string]$Document.completion.document.state -cne "completed" -or
        [int]$Document.completion.document.attempt -ne $ExpectedAttempt -or
        [string]$Document.completion.document.scenario -cne $scenario -or
        (ConvertTo-Json -InputObject $Document.completion.document.lease -Depth 10 -Compress) -cne $leaseBinding -or
        $Document.completion.document.resumed -isnot [bool] -or
        [bool]$Document.completion.document.resumed -ne [bool]$receipt.resumed -or
        (ConvertTo-Json -InputObject $Document.completion.document.resume -Depth 10 -Compress) -cne $resumeBinding -or
        [string]$Document.completion.document.receipt.path -cne $expectedReceiptPath -or
        [string]$Document.completion.document.receipt.sha256 -cne $receiptSha) {
        throw "Manual UAT prepared publication for '$scenario' is malformed or not bound to its exact receipt, completion, and lease lineage (checkpoint '$([string]$checkpoint.path)'/'$expectedCheckpointPath'; receipt '$([string]$Document.receipt.path)'/'$expectedReceiptPath' $([string]$Document.receipt.sha256)/$receiptSha; completion '$([string]$Document.completion.path)'/'$expectedCompletionPath' $([string]$Document.completion.sha256)/$completionSha)."
    }
    [pscustomobject][ordered]@{
        checkpoint = [pscustomobject][ordered]@{ path = [string]$checkpoint.path; sha256 = $checkpointSha }
        receipt = [pscustomobject][ordered]@{
            path = $expectedReceiptPath; full_path = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expectedReceiptPath))
            sha256 = $receiptSha; document = $receipt
        }
        completion = [pscustomobject][ordered]@{
            path = $expectedCompletionPath; full_path = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expectedCompletionPath))
            sha256 = $completionSha; document = $Document.completion.document
        }
    }
}

function Complete-Sprint8AManualUatPreparedEvidencePair {
    param(
        [Parameter(Mandatory)]$PreparedEvidence,
        [Parameter(Mandatory)][string]$Label
    )

    $path = [string]$PreparedEvidence.full_path
    Repair-Sprint7AEvidencePublication -Path $path
    $artifactExists = Test-Path -LiteralPath $path -PathType Leaf
    $sidecarExists = Test-Path -LiteralPath "$path.sha256" -PathType Leaf
    if ($artifactExists -or $sidecarExists) {
        if (-not $artifactExists -or -not $sidecarExists -or
            (Assert-Sprint8AReceiptSidecar -Path $path) -cne [string]$PreparedEvidence.sha256) {
            throw "Manual UAT $Label prepared publication differs from its immutable checkpoint."
        }
    } else {
        Publish-Sprint7AEvidence -Document $PreparedEvidence.document -OutputPath $path | Out-Null
        if ((Assert-Sprint8AReceiptSidecar -Path $path) -cne [string]$PreparedEvidence.sha256) {
            throw "Manual UAT $Label publication differs from its immutable prepared bytes."
        }
    }
    [pscustomobject][ordered]@{ path = [string]$PreparedEvidence.path; sha256 = [string]$PreparedEvidence.sha256 }
}

function Repair-Sprint8AManualUatPreparedPublication {
    param(
        [Parameter(Mandatory)][string]$CheckpointPath,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidenceRootFullPath
    Repair-Sprint7AEvidencePublication -Path $CheckpointPath
    $checkpointRepositoryPath = if ([IO.Path]::IsPathRooted($CheckpointPath)) {
        [IO.Path]::GetRelativePath($RepositoryRoot, [IO.Path]::GetFullPath($CheckpointPath)).Replace("\", "/")
    } else { $CheckpointPath.Replace("\", "/") }
    $checkpoint = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path $checkpointRepositoryPath
    $checkpointReference = [pscustomobject][ordered]@{
        path = [string]$checkpoint.path
        sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$checkpoint.full_path)
    }
    $document = Get-Content -LiteralPath ([string]$checkpoint.full_path) -Raw | ConvertFrom-Json
    $prepared = Assert-Sprint8AManualUatPreparedPublicationCheckpoint `
        -Document $document `
        -CheckpointReference $checkpointReference `
        -ExpectedAttempt $Attempt `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath
    $receiptReference = Complete-Sprint8AManualUatPreparedEvidencePair `
        -PreparedEvidence $prepared.receipt `
        -Label "scenario receipt"
    $completionReference = Complete-Sprint8AManualUatPreparedEvidencePair `
        -PreparedEvidence $prepared.completion `
        -Label "execution-lease completion"
    Assert-Sprint8AManualUatExecutionLeasePair `
        -Receipt $prepared.receipt.document `
        -ReceiptReference $receiptReference `
        -ExpectedScenario ([string]$document.scenario) `
        -ExpectedAttempt $Attempt `
        -CandidateFingerprint ([string]$document.candidate_fingerprint) `
        -EnvironmentFingerprint ([string]$document.environment_fingerprint) `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    $manifestFullPath = Join-Path $evidenceRootFullPath "evidence-manifest.json"
    Assert-Sprint8AReceiptSidecar -Path $manifestFullPath | Out-Null
    $existingManifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    $overrides = @(
        [pscustomobject][ordered]@{
            path = [string]$prepared.checkpoint.path; sha256 = [string]$prepared.checkpoint.sha256
            phase = "uat-manual-publication"; authoritative = $false; status = "prepared"
        },
        [pscustomobject][ordered]@{
            path = [string]$receiptReference.path; sha256 = [string]$receiptReference.sha256
            phase = "uat-manual"; authoritative = [bool]$prepared.receipt.document.authoritative
            status = if ([bool]$prepared.receipt.document.diagnostic) {
                "diagnostic-$([string]$prepared.receipt.document.state)"
            } else { [string]$prepared.receipt.document.state }
        },
        [pscustomobject][ordered]@{
            path = [string]$completionReference.path; sha256 = [string]$completionReference.sha256
            phase = "uat-manual-lease"; authoritative = $false; status = "completed"
        }
    )
    if ([bool]$prepared.receipt.document.resumed) {
        $overrides += [pscustomobject][ordered]@{
            path = [string]$prepared.receipt.document.execution_resume.path
            sha256 = [string]$prepared.receipt.document.execution_resume.sha256
            phase = "uat-manual-resume"; authoritative = $false; status = "resumed"
        }
    }
    $manifestEntries = Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Overrides $overrides
    $authorizedReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
        -ExistingEntries @($existingManifest.entries) `
        -UpdatedEntries $manifestEntries)
    Publish-Sprint8AEvidenceManifest `
        -Entries $manifestEntries `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -OutputPath ([IO.Path]::GetRelativePath($RepositoryRoot, $manifestFullPath).Replace("\", "/")) `
        -Merge `
        -AuthorizedReplacementPaths $authorizedReplacementPaths | Out-Null
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -ManifestPath $manifestFullPath `
        -AllowMissingCanonicalUatCommitment:$AllowMissingCanonicalUatCommitment | Out-Null
    [pscustomobject][ordered]@{
        scenario = [string]$document.scenario
        checkpoint = $prepared.checkpoint
        receipt = $receiptReference
        completion = $completionReference
    }
}

function Repair-Sprint8AManualUatPreparedPublications {
    param(
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot)) }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidenceRootFullPath
    $leaseRoot = Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases"
    if (-not (Test-Path -LiteralPath $leaseRoot -PathType Container)) { return @() }
    $checkpointPaths = @(Get-ChildItem -LiteralPath $leaseRoot -Force -File | Where-Object {
        $_.Name -cmatch '^uat-8a-[0-9]{2}-publication-prepared\.json(?:\.sha256)?$'
    } | ForEach-Object {
        if ($_.Name.EndsWith(".sha256", [StringComparison]::Ordinal)) {
            $_.FullName.Substring(0, $_.FullName.Length - ".sha256".Length)
        } else { $_.FullName }
    } | Sort-Object -CaseSensitive -Unique)
    @($checkpointPaths | ForEach-Object {
        Repair-Sprint8AManualUatPreparedPublication `
            -CheckpointPath $_ `
            -Attempt $Attempt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath `
            -AllowMissingCanonicalUatCommitment:$AllowMissingCanonicalUatCommitment
    })
}

function Publish-Sprint8AManualUatReceipt {
    param(
        [Parameter(Mandatory)][string]$Scenario,
        [Parameter(Mandatory)][ValidateRange(1, 9999)][int]$Attempt,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$CandidateFingerprint,
        [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$EnvironmentFingerprint,
        [ValidateSet("passed", "failed", "blocked")][string]$State = "passed",
        [bool]$Authoritative = $true,
        [bool]$Diagnostic = $false,
        [bool]$AssertionsStarted = $true,
        [AllowNull()][string]$Classification,
        [AllowNull()][string]$FailureMessage,
        [AllowNull()][string]$BlockedReason,
        [Parameter(Mandatory)][string]$Role,
        [Parameter(Mandatory)][string]$StartingState,
        [Parameter(Mandatory)][object[]]$Actions,
        [Parameter(Mandatory)][string]$ExpectedResult,
        [Parameter(Mandatory)][string]$ActualResult,
        [Parameter(Mandatory)][object[]]$Evidence,
        [Parameter(Mandatory)][object[]]$CleanupEvidence,
        [Parameter(Mandatory)][DateTimeOffset]$StartedAt,
        [Parameter(Mandatory)][DateTimeOffset]$EndedAt,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$OutputPath,
        [Parameter(Mandatory)]$ExecutionLease
    )

    if ($EndedAt -lt $StartedAt) { throw "Manual UAT scenario '$Scenario' has invalid chronology." }
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    if ($ExecutionLease.PSObject.Properties.Name -notcontains "stream" -or
        $ExecutionLease.stream -isnot [IO.FileStream] -or -not $ExecutionLease.stream.CanWrite -or
        $ExecutionLease.PSObject.Properties.Name -notcontains "resumed" -or
        $ExecutionLease.resumed -isnot [bool] -or
        $ExecutionLease.PSObject.Properties.Name -notcontains "resume" -or
        ($ExecutionLease.original_process_id -isnot [int] -and $ExecutionLease.original_process_id -isnot [long]) -or
        [int]$ExecutionLease.original_process_id -lt 1 -or
        ($ExecutionLease.current_process_id -isnot [int] -and $ExecutionLease.current_process_id -isnot [long]) -or
        [int]$ExecutionLease.current_process_id -ne $PID -or
        [int]$ExecutionLease.attempt -ne $Attempt -or [string]$ExecutionLease.scenario -cne $Scenario -or
        [string]$ExecutionLease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$ExecutionLease.environment_fingerprint -cne $EnvironmentFingerprint -or
        [string]$ExecutionLease.started_at -cne $StartedAt.ToString("o") -or
        [bool]$ExecutionLease.authoritative -ne $Authoritative -or [bool]$ExecutionLease.diagnostic -ne $Diagnostic) {
        throw "Manual UAT scenario '$Scenario' publication requires its exact live execution lease."
    }
    if ([bool]$ExecutionLease.resumed -and $Authoritative -and $State -ceq "passed") {
        throw "Manual UAT scenario '$Scenario' cannot publish an authoritative pass after execution resume; retain a failed or blocked result."
    }
    $expectedLockPath = [IO.Path]::GetFullPath((Join-Path $evidenceRootFullPath "validation-attempt.lock"))
    if ([IO.Path]::GetFullPath([string]$ExecutionLease.stream.Name) -cne $expectedLockPath) {
        throw "Manual UAT scenario '$Scenario' execution lease does not hold the canonical validation lock."
    }
    $lockIsExclusive = $false
    try {
        $concurrentLock = Open-Sprint8AValidationAttemptLock -Path $expectedLockPath
        $concurrentLock.Dispose()
    } catch [IO.IOException] {
        $lockIsExclusive = $true
    }
    if (-not $lockIsExclusive) {
        throw "Manual UAT scenario '$Scenario' execution lease does not exclude concurrent finalization."
    }
    $leaseReference = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$ExecutionLease.lease.path)
    $expectedLeasePath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-start.json")
    ).Replace("\", "/")
    $retainedLeaseSha = Assert-Sprint8AReceiptSidecar -Path ([string]$leaseReference.full_path)
    $retainedLease = Get-Content -LiteralPath ([string]$leaseReference.full_path) -Raw | ConvertFrom-Json
    $retainedLeaseStartedAt = ConvertTo-Sprint8ADateTimeOffset `
        -Value $retainedLease.started_at `
        -Label "retained manual UAT execution-lease start"
    if ([string]$leaseReference.path -cne $expectedLeasePath -or
        $retainedLeaseSha -cne [string]$ExecutionLease.lease.sha256 -or
        ($retainedLease.schema_version -isnot [int] -and $retainedLease.schema_version -isnot [long]) -or
        [int]$retainedLease.schema_version -ne 1 -or [string]$retainedLease.sprint -cne "sprint-8a" -or
        [string]$retainedLease.phase -cne "uat-manual-execution-lease" -or [string]$retainedLease.state -cne "executing" -or
        [int]$retainedLease.attempt -ne $Attempt -or [string]$retainedLease.scenario -cne $Scenario -or
        [string]$retainedLease.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$retainedLease.environment_fingerprint -cne $EnvironmentFingerprint -or
        $retainedLeaseStartedAt -ne $StartedAt -or
        $retainedLease.authoritative -isnot [bool] -or [bool]$retainedLease.authoritative -ne $Authoritative -or
        $retainedLease.diagnostic -isnot [bool] -or [bool]$retainedLease.diagnostic -ne $Diagnostic -or
        ($retainedLease.process_id -isnot [int] -and $retainedLease.process_id -isnot [long]) -or
        [int]$retainedLease.process_id -ne [int]$ExecutionLease.original_process_id -or
        [string]$retainedLease.checkpoint.path -cne [string]$ExecutionLease.checkpoint.path -or
        [string]$retainedLease.checkpoint.sha256 -cne [string]$ExecutionLease.checkpoint.sha256) {
        throw "Manual UAT scenario '$Scenario' retained execution lease is malformed or bound to another execution."
    }
    if ([bool]$ExecutionLease.resumed) {
        if ($null -eq $ExecutionLease.resume) {
            throw "Manual UAT scenario '$Scenario' resumed execution omits its authenticated marker."
        }
        Assert-Sprint8AManualUatResumeMarker `
            -MarkerReference $ExecutionLease.resume `
            -LeaseReference $ExecutionLease.lease `
            -LeaseDocument $retainedLease `
            -Scenario $Scenario `
            -Attempt $Attempt `
            -CandidateFingerprint $CandidateFingerprint `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -StartedAt $StartedAt `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $evidenceRootFullPath `
            -ExpectedCurrentProcessId $PID | Out-Null
    } else {
        $canonicalResumePath = [string]$leaseReference.full_path
        if (-not $canonicalResumePath.EndsWith("-start.json", [StringComparison]::Ordinal)) {
            throw "Manual UAT scenario '$Scenario' retained execution lease path is non-canonical."
        }
        $canonicalResumePath = $canonicalResumePath.Substring(
            0,
            $canonicalResumePath.Length - "-start.json".Length
        ) + "-resume.json"
        Repair-Sprint7AEvidencePublication -Path $canonicalResumePath
        if ($null -ne $ExecutionLease.resume -or
            [int]$ExecutionLease.current_process_id -ne [int]$ExecutionLease.original_process_id -or
            (Test-Path -LiteralPath $canonicalResumePath -PathType Leaf) -or
            (Test-Path -LiteralPath "$canonicalResumePath.sha256" -PathType Leaf)) {
            throw "Manual UAT scenario '$Scenario' non-resumed execution has inconsistent process lineage."
        }
    }
    $attemptLock = $ExecutionLease.stream
    try {
    $resolveEvidence = {
        param(
            [Parameter(Mandatory)][object[]]$References,
            [Parameter(Mandatory)][ValidateSet("scenario", "cleanup")][string]$Kind
        )
        @($References | ForEach-Object {
        $reference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$_.path)
        if ((Get-Sprint8AFileSha256 -Path ([string]$reference.full_path)) -cne [string]$_.sha256) {
            throw "Manual UAT scenario '$Scenario' evidence digest does not match '$($_.path)'."
        }
        if ($Kind -ceq "scenario") {
            if (($_.step -isnot [int] -and $_.step -isnot [long])) {
                throw "Manual UAT scenario '$Scenario' evidence requires a typed step number."
            }
            [pscustomobject][ordered]@{
                step = [int]$_.step
                path = [string]$reference.path
                sha256 = [string]$_.sha256
            }
        } else {
            if ([string]$_.kind -cne "canonical-restoration") {
                throw "Manual UAT scenario '$Scenario' cleanup evidence requires kind 'canonical-restoration'."
            }
            [pscustomobject][ordered]@{
                kind = "canonical-restoration"
                path = [string]$reference.path
                sha256 = [string]$_.sha256
            }
        }
        })
    }
    $evidenceReferences = & $resolveEvidence -References $Evidence -Kind "scenario"
    $cleanupEvidenceReferences = & $resolveEvidence -References $CleanupEvidence -Kind "cleanup"
    $target = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $OutputPath
    $expectedOutputPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual/$($Scenario.ToLowerInvariant()).json")
    ).Replace("\", "/")
    if ([string]$target.path -cne $expectedOutputPath) {
        throw "Manual UAT scenario '$Scenario' must publish to '$expectedOutputPath'."
    }
    $checkpoint = Assert-Sprint8AManualUatStartCheckpoint `
        -Attempt $Attempt `
        -Scenario $Scenario `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -ScenarioStartedAt $StartedAt `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath
    Assert-Sprint8AManualUatAttemptOpen `
        -Attempt $Attempt `
        -Checkpoint $checkpoint `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath | Out-Null
    if ([string]$checkpoint.receipt.stage -ceq "diagnostic-manual" -and
        ($Authoritative -or -not $Diagnostic)) {
        throw "Manual UAT scenario '$Scenario' must be non-authoritative diagnostic harvest after scripted invalidation."
    }
    foreach ($priorPath in @(Get-ChildItem -LiteralPath (Split-Path -Parent ([string]$target.full_path)) -Filter "uat-8a-*.json" -File -ErrorAction SilentlyContinue)) {
        if ([IO.Path]::GetFullPath($priorPath.FullName) -ceq [IO.Path]::GetFullPath([string]$target.full_path)) { continue }
        try {
            Assert-Sprint8AReceiptSidecar -Path $priorPath.FullName | Out-Null
            $prior = Get-Content -LiteralPath $priorPath.FullName -Raw | ConvertFrom-Json
            if ([int]$prior.attempt -eq $Attempt -and
                [string]$prior.candidate_fingerprint -ceq $CandidateFingerprint -and
                [string]$prior.environment_fingerprint -ceq $EnvironmentFingerprint -and
                [string]$prior.state -ceq "blocked" -and
                [string]$prior.classification -ceq "product-decision" -and
                (ConvertTo-Sprint8ADateTimeOffset -Value $prior.ended_at -Label "prior manual UAT end") -lt $StartedAt) {
                throw "Manual UAT scenario '$Scenario' is paused by unresolved product decision '$($prior.scenario)'."
            }
            if ([int]$prior.attempt -eq $Attempt -and
                [string]$prior.candidate_fingerprint -ceq $CandidateFingerprint -and
                [string]$prior.environment_fingerprint -ceq $EnvironmentFingerprint -and
                [string]$prior.state -ceq "failed" -and [bool]$prior.authoritative -and
                (ConvertTo-Sprint8ADateTimeOffset -Value $prior.ended_at -Label "prior manual UAT end") -lt $StartedAt -and
                ($Authoritative -or -not $Diagnostic)) {
                throw "Manual UAT scenario '$Scenario' must be diagnostic after '$($prior.scenario)' invalidated the candidate."
            }
        } catch {
            if ($_.Exception.Message -like "Manual UAT scenario '$Scenario' must be diagnostic*" -or
                $_.Exception.Message -like "Manual UAT scenario '$Scenario' is paused by unresolved product decision*") { throw }
            throw "Manual UAT scenario '$Scenario' cannot authenticate prior receipt '$($priorPath.FullName)': $($_.Exception.Message)"
        }
    }
    $receipt = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        phase = "uat-manual-scenario"
        attempt = $Attempt
        authoritative = $Authoritative
        diagnostic = $Diagnostic
        scenario = $Scenario
        state = $State
        candidate_fingerprint = $CandidateFingerprint
        environment_fingerprint = $EnvironmentFingerprint
        assertions_started = $AssertionsStarted
        assertions_started_at = if ($AssertionsStarted) { $StartedAt.ToString("o") } else { $null }
        started_at = $StartedAt.ToString("o")
        ended_at = $EndedAt.ToString("o")
        duration_ms = [long][Math]::Max(0, ($EndedAt - $StartedAt).TotalMilliseconds)
        role = $Role
        starting_state = $StartingState
        actions = @($Actions)
        expected_result = $ExpectedResult
        actual_result = $ActualResult
        classification = $Classification
        classification_source = if (-not [string]::IsNullOrWhiteSpace($Classification)) { "manual_operator" } else { $null }
        failure_message = $FailureMessage
        blocked_reason = $BlockedReason
        scenario_contract = (Get-Sprint8AManualUatScenarioContract -Scenario $Scenario).document
        start_checkpoint = $checkpoint.reference
        execution_lease = $ExecutionLease.lease
        resumed = [bool]$ExecutionLease.resumed
        execution_resume = $ExecutionLease.resume
        evidence = $evidenceReferences
        cleanup_restoration = [pscustomobject][ordered]@{
            required = $true
            result = "canonical_topology_verified"
            evidence = $cleanupEvidenceReferences
        }
    }
    Assert-Sprint8AManualUatReceipt `
        -Receipt $receipt `
        -ExpectedScenario $Scenario `
        -ExpectedAttempt $Attempt `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint | Out-Null
    $receipt = $receipt | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    Assert-Sprint8AManualUatReceipt `
        -Receipt $receipt `
        -ExpectedScenario $Scenario `
        -ExpectedAttempt $Attempt `
        -CandidateFingerprint $CandidateFingerprint `
        -EnvironmentFingerprint $EnvironmentFingerprint | Out-Null
    $expectedReceiptSha = Get-Sprint8AStringSha256 -Text (($receipt | ConvertTo-Json -Depth 30) + "`n")
    $receiptReference = [pscustomobject][ordered]@{
        path = [string]$target.path
        sha256 = $expectedReceiptSha
    }
    $completionTarget = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([string]$ExecutionLease.completion_path)
    $expectedCompletionPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-complete.json")
    ).Replace("\", "/")
    if ([string]$completionTarget.path -cne $expectedCompletionPath) {
        throw "Manual UAT scenario '$Scenario' execution lease has a non-canonical completion path."
    }
    $completionDocument = [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-execution-lease"
        authoritative = $false; state = "completed"; attempt = $Attempt; scenario = $Scenario
        candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
        started_at = $StartedAt.ToString("o"); ended_at = $EndedAt.ToString("o")
        lease = $ExecutionLease.lease; resumed = [bool]$ExecutionLease.resumed
        resume = $ExecutionLease.resume; receipt = $receiptReference
    }
    $completionDocument = $completionDocument | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $completionReference = [pscustomobject][ordered]@{
        path = [string]$completionTarget.path
        sha256 = Get-Sprint8AStringSha256 -Text (($completionDocument | ConvertTo-Json -Depth 30) + "`n")
    }
    $preparedTarget = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath `
        -Path ([IO.Path]::GetRelativePath(
            $RepositoryRoot,
            (Join-Path $evidenceRootFullPath "uat/attempt-$Attempt/manual-leases/$($Scenario.ToLowerInvariant())-publication-prepared.json")
        ).Replace("\", "/"))
    $preparedDocument = [pscustomobject][ordered]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "uat-manual-publication-checkpoint"
        authoritative = $false; state = "prepared"; attempt = $Attempt; scenario = $Scenario
        candidate_fingerprint = $CandidateFingerprint; environment_fingerprint = $EnvironmentFingerprint
        prepared_at = [DateTimeOffset]::UtcNow.ToString("o")
        lease = $ExecutionLease.lease; resume = $ExecutionLease.resume
        receipt = [pscustomobject][ordered]@{
            path = [string]$receiptReference.path; sha256 = [string]$receiptReference.sha256; document = $receipt
        }
        completion = [pscustomobject][ordered]@{
            path = [string]$completionReference.path; sha256 = [string]$completionReference.sha256
            document = $completionDocument
        }
    }
    $expectedPreparedSha = Get-Sprint8AStringSha256 -Text (($preparedDocument | ConvertTo-Json -Depth 30) + "`n")
    Repair-Sprint7AEvidencePublication -Path ([string]$preparedTarget.full_path)
    $preparedArtifactExists = Test-Path -LiteralPath ([string]$preparedTarget.full_path) -PathType Leaf
    $preparedSidecarExists = Test-Path -LiteralPath "$([string]$preparedTarget.full_path).sha256" -PathType Leaf
    if ($preparedArtifactExists -or $preparedSidecarExists) {
        if (-not $preparedArtifactExists -or -not $preparedSidecarExists -or
            (Assert-Sprint8AReceiptSidecar -Path ([string]$preparedTarget.full_path)) -cne $expectedPreparedSha) {
            throw "Manual UAT scenario '$Scenario' already has a different or incomplete prepared publication."
        }
    } else {
        Publish-Sprint7AEvidence -Document $preparedDocument -OutputPath ([string]$preparedTarget.full_path) | Out-Null
    }
    $publication = Repair-Sprint8AManualUatPreparedPublication `
        -CheckpointPath ([string]$preparedTarget.full_path) `
        -Attempt $Attempt `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $evidenceRootFullPath
    $publication.receipt
    } finally {
        $attemptLock.Dispose()
    }
}

function Repair-Sprint8AEvidenceRootPublications {
    param([Parameter(Mandatory)][string]$EvidenceRoot)

    $evidence = [IO.Path]::GetFullPath($EvidenceRoot)
    if (-not (Test-Path -LiteralPath $evidence -PathType Container)) {
        throw "Sprint 8A evidence publication recovery root does not exist: '$evidence'."
    }
    $journalSuffix = ".publish-journal.json"
    foreach ($journal in @(Get-ChildItem -LiteralPath $evidence -Recurse -Force -File | Where-Object {
        $_.Name.EndsWith($journalSuffix, [StringComparison]::Ordinal)
    })) {
        $targetPath = $journal.FullName.Substring(0, $journal.FullName.Length - $journalSuffix.Length)
        Repair-Sprint7AEvidencePublication -Path $targetPath
    }
    $remainingControls = @(Get-ChildItem -LiteralPath $evidence -Recurse -Force -File | Where-Object {
        $_.Name.EndsWith($journalSuffix, [StringComparison]::Ordinal) -or
            $_.Name.EndsWith(".rollback", [StringComparison]::Ordinal) -or
            $_.Name -cmatch '^\..+\.[0-9a-f]{32}\.tmp(?:\.sha256)?$' -or
            $_.Name -cmatch '^\..+\.[0-9a-f]{32}\.(?:json|sha256)\.tmp$' -or
            $_.Name -cmatch '^\..+\.publish-journal\.json\.[0-9a-f]{32}\.tmp$'
    } | ForEach-Object {
        [IO.Path]::GetRelativePath($evidence, $_.FullName).Replace("\", "/")
    } | Sort-Object -CaseSensitive)
    if ($remainingControls.Count -gt 0) {
        throw "Sprint 8A evidence inventory found unresolved publisher control file(s): $($remainingControls -join ', ')."
    }
}

function Get-Sprint8AEvidenceFileManifestEntries {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [AllowEmptyCollection()][object[]]$Overrides = @(),
        [switch]$IgnoreExistingManifest
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot))
    }
    if (-not (Test-Path -LiteralPath $evidence -PathType Container)) {
        throw "Sprint 8A evidence root does not exist: '$evidence'."
    }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidence
    $excluded = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($path in @(
        (Join-Path $evidence "evidence-manifest.json"),
        (Join-Path $evidence "evidence-manifest.json.sha256"),
        (Join-Path $evidence "validation-attempt.lock")
    )) {
        $excluded.Add([IO.Path]::GetFullPath($path)) | Out-Null
    }
    $entries = [ordered]@{}
    foreach ($file in @(Get-ChildItem -LiteralPath $evidence -Recurse -Force -File)) {
        if ($excluded.Contains([IO.Path]::GetFullPath($file.FullName))) { continue }
        $path = [IO.Path]::GetRelativePath($repository, $file.FullName).Replace("\", "/")
        $entries[$path] = [pscustomobject][ordered]@{
            path = $path
            sha256 = Get-Sprint8AFileSha256 -Path $file.FullName
            phase = "retained-evidence"
            authoritative = $false
            status = "retained"
        }
    }
    $overridePaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $authorizedMutablePaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $normalizedOverrides = [Collections.Generic.List[object]]::new()
    foreach ($override in @($Overrides)) {
        if ([string]::IsNullOrWhiteSpace([string]$override.path)) {
            throw "Sprint 8A evidence manifest overrides require canonical paths."
        }
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Path ([string]$override.path)
        if ($excluded.Contains([IO.Path]::GetFullPath([string]$resolved.full_path))) {
            throw "Sprint 8A evidence manifest cannot inventory its manifest pair or live attempt lock."
        }
        if (-not $overridePaths.Add([string]$resolved.path)) {
            throw "Sprint 8A evidence manifest overrides contain duplicate path '$($resolved.path)'."
        }
        $authorizedMutablePaths.Add([string]$resolved.path) | Out-Null
        $authorizedMutablePaths.Add("$([string]$resolved.path).sha256") | Out-Null
        $normalizedOverrides.Add([pscustomobject][ordered]@{ override = $override; resolved = $resolved })
    }
    $manifestPath = Join-Path $evidence "evidence-manifest.json"
    if (-not $IgnoreExistingManifest -and (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        Assert-Sprint8AReceiptSidecar -Path $manifestPath | Out-Null
        $existingManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        if (($existingManifest.schema_version -isnot [int] -and $existingManifest.schema_version -isnot [long]) -or
            [int]$existingManifest.schema_version -ne 1 -or
            [string]$existingManifest.sprint -cne "sprint-8a" -or
            [string]$existingManifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
            throw "The existing Sprint 8A evidence manifest is malformed."
        }
        $existingManifestEntries = @($existingManifest.entries)
        $existingManifestPaths = @($existingManifestEntries | ForEach-Object { [string]$_.path })
        if (@($existingManifestPaths | Sort-Object -Unique).Count -ne $existingManifestPaths.Count) {
            throw "The existing Sprint 8A evidence manifest contains duplicate paths."
        }
        $canonicalUatPath = [IO.Path]::GetRelativePath($repository, (Join-Path $evidence "uat-result.json")).Replace("\", "/")
        $canonicalUatSidecarPath = "$canonicalUatPath.sha256"
        foreach ($existing in $existingManifestEntries) {
            $resolved = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -Path ([string]$existing.path)
            if ($excluded.Contains([IO.Path]::GetFullPath([string]$resolved.full_path))) {
                throw "The existing Sprint 8A evidence manifest inventories an excluded control file."
            }
            $declaredSha = [string]$existing.sha256
            $exists = Test-Path -LiteralPath ([string]$resolved.full_path) -PathType Leaf
            $allowedMissingCommitment = -not $exists -and
                [string]$resolved.path -in @($canonicalUatPath, $canonicalUatSidecarPath) -and
                $declaredSha -match '^[0-9a-f]{64}$' -and
                [string]$existing.phase -ceq "uat" -and
                $existing.authoritative -is [bool] -and [bool]$existing.authoritative -and
                [string]$existing.status -ceq "committed"
            if ($declaredSha -notmatch '^[0-9a-f]{64}$' -or
                $existing.authoritative -isnot [bool] -or
                [string]::IsNullOrWhiteSpace([string]$existing.phase) -or
                [string]::IsNullOrWhiteSpace([string]$existing.status) -or
                (-not $exists -and -not $allowedMissingCommitment) -or
                ($exists -and -not $authorizedMutablePaths.Contains([string]$resolved.path) -and
                    (Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)) -cne $declaredSha)) {
                throw "The existing Sprint 8A evidence manifest has stale or malformed entry '$($existing.path)'."
            }
            $entries[[string]$resolved.path] = $existing
        }
    }
    foreach ($normalizedOverride in $normalizedOverrides) {
        $override = $normalizedOverride.override
        $resolved = $normalizedOverride.resolved
        $declaredOverrideSha = if ($override.PSObject.Properties.Name -contains "sha256") {
            [string]$override.sha256
        } else { "" }
        $sha = if (Test-Path -LiteralPath ([string]$resolved.full_path) -PathType Leaf) {
            $actualSha = Get-Sprint8AFileSha256 -Path ([string]$resolved.full_path)
            if (-not [string]::IsNullOrWhiteSpace($declaredOverrideSha) -and $declaredOverrideSha -cne $actualSha) {
                throw "Sprint 8A evidence manifest override for '$($resolved.path)' differs from retained bytes."
            }
            $actualSha
        } else {
            $declaredOverrideSha
        }
        $entries[[string]$resolved.path] = [pscustomobject][ordered]@{
            path = [string]$resolved.path
            sha256 = $sha
            phase = [string]$override.phase
            authoritative = $override.authoritative
            status = [string]$override.status
        }
        $sidecarPath = "$([string]$resolved.path).sha256"
        if (-not $overridePaths.Contains($sidecarPath) -and $entries.Contains($sidecarPath) -and
            (Test-Path -LiteralPath "$([string]$resolved.full_path).sha256" -PathType Leaf)) {
            $entries[$sidecarPath] = [pscustomobject][ordered]@{
                path = $sidecarPath
                sha256 = Get-Sprint8AFileSha256 -Path "$([string]$resolved.full_path).sha256"
                phase = "retained-evidence"
                authoritative = $false
                status = "retained"
            }
        }
    }
    @($entries.Values | Sort-Object path)
}

function Assert-Sprint8AEvidenceManifestCompleteness {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [string]$ManifestPath,
        [switch]$AllowMissingCanonicalUatCommitment
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $EvidenceRoot))
    }
    $manifestFullPath = if ([string]::IsNullOrWhiteSpace($ManifestPath)) {
        Join-Path $evidence "evidence-manifest.json"
    } elseif ([IO.Path]::IsPathRooted($ManifestPath)) {
        [IO.Path]::GetFullPath($ManifestPath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $ManifestPath))
    }
    $expectedManifestPath = [IO.Path]::GetFullPath((Join-Path $evidence "evidence-manifest.json"))
    if ($manifestFullPath -cne $expectedManifestPath) {
        throw "Sprint 8A evidence-manifest completeness must authenticate the canonical manifest."
    }
    Repair-Sprint8AEvidenceRootPublications -EvidenceRoot $evidence
    Assert-Sprint8AReceiptSidecar -Path $manifestFullPath | Out-Null
    $manifest = Get-Content -LiteralPath $manifestFullPath -Raw | ConvertFrom-Json
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 1 -or
        [string]$manifest.sprint -cne "sprint-8a" -or
        [string]$manifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
        throw "The Sprint 8A evidence manifest is malformed."
    }
    $manifestEntries = @($manifest.entries)
    $manifestPaths = @($manifestEntries | ForEach-Object { [string]$_.path })
    if (@($manifestPaths | Sort-Object -Unique).Count -ne $manifestPaths.Count) {
        throw "The Sprint 8A evidence manifest contains duplicate paths."
    }
    $actualEntries = @(Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $repository `
        -EvidenceRoot $evidence `
        -IgnoreExistingManifest)
    $actual = [ordered]@{}
    foreach ($entry in $actualEntries) { $actual[[string]$entry.path] = $entry }
    $canonicalUatPath = [IO.Path]::GetRelativePath(
        $repository,
        (Join-Path $evidence "uat-result.json")
    ).Replace("\", "/")
    $canonicalUatSidecarPath = "$canonicalUatPath.sha256"
    $canonicalCommitments = @($manifestEntries | Where-Object {
        [string]$_.path -in @($canonicalUatPath, $canonicalUatSidecarPath)
    })
    foreach ($entry in $manifestEntries) {
        $resolved = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Path ([string]$entry.path)
        $declaredSha = [string]$entry.sha256
        if ($declaredSha -notmatch '^[0-9a-f]{64}$' -or
            $entry.authoritative -isnot [bool] -or
            [string]::IsNullOrWhiteSpace([string]$entry.phase) -or
            [string]::IsNullOrWhiteSpace([string]$entry.status)) {
            throw "Lifecycle evidence '$($entry.path)' has malformed digest, phase, authority, or status metadata."
        }
        if ($actual.Contains([string]$resolved.path)) {
            if ([string]$actual[[string]$resolved.path].sha256 -cne $declaredSha) {
                throw "Lifecycle evidence '$($entry.path)' differs from its manifest SHA-256."
            }
            $actual.Remove([string]$resolved.path)
            continue
        }
        $allowedMissing = $AllowMissingCanonicalUatCommitment -and
            [string]$resolved.path -in @($canonicalUatPath, $canonicalUatSidecarPath) -and
            [string]$entry.phase -ceq "uat" -and
            [bool]$entry.authoritative -and
            [string]$entry.status -ceq "committed"
        if (-not $allowedMissing) {
            throw "Manifested Sprint 8A evidence '$($entry.path)' is missing."
        }
    }
    if ($actual.Count -ne 0) {
        throw "The Sprint 8A evidence manifest omits retained file(s): $(@($actual.Keys) -join ', ')."
    }
    if ($AllowMissingCanonicalUatCommitment -and $canonicalCommitments.Count -gt 0) {
        if ($canonicalCommitments.Count -ne 2 -or
            @($canonicalCommitments | Where-Object {
                [string]$_.phase -cne "uat" -or $_.authoritative -isnot [bool] -or
                    -not [bool]$_.authoritative -or [string]$_.status -cne "committed"
            }).Count -ne 0) {
            throw "The canonical UAT result commitment must include its exact JSON and SHA-256 sidecar pair."
        }
        $jsonCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatPath)[0]
        $sidecarCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatSidecarPath)[0]
        if ([string]$sidecarCommitment.sha256 -cne
            (Get-Sprint8AStringSha256 -Text "$([string]$jsonCommitment.sha256)`n")) {
            throw "The canonical UAT result sidecar commitment does not authenticate its JSON digest."
        }
    }
    $manifest
}

function Get-Sprint8AEvidenceManifestReplacementPaths {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$ExistingEntries,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$UpdatedEntries
    )

    $existingByPath = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($ExistingEntries)) {
        $path = [string]$entry.path
        if ([string]::IsNullOrWhiteSpace($path) -or -not $existingByPath.TryAdd($path, $entry)) {
            throw "Sprint 8A evidence manifest replacement comparison requires unique nonempty existing paths."
        }
    }
    $updatedPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $replacementPaths = [Collections.Generic.List[string]]::new()
    foreach ($entry in @($UpdatedEntries)) {
        $path = [string]$entry.path
        if ([string]::IsNullOrWhiteSpace($path) -or -not $updatedPaths.Add($path)) {
            throw "Sprint 8A evidence manifest replacement comparison requires unique nonempty updated paths."
        }
        if (-not $existingByPath.ContainsKey($path)) { continue }
        $existing = $existingByPath[$path]
        $authorityChanged = $existing.authoritative -isnot [bool] -or
            $entry.authoritative -isnot [bool] -or
            [bool]$existing.authoritative -ne [bool]$entry.authoritative
        if ([string]$existing.sha256 -cne [string]$entry.sha256 -or
            [string]$existing.phase -cne [string]$entry.phase -or
            $authorityChanged -or
            [string]$existing.status -cne [string]$entry.status) {
            $replacementPaths.Add($path)
        }
    }
    @($replacementPaths)
}

function Publish-Sprint8AEvidenceManifest {
    param(
        [Parameter(Mandatory)][object[]]$Entries,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$OutputPath,
        [switch]$Merge,
        [AllowEmptyCollection()][string[]]$AuthorizedReplacementPaths = @()
    )

    $target = Resolve-Sprint8AEvidenceReference `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRoot $EvidenceRoot `
        -Path $OutputPath
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $expectedManifestPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "evidence-manifest.json")
    ).Replace("\", "/")
    if ([string]$target.path -cne $expectedManifestPath) {
        throw "Sprint 8A evidence manifest must use canonical path '$expectedManifestPath'."
    }
    $existingEntries = @()
    $hasExistingManifest = Test-Path -LiteralPath ([string]$target.full_path) -PathType Leaf
    $replacementPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    if (-not $hasExistingManifest -and @($AuthorizedReplacementPaths).Count -gt 0) {
        $unusedInitialReplacementPaths = @($AuthorizedReplacementPaths | Sort-Object -CaseSensitive -Unique)
        throw "Sprint 8A evidence manifest replacement authorization was not consumed by an exact metadata or digest change: $($unusedInitialReplacementPaths -join ', ')."
    }
    if ($hasExistingManifest) {
        if (-not $Merge) {
            throw "The Sprint 8A evidence manifest already exists; use -Merge to preserve and extend it."
        }
        Assert-Sprint8AReceiptSidecar -Path ([string]$target.full_path) | Out-Null
        $existingManifest = Get-Content -LiteralPath ([string]$target.full_path) -Raw | ConvertFrom-Json
        if (($existingManifest.schema_version -isnot [int] -and $existingManifest.schema_version -isnot [long]) -or
            [int]$existingManifest.schema_version -ne 1 -or
            [string]$existingManifest.sprint -cne "sprint-8a" -or
            [string]$existingManifest.contract -cne "tessara.sprint-8a.evidence-manifest") {
            throw "The existing Sprint 8A evidence manifest is malformed."
        }
        $existingPaths = @($existingManifest.entries | ForEach-Object { [string]$_.path })
        if (@($existingPaths | Sort-Object -Unique).Count -ne $existingPaths.Count) {
            throw "The existing Sprint 8A evidence manifest contains duplicate paths."
        }
        $replacementEntries = [Collections.Generic.List[object]]::new()
        foreach ($replacementPath in @($AuthorizedReplacementPaths)) {
            $resolvedReplacement = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path $replacementPath
            if (-not $replacementPaths.Add([string]$resolvedReplacement.path)) {
                throw "Sprint 8A evidence manifest replacement authorization contains duplicate paths."
            }
            $matches = @($Entries | Where-Object { [string]$_.path -ceq [string]$resolvedReplacement.path })
            if ($matches.Count -ne 1) {
                throw "Sprint 8A evidence manifest replacement '$($resolvedReplacement.path)' lacks one exact update entry."
            }
            $replacementEntries.Add($matches[0])
        }
        Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Overrides @($replacementEntries) | Out-Null
        $existingEntries = @($existingManifest.entries)
    }
    $newEntryPaths = @($Entries | ForEach-Object { [string]$_.path })
    if (@($newEntryPaths | Sort-Object -Unique).Count -ne $newEntryPaths.Count) {
        throw "Sprint 8A evidence manifest update contains duplicate paths."
    }
    $entryMap = [ordered]@{}
    foreach ($entry in @($existingEntries) + @($Entries)) {
        if ([string]::IsNullOrWhiteSpace([string]$entry.path)) {
            throw "Sprint 8A evidence manifest entries require canonical paths."
        }
        $entryMap[[string]$entry.path] = $entry
    }
    $canonicalUatResultPath = [IO.Path]::GetRelativePath(
        $RepositoryRoot,
        (Join-Path $evidenceRootFullPath "uat-result.json")
    ).Replace("\", "/")
    $canonicalUatResultSidecarPath = "$canonicalUatResultPath.sha256"
    $validated = @($entryMap.Values | ForEach-Object {
        $reference = Resolve-Sprint8AEvidenceReference `
            -RepositoryRoot $RepositoryRoot `
            -EvidenceRoot $EvidenceRoot `
            -Path ([string]$_.path)
        $exists = Test-Path -LiteralPath ([string]$reference.full_path) -PathType Leaf
        $declaredSha = [string]$_.sha256
        $status = [string]$_.status
        if (-not $exists -and
            ($status -cne "committed" -or
                $declaredSha -notmatch '^[0-9a-f]{64}$' -or
                [string]$reference.path -notin @($canonicalUatResultPath, $canonicalUatResultSidecarPath) -or
                [string]$_.phase -cne "uat" -or
                $_.authoritative -isnot [bool] -or -not [bool]$_.authoritative)) {
            throw "Required lifecycle evidence '$($_.path)' is missing and has no content-addressed commitment."
        }
        $sha = if ($exists) {
            Get-Sprint8AFileSha256 -Path ([string]$reference.full_path)
        } else {
            $declaredSha
        }
        if (-not [string]::IsNullOrWhiteSpace($declaredSha) -and $declaredSha -cne $sha) {
            throw "Lifecycle evidence '$($_.path)' differs from its declared SHA-256."
        }
        if ($_.authoritative -isnot [bool] -or
            [string]::IsNullOrWhiteSpace([string]$_.phase) -or
            [string]::IsNullOrWhiteSpace($status)) {
            throw "Lifecycle evidence '$($_.path)' has malformed phase, authority, or status metadata."
        }
        [pscustomobject][ordered]@{
            path = [string]$reference.path
            sha256 = $sha
            phase = [string]$_.phase
            authoritative = $_.authoritative
            status = $status
        }
    } | Sort-Object path)
    $canonicalCommitments = @($validated | Where-Object {
        [string]$_.path -in @($canonicalUatResultPath, $canonicalUatResultSidecarPath)
    })
    if ($canonicalCommitments.Count -gt 0) {
        if ($canonicalCommitments.Count -ne 2 -or
            @($canonicalCommitments | Where-Object {
                [string]$_.phase -cne "uat" -or -not [bool]$_.authoritative -or
                    [string]$_.status -cne "committed"
            }).Count -ne 0) {
            throw "The canonical UAT result commitment must include its exact JSON and SHA-256 sidecar pair."
        }
        $jsonCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatResultPath)[0]
        $sidecarCommitment = @($canonicalCommitments | Where-Object path -CEQ $canonicalUatResultSidecarPath)[0]
        if ([string]$sidecarCommitment.sha256 -cne
            (Get-Sprint8AStringSha256 -Text "$([string]$jsonCommitment.sha256)`n")) {
            throw "The canonical UAT result sidecar commitment does not authenticate its JSON digest."
        }
    }
    if ($hasExistingManifest) {
        $normalizedExistingEntries = @($existingEntries | ForEach-Object {
            $reference = Resolve-Sprint8AEvidenceReference `
                -RepositoryRoot $RepositoryRoot `
                -EvidenceRoot $EvidenceRoot `
                -Path ([string]$_.path)
            [pscustomobject][ordered]@{
                path = [string]$reference.path
                sha256 = [string]$_.sha256
                phase = [string]$_.phase
                authoritative = $_.authoritative
                status = [string]$_.status
            }
        })
        $requiredReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
            -ExistingEntries $normalizedExistingEntries `
            -UpdatedEntries $validated)
        $unauthorizedReplacementPaths = @($requiredReplacementPaths | Where-Object {
            -not $replacementPaths.Contains([string]$_)
        })
        if ($unauthorizedReplacementPaths.Count -gt 0) {
            throw "Sprint 8A evidence manifest entry metadata or digest changed without exact replacement authorization: $($unauthorizedReplacementPaths -join ', ')."
        }
        $requiredReplacementPathSet = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($requiredReplacementPath in $requiredReplacementPaths) {
            [void]$requiredReplacementPathSet.Add([string]$requiredReplacementPath)
        }
        $unusedReplacementPaths = @($replacementPaths | Where-Object {
            -not $requiredReplacementPathSet.Contains([string]$_)
        } | Sort-Object -CaseSensitive)
        if ($unusedReplacementPaths.Count -gt 0) {
            throw "Sprint 8A evidence manifest replacement authorization was not consumed by an exact metadata or digest change: $($unusedReplacementPaths -join ', ')."
        }
    }
    $manifest = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.evidence-manifest"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        entries = $validated
    }
    Publish-Sprint7AEvidence -Document $manifest -OutputPath ([string]$target.full_path) -Overwrite:$Merge | Out-Null
    [pscustomobject][ordered]@{
        path = [string]$target.path
        sha256 = Assert-Sprint8AReceiptSidecar -Path ([string]$target.full_path)
    }
}

function Test-Sprint8AEvidenceManifestContract {
    $repository = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
    $relativeRoot = "artifacts/.sprint-8a-manifest-selftest-$([guid]::NewGuid().ToString('N'))"
    $evidence = [IO.Path]::GetFullPath((Join-Path $repository $relativeRoot))
    $artifactsRoot = [IO.Path]::GetFullPath((Join-Path $repository "artifacts"))
    if (-not $evidence.StartsWith("$($artifactsRoot.TrimEnd('\'))\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Sprint 8A evidence-manifest self-test target escaped the repository artifacts root."
    }
    [IO.Directory]::CreateDirectory($evidence) | Out-Null
    try {
        $rawPath = Join-Path $evidence "raw.txt"
        [IO.File]::WriteAllText($rawPath, "retained`n", [Text.UTF8Encoding]::new($false))
        $stablePath = Join-Path $evidence "stable.txt"
        [IO.File]::WriteAllText($stablePath, "stable`n", [Text.UTF8Encoding]::new($false))
        $journalTargetPath = Join-Path $evidence "journal-target.json"
        Publish-Sprint7AEvidence `
            -Document ([pscustomobject][ordered]@{ schema_version = 1; state = "retained" }) `
            -OutputPath $journalTargetPath | Out-Null
        $journalPath = "$journalTargetPath.publish-journal.json"
        [IO.File]::WriteAllText($journalPath, "{", [Text.UTF8Encoding]::new($false))
        $journalRecoveryEntries = @(Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence)
        if ((Test-Path -LiteralPath $journalPath) -or
            @($journalRecoveryEntries | Where-Object { [string]$_.path -like "*.publish-journal.json" }).Count -ne 0) {
            throw "Sprint 8A evidence-manifest self-test retained or inventoried a repaired publisher journal."
        }
        foreach ($transientPath in @(
            "$journalTargetPath.rollback",
            (Join-Path $evidence ".orphan.$([guid]::NewGuid().ToString('N')).tmp")
        )) {
            [IO.File]::WriteAllText($transientPath, "publisher-control", [Text.UTF8Encoding]::new($false))
            $controlRejected = $false
            try {
                Get-Sprint8AEvidenceFileManifestEntries `
                    -RepositoryRoot $repository `
                    -EvidenceRoot $evidence | Out-Null
            } catch {
                if (-not $_.Exception.Message.Contains("unresolved publisher control file")) { throw }
                $controlRejected = $true
            }
            Remove-Item -LiteralPath $transientPath -Force
            if (-not $controlRejected) {
                throw "Sprint 8A evidence-manifest self-test inventoried unresolved publisher control '$transientPath'."
            }
        }
        $manifestPath = Join-Path $evidence "evidence-manifest.json"
        $manifestOutput = "$relativeRoot/evidence-manifest.json"
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput | Out-Null
        [IO.File]::WriteAllText("$manifestPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath | Out-Null
        if (Test-Path -LiteralPath "$manifestPath.publish-journal.json") {
            throw "Sprint 8A evidence-manifest completeness did not repair its target-bound publisher journal."
        }
        [IO.File]::WriteAllText($rawPath, "tampered`n", [Text.UTF8Encoding]::new($false))
        $tamperRejected = $false
        try {
            Assert-Sprint8AEvidenceManifestCompleteness `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -ManifestPath $manifestPath | Out-Null
        } catch { $tamperRejected = $true }
        if (-not $tamperRejected) {
            throw "Sprint 8A evidence-manifest self-test accepted changed retained evidence."
        }
        [IO.File]::WriteAllText($rawPath, "retained`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($rawPath, "intentional-update`n", [Text.UTF8Encoding]::new($false))
        $rawRelative = "$relativeRoot/raw.txt"
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @([pscustomobject]@{
                path = $rawRelative; sha256 = Get-Sprint8AFileSha256 -Path $rawPath
                phase = "self-test-mutable"; authoritative = $false; status = "updated"
            })
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput `
            -Merge `
            -AuthorizedReplacementPaths @($rawRelative) | Out-Null
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath | Out-Null
        $rawSha = Get-Sprint8AFileSha256 -Path $rawPath
        foreach ($metadataCase in @(
            [pscustomobject]@{
                label = "phase"; phase = "self-test-metadata"; authoritative = $false; status = "updated"
            },
            [pscustomobject]@{
                label = "authority"; phase = "self-test-mutable"; authoritative = $true; status = "updated"
            },
            [pscustomobject]@{
                label = "status"; phase = "self-test-mutable"; authoritative = $false; status = "metadata-updated"
            }
        )) {
            $metadataEntries = Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -Overrides @([pscustomobject]@{
                    path = $rawRelative; sha256 = $rawSha
                    phase = [string]$metadataCase.phase
                    authoritative = [bool]$metadataCase.authoritative
                    status = [string]$metadataCase.status
                })
            $currentManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
            $requiredReplacementPaths = @(Get-Sprint8AEvidenceManifestReplacementPaths `
                -ExistingEntries @($currentManifest.entries) `
                -UpdatedEntries $metadataEntries)
            if ($requiredReplacementPaths.Count -ne 1 -or
                [string]$requiredReplacementPaths[0] -cne $rawRelative) {
                throw "Sprint 8A evidence-manifest self-test did not classify the $($metadataCase.label)-only update as one replacement."
            }
            $metadataRejected = $false
            try {
                Publish-Sprint8AEvidenceManifest `
                    -Entries $metadataEntries `
                    -RepositoryRoot $repository `
                    -EvidenceRoot $evidence `
                    -OutputPath $manifestOutput `
                    -Merge | Out-Null
            } catch {
                if (-not $_.Exception.Message.Contains("without exact replacement authorization")) { throw }
                $metadataRejected = $true
            }
            if (-not $metadataRejected) {
                throw "Sprint 8A evidence-manifest self-test accepted an unauthorized $($metadataCase.label)-only replacement."
            }
        }
        $authorizedMetadataEntries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @([pscustomobject]@{
                path = $rawRelative; sha256 = $rawSha
                phase = "self-test-metadata"; authoritative = $true; status = "metadata-updated"
            })
        Publish-Sprint8AEvidenceManifest `
            -Entries $authorizedMetadataEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput `
            -Merge `
            -AuthorizedReplacementPaths @($rawRelative) | Out-Null
        $metadataManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $metadataEntry = @($metadataManifest.entries | Where-Object path -CEQ $rawRelative)
        if ($metadataEntry.Count -ne 1 -or [string]$metadataEntry[0].sha256 -cne $rawSha -or
            [string]$metadataEntry[0].phase -cne "self-test-metadata" -or
            $metadataEntry[0].authoritative -isnot [bool] -or -not [bool]$metadataEntry[0].authoritative -or
            [string]$metadataEntry[0].status -cne "metadata-updated") {
            throw "Sprint 8A evidence-manifest self-test did not retain an authorized full-metadata replacement."
        }
        $unusedAuthorizationRejected = $false
        try {
            Publish-Sprint8AEvidenceManifest `
                -Entries $authorizedMetadataEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -OutputPath $manifestOutput `
                -Merge `
                -AuthorizedReplacementPaths @($rawRelative) | Out-Null
        } catch {
            if (-not $_.Exception.Message.Contains("replacement authorization was not consumed")) { throw }
            $unusedAuthorizationRejected = $true
        }
        if (-not $unusedAuthorizationRejected) {
            throw "Sprint 8A evidence-manifest self-test accepted an unused replacement authorization."
        }
        [IO.File]::WriteAllText($stablePath, "unrelated-tamper`n", [Text.UTF8Encoding]::new($false))
        $unrelatedTamperRejected = $false
        try {
            Get-Sprint8AEvidenceFileManifestEntries `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -Overrides @([pscustomobject]@{
                    path = $rawRelative; sha256 = Get-Sprint8AFileSha256 -Path $rawPath
                    phase = "self-test-mutable"; authoritative = $false; status = "updated"
                }) | Out-Null
        } catch { $unrelatedTamperRejected = $true }
        if (-not $unrelatedTamperRejected) {
            throw "Sprint 8A evidence-manifest self-test allowed an unrelated tamper beside an authorized replacement."
        }
        [IO.File]::WriteAllText($stablePath, "stable`n", [Text.UTF8Encoding]::new($false))
        $canonicalDocument = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "uat"; state = "passed"
        }
        $canonicalSha = Get-Sprint8AStringSha256 -Text (($canonicalDocument | ConvertTo-Json -Depth 30) + "`n")
        $canonicalPath = "$relativeRoot/uat-result.json"
        $commitments = @(
            [pscustomobject][ordered]@{
                path = $canonicalPath; sha256 = $canonicalSha
                phase = "uat"; authoritative = $true; status = "committed"
            },
            [pscustomobject][ordered]@{
                path = "$canonicalPath.sha256"
                sha256 = Get-Sprint8AStringSha256 -Text "$canonicalSha`n"
                phase = "uat"; authoritative = $true; status = "committed"
            }
        )
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides $commitments
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath $manifestOutput `
            -Merge | Out-Null
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath `
            -AllowMissingCanonicalUatCommitment | Out-Null
        $missingRejected = $false
        try {
            Assert-Sprint8AEvidenceManifestCompleteness `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence `
                -ManifestPath $manifestPath | Out-Null
        } catch { $missingRejected = $true }
        if (-not $missingRejected) {
            throw "Sprint 8A evidence-manifest self-test accepted unpublished canonical evidence."
        }
        Publish-Sprint7AEvidence -Document $canonicalDocument -OutputPath (Join-Path $evidence "uat-result.json") | Out-Null
        Assert-Sprint8AEvidenceManifestCompleteness `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -ManifestPath $manifestPath | Out-Null
    } finally {
        if (Test-Path -LiteralPath $evidence -PathType Container) {
            Remove-Item -LiteralPath $evidence -Recurse -Force
        }
    }
}

function Test-Sprint8AManualUatAttemptAuthority {
    $repository = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
    $relativeRoot = "artifacts/.sprint-8a-manual-authority-selftest-$([guid]::NewGuid().ToString('N'))"
    $evidence = [IO.Path]::GetFullPath((Join-Path $repository $relativeRoot))
    $artifactsRoot = [IO.Path]::GetFullPath((Join-Path $repository "artifacts"))
    if (-not $evidence.StartsWith("$($artifactsRoot.TrimEnd('\'))\", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Sprint 8A manual-authority self-test target escaped the repository artifacts root."
    }
    [IO.Directory]::CreateDirectory((Join-Path $evidence "attempts")) | Out-Null
    try {
        $receipt = [pscustomobject][ordered]@{
            schema_version = 1; sprint = "sprint-8a"; phase = "uat"; attempt = 1
            authoritative = $false; state = "executing"; stage = "manual"
            source_verification_state = "verified"
            candidate_fingerprint = "e" * 64; environment_fingerprint = "f" * 64
            prerequisite_receipts = @(
                [pscustomobject]@{ path = "preflight"; sha256 = "1" * 64 },
                [pscustomobject]@{ path = "candidate"; sha256 = "2" * 64 },
                [pscustomobject]@{ path = "sit"; sha256 = "3" * 64 }
            )
            manual_scenarios_pending = @(Get-Sprint8AManualUatScenarioNames)
            scripted_completed_at = "2026-01-01T00:00:00Z"
        }
        $attemptPath = Join-Path $evidence "attempts/uat-1.json"
        $checkpointPath = Join-Path $evidence "attempts/uat-1-manual-checkpoint.json"
        Publish-Sprint7AEvidence -Document $receipt -OutputPath $attemptPath | Out-Null
        Publish-Sprint7AEvidence -Document $receipt -OutputPath $checkpointPath | Out-Null
        $attemptRelative = "$relativeRoot/attempts/uat-1.json"
        $checkpointRelative = "$relativeRoot/attempts/uat-1-manual-checkpoint.json"
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @(
                [pscustomobject]@{ path = $attemptRelative; phase = "uat-attempt"; authoritative = $false; status = "awaiting-manual" },
                [pscustomobject]@{ path = $checkpointRelative; phase = "uat-checkpoint"; authoritative = $false; status = "awaiting-manual" }
            )
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath "$relativeRoot/evidence-manifest.json" | Out-Null
        $checkpoint = [pscustomobject][ordered]@{
            reference = [pscustomobject][ordered]@{
                path = $checkpointRelative
                sha256 = Assert-Sprint8AReceiptSidecar -Path $checkpointPath
            }
            receipt = $receipt
        }
        Assert-Sprint8AManualUatAttemptOpen `
            -Attempt 1 `
            -Checkpoint $checkpoint `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence | Out-Null
        $scenario = "UAT-8A-01"
        $scenarioStarted = [DateTimeOffset]::Parse("2026-01-01T00:00:01Z")
        $lease = Open-Sprint8AManualUatScenarioLease `
            -Scenario $scenario `
            -Attempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -StartedAt $scenarioStarted `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence
        $concurrentRejected = $false
        try {
            $concurrent = Open-Sprint8AValidationAttemptLock -Path (Join-Path $evidence "validation-attempt.lock")
            $concurrent.Dispose()
        } catch { $concurrentRejected = $true }
        if (-not $concurrentRejected) {
            $lease.stream.Dispose()
            throw "Sprint 8A manual-lease self-test allowed concurrent finalization."
        }
        $openLeaseRejected = $false
        try {
            Assert-Sprint8ANoOpenManualUatLeases -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
        } catch { $openLeaseRejected = $true }
        if (-not $openLeaseRejected) {
            $lease.stream.Dispose()
            throw "Sprint 8A manual-lease self-test did not detect an open execution lease."
        }
        $lease.stream.Dispose()
        $lease = Open-Sprint8AManualUatScenarioLease `
            -Scenario $scenario `
            -Attempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -StartedAt $scenarioStarted `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Resume
        $scenarioContract = Get-Sprint8AManualUatScenarioContract -Scenario $scenario
        $scenarioEvidence = [Collections.Generic.List[object]]::new()
        foreach ($step in @($scenarioContract.steps)) {
            $rawPath = Join-Path $evidence "uat/attempt-1/raw/$($scenario.ToLowerInvariant())-step-$([int]$step.step).txt"
            [IO.Directory]::CreateDirectory((Split-Path -Parent $rawPath)) | Out-Null
            [IO.File]::WriteAllText($rawPath, "observed step $([int]$step.step)`n", [Text.UTF8Encoding]::new($false))
            $scenarioEvidence.Add([pscustomobject][ordered]@{
                step = [int]$step.step
                path = [IO.Path]::GetRelativePath($repository, $rawPath).Replace("\", "/")
                sha256 = Get-Sprint8AFileSha256 -Path $rawPath
            })
        }
        $cleanupPath = Join-Path $evidence "uat/attempt-1/raw/$($scenario.ToLowerInvariant())-cleanup.txt"
        [IO.File]::WriteAllText($cleanupPath, "canonical topology restored`n", [Text.UTF8Encoding]::new($false))
        $cleanupEvidence = @([pscustomobject][ordered]@{
            kind = "canonical-restoration"
            path = [IO.Path]::GetRelativePath($repository, $cleanupPath).Replace("\", "/")
            sha256 = Get-Sprint8AFileSha256 -Path $cleanupPath
        })
        $passingActions = @($scenarioContract.steps | ForEach-Object {
            [pscustomobject][ordered]@{
                step = [int]$_.step; action = [string]$_.action
                expected_result = [string]$_.expected_result
                actual_result = "observed step $([int]$_.step)"; state = "passed"
            }
        })
        $publicationArguments = @{
            Scenario = $scenario
            Attempt = 1
            CandidateFingerprint = "e" * 64
            EnvironmentFingerprint = "f" * 64
            State = "passed"
            Authoritative = $true
            Diagnostic = $false
            AssertionsStarted = $true
            Role = [string]$scenarioContract.role
            StartingState = "canonical"
            Actions = $passingActions
            ExpectedResult = "Every canonical step expectation in $scenario is satisfied."
            ActualResult = "All canonical observations passed."
            Evidence = @($scenarioEvidence)
            CleanupEvidence = $cleanupEvidence
            StartedAt = $scenarioStarted
            EndedAt = $scenarioStarted.AddSeconds(1)
            RepositoryRoot = $repository
            EvidenceRoot = $evidence
            OutputPath = "$relativeRoot/uat/attempt-1/manual/$($scenario.ToLowerInvariant()).json"
            ExecutionLease = $lease
        }
        $resumedPassRejected = $false
        try {
            Publish-Sprint8AManualUatReceipt @publicationArguments | Out-Null
        } catch {
            if (-not $_.Exception.Message.Contains("cannot publish an authoritative pass after execution resume")) { throw }
            $resumedPassRejected = $true
        }
        if (-not $resumedPassRejected) {
            throw "Sprint 8A manual-lease self-test accepted an authoritative pass after resume."
        }
        $downgradedLease = $lease | Select-Object *
        $downgradedLease.resumed = $false
        $downgradedLease.resume = $null
        $publicationArguments.ExecutionLease = $downgradedLease
        $resumeDowngradeRejected = $false
        try {
            Publish-Sprint8AManualUatReceipt @publicationArguments | Out-Null
        } catch {
            if (-not $_.Exception.Message.Contains("non-resumed execution has inconsistent process lineage")) { throw }
            $resumeDowngradeRejected = $true
        }
        $publicationArguments.ExecutionLease = $lease
        if (-not $resumeDowngradeRejected) {
            throw "Sprint 8A manual-lease self-test allowed a retained resume marker to be downgraded."
        }
        $preparedPath = Join-Path $evidence "uat/attempt-1/manual-leases/$($scenario.ToLowerInvariant())-publication-prepared.json"
        if (Test-Path -LiteralPath $preparedPath) {
            throw "Sprint 8A manual-lease self-test prepared publication for a forbidden resumed pass."
        }
        $failedActions = @($scenarioContract.steps | ForEach-Object {
                [pscustomobject][ordered]@{
                    step = [int]$_.step; action = [string]$_.action
                    expected_result = [string]$_.expected_result
                    actual_result = "observed step $([int]$_.step)"
                    state = if ([int]$_.step -eq 1) { "failed" } else { "passed" }
                }
            })
        $manifestPath = Join-Path $evidence "evidence-manifest.json"
        $manifestBeforePublication = Get-Content -LiteralPath $manifestPath -Raw
        $manifestSidecarBeforePublication = Get-Content -LiteralPath "$manifestPath.sha256" -Raw
        $publicationArguments.State = "failed"
        $publicationArguments.Actions = $failedActions
        $publicationArguments.ActualResult = "The resumed scenario retained an observed product failure."
        $publicationArguments.Classification = "product"
        $publicationArguments.FailureMessage = "Resumed scenario cannot establish a fresh authoritative pass."
        $manualReference = Publish-Sprint8AManualUatReceipt @publicationArguments
        if (-not (Test-Path -LiteralPath $preparedPath -PathType Leaf)) {
            throw "Sprint 8A manual-lease self-test omitted the immutable prepared publication checkpoint."
        }
        $resumePath = Join-Path $evidence "uat/attempt-1/manual-leases/$($scenario.ToLowerInvariant())-resume.json"
        $resumeDocument = Get-Content -LiteralPath $resumePath -Raw | ConvertFrom-Json
        if ([int]$resumeDocument.original_process_id -ne $PID -or
            [int]$resumeDocument.current_process_id -ne $PID -or
            [string]$resumeDocument.lease.path -cne [string]$lease.lease.path) {
            throw "Sprint 8A manual-lease self-test did not retain exact original/current process lineage."
        }
        $completionPath = Join-Path $evidence "uat/attempt-1/manual-leases/$($scenario.ToLowerInvariant())-complete.json"
        $manualReceiptPath = Join-Path $repository ([string]$manualReference.path)
        foreach ($publishedPath in @($manualReceiptPath, "$manualReceiptPath.sha256", $completionPath, "$completionPath.sha256")) {
            Remove-Item -LiteralPath $publishedPath -Force
        }
        [IO.File]::WriteAllText($manifestPath, $manifestBeforePublication, [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText("$manifestPath.sha256", $manifestSidecarBeforePublication, [Text.UTF8Encoding]::new($false))
        $repairedPublications = @(Repair-Sprint8AManualUatPreparedPublications `
            -Attempt 1 `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence)
        $repairedPublication = @($repairedPublications | Where-Object scenario -CEQ $scenario)
        if ($repairedPublication.Count -ne 1) {
            throw "Sprint 8A manual-lease self-test did not idempotently repair one prepared human result."
        }
        $manualReference = $repairedPublication[0].receipt
        Assert-Sprint8ANoOpenManualUatLeases -Attempt 1 -RepositoryRoot $repository -EvidenceRoot $evidence
        $manualReceipt = Get-Content -LiteralPath $manualReceiptPath -Raw | ConvertFrom-Json
        Assert-Sprint8AManualUatExecutionLeasePair `
            -Receipt $manualReceipt `
            -ReceiptReference $manualReference `
            -ExpectedScenario $scenario `
            -ExpectedAttempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence | Out-Null
        $blockedReceipt = $manualReceipt | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        $blockedReceipt.state = "blocked"
        $blockedReceipt.assertions_started = $false
        $blockedReceipt.assertions_started_at = $null
        $blockedReceipt.classification = "product-decision"
        $blockedReceipt.classification_source = "manual_operator"
        $blockedReceipt.failure_message = $null
        $blockedReceipt.blocked_reason = "A product decision is required after the resumed execution."
        foreach ($action in @($blockedReceipt.actions)) { $action.state = "blocked" }
        Assert-Sprint8AManualUatReceipt `
            -Receipt $blockedReceipt `
            -ExpectedScenario $scenario `
            -ExpectedAttempt 1 `
            -CandidateFingerprint ("e" * 64) `
            -EnvironmentFingerprint ("f" * 64) | Out-Null
        $terminal = $receipt | ConvertTo-Json -Depth 20 | ConvertFrom-Json
        $terminal.state = "passed"
        $terminal.stage = "result-committed"
        Publish-Sprint7AEvidence -Document $terminal -OutputPath $attemptPath -Overwrite | Out-Null
        $terminalRejected = $false
        try {
            Assert-Sprint8AManualUatAttemptOpen `
                -Attempt 1 `
                -Checkpoint $checkpoint `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence | Out-Null
        } catch { $terminalRejected = $true }
        if (-not $terminalRejected) {
            throw "Sprint 8A manual-authority self-test accepted a terminal mutable attempt."
        }
        Publish-Sprint7AEvidence -Document $receipt -OutputPath $attemptPath -Overwrite | Out-Null
        $entries = Get-Sprint8AEvidenceFileManifestEntries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -Overrides @([pscustomobject]@{
                path = $attemptRelative; phase = "uat-attempt"; authoritative = $false; status = "awaiting-manual"
            })
        Publish-Sprint8AEvidenceManifest `
            -Entries $entries `
            -RepositoryRoot $repository `
            -EvidenceRoot $evidence `
            -OutputPath "$relativeRoot/evidence-manifest.json" `
            -Merge | Out-Null
        [IO.File]::WriteAllText((Join-Path $evidence "uat-result.json"), "publication-started", [Text.UTF8Encoding]::new($false))
        $canonicalRejected = $false
        try {
            Assert-Sprint8AManualUatAttemptOpen `
                -Attempt 1 `
                -Checkpoint $checkpoint `
                -RepositoryRoot $repository `
                -EvidenceRoot $evidence | Out-Null
        } catch { $canonicalRejected = $true }
        if (-not $canonicalRejected) {
            throw "Sprint 8A manual-authority self-test accepted publication after the canonical result boundary."
        }
    } finally {
        if (Test-Path -LiteralPath $evidence -PathType Container) {
            Remove-Item -LiteralPath $evidence -Recurse -Force
        }
    }
}

function Test-Sprint8ALifecycleChain {
    $offsetlessRejected = $false
    try {
        ConvertTo-Sprint8ADateTimeOffset `
            -Value "2026-01-01T00:00:00" `
            -Label "lifecycle offsetless timestamp self-test" | Out-Null
    } catch {
        if ($_.Exception.Message -notmatch "has no UTC offset") { throw }
        $offsetlessRejected = $true
    }
    if (-not $offsetlessRejected) {
        throw "Sprint 8A lifecycle self-test accepted an offsetless timestamp."
    }
    Test-Sprint8ALifecycleExclusiveLock
    Test-Sprint8AEvidenceManifestContract
    Test-Sprint8AManualUatAttemptAuthority
    $source = [pscustomobject][ordered]@{
        commit = "a" * 40
        tree = "b" * 40
        dirty = $false
        branch = "codex/sprint-8a"
        acceptance_inventory_sha256 = "c" * 64
        deployment_inputs_sha256 = "d" * 64
    }
    Assert-Sprint8ASourceIdentityObject -Source $source -RequireClean | Out-Null
    $sit = @(Get-Sprint8ASitLaneNames | ForEach-Object {
        [pscustomobject]@{
            name = $_; state = "passed"; assertions_started = $true
            classification = $null; failure_message = $null; blocked_reason = $null
            command = "self-test $_"; exit_status = 0
            started_at = "2026-01-01T00:00:00Z"; ended_at = "2026-01-01T00:00:01Z"
            duration_ms = 1000L
            evidence = @([pscustomobject]@{ path = "artifacts/sprint-8a-closeout/self-test.log"; sha256 = "a" * 64 })
        }
    })
    Assert-Sprint8AExactTerminalIdentities -Results $sit -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "SIT self-test" | Out-Null
    $sitRoundTrip = $sit | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    Assert-Sprint8AExactTerminalIdentities -Results @($sitRoundTrip) -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "SIT JSON round-trip self-test" | Out-Null
    $manualContract = Get-Sprint8AManualUatScenarioContract -Scenario "UAT-8A-01"
    $manual = [pscustomobject]@{
        schema_version = 1
        sprint = "sprint-8a"; phase = "uat-manual-scenario"; attempt = 1; authoritative = $true; diagnostic = $false
        scenario = "UAT-8A-01"; state = "passed"; candidate_fingerprint = "e" * 64
        environment_fingerprint = "f" * 64; assertions_started = $true; role = $manualContract.role
        assertions_started_at = "2026-01-01T00:00:00Z"
        started_at = "2026-01-01T00:00:00Z"; ended_at = "2026-01-01T00:00:01Z"
        duration_ms = 1000L
        starting_state = "canonical"
        actions = @($manualContract.steps | ForEach-Object {
            [pscustomobject]@{
                step = [int]$_.step
                action = [string]$_.action
                expected_result = [string]$_.expected_result
                actual_result = "observed step $($_.step)"
                state = "passed"
            }
        })
        expected_result = "Every canonical step expectation in UAT-8A-01 is satisfied."
        actual_result = "observed"
        evidence = @($manualContract.steps | ForEach-Object {
            [pscustomobject]@{ step = [int]$_.step; path = "evidence-$($_.step)"; sha256 = "a" * 64 }
        })
        classification = $null; classification_source = $null; failure_message = $null; blocked_reason = $null
        scenario_contract = $manualContract.document
        start_checkpoint = [pscustomobject]@{ path = "checkpoint"; sha256 = "c" * 64 }
        execution_lease = [pscustomobject]@{
            path = "artifacts/sprint-8a-closeout/uat/attempt-1/manual-leases/uat-8a-01-start.json"
            sha256 = "d" * 64
        }
        resumed = $false
        execution_resume = $null
        cleanup_restoration = [pscustomobject]@{
            required = $true; result = "canonical_topology_verified"
            evidence = @([pscustomobject]@{ kind = "canonical-restoration"; path = "cleanup"; sha256 = "b" * 64 })
        }
    }
    Assert-Sprint8AManualUatReceipt -Receipt $manual -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $manualRoundTrip = $manual | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $authorityRejected = $false
    try {
        $manualRoundTrip.authoritative = "true"
        Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    } catch { $authorityRejected = $true }
    if (-not $authorityRejected) { throw "Sprint 8A lifecycle-chain self-test accepted string manual authority." }
    $manualRoundTrip.authoritative = $false
    $manualRoundTrip.diagnostic = $true
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $manualRoundTrip.state = "failed"
    $manualRoundTrip.authoritative = $true
    $manualRoundTrip.diagnostic = $false
    $manualRoundTrip.classification = "product"
    $manualRoundTrip.classification_source = "manual_operator"
    $manualRoundTrip.failure_message = "self-test product failure"
    $manualRoundTrip.actions[0].state = "failed"
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $manualRoundTrip.state = "blocked"
    $manualRoundTrip.authoritative = $false
    $manualRoundTrip.diagnostic = $true
    $manualRoundTrip.assertions_started = $false
    $manualRoundTrip.assertions_started_at = $null
    $manualRoundTrip.classification = $null
    $manualRoundTrip.classification_source = $null
    $manualRoundTrip.failure_message = $null
    $manualRoundTrip.blocked_reason = "self-test prerequisite failed"
    $manualRoundTrip.actions[0].state = "blocked"
    Assert-Sprint8AManualUatReceipt -Receipt $manualRoundTrip -ExpectedScenario "UAT-8A-01" -ExpectedAttempt 1 -CandidateFingerprint ("e" * 64) -EnvironmentFingerprint ("f" * 64) | Out-Null
    $rejected = $false
    try {
        Assert-Sprint8AExactTerminalIdentities -Results @($sit | Select-Object -First 3) -ExpectedNames (Get-Sprint8ASitLaneNames) -Label "SIT self-test" | Out-Null
    } catch { $rejected = $true }
    if (-not $rejected) { throw "Sprint 8A lifecycle-chain self-test accepted an incomplete SIT inventory." }
    "Sprint 8A lifecycle-chain schema and identity self-test passed."
}

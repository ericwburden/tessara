[CmdletBinding()]
param(
    [ValidateRange(1, 9999)][int]$Attempt,
    [string]$ReadinessReceipt = "artifacts/sprint-8a-closeout/validation-readiness-result.json",
    [string]$RehearsalReceipt = "artifacts/sprint-8a-closeout/candidate-rehearsal-result.json",
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [string]$ExpectedBranch = "codex/sprint-8a",
    [string]$HandoffUrl = "http://127.0.0.1:8088",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($PSVersionTable.PSEdition -cne "Core" -or $PSVersionTable.PSVersion.Major -lt 7) {
    throw "Sprint 8A validation preflight requires PowerShell 7 or newer."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$script:PreflightAllowedClassifications = @(
    "preflight/setup",
    "product",
    "harness",
    "environment",
    "flaky",
    "evidence-finalization",
    "product-decision"
)

function Get-Sprint8APreflightDeclarations {
    @(
        [pscustomobject][ordered]@{
            name = "receipt-chain"
            depends_on = @()
            command = "acquire exclusive attempt lock; authenticate exact passing Readiness, Rehearsal, environment, validation-state, and correction-consumption chain"
            failure_classification = "preflight/setup"
        }
        [pscustomobject][ordered]@{
            name = "repository-scope"
            depends_on = @()
            command = "reconfirm repository instructions, worktree, branch, Sprint 8A scope, and one implementation commit"
            failure_classification = "preflight/setup"
        }
        [pscustomobject][ordered]@{
            name = "clean-source"
            depends_on = @("receipt-chain", "repository-scope")
            command = "recompute and exactly compare clean commit, tree, acceptance inventory, and deployment inputs"
            failure_classification = "preflight/setup"
        }
        [pscustomobject][ordered]@{
            name = "acceptance-traceability"
            depends_on = @("receipt-chain", "repository-scope")
            command = "reconcile Sprint 8A clauses with 75 browser identities, smoke, and UAT-8A-01 through UAT-8A-08"
            failure_classification = "product"
        }
        [pscustomobject][ordered]@{
            name = "environment-contract"
            depends_on = @("receipt-chain", "repository-scope")
            command = "reconfirm secret-free environment fingerprint, tools, ports, Compose profile, handoff URL, reset authorization, and path mode"
            failure_classification = "environment"
        }
        [pscustomobject][ordered]@{
            name = "database-contract"
            depends_on = @("environment-contract")
            command = "reconfirm six distinct disposable database identities, authenticated reachability, and migration ledgers"
            failure_classification = "environment"
        }
        [pscustomobject][ordered]@{
            name = "deployment-contract"
            depends_on = @("clean-source", "environment-contract")
            command = "audit Compose, provenance labels, non-building lifecycle commands, active slot, and canonical restoration"
            failure_classification = "product"
        }
        [pscustomobject][ordered]@{
            name = "downstream-command-contract"
            depends_on = @("repository-scope")
            command = "parse exact static, Rust, Playwright, deployed SIT command sets and staged formal-UAT interface without executing them"
            failure_classification = "harness"
        }
        [pscustomobject][ordered]@{
            name = "evidence-path-contract"
            depends_on = @("repository-scope")
            command = "validate canonical contained paths, immutable output collisions, and the sole in-root legacy authorization exception"
            failure_classification = "harness"
        }
        [pscustomobject][ordered]@{
            name = "evidence-inventory"
            depends_on = @("acceptance-traceability", "deployment-contract", "downstream-command-contract", "evidence-path-contract")
            command = "declare complete existing, SIT, UAT, conditional convergence, closeout, and manifest evidence before freeze"
            failure_classification = "evidence-finalization"
        }
    )
}

function Assert-Sprint8APreflightGraph {
    param([Parameter(Mandatory)][object[]]$Checks)

    $expected = [ordered]@{
        "receipt-chain" = @()
        "repository-scope" = @()
        "clean-source" = @("receipt-chain", "repository-scope")
        "acceptance-traceability" = @("receipt-chain", "repository-scope")
        "environment-contract" = @("receipt-chain", "repository-scope")
        "database-contract" = @("environment-contract")
        "deployment-contract" = @("clean-source", "environment-contract")
        "downstream-command-contract" = @("repository-scope")
        "evidence-path-contract" = @("repository-scope")
        "evidence-inventory" = @("acceptance-traceability", "deployment-contract", "downstream-command-contract", "evidence-path-contract")
    }
    $names = @($Checks | ForEach-Object { [string]$_.name })
    if ($Checks.Count -ne 10 -or
        @($names | Sort-Object -Unique).Count -ne 10 -or
        ($names -join "`n") -cne (@($expected.Keys) -join "`n")) {
        throw "Sprint 8A preflight must declare the exact ordered ten-check inventory."
    }
    foreach ($check in $Checks) {
        $expectedDependencies = @($expected[[string]$check.name])
        if ((@($check.depends_on) -join "`n") -cne ($expectedDependencies -join "`n") -or
            [string]::IsNullOrWhiteSpace([string]$check.command) -or
            $script:PreflightAllowedClassifications -cnotcontains [string]$check.failure_classification) {
            throw "Sprint 8A preflight check '$($check.name)' has a stale dependency, command, or classification contract."
        }
    }
}

function Get-Sprint8APreflightStringSha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        ([BitConverter]::ToString(
            $algorithm.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))
        ) -replace "-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Get-Sprint8APreflightFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Cannot hash missing Sprint 8A preflight evidence '$fullPath'."
    }
    (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Write-Sprint8APreflightJsonReceipt {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$Path,
        [switch]$Overwrite
    )

    $fullPath = [IO.Path]::GetFullPath($Path)
    $sidecarPath = "$fullPath.sha256"
    if (-not $Overwrite -and
        ((Test-Path -LiteralPath $fullPath) -or (Test-Path -LiteralPath $sidecarPath))) {
        throw "Retained Sprint 8A preflight evidence already exists: $fullPath"
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $fullPath)) | Out-Null
    $temporaryPath = "$fullPath.$([guid]::NewGuid().ToString('N')).tmp"
    $temporarySidecar = "$temporaryPath.sha256"
    try {
        $json = ($Document | ConvertTo-Json -Depth 100) + "`n"
        [IO.File]::WriteAllText($temporaryPath, $json, [Text.UTF8Encoding]::new($false))
        $sha256 = Get-Sprint8APreflightFileSha256 -Path $temporaryPath
        [IO.File]::WriteAllText($temporarySidecar, "$sha256`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::Move($temporaryPath, $fullPath, [bool]$Overwrite)
        [IO.File]::Move($temporarySidecar, $sidecarPath, [bool]$Overwrite)
        if ((Get-Sprint8APreflightFileSha256 -Path $fullPath) -cne $sha256) {
            throw "Sprint 8A preflight evidence changed during publication: $fullPath"
        }
        $sha256
    } finally {
        Remove-Item -LiteralPath $temporaryPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temporarySidecar -Force -ErrorAction SilentlyContinue
    }
}

function Open-Sprint8AValidationAttemptLock {
    param([Parameter(Mandatory)][string]$Path)

    [IO.Directory]::CreateDirectory((Split-Path -Parent $Path)) | Out-Null
    [IO.File]::Open(
        [IO.Path]::GetFullPath($Path),
        [IO.FileMode]::OpenOrCreate,
        [IO.FileAccess]::ReadWrite,
        [IO.FileShare]::None
    )
}

function Assert-Sprint8APreflightSidecar {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    $sidecarPath = "$fullPath.sha256"
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        throw "Sprint 8A preflight receipt or sidecar is missing: $fullPath"
    }
    $actual = Get-Sprint8APreflightFileSha256 -Path $fullPath
    $expected = (Get-Content -LiteralPath $sidecarPath -Raw).Trim()
    if ($expected -notmatch '^[0-9a-f]{64}$' -or $expected -cne $actual) {
        throw "Sprint 8A preflight receipt sidecar is stale: $fullPath"
    }
    $actual
}

function Test-Sprint8APreflightContainedPath {
    param(
        [Parameter(Mandatory)][string]$Parent,
        [Parameter(Mandatory)][string]$Child
    )

    $relative = [IO.Path]::GetRelativePath(
        [IO.Path]::GetFullPath($Parent),
        [IO.Path]::GetFullPath($Child)
    )
    -not [IO.Path]::IsPathRooted($relative) -and
        $relative -cne ".." -and
        -not $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -and
        -not $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)
}

function Resolve-Sprint8APreflightEvidencePath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRootPath,
        [Parameter(Mandatory)][string]$Path,
        [switch]$AllowInRootAbsolute
    )

    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = [IO.Path]::GetFullPath($EvidenceRootPath)
    if (-not (Test-Sprint8APreflightContainedPath -Parent $repository -Child $evidence)) {
        throw "Sprint 8A evidence root must remain inside the repository."
    }
    if ([IO.Path]::IsPathRooted($Path) -and -not $AllowInRootAbsolute) {
        throw "Sprint 8A preflight evidence inputs must be repository-relative."
    }
    $candidate = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repository $Path))
    }
    if (-not (Test-Sprint8APreflightContainedPath -Parent $repository -Child $candidate) -or
        -not (Test-Sprint8APreflightContainedPath -Parent $evidence -Child $candidate)) {
        throw "Sprint 8A preflight evidence path escapes its canonical evidence root."
    }
    [pscustomobject][ordered]@{
        full_path = $candidate
        path = [IO.Path]::GetRelativePath($repository, $candidate).Replace("\", "/")
    }
}

function Get-Sprint8APreflightReceiptReference {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$AllowInRootAbsolute
    )

    $resolved = Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath `
        -Path $Path `
        -AllowInRootAbsolute:$AllowInRootAbsolute
    $sha256 = Assert-Sprint8APreflightSidecar -Path ([string]$resolved.full_path)
    [pscustomobject][ordered]@{
        path = [string]$resolved.path
        sha256 = $sha256
        full_path = [string]$resolved.full_path
        document = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
    }
}

function Resolve-Sprint8APreflightClassification {
    param(
        [Parameter(Mandatory)][string]$Default,
        [Parameter(Mandatory)]$ErrorRecord
    )

    $structured = [string]$ErrorRecord.Exception.Data["Sprint8AClassification"]
    if (-not [string]::IsNullOrWhiteSpace($structured)) {
        if ($script:PreflightAllowedClassifications -cnotcontains $structured) {
            return [pscustomobject][ordered]@{
                classification = "harness"
                source = "invalid_structured_classification"
            }
        }
        return [pscustomobject][ordered]@{
            classification = $structured
            source = "structured_exception"
        }
    }
    if ($script:PreflightAllowedClassifications -cnotcontains $Default) {
        throw "Preflight declaration uses unsupported classification '$Default'."
    }
    [pscustomobject][ordered]@{
        classification = $Default
        source = "declared_check_contract"
    }
}

function ConvertTo-Sprint8APreflightCanonicalJson {
    param([AllowNull()]$InputObject)
    $InputObject | ConvertTo-Json -Depth 100 -Compress
}

function Assert-Sprint8APreflightRehearsalAttemptReceipt {
    param(
        [Parameter(Mandatory)]$RehearsalResult,
        [Parameter(Mandatory)]$ValidationState,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRootPath
    )

    if (($RehearsalResult.attempt -isnot [int] -and $RehearsalResult.attempt -isnot [long]) -or
        [int]$RehearsalResult.attempt -lt 1 -or
        $RehearsalResult.PSObject.Properties.Name -notcontains "attempt_receipt" -or
        $null -eq $RehearsalResult.attempt_receipt -or
        [string]$RehearsalResult.attempt_receipt.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Passing Candidate Rehearsal does not name one immutable attempt receipt."
    }

    $attempt = [int]$RehearsalResult.attempt
    $expectedFullPath = Join-Path $EvidenceRootPath "attempts/candidate-rehearsal-$attempt-attempt.json"
    $expectedPath = [IO.Path]::GetRelativePath(
        [IO.Path]::GetFullPath($RepositoryRoot),
        [IO.Path]::GetFullPath($expectedFullPath)
    ).Replace("\", "/")
    if ([string]$RehearsalResult.attempt_receipt.path -cne $expectedPath) {
        throw "Passing Candidate Rehearsal attempt receipt does not use exact path '$expectedPath'."
    }

    $resolved = Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $RepositoryRoot `
        -EvidenceRootPath $EvidenceRootPath `
        -Path ([string]$RehearsalResult.attempt_receipt.path)
    $sha256 = Assert-Sprint8APreflightSidecar -Path ([string]$resolved.full_path)
    if ($sha256 -cne [string]$RehearsalResult.attempt_receipt.sha256) {
        throw "Passing Candidate Rehearsal attempt receipt differs from its embedded SHA-256 reference."
    }
    $document = Get-Content -LiteralPath ([string]$resolved.full_path) -Raw | ConvertFrom-Json
    if (($document.schema_version -isnot [int] -and $document.schema_version -isnot [long]) -or
        [int]$document.schema_version -ne 2 -or
        [string]$document.sprint -cne "sprint-8a" -or
        [string]$document.phase -cne "candidate-rehearsal" -or
        [int]$document.attempt -ne $attempt -or
        $document.authoritative -isnot [bool] -or [bool]$document.authoritative -or
        [string]$document.state -cne "passed" -or
        [string]$RehearsalResult.state -cne "passed" -or
        (ConvertTo-Sprint8APreflightCanonicalJson $document.mutable_source_identity) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $RehearsalResult.mutable_source_identity) -or
        [string]$document.environment_fingerprint -cne [string]$RehearsalResult.environment_fingerprint -or
        [int]$ValidationState.rehearsal.attempt -ne $attempt -or
        [string]$ValidationState.rehearsal.state -cne "passed") {
        throw "Candidate Rehearsal attempt, result, and validation-state identities are not the same passing attempt."
    }
    if (@($document.prerequisite_receipts).Count -ne 1 -or
        @($RehearsalResult.prerequisite_receipts).Count -ne 1 -or
        (ConvertTo-Sprint8APreflightCanonicalJson @($document.prerequisite_receipts)[0]) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson @($RehearsalResult.prerequisite_receipts)[0])) {
        throw "Candidate Rehearsal attempt and result do not retain the same immutable Readiness prerequisite."
    }

    foreach ($entry in @(
        [pscustomobject]@{ label = "attempt"; document = $document },
        [pscustomobject]@{ label = "result"; document = $RehearsalResult },
        [pscustomobject]@{ label = "validation-state"; document = $ValidationState }
    )) {
        if ($entry.document.PSObject.Properties.Name -notcontains "correction_lineage") {
            throw "Candidate Rehearsal $($entry.label) omits its correction_lineage binding."
        }
    }
    $attemptLineage = $document.PSObject.Properties["correction_lineage"].Value
    $resultLineage = $RehearsalResult.PSObject.Properties["correction_lineage"].Value
    $stateLineage = $ValidationState.PSObject.Properties["correction_lineage"].Value
    $allNull = $null -eq $attemptLineage -and $null -eq $resultLineage -and $null -eq $stateLineage
    if (-not $allNull) {
        if ($null -eq $attemptLineage -or $null -eq $resultLineage -or $null -eq $stateLineage -or
            (ConvertTo-Sprint8APreflightCanonicalJson $attemptLineage) -cne
                (ConvertTo-Sprint8APreflightCanonicalJson $resultLineage) -or
            (ConvertTo-Sprint8APreflightCanonicalJson $attemptLineage) -cne
                (ConvertTo-Sprint8APreflightCanonicalJson $stateLineage)) {
            throw "Candidate Rehearsal attempt, result, and validation-state correction_lineage values differ."
        }
    }

    [pscustomobject][ordered]@{
        path = [string]$resolved.path
        sha256 = $sha256
        full_path = [string]$resolved.full_path
        document = $document
    }
}

function Get-Sprint8AScriptParameterNames {
    param([Parameter(Mandatory)][string]$Path)

    $tokens = $null
    $parseErrors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile(
        [IO.Path]::GetFullPath($Path),
        [ref]$tokens,
        [ref]$parseErrors
    )
    if (@($parseErrors).Count -ne 0 -or $null -eq $ast.ParamBlock) {
        throw "Sprint 8A downstream runner does not parse or has no parameter contract: $Path"
    }
    @($ast.ParamBlock.Parameters | ForEach-Object { [string]$_.Name.VariablePath.UserPath })
}

function Get-Sprint8APlannedEvidenceInventory {
    param([Parameter(Mandatory)][int]$PreflightAttempt)

    $required = @(
        [pscustomobject]@{ path = "validation-readiness-result.json"; phase = "readiness"; condition = "always" },
        [pscustomobject]@{ path = "candidate-rehearsal-result.json"; phase = "rehearsal"; condition = "always" },
        [pscustomobject]@{ path = "attempts/preflight-$PreflightAttempt.json"; phase = "preflight"; condition = "always" },
        [pscustomobject]@{ path = "preflight-result.json"; phase = "preflight"; condition = "always" },
        [pscustomobject]@{ path = "candidate.json"; phase = "candidate"; condition = "always" },
        [pscustomobject]@{ path = "attempts/sit-{attempt}.json"; phase = "sit"; condition = "always" },
        [pscustomobject]@{ path = "attempts/sit-{attempt}-start.json"; phase = "sit"; condition = "always" },
        [pscustomobject]@{ path = "sit/attempts/{lane}-{attempt}.json"; phase = "sit"; condition = "all four lanes" },
        [pscustomobject]@{ path = "sit-result.json"; phase = "sit"; condition = "always" },
        [pscustomobject]@{ path = "attempts/uat-{attempt}.json"; phase = "uat"; condition = "always" },
        [pscustomobject]@{ path = "attempts/uat-{attempt}-manual-checkpoint.json"; phase = "uat"; condition = "always" },
        [pscustomobject]@{ path = "uat/attempt-{attempt}/finalizations/run-{n}/finalization-completion-checkpoint.json"; phase = "uat"; condition = "always" },
        [pscustomobject]@{ path = "uat/attempt-{attempt}/finalizations/run-{n}/result-commit.json"; phase = "uat"; condition = "always" },
        [pscustomobject]@{ path = "uat/attempt-{attempt}/finalizations/run-{n}/publication-retry-checkpoint.json"; phase = "uat"; condition = "conditional" },
        [pscustomobject]@{ path = "uat/attempt-{attempt}/publication-retries/run-{n}/result-commit.json"; phase = "uat"; condition = "conditional" },
        [pscustomobject]@{ path = "uat-result.json"; phase = "uat"; condition = "always" },
        [pscustomobject]@{ path = "uat-defect-harvest.json"; phase = "post-sit-convergence"; condition = "conditional" },
        [pscustomobject]@{ path = "defect-batch.json"; phase = "post-sit-convergence"; condition = "conditional" },
        [pscustomobject]@{ path = "correction-impact-assessment.json"; phase = "post-sit-convergence"; condition = "conditional" },
        [pscustomobject]@{ path = "focused-repair-validation/attempt-{n}.json"; phase = "post-sit-convergence"; condition = "conditional" },
        [pscustomobject]@{ path = "canonical-restoration.json"; phase = "post-sit-convergence"; condition = "conditional" },
        [pscustomobject]@{ path = "final-certification-entry.json"; phase = "post-sit-convergence"; condition = "conditional" },
        [pscustomobject]@{ path = "evidence-manifest.json"; phase = "manifest"; condition = "always" },
        [pscustomobject]@{ path = "evidence-manifest.json.sha256"; phase = "manifest"; condition = "always" },
        [pscustomobject]@{ path = "closeout-authorization.json"; phase = "closeout"; condition = "always" }
    )
    if ($null -eq (Get-Command Get-Sprint8AManualUatEvidencePlan -CommandType Function -ErrorAction SilentlyContinue)) {
        . (Join-Path $PSScriptRoot "sprint-8a-lifecycle-chain.ps1")
    }
    $evidenceRootFullPath = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else { [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot)) }
    $evidenceRootRelative = [IO.Path]::GetRelativePath($repoRoot, $evidenceRootFullPath).Replace("\", "/").TrimEnd("/")
    $toInventoryPath = {
        param([Parameter(Mandatory)][string]$RepositoryPath)
        $prefix = "$evidenceRootRelative/"
        if (-not $RepositoryPath.StartsWith($prefix, [StringComparison]::Ordinal)) {
            throw "Planned manual UAT evidence path escaped the evidence root: '$RepositoryPath'."
        }
        $RepositoryPath.Substring($prefix.Length).Replace("uat/attempt-1/", "uat/attempt-{attempt}/")
    }
    $manualContract = Get-Sprint8AManualUatContractManifest
    $manualContractSources = @(
        [pscustomobject][ordered]@{ path = [string]$manualContract.path; sha256 = [string]$manualContract.sha256 }
    ) + @($manualContract.document.scenarios | ForEach-Object {
        [pscustomobject][ordered]@{ path = [string]$_.document.path; sha256 = [string]$_.document.sha256 }
    })
    foreach ($scenario in @(Get-Sprint8AManualUatScenarioNames)) {
        $lower = $scenario.ToLowerInvariant()
        foreach ($publication in @(
            [pscustomobject]@{ path = "uat/attempt-{attempt}/manual-leases/$lower-start.json"; condition = "all eight scenarios" },
            [pscustomobject]@{ path = "uat/attempt-{attempt}/manual-leases/$lower-resume.json"; condition = "conditional on explicit resume" },
            [pscustomobject]@{ path = "uat/attempt-{attempt}/manual-leases/$lower-publication-prepared.json"; condition = "all eight scenarios" },
            [pscustomobject]@{ path = "uat/attempt-{attempt}/manual-leases/$lower-complete.json"; condition = "all eight scenarios" },
            [pscustomobject]@{ path = "uat/attempt-{attempt}/manual/$lower.json"; condition = "all eight scenarios" }
        )) {
            $required += [pscustomobject][ordered]@{
                path = [string]$publication.path; phase = "uat-manual"; condition = [string]$publication.condition
                scenario = $scenario
            }
            $required += [pscustomobject][ordered]@{
                path = "$([string]$publication.path).sha256"; phase = "uat-manual"; condition = [string]$publication.condition
                scenario = $scenario
            }
        }
        $plan = Get-Sprint8AManualUatEvidencePlan `
            -Scenario $scenario -Attempt 1 -RepositoryRoot $repoRoot -EvidenceRoot $evidenceRootFullPath
        foreach ($evidence in @($plan.evidence)) {
            $required += [pscustomobject][ordered]@{
                path = & $toInventoryPath -RepositoryPath ([string]$evidence.path)
                phase = "uat-manual-raw"; condition = "all eight scenarios"
                scenario = $scenario; requirement_id = [string]$evidence.requirement_id; kind = [string]$evidence.kind
            }
            if (-not [string]::IsNullOrWhiteSpace([string]$evidence.sidecar_path)) {
                $required += [pscustomobject][ordered]@{
                    path = & $toInventoryPath -RepositoryPath ([string]$evidence.sidecar_path)
                    phase = "uat-manual-raw"; condition = "all eight scenarios"
                    scenario = $scenario; requirement_id = [string]$evidence.requirement_id; kind = "authenticated-json-sidecar"
                }
                foreach ($producerPath in @([string]$evidence.producer_path, [string]$evidence.producer_sidecar_path)) {
                    $required += [pscustomobject][ordered]@{
                        path = & $toInventoryPath -RepositoryPath $producerPath
                        phase = "uat-manual-raw"; condition = "all eight scenarios"
                        scenario = $scenario; requirement_id = [string]$evidence.requirement_id
                        kind = if ($producerPath.EndsWith(".sha256", [StringComparison]::Ordinal)) {
                            "authenticated-producer-sidecar"
                        } else { "authenticated-producer" }
                    }
                }
                foreach ($assertionRaw in @($evidence.assertion_raw_evidence)) {
                    $required += [pscustomobject][ordered]@{
                        path = & $toInventoryPath -RepositoryPath ([string]$assertionRaw.path)
                        phase = "uat-manual-raw"; condition = "all eight scenarios"
                        scenario = $scenario; requirement_id = [string]$evidence.requirement_id
                        kind = "authenticated-assertion-raw"; assertion_id = [string]$assertionRaw.assertion_id
                    }
                }
            }
        }
        $required += [pscustomobject][ordered]@{
            path = & $toInventoryPath -RepositoryPath ([string]$plan.cleanup.path)
            phase = "uat-manual-raw"; condition = "all eight scenarios"
            scenario = $scenario; kind = "canonical-restoration"
        }
    }
    $namespaces = @(
        "readiness/", "rehearsal/", "preflight/", "sit/", "uat/attempt-{attempt}/",
        "uat/attempt-{attempt}/manual/", "uat/attempt-{attempt}/manual-leases/",
        "uat/attempt-{attempt}/raw/",
        "uat/attempt-{attempt}/finalizations/", "uat/attempt-{attempt}/publication-retries/",
        "materialization/", "upgrade-rollback/",
        "focused-repair-validation/"
    )
    [pscustomobject][ordered]@{
        required = $required
        namespaces = $namespaces
        sit_lanes = @("static-and-boundaries", "rust-workspace", "playwright", "deployed-acceptance-smoke")
        manual_uat = @(1..8 | ForEach-Object { "UAT-8A-{0:d2}" -f $_ })
        manual_contract_sources = $manualContractSources
    }
}

function Test-Sprint8APreflightSchedulerContract {
    param([Parameter(Mandatory)][object[]]$Checks)

    $states = @{}
    $executed = [Collections.Generic.List[string]]::new()
    foreach ($check in $Checks) {
        $failedDependencies = @($check.depends_on | Where-Object {
            -not $states.ContainsKey([string]$_) -or $states[[string]$_] -cne "passed"
        })
        if ($failedDependencies.Count -gt 0) {
            $states[[string]$check.name] = "blocked"
            continue
        }
        $executed.Add([string]$check.name)
        $states[[string]$check.name] = if ([string]$check.name -ceq "receipt-chain") { "failed" } else { "passed" }
    }
    $expectedExecuted = @("receipt-chain", "repository-scope", "downstream-command-contract", "evidence-path-contract")
    if ($states.Count -ne 10 -or
        (@($executed) -join "`n") -cne ($expectedExecuted -join "`n") -or
        [string]$states["clean-source"] -cne "blocked" -or
        [string]$states["database-contract"] -cne "blocked" -or
        [string]$states["evidence-inventory"] -cne "blocked") {
        throw "Sprint 8A preflight scheduler does not preserve safe independent checks fail-late."
    }
}

function Test-Sprint8AValidationPreflightRunner {
    $checks = @(Get-Sprint8APreflightDeclarations)
    Assert-Sprint8APreflightGraph -Checks $checks
    Test-Sprint8APreflightSchedulerContract -Checks $checks

    $inventory = Get-Sprint8APlannedEvidenceInventory -PreflightAttempt 7
    $expectedRequiredPaths = @(
        "validation-readiness-result.json",
        "candidate-rehearsal-result.json",
        "attempts/preflight-7.json",
        "preflight-result.json",
        "candidate.json",
        "attempts/sit-{attempt}.json",
        "attempts/sit-{attempt}-start.json",
        "sit/attempts/{lane}-{attempt}.json",
        "sit-result.json",
        "attempts/uat-{attempt}.json",
        "attempts/uat-{attempt}-manual-checkpoint.json",
        "uat/attempt-{attempt}/finalizations/run-{n}/finalization-completion-checkpoint.json",
        "uat/attempt-{attempt}/finalizations/run-{n}/result-commit.json",
        "uat/attempt-{attempt}/finalizations/run-{n}/publication-retry-checkpoint.json",
        "uat/attempt-{attempt}/publication-retries/run-{n}/result-commit.json",
        "uat-result.json",
        "uat-defect-harvest.json",
        "defect-batch.json",
        "correction-impact-assessment.json",
        "focused-repair-validation/attempt-{n}.json",
        "canonical-restoration.json",
        "final-certification-entry.json",
        "evidence-manifest.json",
        "evidence-manifest.json.sha256",
        "closeout-authorization.json"
    )
    $manifestPath = Join-Path $repoRoot "docs/sprints/sprint-8a-uat/scenario-contract.json"
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $expectedManualPaths = [Collections.Generic.List[string]]::new()
    foreach ($scenario in @($manifest.scenarios)) {
        $scenarioId = [string]$scenario.id
        $lower = $scenarioId.ToLowerInvariant()
        foreach ($publication in @(
            "uat/attempt-{attempt}/manual-leases/$lower-start.json",
            "uat/attempt-{attempt}/manual-leases/$lower-resume.json",
            "uat/attempt-{attempt}/manual-leases/$lower-publication-prepared.json",
            "uat/attempt-{attempt}/manual-leases/$lower-complete.json",
            "uat/attempt-{attempt}/manual/$lower.json"
        )) {
            $expectedManualPaths.Add($publication)
            $expectedManualPaths.Add("$publication.sha256")
        }
        foreach ($step in @($scenario.steps)) {
            foreach ($requirement in @($step.evidence_requirements)) {
                $extensions = @($manifest.receipt_contract.evidence_kind_extensions.PSObject.Properties[[string]$requirement.kind].Value)
                if ($extensions.Count -ne 1) {
                    throw "Sprint 8A preflight inventory self-test found a non-exact extension contract."
                }
                $raw = "uat/attempt-{attempt}/raw/$lower/$([string]$requirement.id)$([string]$extensions[0])"
                $expectedManualPaths.Add($raw)
                if ([string]$requirement.kind -ceq "authenticated-json") {
                    $expectedManualPaths.Add("$raw.sha256")
                    $producer = "uat/attempt-{attempt}/raw/$lower/$([string]$requirement.id)-producer.json"
                    $expectedManualPaths.Add($producer)
                    $expectedManualPaths.Add("$producer.sha256")
                    foreach ($assertionId in @($requirement.authenticated_contract.assertion_ids | ForEach-Object { [string]$_ })) {
                        $expectedManualPaths.Add(
                            "uat/attempt-{attempt}/raw/$lower/$([string]$requirement.id)-$assertionId-raw.json"
                        )
                    }
                }
            }
        }
        $expectedManualPaths.Add("uat/attempt-{attempt}/raw/$lower/canonical-restoration.json")
    }
    $expectedRequiredPaths += @($expectedManualPaths)
    $expectedNamespaces = @(
        "readiness/", "rehearsal/", "preflight/", "sit/", "uat/attempt-{attempt}/",
        "uat/attempt-{attempt}/manual/", "uat/attempt-{attempt}/manual-leases/",
        "uat/attempt-{attempt}/raw/",
        "uat/attempt-{attempt}/finalizations/", "uat/attempt-{attempt}/publication-retries/",
        "materialization/", "upgrade-rollback/", "focused-repair-validation/"
    )
    $expectedContractSources = @(
        [pscustomobject][ordered]@{
            path = "docs/sprints/sprint-8a-uat/scenario-contract.json"
            sha256 = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    ) + @($manifest.scenarios | ForEach-Object {
        [pscustomobject][ordered]@{ path = [string]$_.document.path; sha256 = [string]$_.document.sha256 }
    })
    if ((@($inventory.required.path) -join "`n") -cne ($expectedRequiredPaths -join "`n") -or
        (@($inventory.namespaces) -join "`n") -cne ($expectedNamespaces -join "`n") -or
        (@($inventory.sit_lanes) -join ",") -cne "static-and-boundaries,rust-workspace,playwright,deployed-acceptance-smoke" -or
        (@($inventory.manual_uat) -join "`n") -cne ((1..8 | ForEach-Object { "UAT-8A-{0:d2}" -f $_ }) -join "`n") -or
        (ConvertTo-Sprint8APreflightCanonicalJson $inventory.manual_contract_sources) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $expectedContractSources) -or
        @($inventory.required.path | Where-Object { $_ -cmatch '\{01\.\.08\}' }).Count -ne 0 -or
        (@($inventory.required.path | Where-Object {
            $_ -cmatch '^uat/attempt-\{attempt\}/(?:manual|manual-leases|raw)/'
        }) -join "`n") -cne (@($expectedManualPaths) -join "`n") -or
        @($inventory.required.path | Sort-Object -Unique).Count -ne @($inventory.required).Count) {
        throw "Sprint 8A preflight evidence inventory self-test is incomplete or duplicated."
    }

    $temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-preflight-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($temporaryRoot) | Out-Null
    try {
        $receiptPath = Join-Path $temporaryRoot "attempt.json"
        $first = Write-Sprint8APreflightJsonReceipt -Document ([ordered]@{ state = "preparing" }) -Path $receiptPath
        if ((Assert-Sprint8APreflightSidecar -Path $receiptPath) -cne $first) {
            throw "Sprint 8A preflight receipt self-test did not retain its exact sidecar."
        }
        $duplicateRejected = $false
        try {
            Write-Sprint8APreflightJsonReceipt -Document ([ordered]@{ state = "invalid" }) -Path $receiptPath | Out-Null
        } catch { $duplicateRejected = $true }
        if (-not $duplicateRejected) {
            throw "Sprint 8A preflight receipt self-test reused immutable evidence."
        }
        Write-Sprint8APreflightJsonReceipt -Document ([ordered]@{ state = "executing" }) -Path $receiptPath -Overwrite | Out-Null
        $rehearsalEvidenceRoot = Join-Path $temporaryRoot "evidence"
        $rehearsalAttemptPath = Join-Path $rehearsalEvidenceRoot "attempts/candidate-rehearsal-4-attempt.json"
        $syntheticSource = [pscustomobject][ordered]@{
            commit = "1" * 40; tree = "2" * 40; dirty = $false; branch = "self-test"
            acceptance_inventory_sha256 = "3" * 64; deployment_inputs_sha256 = "4" * 64
        }
        $syntheticLineage = [pscustomobject][ordered]@{ schema_version = 1; links = @() }
        $syntheticReadinessPrerequisite = [pscustomobject][ordered]@{
            path = "evidence/attempts/readiness-3.json"
            sha256 = "6" * 64
        }
        $rehearsalAttemptSha = Write-Sprint8APreflightJsonReceipt -Document ([pscustomobject][ordered]@{
            schema_version = 2
            sprint = "sprint-8a"
            phase = "candidate-rehearsal"
            attempt = 4
            authoritative = $false
            state = "passed"
            mutable_source_identity = $syntheticSource
            environment_fingerprint = "5" * 64
            prerequisite_receipts = @($syntheticReadinessPrerequisite)
            correction_lineage = $syntheticLineage
        }) -Path $rehearsalAttemptPath
        $syntheticResult = [pscustomobject][ordered]@{
            attempt = 4
            state = "passed"
            mutable_source_identity = $syntheticSource
            environment_fingerprint = "5" * 64
            prerequisite_receipts = @($syntheticReadinessPrerequisite)
            attempt_receipt = [pscustomobject][ordered]@{
                path = "evidence/attempts/candidate-rehearsal-4-attempt.json"
                sha256 = $rehearsalAttemptSha
            }
            correction_lineage = $syntheticLineage
        }
        $syntheticState = [pscustomobject][ordered]@{
            rehearsal = [pscustomobject][ordered]@{ attempt = 4; state = "passed" }
            correction_lineage = $syntheticLineage
        }
        Assert-Sprint8APreflightRehearsalAttemptReceipt `
            -RehearsalResult $syntheticResult `
            -ValidationState $syntheticState `
            -RepositoryRoot $temporaryRoot `
            -EvidenceRootPath $rehearsalEvidenceRoot | Out-Null
        $mismatchedState = [pscustomobject][ordered]@{
            rehearsal = [pscustomobject][ordered]@{ attempt = 4; state = "passed" }
            correction_lineage = [pscustomobject][ordered]@{
                schema_version = 1
                links = @([pscustomobject]@{ sequence = 1 })
            }
        }
        $lineageMismatchRejected = $false
        try {
            Assert-Sprint8APreflightRehearsalAttemptReceipt `
                -RehearsalResult $syntheticResult `
                -ValidationState $mismatchedState `
                -RepositoryRoot $temporaryRoot `
                -EvidenceRootPath $rehearsalEvidenceRoot | Out-Null
        } catch { $lineageMismatchRejected = $true }
        if (-not $lineageMismatchRejected) {
            throw "Sprint 8A preflight self-test accepted divergent rehearsal attempt/result/state lineage."
        }
        $prerequisiteMismatchResult = $syntheticResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
        $prerequisiteMismatchResult.prerequisite_receipts[0].sha256 = "7" * 64
        $prerequisiteMismatchRejected = $false
        try {
            Assert-Sprint8APreflightRehearsalAttemptReceipt `
                -RehearsalResult $prerequisiteMismatchResult `
                -ValidationState $syntheticState `
                -RepositoryRoot $temporaryRoot `
                -EvidenceRootPath $rehearsalEvidenceRoot | Out-Null
        } catch { $prerequisiteMismatchRejected = $true }
        if (-not $prerequisiteMismatchRejected) {
            throw "Sprint 8A preflight self-test accepted divergent rehearsal attempt/result prerequisite."
        }
        $lockPath = Join-Path $temporaryRoot "validation-attempt.lock"
        $firstLock = Open-Sprint8AValidationAttemptLock -Path $lockPath
        try {
            $contentionRejected = $false
            try {
                $secondLock = Open-Sprint8AValidationAttemptLock -Path $lockPath
                $secondLock.Dispose()
            } catch [IO.IOException] {
                $contentionRejected = $true
            }
            if (-not $contentionRejected) {
                throw "Sprint 8A preflight attempt-lock self-test admitted concurrent ownership."
            }
        } finally {
            $firstLock.Dispose()
        }
    } finally {
        if (Test-Sprint8APreflightContainedPath -Parent ([IO.Path]::GetTempPath()) -Child $temporaryRoot) {
            [IO.Directory]::Delete($temporaryRoot, $true)
        }
    }

    $structuredException = [InvalidOperationException]::new("self-test")
    $structuredException.Data["Sprint8AClassification"] = "environment"
    $resolution = Resolve-Sprint8APreflightClassification `
        -Default "preflight/setup" `
        -ErrorRecord ([Management.Automation.ErrorRecord]::new(
            $structuredException,
            "self-test",
            [Management.Automation.ErrorCategory]::InvalidOperation,
            $null
        ))
    if ([string]$resolution.classification -cne "environment" -or
        [string]$resolution.source -cne "structured_exception") {
        throw "Sprint 8A preflight structured classification self-test failed."
    }

    . (Join-Path $PSScriptRoot "sprint-8a-lifecycle-chain.ps1")
    Test-Sprint8ALifecycleChain | Out-Null
    Test-Sprint8AEvidenceReferenceResolution | Out-Null
    Test-Sprint8AEnvironmentContractComparison | Out-Null

    $tokens = $null
    $parseErrors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile(
        $PSCommandPath,
        [ref]$tokens,
        [ref]$parseErrors
    )
    $forbiddenInvocations = @($ast.FindAll({
        param($node)
        if ($node -isnot [Management.Automation.Language.CommandAst]) { return $false }
        $commandName = [string]$node.GetCommandName()
        $commandName -match '(?i)(materialize|validate-e2e|run-sprint-8a-sit|run-sprint-8a-formal-uat|run-sprint-8a-failure-containment)\.ps1$' -or
            $commandName -match '(?i)^(cargo|npm|npx)$' -or
            $commandName -match '(?i)(smoke-sprint-8a|audit-sprint-8a-deployed-inventory|run-sprint-8a-component-upgrade)\.ps1$'
    }, $true))
    if (@($parseErrors).Count -ne 0 -or $forbiddenInvocations.Count -ne 0) {
        throw "Sprint 8A preflight self-test found a parser error or an unauthorized SIT/UAT execution edge."
    }
    $mutatingDockerInvocations = @($ast.FindAll({
        param($node)
        if ($node -isnot [Management.Automation.Language.CommandAst] -or
            [string]$node.GetCommandName() -cne "docker") { return $false }
        $node.Extent.Text -match '(?i)\b(compose\s+)?(build|up|down|create|run|rm|stop|restart)\b'
    }, $true))
    if ($mutatingDockerInvocations.Count -ne 0) {
        throw "Sprint 8A preflight self-test found an image-build or topology-mutation Docker edge."
    }
    $identityCalls = @($ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.CommandAst] -and
            [string]$node.GetCommandName() -ceq "Get-Sprint8ACandidateIdentity"
    }, $true))
    $lifecycleCalls = @($ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.CommandAst] -and
            [string]$node.GetCommandName() -ceq "Publish-Sprint8ALifecycleReceipt"
    }, $true))
    $manifestCalls = @($ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.CommandAst] -and
            [string]$node.GetCommandName() -ceq "Publish-Sprint8AEvidenceManifest"
    }, $true))
    if ($identityCalls.Count -ne 2 -or
        @($identityCalls | Where-Object {
            -not $_.Extent.Text.Contains("-NormalizedDeploymentConfigurationSha256")
        }).Count -ne 0 -or
        $lifecycleCalls.Count -ne 2 -or
        @($lifecycleCalls | Where-Object {
            -not $_.Extent.Text.Contains("-NormalizedDeploymentConfigurationSha256")
        }).Count -ne 0 -or
        $manifestCalls.Count -ne 1 -or
        $manifestCalls[0].Extent.Text.Contains("-AuthorizedReplacementPaths")) {
        throw "Sprint 8A preflight self-test found a stale normalized-Compose or initial-manifest publication boundary."
    }

    $requiredFunctions = @(
        "Open-Sprint8AValidationAttemptLock",
        "Get-Sprint8APreflightDeclaredRunnerChecks",
        "Assert-Sprint8APreflightPassingAttemptChecks",
        "Find-Sprint8APreflightAuthoritativeDownstreamClaim",
        "Assert-Sprint8APreflightDeploymentContract",
        "Get-Sprint8ADownstreamCommandSets",
        "Assert-Sprint8APreflightDownstreamCommands",
        "Assert-Sprint8APreflightEvidencePaths",
        "Publish-Sprint8APreflightInventory",
        "Initialize-Sprint8APreflightEvidenceManifest"
    )
    $functionDefinitions = @($ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst]
    }, $true))
    foreach ($name in $requiredFunctions) {
        $matches = @($functionDefinitions | Where-Object Name -CEQ $name)
        if ($matches.Count -ne 1) {
            throw "Sprint 8A preflight self-test requires exactly one '$name' implementation."
        }
    }
    foreach ($name in @(
        "Get-Sprint8AEvidenceFileManifestEntries",
        "Publish-Sprint8AEvidenceManifest",
        "Assert-Sprint8AEvidenceManifestCompleteness"
    )) {
        if (-not (Get-Command $name -CommandType Function -ErrorAction SilentlyContinue)) {
            throw "Sprint 8A preflight self-test cannot find lifecycle manifest helper '$name'."
        }
    }

    foreach ($name in @(
        "Get-Sprint8APreflightDeclaredRunnerChecks",
        "Find-Sprint8APreflightAuthoritativeDownstreamClaim"
    )) {
        $definition = @($functionDefinitions | Where-Object Name -CEQ $name)[0]
        . ([scriptblock]::Create($definition.Extent.Text))
    }
    $readinessNames = @(Get-Sprint8APreflightDeclaredRunnerChecks `
        -Path (Join-Path $repoRoot "scripts/validate-sprint-8a-readiness.ps1") `
        -EndMarker 'Assert-Sprint8AReadinessFailLateGraph -Checks $declaredChecks')
    $rehearsalNames = @(Get-Sprint8APreflightDeclaredRunnerChecks `
        -Path (Join-Path $repoRoot "scripts/run-sprint-8a-candidate-rehearsal.ps1") `
        -EndMarker 'Assert-RehearsalGraph -Checks $declaredChecks')
    $claimProbe = [pscustomobject]@{
        child = [pscustomobject]@{ phase = "uat"; authoritative = $true }
    }
    if ($readinessNames -cnotcontains "runner-self-tests" -or
        $rehearsalNames -cnotcontains "failure-containment-successor-health" -or
        @(Find-Sprint8APreflightAuthoritativeDownstreamClaim -Value $claimProbe).Count -ne 1) {
        throw "Sprint 8A preflight self-test found stale prerequisite identity or downstream-authority auditing."
    }

    $commandInventoryFunction = @($functionDefinitions | Where-Object Name -CEQ "Get-Sprint8ADownstreamCommandSets")[0]
    . ([scriptblock]::Create($commandInventoryFunction.Extent.Text))
    $commandSets = Get-Sprint8ADownstreamCommandSets
    if (@($commandSets.static_and_boundaries).Count -ne 10 -or
        @($commandSets.rust_workspace).Count -ne 6 -or
        @($commandSets.playwright).Count -ne 3 -or
        @($commandSets.deployed_acceptance_smoke).Count -ne 9 -or
        @($commandSets.formal_uat).Count -ne 2) {
        throw "Sprint 8A preflight self-test found an incomplete downstream command inventory."
    }
    $verificationText = Get-Content -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8a-verification.md") -Raw
    foreach ($setName in @(
        "static_and_boundaries", "rust_workspace", "playwright",
        "deployed_acceptance_smoke", "formal_uat"
    )) {
        $cursor = -1
        foreach ($command in @($commandSets.$setName)) {
            $index = $verificationText.IndexOf([string]$command, $cursor + 1, [StringComparison]::Ordinal)
            if ($index -lt 0) {
                throw "Sprint 8A preflight self-test cannot find ordered '$setName' command '$command'."
            }
            $cursor = $index
        }
    }
    $parameterContracts = [ordered]@{
        "scripts/materialize-sprint-8a.ps1" = @("Attempt", "EvidenceRoot", "EnvironmentFingerprint", "AuthorizeDisposableReset", "VerifyNoOp")
        "scripts/run-sprint-8a-deployed-smoke.ps1" = @("DeploymentEvidencePath")
        "scripts/validate-e2e.ps1" = @("BaseUrl", "DeploymentEvidencePath", "ExpectedDataState", "TransitionCatalogProfile", "EvidencePath", "FailureEvidenceDirectory")
        "scripts/audit-sprint-8a-deployed-inventory.ps1" = @("BaseUrl", "OutputPath")
        "scripts/smoke-sprint-8a.ps1" = @("BaseUrl", "SupervisorUrl", "OutputPath")
        "scripts/run-sprint-8a-component-upgrade.ps1" = @("OutputPath")
        "scripts/run-sprint-8a-failure-containment.ps1" = @("Attempt", "EvidenceRoot", "EnvironmentFingerprint", "OutputPath", "AuthorizeDisposableReset", "SkipBuild")
        "scripts/run-sprint-8a-sit.ps1" = @("Stage", "Attempt", "PreflightReceipt", "CandidateReceipt", "EvidenceRoot", "OutputPath", "BaseUrl", "AuthorizeDisposableReset", "SelfTest")
        "scripts/run-sprint-8a-formal-uat.ps1" = @("Stage", "Attempt", "PreflightReceipt", "CandidateReceipt", "SitReceipt", "EvidenceRoot", "OutputPath", "BaseUrl", "AuthorizeDisposableReset", "SelfTest")
    }
    foreach ($runner in $parameterContracts.Keys) {
        $declared = @(Get-Sprint8AScriptParameterNames -Path (Join-Path $repoRoot $runner))
        $missing = @($parameterContracts[$runner] | Where-Object { $declared -cnotcontains $_ })
        if ($missing.Count -ne 0) {
            throw "Sprint 8A preflight self-test found stale '$runner' parameters: $($missing -join ', ')."
        }
    }
    if (-not (Get-Content -LiteralPath (Join-Path $repoRoot "scripts/materialize-sprint-8a.ps1") -Raw).Contains(
            '[CmdletBinding(SupportsShouldProcess, ConfirmImpact = "High")]'
        )) {
        throw "Sprint 8A preflight self-test found materialization without the documented -Confirm common parameter."
    }
    $sitCoordinatorText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/run-sprint-8a-sit.ps1") -Raw
    foreach ($fragment in @(
        '[ValidateSet("Run", "Finalize")]', 'Open-Sprint8AValidationAttemptLock',
        'validation-attempt.lock', 'Get-Sprint8ASitLaneNames',
        'Get-Sprint8AEvidenceFileManifestEntries', 'Assert-Sprint8AEvidenceManifestCompleteness',
        'Publish-Sprint8ALifecycleReceipt', 'canonical_topology_verified',
        'finalization_only_retry_permitted_if_identity_remains_exact'
    )) {
        if (-not $sitCoordinatorText.Contains($fragment)) {
            throw "Sprint 8A preflight self-test found stale SIT coordinator fragment '$fragment'."
        }
    }

    $sourceText = Get-Content -LiteralPath $PSCommandPath -Raw
    $receiptChainText = @($functionDefinitions | Where-Object Name -CEQ "Assert-Sprint8APreflightReceiptChain")[0].Extent.Text
    $lockAcquisition = $sourceText.LastIndexOf(
        '$script:runtimeContext.attempt_lock = Open-Sprint8AValidationAttemptLock -Path $preflightLockPath',
        [StringComparison]::Ordinal
    )
    $lifecycleLoad = $receiptChainText.IndexOf('sprint-8a-lifecycle-chain.ps1', [StringComparison]::Ordinal)
    $attemptStartFragment = 'Write-Sprint8APreflightJsonReceipt -Document $script:' +
        'attemptReceipt -Path $attemptPath'
    $attemptStart = $sourceText.IndexOf($attemptStartFragment, [StringComparison]::Ordinal)
    $schedulerStart = $sourceText.LastIndexOf('foreach ($declaration in $declaredChecks)', [StringComparison]::Ordinal)
    $manifestScan = $sourceText.LastIndexOf('Get-Sprint8AEvidenceFileManifestEntries', [StringComparison]::Ordinal)
    $manifestPublish = $sourceText.LastIndexOf('Publish-Sprint8AEvidenceManifest', [StringComparison]::Ordinal)
    $manifestAssert = $sourceText.LastIndexOf('Assert-Sprint8AEvidenceManifestCompleteness', [StringComparison]::Ordinal)
    if ($lockAcquisition -lt 0 -or $lifecycleLoad -lt 0 -or
        $attemptStart -le $lockAcquisition -or $schedulerStart -le $attemptStart -or
        $manifestScan -lt 0 -or $manifestPublish -le $manifestScan -or $manifestAssert -le $manifestPublish -or
        -not $sourceText.Contains('future_paths_are_declarations_not_manifest_entries = $true') -or
        -not $sourceText.Contains('mutable_attempt_checkpoints_are_overwritten_only_by_the_owning_runner = $true') -or
        -not $sourceText.Contains('immutable_snapshots_and_terminal_receipts_are_never_overwritten = $true')) {
        throw "Sprint 8A preflight self-test found a stale start-receipt, scheduler, or manifest-initialization boundary."
    }
    "Sprint 8A validation-preflight graph, receipt, classification, lifecycle, and no-execution self-test passed."
}

$declaredChecks = @(Get-Sprint8APreflightDeclarations)
Assert-Sprint8APreflightGraph -Checks $declaredChecks

if ($SelfTest) {
    Test-Sprint8AValidationPreflightRunner
    return
}

if ($Attempt -lt 1) {
    throw "Sprint 8A validation preflight requires a positive unused -Attempt."
}
if ([IO.Path]::IsPathRooted($EvidenceRoot) -or
    $EvidenceRoot -match '(^|[\\/])\.\.([\\/]|$)' -or
    $EvidenceRoot.Replace("\", "/").TrimEnd("/") -cne "artifacts/sprint-8a-closeout") {
    throw "Sprint 8A validation preflight requires canonical repository-relative -EvidenceRoot 'artifacts/sprint-8a-closeout'."
}

$script:evidenceRootPath = [IO.Path]::GetFullPath((Join-Path $repoRoot $EvidenceRoot))
if (-not (Test-Sprint8APreflightContainedPath -Parent $repoRoot -Child $script:evidenceRootPath)) {
    throw "Sprint 8A validation preflight evidence root escapes the repository."
}
$evidenceRootRelative = [IO.Path]::GetRelativePath($repoRoot, $script:evidenceRootPath).Replace("\", "/").TrimEnd("/")
$attemptPath = Join-Path $script:evidenceRootPath "attempts/preflight-$Attempt.json"
$attemptRoot = Join-Path $script:evidenceRootPath "preflight/attempt-$Attempt"
$logRoot = Join-Path $attemptRoot "logs"
$structuredRoot = Join-Path $attemptRoot "evidence"
$inventoryPath = Join-Path $attemptRoot "evidence-inventory.json"
$preflightResultPath = Join-Path $script:evidenceRootPath "preflight-result.json"
$candidatePath = Join-Path $script:evidenceRootPath "candidate.json"
$manifestPath = Join-Path $script:evidenceRootPath "evidence-manifest.json"
$validationStatePath = Join-Path $script:evidenceRootPath "validation-state.json"

foreach ($path in @($attemptPath, "$attemptPath.sha256", $attemptRoot)) {
    if (Test-Path -LiteralPath $path) {
        throw "Sprint 8A preflight attempt $Attempt already exists and cannot be reused: $path"
    }
}
$placeholderSource = [pscustomobject][ordered]@{
    commit = "0" * 40
    tree = "0" * 40
    dirty = $false
    branch = "unverified"
    acceptance_inventory_sha256 = "0" * 64
    deployment_inputs_sha256 = "0" * 64
}
$startedAt = [DateTimeOffset]::UtcNow
$script:attemptReceipt = [pscustomobject][ordered]@{
    schema_version = 1
    sprint = "sprint-8a"
    phase = "validation-preflight-attempt"
    attempt = $Attempt
    authoritative = $false
    state = "preparing"
    assertions_started = $false
    assertions_started_at = $null
    started_at = $startedAt.ToString("o")
    ended_at = $null
    duration_ms = 0L
    mutable_source_identity = $placeholderSource
    source_identity_verification_state = "unverified"
    environment_fingerprint = "0" * 64
    environment_verification_state = "unverified"
    normalized_deployment_configuration_sha256 = "0" * 64
    prerequisite_receipts = @()
    declared_checks = $declaredChecks
    checks = @()
    assertion_count = 0
    failure_count = 0
    blocked_count = 0
    classification = $null
    classifications = @()
    active_check = $null
    finalization_failure = $null
    correction = $null
    narrow_proof = $null
    invalidation_decision = "candidate freeze forbidden until this exact preflight passes"
    cleanup_restoration = [pscustomobject][ordered]@{
        required = $false
        result = "not_applicable"
        evidence = @()
    }
    preflight_result = $null
    candidate = $null
    evidence_manifest = $null
}
$script:runtimeContext = [ordered]@{
    lifecycle_loaded = $false
    readiness = $null
    readiness_reference = $null
    readiness_immutable_reference = $null
    rehearsal = $null
    rehearsal_reference = $null
    rehearsal_attempt_reference = $null
    validation_state = $null
    validation_state_reference = $null
    environment = $null
    source = $null
    candidate_identity = $null
    normalized_deployment_configuration_sha256 = $null
    correction_authorization_reference = $null
    correction_lineage_validation = $null
    attempt_lock = $null
    attempt_lock_path = $null
}
$script:terminalChecks = [Collections.Generic.List[object]]::new()
$script:terminalByName = @{}
$script:producedEvidenceByCheck = @{}

function Update-Sprint8APreflightAttemptIdentity {
    if ($null -ne $script:runtimeContext.readiness_reference -and
        $null -ne $script:runtimeContext.rehearsal_reference) {
        $script:attemptReceipt.prerequisite_receipts = @(
            [pscustomobject][ordered]@{
                path = [string]$script:runtimeContext.readiness_reference.path
                sha256 = [string]$script:runtimeContext.readiness_reference.sha256
            }
            [pscustomobject][ordered]@{
                path = [string]$script:runtimeContext.rehearsal_reference.path
                sha256 = [string]$script:runtimeContext.rehearsal_reference.sha256
            }
        )
    }
    if ($null -ne $script:runtimeContext.source) {
        $script:attemptReceipt.mutable_source_identity = $script:runtimeContext.source
    } elseif ($null -ne $script:runtimeContext.rehearsal) {
        $script:attemptReceipt.mutable_source_identity = $script:runtimeContext.rehearsal.mutable_source_identity
    }
    if ($null -ne $script:runtimeContext.environment) {
        $script:attemptReceipt.environment_fingerprint = [string]$script:runtimeContext.environment.fingerprint
    } elseif ($null -ne $script:runtimeContext.rehearsal) {
        $script:attemptReceipt.environment_fingerprint = [string]$script:runtimeContext.rehearsal.environment_fingerprint
    }
    if ([string]$script:runtimeContext.normalized_deployment_configuration_sha256 -match '^[0-9a-f]{64}$') {
        $script:attemptReceipt.normalized_deployment_configuration_sha256 = [string]$script:runtimeContext.normalized_deployment_configuration_sha256
    }
}

function Checkpoint-Sprint8APreflightAttempt {
    Update-Sprint8APreflightAttemptIdentity
    $script:attemptReceipt.checks = @($script:terminalChecks)
    $script:attemptReceipt.assertion_count = @($script:terminalChecks | Where-Object assertions_started -EQ $true).Count
    $finalizationFailureCount = if ($null -eq $script:attemptReceipt.finalization_failure) { 0 } else { 1 }
    $script:attemptReceipt.failure_count = @($script:terminalChecks | Where-Object state -CEQ "failed").Count + $finalizationFailureCount
    $script:attemptReceipt.blocked_count = @($script:terminalChecks | Where-Object state -CEQ "blocked").Count
    $classifications = @($script:terminalChecks | Where-Object state -CEQ "failed" | ForEach-Object {
        [string]$_.classification
    })
    if ($null -ne $script:attemptReceipt.finalization_failure) {
        $classifications += [string]$script:attemptReceipt.finalization_failure.classification
    }
    $classifications = @($classifications | Where-Object {
        -not [string]::IsNullOrWhiteSpace($_)
    } | Sort-Object -Unique)
    $script:attemptReceipt.classifications = $classifications
    $script:attemptReceipt.classification = if ($classifications.Count -eq 1) {
        [string]$classifications[0]
    } else { $null }
    Write-Sprint8APreflightJsonReceipt `
        -Document $script:attemptReceipt `
        -Path $attemptPath `
        -Overwrite | Out-Null
}

function Publish-Sprint8APreflightStructuredEvidence {
    param(
        [Parameter(Mandatory)][string]$CheckName,
        [Parameter(Mandatory)]$Document
    )

    $path = Join-Path $structuredRoot "$CheckName.json"
    $sha256 = Write-Sprint8APreflightJsonReceipt -Document $Document -Path $path
    $reference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $path).Replace("\", "/")
        sha256 = $sha256
    }
    $script:producedEvidenceByCheck[$CheckName] = @($reference)
    $reference
}

function Invoke-Sprint8APreflightCheck {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][scriptblock]$Action
    )

    $declaration = @($declaredChecks | Where-Object name -CEQ $Name)
    if ($declaration.Count -ne 1 -or $script:terminalByName.ContainsKey($Name)) {
        throw "Sprint 8A preflight check '$Name' is missing, duplicated, or already terminal."
    }
    $failedDependencies = @($declaration[0].depends_on | Where-Object {
        -not $script:terminalByName.ContainsKey([string]$_) -or
            [string]$script:terminalByName[[string]$_].state -cne "passed"
    })
    if ($failedDependencies.Count -gt 0) {
        $blocked = [pscustomobject][ordered]@{
            name = $Name
            depends_on = @($declaration[0].depends_on)
            command = [string]$declaration[0].command
            state = "blocked"
            assertions_started = $false
            assertions_started_at = $null
            started_at = $null
            ended_at = [DateTimeOffset]::UtcNow.ToString("o")
            duration_ms = 0L
            exit_status = $null
            classification = $null
            classification_source = $null
            failure_message = $null
            blocked_reason = "blocked by failed prerequisite(s): $($failedDependencies -join ', ')"
            evidence = @()
        }
        $script:terminalChecks.Add($blocked)
        $script:terminalByName[$Name] = $blocked
        Checkpoint-Sprint8APreflightAttempt
        return
    }

    $checkStartedAt = [DateTimeOffset]::UtcNow
    $logPath = Join-Path $logRoot "$Name.log"
    [IO.File]::WriteAllText(
        $logPath,
        "[$($checkStartedAt.ToString('o'))] assertion_started name=$Name`n",
        [Text.UTF8Encoding]::new($false)
    )
    if (-not [bool]$script:attemptReceipt.assertions_started) {
        $script:attemptReceipt.assertions_started = $true
        $script:attemptReceipt.assertions_started_at = $checkStartedAt.ToString("o")
    }
    if ([string]$script:attemptReceipt.state -cne "harvesting") {
        $script:attemptReceipt.state = "executing"
    }
    $script:attemptReceipt.active_check = $Name
    Checkpoint-Sprint8APreflightAttempt

    $passed = $false
    $failureMessage = $null
    $classification = $null
    try {
        & $Action *>&1 | ForEach-Object {
            $line = ($_ | Out-String).TrimEnd()
            if (-not [string]::IsNullOrWhiteSpace($line)) {
                [IO.File]::AppendAllText(
                    $logPath,
                    "[$([DateTimeOffset]::UtcNow.ToString('o'))] $line`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        $passed = $true
    } catch {
        $failureMessage = $_.Exception.Message
        $classification = Resolve-Sprint8APreflightClassification `
            -Default ([string]$declaration[0].failure_classification) `
            -ErrorRecord $_
        [IO.File]::AppendAllText(
            $logPath,
            "[$([DateTimeOffset]::UtcNow.ToString('o'))] failure`n$($_ | Out-String)`n",
            [Text.UTF8Encoding]::new($false)
        )
    }
    $checkEndedAt = [DateTimeOffset]::UtcNow
    $evidence = @(
        [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $logPath).Replace("\", "/")
            sha256 = Get-Sprint8APreflightFileSha256 -Path $logPath
        }
    ) + @($script:producedEvidenceByCheck[$Name])
    $entry = [pscustomobject][ordered]@{
        name = $Name
        depends_on = @($declaration[0].depends_on)
        command = [string]$declaration[0].command
        state = if ($passed) { "passed" } else { "failed" }
        assertions_started = $true
        assertions_started_at = $checkStartedAt.ToString("o")
        started_at = $checkStartedAt.ToString("o")
        ended_at = $checkEndedAt.ToString("o")
        duration_ms = [long][Math]::Max(0, ($checkEndedAt - $checkStartedAt).TotalMilliseconds)
        exit_status = if ($passed) { 0 } else { 1 }
        classification = if ($passed) { $null } else { [string]$classification.classification }
        classification_source = if ($passed) { $null } else { [string]$classification.source }
        failure_message = $failureMessage
        blocked_reason = $null
        evidence = $evidence
    }
    $script:terminalChecks.Add($entry)
    $script:terminalByName[$Name] = $entry
    if (-not $passed) {
        $script:attemptReceipt.state = "harvesting"
    }
    $script:attemptReceipt.active_check = $null
    Checkpoint-Sprint8APreflightAttempt
}

function Get-Sprint8APreflightDeclaredRunnerChecks {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$EndMarker
    )

    $source = Get-Content -LiteralPath $Path -Raw
    if (-not $source.Contains($EndMarker)) {
        throw "Sprint 8A cannot extract the canonical declared check inventory from '$Path'."
    }
    $tokens = $null
    $parseErrors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile(
        [IO.Path]::GetFullPath($Path),
        [ref]$tokens,
        [ref]$parseErrors
    )
    $assignments = @($ast.FindAll({
        param($node)
        $node -is [Management.Automation.Language.AssignmentStatementAst] -and
            $node.Left -is [Management.Automation.Language.VariableExpressionAst] -and
            [string]$node.Left.VariablePath.UserPath -ceq "declaredChecks"
    }, $true))
    if (@($parseErrors).Count -ne 0 -or $assignments.Count -ne 1) {
        throw "Sprint 8A cannot identify one parsed declared check inventory in '$Path'."
    }
    $block = [string]$assignments[0].Right.Extent.Text
    $names = @([regex]::Matches(
        $block,
        '(?m)\[ordered\]@\{\s*name\s*=\s*"([^"]+)"'
    ) | ForEach-Object { [string]$_.Groups[1].Value })
    if ($names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count) {
        throw "Sprint 8A runner '$Path' has an empty or duplicated declared check inventory."
    }
    $names
}

function Assert-Sprint8APreflightPassingAttemptChecks {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    $checks = @($Receipt.checks)
    $actualNames = @($checks | ForEach-Object { [string]$_.name })
    if ($checks.Count -ne $ExpectedNames.Count -or
        @($actualNames | Sort-Object -Unique).Count -ne $checks.Count -or
        (($actualNames | Sort-Object) -join "`n") -cne (($ExpectedNames | Sort-Object) -join "`n")) {
        throw "$Label does not retain its exact current runner check identities."
    }
    foreach ($check in $checks) {
        if ([string]$check.state -cne "passed" -or
            $check.assertions_started -isnot [bool] -or -not [bool]$check.assertions_started -or
            [int]$check.exit_status -ne 0 -or
            @($check.evidence).Count -lt 1) {
            throw "$Label check '$($check.name)' is not one complete passing asserted result."
        }
        foreach ($evidence in @($check.evidence)) {
            $resolved = Resolve-Sprint8APreflightEvidencePath `
                -RepositoryRoot $repoRoot `
                -EvidenceRootPath $script:evidenceRootPath `
                -Path ([string]$evidence.path)
            if ([string]$evidence.sha256 -notmatch '^[0-9a-f]{64}$' -or
                (Get-Sprint8APreflightFileSha256 -Path ([string]$resolved.full_path)) -cne [string]$evidence.sha256) {
                throw "$Label check '$($check.name)' has stale raw evidence '$($evidence.path)'."
            }
        }
    }
}

function Find-Sprint8APreflightAuthoritativeDownstreamClaim {
    param(
        [AllowNull()]$Value,
        [string]$Path = '$'
    )

    if ($null -eq $Value -or $Value -is [string] -or $Value.GetType().IsPrimitive) { return }
    if ($Value -is [Collections.IDictionary]) {
        $phase = if ($Value.Contains("phase")) { [string]$Value["phase"] } else { $null }
        $authority = if ($Value.Contains("authoritative")) { $Value["authoritative"] } else { $null }
        if ($authority -is [bool] -and [bool]$authority -and
            $phase -match '(?i)(^sit$|uat)') {
            $Path
        }
        foreach ($key in @($Value.Keys)) {
            Find-Sprint8APreflightAuthoritativeDownstreamClaim -Value $Value[$key] -Path "$Path.$key"
        }
        return
    }
    if ($Value -is [Collections.IEnumerable]) {
        $index = 0
        foreach ($item in $Value) {
            Find-Sprint8APreflightAuthoritativeDownstreamClaim -Value $item -Path "$Path[$index]"
            $index++
        }
        return
    }
    $phaseProperty = $Value.PSObject.Properties["phase"]
    $authorityProperty = $Value.PSObject.Properties["authoritative"]
    if ($null -ne $phaseProperty -and $null -ne $authorityProperty -and
        $authorityProperty.Value -is [bool] -and [bool]$authorityProperty.Value -and
        [string]$phaseProperty.Value -match '(?i)(^sit$|uat)') {
        $Path
    }
    foreach ($property in $Value.PSObject.Properties) {
        Find-Sprint8APreflightAuthoritativeDownstreamClaim `
            -Value $property.Value `
            -Path "$Path.$($property.Name)"
    }
}

function Assert-Sprint8APreflightReceiptChain {
    $lockPath = Join-Path $script:evidenceRootPath "validation-attempt.lock"
    if ($script:runtimeContext.attempt_lock -isnot [IO.FileStream] -or
        -not $script:runtimeContext.attempt_lock.CanWrite -or
        [IO.Path]::GetFullPath([string]$script:runtimeContext.attempt_lock.Name) -cne
            [IO.Path]::GetFullPath($lockPath) -or
        [string]$script:runtimeContext.attempt_lock_path -cne $lockPath) {
        throw "Sprint 8A preflight attempt lock was not acquired before evidence-root mutation."
    }

    . (Join-Path $PSScriptRoot "sprint-8a-lifecycle-chain.ps1")
    $script:runtimeContext.lifecycle_loaded = $true

    $readinessReference = Get-Sprint8APreflightReceiptReference -Path $ReadinessReceipt
    $rehearsalReference = Get-Sprint8APreflightReceiptReference -Path $RehearsalReceipt
    $readiness = $readinessReference.document
    $rehearsal = $rehearsalReference.document

    if (($readiness.schema_version -isnot [int] -and $readiness.schema_version -isnot [long]) -or
        [int]$readiness.schema_version -ne 2 -or
        [string]$readiness.sprint -cne "sprint-8a" -or
        [string]$readiness.phase -cne "validation-readiness" -or
        $readiness.authoritative -isnot [bool] -or $readiness.authoritative -ne $false -or
        [string]$readiness.state -cne "passed" -or
        [string]$readiness.source_identity_verification_state -cne "verified" -or
        [string]$readiness.environment_identity.verification_state -cne "verified" -or
        [string]$readiness.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [int]$readiness.failure_count -ne 0 -or [int]$readiness.blocked_count -ne 0 -or
        -not [bool]$readiness.assertions_started) {
        throw "Sprint 8A preflight requires one exact complete passing non-authoritative Readiness receipt."
    }
    Assert-Sprint8ASourceIdentityObject -Source $readiness.mutable_source_identity -RequireClean | Out-Null

    $readinessChecks = @(Get-Sprint8APreflightDeclaredRunnerChecks `
        -Path (Join-Path $repoRoot "scripts/validate-sprint-8a-readiness.ps1") `
        -EndMarker 'Assert-Sprint8AReadinessFailLateGraph -Checks $declaredChecks')
    Assert-Sprint8AExactTerminalIdentities `
        -Results @($readiness.checks) `
        -ExpectedNames $readinessChecks `
        -Label "Validation Readiness prerequisite" | Out-Null
    Assert-Sprint8APreflightPassingAttemptChecks `
        -Receipt $readiness `
        -ExpectedNames $readinessChecks `
        -Label "Validation Readiness"
    $readinessImmutableReference = Get-Sprint8APreflightReceiptReference `
        -Path "$evidenceRootRelative/attempts/readiness-$([int]$readiness.attempt).json"
    if ([string]$readinessImmutableReference.sha256 -cne [string]$readinessReference.sha256 -or
        (ConvertTo-Sprint8APreflightCanonicalJson $readinessImmutableReference.document) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $readiness)) {
        throw "Current passing Readiness alias differs from its exact immutable attempt counterpart."
    }

    if (($rehearsal.schema_version -isnot [int] -and $rehearsal.schema_version -isnot [long]) -or
        [int]$rehearsal.schema_version -ne 2 -or
        [string]$rehearsal.sprint -cne "sprint-8a" -or
        [string]$rehearsal.phase -cne "candidate-rehearsal" -or
        $rehearsal.authoritative -isnot [bool] -or $rehearsal.authoritative -ne $false -or
        [string]$rehearsal.state -cne "passed" -or
        [string]$rehearsal.environment_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [int]$rehearsal.failure_count -ne 0 -or [int]$rehearsal.blocked_count -ne 0 -or
        [int]$rehearsal.nested_blocked_count -ne 0 -or [int]$rehearsal.nested_failure_count -ne 0 -or
        [string]$rehearsal.cleanup_restoration.result -cne "canonical_successor_healthy") {
        throw "Sprint 8A preflight requires one exact complete passing non-authoritative Candidate Rehearsal receipt."
    }
    Assert-Sprint8ASourceIdentityObject -Source $rehearsal.mutable_source_identity -RequireClean | Out-Null
    $rehearsalChecks = @(Get-Sprint8APreflightDeclaredRunnerChecks `
        -Path (Join-Path $repoRoot "scripts/run-sprint-8a-candidate-rehearsal.ps1") `
        -EndMarker 'Assert-RehearsalGraph -Checks $declaredChecks')
    Assert-Sprint8AExactTerminalIdentities `
        -Results @($rehearsal.checks) `
        -ExpectedNames $rehearsalChecks `
        -Label "Candidate Rehearsal prerequisite" | Out-Null
    Assert-Sprint8APreflightPassingAttemptChecks `
        -Receipt $rehearsal `
        -ExpectedNames $rehearsalChecks `
        -Label "Candidate Rehearsal"
    $downstreamClaims = @(Find-Sprint8APreflightAuthoritativeDownstreamClaim -Value $rehearsal)
    if ($downstreamClaims.Count -ne 0) {
        throw "Candidate Rehearsal contains forbidden authoritative SIT/UAT claim(s): $($downstreamClaims -join ', ')."
    }
    if (-not (Test-Sprint8ASourceIdentityMatch `
            -Expected $readiness.mutable_source_identity `
            -Actual $rehearsal.mutable_source_identity) -or
        [string]$readiness.environment_fingerprint -cne [string]$rehearsal.environment_fingerprint -or
        @($rehearsal.prerequisite_receipts).Count -ne 1 -or
        -not (Test-Sprint8AReceiptReferenceMatch `
            -References @($rehearsal.prerequisite_receipts) `
            -ExpectedReference $readinessImmutableReference)) {
        throw "Passing Readiness and Rehearsal do not form one exact source/environment/prerequisite chain."
    }

    $environmentReference = Get-Sprint8APreflightReceiptReference -Path ([string]$readiness.environment_identity.path)
    if ([string]$environmentReference.sha256 -cne [string]$readiness.environment_identity.sha256) {
        throw "Readiness environment evidence differs from its exact receipt reference."
    }
    $environment = $environmentReference.document
    if ([string]$environment.fingerprint -cne [string]$readiness.environment_fingerprint -or
        [string]$environment.fingerprint -notmatch '^[0-9a-f]{64}$' -or
        (Get-Sprint8APreflightStringSha256 -Text (ConvertTo-Sprint8APreflightCanonicalJson $environment.contract)) -cne
            [string]$environment.fingerprint -or
        [string]$environment.contract.contract -cne "tessara.sprint-8a.validation-environment") {
        throw "Readiness environment artifact is malformed or has a stale fingerprint."
    }

    $stateReference = Get-Sprint8APreflightReceiptReference `
        -Path "$evidenceRootRelative/validation-state.json"
    $state = $stateReference.document
    $stateReadinessValidation = Assert-Sprint8ACurrentReadinessReference `
        -StateReadiness $state.readiness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -RequirePassed
    if (($state.schema_version -isnot [int] -and $state.schema_version -isnot [long]) -or
        [int]$state.schema_version -ne 1 -or
        [string]$state.sprint -cne "sprint-8a" -or
        $state.preflight_eligible -isnot [bool] -or $state.preflight_eligible -ne $true -or
        [string]$state.environment_fingerprint -cne [string]$environment.fingerprint -or
        -not (Test-Sprint8ASourceIdentityMatch `
            -Expected $rehearsal.mutable_source_identity `
            -Actual $state.source_identity) -or
        [string]$state.readiness.state -cne "passed" -or
        [int]$state.readiness.attempt -ne [int]$readiness.attempt -or
        [string]$state.readiness.receipt -cne [string]$readinessReference.path -or
        [string]$state.readiness.sha256 -cne [string]$readinessReference.sha256 -or
        [string]$stateReadinessValidation.current.path -cne [string]$readinessReference.path -or
        [string]$stateReadinessValidation.immutable.path -cne [string]$readinessImmutableReference.path -or
        [string]$stateReadinessValidation.immutable.sha256 -cne [string]$readinessImmutableReference.sha256 -or
        [string]$state.rehearsal.state -cne "passed" -or
        [int]$state.rehearsal.attempt -ne [int]$rehearsal.attempt -or
        [string]$state.rehearsal.receipt -cne [string]$rehearsalReference.path -or
        [string]$state.rehearsal.sha256 -cne [string]$rehearsalReference.sha256) {
        throw "Validation-state does not authorize preflight for the exact passing Readiness/Rehearsal pair."
    }
    $rehearsalAttemptReference = Assert-Sprint8APreflightRehearsalAttemptReceipt `
        -RehearsalResult $rehearsal `
        -ValidationState $state `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath

    $lineageProperty = $state.PSObject.Properties["correction_lineage"]
    $lineage = if ($null -eq $lineageProperty) { $null } else { $lineageProperty.Value }
    if ($null -eq $lineage) {
        if ($null -ne $readiness.predecessor_correction_authorization -or
            $null -ne $readiness.correction_consumption_receipt) {
            throw "Readiness names correction authorization without one canonical validation-state lineage."
        }
    } else {
        $lineageValidation = Assert-Sprint8ACorrectionLineage `
            -Lineage $lineage `
            -RepositoryRoot $repoRoot `
            -EvidenceRoot $script:evidenceRootPath `
            -ExpectedCurrentReadiness ([pscustomobject]@{
                attempt = [int]$state.readiness.attempt
                receipt = [string]$readinessReference.path
                sha256 = [string]$readinessReference.sha256
                state = "passed"
            }) `
            -RequireConsumedTip
        $tip = $lineageValidation.tip
        $consumptionReference = $tip.consumed_by_readiness.consumption_receipt
        if ([string]$readiness.predecessor_correction_authorization.sha256 -cne [string]$tip.authorization.sha256 -or
            [string]$readiness.correction_consumption_receipt.path -cne [string]$consumptionReference.path -or
            [string]$readiness.correction_consumption_receipt.sha256 -cne [string]$consumptionReference.sha256) {
            throw "Passing Readiness does not bind the exact correction-lineage tip."
        }
        $authorizationReference = Get-Sprint8APreflightReceiptReference -Path ([string]$tip.authorization.path) -AllowInRootAbsolute
        $script:runtimeContext.correction_authorization_reference = [pscustomobject][ordered]@{
            path = [string]$authorizationReference.path
            sha256 = [string]$authorizationReference.sha256
        }
        $script:runtimeContext.correction_lineage_validation = $lineageValidation
    }

    $script:runtimeContext.readiness = $readiness
    $script:runtimeContext.readiness_reference = $readinessReference
    $script:runtimeContext.readiness_immutable_reference = $readinessImmutableReference
    $script:runtimeContext.rehearsal = $rehearsal
    $script:runtimeContext.rehearsal_reference = $rehearsalReference
    $script:runtimeContext.rehearsal_attempt_reference = $rehearsalAttemptReference
    $script:runtimeContext.validation_state = $state
    $script:runtimeContext.validation_state_reference = $stateReference
    $script:runtimeContext.environment = $environment
    $script:attemptReceipt.source_identity_verification_state = "claimed_from_prerequisites"
    $script:attemptReceipt.environment_verification_state = "claimed_from_prerequisites"

    Publish-Sprint8APreflightStructuredEvidence -CheckName "receipt-chain" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-receipt-chain"
        readiness = [pscustomobject]@{ path = $readinessReference.path; sha256 = $readinessReference.sha256 }
        readiness_immutable = [pscustomobject]@{ path = $readinessImmutableReference.path; sha256 = $readinessImmutableReference.sha256 }
        rehearsal = [pscustomobject]@{ path = $rehearsalReference.path; sha256 = $rehearsalReference.sha256 }
        rehearsal_attempt = [pscustomobject]@{ path = $rehearsalAttemptReference.path; sha256 = $rehearsalAttemptReference.sha256 }
        validation_state = [pscustomobject]@{ path = $stateReference.path; sha256 = $stateReference.sha256 }
        environment = [pscustomobject]@{ path = $environmentReference.path; sha256 = $environmentReference.sha256; fingerprint = $environment.fingerprint }
        source_identity = $rehearsal.mutable_source_identity
        correction_authorization = $script:runtimeContext.correction_authorization_reference
        correction_lineage = if ($null -eq $script:runtimeContext.correction_lineage_validation) { @() } else {
            @($script:runtimeContext.correction_lineage_validation.references | ForEach-Object {
                [pscustomobject]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
            })
        }
        readiness_check_identities = $readinessChecks
        rehearsal_check_identities = $rehearsalChecks
        authoritative_downstream_claim_count = 0
        attempt_lock = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $lockPath).Replace("\", "/")
            acquired = $true
            sharing = "none"
            held_through_final_publication = $true
        }
    }) | Out-Null
    "authenticated exact Readiness -> Rehearsal -> preflight authorization chain"
}

function Assert-Sprint8APreflightRepositoryScope {
    $topLevel = @(& git -C $repoRoot rev-parse --show-toplevel 2>&1)
    if ($LASTEXITCODE -ne 0 -or $topLevel.Count -ne 1 -or
        [IO.Path]::GetFullPath([string]$topLevel[0]) -cne [IO.Path]::GetFullPath($repoRoot)) {
        throw "Sprint 8A preflight is not running against its exact repository worktree."
    }
    $branch = (@(& git -C $repoRoot branch --show-current 2>&1) -join "").Trim()
    if ($LASTEXITCODE -ne 0 -or [string]$branch -cne $ExpectedBranch) {
        throw "Sprint 8A preflight branch '$branch' does not equal intended branch '$ExpectedBranch'."
    }
    $headParents = ((@(& git -C $repoRoot rev-list --parents -n 1 HEAD 2>&1) -join " ").Trim() -split '\s+')
    if ($LASTEXITCODE -ne 0 -or $headParents.Count -ne 2) {
        throw "Sprint 8A preflight requires one ordinary non-merge implementation commit at HEAD."
    }
    $requiredTracked = @(
        "AGENTS.md",
        "docs/roadmap.md",
        "docs/sprints/sprint-8a-plan.md",
        "docs/sprints/sprint-8a-verification.md",
        "scripts/sprint-8a-lifecycle-chain.ps1",
        "scripts/sprint-8a-validation-environment.ps1",
        "scripts/run-sprint-8a-validation-preflight.ps1",
        "scripts/run-sprint-8a-sit.ps1",
        "scripts/run-sprint-8a-formal-uat.ps1"
    )
    $tracked = @(& git -C $repoRoot ls-files -- @requiredTracked)
    if ($LASTEXITCODE -ne 0 -or
        (@($tracked | Sort-Object) -join "`n") -cne (@($requiredTracked | Sort-Object) -join "`n")) {
        throw "Sprint 8A repository instructions, plan, lifecycle, or preflight runner are not all tracked."
    }
    $instructions = Get-Content -LiteralPath (Join-Path $repoRoot "AGENTS.md") -Raw
    $roadmap = Get-Content -LiteralPath (Join-Path $repoRoot "docs/roadmap.md") -Raw
    if (-not $instructions.Contains('repository-local `tessara-implementation` skill') -or
        -not $instructions.Contains("Treat Tessara as pre-production") -or
        -not $roadmap.Contains("Sprint 8A") -or
        -not $roadmap.Contains("Component Module Separation")) {
        throw "Sprint 8A repository instructions or roadmap scope are stale."
    }
    Publish-Sprint8APreflightStructuredEvidence -CheckName "repository-scope" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        repository_root = [IO.Path]::GetFullPath($repoRoot)
        branch = $branch
        implementation_commit = [string]$headParents[0]
        parent_commit = [string]$headParents[1]
        required_tracked_inputs = $requiredTracked
    }) | Out-Null
    "repository scope and one ordinary implementation commit confirmed"
}

function Assert-Sprint8APreflightCleanSource {
    $source = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $source -RequireClean | Out-Null
    if (-not (Test-Sprint8ASourceIdentityMatch `
            -Expected $script:runtimeContext.readiness.mutable_source_identity `
            -Actual $source) -or
        -not (Test-Sprint8ASourceIdentityMatch `
            -Expected $script:runtimeContext.rehearsal.mutable_source_identity `
            -Actual $source) -or
        [string]$source.branch -cne $ExpectedBranch) {
        throw "Current clean source differs from the exact passing Readiness/Rehearsal source identity."
    }
    $script:runtimeContext.source = $source
    $script:attemptReceipt.source_identity_verification_state = "verified"
    Publish-Sprint8APreflightStructuredEvidence -CheckName "clean-source" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        source_identity = $source
        exact_readiness_match = $true
        exact_rehearsal_match = $true
    }) | Out-Null
    "clean source identity exactly matches both passing mutable receipts"
}

function Assert-Sprint8APreflightAcceptanceTraceability {
    $planPath = Join-Path $repoRoot "docs/sprints/sprint-8a-plan.md"
    $verificationPath = Join-Path $repoRoot "docs/sprints/sprint-8a-verification.md"
    $manifestPath = Join-Path $repoRoot "end2end/acceptance-manifest.json"
    $plan = Get-Content -LiteralPath $planPath -Raw
    $verification = Get-Content -LiteralPath $verificationPath -Raw
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

    $acceptanceClauses = @(1..19 | ForEach-Object { "AC-{0:d2}" -f $_ })
    foreach ($clause in $acceptanceClauses) {
        if (-not $plan.Contains("**${clause}:**") -or -not $verification.Contains($clause)) {
            throw "Sprint 8A acceptance clause '$clause' is missing from the plan or verification traceability."
        }
    }
    if (($manifest.schema_version -isnot [int] -and $manifest.schema_version -isnot [long]) -or
        [int]$manifest.schema_version -ne 2 -or
        [int]$manifest.expected_total -ne 75) {
        throw "Sprint 8A Playwright acceptance manifest is not the exact schema-v2 75-scenario inventory."
    }
    $identities = @($manifest.files | ForEach-Object { @($_.tests) })
    if ($identities.Count -ne 75 -or @($identities | Sort-Object -Unique).Count -ne 75) {
        throw "Sprint 8A Playwright acceptance identities are incomplete or duplicated."
    }
    foreach ($file in @($manifest.files)) {
        $specPath = Join-Path $repoRoot "end2end/tests/$($file.path)"
        if (-not (Test-Path -LiteralPath $specPath -PathType Leaf)) {
            throw "Sprint 8A acceptance manifest names missing specification '$($file.path)'."
        }
        $specText = Get-Content -LiteralPath $specPath -Raw
        foreach ($identity in @($file.tests)) {
            if (-not $specText.Contains([string]$identity)) {
                throw "Sprint 8A acceptance identity '$identity' is not present in '$($file.path)'."
            }
        }
    }

    $scopeStart = $verification.IndexOf("## Scope And Acceptance Inventory", [StringComparison]::Ordinal)
    $scopeEnd = $verification.IndexOf("## Required Evidence Inventory", [StringComparison]::Ordinal)
    if ($scopeStart -lt 0 -or $scopeEnd -le $scopeStart) {
        throw "Sprint 8A verification lacks its frozen scope-and-acceptance inventory."
    }
    $scopeInventory = $verification.Substring($scopeStart, $scopeEnd - $scopeStart)
    $scopeRows = @($scopeInventory -split "`r?`n" | Where-Object {
        $_ -match '^\| .+ \| .+ \| .+ \| .+ \| UAT-8A-' -and $_ -notmatch '^\|---'
    })
    if ($scopeRows.Count -lt 28 -or $scopeInventory -match '(?i)\|\s*N/A\s*\|') {
        throw "Sprint 8A scope inventory does not retain automated, smoke, and manual proof for every row."
    }

    $uatContracts = @(Get-Sprint8AManualUatScenarioNames | ForEach-Object {
        $scenario = [string]$_
        $contract = Get-Sprint8AManualUatScenarioContract -Scenario $scenario
        if (-not $verification.Contains($scenario) -or
            -not (Test-Path -LiteralPath (Join-Path $repoRoot $contract.document.path) -PathType Leaf)) {
            throw "Sprint 8A manual UAT contract '$scenario' is missing from verification or source."
        }
        $contract
    })
    foreach ($requiredRunner in @(
        "scripts/smoke-sprint-8a.ps1",
        "scripts/audit-sprint-8a-deployed-inventory.ps1",
        "scripts/uat-sprint-8a.ps1"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $repoRoot $requiredRunner) -PathType Leaf)) {
            throw "Sprint 8A acceptance proof runner '$requiredRunner' is missing."
        }
    }

    Publish-Sprint8APreflightStructuredEvidence -CheckName "acceptance-traceability" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-acceptance-traceability"
        acceptance_clauses = $acceptanceClauses
        scope_inventory_rows = $scopeRows.Count
        playwright = [pscustomobject][ordered]@{
            manifest_path = "end2end/acceptance-manifest.json"
            manifest_sha256 = Get-Sprint8APreflightFileSha256 -Path $manifestPath
            identity_count = $identities.Count
        }
        manual_uat = @($uatContracts | ForEach-Object {
            [pscustomobject]@{
                scenario = $_.scenario
                role = $_.role
                step_count = $_.step_count
                document = $_.document
            }
        })
        smoke_runner = "scripts/smoke-sprint-8a.ps1"
    }) | Out-Null
    "acceptance traceability retains 19 clauses, 75 browser identities, smoke, and eight manual scenarios"
}

function Assert-Sprint8APreflightEnvironmentContract {
    $stored = $script:runtimeContext.environment
    $currentProbe = Get-Sprint8ADeploymentEnvironmentProbe `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot
    $currentToolchain = Get-Sprint8AToolchainEnvironmentContract -RepositoryRoot $repoRoot

    if ([string]$stored.deployment_probe_fingerprint -notmatch '^[0-9a-f]{64}$' -or
        [string]$stored.deployment_probe_fingerprint -cne [string]$currentProbe.fingerprint -or
        (ConvertTo-Sprint8APreflightCanonicalJson $currentProbe.contract.operating_system) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $stored.contract.operating_system) -or
        (ConvertTo-Sprint8APreflightCanonicalJson $currentProbe.contract.compose) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $stored.contract.compose) -or
        (ConvertTo-Sprint8APreflightCanonicalJson $currentProbe.contract.endpoints) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $stored.contract.endpoints) -or
        (ConvertTo-Sprint8APreflightCanonicalJson @($currentProbe.contract.fixture_identities)) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson @($stored.contract.fixture_identities)) -or
        (ConvertTo-Sprint8APreflightCanonicalJson $currentToolchain) -cne
            (ConvertTo-Sprint8APreflightCanonicalJson $stored.contract.toolchain) -or
        -not [bool]$currentProbe.contract.reset_authorization_present -or
        -not [bool]$stored.contract.reset_authorization_present) {
        throw "Current non-secret toolchain, Compose, endpoint, fixture, OS, or reset contract differs from passing Readiness."
    }
    if ([string]$stored.contract.endpoints.gateway -cne $HandoffUrl -or
        [string]$stored.contract.endpoints.supervisor -cne "http://127.0.0.1:8098" -or
        [string]$stored.contract.endpoints.materialization_control -cne "http://127.0.0.1:18088" -or
        [string]$stored.contract.compose.project -cne "tessara-sprint-8a" -or
        [string]$stored.contract.compose.profile -cne "reference" -or
        [string]$stored.contract.evidence.root.Replace("\", "/").TrimEnd("/") -cne $evidenceRootRelative -or
        [string]$stored.contract.evidence.output_mode -cne "attempt_scoped_append_only") {
        throw "Sprint 8A handoff, profile, endpoint, or evidence-root contract is not canonical."
    }

    $expectedDatabaseVariables = @(
        "TEST_API_DATABASE_URL",
        "TEST_API_FRESH_DATABASE_URL",
        "TEST_REFERENCE_MODULE_DATABASE_URL",
        "TEST_COMPONENT_MODULE_DATABASE_URL",
        "TEST_API_ENROLLMENT_DATABASE_URL",
        "TEST_INSTALLATION_CONTROL_DATABASE_URL"
    )
    if (@($stored.contract.databases).Count -ne 6 -or @($currentProbe.contract.databases).Count -ne 6) {
        throw "Sprint 8A environment does not retain exactly six database bindings."
    }
    foreach ($variable in $expectedDatabaseVariables) {
        $expectedBinding = @($stored.contract.databases | Where-Object variable -CEQ $variable)
        $actualBinding = @($currentProbe.contract.databases | Where-Object variable -CEQ $variable)
        if ($expectedBinding.Count -ne 1 -or $actualBinding.Count -ne 1) {
            throw "Sprint 8A database variable '$variable' is missing or duplicated."
        }
        foreach ($field in @("variable", "value_sha256", "host", "port", "database", "role", "canonical_server")) {
            if ([string]$expectedBinding[0].$field -cne [string]$actualBinding[0].$field) {
                throw "Current database binding '$variable' differs from passing Readiness field '$field'."
            }
        }
        if ($null -eq $expectedBinding[0].authenticated_probe -or
            -not [bool]$expectedBinding[0].authenticated_probe.transaction_round_trip -or
            [string]$expectedBinding[0].authenticated_probe.database -cne [string]$expectedBinding[0].database -or
            [string]$expectedBinding[0].authenticated_probe.role -cne [string]$expectedBinding[0].role) {
            throw "Passing Readiness did not retain an authenticated probe for '$variable'."
        }
    }

    $composePath = Join-Path $repoRoot "deploy/sprint-8a/compose.yaml"
    $composeOutput = @(& docker compose -f $composePath --profile reference config --format json 2>&1)
    if ($LASTEXITCODE -ne 0 -or $composeOutput.Count -eq 0) {
        throw "Sprint 8A preflight could not normalize the exact Compose profile."
    }
    $compose = ($composeOutput -join "`n") | ConvertFrom-Json
    $projection = Get-Sprint8AComposeServiceProjection -Services $compose.services
    $expectedPorts = @(
        "core|127.0.0.1|18088|8080",
        "gateway|127.0.0.1|8088|8080",
        "supervisor|127.0.0.1|8098|8090"
    )
    $actualPorts = @($projection.ports | ForEach-Object {
        "$($_.service)|$($_.host_ip)|$($_.published)|$($_.target)"
    } | Sort-Object)
    if (($actualPorts -join "`n") -cne (($expectedPorts | Sort-Object) -join "`n")) {
        throw "Sprint 8A normalized Compose profile exposes a stale required-port inventory."
    }
    Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath `
        -Path "$evidenceRootRelative/preflight-result.json" | Out-Null
    $script:runtimeContext.normalized_deployment_configuration_sha256 = [string]$currentProbe.contract.compose.normalized_config_sha256
    $script:attemptReceipt.environment_verification_state = "verified"

    Publish-Sprint8APreflightStructuredEvidence -CheckName "environment-contract" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-environment-reconfirmation"
        environment_fingerprint = [string]$stored.fingerprint
        deployment_probe_fingerprint = [string]$currentProbe.fingerprint
        database_binding_source = "scripts/sprint-8a-validation-environment.ps1:Get-Sprint8ADeploymentEnvironmentProbe"
        toolchain = $currentToolchain
        database_identities = @($currentProbe.contract.databases | ForEach-Object {
            [pscustomobject]@{
                variable = $_.variable
                value_sha256 = $_.value_sha256
                canonical_server = $_.canonical_server
                database = $_.database
                role = $_.role
            }
        })
        compose = $currentProbe.contract.compose
        ports = $projection.ports
        endpoints = $currentProbe.contract.endpoints
        evidence = $currentProbe.contract.evidence
    }) | Out-Null
    "environment fingerprint and all non-secret reconfirmation fields match passing Readiness"
}

function Invoke-Sprint8APreflightSqlLines {
    param(
        [Parameter(Mandatory)]$Binding,
        [Parameter(Mandatory)][string]$Sql
    )

    $containerId = [Environment]::GetEnvironmentVariable("TEST_POSTGRES_CLIENT_CONTAINER_ID")
    $output = @()
    if (-not [string]::IsNullOrWhiteSpace($containerId)) {
        $inspectOutput = @(& docker inspect $containerId 2>&1)
        if ($LASTEXITCODE -ne 0 -or $inspectOutput.Count -eq 0) {
            throw "TEST_POSTGRES_CLIENT_CONTAINER_ID does not identify an inspectable preflight client."
        }
        $inspect = @($inspectOutput | ConvertFrom-Json)[0]
        if (-not [bool]$inspect.State.Running) {
            throw "The approved PostgreSQL preflight client is not running."
        }
        $arguments = @("exec")
        if (-not [string]::IsNullOrEmpty([string]$Binding.password)) {
            $arguments += @("-e", "PGPASSWORD=$($Binding.password)")
        }
        $arguments += @(
            [string]$inspect.Id, "psql", "-X", "-v", "ON_ERROR_STOP=1", "-At",
            "-h", "127.0.0.1", "-p", "5432", "-U", [string]$Binding.role,
            "-d", [string]$Binding.database, "-c", $Sql
        )
        $output = @(& docker @arguments 2>&1 | ForEach-Object { [string]$_ })
    } else {
        if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
            throw "Sprint 8A database preflight requires psql or TEST_POSTGRES_CLIENT_CONTAINER_ID."
        }
        $priorPassword = [Environment]::GetEnvironmentVariable("PGPASSWORD", "Process")
        try {
            [Environment]::SetEnvironmentVariable("PGPASSWORD", [string]$Binding.password, "Process")
            $output = @(& psql -X -v ON_ERROR_STOP=1 -At `
                -h ([string]$Binding.host) -p ([string]$Binding.port) `
                -U ([string]$Binding.role) -d ([string]$Binding.database) `
                -c $Sql 2>&1 | ForEach-Object { [string]$_ })
        } finally {
            [Environment]::SetEnvironmentVariable("PGPASSWORD", $priorPassword, "Process")
        }
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Sprint 8A authenticated SQL preflight failed for $($Binding.name)."
    }
    @($output | Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) })
}

function Assert-Sprint8APreflightDatabaseContract {
    $variables = @(
        "TEST_API_DATABASE_URL",
        "TEST_API_FRESH_DATABASE_URL",
        "TEST_REFERENCE_MODULE_DATABASE_URL",
        "TEST_COMPONENT_MODULE_DATABASE_URL",
        "TEST_API_ENROLLMENT_DATABASE_URL",
        "TEST_INSTALLATION_CONTROL_DATABASE_URL"
    )
    $bindings = @($variables | ForEach-Object {
        $value = [Environment]::GetEnvironmentVariable($_)
        if ([string]::IsNullOrWhiteSpace($value)) {
            throw "Sprint 8A database preflight is missing exact variable '$_'."
        }
        ConvertTo-Sprint8ADatabaseBinding -Name $_ -Value $value
    })
    if (@($bindings.identity | Sort-Object -Unique).Count -ne 6) {
        throw "Sprint 8A database preflight identities are not pairwise distinct."
    }
    $results = @($bindings | ForEach-Object {
        $binding = $_
        $probe = Invoke-Sprint8ADatabaseProbe `
            -Binding $binding `
            -PostgresContainerId ([Environment]::GetEnvironmentVariable("TEST_POSTGRES_CLIENT_CONTAINER_ID"))
        $ledger = @(Invoke-Sprint8APreflightSqlLines `
            -Binding $binding `
            -Sql "SELECT version::text || '|' || success::text FROM _sqlx_migrations ORDER BY version;")
        if (($ledger -join ",") -cne "1|true") {
            throw "Sprint 8A database '$($binding.name)' does not retain the exact successful squashed baseline ledger."
        }
        [pscustomobject][ordered]@{
            variable = [string]$binding.name
            canonical_server = [string]$binding.canonical_server
            database = [string]$binding.database
            role = [string]$binding.role
            transaction_round_trip = [bool]$probe.transaction_round_trip
            client = $probe.client
            migration_ledger = $ledger
        }
    })
    Publish-Sprint8APreflightStructuredEvidence -CheckName "database-contract" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-database-reconfirmation"
        pairwise_distinct = $true
        databases = $results
    }) | Out-Null
    "all six disposable databases are distinct, reachable, and on exact successful baseline 1"
}

function Get-Sprint8APreflightPropertyValue {
    param(
        [AllowNull()]$InputObject,
        [Parameter(Mandatory)][string]$Name
    )

    if ($null -eq $InputObject) { return $null }
    $property = $InputObject.PSObject.Properties[$Name]
    if ($null -eq $property) { return $null }
    $property.Value
}

function Assert-Sprint8APreflightRunnerParameters {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string[]]$Required
    )

    $actual = @(Get-Sprint8AScriptParameterNames -Path (Join-Path $repoRoot $Path))
    $missing = @($Required | Where-Object { $actual -cnotcontains $_ })
    if ($missing.Count -ne 0) {
        throw "Sprint 8A runner '$Path' is missing required parameter(s): $($missing -join ', ')."
    }
    [pscustomobject][ordered]@{
        path = $Path
        required = $Required
        declared = $actual
    }
}

function Assert-Sprint8APreflightDeploymentContract {
    $composePath = Join-Path $repoRoot "deploy/sprint-8a/compose.yaml"
    $source = $script:runtimeContext.source
    if ($null -eq $source -or $null -eq $script:runtimeContext.environment) {
        throw "Sprint 8A deployment preflight requires verified clean source and environment identities."
    }

    $composeOutput = @(& docker compose -f $composePath --profile reference config --format json 2>&1)
    if ($LASTEXITCODE -ne 0 -or $composeOutput.Count -eq 0) {
        throw "Sprint 8A deployment preflight cannot normalize Compose without building images."
    }
    $composeText = $composeOutput -join "`n"
    $compose = $composeText | ConvertFrom-Json
    $normalizedConfigSha256 = Get-Sprint8APreflightStringSha256 -Text $composeText
    if ([string]$compose.name -cne "tessara-sprint-8a") {
        throw "Sprint 8A deployment preflight resolved unexpected Compose project '$($compose.name)'."
    }
    if ([string]$script:runtimeContext.normalized_deployment_configuration_sha256 -notmatch '^[0-9a-f]{64}$' -or
        [string]$script:runtimeContext.normalized_deployment_configuration_sha256 -cne $normalizedConfigSha256) {
        throw "Sprint 8A deployment preflight normalized Compose digest differs from the verified environment contract."
    }

    $requiredServices = @(
        "components", "core", "dashboards", "gateway", "postgres", "scoped-records", "supervisor"
    )
    $configuredServices = @($compose.services.PSObject.Properties | ForEach-Object { [string]$_.Name })
    $missingServices = @($requiredServices | Where-Object { $configuredServices -cnotcontains $_ })
    if ($missingServices.Count -ne 0) {
        throw "Sprint 8A Compose configuration omits required service(s): $($missingServices -join ', ')."
    }

    $dockerfiles = [ordered]@{
        core = "Dockerfile"
        components = "Dockerfile.component"
        dashboards = "Dockerfile.dashboard"
        "scoped-records" = "Dockerfile.scoped-records"
        supervisor = "Dockerfile.supervisor"
    }
    $expectedBindings = [ordered]@{
        core = [pscustomobject]@{ target = "8080/tcp"; host_ip = "127.0.0.1"; host_port = "18088" }
        gateway = [pscustomobject]@{ target = "8080/tcp"; host_ip = "127.0.0.1"; host_port = "8088" }
        supervisor = [pscustomobject]@{ target = "8090/tcp"; host_ip = "127.0.0.1"; host_port = "8098" }
    }
    $expectedModuleLabels = [ordered]@{
        components = [ordered]@{
            "com.tessara.module-definition" = "tessara.components"
            "com.tessara.module-release" = "1.0.0"
        }
        dashboards = [ordered]@{
            "com.tessara.module-definition" = "tessara.dashboards"
            "com.tessara.module-release" = "3.0.0"
        }
    }
    $labelKeys = @(
        "org.opencontainers.image.revision",
        "com.tessara.source-tree",
        "com.tessara.source-dirty",
        "com.tessara.build-profile"
    )
    $dockerfileEvidence = @($dockerfiles.GetEnumerator() | ForEach-Object {
        $path = Join-Path $repoRoot ([string]$_.Value)
        $text = Get-Content -LiteralPath $path -Raw
        foreach ($fragment in @(
            'ARG TESSARA_SOURCE_COMMIT=unknown',
            'ARG TESSARA_SOURCE_TREE=unknown',
            'ARG TESSARA_SOURCE_DIRTY=unknown',
            'org.opencontainers.image.revision="$TESSARA_SOURCE_COMMIT"',
            'com.tessara.source-tree="$TESSARA_SOURCE_TREE"',
            'com.tessara.source-dirty="$TESSARA_SOURCE_DIRTY"',
            'com.tessara.build-profile="release"'
        )) {
            if (-not $text.Contains($fragment)) {
                throw "Sprint 8A Dockerfile '$($_.Value)' omits exact provenance fragment '$fragment'."
            }
        }
        [pscustomobject][ordered]@{
            service = [string]$_.Key
            path = [string]$_.Value
            sha256 = Get-Sprint8APreflightFileSha256 -Path $path
            label_keys = $labelKeys
        }
    })

    $runningServices = @(& docker compose -f $composePath --profile reference ps --status running --services 2>&1 |
        ForEach-Object { [string]$_ } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0 -or
        ($runningServices -join "`n") -cne (($requiredServices | Sort-Object) -join "`n")) {
        throw "Sprint 8A active slot must contain exactly the seven canonical running services."
    }

    $serviceEvidence = @($requiredServices | ForEach-Object {
        $service = $_
        $containerIds = @(& docker compose -f $composePath --profile reference ps -q $service 2>&1 |
            ForEach-Object { [string]$_ } |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        if ($LASTEXITCODE -ne 0 -or $containerIds.Count -ne 1) {
            throw "Sprint 8A active slot requires exactly one '$service' container."
        }
        $containerOutput = @(& docker inspect $containerIds[0] 2>&1)
        if ($LASTEXITCODE -ne 0 -or $containerOutput.Count -eq 0) {
            throw "Sprint 8A deployment preflight cannot inspect '$service'."
        }
        $container = @($containerOutput | ConvertFrom-Json)[0]
        if (-not [bool]$container.State.Running -or
            [string](Get-Sprint8APreflightPropertyValue -InputObject $container.Config.Labels -Name "com.docker.compose.project") -cne "tessara-sprint-8a" -or
            [string](Get-Sprint8APreflightPropertyValue -InputObject $container.Config.Labels -Name "com.docker.compose.service") -cne $service) {
            throw "Sprint 8A '$service' container is not the canonical running Compose owner."
        }
        $health = Get-Sprint8APreflightPropertyValue -InputObject $container.State -Name "Health"
        if ($null -ne $health -and [string]$health.Status -cne "healthy") {
            throw "Sprint 8A '$service' container is not healthy."
        }

        $publishedPorts = @()
        if ($expectedBindings.Contains($service)) {
            $binding = $expectedBindings[$service]
            $publishedPorts = @(Get-Sprint8APreflightPropertyValue `
                -InputObject $container.NetworkSettings.Ports `
                -Name ([string]$binding.target))
            if ($publishedPorts.Count -ne 1 -or
                [string]$publishedPorts[0].HostIp -cne [string]$binding.host_ip -or
                [string]$publishedPorts[0].HostPort -cne [string]$binding.host_port) {
                throw "Sprint 8A '$service' active port binding differs from the canonical loopback slot."
            }
        }

        $provenance = $null
        if ($dockerfiles.Contains($service)) {
            $imageOutput = @(& docker image inspect ([string]$container.Image) 2>&1)
            if ($LASTEXITCODE -ne 0 -or $imageOutput.Count -eq 0) {
                throw "Sprint 8A deployment preflight cannot inspect the '$service' image."
            }
            $image = @($imageOutput | ConvertFrom-Json)[0]
            $labels = $image.Config.Labels
            $expectedLabels = [ordered]@{
                "org.opencontainers.image.revision" = [string]$source.commit
                "com.tessara.source-tree" = [string]$source.tree
                "com.tessara.source-dirty" = "false"
                "com.tessara.build-profile" = "release"
            }
            if ($expectedModuleLabels.Contains($service)) {
                foreach ($moduleLabel in $expectedModuleLabels[$service].GetEnumerator()) {
                    $expectedLabels[[string]$moduleLabel.Key] = [string]$moduleLabel.Value
                }
            }
            foreach ($label in $expectedLabels.GetEnumerator()) {
                if ([string](Get-Sprint8APreflightPropertyValue -InputObject $labels -Name ([string]$label.Key)) -cne [string]$label.Value) {
                    throw "Sprint 8A '$service' image provenance label '$($label.Key)' does not match the verified source."
                }
            }
            $provenance = [pscustomobject]$expectedLabels
        }
        [pscustomobject][ordered]@{
            service = $service
            container_id = [string]$container.Id
            image_id = [string]$container.Image
            state = [string]$container.State.Status
            health = if ($null -eq $health) { "running_no_healthcheck" } else { [string]$health.Status }
            published_ports = @($publishedPorts | ForEach-Object {
                [pscustomobject][ordered]@{ host_ip = [string]$_.HostIp; host_port = [string]$_.HostPort }
            })
            provenance = $provenance
        }
    })

    $parameterContracts = @(
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/materialize-sprint-8a.ps1" -Required @(
            "Attempt", "EvidenceRoot", "EnvironmentFingerprint", "AuthorizeDisposableReset", "VerifyNoOp"
        )
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-failure-containment.ps1" -Required @(
            "Attempt", "EvidenceRoot", "EnvironmentFingerprint", "OutputPath", "AuthorizeDisposableReset", "SkipBuild"
        )
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-component-upgrade.ps1" -Required @("OutputPath")
    )
    $verificationText = Get-Content -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8a-verification.md") -Raw
    $lifecycleCommands = @(
        '.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp',
        '.\scripts\run-sprint-8a-failure-containment.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/failure-containment.json" -AuthorizeDisposableReset -SkipBuild',
        '.\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/component-upgrade.json"'
    )
    foreach ($command in $lifecycleCommands) {
        if (-not $verificationText.Contains($command)) {
            throw "Sprint 8A verification no longer freezes canonical lifecycle command '$command'."
        }
    }

    $candidateIdentity = Get-Sprint8ACandidateIdentity `
        -RepositoryRoot $repoRoot `
        -Source $source `
        -NormalizedDeploymentConfigurationSha256 $normalizedConfigSha256
    $script:runtimeContext.candidate_identity = $candidateIdentity
    Publish-Sprint8APreflightStructuredEvidence -CheckName "deployment-contract" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-deployment-reconfirmation"
        project = "tessara-sprint-8a"
        profile = "reference"
        normalized_config_sha256 = $normalizedConfigSha256
        candidate_fingerprint = [string]$candidateIdentity.fingerprint
        required_services = $requiredServices
        running_services = $serviceEvidence
        dockerfiles = $dockerfileEvidence
        lifecycle_commands = $lifecycleCommands
        runner_parameters = $parameterContracts
        canonical_restoration = [pscustomobject][ordered]@{
            rehearsal_cleanup_result = [string]$script:runtimeContext.rehearsal.cleanup_restoration.result
            active_slot_verified = $true
            source_exact_provenance_verified = $true
        }
        builds_executed = $false
    }) | Out-Null
    "Compose, exact source provenance, canonical active slot, and non-building lifecycle command contracts pass"
}

function Get-Sprint8ADownstreamCommandSets {
    [pscustomobject][ordered]@{
        static_and_boundaries = @(
            "cargo fmt --all -- --check",
            "cargo check --workspace --all-features --locked --offline",
            "cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings",
            "docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config --quiet",
            ". .\scripts\sprint-8a-acceptance-contract.ps1",
            "Test-Sprint8AAcceptanceContract",
            ".\scripts\check-web-crate-boundaries.ps1",
            ".\scripts\verify-module-sdk-boundaries.ps1",
            ".\scripts\verify-sprint-6e-boundaries.ps1",
            ".\scripts\verify-markdown-links.ps1"
        )
        rust_workspace = @(
            "cargo test --workspace --all-features --locked --offline",
            "cargo test -p tessara-api --test modules --release --locked --offline resource_reference_restricted_known_random_latency_profile -- --exact --nocapture",
            "cargo test --locked --offline -p tessara-components-contract",
            "cargo test --locked --offline -p tessara-dashboard-module",
            "cargo test --locked --offline -p tessara-component-module",
            "cargo test --locked --offline -p tessara-module-testkit"
        )
        playwright = @(
            '.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/playwright-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp',
            '.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/deployment.json"',
            '.\scripts\validate-e2e.ps1 -BaseUrl "http://127.0.0.1:8088" -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/deployment.json" -ExpectedDataState fresh -TransitionCatalogProfile sprint-8a -EvidencePath "artifacts/sprint-8a-closeout/sit/playwright-<n>/playwright.json" -FailureEvidenceDirectory "artifacts/sprint-8a-closeout/sit/playwright-<n>/failures"'
        )
        deployed_acceptance_smoke = @(
            '.\scripts\materialize-sprint-8a.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -AuthorizeDisposableReset -Confirm:$false -VerifyNoOp',
            '.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-inventory.json"',
            '.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-deployment.json"',
            '.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/initial-smoke.json"',
            '.\scripts\run-sprint-8a-component-upgrade.ps1 -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/component-upgrade.json"',
            '.\scripts\run-sprint-8a-failure-containment.ps1 -Attempt <n> -EvidenceRoot "artifacts/sprint-8a-closeout/sit/attempt-<n>" -EnvironmentFingerprint <environment-fingerprint> -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/failure-containment.json" -AuthorizeDisposableReset -SkipBuild',
            '.\scripts\audit-sprint-8a-deployed-inventory.ps1 -BaseUrl "http://127.0.0.1:8088" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-inventory.json"',
            '.\scripts\run-sprint-8a-deployed-smoke.ps1 -DeploymentEvidencePath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-deployment.json"',
            '.\scripts\smoke-sprint-8a.ps1 -BaseUrl "http://127.0.0.1:8088" -SupervisorUrl "http://127.0.0.1:8098" -OutputPath "artifacts/sprint-8a-closeout/sit/attempt-<n>/restored-smoke.json"'
        )
        formal_uat = @(
            '.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Start -Attempt <n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088"',
            '.\scripts\run-sprint-8a-formal-uat.ps1 -Stage Finalize -Attempt <same-n> -PreflightReceipt "artifacts/sprint-8a-closeout/preflight-result.json" -CandidateReceipt "artifacts/sprint-8a-closeout/candidate.json" -SitReceipt "artifacts/sprint-8a-closeout/sit-result.json" -EvidenceRoot "artifacts/sprint-8a-closeout" -OutputPath "artifacts/sprint-8a-closeout/uat-result.json" -BaseUrl "http://127.0.0.1:8088" -AuthorizeDisposableReset'
        )
    }
}

function Assert-Sprint8APreflightOrderedCommandSet {
    param(
        [Parameter(Mandatory)][string]$Document,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string[]]$Commands
    )

    $cursor = -1
    foreach ($command in $Commands) {
        $index = $Document.IndexOf($command, $cursor + 1, [StringComparison]::Ordinal)
        if ($index -lt 0) {
            throw "Sprint 8A verification omits or reorders '$Name' command '$command'."
        }
        $cursor = $index
    }
}

function Assert-Sprint8APreflightDownstreamCommands {
    $verification = Get-Content -LiteralPath (Join-Path $repoRoot "docs/sprints/sprint-8a-verification.md") -Raw
    $sets = Get-Sprint8ADownstreamCommandSets
    Assert-Sprint8APreflightOrderedCommandSet -Document $verification -Name "static-and-boundaries" -Commands @($sets.static_and_boundaries)
    Assert-Sprint8APreflightOrderedCommandSet -Document $verification -Name "rust-workspace" -Commands @($sets.rust_workspace)
    Assert-Sprint8APreflightOrderedCommandSet -Document $verification -Name "playwright" -Commands @($sets.playwright)
    Assert-Sprint8APreflightOrderedCommandSet -Document $verification -Name "deployed-acceptance-smoke" -Commands @($sets.deployed_acceptance_smoke)
    Assert-Sprint8APreflightOrderedCommandSet -Document $verification -Name "formal-uat" -Commands @($sets.formal_uat)

    $scriptPaths = @(
        "scripts/sprint-8a-acceptance-contract.ps1",
        "scripts/check-web-crate-boundaries.ps1",
        "scripts/verify-module-sdk-boundaries.ps1",
        "scripts/verify-sprint-6e-boundaries.ps1",
        "scripts/verify-markdown-links.ps1"
    )
    $staticScripts = @($scriptPaths | ForEach-Object {
        $path = Join-Path $repoRoot $_
        $tokens = $null
        $errors = $null
        [void][Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors)
        if (@($errors).Count -ne 0) {
            throw "Sprint 8A static lane runner '$_' does not parse."
        }
        [pscustomobject][ordered]@{ path = $_; sha256 = Get-Sprint8APreflightFileSha256 -Path $path }
    })

    $parameterContracts = @(
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/materialize-sprint-8a.ps1" -Required @(
            "Attempt", "EvidenceRoot", "EnvironmentFingerprint", "AuthorizeDisposableReset", "VerifyNoOp"
        )
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-deployed-smoke.ps1" -Required @("DeploymentEvidencePath")
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/validate-e2e.ps1" -Required @(
            "BaseUrl", "DeploymentEvidencePath", "ExpectedDataState", "TransitionCatalogProfile", "EvidencePath", "FailureEvidenceDirectory"
        )
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/audit-sprint-8a-deployed-inventory.ps1" -Required @("BaseUrl", "OutputPath")
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/smoke-sprint-8a.ps1" -Required @("BaseUrl", "SupervisorUrl", "OutputPath")
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-component-upgrade.ps1" -Required @("OutputPath")
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-failure-containment.ps1" -Required @(
            "Attempt", "EvidenceRoot", "EnvironmentFingerprint", "OutputPath", "AuthorizeDisposableReset", "SkipBuild"
        )
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-sit.ps1" -Required @(
            "Stage", "Attempt", "PreflightReceipt", "CandidateReceipt", "EvidenceRoot", "OutputPath", "BaseUrl", "AuthorizeDisposableReset", "SelfTest"
        )
        Assert-Sprint8APreflightRunnerParameters -Path "scripts/run-sprint-8a-formal-uat.ps1" -Required @(
            "Stage", "Attempt", "PreflightReceipt", "CandidateReceipt", "SitReceipt", "EvidenceRoot", "OutputPath", "BaseUrl", "AuthorizeDisposableReset", "SelfTest"
        )
    )
    $materializeText = Get-Content -LiteralPath (Join-Path $repoRoot "scripts/materialize-sprint-8a.ps1") -Raw
    if (-not $materializeText.Contains('[CmdletBinding(SupportsShouldProcess, ConfirmImpact = "High")]')) {
        throw "Sprint 8A materialization no longer accepts the documented -Confirm common parameter."
    }

    $sitCoordinatorPath = Join-Path $repoRoot "scripts/run-sprint-8a-sit.ps1"
    $sitCoordinator = Get-Content -LiteralPath $sitCoordinatorPath -Raw
    $sitCoordinatorFragments = @(
        '[ValidateSet("Run", "Finalize")]',
        '[switch]$AuthorizeDisposableReset',
        '[switch]$SelfTest',
        'sprint-8a-lifecycle-chain.ps1',
        'Open-Sprint8AValidationAttemptLock',
        'validation-attempt.lock',
        'attempts/sit-$Attempt.json',
        'Get-Sprint8ASitLaneNames',
        '"static-and-boundaries"',
        '"rust-workspace"',
        '"playwright"',
        '"deployed-acceptance-smoke"',
        'Get-Sprint8AEvidenceFileManifestEntries',
        'Assert-Sprint8AEvidenceManifestCompleteness',
        'Publish-Sprint8ALifecycleReceipt',
        'canonical_topology_verified',
        'finalization_only_retry_permitted_if_identity_remains_exact'
    )
    foreach ($fragment in $sitCoordinatorFragments) {
        if (-not $sitCoordinator.Contains($fragment)) {
            throw "Sprint 8A SIT coordinator interface omits '$fragment'."
        }
    }

    $formalUatPath = Join-Path $repoRoot "scripts/run-sprint-8a-formal-uat.ps1"
    $formalUat = Get-Content -LiteralPath $formalUatPath -Raw
    foreach ($fragment in @(
        '[ValidateSet("Start", "Finalize")]',
        'Publish-Sprint8ALifecycleReceipt',
        '-PrepareOnly',
        'Publish-Sprint8AEvidenceManifest',
        'attempts/uat-$Attempt-manual-checkpoint.json'
    )) {
        if (-not $formalUat.Contains($fragment)) {
            throw "Sprint 8A formal UAT interface omits '$fragment'."
        }
    }

    $cargoToml = Get-Content -LiteralPath (Join-Path $repoRoot "Cargo.toml") -Raw
    foreach ($package in @(
        "tessara-api", "tessara-components-contract", "tessara-dashboard-module",
        "tessara-component-module", "tessara-module-testkit"
    )) {
        if (-not $cargoToml.Contains($package)) {
            throw "Sprint 8A Rust lane command references missing workspace package '$package'."
        }
    }

    Publish-Sprint8APreflightStructuredEvidence -CheckName "downstream-command-contract" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-downstream-commands"
        command_sets = $sets
        parameter_contracts = $parameterContracts
        coordinator_interfaces = [pscustomobject][ordered]@{
            sit = [pscustomobject][ordered]@{
                path = "scripts/run-sprint-8a-sit.ps1"
                required_fragments = $sitCoordinatorFragments
            }
            uat = [pscustomobject][ordered]@{
                path = "scripts/run-sprint-8a-formal-uat.ps1"
                staged = @("Start", "Finalize")
            }
        }
        static_scripts = $staticScripts
        parsed_only = $true
        sit_or_uat_executed = $false
    }) | Out-Null
    "all four exact SIT command sets and staged formal-UAT interface parse without execution"
}

function Assert-Sprint8APreflightEvidencePaths {
    $expectedReadiness = "$evidenceRootRelative/validation-readiness-result.json"
    $expectedRehearsal = "$evidenceRootRelative/candidate-rehearsal-result.json"
    $resolvedReadiness = Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath `
        -Path $ReadinessReceipt
    $resolvedRehearsal = Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath `
        -Path $RehearsalReceipt
    if ([string]$resolvedReadiness.path -cne $expectedReadiness -or
        [string]$resolvedRehearsal.path -cne $expectedRehearsal) {
        throw "Sprint 8A preflight must consume the canonical Readiness and Rehearsal receipts."
    }

    $probeRelative = "$evidenceRootRelative/preflight/path-contract/probe.json"
    $probe = Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath `
        -Path $probeRelative
    if ([string]$probe.path -cne $probeRelative) {
        throw "Sprint 8A contained evidence-path normalization is not stable."
    }
    $rejectionCases = @(
        [pscustomobject]@{ kind = "traversal"; path = "$evidenceRootRelative/../outside.json"; allow_absolute = $false },
        [pscustomobject]@{ kind = "unsupported_absolute"; path = (Join-Path $script:evidenceRootPath "absolute.json"); allow_absolute = $false },
        [pscustomobject]@{ kind = "outside_root_absolute"; path = (Join-Path $repoRoot "outside.json"); allow_absolute = $true }
    )
    foreach ($case in $rejectionCases) {
        $rejected = $false
        try {
            Resolve-Sprint8APreflightEvidencePath `
                -RepositoryRoot $repoRoot `
                -EvidenceRootPath $script:evidenceRootPath `
                -Path ([string]$case.path) `
                -AllowInRootAbsolute:([bool]$case.allow_absolute) | Out-Null
        } catch {
            $rejected = $true
        }
        if (-not $rejected) {
            throw "Sprint 8A evidence-path contract accepted '$($case.kind)'."
        }
    }

    $reserved = @(
        $preflightResultPath,
        "$preflightResultPath.sha256",
        $candidatePath,
        "$candidatePath.sha256",
        $manifestPath,
        "$manifestPath.sha256",
        (Join-Path $script:evidenceRootPath "sit-result.json"),
        (Join-Path $script:evidenceRootPath "sit-result.json.sha256"),
        (Join-Path $script:evidenceRootPath "uat-result.json"),
        (Join-Path $script:evidenceRootPath "uat-result.json.sha256"),
        (Join-Path $script:evidenceRootPath "closeout-authorization.json"),
        (Join-Path $script:evidenceRootPath "closeout-authorization.json.sha256")
    )
    $collisions = @($reserved | Where-Object { Test-Path -LiteralPath $_ })
    if ($collisions.Count -ne 0) {
        throw "Sprint 8A immutable lifecycle output collision(s): $($collisions -join ', ')."
    }

    $legacyException = $null
    if ($null -ne $script:runtimeContext.correction_authorization_reference) {
        $legacyException = [pscustomobject][ordered]@{
            kind = "canonical_consumption_of_current_correction_lineage_tip"
            path = [string]$script:runtimeContext.correction_authorization_reference.path
            sha256 = [string]$script:runtimeContext.correction_authorization_reference.sha256
            authenticated = $true
        }
    }
    Publish-Sprint8APreflightStructuredEvidence -CheckName "evidence-path-contract" -Document ([pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-evidence-paths"
        evidence_root = $evidenceRootRelative
        normalized_inputs = @([string]$resolvedReadiness.path, [string]$resolvedRehearsal.path, [string]$probe.path)
        rejected_cases = @($rejectionCases | ForEach-Object { [string]$_.kind })
        immutable_reserved_paths = @($reserved | ForEach-Object {
            [IO.Path]::GetRelativePath($repoRoot, $_).Replace("\", "/")
        })
        collision_count = 0
        absolute_path_exception = $legacyException
    }) | Out-Null
    "canonical paths are contained, unsupported paths rejected, and immutable freeze outputs unused"
}

function Publish-Sprint8APreflightInventory {
    $freezeSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    if (-not (Test-Sprint8ASourceIdentityMatch `
            -Expected $script:runtimeContext.source `
            -Actual $freezeSource)) {
        $exception = [InvalidOperationException]::new(
            "Sprint 8A source changed after clean-source and before freeze-boundary inventory."
        )
        $exception.Data["Sprint8AClassification"] = "preflight/setup"
        throw $exception
    }
    $freezeEnvironment = Get-Sprint8AEnvironmentContract `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot
    if ([string]$freezeEnvironment.fingerprint -cne [string]$script:runtimeContext.environment.fingerprint) {
        $exception = [InvalidOperationException]::new(
            "Sprint 8A environment changed after environment-contract and before freeze-boundary inventory."
        )
        $exception.Data["Sprint8AClassification"] = "environment"
        throw $exception
    }

    $planned = Get-Sprint8APlannedEvidenceInventory -PreflightAttempt $Attempt
    if (@($planned.required.path | Sort-Object -Unique).Count -ne @($planned.required).Count -or
        @($planned.sit_lanes).Count -ne 4 -or
        @($planned.manual_uat).Count -ne 8 -or
        @($planned.manual_contract_sources).Count -ne 9 -or
        @($planned.required.path | Where-Object { $_ -cmatch '\{01\.\.08\}' }).Count -ne 0) {
        throw "Sprint 8A planned evidence inventory is incomplete."
    }
    $existing = @(Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath)
    $document = [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8a"
        contract = "tessara.sprint-8a.preflight-evidence-inventory"
        generated_at = [DateTimeOffset]::UtcNow.ToString("o")
        preflight_attempt = $Attempt
        mutable_source_identity = $script:runtimeContext.source
        environment_fingerprint = [string]$script:runtimeContext.environment.fingerprint
        normalized_deployment_configuration_sha256 = [string]$script:runtimeContext.normalized_deployment_configuration_sha256
        freeze_boundary_reconfirmation = [pscustomobject][ordered]@{
            source_exact = $true
            environment_exact = $true
        }
        prerequisite_receipts = @(
            [pscustomobject][ordered]@{
                path = [string]$script:runtimeContext.readiness_reference.path
                sha256 = [string]$script:runtimeContext.readiness_reference.sha256
            }
            [pscustomobject][ordered]@{
                path = [string]$script:runtimeContext.rehearsal_reference.path
                sha256 = [string]$script:runtimeContext.rehearsal_reference.sha256
            }
        )
        existing_paths_at_declaration = @($existing | ForEach-Object { [string]$_.path })
        future_and_conditional = [pscustomobject][ordered]@{
            required_paths = $planned.required
            namespaces = $planned.namespaces
            sit_lanes = $planned.sit_lanes
            manual_uat_scenarios = $planned.manual_uat
        }
        downstream_command_sets = Get-Sprint8ADownstreamCommandSets
        rules = [pscustomobject][ordered]@{
            existing_files_are_manifested_only_after_freeze_publication = $true
            declaration_does_not_embed_mutable_preterminal_hashes = $true
            future_paths_are_declarations_not_manifest_entries = $true
            each_lifecycle_receipt_requires_sha256_sidecar = $true
            mutable_attempt_checkpoints_are_overwritten_only_by_the_owning_runner = $true
            immutable_snapshots_and_terminal_receipts_are_never_overwritten = $true
            manifest_must_be_complete_after_each_authoritative_phase = $true
        }
    }
    $sha256 = Write-Sprint8APreflightJsonReceipt -Document $document -Path $inventoryPath
    $reference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $inventoryPath).Replace("\", "/")
        sha256 = $sha256
    }
    if (-not $script:producedEvidenceByCheck.ContainsKey("evidence-inventory")) {
        $script:producedEvidenceByCheck["evidence-inventory"] = [Collections.Generic.List[object]]::new()
    }
    $script:producedEvidenceByCheck["evidence-inventory"].Add($reference)
    "complete existing/future/conditional evidence inventory retained at $($reference.path)"
}

function New-Sprint8APreflightManifestOverride {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][bool]$Authoritative,
        [Parameter(Mandatory)][string]$Status
    )

    $resolved = Resolve-Sprint8APreflightEvidencePath `
        -RepositoryRoot $repoRoot `
        -EvidenceRootPath $script:evidenceRootPath `
        -Path $Path
    [pscustomobject][ordered]@{
        path = [string]$resolved.path
        sha256 = Get-Sprint8APreflightFileSha256 -Path ([string]$resolved.full_path)
        phase = $Phase
        authoritative = $Authoritative
        status = $Status
    }
}

function Initialize-Sprint8APreflightEvidenceManifest {
    param(
        [Parameter(Mandatory)]$PreflightReference,
        [Parameter(Mandatory)]$CandidateReference
    )

    $overrides = [Collections.Generic.List[object]]::new()
    foreach ($descriptor in @(
        [pscustomobject]@{ path = [string]$script:runtimeContext.readiness_reference.path; phase = "readiness"; authoritative = $false; status = "passed" },
        [pscustomobject]@{ path = [string]$script:runtimeContext.readiness_immutable_reference.path; phase = "readiness-attempt"; authoritative = $false; status = "passed" },
        [pscustomobject]@{ path = [string]$script:runtimeContext.rehearsal_reference.path; phase = "rehearsal"; authoritative = $false; status = "passed" },
        [pscustomobject]@{ path = [string]$script:runtimeContext.rehearsal_attempt_reference.path; phase = "rehearsal-attempt"; authoritative = $false; status = "passed" },
        [pscustomobject]@{ path = [string]$script:runtimeContext.validation_state_reference.path; phase = "validation-state"; authoritative = $false; status = "preflight-eligible" },
        [pscustomobject]@{ path = [string]$PreflightReference.path; phase = "validation-preflight"; authoritative = $true; status = "passed" },
        [pscustomobject]@{ path = [string]$CandidateReference.path; phase = "candidate-freeze"; authoritative = $true; status = "passed" },
        [pscustomobject]@{ path = [IO.Path]::GetRelativePath($repoRoot, $attemptPath).Replace("\", "/"); phase = "validation-preflight-attempt"; authoritative = $false; status = "passed" },
        [pscustomobject]@{ path = [IO.Path]::GetRelativePath($repoRoot, $inventoryPath).Replace("\", "/"); phase = "validation-preflight"; authoritative = $false; status = "passed" }
    )) {
        foreach ($path in @([string]$descriptor.path, "$([string]$descriptor.path).sha256")) {
            $overrides.Add((New-Sprint8APreflightManifestOverride `
                -Path $path `
                -Phase ([string]$descriptor.phase) `
                -Authoritative ([bool]$descriptor.authoritative) `
                -Status ([string]$descriptor.status)))
        }
    }
    $attemptDirectory = Join-Path $script:evidenceRootPath "attempts"
    foreach ($priorAttempt in @(Get-ChildItem `
        -LiteralPath $attemptDirectory `
        -Filter "preflight-*.json" `
        -File `
        -ErrorAction SilentlyContinue)) {
        if ([IO.Path]::GetFullPath($priorAttempt.FullName) -ceq [IO.Path]::GetFullPath($attemptPath)) {
            continue
        }
        $priorPath = [IO.Path]::GetRelativePath($repoRoot, $priorAttempt.FullName).Replace("\", "/")
        foreach ($path in @($priorPath, "$priorPath.sha256")) {
            $overrides.Add((New-Sprint8APreflightManifestOverride `
                -Path $path `
                -Phase "validation-preflight-attempt" `
                -Authoritative $false `
                -Status "superseded"))
        }
    }

    $entries = @(Get-Sprint8AEvidenceFileManifestEntries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -Overrides @($overrides))
    if ($entries.Count -lt $overrides.Count -or
        @($entries.path | Sort-Object -Unique).Count -ne $entries.Count) {
        throw "Sprint 8A freeze manifest initialization is incomplete or duplicated."
    }
    $reference = Publish-Sprint8AEvidenceManifest `
        -Entries $entries `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -OutputPath "$evidenceRootRelative/evidence-manifest.json"
    Assert-Sprint8AEvidenceManifestCompleteness `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -ManifestPath ([string]$reference.path) | Out-Null
    $reference
}

$preflightLockPath = Join-Path $script:evidenceRootPath "validation-attempt.lock"
$script:runtimeContext.attempt_lock = Open-Sprint8AValidationAttemptLock -Path $preflightLockPath
$script:runtimeContext.attempt_lock_path = $preflightLockPath
try {
[IO.Directory]::CreateDirectory((Split-Path -Parent $attemptPath)) | Out-Null
[IO.Directory]::CreateDirectory($logRoot) | Out-Null
[IO.Directory]::CreateDirectory($structuredRoot) | Out-Null
Write-Sprint8APreflightJsonReceipt -Document $script:attemptReceipt -Path $attemptPath | Out-Null

$actions = [ordered]@{
    "receipt-chain" = { Assert-Sprint8APreflightReceiptChain }
    "repository-scope" = { Assert-Sprint8APreflightRepositoryScope }
    "clean-source" = { Assert-Sprint8APreflightCleanSource }
    "acceptance-traceability" = { Assert-Sprint8APreflightAcceptanceTraceability }
    "environment-contract" = { Assert-Sprint8APreflightEnvironmentContract }
    "database-contract" = { Assert-Sprint8APreflightDatabaseContract }
    "deployment-contract" = { Assert-Sprint8APreflightDeploymentContract }
    "downstream-command-contract" = { Assert-Sprint8APreflightDownstreamCommands }
    "evidence-path-contract" = { Assert-Sprint8APreflightEvidencePaths }
    "evidence-inventory" = { Publish-Sprint8APreflightInventory }
}

foreach ($declaration in $declaredChecks) {
    Invoke-Sprint8APreflightCheck `
        -Name ([string]$declaration.name) `
        -Action $actions[[string]$declaration.name]
}

if ($script:terminalChecks.Count -ne 10 -or $script:terminalByName.Count -ne 10) {
    throw "Sprint 8A preflight scheduler did not produce all ten terminal check results."
}
$nonpassing = @($script:terminalChecks | Where-Object state -CNE "passed")
if ($nonpassing.Count -ne 0) {
    $endedAt = [DateTimeOffset]::UtcNow
    $script:attemptReceipt.state = "failed"
    $script:attemptReceipt.active_check = $null
    $script:attemptReceipt.ended_at = $endedAt.ToString("o")
    $script:attemptReceipt.duration_ms = [long][Math]::Max(0, ($endedAt - $startedAt).TotalMilliseconds)
    $script:attemptReceipt.invalidation_decision = "preflight failed after complete safe fail-late harvest; candidate freeze forbidden"
    Checkpoint-Sprint8APreflightAttempt
    $failedNames = @($script:terminalChecks | Where-Object state -CEQ "failed" | ForEach-Object { [string]$_.name })
    $blockedNames = @($script:terminalChecks | Where-Object state -CEQ "blocked" | ForEach-Object { [string]$_.name })
    throw "Sprint 8A validation preflight failed after diagnostic harvest. Failed: $($failedNames -join ', '); blocked: $($blockedNames -join ', ')."
}

try {
    $script:attemptReceipt.state = "finalizing"
    $script:attemptReceipt.active_check = $null
    Checkpoint-Sprint8APreflightAttempt

    Assert-Sprint8AExactTerminalIdentities `
        -Results @($script:terminalChecks) `
        -ExpectedNames (Get-Sprint8APreflightCheckNames) `
        -Label "validation-preflight" | Out-Null
    $publicationSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $publicationSource -RequireClean | Out-Null
    if (-not (Test-Sprint8ASourceIdentityMatch `
            -Expected $script:runtimeContext.source `
            -Actual $publicationSource)) {
        throw "Sprint 8A source changed after the terminal inventory and before lifecycle publication."
    }
    $publicationEnvironment = Get-Sprint8AEnvironmentContract `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $EvidenceRoot
    if ([string]$publicationEnvironment.fingerprint -cne [string]$script:runtimeContext.environment.fingerprint) {
        throw "Sprint 8A environment changed after the terminal inventory and before lifecycle publication."
    }

    $prerequisites = @(
        [pscustomobject][ordered]@{
            path = [string]$script:runtimeContext.readiness_reference.path
            sha256 = [string]$script:runtimeContext.readiness_reference.sha256
        }
        [pscustomobject][ordered]@{
            path = [string]$script:runtimeContext.rehearsal_reference.path
            sha256 = [string]$script:runtimeContext.rehearsal_reference.sha256
        }
    )
    $inventoryReference = [pscustomobject][ordered]@{
        path = [IO.Path]::GetRelativePath($repoRoot, $inventoryPath).Replace("\", "/")
        sha256 = Assert-Sprint8APreflightSidecar -Path $inventoryPath
    }
    $preflightReference = Publish-Sprint8ALifecycleReceipt `
        -Phase "validation-preflight" `
        -Attempt $Attempt `
        -Source $publicationSource `
        -EnvironmentFingerprint ([string]$script:runtimeContext.environment.fingerprint) `
        -NormalizedDeploymentConfigurationSha256 ([string]$script:runtimeContext.normalized_deployment_configuration_sha256) `
        -PrerequisiteReceipts $prerequisites `
        -Checks @($script:terminalChecks) `
        -Details ([pscustomobject][ordered]@{
            evidence_inventory = $inventoryReference
            validation_state = [pscustomobject][ordered]@{
                path = [string]$script:runtimeContext.validation_state_reference.path
                sha256 = [string]$script:runtimeContext.validation_state_reference.sha256
            }
            rehearsal_attempt = [pscustomobject][ordered]@{
                path = [string]$script:runtimeContext.rehearsal_attempt_reference.path
                sha256 = [string]$script:runtimeContext.rehearsal_attempt_reference.sha256
            }
            handoff_url = $HandoffUrl.TrimEnd("/")
            normalized_deployment_configuration_sha256 = [string]$script:runtimeContext.normalized_deployment_configuration_sha256
            source_and_environment_exact = $true
            safe_fail_late_complete = $true
        }) `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -OutputPath "$evidenceRootRelative/preflight-result.json"

    $candidatePublicationSource = Get-Sprint8ASourceIdentity -RepositoryRoot $repoRoot
    Assert-Sprint8ASourceIdentityObject -Source $candidatePublicationSource -RequireClean | Out-Null
    if (-not (Test-Sprint8ASourceIdentityMatch `
            -Expected $publicationSource `
            -Actual $candidatePublicationSource)) {
        throw "Sprint 8A source changed between preflight publication and candidate freeze."
    }
    $candidateIdentity = Get-Sprint8ACandidateIdentity `
        -RepositoryRoot $repoRoot `
        -Source $candidatePublicationSource `
        -NormalizedDeploymentConfigurationSha256 ([string]$script:runtimeContext.normalized_deployment_configuration_sha256)
    if ($null -ne $script:runtimeContext.candidate_identity -and
        [string]$script:runtimeContext.candidate_identity.fingerprint -cne [string]$candidateIdentity.fingerprint) {
        throw "Sprint 8A candidate identity changed between deployment audit and freeze."
    }
    $candidateReference = Publish-Sprint8ALifecycleReceipt `
        -Phase "candidate-freeze" `
        -Attempt $Attempt `
        -Source $candidatePublicationSource `
        -EnvironmentFingerprint ([string]$script:runtimeContext.environment.fingerprint) `
        -NormalizedDeploymentConfigurationSha256 ([string]$script:runtimeContext.normalized_deployment_configuration_sha256) `
        -CandidateFingerprint ([string]$candidateIdentity.fingerprint) `
        -PrerequisiteReceipts @([pscustomobject][ordered]@{
            path = [string]$preflightReference.path
            sha256 = [string]$preflightReference.sha256
        }) `
        -Checks @() `
        -Details ([pscustomobject][ordered]@{
            preflight_attempt = $Attempt
            evidence_inventory = $inventoryReference
            normalized_deployment_configuration_sha256 = [string]$script:runtimeContext.normalized_deployment_configuration_sha256
            immutable_source_and_inputs = $true
        }) `
        -RepositoryRoot $repoRoot `
        -EvidenceRoot $script:evidenceRootPath `
        -OutputPath "$evidenceRootRelative/candidate.json"

    $script:attemptReceipt.preflight_result = [pscustomobject][ordered]@{
        path = [string]$preflightReference.path
        sha256 = [string]$preflightReference.sha256
    }
    $script:attemptReceipt.candidate = [pscustomobject][ordered]@{
        path = [string]$candidateReference.path
        sha256 = [string]$candidateReference.sha256
        fingerprint = [string]$candidateIdentity.fingerprint
    }
    $script:attemptReceipt.evidence_manifest = [pscustomobject][ordered]@{
        path = "$evidenceRootRelative/evidence-manifest.json"
        publication = "immediately_after_terminal_attempt_checkpoint"
    }
    $endedAt = [DateTimeOffset]::UtcNow
    $script:attemptReceipt.state = "passed"
    $script:attemptReceipt.ended_at = $endedAt.ToString("o")
    $script:attemptReceipt.duration_ms = [long][Math]::Max(0, ($endedAt - $startedAt).TotalMilliseconds)
    $script:attemptReceipt.invalidation_decision = "none_required"
    Checkpoint-Sprint8APreflightAttempt

    $manifestReference = Initialize-Sprint8APreflightEvidenceManifest `
        -PreflightReference $preflightReference `
        -CandidateReference $candidateReference
    [pscustomobject][ordered]@{
        state = "passed"
        attempt = $Attempt
        preflight = [pscustomobject][ordered]@{
            path = [string]$preflightReference.path
            sha256 = [string]$preflightReference.sha256
        }
        candidate = [pscustomobject][ordered]@{
            path = [string]$candidateReference.path
            sha256 = [string]$candidateReference.sha256
            fingerprint = [string]$candidateIdentity.fingerprint
        }
        evidence_manifest = [pscustomobject][ordered]@{
            path = [string]$manifestReference.path
            sha256 = [string]$manifestReference.sha256
            completeness = "verified"
        }
    }
} catch {
    $finalizationError = $_
    $failurePath = Join-Path $attemptRoot "finalization-failure.json"
    $failureReference = $null
    try {
        $failureDocument = [pscustomobject][ordered]@{
            schema_version = 1
            sprint = "sprint-8a"
            phase = "validation-preflight-finalization"
            attempt = $Attempt
            state = "failed"
            classification = "evidence-finalization"
            classification_source = "runner_finalization_boundary"
            failed_at = [DateTimeOffset]::UtcNow.ToString("o")
            failure_message = $finalizationError.Exception.Message
            preflight_result_present = Test-Path -LiteralPath $preflightResultPath -PathType Leaf
            candidate_present = Test-Path -LiteralPath $candidatePath -PathType Leaf
            manifest_present = Test-Path -LiteralPath $manifestPath -PathType Leaf
        }
        $failureSha = Write-Sprint8APreflightJsonReceipt -Document $failureDocument -Path $failurePath
        $failureReference = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath($repoRoot, $failurePath).Replace("\", "/")
            sha256 = $failureSha
        }
    } catch {
        $failureReference = $null
    }
    $failedAt = [DateTimeOffset]::UtcNow
    $script:attemptReceipt.finalization_failure = [pscustomobject][ordered]@{
        classification = "evidence-finalization"
        classification_source = "runner_finalization_boundary"
        failure_message = $finalizationError.Exception.Message
        evidence = if ($null -eq $failureReference) { @() } else { @($failureReference) }
    }
    $script:attemptReceipt.state = "failed"
    $script:attemptReceipt.ended_at = $failedAt.ToString("o")
    $script:attemptReceipt.duration_ms = [long][Math]::Max(0, ($failedAt - $startedAt).TotalMilliseconds)
    $script:attemptReceipt.invalidation_decision = "freeze finalization failed; candidate, SIT, and UAT forbidden"
    Checkpoint-Sprint8APreflightAttempt
    throw
}
} finally {
    if ($null -ne $script:runtimeContext.attempt_lock) {
        $script:runtimeContext.attempt_lock.Dispose()
        $script:runtimeContext.attempt_lock = $null
    }
}

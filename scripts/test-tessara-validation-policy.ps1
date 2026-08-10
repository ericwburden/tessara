[CmdletBinding()]
param([switch]$SelfTest)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if (-not $SelfTest) {
    throw "This script is an adversarial policy self-test. Invoke it with -SelfTest."
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $PSScriptRoot "tessara-validation-policy.psm1") -Force

function Assert-Throws {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$Label
    )

    try {
        & $Action
    } catch {
        return
    }
    throw "Expected rejection did not occur: $Label"
}

function New-Reference {
    param([string]$Path, [string]$Sha = ("a" * 64))
    return [pscustomobject]@{ path = $Path; sha256 = $Sha }
}

$temporaryRoot = Join-Path $repositoryRoot ("tmp/validation-policy-v2-selftest-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temporaryRoot -Force | Out-Null

try {
    $templatePath = Join-Path $repositoryRoot ".codex/skills/tessara-sprint-validation/assets/sprint-validation-contract.json"
    $templateContract = Get-Content -LiteralPath $templatePath -Raw | ConvertFrom-Json
    $null = Assert-TessaraValidationContract -Contract $templateContract

    $contract = [pscustomobject]@{
        schema_version = 2
        contract = "tessara.validation-contract"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        requirements = @(
            [pscustomobject]@{
                id = "req-product"
                implementation_targets = @("target-static", "target-materialize")
                validation_lanes = @("readiness-contract", "rehearsal-product", "sit-product", "uat-product")
            }
        )
        dependency_domains = @(
            [pscustomobject]@{ name = "product-source"; tracked_inputs = @("crates/**"); environment_sections = @() },
            [pscustomobject]@{ name = "deployment-materialization"; tracked_inputs = @("deploy/**"); environment_sections = @("compose") },
            [pscustomobject]@{ name = "readiness-runner"; tracked_inputs = @("scripts/readiness*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "rehearsal-runner"; tracked_inputs = @("scripts/rehearsal*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "preflight-runner"; tracked_inputs = @("scripts/preflight*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "sit-runner"; tracked_inputs = @("scripts/sit*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "uat-runner"; tracked_inputs = @("scripts/uat*.ps1"); environment_sections = @() },
            [pscustomobject]@{ name = "evidence-publication"; tracked_inputs = @("scripts/evidence*.ps1"); environment_sections = @() }
        )
        implementation_targets = @(
            [pscustomobject]@{
                id = "target-static"
                command = "cargo check --workspace --locked"
                dependency_domains = @("product-source")
                required = $true
                clean_environment = $false
            },
            [pscustomobject]@{
                id = "target-materialize"
                command = ".\scripts\materialize.ps1 -Clean"
                dependency_domains = @("deployment-materialization")
                required = $false
                clean_environment = $true
            }
        )
        lanes = @(
            [pscustomobject]@{ id = "readiness-contract"; phase = "validation-readiness"; dependency_domains = @("readiness-runner"); prerequisites = @(); touches_live_state = $false },
            [pscustomobject]@{ id = "rehearsal-product"; phase = "candidate-rehearsal"; dependency_domains = @("product-source", "rehearsal-runner"); prerequisites = @(); touches_live_state = $false },
            [pscustomobject]@{ id = "rehearsal-materialize"; phase = "candidate-rehearsal"; dependency_domains = @("deployment-materialization", "rehearsal-runner"); prerequisites = @("rehearsal-product"); touches_live_state = $true },
            [pscustomobject]@{ id = "preflight-freeze"; phase = "validation-preflight"; dependency_domains = @("preflight-runner"); prerequisites = @(); touches_live_state = $false },
            [pscustomobject]@{ id = "sit-product"; phase = "sit"; dependency_domains = @("product-source", "sit-runner"); prerequisites = @(); touches_live_state = $true },
            [pscustomobject]@{ id = "uat-product"; phase = "uat"; dependency_domains = @("product-source", "uat-runner"); prerequisites = @(); touches_live_state = $true }
        )
        evidence_policy = [pscustomobject]@{
            root = "artifacts/sprint-9a-closeout"
            tracked = $false
            successful_raw = "retained_cold"
            phase_local_indexes = $true
            final_full_integrity_audit = $true
        }
    }

    $null = Assert-TessaraValidationContract -Contract $contract
    $fingerprintPassOne = @(Get-TessaraDependencyFingerprints -Contract $contract -RepositoryRoot $repositoryRoot)
    $fingerprintPassTwo = @(Get-TessaraDependencyFingerprints -Contract $contract -RepositoryRoot $repositoryRoot)
    if ($fingerprintPassOne.Count -ne @($contract.dependency_domains).Count -or
        (($fingerprintPassOne | ConvertTo-Json -Depth 10 -Compress) -cne ($fingerprintPassTwo | ConvertTo-Json -Depth 10 -Compress))) {
        throw "Dependency fingerprints are incomplete or nondeterministic."
    }
    $duplicateContract = $contract | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $duplicateContract.dependency_domains += $duplicateContract.dependency_domains[0]
    Assert-Throws -Label "duplicate dependency domain" -Action { Assert-TessaraValidationContract -Contract $duplicateContract }

    $preflightImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("scripts/preflight-freeze.ps1")
    if (($preflightImpact.phase_decisions | Where-Object phase -eq "validation-readiness").action -cne "reuse_certificate" -or
        ($preflightImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").action -cne "reuse_certificate" -or
        ($preflightImpact.phase_decisions | Where-Object phase -eq "validation-preflight").action -cne "rerun_full_phase") {
        throw "A Preflight-only change did not preserve Readiness and Rehearsal certificates."
    }

    $productImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("crates/example/src/lib.rs") -CandidateChanged
    if (($productImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").action -cne "recertify_affected_lanes" -or
        ($productImpact.phase_decisions | Where-Object phase -eq "sit").action -cne "rerun_full_phase" -or
        ($productImpact.phase_decisions | Where-Object phase -eq "uat").action -cne "rerun_full_phase" -or
        -not $productImpact.require_complete_sit -or -not $productImpact.require_complete_uat) {
        throw "Candidate-changing product impact did not require pre-freeze recertification plus complete SIT/UAT."
    }

    $materializationImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("deploy/sprint-9a/compose.yaml")
    $materializationLanes = @(($materializationImpact.phase_decisions | Where-Object phase -eq "candidate-rehearsal").affected_lanes)
    if ("rehearsal-materialize" -notin $materializationLanes -or "rehearsal-product" -notin $materializationLanes) {
        throw "Affected-lane selection did not include the safe prerequisite closure."
    }

    $unknownImpact = Get-TessaraValidationImpact -Contract $contract -ChangedPaths @("unknown/new-input.txt")
    if (@($unknownImpact.phase_decisions | Where-Object action -ne "rerun_full_phase").Count -ne 0) {
        throw "An unknown tracked path did not select the conservative full-phase fallback."
    }

    $contractPath = Join-Path $temporaryRoot "validation-contract.json"
    $contract | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $contractPath -Encoding utf8NoBOM
    $contractSha = Get-TessaraValidationSha256 -Path $contractPath
    $source = [pscustomobject]@{ commit = ("1" * 40); tree = ("2" * 40); dirty = $false }
    $impactDocument = [pscustomobject]@{
        schema_version = 2
        contract = "tessara.validation.correction-impact-assessment"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        baseline_source = [pscustomobject]@{ commit = ("7" * 40); tree = ("8" * 40); dirty = $false }
        current_source = $source
        changed_paths = $productImpact.changed_paths
        changed_domains = $productImpact.changed_domains
        unknown_paths = $productImpact.unknown_paths
        phase_decisions = $productImpact.phase_decisions
        candidate_changed = $productImpact.candidate_changed
        require_complete_sit = $productImpact.require_complete_sit
        require_complete_uat = $productImpact.require_complete_uat
        authorized_at = "2026-08-10T10:00:00Z"
    }
    $null = Assert-TessaraCorrectionImpactAssessment -Assessment $impactDocument -Contract $contract
    $unsafeImpact = $impactDocument | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $unsafeImpact.require_complete_uat = $false
    Assert-Throws -Label "candidate correction without complete UAT" -Action {
        Assert-TessaraCorrectionImpactAssessment -Assessment $unsafeImpact -Contract $contract
    }
    $implementationResult = [pscustomobject]@{
        schema_version = 1
        contract = "tessara.implementation-readiness-result"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        state = "passed"
        authoritative = $false
        source_identity = $source
        validation_contract = New-Reference -Path "docs/sprints/sprint-9a-validation-contract.json" -Sha $contractSha
        affected_domains = @("product-source")
        targets = @(
            [pscustomobject]@{
                id = "target-static"; state = "passed"; command = "cargo check --workspace --locked"
                clean_environment = $false; evidence = New-Reference -Path "artifacts/sprint-9a-closeout/implementation/static.json"
            }
        )
        known_failure_count = 0
        materialization = [pscustomobject]@{
            required = $false
            first_apply = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
            semantic_no_op = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
            recovery = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
        }
        cleanup_restoration = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
        evidence_index = New-Reference -Path "artifacts/sprint-9a-closeout/implementation/evidence-index.json"
    }
    $null = Assert-TessaraImplementationReadinessResult -Result $implementationResult -Contract $contract -ContractPath $contractPath

    $failedImplementation = $implementationResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $failedImplementation.targets[0].state = "failed"
    $failedImplementation.state = "failed"
    $failedImplementation.known_failure_count = 1
    Assert-Throws -Label "known implementation failure" -Action {
        Assert-TessaraImplementationReadinessResult -Result $failedImplementation -Contract $contract -ContractPath $contractPath
    }

    $missingMaterialization = $implementationResult | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $missingMaterialization.affected_domains = @("deployment-materialization")
    Assert-Throws -Label "missing selected clean materialization target" -Action {
        Assert-TessaraImplementationReadinessResult -Result $missingMaterialization -Contract $contract -ContractPath $contractPath
    }

    $fingerprints = @(
        [pscustomobject]@{ domain = "product-source"; sha256 = ("3" * 64) },
        [pscustomobject]@{ domain = "rehearsal-runner"; sha256 = ("4" * 64) }
    )
    $phaseCertificate = [pscustomobject]@{
        schema_version = 1
        contract = "tessara.validation.phase-certificate"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"
        phase = "candidate-rehearsal"
        attempt = 2
        state = "passed"
        authoritative = $false
        certified_at = "2026-08-10T12:00:00Z"
        source_identity = $source
        environment_fingerprint = ("5" * 64)
        candidate_fingerprint = $null
        prerequisite_certificates = @(New-Reference -Path "artifacts/sprint-9a-closeout/validation-readiness-result.json")
        dependency_fingerprints = $fingerprints
        coverage = [pscustomobject]@{ declared_lanes_sha256 = ("6" * 64); lane_count = 2; executed_count = 1; inherited_count = 1 }
        lanes = @(
            [pscustomobject]@{
                name = "rehearsal-product"; state = "passed"; certification_basis = "executed"
                dependency_domains = @("product-source"); receipt = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-2/product.json"
                started_at = "2026-08-10T11:00:00Z"; ended_at = "2026-08-10T11:01:00Z"; duration_ms = 60000; inheritance = $null
            },
            [pscustomobject]@{
                name = "rehearsal-runner-contract"; state = "passed"; certification_basis = "inherited_nonimpact"
                dependency_domains = @("rehearsal-runner"); receipt = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-1/runner.json"
                started_at = $null; ended_at = $null; duration_ms = $null
                inheritance = [pscustomobject]@{
                    prior_receipt = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-1/runner.json"
                    prior_source_identity = [pscustomobject]@{ commit = ("7" * 40); tree = ("8" * 40); dirty = $false }
                    prior_environment_fingerprint = ("5" * 64)
                    prior_dependency_fingerprints = @([pscustomobject]@{ domain = "rehearsal-runner"; sha256 = ("4" * 64) })
                    nonimpact_rationale = "The runner contract fingerprint is unchanged."
                }
            }
        )
        open_defect_count = 0
        cleanup_restoration = [pscustomobject]@{ required = $false; state = "not_applicable"; evidence = $null }
        evidence_index = New-Reference -Path "artifacts/sprint-9a-closeout/rehearsal/attempt-2/evidence-index.json"
    }
    $null = Assert-TessaraPhaseCertificate -Certificate $phaseCertificate

    $changedInheritance = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $changedInheritance.lanes[1].inheritance.prior_dependency_fingerprints[0].sha256 = ("9" * 64)
    Assert-Throws -Label "changed inherited dependency" -Action { Assert-TessaraPhaseCertificate -Certificate $changedInheritance }

    $sitInheritance = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $sitInheritance.phase = "sit"
    $sitInheritance.authoritative = $true
    $sitInheritance.candidate_fingerprint = ("a" * 64)
    Assert-Throws -Label "candidate-bound inherited lane" -Action { Assert-TessaraPhaseCertificate -Certificate $sitInheritance }

    $coldIndex = [pscustomobject]@{
        schema_version = 1; contract = "tessara.validation.phase-evidence-index"; policy_version = "tessara-validation-v2"
        sprint = "sprint-9a"; phase = "candidate-rehearsal"; attempt = 2
        evidence_root = "artifacts/sprint-9a-closeout/rehearsal/attempt-2"
        sealed_at = "2026-08-10T12:00:00Z"
        entry_count = 1
        entries = @([pscustomobject]@{ path = "artifacts/sprint-9a-closeout/rehearsal/attempt-2/cold.log"; sha256 = ("b" * 64); size = 12; kind = "log" })
    }
    $null = Assert-TessaraPhaseEvidenceIndex -Index $coldIndex
    Assert-Throws -Label "explicit audit detects missing cold evidence" -Action {
        Assert-TessaraPhaseEvidenceIndex -Index $coldIndex -RepositoryRoot $repositoryRoot -AuditFiles
    }

    $rawPath = Join-Path $temporaryRoot "raw.log"
    "passing raw evidence" | Set-Content -LiteralPath $rawPath -Encoding utf8NoBOM -NoNewline
    $relativeRaw = [IO.Path]::GetRelativePath($repositoryRoot, $rawPath).Replace("\", "/")
    $realIndex = $coldIndex | ConvertTo-Json -Depth 20 | ConvertFrom-Json
    $realIndex.evidence_root = [IO.Path]::GetRelativePath($repositoryRoot, $temporaryRoot).Replace("\", "/")
    $realIndex.entries[0].path = $relativeRaw
    $realIndex.entries[0].sha256 = Get-TessaraValidationSha256 -Path $rawPath
    $realIndex.entries[0].size = (Get-Item -LiteralPath $rawPath).Length
    $null = Assert-TessaraPhaseEvidenceIndex -Index $realIndex -RepositoryRoot $repositoryRoot -AuditFiles
    Add-Content -LiteralPath $rawPath -Value "tamper" -NoNewline
    Assert-Throws -Label "final audit detects raw evidence tamper" -Action {
        Assert-TessaraPhaseEvidenceIndex -Index $realIndex -RepositoryRoot $repositoryRoot -AuditFiles
    }

    $chainIndexPath = Join-Path $temporaryRoot "evidence-index.json"
    $coldIndex | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $chainIndexPath -Encoding utf8NoBOM
    $phaseCertificate.evidence_index = New-Reference `
        -Path ([IO.Path]::GetRelativePath($repositoryRoot, $chainIndexPath).Replace("\", "/")) `
        -Sha (Get-TessaraValidationSha256 -Path $chainIndexPath)
    $certificatePath = Join-Path $temporaryRoot "candidate-rehearsal-result.json"
    $phaseCertificate | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $certificatePath -Encoding utf8NoBOM
    $relativeCertificate = [IO.Path]::GetRelativePath($repositoryRoot, $certificatePath).Replace("\", "/")
    $chain = [pscustomobject]@{
        schema_version = 1; contract = "tessara.validation.evidence-chain"; policy_version = "tessara-validation-v2"; sprint = "sprint-9a"
        certificates = @([pscustomobject]@{ phase = "candidate-rehearsal"; path = $relativeCertificate; sha256 = (Get-TessaraValidationSha256 -Path $certificatePath) })
        corrections = @()
        final_integrity_audit = [pscustomobject]@{ state = "pending"; audited_at = $null; phase_index_count = 0; artifact_count = 0 }
    }
    $null = Assert-TessaraEvidenceChain -Chain $chain -RepositoryRoot $repositoryRoot
    Assert-Throws -Label "closeout rejects pending final audit" -Action {
        Assert-TessaraEvidenceChain -Chain $chain -RepositoryRoot $repositoryRoot -FinalAudit
    }

    "passing raw evidence" | Set-Content -LiteralPath $rawPath -Encoding utf8NoBOM -NoNewline
    $realIndex.entries[0].sha256 = Get-TessaraValidationSha256 -Path $rawPath
    $realIndex.entries[0].size = (Get-Item -LiteralPath $rawPath).Length
    $auditedIndexPath = Join-Path $temporaryRoot "audited-evidence-index.json"
    $realIndex | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $auditedIndexPath -Encoding utf8NoBOM
    $auditedCertificate = $phaseCertificate | ConvertTo-Json -Depth 30 | ConvertFrom-Json
    $auditedCertificate.evidence_index = New-Reference `
        -Path ([IO.Path]::GetRelativePath($repositoryRoot, $auditedIndexPath).Replace("\", "/")) `
        -Sha (Get-TessaraValidationSha256 -Path $auditedIndexPath)
    $auditedCertificatePath = Join-Path $temporaryRoot "audited-candidate-rehearsal-result.json"
    $auditedCertificate | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $auditedCertificatePath -Encoding utf8NoBOM
    $auditedChain = [pscustomobject]@{
        schema_version = 1; contract = "tessara.validation.evidence-chain"; policy_version = "tessara-validation-v2"; sprint = "sprint-9a"
        certificates = @([pscustomobject]@{
            phase = "candidate-rehearsal"
            path = [IO.Path]::GetRelativePath($repositoryRoot, $auditedCertificatePath).Replace("\", "/")
            sha256 = Get-TessaraValidationSha256 -Path $auditedCertificatePath
        })
        corrections = @()
        final_integrity_audit = [pscustomobject]@{
            state = "passed"; audited_at = "2026-08-10T13:00:00Z"; phase_index_count = 1; artifact_count = 1
        }
    }
    $null = Assert-TessaraEvidenceChain -Chain $auditedChain -RepositoryRoot $repositoryRoot -FinalAudit
    Add-Content -LiteralPath $rawPath -Value "tamper" -NoNewline
    Assert-Throws -Label "evidence-chain final audit detects raw tamper" -Action {
        Assert-TessaraEvidenceChain -Chain $auditedChain -RepositoryRoot $repositoryRoot -FinalAudit
    }

    $ignoreOutput = & git -C $repositoryRoot check-ignore "artifacts/validation-policy-v2-probe.json" 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace(($ignoreOutput | Out-String))) {
        throw "Generated validation evidence is no longer ignored by Git."
    }

    Write-Output "Tessara validation policy v2 self-tests passed."
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

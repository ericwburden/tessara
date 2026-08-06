[CmdletBinding()]
param(
    [ValidateRange(1, 2147483647)]
    [int]$Attempt = 1,
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout/rehearsal",
    [string]$EnvironmentFingerprint,
    [string]$OutputPath,
    [string]$ComposeFile = "deploy/sprint-8a/compose.yaml",
    [string]$BlueprintPath = "deploy/sprint-8a/blueprints/reference.json",
    [string]$CoreUrl = "http://127.0.0.1:8088",
    [string]$SupervisorUrl = "http://127.0.0.1:8098",
    [switch]$AuthorizeDisposableReset,
    [switch]$SkipBuild,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

$faultId = "sprint-8a.dashboard-owner-bootstrap-invalid-layout"
$faultPattern = "Dashboard bootstrap layout is invalid"

function Resolve-Sprint8AContainmentPath {
    param([Parameter(Mandatory)][string]$Path)
    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

function Get-Sprint8AContainmentDisplayPath {
    param([Parameter(Mandatory)][string]$Path)
    $fullPath = [IO.Path]::GetFullPath($Path)
    $rootPrefix = [IO.Path]::GetFullPath($repoRoot).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if ($fullPath.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        return $fullPath.Substring($rootPrefix.Length).Replace("\", "/")
    }
    $fullPath.Replace("\", "/")
}

function Get-Sprint8AContainmentArtifact {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$RequireSidecar
    )
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Failure-containment artifact is missing: $fullPath"
    }
    $digest = Get-Sprint7AFileSha256 -Path $fullPath
    $artifact = [ordered]@{
        path = Get-Sprint8AContainmentDisplayPath -Path $fullPath
        sha256 = $digest
        bytes = [IO.FileInfo]::new($fullPath).Length
    }
    if ($RequireSidecar) {
        $sidecarPath = "$fullPath.sha256"
        if (-not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
            throw "Failure-containment artifact sidecar is missing: $sidecarPath"
        }
        $expectedSidecarBytes = [Text.UTF8Encoding]::new($false).GetBytes("$digest`n")
        $actualSidecarBytes = [IO.File]::ReadAllBytes($sidecarPath)
        if ([Convert]::ToBase64String($actualSidecarBytes) -cne
            [Convert]::ToBase64String($expectedSidecarBytes)) {
            throw "Failure-containment artifact sidecar does not exactly bind '$fullPath'."
        }
        $artifact.sidecar = [ordered]@{
            path = Get-Sprint8AContainmentDisplayPath -Path $sidecarPath
            sha256 = Get-Sprint7AFileSha256 -Path $sidecarPath
            bytes = [IO.FileInfo]::new($sidecarPath).Length
        }
    }
    [pscustomobject]$artifact
}

function Assert-Sprint8AContainmentReferencedArtifact {
    param(
        [Parameter(Mandatory)]$Artifact,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Artifact.PSObject.Properties.Name -notcontains "path" -or
        $Artifact.path -isnot [string] -or
        [string]::IsNullOrWhiteSpace([string]$Artifact.path) -or
        $Artifact.PSObject.Properties.Name -notcontains "sha256" -or
        $Artifact.sha256 -isnot [string] -or
        [string]$Artifact.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "$Label must declare one exact path and lowercase SHA-256 digest."
    }
    $verified = Get-Sprint8AContainmentArtifact `
        -Path (Resolve-Sprint8AContainmentPath -Path ([string]$Artifact.path)) `
        -RequireSidecar
    if ([string]$verified.sha256 -cne [string]$Artifact.sha256) {
        throw "$Label declared digest does not match its retained file."
    }
    if ($Artifact.PSObject.Properties.Name -contains "bytes" -and
        [long]$Artifact.bytes -ne [long]$verified.bytes) {
        throw "$Label declared byte count does not match its retained file."
    }
    $verified
}

function Get-Sprint8AContainmentPaths {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][int]$AttemptNumber,
        [string]$RequestedOutputPath
    )
    $containmentDirectory = Join-Path $Root "failure-containment/attempt-$AttemptNumber"
    $faultRoot = Join-Path $containmentDirectory "fault"
    $successorRoot = Join-Path $containmentDirectory "successor"
    $recoveryRoot = Join-Path $containmentDirectory "recovery"
    $resultPath = if ([string]::IsNullOrWhiteSpace($RequestedOutputPath)) {
        Join-Path $containmentDirectory "failure-containment-result.json"
    } else {
        Resolve-Sprint8AContainmentPath -Path $RequestedOutputPath
    }
    [pscustomobject][ordered]@{
        containment_directory = $containmentDirectory
        fault_root = $faultRoot
        fault_materialization_directory = Join-Path $faultRoot "materialization/attempt-$AttemptNumber"
        successor_root = $successorRoot
        successor_materialization_directory = Join-Path $successorRoot "materialization/attempt-$AttemptNumber"
        recovery_root = $recoveryRoot
        recovery_materialization_directory = Join-Path $recoveryRoot "materialization/attempt-$AttemptNumber"
        output_path = $resultPath
    }
}

function Assert-Sprint8ADestructiveEndpoint {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][int]$ExpectedPort
    )
    $uri = $null
    $parsed = -not [string]::IsNullOrWhiteSpace($Value) -and
        [Uri]::TryCreate($Value, [UriKind]::Absolute, [ref]$uri)
    $exactRoot = $false
    if ($parsed) {
        $canonicalRoot = "http://$($uri.Authority)/"
        $canonicalWithoutSlash = $canonicalRoot.Substring(0, $canonicalRoot.Length - 1)
        $exactRoot = [string]::Equals($Value, $canonicalRoot, [StringComparison]::OrdinalIgnoreCase) -or
            [string]::Equals($Value, $canonicalWithoutSlash, [StringComparison]::OrdinalIgnoreCase)
    }
    if (-not $parsed -or
        $uri.Scheme -cne [Uri]::UriSchemeHttp -or
        -not $uri.IsLoopback -or
        $uri.Port -ne $ExpectedPort -or
        -not [string]::IsNullOrEmpty($uri.UserInfo) -or
        -not [string]::IsNullOrEmpty($uri.Query) -or
        -not [string]::IsNullOrEmpty($uri.Fragment) -or
        $uri.AbsolutePath -cne "/" -or
        -not $exactRoot) {
        throw "Destructive Sprint 8A execution requires $Name to be an exact HTTP loopback root endpoint on port $ExpectedPort with no credentials, query, or fragment."
    }
}

function Assert-Sprint8AContainmentLiveArguments {
    param(
        [Parameter(Mandatory)][bool]$Authorized,
        [AllowEmptyString()][string]$Fingerprint
    )
    if (-not $Authorized) {
        throw "Failure-containment execution requires -AuthorizeDisposableReset."
    }
    if ([string]::IsNullOrWhiteSpace($Fingerprint) -or $Fingerprint -notmatch '^[0-9a-fA-F]{64}$') {
        throw "Failure-containment execution requires one Validation Readiness SHA-256 EnvironmentFingerprint."
    }
}

function Get-Sprint8AContainmentSourceIdentity {
    $commit = (& git -C $repoRoot rev-parse HEAD).Trim()
    $tree = (& git -C $repoRoot rev-parse "HEAD^{tree}").Trim()
    $status = @(& git -C $repoRoot status --porcelain=v1 --untracked-files=all | ForEach-Object { [string]$_ })
    if ($LASTEXITCODE -ne 0 -or $commit -notmatch '^[0-9a-f]{40}$' -or $tree -notmatch '^[0-9a-f]{40}$') {
        throw "Could not resolve the failure-containment source identity."
    }
    [pscustomobject][ordered]@{
        commit = $commit
        tree = $tree
        dirty = $status.Count -ne 0
        dirty_paths = $status
    }
}

function New-Sprint8AOwnerBootstrapFaultBlueprint {
    param(
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$DestinationPath
    )
    $source = [IO.Path]::GetFullPath($SourcePath)
    $destination = [IO.Path]::GetFullPath($DestinationPath)
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Canonical Sprint 8A Blueprint is missing: $source"
    }
    if ((Test-Path -LiteralPath $destination) -or (Test-Path -LiteralPath "$destination.sha256")) {
        throw "Refusing to replace retained fault Blueprint: $destination"
    }
    $sourceHashBefore = Get-Sprint7AFileSha256 -Path $source
    $blueprint = Get-Content -LiteralPath $source -Raw | ConvertFrom-Json
    $dashboardModules = @($blueprint.modules | Where-Object definition_id -CEQ "tessara.dashboards")
    if ($dashboardModules.Count -ne 1 -or $null -eq $dashboardModules[0].bootstrap.value) {
        throw "Fault injection requires exactly one inline Dashboard owner bootstrap."
    }
    $placements = @($dashboardModules[0].bootstrap.value.placements)
    $targetPlacements = @($placements | Where-Object placement_key -CEQ "row-count")
    if ($targetPlacements.Count -ne 1) {
        throw "Fault injection requires the exact 'row-count' Dashboard placement."
    }
    if ($null -ne $targetPlacements[0].component_reference -or [int]$targetPlacements[0].width -ne 4) {
        throw "The canonical Dashboard placement does not have the expected receipt-bound layout."
    }
    $targetPlacements[0].width = 0

    [IO.Directory]::CreateDirectory((Split-Path -Parent $destination)) | Out-Null
    [IO.File]::WriteAllText(
        $destination,
        ($blueprint | ConvertTo-Json -Depth 100) + "`n",
        [Text.UTF8Encoding]::new($false)
    )
    $faultHash = Get-Sprint7AFileSha256 -Path $destination
    [IO.File]::WriteAllText("$destination.sha256", "$faultHash`n", [Text.UTF8Encoding]::new($false))
    if ((Get-Sprint7AFileSha256 -Path $source) -cne $sourceHashBefore) {
        throw "Fault injection changed the canonical Blueprint."
    }
    if ($faultHash -ceq $sourceHashBefore) {
        throw "Fault injection did not change the Blueprint identity."
    }
    [pscustomobject][ordered]@{
        id = $faultId
        classification = "validation_only_fault_injection"
        owner = "tessara.dashboards"
        placement_key = "row-count"
        field = "placements[row-count].width"
        original_width = 4
        injected_width = 0
        canonical_blueprint = Get-Sprint8AContainmentArtifact -Path $source
        fault_blueprint = Get-Sprint8AContainmentArtifact -Path $destination
        expected_failure_pattern = $faultPattern
    }
}

function Assert-Sprint8AFailureReceipt {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Fingerprint,
        [Parameter(Mandatory)][int]$AttemptNumber
    )
    $receipt = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    if ([string]$receipt.contract -cne "tessara.sprint-8a.materialization-failure" -or
        [int]$receipt.attempt -ne $AttemptNumber -or
        [string]$receipt.environment.declared_fingerprint -cne $Fingerprint.ToLowerInvariant() -or
        [string]$receipt.expected_fault.id -cne $faultId -or
        [string]$receipt.expected_fault.classification -cne "validation_only_fault_injection" -or
        -not [bool]$receipt.expected_fault.observed -or
        -not [bool]$receipt.teardown_passed -or
        [bool]$receipt.retained_partial_topology) {
        throw "Induced owner-bootstrap failure receipt is incomplete or not exact."
    }
    $rawApply = @($receipt.raw_artifacts | Where-Object path -CLike '*/failed-apply-response.log')
    if ($rawApply.Count -ne 1) {
        throw "Induced owner-bootstrap failure did not retain one hashed raw apply response."
    }
    $verifiedRawArtifacts = @($receipt.raw_artifacts | ForEach-Object {
        Assert-Sprint8AContainmentReferencedArtifact -Artifact $_ -Label "Induced owner-bootstrap raw evidence"
    })
    $verifiedTeardown = Assert-Sprint8AContainmentReferencedArtifact `
        -Artifact $receipt.teardown `
        -Label "Induced owner-bootstrap teardown evidence"
    $teardownPath = Resolve-Sprint8AContainmentPath -Path ([string]$verifiedTeardown.path)
    $teardown = Get-Content -LiteralPath $teardownPath -Raw | ConvertFrom-Json
    if (-not [bool]$teardown.teardown.passed -or -not [bool]$teardown.teardown.after.empty -or
        @($teardown.teardown.after.containers).Count -ne 0 -or
        @($teardown.teardown.after.present_volumes).Count -ne 0 -or
        @($teardown.teardown.after.present_networks).Count -ne 0) {
        throw "Induced owner-bootstrap failure teardown did not prove the exact topology absent."
    }
    $receipt | Add-Member -NotePropertyName verified_evidence -NotePropertyValue ([pscustomobject][ordered]@{
        raw_artifacts = $verifiedRawArtifacts
        teardown = $verifiedTeardown
    }) -Force
    $receipt
}

function Assert-Sprint8ASuccessorReceipt {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Fingerprint,
        [Parameter(Mandatory)][int]$AttemptNumber,
        [Parameter(Mandatory)][string]$ExpectedBlueprintHash
    )
    $receipt = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    if ([string]$receipt.contract -cne "tessara.sprint-8a.materialization-result" -or
        -not [bool]$receipt.passed -or
        [int]$receipt.attempt -ne $AttemptNumber -or
        [string]$receipt.environment.declared_fingerprint -cne $Fingerprint.ToLowerInvariant() -or
        [string]$receipt.inputs.blueprint.sha256 -cne $ExpectedBlueprintHash -or
        [bool]$receipt.first_apply.no_op -or
        -not [bool]$receipt.no_op_apply.no_op -or
        -not [bool]$receipt.final_health_passed) {
        throw "Clean successor materialization did not prove first apply, no-op, and final health."
    }
    $verifiedEmptyBaseline = Assert-Sprint8AContainmentReferencedArtifact `
        -Artifact $receipt.evidence.empty_baseline `
        -Label "Clean successor empty-baseline evidence"
    $baselinePath = Resolve-Sprint8AContainmentPath -Path ([string]$verifiedEmptyBaseline.path)
    $baseline = Get-Content -LiteralPath $baselinePath -Raw | ConvertFrom-Json
    if (-not [bool]$baseline.empty -or -not [bool]$baseline.teardown.after.empty -or
        @($baseline.teardown.after.containers).Count -ne 0 -or
        @($baseline.teardown.after.present_volumes).Count -ne 0 -or
        @($baseline.teardown.after.present_networks).Count -ne 0) {
        throw "Clean successor did not begin from an independently proven empty baseline."
    }
    $verifiedSuccessorEvidence = [ordered]@{ empty_baseline = $verifiedEmptyBaseline }
    foreach ($artifactName in @("first_apply_response", "no_op_apply_response", "final_health")) {
        $artifact = $receipt.evidence.PSObject.Properties[$artifactName].Value
        if ($null -eq $artifact) {
            throw "Clean successor is missing hashed '$artifactName' evidence."
        }
        $verifiedSuccessorEvidence[$artifactName] = Assert-Sprint8AContainmentReferencedArtifact `
            -Artifact $artifact `
            -Label "Clean successor $artifactName evidence"
    }
    $finalHealthPath = Resolve-Sprint8AContainmentPath -Path ([string]$verifiedSuccessorEvidence.final_health.path)
    $finalHealth = Get-Content -LiteralPath $finalHealthPath -Raw | ConvertFrom-Json
    $verifiedPrecedingApply = Assert-Sprint8AContainmentReferencedArtifact `
        -Artifact $finalHealth.preceding_apply_response `
        -Label "Clean successor final-health preceding-apply evidence"
    if ([string]$finalHealth.contract -cne "tessara.sprint-8a.final-health" -or
        [int]$finalHealth.attempt -ne $AttemptNumber -or
        [string]$finalHealth.environment_fingerprint -cne $Fingerprint.ToLowerInvariant() -or
        -not [bool]$finalHealth.preceding_apply_no_op -or
        [string]$finalHealth.preceding_apply_response.sha256 -cne
            [string]$receipt.evidence.no_op_apply_response.sha256 -or
        -not [bool]$finalHealth.health.passed) {
        throw "Clean successor final health is not bound to its exact no-op apply response."
    }
    $verifiedPrecedingPath = Resolve-Sprint8AContainmentPath -Path ([string]$verifiedPrecedingApply.path)
    $verifiedNoOpPath = Resolve-Sprint8AContainmentPath -Path ([string]$verifiedSuccessorEvidence.no_op_apply_response.path)
    if ([string]$verifiedPrecedingApply.sha256 -cne [string]$verifiedSuccessorEvidence.no_op_apply_response.sha256 -or
        -not [string]::Equals($verifiedPrecedingPath, $verifiedNoOpPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Clean successor final health references a different retained no-op apply artifact."
    }
    $verifiedSuccessorEvidence.final_health_preceding_apply_response = $verifiedPrecedingApply
    $receipt | Add-Member -NotePropertyName verified_evidence -NotePropertyValue ([pscustomobject]$verifiedSuccessorEvidence) -Force
    $receipt
}

function Invoke-Sprint8ACanonicalRestoration {
    param(
        [Parameter(Mandatory)]$Paths,
        [Parameter(Mandatory)][string]$CanonicalBlueprintPath,
        [Parameter(Mandatory)][string]$ComposePath,
        [Parameter(Mandatory)][string]$CoreEndpoint,
        [Parameter(Mandatory)][string]$SupervisorEndpoint,
        [Parameter(Mandatory)][string]$Fingerprint,
        [Parameter(Mandatory)][int]$AttemptNumber,
        [Parameter(Mandatory)]$ExpectedSource,
        [Parameter(Mandatory)][bool]$SkipImageBuild
    )
    $resultPath = Join-Path $Paths.recovery_materialization_directory "materialization-result.json"
    $failurePath = Join-Path $Paths.recovery_materialization_directory "materialization-failure.json"
    $outcome = [ordered]@{
        contract = "tessara.sprint-8a.canonical-restoration"
        passed = $false
        restored = $false
        materialization_directory = Get-Sprint8AContainmentDisplayPath -Path $Paths.recovery_materialization_directory
        materialization_result = $null
        materialization_failure = $null
        exact_teardown = $null
        teardown_passed = $false
        final_health = $null
        message = $null
    }
    try {
        & (Join-Path $PSScriptRoot "materialize-sprint-8a.ps1") `
            -ComposeFile $ComposePath `
            -BlueprintPath $CanonicalBlueprintPath `
            -CoreUrl $CoreEndpoint `
            -SupervisorUrl $SupervisorEndpoint `
            -Attempt $AttemptNumber `
            -EvidenceRoot $Paths.recovery_root `
            -EnvironmentFingerprint $Fingerprint `
            -AuthorizeDisposableReset `
            -SkipBuild:$SkipImageBuild `
            -VerifyNoOp `
            -Confirm:$false | Out-Host

        $receipt = Assert-Sprint8ASuccessorReceipt `
            -Path $resultPath `
            -Fingerprint $Fingerprint `
            -AttemptNumber $AttemptNumber `
            -ExpectedBlueprintHash (Get-Sprint7AFileSha256 -Path $CanonicalBlueprintPath)
        if ([string]$receipt.source.commit -cne [string]$ExpectedSource.commit -or
            [string]$receipt.source.tree -cne [string]$ExpectedSource.tree) {
            throw "Canonical recovery materialization does not bind the original failure-containment source identity."
        }
        $outcome.materialization_result = Get-Sprint8AContainmentArtifact -Path $resultPath -RequireSidecar
        $outcome.exact_teardown = $receipt.verified_evidence.empty_baseline
        $outcome.teardown_passed = $true
        $outcome.final_health = [ordered]@{
            receipt = $receipt.verified_evidence.final_health
            preceding_apply_response = $receipt.verified_evidence.final_health_preceding_apply_response
        }
        $outcome.passed = $true
        $outcome.restored = $true
    } catch {
        $outcome.message = $_.Exception.Message
        if (Test-Path -LiteralPath $failurePath -PathType Leaf) {
            try {
                $failureReceipt = Get-Content -LiteralPath $failurePath -Raw | ConvertFrom-Json
                $outcome.materialization_failure = Get-Sprint8AContainmentArtifact -Path $failurePath -RequireSidecar
                $outcome.exact_teardown = Assert-Sprint8AContainmentReferencedArtifact `
                    -Artifact $failureReceipt.teardown `
                    -Label "Canonical recovery failure teardown evidence"
                $outcome.teardown_passed = [bool]$failureReceipt.teardown_passed
            } catch {
                $outcome.message = "$($outcome.message) Recovery failure evidence inspection also failed: $($_.Exception.Message)"
            }
        } elseif (Test-Path -LiteralPath $resultPath -PathType Leaf) {
            try {
                $resultReceipt = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
                $outcome.materialization_result = Get-Sprint8AContainmentArtifact -Path $resultPath -RequireSidecar
                $outcome.exact_teardown = Assert-Sprint8AContainmentReferencedArtifact `
                    -Artifact $resultReceipt.evidence.empty_baseline `
                    -Label "Canonical recovery empty-baseline evidence"
                $outcome.final_health = Assert-Sprint8AContainmentReferencedArtifact `
                    -Artifact $resultReceipt.evidence.final_health `
                    -Label "Canonical recovery final-health evidence"
            } catch {
                $outcome.message = "$($outcome.message) Recovery result evidence inspection also failed: $($_.Exception.Message)"
            }
        }
    }
    [pscustomobject]$outcome
}

function Invoke-Sprint8ARequiredRecovery {
    param(
        [Parameter(Mandatory)][bool]$FaultExecutionBegan,
        [Parameter(Mandatory)][bool]$TargetsAuthorized,
        [AllowEmptyString()][string]$AuthorizationFailure,
        [Parameter(Mandatory)][scriptblock]$Action
    )
    $outcome = [ordered]@{
        contract = "tessara.sprint-8a.required-recovery"
        required = $FaultExecutionBegan
        targets_authorized = $TargetsAuthorized
        attempted = $false
        passed = $false
        restored = $false
        reason = $null
        details = $null
    }
    if (-not $FaultExecutionBegan) {
        $outcome.reason = "fault_execution_not_started"
        return [pscustomobject]$outcome
    }
    if (-not $TargetsAuthorized) {
        $outcome.reason = if ([string]::IsNullOrWhiteSpace($AuthorizationFailure)) {
            "destructive_targets_not_authorized"
        } else {
            $AuthorizationFailure
        }
        return [pscustomobject]$outcome
    }

    $outcome.attempted = $true
    try {
        $details = & $Action
        $outcome.details = $details
        $teardownProved = $null -ne $details -and
            [bool]$details.teardown_passed -and
            $null -ne $details.exact_teardown
        $outcome.passed = $null -ne $details -and
            [bool]$details.passed -and
            [bool]$details.restored -and
            $teardownProved
        $outcome.restored = $outcome.passed
        if (-not $outcome.restored) {
            $outcome.reason = if ($null -ne $details -and $details.message) {
                [string]$details.message
            } else {
                "canonical_restoration_did_not_pass"
            }
        }
    } catch {
        $outcome.reason = $_.Exception.Message
    }
    [pscustomobject]$outcome
}

function New-Sprint8AContainmentFailureState {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Classification,
        [Parameter(Mandatory)][string]$Kind,
        [Parameter(Mandatory)][string]$Message,
        [Parameter(Mandatory)][string]$ExceptionType,
        [AllowEmptyString()][string]$ScriptStackTrace,
        [Parameter(Mandatory)][bool]$UnexpectedFaultAcceptance,
        [AllowNull()]$OriginalEvidence,
        [Parameter(Mandatory)][bool]$FaultExecutionBegan,
        [Parameter(Mandatory)][bool]$TargetsAuthorized,
        [AllowEmptyString()][string]$AuthorizationFailure,
        [Parameter(Mandatory)][scriptblock]$RecoveryAction
    )
    [pscustomobject][ordered]@{
        original_defect = [ordered]@{
            phase = $Phase
            classification = $Classification
            kind = $Kind
            message = $Message
            exception_type = $ExceptionType
            script_stack_trace = $ScriptStackTrace
            unexpected_fault_acceptance = $UnexpectedFaultAcceptance
            evidence = $OriginalEvidence
        }
        recovery = Invoke-Sprint8ARequiredRecovery `
            -FaultExecutionBegan $FaultExecutionBegan `
            -TargetsAuthorized $TargetsAuthorized `
            -AuthorizationFailure $AuthorizationFailure `
            -Action $RecoveryAction
    }
}

function Assert-Sprint8AScriptParameters {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string[]]$Expected
    )
    $tokens = $null
    $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile(
        [IO.Path]::GetFullPath($Path),
        [ref]$tokens,
        [ref]$errors
    )
    if ($errors.Count -ne 0) {
        throw "PowerShell parser rejected '$Path': $($errors[0])"
    }
    $actual = @($ast.ParamBlock.Parameters | ForEach-Object { $_.Name.VariablePath.UserPath })
    foreach ($name in $Expected) {
        if ($actual -cnotcontains $name) {
            throw "Script '$Path' is missing required parameter '$name'."
        }
    }
}

function Assert-Sprint8AThrows {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$ExpectedFragment
    )
    $threw = $false
    try {
        & $Action
    } catch {
        $threw = $true
        if (-not $_.Exception.Message.Contains($ExpectedFragment)) {
            throw "Self-test caught the wrong failure: $($_.Exception.Message)"
        }
    }
    if (-not $threw) { throw "Self-test expected failure containing '$ExpectedFragment'." }
}

function Invoke-Sprint8AFailureContainmentSelfTest {
    Assert-Sprint8AScriptParameters `
        -Path (Join-Path $PSScriptRoot "materialize-sprint-8a.ps1") `
        -Expected @(
            "Attempt", "EvidenceRoot", "EnvironmentFingerprint", "BlueprintPath",
            "CoreUrl", "SupervisorUrl", "ExpectedFaultId", "ExpectedFailurePattern"
        )
    Assert-Sprint8AScriptParameters `
        -Path (Join-Path $PSScriptRoot "bootstrap-sprint-7a-composition.ps1") `
        -Expected @("BlueprintPath", "RuntimeDirectory")

    Assert-Sprint8AThrows `
        -Action { Assert-Sprint8AContainmentLiveArguments -Authorized $false -Fingerprint ('a' * 64) } `
        -ExpectedFragment "requires -AuthorizeDisposableReset"
    Assert-Sprint8AThrows `
        -Action { Assert-Sprint8AContainmentLiveArguments -Authorized $true -Fingerprint "not-a-sha" } `
        -ExpectedFragment "EnvironmentFingerprint"

    Assert-Sprint8ADestructiveEndpoint `
        -Value "http://127.0.0.1:8088" `
        -Name "CoreUrl" `
        -ExpectedPort 8088
    Assert-Sprint8ADestructiveEndpoint `
        -Value "http://[::1]:8098/" `
        -Name "SupervisorUrl" `
        -ExpectedPort 8098

    $unsafeEndpoints = @(
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "ftp://127.0.0.1:8088/" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "https://127.0.0.1:8088/" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://192.0.2.10:8088/" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://user:secret@127.0.0.1:8088/" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://@127.0.0.1:8088/" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://127.0.0.1:8088/?probe=1" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://127.0.0.1:8088/#probe" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://127.0.0.1:8088/api" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://127.0.0.1:8088/api/.." },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "http://127.0.0.1:8098/" },
        [pscustomobject]@{ name = "CoreUrl"; expected_port = 8088; value = "127.0.0.1:8088" },
        [pscustomobject]@{ name = "SupervisorUrl"; expected_port = 8098; value = "http://127.0.0.1:8088/" }
    )
    foreach ($endpoint in $unsafeEndpoints) {
        Assert-Sprint8AThrows `
            -Action {
                Assert-Sprint8ADestructiveEndpoint `
                    -Value $endpoint.value `
                    -Name $endpoint.name `
                    -ExpectedPort $endpoint.expected_port
            } `
            -ExpectedFragment "requires $($endpoint.name)"
    }

    $recoveryProbe = [pscustomobject]@{ calls = 0 }
    $unexpectedAcceptanceState = New-Sprint8AContainmentFailureState `
        -Phase "induced_owner_bootstrap_failure" `
        -Classification "product" `
        -Kind "unexpected_fault_acceptance" `
        -Message "fault input was accepted" `
        -ExceptionType "System.InvalidOperationException" `
        -ScriptStackTrace "self-test" `
        -UnexpectedFaultAcceptance $true `
        -OriginalEvidence ([pscustomobject]@{ path = "fault/materialization-result.json" }) `
        -FaultExecutionBegan $true `
        -TargetsAuthorized $true `
        -RecoveryAction {
            $recoveryProbe.calls++
            [pscustomobject]@{
                passed = $true
                restored = $true
                teardown_passed = $true
                exact_teardown = [pscustomobject]@{ path = "recovery/empty-baseline.json" }
            }
        }
    if ($recoveryProbe.calls -ne 1 -or
        [string]$unexpectedAcceptanceState.original_defect.classification -cne "product" -or
        [string]$unexpectedAcceptanceState.original_defect.kind -cne "unexpected_fault_acceptance" -or
        -not [bool]$unexpectedAcceptanceState.original_defect.unexpected_fault_acceptance -or
        -not [bool]$unexpectedAcceptanceState.recovery.attempted -or
        -not [bool]$unexpectedAcceptanceState.recovery.passed -or
        -not [bool]$unexpectedAcceptanceState.recovery.restored -or
        [string]$unexpectedAcceptanceState.original_defect.evidence.path -cne "fault/materialization-result.json") {
        throw "Failure-containment self-test did not retain a product-classified unexpected acceptance through successful recovery."
    }

    $unprovedRecovery = Invoke-Sprint8ARequiredRecovery `
        -FaultExecutionBegan $true `
        -TargetsAuthorized $true `
        -Action {
            [pscustomobject]@{
                passed = $true
                restored = $true
                teardown_passed = $false
                exact_teardown = $null
            }
        }
    if (-not [bool]$unprovedRecovery.attempted -or
        [bool]$unprovedRecovery.passed -or
        [bool]$unprovedRecovery.restored) {
        throw "Failure-containment self-test accepted canonical restoration without exact teardown proof."
    }

    $failedRecoveryState = New-Sprint8AContainmentFailureState `
        -Phase "clean_from_empty_successor" `
        -Classification "product" `
        -Kind "canonical_successor_validation_failure" `
        -Message "successor validation failed" `
        -ExceptionType "System.InvalidOperationException" `
        -ScriptStackTrace "self-test" `
        -UnexpectedFaultAcceptance $false `
        -OriginalEvidence $null `
        -FaultExecutionBegan $true `
        -TargetsAuthorized $true `
        -RecoveryAction { throw "injected recovery failure" }
    if (-not [bool]$failedRecoveryState.recovery.attempted -or
        [bool]$failedRecoveryState.recovery.passed -or
        -not ([string]$failedRecoveryState.recovery.reason).Contains("injected recovery failure") -or
        [string]$failedRecoveryState.original_defect.message -cne "successor validation failed") {
        throw "Failure-containment self-test did not retain the original defect alongside a failed recovery outcome."
    }

    $unauthorizedRecoveryProbe = [pscustomobject]@{ calls = 0 }
    $unauthorizedRecovery = Invoke-Sprint8ARequiredRecovery `
        -FaultExecutionBegan $true `
        -TargetsAuthorized $false `
        -AuthorizationFailure "injected target authorization loss" `
        -Action { $unauthorizedRecoveryProbe.calls++ }
    if ($unauthorizedRecoveryProbe.calls -ne 0 -or
        [bool]$unauthorizedRecovery.attempted -or
        [string]$unauthorizedRecovery.reason -cne "injected target authorization loss") {
        throw "Failure-containment self-test attempted destructive recovery after target authorization was lost."
    }

    $temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-8a-containment-$([guid]::NewGuid().ToString('N'))"
    try {
        [IO.Directory]::CreateDirectory($temporaryRoot) | Out-Null
        $artifactFixtureRoot = Join-Path $temporaryRoot "artifact-contract"
        $rawApplyPath = Join-Path $artifactFixtureRoot "failed-apply-response.log"
        $rawApplyDocument = [ordered]@{ message = "Dashboard bootstrap layout is invalid" }
        Publish-Sprint7AEvidence -Document $rawApplyDocument -OutputPath $rawApplyPath | Out-Null
        $rawApplyArtifact = Get-Sprint8AContainmentArtifact -Path $rawApplyPath
        $teardownPath = Join-Path $artifactFixtureRoot "failure-teardown.json"
        $teardownDocument = [ordered]@{
            teardown = [ordered]@{
                passed = $true
                after = [ordered]@{ empty = $true; containers = @(); present_volumes = @(); present_networks = @() }
            }
        }
        Publish-Sprint7AEvidence -Document $teardownDocument -OutputPath $teardownPath | Out-Null
        $teardownArtifact = Get-Sprint8AContainmentArtifact -Path $teardownPath
        $failureReceiptPath = Join-Path $artifactFixtureRoot "materialization-failure.json"
        $failureReceiptDocument = [ordered]@{
            contract = "tessara.sprint-8a.materialization-failure"; attempt = 7
            environment = [ordered]@{ declared_fingerprint = "a" * 64 }
            expected_fault = [ordered]@{ id = $faultId; classification = "validation_only_fault_injection"; observed = $true }
            teardown_passed = $true; retained_partial_topology = $false
            raw_artifacts = @($rawApplyArtifact); teardown = $teardownArtifact
        }
        Publish-Sprint7AEvidence -Document $failureReceiptDocument -OutputPath $failureReceiptPath | Out-Null
        $verifiedFailure = Assert-Sprint8AFailureReceipt -Path $failureReceiptPath -Fingerprint ("a" * 64) -AttemptNumber 7
        if ([string]$verifiedFailure.verified_evidence.raw_artifacts[0].sidecar.sha256 -notmatch '^[0-9a-f]{64}$' -or
            [string]$verifiedFailure.verified_evidence.teardown.sidecar.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "Failure-containment self-test did not retain actual sidecar hashes for failure evidence."
        }
        Get-Sprint8AContainmentArtifact -Path $failureReceiptPath -RequireSidecar | Out-Null
        [IO.File]::AppendAllText($rawApplyPath, "tampered", [Text.UTF8Encoding]::new($false))
        Assert-Sprint8AThrows `
            -Action { Assert-Sprint8AFailureReceipt -Path $failureReceiptPath -Fingerprint ("a" * 64) -AttemptNumber 7 | Out-Null } `
            -ExpectedFragment "sidecar does not exactly bind"
        Publish-Sprint7AEvidence -Document $rawApplyDocument -OutputPath $rawApplyPath -Overwrite | Out-Null
        [IO.File]::WriteAllText("$teardownPath.sha256", "$('0' * 64)`n", [Text.UTF8Encoding]::new($false))
        Assert-Sprint8AThrows `
            -Action { Assert-Sprint8AFailureReceipt -Path $failureReceiptPath -Fingerprint ("a" * 64) -AttemptNumber 7 | Out-Null } `
            -ExpectedFragment "sidecar does not exactly bind"
        Publish-Sprint7AEvidence -Document $teardownDocument -OutputPath $teardownPath -Overwrite | Out-Null

        $baselinePath = Join-Path $artifactFixtureRoot "empty-baseline.json"
        $baselineDocument = [ordered]@{
            empty = $true
            teardown = [ordered]@{ after = [ordered]@{ empty = $true; containers = @(); present_volumes = @(); present_networks = @() } }
        }
        Publish-Sprint7AEvidence -Document $baselineDocument -OutputPath $baselinePath | Out-Null
        $firstApplyPath = Join-Path $artifactFixtureRoot "first-apply-response.log"
        Publish-Sprint7AEvidence -Document ([ordered]@{ no_op = $false }) -OutputPath $firstApplyPath | Out-Null
        $noOpApplyPath = Join-Path $artifactFixtureRoot "no-op-apply-response.log"
        $noOpDocument = [ordered]@{ no_op = $true }
        Publish-Sprint7AEvidence -Document $noOpDocument -OutputPath $noOpApplyPath | Out-Null
        $noOpArtifact = Get-Sprint8AContainmentArtifact -Path $noOpApplyPath
        $finalHealthPath = Join-Path $artifactFixtureRoot "final-health.json"
        $finalHealthDocument = [ordered]@{
            contract = "tessara.sprint-8a.final-health"; attempt = 7
            environment_fingerprint = "a" * 64; preceding_apply_no_op = $true
            preceding_apply_response = $noOpArtifact; health = [ordered]@{ passed = $true }
        }
        Publish-Sprint7AEvidence -Document $finalHealthDocument -OutputPath $finalHealthPath | Out-Null
        $successorReceiptPath = Join-Path $artifactFixtureRoot "materialization-result.json"
        $successorReceiptDocument = [ordered]@{
            contract = "tessara.sprint-8a.materialization-result"; passed = $true; attempt = 7
            environment = [ordered]@{ declared_fingerprint = "a" * 64 }
            inputs = [ordered]@{ blueprint = [ordered]@{ sha256 = "b" * 64 } }
            first_apply = [ordered]@{ no_op = $false }; no_op_apply = [ordered]@{ no_op = $true }; final_health_passed = $true
            evidence = [ordered]@{
                empty_baseline = Get-Sprint8AContainmentArtifact -Path $baselinePath
                first_apply_response = Get-Sprint8AContainmentArtifact -Path $firstApplyPath
                no_op_apply_response = $noOpArtifact
                final_health = Get-Sprint8AContainmentArtifact -Path $finalHealthPath
            }
        }
        Publish-Sprint7AEvidence -Document $successorReceiptDocument -OutputPath $successorReceiptPath | Out-Null
        $verifiedSuccessor = Assert-Sprint8ASuccessorReceipt -Path $successorReceiptPath -Fingerprint ("a" * 64) -AttemptNumber 7 -ExpectedBlueprintHash ("b" * 64)
        foreach ($verifiedName in @("empty_baseline", "first_apply_response", "no_op_apply_response", "final_health", "final_health_preceding_apply_response")) {
            if ([string]$verifiedSuccessor.verified_evidence.$verifiedName.sidecar.sha256 -notmatch '^[0-9a-f]{64}$') {
                throw "Failure-containment self-test did not retain the '$verifiedName' sidecar hash."
            }
        }
        [IO.File]::WriteAllText("$finalHealthPath.sha256", "$('0' * 64)`n", [Text.UTF8Encoding]::new($false))
        Assert-Sprint8AThrows `
            -Action { Assert-Sprint8ASuccessorReceipt -Path $successorReceiptPath -Fingerprint ("a" * 64) -AttemptNumber 7 -ExpectedBlueprintHash ("b" * 64) | Out-Null } `
            -ExpectedFragment "sidecar does not exactly bind"
        Publish-Sprint7AEvidence -Document $finalHealthDocument -OutputPath $finalHealthPath -Overwrite | Out-Null

        $materializeScript = Join-Path $PSScriptRoot "materialize-sprint-8a.ps1"
        foreach ($endpoint in $unsafeEndpoints) {
            $guardEvidenceRoot = Join-Path $temporaryRoot "materialize-guard-$([guid]::NewGuid().ToString('N'))"
            $materializeArguments = @{
                AuthorizeDisposableReset = $true
                EvidenceRoot = $guardEvidenceRoot
                WhatIf = $true
                Confirm = $false
            }
            $materializeArguments[$endpoint.name] = $endpoint.value
            Assert-Sprint8AThrows `
                -Action { & $materializeScript @materializeArguments } `
                -ExpectedFragment "requires $($endpoint.name)"
            if (Test-Path -LiteralPath $guardEvidenceRoot) {
                throw "Materialization endpoint guard mutated its evidence root for rejected $($endpoint.name)."
            }
        }

        $containmentScript = Join-Path $PSScriptRoot "run-sprint-8a-failure-containment.ps1"
        foreach ($endpoint in @(
            $unsafeEndpoints | Where-Object {
                ($_.name -ceq "CoreUrl" -and $_.value -ceq "http://192.0.2.10:8088/") -or
                $_.name -ceq "SupervisorUrl"
            }
        )) {
            $guardEvidenceRoot = Join-Path $temporaryRoot "containment-guard-$([guid]::NewGuid().ToString('N'))"
            $containmentArguments = @{
                AuthorizeDisposableReset = $true
                EnvironmentFingerprint = "a" * 64
                EvidenceRoot = $guardEvidenceRoot
            }
            $containmentArguments[$endpoint.name] = $endpoint.value
            Assert-Sprint8AThrows `
                -Action { & $containmentScript @containmentArguments } `
                -ExpectedFragment "requires $($endpoint.name)"
            if (Test-Path -LiteralPath $guardEvidenceRoot) {
                throw "Failure-containment endpoint guard mutated its evidence root for rejected $($endpoint.name)."
            }
        }

        $canonicalPath = Resolve-Sprint8AContainmentPath -Path "deploy/sprint-8a/blueprints/reference.json"
        $canonicalHash = Get-Sprint7AFileSha256 -Path $canonicalPath
        $faultPath = Join-Path $temporaryRoot "fault.json"
        $fault = New-Sprint8AOwnerBootstrapFaultBlueprint -SourcePath $canonicalPath -DestinationPath $faultPath
        if ($fault.fault_blueprint.sha256 -ceq $canonicalHash -or
            [int]$fault.injected_width -ne 0) {
            throw "Fault Blueprint self-test did not produce the exact deterministic mutation."
        }
        $faultBlueprint = Get-Content -LiteralPath $faultPath -Raw | ConvertFrom-Json
        $dashboard = @($faultBlueprint.modules | Where-Object definition_id -CEQ "tessara.dashboards")[0]
        $placement = @($dashboard.bootstrap.value.placements | Where-Object placement_key -CEQ "row-count")[0]
        if ([int]$placement.width -ne 0 -or $null -ne $placement.component_reference -or
            @($dashboard.bootstrap.receipt_bindings).Count -ne 7 -or
            (Get-Sprint7AFileSha256 -Path $canonicalPath) -cne $canonicalHash) {
            throw "Fault Blueprint self-test changed the wrong identity or canonical input."
        }
        Assert-Sprint8AThrows `
            -Action { New-Sprint8AOwnerBootstrapFaultBlueprint -SourcePath $canonicalPath -DestinationPath $faultPath | Out-Null } `
            -ExpectedFragment "Refusing to replace"

        $paths = Get-Sprint8AContainmentPaths -Root $temporaryRoot -AttemptNumber 7
        if ($paths.fault_materialization_directory -cnotlike '*failure-containment*attempt-7*fault*materialization*attempt-7' -or
            $paths.successor_materialization_directory -cnotlike '*failure-containment*attempt-7*successor*materialization*attempt-7' -or
            $paths.recovery_materialization_directory -cnotlike '*failure-containment*attempt-7*recovery*materialization*attempt-7' -or
            $paths.output_path -cnotlike '*failure-containment*attempt-7*failure-containment-result.json') {
            throw "Failure-containment self-test produced unstable attempt paths."
        }
    } finally {
        if (Test-Path -LiteralPath $temporaryRoot) {
            Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
        }
    }
    Write-Host "Sprint 8A failure-containment self-test passed without Docker or topology mutation."
    Write-Output ([pscustomobject][ordered]@{
        contract = "tessara.sprint-8a.failure-containment-self-test"
        passed = $true
    })
}

if ($SelfTest) {
    Invoke-Sprint8AFailureContainmentSelfTest
    return
}

Assert-Sprint8AContainmentLiveArguments `
    -Authorized ([bool]$AuthorizeDisposableReset) `
    -Fingerprint $EnvironmentFingerprint
Assert-Sprint8ADestructiveEndpoint -Value $CoreUrl -Name "CoreUrl" -ExpectedPort 8088
Assert-Sprint8ADestructiveEndpoint -Value $SupervisorUrl -Name "SupervisorUrl" -ExpectedPort 8098

$resolvedEvidenceRoot = Resolve-Sprint8AContainmentPath -Path $EvidenceRoot
$paths = Get-Sprint8AContainmentPaths `
    -Root $resolvedEvidenceRoot `
    -AttemptNumber $Attempt `
    -RequestedOutputPath $OutputPath
$canonicalBlueprintPath = Resolve-Sprint8AContainmentPath -Path $BlueprintPath
$source = Get-Sprint8AContainmentSourceIdentity
if ($source.dirty) {
    throw "Failure-containment execution requires a clean Git worktree."
}
if (Test-Path -LiteralPath $paths.containment_directory) {
    throw "Refusing to replace retained failure-containment attempt: $($paths.containment_directory)"
}
if ((Test-Path -LiteralPath $paths.output_path) -or (Test-Path -LiteralPath "$($paths.output_path).sha256")) {
    throw "Refusing to replace retained failure-containment result: $($paths.output_path)"
}

[IO.Directory]::CreateDirectory($paths.containment_directory) | Out-Null
$phase = "fault_input"
$faultReceiptArtifact = $null
$successorReceiptArtifact = $null
$faultExecutionBegan = $false
$unexpectedFaultAcceptance = $false
$defectClassification = "harness"
$defectKind = "fault_input_failure"
try {
    $faultBlueprintPath = Join-Path $paths.containment_directory "inputs/dashboard-owner-bootstrap-fault.json"
    $fault = New-Sprint8AOwnerBootstrapFaultBlueprint `
        -SourcePath $canonicalBlueprintPath `
        -DestinationPath $faultBlueprintPath

    $phase = "induced_owner_bootstrap_failure"
    $defectClassification = "harness"
    $defectKind = "fault_execution_or_evidence_failure"
    $faultFailed = $false
    $faultExecutionBegan = $true
    try {
        & (Join-Path $PSScriptRoot "materialize-sprint-8a.ps1") `
            -ComposeFile $ComposeFile `
            -BlueprintPath $faultBlueprintPath `
            -CoreUrl $CoreUrl `
            -SupervisorUrl $SupervisorUrl `
            -Attempt $Attempt `
            -EvidenceRoot $paths.fault_root `
            -EnvironmentFingerprint $EnvironmentFingerprint `
            -AuthorizeDisposableReset `
            -SkipBuild:$SkipBuild `
            -ExpectedFaultId $faultId `
            -ExpectedFailurePattern ([regex]::Escape($faultPattern)) `
            -Confirm:$false | Out-Host
    } catch {
        $faultFailed = $true
        $faultException = $_
    }
    if (-not $faultFailed) {
        $unexpectedFaultAcceptance = $true
        $defectClassification = "product"
        $defectKind = "unexpected_fault_acceptance"
        $unexpectedAcceptancePath = Join-Path $paths.fault_materialization_directory "materialization-result.json"
        if (Test-Path -LiteralPath $unexpectedAcceptancePath -PathType Leaf) {
            $faultReceiptArtifact = Get-Sprint8AContainmentArtifact -Path $unexpectedAcceptancePath -RequireSidecar
        }
        throw "The deterministic Dashboard owner-bootstrap fault unexpectedly materialized successfully."
    }
    $faultFailureReceiptPath = Join-Path $paths.fault_materialization_directory "materialization-failure.json"
    if (-not (Test-Path -LiteralPath $faultFailureReceiptPath -PathType Leaf)) {
        throw "Induced owner-bootstrap failure did not publish its deterministic failure receipt: $($faultException.Exception.Message)"
    }
    $faultReceipt = Assert-Sprint8AFailureReceipt `
        -Path $faultFailureReceiptPath `
        -Fingerprint $EnvironmentFingerprint `
        -AttemptNumber $Attempt
    $faultReceiptArtifact = Get-Sprint8AContainmentArtifact -Path $faultFailureReceiptPath -RequireSidecar

    $phase = "clean_from_empty_successor"
    $defectClassification = "product"
    $defectKind = "canonical_successor_validation_failure"
    & (Join-Path $PSScriptRoot "materialize-sprint-8a.ps1") `
        -ComposeFile $ComposeFile `
        -BlueprintPath $canonicalBlueprintPath `
        -CoreUrl $CoreUrl `
        -SupervisorUrl $SupervisorUrl `
        -Attempt $Attempt `
        -EvidenceRoot $paths.successor_root `
        -EnvironmentFingerprint $EnvironmentFingerprint `
        -AuthorizeDisposableReset `
        -SkipBuild `
        -VerifyNoOp `
        -Confirm:$false | Out-Host
    $successorResultPath = Join-Path $paths.successor_materialization_directory "materialization-result.json"
    $canonicalBlueprintHash = Get-Sprint7AFileSha256 -Path $canonicalBlueprintPath
    $successorReceipt = Assert-Sprint8ASuccessorReceipt `
        -Path $successorResultPath `
        -Fingerprint $EnvironmentFingerprint `
        -AttemptNumber $Attempt `
        -ExpectedBlueprintHash $canonicalBlueprintHash
    if ([string]$successorReceipt.source.commit -cne [string]$faultReceipt.source.commit -or
        [string]$successorReceipt.source.tree -cne [string]$faultReceipt.source.tree) {
        throw "Failure and successor materializations do not bind the same source identity."
    }
    $successorReceiptArtifact = Get-Sprint8AContainmentArtifact -Path $successorResultPath -RequireSidecar

    $phase = "publication"
    $defectClassification = "evidence-finalization"
    $defectKind = "failure_containment_publication_failure"
    $result = [ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.failure-containment-result"
        attempt = $Attempt
        passed = $true
        completed_at = [DateTimeOffset]::UtcNow.ToString("o")
        source = $source
        environment_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
        fault = $fault
        failure = [ordered]@{
            receipt = $faultReceiptArtifact
            raw_artifacts = $faultReceipt.verified_evidence.raw_artifacts
            teardown = $faultReceipt.verified_evidence.teardown
            expected_fault_observed = [bool]$faultReceipt.expected_fault.observed
        }
        successor = [ordered]@{
            receipt = $successorReceiptArtifact
            empty_baseline = $successorReceipt.verified_evidence.empty_baseline
            first_apply_response = $successorReceipt.verified_evidence.first_apply_response
            no_op_apply_response = $successorReceipt.verified_evidence.no_op_apply_response
            final_health = [ordered]@{
                receipt = $successorReceipt.verified_evidence.final_health
                preceding_apply_response = $successorReceipt.verified_evidence.final_health_preceding_apply_response
            }
        }
    }
    $publication = Publish-Sprint7AEvidence -Document $result -OutputPath $paths.output_path
    Write-Host "Sprint 8A failure containment passed: induced failure retained, exact teardown proved, clean successor healthy."
    Write-Host "Failure-containment summary: $($publication.path)"
    Write-Output ([pscustomobject][ordered]@{
        contract = "tessara.sprint-8a.failure-containment-result-location"
        attempt = $Attempt
        output_path = $publication.path
        output_sha256 = $publication.sha256
    })
} catch {
    $failure = $_
    $targetsRemainAuthorized = [bool]$AuthorizeDisposableReset
    $authorizationFailure = ""
    if ($targetsRemainAuthorized) {
        try {
            Assert-Sprint8ADestructiveEndpoint -Value $CoreUrl -Name "CoreUrl" -ExpectedPort 8088
            Assert-Sprint8ADestructiveEndpoint -Value $SupervisorUrl -Name "SupervisorUrl" -ExpectedPort 8098
        } catch {
            $targetsRemainAuthorized = $false
            $authorizationFailure = $_.Exception.Message
        }
    }
    $failureState = New-Sprint8AContainmentFailureState `
        -Phase $phase `
        -Classification $defectClassification `
        -Kind $defectKind `
        -Message $failure.Exception.Message `
        -ExceptionType $failure.Exception.GetType().FullName `
        -ScriptStackTrace ([string]$failure.ScriptStackTrace) `
        -UnexpectedFaultAcceptance $unexpectedFaultAcceptance `
        -OriginalEvidence ([ordered]@{
            fault_receipt = $faultReceiptArtifact
            successor_receipt = $successorReceiptArtifact
        }) `
        -FaultExecutionBegan $faultExecutionBegan `
        -TargetsAuthorized $targetsRemainAuthorized `
        -AuthorizationFailure $authorizationFailure `
        -RecoveryAction {
            Invoke-Sprint8ACanonicalRestoration `
                -Paths $paths `
                -CanonicalBlueprintPath $canonicalBlueprintPath `
                -ComposePath $ComposeFile `
                -CoreEndpoint $CoreUrl `
                -SupervisorEndpoint $SupervisorUrl `
                -Fingerprint $EnvironmentFingerprint `
                -AttemptNumber $Attempt `
                -ExpectedSource $source `
                -SkipImageBuild ([bool]$SkipBuild)
        }

    $partial = [ordered]@{
        schema_version = 2
        contract = "tessara.sprint-8a.failure-containment-result"
        attempt = $Attempt
        passed = $false
        failed_at = [DateTimeOffset]::UtcNow.ToString("o")
        failed_phase = $phase
        message = $failure.Exception.Message
        source = $source
        environment_fingerprint = $EnvironmentFingerprint.ToLowerInvariant()
        fault_failure_receipt = $faultReceiptArtifact
        successor_receipt = $successorReceiptArtifact
        original_defect = $failureState.original_defect
        recovery = $failureState.recovery
    }
    $partialPath = if (-not (Test-Path -LiteralPath $paths.output_path) -and
        -not (Test-Path -LiteralPath "$($paths.output_path).sha256")) {
        $paths.output_path
    } else {
        Join-Path $paths.containment_directory "failure-containment-failure.json"
    }
    try {
        $partialPublication = Publish-Sprint7AEvidence -Document $partial -OutputPath $partialPath
        throw "Sprint 8A failure-containment run failed in phase '$phase'; retained result: $($partialPublication.path); recovery attempted: $([bool]$failureState.recovery.attempted); restored: $([bool]$failureState.recovery.restored). $($failure.Exception.Message)"
    } catch {
        if ($_.Exception.Message.StartsWith("Sprint 8A failure-containment run failed")) { throw }
        throw "Sprint 8A failure-containment run failed in phase '$phase', and result publication failed: $($_.Exception.Message). Recovery attempted: $([bool]$failureState.recovery.attempted); restored: $([bool]$failureState.recovery.restored). Original failure: $($failure.Exception.Message)"
    }
}

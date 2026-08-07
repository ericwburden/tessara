[CmdletBinding()]
param(
    [string]$AttemptPath,
    [string]$HarvestPath,
    [string]$DefectBatchPath,
    [string]$CorrectionAuthorizationPath,
    [string]$EvidenceRoot = "artifacts/sprint-8a-closeout",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")
. (Join-Path $PSScriptRoot "sprint-8a-validation-environment.ps1")
$allowedClassifications = @(
    "preflight/setup",
    "product",
    "harness",
    "environment",
    "flaky",
    "evidence-finalization",
    "product-decision"
)

function Assert-EqualIdentity {
    param($Expected, $Actual, [string]$Label)
    $expectedJson = $Expected | ConvertTo-Json -Depth 30 -Compress
    $actualJson = $Actual | ConvertTo-Json -Depth 30 -Compress
    if ($actualJson -cne $expectedJson) { throw "$Label identity does not match the attempt." }
}

function Assert-DiagnosticReceiptHeader {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][int]$SchemaVersion,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Label
    )
    $schema = $Document.schema_version
    $authoritative = $Document.authoritative
    if ($Document.PSObject.Properties.Name -notcontains "schema_version" -or
        $Document.PSObject.Properties.Name -notcontains "sprint" -or
        $Document.PSObject.Properties.Name -notcontains "phase" -or
        $Document.PSObject.Properties.Name -notcontains "authoritative" -or
        -not ($schema -is [int] -or $schema -is [long]) -or
        [long]$schema -ne $SchemaVersion -or
        $Document.sprint -isnot [string] -or
        [string]$Document.sprint -cne "sprint-8a" -or
        $Document.phase -isnot [string] -or
        [string]$Document.phase -cne $Phase -or
        $authoritative -isnot [bool] -or
        $authoritative -ne $false) {
        throw "$Label is not the exact non-authoritative Sprint 8A receipt type."
    }
}

function Assert-MutableSourceIdentity {
    param(
        [Parameter(Mandatory)]$Source,
        [Parameter(Mandatory)][string]$VerificationState
    )
    $expectedProperties = @(
        "commit", "tree", "dirty", "branch",
        "acceptance_inventory_sha256", "deployment_inputs_sha256"
    )
    $actualProperties = @($Source.PSObject.Properties.Name | Sort-Object)
    if (($actualProperties | ConvertTo-Json -Compress) -cne
        (@($expectedProperties | Sort-Object) | ConvertTo-Json -Compress) -or
        $Source.commit -isnot [string] -or
        [string]$Source.commit -notmatch '^[0-9a-f]{40}$' -or
        $Source.tree -isnot [string] -or
        [string]$Source.tree -notmatch '^[0-9a-f]{40}$' -or
        $Source.dirty -isnot [bool] -or
        $Source.branch -isnot [string] -or
        [string]::IsNullOrWhiteSpace([string]$Source.branch) -or
        $Source.acceptance_inventory_sha256 -isnot [string] -or
        [string]$Source.acceptance_inventory_sha256 -notmatch '^[0-9a-f]{64}$' -or
        $Source.deployment_inputs_sha256 -isnot [string] -or
        [string]$Source.deployment_inputs_sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "The attempt mutable source identity claim is malformed."
    }
    if ($VerificationState -ceq "verified") {
        if ([string]$Source.branch -ceq "unverified" -or
            [string]$Source.commit -ceq ("0" * 40) -or [string]$Source.tree -ceq ("0" * 40)) {
            throw "A verified mutable source identity still carries placeholder claims."
        }
    } elseif ($VerificationState -in @("unverified", "failed")) {
        if ($Source.dirty -ne $false -or [string]$Source.branch -cne "unverified" -or
            [string]$Source.commit -cne ("0" * 40) -or [string]$Source.tree -cne ("0" * 40) -or
            [string]$Source.acceptance_inventory_sha256 -cne ("0" * 64) -or
            [string]$Source.deployment_inputs_sha256 -cne ("0" * 64)) {
            throw "An unverified mutable source claim must remain the exact explicit placeholder identity."
        }
    } else {
        throw "The attempt omits an explicit source identity verification state."
    }
}

function Assert-EnvironmentFingerprint {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Document.PSObject.Properties.Name -notcontains "environment_fingerprint" -or
        $Document.environment_fingerprint -isnot [string] -or
        [string]$Document.environment_fingerprint -notmatch '^[0-9a-f]{64}$') {
        throw "$Label environment fingerprint must be exactly 64 lowercase hexadecimal characters."
    }
}

function Assert-HashedFileEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Label,
        [switch]$SkipFileEvidence
    )

    if ([string]::IsNullOrWhiteSpace([string]$Evidence.path) -or
        [string]$Evidence.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "$Label lacks an exact path and SHA-256 digest."
    }
    if (-not $SkipFileEvidence) {
        $path = if ([IO.Path]::IsPathRooted([string]$Evidence.path)) {
            [IO.Path]::GetFullPath([string]$Evidence.path)
        } else {
            [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$Evidence.path)))
        }
        if ((Get-Sprint8AFileSha256 -Path $path) -cne [string]$Evidence.sha256) {
            throw "$Label digest does not match its retained file."
        }
    }
}

function Assert-AcyclicCheckGraph {
    param([Parameter(Mandatory)][object[]]$Checks)

    $names = @($Checks | ForEach-Object { [string]$_.name })
    if ($names.Count -eq 0 -or @($names | Sort-Object -Unique).Count -ne $names.Count -or
        @($names | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0) {
        throw "The attempt must declare one nonempty unique name for every check."
    }
    foreach ($check in $Checks) {
        foreach ($dependency in @($check.depends_on)) {
            if ([string]$dependency -ceq [string]$check.name -or $names -cnotcontains [string]$dependency) {
                throw "Check '$($check.name)' contains an invalid dependency '$dependency'."
            }
        }
    }
    $visiting = @{}
    $visited = @{}
    function Visit-Check([string]$Name) {
        if ($visiting.ContainsKey($Name)) { throw "The declared check graph contains a cycle through '$Name'." }
        if ($visited.ContainsKey($Name)) { return }
        $visiting[$Name] = $true
        $check = @($Checks | Where-Object name -CEQ $Name)[0]
        foreach ($dependency in @($check.depends_on)) { Visit-Check -Name ([string]$dependency) }
        $visiting.Remove($Name)
        $visited[$Name] = $true
    }
    foreach ($name in $names) { Visit-Check -Name $name }
}

function Assert-TerminalCheckEvidence {
    param(
        [Parameter(Mandatory)]$Declared,
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)][object[]]$TerminalChecks,
        [switch]$SkipFileEvidence
    )

    if ([string]$Result.command -cne [string]$Declared.command) {
        throw "Check '$($Declared.name)' terminal command differs from its declaration."
    }
    $state = [string]$Result.state
    if (@("passed", "failed", "blocked") -cnotcontains $state) {
        throw "Check '$($Declared.name)' is not passed, failed, or blocked."
    }
    if ($Result.PSObject.Properties.Name -notcontains "assertions_started" -or
        $Result.assertions_started -isnot [bool] -or
        $Result.PSObject.Properties.Name -notcontains "assertions_started_at") {
        throw "Check '$($Declared.name)' omits its exact assertion-start boundary."
    }
    if ($state -ceq "blocked") {
        $failedDependencies = @($Declared.depends_on | Where-Object {
            $dependencyName = [string]$_
            $dependency = @($TerminalChecks | Where-Object name -CEQ $dependencyName)
            $dependency.Count -ne 1 -or [string]$dependency[0].state -cne "passed"
        })
        if ($failedDependencies.Count -eq 0) {
            throw "Blocked check '$($Declared.name)' has no failed or blocked declared prerequisite."
        }
        $reason = [string]$Result.dependency_reason
        if ([string]::IsNullOrWhiteSpace($reason) -or
            @($failedDependencies | Where-Object { -not $reason.Contains([string]$_) }).Count -gt 0) {
            throw "Blocked check '$($Declared.name)' lacks its exact failed prerequisite names."
        }
        if ($null -ne $Result.exit_status -or
            -not [string]::IsNullOrWhiteSpace([string]$Result.started_at) -or
            [bool]$Result.assertions_started -or
            -not [string]::IsNullOrWhiteSpace([string]$Result.assertions_started_at)) {
            throw "Blocked check '$($Declared.name)' must not claim command execution."
        }
        return
    }

    if ([string]::IsNullOrWhiteSpace([string]$Result.command) -or
        [string]::IsNullOrWhiteSpace([string]$Result.started_at) -or
        [string]::IsNullOrWhiteSpace([string]$Result.ended_at)) {
        throw "Executed check '$($Declared.name)' lacks command or timestamps."
    }
    $started = ConvertTo-Sprint8ADateTimeOffset -Value $Result.started_at -Label "harvest check start"
    $ended = ConvertTo-Sprint8ADateTimeOffset -Value $Result.ended_at -Label "harvest check end"
    if (-not [bool]$Result.assertions_started -or
        [string]::IsNullOrWhiteSpace([string]$Result.assertions_started_at)) {
        throw "Executed check '$($Declared.name)' does not prove that assertions started."
    }
    $assertionsStarted = ConvertTo-Sprint8ADateTimeOffset `
        -Value $Result.assertions_started_at `
        -Label "harvest assertion start"
    if ($ended -lt $started -or
        $assertionsStarted -lt $started -or
        $assertionsStarted -gt $ended -or
        [double]$Result.duration_ms -lt 0) {
        throw "Executed check '$($Declared.name)' has invalid chronology."
    }
    if (($state -ceq "passed" -and [int]$Result.exit_status -ne 0) -or
        ($state -ceq "failed" -and [int]$Result.exit_status -eq 0)) {
        throw "Executed check '$($Declared.name)' has an exit status inconsistent with '$state'."
    }
    if ($state -ceq "failed" -and $allowedClassifications -cnotcontains [string]$Result.classification) {
        throw "Failed check '$($Declared.name)' has an unsupported classification."
    }
    if ($state -ceq "passed" -and -not [string]::IsNullOrWhiteSpace([string]$Result.classification)) {
        throw "Passed check '$($Declared.name)' must not retain a failure classification."
    }
    Assert-HashedFileEvidence -Evidence ([pscustomobject]@{
        path = [string]$Result.evidence_path
        sha256 = [string]$Result.evidence_sha256
    }) -Label "Executed check '$($Declared.name)' raw evidence" -SkipFileEvidence:$SkipFileEvidence
    if ($Result.PSObject.Properties.Name -contains "produced_evidence") {
        foreach ($evidence in @($Result.produced_evidence)) {
            Assert-HashedFileEvidence -Evidence $evidence `
                -Label "Executed check '$($Declared.name)' produced evidence" `
                -SkipFileEvidence:$SkipFileEvidence
        }
    }
}

function Assert-HarvestComplete {
    param(
        [Parameter(Mandatory)]$Attempt,
        [Parameter(Mandatory)]$Harvest,
        [Parameter(Mandatory)]$Batch,
        [switch]$SkipFileEvidence
    )

    Assert-DiagnosticReceiptHeader -Document $Attempt -SchemaVersion 2 -Phase "candidate-rehearsal" -Label "Attempt"
    Assert-DiagnosticReceiptHeader -Document $Harvest -SchemaVersion 1 -Phase "candidate-rehearsal-harvest" -Label "Harvest"
    Assert-DiagnosticReceiptHeader -Document $Batch -SchemaVersion 1 -Phase "candidate-rehearsal-defect-batch" -Label "Defect batch"
    if ($Attempt.PSObject.Properties.Name -notcontains "source_identity_verification_state") {
        throw "Attempt omits explicit mutable-source verification state."
    }
    if ($Attempt.PSObject.Properties.Name -notcontains "environment_identity" -or
        $Attempt.environment_identity.PSObject.Properties.Name -notcontains "verification_state" -or
        @("verified", "unverified", "failed") -cnotcontains [string]$Attempt.environment_identity.verification_state) {
        throw "Attempt omits explicit environment verification state."
    }
    Assert-MutableSourceIdentity -Source $Attempt.mutable_source_identity -VerificationState ([string]$Attempt.source_identity_verification_state)
    Assert-EnvironmentFingerprint -Document $Attempt -Label "Attempt"
    Assert-EnvironmentFingerprint -Document $Harvest -Label "Harvest"
    Assert-EnvironmentFingerprint -Document $Batch -Label "Defect batch"
    if ([string]$Attempt.state -cnotin @("harvesting", "failed") -or
        -not [bool]$Attempt.assertions_started -or
        [string]::IsNullOrWhiteSpace([string]$Attempt.assertions_started_at)) {
        throw "Only an assertion-bearing failed/harvesting attempt may authorize correction."
    }
    if ([string]$Harvest.state -cne "harvest_complete") {
        throw "Correction/restart is forbidden until the attempt reaches harvest_complete."
    }
    if ([string]::IsNullOrWhiteSpace([string]$Harvest.attempt_receipt.path) -or
        [string]$Harvest.attempt_receipt.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Harvest does not bind the exact failed attempt receipt and digest."
    }
    Assert-HashedFileEvidence -Evidence $Harvest.attempt_receipt -Label "Harvest attempt receipt" -SkipFileEvidence:$SkipFileEvidence
    foreach ($document in @($Harvest, $Batch)) {
        if ([int]$document.attempt -ne [int]$Attempt.attempt -or [string]$document.sprint -cne [string]$Attempt.sprint) {
            throw "Harvest and batch must bind the exact sprint and attempt."
        }
        Assert-EqualIdentity -Expected $Attempt.mutable_source_identity -Actual $document.mutable_source_identity -Label "Harvest/batch source"
        if ([string]$document.environment_fingerprint -cne [string]$Attempt.environment_fingerprint) {
            throw "Harvest/batch environment fingerprint does not match the attempt."
        }
    }

    $declared = @($Attempt.checks)
    Assert-AcyclicCheckGraph -Checks $declared
    $terminal = @($Harvest.checks)
    foreach ($check in $declared) {
        $result = @($terminal | Where-Object name -CEQ ([string]$check.name))
        if ($result.Count -ne 1) { throw "Check '$($check.name)' does not have exactly one terminal result." }
        Assert-TerminalCheckEvidence -Declared $check -Result $result[0] -TerminalChecks $terminal -SkipFileEvidence:$SkipFileEvidence
    }
    if ($terminal.Count -ne $declared.Count) { throw "Harvest contains undeclared or duplicate check results." }

    $failedTerminal = @($terminal | Where-Object state -CEQ "failed")
    $blockedTerminal = @($terminal | Where-Object state -CEQ "blocked")
    $passedTerminal = @($terminal | Where-Object state -CEQ "passed")
    $assertionBearingTerminal = @($terminal | Where-Object assertions_started -EQ $true)
    if (([string]$Attempt.source_identity_verification_state -cne "verified" -or
            [bool]$Attempt.mutable_source_identity.dirty) -and
        @($failedTerminal | Where-Object name -CEQ "validation-readiness-prerequisite").Count -ne 1) {
        throw "Unverified or dirty source claims are harvestable only when the declared source/readiness collection lane failed."
    }
    if (([string]$Attempt.environment_identity.verification_state -cne "verified" -or
            [string]$Attempt.environment_fingerprint -ceq ("0" * 64)) -and
        @($failedTerminal | Where-Object name -CEQ "validation-readiness-prerequisite").Count -ne 1) {
        throw "Unverified environment claims are harvestable only when the readiness/environment authentication lane failed."
    }
    $nestedBlocked = [Collections.Generic.List[object]]::new()
    $nestedFailed = [Collections.Generic.List[object]]::new()
    foreach ($result in $terminal) {
        if ($result.PSObject.Properties.Name -notcontains "nested_blocked_checks" -or
            $result.PSObject.Properties.Name -notcontains "nested_failed_checks") {
            throw "Terminal check '$($result.name)' omits its nested UAT terminal inventories."
        }
        $nested = @($result.nested_blocked_checks)
        if ($nested.Count -gt 0 -and [string]$result.name -cne "uat-diagnostics") {
            throw "Only the UAT diagnostic projection may report nested blocked checks."
        }
        foreach ($blockedScenario in $nested) {
            if ([string]$blockedScenario.name -notmatch '^uat-diagnostics/UAT-8A-[0-9]{2}$' -or
                [string]$blockedScenario.parent_check -cne "uat-diagnostics" -or
                [string]::IsNullOrWhiteSpace([string]$blockedScenario.dependency_reason)) {
                throw "Nested blocked UAT scenarios require exact identity, parent, and dependency reason."
            }
            $blockedBy = @($blockedScenario.blocked_by | ForEach-Object { [string]$_ })
            if ($blockedBy.Count -eq 0 -or
                @($blockedBy | Sort-Object -Unique).Count -ne $blockedBy.Count -or
                @($blockedBy | Where-Object {
                    $dependencyName = [string]$_
                    $dependency = @($terminal | Where-Object { [string]$_.name -ceq $dependencyName })
                    $dependency.Count -ne 1 -or [string]$dependency[0].state -ceq "passed" -or
                        -not ([string]$blockedScenario.dependency_reason).Contains($dependencyName)
                }).Count -gt 0) {
                throw "Nested blocked UAT scenario '$($blockedScenario.name)' does not bind every exact nonpassing prerequisite."
            }
            $nestedBlocked.Add($blockedScenario)
        }
        $nestedFailures = @($result.nested_failed_checks)
        if ($nestedFailures.Count -gt 0 -and [string]$result.name -cne "uat-diagnostics") {
            throw "Only the UAT diagnostic projection may report nested failed semantic assertions."
        }
        if ($nestedFailures.Count -gt 0 -and [string]$result.state -cne "passed") {
            throw "Nested semantic assertion failures must not double count the outer UAT projection lane as failed."
        }
        foreach ($semanticFailure in $nestedFailures) {
            $expectedName = "uat-diagnostics/$([string]$semanticFailure.scenario)/$([string]$semanticFailure.assertion_id)"
            if ([string]$semanticFailure.name -cne $expectedName -or
                [string]$semanticFailure.scenario -notmatch '^UAT-8A-[0-9]{2}$' -or
                [string]$semanticFailure.assertion_id -notmatch '^[a-z0-9-]+$' -or
                [string]$semanticFailure.parent_check -cne "uat-diagnostics" -or
                $allowedClassifications -cnotcontains [string]$semanticFailure.classification -or
                [string]::IsNullOrWhiteSpace([string]$semanticFailure.failure_reason) -or
                @($semanticFailure.raw_evidence).Count -lt 1) {
                throw "Nested failed UAT semantic assertions require exact identity, reason, evidence, parent, and allowed classification."
            }
            foreach ($evidence in @($semanticFailure.raw_evidence)) {
                Assert-HashedFileEvidence -Evidence $evidence -Label "Nested semantic failure '$expectedName' raw evidence" -SkipFileEvidence:$SkipFileEvidence
            }
            $nestedFailed.Add($semanticFailure)
        }
    }
    $nestedNames = @($nestedBlocked | ForEach-Object { [string]$_.name })
    if (@($nestedNames | Sort-Object -Unique).Count -ne $nestedNames.Count) {
        throw "Nested blocked UAT scenario identities must be unique."
    }
    $nestedFailedNames = @($nestedFailed | ForEach-Object { [string]$_.name })
    if (@($nestedFailedNames | Sort-Object -Unique).Count -ne $nestedFailedNames.Count) {
        throw "Nested failed UAT semantic assertion identities must be unique."
    }
    foreach ($field in @("assertion_count", "failure_count", "blocked_count", "nested_blocked_count", "nested_failure_count")) {
        if ($Attempt.PSObject.Properties.Name -notcontains $field) {
            throw "Attempt receipt omits exact '$field' accounting."
        }
    }
    foreach ($field in @("failed_count", "blocked_count", "passed_count", "nested_blocked_count", "nested_failed_count")) {
        if ($Harvest.PSObject.Properties.Name -notcontains $field) {
            throw "Harvest receipt omits exact '$field' accounting."
        }
    }
    if ([int]$Attempt.assertion_count -ne $assertionBearingTerminal.Count -or
        [int]$Attempt.failure_count -ne $failedTerminal.Count -or
        [int]$Attempt.blocked_count -ne $blockedTerminal.Count -or
        [int]$Attempt.nested_blocked_count -ne $nestedBlocked.Count -or
        [int]$Attempt.nested_failure_count -ne $nestedFailed.Count -or
        [int]$Harvest.failed_count -ne $failedTerminal.Count -or
        [int]$Harvest.blocked_count -ne $blockedTerminal.Count -or
        [int]$Harvest.passed_count -ne $passedTerminal.Count -or
        [int]$Harvest.nested_blocked_count -ne $nestedBlocked.Count -or
        [int]$Harvest.nested_failed_count -ne $nestedFailed.Count) {
        throw "Attempt/harvest pass, fail, block, or nested-block counts do not match terminal evidence."
    }

    if ([int]$Batch.batch -ne 1 -or [string]$Batch.state -cne "open" -or
        [string]$Batch.harvest_receipt.path -cne [string]$Harvest.receipt_path -or
        [string]$Batch.harvest_receipt.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw "The diagnostic pass must produce exactly one open batch bound to its harvest receipt and digest."
    }
    Assert-HashedFileEvidence -Evidence $Batch.harvest_receipt -Label "Defect-batch harvest receipt" -SkipFileEvidence:$SkipFileEvidence
    if ($Batch.PSObject.Properties.Name -notcontains "blocked_checks") {
        throw "The consolidated batch omits its exact blocked-check inventory."
    }
    $expectedBlockedKeys = @(
        @($blockedTerminal | ForEach-Object {
            "lane|$([string]$_.name)||$([string]$_.dependency_reason)"
        }) + @($nestedBlocked | ForEach-Object {
            $blockedBy = @($_.blocked_by | ForEach-Object { [string]$_ } | Sort-Object) -join ','
            "scenario|$([string]$_.name)|$([string]$_.parent_check)|$blockedBy|$([string]$_.dependency_reason)"
        }) | Sort-Object
    )
    $actualBlockedKeys = @($Batch.blocked_checks | ForEach-Object {
        $scope = [string]$_.scope
        $name = [string]$_.name
        $parent = [string]$_.parent_check
        $blockedBy = @(if ($_.PSObject.Properties.Name -contains "blocked_by") {
            $_.blocked_by | ForEach-Object { [string]$_ } | Sort-Object
        })
        $reason = [string]$_.dependency_reason
        if (@("lane", "scenario") -cnotcontains $scope -or
            [string]::IsNullOrWhiteSpace($name) -or
            [string]::IsNullOrWhiteSpace($reason) -or
            ($scope -ceq "lane" -and -not [string]::IsNullOrWhiteSpace($parent)) -or
            ($scope -ceq "scenario" -and ($parent -cne "uat-diagnostics" -or $blockedBy.Count -eq 0))) {
            throw "The consolidated batch contains an invalid blocked-check record."
        }
        if ($scope -ceq "scenario") {
            "$scope|$name|$parent|$($blockedBy -join ',')|$reason"
        } else {
            "$scope|$name|$parent|$reason"
        }
    } | Sort-Object)
    if (($expectedBlockedKeys -join "`n") -cne ($actualBlockedKeys -join "`n")) {
        throw "The consolidated batch blocked-check inventory does not match terminal lane and nested UAT evidence."
    }
    $defects = @($Batch.defects)
    if ([int]$Batch.defect_count -ne $defects.Count -or $defects.Count -eq 0) {
        throw "The consolidated defect count must equal its nonempty defect inventory."
    }
    $defectIds = @($defects | ForEach-Object { [string]$_.id })
    if (@($defectIds | Sort-Object -Unique).Count -ne $defectIds.Count -or
        @($defectIds | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -gt 0) {
        throw "The consolidated batch contains missing or duplicate defect identities."
    }
    foreach ($defect in $defects) {
        $checkNames = @($defect.check_names)
        $nestedAssertion = if ($defect.PSObject.Properties.Name -contains "nested_assertion") { [string]$defect.nested_assertion } else { "" }
        if ($allowedClassifications -cnotcontains [string]$defect.classification -or
            [string]::IsNullOrWhiteSpace([string]$defect.summary) -or
            (($checkNames.Count -eq 0) -eq [string]::IsNullOrWhiteSpace($nestedAssertion))) {
            throw "Defect '$($defect.id)' must bind exactly one failed lane set or one nested semantic assertion."
        }
        foreach ($checkName in $checkNames) {
            $result = @($terminal | Where-Object name -CEQ ([string]$checkName))
            if ($result.Count -ne 1 -or [string]$result[0].state -cne "failed") {
                throw "Defect '$($defect.id)' binds nonfailed or unknown check '$checkName'."
            }
        }
        $expectedRawEvidence = @($checkNames | ForEach-Object {
            $result = @($terminal | Where-Object name -CEQ ([string]$_))[0]
            [pscustomobject]@{
                path = [string]$result.evidence_path
                sha256 = [string]$result.evidence_sha256
            }
            if ($result.PSObject.Properties.Name -contains "produced_evidence") {
                @($result.produced_evidence)
            }
        })
        if (-not [string]::IsNullOrWhiteSpace($nestedAssertion)) {
            $nestedResult = @($nestedFailed | Where-Object name -CEQ $nestedAssertion)
            if ($nestedResult.Count -ne 1 -or
                [string]$nestedResult[0].classification -cne [string]$defect.classification -or
                [string]$nestedResult[0].failure_reason -cne [string]$defect.summary) {
                throw "Defect '$($defect.id)' does not bind the exact nested failed semantic assertion."
            }
            $expectedRawEvidence = @($nestedResult[0].raw_evidence)
        }
        $actualRawEvidence = @($defect.raw_evidence)
        $expectedKeys = @($expectedRawEvidence | ForEach-Object { "$([string]$_.path)|$([string]$_.sha256)" } | Sort-Object -Unique)
        $actualKeys = @($actualRawEvidence | ForEach-Object {
            Assert-HashedFileEvidence -Evidence $_ -Label "Defect '$($defect.id)' raw evidence" -SkipFileEvidence:$SkipFileEvidence
            "$([string]$_.path)|$([string]$_.sha256)"
        } | Sort-Object -Unique)
        if (($expectedKeys | ConvertTo-Json -Compress) -cne ($actualKeys | ConvertTo-Json -Compress)) {
            throw "Defect '$($defect.id)' does not bind every retained raw artifact from its failed checks."
        }
    }
    foreach ($failed in @($terminal | Where-Object state -CEQ "failed")) {
        if (@($defects | Where-Object { @($_.check_names) -ccontains [string]$failed.name }).Count -eq 0) {
            throw "Failed check '$($failed.name)' is absent from the consolidated defect batch."
        }
    }
    foreach ($nestedFailure in $nestedFailed) {
        if (@($defects | Where-Object { [string]$_.nested_assertion -ceq [string]$nestedFailure.name }).Count -ne 1) {
            throw "Nested semantic failure '$($nestedFailure.name)' is absent from or duplicated in the consolidated defect batch."
        }
    }
    if ($defects.Count -ne ($failedTerminal.Count + $nestedFailed.Count)) {
        throw "The consolidated batch double counts or omits lane and nested semantic failures."
    }
}

function Invoke-ExpectedGuardFailure {
    param([scriptblock]$Action, [string]$Label)
    try {
        & $Action
        throw "Self-test failed: $Label was accepted."
    } catch {
        if ($_.Exception.Message -like "Self-test failed:*") { throw }
    }
}

if ($SelfTest) {
    Test-Sprint8AEvidenceReferenceResolution | Out-Null
    $source = [pscustomobject]@{ commit = "a" * 40; tree = "b" * 40; dirty = $false; branch = "sprint-8a"; acceptance_inventory_sha256 = "c" * 64; deployment_inputs_sha256 = "d" * 64 }
    $attempt = [pscustomobject]@{
        schema_version = 2; sprint = "sprint-8a"; phase = "candidate-rehearsal"; authoritative = $false
        attempt = 1; state = "harvesting"; assertions_started = $true
        assertions_started_at = "2026-01-01T00:00:00Z"; mutable_source_identity = $source; environment_fingerprint = "e" * 64
        source_identity_verification_state = "verified"
        environment_identity = [pscustomobject]@{ verification_state = "verified" }
        assertion_count = 2; failure_count = 1; blocked_count = 1; nested_blocked_count = 1; nested_failure_count = 1
        checks = @(
            [pscustomobject]@{ name = "independent"; depends_on = @(); command = "fail" },
            [pscustomobject]@{ name = "uat-diagnostics"; depends_on = @(); command = "pass" },
            [pscustomobject]@{ name = "dependent"; depends_on = @("independent"); command = "blocked" }
        )
    }
    $harvest = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-harvest"; authoritative = $false
        attempt = 1; state = "harvest_complete"; receipt_path = "attempts/harvest.json"
        mutable_source_identity = $source; environment_fingerprint = "e" * 64
        attempt_receipt = [pscustomobject]@{ path = "attempts/attempt.json"; sha256 = "d" * 64 }
        failed_count = 1; blocked_count = 1; passed_count = 1; nested_blocked_count = 1; nested_failed_count = 1
        checks = @(
            [pscustomobject]@{ name = "independent"; command = "fail"; state = "failed"; classification = "harness"; dependency_reason = $null; started_at = "2026-01-01T00:00:01Z"; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:01Z"; duration_ms = 1000; exit_status = 1; evidence_path = "raw/fail.log"; evidence_sha256 = "f" * 64; produced_evidence = @(); nested_blocked_checks = @(); nested_failed_checks = @() },
            [pscustomobject]@{ name = "uat-diagnostics"; command = "pass"; state = "passed"; classification = $null; dependency_reason = $null; started_at = "2026-01-01T00:00:01Z"; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $true; assertions_started_at = "2026-01-01T00:00:01Z"; duration_ms = 1000; exit_status = 0; evidence_path = "raw/pass.log"; evidence_sha256 = "a" * 64; produced_evidence = @(); nested_blocked_checks = @([pscustomobject]@{ name = "uat-diagnostics/UAT-8A-01"; parent_check = "uat-diagnostics"; blocked_by = @("independent"); dependency_reason = "blocked by invalid prerequisite(s): independent" }); nested_failed_checks = @([pscustomobject]@{ name = "uat-diagnostics/UAT-8A-02/semantic-proof"; parent_check = "uat-diagnostics"; scenario = "UAT-8A-02"; assertion_id = "semantic-proof"; classification = "product"; failure_reason = "semantic mismatch"; raw_evidence = @([pscustomobject]@{ path = "raw/semantic.json"; sha256 = "b" * 64 }) }) },
            [pscustomobject]@{ name = "dependent"; command = "blocked"; state = "blocked"; classification = "harness"; dependency_reason = "independent failed"; started_at = $null; ended_at = "2026-01-01T00:00:02Z"; assertions_started = $false; assertions_started_at = $null; duration_ms = 0; exit_status = $null; evidence_path = $null; evidence_sha256 = $null; nested_blocked_checks = @(); nested_failed_checks = @() }
        )
    }
    $batch = [pscustomobject]@{
        schema_version = 1; sprint = "sprint-8a"; phase = "candidate-rehearsal-defect-batch"; authoritative = $false
        attempt = 1; batch = 1; state = "open"; mutable_source_identity = $source; environment_fingerprint = "e" * 64
        harvest_receipt = [pscustomobject]@{ path = "attempts/harvest.json"; sha256 = "e" * 64 }
        defect_count = 2; defects = @(
            [pscustomobject]@{ id = "R1"; classification = "harness"; summary = "failure"; check_names = @("independent"); nested_assertion = $null; raw_evidence = @([pscustomobject]@{ path = "raw/fail.log"; sha256 = "f" * 64 }) },
            [pscustomobject]@{ id = "R2"; classification = "product"; summary = "semantic mismatch"; check_names = @(); nested_assertion = "uat-diagnostics/UAT-8A-02/semantic-proof"; raw_evidence = @([pscustomobject]@{ path = "raw/semantic.json"; sha256 = "b" * 64 }) }
        )
        blocked_checks = @(
            [pscustomobject]@{ scope = "lane"; name = "dependent"; parent_check = $null; dependency_reason = "independent failed" },
            [pscustomobject]@{ scope = "scenario"; name = "uat-diagnostics/UAT-8A-01"; parent_check = "uat-diagnostics"; blocked_by = @("independent"); dependency_reason = "blocked by invalid prerequisite(s): independent" }
        )
    }
    Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence
    $placeholderSource = [pscustomobject]@{
        commit = "0" * 40; tree = "0" * 40; dirty = $false; branch = "unverified"
        acceptance_inventory_sha256 = "0" * 64; deployment_inputs_sha256 = "0" * 64
    }
    Assert-MutableSourceIdentity -Source $placeholderSource -VerificationState "failed"
    Invoke-ExpectedGuardFailure {
        Assert-MutableSourceIdentity -Source $placeholderSource -VerificationState "verified"
    } "a placeholder source claim marked verified"

    $attempt.schema_version = "2"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a string-coerced attempt schema"
    $attempt.schema_version = 2
    $harvest.phase = "candidate-rehearsal"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an inexact harvest phase"
    $harvest.phase = "candidate-rehearsal-harvest"
    $batch.authoritative = $true
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an authoritative diagnostic batch"
    $batch.authoritative = $false
    $harvest.authoritative = 0
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a numerically coerced diagnostic authority flag"
    $harvest.authoritative = $false
    $savedEnvironmentFingerprint = $attempt.environment_fingerprint
    $attempt.environment_fingerprint = "E" * 64
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an uppercase environment fingerprint"
    $attempt.environment_fingerprint = $savedEnvironmentFingerprint
    $harvest.environment_fingerprint = "e" * 63
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a malformed nested environment fingerprint"
    $harvest.environment_fingerprint = $savedEnvironmentFingerprint
    $source.dirty = 0
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a numerically coerced clean-source flag"
    $source.dirty = $false

    $harvest.checks[2].dependency_reason = ""
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a blocked check without its dependency"
    $harvest.checks[2].dependency_reason = "independent failed"
    $batch.defects[0].classification = "test"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "an unsupported classification"
    $batch.defects[0].classification = "harness"
    $batch.defect_count = 3
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a false defect count"
    $batch.defect_count = 2
    $batch.defects[0].raw_evidence = @()
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a defect that drops retained raw evidence"
    $batch.defects[0].raw_evidence = @([pscustomobject]@{ path = "raw/fail.log"; sha256 = "f" * 64 })
    $attempt.checks[2].depends_on = @("uat-diagnostics")
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a block attributed to a passing dependency"
    $attempt.checks[2].depends_on = @("independent")
    $harvest.failed_count = 2
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a false harvest failure count"
    $harvest.failed_count = 1
    $attempt.blocked_count = 2
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a false attempt blocked count"
    $attempt.blocked_count = 1
    $savedBlockedChecks = @($batch.blocked_checks)
    $batch.blocked_checks = @($batch.blocked_checks | Where-Object scope -CNE "scenario")
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a batch that drops a nested blocked scenario"
    $batch.blocked_checks = $savedBlockedChecks
    $harvest.checks[1].nested_blocked_checks[0].dependency_reason = ""
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested block without its dependency reason"
    $harvest.checks[1].nested_blocked_checks[0].dependency_reason = "blocked by invalid prerequisite(s): independent"
    $harvest.checks[1].nested_blocked_checks[0].blocked_by = @("uat-diagnostics")
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested block attributed to a passing prerequisite"
    $harvest.checks[1].nested_blocked_checks[0].blocked_by = @("independent")
    $harvest.checks[1].nested_failed_checks[0].classification = "test"
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested semantic failure with an unsupported classification"
    $harvest.checks[1].nested_failed_checks[0].classification = "product"
    $harvest.checks[1].nested_failed_checks[0].raw_evidence = @()
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a nested semantic failure without raw evidence"
    $harvest.checks[1].nested_failed_checks[0].raw_evidence = @([pscustomobject]@{ path = "raw/semantic.json"; sha256 = "b" * 64 })
    $savedNestedDefect = $batch.defects[1]
    $batch.defects = @($batch.defects[0]); $batch.defect_count = 1
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a batch that drops a nested semantic failure"
    $batch.defects = @($batch.defects[0], $savedNestedDefect); $batch.defect_count = 2
    $harvest.checks[1].state = "failed"; $harvest.checks[1].classification = "harness"; $harvest.checks[1].exit_status = 1
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "double counting a nested semantic failure as the outer UAT lane"
    $harvest.checks[1].state = "passed"; $harvest.checks[1].classification = $null; $harvest.checks[1].exit_status = 0
    $harvest.mutable_source_identity = [pscustomobject]@{ commit = "wrong" }
    Invoke-ExpectedGuardFailure { Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch -SkipFileEvidence } "a mismatched source identity"
    Write-Host "Sprint validation harvest guard adversarial self-test passed."
    return
}

foreach ($path in @($AttemptPath, $HarvestPath, $DefectBatchPath)) {
    if ([string]::IsNullOrWhiteSpace($path) -or -not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Attempt, harvest, and defect-batch paths are required and must exist."
    }
}
if ([string]::IsNullOrWhiteSpace($CorrectionAuthorizationPath)) {
    throw "CorrectionAuthorizationPath is required; correction cannot be authorized implicitly."
}
$attemptReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $AttemptPath `
    -AllowLegacyAbsolute
$harvestReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $HarvestPath `
    -AllowLegacyAbsolute
$batchReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $DefectBatchPath `
    -AllowLegacyAbsolute
$authorizationReference = Resolve-Sprint8AEvidenceReference `
    -RepositoryRoot $repoRoot `
    -EvidenceRoot $EvidenceRoot `
    -Path $CorrectionAuthorizationPath `
    -AllowLegacyAbsolute
$attemptSha = Assert-Sprint8AReceiptSidecar -Path $AttemptPath
$harvestSha = Assert-Sprint8AReceiptSidecar -Path $HarvestPath
$batchSha = Assert-Sprint8AReceiptSidecar -Path $DefectBatchPath
$attempt = Get-Content -LiteralPath $AttemptPath -Raw | ConvertFrom-Json
$harvest = Get-Content -LiteralPath $HarvestPath -Raw | ConvertFrom-Json
$batch = Get-Content -LiteralPath $DefectBatchPath -Raw | ConvertFrom-Json
Assert-HarvestComplete -Attempt $attempt -Harvest $harvest -Batch $batch

$boundAttemptPath = if ([IO.Path]::IsPathRooted([string]$harvest.attempt_receipt.path)) {
    [IO.Path]::GetFullPath([string]$harvest.attempt_receipt.path)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$harvest.attempt_receipt.path)))
}
$boundHarvestPath = if ([IO.Path]::IsPathRooted([string]$batch.harvest_receipt.path)) {
    [IO.Path]::GetFullPath([string]$batch.harvest_receipt.path)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot ([string]$batch.harvest_receipt.path)))
}
if ($boundAttemptPath -cne [IO.Path]::GetFullPath($AttemptPath) -or
    $attemptSha -cne [string]$harvest.attempt_receipt.sha256) {
    throw "The harvest does not bind the retained failed-attempt digest."
}
if ($boundHarvestPath -cne [IO.Path]::GetFullPath($HarvestPath) -or
    $harvestSha -cne [string]$batch.harvest_receipt.sha256) {
    throw "The consolidated batch does not bind the retained harvest digest."
}
$authorization = [ordered]@{
    schema_version = 1
    sprint = [string]$attempt.sprint
    phase = "candidate-rehearsal-correction-authorization"
    attempt = [int]$attempt.attempt
    authoritative = $false
    state = "authorized"
    consumption_state = "unconsumed"
    allowed_successor_phase = "validation-readiness"
    allowed_successor_count = 1
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    mutable_source_identity = $attempt.mutable_source_identity
    environment_fingerprint = [string]$attempt.environment_fingerprint
    predecessor_attempt_receipt = [ordered]@{ path = [string]$attemptReference.path; sha256 = $attemptSha }
    harvest_receipt = [ordered]@{ path = [string]$harvestReference.path; sha256 = $harvestSha }
    defect_batch = [ordered]@{ path = [string]$batchReference.path; sha256 = $batchSha }
    authorization = "tracked correction and one successor readiness attempt are permitted for this consolidated batch"
}
Publish-Sprint7AEvidence -Document $authorization -OutputPath ([string]$authorizationReference.full_path) | Out-Null
Write-Host "Validation harvest is complete; one consolidated correction/restart is authorized."

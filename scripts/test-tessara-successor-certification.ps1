[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$WarningPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $PSScriptRoot "tessara-validation-platform.psm1") -Force

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Assert-Throws([scriptblock]$Action, [string]$Pattern) {
    try { & $Action; throw "Expected failure matching '$Pattern'." }
    catch { if ($_.Exception.Message -notmatch $Pattern) { throw "Unexpected error: $($_.Exception.Message)" } }
}
function Copy-Object($Value) { $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100 }
function Get-Hash([string]$Text) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.UTF8Encoding]::new($false).GetBytes($Text)
    )).ToLowerInvariant()
}
function Get-CanonicalHash($Value) {
    $module = Get-Module tessara-validation-platform
    & $module { param($Document) Get-TessaraPlatformCanonicalJsonSha256 -Value $Document } $Value
}
function Set-PlanFingerprint($Plan) {
    $copy = [ordered]@{}
    foreach ($property in $Plan.PSObject.Properties | Where-Object Name -ne plan_fingerprint) {
        $copy[$property.Name] = $property.Value
    }
    $Plan.plan_fingerprint = Get-CanonicalHash ([pscustomobject]$copy)
}

function New-Domain([string]$Name, [string]$Class, [string]$Path,
        [string]$TestPath, [bool]$CandidateBinding, [string]$DefaultImpact) {
    [pscustomobject][ordered]@{
        name = $Name
        class = $Class
        inputs = @(
            [pscustomobject][ordered]@{ path = $Path; role = "producer" }
            [pscustomobject][ordered]@{ path = $TestPath; role = "test" }
        )
        candidate_binding = $CandidateBinding
        default_impact = $DefaultImpact
        bounded_rationale = if ($DefaultImpact -ceq "bounded") {
            "Exact producer, consumer, and test ownership closes this synthetic certification cone."
        } else { $null }
        consumers = [pscustomobject][ordered]@{
            implementation_targets = @()
            validation_lanes = @()
        }
    }
}

function New-SuccessorContract {
    $template = Get-Content -Raw (Join-Path $repo ".codex/skills/tessara-sprint-validation/assets/sprint-validation-contract.json") |
        ConvertFrom-Json -Depth 100
    $template.sprint = "sprint-9a-successor-certification"
    $template.dependency_domains = @(
        (New-Domain "module-ui-shell" "module-ui-shell" "crates/tessara-module-ui/src/application_shell.rs" "end2end/tests/composition.spec.ts" $true "bounded")
        (New-Domain "authentication-session" "authentication-session" "crates/tessara-module-ui/assets/module-shell.js" "end2end/tests/composition.spec.ts" $true "bounded")
        (New-Domain "module-ui-navigation" "module-ui-navigation" "crates/tessara-module-ui/src/shell_sidebar.rs" "end2end/tests/composition.spec.ts" $true "bounded")
        (New-Domain "module-ui-theme-layout" "module-ui-theme-layout" "crates/tessara-module-ui/src/application_shell.rs" "end2end/tests/module-ui-visual.spec.ts" $true "bounded")
        (New-Domain "response-owner" "response-owner" "crates/tessara-api/src/lib.rs" "crates/tessara-api/tests" $true "bounded")
        (New-Domain "response-consumers" "response-consumers" "crates/tessara-dashboard-ui/src" "end2end/tests/composition.spec.ts" $true "bounded")
        (New-Domain "dataset-refresh-dag" "dataset-refresh-dag" "crates/tessara-datasets/src" "end2end/tests/datasets.spec.ts" $true "bounded")
        (New-Domain "provider-contracts" "provider-contracts" "crates/tessara-module-contract/src" "crates/tessara-module-contract/tests" $true "full-replay")
        (New-Domain "migrations-seeds" "migrations-seeds" "migrations/**" "crates/tessara-api/tests" $true "full-replay")
        (New-Domain "deployment-materialization" "deployment-materialization" "deploy/**" "scripts/test-sprint-8b-dataset-module.ps1" $true "bounded")
        (New-Domain "fixtures" "fixtures" "deploy/sprint-8b/fixtures/**" "end2end/tests/datasets.spec.ts" $true "full-replay")
        (New-Domain "acceptance-inventory" "acceptance-inventory" "end2end/acceptance-manifest.json" "end2end/tests/**" $false "full-replay")
        (New-Domain "environment-contract" "environment-contract" "scripts/sprint-8a-validation-environment.ps1" "scripts/test-tessara-validation-platform.ps1" $false "full-replay")
        (New-Domain "evidence-publication" "evidence-publication" "scripts/tessara-validation-evidence-browser-batches.ps1" "scripts/test-tessara-validation-policy.ps1" $false "full-replay")
        (New-Domain "sit-runner" "phase-runner" "scripts/run-sprint-8b-sit.ps1" "scripts/test-tessara-validation-platform.ps1" $false "full-replay")
    )
    $allDomains = @($template.dependency_domains.name)
    $target = Copy-Object $template.implementation_targets[0]
    $target.id = "successor-focused-target"
    $target.dependency_domains = $allDomains
    $template.implementation_targets = @($target)
    $lanes = @(
        @{ id="shell-implementation"; phase="implementation"; kind="lane"; rank=10; domains=@("module-ui-shell","authentication-session") ; prerequisites=@() },
        @{ id="auth-readiness"; phase="validation-readiness"; kind="lane"; rank=10; domains=@("module-ui-shell","authentication-session") ; prerequisites=@("shell-implementation") },
        @{ id="shell-rehearsal"; phase="candidate-rehearsal"; kind="lane"; rank=20; domains=@("module-ui-shell","authentication-session","module-ui-navigation") ; prerequisites=@("auth-readiness") },
        @{ id="sit-runner-selftest"; phase="validation-readiness"; kind="runner-self-test"; rank=0; domains=@("sit-runner") ; prerequisites=@() },
        @{ id="candidate-freeze"; phase="validation-preflight"; kind="lane"; rank=0; domains=@("environment-contract","acceptance-inventory") ; prerequisites=@("shell-rehearsal") },
        @{ id="auth-session-sit"; phase="sit"; kind="lane"; rank=10; domains=@("module-ui-shell","authentication-session") ; prerequisites=@("candidate-freeze") },
        @{ id="browser-shell-sit"; phase="sit"; kind="lane"; rank=20; domains=@("module-ui-shell","module-ui-navigation","module-ui-theme-layout") ; prerequisites=@("candidate-freeze") },
        @{ id="deployed-smoke-sit"; phase="sit"; kind="lane"; rank=30; domains=@("module-ui-shell","authentication-session","deployment-materialization") ; prerequisites=@("auth-session-sit") },
        @{ id="dataset-dag-sit"; phase="sit"; kind="lane"; rank=200; domains=@("dataset-refresh-dag","response-owner","response-consumers") ; prerequisites=@("candidate-freeze") },
        @{ id="migration-upgrade-sit"; phase="sit"; kind="lane"; rank=300; domains=@("migrations-seeds","provider-contracts","fixtures") ; prerequisites=@("candidate-freeze") },
        @{ id="logout-scripted-uat"; phase="uat"; kind="scripted-scenario"; rank=10; domains=@("module-ui-shell","authentication-session","module-ui-navigation") ; prerequisites=@("deployed-smoke-sit") },
        @{ id="logout-manual-uat"; phase="uat"; kind="manual-scenario"; rank=20; domains=@("module-ui-shell","authentication-session","module-ui-navigation") ; prerequisites=@("logout-scripted-uat") },
        @{ id="dataset-manual-uat"; phase="uat"; kind="manual-scenario"; rank=200; domains=@("dataset-refresh-dag","fixtures") ; prerequisites=@("dataset-dag-sit") }
    ) | ForEach-Object {
        [pscustomobject][ordered]@{
            id = [string]$_.id; phase = [string]$_.phase; coverage_kind = [string]$_.kind
            risk_rank = [int]$_.rank; dependency_domains = @($_.domains)
            prerequisites = @($_.prerequisites); touches_live_state = [string]$_.phase -in @("sit", "uat")
        }
    }
    $template.lanes = $lanes
    $template.requirements = @([pscustomobject][ordered]@{
            id = "successor-requirement"
            implementation_targets = @("successor-focused-target")
            validation_lanes = @($lanes.id)
        })
    $template.implementation_slices[0].exit_targets = @("successor-focused-target")
    $template.controlled_artifact_edges[0].reconciliation_target = "successor-focused-target"
    foreach ($domain in @($template.dependency_domains)) {
        $domain.consumers.implementation_targets = @("successor-focused-target")
        $domain.consumers.validation_lanes = @($lanes | Where-Object {
                @($_.dependency_domains) -ccontains [string]$domain.name
            } | ForEach-Object id)
    }
    $template
}

function New-CompatibilityPlan($Contract, [string]$Candidate, [string]$Source,
        [string[]]$ChangedDomains = @()) {
    $fingerprints = @{}
    foreach ($domain in @($Contract.dependency_domains)) {
        $fingerprints[[string]$domain.name] = Get-Hash (
            "$([string]$domain.name)/$(if ([string]$domain.name -in $ChangedDomains) { 'changed' } else { 'stable' })"
        )
    }
    $lanes = @($Contract.lanes | ForEach-Object {
            $lane = $_
            $deps = @($lane.dependency_domains | Sort-Object | ForEach-Object {
                    [pscustomobject][ordered]@{ domain = [string]$_; sha256 = $fingerprints[[string]$_] }
                })
            $inheritance = Get-CanonicalHash ([pscustomobject][ordered]@{
                    lane = [string]$lane.id; dependencies = $deps; prerequisites = @($lane.prerequisites)
                })
            [pscustomobject][ordered]@{
                lane_id = [string]$lane.id; phase = [string]$lane.phase
                compatibility_fingerprint = Get-Hash "$Candidate/$inheritance"
                inheritance_fingerprint = $inheritance
                platform_execution_fingerprint = Get-Hash "platform"
                environment_fingerprint = Get-Hash "environment/$([string]$lane.id)"
                dependency_fingerprints = $deps
                prerequisite_compatibility_fingerprints = @()
                coverage_kind = [string]$lane.coverage_kind
                risk_rank = [int]$lane.risk_rank
                implementation_targets = @("successor-focused-target")
            }
        })
    $body = [pscustomobject][ordered]@{
        schema_version = 1; contract = "tessara.validation.compatibility-plan"; sprint = [string]$Contract.sprint
        source_identity = [pscustomobject][ordered]@{ commit=("a"*40); tree=("b"*40); dirty=$false }
        source_fingerprint = $Source; candidate_fingerprint = $Candidate
        validation_contract = [pscustomobject][ordered]@{ path="contract.json"; sha256=(Get-Hash "contract") }
        adapter = [pscustomobject][ordered]@{ path="adapter.json"; sha256=(Get-Hash "adapter") }
        platform_execution_fingerprint = Get-Hash "platform"; lanes = $lanes
    }
    [pscustomobject][ordered]@{
        schema_version = 1; contract = "tessara.validation.compatibility-plan"
        fingerprint = Get-CanonicalHash $body; body = $body
    }
}

$contract = New-SuccessorContract
$policyModule = Join-Path $PSScriptRoot "tessara-validation-policy.psm1"
Import-Module $policyModule -Force
Assert-True (Assert-TessaraValidationContract $contract) "Synthetic v3 successor contract did not validate."
$candidateA = Get-Hash "candidate-a"; $candidateB = Get-Hash "candidate-b"
$sourceA = Get-Hash "source-a"; $sourceB = Get-Hash "source-b"
$diff = Get-Hash "correction-batch"

$stableA = New-CompatibilityPlan $contract $candidateA $sourceA
$human = New-TessaraSuccessorImpactPlan $contract $stableA $stableA `
    "human-execution-mistake" $diff -AffectedItemIds @("logout-manual-uat")
Assert-True ($human.certification_mode -ceq "scenario-only") "Human mistake did not select scenario-only certification."
Assert-True ((@($human.coverage | Where-Object disposition -eq execute).id -join ',') -ceq "logout-manual-uat") `
    "Human mistake selected more than the affected manual scenario."

$evidenceB = New-CompatibilityPlan $contract $candidateA $sourceB @("evidence-publication")
$evidence = New-TessaraSuccessorImpactPlan $contract $stableA $evidenceB `
    "evidence-publication-defect" $diff -ChangedPaths @("scripts/tessara-validation-evidence-browser-batches.ps1")
Assert-True ($evidence.certification_mode -ceq "finalization-only" -and
    @($evidence.coverage | Where-Object disposition -eq execute).Count -eq 0) `
    "Evidence publication defect did not preserve immutable raw results."

$logoutB = New-CompatibilityPlan $contract $candidateB $sourceB @("module-ui-shell", "authentication-session")
$logout = New-TessaraSuccessorImpactPlan $contract $stableA $logoutB `
    "candidate-product-bounded" $diff -ChangedPaths @(
        "crates/tessara-module-ui/src/application_shell.rs",
        "crates/tessara-module-ui/assets/module-shell.js"
    ) -PreviouslyFailedItemIds @("logout-manual-uat")
$logoutExecuted = @($logout.coverage | Where-Object disposition -eq execute | ForEach-Object id)
foreach ($required in @("auth-session-sit", "browser-shell-sit", "deployed-smoke-sit", "logout-scripted-uat", "logout-manual-uat")) {
    Assert-True ($required -in $logoutExecuted) "Logout impact omitted '$required'."
}
Assert-True ("dataset-dag-sit" -notin $logoutExecuted -and "migration-upgrade-sit" -notin $logoutExecuted -and
    "dataset-manual-uat" -notin $logoutExecuted) "Logout impact included unrelated Dataset or migration coverage."

$migrationB = New-CompatibilityPlan $contract $candidateB $sourceB @("migrations-seeds")
$migration = New-TessaraSuccessorImpactPlan $contract $stableA $migrationB `
    "candidate-product-bounded" $diff -ChangedPaths @("migrations/0001.sql")
Assert-True ($migration.certification_mode -ceq "full-replay") "Migration correction did not select broad materialization replay."

$unknown = New-TessaraSuccessorImpactPlan $contract $stableA $logoutB `
    "candidate-product-bounded" $diff -ChangedPaths @("unknown/new-file.rs")
Assert-True ($unknown.certification_mode -ceq "full-replay") "Unknown path did not force complete replay."

$fingerprintMismatch = Copy-Object $logout
$inherited = @($fingerprintMismatch.coverage | Where-Object disposition -eq inherit)[0]
$inherited.successor_inheritance_fingerprint = Get-Hash "mismatch"
Set-PlanFingerprint $fingerprintMismatch
Assert-Throws { Assert-TessaraSuccessorImpactPlan $fingerprintMismatch $contract } "inheritance compatibility"

$priorCertificate = [pscustomobject]@{
    schema_version=3; policy_version="tessara-validation-v3"; phase="sit"; state="passed"
    open_defect_count=0; candidate_fingerprint=(Get-Hash "not-immediate")
    lanes=@($logout.coverage | Where-Object { $_.phase -eq "sit" } | ForEach-Object {
            [pscustomobject]@{ name=$_.id; state="passed" }
        })
}
Assert-Throws { Assert-TessaraSuccessorPredecessorCertificate $logout $priorCertificate "sit" } "immediate authenticated predecessor"

foreach ($expansion in @(
        @{ path="deploy/sprint-8b/fixtures/reference-fixture-contract.json"; domain="fixtures" },
        @{ path="scripts/sprint-8a-validation-environment.ps1"; domain="environment-contract" },
        @{ path="end2end/acceptance-manifest.json"; domain="acceptance-inventory" }
    )) {
    $next = New-CompatibilityPlan $contract $candidateB $sourceB @([string]$expansion.domain)
    $plan = New-TessaraSuccessorImpactPlan $contract $stableA $next `
        "candidate-product-bounded" $diff -ChangedPaths @([string]$expansion.path)
    Assert-True ($plan.certification_mode -ceq "full-replay") `
        "Changed $([string]$expansion.domain) did not conservatively expand the cone."
}

$certificateShape = [pscustomobject]@{
    schema_version=3; contract="tessara.validation.phase-certificate"; policy_version="tessara-validation-v3"
    platform_identity=[pscustomobject]@{release_version="2.0.0";platform_fingerprint=(Get-Hash "platform")}
    validation_adapter=[pscustomobject]@{path="adapter.json";sha256=(Get-Hash "adapter")}
    sprint=$contract.sprint;phase="sit";attempt=2;state="passed";authoritative=$true;certified_at=[datetimeoffset]::UtcNow.ToString("o")
    source_identity=[pscustomobject]@{commit=("a"*40);tree=("b"*40);dirty=$false}
    environment_fingerprint=(Get-Hash "environment");candidate_fingerprint=$candidateB
    compatibility_plan=[pscustomobject]@{path="plan.json";sha256=(Get-Hash "plan")}
    successor_impact_plan=[pscustomobject]@{path="impact.json";sha256=(Get-Hash "impact")}
    prerequisite_certificates=@();dependency_fingerprints=@([pscustomobject]@{domain="module-ui-shell";sha256=(Get-Hash "shell")})
    coverage=[pscustomobject]@{declared_lanes_sha256=(Get-Hash "lanes");lane_count=2;executed_count=1;inherited_count=1}
    lanes=@(
        [pscustomobject]@{name="auth-session-sit";state="passed";certification_basis="executed";compatibility_fingerprint=(Get-Hash "executed");dependency_domains=@("module-ui-shell");receipt=[pscustomobject]@{path="executed.json";sha256=(Get-Hash "executed-receipt")};started_at=[datetimeoffset]::UtcNow.AddMinutes(-1).ToString("o");ended_at=[datetimeoffset]::UtcNow.ToString("o");duration_ms=60000;inheritance=$null},
        [pscustomobject]@{name="dataset-dag-sit";state="passed";certification_basis="inherited_nonimpact";compatibility_fingerprint=(Get-Hash "current");dependency_domains=@("dataset-refresh-dag");receipt=[pscustomobject]@{path="prior.json";sha256=(Get-Hash "prior-receipt")};started_at=$null;ended_at=$null;duration_ms=$null;inheritance=[pscustomobject]@{prior_certificate=[pscustomobject]@{path="prior-certificate.json";sha256=(Get-Hash "prior-certificate")};prior_receipt=[pscustomobject]@{path="prior.json";sha256=(Get-Hash "prior-receipt")};predecessor_candidate_fingerprint=$candidateA;prior_compatibility_fingerprint=(Get-Hash "prior");current_compatibility_fingerprint=(Get-Hash "current");prior_inheritance_fingerprint=(Get-Hash "same");current_inheritance_fingerprint=(Get-Hash "same");prior_dependency_fingerprints=@([pscustomobject]@{domain="dataset-refresh-dag";sha256=(Get-Hash "dataset")});non_impact_rationale="Dataset dependencies are unchanged."}}
    );open_defect_count=0;cleanup_restoration=[pscustomobject]@{required=$true;state="passed";evidence=$null};evidence_index=[pscustomobject]@{path="index.json";sha256=(Get-Hash "index")}
}
Assert-TessaraJsonSchema $certificateShape phase_certificate_v3 "Successor certificate shape"
Assert-True ($null -eq $certificateShape.lanes[1].started_at -and
    $certificateShape.lanes[1].certification_basis -ceq "inherited_nonimpact") `
    "Inherited certificate coverage falsely claimed successor execution."

$unsafe = Copy-Object $logout
$unsafe.open_defect_count = 1
Set-PlanFingerprint $unsafe
Assert-Throws { Assert-TessaraSuccessorImpactPlan $unsafe $contract } "open defects"
$missingHash = Copy-Object $logout
$missingHash.correction_batch.diff_sha256 = "missing"
Assert-Throws { Assert-TessaraSuccessorImpactPlan $missingHash $contract } "schema|pattern"

"Tessara successor-certification self-test passed."

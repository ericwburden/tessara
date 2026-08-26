[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$WarningPreference = "Stop"

$repo = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $PSScriptRoot "tessara-validation-platform.psm1") -Force
Import-Module (Join-Path $PSScriptRoot "tessara-validation-policy.psm1") -Force

function Copy-Json($Value) { ($Value | ConvertTo-Json -Depth 100) | ConvertFrom-Json -Depth 100 }
function Write-Json([string]$Path, $Value) {
    $null = New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Path)
    [IO.File]::WriteAllText($Path, (($Value | ConvertTo-Json -Depth 100) + "`n"), [Text.UTF8Encoding]::new($false))
}
function Assert-Throws([scriptblock]$Action, [string]$Pattern) {
    try { & $Action; throw "Expected failure matching '$Pattern'." }
    catch {
        if ($_.Exception.Message -notmatch $Pattern) { throw "Unexpected error: $($_.Exception.Message)" }
    }
}
function Assert-True([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Get-Sha([string]$Path) { (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant() }

$contractAsset = Join-Path $repo ".codex/skills/tessara-sprint-validation/assets/sprint-validation-contract.json"
$adapterAsset = Join-Path $repo ".codex/skills/tessara-sprint-validation/assets/sprint-validation-adapter.json"
$contractTemplate = Get-Content -Raw $contractAsset | ConvertFrom-Json -Depth 100
$adapterTemplate = Get-Content -Raw $adapterAsset | ConvertFrom-Json -Depth 100
$testRoot = Join-Path $repo "artifacts/future-sprint-guardrails-selftest"
if (Test-Path -LiteralPath $testRoot) {
    $resolved = [IO.Path]::GetFullPath($testRoot)
    if (-not $resolved.StartsWith([IO.Path]::GetFullPath((Join-Path $repo "artifacts")) + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Self-test cleanup target escaped artifacts."
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
$null = New-Item -ItemType Directory -Path $testRoot

try {
    $validated = Assert-TessaraValidationAdapter -AdapterPath ([IO.Path]::GetRelativePath($repo, $adapterAsset)) -RepositoryRoot $repo
    Assert-True ($validated.validation_contract.schema_version -eq 3) "Conforming future contract and adapter did not pass."

    Assert-Throws { Assert-TessaraValidationAdapter -AdapterPath "artifacts/future-sprint-guardrails-selftest/missing.json" -RepositoryRoot $repo } "missing"

    function Write-Pair($Contract, $Adapter, [string]$Name) {
        $contractPath = Join-Path $testRoot "$Name-contract.json"
        $adapterPath = Join-Path $testRoot "$Name-adapter.json"
        $contractRelative = [IO.Path]::GetRelativePath($repo, $contractPath).Replace('\', '/')
        $adapterRelative = [IO.Path]::GetRelativePath($repo, $adapterPath).Replace('\', '/')
        $Contract.validation_platform.adapter_path = $adapterRelative
        $Adapter.validation_contract_path = $contractRelative
        Write-Json $contractPath $Contract
        Write-Json $adapterPath $Adapter
        [pscustomobject]@{ contract = $contractPath; adapter = $adapterPath; adapter_relative = $adapterRelative }
    }

    $missingLane = Write-Pair (Copy-Json $contractTemplate) (Copy-Json $adapterTemplate) "missing-lane"
    $missingLaneAdapter = Get-Content -Raw $missingLane.adapter | ConvertFrom-Json -Depth 100
    $missingLaneAdapter.lanes = @($missingLaneAdapter.lanes | Select-Object -SkipLast 1)
    Write-Json $missingLane.adapter $missingLaneAdapter
    Assert-Throws { Assert-TessaraValidationAdapter $missingLane.adapter_relative -RepositoryRoot $repo } "exactly cover"

    $extraContract = Copy-Json $contractTemplate
    $extraAdapter = Copy-Json $adapterTemplate
    $extraLane = Copy-Json $extraAdapter.lanes[0]
    $extraLane.id = "extra-lane"
    $extraLane.actions[0].id = "extra-action"
    $extraAdapter.lanes = @($extraAdapter.lanes) + @($extraLane)
    $extra = Write-Pair $extraContract $extraAdapter "extra-lane"
    Assert-Throws { Assert-TessaraValidationAdapter $extra.adapter_relative -RepositoryRoot $repo } "exactly cover"

    $malformedPath = Join-Path $testRoot "malformed-adapter.json"
    [IO.File]::WriteAllText($malformedPath, "{bad", [Text.UTF8Encoding]::new($false))
    Assert-Throws { Assert-TessaraValidationAdapter ([IO.Path]::GetRelativePath($repo, $malformedPath)) -RepositoryRoot $repo } "does not satisfy"

    $unsupportedContract = Copy-Json $contractTemplate
    $unsupportedContract.validation_platform.supported_release = "9.0.0"
    $unsupported = Write-Pair $unsupportedContract (Copy-Json $adapterTemplate) "unsupported-platform"
    Assert-Throws { Assert-TessaraValidationAdapter $unsupported.adapter_relative -RepositoryRoot $repo } "Governing validation contract is invalid|unsupported"

    $prereqAdapter = Copy-Json $adapterTemplate
    $prereqAdapter.lanes[1].prerequisites = @()
    $prereq = Write-Pair (Copy-Json $contractTemplate) $prereqAdapter "prerequisite-mismatch"
    Assert-Throws { Assert-TessaraValidationAdapter $prereq.adapter_relative -RepositoryRoot $repo } "prerequisite"

    $missingProvenance = [pscustomobject]@{
        schema_version = 3; contract = "tessara.validation.phase-certificate"; policy_version = "tessara-validation-v3"
        sprint = "sprint-9a"; phase = "sit"; state = "passed"; source_identity = @{}; compatibility_plan = @{}; lanes = @(); evidence_index = @{}
    }
    Assert-Throws { Assert-TessaraJsonSchema $missingProvenance phase_certificate_v3 "Formal receipt" } "platform_identity|validation_adapter|does not satisfy"

    $fanoutRoot = Join-Path $testRoot "fanout"
    $null = New-Item -ItemType Directory -Path $fanoutRoot
    $producer = Join-Path $fanoutRoot "producer.sql"
    $projection = Join-Path $fanoutRoot "checksum.txt"
    [IO.File]::WriteAllText($producer, "old", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($projection, "old", [Text.UTF8Encoding]::new($false))
    $fanoutContract = Copy-Json $contractTemplate
    $fanoutContract.controlled_artifact_edges[0].kind = "migration-checksum"
    $fanoutContract.controlled_artifact_edges[0].producer = [IO.Path]::GetRelativePath($repo, $producer).Replace('\', '/')
    $fanoutContract.controlled_artifact_edges[0].producer_baseline_sha256 = Get-Sha $producer
    $fanoutContract.controlled_artifact_edges[0].projections[0].path = [IO.Path]::GetRelativePath($repo, $projection).Replace('\', '/')
    $fanoutContract.controlled_artifact_edges[0].projections[0].baseline_sha256 = Get-Sha $projection
    [IO.File]::WriteAllText($producer, "new", [Text.UTF8Encoding]::new($false))
    Assert-Throws { Assert-TessaraControlledArtifactFanout $fanoutContract $repo @($fanoutContract.controlled_artifact_edges[0].producer) } "stale"
    [IO.File]::WriteAllText($projection, "new", [Text.UTF8Encoding]::new($false))
    $fanoutPass = Assert-TessaraControlledArtifactFanout $fanoutContract $repo @($fanoutContract.controlled_artifact_edges[0].producer, $fanoutContract.controlled_artifact_edges[0].projections[0].path)
    Assert-True ($fanoutPass.reconciled_edges.Count -eq 1) "Reconciled migration fanout did not pass."
    $fanoutContract.controlled_artifact_edges[0].kind = "manifest-release-catalog"
    [IO.File]::WriteAllText($projection, "old", [Text.UTF8Encoding]::new($false))
    Assert-Throws { Assert-TessaraControlledArtifactFanout $fanoutContract $repo @($fanoutContract.controlled_artifact_edges[0].producer) } "stale"

    $matrix = [pscustomobject]@{
        schema_version = 1; contract = "tessara.authorization-matrix"; boundaries = @("public")
        entries = @([pscustomobject]@{ id="one"; actor="admin"; action="read"; route="/x"; capability="x.read"; boundaries=@("public"); cases=@("positive") })
    }
    Assert-Throws { Assert-TessaraJsonSchema $matrix authorization_matrix "Authorization matrix" } "does not satisfy|not valid with the schema|at least 8"

    $phase8 = Copy-Json $contractTemplate
    $phase8.implementation_profile = [pscustomobject]@{
        kind="phase8-module-extraction"; playbook="docs/architecture/module-extraction-playbook.md"; module_definition="module.test"; transition_identity="transition.test"
        authorization_matrix="artifacts/future-sprint-guardrails-selftest/matrix.json"; authorization_target="replace-focused-target"
        ui_ownership_gate=[pscustomobject]@{ target="replace-focused-target"; consumer_cutover_targets=@("browser-smoke"); proofs=@("complete-css-ownership","generated-assets-and-digests","responsive-behavior","direct-document-navigation-parity","route-identity","bootstrap-media-types","ssr-hydration-accessibility-clean-console") }
    }
    $browser = Copy-Json $phase8.implementation_targets[0]
    $browser.id = "browser-smoke"; $browser.proof_classes = @("deployed-smoke"); $browser.prerequisites = @()
    $phase8.implementation_targets = @($phase8.implementation_targets) + @($browser)
    $phase8.implementation_slices[0].exit_targets = @($phase8.implementation_slices[0].exit_targets) + @("browser-smoke")
    $browserLane = Copy-Json $phase8.lanes[0]
    $browserLane.id = "browser-implementation"; $browserLane.prerequisites = @()
    $phase8.lanes += $browserLane
    $phase8.requirements += [pscustomobject]@{ id="browser-requirement"; implementation_targets=@("browser-smoke"); validation_lanes=@("browser-implementation") }
    Assert-Throws { Assert-TessaraValidationContract $phase8 } "authorization target"
    $phase8.implementation_targets[1].prerequisites = @("replace-focused-target")
    $phase8.lanes[-1].prerequisites = @("replace-implementation")
    $phase8.implementation_profile.ui_ownership_gate.target = "missing-ui-proof"
    Assert-Throws { Assert-TessaraValidationContract $phase8 } "UI ownership targets|independent UI"

    $predicted = Copy-Json $contractTemplate
    $predicted.fixture_policy.identity_source = "predicted-identities"
    Assert-Throws { Assert-TessaraValidationContract $predicted } "fixture_policy|does not satisfy|not valid with the schema|Required properties"
    $mutableVisual = Copy-Json $contractTemplate
    $mutableVisual.visual_contracts[0].fixture_content = "mutable-owner-content"
    Assert-Throws { Assert-TessaraValidationContract $mutableVisual } "mutable fixture content|whole-frame|not valid with the schema|Required properties"

    $harvestContract = Copy-Json $contractTemplate
    $harvestAdapter = Copy-Json $adapterTemplate
    foreach ($spec in @(
            @{ id="failure-a"; prerequisites=@(); continuation="safe-independent" },
            @{ id="sibling-b"; prerequisites=@(); continuation="safe-independent" },
            @{ id="dependent-c"; prerequisites=@("failure-a"); continuation="safe-independent" },
            @{ id="unsafe-d"; prerequisites=@(); continuation="unsafe-live-state" }
        )) {
        $target = Copy-Json $harvestContract.implementation_targets[0]
        $target.id = $spec.id; $target.prerequisites = @($spec.prerequisites); $target.continuation = $spec.continuation
        $harvestContract.implementation_targets += $target
        $harvestContract.implementation_slices[0].exit_targets += $spec.id
        $laneId = "$($spec.id)-lane"
        $targetLane = Copy-Json $harvestContract.lanes[0]
        $targetLane.id = $laneId
        $targetLane.prerequisites = @($spec.prerequisites | ForEach-Object { "$($_)-lane" })
        $harvestContract.lanes += $targetLane
        $harvestContract.requirements += [pscustomobject]@{ id="$($spec.id)-requirement"; implementation_targets=@($spec.id); validation_lanes=@($laneId) }
        $adapterLane = Copy-Json $harvestAdapter.lanes[0]
        $adapterLane.id = $laneId
        $adapterLane.prerequisites = @($targetLane.prerequisites)
        $adapterLane.actions[0].id = "$($spec.id)-action"
        $adapterLane.actions[0].implementation_target = $spec.id
        $harvestAdapter.lanes += $adapterLane
        $harvestAdapter.acceptance_inputs[0].lanes += $laneId
        $harvestAdapter.execution_inputs[0].lanes += $laneId
    }
    $harvestPair = Write-Pair $harvestContract $harvestAdapter "harvest"
    $invoker = { param($Target, $Lane) [pscustomobject]@{ state = if ($Target -eq "failure-a") { "failed" } else { "passed" }; cleanup_restoration = [pscustomobject]@{ state="passed" } } }
    $harvest = Invoke-TessaraImplementationHarvest $harvestPair.adapter_relative ("0" * 64) (Join-Path $testRoot "evidence-one") -RepositoryRoot $repo -LaneInvoker $invoker
    $stateMap = @{}; foreach ($target in $harvest.batch.targets) { $stateMap[[string]$target.id] = [string]$target.state }
    Assert-True ($stateMap["failure-a"] -eq "failed" -and $stateMap["sibling-b"] -eq "passed") "Safe independent sibling did not continue after failure."
    Assert-True ($stateMap["dependent-c"] -eq "blocked" -and $stateMap["unsafe-d"] -eq "blocked") "Dependent or unsafe target did not block."
    $provenancePath = Join-Path $repo ([string]$harvest.batch.findings[0].provenance.path)
    Assert-True ($harvest.batch.findings.Count -eq 1 -and (Test-Path $provenancePath) -and (Test-Path $harvest.batch_path)) "Failure provenance or harvested batch was not retained."
    $harvestAgain = Invoke-TessaraImplementationHarvest $harvestPair.adapter_relative ("0" * 64) (Join-Path $testRoot "evidence-one") -RepositoryRoot $repo -LaneInvoker $invoker
    Assert-True ((($harvest.batch.targets | ConvertTo-Json -Depth 20 -Compress) -ceq ($harvestAgain.batch.targets | ConvertTo-Json -Depth 20 -Compress))) "Harvested target batch is not deterministic."

    $identity = Get-TessaraValidationPlatformIdentity
    $head = (& git -C $repo rev-parse HEAD).Trim(); $tree = (& git -C $repo rev-parse 'HEAD^{tree}').Trim()
    $fakePath = Join-Path $testRoot "fake.json"
    $fakeDocument = [pscustomobject]@{
        schema_version=1; contract="tessara.validation.implementation-defect-batch"; policy_version="tessara-validation-v3"; sprint=$contractTemplate.sprint
        source_identity=[pscustomobject]@{ commit=$head; tree=$tree; dirty=$false }
        validation_contract=[pscustomobject]@{ path=[IO.Path]::GetRelativePath($repo,$contractAsset).Replace('\','/'); sha256=(Get-Sha $contractAsset) }
        validation_adapter=[pscustomobject]@{ path=[IO.Path]::GetRelativePath($repo,$adapterAsset).Replace('\','/'); sha256=(Get-Sha $adapterAsset) }
        platform_identity=[pscustomobject]@{ release_version=$identity.release_version; platform_fingerprint=$identity.platform_fingerprint }
        targets=@(); findings=@()
    }
    Write-Json $fakePath $fakeDocument
    $fakeRef = [pscustomobject]@{ path=[IO.Path]::GetRelativePath($repo,$fakePath).Replace('\','/'); sha256=(Get-Sha $fakePath) }
    $state = [pscustomobject]@{ required=$false; state="not_applicable"; evidence=$null }
    $readiness = [pscustomobject]@{
        schema_version=2; contract="tessara.implementation-readiness-result"; policy_version="tessara-validation-v3"; sprint=$contractTemplate.sprint; state="passed"; authoritative=$false
        source_identity=[pscustomobject]@{ commit=$head; tree=$tree; dirty=$false }
        validation_contract=[pscustomobject]@{ path=[IO.Path]::GetRelativePath($repo,$contractAsset).Replace('\','/'); sha256=(Get-Sha $contractAsset) }
        validation_adapter=[pscustomobject]@{ path=[IO.Path]::GetRelativePath($repo,$adapterAsset).Replace('\','/'); sha256=(Get-Sha $adapterAsset) }
        platform_identity=[pscustomobject]@{ release_version=$identity.release_version; platform_fingerprint=$identity.platform_fingerprint }
        affected_domains=@("product-source"); changed_paths=@(); fanout=@([pscustomobject]@{ edge="replace-fanout"; state="passed"; receipt=$fakeRef })
        targets=@([pscustomobject]@{ id="replace-focused-target"; state="passed"; source_identity=[pscustomobject]@{ commit=("0" * 40); tree=$tree; dirty=$false }; validation_contract_sha256=(Get-Sha $contractAsset); adapter_sha256=(Get-Sha $adapterAsset); clean_environment=$true; evidence=$fakeRef })
        slices=@([pscustomobject]@{ id="replace-slice"; state="passed"; exit_targets=@("replace-focused-target"); fanout_edges=@("replace-fanout") })
        harvested_defects=$fakeRef; known_failure_count=0; materialization=[pscustomobject]@{ required=$false; first_apply=$state; semantic_no_op=$state; recovery=$state }; cleanup_restoration=$state; evidence_index=$fakeRef
    }
    Assert-Throws { Assert-TessaraImplementationReadinessResult $readiness $contractTemplate $contractAsset -AdapterPath $adapterAsset } "stale|older source"

    $efficiency = [pscustomobject]@{
        schema_version=1; contract="tessara.validation.closeout-efficiency"; policy_version="tessara-validation-v3"; sprint="sprint-9a"
        target_count=4; attempt_count=6; first_pass_pass_count=3; first_pass_pass_rate=0.75
        findings_by_classification=@([pscustomobject]@{name="product";count=1}); findings_by_target=@([pscustomobject]@{name="target-a";count=1})
        repeated_failure_hotspots=@("target-a"); implementation_exit_gaps=@(); cleanup_restoration_failures=0
        formal_phases_avoided=@("validation-readiness"); reusable_process_lessons=@("keep focused exits current")
    }
    Assert-True (Assert-TessaraCloseoutEfficiencyReport $efficiency) "Valid closeout efficiency report failed."
    $efficiency.first_pass_pass_rate = 0.5
    Assert-Throws { Assert-TessaraCloseoutEfficiencyReport $efficiency } "pass rate"

    $historicalPath = Join-Path $repo "docs/sprints/sprint-8b-validation-contract.json"
    $historicalBefore = Get-Sha $historicalPath
    $historical = Get-Content -Raw $historicalPath | ConvertFrom-Json -Depth 100
    Assert-True (Assert-TessaraValidationContract $historical) "Historical v2 contract no longer validates."
    Assert-True (($historicalBefore -ceq (Get-Sha $historicalPath))) "Historical validation artifact was rewritten."

    "Tessara future-sprint guardrail self-test passed."
} finally {
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force }
}

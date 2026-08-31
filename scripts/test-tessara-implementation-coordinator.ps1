[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$testRoot = Join-Path $repo "artifacts/implementation-coordinator-selftest-$([guid]::NewGuid().ToString('N'))"
$module = Join-Path $repo 'scripts/tessara-validation-platform.psm1'
$sourceProbe = Join-Path $repo 'scripts/validation-platform/fixtures/synthetic-acceptance.json'
$sourceProbeBytes = [IO.File]::ReadAllBytes($sourceProbe)

function Assert-True([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Write-Json([string]$Path, $Document) {
    $null = New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Path)
    [IO.File]::WriteAllText($Path, (($Document | ConvertTo-Json -Depth 100) + "`n"), [Text.UTF8Encoding]::new($false))
}
function Get-Sha([string]$Path) { (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant() }
function Get-Ref([string]$Path) { [pscustomobject]@{ path=[IO.Path]::GetRelativePath($repo,$Path).Replace('\','/'); sha256=Get-Sha $Path } }
function Copy-Object($Value) { $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100 }
function Get-Candidate($Pair) {
    [string](Get-TessaraValidationCandidateIdentity -AdapterPath $Pair.adapter_path `
        -RepositoryRoot $repo).candidate_fingerprint
}

function New-CoordinatorPair([string]$Name, [object[]]$Specifications) {
    $pairRoot = Join-Path $testRoot $Name
    $contractPath = Join-Path $pairRoot 'contract.json'
    $adapterPath = Join-Path $pairRoot 'adapter.json'
    $evidenceRoot = Join-Path $pairRoot 'evidence'
    $contractRel = [IO.Path]::GetRelativePath($repo,$contractPath).Replace('\','/')
    $adapterRel = [IO.Path]::GetRelativePath($repo,$adapterPath).Replace('\','/')
    $evidenceRel = [IO.Path]::GetRelativePath($repo,$evidenceRoot).Replace('\','/')
    $changed = @(& git -C $repo ls-files --modified --deleted) +
        @(& git -C $repo ls-files --others --exclude-standard) +
        @(& git -C $repo diff --cached --name-only --diff-filter=A)
    $changed = @($changed | ForEach-Object { ([string]$_).Replace('\','/') } | Where-Object { $_ -and -not $_.StartsWith('artifacts/') } | Sort-Object -Unique)
    if ($changed.Count -eq 0) { $changed = @('AGENTS.md') }
    $domains = @(
        [pscustomobject]@{name='coordinator-test-input';class='other';inputs=@([pscustomobject]@{path='scripts/validation-platform/fixtures/synthetic-acceptance.json';role='producer'},[pscustomobject]@{path='scripts/validation-platform/fixtures/synthetic-acceptance.json';role='test'});candidate_binding=$true;default_impact='bounded';bounded_rationale='Synthetic target and test share one exact input.';consumers=[pscustomobject]@{implementation_targets=@();validation_lanes=@()}},
        [pscustomobject]@{name='working-tree-changes';class='other';inputs=@($changed|ForEach-Object{[pscustomobject]@{path=$_;role='producer'}});candidate_binding=$false;default_impact='full-replay';bounded_rationale=$null;consumers=[pscustomobject]@{implementation_targets=@();validation_lanes=@()}}
    )
    $targets=[Collections.Generic.List[object]]::new(); $lanes=[Collections.Generic.List[object]]::new(); $requirements=[Collections.Generic.List[object]]::new(); $adapterLanes=[Collections.Generic.List[object]]::new()
    foreach($spec in $Specifications) {
        $id=[string]$spec.id; $lane="$id-lane"; $claims=@([pscustomobject]@{kind='evidence-path';identity=$id;mode='exclusive'})
        if([bool]$spec.live){$claims += [pscustomobject]@{kind='database';identity='coordinator-live-database';mode='exclusive'}}
        $target=[pscustomobject]@{id=$id;command=[pscustomobject]@{program='pwsh';arguments=@('-NoProfile','-NonInteractive','-File','${repository_root}/scripts/verify-markdown-links.ps1');input_paths=@('scripts/verify-markdown-links.ps1');tools=@()};dependency_domains=@('coordinator-test-input');proof_classes=@('static-quality');required=$true;clean_environment=$false;prerequisites=@($spec.prerequisites);resource_claims=$claims;continuation=$(if([bool]$spec.unsafe){'unsafe-live-state'}else{'safe-independent'});slice='coordinator-slice';fanout_edges=@('coordinator-fanout')}
        $targets.Add($target)
        $lanePrereqs=@($spec.prerequisites|ForEach-Object{"$($_)-lane"})
        $lanes.Add([pscustomobject]@{id=$lane;phase='implementation';coverage_kind='lane';risk_rank=100;dependency_domains=@('coordinator-test-input');prerequisites=$lanePrereqs;touches_live_state=[bool]$spec.live})
        $requirements.Add([pscustomobject]@{id="$id-requirement";implementation_targets=@($id);validation_lanes=@($lane)})
        $adapterLanes.Add([pscustomobject]@{id=$lane;prerequisites=$lanePrereqs;environment=@();blocked_environment=@('DATABASE_URL');deadline_seconds=300;topology=[pscustomobject]@{mode='none';provider='none';on_success='destroy'};actions=@([pscustomobject]@{id="$id-action";stage='assertion';implementation_target=$id;proof_classes=@('static-quality');program='pwsh';arguments=@('-NoProfile','-NonInteractive','-File','${repository_root}/scripts/verify-markdown-links.ps1');input_paths=@('scripts/verify-markdown-links.ps1');timeout_seconds=240;tools=@()})})
    }
    $formalPhases=@('validation-readiness','candidate-rehearsal','validation-preflight','sit','uat'); $priorLane="$([string]$Specifications[0].id)-lane"
    foreach($phase in $formalPhases){$id="formal-$phase";$kind=if($phase -eq 'uat'){'manual-scenario'}else{'lane'};$lanes.Add([pscustomobject]@{id=$id;phase=$phase;coverage_kind=$kind;risk_rank=100;dependency_domains=@('coordinator-test-input');prerequisites=@($priorLane);touches_live_state=($phase -in @('sit','uat'))});$requirements[0].validation_lanes += $id;$adapterLanes.Add([pscustomobject]@{id=$id;prerequisites=@($priorLane);environment=@();blocked_environment=@('DATABASE_URL');deadline_seconds=300;topology=[pscustomobject]@{mode='none';provider='none';on_success='destroy'};actions=@([pscustomobject]@{id="$id-action";stage='assertion';implementation_target=[string]$Specifications[0].id;proof_classes=@('static-quality');program='pwsh';arguments=@('-NoProfile','-NonInteractive','-File','${repository_root}/scripts/verify-markdown-links.ps1');input_paths=@('scripts/verify-markdown-links.ps1');timeout_seconds=240;tools=@()})});$priorLane=$id}
    $domains[0].consumers.implementation_targets=@($targets.id);$domains[0].consumers.validation_lanes=@($lanes.id)
    $successor=[pscustomobject]@{enabled=$true;plan_contract='tessara.validation.successor-impact-plan';planner_entrypoint='New-TessaraSuccessorImpactPlan';validator_entrypoint='Assert-TessaraSuccessorImpactPlan';ordering='failed-direct-prerequisite-risk-v1';inheritance_policy='immediate-predecessor-authenticated-nonimpact-v1';fallback='complete-readiness-rehearsal-preflight-sit-uat'}
    $contract=[pscustomobject]@{schema_version=3;contract='tessara.validation-contract';policy_version='tessara-validation-v3';sprint='sprint-0a-coordinator';implementation_profile=[pscustomobject]@{kind='standard'};validation_platform=[pscustomobject]@{adapter_path=$adapterRel;supported_release='2.0.0';lane_entrypoint='Invoke-TessaraValidationLane';implementation_coordinator=[pscustomobject]@{entrypoint='Invoke-TessaraImplementationHarvest';ordering='evidentiary-priority-v1';execution_mode='serial-resource-safe';reuse_policy='authenticated-unchanged'};exception=$null};successor_certification=$successor;requirements=@($requirements);dependency_domains=$domains;implementation_targets=@($targets);lanes=@($lanes);controlled_artifact_edges=@([pscustomobject]@{id='coordinator-fanout';kind='other';producer='Cargo.toml';producer_baseline_sha256=('0'*64);projections=@([pscustomobject]@{path='Cargo.lock';baseline_sha256=('0'*64)});reconciliation_target=[string]$Specifications[0].id});implementation_slices=@([pscustomobject]@{id='coordinator-slice';prerequisites=@();exit_targets=@($Specifications.id);fanout_edges=@('coordinator-fanout')});fixture_policy=[pscustomobject]@{identity_source='signed-owner-readback';logical_keys=$true;signed_owner_readback=$true;prohibited_sources=@('predicted-identities','copied-inventories','historical-demo-counts','reduced-dto-replicas');cross_owner_writes=$false};visual_contracts=@([pscustomobject]@{id='coordinator-stable';fixture_content='invariant';comparison='whole-frame';stable_regions=@();semantic_assertion_targets=@()});evidence_policy=[pscustomobject]@{root=$evidenceRel;tracked=$false;successful_raw='retained_cold';phase_local_indexes=$true;final_full_integrity_audit=$true}}
    $allLaneIds=@($lanes.id);$adapter=[pscustomobject]@{schema_version=2;contract='tessara.validation.adapter';adapter_id="coordinator-$Name";validation_contract_path=$contractRel;acceptance_inputs=@([pscustomobject]@{path='docs/development-workflow.md';lanes=$allLaneIds});fixture_inputs=@();execution_inputs=@([pscustomobject]@{path='scripts/verify-markdown-links.ps1';lanes=$allLaneIds});execution_policy=[pscustomobject]@{max_parallel_lanes=4;max_attempts_per_lane=64;max_evidence_bytes=1073741824};lanes=@($adapterLanes)}
    Write-Json $contractPath $contract; Write-Json $adapterPath $adapter
    [pscustomobject]@{contract=$contract;contract_path=$contractPath;adapter=$adapter;adapter_path=$adapterPath;adapter_relative=$adapterRel;evidence_root=$evidenceRoot}
}

function Write-ModifiedReceipt([string]$Name, $Reference, [scriptblock]$Mutation) {
    $source=Join-Path $repo ([string]$Reference.path);$document=Get-Content $source -Raw|ConvertFrom-Json -Depth 100;& $Mutation $document
    $path=Join-Path $testRoot "state-receipts/$Name.json";Write-Json $path $document;Get-Ref $path
}

try {
    $null=New-Item -ItemType Directory -Force -Path $testRoot
    Import-Module $module -Force
    $specs=@(
        [pscustomobject]@{id='prerequisite-never';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='corrected-failure';prerequisites=@('prerequisite-never');live=$false;unsafe=$false},
        [pscustomobject]@{id='ordinary-never';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='command-changed';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='adapter-changed';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='contract-changed';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='environment-changed';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='dependency-changed';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='authenticated-unchanged';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='provenance-blocked';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='runtime-failure';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='runtime-safe-sibling';prerequisites=@();live=$false;unsafe=$false},
        [pscustomobject]@{id='runtime-dependent';prerequisites=@('runtime-failure');live=$false;unsafe=$false},
        [pscustomobject]@{id='live-success';prerequisites=@();live=$true;unsafe=$false},
        [pscustomobject]@{id='live-failure';prerequisites=@();live=$true;unsafe=$false},
        [pscustomobject]@{id='live-timeout';prerequisites=@();live=$true;unsafe=$false},
        [pscustomobject]@{id='live-interruption';prerequisites=@();live=$true;unsafe=$false}
    )
    $pair=New-CoordinatorPair 'priority' $specs
    $passInvoker={param($Target,$Lane,$Claims)[pscustomobject]@{state='passed';cleanup_restoration=[pscustomobject]@{state='passed'}}}
    $candidateRejected=$false
    try { Invoke-TessaraImplementationHarvest $pair.adapter_relative ('0'*64) `
            (Join-Path $pair.evidence_root 'candidate-mismatch') -RepositoryRoot $repo `
            -LaneInvoker $passInvoker | Out-Null } catch { $candidateRejected=$true }
    Assert-True $candidateRejected 'Coordinator accepted a caller candidate fingerprint mismatch.'
    Assert-True (-not (Test-Path (Join-Path $pair.evidence_root 'candidate-mismatch/implementation-coordinator'))) 'Rejected candidate published coordinator evidence.'
    $baseline=Invoke-TessaraImplementationHarvest $pair.adapter_relative (Get-Candidate $pair) $pair.evidence_root -RepositoryRoot $repo -LaneInvoker $passInvoker
    Assert-True ($baseline.state -eq 'passed') 'Baseline coordinator run did not pass.'
    $baselineRefs=@{};foreach($item in $baseline.finalization.targets){$baselineRefs[[string]$item.target]=$item.receipt}
    $failedRef=Write-ModifiedReceipt 'corrected-failed' $baselineRefs['corrected-failure'] {param($d)$d.state='failed';$d.reason='synthetic-failure';$d.disposition='executed';$d.newly_executed=$true}
    $blockedRef=Write-ModifiedReceipt 'blocked-failed' $baselineRefs['provenance-blocked'] {param($d)$d.state='failed';$d.reason='synthetic-open-failure';$d.disposition='executed';$d.newly_executed=$true}
    $focusedRef=$baselineRefs['corrected-failure'];$correctionIdentity='c'*64
    $correctedProvenancePath=Join-Path $testRoot 'corrected-provenance.json';$correctedProvenance=[pscustomobject]@{schema_version=2;contract='tessara.validation.defect-provenance';policy_version='tessara-validation-v3';sprint=$pair.contract.sprint;record_id='corrected-failure-record';status='corrected';target='corrected-failure';lane='corrected-failure-lane';classification='product';reason='corrected';source_identity=$baseline.plan.source_identity;validation_contract=Get-Ref $pair.contract_path;validation_adapter=Get-Ref $pair.adapter_path;platform_identity=[pscustomobject]@{release_version=$baseline.finalization.platform_identity.release_version;platform_fingerprint=$baseline.finalization.platform_identity.platform_fingerprint};failed_receipt=$failedRef;correction=[pscustomobject]@{identity=$correctionIdentity;changed_paths=@('scripts/validation-platform/fixtures/synthetic-acceptance.json');changed_domains=@('coordinator-test-input')};focused_reproducers=@([pscustomobject]@{id='corrected-focused';state='passed';evidence=$focusedRef})};Write-Json $correctedProvenancePath $correctedProvenance
    $openProvenancePath=Join-Path $testRoot 'open-provenance.json';$openProvenance=Copy-Object $correctedProvenance;$openProvenance.record_id='open-failure-record';$openProvenance.status='open';$openProvenance.target='provenance-blocked';$openProvenance.lane='provenance-blocked-lane';$openProvenance.classification='unresolved';$openProvenance.failed_receipt=$blockedRef;$openProvenance.correction=$null;$openProvenance.focused_reproducers=@();Write-Json $openProvenancePath $openProvenance
    $prior=@{};foreach($id in $baselineRefs.Keys){$prior[$id]=$baselineRefs[$id]}
    $prior['command-changed']=Write-ModifiedReceipt 'command' $prior['command-changed'] {param($d)$d.command_identity='1'*64}
    $prior['adapter-changed']=Write-ModifiedReceipt 'adapter' $prior['adapter-changed'] {param($d)$d.validation_adapter.sha256='2'*64}
    $prior['contract-changed']=Write-ModifiedReceipt 'contract' $prior['contract-changed'] {param($d)$d.validation_contract.sha256='3'*64}
    $prior['environment-changed']=Write-ModifiedReceipt 'environment' $prior['environment-changed'] {param($d)$d.environment_fingerprint='4'*64}
    $prior['dependency-changed']=Write-ModifiedReceipt 'dependency' $prior['dependency-changed'] {param($d)$d.dependency_fingerprints[0].sha256='5'*64}
    $identity=Get-TessaraValidationPlatformIdentity;$stateTargets=[Collections.Generic.List[object]]::new()
    foreach($spec in $specs){$id=[string]$spec.id;$previous=if($id -in @('prerequisite-never','ordinary-never','runtime-failure','runtime-safe-sibling','runtime-dependent','live-success','live-failure','live-timeout','live-interruption')){$null}elseif($id -eq 'corrected-failure'){$failedRef}elseif($id -eq 'provenance-blocked'){$blockedRef}else{$prior[$id]};$provenance=if($id -eq 'corrected-failure'){[pscustomobject]@{record=Get-Ref $correctedProvenancePath;status='corrected';classification='product';correction_identity=$correctionIdentity;focused_reproducers=@([pscustomobject]@{id='corrected-focused';state='passed';evidence=$focusedRef})}}elseif($id -eq 'provenance-blocked'){[pscustomobject]@{record=Get-Ref $openProvenancePath;status='open';classification='unresolved';correction_identity=$null;focused_reproducers=@()}}else{$null};$stateTargets.Add([pscustomobject]@{id=$id;previous_receipt=$previous;provenance=$provenance})}
    $statePath=Join-Path $testRoot 'target-state.json';$state=[pscustomobject]@{schema_version=1;contract='tessara.validation.implementation-target-state';policy_version='tessara-validation-v3';sprint=$pair.contract.sprint;validation_contract=Get-Ref $pair.contract_path;validation_adapter=Get-Ref $pair.adapter_path;platform_identity=[pscustomobject]@{release_version=$identity.release_version;platform_fingerprint=$identity.platform_fingerprint};targets=@($stateTargets)};Write-Json $statePath $state
    $calls=[Collections.Generic.List[string]]::new();$script:CoordinatorActive=0;$script:CoordinatorMaxActive=0
    $priorityInvoker={param($Target,$Lane,$Claims)$script:CoordinatorActive++;if($script:CoordinatorActive -gt $script:CoordinatorMaxActive){$script:CoordinatorMaxActive=$script:CoordinatorActive};$calls.Add($Target);$result=switch($Target){'runtime-failure'{[pscustomobject]@{state='failed';failure_stage='assertion';cleanup_restoration=[pscustomobject]@{state='passed'}}}'live-failure'{[pscustomobject]@{state='failed';failure_stage='assertion';cleanup_restoration=[pscustomobject]@{state='passed'}}}'live-timeout'{[pscustomobject]@{state='failed';failure_stage='timeout';cleanup_restoration=[pscustomobject]@{state='passed'}}}'live-interruption'{[pscustomobject]@{state='failed';failure_stage='interruption';cleanup_restoration=[pscustomobject]@{state='passed'}}}default{[pscustomobject]@{state='passed';cleanup_restoration=[pscustomobject]@{state='passed'}}}};$script:CoordinatorActive--;return $result}
    $priority=Invoke-TessaraImplementationHarvest $pair.adapter_relative (Get-Candidate $pair) (Join-Path $pair.evidence_root 'priority') -RepositoryRoot $repo -TargetStatePath ([IO.Path]::GetRelativePath($repo,$statePath).Replace('\','/')) -LaneInvoker $priorityInvoker
    $runnable=@($priority.plan.schedule|Where-Object{$_.disposition -ne 'blocked'});$position=@{};for($i=0;$i-lt$runnable.Count;$i++){$position[[string]$runnable[$i].target]=$i}
    Assert-True ($position['prerequisite-never'] -lt $position['corrected-failure']) 'Prerequisite closure did not override target priority.'
    Assert-True ($position['corrected-failure'] -lt $position['ordinary-never']) 'Corrected failure was not prioritized before an ordinary never-run target.'
    Assert-True ($position['ordinary-never'] -lt $position['command-changed']) 'Never-run target was not prioritized before affected targets.'
    Assert-True ($position['command-changed'] -lt $position['authenticated-unchanged']) 'Affected target was not prioritized before unchanged reuse.'
    foreach($id in @('command-changed','adapter-changed','contract-changed','environment-changed','dependency-changed')){Assert-True (($priority.plan.schedule|Where-Object target -eq $id).disposition -eq 'execute') "Reuse was not rejected for $id."}
    Assert-True (($priority.plan.schedule|Where-Object target -eq 'authenticated-unchanged').disposition -eq 'reuse') 'Authenticated unchanged target was not reused.'
    $reuseReceiptPath=Join-Path $repo ([string]($priority.finalization.targets|Where-Object target -eq 'authenticated-unchanged').receipt.path);$reuseReceipt=Get-Content $reuseReceiptPath -Raw|ConvertFrom-Json -Depth 100
    Assert-True ($reuseReceipt.disposition -eq 'reused' -and -not $reuseReceipt.newly_executed) 'Reused evidence was represented as newly executed.'
    Assert-True (($priority.plan.schedule|Where-Object target -eq 'provenance-blocked').disposition -eq 'blocked') 'Open provenance did not block execution.'
    Assert-True ($calls -contains 'runtime-safe-sibling') 'Safe independent sibling did not continue fail-late.'
    Assert-True (($priority.finalization.targets|Where-Object target -eq 'runtime-dependent').state -eq 'blocked') 'Dependent target did not block after prerequisite failure.'
    Assert-True ($script:CoordinatorMaxActive -eq 1) 'Conflicting resource claims executed concurrently.'
    foreach($id in @('live-success','live-failure','live-timeout','live-interruption')){$receiptPath=Join-Path $repo ([string]($priority.finalization.targets|Where-Object target -eq $id).receipt.path);$receipt=Get-Content $receiptPath -Raw|ConvertFrom-Json -Depth 100;Assert-True ($receipt.cleanup_restoration.state -eq 'passed') "Cleanup/restoration did not run after $id."}
    Assert-True ($priority.finalization.state -eq 'failed' -and $priority.finalization.open_defect_count -ge 1) 'Finalization admitted failed, blocked, or open-defect evidence.'

    $unknownPair=New-CoordinatorPair 'unknown-impact' @([pscustomobject]@{id='unknown-target';prerequisites=@();live=$false;unsafe=$false})
    $unknownPair.contract.dependency_domains=@($unknownPair.contract.dependency_domains|Where-Object name -eq 'coordinator-test-input');Write-Json $unknownPair.contract_path $unknownPair.contract
    $unknownBaseline=Invoke-TessaraImplementationHarvest $unknownPair.adapter_relative (Get-Candidate $unknownPair) $unknownPair.evidence_root -RepositoryRoot $repo -LaneInvoker $passInvoker
    $unknownIdentity=Get-TessaraValidationPlatformIdentity;$unknownStatePath=Join-Path $testRoot 'unknown-state.json';$unknownState=[pscustomobject]@{schema_version=1;contract='tessara.validation.implementation-target-state';policy_version='tessara-validation-v3';sprint=$unknownPair.contract.sprint;validation_contract=Get-Ref $unknownPair.contract_path;validation_adapter=Get-Ref $unknownPair.adapter_path;platform_identity=[pscustomobject]@{release_version=$unknownIdentity.release_version;platform_fingerprint=$unknownIdentity.platform_fingerprint};targets=@([pscustomobject]@{id='unknown-target';previous_receipt=$unknownBaseline.finalization.targets[0].receipt;provenance=$null})};Write-Json $unknownStatePath $unknownState
    $unknownRun=Invoke-TessaraImplementationHarvest $unknownPair.adapter_relative (Get-Candidate $unknownPair) (Join-Path $unknownPair.evidence_root 'successor') -RepositoryRoot $repo -TargetStatePath ([IO.Path]::GetRelativePath($repo,$unknownStatePath).Replace('\','/')) -LaneInvoker $passInvoker
    Assert-True ($unknownRun.plan.schedule[0].disposition -eq 'execute' -and $unknownRun.plan.schedule[0].rationale -eq 'unknown-dependency-impact-conservative-execution') 'Unknown dependency mapping did not force conservative execution.'

    $recoveryPair=New-CoordinatorPair 'recovery' @([pscustomobject]@{id='recovery-first';prerequisites=@();live=$false;unsafe=$false},[pscustomobject]@{id='recovery-second';prerequisites=@();live=$false;unsafe=$false})
    $recoveryCalls=@{};$recoveryInvoker={param($Target,$Lane,$Claims)if(-not$recoveryCalls.ContainsKey($Target)){$recoveryCalls[$Target]=0};$recoveryCalls[$Target]++;[pscustomobject]@{state='passed';cleanup_restoration=[pscustomobject]@{state='passed'}}};$interrupted=$false
    try{Invoke-TessaraImplementationHarvest $recoveryPair.adapter_relative (Get-Candidate $recoveryPair) $recoveryPair.evidence_root -RepositoryRoot $repo -LaneInvoker $recoveryInvoker -AfterCheckpoint {param($Target,$Ordinal)if($Ordinal -eq 1){throw 'synthetic controller loss'}}|Out-Null}catch{$interrupted=$true}
    Assert-True $interrupted 'Recovery self-test did not interrupt after a committed checkpoint.'
    $resumed=Invoke-TessaraImplementationHarvest $recoveryPair.adapter_relative (Get-Candidate $recoveryPair) $recoveryPair.evidence_root -RepositoryRoot $repo -LaneInvoker $recoveryInvoker
    Assert-True ($resumed.state -eq 'passed' -and $recoveryCalls['recovery-first'] -eq 1 -and $recoveryCalls['recovery-second'] -eq 1) 'Recovery duplicated completed work or failed to resume.'
    $failedRecoveryPair=New-CoordinatorPair 'failed-recovery' @([pscustomobject]@{id='failed-before-loss';prerequisites=@();live=$false;unsafe=$false},[pscustomobject]@{id='safe-after-loss';prerequisites=@();live=$false;unsafe=$false})
    $failedRecoveryCalls=@{};$failedRecoveryInvoker={param($Target,$Lane,$Claims)if(-not$failedRecoveryCalls.ContainsKey($Target)){$failedRecoveryCalls[$Target]=0};$failedRecoveryCalls[$Target]++;if($Target -eq 'failed-before-loss'){[pscustomobject]@{state='failed';failure_stage='assertion';cleanup_restoration=[pscustomobject]@{state='passed'}}}else{[pscustomobject]@{state='passed';cleanup_restoration=[pscustomobject]@{state='passed'}}}};$failedInterrupted=$false
    try{Invoke-TessaraImplementationHarvest $failedRecoveryPair.adapter_relative (Get-Candidate $failedRecoveryPair) $failedRecoveryPair.evidence_root -RepositoryRoot $repo -LaneInvoker $failedRecoveryInvoker -AfterCheckpoint {param($Target,$Ordinal)if($Ordinal -eq 1){throw 'synthetic controller loss after failure'}}|Out-Null}catch{$failedInterrupted=$true}
    Assert-True $failedInterrupted 'Failed-target recovery self-test did not interrupt after checkpoint.'
    $failedResumed=Invoke-TessaraImplementationHarvest $failedRecoveryPair.adapter_relative (Get-Candidate $failedRecoveryPair) $failedRecoveryPair.evidence_root -RepositoryRoot $repo -LaneInvoker $failedRecoveryInvoker
    Assert-True ($failedRecoveryCalls['failed-before-loss'] -eq 1 -and $failedRecoveryCalls['safe-after-loss'] -eq 1) 'Failed-target recovery duplicated work or blocked a safe sibling.'
    Assert-True ($failedResumed.batch.findings.Count -eq 1 -and $failedResumed.finalization.open_defect_count -eq 1) 'Recovery did not reconstruct the retained failed-target provenance.'
    [IO.File]::WriteAllBytes($sourceProbe, @($sourceProbeBytes + [byte[]](10)))
    $successor=Invoke-TessaraImplementationHarvest $recoveryPair.adapter_relative (Get-Candidate $recoveryPair) $recoveryPair.evidence_root -RepositoryRoot $repo -LaneInvoker $recoveryInvoker
    Assert-True ($successor.plan.schedule_digest -cne $resumed.plan.schedule_digest) 'Source-affecting correction did not require a new immutable plan.'
    'Tessara implementation coordinator self-test passed.'
} finally {
    [IO.File]::WriteAllBytes($sourceProbe, $sourceProbeBytes)
    if(Test-Path -LiteralPath $testRoot){Remove-Item -LiteralPath $testRoot -Recurse -Force}
}

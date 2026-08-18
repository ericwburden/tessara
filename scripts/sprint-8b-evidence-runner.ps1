Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:Sprint8BRepositoryRoot = Split-Path -Parent $PSScriptRoot
$script:Sprint8BContractPath = Join-Path $script:Sprint8BRepositoryRoot "docs/sprints/sprint-8b-validation-contract.json"
$script:Sprint8BScenarioContractPath = Join-Path $script:Sprint8BRepositoryRoot "docs/sprints/sprint-8b-uat/scenario-contract.json"
$script:Sprint8BPolicyPath = Join-Path $PSScriptRoot "tessara-validation-policy.psm1"
$script:Sprint8BResetAuthorization = "I_AUTHORIZE_THE_EXACT_SPRINT_8B_LANE_DISPOSABLE_TOPOLOGY_RESET"

Import-Module $script:Sprint8BPolicyPath -Force

function Get-Sprint8BContract {
    if (-not (Test-Path -LiteralPath $script:Sprint8BContractPath -PathType Leaf)) {
        throw "Sprint 8B validation contract is missing: $script:Sprint8BContractPath"
    }
    $contract = Get-Content -Raw -LiteralPath $script:Sprint8BContractPath | ConvertFrom-Json -Depth 100
    $null = Assert-TessaraValidationContract -Contract $contract
    if ([string]$contract.sprint -cne "sprint-8b" -or
        [string]$contract.policy_version -cne "tessara-validation-v2") {
        throw "The formal runner loaded a foreign validation contract."
    }
    return $contract
}

function Get-Sprint8BScenarioContract {
    if (-not (Test-Path -LiteralPath $script:Sprint8BScenarioContractPath -PathType Leaf)) {
        throw "Sprint 8B UAT scenario contract is missing: $script:Sprint8BScenarioContractPath"
    }
    $document = Get-Content -Raw -LiteralPath $script:Sprint8BScenarioContractPath | ConvertFrom-Json -Depth 100
    if ([int]$document.schema_version -ne 1 -or
        [string]$document.contract -cne "tessara.sprint-8b.uat-scenarios") {
        throw "Sprint 8B UAT scenario contract has the wrong identity."
    }
    $ids = @($document.scenarios | ForEach-Object { [string]$_.id })
    $expected = 1..11 | ForEach-Object { "UAT-8B-{0:D2}" -f $_ }
    Assert-Sprint8BExactSequence -Expected $expected -Actual $ids -Label "UAT scenario inventory"
    return $document
}

function Assert-Sprint8BExactSequence {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Expected,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Actual,
        [Parameter(Mandatory)][string]$Label
    )
    $expectedText = @($Expected | ForEach-Object { [string]$_ })
    $actualText = @($Actual | ForEach-Object { [string]$_ })
    if ($expectedText.Count -ne $actualText.Count) {
        throw "$Label count mismatch: expected $($expectedText.Count), found $($actualText.Count)."
    }
    for ($index = 0; $index -lt $expectedText.Count; $index++) {
        if ($expectedText[$index] -cne $actualText[$index]) {
            throw "$Label mismatch at index ${index}: expected '$($expectedText[$index])', found '$($actualText[$index])'."
        }
    }
}

function Get-Sprint8BPhaseRunnerName {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { "validate-sprint-8b-readiness.ps1" }
        "candidate-rehearsal" { "run-sprint-8b-candidate-rehearsal.ps1" }
        "validation-preflight" { "run-sprint-8b-validation-preflight.ps1" }
        "sit" { "run-sprint-8b-sit.ps1" }
        "uat" { "run-sprint-8b-formal-uat.ps1" }
        default { throw "Unknown Sprint 8B phase '$Phase'." }
    }
}

function Get-Sprint8BPhaseResultName {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { "validation-readiness-result.json" }
        "candidate-rehearsal" { "candidate-rehearsal-result.json" }
        "validation-preflight" { "preflight-result.json" }
        "sit" { "sit-result.json" }
        "uat" { "uat-result.json" }
        default { throw "Unknown Sprint 8B phase '$Phase'." }
    }
}

function Get-Sprint8BPhaseRoot {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )
    $relative = ([string]$Contract.evidence_policy.root).TrimEnd("/", "\") + "/$Phase"
    return [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot $relative))
}

function Get-Sprint8BLaneEvidencePaths {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $phaseRoot = Get-Sprint8BPhaseRoot -Contract $Contract -Phase $Phase
    $laneRoot = Join-Path $phaseRoot "lanes/$Lane"
    [pscustomobject][ordered]@{
        phase_root = $phaseRoot
        lane_root = $laneRoot
        result = Join-Path $laneRoot "result.json"
        command_log = Join-Path $laneRoot "command.log"
        evidence_references = Join-Path $laneRoot "evidence-references.json"
    }
}

function Get-Sprint8BRepositoryRelativePath {
    param([Parameter(Mandatory)][string]$Path)
    $fullRoot = [IO.Path]::GetFullPath($script:Sprint8BRepositoryRoot)
    $fullPath = [IO.Path]::GetFullPath($Path)
    if (-not $fullPath.StartsWith($fullRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Path escapes the Sprint 8B repository: $Path"
    }
    return [IO.Path]::GetRelativePath($fullRoot, $fullPath).Replace("\", "/")
}

function Get-Sprint8BSha256Text {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
    $hash = [Security.Cryptography.SHA256]::HashData($bytes)
    return [Convert]::ToHexString($hash).ToLowerInvariant()
}

function Get-Sprint8BFileReference {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Evidence file is missing: $Path"
    }
    [pscustomobject][ordered]@{
        path = Get-Sprint8BRepositoryRelativePath -Path $Path
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

function Write-Sprint8BNewUtf8File {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text
    )
    $parent = Split-Path -Parent $Path
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
}

function Write-Sprint8BNewJsonFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document
    )
    $json = $Document | ConvertTo-Json -Depth 100
    Write-Sprint8BNewUtf8File -Path $Path -Text "$json`n"
}

function Publish-Sprint8BJsonAndSidecar {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document
    )
    Write-Sprint8BNewJsonFile -Path $Path -Document $Document
    try {
        $sha = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        Write-Sprint8BNewUtf8File -Path "$Path.sha256" -Text "$sha`n"
        return $sha
    } catch {
        if (Test-Path -LiteralPath $Path -PathType Leaf) { Remove-Item -LiteralPath $Path -Force }
        throw
    }
}

function Copy-Sprint8BNewFile {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination
    )
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        throw "Publication source is missing: $Source"
    }
    $parent = Split-Path -Parent $Destination
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $input = [IO.File]::OpenRead($Source)
    $output = $null
    try {
        $output = [IO.File]::Open($Destination, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
        $input.CopyTo($output)
    } finally {
        if ($null -ne $output) { $output.Dispose() }
        $input.Dispose()
    }
}

function Get-Sprint8BSourceIdentity {
    $commit = (& git -C $script:Sprint8BRepositoryRoot rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Unable to resolve the Sprint 8B source commit."
    }
    $tree = (& git -C $script:Sprint8BRepositoryRoot rev-parse 'HEAD^{tree}').Trim()
    if ($LASTEXITCODE -ne 0 -or $tree -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Unable to resolve the Sprint 8B source tree."
    }
    $branch = (& git -C $script:Sprint8BRepositoryRoot branch --show-current).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($branch)) {
        throw "Unable to resolve the Sprint 8B branch."
    }
    $status = @(& git -C $script:Sprint8BRepositoryRoot status --short --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "Unable to resolve the Sprint 8B source status." }
    [pscustomobject][ordered]@{
        commit = $commit
        tree = $tree
        dirty = @($status).Count -gt 0
        branch = $branch
    }
}

function Assert-Sprint8BCleanSource {
    param([Parameter(Mandatory)]$Source)
    if ([bool]$Source.dirty) {
        throw "Formal Sprint 8B validation refuses a dirty source tree."
    }
}

function Get-Sprint8BProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][string[]]$Names)
    $snapshot = [ordered]@{}
    foreach ($name in $Names) {
        $snapshot[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
    }
    return $snapshot
}

function Restore-Sprint8BProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)]$Snapshot)
    foreach ($name in @($Snapshot.Keys)) {
        [Environment]::SetEnvironmentVariable([string]$name, $Snapshot[$name], "Process")
    }
}

function Assert-Sprint8BSha256Sidecar {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedSha256
    )
    $sidecar = "$Path.sha256"
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecar -PathType Leaf)) {
        throw "Authenticated evidence pair is missing: $Path"
    }
    $sidecarSha = (Get-Content -LiteralPath $sidecar -Raw).Trim()
    $actualSha = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sidecarSha -cnotmatch '^[0-9a-f]{64}$' -or $sidecarSha -cne $actualSha -or
        (-not [string]::IsNullOrWhiteSpace($ExpectedSha256) -and $actualSha -cne $ExpectedSha256)) {
        throw "Evidence sidecar authentication failed: $Path"
    }
    return $actualSha
}

function Resolve-Sprint8BRepositoryPath {
    param([Parameter(Mandatory)][string]$Path)
    $full = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot $Path))
    }
    $null = Get-Sprint8BRepositoryRelativePath -Path $full
    return $full
}

function Get-Sprint8BEnvironmentFingerprintFromDocument {
    param([Parameter(Mandatory)]$Document)
    if ($Document.PSObject.Properties.Name -contains "environment_fingerprint_sha256") {
        return [string]$Document.environment_fingerprint_sha256
    }
    if ($Document.PSObject.Properties.Name -contains "environment_fingerprint") {
        return [string]$Document.environment_fingerprint
    }
    if ($Document.PSObject.Properties.Name -contains "environment" -and $null -ne $Document.environment) {
        if ($Document.environment.PSObject.Properties.Name -contains "fingerprint_sha256") {
            return [string]$Document.environment.fingerprint_sha256
        }
        if ($Document.environment.PSObject.Properties.Name -contains "fingerprint") {
            return [string]$Document.environment.fingerprint
        }
    }
    return ""
}

function Assert-Sprint8BTopologyContext {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$ExpectedProject,
        [AllowNull()][string]$CandidateFingerprint
    )
    if ([string]$Context.compose_project -cne $ExpectedProject -or
        [string]$Context.environment.COMPOSE_PROJECT_NAME -cne $ExpectedProject) {
        throw "Retained topology context does not bind Compose project '$ExpectedProject'."
    }
    if (-not [string]::IsNullOrWhiteSpace($CandidateFingerprint) -and
        [string]$Context.candidate_fingerprint -cne $CandidateFingerprint) {
        throw "Retained topology context is not bound to the frozen candidate."
    }
    $fingerprint = [string]$Context.environment_fingerprint_sha256
    if ($fingerprint -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$Context.environment.fingerprint_sha256 -cne $fingerprint) {
        throw "Retained topology context omitted its secret-free environment fingerprint."
    }
    $ports = @(
        [string]$Context.environment.TESSARA_GATEWAY_PORT,
        [string]$Context.environment.TESSARA_CORE_CONTROL_PORT,
        [string]$Context.environment.TESSARA_SUPERVISOR_PORT
    )
    foreach ($port in $ports) {
        $parsed = 0
        if (-not [int]::TryParse($port, [ref]$parsed) -or $parsed -lt 1 -or $parsed -gt 65535) {
            throw "Retained topology context contains an invalid loopback port."
        }
    }
    if (@($ports | Sort-Object -Unique).Count -ne $ports.Count) {
        throw "Retained topology context reuses a loopback port."
    }
    $fixturePath = Resolve-Sprint8BRepositoryPath -Path ([string]$Context.fixture_receipt.path)
    $fixtureSha = Assert-Sprint8BSha256Sidecar -Path $fixturePath `
        -ExpectedSha256 ([string]$Context.fixture_receipt.sha256)
    $fixture = Read-Sprint8BAuthenticatedJson -Path $fixturePath -ExpectedSha256 $fixtureSha
    if ([string]$fixture.sprint -cne "sprint-8b" -or [string]$fixture.state -cne "passed" -or
        [string]$fixture.proof -cne "owner-controlled-uat-fixture-preparation" -or
        [string]$fixture.compose_project -cne $ExpectedProject -or
        [string]$fixture.restoration.state -cne "passed") {
        throw "Retained topology context has an invalid fixture/restoration receipt."
    }
    return $Context
}

function Get-Sprint8BTopologyContextFromHarness {
    param(
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$ExpectedProject,
        [AllowNull()][string]$CandidateFingerprint
    )
    if ($null -eq $Document.environment -or
        [string]$Document.cleanup_restoration.mode -cne "retained-for-caller") {
        return $null
    }
    $fixturePath = Resolve-Sprint8BRepositoryPath -Path ([string]$Document.fixture_receipt_path)
    $fixtureSha = Assert-Sprint8BSha256Sidecar -Path $fixturePath `
        -ExpectedSha256 ([string]$Document.fixture_receipt_sha256)
    $fingerprint = Get-Sprint8BEnvironmentFingerprintFromDocument -Document $Document
    $context = [pscustomobject][ordered]@{
        compose_project = $ExpectedProject
        candidate_fingerprint = if ([string]::IsNullOrWhiteSpace($CandidateFingerprint)) { $null } else { $CandidateFingerprint }
        environment_fingerprint_sha256 = $fingerprint
        environment = [pscustomobject][ordered]@{
            COMPOSE_PROJECT_NAME = [string]$Document.environment.COMPOSE_PROJECT_NAME
            TESSARA_GATEWAY_PORT = [string]$Document.environment.TESSARA_GATEWAY_PORT
            TESSARA_CORE_CONTROL_PORT = [string]$Document.environment.TESSARA_CORE_CONTROL_PORT
            TESSARA_SUPERVISOR_PORT = [string]$Document.environment.TESSARA_SUPERVISOR_PORT
            fingerprint_sha256 = [string]$Document.environment.fingerprint_sha256
        }
        fixture_receipt = [pscustomobject][ordered]@{
            path = Get-Sprint8BRepositoryRelativePath -Path $fixturePath
            sha256 = $fixtureSha
        }
    }
    return Assert-Sprint8BTopologyContext -Context $context -ExpectedProject $ExpectedProject `
        -CandidateFingerprint $CandidateFingerprint
}

function Set-Sprint8BTopologyEnvironment {
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$ExpectedProject,
        [AllowNull()][string]$CandidateFingerprint
    )
    $null = Assert-Sprint8BTopologyContext -Context $Context -ExpectedProject $ExpectedProject `
        -CandidateFingerprint $CandidateFingerprint
    foreach ($name in @(
        "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
        "TESSARA_SUPERVISOR_PORT"
    )) {
        [Environment]::SetEnvironmentVariable($name, [string]$Context.environment.$name, "Process")
    }
    [Environment]::SetEnvironmentVariable(
        "PLAYWRIGHT_BASE_URL", "http://127.0.0.1:$([string]$Context.environment.TESSARA_GATEWAY_PORT)", "Process"
    )
    [Environment]::SetEnvironmentVariable("TESSARA_PLAYWRIGHT_ACCEPTANCE", "1", "Process")
}

function New-Sprint8BAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [ValidateSet("program", "pwsh", "internal", "teardown", "manual")][string]$Kind,
        [string]$Command,
        [string[]]$Arguments = @(),
        [switch]$ProducesEvidence,
        [switch]$ProvidesEnvironment,
        [switch]$ProvidesRestoration,
        [ValidateSet("", "fresh", "upgraded")][string]$PlaywrightDataState = "",
        [string[]]$Scenarios = @()
    )
    [pscustomobject][ordered]@{
        id = $Id
        kind = $Kind
        command = $Command
        arguments = @($Arguments)
        produces_evidence = [bool]$ProducesEvidence
        provides_environment = [bool]$ProvidesEnvironment
        provides_restoration = [bool]$ProvidesRestoration
        playwright_data_state = $PlaywrightDataState
        scenarios = @($Scenarios)
    }
}

function New-Sprint8BPowerShellAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Script,
        [string[]]$Arguments = @(),
        [switch]$ProducesEvidence,
        [switch]$ProvidesEnvironment,
        [switch]$ProvidesRestoration
    )
    New-Sprint8BAction -Id $Id -Kind pwsh -Command $Script -Arguments $Arguments `
        -ProducesEvidence:$ProducesEvidence -ProvidesEnvironment:$ProvidesEnvironment `
        -ProvidesRestoration:$ProvidesRestoration
}

function New-Sprint8BProgramAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][string[]]$Arguments,
        [ValidateSet("", "fresh", "upgraded")][string]$PlaywrightDataState = ""
    )
    New-Sprint8BAction -Id $Id -Kind program -Command $Program -Arguments $Arguments `
        -PlaywrightDataState $PlaywrightDataState
}

function New-Sprint8BMaterializeAction {
    param(
        [string]$Id = "materialize",
        [ValidateSet("CoreFresh", "DatasetBootstrap", "Reference", "ReferenceNoOp", "All")][string]$Target = "Reference",
        [switch]$KeepTopology
    )
    if ($Target -in @("CoreFresh", "DatasetBootstrap", "All")) {
        if ($KeepTopology) { throw "Focused materialization target '$Target' cannot retain a Compose topology." }
        return New-Sprint8BPowerShellAction -Id $Id -Script "scripts/materialize-sprint-8b.ps1" `
            -Arguments @("-Target", $Target, "-EvidencePath", "{evidence}") `
            -ProducesEvidence -ProvidesRestoration
    }
    $arguments = @(
        "-Target", $Target,
        "-ComposeProject", "{project}",
        "-EvidencePath", "{evidence}",
        "-AuthorizeDisposableReset"
    )
    if ($KeepTopology) { $arguments += "-KeepTopology" }
    New-Sprint8BPowerShellAction -Id $Id -Script "scripts/materialize-sprint-8b.ps1" `
        -Arguments $arguments -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration:(-not $KeepTopology)
}

function New-Sprint8BSmokeAction {
    param([string]$Id = "deployed-smoke", [switch]$UseExistingTopology)
    $arguments = @(
        "-ComposeProject", "{project}",
        "-EvidencePath", "{evidence}"
    )
    if ($UseExistingTopology) {
        $arguments += @("-UseExistingTopology", "-FixtureReceiptPath", "{fixture}")
    } else {
        $arguments += "-AuthorizeDisposableReset"
    }
    New-Sprint8BPowerShellAction -Id $Id -Script "scripts/run-sprint-8b-deployed-smoke.ps1" `
        -Arguments $arguments -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function New-Sprint8BUatAction {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Scenario,
        [switch]$UseExistingTopology
    )
    $arguments = @(
        "-Scenario", $Scenario,
        "-ComposeProject", "{project}",
        "-EvidencePath", "{evidence}"
    )
    if ($UseExistingTopology) {
        $arguments += @("-UseExistingTopology", "-FixtureReceiptPath", "{fixture}")
    } else {
        $arguments += "-AuthorizeDisposableReset"
    }
    New-Sprint8BPowerShellAction -Id $Id -Script "scripts/uat-sprint-8b.ps1" `
        -Arguments $arguments -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function New-Sprint8BFailureAction {
    param([string]$Id = "failure-containment")
    New-Sprint8BPowerShellAction -Id $Id -Script "scripts/run-sprint-8b-failure-containment.ps1" `
        -Arguments @("-ComposeProject", "{project}", "-EvidencePath", "{evidence}", "-AuthorizeDisposableReset") `
        -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function New-Sprint8BUpgradeAction {
    param([string]$Id = "dataset-upgrade")
    New-Sprint8BPowerShellAction -Id $Id -Script "scripts/run-sprint-8b-dataset-upgrade.ps1" `
        -Arguments @("-ComposeProject", "{project}", "-EvidencePath", "{evidence}", "-AuthorizeDisposableReset") `
        -ProducesEvidence -ProvidesEnvironment -ProvidesRestoration
}

function Get-Sprint8BStaticActions {
    @(
        New-Sprint8BProgramAction -Id "cargo-fmt" -Program "cargo" -Arguments @("fmt", "--all", "--", "--check")
        New-Sprint8BProgramAction -Id "cargo-check" -Program "cargo" -Arguments @("check", "--workspace", "--all-targets", "--all-features", "--locked", "--offline", "--jobs", "1")
        New-Sprint8BProgramAction -Id "cargo-clippy" -Program "cargo" -Arguments @("clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--offline", "--jobs", "1", "--", "-D", "warnings")
        New-Sprint8BPowerShellAction -Id "web-boundaries" -Script "scripts/check-web-crate-boundaries.ps1"
        New-Sprint8BPowerShellAction -Id "sdk-boundaries" -Script "scripts/verify-module-sdk-boundaries.ps1"
        New-Sprint8BPowerShellAction -Id "ui-conformance" -Script "scripts/ui-sdk-conformance.ps1"
        New-Sprint8BPowerShellAction -Id "asset-identity" -Script "scripts/build-module-ui-browser-assets.ps1" -Arguments @("-Module", "all", "-Check")
    )
}

function Get-Sprint8BFormalActionMap {
    $map = [ordered]@{}

    $map["readiness-contract"] = @(
        New-Sprint8BAction -Id "implementation-gate" -Kind internal -Command "implementation-gate"
        New-Sprint8BPowerShellAction -Id "planning-alignment" -Script "scripts/assert-sprint-8b-planning-contract.ps1"
        New-Sprint8BPowerShellAction -Id "policy-selftest" -Script "scripts/test-tessara-validation-policy.ps1" -Arguments @("-SelfTest")
        New-Sprint8BPowerShellAction -Id "implementation-runner-selftest" -Script "scripts/run-sprint-8b-implementation-readiness.ps1" -Arguments @("-SelfTest")
        New-Sprint8BPowerShellAction -Id "acceptance-contract-selftest" -Script "scripts/sprint-8b-acceptance-contract.ps1" -Arguments @("-SelfTest")
    )
    $map["readiness-materialization"] = @(
        New-Sprint8BMaterializeAction -Id "focused-owner-materialization" -Target All
        New-Sprint8BMaterializeAction -Id "reference-materialization" -Target ReferenceNoOp
    )
    $map["readiness-acceptance"] = @(
        New-Sprint8BPowerShellAction -Id "acceptance-contract" -Script "scripts/sprint-8b-acceptance-contract.ps1" -Arguments @("-EvidencePath", "{evidence}") -ProducesEvidence
        New-Sprint8BPowerShellAction -Id "uat-runner-selftest" -Script "scripts/uat-sprint-8b.ps1" -Arguments @("-SelfTest")
        New-Sprint8BPowerShellAction -Id "browser-inventory" -Script "scripts/validate-e2e.ps1" -Arguments @("-InventoryOnly", "-EvidencePath", "{evidence}") -ProducesEvidence
    )

    $map["rehearsal-static"] = @(Get-Sprint8BStaticActions) + @(
        New-Sprint8BPowerShellAction -Id "planning-alignment" -Script "scripts/assert-sprint-8b-planning-contract.ps1"
    )
    $map["rehearsal-rust"] = @(
        New-Sprint8BMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8BProgramAction -Id "workspace-rust" -Program "cargo" -Arguments @("test", "--workspace", "--all-features", "--locked", "--offline", "--jobs", "1")
        New-Sprint8BSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8BAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-materialization"] = @(
        New-Sprint8BMaterializeAction -Id "focused-owner-materialization" -Target All
        New-Sprint8BMaterializeAction -Id "reference-materialization" -Target ReferenceNoOp
    )
    $map["rehearsal-browser"] = @(
        New-Sprint8BMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8BProgramAction -Id "browser-acceptance" -Program "npm" `
            -Arguments @("--prefix", ".\end2end", "test") -PlaywrightDataState fresh
        New-Sprint8BSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8BAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-conformance"] = @(
        New-Sprint8BMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8BPowerShellAction -Id "dataset-boundary" -Script "scripts/check-sprint-8b-dataset-boundaries.ps1" -Arguments @("-Mode", "RequireClean")
        New-Sprint8BPowerShellAction -Id "ui-conformance" -Script "scripts/ui-sdk-conformance.ps1"
        New-Sprint8BSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8BAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-source-sync"] = @(
        New-Sprint8BMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8BPowerShellAction -Id "sync-integration" -Script "scripts/test-sprint-8b-dataset-module.ps1" -Arguments @("-Suite", "Sync")
        New-Sprint8BPowerShellAction -Id "dag-integration" -Script "scripts/test-sprint-8b-dataset-module.ps1" -Arguments @("-Suite", "Dag")
        New-Sprint8BSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8BAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-reverse-consumers"] = @(
        New-Sprint8BMaterializeAction -Id "setup" -Target Reference -KeepTopology
        New-Sprint8BPowerShellAction -Id "provider-integration" -Script "scripts/test-sprint-8b-dataset-module.ps1" -Arguments @("-Suite", "Provider")
        New-Sprint8BPowerShellAction -Id "component-consumer" -Script "scripts/test-sprint-8b-component-consumer.ps1"
        New-Sprint8BSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
        New-Sprint8BAction -Id "teardown" -Kind teardown -Command "compose-down"
    )
    $map["rehearsal-smoke"] = @(New-Sprint8BSmokeAction -Id "rehearsal-smoke")
    $map["rehearsal-recovery"] = @(New-Sprint8BFailureAction -Id "rehearsal-recovery")
    $map["rehearsal-upgrade"] = @(New-Sprint8BUpgradeAction -Id "rehearsal-upgrade")
    $map["rehearsal-uat"] = @(New-Sprint8BUatAction -Id "rehearsal-uat" -Scenario All)

    $map["preflight-freeze"] = @(New-Sprint8BAction -Id "freeze-candidate" -Kind internal -Command "freeze-candidate")

    $map["sit-static"] = @(Get-Sprint8BStaticActions) + @(
        New-Sprint8BPowerShellAction -Id "dataset-boundary" -Script "scripts/check-sprint-8b-dataset-boundaries.ps1" -Arguments @("-Mode", "RequireClean")
    )
    $map["sit-rust"] = @(
        New-Sprint8BMaterializeAction -Id "frozen-sit-setup" -Target Reference -KeepTopology
        New-Sprint8BProgramAction -Id "workspace-rust" -Program "cargo" -Arguments @("test", "--workspace", "--all-features", "--locked", "--offline", "--jobs", "1")
        New-Sprint8BSmokeAction -Id "restoration-checkpoint" -UseExistingTopology
    )
    $map["sit-browser"] = @(
        New-Sprint8BProgramAction -Id "browser-acceptance" -Program "npm" `
            -Arguments @("--prefix", ".\end2end", "test") -PlaywrightDataState fresh
        New-Sprint8BSmokeAction -Id "browser-restoration" -UseExistingTopology
    )
    $map["sit-smoke"] = @(
        New-Sprint8BSmokeAction -Id "sit-smoke" -UseExistingTopology
        New-Sprint8BAction -Id "teardown" -Kind teardown -Command "compose-down"
    )

    $map["uat-scripted"] = @(New-Sprint8BUatAction -Id "uat-scripted" -Scenario All)
    $map["uat-product"] = @(
        New-Sprint8BUatAction -Id "uat-8b-01" -Scenario "UAT-8B-01"
        New-Sprint8BUatAction -Id "uat-8b-09" -Scenario "UAT-8B-09"
        New-Sprint8BAction -Id "manual-product" -Kind manual -Scenarios @("UAT-8B-01", "UAT-8B-09")
    )
    $map["uat-materialization"] = @(
        New-Sprint8BMaterializeAction -Id "uat-materialization-proof" -Target ReferenceNoOp
        New-Sprint8BUatAction -Id "uat-8b-02" -Scenario "UAT-8B-02"
        New-Sprint8BAction -Id "manual-materialization" -Kind manual -Scenarios @("UAT-8B-02")
    )
    $map["uat-operations"] = @(
        New-Sprint8BUatAction -Id "uat-8b-03" -Scenario "UAT-8B-03"
        New-Sprint8BUatAction -Id "uat-8b-10" -Scenario "UAT-8B-10"
        New-Sprint8BAction -Id "manual-operations" -Kind manual -Scenarios @("UAT-8B-03", "UAT-8B-10")
    )
    $map["uat-providers"] = @(
        New-Sprint8BUatAction -Id "uat-8b-04" -Scenario "UAT-8B-04"
        New-Sprint8BUatAction -Id "uat-8b-09" -Scenario "UAT-8B-09"
        New-Sprint8BAction -Id "manual-providers" -Kind manual -Scenarios @("UAT-8B-04", "UAT-8B-09")
    )
    $map["uat-reverse-consumers"] = @(
        New-Sprint8BUatAction -Id "uat-8b-10" -Scenario "UAT-8B-10"
        New-Sprint8BAction -Id "manual-reverse-consumers" -Kind manual -Scenarios @("UAT-8B-10")
    )
    $map["uat-resource-resolution"] = @(
        New-Sprint8BUatAction -Id "uat-8b-11" -Scenario "UAT-8B-11"
        New-Sprint8BAction -Id "manual-resource-resolution" -Kind manual -Scenarios @("UAT-8B-11")
    )
    $map["uat-replay-refresh"] = @(
        New-Sprint8BUatAction -Id "uat-8b-04" -Scenario "UAT-8B-04"
        New-Sprint8BUatAction -Id "uat-8b-11" -Scenario "UAT-8B-11"
        New-Sprint8BAction -Id "manual-replay-refresh" -Kind manual -Scenarios @("UAT-8B-04", "UAT-8B-11")
    )
    $map["uat-crossmodule"] = @(
        New-Sprint8BUatAction -Id "uat-8b-05" -Scenario "UAT-8B-05"
        New-Sprint8BAction -Id "manual-crossmodule" -Kind manual -Scenarios @("UAT-8B-05")
    )
    $map["uat-subtraction"] = @(
        New-Sprint8BUatAction -Id "uat-8b-06" -Scenario "UAT-8B-06"
        New-Sprint8BAction -Id "manual-subtraction" -Kind manual -Scenarios @("UAT-8B-06")
    )
    $map["uat-recovery"] = @(
        New-Sprint8BFailureAction -Id "uat-recovery-proof"
        New-Sprint8BUatAction -Id "uat-8b-07" -Scenario "UAT-8B-07"
        New-Sprint8BAction -Id "manual-recovery" -Kind manual -Scenarios @("UAT-8B-07")
    )
    $map["uat-upgrade"] = @(
        New-Sprint8BUpgradeAction -Id "uat-upgrade-proof"
        New-Sprint8BUatAction -Id "uat-8b-08" -Scenario "UAT-8B-08"
        New-Sprint8BAction -Id "manual-upgrade" -Kind manual -Scenarios @("UAT-8B-08")
    )

    return $map
}

function Get-Sprint8BExpectedLaneIds {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { @("readiness-contract", "readiness-materialization", "readiness-acceptance") }
        "candidate-rehearsal" { @("rehearsal-static", "rehearsal-rust", "rehearsal-materialization", "rehearsal-browser", "rehearsal-conformance", "rehearsal-source-sync", "rehearsal-reverse-consumers", "rehearsal-smoke", "rehearsal-recovery", "rehearsal-upgrade", "rehearsal-uat") }
        "validation-preflight" { @("preflight-freeze") }
        "sit" { @("sit-static", "sit-rust", "sit-browser", "sit-smoke") }
        "uat" { @("uat-scripted", "uat-product", "uat-materialization", "uat-operations", "uat-providers", "uat-reverse-consumers", "uat-resource-resolution", "uat-replay-refresh", "uat-crossmodule", "uat-subtraction", "uat-recovery", "uat-upgrade") }
        default { throw "Unknown Sprint 8B phase '$Phase'." }
    }
}

function Get-Sprint8BExpectedLiveLaneIds {
    param([Parameter(Mandatory)][string]$Phase)
    switch ($Phase) {
        "validation-readiness" { @("readiness-materialization") }
        "candidate-rehearsal" { @(
            "rehearsal-rust", "rehearsal-materialization", "rehearsal-browser",
            "rehearsal-conformance", "rehearsal-source-sync",
            "rehearsal-reverse-consumers", "rehearsal-smoke", "rehearsal-recovery",
            "rehearsal-upgrade", "rehearsal-uat"
        ) }
        "validation-preflight" { @() }
        "sit" { @("sit-rust", "sit-browser", "sit-smoke") }
        "uat" { @(Get-Sprint8BExpectedLaneIds -Phase "uat") }
        default { throw "Unknown Sprint 8B phase '$Phase'." }
    }
}

function Get-Sprint8BExpectedProject {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$LaneContract
    )
    if (-not [bool]$LaneContract.touches_live_state) { return $null }
    if ($Phase -ceq "sit") { return "tessara-s8b-sit" }
    return "tessara-s8b-$([string]$LaneContract.id)"
}

function Get-Sprint8BSelector {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $runner = Get-Sprint8BPhaseRunnerName -Phase $Phase
    if ($Phase -ceq "validation-preflight") {
        return "pwsh -NoProfile -File .\scripts\$runner"
    }
    return "pwsh -NoProfile -File .\scripts\$runner -Lane $Lane"
}

function Assert-Sprint8BFormalProfile {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$ActionMap
    )
    if ([string]$Contract.evidence_policy.root -cne "artifacts/sprint-8b-closeout") {
        throw "Sprint 8B formal evidence root is not canonical."
    }
    $expectedIds = @(Get-Sprint8BExpectedLaneIds -Phase $Phase)
    $expectedLiveIds = @(Get-Sprint8BExpectedLiveLaneIds -Phase $Phase)
    $phaseLanes = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase })
    Assert-Sprint8BExactSequence -Expected $expectedIds -Actual @($phaseLanes.id) -Label "$Phase lane order"

    $seenActions = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    for ($index = 0; $index -lt $phaseLanes.Count; $index++) {
        $lane = $phaseLanes[$index]
        $id = [string]$lane.id
        $expectedPrerequisite = @(if ($index -eq 0) {
            switch ($Phase) {
                "validation-readiness" { @() }
                "candidate-rehearsal" { @("readiness-acceptance") }
                "validation-preflight" { @("rehearsal-uat") }
                "sit" { @("preflight-freeze") }
                "uat" { @("sit-smoke") }
            }
        } else { [string]$phaseLanes[$index - 1].id })
        Assert-Sprint8BExactSequence -Expected $expectedPrerequisite -Actual @($lane.prerequisites) -Label "$id prerequisites"

        $project = Get-Sprint8BExpectedProject -Phase $Phase -LaneContract $lane
        $expectedLive = $expectedLiveIds -ccontains $id
        if ([bool]$lane.touches_live_state -ne $expectedLive) {
            throw "Lane '$id' live-state identity does not match the approved Sprint 8B profile."
        }
        if ([bool]$lane.touches_live_state) {
            if ($project -cnotmatch '^tessara-s8b-[a-z0-9-]+$') {
                throw "Lane '$id' has an unsafe Compose project identity '$project'."
            }
        } elseif ($null -ne $project) {
            throw "Offline lane '$id' unexpectedly has a Compose project."
        }

        if (-not $ActionMap.Contains($id)) { throw "Formal profile omitted action mapping for '$id'." }
        $actions = @($ActionMap[$id])
        if ($actions.Count -eq 0) { throw "Formal lane '$id' has no assertion actions." }
        $localActions = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $environmentProviders = 0
        $restorationProviders = 0
        foreach ($action in $actions) {
            if (-not $localActions.Add([string]$action.id)) {
                throw "Formal lane '$id' contains duplicate action '$($action.id)'."
            }
            $null = $seenActions.Add("$id/$([string]$action.id)")
            if ([string]$action.kind -ceq "manual" -and $Phase -cne "uat") {
                throw "Only formal UAT may consume manual scenario evidence."
            }
            if ([bool]$action.produces_evidence -and @($action.arguments) -cnotcontains "{evidence}") {
                throw "Evidence-producing action '$id/$($action.id)' omits the attempt-scoped evidence path."
            }
            if ([bool]$action.provides_environment) {
                $environmentProviders++
                if (@($action.arguments) -cnotcontains "{project}") {
                    throw "Environment action '$id/$($action.id)' omits the exact Compose project binding."
                }
            }
            if ([bool]$action.provides_restoration -or [string]$action.kind -ceq "teardown") {
                $restorationProviders++
            }
            if (@($action.arguments) -ccontains "-UseExistingTopology" -and
                @($action.arguments) -cnotcontains "{fixture}") {
                throw "Existing-topology action '$id/$($action.id)' omits its authenticated fixture receipt."
            }
            $isPlaywrightAcceptance = [string]$action.kind -ceq "program" -and
                [string]$action.command -ceq "npm" -and
                (@($action.arguments) -join "`n") -ceq
                    (@("--prefix", ".\end2end", "test") -join "`n")
            if ($isPlaywrightAcceptance -and
                [string]$action.playwright_data_state -notin @("fresh", "upgraded")) {
                throw "Playwright action '$id/$($action.id)' omits its exact data-state identity."
            }
            if (-not $isPlaywrightAcceptance -and
                -not [string]::IsNullOrWhiteSpace([string]$action.playwright_data_state)) {
                throw "Non-Playwright action '$id/$($action.id)' declares a Playwright data state."
            }
            if ($isPlaywrightAcceptance -and $id -in @("rehearsal-browser", "sit-browser") -and
                [string]$action.playwright_data_state -cne "fresh") {
                throw "Reference browser lane '$id' must bind Playwright to the fresh data state."
            }
        }
        if ([bool]$lane.touches_live_state -and ($environmentProviders -eq 0 -or $restorationProviders -eq 0)) {
            throw "Live lane '$id' lacks an environment or restoration evidence provider."
        }
        if (-not [bool]$lane.touches_live_state -and $environmentProviders -ne 0) {
            throw "Offline lane '$id' maps a live environment provider."
        }

        $paths = Get-Sprint8BLaneEvidencePaths -Contract $Contract -Phase $Phase -Lane $id
        $expectedSuffix = ([string]$Contract.evidence_policy.root).TrimEnd("/", "\") + "/$Phase/lanes/$id"
        $actualSuffix = (Get-Sprint8BRepositoryRelativePath -Path $paths.lane_root)
        if ($actualSuffix -cne $expectedSuffix.Replace("\", "/")) {
            throw "Lane '$id' evidence mapping is not canonical."
        }
        if ((Split-Path -Leaf $paths.result) -cne "result.json" -or
            (Split-Path -Leaf $paths.command_log) -cne "command.log" -or
            (Split-Path -Leaf $paths.evidence_references) -cne "evidence-references.json") {
            throw "Lane '$id' evidence filenames are not canonical."
        }
        $selector = Get-Sprint8BSelector -Phase $Phase -Lane $id
        if (-not $selector.Contains($id) -and $Phase -cne "validation-preflight") {
            throw "Lane '$id' selector does not bind its exact identity."
        }
    }

    if ($Phase -ceq "uat") {
        $scenarioContract = Get-Sprint8BScenarioContract
        $mapped = @($phaseLanes | ForEach-Object {
            @($ActionMap[[string]$_.id] | Where-Object { [string]$_.kind -ceq "manual" } | ForEach-Object { @($_.scenarios) })
        } | ForEach-Object { [string]$_ } | Sort-Object -Unique)
        Assert-Sprint8BExactSequence -Expected @($scenarioContract.scenarios.id | Sort-Object) -Actual $mapped -Label "formal UAT manual scenario coverage"
    }

    return $true
}

function Get-Sprint8BLaneContract {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $matches = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase -and [string]$_.id -ceq $Lane })
    if ($matches.Count -ne 1) { throw "Unknown $Phase lane '$Lane'." }
    return $matches[0]
}

function Get-Sprint8BLaneResultPath {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$LaneId
    )
    $matches = @($Contract.lanes | Where-Object { [string]$_.id -ceq $LaneId })
    if ($matches.Count -ne 1) { throw "Prerequisite lane '$LaneId' is not unique in the contract." }
    return (Get-Sprint8BLaneEvidencePaths -Contract $Contract -Phase ([string]$matches[0].phase) -Lane $LaneId).result
}

function Read-Sprint8BAuthenticatedJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedSha256,
        [switch]$RequireSidecar
    )
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required evidence is missing: $Path"
    }
    if ($RequireSidecar) {
        $null = Assert-Sprint8BSha256Sidecar -Path $Path -ExpectedSha256 $ExpectedSha256
    } elseif (-not [string]::IsNullOrWhiteSpace($ExpectedSha256)) {
        $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -cne $ExpectedSha256) { throw "Evidence authentication failed: $Path" }
    }
    try { return Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json -Depth 100 } catch {
        throw "Evidence is not valid JSON: $Path. $($_.Exception.Message)"
    }
}

function Assert-Sprint8BSourceMatches {
    param(
        [Parameter(Mandatory)]$Expected,
        [Parameter(Mandatory)]$Actual,
        [Parameter(Mandatory)][string]$Label
    )
    if ([string]$Expected.commit -cne [string]$Actual.commit -or
        [string]$Expected.tree -cne [string]$Actual.tree -or
        [bool]$Expected.dirty -or [bool]$Actual.dirty) {
        throw "$Label is not bound to the current clean source."
    }
}

function Get-Sprint8BImplementationResultPath {
    param([Parameter(Mandatory)]$Contract)
    $root = [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot ([string]$Contract.evidence_policy.root)))
    return Join-Path $root "implementation/implementation-readiness-result.json"
}

function Get-Sprint8BImpactPath {
    param([Parameter(Mandatory)]$Contract)
    $root = [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot ([string]$Contract.evidence_policy.root)))
    return Join-Path $root "validation-impact.json"
}

function Assert-Sprint8BImplementationGate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Source
    )
    $path = Get-Sprint8BImplementationResultPath -Contract $Contract
    $result = Read-Sprint8BAuthenticatedJson -Path $path -RequireSidecar
    $null = Assert-TessaraImplementationReadinessResult -Result $result -Contract $Contract -ContractPath $script:Sprint8BContractPath
    Assert-Sprint8BSourceMatches -Expected $Source -Actual $result.source_identity -Label "Implementation readiness"

    $impactPath = Get-Sprint8BImpactPath -Contract $Contract
    $impact = Read-Sprint8BAuthenticatedJson -Path $impactPath -RequireSidecar
    $null = Assert-TessaraCorrectionImpactAssessment -Assessment $impact -Contract $Contract
    Assert-Sprint8BSourceMatches -Expected $Source -Actual $impact.current_source -Label "Validation impact assessment"
    $decision = @($impact.phase_decisions | Where-Object { [string]$_.phase -ceq "validation-readiness" })
    if ($decision.Count -ne 1 -or [string]$decision[0].action -eq "reuse_certificate") {
        throw "Validation impact does not authorize a current Readiness execution plan."
    }
    return @(
        Get-Sprint8BFileReference -Path $path
        Get-Sprint8BFileReference -Path $impactPath
    )
}

function Get-Sprint8BCertificatePath {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )
    return Join-Path (Get-Sprint8BPhaseRoot -Contract $Contract -Phase $Phase) (Get-Sprint8BPhaseResultName -Phase $Phase)
}

function Get-Sprint8BCandidatePath {
    param([Parameter(Mandatory)]$Contract)
    $root = [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot ([string]$Contract.evidence_policy.root)))
    return Join-Path $root "candidate.json"
}

function Read-Sprint8BCandidate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Source
    )
    $path = Get-Sprint8BCandidatePath -Contract $Contract
    $candidate = Read-Sprint8BAuthenticatedJson -Path $path -RequireSidecar
    if ([int]$candidate.schema_version -ne 1 -or
        [string]$candidate.contract -cne "tessara.validation.candidate" -or
        [string]$candidate.sprint -cne "sprint-8b" -or
        [string]$candidate.candidate_fingerprint -cnotmatch '^[0-9a-f]{64}$') {
        throw "Frozen Sprint 8B candidate has the wrong identity."
    }
    Assert-Sprint8BSourceMatches -Expected $Source -Actual $candidate.source_identity -Label "Frozen candidate"
    return $candidate
}

function Assert-Sprint8BPhaseCertificateForContract {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$Source,
        [string]$CandidateFingerprint
    )
    $path = Get-Sprint8BCertificatePath -Contract $Contract -Phase $Phase
    $certificate = Read-Sprint8BAuthenticatedJson -Path $path -RequireSidecar
    $null = Assert-TessaraPhaseCertificate -Certificate $certificate
    if ([string]$certificate.sprint -cne "sprint-8b" -or [string]$certificate.phase -cne $Phase) {
        throw "Certificate '$path' has the wrong Sprint 8B phase identity."
    }
    Assert-Sprint8BSourceMatches -Expected $Source -Actual $certificate.source_identity -Label "$Phase certificate"
    $expectedLanes = @(Get-Sprint8BExpectedLaneIds -Phase $Phase)
    Assert-Sprint8BExactSequence -Expected $expectedLanes -Actual @($certificate.lanes.name) -Label "$Phase certificate lane coverage"
    if (-not [string]::IsNullOrWhiteSpace($CandidateFingerprint) -and
        [string]$certificate.candidate_fingerprint -cne $CandidateFingerprint) {
        throw "$Phase certificate is not bound to the frozen candidate."
    }
    $indexPath = Join-Path $script:Sprint8BRepositoryRoot ([string]$certificate.evidence_index.path)
    $index = Read-Sprint8BAuthenticatedJson -Path $indexPath `
        -ExpectedSha256 ([string]$certificate.evidence_index.sha256) -RequireSidecar
    $null = Assert-TessaraPhaseEvidenceIndex -Index $index -RepositoryRoot $script:Sprint8BRepositoryRoot -AuditFiles
    return [pscustomobject]@{
        certificate = $certificate
        reference = Get-Sprint8BFileReference -Path $path
    }
}

function Assert-Sprint8BLanePrerequisites {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$LaneContract,
        [Parameter(Mandatory)]$Source
    )
    $references = [Collections.Generic.List[object]]::new()
    $phase = [string]$LaneContract.phase
    $candidateFingerprint = $null
    $topologyContext = $null

    if ([string]$LaneContract.id -ceq "readiness-contract") {
        foreach ($reference in @(Assert-Sprint8BImplementationGate -Contract $Contract -Source $Source)) {
            $references.Add($reference)
        }
    }

    foreach ($prerequisite in @($LaneContract.prerequisites)) {
        $path = Get-Sprint8BLaneResultPath -Contract $Contract -LaneId ([string]$prerequisite)
        $result = Read-Sprint8BAuthenticatedJson -Path $path -RequireSidecar
        if ([string]$result.contract -cne "tessara.validation.lane-result" -or
            [string]$result.lane -cne [string]$prerequisite -or
            [string]$result.state -cne "passed") {
            throw "Prerequisite lane '$prerequisite' is not a passing authenticated result."
        }
        Assert-Sprint8BSourceMatches -Expected $Source -Actual $result.source_identity -Label "Prerequisite lane '$prerequisite'"
        $references.Add((Get-Sprint8BFileReference -Path $path))
        if ($phase -ceq "sit" -and $result.PSObject.Properties.Name -contains "topology_context" -and
            $null -ne $result.topology_context) {
            $topologyContext = $result.topology_context
        }
    }

    if ([string]$LaneContract.id -ceq (Get-Sprint8BExpectedLaneIds -Phase $phase)[0]) {
        switch ($phase) {
            "candidate-rehearsal" {
                $prior = Assert-Sprint8BPhaseCertificateForContract -Contract $Contract -Phase "validation-readiness" -Source $Source
                $references.Add($prior.reference)
            }
            "validation-preflight" {
                foreach ($priorPhase in @("validation-readiness", "candidate-rehearsal")) {
                    $prior = Assert-Sprint8BPhaseCertificateForContract -Contract $Contract -Phase $priorPhase -Source $Source
                    $references.Add($prior.reference)
                }
            }
            "sit" {
                $candidate = Read-Sprint8BCandidate -Contract $Contract -Source $Source
                $candidateFingerprint = [string]$candidate.candidate_fingerprint
                $prior = Assert-Sprint8BPhaseCertificateForContract -Contract $Contract -Phase "validation-preflight" -Source $Source -CandidateFingerprint $candidateFingerprint
                $references.Add($prior.reference)
                $references.Add((Get-Sprint8BFileReference -Path (Get-Sprint8BCandidatePath -Contract $Contract)))
            }
            "uat" {
                $candidate = Read-Sprint8BCandidate -Contract $Contract -Source $Source
                $candidateFingerprint = [string]$candidate.candidate_fingerprint
                $prior = Assert-Sprint8BPhaseCertificateForContract -Contract $Contract -Phase "sit" -Source $Source -CandidateFingerprint $candidateFingerprint
                $references.Add($prior.reference)
                $references.Add((Get-Sprint8BFileReference -Path (Get-Sprint8BCandidatePath -Contract $Contract)))
            }
        }
    }

    if ($phase -in @("sit", "uat") -and [string]::IsNullOrWhiteSpace($candidateFingerprint)) {
        $candidate = Read-Sprint8BCandidate -Contract $Contract -Source $Source
        $candidateFingerprint = [string]$candidate.candidate_fingerprint
    }

    if ($phase -ceq "sit" -and [string]$LaneContract.id -in @("sit-browser", "sit-smoke")) {
        if ($null -eq $topologyContext) {
            throw "SIT lane '$($LaneContract.id)' requires the authenticated retained tessara-s8b-sit topology context."
        }
        $topologyContext = Assert-Sprint8BTopologyContext -Context $topologyContext `
            -ExpectedProject "tessara-s8b-sit" -CandidateFingerprint $candidateFingerprint
    }

    return [pscustomobject][ordered]@{
        references = @($references)
        candidate_fingerprint = $candidateFingerprint
        topology_context = $topologyContext
    }
}

function Assert-Sprint8BActionAvailability {
    param([Parameter(Mandatory)][object[]]$Actions)
    foreach ($action in $Actions) {
        switch ([string]$action.kind) {
            "pwsh" {
                $path = Join-Path $script:Sprint8BRepositoryRoot ([string]$action.command)
                if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                    throw "Required Sprint 8B lane harness is missing: $($action.command)"
                }
            }
            "program" {
                if ($null -eq (Get-Command ([string]$action.command) -ErrorAction SilentlyContinue)) {
                    throw "Required Sprint 8B lane program is unavailable: $($action.command)"
                }
            }
            "teardown" {
                if ($null -eq (Get-Command "docker" -ErrorAction SilentlyContinue)) {
                    throw "Docker is required for exact Sprint 8B lane teardown."
                }
            }
            "internal" { }
            "manual" { }
            default { throw "Lane action '$($action.id)' has unknown kind '$($action.kind)'." }
        }
    }
}

function Expand-Sprint8BActionArguments {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Arguments,
        [AllowNull()][string]$Project,
        [Parameter(Mandatory)][string]$EvidencePath,
        [AllowNull()][string]$CandidateFingerprint,
        [AllowNull()]$TopologyContext
    )
    $fixturePath = if ($null -eq $TopologyContext) { "" } else { [string]$TopologyContext.fixture_receipt.path }
    @($Arguments | ForEach-Object {
        ([string]$_).Replace("{project}", [string]$Project).
            Replace("{evidence}", $EvidencePath).
            Replace("{repo}", $script:Sprint8BRepositoryRoot).
            Replace("{candidate}", [string]$CandidateFingerprint).
            Replace("{fixture}", $fixturePath)
    })
}

function Assert-Sprint8BHarnessEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Action,
        [AllowNull()][string]$ExpectedProject
    )
    $document = Read-Sprint8BAuthenticatedJson -Path $Path `
        -RequireSidecar:([bool]$Action.provides_environment -or [bool]$Action.provides_restoration)
    if ([bool]$Action.provides_environment -or [bool]$Action.provides_restoration) {
        if ([string]$document.sprint -cne "sprint-8b" -or [string]$document.state -cne "passed") {
            throw "Harness '$($action.id)' did not publish a passing Sprint 8B result."
        }
    }
    if ([bool]$Action.provides_environment) {
        if ([string]$document.compose_project -cne $ExpectedProject) {
            throw "Harness '$($action.id)' used Compose project '$($document.compose_project)' instead of '$ExpectedProject'."
        }
        $fingerprint = Get-Sprint8BEnvironmentFingerprintFromDocument -Document $document
        if ($fingerprint -cnotmatch '^[0-9a-f]{64}$') {
            throw "Harness '$($action.id)' omitted its secret-free environment fingerprint."
        }
    }
    if ([bool]$Action.provides_restoration) {
        if (-not ($document.PSObject.Properties.Name -contains "cleanup_restoration") -or
            [string]$document.cleanup_restoration.state -cne "passed") {
            throw "Harness '$($action.id)' did not prove cleanup/canonical restoration."
        }
    }
    return $document
}

function Invoke-Sprint8BComposeTeardown {
    param(
        [Parameter(Mandatory)][string]$Project,
        [Parameter(Mandatory)][string]$EvidencePath
    )
    if ($Project -cnotmatch '^tessara-s8b-[a-z0-9-]+$') {
        throw "Refusing to tear down unsafe Compose project '$Project'."
    }
    $composePath = Join-Path $script:Sprint8BRepositoryRoot "deploy/sprint-8b/compose.yaml"
    if (-not (Test-Path -LiteralPath $composePath -PathType Leaf)) {
        throw "Sprint 8B Compose profile is missing."
    }
    & docker compose -f $composePath -p $Project --profile reference down --volumes --remove-orphans
    if ($LASTEXITCODE -ne 0) { throw "Exact Compose teardown failed for '$Project'." }
    $remainingContainers = @(& docker ps -a --filter "label=com.docker.compose.project=$Project" --format "{{.ID}}")
    if ($LASTEXITCODE -ne 0 -or @($remainingContainers | Where-Object { $_ }).Count -ne 0) {
        throw "Compose teardown left containers for '$Project'."
    }
    $environmentFingerprint = Get-Sprint8BSha256Text -Text "sprint-8b`n$Project`nreference`n"
    $receipt = [ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8b.compose-teardown"
        sprint = "sprint-8b"
        state = "passed"
        compose_project = $Project
        environment_fingerprint = $environmentFingerprint
        cleanup_restoration = [ordered]@{ state = "passed"; mode = "exact_project_removed" }
    }
    Write-Sprint8BNewJsonFile -Path $EvidencePath -Document $receipt
    return $receipt
}

function Get-Sprint8BManualScenarioPath {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Scenario
    )
    $root = [IO.Path]::GetFullPath((Join-Path $script:Sprint8BRepositoryRoot ([string]$Contract.evidence_policy.root)))
    return Join-Path $root "uat/scenarios/$Scenario/result.json"
}

function Assert-Sprint8BManualScenarioEvidence {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string[]]$Scenarios,
        [Parameter(Mandatory)][string]$CandidateFingerprint
    )
    $scenarioContract = Get-Sprint8BScenarioContract
    $references = [Collections.Generic.List[object]]::new()
    foreach ($scenario in $Scenarios) {
        $expected = @($scenarioContract.scenarios | Where-Object { [string]$_.id -ceq $scenario })
        if ($expected.Count -ne 1) { throw "Unknown manual UAT scenario '$scenario'." }
        $path = Get-Sprint8BManualScenarioPath -Contract $Contract -Scenario $scenario
        $result = Read-Sprint8BAuthenticatedJson -Path $path -RequireSidecar
        if ([string]$result.scenario_id -cne $scenario -or
            [string]$result.state -cne "passed" -or
            -not [bool]$result.manual_acceptance_claimed -or
            [string]$result.candidate_fingerprint -cne $CandidateFingerprint) {
            throw "Manual evidence for '$scenario' is missing a passing, candidate-bound human acceptance claim."
        }
        Assert-Sprint8BExactSequence -Expected @($expected[0].assertions) -Actual @($result.assertions) -Label "$scenario manual assertions"
        if (-not ($result.PSObject.Properties.Name -contains "evidence_references") -or
            @($result.evidence_references).Count -eq 0) {
            throw "Manual evidence for '$scenario' has no retained evidence references."
        }
        $references.Add((Get-Sprint8BFileReference -Path $path))
    }
    return @($references)
}

function Get-Sprint8BTrackedInventory {
    param([Parameter(Mandatory)][string[]]$Paths)
    $items = [Collections.Generic.List[object]]::new()
    foreach ($relative in @($Paths | Sort-Object -Unique)) {
        $full = Join-Path $script:Sprint8BRepositoryRoot $relative
        if (-not (Test-Path -LiteralPath $full -PathType Leaf)) {
            throw "Candidate input is missing: $relative"
        }
        $items.Add([ordered]@{
            path = $relative.Replace("\", "/")
            sha256 = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLowerInvariant()
        })
    }
    return @($items)
}

function New-Sprint8BCandidate {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Source
    )
    $readiness = Assert-Sprint8BPhaseCertificateForContract -Contract $Contract -Phase "validation-readiness" -Source $Source
    $rehearsal = Assert-Sprint8BPhaseCertificateForContract -Contract $Contract -Phase "candidate-rehearsal" -Source $Source
    $implementationPath = Get-Sprint8BImplementationResultPath -Contract $Contract
    $implementation = Read-Sprint8BAuthenticatedJson -Path $implementationPath -RequireSidecar
    $null = Assert-TessaraImplementationReadinessResult -Result $implementation -Contract $Contract -ContractPath $script:Sprint8BContractPath
    Assert-Sprint8BSourceMatches -Expected $Source -Actual $implementation.source_identity -Label "Frozen implementation prerequisite"

    $acceptanceInputs = Get-Sprint8BTrackedInventory -Paths @(
        "end2end/acceptance-manifest.json",
        "docs/sprints/sprint-8b-test-change-log.md",
        "docs/sprints/sprint-8b-uat/scenario-contract.json",
        "docs/audits/sprint-8b-dataset-ui-baseline/baseline-index.json"
    )
    $deploymentPaths = @(
        "deploy/sprint-8b/compose.yaml",
        "deploy/sprint-8b/compose.override.yaml",
        "deploy/sprint-8b/catalogs/local-release-catalog.json",
        "deploy/sprint-8b/fixtures/reference-fixture-contract.json",
        "deploy/sprint-8b/fixtures/provider-fault-contract.json",
        "deploy/sprint-8b/fixtures/upgrade-fixture-contract.json",
        "crates/tessara-dataset-module/manifest.json"
    )
    $deploymentInputs = Get-Sprint8BTrackedInventory -Paths $deploymentPaths
    $payload = [ordered]@{
        source_identity = $Source
        validation_contract = Get-Sprint8BFileReference -Path $script:Sprint8BContractPath
        implementation_readiness = Get-Sprint8BFileReference -Path $implementationPath
        readiness_certificate = $readiness.reference
        rehearsal_certificate = $rehearsal.reference
        acceptance_inputs = $acceptanceInputs
        deployment_inputs = $deploymentInputs
    }
    $fingerprint = Get-Sprint8BSha256Text -Text (($payload | ConvertTo-Json -Depth 100 -Compress) + "`n")
    $candidate = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.candidate"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8b"
        frozen_at = [DateTimeOffset]::UtcNow.ToString("O")
        source_identity = $Source
        validation_contract = $payload.validation_contract
        implementation_readiness = $payload.implementation_readiness
        readiness_certificate = $payload.readiness_certificate
        rehearsal_certificate = $payload.rehearsal_certificate
        acceptance_inputs = $acceptanceInputs
        deployment_inputs = $deploymentInputs
        candidate_fingerprint = $fingerprint
    }
    $path = Get-Sprint8BCandidatePath -Contract $Contract
    $null = Publish-Sprint8BJsonAndSidecar -Path $path -Document $candidate
    return [pscustomobject]@{ document = $candidate; reference = Get-Sprint8BFileReference -Path $path }
}

function Invoke-Sprint8BAction {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)]$Action,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [AllowNull()][string]$Project,
        [AllowNull()][string]$CandidateFingerprint,
        [AllowNull()]$TopologyContext,
        [Parameter(Mandatory)]$Source,
        [switch]$SuppressChildOutput
    )
    $actionsRoot = Join-Path $AttemptRoot "actions"
    [IO.Directory]::CreateDirectory($actionsRoot) | Out-Null
    $actionResultPath = Join-Path $actionsRoot "$($Action.id).json"
    $harnessEvidencePath = Join-Path $actionsRoot "$($Action.id)-evidence.json"
    $started = [DateTimeOffset]::UtcNow
    $references = [Collections.Generic.List[object]]::new()
    $harnessDocument = $null
    $harnessEvidenceReference = $null
    $commandText = $null
    $resultTopologyContext = $TopologyContext
    $topologyRemoved = $false

    switch ([string]$Action.kind) {
        "pwsh" {
            $scriptPath = Join-Path $script:Sprint8BRepositoryRoot ([string]$Action.command)
            $arguments = @(Expand-Sprint8BActionArguments -Arguments @($Action.arguments) -Project $Project `
                -EvidencePath $harnessEvidencePath -CandidateFingerprint $CandidateFingerprint `
                -TopologyContext $TopologyContext)
            $commandText = "pwsh -NoProfile -File $($Action.command) $($arguments -join ' ')"
            if ($SuppressChildOutput) {
                & pwsh -NoProfile -File $scriptPath @arguments | Out-Null
            } else {
                & pwsh -NoProfile -File $scriptPath @arguments | Out-Host
            }
            $exitCode = $LASTEXITCODE
            if ($exitCode -ne 0) { throw "Action '$($Action.id)' exited $exitCode." }
            if ([bool]$Action.produces_evidence) {
                $harnessDocument = Assert-Sprint8BHarnessEvidence -Path $harnessEvidencePath -Action $Action -ExpectedProject $Project
                $harnessEvidenceReference = Get-Sprint8BFileReference -Path $harnessEvidencePath
                $references.Add($harnessEvidenceReference)
                if ($harnessDocument.PSObject.Properties.Name -contains "cleanup_restoration" -and
                    $harnessDocument.cleanup_restoration.PSObject.Properties.Name -contains "mode" -and
                    [string]$harnessDocument.cleanup_restoration.mode -ceq "retained-for-caller") {
                    $resultTopologyContext = Get-Sprint8BTopologyContextFromHarness -Document $harnessDocument `
                        -ExpectedProject $Project -CandidateFingerprint $CandidateFingerprint
                }
            }
        }
        "program" {
            $arguments = @(Expand-Sprint8BActionArguments -Arguments @($Action.arguments) -Project $Project `
                -EvidencePath $harnessEvidencePath -CandidateFingerprint $CandidateFingerprint `
                -TopologyContext $TopologyContext)
            $commandText = "$($Action.command) $($arguments -join ' ')"
            $expectedDataState = [string]$Action.playwright_data_state
            $dataStateBefore = [Environment]::GetEnvironmentVariable(
                "TESSARA_PLAYWRIGHT_DATA_STATE", "Process"
            )
            try {
                if (-not [string]::IsNullOrWhiteSpace($expectedDataState)) {
                    if ($null -eq $TopologyContext -or
                        [Environment]::GetEnvironmentVariable(
                            "TESSARA_PLAYWRIGHT_ACCEPTANCE", "Process"
                        ) -cne "1") {
                        throw "Playwright action '$($Action.id)' lacks an authenticated retained topology/acceptance binding."
                    }
                    [Environment]::SetEnvironmentVariable(
                        "TESSARA_PLAYWRIGHT_DATA_STATE", $expectedDataState, "Process"
                    )
                }
                if ($SuppressChildOutput) {
                    & ([string]$Action.command) @arguments | Out-Null
                } else {
                    & ([string]$Action.command) @arguments | Out-Host
                }
                $exitCode = $LASTEXITCODE
            } finally {
                [Environment]::SetEnvironmentVariable(
                    "TESSARA_PLAYWRIGHT_DATA_STATE", $dataStateBefore, "Process"
                )
            }
            if ($exitCode -ne 0) { throw "Action '$($Action.id)' exited $exitCode." }
        }
        "teardown" {
            $commandText = "docker compose -f deploy/sprint-8b/compose.yaml -p $Project --profile reference down --volumes --remove-orphans"
            $harnessDocument = Invoke-Sprint8BComposeTeardown -Project $Project -EvidencePath $harnessEvidencePath
            $harnessEvidenceReference = Get-Sprint8BFileReference -Path $harnessEvidencePath
            $references.Add($harnessEvidenceReference)
            $resultTopologyContext = $null
            $topologyRemoved = $true
        }
        "manual" {
            $commandText = "authenticate manual UAT evidence: $(@($Action.scenarios) -join ', ')"
            foreach ($reference in @(Assert-Sprint8BManualScenarioEvidence -Contract $Contract -Scenarios @($Action.scenarios) -CandidateFingerprint $CandidateFingerprint)) {
                $references.Add($reference)
            }
        }
        "internal" {
            $commandText = [string]$Action.command
            switch ([string]$Action.command) {
                "implementation-gate" {
                    foreach ($reference in @(Assert-Sprint8BImplementationGate -Contract $Contract -Source $Source)) {
                        $references.Add($reference)
                    }
                }
                "freeze-candidate" {
                    $candidate = New-Sprint8BCandidate -Contract $Contract -Source $Source
                    $references.Add($candidate.reference)
                    $harnessDocument = $candidate.document
                }
                default { throw "Unknown internal action '$($Action.command)'." }
            }
        }
        default { throw "Unknown action kind '$($Action.kind)'." }
    }

    $completed = [DateTimeOffset]::UtcNow
    $result = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-action-result"
        sprint = "sprint-8b"
        action = [string]$Action.id
        state = "passed"
        command = $commandText
        started_at = $started.ToString("O")
        completed_at = $completed.ToString("O")
        duration_ms = [long]($completed - $started).TotalMilliseconds
        playwright_data_state = if ([string]::IsNullOrWhiteSpace(
            [string]$Action.playwright_data_state
        )) { $null } else { [string]$Action.playwright_data_state }
        evidence_references = @($references)
    }
    Write-Sprint8BNewJsonFile -Path $actionResultPath -Document $result
    $references.Add((Get-Sprint8BFileReference -Path $actionResultPath))

    $environmentFingerprint = $null
    $restorationPassed = $false
    $restorationReference = $null
    if ($null -ne $harnessDocument) {
        $environmentFingerprint = Get-Sprint8BEnvironmentFingerprintFromDocument -Document $harnessDocument
        if (($Action.kind -ceq "teardown" -or [bool]$Action.provides_restoration) -and
            $harnessDocument.PSObject.Properties.Name -contains "cleanup_restoration") {
            $restorationPassed = [string]$harnessDocument.cleanup_restoration.state -ceq "passed"
            if ($restorationPassed) { $restorationReference = $harnessEvidenceReference }
        }
    }

    return [pscustomobject][ordered]@{
        result_reference = Get-Sprint8BFileReference -Path $actionResultPath
        evidence_references = @($references)
        environment_fingerprint = $environmentFingerprint
        restoration_passed = $restorationPassed
        restoration_reference = $restorationReference
        topology_context = $resultTopologyContext
        topology_removed = $topologyRemoved
    }
}

function Get-Sprint8BPhasePrerequisiteReferences {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase
    )
    switch ($Phase) {
        "validation-readiness" {
            @(
                Get-Sprint8BFileReference -Path (Get-Sprint8BImplementationResultPath -Contract $Contract)
                Get-Sprint8BFileReference -Path (Get-Sprint8BImpactPath -Contract $Contract)
            )
        }
        "candidate-rehearsal" { @((Get-Sprint8BFileReference -Path (Get-Sprint8BCertificatePath -Contract $Contract -Phase "validation-readiness"))) }
        "validation-preflight" {
            @(
                Get-Sprint8BFileReference -Path (Get-Sprint8BCertificatePath -Contract $Contract -Phase "validation-readiness")
                Get-Sprint8BFileReference -Path (Get-Sprint8BCertificatePath -Contract $Contract -Phase "candidate-rehearsal")
            )
        }
        "sit" { @((Get-Sprint8BFileReference -Path (Get-Sprint8BCertificatePath -Contract $Contract -Phase "validation-preflight"))) }
        "uat" { @((Get-Sprint8BFileReference -Path (Get-Sprint8BCertificatePath -Contract $Contract -Phase "sit"))) }
    }
}

function Complete-Sprint8BPhase {
    param(
        [Parameter(Mandatory)]$Contract,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)]$Source,
        [AllowNull()][string]$CandidateFingerprint
    )
    $phaseRoot = Get-Sprint8BPhaseRoot -Contract $Contract -Phase $Phase
    $certificatePath = Get-Sprint8BCertificatePath -Contract $Contract -Phase $Phase
    if (Test-Path -LiteralPath $certificatePath) {
        throw "$Phase certificate already exists and will not be overwritten."
    }
    $laneIds = @(Get-Sprint8BExpectedLaneIds -Phase $Phase)
    $laneDocuments = [Collections.Generic.List[object]]::new()
    $environmentFingerprints = [Collections.Generic.List[string]]::new()
    foreach ($laneId in $laneIds) {
        $path = (Get-Sprint8BLaneEvidencePaths -Contract $Contract -Phase $Phase -Lane $laneId).result
        $document = Read-Sprint8BAuthenticatedJson -Path $path -RequireSidecar
        if ([string]$document.contract -cne "tessara.validation.lane-result" -or
            [string]$document.lane -cne $laneId -or [string]$document.state -cne "passed") {
            throw "$Phase cannot finalize because lane '$laneId' is not passed."
        }
        Assert-Sprint8BSourceMatches -Expected $Source -Actual $document.source_identity -Label "$laneId result"
        if (-not [string]::IsNullOrWhiteSpace($CandidateFingerprint) -and
            [string]$document.candidate_fingerprint -cne $CandidateFingerprint) {
            throw "$laneId is not bound to the frozen candidate."
        }
        if ([string]$document.environment.fingerprint -cnotmatch '^[0-9a-f]{64}$') {
            throw "$laneId has no authenticated environment fingerprint."
        }
        $environmentFingerprints.Add([string]$document.environment.fingerprint)
        $laneDocuments.Add($document)
    }

    $indexPath = Join-Path $phaseRoot "evidence-index.json"
    $excluded = @(
        [IO.Path]::GetFullPath($indexPath),
        [IO.Path]::GetFullPath("$indexPath.sha256"),
        [IO.Path]::GetFullPath($certificatePath),
        [IO.Path]::GetFullPath("$certificatePath.sha256")
    )
    $entries = @(
        Get-ChildItem -LiteralPath $phaseRoot -File -Recurse | Where-Object {
            [IO.Path]::GetFullPath($_.FullName) -notin $excluded
        } | Sort-Object FullName | ForEach-Object {
            [ordered]@{
                path = Get-Sprint8BRepositoryRelativePath -Path $_.FullName
                sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                size = [long]$_.Length
                kind = if ($_.Name -eq "command.log") { "log" } elseif ($_.Name -match 'result|receipt') { "receipt" } else { "structured" }
            }
        }
    )
    $index = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.phase-evidence-index"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8b"
        phase = $Phase
        attempt = 1
        evidence_root = Get-Sprint8BRepositoryRelativePath -Path $phaseRoot
        sealed_at = [DateTimeOffset]::UtcNow.ToString("O")
        entry_count = $entries.Count
        entries = $entries
    }
    $null = Assert-TessaraPhaseEvidenceIndex -Index $index
    $indexSha = Publish-Sprint8BJsonAndSidecar -Path $indexPath -Document $index

    $phaseDomains = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase } | ForEach-Object { @($_.dependency_domains) } | ForEach-Object { [string]$_ } | Sort-Object -Unique)
    $allFingerprints = @(Get-TessaraDependencyFingerprints -Contract $Contract -RepositoryRoot $script:Sprint8BRepositoryRoot)
    $fingerprints = @($allFingerprints | Where-Object { [string]$_.domain -in $phaseDomains } | Sort-Object domain)
    $declaredDigest = Get-Sprint8BSha256Text -Text (($laneIds -join "`n") + "`n")
    $environmentDigest = Get-Sprint8BSha256Text -Text ((@($environmentFingerprints) -join "`n") + "`n")
    $touchesLive = @($Contract.lanes | Where-Object { [string]$_.phase -ceq $Phase -and [bool]$_.touches_live_state }).Count -gt 0
    $lastLive = @($laneDocuments | Where-Object { [bool]$_.environment.touches_live_state }) | Select-Object -Last 1
    $cleanupReference = if ($touchesLive) {
        if ($null -eq $lastLive -or [string]$lastLive.cleanup_restoration.state -cne "passed") {
            throw "$Phase cannot finalize without passing canonical restoration."
        }
        Get-Sprint8BFileReference -Path (Get-Sprint8BLaneResultPath -Contract $Contract -LaneId ([string]$lastLive.lane))
    } else { $null }
    $authoritative = $Phase -in @("validation-preflight", "sit", "uat")
    $laneSummaries = @($laneDocuments | ForEach-Object {
        [ordered]@{
            name = [string]$_.lane
            state = "passed"
            certification_basis = "executed"
            dependency_domains = @($_.dependency_domains)
            receipt = Get-Sprint8BFileReference -Path (Get-Sprint8BLaneResultPath -Contract $Contract -LaneId ([string]$_.lane))
            started_at = [string]$_.started_at
            ended_at = [string]$_.completed_at
            duration_ms = [long]$_.duration_ms
            inheritance = $null
        }
    })
    $certificate = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.phase-certificate"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8b"
        phase = $Phase
        attempt = 1
        state = "passed"
        authoritative = $authoritative
        certified_at = [DateTimeOffset]::UtcNow.ToString("O")
        source_identity = $Source
        environment_fingerprint = $environmentDigest
        candidate_fingerprint = if ($authoritative) { $CandidateFingerprint } else { $null }
        prerequisite_certificates = @(Get-Sprint8BPhasePrerequisiteReferences -Contract $Contract -Phase $Phase)
        dependency_fingerprints = $fingerprints
        coverage = [ordered]@{
            declared_lanes_sha256 = $declaredDigest
            lane_count = $laneIds.Count
            executed_count = $laneIds.Count
            inherited_count = 0
        }
        lanes = $laneSummaries
        open_defect_count = 0
        cleanup_restoration = [ordered]@{
            required = $touchesLive
            state = if ($touchesLive) { "passed" } else { "not_applicable" }
            evidence = $cleanupReference
        }
        evidence_index = [ordered]@{
            path = Get-Sprint8BRepositoryRelativePath -Path $indexPath
            sha256 = $indexSha
        }
    }
    $null = Assert-TessaraPhaseCertificate -Certificate $certificate
    $null = Publish-Sprint8BJsonAndSidecar -Path $certificatePath -Document $certificate
    return $certificate
}

function Invoke-Sprint8BFormalLane {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Lane
    )
    $contract = Get-Sprint8BContract
    $actionMap = Get-Sprint8BFormalActionMap
    $null = Assert-Sprint8BFormalProfile -Contract $contract -Phase $Phase -ActionMap $actionMap
    $laneContract = Get-Sprint8BLaneContract -Contract $contract -Phase $Phase -Lane $Lane
    $actions = @($actionMap[$Lane])
    Assert-Sprint8BActionAvailability -Actions $actions

    $sourceBefore = Get-Sprint8BSourceIdentity
    Assert-Sprint8BCleanSource -Source $sourceBefore
    $prerequisites = Assert-Sprint8BLanePrerequisites -Contract $contract -LaneContract $laneContract -Source $sourceBefore
    $project = Get-Sprint8BExpectedProject -Phase $Phase -LaneContract $laneContract
    $topologyContext = $prerequisites.topology_context
    if ([bool]$laneContract.touches_live_state -and
        [Environment]::GetEnvironmentVariable("TESSARA_SPRINT_8B_DISPOSABLE_RESET_AUTHORIZATION", "Process") -cne $script:Sprint8BResetAuthorization) {
        throw "Live lane '$Lane' requires the exact process-scoped TESSARA_SPRINT_8B_DISPOSABLE_RESET_AUTHORIZATION acknowledgement."
    }

    $paths = Get-Sprint8BLaneEvidencePaths -Contract $contract -Phase $Phase -Lane $Lane
    if (Test-Path -LiteralPath $paths.result -PathType Leaf) {
        throw "Lane '$Lane' already has a canonical result and will not be overwritten."
    }
    [IO.Directory]::CreateDirectory($paths.lane_root) | Out-Null
    $attemptId = "{0}-{1}" -f [DateTimeOffset]::UtcNow.ToString("yyyyMMddTHHmmssfffZ"), ([guid]::NewGuid().ToString("N").Substring(0, 8))
    $attemptRoot = Join-Path $paths.lane_root "attempts/$attemptId"
    [IO.Directory]::CreateDirectory($attemptRoot) | Out-Null
    $attemptLog = Join-Path $attemptRoot "command.log"
    $started = [DateTimeOffset]::UtcNow
    $actionResults = [Collections.Generic.List[object]]::new()
    $references = [Collections.Generic.List[object]]::new()
    foreach ($reference in @($prerequisites.references)) { $references.Add($reference) }
    $environmentFingerprints = [Collections.Generic.List[string]]::new()
    $restorationPassed = -not [bool]$laneContract.touches_live_state
    $restorationReference = $null
    $environmentNames = @(
        "COMPOSE_PROJECT_NAME", "TESSARA_GATEWAY_PORT", "TESSARA_CORE_CONTROL_PORT",
        "TESSARA_SUPERVISOR_PORT", "PLAYWRIGHT_BASE_URL", "TESSARA_PLAYWRIGHT_ACCEPTANCE",
        "TESSARA_PLAYWRIGHT_DATA_STATE"
    )
    $environmentBefore = Get-Sprint8BProcessEnvironmentSnapshot -Names $environmentNames
    $transcribing = $false
    $pushed = $false
    try {
        if ($null -ne $topologyContext) {
            Set-Sprint8BTopologyEnvironment -Context $topologyContext -ExpectedProject $project `
                -CandidateFingerprint ([string]$prerequisites.candidate_fingerprint)
        }
        Start-Transcript -LiteralPath $attemptLog -Force | Out-Null
        $transcribing = $true
        Push-Location $script:Sprint8BRepositoryRoot
        $pushed = $true
        foreach ($action in $actions) {
            $actionResult = Invoke-Sprint8BAction -Contract $contract -Action $action -AttemptRoot $attemptRoot `
                -Project $project -CandidateFingerprint ([string]$prerequisites.candidate_fingerprint) `
                -TopologyContext $topologyContext -Source $sourceBefore
            $actionResults.Add($actionResult.result_reference)
            foreach ($reference in @($actionResult.evidence_references)) { $references.Add($reference) }
            if ([string]$actionResult.environment_fingerprint -match '^[0-9a-f]{64}$') {
                $environmentFingerprints.Add([string]$actionResult.environment_fingerprint)
            }
            if ([bool]$actionResult.restoration_passed) { $restorationPassed = $true }
            if ($null -ne $actionResult.restoration_reference) {
                $restorationReference = $actionResult.restoration_reference
            }
            $topologyContext = $actionResult.topology_context
            if ($null -ne $topologyContext) {
                Set-Sprint8BTopologyEnvironment -Context $topologyContext -ExpectedProject $project `
                    -CandidateFingerprint ([string]$prerequisites.candidate_fingerprint)
            }
        }
    } catch {
        $laneError = $_
        $emergencyCleanup = $null
        if ([bool]$laneContract.touches_live_state -and $null -ne $topologyContext) {
            try {
                $emergencyPath = Join-Path $attemptRoot "emergency-teardown.json"
                $null = Invoke-Sprint8BComposeTeardown -Project $project -EvidencePath $emergencyPath
                $emergencyCleanup = [ordered]@{
                    state = "passed"
                    evidence = Get-Sprint8BFileReference -Path $emergencyPath
                }
                $topologyContext = $null
            } catch {
                $emergencyCleanup = [ordered]@{ state = "failed"; message = $_.Exception.Message }
            }
        }
        $failure = [ordered]@{
            schema_version = 1
            contract = "tessara.validation.lane-attempt-failure"
            sprint = "sprint-8b"
            phase = $Phase
            lane = $Lane
            state = "failed"
            failed_at = [DateTimeOffset]::UtcNow.ToString("O")
            message = $laneError.Exception.Message
            emergency_cleanup = $emergencyCleanup
        }
        Write-Sprint8BNewJsonFile -Path (Join-Path $attemptRoot "failure.json") -Document $failure
        if ($null -ne $emergencyCleanup -and [string]$emergencyCleanup.state -ceq "failed") {
            throw "Formal lane '$Lane' failed: $($laneError.Exception.Message) Emergency cleanup also failed: $([string]$emergencyCleanup.message)"
        }
        throw $laneError
    } finally {
        if ($pushed) { Pop-Location }
        if ($transcribing) { Stop-Transcript | Out-Null }
        Restore-Sprint8BProcessEnvironmentSnapshot -Snapshot $environmentBefore
    }

    $sourceAfter = Get-Sprint8BSourceIdentity
    Assert-Sprint8BCleanSource -Source $sourceAfter
    Assert-Sprint8BSourceMatches -Expected $sourceBefore -Actual $sourceAfter -Label "Formal lane '$Lane'"
    if ([bool]$laneContract.touches_live_state -and $environmentFingerprints.Count -eq 0) {
        throw "Live lane '$Lane' did not retain an authenticated environment fingerprint."
    }
    if ([bool]$laneContract.touches_live_state -and -not $restorationPassed) {
        throw "Live lane '$Lane' did not prove cleanup/canonical restoration."
    }
    if ($Phase -ceq "sit" -and $Lane -in @("sit-rust", "sit-browser") -and $null -eq $topologyContext) {
        throw "SIT lane '$Lane' did not retain its authenticated frozen topology context."
    }
    if ($Phase -ceq "sit" -and $Lane -ceq "sit-smoke" -and $null -ne $topologyContext) {
        throw "Final SIT smoke did not remove its retained frozen topology."
    }
    $completed = [DateTimeOffset]::UtcNow
    $environmentFingerprint = if ([bool]$laneContract.touches_live_state) {
        Get-Sprint8BSha256Text -Text ((@($environmentFingerprints) -join "`n") + "`n")
    } else {
        Get-Sprint8BSha256Text -Text "offline`n$Phase`n$Lane`n$($sourceAfter.tree)`n"
    }
    $referenceDocument = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-evidence-references"
        sprint = "sprint-8b"
        phase = $Phase
        lane = $Lane
        references = @($references | Sort-Object path, sha256 -Unique)
    }
    $attemptReferences = Join-Path $attemptRoot "evidence-references.json"
    Write-Sprint8BNewJsonFile -Path $attemptReferences -Document $referenceDocument
    $result = [ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-result"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-8b"
        phase = $Phase
        lane = $Lane
        state = "passed"
        selector = Get-Sprint8BSelector -Phase $Phase -Lane $Lane
        source_identity = $sourceAfter
        candidate_fingerprint = if ($Phase -in @("sit", "uat")) { [string]$prerequisites.candidate_fingerprint } elseif ($Phase -ceq "validation-preflight") {
            $candidate = Read-Sprint8BCandidate -Contract $contract -Source $sourceAfter
            [string]$candidate.candidate_fingerprint
        } else { $null }
        dependency_domains = @($laneContract.dependency_domains)
        prerequisites = @($prerequisites.references)
        environment = [ordered]@{
            kind = if ([bool]$laneContract.touches_live_state) { if ($Phase -ceq "sit") { "frozen-sit" } else { "isolated-live" } } else { "offline" }
            touches_live_state = [bool]$laneContract.touches_live_state
            compose_project = $project
            fingerprint = $environmentFingerprint
        }
        assertions = @($actions.id)
        action_results = @($actionResults)
        started_at = $started.ToString("O")
        completed_at = $completed.ToString("O")
        duration_ms = [long]($completed - $started).TotalMilliseconds
        cleanup_restoration = [ordered]@{
            required = [bool]$laneContract.touches_live_state
            state = if ([bool]$laneContract.touches_live_state) { "passed" } else { "not_applicable" }
            evidence = if ([bool]$laneContract.touches_live_state) { $restorationReference } else { $null }
        }
        topology_context = if ($Phase -ceq "sit" -and $Lane -in @("sit-rust", "sit-browser")) { $topologyContext } else { $null }
        command_log = $null
        evidence_references = $null
    }

    $published = [Collections.Generic.List[string]]::new()
    try {
        Copy-Sprint8BNewFile -Source $attemptLog -Destination $paths.command_log
        $published.Add($paths.command_log)
        Copy-Sprint8BNewFile -Source $attemptReferences -Destination $paths.evidence_references
        $published.Add($paths.evidence_references)
        $result.command_log = Get-Sprint8BFileReference -Path $paths.command_log
        $result.evidence_references = Get-Sprint8BFileReference -Path $paths.evidence_references
        $null = Publish-Sprint8BJsonAndSidecar -Path $paths.result -Document $result
        $published.Add($paths.result)
        $published.Add("$($paths.result).sha256")
    } catch {
        foreach ($path in @($published | Sort-Object -Descending)) {
            if (Test-Path -LiteralPath $path -PathType Leaf) { Remove-Item -LiteralPath $path -Force }
        }
        throw
    }

    $phaseLaneIds = @(Get-Sprint8BExpectedLaneIds -Phase $Phase)
    if ($Lane -ceq $phaseLaneIds[-1]) {
        $candidateFingerprint = if ($Phase -in @("validation-preflight", "sit", "uat")) {
            [string]$result.candidate_fingerprint
        } else { $null }
        $null = Complete-Sprint8BPhase -Contract $contract -Phase $Phase -Source $sourceAfter -CandidateFingerprint $candidateFingerprint
    }

    return $result
}

function Assert-Sprint8BExpectedFailure {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$Label
    )
    try { & $Action } catch { return }
    throw "Formal runner self-test expected rejection: $Label"
}

function Test-Sprint8BFormalRunner {
    param([Parameter(Mandatory)][string]$Phase)
    $contract = Get-Sprint8BContract
    $actionMap = Get-Sprint8BFormalActionMap
    $null = Assert-Sprint8BFormalProfile -Contract $contract -Phase $Phase -ActionMap $actionMap
    $expectedIds = @(Get-Sprint8BExpectedLaneIds -Phase $Phase)

    $mutatedIdentity = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $identityLane = @($mutatedIdentity.lanes | Where-Object { [string]$_.phase -ceq $Phase })[0]
    $identityLane.id = "$([string]$identityLane.id)-wrong"
    Assert-Sprint8BExpectedFailure -Label "$Phase lane identity" -Action {
        Assert-Sprint8BFormalProfile -Contract $mutatedIdentity -Phase $Phase -ActionMap $actionMap
    }

    $mutatedOrder = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $phaseIndexes = @(for ($index = 0; $index -lt $mutatedOrder.lanes.Count; $index++) {
        if ([string]$mutatedOrder.lanes[$index].phase -ceq $Phase) { $index }
    })
    if ($phaseIndexes.Count -gt 1) {
        $first = $mutatedOrder.lanes[$phaseIndexes[0]]
        $mutatedOrder.lanes[$phaseIndexes[0]] = $mutatedOrder.lanes[$phaseIndexes[1]]
        $mutatedOrder.lanes[$phaseIndexes[1]] = $first
        Assert-Sprint8BExpectedFailure -Label "$Phase reordered lane" -Action {
            Assert-Sprint8BFormalProfile -Contract $mutatedOrder -Phase $Phase -ActionMap $actionMap
        }
    }

    $mutatedPrerequisite = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $targetLane = @($mutatedPrerequisite.lanes | Where-Object { [string]$_.id -ceq $expectedIds[-1] })[0]
    $targetLane.prerequisites = @()
    Assert-Sprint8BExpectedFailure -Label "$Phase prerequisite removal" -Action {
        Assert-Sprint8BFormalProfile -Contract $mutatedPrerequisite -Phase $Phase -ActionMap $actionMap
    }

    $mutatedEnvironment = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $environmentLane = @($mutatedEnvironment.lanes | Where-Object { [string]$_.phase -ceq $Phase })[0]
    $environmentLane.touches_live_state = -not [bool]$environmentLane.touches_live_state
    Assert-Sprint8BExpectedFailure -Label "$Phase environment identity" -Action {
        Assert-Sprint8BFormalProfile -Contract $mutatedEnvironment -Phase $Phase -ActionMap $actionMap
    }

    $mutatedEvidence = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $mutatedEvidence.evidence_policy.root = "artifacts/not-sprint-8b-closeout"
    Assert-Sprint8BExpectedFailure -Label "$Phase evidence root" -Action {
        Assert-Sprint8BFormalProfile -Contract $mutatedEvidence -Phase $Phase -ActionMap $actionMap
    }

    $browserLaneId = if ($Phase -ceq "candidate-rehearsal") {
        "rehearsal-browser"
    } elseif ($Phase -ceq "sit") {
        "sit-browser"
    } else { $null }
    if ($null -ne $browserLaneId) {
        $browserAction = @($actionMap[$browserLaneId] | Where-Object {
            [string]$_.id -ceq "browser-acceptance"
        })
        if ($browserAction.Count -ne 1 -or
            [string]$browserAction[0].playwright_data_state -cne "fresh") {
            throw "$browserLaneId does not bind its browser action to fresh reference data."
        }
        $browserAction[0].playwright_data_state = ""
        try {
            Assert-Sprint8BExpectedFailure -Label "$Phase missing Playwright data state" -Action {
                Assert-Sprint8BFormalProfile -Contract $contract -Phase $Phase -ActionMap $actionMap
            }
        } finally {
            $browserAction[0].playwright_data_state = "fresh"
        }
    }

    $firstLane = Get-Sprint8BLaneContract -Contract $contract -Phase $Phase -Lane $expectedIds[0]
    $contractCopy = $contract | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $contractCopy.evidence_policy.root = "tmp/formal-selftest-$([guid]::NewGuid().ToString('N'))"
    $source = [pscustomobject]@{ commit = "1" * 40; tree = "2" * 40; dirty = $false; branch = "codex/selftest" }
    Assert-Sprint8BExpectedFailure -Label "$Phase missing prerequisite" -Action {
        Assert-Sprint8BLanePrerequisites -Contract $contractCopy -LaneContract $firstLane -Source $source
    }

    $missingHarness = @(New-Sprint8BPowerShellAction -Id "missing" -Script "scripts/definitely-missing-sprint-8b-harness.ps1")
    Assert-Sprint8BExpectedFailure -Label "$Phase missing harness" -Action {
        Assert-Sprint8BActionAvailability -Actions $missingHarness
    }

    $emptyArguments = @(Expand-Sprint8BActionArguments -Arguments @() -Project $null `
        -EvidencePath "tmp/formal-selftest-evidence.json" -CandidateFingerprint $null -TopologyContext $null)
    if ($emptyArguments.Count -ne 0) {
        throw "$Phase zero-argument action expansion produced unexpected arguments."
    }

    $stdoutAttemptRoot = Join-Path $script:Sprint8BRepositoryRoot `
        "tmp/formal-action-stdout-$([guid]::NewGuid().ToString('N'))"
    try {
        $stdoutAction = New-Sprint8BProgramAction -Id "stdout" -Program "pwsh" `
            -Arguments @("-NoProfile", "-Command", "Write-Output 'child-output'")
        $stdoutResult = @(Invoke-Sprint8BAction -Contract $contract -Action $stdoutAction `
            -AttemptRoot $stdoutAttemptRoot -Project $null -CandidateFingerprint $null `
            -TopologyContext $null -Source $source -SuppressChildOutput)
        if ($stdoutResult.Count -ne 1 -or
            -not ($stdoutResult[0].PSObject.Properties.Name -contains "result_reference")) {
            throw "$Phase child stdout contaminated the typed action result."
        }
    } finally {
        if (Test-Path -LiteralPath $stdoutAttemptRoot) {
            Remove-Item -LiteralPath $stdoutAttemptRoot -Recurse -Force
        }
    }

    Assert-Sprint8BExpectedFailure -Label "$Phase unmapped selector" -Action {
        Invoke-Sprint8BPhaseRunner -Phase $Phase -Lane "not-a-sprint-8b-lane"
    }
    Assert-Sprint8BExpectedFailure -Label "$Phase ambiguous selector mode" -Action {
        Invoke-Sprint8BPhaseRunner -Phase $Phase -ListLanes -SelfTest
    }

    $phaseLanes = @($contract.lanes | Where-Object { [string]$_.phase -ceq $Phase })
    foreach ($lane in $phaseLanes) {
        $expectedProject = Get-Sprint8BExpectedProject -Phase $Phase -LaneContract $lane
        if ([bool]$lane.touches_live_state -and $expectedProject -cnotmatch '^tessara-s8b-[a-z0-9-]+$') {
            throw "Self-test found unsafe project identity for '$($lane.id)'."
        }
        $paths = Get-Sprint8BLaneEvidencePaths -Contract $contract -Phase $Phase -Lane ([string]$lane.id)
        if ((Get-Sprint8BRepositoryRelativePath -Path $paths.result) -cne
            "$([string]$contract.evidence_policy.root)/$Phase/lanes/$([string]$lane.id)/result.json") {
            throw "Self-test found an incorrect result path for '$($lane.id)'."
        }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8b.formal-runner-selftest"
        sprint = "sprint-8b"
        phase = $Phase
        state = "passed"
        lane_count = $expectedIds.Count
        lanes = $expectedIds
        verified = @(
            "identity", "order", "prerequisites", "environment", "evidence-mapping",
            "playwright-data-state", "missing-prerequisite", "missing-harness",
            "zero-argument-action", "typed-action-result", "unmapped-selector", "exclusive-mode"
        )
    }
}

function Invoke-Sprint8BPhaseRunner {
    param(
        [Parameter(Mandatory)][string]$Phase,
        [string]$Lane,
        [switch]$ListLanes,
        [switch]$SelfTest
    )
    $expected = @(Get-Sprint8BExpectedLaneIds -Phase $Phase)
    if ($ListLanes -and $SelfTest) {
        throw "-ListLanes and -SelfTest are mutually exclusive."
    }
    if (($ListLanes -or $SelfTest) -and -not [string]::IsNullOrWhiteSpace($Lane)) {
        throw "-Lane cannot be combined with -ListLanes or -SelfTest."
    }
    if ($ListLanes) { return $expected }
    if ($SelfTest) { return Test-Sprint8BFormalRunner -Phase $Phase }
    if ($Phase -ceq "validation-preflight") {
        if (-not [string]::IsNullOrWhiteSpace($Lane) -and $Lane -cne "preflight-freeze") {
            throw "Preflight has exactly one implicit lane: preflight-freeze."
        }
        $Lane = "preflight-freeze"
    } elseif ([string]::IsNullOrWhiteSpace($Lane)) {
        throw "-Lane is required. Select exactly one of: $($expected -join ', ')."
    }
    if ($Lane -cnotin $expected) { throw "Unknown $Phase lane '$Lane'." }
    return Invoke-Sprint8BFormalLane -Phase $Phase -Lane $Lane
}
